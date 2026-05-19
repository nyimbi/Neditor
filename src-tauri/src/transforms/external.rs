use super::{transform_cache_key, TransformArtifact};
use crate::{diagnostics::diag, escape_html, path_to_string, sha256_hex};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const DEFAULT_TRANSFORM_TIMEOUT_MS: u64 = 5_000;
const MAX_TRANSFORM_TIMEOUT_MS: u64 = 30_000;
const MAX_TRANSFORM_INPUT_BYTES: usize = 1_048_576;
const MAX_TRANSFORM_OUTPUT_BYTES: usize = 2_097_152;
const MAX_EXTERNAL_TRANSFORM_CACHE_ENTRIES: usize = 64;
const MAX_EXTERNAL_TRANSFORM_CACHE_FILE_BYTES: u64 = 4_194_304;
const EXTERNAL_TRANSFORM_RENDERER_VERSION: &str = "external-render-v3";

static EXTERNAL_TRANSFORM_CACHE: OnceLock<Mutex<HashMap<String, TransformArtifact>>> =
    OnceLock::new();

#[derive(Debug, Deserialize)]
pub(crate) struct ExternalTransformRequest {
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) engine_path: Option<String>,
    pub(crate) trusted: bool,
    pub(crate) input_mode: Option<String>,
    pub(crate) timeout_ms: Option<u64>,
    pub(crate) max_input_bytes: Option<usize>,
    pub(crate) max_output_bytes: Option<usize>,
}

#[tauri::command]
pub(crate) fn list_transform_engines() -> Vec<Value> {
    vec![
        transform_engine("calc", "rust-native", true, false),
        transform_engine("csv", "rust-native", true, false),
        transform_engine("tsv", "rust-native", true, false),
        transform_engine("json", "rust-native", true, false),
        transform_engine("yaml", "rust-native", true, false),
        transform_engine("glossary", "rust-native", true, false),
        transform_engine("layout", "rust-native", true, false),
        transform_engine("timeline", "rust-native-svg", true, false),
        transform_engine("roadmap", "rust-native", true, false),
        transform_engine("adr", "rust-native", true, false),
        transform_engine("diff", "rust-native", true, false),
        transform_engine("qr", "rust-native-svg", true, false),
        transform_engine("chart", "rust-native-svg", true, false),
        transform_engine("mermaid", "rust-native-svg", true, false),
        transform_engine("pikchr", "external-sidecar", false, true),
        transform_engine("dot", "external-sidecar", false, true),
        transform_engine("graphviz", "external-sidecar", false, true),
        transform_engine("plantuml", "external-sidecar", false, true),
        transform_engine("d2", "external-sidecar", false, true),
        transform_engine("vega-lite", "rust-native-svg", true, false),
        transform_engine("geojson", "rust-native-svg", true, false),
        transform_engine("topojson", "rust-native-svg", true, false),
        transform_engine("stl", "rust-native-svg", true, false),
        transform_engine("openapi", "rust-native", true, false),
        transform_engine("json-schema", "rust-native", true, false),
        transform_engine("bibtex", "rust-native", true, false),
    ]
}

