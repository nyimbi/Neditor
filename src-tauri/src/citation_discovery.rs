use crate::{path_to_string, sha256_hex};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Deserialize)]
pub(crate) struct CitationSearchRequest {
    pub(crate) query: String,
    pub(crate) provider: Option<String>,
    pub(crate) searxng_url: Option<String>,
    pub(crate) tavily_api_key: Option<String>,
    pub(crate) limit: Option<usize>,
    pub(crate) document_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CitationSearchResponse {
    pub(crate) query: String,
    pub(crate) provider: String,
    pub(crate) associated_dir: Option<String>,
    pub(crate) results: Vec<CitationSearchResult>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CitationSearchResult {
    pub(crate) title: String,
    pub(crate) url: String,
    pub(crate) snippet: String,
    pub(crate) source: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CitationDownloadRequest {
    pub(crate) document_path: String,
    pub(crate) url: String,
    pub(crate) title: Option<String>,
    pub(crate) snippet: Option<String>,
    pub(crate) citation_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CitationDownloadResponse {
    pub(crate) path: String,
    pub(crate) relative_path: String,
    pub(crate) manifest_path: String,
    pub(crate) citation_key: String,
    pub(crate) bibliography_stub: String,
    pub(crate) bytes: usize,
}

#[tauri::command]
pub(crate) fn search_citation_sources(
    request: CitationSearchRequest,
) -> Result<CitationSearchResponse, String> {
    let query = request.query.trim();
    if query.is_empty() {
        return Err("Enter a citation search query.".to_string());
    }
    let provider = normalize_provider(request.provider.as_deref());
    let limit = request.limit.unwrap_or(8).clamp(1, 20);
    let associated_dir = request
        .document_path
        .as_deref()
        .and_then(|path| associated_source_dir(Path::new(path)).ok())
        .map(|path| path_to_string(&path));
    let results = match provider.as_str() {
        "searxng" => search_searxng(query, request.searxng_url.as_deref(), limit)?,
        "tavily" => search_tavily(query, request.tavily_api_key.as_deref(), limit)?,
        _ => search_duckduckgo(query, limit)?,
    };
    Ok(CitationSearchResponse {
        query: query.to_string(),
        provider,
        associated_dir,
        results,
    })
}

#[tauri::command]
pub(crate) fn download_citation_source(
    request: CitationDownloadRequest,
) -> Result<CitationDownloadResponse, String> {
    let document_path = Path::new(request.document_path.trim());
    if request.url.trim().is_empty() || !is_http_url(&request.url) {
        return Err("Citation source downloads require an http:// or https:// URL.".to_string());
    }
    let associated_dir = associated_source_dir(document_path)?;
    fs::create_dir_all(&associated_dir)
        .map_err(|err| format!("Could not create citation source directory: {err}"))?;
    let bytes = curl_bytes(&request.url)?;
    let key = request
        .citation_key
        .as_deref()
        .map(safe_citation_key)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| citation_key_from_title_or_url(request.title.as_deref(), &request.url));
    let extension = extension_from_url(&request.url).unwrap_or("html");
    let filename = unique_source_filename(&associated_dir, &key, extension);
    let output_path = associated_dir.join(filename);
    fs::write(&output_path, &bytes)
        .map_err(|err| format!("Could not write downloaded citation source: {err}"))?;
    let relative_path = relative_source_path(document_path, &output_path);
    let manifest_path = associated_dir.join("sources.json");
    write_source_manifest(
        &manifest_path,
        &CitationManifestItem {
            citation_key: key.clone(),
            title: request.title.clone().unwrap_or_else(|| key.clone()),
            url: request.url.clone(),
            snippet: request.snippet.unwrap_or_default(),
            path: path_to_string(&output_path),
            relative_path: relative_path.clone(),
            sha256: sha256_hex(&bytes),
            bytes: bytes.len(),
        },
    )?;
    let bibliography_stub = bibliography_stub_for_download(
        &key,
        request.title.as_deref().unwrap_or(&key),
        &request.url,
        &relative_path,
    );
    Ok(CitationDownloadResponse {
        path: path_to_string(&output_path),
        relative_path,
        manifest_path: path_to_string(&manifest_path),
        citation_key: key,
        bibliography_stub,
        bytes: bytes.len(),
    })
}

fn search_searxng(
    query: &str,
    searxng_url: Option<&str>,
    limit: usize,
) -> Result<Vec<CitationSearchResult>, String> {
    let base = searxng_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("http://127.0.0.1:8080");
    let endpoint = format!(
        "{}/search?q={}&format=json",
        base.trim_end_matches('/'),
        percent_encode(query)
    );
    let body = curl_text(&endpoint)?;
    let parsed = serde_json::from_str::<Value>(&body)
        .map_err(|err| format!("SearXNG did not return JSON search results: {err}"))?;
    let results = parsed
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let title = item.get("title").and_then(Value::as_str)?.trim();
            let url = item.get("url").and_then(Value::as_str)?.trim();
            if title.is_empty() || !is_http_url(url) {
                return None;
            }
            Some(CitationSearchResult {
                title: html_decode(title),
                url: url.to_string(),
                snippet: item
                    .get("content")
                    .or_else(|| item.get("snippet"))
                    .and_then(Value::as_str)
                    .map(html_decode)
                    .unwrap_or_default(),
                source: item
                    .get("engine")
                    .or_else(|| item.get("engines"))
                    .and_then(search_engine_label)
                    .unwrap_or_else(|| "SearXNG".to_string()),
            })
        })
        .take(limit)
        .collect::<Vec<_>>();
    Ok(results)
}

fn search_duckduckgo(query: &str, limit: usize) -> Result<Vec<CitationSearchResult>, String> {
    let endpoint = format!("https://duckduckgo.com/html/?q={}", percent_encode(query));
    let body = curl_text(&endpoint)?;
    let mut results = Vec::new();
    for chunk in body.split("result__a").skip(1) {
        let href = extract_attr_after(chunk, "href=").unwrap_or_default();
        let title = strip_tags(chunk.split("</a>").next().unwrap_or_default());
        let url = duckduckgo_result_url(&href);
        if title.trim().is_empty() || !is_http_url(&url) {
            continue;
        }
        if results
            .iter()
            .any(|item: &CitationSearchResult| item.url == url)
        {
            continue;
        }
        results.push(CitationSearchResult {
            title: html_decode(&title),
            url,
            snippet: String::new(),
            source: "DuckDuckGo".to_string(),
        });
        if results.len() >= limit {
            break;
        }
    }
    Ok(results)
}

fn search_tavily(
    query: &str,
    api_key: Option<&str>,
    limit: usize,
) -> Result<Vec<CitationSearchResult>, String> {
    let key = api_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| std::env::var("TAVILY_API_KEY").ok())
        .ok_or_else(|| {
            "Tavily search needs a session API key or TAVILY_API_KEY environment variable."
                .to_string()
        })?;
    let body = json!({
        "api_key": key,
        "query": query,
        "search_depth": "advanced",
        "max_results": limit,
        "include_answer": false,
        "include_raw_content": false
    })
    .to_string();
    let output = Command::new("curl")
        .args([
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "30",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "--data",
            &body,
            "https://api.tavily.com/search",
        ])
        .output()
        .map_err(|err| format!("Could not run curl for Tavily search: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Tavily search failed with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|err| format!("Tavily response was not UTF-8 text: {err}"))?;
    let parsed = serde_json::from_str::<Value>(&text)
        .map_err(|err| format!("Tavily did not return JSON search results: {err}"))?;
    let results = parsed
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let title = item.get("title").and_then(Value::as_str)?.trim();
            let url = item.get("url").and_then(Value::as_str)?.trim();
            if title.is_empty() || !is_http_url(url) {
                return None;
            }
            Some(CitationSearchResult {
                title: html_decode(title),
                url: url.to_string(),
                snippet: item
                    .get("content")
                    .or_else(|| item.get("snippet"))
                    .and_then(Value::as_str)
                    .map(html_decode)
                    .unwrap_or_default(),
                source: "Tavily".to_string(),
            })
        })
        .take(limit)
        .collect::<Vec<_>>();
    Ok(results)
}

