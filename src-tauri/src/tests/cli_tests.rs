use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn ned_cli_opens_markdown_paths_in_dry_run() {
    let path = temp_markdown_path("open");
    fs::write(&path, "# CLI Open\n").expect("write markdown");
    let args = vec![
        "ned".to_string(),
        "open".to_string(),
        path.to_string_lossy().to_string(),
        "--dry-run".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("dry-run open");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("Would open"));
    assert!(outcome.message.contains("NEditor"));
}

#[test]
fn ned_cli_opens_markdown_paths_without_subcommand() {
    let path = temp_markdown_path("direct-open");
    fs::write(&path, "# Direct Open\n").expect("write markdown");
    let args = vec![
        "ned".to_string(),
        path.to_string_lossy().to_string(),
        "--dry-run".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("direct open");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("Would open"));
    assert!(outcome.message.contains(path.to_string_lossy().as_ref()));
}

#[test]
fn ned_cli_creates_new_business_document_from_template() {
    let path = temp_markdown_path("new-proposal");
    let args = vec![
        "ned".to_string(),
        "new".to_string(),
        path.to_string_lossy().to_string(),
        "--template".to_string(),
        "proposal".to_string(),
        "--title".to_string(),
        "Client Expansion Proposal".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("new proposal");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("Created"));
    let markdown = fs::read_to_string(&path).expect("new markdown");
    assert!(markdown.contains("documentType: proposal"));
    assert!(markdown.contains("# Client Expansion Proposal"));
    assert!(markdown.contains("{{owner}}"));
    assert!(markdown.contains("## Review Handoff"));

    let duplicate = crate::cli::run_cli_with_args(&args).expect_err("refuse overwrite");
    assert!(duplicate.contains("already exists"));
}

#[test]
fn ned_cli_doctor_reports_json_capabilities() {
    let args = vec![
        "ned".to_string(),
        "doctor".to_string(),
        "--json".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("doctor json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("doctor json");
    assert_eq!(report["schema"], "neditor.ned-doctor.v1");
    assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
    assert!(report["exportTargets"]
        .as_array()
        .expect("targets")
        .contains(&serde_json::json!("pdf")));
    assert!(report["templates"]
        .as_array()
        .expect("templates")
        .contains(&serde_json::json!("rfp-response")));
}

#[test]
fn ned_cli_lists_templates_and_targets_for_terminal_discovery() {
    let templates = crate::cli::run_cli_with_args(&["ned".to_string(), "templates".to_string()])
        .expect("templates list");
    assert_eq!(templates.exit_code, 0);
    assert!(templates.message.contains("proposal"));
    assert!(templates.message.contains("rfp-response"));

    let targets = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "targets".to_string(),
        "--json".to_string(),
    ])
    .expect("targets json");
    assert_eq!(targets.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&targets.message).expect("targets json");
    assert_eq!(report["schema"], "neditor.ned-targets.v1");
    assert!(report["targets"]
        .as_array()
        .expect("targets")
        .contains(&serde_json::json!("docx")));
    assert!(report["targets"]
        .as_array()
        .expect("targets")
        .contains(&serde_json::json!("epub")));
}

#[test]
fn ned_cli_generates_shell_completions_without_external_dependencies() {
    let bash = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "completions".to_string(),
        "bash".to_string(),
    ])
    .expect("bash completions");
    assert_eq!(bash.exit_code, 0);
    assert!(bash.message.contains("complete -F _ned ned"));
    assert!(bash.message.contains("rfp-response"));
    assert!(bash.message.contains("markdown-bundle"));

    let zsh = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "completion".to_string(),
        "zsh".to_string(),
    ])
    .expect("zsh completions");
    assert_eq!(zsh.exit_code, 0);
    assert!(zsh.message.contains("#compdef ned"));
    assert!(zsh.message.contains("--output-dir"));
    assert!(zsh.message.contains("--stdout"));

    let fish = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "completions".to_string(),
        "fish".to_string(),
    ])
    .expect("fish completions");
    assert_eq!(fish.exit_code, 0);
    assert!(fish.message.contains("complete -c ned"));
    assert!(fish.message.contains("epub"));

    let unsupported = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "completions".to_string(),
        "powershell".to_string(),
    ])
    .expect_err("unsupported shell");
    assert!(unsupported.contains("Supported shells: bash, zsh, fish"));
}

#[test]
fn ned_cli_converts_markdown_to_html_export() {
    let source = temp_markdown_path("convert");
    let output = source.with_extension("html");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let args = vec![
        "ned".to_string(),
        "convert".to_string(),
        source.to_string_lossy().to_string(),
        "--to".to_string(),
        "html".to_string(),
        "--output".to_string(),
        output.to_string_lossy().to_string(),
        "--no-manifest".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("convert html");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("Exported html"));
    let html = fs::read_to_string(&output).expect("html output");
    assert!(html.contains("Test Report"));
    assert!(!output.with_extension("html.manifest.json").exists());
}

