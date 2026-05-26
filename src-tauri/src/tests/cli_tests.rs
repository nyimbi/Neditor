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
fn ned_cli_help_names_supported_conversion_targets() {
    let args = vec!["ned".to_string(), "--help".to_string()];
    let outcome = crate::cli::run_cli_with_args(&args).expect("help");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("ned convert"));
    assert!(outcome.message.contains("docx"));
    assert!(outcome.message.contains("epub"));
}

fn temp_markdown_path(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("neditor-ned-{label}-{unique}.md"))
}