fn curl_text(url: &str) -> Result<String, String> {
    let bytes = curl_bytes(url)?;
    String::from_utf8(bytes).map_err(|err| format!("Search response was not UTF-8 text: {err}"))
}

fn curl_bytes(url: &str) -> Result<Vec<u8>, String> {
    let output = Command::new("curl")
        .args([
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "30",
            "--user-agent",
            "NEditor citation acquisition",
            url,
        ])
        .output()
        .map_err(|err| format!("Could not run curl for citation source acquisition: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Citation source request failed with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    Ok(output.stdout)
}

fn associated_source_dir(document_path: &Path) -> Result<PathBuf, String> {
    if document_path.as_os_str().is_empty() {
        return Err("Save the document before downloading citation sources.".to_string());
    }
    let parent = document_path
        .parent()
        .ok_or_else(|| "Could not resolve the document folder for citation sources.".to_string())?;
    let stem = document_path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(safe_file_stem)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "document".to_string());
    Ok(parent.join(format!("{stem}.neditor-sources")))
}

fn write_source_manifest(path: &Path, item: &CitationManifestItem) -> Result<(), String> {
    let mut items = fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Vec<CitationManifestItem>>(&text).ok())
        .unwrap_or_default();
    items.retain(|existing| existing.sha256 != item.sha256 && existing.url != item.url);
    items.push(item.clone());
    let text = serde_json::to_string_pretty(&items)
        .map_err(|err| format!("Could not serialize citation source manifest: {err}"))?;
    fs::write(path, text).map_err(|err| format!("Could not write citation source manifest: {err}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CitationManifestItem {
    citation_key: String,
    title: String,
    url: String,
    snippet: String,
    path: String,
    relative_path: String,
    sha256: String,
    bytes: usize,
}

fn bibliography_stub_for_download(
    key: &str,
    title: &str,
    url: &str,
    relative_path: &str,
) -> String {
    let now = chrono::Local::now();
    let issued = format!(
        "{{\"date-parts\":[[{}, {}, {}]]}}",
        now.year(),
        now.month(),
        now.day()
    );
    let key = json_string(key);
    let title = json_string(title);
    let url = json_string(url);
    let note = json_string(&format!("Downloaded source: {relative_path}"));
    format!(
        "```bibliography\n[{{\"id\":{},\"type\":\"webpage\",\"title\":{},\"URL\":{},\"accessed\":{},\"note\":{}}}]\n```\n",
        key,
        title,
        url,
        issued,
        note
    )
}

fn normalize_provider(value: Option<&str>) -> String {
    match value
        .unwrap_or("duckduckgo")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "searx" | "searxng" => "searxng".to_string(),
        "tavily" => "tavily".to_string(),
        _ => "duckduckgo".to_string(),
    }
}

fn is_http_url(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

fn search_engine_label(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(ToString::to_string)
        .or_else(|| {
            value.as_array().map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
        })
        .filter(|value| !value.trim().is_empty())
}

fn duckduckgo_result_url(href: &str) -> String {
    let decoded = html_decode(href);
    if let Some(index) = decoded.find("uddg=") {
        let encoded = decoded[index + 5..].split('&').next().unwrap_or_default();
        return percent_decode(encoded);
    }
    decoded
        .trim_start_matches("//duckduckgo.com/l/?")
        .trim()
        .to_string()
}

fn extract_attr_after(chunk: &str, marker: &str) -> Option<String> {
    let start = chunk.find(marker)? + marker.len();
    let rest = chunk[start..].trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = rest[1..].find(quote)? + 1;
    Some(rest[1..end].to_string())
}

fn strip_tags(value: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn percent_decode(value: &str) -> String {
    let mut output = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                output.push(hex);
                index += 3;
                continue;
            }
        }
        output.push(if bytes[index] == b'+' {
            b' '
        } else {
            bytes[index]
        });
        index += 1;
    }
    String::from_utf8_lossy(&output).to_string()
}

