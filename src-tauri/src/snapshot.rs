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
use std::{
    fs,
    path::{Path, PathBuf},
};

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

#[derive(Debug, Deserialize)]
pub(crate) struct SnapshotRestoreRequest {
    snapshot_path: String,
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
pub(crate) fn restore_snapshot(
    app: tauri::AppHandle,
    request: SnapshotRestoreRequest,
) -> Result<FileResponse, String> {
    let workspace_id = snapshot_workspace_id(request.file_path.as_deref());
    let root = snapshot_root(
        &app,
        request.file_path.as_deref(),
        &workspace_id,
        request.storage.as_deref(),
    )?;
    restore_snapshot_from_root(&request.snapshot_path, &root, request.file_path.as_deref())
}

fn restore_snapshot_from_root(
    snapshot_path: &str,
    root: &Path,
    active_file_path: Option<&str>,
) -> Result<FileResponse, String> {
    let snapshot_path = PathBuf::from(snapshot_path);
    if snapshot_path
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("md")
    {
        return Err("Snapshot restore requires a Markdown snapshot file.".to_string());
    }
    let root = root.canonicalize().map_err(|err| err.to_string())?;
    let snapshot_path = snapshot_path
        .canonicalize()
        .map_err(|err| err.to_string())?;
    if !snapshot_path.starts_with(&root) {
        return Err(
            "Snapshot restore path must stay inside the configured snapshot store.".to_string(),
        );
    }

    let metadata_path = snapshot_path.with_extension("json");
    let metadata_text = fs::read_to_string(&metadata_path)
        .map_err(|_| "Snapshot restore requires matching snapshot metadata.".to_string())?;
    let metadata = serde_json::from_str::<Value>(&metadata_text)
        .map_err(|_| "Snapshot restore metadata is not valid JSON.".to_string())?;
    validate_snapshot_source(&metadata, active_file_path)?;
    read_file(path_to_string(&snapshot_path))
}

fn validate_snapshot_source(
    metadata: &Value,
    active_file_path: Option<&str>,
) -> Result<(), String> {
    let source_path = metadata.get("sourcePath").and_then(Value::as_str);
    match (source_path, active_file_path) {
        (None, None) => Ok(()),
        (Some(source), Some(active)) if same_snapshot_source(source, active) => Ok(()),
        (Some(_), None) => Err(
            "Snapshot belongs to a saved document, but no saved document is active.".to_string(),
        ),
        (None, Some(_)) => {
            Err("Snapshot metadata does not match the active saved document.".to_string())
        }
        (Some(_), Some(_)) => {
            Err("Snapshot metadata does not match the active document.".to_string())
        }
    }
}

fn same_snapshot_source(source_path: &str, active_file_path: &str) -> bool {
    if source_path == active_file_path {
        return true;
    }
    let source = PathBuf::from(source_path);
    let active = PathBuf::from(active_file_path);
    match (source.canonicalize(), active.canonicalize()) {
        (Ok(source), Ok(active)) => source == active,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("neditor-{prefix}-{unique}"))
    }

    #[test]
    fn snapshot_restore_is_scoped_to_active_document_store() {
        let root = temp_root("snapshot-restore");
        fs::create_dir_all(&root).expect("create snapshot root");
        let doc = root.join("doc.md");
        let snapshot = root.join("snap.md");
        let metadata = root.join("snap.json");
        fs::write(&doc, "# Current\n").expect("write doc");
        fs::write(&snapshot, "# Previous\n").expect("write snapshot");
        fs::write(
            &metadata,
            json!({
                "hash": sha256_hex(b"# Previous\n"),
                "sourcePath": path_to_string(&doc),
                "label": "manual"
            })
            .to_string(),
        )
        .expect("write metadata");

        let restored = restore_snapshot_from_root(
            &path_to_string(&snapshot),
            &root,
            Some(&path_to_string(&doc)),
        )
        .expect("restore snapshot");
        assert_eq!(restored.text, "# Previous\n");
        assert_eq!(restored.hash, sha256_hex(b"# Previous\n"));
        fs::remove_dir_all(root).expect("clean snapshot restore test");
    }

    #[test]
    fn snapshot_restore_rejects_out_of_scope_and_mismatched_sources() {
        let root = temp_root("snapshot-restore-scope");
        let outside = temp_root("snapshot-restore-outside");
        fs::create_dir_all(&root).expect("create snapshot root");
        fs::create_dir_all(&outside).expect("create outside root");
        let active_doc = root.join("doc.md");
        let other_doc = root.join("other.md");
        let snapshot = root.join("snap.md");
        let metadata = root.join("snap.json");
        let outside_snapshot = outside.join("outside.md");
        fs::write(&active_doc, "# Current\n").expect("write active doc");
        fs::write(&other_doc, "# Other\n").expect("write other doc");
        fs::write(&snapshot, "# Previous\n").expect("write snapshot");
        fs::write(
            &metadata,
            json!({ "sourcePath": path_to_string(&other_doc) }).to_string(),
        )
        .expect("write metadata");
        fs::write(&outside_snapshot, "# Outside\n").expect("write outside snapshot");
        fs::write(
            outside_snapshot.with_extension("json"),
            json!({ "sourcePath": path_to_string(&active_doc) }).to_string(),
        )
        .expect("write outside metadata");

        let mismatch = restore_snapshot_from_root(
            &path_to_string(&snapshot),
            &root,
            Some(&path_to_string(&active_doc)),
        )
        .expect_err("mismatched source should be rejected");
        assert!(mismatch.contains("active document"));

        let outside_error = restore_snapshot_from_root(
            &path_to_string(&outside_snapshot),
            &root,
            Some(&path_to_string(&active_doc)),
        )
        .expect_err("outside snapshot should be rejected");
        assert!(outside_error.contains("snapshot store"));

        let wrong_extension = restore_snapshot_from_root(
            &path_to_string(&metadata),
            &root,
            Some(&path_to_string(&active_doc)),
        )
        .expect_err("metadata file should not restore as Markdown");
        assert!(wrong_extension.contains("Markdown snapshot"));

        fs::remove_dir_all(root).expect("clean snapshot scope test");
        fs::remove_dir_all(outside).expect("clean outside snapshot scope test");
    }
}
