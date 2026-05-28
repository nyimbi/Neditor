use crate::{
    compile_with_options,
    export_commands::{
        export_document, prepare_for_export, ExportReadinessReport, ExportRequest,
        PrepareExportRequest,
    },
    metadata_string, metadata_string_list,
    rfp_import::{import_rfp_source, ImportRfpSourceRequest},
    transform_install::{
        installable_external_transform_engines, transform_handler_installer_plans_for_platform,
        TransformHandlerInstallerPlan,
    },
    CompileRequest, CompileResponse, DocumentDiagnostic,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const APP_BUNDLE_NAME: &str = "NEditor";
const APP_BINARY_NAME: &str = "neditor";
const APP_BUNDLE_ID: &str = "com.neditor.desktop";
const LINUX_DESKTOP_ID: &str = "com.neditor.desktop.desktop";
const SUPPORTED_EXPORT_TARGETS: &[&str] = &[
    "html",
    "pdf",
    "docx",
    "pptx",
    "markdown-bundle",
    "blog",
    "substack",
    "latex",
    "google-docs",
    "epub",
];
const STDOUT_EXPORT_TARGETS: &[&str] = &["html", "latex"];
const TRANSFORM_TEMPLATE_SOURCE: &str = include_str!("../../src/lib/transformTemplates.ts");
const NEW_DOCUMENT_TEMPLATES: &[&str] = &[
    "blank",
    "proposal",
    "rfp",
    "rfp-response",
    "rfq",
    "tender",
    "sow",
    "capability-statement",
    "case-study",
    "report",
    "tutorial",
    "lesson-plan",
    "lesson-content",
    "textbook",
    "technical-textbook",
    "novel",
    "podcast-script",
    "movie-script",
    "business-case",
    "executive-brief",
    "board-decision-memo",
    "policy-brief",
    "research-report",
    "grant-application",
    "standard-operating-procedure",
    "product-requirements-document",
    "project-charter",
    "quarterly-business-review",
    "due-diligence-memo",
    "contract-review-brief",
    "implementation-playbook",
    "incident-postmortem",
    "meeting-decision-pack",
    "market-research-report",
];
const CLI_COMMANDS: &[&str] = &[
    "init",
    "new",
    "open",
    "convert",
    "export",
    "publish",
    "inspect",
    "validate",
    "check",
    "templates",
    "outlines",
    "snippets",
    "parts",
    "transform-templates",
    "xforms",
    "profile",
    "business-profile",
    "rfp",
    "rfp-response",
    "analyze-rfp",
    "targets",
    "handlers",
    "transform-handlers",
    "readiness",
    "release-readiness",
    "evidence",
    "evidence-status",
    "support",
    "support-bundle",
    "completions",
    "default-reader",
    "doctor",
    "help",
    "version",
];
const COMPLETION_SHELLS: &[&str] = &["bash", "zsh", "fish"];

#[derive(Debug, Clone, PartialEq)]
pub struct CliOutcome {
    pub message: String,
    pub exit_code: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DefaultMarkdownReaderRequest {
    pub(crate) enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SupportBundleRequest {
    pub(crate) workspace: Option<String>,
    pub(crate) readiness_report: Option<String>,
    pub(crate) spec_report: Option<String>,
    pub(crate) engine_report: Option<String>,
    pub(crate) evidence_root: Option<String>,
    pub(crate) output: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct DefaultMarkdownReaderResponse {
    pub(crate) platform: String,
    pub(crate) enabled: bool,
    pub(crate) applied: bool,
    pub(crate) supported: bool,
    pub(crate) message: String,
    pub(crate) commands: Vec<String>,
    pub(crate) manual_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct WorkspaceScaffoldStatus {
    workspace: String,
    neditor_directory: String,
    status: String,
    required_files: Vec<String>,
    missing_files: Vec<String>,
    recommended_command: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentTemplateInfo {
    id: &'static str,
    label: &'static str,
    category: &'static str,
    summary: &'static str,
    best_for: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentOutlineInfo {
    id: &'static str,
    label: &'static str,
    category: &'static str,
    summary: &'static str,
    docs_live_type: &'static str,
    best_for: &'static [&'static str],
    outline: &'static [&'static str],
    tags: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceDocumentOutline {
    id: String,
    label: String,
    category: String,
    summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    docs_live_type: Option<String>,
    #[serde(default)]
    best_for: Vec<String>,
    outline: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentOutlineEntry {
    id: String,
    label: String,
    category: String,
    summary: String,
    docs_live_type: String,
    best_for: Vec<String>,
    outline: Vec<String>,
    tags: Vec<String>,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceOutlineLibrary {
    schema: String,
    outlines: Vec<WorkspaceDocumentOutline>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentSnippetInfo {
    id: &'static str,
    label: &'static str,
    kind: &'static str,
    summary: &'static str,
    body: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentSnippetEntry {
    id: String,
    label: String,
    kind: String,
    summary: String,
    body: String,
    source: String,
    path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransformTemplateEntry {
    id: String,
    name: String,
    category: String,
    transform: String,
    summary: String,
    body: String,
    tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct BusinessProfile {
    full_name: String,
    email: String,
    phone: String,
    role_title: String,
    company_name: String,
    company_address: String,
    website: String,
    industry: String,
    default_client_name: String,
    brand_voice: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RfpCliSource {
    kind: String,
    title: String,
    path: Option<String>,
    url: Option<String>,
    extraction_method: String,
    line_count: usize,
    word_count: usize,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RfpCliRequirement {
    id: String,
    text: String,
    category: String,
    source_line: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RfpCliComplianceRow {
    id: String,
    requirement: String,
    category: String,
    compliance_status: String,
    response_section: String,
    suggested_response: String,
    evidence_needed: String,
    owner: String,
    verification: String,
    source_line: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RfpCliVerificationSummary {
    total_requirements: usize,
    compliance_rows: usize,
    all_requirements_mapped: bool,
    rows_needing_evidence: usize,
    checklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RfpCliAnalysis {
    source: RfpCliSource,
    requirements: Vec<RfpCliRequirement>,
    compliance_rows: Vec<RfpCliComplianceRow>,
    verification_summary: RfpCliVerificationSummary,
    capabilities: Vec<String>,
    stated_intent: Vec<String>,
    implied_intent: Vec<String>,
    timelines: Vec<String>,
    budget_hints: Vec<String>,
    evaluation_criteria: Vec<String>,
    mandatory_attachments: Vec<String>,
    risks: Vec<String>,
    questions: Vec<String>,
    completeness_score: u8,
}

pub fn run_cli() -> i32 {
    let args = env::args().collect::<Vec<_>>();
    match run_cli_with_args(&args) {
        Ok(outcome) => {
            if !outcome.message.is_empty() {
                println!("{}", outcome.message);
            }
            outcome.exit_code
        }
        Err(error) => {
            eprintln!("{error}");
            2
        }
    }
}

pub fn run_cli_with_args(args: &[String]) -> Result<CliOutcome, String> {
    run_cli_with_args_and_stdin(args, None)
}

pub(crate) fn run_cli_with_args_and_stdin(
    args: &[String],
    stdin_text: Option<&str>,
) -> Result<CliOutcome, String> {
    let command = args.get(1).map(String::as_str).unwrap_or("--help");
    if command != "--help" && command != "-h" && is_direct_open_candidate(command) {
        return run_open_command(&args[1..]);
    }
    match command {
        "-h" | "--help" | "help" => Ok(CliOutcome {
            message: help_text(),
            exit_code: 0,
        }),
        "-V" | "--version" | "version" => Ok(CliOutcome {
            message: format!("ned {}", env!("CARGO_PKG_VERSION")),
            exit_code: 0,
        }),
        "init" => run_init_command(&args[2..]),
        "new" => run_new_command(&args[2..]),
        "open" => run_open_command(&args[2..]),
        "convert" | "export" => run_convert_command(&args[2..], stdin_text),
        "publish" => run_publish_command(&args[2..], stdin_text),
        "inspect" => run_inspect_command(&args[2..], stdin_text),
        "validate" | "check" => run_validate_command(&args[2..], stdin_text),
        "templates" => run_templates_command(&args[2..]),
        "outlines" => run_outlines_command(&args[2..]),
        "snippets" | "parts" => run_snippets_command(&args[2..]),
        "transform-templates" | "xforms" => run_transform_templates_command(&args[2..]),
        "profile" | "business-profile" => run_profile_command(&args[2..]),
        "rfp" | "rfp-response" | "analyze-rfp" => run_rfp_response_command(&args[2..], stdin_text),
        "targets" => run_list_command("targets", SUPPORTED_EXPORT_TARGETS, &args[2..]),
        "handlers" | "transform-handlers" => run_handlers_command(&args[2..]),
        "readiness" | "release-readiness" => run_readiness_command(&args[2..]),
        "evidence" | "evidence-status" => run_evidence_command(&args[2..]),
        "support" | "support-bundle" => run_support_bundle_command(&args[2..]),
        "completions" | "completion" => run_completions_command(&args[2..]),
        "default-reader" => run_default_reader_command(&args[2..]),
        "doctor" => run_doctor_command(&args[2..]),
        other => Err(format!("Unknown ned command '{other}'.\n\n{}", help_text())),
    }
}

#[tauri::command]
pub(crate) fn pending_cli_open_paths() -> Vec<String> {
    env::args()
        .skip(1)
        .filter(|arg| is_markdown_like_path_argument(arg))
        .filter_map(|arg| canonical_path_string(&PathBuf::from(arg)).ok())
        .collect()
}

#[tauri::command]
pub(crate) fn default_markdown_reader_plan() -> DefaultMarkdownReaderResponse {
    default_markdown_reader_response(false, false)
}

#[tauri::command]
pub(crate) fn configure_default_markdown_reader(
    request: DefaultMarkdownReaderRequest,
) -> DefaultMarkdownReaderResponse {
    default_markdown_reader_response(request.enabled, request.enabled)
}

#[tauri::command]
pub(crate) fn create_support_bundle(request: SupportBundleRequest) -> Result<Value, String> {
    let workspace = request
        .workspace
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let readiness_report_path = request
        .readiness_report
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".tmp/release-readiness/report.json"));
    let spec_report_path = request
        .spec_report
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".tmp/spec-completion/report.json"));
    let engine_report_path = request
        .engine_report
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".tmp/external-engines/probe-report.json"));
    let evidence_root_path = request
        .evidence_root
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".tmp"));
    let output_path = request
        .output
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from);
    let (report, _) = build_support_bundle_report(
        &workspace,
        &readiness_report_path,
        &spec_report_path,
        &engine_report_path,
        &evidence_root_path,
        output_path.as_deref(),
    )?;
    Ok(report)
}

fn run_open_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut dry_run = false;
    let mut json_output = false;
    let mut raw_paths = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            value if value.starts_with('-') => {
                return Err(format!("Unsupported open option '{value}'"));
            }
            value => raw_paths.push(value.to_string()),
        }
    }
    let paths = raw_paths
        .iter()
        .map(|arg| canonical_path_string(&PathBuf::from(arg)))
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() {
        return Err("Usage: ned open <file.md> [more.md] [--dry-run] [--json]".to_string());
    }
    if dry_run {
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-open.v1",
                    "dryRun": true,
                    "opened": false,
                    "count": paths.len(),
                    "paths": paths,
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: format!("Would open {} in NEditor", paths.join(", ")),
            exit_code: 0,
        });
    }
    open_paths_in_neditor(&paths)?;
    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-open.v1",
                "dryRun": false,
                "opened": true,
                "count": paths.len(),
                "paths": paths,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: format!("Opening {} in NEditor", paths.join(", ")),
        exit_code: 0,
    })
}

fn run_init_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut directory: Option<String> = None;
    let mut force = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--force" => force = true,
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            value if value.starts_with('-') => {
                return Err(format!("Unsupported init option '{value}'"));
            }
            value => {
                if directory.is_some() {
                    return Err(
                        "Only one workspace directory can be initialized at a time.".to_string()
                    );
                }
                directory = Some(value.to_string());
            }
        }
        index += 1;
    }
    let root = directory
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let neditor_dir = root.join(".neditor");
    let entries = workspace_init_entries(&root);
    let mut created = Vec::new();
    let mut updated = Vec::new();
    let mut kept = Vec::new();

    if !dry_run {
        fs::create_dir_all(&neditor_dir).map_err(|err| {
            format!(
                "Could not create NEditor workspace directory {}: {err}",
                neditor_dir.display()
            )
        })?;
    }

    for (path, content) in entries {
        let existed = path.exists();
        if dry_run {
            if existed && !force {
                kept.push(path_to_display(&path));
            } else if existed {
                updated.push(path_to_display(&path));
            } else {
                created.push(path_to_display(&path));
            }
            continue;
        }
        if existed && !force {
            kept.push(path_to_display(&path));
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Could not create workspace directory {}: {err}",
                    parent.display()
                )
            })?;
        }
        fs::write(&path, content)
            .map_err(|err| format!("Could not write workspace file {}: {err}", path.display()))?;
        if existed {
            updated.push(path_to_display(&path));
        } else {
            created.push(path_to_display(&path));
        }
    }

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-init.v1",
                "workspace": path_to_display(&root),
                "neditorDirectory": path_to_display(&neditor_dir),
                "dryRun": dry_run,
                "force": force,
                "created": created,
                "updated": updated,
                "kept": kept,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }

    Ok(CliOutcome {
        message: init_text_report(&root, dry_run, force, &created, &updated, &kept),
        exit_code: 0,
    })
}

fn run_new_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut output: Option<String> = None;
    let mut template = "blank".to_string();
    let mut title: Option<String> = None;
    let mut workspace = PathBuf::from(".");
    let mut fill_profile = false;
    let mut should_open = false;
    let mut force = false;
    let mut dry_run = false;
    let mut json_output = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--template" | "-t" => {
                index += 1;
                template = args
                    .get(index)
                    .ok_or_else(|| "--template requires a template name".to_string())?
                    .to_string();
            }
            "--title" => {
                index += 1;
                title = Some(
                    args.get(index)
                        .ok_or_else(|| "--title requires a title".to_string())?
                        .to_string(),
                );
            }
            "--workspace" | "-w" => {
                index += 1;
                workspace = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--workspace requires a directory path".to_string())?,
                );
            }
            "--fill-profile" | "--profile" => fill_profile = true,
            "--open" => should_open = true,
            "--force" => force = true,
            "--dry-run" => dry_run = true,
            "--json" => json_output = true,
            value if value.starts_with('-') => {
                return Err(format!("Unsupported new option '{value}'"));
            }
            value => {
                if output.is_some() {
                    return Err("Only one output document can be created at a time.".to_string());
                }
                output = Some(value.to_string());
            }
        }
        index += 1;
    }
    let output = output.map(PathBuf::from).ok_or_else(|| {
        "Usage: ned new <file.md> --template proposal --title \"Client Proposal\"".to_string()
    })?;
    if !is_markdown_like_output_path(&output) {
        return Err(
            "New documents must use a Markdown extension: .md, .markdown, .mdown, or .mkd"
                .to_string(),
        );
    }
    let resolved_title = title.unwrap_or_else(|| title_from_path(&output));
    let raw_markdown = new_document_markdown(&template, &resolved_title)?;
    let (profile_path, profile) = read_workspace_profile_if_requested(&workspace, fill_profile)?;
    let markdown = profile
        .as_ref()
        .map(|profile| fill_business_profile_placeholders(&raw_markdown, profile))
        .unwrap_or_else(|| raw_markdown.to_string());
    if dry_run {
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-new.v1",
                    "dryRun": true,
                    "created": false,
                    "opened": false,
                    "output": path_to_display(&output),
                    "template": template,
                    "title": resolved_title,
                    "force": force,
                    "workspace": path_to_display(&workspace),
                    "profileApplied": profile.is_some(),
                    "profilePath": path_to_display(&profile_path),
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: format!(
                "Would create {} from template '{}' with title '{}'",
                output.display(),
                template,
                resolved_title
            ) + if profile.is_some() {
                " with saved business profile values"
            } else {
                ""
            },
            exit_code: 0,
        });
    }
    if output.exists() && !force {
        return Err(format!(
            "{} already exists. Use --force to replace it.",
            output.display()
        ));
    }
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Could not create directory {}: {err}", parent.display()))?;
    }
    fs::write(&output, markdown)
        .map_err(|err| format!("Could not write new document {}: {err}", output.display()))?;
    let path = canonical_path_string(&output)?;
    if should_open {
        open_paths_in_neditor(std::slice::from_ref(&path))?;
    }
    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-new.v1",
                "dryRun": false,
                "created": true,
                "opened": should_open,
                "output": path,
                "template": template,
                "title": resolved_title,
                "force": force,
                "workspace": path_to_display(&workspace),
                "profileApplied": profile.is_some(),
                "profilePath": path_to_display(&profile_path),
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: if should_open {
            format!("Created and opened {path}")
        } else {
            format!("Created {path}")
        },
        exit_code: 0,
    })
}

fn run_convert_command(args: &[String], stdin_text: Option<&str>) -> Result<CliOutcome, String> {
    let mut input: Option<String> = None;
    let mut target = "pdf".to_string();
    let mut output: Option<String> = None;
    let mut output_dir: Option<String> = None;
    let mut include_manifest = true;
    let mut options = json!({});
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--to" | "-t" => {
                index += 1;
                target = args
                    .get(index)
                    .ok_or_else(|| "--to requires an export target".to_string())?
                    .to_string();
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(
                    args.get(index)
                        .ok_or_else(|| "--output requires a path".to_string())?
                        .to_string(),
                );
            }
            "--stdout" => {
                if output.as_deref().is_some_and(|path| path != "-") {
                    return Err("--stdout cannot be combined with --output.".to_string());
                }
                output = Some("-".to_string());
            }
            "--output-dir" | "-d" => {
                index += 1;
                output_dir = Some(
                    args.get(index)
                        .ok_or_else(|| "--output-dir requires a directory".to_string())?
                        .to_string(),
                );
            }
            "--no-manifest" => include_manifest = false,
            "--option" => {
                index += 1;
                let pair = args
                    .get(index)
                    .ok_or_else(|| "--option requires key=value".to_string())?;
                apply_cli_option(&mut options, pair)?;
            }
            value => {
                if value.starts_with('-') && value != "-" {
                    return Err(format!("Unsupported convert option '{value}'"));
                }
                if input.is_some() {
                    return Err("Only one input document can be converted at a time.".to_string());
                }
                input = Some(value.to_string());
            }
        }
        index += 1;
    }
    let targets = parse_export_targets(&target)?;
    if targets.len() > 1 && output.is_some() {
        return Err(
            "Use --output-dir for multi-target conversion; --output is only valid for one target."
                .to_string(),
        );
    }
    let output_to_stdout = output.as_deref() == Some("-");
    if output_to_stdout {
        if targets.len() != 1 {
            return Err("--stdout supports exactly one export target.".to_string());
        }
        if output_dir.is_some() {
            return Err("--stdout cannot be combined with --output-dir.".to_string());
        }
        if !is_stdout_export_target(&targets[0]) {
            return Err(format!(
                "--stdout is only supported for text export targets: {}",
                STDOUT_EXPORT_TARGETS.join(", ")
            ));
        }
        include_manifest = false;
    }
    let input_arg = input.ok_or_else(|| {
        "Usage: ned convert <file.md> --to pdf,docx --output-dir exports".to_string()
    })?;
    let (text, file_path, input_path) = read_cli_input_document(&input_arg, stdin_text)?;
    let output_dir = output_dir.map(PathBuf::from);
    if let Some(directory) = output_dir.as_ref() {
        fs::create_dir_all(directory).map_err(|err| {
            format!(
                "Could not create output directory {}: {err}",
                directory.display()
            )
        })?;
    }
    let mut object = options.as_object().cloned().unwrap_or_default();
    object.insert("includeManifest".to_string(), Value::Bool(include_manifest));
    options = Value::Object(object);

    let mut messages = Vec::new();
    let include_target_suffix = targets.len() > 1;
    for target in targets {
        let output_path = if output_to_stdout {
            stdout_temp_output_path(&target)
        } else {
            target_output_path(
                &input_path,
                &target,
                output.as_ref(),
                output_dir.as_ref(),
                include_target_suffix,
            )
        };
        let response = export_document(ExportRequest {
            text: text.clone(),
            file_path: file_path.clone(),
            target: target.clone(),
            output_path: output_path.to_string_lossy().to_string(),
            options: options.clone(),
        })?;
        if output_to_stdout {
            let payload = fs::read_to_string(&response.output_path)
                .map_err(|err| format!("Could not read stdout export: {err}"))?;
            let _ = fs::remove_file(&response.output_path);
            return Ok(CliOutcome {
                message: payload,
                exit_code: 0,
            });
        }
        messages.push(format!(
            "Exported {target} to {}{}",
            response.output_path,
            response
                .manifest_path
                .map(|path| format!(" with manifest {path}"))
                .unwrap_or_default()
        ));
    }
    Ok(CliOutcome {
        message: messages.join("\n"),
        exit_code: 0,
    })
}

fn run_publish_command(args: &[String], stdin_text: Option<&str>) -> Result<CliOutcome, String> {
    let mut input: Option<String> = None;
    let mut export_target = "blog".to_string();
    let mut destination_kind = "generic-webhook".to_string();
    let mut endpoint_url = String::new();
    let mut content_format = "html".to_string();
    let mut auth_header_name = "Authorization".to_string();
    let mut token_env = String::new();
    let mut output: Option<String> = None;
    let mut json_output = false;
    let mut allow_not_ready = false;
    let mut options = json!({});
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--target" | "--to" | "-t" => {
                index += 1;
                export_target = args
                    .get(index)
                    .ok_or_else(|| "--target requires blog, substack, or html".to_string())?
                    .to_string();
            }
            "--destination" | "--kind" => {
                index += 1;
                destination_kind = args
                    .get(index)
                    .ok_or_else(|| "--destination requires generic-webhook, wordpress-rest, ghost-admin, or substack-manual".to_string())?
                    .to_string();
            }
            "--endpoint" => {
                index += 1;
                endpoint_url = args
                    .get(index)
                    .ok_or_else(|| "--endpoint requires a URL".to_string())?
                    .to_string();
            }
            "--format" => {
                index += 1;
                content_format = args
                    .get(index)
                    .ok_or_else(|| "--format requires html, markdown, or text".to_string())?
                    .to_string();
            }
            "--auth-header" => {
                index += 1;
                auth_header_name = args
                    .get(index)
                    .ok_or_else(|| "--auth-header requires a header name".to_string())?
                    .to_string();
            }
            "--token-env" => {
                index += 1;
                token_env = args
                    .get(index)
                    .ok_or_else(|| "--token-env requires an environment variable name".to_string())?
                    .to_string();
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(
                    args.get(index)
                        .ok_or_else(|| "--output requires a path".to_string())?
                        .to_string(),
                );
            }
            "--json" => json_output = true,
            "--allow-not-ready" => allow_not_ready = true,
            "--option" => {
                index += 1;
                let pair = args
                    .get(index)
                    .ok_or_else(|| "--option requires key=value".to_string())?;
                apply_cli_option(&mut options, pair)?;
            }
            value => {
                if value.starts_with('-') && value != "-" {
                    return Err(format!("Unsupported publish option '{value}'"));
                }
                if input.is_some() {
                    return Err("Only one input document can be published at a time.".to_string());
                }
                input = Some(value.to_string());
            }
        }
        index += 1;
    }

    validate_publish_target(&export_target)?;
    validate_publish_destination(&destination_kind)?;
    validate_publish_content_format(&content_format)?;
    validate_publish_auth_header(&auth_header_name)?;
    validate_publish_token_env(&token_env)?;
    if !endpoint_url.trim().is_empty() && !publish_endpoint_is_allowed(&endpoint_url) {
        return Err("Publishing endpoint must use HTTPS, or HTTP only for localhost/private development endpoints.".to_string());
    }

    let input_arg = input.ok_or_else(|| {
        "Usage: ned publish <file.md|-> --target blog --endpoint https://cms.example/hook --json".to_string()
    })?;
    let (text, file_path, input_path) = read_cli_input_document(&input_arg, stdin_text)?;
    let mut options_object = options.as_object().cloned().unwrap_or_default();
    options_object.insert("includeManifest".to_string(), Value::Bool(true));
    let options = Value::Object(options_object);

    let response = compile_with_options(
        CompileRequest {
            text: text.clone(),
            file_path: file_path.clone(),
        },
        &options,
    );
    let readiness = prepare_for_export(PrepareExportRequest {
        text,
        file_path,
        target: export_target.clone(),
        options: options.clone(),
    });
    if !allow_not_ready && readiness.error_count > 0 {
        return Err(format!(
            "Publish payload blocked by {} export readiness error(s). Re-run with --allow-not-ready to inspect the payload.",
            readiness.error_count
        ));
    }

    let payload = build_cli_publish_payload(
        &response,
        &readiness,
        &CliPublishPayloadOptions {
            input_path,
            export_target,
            destination_kind,
            endpoint_url,
            content_format,
            auth_header_name,
            token_env,
        },
    );
    let payload_text = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    let output_path = if let Some(path) = output {
        fs::write(&path, &payload_text)
            .map_err(|err| format!("Could not write publishing payload {path}: {err}"))?;
        Some(path)
    } else {
        None
    };

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-publish.v1",
                "ready": readiness.ready,
                "output": output_path,
                "payload": payload,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: cli_publish_text_report(&payload, output_path.as_deref()),
        exit_code: 0,
    })
}

fn run_validate_command(args: &[String], stdin_text: Option<&str>) -> Result<CliOutcome, String> {
    let mut input: Option<String> = None;
    let mut target = "pdf".to_string();
    let mut json_output = false;
    let mut strict = false;
    let mut options = json!({});
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--to" | "-t" => {
                index += 1;
                target = args
                    .get(index)
                    .ok_or_else(|| "--to requires an export target".to_string())?
                    .to_string();
            }
            "--json" => json_output = true,
            "--strict" => strict = true,
            "--option" => {
                index += 1;
                let pair = args
                    .get(index)
                    .ok_or_else(|| "--option requires key=value".to_string())?;
                apply_cli_option(&mut options, pair)?;
            }
            value => {
                if value.starts_with('-') && value != "-" {
                    return Err(format!("Unsupported validate option '{value}'"));
                }
                if input.is_some() {
                    return Err("Only one input document can be validated at a time.".to_string());
                }
                input = Some(value.to_string());
            }
        }
        index += 1;
    }
    let targets = parse_export_targets(&target)?;
    if targets.len() != 1 {
        return Err("Validate checks exactly one export target at a time.".to_string());
    }
    let input_arg =
        input.ok_or_else(|| "Usage: ned validate <file.md|-> --to pdf [--json]".to_string())?;
    let (text, file_path, _) = read_cli_input_document(&input_arg, stdin_text)?;
    let report = prepare_for_export(PrepareExportRequest {
        text,
        file_path,
        target: targets[0].clone(),
        options,
    });
    let exit_code = if report.error_count > 0 || (strict && report.warning_count > 0) {
        1
    } else {
        0
    };
    if json_output {
        let message = serde_json::to_string_pretty(&json!({
            "schema": "neditor.ned-validate.v1",
            "target": targets[0],
            "strict": strict,
            "ready": report.ready,
            "exitCode": exit_code,
            "errorCount": report.error_count,
            "warningCount": report.warning_count,
            "infoCount": report.info_count,
            "diagnostics": report.diagnostics,
            "manifest": report.manifest,
            "progressSteps": report.progress_steps,
        }))
        .map_err(|err| err.to_string())?;
        return Ok(CliOutcome { message, exit_code });
    }
    Ok(CliOutcome {
        message: validate_text_report(&targets[0], strict, &report),
        exit_code,
    })
}

fn run_inspect_command(args: &[String], stdin_text: Option<&str>) -> Result<CliOutcome, String> {
    let mut input: Option<String> = None;
    let mut json_output = false;
    let mut options = json!({});
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--option" => {
                index += 1;
                let pair = args
                    .get(index)
                    .ok_or_else(|| "--option requires key=value".to_string())?;
                apply_cli_option(&mut options, pair)?;
            }
            value => {
                if value.starts_with('-') && value != "-" {
                    return Err(format!("Unsupported inspect option '{value}'"));
                }
                if input.is_some() {
                    return Err("Only one input document can be inspected at a time.".to_string());
                }
                input = Some(value.to_string());
            }
        }
        index += 1;
    }
    let input_arg = input.ok_or_else(|| "Usage: ned inspect <file.md|-> [--json]".to_string())?;
    let (text, file_path, input_path) = read_cli_input_document(&input_arg, stdin_text)?;
    let source_line_count = text.lines().count();
    let source_word_count = count_words(&text);
    let response = compile_with_options(CompileRequest { text, file_path }, &options);
    let (error_count, warning_count, info_count) = diagnostic_counts(&response.diagnostics);
    let exit_code = if error_count > 0 { 1 } else { 0 };
    let document_type = metadata_string(&response.metadata, "documentType")
        .or_else(|| metadata_string(&response.metadata, "document_type"));
    let source_path = if input_arg == "-" {
        None
    } else {
        Some(input_path.clone())
    };

    if json_output {
        let message = serde_json::to_string_pretty(&json!({
            "schema": "neditor.ned-inspect.v1",
            "source": input_path,
            "sourcePath": source_path,
            "exitCode": exit_code,
            "document": {
                "title": response.semantic.title,
                "status": response.semantic.status,
                "documentType": document_type,
                "version": response.export_manifest.document_version,
                "sourceHash": response.export_manifest.source_hash,
                "appVersion": response.export_manifest.app_version,
            },
            "counts": {
                "words": source_word_count,
                "sourceLines": source_line_count,
                "compiledLines": response.compiled_markdown.lines().count(),
                "headings": response.semantic.headings.len(),
                "outlineItems": response.semantic.outline.len(),
                "tables": response.semantic.tables,
                "figures": response.semantic.figures,
                "equations": response.semantic.equations,
                "citations": response.semantic.citations.len(),
                "glossaryTerms": response.semantic.glossary.len(),
                "comments": response.semantic.comments.len(),
                "changeNotes": response.semantic.change_notes.len(),
                "aiSources": response.semantic.ai_sources.len(),
                "aiAssistedSections": response.semantic.ai_assisted_sections.len(),
                "crossReferences": response.semantic.cross_references.len(),
                "includes": response.include_graph.len(),
                "sourceMapEntries": response.source_map.len(),
                "formulas": response.formula_graph.len(),
                "formulaDependencies": response.formula_dependency_edges.len(),
                "transformArtifacts": response.transform_artifacts.len(),
                "diagnostics": {
                    "errors": error_count,
                    "warnings": warning_count,
                    "info": info_count,
                },
            },
            "headings": response.semantic.headings,
            "outline": response.semantic.outline,
            "includeGraph": response.include_graph,
            "diagnostics": response.diagnostics,
            "transformArtifacts": response.transform_artifacts,
            "exportTargets": SUPPORTED_EXPORT_TARGETS,
        }))
        .map_err(|err| err.to_string())?;
        return Ok(CliOutcome { message, exit_code });
    }

    Ok(CliOutcome {
        message: inspect_text_report(
            &input_path,
            document_type.as_deref(),
            source_word_count,
            source_line_count,
            error_count,
            warning_count,
            info_count,
            &response,
        ),
        exit_code,
    })
}

fn run_default_reader_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut enable = false;
    let mut status_only = args.is_empty();
    let mut json_output = false;
    for arg in args {
        match arg.as_str() {
            "--enable" => enable = true,
            "--status" => status_only = true,
            "--json" => json_output = true,
            other => return Err(format!("Unsupported default-reader option '{other}'")),
        }
    }
    if enable {
        status_only = false;
    }
    let response = default_markdown_reader_response(enable, enable);
    let exit_code = if enable && !response.applied { 1 } else { 0 };
    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&default_reader_json_report(
                &response,
                enable,
                status_only,
            ))
            .map_err(|err| err.to_string())?,
            exit_code,
        });
    }
    Ok(CliOutcome {
        message: default_reader_message(&response),
        exit_code,
    })
}

