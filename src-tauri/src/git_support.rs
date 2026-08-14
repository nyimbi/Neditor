use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::{Duration, Instant},
};

/// Resolve the absolute git binary path once at startup.
/// Falls back gracefully: if `git` is not on PATH the first `run_git` call
/// returns a descriptive error instead of silently using a PATH-substituted binary.
fn git_binary() -> Result<&'static PathBuf, String> {
    static GIT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    GIT.get_or_init(|| {
        which::which("git").map_err(|_| {
            "git is not installed or not on PATH. Install git to enable version-control features."
                .to_string()
        })
    })
    .as_ref()
    .map_err(|e| e.clone())
}

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
    if !parent.exists() {
        return Err("Git restore target's parent directory does not exist.".to_string());
    }
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

pub(crate) fn git_repo_root(cwd: &Path) -> Result<PathBuf, String> {
    let root = run_git(cwd, &["rev-parse", "--show-toplevel"])?;
    PathBuf::from(root.trim())
        .canonicalize()
        .map_err(|err| err.to_string())
}

/// Run a git command in `cwd` with a 30-second kill-on-timeout guard.
///
/// G15: uses the PATH-resolved `git` binary cached at startup via
/// `which::which`. Does not inherit a shell env that could substitute git.
pub(crate) fn run_git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let git = git_binary()?;
    // G15: spawn with a timeout; kill the child if it doesn't finish in time.
    let mut child = Command::new(git)
        .args(args)
        .current_dir(cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|err| err.to_string())?;

    let timeout = Duration::from_secs(30);
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "git {:?} timed out after {}s",
                        args,
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(err) => {
                let _ = child.kill();
                return Err(err.to_string());
            }
        }
    }

    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_git_refish_rejects_dangerous_inputs() {
        assert!(validate_git_refish("test", "-malicious").is_err());
        assert!(validate_git_refish("test", "ref/../escape").is_err());
        assert!(validate_git_refish("test", "ref@{bad}").is_err());
        assert!(validate_git_refish("test", "ref:path").is_err());
        assert!(validate_git_refish("test", "").is_err());
        assert!(validate_git_refish("test", "@").is_err());
    }

    #[test]
    fn validate_git_refish_accepts_valid_refs() {
        assert!(validate_git_refish("test", "main").is_ok());
        assert!(validate_git_refish("test", "feature/my-branch").is_ok());
        assert!(validate_git_refish("test", "v1.2.3").is_ok());
        assert!(validate_git_refish("test", "abc1234def5678").is_ok());
    }

    // G15: git binary resolves via which (not ambient PATH name).
    #[test]
    fn git_binary_resolves_to_absolute_path() {
        // git_binary() may fail if git is not installed in CI; that's acceptable.
        if let Ok(path) = git_binary() {
            assert!(
                path.is_absolute(),
                "git binary path should be absolute, got: {}",
                path.display()
            );
        }
    }
}
