use std::{
    fs,
    process::Command,
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
    assert!(root.join(".neditor").join("latex-templates.json").is_file());
    assert!(root.join(".neditor").join("export-profiles.json").is_file());
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
    let latex_templates = fs::read_to_string(root.join(".neditor").join("latex-templates.json"))
        .expect("latex templates");
    assert!(latex_templates.contains("neditor.workspace-latex-templates.v1"));
    assert!(latex_templates.contains("Client Report House Style"));
    let export_profiles = fs::read_to_string(root.join(".neditor").join("export-profiles.json"))
        .expect("export profiles");
    assert!(export_profiles.contains("neditor.workspace-export-profiles.v1"));
    assert!(export_profiles.contains("Client PDF Delivery"));
    assert!(profile.contains("\"brandVoice\""));
    assert!(profile.contains("\"companyLegalName\""));
    assert!(profile.contains("\"taxIdentifier\""));
    assert!(profile.contains("\"credentialsSummary\""));
    let snippet = fs::read_to_string(root.join(".neditor").join("snippets").join("business.md"))
        .expect("snippet");
    assert!(snippet.contains("Compliance Matrix Starter"));
    assert!(snippet.contains("{{profile.owner}}"));
    assert!(snippet.contains("{{company.name}}"));
    assert!(snippet.contains("{{company.website}}"));

    let filled_scaffold_snippet = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "snippets".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--markdown".to_string(),
        "business-contact-block".to_string(),
        "--fill-profile".to_string(),
    ])
    .expect("filled scaffold snippet");
    assert!(filled_scaffold_snippet
        .message
        .contains("**Prepared by:** Your Name"));
    assert!(filled_scaffold_snippet
        .message
        .contains("**Company:** Your Company"));
    assert!(filled_scaffold_snippet
        .message
        .contains("**Website:** https://example.com"));
    assert!(!filled_scaffold_snippet.message.contains("{{company.name}}"));
    assert!(!filled_scaffold_snippet
        .message
        .contains("{{company.website}}"));

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
        "companyLegalName=Acme Advisory LLC".to_string(),
        "--set".to_string(),
        "registration=DE-123456".to_string(),
        "--set".to_string(),
        "taxId=VAT-999".to_string(),
        "--set".to_string(),
        "duns=12-345-6789".to_string(),
        "--set".to_string(),
        "companyCountry=United States".to_string(),
        "--set".to_string(),
        "phone=+1 555 0100".to_string(),
        "--set".to_string(),
        "website=https://acme.example".to_string(),
        "--set".to_string(),
        "linkedin=https://linkedin.example/acme".to_string(),
        "--set".to_string(),
        "credentials=Approved supplier with climate analytics and procurement response credentials"
            .to_string(),
        "--set".to_string(),
        "certifications=ISO 9001; SOC 2".to_string(),
        "--set".to_string(),
        "legalDisclaimer=Confidential draft for review".to_string(),
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
    assert_eq!(report["profile"]["companyLegalName"], "Acme Advisory LLC");
    assert_eq!(report["profile"]["companyRegistrationNumber"], "DE-123456");
    assert_eq!(report["profile"]["taxIdentifier"], "VAT-999");
    assert_eq!(report["profile"]["dunsNumber"], "12-345-6789");
    assert_eq!(
        report["profile"]["credentialsSummary"],
        "Approved supplier with climate analytics and procurement response credentials"
    );
    assert_eq!(
        report["profile"]["legalDisclaimer"],
        "Confidential draft for review"
    );
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
    assert!(markdown.message.contains("Acme Advisory LLC"));
    assert!(markdown.message.contains("VAT-999"));
    assert!(markdown.message.contains("ISO 9001; SOC 2"));

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
    assert!(placeholders
        .message
        .contains("credentialsSummary: Approved supplier"));

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
    assert!(filled_snippet.message.contains("DE-123456"));
    assert!(filled_snippet.message.contains("VAT-999"));
    assert!(filled_snippet.message.contains("United States"));
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
    assert!(fields_report["fields"]
        .as_array()
        .expect("profile fields")
        .iter()
        .any(|field| field["field"] == "credentialsSummary"
            && field["aliases"]
                .as_array()
                .expect("aliases")
                .contains(&serde_json::json!("credentials"))));

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

    let legal_single = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "profile".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--get".to_string(),
        "legalName".to_string(),
    ])
    .expect("legal profile value");
    assert_eq!(legal_single.message, "Acme Advisory LLC");

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
fn ned_cli_manages_workspace_export_profiles_and_applies_them() {
    let workspace = temp_workspace_path("export-profiles");
    fs::create_dir_all(&workspace).expect("workspace");
    let source = temp_markdown_path("profile-export-source");
    fs::write(
        &source,
        "---\ntitle: Profile Export\n---\n\n# Profile Export\n\nA reusable delivery profile test.",
    )
    .expect("write source");
    let output_dir = workspace.join("exports");

    let saved = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "export-profiles".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--save".to_string(),
        "client-html".to_string(),
        "--name".to_string(),
        "Client HTML".to_string(),
        "--target".to_string(),
        "html".to_string(),
        "--layout-preset".to_string(),
        "business".to_string(),
        "--no-comments".to_string(),
        "--brand".to_string(),
        "name=Acme".to_string(),
        "--brand".to_string(),
        "color=#123456".to_string(),
        "--citation-style".to_string(),
        "apa".to_string(),
        "--json".to_string(),
    ])
    .expect("save export profile");
    let saved_report: serde_json::Value =
        serde_json::from_str(&saved.message).expect("save profile json");
    assert_eq!(saved_report["schema"], "neditor.ned-export-profiles.v1");
    assert_eq!(
        saved_report["library"]["activeExportProfileId"],
        "client-html"
    );
    assert!(workspace
        .join(".neditor")
        .join("export-profiles.json")
        .is_file());

    let ids = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "export-profiles".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--ids-only".to_string(),
    ])
    .expect("ids only");
    assert_eq!(ids.message.trim(), "client-html");

    let markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "delivery-profiles".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--markdown".to_string(),
        "client-html".to_string(),
    ])
    .expect("profile markdown");
    assert!(markdown.message.contains("# Export Profile: Client HTML"));
    assert!(markdown
        .message
        .contains("\"defaultCitationStyle\": \"apa\""));
    assert!(markdown.message.contains("\"name\": \"Acme\""));

    let dry_run = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "export-profiles".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--apply".to_string(),
        "client-html".to_string(),
        "--document".to_string(),
        source.to_string_lossy().to_string(),
        "--output-dir".to_string(),
        output_dir.to_string_lossy().to_string(),
        "--dry-run".to_string(),
        "--json".to_string(),
    ])
    .expect("profile dry-run apply");
    let dry_report: serde_json::Value =
        serde_json::from_str(&dry_run.message).expect("dry profile json");
    assert_eq!(dry_report["export"]["dryRun"], true);
    assert_eq!(dry_report["export"]["target"], "html");
    assert_eq!(
        dry_report["export"]["options"]["defaultBrandProfile"]["color"],
        "#123456"
    );
    assert!(!output_dir.exists());

    let applied = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "export-profiles".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--apply".to_string(),
        "client-html".to_string(),
        "--document".to_string(),
        source.to_string_lossy().to_string(),
        "--output-dir".to_string(),
        output_dir.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("profile apply export");
    let applied_report: serde_json::Value =
        serde_json::from_str(&applied.message).expect("applied profile json");
    let output_path = applied_report["export"]["outputPath"]
        .as_str()
        .expect("output path");
    assert!(std::path::Path::new(output_path).is_file());
    assert!(output_path.ends_with(".html"));
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
            "5. Failure to submit the bid bond certificate will be rejected as non-responsive.",
            "Evaluation criteria: technical merit 40 points, price 30 points, experience 30 points.",
            "Annex A - Declaration of Undertaking must be signed and included.",
            "| Role | Minimum Requirements | Points |",
            "| --- | --- | ---: |",
            "| Software Architect | 5+ years, Bachelor's degree, cloud integration experience | 20 points |",
            "| Legal/IPR Expert | Intellectual property review experience and bilingual EN/FR workshop support | Mandatory |",
            "Placeholder text such as TBD or to be confirmed must be resolved before submission.",
        ]
        .join("\n"),
    )
    .expect("write rfp");
    let response = temp_markdown_path("customer-support-rfp-response");
    let matrix = temp_markdown_path("customer-support-rfp-matrix");
    let checklist = temp_markdown_path("customer-support-rfp-checklist");
    let outline = temp_markdown_path("customer-support-rfp-outline");
    let coverage = temp_markdown_path("customer-support-rfp-coverage");
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
        "--checklist-output".to_string(),
        checklist.to_string_lossy().to_string(),
        "--outline-output".to_string(),
        outline.to_string_lossy().to_string(),
        "--coverage-output".to_string(),
        coverage.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("rfp response json");
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("rfp json");
    assert_eq!(report["schema"], "neditor.ned-rfp-response.v1");
    assert_eq!(report["profileApplied"], true);
    assert!(report["profilePath"]
        .as_str()
        .expect("profile path")
        .ends_with(".neditor/business-profile.json"));
    assert_eq!(report["analysis"]["source"]["kind"], "markdown");
    assert_eq!(
        report["analysis"]["requirements"]
            .as_array()
            .expect("requirements")
            .len(),
        10
    );
    assert!(report["analysis"]["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .any(|item| item["text"]
            .as_str()
            .is_some_and(|value| value.contains("bid bond certificate"))
            && item["requirementType"] == "MANDATORY"
            && item["disqualificationRisk"] == true
            && item["confidence"] == "high"));
    assert!(report["analysis"]["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .any(|item| item["text"]
            .as_str()
            .is_some_and(|value| value.contains("Software Architect")
                && value.contains("5+ years")
                && value.contains("20 points"))
            && item["requirementType"] == "SCORED"));
    assert!(report["analysis"]["requirements"]
        .as_array()
        .expect("requirements")
        .iter()
        .any(|item| item["text"]
            .as_str()
            .is_some_and(|value| value.contains("Legal/IPR Expert")
                && value.contains("bilingual EN/FR")
                && value.contains("Mandatory"))
            && item["requirementType"] == "MANDATORY"));
    assert!(report["analysis"]["criticalDisqualifiers"]
        .as_array()
        .expect("critical disqualifiers")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("bid bond certificate"))));
    assert!(report["analysis"]["complianceRows"]
        .as_array()
        .expect("compliance rows")
        .iter()
        .any(|item| item["disqualificationRisk"] == true
            && item["requirementType"] == "MANDATORY"
            && item["requirement"]
                .as_str()
                .is_some_and(|value| value.contains("bid bond certificate"))));
    assert!(report["analysis"]["complianceChecklist"]
        .as_array()
        .expect("compliance checklist")
        .iter()
        .any(|item| item["section"] == "Critical disqualification traps"
            && item["risk"] == "critical"
            && item["requirement"]
                .as_str()
                .is_some_and(|value| value.contains("bid bond certificate"))));
    assert!(report["analysis"]["complianceChecklist"]
        .as_array()
        .expect("compliance checklist")
        .iter()
        .any(|item| item["section"] == "Scored criteria and win themes"
            && item["requirement"]
                .as_str()
                .is_some_and(|value| value.contains("technical merit"))));
    assert!(report["analysis"]["complianceChecklist"]
        .as_array()
        .expect("compliance checklist")
        .iter()
        .any(
            |item| item["section"] == "Document checklist - attachments required"
                && item["requirement"]
                    .as_str()
                    .is_some_and(|value| value.contains("insurance certificate"))
        ));
    assert!(report["analysis"]["complianceChecklist"]
        .as_array()
        .expect("compliance checklist")
        .iter()
        .any(|item| item["section"] == "Annex references"
            && item["reference"]
                .as_str()
                .is_some_and(|value| value.contains("Source line"))));
    assert!(report["analysis"]["scoringWeights"]
        .as_array()
        .expect("scoring weights")
        .iter()
        .any(|item| item["criterion"]
            .as_str()
            .is_some_and(|value| value.contains("technical merit"))
            && item["weight"] == 40
            && item["unit"] == "points"));
    assert!(report["analysis"]["annexReferences"]
        .as_array()
        .expect("annex references")
        .iter()
        .any(|item| item["annex"] == "Annex A"
            && item["requirement"]
                .as_str()
                .is_some_and(|value| value.contains("Declaration of Undertaking"))));
    assert!(report["analysis"]["bilingualRequirements"]
        .as_array()
        .expect("bilingual requirements")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("bilingual EN/FR"))));
    assert!(report["analysis"]["placeholderRisks"]
        .as_array()
        .expect("placeholder risks")
        .iter()
        .any(|item| item.as_str().is_some_and(|value| value.contains("TBD"))));
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
    assert!(report["complianceChecklistMarkdown"]
        .as_str()
        .expect("checklist markdown")
        .contains("## Compliance Checklist"));
    assert!(report["outputs"]["checklist"]
        .as_str()
        .expect("checklist output")
        .ends_with(".md"));
    assert!(report["outputs"]["outline"]
        .as_str()
        .expect("outline output")
        .ends_with(".md"));
    assert!(report["outputs"]["coverage"]
        .as_str()
        .expect("coverage output")
        .ends_with(".md"));
    assert!(report["coverageValidatorMarkdown"]
        .as_str()
        .expect("coverage markdown")
        .contains("## RFP Requirement Coverage Validator"));
    assert!(report["proposalOutlineMarkdown"]
        .as_str()
        .expect("outline markdown")
        .contains("## Proposal Outline"));
    assert!(
        report["proposalOutlineMarkdown"]
            .as_str()
            .expect("outline markdown")
            .find("## Compliance Checklist")
            .expect("outline checklist")
            < report["proposalOutlineMarkdown"]
                .as_str()
                .expect("outline markdown")
                .find("## Proposal Planning Prompt")
                .expect("outline planning prompt")
    );
    let response_text = fs::read_to_string(&response).expect("response markdown");
    assert!(response_text.contains("## Compliance Checklist"));
    assert!(response_text.contains("Scored criteria and win themes"));
    assert!(response_text.contains("Document checklist - attachments required"));
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
    assert!(response_text.contains("### 1. RFP Metadata"));
    assert!(response_text.contains("### 2. Scoring Scheme and Page Allocation"));
    assert!(response_text.contains("| Criterion | Weight | Sub-criterion | Sub-weight |"));
    assert!(response_text.contains("### 3. Mandatory Pass/Fail Gates"));
    assert!(response_text.contains("### 4. Terms of Reference Map"));
    assert!(response_text.contains("#### 12. Required Annexes"));
    assert!(response_text.contains("### Critical Disqualifiers Checklist"));
    assert!(response_text.contains("### Technical Methodology Draft"));
    assert!(response_text.contains("### Sustainability and Transition Draft"));
    assert!(response_text.contains("### Risk, QA, Validation, and KPI Draft"));
    assert!(response_text.contains("## Requirement Response Drafts"));
    assert!(response_text.contains("Win theme: reduce implementation risk."));
    assert!(response_text.contains("SOC 2 security controls"));
    assert!(response_text.contains("bid bond certificate"));
    assert!(response_text.contains("### Scoring Weights"));
    assert!(response_text.contains("technical merit: 40 points"));
    assert!(response_text.contains("### Annex References"));
    assert!(response_text.contains("Annex A"));
    assert!(response_text.contains("### Bilingual and Language Obligations"));
    assert!(response_text.contains("### Placeholder and Readiness Traps"));
    assert!(response_text.contains("Software Architect"));
    assert!(response_text.contains("Legal/IPR Expert"));
    assert!(response_text.contains("Acme Advisory"));
    assert!(response_text.contains("Globex"));
    assert!(response_text
        .contains("<!-- ai-assisted: status=needs-review | source=NEditor ned RFP Response"));
    let matrix_text = fs::read_to_string(&matrix).expect("matrix markdown");
    assert!(matrix_text.contains("| ID | Requirement | Category | Compliance status |"));
    assert!(matrix_text.contains("RFP-REQ-001"));
    let checklist_text = fs::read_to_string(&checklist).expect("checklist markdown");
    assert!(checklist_text.contains("## Compliance Checklist"));
    assert!(checklist_text.contains("Scored criteria and win themes"));
    assert!(checklist_text.contains("Document checklist - attachments required"));
    let outline_text = fs::read_to_string(&outline).expect("outline markdown");
    assert!(outline_text.contains("## Compliance Checklist"));
    assert!(outline_text.contains("## Proposal Planning Prompt"));
    assert!(outline_text.contains("## Proposal Outline"));
    assert!(
        outline_text
            .find("## Compliance Checklist")
            .expect("outline checklist")
            < outline_text
                .find("## Proposal Planning Prompt")
                .expect("outline planning")
    );
    assert!(
        outline_text
            .find("## Proposal Planning Prompt")
            .expect("outline planning")
            < outline_text
                .find("## Proposal Outline")
                .expect("outline body")
    );
    assert!(outline_text.contains("### 2. Scoring Scheme and Page Allocation"));
    assert!(outline_text.contains("### 3. Mandatory Pass/Fail Gates"));
    let coverage_text = fs::read_to_string(&coverage).expect("coverage markdown");
    assert!(coverage_text.contains("## RFP Requirement Coverage Validator"));
    assert!(coverage_text.contains("Coverage status: needs-evidence-review"));
    assert!(coverage_text.contains("### Mandatory and Disqualification Coverage"));
    assert!(coverage_text.contains("bid bond certificate"));
    assert!(coverage_text.contains("### Attachment and Annex Coverage"));
    assert!(coverage_text.contains("Annex A"));
    assert!(coverage_text.contains("### Language and Placeholder Coverage"));
    assert!(coverage_text.contains("bilingual EN/FR"));
    assert!(coverage_text.contains("TBD"));
    assert!(coverage_text.contains("### Verification Checklist"));

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

    let stdin_checklist = crate::cli::run_cli_with_args_and_stdin(
        &[
            "ned".to_string(),
            "analyze-rfp".to_string(),
            "-".to_string(),
            "--checklist".to_string(),
        ],
        Some("Failure to submit the signed declaration will be rejected."),
    )
    .expect("stdin rfp checklist");
    assert!(stdin_checklist.message.contains("## Compliance Checklist"));
    assert!(stdin_checklist
        .message
        .contains("Critical Disqualification Traps"));
    assert!(stdin_checklist.message.contains("signed declaration"));

    let stdin_outline = crate::cli::run_cli_with_args_and_stdin(
        &[
            "ned".to_string(),
            "analyze-rfp".to_string(),
            "-".to_string(),
            "--outline".to_string(),
        ],
        Some("Evaluation criteria: methodology 60%, price 40%. Vendor must submit Annex B signed declaration."),
    )
    .expect("stdin rfp outline");
    assert!(stdin_outline.message.contains("## Compliance Checklist"));
    assert!(stdin_outline
        .message
        .contains("## Proposal Planning Prompt"));
    assert!(stdin_outline.message.contains("## Proposal Outline"));
    assert!(stdin_outline.message.contains("methodology"));
    assert!(stdin_outline.message.contains("Annex B"));

    let stdin_coverage = crate::cli::run_cli_with_args_and_stdin(
        &[
            "ned".to_string(),
            "analyze-rfp".to_string(),
            "-".to_string(),
            "--coverage".to_string(),
        ],
        Some("Vendor must submit Annex B signed declaration. Failure to submit the bid bond will be rejected."),
    )
    .expect("stdin rfp coverage");
    assert!(stdin_coverage
        .message
        .contains("## RFP Requirement Coverage Validator"));
    assert!(stdin_coverage
        .message
        .contains("Mandatory and Disqualification Coverage"));
    assert!(stdin_coverage.message.contains("Annex B"));
    assert!(stdin_coverage.message.contains("bid bond"));
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
        if template == "rfq" {
            assert!(markdown.contains("## Quotation Summary"));
            assert!(markdown.contains("## Buyer Requirements"));
            assert!(markdown.contains("## Inclusions And Exclusions"));
        }
        if template == "tender" {
            assert!(markdown.contains("## Bid Summary"));
            assert!(markdown.contains("## Mandatory Submission Checklist"));
            assert!(markdown.contains("## Technical Methodology"));
        }
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
fn ned_cli_manages_citation_source_library_from_terminal() {
    let root = temp_workspace_path("sources");
    fs::create_dir_all(&root).expect("create sources workspace");
    let document = root.join("proposal.md");
    fs::write(
        &document,
        "# Proposal\n\nClimate procurement controls need evidence.\n",
    )
    .expect("write document");
    let source_dir = root.join("proposal.neditor-sources");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source_path = source_dir.join("climate.html");
    let source_body =
        "<html><body>Climate procurement controls evidence and audit timeline.</body></html>";
    fs::write(&source_path, source_body).expect("write saved source");
    fs::write(
        source_dir.join("sources.json"),
        serde_json::to_string_pretty(&serde_json::json!([
            {
                "citation_key": "climate",
                "title": "Climate Procurement Evidence",
                "url": "https://agency.gov/climate-procurement.html",
                "snippet": "Climate procurement controls evidence for source review.",
                "source": "SearXNG",
                "path": source_path.to_string_lossy(),
                "relative_path": "proposal.neditor-sources/climate.html",
                "sha256": "placeholder-hash",
                "bytes": source_body.len(),
                "downloaded_at": "2026-05-30T08:00:00+03:00",
                "media_type": "text/html",
                "fit_score": 91,
                "fit_label": "strong",
                "fit_reasons": ["query term in title", "saved source document"]
            }
        ]))
        .expect("source manifest json"),
    )
    .expect("write source manifest");

    let list = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "sources".to_string(),
        "--document".to_string(),
        document.to_string_lossy().to_string(),
    ])
    .expect("list source library");
    assert_eq!(list.exit_code, 0);
    assert!(list.message.contains("NEditor citation source library"));
    assert!(list.message.contains("@climate"));
    assert!(list.message.contains("Climate Procurement Evidence"));
    assert!(list.message.contains("Status: modified"));

    let audit_path = root.join("source-audit.md");
    let audit = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "citation-sources".to_string(),
        "--document".to_string(),
        document.to_string_lossy().to_string(),
        "--audit".to_string(),
        "--output".to_string(),
        audit_path.to_string_lossy().to_string(),
    ])
    .expect("write source audit");
    assert_eq!(audit.exit_code, 0);
    assert!(audit
        .message
        .contains("Wrote citation source library output"));
    let audit_markdown = fs::read_to_string(&audit_path).expect("audit markdown");
    assert!(audit_markdown.contains("## Source Library Audit"));
    assert!(audit_markdown.contains("@climate"));
    assert!(audit_markdown.contains("current sha256"));
    assert!(audit_markdown.contains("saved source document"));

    let search = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "sources".to_string(),
        "--document".to_string(),
        document.to_string_lossy().to_string(),
        "--query".to_string(),
        "climate procurement evidence".to_string(),
        "--provider".to_string(),
        "local-library".to_string(),
        "--json".to_string(),
    ])
    .expect("local source search");
    assert_eq!(search.exit_code, 0);
    let search_report: serde_json::Value =
        serde_json::from_str(&search.message).expect("source search json");
    assert_eq!(
        search_report["schema"],
        "neditor.ned-citation-source-search.v1"
    );
    assert_eq!(search_report["search"]["provider"], "local-library");
    assert_eq!(
        search_report["search"]["results"][0]["title"],
        "Climate Procurement Evidence"
    );
    assert!(search_report["search"]["results"][0]["snippet"]
        .as_str()
        .expect("snippet")
        .contains("Match score"));

    let missing_document = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "sources".to_string(),
        "--audit".to_string(),
    ])
    .expect_err("missing document should fail");
    assert!(missing_document.contains("--document is required"));
}

