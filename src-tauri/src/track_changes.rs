use serde::{Deserialize, Serialize};

const SUGGEST_DELETE_OPEN: &str = "<!--suggest-delete:";
const SUGGEST_INSERT_OPEN: &str = "<!--suggest-insert:";
const SUGGEST_CLOSE: &str = "-->";
const END_DELETE: &str = "<!--/suggest-delete-->";
const END_INSERT: &str = "<!--/suggest-insert-->";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub kind: String, // "delete" | "insert"
    pub text: String,
    pub author: String,
    pub line: usize,
}

fn generate_suggestion_id() -> Result<String, String> {
    let mut bytes = [0u8; 6];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| format!("Failed to generate suggestion ID: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

#[tauri::command]
pub(crate) fn create_delete_suggestion(
    text_to_delete: String,
    author: String,
) -> Result<String, String> {
    let id = generate_suggestion_id()?;
    Ok(format!(
        "{SUGGEST_DELETE_OPEN}{id} author:{author}{SUGGEST_CLOSE}{text_to_delete}{END_DELETE}"
    ))
}

#[tauri::command]
pub(crate) fn create_insert_suggestion(
    text_to_insert: String,
    author: String,
) -> Result<String, String> {
    let id = generate_suggestion_id()?;
    Ok(format!(
        "{SUGGEST_INSERT_OPEN}{id} author:{author}{SUGGEST_CLOSE}{text_to_insert}{END_INSERT}"
    ))
}

#[tauri::command]
pub(crate) fn accept_suggestion(document: String, suggestion_id: String) -> Result<String, String> {
    let result = process_suggestion(&document, &suggestion_id, true);
    Ok(result)
}

#[tauri::command]
pub(crate) fn reject_suggestion(document: String, suggestion_id: String) -> Result<String, String> {
    let result = process_suggestion(&document, &suggestion_id, false);
    Ok(result)
}

fn process_suggestion(doc: &str, id: &str, accept: bool) -> String {
    let mut result = doc.to_string();

    // Anchor with trailing space so id "abc" doesn't match "abcdef"
    let delete_marker = format!("{SUGGEST_DELETE_OPEN}{id} ");
    if let Some(start) = result.find(&delete_marker) {
        if let Some(close_pos) = result[start..].find(SUGGEST_CLOSE) {
            let content_start = start + close_pos + SUGGEST_CLOSE.len();
            if let Some(end_pos) = result[content_start..].find(END_DELETE) {
                let content = result[content_start..content_start + end_pos].to_string();
                let full_end = content_start + end_pos + END_DELETE.len();
                if accept {
                    // Keep the deleted text (reject the deletion)
                    result.replace_range(start..full_end, &content);
                } else {
                    // Remove the deleted text (accept the deletion)
                    result.replace_range(start..full_end, "");
                }
                return result;
            }
        }
    }

    // Handle insert suggestions
    let insert_marker = format!("{SUGGEST_INSERT_OPEN}{id} ");
    if let Some(start) = result.find(&insert_marker) {
        if let Some(close_pos) = result[start..].find(SUGGEST_CLOSE) {
            let content_start = start + close_pos + SUGGEST_CLOSE.len();
            if let Some(end_pos) = result[content_start..].find(END_INSERT) {
                let content = result[content_start..content_start + end_pos].to_string();
                let full_end = content_start + end_pos + END_INSERT.len();
                if accept {
                    // Keep the inserted text
                    result.replace_range(start..full_end, &content);
                } else {
                    // Remove the inserted text
                    result.replace_range(start..full_end, "");
                }
                return result;
            }
        }
    }

    result
}

#[tauri::command]
pub(crate) fn accept_all_suggestions(document: String) -> String {
    let mut result = document;
    // Accept all delete suggestions (keep text)
    result = accept_all_of_kind(result, SUGGEST_DELETE_OPEN, SUGGEST_CLOSE, END_DELETE, true);
    // Accept all insert suggestions (keep text)
    result = accept_all_of_kind(result, SUGGEST_INSERT_OPEN, SUGGEST_CLOSE, END_INSERT, true);
    result
}

#[tauri::command]
pub(crate) fn reject_all_suggestions(document: String) -> String {
    let mut result = document;
    // Reject all delete suggestions (remove text)
    result = accept_all_of_kind(
        result,
        SUGGEST_DELETE_OPEN,
        SUGGEST_CLOSE,
        END_DELETE,
        false,
    );
    // Reject all insert suggestions (remove text)
    result = accept_all_of_kind(
        result,
        SUGGEST_INSERT_OPEN,
        SUGGEST_CLOSE,
        END_INSERT,
        false,
    );
    result
}

fn accept_all_of_kind(mut doc: String, open: &str, close: &str, end: &str, accept: bool) -> String {
    loop {
        let Some(start) = doc.find(open) else { break };
        let Some(close_pos) = doc[start..].find(close) else {
            break;
        };
        let content_start = start + close_pos + close.len();
        let Some(end_pos) = doc[content_start..].find(end) else {
            break;
        };
        let content = doc[content_start..content_start + end_pos].to_string();
        let full_end = content_start + end_pos + end.len();
        if accept {
            doc.replace_range(start..full_end, &content);
        } else {
            doc.replace_range(start..full_end, "");
        }
    }
    doc
}

#[tauri::command]
pub(crate) fn list_suggestions(document: String) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    parse_suggestions_of_kind(
        &document,
        SUGGEST_DELETE_OPEN,
        SUGGEST_CLOSE,
        END_DELETE,
        "delete",
        &mut suggestions,
    );
    parse_suggestions_of_kind(
        &document,
        SUGGEST_INSERT_OPEN,
        SUGGEST_CLOSE,
        END_INSERT,
        "insert",
        &mut suggestions,
    );

    suggestions.sort_by_key(|s| s.line);
    suggestions
}

fn parse_suggestions_of_kind(
    doc: &str,
    open: &str,
    close: &str,
    end: &str,
    kind: &str,
    out: &mut Vec<Suggestion>,
) {
    let mut search = 0usize;
    loop {
        let Some(rel_start) = doc[search..].find(open) else {
            break;
        };
        let start = search + rel_start;
        let Some(rel_close) = doc[start..].find(close) else {
            break;
        };
        let meta_raw = &doc[start + open.len()..start + rel_close];
        // Require at least one space after the id to prevent prefix collisions
        if !meta_raw.contains(' ') {
            search = start + 1;
            continue;
        }
        let meta = meta_raw;
        let content_start = start + rel_close + close.len();
        let Some(rel_end) = doc[content_start..].find(end) else {
            break;
        };

        // Parse id and author from meta: "id123 author:Name"
        let parts: Vec<&str> = meta.splitn(2, ' ').collect();
        let id = parts.first().unwrap_or(&"").trim().to_string();
        let author = parts
            .get(1)
            .and_then(|s| s.strip_prefix("author:"))
            .unwrap_or("unknown")
            .to_string();
        let text = doc[content_start..content_start + rel_end].to_string();

        // Calculate line number
        let line = doc[..start].chars().filter(|&c| c == '\n').count() + 1;

        out.push(Suggestion {
            id,
            kind: kind.to_string(),
            text,
            author,
            line,
        });
        search = content_start + rel_end + end.len();
    }
}