#[tauri::command]
pub(crate) fn run_external_transform(
    request: ExternalTransformRequest,
) -> Result<TransformArtifact, String> {
    if !external_transform_supported(&request.name) {
        return Err(format!(
            "External execution is not available for transform '{}'.",
            request.name
        ));
    }
    if !request.trusted {
        return Err(format!(
            "{} requires explicit trust before external execution.",
            request.name
        ));
    }

    let engine_path = request
        .engine_path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| format!("Missing engine path for {}.", request.name))?;
    let engine_path = PathBuf::from(engine_path);
    if !engine_path.is_absolute() {
        return Err(
            "Engine path must be absolute; shell lookup is intentionally disabled.".to_string(),
        );
    }
    if !engine_path.is_file() {
        return Err(format!(
            "Engine path does not exist: {}",
            engine_path.display()
        ));
    }

    let input_limit = request
        .max_input_bytes
        .unwrap_or(MAX_TRANSFORM_INPUT_BYTES)
        .min(MAX_TRANSFORM_INPUT_BYTES);
    if request.body.len() > input_limit {
        return Err(format!(
            "{} input is {} bytes, above the {} byte limit.",
            request.name,
            request.body.len(),
            input_limit
        ));
    }
    let output_limit = request
        .max_output_bytes
        .unwrap_or(MAX_TRANSFORM_OUTPUT_BYTES)
        .min(MAX_TRANSFORM_OUTPUT_BYTES);

    let timeout_ms = request
        .timeout_ms
        .unwrap_or(DEFAULT_TRANSFORM_TIMEOUT_MS)
        .clamp(1, MAX_TRANSFORM_TIMEOUT_MS);
    let input_mode = request.input_mode.as_deref().unwrap_or("stdin");
    if !matches!(input_mode, "stdin" | "file") {
        return Err("External transform input_mode must be 'stdin' or 'file'.".to_string());
    }
    let source_hash = sha256_hex(request.body.as_bytes());
    let (engine_identity, engine_version) = external_engine_identity(&engine_path)?;
    let renderer_identity = format!("{engine_identity};{EXTERNAL_TRANSFORM_RENDERER_VERSION}");
    let cache_key =
        transform_cache_key(&request.name, input_mode, &renderer_identity, &source_hash);
    if let Some(artifact) = cached_external_transform(&cache_key, &request.name, output_limit) {
        return Ok(artifact);
    }

    let artifact = execute_external_transform(ExternalTransformExecution {
        name: &request.name,
        body: &request.body,
        engine_path: &engine_path,
        input_mode,
        timeout_ms,
        max_output_bytes: output_limit,
        engine_identity: &renderer_identity,
        engine_version: &engine_version,
    })?;
    store_external_transform(artifact.clone());
    Ok(artifact)
}

fn external_transform_supported(name: &str) -> bool {
    matches!(name, "pikchr" | "dot" | "graphviz" | "plantuml" | "d2")
}

fn external_transform_cache() -> &'static Mutex<HashMap<String, TransformArtifact>> {
    EXTERNAL_TRANSFORM_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cached_external_transform(
    cache_key: &str,
    name: &str,
    max_output_bytes: usize,
) -> Option<TransformArtifact> {
    cached_memory_external_transform(cache_key, name, max_output_bytes)
        .or_else(|| cached_disk_external_transform(cache_key, name, max_output_bytes))
}

fn cached_memory_external_transform(
    cache_key: &str,
    name: &str,
    max_output_bytes: usize,
) -> Option<TransformArtifact> {
    let cache = external_transform_cache().lock().ok()?;
    let mut artifact = cache.get(cache_key)?.clone();
    prepare_cached_external_transform(&mut artifact, name, max_output_bytes, false)?;
    Some(artifact)
}

fn cached_disk_external_transform(
    cache_key: &str,
    name: &str,
    max_output_bytes: usize,
) -> Option<TransformArtifact> {
    let path = external_transform_disk_cache_path(cache_key);
    let metadata = fs::metadata(&path).ok()?;
    if metadata.len() > MAX_EXTERNAL_TRANSFORM_CACHE_FILE_BYTES {
        let _ = fs::remove_file(path);
        return None;
    }
    let text = fs::read_to_string(&path).ok()?;
    let mut artifact: TransformArtifact = serde_json::from_str(&text).ok()?;
    prepare_cached_external_transform(&mut artifact, name, max_output_bytes, true)?;
    store_external_transform_memory(artifact.clone());
    Some(artifact)
}

fn prepare_cached_external_transform(
    artifact: &mut TransformArtifact,
    name: &str,
    max_output_bytes: usize,
    persistent: bool,
) -> Option<()> {
    if artifact.html.len() > max_output_bytes {
        return None;
    }
    artifact.duration_ms = Some(0);
    let cache_scope = if persistent {
        "persistent cache"
    } else {
        "memory cache"
    };
    artifact.diagnostics.push(diag(
        "info",
        format!("{name} external transform served from cache ({cache_scope})."),
        None,
        None,
        Some("Cache key includes transform name, engine path, input mode, and source hash."),
    ));
    Some(())
}

fn store_external_transform(artifact: TransformArtifact) {
    store_external_transform_memory(artifact.clone());
    store_external_transform_disk(&artifact);
}

