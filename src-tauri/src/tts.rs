use serde::{Deserialize, Serialize};
use std::{
    env,
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
};
use tauri::State;

#[derive(Default)]
pub(crate) struct NativeTtsState {
    children: Mutex<Vec<Child>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NativeTtsRequest {
    pub(crate) engine: String,
    pub(crate) text: String,
    pub(crate) voice: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) rate: Option<u16>,
    pub(crate) command_path: Option<String>,
    pub(crate) speed: Option<f32>,
    pub(crate) model_download_acknowledged: Option<bool>,
    pub(crate) model_storage_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NativeTtsModelDownloadRequest {
    pub(crate) engine: String,
    pub(crate) command_path: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) approximate_size: Option<String>,
    pub(crate) storage_path: Option<String>,
    pub(crate) acknowledged: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NativeTtsInspectionRequest {
    pub(crate) supertonic_command: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeTtsResponse {
    pub(crate) engine: String,
    pub(crate) characters: usize,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeTtsEngineStatus {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) available: bool,
    pub(crate) detail: String,
    pub(crate) executable_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeTtsInspectionResponse {
    pub(crate) engines: Vec<NativeTtsEngineStatus>,
    pub(crate) available_native_engines: usize,
}

#[derive(Debug, PartialEq)]
pub(crate) struct NativeTtsCommand {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) stdin_text: Option<String>,
}

#[tauri::command]
pub(crate) fn inspect_native_tts(
    request: NativeTtsInspectionRequest,
) -> Result<NativeTtsInspectionResponse, String> {
    let mut engines = vec![browser_speech_status(), macos_say_status()];
    engines.push(supertonic_status(request.supertonic_command.as_deref()));
    let available_native_engines = engines
        .iter()
        .filter(|engine| engine.id != "browser-speech" && engine.available)
        .count();
    Ok(NativeTtsInspectionResponse {
        engines,
        available_native_engines,
    })
}

#[tauri::command]
pub(crate) fn read_text_aloud(
    request: NativeTtsRequest,
    state: State<'_, NativeTtsState>,
) -> Result<NativeTtsResponse, String> {
    let command = native_tts_command_for_request(&request)?;
    let characters = request.text.trim().chars().count();
    let mut child = Command::new(&command.program)
        .args(&command.args)
        .stdin(if command.stdin_text.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("Could not start text-to-speech engine: {err}"))?;

    if let Some(text) = command.stdin_text {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|err| format!("Could not send text to speech engine: {err}"))?;
        }
    }
    state
        .children
        .lock()
        .map_err(|_| "Could not track text-to-speech process.".to_string())?
        .push(child);

    Ok(NativeTtsResponse {
        engine: request.engine,
        characters,
        message: format!("Started reading {characters} characters"),
    })
}

#[tauri::command]
pub(crate) fn download_tts_model(
    request: NativeTtsModelDownloadRequest,
    state: State<'_, NativeTtsState>,
) -> Result<NativeTtsResponse, String> {
    let command = native_tts_model_download_command_for_request(&request)?;
    let mut child = Command::new(&command.program)
        .args(&command.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("Could not start TTS model download: {err}"))?;
    let _ = child.try_wait();
    state
        .children
        .lock()
        .map_err(|_| "Could not track text-to-speech model download.".to_string())?
        .push(child);
    let model =
        safe_cli_value(request.model.as_deref(), 80).unwrap_or_else(|| "supertonic-3".to_string());
    let size = safe_cli_value(request.approximate_size.as_deref(), 40)
        .unwrap_or_else(|| "~305 MB".to_string());
    let location = safe_storage_path(request.storage_path.as_deref())
        .unwrap_or_else(|| "the Supertonic model cache".to_string());
    Ok(NativeTtsResponse {
        engine: request.engine,
        characters: 0,
        message: format!("Started downloading {model} ({size}) to {location}"),
    })
}

#[tauri::command]
pub(crate) fn stop_text_aloud(
    state: State<'_, NativeTtsState>,
) -> Result<NativeTtsResponse, String> {
    let mut children = state
        .children
        .lock()
        .map_err(|_| "Could not access text-to-speech process state.".to_string())?;
    let count = children.len();
    for child in children.iter_mut() {
        let _ = child.kill();
    }
    children.clear();
    Ok(NativeTtsResponse {
        engine: "native".to_string(),
        characters: 0,
        message: format!(
            "Stopped {count} native text-to-speech process{}",
            if count == 1 { "" } else { "es" }
        ),
    })
}