fn html_decode(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn extension_from_url(url: &str) -> Option<&'static str> {
    let path = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
    for extension in ["pdf", "docx", "md", "markdown", "txt", "rtf", "html", "htm"] {
        if path.ends_with(&format!(".{extension}")) {
            return Some(if extension == "htm" {
                "html"
            } else {
                extension
            });
        }
    }
    None
}

fn citation_key_from_title_or_url(title: Option<&str>, url: &str) -> String {
    let base = title
        .map(safe_citation_key)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| safe_citation_key(url));
    let year = chrono::Local::now().year();
    let hash = &sha256_hex(url.as_bytes())[..8];
    format!(
        "{}{}{}",
        base.chars().take(28).collect::<String>(),
        year,
        hash
    )
}

fn safe_citation_key(value: &str) -> String {
    let mut key = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '-' | '_' | ':' | '.') && !key.ends_with('-') {
            key.push('-');
        } else if ch.is_whitespace() && !key.ends_with('-') {
            key.push('-');
        }
    }
    key.trim_matches('-').to_string()
}

fn safe_file_stem(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn unique_source_filename(dir: &Path, key: &str, extension: &str) -> String {
    let stem = safe_file_stem(key);
    let mut filename = format!("{stem}.{extension}");
    let mut index = 2;
    while dir.join(&filename).exists() {
        filename = format!("{stem}-{index}.{extension}");
        index += 1;
    }
    filename
}

fn relative_source_path(document_path: &Path, output_path: &Path) -> String {
    document_path
        .parent()
        .and_then(|parent| output_path.strip_prefix(parent).ok())
        .map(path_to_string)
        .unwrap_or_else(|| path_to_string(output_path))
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citation_source_directory_is_document_associated() {
        let dir = associated_source_dir(Path::new("/tmp/proposal.md")).expect("source dir");
        assert!(dir.ends_with("proposal.neditor-sources"));
    }

    #[test]
    fn duckduckgo_result_links_decode_target_url() {
        let url = duckduckgo_result_url(
            "//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fpaper.pdf&rut=abc",
        );
        assert_eq!(url, "https://example.com/paper.pdf");
    }
}
