use chrono::Utc;
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
    time::{Duration, Instant},
};
use tauri::Manager;

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
    let mut response = compile(CompileRequest {
        text: request.text,
        file_path: request.file_path,
    });
    response.export_manifest.export_target = request.target.clone();
    response.export_manifest.export_options = request.options.clone();
    validate_export_settings(&request.target, &request.options, &mut response.diagnostics);
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
    if options
        .get("includeManifest")
        .is_some_and(|value| !value.is_boolean())
    {
        diagnostics.push(diag(
            "error",
            "includeManifest must be true or false.",
            None,
            None,
            Some("Use a boolean includeManifest export option."),
        ));
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
        transform_engine("chart", "rust-native-svg", true, false),
        transform_engine("mermaid", "static-diagnostic", true, false),
        transform_engine("pikchr", "external-sidecar", false, true),
        transform_engine("dot", "external-sidecar", false, true),
        transform_engine("graphviz", "external-sidecar", false, true),
        transform_engine("plantuml", "external-sidecar", false, true),
        transform_engine("d2", "external-sidecar", false, true),
        transform_engine("vega-lite", "static-diagnostic", true, false),
        transform_engine("geojson", "static-diagnostic", true, false),
        transform_engine("topojson", "static-diagnostic", true, false),
        transform_engine("stl", "static-diagnostic", true, false),
        transform_engine("openapi", "rust-native", true, false),
        transform_engine("json-schema", "rust-native", true, false),
        transform_engine("bibtex", "static-diagnostic", true, false),
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
    let (metadata, body, body_start_line) =
        parse_front_matter(&source, &mut diagnostics, Some(root_file.clone()));
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
    let glossary = collect_glossary(&interpolated);
    let citations = collect_citations(&interpolated);
    let labels = collect_labels(&interpolated);
    let cross_references = collect_cross_references(&interpolated, &labels, &mut diagnostics);
    let index_terms = collect_index_terms(&interpolated, &headings, &glossary);
    let layout_directives = collect_fence_bodies(&interpolated, "layout");
    let comments = collect_comments(&interpolated);
    let ai_sources = collect_ai_sources(&interpolated);
    let with_toc = inject_generated_sections(
        &interpolated,
        &metadata,
        &headings,
        &index_terms,
        &bibliography,
    );
    let (transformed_markdown, transform_artifacts) = apply_transforms(&with_toc, &mut diagnostics);
    let table_formula_markdown =
        evaluate_markdown_table_formulas(&transformed_markdown, &mut diagnostics);
    validate_image_paths(
        &table_formula_markdown,
        root_path.as_deref(),
        &mut diagnostics,
    );
    let figure_markdown = render_figures(&table_formula_markdown);
    let equation_markdown = render_equations(&figure_markdown);
    let layout_markdown = render_layout_tokens(&equation_markdown);
    let mut document_ast = build_document_ast(&layout_markdown);
    attach_source_ranges(&mut document_ast, |line, end_line| {
        ast_source_range_for_generated_lines(&source_map, line, end_line)
    });
    let html = markdown_to_html(&layout_markdown);
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
                metadata_lookup(metadata, token).map(value_to_string)
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
        "json" => format!("<pre class=\"transform transform-json\">{}</pre>", escape_html(body)),
        "yaml" => format!("<pre class=\"transform transform-yaml\">{}</pre>", escape_html(body)),
        "glossary" => render_glossary_html(body),
        "layout" => render_layout_block_html(body),
        "timeline" => render_timeline_svg(body),
        "chart" => render_chart_svg(body),
        "openapi" => render_openapi_html(body, &mut artifact_diags, diagnostics),
        "json-schema" => render_json_schema_html(body, &mut artifact_diags, diagnostics),
        "mermaid" | "pikchr" | "dot" | "graphviz" | "plantuml" | "d2" | "vega-lite" | "geojson" | "topojson" | "stl" | "bibtex" => {
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

fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
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
    index_terms: &[String],
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
        let toc = headings
            .iter()
            .map(|heading| {
                format!(
                    "{}- [{}](#{})",
                    "  ".repeat(heading.level.saturating_sub(1)),
                    heading.text,
                    heading.anchor
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        output = output.replace("[TOC]", &format!("## Table of Contents\n\n{toc}"));
        if !text.contains("[TOC]") {
            output = format!("## Table of Contents\n\n{toc}\n\n{output}");
        }
    }
    if output.contains("[INDEX]") {
        let index = index_terms
            .iter()
            .map(|term| format!("- {term}"))
            .collect::<Vec<_>>()
            .join("\n");
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
        .flat_map(|body| {
            if let Ok(entries) = parse_csl_json_bibliography(&body) {
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
                .collect::<Vec<_>>()
        })
        .collect()
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

fn collect_citations(text: &str) -> Vec<String> {
    let mut citations = BTreeSet::new();
    for segment in text.split('[').skip(1) {
        if let Some((inside, _)) = segment.split_once(']') {
            if !inside.contains('@') {
                continue;
            }
            for key in citation_keys_from_bracket(inside) {
                citations.insert(key);
            }
        }
    }
    citations.into_iter().collect()
}

fn citation_keys_from_bracket(text: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find('@') {
        let after_at = &rest[index + 1..];
        let key = after_at
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':'))
            .collect::<String>();
        if !key.is_empty() {
            keys.push(key);
        }
        rest = &after_at[keys.last().map(String::len).unwrap_or(0)..];
    }
    keys
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

fn collect_index_terms(
    text: &str,
    headings: &[Heading],
    glossary: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut terms = BTreeSet::new();
    for heading in headings {
        terms.insert(heading.text.clone());
    }
    for term in glossary.keys() {
        terms.insert(term.clone());
    }
    for segment in text.split("{#index:").skip(1) {
        if let Some((term, _)) = segment.split_once('}') {
            terms.insert(term.trim().to_string());
        }
    }
    for segment in text.split("**").skip(1).step_by(2) {
        let term = segment.trim();
        if !term.is_empty() && term.len() <= 80 {
            terms.insert(term.to_string());
        }
    }
    terms.into_iter().collect()
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
            Some(ReviewComment {
                line: index + 1,
                author: "local".to_string(),
                state: if content.contains("resolved") {
                    "resolved"
                } else {
                    "unresolved"
                }
                .to_string(),
                text: content.trim().to_string(),
            })
        })
        .collect()
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

fn render_chart_svg(body: &str) -> String {
    let values = body
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter_map(|(label, value)| {
            value
                .trim()
                .parse::<f64>()
                .ok()
                .map(|value| (label.trim(), value))
        })
        .collect::<Vec<_>>();
    let max = values
        .iter()
        .map(|(_, value)| *value)
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(1.0);
    let height = 260;
    let width = 760;
    let bar_width = if values.is_empty() {
        1
    } else {
        600 / values.len().max(1)
    };
    let mut svg = format!("<svg class=\"chart\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\">");
    for (index, (label, value)) in values.iter().enumerate() {
        let bar_height = ((*value / max) * 180.0) as usize;
        let x = 80 + index * bar_width;
        let y = 220 - bar_height;
        svg.push_str(&format!("<rect x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{bar_height}\" fill=\"#275DA8\"/><text x=\"{x}\" y=\"242\" font-size=\"12\">{}</text><text x=\"{x}\" y=\"{}\" font-size=\"12\">{value}</text>", bar_width.saturating_sub(10), escape_html(label), y.saturating_sub(8)));
    }
    svg.push_str("</svg>");
    svg
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            read_file,
            save_file,
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
        assert!(response.index_terms.iter().any(|term| term == "ARR"));
        assert_eq!(response.export_manifest.document_version, "1.2.0");
        assert!(response
            .formula_graph
            .iter()
            .any(|formula| formula.name == "profit" && formula.value == Some(60.0)));
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
        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        assert!(pptx.len() > 100);
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide1.xml");
        assert!(pptx_slide.contains("Test Report"));
        assert!(pptx_slide.contains("Page 1 of 1"));
        assert!(
            render_markdown_bundle_bytes(&response, &response.export_manifest)
                .expect("bundle bytes")
                .starts_with(b"PK")
        );
    }

    #[test]
    fn semantic_exporters_map_ast_blocks() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Semantic Export\nstatus: approved\napprovedBy: QA\n---\n# Semantic Exports\nBusiness paragraph.\n\n| Metric | Value |\n| --- | ---: |\n| Total | =SUM(1,2) |\n\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:diagram caption=\"System diagram\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n{{page-break}}\n\n## Appendix\nAfter the break.\n".to_string(),
            file_path: None,
        });
        let options = json!({ "watermark": "DRAFT" });

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading1""#));
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading2""#));
        assert!(docx_document.contains("<w:tbl>"));
        assert!(docx_document.contains(r#"<w:br w:type="page""#));
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
        assert!(slide_three.contains("Continued") || slide_three.contains("Appendix"));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("/Count 3"));
        assert!(pdf_text.contains("System diagram"));
        assert!(pdf_text.contains("After the break."));
    }

    #[test]
    fn export_conformance_fixture_maps_business_features() {
        let response = compile(CompileRequest {
            text: include_str!("../fixtures/export/business_report.md").to_string(),
            file_path: None,
        });
        let options = json!({ "watermark": "APPROVED" });

        assert_eq!(response.semantic.title, "Export Conformance Report");
        assert_eq!(response.semantic.status, "approved");
        assert_eq!(response.export_manifest.document_version, "2.0.0");
        assert!(response
            .semantic
            .citations
            .iter()
            .any(|citation| citation == "porter1985"));
        assert!(response.semantic.glossary.contains_key("ARR"));
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

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf_text.contains("/Count 3"));
        assert!(pdf_text.contains("Export Conformance Report"));
        assert!(pdf_text.contains("Reference architecture"));

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading1""#));
        assert!(docx_document.contains("<w:tbl>"));
        assert!(docx_document.contains(r#"<w:br w:type="page""#));
        assert!(docx_document.contains("Reference architecture"));
        assert!(docx_document.contains("Competitive Advantage"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let pptx_presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
        let pptx_slide_two = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        let pptx_slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
        assert!(pptx_presentation.contains(r#"r:id="rId2""#));
        assert!(pptx_slide_two.contains("Export Conformance Report"));
        assert!(pptx_slide_three.contains("Table: Region | Revenue | Margin"));
        assert!(pptx_slide_three.contains("Reference architecture"));

        let bundle =
            render_markdown_bundle_bytes(&response, &response.export_manifest).expect("bundle");
        let bundled_markdown = zip_entry_text(&bundle, "document.md");
        let bundled_text = zip_entry_text(&bundle, "document.txt");
        let bundled_manifest = zip_entry_text(&bundle, "manifest.json");
        assert!(bundled_markdown.contains("Competitive Advantage"));
        assert!(bundled_text.contains("Figure: fig:architecture: Reference architecture"));
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
            options: json!({ "watermark": 42, "includeManifest": "yes" }),
        });

        assert!(!report.ready);
        assert_eq!(report.error_count, 3);
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
}
