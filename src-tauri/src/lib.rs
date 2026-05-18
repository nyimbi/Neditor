use chrono::Utc;
#[cfg(feature = "native-watch")]
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Mutex,
    time::{Duration, Instant},
};
#[cfg(feature = "native-watch")]
use tauri::Emitter;
use tauri::{Manager, State};

mod document_ast;
mod export;

#[cfg(test)]
use document_ast::DocumentBlock;
use document_ast::{attach_source_ranges, build_document_ast, AstSourceRange, DocumentAst};
use export::{
    render_docx_bytes, render_full_html, render_markdown_bundle_bytes, render_pdf_bytes,
    render_pptx_bytes,
};

const MAX_INCLUDE_DEPTH: usize = 16;
const MAX_WORKSPACE_SCAN_DEPTH: usize = 12;
const MAX_WORKSPACE_SCAN_ITEMS: usize = 2000;
const DEFAULT_TRANSFORM_TIMEOUT_MS: u64 = 5_000;
const MAX_TRANSFORM_TIMEOUT_MS: u64 = 30_000;
const MAX_TRANSFORM_INPUT_BYTES: usize = 1_048_576;

#[derive(Default)]
struct FileWatcherState {
    watcher: Mutex<Option<ActiveFileWatcher>>,
}

struct ActiveFileWatcher {
    #[cfg(feature = "native-watch")]
    _watcher: RecommendedWatcher,
    #[allow(dead_code)]
    signature: String,
}