fn run_templates_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut ids_only = false;
    let mut markdown_id: Option<String> = None;
    let mut title: Option<String> = None;
    let mut category: Option<String> = None;
    let mut query: Option<String> = None;
    let mut workspace = PathBuf::from(".");
    let mut fill_profile = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--ids-only" => ids_only = true,
            "--workspace" | "-w" => {
                index += 1;
                workspace = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--workspace requires a directory path".to_string())?,
                );
            }
            "--fill-profile" | "--profile" => fill_profile = true,
            "--markdown" | "--body" => {
                index += 1;
                markdown_id = Some(
                    args.get(index)
                        .ok_or_else(|| "--markdown requires a template id".to_string())?
                        .to_string(),
                );
            }
            "--title" => {
                index += 1;
                title = Some(
                    args.get(index)
                        .ok_or_else(|| "--title requires a document title".to_string())?
                        .to_string(),
                );
            }
            "--category" => {
                index += 1;
                category = Some(
                    args.get(index)
                        .ok_or_else(|| "--category requires a category name".to_string())?
                        .to_string(),
                );
            }
            "--query" | "--search" => {
                index += 1;
                query = Some(
                    args.get(index)
                        .ok_or_else(|| "--query requires search text".to_string())?
                        .to_string(),
                );
            }
            value => return Err(format!("Unsupported templates option '{value}'")),
        }
        index += 1;
    }
    if fill_profile && markdown_id.is_none() {
        return Err("--fill-profile is only valid with templates --markdown <id>.".to_string());
    }

    let category_filter = category
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let query_filter = query
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let templates = document_template_catalog()
        .into_iter()
        .filter(|template| {
            category_filter.as_deref().map_or(true, |category| {
                template.category.to_ascii_lowercase() == category
            })
        })
        .filter(|template| {
            query_filter
                .as_deref()
                .map_or(true, |query| template_matches_query(template, query))
        })
        .collect::<Vec<_>>();
    let ids = templates
        .iter()
        .map(|template| template.id)
        .collect::<Vec<_>>();

    if let Some(markdown_id) = markdown_id {
        let normalized = markdown_id.trim().to_ascii_lowercase();
        let template = document_template_catalog()
            .into_iter()
            .find(|template| template.id == normalized)
            .ok_or_else(|| {
                format!(
                    "Unknown template '{}'. Available templates: {}",
                    markdown_id,
                    document_template_catalog()
                        .iter()
                        .map(|template| template.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        let resolved_title = title.unwrap_or_else(|| template.label.to_string());
        let raw_markdown = new_document_markdown(template.id, &resolved_title)?;
        let (profile_path, profile) =
            read_workspace_profile_if_requested(&workspace, fill_profile)?;
        let markdown = profile
            .as_ref()
            .map(|profile| fill_business_profile_placeholders(&raw_markdown, profile))
            .unwrap_or_else(|| raw_markdown.to_string());
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-template.v1",
                    "template": template.id,
                    "title": resolved_title,
                    "templateDetails": template,
                    "markdown": markdown,
                    "rawMarkdown": raw_markdown,
                    "workspace": path_to_display(&workspace),
                    "profileApplied": profile.is_some(),
                    "profilePath": path_to_display(&profile_path),
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: markdown,
            exit_code: 0,
        });
    }

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-templates.v1",
                "count": templates.len(),
                "filters": {
                    "category": category_filter,
                    "query": query_filter,
                },
                "templates": ids,
                "templateDetails": templates,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    if ids_only {
        return Ok(CliOutcome {
            message: ids.join("\n"),
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: templates_text_report(&templates),
        exit_code: 0,
    })
}

fn run_outlines_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut ids_only = false;
    let mut markdown_id: Option<String> = None;
    let mut save_id: Option<String> = None;
    let mut delete_id: Option<String> = None;
    let mut category: Option<String> = None;
    let mut query: Option<String> = None;
    let mut workspace = PathBuf::from(".");
    let mut label: Option<String> = None;
    let mut summary: Option<String> = None;
    let mut docs_live_type: Option<String> = None;
    let mut outline_file: Option<String> = None;
    let mut sections: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    let mut best_for: Vec<String> = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--ids-only" => ids_only = true,
            "--workspace" | "-w" => {
                index += 1;
                workspace = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--workspace requires a directory path".to_string())?,
                );
            }
            "--markdown" | "--body" => {
                index += 1;
                markdown_id = Some(
                    args.get(index)
                        .ok_or_else(|| "--markdown requires an outline id".to_string())?
                        .to_string(),
                );
            }
            "--save" => {
                index += 1;
                save_id = Some(
                    args.get(index)
                        .ok_or_else(|| "--save requires an outline id".to_string())?
                        .to_string(),
                );
            }
            "--delete" => {
                index += 1;
                delete_id = Some(
                    args.get(index)
                        .ok_or_else(|| "--delete requires an outline id".to_string())?
                        .to_string(),
                );
            }
            "--name" | "--label" => {
                index += 1;
                label = Some(
                    args.get(index)
                        .ok_or_else(|| "--name requires an outline name".to_string())?
                        .to_string(),
                );
            }
            "--category" => {
                index += 1;
                category = Some(
                    args.get(index)
                        .ok_or_else(|| "--category requires a category name".to_string())?
                        .to_string(),
                );
            }
            "--summary" => {
                index += 1;
                summary = Some(
                    args.get(index)
                        .ok_or_else(|| "--summary requires text".to_string())?
                        .to_string(),
                );
            }
            "--docs-live-type" | "--document-type" => {
                index += 1;
                docs_live_type = Some(
                    args.get(index)
                        .ok_or_else(|| {
                            "--docs-live-type requires a Docs Live document type".to_string()
                        })?
                        .to_string(),
                );
            }
            "--outline-file" => {
                index += 1;
                outline_file = Some(
                    args.get(index)
                        .ok_or_else(|| "--outline-file requires a path".to_string())?
                        .to_string(),
                );
            }
            "--section" => {
                index += 1;
                sections.push(
                    args.get(index)
                        .ok_or_else(|| "--section requires heading text".to_string())?
                        .to_string(),
                );
            }
            "--tag" => {
                index += 1;
                tags.push(
                    args.get(index)
                        .ok_or_else(|| "--tag requires text".to_string())?
                        .to_string(),
                );
            }
            "--best-for" => {
                index += 1;
                best_for.push(
                    args.get(index)
                        .ok_or_else(|| "--best-for requires text".to_string())?
                        .to_string(),
                );
            }
            "--query" | "--search" => {
                index += 1;
                query = Some(
                    args.get(index)
                        .ok_or_else(|| "--query requires search text".to_string())?
                        .to_string(),
                );
            }
            value => return Err(format!("Unsupported outlines option '{value}'")),
        }
        index += 1;
    }

    if save_id.is_some() && delete_id.is_some() {
        return Err("Use either --save or --delete, not both.".to_string());
    }
    if save_id.is_some() && markdown_id.is_some() {
        return Err("Use either --save or --markdown, not both.".to_string());
    }
    if delete_id.is_some() && markdown_id.is_some() {
        return Err("Use either --delete or --markdown, not both.".to_string());
    }

    if let Some(id) = save_id {
        let saved = save_workspace_outline(
            &workspace,
            WorkspaceOutlineSaveInput {
                id,
                label,
                category,
                summary,
                docs_live_type,
                outline_file,
                sections,
                tags,
                best_for,
            },
        )?;
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-outline-save.v1",
                    "workspace": path_to_display(&workspace),
                    "libraryPath": path_to_display(&workspace_outline_library_path(&workspace)),
                    "outline": saved,
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: format!(
                "Saved outline {} to {}",
                saved.id,
                path_to_display(&workspace_outline_library_path(&workspace))
            ),
            exit_code: 0,
        });
    }

    if let Some(id) = delete_id {
        let deleted = delete_workspace_outline(&workspace, &id)?;
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-outline-delete.v1",
                    "workspace": path_to_display(&workspace),
                    "libraryPath": path_to_display(&workspace_outline_library_path(&workspace)),
                    "outline": id,
                    "deleted": deleted,
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: if deleted {
                format!("Deleted outline {id}")
            } else {
                format!("No workspace outline named {id} was found")
            },
            exit_code: 0,
        });
    }

    let category_filter = category
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let query_filter = query
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let catalog = document_outline_catalog_entries(&workspace)?;
    let outlines = catalog
        .into_iter()
        .filter(|outline| {
            category_filter.as_deref().map_or(true, |category| {
                outline.category.to_ascii_lowercase() == category
            })
        })
        .filter(|outline| {
            query_filter
                .as_deref()
                .map_or(true, |query| outline_matches_query(outline, query))
        })
        .collect::<Vec<_>>();

    if let Some(id) = markdown_id {
        let catalog = document_outline_catalog_entries(&workspace)?;
        let outline = find_outline_by_id_or_alias(catalog, &id).ok_or_else(|| {
            format!(
                "Unknown outline '{}'. Available outlines: {}",
                id,
                document_outline_catalog_entries(&workspace)
                    .unwrap_or_default()
                    .iter()
                    .map(|outline| outline.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        let markdown = outline_markdown(&outline);
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-outline-markdown.v1",
                    "outline": outline.id,
                    "source": outline.source,
                    "markdown": markdown,
                    "sectionCount": outline.outline.len(),
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: markdown,
            exit_code: 0,
        });
    }

    let ids = outlines
        .iter()
        .map(|outline| outline.id.as_str())
        .collect::<Vec<_>>();
    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-outlines.v1",
                "count": outlines.len(),
                "workspace": path_to_display(&workspace),
                "libraryPath": path_to_display(&workspace_outline_library_path(&workspace)),
                "filters": {
                    "category": category_filter,
                    "query": query_filter,
                },
                "outlines": ids,
                "outlineDetails": outlines,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    if ids_only {
        return Ok(CliOutcome {
            message: ids.join("\n"),
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: outlines_text_report(&outlines),
        exit_code: 0,
    })
}

fn run_snippets_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut ids_only = false;
    let mut markdown_id: Option<String> = None;
    let mut kind: Option<String> = None;
    let mut query: Option<String> = None;
    let mut workspace = PathBuf::from(".");
    let mut fill_profile = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--ids-only" => ids_only = true,
            "--workspace" | "-w" => {
                index += 1;
                workspace = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--workspace requires a directory path".to_string())?,
                );
            }
            "--fill-profile" | "--profile" => fill_profile = true,
            "--markdown" | "--body" => {
                index += 1;
                markdown_id = Some(
                    args.get(index)
                        .ok_or_else(|| "--markdown requires a snippet id".to_string())?
                        .to_string(),
                );
            }
            "--kind" => {
                index += 1;
                kind = Some(
                    args.get(index)
                        .ok_or_else(|| "--kind requires a snippet kind".to_string())?
                        .to_string(),
                );
            }
            "--query" | "--search" => {
                index += 1;
                query = Some(
                    args.get(index)
                        .ok_or_else(|| "--query requires search text".to_string())?
                        .to_string(),
                );
            }
            value => return Err(format!("Unsupported snippets option '{value}'")),
        }
        index += 1;
    }

    let snippets = document_snippet_catalog_entries(&workspace)?;
    if let Some(id) = markdown_id {
        let normalized = id.trim().to_ascii_lowercase();
        let snippet = snippets
            .iter()
            .find(|snippet| snippet.id == normalized)
            .ok_or_else(|| {
                format!(
                    "Unknown snippet '{}'. Available snippets: {}",
                    id,
                    snippets
                        .iter()
                        .map(|snippet| snippet.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        let (profile_path, profile) =
            read_workspace_profile_if_requested(&workspace, fill_profile)?;
        let markdown = profile
            .as_ref()
            .map(|profile| fill_business_profile_placeholders(&snippet.body, profile))
            .unwrap_or_else(|| snippet.body.to_string());
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-snippet.v1",
                    "snippet": snippet,
                    "markdown": markdown,
                    "rawMarkdown": snippet.body,
                    "profileApplied": profile.is_some(),
                    "profilePath": path_to_display(&profile_path),
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: markdown,
            exit_code: 0,
        });
    }

    let kind_filter = kind
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let query_filter = query
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let filtered = snippets
        .into_iter()
        .filter(|snippet| {
            kind_filter
                .as_deref()
                .map_or(true, |kind| snippet.kind.to_ascii_lowercase() == kind)
        })
        .filter(|snippet| {
            query_filter
                .as_deref()
                .map_or(true, |query| snippet_matches_query(snippet, query))
        })
        .collect::<Vec<_>>();
    let ids = filtered
        .iter()
        .map(|snippet| snippet.id.as_str())
        .collect::<Vec<_>>();

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-snippets.v1",
                "count": filtered.len(),
                "workspace": path_to_display(&workspace),
                "workspaceSnippetPath": path_to_display(&workspace_snippets_dir(&workspace)),
                "filters": {
                    "kind": kind_filter,
                    "query": query_filter,
                },
                "snippets": ids,
                "snippetDetails": filtered,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    if ids_only {
        return Ok(CliOutcome {
            message: ids.join("\n"),
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: snippets_text_report(&filtered),
        exit_code: 0,
    })
}

fn run_transform_templates_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut ids_only = false;
    let mut category: Option<String> = None;
    let mut transform: Option<String> = None;
    let mut query: Option<String> = None;
    let mut markdown_id: Option<String> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--ids-only" => ids_only = true,
            "--category" => {
                index += 1;
                category = Some(
                    args.get(index)
                        .ok_or_else(|| "--category requires a category name".to_string())?
                        .to_string(),
                );
            }
            "--transform" | "--kind" => {
                index += 1;
                transform = Some(
                    args.get(index)
                        .ok_or_else(|| "--transform requires a transform name".to_string())?
                        .to_string(),
                );
            }
            "--query" | "--search" => {
                index += 1;
                query = Some(
                    args.get(index)
                        .ok_or_else(|| "--query requires search text".to_string())?
                        .to_string(),
                );
            }
            "--markdown" | "--body" => {
                index += 1;
                markdown_id = Some(
                    args.get(index)
                        .ok_or_else(|| "--markdown requires a transform template id".to_string())?
                        .to_string(),
                );
            }
            value => return Err(format!("Unsupported transform-templates option '{value}'")),
        }
        index += 1;
    }

    let templates = transform_template_catalog()?;
    if let Some(id) = markdown_id {
        let template = templates
            .into_iter()
            .find(|template| template.id == id)
            .ok_or_else(|| {
                format!(
                    "Unknown transform template '{}'. Available transform templates: {}",
                    id,
                    transform_template_catalog()
                        .unwrap_or_default()
                        .iter()
                        .map(|template| template.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        if json_output {
            let markdown = template.body.clone();
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-transform-template.v1",
                    "template": template,
                    "markdown": markdown,
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: template.body,
            exit_code: 0,
        });
    }

    let category_filter = category
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let transform_filter = transform
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let query_filter = query
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let filtered = templates
        .into_iter()
        .filter(|template| {
            category_filter.as_deref().map_or(true, |category| {
                template.category.to_ascii_lowercase() == category
            })
        })
        .filter(|template| {
            transform_filter.as_deref().map_or(true, |transform| {
                template.transform.to_ascii_lowercase() == transform
            })
        })
        .filter(|template| {
            query_filter.as_deref().map_or(true, |query| {
                transform_template_matches_query(template, query)
            })
        })
        .collect::<Vec<_>>();
    let ids = filtered
        .iter()
        .map(|template| template.id.as_str())
        .collect::<Vec<_>>();
    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-transform-templates.v1",
                "count": filtered.len(),
                "filters": {
                    "category": category_filter,
                    "transform": transform_filter,
                    "query": query_filter,
                },
                "templates": ids,
                "templateDetails": filtered,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    if ids_only {
        return Ok(CliOutcome {
            message: ids.join("\n"),
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: transform_templates_text_report(&filtered),
        exit_code: 0,
    })
}

fn run_profile_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut workspace = PathBuf::from(".");
    let mut json_output = false;
    let mut markdown_output = false;
    let mut placeholders_output = false;
    let mut fields_output = false;
    let mut get_field: Option<String> = None;
    let mut init = false;
    let mut force = false;
    let mut dry_run = false;
    let mut updates: Vec<(String, String)> = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--workspace" | "-w" => {
                index += 1;
                workspace = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--workspace requires a directory path".to_string())?,
                );
            }
            "--set" => {
                index += 1;
                let raw = args
                    .get(index)
                    .ok_or_else(|| "--set requires key=value".to_string())?;
                updates.push(parse_profile_assignment(raw)?);
            }
            "--json" => json_output = true,
            "--markdown" => markdown_output = true,
            "--placeholders" | "--placeholder-text" => placeholders_output = true,
            "--fields" => fields_output = true,
            "--get" => {
                index += 1;
                get_field = Some(
                    args.get(index)
                        .ok_or_else(|| "--get requires a profile field".to_string())?
                        .to_string(),
                );
            }
            "--init" => init = true,
            "--force" => force = true,
            "--dry-run" => dry_run = true,
            value if value.starts_with('-') => {
                return Err(format!("Unsupported profile option '{value}'"));
            }
            value => updates.push(parse_profile_assignment(value)?),
        }
        index += 1;
    }

    if fields_output {
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-profile-fields.v1",
                    "fields": business_profile_field_catalog(),
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: business_profile_fields_text_report(),
            exit_code: 0,
        });
    }

    let neditor_dir = workspace.join(".neditor");
    let profile_path = neditor_dir.join("business-profile.json");
    let existed = profile_path.exists();
    let mut profile = if existed && !force {
        read_business_profile(&profile_path)?
    } else {
        BusinessProfile::default()
    };
    for (key, value) in &updates {
        set_business_profile_field(&mut profile, key, value)?;
    }
    let should_write = (!existed && init) || force || !updates.is_empty();
    if should_write && !dry_run {
        fs::create_dir_all(&neditor_dir).map_err(|err| {
            format!(
                "Could not create NEditor workspace directory {}: {err}",
                neditor_dir.display()
            )
        })?;
        write_business_profile(&profile_path, &profile)?;
    }

    if let Some(field) = get_field.as_deref() {
        let (canonical, value) = business_profile_field_value(&profile, field)?;
        if json_output {
            return Ok(CliOutcome {
                message: serde_json::to_string_pretty(&json!({
                    "schema": "neditor.ned-profile-value.v1",
                    "workspace": path_to_display(&workspace),
                    "profilePath": path_to_display(&profile_path),
                    "exists": existed,
                    "field": canonical,
                    "value": value,
                    "placeholder": profile_value(&value, canonical),
                }))
                .map_err(|err| err.to_string())?,
                exit_code: 0,
            });
        }
        return Ok(CliOutcome {
            message: profile_value(&value, canonical),
            exit_code: 0,
        });
    }

    if markdown_output && !json_output {
        return Ok(CliOutcome {
            message: business_profile_markdown(&profile),
            exit_code: 0,
        });
    }
    if placeholders_output && !json_output {
        return Ok(CliOutcome {
            message: business_profile_placeholder_text(&profile),
            exit_code: 0,
        });
    }

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-profile.v1",
                "workspace": path_to_display(&workspace),
                "profilePath": path_to_display(&profile_path),
                "exists": existed,
                "initialized": init && !existed,
                "updated": !updates.is_empty(),
                "dryRun": dry_run,
                "written": should_write && !dry_run,
                "profile": profile,
                "placeholderText": business_profile_placeholder_text(&profile),
                "markdown": business_profile_markdown(&profile),
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }

    Ok(CliOutcome {
        message: business_profile_text_report(
            &profile_path,
            existed,
            should_write,
            dry_run,
            &profile,
        ),
        exit_code: 0,
    })
}

fn run_rfp_response_command(
    args: &[String],
    stdin_text: Option<&str>,
) -> Result<CliOutcome, String> {
    let mut source: Option<String> = None;
    let mut source_type: Option<String> = None;
    let mut url: Option<String> = None;
    let mut output: Option<PathBuf> = None;
    let mut matrix_output: Option<PathBuf> = None;
    let mut workspace = PathBuf::from(".");
    let mut context_notes = String::new();
    let mut json_output = false;
    let mut markdown_output = false;
    let mut matrix_only = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--source-type" | "--kind" => {
                index += 1;
                source_type = Some(
                    args.get(index)
                        .ok_or_else(|| {
                            "--source-type requires markdown, pdf, docx, or url".to_string()
                        })?
                        .to_string(),
                );
            }
            "--url" => {
                index += 1;
                url = Some(
                    args.get(index)
                        .ok_or_else(|| "--url requires an http:// or https:// URL".to_string())?
                        .to_string(),
                );
            }
            "--output" | "-o" => {
                index += 1;
                output = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    "--output requires a Markdown output path".to_string()
                })?));
            }
            "--matrix-output" => {
                index += 1;
                matrix_output = Some(PathBuf::from(args.get(index).ok_or_else(|| {
                    "--matrix-output requires a Markdown output path".to_string()
                })?));
            }
            "--workspace" | "-w" => {
                index += 1;
                workspace = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--workspace requires a directory path".to_string())?,
                );
            }
            "--context" | "--notes" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--context requires response guidance text".to_string())?;
                context_notes = append_cli_block(&context_notes, value);
            }
            "--json" => json_output = true,
            "--markdown" => markdown_output = true,
            "--matrix" => matrix_only = true,
            "-" => {
                if source.is_some() {
                    return Err("Only one RFP source can be analyzed at a time.".to_string());
                }
                source = Some("-".to_string());
            }
            value if value.starts_with('-') => {
                return Err(format!("Unsupported RFP option '{value}'"));
            }
            value => {
                if source.is_some() {
                    return Err("Only one RFP source can be analyzed at a time.".to_string());
                }
                source = Some(value.to_string());
            }
        }
        index += 1;
    }

    if url.is_some() && source.is_none() {
        source = url.clone();
    }
    let source = source.ok_or_else(|| {
        "Usage: ned rfp-response <rfp.md|rfp.docx|rfp.pdf|url|-> [--output response.md] [--matrix-output matrix.md] [--json|--markdown|--matrix]"
            .to_string()
    })?;
    let inferred_type =
        source_type.unwrap_or_else(|| infer_rfp_source_type(&source, url.as_deref()));
    let (path, source_url, text) = if source == "-" {
        let text = if let Some(text) = stdin_text {
            text.to_string()
        } else {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .map_err(|err| format!("Could not read RFP source from stdin: {err}"))?;
            text
        };
        (None, url.clone(), Some(text))
    } else if inferred_type == "url" {
        (None, Some(url.unwrap_or_else(|| source.clone())), None)
    } else {
        (Some(source.clone()), url.clone(), None)
    };
    let imported = import_rfp_source(ImportRfpSourceRequest {
        source_type: inferred_type,
        path,
        url: source_url,
        text,
    })?;
    let profile_path = workspace_business_profile_path(&workspace);
    let profile_applied = profile_path.exists();
    let profile = if profile_applied {
        read_business_profile(&profile_path)?
    } else {
        BusinessProfile::default()
    };
    let analysis = analyze_rfp_text(&imported, &profile);
    let matrix_markdown = rfp_cli_compliance_matrix_markdown(&analysis);
    let response_markdown = rfp_cli_response_markdown(&analysis, &profile, &context_notes);

    if let Some(path) = output.as_ref() {
        write_cli_markdown_output(path, &response_markdown)?;
    }
    if let Some(path) = matrix_output.as_ref() {
        write_cli_markdown_output(path, &matrix_markdown)?;
    }

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-rfp-response.v1",
                "analysis": analysis,
                "responseMarkdown": response_markdown,
                "complianceMatrixMarkdown": matrix_markdown,
                "profileApplied": profile_applied,
                "profilePath": path_to_display(&profile_path),
                "outputs": {
                    "response": output.as_ref().map(|path| path_to_display(path)),
                    "matrix": matrix_output.as_ref().map(|path| path_to_display(path)),
                },
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    if matrix_only {
        return Ok(CliOutcome {
            message: matrix_markdown,
            exit_code: 0,
        });
    }
    if markdown_output || output.is_none() {
        return Ok(CliOutcome {
            message: response_markdown,
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: format!(
            "Analyzed {} RFP requirement(s); wrote response to {}{}.",
            analysis.requirements.len(),
            output
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "(stdout)".to_string()),
            matrix_output
                .as_ref()
                .map(|path| format!(" and matrix to {}", path.display()))
                .unwrap_or_default()
        ),
        exit_code: 0,
    })
}

fn run_list_command(kind: &str, values: &[&str], args: &[String]) -> Result<CliOutcome, String> {
    let json_output = args.iter().any(|arg| arg == "--json");
    if let Some(unsupported) = args.iter().find(|arg| !matches!(arg.as_str(), "--json")) {
        return Err(format!("Unsupported {kind} option '{unsupported}'"));
    }
    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": format!("neditor.ned-{kind}.v1"),
                kind: values,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }
    Ok(CliOutcome {
        message: values.join("\n"),
        exit_code: 0,
    })
}

fn run_handlers_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut commands_only = false;
    let mut platform = env::consts::OS.to_string();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--commands-only" => commands_only = true,
            "--platform" => {
                index += 1;
                platform = args
                    .get(index)
                    .ok_or_else(|| {
                        "--platform requires macos, windows, linux, or manual".to_string()
                    })?
                    .to_ascii_lowercase();
            }
            value => return Err(format!("Unsupported handlers option '{value}'")),
        }
        index += 1;
    }
    let plans = transform_handler_installer_plans_for_platform(&platform);
    let registered_engines = installable_external_transform_engines();
    let missing = missing_transform_handler_engines(&plans, &registered_engines);

    if commands_only {
        let commands = plans
            .iter()
            .flat_map(|plan| plan.commands.iter())
            .cloned()
            .collect::<Vec<_>>();
        if commands.is_empty() {
            return Err("No transform handler setup commands are available.".to_string());
        }
        return Ok(CliOutcome {
            message: commands.join("\n"),
            exit_code: 0,
        });
    }

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&json!({
                "schema": "neditor.ned-handlers.v1",
                "platform": platform,
                "registeredEngines": registered_engines,
                "missingRegisteredEngines": missing,
                "plans": plans,
            }))
            .map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }

    Ok(CliOutcome {
        message: handlers_text_report(&platform, &registered_engines, &missing, &plans),
        exit_code: if missing.is_empty() { 0 } else { 1 },
    })
}

fn run_completions_command(args: &[String]) -> Result<CliOutcome, String> {
    let shell = args
        .first()
        .ok_or_else(|| "Usage: ned completions <bash|zsh|fish>".to_string())?
        .as_str();
    if args.len() > 1 {
        return Err("Only one shell can be generated at a time.".to_string());
    }
    let script = match shell {
        "bash" => bash_completion_script(),
        "zsh" => zsh_completion_script(),
        "fish" => fish_completion_script(),
        other => {
            return Err(format!(
                "Unsupported completion shell '{}'. Supported shells: {}",
                other,
                COMPLETION_SHELLS.join(", ")
            ))
        }
    };
    Ok(CliOutcome {
        message: script,
        exit_code: 0,
    })
}

fn run_readiness_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut strict = false;
    let mut report_path = PathBuf::from(".tmp/release-readiness/report.json");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--strict" => strict = true,
            "--report" => {
                index += 1;
                report_path = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--report requires a JSON report path".to_string())?,
                );
            }
            value => return Err(format!("Unsupported readiness option '{value}'")),
        }
        index += 1;
    }

    let report = read_readiness_report(&report_path)?;
    let status = readiness_string_field(&report, "status").unwrap_or("unknown");
    let summary = report.get("summary").cloned().unwrap_or_else(|| json!({}));
    let checks = readiness_array_field(&report, "checks");
    let evidence_gaps = readiness_array_field(&report, "evidenceGaps");
    let failures = readiness_array_field(&report, "failures");
    let release_ready = readiness_release_ready(&report);
    let exit_code = if strict && !release_ready { 1 } else { 0 };
    let report_path_display = path_to_display(&report_path);

    if json_output {
        let normalized = json!({
            "schema": "neditor.ned-readiness.v1",
            "status": status,
            "releaseReady": release_ready,
            "reportPath": report_path_display,
            "generatedAt": readiness_string_field(&report, "generatedAt"),
            "platform": readiness_string_field(&report, "platform"),
            "arch": readiness_string_field(&report, "arch"),
            "summary": summary,
            "checks": checks,
            "evidenceGaps": evidence_gaps,
            "failures": failures,
            "nextCommands": readiness_next_commands(&report),
        });
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&normalized).map_err(|err| err.to_string())?,
            exit_code,
        });
    }

    Ok(CliOutcome {
        message: readiness_text_report(&report, &report_path_display, release_ready),
        exit_code,
    })
}

fn run_evidence_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut strict = false;
    let mut evidence_root_path = PathBuf::from(".tmp");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--strict" => strict = true,
            "--evidence-root" => {
                index += 1;
                evidence_root_path = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--evidence-root requires a directory path".to_string())?,
                );
            }
            value => return Err(format!("Unsupported evidence option '{value}'")),
        }
        index += 1;
    }

    let reports = support_bundle_evidence_reports(&evidence_root_path);
    let summary = support_bundle_evidence_report_summary(&reports);
    let attention = number_field_u64(&summary, "attention");
    let missing = number_field_u64(&summary, "missing");
    let failed = number_field_u64(&summary, "failed");
    let ready = number_field_u64(&summary, "ready");
    let total = number_field_u64(&summary, "total");
    let status = if failed > 0 {
        "failed"
    } else if attention > 0 || missing > 0 {
        "needs-attention"
    } else {
        "ready"
    };
    let report = json!({
        "schema": "neditor.ned-evidence-status.v1",
        "generatedAtUnixSeconds": unix_timestamp_seconds(),
        "status": status,
        "evidenceRoot": path_to_display(&evidence_root_path),
        "summary": summary,
        "reports": reports,
        "nextCommands": evidence_next_commands(status, &evidence_root_path),
    });
    let exit_code = if strict && status != "ready" { 1 } else { 0 };

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
            exit_code,
        });
    }

    Ok(CliOutcome {
        message: evidence_text_report(&report, total, ready, attention, missing, failed),
        exit_code,
    })
}

fn run_support_bundle_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut workspace = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut readiness_report_path = PathBuf::from(".tmp/release-readiness/report.json");
    let mut spec_report_path = PathBuf::from(".tmp/spec-completion/report.json");
    let mut engine_report_path = PathBuf::from(".tmp/external-engines/probe-report.json");
    let mut evidence_root_path = PathBuf::from(".tmp");
    let mut output_path: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--workspace" => {
                index += 1;
                workspace = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--workspace requires a directory path".to_string())?,
                );
            }
            "--readiness-report" => {
                index += 1;
                readiness_report_path =
                    PathBuf::from(args.get(index).ok_or_else(|| {
                        "--readiness-report requires a JSON report path".to_string()
                    })?);
            }
            "--spec-report" => {
                index += 1;
                spec_report_path = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--spec-report requires a JSON report path".to_string())?,
                );
            }
            "--engine-report" => {
                index += 1;
                engine_report_path =
                    PathBuf::from(args.get(index).ok_or_else(|| {
                        "--engine-report requires a JSON report path".to_string()
                    })?);
            }
            "--evidence-root" => {
                index += 1;
                evidence_root_path = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--evidence-root requires a directory path".to_string())?,
                );
            }
            "--output" | "-o" => {
                index += 1;
                output_path =
                    Some(PathBuf::from(args.get(index).ok_or_else(|| {
                        "--output requires a JSON file path".to_string()
                    })?));
            }
            value => return Err(format!("Unsupported support-bundle option '{value}'")),
        }
        index += 1;
    }

    let (report, written_to) = build_support_bundle_report(
        &workspace,
        &readiness_report_path,
        &spec_report_path,
        &engine_report_path,
        &evidence_root_path,
        output_path.as_deref(),
    )?;

    if json_output {
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
            exit_code: 0,
        });
    }

    Ok(CliOutcome {
        message: support_bundle_text_report(&report, written_to.as_deref()),
        exit_code: 0,
    })
}

fn run_doctor_command(args: &[String]) -> Result<CliOutcome, String> {
    let mut json_output = false;
    let mut strict = false;
    let mut workspace = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json_output = true,
            "--strict" => strict = true,
            "--workspace" => {
                index += 1;
                workspace = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--workspace requires a directory path".to_string())?,
                );
            }
            value => return Err(format!("Unsupported doctor option '{value}'")),
        }
        index += 1;
    }
    let current_exe = env::current_exe().ok();
    let app_binary = find_neditor_binary();
    let default_reader = default_markdown_reader_response(false, false);
    let workspace_scaffold = workspace_scaffold_status(&workspace);
    let handler_plans = transform_handler_installer_plans_for_platform(env::consts::OS);
    let registered_engines = installable_external_transform_engines();
    let missing_handler_engines =
        missing_transform_handler_engines(&handler_plans, &registered_engines);
    let warnings = doctor_warnings(
        app_binary.as_ref(),
        &default_reader,
        &workspace_scaffold,
        &missing_handler_engines,
    );
    let status = if warnings.is_empty() {
        "ready"
    } else {
        "warning"
    };
    let exit_code = if strict && !warnings.is_empty() { 1 } else { 0 };
    if json_output {
        let report = json!({
            "schema": "neditor.ned-doctor.v1",
            "status": status,
            "version": env!("CARGO_PKG_VERSION"),
            "platform": env::consts::OS,
            "arch": env::consts::ARCH,
            "cliPath": current_exe.map(|path| path.to_string_lossy().to_string()),
            "appBinary": app_binary.map(|path| path.to_string_lossy().to_string()),
            "defaultReader": default_reader,
            "workspaceScaffold": workspace_scaffold,
            "transformHandlers": {
                "registeredEngines": registered_engines,
                "missingRegisteredEngines": missing_handler_engines,
                "plans": handler_plans,
            },
            "exportTargets": SUPPORTED_EXPORT_TARGETS,
            "templates": NEW_DOCUMENT_TEMPLATES,
            "warnings": warnings,
        });
        return Ok(CliOutcome {
            message: serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
            exit_code,
        });
    }
    let mut lines = vec![
        format!("ned {}", env!("CARGO_PKG_VERSION")),
        format!("Status: {status}"),
        format!("Platform: {}-{}", env::consts::OS, env::consts::ARCH),
        format!(
            "CLI path: {}",
            current_exe
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ),
        format!(
            "NEditor app binary: {}",
            app_binary
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| "not found next to ned".to_string())
        ),
        format!("Default reader automation: {}", default_reader.message),
        format!(
            "Workspace scaffold: {} at {}",
            workspace_scaffold.status, workspace_scaffold.neditor_directory
        ),
        format!(
            "Transform handler setup coverage: {}",
            if missing_handler_engines.is_empty() {
                "all registered engines covered".to_string()
            } else {
                format!("missing {}", missing_handler_engines.join(", "))
            }
        ),
        format!("Export targets: {}", SUPPORTED_EXPORT_TARGETS.join(", ")),
        format!(
            "New document templates: {}",
            NEW_DOCUMENT_TEMPLATES.join(", ")
        ),
    ];
    if !warnings.is_empty() {
        lines.push("Warnings:".to_string());
        lines.extend(warnings.iter().map(|warning| format!("  - {warning}")));
    }
    if let Some(command) = workspace_scaffold.recommended_command.as_ref() {
        lines.push(format!("Workspace setup command: {command}"));
    }
    Ok(CliOutcome {
        message: lines.join("\n"),
        exit_code,
    })
}

fn open_paths_in_neditor(paths: &[String]) -> Result<(), String> {
    if let Some(app_binary) = find_neditor_binary() {
        Command::new(app_binary)
            .args(paths)
            .stdin(Stdio::null())
            .spawn()
            .map_err(|err| format!("Could not launch NEditor: {err}"))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", APP_BUNDLE_NAME, "--args"])
            .args(paths)
            .stdin(Stdio::null())
            .spawn()
            .map_err(|err| format!("Could not open NEditor.app: {err}"))?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        for path in paths {
            Command::new("cmd")
                .args(["/C", "start", "", path])
                .stdin(Stdio::null())
                .spawn()
                .map_err(|err| format!("Could not open {path}: {err}"))?;
        }
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for path in paths {
            Command::new("xdg-open")
                .arg(path)
                .stdin(Stdio::null())
                .spawn()
                .map_err(|err| format!("Could not xdg-open {path}: {err}"))?;
        }
        Ok(())
    }
}

fn find_neditor_binary() -> Option<PathBuf> {
    if let Some(path) = env::var_os("NEDITOR_APP_BINARY").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    let current = env::current_exe().ok()?;
    let directory = current.parent()?;
    let candidate = directory.join(executable_name(APP_BINARY_NAME));
    if candidate.is_file() && candidate != current {
        Some(candidate)
    } else {
        None
    }
}

