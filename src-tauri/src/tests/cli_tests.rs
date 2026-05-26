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

    let json_args = vec![
        "ned".to_string(),
        "open".to_string(),
        path.to_string_lossy().to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ];
    let json = crate::cli::run_cli_with_args(&json_args).expect("dry-run open json");
    assert_eq!(json.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&json.message).expect("open json");
    assert_eq!(report["schema"], "neditor.ned-open.v1");
    assert_eq!(report["dryRun"], true);
    assert_eq!(report["opened"], false);
    assert_eq!(report["count"], 1);
    assert_eq!(
        report["paths"][0],
        fs::canonicalize(&path)
            .expect("canonical open path")
            .to_string_lossy()
            .as_ref()
    );

    let error = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "open".to_string(),
        path.to_string_lossy().to_string(),
        "--mystery".to_string(),
    ])
    .expect_err("unsupported open option");
    assert!(error.contains("Unsupported open option"));
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
fn ned_cli_initializes_project_workspace_scaffold() {
    let root = temp_workspace_path("init");
    fs::create_dir_all(&root).expect("create workspace root");
    let args = vec![
        "ned".to_string(),
        "init".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("init workspace");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("init json");
    assert_eq!(report["schema"], "neditor.ned-init.v1");
    assert_eq!(report["dryRun"], false);
    assert!(root.join(".neditor").join("README.md").is_file());
    assert!(root.join(".neditor").join("variables.yaml").is_file());
    assert!(root
        .join(".neditor")
        .join("snippets")
        .join("business.md")
        .is_file());
    assert!(root
        .join(".neditor")
        .join("agent-handoffs")
        .join(".gitkeep")
        .is_file());
    let variables =
        fs::read_to_string(root.join(".neditor").join("variables.yaml")).expect("variables");
    assert!(variables.contains("company:"));
    assert!(variables.contains("review_date"));
    let snippet = fs::read_to_string(root.join(".neditor").join("snippets").join("business.md"))
        .expect("snippet");
    assert!(snippet.contains("Compliance Matrix Starter"));
    assert!(snippet.contains("{{profile.owner}}"));

    let rerun = crate::cli::run_cli_with_args(&args).expect("idempotent init");
    let rerun_report: serde_json::Value = serde_json::from_str(&rerun.message).expect("rerun json");
    assert!(rerun_report["created"]
        .as_array()
        .expect("created")
        .is_empty());
    assert!(rerun_report["kept"].as_array().expect("kept").len() >= 4);

    let dry_root = temp_workspace_path("init-dry-run");
    let dry_args = vec![
        "ned".to_string(),
        "init".to_string(),
        dry_root.to_string_lossy().to_string(),
        "--dry-run".to_string(),
    ];
    let dry_run = crate::cli::run_cli_with_args(&dry_args).expect("dry-run init");
    assert_eq!(dry_run.exit_code, 0);
    assert!(dry_run.message.contains("Would initialize"));
    assert!(!dry_root.exists());
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

    let json_path = temp_markdown_path("new-proposal-json");
    let json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "new".to_string(),
        json_path.to_string_lossy().to_string(),
        "--template".to_string(),
        "proposal".to_string(),
        "--title".to_string(),
        "JSON Proposal".to_string(),
        "--json".to_string(),
    ])
    .expect("new proposal json");
    assert_eq!(json.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&json.message).expect("new json");
    assert_eq!(report["schema"], "neditor.ned-new.v1");
    assert_eq!(report["created"], true);
    assert_eq!(report["opened"], false);
    assert_eq!(report["template"], "proposal");
    assert_eq!(report["title"], "JSON Proposal");
    assert!(json_path.is_file());

    let dry_path = temp_markdown_path("new-proposal-dry-json");
    let dry_json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "new".to_string(),
        dry_path.to_string_lossy().to_string(),
        "--template".to_string(),
        "proposal".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ])
    .expect("new dry-run json");
    let dry_report: serde_json::Value =
        serde_json::from_str(&dry_json.message).expect("new dry json");
    assert_eq!(dry_report["schema"], "neditor.ned-new.v1");
    assert_eq!(dry_report["dryRun"], true);
    assert_eq!(dry_report["created"], false);
    assert!(!dry_path.exists());

    for (template, expected) in [
        ("rfp", "documentType: rfp"),
        ("rfq", "documentType: rfq"),
        ("tender", "documentType: tender"),
        ("tutorial", "documentType: tutorial"),
        ("lesson-content", "documentType: lesson-content"),
        ("technical-textbook", "documentType: textbook"),
        ("podcast-script", "documentType: podcast-script"),
        ("movie-script", "documentType: movie-script"),
        ("business-case", "documentType: business-case"),
        ("executive-brief", "documentType: executive-brief"),
    ] {
        let template_path = temp_markdown_path(template);
        let outcome = crate::cli::run_cli_with_args(&[
            "ned".to_string(),
            "new".to_string(),
            template_path.to_string_lossy().to_string(),
            "--template".to_string(),
            template.to_string(),
            "--title".to_string(),
            format!("{template} starter"),
            "--json".to_string(),
        ])
        .expect("new expanded template");
        assert_eq!(outcome.exit_code, 0);
        let report: serde_json::Value =
            serde_json::from_str(&outcome.message).expect("expanded template json");
        assert_eq!(report["template"], template);
        let markdown = fs::read_to_string(&template_path).expect("expanded template markdown");
        assert!(
            markdown.contains(expected),
            "missing {expected} for {template}"
        );
        assert!(
            markdown.contains("{{"),
            "missing fillable placeholders for {template}"
        );
    }
}

