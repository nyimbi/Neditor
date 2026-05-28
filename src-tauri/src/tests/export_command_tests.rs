use super::*;
use crate::export_commands::{ExportReadinessReport, ExportResponse};
use std::path::Path;

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
fn desktop_native_command_workflow_smoke_uses_real_files_and_exports() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-desktop-command-smoke-{unique}"));
    let chapters = root.join("chapters");
    let exports = root.join("exports");
    fs::create_dir_all(&chapters).expect("create smoke chapters");
    fs::create_dir_all(&exports).expect("create smoke exports");
    let root_doc = root.join("board-pack.md");
    let summary = chapters.join("summary.md");
    let source = "---\ntitle: Native Workflow Smoke\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21T08:00:00Z\nowner: Release QA\nreleaseTarget: native export smoke\ntoc: true\n---\n# Native Workflow Smoke\n\n!include chapters/summary.md\n\n[TOC]\n\n```calc\nrevenue = 125000\ncost = 74000\nprofit = revenue - cost\nmargin = profit / revenue\n```\n\nExpected margin: {{=margin | percent}}\n\nTable: Budget controls {#tbl:budget}\n| Metric | Value |\n| --- | ---: |\n| Revenue | {{=revenue | currency}} |\n| Cost | {{=cost | currency}} |\n| Profit | {{=profit | currency}} |\n\n```chart\ntype: bar\ntitle: Quarterly Revenue\ndata:\n  - quarter: Q1\n    revenue: 120\n  - quarter: Q2\n    revenue: 148\nx: quarter\ny: revenue\n```\n\n![Architecture](data:image/svg+xml;base64,PHN2Zy8+){#fig:architecture caption=\"Architecture diagram\"}\n\nSee {@tbl:budget} and {@fig:architecture}.\n".to_string();
    fs::write(
        &summary,
        "## Executive Summary\n\nThe native workflow smoke uses real file operations and export commands.\n",
    )
    .expect("write smoke include");

    let saved = save_file_as(SaveFileRequest {
        path: path_to_string(&root_doc),
        text: source.clone(),
        expected_hash: Some("ignored-for-save-as".to_string()),
    })
    .expect("save desktop smoke source");
    assert_eq!(saved.path, path_to_string(&root_doc));
    assert!(saved.text.contains("Native Workflow Smoke"));

    let opened = open_file(path_to_string(&root_doc)).expect("open desktop smoke source");
    assert_eq!(opened.hash, saved.hash);
    assert!(opened.text.contains("!include chapters/summary.md"));

    let watched = watch_file(WatchFileRequest {
        root: path_to_string(&root_doc),
        open_roots: vec![],
        included: vec![],
    })
    .expect("watch desktop smoke source");
    assert!(watched
        .paths
        .iter()
        .any(|path| path.role == "root" && path.path.ends_with("board-pack.md") && path.exists));
    assert!(watched.paths.iter().any(|path| path.role == "include"
        && path.path.ends_with("chapters/summary.md")
        && path.exists));

    let compile_response = compile_document(CompileRequest {
        text: opened.text.clone(),
        file_path: Some(path_to_string(&root_doc)),
    })
    .expect("compile desktop smoke source");
    assert!(!compile_response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(compile_response
        .compiled_markdown
        .contains("Executive Summary"));
    assert!(compile_response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "chart"));

    let readiness = prepare_for_export(PrepareExportRequest {
        text: opened.text.clone(),
        file_path: Some(path_to_string(&root_doc)),
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    });
    assert!(readiness.ready, "{:#?}", readiness.diagnostics);
    assert_eq!(readiness.manifest.document_title, "Native Workflow Smoke");
    assert!(readiness
        .manifest
        .include_graph
        .iter()
        .any(|edge| edge.child.ends_with("chapters/summary.md")));

    for (target, extension) in [
        ("html", "html"),
        ("pdf", "pdf"),
        ("docx", "docx"),
        ("pptx", "pptx"),
        ("markdown-bundle", "zip"),
        ("blog", "zip"),
        ("substack", "zip"),
        ("latex", "tex"),
        ("google-docs", "zip"),
        ("epub", "epub"),
    ] {
        let output_path = exports.join(format!("native-smoke.{extension}"));
        let response = export_document(ExportRequest {
            text: opened.text.clone(),
            file_path: Some(path_to_string(&root_doc)),
            target: target.to_string(),
            output_path: path_to_string(&output_path),
            options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
        })
        .unwrap_or_else(|error| panic!("{target} export should pass: {error}"));
        assert_eq!(response.output_path, path_to_string(&output_path));
        assert_eq!(response.manifest.export_target, target);
        assert!(response.manifest.output_hash.is_some());
        assert!(output_path.exists(), "{target} output should exist");
        assert!(
            response
                .manifest_path
                .as_deref()
                .is_some_and(|path| Path::new(path).exists()),
            "{target} sidecar manifest should exist"
        );
        assert!(response
            .progress_steps
            .iter()
            .any(|step| step.id == "render" && step.state == "complete"));
    }

    let reveal = crate::filesystem::reveal_command_for_path(path_to_string(&root_doc).as_str())
        .expect("desktop smoke reveal command");
    assert!(!reveal.program.is_empty());
    assert!(!reveal.args.is_empty());

    fs::remove_dir_all(root).expect("clean desktop command smoke");
}

