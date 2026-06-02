use crate::{path_to_string, sha256_hex};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const MAX_CITATION_SOURCE_DOWNLOAD_BYTES: usize = 25 * 1024 * 1024;

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
    pub(crate) source: Option<String>,
    pub(crate) citation_key: Option<String>,
    pub(crate) fit_score: Option<u8>,
    pub(crate) fit_label: Option<String>,
    pub(crate) fit_reasons: Option<Vec<String>>,
    pub(crate) force_refresh: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CitationSourceLibraryRequest {
    pub(crate) document_path: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CitationDownloadResponse {
    pub(crate) path: String,
    pub(crate) relative_path: String,
    pub(crate) manifest_path: String,
    pub(crate) source_dir: String,
    pub(crate) citation_key: String,
    pub(crate) bibliography_stub: String,
    pub(crate) bytes: usize,
    pub(crate) sha256: String,
    pub(crate) downloaded_at: Option<String>,
    pub(crate) media_type: Option<String>,
    pub(crate) reused: bool,
    pub(crate) manifest_entry_count: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct CitationSourceLibraryResponse {
    pub(crate) associated_dir: String,
    pub(crate) manifest_path: String,
    pub(crate) sources: Vec<CitationManifestItem>,
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
        "local-library" => {
            search_local_source_library(query, request.document_path.as_deref(), limit)?
        }
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
    let manifest_path = associated_dir.join("sources.json");
    if request.force_refresh != Some(true) {
        if let Some(existing) = read_source_manifest(&manifest_path)?
            .into_iter()
            .find(|item| item.url == request.url && Path::new(&item.path).exists())
        {
            return citation_download_response(
                document_path,
                &associated_dir,
                &manifest_path,
                existing,
                true,
            );
        }
    }
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
    let sha256 = sha256_hex(&bytes);
    let downloaded_at = chrono::Local::now().to_rfc3339();
    write_source_manifest(
        &manifest_path,
        &CitationManifestItem {
            citation_key: key.clone(),
            title: request.title.clone().unwrap_or_else(|| key.clone()),
            url: request.url.clone(),
            snippet: request.snippet.unwrap_or_default(),
            source: request.source.as_deref().and_then(normalize_manifest_text),
            path: path_to_string(&output_path),
            relative_path: relative_path.clone(),
            sha256,
            bytes: bytes.len(),
            downloaded_at: Some(downloaded_at),
            media_type: media_type_from_extension(extension).map(ToString::to_string),
            fit_score: request.fit_score.map(|score| score.min(100)),
            fit_label: request
                .fit_label
                .as_deref()
                .and_then(normalize_manifest_text),
            fit_reasons: normalize_fit_reasons(request.fit_reasons.as_deref()),
            file_exists: None,
            hash_matches: None,
            current_sha256: None,
            current_bytes: None,
        },
    )?;
    let item = read_source_manifest(&manifest_path)?
        .into_iter()
        .find(|item| item.url == request.url)
        .ok_or_else(|| {
            "Downloaded citation source was not recorded in the manifest.".to_string()
        })?;
    citation_download_response(document_path, &associated_dir, &manifest_path, item, false)
}

#[tauri::command]
pub(crate) fn list_citation_sources(
    request: CitationSourceLibraryRequest,
) -> Result<CitationSourceLibraryResponse, String> {
    let document_path = Path::new(request.document_path.trim());
    let associated_dir = associated_source_dir(document_path)?;
    let manifest_path = associated_dir.join("sources.json");
    let mut sources = read_source_manifest(&manifest_path)?;
    for source in &mut sources {
        annotate_source_file_status(source);
    }
    sources.sort_by(|left, right| {
        right
            .downloaded_at
            .cmp(&left.downloaded_at)
            .then_with(|| left.citation_key.cmp(&right.citation_key))
    });
    Ok(CitationSourceLibraryResponse {
        associated_dir: path_to_string(&associated_dir),
        manifest_path: path_to_string(&manifest_path),
        sources,
    })
}

fn search_searxng(
    query: &str,
    searxng_url: Option<&str>,
    limit: usize,
) -> Result<Vec<CitationSearchResult>, String> {
    let base = validate_http_url(
        searxng_url
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("http://127.0.0.1:8080"),
        "SearXNG URL",
    )?;
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

fn search_local_source_library(
    query: &str,
    document_path: Option<&str>,
    limit: usize,
) -> Result<Vec<CitationSearchResult>, String> {
    let document_path = document_path
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| {
            "Save the document before searching the local source library.".to_string()
        })?;
    let document_path = Path::new(document_path);
    let associated_dir = associated_source_dir(document_path)?;
    let manifest_path = associated_dir.join("sources.json");
    let query_terms = local_search_terms(query);
    let mut scored = read_source_manifest(&manifest_path)?
        .into_iter()
        .filter_map(|mut item| {
            annotate_source_file_status(&mut item);
            let content_preview = local_source_content_preview(&item, &query_terms);
            let score = local_source_score(&item, &query_terms, content_preview.as_deref());
            if score == 0 {
                return None;
            }
            let snippet = local_source_search_snippet(&item, content_preview.as_deref(), score);
            Some((
                score,
                item.downloaded_at.clone().unwrap_or_default(),
                CitationSearchResult {
                    title: item.title.clone(),
                    url: item.url.clone(),
                    snippet,
                    source: format!("Local source library (@{})", item.citation_key),
                },
            ))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| left.2.title.cmp(&right.2.title))
    });
    Ok(scored
        .into_iter()
        .take(limit)
        .map(|(_, _, result)| result)
        .collect())
}

fn curl_text(url: &str) -> Result<String, String> {
    let bytes = curl_bytes(url)?;
    String::from_utf8(bytes).map_err(|err| format!("Search response was not UTF-8 text: {err}"))
}

fn curl_bytes(url: &str) -> Result<Vec<u8>, String> {
    let url = validate_http_url(url, "Citation source URL")?;
    let output = Command::new("curl")
        .args([
            "--location",
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "30",
            "--proto",
            "=http,https",
            "--proto-redir",
            "=http,https",
            "--max-filesize",
            &MAX_CITATION_SOURCE_DOWNLOAD_BYTES.to_string(),
            "--user-agent",
            "NEditor citation acquisition",
            &url,
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
    if output.stdout.len() > MAX_CITATION_SOURCE_DOWNLOAD_BYTES {
        return Err(format!(
            "Citation source response is {} bytes, above the {} byte limit.",
            output.stdout.len(),
            MAX_CITATION_SOURCE_DOWNLOAD_BYTES
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
    let mut items = read_source_manifest(path)?;
    items.retain(|existing| existing.sha256 != item.sha256 && existing.url != item.url);
    let mut persisted = item.clone();
    persisted.file_exists = None;
    persisted.hash_matches = None;
    persisted.current_sha256 = None;
    persisted.current_bytes = None;
    items.push(persisted);
    let text = serde_json::to_string_pretty(&items)
        .map_err(|err| format!("Could not serialize citation source manifest: {err}"))?;
    fs::write(path, text).map_err(|err| format!("Could not write citation source manifest: {err}"))
}

fn read_source_manifest(path: &Path) -> Result<Vec<CitationManifestItem>, String> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str::<Vec<CitationManifestItem>>(&text)
            .map_err(|err| format!("Could not read citation source manifest: {err}")),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(format!("Could not read citation source manifest: {err}")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CitationManifestItem {
    citation_key: String,
    title: String,
    url: String,
    snippet: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    path: String,
    relative_path: String,
    sha256: String,
    bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    downloaded_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    media_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fit_score: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fit_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fit_reasons: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file_exists: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hash_matches: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current_bytes: Option<usize>,
}

fn annotate_source_file_status(source: &mut CitationManifestItem) {
    match fs::read(&source.path) {
        Ok(bytes) => {
            let current_sha256 = sha256_hex(&bytes);
            source.file_exists = Some(true);
            source.hash_matches =
                Some(current_sha256 == source.sha256 && bytes.len() == source.bytes);
            source.current_sha256 = Some(current_sha256);
            source.current_bytes = Some(bytes.len());
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            source.file_exists = Some(false);
            source.hash_matches = None;
            source.current_sha256 = None;
            source.current_bytes = None;
        }
        Err(_) => {
            source.file_exists = Some(false);
            source.hash_matches = None;
            source.current_sha256 = None;
            source.current_bytes = None;
        }
    }
}

fn citation_download_response(
    document_path: &Path,
    associated_dir: &Path,
    manifest_path: &Path,
    item: CitationManifestItem,
    reused: bool,
) -> Result<CitationDownloadResponse, String> {
    let manifest_entry_count = read_source_manifest(manifest_path)?.len();
    let relative_path = if item.relative_path.trim().is_empty() {
        relative_source_path(document_path, Path::new(&item.path))
    } else {
        item.relative_path.clone()
    };
    let bibliography_stub =
        bibliography_stub_for_download(&item.citation_key, &item.title, &item.url, &relative_path);
    Ok(CitationDownloadResponse {
        path: item.path,
        relative_path,
        manifest_path: path_to_string(manifest_path),
        source_dir: path_to_string(associated_dir),
        citation_key: item.citation_key,
        bibliography_stub,
        bytes: item.bytes,
        sha256: item.sha256,
        downloaded_at: item.downloaded_at,
        media_type: item.media_type,
        reused,
        manifest_entry_count,
    })
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
        "local" | "library" | "local-library" | "source-library" | "saved-sources" => {
            "local-library".to_string()
        }
        "searx" | "searxng" => "searxng".to_string(),
        "tavily" => "tavily".to_string(),
        _ => "duckduckgo".to_string(),
    }
}

fn local_search_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    for term in query
        .split(|ch: char| !ch.is_alphanumeric())
        .map(|term| term.trim().to_ascii_lowercase())
        .filter(|term| term.len() > 1)
    {
        if !terms.contains(&term) {
            terms.push(term);
        }
    }
    terms
}

fn local_source_score(
    item: &CitationManifestItem,
    query_terms: &[String],
    content_preview: Option<&str>,
) -> u32 {
    if query_terms.is_empty() {
        return 0;
    }
    let title = item.title.to_ascii_lowercase();
    let snippet = item.snippet.to_ascii_lowercase();
    let source = item.source.clone().unwrap_or_default().to_ascii_lowercase();
    let path = format!(
        "{} {} {} {}",
        item.url, item.relative_path, item.citation_key, source
    )
    .to_ascii_lowercase();
    let content = content_preview.unwrap_or_default().to_ascii_lowercase();
    let mut score: u32 = 0;
    for term in query_terms {
        if title.contains(term) {
            score += 8;
        }
        if snippet.contains(term) {
            score += 5;
        }
        if path.contains(term) {
            score += 3;
        }
        if content.contains(term) {
            score += 2;
        }
    }
    if score > 0 && item.file_exists == Some(true) {
        score += 2;
    }
    if score > 0 && (item.hash_matches == Some(false) || item.file_exists == Some(false)) {
        score = score.saturating_sub(3);
    }
    score
}

fn local_source_content_preview(
    item: &CitationManifestItem,
    query_terms: &[String],
) -> Option<String> {
    if item.file_exists == Some(false) {
        return None;
    }
    let metadata = fs::metadata(&item.path).ok()?;
    if metadata.len() > 2 * 1024 * 1024 {
        return None;
    }
    let bytes = fs::read(&item.path).ok()?;
    let raw = String::from_utf8_lossy(&bytes);
    let cleaned = if item
        .media_type
        .as_deref()
        .unwrap_or_default()
        .contains("html")
        || item.path.to_ascii_lowercase().ends_with(".html")
    {
        strip_tags(&raw)
    } else {
        raw.to_string()
    };
    let compact = collapse_whitespace(&cleaned);
    if compact.is_empty() {
        return None;
    }
    let lower = compact.to_ascii_lowercase();
    let start = query_terms
        .iter()
        .filter_map(|term| lower.find(term))
        .min()
        .unwrap_or(0)
        .saturating_sub(80);
    Some(trim_to_char_boundary(&compact, start, 360))
}

fn local_source_search_snippet(
    item: &CitationManifestItem,
    content_preview: Option<&str>,
    score: u32,
) -> String {
    let integrity = match (item.file_exists, item.hash_matches) {
        (Some(false), _) => "Local file missing; re-download before relying on this source.",
        (Some(true), Some(false)) => {
            "Local file changed after download; verify before relying on this source."
        }
        _ => "Local copy available.",
    };
    let evidence = content_preview
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            let snippet = item.snippet.trim();
            if snippet.is_empty() {
                None
            } else {
                Some(snippet)
            }
        })
        .unwrap_or("No saved snippet; open the local file for review.");
    format!(
        "Saved @{} at {}. Match score: {}. {} {}",
        item.citation_key, item.relative_path, score, integrity, evidence
    )
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn trim_to_char_boundary(value: &str, start: usize, max_len: usize) -> String {
    let start = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= start)
        .last()
        .unwrap_or(0);
    value[start..].chars().take(max_len).collect()
}

fn is_http_url(value: &str) -> bool {
    let lower = value.trim().to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

fn validate_http_url(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if !is_http_url(trimmed) {
        return Err(format!("{label} must be an http:// or https:// URL."));
    }
    if trimmed
        .chars()
        .any(|character| character == '\0' || character == '\n' || character == '\r')
    {
        return Err(format!("{label} cannot contain control characters."));
    }
    Ok(trimmed.to_string())
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

fn media_type_from_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "pdf" => Some("application/pdf"),
        "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
        "md" | "markdown" => Some("text/markdown"),
        "txt" => Some("text/plain"),
        "rtf" => Some("application/rtf"),
        "html" | "htm" => Some("text/html"),
        _ => None,
    }
}

fn normalize_manifest_text(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.chars().take(240).collect())
    }
}

fn normalize_fit_reasons(value: Option<&[String]>) -> Option<Vec<String>> {
    let reasons = value?
        .iter()
        .filter_map(|reason| normalize_manifest_text(reason))
        .take(6)
        .collect::<Vec<_>>();
    if reasons.is_empty() {
        None
    } else {
        Some(reasons)
    }
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

#[derive(Debug, Deserialize)]
pub struct DoiLookupRequest {
    pub doi: String,
}

#[tauri::command]
pub(crate) fn lookup_doi(request: DoiLookupRequest) -> Result<String, String> {
    let doi = request.doi.trim().trim_start_matches("https://doi.org/").trim_start_matches("doi:");
    let url = format!("https://api.crossref.org/works/{}/transform/application/x-bibtex", doi);
    let output = std::process::Command::new("curl")
        .args(["-sL", "--max-time", "10", "--user-agent", "NEditor/0.1 (mailto:support@neditor.app)", &url])
        .output()
        .map_err(|e| e.to_string())?;
    let bibtex = String::from_utf8_lossy(&output.stdout).to_string();
    if bibtex.trim().is_empty() || !bibtex.contains('@') {
        return Err(format!("No BibTeX found for DOI: {doi}. Check the DOI is correct."));
    }
    Ok(bibtex)
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

    #[test]
    fn citation_fetch_urls_must_be_http_without_control_characters() {
        assert_eq!(
            validate_http_url(" https://example.com/source.pdf ", "Citation source URL").unwrap(),
            "https://example.com/source.pdf"
        );
        assert!(
            validate_http_url("file:///etc/passwd", "Citation source URL")
                .expect_err("file URL rejected")
                .contains("http:// or https://")
        );
        assert!(validate_http_url(
            "https://example.com/source.pdf\nX-Test: injected",
            "Citation source URL"
        )
        .expect_err("control characters rejected")
        .contains("control characters"));
    }

    #[test]
    fn citation_source_manifest_replaces_duplicate_urls() {
        let dir =
            std::env::temp_dir().join(format!("neditor-citation-manifest-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let manifest = dir.join("sources.json");
        let first = CitationManifestItem {
            citation_key: "first".to_string(),
            title: "First".to_string(),
            url: "https://example.com/source.pdf".to_string(),
            snippet: String::new(),
            path: path_to_string(&dir.join("first.pdf")),
            relative_path: "doc.neditor-sources/first.pdf".to_string(),
            sha256: "first-hash".to_string(),
            bytes: 12,
            downloaded_at: Some("2026-05-28T10:00:00+03:00".to_string()),
            media_type: Some("application/pdf".to_string()),
            source: Some("SearXNG".to_string()),
            fit_score: Some(82),
            fit_label: Some("strong".to_string()),
            fit_reasons: Some(vec![
                "government source domain".to_string(),
                "downloadable PDF source".to_string(),
            ]),
            file_exists: Some(true),
            hash_matches: Some(true),
            current_sha256: Some("ignored-live-status".to_string()),
            current_bytes: Some(999),
        };
        let second = CitationManifestItem {
            citation_key: "second".to_string(),
            title: "Second".to_string(),
            sha256: "second-hash".to_string(),
            bytes: 24,
            ..first.clone()
        };
        write_source_manifest(&manifest, &first).expect("write first");
        write_source_manifest(&manifest, &second).expect("write second");
        let items = read_source_manifest(&manifest).expect("read manifest");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].citation_key, "second");
        assert_eq!(items[0].bytes, 24);
        assert_eq!(items[0].fit_score, Some(82));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_source_library_search_ranks_saved_sources() {
        let dir = std::env::temp_dir().join(format!(
            "neditor-local-source-search-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("temp dir");
        let document = dir.join("proposal.md");
        fs::write(&document, "# Proposal\n").expect("document");
        let source_dir = associated_source_dir(&document).expect("source dir");
        fs::create_dir_all(&source_dir).expect("source dir create");
        let climate_path = source_dir.join("climate-procurement.html");
        let generic_path = source_dir.join("generic.html");
        fs::write(
            &climate_path,
            "<html><body>Climate procurement controls evidence and audit timeline.</body></html>",
        )
        .expect("climate file");
        fs::write(&generic_path, "General business guidance.").expect("generic file");
        let manifest = source_dir.join("sources.json");
        write_source_manifest(
            &manifest,
            &CitationManifestItem {
                citation_key: "generic".to_string(),
                title: "Generic Guidance".to_string(),
                url: "https://example.com/generic.html".to_string(),
                snippet: "General business guidance.".to_string(),
                path: path_to_string(&generic_path),
                relative_path: relative_source_path(&document, &generic_path),
                sha256: sha256_hex(b"General business guidance."),
                bytes: "General business guidance.".len(),
                downloaded_at: Some("2026-05-28T09:00:00+03:00".to_string()),
                media_type: Some("text/html".to_string()),
                source: Some("DuckDuckGo".to_string()),
                fit_score: Some(40),
                fit_label: Some("weak".to_string()),
                fit_reasons: Some(vec!["generic source".to_string()]),
                file_exists: None,
                hash_matches: None,
                current_sha256: None,
                current_bytes: None,
            },
        )
        .expect("write generic");
        write_source_manifest(
            &manifest,
            &CitationManifestItem {
                citation_key: "climate".to_string(),
                title: "Climate Procurement Evidence".to_string(),
                url: "https://agency.gov/climate-procurement.html".to_string(),
                snippet: "Saved source about controls.".to_string(),
                path: path_to_string(&climate_path),
                relative_path: relative_source_path(&document, &climate_path),
                sha256: sha256_hex(
                    b"<html><body>Climate procurement controls evidence and audit timeline.</body></html>",
                ),
                bytes: "<html><body>Climate procurement controls evidence and audit timeline.</body></html>".len(),
                downloaded_at: Some("2026-05-28T10:00:00+03:00".to_string()),
                media_type: Some("text/html".to_string()),
                source: Some("SearXNG".to_string()),
                fit_score: Some(91),
                fit_label: Some("strong".to_string()),
                fit_reasons: Some(vec!["saved source".to_string()]),
                file_exists: None,
                hash_matches: None,
                current_sha256: None,
                current_bytes: None,
            },
        )
        .expect("write climate");

        let response = search_citation_sources(CitationSearchRequest {
            query: "climate procurement evidence".to_string(),
            provider: Some("source-library".to_string()),
            searxng_url: None,
            tavily_api_key: None,
            limit: Some(5),
            document_path: Some(path_to_string(&document)),
        })
        .expect("search local library");
        assert_eq!(response.provider, "local-library");
        assert!(response
            .associated_dir
            .unwrap()
            .ends_with("proposal.neditor-sources"));
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].title, "Climate Procurement Evidence");
        assert!(response.results[0].source.contains("@climate"));
        assert!(response.results[0].snippet.contains("Match score"));
        assert!(response.results[0]
            .snippet
            .contains("Climate procurement controls evidence"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn citation_source_library_lists_manifest_items_newest_first() {
        let dir =
            std::env::temp_dir().join(format!("neditor-citation-library-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let document = dir.join("proposal.md");
        fs::write(&document, "# Proposal\n").expect("document");
        let source_dir = associated_source_dir(&document).expect("source dir");
        fs::create_dir_all(&source_dir).expect("source dir create");
        let older_path = source_dir.join("older.html");
        let newer_path = source_dir.join("newer.pdf");
        fs::write(&older_path, "older").expect("older file");
        fs::write(&newer_path, "newer").expect("newer file");
        let manifest = source_dir.join("sources.json");
        write_source_manifest(
            &manifest,
            &CitationManifestItem {
                citation_key: "older".to_string(),
                title: "Older".to_string(),
                url: "https://example.com/older.html".to_string(),
                snippet: String::new(),
                path: path_to_string(&older_path),
                relative_path: relative_source_path(&document, &older_path),
                sha256: sha256_hex(b"older"),
                bytes: 5,
                downloaded_at: Some("2026-05-27T10:00:00+03:00".to_string()),
                media_type: Some("text/html".to_string()),
                source: Some("DuckDuckGo".to_string()),
                fit_score: Some(44),
                fit_label: Some("review".to_string()),
                fit_reasons: Some(vec!["reviewable text source".to_string()]),
                file_exists: None,
                hash_matches: None,
                current_sha256: None,
                current_bytes: None,
            },
        )
        .expect("write older");
        write_source_manifest(
            &manifest,
            &CitationManifestItem {
                citation_key: "newer".to_string(),
                title: "Newer".to_string(),
                url: "https://example.com/newer.pdf".to_string(),
                snippet: String::new(),
                path: path_to_string(&newer_path),
                relative_path: relative_source_path(&document, &newer_path),
                sha256: sha256_hex(b"newer"),
                bytes: 5,
                downloaded_at: Some("2026-05-28T10:00:00+03:00".to_string()),
                media_type: Some("application/pdf".to_string()),
                source: Some("SearXNG".to_string()),
                fit_score: Some(91),
                fit_label: Some("strong".to_string()),
                fit_reasons: Some(vec!["downloadable PDF source".to_string()]),
                file_exists: None,
                hash_matches: None,
                current_sha256: None,
                current_bytes: None,
            },
        )
        .expect("write newer");
        let library = list_citation_sources(CitationSourceLibraryRequest {
            document_path: path_to_string(&document),
        })
        .expect("library");
        assert!(library.associated_dir.ends_with("proposal.neditor-sources"));
        assert_eq!(library.sources.len(), 2);
        assert_eq!(library.sources[0].citation_key, "newer");
        assert_eq!(library.sources[0].source.as_deref(), Some("SearXNG"));
        assert_eq!(library.sources[0].fit_label.as_deref(), Some("strong"));
        assert_eq!(library.sources[0].file_exists, Some(true));
        assert_eq!(library.sources[0].hash_matches, Some(true));
        assert_eq!(library.sources[0].current_bytes, Some(5));
        assert_eq!(library.sources[1].citation_key, "older");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn citation_source_library_marks_missing_files_without_persisting_status() {
        let dir =
            std::env::temp_dir().join(format!("neditor-citation-missing-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let document = dir.join("proposal.md");
        fs::write(&document, "# Proposal\n").expect("document");
        let source_dir = associated_source_dir(&document).expect("source dir");
        fs::create_dir_all(&source_dir).expect("source dir create");
        let missing_path = source_dir.join("missing.pdf");
        let manifest = source_dir.join("sources.json");
        write_source_manifest(
            &manifest,
            &CitationManifestItem {
                citation_key: "missing".to_string(),
                title: "Missing".to_string(),
                url: "https://example.com/missing.pdf".to_string(),
                snippet: String::new(),
                path: path_to_string(&missing_path),
                relative_path: relative_source_path(&document, &missing_path),
                sha256: "missing-hash".to_string(),
                bytes: 42,
                downloaded_at: Some("2026-05-28T10:00:00+03:00".to_string()),
                media_type: Some("application/pdf".to_string()),
                source: Some("SearXNG".to_string()),
                fit_score: Some(90),
                fit_label: Some("strong".to_string()),
                fit_reasons: Some(vec!["downloadable PDF source".to_string()]),
                file_exists: Some(true),
                hash_matches: Some(true),
                current_sha256: Some("ignored-live-status".to_string()),
                current_bytes: Some(999),
            },
        )
        .expect("write missing");

        let library = list_citation_sources(CitationSourceLibraryRequest {
            document_path: path_to_string(&document),
        })
        .expect("library");
        assert_eq!(library.sources.len(), 1);
        assert_eq!(library.sources[0].file_exists, Some(false));
        assert_eq!(library.sources[0].hash_matches, None);

        let manifest_text = fs::read_to_string(&manifest).expect("manifest text");
        assert!(!manifest_text.contains("file_exists"));
        assert!(!manifest_text.contains("hash_matches"));
        assert!(!manifest_text.contains("current_sha256"));
        assert!(!manifest_text.contains("current_bytes"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn citation_source_library_marks_modified_files_against_manifest_hash() {
        let dir =
            std::env::temp_dir().join(format!("neditor-citation-modified-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let document = dir.join("proposal.md");
        fs::write(&document, "# Proposal\n").expect("document");
        let source_dir = associated_source_dir(&document).expect("source dir");
        fs::create_dir_all(&source_dir).expect("source dir create");
        let source_path = source_dir.join("source.html");
        fs::write(&source_path, "trusted original").expect("original source");
        let manifest = source_dir.join("sources.json");
        write_source_manifest(
            &manifest,
            &CitationManifestItem {
                citation_key: "source".to_string(),
                title: "Source".to_string(),
                url: "https://example.com/source.html".to_string(),
                snippet: String::new(),
                path: path_to_string(&source_path),
                relative_path: relative_source_path(&document, &source_path),
                sha256: sha256_hex(b"trusted original"),
                bytes: "trusted original".len(),
                downloaded_at: Some("2026-05-28T10:00:00+03:00".to_string()),
                media_type: Some("text/html".to_string()),
                source: Some("DuckDuckGo".to_string()),
                fit_score: Some(70),
                fit_label: Some("review".to_string()),
                fit_reasons: Some(vec!["reviewable text source".to_string()]),
                file_exists: None,
                hash_matches: None,
                current_sha256: None,
                current_bytes: None,
            },
        )
        .expect("write source");
        fs::write(&source_path, "tampered source").expect("modify source");

        let library = list_citation_sources(CitationSourceLibraryRequest {
            document_path: path_to_string(&document),
        })
        .expect("library");
        assert_eq!(library.sources.len(), 1);
        assert_eq!(library.sources[0].file_exists, Some(true));
        assert_eq!(library.sources[0].hash_matches, Some(false));
        assert_eq!(
            library.sources[0].current_bytes,
            Some("tampered source".len())
        );
        assert_eq!(
            library.sources[0].current_sha256.as_deref(),
            Some(sha256_hex(b"tampered source").as_str())
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn citation_source_manifest_normalizes_fit_metadata() {
        let reasons = normalize_fit_reasons(Some(&[
            "  query term in title  ".to_string(),
            "".to_string(),
            "downloadable PDF source".to_string(),
        ]))
        .expect("reasons");
        assert_eq!(
            reasons,
            vec!["query term in title", "downloadable PDF source"]
        );
        assert_eq!(
            normalize_manifest_text("  SearXNG   Result  ").as_deref(),
            Some("SearXNG Result")
        );
    }
}