#[test]
fn ned_cli_creates_deep_research_dossiers_from_terminal() {
    let root = temp_workspace_path("deep-research");
    fs::create_dir_all(&root).expect("create deep research workspace");
    let document = root.join("proposal.md");
    fs::write(
        &document,
        "# Proposal\n\nClimate procurement controls need evidence.\n",
    )
    .expect("write document");
    let source_dir = root.join("proposal.neditor-sources");
    fs::create_dir_all(&source_dir).expect("create source dir");
    let source_path = source_dir.join("climate.html");
    let source_body =
        "<html><body>Climate procurement controls evidence and audit timeline.</body></html>";
    fs::write(&source_path, source_body).expect("write saved source");
    fs::write(
        source_dir.join("sources.json"),
        serde_json::to_string_pretty(&serde_json::json!([
            {
                "citation_key": "climate-procurement-evidence",
                "title": "Climate Procurement Evidence",
                "url": "https://agency.gov/climate-procurement.html",
                "snippet": "Climate procurement controls evidence for source review.",
                "source": "SearXNG",
                "path": source_path.to_string_lossy(),
                "relative_path": "proposal.neditor-sources/climate.html",
                "sha256": "placeholder-hash",
                "bytes": source_body.len(),
                "downloaded_at": "2026-05-30T08:00:00+03:00",
                "media_type": "text/html",
                "fit_score": 91,
                "fit_label": "strong",
                "fit_reasons": ["query term in title", "saved source document"]
            }
        ]))
        .expect("source manifest json"),
    )
    .expect("write source manifest");

    let output = root.join("deep-research.md");
    let outcome = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "deep-research".to_string(),
        "--topic".to_string(),
        "climate procurement evidence".to_string(),
        "--document".to_string(),
        document.to_string_lossy().to_string(),
        "--provider".to_string(),
        "local-library".to_string(),
        "--pages".to_string(),
        "12".to_string(),
        "--iterations".to_string(),
        "2".to_string(),
        "--results".to_string(),
        "2".to_string(),
        "--output".to_string(),
        output.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("deep research json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value =
        serde_json::from_str(&outcome.message).expect("deep research json");
    assert_eq!(report["schema"], "neditor.ned-deep-research.v1");
    assert_eq!(report["settings"]["topic"], "climate procurement evidence");
    assert_eq!(report["settings"]["searchProvider"], "local-library");
    assert_eq!(report["settings"]["targetPages"], 12);
    assert!(report["settings"]["supportedSearchProviders"]
        .as_array()
        .expect("supported search providers")
        .iter()
        .any(|provider| provider["provider"] == "searxng"
            && provider["requiredSetup"]
                .as_str()
                .expect("searxng setup")
                .contains("--searxng-url")));
    assert_eq!(
        report["sourceLibrary"]["sources"]
            .as_array()
            .expect("source library sources")
            .len(),
        1
    );
    assert!(report["auditPacket"]
        .as_str()
        .expect("audit packet")
        .contains("## Research Audit Packet"));
    assert!(report["auditPacket"]
        .as_str()
        .expect("audit packet")
        .contains("### Search Provider Choices"));
    assert!(report["auditPacket"]
        .as_str()
        .expect("audit packet")
        .contains("Source vault directory"));
    assert_eq!(
        report["iterations"].as_array().expect("iterations").len(),
        2
    );
    assert_eq!(report["iterations"][0]["provider"], "local-library");
    assert!(report["iterations"][0]["results"]
        .as_array()
        .expect("results")
        .iter()
        .any(|result| result["title"] == "Climate Procurement Evidence"
            && result["citationKey"] == "climate-procurement-evidence"
            && result["fitScore"].as_u64().expect("fit score") > 0));
    assert!(report["markdown"]
        .as_str()
        .expect("markdown")
        .contains("provider: NEditor Deep Research CLI"));

    let markdown = fs::read_to_string(&output).expect("deep research markdown");
    assert!(markdown.contains("# climate procurement evidence"));
    assert!(markdown.contains("## Report Length Plan"));
    assert!(markdown.contains("| Evidence-Backed Findings |"));
    assert!(markdown.contains("## Draft Section Queue"));
    assert!(markdown.contains("### Evidence-Backed Findings"));
    assert!(markdown.contains("Evidence to inspect first: @climate-procurement-evidence"));
    assert!(markdown.contains("## Source Quality Review"));
    assert!(markdown.contains("## Source Citation Index"));
    assert!(markdown.contains("## Deep Research Evidence Log"));
    assert!(markdown.contains("## Source Vault State"));
    assert!(markdown.contains("## Research Audit Packet"));
    assert!(markdown.contains("### Search Provider Choices"));
    assert!(markdown.contains("proposal.neditor-sources"));
    assert!(markdown.contains("@climate-procurement-evidence"));
    assert!(markdown.contains("```bibliography"));

    let missing_topic = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "research-report".to_string(),
        "--provider".to_string(),
        "local-library".to_string(),
    ])
    .expect_err("missing topic should fail");
    assert!(missing_topic.contains("--topic is required"));
}

