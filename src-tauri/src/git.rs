use crate::{
    git_support::{
        git_cwd_for_path, git_pathspec, run_git, validate_git_restore_target,
        validate_git_revision, validate_git_tag_name,
    },
    git_types::{
        GitCommitRequest, GitHistoryEntry, GitPathRequest, GitRestoreRequest, GitStatus,
        GitTagRequest,
    },
    path_to_string, read_file, FileResponse,
};
use std::{fs, path::PathBuf};

#[tauri::command]
pub(crate) fn get_git_status(path: Option<String>) -> Result<GitStatus, String> {
    let cwd = path
        .as_deref()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .map(|path| git_cwd_for_path(&path))
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
    let cwd = git_cwd_for_path(&path);
    let pathspec = git_pathspec(&path, request.path.as_str());
    let output = run_git(
        &cwd,
        &[
            "log",
            "--date=iso-strict",
            "--format=%H%x1f%an%x1f%ad%x1f%s",
            "--",
            pathspec,
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
    let cwd = git_cwd_for_path(&path);
    run_git(
        &cwd,
        &["diff", "--", git_pathspec(&path, request.path.as_str())],
    )
}

#[tauri::command]
pub(crate) fn commit_document_changes(request: GitCommitRequest) -> Result<GitStatus, String> {
    let path = PathBuf::from(&request.path);
    let cwd = git_cwd_for_path(&path);
    let file_name = git_pathspec(&path, request.path.as_str());
    run_git(&cwd, &["add", "--", file_name])?;
    run_git(&cwd, &["commit", "-m", &request.message, "--", file_name])?;
    get_git_status(Some(request.path))
}

#[tauri::command]
pub(crate) fn tag_release(request: GitTagRequest) -> Result<String, String> {
    let path = PathBuf::from(&request.path);
    let cwd = git_cwd_for_path(&path);
    let tag = request.tag.trim();
    validate_git_tag_name(tag)?;
    run_git(&cwd, &["tag", "-a", tag, "-m", &request.message])?;
    Ok(tag.to_string())
}

#[tauri::command]
pub(crate) fn restore_git_revision(request: GitRestoreRequest) -> Result<FileResponse, String> {
    let path = PathBuf::from(&request.path);
    let cwd = git_cwd_for_path(&path);
    let file_name = git_pathspec(&path, request.path.as_str());
    let revision = request.revision.trim();
    validate_git_revision(revision)?;
    validate_git_restore_target(&cwd, &path)?;
    let content = run_git(&cwd, &["show", &format!("{revision}:{file_name}")])?;
    fs::write(&path, content.as_bytes()).map_err(|err| err.to_string())?;
    read_file(path_to_string(&path))
}