#[test]
fn export_document_writes_blog_and_substack_publish_packages() {
    let source = "---\ntitle: Board Notes\nsubtitle: Weekly operating summary\nslug: board-notes-weekly\nauthor: NEditor QA\ndate: 2026-05-21\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\nowner: Publishing Ops\nreleaseTarget: executive newsletter\ntags:\n  - strategy\n  - operations\ncanonicalUrl: https://example.com/board-notes\n---\n# Board Notes\n\nA copy-ready post with **business** context.\n".to_string();

    for target in ["blog", "substack"] {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let output = std::env::temp_dir().join(format!("neditor-{target}-{unique}.zip"));
        let response = export_document(ExportRequest {
            text: source.clone(),
            file_path: None,
            target: target.to_string(),
            output_path: path_to_string(&output),
            options: json!({ "warnOnDirtyGit": false, "includeManifest": false }),
        })
        .unwrap_or_else(|error| panic!("{target} publish package export should pass: {error}"));
        let bytes = fs::read(&output).expect("publish package bytes");

        assert_eq!(response.manifest.export_target, target);
        assert!(response.manifest_path.is_none());
        assert!(zip_has_entry(&bytes, "post.md"));
        assert!(zip_has_entry(&bytes, "post.html"));
        assert!(zip_has_entry(&bytes, "substack-copy.html"));
        assert!(zip_has_entry(&bytes, "post.txt"));
        assert!(zip_has_entry(&bytes, "metadata.json"));
        assert!(zip_has_entry(&bytes, "manifest.json"));
        assert!(zip_has_entry(&bytes, "rss-item.xml"));
        assert!(zip_has_entry(&bytes, "README.md"));

        let metadata = zip_entry_text(&bytes, "metadata.json");
        assert!(metadata.contains("\"slug\": \"board-notes-weekly\""));
        assert!(metadata.contains("\"exportTarget\": "));
        assert!(metadata.contains("\"packageType\": \"publishing-handoff\""));
        assert!(metadata.contains("\"primaryPublishFile\": "));
        assert!(metadata.contains("\"publishingSteps\": "));
        assert!(metadata.contains("\"strategy\""));
        assert!(metadata.contains("https://example.com/board-notes"));
        let readme = zip_entry_text(&bytes, "README.md");
        assert!(readme.contains("## Publish Workflow"));
        assert!(readme.contains("Keep `manifest.json` with the published record"));
        let copy_html = zip_entry_text(&bytes, "substack-copy.html");
        assert!(copy_html.contains("<h1>Board Notes</h1>"));
        assert!(copy_html.contains("<strong>business</strong>"));
        let manifest = zip_entry_text(&bytes, "manifest.json");
        assert!(manifest.contains(&format!("\"export_target\": \"{target}\"")));
        assert!(response.progress_steps.iter().any(|step| {
            step.id == "manifest"
                && step.label == "Embed package manifest"
                && step.detail.contains("sidecar manifest output is disabled")
        }));

        fs::remove_file(output).expect("clean publish package");
    }
}