pub(crate) fn native_tts_model_download_command_for_request(
    request: &NativeTtsModelDownloadRequest,
) -> Result<NativeTtsCommand, String> {
    if request.engine.trim() != "supertonic-cli" {
        return Err(
            "Only Supertonic currently has a downloadable TTS model in NEditor.".to_string(),
        );
    }
    if request.acknowledged != Some(true) {
        return Err("Review the Supertonic model name, download size, and storage location before starting the download.".to_string());
    }
    let command_path = safe_command_path(request.command_path.as_deref())?;
    Ok(NativeTtsCommand {
        program: command_path,
        args: vec!["download".to_string()],
        stdin_text: None,
    })
}

pub(crate) fn native_tts_command_for_request(
    request: &NativeTtsRequest,
) -> Result<NativeTtsCommand, String> {
    let text = request.text.trim();
    if text.is_empty() {
        return Err("No text is available to read aloud.".to_string());
    }

    match request.engine.trim() {
        "macos-say" => macos_say_command(request, text),
        "supertonic-cli" => supertonic_command(request, text),
        _ => Err("Unsupported native text-to-speech engine.".to_string()),
    }
}

fn macos_say_command(request: &NativeTtsRequest, text: &str) -> Result<NativeTtsCommand, String> {
    #[cfg(target_os = "macos")]
    {
        let mut args = Vec::new();
        if let Some(voice) = safe_cli_value(request.voice.as_deref(), 80) {
            args.push("-v".to_string());
            args.push(voice);
        }
        if let Some(rate) = request.rate.filter(|value| (80..=420).contains(value)) {
            args.push("-r".to_string());
            args.push(rate.to_string());
        }
        Ok(NativeTtsCommand {
            program: "say".to_string(),
            args,
            stdin_text: Some(text.to_string()),
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = request;
        let _ = text;
        Err(
            "macOS Say is only available on macOS. Use browser speech or Supertonic on this host."
                .to_string(),
        )
    }
}

fn supertonic_command(request: &NativeTtsRequest, text: &str) -> Result<NativeTtsCommand, String> {
    if request.model_download_acknowledged != Some(true) {
        let location = safe_storage_path(request.model_storage_path.as_deref())
            .unwrap_or_else(|| "the Supertonic model cache".to_string());
        return Err(format!(
            "Supertonic may download the supertonic-3 model on first use. Review and acknowledge the ~305 MB model download and storage location ({location}) before reading aloud."
        ));
    }
    let command_path = safe_command_path(request.command_path.as_deref())?;
    let mut args = vec![
        "say".to_string(),
        text.to_string(),
        "--model".to_string(),
        "supertonic-3".to_string(),
    ];
    if let Some(voice) = safe_cli_value(request.voice.as_deref(), 80) {
        args.push("--voice".to_string());
        args.push(voice);
    }
    if let Some(language) = safe_cli_value(request.language.as_deref(), 24) {
        args.push("--lang".to_string());
        args.push(language);
    }
    if let Some(speed) = request
        .speed
        .filter(|value| value.is_finite() && (0.7..=2.0).contains(value))
    {
        args.push("--speed".to_string());
        args.push(format!("{speed:.2}"));
    }
    Ok(NativeTtsCommand {
        program: command_path,
        args,
        stdin_text: None,
    })
}

fn safe_storage_path(value: Option<&str>) -> Option<String> {
    let text = value?.trim();
    if text.is_empty() || text.len() > 400 {
        return None;
    }
    if text
        .chars()
        .any(|character| character == '\0' || character == '\n' || character == '\r')
    {
        return None;
    }
    Some(text.to_string())
}

fn safe_command_path(value: Option<&str>) -> Result<String, String> {
    let path = value.unwrap_or("supertonic").trim();
    if path.is_empty() {
        return Err("Configure the Supertonic command before reading aloud.".to_string());
    }
    if path.contains('\0') || path.contains('\n') || path.contains('\r') {
        return Err("Text-to-speech command paths cannot contain control characters.".to_string());
    }
    if path.contains(std::path::MAIN_SEPARATOR) {
        let candidate = PathBuf::from(path);
        if !candidate.exists() || !candidate.is_file() {
            return Err(format!(
                "Configured text-to-speech command is not an executable file: {path}"
            ));
        }
    }
    Ok(path.to_string())
}

fn browser_speech_status() -> NativeTtsEngineStatus {
    NativeTtsEngineStatus {
        id: "browser-speech".to_string(),
        label: "Browser or system speech".to_string(),
        available: true,
        detail: "Checked in the web runtime before playback; no native command is required."
            .to_string(),
        executable_path: None,
    }
}

fn macos_say_status() -> NativeTtsEngineStatus {
    #[cfg(target_os = "macos")]
    {
        let executable_path = command_path("say");
        NativeTtsEngineStatus {
            id: "macos-say".to_string(),
            label: "macOS Say".to_string(),
            available: executable_path.is_some(),
            detail: executable_path
                .as_ref()
                .map(|path| format!("Found native Say executable at {path}."))
                .unwrap_or_else(|| "macOS Say was not found on PATH.".to_string()),
            executable_path,
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        NativeTtsEngineStatus {
            id: "macos-say".to_string(),
            label: "macOS Say".to_string(),
            available: false,
            detail: "macOS Say is only available on macOS.".to_string(),
            executable_path: None,
        }
    }
}

fn supertonic_status(command: Option<&str>) -> NativeTtsEngineStatus {
    let configured = command.unwrap_or("supertonic").trim();
    if configured.is_empty()
        || configured.contains('\0')
        || configured.contains('\n')
        || configured.contains('\r')
    {
        return NativeTtsEngineStatus {
            id: "supertonic-cli".to_string(),
            label: "Supertonic CLI".to_string(),
            available: false,
            detail: "Configure a valid Supertonic command path before using Supertonic."
                .to_string(),
            executable_path: None,
        };
    }
    let executable_path = command_path(configured);
    NativeTtsEngineStatus {
        id: "supertonic-cli".to_string(),
        label: "Supertonic CLI".to_string(),
        available: executable_path.is_some(),
        detail: executable_path
            .as_ref()
            .map(|path| format!("Found Supertonic command at {path}."))
            .unwrap_or_else(|| {
                "Supertonic command was not found. Install the CLI or configure its full path."
                    .to_string()
            }),
        executable_path,
    }
}

fn command_path(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }
    let has_path_separator = trimmed.contains('/') || trimmed.contains('\\');
    if has_path_separator {
        let path = PathBuf::from(trimmed);
        return executable_file(&path).then(|| path.to_string_lossy().to_string());
    }
    for dir in env::split_paths(&env::var_os("PATH").unwrap_or_else(OsString::new)) {
        for candidate in executable_candidates(&dir, trimmed) {
            if executable_file(&candidate) {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let fallback = PathBuf::from("/usr/bin").join(trimmed);
        if executable_file(&fallback) {
            return Some(fallback.to_string_lossy().to_string());
        }
    }
    None
}

fn executable_candidates(dir: &Path, command: &str) -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let pathext = env::var_os("PATHEXT")
            .map(|value| {
                value
                    .to_string_lossy()
                    .split(';')
                    .filter(|ext| !ext.trim().is_empty())
                    .map(|ext| ext.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![".exe".to_string(), ".cmd".to_string(), ".bat".to_string()]);
        let mut candidates = vec![dir.join(command)];
        candidates.extend(
            pathext
                .into_iter()
                .map(|ext| dir.join(format!("{command}{ext}"))),
        );
        candidates
    }

    #[cfg(not(windows))]
    {
        vec![dir.join(command)]
    }
}

fn executable_file(path: &Path) -> bool {
    path.is_file()
}

fn safe_cli_value(value: Option<&str>, max_len: usize) -> Option<String> {
    let text = value?.trim();
    if text.is_empty() || text.len() > max_len {
        return None;
    }
    if text
        .chars()
        .any(|character| character == '\0' || character == '\n' || character == '\r')
    {
        return None;
    }
    Some(text.to_string())
}
