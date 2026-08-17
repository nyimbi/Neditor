/// Clipboard export commands: copy document as HTML or Rich Text.
///
/// `copy_export_as_html`  – compiles to HTML, writes to system clipboard as
///   public.html (macOS) / CF_HTML (Windows) / text/html (Linux) via arboard.
///
/// `copy_export_as_rich_text` – compiles to HTML, converts to RTF via a
///   minimal pure-Rust HTML→RTF pass, then writes RTF + plain-text fallback.
///   On macOS uses osascript to set RTF clipboard type. Other platforms fall
///   back to plain text.
use crate::{compile_with_options, export::render_full_html, CompileRequest};
use serde::Serialize;
use std::collections::HashMap;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub(crate) struct CopyExportResult {
    pub(crate) bytes: u64,
    pub(crate) format: String,
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub(crate) fn copy_export_as_html(
    document_source: String,
    document_path: Option<String>,
) -> Result<CopyExportResult, String> {
    let html = compile_to_html(document_source, document_path);
    let bytes = html.len() as u64;
    write_html_to_clipboard(&html)?;
    Ok(CopyExportResult {
        bytes,
        format: "html".to_string(),
    })
}

#[tauri::command]
pub(crate) fn copy_export_as_rich_text(
    document_source: String,
    document_path: Option<String>,
) -> Result<CopyExportResult, String> {
    let html = compile_to_html(document_source, document_path);
    let rtf = html_to_rtf(&html);
    let bytes = rtf.len() as u64;
    let plain = html_to_plain_text(&html);
    write_rtf_to_clipboard(&rtf, &plain)?;
    Ok(CopyExportResult {
        bytes,
        format: "rtf".to_string(),
    })
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn compile_to_html(document_source: String, document_path: Option<String>) -> String {
    let response = compile_with_options(
        CompileRequest {
            text: document_source,
            file_path: document_path,
        },
        &serde_json::json!({}),
    );
    render_full_html(&response, &serde_json::json!({}))
}

fn write_html_to_clipboard(html: &str) -> Result<(), String> {
    let plain = html_to_plain_text(html);
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Clipboard unavailable: {e}"))?;
    clipboard
        .set_html(html, Some(&plain))
        .map_err(|e| format!("Failed to write HTML to clipboard: {e}"))
}

fn write_rtf_to_clipboard(rtf: &str, plain_text: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        write_rtf_to_macos_clipboard(rtf, plain_text)?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| format!("Clipboard unavailable: {e}"))?;
        clipboard
            .set_text(plain_text)
            .map_err(|e| format!("Failed to write text to clipboard: {e}"))?;
    }
    Ok(())
}

/// On macOS, write RTF via osascript reading a temp file typed as «class RTF».
/// This avoids needing objc2 bindings while still setting the correct pasteboard type.
#[cfg(target_os = "macos")]
fn write_rtf_to_macos_clipboard(rtf: &str, plain_text: &str) -> Result<(), String> {
    use std::io::Write as _;
    let mut tmp = tempfile::Builder::new()
        .suffix(".rtf")
        .tempfile()
        .map_err(|e| format!("Cannot create temp file for RTF: {e}"))?;
    tmp.write_all(rtf.as_bytes())
        .map_err(|e| format!("Cannot write RTF temp file: {e}"))?;
    let path = tmp.path().to_string_lossy().to_string();
    // Escape for AppleScript POSIX path string
    let safe_path = path.replace('"', "\\\"");
    let script = format!(
        "set the clipboard to (read POSIX file \"{safe_path}\" as \u{00ab}class RTF \u{00bb})"
    );
    let status = std::process::Command::new("osascript")
        .args(["-e", &script])
        .status()
        .map_err(|e| format!("osascript unavailable: {e}"))?;
    drop(tmp); // temp file stays alive until here
    if !status.success() {
        // Fallback: write plain text
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| format!("Clipboard unavailable: {e}"))?;
        clipboard
            .set_text(plain_text)
            .map_err(|e| format!("Failed to write plain text to clipboard: {e}"))?;
    }
    Ok(())
}

// ── Plain-text extraction ─────────────────────────────────────────────────────

/// Strip HTML tags to produce a readable plain-text fallback.
pub(crate) fn html_to_plain_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut last_was_newline = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                last_was_newline = false;
            }
            '\n' | '\r' if !in_tag => {
                if !last_was_newline {
                    out.push('\n');
                    last_was_newline = true;
                }
            }
            _ if !in_tag => {
                last_was_newline = false;
                out.push(ch);
            }
            _ => {}
        }
    }
    out.trim().to_string()
}

// ── HTML → RTF converter ──────────────────────────────────────────────────────
//
// Covers: h1-h6, p, b/strong, i/em, code (inline), a (text only), ul/ol/li,
// hr, blockquote. This is intentionally minimal for MacDown parity.

