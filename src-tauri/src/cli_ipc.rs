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
    let _ = fs::create_dir_all(&dir);
    let info = InstanceInfo {
        pid: process::id(),
        started_at: Utc::now().to_rfc3339(),
    };
    if let Ok(json) = serde_json::to_string(&info) {
        let _ = fs::write(instance_file(), json);
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
    let _ = fs::create_dir_all(&dir);
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(queue_file())
        .map_err(|e| e.to_string())?;
    for path in paths {
        let line = serde_json::to_string(path).unwrap_or_default();
        writeln!(file, "{line}").map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Read all queued paths and clear the queue. Called by the running NEditor on focus.
#[tauri::command]
pub(crate) fn drain_cli_open_queue() -> Vec<String> {
    let path = queue_file();
    let content = match fs::read_to_string(&path) {
        Ok(c) if !c.trim().is_empty() => c,
        _ => return Vec::new(),
    };
    // Clear the queue before parsing so a crash doesn't replay entries
    let _ = fs::write(&path, "");
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