#[test]
fn ned_cli_doctor_reports_json_capabilities() {
    let workspace = temp_workspace_path("doctor");
    fs::create_dir_all(&workspace).expect("create doctor workspace");
    let args = vec![
        "ned".to_string(),
        "doctor".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
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
    assert_eq!(report["workspaceScaffold"]["status"], "not-initialized");
    assert!(report["workspaceScaffold"]["recommended_command"]
        .as_str()
        .expect("recommended command")
        .contains("ned init"));
    assert!(report["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .any(|warning| warning
            .as_str()
            .is_some_and(|value| value.contains("Workspace scaffold is not-initialized"))));
    assert!(report["transformHandlers"]["registeredEngines"]
        .as_array()
        .expect("registered engines")
        .contains(&serde_json::json!("plantuml")));
    assert!(report["transformHandlers"]["missingRegisteredEngines"]
        .as_array()
        .expect("missing transform engines")
        .is_empty());

    crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "init".to_string(),
        workspace.to_string_lossy().to_string(),
    ])
    .expect("init doctor workspace");
    let ready = crate::cli::run_cli_with_args(&args).expect("doctor ready json");
    let ready_report: serde_json::Value =
        serde_json::from_str(&ready.message).expect("ready doctor json");
    assert_eq!(ready_report["workspaceScaffold"]["status"], "ready");
    assert!(ready_report["workspaceScaffold"]["recommended_command"].is_null());
    assert!(!ready_report["warnings"]
        .as_array()
        .expect("ready warnings")
        .iter()
        .any(|warning| warning
            .as_str()
            .is_some_and(|value| value.contains("Workspace scaffold"))));
}