fn default_markdown_reader_response(enabled: bool, apply: bool) -> DefaultMarkdownReaderResponse {
    let platform = env::consts::OS.to_string();
    let mut response = match env::consts::OS {
        "macos" => macos_default_reader_plan(enabled),
        "linux" => linux_default_reader_plan(enabled),
        "windows" => windows_default_reader_plan(enabled),
        other => DefaultMarkdownReaderResponse {
            platform: other.to_string(),
            enabled,
            applied: false,
            supported: false,
            message: "Default Markdown reader setup is not automated on this platform.".to_string(),
            commands: Vec::new(),
            manual_steps: vec![
                "Use the operating system's Open With settings and choose NEditor for .md and .markdown files.".to_string(),
            ],
        },
    };
    response.platform = platform;
    if apply && response.supported {
        response.applied = apply_default_reader_commands(&response.commands).is_ok();
        if response.applied {
            response.message = "NEditor was requested as the default Markdown reader.".to_string();
        } else {
            response.message =
                "Could not apply automatically. Copy the commands or use the manual steps."
                    .to_string();
        }
    }
    response
}

fn macos_default_reader_plan(enabled: bool) -> DefaultMarkdownReaderResponse {
    DefaultMarkdownReaderResponse {
        platform: "macos".to_string(),
        enabled,
        applied: false,
        supported: command_available("duti"),
        message: "macOS default-app changes require LaunchServices. NEditor can use duti when it is installed.".to_string(),
        commands: vec![
            format!("duti -s {APP_BUNDLE_ID} net.daringfireball.markdown all"),
            format!("duti -s {APP_BUNDLE_ID} public.markdown all"),
        ],
        manual_steps: vec![
            "Install duti with Homebrew if your organization allows it, then rerun this setup.".to_string(),
            "Or use Finder: select a .md file, choose Get Info, set Open with to NEditor, and choose Change All.".to_string(),
        ],
    }
}

fn linux_default_reader_plan(enabled: bool) -> DefaultMarkdownReaderResponse {
    DefaultMarkdownReaderResponse {
        platform: "linux".to_string(),
        enabled,
        applied: false,
        supported: command_available("xdg-mime"),
        message: "Linux default Markdown reader setup uses xdg-mime and the installed NEditor desktop entry.".to_string(),
        commands: vec![
            format!("xdg-mime default {LINUX_DESKTOP_ID} text/markdown"),
            format!("xdg-mime default {LINUX_DESKTOP_ID} text/x-markdown"),
        ],
        manual_steps: vec![
            "If the desktop entry is not installed yet, install the NEditor package first.".to_string(),
            "Then choose NEditor as the default application for .md or .markdown files in your file manager.".to_string(),
        ],
    }
}

fn windows_default_reader_plan(enabled: bool) -> DefaultMarkdownReaderResponse {
    DefaultMarkdownReaderResponse {
        platform: "windows".to_string(),
        enabled,
        applied: false,
        supported: false,
        message: "Windows protects default-app changes behind user confirmation.".to_string(),
        commands: Vec::new(),
        manual_steps: vec![
            "Open Windows Settings > Apps > Default apps.".to_string(),
            "Search for .md and .markdown, then choose NEditor for each extension.".to_string(),
        ],
    }
}

fn apply_default_reader_commands(commands: &[String]) -> Result<(), String> {
    for command in commands {
        let mut parts = command.split_whitespace();
        let program = parts.next().ok_or_else(|| "empty command".to_string())?;
        let args = parts.collect::<Vec<_>>();
        let status = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|err| format!("Could not run {program}: {err}"))?;
        if !status.success() {
            return Err(format!("{program} exited with {status}"));
        }
    }
    Ok(())
}

fn command_available(command: &str) -> bool {
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|directory| directory.join(command).is_file()))
        .unwrap_or(false)
}

fn readiness_release_ready(report: &Value) -> bool {
    let status = readiness_string_field(report, "status")
        .unwrap_or("unknown")
        .to_ascii_lowercase();
    if status == "unknown"
        || ["gap", "fail", "pending", "partial", "blocker"]
            .iter()
            .any(|token| status.contains(token))
    {
        return false;
    }

    let required = readiness_summary_count(report, "requiredChecks");
    let accepted = readiness_summary_count(report, "accepted");
    let failed =
        readiness_summary_count(report, "failed") + readiness_array_field(report, "failures").len();
    let evidence_gaps = readiness_summary_count(report, "evidenceGaps")
        + readiness_array_field(report, "evidenceGaps").len();

    failed == 0 && evidence_gaps == 0 && (required == 0 || accepted >= required)
}

fn readiness_text_report(report: &Value, report_path: &str, release_ready: bool) -> String {
    let status = readiness_string_field(report, "status").unwrap_or("unknown");
    let generated = readiness_string_field(report, "generatedAt").unwrap_or("unknown");
    let platform = readiness_string_field(report, "platform").unwrap_or(env::consts::OS);
    let arch = readiness_string_field(report, "arch").unwrap_or(env::consts::ARCH);
    let required = readiness_summary_count(report, "requiredChecks");
    let accepted = readiness_summary_count(report, "accepted");
    let failed = readiness_summary_count(report, "failed");
    let evidence_gap_count = readiness_summary_count(report, "evidenceGaps")
        .max(readiness_array_field(report, "evidenceGaps").len());
    let checks = readiness_array_field(report, "checks");
    let failures = readiness_array_field(report, "failures");
    let evidence_gaps = readiness_array_field(report, "evidenceGaps");
    let mut lines = vec![
        format!("Release readiness: {status}"),
        format!(
            "Release-ready for publication: {}",
            if release_ready { "yes" } else { "no" }
        ),
        format!("Report: {report_path}"),
        format!("Generated: {generated}"),
        format!("Platform: {platform}-{arch}"),
        format!(
            "Summary: required checks {required}, accepted {accepted}, failed {failed}, evidence gaps {evidence_gap_count}"
        ),
    ];
    if !checks.is_empty() {
        lines.push("Checks:".to_string());
        for check in checks {
            lines.push(format!("  - {}", readiness_item_summary(&check)));
        }
    }
    if !failures.is_empty() {
        lines.push("Failures:".to_string());
        for failure in failures {
            lines.push(format!("  - {}", readiness_item_summary(&failure)));
        }
    }
    if evidence_gaps.is_empty() {
        lines.push("Evidence gaps: none".to_string());
    } else {
        lines.push("Evidence gaps:".to_string());
        for gap in evidence_gaps {
            lines.push(format!("  - {}", readiness_item_summary(&gap)));
            if let Some(detail) = readiness_string_field(&gap, "detail") {
                lines.push(format!("    {detail}"));
            }
        }
    }
    lines.push("Next commands:".to_string());
    lines.extend(
        readiness_next_commands(report)
            .iter()
            .map(|command| format!("  - {command}")),
    );
    lines.join("\n")
}

fn readiness_next_commands(report: &Value) -> Vec<String> {
    let mut commands = vec!["pnpm run check:release-readiness".to_string()];
    if !readiness_array_field(report, "evidenceGaps").is_empty()
        || readiness_summary_count(report, "evidenceGaps") > 0
    {
        commands.push("pnpm run collect:evidence-kit".to_string());
    }
    if !readiness_array_field(report, "failures").is_empty()
        || readiness_summary_count(report, "failed") > 0
    {
        commands.push("Inspect failed check report paths, fix them, then rerun pnpm run check:release-readiness".to_string());
    }
    if readiness_release_ready(report) {
        commands.push("Proceed to signed package and distribution publishing checks".to_string());
    }
    commands
}

fn evidence_next_commands(status: &str, evidence_root: &Path) -> Vec<String> {
    let root = path_to_display(evidence_root);
    let mut commands = vec![format!("ned evidence --evidence-root {root} --json")];
    if status != "ready" {
        commands.push("pnpm run collect:evidence-kit".to_string());
        commands.push("pnpm run ingest:evidence -- <evidence-kit-directory>".to_string());
        commands.push("pnpm run check:release-readiness".to_string());
    }
    commands
}

fn evidence_text_report(
    report: &Value,
    total: u64,
    ready: u64,
    attention: u64,
    missing: u64,
    failed: u64,
) -> String {
    let status = readiness_string_field(report, "status").unwrap_or("unknown");
    let root = readiness_string_field(report, "evidenceRoot").unwrap_or(".tmp");
    let reports = readiness_array_field(report, "reports");
    let mut lines = vec![
        format!("NEditor evidence status: {status}"),
        format!("Evidence root: {root}"),
        format!(
            "Reports: {ready} ready, {attention} need attention, {missing} missing, {failed} failed ({total} total)"
        ),
    ];
    if !reports.is_empty() {
        lines.push("Evidence reports:".to_string());
        for item in reports {
            let label = readiness_string_field(&item, "label").unwrap_or("Evidence report");
            let item_status = readiness_string_field(&item, "status").unwrap_or("unknown");
            let bucket = readiness_string_field(&item, "bucket").unwrap_or("attention");
            let path = readiness_string_field(&item, "reportPath").unwrap_or("unknown");
            lines.push(format!("  - {label}: {item_status} [{bucket}] {path}"));
        }
    }
    let next_commands = readiness_array_field(report, "nextCommands");
    if !next_commands.is_empty() {
        lines.push("Next commands:".to_string());
        for command in next_commands.iter().filter_map(Value::as_str) {
            lines.push(format!("  - {command}"));
        }
    }
    lines.join("\n")
}

fn build_support_bundle_report(
    workspace: &Path,
    readiness_report_path: &Path,
    spec_report_path: &Path,
    engine_report_path: &Path,
    evidence_root_path: &Path,
    output_path: Option<&Path>,
) -> Result<(Value, Option<String>), String> {
    let doctor_args = vec![
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--json".to_string(),
    ];
    let doctor_outcome = run_doctor_command(&doctor_args)?;
    let doctor: Value =
        serde_json::from_str(&doctor_outcome.message).map_err(|err| err.to_string())?;
    let readiness = read_readiness_report(readiness_report_path).unwrap_or_else(|err| {
        json!({
            "status": "missing",
            "error": err,
            "summary": {
                "requiredChecks": 0,
                "accepted": 0,
                "failed": 0,
                "evidenceGaps": 0
            },
            "checks": [],
            "evidenceGaps": [],
            "failures": []
        })
    });
    let release_ready = readiness_release_ready(&readiness);
    let spec_completion = read_json_report(spec_report_path).unwrap_or_else(|err| {
        json!({
            "status": "missing",
            "error": err,
            "summary": {
                "totalRows": 0,
                "completeRows": 0,
                "partialRows": 0,
                "unverifiedRows": 0,
                "missingRows": 0,
                "deferredRows": 0,
                "openRows": 0
            },
            "openRows": []
        })
    });
    let engine_probe = read_json_report(engine_report_path).unwrap_or_else(|err| {
        json!({
            "status": "missing",
            "error": err,
            "summary": {
                "installed": 0,
                "missingLocal": 0,
                "incompatible": 0,
                "acceptedExternalEvidence": 0,
                "invalidExternalEvidence": 0,
                "unresolvedMissingEvidence": 0
            },
            "engines": []
        })
    });
    let evidence_reports = support_bundle_evidence_reports(evidence_root_path);
    let evidence_report_summary = support_bundle_evidence_report_summary(&evidence_reports);
    let recommendations = support_bundle_recommendations(
        &doctor,
        &readiness,
        &spec_completion,
        &engine_probe,
        &evidence_report_summary,
    );
    let mut report = json!({
        "schema": "neditor.ned-support-bundle.v1",
        "generatedAtUnixSeconds": unix_timestamp_seconds(),
        "version": env!("CARGO_PKG_VERSION"),
        "platform": env::consts::OS,
        "arch": env::consts::ARCH,
        "workspace": path_to_display(workspace),
        "privacy": {
            "documentContentIncluded": false,
            "secretsIncluded": false,
            "note": "This bundle includes setup status, command paths, report paths, transform engine health, and release evidence summaries only."
        },
        "doctor": doctor,
        "releaseReadiness": {
            "reportPath": path_to_display(readiness_report_path),
            "status": readiness_string_field(&readiness, "status").unwrap_or("unknown"),
            "releaseReady": release_ready,
            "generatedAt": readiness_string_field(&readiness, "generatedAt"),
            "summary": readiness.get("summary").cloned().unwrap_or_else(|| json!({})),
            "evidenceGaps": readiness_array_field(&readiness, "evidenceGaps"),
            "failures": readiness_array_field(&readiness, "failures"),
            "nextCommands": readiness_next_commands(&readiness),
        },
        "specCompletion": {
            "reportPath": path_to_display(spec_report_path),
            "status": readiness_string_field(&spec_completion, "status").unwrap_or("unknown"),
            "generatedAt": readiness_string_field(&spec_completion, "generatedAt"),
            "summary": spec_completion.get("summary").cloned().unwrap_or_else(|| json!({})),
            "openRows": support_bundle_open_spec_rows(&spec_completion, 20),
        },
        "engineProbe": {
            "reportPath": path_to_display(engine_report_path),
            "status": readiness_string_field(&engine_probe, "status").unwrap_or("unknown"),
            "generatedAt": readiness_string_field(&engine_probe, "generatedAt"),
            "summary": engine_probe.get("summary").cloned().unwrap_or_else(|| json!({})),
            "engines": support_bundle_engine_rows(&engine_probe, 20),
        },
        "evidenceReports": evidence_reports,
        "evidenceReportSummary": evidence_report_summary,
        "recommendations": recommendations,
    });

    let written_to = if let Some(path) = output_path {
        write_json_report(path, &report)?;
        let written = path_to_display(path);
        if let Value::Object(fields) = &mut report {
            fields.insert("writtenTo".to_string(), json!(written));
        }
        Some(path_to_display(path))
    } else {
        None
    };

    Ok((report, written_to))
}

fn support_bundle_recommendations(
    doctor: &Value,
    readiness: &Value,
    spec_completion: &Value,
    engine_probe: &Value,
    evidence_report_summary: &Value,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    if readiness_string_field(doctor, "status").unwrap_or("unknown") != "ready" {
        recommendations
            .push("Review ned doctor warnings before sending files to business users.".to_string());
    }
    if !readiness_release_ready(readiness) {
        recommendations.push(
            "Close release-readiness evidence gaps before publishing installers or Homebrew casks."
                .to_string(),
        );
    }
    if readiness_string_field(readiness, "status").unwrap_or("unknown") == "missing" {
        recommendations.push(
            "Run pnpm run check:release-readiness in a developer checkout before release review."
                .to_string(),
        );
    }
    let open_spec_rows = spec_completion
        .get("summary")
        .and_then(|summary| summary.get("openRows"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if open_spec_rows > 0 {
        recommendations.push(format!(
            "Review {open_spec_rows} open specification row(s) before claiming production readiness."
        ));
    }
    if readiness_string_field(engine_probe, "status").unwrap_or("unknown") == "missing" {
        recommendations.push(
            "Run pnpm run check:engines to attach transform engine setup evidence.".to_string(),
        );
    }
    let missing_engines = summary_count_u64(engine_probe, "missingLocal");
    let incompatible_engines = summary_count_u64(engine_probe, "incompatible");
    let invalid_engine_evidence = summary_count_u64(engine_probe, "invalidExternalEvidence");
    if missing_engines > 0 || incompatible_engines > 0 || invalid_engine_evidence > 0 {
        recommendations.push(format!(
            "Review transform engine setup: {missing_engines} missing, {incompatible_engines} incompatible, {invalid_engine_evidence} invalid external evidence item(s)."
        ));
    }
    let evidence_attention = number_field_u64(evidence_report_summary, "attention")
        + number_field_u64(evidence_report_summary, "missing");
    if evidence_attention > 0 {
        recommendations.push(format!(
            "Collect or refresh {evidence_attention} release evidence report(s) before production publishing."
        ));
    }
    if recommendations.is_empty() {
        recommendations
            .push("Support bundle is ready for installation or release review.".to_string());
    }
    recommendations
}

fn support_bundle_text_report(report: &Value, written_to: Option<&str>) -> String {
    let doctor_status = report
        .get("doctor")
        .and_then(|doctor| readiness_string_field(doctor, "status"))
        .unwrap_or("unknown");
    let readiness = report.get("releaseReadiness").unwrap_or(&Value::Null);
    let readiness_status = readiness_string_field(readiness, "status").unwrap_or("unknown");
    let spec_completion = report.get("specCompletion").unwrap_or(&Value::Null);
    let spec_status = readiness_string_field(spec_completion, "status").unwrap_or("unknown");
    let open_spec_rows = spec_completion
        .get("summary")
        .and_then(|summary| summary.get("openRows"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let engine_probe = report.get("engineProbe").unwrap_or(&Value::Null);
    let engine_status = readiness_string_field(engine_probe, "status").unwrap_or("unknown");
    let installed_engines = summary_count_u64(engine_probe, "installed");
    let missing_engines = summary_count_u64(engine_probe, "missingLocal");
    let incompatible_engines = summary_count_u64(engine_probe, "incompatible");
    let evidence_report_summary = report.get("evidenceReportSummary").unwrap_or(&Value::Null);
    let evidence_ready = number_field_u64(evidence_report_summary, "ready");
    let evidence_attention = number_field_u64(evidence_report_summary, "attention");
    let evidence_missing = number_field_u64(evidence_report_summary, "missing");
    let evidence_gaps = readiness_array_field(readiness, "evidenceGaps").len();
    let failures = readiness_array_field(readiness, "failures").len();
    let recommendations = readiness_array_field(report, "recommendations")
        .iter()
        .filter_map(Value::as_str)
        .map(|value| format!("  - {value}"))
        .collect::<Vec<_>>();
    let mut lines = vec![
        "NEditor support bundle".to_string(),
        format!(
            "Workspace: {}",
            readiness_string_field(report, "workspace").unwrap_or("unknown")
        ),
        format!("Doctor: {doctor_status}"),
        format!("Release readiness: {readiness_status}"),
        format!("Evidence gaps: {evidence_gaps}"),
        format!("Failures: {failures}"),
        format!("Spec completion: {spec_status} ({open_spec_rows} open rows)"),
        format!(
            "Transform engines: {engine_status} ({installed_engines} installed, {missing_engines} missing, {incompatible_engines} incompatible)"
        ),
        format!(
            "Evidence reports: {evidence_ready} ready, {evidence_attention} need attention, {evidence_missing} missing"
        ),
        "Privacy: no document content or secrets included".to_string(),
    ];
    if let Some(path) = written_to {
        lines.push(format!("Wrote support bundle: {path}"));
    }
    if !recommendations.is_empty() {
        lines.push("Recommendations:".to_string());
        lines.extend(recommendations);
    }
    lines.push(
        "Use --json or --output support.json when a help desk needs machine-readable evidence."
            .to_string(),
    );
    lines.join("\n")
}

fn readiness_item_summary(item: &Value) -> String {
    let id = readiness_string_field(item, "id").unwrap_or("unknown");
    let status = readiness_string_field(item, "status").unwrap_or("unknown");
    let evidence =
        readiness_string_field(item, "evidence").or_else(|| readiness_string_field(item, "path"));
    match evidence {
        Some(evidence) => format!("{id} [{status}] {evidence}"),
        None => format!("{id} [{status}]"),
    }
}

fn readiness_string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn readiness_array_field(value: &Value, field: &str) -> Vec<Value> {
    value
        .get(field)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn support_bundle_open_spec_rows(report: &Value, limit: usize) -> Vec<Value> {
    readiness_array_field(report, "openRows")
        .into_iter()
        .take(limit)
        .collect()
}

fn support_bundle_engine_rows(report: &Value, limit: usize) -> Vec<Value> {
    readiness_array_field(report, "engines")
        .into_iter()
        .take(limit)
        .map(|engine| {
            json!({
                "key": readiness_string_field(&engine, "key"),
                "name": readiness_string_field(&engine, "name"),
                "status": readiness_string_field(&engine, "status").unwrap_or("unknown"),
                "command": readiness_string_field(&engine, "command"),
                "path": readiness_string_field(&engine, "path"),
                "version": readiness_string_field(&engine, "version"),
                "smoke": engine.get("smoke").map(|smoke| json!({
                    "status": readiness_string_field(smoke, "status").unwrap_or("unknown"),
                    "artifact": readiness_string_field(smoke, "artifact"),
                    "bytes": smoke.get("bytes").and_then(Value::as_u64),
                })),
                "externalEvidence": engine.get("externalEvidence").map(|evidence| json!({
                    "status": readiness_string_field(evidence, "status").unwrap_or("unknown"),
                    "path": readiness_string_field(evidence, "path"),
                })),
            })
        })
        .collect()
}

fn support_bundle_evidence_reports(evidence_root: &Path) -> Vec<Value> {
    [
        (
            "platform-evidence",
            "Windows/Linux platform evidence",
            "platform-evidence/report.json",
        ),
        (
            "release-signing",
            "Release signing and notarization",
            "release-signing/report.json",
        ),
        (
            "google-docs-import",
            "Google Docs import/readback",
            "google-docs-import/report.json",
        ),
        (
            "homebrew-packaging",
            "Homebrew packaging",
            "homebrew/homebrew-packaging-report.json",
        ),
        (
            "ai-provider-endpoint",
            "AI provider endpoint",
            "ai-provider-evidence/report.json",
        ),
        (
            "ai-runtime-device",
            "AI runtime device",
            "ai-runtime-evidence/report.json",
        ),
        (
            "performance-profile",
            "Release-device performance profile",
            "performance-profile/report.json",
        ),
        (
            "security-review",
            "Independent security review",
            "security-review/report.json",
        ),
        (
            "rendered-export-visual-signoff",
            "Rendered export native-viewer signoff",
            "rendered-export-audit/visual-review-summary.json",
        ),
        (
            "accessibility-manual-signoff",
            "Accessibility assistive-technology signoff",
            "accessibility/manual-review-summary.json",
        ),
    ]
    .into_iter()
    .map(|(id, label, relative_path)| {
        let path = evidence_root.join(relative_path);
        match read_json_report(&path) {
            Ok(report) => {
                let summary = report.get("summary").cloned().unwrap_or_else(|| json!({}));
                let status = support_bundle_evidence_status(&report, &summary);
                json!({
                    "id": id,
                    "label": label,
                    "reportPath": path_to_display(&path),
                    "status": status,
                    "bucket": support_bundle_evidence_bucket(status),
                    "generatedAt": readiness_string_field(&report, "generatedAt"),
                    "summary": summary,
                })
            }
            Err(error) => json!({
                "id": id,
                "label": label,
                "reportPath": path_to_display(&path),
                "status": "missing",
                "bucket": "missing",
                "error": error,
                "summary": {},
            }),
        }
    })
    .collect()
}

fn support_bundle_evidence_status<'a>(report: &'a Value, summary: &'a Value) -> &'a str {
    readiness_string_field(report, "status")
        .or_else(|| readiness_string_field(report, "result"))
        .or_else(|| readiness_string_field(summary, "status"))
        .or_else(|| {
            report
                .get("humanSignoff")
                .and_then(|value| readiness_string_field(value, "status"))
        })
        .or_else(|| {
            report
                .get("automatedVisualReview")
                .and_then(|value| readiness_string_field(value, "status"))
        })
        .unwrap_or("present")
}

fn support_bundle_evidence_report_summary(reports: &[Value]) -> Value {
    let mut ready = 0_u64;
    let mut attention = 0_u64;
    let mut missing = 0_u64;
    let mut failed = 0_u64;
    for report in reports {
        match readiness_string_field(report, "bucket").unwrap_or("attention") {
            "ready" => ready += 1,
            "missing" => missing += 1,
            "failed" => failed += 1,
            _ => attention += 1,
        }
    }
    json!({
        "total": reports.len(),
        "ready": ready,
        "attention": attention,
        "missing": missing,
        "failed": failed,
    })
}

fn support_bundle_evidence_bucket(status: &str) -> &'static str {
    let normalized = status.to_ascii_lowercase();
    if normalized.contains("failed")
        || normalized.contains("invalid")
        || normalized.contains("incomplete")
    {
        return "failed";
    }
    if normalized == "missing" {
        return "missing";
    }
    if normalized.starts_with("pending") || normalized.contains("blocker") {
        return "attention";
    }
    if matches!(
        normalized.as_str(),
        "accepted" | "complete" | "passed" | "human-reviewed" | "automated-reviewed"
    ) {
        return "ready";
    }
    "attention"
}

fn readiness_summary_count(report: &Value, field: &str) -> usize {
    report
        .get("summary")
        .and_then(|summary| summary.get(field))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

fn read_readiness_report(report_path: &Path) -> Result<Value, String> {
    read_json_report(report_path).map_err(|err| {
        format!("{err}. Run pnpm run check:release-readiness or pass --report <path>.")
    })
}

fn read_json_report(report_path: &Path) -> Result<Value, String> {
    let report_text = fs::read_to_string(report_path)
        .map_err(|err| format!("Could not read report at {}: {err}", report_path.display()))?;
    serde_json::from_str(&report_text)
        .map_err(|err| format!("Could not parse report at {}: {err}", report_path.display()))
}

fn write_json_report(path: &Path, report: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Could not create {}: {err}", parent.display()))?;
        }
    }
    let report_text = serde_json::to_string_pretty(report).map_err(|err| err.to_string())?;
    fs::write(path, format!("{report_text}\n"))
        .map_err(|err| format!("Could not write {}: {err}", path.display()))
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn summary_count_u64(report: &Value, field: &str) -> u64 {
    report
        .get("summary")
        .and_then(|summary| summary.get(field))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn number_field_u64(value: &Value, field: &str) -> u64 {
    value.get(field).and_then(Value::as_u64).unwrap_or(0)
}

fn doctor_warnings(
    app_binary: Option<&PathBuf>,
    default_reader: &DefaultMarkdownReaderResponse,
    workspace_scaffold: &WorkspaceScaffoldStatus,
    missing_handler_engines: &[String],
) -> Vec<String> {
    let mut warnings = Vec::new();
    if app_binary.is_none() {
        warnings.push(
            "NEditor app binary was not found next to ned; installed app open commands may use OS fallback routing."
                .to_string(),
        );
    }
    if !default_reader.supported {
        warnings.push(format!(
            "Automatic default-reader setup is not currently available on this host: {}",
            default_reader.message
        ));
    }
    if workspace_scaffold.status != "ready" {
        warnings.push(format!(
            "Workspace scaffold is {}; run {}",
            workspace_scaffold.status,
            workspace_scaffold
                .recommended_command
                .as_deref()
                .unwrap_or("ned init . --json")
        ));
    }
    if !missing_handler_engines.is_empty() {
        warnings.push(format!(
            "Transform handler setup plan is missing coverage for: {}",
            missing_handler_engines.join(", ")
        ));
    }
    warnings
}

fn workspace_scaffold_status(root: &Path) -> WorkspaceScaffoldStatus {
    let entries = workspace_init_entries(root);
    let required_files = entries
        .iter()
        .map(|(path, _)| path_to_display(path))
        .collect::<Vec<_>>();
    let missing_files = entries
        .iter()
        .filter(|(path, _)| !path.is_file())
        .map(|(path, _)| path_to_display(path))
        .collect::<Vec<_>>();
    let status = if root.exists() && missing_files.is_empty() {
        "ready"
    } else if !root.exists() {
        "workspace-missing"
    } else if missing_files.len() == required_files.len() {
        "not-initialized"
    } else {
        "incomplete"
    };
    WorkspaceScaffoldStatus {
        workspace: path_to_display(root),
        neditor_directory: path_to_display(&root.join(".neditor")),
        status: status.to_string(),
        required_files,
        missing_files,
        recommended_command: if status == "ready" {
            None
        } else {
            Some(format!("ned init {} --json", root.display()))
        },
    }
}

fn canonical_path_string(path: &Path) -> Result<String, String> {
    fs::canonicalize(path)
        .map_err(|err| format!("Could not find {}: {err}", path.display()))
        .map(|path| path.to_string_lossy().to_string())
}

fn is_direct_open_candidate(value: &str) -> bool {
    is_markdown_like_output_path(Path::new(value))
}

fn is_markdown_like_path_argument(value: &str) -> bool {
    if value.starts_with('-') {
        return false;
    }
    let path = Path::new(value);
    is_markdown_like_output_path(path) && path.is_file()
}

fn is_markdown_like_output_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("md" | "markdown" | "mdown" | "mkd")
    )
}

struct CliPublishPayloadOptions {
    input_path: String,
    export_target: String,
    destination_kind: String,
    endpoint_url: String,
    content_format: String,
    auth_header_name: String,
    token_env: String,
}

fn build_cli_publish_payload(
    response: &CompileResponse,
    readiness: &ExportReadinessReport,
    options: &CliPublishPayloadOptions,
) -> Value {
    let title = response.semantic.title.clone();
    let description = first_cli_metadata_string(
        &response.metadata,
        &["description", "summary", "subtitle", "excerpt"],
    )
    .unwrap_or_else(|| first_markdown_paragraph(&response.compiled_markdown));
    let tags = {
        let tags = metadata_string_list(&response.metadata, "tags");
        if tags.is_empty() {
            metadata_string_list(&response.metadata, "keywords")
        } else {
            tags
        }
    };
    let status = response.semantic.status.clone();
    let slug = metadata_string(&response.metadata, "slug")
        .map(|value| cli_slugify(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| cli_slugify(&title));
    let canonical_url = first_cli_metadata_string(
        &response.metadata,
        &["canonicalUrl", "canonical_url", "url"],
    );
    let language = first_cli_metadata_string(&response.metadata, &["language", "lang", "locale"])
        .unwrap_or_else(|| "en".to_string());
    let markdown = response.compiled_markdown.clone();
    let html = response.html.clone();
    let text = markdown_to_plain_text(&markdown);
    let content = match options.content_format.as_str() {
        "markdown" => markdown.clone(),
        "text" => text.clone(),
        _ => html.clone(),
    };
    let token_present = if options.token_env.trim().is_empty() {
        false
    } else {
        env::var(options.token_env.trim())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    };
    json!({
        "schema": "neditor.publish-payload.v1",
        "packageType": "neditor-publishing-handoff",
        "input": options.input_path,
        "target": options.export_target,
        "destinationKind": options.destination_kind,
        "method": "POST",
        "endpointUrl": options.endpoint_url.trim(),
        "title": title,
        "slug": slug,
        "status": status,
        "description": description,
        "canonicalUrl": canonical_url,
        "language": language,
        "tags": tags,
        "contentFormat": options.content_format,
        "content": content,
        "markdown": markdown,
        "html": html,
        "text": text,
        "auth": {
            "headerName": options.auth_header_name.trim(),
            "tokenEnv": options.token_env.trim(),
            "tokenPresent": token_present,
            "tokenPersisted": false
        },
        "audit": {
            "sourceHash": response.export_manifest.source_hash,
            "appVersion": response.export_manifest.app_version,
            "readiness": readiness.readiness,
            "diagnosticCount": readiness.diagnostics.len(),
            "generatedAt": response.export_manifest.exported_at
        },
        "curlTemplate": cli_publish_curl_template(options),
    })
}

fn cli_publish_text_report(payload: &Value, output_path: Option<&str>) -> String {
    let title = payload
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Untitled");
    let destination = payload
        .get("destinationKind")
        .and_then(Value::as_str)
        .unwrap_or("generic-webhook");
    let endpoint = payload
        .get("endpointUrl")
        .and_then(Value::as_str)
        .unwrap_or("");
    let output = output_path
        .map(|path| format!("\nPayload: {path}"))
        .unwrap_or_default();
    format!(
        "Prepared publishing payload for {title}\nDestination: {destination}\nEndpoint: {}{output}\nSecrets: not persisted; token is referenced by environment variable only",
        if endpoint.is_empty() { "(not set)" } else { endpoint }
    )
}

fn cli_publish_curl_template(options: &CliPublishPayloadOptions) -> String {
    if options.endpoint_url.trim().is_empty() {
        return "Set --endpoint before posting this payload.".to_string();
    }
    let auth = if options.token_env.trim().is_empty() {
        String::new()
    } else {
        format!(
            " -H '{}: ${{{}}}'",
            shell_single_quote(options.auth_header_name.trim()),
            options.token_env.trim()
        )
    };
    format!(
        "curl -X POST -H 'Content-Type: application/json'{auth} --data @payload.json '{}'",
        shell_single_quote(options.endpoint_url.trim())
    )
}

fn shell_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

fn first_cli_metadata_string(metadata: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| metadata_string(metadata, key))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn first_markdown_paragraph(markdown: &str) -> String {
    markdown
        .split("\n\n")
        .map(str::trim)
        .find(|block| !block.is_empty() && !block.starts_with('#') && !block.starts_with("---"))
        .unwrap_or("")
        .replace('\n', " ")
        .chars()
        .take(280)
        .collect()
}

fn markdown_to_plain_text(markdown: &str) -> String {
    markdown
        .lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .map(|line| {
            line.trim_start_matches('#')
                .trim_start_matches('>')
                .trim_start_matches("- ")
                .trim()
                .replace("**", "")
                .replace('*', "")
                .replace('`', "")
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn cli_slugify(value: &str) -> String {
    let mut output = String::new();
    let mut previous_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
            previous_dash = false;
        } else if !previous_dash && !output.is_empty() {
            output.push('-');
            previous_dash = true;
        }
    }
    output.trim_matches('-').to_string()
}

fn validate_publish_target(target: &str) -> Result<(), String> {
    if matches!(target, "blog" | "substack" | "html") {
        Ok(())
    } else {
        Err("Publish target must be blog, substack, or html.".to_string())
    }
}

fn validate_publish_destination(destination: &str) -> Result<(), String> {
    if matches!(
        destination,
        "generic-webhook" | "wordpress-rest" | "ghost-admin" | "substack-manual"
    ) {
        Ok(())
    } else {
        Err("Publish destination must be generic-webhook, wordpress-rest, ghost-admin, or substack-manual.".to_string())
    }
}

fn validate_publish_content_format(format: &str) -> Result<(), String> {
    if matches!(format, "html" | "markdown" | "text") {
        Ok(())
    } else {
        Err("Publish format must be html, markdown, or text.".to_string())
    }
}

fn validate_publish_auth_header(header: &str) -> Result<(), String> {
    let header = header.trim();
    if header.is_empty() {
        return Err("Publish auth header cannot be blank.".to_string());
    }
    if header.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '!' | '#'
                    | '$'
                    | '%'
                    | '&'
                    | '\''
                    | '*'
                    | '+'
                    | '-'
                    | '.'
                    | '^'
                    | '_'
                    | '`'
                    | '|'
                    | '~'
            )
    }) {
        Ok(())
    } else {
        Err("Publish auth header must be a valid HTTP header name.".to_string())
    }
}

fn validate_publish_token_env(token_env: &str) -> Result<(), String> {
    let token_env = token_env.trim();
    if token_env.is_empty() {
        return Ok(());
    }
    let mut chars = token_env.chars();
    let Some(first) = chars.next() else {
        return Ok(());
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(
            "Publish token environment variable must start with a letter or underscore."
                .to_string(),
        );
    }
    if chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric()) {
        Ok(())
    } else {
        Err("Publish token environment variable may only contain letters, numbers, and underscores.".to_string())
    }
}

fn publish_endpoint_is_allowed(value: &str) -> bool {
    let trimmed = value.trim().to_ascii_lowercase();
    if trimmed.starts_with("https://") {
        return true;
    }
    if !trimmed.starts_with("http://") {
        return false;
    }
    let host = trimmed
        .trim_start_matches("http://")
        .split(['/', ':'])
        .next()
        .unwrap_or("");
    host == "localhost" || host == "127.0.0.1" || host == "::1" || host.ends_with(".local")
}

fn read_cli_input_document(
    input_arg: &str,
    stdin_text: Option<&str>,
) -> Result<(String, Option<String>, String), String> {
    if input_arg == "-" {
        let text = if let Some(text) = stdin_text {
            text.to_string()
        } else {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .map_err(|err| format!("Could not read Markdown from stdin: {err}"))?;
            text
        };
        return Ok((text, None, "stdin.md".to_string()));
    }
    let input_path = canonical_path_string(&PathBuf::from(input_arg))?;
    let text = fs::read_to_string(&input_path)
        .map_err(|err| format!("Could not read input document {input_path}: {err}"))?;
    Ok((text, Some(input_path.clone()), input_path))
}

fn is_stdout_export_target(target: &str) -> bool {
    STDOUT_EXPORT_TARGETS.contains(&target)
}

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| {
            stem.replace(['-', '_'], " ")
                .split_whitespace()
                .map(capitalize_word)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| "Untitled".to_string())
}

fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn document_template_catalog() -> Vec<DocumentTemplateInfo> {
    vec![
        DocumentTemplateInfo {
            id: "blank",
            label: "Blank Document",
            category: "General",
            summary: "Minimal Markdown document with title and draft status.",
            best_for: &["freeform writing", "scratch drafts", "custom structure"],
        },
        DocumentTemplateInfo {
            id: "proposal",
            label: "Client Proposal",
            category: "Business development",
            summary: "Client-facing proposal with executive summary, approach, timeline, commercials, and review handoff.",
            best_for: &["sales proposals", "consulting offers", "commercial recommendations"],
        },
        DocumentTemplateInfo {
            id: "rfp",
            label: "Request For Proposal",
            category: "Procurement",
            summary: "Buyer-side RFP scaffold with scope, vendor instructions, evaluation criteria, and response matrix.",
            best_for: &["procurement packages", "vendor selection", "formal buyer requirements"],
        },
        DocumentTemplateInfo {
            id: "rfp-response",
            label: "RFP Response",
            category: "Business development",
            summary: "Seller-side response scaffold with compliance matrix, technical response, delivery plan, pricing, and final verification.",
            best_for: &["RFP responses", "compliance matrices", "bid teams"],
        },
        DocumentTemplateInfo {
            id: "rfq",
            label: "Request For Quote",
            category: "Procurement",
            summary: "Quote request with line items, commercial terms, award criteria, and vendor quote instructions.",
            best_for: &["pricing requests", "goods and services quotes", "supplier comparison"],
        },
        DocumentTemplateInfo {
            id: "tender",
            label: "Tender Package",
            category: "Procurement",
            summary: "Formal tender scaffold with eligibility, mandatory documents, specifications, instructions, and evaluation method.",
            best_for: &["public tenders", "formal bids", "regulated procurement"],
        },
        DocumentTemplateInfo {
            id: "sow",
            label: "Statement of Work",
            category: "Business development",
            summary: "Delivery-ready SOW with scope, deliverables, responsibilities, acceptance criteria, and change control.",
            best_for: &["delivery kickoff", "contract attachments", "services governance"],
        },
        DocumentTemplateInfo {
            id: "capability-statement",
            label: "Capability Statement",
            category: "Business development",
            summary: "Company qualifications, differentiators, proof points, certifications, and relevant experience.",
            best_for: &["vendor registration", "introductory submissions", "sales proof"],
        },
        DocumentTemplateInfo {
            id: "case-study",
            label: "Case Study",
            category: "Marketing",
            summary: "Customer proof story with challenge, solution, implementation, outcomes, quote prompts, and approvals.",
            best_for: &["sales proof", "marketing collateral", "client success stories"],
        },
        DocumentTemplateInfo {
            id: "report",
            label: "Business Report",
            category: "Business analysis",
            summary: "Decision report with executive summary, evidence, calculation starter, recommendations, risks, and next steps.",
            best_for: &["management reports", "analysis memos", "decision support"],
        },
        DocumentTemplateInfo {
            id: "tutorial",
            label: "Tutorial",
            category: "Education",
            summary: "Step-by-step tutorial with outcome, prerequisites, checks, troubleshooting, and next steps.",
            best_for: &["how-to guides", "training material", "customer enablement"],
        },
        DocumentTemplateInfo {
            id: "lesson-plan",
            label: "Lesson Plan",
            category: "Education",
            summary: "Instructor plan with objectives, audience, prerequisites, flow, learner evidence, and assessment.",
            best_for: &["classroom planning", "workshops", "training sessions"],
        },
        DocumentTemplateInfo {
            id: "lesson-content",
            label: "Lesson Content",
            category: "Education",
            summary: "Learner-facing lesson content with concept sequence, guided practice, independent practice, and assessment items.",
            best_for: &["course content", "student handouts", "learning modules"],
        },
        DocumentTemplateInfo {
            id: "textbook",
            label: "Textbook",
            category: "Long-form",
            summary: "Book scaffold with audience positioning, chapter outline, worked examples, exercises, and drafting plan.",
            best_for: &["textbooks", "manuals", "structured long-form education"],
        },
        DocumentTemplateInfo {
            id: "technical-textbook",
            label: "Technical Textbook",
            category: "Long-form",
            summary: "Technical long-form scaffold sharing the textbook structure with audience, level, prerequisites, chapters, and examples.",
            best_for: &["technical books", "engineering manuals", "expert curriculum"],
        },
        DocumentTemplateInfo {
            id: "novel",
            label: "Novel",
            category: "Creative writing",
            summary: "Narrative scaffold with premise, cast, act outline, and review checklist for voice, pacing, continuity, and scene purpose.",
            best_for: &["fiction planning", "plot outlines", "chapter drafting"],
        },
        DocumentTemplateInfo {
            id: "podcast-script",
            label: "Podcast Script",
            category: "Media",
            summary: "Episode script scaffold with brief, cold open, segment rundown, host script, and production notes.",
            best_for: &["podcasts", "interviews", "audio publishing"],
        },
        DocumentTemplateInfo {
            id: "movie-script",
            label: "Movie Script",
            category: "Media",
            summary: "Screen story scaffold with logline, characters, treatment, scene starter, and script review checklist.",
            best_for: &["screenplays", "film treatments", "story development"],
        },
        DocumentTemplateInfo {
            id: "business-case",
            label: "Business Case",
            category: "Business analysis",
            summary: "Investment case with decision required, rationale, options, financial calculation starter, and implementation plan.",
            best_for: &["investment decisions", "project approvals", "ROI analysis"],
        },
        DocumentTemplateInfo {
            id: "executive-brief",
            label: "Executive Brief",
            category: "Executive communication",
            summary: "Concise executive briefing with bottom line, what changed, evidence, options, and decision ask.",
            best_for: &["leadership updates", "board briefs", "decision memos"],
        },
        DocumentTemplateInfo {
            id: "board-decision-memo",
            label: "Board Decision Memo",
            category: "Executive communication",
            summary: "Board-ready decision paper with options, financial case, risks, recommendation, and approvals.",
            best_for: &["board packs", "investment approvals", "executive decisions"],
        },
        DocumentTemplateInfo {
            id: "policy-brief",
            label: "Policy Brief",
            category: "Policy",
            summary: "Evidence-led policy brief with problem framing, options, impacts, tradeoffs, and recommendation.",
            best_for: &["public policy", "research translation", "advisory briefs"],
        },
        DocumentTemplateInfo {
            id: "research-report",
            label: "Research Report",
            category: "Research",
            summary: "Structured research report with questions, method, findings, limitations, recommendations, and bibliography.",
            best_for: &["deep research", "evidence reports", "analyst deliverables"],
        },
        DocumentTemplateInfo {
            id: "grant-application",
            label: "Grant Application",
            category: "Funding",
            summary: "Funder-aligned application with need, beneficiaries, theory of change, work plan, budget, and measurement.",
            best_for: &["grant proposals", "foundation applications", "nonprofit funding"],
        },
        DocumentTemplateInfo {
            id: "standard-operating-procedure",
            label: "Standard Operating Procedure",
            category: "Operations",
            summary: "Repeatable SOP with ownership, procedure steps, controls, exceptions, evidence, and revision history.",
            best_for: &["SOPs", "work instructions", "operational controls"],
        },
        DocumentTemplateInfo {
            id: "product-requirements-document",
            label: "Product Requirements Document",
            category: "Product",
            summary: "PRD with goals, users, requirements, acceptance criteria, dependencies, metrics, and release risks.",
            best_for: &["product teams", "feature planning", "release scoping"],
        },
        DocumentTemplateInfo {
            id: "project-charter",
            label: "Project Charter",
            category: "Delivery",
            summary: "Project kickoff document with objectives, scope, stakeholders, governance, milestones, risks, and approvals.",
            best_for: &["project kickoff", "delivery governance", "executive approvals"],
        },
        DocumentTemplateInfo {
            id: "quarterly-business-review",
            label: "Quarterly Business Review",
            category: "Executive communication",
            summary: "Client or leadership review with scorecards, delivered value, risks, opportunities, and next-quarter actions.",
            best_for: &["client reviews", "account management", "leadership updates"],
        },
        DocumentTemplateInfo {
            id: "due-diligence-memo",
            label: "Due Diligence Memo",
            category: "Strategy",
            summary: "Investment or vendor diligence memo with thesis, evidence, risks, financials, operations, and recommendation.",
            best_for: &["investment diligence", "vendor assessment", "M&A review"],
        },
        DocumentTemplateInfo {
            id: "contract-review-brief",
            label: "Contract Review Brief",
            category: "Legal",
            summary: "Commercial contract review with key terms, obligations, service levels, data/IP terms, risks, and approvals.",
            best_for: &["contract review", "legal handoff", "commercial negotiation"],
        },
        DocumentTemplateInfo {
            id: "implementation-playbook",
            label: "Implementation Playbook",
            category: "Delivery",
            summary: "Operational playbook for implementing a project, process, tool, or platform.",
            best_for: &["delivery teams", "rollouts", "internal operating guides"],
        },
        DocumentTemplateInfo {
            id: "incident-postmortem",
            label: "Incident Postmortem",
            category: "Operations",
            summary: "Blameless incident review with impact, timeline, response, root cause, corrective actions, and prevention.",
            best_for: &["service incidents", "quality failures", "operational reviews"],
        },
        DocumentTemplateInfo {
            id: "meeting-decision-pack",
            label: "Meeting Decision Pack",
            category: "Executive communication",
            summary: "Meeting-ready packet with agenda, pre-read, options, evidence, risks, decisions, and actions.",
            best_for: &["steering committees", "executive meetings", "decision forums"],
        },
        DocumentTemplateInfo {
            id: "market-research-report",
            label: "Market Research Report",
            category: "Research",
            summary: "Market analysis report with objectives, method, segments, competitors, trends, findings, and recommendations.",
            best_for: &["market sizing", "competitive analysis", "strategy research"],
        },
    ]
}

fn template_matches_query(template: &DocumentTemplateInfo, query: &str) -> bool {
    template.id.to_ascii_lowercase().contains(query)
        || template.label.to_ascii_lowercase().contains(query)
        || template.category.to_ascii_lowercase().contains(query)
        || template.summary.to_ascii_lowercase().contains(query)
        || template
            .best_for
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(query))
}

fn templates_text_report(templates: &[DocumentTemplateInfo]) -> String {
    if templates.is_empty() {
        return "No NEditor document templates match those filters.".to_string();
    }
    let mut lines = vec![format!("NEditor document templates ({}):", templates.len())];
    for template in templates {
        lines.push(format!(
            "  - {} [{}] {}: {}",
            template.id, template.category, template.label, template.summary
        ));
    }
    lines.push("Use `ned new <file.md> --template <id> --json` to create one.".to_string());
    lines.join("\n")
}

fn find_outline_by_id_or_alias(
    catalog: Vec<DocumentOutlineEntry>,
    id: &str,
) -> Option<DocumentOutlineEntry> {
    let normalized = id.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if let Some(outline) = catalog.iter().find(|outline| outline.id == normalized) {
        return Some(outline.clone());
    }
    let builtin_alias = normalized
        .strip_prefix("business-")
        .or_else(|| normalized.strip_prefix("outline-"))?;
    catalog
        .iter()
        .find(|outline| outline.source == "builtin" && outline.id == builtin_alias)
        .cloned()
}

fn document_outline_catalog() -> Vec<DocumentOutlineInfo> {
    vec![
        DocumentOutlineInfo {
            id: "proposal",
            label: "Client Proposal",
            category: "Business development",
            summary: "Reusable client proposal structure for scope, delivery, commercials, and next steps.",
            docs_live_type: "proposal",
            best_for: &["consulting offers", "implementation projects", "commercial services"],
            outline: &["Executive Summary", "Client Situation", "Recommended Approach", "Scope of Work", "Deliverables", "Timeline", "Investment", "Assumptions", "Next Steps"],
            tags: &["proposal", "business-development", "commercial"],
        },
        DocumentOutlineInfo {
            id: "rfp",
            label: "Request for Proposal",
            category: "Procurement",
            summary: "Buyer-side RFP outline for opportunity framing, scope, instructions, evaluation, and response matrix.",
            docs_live_type: "project-plan",
            best_for: &["procurement packages", "vendor selection", "formal buyer requirements"],
            outline: &["Opportunity Summary", "Scope of Work", "Vendor Instructions", "Evaluation Criteria", "Required Response Matrix", "Submission Checklist", "Clarification Process"],
            tags: &["rfp", "buyer", "procurement", "vendor-selection"],
        },
        DocumentOutlineInfo {
            id: "rfq",
            label: "RFQ Response",
            category: "Procurement",
            summary: "Quotation response outline for buyer requirements, pricing, inclusions, exclusions, delivery, and validity.",
            docs_live_type: "rfq-response",
            best_for: &["price quotations", "supplier comparisons", "standardized services"],
            outline: &["Quotation Summary", "Buyer Requirements", "Quoted Items", "Pricing Table", "Inclusions", "Exclusions", "Delivery Schedule", "Commercial Terms", "Validity and Acceptance"],
            tags: &["rfq", "quotation", "pricing", "response"],
        },
        DocumentOutlineInfo {
            id: "tender",
            label: "Tender Response",
            category: "Procurement",
            summary: "Tender response outline for bid strategy, compliance, methodology, work plan, quality, and attachments.",
            docs_live_type: "tender-response",
            best_for: &["government tenders", "formal bids", "regulated procurement"],
            outline: &["Bid Summary", "Mandatory Submission Checklist", "Compliance Statement", "Technical Methodology", "Work Plan", "Key Personnel", "Quality and Risk Management", "Commercial Offer", "Required Attachments"],
            tags: &["tender", "bid", "procurement", "response"],
        },
        DocumentOutlineInfo {
            id: "sow",
            label: "Statement of Work",
            category: "Business Development",
            summary: "Delivery-ready SOW outline for scope, responsibilities, acceptance, and change control.",
            docs_live_type: "project-plan",
            best_for: &["delivery kickoff", "contract attachments", "services governance"],
            outline: &["Purpose", "Scope", "Deliverables", "Out of Scope", "Project Plan", "Roles and Responsibilities", "Acceptance Criteria", "Change Control", "Commercial Terms"],
            tags: &["sow", "statement-of-work", "scope", "delivery"],
        },
        DocumentOutlineInfo {
            id: "capability-statement",
            label: "Capability Statement",
            category: "Business Development",
            summary: "Business development outline for qualifications, differentiators, proof points, and relevant experience.",
            docs_live_type: "marketing-brief",
            best_for: &["business development", "vendor registration", "introductory submissions"],
            outline: &["Company Overview", "Core Capabilities", "Differentiators", "Relevant Experience", "Certifications and Compliance", "Representative Clients", "Contact"],
            tags: &["capability", "business-development", "proof", "marketing"],
        },
        DocumentOutlineInfo {
            id: "case-study",
            label: "Case Study",
            category: "Business Development",
            summary: "Customer proof story outline with challenge, solution, implementation, outcomes, and review approvals.",
            docs_live_type: "customer-case-study",
            best_for: &["sales proof", "marketing collateral", "client success stories"],
            outline: &["Customer Snapshot", "Challenge", "Solution", "Implementation", "Results", "Quote Prompts", "Review Approvals"],
            tags: &["case-study", "customer", "proof", "approvals"],
        },
        DocumentOutlineInfo {
            id: "report",
            label: "Business Report",
            category: "Business Analysis",
            summary: "Decision report outline for situation, evidence, analysis, recommendations, risks, and review handoff.",
            docs_live_type: "business-brief",
            best_for: &["management reports", "analysis memos", "decision support"],
            outline: &["Executive Summary", "Situation", "Evidence Base", "Analysis", "Recommendations", "Risks and Next Steps", "Review Handoff"],
            tags: &["report", "analysis", "decision-support", "business"],
        },
        DocumentOutlineInfo {
            id: "rfp-response",
            label: "RFP Response",
            category: "Procurement",
            summary: "Seller-side RFP response outline with compliance, solution, team, pricing, and appendices.",
            docs_live_type: "rfp-response",
            best_for: &["public procurement", "enterprise vendor selection", "competitive bids"],
            outline: &["Executive Response", "Compliance Matrix", "Understanding of Requirements", "Proposed Solution", "Implementation Plan", "Team and Experience", "Pricing Response", "Risk and Assumptions", "Appendices"],
            tags: &["rfp", "proposal", "compliance"],
        },
        DocumentOutlineInfo {
            id: "rfp-technical-proposal",
            label: "RFP Technical Proposal",
            category: "Procurement",
            summary: "Compliance-first technical proposal outline with checklist before content drafting.",
            docs_live_type: "rfp-response",
            best_for: &["technical proposals", "evaluated RFPs", "compliance-heavy bids"],
            outline: &["Cover", "Compliance Checklist", "Table of Contents", "Executive Summary", "Assignment Understanding", "Proposed Methodology", "Work Plan and Timeline", "Team Organization", "Past Performance", "Risk and Quality Management", "Sustainability and Transition", "Required Annexes"],
            tags: &["rfp", "technical", "compliance", "outline-first"],
        },
        DocumentOutlineInfo {
            id: "rfp-compliance-review",
            label: "RFP Compliance Review Pack",
            category: "Procurement",
            summary: "Reviewer-focused outline for disqualifiers, attachments, owners, and submission sign-off.",
            docs_live_type: "rfp-response",
            best_for: &["bid QA", "procurement review", "submission readiness"],
            outline: &["Source Intake Summary", "Critical Disqualifiers", "Mandatory Submission Checklist", "Compliance Matrix", "Attachment Register", "Evidence Owner Map", "Open Clarifications", "Submission QA Sign-off"],
            tags: &["rfp", "compliance", "qa", "attachments"],
        },
        DocumentOutlineInfo {
            id: "rfq-response",
            label: "RFQ Response",
            category: "Procurement",
            summary: "Concise quotation response outline for pricing, inclusions, exclusions, and validity.",
            docs_live_type: "rfq-response",
            best_for: &["price quotations", "supplier comparisons", "standardized services"],
            outline: &["Quotation Summary", "Buyer Requirements", "Quoted Items", "Pricing Table", "Inclusions", "Exclusions", "Delivery Schedule", "Commercial Terms", "Validity and Acceptance"],
            tags: &["rfq", "quotation", "pricing"],
        },
        DocumentOutlineInfo {
            id: "tender-response",
            label: "Tender Response",
            category: "Procurement",
            summary: "Formal tender response outline with mandatory checklist, method statement, quality, and attachments.",
            docs_live_type: "tender-response",
            best_for: &["government tenders", "formal bids", "regulated procurement"],
            outline: &["Bid Summary", "Mandatory Submission Checklist", "Compliance Statement", "Technical Methodology", "Work Plan", "Key Personnel", "Quality and Risk Management", "Commercial Offer", "Required Attachments"],
            tags: &["tender", "bid", "procurement"],
        },
        DocumentOutlineInfo {
            id: "tutorial",
            label: "Tutorial or Training Guide",
            category: "Learning",
            summary: "Practical tutorial outline with goals, prerequisites, walkthrough, practice, and troubleshooting.",
            docs_live_type: "tutorial",
            best_for: &["customer enablement", "internal training", "step-by-step adoption"],
            outline: &["Learning Goals", "Audience and Prerequisites", "Before You Begin", "Step-by-Step Walkthrough", "Practice Exercise", "Troubleshooting", "Next Steps"],
            tags: &["tutorial", "training", "learning"],
        },
        DocumentOutlineInfo {
            id: "lesson-plan",
            label: "Lesson Plan",
            category: "Learning",
            summary: "Instructor-ready lesson plan outline with objectives, flow, assessment, and differentiation.",
            docs_live_type: "lesson-plan",
            best_for: &["teachers", "corporate training", "workshop facilitators"],
            outline: &["Learning Objectives", "Standards and Prerequisites", "Materials", "Lesson Flow", "Guided Practice", "Assessment", "Differentiation", "Homework or Extension"],
            tags: &["lesson", "education", "training"],
        },
        DocumentOutlineInfo {
            id: "lesson-content",
            label: "Lesson Content",
            category: "Learning",
            summary: "Learner-facing lesson content outline with explanations, examples, practice, checks, and handouts.",
            docs_live_type: "lesson-content",
            best_for: &["course authors", "instructional designers", "enablement teams"],
            outline: &["Opening Hook", "Core Explanation", "Worked Example", "Practice Activity", "Knowledge Check", "Discussion Prompts", "Teacher Notes", "Learner Handout"],
            tags: &["lesson", "content", "education", "handouts"],
        },
        DocumentOutlineInfo {
            id: "technical-textbook",
            label: "Technical Textbook",
            category: "Learning",
            summary: "Outline-first textbook architecture for sequential chapter drafting and quality review.",
            docs_live_type: "technical-textbook",
            best_for: &["technical education", "certification courses", "engineering documentation"],
            outline: &["Textbook Architecture", "Chapter Outline", "Reader Prerequisites", "Chapter 1 - Conceptual Foundation", "Chapter 2 - Technical Model", "Chapter 3 - Worked Examples", "Chapter 4 - Practice Exercises", "Chapter 5 - Pitfalls and Review", "Instructional Quality Review"],
            tags: &["textbook", "technical", "long-form"],
        },
        DocumentOutlineInfo {
            id: "novel",
            label: "Novel",
            category: "Creative",
            summary: "Plot-first novel outline for story architecture, chapter sequence, and narrative review.",
            docs_live_type: "novel",
            best_for: &["fiction drafting", "story bibles", "developmental editing"],
            outline: &["Story Premise", "Character Arcs", "World and Continuity Rules", "Plot Outline", "Chapter 1 - Opening Image", "Chapter 2 - Inciting Incident", "Chapter 3 - Rising Complications", "Chapter 4 - Midpoint Reversal", "Chapter 5 - Crisis and Climax", "Chapter 6 - Resolution", "Narrative Quality Review"],
            tags: &["novel", "fiction", "plot"],
        },
        DocumentOutlineInfo {
            id: "podcast-script",
            label: "Podcast Script",
            category: "Creative",
            summary: "Episode architecture outline for segment planning, host script, production notes, and review.",
            docs_live_type: "podcast-script",
            best_for: &["podcast episodes", "interview shows", "narrative audio"],
            outline: &["Episode Architecture", "Segment Rundown", "Cold Open", "Intro", "Segment 1", "Segment 2", "Guest Questions", "Sponsor or Promo Read", "Outro", "Production Notes", "Audio Production Review"],
            tags: &["podcast", "script", "audio"],
        },
        DocumentOutlineInfo {
            id: "movie-script",
            label: "Movie Script",
            category: "Creative",
            summary: "Screen story outline for logline, characters, beat sheet, acts, key scenes, and production constraints.",
            docs_live_type: "movie-script",
            best_for: &["screenplays", "film treatments", "scene planning"],
            outline: &["Screen Story Architecture", "Logline", "Characters", "World and Tone", "Beat Sheet", "Act I", "Act II", "Act III", "Key Scenes", "Dialogue Notes", "Production Constraints", "Screenplay Quality Review"],
            tags: &["movie", "screenplay", "script"],
        },
        DocumentOutlineInfo {
            id: "board-decision-memo",
            label: "Board Decision Memo",
            category: "Executive",
            summary: "Decision-oriented outline for board or executive approval papers.",
            docs_live_type: "board-memo",
            best_for: &["board packs", "investment approvals", "executive decisions"],
            outline: &["Decision Requested", "Executive Summary", "Strategic Context", "Options Considered", "Financial Case", "Risk Assessment", "Implementation Plan", "Recommendation", "Appendices"],
            tags: &["board", "decision", "executive"],
        },
        DocumentOutlineInfo {
            id: "business-case",
            label: "Business Case",
            category: "Executive",
            summary: "Decision-ready business case outline for options, financial case, risks, and recommendation.",
            docs_live_type: "business-case",
            best_for: &["investment approval", "operating changes", "portfolio decisions"],
            outline: &["Executive Summary", "Decision Needed", "Problem", "Options", "Financial Case", "Risks", "Recommendation", "Implementation Plan"],
            tags: &["business-case", "decision", "finance"],
        },
        DocumentOutlineInfo {
            id: "executive-brief",
            label: "Executive Brief",
            category: "Executive",
            summary: "Concise senior-audience outline for context, options, risks, recommendation, and requested action.",
            docs_live_type: "business-brief",
            best_for: &["leadership updates", "decision meetings", "board pre-reads"],
            outline: &["Executive Summary", "Decision Needed", "Context", "Options", "Risks", "Recommendation", "Next Steps"],
            tags: &["executive", "brief", "decision", "leadership"],
        },
        DocumentOutlineInfo {
            id: "policy-brief",
            label: "Policy Brief",
            category: "Policy",
            summary: "Evidence-led policy outline with options, impacts, risks, and recommendation.",
            docs_live_type: "policy",
            best_for: &["public policy", "research translation", "advisory briefs"],
            outline: &["Executive Summary", "Problem Definition", "Policy Context", "Evidence Base", "Options", "Impact Assessment", "Risks and Tradeoffs", "Recommendation", "Implementation Considerations"],
            tags: &["policy", "brief", "evidence"],
        },
        DocumentOutlineInfo {
            id: "research-report",
            label: "Research Report",
            category: "Research",
            summary: "Structured research report outline with methodology, findings, citations, and recommendations.",
            docs_live_type: "research-brief",
            best_for: &["deep research", "evidence reports", "analyst deliverables"],
            outline: &["Abstract", "Introduction", "Research Questions", "Methodology", "Literature and Source Review", "Findings", "Analysis", "Limitations", "Recommendations", "Bibliography"],
            tags: &["research", "report", "citations"],
        },
        DocumentOutlineInfo {
            id: "implementation-playbook",
            label: "Implementation Playbook",
            category: "Delivery",
            summary: "Operational outline for implementing a project, tool, process, or platform.",
            docs_live_type: "project-plan",
            best_for: &["delivery teams", "rollouts", "internal operating guides"],
            outline: &["Purpose", "Operating Model", "Scope", "Roles and Responsibilities", "Implementation Phases", "Change Management", "Training Plan", "Risks and Controls", "Success Metrics", "Runbook"],
            tags: &["implementation", "delivery", "playbook"],
        },
        DocumentOutlineInfo {
            id: "grant-application",
            label: "Grant Application",
            category: "Funding",
            summary: "Funder-aligned application outline with need, theory of change, work plan, budget, and measurement.",
            docs_live_type: "proposal",
            best_for: &["grant proposals", "foundation applications", "nonprofit funding"],
            outline: &["Cover Summary", "Applicant Profile", "Problem and Need", "Target Beneficiaries", "Theory of Change", "Program Design", "Work Plan", "Monitoring and Evaluation", "Budget Narrative", "Sustainability", "Attachments"],
            tags: &["grant", "funding", "proposal", "impact"],
        },
        DocumentOutlineInfo {
            id: "standard-operating-procedure",
            label: "Standard Operating Procedure",
            category: "Operations",
            summary: "Repeatable SOP outline with ownership, steps, controls, records, exceptions, and revision history.",
            docs_live_type: "operating-procedure",
            best_for: &["SOPs", "work instructions", "operational controls"],
            outline: &["Purpose", "Scope", "Owner and Approvers", "Definitions", "Inputs and Systems", "Procedure", "Controls and Checks", "Exceptions", "Records and Evidence", "Revision History"],
            tags: &["sop", "procedure", "operations", "controls"],
        },
        DocumentOutlineInfo {
            id: "product-requirements-document",
            label: "Product Requirements Document",
            category: "Product",
            summary: "Product planning outline for goals, personas, requirements, acceptance criteria, release risks, and metrics.",
            docs_live_type: "project-plan",
            best_for: &["product teams", "feature planning", "release scoping"],
            outline: &["Problem Statement", "Goals and Non-Goals", "Personas and Use Cases", "Requirements", "User Experience Notes", "Technical Dependencies", "Acceptance Criteria", "Metrics", "Risks and Open Questions", "Release Plan"],
            tags: &["product", "prd", "requirements", "release"],
        },
        DocumentOutlineInfo {
            id: "project-charter",
            label: "Project Charter",
            category: "Delivery",
            summary: "Project start outline for objectives, scope, governance, milestones, assumptions, and success criteria.",
            docs_live_type: "project-plan",
            best_for: &["project kickoff", "delivery governance", "executive approvals"],
            outline: &["Executive Summary", "Objectives", "Scope", "Out of Scope", "Stakeholders", "Governance", "Milestones", "Budget and Resources", "Risks and Assumptions", "Success Criteria", "Approvals"],
            tags: &["project", "charter", "governance", "delivery"],
        },
        DocumentOutlineInfo {
            id: "quarterly-business-review",
            label: "Quarterly Business Review",
            category: "Executive",
            summary: "Client or leadership review outline with outcomes, scorecards, risks, opportunities, and next-quarter plan.",
            docs_live_type: "meeting-brief",
            best_for: &["client reviews", "account management", "leadership updates"],
            outline: &["Executive Summary", "Period Highlights", "Goals and Scorecard", "Usage or Financial Trends", "Delivered Value", "Risks and Blockers", "Opportunities", "Next Quarter Plan", "Decisions and Actions"],
            tags: &["qbr", "review", "scorecard", "customer"],
        },
        DocumentOutlineInfo {
            id: "due-diligence-memo",
            label: "Due Diligence Memo",
            category: "Strategy",
            summary: "Investment or vendor diligence outline for thesis, evidence, risks, financials, operations, and recommendation.",
            docs_live_type: "business-case",
            best_for: &["investment diligence", "vendor assessment", "M&A review"],
            outline: &["Executive Summary", "Transaction or Decision Context", "Investment Thesis", "Market and Customer Evidence", "Product or Service Assessment", "Financial Review", "Operational Review", "Legal and Compliance Review", "Risk Register", "Recommendation", "Appendices"],
            tags: &["diligence", "investment", "vendor", "risk"],
        },
        DocumentOutlineInfo {
            id: "incident-postmortem",
            label: "Incident Postmortem",
            category: "Operations",
            summary: "Blameless incident review outline with timeline, impact, root causes, corrective actions, and prevention.",
            docs_live_type: "operating-procedure",
            best_for: &["service incidents", "quality failures", "operational reviews"],
            outline: &["Summary", "Customer or Business Impact", "Timeline", "Detection and Response", "Root Cause Analysis", "What Went Well", "What Did Not Go Well", "Corrective Actions", "Preventive Controls", "Owner Sign-off"],
            tags: &["incident", "postmortem", "operations", "quality"],
        },
        DocumentOutlineInfo {
            id: "meeting-decision-pack",
            label: "Meeting Decision Pack",
            category: "Executive",
            summary: "Meeting-ready outline that combines agenda, pre-read, decisions, evidence, risks, and action register.",
            docs_live_type: "meeting-brief",
            best_for: &["steering committees", "executive meetings", "decision forums"],
            outline: &["Meeting Purpose", "Required Decisions", "Agenda", "Pre-Read Summary", "Options and Tradeoffs", "Evidence Pack", "Risks", "Decision Log", "Action Register", "Follow-up Communications"],
            tags: &["meeting", "agenda", "decision", "actions"],
        },
        DocumentOutlineInfo {
            id: "market-research-report",
            label: "Market Research Report",
            category: "Research",
            summary: "Market analysis outline with questions, method, segmentation, competitors, findings, and recommendations.",
            docs_live_type: "research-brief",
            best_for: &["market sizing", "competitive analysis", "strategy research"],
            outline: &["Executive Summary", "Research Objectives", "Methodology", "Market Definition", "Customer Segments", "Competitor Landscape", "Trends and Drivers", "Findings", "Implications", "Recommendations", "Bibliography"],
            tags: &["market", "research", "competitors", "strategy"],
        },
        DocumentOutlineInfo {
            id: "contract-review-brief",
            label: "Contract Review Brief",
            category: "Legal",
            summary: "Commercial contract review outline for key terms, obligations, risks, negotiation positions, and approvals.",
            docs_live_type: "contract-brief",
            best_for: &["contract review", "legal handoff", "commercial negotiation"],
            outline: &["Contract Summary", "Parties and Scope", "Commercial Terms", "Delivery Obligations", "Service Levels", "Data and IP Terms", "Liability and Indemnities", "Termination and Renewal", "Negotiation Positions", "Approval Checklist"],
            tags: &["contract", "legal", "commercial", "risk"],
        },
    ]
}

