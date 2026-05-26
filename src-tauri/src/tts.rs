use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::PathBuf,
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
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeTtsResponse {
    pub(crate) engine: String,
    pub(crate) characters: usize,
    pub(crate) message: String,
}

#[derive(Debug, PartialEq)]
pub(crate) struct NativeTtsCommand {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) stdin_text: Option<String>,
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
pub(crate) fn stop_text_aloud(state: State<'_, NativeTtsState>) -> Result<NativeTtsResponse, String> {
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
        message: format!("Stopped {count} native text-to-speech process{}", if count == 1 { "" } else { "es" }),
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
        Err("macOS Say is only available on macOS. Use browser speech or Supertonic on this host.".to_string())
    }
}

fn supertonic_command(
    request: &NativeTtsRequest,
    text: &str,
) -> Result<NativeTtsCommand, String> {
    let command_path = safe_command_path(request.command_path.as_deref())?;
    let mut args = vec!["say".to_string(), text.to_string(), "--model".to_string(), "supertonic-3".to_string()];
    if let Some(voice) = safe_cli_value(request.voice.as_deref(), 80) {
        args.push("--voice".to_string());
        args.push(voice);
    }
    if let Some(language) = safe_cli_value(request.language.as_deref(), 24) {
        args.push("--lang".to_string());
        args.push(language);
    }
    if let Some(speed) = request.speed.filter(|value| value.is_finite() && (0.7..=2.0).contains(value)) {
        args.push("--speed".to_string());
        args.push(format!("{speed:.2}"));
    }
    Ok(NativeTtsCommand {
        program: command_path,
        args,
        stdin_text: None,
    })
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
            return Err(format!("Configured text-to-speech command is not an executable file: {path}"));
        }
    }
    Ok(path.to_string())
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