#[test]
fn export_document_applies_public_metadata_options_to_publish_packages() {
    let source = "---\ntitle: Metadata Options\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\nowner: Publishing Ops\nreleaseTarget: public site\ntags:\n  - launch\n---\n# Metadata Options\n\nPublic package body.\n".to_string();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let output = std::env::temp_dir().join(format!("neditor-public-metadata-{unique}.zip"));
    let response = export_document(ExportRequest {
        text: source,
        file_path: None,
        target: "blog".to_string(),
        output_path: path_to_string(&output),
        options: json!({
            "warnOnDirtyGit": false,
            "includeManifest": false,
            "canonicalUrl": "https://example.com/metadata-options",
            "htmlDescription": "A concise public preview for publishing.",
            "language": "en-GB"
        }),
    })
    .expect("blog export with public metadata options should pass");
    let bytes = fs::read(&output).expect("publish package bytes");

    assert_eq!(response.manifest.export_target, "blog");
    let metadata = zip_entry_text(&bytes, "metadata.json");
    assert!(metadata.contains("\"canonicalUrl\": \"https://example.com/metadata-options\""));
    assert!(metadata.contains("\"description\": \"A concise public preview for publishing.\""));
    assert!(metadata.contains("\"language\": \"en-GB\""));
    let post_html = zip_entry_text(&bytes, "post.html");
    assert!(post_html.contains(r#"<html lang="en-GB">"#));
    assert!(post_html.contains(
        r#"<meta name="description" content="A concise public preview for publishing.">"#
    ));
    assert!(
        post_html.contains(r#"<link rel="canonical" href="https://example.com/metadata-options">"#)
    );
    let rss = zip_entry_text(&bytes, "rss-item.xml");
    assert!(rss.contains("<link>https://example.com/metadata-options</link>"));
    assert!(rss.contains("A concise public preview for publishing."));

    fs::remove_file(output).expect("clean publish package");
}

#[test]
fn export_document_writes_latex_and_google_docs_outputs() {
    let source = "---\ntitle: Research Brief\nsubtitle: Import-ready evidence pack\nauthor: NEditor QA\ndate: 2026-05-21\nversion: 2.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\nowner: Research Ops\nreleaseTarget: Google Docs review room\n---\n# Research Brief\n\nA **business** brief with a [source link](https://example.com/evidence).\n\nTable: Controls {#tbl:controls}\n| Control | Owner |\n| --- | --- |\n| Review | Operations |\n\nSee [Table controls](#tbl:controls).\n\n$$\nROI = \\frac{Gain}{Cost}\n$$ {#eq:roi caption=\"Return on investment\"}\n".to_string();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();

    let latex_output = std::env::temp_dir().join(format!("neditor-latex-{unique}.tex"));
    let latex_response = export_document(ExportRequest {
        text: source.clone(),
        file_path: None,
        target: "latex".to_string(),
        output_path: path_to_string(&latex_output),
        options: json!({ "warnOnDirtyGit": false, "includeManifest": false }),
    })
    .expect("latex export should pass");
    let latex = fs::read_to_string(&latex_output).expect("latex output");
    assert_eq!(latex_response.manifest.export_target, "latex");
    assert!(latex_response.manifest_path.is_none());
    assert!(latex.contains("\\documentclass"));
    assert!(latex.contains("\\title{Research Brief}"));
    assert!(latex.contains("\\section{Research Brief}"));
    assert!(latex.contains("\\textbf{business}"));
    assert!(latex.contains("\\href{https://example.com/evidence}{source link}"));
    assert!(latex.contains("\\hyperref[tbl:controls]{Table controls}"));
    assert!(latex.contains("\\begin{longtable}"));
    assert!(latex.contains("\\label{tbl:controls}"));
    assert!(latex.contains("ROI = \\frac{Gain}{Cost}"));
    assert!(latex.contains("\\label{eq:roi}"));

    let google_docs_output = std::env::temp_dir().join(format!("neditor-google-docs-{unique}.zip"));
    let google_docs_response = export_document(ExportRequest {
        text: source,
        file_path: None,
        target: "google-docs".to_string(),
        output_path: path_to_string(&google_docs_output),
        options: json!({ "warnOnDirtyGit": false, "includeManifest": false }),
    })
    .expect("google docs package export should pass");
    let bytes = fs::read(&google_docs_output).expect("google docs package");
    assert_eq!(google_docs_response.manifest.export_target, "google-docs");
    assert!(google_docs_response.manifest_path.is_none());
    assert!(zip_has_entry(&bytes, "document.docx"));
    assert!(zip_has_entry(&bytes, "document.html"));
    assert!(zip_has_entry(&bytes, "document.md"));
    assert!(zip_has_entry(&bytes, "document.txt"));
    assert!(zip_has_entry(&bytes, "metadata.json"));
    assert!(zip_has_entry(&bytes, "manifest.json"));
    assert!(zip_has_entry(&bytes, "README.md"));
    assert!(zip_entry_text(&bytes, "metadata.json").contains("\"exportTarget\": \"google-docs\""));
    assert!(zip_entry_text(&bytes, "metadata.json")
        .contains("\"packageType\": \"google-docs-import-handoff\""));
    assert!(zip_entry_text(&bytes, "metadata.json")
        .contains("\"primaryImportFile\": \"document.docx\""));
    assert!(zip_entry_text(&bytes, "metadata.json").contains("\"importSteps\": "));
    assert!(zip_entry_text(&bytes, "metadata.json").contains("Upload document.docx to Google Docs"));
    assert!(zip_entry_text(&bytes, "manifest.json").contains("\"export_target\": \"google-docs\""));
    assert!(zip_entry_text(&bytes, "README.md").contains("## Import Workflow"));
    assert!(zip_entry_text(&bytes, "README.md").contains("Open it with Google Docs"));
    assert!(google_docs_response.progress_steps.iter().any(|step| {
        step.id == "manifest"
            && step.label == "Embed package manifest"
            && step.detail.contains("sidecar manifest output is disabled")
    }));

    fs::remove_file(latex_output).expect("clean latex export");
    fs::remove_file(google_docs_output).expect("clean google docs package");
}

#[test]
fn export_document_writes_epub_package() {
    let source = "---\ntitle: Ebook Brief\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\nowner: Publishing\nreleaseTarget: EPUB readers\n---\n# Ebook Brief\n\nA portable **ebook** with a [source link](https://example.com).\n\n## Scoring\n\n$$ {#eq:score caption=\"Weighted score\"}\nScore = \\sum w_i x_i\n$$\n".to_string();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let output = std::env::temp_dir().join(format!("neditor-epub-{unique}.epub"));
    let response = export_document(ExportRequest {
        text: source,
        file_path: None,
        target: "epub".to_string(),
        output_path: path_to_string(&output),
        options: json!({ "warnOnDirtyGit": false, "includeManifest": false, "language": "fr-CA" }),
    })
    .expect("epub export should pass");
    let bytes = fs::read(&output).expect("epub output");
    assert_eq!(response.manifest.export_target, "epub");
    assert!(response.manifest_path.is_none());
    assert!(zip_has_entry(&bytes, "mimetype"));
    assert_eq!(zip_entry_text(&bytes, "mimetype"), "application/epub+zip");
    assert!(zip_has_entry(&bytes, "META-INF/container.xml"));
    assert!(zip_has_entry(&bytes, "OEBPS/content.opf"));
    assert!(zip_has_entry(&bytes, "OEBPS/nav.xhtml"));
    assert!(zip_has_entry(&bytes, "OEBPS/document.xhtml"));
    assert!(zip_has_entry(&bytes, "OEBPS/styles/neditor.css"));
    assert!(zip_has_entry(&bytes, "OEBPS/metadata/manifest.json"));
    assert!(
        zip_entry_text(&bytes, "META-INF/container.xml").contains("application/oebps-package+xml")
    );
    assert!(
        zip_entry_text(&bytes, "OEBPS/content.opf").contains("<dc:title>Ebook Brief</dc:title>")
    );
    assert!(
        zip_entry_text(&bytes, "OEBPS/content.opf").contains("<dc:language>fr-CA</dc:language>")
    );
    assert!(zip_entry_text(&bytes, "OEBPS/nav.xhtml").contains(r#"lang="fr-CA""#));
    assert!(zip_entry_text(&bytes, "OEBPS/document.xhtml").contains(r#"lang="fr-CA""#));
    assert!(zip_entry_text(&bytes, "OEBPS/nav.xhtml").contains("Ebook Brief"));
    assert!(zip_entry_text(&bytes, "OEBPS/document.xhtml").contains("Score"));
    assert!(zip_entry_text(&bytes, "OEBPS/metadata/manifest.json")
        .contains("\"export_target\": \"epub\""));
    assert!(response.progress_steps.iter().any(|step| {
        step.id == "manifest"
            && step.label == "Embed package manifest"
            && step.detail.contains("sidecar manifest output is disabled")
    }));
    fs::remove_file(output).expect("clean epub package");
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
                && diagnostic.column == Some(1)
                && diagnostic.end_column > diagnostic.column
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
                && diagnostic.column == Some(1)
                && diagnostic.end_column > diagnostic.column
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
fn prepare_for_export_reports_missing_citation_sources_with_precise_ranges() {
    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Citation Audit\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# Citation Audit\nClaim [@missing2026, p. 4; @other2026].\nRepeated [@missing2026].\n"
            .to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 3);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Document contains citations but no bibliography source."
            && diagnostic.source_file.as_deref() == Some("untitled.md")
            && diagnostic.line == Some(9)
            && diagnostic.column == Some(8)
            && diagnostic.end_line == Some(9)
            && diagnostic.end_column == Some(20)
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Missing citation bibliography entry: missing2026"
            && diagnostic.source_file.as_deref() == Some("untitled.md")
            && diagnostic.line == Some(9)
            && diagnostic.column == Some(8)
            && diagnostic.end_line == Some(9)
            && diagnostic.end_column == Some(20)
            && diagnostic
                .related
                .iter()
                .any(|related| related.contains("@missing2026"))
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Missing citation bibliography entry: other2026"
            && diagnostic.source_file.as_deref() == Some("untitled.md")
            && diagnostic.line == Some(9)
            && diagnostic.column == Some(28)
            && diagnostic.end_line == Some(9)
            && diagnostic.end_column == Some(38)
    }));
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    assert!(report.manifest.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Missing citation bibliography entry: missing2026"
    }));
}

#[test]
fn prepare_for_export_blocks_duplicate_reference_labels_in_manifest() {
    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Duplicate Reference Labels\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# Strategy {#sec:strategy}\n\n![Duplicate](data:image/svg+xml;base64,PHN2Zy8+){#sec:strategy caption=\"Duplicate\"}\n\nSee {@sec:strategy}.\n"
            .to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 1, "{:#?}", report.diagnostics);
    assert_eq!(report.manifest.readiness.error_count, 1);
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    let duplicate = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message == "Duplicate reference label: sec:strategy")
        .expect("duplicate label diagnostic");
    assert_eq!(duplicate.severity, "error");
    assert_eq!(duplicate.source_file.as_deref(), Some("untitled.md"));
    assert_eq!(duplicate.line, Some(10));
    assert!(duplicate
        .related
        .iter()
        .any(|related| related == "First occurrence: untitled.md:8"));
    assert!(report.manifest.diagnostics.iter().any(|diagnostic| {
        diagnostic.message == "Duplicate reference label: sec:strategy"
            && diagnostic.suggestion.as_deref()
                == Some("Rename one label so cross references resolve to one stable target.")
    }));
}