struct WorkspaceOutlineSaveInput {
    id: String,
    label: Option<String>,
    category: Option<String>,
    summary: Option<String>,
    docs_live_type: Option<String>,
    outline_file: Option<String>,
    sections: Vec<String>,
    tags: Vec<String>,
    best_for: Vec<String>,
}

fn document_outline_catalog_entries(workspace: &Path) -> Result<Vec<DocumentOutlineEntry>, String> {
    let mut outlines = document_outline_catalog()
        .into_iter()
        .map(built_in_outline_entry)
        .collect::<Vec<_>>();
    outlines.extend(
        read_workspace_outline_library(workspace)?
            .outlines
            .into_iter()
            .map(workspace_outline_entry),
    );
    Ok(outlines)
}

fn built_in_outline_entry(outline: DocumentOutlineInfo) -> DocumentOutlineEntry {
    DocumentOutlineEntry {
        id: outline.id.to_string(),
        label: outline.label.to_string(),
        category: outline.category.to_string(),
        summary: outline.summary.to_string(),
        docs_live_type: outline.docs_live_type.to_string(),
        best_for: outline
            .best_for
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        outline: outline
            .outline
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        tags: outline
            .tags
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        source: "builtin".to_string(),
    }
}

fn workspace_outline_entry(outline: WorkspaceDocumentOutline) -> DocumentOutlineEntry {
    DocumentOutlineEntry {
        id: outline.id,
        label: outline.label,
        category: outline.category,
        summary: outline.summary,
        docs_live_type: outline
            .docs_live_type
            .unwrap_or_else(|| "business-brief".to_string()),
        best_for: outline.best_for,
        outline: outline.outline,
        tags: outline.tags,
        source: "workspace".to_string(),
    }
}

fn workspace_outline_library_path(workspace: &Path) -> PathBuf {
    workspace.join(".neditor").join("outlines.json")
}

fn read_workspace_outline_library(workspace: &Path) -> Result<WorkspaceOutlineLibrary, String> {
    let path = workspace_outline_library_path(workspace);
    if !path.exists() {
        return Ok(empty_workspace_outline_library());
    }
    let raw = fs::read_to_string(&path).map_err(|err| {
        format!(
            "Could not read workspace outline library {}: {err}",
            path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&raw)
        .map_err(|err| format!("Workspace outline library is not valid JSON: {err}"))?;
    let outlines_value = if value.is_array() {
        value
    } else {
        value
            .get("outlines")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()))
    };
    let outlines = serde_json::from_value::<Vec<WorkspaceDocumentOutline>>(outlines_value)
        .map_err(|err| format!("Workspace outline library outlines are invalid: {err}"))?
        .into_iter()
        .filter_map(normalize_workspace_outline)
        .collect::<Vec<_>>();
    Ok(WorkspaceOutlineLibrary {
        schema: "neditor.workspace-outlines.v1".to_string(),
        outlines,
    })
}

fn write_workspace_outline_library(
    workspace: &Path,
    library: &WorkspaceOutlineLibrary,
) -> Result<(), String> {
    let path = workspace_outline_library_path(workspace);
    let parent = path
        .parent()
        .ok_or_else(|| format!("Could not determine parent for {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|err| format!("Could not create {}: {err}", parent.display()))?;
    let mut normalized = WorkspaceOutlineLibrary {
        schema: "neditor.workspace-outlines.v1".to_string(),
        outlines: library
            .outlines
            .iter()
            .cloned()
            .filter_map(normalize_workspace_outline)
            .collect(),
    };
    normalized
        .outlines
        .sort_by(|left, right| left.id.cmp(&right.id));
    fs::write(
        &path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&normalized).map_err(|err| err.to_string())?
        ),
    )
    .map_err(|err| {
        format!(
            "Could not write workspace outline library {}: {err}",
            path.display()
        )
    })
}

fn empty_workspace_outline_library() -> WorkspaceOutlineLibrary {
    WorkspaceOutlineLibrary {
        schema: "neditor.workspace-outlines.v1".to_string(),
        outlines: Vec::new(),
    }
}

fn save_workspace_outline(
    workspace: &Path,
    input: WorkspaceOutlineSaveInput,
) -> Result<WorkspaceDocumentOutline, String> {
    let id = sanitize_outline_id(&input.id).ok_or_else(|| {
        "--save requires an id with letters, numbers, dots, underscores, or hyphens".to_string()
    })?;
    let docs_live_type = if input.docs_live_type.is_some() {
        normalize_docs_live_type(input.docs_live_type.clone()).ok_or_else(|| {
            "--docs-live-type must be a supported Docs Live document type".to_string()
        })?
    } else {
        "business-brief".to_string()
    };
    let mut outline = input
        .sections
        .iter()
        .filter_map(|section| normalize_outline_heading(section))
        .collect::<Vec<_>>();
    if let Some(path) = input.outline_file.as_deref() {
        let file_text = fs::read_to_string(path)
            .map_err(|err| format!("Could not read outline file {path}: {err}"))?;
        outline.extend(parse_outline_text(&file_text));
    }
    outline = dedupe_string_vec(outline, 80);
    if outline.is_empty() {
        return Err(
            "Add at least one --section or supply --outline-file with headings or bullet lines."
                .to_string(),
        );
    }
    let saved = WorkspaceDocumentOutline {
        id,
        label: outline_string(input.label).unwrap_or_else(|| "Custom outline".to_string()),
        category: outline_string(input.category).unwrap_or_else(|| "Custom".to_string()),
        summary: outline_string(input.summary).unwrap_or_else(|| {
            format!(
                "Reusable workspace outline with {} section(s).",
                outline.len()
            )
        }),
        docs_live_type: Some(docs_live_type),
        best_for: dedupe_string_vec(input.best_for, 12),
        outline,
        tags: dedupe_string_vec(input.tags, 16),
    };
    let mut library = read_workspace_outline_library(workspace)?;
    library.outlines.retain(|outline| outline.id != saved.id);
    library.outlines.push(saved.clone());
    write_workspace_outline_library(workspace, &library)?;
    Ok(saved)
}

fn delete_workspace_outline(workspace: &Path, id: &str) -> Result<bool, String> {
    let mut library = read_workspace_outline_library(workspace)?;
    let original_len = library.outlines.len();
    library.outlines.retain(|outline| outline.id != id);
    let deleted = library.outlines.len() != original_len;
    if deleted {
        write_workspace_outline_library(workspace, &library)?;
    }
    Ok(deleted)
}

fn normalize_workspace_outline(
    outline: WorkspaceDocumentOutline,
) -> Option<WorkspaceDocumentOutline> {
    let id = sanitize_outline_id(&outline.id)?;
    let normalized_outline = dedupe_string_vec(outline.outline, 80);
    if normalized_outline.is_empty() {
        return None;
    }
    Some(WorkspaceDocumentOutline {
        id,
        label: outline_string(Some(outline.label)).unwrap_or_else(|| "Custom outline".to_string()),
        category: outline_string(Some(outline.category)).unwrap_or_else(|| "Custom".to_string()),
        summary: outline_string(Some(outline.summary)).unwrap_or_else(|| {
            format!(
                "Reusable workspace outline with {} section(s).",
                normalized_outline.len()
            )
        }),
        docs_live_type: normalize_docs_live_type(outline.docs_live_type),
        best_for: dedupe_string_vec(outline.best_for, 12),
        outline: normalized_outline,
        tags: dedupe_string_vec(outline.tags, 16),
    })
}

fn sanitize_outline_id(value: &str) -> Option<String> {
    let id = value.trim().to_ascii_lowercase();
    if id.is_empty()
        || id.len() > 80
        || !id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return None;
    }
    Some(id)
}

fn parse_outline_text(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(normalize_outline_heading)
        .collect::<Vec<_>>()
}

fn normalize_outline_heading(value: &str) -> Option<String> {
    let mut trimmed = value
        .trim()
        .trim_start_matches('#')
        .trim()
        .trim_start_matches(|ch: char| ch == '-' || ch == '*' || ch == '+')
        .trim();
    if let Some(numbered) = trimmed
        .split_once(|ch: char| ch == '.' || ch == ')')
        .and_then(|(prefix, rest)| {
            (!prefix.is_empty() && prefix.chars().all(|ch| ch.is_ascii_digit()))
                .then_some(rest.trim())
        })
        .filter(|rest| !rest.is_empty())
    {
        trimmed = numbered;
    }
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.chars().take(160).collect::<String>())
}

fn outline_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().chars().take(500).collect::<String>())
        .filter(|value| !value.is_empty())
}

fn normalize_docs_live_type(value: Option<String>) -> Option<String> {
    let value = value?.trim().to_ascii_lowercase();
    if value.is_empty() {
        return None;
    }
    let allowed = [
        "business-brief",
        "board-memo",
        "proposal",
        "rfp-response",
        "rfq-response",
        "tender-response",
        "tutorial",
        "lesson-plan",
        "lesson-content",
        "technical-textbook",
        "novel",
        "podcast-script",
        "movie-script",
        "strategy-plan",
        "project-plan",
        "research-brief",
        "policy",
        "meeting-brief",
        "business-case",
        "operating-procedure",
        "technical-architecture",
        "adr",
        "release-notes",
        "contract-brief",
        "marketing-brief",
        "customer-case-study",
    ];
    allowed.contains(&value.as_str()).then_some(value)
}

fn dedupe_string_vec(values: Vec<String>, limit: usize) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut output = Vec::new();
    for value in values {
        let cleaned = value.trim().chars().take(500).collect::<String>();
        if cleaned.is_empty() || !seen.insert(cleaned.to_ascii_lowercase()) {
            continue;
        }
        output.push(cleaned);
        if output.len() >= limit {
            break;
        }
    }
    output
}

fn outline_matches_query(outline: &DocumentOutlineEntry, query: &str) -> bool {
    outline.id.to_ascii_lowercase().contains(query)
        || outline.label.to_ascii_lowercase().contains(query)
        || outline.category.to_ascii_lowercase().contains(query)
        || outline.summary.to_ascii_lowercase().contains(query)
        || outline.docs_live_type.to_ascii_lowercase().contains(query)
        || outline
            .best_for
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(query))
        || outline
            .tags
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(query))
        || outline
            .outline
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(query))
}

fn outline_markdown(outline: &DocumentOutlineEntry) -> String {
    outline
        .outline
        .iter()
        .map(|heading| format!("- {heading}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn outlines_text_report(outlines: &[DocumentOutlineEntry]) -> String {
    if outlines.is_empty() {
        return "No NEditor document outlines match those filters.".to_string();
    }
    let mut lines = vec![format!("NEditor document outlines ({}):", outlines.len())];
    for outline in outlines {
        lines.push(format!(
            "  - {} [{} | Docs Live: {}] {}: {}",
            outline.id, outline.category, outline.docs_live_type, outline.label, outline.summary
        ));
    }
    lines.push("Use `ned outlines --markdown <id>` to print a reusable outline, or `ned outlines --workspace . --save <id> --docs-live-type proposal --section \"Executive Summary\"` to add one to .neditor/outlines.json.".to_string());
    lines.join("\n")
}

fn transform_template_catalog() -> Result<Vec<TransformTemplateEntry>, String> {
    let source = TRANSFORM_TEMPLATE_SOURCE;
    let start = source
        .find("export const builtinTransformTemplates")
        .ok_or_else(|| "Could not find built-in transform template source.".to_string())?;
    let initializer = source[start..]
        .find('=')
        .map(|index| start + index)
        .ok_or_else(|| "Could not find transform template initializer.".to_string())?;
    let array_open = source[initializer..]
        .find('[')
        .map(|index| initializer + index)
        .ok_or_else(|| "Could not find transform template array.".to_string())?;
    let array_close = find_matching_delimiter(source, array_open, '[', ']')
        .ok_or_else(|| "Could not parse transform template array.".to_string())?;
    let mut templates = Vec::new();
    let mut cursor = array_open + 1;
    while let Some(relative) = source[cursor..array_close].find("template(") {
        let template_start = cursor + relative;
        let open = template_start + "template".len();
        let Some(close) = find_matching_delimiter(source, open, '(', ')') else {
            break;
        };
        if close > array_close {
            break;
        }
        let args = split_top_level_arguments(&source[open + 1..close]);
        if args.len() >= 6 {
            if let (Some(id), Some(category), Some(transform), Some(name), Some(summary)) = (
                parse_js_string(args[0].trim()),
                parse_js_string(args[1].trim()),
                parse_js_string(args[2].trim()),
                parse_js_string(args[3].trim()),
                parse_js_string(args[4].trim()),
            ) {
                templates.push(TransformTemplateEntry {
                    id,
                    name,
                    category,
                    transform,
                    summary,
                    body: render_transform_template_body(args[5].trim()).unwrap_or_default(),
                    tags: args
                        .get(6)
                        .map(|value| parse_js_string_array(value))
                        .unwrap_or_default(),
                });
            }
        }
        cursor = close + 1;
    }
    if templates.is_empty() {
        return Err("No built-in transform templates could be parsed.".to_string());
    }
    Ok(templates)
}

fn render_transform_template_body(expression: &str) -> Option<String> {
    let expression = expression.trim();
    if expression.starts_with("calc(") {
        let open = expression.find('(')?;
        let close = find_matching_delimiter(expression, open, '(', ')')?;
        let args = split_top_level_arguments(&expression[open + 1..close]);
        let content = parse_js_string(args.first()?.trim())?;
        let after = args
            .get(1)
            .and_then(|value| parse_js_string(value.trim()))
            .unwrap_or_default();
        return Some(format!("```calc\n{}\n```\n{}", content.trim(), after));
    }
    if expression.starts_with("fenced(") {
        return render_fenced_expression(expression);
    }
    parse_js_string(expression)
}

fn render_fenced_expression(expression: &str) -> Option<String> {
    let open = expression.find('(')?;
    let close = find_matching_delimiter(expression, open, '(', ')')?;
    let args = split_top_level_arguments(&expression[open + 1..close]);
    let transform = parse_js_string(args.first()?.trim())?;
    let content = parse_js_string(args.get(1)?.trim())?;
    let options = args
        .get(2)
        .and_then(|value| parse_js_string(value.trim()))
        .unwrap_or_default();
    let suffix = if options.trim().is_empty() {
        String::new()
    } else {
        format!(" {}", options.trim())
    };
    Some(format!(
        "```{}{}\n{}\n```\n",
        transform,
        suffix,
        content.trim()
    ))
}

fn find_matching_delimiter(
    source: &str,
    open_index: usize,
    open_delimiter: char,
    close_delimiter: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (offset, ch) in source[open_index..].char_indices() {
        let index = open_index + offset;
        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote_char {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'' | '`') {
            quote = Some(ch);
            continue;
        }
        if ch == open_delimiter {
            depth += 1;
        } else if ch == close_delimiter {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

fn split_top_level_arguments(source: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, ch) in source.char_indices() {
        if let Some(quote_char) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote_char {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                args.push(source[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    let tail = source[start..].trim();
    if !tail.is_empty() {
        args.push(tail);
    }
    args
}

fn parse_js_string(source: &str) -> Option<String> {
    let source = source.trim();
    let quote = source.chars().next()?;
    if !matches!(quote, '"' | '\'' | '`') {
        return None;
    }
    let mut output = String::new();
    let mut escaped = false;
    for ch in source[quote.len_utf8()..].chars() {
        if escaped {
            output.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                '`' => '`',
                other => other,
            });
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some(output);
        }
        output.push(ch);
    }
    None
}

fn parse_js_string_array(source: &str) -> Vec<String> {
    let source = source.trim();
    if !source.starts_with('[') {
        return Vec::new();
    }
    split_top_level_arguments(source.trim_start_matches('[').trim_end_matches(']').trim())
        .into_iter()
        .filter_map(|value| parse_js_string(value.trim()))
        .collect()
}

fn transform_template_matches_query(template: &TransformTemplateEntry, query: &str) -> bool {
    template.id.to_ascii_lowercase().contains(query)
        || template.name.to_ascii_lowercase().contains(query)
        || template.category.to_ascii_lowercase().contains(query)
        || template.transform.to_ascii_lowercase().contains(query)
        || template.summary.to_ascii_lowercase().contains(query)
        || template
            .tags
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(query))
        || template.body.to_ascii_lowercase().contains(query)
}

fn transform_templates_text_report(templates: &[TransformTemplateEntry]) -> String {
    if templates.is_empty() {
        return "No NEditor transform templates match those filters.".to_string();
    }
    let mut lines = vec![format!(
        "NEditor transform templates ({}):",
        templates.len()
    )];
    for template in templates {
        lines.push(format!(
            "  - {} [{} | {}] {}: {}",
            template.id, template.category, template.transform, template.name, template.summary
        ));
    }
    lines.push("Use `ned transform-templates --markdown <id>` to print a reusable calc, chart, diagram, data, API, or planning transform block.".to_string());
    lines.join("\n")
}

fn document_snippet_catalog() -> Vec<DocumentSnippetInfo> {
    vec![
        DocumentSnippetInfo {
            id: "company-contact-block",
            label: "Company contact block",
            kind: "identity",
            summary: "Reusable sender and organization block for cover pages, letters, and submissions.",
            body: "**Prepared by:** {{fullName}}, {{roleTitle}}\n\n**Company:** {{companyName}}\n\n**Address:** {{companyAddress}}\n\n**Email:** {{email}}  \n**Phone:** {{phone}}  \n**Website:** {{website}}\n",
        },
        DocumentSnippetInfo {
            id: "company-overview",
            label: "Company overview",
            kind: "identity",
            summary: "Short boilerplate overview for proposals, tenders, and capability statements.",
            body: "{{companyName}} is a {{industry}} organization. We help {{defaultClientName}} make practical decisions with clear evidence, disciplined delivery, and {{brandVoice}} communication.\n",
        },
        DocumentSnippetInfo {
            id: "executive-summary",
            label: "Executive summary starter",
            kind: "proposal",
            summary: "A compact executive summary scaffold with reader outcome and recommendation placeholders.",
            body: "## Executive Summary\n\n{{defaultClientName}} needs {{outcome}}. {{companyName}} recommends {{recommendation}} because {{evidence}}.\n\nThe proposed approach focuses on {{scope}}, with delivery led by {{fullName}} and reviewed against {{success_criteria}}.\n",
        },
        DocumentSnippetInfo {
            id: "scope-of-work",
            label: "Scope of work",
            kind: "delivery",
            summary: "Reusable scope, deliverables, out-of-scope, and acceptance block.",
            body: "## Scope of Work\n\n### In Scope\n\n- {{scope_item_1}}\n- {{scope_item_2}}\n- {{scope_item_3}}\n\n### Deliverables\n\n| Deliverable | Acceptance criteria | Owner |\n| --- | --- | --- |\n| {{deliverable}} | {{acceptance_criteria}} | {{owner}} |\n\n### Out of Scope\n\n- {{out_of_scope_item}}\n",
        },
        DocumentSnippetInfo {
            id: "pricing-assumptions",
            label: "Pricing assumptions",
            kind: "proposal",
            summary: "Commercial assumptions that make quotes and proposals easier to review.",
            body: "## Pricing Assumptions\n\n- Pricing is based on {{pricing_basis}}.\n- Fees exclude {{exclusions}} unless stated otherwise.\n- The estimate assumes timely access to {{client_inputs}}.\n- Pricing remains valid until {{valid_until}}.\n",
        },
        DocumentSnippetInfo {
            id: "rfp-compliance-matrix",
            label: "RFP compliance matrix",
            kind: "procurement",
            summary: "Response matrix for buyer requirements, compliance status, and evidence references.",
            body: "## Compliance Matrix\n\n| Requirement | Response | Evidence | Owner |\n| --- | --- | --- | --- |\n| {{requirement_id}} - {{requirement_text}} | {{compliant_partial_or_exception}} | {{evidence_reference}} | {{owner}} |\n",
        },
        DocumentSnippetInfo {
            id: "tender-submission-checklist",
            label: "Tender submission checklist",
            kind: "procurement",
            summary: "Checklist for mandatory tender attachments, sign-offs, and submission readiness.",
            body: "## Mandatory Submission Checklist\n\n- [ ] Signed submission form\n- [ ] Pricing schedule\n- [ ] Technical response\n- [ ] Compliance declarations\n- [ ] Insurance, tax, or registration evidence\n- [ ] Authorized sign-off by {{approver}}\n",
        },
        DocumentSnippetInfo {
            id: "tutorial-step",
            label: "Tutorial step",
            kind: "delivery",
            summary: "Repeatable instruction block for tutorials and training guides.",
            body: "### Step {{step_number}}: {{step_title}}\n\n**Goal:** {{step_goal}}\n\n1. {{instruction_1}}\n2. {{instruction_2}}\n3. {{instruction_3}}\n\n**Check:** {{completion_check}}\n\n**If this fails:** {{troubleshooting_tip}}\n",
        },
        DocumentSnippetInfo {
            id: "risk-register",
            label: "Risk register",
            kind: "governance",
            summary: "Standard business risk table for proposals, RFPs, tenders, and plans.",
            body: "## Risk Register\n\n| Risk | Impact | Likelihood | Mitigation | Owner |\n| --- | --- | --- | --- | --- |\n| {{risk}} | {{impact}} | {{likelihood}} | {{mitigation}} | {{owner}} |\n",
        },
        DocumentSnippetInfo {
            id: "review-handoff",
            label: "Review handoff",
            kind: "review",
            summary: "Review instructions that keep unresolved assumptions visible before export.",
            body: "## Review Handoff\n\n- Confirm all client names, figures, dates, and claims.\n- Resolve placeholders before sending: {{open_placeholders}}.\n- Confirm legal, finance, and delivery owner approvals where required.\n- Final reviewer: {{reviewer}}.\n",
        },
    ]
}

fn document_snippet_catalog_entries(workspace: &Path) -> Result<Vec<DocumentSnippetEntry>, String> {
    let mut snippets = document_snippet_catalog()
        .into_iter()
        .map(built_in_snippet_entry)
        .collect::<Vec<_>>();
    snippets.extend(read_workspace_snippet_entries(workspace)?);
    snippets.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(snippets)
}

fn built_in_snippet_entry(snippet: DocumentSnippetInfo) -> DocumentSnippetEntry {
    DocumentSnippetEntry {
        id: snippet.id.to_string(),
        label: snippet.label.to_string(),
        kind: snippet.kind.to_string(),
        summary: snippet.summary.to_string(),
        body: snippet.body.to_string(),
        source: "builtin".to_string(),
        path: None,
    }
}

fn workspace_snippets_dir(workspace: &Path) -> PathBuf {
    workspace.join(".neditor").join("snippets")
}

fn read_workspace_snippet_entries(workspace: &Path) -> Result<Vec<DocumentSnippetEntry>, String> {
    let dir = workspace_snippets_dir(workspace);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut snippets = Vec::new();
    let entries = fs::read_dir(&dir)
        .map_err(|err| format!("Could not read workspace snippets {}: {err}", dir.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|err| format!("Could not read workspace snippet entry: {err}"))?;
        let path = entry.path();
        if !path.is_file()
            || path
                .extension()
                .and_then(|value| value.to_str())
                .map_or(true, |ext| !ext.eq_ignore_ascii_case("md"))
        {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|err| format!("Could not read workspace snippet {}: {err}", path.display()))?;
        snippets.extend(parse_workspace_snippet_file(&path, &text));
    }
    Ok(snippets)
}

fn parse_workspace_snippet_file(path: &Path, text: &str) -> Vec<DocumentSnippetEntry> {
    let kind = path
        .file_stem()
        .and_then(|value| value.to_str())
        .and_then(slugify_workspace_snippet_part)
        .unwrap_or_else(|| "workspace".to_string());
    let lines = text.lines().collect::<Vec<_>>();
    let mut heading_indexes = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| workspace_h2_heading_label(line).map(|label| (index, label)))
        .collect::<Vec<_>>();
    if heading_indexes.is_empty() {
        let body = text.trim();
        if body.is_empty() {
            return Vec::new();
        }
        let label = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(humanize_workspace_snippet_label)
            .unwrap_or_else(|| "Workspace snippet".to_string());
        return vec![workspace_snippet_entry(&kind, &label, body, path, 0)];
    }
    heading_indexes.push((lines.len(), String::new()));
    let mut snippets = Vec::new();
    for pair in heading_indexes.windows(2) {
        let (start, label) = &pair[0];
        let (end, _) = pair[1];
        let body = lines[*start..end].join("\n").trim().to_string();
        if body.is_empty() {
            continue;
        }
        snippets.push(workspace_snippet_entry(
            &kind,
            label,
            &body,
            path,
            snippets.len(),
        ));
    }
    snippets
}

fn workspace_snippet_entry(
    kind: &str,
    label: &str,
    body: &str,
    path: &Path,
    suffix: usize,
) -> DocumentSnippetEntry {
    let id_base = format!(
        "{}-{}",
        kind,
        slugify_workspace_snippet_part(label).unwrap_or_else(|| "snippet".to_string())
    );
    let id = if suffix == 0 {
        id_base
    } else {
        format!("{id_base}-{suffix}")
    };
    DocumentSnippetEntry {
        id,
        label: label.to_string(),
        kind: kind.to_string(),
        summary: workspace_snippet_summary(body, path),
        body: format!("{}\n", body.trim()),
        source: "workspace".to_string(),
        path: Some(path_to_display(path)),
    }
}

fn workspace_h2_heading_label(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("##") || trimmed.starts_with("###") {
        return None;
    }
    let label = trimmed.trim_start_matches('#').trim();
    if label.is_empty() {
        None
    } else {
        Some(label.chars().take(120).collect())
    }
}

fn workspace_snippet_summary(body: &str, path: &Path) -> String {
    body.lines()
        .map(|line| line.trim())
        .find(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with('|')
                && !line.starts_with("- ")
                && !line.starts_with("* ")
        })
        .map(|line| line.chars().take(180).collect())
        .unwrap_or_else(|| {
            format!(
                "Workspace snippet from {}.",
                path.file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("snippets")
            )
        })
}

fn slugify_workspace_snippet_part(value: &str) -> Option<String> {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
        if slug.len() >= 80 {
            break;
        }
    }
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        None
    } else {
        Some(slug)
    }
}

fn humanize_workspace_snippet_label(value: &str) -> String {
    value
        .split(|ch: char| matches!(ch, '-' | '_' | '.'))
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn snippet_matches_query(snippet: &DocumentSnippetEntry, query: &str) -> bool {
    snippet.id.to_ascii_lowercase().contains(query)
        || snippet.label.to_ascii_lowercase().contains(query)
        || snippet.kind.to_ascii_lowercase().contains(query)
        || snippet.summary.to_ascii_lowercase().contains(query)
        || snippet.body.to_ascii_lowercase().contains(query)
}

fn snippets_text_report(snippets: &[DocumentSnippetEntry]) -> String {
    if snippets.is_empty() {
        return "No NEditor document snippets match those filters.".to_string();
    }
    let mut lines = vec![format!("NEditor document snippets ({}):", snippets.len())];
    for snippet in snippets {
        lines.push(format!(
            "  - {} [{}] {}: {}",
            snippet.id, snippet.kind, snippet.label, snippet.summary
        ));
    }
    lines.push("Use `ned snippets --workspace . --markdown <id>` to print a built-in or .neditor/snippets Markdown part; add `--fill-profile` to merge saved business identity values.".to_string());
    lines.join("\n")
}

fn parse_profile_assignment(raw: &str) -> Result<(String, String), String> {
    let (key, value) = raw
        .split_once('=')
        .or_else(|| raw.split_once(':'))
        .ok_or_else(|| format!("Profile values must be key=value, got '{raw}'"))?;
    let key = normalize_profile_key(key);
    if key.is_empty() {
        return Err("Profile value key cannot be empty".to_string());
    }
    Ok((key, value.trim().to_string()))
}

fn normalize_profile_key(key: &str) -> String {
    let mut spaced = String::new();
    let mut previous_was_lower_or_digit = false;
    for character in key.trim().chars() {
        if character == '_' || character == '-' {
            spaced.push(' ');
            previous_was_lower_or_digit = false;
            continue;
        }
        if character.is_ascii_uppercase() && previous_was_lower_or_digit {
            spaced.push(' ');
        }
        previous_was_lower_or_digit = character.is_ascii_lowercase() || character.is_ascii_digit();
        spaced.push(character);
    }
    spaced
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn set_business_profile_field(
    profile: &mut BusinessProfile,
    key: &str,
    value: &str,
) -> Result<(), String> {
    match canonical_profile_field(key)? {
        "fullName" => profile.full_name = value.to_string(),
        "email" => profile.email = value.to_string(),
        "phone" => profile.phone = value.to_string(),
        "roleTitle" => profile.role_title = value.to_string(),
        "companyName" => profile.company_name = value.to_string(),
        "companyAddress" => profile.company_address = value.to_string(),
        "website" => profile.website = value.to_string(),
        "industry" => profile.industry = value.to_string(),
        "defaultClientName" => profile.default_client_name = value.to_string(),
        "brandVoice" => profile.brand_voice = value.to_string(),
        _ => unreachable!("canonical profile field list is exhaustive"),
    }
    Ok(())
}

fn business_profile_fields() -> Vec<&'static str> {
    vec![
        "fullName",
        "email",
        "phone",
        "roleTitle",
        "companyName",
        "companyAddress",
        "website",
        "industry",
        "defaultClientName",
        "brandVoice",
    ]
}

fn canonical_profile_field(key: &str) -> Result<&'static str, String> {
    let normalized = normalize_profile_key(key);
    match normalized.as_str() {
        "full name" | "name" | "owner" | "prepared by" => Ok("fullName"),
        "email" | "email address" => Ok("email"),
        "phone" | "phone number" | "telephone" => Ok("phone"),
        "role title" | "role" | "title" | "job title" => Ok("roleTitle"),
        "company name" | "company" | "organization" | "organisation" => Ok("companyName"),
        "company address" | "address" | "mailing address" => Ok("companyAddress"),
        "website" | "web site" | "url" => Ok("website"),
        "industry" | "sector" => Ok("industry"),
        "default client name" | "default client" | "client" | "client name" => {
            Ok("defaultClientName")
        }
        "brand voice" | "voice" | "tone" => Ok("brandVoice"),
        other => Err(format!(
            "Unknown profile field '{other}'. Supported fields: {}",
            business_profile_fields().join(", ")
        )),
    }
}

fn business_profile_field_value(
    profile: &BusinessProfile,
    key: &str,
) -> Result<(&'static str, String), String> {
    let canonical = canonical_profile_field(key)?;
    let value = match canonical {
        "fullName" => &profile.full_name,
        "email" => &profile.email,
        "phone" => &profile.phone,
        "roleTitle" => &profile.role_title,
        "companyName" => &profile.company_name,
        "companyAddress" => &profile.company_address,
        "website" => &profile.website,
        "industry" => &profile.industry,
        "defaultClientName" => &profile.default_client_name,
        "brandVoice" => &profile.brand_voice,
        _ => unreachable!("canonical profile field list is exhaustive"),
    };
    Ok((canonical, value.clone()))
}

fn business_profile_field_catalog() -> Vec<Value> {
    vec![
        json!({"field": "fullName", "label": "Full name", "aliases": ["name", "owner", "preparedBy"], "usedFor": "sender, author, reviewer, and prepared-by placeholders"}),
        json!({"field": "email", "label": "Email", "aliases": ["emailAddress"], "usedFor": "contact blocks, cover pages, and agent handoff metadata"}),
        json!({"field": "phone", "label": "Phone", "aliases": ["phoneNumber", "telephone"], "usedFor": "contact blocks and submission forms"}),
        json!({"field": "roleTitle", "label": "Role title", "aliases": ["role", "title", "jobTitle"], "usedFor": "prepared-by lines and reviewer handoffs"}),
        json!({"field": "companyName", "label": "Company name", "aliases": ["company", "organization", "organisation"], "usedFor": "company boilerplate, proposals, and procurement responses"}),
        json!({"field": "companyAddress", "label": "Company address", "aliases": ["address", "mailingAddress"], "usedFor": "cover pages, letters, tenders, and official submissions"}),
        json!({"field": "website", "label": "Website", "aliases": ["webSite", "url"], "usedFor": "contact blocks, publishing metadata, and capability statements"}),
        json!({"field": "industry", "label": "Industry", "aliases": ["sector"], "usedFor": "company overview snippets and proposal positioning"}),
        json!({"field": "defaultClientName", "label": "Default client name", "aliases": ["defaultClient", "client", "clientName"], "usedFor": "starter documents, Docs Live placeholders, and reusable snippets"}),
        json!({"field": "brandVoice", "label": "Brand voice", "aliases": ["voice", "tone"], "usedFor": "Docs Live drafting, snippets, humanization, and agent handoffs"}),
    ]
}

fn business_profile_fields_text_report() -> String {
    let mut lines = vec!["NEditor business profile fields:".to_string()];
    for field in business_profile_field_catalog() {
        let name = field["field"].as_str().unwrap_or_default();
        let label = field["label"].as_str().unwrap_or_default();
        let used_for = field["usedFor"].as_str().unwrap_or_default();
        let aliases = field["aliases"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        lines.push(format!(
            "  - {name}: {label}. Aliases: {aliases}. Used for {used_for}."
        ));
    }
    lines.push("Use `ned profile --set field=value` to update values and `ned profile --get field` to print one value.".to_string());
    lines.join("\n")
}

fn read_business_profile(path: &Path) -> Result<BusinessProfile, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("Could not read business profile {}: {err}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("Could not parse business profile {}: {err}", path.display()))
}

fn workspace_business_profile_path(workspace: &Path) -> PathBuf {
    workspace.join(".neditor").join("business-profile.json")
}

fn read_workspace_profile_if_requested(
    workspace: &Path,
    fill_profile: bool,
) -> Result<(PathBuf, Option<BusinessProfile>), String> {
    let profile_path = workspace_business_profile_path(workspace);
    let profile = if fill_profile && profile_path.exists() {
        Some(read_business_profile(&profile_path)?)
    } else {
        None
    };
    Ok((profile_path, profile))
}

fn write_business_profile(path: &Path, profile: &BusinessProfile) -> Result<(), String> {
    let text = serde_json::to_string_pretty(profile).map_err(|err| err.to_string())?;
    fs::write(path, format!("{text}\n"))
        .map_err(|err| format!("Could not write business profile {}: {err}", path.display()))
}

fn profile_value(value: &str, placeholder: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        format!("{{{{{placeholder}}}}}")
    } else {
        trimmed.to_string()
    }
}

fn fill_business_profile_placeholders(markdown: &str, profile: &BusinessProfile) -> String {
    let mut output = String::new();
    let mut remaining = markdown;
    while let Some(start) = remaining.find("{{") {
        output.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        if let Some(end) = after_start.find("}}") {
            let placeholder = after_start[..end].trim();
            if let Some(value) = business_profile_placeholder_value(profile, placeholder) {
                output.push_str(&value);
            } else {
                output.push_str("{{");
                output.push_str(&after_start[..end]);
                output.push_str("}}");
            }
            remaining = &after_start[end + 2..];
        } else {
            output.push_str(&remaining[start..]);
            remaining = "";
        }
    }
    output.push_str(remaining);
    output
}

fn business_profile_placeholder_value(
    profile: &BusinessProfile,
    placeholder: &str,
) -> Option<String> {
    let normalized = normalize_business_profile_placeholder_key(placeholder);
    let value = match normalized.as_str() {
        "full name" | "name" | "owner" | "prepared by" | "author" | "reviewer" | "approver" => {
            &profile.full_name
        }
        "email" | "email address" => &profile.email,
        "phone" | "phone number" | "telephone" => &profile.phone,
        "role title" | "role" | "title" | "job title" => &profile.role_title,
        "company name" | "company" | "organization" | "organisation" => &profile.company_name,
        "company address" | "address" | "mailing address" => &profile.company_address,
        "website" | "web site" | "url" => &profile.website,
        "industry" | "sector" => &profile.industry,
        "default client name" | "default client" | "client" | "client name" => {
            &profile.default_client_name
        }
        "brand voice" | "voice" | "tone" => &profile.brand_voice,
        _ => return None,
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_business_profile_placeholder_key(placeholder: &str) -> String {
    let trimmed = placeholder.trim();
    if let Some(rest) = trimmed.strip_prefix("profile.") {
        return normalize_profile_key(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("company.") {
        return match normalize_profile_key(rest).as_str() {
            "name" => "company name".to_string(),
            "address" | "mailing address" => "company address".to_string(),
            "website" | "web site" | "url" => "website".to_string(),
            "industry" | "sector" => "industry".to_string(),
            other => other.to_string(),
        };
    }
    if let Some(rest) = trimmed.strip_prefix("client.") {
        return match normalize_profile_key(rest).as_str() {
            "name" | "default name" => "default client name".to_string(),
            other => other.to_string(),
        };
    }
    normalize_profile_key(trimmed)
}

fn business_profile_placeholder_text(profile: &BusinessProfile) -> String {
    [
        ("fullName", &profile.full_name),
        ("email", &profile.email),
        ("phone", &profile.phone),
        ("roleTitle", &profile.role_title),
        ("companyName", &profile.company_name),
        ("companyAddress", &profile.company_address),
        ("website", &profile.website),
        ("industry", &profile.industry),
        ("defaultClientName", &profile.default_client_name),
        ("brandVoice", &profile.brand_voice),
    ]
    .iter()
    .map(|(key, value)| format!("{key}: {}", profile_value(value, key)))
    .collect::<Vec<_>>()
    .join("\n")
}

fn business_profile_markdown(profile: &BusinessProfile) -> String {
    vec![
        "## Business Identity".to_string(),
        "".to_string(),
        format!(
            "**Prepared by:** {}, {}",
            profile_value(&profile.full_name, "fullName"),
            profile_value(&profile.role_title, "roleTitle")
        ),
        "".to_string(),
        format!(
            "**Company:** {}",
            profile_value(&profile.company_name, "companyName")
        ),
        format!(
            "**Address:** {}",
            profile_value(&profile.company_address, "companyAddress")
        ),
        "".to_string(),
        format!("**Email:** {}", profile_value(&profile.email, "email")),
        format!("**Phone:** {}", profile_value(&profile.phone, "phone")),
        format!(
            "**Website:** {}",
            profile_value(&profile.website, "website")
        ),
        "".to_string(),
        format!(
            "**Industry:** {}",
            profile_value(&profile.industry, "industry")
        ),
        format!(
            "**Default client:** {}",
            profile_value(&profile.default_client_name, "defaultClientName")
        ),
        format!(
            "**Brand voice:** {}",
            profile_value(&profile.brand_voice, "brandVoice")
        ),
    ]
    .join("\n")
}

fn business_profile_text_report(
    path: &Path,
    existed: bool,
    should_write: bool,
    dry_run: bool,
    profile: &BusinessProfile,
) -> String {
    let status = if should_write && dry_run {
        "would write"
    } else if should_write {
        "written"
    } else if existed {
        "loaded"
    } else {
        "not initialized"
    };
    [
        format!("NEditor business profile: {status}"),
        format!("Profile path: {}", path.display()),
        "Use `ned profile --init --set companyName=... --set fullName=...` to create or update it.".to_string(),
        "Use `ned profile --markdown` to print a reusable contact block or `--placeholders` for Docs Live.".to_string(),
        "".to_string(),
        business_profile_placeholder_text(profile),
    ]
    .join("\n")
}

fn append_cli_block(existing: &str, value: &str) -> String {
    if existing.trim().is_empty() {
        value.trim().to_string()
    } else {
        format!("{}\n\n{}", existing.trim(), value.trim())
    }
}

fn infer_rfp_source_type(source: &str, url: Option<&str>) -> String {
    if url.is_some() || source.starts_with("http://") || source.starts_with("https://") {
        return "url".to_string();
    }
    if source == "-" {
        return "markdown".to_string();
    }
    match Path::new(source)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "pdf" => "pdf".to_string(),
        "docx" => "docx".to_string(),
        _ => "markdown".to_string(),
    }
}

fn write_cli_markdown_output(path: &Path, markdown: &str) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Could not create directory {}: {err}", parent.display()))?;
    }
    fs::write(path, format!("{}\n", markdown.trim_end()))
        .map_err(|err| format!("Could not write {}: {err}", path.display()))
}

fn analyze_rfp_text(
    imported: &crate::rfp_import::ImportRfpSourceResponse,
    profile: &BusinessProfile,
) -> RfpCliAnalysis {
    let significant_lines = imported
        .text
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, normalize_cli_whitespace(line)))
        .filter(|(_, line)| !line.is_empty())
        .collect::<Vec<_>>();
    let mut requirements = significant_lines
        .iter()
        .filter(|(_, line)| is_rfp_requirement_line(line))
        .enumerate()
        .map(|(index, (line_number, line))| RfpCliRequirement {
            id: format!("RFP-REQ-{:03}", index + 1),
            text: trim_requirement_marker(line),
            category: rfp_requirement_category(line),
            source_line: *line_number,
        })
        .collect::<Vec<_>>();
    requirements = dedupe_rfp_requirements(requirements);
    if requirements.is_empty() {
        requirements = significant_lines
            .iter()
            .take(6)
            .enumerate()
            .map(|(index, (line_number, line))| RfpCliRequirement {
                id: format!("RFP-REQ-{:03}", index + 1),
                text: trim_requirement_marker(line),
                category: rfp_requirement_category(line),
                source_line: *line_number,
            })
            .collect();
    }
    for (index, requirement) in requirements.iter_mut().enumerate() {
        requirement.id = format!("RFP-REQ-{:03}", index + 1);
    }

    let timelines = extract_rfp_lines(
        &significant_lines,
        &[
            "deadline",
            "due",
            "schedule",
            "timeline",
            "milestone",
            "weeks",
            "months",
            "days",
            "implementation",
            "submission",
        ],
        8,
    );
    let budget_hints = extract_rfp_lines(
        &significant_lines,
        &[
            "budget",
            "price",
            "pricing",
            "cost",
            "fee",
            "commercial",
            "payment",
            "invoice",
            "rate",
            "$",
        ],
        8,
    );
    let evaluation_criteria = extract_rfp_lines(
        &significant_lines,
        &[
            "evaluation",
            "score",
            "scoring",
            "weight",
            "criteria",
            "points",
            "award",
            "selection",
            "technical merit",
            "best value",
        ],
        8,
    );
    let mandatory_attachments = extract_rfp_lines(
        &significant_lines,
        &[
            "attachment",
            "appendix",
            "form",
            "certificate",
            "insurance",
            "tax",
            "license",
            "registration",
            "declaration",
            "signature",
            "signed",
            "mandatory document",
        ],
        10,
    );
    let capabilities = infer_rfp_capabilities(&requirements, &imported.text, profile);
    let stated_intent = infer_rfp_stated_intent(&significant_lines, &requirements);
    let implied_intent = infer_rfp_implied_intent(
        &requirements,
        &timelines,
        &budget_hints,
        &evaluation_criteria,
        &mandatory_attachments,
    );
    let compliance_rows = requirements
        .iter()
        .map(|requirement| build_rfp_compliance_row(requirement, profile))
        .collect::<Vec<_>>();
    let rows_needing_evidence = compliance_rows
        .iter()
        .filter(|row| row.evidence_needed.contains("Attach"))
        .count();
    let verification_summary = RfpCliVerificationSummary {
        total_requirements: requirements.len(),
        compliance_rows: compliance_rows.len(),
        all_requirements_mapped: !requirements.is_empty()
            && requirements.len() == compliance_rows.len(),
        rows_needing_evidence,
        checklist: rfp_verification_checklist(
            requirements.len(),
            rows_needing_evidence,
            &mandatory_attachments,
            &evaluation_criteria,
        ),
    };
    let risks = infer_rfp_risks(
        &requirements,
        &timelines,
        &budget_hints,
        &mandatory_attachments,
    );
    let questions = infer_rfp_questions(
        &requirements,
        &timelines,
        &budget_hints,
        &evaluation_criteria,
        &mandatory_attachments,
    );
    let warnings = imported.warnings.clone();
    let completeness_score = rfp_completeness_score(
        requirements.len(),
        capabilities.len(),
        timelines.len(),
        budget_hints.len(),
        evaluation_criteria.len(),
        mandatory_attachments.len(),
        warnings.len(),
    );
    RfpCliAnalysis {
        source: RfpCliSource {
            kind: imported.source_type.clone(),
            title: imported.title.clone(),
            path: imported.path.clone(),
            url: imported.url.clone(),
            extraction_method: imported.extraction_method.clone(),
            line_count: significant_lines.len(),
            word_count: imported.text.split_whitespace().count(),
            warnings,
        },
        requirements,
        compliance_rows,
        verification_summary,
        capabilities,
        stated_intent,
        implied_intent,
        timelines,
        budget_hints,
        evaluation_criteria,
        mandatory_attachments,
        risks,
        questions,
        completeness_score,
    }
}

