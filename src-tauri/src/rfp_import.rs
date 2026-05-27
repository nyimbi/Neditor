use crate::path_to_string;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
pub(crate) struct ImportRfpSourceRequest {
    pub(crate) source_type: String,
    pub(crate) path: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) text: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportRfpSourceResponse {
    pub(crate) source_type: String,
    pub(crate) title: String,
    pub(crate) path: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) text: String,
    pub(crate) extraction_method: String,
    pub(crate) warnings: Vec<String>,
}

#[tauri::command]
pub(crate) fn import_rfp_source(
    request: ImportRfpSourceRequest,
) -> Result<ImportRfpSourceResponse, String> {
    let source_type = normalize_source_type(&request.source_type);
    let mut warnings = Vec::new();
    let (text, title, extraction_method) = match source_type.as_str() {
        "markdown" => import_markdown(&request)?,
        "docx" => import_docx(&request)?,
        "pdf" => import_pdf(&request, &mut warnings)?,
        "url" => import_url(&request, &mut warnings)?,
        _ => {
            return Err("Unsupported RFP source type. Use markdown, pdf, docx, or url.".to_string())
        }
    };
    let cleaned = normalize_text(&text);
    if cleaned.len() < 400 {
        warnings.push("Imported source is short; confirm the full RFP, attachments, and tables were captured.".to_string());
    }
    Ok(ImportRfpSourceResponse {
        source_type,
        title,
        path: request
            .path
            .map(|path| path_to_string(&PathBuf::from(path))),
        url: request
            .url
            .map(|url| url.trim().to_string())
            .filter(|url| !url.is_empty()),
        text: cleaned,
        extraction_method,
        warnings,
    })
}

fn normalize_source_type(value: &str) -> String {
    match value.trim().to_lowercase().as_str() {
        "md" | "markdown" | "txt" | "text" => "markdown".to_string(),
        "pdf" => "pdf".to_string(),
        "docx" | "word" => "docx".to_string(),
        "url" | "uri" | "link" => "url".to_string(),
        other => other.to_string(),
    }
}

fn import_markdown(request: &ImportRfpSourceRequest) -> Result<(String, String, String), String> {
    if let Some(text) = request
        .text
        .as_deref()
        .filter(|text| !text.trim().is_empty())
    {
        return Ok((
            text.to_string(),
            request_title(request),
            "pasted-markdown-text".to_string(),
        ));
    }
    let path = request_path(request)?;
    let text =
        fs::read_to_string(&path).map_err(|err| format!("Could not read Markdown RFP: {err}"))?;
    Ok((
        text,
        title_from_path(&path),
        "native-markdown-file".to_string(),
    ))
}

fn import_docx(request: &ImportRfpSourceRequest) -> Result<(String, String, String), String> {
    let path = request_path(request)?;
    let file = fs::File::open(&path).map_err(|err| format!("Could not open DOCX RFP: {err}"))?;
    let mut archive =
        ZipArchive::new(file).map_err(|err| format!("Could not inspect DOCX package: {err}"))?;
    let mut parts = Vec::new();
    let names = archive.file_names().map(str::to_string).collect::<Vec<_>>();
    for name in docx_text_part_names(names) {
        if let Ok(mut file) = archive.by_name(&name) {
            let mut xml = String::new();
            file.read_to_string(&mut xml)
                .map_err(|err| format!("Could not read {name} from DOCX: {err}"))?;
            let text = docx_xml_to_text(&xml);
            if !text.trim().is_empty() {
                parts.push(text);
            }
        }
    }
    if parts.is_empty() {
        return Err("DOCX did not contain readable Word document XML.".to_string());
    }
    Ok((
        parts.join("\n\n"),
        title_from_path(&path),
        "native-docx-package-text".to_string(),
    ))
}

fn docx_text_part_names(names: Vec<String>) -> Vec<String> {
    let mut ordered = Vec::new();
    if names.iter().any(|name| name == "word/document.xml") {
        ordered.push("word/document.xml".to_string());
    }
    for prefix in ["word/header", "word/footer"] {
        let mut matches = names
            .iter()
            .filter(|name| docx_numbered_part_name(name, prefix))
            .map(String::as_str)
            .collect::<Vec<_>>();
        matches.sort_unstable();
        for name in matches {
            ordered.push(name.to_string());
        }
    }
    for name in [
        "word/footnotes.xml",
        "word/endnotes.xml",
        "word/comments.xml",
    ] {
        if names.iter().any(|candidate| candidate == name) {
            ordered.push(name.to_string());
        }
    }
    ordered
}