#[test]
fn ned_cli_generates_quality_review_recommendations() {
    let source = temp_markdown_path("quality");
    let long_paragraph = (0..130).map(|_| "review").collect::<Vec<_>>().join(" ");
    fs::write(
        &source,
        format!(
            "# Draft\n\nThis seamless world-class section needs evidence [@missing].\n\n{{{{clientName}}}}\n\n{long_paragraph}\n\n| A | B | C | D | E | F | G |\n| --- | --- | --- | --- | --- | --- | --- |\n| 1 | 2 | 3 | 4 | 5 | 6 | 7 |\n\n<!-- comment: unresolved | author: reviewer | Resolve before release -->\n<!-- ai_assisted: status=needs-review | source=Docs Live | prompt=Draft section -->\n"
        ),
    )
    .expect("write quality source");

    let json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "quality".to_string(),
        source.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("quality json");
    assert_eq!(json.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&json.message).expect("quality json");
    assert_eq!(report["schema"], "neditor.ned-quality.v1");
    assert_eq!(report["status"], "needs-review");
    assert!(report["summary"]["risk"].as_u64().expect("risk count") >= 4);
    let ids = report["recommendations"]
        .as_array()
        .expect("recommendations")
        .iter()
        .map(|item| item["id"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    for expected in [
        "placeholders",
        "citation-evidence",
        "review-comments",
        "ai-provenance",
        "readability",
        "humanization",
        "layout-wide-tables",
    ] {
        assert!(
            ids.contains(&expected),
            "missing quality finding {expected}"
        );
    }

    let output = source.with_file_name("quality-review.md");
    let markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "qa".to_string(),
        source.to_string_lossy().to_string(),
        "--markdown".to_string(),
        "--output".to_string(),
        output.to_string_lossy().to_string(),
    ])
    .expect("quality markdown output");
    assert_eq!(markdown.exit_code, 0);
    assert!(markdown.message.contains("Wrote quality review"));
    let quality_markdown = fs::read_to_string(&output).expect("quality markdown");
    assert!(quality_markdown.contains("# NEditor Quality Review"));
    assert!(quality_markdown.contains("Citation evidence"));
    assert!(quality_markdown.contains("Review Gate"));

    let strict = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "review".to_string(),
        source.to_string_lossy().to_string(),
        "--strict".to_string(),
    ])
    .expect("quality strict");
    assert_eq!(strict.exit_code, 1);
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
            && template["label"] == "Tender Response"
            && template["category"] == "Procurement"
            && template["summary"]
                .as_str()
                .is_some_and(|summary| summary.contains("tender response"))));
    assert!(template_report["templateDetails"]
        .as_array()
        .expect("template details")
        .iter()
        .any(|template| template["id"] == "rfq"
            && template["label"] == "RFQ Response"
            && template["summary"]
                .as_str()
                .is_some_and(|summary| summary.contains("Quotation response"))));
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

    let report_preview = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--markdown".to_string(),
        "report".to_string(),
        "--title".to_string(),
        "Weekly Operating Report".to_string(),
    ])
    .expect("template markdown preview");
    assert!(report_preview
        .message
        .contains("title: Weekly Operating Report"));
    assert!(report_preview
        .message
        .contains("documentType: business-report"));
    assert!(report_preview.message.contains("## Analysis"));

    let rfp_response_preview_json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--markdown".to_string(),
        "rfp-response".to_string(),
        "--title".to_string(),
        "Buyer Response".to_string(),
        "--json".to_string(),
    ])
    .expect("template markdown preview json");
    let preview_report: serde_json::Value =
        serde_json::from_str(&rfp_response_preview_json.message).expect("template preview json");
    assert_eq!(preview_report["schema"], "neditor.ned-template.v1");
    assert_eq!(preview_report["template"], "rfp-response");
    assert!(preview_report["markdown"]
        .as_str()
        .expect("preview markdown")
        .contains("## Compliance Matrix"));

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

    let filled_template_preview = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "templates".to_string(),
        "--workspace".to_string(),
        snippet_workspace.to_string_lossy().to_string(),
        "--markdown".to_string(),
        "capability-statement".to_string(),
        "--title".to_string(),
        "Acme Capability".to_string(),
        "--fill-profile".to_string(),
        "--json".to_string(),
    ])
    .expect("filled template preview json");
    let filled_template_report: serde_json::Value =
        serde_json::from_str(&filled_template_preview.message).expect("filled template json");
    assert_eq!(filled_template_report["schema"], "neditor.ned-template.v1");
    assert_eq!(filled_template_report["profileApplied"], true);
    assert!(filled_template_report["markdown"]
        .as_str()
        .expect("filled template markdown")
        .contains("**Prepared by:** Jane Doe"));
    assert!(filled_template_report["markdown"]
        .as_str()
        .expect("filled template markdown")
        .contains("**Email:** jane@example.com"));
    assert!(filled_template_report["markdown"]
        .as_str()
        .expect("filled template markdown")
        .contains("**Website:** https://acme.example"));
    assert!(filled_template_report["rawMarkdown"]
        .as_str()
        .expect("raw template markdown")
        .contains("{{owner}}"));

    let filled_new_path = snippet_workspace.join("case-study.md");
    let filled_new = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "new".to_string(),
        filled_new_path.to_string_lossy().to_string(),
        "--template".to_string(),
        "case-study".to_string(),
        "--title".to_string(),
        "Globex Case Study".to_string(),
        "--workspace".to_string(),
        snippet_workspace.to_string_lossy().to_string(),
        "--fill-profile".to_string(),
        "--json".to_string(),
    ])
    .expect("filled new document json");
    let filled_new_report: serde_json::Value =
        serde_json::from_str(&filled_new.message).expect("filled new json");
    assert_eq!(filled_new_report["schema"], "neditor.ned-new.v1");
    assert_eq!(filled_new_report["profileApplied"], true);
    let filled_new_markdown = fs::read_to_string(&filled_new_path).expect("filled new markdown");
    assert!(filled_new_markdown.contains("| Industry | Consulting |"));
    assert!(filled_new_markdown.contains("| Customer | {{customer_name}} |"));
    assert!(filled_new_markdown.contains("| {{phase}} | {{action}} | {{evidence}} | Jane Doe |"));
    assert!(filled_new_markdown.contains("| Review status | {{approval_status}} |"));

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
    assert_eq!(outline_report["count"], 50);
    assert!(outline_report["outlines"]
        .as_array()
        .expect("outlines")
        .contains(&serde_json::json!("outline-rfp-technical-proposal")));
    for id in [
        "business-blank",
        "business-rfp",
        "business-rfq",
        "business-tender",
        "business-sow",
        "business-capability-statement",
        "business-case-study",
        "business-report",
        "business-textbook",
        "business-lesson-content",
        "business-executive-brief",
        "business-grant-application",
        "business-standard-operating-procedure",
        "business-product-requirements-document",
        "business-project-charter",
        "business-quarterly-business-review",
        "business-due-diligence-memo",
        "business-incident-postmortem",
        "business-meeting-decision-pack",
        "business-market-research-report",
        "business-contract-review-brief",
        "outline-contract-review-brief",
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
        .any(|outline| outline["id"] == "outline-rfp-technical-proposal"
            && outline["category"] == "Procurement"
            && outline["outline"]
                .as_array()
                .expect("outline")
                .contains(&serde_json::json!("Compliance Checklist"))));
    assert!(outline_report["outlineDetails"]
        .as_array()
        .expect("outline details")
        .iter()
        .any(|outline| outline["id"] == "business-rfp"
            && outline["category"] == "Procurement"
            && outline["outline"]
                .as_array()
                .expect("outline")
                .contains(&serde_json::json!("Required Response Matrix"))));

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
    assert_eq!(filtered_outlines.message, "outline-research-report");

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
    assert_eq!(learning_outlines.message, "business-lesson-content");

    let case_study_outlines = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--category".to_string(),
        "Business Development".to_string(),
        "--query".to_string(),
        "customer challenge".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("case study outline ids");
    assert_eq!(case_study_outlines.message, "business-case-study");

    let buyer_rfp_outlines = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--category".to_string(),
        "Procurement".to_string(),
        "--query".to_string(),
        "buyer-side".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("buyer rfp outline ids");
    assert_eq!(buyer_rfp_outlines.message, "business-rfp");

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
        .any(|outline| outline["id"] == "outline-contract-review-brief"
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

    let app_prefixed_outline = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--markdown".to_string(),
        "business-report".to_string(),
        "--json".to_string(),
    ])
    .expect("app-prefixed outline markdown");
    let app_prefixed_report: serde_json::Value =
        serde_json::from_str(&app_prefixed_outline.message).expect("app-prefixed outline json");
    assert_eq!(app_prefixed_report["outline"], "business-report");
    assert!(app_prefixed_report["markdown"]
        .as_str()
        .expect("app-prefixed markdown")
        .contains("- Analysis"));

    let specialist_prefixed_outline = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "outlines".to_string(),
        "--markdown".to_string(),
        "outline-rfp-technical-proposal".to_string(),
    ])
    .expect("specialist-prefixed outline markdown");
    assert!(specialist_prefixed_outline
        .message
        .contains("- Compliance Checklist"));

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
fn ned_cli_manages_latex_template_libraries() {
    let builtin = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "latex-templates".to_string(),
        "--json".to_string(),
    ])
    .expect("latex template json");
    let builtin_report: serde_json::Value =
        serde_json::from_str(&builtin.message).expect("latex template report");
    assert_eq!(builtin_report["schema"], "neditor.ned-latex-templates.v1");
    assert_eq!(builtin_report["count"], 8);
    assert!(builtin_report["templates"]
        .as_array()
        .expect("latex templates")
        .contains(&serde_json::json!("rfp-response")));

    let workspace = temp_workspace_path("latex-template-library");
    let saved = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "latex-templates".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--save".to_string(),
        "custom-latex-board".to_string(),
        "--name".to_string(),
        "Board House Style".to_string(),
        "--summary".to_string(),
        "Tight board-paper LaTeX profile.".to_string(),
        "--document-class".to_string(),
        "memoir".to_string(),
        "--class-options".to_string(),
        "12pt,oneside".to_string(),
        "--package".to_string(),
        "\\usepackage{booktabs}".to_string(),
        "--package".to_string(),
        "\\usepackage{longtable}".to_string(),
        "--geometry".to_string(),
        "margin=0.75in".to_string(),
        "--hypersetup".to_string(),
        "colorlinks=true,linkcolor=black,urlcolor=blue".to_string(),
        "--header".to_string(),
        "\\pagestyle{plain}".to_string(),
        "--chapter-style".to_string(),
        "--best-for".to_string(),
        "board packs".to_string(),
        "--source-path".to_string(),
        "templates/board.tex".to_string(),
        "--json".to_string(),
    ])
    .expect("save latex template");
    let saved_report: serde_json::Value =
        serde_json::from_str(&saved.message).expect("saved latex template json");
    assert_eq!(saved_report["schema"], "neditor.ned-latex-template-save.v1");
    assert_eq!(saved_report["template"]["id"], "custom-latex-board");
    assert_eq!(saved_report["template"]["documentClass"], "memoir");
    assert_eq!(saved_report["template"]["chapterStyle"], true);
    assert!(workspace
        .join(".neditor")
        .join("latex-templates.json")
        .is_file());

    let filtered = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "latex-templates".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--query".to_string(),
        "board".to_string(),
        "--json".to_string(),
    ])
    .expect("filtered latex templates");
    let filtered_report: serde_json::Value =
        serde_json::from_str(&filtered.message).expect("filtered latex template json");
    assert!(filtered_report["templateDetails"]
        .as_array()
        .expect("template details")
        .iter()
        .any(|template| template["id"] == "custom-latex-board"
            && template["source"] == "workspace"
            && template["sourcePath"] == "templates/board.tex"));

    let preamble = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "latex-templates".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--preamble".to_string(),
        "custom-latex-board".to_string(),
    ])
    .expect("latex template preamble");
    assert!(preamble
        .message
        .contains("\\documentclass[12pt,oneside]{memoir}"));
    assert!(preamble.message.contains("\\usepackage{booktabs}"));
    assert!(preamble.message.contains("\\pagestyle{plain}"));

    let export_path = workspace.join("company-latex-templates.json");
    let exported = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "latex-templates".to_string(),
        "--workspace".to_string(),
        workspace.to_string_lossy().to_string(),
        "--export-library".to_string(),
        export_path.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("export latex templates");
    let exported_report: serde_json::Value =
        serde_json::from_str(&exported.message).expect("export latex template json");
    assert_eq!(exported_report["exported"], 1);
    assert!(export_path.is_file());

    let imported_workspace = temp_workspace_path("latex-template-import");
    let imported = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "latex-templates".to_string(),
        "--workspace".to_string(),
        imported_workspace.to_string_lossy().to_string(),
        "--import".to_string(),
        export_path.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("import latex templates");
    let imported_report: serde_json::Value =
        serde_json::from_str(&imported.message).expect("import latex template json");
    assert_eq!(
        imported_report["schema"],
        "neditor.ned-latex-template-import.v1"
    );
    assert_eq!(imported_report["imported"], 1);

    let imported_ids = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "latex-templates".to_string(),
        "--workspace".to_string(),
        imported_workspace.to_string_lossy().to_string(),
        "--query".to_string(),
        "board".to_string(),
        "--ids-only".to_string(),
    ])
    .expect("imported latex ids");
    assert!(imported_ids
        .message
        .lines()
        .any(|id| id == "custom-latex-board"));

    let deleted = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "latex-templates".to_string(),
        "--workspace".to_string(),
        imported_workspace.to_string_lossy().to_string(),
        "--delete".to_string(),
        "custom-latex-board".to_string(),
        "--json".to_string(),
    ])
    .expect("delete latex template");
    let deleted_report: serde_json::Value =
        serde_json::from_str(&deleted.message).expect("delete latex template json");
    assert_eq!(deleted_report["deleted"], true);
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
fn ned_cli_reports_unified_setup_packet() {
    let outcome = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "setup".to_string(),
        "--platform".to_string(),
        "macos".to_string(),
        "--ollama-endpoint".to_string(),
        "https://ollama.example.com/team/api/chat".to_string(),
        "--json".to_string(),
    ])
    .expect("setup json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("setup json");
    assert_eq!(report["schema"], "neditor.ned-setup.v1");
    assert_eq!(report["platform"], "macos");
    assert!(report["configurationCenter"]
        .as_array()
        .expect("configuration center")
        .iter()
        .any(|section| section["id"] == "transforms"
            && section["purpose"]
                .as_str()
                .expect("transform purpose")
                .contains("installer plans")));
    assert!(report["guidedProviderSetup"]
        .as_array()
        .expect("provider setup")
        .iter()
        .any(|provider| provider["id"] == "ollama-local"
            && provider["setupSteps"]
                .as_array()
                .expect("setup steps")
                .iter()
                .any(|step| step
                    .as_str()
                    .is_some_and(|value| value.contains("Confirm endpoint and model")))));
    assert!(report["guidedProviderSetup"]
        .as_array()
        .expect("provider setup")
        .iter()
        .any(|provider| provider["id"] == "codex-cli"));
    assert_eq!(
        report["ollamaModelPicker"]["tagsEndpoint"],
        "https://ollama.example.com/team/api/tags"
    );
    assert!(report["ollamaModelPicker"]["modelSelectionWorkflows"]
        .as_array()
        .expect("model workflows")
        .contains(&serde_json::json!("Deep Research")));
    assert!(report["transformHandlerInstaller"]["coverageComplete"]
        .as_bool()
        .expect("handler coverage"));
    assert!(report["setupChecklist"]
        .as_array()
        .expect("setup checklist")
        .iter()
        .any(|item| item["id"] == "ollama-models"
            && item["evidence"]
                .as_str()
                .expect("ollama evidence")
                .contains("/api/tags")));

    let markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "configurator".to_string(),
        "--platform".to_string(),
        "linux".to_string(),
        "--markdown".to_string(),
    ])
    .expect("setup markdown");
    assert!(markdown.message.contains("# NEditor Setup Packet"));
    assert!(markdown.message.contains("## Configuration Center"));
    assert!(markdown.message.contains("## Guided Provider Setup"));
    assert!(markdown.message.contains("## Ollama Model Picker"));
    assert!(markdown.message.contains("## Transform Handler Installer"));
}