fn normalize_cli_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_rfp_requirement_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let starts_like_requirement = line
        .trim_start()
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit() || matches!(ch, '-' | '*' | '•'));
    starts_like_requirement
        || [
            "must",
            "shall",
            "required",
            "requirement",
            "vendor",
            "proposer",
            "supplier",
            "contractor",
            "respondent",
            "provide",
            "submit",
            "demonstrate",
            "include",
            "comply",
            "certify",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn trim_requirement_marker(line: &str) -> String {
    line.trim()
        .trim_start_matches(|ch: char| {
            ch.is_ascii_digit() || matches!(ch, '.' | ')' | '-' | '*' | '•' | ':')
        })
        .trim()
        .to_string()
}

fn rfp_requirement_category(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    let category = if contains_any(
        &lower,
        &[
            "security",
            "privacy",
            "data",
            "soc",
            "compliance",
            "control",
            "certif",
        ],
    ) {
        "Compliance"
    } else if contains_any(
        &lower,
        &["price", "pricing", "cost", "fee", "commercial", "payment"],
    ) {
        "Pricing"
    } else if contains_any(
        &lower,
        &[
            "deadline",
            "timeline",
            "schedule",
            "days",
            "weeks",
            "months",
            "milestone",
            "implementation",
        ],
    ) {
        "Delivery Plan"
    } else if contains_any(
        &lower,
        &[
            "experience",
            "reference",
            "team",
            "staff",
            "resume",
            "cv",
            "case study",
        ],
    ) {
        "Team and Experience"
    } else if contains_any(
        &lower,
        &[
            "technical",
            "solution",
            "system",
            "integration",
            "support",
            "training",
            "implementation",
        ],
    ) {
        "Technical Solution"
    } else {
        "Requirements Analysis"
    };
    category.to_string()
}

fn dedupe_rfp_requirements(requirements: Vec<RfpCliRequirement>) -> Vec<RfpCliRequirement> {
    let mut seen = Vec::<String>::new();
    let mut deduped = Vec::new();
    for requirement in requirements {
        let key = requirement.text.to_ascii_lowercase();
        if seen.iter().any(|seen_key| seen_key == &key) {
            continue;
        }
        seen.push(key);
        deduped.push(requirement);
    }
    deduped
}

fn extract_rfp_lines(lines: &[(usize, String)], needles: &[&str], limit: usize) -> Vec<String> {
    lines
        .iter()
        .filter(|(_, line)| contains_any(&line.to_ascii_lowercase(), needles))
        .map(|(_, line)| line.clone())
        .take(limit)
        .collect()
}

fn infer_rfp_capabilities(
    requirements: &[RfpCliRequirement],
    text: &str,
    profile: &BusinessProfile,
) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let mut capabilities = Vec::new();
    for (needle, capability) in [
        ("security", "Security controls and data protection"),
        ("support", "Support operations and service management"),
        (
            "implementation",
            "Implementation planning and delivery governance",
        ),
        ("training", "Training and adoption enablement"),
        ("integration", "Systems integration and technical delivery"),
        ("pricing", "Commercial pricing and assumption management"),
        (
            "reference",
            "Relevant customer references and delivery proof",
        ),
    ] {
        if lower.contains(needle)
            || requirements
                .iter()
                .any(|requirement| requirement.text.to_ascii_lowercase().contains(needle))
        {
            capabilities.push(capability.to_string());
        }
    }
    if capabilities.is_empty() {
        capabilities.push(format!(
            "{} capability narrative to be completed from the RFP source and evidence pack",
            profile_fallback(&profile.company_name, "Bid team")
        ));
    }
    capabilities
}

fn infer_rfp_stated_intent(
    lines: &[(usize, String)],
    requirements: &[RfpCliRequirement],
) -> Vec<String> {
    let mut intent = extract_rfp_lines(
        lines,
        &[
            "purpose",
            "objective",
            "seeks",
            "seeking",
            "scope",
            "goal",
            "outcome",
            "award",
            "improve",
            "reduce",
        ],
        5,
    );
    if intent.is_empty() {
        intent = requirements
            .iter()
            .take(3)
            .map(|requirement| format!("Buyer explicitly asks for {}", requirement.text))
            .collect();
    }
    if intent.is_empty() {
        intent.push("Add stated buyer intent from the RFP overview, purpose, objectives, scope, and award language.".to_string());
    }
    intent
}

fn infer_rfp_implied_intent(
    requirements: &[RfpCliRequirement],
    timelines: &[String],
    budget_hints: &[String],
    evaluation_criteria: &[String],
    mandatory_attachments: &[String],
) -> Vec<String> {
    let mut intent = Vec::new();
    if !evaluation_criteria.is_empty() {
        intent.push("The buyer needs an easily scored response; mirror evaluation criteria, labels, and evidence in the executive summary and compliance matrix.".to_string());
    }
    if !timelines.is_empty() {
        intent.push("Timeline language implies delivery-risk sensitivity; make milestones, mobilization, and governance concrete.".to_string());
    }
    if !budget_hints.is_empty() {
        intent.push("Pricing language implies commercial scrutiny; state assumptions, exclusions, payment terms, and value drivers clearly.".to_string());
    }
    if !mandatory_attachments.is_empty()
        || requirements
            .iter()
            .any(|requirement| requirement.category == "Compliance")
    {
        intent.push("The buyer is managing procurement risk; include declarations, certificates, controls, and reviewer sign-off instead of broad claims.".to_string());
    }
    if requirements
        .iter()
        .any(|requirement| requirement.category == "Team and Experience")
    {
        intent.push("The buyer needs confidence in delivery capacity; foreground relevant team credentials, references, and comparable work.".to_string());
    }
    if intent.is_empty() {
        intent.push("Infer unstated priorities from criteria, mandatory evidence, timeline pressure, budget language, and risk signals.".to_string());
    }
    intent
}

fn build_rfp_compliance_row(
    requirement: &RfpCliRequirement,
    profile: &BusinessProfile,
) -> RfpCliComplianceRow {
    let owner = profile_fallback(&profile.full_name, "Bid owner");
    let response_section = match requirement.category.as_str() {
        "Compliance" => "Compliance and Security",
        "Pricing" => "Pricing and Assumptions",
        "Delivery Plan" => "Implementation Plan and Timeline",
        "Team and Experience" => "Team and Experience",
        "Technical Solution" => "Technical Response",
        _ => "Requirements Analysis",
    }
    .to_string();
    let requirement_text = requirement
        .text
        .trim_end_matches(|ch| matches!(ch, '.' | ';' | ':'));
    let suggested_response = match requirement.category.as_str() {
        "Compliance" => format!(
            "Specific requirement: {}. Respond with mapped controls, certificates, policies, and named evidence owner.",
            requirement_text
        ),
        "Pricing" => format!(
            "Specific requirement: {}. State pricing, payment terms, assumptions, exclusions, and validity period.",
            requirement_text
        ),
        "Delivery Plan" => format!(
            "Specific requirement: {}. Provide milestone plan, dependencies, risks, and governance.",
            requirement_text
        ),
        _ => format!(
            "Specific requirement: {}. Answer directly, cite evidence, and avoid generic proposal copy.",
            requirement_text
        ),
    };
    RfpCliComplianceRow {
        id: requirement.id.clone(),
        requirement: requirement.text.clone(),
        category: requirement.category.clone(),
        compliance_status: "Needs evidence review".to_string(),
        response_section,
        suggested_response,
        evidence_needed: format!(
            "Attach source proof, owner approval, and reviewer sign-off for {}",
            requirement.id
        ),
        owner,
        verification: format!(
            "Compliance Matrix row maps source line {} and needs suggested answer review.",
            requirement.source_line
        ),
        source_line: requirement.source_line,
    }
}

fn rfp_verification_checklist(
    requirements: usize,
    rows_needing_evidence: usize,
    attachments: &[String],
    criteria: &[String],
) -> Vec<String> {
    vec![
        format!(
            "{requirements} extracted requirement(s) mapped to {requirements} compliance row(s)."
        ),
        "Every extracted requirement has a response section and owner.".to_string(),
        format!("{rows_needing_evidence} row(s) need evidence attachment and reviewer sign-off."),
        if attachments.is_empty() {
            "Review the source for mandatory forms, certificates, declarations, and signatures."
                .to_string()
        } else {
            format!(
                "Track {} mandatory attachment hint(s) through submission.",
                attachments.len()
            )
        },
        if criteria.is_empty() {
            "Confirm scoring criteria and buyer priorities before final response review."
                .to_string()
        } else {
            format!(
                "Mirror {} evaluation criteria hint(s) in the response structure.",
                criteria.len()
            )
        },
    ]
}

fn infer_rfp_risks(
    requirements: &[RfpCliRequirement],
    timelines: &[String],
    budget_hints: &[String],
    attachments: &[String],
) -> Vec<String> {
    let mut risks = Vec::new();
    if requirements.len() > 30 {
        risks.push(
            "Large requirement set; assign matrix owners and verify every row before submission."
                .to_string(),
        );
    }
    if !timelines.is_empty() {
        risks.push(
            "Timeline obligations need delivery-owner validation before commitments are made."
                .to_string(),
        );
    }
    if !budget_hints.is_empty() {
        risks.push("Commercial assumptions need finance approval before submission.".to_string());
    }
    if !attachments.is_empty() {
        risks.push("Mandatory attachments can block submission if certificates, declarations, or signatures are missing.".to_string());
    }
    if risks.is_empty() {
        risks.push("Review source RFP for exceptions, addenda, submission portal rules, and attachments before final delivery.".to_string());
    }
    risks
}

fn infer_rfp_questions(
    requirements: &[RfpCliRequirement],
    timelines: &[String],
    budget_hints: &[String],
    criteria: &[String],
    attachments: &[String],
) -> Vec<String> {
    let mut questions = Vec::new();
    if requirements.is_empty() {
        questions.push(
            "Can the full RFP text be imported or pasted so every requirement is mapped?"
                .to_string(),
        );
    }
    if timelines.is_empty() {
        questions.push(
            "What is the submission deadline, question deadline, and expected delivery timeline?"
                .to_string(),
        );
    }
    if budget_hints.is_empty() {
        questions.push(
            "What pricing format, payment terms, or commercial assumptions does the buyer require?"
                .to_string(),
        );
    }
    if criteria.is_empty() {
        questions.push(
            "What evaluation criteria and scoring weights should the response mirror?".to_string(),
        );
    }
    if attachments.is_empty() {
        questions.push(
            "Which certificates, forms, declarations, references, or signatures are mandatory?"
                .to_string(),
        );
    }
    questions
}

fn rfp_completeness_score(
    requirements: usize,
    capabilities: usize,
    timelines: usize,
    budget_hints: usize,
    criteria: usize,
    attachments: usize,
    warnings: usize,
) -> u8 {
    let score = 20
        + requirements.min(12) * 4
        + capabilities.min(6) * 3
        + timelines.min(4) * 3
        + budget_hints.min(4) * 3
        + criteria.min(4) * 3
        + attachments.min(5) * 2;
    score.saturating_sub(warnings * 5).min(100) as u8
}

fn rfp_cli_compliance_matrix_markdown(analysis: &RfpCliAnalysis) -> String {
    let mut lines = vec![
        "## Compliance Matrix".to_string(),
        "".to_string(),
        "| ID | Requirement | Category | Compliance status | Response section | Suggested response | Evidence / proof | Verification |".to_string(),
        "| --- | --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];
    if analysis.compliance_rows.is_empty() {
        lines.push("| RFP-REQ-001 | Import or paste the RFP text to populate requirements. | Intake | Needs evidence review | Requirements Analysis | Analyze the full source RFP before drafting. | Source RFP text | Not verified. |".to_string());
    } else {
        for row in &analysis.compliance_rows {
            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                table_cell(&row.id),
                table_cell(&row.requirement),
                table_cell(&row.category),
                table_cell(&row.compliance_status),
                table_cell(&row.response_section),
                table_cell(&row.suggested_response),
                table_cell(&row.evidence_needed),
                table_cell(&row.verification),
            ));
        }
    }
    lines.join("\n")
}

fn rfp_cli_response_markdown(
    analysis: &RfpCliAnalysis,
    profile: &BusinessProfile,
    context_notes: &str,
) -> String {
    let company = profile_fallback(&profile.company_name, "Bid team");
    let client = profile_fallback(&profile.default_client_name, &analysis.source.title);
    let prepared_by = profile_fallback(&profile.full_name, "Response owner");
    let mut lines = vec![
        "---".to_string(),
        format!(
            "title: {}",
            yaml_scalar(&format!("RFP response for {client}"))
        ),
        "status: draft".to_string(),
        "documentType: RFP response".to_string(),
        format!("company: {}", yaml_scalar(&company)),
        format!("preparedBy: {}", yaml_scalar(&prepared_by)),
        format!("rfpSource: {}", yaml_scalar(&analysis.source.title)),
    ];
    if let Some(url) = analysis.source.url.as_deref().filter(|url| !url.is_empty()) {
        lines.push(format!("rfpUrl: {}", yaml_scalar(url)));
    }
    lines.extend([
        "toc: true".to_string(),
        "---".to_string(),
        "".to_string(),
        format!("# RFP response for {client}"),
        "".to_string(),
        rfp_cli_compliance_checklist_markdown(analysis),
        "".to_string(),
        "[TOC]".to_string(),
        "".to_string(),
        rfp_cli_proposal_planning_prompt_markdown(analysis, context_notes),
        "".to_string(),
        "## Proposal Outline".to_string(),
        "".to_string(),
    ]);
    lines.extend(rfp_cli_proposal_outline_bullets(analysis));
    lines.extend([
        "".to_string(),
        rfp_cli_evaluator_section_drafts_markdown(analysis, &company, &client, context_notes),
        "".to_string(),
        "## Executive Response".to_string(),
        "".to_string(),
        format!("{company} has prepared a responsive draft for {client}. This response mirrors extracted RFP requirements, maps every detected requirement into a compliance matrix, and keeps evidence review visible before submission."),
        "".to_string(),
        "## RFP Intake Summary".to_string(),
        "".to_string(),
        format!("- Source type: {}", analysis.source.kind.to_uppercase()),
        format!("- Source title: {}", analysis.source.title),
        format!("- Extraction method: {}", analysis.source.extraction_method),
        format!("- Extracted requirements: {}", analysis.requirements.len()),
        format!("- Completeness score: {}/100", analysis.completeness_score),
        format!("- Source size: {} words across {} non-empty lines", analysis.source.word_count, analysis.source.line_count),
    ]);
    if !context_notes.trim().is_empty() {
        lines.extend([
            "".to_string(),
            "### Response Context and Decision Notes".to_string(),
            "".to_string(),
            context_notes.trim().to_string(),
        ]);
    }
    lines.extend([
        "".to_string(),
        "## Requirements Analysis".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis
            .requirements
            .iter()
            .map(|requirement| format!("**{}:** {}", requirement.id, requirement.text))
            .collect::<Vec<_>>(),
        "No requirements were extracted. Import or paste the full RFP and re-run analysis.",
    ));
    lines.extend([
        "".to_string(),
        "## Buyer Intent Analysis".to_string(),
        "".to_string(),
        "### Stated Intent".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.stated_intent.clone(),
        "Add stated buyer intent from the RFP overview, purpose, objectives, scope, and award language.",
    ));
    lines.extend([
        "".to_string(),
        "### Implied Intent".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.implied_intent.clone(),
        "Infer unstated priorities from criteria, mandatory evidence, timeline pressure, budget language, and risk signals.",
    ));
    lines.extend([
        "".to_string(),
        rfp_cli_compliance_matrix_markdown(analysis),
        "".to_string(),
        "## Requirement Response Drafts".to_string(),
        "".to_string(),
        "These draft answers are generated from the compliance matrix and must remain evidence-gated until the named owner attaches proof and a reviewer signs off.".to_string(),
        "".to_string(),
    ]);
    for row in &analysis.compliance_rows {
        lines.extend([
            format!("### {} - {}", row.id, row.response_section),
            "".to_string(),
            format!("Suggested response: {}", row.suggested_response),
            "".to_string(),
            format!("- [ ] Evidence owner: {}", row.owner),
            format!("- [ ] Evidence needed: {}", row.evidence_needed),
            format!("- [ ] Verification: {}", row.verification),
            "".to_string(),
        ]);
    }
    lines.extend(["## Requirement Verification".to_string(), "".to_string()]);
    lines.extend(
        analysis
            .verification_summary
            .checklist
            .iter()
            .map(|item| format!("- [ ] {item}")),
    );
    lines.extend([
        "".to_string(),
        "## Capability Match".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.capabilities.clone(),
        "Add capability evidence.",
    ));
    lines.extend([
        "".to_string(),
        "## Implementation Plan and Timeline".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.timelines.clone(),
        "Add the buyer deadline, milestones, and submission time zone.",
    ));
    lines.extend([
        "".to_string(),
        "## Pricing and Budget Response".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.budget_hints.clone(),
        "Add pricing basis, budget ceiling, required forms, and assumptions.",
    ));
    lines.extend([
        "".to_string(),
        "## Evaluation Criteria Response".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.evaluation_criteria.clone(),
        "Add the evaluation criteria and scoring weights.",
    ));
    lines.extend([
        "".to_string(),
        "## Mandatory Attachments".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.mandatory_attachments.clone(),
        "Add mandatory forms, certificates, declarations, and signatures.",
    ));
    lines.extend([
        "".to_string(),
        "## Risk and Assumptions".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.risks.clone(),
        "Review source RFP for risks, exceptions, and buyer constraints.",
    ));
    lines.extend([
        "".to_string(),
        "## Open Questions for Buyer or Bid Team".to_string(),
        "".to_string(),
    ]);
    lines.extend(markdown_bullets(
        analysis.questions.clone(),
        "No open questions detected.",
    ));
    lines.extend([
        "".to_string(),
        "## Submission QA Checklist".to_string(),
        "".to_string(),
        "- [ ] Every RFP requirement appears in the compliance matrix.".to_string(),
        "- [ ] Every matrix row has a response section and evidence owner.".to_string(),
        "- [ ] Mandatory forms, certificates, declarations, and signatures are attached.".to_string(),
        "- [ ] Pricing matches the required format and stated assumptions.".to_string(),
        "- [ ] Timeline, delivery milestones, and submission deadline are confirmed.".to_string(),
        "- [ ] Legal, finance, and delivery reviewers have approved the final response.".to_string(),
        "".to_string(),
        "<!-- ai-assisted: status=needs-review | source=NEditor ned RFP Response | promptSummary=Analyze RFP, build compliance matrix, draft responsive response -->".to_string(),
    ]);
    lines.join("\n")
}

fn rfp_cli_compliance_checklist_markdown(analysis: &RfpCliAnalysis) -> String {
    let mut lines = vec![
        "## Compliance Checklist".to_string(),
        "".to_string(),
        "### Critical Disqualification Traps".to_string(),
        "".to_string(),
    ];
    let critical_rows = analysis
        .compliance_rows
        .iter()
        .filter(|row| rfp_cli_disqualification_risk(&row.requirement))
        .collect::<Vec<_>>();
    if critical_rows.is_empty() {
        lines.push("- [ ] No explicit automatic-exclusion wording detected; reviewer must still inspect the source RFP.".to_string());
    } else {
        for row in critical_rows {
            lines.push(format!(
                "- [ ] **{}:** {} (source line {}) - {}",
                row.id, row.requirement, row.source_line, row.verification
            ));
        }
    }
    lines.extend([
        "".to_string(),
        "### Full Checklist".to_string(),
        "".to_string(),
        "| ID | Section | Risk | Requirement | Verification method | Owner | Reference |"
            .to_string(),
        "| --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ]);
    if analysis.compliance_rows.is_empty() {
        lines.push("| RFP-CHECK-001 | Intake | high | Import or paste the full RFP source. | Re-run RFP analysis and confirm all sections, annexes, and tables were captured. | Bid Owner | RFP source |".to_string());
    } else {
        for row in &analysis.compliance_rows {
            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} | Source line {} |",
                table_cell(&row.id.replace("RFP-REQ", "RFP-CHECK")),
                table_cell(&row.response_section),
                if rfp_cli_disqualification_risk(&row.requirement) {
                    "critical"
                } else if row.compliance_status.contains("Needs") {
                    "high"
                } else {
                    "standard"
                },
                table_cell(&row.requirement),
                table_cell(&row.verification),
                table_cell(&row.owner),
                row.source_line,
            ));
        }
    }
    for (index, attachment) in analysis.mandatory_attachments.iter().enumerate() {
        lines.push(format!(
            "| RFP-CHECK-A{:02} | Document checklist - attachments required | high | Include and verify mandatory attachment: {} | Confirm complete, signed where required, current, and included in the final package. | Bid Coordinator | Attachment scan |",
            index + 1,
            table_cell(attachment),
        ));
    }
    lines.join("\n")
}

fn rfp_cli_proposal_planning_prompt_markdown(
    analysis: &RfpCliAnalysis,
    context_notes: &str,
) -> String {
    let scoring = if analysis.evaluation_criteria.is_empty() {
        "No explicit scoring weights detected; infer equal review emphasis until the RFP is confirmed.".to_string()
    } else {
        analysis.evaluation_criteria.join("; ")
    };
    let pass_fail = analysis
        .compliance_rows
        .iter()
        .filter(|row| rfp_cli_disqualification_risk(&row.requirement))
        .map(|row| format!("{}: {}", row.id, row.requirement))
        .collect::<Vec<_>>();
    let technical = rfp_cli_lines_for_categories(
        analysis,
        &["Technical Solution", "Compliance"],
        "technical standards, integrations, hosting, APIs, data formats, licensing, and interoperability",
    );
    let team = rfp_cli_lines_for_categories(
        analysis,
        &["Team and Experience"],
        "team composition, minimum credentials, CV evidence, references, and language coverage",
    );
    let risk = if analysis.risks.is_empty() {
        "risk, QA, validation, monitoring, acceptance, and KPI controls".to_string()
    } else {
        analysis.risks.join("; ")
    };
    let mut lines = vec![
        "## Proposal Planning Prompt".to_string(),
        "".to_string(),
        "Use this evaluator-driven planning prompt before drafting response prose:".to_string(),
        "".to_string(),
        format!("- Extract the evaluator model, scoring weights, sub-criteria, and likely reviewer evidence checks; mirror these signals: {scoring}"),
        format!(
            "- Treat pass/fail gates as hard blockers: {}",
            if pass_fail.is_empty() {
                "No explicit automatic-exclusion gate detected; reviewer must inspect mandatory language, submission rules, and annexes.".to_string()
            } else {
                pass_fail.join("; ")
            }
        ),
        format!(
            "- Build the Terms of Reference map from activities, deliverables, milestones, approval periods, and annexes: {} timeline hint(s), {} mandatory attachment hint(s), {} extracted requirement(s).",
            analysis.timelines.len(),
            analysis.mandatory_attachments.len(),
            analysis.requirements.len()
        ),
        format!("- Convert team and experience requirements into a role matrix: {team}."),
        format!("- Turn technical mandates into section requirements: {technical}."),
        "- Turn sustainability, transition, maintenance, handover, and support requirements into an operating model; add explicit placeholders when the RFP is silent.".to_string(),
        format!("- Turn risk, QA, validation, monitoring, acceptance, and KPI language into controls and reviewer checks: {risk}."),
        "- Draft sections sequentially only after the checklist and outline are reviewed; leave evidence gaps as visible placeholders instead of unsupported claims.".to_string(),
    ];
    if !context_notes.trim().is_empty() {
        lines.push(format!(
            "- Apply bid-team context notes while preserving RFP traceability: {}",
            context_notes.trim()
        ));
    }
    lines.join("\n")
}

