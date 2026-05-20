use super::*;

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
fn prepare_for_export_reports_review_change_note_audit_metadata() {
    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Review Audit\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# Review Audit\n<!-- comment: resolved | author: Dana | Confirmed numbers. -->\n<!-- change: at: 2026-05-20T09:00:00Z | Updated forecast assumptions. -->\n"
            .to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 2);
    assert_eq!(report.readiness.warning_count, report.warning_count);
    assert_eq!(
        report.manifest.readiness.warning_count,
        report.warning_count
    );
    assert!(!report.manifest.readiness.ready);
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.message == "Review comment is missing audit metadata."
                && diagnostic.source_file.as_deref() == Some("untitled.md")
                && diagnostic.line == Some(9)
                && diagnostic
                    .related
                    .iter()
                    .any(|related| related.contains("at=missing"))
        }),
        "{:#?}",
        report.diagnostics
    );
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.message == "Change note is missing audit metadata."
                && diagnostic.source_file.as_deref() == Some("untitled.md")
                && diagnostic.line == Some(10)
                && diagnostic
                    .related
                    .iter()
                    .any(|related| related.contains("author=missing"))
        }),
        "{:#?}",
        report.diagnostics
    );
    assert!(report
        .manifest
        .diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.message == "Change note is missing audit metadata." }));
}

#[test]
fn prepare_for_export_reports_ai_provenance_audit_metadata() {
    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: AI Audit\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# AI Audit\n```ai-source\nprovider: OpenAI\ndate: 2026-05-20\npromptSummary: Board pack outline\nreviewedBy: QA\nstatus: human-reviewed\n```\n\n<!-- ai-assisted: status=human-reviewed | reviewedBy=QA | source= | promptSummary= -->\n## Drafted Section\nReviewed content.\n"
            .to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "includeProvenance": true, "warnOnDirtyGit": false }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 4);
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.message == "AI source block is missing provenance metadata."
                && diagnostic
                    .related
                    .iter()
                    .any(|related| related.contains("model=missing"))
        }),
        "{:#?}",
        report.diagnostics
    );
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.message == "AI source is marked human-reviewed without reviewer metadata."
                && diagnostic
                    .related
                    .iter()
                    .any(|related| related.contains("reviewedAt=missing"))
        }),
        "{:#?}",
        report.diagnostics
    );
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.message == "AI-assisted section marker is missing provenance metadata."
                && diagnostic
                    .related
                    .iter()
                    .any(|related| related.contains("source=missing"))
        }),
        "{:#?}",
        report.diagnostics
    );
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.message
                == "AI-assisted section is marked human-reviewed without reviewer metadata."
                && diagnostic
                    .related
                    .iter()
                    .any(|related| related.contains("reviewedAt=missing"))
        }),
        "{:#?}",
        report.diagnostics
    );
    assert!(report
        .manifest
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message
            == "AI-assisted section marker is missing provenance metadata."));
}

#[test]
fn prepare_for_export_reports_invalid_ai_review_statuses() {
    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: AI Status Audit\nversion: 1.0.0\nstatus: archived\n---\n# AI Status Audit\n```ai-source\nprovider: OpenAI\nmodel: ChatGPT\ndate: 2026-05-20\npromptSummary: Board pack outline\nreviewedBy: QA\nreviewedAt: 2026-05-20T10:00:00Z\nstatus: reviewed\n```\n\n<!-- ai-assisted: status=done | reviewedBy=QA | reviewedAt=2026-05-20T10:30:00Z | source=Claude | promptSummary=Drafted summary -->\n## Drafted Section\nReviewed content.\n"
            .to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "includeProvenance": true, "warnOnDirtyGit": false }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 3);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.message == "Invalid AI source review status: reviewed" }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Invalid AI-assisted section review status: done"
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("AI-assisted sections that are not human-reviewed")));
}

#[test]
fn prepare_for_export_reports_missing_caption_labels() {
    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Caption Audit\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# Caption Audit\n<figure class=\"figure\"><img src=\"data:image/svg+xml;base64,PHN2Zy8+\" alt=\"Architecture\"/></figure>\n\n| Metric | Value |\n| --- | ---: |\n| Revenue | 42 |\n\n<figure class=\"equation\"><code>ROI = Gain / Cost</code></figure>\n"
            .to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 3);
    assert_eq!(report.readiness.warning_count, 3);
    assert_eq!(report.manifest.readiness.warning_count, 3);
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Figure is missing a stable label or caption."
            && diagnostic
                .related
                .iter()
                .any(|related| related.contains("label=missing"))
            && diagnostic
                .related
                .iter()
                .any(|related| related.contains("caption=missing"))
    }));
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.message == "Table is missing a stable label or caption." }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Equation is missing a stable label or caption."
    }));
    assert!(report.manifest.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Figure is missing a stable label or caption."
    }));
}

