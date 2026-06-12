use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Command};

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub path: String,
    pub workspace_root: String,
}

fn safe_path(path: &str, workspace_root: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(path);
    let canonical = p.canonicalize().map_err(|e| format!("Cannot resolve path '{}': {e}", path))?;
    let root_canon = PathBuf::from(workspace_root)
        .canonicalize()
        .map_err(|e| format!("Cannot resolve workspace_root '{}': {e}", workspace_root))?;
    if !canonical.starts_with(&root_canon) {
        return Err(format!("Path '{}' is outside the workspace root.", path));
    }
    Ok(canonical)
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub markdown: String,
    pub source_format: String,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub(crate) fn import_document(request: ImportRequest) -> Result<ImportResponse, String> {
    let safe = safe_path(&request.path, &request.workspace_root)?;
    let ext = safe.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
    let format = match ext.as_str() {
        "docx" | "doc" => "docx",
        "pptx" | "ppt" => "pptx",
        "odt" => "odt",
        "rtf" => "rtf",
        "html" | "htm" => "html",
        other => return Err(format!("Unsupported import format: .{other}. Supported: docx, pptx, odt, rtf, html.")),
    };
    // Check pandoc is available
    let pandoc_check = Command::new("pandoc").arg("--version").output();
    if pandoc_check.is_err() {
        return Err("pandoc is not installed. Install it from pandoc.org to enable document import.".to_string());
    }
    let safe_path_str = safe.to_string_lossy();
    let output = Command::new("pandoc")
        .args(["-f", format, "-t", "markdown-raw_html+pipe_tables+yaml_metadata_block", "--wrap=none", safe_path_str.as_ref()])
        .output()
        .map_err(|e| format!("pandoc error: {e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pandoc conversion failed: {err}"));
    }
    let markdown = String::from_utf8_lossy(&output.stdout).to_string();
    let warnings: Vec<String> = String::from_utf8_lossy(&output.stderr)
        .lines().filter(|l| !l.is_empty()).map(String::from).collect();
    Ok(ImportResponse { markdown, source_format: format.to_string(), warnings })
}