fn rfp_cli_proposal_outline_bullets(analysis: &RfpCliAnalysis) -> Vec<String> {
    let mut lines = vec![
        "- Executive Summary".to_string(),
        "- Assignment Understanding & Delivery Approach".to_string(),
        "- Proposed Methodology & Technical Approach".to_string(),
    ];
    let technical_rows = analysis
        .compliance_rows
        .iter()
        .filter(|row| row.response_section == "Technical Response")
        .take(6)
        .collect::<Vec<_>>();
    if technical_rows.is_empty() {
        lines.push("  - Primary ToR activity".to_string());
    } else {
        for row in technical_rows {
            lines.push(format!("  - {}: {}", row.id, row.requirement));
        }
    }
    lines.extend([
        "- Work Plan & Timeline".to_string(),
        "- Team Organization & Key Personnel".to_string(),
        "- Organizational Capacity & Past Performance".to_string(),
        "- Technical Standards, Data, and Integration Approach".to_string(),
        "- Risk Management & Mitigation".to_string(),
        "- Quality Assurance & Monitoring".to_string(),
        "- Sustainability & Transition Plan".to_string(),
        "- Compliance Summary Table".to_string(),
        "- Required Annexes".to_string(),
        "- Critical Disqualifiers Checklist".to_string(),
    ]);
    lines
}

fn rfp_cli_evaluator_section_drafts_markdown(
    analysis: &RfpCliAnalysis,
    company: &str,
    client: &str,
    context_notes: &str,
) -> String {
    let technical = rfp_cli_lines_for_categories(
        analysis,
        &["Technical Solution", "Compliance"],
        "Confirm technical standards, data formats, integrations, hosting, API, licensing, and interoperability requirements.",
    );
    let team = rfp_cli_lines_for_categories(
        analysis,
        &["Team and Experience"],
        "Confirm team roles, minimum experience, credentials, CV evidence, references, and language coverage.",
    );
    let timeline = if analysis.timelines.is_empty() {
        "Confirm submission deadline, work plan, milestones, dependencies, and approval windows."
            .to_string()
    } else {
        analysis.timelines.join("; ")
    };
    let risk = if analysis.risks.is_empty() {
        "Confirm delivery, compliance, schedule, commercial, technical, QA, validation, KPI, and acceptance risks.".to_string()
    } else {
        analysis.risks.join("; ")
    };
    let notes = context_notes.trim();
    let emphasis = if notes.is_empty() {
        String::new()
    } else {
        format!("\n\nBid-team emphasis: {notes}")
    };
    [
        "## Evaluator-Aligned Section Drafts".to_string(),
        "".to_string(),
        "These draft sections are generated from the compliance checklist, scoring signals, pass/fail gates, Terms of Reference map, team requirements, technical mandates, sustainability obligations, and risk/QA/KPI signals. They remain evidence-gated until owners attach proof.".to_string(),
        "".to_string(),
        "### Executive Summary Draft".to_string(),
        "".to_string(),
        format!("{company} will respond to {client} with a compliance-first, evaluator-readable proposal. The response will show how each mandatory requirement is met, how scored criteria are answered with evidence, and where reviewer sign-off is still required before submission.{emphasis}"),
        "".to_string(),
        "### Assignment Understanding and ToR Response Draft".to_string(),
        "".to_string(),
        format!("Draft response: Map extracted requirements, buyer intent, and timeline signals into work packages. ToR/timeline basis: {timeline}"),
        "".to_string(),
        "### Technical Methodology Draft".to_string(),
        "".to_string(),
        format!("Draft response: Convert technical mandates into implementation choices, integration patterns, data or documentation standards, test evidence, and acceptance criteria. Technical basis: {technical}"),
        "".to_string(),
        "### Team and Experience Draft".to_string(),
        "".to_string(),
        format!("Draft response: Link each named role to ToR activities, credentials, comparable work, language or local-knowledge requirements, and CV/reference evidence. Team basis: {team}"),
        "".to_string(),
        "### Sustainability and Transition Draft".to_string(),
        "".to_string(),
        "Draft response: Explain maintenance, handover, knowledge transfer, operational ownership, support model, and post-project continuity without promising unsupported capacity.".to_string(),
        "".to_string(),
        "### Risk, QA, Validation, and KPI Draft".to_string(),
        "".to_string(),
        format!("Draft response: List top risks, mitigations, owners, QA checks, validation evidence, monitoring cadence, KPIs, and acceptance criteria. Risk basis: {risk}"),
        "".to_string(),
        "### Compliance Summary Draft".to_string(),
        "".to_string(),
        "Draft response: Point reviewers back to the front-of-document checklist and compliance matrix. Show requirement ID, response section, evidence owner, verification method, and unresolved proof gaps for every extracted requirement.".to_string(),
    ]
    .join("\n")
}

fn rfp_cli_lines_for_categories(
    analysis: &RfpCliAnalysis,
    categories: &[&str],
    fallback: &str,
) -> String {
    let values = analysis
        .compliance_rows
        .iter()
        .filter(|row| categories.iter().any(|category| row.category == *category))
        .map(|row| row.requirement.clone())
        .take(6)
        .collect::<Vec<_>>();
    if values.is_empty() {
        fallback.to_string()
    } else {
        values.join("; ")
    }
}

fn rfp_cli_disqualification_risk(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    contains_any(
        &lower,
        &[
            "disqualif",
            "will be rejected",
            "automatic",
            "failure to",
            "non-compliant",
            "noncompliant",
            "mandatory",
            "must submit",
            "shall submit",
        ],
    )
}

fn markdown_bullets(values: Vec<String>, fallback: &str) -> Vec<String> {
    if values.is_empty() {
        vec![format!("- {fallback}")]
    } else {
        values
            .into_iter()
            .map(|value| format!("- {value}"))
            .collect()
    }
}

fn profile_fallback(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}

fn table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn new_document_markdown(template: &str, title: &str) -> Result<String, String> {
    let template = template.trim().to_ascii_lowercase();
    if !NEW_DOCUMENT_TEMPLATES.contains(&template.as_str()) {
        return Err(format!(
            "Unknown template '{}'. Available templates: {}",
            template,
            NEW_DOCUMENT_TEMPLATES.join(", ")
        ));
    }
    let escaped_title = yaml_scalar(title);
    let body = match template.as_str() {
        "blank" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\n---\n\n# {title}\n\nStart writing here.\n"
        ),
        "proposal" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: proposal\ntoc: true\n---\n\n# {title}\n\n## Executive Summary\n\nState the client outcome, recommendation, and commercial value.\n\n## Client Context\n\nDescribe the buyer, problem, constraints, and decision process.\n\n## Proposed Approach\n\n| Phase | Work | Output | Owner |\n| --- | --- | --- | --- |\n| Discover | Confirm objectives and evidence | Findings memo | {{{{owner}}}} |\n| Deliver | Execute the agreed work plan | Review-ready deliverable | {{{{owner}}}} |\n\n## Timeline\n\n- Kickoff: {{{{kickoff_date}}}}\n- Draft review: {{{{draft_review_date}}}}\n- Final delivery: {{{{final_delivery_date}}}}\n\n## Commercials\n\nSummarize fees, assumptions, payment terms, and exclusions.\n\n## Review Handoff\n\n- Confirm scope, pricing, legal terms, and cited evidence.\n- Resolve all placeholders before sending.\n"
        ),
        "rfp" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: rfp\ntoc: true\n---\n\n# {title}\n\n## Opportunity Summary\n\nDescribe the procurement goal, business outcome, buyer context, and decision deadline.\n\n## Scope Of Work\n\n| Workstream | Required Outcome | Deliverable | Acceptance Criteria |\n| --- | --- | --- | --- |\n| {{{{workstream}}}} | {{{{outcome}}}} | {{{{deliverable}}}} | {{{{criteria}}}} |\n\n## Vendor Instructions\n\n- Submission deadline: {{{{submission_deadline}}}}\n- Question deadline: {{{{question_deadline}}}}\n- Required format: {{{{submission_format}}}}\n- Contact: {{{{procurement_contact}}}}\n\n## Evaluation Criteria\n\n| Criterion | Weight | Evidence Expected |\n| --- | ---: | --- |\n| Technical fit | 40% | Relevant approach and proof |\n| Delivery confidence | 30% | Timeline, staffing, risk plan |\n| Commercial value | 30% | Pricing, terms, assumptions |\n\n## Required Response Matrix\n\n| ID | Requirement | Mandatory | Vendor Response | Evidence |\n| --- | --- | --- | --- | --- |\n| R1 | {{{{requirement}}}} | Yes |  |  |\n"
        ),
        "rfp-response" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: rfp-response\ntoc: true\n---\n\n# {title}\n\n## Response Strategy\n\nSummarize the buyer's stated and implied intent, win themes, and response posture.\n\n## Compliance Matrix\n\n| ID | Requirement | Response | Evidence | Owner | Status |\n| --- | --- | --- | --- | --- | --- |\n| R1 | {{{{requirement}}}} | {{{{response}}}} | {{{{evidence}}}} | {{{{owner}}}} | Draft |\n\n## Technical Response\n\nAddress every mandatory requirement with clear evidence and assumptions.\n\n## Delivery Plan\n\nOutline milestones, dependencies, risks, and governance.\n\n## Pricing And Assumptions\n\nState pricing, exclusions, validity period, and approval requirements.\n\n## Final Verification\n\n- Every stated requirement has a mapped response.\n- Implied intent and evaluation criteria have been addressed.\n- Attachments, forms, and signatures are tracked.\n"
        ),
        "rfq" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: rfq\ntoc: true\n---\n\n# {title}\n\n## Buying Need\n\nSummarize the goods, services, quantities, service levels, and decision date.\n\n## Quote Instructions\n\n- Quote deadline: {{{{quote_deadline}}}}\n- Validity period: {{{{validity_period}}}}\n- Delivery location: {{{{delivery_location}}}}\n- Required currency: {{{{currency}}}}\n\n## Line Items\n\n| Item | Description | Quantity | Unit | Required Date | Vendor Price |\n| --- | --- | ---: | --- | --- | ---: |\n| 1 | {{{{item_description}}}} | {{{{quantity}}}} | {{{{unit}}}} | {{{{required_date}}}} |  |\n\n## Commercial Terms\n\nState taxes, shipping, warranties, payment terms, substitutions, and exclusions.\n\n## Award Criteria\n\nExplain price, availability, compliance, service, and risk considerations.\n"
        ),
        "tender" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: tender\ntoc: true\n---\n\n# {title}\n\n## Tender Notice\n\nState the contracting authority, opportunity, procurement method, eligibility, and closing date.\n\n## Instructions To Tenderers\n\n- Tender reference: {{{{tender_reference}}}}\n- Submission portal/location: {{{{submission_location}}}}\n- Closing date and time: {{{{closing_datetime}}}}\n- Clarification process: {{{{clarification_process}}}}\n\n## Scope And Specifications\n\nDescribe mandatory specifications, service levels, deliverables, milestones, and acceptance tests.\n\n## Eligibility And Mandatory Documents\n\n| Document | Required | Notes |\n| --- | --- | --- |\n| Company registration | Yes | {{{{notes}}}} |\n| Tax compliance | Yes | {{{{notes}}}} |\n\n## Evaluation Method\n\n| Stage | Criteria | Pass Mark / Weight |\n| --- | --- | --- |\n| Administrative compliance | Mandatory documents | Pass/fail |\n| Technical evaluation | Methodology and capability | {{{{technical_weight}}}} |\n| Financial evaluation | Price and value | {{{{financial_weight}}}} |\n"
        ),
        "sow" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: project-plan\ntoc: true\n---\n\n# {title}\n\n## Purpose\n\nState the business outcome, contracting context, and decision this statement of work supports.\n\n## Scope\n\n| Workstream | Included Work | Deliverable | Acceptance Criteria |\n| --- | --- | --- | --- |\n| {{{{workstream}}}} | {{{{included_work}}}} | {{{{deliverable}}}} | {{{{acceptance_criteria}}}} |\n\n## Out Of Scope\n\nList excluded services, client-owned responsibilities, and assumptions that prevent scope drift.\n\n## Project Plan\n\n- Kickoff: {{{{kickoff_date}}}}\n- Draft milestone: {{{{draft_milestone}}}}\n- Final acceptance: {{{{final_acceptance_date}}}}\n\n## Roles And Responsibilities\n\n| Role | Organization | Responsibility | Decision Authority |\n| --- | --- | --- | --- |\n| {{{{role}}}} | {{{{organization}}}} | {{{{responsibility}}}} | {{{{authority}}}} |\n\n## Acceptance And Change Control\n\nDefine review windows, acceptance evidence, change request routing, pricing impacts, and approval owners.\n"
        ),
        "capability-statement" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: marketing-brief\ntoc: true\n---\n\n# {title}\n\n## Company Overview\n\nSummarize who you are, who you serve, and the business outcomes you reliably deliver.\n\n## Core Capabilities\n\n| Capability | What We Do | Proof Point | Relevant Buyer Need |\n| --- | --- | --- | --- |\n| {{{{capability}}}} | {{{{description}}}} | {{{{proof_point}}}} | {{{{buyer_need}}}} |\n\n## Differentiators\n\n- {{{{differentiator_1}}}}\n- {{{{differentiator_2}}}}\n- {{{{differentiator_3}}}}\n\n## Relevant Experience\n\n| Client Or Sector | Assignment | Outcome | Reference Status |\n| --- | --- | --- | --- |\n| {{{{client_or_sector}}}} | {{{{assignment}}}} | {{{{outcome}}}} | {{{{reference_status}}}} |\n\n## Certifications And Compliance\n\nList registrations, certifications, security posture, quality controls, insurance, and partner credentials.\n\n## Contact\n\n**Prepared by:** {{{{owner}}}}  \n**Email:** {{{{email}}}}  \n**Website:** {{{{website}}}}\n"
        ),
        "case-study" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: customer-case-study\ntoc: true\n---\n\n# {title}\n\n## Customer Snapshot\n\n| Field | Detail |\n| --- | --- |\n| Customer | {{{{customer_name}}}} |\n| Industry | {{{{industry}}}} |\n| Engagement | {{{{engagement_type}}}} |\n| Review status | {{{{approval_status}}}} |\n\n## Challenge\n\nDescribe the customer situation, stakes, constraints, and why the issue mattered.\n\n## Solution\n\nExplain the approach, implementation model, team roles, and why this solution fit the customer context.\n\n## Implementation\n\n| Phase | Action | Evidence | Owner |\n| --- | --- | --- | --- |\n| {{{{phase}}}} | {{{{action}}}} | {{{{evidence}}}} | {{{{owner}}}} |\n\n## Results\n\n| Metric | Before | After | Source |\n| --- | ---: | ---: | --- |\n| {{{{metric}}}} | {{{{before}}}} | {{{{after}}}} | {{{{source}}}} |\n\n## Quote Prompts\n\n- Customer quote to approve: {{{{customer_quote}}}}\n- Internal approver: {{{{approver}}}}\n\n## Review Approvals\n\nConfirm customer permission, claim evidence, logo usage, confidentiality, and distribution channels before publishing.\n"
        ),
        "report" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: business-report\ntoc: true\n---\n\n# {title}\n\n## Executive Summary\n\nSummarize the finding, implication, and recommended decision.\n\n## Situation\n\nDescribe the context, evidence base, constraints, and stakeholders.\n\n## Analysis\n\n```calc\nrevenue = 0\ncost = 0\nprofit = revenue - cost\nmargin = profit / revenue\n```\n\nExpected margin: {{{{=margin | percent}}}}\n\n## Recommendations\n\n1. Recommendation one.\n2. Recommendation two.\n3. Recommendation three.\n\n## Risks And Next Steps\n\n| Risk | Impact | Mitigation | Owner |\n| --- | --- | --- | --- |\n| {{{{risk}}}} | {{{{impact}}}} | {{{{mitigation}}}} | {{{{owner}}}} |\n"
        ),
        "tutorial" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: tutorial\ntoc: true\n---\n\n# {title}\n\n## Outcome\n\nState what the reader will be able to do by the end of the tutorial.\n\n## Prerequisites\n\n- Audience: {{{{audience}}}}\n- Tools/accounts needed: {{{{tools}}}}\n- Starting files or data: {{{{starting_point}}}}\n\n## Steps\n\n### Step 1: {{{{first_step}}}}\n\nExplain the action, expected result, and common mistakes.\n\n### Step 2: {{{{second_step}}}}\n\nContinue with concise instructions and verification screenshots or outputs.\n\n## Check Your Work\n\n| Check | Expected Result | Troubleshooting |\n| --- | --- | --- |\n| {{{{check}}}} | {{{{expected}}}} | {{{{fix}}}} |\n\n## Next Steps\n\nSuggest practice tasks, references, and escalation paths.\n"
        ),
        "lesson-plan" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: lesson-plan\ntoc: true\n---\n\n# {title}\n\n## Learning Objectives\n\n- Learners can explain {{{{concept}}}}.\n- Learners can apply {{{{skill}}}} in a realistic scenario.\n\n## Audience And Prerequisites\n\nDescribe learner profile, prior knowledge, materials, and accessibility needs.\n\n## Lesson Flow\n\n| Time | Activity | Instructor Action | Learner Evidence |\n| ---: | --- | --- | --- |\n| 10 min | Opening | Frame the problem | Questions captured |\n| 30 min | Practice | Guide the worked example | Exercise completed |\n| 10 min | Review | Check understanding | Exit ticket |\n\n## Assessment\n\nDefine rubric, success criteria, remediation, and extension activities.\n"
        ),
        "lesson-content" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: lesson-content\ntoc: true\n---\n\n# {title}\n\n## Learner Context\n\nDescribe grade/level, prior knowledge, accommodations, and materials.\n\n## Content Sequence\n\n### Concept 1: {{{{concept}}}}\n\nExplain the concept with a concrete example, visual cue, and misconception check.\n\n### Guided Practice\n\nProvide worked examples, prompts, and expected learner responses.\n\n### Independent Practice\n\n| Activity | Instructions | Evidence Of Learning |\n| --- | --- | --- |\n| {{{{activity}}}} | {{{{instructions}}}} | {{{{evidence}}}} |\n\n## Assessment Items\n\nAdd questions, answer key, rubric, remediation, and extension activities.\n"
        ),
        "textbook" | "technical-textbook" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: textbook\ntoc: true\n---\n\n# {title}\n\n## Book Positioning\n\n- Audience: {{{{audience}}}}\n- Level: {{{{level}}}}\n- Prerequisites: {{{{prerequisites}}}}\n\n## Book Outline\n\n### Chapter 1: Foundations\n\n- Learning goals\n- Key concepts\n- Worked examples\n- Exercises\n\n### Chapter 2: Applied Practice\n\n- Case study\n- Common errors\n- Review questions\n\n## Drafting Plan\n\nUse Docs Live to flesh out chapters sequentially only after the outline is reviewed.\n"
        ),
        "novel" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: novel\ntoc: true\n---\n\n# {title}\n\n## Premise\n\nWrite the central dramatic question, protagonist want, stakes, and setting.\n\n## Cast\n\n| Character | Desire | Conflict | Arc |\n| --- | --- | --- | --- |\n| {{{{name}}}} | {{{{desire}}}} | {{{{conflict}}}} | {{{{arc}}}} |\n\n## Plot Outline\n\n### Act I\n\nSet up the world, inciting incident, and first irreversible choice.\n\n### Act II\n\nEscalate pressure, reversals, midpoint, and cost.\n\n### Act III\n\nResolve the conflict, consequence, and final image.\n\n## Narrative Review\n\nCheck voice, pacing, continuity, scene purpose, and emotional progression before drafting chapters.\n"
        ),
        "podcast-script" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: podcast-script\ntoc: true\n---\n\n# {title}\n\n## Episode Brief\n\n- Show: {{{{show_name}}}}\n- Episode objective: {{{{objective}}}}\n- Guest(s): {{{{guests}}}}\n- Target length: {{{{duration}}}}\n\n## Cold Open\n\nWrite a concise hook that frames the listener problem and stakes.\n\n## Segment Rundown\n\n| Segment | Time | Purpose | Notes |\n| --- | ---: | --- | --- |\n| Intro | 00:00 | Set context | {{{{intro_notes}}}} |\n| Main discussion | 05:00 | Develop argument/story | {{{{main_notes}}}} |\n| Close | {{{{close_time}}}} | Summarize and call to action | {{{{cta}}}} |\n\n## Host Script\n\n**Host:** {{{{host_line}}}}\n\n## Production Notes\n\nList music, ads, legal review, links, transcript needs, and publishing checklist.\n"
        ),
        "movie-script" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: movie-script\ntoc: true\n---\n\n# {title}\n\n## Logline\n\nState protagonist, goal, obstacle, stakes, and hook in one sentence.\n\n## Characters\n\n| Character | Role | Want | Need | Conflict |\n| --- | --- | --- | --- | --- |\n| {{{{character}}}} | {{{{role}}}} | {{{{want}}}} | {{{{need}}}} | {{{{conflict}}}} |\n\n## Treatment\n\n### Act I\n\nSet up world, inciting incident, and first turning point.\n\n### Act II\n\nEscalate conflict, midpoint reversal, low point, and renewed choice.\n\n### Act III\n\nResolve climax, consequence, and final image.\n\n## Scene Starter\n\n**INT./EXT. LOCATION - DAY/NIGHT**\n\nAction paragraph.\n\n**CHARACTER**\n\nDialogue.\n\n## Script Review\n\nCheck motivation, stakes, scene purpose, continuity, pacing, tone, and production constraints.\n"
        ),
        "business-case" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: business-case\ntoc: true\n---\n\n# {title}\n\n## Decision Required\n\nState the decision, sponsor, required date, and recommendation.\n\n## Strategic Rationale\n\nExplain the business problem, opportunity, alignment, and consequences of inaction.\n\n## Options Considered\n\n| Option | Benefits | Costs | Risks | Recommendation |\n| --- | --- | ---: | --- | --- |\n| Do nothing | {{{{benefit}}}} | {{{{cost}}}} | {{{{risk}}}} | No |\n| Recommended option | {{{{benefit}}}} | {{{{cost}}}} | {{{{risk}}}} | Yes |\n\n## Financial Case\n\n```calc\nbenefit = 0\ncost = 0\nnet_value = benefit - cost\nroi = net_value / cost\n```\n\nEstimated ROI: {{{{=roi | percent}}}}\n\n## Implementation Plan\n\nList milestones, dependencies, owners, governance, and success measures.\n"
        ),
        "executive-brief" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: executive-brief\ntoc: true\n---\n\n# {title}\n\n## Bottom Line\n\nGive the decision, implication, and recommendation in three to five sentences.\n\n## What Changed\n\nSummarize the new information, trend, event, or risk that requires attention.\n\n## Evidence\n\n| Signal | Source | Confidence | Implication |\n| --- | --- | --- | --- |\n| {{{{signal}}}} | {{{{source}}}} | {{{{confidence}}}} | {{{{implication}}}} |\n\n## Options\n\n1. Recommended action.\n2. Alternative action.\n3. No action.\n\n## Ask\n\nState the approval, resources, timing, or executive decision required.\n"
        ),
        "board-decision-memo" => structured_new_document_markdown(
            &escaped_title,
            title,
            "board-memo",
            &[
                "Decision Requested",
                "Executive Summary",
                "Strategic Context",
                "Options Considered",
                "Financial Case",
                "Risk Assessment",
                "Implementation Plan",
                "Recommendation",
                "Appendices",
            ],
            "Keep the decision ask, evidence, tradeoffs, financial exposure, risks, and approval path clear for board review.",
        ),
        "policy-brief" => structured_new_document_markdown(
            &escaped_title,
            title,
            "policy",
            &[
                "Executive Summary",
                "Problem Definition",
                "Policy Context",
                "Evidence Base",
                "Options",
                "Impact Assessment",
                "Risks and Tradeoffs",
                "Recommendation",
                "Implementation Considerations",
            ],
            "Translate the evidence into practical policy options, impacts, risks, and an actionable recommendation.",
        ),
        "research-report" => structured_new_document_markdown(
            &escaped_title,
            title,
            "research-brief",
            &[
                "Abstract",
                "Introduction",
                "Research Questions",
                "Methodology",
                "Literature and Source Review",
                "Findings",
                "Analysis",
                "Limitations",
                "Recommendations",
                "Bibliography",
            ],
            "Keep research questions, methods, sources, findings, limitations, and recommendations traceable.",
        ),
        "grant-application" => structured_new_document_markdown(
            &escaped_title,
            title,
            "proposal",
            &[
                "Cover Summary",
                "Applicant Profile",
                "Problem and Need",
                "Target Beneficiaries",
                "Theory of Change",
                "Program Design",
                "Work Plan",
                "Monitoring and Evaluation",
                "Budget Narrative",
                "Sustainability",
                "Attachments",
            ],
            "Map the funder's priorities to need, beneficiaries, work plan, measurement, budget, sustainability, and attachments.",
        ),
        "standard-operating-procedure" => structured_new_document_markdown(
            &escaped_title,
            title,
            "operating-procedure",
            &[
                "Purpose",
                "Scope",
                "Owner and Approvers",
                "Definitions",
                "Inputs and Systems",
                "Procedure",
                "Controls and Checks",
                "Exceptions",
                "Records and Evidence",
                "Revision History",
            ],
            "Make the procedure executable, auditable, trainable, and easy to update.",
        ),
        "product-requirements-document" => structured_new_document_markdown(
            &escaped_title,
            title,
            "project-plan",
            &[
                "Problem Statement",
                "Goals and Non-Goals",
                "Personas and Use Cases",
                "Requirements",
                "User Experience Notes",
                "Technical Dependencies",
                "Acceptance Criteria",
                "Metrics",
                "Risks and Open Questions",
                "Release Plan",
            ],
            "Tie user value, requirements, UX notes, acceptance criteria, dependencies, metrics, and release risk together.",
        ),
        "project-charter" => structured_new_document_markdown(
            &escaped_title,
            title,
            "project-plan",
            &[
                "Executive Summary",
                "Objectives",
                "Scope",
                "Out of Scope",
                "Stakeholders",
                "Governance",
                "Milestones",
                "Budget and Resources",
                "Risks and Assumptions",
                "Success Criteria",
                "Approvals",
            ],
            "Give sponsors and delivery teams a shared scope, governance model, milestone plan, risk posture, and success criteria.",
        ),
        "quarterly-business-review" => structured_new_document_markdown(
            &escaped_title,
            title,
            "meeting-brief",
            &[
                "Executive Summary",
                "Period Highlights",
                "Goals and Scorecard",
                "Usage or Financial Trends",
                "Delivered Value",
                "Risks and Blockers",
                "Opportunities",
                "Next Quarter Plan",
                "Decisions and Actions",
            ],
            "Connect outcomes, scorecards, delivered value, risks, opportunities, decisions, and next-quarter ownership.",
        ),
        "due-diligence-memo" => structured_new_document_markdown(
            &escaped_title,
            title,
            "business-case",
            &[
                "Executive Summary",
                "Transaction or Decision Context",
                "Investment Thesis",
                "Market and Customer Evidence",
                "Product or Service Assessment",
                "Financial Review",
                "Operational Review",
                "Legal and Compliance Review",
                "Risk Register",
                "Recommendation",
                "Appendices",
            ],
            "Separate the thesis, evidence, financials, operations, legal/compliance risks, open questions, and recommendation.",
        ),
        "contract-review-brief" => structured_new_document_markdown(
            &escaped_title,
            title,
            "contract-brief",
            &[
                "Contract Summary",
                "Parties and Scope",
                "Commercial Terms",
                "Delivery Obligations",
                "Service Levels",
                "Data and IP Terms",
                "Liability and Indemnities",
                "Termination and Renewal",
                "Negotiation Positions",
                "Approval Checklist",
            ],
            "Help business, legal, and delivery reviewers see obligations, risks, negotiation positions, and approvals quickly.",
        ),
        "implementation-playbook" => structured_new_document_markdown(
            &escaped_title,
            title,
            "project-plan",
            &[
                "Purpose",
                "Operating Model",
                "Scope",
                "Roles and Responsibilities",
                "Implementation Phases",
                "Change Management",
                "Training Plan",
                "Risks and Controls",
                "Success Metrics",
                "Runbook",
            ],
            "Give teams a practical operating model, phased rollout, roles, training, controls, success metrics, and runbook.",
        ),
        "incident-postmortem" => structured_new_document_markdown(
            &escaped_title,
            title,
            "operating-procedure",
            &[
                "Summary",
                "Customer or Business Impact",
                "Timeline",
                "Detection and Response",
                "Root Cause Analysis",
                "What Went Well",
                "What Did Not Go Well",
                "Corrective Actions",
                "Preventive Controls",
                "Owner Sign-off",
            ],
            "Preserve facts, customer impact, timeline, detection and response, causes, corrective actions, prevention, and owner sign-off.",
        ),
        "meeting-decision-pack" => structured_new_document_markdown(
            &escaped_title,
            title,
            "meeting-brief",
            &[
                "Meeting Purpose",
                "Required Decisions",
                "Agenda",
                "Pre-Read Summary",
                "Options and Tradeoffs",
                "Evidence Pack",
                "Risks",
                "Decision Log",
                "Action Register",
                "Follow-up Communications",
            ],
            "Make required decisions, options, evidence, risks, the decision log, and action register easy to review.",
        ),
        "market-research-report" => structured_new_document_markdown(
            &escaped_title,
            title,
            "research-brief",
            &[
                "Executive Summary",
                "Research Objectives",
                "Methodology",
                "Market Definition",
                "Customer Segments",
                "Competitor Landscape",
                "Trends and Drivers",
                "Findings",
                "Implications",
                "Recommendations",
                "Bibliography",
            ],
            "Keep research objectives, methodology, market definition, customer segments, competitors, trends, findings, implications, and recommendations connected.",
        ),
        _ => unreachable!(),
    };
    Ok(body)
}

