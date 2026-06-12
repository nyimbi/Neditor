use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{env, fs, io::Write, path::PathBuf, process};

fn neditor_ipc_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neditor")
}

fn instance_file() -> PathBuf {
    neditor_ipc_dir().join("instance.json")
}

fn queue_file() -> PathBuf {
    neditor_ipc_dir().join("cli-open-queue.jsonl")
}

#[derive(Debug, Serialize, Deserialize)]
struct InstanceInfo {
    pid: u32,
    started_at: String,
}

/// Write this process's PID so `ned` can detect a running instance.
pub(crate) fn register_neditor_instance() {
    let dir = neditor_ipc_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("[neditor cli_ipc] WARNING: cannot create IPC dir '{}': {e}", dir.display());
        return;
    }
    let info = InstanceInfo {
        pid: process::id(),
        started_at: Utc::now().to_rfc3339(),
    };
    match serde_json::to_string(&info) {
        Ok(json) => {
            if let Err(e) = fs::write(instance_file(), json) {
                eprintln!("[neditor cli_ipc] WARNING: cannot write instance.json: {e}");
            }
        }
        Err(e) => eprintln!("[neditor cli_ipc] WARNING: cannot serialize instance info: {e}"),
    }
}

/// Returns the PID of the running NEditor if one is alive, otherwise None.
pub(crate) fn running_instance_pid() -> Option<u32> {
    let content = fs::read_to_string(instance_file()).ok()?;
    let info: InstanceInfo = serde_json::from_str(&content).ok()?;
    if info.pid == process::id() {
        return None; // don't detect ourselves
    }
    if pid_alive(info.pid) {
        Some(info.pid)
    } else {
        None
    }
}

fn pid_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Append file paths to the open queue for the running instance to pick up.
pub(crate) fn queue_paths_for_open(paths: &[String]) -> Result<(), String> {
    let dir = neditor_ipc_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create IPC dir: {e}"))?;
    // Bound the queue: refuse to grow beyond 100 entries to prevent indefinite growth
    // if NEditor crashes and drain never runs.
    const MAX_QUEUE_ENTRIES: usize = 100;
    let qf = queue_file();
    if qf.exists() {
        let existing = fs::read_to_string(&qf).unwrap_or_default();
        let count = existing.lines().filter(|l| !l.trim().is_empty()).count();
        if count + paths.len() > MAX_QUEUE_ENTRIES {
            return Err(format!(
                "CLI open queue is full ({count}/{MAX_QUEUE_ENTRIES}). \
                 Start NEditor to drain the queue."
            ));
        }
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&qf)
        .map_err(|e| e.to_string())?;
    for path in paths {
        let line = serde_json::to_string(path).map_err(|e| format!("Cannot serialize path: {e}"))?;
        writeln!(file, "{line}").map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Read all queued paths and clear the queue. Called by the running NEditor on focus.
///
/// Uses an atomic rename-then-replace strategy to clear the queue: the queue file is
/// renamed to a temp path before parsing, so a crash mid-parse cannot cause re-replay.
/// If the rename fails (e.g. permissions, cross-device), we fall back to truncating in
/// place and log the error rather than silently swallowing it.
#[tauri::command]
pub(crate) fn drain_cli_open_queue() -> Vec<String> {
    let path = queue_file();
    let content = match fs::read_to_string(&path) {
        Ok(c) if !c.trim().is_empty() => c,
        _ => return Vec::new(),
    };

    // Atomically remove the queue by renaming it to a sibling temp file, then
    // deleting the temp file. This guarantees that even if we crash while
    // parsing, the original queue file is gone and entries are not replayed.
    let tmp_path = path.with_extension("jsonl.drain_tmp");
    if let Err(rename_err) = fs::rename(&path, &tmp_path) {
        // Rename failed (e.g. cross-device or permissions). Fall back to
        // truncating in place. Log so the failure is observable.
        eprintln!(
            "[neditor cli_ipc] WARNING: could not rename queue file for atomic drain \
             ({rename_err}); falling back to truncate"
        );
        if let Err(trunc_err) = fs::write(&path, "") {
            // Truncation also failed: the queue is in an ambiguous state.
            // Return empty to avoid replaying entries we already read; the
            // caller will not act on stale data, and the next poll cycle will
            // re-evaluate.
            eprintln!(
                "[neditor cli_ipc] ERROR: could not clear queue file after read \
                 ({trunc_err}); entries will not be replayed this cycle"
            );
            return Vec::new();
        }
    } else {
        // Best-effort removal of the temp file; ignore errors (it will be
        // overwritten on the next drain cycle if it lingers).
        let _ = fs::remove_file(&tmp_path);
    }

    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<String>(l).ok())
        .collect()
}

/// Returns the path of the queue file so the frontend can watch it.
#[tauri::command]
pub(crate) fn cli_queue_file_path() -> String {
    queue_file().to_string_lossy().to_string()
}

/// Register this instance (Tauri command wrapper).
#[tauri::command]
pub(crate) fn register_instance() {
    register_neditor_instance();
}