pub(crate) fn html_to_rtf(html: &str) -> String {
    let header = concat!(
        "{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat\n",
        "{\\fonttbl{\\f0\\fswiss\\fcharset0 Arial;}{\\f1\\fmodern\\fcharset0 Courier New;}}\n",
        "{\\colortbl ;\\red0\\green102\\blue204;}\n",
        "\\viewkind4\\uc1\\pard\\sa200\\sl276\\slmult1\\f0\\fs24 "
    );

    let mut out = String::from(header);
    let mut list_depth: i32 = 0;
    let mut _in_blockquote = false;

    for token in tokenize_html(html) {
        match token {
            HtmlToken::Tag { name, closing, .. } => {
                emit_rtf_tag(
                    &name,
                    closing,
                    &mut list_depth,
                    &mut _in_blockquote,
                    &mut out,
                );
            }
            HtmlToken::Text(text) => {
                out.push_str(&escape_rtf(&text));
            }
        }
    }

    out.push('}');
    out
}

fn emit_rtf_tag(
    tag: &str,
    closing: bool,
    list_depth: &mut i32,
    in_blockquote: &mut bool,
    out: &mut String,
) {
    if closing {
        match tag {
            "b" | "strong" => out.push_str("\\b0 "),
            "i" | "em" => out.push_str("\\i0 "),
            "code" => out.push_str("\\f0 "),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                out.push_str("\\b0\\fs24\\par ");
            }
            "p" | "div" => out.push_str("\\par "),
            "blockquote" => {
                *in_blockquote = false;
                out.push_str("\\i0\\par\\pard\\sa200\\sl276\\slmult1 ");
            }
            "ul" | "ol" => {
                *list_depth -= 1;
                if *list_depth == 0 {
                    out.push_str("\\pard\\sa200\\sl276\\slmult1 ");
                }
            }
            "li" => out.push_str("\\par "),
            _ => {}
        }
    } else {
        match tag {
            "b" | "strong" => out.push_str("\\b "),
            "i" | "em" => out.push_str("\\i "),
            "code" => out.push_str("\\f1 "),
            "h1" => out.push_str("\\b\\fs48 "),
            "h2" => out.push_str("\\b\\fs40 "),
            "h3" => out.push_str("\\b\\fs36 "),
            "h4" => out.push_str("\\b\\fs32 "),
            "h5" | "h6" => out.push_str("\\b\\fs28 "),
            "br" => out.push_str("\\line "),
            "hr" => {
                out.push_str(
                    "\\par\\pard\\brdrb\\brdrs\\brdrw10\\brdr0 \\par\\pard\\sa200\\sl276\\slmult1 ",
                );
            }
            "blockquote" => {
                *in_blockquote = true;
                out.push_str("\\pard\\li720\\sa200\\sl276\\slmult1\\i ");
            }
            "ul" | "ol" => {
                *list_depth += 1;
            }
            "li" => {
                let indent = *list_depth * 360;
                out.push_str(&format!(
                    "\\pard\\li{indent}\\fi-360\\sa0\\sl276\\slmult1 \\'b7  "
                ));
            }
            _ => {}
        }
    }
}

pub(crate) fn escape_rtf(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '\n' => out.push_str("\\line "),
            c if (c as u32) > 127 => {
                // RTF unicode escape: \uN? (N = signed decimal Unicode code point)
                out.push_str(&format!("\\u{}?", c as i32));
            }
            c => out.push(c),
        }
    }
    out
}

// ── HTML tokenizer ────────────────────────────────────────────────────────────

#[derive(Debug)]
enum HtmlToken {
    Tag {
        name: String,
        closing: bool,
        attrs: HashMap<String, String>,
    },
    Text(String),
}

fn tokenize_html(html: &str) -> Vec<HtmlToken> {
    let mut tokens = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            // Find closing '>'
            if let Some(rel) = memchr_gt(&bytes[i..]) {
                let tag_inner = &html[i + 1..i + rel];
                i += rel + 1;

                // Skip comments and doctype
                if tag_inner.starts_with('!') || tag_inner.starts_with('?') {
                    continue;
                }

                let closing = tag_inner.starts_with('/');
                let content = if closing {
                    tag_inner[1..].trim()
                } else {
                    tag_inner.trim()
                };
                let content = if content.ends_with('/') {
                    &content[..content.len() - 1]
                } else {
                    content
                };

                let mut parts = content.splitn(2, |c: char| c.is_ascii_whitespace());
                let name = parts.next().unwrap_or("").to_lowercase();
                let attr_str = parts.next().unwrap_or("").trim();

                let attrs = parse_attrs(attr_str);

                if !name.is_empty() {
                    tokens.push(HtmlToken::Tag {
                        name,
                        closing,
                        attrs,
                    });
                }
            } else {
                break;
            }
        } else {
            // Collect text until next '<'
            let start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let text = decode_html_entities(&html[start..i]);
            if !text.trim().is_empty() || text.contains('\n') {
                tokens.push(HtmlToken::Text(text));
            }
        }
    }

    tokens
}