fn structured_new_document_markdown(
    escaped_title: &str,
    title: &str,
    document_type: &str,
    headings: &[&str],
    drafting_brief: &str,
) -> String {
    let mut lines = vec![
        "---".to_string(),
        format!("title: {escaped_title}"),
        "status: draft".to_string(),
        format!("documentType: {}", yaml_scalar(document_type)),
        "toc: true".to_string(),
        "---".to_string(),
        String::new(),
        format!("# {title}"),
        String::new(),
        "[TOC]".to_string(),
        String::new(),
    ];
    for heading in headings {
        lines.push(format!("## {heading}"));
        lines.push(String::new());
        lines.push(
            "Draft this section with verified facts, named owners, source evidence, and unresolved placeholders clearly marked for review."
                .to_string(),
        );
        lines.push(String::new());
    }
    lines.extend([
        "## AI Drafting Brief".to_string(),
        String::new(),
        drafting_brief.to_string(),
        String::new(),
        "## Review Handoff".to_string(),
        String::new(),
        "- Confirm all placeholders, facts, figures, citations, risks, and approvals.".to_string(),
        "- Run quality assurance and export readiness before external distribution.".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

fn workspace_init_entries(root: &Path) -> Vec<(PathBuf, &'static str)> {
    let base = root.join(".neditor");
    vec![
        (
            base.join("README.md"),
            "# NEditor Workspace\n\nThis folder stores reusable local project material for NEditor.\n\n- `business-profile.json` stores reusable sender, company, client, website, and brand voice values for templates, Docs Live, and handoff packages.\n- `variables.yaml` supplies project variables that documents can reference with `{{variable}}` placeholders.\n- `outlines.json` stores reusable custom document outlines for planners, Docs Live, and scripted document creation.\n- `snippets/` stores reusable document parts for proposals, RFPs, reports, tutorials, and review handoffs.\n- `agent-handoffs/` stores generated local-agent packets for Claude Code, Codex, OpenCode, or private workflows.\n\nDo not store API keys, passwords, or client secrets in this folder.\n",
        ),
        (
            base.join("variables.yaml"),
            "# Project variables available to NEditor documents.\n# Replace these examples with values your documents reuse often.\nprofile:\n  owner: \"Your Name\"\n  email: \"you@example.com\"\ncompany:\n  name: \"Your Company\"\n  website: \"https://example.com\"\nclient:\n  name: \"Client Name\"\nproject:\n  name: \"Project Name\"\n  review_date: \"YYYY-MM-DD\"\n",
        ),
        (
            base.join("business-profile.json"),
            "{\n  \"fullName\": \"Your Name\",\n  \"email\": \"you@example.com\",\n  \"phone\": \"\",\n  \"roleTitle\": \"Your Role\",\n  \"companyName\": \"Your Company\",\n  \"companyAddress\": \"\",\n  \"website\": \"https://example.com\",\n  \"industry\": \"\",\n  \"defaultClientName\": \"Client Name\",\n  \"brandVoice\": \"clear and practical\"\n}\n",
        ),
        (
            base.join("outlines.json"),
            "{\n  \"schema\": \"neditor.workspace-outlines.v1\",\n  \"outlines\": [\n    {\n      \"id\": \"quarterly-business-review\",\n      \"label\": \"Quarterly Business Review\",\n      \"category\": \"Business\",\n      \"summary\": \"Reusable executive review outline for quarterly performance, decisions, and next-step accountability.\",\n      \"bestFor\": [\n        \"board updates\",\n        \"client QBRs\",\n        \"executive operating reviews\"\n      ],\n      \"outline\": [\n        \"Executive Summary\",\n        \"Quarterly Performance Snapshot\",\n        \"Wins and Evidence\",\n        \"Issues and Risks\",\n        \"Financial Review\",\n        \"Customer and Market Signals\",\n        \"Decisions Requested\",\n        \"Next Quarter Priorities\",\n        \"Action Register\"\n      ],\n      \"tags\": [\n        \"business\",\n        \"review\",\n        \"executive\"\n      ]\n    }\n  ]\n}\n",
        ),
        (
            base.join("snippets").join("business.md"),
            "# Standard Business Snippets\n\n## Contact Block\n\n**Prepared by:** {{profile.owner}}  \n**Email:** {{profile.email}}  \n**Company:** {{company.name}}  \n**Website:** {{company.website}}\n\n## Review Handoff\n\n- Confirm scope, assumptions, pricing, dates, and cited evidence.\n- Resolve all placeholders and citation TODOs before external distribution.\n- Run export readiness for the required delivery formats.\n\n## Compliance Matrix Starter\n\n| ID | Requirement | Response | Evidence | Owner | Status |\n| --- | --- | --- | --- | --- | --- |\n| R1 | {{requirement}} | {{response}} | {{evidence}} | {{profile.owner}} | Draft |\n",
        ),
        (
            base.join("agent-handoffs").join(".gitkeep"),
            "",
        ),
    ]
}

fn init_text_report(
    root: &Path,
    dry_run: bool,
    force: bool,
    created: &[String],
    updated: &[String],
    kept: &[String],
) -> String {
    let action = if dry_run {
        "Would initialize"
    } else {
        "Initialized"
    };
    let mut lines = vec![format!("{action} NEditor workspace at {}", root.display())];
    if force {
        lines.push("Force mode is enabled; existing scaffold files are overwritten.".to_string());
    }
    append_init_paths(&mut lines, "Created", created);
    append_init_paths(&mut lines, "Updated", updated);
    append_init_paths(&mut lines, "Kept existing", kept);
    lines.push("Next steps:".to_string());
    lines.push("  - Run `ned profile --workspace . --set fullName=... --set companyName=...` to set reusable business identity values.".to_string());
    lines.push("  - Edit .neditor/variables.yaml with project values that are not part of the reusable business profile.".to_string());
    lines.push("  - Run `ned outlines --workspace . --save custom-outline --section \"Executive Summary\" --section \"Recommendations\"` to add reusable custom outlines.".to_string());
    lines.push("  - Add reusable proposal, RFP, tutorial, and review handoff parts under .neditor/snippets/.".to_string());
    lines.push(
        "  - Use the Agent Workspace when you want governed local-agent handoff files.".to_string(),
    );
    lines.join("\n")
}

fn append_init_paths(lines: &mut Vec<String>, label: &str, paths: &[String]) {
    if paths.is_empty() {
        return;
    }
    lines.push(format!("{label}:"));
    lines.extend(paths.iter().map(|path| format!("  - {path}")));
}

fn path_to_display(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn yaml_scalar(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '-' | '_' | '/' | '.'))
    {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

fn default_output_path(input_path: &str, target: &str) -> PathBuf {
    PathBuf::from(input_path).with_extension(target_extension(target))
}

fn target_output_path(
    input_path: &str,
    target: &str,
    explicit_output: Option<&String>,
    output_dir: Option<&PathBuf>,
    include_target_suffix: bool,
) -> PathBuf {
    if let Some(path) = explicit_output {
        return PathBuf::from(path);
    }
    if let Some(directory) = output_dir {
        return directory.join(default_output_file_name(input_path, target, true));
    }
    if include_target_suffix {
        let input = Path::new(input_path);
        return input.with_file_name(default_output_file_name(input_path, target, true));
    }
    default_output_path(input_path, target)
}

fn default_output_file_name(input_path: &str, target: &str, include_target_suffix: bool) -> String {
    let path = Path::new(input_path);
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("document");
    let extension = target_extension(target);
    if include_target_suffix {
        format!("{stem}-{target}.{extension}")
    } else {
        format!("{stem}.{extension}")
    }
}

fn target_extension(target: &str) -> &'static str {
    match target {
        "html" => "html",
        "pdf" => "pdf",
        "docx" => "docx",
        "pptx" => "pptx",
        "latex" => "tex",
        "markdown-bundle" | "blog" | "substack" | "google-docs" => "zip",
        "epub" => "epub",
        _ => "out",
    }
}

fn stdout_temp_output_path(target: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    env::temp_dir().join(format!(
        "neditor-ned-stdout-{}-{nanos}.{}",
        std::process::id(),
        target_extension(target)
    ))
}

fn bash_completion_script() -> String {
    let commands = CLI_COMMANDS.join(" ");
    let templates = NEW_DOCUMENT_TEMPLATES.join(" ");
    let targets = format!("{} all", SUPPORTED_EXPORT_TARGETS.join(" "));
    let publish_targets = "blog substack html";
    let publish_destinations = "generic-webhook wordpress-rest ghost-admin substack-manual";
    let publish_formats = "html markdown text";
    let shells = COMPLETION_SHELLS.join(" ");
    let handler_platforms = "macos windows linux manual";
    format!(
        r#"# bash completion for ned
_ned() {{
  local cur prev command
  COMPREPLY=()
  cur="${{COMP_WORDS[COMP_CWORD]}}"
  prev="${{COMP_WORDS[COMP_CWORD-1]}}"
  command="${{COMP_WORDS[1]}}"

  case "$prev" in
    --template)
      COMPREPLY=( $(compgen -W "{templates}" -- "$cur") )
      return 0
      ;;
    -t)
      if [[ "$command" == "publish" ]]; then
        COMPREPLY=( $(compgen -W "{publish_targets}" -- "$cur") )
      else
        COMPREPLY=( $(compgen -W "{templates}" -- "$cur") )
      fi
      return 0
      ;;
    --target)
      COMPREPLY=( $(compgen -W "{publish_targets}" -- "$cur") )
      return 0
      ;;
    --to)
      if [[ "$command" == "publish" ]]; then
        COMPREPLY=( $(compgen -W "{publish_targets}" -- "$cur") )
      else
        COMPREPLY=( $(compgen -W "{targets}" -- "$cur") )
      fi
      return 0
      ;;
    --destination|--kind)
      COMPREPLY=( $(compgen -W "{publish_destinations}" -- "$cur") )
      return 0
      ;;
    --format)
      COMPREPLY=( $(compgen -W "{publish_formats}" -- "$cur") )
      return 0
      ;;
    completions|completion)
      COMPREPLY=( $(compgen -W "{shells}" -- "$cur") )
      return 0
      ;;
    --platform)
      COMPREPLY=( $(compgen -W "{handler_platforms}" -- "$cur") )
      return 0
      ;;
  esac

  if [[ "$cur" == -* ]]; then
    case "$command" in
      init)
        COMPREPLY=( $(compgen -W "--dry-run --force --json" -- "$cur") )
        ;;
      new)
        COMPREPLY=( $(compgen -W "--template --title --workspace --fill-profile --profile --open --force --dry-run --json" -- "$cur") )
        ;;
      open)
        COMPREPLY=( $(compgen -W "--dry-run --json" -- "$cur") )
        ;;
      convert|export)
        COMPREPLY=( $(compgen -W "--to --output --output-dir --stdout --no-manifest --option" -- "$cur") )
        ;;
      publish)
        COMPREPLY=( $(compgen -W "--target --to --destination --kind --endpoint --format --auth-header --token-env --output --json --allow-not-ready --option" -- "$cur") )
        ;;
      inspect)
        COMPREPLY=( $(compgen -W "--json --option" -- "$cur") )
        ;;
      validate|check)
        COMPREPLY=( $(compgen -W "--to --json --strict --option" -- "$cur") )
        ;;
      targets)
        COMPREPLY=( $(compgen -W "--json" -- "$cur") )
        ;;
      templates)
        COMPREPLY=( $(compgen -W "--json --ids-only --category --query --search --markdown --body --title --workspace --fill-profile --profile" -- "$cur") )
        ;;
      outlines)
        COMPREPLY=( $(compgen -W "--json --ids-only --category --query --search --markdown --body --workspace --save --delete --name --label --summary --docs-live-type --document-type --outline-file --section --tag --best-for" -- "$cur") )
        ;;
      snippets|parts)
        COMPREPLY=( $(compgen -W "--json --ids-only --kind --query --search --markdown --body --workspace --fill-profile --profile" -- "$cur") )
        ;;
      transform-templates|xforms)
        COMPREPLY=( $(compgen -W "--json --ids-only --category --transform --kind --query --search --markdown --body" -- "$cur") )
        ;;
      profile|business-profile)
        COMPREPLY=( $(compgen -W "--workspace --set --get --fields --init --force --dry-run --json --markdown --placeholders --placeholder-text" -- "$cur") )
        ;;
      rfp|rfp-response|analyze-rfp)
        COMPREPLY=( $(compgen -W "--source-type --kind --url --output --matrix-output --workspace --context --notes --json --markdown --matrix" -- "$cur") )
        ;;
      handlers|transform-handlers)
        COMPREPLY=( $(compgen -W "--json --commands-only --platform" -- "$cur") )
        ;;
      readiness|release-readiness)
        COMPREPLY=( $(compgen -W "--json --strict --report" -- "$cur") )
        ;;
      evidence|evidence-status)
        COMPREPLY=( $(compgen -W "--json --strict --evidence-root" -- "$cur") )
        ;;
      support|support-bundle)
        COMPREPLY=( $(compgen -W "--json --workspace --readiness-report --spec-report --engine-report --evidence-root --output" -- "$cur") )
        ;;
      doctor)
        COMPREPLY=( $(compgen -W "--json --strict --workspace" -- "$cur") )
        ;;
      default-reader)
        COMPREPLY=( $(compgen -W "--status --enable --json" -- "$cur") )
        ;;
      *)
        COMPREPLY=( $(compgen -W "--help --version" -- "$cur") )
        ;;
    esac
    return 0
  fi

  if [[ $COMP_CWORD -eq 1 ]]; then
    COMPREPLY=( $(compgen -W "{commands}" -- "$cur") )
  fi
}}
complete -F _ned ned
"#
    )
}

fn zsh_completion_script() -> String {
    let commands = CLI_COMMANDS
        .iter()
        .map(|command| format!("{command}\\:{command}"))
        .collect::<Vec<_>>()
        .join(" ");
    let templates = NEW_DOCUMENT_TEMPLATES.join(" ");
    let targets = format!("{} all", SUPPORTED_EXPORT_TARGETS.join(" "));
    let publish_targets = "blog substack html";
    let publish_destinations = "generic-webhook wordpress-rest ghost-admin substack-manual";
    let publish_formats = "html markdown text";
    let shells = COMPLETION_SHELLS.join(" ");
    let handler_platforms = "macos windows linux manual";
    format!(
        r#"#compdef ned
# zsh completion for ned
_ned() {{
  local -a commands templates targets publish_targets publish_destinations publish_formats shells handler_platforms
  commands=({commands})
  templates=({templates})
  targets=({targets})
  publish_targets=({publish_targets})
  publish_destinations=({publish_destinations})
  publish_formats=({publish_formats})
  shells=({shells})
  handler_platforms=({handler_platforms})

  case $words[2] in
    init)
      _arguments '1:workspace directory:_files -/' '--dry-run[preview action]' '--force[replace scaffold files]' '--json[print machine-readable JSON]'
      ;;
    new)
      _arguments '*:markdown file:_files -g "*.md"' '--template[choose starter template]:template:($templates)' '--title[set document title]:title:' '--workspace[workspace containing .neditor]:directory:_files -/' '--fill-profile[merge saved business profile values into the starter document]' '--profile[alias for --fill-profile]' '--open[open after creating]' '--force[replace existing file]' '--dry-run[preview action]' '--json[print machine-readable JSON]'
      ;;
    open)
      _arguments '*:markdown file:_files -g "*.md"' '--dry-run[preview action]' '--json[print machine-readable JSON]'
      ;;
    convert|export)
      _arguments '*:markdown file:_files -g "*.md"' '--to[export target]:target:($targets)' '--output[output file, or - for text stdout]:file:_files' '--output-dir[output directory]:directory:_files -/' '--stdout[write supported text export to stdout]' '--no-manifest[skip sidecar manifest]' '--option[set export option key=value]:option:'
      ;;
    publish)
      _arguments '*:markdown file:_files -g "*.md"' '--target[publishing target]:target:($publish_targets)' '--to[publishing target alias]:target:($publish_targets)' '--destination[publishing destination]:destination:($publish_destinations)' '--kind[publishing destination alias]:destination:($publish_destinations)' '--endpoint[HTTPS publishing endpoint]:url:' '--format[payload content format]:format:($publish_formats)' '--auth-header[header name for token at handoff time]:header:' '--token-env[environment variable containing token at handoff time]:name:' '--output[write JSON payload]:file:_files' '--json[print machine-readable JSON]' '--allow-not-ready[prepare payload despite readiness errors]' '--option[set export option key=value]:option:'
      ;;
    inspect)
      _arguments '*:markdown file:_files -g "*.md"' '--json[print machine-readable JSON]' '--option[set compile option key=value]:option:'
      ;;
    validate|check)
      _arguments '*:markdown file:_files -g "*.md"' '--to[export target]:target:($targets)' '--json[print machine-readable JSON]' '--strict[treat warnings as non-zero]' '--option[set export option key=value]:option:'
      ;;
    templates)
      _arguments '--json[print machine-readable JSON]' '--ids-only[print matching template ids only]' '--category[filter by category]:category:' '--query[search templates by text]:query:' '--search[alias for --query]:query:' '--markdown[print one starter document template]:id:($templates)' '--body[alias for --markdown]:id:($templates)' '--title[set preview document title]:title:' '--workspace[workspace containing .neditor]:directory:_files -/' '--fill-profile[merge saved business profile values into the starter preview]' '--profile[alias for --fill-profile]'
      ;;
    outlines)
      _arguments '--json[print machine-readable JSON]' '--ids-only[print matching outline ids only]' '--category[filter by category]:category:' '--query[search outlines by text]:query:' '--search[alias for --query]:query:' '--markdown[print one outline as planner Markdown]:id:' '--body[alias for --markdown]:id:' '--workspace[workspace containing .neditor]:directory:_files -/' '--save[save a workspace outline id]:id:' '--delete[delete a workspace outline id]:id:' '--name[set outline display name]:name:' '--label[alias for --name]:name:' '--summary[set outline summary]:summary:' '--docs-live-type[set Docs Live workflow]:type:' '--document-type[alias for --docs-live-type]:type:' '--outline-file[read headings from a Markdown/text file]:file:_files' '--section[add one section heading]:heading:' '--tag[add a search tag]:tag:' '--best-for[add a best-fit use case]:use:'
      ;;
    snippets|parts)
      _arguments '--json[print machine-readable JSON]' '--ids-only[print matching snippet ids only]' '--kind[filter by snippet kind]:kind:' '--query[search snippets by text]:query:' '--search[alias for --query]:query:' '--markdown[print one snippet body]:id:' '--body[alias for --markdown]:id:' '--workspace[workspace containing .neditor]:directory:_files -/' '--fill-profile[merge saved business profile values into printed snippet Markdown]' '--profile[alias for --fill-profile]'
      ;;
    transform-templates|xforms)
      _arguments '--json[print machine-readable JSON]' '--ids-only[print matching transform template ids only]' '--category[filter by template category]:category:' '--transform[filter by transform type]:transform:' '--kind[alias for --transform]:transform:' '--query[search transform templates by text]:query:' '--search[alias for --query]:query:' '--markdown[print one transform template body]:id:' '--body[alias for --markdown]:id:'
      ;;
    profile|business-profile)
      _arguments '--workspace[workspace containing .neditor]:directory:_files -/' '--set[set profile field key=value]:assignment:' '--get[print one profile field]:field:' '--fields[list supported profile fields and aliases]' '--init[create profile file]' '--force[replace existing profile when initializing]' '--dry-run[preview write]' '--json[print machine-readable JSON]' '--markdown[print reusable identity block]' '--placeholders[print Docs Live placeholder values]' '--placeholder-text[alias for --placeholders]'
      ;;
    rfp|rfp-response|analyze-rfp)
      _arguments '*:RFP source:_files' '--source-type[source type]:kind:(markdown pdf docx url)' '--kind[source type alias]:kind:(markdown pdf docx url)' '--url[fetch public RFP URL]:url:' '--output[write response Markdown]:file:_files' '--matrix-output[write compliance matrix Markdown]:file:_files' '--workspace[workspace containing .neditor]:directory:_files -/' '--context[response guidance]:notes:' '--notes[response guidance alias]:notes:' '--json[print machine-readable JSON]' '--markdown[print response Markdown]' '--matrix[print compliance matrix Markdown]'
      ;;
    targets)
      _arguments '--json[print machine-readable JSON]'
      ;;
    handlers|transform-handlers)
      _arguments '--json[print machine-readable JSON]' '--commands-only[print copyable commands only]' '--platform[show setup for another platform]:platform:($handler_platforms)'
      ;;
    readiness|release-readiness)
      _arguments '--json[print machine-readable JSON]' '--strict[fail when release gaps remain]' '--report[read a specific release-readiness report]:file:_files'
      ;;
    evidence|evidence-status)
      _arguments '--json[print machine-readable JSON]' '--strict[fail when any evidence report needs attention]' '--evidence-root[read standard release evidence reports from a .tmp-style root]:directory:_files -/'
      ;;
    support|support-bundle)
      _arguments '--json[print machine-readable JSON]' '--workspace[inspect NEditor project scaffold]:directory:_files -/' '--readiness-report[attach a specific release-readiness report]:file:_files' '--spec-report[attach a specific spec-completion report]:file:_files' '--engine-report[attach a specific transform engine probe report]:file:_files' '--evidence-root[attach standard release evidence reports from a .tmp-style root]:directory:_files -/' '--output[write support bundle JSON]:file:_files'
      ;;
    completions|completion)
      _arguments '1:shell:($shells)'
      ;;
    default-reader)
      _arguments '--status[show setup status]' '--enable[request default Markdown reader setup]' '--json[print machine-readable JSON]'
      ;;
    doctor)
      _arguments '--json[print machine-readable JSON]' '--strict[fail when warnings exist]' '--workspace[inspect NEditor project scaffold]:directory:_files -/'
      ;;
    *)
      _arguments '1:command:($commands)'
      ;;
  esac
}}
_ned "$@"
"#
    )
}

fn fish_completion_script() -> String {
    let mut lines = vec![
        "# fish completion for ned".to_string(),
        "complete -c ned -f".to_string(),
    ];
    for command in CLI_COMMANDS {
        lines.push(format!(
            "complete -c ned -n '__fish_use_subcommand' -a '{command}'"
        ));
    }
    for template in NEW_DOCUMENT_TEMPLATES {
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from new' -l template -s t -a '{template}'"
        ));
    }
    for target in SUPPORTED_EXPORT_TARGETS.iter().chain(["all"].iter()) {
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from convert export' -l to -s t -a '{target}'"
        ));
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from validate check' -l to -s t -a '{target}'"
        ));
    }
    for target in ["blog", "substack", "html"] {
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from publish' -l target -s t -a '{target}'"
        ));
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from publish' -l to -a '{target}'"
        ));
    }
    for destination in [
        "generic-webhook",
        "wordpress-rest",
        "ghost-admin",
        "substack-manual",
    ] {
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from publish' -l destination -a '{destination}'"
        ));
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from publish' -l kind -a '{destination}'"
        ));
    }
    for format in ["html", "markdown", "text"] {
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from publish' -l format -a '{format}'"
        ));
    }
    for shell in COMPLETION_SHELLS {
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from completions completion' -a '{shell}'"
        ));
    }
    for platform in ["macos", "windows", "linux", "manual"] {
        lines.push(format!(
            "complete -c ned -n '__fish_seen_subcommand_from handlers transform-handlers' -l platform -a '{platform}'"
        ));
    }
    lines.extend([
        "complete -c ned -n '__fish_seen_subcommand_from init' -l json".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from init' -l force".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from init' -l dry-run".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new' -l title".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new' -l workspace -s w -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new' -l fill-profile".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new' -l profile".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new' -l open".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new' -l force".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new open' -l dry-run".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new open' -l json".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l output -s o -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l output-dir -s d -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l stdout".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l no-manifest"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l option -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from publish' -l endpoint -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from publish' -l auth-header -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from publish' -l token-env -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from publish' -l output -s o -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from publish' -l json".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from publish' -l allow-not-ready".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from publish' -l option -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates outlines transform-templates xforms targets inspect doctor' -l json"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l ids-only".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l category -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l query -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l search -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l markdown -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l body -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l title -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l workspace -s w -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l fill-profile".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates' -l profile".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l ids-only".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l category -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l query -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l search -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l markdown -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l body -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l workspace -s w -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l save -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l delete -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l name -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l label -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l summary -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l docs-live-type -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l document-type -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l outline-file -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l section -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l tag -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from outlines' -l best-for -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from transform-templates xforms' -l ids-only".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from transform-templates xforms' -l category -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from transform-templates xforms' -l transform -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from transform-templates xforms' -l kind -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from transform-templates xforms' -l query -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from transform-templates xforms' -l search -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from transform-templates xforms' -l markdown -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from transform-templates xforms' -l body -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l json".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l ids-only".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l kind -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l query -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l search -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l markdown -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l body -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l workspace -s w -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l fill-profile".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from snippets parts' -l profile".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l workspace -s w -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l set -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l get -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l fields".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l init".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l force".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l dry-run".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l json".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l markdown".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l placeholders".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from profile business-profile' -l placeholder-text".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l source-type -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l kind -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l url -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l output -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l matrix-output -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l workspace -s w -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l context -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l notes -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l json".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l markdown".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from rfp rfp-response analyze-rfp' -l matrix".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from handlers transform-handlers' -l json"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from handlers transform-handlers' -l commands-only"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from readiness release-readiness' -l json"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from readiness release-readiness' -l strict"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from readiness release-readiness' -l report -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from evidence evidence-status' -l json"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from evidence evidence-status' -l strict"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from evidence evidence-status' -l evidence-root -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from support support-bundle' -l json"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from support support-bundle' -l workspace -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from support support-bundle' -l readiness-report -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from support support-bundle' -l spec-report -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from support support-bundle' -l engine-report -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from support support-bundle' -l evidence-root -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from support support-bundle' -l output -s o -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from inspect' -l option -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from validate check' -l json".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from validate check' -l strict".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from validate check' -l option -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from doctor' -l strict".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from doctor' -l workspace -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from default-reader' -l status".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from default-reader' -l enable".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from default-reader' -l json".to_string(),
    ]);
    lines.join("\n")
}

fn parse_export_targets(value: &str) -> Result<Vec<String>, String> {
    let requested = value
        .split(',')
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .collect::<Vec<_>>();
    if requested.is_empty() {
        return Err("--to requires at least one export target".to_string());
    }
    let mut targets = Vec::new();
    for target in requested {
        if target == "all" {
            for supported_target in SUPPORTED_EXPORT_TARGETS {
                if !targets.iter().any(|existing| existing == supported_target) {
                    targets.push(supported_target.to_string());
                }
            }
            continue;
        }
        if !SUPPORTED_EXPORT_TARGETS.contains(&target) {
            return Err(format!(
                "Unsupported export target '{}'. Supported targets: {}",
                target,
                SUPPORTED_EXPORT_TARGETS.join(", ")
            ));
        }
        if !targets.iter().any(|existing| existing == target) {
            targets.push(target.to_string());
        }
    }
    Ok(targets)
}

fn apply_cli_option(options: &mut Value, pair: &str) -> Result<(), String> {
    let (key, value) = pair
        .split_once('=')
        .ok_or_else(|| "--option values must use key=value".to_string())?;
    let parsed = match value {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => value
            .parse::<i64>()
            .map(|number| json!(number))
            .unwrap_or_else(|_| json!(value)),
    };
    let object = options
        .as_object_mut()
        .ok_or_else(|| "CLI options must be an object".to_string())?;
    object.insert(key.to_string(), parsed);
    Ok(())
}

fn executable_name(name: &str) -> String {
    #[cfg(windows)]
    {
        format!("{name}.exe")
    }
    #[cfg(not(windows))]
    {
        name.to_string()
    }
}

fn default_reader_message(response: &DefaultMarkdownReaderResponse) -> String {
    let mut lines = vec![
        format!(
            "Default Markdown reader: {}",
            default_reader_status(response)
        ),
        response.message.clone(),
    ];
    if !response.commands.is_empty() {
        lines.push("Commands:".to_string());
        lines.extend(
            response
                .commands
                .iter()
                .map(|command| format!("  {command}")),
        );
    }
    if !response.manual_steps.is_empty() {
        lines.push("Manual steps:".to_string());
        lines.extend(
            response
                .manual_steps
                .iter()
                .map(|step| format!("  - {step}")),
        );
    }
    lines.join("\n")
}

fn default_reader_json_report(
    response: &DefaultMarkdownReaderResponse,
    requested_enable: bool,
    status_only: bool,
) -> Value {
    json!({
        "schema": "neditor.ned-default-reader.v1",
        "platform": &response.platform,
        "status": default_reader_status(response),
        "requestedEnable": requested_enable,
        "statusOnly": status_only,
        "enabled": response.enabled,
        "applied": response.applied,
        "supported": response.supported,
        "message": &response.message,
        "commands": &response.commands,
        "manualSteps": &response.manual_steps,
        "nextCommands": default_reader_next_commands(response),
    })
}

fn default_reader_status(response: &DefaultMarkdownReaderResponse) -> &'static str {
    if response.applied {
        "applied"
    } else if response.supported {
        "automation-available"
    } else {
        "manual-setup-required"
    }
}

fn default_reader_next_commands(response: &DefaultMarkdownReaderResponse) -> Vec<String> {
    if response.applied {
        Vec::new()
    } else if response.supported {
        vec!["ned default-reader --enable".to_string()]
    } else {
        vec!["ned default-reader --status".to_string()]
    }
}

fn validate_text_report(target: &str, strict: bool, report: &ExportReadinessReport) -> String {
    let status = if report.ready { "ready" } else { "not ready" };
    let mut lines = vec![format!(
        "Export readiness for {target}: {status} ({} errors, {} warnings, {} info)",
        report.error_count, report.warning_count, report.info_count
    )];
    if strict && report.warning_count > 0 {
        lines.push("Strict mode treats warnings as a non-zero result.".to_string());
    }
    if report.diagnostics.is_empty() {
        lines.push("No readiness diagnostics.".to_string());
        return lines.join("\n");
    }
    lines.push("Diagnostics:".to_string());
    for diagnostic in &report.diagnostics {
        let location = match (&diagnostic.source_file, diagnostic.line) {
            (Some(source), Some(line)) => format!(" [{source}:{line}]"),
            (Some(source), None) => format!(" [{source}]"),
            (None, Some(line)) => format!(" [line {line}]"),
            (None, None) => String::new(),
        };
        lines.push(format!(
            "- {}{}: {}",
            diagnostic.severity, location, diagnostic.message
        ));
        if let Some(suggestion) = diagnostic.suggestion.as_ref() {
            lines.push(format!("  suggestion: {suggestion}"));
        }
    }
    lines.join("\n")
}

fn inspect_text_report(
    input_path: &str,
    document_type: Option<&str>,
    source_word_count: usize,
    source_line_count: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    response: &CompileResponse,
) -> String {
    let mut lines = vec![
        "NEditor document inspection".to_string(),
        format!("Source: {input_path}"),
        format!("Title: {}", response.semantic.title),
        format!("Status: {}", response.semantic.status),
        format!(
            "Document type: {}",
            document_type.unwrap_or("not specified")
        ),
        format!("Words: {source_word_count}"),
        format!(
            "Lines: {source_line_count} source, {} compiled",
            response.compiled_markdown.lines().count()
        ),
        format!(
            "Structure: {} headings, {} tables, {} figures, {} equations",
            response.semantic.headings.len(),
            response.semantic.tables,
            response.semantic.figures,
            response.semantic.equations
        ),
        format!(
            "References: {} citations, {} glossary terms, {} cross-references",
            response.semantic.citations.len(),
            response.semantic.glossary.len(),
            response.semantic.cross_references.len()
        ),
        format!(
            "Automation: {} includes, {} formulas, {} transform artifacts",
            response.include_graph.len(),
            response.formula_graph.len(),
            response.transform_artifacts.len()
        ),
        format!("Diagnostics: {error_count} errors, {warning_count} warnings, {info_count} info"),
    ];
    if !response.semantic.headings.is_empty() {
        lines.push("Outline:".to_string());
        for heading in response.semantic.headings.iter().take(12) {
            lines.push(format!(
                "  - H{} line {}: {}",
                heading.level, heading.line, heading.text
            ));
        }
        if response.semantic.headings.len() > 12 {
            lines.push(format!(
                "  - ... {} more headings",
                response.semantic.headings.len() - 12
            ));
        }
    }
    if !response.diagnostics.is_empty() {
        lines.push("Diagnostic details:".to_string());
        for diagnostic in response.diagnostics.iter().take(10) {
            let location = match (&diagnostic.source_file, diagnostic.line) {
                (Some(source), Some(line)) => format!(" [{source}:{line}]"),
                (Some(source), None) => format!(" [{source}]"),
                (None, Some(line)) => format!(" [line {line}]"),
                (None, None) => String::new(),
            };
            lines.push(format!(
                "  - {}{}: {}",
                diagnostic.severity, location, diagnostic.message
            ));
        }
        if response.diagnostics.len() > 10 {
            lines.push(format!(
                "  - ... {} more diagnostics",
                response.diagnostics.len() - 10
            ));
        }
    }
    lines.join("\n")
}

fn handlers_text_report(
    platform: &str,
    registered_engines: &[&str],
    missing: &[String],
    plans: &[TransformHandlerInstallerPlan],
) -> String {
    let mut lines = vec![
        format!("Transform handler setup for {platform}"),
        format!(
            "Registered external engines: {}",
            registered_engines.join(", ")
        ),
    ];
    if missing.is_empty() {
        lines.push(
            "Coverage: every registered external engine appears in the setup plan.".to_string(),
        );
    } else {
        lines.push(format!(
            "Coverage gap: missing setup coverage for {}",
            missing.join(", ")
        ));
    }
    for plan in plans {
        lines.push(String::new());
        lines.push(format!("Plan: {} ({})", plan.label, plan.id));
        lines.push(format!("Manager: {}", plan.manager));
        lines.push(format!(
            "Mode: {}{}",
            if plan.installable {
                "installable"
            } else {
                "copy-only"
            },
            if plan.requires_admin {
                ", may require administrator privileges"
            } else {
                ""
            }
        ));
        lines.push(format!("Handlers: {}", plan.handlers.join("; ")));
        if !plan.commands.is_empty() {
            lines.push("Commands:".to_string());
            lines.extend(plan.commands.iter().map(|command| format!("  {command}")));
        }
        if !plan.notes.is_empty() {
            lines.push("Notes:".to_string());
            lines.extend(plan.notes.iter().map(|note| format!("  - {note}")));
        }
    }
    lines.join("\n")
}

fn missing_transform_handler_engines(
    plans: &[TransformHandlerInstallerPlan],
    registered_engines: &[&str],
) -> Vec<String> {
    registered_engines
        .iter()
        .filter(|engine| {
            !plans.iter().any(|plan| {
                plan.engine_names
                    .iter()
                    .any(|candidate| candidate == **engine)
            })
        })
        .map(|engine| (*engine).to_string())
        .collect()
}

fn diagnostic_counts(diagnostics: &[DocumentDiagnostic]) -> (usize, usize, usize) {
    (
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == "error")
            .count(),
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == "warning")
            .count(),
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == "info")
            .count(),
    )
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

fn help_text() -> String {
    vec![
        "ned - NEditor command line".to_string(),
        "".to_string(),
        "Usage:".to_string(),
        "  ned <file.md> [more.md]".to_string(),
        "  ned init [workspace] [--dry-run] [--force] [--json]".to_string(),
        "  ned new <file.md> [--template proposal] [--title \"Client Proposal\"] [--workspace path --fill-profile] [--open] [--json]"
            .to_string(),
        "  ned open <file.md> [more.md] [--dry-run] [--json]".to_string(),
        "  ned convert <file.md|-> --to pdf,docx --output-dir exports [--no-manifest]".to_string(),
        "  ned convert <file.md|-> --to html --stdout".to_string(),
        "  ned publish <file.md|-> --target blog --endpoint https://cms.example/hook --output payload.json [--json]".to_string(),
        "  ned inspect <file.md|-> [--json]".to_string(),
        "  ned validate <file.md|-> --to pdf [--json] [--strict]".to_string(),
        "  ned export <file.md> --to docx --output out.docx".to_string(),
        "  ned templates [--json] [--category procurement] [--query tender] [--ids-only] [--markdown id] [--title title] [--workspace path --fill-profile]".to_string(),
        "  ned outlines [--workspace .] [--json] [--category Procurement] [--query RFP] [--ids-only] [--markdown id]".to_string(),
        "  ned outlines --workspace . --save custom-id --docs-live-type proposal --section \"Executive Summary\" --section \"Recommendations\" [--json]".to_string(),
        "  ned snippets [--json] [--kind procurement] [--query risk] [--ids-only] [--markdown id] [--workspace . --fill-profile]".to_string(),
        "  ned transform-templates [--json] [--category Business] [--transform calc] [--query ROI] [--ids-only] [--markdown id]".to_string(),
        "  ned profile [--workspace path] [--init] [--set fullName=...] [--get field|--fields] [--json|--markdown|--placeholders]".to_string(),
        "  ned rfp-response <rfp.md|rfp.docx|rfp.pdf|url|-> [--output response.md] [--matrix-output matrix.md] [--json|--markdown|--matrix]".to_string(),
        "  ned targets [--json]".to_string(),
        "  ned handlers [--json] [--commands-only] [--platform macos|windows|linux]".to_string(),
        "  ned readiness [--json] [--strict] [--report .tmp/release-readiness/report.json]"
            .to_string(),
        "  ned evidence [--json] [--strict] [--evidence-root .tmp]".to_string(),
        "  ned support-bundle [--json] [--workspace path] [--readiness-report path] [--spec-report path] [--engine-report path] [--evidence-root .tmp] [--output support.json]"
            .to_string(),
        "  ned completions <bash|zsh|fish>".to_string(),
        "  ned doctor [--json] [--strict] [--workspace path]".to_string(),
        "  ned default-reader --status [--json]".to_string(),
        "  ned default-reader --enable [--json]".to_string(),
        "  ned --version".to_string(),
        "".to_string(),
        format!("Templates: {}", NEW_DOCUMENT_TEMPLATES.join(", ")),
        "Outlines: proposal, rfp, rfq, tender, report, sow, capability-statement, case-study, rfp-response, rfp-technical-proposal, grant-application, standard-operating-procedure, product-requirements-document, project-charter, and more.".to_string(),
        format!(
            "Targets: {}, or all. Use comma-separated targets for delivery packs.",
            SUPPORTED_EXPORT_TARGETS.join(", ")
        ),
    ]
    .join("\n")
}
