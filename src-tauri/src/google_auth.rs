use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const GOOGLE_AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const CALLBACK_PATH: &str = "/google-oauth-callback";
const SESSION_TTL_SECONDS: u64 = 300;
const OAUTH_TOKEN_BYTES: usize = 32;

const MAX_CONCURRENT_OAUTH_LISTENERS: u32 = 1;

#[derive(Clone, Default)]
pub(crate) struct GoogleAuthState {
    sessions: Arc<Mutex<HashMap<String, GoogleAuthSession>>>,
    active_listeners: Arc<Mutex<u32>>,
}

#[derive(Clone, Debug)]
struct GoogleAuthSession {
    code: Option<String>,
    error: Option<String>,
    received: bool,
    created_at_ms: u128,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleAuthStartRequest {
    pub(crate) client_id: String,
    pub(crate) scopes: Vec<String>,
    pub(crate) login_hint: Option<String>,
    pub(crate) offline_access: Option<bool>,
}

#[derive(Debug, Serialize)]
pub(crate) struct GoogleAuthStartResponse {
    authorization_url: String,
    redirect_uri: String,
    state: String,
    code_verifier: String,
    scopes: Vec<String>,
    offline_access: bool,
    expires_in_seconds: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct GoogleAuthCallbackResponse {
    state: String,
    code: Option<String>,
    error: Option<String>,
    received: bool,
}

#[tauri::command]
pub(crate) fn start_google_oauth_sign_in(
    state: tauri::State<'_, GoogleAuthState>,
    request: GoogleAuthStartRequest,
) -> Result<GoogleAuthStartResponse, String> {
    let client_id = request.client_id.trim();
    if client_id.is_empty() {
        return Err("Google OAuth client ID is required".to_string());
    }
    let scopes = normalized_google_scopes(&request.scopes);
    if scopes.is_empty() {
        return Err("At least one Google OAuth scope is required".to_string());
    }

    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|err| format!("Could not start Google OAuth callback listener: {err}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("Could not configure Google OAuth callback listener: {err}"))?;
    let port = listener
        .local_addr()
        .map_err(|err| format!("Could not read Google OAuth callback address: {err}"))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");
    let state_id = token_material()?;
    let code_verifier = token_material()?;
    let code_challenge = pkce_code_challenge(&code_verifier);
    let created_at_ms = epoch_ms();

    {
        let mut count = state
            .active_listeners
            .lock()
            .map_err(|_| "Google OAuth listener lock poisoned".to_string())?;
        if *count >= MAX_CONCURRENT_OAUTH_LISTENERS {
            return Err(
                "A Google OAuth sign-in is already in progress. Complete or cancel it first."
                    .to_string(),
            );
        }
        *count += 1;
    }

    {
        let mut sessions = state
            .sessions
            .lock()
            .map_err(|_| "Google OAuth session lock poisoned".to_string())?;
        prune_expired_sessions(&mut sessions);
        sessions.insert(
            state_id.clone(),
            GoogleAuthSession {
                code: None,
                error: None,
                received: false,
                created_at_ms,
            },
        );
    }

    let sessions = state.sessions.clone();
    let active_listeners = state.active_listeners.clone();
    let expected_state = state_id.clone();
    thread::spawn(move || {
        listen_for_google_callback(listener, sessions, expected_state);
        if let Ok(mut count) = active_listeners.lock() {
            *count = count.saturating_sub(1);
        }
    });

    let authorization_url = google_authorization_url(
        client_id,
        &redirect_uri,
        &state_id,
        &code_challenge,
        &scopes,
        request.login_hint.as_deref().unwrap_or(""),
        request.offline_access.unwrap_or(false),
    );
    Ok(GoogleAuthStartResponse {
        authorization_url,
        redirect_uri,
        state: state_id,
        code_verifier,
        scopes,
        offline_access: request.offline_access.unwrap_or(false),
        expires_in_seconds: SESSION_TTL_SECONDS,
    })
}

#[tauri::command]
pub(crate) fn poll_google_oauth_sign_in(
    state: tauri::State<'_, GoogleAuthState>,
    state_id: String,
) -> Result<GoogleAuthCallbackResponse, String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "Google OAuth session lock poisoned".to_string())?;
    prune_expired_sessions(&mut sessions);
    let Some(session) = sessions.get(&state_id).cloned() else {
        return Ok(GoogleAuthCallbackResponse {
            state: state_id,
            code: None,
            error: None,
            received: false,
        });
    };
    if session.received {
        sessions.remove(&state_id);
    }
    Ok(GoogleAuthCallbackResponse {
        state: state_id,
        code: session.code,
        error: session.error,
        received: session.received,
    })
}

#[tauri::command]
pub(crate) fn cancel_google_oauth_sign_in(
    state: tauri::State<'_, GoogleAuthState>,
    state_id: String,
) -> Result<(), String> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| "Google OAuth session lock poisoned".to_string())?;
    sessions.remove(&state_id);
    Ok(())
}

