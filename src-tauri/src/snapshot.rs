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

#[derive(Debug, Deserialize)]
pub(crate) struct SnapshotDeleteRequest {
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
    // Enforce snapshot count limit: prune oldest when over 200 to prevent unbounded growth.
    const MAX_SNAPSHOTS: usize = 200;
    if let Ok(entries) = fs::read_dir(&root) {
        let mut md_files: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            .collect();
        if md_files.len() >= MAX_SNAPSHOTS {
            md_files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));
            let remove_count = (md_files.len() + 1).saturating_sub(MAX_SNAPSHOTS);
            for entry in md_files.into_iter().take(remove_count) {
                let _ = fs::remove_file(entry.path());
                let _ = fs::remove_file(entry.path().with_extension("json"));
            }
        }
    }
    let snapshot_path = root.join(format!("{timestamp}-{label}.md"));
    let metadata_path = root.join(format!("{timestamp}-{label}.json"));
    // Write atomically: use temp files + rename so a crash cannot leave a partial snapshot.
    let tmp_md = snapshot_path.with_extension("md.tmp");
    let tmp_json = metadata_path.with_extension("json.tmp");
    fs::write(&tmp_md, request.text.as_bytes()).map_err(|err| err.to_string())?;
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
        &tmp_json,
        serde_json::to_vec_pretty(&metadata).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    // Atomic promotion: rename both temp files into their final names.
    // If either rename fails, clean up and surface the error.
    if let Err(e) = fs::rename(&tmp_md, &snapshot_path) {
        let _ = fs::remove_file(&tmp_md);
        let _ = fs::remove_file(&tmp_json);
        return Err(format!("Cannot commit snapshot file: {e}"));
    }
    if let Err(e) = fs::rename(&tmp_json, &metadata_path) {
        // The .md is already promoted; remove it to avoid an orphan.
        let _ = fs::remove_file(&snapshot_path);
        let _ = fs::remove_file(&tmp_json);
        return Err(format!("Cannot commit snapshot metadata: {e}"));
    }
    // Prune old auto-snapshots so the directory does not grow without bound.
    let _ = prune_old_snapshots(&root, DEFAULT_SNAPSHOT_MAX_COUNT);
    Ok(SnapshotResponse {
        snapshot_path: path_to_string(&snapshot_path),
        metadata_path: path_to_string(&metadata_path),
        hash: source_hash,
    })
}

/// Default maximum number of auto-snapshots retained per workspace directory.
pub(crate) const DEFAULT_SNAPSHOT_MAX_COUNT: usize = 50;

/// Delete the oldest `.md` / `.json` snapshot pairs beyond `max_count` in `root`.
/// Snapshots are ordered by filename (which starts with a UTC timestamp), so
/// lexicographic sort gives chronological order.  Returns the number of pairs
/// removed.  Errors from individual deletes are ignored so a single locked file
/// does not abort the whole prune.
pub(crate) fn prune_old_snapshots(root: &Path, max_count: usize) -> Result<usize, String> {
    if !root.exists() {
        return Ok(0);
    }
    // Collect all .json sidecar files — each corresponds to one snapshot pair.
    let mut json_paths: Vec<PathBuf> = fs::read_dir(root)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    if json_paths.len() <= max_count {
        return Ok(0);
    }
    // Sort ascending (oldest first) — filenames begin with YYYYMMDDTHHMMSSz so
    // lexicographic order is chronological.
    json_paths.sort();
    let excess = json_paths.len() - max_count;
    let mut removed = 0usize;
    for json_path in json_paths.iter().take(excess) {
        let md_path = json_path.with_extension("md");
        // Attempt both deletes; ignore individual errors (e.g. already gone).
        let _ = fs::remove_file(&md_path);
        if fs::remove_file(json_path).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
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
        // Skip corrupt/truncated metadata files instead of silently substituting {}
        let metadata = match serde_json::from_str::<Value>(&metadata_text) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[neditor snapshot] WARNING: skipping corrupt metadata '{}': {e}", path.display());
                continue;
            }
        };
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

