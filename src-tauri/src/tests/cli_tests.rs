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
        .join("business-profile.json")
        .is_file());
    assert!(root.join(".neditor").join("outlines.json").is_file());
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
    let profile =
        fs::read_to_string(root.join(".neditor").join("business-profile.json")).expect("profile");
    assert!(profile.contains("\"companyName\""));
    let outlines =
        fs::read_to_string(root.join(".neditor").join("outlines.json")).expect("outlines");
    assert!(outlines.contains("neditor.workspace-outlines.v1"));
    assert!(outlines.contains("Quarterly Business Review"));
    assert!(profile.contains("\"brandVoice\""));
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
fn ned_cli_manages_reusable_business_profile() {
    let root = temp_workspace_path("profile");
    fs::create_dir_all(&root).expect("create profile workspace");
    let outcome = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "profile".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--init".to_string(),
        "--set".to_string(),
        "fullName=Jane Doe".to_string(),
        "--set".to_string(),
        "email=jane@example.com".to_string(),
        "--set".to_string(),
        "roleTitle=Managing Partner".to_string(),
        "--set".to_string(),
        "companyName=Acme Advisory".to_string(),
        "--set".to_string(),
        "phone=+1 555 0100".to_string(),
        "--set".to_string(),
        "website=https://acme.example".to_string(),
        "--set".to_string(),
        "brandVoice=clear and practical".to_string(),
        "--json".to_string(),
    ])
    .expect("profile json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("profile json");
    assert_eq!(report["schema"], "neditor.ned-profile.v1");
    assert_eq!(report["written"], true);
    assert_eq!(report["profile"]["fullName"], "Jane Doe");
    assert_eq!(report["profile"]["companyName"], "Acme Advisory");
    assert!(report["placeholderText"]
        .as_str()
        .expect("placeholder text")
        .contains("brandVoice: clear and practical"));
    assert!(root
        .join(".neditor")
        .join("business-profile.json")
        .is_file());

    let markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "profile".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--markdown".to_string(),
    ])
    .expect("profile markdown");
    assert!(markdown.message.contains("## Business Identity"));
    assert!(markdown.message.contains("Jane Doe"));
    assert!(markdown.message.contains("Acme Advisory"));

    let placeholders = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "business-profile".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--placeholders".to_string(),
    ])
    .expect("profile placeholders");
    assert!(placeholders.message.contains("fullName: Jane Doe"));
    assert!(placeholders.message.contains("companyName: Acme Advisory"));

    let filled_snippet = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "snippets".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--markdown".to_string(),
        "company-contact-block".to_string(),
        "--fill-profile".to_string(),
    ])
    .expect("filled profile snippet");
    assert!(filled_snippet
        .message
        .contains("Jane Doe, Managing Partner"));
    assert!(filled_snippet.message.contains("Acme Advisory"));
    assert!(filled_snippet.message.contains("jane@example.com"));
    assert!(filled_snippet.message.contains("https://acme.example"));
    assert!(!filled_snippet.message.contains("{{fullName}}"));

    let filled_snippet_json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "parts".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--markdown".to_string(),
        "review-handoff".to_string(),
        "--fill-profile".to_string(),
        "--json".to_string(),
    ])
    .expect("filled profile snippet json");
    let filled_snippet_report: serde_json::Value =
        serde_json::from_str(&filled_snippet_json.message).expect("filled snippet json");
    assert_eq!(filled_snippet_report["schema"], "neditor.ned-snippet.v1");
    assert_eq!(filled_snippet_report["profileApplied"], true);
    assert!(filled_snippet_report["markdown"]
        .as_str()
        .expect("filled markdown")
        .contains("Final reviewer: Jane Doe."));
    assert!(filled_snippet_report["rawMarkdown"]
        .as_str()
        .expect("raw markdown")
        .contains("{{reviewer}}"));

    let fields = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "profile".to_string(),
        "--fields".to_string(),
        "--json".to_string(),
    ])
    .expect("profile fields");
    let fields_report: serde_json::Value =
        serde_json::from_str(&fields.message).expect("fields json");
    assert_eq!(fields_report["schema"], "neditor.ned-profile-fields.v1");
    assert!(fields_report["fields"]
        .as_array()
        .expect("profile fields")
        .iter()
        .any(|field| field["field"] == "defaultClientName"
            && field["aliases"]
                .as_array()
                .expect("aliases")
                .contains(&serde_json::json!("client"))));

    let single = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "profile".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--get".to_string(),
        "company".to_string(),
        "--json".to_string(),
    ])
    .expect("single profile value");
    let single_report: serde_json::Value =
        serde_json::from_str(&single.message).expect("single value json");
    assert_eq!(single_report["schema"], "neditor.ned-profile-value.v1");
    assert_eq!(single_report["field"], "companyName");
    assert_eq!(single_report["value"], "Acme Advisory");

    let unset_single = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "business-profile".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--get".to_string(),
        "companyAddress".to_string(),
    ])
    .expect("unset profile value");
    assert_eq!(unset_single.message, "{{companyAddress}}");

    let dry = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "profile".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--set".to_string(),
        "companyName=Dry Run Company".to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ])
    .expect("profile dry-run");
    let dry_report: serde_json::Value = serde_json::from_str(&dry.message).expect("dry json");
    assert_eq!(dry_report["dryRun"], true);
    assert_eq!(dry_report["written"], false);
    let still_acme =
        fs::read_to_string(root.join(".neditor").join("business-profile.json")).expect("profile");
    assert!(still_acme.contains("Acme Advisory"));

    let unknown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "profile".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--set".to_string(),
        "secretKey=value".to_string(),
    ])
    .expect_err("unknown profile field");
    assert!(unknown.contains("Unknown profile field"));
}