fn listen_for_google_callback(
    listener: TcpListener,
    sessions: Arc<Mutex<HashMap<String, GoogleAuthSession>>>,
    expected_state: String,
) {
    for _ in 0..(SESSION_TTL_SECONDS * 2) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let request = read_http_request(&mut stream);
                let result = parse_callback_request(&request);
                let response_html = callback_html(result.as_ref().err().map(String::as_str));
                let _ = write_http_response(&mut stream, response_html);
                if let Ok(callback) = result {
                    if callback.state == expected_state {
                        if let Ok(mut guard) = sessions.lock() {
                            if let Some(session) = guard.get_mut(&expected_state) {
                                session.code = callback.code;
                                session.error = callback.error;
                                session.received = true;
                            }
                        }
                        return;
                    }
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(500));
                if let Ok(guard) = sessions.lock() {
                    if !guard.contains_key(&expected_state) {
                        return;
                    }
                }
            }
            Err(_) => return,
        }
    }
    if let Ok(mut guard) = sessions.lock() {
        guard.remove(&expected_state);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct GoogleCallback {
    state: String,
    code: Option<String>,
    error: Option<String>,
}

fn read_http_request(stream: &mut TcpStream) -> String {
    let mut buffer = [0_u8; 4096];
    stream
        .read(&mut buffer)
        .map(|count| String::from_utf8_lossy(&buffer[..count]).to_string())
        .unwrap_or_default()
}

fn write_http_response(stream: &mut TcpStream, body: String) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes())
}

fn callback_html(error: Option<&str>) -> String {
    let message = error.unwrap_or("Google sign-in complete. Return to NEditor.");
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>NEditor Google Sign-In</title></head><body><h1>{}</h1><p>You can close this browser tab.</p></body></html>",
        crate::escape_html(message)
    )
}

fn parse_callback_request(request: &str) -> Result<GoogleCallback, String> {
    let request_line = request.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    if method != "GET" {
        return Err("Unsupported Google OAuth callback method".to_string());
    }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if path != CALLBACK_PATH {
        return Err("Unexpected Google OAuth callback path".to_string());
    }
    let params = query_params(query);
    let state = params.get("state").cloned().unwrap_or_default();
    if state.is_empty() {
        return Err("Google OAuth callback was missing state".to_string());
    }
    Ok(GoogleCallback {
        state,
        code: params.get("code").cloned(),
        error: params.get("error").cloned(),
    })
}

fn query_params(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            if pair.is_empty() {
                return None;
            }
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            Some((percent_decode(key), percent_decode(value)))
        })
        .collect()
}

fn normalized_google_scopes(scopes: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for scope in scopes {
        let scope = scope.trim();
        if !scope.starts_with("https://www.googleapis.com/auth/")
            || normalized.iter().any(|existing| existing == scope)
        {
            continue;
        }
        normalized.push(scope.to_string());
        if normalized.len() >= 8 {
            break;
        }
    }
    normalized
}