#[test]
fn prepare_for_export_blocks_malformed_reference_markers_in_manifest() {
    let report = prepare_for_export(PrepareExportRequest {
            text: "---\ntitle: Malformed Reference Markers\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\n---\n# Strategy {#sec:bad label}\n\nSee {@sec:bad label}.\n".to_string(),
            file_path: None,
            target: "pdf".to_string(),
            options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
        });

    assert!(!report.ready);
    assert_eq!(report.error_count, 2, "{:#?}", report.diagnostics);
    assert_eq!(report.manifest.readiness.error_count, 2);
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    for expected in [
        "Malformed reference label: sec:bad label",
        "Malformed reference cross reference: sec:bad label",
    ] {
        let diagnostic = report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message == expected)
            .unwrap_or_else(|| panic!("missing diagnostic: {expected}\n{:#?}", report.diagnostics));
        assert_eq!(diagnostic.severity, "error");
        assert_eq!(diagnostic.source_file.as_deref(), Some("untitled.md"));
        assert_eq!(
            diagnostic.suggestion.as_deref(),
            Some(
                "Use only letters, numbers, colon, underscore, dash, or period in reference keys."
            )
        );
        assert!(report
            .manifest
            .diagnostics
            .iter()
            .any(|manifest_diagnostic| manifest_diagnostic.message == expected));
    }
}

#[test]
fn prepare_for_export_reports_empty_generated_reference_sections() {
    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Empty Reference Sections\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20\ntoc:\n  enabled: true\nindex: true\nglossarySection: true\ncaptionLists:\n  figures: true\n  tables: true\n---\nplain text only.\n\n[BIBLIOGRAPHY]\n"
            .to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 6, "{:#?}", report.diagnostics);
    assert_eq!(report.manifest.readiness.warning_count, 6);
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    assert_readiness_contains(
        &report,
        "Generated table of contents was requested but no headings were found.",
    );
    assert_readiness_contains(
        &report,
        "Generated index was requested but no index terms were found.",
    );
    assert_readiness_contains(
        &report,
        "Generated bibliography was requested but no bibliography entries were found.",
    );
    assert_readiness_contains(
        &report,
        "Generated glossary was requested but no glossary entries were found.",
    );
    assert_readiness_contains(
        &report,
        "Generated list of figures was requested but no figures were found.",
    );
    assert_readiness_contains(
        &report,
        "Generated list of tables was requested but no tables were found.",
    );
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .related
        .iter()
        .any(|related| related == "headings: 0")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .related
        .iter()
        .any(|related| related == "index terms: 0")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .related
        .iter()
        .any(|related| related == "bibliography entries: 0")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .related
        .iter()
        .any(|related| related == "glossary entries: 0")));
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
fn export_document_blocks_target_extension_mismatches_before_writing() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-export-extension-test-{unique}"));
    fs::create_dir_all(&root).expect("create export extension dir");
    let output = root.join("board-deck.pdf");

    let error = export_document(ExportRequest {
        text: "---\ntitle: Board Deck\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\nowner: Release QA\nreleaseTarget: board deck\n---\n# Board Deck\n".to_string(),
        file_path: None,
        target: "pptx".to_string(),
        output_path: path_to_string(&output),
        options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
    })
    .expect_err("mismatched target extension should block export");

    assert!(
        error.contains("PPTX export target must write to .pptx files"),
        "{error}"
    );
    assert!(!output.exists());
    assert!(!PathBuf::from(format!("{}.manifest.json", output.display())).exists());
    fs::remove_dir_all(root).expect("clean export extension dir");
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
            "---\ntitle: Manifest Ready\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-19\nowner: Release QA\nreleaseTarget: manifest smoke\nversion: 1.0.0\n---\n# Ready\n";

    let response = export_document(ExportRequest {
        text: source.to_string(),
        file_path: Some(path_to_string(&root.join("root.md"))),
        target: "html".to_string(),
        output_path: path_to_string(&output),
        options: json!({ "includeManifest": true }),
    })
    .expect("successful html export");

    let manifest_path = response.manifest_path.as_deref().expect("manifest path");
    let manifest_text = fs::read_to_string(manifest_path).expect("manifest file");
    let output_bytes = fs::read(&output).expect("html output bytes");
    assert!(output.exists());
    assert!(manifest_text.contains("\"document_title\": \"Manifest Ready\""));
    assert!(manifest_text.contains("\"document_version\": \"1.0.0\""));
    assert!(manifest_text.contains("\"approved_by\": \"QA\""));
    assert!(manifest_text.contains("\"approved_at\": \"2026-05-19\""));
    assert!(manifest_text.contains("\"owner\": \"Release QA\""));
    assert!(manifest_text.contains("\"release_target\": \"manifest smoke\""));
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
    assert_eq!(response.manifest.approved_by.as_deref(), Some("QA"));
    assert_eq!(response.manifest.approved_at.as_deref(), Some("2026-05-19"));
    assert_eq!(response.manifest.owner.as_deref(), Some("Release QA"));
    assert_eq!(
        response.manifest.release_target.as_deref(),
        Some("manifest smoke")
    );
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
    assert_export_manifest_matches_response(&manifest_text, &response, &output, &output_bytes);
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
    let docx_manifest_path = docx_response
        .manifest_path
        .as_deref()
        .expect("docx manifest path");
    let docx_manifest_text = fs::read_to_string(docx_manifest_path).expect("docx manifest file");
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
    assert_export_manifest_matches_response(
        &docx_manifest_text,
        &docx_response,
        &docx_output,
        &docx_bytes,
    );

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
        assert_export_manifest_matches_response(
            &target_manifest_text,
            &target_response,
            &target_output,
            &target_bytes,
        );
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
    assert_eq!(no_manifest.manifest.readiness.info_count, 1);
    assert!(no_manifest.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("sidecar audit manifest for this export target")
    }));

    let bundle_without_sidecar_output = root.join("ready-no-sidecar-bundle.zip");
    let bundle_without_sidecar = export_document(ExportRequest {
        text: source.to_string(),
        file_path: Some(path_to_string(&root.join("root.md"))),
        target: "markdown-bundle".to_string(),
        output_path: path_to_string(&bundle_without_sidecar_output),
        options: json!({ "includeManifest": false, "warnOnDirtyGit": false }),
    })
    .expect("successful markdown bundle export without sidecar manifest");
    let bundle_without_sidecar_bytes =
        fs::read(&bundle_without_sidecar_output).expect("markdown bundle output bytes");
    let embedded_manifest = zip_entry_text(&bundle_without_sidecar_bytes, "manifest.json");
    assert!(bundle_without_sidecar_output.exists());
    assert!(bundle_without_sidecar.manifest_path.is_none());
    assert!(!PathBuf::from(format!(
        "{}.manifest.json",
        bundle_without_sidecar_output.display()
    ))
    .exists());
    assert!(zip_has_entry(&bundle_without_sidecar_bytes, "document.md"));
    assert!(embedded_manifest.contains("\"export_target\": \"markdown-bundle\""));
    assert!(embedded_manifest.contains("\"includeManifest\": false"));
    assert!(embedded_manifest.contains("\"output_hash\": null"));
    assert_eq!(
        bundle_without_sidecar.manifest.output_path.as_deref(),
        Some(path_to_string(&bundle_without_sidecar_output).as_str())
    );
    assert!(bundle_without_sidecar
        .manifest
        .output_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("sha256:")));
    assert_eq!(bundle_without_sidecar.manifest.readiness.info_count, 1);
    assert!(bundle_without_sidecar.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("package exports still embed manifest.json")
    }));
    assert!(bundle_without_sidecar.progress_steps.iter().any(|step| {
        step.id == "manifest"
            && step.label == "Embed package manifest"
            && step.state == "complete"
            && step.detail.contains("sidecar manifest output is disabled")
    }));

    fs::remove_dir_all(root).expect("clean export manifest dir");
}

