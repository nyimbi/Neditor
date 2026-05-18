use crate::{path_to_string, read_file, FileResponse};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Serialize)]
pub(crate) struct GitStatus {
    pub(crate) inside_repo: bool,
    pub(crate) branch: Option<String>,
    pub(crate) dirty: bool,
    pub(crate) summary: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitPathRequest {
    pub(crate) path: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitCommitRequest {
    pub(crate) path: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitTagRequest {
    pub(crate) path: String,
    pub(crate) tag: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GitRestoreRequest {
    pub(crate) path: String,
    pub(crate) revision: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct GitHistoryEntry {
    pub(crate) revision: String,
    pub(crate) author: String,
    pub(crate) date: String,
    pub(crate) subject: String,
}

#[tauri::command]
pub(crate) fn get_git_status(path: Option<String>) -> Result<GitStatus, String> {
    let cwd = path
        .as_deref()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .and_then(|path| {
            if path.is_file() {
                path.parent().map(Path::to_path_buf)
            } else {
                Some(path)
            }
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let inside = run_git(&cwd, &["rev-parse", "--is-inside-work-tree"])?;
    if inside.trim() != "true" {
        return Ok(GitStatus {
            inside_repo: false,
            branch: None,
            dirty: false,
            summary: Vec::new(),
        });
    }

    let branch = run_git(&cwd, &["branch", "--show-current"])
        .ok()
        .map(|branch| branch.trim().to_string())
        .filter(|branch| !branch.is_empty());
    let status = run_git(&cwd, &["status", "--short"]).unwrap_or_default();
    let summary = status.lines().map(ToString::to_string).collect::<Vec<_>>();
    Ok(GitStatus {
        inside_repo: true,
        branch,
        dirty: !summary.is_empty(),
        summary,
    })
}

#[tauri::command]
pub(crate) fn git_history(request: GitPathRequest) -> Result<Vec<GitHistoryEntry>, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let output = run_git(
        &cwd,
        &[
            "log",
            "--date=iso-strict",
            "--format=%H%x1f%an%x1f%ad%x1f%s",
            "--",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(request.path.as_str()),
        ],
    )?;
    Ok(output
        .lines()
        .filter_map(|line| {
            let parts = line.split('\u{1f}').collect::<Vec<_>>();
            if parts.len() < 4 {
                return None;
            }
            Some(GitHistoryEntry {
                revision: parts[0].to_string(),
                author: parts[1].to_string(),
                date: parts[2].to_string(),
                subject: parts[3].to_string(),
            })
        })
        .collect())
}

#[tauri::command]
pub(crate) fn git_diff(request: GitPathRequest) -> Result<String, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    run_git(
        &cwd,
        &[
            "diff",
            "--",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(request.path.as_str()),
        ],
    )
}

#[tauri::command]
pub(crate) fn commit_document_changes(request: GitCommitRequest) -> Result<GitStatus, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(request.path.as_str());
    run_git(&cwd, &["add", "--", file_name])?;
    run_git(&cwd, &["commit", "-m", &request.message, "--", file_name])?;
    get_git_status(Some(request.path))
}

#[tauri::command]
pub(crate) fn tag_release(request: GitTagRequest) -> Result<String, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    run_git(&cwd, &["tag", "-a", &request.tag, "-m", &request.message])?;
    Ok(request.tag)
}

#[tauri::command]
pub(crate) fn restore_git_revision(request: GitRestoreRequest) -> Result<FileResponse, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(request.path.as_str());
    let content = run_git(
        &cwd,
        &["show", &format!("{}:{file_name}", request.revision)],
    )?;
    fs::write(&path, content.as_bytes()).map_err(|err| err.to_string())?;
    read_file(path_to_string(&path))
}

pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