fn google_authorization_url(
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
    scopes: &[String],
    login_hint: &str,
    offline_access: bool,
) -> String {
    let mut params = vec![
        ("client_id", client_id.to_string()),
        ("redirect_uri", redirect_uri.to_string()),
        ("response_type", "code".to_string()),
        ("scope", scopes.join(" ")),
        ("state", state.to_string()),
        ("code_challenge", code_challenge.to_string()),
        ("code_challenge_method", "S256".to_string()),
        ("include_granted_scopes", "true".to_string()),
    ];
    if offline_access {
        params.push(("access_type", "offline".to_string()));
        params.push(("prompt", "consent".to_string()));
    }
    if !login_hint.trim().is_empty() {
        params.push(("login_hint", login_hint.trim().to_string()));
    }
    let query = params
        .into_iter()
        .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(&value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{GOOGLE_AUTH_ENDPOINT}?{query}")
}

fn token_material() -> Result<String, String> {
    let mut bytes = [0_u8; OAUTH_TOKEN_BYTES];
    getrandom::getrandom(&mut bytes)
        .map_err(|err| format!("Could not create secure OAuth token material: {err}"))?;
    Ok(base64url_no_pad(&bytes))
}

fn pkce_code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64url_no_pad(&digest)
}

fn base64url_no_pad(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        output.push(TABLE[(b0 >> 2) as usize] as char);
        output.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        }
        if chunk.len() > 2 {
            output.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        }
    }
    output
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                    output.push(hex);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).to_string()
}

fn prune_expired_sessions(sessions: &mut HashMap<String, GoogleAuthSession>) {
    let cutoff = epoch_ms().saturating_sub((SESSION_TTL_SECONDS as u128) * 1000);
    sessions.retain(|_, session| session.created_at_ms >= cutoff);
}

fn epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn google_oauth_authorization_url_uses_pkce_and_loopback_values() {
        let scopes = normalized_google_scopes(&[
            "https://www.googleapis.com/auth/drive.file".to_string(),
            "https://www.googleapis.com/auth/documents".to_string(),
        ]);
        let url = google_authorization_url(
            "client-id",
            "http://127.0.0.1:1234/google-oauth-callback",
            "state-123",
            "challenge-123",
            &scopes,
            "user@example.com",
            true,
        );
        assert!(url.starts_with(GOOGLE_AUTH_ENDPOINT));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
        assert!(url.contains("code_challenge=challenge-123"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A1234%2Fgoogle-oauth-callback"));
        assert!(url.contains("login_hint=user%40example.com"));
    }

    #[test]
    fn google_oauth_callback_parser_decodes_code_state_and_error() {
        let parsed = parse_callback_request(
            "GET /google-oauth-callback?code=abc%20123&state=state%2Bvalue HTTP/1.1\r\n\r\n",
        )
        .unwrap();
        assert_eq!(
            parsed,
            GoogleCallback {
                state: "state+value".to_string(),
                code: Some("abc 123".to_string()),
                error: None,
            }
        );
        let denied = parse_callback_request(
            "GET /google-oauth-callback?error=access_denied&state=s HTTP/1.1\r\n\r\n",
        )
        .unwrap();
        assert_eq!(denied.error.as_deref(), Some("access_denied"));
    }

    #[test]
    fn google_oauth_pkce_material_is_url_safe() {
        let verifier = token_material().expect("secure token material");
        let challenge = pkce_code_challenge(&verifier);
        assert!(verifier.len() >= 43);
        assert!(verifier
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'));
        assert_eq!(challenge.len(), 43);
        assert!(challenge
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'));
    }

    #[test]
    fn google_oauth_token_material_is_not_deterministic() {
        let first = token_material().expect("first token");
        let second = token_material().expect("second token");
        assert_ne!(first, second);
        assert_eq!(first.len(), 43);
        assert_eq!(second.len(), 43);
    }

    #[test]
    fn google_oauth_scope_normalizer_deduplicates_google_scopes() {
        let scopes = normalized_google_scopes(&[
            "https://www.googleapis.com/auth/drive.file".to_string(),
            "https://www.googleapis.com/auth/drive.file".to_string(),
            "profile".to_string(),
        ]);
        assert_eq!(
            scopes,
            vec!["https://www.googleapis.com/auth/drive.file".to_string()]
        );
    }
}
