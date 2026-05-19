use crate::path_to_string;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const MAX_WORKSPACE_SCAN_DEPTH: usize = 12;
const MAX_WORKSPACE_SCAN_ITEMS: usize = 2000;

#[derive(Debug, Deserialize)]
pub(crate) struct WorkspaceFileRequest {
    pub(crate) root: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct WorkspaceFileEntry {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) relative_path: String,
    pub(crate) kind: String,
    pub(crate) depth: usize,
}

#[tauri::command]
pub(crate) fn list_workspace_files(
    request: WorkspaceFileRequest,
) -> Result<Vec<WorkspaceFileEntry>, String> {
    let root = PathBuf::from(&request.root);
    if !root.exists() {
        return Err(format!("Workspace root does not exist: {}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!(
            "Workspace root is not a folder: {}",
            root.display()
        ));
    }

    let canonical_root = root.canonicalize().unwrap_or(root);
    let mut entries = Vec::new();
    scan_workspace_dir(&canonical_root, &canonical_root, 0, &mut entries)?;
    entries.sort_by(|left, right| {
        left.relative_path
            .to_ascii_lowercase()
            .cmp(&right.relative_path.to_ascii_lowercase())
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    Ok(entries)
}

fn scan_workspace_dir(
    root: &Path,
    dir: &Path,
    depth: usize,
    entries: &mut Vec<WorkspaceFileEntry>,
) -> Result<(), String> {
    if depth >= MAX_WORKSPACE_SCAN_DEPTH || entries.len() >= MAX_WORKSPACE_SCAN_ITEMS {
        return Ok(());
    }

    let mut children = fs::read_dir(dir)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        if entries.len() >= MAX_WORKSPACE_SCAN_ITEMS {
            break;
        }
        let path = child.path();
        let name = child.file_name().to_string_lossy().to_string();
        if should_skip_workspace_path(&path, &name) {
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .map(path_to_string)
            .unwrap_or_else(|_| path_to_string(&path));
        if path.is_dir() {
            entries.push(WorkspaceFileEntry {
                path: path_to_string(&path),
                name,
                relative_path,
                kind: "directory".to_string(),
                depth,
            });
            scan_workspace_dir(root, &path, depth + 1, entries)?;
        } else if is_workspace_document(&path) {
            let kind = path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.to_ascii_lowercase())
                .unwrap_or_else(|| "file".to_string());
            entries.push(WorkspaceFileEntry {
                path: path_to_string(&path),
                name,
                relative_path,
                kind,
                depth,
            });
        }
    }

    Ok(())
}

fn should_skip_workspace_path(path: &Path, name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }
    if path.is_dir()
        && matches!(
            name,
            "node_modules" | "target" | "dist" | "build" | ".git" | ".pnpm-store"
        )
    {
        return true;
    }
    false
}

fn is_workspace_document(path: &Path) -> bool {
    let Some(extension) = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
    else {
        return false;
    };
    matches!(
        extension.as_str(),
        "md" | "markdown"
            | "mdown"
            | "txt"
            | "text"
            | "csv"
            | "tsv"
            | "json"
            | "jsonc"
            | "yaml"
            | "yml"
            | "toml"
            | "bib"
            | "csl"
            | "tex"
            | "html"
            | "css"
            | "js"
            | "ts"
            | "vue"
            | "rs"
    )
}
