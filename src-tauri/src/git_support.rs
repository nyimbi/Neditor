use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) fn git_cwd_for_path(path: &Path) -> PathBuf {
    if path.exists() && path.is_dir() {
        return path.to_path_buf();
    }
    path.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub(crate) fn git_pathspec<'a>(path: &'a Path, fallback: &'a str) -> &'a str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(fallback)
}

pub(crate) fn validate_git_tag_name(tag: &str) -> Result<(), String> {
    validate_git_refish("Git tag", tag)
}

pub(crate) fn validate_git_revision(revision: &str) -> Result<(), String> {
    validate_git_refish("Git revision", revision)
}

pub(crate) fn validate_git_restore_target(cwd: &Path, path: &Path) -> Result<(), String> {
    if let Ok(metadata) = fs::symlink_metadata(path) {
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            return Err("Refusing to restore through a symlink.".to_string());
        }
        if metadata.is_dir() {
            return Err("Git restore target must be a file.".to_string());
        }
    }

    let repo_root = git_repo_root(cwd)?;
    let parent = path
        .parent()
        .ok_or_else(|| "Git restore target must have a parent directory.".to_string())?;
    let parent = parent.canonicalize().map_err(|err| err.to_string())?;
    if !parent.starts_with(&repo_root) {
        return Err("Git restore target must stay inside the repository.".to_string());
    }
    Ok(())
}

fn validate_git_refish(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{label} cannot be empty."));
    }
    if value.starts_with('-') {
        return Err(format!("{label} cannot start with '-'."));
    }
    if value == "@" {
        return Err(format!("{label} cannot be '@'."));
    }
    if value.starts_with('/') || value.ends_with('/') || value.contains("//") {
        return Err(format!("{label} cannot contain empty path components."));
    }
    if value.ends_with('.') {
        return Err(format!("{label} cannot end with '.'."));
    }
    if value.contains("..") || value.contains("@{") {
        return Err(format!("{label} contains unsupported ref syntax."));
    }
    if value.chars().any(|character| {
        character.is_control()
            || character.is_whitespace()
            || matches!(character, ':' | '?' | '*' | '[' | '\\' | '^' | '~')
    }) {
        return Err(format!("{label} contains unsupported characters."));
    }
    if value
        .split('/')
        .any(|component| component.starts_with('.') || component.ends_with(".lock"))
    {
        return Err(format!("{label} contains an unsupported ref component."));
    }
    Ok(())
}

fn git_repo_root(cwd: &Path) -> Result<PathBuf, String> {
    let root = run_git(cwd, &["rev-parse", "--show-toplevel"])?;
    PathBuf::from(root.trim())
        .canonicalize()
        .map_err(|err| err.to_string())
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
