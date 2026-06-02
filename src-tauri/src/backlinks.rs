use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct Backlink {
    pub source_path: String,
    pub line: usize,
    pub excerpt: String,
}

#[tauri::command]
pub(crate) fn find_backlinks(target_path: String, workspace_root: String) -> Result<Vec<Backlink>, String> {
    let root = PathBuf::from(&workspace_root);
    let target = PathBuf::from(&target_path);
    let target_name = target.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_ascii_lowercase();
    let mut backlinks = Vec::new();
    find_in_dir(&root, &root, &target_name, &mut backlinks);
    Ok(backlinks)
}

fn find_in_dir(root: &PathBuf, dir: &PathBuf, target: &str, results: &mut Vec<Backlink>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if name.starts_with('.') { continue; }
        // Skip symlinks to prevent infinite recursion on cyclic symlinks
        if path.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false) { continue; }
        if path.is_dir() { find_in_dir(root, &path, target, results); continue; }
        if path.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
        let Ok(content) = fs::read_to_string(&path) else { continue };
        let rel = path.strip_prefix(root).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        for (li, line) in content.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            // Check for [[target]] wiki links or include directives
            if lower.contains(&format!("[[{target}]]")) || lower.contains(&format!("include: {target}")) {
                results.push(Backlink { source_path: rel.clone(), line: li + 1, excerpt: line.trim().to_string() });
            }
        }
    }
}

#[tauri::command]
pub(crate) fn check_document_approval(path: String) -> Result<bool, String> {
    // Check if document has status: approved or status: signed in front matter (locked)
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let locked = content.lines().take(30).any(|l| {
        let l = l.trim().to_ascii_lowercase();
        if !l.starts_with("status:") { return false; }
        let val = l["status:".len()..].trim();
        val == "approved" || val == "signed" || val == "locked"
    });
    Ok(locked)
}
