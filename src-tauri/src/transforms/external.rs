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
const EXTERNAL_TRANSFORM_RENDERER_VERSION: &str = "external-render-v4";

static EXTERNAL_TRANSFORM_CACHE: OnceLock<Mutex<HashMap<String, TransformArtifact>>> =
    OnceLock::new();

#[derive(Debug, Deserialize)]
pub(crate) struct ExternalTransformRequest {
    pub(crate) name: String,
    pub(crate) body: String,
    pub(crate) engine_path: Option<String>,
    pub(crate) trusted: bool,
    pub(crate) input_mode: Option<String>,
    pub(crate) output_format: Option<String>,
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
        for_graphviz_engine("dot"),
        for_graphviz_engine("graphviz"),
        for_graphviz_engine("circo"),
        for_graphviz_engine("neato"),
        for_graphviz_engine("fdp"),
        for_graphviz_engine("osage"),
        for_graphviz_engine("twopi"),
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
    validate_external_engine_executable(&engine_path)?;

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
    let output_format =
        normalize_external_output_format(&request.name, request.output_format.as_deref())?;
    let source_hash = sha256_hex(request.body.as_bytes());
    let (engine_identity, engine_version) = external_engine_identity(&engine_path)?;
    let adapter =
        external_engine_adapter(&request.name, input_mode, output_format, None, &engine_path)?;
    let renderer_identity = adapter.cache_identity(&format!(
        "{engine_identity};{EXTERNAL_TRANSFORM_RENDERER_VERSION}"
    ));
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
        output_format,
        timeout_ms,
        max_output_bytes: output_limit,
        renderer_identity: &renderer_identity,
        engine_version: &engine_version,
    })?;
    store_external_transform(artifact.clone());
    Ok(artifact)
}

fn external_transform_supported(name: &str) -> bool {
    graphviz_command(name).is_some() || matches!(name, "pikchr" | "plantuml" | "d2")
}

#[derive(Debug)]
struct ExternalEngineAdapter {
    engine: &'static str,
    input_mode: &'static str,
    args: Vec<String>,
    stdin: bool,
    sidecar_output_suffix: Option<&'static str>,
}

impl ExternalEngineAdapter {
    fn stdout(engine: &'static str, input_mode: &'static str, args: Vec<String>) -> Self {
        Self {
            engine,
            input_mode,
            args,
            stdin: input_mode == "stdin",
            sidecar_output_suffix: None,
        }
    }

    fn sidecar(
        engine: &'static str,
        input_mode: &'static str,
        args: Vec<String>,
        output_suffix: &'static str,
    ) -> Self {
        Self {
            engine,
            input_mode,
            args,
            stdin: false,
            sidecar_output_suffix: Some(output_suffix),
        }
    }

    fn cache_identity(&self, engine_identity: &str) -> String {
        format!(
            "{engine_identity};adapter={};mode={};args={}",
            self.engine,
            self.input_mode,
            self.args.join("\u{1f}")
        )
    }
}