#[derive(Debug, Deserialize)]
struct CompileRequest {
    text: String,
    file_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct CompileResponse {
    compiled_markdown: String,
    html: String,
    semantic: SemanticDocument,
    document_ast: DocumentAst,
    diagnostics: Vec<DocumentDiagnostic>,
    include_graph: Vec<IncludeEdge>,
    source_map: Vec<SourceMapEntry>,
    metadata: Value,
    bibliography: Vec<BibliographyEntry>,
    index_terms: Vec<String>,
    formula_graph: Vec<FormulaValue>,
    transform_artifacts: Vec<TransformArtifact>,
    export_manifest: ExportManifest,
}

#[derive(Debug, Serialize)]
struct SemanticDocument {
    title: String,
    status: String,
    headings: Vec<Heading>,
    outline: Vec<Heading>,
    tables: usize,
    table_summaries: Vec<TableSummary>,
    figures: usize,
    equations: usize,
    citations: Vec<String>,
    citation_references: Vec<CitationReference>,
    duplicate_bibliography_keys: Vec<String>,
    glossary: BTreeMap<String, String>,
    layout_directives: Vec<String>,
    comments: Vec<ReviewComment>,
    ai_sources: Vec<AiSource>,
    labels: Vec<String>,
    cross_references: Vec<CrossReference>,
}

#[derive(Clone, Debug, Serialize)]
struct Heading {
    level: usize,
    text: String,
    anchor: String,
    line: usize,
}

#[derive(Debug, Serialize, Clone)]
struct DocumentDiagnostic {
    severity: String,
    message: String,
    source_file: Option<String>,
    line: Option<usize>,
    suggestion: Option<String>,
    related: Vec<String>,
}

#[derive(Debug, Serialize)]
struct IncludeEdge {
    parent: String,
    child: String,
    depth: usize,
}

#[derive(Debug, Serialize)]
struct SourceMapEntry {
    generated_line: usize,
    source_file: String,
    source_line: usize,
}

#[derive(Debug, Serialize)]
struct BibliographyEntry {
    key: String,
    title: String,
    raw: String,
}

#[derive(Debug, Serialize)]
struct CitationReference {
    key: String,
    locator: Option<String>,
    raw: String,
}

#[derive(Debug, Serialize)]
struct FormulaValue {
    name: String,
    expression: String,
    value: Option<f64>,
    error: Option<String>,
    dependencies: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TransformArtifact {
    id: String,
    name: String,
    output_kind: String,
    source_hash: String,
    cache_key: String,
    execution_kind: String,
    engine_path: Option<String>,
    input_mode: String,
    duration_ms: Option<u64>,
    html: String,
    diagnostics: Vec<DocumentDiagnostic>,
}

#[derive(Debug, Deserialize)]
struct ExternalTransformRequest {
    name: String,
    body: String,
    engine_path: Option<String>,
    trusted: bool,
    input_mode: Option<String>,
    timeout_ms: Option<u64>,
    max_input_bytes: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
struct ExportManifest {
    document_title: String,
    document_version: String,
    status: String,
    exported_at: String,
    source_hash: String,
    included_files: Vec<ManifestFile>,
    export_target: String,
    export_options: Value,
    transform_artifacts: Vec<Value>,
    app_version: String,
}

#[derive(Clone, Debug, Serialize)]
struct ManifestFile {
    path: String,
    hash: String,
}

#[derive(Debug, Serialize)]
struct ReviewComment {
    line: usize,
    author: String,
    created_at: String,
    state: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct AiSource {
    provider: String,
    model: String,
    date: String,
    reviewed_by: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct CrossReference {
    key: String,
    target_kind: String,
    resolved: bool,
}

#[derive(Debug, Serialize)]
struct TableSummary {
    line: usize,
    columns: Vec<String>,
    rows: usize,
    numeric_columns: BTreeMap<String, f64>,
}

#[derive(Clone, Debug)]
struct IndexEntry {
    term: String,
    anchor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExportRequest {
    text: String,
    file_path: Option<String>,
    target: String,
    output_path: String,
    options: Value,
}

#[derive(Debug, Deserialize)]
struct PrepareExportRequest {
    text: String,
    file_path: Option<String>,
    target: String,
    options: Value,
}

#[derive(Debug, Serialize)]
struct ExportResponse {
    output_path: String,
    manifest_path: Option<String>,
    manifest: ExportManifest,
    diagnostics: Vec<DocumentDiagnostic>,
}

#[derive(Debug, Deserialize)]
struct SaveFileRequest {
    path: String,
    text: String,
    expected_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RenameFileRequest {
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
struct DuplicateFileRequest {
    from: String,
    to: String,
}

#[derive(Debug, Serialize)]
struct FileResponse {
    path: String,
    text: String,
    hash: String,
    modified: Option<String>,
}

#[derive(Debug, Serialize)]
struct FileMetadata {
    path: String,
    exists: bool,
    hash: Option<String>,
    modified: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WatchFileRequest {
    root: String,
    included: Vec<String>,
}

#[derive(Debug, Serialize)]
struct WatchFileResponse {
    paths: Vec<FileMetadata>,
    native_watcher: bool,
}

#[cfg(feature = "native-watch")]
#[derive(Clone, Debug, Serialize)]
struct FileWatchEventPayload {
    paths: Vec<String>,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct WorkspaceFileRequest {
    root: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceFileEntry {
    path: String,
    name: String,
    relative_path: String,
    kind: String,
    depth: usize,
}

#[derive(Debug, Deserialize)]
struct SnapshotRequest {
    text: String,
    file_path: Option<String>,
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AiCleanupRequest {
    text: String,
    add_provenance: bool,
    mark_as_draft: bool,
}

#[derive(Debug, Serialize)]
struct AiCleanupResponse {
    cleaned_markdown: String,
    issues: Vec<String>,
    provenance_block: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExportReadinessReport {
    ready: bool,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    diagnostics: Vec<DocumentDiagnostic>,
    manifest: ExportManifest,
}

#[derive(Debug, Serialize)]
struct SnapshotListItem {
    snapshot_path: String,
    metadata_path: String,
    hash: Option<String>,
    created_at: Option<String>,
    label: Option<String>,
}

#[derive(Debug, Serialize)]
struct SnapshotResponse {
    snapshot_path: String,
    metadata_path: String,
    hash: String,
}

#[derive(Debug, Serialize)]
struct GitStatus {
    inside_repo: bool,
    branch: Option<String>,
    dirty: bool,
    summary: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GitPathRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
struct GitCommitRequest {
    path: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct GitTagRequest {
    path: String,
    tag: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct GitRestoreRequest {
    path: String,
    revision: String,
}

#[derive(Debug, Serialize)]
struct GitHistoryEntry {
    revision: String,
    author: String,
    date: String,
    subject: String,
}

#[tauri::command]
fn read_file(path: String) -> Result<FileResponse, String> {
    let path_buf = PathBuf::from(path);
    let text = fs::read_to_string(&path_buf).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&path_buf).ok();
    Ok(FileResponse {
        path: path_to_string(&path_buf),
        hash: sha256_hex(text.as_bytes()),
        modified: metadata.and_then(modified_time),
        text,
    })
}

#[tauri::command]
fn open_file(path: String) -> Result<FileResponse, String> {
    read_file(path)
}

#[tauri::command]
fn save_file(request: SaveFileRequest) -> Result<FileResponse, String> {
    let path = PathBuf::from(&request.path);
    if let Some(expected_hash) = &request.expected_hash {
        if path.exists() {
            let current = fs::read(&path).map_err(|err| err.to_string())?;
            let current_hash = sha256_hex(&current);
            if &current_hash != expected_hash {
                return Err(
                    "File changed on disk since it was opened; resolve the external conflict before saving."
                        .to_string(),
                );
            }
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(&path, request.text.as_bytes()).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&path).ok();
    Ok(FileResponse {
        path: path_to_string(&path),
        hash: sha256_hex(request.text.as_bytes()),
        modified: metadata.and_then(modified_time),
        text: request.text,
    })
}

#[tauri::command]
fn save_file_as(request: SaveFileRequest) -> Result<FileResponse, String> {
    save_file(SaveFileRequest {
        expected_hash: None,
        ..request
    })
}

#[tauri::command]
fn rename_file(request: RenameFileRequest) -> Result<FileMetadata, String> {
    let from = PathBuf::from(&request.from);
    let to = PathBuf::from(&request.to);
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::rename(&from, &to).map_err(|err| err.to_string())?;
    file_metadata(path_to_string(&to))
}

#[tauri::command]
fn duplicate_file(request: DuplicateFileRequest) -> Result<FileResponse, String> {
    let from = PathBuf::from(&request.from);
    let to = PathBuf::from(&request.to);
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::copy(&from, &to).map_err(|err| err.to_string())?;
    read_file(path_to_string(&to))
}

#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg("-R").arg(&path);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("explorer");
        command.arg(format!("/select,{path}"));
        command
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let target = PathBuf::from(&path)
            .parent()
            .map(path_to_string)
            .unwrap_or(path);
        let mut command = Command::new("xdg-open");
        command.arg(target);
        command
    };

    let status = command.status().map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Unable to reveal path; command exited with {status}"
        ))
    }
}

#[tauri::command]
fn file_metadata(path: String) -> Result<FileMetadata, String> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Ok(FileMetadata {
            path: path_to_string(&path_buf),
            exists: false,
            hash: None,
            modified: None,
        });
    }
    let text = fs::read(&path_buf).map_err(|err| err.to_string())?;
    let metadata = fs::metadata(&path_buf).ok();
    Ok(FileMetadata {
        path: path_to_string(&path_buf),
        exists: true,
        hash: Some(sha256_hex(&text)),
        modified: metadata.and_then(modified_time),
    })
}

#[tauri::command]
fn watch_file(request: WatchFileRequest) -> Result<WatchFileResponse, String> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for path in std::iter::once(request.root).chain(request.included.into_iter()) {
        let metadata = file_metadata(path)?;
        if seen.insert(metadata.path.clone()) {
            paths.push(metadata);
        }
    }
    Ok(WatchFileResponse {
        paths,
        native_watcher: false,
    })
}

#[tauri::command]
#[cfg(feature = "native-watch")]
fn start_file_watcher(
    app: tauri::AppHandle,
    state: State<FileWatcherState>,
    request: WatchFileRequest,
) -> Result<WatchFileResponse, String> {
    let mut response = watch_file(request)?;
    let watch_paths = response
        .paths
        .iter()
        .filter(|metadata| metadata.exists)
        .map(|metadata| metadata.path.clone())
        .collect::<Vec<_>>();
    let signature = watch_paths.join("\n");
    let mut active = state
        .watcher
        .lock()
        .map_err(|_| "File watcher state lock poisoned.".to_string())?;

    if active
        .as_ref()
        .map(|watcher| watcher.signature.as_str() == signature.as_str())
        .unwrap_or(false)
    {
        response.native_watcher = !watch_paths.is_empty();
        return Ok(response);
    }

    *active = None;
    if watch_paths.is_empty() {
        return Ok(response);
    }

    let event_app = app.clone();
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<Event>| match result {
            Ok(event) => {
                if !notify_event_should_emit(&event.kind) {
                    return;
                }
                let payload = FileWatchEventPayload {
                    paths: event
                        .paths
                        .iter()
                        .map(|path| path_to_string(path))
                        .collect(),
                    kind: format!("{:?}", event.kind),
                };
                let _ = event_app.emit("neditor-file-watch-event", payload);
            }
            Err(error) => {
                let _ = event_app.emit("neditor-file-watch-error", error.to_string());
            }
        },
        Config::default(),
    )
    .map_err(|err| err.to_string())?;

    for path in &watch_paths {
        watcher
            .watch(Path::new(path), RecursiveMode::NonRecursive)
            .map_err(|err| err.to_string())?;
    }

    *active = Some(ActiveFileWatcher {
        _watcher: watcher,
        signature,
    });
    response.native_watcher = true;
    Ok(response)
}

#[tauri::command]
#[cfg(not(feature = "native-watch"))]
fn start_file_watcher(
    state: State<FileWatcherState>,
    request: WatchFileRequest,
) -> Result<WatchFileResponse, String> {
    let response = watch_file(request)?;
    let signature = response
        .paths
        .iter()
        .filter(|metadata| metadata.exists)
        .map(|metadata| metadata.path.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let mut active = state
        .watcher
        .lock()
        .map_err(|_| "File watcher state lock poisoned.".to_string())?;
    *active = Some(ActiveFileWatcher { signature });
    Ok(response)
}

#[tauri::command]
fn stop_file_watcher(state: State<FileWatcherState>) -> Result<(), String> {
    let mut active = state
        .watcher
        .lock()
        .map_err(|_| "File watcher state lock poisoned.".to_string())?;
    *active = None;
    Ok(())
}

#[cfg(feature = "native-watch")]
fn notify_event_should_emit(kind: &EventKind) -> bool {
    !matches!(kind, EventKind::Access(_))
}

#[tauri::command]
fn list_workspace_files(request: WorkspaceFileRequest) -> Result<Vec<WorkspaceFileEntry>, String> {
    let root = PathBuf::from(&request.root);
    if !root.exists() {
        return Err(format!("Workspace root does not exist: {}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!(
            "Workspace root is not a folder: {}",
            root.display()
        ));
    }

    let canonical_root = root.canonicalize().unwrap_or(root);
    let mut entries = Vec::new();
    scan_workspace_dir(&canonical_root, &canonical_root, 0, &mut entries)?;
    entries.sort_by(|left, right| {
        left.relative_path
            .to_ascii_lowercase()
            .cmp(&right.relative_path.to_ascii_lowercase())
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    Ok(entries)
}

#[tauri::command]
fn compile_document(request: CompileRequest) -> Result<CompileResponse, String> {
    Ok(compile(request))
}

#[tauri::command]
fn export_document(request: ExportRequest) -> Result<ExportResponse, String> {
    let compile_response = compile(CompileRequest {
        text: request.text,
        file_path: request.file_path,
    });
    let mut manifest = compile_response.export_manifest.clone();
    manifest.export_target = request.target.clone();
    manifest.export_options = request.options.clone();

    let output_path = PathBuf::from(&request.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    match request.target.as_str() {
        "html" => fs::write(
            &output_path,
            render_full_html(&compile_response, &request.options),
        )
        .map_err(|err| err.to_string())?,
        "pdf" => fs::write(
            &output_path,
            render_pdf_bytes(&compile_response, &request.options),
        )
        .map_err(|err| err.to_string())?,
        "docx" => fs::write(
            &output_path,
            render_docx_bytes(&compile_response, &request.options)?,
        )
        .map_err(|err| err.to_string())?,
        "pptx" => fs::write(
            &output_path,
            render_pptx_bytes(&compile_response, &request.options)?,
        )
        .map_err(|err| err.to_string())?,
        "markdown-bundle" | "markdown" => fs::write(
            &output_path,
            render_markdown_bundle_bytes(&compile_response, &manifest)?,
        )
        .map_err(|err| err.to_string())?,
        other => {
            return Err(format!(
                "Unsupported export target '{other}'. Use html, pdf, docx, pptx, or markdown-bundle."
            ));
        }
    }

    let manifest_path = if request
        .options
        .get("includeManifest")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        let manifest_path = PathBuf::from(format!("{}.manifest.json", output_path.display()));
        let manifest_json =
            serde_json::to_string_pretty(&manifest).map_err(|err| err.to_string())?;
        fs::write(&manifest_path, manifest_json).map_err(|err| err.to_string())?;
        Some(path_to_string(&manifest_path))
    } else {
        None
    };

    Ok(ExportResponse {
        output_path: path_to_string(&output_path),
        manifest_path,
        manifest,
        diagnostics: compile_response.diagnostics,
    })
}

#[tauri::command]
fn prepare_for_export(request: PrepareExportRequest) -> ExportReadinessReport {
    let file_path = request.file_path.clone();
    let mut response = compile(CompileRequest {
        text: request.text,
        file_path,
    });
    response.export_manifest.export_target = request.target.clone();
    response.export_manifest.export_options = request.options.clone();
    validate_export_settings(&request.target, &request.options, &mut response.diagnostics);
    validate_git_export_cleanliness(request.file_path.as_deref(), &mut response.diagnostics);
    let error_count = response
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "error")
        .count();
    let warning_count = response
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "warning")
        .count();
    let info_count = response
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "info")
        .count();
    ExportReadinessReport {
        ready: error_count == 0 && warning_count == 0,
        error_count,
        warning_count,
        info_count,
        diagnostics: response.diagnostics,
        manifest: response.export_manifest,
    }
}

fn validate_git_export_cleanliness(
    file_path: Option<&str>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(path) = file_path else {
        return;
    };
    let Ok(status) = get_git_status(Some(path.to_string())) else {
        return;
    };
    if status.inside_repo && status.dirty {
        let summary = if status.summary.is_empty() {
            "working tree has uncommitted changes".to_string()
        } else {
            status.summary.join("; ")
        };
        diagnostics.push(diag(
            "warning",
            format!("Git working tree is dirty before export: {summary}"),
            Some(path.to_string()),
            None,
            Some("Commit, stash, or intentionally document the dirty state before exporting."),
        ));
    }
}

fn validate_export_settings(
    target: &str,
    options: &Value,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if !matches!(
        target,
        "html" | "pdf" | "docx" | "pptx" | "markdown-bundle" | "markdown"
    ) {
        diagnostics.push(diag(
            "error",
            format!("Unsupported export target: {target}"),
            None,
            None,
            Some("Use html, pdf, docx, pptx, or markdown-bundle."),
        ));
    }
    if options
        .get("watermark")
        .is_some_and(|value| !value.is_string())
    {
        diagnostics.push(diag(
            "error",
            "Export watermark must be a string.",
            None,
            None,
            Some("Use a text watermark or remove the option."),
        ));
    }
    for option in [
        "includeManifest",
        "includeGlossary",
        "includeComments",
        "includeProvenance",
    ] {
        if options.get(option).is_some_and(|value| !value.is_boolean()) {
            diagnostics.push(diag(
                "error",
                format!("{option} must be true or false."),
                None,
                None,
                Some("Use boolean values for export options."),
            ));
        }
    }
}

#[tauri::command]
fn create_snapshot(
    app: tauri::AppHandle,
    request: SnapshotRequest,
) -> Result<SnapshotResponse, String> {
    let source_hash = sha256_hex(request.text.as_bytes());
    let workspace_id = snapshot_workspace_id(request.file_path.as_deref());
    let label = request
        .label
        .unwrap_or_else(|| "snapshot".to_string())
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect::<String>();
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let root = app_snapshot_root(&app, Some(&workspace_id))?;
    fs::create_dir_all(&root).map_err(|err| err.to_string())?;
    let snapshot_path = root.join(format!("{timestamp}-{label}.md"));
    let metadata_path = root.join(format!("{timestamp}-{label}.json"));
    fs::write(&snapshot_path, request.text.as_bytes()).map_err(|err| err.to_string())?;
    let metadata = json!({
        "hash": source_hash,
        "createdAt": Utc::now().to_rfc3339(),
        "sourcePath": request.file_path,
        "label": label
    });
    fs::write(
        &metadata_path,
        serde_json::to_vec_pretty(&metadata).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(SnapshotResponse {
        snapshot_path: path_to_string(&snapshot_path),
        metadata_path: path_to_string(&metadata_path),
        hash: source_hash,
    })
}

#[tauri::command]
fn list_snapshots(
    app: tauri::AppHandle,
    file_path: Option<String>,
) -> Result<Vec<SnapshotListItem>, String> {
    let root = app_snapshot_root(&app, file_path.as_deref())?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(&root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let metadata_text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let metadata = serde_json::from_str::<Value>(&metadata_text).unwrap_or_else(|_| json!({}));
        let snapshot_path = path.with_extension("md");
        items.push(SnapshotListItem {
            snapshot_path: path_to_string(&snapshot_path),
            metadata_path: path_to_string(&path),
            hash: metadata
                .get("hash")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            created_at: metadata
                .get("createdAt")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            label: metadata
                .get("label")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        });
    }
    items.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(items)
}

#[tauri::command]
fn restore_snapshot(snapshot_path: String) -> Result<FileResponse, String> {
    read_file(snapshot_path)
}

#[tauri::command]
fn get_git_status(path: Option<String>) -> Result<GitStatus, String> {
    let cwd = path
        .as_deref()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .and_then(|path| {
            if path.is_file() {
                path.parent().map(Path::to_path_buf)
            } else {
                Some(path)
            }
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let inside = run_git(&cwd, &["rev-parse", "--is-inside-work-tree"])?;
    if inside.trim() != "true" {
        return Ok(GitStatus {
            inside_repo: false,
            branch: None,
            dirty: false,
            summary: Vec::new(),
        });
    }

    let branch = run_git(&cwd, &["branch", "--show-current"])
        .ok()
        .map(|branch| branch.trim().to_string())
        .filter(|branch| !branch.is_empty());
    let status = run_git(&cwd, &["status", "--short"]).unwrap_or_default();
    let summary = status.lines().map(ToString::to_string).collect::<Vec<_>>();
    Ok(GitStatus {
        inside_repo: true,
        branch,
        dirty: !summary.is_empty(),
        summary,
    })
}

#[tauri::command]
fn git_history(request: GitPathRequest) -> Result<Vec<GitHistoryEntry>, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let output = run_git(
        &cwd,
        &[
            "log",
            "--date=iso-strict",
            "--format=%H%x1f%an%x1f%ad%x1f%s",
            "--",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(request.path.as_str()),
        ],
    )?;
    Ok(output
        .lines()
        .filter_map(|line| {
            let parts = line.split('\u{1f}').collect::<Vec<_>>();
            if parts.len() < 4 {
                return None;
            }
            Some(GitHistoryEntry {
                revision: parts[0].to_string(),
                author: parts[1].to_string(),
                date: parts[2].to_string(),
                subject: parts[3].to_string(),
            })
        })
        .collect())
}

#[tauri::command]
fn git_diff(request: GitPathRequest) -> Result<String, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    run_git(
        &cwd,
        &[
            "diff",
            "--",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(request.path.as_str()),
        ],
    )
}

#[tauri::command]
fn commit_document_changes(request: GitCommitRequest) -> Result<GitStatus, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(request.path.as_str());
    run_git(&cwd, &["add", "--", file_name])?;
    run_git(&cwd, &["commit", "-m", &request.message, "--", file_name])?;
    get_git_status(Some(request.path))
}

#[tauri::command]
fn tag_release(request: GitTagRequest) -> Result<String, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    run_git(&cwd, &["tag", "-a", &request.tag, "-m", &request.message])?;
    Ok(request.tag)
}

#[tauri::command]
fn restore_git_revision(request: GitRestoreRequest) -> Result<FileResponse, String> {
    let path = PathBuf::from(&request.path);
    let cwd = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(request.path.as_str());
    let content = run_git(
        &cwd,
        &["show", &format!("{}:{file_name}", request.revision)],
    )?;
    fs::write(&path, content.as_bytes()).map_err(|err| err.to_string())?;
    read_file(path_to_string(&path))
}

#[tauri::command]
fn list_transform_engines() -> Vec<Value> {
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
fn run_transform(name: String, body: String) -> Result<TransformArtifact, String> {
    if !supported_transform(&name) {
        return Err(format!("Unknown transform: {name}"));
    }
    let mut diagnostics = Vec::new();
    Ok(render_transform(&name, &body, &mut diagnostics))
}

#[tauri::command]
fn run_external_transform(request: ExternalTransformRequest) -> Result<TransformArtifact, String> {
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

    let timeout_ms = request
        .timeout_ms
        .unwrap_or(DEFAULT_TRANSFORM_TIMEOUT_MS)
        .clamp(1, MAX_TRANSFORM_TIMEOUT_MS);
    let input_mode = request.input_mode.as_deref().unwrap_or("stdin");
    if !matches!(input_mode, "stdin" | "file") {
        return Err("External transform input_mode must be 'stdin' or 'file'.".to_string());
    }

    execute_external_transform(
        &request.name,
        &request.body,
        &engine_path,
        input_mode,
        timeout_ms,
    )
}

#[tauri::command]
fn cleanup_ai_paste(request: AiCleanupRequest) -> AiCleanupResponse {
    let mut issues = Vec::new();
    let mut cleaned = request.text.replace("\r\n", "\n");
    let chat_labels = [
        "ChatGPT said:",
        "Claude said:",
        "Gemini said:",
        "Copilot said:",
        "Assistant:",
        "User:",
    ];
    for label in chat_labels {
        if cleaned.contains(label) {
            cleaned = cleaned.replace(label, "");
            issues.push(format!("Removed chat label '{label}'"));
        }
    }
    cleaned = normalize_markdown_lists(&cleaned, &mut issues);
    cleaned = normalize_markdown_tables(&cleaned, &mut issues);
    if request.mark_as_draft && !cleaned.contains("status: draft") {
        cleaned = format!("<!-- draft: AI paste cleanup review required -->\n\n{cleaned}");
        issues.push("Marked inserted content as draft.".to_string());
    }
    let provenance_block = if request.add_provenance {
        Some(format!(
            "```ai-source\nprovider: unknown\nmodel: unknown\ndate: {}\nreviewedBy: \nstatus: needs-review\n```",
            Utc::now().date_naive()
        ))
    } else {
        None
    };
    if let Some(block) = &provenance_block {
        cleaned = format!("{cleaned}\n\n{block}\n");
    }
    AiCleanupResponse {
        cleaned_markdown: cleaned.trim().to_string(),
        issues,
        provenance_block,
    }
}

fn compile(request: CompileRequest) -> CompileResponse {
    let mut diagnostics = Vec::new();
    let mut include_graph = Vec::new();
    let root_path = request.file_path.as_deref().map(PathBuf::from);
    let root_file = root_path
        .as_ref()
        .map(|path| path_to_string(path.as_path()))
        .unwrap_or_else(|| "untitled.md".to_string());
    let mut visited = HashSet::new();
    let mut source_map = Vec::new();
    let mut generated_line_count = 0usize;
    let source = expand_includes(
        &request.text,
        root_path.as_deref(),
        &root_file,
        0,
        &mut visited,
        &mut include_graph,
        &mut source_map,
        &mut generated_line_count,
        &mut diagnostics,
    );
    let (mut metadata, body, body_start_line) =
        parse_front_matter(&source, &mut diagnostics, Some(root_file.clone()));
    merge_project_variables(&mut metadata, root_path.as_deref(), &mut diagnostics);
    normalize_source_map_after_front_matter(&mut source_map, body_start_line);
    let mut calculation_context = HashMap::new();
    let formula_graph = collect_calculations(&body, &mut calculation_context, &mut diagnostics);
    let interpolated =
        interpolate_variables(&body, &metadata, &calculation_context, &mut diagnostics);
    let headings = extract_headings(&interpolated);
    let bibliography = collect_bibliography(
        &interpolated,
        &metadata,
        root_path.as_deref(),
        &mut diagnostics,
    );
    let duplicate_bibliography_keys = duplicate_bibliography_keys(&bibliography);
    let glossary = collect_glossary(&interpolated);
    let citation_references = collect_citation_references(&interpolated);
    let citations = citation_keys_from_references(&citation_references);
    let labels = collect_labels(&interpolated);
    let cross_references = collect_cross_references(&interpolated, &labels, &mut diagnostics);
    let index_entries = collect_index_entries(&interpolated, &metadata, &headings, &glossary);
    let index_terms = index_entries
        .iter()
        .map(|entry| entry.term.clone())
        .collect::<Vec<_>>();
    let layout_directives = collect_fence_bodies(&interpolated, "layout");
    let comments = collect_comments(&interpolated);
    let ai_sources = collect_ai_sources(&interpolated);
    let with_toc = inject_generated_sections(
        &interpolated,
        &metadata,
        &headings,
        &index_entries,
        &bibliography,
    );
    let index_marker_markdown = strip_index_markers(&with_toc);
    let (transformed_markdown, transform_artifacts) =
        apply_transforms(&index_marker_markdown, &mut diagnostics);
    let citation_markdown = render_citations(&transformed_markdown, &bibliography);
    let table_formula_markdown =
        evaluate_markdown_table_formulas(&citation_markdown, &mut diagnostics);
    validate_image_paths(
        &table_formula_markdown,
        root_path.as_deref(),
        &mut diagnostics,
    );
    validate_link_paths(
        &table_formula_markdown,
        root_path.as_deref(),
        &mut diagnostics,
    );
    let figure_markdown = render_figures(&table_formula_markdown);
    let equation_markdown = render_equations(&figure_markdown);
    let callout_markdown = render_callouts(&equation_markdown);
    let layout_markdown = render_layout_tokens(&callout_markdown);
    let mut document_ast = build_document_ast(&layout_markdown);
    attach_source_ranges(&mut document_ast, |line, end_line| {
        ast_source_range_for_generated_lines(&source_map, line, end_line)
    });
    let html = markdown_to_html(&layout_markdown, &glossary);
    let title = metadata
        .get("title")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| headings.first().map(|heading| heading.text.clone()))
        .unwrap_or_else(|| "Untitled Document".to_string());
    let status = metadata
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft")
        .to_string();
    validate_document(
        &metadata,
        &citations,
        &bibliography,
        &duplicate_bibliography_keys,
        &comments,
        &ai_sources,
        !bibliography.is_empty(),
        &mut diagnostics,
    );
    let included_files = include_graph
        .iter()
        .filter_map(|edge| manifest_file(&edge.child))
        .collect::<Vec<_>>();
    let manifest = ExportManifest {
        document_title: title.clone(),
        document_version: metadata
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("0.1.0")
            .to_string(),
        status: status.clone(),
        exported_at: Utc::now().to_rfc3339(),
        source_hash: sha256_hex(source.as_bytes()),
        included_files,
        export_target: "preview".to_string(),
        export_options: json!({}),
        transform_artifacts: transform_artifacts
            .iter()
            .map(|artifact| {
                json!({
                    "id": artifact.id,
                    "name": artifact.name,
                    "outputKind": artifact.output_kind,
                    "sourceHash": artifact.source_hash
                })
            })
            .collect(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let table_summaries = collect_table_summaries(&table_formula_markdown);
    let semantic = SemanticDocument {
        title,
        status,
        headings: headings.clone(),
        outline: headings,
        tables: table_summaries.len(),
        table_summaries,
        figures: count_figures(&transformed_markdown),
        equations: count_equations(&transformed_markdown),
        citations,
        citation_references,
        duplicate_bibliography_keys,
        glossary,
        layout_directives,
        comments,
        ai_sources,
        labels,
        cross_references,
    };
    CompileResponse {
        compiled_markdown: layout_markdown,
        html,
        semantic,
        document_ast,
        diagnostics,
        include_graph,
        source_map,
        metadata,
        bibliography,
        index_terms,
        formula_graph,
        transform_artifacts,
        export_manifest: manifest,
    }
}

#[allow(clippy::too_many_arguments)]
fn expand_includes(
    text: &str,
    current_path: Option<&Path>,
    source_file: &str,
    depth: usize,
    visited: &mut HashSet<PathBuf>,
    include_graph: &mut Vec<IncludeEdge>,
    source_map: &mut Vec<SourceMapEntry>,
    generated_line_count: &mut usize,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    if depth > MAX_INCLUDE_DEPTH {
        diagnostics.push(diag(
            "error",
            "Maximum include depth exceeded.",
            Some(source_file.to_string()),
            None,
            Some("Reduce nested include directives."),
        ));
        return String::new();
    }

    let base_dir = current_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut output = String::new();
    for (line_index, line) in text.lines().enumerate() {
        if let Some(include_target) = parse_include_directive(line) {
            let child = base_dir.join(include_target);
            let canonical = child.canonicalize().unwrap_or(child.clone());
            if visited.contains(&canonical) {
                diagnostics.push(diag(
                    "error",
                    "Circular include detected.",
                    Some(source_file.to_string()),
                    Some(line_index + 1),
                    Some("Remove the cycle or include a different file."),
                ));
                continue;
            }
            if !child.exists() {
                diagnostics.push(diag(
                    "error",
                    format!("Missing include file: {}", child.display()),
                    Some(source_file.to_string()),
                    Some(line_index + 1),
                    Some("Create the file or update the include path."),
                ));
                continue;
            }
            match fs::read_to_string(&child) {
                Ok(child_text) => {
                    include_graph.push(IncludeEdge {
                        parent: source_file.to_string(),
                        child: path_to_string(&child),
                        depth: depth + 1,
                    });
                    visited.insert(canonical.clone());
                    let child_without_front_matter = strip_front_matter(&child_text);
                    push_unmapped_expanded_text(
                        &mut output,
                        generated_line_count,
                        &format!("\n\n<!-- begin include: {} -->\n", child.display()),
                    );
                    output.push_str(&expand_includes(
                        &child_without_front_matter,
                        Some(&child),
                        &path_to_string(&child),
                        depth + 1,
                        visited,
                        include_graph,
                        source_map,
                        generated_line_count,
                        diagnostics,
                    ));
                    push_unmapped_expanded_text(
                        &mut output,
                        generated_line_count,
                        &format!("\n<!-- end include: {} -->\n\n", child.display()),
                    );
                    visited.remove(&canonical);
                }
                Err(err) => diagnostics.push(diag(
                    "error",
                    format!("Unable to read include file: {err}"),
                    Some(source_file.to_string()),
                    Some(line_index + 1),
                    Some("Check file permissions."),
                )),
            }
        } else {
            let generated_line = *generated_line_count + 1;
            source_map.push(SourceMapEntry {
                generated_line,
                source_file: source_file.to_string(),
                source_line: line_index + 1,
            });
            output.push_str(line);
            output.push('\n');
            *generated_line_count += 1;
        }
    }
    output
}

fn push_unmapped_expanded_text(output: &mut String, generated_line_count: &mut usize, text: &str) {
    output.push_str(text);
    *generated_line_count += text.chars().filter(|ch| *ch == '\n').count();
}

fn parse_include_directive(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("!include ") {
        return Some(rest.trim());
    }
    if let Some(rest) = trimmed.strip_prefix("{{include ") {
        return rest.strip_suffix("}}").map(str::trim);
    }
    if let Some(rest) = trimmed.strip_prefix("<!-- include:") {
        return rest.strip_suffix("-->").map(str::trim);
    }
    None
}

fn parse_front_matter(
    text: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
    source_file: Option<String>,
) -> (Value, String, usize) {
    if !text.starts_with("---\n") {
        return (json!({}), text.to_string(), 1);
    }
    let mut lines = text.lines();
    lines.next();
    let mut consumed_lines = 1usize;
    let mut yaml = String::new();
    for line in &mut lines {
        consumed_lines += 1;
        if line.trim() == "---" {
            let body = lines.collect::<Vec<_>>().join("\n");
            let metadata = serde_yaml::from_str::<Value>(&yaml).unwrap_or_else(|err| {
                diagnostics.push(diag(
                    "error",
                    format!("Invalid YAML front matter: {err}"),
                    source_file.clone(),
                    None,
                    Some("Fix the YAML syntax between the opening and closing --- markers."),
                ));
                json!({})
            });
            return (metadata, body, consumed_lines + 1);
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    diagnostics.push(diag(
        "error",
        "Front matter was opened but not closed.",
        source_file,
        Some(1),
        Some("Add a closing --- marker."),
    ));
    (json!({}), text.to_string(), 1)
}

fn merge_project_variables(
    metadata: &mut Value,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(path) = project_variables_path(root_path) else {
        return;
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            diagnostics.push(diag(
                "warning",
                format!("Unable to read project variables {}: {err}", path.display()),
                Some(path_to_string(&path)),
                None,
                Some("Check permissions or remove the project variables file."),
            ));
            return;
        }
    };
    let mut variables = match serde_yaml::from_str::<Value>(&text) {
        Ok(value) => value,
        Err(err) => {
            diagnostics.push(diag(
                "error",
                format!("Invalid project variables YAML {}: {err}", path.display()),
                Some(path_to_string(&path)),
                None,
                Some("Fix .neditor/variables.yaml or variables.yml."),
            ));
            return;
        }
    };
    if let Some(inner) = variables.get("variables").cloned() {
        variables = inner;
    }
    let (Some(target), Some(source)) = (metadata.as_object_mut(), variables.as_object()) else {
        diagnostics.push(diag(
            "warning",
            format!(
                "Project variables {} must be a YAML mapping.",
                path.display()
            ),
            Some(path_to_string(&path)),
            None,
            Some("Use key-value YAML such as client: Acme."),
        ));
        return;
    };
    for (key, value) in source {
        target.entry(key.clone()).or_insert_with(|| value.clone());
    }
}

fn project_variables_path(root_path: Option<&Path>) -> Option<PathBuf> {
    let mut dir = root_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())?;
    loop {
        for name in ["variables.yaml", "variables.yml"] {
            let candidate = dir.join(".neditor").join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn normalize_source_map_after_front_matter(
    source_map: &mut Vec<SourceMapEntry>,
    body_start_line: usize,
) {
    let offset = body_start_line.saturating_sub(1);
    source_map.retain(|entry| entry.generated_line >= body_start_line);
    for entry in source_map {
        entry.generated_line = entry.generated_line.saturating_sub(offset);
    }
}

fn ast_source_range_for_generated_lines(
    source_map: &[SourceMapEntry],
    line: usize,
    end_line: usize,
) -> Option<AstSourceRange> {
    let start = source_map
        .iter()
        .find(|entry| entry.generated_line == line)?;
    let end = source_map
        .iter()
        .rev()
        .find(|entry| {
            entry.generated_line >= line
                && entry.generated_line <= end_line
                && entry.source_file == start.source_file
        })
        .unwrap_or(start);
    Some(AstSourceRange {
        source_file: start.source_file.clone(),
        source_line: start.source_line,
        end_source_line: end.source_line,
    })
}

fn strip_front_matter(text: &str) -> String {
    if !text.starts_with("---\n") {
        return text.to_string();
    }
    let mut lines = text.lines();
    lines.next();
    for line in &mut lines {
        if line.trim() == "---" {
            return lines.collect::<Vec<_>>().join("\n");
        }
    }
    text.to_string()
}

fn interpolate_variables(
    text: &str,
    metadata: &Value,
    calculations: &HashMap<String, f64>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let (before, after_start) = rest.split_at(start);
        output.push_str(before);
        if let Some(end) = after_start.find("}}") {
            let token = after_start[2..end].trim();
            let replacement = if let Some(expr) = token.strip_prefix('=') {
                let expr = expr.trim();
                if let Some((name, filter)) = expr.split_once('|') {
                    calculations
                        .get(name.trim())
                        .map(|value| format_value(*value, filter.trim()))
                } else {
                    calculations.get(expr).map(|value| value.to_string())
                }
            } else {
                let (path, default_value) = variable_path_and_default(token);
                metadata_lookup(metadata, path)
                    .map(value_to_string)
                    .or(default_value)
            };
            if let Some(value) = replacement {
                output.push_str(&value);
            } else if matches!(token, "page" | "pages") {
                output.push_str(&format!("{{{{{token}}}}}"));
            } else {
                diagnostics.push(diag(
                    "warning",
                    format!("Missing document variable: {token}"),
                    None,
                    None,
                    Some("Define the variable in front matter or a calc block."),
                ));
                output.push_str(&format!("{{{{{token}}}}}"));
            }
            rest = &after_start[end + 2..];
        } else {
            output.push_str(after_start);
            rest = "";
        }
    }
    output.push_str(rest);
    output
}

fn variable_path_and_default(token: &str) -> (&str, Option<String>) {
    let Some((path, filter)) = token.split_once('|') else {
        return (token.trim(), None);
    };
    let filter = filter.trim();
    let default = filter
        .strip_prefix("default:")
        .or_else(|| filter.strip_prefix("default="))
        .or_else(|| filter.strip_prefix("default "))
        .map(unquote_variable_default);
    (path.trim(), default)
}

fn unquote_variable_default(value: &str) -> String {
    value
        .trim()
        .trim_matches(|ch| ch == '"' || ch == '\'')
        .to_string()
}

fn collect_calculations(
    text: &str,
    context: &mut HashMap<String, f64>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<FormulaValue> {
    let mut formulas = Vec::new();
    for block in collect_fence_bodies(text, "calc") {
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((name, expression)) = trimmed.split_once('=') {
                let name = name.trim().to_string();
                let expression = expression.trim().to_string();
                let dependencies = expression_dependencies(&expression);
                match eval_expression(&expression, context) {
                    Ok(value) => {
                        context.insert(name.clone(), value);
                        formulas.push(FormulaValue {
                            name,
                            expression,
                            value: Some(value),
                            error: None,
                            dependencies,
                        });
                    }
                    Err(error) => {
                        diagnostics.push(diag(
                            "error",
                            format!("Formula error for {name}: {error}"),
                            None,
                            None,
                            Some("Use numeric expressions, supported functions, or previously defined names."),
                        ));
                        formulas.push(FormulaValue {
                            name,
                            expression,
                            value: None,
                            error: Some(error),
                            dependencies,
                        });
                    }
                }
            }
        }
    }
    formulas
}

fn eval_expression(expression: &str, context: &HashMap<String, f64>) -> Result<f64, String> {
    let tokens = tokenize_expression(expression)?;
    let mut parser = FormulaParser {
        tokens,
        index: 0,
        context,
    };
    let value = parser.parse_expression()?;
    if parser.index != parser.tokens.len() {
        return Err("unexpected trailing input".to_string());
    }
    Ok(value)
}

#[derive(Clone, Debug, PartialEq)]
enum FormulaToken {
    Number(f64),
    Name(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LParen,
    RParen,
    Comma,
}

struct FormulaParser<'a> {
    tokens: Vec<FormulaToken>,
    index: usize,
    context: &'a HashMap<String, f64>,
}

impl FormulaParser<'_> {
    fn parse_expression(&mut self) -> Result<f64, String> {
        let mut value = self.parse_term()?;
        loop {
            match self.peek() {
                Some(FormulaToken::Plus) => {
                    self.index += 1;
                    value += self.parse_term()?;
                }
                Some(FormulaToken::Minus) => {
                    self.index += 1;
                    value -= self.parse_term()?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut value = self.parse_factor()?;
        loop {
            match self.peek() {
                Some(FormulaToken::Star) => {
                    self.index += 1;
                    value *= self.parse_factor()?;
                }
                Some(FormulaToken::Slash) => {
                    self.index += 1;
                    let divisor = self.parse_factor()?;
                    if divisor == 0.0 {
                        return Err("#DIV/0!".to_string());
                    }
                    value /= divisor;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(FormulaToken::Number(value)) => Ok(value),
            Some(FormulaToken::Minus) => Ok(-self.parse_factor()?),
            Some(FormulaToken::LParen) => {
                let value = self.parse_expression()?;
                self.expect(FormulaToken::RParen)?;
                Ok(value)
            }
            Some(FormulaToken::Name(name)) => {
                if matches!(self.peek(), Some(FormulaToken::LParen)) {
                    self.index += 1;
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(FormulaToken::RParen)) {
                        loop {
                            args.push(self.parse_expression()?);
                            if matches!(self.peek(), Some(FormulaToken::Comma)) {
                                self.index += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(FormulaToken::RParen)?;
                    eval_function(&name, &args)
                } else {
                    self.context
                        .get(&name)
                        .copied()
                        .ok_or_else(|| format!("#NAME? {name}"))
                }
            }
            other => Err(format!("unexpected token {other:?}")),
        }
    }

    fn expect(&mut self, token: FormulaToken) -> Result<(), String> {
        match self.next() {
            Some(found) if found == token => Ok(()),
            other => Err(format!("expected {token:?}, found {other:?}")),
        }
    }

    fn peek(&self) -> Option<&FormulaToken> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<FormulaToken> {
        let token = self.tokens.get(self.index).cloned();
        self.index += usize::from(token.is_some());
        token
    }
}

fn tokenize_expression(expression: &str) -> Result<Vec<FormulaToken>, String> {
    let mut tokens = Vec::new();
    let chars = expression.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
        } else if ch.is_ascii_digit() || ch == '.' {
            let start = index;
            index += 1;
            while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
                index += 1;
            }
            let value = chars[start..index]
                .iter()
                .collect::<String>()
                .parse::<f64>()
                .map_err(|_| "#VALUE?".to_string())?;
            tokens.push(FormulaToken::Number(value));
        } else if ch.is_ascii_alphabetic() || ch == '_' {
            let start = index;
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric() || chars[index] == '_')
            {
                index += 1;
            }
            tokens.push(FormulaToken::Name(chars[start..index].iter().collect()));
        } else {
            tokens.push(match ch {
                '+' => FormulaToken::Plus,
                '-' => FormulaToken::Minus,
                '*' => FormulaToken::Star,
                '/' => FormulaToken::Slash,
                '%' => FormulaToken::Percent,
                '(' => FormulaToken::LParen,
                ')' => FormulaToken::RParen,
                ',' => FormulaToken::Comma,
                _ => return Err(format!("unsupported formula character '{ch}'")),
            });
            index += 1;
        }
    }
    Ok(tokens)
}

fn eval_function(name: &str, args: &[f64]) -> Result<f64, String> {
    match name.to_ascii_uppercase().as_str() {
        "SUM" => Ok(args.iter().sum()),
        "AVG" => {
            if args.is_empty() {
                Err("#DIV/0!".to_string())
            } else {
                Ok(args.iter().sum::<f64>() / args.len() as f64)
            }
        }
        "MIN" => args
            .iter()
            .copied()
            .reduce(f64::min)
            .ok_or_else(|| "#VALUE?".to_string()),
        "MAX" => args
            .iter()
            .copied()
            .reduce(f64::max)
            .ok_or_else(|| "#VALUE?".to_string()),
        "COUNT" => Ok(args.len() as f64),
        "ROUND" => {
            let value = *args.first().ok_or_else(|| "#VALUE?".to_string())?;
            let places = args.get(1).copied().unwrap_or(0.0);
            let factor = 10f64.powf(places);
            Ok((value * factor).round() / factor)
        }
        "ABS" => args
            .first()
            .copied()
            .map(f64::abs)
            .ok_or_else(|| "#VALUE?".to_string()),
        "IF" => {
            if args.len() < 3 {
                Err("#VALUE?".to_string())
            } else if args[0] != 0.0 {
                Ok(args[1])
            } else {
                Ok(args[2])
            }
        }
        "PERCENT" => args
            .first()
            .copied()
            .map(|value| value * 100.0)
            .ok_or_else(|| "#VALUE?".to_string()),
        "CURRENCY" => args.first().copied().ok_or_else(|| "#VALUE?".to_string()),
        _ => Err(format!("#NAME? {name}")),
    }
}

fn apply_transforms(
    text: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> (String, Vec<TransformArtifact>) {
    let mut output = String::new();
    let mut artifacts = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        if let Some(info) = line.trim().strip_prefix("```") {
            let name = info.split_whitespace().next().unwrap_or("");
            if supported_transform(name) {
                let mut body = String::new();
                for body_line in lines.by_ref() {
                    if body_line.trim() == "```" {
                        break;
                    }
                    body.push_str(body_line);
                    body.push('\n');
                }
                let artifact = render_transform(name, &body, diagnostics);
                output.push_str(&artifact.html);
                output.push('\n');
                artifacts.push(artifact);
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }
    (output, artifacts)
}

fn render_transform(
    name: &str,
    body: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> TransformArtifact {
    let source_hash = sha256_hex(body.as_bytes());
    let mut artifact_diags = Vec::new();
    let html = match name {
        "calc" => "<aside class=\"transform transform-calc\">Calculations resolved into document variables.</aside>".to_string(),
        "csv" => render_delimited_table(body, ',', &mut artifact_diags, diagnostics),
        "tsv" => render_delimited_table(body, '\t', &mut artifact_diags, diagnostics),
        "json" => render_structured_data_html("json", body, &mut artifact_diags, diagnostics),
        "yaml" => render_structured_data_html("yaml", body, &mut artifact_diags, diagnostics),
        "glossary" => render_glossary_html(body),
        "layout" => render_layout_block_html(body),
        "timeline" => render_timeline_svg(body),
        "roadmap" => render_roadmap_html(body),
        "adr" => render_adr_html(body),
        "diff" => render_diff_html(body),
        "chart" => render_chart_svg(body),
        "openapi" => render_openapi_html(body, &mut artifact_diags, diagnostics),
        "json-schema" => render_json_schema_html(body, &mut artifact_diags, diagnostics),
        "bibtex" => render_bibtex_html(body, &mut artifact_diags, diagnostics),
        "geojson" => render_geojson_svg(body, &mut artifact_diags, diagnostics),
        "topojson" => render_topojson_svg(body, &mut artifact_diags, diagnostics),
        "stl" => render_stl_svg(body, &mut artifact_diags, diagnostics),
        "vega-lite" => render_vega_lite_svg(body, &mut artifact_diags, diagnostics),
        "mermaid" => render_mermaid_svg(body, &mut artifact_diags, diagnostics),
        "pikchr" | "dot" | "graphviz" | "plantuml" | "d2" => {
            let message = format!("{name} transform captured as source artifact; configure an engine for rendered output.");
            let diagnostic = diag(
                "warning",
                message.clone(),
                None,
                None,
                Some("Set the transform engine path in preferences when native rendering is unavailable."),
            );
            diagnostics.push(diagnostic.clone());
            artifact_diags.push(diagnostic);
            format!(
                "<section class=\"transform transform-pending\"><strong>{}</strong><pre>{}</pre><p>{}</p></section>",
                escape_html(name),
                escape_html(body),
                escape_html(&message)
            )
        }
        _ => format!("<pre>{}</pre>", escape_html(body)),
    };
    TransformArtifact {
        id: format!("{name}-{source_hash}"),
        name: name.to_string(),
        output_kind: if html.contains("<svg") { "svg" } else { "html" }.to_string(),
        cache_key: transform_cache_key(name, "embedded", "rust-native", &source_hash),
        execution_kind: "embedded".to_string(),
        engine_path: None,
        input_mode: "embedded".to_string(),
        duration_ms: None,
        source_hash,
        html,
        diagnostics: artifact_diags,
    }
}

fn normalize_markdown_lists(text: &str, issues: &mut Vec<String>) -> String {
    let mut changed = false;
    let output = text
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("• ") {
                changed = true;
                format!("{}- {}", &line[..line.len() - trimmed.len()], rest)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if changed {
        issues.push("Normalized bullet characters to Markdown list markers.".to_string());
    }
    output
}

fn normalize_markdown_tables(text: &str, issues: &mut Vec<String>) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    let mut changed = false;
    while index < lines.len() {
        let line = lines[index];
        if line.contains('\t') {
            let cells = line.split('\t').map(str::trim).collect::<Vec<_>>();
            output.push(format!("| {} |", cells.join(" | ")));
            if index + 1 < lines.len() && lines[index + 1].contains('\t') {
                output.push(format!(
                    "| {} |",
                    cells.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")
                ));
            }
            changed = true;
        } else {
            output.push(line.to_string());
        }
        index += 1;
    }
    if changed {
        issues.push("Converted tab-separated rows to Markdown table rows.".to_string());
    }
    output.join("\n")
}

fn supported_transform(name: &str) -> bool {
    matches!(
        name,
        "calc"
            | "csv"
            | "tsv"
            | "json"
            | "yaml"
            | "glossary"
            | "layout"
            | "timeline"
            | "roadmap"
            | "adr"
            | "diff"
            | "chart"
            | "mermaid"
            | "pikchr"
            | "dot"
            | "graphviz"
            | "plantuml"
            | "d2"
            | "vega-lite"
            | "geojson"
            | "topojson"
            | "stl"
            | "openapi"
            | "json-schema"
            | "bibtex"
    )
}

fn external_transform_supported(name: &str) -> bool {
    matches!(name, "pikchr" | "dot" | "graphviz" | "plantuml" | "d2")
}

fn markdown_to_html(markdown: &str, glossary: &BTreeMap<String, String>) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    let html_with_heading_ids = add_heading_ids(&html_output, &extract_headings(markdown));
    annotate_glossary_terms(&html_with_heading_ids, glossary)
}

fn add_heading_ids(html: &str, headings: &[Heading]) -> String {
    if headings.is_empty() {
        return html.to_string();
    }
    let mut output = String::with_capacity(html.len());
    let mut rest = html;
    let mut heading_index = 0usize;
    while let Some(start) = rest.find("<h") {
        output.push_str(&rest[..start]);
        let candidate = &rest[start..];
        let Some(level) = candidate
            .as_bytes()
            .get(2)
            .and_then(|byte| char::from(*byte).to_digit(10))
            .map(|digit| digit as usize)
        else {
            output.push_str("<h");
            rest = &candidate[2..];
            continue;
        };
        if !(1..=6).contains(&level) {
            output.push_str("<h");
            rest = &candidate[2..];
            continue;
        }
        let Some(tag_end) = candidate.find('>') else {
            output.push_str(candidate);
            return output;
        };
        let tag = &candidate[..=tag_end];
        if tag.contains(" id=") {
            output.push_str(tag);
        } else if let Some(heading) = headings.get(heading_index) {
            output.push_str(&tag[..tag.len() - 1]);
            output.push_str(&format!(" id=\"{}\">", escape_html(&heading.anchor)));
        } else {
            output.push_str(tag);
        }
        heading_index += 1;
        rest = &candidate[tag_end + 1..];
    }
    output.push_str(rest);
    output
}

fn annotate_glossary_terms(html: &str, glossary: &BTreeMap<String, String>) -> String {
    if glossary.is_empty() {
        return html.to_string();
    }
    let terms = glossary
        .iter()
        .filter(|(term, _)| !term.trim().is_empty())
        .map(|(term, definition)| (term.as_str(), definition.as_str()))
        .collect::<Vec<_>>();
    if terms.is_empty() {
        return html.to_string();
    }

    let mut output = String::with_capacity(html.len());
    let mut text_segment = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            if !text_segment.is_empty() {
                output.push_str(&annotate_glossary_text_segment(&text_segment, &terms));
                text_segment.clear();
            }
            in_tag = true;
            output.push(ch);
        } else if ch == '>' {
            in_tag = false;
            output.push(ch);
        } else if in_tag {
            output.push(ch);
        } else {
            text_segment.push(ch);
        }
    }
    if !text_segment.is_empty() {
        output.push_str(&annotate_glossary_text_segment(&text_segment, &terms));
    }
    output
}

fn annotate_glossary_text_segment(segment: &str, terms: &[(&str, &str)]) -> String {
    let mut output = String::with_capacity(segment.len());
    let mut index = 0;
    while index < segment.len() {
        if let Some((term, definition)) = terms
            .iter()
            .filter(|(term, _)| segment[index..].starts_with(*term))
            .filter(|(term, _)| glossary_term_has_boundaries(segment, index, index + term.len()))
            .max_by_key(|(term, _)| term.len())
        {
            let matched = &segment[index..index + term.len()];
            output.push_str(&format!(
                "<span class=\"glossary-term\" tabindex=\"0\" title=\"{}\" data-definition=\"{}\">{}</span>",
                escape_html(definition),
                escape_html(definition),
                matched
            ));
            index += term.len();
        } else if let Some(ch) = segment[index..].chars().next() {
            output.push(ch);
            index += ch.len_utf8();
        } else {
            break;
        }
    }
    output
}

fn glossary_term_has_boundaries(segment: &str, start: usize, end: usize) -> bool {
    let before = segment[..start].chars().next_back();
    let after = segment[end..].chars().next();
    !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn render_figures(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| render_figure_line(line).unwrap_or_else(|| line.to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_figure_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let alt_start = trimmed.strip_prefix("![")?;
    let (alt, after_alt) = alt_start.split_once("](")?;
    let (src, after_src) = after_alt.split_once(')')?;
    let attrs = after_src.trim();
    if !attrs.starts_with("{#fig:") || !attrs.ends_with('}') {
        return None;
    }
    let id = extract_label(attrs)?;
    let caption = extract_quoted_attribute(attrs, "caption").unwrap_or_else(|| alt.to_string());
    Some(format!(
        "<figure id=\"{}\" class=\"figure\"><img src=\"{}\" alt=\"{}\"/><figcaption>{}</figcaption></figure>",
        escape_html(&id),
        escape_html(src),
        escape_html(alt),
        escape_html(&caption)
    ))
}

fn validate_image_paths(
    markdown: &str,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let base_dir = root_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    for (line_index, line) in markdown.lines().enumerate() {
        let Some((_, after_alt)) = line
            .trim()
            .strip_prefix("![")
            .and_then(|rest| rest.split_once("]("))
        else {
            continue;
        };
        let Some((src, _)) = after_alt.split_once(')') else {
            continue;
        };
        if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
            continue;
        }
        let path = base_dir.join(src);
        if !path.exists() {
            diagnostics.push(diag(
                "warning",
                format!("Broken image path: {}", path.display()),
                Some(path_to_string(&path)),
                Some(line_index + 1),
                Some("Create the image file or update the image path."),
            ));
        }
    }
}

fn validate_link_paths(
    markdown: &str,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let base_dir = root_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    for (line_index, line) in markdown.lines().enumerate() {
        let mut search_from = 0usize;
        while let Some(relative_close) = line[search_from..].find("](") {
            let close_index = search_from + relative_close;
            let Some(open_index) = line[..close_index].rfind('[') else {
                search_from = close_index + 2;
                continue;
            };
            if open_index > 0 && line.as_bytes().get(open_index - 1) == Some(&b'!') {
                search_from = close_index + 2;
                continue;
            }
            let target_start = close_index + 2;
            let Some(relative_end) = line[target_start..].find(')') else {
                break;
            };
            let target_end = target_start + relative_end;
            if let Some(destination) = markdown_link_destination(&line[target_start..target_end]) {
                if should_validate_local_link(&destination) {
                    let path_part = destination
                        .split_once('#')
                        .map_or(destination.as_str(), |(path, _)| path);
                    if !path_part.is_empty() {
                        let path = base_dir.join(path_part);
                        if !path.exists() {
                            diagnostics.push(diag(
                                "warning",
                                format!("Broken link path: {}", path.display()),
                                Some(path_to_string(&path)),
                                Some(line_index + 1),
                                Some("Create the linked file or update the Markdown link."),
                            ));
                        }
                    }
                }
            }
            search_from = target_end + 1;
        }
    }
}

fn markdown_link_destination(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(stripped) = trimmed.strip_prefix('<') {
        return stripped
            .split_once('>')
            .map(|(destination, _)| destination.to_string());
    }
    Some(trimmed.split_whitespace().next()?.to_string())
}

fn should_validate_local_link(destination: &str) -> bool {
    !destination.starts_with('#')
        && !destination.starts_with("mailto:")
        && !destination.starts_with("tel:")
        && !destination.starts_with("data:")
        && !destination.starts_with("{{")
        && !destination.contains("://")
}

fn render_equations(markdown: &str) -> String {
    let mut output = String::new();
    let mut lines = markdown.lines().peekable();
    let mut equation_number = 1usize;
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed == "$$" || trimmed.starts_with("$$ ") {
            let mut body = String::new();
            let mut label = String::new();
            for equation_line in lines.by_ref() {
                let equation_trimmed = equation_line.trim();
                if equation_trimmed.starts_with("$$") {
                    label = extract_label(equation_trimmed).unwrap_or_default();
                    break;
                }
                body.push_str(equation_line);
                body.push('\n');
            }
            let id = if label.is_empty() {
                format!("eq:{equation_number}")
            } else {
                label
            };
            output.push_str(&format!(
                "<figure class=\"equation\" id=\"{}\"><pre><code>{}</code></pre><figcaption>Equation {}</figcaption></figure>\n",
                escape_html(&id),
                escape_html(body.trim()),
                equation_number
            ));
            equation_number += 1;
        } else {
            output.push_str(&render_inline_math(line));
            output.push('\n');
        }
    }
    output
}

fn render_inline_math(line: &str) -> String {
    let mut output = String::new();
    let mut rest = line;
    while let Some(start) = rest.find("\\(") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        if let Some(end) = after_start.find("\\)") {
            let math = &after_start[..end];
            output.push_str(&format!(
                "<span class=\"math\"><code>{}</code></span>",
                escape_html(math)
            ));
            rest = &after_start[end + 2..];
        } else {
            output.push_str(&rest[start..]);
            rest = "";
        }
    }
    output.push_str(rest);
    output
}

fn render_callouts(markdown: &str) -> String {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        let Some(after_marker) = trimmed.strip_prefix("> [!") else {
            output.push(line.to_string());
            index += 1;
            continue;
        };
        let Some(marker_end) = after_marker.find(']') else {
            output.push(line.to_string());
            index += 1;
            continue;
        };
        let callout_type = after_marker[..marker_end].trim().to_ascii_lowercase();
        if callout_type.is_empty()
            || !callout_type
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        {
            output.push(line.to_string());
            index += 1;
            continue;
        }
        let title = after_marker[marker_end + 1..].trim();
        let title = if title.is_empty() {
            callout_type.to_ascii_uppercase()
        } else {
            title.to_string()
        };
        index += 1;
        let mut body_lines = Vec::new();
        while index < lines.len() {
            let quoted = lines[index].trim_start();
            if !quoted.starts_with('>') {
                break;
            }
            body_lines.push(strip_callout_quote(quoted));
            index += 1;
        }
        let body = body_lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| escape_html(line.trim()))
            .collect::<Vec<_>>()
            .join("<br/>");
        output.push(format!(
            "<aside class=\"callout callout-{}\" data-callout=\"{}\"><strong>{}</strong><p>{}</p></aside>",
            escape_html(&callout_type),
            escape_html(&callout_type),
            escape_html(&title),
            body
        ));
    }
    output.join("\n")
}

fn strip_callout_quote(line: &str) -> String {
    line.strip_prefix('>')
        .map(str::trim_start)
        .unwrap_or(line)
        .to_string()
}

fn render_layout_tokens(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed == "{{page-break}}" {
                "<div class=\"page-break\" data-layout=\"page-break\"></div>".to_string()
            } else if let Some(rest) = trimmed.strip_prefix("{{section-break") {
                let attributes = rest.trim_end_matches("}}").trim();
                let style = layout_style_from_text(attributes);
                format!(
                    "<section class=\"section-break\" data-layout=\"section-break\" data-options=\"{}\"{}></section>",
                    escape_html(attributes),
                    style_attribute(&style)
                )
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_layout_block_html(body: &str) -> String {
    let style = layout_style_from_text(body);
    format!(
        "<section class=\"layout-directive\" data-layout=\"layout\" data-options=\"{}\"{}></section>",
        escape_html(body.trim()),
        style_attribute(&style)
    )
}

fn layout_style_from_text(text: &str) -> String {
    let mut styles = Vec::new();
    if let Some(columns) = layout_option(text, "columns") {
        if columns.chars().all(|ch| ch.is_ascii_digit()) && columns != "0" {
            styles.push(format!("column-count:{columns}"));
            styles.push("column-gap:32px".to_string());
        }
    }
    styles.join(";")
}

fn layout_option(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((candidate, value)) = line.split_once(':') {
            if candidate.trim() == key {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    for part in text.split_whitespace() {
        if let Some((candidate, value)) = part.split_once('=') {
            if candidate.trim() == key {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

fn style_attribute(style: &str) -> String {
    if style.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", escape_html(style))
    }
}

fn extract_label(text: &str) -> Option<String> {
    text.split("{#")
        .nth(1)
        .and_then(|rest| rest.split_once('}'))
        .map(|(label, _)| label.split_whitespace().next().unwrap_or("").to_string())
        .filter(|label| !label.is_empty())
}

fn extract_quoted_attribute(text: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=\"");
    let after_marker = text.split(&marker).nth(1)?;
    let (value, _) = after_marker.split_once('"')?;
    Some(value.to_string())
}

fn extract_headings(text: &str) -> Vec<Heading> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim_start();
            let level = trimmed.chars().take_while(|ch| *ch == '#').count();
            if (1..=6).contains(&level) && trimmed.chars().nth(level) == Some(' ') {
                let text = trimmed[level..].trim().to_string();
                Some(Heading {
                    level,
                    anchor: slugify(&text),
                    text,
                    line: index + 1,
                })
            } else {
                None
            }
        })
        .collect()
}

fn inject_generated_sections(
    text: &str,
    metadata: &Value,
    headings: &[Heading],
    index_entries: &[IndexEntry],
    bibliography: &[BibliographyEntry],
) -> String {
    let wants_toc = text.contains("[TOC]")
        || metadata
            .get("toc")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        || metadata
            .get("tableOfContents")
            .and_then(Value::as_bool)
            .unwrap_or(false);
    let mut output = text.to_string();
    if wants_toc {
        let toc = render_toc(
            headings,
            toc_depth(metadata),
            toc_numbering_enabled(metadata),
        );
        output = output.replace("[TOC]", &format!("## Table of Contents\n\n{toc}"));
        if !text.contains("[TOC]") {
            output = format!("## Table of Contents\n\n{toc}\n\n{output}");
        }
    }
    if output.contains("[INDEX]") {
        let index = render_index_entries(index_entries);
        output = output.replace("[INDEX]", &format!("## Index\n\n{index}"));
    }
    if output.contains("[BIBLIOGRAPHY]") {
        let references = bibliography
            .iter()
            .map(|entry| format!("- **{}**. {}", entry.key, entry.title))
            .collect::<Vec<_>>()
            .join("\n");
        output = output.replace(
            "[BIBLIOGRAPHY]",
            &format!("## Bibliography\n\n{references}"),
        );
    }
    output
}

fn render_index_entries(entries: &[IndexEntry]) -> String {
    if entries.is_empty() {
        return "_No index terms found._".to_string();
    }
    entries
        .iter()
        .map(|entry| {
            if let Some(anchor) = &entry.anchor {
                format!(
                    "- [{}](#{})",
                    escape_markdown_link_text(&entry.term),
                    anchor
                )
            } else {
                format!("- {}", entry.term)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_markdown_link_text(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}

fn render_toc(headings: &[Heading], depth: usize, numbered: bool) -> String {
    let mut counters = [0usize; 6];
    headings
        .iter()
        .filter(|heading| heading.level <= depth)
        .map(|heading| {
            let label = if numbered {
                let number = toc_number_for_heading(heading.level, &mut counters);
                format!("{number} {}", heading.text)
            } else {
                heading.text.clone()
            };
            format!(
                "{}- [{}](#{})",
                "  ".repeat(heading.level.saturating_sub(1)),
                label,
                heading.anchor
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn toc_number_for_heading(level: usize, counters: &mut [usize; 6]) -> String {
    let index = level.saturating_sub(1).min(5);
    counters[index] += 1;
    for counter in counters.iter_mut().skip(index + 1) {
        *counter = 0;
    }
    counters[..=index]
        .iter()
        .copied()
        .filter(|value| *value > 0)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn toc_depth(metadata: &Value) -> usize {
    metadata
        .get("tocDepth")
        .or_else(|| metadata.get("toc_depth"))
        .and_then(Value::as_u64)
        .map(|depth| depth.clamp(1, 6) as usize)
        .unwrap_or(6)
}

fn toc_numbering_enabled(metadata: &Value) -> bool {
    metadata
        .get("tocNumbered")
        .or_else(|| metadata.get("numberedHeadings"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn collect_bibliography(
    text: &str,
    metadata: &Value,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<BibliographyEntry> {
    let mut sources = collect_fence_bodies(text, "bibtex");
    if let Some(path) = metadata.get("bibliography").and_then(Value::as_str) {
        let base = root_path
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let bibliography_path = base.join(path);
        match fs::read_to_string(&bibliography_path) {
            Ok(contents) => sources.push(contents),
            Err(err) => diagnostics.push(diag(
                "error",
                format!(
                    "Missing bibliography file {}: {err}",
                    bibliography_path.display()
                ),
                Some(path_to_string(&bibliography_path)),
                None,
                Some("Create the bibliography file or update front matter."),
            )),
        }
    }

    sources
        .into_iter()
        .flat_map(|body| parse_bibliography_source(&body))
        .collect()
}

fn parse_bibliography_source(body: &str) -> Vec<BibliographyEntry> {
    if let Ok(entries) = parse_csl_json_bibliography(body) {
        return entries;
    }
    body.split('@')
        .filter_map(|entry| {
            let (kind_and_key, rest) = entry.split_once('{')?;
            let (key, raw) = rest.split_once(',')?;
            let title = raw
                .lines()
                .find(|line| line.trim_start().starts_with("title"))
                .and_then(|line| line.split_once('='))
                .map(|(_, value)| {
                    value
                        .trim()
                        .trim_matches(&['{', '}', ',', '"'][..])
                        .to_string()
                })
                .unwrap_or_else(|| kind_and_key.trim().to_string());
            Some(BibliographyEntry {
                key: key.trim().to_string(),
                title,
                raw: raw.to_string(),
            })
        })
        .collect()
}

fn duplicate_bibliography_keys(bibliography: &[BibliographyEntry]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut duplicates = BTreeSet::new();
    for entry in bibliography {
        if !seen.insert(entry.key.as_str()) {
            duplicates.insert(entry.key.clone());
        }
    }
    duplicates.into_iter().collect()
}

fn parse_csl_json_bibliography(body: &str) -> Result<Vec<BibliographyEntry>, serde_json::Error> {
    let value = serde_json::from_str::<Value>(body)?;
    let entries = value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let key = entry
                .get("id")
                .or_else(|| entry.get("citation-key"))
                .and_then(Value::as_str)?;
            let title = entry
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(key)
                .to_string();
            Some(BibliographyEntry {
                key: key.to_string(),
                title,
                raw: entry.to_string(),
            })
        })
        .collect();
    Ok(entries)
}

fn collect_glossary(text: &str) -> BTreeMap<String, String> {
    let mut glossary = BTreeMap::new();
    for body in collect_fence_bodies(text, "glossary") {
        for line in body.lines() {
            if let Some((term, definition)) = line.split_once(':') {
                glossary.insert(term.trim().to_string(), definition.trim().to_string());
            }
        }
    }
    glossary
}

fn collect_citation_references(text: &str) -> Vec<CitationReference> {
    let mut citations = Vec::new();
    for segment in text.split('[').skip(1) {
        if let Some((inside, _)) = segment.split_once(']') {
            if !inside.contains('@') {
                continue;
            }
            citations.extend(citation_references_from_bracket(inside));
        }
    }
    citations
}

fn citation_keys_from_references(references: &[CitationReference]) -> Vec<String> {
    let mut citations = BTreeSet::new();
    for reference in references {
        citations.insert(reference.key.clone());
    }
    citations.into_iter().collect()
}

fn citation_references_from_bracket(text: &str) -> Vec<CitationReference> {
    let mut references = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find('@') {
        let after_at = &rest[index + 1..];
        let key = after_at
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':'))
            .collect::<String>();
        let key_len = key.len();
        if !key.is_empty() {
            let after_key = &after_at[key_len..];
            let locator_end = after_key.find('@').unwrap_or(after_key.len());
            let locator = after_key[..locator_end]
                .trim()
                .trim_start_matches(|ch| ch == ',' || ch == ';')
                .trim()
                .trim_end_matches(';')
                .trim();
            references.push(CitationReference {
                key,
                locator: (!locator.is_empty()).then(|| locator.to_string()),
                raw: text.to_string(),
            });
        }
        rest = &after_at[key_len..];
    }
    references
}

fn render_citations(markdown: &str, bibliography: &[BibliographyEntry]) -> String {
    let titles = bibliography
        .iter()
        .map(|entry| (entry.key.as_str(), entry.title.as_str()))
        .collect::<HashMap<_, _>>();
    let mut output = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(start) = rest.find('[') {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find(']') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let inside = &after_start[..end];
        if inside.contains('@') {
            let references = citation_references_from_bracket(inside);
            output.push_str(&render_citation_span(&references, &titles));
        } else {
            output.push('[');
            output.push_str(inside);
            output.push(']');
        }
        rest = &after_start[end + 1..];
    }
    output.push_str(rest);
    output
}

fn render_citation_span(references: &[CitationReference], titles: &HashMap<&str, &str>) -> String {
    if references.is_empty() {
        return String::new();
    }
    let keys = references
        .iter()
        .map(|reference| reference.key.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let label = references
        .iter()
        .map(|reference| {
            let mut label = titles
                .get(reference.key.as_str())
                .copied()
                .unwrap_or(reference.key.as_str())
                .to_string();
            if let Some(locator) = &reference.locator {
                label.push_str(", ");
                label.push_str(locator);
            }
            label
        })
        .collect::<Vec<_>>()
        .join("; ");
    let details = references
        .iter()
        .map(|reference| {
            let title = titles
                .get(reference.key.as_str())
                .copied()
                .unwrap_or("missing bibliography entry");
            match &reference.locator {
                Some(locator) => format!("@{} ({locator}): {title}", reference.key),
                None => format!("@{}: {title}", reference.key),
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "<span class=\"citation\" tabindex=\"0\" title=\"{}\" aria-label=\"Citation: {}\" data-citation-keys=\"{}\">({})</span>",
        escape_html(&details),
        escape_html(&details),
        escape_html(&keys),
        escape_html(&label)
    )
}

fn collect_labels(text: &str) -> Vec<String> {
    let mut labels = BTreeSet::new();
    for segment in text.split("{#").skip(1) {
        if let Some((label, _)) = segment.split_once('}') {
            let label = label.split_whitespace().next().unwrap_or("").trim();
            if !label.is_empty() {
                labels.insert(label.to_string());
            }
        }
    }
    labels.into_iter().collect()
}

fn collect_cross_references(
    text: &str,
    labels: &[String],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<CrossReference> {
    let known = labels.iter().map(String::as_str).collect::<HashSet<_>>();
    let mut references = Vec::new();
    for segment in text.split("{@").skip(1) {
        if let Some((key, _)) = segment.split_once('}') {
            let key = key.trim().to_string();
            if key.is_empty() {
                continue;
            }
            let resolved = known.contains(key.as_str());
            if !resolved {
                diagnostics.push(diag(
                    "error",
                    format!("Broken cross reference: {key}"),
                    None,
                    None,
                    Some("Add a matching label such as {#fig:name}, {#tbl:name}, or {#eq:name}."),
                ));
            }
            references.push(CrossReference {
                target_kind: key
                    .split_once(':')
                    .map(|(kind, _)| kind.to_string())
                    .unwrap_or_else(|| "section".to_string()),
                key,
                resolved,
            });
        }
    }
    references
}

fn collect_index_entries(
    text: &str,
    metadata: &Value,
    headings: &[Heading],
    glossary: &BTreeMap<String, String>,
) -> Vec<IndexEntry> {
    let excluded = index_exclude_terms(metadata);
    let mut entries: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut proper_nouns: BTreeMap<String, (usize, Option<String>)> = BTreeMap::new();
    let mut heading_index = 0usize;
    let mut current_anchor = headings.first().map(|heading| heading.anchor.clone());
    let mut in_fence = false;

    for (zero_index, line) in text.lines().enumerate() {
        let line_number = zero_index + 1;
        while heading_index < headings.len() && headings[heading_index].line <= line_number {
            current_anchor = Some(headings[heading_index].anchor.clone());
            heading_index += 1;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        for term in explicit_index_terms(line) {
            insert_index_entry(&mut entries, &excluded, term, current_anchor.clone());
        }
        for term in bold_index_terms(line) {
            insert_index_entry(&mut entries, &excluded, term, current_anchor.clone());
        }
        for term in proper_noun_candidates(line) {
            if excluded.contains(&term) {
                continue;
            }
            let entry = proper_nouns
                .entry(term)
                .or_insert_with(|| (0, current_anchor.clone()));
            entry.0 += 1;
            if entry.1.is_none() {
                entry.1 = current_anchor.clone();
            }
        }
    }

    for heading in headings {
        insert_index_entry(
            &mut entries,
            &excluded,
            heading.text.clone(),
            Some(heading.anchor.clone()),
        );
    }
    for term in glossary.keys() {
        let anchor = first_term_anchor(text, headings, term).or_else(|| current_anchor.clone());
        insert_index_entry(&mut entries, &excluded, term.clone(), anchor);
    }
    for (term, (count, anchor)) in proper_nouns {
        if count >= 2 {
            insert_index_entry(&mut entries, &excluded, term, anchor);
        }
    }

    entries
        .into_iter()
        .map(|(term, anchor)| IndexEntry { term, anchor })
        .collect()
}

fn insert_index_entry(
    entries: &mut BTreeMap<String, Option<String>>,
    excluded: &BTreeSet<String>,
    term: String,
    anchor: Option<String>,
) {
    let normalized = term.trim().trim_matches('"').trim_matches('\'').to_string();
    if normalized.is_empty() || normalized.len() > 100 || excluded.contains(&normalized) {
        return;
    }
    entries
        .entry(normalized)
        .and_modify(|existing| {
            if existing.is_none() {
                *existing = anchor.clone();
            }
        })
        .or_insert(anchor);
}

fn index_exclude_terms(metadata: &Value) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    if let Some(values) = metadata.get("indexExclude").and_then(Value::as_array) {
        for value in values {
            if let Some(term) = value.as_str() {
                terms.insert(term.to_string());
            }
        }
    }
    if let Some(values) = metadata
        .get("index")
        .and_then(|index| index.get("exclude"))
        .and_then(Value::as_array)
    {
        for value in values {
            if let Some(term) = value.as_str() {
                terms.insert(term.to_string());
            }
        }
    }
    terms
}

fn explicit_index_terms(line: &str) -> Vec<String> {
    line.split("{#index:")
        .skip(1)
        .filter_map(|segment| {
            segment
                .split_once('}')
                .map(|(term, _)| term.trim().to_string())
        })
        .collect()
}

fn bold_index_terms(line: &str) -> Vec<String> {
    line.split("**")
        .skip(1)
        .step_by(2)
        .map(str::trim)
        .filter(|term| !term.is_empty() && term.len() <= 80)
        .map(ToString::to_string)
        .collect()
}

fn proper_noun_candidates(line: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut current = Vec::new();
    for raw in line.split_whitespace() {
        let token = raw
            .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-' && ch != '&')
            .trim();
        if token.len() > 2
            && token
                .chars()
                .next()
                .map(|ch| ch.is_uppercase())
                .unwrap_or(false)
            && !index_stop_word(token)
        {
            current.push(token.to_string());
        } else {
            push_proper_noun_candidate(&mut candidates, &mut current);
        }
    }
    push_proper_noun_candidate(&mut candidates, &mut current);
    candidates
}

fn push_proper_noun_candidate(candidates: &mut Vec<String>, current: &mut Vec<String>) {
    if current.is_empty() {
        return;
    }
    if current.len() == 1 || current.iter().map(String::len).sum::<usize>() <= 80 {
        candidates.push(current.join(" "));
    }
    current.clear();
}

fn index_stop_word(token: &str) -> bool {
    matches!(
        token,
        "The" | "This" | "That" | "These" | "Those" | "Prepared" | "Expected" | "Figure" | "Table"
    )
}

fn first_term_anchor(text: &str, headings: &[Heading], term: &str) -> Option<String> {
    let mut heading_index = 0usize;
    let mut current_anchor = headings.first().map(|heading| heading.anchor.clone());
    for (zero_index, line) in text.lines().enumerate() {
        let line_number = zero_index + 1;
        while heading_index < headings.len() && headings[heading_index].line <= line_number {
            current_anchor = Some(headings[heading_index].anchor.clone());
            heading_index += 1;
        }
        if line.contains(term) {
            return current_anchor;
        }
    }
    None
}

fn strip_index_markers(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{#index:") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + "{#index:".len()..];
        if let Some(end) = after_start.find('}') {
            rest = &after_start[end + 1..];
        } else {
            output.push_str("{#index:");
            output.push_str(after_start);
            return output;
        }
    }
    output.push_str(rest);
    output
}

fn collect_fence_bodies(text: &str, target: &str) -> Vec<String> {
    let mut bodies = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line
            .trim()
            .strip_prefix("```")
            .map(|info| info.split_whitespace().next().unwrap_or("") == target)
            .unwrap_or(false)
        {
            let mut body = String::new();
            for body_line in lines.by_ref() {
                if body_line.trim() == "```" {
                    break;
                }
                body.push_str(body_line);
                body.push('\n');
            }
            bodies.push(body);
        }
    }
    bodies
}

fn collect_comments(text: &str) -> Vec<ReviewComment> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let content = line
                .trim()
                .strip_prefix("<!-- comment:")?
                .strip_suffix("-->")?;
            Some(parse_review_comment(index + 1, content))
        })
        .collect()
}

fn parse_review_comment(line: usize, content: &str) -> ReviewComment {
    let mut author = "local".to_string();
    let mut created_at = String::new();
    let mut state = if content.contains("resolved") {
        "resolved"
    } else {
        "unresolved"
    }
    .to_string();
    let mut text_parts = Vec::new();

    for part in content
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        if part == "resolved" || part == "unresolved" {
            state = part.to_string();
        } else if let Some(value) = part
            .strip_prefix("author:")
            .or_else(|| part.strip_prefix("author="))
        {
            author = value.trim().to_string();
        } else if let Some(value) = part
            .strip_prefix("at:")
            .or_else(|| part.strip_prefix("at="))
            .or_else(|| part.strip_prefix("createdAt:"))
            .or_else(|| part.strip_prefix("createdAt="))
        {
            created_at = value.trim().to_string();
        } else {
            text_parts.push(part.to_string());
        }
    }

    ReviewComment {
        line,
        author,
        created_at,
        state,
        text: text_parts.join(" | "),
    }
}

fn collect_ai_sources(text: &str) -> Vec<AiSource> {
    collect_fence_bodies(text, "ai-source")
        .into_iter()
        .map(|body| {
            let map = body
                .lines()
                .filter_map(|line| line.split_once(':'))
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                .collect::<HashMap<_, _>>();
            AiSource {
                provider: map.get("provider").cloned().unwrap_or_default(),
                model: map.get("model").cloned().unwrap_or_default(),
                date: map.get("date").cloned().unwrap_or_default(),
                reviewed_by: map.get("reviewedBy").cloned().unwrap_or_default(),
                status: map
                    .get("status")
                    .cloned()
                    .unwrap_or_else(|| "unreviewed".to_string()),
            }
        })
        .collect()
}

fn validate_document(
    metadata: &Value,
    citations: &[String],
    bibliography: &[BibliographyEntry],
    duplicate_bibliography_keys: &[String],
    comments: &[ReviewComment],
    ai_sources: &[AiSource],
    has_bibliography_source: bool,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if metadata
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .is_empty()
    {
        diagnostics.push(diag(
            "warning",
            "Missing title metadata.",
            None,
            None,
            Some("Add title to YAML front matter."),
        ));
    }
    if metadata
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft")
        == "draft"
    {
        diagnostics.push(diag(
            "warning",
            "Document status is draft.",
            None,
            None,
            Some("Set status to approved or published before final export."),
        ));
    }
    if matches!(
        metadata.get("status").and_then(Value::as_str),
        Some("approved" | "published")
    ) && metadata
        .get("approvedBy")
        .and_then(Value::as_str)
        .unwrap_or("")
        .is_empty()
    {
        diagnostics.push(diag(
            "warning",
            "Approved or published document is missing approval metadata.",
            None,
            None,
            Some("Add approvedBy and approvedAt front matter for release auditability."),
        ));
    }
    let known_keys = bibliography
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<HashSet<_>>();
    for key in duplicate_bibliography_keys {
        diagnostics.push(diag(
            "error",
            format!("Duplicate bibliography key: {key}"),
            None,
            None,
            Some("Keep bibliography keys unique so citations resolve deterministically."),
        ));
    }
    if !citations.is_empty() && !has_bibliography_source {
        diagnostics.push(diag(
            "warning",
            "Document contains citations but no bibliography source.",
            None,
            None,
            Some("Add bibliography front matter, a bibtex fence, or a bibliography marker."),
        ));
    }
    for citation in citations {
        if !known_keys.is_empty() && !known_keys.contains(citation.as_str()) {
            diagnostics.push(diag(
                "error",
                format!("Broken citation: {citation}"),
                None,
                None,
                Some("Add the key to a BibTeX or CSL bibliography source."),
            ));
        }
    }
    if comments.iter().any(|comment| comment.state != "resolved") {
        diagnostics.push(diag(
            "warning",
            "Document has unresolved review comments.",
            None,
            None,
            Some("Resolve comments before publishing."),
        ));
    }
    if ai_sources
        .iter()
        .any(|source| source.status != "human-reviewed")
    {
        diagnostics.push(diag(
            "warning",
            "Document has AI-assisted sections that are not human-reviewed.",
            None,
            None,
            Some("Mark AI source blocks as human-reviewed after review."),
        ));
    }
}

fn render_delimited_table(
    body: &str,
    delimiter: char,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let rows = body
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_index, line)| {
            line.split(delimiter)
                .map(|cell| {
                    render_table_cell(cell.trim(), line_index + 1, artifact_diags, diagnostics)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return "<table></table>".to_string();
    }
    let mut html = String::from("<table class=\"transform-table\"><thead><tr>");
    for cell in &rows[0] {
        html.push_str(&format!("<th>{cell}</th>"));
    }
    html.push_str("</tr></thead><tbody>");
    for row in rows.iter().skip(1) {
        html.push_str("<tr>");
        for cell in row {
            html.push_str(&format!("<td>{cell}</td>"));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    html
}

fn render_structured_data_html(
    format: &str,
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let parsed = if format == "json" {
        serde_json::from_str::<Value>(body).map_err(|err| err.to_string())
    } else {
        serde_yaml::from_str::<Value>(body).map_err(|err| err.to_string())
    };
    match parsed {
        Ok(value) => {
            if let Some(table) = render_structured_table(format, &value) {
                table
            } else {
                format!(
                    "<section class=\"transform transform-{format} structured-tree\">{}</section>",
                    render_structured_tree("root", &value)
                )
            }
        }
        Err(error) => {
            let diagnostic = diag(
                "error",
                format!(
                    "Invalid {} transform input: {error}",
                    format.to_ascii_uppercase()
                ),
                None,
                None,
                Some("Check the structured data syntax."),
            );
            diagnostics.push(diagnostic.clone());
            artifact_diags.push(diagnostic);
            format!(
                "<pre class=\"transform transform-{format} transform-error\">{}</pre>",
                escape_html(body)
            )
        }
    }
}

fn render_structured_table(format: &str, value: &Value) -> Option<String> {
    let rows = value.as_array()?;
    if rows.is_empty() || !rows.iter().all(Value::is_object) {
        return None;
    }
    let headers = rows
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|object| object.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if headers.is_empty() {
        return None;
    }
    let mut html = format!("<table class=\"transform-table transform-{format}\"><thead><tr>");
    for header in &headers {
        html.push_str(&format!("<th>{}</th>", escape_html(header)));
    }
    html.push_str("</tr></thead><tbody>");
    for row in rows {
        let object = row.as_object()?;
        html.push_str("<tr>");
        for header in &headers {
            let cell = object
                .get(header)
                .map(structured_value_summary)
                .unwrap_or_default();
            html.push_str(&format!("<td>{}</td>", escape_html(&cell)));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    Some(html)
}

fn render_structured_tree(label: &str, value: &Value) -> String {
    match value {
        Value::Object(object) => {
            let mut html = format!(
                "<details open><summary>{}</summary><dl>",
                escape_html(label)
            );
            for (key, value) in object {
                html.push_str("<dt>");
                html.push_str(&escape_html(key));
                html.push_str("</dt><dd>");
                html.push_str(&render_structured_tree(key, value));
                html.push_str("</dd>");
            }
            html.push_str("</dl></details>");
            html
        }
        Value::Array(values) => {
            let mut html = format!(
                "<details open><summary>{} [{}]</summary><ol>",
                escape_html(label),
                values.len()
            );
            for value in values {
                html.push_str("<li>");
                html.push_str(&render_structured_tree("item", value));
                html.push_str("</li>");
            }
            html.push_str("</ol></details>");
            html
        }
        _ => escape_html(&structured_value_summary(value)),
    }
}

fn structured_value_summary(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => format!("[{} items]", values.len()),
        Value::Object(object) => format!("{{{} fields}}", object.len()),
    }
}

fn render_table_cell(
    cell: &str,
    line: usize,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let Some(expression) = cell.strip_prefix('=') else {
        return escape_html(cell);
    };
    match evaluate_table_formula(expression) {
        Ok(value) => escape_html(&value),
        Err(error) => {
            let diagnostic = diag(
                "error",
                format!("Table formula error on row {line}: {error}"),
                None,
                Some(line),
                Some("Use numeric formulas such as =SUM(1,2) in CSV/TSV cells."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            "#ERROR".to_string()
        }
    }
}

fn evaluate_markdown_table_formulas(
    markdown: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    markdown
        .lines()
        .enumerate()
        .map(|(line_index, line)| {
            let trimmed = line.trim();
            if !is_markdown_table_row(trimmed) || is_markdown_table_separator(trimmed) {
                return line.to_string();
            }

            let mut changed = false;
            let cells = split_table_row(trimmed)
                .into_iter()
                .map(|cell| {
                    let Some(expression) = cell.strip_prefix('=') else {
                        return cell;
                    };
                    changed = true;
                    match evaluate_table_formula(expression) {
                        Ok(value) => value,
                        Err(error) => {
                            diagnostics.push(diag(
                                "error",
                                format!(
                                    "Markdown table formula error on row {}: {error}",
                                    line_index + 1
                                ),
                                None,
                                Some(line_index + 1),
                                Some("Use numeric formulas such as =10+15 or =SUM(1,2)."),
                            ));
                            "#ERROR".to_string()
                        }
                    }
                })
                .collect::<Vec<_>>();

            if changed {
                format!("| {} |", cells.join(" | "))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn evaluate_table_formula(expression: &str) -> Result<String, String> {
    eval_expression(expression, &HashMap::new()).map(|value| format_value(value, "round"))
}

fn render_glossary_html(body: &str) -> String {
    let mut html = String::from("<dl class=\"glossary\">");
    for line in body.lines() {
        if let Some((term, definition)) = line.split_once(':') {
            html.push_str(&format!(
                "<dt>{}</dt><dd>{}</dd>",
                escape_html(term.trim()),
                escape_html(definition.trim())
            ));
        }
    }
    html.push_str("</dl>");
    html
}

fn render_bibtex_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let entries = parse_bibliography_source(body);
    if entries.is_empty() {
        let diagnostic = diag(
            "warning",
            "BibTeX transform did not contain any bibliography entries.",
            None,
            None,
            Some("Add BibTeX entries such as @book{key, title={Title}}."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-bibtex transform-error\">No bibliography entries found.</section>".to_string();
    }
    let mut html = String::from("<dl class=\"transform transform-bibtex\">");
    for entry in entries {
        html.push_str(&format!(
            "<dt>{}</dt><dd>{}</dd>",
            escape_html(&entry.key),
            escape_html(&entry.title)
        ));
    }
    html.push_str("</dl>");
    html
}

fn render_timeline_svg(body: &str) -> String {
    let items = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let height = 80 + items.len() * 54;
    let mut svg = format!("<svg class=\"timeline\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 {height}\" role=\"img\"><line x1=\"120\" y1=\"40\" x2=\"120\" y2=\"{}\" stroke=\"#275DA8\" stroke-width=\"3\"/>", height - 30);
    for (index, item) in items.iter().enumerate() {
        let y = 50 + index * 54;
        svg.push_str(&format!("<circle cx=\"120\" cy=\"{y}\" r=\"8\" fill=\"#275DA8\"/><text x=\"150\" y=\"{}\" font-size=\"18\" fill=\"#1f2937\">{}</text>", y + 6, escape_html(item)));
    }
    svg.push_str("</svg>");
    svg
}

fn render_roadmap_html(body: &str) -> String {
    let items = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (stage, text) = line
                .split_once(':')
                .or_else(|| line.split_once('-'))
                .map(|(stage, text)| (stage.trim(), text.trim()))
                .unwrap_or(("Item", line));
            format!(
                "<article><strong>{}</strong><p>{}</p></article>",
                escape_html(stage),
                escape_html(text)
            )
        })
        .collect::<String>();
    format!(
        "<section class=\"transform transform-roadmap\"><h3>Roadmap</h3><div>{items}</div></section>"
    )
}

fn render_adr_html(body: &str) -> String {
    let rows = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (key, value) = line
                .split_once(':')
                .map(|(key, value)| (key.trim(), value.trim()))
                .unwrap_or(("Note", line));
            format!(
                "<tr><th>{}</th><td>{}</td></tr>",
                escape_html(key),
                escape_html(value)
            )
        })
        .collect::<String>();
    format!(
        "<section class=\"transform transform-adr\"><h3>Architecture Decision Record</h3><table><tbody>{rows}</tbody></table></section>"
    )
}

fn render_diff_html(body: &str) -> String {
    let lines = body
        .lines()
        .map(|line| {
            let class = if line.starts_with('+') && !line.starts_with("+++") {
                "add"
            } else if line.starts_with('-') && !line.starts_with("---") {
                "del"
            } else if line.starts_with("@@") {
                "hunk"
            } else {
                "ctx"
            };
            format!("<code class=\"diff-{class}\">{}</code>", escape_html(line))
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<pre class=\"transform transform-diff\">{lines}</pre>")
}

fn render_chart_svg(body: &str) -> String {
    let chart = parse_chart_spec(body);
    let values = chart.values;
    let title = chart.title.unwrap_or_else(|| "Chart".to_string());
    let chart_type = chart.chart_type.unwrap_or_else(|| "bar".to_string());
    let max = values
        .iter()
        .map(|(_, value)| *value)
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(1.0);
    let height = 300;
    let width = 760;
    let mut svg = format!(
        "<svg class=\"chart\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\"><title>{}</title><text x=\"80\" y=\"32\" font-size=\"18\" font-weight=\"700\" fill=\"#1f2937\">{}</text>",
        escape_html(&title),
        escape_html(&title)
    );
    if chart_type.eq_ignore_ascii_case("line") || chart_type.eq_ignore_ascii_case("area") {
        svg.push_str(&render_line_chart_svg(&values, max));
    } else {
        svg.push_str(&render_bar_chart_svg(&values, max));
    }
    svg.push_str("</svg>");
    svg
}

#[derive(Debug)]
struct ChartSpec {
    title: Option<String>,
    chart_type: Option<String>,
    values: Vec<(String, f64)>,
}

fn parse_chart_spec(body: &str) -> ChartSpec {
    if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(body) {
        let title = yaml_get(&value, "title").and_then(yaml_scalar_string);
        let chart_type = yaml_get(&value, "type").and_then(yaml_scalar_string);
        let x_key = yaml_get(&value, "x")
            .and_then(yaml_scalar_string)
            .unwrap_or_else(|| "label".to_string());
        let y_key = yaml_get(&value, "y")
            .and_then(yaml_scalar_string)
            .unwrap_or_else(|| "value".to_string());
        let values = yaml_get(&value, "data")
            .and_then(serde_yaml::Value::as_sequence)
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| {
                        let label = yaml_get(row, &x_key).and_then(yaml_scalar_string)?;
                        let value = yaml_get(row, &y_key).and_then(yaml_number)?;
                        Some((label, value))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !values.is_empty() {
            return ChartSpec {
                title,
                chart_type,
                values,
            };
        }
    }

    ChartSpec {
        title: None,
        chart_type: Some("bar".to_string()),
        values: body
            .lines()
            .filter_map(|line| line.split_once(':'))
            .filter_map(|(label, value)| {
                value
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .map(|value| (label.trim().to_string(), value))
            })
            .collect(),
    }
}

fn yaml_get<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    match value {
        serde_yaml::Value::Mapping(map) => {
            let key = serde_yaml::Value::String(key.to_string());
            map.get(&key)
        }
        _ => None,
    }
}

fn yaml_scalar_string(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(text) => Some(text.clone()),
        serde_yaml::Value::Number(number) => Some(number.to_string()),
        serde_yaml::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn yaml_number(value: &serde_yaml::Value) -> Option<f64> {
    match value {
        serde_yaml::Value::Number(number) => number.as_f64(),
        serde_yaml::Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
}

fn render_bar_chart_svg(values: &[(String, f64)], max: f64) -> String {
    let bar_width = if values.is_empty() {
        1
    } else {
        600 / values.len().max(1)
    };
    let mut svg =
        String::from("<line x1=\"70\" y1=\"240\" x2=\"710\" y2=\"240\" stroke=\"#94a3b8\"/>");
    for (index, (label, value)) in values.iter().enumerate() {
        let bar_height = ((*value / max) * 170.0) as usize;
        let x = 80 + index * bar_width;
        let y = 240 - bar_height;
        svg.push_str(&format!(
            "<rect x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{bar_height}\" fill=\"#275DA8\"/><text x=\"{x}\" y=\"264\" font-size=\"12\">{}</text><text x=\"{x}\" y=\"{}\" font-size=\"12\">{value}</text>",
            bar_width.saturating_sub(10),
            escape_html(label),
            y.saturating_sub(8)
        ));
    }
    svg
}

fn render_line_chart_svg(values: &[(String, f64)], max: f64) -> String {
    let step = if values.len() <= 1 {
        1.0
    } else {
        600.0 / (values.len() - 1) as f64
    };
    let points = values
        .iter()
        .enumerate()
        .map(|(index, (_, value))| {
            let x = 80.0 + index as f64 * step;
            let y = 240.0 - ((*value / max) * 170.0);
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut svg = format!(
        "<line x1=\"70\" y1=\"240\" x2=\"710\" y2=\"240\" stroke=\"#94a3b8\"/><polyline fill=\"none\" stroke=\"#275DA8\" stroke-width=\"3\" points=\"{points}\"/>"
    );
    for (index, (label, value)) in values.iter().enumerate() {
        let x = 80.0 + index as f64 * step;
        let y = 240.0 - ((*value / max) * 170.0);
        svg.push_str(&format!(
            "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"#275DA8\"/><text x=\"{x:.1}\" y=\"264\" font-size=\"12\">{}</text><text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"12\">{value}</text>",
            escape_html(label),
            y - 10.0
        ));
    }
    svg
}

fn render_vega_lite_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let spec = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid Vega-Lite JSON: {err}"),
                None,
                None,
                Some("Provide a JSON Vega-Lite spec with data.values and x/y encodings."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-vega-lite transform-error\">Invalid Vega-Lite JSON</section>".to_string();
        }
    };
    let mark = vega_lite_mark(&spec);
    if !matches!(mark.as_str(), "bar" | "line" | "point") {
        let diagnostic = diag(
            "warning",
            format!("Unsupported Vega-Lite mark for native preview: {mark}"),
            None,
            None,
            Some("Use bar, line, or point marks for the native static preview."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">Unsupported Vega-Lite mark</section>".to_string();
    }
    let Some(x_field) = vega_lite_encoding_field(&spec, "x") else {
        return vega_lite_missing_field("x", artifact_diags, diagnostics);
    };
    let Some(y_field) = vega_lite_encoding_field(&spec, "y") else {
        return vega_lite_missing_field("y", artifact_diags, diagnostics);
    };
    let values = vega_lite_values(&spec, &x_field, &y_field);
    if values.is_empty() {
        let diagnostic = diag(
            "warning",
            "Vega-Lite native preview did not find numeric data.values rows.",
            None,
            None,
            Some("Use inline data.values with a numeric y encoding."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">No drawable Vega-Lite rows</section>".to_string();
    }
    let title = spec
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Vega-Lite chart");
    render_vega_lite_chart_svg(title, &mark, &values)
}

fn vega_lite_mark(spec: &Value) -> String {
    spec.get("mark")
        .and_then(|mark| {
            mark.as_str().map(ToString::to_string).or_else(|| {
                mark.get("type")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
        })
        .unwrap_or_else(|| "bar".to_string())
}

fn vega_lite_encoding_field(spec: &Value, channel: &str) -> Option<String> {
    spec.pointer(&format!("/encoding/{channel}/field"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn vega_lite_values(spec: &Value, x_field: &str, y_field: &str) -> Vec<(String, f64)> {
    spec.pointer("/data/values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let x = row.get(x_field).map(value_to_axis_label)?;
            let y = row
                .get(y_field)
                .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))?;
            Some((x, y))
        })
        .collect()
}

fn value_to_axis_label(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value_to_string(value))
}

fn vega_lite_missing_field(
    channel: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let diagnostic = diag(
        "warning",
        format!("Vega-Lite native preview is missing {channel} field encoding."),
        None,
        None,
        Some("Set encoding.x.field and encoding.y.field."),
    );
    artifact_diags.push(diagnostic.clone());
    diagnostics.push(diagnostic);
    format!(
        "<section class=\"transform transform-vega-lite transform-error\">Missing {channel} encoding</section>"
    )
}

fn render_vega_lite_chart_svg(title: &str, mark: &str, values: &[(String, f64)]) -> String {
    let max = values
        .iter()
        .map(|(_, value)| *value)
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(1.0);
    let width = 820usize;
    let height = 320usize;
    let plot_left = 72usize;
    let plot_bottom = 262usize;
    let plot_width = 680usize;
    let step = plot_width / values.len().max(1);
    let mut svg = format!(
        "<svg class=\"transform transform-vega-lite\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\"><text x=\"72\" y=\"34\" font-size=\"18\" fill=\"#111827\">{}</text><line x1=\"72\" y1=\"262\" x2=\"770\" y2=\"262\" stroke=\"#94a3b8\"/><line x1=\"72\" y1=\"54\" x2=\"72\" y2=\"262\" stroke=\"#94a3b8\"/>",
        escape_html(title)
    );
    let points = values
        .iter()
        .enumerate()
        .map(|(index, (_, value))| {
            let x = plot_left + index * step + step / 2;
            let y = plot_bottom - ((*value / max) * 190.0) as usize;
            (x, y)
        })
        .collect::<Vec<_>>();
    if mark == "bar" {
        for (index, (label, value)) in values.iter().enumerate() {
            let bar_height = ((*value / max) * 190.0) as usize;
            let x = plot_left + index * step + 8;
            let y = plot_bottom - bar_height;
            svg.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{bar_height}\" fill=\"#275DA8\"/><text x=\"{x}\" y=\"286\" font-size=\"12\">{}</text>",
                step.saturating_sub(16),
                escape_html(label)
            ));
        }
    } else {
        if mark == "line" {
            let polyline = points
                .iter()
                .map(|(x, y)| format!("{x},{y}"))
                .collect::<Vec<_>>()
                .join(" ");
            svg.push_str(&format!(
                "<polyline points=\"{polyline}\" fill=\"none\" stroke=\"#275DA8\" stroke-width=\"3\"/>"
            ));
        }
        for ((x, y), (label, _)) in points.iter().zip(values.iter()) {
            svg.push_str(&format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"5\" fill=\"#275DA8\"/><text x=\"{}\" y=\"286\" font-size=\"12\">{}</text>",
                x.saturating_sub(12),
                escape_html(label)
            ));
        }
    }
    svg.push_str("</svg>");
    svg
}

fn render_mermaid_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let graph = parse_mermaid_flowchart(body);
    if graph.nodes.is_empty() || graph.edges.is_empty() {
        let diagnostic = diag(
            "warning",
            "Mermaid native preview only supports simple flowchart edges.",
            None,
            None,
            Some("Use flowchart or graph syntax with edges such as A[Start] --> B[End]."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-mermaid transform-error\">Unsupported Mermaid diagram</section>".to_string();
    }
    let columns = 3usize;
    let node_width = 170usize;
    let node_height = 54usize;
    let x_gap = 250usize;
    let y_gap = 120usize;
    let rows = graph.nodes.len().div_ceil(columns);
    let width = 120 + columns * x_gap;
    let height = 90 + rows * y_gap;
    let positions = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let x = 60 + (index % columns) * x_gap;
            let y = 55 + (index / columns) * y_gap;
            (node.id.clone(), (x, y))
        })
        .collect::<HashMap<_, _>>();
    let mut svg = format!(
        "<svg class=\"transform transform-mermaid\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\"><defs><marker id=\"arrow\" markerWidth=\"10\" markerHeight=\"10\" refX=\"8\" refY=\"3\" orient=\"auto\" markerUnits=\"strokeWidth\"><path d=\"M0,0 L0,6 L9,3 z\" fill=\"#275DA8\"/></marker></defs>"
    );
    for edge in &graph.edges {
        if let (Some((from_x, from_y)), Some((to_x, to_y))) =
            (positions.get(&edge.from), positions.get(&edge.to))
        {
            let x1 = from_x + node_width;
            let y1 = from_y + node_height / 2;
            let x2 = *to_x;
            let y2 = to_y + node_height / 2;
            svg.push_str(&format!(
                "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"#275DA8\" stroke-width=\"3\" marker-end=\"url(#arrow)\"/>"
            ));
        }
    }
    for node in &graph.nodes {
        if let Some((x, y)) = positions.get(&node.id) {
            svg.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{node_width}\" height=\"{node_height}\" rx=\"8\" fill=\"#eff6ff\" stroke=\"#275DA8\" stroke-width=\"2\"/><text x=\"{}\" y=\"{}\" font-size=\"15\" text-anchor=\"middle\" fill=\"#1f2937\">{}</text>",
                x + node_width / 2,
                y + 33,
                escape_html(&node.label)
            ));
        }
    }
    svg.push_str("</svg>");
    svg
}

#[derive(Debug)]
struct MermaidGraph {
    nodes: Vec<MermaidNode>,
    edges: Vec<MermaidEdge>,
}

#[derive(Debug)]
struct MermaidNode {
    id: String,
    label: String,
}

#[derive(Debug)]
struct MermaidEdge {
    from: String,
    to: String,
}

fn parse_mermaid_flowchart(body: &str) -> MermaidGraph {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();
    let mut edges = Vec::new();
    for line in body.lines() {
        let line = line.trim().trim_end_matches(';').trim();
        if line.is_empty()
            || line.starts_with("%%")
            || line.starts_with("graph ")
            || line.starts_with("flowchart ")
        {
            continue;
        }
        let Some((left, right)) = split_mermaid_edge(line) else {
            continue;
        };
        let from = parse_mermaid_node(left);
        let to = parse_mermaid_node(strip_mermaid_edge_label(right));
        add_mermaid_node(&mut nodes, &mut seen, &from);
        add_mermaid_node(&mut nodes, &mut seen, &to);
        edges.push(MermaidEdge {
            from: from.id,
            to: to.id,
        });
    }
    MermaidGraph { nodes, edges }
}

fn split_mermaid_edge(line: &str) -> Option<(&str, &str)> {
    for operator in ["-->", "==>", "-.->", "---"] {
        if let Some((left, right)) = line.split_once(operator) {
            return Some((left.trim(), right.trim()));
        }
    }
    None
}

fn strip_mermaid_edge_label(text: &str) -> &str {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix('|') {
        if let Some((_, after_label)) = rest.split_once('|') {
            return after_label.trim();
        }
    }
    text
}

fn parse_mermaid_node(text: &str) -> MermaidNode {
    let text = text.trim();
    for (open, close) in [('[', ']'), ('(', ')'), ('{', '}')] {
        if let Some(start) = text.find(open) {
            if let Some(end) = text.rfind(close) {
                let id = text[..start].trim();
                let label = text[start + 1..end].trim().trim_matches('"');
                return MermaidNode {
                    id: id.to_string(),
                    label: label.to_string(),
                };
            }
        }
    }
    let id = text
        .split_whitespace()
        .next()
        .unwrap_or(text)
        .trim_matches('"')
        .to_string();
    MermaidNode {
        label: id.clone(),
        id,
    }
}

fn add_mermaid_node(nodes: &mut Vec<MermaidNode>, seen: &mut HashSet<String>, node: &MermaidNode) {
    if seen.insert(node.id.clone()) {
        nodes.push(MermaidNode {
            id: node.id.clone(),
            label: node.label.clone(),
        });
    }
}

fn render_geojson_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid GeoJSON document: {err}"),
                None,
                None,
                Some("Provide valid GeoJSON Feature, FeatureCollection, or Geometry JSON."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-geojson transform-error\">Invalid GeoJSON document</section>".to_string();
        }
    };
    let mut positions = Vec::new();
    collect_geojson_positions(&value, &mut positions);
    if positions.is_empty() {
        let diagnostic = diag(
            "warning",
            "GeoJSON transform did not contain drawable coordinates.",
            None,
            None,
            Some("Add Point, LineString, Polygon, or Multi* coordinates."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-geojson transform-error\">No GeoJSON coordinates found</section>".to_string();
    }
    let positions = positions.into_iter().take(2000).collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let points = positions
        .iter()
        .map(|position| {
            let (x, y) = project_geojson_position(*position, min_x, max_x, min_y, max_y);
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>();
    let markers = points
        .iter()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap_or(("0", "0"));
            format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"3\" fill=\"#0f766e\"/>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<svg class=\"transform transform-geojson\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"#ecfeff\" stroke=\"#67e8f9\"/><polyline points=\"{}\" fill=\"none\" stroke=\"#275DA8\" stroke-width=\"3\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>{markers}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"#134e4a\">{} coordinates</text></svg>",
        points.join(" "),
        positions.len()
    )
}

fn render_topojson_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid TopoJSON document: {err}"),
                None,
                None,
                Some("Provide valid TopoJSON with an arcs array."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-topojson transform-error\">Invalid TopoJSON document</section>".to_string();
        }
    };
    let arcs = decode_topojson_arcs(&value);
    if arcs.is_empty() {
        let diagnostic = diag(
            "warning",
            "TopoJSON transform did not contain drawable arcs.",
            None,
            None,
            Some("Add a Topology arcs array or verify the TopoJSON source."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-topojson transform-error\">No TopoJSON arcs found</section>".to_string();
    }
    let positions = arcs
        .iter()
        .flatten()
        .copied()
        .take(4000)
        .collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let polylines = arcs
        .iter()
        .map(|arc| {
            let points = arc
                .iter()
                .map(|position| {
                    let (x, y) = project_geojson_position(*position, min_x, max_x, min_y, max_y);
                    format!("{x:.2},{y:.2}")
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!(
                "<polyline points=\"{points}\" fill=\"none\" stroke=\"#275DA8\" stroke-width=\"3\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<svg class=\"transform transform-topojson\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"#f8fafc\" stroke=\"#94a3b8\"/>{polylines}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"#334155\">{} arcs</text></svg>",
        arcs.len()
    )
}

fn decode_topojson_arcs(value: &Value) -> Vec<Vec<(f64, f64)>> {
    let scale = value
        .pointer("/transform/scale")
        .and_then(Value::as_array)
        .and_then(|items| Some((items.first()?.as_f64()?, items.get(1)?.as_f64()?)))
        .unwrap_or((1.0, 1.0));
    let translate = value
        .pointer("/transform/translate")
        .and_then(Value::as_array)
        .and_then(|items| Some((items.first()?.as_f64()?, items.get(1)?.as_f64()?)))
        .unwrap_or((0.0, 0.0));
    value
        .get("arcs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|arc| decode_topojson_arc(arc, scale, translate))
        .collect()
}

fn decode_topojson_arc(
    arc: &Value,
    (scale_x, scale_y): (f64, f64),
    (translate_x, translate_y): (f64, f64),
) -> Option<Vec<(f64, f64)>> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut positions = Vec::new();
    for point in arc.as_array()? {
        let coordinates = point.as_array()?;
        x += coordinates.first()?.as_f64()?;
        y += coordinates.get(1)?.as_f64()?;
        positions.push((x * scale_x + translate_x, y * scale_y + translate_y));
    }
    (!positions.is_empty()).then_some(positions)
}

fn render_stl_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let vertices = parse_ascii_stl_vertices(body);
    if vertices.is_empty() {
        let diagnostic = diag(
            "warning",
            "STL transform did not contain ASCII vertex data.",
            None,
            None,
            Some("Use ASCII STL fences for static previews, or configure an external STL renderer later."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-stl transform-error\">No ASCII STL vertices found</section>".to_string();
    }
    let positions = vertices
        .iter()
        .map(|(x, y, _)| (*x, *y))
        .collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let triangles = vertices
        .chunks(3)
        .filter(|triangle| triangle.len() == 3)
        .map(|triangle| {
            let points = triangle
                .iter()
                .map(|(x, y, _)| {
                    let (x, y) = project_geojson_position((*x, *y), min_x, max_x, min_y, max_y);
                    format!("{x:.2},{y:.2}")
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!("<polygon points=\"{points}\" fill=\"rgba(39,93,168,.18)\" stroke=\"#275DA8\" stroke-width=\"2\"/>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<svg class=\"transform transform-stl\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"#f8fafc\" stroke=\"#cbd5e1\"/>{triangles}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"#334155\">{} triangles / {} vertices</text></svg>",
        vertices.len() / 3,
        vertices.len()
    )
}

fn parse_ascii_stl_vertices(body: &str) -> Vec<(f64, f64, f64)> {
    body.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "vertex" {
                return None;
            }
            Some((
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
            ))
        })
        .collect()
}

fn collect_geojson_positions(value: &Value, positions: &mut Vec<(f64, f64)>) {
    match value {
        Value::Array(items) => {
            if items.len() >= 2 {
                if let (Some(x), Some(y)) = (items[0].as_f64(), items[1].as_f64()) {
                    positions.push((x, y));
                    return;
                }
            }
            for item in items {
                collect_geojson_positions(item, positions);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                collect_geojson_positions(value, positions);
            }
        }
        _ => {}
    }
}

fn geojson_bounds(positions: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    positions.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), (x, y)| {
            (min_x.min(*x), max_x.max(*x), min_y.min(*y), max_y.max(*y))
        },
    )
}

fn project_geojson_position(
    (x, y): (f64, f64),
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> (f64, f64) {
    let width = (max_x - min_x).abs().max(0.000_001);
    let height = (max_y - min_y).abs().max(0.000_001);
    let projected_x = 48.0 + ((x - min_x) / width) * 804.0;
    let projected_y = 412.0 - ((y - min_y) / height) * 364.0;
    (projected_x, projected_y)
}

fn render_openapi_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match parse_json_or_yaml(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid OpenAPI document: {err}"),
                None,
                None,
                Some("Provide valid JSON or YAML OpenAPI content."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-error\">Invalid OpenAPI document</section>"
                .to_string();
        }
    };
    let mut html = String::from(
        "<table class=\"transform-table openapi\"><thead><tr><th>Method</th><th>Path</th><th>Summary</th></tr></thead><tbody>",
    );
    if let Some(paths) = value.get("paths").and_then(Value::as_object) {
        for (path, methods) in paths {
            if let Some(methods) = methods.as_object() {
                for (method, operation) in methods {
                    let summary = operation
                        .get("summary")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                        escape_html(&method.to_ascii_uppercase()),
                        escape_html(path),
                        escape_html(summary)
                    ));
                }
            }
        }
    }
    html.push_str("</tbody></table>");
    html
}

fn render_json_schema_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match parse_json_or_yaml(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid JSON Schema document: {err}"),
                None,
                None,
                Some("Provide valid JSON or YAML JSON Schema content."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-error\">Invalid JSON Schema document</section>"
                .to_string();
        }
    };
    let required = value
        .get("required")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let mut html = String::from(
        "<table class=\"transform-table json-schema\"><thead><tr><th>Field</th><th>Type</th><th>Required</th><th>Description</th></tr></thead><tbody>",
    );
    if let Some(properties) = value.get("properties").and_then(Value::as_object) {
        for (field, schema) in properties {
            let kind = schema.get("type").and_then(Value::as_str).unwrap_or("");
            let description = schema
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape_html(field),
                escape_html(kind),
                if required.contains(field.as_str()) {
                    "yes"
                } else {
                    "no"
                },
                escape_html(description)
            ));
        }
    }
    html.push_str("</tbody></table>");
    html
}

fn parse_json_or_yaml(body: &str) -> Result<Value, String> {
    serde_json::from_str::<Value>(body)
        .or_else(|_| serde_yaml::from_str::<Value>(body))
        .map_err(|err| err.to_string())
}

fn run_git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn execute_external_transform(
    name: &str,
    body: &str,
    engine_path: &Path,
    input_mode: &str,
    timeout_ms: u64,
) -> Result<TransformArtifact, String> {
    let source_hash = sha256_hex(body.as_bytes());
    let started = Instant::now();
    let mut diagnostics = Vec::new();
    let mut temp_input = None;
    let mut command = Command::new(engine_path);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    if input_mode == "file" {
        let path = env::temp_dir().join(format!("neditor-{name}-{source_hash}.input"));
        fs::write(&path, body.as_bytes()).map_err(|err| err.to_string())?;
        command.arg(&path);
        temp_input = Some(path);
    } else {
        command.stdin(Stdio::piped());
    }

    let mut child = command.spawn().map_err(|err| err.to_string())?;
    if input_mode == "stdin" {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(body.as_bytes())
                .map_err(|err| err.to_string())?;
        }
    }

    let status = loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            break status;
        }
        if started.elapsed() >= Duration::from_millis(timeout_ms) {
            let _ = child.kill();
            let _ = child.wait();
            if let Some(path) = temp_input {
                let _ = fs::remove_file(path);
            }
            return Err(format!(
                "{name} external transform timed out after {timeout_ms}ms."
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    let output = child.wait_with_output().map_err(|err| err.to_string())?;
    if let Some(path) = temp_input {
        let _ = fs::remove_file(path);
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
        return Err(format!(
            "{name} external transform exited with status {}.",
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".to_string())
        ));
    }

    let output_text = String::from_utf8_lossy(&output.stdout).to_string();
    let html = if output_text.trim_start().starts_with('<') {
        output_text
    } else {
        format!(
            "<pre class=\"transform transform-external\">{}</pre>",
            escape_html(&output_text)
        )
    };
    let duration_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    diagnostics.push(diag(
        "info",
        format!("{name} external transform completed in {duration_ms}ms."),
        None,
        None,
        Some("Output was captured without invoking a shell."),
    ));

    Ok(TransformArtifact {
        id: format!("{name}-{source_hash}"),
        name: name.to_string(),
        output_kind: if html.contains("<svg") { "svg" } else { "html" }.to_string(),
        cache_key: transform_cache_key(
            name,
            input_mode,
            &path_to_string(engine_path),
            &source_hash,
        ),
        execution_kind: "external".to_string(),
        engine_path: Some(path_to_string(engine_path)),
        input_mode: input_mode.to_string(),
        duration_ms: Some(duration_ms),
        source_hash,
        html,
        diagnostics,
    })
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
    json!({
        "name": name,
        "execution": execution,
        "safeByDefault": safe_by_default,
        "requiresNetwork": false,
        "requiresExecution": requires_execution,
        "trustRequired": requires_execution,
        "preferenceKey": format!("transforms.{name}.path"),
        "defaultCommand": name,
        "inputModes": input_modes,
        "limits": {
            "timeoutMs": DEFAULT_TRANSFORM_TIMEOUT_MS,
            "maxTimeoutMs": MAX_TRANSFORM_TIMEOUT_MS,
            "maxInputBytes": MAX_TRANSFORM_INPUT_BYTES
        },
        "cacheScope": "name+enginePath+inputMode+sourceHash",
        "exportTargets": ["html", "pdf", "docx", "pptx"]
    })
}

fn transform_cache_key(
    name: &str,
    input_mode: &str,
    engine_path: &str,
    source_hash: &str,
) -> String {
    sha256_hex(format!("{name}:{input_mode}:{engine_path}:{source_hash}").as_bytes())
}

fn metadata_lookup<'a>(metadata: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = metadata;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn metadata_string(metadata: &Value, path: &str) -> Option<String> {
    metadata_lookup(metadata, path).map(value_to_string)
}

fn render_export_template(
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

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn format_value(value: f64, filter: &str) -> String {
    match filter {
        "percent" => format!("{:.2}%", value * 100.0),
        "currency" => format!("${value:.2}"),
        "round" => format!("{value:.0}"),
        _ => value.to_string(),
    }
}

fn expression_dependencies(expression: &str) -> Vec<String> {
    expression
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_alphabetic())
        })
        .filter(|part| {
            !matches!(
                part.to_ascii_uppercase().as_str(),
                "SUM"
                    | "AVG"
                    | "MIN"
                    | "MAX"
                    | "COUNT"
                    | "ROUND"
                    | "ABS"
                    | "IF"
                    | "PERCENT"
                    | "CURRENCY"
            )
        })
        .map(ToString::to_string)
        .collect()
}

fn collect_table_summaries(text: &str) -> Vec<TableSummary> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut tables = Vec::new();
    let mut index = 0;
    while index + 1 < lines.len() {
        let header = lines[index].trim();
        let separator = lines[index + 1].trim();
        if is_markdown_table_row(header) && is_markdown_table_separator(separator) {
            let columns = split_table_row(header);
            let mut row_count = 0usize;
            let mut numeric_columns = columns
                .iter()
                .map(|column| (column.clone(), 0.0))
                .collect::<BTreeMap<_, _>>();
            index += 2;
            while index < lines.len() && is_markdown_table_row(lines[index].trim()) {
                let cells = split_table_row(lines[index].trim());
                for (column_index, cell) in cells.iter().enumerate() {
                    if let Some(column) = columns.get(column_index) {
                        if let Ok(value) = cell.replace([',', '$', '%'], "").parse::<f64>() {
                            *numeric_columns.entry(column.clone()).or_insert(0.0) += value;
                        }
                    }
                }
                row_count += 1;
                index += 1;
            }
            numeric_columns.retain(|_, value| *value != 0.0);
            tables.push(TableSummary {
                line: index.saturating_sub(row_count + 1),
                columns,
                rows: row_count,
                numeric_columns,
            });
        } else {
            index += 1;
        }
    }
    tables
}

fn is_markdown_table_row(line: &str) -> bool {
    line.starts_with('|') && line.ends_with('|') && line.matches('|').count() >= 2
}

fn is_markdown_table_separator(line: &str) -> bool {
    is_markdown_table_row(line)
        && line
            .trim_matches('|')
            .split('|')
            .all(|cell| cell.trim().chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

fn split_table_row(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn count_figures(text: &str) -> usize {
    text.matches("![").count()
}

fn count_equations(text: &str) -> usize {
    text.matches("$$").count() / 2
}

fn manifest_file(path: &str) -> Option<ManifestFile> {
    let bytes = fs::read(path).ok()?;
    Some(ManifestFile {
        path: path.to_string(),
        hash: sha256_hex(&bytes),
    })
}

fn scan_workspace_dir(
    root: &Path,
    dir: &Path,
    depth: usize,
    entries: &mut Vec<WorkspaceFileEntry>,
) -> Result<(), String> {
    if depth >= MAX_WORKSPACE_SCAN_DEPTH || entries.len() >= MAX_WORKSPACE_SCAN_ITEMS {
        return Ok(());
    }

    let mut children = fs::read_dir(dir)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        if entries.len() >= MAX_WORKSPACE_SCAN_ITEMS {
            break;
        }
        let path = child.path();
        let name = child.file_name().to_string_lossy().to_string();
        if should_skip_workspace_path(&path, &name) {
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .map(path_to_string)
            .unwrap_or_else(|_| path_to_string(&path));
        if path.is_dir() {
            entries.push(WorkspaceFileEntry {
                path: path_to_string(&path),
                name,
                relative_path,
                kind: "directory".to_string(),
                depth,
            });
            scan_workspace_dir(root, &path, depth + 1, entries)?;
        } else if is_workspace_document(&path) {
            let kind = path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.to_ascii_lowercase())
                .unwrap_or_else(|| "file".to_string());
            entries.push(WorkspaceFileEntry {
                path: path_to_string(&path),
                name,
                relative_path,
                kind,
                depth,
            });
        }
    }

    Ok(())
}

fn should_skip_workspace_path(path: &Path, name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }
    if path.is_dir()
        && matches!(
            name,
            "node_modules" | "target" | "dist" | "build" | ".git" | ".pnpm-store"
        )
    {
        return true;
    }
    false
}

fn is_workspace_document(path: &Path) -> bool {
    let Some(extension) = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
    else {
        return false;
    };
    matches!(
        extension.as_str(),
        "md" | "markdown"
            | "mdown"
            | "txt"
            | "text"
            | "csv"
            | "tsv"
            | "json"
            | "jsonc"
            | "yaml"
            | "yml"
            | "toml"
            | "bib"
            | "csl"
            | "tex"
            | "html"
            | "css"
            | "js"
            | "ts"
            | "vue"
            | "rs"
    )
}

fn snapshot_workspace_id(file_path: Option<&str>) -> String {
    file_path
        .map(|path| sha256_hex(path.as_bytes()))
        .unwrap_or_else(|| "unsaved".to_string())
}

fn app_snapshot_root(
    app: &tauri::AppHandle,
    file_path_or_id: Option<&str>,
) -> Result<PathBuf, String> {
    let app_data = app.path().app_data_dir().map_err(|err| err.to_string())?;
    let workspace_id = file_path_or_id
        .map(|value| {
            if value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit()) {
                value.to_string()
            } else {
                snapshot_workspace_id(Some(value))
            }
        })
        .unwrap_or_else(|| snapshot_workspace_id(None));
    Ok(app_data.join("snapshots").join(workspace_id))
}

fn modified_time(metadata: fs::Metadata) -> Option<String> {
    metadata
        .modified()
        .ok()
        .map(chrono::DateTime::<Utc>::from)
        .map(|time| time.to_rfc3339())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn slugify(text: &str) -> String {
    text.to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn diag(
    severity: impl Into<String>,
    message: impl Into<String>,
    source_file: Option<String>,
    line: Option<usize>,
    suggestion: Option<&str>,
) -> DocumentDiagnostic {
    DocumentDiagnostic {
        severity: severity.into(),
        message: message.into(),
        source_file,
        line,
        suggestion: suggestion.map(ToString::to_string),
        related: Vec::new(),
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_xml(text: &str) -> String {
    escape_html(text).replace('\'', "&apos;")
}

fn escape_pdf(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .chars()
        .filter(|ch| ch.is_ascii())
        .collect()
}

fn escape_css(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\'', "\\'")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(FileWatcherState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            open_file,
            read_file,
            save_file_as,
            save_file,
            watch_file,
            start_file_watcher,
            stop_file_watcher,
            rename_file,
            duplicate_file,
            reveal_path,
            file_metadata,
            list_workspace_files,
            compile_document,
            export_document,
            prepare_for_export,
            create_snapshot,
            list_snapshots,
            restore_snapshot,
            get_git_status,
            git_history,
            git_diff,
            commit_document_changes,
            tag_release,
            restore_git_revision,
            list_transform_engines,
            run_transform,
            run_external_transform,
            cleanup_ai_paste
        ])
        .run(tauri::generate_context!())
        .expect("error while running NEditor");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::ZipArchive;

    fn sample_document() -> String {
        r#"---
title: Test Report
version: 1.2.0
status: approved
approvedBy: QA
toc: true
client: Acme
---

# Test Report

[TOC]

Prepared for {{client}}.

```calc
revenue = 100
cost = 40
profit = revenue - cost
margin = profit / revenue
```

Margin: {{=margin | percent}}

```csv
Region,Revenue
East,100
West,80
```

```glossary
ARR: Annual recurring revenue.
```

[INDEX]
"#
        .to_string()
    }

    #[test]
    fn compiler_resolves_metadata_variables_transforms_and_manifest() {
        let response = compile(CompileRequest {
            text: sample_document(),
            file_path: None,
        });

        assert_eq!(response.semantic.title, "Test Report");
        assert_eq!(response.semantic.status, "approved");
        assert!(response.compiled_markdown.contains("Prepared for Acme."));
        assert!(response.compiled_markdown.contains("Margin: 60.00%"));
        assert!(response.html.contains("Table of Contents"));
        assert!(response.html.contains("transform-table"));
        assert!(response.html.contains("<h1 id=\"test-report\">"));
        assert!(response.html.contains("href=\"#test-report\""));
        assert!(response.index_terms.iter().any(|term| term == "ARR"));
        assert_eq!(response.export_manifest.document_version, "1.2.0");
        assert!(response
            .formula_graph
            .iter()
            .any(|formula| formula.name == "profit" && formula.value == Some(60.0)));
    }

    #[test]
    fn compiler_supports_default_document_variables() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Defaults\nstatus: approved\napprovedBy: QA\nclient: Acme\n---\n# Defaults\nPrepared for {{client | default:Fallback}} in {{region | default:\"East Africa\"}}.\nStill missing {{owner}}.\n".to_string(),
            file_path: None,
        });

        assert!(response
            .compiled_markdown
            .contains("Prepared for Acme in East Africa."));
        assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("Missing document variable: owner")));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("region")));
    }

    #[test]
    fn compiler_loads_project_level_variables_without_overriding_front_matter() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-project-vars-test-{unique}"));
        fs::create_dir_all(root.join(".neditor")).expect("create project vars dir");
        fs::write(
            root.join(".neditor").join("variables.yaml"),
            "client: Project Client\nregion: West\nowner: Strategy Office\n",
        )
        .expect("write project variables");
        let doc = root.join("docs").join("report.md");
        fs::create_dir_all(doc.parent().expect("doc parent")).expect("create docs dir");
        fs::write(&doc, "# Report").expect("write doc");

        let response = compile(CompileRequest {
            text: "---\ntitle: Project Vars\nstatus: approved\napprovedBy: QA\nclient: Front Matter Client\n---\n# Project Vars\nPrepared for {{client}} in {{region}} by {{owner}}.\n".to_string(),
            file_path: Some(path_to_string(&doc)),
        });

        assert!(response
            .compiled_markdown
            .contains("Prepared for Front Matter Client in West by Strategy Office."));
        assert_eq!(response.metadata["client"], "Front Matter Client");
        assert_eq!(response.metadata["region"], "West");
        fs::remove_dir_all(root).expect("clean project vars test dir");
    }

    #[test]
    fn compiler_honors_toc_depth_and_numbering() {
        let response = compile(CompileRequest {
            text: "---\ntitle: TOC\nstatus: approved\napprovedBy: QA\ntoc: true\ntocDepth: 2\ntocNumbered: true\n---\n# Alpha\n## Beta\n### Gamma\n## Delta\n".to_string(),
            file_path: None,
        });

        assert!(response.compiled_markdown.contains("- [1 Alpha](#alpha)"));
        assert!(response.compiled_markdown.contains("  - [1.1 Beta](#beta)"));
        assert!(response
            .compiled_markdown
            .contains("  - [1.2 Delta](#delta)"));
        assert!(!response.compiled_markdown.contains("[1.1.1 Gamma](#gamma)"));
    }

    #[test]
    fn compiler_adds_glossary_hover_terms_to_preview_html() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Glossary Hover\nstatus: approved\napprovedBy: QA\n---\n# Glossary Hover\nARR informs planning.\n\n```glossary\nARR: Annual recurring revenue.\n```\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("class=\"glossary-term\""));
        assert!(response
            .html
            .contains("title=\"Annual recurring revenue.\""));
        assert!(response.html.contains(">ARR</span> informs planning"));
    }

    #[test]
    fn compiler_generates_linked_index_with_exclusions_and_proper_terms() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Index\nstatus: approved\napprovedBy: QA\nindexExclude:\n  - Internal Draft\n---\n# Market Analysis\nAcme Strategy appears here. **Working Capital** matters.\n\n## Follow Up\nAcme Strategy returns. Internal Draft should stay out. Working capital{#index:Liquidity} marker.\n\n[INDEX]\n".to_string(),
            file_path: None,
        });

        assert!(response
            .index_terms
            .iter()
            .any(|term| term == "Acme Strategy"));
        assert!(response.index_terms.iter().any(|term| term == "Liquidity"));
        assert!(response
            .index_terms
            .iter()
            .any(|term| term == "Working Capital"));
        assert!(!response
            .index_terms
            .iter()
            .any(|term| term == "Internal Draft"));
        assert!(response.html.contains("href=\"#market-analysis\""));
        assert!(response.html.contains("Acme Strategy"));
        assert!(response.html.contains("Liquidity"));
        assert!(!response.html.contains("{#index:Liquidity}"));
    }

    #[test]
    fn compiler_parses_review_comment_metadata() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Review\nstatus: approved\napprovedBy: QA\n---\n# Review\n<!-- comment: unresolved | author: Dana | at: 2026-05-18T10:00:00Z | Clarify the risk note. -->\n".to_string(),
            file_path: None,
        });
        let comment = response.semantic.comments.first().expect("review comment");

        assert_eq!(comment.state, "unresolved");
        assert_eq!(comment.author, "Dana");
        assert_eq!(comment.created_at, "2026-05-18T10:00:00Z");
        assert_eq!(comment.text, "Clarify the risk note.");
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unresolved review comments")));
    }

    #[test]
    fn compiler_reports_missing_include_without_panicking() {
        let response = compile(CompileRequest {
            text: "!include missing/chapter.md\n".to_string(),
            file_path: None,
        });

        assert!(response.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == "error" && diagnostic.message.contains("Missing include file")
        }));
    }

    #[test]
    fn compiler_reports_broken_local_markdown_links() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-link-test-{unique}"));
        fs::create_dir_all(root.join("docs")).expect("create link test dir");
        fs::write(root.join("docs").join("existing.md"), "# Existing").expect("write linked doc");

        let response = compile(CompileRequest {
            text: "---\ntitle: Links\nstatus: approved\napprovedBy: QA\n---\n# Links\nRead [existing](docs/existing.md), [missing](docs/missing.md), [section](#links), and [web](https://example.com).\n![Missing image](docs/missing.png)\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Broken link path")));
        assert_eq!(
            response
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.message.contains("Broken link path"))
                .count(),
            1
        );
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Broken image path")));
        fs::remove_dir_all(root).expect("clean link test dir");
    }

    #[test]
    fn compiler_loads_external_bibliography_and_validates_cross_refs() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-bib-test-{unique}"));
        fs::create_dir_all(&root).expect("create bib test dir");
        fs::write(
            root.join("refs.bib"),
            "@book{porter1985, title={Competitive Advantage}}\n@article{doe2026, title={Evidence Based Reports}}",
        )
        .expect("write bibliography");
        fs::write(root.join("diagram.svg"), "<svg></svg>").expect("write figure");

        let response = compile(CompileRequest {
            text: "---\ntitle: Cited\nstatus: approved\napprovedBy: QA\nbibliography: refs.bib\n---\n# Cited\nClaim [@porter1985, p. 42; @doe2026].\n\n![Diagram](diagram.svg){#fig:diagram caption=\"System diagram\"}\nSee {@fig:diagram} and {@fig:missing}.\n\n![Missing](missing.png){#fig:missing-image caption=\"Missing image\"}".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert_eq!(response.bibliography.len(), 2);
        assert_eq!(response.semantic.citations, vec!["doe2026", "porter1985"]);
        assert!(response
            .semantic
            .citation_references
            .iter()
            .any(|citation| {
                citation.key == "porter1985" && citation.locator.as_deref() == Some("p. 42")
            }));
        assert!(response
            .html
            .contains("Competitive Advantage, p. 42; Evidence Based Reports"));
        assert!(response
            .html
            .contains("title=\"@porter1985 (p. 42): Competitive Advantage; @doe2026: Evidence Based Reports\""));
        assert!(response
            .html
            .contains("aria-label=\"Citation: @porter1985 (p. 42): Competitive Advantage; @doe2026: Evidence Based Reports\""));
        assert!(response.html.contains("<figure"));
        assert!(response.html.contains("System diagram"));
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Broken image path")));
        assert!(response
            .semantic
            .cross_references
            .iter()
            .any(|reference| reference.key == "fig:diagram" && reference.resolved));
        assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("Broken cross reference: fig:missing")));
        fs::remove_dir_all(root).expect("clean bib test dir");
    }

    #[test]
    fn compiler_loads_csl_json_bibliography() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-csl-test-{unique}"));
        fs::create_dir_all(&root).expect("create csl test dir");
        fs::write(
            root.join("refs.json"),
            r#"[{"id":"doe2026","title":"Evidence Based Reports"}]"#,
        )
        .expect("write csl bibliography");

        let response = compile(CompileRequest {
            text: "---\ntitle: CSL\nstatus: approved\napprovedBy: QA\nbibliography: refs.json\n---\n# CSL\nClaim [@doe2026].\n[BIBLIOGRAPHY]".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert_eq!(response.bibliography[0].key, "doe2026");
        assert!(response.html.contains("Evidence Based Reports"));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Broken citation")));
        fs::remove_dir_all(root).expect("clean csl test dir");
    }

    #[test]
    fn compiler_reports_duplicate_bibliography_keys() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Duplicate Bibliography\nstatus: approved\napprovedBy: QA\n---\n# Duplicate Bibliography\nClaim [@porter1985].\n\n```bibtex\n@book{porter1985, title={Competitive Advantage}}\n@article{porter1985, title={Duplicate Entry}}\n```\n[BIBLIOGRAPHY]".to_string(),
            file_path: None,
        });

        assert_eq!(
            response.semantic.duplicate_bibliography_keys,
            vec!["porter1985".to_string()]
        );
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Duplicate bibliography key")));
    }

    #[test]
    fn compiler_renders_block_and_inline_equations() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Math\nstatus: approved\napprovedBy: QA\n---\n# Math\nInline \\(ROI = x\\).\n\n$$\nROI = \\frac{Gain - Cost}{Cost}\n$$ {#eq:roi}\n\nSee {@eq:roi}.".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("class=\"equation\""));
        assert!(response.html.contains("id=\"eq:roi\""));
        assert!(response.html.contains("Equation 1"));
        assert!(response.html.contains("class=\"math\""));
        assert!(response
            .semantic
            .cross_references
            .iter()
            .any(|reference| reference.key == "eq:roi" && reference.resolved));
    }

    #[test]
    fn compiler_summarizes_markdown_tables() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Tables\nstatus: approved\napprovedBy: QA\n---\n# Tables\n| Region | Revenue |\n| --- | ---: |\n| East | 100 |\n| West | =SUM(70,10) |\n".to_string(),
            file_path: None,
        });