#[test]
fn ned_cli_lists_templates_and_targets_for_terminal_discovery() {
    let templates = crate::cli::run_cli_with_args(&["ned".to_string(), "templates".to_string()])
        .expect("templates list");
    assert_eq!(templates.exit_code, 0);
    assert!(templates.message.contains("NEditor document templates"));
    assert!(templates.message.contains("Business development"));
    assert!(templates.message.contains("proposal"));
    assert!(templates.message.contains("tender"));
    assert!(templates.message.contains("rfp-response"));
    assert!(templates.message.contains("podcast-script"));

    let templates_json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--json".to_string(),
    ])
    .expect("templates json");
    let template_report: serde_json::Value =
        serde_json::from_str(&templates_json.message).expect("templates json");
    assert_eq!(template_report["schema"], "neditor.ned-templates.v1");
    assert_eq!(template_report["count"], 17);
    assert!(template_report["templateDetails"]
        .as_array()
        .expect("template details")
        .iter()
        .any(|template| template["id"] == "tender"
            && template["category"] == "Procurement"
            && template["summary"]
                .as_str()
                .is_some_and(|summary| summary.contains("tender"))));
    for template in [
        "rfp",
        "rfq",
        "tender",
        "tutorial",
        "podcast-script",
        "movie-script",
    ] {
        assert!(template_report["templates"]
            .as_array()
            .expect("templates")
            .contains(&serde_json::json!(template)));
    }
    let filtered = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--category".to_string(),
        "Procurement".to_string(),
        "--query".to_string(),
        "quote".to_string(),
        "--json".to_string(),
    ])
    .expect("filtered templates json");
    let filtered_report: serde_json::Value =
        serde_json::from_str(&filtered.message).expect("filtered templates json");
    assert_eq!(filtered_report["count"], 1);
    assert_eq!(filtered_report["templates"][0], "rfq");

    let ids_only = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--category".to_string(),
        "Media".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("template ids only");
    assert_eq!(ids_only.message, "podcast-script\nmovie-script");

    let snippets = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "snippets".to_string(),
        "--kind".to_string(),
        "procurement".to_string(),
        "--json".to_string(),
    ])
    .expect("snippets json");
    let snippet_report: serde_json::Value =
        serde_json::from_str(&snippets.message).expect("snippets json");
    assert_eq!(snippet_report["schema"], "neditor.ned-snippets.v1");
    assert_eq!(snippet_report["count"], 2);
    assert!(snippet_report["snippetDetails"]
        .as_array()
        .expect("snippet details")
        .iter()
        .any(|snippet| snippet["id"] == "rfp-compliance-matrix"
            && snippet["kind"] == "procurement"
            && snippet["body"]
                .as_str()
                .is_some_and(|body| body.contains("Compliance Matrix"))));

    let snippet_ids = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "parts".to_string(),
        "--query".to_string(),
        "risk".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("snippet ids only");
    assert_eq!(snippet_ids.message, "risk-register");

    let snippet_body = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "snippets".to_string(),
        "--markdown".to_string(),
        "review-handoff".to_string(),
    ])
    .expect("snippet markdown");
    assert!(snippet_body.message.contains("## Review Handoff"));
    assert!(snippet_body.message.contains("{{reviewer}}"));

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
fn ned_cli_lists_transform_handler_setup_plans() {
    let args = vec![
        "ned".to_string(),
        "handlers".to_string(),
        "--platform".to_string(),
        "macos".to_string(),
        "--json".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("handlers json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("handlers json");
    assert_eq!(report["schema"], "neditor.ned-handlers.v1");
    assert_eq!(report["platform"], "macos");
    assert!(report["registeredEngines"]
        .as_array()
        .expect("registered engines")
        .contains(&serde_json::json!("plantuml")));
    assert!(report["missingRegisteredEngines"]
        .as_array()
        .expect("missing engines")
        .is_empty());
    assert!(report["plans"][0]["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .any(|command| command
            .as_str()
            .is_some_and(|value| value.contains("brew install"))));

    let commands = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "transform-handlers".to_string(),
        "--platform".to_string(),
        "windows".to_string(),
        "--commands-only".to_string(),
    ])
    .expect("handler commands");
    assert_eq!(commands.exit_code, 0);
    assert!(commands.message.contains("winget install"));
    assert!(commands
        .message
        .contains("cargo install pikchr-cli --locked"));

    let text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "handlers".to_string(),
        "--platform".to_string(),
        "linux".to_string(),
    ])
    .expect("handlers text");
    assert_eq!(text.exit_code, 0);
    assert!(text.message.contains("Transform handler setup for linux"));
    assert!(text.message.contains("copy-only"));
}

