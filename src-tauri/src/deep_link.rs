/// URL-scheme handler for `neditor://` deep links.
///
/// Supported forms:
///   neditor://open?path=/abs/path.md
///   neditor://open?path=/abs/path.md&line=42&col=7
///   neditor://export?path=/abs/path.md&format=html
///
/// Security:
///   - `path` must be absolute.
///   - URL length capped at 4 KiB.
///   - `file://` injection is rejected.
///   - No relative traversal allowed.
use crate::cli_ipc::queue_paths_for_open;
use serde::Serialize;
use tauri::AppHandle;

const MAX_URL_BYTES: usize = 4 * 1024;

/// Parsed representation of a neditor:// URL.
#[derive(Debug, PartialEq)]
pub(crate) enum DeepLinkAction {
    Open {
        path: String,
        line: Option<u32>,
        col: Option<u32>,
    },
    Export {
        path: String,
        format: String,
    },
}

/// Cursor position emitted as a Tauri event after `open?line=&col=`.
#[derive(Clone, Serialize)]
pub(crate) struct DeepLinkCursorPayload {
    pub(crate) path: String,
    pub(crate) line: u32,
    pub(crate) col: u32,
}

/// Parse a `neditor://` URL string into a `DeepLinkAction`.
///
/// Returns `Err` for any invalid, unsafe, or oversized URL.
pub(crate) fn parse_deep_link(url: &str) -> Result<DeepLinkAction, String> {
    if url.len() > MAX_URL_BYTES {
        return Err(format!(
            "URL exceeds maximum length of {MAX_URL_BYTES} bytes"
        ));
    }

    // Basic scheme check
    let without_scheme = url
        .strip_prefix("neditor://")
        .ok_or_else(|| "URL must use the neditor:// scheme".to_string())?;

    // Split host/path from query
    let (command_part, query_part) = match without_scheme.find('?') {
        Some(pos) => (&without_scheme[..pos], &without_scheme[pos + 1..]),
        None => (without_scheme, ""),
    };

    let command = command_part.trim_matches('/');

    let params = parse_query(query_part);

    match command {
        "open" => {
            let path = params
                .get("path")
                .ok_or("neditor://open requires a ?path= parameter")?
                .clone();
            validate_path(&path)?;
            let line = params.get("line").and_then(|v| v.parse::<u32>().ok());
            let col = params.get("col").and_then(|v| v.parse::<u32>().ok());
            Ok(DeepLinkAction::Open { path, line, col })
        }
        "export" => {
            let path = params
                .get("path")
                .ok_or("neditor://export requires a ?path= parameter")?
                .clone();
            validate_path(&path)?;
            let format = params
                .get("format")
                .cloned()
                .unwrap_or_else(|| "html".to_string());
            Ok(DeepLinkAction::Export { path, format })
        }
        other => Err(format!("Unknown neditor:// command: {other}")),
    }
}

fn validate_path(path: &str) -> Result<(), String> {
    // Reject file:// injection
    if path.starts_with("file://") {
        return Err("path must not use file:// scheme".to_string());
    }
    // Must be absolute
    if !std::path::Path::new(path).is_absolute() {
        return Err("path must be absolute".to_string());
    }
    // No traversal components
    if path.contains("/../") || path.ends_with("/..") {
        return Err("path must not contain .. traversal".to_string());
    }
    Ok(())
}

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for pair in query.split('&') {
        if let Some(eq) = pair.find('=') {
            let key = percent_decode(&pair[..eq]);
            let val = percent_decode(&pair[eq + 1..]);
            map.insert(key, val);
        }
    }
    map
}

fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte as char);
                    i += 3;
                    continue;
                }
            }
        } else if bytes[i] == b'+' {
            out.push(' ');
            i += 1;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Handle a parsed `DeepLinkAction`: queue open or run export.
pub(crate) fn handle_deep_link(action: DeepLinkAction, app: &AppHandle) {
    use tauri::Emitter;
    match action {
        DeepLinkAction::Open { path, line, col } => {
            let _ = queue_paths_for_open(&[path.clone()]);
            if let (Some(line), Some(col)) = (line, col) {
                let _ = app.emit(
                    "deep-link-cursor",
                    DeepLinkCursorPayload { path, line, col },
                );
            }
        }
        DeepLinkAction::Export { path, format } => {
            // Headless export: run the compiler and write the output file next to the source.
            // For now queue the path so the frontend can trigger the export.
            let _ = queue_paths_for_open(&[path.clone()]);
            let _ = app.emit(
                "deep-link-export",
                serde_json::json!({ "path": path, "format": format }),
            );
        }
    }
}

/// Register the deep-link handler with tauri-plugin-deep-link during app setup.
pub(crate) fn setup_deep_link_handler(app: &AppHandle) {
    use tauri_plugin_deep_link::DeepLinkExt;
    let handle = app.clone();
    let _ = app.deep_link().on_open_url(move |event| {
        for url in event.urls() {
            let url_str = url.as_str();
            match parse_deep_link(url_str) {
                Ok(action) => handle_deep_link(action, &handle),
                Err(e) => {
                    eprintln!("[neditor deep-link] rejected URL {url_str:?}: {e}");
                }
            }
        }
    });
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_open_path_only() {
        let result = parse_deep_link("neditor://open?path=/Users/alice/notes.md").unwrap();
        assert_eq!(
            result,
            DeepLinkAction::Open {
                path: "/Users/alice/notes.md".to_string(),
                line: None,
                col: None
            }
        );
    }

    #[test]
    fn parse_open_with_line_and_col() {
        let result = parse_deep_link("neditor://open?path=/abs/doc.md&line=42&col=7").unwrap();
        assert_eq!(
            result,
            DeepLinkAction::Open {
                path: "/abs/doc.md".to_string(),
                line: Some(42),
                col: Some(7)
            }
        );
    }

    #[test]
    fn parse_export_html() {
        let result = parse_deep_link("neditor://export?path=/abs/report.md&format=html").unwrap();
        assert_eq!(
            result,
            DeepLinkAction::Export {
                path: "/abs/report.md".to_string(),
                format: "html".to_string()
            }
        );
    }

    #[test]
    fn rejects_file_injection() {
        let err = parse_deep_link("neditor://open?path=file:///etc/passwd").unwrap_err();
        assert!(
            err.contains("file://"),
            "should explain the rejection: {err}"
        );
    }

    #[test]
    fn rejects_non_absolute_path() {
        let err = parse_deep_link("neditor://open?path=relative/path.md").unwrap_err();
        assert!(err.contains("absolute"), "{err}");
    }

    #[test]
    fn rejects_traversal_in_path() {
        let err = parse_deep_link("neditor://open?path=/safe/../etc/shadow").unwrap_err();
        assert!(err.contains(".."), "{err}");
    }

    #[test]
    fn rejects_url_over_4kb() {
        let long_path = "/abs/".to_string() + &"a".repeat(4096);
        let url = format!("neditor://open?path={long_path}");
        let err = parse_deep_link(&url).unwrap_err();
        assert!(err.contains("maximum length"), "{err}");
    }

    #[test]
    fn rejects_unknown_command() {
        let err = parse_deep_link("neditor://frobnicate?path=/abs/x.md").unwrap_err();
        assert!(err.contains("Unknown"), "{err}");
    }

    #[test]
    fn rejects_wrong_scheme() {
        let err = parse_deep_link("https://neditor.app/open?path=/abs/x.md").unwrap_err();
        assert!(err.contains("neditor://"), "{err}");
    }

    #[test]
    fn percent_decode_works() {
        assert_eq!(percent_decode("/path%20with%20spaces"), "/path with spaces");
        assert_eq!(percent_decode("a+b"), "a b");
    }
}