#[test]
fn ned_cli_writes_supported_text_exports_to_stdout() {
    let source = temp_markdown_path("convert-stdout");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let args = vec![
        "ned".to_string(),
        "convert".to_string(),
        source.to_string_lossy().to_string(),
        "--to".to_string(),
        "html".to_string(),
        "--stdout".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("stdout html");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("<!doctype html>"));
    assert!(outcome.message.contains("Test Report"));
    assert!(!outcome.message.contains("Exported html"));

    let stdin_args = vec![
        "ned".to_string(),
        "convert".to_string(),
        "-".to_string(),
        "--to".to_string(),
        "latex".to_string(),
        "--stdout".to_string(),
    ];
    let latex = crate::cli::run_cli_with_args_and_stdin(
        &stdin_args,
        Some("# Pipe Report\n\nA scripted document.\n"),
    )
    .expect("stdin latex");
    assert_eq!(latex.exit_code, 0);
    assert!(latex.message.contains("Pipe Report"));
    assert!(!latex.message.contains("Exported latex"));
}

#[test]
fn ned_cli_validates_export_readiness_without_writing_artifacts() {
    let source = temp_markdown_path("validate-ready");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let args = vec![
        "ned".to_string(),
        "validate".to_string(),
        source.to_string_lossy().to_string(),
        "--to".to_string(),
        "html".to_string(),
        "--json".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("validate json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("validate json");
    assert_eq!(report["schema"], "neditor.ned-validate.v1");
    assert_eq!(report["target"], "html");
    assert_eq!(report["errorCount"], 0);
    assert!(report["warningCount"].as_u64().is_some());
    assert_eq!(report["manifest"]["export_target"], "html");
    assert!(!source.with_extension("html").exists());

    let blocked_args = vec![
        "ned".to_string(),
        "check".to_string(),
        "-".to_string(),
        "--to".to_string(),
        "pptx".to_string(),
    ];
    let blocked = crate::cli::run_cli_with_args_and_stdin(
        &blocked_args,
        Some("# Pipeline Draft\n\nA draft without release metadata.\n"),
    )
    .expect("blocked readiness");
    assert_eq!(blocked.exit_code, 1);
    assert!(blocked
        .message
        .contains("Export readiness for pptx: not ready"));
    assert!(blocked.message.contains("Diagnostics:"));
    assert!(blocked.message.contains("release"));
}

#[test]
fn ned_cli_rejects_binary_stdout_exports() {
    let source = temp_markdown_path("convert-stdout-binary");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let args = vec![
        "ned".to_string(),
        "convert".to_string(),
        source.to_string_lossy().to_string(),
        "--to".to_string(),
        "pdf".to_string(),
        "--stdout".to_string(),
    ];
    let error = crate::cli::run_cli_with_args(&args).expect_err("binary stdout blocked");
    assert!(error.contains("text export targets: html, latex"));
}

#[test]
fn ned_cli_converts_to_multiple_targets_in_output_directory() {
    let source = temp_markdown_path("convert-batch");
    let output_dir = source.with_extension("outputs");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let args = vec![
        "ned".to_string(),
        "convert".to_string(),
        source.to_string_lossy().to_string(),
        "--to".to_string(),
        "html,markdown-bundle".to_string(),
        "--output-dir".to_string(),
        output_dir.to_string_lossy().to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("batch convert");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("Exported html"));
    assert!(outcome.message.contains("Exported markdown-bundle"));

    let stem = source.file_stem().and_then(|stem| stem.to_str()).unwrap();
    let html = output_dir.join(format!("{stem}-html.html"));
    let bundle = output_dir.join(format!("{stem}-markdown-bundle.zip"));
    assert!(html.is_file());
    assert!(bundle.is_file());
    assert!(html.with_extension("html.manifest.json").is_file());
    assert!(bundle.with_extension("zip.manifest.json").is_file());
    assert!(fs::read_to_string(html)
        .expect("html output")
        .contains("Test Report"));
}

#[test]
fn ned_cli_gives_batch_exports_distinct_default_names() {
    let source = temp_markdown_path("convert-batch-defaults");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let args = vec![
        "ned".to_string(),
        "convert".to_string(),
        source.to_string_lossy().to_string(),
        "--to".to_string(),
        "html,latex".to_string(),
        "--no-manifest".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("batch convert default names");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("Exported html"));
    assert!(outcome.message.contains("Exported latex"));

    let html = source.with_file_name(format!(
        "{}-html.html",
        source.file_stem().and_then(|stem| stem.to_str()).unwrap()
    ));
    let latex = source.with_file_name(format!(
        "{}-latex.tex",
        source.file_stem().and_then(|stem| stem.to_str()).unwrap()
    ));
    assert!(html.is_file());
    assert!(latex.is_file());
    assert_ne!(fs::metadata(html).expect("html export").len(), 0);
    assert_ne!(fs::metadata(latex).expect("latex export").len(), 0);
}

#[test]
fn ned_cli_help_names_supported_conversion_targets() {
    let args = vec!["ned".to_string(), "--help".to_string()];
    let outcome = crate::cli::run_cli_with_args(&args).expect("help");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("ned convert"));
    assert!(outcome.message.contains("--output-dir"));
    assert!(outcome.message.contains("--stdout"));
    assert!(outcome.message.contains("ned new"));
    assert!(outcome.message.contains("ned validate"));
    assert!(outcome.message.contains("ned templates"));
    assert!(outcome.message.contains("ned targets"));
    assert!(outcome.message.contains("ned completions"));
    assert!(outcome.message.contains("ned doctor"));
    assert!(outcome.message.contains("docx"));
    assert!(outcome.message.contains("epub"));
    assert!(outcome.message.contains("or all"));
    assert!(outcome.message.contains("rfp-response"));
}

fn temp_markdown_path(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("neditor-ned-{label}-{unique}.md"))
}