#[test]
fn ned_cli_routes_voice_commands_to_creation_revision_and_read_aloud() {
    let outcome = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "voice".to_string(),
        "--text".to_string(),
        "Create a board memo for the CFO; client: Acme; make section 3 more formal; read selected text aloud".to_string(),
        "--selected-text".to_string(),
        "This section needs work.".to_string(),
        "--json".to_string(),
    ])
    .expect("voice command json");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("voice json");
    assert_eq!(report["schema"], "neditor.ned-voice-command.v1");
    assert_eq!(
        report["documentWizard"]["documentType"],
        "board-decision-memo"
    );
    assert!(report["documentWizard"]["outlineFirst"]
        .as_bool()
        .expect("outline first"));
    assert!(report["documentWizard"]["placeholderSignals"]
        .as_array()
        .expect("placeholder signals")
        .iter()
        .any(|item| item["key"] == "client" && item["value"] == "Acme"));
    for route_id in ["docs-live-wizard", "voice-correction-loop", "read-aloud"] {
        assert!(report["routes"]
            .as_array()
            .expect("routes")
            .iter()
            .any(|route| route["id"] == route_id));
    }
    assert!(report["correctionLoop"]["requestedEdits"]
        .as_array()
        .expect("requested edits")
        .contains(&serde_json::json!("tone-adjustment")));
    assert_eq!(report["readAloud"]["supportsEngines"][2], "supertonic-cli");
    assert!(report["readAloud"]["consentGate"]
        .as_str()
        .expect("consent gate")
        .contains("explicit model download acknowledgement"));
    assert!(!outcome.message.contains("API_KEY"));

    let markdown = crate::cli::run_cli_with_args_and_stdin(
        &[
            "ned".to_string(),
            "dictate".to_string(),
            "-".to_string(),
            "--document-type".to_string(),
            "proposal".to_string(),
            "--markdown".to_string(),
        ],
        Some("Draft a proposal outline, then humanize the executive summary."),
    )
    .expect("voice markdown");
    assert!(markdown.message.contains("# NEditor Voice Command Packet"));
    assert!(markdown.message.contains("## Voice-First Wizard"));
    assert!(markdown.message.contains("## Correction Loop"));
    assert!(markdown.message.contains("proposal"));
}

#[test]
fn ned_cli_reports_accessibility_qa_and_release_dashboard() {
    let a11y = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "accessibility".to_string(),
        "--json".to_string(),
    ])
    .expect("accessibility json");
    assert_eq!(a11y.exit_code, 0);
    let a11y_report: serde_json::Value =
        serde_json::from_str(&a11y.message).expect("accessibility report json");
    assert_eq!(a11y_report["schema"], "neditor.ned-accessibility-qa.v1");
    assert_eq!(a11y_report["screenReaderQaMode"]["enabledInApp"], true);
    assert_eq!(a11y_report["status"], "needs-review");
    assert!(a11y_report["items"]
        .as_array()
        .expect("accessibility items")
        .iter()
        .any(|item| item["id"] == "manual-assistive-tech-signoff"
            && item["status"] == "needs-review"));
    assert!(a11y_report["nextCommands"]
        .as_array()
        .expect("a11y next commands")
        .iter()
        .any(|command| command == "pnpm run check:a11y:manual"));

    let a11y_markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "screen-reader-qa".to_string(),
        "--markdown".to_string(),
    ])
    .expect("accessibility markdown");
    assert!(a11y_markdown
        .message
        .contains("# NEditor Screen-Reader QA Mode"));

    let dashboard = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "release-dashboard".to_string(),
        "--json".to_string(),
    ])
    .expect("release dashboard json");
    assert_eq!(dashboard.exit_code, 0);
    let dashboard_report: serde_json::Value =
        serde_json::from_str(&dashboard.message).expect("release dashboard report json");
    assert_eq!(
        dashboard_report["schema"],
        "neditor.ned-release-dashboard.v1"
    );
    assert_eq!(dashboard_report["productionReady"], false);
    for lane in ["credentialed", "manual", "ready-to-send"] {
        assert!(dashboard_report["showsLanes"]
            .as_array()
            .expect("shown lanes")
            .iter()
            .any(|item| item == lane));
    }
    for item_id in [
        "provider-runtime",
        "google-docs-import",
        "homebrew-signing",
        "ready-to-send",
    ] {
        assert!(dashboard_report["items"]
            .as_array()
            .expect("dashboard items")
            .iter()
            .any(|item| item["id"] == item_id));
    }
    let homebrew_item = dashboard_report["items"]
        .as_array()
        .expect("dashboard items")
        .iter()
        .find(|item| item["id"] == "homebrew-signing")
        .expect("homebrew release dashboard item");
    let homebrew_detail = homebrew_item["detail"]
        .as_str()
        .expect("homebrew dashboard detail");
    assert!(
        homebrew_detail.contains("concrete cask SHA")
            || homebrew_detail.contains("cask, artifact SHA, and materialization are checked"),
        "homebrew dashboard detail should distinguish missing SHA proof from checked local cask/artifact proof: {homebrew_detail}"
    );
    assert!(dashboard_report["nextCommands"]
        .as_array()
        .expect("dashboard next commands")
        .iter()
        .any(|command| command == "ned accessibility --json"));

    let dashboard_markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "release-evidence-dashboard".to_string(),
        "--markdown".to_string(),
    ])
    .expect("release dashboard markdown");
    assert!(dashboard_markdown
        .message
        .contains("# NEditor Release Evidence Dashboard"));
}

#[test]
fn ned_release_dashboard_distinguishes_provider_and_runtime_evidence() {
    let accepted_provider = serde_json::json!({
        "status": "accepted",
        "summary": {
            "acceptedEvidence": 1
        }
    });
    let accepted_runtime = serde_json::json!({
        "status": "accepted",
        "summary": {
            "acceptedEvidence": 1
        }
    });

    let (lane, detail) = crate::cli::release_dashboard_provider_runtime_state_from_reports(
        Some(&accepted_provider),
        None,
    );
    assert_eq!(lane, "credentialed");
    assert!(
        detail.contains("AI provider endpoint evidence is accepted")
            && detail.contains("runtime proof remains pending"),
        "provider accepted/runtime pending detail should be specific: {detail}"
    );

    let (lane, detail) = crate::cli::release_dashboard_provider_runtime_state_from_reports(
        Some(&accepted_provider),
        Some(&accepted_runtime),
    );
    assert_eq!(lane, "complete");
    assert!(
        detail.contains("AI provider endpoint evidence")
            && detail.contains("runtime proof are accepted"),
        "complete provider/runtime detail should mention both evidence types: {detail}"
    );
}