#[test]
fn ned_cli_reads_release_readiness_reports_without_rerunning_checks() {
    let root = temp_workspace_path("readiness");
    fs::create_dir_all(&root).expect("create readiness root");
    let report_path = root.join("report.json");
    let report = serde_json::json!({
        "generatedAt": "2026-05-26T12:00:00.000Z",
        "platform": "darwin",
        "arch": "arm64",
        "status": "current-host-ready-with-external-gaps",
        "summary": {
            "requiredChecks": 2,
            "accepted": 2,
            "failed": 0,
            "evidenceGaps": 1
        },
        "checks": [
            {
                "id": "desktop-command-smoke",
                "path": ".tmp/desktop-smoke/native-command-report.json",
                "status": "passed",
                "accepted": true
            }
        ],
        "evidenceGaps": [
            {
                "id": "homebrew-final-cask",
                "status": "pending-release-cask",
                "evidence": ".tmp/homebrew/homebrew-packaging-report.json",
                "detail": "Set NEDITOR_HOMEBREW_CASK before publishing a tap."
            }
        ],
        "failures": []
    });
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&report).expect("report json"),
    )
    .expect("write readiness report");

    let text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "readiness".to_string(),
        "--report".to_string(),
        report_path.to_string_lossy().to_string(),
    ])
    .expect("readiness text");
    assert_eq!(text.exit_code, 0);
    assert!(text
        .message
        .contains("Release readiness: current-host-ready-with-external-gaps"));
    assert!(text.message.contains("Release-ready for publication: no"));
    assert!(text.message.contains("homebrew-final-cask"));
    assert!(text.message.contains("pnpm run collect:evidence-kit"));

    let json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "release-readiness".to_string(),
        "--report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("readiness json");
    assert_eq!(json.exit_code, 0);
    let normalized: serde_json::Value =
        serde_json::from_str(&json.message).expect("readiness normalized json");
    assert_eq!(normalized["schema"], "neditor.ned-readiness.v1");
    assert_eq!(normalized["releaseReady"], false);
    assert_eq!(normalized["summary"]["evidenceGaps"], 1);
    assert_eq!(normalized["evidenceGaps"][0]["id"], "homebrew-final-cask");

    let strict = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "readiness".to_string(),
        "--report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--strict".to_string(),
    ])
    .expect("strict readiness");
    assert_eq!(strict.exit_code, 1);

    let ready_report = serde_json::json!({
        "generatedAt": "2026-05-26T12:10:00.000Z",
        "platform": "darwin",
        "arch": "arm64",
        "status": "release-ready",
        "summary": {
            "requiredChecks": 2,
            "accepted": 2,
            "failed": 0,
            "evidenceGaps": 0
        },
        "checks": [],
        "evidenceGaps": [],
        "failures": []
    });
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&ready_report).expect("ready json"),
    )
    .expect("write ready report");
    let ready = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "readiness".to_string(),
        "--report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--strict".to_string(),
        "--json".to_string(),
    ])
    .expect("ready strict json");
    assert_eq!(ready.exit_code, 0);
    let ready_json: serde_json::Value =
        serde_json::from_str(&ready.message).expect("ready normalized json");
    assert_eq!(ready_json["releaseReady"], true);
}