fn assert_export_manifest_matches_response(
    manifest_text: &str,
    response: &ExportResponse,
    output_path: &Path,
    output_bytes: &[u8],
) {
    let sidecar_manifest: Value =
        serde_json::from_str(manifest_text).expect("sidecar manifest json");
    let response_manifest =
        serde_json::to_value(&response.manifest).expect("response manifest json");
    assert_eq!(sidecar_manifest, response_manifest);
    assert_eq!(response.output_path, path_to_string(output_path));
    assert_eq!(
        response.manifest.output_path.as_deref(),
        Some(path_to_string(output_path).as_str())
    );
    assert_eq!(
        response.manifest.output_hash.as_deref(),
        Some(sha256_uri(output_bytes).as_str())
    );
    assert_eq!(
        sidecar_manifest.get("output_hash").and_then(Value::as_str),
        Some(sha256_uri(output_bytes).as_str())
    );
    assert_eq!(
        sidecar_manifest
            .get("readiness")
            .and_then(|readiness| readiness.get("ready"))
            .and_then(Value::as_bool),
        Some(response.manifest.readiness.ready)
    );
    assert!(response
        .manifest
        .progress_steps
        .iter()
        .any(|step| step.id == "render" && step.state == "complete"));
    assert!(response
        .manifest
        .progress_steps
        .iter()
        .any(|step| step.id == "manifest" && step.state == "complete"));
    assert_eq!(
        serde_json::to_value(&response.progress_steps).expect("response progress json"),
        serde_json::to_value(&response.manifest.progress_steps).expect("manifest progress json")
    );
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
fn prepare_for_export_validates_brand_and_default_style_options() {
    let source = "---\ntitle: Branded Export\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\n---\n# Branded Export\n".to_string();
    let valid = prepare_for_export(PrepareExportRequest {
        text: source.clone(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({
            "warnOnDirtyGit": false,
            "brandColor": "#123ABC",
            "defaultCitationStyle": "apa",
            "includeCoverPage": true,
            "includePageNumbers": true,
            "defaultBrandProfile": {
                "name": "Acme",
                "color": "#275DA8",
                "logo": "data:image/svg+xml;base64,PHN2Zy8+",
                "font": "Inter",
                "header": "{{title}}",
                "footer": "Page {{page}} of {{pages}}",
                "watermark": "INTERNAL",
                "legalDisclaimer": "Internal only."
            }
        }),
    });
    assert!(valid.ready, "{:#?}", valid.diagnostics);

    let report = prepare_for_export(PrepareExportRequest {
        text: source,
        file_path: None,
        target: "pdf".to_string(),
        options: json!({
            "warnOnDirtyGit": "no",
            "brandColor": "blue",
            "defaultCitationStyle": "experimental-csl-style",
            "includeCoverPage": "yes",
            "includePageNumbers": "yes",
            "defaultBrandProfile": {
                "color": "blue",
                "logo": 42,
                "header": false
            }
        }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 8, "{:#?}", report.diagnostics);
    for expected in [
        "brandColor must be a hex color",
        "defaultCitationStyle must be a supported citation style",
        "defaultBrandProfile.logo must be a string",
        "defaultBrandProfile.header must be a string",
        "defaultBrandProfile.color must be a hex color",
        "warnOnDirtyGit must be true or false",
        "includeCoverPage must be true or false",
        "includePageNumbers must be true or false",
    ] {
        assert_readiness_contains(&report, expected);
    }
}

#[test]
fn prepare_for_export_reports_target_specific_release_metadata_blockers() {
    let draft_presentation =
        "---\ntitle: Board Deck\nversion: 1.0.0\nstatus: in-review\n---\n# Board Deck\n"
            .to_string();
    for target in ["pptx", "blog", "substack", "google-docs", "epub"] {
        let report = prepare_for_export(PrepareExportRequest {
            text: draft_presentation.clone(),
            file_path: None,
            target: target.to_string(),
            options: json!({ "warnOnDirtyGit": false }),
        });

        assert!(!report.ready, "{target} should require release metadata");
        assert_eq!(report.error_count, 1, "{target}: {:#?}", report.diagnostics);
        let expected_warning_count = match target {
            "blog" | "substack" => 2,
            "epub" => 1,
            _ => 0,
        };
        assert_eq!(
            report.warning_count, expected_warning_count,
            "{target}: {:#?}",
            report.diagnostics
        );
        let diagnostic = report
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic
                    .message
                    .contains("requires release approval metadata")
            })
            .unwrap_or_else(|| panic!("{target} readiness diagnostic"));
        assert_eq!(diagnostic.severity, "error");
        assert!(diagnostic.suggestion.as_deref().is_some_and(|suggestion| {
            suggestion.contains("approvedBy or reviewer")
                && suggestion.contains("owner")
                && suggestion.contains("releaseTarget")
        }));
        assert!(diagnostic
            .related
            .iter()
            .any(|item| item == &format!("target:{target}")));
        assert!(diagnostic
            .related
            .iter()
            .any(|item| item == "status:in-review"));
        assert!(diagnostic
            .related
            .iter()
            .any(|item| item == "missing:approvedBy-or-reviewer"));
        assert!(diagnostic
            .related
            .iter()
            .any(|item| item == "missing:approvedAt"));
        assert!(diagnostic
            .related
            .iter()
            .any(|item| item == "missing:owner"));
        assert!(diagnostic
            .related
            .iter()
            .any(|item| item == "missing:releaseTarget"));
        assert_eq!(report.manifest.readiness.error_count, 1);
        assert!(!report.manifest.readiness.ready);
    }

    let pdf_report = prepare_for_export(PrepareExportRequest {
        text: draft_presentation,
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "warnOnDirtyGit": false }),
    });
    assert!(pdf_report.ready, "{:#?}", pdf_report.diagnostics);
}

