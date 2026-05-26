use crate::export_commands::{export_document, ExportRequest};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
const NEW_DOCUMENT_TEMPLATES: &[&str] = &[
    "blank",
    "proposal",
    "rfp-response",
    "report",
    "lesson-plan",
    "textbook",
    "novel",
];

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
        "new" => run_new_command(&args[2..]),
        "open" => run_open_command(&args[2..]),
        "convert" | "export" => run_convert_command(&args[2..]),
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

fn run_convert_command(args: &[String]) -> Result<CliOutcome, String> {
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
            value if value.starts_with('-') => {
                return Err(format!("Unsupported convert option '{value}'"));
            }
            value => {
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
    let input_path = canonical_path_string(&PathBuf::from(input.ok_or_else(|| {
        "Usage: ned convert <file.md> --to pdf,docx --output-dir exports".to_string()
    })?))?;
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

    let text = fs::read_to_string(&input_path)
        .map_err(|err| format!("Could not read input document {input_path}: {err}"))?;
    let mut messages = Vec::new();
    let include_target_suffix = targets.len() > 1;
    for target in targets {
        let output_path = target_output_path(
            &input_path,
            &target,
            output.as_ref(),
            output_dir.as_ref(),
            include_target_suffix,
        );
        let response = export_document(ExportRequest {
            text: text.clone(),
            file_path: Some(input_path.clone()),
            target: target.clone(),
            output_path: output_path.to_string_lossy().to_string(),
            options: options.clone(),
        })?;
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

fn help_text() -> String {
    vec![
        "ned - NEditor command line".to_string(),
        "".to_string(),
        "Usage:".to_string(),
        "  ned <file.md> [more.md]".to_string(),
        "  ned new <file.md> [--template proposal] [--title \"Client Proposal\"] [--open]"
            .to_string(),
        "  ned open <file.md> [more.md] [--dry-run]".to_string(),
        "  ned convert <file.md> --to pdf,docx --output-dir exports [--no-manifest]".to_string(),
        "  ned export <file.md> --to docx --output out.docx".to_string(),
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