fn store_external_transform_memory(artifact: TransformArtifact) {
    let Ok(mut cache) = external_transform_cache().lock() else {
        return;
    };
    if cache.len() >= MAX_EXTERNAL_TRANSFORM_CACHE_ENTRIES {
        if let Some(key) = cache.keys().next().cloned() {
            cache.remove(&key);
        }
    }
    cache.insert(artifact.cache_key.clone(), artifact);
}

fn store_external_transform_disk(artifact: &TransformArtifact) {
    let root = external_transform_disk_cache_root();
    if fs::create_dir_all(&root).is_err() {
        return;
    }
    let path = root.join(format!("{}.json", artifact.cache_key));
    let temp_path = root.join(format!("{}.tmp", artifact.cache_key));
    let Ok(text) = serde_json::to_string(artifact) else {
        return;
    };
    if fs::write(&temp_path, text.as_bytes()).is_ok() {
        let _ = fs::rename(&temp_path, &path);
    }
    prune_external_transform_disk_cache(&root);
}

fn external_transform_disk_cache_root() -> PathBuf {
    std::env::temp_dir().join("neditor-transform-cache-v1")
}

fn external_transform_disk_cache_path(cache_key: &str) -> PathBuf {
    external_transform_disk_cache_root().join(format!("{cache_key}.json"))
}

fn prune_external_transform_disk_cache(root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut files = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                return None;
            }
            let modified = entry
                .metadata()
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            Some((path, modified))
        })
        .collect::<Vec<_>>();
    if files.len() <= MAX_EXTERNAL_TRANSFORM_CACHE_ENTRIES {
        return;
    }
    files.sort_by_key(|(_, modified)| *modified);
    let remove_count = files.len() - MAX_EXTERNAL_TRANSFORM_CACHE_ENTRIES;
    for (path, _) in files.into_iter().take(remove_count) {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
pub(crate) fn clear_external_transform_memory_cache_for_tests() {
    if let Ok(mut cache) = external_transform_cache().lock() {
        cache.clear();
    }
}

fn external_engine_identity(engine_path: &Path) -> Result<(String, String), String> {
    let metadata = fs::metadata(engine_path).map_err(|err| err.to_string())?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let version = format!("file-size:{};mtime:{modified}", metadata.len());
    Ok((
        format!("{};{version}", path_to_string(engine_path)),
        version,
    ))
}

struct ExternalTransformExecution<'a> {
    name: &'a str,
    body: &'a str,
    engine_path: &'a Path,
    input_mode: &'a str,
    timeout_ms: u64,
    max_output_bytes: usize,
    engine_identity: &'a str,
    engine_version: &'a str,
}

