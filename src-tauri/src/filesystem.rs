use crate::{path_to_string, sha256_hex};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};

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
    let command_spec = reveal_command_for_path(&path)?;
    Command::new(&command_spec.program)
        .args(&command_spec.args)
        .spawn()
        .map_err(|err| err.to_string())?;
    Ok(())
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
