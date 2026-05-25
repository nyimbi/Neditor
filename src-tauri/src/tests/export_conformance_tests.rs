use super::*;

#[test]
fn pdf_export_splits_large_tables_across_pages() {
    let rows = (1..=60)
        .map(|index| format!("| Row {index} | {index} |"))
        .collect::<Vec<_>>()
        .join("\n");
    let response = compile(CompileRequest {
            text: format!(
                "---\ntitle: Large Table\nstatus: approved\napprovedBy: QA\n---\n# Large Table\n\nTable: Row audit {{#tbl:rows}}\n| Label | Value |\n| --- | ---: |\n{rows}\n"
            ),
            file_path: None,
        });

    let pdf = render_pdf_bytes(&response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("/Count 3"));
    assert!(pdf_text.contains("Row audit"));
    assert!(pdf_text.contains("Row audit \\(continued\\)"));
    assert!(pdf_text.contains("(Row 1) Tj"));
    assert!(pdf_text.contains("(Row 60) Tj"));
    assert!(pdf_text.contains("Page 3 of 3"));
}

#[test]
fn pptx_export_can_include_an_agenda_from_options() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Agenda Export\nstatus: approved\napprovedBy: QA\n---\n# Agenda Export\nIntro.\n\n## Market\nBody.\n\n## Finance\nBody.\n".to_string(),
            file_path: None,
        });

    let pptx = render_pptx_bytes(&response, &json!({ "includeAgenda": true })).expect("pptx bytes");
    let agenda_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    let body_slide = zip_entry_text(&pptx, "ppt/slides/slide3.xml");

    assert!(agenda_slide.contains("Agenda"));
    assert!(agenda_slide.contains("Agenda Export"));
    assert!(agenda_slide.contains("Market"));
    assert!(agenda_slide.contains("Finance"));
    assert!(body_slide.contains("Agenda Export"));
}