#[test]
fn prepare_for_export_accepts_case_insensitive_release_statuses() {
    let source = "---\ntitle: Published Deck\nversion: 1.0.0\nstatus: PUBLISHED\napprovedBy: QA\napprovedAt: 2026-05-21\nowner: Publishing Ops\nreleaseTarget: board distribution\n---\n# Published Deck\n".to_string();
    let report = prepare_for_export(PrepareExportRequest {
        text: source,
        file_path: None,
        target: "pptx".to_string(),
        options: json!({ "warnOnDirtyGit": false }),
    });

    assert!(report.ready, "{:#?}", report.diagnostics);
    assert_eq!(report.error_count, 0, "{:#?}", report.diagnostics);
    assert!(!report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("requires release approval metadata")));
    assert!(report.manifest.readiness.ready);
}

#[test]
fn prepare_for_export_validates_public_distribution_metadata() {
    let source = "---\ntitle: Public Metadata\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\nowner: Publishing Ops\nreleaseTarget: public site\nsummary: Ready for public distribution.\ntags: strategy, operations\ncanonicalUrl: draft/public-metadata\nlanguage: en_US\n---\n# Public Metadata\n".to_string();
    let report = prepare_for_export(PrepareExportRequest {
        text: source,
        file_path: None,
        target: "blog".to_string(),
        options: json!({ "warnOnDirtyGit": false }),
    });

    assert!(!report.ready, "{:#?}", report.diagnostics);
    assert_eq!(report.error_count, 2, "{:#?}", report.diagnostics);
    assert_eq!(report.warning_count, 0, "{:#?}", report.diagnostics);
    assert_readiness_contains(&report, "canonicalUrl metadata must be an absolute");
    assert_readiness_contains(
        &report,
        "language metadata must be a valid BCP-47-style language tag",
    );
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .related
        .iter()
        .any(|item| item == "metadata:canonicalUrl")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .related
        .iter()
        .any(|item| item == "metadata:language")));
    assert_eq!(report.manifest.readiness.error_count, 2);
}

#[test]
fn prepare_for_export_reports_public_distribution_metadata_warnings() {
    let source = "---\ntitle: Publishing Metadata\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\nowner: Publishing Ops\nreleaseTarget: executive newsletter\ncanonicalUrl: https://example.com/publishing-metadata\nlanguage: en-US\n---\n# Publishing Metadata\n".to_string();
    let report = prepare_for_export(PrepareExportRequest {
        text: source,
        file_path: None,
        target: "substack".to_string(),
        options: json!({ "warnOnDirtyGit": false }),
    });

    assert!(!report.ready, "{:#?}", report.diagnostics);
    assert_eq!(report.error_count, 0, "{:#?}", report.diagnostics);
    assert_eq!(report.warning_count, 2, "{:#?}", report.diagnostics);
    assert_readiness_contains(&report, "Publishing exports should include a description");
    assert_readiness_contains(
        &report,
        "Publishing exports should include tags or keywords",
    );
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .related
        .iter()
        .any(|item| item == "missing:description-or-excerpt")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .related
        .iter()
        .any(|item| item == "missing:tags-or-keywords")));
    assert_eq!(report.manifest.readiness.warning_count, 2);
}

#[test]
fn prepare_for_export_validates_public_distribution_options() {
    let source = "---\ntitle: Option Metadata\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\n---\n# Option Metadata\n".to_string();
    let report = prepare_for_export(PrepareExportRequest {
        text: source,
        file_path: None,
        target: "html".to_string(),
        options: json!({
            "warnOnDirtyGit": false,
            "canonicalUrl": "example.com/not-absolute",
            "htmlLanguage": "en US",
            "language": 42,
            "htmlDescription": false
        }),
    });

    assert!(!report.ready, "{:#?}", report.diagnostics);
    assert_eq!(report.error_count, 4, "{:#?}", report.diagnostics);
    assert_readiness_contains(&report, "canonicalUrl must be an absolute");
    assert_readiness_contains(
        &report,
        "htmlLanguage must be a valid BCP-47-style language tag",
    );
    assert_readiness_contains(&report, "language must be a string");
    assert_readiness_contains(&report, "htmlDescription must be a string");
}