#[test]
fn export_readiness_and_manifest_report_progress_steps() {
    let source = "---\ntitle: Progress Ready\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\nversion: 1.0.0\n---\n# Progress Ready\n\n```chart\ntype: bar\ntitle: Progress data\ndata:\n  - region: East\n    revenue: 42\n  - region: West\n    revenue: 27\nx: region\ny: revenue\n```\n";
    let report = prepare_for_export(PrepareExportRequest {
        text: source.to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    });

    assert!(report.ready, "{:#?}", report.diagnostics);
    assert_eq!(
        report.progress_steps.len(),
        report.manifest.progress_steps.len()
    );
    assert!(report.progress_steps.iter().any(|step| {
        step.id == "transforms"
            && step.state == "complete"
            && step.work_units == 1
            && step.detail.contains("1 transform artifact")
    }));
    assert!(report.progress_steps.iter().any(|step| {
        step.id == "render" && step.state == "pending" && step.label == "Render pdf artifact"
    }));
    assert!(report.progress_steps.iter().any(|step| {
        step.id == "manifest" && step.state == "pending" && step.detail.contains("will be written")
    }));
}

#[test]
fn export_document_blocks_compiler_errors_before_writing() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-export-block-test-{unique}"));
    fs::create_dir_all(&root).expect("create export block dir");
    let output = root.join("broken.pdf");

    let error = export_document(ExportRequest {
        text: "---\ntitle: Broken\nstatus: approved\napprovedBy: QA\n---\n!include missing.md\n"
            .to_string(),
        file_path: Some(path_to_string(&root.join("root.md"))),
        target: "pdf".to_string(),
        output_path: path_to_string(&output),
        options: json!({ "includeManifest": true }),
    })
    .expect_err("compiler errors should block export");

    assert!(error.contains("Export blocked by compiler error"));
    assert!(error.contains("Missing include"));
    assert!(!output.exists());
    assert!(!PathBuf::from(format!("{}.manifest.json", output.display())).exists());
    fs::remove_dir_all(root).expect("clean export block dir");
}

#[test]
fn export_document_blocks_invalid_options_before_writing() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-export-options-test-{unique}"));
    fs::create_dir_all(&root).expect("create export options dir");
    let output = root.join("invalid.pdf");

    let error = export_document(ExportRequest {
            text: "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-19\n---\n# Ready\n"
                .to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
            target: "pdf".to_string(),
            output_path: path_to_string(&output),
            options: json!({ "includeManifest": "yes" }),
        })
        .expect_err("invalid export options should block export");

    assert!(error.contains("Export blocked by validation error"));
    assert!(error.contains("includeManifest must be true or false"));
    assert!(!output.exists());
    assert!(!PathBuf::from(format!("{}.manifest.json", output.display())).exists());
    fs::remove_dir_all(root).expect("clean export options dir");
}

