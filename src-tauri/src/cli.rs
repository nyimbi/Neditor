use crate::{
    compile_with_options,
    export_commands::{
        export_document, prepare_for_export, ExportReadinessReport, ExportRequest,
        PrepareExportRequest,
    },
    metadata_string,
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
const NEW_DOCUMENT_TEMPLATES: &[&str] = &[
    "blank",
    "proposal",
    "rfp-response",
    "report",
    "lesson-plan",
    "textbook",
    "novel",
];
const CLI_COMMANDS: &[&str] = &[
    "init",
    "new",
    "open",
    "convert",
    "export",
    "inspect",
    "validate",
    "check",
    "templates",
    "targets",
    "handlers",
    "transform-handlers",
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
        "inspect" => run_inspect_command(&args[2..], stdin_text),
        "validate" | "check" => run_validate_command(&args[2..], stdin_text),
        "templates" => run_list_command("templates", NEW_DOCUMENT_TEMPLATES, &args[2..]),
        "targets" => run_list_command("targets", SUPPORTED_EXPORT_TARGETS, &args[2..]),
        "handlers" | "transform-handlers" => run_handlers_command(&args[2..]),
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

fn run_open_command(args: &[String]) -> Result<CliOutcome, String> {
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let paths = args
        .iter()
        .filter(|arg| arg.as_str() != "--dry-run")
        .map(|arg| canonical_path_string(&PathBuf::from(arg)))
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() {
        return Err("Usage: ned open <file.md> [more.md] [--dry-run]".to_string());
    }
    if dry_run {
        return Ok(CliOutcome {
            message: format!("Would open {} in NEditor", paths.join(", ")),
            exit_code: 0,
        });
    }
    open_paths_in_neditor(&paths)?;
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
    let mut should_open = false;
    let mut force = false;
    let mut dry_run = false;
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
            "--open" => should_open = true,
            "--force" => force = true,
            "--dry-run" => dry_run = true,
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
    let markdown = new_document_markdown(&template, &resolved_title)?;
    if dry_run {
        return Ok(CliOutcome {
            message: format!(
                "Would create {} from template '{}' with title '{}'",
                output.display(),
                template,
                resolved_title
            ),
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
    let enable = args.iter().any(|arg| arg == "--enable");
    let status_only = args.is_empty() || args.iter().any(|arg| arg == "--status");
    let response = default_markdown_reader_response(enable, enable);
    Ok(CliOutcome {
        message: default_reader_message(&response),
        exit_code: if status_only || response.supported {
            0
        } else {
            1
        },
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

fn run_doctor_command(args: &[String]) -> Result<CliOutcome, String> {
    let json_output = args.iter().any(|arg| arg == "--json");
    let strict = args.iter().any(|arg| arg == "--strict");
    if let Some(unsupported) = args
        .iter()
        .find(|arg| !matches!(arg.as_str(), "--json" | "--strict"))
    {
        return Err(format!("Unsupported doctor option '{unsupported}'"));
    }
    let current_exe = env::current_exe().ok();
    let app_binary = find_neditor_binary();
    let default_reader = default_markdown_reader_response(false, false);
    let warnings = doctor_warnings(app_binary.as_ref(), &default_reader);
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

fn doctor_warnings(
    app_binary: Option<&PathBuf>,
    default_reader: &DefaultMarkdownReaderResponse,
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
    warnings
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
        "rfp-response" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: rfp-response\ntoc: true\n---\n\n# {title}\n\n## Response Strategy\n\nSummarize the buyer's stated and implied intent, win themes, and response posture.\n\n## Compliance Matrix\n\n| ID | Requirement | Response | Evidence | Owner | Status |\n| --- | --- | --- | --- | --- | --- |\n| R1 | {{{{requirement}}}} | {{{{response}}}} | {{{{evidence}}}} | {{{{owner}}}} | Draft |\n\n## Technical Response\n\nAddress every mandatory requirement with clear evidence and assumptions.\n\n## Delivery Plan\n\nOutline milestones, dependencies, risks, and governance.\n\n## Pricing And Assumptions\n\nState pricing, exclusions, validity period, and approval requirements.\n\n## Final Verification\n\n- Every stated requirement has a mapped response.\n- Implied intent and evaluation criteria have been addressed.\n- Attachments, forms, and signatures are tracked.\n"
        ),
        "report" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: business-report\ntoc: true\n---\n\n# {title}\n\n## Executive Summary\n\nSummarize the finding, implication, and recommended decision.\n\n## Situation\n\nDescribe the context, evidence base, constraints, and stakeholders.\n\n## Analysis\n\n```calc\nrevenue = 0\ncost = 0\nprofit = revenue - cost\nmargin = profit / revenue\n```\n\nExpected margin: {{{{=margin | percent}}}}\n\n## Recommendations\n\n1. Recommendation one.\n2. Recommendation two.\n3. Recommendation three.\n\n## Risks And Next Steps\n\n| Risk | Impact | Mitigation | Owner |\n| --- | --- | --- | --- |\n| {{{{risk}}}} | {{{{impact}}}} | {{{{mitigation}}}} | {{{{owner}}}} |\n"
        ),
        "lesson-plan" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: lesson-plan\ntoc: true\n---\n\n# {title}\n\n## Learning Objectives\n\n- Learners can explain {{{{concept}}}}.\n- Learners can apply {{{{skill}}}} in a realistic scenario.\n\n## Audience And Prerequisites\n\nDescribe learner profile, prior knowledge, materials, and accessibility needs.\n\n## Lesson Flow\n\n| Time | Activity | Instructor Action | Learner Evidence |\n| ---: | --- | --- | --- |\n| 10 min | Opening | Frame the problem | Questions captured |\n| 30 min | Practice | Guide the worked example | Exercise completed |\n| 10 min | Review | Check understanding | Exit ticket |\n\n## Assessment\n\nDefine rubric, success criteria, remediation, and extension activities.\n"
        ),
        "textbook" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: textbook\ntoc: true\n---\n\n# {title}\n\n## Book Outline\n\n### Chapter 1: Foundations\n\n- Learning goals\n- Key concepts\n- Worked examples\n- Exercises\n\n### Chapter 2: Applied Practice\n\n- Case study\n- Common errors\n- Review questions\n\n## Drafting Plan\n\nUse Docs Live to flesh out chapters sequentially only after the outline is reviewed.\n"
        ),
        "novel" => format!(
            "---\ntitle: {escaped_title}\nstatus: draft\ndocumentType: novel\ntoc: true\n---\n\n# {title}\n\n## Premise\n\nWrite the central dramatic question, protagonist want, stakes, and setting.\n\n## Cast\n\n| Character | Desire | Conflict | Arc |\n| --- | --- | --- | --- |\n| {{{{name}}}} | {{{{desire}}}} | {{{{conflict}}}} | {{{{arc}}}} |\n\n## Plot Outline\n\n### Act I\n\nSet up the world, inciting incident, and first irreversible choice.\n\n### Act II\n\nEscalate pressure, reversals, midpoint, and cost.\n\n### Act III\n\nResolve the conflict, consequence, and final image.\n\n## Narrative Review\n\nCheck voice, pacing, continuity, scene purpose, and emotional progression before drafting chapters.\n"
        ),
        _ => unreachable!(),
    };
    Ok(body)
}

fn workspace_init_entries(root: &Path) -> Vec<(PathBuf, &'static str)> {
    let base = root.join(".neditor");
    vec![
        (
            base.join("README.md"),
            "# NEditor Workspace\n\nThis folder stores reusable local project material for NEditor.\n\n- `variables.yaml` supplies project variables that documents can reference with `{{variable}}` placeholders.\n- `snippets/` stores reusable document parts for proposals, RFPs, reports, tutorials, and review handoffs.\n- `agent-handoffs/` stores generated local-agent packets for Claude Code, Codex, OpenCode, or private workflows.\n\nDo not store API keys, passwords, or client secrets in this folder.\n",
        ),
        (
            base.join("variables.yaml"),
            "# Project variables available to NEditor documents.\n# Replace these examples with values your documents reuse often.\nprofile:\n  owner: \"Your Name\"\n  email: \"you@example.com\"\ncompany:\n  name: \"Your Company\"\n  website: \"https://example.com\"\nclient:\n  name: \"Client Name\"\nproject:\n  name: \"Project Name\"\n  review_date: \"YYYY-MM-DD\"\n",
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
    lines.push("  - Edit .neditor/variables.yaml with reusable names, company details, and project values.".to_string());
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
    --template|-t)
      COMPREPLY=( $(compgen -W "{templates}" -- "$cur") )
      return 0
      ;;
    --to)
      COMPREPLY=( $(compgen -W "{targets}" -- "$cur") )
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
        COMPREPLY=( $(compgen -W "--template --title --open --force --dry-run" -- "$cur") )
        ;;
      open)
        COMPREPLY=( $(compgen -W "--dry-run" -- "$cur") )
        ;;
      convert|export)
        COMPREPLY=( $(compgen -W "--to --output --output-dir --stdout --no-manifest --option" -- "$cur") )
        ;;
      inspect)
        COMPREPLY=( $(compgen -W "--json --option" -- "$cur") )
        ;;
      validate|check)
        COMPREPLY=( $(compgen -W "--to --json --strict --option" -- "$cur") )
        ;;
      templates|targets)
        COMPREPLY=( $(compgen -W "--json" -- "$cur") )
        ;;
      handlers|transform-handlers)
        COMPREPLY=( $(compgen -W "--json --commands-only --platform" -- "$cur") )
        ;;
      default-reader)
        COMPREPLY=( $(compgen -W "--status --enable" -- "$cur") )
        ;;
      doctor)
        COMPREPLY=( $(compgen -W "--json --strict" -- "$cur") )
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
    let shells = COMPLETION_SHELLS.join(" ");
    let handler_platforms = "macos windows linux manual";
    format!(
        r#"#compdef ned
# zsh completion for ned
_ned() {{
  local -a commands templates targets shells handler_platforms
  commands=({commands})
  templates=({templates})
  targets=({targets})
  shells=({shells})
  handler_platforms=({handler_platforms})

  case $words[2] in
    init)
      _arguments '1:workspace directory:_files -/' '--dry-run[preview action]' '--force[replace scaffold files]' '--json[print machine-readable JSON]'
      ;;
    new)
      _arguments '*:markdown file:_files -g "*.md"' '--template[choose starter template]:template:($templates)' '--title[set document title]:title:' '--open[open after creating]' '--force[replace existing file]' '--dry-run[preview action]'
      ;;
    open)
      _arguments '*:markdown file:_files -g "*.md"' '--dry-run[preview action]'
      ;;
    convert|export)
      _arguments '*:markdown file:_files -g "*.md"' '--to[export target]:target:($targets)' '--output[output file, or - for text stdout]:file:_files' '--output-dir[output directory]:directory:_files -/' '--stdout[write supported text export to stdout]' '--no-manifest[skip sidecar manifest]' '--option[set export option key=value]:option:'
      ;;
    inspect)
      _arguments '*:markdown file:_files -g "*.md"' '--json[print machine-readable JSON]' '--option[set compile option key=value]:option:'
      ;;
    validate|check)
      _arguments '*:markdown file:_files -g "*.md"' '--to[export target]:target:($targets)' '--json[print machine-readable JSON]' '--strict[treat warnings as non-zero]' '--option[set export option key=value]:option:'
      ;;
    templates|targets)
      _arguments '--json[print machine-readable JSON]'
      ;;
    handlers|transform-handlers)
      _arguments '--json[print machine-readable JSON]' '--commands-only[print copyable commands only]' '--platform[show setup for another platform]:platform:($handler_platforms)'
      ;;
    completions|completion)
      _arguments '1:shell:($shells)'
      ;;
    default-reader)
      _arguments '--status[show setup status]' '--enable[request default Markdown reader setup]'
      ;;
    doctor)
      _arguments '--json[print machine-readable JSON]' '--strict[fail when warnings exist]'
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
        "complete -c ned -n '__fish_seen_subcommand_from new' -l open".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new' -l force".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from new open' -l dry-run".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l output -s o -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l output-dir -s d -r"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l stdout".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l no-manifest"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from convert export' -l option -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from templates targets inspect doctor' -l json"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from handlers transform-handlers' -l json"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from handlers transform-handlers' -l commands-only"
            .to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from inspect' -l option -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from validate check' -l json".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from validate check' -l strict".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from validate check' -l option -r".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from doctor' -l strict".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from default-reader' -l status".to_string(),
        "complete -c ned -n '__fish_seen_subcommand_from default-reader' -l enable".to_string(),
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
    let mut lines = vec![response.message.clone()];
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
        "  ned new <file.md> [--template proposal] [--title \"Client Proposal\"] [--open]"
            .to_string(),
        "  ned open <file.md> [more.md] [--dry-run]".to_string(),
        "  ned convert <file.md|-> --to pdf,docx --output-dir exports [--no-manifest]".to_string(),
        "  ned convert <file.md|-> --to html --stdout".to_string(),
        "  ned inspect <file.md|-> [--json]".to_string(),
        "  ned validate <file.md|-> --to pdf [--json] [--strict]".to_string(),
        "  ned export <file.md> --to docx --output out.docx".to_string(),
        "  ned templates [--json]".to_string(),
        "  ned targets [--json]".to_string(),
        "  ned handlers [--json] [--commands-only] [--platform macos|windows|linux]".to_string(),
        "  ned completions <bash|zsh|fish>".to_string(),
        "  ned doctor [--json] [--strict]".to_string(),
        "  ned default-reader --status".to_string(),
        "  ned default-reader --enable".to_string(),
        "  ned --version".to_string(),
        "".to_string(),
        format!("Templates: {}", NEW_DOCUMENT_TEMPLATES.join(", ")),
        format!(
            "Targets: {}, or all. Use comma-separated targets for delivery packs.",
            SUPPORTED_EXPORT_TARGETS.join(", ")
        ),
    ]
    .join("\n")
}