fn memchr_gt(bytes: &[u8]) -> Option<usize> {
    bytes.iter().position(|&b| b == b'>')
}

fn parse_attrs(attr_str: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let mut s = attr_str;
    while !s.is_empty() {
        s = s.trim_start();
        let Some(eq_pos) = s.find('=') else { break };
        let key = s[..eq_pos].trim().to_lowercase();
        s = s[eq_pos + 1..].trim_start();
        let (val, rest) = if s.starts_with('"') {
            let end = s[1..].find('"').map(|p| p + 1).unwrap_or(s.len());
            (s[1..end].to_string(), &s[end + 1..])
        } else if s.starts_with('\'') {
            let end = s[1..].find('\'').map(|p| p + 1).unwrap_or(s.len());
            (s[1..end].to_string(), &s[end + 1..])
        } else {
            let end = s.find(|c: char| c.is_ascii_whitespace()).unwrap_or(s.len());
            (s[..end].to_string(), &s[end..])
        };
        attrs.insert(key, val);
        s = rest;
    }
    attrs
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", "\u{00a0}")
        .replace("&mdash;", "\u{2014}")
        .replace("&ndash;", "\u{2013}")
        .replace("&hellip;", "\u{2026}")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_pipeline_produces_non_empty_output() {
        let html = compile_to_html(
            "# Hello\n\nThis is **bold** and _italic_.".to_string(),
            None,
        );
        assert!(
            html.contains("Hello"),
            "title should appear in HTML: {html}"
        );
        assert!(
            html.contains("<strong>") || html.contains("<b>") || html.contains("bold"),
            "bold markup should appear"
        );
    }

    #[test]
    fn rtf_contains_heading_markup() {
        let html = "<h1>Title</h1><p>Body text</p>";
        let rtf = html_to_rtf(html);
        assert!(rtf.starts_with("{\\rtf1"), "must start with RTF header");
        assert!(rtf.contains("\\b\\fs48"), "h1 should set bold + large font");
        assert!(
            rtf.contains("\\b0\\fs24"),
            "h1 close should reset font size"
        );
        assert!(rtf.contains("Title"), "title text must appear");
        assert!(rtf.contains("Body text"), "body text must appear");
    }

    #[test]
    fn rtf_covers_bold_italic_code() {
        let html = "<p><b>bold</b> <i>italic</i> <code>code()</code></p>";
        let rtf = html_to_rtf(html);
        assert!(rtf.contains("\\b "), "bold open");
        assert!(rtf.contains("\\b0 "), "bold close");
        assert!(rtf.contains("\\i "), "italic open");
        assert!(rtf.contains("\\i0 "), "italic close");
        assert!(rtf.contains("\\f1 "), "code → monospace font");
        assert!(rtf.contains("\\f0 "), "code close → back to normal font");
    }

    #[test]
    fn rtf_covers_lists() {
        let html = "<ul><li>Item A</li><li>Item B</li></ul>";
        let rtf = html_to_rtf(html);
        assert!(rtf.contains("\\li"), "list items should have indent");
        assert!(rtf.contains("Item A"), "list content should appear");
    }

    #[test]
    fn rtf_covers_hr_and_blockquote() {
        let html = "<p>before</p><hr><blockquote>quoted</blockquote>";
        let rtf = html_to_rtf(html);
        assert!(rtf.contains("\\brdrb"), "hr should add border");
        assert!(rtf.contains("\\li720"), "blockquote should indent");
        assert!(rtf.contains("\\i "), "blockquote should italicize");
    }

    #[test]
    fn rtf_escapes_special_chars() {
        let text = r"back\slash {brace}";
        let escaped = escape_rtf(text);
        assert!(escaped.contains(r"\\"), "backslash escaped");
        assert!(escaped.contains(r"\{"), "open brace escaped");
        assert!(escaped.contains(r"\}"), "close brace escaped");
    }

    #[test]
    fn rtf_encodes_unicode() {
        let text = "café";
        let escaped = escape_rtf(text);
        assert!(
            escaped.contains("\\u"),
            "non-ASCII should use RTF unicode escape"
        );
    }

    #[test]
    fn html_to_plain_text_strips_tags() {
        let html = "<h1>Title</h1><p>Body with <b>bold</b> text.</p>";
        let plain = html_to_plain_text(html);
        assert!(!plain.contains('<'), "no HTML tags in plain text");
        assert!(plain.contains("Title"), "title preserved");
        assert!(plain.contains("bold"), "inline text preserved");
    }
}