#[test]
fn export_document_writes_optional_sidecar_manifest() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-export-manifest-test-{unique}"));
    fs::create_dir_all(&root).expect("create export manifest dir");
    let output = root.join("ready.html");
    let source =
            "---\ntitle: Manifest Ready\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-19\nversion: 1.0.0\n---\n# Ready\n";

    let response = export_document(ExportRequest {
        text: source.to_string(),
        file_path: Some(path_to_string(&root.join("root.md"))),
        target: "html".to_string(),
        output_path: path_to_string(&output),
        options: json!({ "includeManifest": true }),
    })
    .expect("successful html export");

    let manifest_path = response.manifest_path.expect("manifest path");
    let manifest_text = fs::read_to_string(&manifest_path).expect("manifest file");
    assert!(output.exists());
    assert!(manifest_text.contains("\"document_title\": \"Manifest Ready\""));
    assert!(manifest_text.contains("\"document_version\": \"1.0.0\""));
    assert!(manifest_text.contains("\"export_target\": \"html\""));
    assert!(manifest_text.contains("\"source_hash\": \"sha256:"));
    assert!(manifest_text.contains("\"output_path\": "));
    assert!(manifest_text.contains("\"output_hash\": \"sha256:"));
    assert!(manifest_text.contains("\"readiness\": {"));
    assert!(manifest_text.contains("\"progress_steps\": ["));
    assert!(manifest_text.contains("\"id\": \"render\""));
    assert!(manifest_text.contains("\"state\": \"complete\""));
    assert!(manifest_text.contains("\"ready\": true"));
    assert!(manifest_text.contains("\"error_count\": 0"));
    assert!(manifest_text.contains("\"diagnostics\": []"));
    assert!(manifest_text.contains("\"source_map\": ["));
    assert!(manifest_text.contains("\"layout_sections\": ["));
    assert_eq!(response.manifest.document_title, "Manifest Ready");
    assert_eq!(response.manifest.export_target, "html");
    assert_eq!(response.manifest.layout_sections.len(), 1);
    let output_string = path_to_string(&output);
    assert_eq!(
        response.manifest.output_path.as_deref(),
        Some(output_string.as_str())
    );
    assert!(response
        .manifest
        .output_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("sha256:")));
    assert!(response.manifest.diagnostics.is_empty());
    assert!(response.manifest.readiness.ready);
    assert_eq!(response.manifest.readiness.error_count, 0);
    assert_eq!(response.manifest.readiness.warning_count, 0);
    assert_eq!(
        response.progress_steps.len(),
        response.manifest.progress_steps.len()
    );
    assert!(response.progress_steps.iter().any(|step| {
        step.id == "render"
            && step.state == "complete"
            && step.detail.contains(output_string.as_str())
    }));
    assert!(response
        .progress_steps
        .iter()
        .any(|step| step.id == "manifest" && step.state == "complete"));
    assert!(!response.manifest.source_map.is_empty());

    let docx_output = root.join("ready.docx");
    let docx_response = export_document(ExportRequest {
        text: source.to_string(),
        file_path: Some(path_to_string(&root.join("root.md"))),
        target: "docx".to_string(),
        output_path: path_to_string(&docx_output),
        options: json!({ "includeManifest": true }),
    })
    .expect("successful docx export with manifest");
    let docx_manifest_path = docx_response.manifest_path.expect("docx manifest path");
    let docx_manifest_text = fs::read_to_string(&docx_manifest_path).expect("docx manifest file");
    let docx_bytes = fs::read(&docx_output).expect("docx output bytes");
    assert!(docx_output.exists());
    assert!(docx_bytes.starts_with(b"PK"));
    assert!(zip_has_entry(&docx_bytes, "word/document.xml"));
    assert!(docx_manifest_text.contains("\"export_target\": \"docx\""));
    assert!(docx_manifest_text.contains("\"document_title\": \"Manifest Ready\""));
    assert!(docx_manifest_text.contains("\"output_hash\": \"sha256:"));
    assert_eq!(docx_response.manifest.export_target, "docx");
    assert_eq!(
        docx_response.manifest.output_path.as_deref(),
        Some(path_to_string(&docx_output).as_str())
    );
    assert!(docx_response
        .manifest
        .output_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("sha256:")));

    for (target, extension, expected) in [
        ("pdf", "pdf", "PDF-1.4"),
        ("pptx", "pptx", "PK"),
        ("markdown-bundle", "zip", "PK"),
    ] {
        let target_output = root.join(format!("ready-{target}.{extension}"));
        let target_response = export_document(ExportRequest {
            text: source.to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
            target: target.to_string(),
            output_path: path_to_string(&target_output),
            options: json!({ "includeManifest": true }),
        })
        .unwrap_or_else(|error| panic!("successful {target} export with manifest: {error}"));
        let target_manifest_path = target_response
            .manifest_path
            .as_deref()
            .map(PathBuf::from)
            .expect("target manifest path");
        let target_manifest_text =
            fs::read_to_string(&target_manifest_path).expect("target manifest file");
        let target_bytes = fs::read(&target_output).expect("target output bytes");

        assert!(target_output.exists());
        assert!(String::from_utf8_lossy(&target_bytes).contains(expected));
        assert!(target_manifest_text.contains(&format!("\"export_target\": \"{target}\"")));
        assert!(target_manifest_text.contains("\"document_title\": \"Manifest Ready\""));
        assert!(target_manifest_text.contains("\"output_path\": "));
        assert!(target_manifest_text.contains("\"output_hash\": \"sha256:"));
        assert_eq!(target_response.manifest.export_target, target);
        assert_eq!(
            target_response.manifest.output_path.as_deref(),
            Some(path_to_string(&target_output).as_str())
        );
        assert!(target_response
            .manifest
            .output_hash
            .as_deref()
            .is_some_and(|hash| hash.starts_with("sha256:")));
    }

    let no_manifest_output = root.join("ready-no-manifest.html");
    let no_manifest = export_document(ExportRequest {
        text: source.to_string(),
        file_path: Some(path_to_string(&root.join("root.md"))),
        target: "html".to_string(),
        output_path: path_to_string(&no_manifest_output),
        options: json!({ "includeManifest": false }),
    })
    .expect("successful html export without manifest");
    assert!(no_manifest_output.exists());
    assert!(no_manifest.manifest_path.is_none());
    assert!(!PathBuf::from(format!("{}.manifest.json", no_manifest_output.display())).exists());

    fs::remove_dir_all(root).expect("clean export manifest dir");
}