        assert!(response.compiled_markdown.contains("| West | 80 |"));
        assert!(response.html.contains(">80</td>"));
        assert_eq!(response.semantic.tables, 1);
        assert_eq!(response.semantic.table_summaries[0].rows, 2);
        assert_eq!(
            response.semantic.table_summaries[0]
                .numeric_columns
                .get("Revenue"),
            Some(&180.0)
        );
    }

    #[test]
    fn csv_and_tsv_transforms_evaluate_table_formula_cells() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Formula Tables\nstatus: approved\napprovedBy: QA\n---\n# Formula Tables\n```csv\nMetric,Value\nTotal,=10+15\nRounded,=ROUND(2.6)\n```\n\n```tsv\nMetric\tValue\nAbs\t=ABS(-5)\nSum\t=SUM(2,3)\n```\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("<td>25</td>"));
        assert!(response.html.contains("<td>3</td>"));
        assert!(response.html.contains("<td>5</td>"));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Table formula error")));
    }

    #[test]
    fn compiler_renders_layout_break_directives() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Layout\nstatus: approved\napprovedBy: QA\n---\n# Layout\n{{page-break}}\n{{section-break columns=1}}\n\n```layout\ncolumns: 2\n```\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("data-layout=\"page-break\""));
        assert!(response.html.contains("data-layout=\"section-break\""));
        assert!(response.html.contains("columns=1"));
        assert!(response.html.contains("data-layout=\"layout\""));
        assert!(response.html.contains("column-count:2"));
    }

