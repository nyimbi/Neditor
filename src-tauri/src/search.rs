use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub excerpt: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub workspace_root: String,
    pub case_sensitive: bool,
    pub max_results: usize,
}

#[tauri::command]
pub(crate) fn search_workspace(request: SearchRequest) -> Result<Vec<SearchMatch>, String> {
    let root = PathBuf::from(&request.workspace_root);
    if !root.exists() {
        return Err("Workspace root does not exist.".to_string());
    }
    let query = if request.case_sensitive {
        request.query.clone()
    } else {
        request.query.to_ascii_lowercase()
    };
    let max = request.max_results.min(500);
    let mut results = Vec::new();
    search_dir(&root, &root, &query, request.case_sensitive, &mut results, max);
    Ok(results)
}

fn search_dir(root: &PathBuf, dir: &PathBuf, query: &str, case_sensitive: bool, results: &mut Vec<SearchMatch>, max: usize) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if results.len() >= max { return; }
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if name.starts_with('.') || name == "node_modules" || name == "target" { continue; }
        if path.is_dir() {
            search_dir(root, &path, query, case_sensitive, results, max);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let Ok(content) = fs::read_to_string(&path) else { continue };
            let rel = path.strip_prefix(root).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| path.to_string_lossy().to_string());
            for (li, line) in content.lines().enumerate() {
                if results.len() >= max { return; }
                let haystack = if case_sensitive { line.to_string() } else { line.to_ascii_lowercase() };
                if let Some(col) = haystack.find(query) {
                    // Clamp to nearest UTF-8 char boundary to avoid panic on multi-byte chars
                    let start = {
                        let raw = col.saturating_sub(40);
                        (0..=raw).rev().find(|&i| line.is_char_boundary(i)).unwrap_or(0)
                    };
                    let end = {
                        let raw = (col + query.len() + 40).min(line.len());
                        (raw..=line.len()).find(|&i| line.is_char_boundary(i)).unwrap_or(line.len())
                    };
                    let excerpt = format!("…{}…", &line[start..end]);
                    // Bound text to 500 chars to prevent huge lines exhausting memory.
                    let text_raw = line.trim();
                    let text = if text_raw.len() > 500 {
                        // Clamp to nearest char boundary at or below 497 bytes.
                        let cut = (0..=497usize).rev().find(|&i| text_raw.is_char_boundary(i)).unwrap_or(0);
                        format!("{}…", &text_raw[..cut])
                    } else {
                        text_raw.to_string()
                    };
                    results.push(SearchMatch {
                        path: rel.clone(),
                        line: li + 1,
                        column: col + 1,
                        text,
                        excerpt,
                    });
                }
            }
        }
    }
}
