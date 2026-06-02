use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub event: String,
    pub document_path: Option<String>,
    pub document_title: Option<String>,
    pub author: Option<String>,
    pub detail: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordAuditRequest {
    pub workspace_root: String,
    pub event: String,
    pub document_path: Option<String>,
    pub document_title: Option<String>,
    pub author: Option<String>,
    pub detail: Option<String>,
    pub status: Option<String>,
}

#[tauri::command]
pub(crate) fn record_audit_event(request: RecordAuditRequest) -> Result<(), String> {
    let audit_dir = PathBuf::from(&request.workspace_root).join(".neditor");
    fs::create_dir_all(&audit_dir).map_err(|e| e.to_string())?;
    let audit_path = audit_dir.join("audit.jsonl");
    let entry = AuditEntry {
        timestamp: Utc::now().to_rfc3339(),
        event: request.event,
        document_path: request.document_path,
        document_title: request.document_title,
        author: request.author,
        detail: request.detail,
        status: request.status,
    };
    let line = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new().create(true).append(true).open(&audit_path).map_err(|e| e.to_string())?;
    writeln!(file, "{line}").map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub(crate) fn read_audit_log(workspace_root: String, limit: usize) -> Result<Vec<AuditEntry>, String> {
    let audit_path = PathBuf::from(&workspace_root).join(".neditor").join("audit.jsonl");
    if !audit_path.exists() { return Ok(Vec::new()); }
    let content = fs::read_to_string(&audit_path).map_err(|e| e.to_string())?;
    let entries: Vec<AuditEntry> = content.lines()
        .filter(|l| !l.is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    let limit = limit.min(1000);
    Ok(entries.into_iter().rev().take(limit).collect())
}
