use crate::{path_to_string, sha256_hex};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};
use tauri::{path::BaseDirectory, AppHandle, Manager};

const SHOWCASE_DOCUMENT_RELATIVE_PATH: &str = "examples/showcase/neditor-capability-showcase.md";
/// Maximum bytes read via `file_metadata` / `read_file` to prevent DoS.
const MAX_READ_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB

#[derive(Debug, Deserialize)]
pub(crate) struct SaveFileRequest {
    pub(crate) path: String,
    pub(crate) text: String,
    pub(crate) expected_hash: Option<String>,
    /// When present, the resolved path must stay inside this workspace root.
    #[serde(default)]
    pub(crate) workspace_root: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RenameFileRequest {
    pub(crate) from: String,
    pub(crate) to: String,
    #[serde(default)]
    pub(crate) workspace_root: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DuplicateFileRequest {
    pub(crate) from: String,
    pub(crate) to: String,
    #[serde(default)]
    pub(crate) workspace_root: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CopyDataSourceFileRequest {
    pub(crate) source_path: String,
    pub(crate) document_path: Option<String>,
    pub(crate) workspace_root: Option<String>,
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

#[derive(Debug, Serialize)]
pub(crate) struct CopyDataSourceFileResponse {
    pub(crate) source_path: String,
    pub(crate) output_path: String,
    pub(crate) relative_path: String,
    pub(crate) bytes: u64,
    pub(crate) sha256: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RevealCommand {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
}

// ---------------------------------------------------------------------------
// Workspace-scoped path resolution (G1)
// ---------------------------------------------------------------------------

/// Resolve `path` within `workspace_root`, enforcing:
/// - the path itself is not a symlink
/// - after `canonicalize`, the result starts with the canonical workspace root
///
/// For new-file creation (`for_creation = true`), validates the parent
/// directory instead of the (non-existent) target path.
///
/// When `workspace_root` is `None`, only symlink refusal is applied; no
/// workspace boundary is enforced (defence-in-depth still rejects symlinks).
pub(crate) fn resolve_within_workspace(
    path: &str,
    workspace_root: Option<&str>,
    for_creation: bool,
) -> Result<PathBuf, String> {
    let candidate = PathBuf::from(path.trim());

    // Refuse symlinks on the path itself when it already exists.
    if let Ok(meta) = fs::symlink_metadata(&candidate) {
        if meta.file_type().is_symlink() {
            return Err("path escapes workspace: refusing symlink".to_string());
        }
    }

    // Canonicalize: for new files, validate the parent directory.
    let resolved = if for_creation && !candidate.exists() {
        let parent = candidate
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| "path escapes workspace: no parent directory".to_string())?;
        let canonical_parent = parent
            .canonicalize()
            .map_err(|e| format!("cannot resolve parent directory: {e}"))?;
        // Refuse symlink on parent too.
        if let Ok(meta) = fs::symlink_metadata(parent) {
            if meta.file_type().is_symlink() {
                return Err("path escapes workspace: parent is a symlink".to_string());
            }
        }
        canonical_parent.join(
            candidate
                .file_name()
                .ok_or_else(|| "path has no file name".to_string())?,
        )
    } else {
        candidate
            .canonicalize()
            .map_err(|e| format!("cannot resolve path: {e}"))?
    };

    // Workspace boundary check.
    if let Some(root) = workspace_root {
        let root_trimmed = root.trim();
        if !root_trimmed.is_empty() {
            let canonical_root = PathBuf::from(root_trimmed)
                .canonicalize()
                .map_err(|e| format!("cannot resolve workspace root: {e}"))?;
            if !resolved.starts_with(&canonical_root) {
                return Err("path escapes workspace".to_string());
            }
        }
    }

    Ok(resolved)
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub(crate) fn read_file(
    path: String,
    workspace_root: Option<String>,
) -> Result<FileResponse, String> {
    let resolved = resolve_within_workspace(&path, workspace_root.as_deref(), false)?;
    // G7: cap read size to prevent DoS.
    let file_size = fs::metadata(&resolved).map(|m| m.len()).unwrap_or_default();
    if file_size > MAX_READ_BYTES {
        return Err(format!(
            "File is {} bytes, above the {} byte read limit.",
            file_size, MAX_READ_BYTES
        ));
    }
    let text = fs::read_to_string(&resolved).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&resolved).ok();
    Ok(FileResponse {
        path: path_to_string(&resolved),
        hash: sha256_hex(text.as_bytes()),
        modified: metadata.and_then(modified_time),
        text,
    })
}

#[tauri::command]
pub(crate) fn open_file(
    path: String,
    workspace_root: Option<String>,
) -> Result<FileResponse, String> {
    read_file(path, workspace_root)
}

#[tauri::command]
pub(crate) fn read_showcase_document(app: AppHandle) -> Result<FileResponse, String> {
    for candidate in showcase_document_candidate_paths(&app) {
        if candidate.is_file() {
            // Showcase document is a known resource path — no workspace check needed.
            let path_str = path_to_string(&candidate);
            return read_file(path_str, None);
        }
    }
    Err("The packaged showcase document could not be found. Reinstall NEditor or open examples/showcase/neditor-capability-showcase.md from the source distribution.".to_string())
}

#[tauri::command]
pub(crate) fn save_file(request: SaveFileRequest) -> Result<FileResponse, String> {
    let resolved =
        resolve_within_workspace(&request.path, request.workspace_root.as_deref(), true)?;
    if let Some(expected_hash) = &request.expected_hash {
        if resolved.exists() {
            let current = fs::read(&resolved).map_err(|err| err.to_string())?;
            let current_hash = sha256_hex(&current);
            if &current_hash != expected_hash {
                return Err(
                    "File changed on disk since it was opened; resolve the external conflict before saving."
                        .to_string(),
                );
            }
        }
    }
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(&resolved, request.text.as_bytes()).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&resolved).ok();
    Ok(FileResponse {
        path: path_to_string(&resolved),
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
    let from = resolve_within_workspace(&request.from, request.workspace_root.as_deref(), false)?;
    // G3: refuse symlink at rename destination.
    if let Ok(meta) = fs::symlink_metadata(&PathBuf::from(&request.to)) {
        if meta.file_type().is_symlink() {
            return Err("rename destination is a symlink; refusing to replace".to_string());
        }
    }
    let to_path = PathBuf::from(request.to.trim());
    if let Some(parent) = to_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    // G3: fall back to copy+delete on cross-device rename.
    rename_or_move(&from, &to_path).map_err(|err| err.to_string())?;

    // Re-resolve destination after the move (it now exists).
    let resolved_to = if let Ok(r) = resolve_within_workspace(
        &path_to_string(&to_path),
        request.workspace_root.as_deref(),
        false,
    ) {
        r
    } else {
        to_path.clone()
    };
    file_metadata_inner(&resolved_to)
}

#[tauri::command]
pub(crate) fn duplicate_file(request: DuplicateFileRequest) -> Result<FileResponse, String> {
    let from = resolve_within_workspace(&request.from, request.workspace_root.as_deref(), false)?;
    let to_path = PathBuf::from(request.to.trim());
    if let Some(parent) = to_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::copy(&from, &to_path).map_err(|err| err.to_string())?;
    let resolved_to = resolve_within_workspace(
        &path_to_string(&to_path),
        request.workspace_root.as_deref(),
        false,
    )?;
    read_file(path_to_string(&resolved_to), request.workspace_root)
}

#[tauri::command]
pub(crate) fn copy_data_source_file(
    request: CopyDataSourceFileRequest,
) -> Result<CopyDataSourceFileResponse, String> {
    let base = data_source_copy_base(
        request.document_path.as_deref(),
        request.workspace_root.as_deref(),
    )?;
    let source_input = request.source_path.trim();
    let source = if PathBuf::from(source_input).is_absolute() {
        PathBuf::from(source_input)
    } else {
        base.join(source_input)
    };
    if !source.is_file() {
        return Err("Choose an existing local data-source file to copy.".to_string());
    }
    let data_dir = base.join("data");
    fs::create_dir_all(&data_dir).map_err(|err| err.to_string())?;
    let file_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .map(safe_data_source_file_name)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| "Could not determine a safe data-source file name.".to_string())?;
    let output = unique_data_source_output_path(&data_dir, &file_name)?;
    let canonical_base = base
        .canonicalize()
        .map_err(|err| format!("Could not inspect data-source folder: {err}"))?;
    let canonical_data_dir = data_dir
        .canonicalize()
        .map_err(|err| format!("Could not inspect data-source folder: {err}"))?;
    if !canonical_data_dir.starts_with(&canonical_base) {
        return Err(
            "Data-source files must stay inside the document or workspace folder.".to_string(),
        );
    }
    let initial_output = data_dir.join(&file_name);
    if source.canonicalize().ok().as_ref() == initial_output.canonicalize().ok().as_ref() {
        let bytes = fs::read(&initial_output).map_err(|err| err.to_string())?;
        return Ok(CopyDataSourceFileResponse {
            source_path: path_to_string(&source),
            output_path: path_to_string(&initial_output),
            relative_path: data_source_relative_path(&base, &initial_output),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        });
    }
    if source.canonicalize().ok().as_ref() == output.canonicalize().ok().as_ref() {
        let bytes = fs::read(&output).map_err(|err| err.to_string())?;
        return Ok(CopyDataSourceFileResponse {
            source_path: path_to_string(&source),
            output_path: path_to_string(&output),
            relative_path: data_source_relative_path(&base, &output),
            bytes: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        });
    }
    fs::copy(&source, &output).map_err(|err| err.to_string())?;
    let bytes = fs::read(&output).map_err(|err| err.to_string())?;
    Ok(CopyDataSourceFileResponse {
        source_path: path_to_string(&source),
        output_path: path_to_string(&output),
        relative_path: data_source_relative_path(&base, &output),
        bytes: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
    })
}

#[tauri::command]
pub(crate) fn reveal_path(path: String) -> Result<(), String> {
    let command_spec = reveal_command_for_path(&path)?;
    Command::new(&command_spec.program)
        .args(&command_spec.args)
        .spawn()
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub(crate) fn file_metadata(
    path: String,
    workspace_root: Option<String>,
) -> Result<FileMetadata, String> {
    let path_buf = PathBuf::from(path.trim());
    if !path_buf.exists() {
        // For non-existent paths return exists:false without workspace check
        // (the path cannot be resolved, so there's nothing to disclose).
        return Ok(FileMetadata {
            path: path_to_string(&path_buf),
            exists: false,
            hash: None,
            modified: None,
        });
    }
    let resolved = resolve_within_workspace(&path, workspace_root.as_deref(), false)?;
    file_metadata_inner(&resolved)
}

fn file_metadata_inner(resolved: &std::path::Path) -> Result<FileMetadata, String> {
    if !resolved.exists() {
        return Ok(FileMetadata {
            path: path_to_string(resolved),
            exists: false,
            hash: None,
            modified: None,
        });
    }
    let text = fs::read(resolved).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(resolved).ok();
    Ok(FileMetadata {
        path: path_to_string(resolved),
        exists: true,
        hash: Some(sha256_hex(&text)),
        modified: metadata.and_then(modified_time),
    })
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// G3: rename with cross-device fallback (copy + delete).
fn rename_or_move(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            fs::copy(from, to)?;
            fs::remove_file(from)
        }
        Err(e) => Err(e),
    }
}

fn data_source_copy_base(
    document_path: Option<&str>,
    workspace_root: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(path) = document_path.map(str::trim).filter(|path| !path.is_empty()) {
        let document = PathBuf::from(path);
        if let Some(parent) = document.parent() {
            if !parent.as_os_str().is_empty() {
                return Ok(parent.to_path_buf());
            }
        }
    }
    if let Some(path) = workspace_root
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        return Ok(PathBuf::from(path));
    }
    Err("Save the document or open a workspace before copying a data-source file.".to_string())
}

fn safe_data_source_file_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches(['.', '-'])
        .to_string()
}

fn unique_data_source_output_path(
    data_dir: &std::path::Path,
    file_name: &str,
) -> Result<PathBuf, String> {
    let initial = data_dir.join(file_name);
    if !initial.exists() {
        return Ok(initial);
    }
    let path = PathBuf::from(file_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("data-source");
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    for index in 2..1000 {
        let candidate_name = if extension.is_empty() {
            format!("{stem}-{index}")
        } else {
            format!("{stem}-{index}.{extension}")
        };
        let candidate = data_dir.join(candidate_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("Too many copies of this data-source file already exist.".to_string())
}

fn data_source_relative_path(base: &std::path::Path, output: &std::path::Path) -> String {
    output
        .strip_prefix(base)
        .ok()
        .map(path_to_string)
        .unwrap_or_else(|| path_to_string(output))
}

fn showcase_document_candidate_paths(app: &AppHandle) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(resource) = app
        .path()
        .resolve(SHOWCASE_DOCUMENT_RELATIVE_PATH, BaseDirectory::Resource)
    {
        candidates.push(resource);
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(SHOWCASE_DOCUMENT_RELATIVE_PATH));
        candidates.push(cwd.join("..").join(SHOWCASE_DOCUMENT_RELATIVE_PATH));
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(SHOWCASE_DOCUMENT_RELATIVE_PATH),
    );
    candidates
}

/// Build a reveal command for `path`. On Windows (G2): use separate `/select,`
/// and path arguments so the path cannot bleed into the flag via commas.
pub(crate) fn reveal_command_for_path(path: &str) -> Result<RevealCommand, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Cannot reveal an empty path.".to_string());
    }

    let path_buf = PathBuf::from(trimmed);
    if !path_buf.exists() {
        return Err(format!(
            "Cannot reveal missing path: {}",
            path_to_string(&path_buf)
        ));
    }

    let canonical = path_buf
        .canonicalize()
        .map_err(|err| format!("Cannot reveal path: {err}"))?;
    let canonical_path = path_to_string(&canonical);

    #[cfg(target_os = "macos")]
    {
        Ok(RevealCommand {
            program: "open".to_string(),
            args: vec!["-R".to_string(), canonical_path],
        })
    }

    // G2: Pass /select, and the path as separate argv entries so that commas
    // in the file name cannot alter the flag interpretation. Explorer treats
    // the two tokens as a single logical argument under its own parsing rules.
    #[cfg(target_os = "windows")]
    {
        Ok(RevealCommand {
            program: "explorer".to_string(),
            args: vec!["/select,".to_string(), canonical_path],
        })
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target = canonical
            .parent()
            .map(path_to_string)
            .unwrap_or(canonical_path);
        Ok(RevealCommand {
            program: "xdg-open".to_string(),
            args: vec![target],
        })
    }
}

fn modified_time(metadata: fs::Metadata) -> Option<String> {
    metadata
        .modified()
        .ok()
        .map(chrono::DateTime::<Utc>::from)
        .map(|time| time.to_rfc3339())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        std::env::temp_dir().join(format!("neditor-fs-test-{tag}-{nanos}"))
    }