#[test]
fn ned_cli_creates_redaction_safe_support_bundles() {
    let root = temp_workspace_path("support-bundle");
    fs::create_dir_all(&root).expect("create support root");
    let report_path = root.join("readiness.json");
    let spec_path = root.join("spec-completion.json");
    let engine_path = root.join("engine-probe.json");
    let evidence_root = root.join("evidence");
    let output_path = root.join("support").join("bundle.json");
    let readiness = serde_json::json!({
        "generatedAt": "2026-05-26T12:20:00.000Z",
        "platform": "darwin",
        "arch": "arm64",
        "status": "release-ready",
        "summary": {
            "requiredChecks": 1,
            "accepted": 1,
            "failed": 0,
            "evidenceGaps": 0
        },
        "checks": [],
        "evidenceGaps": [],
        "failures": []
    });
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&readiness).expect("readiness json"),
    )
    .expect("write readiness fixture");
    let spec_completion = serde_json::json!({
        "generatedAt": "2026-05-26T12:25:00.000Z",
        "status": "partial-with-release-risks",
        "summary": {
            "totalRows": 3,
            "completeRows": 1,
            "partialRows": 2,
            "openRows": 2
        },
        "openRows": [
            {
                "specSection": "6.5 File Operations",
                "requirementArea": "Native dialogs",
                "status": "Partial",
                "remainingGap": "Broader native proof."
            },
            {
                "specSection": "10.2 Safety",
                "requirementArea": "External engines",
                "status": "Partial",
                "remainingGap": "Cross-platform proof."
            }
        ]
    });
    fs::write(
        &spec_path,
        serde_json::to_string_pretty(&spec_completion).expect("spec json"),
    )
    .expect("write spec fixture");
    let engine_probe = serde_json::json!({
        "generatedAt": "2026-05-26T12:30:00.000Z",
        "status": "complete",
        "summary": {
            "installed": 3,
            "missingLocal": 0,
            "incompatible": 0,
            "acceptedExternalEvidence": 0,
            "invalidExternalEvidence": 0,
            "unresolvedMissingEvidence": 0
        },
        "engines": [
            {
                "key": "graphviz-dot",
                "name": "Graphviz / DOT",
                "status": "installed",
                "command": "dot",
                "path": "/usr/local/bin/dot",
                "version": "dot - graphviz version 1.0",
                "smoke": {
                    "status": "passed",
                    "artifact": ".tmp/external-engines/artifacts/dot.svg",
                    "bytes": 1200
                },
                "externalEvidence": {
                    "status": "missing",
                    "path": ".tmp/external-engines/external/graphviz-dot.json"
                }
            }
        ]
    });
    fs::write(
        &engine_path,
        serde_json::to_string_pretty(&engine_probe).expect("engine json"),
    )
    .expect("write engine fixture");
    fs::create_dir_all(evidence_root.join("platform-evidence")).expect("create platform evidence");
    fs::create_dir_all(evidence_root.join("release-signing")).expect("create signing evidence");
    fs::create_dir_all(evidence_root.join("rendered-export-audit"))
        .expect("create rendered export evidence");
    fs::write(
        evidence_root.join("platform-evidence").join("report.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "generatedAt": "2026-05-26T12:35:00.000Z",
            "status": "accepted",
            "summary": {
                "requiredPlatforms": 2,
                "completePlatforms": 2,
                "missingEvidence": 0,
                "invalidEvidence": 0
            }
        }))
        .expect("platform evidence json"),
    )
    .expect("write platform evidence fixture");
    fs::write(
        evidence_root.join("release-signing").join("report.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "generatedAt": "2026-05-26T12:36:00.000Z",
            "status": "pending-release-credentials",
            "summary": {
                "requiredPlatforms": 3,
                "completePlatforms": 0,
                "missingEvidence": 3,
                "invalidEvidence": 0
            }
        }))
        .expect("signing evidence json"),
    )
    .expect("write signing evidence fixture");

    let json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "support".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--readiness-report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--spec-report".to_string(),
        spec_path.to_string_lossy().to_string(),
        "--engine-report".to_string(),
        engine_path.to_string_lossy().to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("support json");
    assert_eq!(json.exit_code, 0);
    let bundle: serde_json::Value =
        serde_json::from_str(&json.message).expect("support bundle json");
    assert_eq!(bundle["schema"], "neditor.ned-support-bundle.v1");
    assert_eq!(bundle["privacy"]["documentContentIncluded"], false);
    assert_eq!(bundle["privacy"]["secretsIncluded"], false);
    assert_eq!(bundle["doctor"]["schema"], "neditor.ned-doctor.v1");
    assert_eq!(bundle["releaseReadiness"]["status"], "release-ready");
    assert_eq!(bundle["releaseReadiness"]["releaseReady"], true);
    assert_eq!(
        bundle["specCompletion"]["status"],
        "partial-with-release-risks"
    );
    assert_eq!(bundle["specCompletion"]["summary"]["openRows"], 2);
    assert_eq!(
        bundle["specCompletion"]["openRows"][0]["requirementArea"],
        "Native dialogs"
    );
    assert_eq!(bundle["engineProbe"]["status"], "complete");
    assert_eq!(bundle["engineProbe"]["summary"]["installed"], 3);
    assert_eq!(
        bundle["engineProbe"]["engines"][0]["smoke"]["status"],
        "passed"
    );
    assert_eq!(bundle["evidenceReportSummary"]["ready"], 1);
    assert_eq!(bundle["evidenceReportSummary"]["attention"], 1);
    assert_eq!(bundle["evidenceReportSummary"]["missing"], 8);
    assert_eq!(bundle["evidenceReports"][0]["id"], "platform-evidence");
    assert_eq!(bundle["evidenceReports"][0]["bucket"], "ready");
    assert!(bundle["recommendations"]
        .as_array()
        .expect("recommendations")
        .iter()
        .any(|recommendation| recommendation
            .as_str()
            .is_some_and(|value| value.contains("doctor warnings"))));

    let text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "support-bundle".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--readiness-report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--spec-report".to_string(),
        spec_path.to_string_lossy().to_string(),
        "--engine-report".to_string(),
        engine_path.to_string_lossy().to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--output".to_string(),
        output_path.to_string_lossy().to_string(),
    ])
    .expect("support output");
    assert_eq!(text.exit_code, 0);
    assert!(text.message.contains("NEditor support bundle"));
    assert!(text
        .message
        .contains("Spec completion: partial-with-release-risks (2 open rows)"));
    assert!(text
        .message
        .contains("Transform engines: complete (3 installed, 0 missing, 0 incompatible)"));
    assert!(text
        .message
        .contains("Evidence reports: 1 ready, 1 need attention, 8 missing"));
    assert!(text.message.contains("Wrote support bundle"));
    assert!(output_path.is_file());
    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output_path).expect("read support output"))
            .expect("written support json");
    assert_eq!(written["schema"], "neditor.ned-support-bundle.v1");
    assert!(
        written["doctor"]["workspaceScaffold"]["recommended_command"]
            .as_str()
            .expect("recommended command")
            .contains("ned init")
    );

    let ipc_output_path = root.join("support").join("ipc-bundle.json");
    let ipc_bundle = crate::cli::create_support_bundle(crate::cli::SupportBundleRequest {
        workspace: Some(root.to_string_lossy().to_string()),
        readiness_report: Some(report_path.to_string_lossy().to_string()),
        spec_report: Some(spec_path.to_string_lossy().to_string()),
        engine_report: Some(engine_path.to_string_lossy().to_string()),
        evidence_root: Some(evidence_root.to_string_lossy().to_string()),
        output: Some(ipc_output_path.to_string_lossy().to_string()),
    })
    .expect("ipc support bundle");
    assert_eq!(ipc_bundle["schema"], "neditor.ned-support-bundle.v1");
    assert_eq!(
        ipc_bundle["writtenTo"],
        ipc_output_path.to_string_lossy().as_ref()
    );
    assert!(ipc_output_path.is_file());
}

