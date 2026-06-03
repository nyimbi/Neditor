use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaModelListRequest {
    pub(crate) endpoint: String,
    pub(crate) auth_header: Option<String>,
    pub(crate) api_key: Option<String>,
    pub(crate) key_env: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct OllamaModelListResponse {
    pub(crate) endpoint: String,
    pub(crate) models: Vec<OllamaModelSummary>,
    pub(crate) count: usize,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct OllamaModelSummary {
    pub(crate) name: String,
    pub(crate) modified_at: String,
    pub(crate) size: u64,
    pub(crate) digest: String,
    pub(crate) family: String,
    pub(crate) parameter_size: String,
    pub(crate) quantization_level: String,
}

#[tauri::command]
pub(crate) fn list_ollama_models(
    request: OllamaModelListRequest,
) -> Result<OllamaModelListResponse, String> {
    let endpoint = ollama_tags_endpoint(&request.endpoint)?;
    let body = fetch_ollama_tags(
        &endpoint,
        request.auth_header.as_deref(),
        request.api_key.as_deref(),
        request.key_env.as_deref(),
    )?;
    let models = parse_ollama_models(&body)?;
    let mut warnings = Vec::new();
    if models.is_empty() {
        warnings.push("Ollama returned no installed models. Pull a model with Ollama before selecting it in NEditor.".to_string());
    }
    Ok(OllamaModelListResponse {
        endpoint,
        count: models.len(),
        models,
        warnings,
    })
}

fn fetch_ollama_tags(
    endpoint: &str,
    auth_header: Option<&str>,
    api_key: Option<&str>,
    key_env: Option<&str>,
) -> Result<String, String> {
    let mut args = vec![
        "--location".to_string(),
        "--silent".to_string(),
        "--show-error".to_string(),
        "--max-time".to_string(),
        "15".to_string(),
        "--user-agent".to_string(),
        "NEditor Ollama model discovery".to_string(),
    ];
    if let Some((header, value)) = ollama_auth_header(auth_header, api_key, key_env) {
        args.push("-H".to_string());
        args.push(format!("{header}: {value}"));
    }
    args.push(endpoint.to_string());
    let output = Command::new("curl")
        .args(args)
        .output()
        .map_err(|err| format!("Could not run curl to list Ollama models: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Ollama model list failed with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| format!("Ollama model list response was not UTF-8 text: {err}"))
}

fn ollama_auth_header(
    auth_header: Option<&str>,
    api_key: Option<&str>,
    key_env: Option<&str>,
) -> Option<(String, String)> {
    let header = auth_header
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let key = api_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| {
            key_env
                .map(str::trim)
                .filter(|value| is_safe_env_name(value))
                .and_then(|name| std::env::var(name).ok())
        })?;
    let value = if header.eq_ignore_ascii_case("authorization")
        && !key.to_ascii_lowercase().starts_with("bearer ")
    {
        format!("Bearer {key}")
    } else {
        key
    };
    Some((header.to_string(), value))
}

fn ollama_tags_endpoint(endpoint: &str) -> Result<String, String> {
    let trimmed = endpoint.trim();
    if trimmed.is_empty() {
        return Err("Enter an Ollama endpoint before listing models.".to_string());
    }
    let lower = trimmed.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err("Ollama model discovery requires an http:// or https:// endpoint.".to_string());
    }
    let without_query = trimmed
        .split('?')
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches('/');
    for suffix in [
        "/api/chat",
        "/api/generate",
        "/api/embed",
        "/api/embeddings",
    ] {
        if without_query
            .to_ascii_lowercase()
            .ends_with(&suffix.to_ascii_lowercase())
        {
            let prefix = &without_query[..without_query.len() - suffix.len()];
            return Ok(format!("{prefix}/api/tags"));
        }
    }
    if without_query.to_ascii_lowercase().ends_with("/api/tags") {
        return Ok(without_query.to_string());
    }
    if without_query.to_ascii_lowercase().ends_with("/api") {
        return Ok(format!("{without_query}/tags"));
    }
    Ok(format!("{without_query}/api/tags"))
}

fn parse_ollama_models(body: &str) -> Result<Vec<OllamaModelSummary>, String> {
    let parsed = serde_json::from_str::<Value>(body)
        .map_err(|err| format!("Ollama did not return JSON model data: {err}"))?;
    let models = parsed
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| "Ollama model list response did not include a models array.".to_string())?;
    let mut summaries = models
        .iter()
        .filter_map(|model| {
            let name = model
                .get("name")
                .or_else(|| model.get("model"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())?;
            let details = model.get("details").unwrap_or(&Value::Null);
            Some(OllamaModelSummary {
                name: name.to_string(),
                modified_at: string_value(model.get("modified_at")),
                size: model.get("size").and_then(Value::as_u64).unwrap_or(0),
                digest: string_value(model.get("digest")),
                family: string_value(details.get("family")),
                parameter_size: string_value(details.get("parameter_size")),
                quantization_level: string_value(details.get("quantization_level")),
            })
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(summaries)
}

fn string_value(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default()
        .to_string()
}

fn is_safe_env_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[derive(Debug, Serialize)]
pub(crate) struct OllamaHealthResult {
    pub(crate) running: bool,
    pub(crate) endpoint: String,
    pub(crate) model_count: usize,
    pub(crate) version: String,
    pub(crate) error: String,
}

#[tauri::command]
pub(crate) fn check_ollama_health(endpoint: String) -> OllamaHealthResult {
    let base = endpoint
        .trim_end_matches('/')
        .trim_end_matches("/api/chat")
        .trim_end_matches("/api/generate")
        .trim_end_matches("/api/tags");
    let tags_url = format!("{base}/api/tags");
    let output = std::process::Command::new("curl")
        .args(["-sL", "--max-time", "3", "--write-out", "\n%{http_code}", &tags_url])
        .output();
    match output {
        Err(e) => OllamaHealthResult {
            running: false, endpoint: base.to_string(), model_count: 0,
            version: String::new(), error: e.to_string(),
        },
        Ok(out) => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let (body, status_str) = raw.rsplit_once('\n').unwrap_or((&raw, "0"));
            let status: u32 = status_str.trim().parse().unwrap_or(0);
            if status < 200 || status >= 300 {
                return OllamaHealthResult {
                    running: false, endpoint: base.to_string(), model_count: 0,
                    version: String::new(), error: format!("HTTP {status}"),
                };
            }
            let model_count = serde_json::from_str::<serde_json::Value>(body.trim())
                .ok()
                .and_then(|v| v.get("models")?.as_array().map(|a| a.len()))
                .unwrap_or(0);
            let version = {
                let version_url = format!("{base}/api/version");
                std::process::Command::new("curl")
                    .args(["-sL", "--max-time", "2", &version_url])
                    .output()
                    .ok()
                    .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok())
                    .and_then(|v| v.get("version")?.as_str().map(String::from))
                    .unwrap_or_default()
            };
            OllamaHealthResult { running: true, endpoint: base.to_string(), model_count, version, error: String::new() }
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaPullRequest {
    pub(crate) endpoint: String,
    pub(crate) model: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct OllamaPullResult {
    pub(crate) success: bool,
    pub(crate) model: String,
    pub(crate) error: String,
    pub(crate) status: String,
}

#[tauri::command]
pub(crate) fn pull_ollama_model(request: OllamaPullRequest) -> Result<OllamaPullResult, String> {
    let base = request.endpoint.trim_end_matches('/')
        .trim_end_matches("/api/chat").trim_end_matches("/api/generate").trim_end_matches("/api/tags");
    let pull_url = format!("{base}/api/pull");
    let body = serde_json::json!({ "name": &request.model, "stream": false }).to_string();
    let output = std::process::Command::new("curl")
        .args(["-sL", "--max-time", "600", "-X", "POST",
               "-H", "Content-Type: application/json", "-d", &body, &pull_url])
        .output().map_err(|e| e.to_string())?;
    let raw = String::from_utf8_lossy(&output.stdout);
    let status_msg = serde_json::from_str::<serde_json::Value>(raw.trim())
        .ok().and_then(|v| v.get("status")?.as_str().map(String::from)).unwrap_or_default();
    let success = output.status.success() && (status_msg == "success" || raw.contains("\"success\""));
    Ok(OllamaPullResult {
        success, model: request.model,
        error: if success { String::new() } else { raw.trim().chars().take(300).collect() },
        status: status_msg,
    })
}

#[derive(Debug, Deserialize)]
pub(crate) struct OllamaDeleteRequest {
    pub(crate) endpoint: String,
    pub(crate) model: String,
}

#[tauri::command]
pub(crate) fn delete_ollama_model(request: OllamaDeleteRequest) -> Result<(), String> {
    let base = request.endpoint.trim_end_matches('/')
        .trim_end_matches("/api/chat").trim_end_matches("/api/generate").trim_end_matches("/api/tags");
    let delete_url = format!("{base}/api/delete");
    let body = serde_json::json!({ "name": &request.model }).to_string();
    let output = std::process::Command::new("curl")
        .args(["-sL", "--max-time", "30", "-X", "DELETE",
               "-H", "Content-Type: application/json", "-d", &body, &delete_url])
        .output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Delete failed: {}", err.trim().chars().take(200).collect::<String>()));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub(crate) struct OllamaModelInfo {
    pub(crate) name: String,
    pub(crate) family: String,
    pub(crate) parameter_size: String,
    pub(crate) context_length: u64,
    pub(crate) quantization: String,
}

#[tauri::command]
pub(crate) fn show_ollama_model_info(endpoint: String, model: String) -> Result<OllamaModelInfo, String> {
    let base = endpoint.trim_end_matches('/')
        .trim_end_matches("/api/chat").trim_end_matches("/api/generate").trim_end_matches("/api/tags");
    let show_url = format!("{base}/api/show");
    let body = serde_json::json!({ "name": &model }).to_string();
    let output = std::process::Command::new("curl")
        .args(["-sL", "--max-time", "10", "-X", "POST",
               "-H", "Content-Type: application/json", "-d", &body, &show_url])
        .output().map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let details = v.get("details");
    let modelinfo = v.get("model_info");
    let context_length = modelinfo
        .and_then(|m| m.get("llama.context_length").or_else(|| m.get("context_length")))
        .and_then(|c| c.as_u64()).unwrap_or(0);
    Ok(OllamaModelInfo {
        name: model,
        family: details.and_then(|d| d.get("family")?.as_str()).unwrap_or_default().to_string(),
        parameter_size: details.and_then(|d| d.get("parameter_size")?.as_str()).unwrap_or_default().to_string(),
        context_length,
        quantization: details.and_then(|d| d.get("quantization_level")?.as_str()).unwrap_or_default().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_tags_endpoint_normalizes_native_chat_urls() {
        assert_eq!(
            ollama_tags_endpoint("http://127.0.0.1:11434/api/chat").unwrap(),
            "http://127.0.0.1:11434/api/tags"
        );
        assert_eq!(
            ollama_tags_endpoint("https://ollama.example/team/api/generate").unwrap(),
            "https://ollama.example/team/api/tags"
        );
        assert_eq!(
            ollama_tags_endpoint("http://127.0.0.1:11434").unwrap(),
            "http://127.0.0.1:11434/api/tags"
        );
    }

    #[test]
    fn parse_ollama_models_reads_model_names_and_details() {
        let models = parse_ollama_models(
            r#"{
              "models": [
                {
                  "name": "mistral:latest",
                  "modified_at": "2026-05-28T10:00:00Z",
                  "size": 4113301824,
                  "digest": "abc123",
                  "details": {
                    "family": "llama",
                    "parameter_size": "7B",
                    "quantization_level": "Q4_0"
                  }
                },
                { "model": "llama3.1:8b" }
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(models[0].name, "llama3.1:8b");
        assert_eq!(models[1].name, "mistral:latest");
        assert_eq!(models[1].parameter_size, "7B");
        assert_eq!(models[1].quantization_level, "Q4_0");
    }
}