fn external_engine_adapter(
    name: &str,
    input_mode: &str,
    output_format: &'static str,
    temp_path: Option<&Path>,
    engine_path: &Path,
) -> Result<ExternalEngineAdapter, String> {
    let temp = temp_path
        .map(path_to_string)
        .unwrap_or_else(|| "-".to_string());
    match (name, input_mode) {
        (name, "stdin") if graphviz_command(name).is_some() => Ok(ExternalEngineAdapter::stdout(
            "graphviz",
            "stdin",
            vec!["-Tsvg".to_string()],
        )),
        (name, "file") if graphviz_command(name).is_some() => Ok(ExternalEngineAdapter::stdout(
            "graphviz",
            "file",
            vec!["-Tsvg".to_string(), temp],
        )),
        ("d2", "stdin") => Ok(ExternalEngineAdapter::stdout(
            "d2",
            "stdin",
            vec!["-".to_string(), "-".to_string()],
        )),
        ("d2", "file") => Ok(ExternalEngineAdapter::stdout(
            "d2",
            "file",
            vec![temp, "-".to_string()],
        )),
        ("plantuml", "stdin") => Ok(ExternalEngineAdapter::stdout(
            "plantuml",
            "stdin",
            vec![format!("-t{output_format}"), "-pipe".to_string()],
        )),
        ("plantuml", "file") => Ok(ExternalEngineAdapter::sidecar(
            "plantuml",
            "file",
            vec![format!("-t{output_format}"), temp],
            output_format,
        )),
        ("pikchr", "stdin") if pikchr_uses_source_file_argument(engine_path) => {
            Ok(ExternalEngineAdapter::stdout("pikchr", "stdin", vec![temp]))
        }
        ("pikchr", "file") if pikchr_uses_source_file_argument(engine_path) => {
            Ok(ExternalEngineAdapter::stdout("pikchr", "file", vec![temp]))
        }
        ("pikchr", "stdin") => Ok(ExternalEngineAdapter::stdout(
            "pikchr",
            "stdin",
            vec!["-".to_string()],
        )),
        ("pikchr", "file") => Ok(ExternalEngineAdapter::stdout("pikchr", "file", vec![temp])),
        (_, "stdin" | "file") => Err(format!("No external adapter is registered for {name}.")),
        _ => Err("External transform input_mode must be 'stdin' or 'file'.".to_string()),
    }
}

fn pikchr_uses_source_file_argument(engine_path: &Path) -> bool {
    engine_path
        .file_stem()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            let name = name.to_ascii_lowercase();
            name == "pikchr-cli" || name.starts_with("pikchr-cli-")
        })
}

fn normalize_external_output_format(
    name: &str,
    output_format: Option<&str>,
) -> Result<&'static str, String> {
    let requested = output_format
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("svg")
        .to_ascii_lowercase();
    match (name, requested.as_str()) {
        ("plantuml", "svg" | "png") => Ok(if requested == "png" { "png" } else { "svg" }),
        ("plantuml", _) => {
            Err("PlantUML external output_format must be 'svg' or 'png'.".to_string())
        }
        (_, "svg") => Ok("svg"),
        (_, _) => Err(format!(
            "{name} external transform only supports SVG output in the current adapter."
        )),
    }
}

fn external_transform_requires_temp_input(
    name: &str,
    input_mode: &str,
    engine_path: &Path,
) -> bool {
    input_mode == "file" || (name == "pikchr" && pikchr_uses_source_file_argument(engine_path))
}

fn external_engine_temp_suffix(name: &str) -> &'static str {
    match name {
        name if graphviz_command(name).is_some() => "dot",
        "d2" => "d2",
        "plantuml" => "puml",
        "pikchr" => "pikchr",
        _ => "input",
    }
}

