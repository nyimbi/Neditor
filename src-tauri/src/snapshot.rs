use crate::{path_to_string, read_file, sha256_hex, FileResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
use tauri::Manager;

#[derive(Debug, Deserialize)]
pub(crate) struct SnapshotRequest {
    text: String,
    file_path: Option<String>,
    label: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SnapshotListItem {
    snapshot_path: String,
    metadata_path: String,
    hash: Option<String>,
    created_at: Option<String>,
    label: Option<String>,
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
    let root = app_snapshot_root(&app, Some(&workspace_id))?;
    fs::create_dir_all(&root).map_err(|err| err.to_string())?;
    let snapshot_path = root.join(format!("{timestamp}-{label}.md"));
    let metadata_path = root.join(format!("{timestamp}-{label}.json"));
    fs::write(&snapshot_path, request.text.as_bytes()).map_err(|err| err.to_string())?;
    let metadata = json!({
        "hash": source_hash,
        "createdAt": Utc::now().to_rfc3339(),
        "sourcePath": request.file_path,
        "label": label
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
    file_path: Option<String>,
) -> Result<Vec<SnapshotListItem>, String> {
    let root = app_snapshot_root(&app, file_path.as_deref())?;
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
        });
    }
    items.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(items)
}

#[tauri::command]
pub(crate) fn restore_snapshot(snapshot_path: String) -> Result<FileResponse, String> {
    read_file(snapshot_path)
}

fn snapshot_workspace_id(file_path: Option<&str>) -> String {
    file_path
        .map(|path| sha256_hex(path.as_bytes()))
        .unwrap_or_else(|| "unsaved".to_string())
}

fn app_snapshot_root(
    app: &tauri::AppHandle,
    file_path_or_id: Option<&str>,
) -> Result<PathBuf, String> {
    let app_data = app.path().app_data_dir().map_err(|err| err.to_string())?;
    let workspace_id = file_path_or_id
        .map(|value| {
            if value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
                value.to_string()
            } else {
                snapshot_workspace_id(Some(value))
            }
        })
        .unwrap_or_else(|| snapshot_workspace_id(None));
    Ok(app_data.join("snapshots").join(workspace_id))
}