#[test]
fn ned_cli_summarizes_release_evidence_reports() {
    let root = temp_workspace_path("evidence-status");
    let evidence_root = root.join("evidence");
    fs::create_dir_all(evidence_root.join("platform-evidence")).expect("create platform evidence");
    fs::create_dir_all(evidence_root.join("release-signing")).expect("create signing evidence");
    fs::create_dir_all(evidence_root.join("rendered-export-audit"))
        .expect("create rendered export evidence");
    fs::write(
        evidence_root.join("platform-evidence").join("report.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "generatedAt": "2026-05-26T13:00:00.000Z",
            "status": "accepted",
            "summary": {
                "requiredPlatforms": 2,
                "completePlatforms": 2,
                "missingEvidence": 0,
                "invalidEvidence": 0
            }
        }))
        .expect("platform evidence json"),
    )
    .expect("write platform evidence");
    fs::write(
        evidence_root.join("release-signing").join("report.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "generatedAt": "2026-05-26T13:05:00.000Z",
            "status": "pending-release-credentials",
            "summary": {
                "requiredPlatforms": 3,
                "completePlatforms": 0,
                "missingEvidence": 3,
                "invalidEvidence": 0
            }
        }))
        .expect("signing evidence json"),
    )
    .expect("write signing evidence");
    fs::write(
        evidence_root
            .join("rendered-export-audit")
            .join("visual-review-summary.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "generatedAt": "2026-05-26T13:10:00.000Z",
            "humanSignoff": {
                "status": "pending-human-review",
                "template": "visual-review-signoff.template.json"
            },
            "automatedVisualReview": {
                "status": "automated-reviewed",
                "blockers": []
            }
        }))
        .expect("rendered export evidence json"),
    )
    .expect("write rendered export evidence");

    let text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "evidence".to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
    ])
    .expect("evidence text");
    assert_eq!(text.exit_code, 0);
    assert!(text
        .message
        .contains("NEditor evidence status: needs-attention"));
    assert!(text
        .message
        .contains("Reports: 1 ready, 2 need attention, 7 missing, 0 failed (10 total)"));
    assert!(text.message.contains("Windows/Linux platform evidence"));
    assert!(text
        .message
        .contains("Rendered export native-viewer signoff: pending-human-review"));

    let json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "evidence-status".to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("evidence json");
    assert_eq!(json.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&json.message).expect("evidence json");
    assert_eq!(report["schema"], "neditor.ned-evidence-status.v1");
    assert_eq!(report["status"], "needs-attention");
    assert_eq!(report["summary"]["ready"], 1);
    assert_eq!(report["summary"]["attention"], 2);
    assert_eq!(report["summary"]["missing"], 7);
    assert_eq!(report["reports"][0]["bucket"], "ready");
    assert!(report["reports"].as_array().unwrap().iter().any(|item| {
        item["id"] == "rendered-export-visual-signoff"
            && item["status"] == "pending-human-review"
            && item["bucket"] == "attention"
    }));

    let strict = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "evidence".to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--strict".to_string(),
    ])
    .expect("strict evidence");
    assert_eq!(strict.exit_code, 1);
}