#[test]
fn ned_cli_exposes_final_release_handoff_surfaces() {
    let ai_runtime = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "ai-runtime".to_string(),
        "--profile".to_string(),
        "ollama-local".to_string(),
        "--model".to_string(),
        "llama3.1".to_string(),
        "--json".to_string(),
    ])
    .expect("ai runtime json");
    assert_eq!(ai_runtime.exit_code, 0);
    let ai_report: serde_json::Value =
        serde_json::from_str(&ai_runtime.message).expect("ai runtime json");
    assert_eq!(ai_report["schema"], "neditor.ned-ai-runtime.v1");
    assert_eq!(ai_report["status"], "implementation-ready");
    assert_eq!(ai_report["selectedProfile"]["id"], "ollama-local");
    assert_eq!(ai_report["selectedProfile"]["model"], "llama3.1");
    assert!(ai_report["providerProfiles"]
        .as_array()
        .expect("provider profiles")
        .iter()
        .any(|item| item["id"] == "openai-compatible"));
    assert!(ai_report["providerProfiles"]
        .as_array()
        .expect("provider profiles")
        .iter()
        .any(|item| item["id"] == "claude-code-cli"));
    assert_eq!(
        ai_report["requestPackage"]["responseHandling"],
        "Extract Markdown, wrap as needs-review AI provenance, then run quality and export gates."
    );

    let google_docs = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "google-docs-import".to_string(),
        "--json".to_string(),
    ])
    .expect("google docs handoff json");
    assert_eq!(google_docs.exit_code, 0);
    let google_report: serde_json::Value =
        serde_json::from_str(&google_docs.message).expect("google docs handoff json");
    assert_eq!(
        google_report["schema"],
        "neditor.ned-google-docs-import-handoff.v1"
    );
    assert_eq!(google_report["status"], "handoff-ready");
    assert_eq!(
        google_report["templateSchema"],
        "neditor.google-docs-import-evidence.v1"
    );
    assert!(google_report["importWorkflow"]
        .as_array()
        .expect("import workflow")
        .iter()
        .any(|step| step
            .as_str()
            .unwrap_or("")
            .contains("Read back document text")));

    let homebrew = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "homebrew-release".to_string(),
        "--json".to_string(),
    ])
    .expect("homebrew release json");
    assert_eq!(homebrew.exit_code, 0);
    let homebrew_report: serde_json::Value =
        serde_json::from_str(&homebrew.message).expect("homebrew release json");
    assert_eq!(homebrew_report["schema"], "neditor.ned-homebrew-release.v1");
    assert_eq!(homebrew_report["status"], "release-path-ready");
    assert_eq!(
        homebrew_report["paths"]["template"]["path"],
        "packaging/homebrew/Casks/neditor.rb.template"
    );
    assert!(homebrew_report["releaseWorkflow"]
        .as_array()
        .expect("homebrew workflow")
        .iter()
        .any(|step| step.as_str().unwrap_or("").contains("Sign and notarize")));

    let markdown = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "ai-providers".to_string(),
        "--markdown".to_string(),
    ])
    .expect("ai runtime markdown");
    assert!(markdown
        .message
        .contains("# NEditor Provider-Agnostic AI Runtime"));
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
    let kit_dir = root.join("release-evidence-kit");
    fs::create_dir_all(&kit_dir).expect("create evidence kit");
    let source_commit = current_test_git_head();
    fs::write(
        kit_dir.join("manifest.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "neditor.release-evidence-kit.v1",
            "generatedAt": "2026-05-26T12:01:00.000Z",
            "sourceCommit": source_commit,
            "sourceTreeClean": true,
            "gapWorkItems": [
                {
                    "id": "homebrew-final-cask",
                    "status": "pending-release-cask",
                    "detail": "Set the final Homebrew cask SHA.",
                    "evidence": ".tmp/homebrew/homebrew-packaging-report.json",
                    "runbooks": [
                        { "title": "Homebrew release", "path": "runbooks/homebrew-release.md" }
                    ],
                    "returns": [".tmp/homebrew/external/homebrew-cask.json"],
                    "validatorCommands": ["pnpm run check:homebrew"],
                    "ingestCommand": "pnpm run ingest:evidence -- --source /path/to/return-dir",
                    "finalReadinessCommand": "pnpm run check:release-readiness",
                    "readyToSend": true
                }
            ]
        }))
        .expect("kit manifest json"),
    )
    .expect("write evidence kit");

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

    let action_text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "readiness".to_string(),
        "--report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--action-plan".to_string(),
        "--evidence-kit".to_string(),
        kit_dir.to_string_lossy().to_string(),
    ])
    .expect("readiness action plan text");
    assert_eq!(action_text.exit_code, 0);
    assert!(action_text.message.contains("Action plan:"));
    assert!(action_text
        .message
        .contains("Work items ready to send: 1/1"));
    assert!(action_text.message.contains("runbooks/homebrew-release.md"));
    assert!(action_text
        .message
        .contains(".tmp/homebrew/external/homebrew-cask.json"));

    let json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "release-readiness".to_string(),
        "--report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--evidence-kit".to_string(),
        kit_dir.to_string_lossy().to_string(),
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
    assert_eq!(normalized["actionPlan"]["status"], "ready-to-send");
    assert_eq!(normalized["actionPlan"]["readyToSendCount"], 1);
    assert_eq!(
        normalized["actionPlan"]["workItems"][0]["runbooks"][0]["path"],
        "runbooks/homebrew-release.md"
    );

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
    let spec_work_orders_path = root.join("work-orders.json");
    let release_candidate_dir = root.join("release-candidate");
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
    let spec_work_orders = serde_json::json!({
        "schema": "neditor.spec-completion-work-orders.v1",
        "generatedAt": "2026-05-26T12:26:00.000Z",
        "status": "partial-with-release-risks",
        "matrixPath": "docs/spec-completion-matrix.md",
        "reportPath": spec_path.to_string_lossy(),
        "summary": {
            "total": 2,
            "readyToSend": 2,
            "byClassification": {
                "manual-review": 1,
                "cross-platform-evidence": 1
            }
        },
        "workOrders": [
            {
                "id": "001-manual-review-native-dialogs",
                "readyToSend": true,
                "owner": "Named manual reviewer",
                "specSection": "6.5 File Operations",
                "requirementArea": "Native dialogs",
                "classification": "manual-review",
                "remainingGap": "Broader native proof.",
                "runbooks": ["runbooks/manual-review.md"],
                "returns": [".tmp/manual-review/001/signoff.json"],
                "validatorCommands": ["pnpm run check:a11y:manual"],
                "ingestCommand": "pnpm run ingest:evidence -- --source <returned-evidence-dir>",
                "matrixClosureCommand": "pnpm run check:spec-completion"
            },
            {
                "id": "002-cross-platform-external-engines",
                "readyToSend": true,
                "owner": "Supported-host QA owner",
                "specSection": "10.2 Safety",
                "requirementArea": "External engines",
                "classification": "cross-platform-evidence",
                "remainingGap": "Cross-platform proof.",
                "runbooks": ["runbooks/platform-evidence.md"],
                "returns": [".tmp/platform-evidence/external/linux/platform-evidence.json"],
                "validatorCommands": ["pnpm run check:platform-evidence"],
                "ingestCommand": "pnpm run ingest:evidence -- --source <returned-evidence-dir>",
                "matrixClosureCommand": "pnpm run check:spec-completion"
            }
        ]
    });
    fs::write(
        &spec_work_orders_path,
        serde_json::to_string_pretty(&spec_work_orders).expect("spec work orders json"),
    )
    .expect("write spec work orders fixture");
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
    fs::create_dir_all(&release_candidate_dir).expect("create release candidate dir");
    let release_candidate_source_commit = current_test_git_head();
    fs::write(
        release_candidate_dir.join("manifest.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "neditor.local-release-candidate.v1",
            "generatedAt": "2026-05-26T12:40:00.000Z",
            "releaseable": false,
            "product": {
                "name": "NEditor",
                "version": "0.1.0"
            },
            "source": {
                "commit": release_candidate_source_commit,
                "treeCleanBefore": true,
                "treeCleanAfter": true
            },
            "readiness": {
                "status": "current-host-ready-with-external-gaps",
                "evidenceGaps": [
                    {
                        "id": "homebrew-final-cask",
                        "status": "pending-release-cask",
                        "detail": "Set final cask SHA."
                    }
                ]
            },
            "evidenceKit": {
                "currentForSource": true,
                "coversReadiness": true,
                "reportCurrentForReadiness": true
            },
            "artifacts": [
                {
                    "kind": "frontend:index",
                    "path": "dist/index.html",
                    "size": 1200,
                    "sha256": "abc123"
                },
                {
                    "kind": "native:ned-cli",
                    "path": "src-tauri/target/release/ned",
                    "size": 3400,
                    "sha256": "def456"
                }
            ],
            "nextSteps": [
                "Ingest returned evidence with pnpm run ingest:evidence -- --source /path/to/unpacked-artifacts."
            ]
        }))
        .expect("release candidate manifest json"),
    )
    .expect("write release candidate manifest fixture");
    fs::write(
        release_candidate_dir.join("check-report.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "neditor.local-release-candidate-check.v1",
            "generatedAt": "2026-05-26T12:41:00.000Z",
            "status": "passed",
            "summary": {
                "issues": 0,
                "warnings": 1,
                "artifacts": 2
            },
            "issues": [],
            "warnings": ["fixture not final-releaseable"]
        }))
        .expect("release candidate check json"),
    )
    .expect("write release candidate check fixture");
    fs::write(
        release_candidate_dir.join("README.md"),
        "# Fixture Release Candidate\n\nReleaseable on this host: no\n",
    )
    .expect("write release candidate readme fixture");
    fs::write(
        release_candidate_dir.join("SHA256SUMS"),
        "abc123  dist/index.html\ndef456  src-tauri/target/release/ned\n",
    )
    .expect("write release candidate sums fixture");
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
        "--spec-work-orders".to_string(),
        spec_work_orders_path.to_string_lossy().to_string(),
        "--release-candidate-dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
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
    assert_eq!(bundle["releaseActionPlan"]["status"], "no-open-gaps");
    assert_eq!(
        bundle["releaseActionPlan"]["workItems"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        bundle["specCompletion"]["status"],
        "partial-with-release-risks"
    );
    assert_eq!(bundle["specCompletion"]["summary"]["openRows"], 2);
    assert_eq!(
        bundle["specCompletion"]["openRows"][0]["requirementArea"],
        "Native dialogs"
    );
    assert_eq!(bundle["specActionPlan"]["status"], "ready-to-send");
    assert_eq!(bundle["specActionPlan"]["readyToSendCount"], 2);
    assert_eq!(
        bundle["specActionPlan"]["workOrders"][0]["id"],
        "001-manual-review-native-dialogs"
    );
    assert_eq!(
        bundle["specActionPlan"]["workOrders"][1]["runbooks"][0],
        "runbooks/platform-evidence.md"
    );
    assert_eq!(
        bundle["releaseCandidate"]["status"],
        "checked-with-release-gates"
    );
    assert_eq!(bundle["releaseCandidate"]["releaseable"], false);
    assert_eq!(bundle["releaseCandidate"]["summary"]["artifacts"], 2);
    assert_eq!(bundle["releaseCandidate"]["summary"]["evidenceGaps"], 1);
    assert_eq!(
        bundle["improvementAudit"]["schema"],
        "neditor.100-improvements-audit.v1"
    );
    assert_eq!(bundle["improvementAudit"]["total"], 100);
    assert_eq!(bundle["improvementAudit"]["productionReady"], true);
    assert_eq!(bundle["improvementAudit"]["summary"]["open"], 0);
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
            .is_some_and(|value| value.contains("Initialize the NEditor workspace scaffold"))));
    assert!(!bundle["recommendations"]
        .as_array()
        .expect("recommendations")
        .iter()
        .any(|recommendation| recommendation
            .as_str()
            .is_some_and(|value| value.contains("Review ned doctor warnings"))));
    assert!(bundle["recommendations"]
        .as_array()
        .expect("recommendations")
        .iter()
        .any(|recommendation| recommendation
            .as_str()
            .is_some_and(|value| value.contains("Do not publish the release candidate"))));
    assert!(bundle["recommendations"]
        .as_array()
        .expect("recommendations")
        .iter()
        .any(|recommendation| recommendation
            .as_str()
            .is_some_and(|value| value.contains("release-candidate issue"))));
    assert!(bundle["recommendations"]
        .as_array()
        .expect("recommendations")
        .iter()
        .any(|recommendation| recommendation
            .as_str()
            .is_some_and(|value| value.contains("Assign 2 spec-completion work order"))));
    let readiness_gap_path = root.join("readiness-with-gap.json");
    fs::write(
        &readiness_gap_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "generatedAt": "2026-05-26T12:45:00.000Z",
            "platform": "darwin",
            "arch": "arm64",
            "status": "current-host-ready-with-external-gaps",
            "summary": {
                "requiredChecks": 2,
                "accepted": 1,
                "failed": 0,
                "evidenceGaps": 1
            },
            "checks": [],
            "evidenceGaps": [
                {
                    "id": "homebrew-final-cask",
                    "status": "pending-release-cask",
                    "detail": "Return final Homebrew cask."
                }
            ],
            "failures": []
        }))
        .expect("readiness gap json"),
    )
    .expect("write readiness gap fixture");
    let missing_kit_bundle = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "support-bundle".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--readiness-report".to_string(),
        readiness_gap_path.to_string_lossy().to_string(),
        "--spec-report".to_string(),
        spec_path.to_string_lossy().to_string(),
        "--spec-work-orders".to_string(),
        spec_work_orders_path.to_string_lossy().to_string(),
        "--release-candidate-dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
        "--engine-report".to_string(),
        engine_path.to_string_lossy().to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--evidence-kit".to_string(),
        root.join("missing-release-evidence-kit")
            .to_string_lossy()
            .to_string(),
        "--json".to_string(),
    ])
    .expect("missing kit support json");
    let missing_kit_bundle: serde_json::Value =
        serde_json::from_str(&missing_kit_bundle.message).expect("missing kit bundle json");
    assert_eq!(
        missing_kit_bundle["releaseActionPlan"]["status"],
        "missing-evidence-kit"
    );
    assert!(missing_kit_bundle["recommendations"]
        .as_array()
        .expect("missing kit recommendations")
        .iter()
        .any(|recommendation| recommendation
            .as_str()
            .is_some_and(|value| value.contains("Create or refresh the release evidence kit"))));

    let evidence_kit_dir = root.join("release-evidence-kit");
    fs::create_dir_all(&evidence_kit_dir).expect("create release evidence kit fixture");
    fs::write(
        evidence_kit_dir.join("manifest.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "neditor.release-evidence-kit.v1",
            "generatedAt": "2026-05-26T12:46:00.000Z",
            "sourceCommit": current_test_git_head(),
            "sourceTreeClean": true,
            "gapWorkItems": [
                {
                    "id": "homebrew-final-cask",
                    "readyToSend": true,
                    "status": "pending-release-cask",
                    "detail": "Return final Homebrew cask.",
                    "evidence": ".tmp/homebrew/homebrew-packaging-report.json",
                    "runbooks": [
                        {
                            "path": "runbooks/homebrew-release.md",
                            "title": "Homebrew Release Handoff"
                        }
                    ],
                    "returns": [
                        ".tmp/homebrew/Casks/neditor.rb",
                        ".tmp/homebrew/homebrew-packaging-report.json"
                    ],
                    "validatorCommands": ["pnpm run check:release-readiness"],
                    "ingestCommand": "pnpm run ingest:evidence -- --source /path/to/return-dir",
                    "finalReadinessCommand": "pnpm run check:release-readiness"
                }
            ]
        }))
        .expect("release evidence kit manifest json"),
    )
    .expect("write release evidence kit manifest fixture");
    let release_preview_text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "support-bundle".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--readiness-report".to_string(),
        readiness_gap_path.to_string_lossy().to_string(),
        "--spec-report".to_string(),
        spec_path.to_string_lossy().to_string(),
        "--spec-work-orders".to_string(),
        spec_work_orders_path.to_string_lossy().to_string(),
        "--release-candidate-dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
        "--engine-report".to_string(),
        engine_path.to_string_lossy().to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--evidence-kit".to_string(),
        evidence_kit_dir.to_string_lossy().to_string(),
    ])
    .expect("release preview support text");
    assert!(release_preview_text
        .message
        .contains("Release evidence work items:"));
    assert!(release_preview_text
        .message
        .contains("Status lanes: pending-release-cask=1"));
    assert!(release_preview_text.message.contains(
        "homebrew-final-cask [pending-release-cask] evidence: .tmp/homebrew/homebrew-packaging-report.json; returns: 2; runbook: runbooks/homebrew-release.md"
    ));
    assert!(release_preview_text.message.contains(
        "Next commands: pnpm run collect:evidence-kit -> pnpm run ingest:evidence -- --source /path/to/return-dir -> pnpm run check:release-readiness"
    ));

    let evidence_packet_path = root.join("support").join("release-evidence-packet.md");
    let evidence_packet = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "evidence-packet".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--readiness-report".to_string(),
        readiness_gap_path.to_string_lossy().to_string(),
        "--spec-report".to_string(),
        spec_path.to_string_lossy().to_string(),
        "--spec-work-orders".to_string(),
        spec_work_orders_path.to_string_lossy().to_string(),
        "--release-candidate-dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
        "--engine-report".to_string(),
        engine_path.to_string_lossy().to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--evidence-kit".to_string(),
        evidence_kit_dir.to_string_lossy().to_string(),
        "--output".to_string(),
        evidence_packet_path.to_string_lossy().to_string(),
    ])
    .expect("write evidence return packet");
    assert_eq!(evidence_packet.exit_code, 0);
    assert!(evidence_packet
        .message
        .contains("Wrote release evidence return packet"));
    let evidence_packet_markdown =
        fs::read_to_string(&evidence_packet_path).expect("read evidence packet");
    assert!(evidence_packet_markdown.contains("# NEditor Release Evidence Return Packet"));
    assert!(evidence_packet_markdown.contains("Release Evidence Assignments"));
    assert!(evidence_packet_markdown.contains("homebrew-final-cask"));
    assert!(evidence_packet_markdown.contains("Homebrew release owner"));
    assert!(evidence_packet_markdown.contains("Specification And Manual Review Work Orders"));
    assert!(evidence_packet_markdown.contains("001-manual-review-native-dialogs"));
    assert!(
        evidence_packet_markdown.contains("Do not include secrets, customer documents, API keys")
    );
    assert!(evidence_packet_markdown
        .contains("pnpm run ingest:evidence -- --source <returned-evidence-dir>"));
    assert!(evidence_packet_markdown.contains("neditor-platform-evidence-win32-json"));
    assert!(evidence_packet_markdown.contains("neditor-platform-evidence-linux-json"));

    let evidence_packet_json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "evidence-return-packet".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--readiness-report".to_string(),
        readiness_gap_path.to_string_lossy().to_string(),
        "--spec-report".to_string(),
        spec_path.to_string_lossy().to_string(),
        "--spec-work-orders".to_string(),
        spec_work_orders_path.to_string_lossy().to_string(),
        "--release-candidate-dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
        "--engine-report".to_string(),
        engine_path.to_string_lossy().to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--evidence-kit".to_string(),
        evidence_kit_dir.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("evidence return packet json");
    assert_eq!(evidence_packet_json.exit_code, 0);
    let evidence_packet_json: serde_json::Value =
        serde_json::from_str(&evidence_packet_json.message).expect("evidence packet json");
    assert_eq!(
        evidence_packet_json["schema"],
        "neditor.ned-evidence-return-packet.v1"
    );
    assert_eq!(
        evidence_packet_json["releaseActionPlan"]["workItems"][0]["id"],
        "homebrew-final-cask"
    );
    assert_eq!(
        evidence_packet_json["specActionPlan"]["workOrders"][0]["id"],
        "001-manual-review-native-dialogs"
    );
    assert!(evidence_packet_json["markdown"]
        .as_str()
        .expect("evidence packet markdown")
        .contains("Return Folder Layout"));

    let covered_missing_engine_path = root.join("engine-covered-missing.json");
    fs::write(
        &covered_missing_engine_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "generatedAt": "2026-05-26T12:50:00.000Z",
            "status": "complete",
            "summary": {
                "installed": 2,
                "missingLocal": 1,
                "incompatible": 0,
                "acceptedExternalEvidence": 1,
                "invalidExternalEvidence": 0,
                "unresolvedMissingEvidence": 0
            },
            "engines": [
                {
                    "key": "pikchr",
                    "name": "Pikchr",
                    "status": "missing",
                    "command": "pikchr or pikchr-cli",
                    "path": null,
                    "version": null,
                    "smoke": null,
                    "externalEvidence": {
                        "status": "accepted",
                        "path": ".tmp/external-engines/external/pikchr.json"
                    }
                }
            ]
        }))
        .expect("covered missing engine json"),
    )
    .expect("write covered missing engine fixture");
    let covered_missing_engine_bundle = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "support-bundle".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--readiness-report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--spec-report".to_string(),
        spec_path.to_string_lossy().to_string(),
        "--spec-work-orders".to_string(),
        spec_work_orders_path.to_string_lossy().to_string(),
        "--release-candidate-dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
        "--engine-report".to_string(),
        covered_missing_engine_path.to_string_lossy().to_string(),
        "--evidence-root".to_string(),
        evidence_root.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("covered missing engine support json");
    let covered_missing_engine_bundle: serde_json::Value =
        serde_json::from_str(&covered_missing_engine_bundle.message)
            .expect("covered missing engine bundle json");
    assert_eq!(
        covered_missing_engine_bundle["engineProbe"]["summary"]["missingLocal"],
        1
    );
    assert!(covered_missing_engine_bundle["recommendations"]
        .as_array()
        .expect("covered missing recommendations")
        .iter()
        .all(|recommendation| !recommendation
            .as_str()
            .unwrap_or_default()
            .contains("Review transform engine setup")));

    let text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "support-bundle".to_string(),
        "--workspace".to_string(),
        root.to_string_lossy().to_string(),
        "--readiness-report".to_string(),
        report_path.to_string_lossy().to_string(),
        "--spec-report".to_string(),
        spec_path.to_string_lossy().to_string(),
        "--release-candidate-dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
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
    assert!(text
        .message
        .contains("Release action plan: no-open-gaps (0/0 work items ready)"));
    assert!(text
        .message
        .contains("Spec action plan: ready-to-send (2/2 work orders ready)"));
    assert!(text.message.contains("Spec completion work orders:"));
    assert!(text
        .message
        .contains("Classification lanes: cross-platform-evidence=1, manual-review=1"));
    assert!(text
        .message
        .contains("Owner lanes: Named manual reviewer=1, Supported-host QA owner=1"));
    assert!(text.message.contains(
        "001-manual-review-native-dialogs [manual-review] owner: Named manual reviewer; 6.5 File Operations / Native dialogs; returns: 1; runbook: runbooks/manual-review.md"
    ));
    assert!(text
        .message
        .contains("Release candidate: checked-with-release-gates (releaseable: no, artifacts: 2)"));
    assert!(text.message.contains("Recommended next actions:"));
    assert!(text.message.contains("  Local setup:"));
    assert!(text.message.contains("  Release readiness:"));
    assert!(text.message.contains("  Specification closure:"));
    assert!(text.message.contains("  Evidence collection:"));
    assert!(text
        .message
        .contains("    - Initialize the NEditor workspace scaffold"));
    assert!(text
        .message
        .contains("    - Do not publish the release candidate"));
    assert!(text
        .message
        .contains("    - Resolve 1 release-candidate issue"));
    assert!(text
        .message
        .contains("    - Assign 2 spec-completion work order"));
    assert!(text
        .message
        .contains("    - Collect or refresh 9 release evidence report"));
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
        spec_work_orders: Some(spec_work_orders_path.to_string_lossy().to_string()),
        release_candidate_dir: Some(release_candidate_dir.to_string_lossy().to_string()),
        engine_report: Some(engine_path.to_string_lossy().to_string()),
        evidence_root: Some(evidence_root.to_string_lossy().to_string()),
        evidence_kit: None,
        output: Some(ipc_output_path.to_string_lossy().to_string()),
    })
    .expect("ipc support bundle");
    assert_eq!(ipc_bundle["schema"], "neditor.ned-support-bundle.v1");
    assert_eq!(
        ipc_bundle["writtenTo"],
        ipc_output_path.to_string_lossy().as_ref()
    );
    assert!(ipc_output_path.is_file());

    let candidate_json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "release-candidate".to_string(),
        "--candidate-dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
        "--json".to_string(),
    ])
    .expect("release candidate json");
    assert_eq!(candidate_json.exit_code, 0);
    let candidate: serde_json::Value =
        serde_json::from_str(&candidate_json.message).expect("candidate json");
    assert_eq!(candidate["schema"], "neditor.ned-release-candidate.v1");
    assert_eq!(candidate["status"], "checked-with-release-gates");
    assert_eq!(candidate["summary"]["checkStatus"], "passed");
    assert_eq!(
        candidate["nextSteps"][0]
            .as_str()
            .unwrap()
            .contains("ingest:evidence"),
        true
    );

    let candidate_text = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "candidate".to_string(),
        "--dir".to_string(),
        release_candidate_dir.to_string_lossy().to_string(),
    ])
    .expect("release candidate text");
    assert_eq!(candidate_text.exit_code, 0);
    assert!(candidate_text
        .message
        .contains("Release candidate: checked-with-release-gates"));
    assert!(candidate_text
        .message
        .contains("Releaseable on this host: no"));
}