#[test]
fn pptx_export_splits_large_tables_across_slides() {
    let rows = (1..=20)
        .map(|index| format!("| Row {index} | {index} |"))
        .collect::<Vec<_>>()
        .join("\n");
    let response = compile(CompileRequest {
            text: format!(
                "---\ntitle: Large Table Deck\nstatus: approved\napprovedBy: QA\n---\n# Large Table Deck\n\nTable: Row audit {{#tbl:rows}}\n| Label | Value |\n| --- | ---: |\n{rows}\n"
            ),
            file_path: None,
        });

    let pptx = render_pptx_bytes(&response, &json!({})).expect("pptx bytes");
    let presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
    let slide_two = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    let slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
    let slide_four = zip_entry_text(&pptx, "ppt/slides/slide4.xml");
    assert!(presentation.contains(r#"r:id="rId4""#));
    assert!(slide_two.contains("<a:tbl>"));
    assert!(slide_two.contains("Row 1"));
    assert!(slide_two.contains("Row 8"));
    assert!(!slide_two.contains("Row 9"));
    assert!(slide_three.contains("Row audit (continued)"));
    assert!(slide_three.contains("Row 9"));
    assert!(slide_three.contains("Row 16"));
    assert!(slide_four.contains("Row audit (continued)"));
    assert!(slide_four.contains("Row 20"));
}

#[test]
fn representative_rendered_export_artifacts_are_package_inspectable() {
    let response = compile(CompileRequest {
        text: r##"---
title: Rendered Export Audit
subtitle: Board package
version: 3.1.4
status: approved
approvedBy: Release QA
approvedAt: 2026-05-21T09:00:00Z
classification: Confidential
toc: true
legalDisclaimer: "For rendered export audit only."
brand:
  name: NEditor Audit
  color: "#2563EB"
layout:
  header: "{{title}} | {{status}}"
  footer: "{{classification}} | Page {{page}} of {{pages}}"
---

# Rendered Export Audit

[TOC]

> [!INFO] Readiness
> This package is ready for rendered artifact inspection.

Table: Control summary {#tbl:controls}
| Control | Owner | Status |
| --- | --- | --- |
| Export manifest | Release QA | Complete |
| Native package check | Desktop QA | Complete |

```chart
type: line
title: Export Confidence
data:
  - month: Apr
    score: 86
  - month: May
    score: 94
x: month
y: score
```

![Architecture](data:image/svg+xml;base64,PHN2Zy8+){#fig:architecture caption="Architecture diagram" fit="cover"}

{{page-break}}

## Decision

See {@tbl:controls} and {@fig:architecture}.

<!-- comment: author: QA | at: 2026-05-21T09:10:00Z | resolved | Rendered artifact audit passed. -->

```ai-source
provider: OpenAI
model: gpt-5.4
date: 2026-05-21
promptSummary: rendered export audit fixture
reviewedBy: Release QA
reviewedAt: 2026-05-21T09:15:00Z
status: human-reviewed
```
"##
        .to_string(),
        file_path: None,
    });
    let options = json!({
        "includeStyles": true,
        "includeSyntaxHighlighting": true,
        "coverPage": true,
        "pageNumbers": true,
        "includeComments": true,
        "includeProvenance": true,
        "includeManifest": true,
        "watermark": "APPROVED",
        "layoutPreset": "board"
    });

    assert!(
        !response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "error"),
        "{:#?}",
        response.diagnostics
    );
    assert_eq!(
        response.export_manifest.document_title,
        "Rendered Export Audit"
    );
    assert_eq!(response.export_manifest.document_version, "3.1.4");
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "chart" && artifact.output_kind == "svg"));

    let html = render_full_html(&response, &options);
    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("class=\"cover\""));
    assert!(html.contains("Rendered Export Audit | approved"));
    assert!(html.contains("APPROVED"));
    assert!(html.contains("class=\"export-comments\""));
    assert!(html.contains("class=\"export-provenance\""));
    assert!(html.contains("For rendered export audit only."));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf.starts_with(b"%PDF-1.4\n"));
    assert!(pdf_text.contains("/Type /Catalog"));
    assert!(pdf_text.contains("/Type /Pages"));
    assert!(pdf_text.contains("/MediaBox [0 0 "));
    assert!(pdf_text.contains("Cover: Rendered Export Audit"));
    assert!(pdf_text.contains("Watermark: APPROVED"));
    assert!(pdf_text.contains("Review Comments"));
    assert!(pdf_text.contains("AI Provenance"));
    assert!(pdf_text.contains("For rendered export audit only."));

    let docx = render_docx_bytes(&response, &options).expect("docx audit bytes");
    assert!(zip_has_entry(&docx, "[Content_Types].xml"));
    assert!(zip_has_entry(&docx, "_rels/.rels"));
    assert!(zip_has_entry(&docx, "word/document.xml"));
    assert!(zip_has_entry(&docx, "docProps/core.xml"));
    assert!(zip_has_entry(&docx, "docProps/custom.xml"));
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_core = zip_entry_text(&docx, "docProps/core.xml");
    let docx_custom = zip_entry_text(&docx, "docProps/custom.xml");
    assert!(docx_document.contains("Cover: Rendered Export Audit"));
    assert!(docx_document.contains("Table controls"));
    assert!(docx_document.contains("Figure architecture"));
    assert!(docx_document.contains("Review Comments"));
    assert!(docx_document.contains("AI Provenance"));
    assert!(docx_core.contains("<dc:title>Rendered Export Audit</dc:title>"));
    assert!(docx_custom.contains("3.1.4"));
    assert!(docx_custom.contains(r#"name="NEditorApprovedBy""#));
    assert!(docx_custom.contains("Release QA"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx audit bytes");
    assert!(zip_has_entry(&pptx, "[Content_Types].xml"));
    assert!(zip_has_entry(&pptx, "_rels/.rels"));
    assert!(zip_has_entry(&pptx, "ppt/presentation.xml"));
    assert!(zip_has_entry(&pptx, "ppt/_rels/presentation.xml.rels"));
    assert!(zip_entry_count_with_prefix(&pptx, "ppt/slides/", ".xml") >= 2);
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    let pptx_core = zip_entry_text(&pptx, "docProps/core.xml");
    let pptx_custom = zip_entry_text(&pptx, "docProps/custom.xml");
    assert!(pptx_slides.contains("Rendered Export Audit"));
    assert!(pptx_slides.contains("Control summary"));
    assert!(pptx_slides.contains("Review Comments"));
    assert!(pptx_slides.contains("AI Provenance"));
    assert!(pptx_core.contains("<dc:title>Rendered Export Audit</dc:title>"));
    assert!(pptx_custom.contains("3.1.4"));
    assert!(pptx_custom.contains(r#"name="NEditorApprovedBy""#));
    assert!(pptx_custom.contains("Release QA"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_target = "markdown-bundle".to_string();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("audit bundle");
    assert!(zip_has_entry(&bundle, "document.md"));
    assert!(zip_has_entry(&bundle, "document.txt"));
    assert!(zip_has_entry(&bundle, "document-ast.json"));
    assert!(zip_has_entry(&bundle, "manifest.json"));
    assert!(zip_has_entry(&bundle, "transform-artifacts.json"));
    assert!(zip_has_entry(&bundle, "media-uses.json"));
    let manifest: Value =
        serde_json::from_str(&zip_entry_text(&bundle, "manifest.json")).expect("manifest json");
    assert_eq!(
        manifest.get("document_title").and_then(Value::as_str),
        Some("Rendered Export Audit")
    );
    assert_eq!(
        manifest.get("document_version").and_then(Value::as_str),
        Some("3.1.4")
    );
    assert_eq!(
        manifest
            .pointer("/export_options/watermark")
            .and_then(Value::as_str),
        Some("APPROVED")
    );
    assert!(manifest
        .get("transform_artifacts")
        .and_then(Value::as_array)
        .is_some_and(|artifacts| artifacts
            .iter()
            .any(|artifact| artifact.get("name").and_then(Value::as_str) == Some("chart"))));

    let mut blog_manifest = response.export_manifest.clone();
    blog_manifest.export_target = "blog".to_string();
    blog_manifest.export_options = options.clone();
    let blog =
        render_blog_publish_package_bytes(&response, &blog_manifest).expect("blog audit package");
    assert!(zip_has_entry(&blog, "post.md"));
    assert!(zip_has_entry(&blog, "post.html"));
    assert!(zip_has_entry(&blog, "substack-copy.html"));
    assert!(zip_has_entry(&blog, "metadata.json"));
    assert!(zip_has_entry(&blog, "manifest.json"));
    assert!(zip_entry_text(&blog, "post.html").contains("Rendered Export Audit"));
    assert!(zip_entry_text(&blog, "metadata.json").contains("\"exportTarget\": \"blog\""));

    let mut substack_manifest = response.export_manifest.clone();
    substack_manifest.export_target = "substack".to_string();
    substack_manifest.export_options = options.clone();
    let substack = render_blog_publish_package_bytes(&response, &substack_manifest)
        .expect("substack audit package");
    assert!(zip_has_entry(&substack, "post.md"));
    assert!(zip_has_entry(&substack, "post.html"));
    assert!(zip_has_entry(&substack, "substack-copy.html"));
    assert!(zip_has_entry(&substack, "metadata.json"));
    assert!(zip_has_entry(&substack, "manifest.json"));
    assert!(zip_entry_text(&substack, "substack-copy.html").contains("Rendered Export Audit"));
    assert!(zip_entry_text(&substack, "metadata.json").contains("\"exportTarget\": \"substack\""));

    let mut latex_manifest = response.export_manifest.clone();
    latex_manifest.export_target = "latex".to_string();
    latex_manifest.export_options = options.clone();
    let latex = render_latex_bytes(&response, &latex_manifest).expect("latex audit artifact");
    let latex_text = String::from_utf8(latex.clone()).expect("latex utf8");
    assert!(latex_text.contains("\\documentclass"));
    assert!(latex_text.contains("\\title{Rendered Export Audit}"));
    assert!(latex_text.contains("\\begin{longtable}"));
    assert!(latex_text.contains("Source hash"));

    let mut google_docs_manifest = response.export_manifest.clone();
    google_docs_manifest.export_target = "google-docs".to_string();
    google_docs_manifest.export_options = options.clone();
    let google_docs = render_google_docs_package_bytes(&response, &google_docs_manifest)
        .expect("google docs audit package");
    assert!(zip_has_entry(&google_docs, "document.docx"));
    assert!(zip_has_entry(&google_docs, "document.html"));
    assert!(zip_has_entry(&google_docs, "document.md"));
    assert!(zip_has_entry(&google_docs, "metadata.json"));
    assert!(zip_has_entry(&google_docs, "manifest.json"));
    assert!(
        zip_entry_text(&google_docs, "metadata.json").contains("\"exportTarget\": \"google-docs\"")
    );

    let mut epub_manifest = response.export_manifest.clone();
    epub_manifest.export_target = "epub".to_string();
    epub_manifest.export_options = options.clone();
    let epub = render_epub_bytes(&response, &epub_manifest).expect("epub audit package");
    assert!(zip_has_entry(&epub, "mimetype"));
    assert!(zip_has_entry(&epub, "META-INF/container.xml"));
    assert!(zip_has_entry(&epub, "OEBPS/content.opf"));
    assert!(zip_has_entry(&epub, "OEBPS/nav.xhtml"));
    assert!(zip_has_entry(&epub, "OEBPS/document.xhtml"));
    assert!(zip_has_entry(&epub, "OEBPS/metadata/manifest.json"));
    assert!(zip_entry_text(&epub, "OEBPS/content.opf")
        .contains("<dc:title>Rendered Export Audit</dc:title>"));
    assert!(zip_entry_text(&epub, "OEBPS/metadata/manifest.json")
        .contains("\"export_target\": \"epub\""));

    write_rendered_export_audit_artifacts(RenderedExportAuditArtifacts {
        response: &response,
        html: &html,
        pdf: &pdf,
        docx: &docx,
        pptx: &pptx,
        bundle: &bundle,
        blog: &blog,
        substack: &substack,
        latex: &latex,
        google_docs: &google_docs,
        epub: &epub,
    });
}

struct RenderedExportAuditArtifacts<'a> {
    response: &'a CompileResponse,
    html: &'a str,
    pdf: &'a [u8],
    docx: &'a [u8],
    pptx: &'a [u8],
    bundle: &'a [u8],
    blog: &'a [u8],
    substack: &'a [u8],
    latex: &'a [u8],
    google_docs: &'a [u8],
    epub: &'a [u8],
}

fn write_rendered_export_audit_artifacts(artifacts: RenderedExportAuditArtifacts<'_>) {
    let Some(root) = std::env::var_os("NEDITOR_RENDERED_EXPORT_AUDIT_DIR").map(PathBuf::from)
    else {
        return;
    };
    fs::create_dir_all(&root).expect("create rendered export audit dir");
    fs::write(root.join("rendered-export-audit.html"), artifacts.html).expect("write audit html");
    fs::write(root.join("rendered-export-audit.pdf"), artifacts.pdf).expect("write audit pdf");
    fs::write(root.join("rendered-export-audit.docx"), artifacts.docx).expect("write audit docx");
    fs::write(root.join("rendered-export-audit.pptx"), artifacts.pptx).expect("write audit pptx");
    fs::write(
        root.join("rendered-export-audit.markdown-bundle.zip"),
        artifacts.bundle,
    )
    .expect("write audit markdown bundle");
    fs::write(root.join("rendered-export-audit.blog.zip"), artifacts.blog)
        .expect("write audit blog package");
    fs::write(
        root.join("rendered-export-audit.substack.zip"),
        artifacts.substack,
    )
    .expect("write audit substack package");
    fs::write(root.join("rendered-export-audit.tex"), artifacts.latex).expect("write audit latex");
    fs::write(
        root.join("rendered-export-audit.google-docs.zip"),
        artifacts.google_docs,
    )
    .expect("write audit google docs package");
    fs::write(root.join("rendered-export-audit.epub"), artifacts.epub)
        .expect("write audit epub package");
    let review_cases = write_rendered_export_review_cases(&root);
    let report = json!({
        "title": artifacts.response.semantic.title,
        "version": artifacts.response.export_manifest.document_version,
        "targets": [
            { "target": "html", "path": "rendered-export-audit.html", "bytes": artifacts.html.len(), "sha256": sha256_hex(artifacts.html.as_bytes()) },
            { "target": "pdf", "path": "rendered-export-audit.pdf", "bytes": artifacts.pdf.len(), "sha256": sha256_hex(artifacts.pdf) },
            { "target": "docx", "path": "rendered-export-audit.docx", "bytes": artifacts.docx.len(), "sha256": sha256_hex(artifacts.docx) },
            { "target": "pptx", "path": "rendered-export-audit.pptx", "bytes": artifacts.pptx.len(), "sha256": sha256_hex(artifacts.pptx) },
            { "target": "markdown-bundle", "path": "rendered-export-audit.markdown-bundle.zip", "bytes": artifacts.bundle.len(), "sha256": sha256_hex(artifacts.bundle) },
            { "target": "blog", "path": "rendered-export-audit.blog.zip", "bytes": artifacts.blog.len(), "sha256": sha256_hex(artifacts.blog) },
            { "target": "substack", "path": "rendered-export-audit.substack.zip", "bytes": artifacts.substack.len(), "sha256": sha256_hex(artifacts.substack) },
            { "target": "latex", "path": "rendered-export-audit.tex", "bytes": artifacts.latex.len(), "sha256": sha256_hex(artifacts.latex) },
            { "target": "google-docs", "path": "rendered-export-audit.google-docs.zip", "bytes": artifacts.google_docs.len(), "sha256": sha256_hex(artifacts.google_docs) },
            { "target": "epub", "path": "rendered-export-audit.epub", "bytes": artifacts.epub.len(), "sha256": sha256_hex(artifacts.epub) }
        ],
        "reviewCases": review_cases,
        "manualChecklist": [
            "Open rendered-export-audit.html and confirm cover, watermark, comments, provenance, table, figure, and chart are visible.",
            "Open rendered-export-audit.pdf and confirm cover, watermark, pagination, comments, provenance, and decision section are readable.",
            "Open rendered-export-audit.docx and confirm document properties, table, figure caption, comments appendix, and AI provenance appendix are present.",
            "Open rendered-export-audit.pptx and confirm slides contain title, control summary, comments, and AI provenance.",
            "Open rendered-export-audit.tex in a LaTeX editor and confirm headings, tables, equations, and metadata are present.",
            "Open rendered-export-audit.epub in an EPUB reader and confirm the table of contents, equations, metadata, and document body are readable.",
            "Inspect ZIP packages and confirm manifests plus publishing and Google Docs handoff files are present.",
            "Open review-cases/rich-blocks.* and confirm block quote, callout, task list, code, table, figure, equation, and generated figure/table lists remain legible across targets.",
            "Open review-cases/option-heavy.* and confirm cover/page numbers/watermark/style/appendix options are visible without corrupting body content."
        ]
    });
    fs::write(
        root.join("rendered-export-audit-report.json"),
        serde_json::to_string_pretty(&report).expect("audit report json"),
    )
    .expect("write audit report");
    fs::write(
        root.join("README.md"),
        "# Rendered Export Audit\n\nThis directory is generated by `pnpm run test:rendered-exports`.\n\nOpen `manual-review.html` first, then open the HTML/PDF/DOCX/PPTX/EPUB files in native viewers for manual visual review, inspect the LaTeX source, and inspect the ZIP packages for manifest-backed publishing and Google Docs handoff evidence. The JSON report records target paths, sizes, hashes, and the manual checklist; `viewer-proof.json` records executable package/viewer assertions.\n",
    )
    .expect("write audit readme");
}

fn write_rendered_export_review_cases(root: &Path) -> Vec<Value> {
    let review_root = root.join("review-cases");
    fs::create_dir_all(&review_root).expect("create rendered export review case dir");
    let cases = [
        (
            "rich-blocks",
            "Rich Markdown Blocks",
            "---\ntitle: Rich Markdown Blocks\nstatus: approved\napprovedBy: Release QA\napprovedAt: 2026-05-21T10:00:00Z\n---\n# Rich Markdown Blocks\n\n> Quoted evidence\n> across lines\n\n> [!WARNING] Launch Gate\n> Confirm fallback before release.\n\n- First decision\n  - Nested control\n- Second decision\n\n1. First ordered step\n2. Second ordered step\n\n- [x] Reviewed by finance\n- [ ] Attach signed approval\n\n```js\nconst total = 42;\n```\n\nTable: Decision table {#tbl:decisions}\n| Control | Owner |\n| --- | --- |\n| Fallback | Platform |\n\n![Architecture](data:image/svg+xml;base64,PHN2Zy8+){#fig:reference caption=\"Reference architecture\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n[LIST_OF_FIGURES]\n\n[LIST_OF_TABLES]\n\nSee {@tbl:decisions}, {@fig:reference}, and {@eq:roi}.\n",
            json!({ "includeGlossary": false, "includeComments": false, "includeProvenance": false }),
            vec![
                "Rich Markdown Blocks",
                "Launch Gate",
                "Decision table",
                "Reference architecture",
            ],
        ),
        (
            "option-heavy",
            "Option Heavy Export",
            "---\ntitle: Option Heavy Export\nsubtitle: Option matrix review\nversion: 2.0.0\nstatus: approved\napprovedBy: Release QA\napprovedAt: 2026-05-21T10:30:00Z\nclassification: Internal\nlegalDisclaimer: \"For option review only.\"\nlayout:\n  header: \"{{title}} | {{status}}\"\n  footer: \"{{classification}} | Page {{page}} of {{pages}}\"\n---\n# Option Heavy Export\n\nTable: Option status {#tbl:options}\n| Option | State |\n| --- | --- |\n| Cover | Enabled |\n| Watermark | Enabled |\n\n```glossary\nWatermark: A visible export review label.\n```\n\n#index:watermark\n\n<!-- comment: author: QA | at: 2026-05-21T10:35:00Z | resolved | Option-heavy export review passed. -->\n\n```ai-source\nprovider: OpenAI\nmodel: gpt-5.4\ndate: 2026-05-21\npromptSummary: option-heavy export fixture\nreviewedBy: Release QA\nreviewedAt: 2026-05-21T10:40:00Z\nstatus: human-reviewed\n```\n",
            json!({
                "includeStyles": true,
                "includeSyntaxHighlighting": true,
                "coverPage": true,
                "pageNumbers": true,
                "includeComments": true,
                "includeProvenance": true,
                "includeGlossary": true,
                "includeIndex": true,
                "includeTextOutput": true,
                "includeManifest": true,
                "watermark": "INTERNAL",
                "layoutPreset": "report"
            }),
            vec![
                "Option Heavy Export",
                "INTERNAL",
                "Review Comments",
                "AI Provenance",
                "For option review only.",
            ],
        ),
    ];

    cases
        .into_iter()
        .map(|(slug, title, text, options, evidence)| {
            let response = compile(CompileRequest {
                text: text.to_string(),
                file_path: None,
            });
            assert!(
                !response
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.severity == "error"),
                "{:#?}",
                response.diagnostics
            );
            let html = render_full_html(&response, &options);
            let pdf = render_pdf_bytes(&response, &options);
            let docx = render_docx_bytes(&response, &options)
                .unwrap_or_else(|error| panic!("{slug} docx render failed: {error}"));
            let pptx = render_pptx_bytes(&response, &options)
                .unwrap_or_else(|error| panic!("{slug} pptx render failed: {error}"));
            let mut bundle_manifest = response.export_manifest.clone();
            bundle_manifest.export_target = "markdown-bundle".to_string();
            bundle_manifest.export_options = options.clone();
            let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest)
                .unwrap_or_else(|error| panic!("{slug} bundle render failed: {error}"));

            let case_root = review_root.join(slug);
            fs::create_dir_all(&case_root).expect("create rendered export review case subdir");
            let html_path = format!("review-cases/{slug}/{slug}.html");
            let pdf_path = format!("review-cases/{slug}/{slug}.pdf");
            let docx_path = format!("review-cases/{slug}/{slug}.docx");
            let pptx_path = format!("review-cases/{slug}/{slug}.pptx");
            let bundle_path = format!("review-cases/{slug}/{slug}.markdown-bundle.zip");
            fs::write(root.join(&html_path), html.as_bytes())
                .expect("write rendered review html");
            fs::write(root.join(&pdf_path), &pdf).expect("write rendered review pdf");
            fs::write(root.join(&docx_path), &docx).expect("write rendered review docx");
            fs::write(root.join(&pptx_path), &pptx).expect("write rendered review pptx");
            fs::write(root.join(&bundle_path), &bundle).expect("write rendered review bundle");

            let pdf_text = String::from_utf8_lossy(&pdf);
            let docx_text = zip_entry_text(&docx, "word/document.xml");
            let pptx_text = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
            let bundle_text = zip_entry_text(&bundle, "document.md");
            for expected in &evidence {
                assert!(html.contains(expected), "{slug} html missing {expected}");
            }
            assert!(pdf_text.contains(title), "{slug} pdf missing title");
            assert!(docx_text.contains(title), "{slug} docx missing title");
            assert!(pptx_text.contains(title), "{slug} pptx missing title");
            assert!(bundle_text.contains(title), "{slug} bundle missing title");

            json!({
                "slug": slug,
                "title": title,
                "requiredEvidence": evidence,
                "targets": [
                    { "target": "html", "path": html_path, "bytes": html.len(), "sha256": sha256_hex(html.as_bytes()) },
                    { "target": "pdf", "path": pdf_path, "bytes": pdf.len(), "sha256": sha256_hex(&pdf) },
                    { "target": "docx", "path": docx_path, "bytes": docx.len(), "sha256": sha256_hex(&docx) },
                    { "target": "pptx", "path": pptx_path, "bytes": pptx.len(), "sha256": sha256_hex(&pptx) },
                    { "target": "markdown-bundle", "path": bundle_path, "bytes": bundle.len(), "sha256": sha256_hex(&bundle) }
                ]
            })
        })
        .collect()
}

#[test]
fn rich_markdown_blocks_survive_cross_target_exports() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Rich Artifact Blocks\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-20T12:00:00Z\n---\n# Rich Artifact Blocks\n\n> Quoted evidence\n> across lines\n\n> [!WARNING] Launch Gate\n> Confirm fallback before release.\n\n- First decision\n  - Nested control\n- Second decision\n\n1. First ordered step\n2. Second ordered step\n\n- [x] Reviewed by finance\n- [ ] Attach signed approval\n\n```js\nconst total = 42;\n```\n\nTable: Controls {#tbl:controls}\n| Control | Owner |\n| --- | --- |\n| Fallback | Platform |\n\n![Architecture](data:image/svg+xml;base64,PHN2Zy8+){#fig:architecture caption=\"Reference architecture\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n[LIST_OF_FIGURES]\n\n[LIST_OF_TABLES]\n\nSee {@tbl:controls}, {@fig:architecture}, and {@eq:roi}.\n".to_string(),
        file_path: None,
    });
    let options = json!({ "includeGlossary": false });

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response
        .document_ast
        .blocks
        .iter()
        .any(|block| matches!(block, DocumentBlock::BlockQuote { text, .. } if text.contains("Quoted evidence"))));
    assert!(response
        .document_ast
        .blocks
        .iter()
        .any(|block| matches!(block, DocumentBlock::Callout { callout_type, title, text, .. } if callout_type == "warning" && title == "Launch Gate" && text.contains("Confirm fallback"))));
    assert!(response
        .document_ast
        .blocks
        .iter()
        .any(|block| matches!(block, DocumentBlock::CodeBlock { language, code, .. } if language.as_deref() == Some("js") && code.contains("const total = 42"))));
    assert!(response.compiled_markdown.contains("List of Figures"));
    assert!(response
        .compiled_markdown
        .contains("[Figure 1: Reference architecture](#fig:architecture)"));
    assert!(response.compiled_markdown.contains("List of Tables"));
    assert!(response
        .compiled_markdown
        .contains("[Table 1: Controls](#tbl:controls)"));
    assert!(response
        .compiled_markdown
        .contains("[Table controls](#tbl:controls)"));
    assert!(response
        .compiled_markdown
        .contains("[Figure architecture](#fig:architecture)"));
    assert!(response
        .compiled_markdown
        .contains("[Equation roi](#eq:roi)"));

    let html = render_full_html(&response, &options);
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("Quoted evidence"));
    assert!(html.contains("class=\"callout callout-warning\""));
    assert!(html.contains("Launch Gate"));
    assert!(html.contains("Nested control"));
    assert!(html.contains("total"));
    assert!(html.contains("42"));
    assert!(html.contains("List of Figures"));
    assert!(html.contains("List of Tables"));
    assert!(html.contains(r##"<a href="#tbl:controls">Table controls</a>"##));
    assert!(html.contains(r##"<a href="#fig:architecture">Figure architecture</a>"##));
    assert!(html.contains(r##"<a href="#eq:roi">Equation roi</a>"##));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("> Quoted evidence"));
    assert!(pdf_text.contains("> across lines"));
    assert!(pdf_text.contains("Callout: warning: Launch Gate"));
    assert!(pdf_text.contains("- First decision"));
    assert!(pdf_text.contains("Nested control"));
    assert!(pdf_text.contains("1. First ordered step"));
    assert!(pdf_text.contains("- [x] Reviewed by finance"));
    assert!(pdf_text.contains("```js"));
    assert!(pdf_text.contains("const total = 42;"));
    assert!(pdf_text.contains("Controls"));
    assert!(pdf_text.contains("Fallback"));
    assert!(pdf_text.contains("Reference architecture"));
    assert!(pdf_text.contains("Equation roi"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Quote: Quoted evidence"));
    assert!(docx_document.contains("across lines"));
    assert!(docx_document.contains("Callout: warning: Launch Gate"));
    assert!(docx_document.contains("Confirm fallback before release."));
    assert!(docx_document.contains("First decision"));
    assert!(docx_document.contains("Nested control"));
    assert!(docx_document.contains("First ordered step"));
    assert!(docx_document.contains("[x] Reviewed by finance"));
    assert!(docx_document.contains("Code (js)"));
    assert!(docx_document.contains("const total = 42;"));
    assert!(docx_document.contains("List of Figures"));
    assert!(docx_document.contains("List of Tables"));
    assert!(docx_document.contains("Table controls"));
    assert!(docx_document.contains("Figure architecture"));
    assert!(docx_document.contains("Equation roi"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("&gt; Quoted evidence"));
    assert!(pptx_slides.contains("Callout: warning: Launch Gate"));
    assert!(pptx_slides.contains("- First decision"));
    assert!(pptx_slides.contains("Nested control"));
    assert!(pptx_slides.contains("1. First ordered step"));
    assert!(pptx_slides.contains("- [x] Reviewed by finance"));
    assert!(pptx_slides.contains("```js"));
    assert!(pptx_slides.contains("const total = 42;"));
    assert!(pptx_slides.contains("Controls"));
    assert!(pptx_slides.contains("Fallback"));
    assert!(pptx_slides.contains("Reference architecture"));
    assert!(pptx_slides.contains("List of Figures"));
    assert!(pptx_slides.contains("List of Tables"));
    assert!(pptx_slides.contains("Equation roi"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    assert!(bundled_text.contains("> Quoted evidence"));
    assert!(bundled_text.contains("Callout: warning: Launch Gate"));
    assert!(bundled_text.contains("```js"));
    assert!(bundled_text.contains("const total = 42;"));
    assert!(bundled_text.contains("Controls"));
    assert!(bundled_text.contains("Fallback"));
    assert!(bundled_text.contains("Figure: fig:architecture: Reference architecture"));
    assert!(bundled_ast.contains("\"kind\": \"block_quote\""));
    assert!(bundled_ast.contains("\"kind\": \"callout\""));
    assert!(bundled_ast.contains("\"kind\": \"code_block\""));
    assert!(bundled_ast.contains("\"kind\": \"list\""));
    assert!(bundled_ast.contains("\"kind\": \"task_list\""));
}

#[test]
fn safe_business_transforms_survive_cross_target_exports() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Business Transform Pack\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21T08:00:00Z\n---\n# Business Transform Pack\n\n```roadmap\nQ1: Launch beta | status=active | owner=Product\nQ2: Expand exports | due=2026-06-30 | owner=Docs\n```\n\n```adr\nStatus: accepted\nContext: Exports must be auditable.\nDecision: Keep static transform artifacts in every export.\nConsequences: Manifests carry output hashes.\n```\n\n```diff\n@@ -1 +1 @@\n-draft export\n+audited export\n```\n\n```qr\nhttps://neditor.local/export-pack\n```\n".to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    for name in ["roadmap", "adr", "diff", "qr"] {
        let artifact = response
            .transform_artifacts
            .iter()
            .find(|artifact| artifact.name == name)
            .unwrap_or_else(|| panic!("missing {name} artifact"));
        assert_eq!(artifact.execution_kind, "embedded");
        assert_eq!(artifact.source_hash.len(), 64);
        assert_eq!(artifact.output_hash.len(), 64);
        assert!(artifact.source_line.is_some());
        assert!(artifact.end_source_line.is_some());
    }

    let html = render_full_html(&response, &options);
    assert!(html.contains("transform-roadmap"));
    assert!(html.contains("roadmap-meta-status"));
    assert!(html.contains("transform-adr"));
    assert!(html.contains("adr-decision"));
    assert!(html.contains("transform-diff"));
    assert!(html.contains("1 additions / 1 deletions / 1 hunks"));
    assert!(html.contains("transform-qr"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Transform: roadmap"));
    assert!(pdf_text.contains("Launch beta"));
    assert!(pdf_text.contains("Transform: adr"));
    assert!(pdf_text.contains("Keep static transform artifacts"));
    assert!(pdf_text.contains("Transform: diff"));
    assert!(pdf_text.contains("audited export"));
    assert!(pdf_text.contains("Transform: qr"));
    assert!(pdf_text.contains("https://neditor.local/export-pack"));

    let docx = render_docx_bytes(&response, &options).expect("docx transform pack");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Transform: roadmap"));
    assert!(docx_document.contains("Launch beta"));
    assert!(docx_document.contains("Keep static transform artifacts"));
    assert!(docx_document.contains("audited export"));
    assert!(docx_document.contains("https://neditor.local/export-pack"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx transform pack");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Business Transform Pack"));
    assert!(pptx_slides.contains("Transform: roadmap"));
    assert!(pptx_slides.contains("Keep static transform artifacts"));
    assert!(pptx_slides.contains("audited export"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_artifacts = zip_entry_text(&bundle, "transform-artifacts.json");
    assert!(bundled_text.contains("Transform: roadmap"));
    assert!(bundled_text.contains("Keep static transform artifacts"));
    for name in ["roadmap", "adr", "diff", "qr"] {
        assert!(bundled_artifacts.contains(&format!("\"name\": \"{name}\"")));
    }
    assert!(bundled_artifacts.contains("\"source_line\""));
    assert!(bundled_artifacts.contains("\"output_hash\""));
}

#[test]
fn bibtex_transform_survives_cross_target_exports_with_metadata() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Bibliography Transform Pack\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-21T10:00:00Z\n---\n# Bibliography Transform Pack\n\n```bibtex\n@book{porter1985, title={Competitive Advantage}, author={Michael Porter}, year={1985}}\n@article{doe2026, title=\"Evidence Based Reports\", author=\"Jane Doe\", date=\"2026-05-21\"}\n```\n".to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert_eq!(response.bibliography.len(), 2);
    assert!(response
        .bibliography
        .iter()
        .any(|entry| entry.key == "porter1985"
            && entry.author.as_deref() == Some("Michael Porter")
            && entry.issued.as_deref() == Some("1985")));
    assert!(response
        .bibliography
        .iter()
        .any(|entry| entry.key == "doe2026"
            && entry.author.as_deref() == Some("Jane Doe")
            && entry.issued.as_deref() == Some("2026")));
    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "bibtex")
        .expect("bibtex artifact");
    assert_eq!(artifact.output_kind, "html");
    assert_eq!(artifact.execution_kind, "embedded");
    assert!(artifact.html.contains("<cite>Competitive Advantage</cite>"));
    assert!(artifact.html.contains("Michael Porter"));
    assert_eq!(artifact.source_hash.len(), 64);
    assert_eq!(artifact.output_hash.len(), 64);
    assert!(artifact.source_line.is_some());
    assert!(artifact.end_source_line.is_some());

    let html = render_full_html(&response, &options);
    assert!(html.contains("transform-bibtex"));
    assert!(html.contains("Competitive Advantage"));
    assert!(html.contains("Michael Porter"));
    assert!(html.contains("Evidence Based Reports"));
    assert!(html.contains("Jane Doe"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Transform: bibtex"));
    assert!(pdf_text.contains("Competitive Advantage"));
    assert!(pdf_text.contains("Michael Porter"));
    assert!(pdf_text.contains("Evidence Based Reports"));
    assert!(pdf_text.contains("Jane Doe"));

    let docx = render_docx_bytes(&response, &options).expect("docx bibtex transform pack");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Transform: bibtex"));
    assert!(docx_document.contains("Competitive Advantage"));
    assert!(docx_document.contains("Michael Porter"));
    assert!(docx_document.contains("Evidence Based Reports"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bibtex transform pack");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Bibliography Transform Pack"));
    assert!(pptx_slides.contains("Transform: bibtex"));
    assert!(pptx_slides.contains("Competitive Advantage"));
    assert!(pptx_slides.contains("Evidence Based Reports"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_bibliography = zip_entry_text(&bundle, "bibliography.json");
    let bundled_artifacts = zip_entry_text(&bundle, "transform-artifacts.json");
    assert!(bundled_text.contains("Transform: bibtex"));
    assert!(bundled_text.contains("Michael Porter"));
    assert!(bundled_bibliography.contains("\"key\": \"porter1985\""));
    assert!(bundled_bibliography.contains("\"author\": \"Michael Porter\""));
    assert!(bundled_bibliography.contains("\"issued\": \"1985\""));
    assert!(bundled_artifacts.contains("\"name\": \"bibtex\""));
    assert!(bundled_artifacts.contains("\"output_hash\""));
    assert!(bundled_artifacts.contains("\"source_line\""));
}

#[test]
fn api_schema_transforms_survive_cross_target_exports() {
    let response = compile(CompileRequest {
        text: r##"---
title: API Schema Transform Pack
status: approved
approvedBy: QA
approvedAt: 2026-05-21T11:00:00Z
---
# API Schema Transform Pack

```openapi
openapi: 3.1.0
info:
  title: Ledger API
  version: 1.0.0
servers:
  - url: https://api.example.test
    description: Production
security:
  - ApiKeyAuth: []
paths:
  /accounts:
    parameters:
      - name: tenant
        in: header
        required: true
        schema:
          type: string
    get:
      summary: List accounts
      operationId: listAccounts
      deprecated: true
      externalDocs:
        description: Runbook
        url: https://docs.example.test/accounts
      tags:
        - Accounts
      parameters:
        - name: limit
          in: query
          schema:
            type: integer
            maximum: 100
      responses:
        "200":
          description: Account list
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/Account"
              examples:
                success:
                  summary: Example account list
          headers:
            X-RateLimit-Remaining:
              description: Remaining calls
              schema:
                type: integer
          links:
            accountById:
              operationId: getAccount
              description: Fetch a single account
      callbacks:
        paymentStatus:
          '{$request.body#/callbackUrl}':
            post:
              summary: Payment status callback
              operationId: paymentStatusCallback
              responses:
                "204":
                  description: Accepted
webhooks:
  accountChanged:
    post:
      summary: Account changed webhook
      operationId: accountChangedWebhook
      requestBody:
        content:
          application/json:
            schema:
              oneOf:
                - $ref: "#/components/schemas/Account"
                - $ref: "#/components/schemas/AccountClosed"
              discriminator:
                propertyName: eventType
                mapping:
                  account: "#/components/schemas/Account"
                  closed: "#/components/schemas/AccountClosed"
      responses:
        "202":
          description: Queued
components:
  securitySchemes:
    ApiKeyAuth:
      type: apiKey
      in: header
      name: X-API-Key
  schemas:
    Account:
      type: object
      required:
        - id
      properties:
        id:
          type: string
          format: uuid
          description: Account id
        status:
          type: string
          enum: [active, suspended]
    AccountClosed:
      type: object
      required:
        - eventType
      properties:
        eventType:
          type: string
          const: closed
```

```json-schema
{
  "title": "Account Payload",
  "description": "Account payload contract",
  "type": "object",
  "required": ["id", "transactions"],
  "additionalProperties": false,
  "patternProperties": {
    "^x-": { "type": "string", "description": "Extension value" }
  },
  "dependentRequired": {
    "b": ["cc"]
  },
  "dependentSchemas": {
    "cc": {
      "properties": {
        "b": { "type": "string", "minLength": 3 }
      }
    }
  },
  "if": {
    "properties": {
      "status": { "const": "closed" }
    }
  },
  "then": {
    "required": ["closedAt"]
  },
  "$defs": {
    "Money": {
      "type": "number",
      "multipleOf": 0.01
    }
  },
  "properties": {
    "id": { "type": "string", "format": "uuid", "description": "Account id" },
    "balance": { "type": "number", "minimum": 0, "default": 0 },
    "metadata": {
      "type": "object",
      "additionalProperties": { "type": "string" }
    },
    "labels": {
      "type": "object",
      "propertyNames": { "pattern": "^[a-z-]+$" },
      "unevaluatedProperties": false
    },
    "attachments": {
      "type": "array",
      "minContains": 1,
      "maxContains": 3,
      "contains": {
        "type": "object",
        "required": ["kind"],
        "properties": {
          "kind": { "type": "string", "const": "invoice" }
        }
      },
      "unevaluatedItems": false
    },
    "payload": {
      "type": "string",
      "contentEncoding": "base64",
      "contentMediaType": "application/json",
      "contentSchema": {
        "type": "object",
        "properties": {
          "traceId": { "type": "string", "pattern": "^[A-Z0-9]+$" }
        }
      }
    },
    "tuple": {
      "type": "array",
      "prefixItems": [
        { "type": "string", "description": "Code" },
        { "type": "number", "multipleOf": 0.01 }
      ]
    },
    "payment": {
      "oneOf": [
        { "type": "string", "const": "cash" },
        { "$ref": "#/definitions/CardPayment" }
      ]
    },
    "transactions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["amount"],
        "properties": {
          "amount": { "type": "number" },
          "kind": { "type": "string", "enum": ["credit", "debit"] }
        }
      }
    }
  }
}
```
"##
        .to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    for name in ["openapi", "json-schema"] {
        let artifact = response
            .transform_artifacts
            .iter()
            .find(|artifact| artifact.name == name)
            .unwrap_or_else(|| panic!("missing {name} artifact"));
        assert_eq!(artifact.output_kind, "html");
        assert_eq!(artifact.execution_kind, "embedded");
        assert_eq!(artifact.source_hash.len(), 64);
        assert_eq!(artifact.output_hash.len(), 64);
        assert!(artifact.html.contains(&format!("transform-{name}")));
        assert!(artifact.source_line.is_some());
        assert!(artifact.end_source_line.is_some());
    }
    assert!(response
        .export_manifest
        .transform_artifacts
        .iter()
        .any(
            |artifact| artifact.get("name").and_then(Value::as_str) == Some("openapi")
                && artifact.get("outputKind").and_then(Value::as_str) == Some("html")
        ));

    let html = render_full_html(&response, &options);
    assert!(html.contains("transform-openapi"));
    assert!(html.contains("Ledger API"));
    assert!(html.contains("https://api.example.test"));
    assert!(html.contains("listAccounts"));
    assert!(html.contains("deprecated"));
    assert!(html.contains("Runbook"));
    assert!(html.contains("ApiKeyAuth"));
    assert!(html.contains("Security schemes"));
    assert!(html.contains("X-API-Key"));
    assert!(html.contains("tenant"));
    assert!(html.contains("array&lt;ref Account&gt;"));
    assert!(html.contains("examples: success"));
    assert!(html.contains("X-RateLimit-Remaining"));
    assert!(html.contains("getAccount"));
    assert!(html.contains("callbacks: paymentStatus"));
    assert!(html.contains("paymentStatusCallback"));
    assert!(html.contains("WEBHOOK POST"));
    assert!(html.contains("accountChangedWebhook"));
    assert!(html.contains("discriminator eventType"));
    assert!(html.contains("mapping account, closed"));
    assert!(html.contains("Component schemas"));
    assert!(html.contains("transform-json-schema"));
    assert!(html.contains("Account Payload"));
    assert!(html.contains("additionalProperties: false"));
    assert!(html.contains("unevaluatedProperties: false"));
    assert!(html.contains("unevaluatedItems: false"));
    assert!(html.contains("minContains: 1"));
    assert!(html.contains("maxContains: 3"));
    assert!(html.contains("contentEncoding: base64"));
    assert!(html.contains("contentMediaType: application/json"));
    assert!(html.contains("contentSchema: {2 fields}"));
    assert!(html.contains("dependentRequired: b -&gt; cc"));
    assert!(html.contains("patternProperties[^x-]"));
    assert!(html.contains("labels.propertyNames"));
    assert!(html.contains("attachments.contains.kind"));
    assert!(html.contains("payload.contentSchema.traceId"));
    assert!(html.contains("dependentSchemas[cc].b"));
    assert!(html.contains("if.status"));
    assert!(html.contains("then"));
    assert!(html.contains("$defs[Money]"));
    assert!(html.contains("tuple.prefixItems[1]"));
    assert!(html.contains("multipleOf: 0.01"));
    assert!(html.contains("payment.oneOf[2]"));
    assert!(html.contains("ref: CardPayment"));
    assert!(html.contains("transactions[]"));
    assert!(html.contains("enum: credit, debit"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Ledger API"));
    assert!(pdf_text.contains("listAccounts"));
    assert!(pdf_text.contains("ApiKeyAuth"));
    assert!(pdf_text.contains("X-RateLimit-Remaining"));
    assert!(pdf_text.contains("getAccount"));
    assert!(pdf_text.contains("callbacks:"));
    assert!(pdf_text.contains("paymentStatusCallback"));
    assert!(pdf_text.contains("WEBHOOK POST"));
    assert!(pdf_text.contains("accountChangedWebhook"));
    assert!(pdf_text.contains("discriminator eventType"));
    assert!(pdf_text.contains("tenant"));
    assert!(pdf_text.contains("Account id"));
    assert!(pdf_text.contains("Account Payload"));
    assert!(pdf_text.contains("additionalProperties:"));
    assert!(pdf_text.contains("unevaluatedProperties:"));
    assert!(pdf_text.contains("minContains:"));
    assert!(pdf_text.contains("contentEncoding:"));
    assert!(pdf_text.contains("false;"));
    assert!(pdf_text.contains("patternProperties[^x-]"));
    assert!(pdf_text.contains("labels.propertyNames"));
    assert!(pdf_text.contains("dependentSchemas[cc].b"));
    assert!(pdf_text.contains("if.status"));
    assert!(pdf_text.contains("then"));
    assert!(pdf_text.contains("$defs[Money]"));
    assert!(pdf_text.contains("tuple.prefixItems[1]"));
    assert!(pdf_text.contains("payment.oneOf[2]"));
    assert!(pdf_text.contains("transactions[]"));
    assert!(pdf_text.contains("enum: credit, debit"));

    let docx = render_docx_bytes(&response, &options).expect("docx api schema transform pack");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Ledger API"));
    assert!(docx_document.contains("listAccounts"));
    assert!(docx_document.contains("ApiKeyAuth"));
    assert!(docx_document.contains("X-RateLimit-Remaining"));
    assert!(docx_document.contains("paymentStatusCallback"));
    assert!(docx_document.contains("WEBHOOK POST"));
    assert!(docx_document.contains("accountChangedWebhook"));
    assert!(docx_document.contains("discriminator eventType"));
    assert!(docx_document.contains("Account id"));
    assert!(docx_document.contains("Account Payload"));
    assert!(docx_document.contains("patternProperties[^x-]"));
    assert!(docx_document.contains("labels.propertyNames"));
    assert!(docx_document.contains("attachments.contains.kind"));
    assert!(docx_document.contains("payload.contentSchema.traceId"));
    assert!(docx_document.contains("$defs[Money]"));
    assert!(docx_document.contains("tuple.prefixItems[1]"));
    assert!(docx_document.contains("payment.oneOf[2]"));
    assert!(docx_document.contains("transactions[]"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx api schema transform pack");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("API Schema Transform Pack"));
    assert!(pptx_slides.contains("Ledger API"));
    assert!(pptx_slides.contains("ApiKeyAuth"));
    assert!(pptx_slides.contains("X-RateLimit-Remaining"));
    assert!(pptx_slides.contains("paymentStatusCallback"));
    assert!(pptx_slides.contains("WEBHOOK POST"));
    assert!(pptx_slides.contains("accountChangedWebhook"));
    assert!(pptx_slides.contains("Account Payload"));
    assert!(pptx_slides.contains("patternProperties[^x-]"));
    assert!(pptx_slides.contains("labels.propertyNames"));
    assert!(pptx_slides.contains("attachments.contains.kind"));
    assert!(pptx_slides.contains("payload.contentSchema.traceId"));
    assert!(pptx_slides.contains("$defs[Money]"));
    assert!(pptx_slides.contains("tuple.prefixItems[1]"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_artifacts = zip_entry_text(&bundle, "transform-artifacts.json");
    assert!(bundled_text.contains("Ledger API"));
    assert!(bundled_text.contains("ApiKeyAuth"));
    assert!(bundled_text.contains("X-RateLimit-Remaining"));
    assert!(bundled_text.contains("paymentStatusCallback"));
    assert!(bundled_text.contains("WEBHOOK POST"));
    assert!(bundled_text.contains("accountChangedWebhook"));
    assert!(bundled_text.contains("Account Payload"));
    assert!(bundled_text.contains("patternProperties[^x-]"));
    assert!(bundled_text.contains("labels.propertyNames"));
    assert!(bundled_text.contains("attachments.contains.kind"));
    assert!(bundled_text.contains("payload.contentSchema.traceId"));
    assert!(bundled_text.contains("$defs[Money]"));
    assert!(bundled_text.contains("payment.oneOf[2]"));
    for name in ["openapi", "json-schema"] {
        assert!(bundled_artifacts.contains(&format!("\"name\": \"{name}\"")));
    }
    assert!(bundled_artifacts.contains("\"output_kind\": \"html\""));
    assert!(bundled_artifacts.contains("\"source_line\""));
    assert!(bundled_artifacts.contains("\"output_hash\""));
}

#[test]
fn visual_data_transforms_survive_cross_target_exports() {
    let response = compile(CompileRequest {
        text: r#"---
title: Visual Transform Pack
status: approved
approvedBy: QA
approvedAt: 2026-05-21T09:00:00Z
---
# Visual Transform Pack

```chart
type: area
title: Revenue Plan
data:
  - month: Jan
    revenue: 4
  - month: Feb
    revenue: 7
x: month
y: revenue
```

```vega-lite
{"title":"Pipeline Trend","mark":"line","data":{"values":[{"month":"Jan","value":3},{"month":"Feb","value":8}]},"encoding":{"x":{"field":"month"},"y":{"field":"value"}}}
```

```pikchr
box "Intake"; arrow "approve"; diamond "Gate"; arrow; cylinder "Store"; arrow; file "Export"
```

```geojson
{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Polygon","coordinates":[[[36.80,-1.30],[36.86,-1.30],[36.86,-1.24],[36.80,-1.30]]]}},{"type":"Feature","geometry":{"type":"Point","coordinates":[36.83,-1.27]}}]}
```

```topojson
{"type":"Topology","transform":{"scale":[0.01,0.01],"translate":[36.80,-1.30]},"arcs":[[[0,0],[6,0],[0,6],[-6,-6]]],"objects":{"zone":{"type":"Polygon","arcs":[[0]]}}}
```

```stl
solid sample
 facet normal 0 0 1
  outer loop
   vertex 0 0 0
   vertex 1 0 0
   vertex 0 1 0
  endloop
 endfacet
endsolid sample
```

```timeline
2026-05-21: Export proof
2026-06-01: Visual QA
```
"#
        .to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    for (name, output_kind) in [
        ("chart", "svg"),
        ("vega-lite", "svg"),
        ("pikchr", "svg"),
        ("geojson", "svg"),
        ("topojson", "svg"),
        ("stl", "svg"),
        ("timeline", "svg"),
    ] {
        let artifact = response
            .transform_artifacts
            .iter()
            .find(|artifact| artifact.name == name)
            .unwrap_or_else(|| panic!("missing {name} artifact"));
        assert_eq!(artifact.output_kind, output_kind);
        assert_eq!(artifact.execution_kind, "embedded");
        assert_eq!(artifact.source_hash.len(), 64);
        assert_eq!(artifact.output_hash.len(), 64);
        assert!(artifact.html.contains(&format!("transform-{name}")));
        assert!(artifact.source_line.is_some());
        assert!(artifact.end_source_line.is_some());
    }
    assert!(response
        .export_manifest
        .transform_artifacts
        .iter()
        .any(
            |artifact| artifact.get("name").and_then(Value::as_str) == Some("vega-lite")
                && artifact.get("outputKind").and_then(Value::as_str) == Some("svg")
        ));

    let html = render_full_html(&response, &options);
    assert!(html.contains("transform-chart"));
    assert!(html.contains("Revenue Plan"));
    assert!(html.contains("transform-vega-lite"));
    assert!(html.contains("Pipeline Trend"));
    assert!(html.contains("transform-pikchr"));
    assert!(html.contains("pikchr-diamond"));
    assert!(html.contains("approve"));
    assert!(html.contains("transform-geojson"));
    assert!(html.contains("1 polygons / 1 points"));
    assert!(html.contains("transform-topojson"));
    assert!(html.contains("1 polygons"));
    assert!(html.contains("transform-stl"));
    assert!(html.contains("1 triangles / 3 vertices"));
    assert!(html.contains("transform-timeline"));
    assert!(html.contains("Export proof"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Transform: chart"));
    assert!(pdf_text.contains("Revenue Plan"));
    assert!(pdf_text.contains("Transform: vega-lite"));
    assert!(pdf_text.contains("Pipeline Trend"));
    assert!(pdf_text.contains("Transform: pikchr"));
    assert!(pdf_text.contains("Gate"));
    assert!(pdf_text.contains("Transform: geojson"));
    assert!(pdf_text.contains("1 polygons / 1 points"));
    assert!(pdf_text.contains("Transform: topojson"));
    assert!(pdf_text.contains("1 polygons"));
    assert!(pdf_text.contains("Transform: stl"));
    assert!(pdf_text.contains("1 triangles / 3 vertices"));
    assert!(pdf_text.contains("Transform: timeline"));
    assert!(pdf_text.contains("Export proof"));

    let docx = render_docx_bytes(&response, &options).expect("docx visual transform pack");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Transform: chart"));
    assert!(docx_document.contains("Revenue Plan"));
    assert!(docx_document.contains("Pipeline Trend"));
    assert!(docx_document.contains("Transform: pikchr"));
    assert!(docx_document.contains("Gate"));
    assert!(docx_document.contains("1 polygons / 1 points"));
    assert!(docx_document.contains("1 triangles / 3 vertices"));
    assert!(docx_document.contains("Export proof"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx visual transform pack");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Visual Transform Pack"));
    assert!(pptx_slides.contains("Transform: chart"));
    assert!(pptx_slides.contains("Pipeline Trend"));
    assert!(pptx_slides.contains("Transform: pikchr"));
    assert!(pptx_slides.contains("Gate"));
    assert!(pptx_slides.contains("1 polygons / 1 points"));
    assert!(pptx_slides.contains("1 triangles / 3 vertices"));
    assert!(pptx_slides.contains("Export proof"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_artifacts = zip_entry_text(&bundle, "transform-artifacts.json");
    assert!(bundled_text.contains("Transform: chart"));
    assert!(bundled_text.contains("Pipeline Trend"));
    assert!(bundled_text.contains("Transform: pikchr"));
    assert!(bundled_text.contains("1 triangles / 3 vertices"));
    for name in [
        "chart",
        "vega-lite",
        "pikchr",
        "geojson",
        "topojson",
        "stl",
        "timeline",
    ] {
        assert!(bundled_artifacts.contains(&format!("\"name\": \"{name}\"")));
    }
    assert!(bundled_artifacts.contains("\"output_kind\": \"svg\""));
    assert!(bundled_artifacts.contains("\"source_line\""));
    assert!(bundled_artifacts.contains("\"output_hash\""));
}

#[test]
fn captioned_equations_survive_cross_target_exports() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Captioned Equation\nstatus: approved\napprovedBy: QA\n---\n# Captioned Equation\n\n$$\nconfidence = signal / noise\n$$ {#eq:confidence caption=\"Confidence score\"}\n\nSee {@eq:confidence}.\n".to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Equation is missing a stable label or caption."));
    assert!(response.html.contains("Equation 1: Confidence score"));
    assert!(response.html.contains("data-caption=\"Confidence score\""));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Equation { id, caption, text, .. }
                if id.as_deref() == Some("eq:confidence")
                    && caption.as_deref() == Some("Confidence score")
                    && text.contains("confidence = signal / noise")
        )
    }));
    assert!(response
        .compiled_markdown
        .contains("[Equation confidence](#eq:confidence)"));

    let html = render_full_html(&response, &options);
    assert!(html.contains("Equation 1: Confidence score"));
    assert!(html.contains(r##"<a href="#eq:confidence">Equation confidence</a>"##));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Equation confidence"));
    assert!(pdf_text.contains("Confidence score"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Equation confidence"));
    assert!(docx_document.contains("Confidence score"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Equation confidence"));
    assert!(pptx_slides.contains("Confidence score"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options;
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    assert!(bundled_text.contains("Equation: eq:confidence"));
    assert!(bundled_text.contains("Confidence score"));
    assert!(bundled_ast.contains("\"caption\": \"Confidence score\""));
}

#[test]
fn generated_toc_exports_page_numbers_for_pdf_and_docx() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Page TOC\nstatus: approved\napprovedBy: QA\ntoc: true\ntocDepth: 2\ntocNumbered: true\n---\n# Alpha\nIntro.\n\n{{page-break}}\n\n## Beta\nDetails.\n".to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(response.compiled_markdown.contains("- [1 Alpha](#alpha)"));
    assert!(response.compiled_markdown.contains("- [1.1 Beta](#beta)"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("- 1 Alpha .... 2"));
    assert!(pdf_text.contains("- 1.1 Beta .... 3"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains(r#"w:instr="TOC \o &quot;1-2&quot; \h \z \u""#));
    assert!(docx_document.contains("Update table of contents in Word to refresh page numbers."));
    assert!(!docx_document.contains("[1 Alpha](#alpha)"));
}

#[test]
fn named_table_formulas_survive_cross_target_exports() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Formula Export\nstatus: approved\napprovedBy: QA\n---\n# Formula Export\n\nTable: Quarterly revenue {#tbl:quarterly}\n| Quarter | Revenue |\n| --- | ---: |\n| Q1 | 100 |\n| Q2 | 180 |\n| Total | =SUM(B1:B2) |\n\nTable: Board summary {#tbl:summary}\n| Metric | Value |\n| --- | ---: |\n| Revenue rollup | =SUM(tbl:quarterly!B1:B3) |\n| Reported total | =quarterly!B3 |\n".to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response
        .compiled_markdown
        .contains("| Revenue rollup | 560 |"));
    assert!(response
        .compiled_markdown
        .contains("| Reported total | 280 |"));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Table { id, rows, .. }
                if id.as_deref() == Some("tbl:summary")
                    && rows.iter().any(|row| row == &vec![
                        "Revenue rollup".to_string(),
                        "560".to_string()
                    ])
                    && rows.iter().any(|row| row == &vec![
                        "Reported total".to_string(),
                        "280".to_string()
                    ])
        )
    }));

    let html = render_full_html(&response, &options);
    assert!(html.contains("Revenue rollup"));
    assert!(html.contains(">560</td>"));
    assert!(html.contains(">280</td>"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Board summary"));
    assert!(pdf_text.contains("(Revenue rollup) Tj"));
    assert!(pdf_text.contains("(560) Tj"));
    assert!(pdf_text.contains("(280) Tj"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Board summary"));
    assert!(docx_document.contains(">560<"));
    assert!(docx_document.contains(">280<"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Board summary"));
    assert!(pptx_slides.contains("<a:t>560</a:t>"));
    assert!(pptx_slides.contains("<a:t>280</a:t>"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options;
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    assert!(bundled_text.contains("Revenue rollup | 560"));
    assert!(bundled_text.contains("Reported total | 280"));
    assert!(bundled_ast.contains("\"id\": \"tbl:summary\""));
    assert!(bundled_ast.contains("560"));
    assert!(bundled_ast.contains("280"));
}

#[test]
fn front_matter_data_sources_survive_cross_target_exports() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-data-source-export-{unique}"));
    fs::create_dir_all(root.join("data")).expect("create data source export dir");
    fs::write(
        root.join("data").join("accounts.json"),
        r#"[{"account":"Acme","region":"EA","arr":1200},{"account":"Globex","region":"WE","arr":900}]"#,
    )
    .expect("write export json data source");
    fs::write(
        root.join("data").join("settings.yaml"),
        "owner: Strategy Office\ncadence: monthly\n",
    )
    .expect("write export yaml data source");
    fs::write(
        root.join("data").join("revenue.csv"),
        "Metric,Value\nRevenue,450\n",
    )
    .expect("write export csv data source");
    fs::write(
        root.join("data").join("targets.tsv"),
        "Metric\tValue\nPipeline\t375\n",
    )
    .expect("write export tsv data source");

    let response = compile(CompileRequest {
        text: "---\ntitle: Data Source Export\nstatus: approved\napprovedBy: QA\ndataSources:\n  - name: Accounts\n    path: data/accounts.json\n  - name: Settings\n    path: data/settings.yaml\ncsvFiles:\n  - data/revenue.csv\ntsvFiles:\n  - data/targets.tsv\n---\n# Data Source Export\n\nThe appendix is generated from local front matter data sources.\n".to_string(),
        file_path: Some(path_to_string(&root.join("report.md"))),
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "json"));
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "yaml"));
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "csv"));
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "tsv"));
    assert!(response
        .export_manifest
        .included_files
        .iter()
        .any(|file| file.path.ends_with("data/accounts.json")));

    let html = render_full_html(&response, &options);
    assert!(html.contains("Data Source: Accounts"));
    assert!(html.contains("<td>Acme</td>"));
    assert!(html.contains("<td>1200</td>"));
    assert!(html.contains("Data Source: Settings"));
    assert!(html.contains("<dt>owner</dt>"));
    assert!(html.contains("Data Source: revenue"));
    assert!(html.contains("<td>Revenue</td>"));
    assert!(html.contains("<td>450</td>"));
    assert!(html.contains("Data Source: targets"));
    assert!(html.contains("<td>Pipeline</td>"));
    assert!(html.contains("<td>375</td>"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Data Source: Accounts"));
    assert!(pdf_text.contains("(Acme) Tj"));
    assert!(pdf_text.contains("(1200) Tj"));
    assert!(pdf_text.contains("Data Source: Settings"));
    assert!(pdf_text.contains("Strategy Office"));
    assert!(pdf_text.contains("(Revenue) Tj"));
    assert!(pdf_text.contains("(450) Tj"));
    assert!(pdf_text.contains("(Pipeline) Tj"));
    assert!(pdf_text.contains("(375) Tj"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Data Source: Accounts"));
    assert!(docx_document.contains(">Acme<"));
    assert!(docx_document.contains(">1200<"));
    assert!(docx_document.contains("Data Source: Settings"));
    assert!(docx_document.contains("Strategy Office"));
    assert!(docx_document.contains(">Revenue<"));
    assert!(docx_document.contains(">450<"));
    assert!(docx_document.contains(">Pipeline<"));
    assert!(docx_document.contains(">375<"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Data Source: Accounts"));
    assert!(pptx_slides.contains("<a:t>Acme</a:t>"));
    assert!(pptx_slides.contains("<a:t>1200</a:t>"));
    assert!(pptx_slides.contains("Data Source: Settings"));
    assert!(pptx_slides.contains("Strategy Office"));
    assert!(pptx_slides.contains("<a:t>Revenue</a:t>"));
    assert!(pptx_slides.contains("<a:t>450</a:t>"));
    assert!(pptx_slides.contains("<a:t>Pipeline</a:t>"));
    assert!(pptx_slides.contains("<a:t>375</a:t>"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options;
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_manifest = zip_entry_text(&bundle, "manifest.json");
    assert!(bundled_text.contains("Data Source: Accounts"));
    assert!(bundled_text.contains("Acme"));
    assert!(bundled_text.contains("1200"));
    assert!(bundled_text.contains("Data Source: Settings"));
    assert!(bundled_text.contains("Strategy Office"));
    assert!(bundled_text.contains("Revenue"));
    assert!(bundled_text.contains("450"));
    assert!(bundled_text.contains("Pipeline"));
    assert!(bundled_text.contains("375"));
    assert!(bundled_manifest.contains("data/accounts.json"));
    assert!(bundled_manifest.contains("data/settings.yaml"));
    assert!(bundled_manifest.contains("data/revenue.csv"));
    assert!(bundled_manifest.contains("data/targets.tsv"));

    fs::remove_dir_all(root).expect("clean data source export dir");
}

#[test]
fn formatted_document_variables_survive_cross_target_exports() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Variable Export\nstatus: approved\napprovedBy: QA\nclient: acme holdings\nregion: ' east africa '\nbudget: 1234.5\nmargin: 0.275\n---\n# Variable Export\n\nClient: {{client | title}}\nRegion: {{region | trim | upper}}\nBudget: {{budget | currency}}\nMargin: {{margin | percent}}\nOwner: {{owner | default:'strategy office' | title}}\n".to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response.compiled_markdown.contains("Client: Acme Holdings"));
    assert!(response.compiled_markdown.contains("Region: EAST AFRICA"));
    assert!(response.compiled_markdown.contains("Budget: $1234.50"));
    assert!(response.compiled_markdown.contains("Margin: 27.50%"));
    assert!(response
        .compiled_markdown
        .contains("Owner: Strategy Office"));

    let html = render_full_html(&response, &options);
    assert!(html.contains("Client: Acme Holdings"));
    assert!(html.contains("Region: EAST AFRICA"));
    assert!(html.contains("Budget: $1234.50"));
    assert!(html.contains("Margin: 27.50%"));
    assert!(html.contains("Owner: Strategy Office"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Client: Acme Holdings"));
    assert!(pdf_text.contains("Region: EAST AFRICA"));
    assert!(pdf_text.contains("Budget: $1234.50"));
    assert!(pdf_text.contains("Margin: 27.50%"));
    assert!(pdf_text.contains("Owner:"));
    assert!(pdf_text.contains("Strategy"));
    assert!(pdf_text.contains("Office"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Client: Acme Holdings"));
    assert!(docx_document.contains("Region: EAST AFRICA"));
    assert!(docx_document.contains("Budget: $1234.50"));
    assert!(docx_document.contains("Margin: 27.50%"));
    assert!(docx_document.contains("Owner: Strategy Office"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Client: Acme Holdings"));
    assert!(pptx_slides.contains("Region: EAST AFRICA"));
    assert!(pptx_slides.contains("Budget: $1234.50"));
    assert!(pptx_slides.contains("Margin: 27.50%"));
    assert!(pptx_slides.contains("Owner: Strategy Office"));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options;
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    assert!(bundled_text.contains("Client: Acme Holdings"));
    assert!(bundled_text.contains("Region: EAST AFRICA"));
    assert!(bundled_text.contains("Budget: $1234.50"));
    assert!(bundled_text.contains("Margin: 27.50%"));
    assert!(bundled_text.contains("Owner: Strategy Office"));
}

#[test]
fn heading_appendix_and_decision_references_survive_cross_target_exports() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Reference Export\nstatus: approved\napprovedBy: QA\n---\n# Strategy {#sec:strategy}\nSee {@sec:strategy}, {@appendix-a}, and {@decision-record}.\n\n## Appendix A\nSupporting detail.\n\n## Decision Record\nUse local-first exports.\n".to_string(),
        file_path: None,
    });
    let options = json!({});

    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));
    assert!(response.compiled_markdown.contains(
        "See [Section strategy](#sec:strategy), [Appendix A](#appendix-a), and [Decision Record](#decision-record)."
    ));

    let html = render_full_html(&response, &options);
    assert!(html.contains(r##"<a href="#sec:strategy">Section strategy</a>"##));
    assert!(html.contains(r##"<a href="#appendix-a">Appendix A</a>"##));
    assert!(html.contains(r##"<a href="#decision-record">Decision Record</a>"##));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Section strategy"));
    assert!(pdf_text.contains("Appendix A"));
    assert!(pdf_text.contains("Decision Record"));

    let docx = render_docx_bytes(&response, &options).expect("docx reference bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Section strategy"));
    assert!(docx_document.contains("Appendix A"));
    assert!(docx_document.contains("Decision Record"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx reference bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Section strategy"));
    assert!(pptx_slides.contains("Appendix A"));
    assert!(pptx_slides.contains("Decision Record"));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("reference bundle bytes");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    assert!(bundled_text.contains("Section strategy"));
    assert!(bundled_text.contains("Appendix A"));
    assert!(bundled_text.contains("Decision Record"));
}

#[test]
fn front_matter_index_survives_cross_target_exports() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Index Export\nstatus: approved\napprovedBy: QA\nindex: true\n---\n# Market Analysis\nAcme Strategy appears here. **Working Capital** matters.\n\n## Follow Up\nAcme Strategy returns. Working capital{#index:Liquidity} marker.\n".to_string(),
            file_path: None,
        });
    let options = json!({});

    assert!(response.compiled_markdown.starts_with("## Index\n\n"));
    assert!(response
        .compiled_markdown
        .contains("- [Acme Strategy](#market-analysis)"));
    assert!(response
        .compiled_markdown
        .contains("- [Liquidity](#follow-up)"));
    assert!(!response.compiled_markdown.contains("[INDEX]"));
    assert!(!response.compiled_markdown.contains("{#index:Liquidity}"));

    let html = render_full_html(&response, &options);
    assert!(html.contains("<h2 id=\"index\">Index</h2>"));
    assert!(html.contains(r##"<a href="#market-analysis">Acme Strategy</a>"##));
    assert!(html.contains(r##"<a href="#follow-up">Liquidity</a>"##));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Index"));
    assert!(pdf_text.contains("Acme Strategy"));
    assert!(pdf_text.contains("Liquidity"));

    let docx = render_docx_bytes(&response, &options).expect("docx index bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Index"));
    assert!(docx_document.contains("Acme Strategy"));
    assert!(docx_document.contains("Liquidity"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx index bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Index"));
    assert!(pptx_slides.contains("Acme Strategy"));
    assert!(pptx_slides.contains("Liquidity"));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("index bundle bytes");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    assert!(bundled_text.contains("Index"));
    assert!(bundled_text.contains("Acme Strategy"));
    assert!(bundled_text.contains("Liquidity"));
}

#[test]
fn export_conformance_fixture_maps_business_features() {
    let response = compile(CompileRequest {
        text: include_str!("../../fixtures/export/business_report.md").to_string(),
        file_path: None,
    });
    let options = json!({
        "watermark": "APPROVED",
        "includeGlossary": true,
        "includeComments": true,
        "includeProvenance": true
    });

    assert_eq!(response.semantic.title, "Export Conformance Report");
    assert_eq!(response.semantic.status, "approved");
    assert_eq!(response.export_manifest.document_version, "2.0.0");
    assert_eq!(
        response
            .metadata
            .get("legalDisclaimer")
            .and_then(Value::as_str),
        Some("For board review only. Do not distribute externally without approval.")
    );
    assert!(response
        .semantic
        .citations
        .iter()
        .any(|citation| citation == "porter1985"));
    assert!(response.semantic.glossary.contains_key("ARR"));
    assert!(response.semantic.comments.iter().any(|comment| comment
        .text
        .contains("board-pack export fidelity")
        && comment.state == "resolved"));
    assert!(response
        .semantic
        .change_notes
        .iter()
        .any(|note| note.text.contains("export conformance evidence")));
    assert!(response
        .semantic
        .ai_sources
        .iter()
        .any(|source| source.provider == "OpenAI" && source.status == "human-reviewed"));
    assert!(response
        .semantic
        .ai_sources
        .iter()
        .any(|source| source.prompt_summary == "board-pack synthesis"));
    assert!(response
        .semantic
        .ai_sources
        .iter()
        .any(|source| source.line > 0 && source.reviewed_at == "2026-05-18T12:00:00Z"));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::ReviewComment { comment, .. }
                if comment.text.contains("board-pack export fidelity")
                    && comment.state == "resolved"
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::ChangeNote { note, .. }
                if note.text.contains("export conformance evidence")
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::AiSource { provenance, .. }
                if provenance.provider == "OpenAI"
                    && provenance.model == "gpt-5.4"
                    && provenance.status == "human-reviewed"
        )
    }));
    assert_eq!(response.semantic.tables, 1);
    assert_eq!(response.semantic.figures, 1);
    assert_eq!(response.semantic.equations, 1);
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Layout { directive, .. } if directive == "page-break"
        )
    }));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == "error"));

    let html = render_full_html(&response, &options);
    assert!(html.contains("Board Pack Fixture"));
    assert!(html.contains("APPROVED"));
    assert!(html.contains("Competitive Advantage, p. 42"));
    assert!(html.contains("Reference architecture"));
    assert!(html.contains(r##"<a href="#fig:architecture">Figure architecture</a>"##));
    assert!(html.contains(r##"<a href="#eq:roi">Equation roi</a>"##));
    assert!(html.contains("Competitive Advantage"));
    assert!(html.contains("class=\"export-glossary\""));
    assert!(html.contains("<dt>ARR</dt>"));
    assert!(html.contains("class=\"export-comments\""));
    assert!(html.contains("Verify board-pack export fidelity."));
    assert!(html.contains("Added export conformance evidence."));
    assert!(html.contains("class=\"export-provenance\""));
    assert!(html.contains("gpt-5.4"));
    assert!(html.contains("class=\"export-legal\""));
    assert!(html.contains("For board review only. Do not distribute externally without approval."));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf.starts_with(b"%PDF-1.4"));
    assert!(pdf_text.contains("/Count 7"));
    assert!(pdf_text.contains("/Title (Export Conformance Report)"));
    assert!(pdf_text.contains("/Keywords (approved; 2.0.0; restricted)"));
    assert!(pdf_text.contains("Export Conformance Report | restricted"));
    assert!(pdf_text.contains("Page 7 of 7"));
    assert!(pdf_text.contains("Export Conformance Report"));
    assert!(pdf_text.contains("Competitive Advantage"));
    assert!(pdf_text.contains("Competitive Advantage, p."));
    assert!(pdf_text.contains("42\\)"));
    assert!(pdf_text.contains(" re S"));
    assert!(pdf_text.contains("(Region) Tj"));
    assert!(pdf_text.contains("Reference architecture"));
    assert!(pdf_text.contains("Figure architecture"));
    assert!(pdf_text.contains("Equation roi"));
    assert!(pdf_text.contains("Glossary"));
    assert!(pdf_text.contains("Review Comments"));
    assert!(pdf_text.contains("Change Notes"));
    assert!(pdf_text.contains("AI Provenance"));
    assert!(pdf_text.contains("Legal Disclaimer"));
    assert!(
        pdf_text.contains("For board review only. Do not distribute externally without approval.")
    );

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
    let docx_header = zip_entry_text(&docx, "word/header1.xml");
    let docx_footer = zip_entry_text(&docx, "word/footer1.xml");
    let docx_comments = zip_entry_text(&docx, "word/comments.xml");
    let docx_app = zip_entry_text(&docx, "docProps/app.xml");
    let docx_core = zip_entry_text(&docx, "docProps/core.xml");
    let docx_custom = zip_entry_text(&docx, "docProps/custom.xml");
    let docx_svg = zip_entry_text(&docx, "word/media/image1.svg");
    assert!(docx_content_types.contains(r#"ContentType="image/svg+xml""#));
    assert!(docx_content_types.contains(
        r#"ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml""#
    ));
    assert!(docx_content_types.contains(
            r#"ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml""#
        ));
    assert!(docx_relationships.contains(r#"Id="rIdImage1""#));
    assert!(docx_relationships.contains(r#"Target="media/image1.svg""#));
    assert!(docx_relationships.contains(
        r#"Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments""#
    ));
    assert!(docx_document.contains(r#"r:embed="rIdImage1""#));
    assert_eq!(docx_svg, "<svg/>");
    assert!(docx_document.contains(r#"<w:pStyle w:val="Heading1""#));
    assert!(docx_document.contains("w:headerReference"));
    assert!(docx_document.contains("w:footerReference"));
    assert!(docx_document.contains(r#"<w:commentRangeStart w:id="0""#));
    assert!(docx_document.contains(r#"<w:commentReference w:id="0""#));
    assert!(docx_document.contains(r#"<w:commentRangeStart w:id="1""#));
    assert!(docx_document.contains(r#"<w:commentReference w:id="1""#));
    assert!(docx_comments.contains(r#"<w:comment w:id="0" w:author="QA""#));
    assert!(docx_comments.contains("Verify board-pack export fidelity."));
    assert!(docx_comments.contains(r#"<w:comment w:id="1" w:author="QA""#));
    assert!(docx_comments.contains("Change note: Added export conformance evidence."));
    assert!(docx_core.contains("<dc:creator>QA</dc:creator>"));
    assert!(docx_app.contains("<Application>NEditor</Application>"));
    assert!(docx_app.contains("<Company>Acme Strategy</Company>"));
    assert!(docx_custom.contains(r#"name="NEditorStatus""#));
    assert!(docx_custom.contains("<vt:lpwstr>approved</vt:lpwstr>"));
    assert!(docx_custom.contains(r#"name="NEditorVersion""#));
    assert!(docx_custom.contains("<vt:lpwstr>2.0.0</vt:lpwstr>"));
    assert!(docx_custom.contains(r#"name="NEditorApprovedBy""#));
    assert!(docx_custom.contains("<vt:lpwstr>QA</vt:lpwstr>"));
    assert!(docx_custom.contains(r#"name="NEditorApprovedAt""#));
    assert!(docx_custom.contains("<vt:lpwstr>2026-05-18T12:00:00Z</vt:lpwstr>"));
    assert!(docx_custom.contains(r#"name="NEditorLegalDisclaimer""#));
    assert!(docx_custom
        .contains("For board review only. Do not distribute externally without approval."));
    let docx_without_comments = render_docx_bytes(
        &response,
        &json!({
            "watermark": "APPROVED",
            "includeGlossary": true,
            "includeComments": false,
            "includeProvenance": true
        }),
    )
    .expect("docx bytes without comments");
    assert!(!zip_has_entry(&docx_without_comments, "word/comments.xml"));
    assert!(
        !zip_entry_text(&docx_without_comments, "[Content_Types].xml")
            .contains("wordprocessingml.comments+xml")
    );
    assert!(docx_header.contains("Export Conformance Report | restricted"));
    assert!(docx_footer.contains(r#"w:instr="PAGE""#));
    assert!(docx_footer.contains(r#"w:instr="NUMPAGES""#));
    assert!(docx_document.contains("<w:tbl>"));
    assert!(docx_document.contains(r#"<w:br w:type="page""#));
    assert!(docx_document.contains("Competitive Advantage, p. 42"));
    assert!(docx_document.contains("Reference architecture"));
    assert!(docx_document.contains("Figure architecture"));
    assert!(docx_document.contains("Equation roi"));
    assert!(docx_document.contains(r#"w:name="fig_architecture""#));
    assert!(docx_document.contains(r#"w:name="eq_roi""#));
    assert!(docx_document.contains(r#"<w:hyperlink w:anchor="fig_architecture""#));
    assert!(docx_document.contains(r#"<w:hyperlink w:anchor="eq_roi""#));
    assert!(docx_document.contains("Competitive Advantage"));
    assert!(docx_document.contains("Annual recurring revenue"));
    assert!(docx_document.contains("Review Comments"));
    assert!(docx_document.contains("Verify board-pack export fidelity."));
    assert!(docx_document.contains("Change Notes"));
    assert!(docx_document.contains("Added export conformance evidence."));
    assert!(docx_document.contains("AI Provenance"));
    assert!(docx_document.contains("gpt-5.4"));
    assert!(docx_document.contains("Legal Disclaimer"));
    assert!(docx_document
        .contains("For board review only. Do not distribute externally without approval."));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_content_types = zip_entry_text(&pptx, "[Content_Types].xml");
    let pptx_presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
    let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
    let pptx_core = zip_entry_text(&pptx, "docProps/core.xml");
    let pptx_custom = zip_entry_text(&pptx, "docProps/custom.xml");
    let pptx_agenda_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    let pptx_slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
    let pptx_slide_three_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide3.xml.rels");
    let pptx_svg = zip_entry_text(&pptx, "ppt/media/image1.svg");
    let pptx_slide_part_count = zip_entry_count_with_prefix(&pptx, "ppt/slides/slide", ".xml");
    let pptx_media_part_count = zip_entry_count_with_prefix(&pptx, "ppt/media/", "");
    assert!(pptx_content_types.contains(r#"ContentType="image/svg+xml""#));
    assert!(pptx_content_types.contains(
        r#"ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml""#
    ));
    assert!(pptx_presentation.contains(r#"r:id="rId2""#));
    assert!(pptx_core.contains("<dc:creator>QA</dc:creator>"));
    assert!(pptx_custom.contains(r#"name="NEditorApprovedBy""#));
    assert!(pptx_custom.contains(r#"name="NEditorApprovedAt""#));
    assert!(pptx_custom.contains(r#"name="NEditorLegalDisclaimer""#));
    assert!(pptx_custom
        .contains("For board review only. Do not distribute externally without approval."));
    assert!(pptx_app.contains("<Application>NEditor</Application>"));
    assert!(pptx_app.contains(&format!("<Slides>{pptx_slide_part_count}</Slides>")));
    assert_eq!(pptx_media_part_count, 2);
    assert!(pptx_agenda_slide.contains("Agenda"));
    assert!(pptx_agenda_slide.contains("Export Conformance Report"));
    assert!(pptx_agenda_slide.contains("Appendix"));
    assert!(pptx_slide_three.contains("Export Conformance Report"));
    assert!(pptx_slide_three.contains("Competitive Advantage, p. 42"));
    assert!(pptx_slide_three.contains("Figure architecture"));
    assert!(pptx_slide_three.contains("Equation roi"));
    assert!(pptx_slide_three.contains("Table: Region | Revenue | Margin"));
    assert!(pptx_slide_three.contains("<a:tbl>"));
    assert!(pptx_slide_three.contains(r#"<a:pPr algn="r"/>"#));
    assert!(pptx_slide_three.contains("Reference architecture"));
    assert!(pptx_slide_three.contains(r#"name="Header""#));
    assert!(pptx_slide_three.contains(r#"name="Footer""#));
    assert!(pptx_slide_three.contains("Page 3 of 10"));
    assert!(pptx_slide_three.contains(r#"r:embed="rIdImage1""#));
    assert!(pptx_slide_three_relationships.contains(r#"Target="../media/image1.svg""#));
    assert_eq!(pptx_svg, "<svg/>");
    let pptx_glossary_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
        .into_iter()
        .find(|slide| slide.contains("Glossary"))
        .expect("glossary slide");
    assert!(pptx_glossary_slide.contains("Annual recurring revenue"));
    let pptx_comments_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
        .into_iter()
        .find(|slide| slide.contains("Review Comments"))
        .expect("comments slide");
    assert!(pptx_comments_slide.contains("Verify board-pack export fidelity."));
    assert!(pptx_comments_slide.contains("Change Notes"));
    assert!(pptx_comments_slide.contains("Added export conformance evidence."));
    let pptx_provenance_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
        .into_iter()
        .find(|slide| slide.contains("AI Provenance"))
        .expect("provenance slide");
    assert!(pptx_provenance_slide.contains("gpt-5.4"));
    let pptx_legal_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
        .into_iter()
        .find(|slide| slide.contains("Legal Disclaimer"))
        .expect("legal disclaimer slide");
    assert!(pptx_legal_slide
        .contains("For board review only. Do not distribute externally without approval."));

    let exported_text = export::export_text(&response, &options);
    assert!(exported_text.contains("Glossary"));
    assert!(exported_text.contains("ARR: Annual recurring revenue"));
    assert!(exported_text.contains("Review Comments"));
    assert!(exported_text.contains("Change Notes"));
    assert!(exported_text.contains("AI Provenance"));
    assert!(exported_text.contains("Legal Disclaimer"));
    assert!(exported_text
        .contains("For board review only. Do not distribute externally without approval."));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_markdown = zip_entry_text(&bundle, "document.md");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_manifest = zip_entry_text(&bundle, "manifest.json");
    let bundled_semantic = zip_entry_text(&bundle, "semantic.json");
    let bundled_metadata = zip_entry_text(&bundle, "metadata.json");
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    let bundled_source_map = zip_entry_text(&bundle, "source-map.json");
    let bundled_diagnostics = zip_entry_text(&bundle, "diagnostics.json");
    let bundled_bibliography = zip_entry_text(&bundle, "bibliography.json");
    let bundled_formula_graph = zip_entry_text(&bundle, "formula-graph.json");
    let bundled_transform_artifacts = zip_entry_text(&bundle, "transform-artifacts.json");
    let bundled_media_map = zip_entry_text(&bundle, "media-map.json");
    let bundled_svg = zip_entry_text(&bundle, "media/image1.svg");
    assert!(bundled_markdown.contains("Competitive Advantage"));
    assert!(bundled_text.contains("Figure: fig:architecture: Reference architecture"));
    assert!(bundled_text.contains("Verify board-pack export fidelity."));
    assert!(bundled_text.contains("OpenAI / gpt-5.4"));
    assert!(bundled_manifest.contains("\"document_title\": \"Export Conformance Report\""));
    assert!(bundled_semantic.contains("\"title\": \"Export Conformance Report\""));
    assert!(bundled_semantic.contains("\"comments\""));
    assert!(bundled_metadata.contains("\"classification\": \"restricted\""));
    assert!(bundled_metadata.contains("\"approvedBy\": \"QA\""));
    assert!(bundled_metadata.contains("\"approvedAt\": \"2026-05-18T12:00:00Z\""));
    assert!(bundled_metadata.contains(
        "\"legalDisclaimer\": \"For board review only. Do not distribute externally without approval.\""
    ));
    assert!(bundled_ast.contains("\"kind\": \"figure\""));
    assert!(bundled_ast.contains("\"source_file\""));
    assert!(bundled_source_map.contains("\"generated_line\""));
    assert!(bundled_source_map.contains("\"source_line\""));
    assert!(bundled_diagnostics.starts_with('['));
    assert!(bundled_bibliography.contains("\"key\": \"porter1985\""));
    assert!(bundled_formula_graph.contains("\"formulas\""));
    assert!(bundled_formula_graph.contains("\"dependencies\""));
    assert!(bundled_transform_artifacts.contains("\"name\": \"glossary\""));
    assert!(bundled_transform_artifacts.contains("\"output_hash\""));
    assert!(bundled_transform_artifacts.contains("\"source_file\""));
    assert!(bundled_transform_artifacts.contains("\"source_line\""));
    assert!(bundled_transform_artifacts.contains("\"end_source_line\""));
    assert!(bundled_media_map.contains("\"bundle_path\": \"media/image1.svg\""));
    assert!(bundled_media_map.contains("\"hash\": \"sha256:"));
    assert_eq!(bundled_svg, "<svg/>");
}
