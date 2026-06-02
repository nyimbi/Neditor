use crate::sha256_hex;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;

pub(crate) fn snapshot_workspace_id(file_path: Option<&str>) -> String {
    file_path
        .map(|path| {
            let canonical = PathBuf::from(path)
                .canonicalize()
                .ok()
                .and_then(|p| p.to_str().map(str::to_string));
            let bytes = canonical.as_deref().unwrap_or(path).as_bytes();
            sha256_hex(bytes)
        })
        .unwrap_or_else(|| "unsaved".to_string())
}

pub(crate) fn snapshot_root(
    app: &tauri::AppHandle,
    file_path: Option<&str>,
    workspace_id: &str,
    storage: Option<&str>,
) -> Result<PathBuf, String> {
    if snapshot_storage_is_project_local(storage) {
        return project_snapshot_root(file_path, workspace_id);
    }
    app_snapshot_root(app, Some(workspace_id))
}

pub(crate) fn snapshot_storage_is_project_local(storage: Option<&str>) -> bool {
    matches!(storage, Some("project-local"))
}

fn project_snapshot_root(file_path: Option<&str>, workspace_id: &str) -> Result<PathBuf, String> {
    let path = file_path.ok_or_else(|| {
        "Project-local snapshots require the document to be saved first.".to_string()
    })?;
    let document_path = PathBuf::from(path);
    let folder = document_path.parent().ok_or_else(|| {
        "Project-local snapshots require a document with a parent folder.".to_string()
    })?;
    Ok(folder.join(".neditor").join("snapshots").join(workspace_id))
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

pub(crate) fn ensure_project_snapshot_gitignore(file_path: Option<&str>) -> Result<(), String> {
    let Some(path) = file_path else {
        return Ok(());
    };
    let Some(folder) = Path::new(path).parent() else {
        return Ok(());
    };
    let root = git_root_for_path(folder).unwrap_or_else(|| folder.to_path_buf());
    let gitignore = root.join(".gitignore");
    let existing = fs::read_to_string(&gitignore).unwrap_or_default();
    let Some(updated) = gitignore_with_neditor_entry(&existing) else {
        return Ok(());
    };
    fs::write(gitignore, updated).map_err(|err| err.to_string())
}

fn git_root_for_path(folder: &Path) -> Option<PathBuf> {
    folder
        .ancestors()
        .find(|candidate| candidate.join(".git").exists())
        .map(Path::to_path_buf)
}

fn gitignore_with_neditor_entry(existing: &str) -> Option<String> {
    if existing
        .lines()
        .map(str::trim)
        .any(|line| line == ".neditor/" || line == ".neditor")
    {
        return None;
    }
    let mut updated = existing.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(".neditor/\n");
    Some(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_snapshot_gitignore_entry_is_idempotent() {
        assert_eq!(
            gitignore_with_neditor_entry("target\n").as_deref(),
            Some("target\n.neditor/\n")
        );
        assert_eq!(
            gitignore_with_neditor_entry("target").as_deref(),
            Some("target\n.neditor/\n")
        );
        assert!(gitignore_with_neditor_entry("target\n.neditor/\n").is_none());
        assert!(gitignore_with_neditor_entry(".neditor\n").is_none());
    }
}