fn execute_external_transform(
    request: ExternalTransformExecution<'_>,
) -> Result<TransformArtifact, String> {
    let name = request.name;
    let input_mode = request.input_mode;
    let max_output_bytes = request.max_output_bytes;
    let source_hash = sha256_hex(request.body.as_bytes());
    let started = Instant::now();
    let mut diagnostics = Vec::new();
    let mut temp_input = None;
    let mut command = Command::new(request.engine_path);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    if input_mode == "file" {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "neditor-{name}-{source_hash}-{}-{unique}.input",
            std::process::id()
        ));
        fs::write(&path, request.body.as_bytes()).map_err(|err| err.to_string())?;
        command.arg(&path);
        temp_input = Some(path);
    } else {
        command.stdin(Stdio::piped());
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            if let Some(path) = temp_input {
                let _ = fs::remove_file(path);
            }
            return Err(error.to_string());
        }
    };
    let stdin_writer = if input_mode == "stdin" {
        child.stdin.take().map(|mut stdin| {
            let input = request.body.as_bytes().to_vec();
            std::thread::spawn(move || stdin.write_all(&input).map_err(|err| err.to_string()))
        })
    } else {
        None
    };

    let status = loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            break status;
        }
        if started.elapsed() >= Duration::from_millis(request.timeout_ms) {
            let _ = child.kill();
            let _ = child.wait();
            if let Some(path) = temp_input {
                let _ = fs::remove_file(path);
            }
            return Err(format!(
                "{} external transform timed out after {}ms.",
                name, request.timeout_ms
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    if let Some(writer) = stdin_writer {
        match writer.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) if status.success() => {
                return Err(format!(
                    "{name} external transform stdin write failed: {error}"
                ));
            }
            Ok(Err(_)) => {}
            Err(_) if status.success() => {
                return Err(format!("{name} external transform stdin writer panicked."));
            }
            Err(_) => {}
        }
    }

    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    if let Some(path) = temp_input {
        let _ = fs::remove_file(path);
    }
    if output.stdout.len() > max_output_bytes {
        return Err(format!(
            "{name} external transform output is {} bytes, above the {} byte limit.",
            output.stdout.len(),
            max_output_bytes
        ));
    }
    if output.stderr.len() > max_output_bytes {
        return Err(format!(
            "{name} external transform diagnostics are {} bytes, above the {} byte limit.",
            output.stderr.len(),
            max_output_bytes
        ));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        diagnostics.push(diag(
            if status.success() { "info" } else { "error" },
            format!("{name} stderr: {stderr}"),
            None,
            None,
            Some("Review external engine diagnostics."),
        ));
    }

    if !status.success() {
        let status_label = status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_string());
        let stderr_detail = if stderr.is_empty() {
            String::new()
        } else {
            format!(": {stderr}")
        };
        return Err(format!(
            "{name} external transform exited with status {status_label}{stderr_detail}."
        ));
    }

    let output_text = String::from_utf8_lossy(&output.stdout).to_string();
    let rendered_output = if output_text.trim_start().starts_with('<') {
        output_text
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("")
    } else {
        format!("<pre>{}</pre>", escape_external_pre_text(&output_text))
    };
    let html = format!(
        "<section class=\"transform transform-{} transform-external\" data-transform=\"{}\">{rendered_output}</section>",
        escape_html(name),
        escape_html(name)
    );
    let duration_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    diagnostics.push(diag(
        "info",
        format!("{name} external transform completed in {duration_ms}ms."),
        None,
        None,
        Some("Output was captured without invoking a shell."),
    ));

    let output_hash = sha256_hex(html.as_bytes());
    Ok(TransformArtifact {
        id: format!("{name}-{source_hash}"),
        name: name.to_string(),
        output_kind: if html.contains("<svg") { "svg" } else { "html" }.to_string(),
        output_hash,
        cache_key: transform_cache_key(name, input_mode, request.engine_identity, &source_hash),
        execution_kind: "external".to_string(),
        engine_version: Some(request.engine_version.to_string()),
        engine_path: Some(path_to_string(request.engine_path)),
        input_mode: input_mode.to_string(),
        duration_ms: Some(duration_ms),
        source_hash,
        source: request.body.to_string(),
        options: json!({}),
        html,
        diagnostics,
    })
}

fn escape_external_pre_text(text: &str) -> String {
    escape_html(text)
        .replace("\r\n", "&#10;")
        .replace('\n', "&#10;")
}

fn transform_engine(
    name: &str,
    execution: &str,
    safe_by_default: bool,
    requires_execution: bool,
) -> Value {
    let input_modes = if requires_execution {
        vec!["stdin", "file"]
    } else {
        vec!["embedded"]
    };
    let installation_label = if requires_execution {
        "User-installed optional engine; not bundled with NEditor."
    } else {
        "Bundled Rust-native engine."
    };
    json!({
        "name": name,
        "execution": execution,
        "safeByDefault": safe_by_default,
        "bundled": !requires_execution,
        "installationLabel": installation_label,
        "requiresNetwork": false,
        "requiresExecution": requires_execution,
        "trustRequired": requires_execution,
        "preferenceKey": format!("transforms.{name}.path"),
        "defaultCommand": name,
        "inputModes": input_modes,
        "limits": {
            "timeoutMs": DEFAULT_TRANSFORM_TIMEOUT_MS,
            "maxTimeoutMs": MAX_TRANSFORM_TIMEOUT_MS,
            "maxInputBytes": MAX_TRANSFORM_INPUT_BYTES,
            "maxOutputBytes": MAX_TRANSFORM_OUTPUT_BYTES
        },
        "cacheScope": "name+enginePath+inputMode+sourceHash",
        "exportTargets": ["html", "pdf", "docx", "pptx"]
    })
}
