use crate::{
    path_to_string, read_file, sha256_hex,
    snapshot_metadata::snapshot_document_metadata,
    snapshot_storage::{
        ensure_project_snapshot_gitignore, snapshot_root, snapshot_storage_is_project_local,
        snapshot_workspace_id,
    },
    FileResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;

#[derive(Debug, Deserialize)]
pub(crate) struct SnapshotRequest {
    text: String,
    file_path: Option<String>,
    label: Option<String>,
    storage: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SnapshotListRequest {
    file_path: Option<String>,
    storage: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SnapshotListItem {
    snapshot_path: String,
    metadata_path: String,
    hash: Option<String>,
    created_at: Option<String>,
    label: Option<String>,
    document_version: Option<String>,
    status: Option<String>,
    author: Option<String>,
    include_graph_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SnapshotResponse {
    snapshot_path: String,
    metadata_path: String,
    hash: String,
}

#[tauri::command]
pub(crate) fn create_snapshot(
    app: tauri::AppHandle,
    request: SnapshotRequest,
) -> Result<SnapshotResponse, String> {
    let source_hash = sha256_hex(request.text.as_bytes());
    let workspace_id = snapshot_workspace_id(request.file_path.as_deref());
    let label = request
        .label
        .unwrap_or_else(|| "snapshot".to_string())
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>();
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let root = snapshot_root(
        &app,
        request.file_path.as_deref(),
        &workspace_id,
        request.storage.as_deref(),
    )?;
    fs::create_dir_all(&root).map_err(|err| err.to_string())?;
    if snapshot_storage_is_project_local(request.storage.as_deref()) {
        ensure_project_snapshot_gitignore(request.file_path.as_deref())?;
    }
    let snapshot_path = root.join(format!("{timestamp}-{label}.md"));
    let metadata_path = root.join(format!("{timestamp}-{label}.json"));
    fs::write(&snapshot_path, request.text.as_bytes()).map_err(|err| err.to_string())?;
    let document_metadata = snapshot_document_metadata(&request.text);
    let metadata = json!({
        "hash": source_hash,
        "createdAt": Utc::now().to_rfc3339(),
        "sourcePath": request.file_path,
        "label": label,
        "documentVersion": document_metadata.document_version,
        "status": document_metadata.status,
        "author": document_metadata.author,
        "includeGraphHash": document_metadata.include_graph_hash
    });
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(SnapshotResponse {
        snapshot_path: path_to_string(&snapshot_path),
        metadata_path: path_to_string(&metadata_path),
        hash: source_hash,
    })
}

#[tauri::command]
pub(crate) fn list_snapshots(
    app: tauri::AppHandle,
    request: SnapshotListRequest,
) -> Result<Vec<SnapshotListItem>, String> {
    let workspace_id = snapshot_workspace_id(request.file_path.as_deref());
    let root = snapshot_root(
        &app,
        request.file_path.as_deref(),
        &workspace_id,
        request.storage.as_deref(),
    )?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(&root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let metadata_text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let metadata = serde_json::from_str::<Value>(&metadata_text).unwrap_or_else(|_| json!({}));
        let snapshot_path = path.with_extension("md");
        items.push(SnapshotListItem {
            snapshot_path: path_to_string(&snapshot_path),
            metadata_path: path_to_string(&path),
            hash: metadata
                .get("hash")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            created_at: metadata
                .get("createdAt")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            label: metadata
                .get("label")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            document_version: metadata
                .get("documentVersion")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            status: metadata
                .get("status")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            author: metadata
                .get("author")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            include_graph_hash: metadata
                .get("includeGraphHash")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        });
    }
    items.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(items)
}

#[tauri::command]
pub(crate) fn restore_snapshot(snapshot_path: String) -> Result<FileResponse, String> {
    read_file(snapshot_path)
}