#[test]
fn ned_cli_audits_100_improvements_as_actionable_work_orders() {
    let output = temp_markdown_path("improvement-coverage");
    let json = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "improvements".to_string(),
        "--json".to_string(),
        "--output".to_string(),
        output.to_string_lossy().to_string(),
    ])
    .expect("improvements json");
    assert_eq!(json.exit_code, 0);
    let report: serde_json::Value =
        serde_json::from_str(&json.message).expect("improvements report json");
    assert_eq!(report["schema"], "neditor.100-improvements-audit.v1");
    assert_eq!(report["source"], "docs/100-improve.md");
    assert_eq!(report["total"], 100);
    assert_eq!(report["productionReady"], true);
    assert!(
        report["summary"]["implementedEvidencePresent"]
            .as_u64()
            .expect("implemented count")
            > 0
    );
    assert_eq!(report["summary"]["open"], 0);
    assert!(report["items"]
        .as_array()
        .expect("improvement items")
        .iter()
        .any(|item| item["number"] == 11
            && item["title"] == "Reusable business profile"
            && item["status"] == "implemented-evidence-present"));
    assert!(report["items"]
        .as_array()
        .expect("improvement items")
        .iter()
        .any(|item| item["number"] == 18
            && item["title"] == "Approval metadata gate"
            && item["status"] == "implemented-evidence-present"));
    assert!(report["items"]
        .as_array()
        .expect("improvement items")
        .iter()
        .any(|item| item["number"] == 21 && item["title"] == "Native RFP ingestion"));
    for (number, title) in [
        (10, "Provider-agnostic AI runtime"),
        (32, "Search provider choices"),
        (33, "Source document vault"),
        (40, "Research audit packet"),
    ] {
        assert!(report["items"]
            .as_array()
            .expect("improvement items")
            .iter()
            .any(|item| item["number"] == number
                && item["title"] == title
                && item["status"] == "implemented-evidence-present"));
    }
    assert!(report["items"]
        .as_array()
        .expect("improvement items")
        .iter()
        .any(|item| item["number"] == 63
            && item["title"] == "Page design presets"
            && item["status"] == "implemented-evidence-present"));
    assert!(report["items"]
        .as_array()
        .expect("improvement items")
        .iter()
        .any(|item| item["number"] == 67
            && item["title"] == "Chart designer"
            && item["status"] == "implemented-evidence-present"));
    assert!(report["items"]
        .as_array()
        .expect("improvement items")
        .iter()
        .any(|item| item["number"] == 79
            && item["title"] == "Distribution preflight"
            && item["status"] == "implemented-evidence-present"));
    assert!(report["items"]
        .as_array()
        .expect("improvement items")
        .iter()
        .any(|item| item["number"] == 80
            && item["title"] == "Export profiles"
            && item["status"] == "implemented-evidence-present"));
    for (number, title) in [
        (71, "HTML export polish"),
        (72, "EPUB export polish"),
        (73, "Google Docs import handoff"),
        (74, "Substack package export"),
        (75, "Blog and CMS publishing"),
        (76, "LaTeX and PDF build path"),
        (77, "DOCX style mapping"),
        (78, "Markdown bundle export"),
        (81, "Voice command interface"),
        (82, "Voice-first document wizard"),
        (83, "Read selected text aloud"),
        (84, "Consent-gated TTS models"),
        (85, "Voice correction loop"),
        (86, "Screen-reader QA mode"),
        (91, "Unified configurator"),
        (92, "Guided provider setup"),
        (93, "Ollama model picker"),
        (94, "Transform handler installer"),
        (98, "Homebrew release path"),
        (87, "Accessible command palette"),
        (88, "Keyboard-first table editing"),
        (89, "High-contrast and reduced-motion modes"),
        (90, "Plain-language help overlays"),
        (99, "Release evidence dashboard"),
    ] {
        assert!(report["items"]
            .as_array()
            .expect("improvement items")
            .iter()
            .any(|item| item["number"] == number
                && item["title"] == title
                && item["status"] == "implemented-evidence-present"));
    }
    assert_eq!(report["summary"]["implementedEvidencePresent"], 100);
    assert_eq!(report["summary"]["open"], 0);
    assert_eq!(report["productionReady"], true);

    let markdown = fs::read_to_string(&output).expect("improvements markdown");
    assert!(markdown.contains("# NEditor 100 Improvements Coverage Audit"));
    assert!(markdown.contains("## Category Summary"));
    assert!(markdown.contains("## Improvement Work Orders"));
    assert!(markdown.contains("Native RFP ingestion"));

    let strict = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "improvements".to_string(),
        "--strict".to_string(),
    ])
    .expect("strict improvements");
    assert_eq!(strict.exit_code, 0);
    assert!(strict
        .message
        .contains("NEditor 100 Improvements Coverage Audit"));
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
    assert!(bash.message.contains("evidence-packet"));
    assert!(bash.message.contains("improvements"));
    assert!(bash.message.contains("read-aloud"));
    assert!(bash.message.contains("--acknowledge-model-download"));
    assert!(bash.message.contains("sources"));
    assert!(bash.message.contains("--download-url"));
    assert!(bash.message.contains("deep-research"));
    assert!(bash.message.contains("--target-pages"));
    assert!(bash.message.contains("quality"));
    assert!(bash.message.contains("--report"));
    assert!(bash.message.contains("snippets"));
    assert!(bash.message.contains("--markdown"));
    assert!(bash.message.contains("--title"));
    assert!(bash.message.contains("--save"));
    assert!(bash.message.contains("--outline-file"));
    assert!(bash.message.contains("--fill-profile"));
    assert!(bash.message.contains("--fields"));
    assert!(bash.message.contains("--get"));
    assert!(bash.message.contains("support-bundle"));
    assert!(bash.message.contains("inspect"));
    assert!(bash.message.contains("rfp-response"));
    assert!(bash.message.contains("publish"));
    assert!(bash.message.contains("export-profiles"));
    assert!(bash.message.contains("--citation-style"));
    assert!(bash.message.contains("--token-env"));
    assert!(bash.message.contains("--matrix-output"));
    assert!(bash.message.contains("--checklist-output"));
    assert!(bash.message.contains("--outline-output"));
    assert!(bash.message.contains("--coverage-output"));
    assert!(bash.message.contains("--matrix --checklist"));
    assert!(bash.message.contains("--proposal-outline"));
    assert!(bash.message.contains("--validator"));
    assert!(bash.message.contains("improvement-audit"));
    assert!(bash.message.contains("deploy-cli"));
    assert!(bash.message.contains("--target-dir"));
    assert!(bash.message.contains("markdown-bundle"));
    assert!(bash.message.contains("ai-runtime"));
    assert!(bash.message.contains("google-docs-import"));
    assert!(bash.message.contains("homebrew-release"));
    assert!(bash.message.contains("--key-env"));

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
    assert!(zsh.message.contains("evidence-return-packet"));
    assert!(zsh.message.contains("roadmap\\:roadmap"));
    assert!(zsh.message.contains("read-aloud\\:read-aloud"));
    assert!(zsh
        .message
        .contains("--script-output[write auditable shell script]"));
    assert!(zsh
        .message
        .contains("'--output[write Markdown packet]:file:_files'"));
    assert!(zsh.message.contains("--ids-only"));
    assert!(zsh.message.contains("--markdown"));
    assert!(zsh.message.contains("--title"));
    assert!(zsh.message.contains("--best-for"));
    assert!(zsh.message.contains("--docs-live-type"));
    assert!(zsh.message.contains("--fill-profile"));
    assert!(zsh.message.contains("--fields"));
    assert!(zsh.message.contains("--get"));
    assert!(zsh.message.contains("citation-sources"));
    assert!(zsh.message.contains("--provider[search provider]"));
    assert!(zsh.message.contains("research-report"));
    assert!(zsh.message.contains("--save-sources"));
    assert!(zsh.message.contains("quality\\:quality"));
    assert!(zsh
        .message
        .contains("--markdown[print Markdown reviewer handoff]"));
    assert!(zsh.message.contains("--endpoint"));
    assert!(zsh.message.contains("--allow-not-ready"));
    assert!(zsh
        .message
        .contains("delivery-profiles\\:delivery-profiles"));
    assert!(zsh.message.contains("--brand[set brand default key=value]"));
    assert!(zsh.message.contains("--matrix-output"));
    assert!(zsh.message.contains("--checklist-output"));
    assert!(zsh
        .message
        .contains("'--checklist[print compliance checklist Markdown]'"));
    assert!(zsh.message.contains("--outline-output"));
    assert!(zsh
        .message
        .contains("--outline[print compliance checklist and proposal outline Markdown]"));
    assert!(zsh.message.contains("--coverage-output"));
    assert!(zsh
        .message
        .contains("--coverage[print requirement coverage validator Markdown]"));
    assert!(zsh
        .message
        .contains("--strict[fail until all 100 improvement items are evidenced]"));
    assert!(zsh.message.contains("deploy-cli"));
    assert!(zsh.message.contains("--target-dir"));
    assert!(zsh.message.contains("ai-runtime\\:ai-runtime"));
    assert!(zsh.message.contains("google-docs-import"));
    assert!(zsh.message.contains("homebrew-release"));
    assert!(zsh.message.contains("--profile[select provider profile]"));

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
    assert!(fish.message.contains("evidence-packet"));
    assert!(fish.message.contains("release-evidence-packet"));
    assert!(fish.message.contains("improvement-audit"));
    assert!(fish.message.contains("read-aloud"));
    assert!(fish.message.contains("acknowledge-model-download"));
    assert!(fish.message.contains("citation-sources"));
    assert!(fish.message.contains("download-url"));
    assert!(fish.message.contains("deep-research"));
    assert!(fish.message.contains("target-pages"));
    assert!(fish
        .message
        .contains("__fish_seen_subcommand_from quality qa review"));
    assert!(fish.message.contains("-l title"));
    assert!(fish.message.contains("snippets"));
    assert!(fish.message.contains("ids-only"));
    assert!(fish.message.contains("outline-file"));
    assert!(fish.message.contains("fill-profile"));
    assert!(fish.message.contains("fields"));
    assert!(fish.message.contains("get"));
    assert!(fish.message.contains("matrix-output"));
    assert!(fish.message.contains("checklist-output"));
    assert!(fish.message.contains("outline-output"));
    assert!(fish.message.contains("coverage-output"));
    assert!(fish.message.contains("-l checklist"));
    assert!(fish.message.contains("-l proposal-outline"));
    assert!(fish.message.contains("-l validator"));
    assert!(fish
        .message
        .contains("__fish_seen_subcommand_from improvements improvement-audit roadmap"));
    assert!(fish.message.contains("support-bundle"));
    assert!(fish.message.contains("inspect"));
    assert!(fish.message.contains("publish"));
    assert!(fish.message.contains("export-profiles"));
    assert!(fish.message.contains("citation-style"));
    assert!(fish.message.contains("token-env"));
    assert!(fish.message.contains("deploy-cli"));
    assert!(fish.message.contains("target-dir"));
    assert!(fish.message.contains("epub"));
    assert!(fish
        .message
        .contains("__fish_seen_subcommand_from ai-runtime ai-providers"));
    assert!(fish
        .message
        .contains("__fish_seen_subcommand_from google-docs-import google-docs-handoff"));
    assert!(fish
        .message
        .contains("__fish_seen_subcommand_from homebrew-release homebrew"));

    let unsupported = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "completions".to_string(),
        "powershell".to_string(),
    ])
    .expect_err("unsupported shell");
    assert!(unsupported.contains("Supported shells: bash, zsh, fish"));
}

