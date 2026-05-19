use crate::{path_to_string, sha256_hex};
use chrono::Utc;
#[cfg(feature = "native-watch")]
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};
#[cfg(feature = "native-watch")]
use tauri::Emitter;
use tauri::State;

const MAX_WORKSPACE_SCAN_DEPTH: usize = 12;
const MAX_WORKSPACE_SCAN_ITEMS: usize = 2000;

#[derive(Default)]
pub(crate) struct FileWatcherState {
    watcher: Mutex<Option<ActiveFileWatcher>>,
}

struct ActiveFileWatcher {
    #[cfg(feature = "native-watch")]
    _watcher: RecommendedWatcher,
    #[allow(dead_code)]
    signature: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveFileRequest {
    pub(crate) path: String,
    pub(crate) text: String,
    pub(crate) expected_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RenameFileRequest {
    pub(crate) from: String,
    pub(crate) to: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DuplicateFileRequest {
    pub(crate) from: String,
    pub(crate) to: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct FileResponse {
    pub(crate) path: String,
    pub(crate) text: String,
    pub(crate) hash: String,
    pub(crate) modified: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct FileMetadata {
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) hash: Option<String>,
    pub(crate) modified: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WatchFileRequest {
    pub(crate) root: String,
    pub(crate) included: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct WatchFileResponse {
    pub(crate) paths: Vec<WatchedFileMetadata>,
    pub(crate) native_watcher: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct WatchedFileMetadata {
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) hash: Option<String>,
    pub(crate) modified: Option<String>,
    pub(crate) role: String,
}

#[cfg(feature = "native-watch")]
#[derive(Clone, Debug, Serialize)]
struct FileWatchEventPayload {
    paths: Vec<String>,
    kind: String,
}

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
pub(crate) fn read_file(path: String) -> Result<FileResponse, String> {
    let path_buf = PathBuf::from(path);
    let text = fs::read_to_string(&path_buf).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&path_buf).ok();
    Ok(FileResponse {
        path: path_to_string(&path_buf),
        hash: sha256_hex(text.as_bytes()),
        modified: metadata.and_then(modified_time),
        text,
    })
}

#[tauri::command]
pub(crate) fn open_file(path: String) -> Result<FileResponse, String> {
    read_file(path)
}

#[tauri::command]
pub(crate) fn save_file(request: SaveFileRequest) -> Result<FileResponse, String> {
    let path = PathBuf::from(&request.path);
    if let Some(expected_hash) = &request.expected_hash {
        if path.exists() {
            let current = fs::read(&path).map_err(|err| err.to_string())?;
            let current_hash = sha256_hex(&current);
            if &current_hash != expected_hash {
                return Err(
                    "File changed on disk since it was opened; resolve the external conflict before saving."
                        .to_string(),
                );
            }
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(&path, request.text.as_bytes()).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&path).ok();
    Ok(FileResponse {
        path: path_to_string(&path),
        hash: sha256_hex(request.text.as_bytes()),
        modified: metadata.and_then(modified_time),
        text: request.text,
    })
}

#[tauri::command]
pub(crate) fn save_file_as(request: SaveFileRequest) -> Result<FileResponse, String> {
    save_file(SaveFileRequest {
        expected_hash: None,
        ..request
    })
}

#[tauri::command]
pub(crate) fn rename_file(request: RenameFileRequest) -> Result<FileMetadata, String> {
    let from = PathBuf::from(&request.from);
    let to = PathBuf::from(&request.to);
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::rename(&from, &to).map_err(|err| err.to_string())?;
    file_metadata(path_to_string(&to))
}

#[tauri::command]
pub(crate) fn duplicate_file(request: DuplicateFileRequest) -> Result<FileResponse, String> {
    let from = PathBuf::from(&request.from);
    let to = PathBuf::from(&request.to);
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::copy(&from, &to).map_err(|err| err.to_string())?;
    read_file(path_to_string(&to))
}

#[tauri::command]
pub(crate) fn reveal_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg("-R").arg(&path);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer");
        command.arg(format!("/select,{path}"));
        command
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let target = PathBuf::from(&path)
            .parent()
            .map(path_to_string)
            .unwrap_or(path);
        let mut command = Command::new("xdg-open");
        command.arg(target);
        command
    };

    let status = command.status().map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Unable to reveal path; command exited with {status}"
        ))
    }
}

#[tauri::command]
pub(crate) fn file_metadata(path: String) -> Result<FileMetadata, String> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Ok(FileMetadata {
            path: path_to_string(&path_buf),
            exists: false,
            hash: None,
            modified: None,
        });
    }
    let text = fs::read(&path_buf).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&path_buf).ok();
    Ok(FileMetadata {
        path: path_to_string(&path_buf),
        exists: true,
        hash: Some(sha256_hex(&text)),
        modified: metadata.and_then(modified_time),
    })
}