#[test]
fn export_document_manifest_records_dirty_git_warning() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-export-dirty-git-test-{unique}"));
    fs::create_dir_all(&root).expect("create dirty git export dir");
    run_git(&root, &["init"]).expect("git init");
    run_git(&root, &["config", "user.email", "neditor@example.test"]).expect("git email");
    run_git(&root, &["config", "user.name", "NEditor Test"]).expect("git name");

    let doc = root.join("doc.md");
    let clean_source =
        "---\ntitle: Dirty Git Export\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# Ready\n";
    fs::write(&doc, clean_source).expect("write clean doc");
    run_git(&root, &["add", "doc.md"]).expect("git add doc");
    run_git(&root, &["commit", "-m", "Initial document"]).expect("git commit doc");

    let dirty_source =
        "---\ntitle: Dirty Git Export\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# Ready\n\nUncommitted export content.\n";
    fs::write(&doc, dirty_source).expect("write dirty doc");
    let output = root.join("dirty.html");

    let response = export_document(ExportRequest {
        text: dirty_source.to_string(),
        file_path: Some(path_to_string(&doc)),
        target: "html".to_string(),
        output_path: path_to_string(&output),
        options: json!({ "includeManifest": true }),
    })
    .expect("dirty git export should warn but still write");

    assert!(output.exists());
    assert!(!response.manifest.readiness.ready);
    assert_eq!(response.manifest.readiness.error_count, 0);
    assert_eq!(response.manifest.readiness.warning_count, 1);
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Git working tree is dirty before export")));
    assert!(response
        .manifest
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic
            .message
            .contains("Git working tree is dirty before export")));

    let manifest_path = response.manifest_path.expect("manifest path");
    let manifest_text = fs::read_to_string(&manifest_path).expect("manifest file");
    assert!(manifest_text.contains("\"ready\": false"));
    assert!(manifest_text.contains("\"warning_count\": 1"));
    assert!(manifest_text.contains("Git working tree is dirty before export"));

    let suppressed_output = root.join("suppressed.html");
    let suppressed = export_document(ExportRequest {
        text: dirty_source.to_string(),
        file_path: Some(path_to_string(&doc)),
        target: "html".to_string(),
        output_path: path_to_string(&suppressed_output),
        options: json!({ "includeManifest": false, "warnOnDirtyGit": false }),
    })
    .expect("suppressed dirty git warning export");
    assert!(suppressed.manifest.readiness.ready);
    assert!(suppressed.diagnostics.iter().all(|diagnostic| !diagnostic
        .message
        .contains("Git working tree is dirty before export")));

    fs::remove_dir_all(root).expect("clean dirty git export dir");
}

#[test]
fn prepare_for_export_validates_target_and_options() {
    let ready_report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Ready Layout\nversion: 1.0.0\nstatus: archived\n---\n# Ready Layout\n\n{{section-break columns=2}}\nColumned content.".to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "warnOnDirtyGit": false }),
    });

    assert!(ready_report.ready);
    assert_eq!(ready_report.paged_document.sections.len(), 2);
    assert!(ready_report
        .paged_document
        .sections
        .iter()
        .any(|section| section.layout.columns == Some(2)));

    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\n---\n# Ready".to_string(),
        file_path: None,
        target: "rtf".to_string(),
        options: json!({
            "watermark": 42,
            "includeManifest": "yes",
            "includeStyles": "yes",
            "includeSyntaxHighlighting": "yes",
            "coverPage": "yes",
            "pageNumbers": "yes",
            "layoutPreset": "dense",
            "includeGlossary": "yes",
            "includeComments": "yes",
            "includeProvenance": "yes",
            "includeAgenda": "yes"
        }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 12);
    assert_eq!(report.manifest.export_target, "rtf");
    assert!(report.manifest.output_path.is_none());
    assert!(report.manifest.output_hash.is_none());
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    assert_eq!(report.manifest.source_map.len(), report.source_map.len());
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
        .contains("includeStyles must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("includeSyntaxHighlighting must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("coverPage must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("pageNumbers must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("layoutPreset must be business, compact, or presentation")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("includeGlossary must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("includeComments must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("includeProvenance must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("includeAgenda must be true or false")));
}

#[test]
fn prepare_for_export_validates_transform_engine_options() {
    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\n---\n# Ready".to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({
            "transformTimeoutMs": 50000,
            "transformEnginePaths": { "dot": "dot" },
            "trustedTransformEngines": { "dot": "yes" },
            "transformInputModes": { "dot": "pipe" }
        }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 4);
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("transformTimeoutMs must be between 1 and 30000")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("transformEnginePaths.dot must be an absolute path")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("trustedTransformEngines.dot must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("transformInputModes.dot must be stdin or file")));
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

    let suppressed = prepare_for_export(PrepareExportRequest {
        text: fs::read_to_string(&doc).expect("read doc"),
        file_path: Some(path_to_string(&doc)),
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    });
    assert!(!suppressed.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Git working tree is dirty before export")));
    fs::remove_dir_all(root).expect("clean export git test dir");
}