    #[test]
    fn compiler_renders_callouts_as_semantic_blocks() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Callouts\nstatus: approved\napprovedBy: QA\n---\n# Callouts\n> [!NOTE] Board review\n> Confirm the launch criteria.\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("class=\"callout callout-note\""));
        assert!(response.html.contains("Board review"));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Callout { callout_type, title, text, .. }
                    if callout_type == "note"
                        && title == "Board review"
                        && text.contains("Confirm the launch criteria")
            )
        }));

        let options = json!({});
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        assert!(zip_entry_text(&docx, "word/document.xml")
            .contains("Callout: note: Board review: Confirm the launch criteria."));
        let pdf = render_pdf_bytes(&response, &options);
        assert!(String::from_utf8_lossy(&pdf).contains("Callout: note: Board review"));
    }

    #[test]
    fn compiler_builds_document_ast_blocks_for_exports() {
        let response = compile(CompileRequest {
            text: "---\ntitle: AST\nstatus: approved\napprovedBy: QA\n---\n# AST\nBusiness paragraph.\n\n| Metric | Value |\n| --- | ---: |\n| Total | =SUM(1,2) |\n\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:diagram caption=\"System diagram\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n{{page-break}}\n".to_string(),
            file_path: None,
        });

        assert!(response
            .document_ast
            .blocks
            .iter()
            .any(|block| matches!(block, DocumentBlock::Heading { text, anchor, .. } if text == "AST" && anchor == "ast")));
        assert!(response
            .document_ast
            .blocks
            .iter()
            .any(|block| matches!(block, DocumentBlock::Paragraph { text, line, end_line, .. } if text == "Business paragraph." && line == end_line)));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Table { line, end_line, headers, rows, .. }
                    if headers == &vec!["Metric".to_string(), "Value".to_string()]
                        && *end_line == *line + 2
                        && rows.iter().any(|row| row == &vec!["Total".to_string(), "3".to_string()])
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Figure { id, caption, .. }
                    if id.as_deref() == Some("fig:diagram")
                        && caption.as_deref() == Some("System diagram")
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Equation { id, text, .. }
                    if id.as_deref() == Some("eq:roi") && text.contains("ROI")
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Layout { directive, .. } if directive == "page-break"
            )
        }));

        let exported = export::export_text(&response, &json!({}));
        assert!(exported.contains("Table: Metric | Value"));
        assert!(exported.contains("Figure: fig:diagram: System diagram"));
        assert!(exported.contains("Equation: eq:roi: ROI = Gain / Cost"));
    }

    #[test]
    fn compiler_renders_openapi_and_json_schema_tables() {
        let response = compile(CompileRequest {
            text: r#"---
title: API
status: approved
approvedBy: QA
---
# API

```openapi
openapi: 3.1.0
paths:
  /accounts:
    get:
      summary: List accounts
```

```json-schema
{
  "type": "object",
  "required": ["id"],
  "properties": {
    "id": { "type": "string", "description": "Account id" },
    "balance": { "type": "number" }
  }
}
```
"#
            .to_string(),
            file_path: None,
        });

        assert!(response.html.contains("List accounts"));
        assert!(response.html.contains("Account id"));
        assert!(response.html.contains("<td>yes</td>"));
    }

    #[test]
    fn transform_registry_covers_required_first_release_transforms() {
        let engines = list_transform_engines();
        let names = engines
            .iter()
            .filter_map(|engine| engine.get("name").and_then(Value::as_str))
            .collect::<BTreeSet<_>>();
        let pikchr = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("pikchr"))
            .expect("pikchr engine metadata");
        assert_eq!(
            pikchr.get("trustRequired").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(
            pikchr.get("preferenceKey").and_then(Value::as_str),
            Some("transforms.pikchr.path")
        );

        for name in [
            "calc",
            "mermaid",
            "pikchr",
            "dot",
            "graphviz",
            "plantuml",
            "d2",
            "vega-lite",
            "chart",
            "geojson",
            "topojson",
            "stl",
            "csv",
            "tsv",
            "json",
            "yaml",
            "openapi",
            "json-schema",
            "bibtex",
            "glossary",
            "layout",
            "timeline",
            "roadmap",
            "adr",
            "diff",
        ] {
            assert!(
                names.contains(name),
                "missing transform registry entry: {name}"
            );
            assert!(supported_transform(name), "unsupported transform: {name}");
        }

        let response = compile(CompileRequest {
            text: "---\ntitle: Diagram\n---\n# Diagram\n```pikchr\nbox \"A\"\n```\n".to_string(),
            file_path: None,
        });
        assert!(response.html.contains("transform-pending"));
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("pikchr transform captured")));
    }

    #[test]
    fn bibtex_transform_renders_bibliography_preview() {
        let artifact = run_transform(
            "bibtex".to_string(),
            "@book{porter1985, title={Competitive Advantage}}".to_string(),
        )
        .expect("bibtex transform");

        assert_eq!(artifact.output_kind, "html");
        assert!(artifact.html.contains("transform-bibtex"));
        assert!(artifact.html.contains("<dt>porter1985</dt>"));
        assert!(artifact.html.contains("<dd>Competitive Advantage</dd>"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let bibtex = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("bibtex"))
            .expect("bibtex engine metadata");
        assert_eq!(
            bibtex.get("execution").and_then(Value::as_str),
            Some("rust-native")
        );
    }

    #[test]
    fn structured_data_transforms_render_tables_and_trees() {
        let json_artifact = run_transform(
            "json".to_string(),
            r#"[{"region":"East","revenue":120},{"region":"West","revenue":98}]"#.to_string(),
        )
        .expect("json transform");
        assert_eq!(json_artifact.output_kind, "html");
        assert!(json_artifact.html.contains("transform-json"));
        assert!(json_artifact.html.contains("<th>region</th>"));
        assert!(json_artifact.html.contains("<td>East</td>"));
        assert!(json_artifact.diagnostics.is_empty());

        let yaml_artifact = run_transform(
            "yaml".to_string(),
            "api:\n  version: v1\n  endpoints:\n    - /accounts\n".to_string(),
        )
        .expect("yaml transform");
        assert_eq!(yaml_artifact.output_kind, "html");
        assert!(yaml_artifact.html.contains("structured-tree"));
        assert!(yaml_artifact.html.contains("<dt>version</dt>"));
        assert!(yaml_artifact.html.contains("/accounts"));
        assert!(yaml_artifact.diagnostics.is_empty());
    }

    #[test]
    fn chart_transform_renders_yaml_business_chart_specs() {
        let artifact = run_transform(
            "chart".to_string(),
            "type: bar\ntitle: Revenue by Region\ndata:\n  - region: East\n    revenue: 120\n  - region: West\n    revenue: 98\nx: region\ny: revenue\n".to_string(),
        )
        .expect("chart transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("class=\"chart\""));
        assert!(artifact.html.contains("Revenue by Region"));
        assert!(artifact.html.contains(">East<"));
        assert!(artifact.html.contains(">120<"));
        assert!(artifact.diagnostics.is_empty());
    }

    #[test]
    fn timeline_transform_renders_static_svg_preview() {
        let artifact = run_transform(
            "timeline".to_string(),
            "2026-05-18: Kickoff\n2026-06-01: Review\n2026-06-15: Release\n".to_string(),
        )
        .expect("timeline transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("class=\"timeline\""));
        assert!(artifact.html.contains("Kickoff"));
        assert!(artifact.html.contains("Release"));
        assert!(artifact.diagnostics.is_empty());
    }

    #[test]
    fn business_workflow_transforms_render_static_html() {
        let roadmap = run_transform(
            "roadmap".to_string(),
            "Now: Drafting\nNext: Review\nLater: Publish".to_string(),
        )
        .expect("roadmap transform");
        assert_eq!(roadmap.output_kind, "html");
        assert!(roadmap.html.contains("transform-roadmap"));
        assert!(roadmap.html.contains("Review"));

        let adr = run_transform(
            "adr".to_string(),
            "Status: accepted\nDecision: Use local-first exports".to_string(),
        )
        .expect("adr transform");
        assert_eq!(adr.output_kind, "html");
        assert!(adr.html.contains("transform-adr"));
        assert!(adr.html.contains("Use local-first exports"));

        let diff = run_transform("diff".to_string(), "@@ -1 +1 @@\n-old\n+new".to_string())
            .expect("diff transform");
        assert_eq!(diff.output_kind, "html");
        assert!(diff.html.contains("transform-diff"));
        assert!(diff.html.contains("diff-del"));
        assert!(diff.html.contains("diff-add"));
    }

    #[test]
    fn geojson_transform_renders_static_svg_preview() {
        let artifact = run_transform(
            "geojson".to_string(),
            r#"{"type":"Feature","geometry":{"type":"LineString","coordinates":[[36.80,-1.30],[36.85,-1.26],[36.90,-1.28]]}}"#.to_string(),
        )
        .expect("geojson transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-geojson"));
        assert!(artifact.html.contains("<polyline"));
        assert!(artifact.html.contains("3 coordinates"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let geojson = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("geojson"))
            .expect("geojson engine metadata");
        assert_eq!(
            geojson.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn topojson_transform_renders_static_svg_preview() {
        let artifact = run_transform(
            "topojson".to_string(),
            r#"{"type":"Topology","transform":{"scale":[0.01,0.01],"translate":[36.8,-1.3]},"objects":{},"arcs":[[[0,0],[5,4],[5,-2]]]}"#.to_string(),
        )
        .expect("topojson transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-topojson"));
        assert!(artifact.html.contains("<polyline"));
        assert!(artifact.html.contains("1 arcs"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let topojson = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("topojson"))
            .expect("topojson engine metadata");
        assert_eq!(
            topojson.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn stl_transform_renders_ascii_static_svg_preview() {
        let artifact = run_transform(
            "stl".to_string(),
            "solid test\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 10 0 0\nvertex 0 10 0\nendloop\nendfacet\nendsolid test".to_string(),
        )
        .expect("stl transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-stl"));
        assert!(artifact.html.contains("<polygon"));
        assert!(artifact.html.contains("1 triangles / 3 vertices"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let stl = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("stl"))
            .expect("stl engine metadata");
        assert_eq!(
            stl.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn vega_lite_transform_renders_static_svg_preview() {
        let artifact = run_transform(
            "vega-lite".to_string(),
            r#"{"mark":"bar","title":"Revenue","data":{"values":[{"region":"East","revenue":120},{"region":"West","revenue":98}]},"encoding":{"x":{"field":"region","type":"nominal"},"y":{"field":"revenue","type":"quantitative"}}}"#.to_string(),
        )
        .expect("vega-lite transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-vega-lite"));
        assert!(artifact.html.contains("Revenue"));
        assert!(artifact.html.contains("<rect"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let vega_lite = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("vega-lite"))
            .expect("vega-lite engine metadata");
        assert_eq!(
            vega_lite.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn mermaid_transform_renders_simple_flowchart_svg() {
        let artifact = run_transform(
            "mermaid".to_string(),
            "flowchart TD\nA[Start] --> B{Review}\nB -->|Approve| C[Publish]".to_string(),
        )
        .expect("mermaid transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-mermaid"));
        assert!(artifact.html.contains("Start"));
        assert!(artifact.html.contains("Publish"));
        assert!(artifact.html.contains("marker-end"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let mermaid = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("mermaid"))
            .expect("mermaid engine metadata");
        assert_eq!(
            mermaid.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn external_transforms_are_trust_gated_and_limited() {
        let trust_error = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "digraph {}".to_string(),
            engine_path: Some("/bin/cat".to_string()),
            trusted: false,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
        })
        .unwrap_err();
        assert!(trust_error.contains("explicit trust"));

        let cat = Path::new("/bin/cat");
        if !cat.exists() {
            return;
        }
        let cat_path = path_to_string(cat);
        let limit_error = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "1234".to_string(),
            engine_path: Some(cat_path.clone()),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(3),
        })
        .unwrap_err();
        assert!(limit_error.contains("above the 3 byte limit"));

        let stdin_artifact = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "<svg>ok</svg>".to_string(),
            engine_path: Some(cat_path.clone()),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
        })
        .expect("stdin external transform");
        assert_eq!(stdin_artifact.execution_kind, "external");
        assert_eq!(stdin_artifact.input_mode, "stdin");
        assert!(stdin_artifact.html.contains("<svg>ok</svg>"));
        assert!(!stdin_artifact.cache_key.is_empty());

        let file_artifact = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "digraph {}".to_string(),
            engine_path: Some(cat_path),
            trusted: true,
            input_mode: Some("file".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
        })
        .expect("file external transform");
        assert_eq!(file_artifact.input_mode, "file");
        assert!(file_artifact.html.contains("digraph"));
    }

    #[test]
    fn include_expansion_strips_child_front_matter() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-test-{unique}"));
        let chapter_dir = root.join("chapters");
        fs::create_dir_all(&chapter_dir).expect("create test dirs");
        fs::write(
            chapter_dir.join("intro.md"),
            "---\ntitle: Child\n---\n\n## Included\n\nBody",
        )
        .expect("write include");

        let response = compile(CompileRequest {
            text: "---\ntitle: Root\n---\n\n!include chapters/intro.md\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert!(response.compiled_markdown.contains("## Included"));
        assert!(!response.compiled_markdown.contains("title: Child"));
        assert_eq!(response.include_graph.len(), 1);
        let included_line = response
            .compiled_markdown
            .lines()
            .position(|line| line == "## Included")
            .map(|index| index + 1)
            .expect("included heading line");
        assert!(response.source_map.iter().any(|entry| {
            entry.generated_line == included_line
                && entry.source_file.ends_with("chapters/intro.md")
                && entry.source_line == 2
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Heading { text, source: Some(source), .. }
                    if text == "Included"
                        && source.source_file.ends_with("chapters/intro.md")
                        && source.source_line == 2
                        && source.end_source_line == 2
            )
        }));
        fs::remove_dir_all(root).expect("clean test dirs");
    }

    #[test]
    fn include_expansion_supports_documented_directive_forms() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-include-forms-test-{unique}"));
        fs::create_dir_all(root.join("chapters")).expect("create include forms dirs");
        fs::create_dir_all(root.join("appendices")).expect("create appendices dir");
        fs::write(root.join("chapters").join("intro.md"), "## Bang Include\n")
            .expect("write bang include");
        fs::write(
            root.join("chapters").join("market.md"),
            "## Brace Include\n",
        )
        .expect("write brace include");
        fs::write(
            root.join("appendices").join("financials.md"),
            "## Comment Include\n",
        )
        .expect("write comment include");

        let response = compile(CompileRequest {
            text: "!include chapters/intro.md\n{{include chapters/market.md}}\n<!-- include: appendices/financials.md -->\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert!(response.compiled_markdown.contains("## Bang Include"));
        assert!(response.compiled_markdown.contains("## Brace Include"));
        assert!(response.compiled_markdown.contains("## Comment Include"));
        assert_eq!(response.include_graph.len(), 3);
        assert!(response
            .export_manifest
            .included_files
            .iter()
            .any(|file| file.path.ends_with("chapters/market.md")));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Missing include")));
        fs::remove_dir_all(root).expect("clean include forms test dir");
    }

    #[test]
    fn export_renderers_return_non_empty_artifacts() {
        let response = compile(CompileRequest {
            text: sample_document(),
            file_path: None,
        });

        let html = render_full_html(&response, &json!({ "watermark": "DRAFT" }));
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("class=\"cover\""));
        assert!(html.contains("Page {{page}} of {{pages}}") || html.contains("Page 1 of 1"));
        assert!(html.contains("DRAFT"));
        let options = json!({ "watermark": "DRAFT" });
        let pdf = render_pdf_bytes(&response, &options);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(String::from_utf8_lossy(&pdf).contains("Page 1 of 1"));
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        assert!(docx.len() > 100);
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains("Cover: Test Report"));
        assert!(docx_document.contains("Watermark: DRAFT"));
        let docx_core = zip_entry_text(&docx, "docProps/core.xml");
        assert!(docx_core.contains("<dc:title>Test Report</dc:title>"));
        assert!(docx_core.contains("<cp:category>approved</cp:category>"));
        let docx_relationships = zip_entry_text(&docx, "_rels/.rels");
        assert!(docx_relationships.contains("metadata/core-properties"));
        let docx_document_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
        assert!(docx_document_relationships.contains("relationships/header"));
        assert!(docx_document_relationships.contains("relationships/footer"));
        assert!(zip_entry_text(&docx, "word/header1.xml").contains("Test Report"));
        assert!(zip_entry_text(&docx, "word/footer1.xml").contains("Page 1 of 1"));
        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        assert!(pptx.len() > 100);
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide1.xml");
        assert!(pptx_slide.contains("Test Report"));
        assert!(pptx_slide.contains("Page 1 of 1"));
        let pptx_core = zip_entry_text(&pptx, "docProps/core.xml");
        assert!(pptx_core.contains("<dc:title>Test Report</dc:title>"));
        assert!(pptx_core.contains("<cp:category>approved</cp:category>"));
        assert!(
            render_markdown_bundle_bytes(&response, &response.export_manifest)
                .expect("bundle bytes")
                .starts_with(b"PK")
        );
    }

    #[test]
    fn semantic_exporters_map_ast_blocks() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Semantic Export\nstatus: approved\napprovedBy: QA\n---\n# Semantic Exports\nBusiness paragraph.\n\n| Metric | Value |\n| --- | ---: |\n| Total | =SUM(1,2) |\n\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:diagram caption=\"System diagram\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n{{page-break}}\n{{section-break columns=2}}\n\n## Appendix\nAfter the break.\n".to_string(),
            file_path: None,
        });
        let options = json!({ "watermark": "DRAFT" });

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading1""#));
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading2""#));
        assert!(docx_document.contains("<w:tbl>"));
        assert!(docx_document.contains(r#"<w:br w:type="page""#));
        assert!(docx_document.contains(r#"<w:cols w:num="2""#));
        assert!(docx_document.contains("System diagram"));
        assert!(docx_document.contains("ROI = Gain / Cost"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
        assert!(presentation.contains(r#"r:id="rId2""#));
        let slide_two = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert!(slide_two.contains("Semantic Exports"));
        assert!(slide_two.contains("Table: Metric | Value"));
        assert!(slide_two.contains("System diagram"));
        let slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
        assert!(slide_three.contains("Section"));
        assert!(slide_three.contains("Section break: columns=2"));
        let slide_four = zip_entry_text(&pptx, "ppt/slides/slide4.xml");
        assert!(slide_four.contains("Appendix"));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("/Count 3"));
        assert!(pdf_text.contains("Section break: columns=2"));
        assert!(pdf_text.contains("System diagram"));
        assert!(pdf_text.contains("After the break."));
    }

    #[test]
    fn export_conformance_fixture_maps_business_features() {
        let response = compile(CompileRequest {
            text: include_str!("../fixtures/export/business_report.md").to_string(),
            file_path: None,
        });
        let options = json!({
            "watermark": "APPROVED",
            "includeGlossary": true,
            "includeComments": true,
            "includeProvenance": true
        });

        assert_eq!(response.semantic.title, "Export Conformance Report");
        assert_eq!(response.semantic.status, "approved");
        assert_eq!(response.export_manifest.document_version, "2.0.0");
        assert!(response
            .semantic
            .citations
            .iter()
            .any(|citation| citation == "porter1985"));
        assert!(response.semantic.glossary.contains_key("ARR"));
        assert!(response.semantic.comments.iter().any(|comment| comment
            .text
            .contains("board-pack export fidelity")
            && comment.state == "resolved"));
        assert!(response
            .semantic
            .ai_sources
            .iter()
            .any(|source| source.provider == "OpenAI" && source.status == "human-reviewed"));
        assert_eq!(response.semantic.tables, 1);
        assert_eq!(response.semantic.figures, 1);
        assert_eq!(response.semantic.equations, 1);
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Layout { directive, .. } if directive == "page-break"
            )
        }));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "error"));

        let html = render_full_html(&response, &options);
        assert!(html.contains("Board Pack Fixture"));
        assert!(html.contains("APPROVED"));
        assert!(html.contains("Reference architecture"));
        assert!(html.contains("Competitive Advantage"));
        assert!(html.contains("class=\"export-glossary\""));
        assert!(html.contains("<dt>ARR</dt>"));
        assert!(html.contains("class=\"export-comments\""));
        assert!(html.contains("Verify board-pack export fidelity."));
        assert!(html.contains("class=\"export-provenance\""));
        assert!(html.contains("gpt-5.4"));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf_text.contains("/Count 6"));
        assert!(pdf_text.contains("Export Conformance Report"));
        assert!(pdf_text.contains("Reference architecture"));
        assert!(pdf_text.contains("Glossary"));
        assert!(pdf_text.contains("Review Comments"));
        assert!(pdf_text.contains("AI Provenance"));

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        let docx_header = zip_entry_text(&docx, "word/header1.xml");
        let docx_footer = zip_entry_text(&docx, "word/footer1.xml");
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading1""#));
        assert!(docx_document.contains("w:headerReference"));
        assert!(docx_document.contains("w:footerReference"));
        assert!(docx_header.contains("Export Conformance Report | restricted"));
        assert!(docx_footer.contains("Page 1 of 1"));
        assert!(docx_document.contains("<w:tbl>"));
        assert!(docx_document.contains(r#"<w:br w:type="page""#));
        assert!(docx_document.contains("Reference architecture"));
        assert!(docx_document.contains("Competitive Advantage"));
        assert!(docx_document.contains("Annual recurring revenue"));
        assert!(docx_document.contains("Review Comments"));
        assert!(docx_document.contains("Verify board-pack export fidelity."));
        assert!(docx_document.contains("AI Provenance"));
        assert!(docx_document.contains("gpt-5.4"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let pptx_presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
        let pptx_slide_two = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        let pptx_slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
        assert!(pptx_presentation.contains(r#"r:id="rId2""#));
        assert!(pptx_slide_two.contains("Export Conformance Report"));
        assert!(pptx_slide_three.contains("Table: Region | Revenue | Margin"));
        assert!(pptx_slide_three.contains("Reference architecture"));
        let pptx_glossary_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
            .into_iter()
            .find(|slide| slide.contains("Glossary"))
            .expect("glossary slide");
        assert!(pptx_glossary_slide.contains("Annual recurring revenue"));
        let pptx_comments_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
            .into_iter()
            .find(|slide| slide.contains("Review Comments"))
            .expect("comments slide");
        assert!(pptx_comments_slide.contains("Verify board-pack export fidelity."));
        let pptx_provenance_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
            .into_iter()
            .find(|slide| slide.contains("AI Provenance"))
            .expect("provenance slide");
        assert!(pptx_provenance_slide.contains("gpt-5.4"));

        let exported_text = export::export_text(&response, &options);
        assert!(exported_text.contains("Glossary"));
        assert!(exported_text.contains("ARR: Annual recurring revenue"));
        assert!(exported_text.contains("Review Comments"));
        assert!(exported_text.contains("AI Provenance"));

        let mut bundle_manifest = response.export_manifest.clone();
        bundle_manifest.export_options = options.clone();
        let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
        let bundled_markdown = zip_entry_text(&bundle, "document.md");
        let bundled_text = zip_entry_text(&bundle, "document.txt");
        let bundled_manifest = zip_entry_text(&bundle, "manifest.json");
        assert!(bundled_markdown.contains("Competitive Advantage"));
        assert!(bundled_text.contains("Figure: fig:architecture: Reference architecture"));
        assert!(bundled_text.contains("Verify board-pack export fidelity."));
        assert!(bundled_text.contains("OpenAI / gpt-5.4"));
        assert!(bundled_manifest.contains("\"document_title\": \"Export Conformance Report\""));
    }

    fn zip_entry_text(bytes: &[u8], path: &str) -> String {
        let cursor = Cursor::new(bytes.to_vec());
        let mut archive = ZipArchive::new(cursor).expect("zip archive");
        let mut entry = archive.by_name(path).expect("zip entry");
        let mut text = String::new();
        entry.read_to_string(&mut text).expect("zip text");
        text
    }

    fn zip_entry_texts_with_prefix(bytes: &[u8], prefix: &str) -> Vec<String> {
        let cursor = Cursor::new(bytes.to_vec());
        let mut archive = ZipArchive::new(cursor).expect("zip archive");
        let mut entries = Vec::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("zip entry by index");
            if !entry.name().starts_with(prefix) || !entry.name().ends_with(".xml") {
                continue;
            }
            let mut text = String::new();
            entry.read_to_string(&mut text).expect("zip text");
            entries.push(text);
        }
        entries
    }

    #[test]
    fn prepare_for_export_blocks_warning_cleanliness() {
        let report = prepare_for_export(PrepareExportRequest {
            text: "---\ntitle: Draft\nstatus: draft\n---\n# Draft".to_string(),
            file_path: None,
            target: "pdf".to_string(),
            options: json!({ "watermark": "DRAFT", "includeManifest": true }),
        });

        assert!(!report.ready);
        assert_eq!(report.error_count, 0);
        assert!(report.warning_count > 0);
    }

    #[test]
    fn prepare_for_export_validates_target_and_options() {
        let report = prepare_for_export(PrepareExportRequest {
            text: "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\n---\n# Ready".to_string(),
            file_path: None,
            target: "rtf".to_string(),
            options: json!({
                "watermark": 42,
                "includeManifest": "yes",
                "includeGlossary": "yes",
                "includeComments": "yes",
                "includeProvenance": "yes"
            }),
        });

        assert!(!report.ready);
        assert_eq!(report.error_count, 6);
        assert_eq!(report.manifest.export_target, "rtf");
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Unsupported export target")));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("watermark must be a string")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeManifest must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeGlossary must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeComments must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeProvenance must be true or false")));
    }

    #[test]
    fn prepare_for_export_warns_on_dirty_git_tree() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-export-git-test-{unique}"));
        fs::create_dir_all(&root).expect("create export git test dir");
        run_git(&root, &["init"]).expect("init git repo");
        let doc = root.join("doc.md");
        fs::write(
            &doc,
            "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\n---\n# Ready",
        )
        .expect("write doc");

        let report = prepare_for_export(PrepareExportRequest {
            text: fs::read_to_string(&doc).expect("read doc"),
            file_path: Some(path_to_string(&doc)),
            target: "pdf".to_string(),
            options: json!({ "includeManifest": true }),
        });

        assert!(!report.ready);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("Git working tree is dirty before export")));
        fs::remove_dir_all(root).expect("clean export git test dir");
    }

    #[test]
    fn approved_documents_require_approval_metadata() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Approved\nstatus: approved\n---\n# Approved\n".to_string(),
            file_path: None,
        });

        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("missing approval metadata")));
    }

    #[test]
    fn file_duplicate_and_rename_commands_move_content() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-file-test-{unique}"));
        fs::create_dir_all(&root).expect("create test dir");
        let source = root.join("source.md");
        let copy = root.join("copy.md");
        let renamed = root.join("renamed.md");
        fs::write(&source, "hello").expect("write source");

        let duplicated = duplicate_file(DuplicateFileRequest {
            from: path_to_string(&source),
            to: path_to_string(&copy),
        })
        .expect("duplicate file");
        assert_eq!(duplicated.text, "hello");

        let metadata = rename_file(RenameFileRequest {
            from: path_to_string(&copy),
            to: path_to_string(&renamed),
        })
        .expect("rename file");
        assert!(metadata.exists);
        assert!(renamed.exists());
        fs::remove_dir_all(root).expect("clean test dir");
    }

    #[test]
    fn save_file_rejects_stale_expected_hash() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-save-conflict-test-{unique}"));
        fs::create_dir_all(&root).expect("create test dir");
        let doc = root.join("doc.md");
        fs::write(&doc, "external").expect("write external content");

        let result = save_file(SaveFileRequest {
            path: path_to_string(&doc),
            text: "local".to_string(),
            expected_hash: Some(sha256_hex(b"old")),
        });

        assert!(result
            .expect_err("stale save should fail")
            .contains("File changed on disk"));
        assert_eq!(fs::read_to_string(&doc).expect("read doc"), "external");
        fs::remove_dir_all(root).expect("clean save conflict test dir");
    }

    #[test]
    fn stable_file_ipc_aliases_open_save_as_and_watch_paths() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-ipc-alias-test-{unique}"));
        fs::create_dir_all(root.join("chapters")).expect("create chapters");
        let doc = root.join("doc.md");
        let included = root.join("chapters").join("intro.md");
        let copy = root.join("copy.md");
        fs::write(&doc, "# Root").expect("write root");
        fs::write(&included, "# Intro").expect("write include");

        let opened = open_file(path_to_string(&doc)).expect("open file alias");
        assert_eq!(opened.text, "# Root");

        let saved = save_file_as(SaveFileRequest {
            path: path_to_string(&copy),
            text: "# Copy".to_string(),
            expected_hash: Some("stale-hash-ignored-for-save-as".to_string()),
        })
        .expect("save file as alias");
        assert_eq!(saved.text, "# Copy");

        let watched = watch_file(WatchFileRequest {
            root: path_to_string(&doc),
            included: vec![path_to_string(&included), path_to_string(&included)],
        })
        .expect("watch file command");
        assert_eq!(watched.paths.len(), 2);
        assert!(watched.paths.iter().all(|metadata| metadata.exists));
        assert!(watched
            .paths
            .iter()
            .any(|metadata| metadata.path.ends_with("chapters/intro.md")));
        fs::remove_dir_all(root).expect("clean ipc alias test dir");
    }

    #[cfg(feature = "native-watch")]
    #[test]
    fn notify_watcher_ignores_access_only_events() {
        assert!(!notify_event_should_emit(&notify::EventKind::Access(
            notify::event::AccessKind::Any
        )));
        assert!(notify_event_should_emit(&notify::EventKind::Modify(
            notify::event::ModifyKind::Any
        )));
    }

    #[test]
    fn workspace_listing_skips_hidden_and_build_artifacts() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-workspace-test-{unique}"));
        fs::create_dir_all(root.join("chapters")).expect("create chapters");
        fs::create_dir_all(root.join(".git")).expect("create hidden dir");
        fs::create_dir_all(root.join("node_modules")).expect("create node modules");
        fs::write(root.join("root.md"), "# Root").expect("write root doc");
        fs::write(root.join("chapters").join("intro.md"), "# Intro").expect("write child doc");
        fs::write(root.join(".secret.md"), "# Secret").expect("write hidden file");
        fs::write(root.join("node_modules").join("package.md"), "# Dependency")
            .expect("write ignored dependency doc");
        fs::write(root.join("binary.bin"), [0, 1, 2, 3]).expect("write binary");

        let entries = list_workspace_files(WorkspaceFileRequest {
            root: path_to_string(&root),
        })
        .expect("workspace listing");
        let paths = entries
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"root.md"));
        assert!(paths.contains(&"chapters"));
        assert!(paths.contains(&"chapters/intro.md"));
        assert!(!paths.iter().any(|path| path.contains(".secret")));
        assert!(!paths.iter().any(|path| path.contains("node_modules")));
        assert!(!paths.iter().any(|path| path.contains("binary.bin")));
        fs::remove_dir_all(root).expect("clean workspace test dir");
    }

    #[test]
    fn git_history_diff_commit_tag_and_restore_workflow() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-git-test-{unique}"));
        fs::create_dir_all(&root).expect("create git test dir");
        run_git(&root, &["init"]).expect("git init");
        run_git(&root, &["config", "user.email", "neditor@example.test"]).expect("git email");
        run_git(&root, &["config", "user.name", "NEditor Test"]).expect("git name");
        let doc = root.join("doc.md");
        fs::write(&doc, "one\n").expect("write initial doc");
        run_git(&root, &["add", "doc.md"]).expect("git add");
        run_git(&root, &["commit", "-m", "Initial document"]).expect("git commit");
        fs::write(&doc, "two\n").expect("write changed doc");

        let diff = git_diff(GitPathRequest {
            path: path_to_string(&doc),
        })
        .expect("git diff");
        assert!(diff.contains("-one"));
        assert!(diff.contains("+two"));

        commit_document_changes(GitCommitRequest {
            path: path_to_string(&doc),
            message: "Update document".to_string(),
        })
        .expect("commit command");
        let history = git_history(GitPathRequest {
            path: path_to_string(&doc),
        })
        .expect("history command");
        assert!(history.len() >= 2);

        let tag = tag_release(GitTagRequest {
            path: path_to_string(&doc),
            tag: format!("test-{unique}"),
            message: "Test release".to_string(),
        })
        .expect("tag command");
        assert!(tag.starts_with("test-"));

        let restored = restore_git_revision(GitRestoreRequest {
            path: path_to_string(&doc),
            revision: history
                .last()
                .expect("initial history entry")
                .revision
                .clone(),
        })
        .expect("restore revision");
        assert_eq!(restored.text, "one\n");
        fs::remove_dir_all(root).expect("clean git test dir");
    }

    #[test]
    fn ai_cleanup_normalizes_chat_artifacts() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "ChatGPT said:\n• First\tSecond\nA\tB".to_string(),
            add_provenance: true,
            mark_as_draft: true,
        });

        assert!(response.cleaned_markdown.contains("- First"));
        assert!(response.cleaned_markdown.contains("| A | B |"));
        assert!(response.cleaned_markdown.contains("```ai-source"));
        assert!(response.issues.len() >= 3);
    }

    #[test]
    fn ai_cleanup_respects_preview_options() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "Assistant:\nClean paragraph.".to_string(),
            add_provenance: false,
            mark_as_draft: false,
        });

        assert!(!response.cleaned_markdown.contains("draft: AI paste"));
        assert!(!response.cleaned_markdown.contains("```ai-source"));
        assert!(response.provenance_block.is_none());
    }
}