#[test]
fn ned_cli_read_aloud_builds_consent_gated_tts_plans() {
    let blocked = crate::cli::run_cli_with_args_and_stdin(
        &[
            "ned".to_string(),
            "read-aloud".to_string(),
            "-".to_string(),
            "--engine".to_string(),
            "supertonic-cli".to_string(),
            "--model-storage".to_string(),
            "~/.cache/supertonic/models".to_string(),
            "--dry-run".to_string(),
            "--json".to_string(),
        ],
        Some("# Board Memo\n\nRead the confidential quarterly plan."),
    )
    .expect_err("supertonic should require explicit model acknowledgement");
    assert!(blocked.contains("~305 MB"));
    assert!(blocked.contains("~/.cache/supertonic/models"));

    let script_path = temp_markdown_path("read-aloud").with_extension("sh");
    let outcome = crate::cli::run_cli_with_args_and_stdin(
        &[
            "ned".to_string(),
            "tts".to_string(),
            "-".to_string(),
            "--engine".to_string(),
            "supertonic-cli".to_string(),
            "--acknowledge-model-download".to_string(),
            "--model-storage".to_string(),
            "~/.cache/supertonic/models".to_string(),
            "--voice".to_string(),
            "F1".to_string(),
            "--dry-run".to_string(),
            "--json".to_string(),
            "--script-output".to_string(),
            script_path.to_string_lossy().to_string(),
        ],
        Some("# Board Memo\n\nRead the confidential quarterly plan."),
    )
    .expect("acknowledged supertonic dry-run");
    assert_eq!(outcome.exit_code, 0);
    let report: serde_json::Value =
        serde_json::from_str(&outcome.message).expect("read-aloud json");
    assert_eq!(report["schema"], "neditor.ned-read-aloud.v1");
    assert_eq!(report["engine"], "supertonic-cli");
    assert_eq!(report["dryRun"], true);
    assert_eq!(report["modelDownload"]["acknowledged"], true);
    assert_eq!(report["modelDownload"]["approximateSize"], "~305 MB");
    assert_eq!(report["command"]["args"][1], "<text:48 chars>");
    assert!(!outcome.message.contains("confidential quarterly plan"));
    assert!(script_path.is_file());
    let script = fs::read_to_string(&script_path).expect("read generated TTS script");
    assert!(script.contains("supertonic"));
    assert!(script.contains("confidential quarterly plan"));
}

