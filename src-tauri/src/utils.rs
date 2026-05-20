use crate::CompileResponse;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub(crate) fn metadata_lookup<'a>(metadata: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = metadata;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

pub(crate) fn metadata_string(metadata: &Value, path: &str) -> Option<String> {
    metadata_lookup(metadata, path).map(value_to_string)
}

pub(crate) fn render_export_template(
    template: &str,
    response: &CompileResponse,
    classification: &str,
) -> String {
    template
        .replace("{{title}}", &response.semantic.title)
        .replace("{{status}}", &response.semantic.status)
        .replace("{{classification}}", classification)
        .replace("{{page}}", "1")
        .replace("{{pages}}", "1")
}

pub(crate) fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

pub(crate) fn format_value(value: f64, filter: &str) -> String {
    match filter {
        "percent" => format!("{:.2}%", value * 100.0),
        "currency" => format!("${value:.2}"),
        "round" => format!("{value:.0}"),
        _ => value.to_string(),
    }
}

pub(crate) fn sha256_uri(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn escape_xml(text: &str) -> String {
    escape_html(text).replace('\'', "&apos;")
}

pub(crate) fn escape_pdf(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .chars()
        .filter(|ch| ch.is_ascii())
        .collect()
}

pub(crate) fn escape_css(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\'', "\\'")
}