#[tauri::command]
pub(crate) fn watch_file(request: WatchFileRequest) -> Result<WatchFileResponse, String> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for (path, role) in std::iter::once((request.root, "root"))
        .chain(request.included.into_iter().map(|path| (path, "include")))
    {
        let metadata = file_metadata(path)?;
        if seen.insert(metadata.path.clone()) {
            paths.push(WatchedFileMetadata {
                path: metadata.path,
                exists: metadata.exists,
                hash: metadata.hash,
                modified: metadata.modified,
                role: role.to_string(),
            });
        }
    }
    Ok(WatchFileResponse {
        paths,
        native_watcher: false,
    })
}

#[tauri::command]
#[cfg(feature = "native-watch")]
pub(crate) fn start_file_watcher(
    app: tauri::AppHandle,
    state: State<FileWatcherState>,
    request: WatchFileRequest,
) -> Result<WatchFileResponse, String> {
    let mut response = watch_file(request)?;
    let watch_paths = response
        .paths
        .iter()
        .filter(|metadata| metadata.exists)
        .map(|metadata| metadata.path.clone())
        .collect::<Vec<_>>();
    let signature = watch_paths.join("\n");
    let mut active = state
        .watcher
        .lock()
        .map_err(|_| "File watcher state lock poisoned.".to_string())?;

    if active
        .as_ref()
        .map(|watcher| watcher.signature.as_str() == signature.as_str())
        .unwrap_or(false)
    {
        response.native_watcher = !watch_paths.is_empty();
        return Ok(response);
    }

    *active = None;
    if watch_paths.is_empty() {
        return Ok(response);
    }

    let event_app = app.clone();
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| match result {
            Ok(event) => {
                if !notify_event_should_emit(&event.kind) {
                    return;
                }
                let payload = FileWatchEventPayload {
                    paths: event
                        .paths
                        .iter()
                        .map(|path| path_to_string(path))
                        .collect(),
                    kind: format!("{:?}", event.kind),
                };
                let _ = event_app.emit("neditor-file-watch-event", payload);
            }
            Err(error) => {
                let _ = event_app.emit("neditor-file-watch-error", error.to_string());
            }
        },
        Config::default(),
    )
    .map_err(|err| err.to_string())?;

    for path in &watch_paths {
        watcher
            .watch(Path::new(path), RecursiveMode::NonRecursive)
            .map_err(|err| err.to_string())?;
    }

    *active = Some(ActiveFileWatcher {
        _watcher: watcher,
        signature,
    });
    response.native_watcher = true;
    Ok(response)
}

#[tauri::command]
#[cfg(not(feature = "native-watch"))]
pub(crate) fn start_file_watcher(
    state: State<FileWatcherState>,
    request: WatchFileRequest,
) -> Result<WatchFileResponse, String> {
    let response = watch_file(request)?;
    let signature = response
        .paths
        .iter()
        .filter(|metadata| metadata.exists)
        .map(|metadata| metadata.path.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let mut active = state
        .watcher
        .lock()
        .map_err(|_| "File watcher state lock poisoned.".to_string())?;
    *active = Some(ActiveFileWatcher { signature });
    Ok(response)
}

#[tauri::command]
pub(crate) fn stop_file_watcher(state: State<FileWatcherState>) -> Result<(), String> {
    let mut active = state
        .watcher
        .lock()
        .map_err(|_| "File watcher state lock poisoned.".to_string())?;
    *active = None;
    Ok(())
}

#[cfg(feature = "native-watch")]
pub(crate) fn notify_event_should_emit(kind: &EventKind) -> bool {
    !matches!(kind, EventKind::Access(_))
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

fn modified_time(metadata: fs::Metadata) -> Option<String> {
    metadata
        .modified()
        .ok()
        .map(chrono::DateTime::<Utc>::from)
        .map(|time| time.to_rfc3339())
}