#[test]
fn prepare_for_export_reports_target_specific_option_info() {
    let source = "---\ntitle: Option Audit\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\n---\n# Option Audit\n".to_string();
    let pdf_report = prepare_for_export(PrepareExportRequest {
        text: source.clone(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({ "warnOnDirtyGit": false, "includeAgenda": true }),
    });

    assert!(pdf_report.ready, "{:#?}", pdf_report.diagnostics);
    assert_eq!(pdf_report.error_count, 0);
    assert_eq!(pdf_report.warning_count, 0);
    assert_eq!(pdf_report.info_count, 1);
    assert_eq!(pdf_report.manifest.readiness.info_count, 1);
    let agenda_diagnostic = pdf_report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("includeAgenda is only used"))
        .expect("agenda target option diagnostic");
    assert_eq!(agenda_diagnostic.severity, "info");
    assert!(agenda_diagnostic
        .related
        .iter()
        .any(|item| item == "target:pdf"));
    assert!(agenda_diagnostic
        .related
        .iter()
        .any(|item| item == "option:includeAgenda"));

    let html_report = prepare_for_export(PrepareExportRequest {
        text: source.clone(),
        file_path: None,
        target: "html".to_string(),
        options: json!({ "warnOnDirtyGit": false, "includeManifest": false }),
    });

    assert!(html_report.ready, "{:#?}", html_report.diagnostics);
    assert_eq!(html_report.error_count, 0);
    assert_eq!(html_report.warning_count, 0);
    assert_eq!(html_report.info_count, 1);
    assert_eq!(html_report.manifest.readiness.info_count, 1);
    let manifest_diagnostic = html_report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("sidecar audit manifest"))
        .expect("sidecar manifest target option diagnostic");
    assert_eq!(manifest_diagnostic.severity, "info");
    assert!(manifest_diagnostic
        .related
        .iter()
        .any(|item| item == "target:html"));
    assert!(manifest_diagnostic
        .related
        .iter()
        .any(|item| item == "option:includeManifest"));

    let bundle_report = prepare_for_export(PrepareExportRequest {
        text: source,
        file_path: None,
        target: "markdown-bundle".to_string(),
        options: json!({
            "warnOnDirtyGit": false,
            "includeAgenda": true,
            "includeStyles": true,
            "includeSyntaxHighlighting": true,
            "coverPage": true,
            "pageNumbers": true
        }),
    });

    assert!(bundle_report.ready, "{:#?}", bundle_report.diagnostics);
    assert_eq!(bundle_report.error_count, 0);
    assert_eq!(bundle_report.warning_count, 0);
    assert_eq!(bundle_report.info_count, 5);
    assert_eq!(bundle_report.manifest.readiness.info_count, 5);
    for option in [
        "includeAgenda",
        "includeStyles",
        "includeSyntaxHighlighting",
        "coverPage",
        "pageNumbers",
    ] {
        assert!(
            bundle_report.diagnostics.iter().any(|diagnostic| diagnostic
                .related
                .iter()
                .any(|item| item == &format!("option:{option}"))),
            "missing info diagnostic for {option}: {:#?}",
            bundle_report.diagnostics
        );
    }
}

#[test]
fn prepare_for_export_reports_markdown_bundle_manifest_sidecar_info() {
    let source = "---\ntitle: Bundle Manifest Audit\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\n---\n# Bundle Manifest Audit\n".to_string();
    let report = prepare_for_export(PrepareExportRequest {
        text: source,
        file_path: None,
        target: "markdown-bundle".to_string(),
        options: json!({ "warnOnDirtyGit": false, "includeManifest": false }),
    });

    assert!(report.ready, "{:#?}", report.diagnostics);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 0);
    assert_eq!(report.info_count, 1);
    assert_eq!(report.manifest.readiness.info_count, 1);
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("sidecar manifest"))
        .expect("markdown bundle sidecar diagnostic");
    assert_eq!(diagnostic.severity, "info");
    assert!(diagnostic
        .related
        .iter()
        .any(|item| item == "target:markdown-bundle"));
    assert!(diagnostic
        .related
        .iter()
        .any(|item| item == "option:includeManifest"));
    assert!(report.progress_steps.iter().any(|step| {
        step.id == "manifest"
            && step.label == "Embed package manifest"
            && step.detail.contains("sidecar manifest output is disabled")
    }));
}

#[test]
fn prepare_for_export_reports_empty_appendix_options_as_info() {
    let source = "---\ntitle: Empty Appendix Options\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\n---\n# Empty Appendix Options\nNo appendices are available.\n".to_string();
    let report = prepare_for_export(PrepareExportRequest {
        text: source.clone(),
        file_path: None,
        target: "docx".to_string(),
        options: json!({
            "warnOnDirtyGit": false,
            "includeGlossary": true,
            "includeComments": true,
            "includeProvenance": true
        }),
    });

    assert!(report.ready, "{:#?}", report.diagnostics);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.warning_count, 0);
    assert_eq!(report.info_count, 3);
    assert_eq!(report.manifest.readiness.info_count, 3);
    for option in ["includeGlossary", "includeComments", "includeProvenance"] {
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == "info"
                    && diagnostic
                        .related
                        .iter()
                        .any(|related| related == &format!("option:{option}"))),
            "missing content-sensitive info diagnostic for {option}: {:#?}",
            report.diagnostics
        );
    }

    let populated = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Populated Appendix Options\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\n---\n# Populated Appendix Options\n```glossary\nSLA: Service-level agreement.\n```\n<!-- comment: resolved | author: QA | at: 2026-05-21T10:00:00Z | Looks good. -->\n```ai-source\nprovider: OpenAI\nmodel: gpt-5.4\ndate: 2026-05-21\npromptSummary: Drafted appendix notes\nreviewedBy: QA\nreviewedAt: 2026-05-21T10:30:00Z\nstatus: human-reviewed\n```\n"
            .to_string(),
        file_path: None,
        target: "docx".to_string(),
        options: json!({
            "warnOnDirtyGit": false,
            "includeGlossary": true,
            "includeComments": true,
            "includeProvenance": true
        }),
    });
    assert!(populated.ready, "{:#?}", populated.diagnostics);
    assert_eq!(populated.info_count, 0, "{:#?}", populated.diagnostics);
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
            "disabledTransformEngines": { "dot": "no" },
            "transformInputModes": { "dot": "pipe" }
        }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 5);
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
        .contains("disabledTransformEngines.dot must be true or false")));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("transformInputModes.dot must be stdin or file")));
}