/// Delete a single snapshot pair (`.md` + `.json`).  The path must stay inside
/// the configured snapshot root for the workspace to prevent directory traversal.
#[tauri::command]
pub(crate) fn delete_snapshot(
    app: tauri::AppHandle,
    request: SnapshotDeleteRequest,
) -> Result<(), String> {
    let workspace_id = snapshot_workspace_id(request.file_path.as_deref());
    let root = snapshot_root(
        &app,
        request.file_path.as_deref(),
        &workspace_id,
        request.storage.as_deref(),
    )?;
    let root = root.canonicalize().map_err(|err| err.to_string())?;
    let snapshot_path = PathBuf::from(&request.snapshot_path);
    // Accept either the .md or the .json path; normalise to .md.
    let snapshot_path = if snapshot_path.extension().and_then(|e| e.to_str()) == Some("json") {
        snapshot_path.with_extension("md")
    } else {
        snapshot_path
    };
    if snapshot_path.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err("delete_snapshot requires a .md or .json snapshot path.".to_string());
    }
    let snapshot_path = snapshot_path
        .canonicalize()
        .map_err(|err| err.to_string())?;
    if !snapshot_path.starts_with(&root) {
        return Err(
            "Snapshot delete path must stay inside the configured snapshot store.".to_string(),
        );
    }
    let metadata_path = snapshot_path.with_extension("json");
    // Remove both files; ignore "not found" errors for each independently.
    if let Err(err) = fs::remove_file(&snapshot_path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            return Err(err.to_string());
        }
    }
    if let Err(err) = fs::remove_file(&metadata_path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            return Err(err.to_string());
        }
    }
    Ok(())
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

    #[test]
    fn prune_old_snapshots_removes_oldest_beyond_max() {
        let root = temp_root("prune-snapshots");
        fs::create_dir_all(&root).expect("create snapshot root");
        // Write 5 snapshot pairs with ascending timestamps in filenames.
        for i in 0..5u8 {
            let base = format!("2024010{i}T000000Z-auto");
            fs::write(root.join(format!("{base}.md")), format!("# snap {i}"))
                .expect("write snapshot");
            fs::write(root.join(format!("{base}.json")), format!("{{\"label\":\"{i}\"}}"))
                .expect("write metadata");
        }
        // Prune keeping only 3.
        let removed = prune_old_snapshots(&root, 3).expect("prune snapshots");
        assert_eq!(removed, 2, "expected 2 pairs removed");
        // The 2 oldest (index 0 and 1) should be gone.
        assert!(!root.join("20240100T000000Z-auto.md").exists());
        assert!(!root.join("20240100T000000Z-auto.json").exists());
        assert!(!root.join("20240101T000000Z-auto.md").exists());
        assert!(!root.join("20240101T000000Z-auto.json").exists());
        // The 3 newest must survive.
        for i in 2..5u8 {
            let base = format!("2024010{i}T000000Z-auto");
            assert!(root.join(format!("{base}.md")).exists(), "snap {i} .md missing");
            assert!(root.join(format!("{base}.json")).exists(), "snap {i} .json missing");
        }
        fs::remove_dir_all(root).expect("clean prune test");
    }

    #[test]
    fn prune_old_snapshots_noop_when_under_limit() {
        let root = temp_root("prune-noop");
        fs::create_dir_all(&root).expect("create snapshot root");
        for i in 0..3u8 {
            let base = format!("2024010{i}T000000Z-auto");
            fs::write(root.join(format!("{base}.md")), "x").expect("write");
            fs::write(root.join(format!("{base}.json")), "{}").expect("write");
        }
        let removed = prune_old_snapshots(&root, 5).expect("prune no-op");
        assert_eq!(removed, 0);
        fs::remove_dir_all(root).expect("clean prune noop test");
    }
}
