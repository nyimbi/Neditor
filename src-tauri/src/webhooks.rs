use serde::{Deserialize, Serialize};
use std::process::Command;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,  // "status-changed", "exported", "approved", "saved"
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct FireWebhookRequest {
    pub url: String,
    pub event: String,
    pub document_path: Option<String>,
    pub document_title: Option<String>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct WebhookFireResult {
    pub success: bool,
    pub status_code: Option<u32>,
    pub error: Option<String>,
}

#[tauri::command]
pub(crate) fn fire_webhook(request: FireWebhookRequest) -> Result<WebhookFireResult, String> {
    let payload = serde_json::json!({
        "event": request.event,
        "timestamp": Utc::now().to_rfc3339(),
        "document": {
            "path": request.document_path,
            "title": request.document_title,
            "status": request.status,
        },
        "metadata": request.metadata,
        "source": "neditor"
    });
    let json_str = serde_json::to_string(&payload)
        .map_err(|e| format!("Failed to serialize webhook payload: {e}"))?;
    let result = Command::new("curl")
        .args([
            "-s", "-o", "/dev/null", "-w", "%{http_code}",
            "-X", "POST",
            "-H", "Content-Type: application/json",
            "-H", "User-Agent: NEditor/0.1",
            "--max-time", "10",
            "-d", &json_str,
            &request.url,
        ])
        .output();
    match result {
        Ok(out) => {
            let code_str = String::from_utf8_lossy(&out.stdout);
            let code: u32 = code_str.trim().parse().unwrap_or(0);
            Ok(WebhookFireResult { success: code >= 200 && code < 300, status_code: Some(code), error: None })
        }
        Err(e) => Ok(WebhookFireResult { success: false, status_code: None, error: Some(e.to_string()) }),
    }
}
