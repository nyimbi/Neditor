use crate::{filesystem::file_metadata, path_to_string, source_mapping::parse_include_directive};
#[cfg(feature = "native-watch")]
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
#[cfg(feature = "native-watch")]
use tauri::Emitter;
use tauri::State;

const MAX_WATCH_INCLUDE_DEPTH: usize = 12;

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
pub(crate) struct WatchFileRequest {
    pub(crate) root: String,
    pub(crate) included: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct WatchFileResponse {
    pub(crate) paths: Vec<WatchedFileMetadata>,
    pub(crate) native_watcher: bool,
    pub(crate) watcher_error: Option<String>,
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

#[tauri::command]
pub(crate) fn watch_file(request: WatchFileRequest) -> Result<WatchFileResponse, String> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for (path, role) in expanded_watch_paths(request) {
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
        watcher_error: None,
    })
}

fn expanded_watch_paths(request: WatchFileRequest) -> Vec<(String, &'static str)> {
    let mut output = Vec::new();
    let mut discovered = HashSet::new();
    let root = PathBuf::from(&request.root);
    output.push((request.root.clone(), "root"));
    collect_nested_watch_includes(&root, 0, &mut discovered, &mut output);
    for included in request.included {
        let include_path = PathBuf::from(&included);
        output.push((included, "include"));
        collect_nested_watch_includes(&include_path, 0, &mut discovered, &mut output);
    }
    output
}

fn collect_nested_watch_includes(
    path: &Path,
    depth: usize,
    discovered: &mut HashSet<String>,
    output: &mut Vec<(String, &'static str)>,
) {
    if depth >= MAX_WATCH_INCLUDE_DEPTH {
        return;
    }
    let normalized = path_to_string(path);
    if !discovered.insert(normalized) {
        return;
    }
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };
    let base_dir = path.parent().unwrap_or_else(|| Path::new(""));
    for line in text.lines() {
        let Some(target) = parse_include_directive(line) else {
            continue;
        };
        let child = resolve_watch_include_path(base_dir, target);
        output.push((path_to_string(&child), "include"));
        collect_nested_watch_includes(&child, depth + 1, discovered, output);
    }
}

fn resolve_watch_include_path(base_dir: &Path, target: &str) -> PathBuf {
    let target_path = PathBuf::from(target);
    if target_path.is_absolute() {
        target_path
    } else {
        base_dir.join(target_path)
    }
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
    let mut watcher = match RecommendedWatcher::new(
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
    ) {
        Ok(watcher) => watcher,
        Err(err) => {
            response.watcher_error = Some(format!("Native file watcher unavailable: {err}"));
            return Ok(response);
        }
    };

    for path in &watch_paths {
        if let Err(err) = watcher.watch(Path::new(path), RecursiveMode::NonRecursive) {
            response.watcher_error = Some(format!(
                "Native file watcher unavailable for {}: {err}",
                path
            ));
            return Ok(response);
        }
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