#[cfg(target_os = "macos")]
#[test]
fn ned_cli_read_aloud_uses_macos_say_without_shell_interpolation() {
    let outcome = crate::cli::run_cli_with_args_and_stdin(
        &[
            "ned".to_string(),
            "speak".to_string(),
            "-".to_string(),
            "--engine".to_string(),
            "macos-say".to_string(),
            "--voice".to_string(),
            "Samantha".to_string(),
            "--rate".to_string(),
            "180".to_string(),
            "--dry-run".to_string(),
            "--json".to_string(),
        ],
        Some("# Memo\n\nRead this safely."),
    )
    .expect("macOS Say dry-run");
    let report: serde_json::Value = serde_json::from_str(&outcome.message).expect("macos say json");
    assert_eq!(report["schema"], "neditor.ned-read-aloud.v1");
    assert_eq!(report["engine"], "macos-say");
    assert_eq!(report["command"]["program"], "say");
    assert_eq!(report["command"]["usesStdin"], true);
    assert_eq!(report["command"]["stdin"], "<text:22 chars>");
    assert!(!outcome.message.contains("Read this safely"));
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
    let blocked = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "publish".to_string(),
        source.to_string_lossy().to_string(),
        "--target".to_string(),
        "blog".to_string(),
        "--endpoint".to_string(),
        "https://cms.example.com/wp-json/wp/v2/posts".to_string(),
    ])
    .expect_err("publish should be blocked without approval metadata");
    assert!(blocked.contains("Publish payload blocked by"));
    assert!(blocked.contains("--allow-not-ready"));

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
    assert_eq!(
        report["payload"]["destinationWorkflow"],
        "session-token-api-payload"
    );
    assert_eq!(report["payload"]["contentFormat"], "markdown");
    assert_eq!(report["payload"]["title"], "Test Report");
    assert_eq!(report["payload"]["targetPayload"]["status"], "draft");
    assert!(report["payload"]["targetPayload"]["content"]
        .as_str()
        .expect("wordpress content")
        .contains("# Test Report"));
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
fn ned_cli_prepares_substack_and_static_site_publishing_handoffs() {
    let source = temp_markdown_path("publish-handoffs");
    fs::write(&source, super::sample_document()).expect("write source markdown");

    let substack = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "publish".to_string(),
        source.to_string_lossy().to_string(),
        "--target".to_string(),
        "substack".to_string(),
        "--destination".to_string(),
        "substack-manual".to_string(),
        "--format".to_string(),
        "html".to_string(),
        "--allow-not-ready".to_string(),
        "--json".to_string(),
    ])
    .expect("substack handoff");
    let substack_report: serde_json::Value =
        serde_json::from_str(&substack.message).expect("substack json");
    assert_eq!(
        substack_report["payload"]["destinationKind"],
        "substack-manual"
    );
    assert_eq!(
        substack_report["payload"]["destinationWorkflow"],
        "manual-copy-package"
    );
    assert!(
        substack_report["payload"]["substackPackage"]["copyReadyHtml"]
            .as_str()
            .expect("substack copy html")
            .contains("Test Report")
    );
    assert!(substack_report["payload"]["handoffFiles"]
        .as_array()
        .expect("substack files")
        .iter()
        .any(|file| file["path"] == "substack-copy.html"));
    assert!(substack_report["payload"]["deliveryInstructions"]
        .as_array()
        .expect("substack instructions")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("Substack editor"))));
    assert!(substack_report["payload"]["curlTemplate"]
        .as_str()
        .expect("manual curl")
        .contains("Manual publishing destination"));

    let static_site = crate::cli::run_cli_with_args(&[
        "ned".to_string(),
        "publish".to_string(),
        source.to_string_lossy().to_string(),
        "--target".to_string(),
        "blog".to_string(),
        "--destination".to_string(),
        "static-site-bundle".to_string(),
        "--format".to_string(),
        "html".to_string(),
        "--allow-not-ready".to_string(),
        "--json".to_string(),
    ])
    .expect("static site handoff");
    let static_report: serde_json::Value =
        serde_json::from_str(&static_site.message).expect("static site json");
    assert_eq!(
        static_report["payload"]["destinationKind"],
        "static-site-bundle"
    );
    assert_eq!(
        static_report["payload"]["destinationWorkflow"],
        "static-file-bundle"
    );
    assert!(static_report["payload"]["staticSiteBundle"]["files"]
        .as_array()
        .expect("static bundle files")
        .contains(&serde_json::json!("index.html")));
    assert!(static_report["payload"]["targetPayload"]["files"]
        .as_array()
        .expect("static target files")
        .iter()
        .any(|file| file["path"] == "neditor-manifest.json"));
    assert!(static_report["payload"]["deliveryInstructions"]
        .as_array()
        .expect("static instructions")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("site repository"))));
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
fn deploy_cli_installs_user_level_ned_launcher_without_overwriting_conflicts() {
    let source_dir = temp_workspace_path("cli-deploy-source");
    let target_dir = temp_workspace_path("cli-deploy-target");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::create_dir_all(&target_dir).expect("create target dir");
    let source = source_dir.join(format!("ned{}", std::env::consts::EXE_SUFFIX));
    write_fake_ned_binary(&source);

    let deployed = crate::cli::deploy_cli_from_source(
        &source,
        Some(target_dir.to_string_lossy().as_ref()),
        false,
    )
    .expect("deploy cli");
    assert!(deployed.applied);
    assert!(deployed.supported);
    assert!(deployed
        .deployed_path
        .ends_with(&format!("ned{}", std::env::consts::EXE_SUFFIX)));
    assert!(target_dir
        .join(format!("ned{}", std::env::consts::EXE_SUFFIX))
        .exists());
    assert!(deployed
        .manual_steps
        .iter()
        .any(|step| step.contains("ned doctor --json")));

    let idempotent = crate::cli::deploy_cli_from_source(
        &source,
        Some(target_dir.to_string_lossy().as_ref()),
        false,
    )
    .expect("idempotent deploy cli");
    assert!(idempotent.applied);
    assert!(idempotent.message.contains("already deployed"));

    let copied_dir = temp_workspace_path("cli-deploy-copied");
    fs::create_dir_all(&copied_dir).expect("create copied dir");
    let copied_path = copied_dir.join(format!("ned{}", std::env::consts::EXE_SUFFIX));
    fs::copy(&source, &copied_path).expect("copy existing ned");
    let copied = crate::cli::deploy_cli_from_source(
        &source,
        Some(copied_dir.to_string_lossy().as_ref()),
        false,
    )
    .expect("copied deploy cli");
    assert!(copied.applied);
    assert!(copied.message.contains("already deployed"));

    let conflict_dir = temp_workspace_path("cli-deploy-conflict");
    fs::create_dir_all(&conflict_dir).expect("create conflict dir");
    let conflict_path = conflict_dir.join(format!("ned{}", std::env::consts::EXE_SUFFIX));
    fs::write(&conflict_path, "different ned").expect("write existing ned");
    let conflict = crate::cli::deploy_cli_from_source(
        &source,
        Some(conflict_dir.to_string_lossy().as_ref()),
        false,
    )
    .expect("conflict deploy cli");
    assert!(!conflict.applied);
    assert!(!conflict.supported);
    assert!(conflict.message.contains("was not overwritten"));
    assert_eq!(
        fs::read_to_string(conflict_path).expect("conflict file"),
        "different ned"
    );
}

#[test]
fn ned_cli_deploy_cli_reports_status_and_installs_from_terminal() {
    let source_dir = temp_workspace_path("cli-deploy-command-source");
    let target_dir = temp_workspace_path("cli-deploy-command-target");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::create_dir_all(&target_dir).expect("create target dir");
    let source = source_dir.join(format!("ned{}", std::env::consts::EXE_SUFFIX));
    write_fake_ned_binary(&source);

    let status = crate::cli::run_deploy_cli_command_with_source(
        &[
            "--status".to_string(),
            "--target-dir".to_string(),
            target_dir.to_string_lossy().to_string(),
            "--json".to_string(),
        ],
        Some(&source),
    )
    .expect("deploy cli status");
    assert_eq!(status.exit_code, 0);
    let status_report: serde_json::Value =
        serde_json::from_str(&status.message).expect("deploy cli status json");
    assert_eq!(status_report["schema"], "neditor.ned-deploy-cli.v1");
    assert_eq!(status_report["status"], "ready");
    assert_eq!(status_report["requestedDeploy"], false);
    assert_eq!(status_report["statusOnly"], true);
    assert_eq!(status_report["deployment"]["applied"], false);
    assert_eq!(status_report["deployment"]["supported"], true);
    assert_eq!(
        status_report["deployment"]["deployedPath"],
        serde_json::json!(target_dir
            .join(format!("ned{}", std::env::consts::EXE_SUFFIX))
            .to_string_lossy()
            .to_string())
    );
    assert!(status_report["nextCommands"]
        .as_array()
        .expect("next commands")
        .contains(&serde_json::json!("ned deploy-cli")));

    let deploy = crate::cli::run_deploy_cli_command_with_source(
        &[
            "--target-dir".to_string(),
            target_dir.to_string_lossy().to_string(),
            "--json".to_string(),
        ],
        Some(&source),
    )
    .expect("deploy cli install");
    assert_eq!(deploy.exit_code, 0);
    let deploy_report: serde_json::Value =
        serde_json::from_str(&deploy.message).expect("deploy cli install json");
    assert_eq!(deploy_report["status"], "deployed");
    assert_eq!(deploy_report["requestedDeploy"], true);
    assert_eq!(deploy_report["deployment"]["applied"], true);
    assert!(target_dir
        .join(format!("ned{}", std::env::consts::EXE_SUFFIX))
        .exists());

    let text = crate::cli::run_deploy_cli_command_with_source(
        &[
            "--status".to_string(),
            "--target-dir".to_string(),
            target_dir.to_string_lossy().to_string(),
        ],
        Some(&source),
    )
    .expect("deploy cli text status");
    assert_eq!(text.exit_code, 0);
    assert!(text.message.contains("Deploy CLI: deployed"));
    assert!(text.message.contains("Target:"));
    assert!(text.message.contains("ned doctor --json"));
}

#[test]
fn ned_cli_deploy_cli_refuses_unknown_target_without_overwrite() {
    let source_dir = temp_workspace_path("cli-deploy-command-conflict-source");
    let target_dir = temp_workspace_path("cli-deploy-command-conflict-target");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::create_dir_all(&target_dir).expect("create target dir");
    let source = source_dir.join(format!("ned{}", std::env::consts::EXE_SUFFIX));
    write_fake_ned_binary(&source);
    let conflict_path = target_dir.join(format!("ned{}", std::env::consts::EXE_SUFFIX));
    fs::write(&conflict_path, "different ned").expect("write existing ned");

    let blocked = crate::cli::run_deploy_cli_command_with_source(
        &[
            "--target-dir".to_string(),
            target_dir.to_string_lossy().to_string(),
            "--json".to_string(),
        ],
        Some(&source),
    )
    .expect("deploy cli blocked conflict");
    assert_eq!(blocked.exit_code, 1);
    let blocked_report: serde_json::Value =
        serde_json::from_str(&blocked.message).expect("deploy cli blocked json");
    assert_eq!(blocked_report["schema"], "neditor.ned-deploy-cli.v1");
    assert_eq!(blocked_report["status"], "manual-setup-required");
    assert_eq!(blocked_report["deployment"]["applied"], false);
    assert_eq!(blocked_report["deployment"]["supported"], false);
    assert!(blocked_report["deployment"]["message"]
        .as_str()
        .expect("message")
        .contains("was not overwritten"));
    assert_eq!(
        fs::read_to_string(conflict_path).expect("conflict file"),
        "different ned"
    );
}

#[test]
fn deploy_cli_refuses_generated_sidecar_placeholder() {
    let source_dir = temp_workspace_path("cli-deploy-placeholder-source");
    let target_dir = temp_workspace_path("cli-deploy-placeholder-target");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::create_dir_all(&target_dir).expect("create target dir");
    let source = source_dir.join(format!("ned{}", std::env::consts::EXE_SUFFIX));
    fs::write(
        &source,
        "placeholder generated by build.rs; run pnpm run prepare:sidecars for release packaging\n",
    )
    .expect("write placeholder ned");

    let error = crate::cli::deploy_cli_from_source(
        &source,
        Some(target_dir.to_string_lossy().as_ref()),
        false,
    )
    .expect_err("placeholder sidecar rejected");
    assert!(error.contains("generated sidecar placeholder"));
    assert!(!target_dir
        .join(format!("ned{}", std::env::consts::EXE_SUFFIX))
        .exists());
}

#[test]
fn ned_cli_help_names_supported_conversion_targets() {
    let args = vec!["ned".to_string(), "--help".to_string()];
    let outcome = crate::cli::run_cli_with_args(&args).expect("help");
    assert_eq!(outcome.exit_code, 0);
    assert!(outcome.message.contains("ned convert"));
    assert!(outcome.message.contains("ned publish"));
    assert!(outcome.message.contains("ned export-profiles"));
    assert!(outcome.message.contains("--output-dir"));
    assert!(outcome.message.contains("--stdout"));
    assert!(outcome.message.contains("ned init"));
    assert!(outcome.message.contains("ned new"));
    assert!(outcome.message.contains("ned inspect"));
    assert!(outcome.message.contains("ned validate"));
    assert!(outcome.message.contains("ned quality"));
    assert!(outcome.message.contains("ned templates"));
    assert!(outcome.message.contains("ned snippets"));
    assert!(outcome.message.contains("ned targets"));
    assert!(outcome.message.contains("ned handlers"));
    assert!(outcome.message.contains("ned readiness"));
    assert!(outcome.message.contains("ned evidence"));
    assert!(outcome.message.contains("ned evidence-packet"));
    assert!(outcome.message.contains("ned improvements"));
    assert!(outcome.message.contains("ned sources --document"));
    assert!(outcome.message.contains("ned deep-research"));
    assert!(outcome.message.contains("ned support-bundle"));
    assert!(outcome
        .message
        .contains("ned default-reader --status [--json]"));
    assert!(outcome.message.contains("ned deploy-cli"));
    assert!(outcome.message.contains("ned completions"));
    assert!(outcome.message.contains("ned doctor"));
    assert!(outcome.message.contains("--workspace"));
    assert!(outcome.message.contains("docx"));
    assert!(outcome.message.contains("epub"));
    assert!(outcome.message.contains("or all"));
    assert!(outcome.message.contains("rfp-response"));
    assert!(outcome.message.contains("--outline-output outline.md"));
    assert!(outcome.message.contains("--coverage-output coverage.md"));
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

fn current_test_git_head() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|commit| !commit.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn write_fake_ned_binary(path: &std::path::Path) {
    let mut bytes = vec![0u8; 128 * 1024];
    bytes[..16].copy_from_slice(b"ned test helper\n");
    fs::write(path, bytes).expect("write fake ned binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).expect("fake ned metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("make fake ned executable");
    }
}
