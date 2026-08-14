use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: String,
    pub url: String,
    pub events: Vec<String>, // "status-changed", "exported", "approved", "saved"
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
    /// Set to `true` to allow webhook delivery to RFC 6890 private IP ranges
    /// (e.g. for localhost dev servers). Absent / false = public-only.
    #[serde(default)]
    pub allow_private: bool,
}

#[derive(Debug, Serialize)]
pub struct WebhookFireResult {
    pub success: bool,
    pub status_code: Option<u32>,
    pub error: Option<String>,
}

#[tauri::command]
pub(crate) fn fire_webhook(request: FireWebhookRequest) -> Result<WebhookFireResult, String> {
    // G5: validate URL scheme and control characters.
    let validated_url = crate::net_guard::validate_http_url(&request.url, "Webhook URL")?;

    // G5: RFC 6890 private-range denial unless the caller opts in.
    if !request.allow_private && !crate::net_guard::is_public_destination(&validated_url) {
        return Err("Webhook URL resolves to a private or loopback address; \
             set allow_private to true to deliver to local endpoints."
            .to_string());
    }

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
            "--proto",
            "=http,https",
            "--proto-redir",
            "=http,https",
            "--max-redirs",
            "5",
            "--max-filesize",
            "1048576", // 1 MiB response cap for webhooks
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-H",
            "User-Agent: NEditor/0.1",
            "--max-time",
            "10",
            "-d",
            &json_str,
            "--", // G5: prevent URL starting with "-" being parsed as a curl flag
            &validated_url,
        ])
        .output();

    match result {
        Ok(out) => {
            let code_str = String::from_utf8_lossy(&out.stdout);
            let code: u32 = code_str.trim().parse().unwrap_or(0);
            Ok(WebhookFireResult {
                success: code >= 200 && code < 300,
                status_code: Some(code),
                error: None,
            })
        }
        Err(e) => Ok(WebhookFireResult {
            success: false,
            status_code: None,
            error: Some(e.to_string()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_webhook_rejects_file_url() {
        let req = FireWebhookRequest {
            url: "file:///etc/passwd".to_string(),
            event: "test".to_string(),
            document_path: None,
            document_title: None,
            status: None,
            metadata: None,
            allow_private: false,
        };
        let err = fire_webhook(req).unwrap_err();
        assert!(err.contains("http://") || err.contains("https://"));
    }

    #[test]
    fn fire_webhook_rejects_private_ip() {
        let req = FireWebhookRequest {
            url: "http://169.254.169.254/meta-data/".to_string(),
            event: "test".to_string(),
            document_path: None,
            document_title: None,
            status: None,
            metadata: None,
            allow_private: false,
        };
        let err = fire_webhook(req).unwrap_err();
        assert!(err.contains("private") || err.contains("loopback"));
    }

    #[test]
    fn fire_webhook_allows_private_when_opted_in() {
        // With allow_private=true the private-range guard is bypassed.
        // The actual curl call will fail (no server), but validation passes.
        let req = FireWebhookRequest {
            url: "http://127.0.0.1:9999/hook".to_string(),
            event: "test".to_string(),
            document_path: None,
            document_title: None,
            status: None,
            metadata: None,
            allow_private: true,
        };
        // Should not return an Err from validation (curl failure is Ok variant).
        // If curl is not installed this returns Err — accept both Ok and Err
        // but not the private-range error.
        match fire_webhook(req) {
            Ok(_) => {}
            Err(e) => {
                assert!(
                    !e.contains("private"),
                    "should not fail on private-range check when allow_private=true, got: {e}"
                );
            }
        }
    }

    #[test]
    fn fire_webhook_rejects_crlf_in_url() {
        let req = FireWebhookRequest {
            url: "https://example.com/\r\nX-Evil: injected".to_string(),
            event: "test".to_string(),
            document_path: None,
            document_title: None,
            status: None,
            metadata: None,
            allow_private: false,
        };
        assert!(fire_webhook(req).is_err());
    }
}