#[test]
fn ned_cli_inspects_documents_without_writing_artifacts() {
    let source = temp_markdown_path("inspect");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let args = vec![
        "ned".to_string(),
        "inspect".to_string(),
        source.to_string_lossy().to_string(),
        "--json".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("inspect json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("inspect json");
    assert_eq!(report["schema"], "neditor.ned-inspect.v1");
    assert_eq!(report["document"]["title"], "Test Report");
    assert_eq!(report["counts"]["diagnostics"]["errors"], 0);
    assert!(report["counts"]["words"].as_u64().expect("word count") > 0);
    assert!(report["headings"]
        .as_array()
        .expect("headings")
        .iter()
        .any(|heading| heading["text"] == "Test Report"));
    assert!(report["exportTargets"]
        .as_array()
        .expect("targets")
        .contains(&serde_json::json!("docx")));
    assert!(!source.with_extension("html").exists());
    assert!(!source.with_extension("pdf").exists());

    let stdin_args = vec!["ned".to_string(), "inspect".to_string(), "-".to_string()];
    let stdin_report = crate::cli::run_cli_with_args_and_stdin(
        &stdin_args,
        Some("---\ntitle: Pipeline Memo\nstatus: draft\n---\n\n# Pipeline Memo\n\nDraft text.\n"),
    )
    .expect("stdin inspect");
    assert_eq!(stdin_report.exit_code, 0);
    assert!(stdin_report.message.contains("NEditor document inspection"));
    assert!(stdin_report.message.contains("Source: stdin.md"));
    assert!(stdin_report.message.contains("Title: Pipeline Memo"));
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
    assert!(bash.message.contains("init"));
    assert!(bash.message.contains("handlers"));
    assert!(bash.message.contains("readiness"));
    assert!(bash.message.contains("evidence"));
    assert!(bash.message.contains("snippets"));
    assert!(bash.message.contains("--markdown"));
    assert!(bash.message.contains("support-bundle"));
    assert!(bash.message.contains("inspect"));
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
    assert!(zsh.message.contains("--workspace"));
    assert!(zsh.message.contains("--report"));
    assert!(zsh.message.contains("--readiness-report"));
    assert!(zsh.message.contains("--spec-report"));
    assert!(zsh.message.contains("--engine-report"));
    assert!(zsh.message.contains("--evidence-root"));
    assert!(zsh.message.contains("--ids-only"));
    assert!(zsh.message.contains("--markdown"));

    let fish = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "completions".to_string(),
        "fish".to_string(),
    ])
    .expect("fish completions");
    assert_eq!(fish.exit_code, 0);
    assert!(fish.message.contains("complete -c ned"));
    assert!(fish.message.contains("init"));
    assert!(fish.message.contains("handlers"));
    assert!(fish.message.contains("readiness"));
    assert!(fish.message.contains("evidence"));
    assert!(fish.message.contains("snippets"));
    assert!(fish.message.contains("ids-only"));
    assert!(fish.message.contains("support-bundle"));
    assert!(fish.message.contains("inspect"));
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
fn ned_cli_reports_default_reader_setup_as_json() {
    let outcome = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "default-reader".to_string(),
        "--status".to_string(),
        "--json".to_string(),
    ])
    .expect("default reader json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value =
        serde_json::from_str(&outcome.message).expect("default reader json");
    assert_eq!(report["schema"], "neditor.ned-default-reader.v1");
    assert!(report["platform"].as_str().is_some());
    assert!(report["status"].as_str().is_some());
    assert_eq!(report["requestedEnable"], false);
    assert_eq!(report["statusOnly"], true);
    assert!(report["supported"].as_bool().is_some());
    assert!(report["commands"].as_array().is_some());
    assert!(report["manualSteps"].as_array().is_some());
    assert!(report["nextCommands"].as_array().is_some());

    let text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "default-reader".to_string(),
        "--status".to_string(),
    ])
    .expect("default reader text");
    assert_eq!(text.exit_code, 0);
    assert!(text.message.contains("Default Markdown reader:"));

    let error = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "default-reader".to_string(),
        "--mystery".to_string(),
    ])
    .expect_err("unsupported default reader option");
    assert!(error.contains("Unsupported default-reader option"));
}

#[test]
fn ned_cli_help_names_supported_conversion_targets() {
    let args = vec!["ned".to_string(), "--help".to_string()];
    let outcome = crate::cli::run_cli_with_args(&args).expect("help");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("ned convert"));
    assert!(outcome.message.contains("--output-dir"));
    assert!(outcome.message.contains("--stdout"));
    assert!(outcome.message.contains("ned init"));
    assert!(outcome.message.contains("ned new"));
    assert!(outcome.message.contains("ned inspect"));
    assert!(outcome.message.contains("ned validate"));
    assert!(outcome.message.contains("ned templates"));
    assert!(outcome.message.contains("ned snippets"));
    assert!(outcome.message.contains("ned targets"));
    assert!(outcome.message.contains("ned handlers"));
    assert!(outcome.message.contains("ned readiness"));
    assert!(outcome.message.contains("ned evidence"));
    assert!(outcome.message.contains("ned support-bundle"));
    assert!(outcome
        .message
        .contains("ned default-reader --status [--json]"));
    assert!(outcome.message.contains("ned completions"));
    assert!(outcome.message.contains("ned doctor"));
    assert!(outcome.message.contains("--workspace"));
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

fn temp_workspace_path(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("neditor-ned-{label}-{unique}"))
}