fn docx_numbered_part_name(name: &str, prefix: &str) -> bool {
    let Some(number) = name
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(".xml"))
    else {
        return false;
    };
    !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit())
}

fn import_pdf(
    request: &ImportRfpSourceRequest,
    warnings: &mut Vec<String>,
) -> Result<(String, String, String), String> {
    let path = request_path(request)?;
    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg(&path)
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            warnings.push("PDF text extraction depends on the local pdftotext utility; verify tables and scanned pages manually.".to_string());
            Ok((text, title_from_path(&path), "pdftotext-layout".to_string()))
        }
        Ok(output) => Err(format!(
            "Could not extract PDF text with pdftotext: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(err) => Err(format!(
            "PDF import needs the pdftotext utility on this machine, or paste extracted RFP text manually: {err}"
        )),
    }
}

fn import_url(
    request: &ImportRfpSourceRequest,
    warnings: &mut Vec<String>,
) -> Result<(String, String, String), String> {
    let url = request
        .url
        .as_deref()
        .map(str::trim)
        .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
        .ok_or_else(|| "Enter an http:// or https:// RFP URL.".to_string())?;
    let output = Command::new("curl")
        .args([
            "--location",
            "--max-time",
            "25",
            "--fail",
            "--silent",
            "--show-error",
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| format!("Could not fetch RFP URL with curl: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "Could not fetch RFP URL: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    warnings.push("URL import captures the public page response; verify linked attachments and addenda manually.".to_string());
    Ok((
        html_to_text(&raw),
        html_title(&raw).unwrap_or_else(|| url.to_string()),
        "curl-url-text".to_string(),
    ))
}

fn request_path(request: &ImportRfpSourceRequest) -> Result<PathBuf, String> {
    request
        .path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "Choose an RFP source file first.".to_string())
}

fn request_title(request: &ImportRfpSourceRequest) -> String {
    request
        .url
        .as_deref()
        .or(request.path.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Pasted RFP")
        .to_string()
}

fn title_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Imported RFP")
        .to_string()
}

fn docx_xml_to_text(xml: &str) -> String {
    decode_text_entities(
        &xml.replace("</w:p>", "\n")
            .replace("</w:tr>", "\n")
            .replace("</w:tc>", "\t")
            .replace("<w:tab/>", "\t")
            .replace("<w:br/>", "\n")
            .split('<')
            .map(|chunk| chunk.split_once('>').map(|(_, text)| text).unwrap_or(""))
            .collect::<Vec<_>>()
            .join(""),
    )
}

fn html_to_text(html: &str) -> String {
    let cleaned = remove_html_comments(&remove_html_element_blocks(
        html,
        &["script", "style", "noscript", "svg"],
    ));
    let mut output = String::new();
    let mut tag = String::new();
    let mut in_tag = false;
    for ch in cleaned.chars() {
        if in_tag {
            if ch == '>' {
                push_html_tag_separator(&tag, &mut output);
                tag.clear();
                in_tag = false;
            } else {
                tag.push(ch);
            }
        } else if ch == '<' {
            in_tag = true;
        } else {
            output.push(ch);
        }
    }
    normalize_text(&decode_text_entities(&output))
}

fn html_title(html: &str) -> Option<String> {
    let cleaned = remove_html_element_blocks(html, &["script", "style", "noscript", "svg"]);
    let lower = cleaned.to_lowercase();
    let title_open = lower.find("<title")?;
    let title_body = lower[title_open..]
        .find('>')
        .map(|offset| title_open + offset + 1)?;
    let title_close = lower[title_body..]
        .find("</title>")
        .map(|offset| title_body + offset)?;
    let title = normalize_text(&decode_text_entities(&cleaned[title_body..title_close]));
    (!title.is_empty()).then_some(title)
}

fn push_html_tag_separator(raw_tag: &str, output: &mut String) {
    let tag = raw_tag
        .trim()
        .trim_start_matches('/')
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches('/')
        .to_ascii_lowercase();
    if tag.is_empty() || raw_tag.trim_start().starts_with('!') {
        return;
    }
    if matches!(tag.as_str(), "td" | "th") {
        push_separator(output, '\t');
    } else if matches!(
        tag.as_str(),
        "br" | "p"
            | "div"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "li"
            | "ul"
            | "ol"
            | "tr"
            | "table"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
    ) {
        push_separator(output, '\n');
    }
}

fn push_separator(output: &mut String, separator: char) {
    if output.ends_with(separator) {
        return;
    }
    output.push(separator);
}

fn remove_html_element_blocks(html: &str, element_names: &[&str]) -> String {
    let mut output = html.to_string();
    for element_name in element_names {
        loop {
            let lower = output.to_lowercase();
            let Some(start) = lower.find(&format!("<{element_name}")) else {
                break;
            };
            let body_start = lower[start..]
                .find('>')
                .map(|offset| start + offset + 1)
                .unwrap_or(start);
            let end_pattern = format!("</{element_name}>");
            let end = lower[body_start..]
                .find(&end_pattern)
                .map(|offset| body_start + offset + end_pattern.len())
                .unwrap_or(body_start);
            output.replace_range(start..end, " ");
        }
    }
    output
}

fn remove_html_comments(html: &str) -> String {
    let mut output = html.to_string();
    while let Some(start) = output.find("<!--") {
        let end = output[start + 4..]
            .find("-->")
            .map(|offset| start + 4 + offset + 3)
            .unwrap_or(output.len());
        output.replace_range(start..end, " ");
    }
    output
}

fn decode_text_entities(text: &str) -> String {
    let mut output = String::new();
    let mut remaining = text;
    while let Some(start) = remaining.find('&') {
        output.push_str(&remaining[..start]);
        let after_amp = &remaining[start + 1..];
        let Some(end) = after_amp.find(';') else {
            output.push('&');
            remaining = after_amp;
            continue;
        };
        let entity = &after_amp[..end];
        if let Some(decoded) = decode_entity(entity) {
            output.push(decoded);
        } else {
            output.push('&');
            output.push_str(entity);
            output.push(';');
        }
        remaining = &after_amp[end + 1..];
    }
    output.push_str(remaining);
    output
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "nbsp" => Some(' '),
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ if entity.starts_with("#x") || entity.starts_with("#X") => {
            u32::from_str_radix(&entity[2..], 16)
                .ok()
                .and_then(char::from_u32)
        }
        _ if entity.starts_with('#') => entity[1..].parse::<u32>().ok().and_then(char::from_u32),
        _ => None,
    }
}

fn normalize_text(text: &str) -> String {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\n\n\n", "\n\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docx_xml_text_preserves_paragraphs_and_table_cells() {
        let text = docx_xml_to_text(
            r#"<w:p><w:r><w:t>Requirement &amp; A</w:t></w:r></w:p><w:tr><w:tc><w:p><w:r><w:t>Cell &#x32;</w:t></w:r></w:p></w:tc></w:tr>"#,
        );
        assert!(text.contains("Requirement & A"));
        assert!(text.contains("Cell 2"));
    }

    #[test]
    fn docx_text_part_names_cover_review_and_all_header_footer_parts() {
        let names = docx_text_part_names(vec![
            "[Content_Types].xml".to_string(),
            "word/footer2.xml".to_string(),
            "word/header3.xml".to_string(),
            "word/headerStyles.xml".to_string(),
            "word/document.xml".to_string(),
            "word/comments.xml".to_string(),
            "word/_rels/document.xml.rels".to_string(),
            "word/footnotes.xml".to_string(),
            "word/header1.xml".to_string(),
            "word/endnotes.xml".to_string(),
            "word/footer1.xml".to_string(),
        ]);

        assert_eq!(
            names,
            vec![
                "word/document.xml",
                "word/header1.xml",
                "word/header3.xml",
                "word/footer1.xml",
                "word/footer2.xml",
                "word/footnotes.xml",
                "word/endnotes.xml",
                "word/comments.xml",
            ]
        );
    }

    #[test]
    fn url_html_text_strips_markup() {
        let text = html_to_text(
            "<html><head><title>Support RFP</title><style>.x{}</style><script>alert('ignore')</script></head><body><h1>RFP</h1><p>Vendor must provide support &amp; training.</p><table><tr><th>ID</th><th>Requirement</th></tr><tr><td>1</td><td>Submit certificates &#x2713;</td></tr></table></body></html>",
        );
        assert!(text.contains("RFP"));
        assert!(text.contains("Vendor must provide support & training."));
        assert!(text.contains("Submit certificates ✓"));
        assert!(!text.contains("alert"));
        assert!(!text.contains(".x"));
        assert!(!text.contains("<p>"));
    }

    #[test]
    fn url_html_title_uses_page_title() {
        let title = html_title(
            "<html><head><title>Customer Support RFP &amp; Addendum</title></head><body>Body</body></html>",
        );
        assert_eq!(title, Some("Customer Support RFP & Addendum".to_string()));
    }
}