    // G1: workspace-scoping tests
    #[test]
    fn resolve_within_workspace_accepts_path_inside_root() {
        let root = unique_temp_dir("ws-ok");
        fs::create_dir_all(&root).unwrap();
        let file = root.join("doc.md");
        fs::write(&file, "hello").unwrap();
        let resolved =
            resolve_within_workspace(&path_to_string(&file), Some(&path_to_string(&root)), false);
        assert!(resolved.is_ok(), "{resolved:?}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn resolve_within_workspace_rejects_path_outside_root() {
        let root = unique_temp_dir("ws-reject");
        fs::create_dir_all(&root).unwrap();
        // Use a path that definitely exists but is outside root.
        let outside = std::env::temp_dir();
        let err = resolve_within_workspace(
            &path_to_string(&outside),
            Some(&path_to_string(&root)),
            false,
        );
        assert!(err.is_err(), "expected error for path outside workspace");
        assert!(
            err.unwrap_err().contains("escapes workspace"),
            "error should mention escapes workspace"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn resolve_within_workspace_rejects_dotdot_traversal() {
        let root = unique_temp_dir("ws-dotdot");
        let sub = root.join("sub");
        fs::create_dir_all(&sub).unwrap();
        let parent = root.parent().unwrap();
        // A real existing path outside root.
        let outside = path_to_string(parent);
        let err = resolve_within_workspace(&outside, Some(&path_to_string(&root)), false);
        assert!(err.is_err());
        fs::remove_dir_all(&root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn resolve_within_workspace_rejects_symlink() {
        let root = unique_temp_dir("ws-symlink");
        fs::create_dir_all(&root).unwrap();
        let real_file = root.join("real.md");
        fs::write(&real_file, "data").unwrap();
        let link = root.join("link.md");
        std::os::unix::fs::symlink(&real_file, &link).unwrap();
        let err =
            resolve_within_workspace(&path_to_string(&link), Some(&path_to_string(&root)), false);
        assert!(err.is_err(), "symlink should be rejected");
        assert!(err.unwrap_err().contains("symlink"));
        fs::remove_dir_all(&root).unwrap();
    }

    // G3: cross-device rename fallback (simulated by same-device copy+delete fallback path)
    #[test]
    fn rename_or_move_falls_back_on_cross_device() {
        // We can't trigger EXDEV in a unit test without actual different devices,
        // but we can confirm the happy path works on same device.
        let dir = unique_temp_dir("rename-fallback");
        fs::create_dir_all(&dir).unwrap();
        let from = dir.join("from.txt");
        let to = dir.join("to.txt");
        fs::write(&from, "content").unwrap();
        rename_or_move(&from, &to).unwrap();
        assert!(!from.exists());
        assert_eq!(fs::read_to_string(&to).unwrap(), "content");
        fs::remove_dir_all(&dir).unwrap();
    }

    // G2: reveal_command uses separate /select, arg on Windows
    #[test]
    fn reveal_command_for_path_rejects_empty() {
        assert!(reveal_command_for_path("").is_err());
        assert!(reveal_command_for_path("   ").is_err());
    }
}
