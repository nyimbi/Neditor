use crate::{path_to_string, sha256_hex};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};
use tauri::{path::BaseDirectory, AppHandle, Manager};

const SHOWCASE_DOCUMENT_RELATIVE_PATH: &str = "examples/showcase/neditor-capability-showcase.md";

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
pub(crate) fn read_showcase_document(app: AppHandle) -> Result<FileResponse, String> {
    for candidate in showcase_document_candidate_paths(&app) {
        if candidate.is_file() {
            return read_file(path_to_string(&candidate));
        }
    }
    Err("The packaged showcase document could not be found. Reinstall NEditor or open examples/showcase/neditor-capability-showcase.md from the source distribution.".to_string())
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

    #[cfg(target_os = "windows")]
    {
        Ok(RevealCommand {
            program: "explorer".to_string(),
            args: vec![format!("/select,{canonical_path}")],
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

fn modified_time(metadata: fs::Metadata) -> Option<String> {
    metadata
        .modified()
        .ok()
        .map(chrono::DateTime::<Utc>::from)
        .map(|time| time.to_rfc3339())
}