#[test]
fn ned_cli_analyzes_rfp_sources_and_writes_response() {
    let workspace = temp_workspace_path("rfp-profile");
    fs::create_dir_all(&workspace).expect("create rfp workspace");
    crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "profile".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--init".to_string(),
        "--set".to_string(),
        "fullName=Jane Doe".to_string(),
        "--set".to_string(),
        "companyName=Acme Advisory".to_string(),
        "--set".to_string(),
        "defaultClientName=Globex".to_string(),
        "--json".to_string(),
    ])
    .expect("profile setup");
    let source = temp_markdown_path("customer-support-rfp");
    fs::write(
        &source,
        [
            "# Globex Customer Support RFP",
            "",
            "Purpose: Globex seeks a partner to improve customer support operations and reduce implementation risk.",
            "1. Vendor must provide a phased implementation plan within 90 days.",
            "2. Proposer shall include pricing, payment terms, and all assumptions.",
            "3. Vendor must demonstrate SOC 2 security controls and data protection practices.",
            "4. Submit signed insurance certificate and three relevant customer references.",
            "Evaluation criteria: technical merit 40 points, price 30 points, experience 30 points.",
        ]
        .join("\n"),
    )
    .expect("write rfp");
    let response = temp_markdown_path("customer-support-rfp-response");
    let matrix = temp_markdown_path("customer-support-rfp-matrix");
    let outcome = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "rfp-response".to_string(),
        source.to_string_lossy().to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--context".to_string(),
        "Win theme: reduce implementation risk.".to_string(),
        "--output".to_string(),
        response.to_string_lossy().to_string(),
        "--matrix-output".to_string(),
        matrix.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("rfp response json");
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("rfp json");
    assert_eq!(report["schema"], "neditor.ned-rfp-response.v1");
    assert_eq!(report["analysis"]["source"]["kind"], "markdown");
    assert_eq!(
        report["analysis"]["requirements"]
            .as_array()
            .expect("requirements")
            .len(),
        4
    );
    assert!(report["analysis"]["evaluationCriteria"]
        .as_array()
        .expect("criteria")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("technical merit"))));
    assert!(report["analysis"]["mandatoryAttachments"]
        .as_array()
        .expect("attachments")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("insurance certificate"))));
    assert!(report["analysis"]["statedIntent"]
        .as_array()
        .expect("stated")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("improve customer support"))));
    assert!(report["analysis"]["impliedIntent"]
        .as_array()
        .expect("implied")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("easily scored response"))));
    assert_eq!(
        report["analysis"]["verificationSummary"]["allRequirementsMapped"],
        true
    );
    let response_text = fs::read_to_string(&response).expect("response markdown");
    assert!(response_text.contains("## Compliance Checklist"));
    assert!(response_text.contains("[TOC]"));
    assert!(
        response_text
            .find("## Compliance Checklist")
            .expect("checklist")
            < response_text.find("[TOC]").expect("toc")
    );
    assert!(
        response_text.find("[TOC]").expect("toc")
            < response_text
                .find("## Proposal Planning Prompt")
                .expect("planning prompt")
    );
    assert!(
        response_text
            .find("## Proposal Planning Prompt")
            .expect("planning prompt")
            < response_text
                .find("## Proposal Outline")
                .expect("proposal outline")
    );
    assert!(
        response_text
            .find("## Proposal Outline")
            .expect("proposal outline")
            < response_text
                .find("## Evaluator-Aligned Section Drafts")
                .expect("section drafts")
    );
    assert!(response_text.contains("## Compliance Matrix"));
    assert!(response_text.contains("Extract the evaluator model, scoring weights"));
    assert!(response_text.contains("Terms of Reference map"));
    assert!(response_text.contains("### Technical Methodology Draft"));
    assert!(response_text.contains("### Sustainability and Transition Draft"));
    assert!(response_text.contains("### Risk, QA, Validation, and KPI Draft"));
    assert!(response_text.contains("## Requirement Response Drafts"));
    assert!(response_text.contains("Win theme: reduce implementation risk."));
    assert!(response_text.contains("SOC 2 security controls"));
    assert!(response_text
        .contains("<!-- ai-assisted: status=needs-review | source=NEditor ned RFP Response"));
    let matrix_text = fs::read_to_string(&matrix).expect("matrix markdown");
    assert!(matrix_text.contains("| ID | Requirement | Category | Compliance status |"));
    assert!(matrix_text.contains("RFP-REQ-001"));

    let stdin_matrix = crate::cli::run_cli_with_args_and_stdin(
        &[
            "ned".to_string(),
            "analyze-rfp".to_string(),
            "-".to_string(),
            "--matrix".to_string(),
        ],
        Some("Vendor must submit pricing and implementation timeline."),
    )
    .expect("stdin rfp matrix");
    assert!(stdin_matrix.message.contains("## Compliance Matrix"));
    assert!(stdin_matrix
        .message
        .contains("pricing and implementation timeline"));
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
        ("sow", "documentType: project-plan"),
        ("capability-statement", "documentType: marketing-brief"),
        ("case-study", "documentType: customer-case-study"),
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
    assert!(templates.message.contains("capability-statement"));
    assert!(templates.message.contains("case-study"));
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
    assert_eq!(template_report["count"], 34);
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
        "sow",
        "capability-statement",
        "case-study",
        "tutorial",
        "podcast-script",
        "movie-script",
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

    let business_development = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--category".to_string(),
        "Business development".to_string(),
        "--query".to_string(),
        "statement".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("business development template ids");
    assert_eq!(business_development.message, "sow\ncapability-statement");

    let marketing = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--category".to_string(),
        "Marketing".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("marketing template ids only");
    assert_eq!(marketing.message, "case-study");

    let ids_only = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--category".to_string(),
        "Media".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("template ids only");
    assert_eq!(ids_only.message, "podcast-script\nmovie-script");

    let research_ids = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--category".to_string(),
        "Research".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("research template ids only");
    assert_eq!(
        research_ids.message,
        "research-report\nmarket-research-report"
    );

    let prd_path = temp_markdown_path("new-prd-template");
    let prd_template = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "new".to_string(),
        prd_path.to_string_lossy().to_string(),
        "--template".to_string(),
        "product-requirements-document".to_string(),
        "--title".to_string(),
        "Mobile App PRD".to_string(),
        "--json".to_string(),
    ])
    .expect("new prd json");
    assert_eq!(prd_template.exit_code, 0);
    let prd_markdown = fs::read_to_string(&prd_path).expect("prd markdown");
    assert!(prd_markdown.contains("documentType: project-plan"));
    assert!(prd_markdown.contains("## Acceptance Criteria"));
    assert!(prd_markdown.contains("## AI Drafting Brief"));

    let playbook_path = temp_markdown_path("new-implementation-playbook");
    let playbook_template = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "new".to_string(),
        playbook_path.to_string_lossy().to_string(),
        "--template".to_string(),
        "implementation-playbook".to_string(),
        "--title".to_string(),
        "Customer Rollout Playbook".to_string(),
        "--json".to_string(),
    ])
    .expect("new implementation playbook json");
    assert_eq!(playbook_template.exit_code, 0);
    let playbook_markdown = fs::read_to_string(&playbook_path).expect("playbook markdown");
    assert!(playbook_markdown.contains("documentType: project-plan"));
    assert!(playbook_markdown.contains("## Implementation Phases"));
    assert!(playbook_markdown.contains("## Runbook"));

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

    let transform_templates = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "transform-templates".to_string(),
        "--json".to_string(),
    ])
    .expect("transform templates json");
    let transform_template_report: serde_json::Value =
        serde_json::from_str(&transform_templates.message).expect("transform templates json");
    assert_eq!(
        transform_template_report["schema"],
        "neditor.ned-transform-templates.v1"
    );
    assert!(
        transform_template_report["count"]
            .as_u64()
            .expect("transform template count")
            >= 50
    );
    for template in [
        "calc-business-roi",
        "chart-business-horizontal-risk",
        "plantuml-enterprise-components",
        "vega-lite-sla-thresholds",
    ] {
        assert!(transform_template_report["templates"]
            .as_array()
            .expect("transform templates")
            .contains(&serde_json::json!(template)));
    }

    let runway_template = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "xforms".to_string(),
        "--category".to_string(),
        "Business".to_string(),
        "--transform".to_string(),
        "calc".to_string(),
        "--query".to_string(),
        "runway".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("filtered transform template ids");
    assert_eq!(runway_template.message, "calc-business-runway");

    let transform_template_body = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "transform-templates".to_string(),
        "--markdown".to_string(),
        "chart-business-horizontal-risk".to_string(),
    ])
    .expect("transform template markdown");
    assert!(transform_template_body.message.contains("```chart"));
    assert!(transform_template_body
        .message
        .contains("targetLabel: Escalation"));

    let snippet_workspace = temp_workspace_path("workspace-snippets");
    fs::create_dir_all(snippet_workspace.join(".neditor").join("snippets"))
        .expect("create workspace snippets");
    fs::write(
        snippet_workspace.join(".neditor").join("snippets").join("business.md"),
        "## Client Brief\n\nPrepared for {{companyName}}.\n\n## Decision Log\n\n| Decision | Owner |\n| --- | --- |\n| {{decision}} | {{owner}} |\n",
    )
    .expect("write workspace snippets");
    fs::write(
        snippet_workspace
            .join(".neditor")
            .join("business-profile.json"),
        "{\n  \"fullName\": \"Jane Doe\",\n  \"email\": \"jane@example.com\",\n  \"phone\": \"\",\n  \"roleTitle\": \"Managing Partner\",\n  \"companyName\": \"Acme Advisory\",\n  \"companyAddress\": \"\",\n  \"website\": \"https://acme.example\",\n  \"industry\": \"Consulting\",\n  \"defaultClientName\": \"Globex\",\n  \"brandVoice\": \"clear and practical\"\n}\n",
    )
    .expect("write workspace profile");

    let workspace_snippets = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "snippets".to_string(),
        "--workspace".to_string(),
        snippet_workspace.to_string_lossy().to_string(),
        "--kind".to_string(),
        "business".to_string(),
        "--json".to_string(),
    ])
    .expect("workspace snippets json");
    let workspace_snippet_report: serde_json::Value =
        serde_json::from_str(&workspace_snippets.message).expect("workspace snippets json");
    assert_eq!(
        workspace_snippet_report["schema"],
        "neditor.ned-snippets.v1"
    );
    assert_eq!(workspace_snippet_report["count"], 2);
    assert!(workspace_snippet_report["snippetDetails"]
        .as_array()
        .expect("workspace snippet details")
        .iter()
        .any(|snippet| snippet["id"] == "business-client-brief"
            && snippet["source"] == "workspace"
            && snippet["path"]
                .as_str()
                .is_some_and(|path| path.ends_with(".neditor/snippets/business.md"))));

    let workspace_snippet_body = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "snippets".to_string(),
        "--workspace".to_string(),
        snippet_workspace.to_string_lossy().to_string(),
        "--markdown".to_string(),
        "business-client-brief".to_string(),
        "--fill-profile".to_string(),
    ])
    .expect("workspace snippet markdown");
    assert!(workspace_snippet_body
        .message
        .contains("Prepared for Acme Advisory."));

    let outlines = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--json".to_string(),
    ])
    .expect("outlines json");
    let outline_report: serde_json::Value =
        serde_json::from_str(&outlines.message).expect("outlines json");
    assert_eq!(outline_report["schema"], "neditor.ned-outlines.v1");
    assert!(outline_report["count"].as_u64().expect("outline count") >= 32);
    assert!(outline_report["outlines"]
        .as_array()
        .expect("outlines")
        .contains(&serde_json::json!("rfp-technical-proposal")));
    for id in [
        "sow",
        "capability-statement",
        "case-study",
        "lesson-content",
        "executive-brief",
        "grant-application",
        "standard-operating-procedure",
        "product-requirements-document",
        "project-charter",
        "quarterly-business-review",
        "due-diligence-memo",
        "incident-postmortem",
        "meeting-decision-pack",
        "market-research-report",
        "contract-review-brief",
    ] {
        assert!(
            outline_report["outlines"]
                .as_array()
                .expect("outlines")
                .contains(&serde_json::json!(id)),
            "missing CLI outline {id}"
        );
    }
    assert!(outline_report["outlineDetails"]
        .as_array()
        .expect("outline details")
        .iter()
        .any(|outline| outline["id"] == "rfp-technical-proposal"
            && outline["category"] == "Procurement"
            && outline["outline"]
                .as_array()
                .expect("outline")
                .contains(&serde_json::json!("Compliance Checklist"))));

    let filtered_outlines = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--category".to_string(),
        "Research".to_string(),
        "--query".to_string(),
        "literature".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("filtered outline ids");
    assert_eq!(filtered_outlines.message, "research-report");

    let learning_outlines = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--category".to_string(),
        "Learning".to_string(),
        "--query".to_string(),
        "handout".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("learning outline ids");
    assert_eq!(learning_outlines.message, "lesson-content");

    let case_study_outlines = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--category".to_string(),
        "Business Development".to_string(),
        "--query".to_string(),
        "customer proof".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("case study outline ids");
    assert_eq!(case_study_outlines.message, "case-study");

    let legal_outlines = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--category".to_string(),
        "Legal".to_string(),
        "--json".to_string(),
    ])
    .expect("legal outline json");
    let legal_outline_report: serde_json::Value =
        serde_json::from_str(&legal_outlines.message).expect("legal outline json");
    assert!(legal_outline_report["outlineDetails"]
        .as_array()
        .expect("legal outline details")
        .iter()
        .any(|outline| outline["id"] == "contract-review-brief"
            && outline["docsLiveType"] == "contract-brief"
            && outline["outline"]
                .as_array()
                .expect("contract outline")
                .contains(&serde_json::json!("Approval Checklist"))));

    let outline_markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--markdown".to_string(),
        "rfp-technical-proposal".to_string(),
    ])
    .expect("outline markdown");
    assert!(outline_markdown.message.contains("- Compliance Checklist"));
    assert!(outline_markdown.message.contains("- Table of Contents"));

    let sop_markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--markdown".to_string(),
        "standard-operating-procedure".to_string(),
    ])
    .expect("sop outline markdown");
    assert!(sop_markdown.message.contains("- Purpose"));
    assert!(sop_markdown.message.contains("- Controls and Checks"));

    let sow_markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--markdown".to_string(),
        "sow".to_string(),
    ])
    .expect("sow outline markdown");
    assert!(sow_markdown.message.contains("- Scope"));
    assert!(sow_markdown.message.contains("- Acceptance Criteria"));

    let outline_workspace = temp_workspace_path("outline-library");
    let saved_outline = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--workspace".to_string(),
        outline_workspace.to_string_lossy().to_string(),
        "--save".to_string(),
        "qbr-custom".to_string(),
        "--name".to_string(),
        "QBR Custom".to_string(),
        "--category".to_string(),
        "Business".to_string(),
        "--summary".to_string(),
        "Quarterly business review for account teams.".to_string(),
        "--docs-live-type".to_string(),
        "board-memo".to_string(),
        "--section".to_string(),
        "Executive Summary".to_string(),
        "--section".to_string(),
        "Revenue Review".to_string(),
        "--section".to_string(),
        "Decisions Requested".to_string(),
        "--tag".to_string(),
        "quarterly".to_string(),
        "--best-for".to_string(),
        "client reviews".to_string(),
        "--json".to_string(),
    ])
    .expect("save workspace outline");
    let saved_outline_report: serde_json::Value =
        serde_json::from_str(&saved_outline.message).expect("saved outline json");
    assert_eq!(
        saved_outline_report["schema"],
        "neditor.ned-outline-save.v1"
    );
    assert_eq!(saved_outline_report["outline"]["id"], "qbr-custom");
    assert_eq!(
        saved_outline_report["outline"]["docsLiveType"],
        "board-memo"
    );
    assert!(outline_workspace
        .join(".neditor")
        .join("outlines.json")
        .is_file());

    let workspace_outlines = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--workspace".to_string(),
        outline_workspace.to_string_lossy().to_string(),
        "--query".to_string(),
        "quarterly".to_string(),
        "--json".to_string(),
    ])
    .expect("workspace outlines json");
    let workspace_outline_report: serde_json::Value =
        serde_json::from_str(&workspace_outlines.message).expect("workspace outlines json");
    assert!(workspace_outline_report["outlineDetails"]
        .as_array()
        .expect("workspace outline details")
        .iter()
        .any(|outline| outline["id"] == "qbr-custom"
            && outline["source"] == "workspace"
            && outline["docsLiveType"] == "board-memo"));

    let workspace_markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--workspace".to_string(),
        outline_workspace.to_string_lossy().to_string(),
        "--markdown".to_string(),
        "qbr-custom".to_string(),
    ])
    .expect("workspace outline markdown");
    assert!(workspace_markdown.message.contains("- Revenue Review"));

    let deleted_outline = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--workspace".to_string(),
        outline_workspace.to_string_lossy().to_string(),
        "--delete".to_string(),
        "qbr-custom".to_string(),
        "--json".to_string(),
    ])
    .expect("delete workspace outline");
    let deleted_outline_report: serde_json::Value =
        serde_json::from_str(&deleted_outline.message).expect("deleted outline json");
    assert_eq!(
        deleted_outline_report["schema"],
        "neditor.ned-outline-delete.v1"
    );
    assert_eq!(deleted_outline_report["deleted"], true);

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
    assert!(bash.message.contains("--save"));
    assert!(bash.message.contains("--outline-file"));
    assert!(bash.message.contains("--fill-profile"));
    assert!(bash.message.contains("--fields"));
    assert!(bash.message.contains("--get"));
    assert!(bash.message.contains("support-bundle"));
    assert!(bash.message.contains("inspect"));
    assert!(bash.message.contains("rfp-response"));
    assert!(bash.message.contains("publish"));
    assert!(bash.message.contains("--token-env"));
    assert!(bash.message.contains("--matrix-output"));
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
    assert!(zsh.message.contains("--best-for"));
    assert!(zsh.message.contains("--docs-live-type"));
    assert!(zsh.message.contains("--fill-profile"));
    assert!(zsh.message.contains("--fields"));
    assert!(zsh.message.contains("--get"));
    assert!(zsh.message.contains("--endpoint"));
    assert!(zsh.message.contains("--allow-not-ready"));
    assert!(zsh.message.contains("--matrix-output"));

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
    assert!(fish.message.contains("outline-file"));
    assert!(fish.message.contains("fill-profile"));
    assert!(fish.message.contains("fields"));
    assert!(fish.message.contains("get"));
    assert!(fish.message.contains("matrix-output"));
    assert!(fish.message.contains("support-bundle"));
    assert!(fish.message.contains("inspect"));
    assert!(fish.message.contains("publish"));
    assert!(fish.message.contains("token-env"));
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
fn ned_cli_prepares_publish_payload_without_persisting_secrets() {
    let source = temp_markdown_path("publish");
    let output = source.with_extension("publish.json");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let args = vec![
        "ned".to_string(),
        "publish".to_string(),
        source.to_string_lossy().to_string(),
        "--target".to_string(),
        "blog".to_string(),
        "--destination".to_string(),
        "wordpress-rest".to_string(),
        "--endpoint".to_string(),
        "https://cms.example.com/wp-json/wp/v2/posts".to_string(),
        "--format".to_string(),
        "markdown".to_string(),
        "--auth-header".to_string(),
        "X-NEditor-Token".to_string(),
        "--token-env".to_string(),
        "NEDITOR_TEST_PUBLISH_TOKEN".to_string(),
        "--output".to_string(),
        output.to_string_lossy().to_string(),
        "--allow-not-ready".to_string(),
        "--json".to_string(),
    ];
    let outcome = crate::cli::run_cli_with_args(&args).expect("publish payload");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("publish json");
    assert_eq!(report["schema"], "neditor.ned-publish.v1");
    assert_eq!(report["payload"]["schema"], "neditor.publish-payload.v1");
    assert_eq!(report["payload"]["target"], "blog");
    assert_eq!(report["payload"]["destinationKind"], "wordpress-rest");
    assert_eq!(report["payload"]["contentFormat"], "markdown");
    assert_eq!(report["payload"]["title"], "Test Report");
    assert_eq!(report["payload"]["auth"]["headerName"], "X-NEditor-Token");
    assert_eq!(
        report["payload"]["auth"]["tokenEnv"],
        "NEDITOR_TEST_PUBLISH_TOKEN"
    );
    assert_eq!(report["payload"]["auth"]["tokenPersisted"], false);
    assert_eq!(report["payload"]["auth"]["tokenPresent"], false);
    assert!(report["payload"]["content"]
        .as_str()
        .unwrap()
        .contains("# Test Report"));
    assert!(report["payload"]["curlTemplate"]
        .as_str()
        .unwrap()
        .contains("${NEDITOR_TEST_PUBLISH_TOKEN}"));
    assert!(!outcome.message.contains("secret-token-value"));

    let written = fs::read_to_string(&output).expect("written publish payload");
    let payload: serde_json::Value = serde_json::from_str(&written).expect("payload json");
    assert_eq!(payload["schema"], "neditor.publish-payload.v1");
    assert_eq!(
        payload["endpointUrl"],
        "https://cms.example.com/wp-json/wp/v2/posts"
    );
    assert!(!written.contains("secret-token-value"));
}

#[test]
fn ned_cli_rejects_unsafe_publish_endpoint_and_token_env() {
    let source = temp_markdown_path("publish-reject");
    fs::write(&source, super::sample_document()).expect("write source markdown");
    let unsafe_endpoint = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "publish".to_string(),
        source.to_string_lossy().to_string(),
        "--endpoint".to_string(),
        "http://example.com/hook".to_string(),
    ])
    .expect_err("unsafe endpoint rejected");
    assert!(unsafe_endpoint.contains("HTTPS"));

    let bad_token_env = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "publish".to_string(),
        source.to_string_lossy().to_string(),
        "--endpoint".to_string(),
        "https://cms.example.com/hook".to_string(),
        "--token-env".to_string(),
        "bad-name".to_string(),
    ])
    .expect_err("bad token env rejected");
    assert!(bad_token_env.contains("environment variable"));
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
    assert!(outcome.message.contains("ned publish"));
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
