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
    let root = app_snapshot_root(&app, Some(&workspace_id))?;
    fs::create_dir_all(&root).map_err(|err| err.to_string())?;
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

struct SnapshotDocumentMetadata {
    document_version: Option<String>,
    status: Option<String>,
    author: Option<String>,
    include_graph_hash: String,
}

fn snapshot_document_metadata(text: &str) -> SnapshotDocumentMetadata {
    let metadata = snapshot_front_matter(text);
    SnapshotDocumentMetadata {
        document_version: metadata
            .as_ref()
            .and_then(|value| metadata_string(value, "version")),
        status: metadata
            .as_ref()
            .and_then(|value| metadata_string(value, "status")),
        author: metadata
            .as_ref()
            .and_then(|value| metadata_string(value, "author")),
        include_graph_hash: include_graph_hash(text),
    }
}

fn snapshot_front_matter(text: &str) -> Option<Value> {
    if !text.starts_with("---\n") {
        return None;
    }
    let lines = text.lines().collect::<Vec<_>>();
    let end_index = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, line)| (line.trim() == "---").then_some(index))?;
    let yaml = lines[1..end_index].join("\n");
    serde_yaml::from_str::<Value>(&yaml).ok()
}

fn metadata_string(metadata: &Value, key: &str) -> Option<String> {
    metadata.get(key).and_then(|value| match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    })
}

fn include_graph_hash(text: &str) -> String {
    let mut includes = text
        .lines()
        .filter_map(snapshot_include_target)
        .collect::<Vec<_>>();
    includes.sort();
    sha256_hex(includes.join("\n").as_bytes())
}

fn snapshot_include_target(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if let Some(path) = trimmed.strip_prefix("!include ") {
        return Some(path.trim().to_string());
    }
    if let Some(path) = trimmed
        .strip_prefix("{{include ")
        .and_then(|value| value.strip_suffix("}}"))
    {
        return Some(path.trim().to_string());
    }
    if let Some(path) = trimmed
        .strip_prefix("<!-- include:")
        .and_then(|value| value.strip_suffix("-->"))
    {
        return Some(path.trim().to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_metadata_captures_release_fields_and_include_hash() {
        let text = "---\ntitle: Report\nversion: 2.0.0\nstatus: approved\nauthor: Strategy Team\n---\n# Report\n!include chapters/a.md\n{{include chapters/b.md}}\n<!-- include: chapters/c.md -->\n";
        let metadata = snapshot_document_metadata(text);

        assert_eq!(metadata.document_version.as_deref(), Some("2.0.0"));
        assert_eq!(metadata.status.as_deref(), Some("approved"));
        assert_eq!(metadata.author.as_deref(), Some("Strategy Team"));
        assert_ne!(metadata.include_graph_hash, sha256_hex("".as_bytes()));
    }
}
