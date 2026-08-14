use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceTask {
    pub file_path: String,
    pub line: usize,
    pub text: String,
    pub done: bool,
    pub tags: Vec<String>,
    pub due_date: Option<String>,
    pub heading_context: String,
}

fn extract_tags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '#' {
            let tag: String = chars
                .by_ref()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !tag.is_empty() {
                tags.push(tag);
            }
        }
    }
    tags
}

fn extract_due_date(text: &str) -> Option<String> {
    // Match @YYYY-MM-DD
    if let Some(pos) = text.find('@') {
        let rest = &text[pos + 1..];
        let candidate: String = rest.chars().take(10).collect();
        if candidate.len() == 10 && is_date_like(&candidate) {
            return Some(candidate);
        }
    }
    // Match due:YYYY-MM-DD or due: YYYY-MM-DD
    let lower = text.to_ascii_lowercase();
    if let Some(pos) = lower.find("due:") {
        let rest = text[pos + 4..].trim_start();
        let candidate: String = rest.chars().take(10).collect();
        if candidate.len() == 10 && is_date_like(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_date_like(s: &str) -> bool {
    s.len() == 10
        && s.chars().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                c == '-'
            } else {
                c.is_ascii_digit()
            }
        })
}

fn scan_file(root: &PathBuf, path: &PathBuf, results: &mut Vec<WorkspaceTask>, max: usize) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string());

    let mut current_heading = String::new();

    for (li, line) in content.lines().enumerate() {
        if results.len() >= max {
            return;
        }

        let trimmed = line.trim();

        // Track nearest heading context
        if trimmed.starts_with('#') {
            current_heading = trimmed.trim_start_matches('#').trim().to_string();
            continue;
        }

        // Match todo/done checkbox lines
        let (matched, done) = if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
            (true, false)
        } else if trimmed.starts_with("- [x]")
            || trimmed.starts_with("- [X]")
            || trimmed.starts_with("* [x]")
            || trimmed.starts_with("* [X]")
        {
            (true, true)
        } else {
            (false, false)
        };

        if matched {
            // Strip the checkbox prefix to get the task text
            let text = trimmed
                .trim_start_matches("- [ ]")
                .trim_start_matches("- [x]")
                .trim_start_matches("- [X]")
                .trim_start_matches("* [ ]")
                .trim_start_matches("* [x]")
                .trim_start_matches("* [X]")
                .trim()
                .to_string();

            let tags = extract_tags(&text);
            let due_date = extract_due_date(&text);

            results.push(WorkspaceTask {
                file_path: rel.clone(),
                line: li + 1,
                text,
                done,
                tags,
                due_date,
                heading_context: current_heading.clone(),
            });
        }
    }
}

fn scan_dir(root: &PathBuf, dir: &PathBuf, results: &mut Vec<WorkspaceTask>, max: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if results.len() >= max {
            return;
        }
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        if path
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            continue;
        }
        if path.is_dir() {
            scan_dir(root, &path, results, max);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            scan_file(root, &path, results, max);
        }
    }
}

#[tauri::command]
pub(crate) fn collect_workspace_tasks(
    workspace_root: String,
) -> Result<Vec<WorkspaceTask>, String> {
    let root = PathBuf::from(&workspace_root);
    if !root.exists() {
        return Err(format!("Workspace root does not exist: {workspace_root}"));
    }
    let mut results = Vec::new();
    scan_dir(&root, &root, &mut results, 2000);
    Ok(results)
}