#[cfg(unix)]
#[test]
fn prepare_for_export_validates_transform_engine_paths_before_export() {
    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-transform-path-test-{unique}"));
    fs::create_dir_all(&root).expect("create transform path test dir");
    let missing = root.join("missing-dot");
    let directory = root.join("engine-directory");
    fs::create_dir_all(&directory).expect("create directory engine path");
    let non_executable = root.join("plantuml");
    fs::write(&non_executable, "#!/bin/sh\nexit 0\n").expect("write non executable engine");
    let mut permissions = fs::metadata(&non_executable)
        .expect("non executable metadata")
        .permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(&non_executable, permissions).expect("set non executable permissions");
    let executable = write_executable_script("export-path-pikchr", "#!/bin/sh\nprintf '<svg />'\n");

    let report = prepare_for_export(PrepareExportRequest {
        text: "---\ntitle: Engine Paths\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21\n---\n# Ready".to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({
            "warnOnDirtyGit": false,
            "transformEnginePaths": {
                "dot": path_to_string(&missing),
                "d2": path_to_string(&directory),
                "plantuml": path_to_string(&non_executable),
                "pikchr": path_to_string(&executable),
                "graphviz": path_to_string(&missing)
            },
            "disabledTransformEngines": {
                "graphviz": true
            }
        }),
    });

    assert!(!report.ready);
    assert_eq!(report.error_count, 3, "{:#?}", report.diagnostics);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("transformEnginePaths.dot does not point to an executable file")
            && diagnostic.source_file.as_deref() == Some(path_to_string(&missing).as_str())
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("transformEnginePaths.d2 does not point to an executable file")
            && diagnostic.source_file.as_deref() == Some(path_to_string(&directory).as_str())
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("transformEnginePaths.plantuml is not executable")
            && diagnostic.source_file.as_deref() == Some(path_to_string(&non_executable).as_str())
    }));
    assert!(!report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("transformEnginePaths.pikchr")));
    assert!(!report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("transformEnginePaths.graphviz")));

    let _ = fs::remove_file(executable);
    fs::remove_dir_all(root).expect("clean transform path test dir");
}

#[test]
fn prepare_for_export_carries_broad_readiness_audit_to_manifest() {
    let source = r#"---
status: approved
layout:
  pageSize: billboard
---
# Readiness Audit

Unsupported claim [@missing].

Broken margin: {{=missing + }}

<!-- comment: author: QA | at: 2026-05-20 | open | Resolve before release. -->
<!-- change: author: QA | Updated claim without a timestamp. -->

```ai-source
provider: OpenAI
date: 2026-05-20
promptSummary: rough claim
status: human-reviewed
```

<!-- ai-assisted: status=needs-review | source=OpenAI | promptSummary=Drafted unsupported claim -->
## AI Draft

![Missing](missing.png)

<figure class="figure"><img src="data:image/svg+xml;base64,PHN2Zy8+" alt="No caption"/></figure>

[Missing appendix](missing.md)

| Metric | Value |
| --- | ---: |
| Revenue | 42 |
| Broken | =SUM(BAD |

<figure class="equation"><code>ROI = Gain / Cost</code></figure>
"#;
    let report = prepare_for_export(PrepareExportRequest {
        text: source.to_string(),
        file_path: None,
        target: "pdf".to_string(),
        options: json!({
            "includeManifest": true,
            "warnOnDirtyGit": false,
            "transformTimeoutMs": 50000,
            "transformEnginePaths": { "dot": "dot" },
            "trustedTransformEngines": { "dot": "yes" },
            "transformInputModes": { "dot": "pipe" }
        }),
    });

    assert!(!report.ready);
    assert!(report.error_count > 0, "{:#?}", report.diagnostics);
    assert!(report.warning_count > 0, "{:#?}", report.diagnostics);
    assert_eq!(report.readiness.error_count, report.error_count);
    assert_eq!(report.readiness.warning_count, report.warning_count);
    assert_eq!(report.manifest.readiness.error_count, report.error_count);
    assert_eq!(
        report.manifest.readiness.warning_count,
        report.warning_count
    );
    assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
    assert_eq!(report.manifest.source_map.len(), report.source_map.len());
    assert!(report.manifest.output_path.is_none());
    assert!(report.manifest.output_hash.is_none());
    assert!(report
        .manifest
        .progress_steps
        .iter()
        .any(|step| step.id == "render" && step.state == "pending"));

    for expected in [
        "Missing title metadata.",
        "Missing version metadata.",
        "Approved or published document is missing approval metadata.",
        "Unsupported layout pageSize: billboard",
        "Document contains citations but no bibliography source.",
        "Inline formula error",
        "Document has unresolved review comments.",
        "Change note is missing audit metadata.",
        "Document has AI-assisted sections that are not human-reviewed.",
        "AI source block is missing provenance metadata.",
        "AI source is marked human-reviewed without reviewer metadata.",
        "Broken image path",
        "Broken link path",
        "Markdown table formula error",
        "Figure is missing a stable label or caption.",
        "Table is missing a stable label or caption.",
        "Equation is missing a stable label or caption.",
        "transformTimeoutMs must be between 1 and 30000.",
        "transformEnginePaths.dot must be an absolute path.",
        "trustedTransformEngines.dot must be true or false.",
        "transformInputModes.dot must be stdin or file.",
    ] {
        assert_readiness_contains(&report, expected);
    }
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

fn assert_readiness_contains(report: &ExportReadinessReport, expected: &str) {
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains(expected)),
        "readiness report missing {expected:?}: {:#?}",
        report.diagnostics
    );
    assert!(
        report
            .manifest
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains(expected)),
        "manifest missing {expected:?}: {:#?}",
        report.manifest.diagnostics
    );
}