#[cfg(unix)]
fn validate_external_engine_executable(engine_path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(engine_path).map_err(|err| err.to_string())?;
    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(format!(
            "Engine path is not executable: {}",
            engine_path.display()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_external_engine_executable(_engine_path: &Path) -> Result<(), String> {
    Ok(())
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
    let mut diagnostic = diag(
        "info",
        format!("{name} external transform served from cache ({cache_scope})."),
        None,
        None,
        Some(
            "Cache key includes transform name, engine path, engine file identity, adapter args, input mode, and source hash.",
        ),
    );
    diagnostic
        .related
        .push(format!("cache_key: {}", artifact.cache_key));
    diagnostic
        .related
        .push(format!("output_hash: {}", artifact.output_hash));
    diagnostic
        .related
        .push(format!("cached_output_bytes: {}", artifact.html.len()));
    diagnostic
        .related
        .push(format!("input_mode: {}", artifact.input_mode));
    if let Some(engine_path) = &artifact.engine_path {
        diagnostic
            .related
            .push(format!("engine_path: {engine_path}"));
    }
    if let Some(engine_version) = &artifact.engine_version {
        diagnostic
            .related
            .push(format!("engine_version: {engine_version}"));
    }
    artifact.diagnostics.push(diagnostic);
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

#[cfg(all(test, unix))]
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
    output_format: &'static str,
    timeout_ms: u64,
    max_output_bytes: usize,
    renderer_identity: &'a str,
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
    let mut temp_output = None;

    if external_transform_requires_temp_input(name, input_mode, request.engine_path) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let suffix = external_engine_temp_suffix(name);
        let path = std::env::temp_dir().join(format!(
            "neditor-{name}-{source_hash}-{}-{unique}.{suffix}",
            std::process::id()
        ));
        fs::write(&path, request.body.as_bytes()).map_err(|err| err.to_string())?;
        temp_input = Some(path);
    }

    let adapter = external_engine_adapter(
        name,
        input_mode,
        request.output_format,
        temp_input.as_deref(),
        request.engine_path,
    )?;
    if let (Some(input_path), Some(output_suffix)) =
        (temp_input.as_deref(), adapter.sidecar_output_suffix)
    {
        temp_output = Some(input_path.with_extension(output_suffix));
    }

    let mut command = Command::new(request.engine_path);
    command.args(&adapter.args);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    if adapter.stdin {
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
    let stdin_writer = if adapter.stdin {
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
            if let Some(path) = temp_output {
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
    let output_channel = if adapter.sidecar_output_suffix.is_some() {
        format!(
            "sidecar {}",
            adapter.sidecar_output_suffix.unwrap_or_default()
        )
    } else {
        "stdout".to_string()
    };
    let sidecar_output = if let Some(path) = temp_output.as_deref() {
        Some(fs::read(path).map_err(|err| {
            format!(
                "{name} external transform did not produce expected sidecar output {}: {err}",
                path.display()
            )
        })?)
    } else {
        None
    };
    if let Some(path) = temp_input {
        let _ = fs::remove_file(path);
    }
    if let Some(path) = temp_output {
        let _ = fs::remove_file(path);
    }
    let stdout = sidecar_output.as_deref().unwrap_or(&output.stdout);
    if stdout.len() > max_output_bytes {
        return Err(format!(
            "{name} external transform output is {} bytes, above the {} byte limit.",
            stdout.len(),
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
            Some(transform_failure_suggestion(name)),
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
        let hint = transform_failure_suggestion(name);
        return Err(format!(
            "{name} external transform exited with status {status_label}{stderr_detail}. Hint: {hint}"
        ));
    }

    let (rendered_output, output_kind) = if request.output_format == "png" {
        (
            format!(
                "<img class=\"transform-image transform-{}-png\" alt=\"{} diagram\" src=\"data:image/png;base64,{}\"/>",
                escape_html(name),
                escape_html(name),
                encode_base64(stdout)
            ),
            "png",
        )
    } else {
        let output_text = String::from_utf8_lossy(stdout).to_string();
        let rendered = if output_text.trim_start().starts_with('<') {
            output_text
                .lines()
                .map(str::trim)
                .collect::<Vec<_>>()
                .join("")
        } else {
            format!("<pre>{}</pre>", escape_external_pre_text(&output_text))
        };
        let output_kind = if rendered.contains("<svg") {
            "svg"
        } else {
            "html"
        };
        (rendered, output_kind)
    };
    let html = format!(
        "<section class=\"transform transform-{} transform-external\" data-transform=\"{}\" data-output-kind=\"{output_kind}\">{rendered_output}</section>",
        escape_html(name),
        escape_html(name)
    );
    let duration_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let output_hash = sha256_hex(html.as_bytes());
    let cache_key = transform_cache_key(name, input_mode, request.renderer_identity, &source_hash);
    let mut diagnostic = diag(
        "info",
        format!("{name} external transform completed in {duration_ms}ms."),
        None,
        None,
        Some("Output was captured without invoking a shell."),
    );
    diagnostic.related.push(format!(
        "engine_path: {}",
        path_to_string(request.engine_path)
    ));
    diagnostic
        .related
        .push(format!("adapter: {}", adapter.engine));
    diagnostic
        .related
        .push(format!("adapter_args: {}", adapter.args.join(" ")));
    diagnostic
        .related
        .push(format!("engine_version: {}", request.engine_version));
    diagnostic.related.push(format!("input_mode: {input_mode}"));
    diagnostic
        .related
        .push(format!("input_bytes: {}", request.body.len()));
    diagnostic
        .related
        .push(format!("output_bytes: {}", stdout.len()));
    diagnostic
        .related
        .push(format!("stderr_bytes: {}", output.stderr.len()));
    diagnostic
        .related
        .push(format!("timeout_ms: {}", request.timeout_ms));
    diagnostic
        .related
        .push(format!("output_channel: {output_channel}"));
    diagnostic
        .related
        .push(format!("output_format: {}", request.output_format));
    diagnostic.related.push(format!(
        "status: {}",
        status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_string())
    ));
    diagnostic.related.push(format!("cache_key: {cache_key}"));
    diagnostic
        .related
        .push(format!("output_hash: {output_hash}"));
    diagnostics.push(diagnostic);

    Ok(TransformArtifact {
        id: format!("{name}-{source_hash}"),
        name: name.to_string(),
        output_kind: output_kind.to_string(),
        output_hash,
        cache_key,
        execution_kind: "external".to_string(),
        engine_version: Some(request.engine_version.to_string()),
        engine_path: Some(path_to_string(request.engine_path)),
        input_mode: input_mode.to_string(),
        duration_ms: Some(duration_ms),
        source_hash,
        source: request.body.to_string(),
        source_file: None,
        source_line: None,
        end_source_line: None,
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

fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(ALPHABET[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(ALPHABET[(third & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
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
        "aliases": transform_aliases(name),
        "output": transform_output(name),
        "execution": execution,
        "safeByDefault": safe_by_default,
        "bundled": !requires_execution,
        "installationLabel": installation_label,
        "setupHint": transform_setup_hint(name, requires_execution),
        "securitySummary": transform_security_summary(requires_execution),
        "requiresNetwork": false,
        "requiresExecution": requires_execution,
        "trustRequired": requires_execution,
        "preferenceKey": format!("transforms.{name}.path"),
        "defaultCommand": transform_default_command(name),
        "adapterProfile": transform_adapter_profile(name),
        "diagnosticProfile": transform_diagnostic_profile(name, requires_execution),
        "inputModes": input_modes,
        "limits": {
            "timeoutMs": DEFAULT_TRANSFORM_TIMEOUT_MS,
            "maxTimeoutMs": MAX_TRANSFORM_TIMEOUT_MS,
            "maxInputBytes": MAX_TRANSFORM_INPUT_BYTES,
            "maxOutputBytes": MAX_TRANSFORM_OUTPUT_BYTES
        },
        "cacheScope": "name+enginePath+engineFileIdentity+adapterArgs+inputMode+sourceHash",
        "exportTargets": ["html", "pdf", "docx", "pptx"]
    })
}

fn for_graphviz_engine(name: &str) -> Value {
    transform_engine(name, "external-sidecar", false, true)
}

fn transform_aliases(name: &str) -> Vec<&'static str> {
    match name {
        "dot" => vec!["graph"],
        "graphviz" => vec!["dot"],
        "vega-lite" => vec!["vegalite"],
        "json-schema" => vec!["jsonschema", "schema"],
        "yaml" => vec!["yml"],
        _ => Vec::new(),
    }
}

fn transform_output(name: &str) -> &'static str {
    match name {
        "calc" => "variables",
        "csv" | "tsv" => "table",
        "json" | "yaml" | "glossary" | "layout" | "roadmap" | "adr" | "diff" | "openapi"
        | "json-schema" | "bibtex" => "html",
        "plantuml" => "svg-or-png",
        "timeline" | "qr" | "chart" | "mermaid" | "pikchr" | "dot" | "graphviz" | "circo"
        | "neato" | "fdp" | "osage" | "twopi" | "d2" | "vega-lite" | "geojson" | "topojson"
        | "stl" => "svg",
        _ => "html",
    }
}

fn transform_diagnostic_profile(name: &str, requires_execution: bool) -> Value {
    if !requires_execution {
        return json!({
            "versionProbe": null,
            "successRelated": ["renderer", "output_hash"],
            "failureRelated": ["diagnostic", "source_range"],
            "cacheKeyIncludes": ["transform", "renderer", "source_hash"]
        });
    }
    json!({
        "versionProbe": transform_version_probe(name),
        "failureHint": transform_failure_suggestion(name),
        "stderrHint": transform_stderr_suggestion(name),
        "successRelated": [
            "engine_path",
            "engine_version",
            "adapter",
            "adapter_args",
            "input_mode",
            "input_bytes",
            "output_bytes",
            "stderr_bytes",
            "timeout_ms",
            "output_channel",
            "status",
            "cache_key",
            "output_hash"
        ],
        "failureRelated": [
            "engine_path",
            "adapter",
            "input_mode",
            "timeout_ms",
            "exit_status",
            "stderr",
            "output_limit"
        ],
        "cacheKeyIncludes": ["transform", "engine_path", "engine_file_size", "engine_mtime", "adapter", "adapter_args", "input_mode", "source_hash"]
    })
}

fn transform_version_probe(name: &str) -> Option<&'static str> {
    match name {
        "dot" | "graphviz" => Some("dot -V"),
        "circo" => Some("circo -V"),
        "neato" => Some("neato -V"),
        "fdp" => Some("fdp -V"),
        "osage" => Some("osage -V"),
        "twopi" => Some("twopi -V"),
        "d2" => Some("d2 --version"),
        "plantuml" => Some("plantuml -version"),
        "pikchr" => Some("pikchr --version"),
        _ => None,
    }
}

fn transform_setup_hint(name: &str, requires_execution: bool) -> &'static str {
    if !requires_execution {
        return "No setup required; this renderer is built into NEditor.";
    }
    match name {
        "pikchr" => "Choose a local Pikchr executable. NEditor does not bundle Pikchr by default.",
        "dot" | "graphviz" => "Choose a local Graphviz executable such as dot. NEditor invokes it directly, not through a shell.",
        "circo" => "Choose a local Graphviz circo executable. NEditor invokes it directly, not through a shell.",
        "neato" => "Choose a local Graphviz neato executable. NEditor invokes it directly, not through a shell.",
        "fdp" => "Choose a local Graphviz fdp executable. NEditor invokes it directly, not through a shell.",
        "osage" => "Choose a local Graphviz osage executable. NEditor invokes it directly, not through a shell.",
        "twopi" => "Choose a local Graphviz twopi executable. NEditor invokes it directly, not through a shell.",
        "plantuml" => {
            "Choose a local PlantUML launcher or wrapper script. Java and PlantUML remain user-installed."
        }
        "d2" => "Choose a local D2 executable. Bundling is intentionally deferred to license/package review.",
        "stl" => "Choose a local STL renderer only if static SVG fallback is insufficient.",
        _ => "Choose an absolute path to a local executable for this optional transform engine.",
    }
}

fn transform_default_command(name: &str) -> String {
    match name {
        "dot" | "graphviz" => "dot".to_string(),
        "circo" | "neato" | "fdp" | "osage" | "twopi" => name.to_string(),
        "plantuml" => "plantuml".to_string(),
        "d2" => "d2".to_string(),
        "pikchr" => "pikchr".to_string(),
        _ => name.to_string(),
    }
}

fn transform_adapter_profile(name: &str) -> &'static str {
    match name {
        "dot" | "graphviz" => "Graphviz DOT adapter: invokes -Tsvg and captures SVG from stdout.",
        "circo" => "Graphviz circo adapter: invokes -Tsvg and captures SVG from stdout.",
        "neato" => "Graphviz neato adapter: invokes -Tsvg and captures SVG from stdout.",
        "fdp" => "Graphviz fdp adapter: invokes -Tsvg and captures SVG from stdout.",
        "osage" => "Graphviz osage adapter: invokes -Tsvg and captures SVG from stdout.",
        "twopi" => "Graphviz twopi adapter: invokes -Tsvg and captures SVG from stdout.",
        "d2" => "D2 adapter: invokes input-to-stdout mode with '-' as the output target.",
        "plantuml" => "PlantUML adapter: uses -tsvg/-tpng with -pipe for stdin, or reads PlantUML's SVG/PNG sidecar for file mode.",
        "pikchr" => "Pikchr adapter: passes stdin with '-', a temporary Pikchr source file, or a temporary source file path for pikchr-cli.",
        _ => "No external adapter; rendered by the embedded Rust engine.",
    }
}

fn transform_failure_suggestion(name: &str) -> &'static str {
    match name {
        "dot" | "graphviz" => "Check DOT syntax and verify the selected executable is Graphviz dot; NEditor invokes -Tsvg without a shell.",
        "circo" => "Check DOT syntax and verify the selected executable is Graphviz circo; NEditor invokes -Tsvg without a shell.",
        "neato" => "Check DOT syntax and verify the selected executable is Graphviz neato; NEditor invokes -Tsvg without a shell.",
        "fdp" => "Check DOT syntax and verify the selected executable is Graphviz fdp; NEditor invokes -Tsvg without a shell.",
        "osage" => "Check DOT syntax and verify the selected executable is Graphviz osage; NEditor invokes -Tsvg without a shell.",
        "twopi" => "Check DOT syntax and verify the selected executable is Graphviz twopi; NEditor invokes -Tsvg without a shell.",
        "d2" => {
            "Check D2 syntax and verify the selected executable supports '-' input with '-' output."
        }
        "plantuml" => {
            "Check PlantUML syntax, Java availability, and -tsvg/-pipe support for the selected launcher."
        }
        "pikchr" => {
            "Check Pikchr syntax; NEditor passes stdin, a temporary .pikchr file, or a temporary source file path for pikchr-cli."
        }
        _ => "Review the transform source and renderer diagnostics.",
    }
}

fn transform_stderr_suggestion(name: &str) -> &'static str {
    match name {
        name if graphviz_command(name).is_some() => "Graphviz stderr is captured verbatim; DOT parse errors usually include the source line.",
        "d2" => "D2 stderr is captured verbatim; syntax and layout errors usually identify the failed shape or edge.",
        "plantuml" => "PlantUML stderr is captured verbatim; launcher, Java, and syntax errors are reported by the selected wrapper.",
        "pikchr" => "Pikchr stderr is captured verbatim; parser errors usually reference the malformed statement.",
        _ => "Renderer stderr is captured verbatim when available.",
    }
}

pub(crate) fn graphviz_command(name: &str) -> Option<&'static str> {
    match name {
        "dot" | "graphviz" => Some("dot"),
        "circo" => Some("circo"),
        "neato" => Some("neato"),
        "fdp" => Some("fdp"),
        "osage" => Some("osage"),
        "twopi" => Some("twopi"),
        _ => None,
    }
}

fn transform_security_summary(requires_execution: bool) -> &'static str {
    if requires_execution {
        "Disabled until trusted. Runs with an absolute path, no shell interpolation, timeout limits, input size limits, output size limits, and cache-keyed diagnostics."
    } else {
        "Rust-native transform. Does not execute external programs or perform network access."
    }
}
