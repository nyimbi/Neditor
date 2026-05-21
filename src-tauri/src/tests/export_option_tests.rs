use super::*;

#[test]
fn compile_options_supply_brand_profile_defaults() {
    let response = compile_with_options(
        CompileRequest {
            text: "# Branded\n".to_string(),
            file_path: None,
        },
        &json!({
            "defaultBrandProfile": {
                "name": "Acme Strategy",
                "color": "#0F766E",
                "logo": "brand/acme.svg",
                "font": "Aptos",
                "header": "{{title}}",
                "footer": "Confidential | Page {{page}}",
                "legalDisclaimer": "Internal use only."
            }
        }),
    );

    assert_eq!(
        response
            .metadata
            .pointer("/brand/name")
            .and_then(Value::as_str),
        Some("Acme Strategy")
    );
    assert_eq!(
        response
            .metadata
            .pointer("/brand/color")
            .and_then(Value::as_str),
        Some("#0F766E")
    );
    assert_eq!(
        response
            .metadata
            .pointer("/brand/logo")
            .and_then(Value::as_str),
        Some("brand/acme.svg")
    );
    assert_eq!(
        response
            .metadata
            .pointer("/brand/font")
            .and_then(Value::as_str),
        Some("Aptos")
    );
    assert_eq!(
        response
            .metadata
            .pointer("/layout/header")
            .and_then(Value::as_str),
        Some("{{title}}")
    );
    assert_eq!(
        response
            .metadata
            .pointer("/layout/footer")
            .and_then(Value::as_str),
        Some("Confidential | Page {{page}}")
    );
    assert_eq!(
        response
            .metadata
            .get("legalDisclaimer")
            .and_then(Value::as_str),
        Some("Internal use only.")
    );
    let options = json!({ "watermark": "BOARD" });
    let html = render_full_html(&response, &options);
    assert!(html.contains("font-family:Aptos"));
    assert!(html.contains("Legal Disclaimer"));
    assert!(html.contains("Internal use only."));
    let exported_text = export::export_text(&response, &options);
    assert!(exported_text.contains("Header: Branded"));
    assert!(exported_text.contains("Footer: Confidential | Page 1"));
    assert!(exported_text.contains("Watermark: BOARD"));
    assert!(exported_text.contains("Legal Disclaimer"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let legal_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
        .into_iter()
        .find(|slide| slide.contains("Legal Disclaimer"))
        .expect("legal disclaimer slide");
    assert!(legal_slide.contains("<a:t>Legal Disclaimer</a:t>"));
    assert!(legal_slide.contains("Internal use only."));
}

#[test]
fn compile_options_do_not_override_document_brand_profile() {
    let response = compile_with_options(
        CompileRequest {
            text: "---\nbrand:\n  name: Document Brand\n  color: \"#111111\"\n---\n# Branded\n"
                .to_string(),
            file_path: None,
        },
        &json!({
            "defaultBrandProfile": {
                "name": "Acme Strategy",
                "color": "#0F766E",
                "logo": "brand/acme.svg"
            }
        }),
    );

    assert_eq!(
        response
            .metadata
            .pointer("/brand/name")
            .and_then(Value::as_str),
        Some("Document Brand")
    );
    assert_eq!(
        response
            .metadata
            .pointer("/brand/color")
            .and_then(Value::as_str),
        Some("#111111")
    );
    assert_eq!(
        response
            .metadata
            .pointer("/brand/logo")
            .and_then(Value::as_str),
        Some("brand/acme.svg")
    );
}

#[test]
fn export_options_control_cover_styles_and_page_numbers() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Export Options\nstatus: approved\napprovedBy: QA\n---\n# Export Options\n\nBody."
                .to_string(),
            file_path: None,
        });
    let options = json!({
        "includeStyles": false,
        "coverPage": false,
        "pageNumbers": false
    });

    let html = render_full_html(&response, &options);
    assert!(!html.contains("<style>"));
    assert!(!html.contains("class=\"cover\""));
    assert!(!html.contains("Page 1 of 1"));
    assert!(html.contains("<main>"));

    let exported_text = export::export_text(&response, &options);
    assert!(!exported_text.contains("Cover: Export Options"));
    assert!(!exported_text.contains("Page 1 of 1"));
    assert!(exported_text.contains("Status: approved"));
}

#[test]
fn export_layout_preset_controls_html_css_and_metadata() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Layout Options\nstatus: approved\napprovedBy: QA\n---\n# Layout Options\n\nBody."
                .to_string(),
            file_path: None,
        });
    let options = json!({ "layoutPreset": "compact" });

    let html = render_full_html(&response, &options);
    assert!(html.contains("margin:32px"));
    assert!(html.contains("line-height:1.42"));
    assert!(html.contains("p,li,blockquote{orphans:2;widows:2}"));
    assert!(html.contains("@page{size:A4;margin:18mm"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let document = zip_entry_text(&docx, "word/document.xml");
    assert!(document.contains("<w:widowControl/>"));

    let exported_text = export::export_text(&response, &options);
    assert!(exported_text.contains("Layout preset: compact"));
}

#[test]
fn export_layout_metadata_controls_page_size_and_margins() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Page Layout\nstatus: approved\napprovedBy: QA\nlayout:\n  pageSize: Letter\n  margins: wide\n  orientation: landscape\n---\n# Page Layout\n\nBody.".to_string(),
            file_path: None,
        });
    let options = json!({});

    let html = render_full_html(&response, &options);
    assert!(html.contains("@page{size:Letter landscape;margin:32mm"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let document = zip_entry_text(&docx, "word/document.xml");
    assert!(document.contains(r#"<w:pgSz w:w="15840" w:h="12240" w:orient="landscape"/>"#));
    assert!(document
        .contains(r#"<w:pgMar w:top="1800" w:right="1800" w:bottom="1800" w:left="1800"/>"#));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("/MediaBox [0 0 792 612]"));
    assert!(pdf_text.contains("BT /F1 10 Tf 91 521 Td"));
}

#[test]
fn compiler_validates_layout_page_metadata() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Bad Layout\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-18\nlayout:\n  pageSize: Tabloid\n  margins: huge\n  orientation: diagonal\n---\n# Bad Layout\n".to_string(),
            file_path: None,
        });

    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Unsupported layout pageSize: Tabloid")));
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Unsupported layout margins: huge")));
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Unsupported layout orientation: diagonal")));

    let directive_response = compile(CompileRequest {
            text: "---\ntitle: Bad Directive Layout\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-18\n---\n# Bad Directive Layout\n\n{{section-break pageSize=Tabloid orientation=sideways margins=huge}}\n".to_string(),
            file_path: None,
        });
    assert!(directive_response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic
            .message
            .contains("Unsupported layout directive pageSize: Tabloid")));
    assert!(directive_response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic
            .message
            .contains("Unsupported layout directive orientation: sideways")));
    assert!(directive_response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic
            .message
            .contains("Unsupported layout directive margins: huge")));
}

#[test]
fn export_syntax_highlighting_can_be_included_or_omitted() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Syntax Options\nstatus: approved\napprovedBy: QA\n---\n# Syntax Options\n\n```js\nconst total = 42; // amount\n```\n"
                .to_string(),
            file_path: None,
        });

    let highlighted = render_full_html(&response, &json!({}));
    assert!(highlighted.contains("class=\"syn-keyword\""));
    assert!(highlighted.contains("class=\"syn-number\""));
    assert!(highlighted.contains("class=\"syn-comment\""));
    assert!(highlighted.contains(".syn-keyword"));

    let plain = render_full_html(&response, &json!({ "includeSyntaxHighlighting": false }));
    assert!(!plain.contains("class=\"syn-keyword\""));
    assert!(!plain.contains(".syn-keyword"));
    assert!(plain.contains("const total = 42; // amount"));

    let exported_text =
        export::export_text(&response, &json!({ "includeSyntaxHighlighting": false }));
    assert!(exported_text.contains("Syntax highlighting: omitted"));
}

#[test]
fn export_appendix_options_control_target_outputs() {
    let response = compile(CompileRequest {
        text: include_str!("../../fixtures/export/business_report.md").to_string(),
        file_path: None,
    });
    let included = json!({
        "includeGlossary": true,
        "includeComments": true,
        "includeProvenance": true
    });
    let omitted = json!({
        "includeGlossary": false,
        "includeComments": false,
        "includeProvenance": false
    });

    let html = render_full_html(&response, &included);
    assert!(html.contains("class=\"export-glossary\""));
    assert!(html.contains("class=\"export-comments\""));
    assert!(html.contains("class=\"export-provenance\""));
    let html_without_appendix = render_full_html(&response, &omitted);
    assert!(!html_without_appendix.contains("class=\"export-glossary\""));
    assert!(!html_without_appendix.contains("class=\"export-comments\""));
    assert!(!html_without_appendix.contains("class=\"export-provenance\""));

    let pdf = String::from_utf8_lossy(&render_pdf_bytes(&response, &included)).into_owned();
    assert!(pdf.contains("Glossary"));
    assert!(pdf.contains("Review Comments"));
    assert!(pdf.contains("AI Provenance"));
    let pdf_without_appendix =
        String::from_utf8_lossy(&render_pdf_bytes(&response, &omitted)).into_owned();
    assert!(!pdf_without_appendix.contains("Review Comments"));
    assert!(!pdf_without_appendix.contains("AI Provenance"));

    let docx = render_docx_bytes(&response, &included).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Glossary"));
    assert!(docx_document.contains("Review Comments"));
    assert!(docx_document.contains("AI Provenance"));
    let docx_without_appendix = render_docx_bytes(&response, &omitted).expect("docx bytes");
    let docx_without_document = zip_entry_text(&docx_without_appendix, "word/document.xml");
    assert!(!docx_without_document.contains("Review Comments"));
    assert!(!docx_without_document.contains("AI Provenance"));

    let pptx = render_pptx_bytes(&response, &included).expect("pptx bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Glossary"));
    assert!(pptx_slides.contains("Review Comments"));
    assert!(pptx_slides.contains("AI Provenance"));
    let pptx_without_appendix = render_pptx_bytes(&response, &omitted).expect("pptx bytes");
    let pptx_without_slides =
        zip_entry_texts_with_prefix(&pptx_without_appendix, "ppt/slides/").join("\n");
    assert!(!pptx_without_slides.contains("Review Comments"));
    assert!(!pptx_without_slides.contains("AI Provenance"));

    let text = export::export_text(&response, &included);
    assert!(text.contains("Glossary"));
    assert!(text.contains("Review Comments"));
    assert!(text.contains("AI Provenance"));
    let text_without_appendix = export::export_text(&response, &omitted);
    assert!(!text_without_appendix.contains("Review Comments"));
    assert!(!text_without_appendix.contains("AI Provenance"));
}

#[test]
fn export_option_matrix_is_preserved_across_targets_and_bundle_evidence() {
    let response = compile(CompileRequest {
        text: r##"---
title: Option Matrix Report
status: approved
approvedBy: QA
approvedAt: 2026-05-20T09:00:00Z
version: 1.0.0
toc: true
legalDisclaimer: "Internal proof only."
brand:
  name: Matrix Brand
  color: "#123456"
---

# Option Matrix Report

[TOC]

This report proves target-specific export options from one semantic document.

```js
const total = 42; // evidence
```

<!-- comment: author: QA | at: 2026-05-20 | resolved | Check option parity. -->
<!-- change: author: QA | at: 2026-05-20T10:00:00Z | Confirmed target matrix. -->

```glossary
SLA: Service-level agreement.
```

```ai-source
provider: OpenAI
model: gpt-5.4
date: 2026-05-20
promptSummary: option matrix synthesis
reviewedBy: QA
reviewedAt: 2026-05-20T09:30:00Z
status: human-reviewed
```
"##
        .to_string(),
        file_path: None,
    });
    let options = json!({
        "includeStyles": false,
        "includeSyntaxHighlighting": false,
        "coverPage": false,
        "pageNumbers": false,
        "includeGlossary": false,
        "includeComments": false,
        "includeProvenance": false,
        "includeAgenda": false,
        "layoutPreset": "compact",
        "watermark": "INTERNAL"
    });

    let html = render_full_html(&response, &options);
    assert!(!html.contains("<style>"));
    assert!(!html.contains("class=\"cover\""));
    assert!(!html.contains("syn-keyword"));
    assert!(!html.contains("class=\"export-glossary\""));
    assert!(!html.contains("class=\"export-comments\""));
    assert!(!html.contains("class=\"export-provenance\""));
    assert!(html.contains("class=\"export-legal\""));
    assert!(html.contains("Internal proof only."));

    let pdf = String::from_utf8_lossy(&render_pdf_bytes(&response, &options)).into_owned();
    assert!(!pdf.contains("Page 1 of"));
    assert!(!pdf.contains("Glossary"));
    assert!(!pdf.contains("Review Comments"));
    assert!(!pdf.contains("AI Provenance"));
    assert!(pdf.contains("Legal Disclaimer"));
    assert!(pdf.contains("Internal proof only."));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_footer = zip_entry_text(&docx, "word/footer1.xml");
    assert!(!docx_footer.contains("PAGE"));
    assert!(!docx_document.contains("Glossary"));
    assert!(!docx_document.contains("Review Comments"));
    assert!(!docx_document.contains("AI Provenance"));
    assert!(docx_document.contains("Legal Disclaimer"));
    assert!(docx_document.contains("Internal proof only."));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(!pptx_slides.contains("Agenda"));
    assert!(!pptx_slides.contains("Glossary"));
    assert!(!pptx_slides.contains("Review Comments"));
    assert!(!pptx_slides.contains("AI Provenance"));
    assert!(pptx_slides.contains("Legal Disclaimer"));
    assert!(pptx_slides.contains("Internal proof only."));

    let exported_text = export::export_text(&response, &options);
    assert!(!exported_text.contains("Cover: Option Matrix Report"));
    assert!(!exported_text.contains("Page 1 of"));
    assert!(exported_text.contains("Layout preset: compact"));
    assert!(exported_text.contains("Syntax highlighting: omitted"));
    assert!(!exported_text.contains("Glossary"));
    assert!(!exported_text.contains("Review Comments"));
    assert!(!exported_text.contains("AI Provenance"));
    assert!(exported_text.contains("Legal Disclaimer"));
    assert!(exported_text.contains("Internal proof only."));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_target = "markdown-bundle".to_string();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_manifest = zip_entry_text(&bundle, "manifest.json");
    assert!(bundled_text.contains("Layout preset: compact"));
    assert!(bundled_text.contains("Syntax highlighting: omitted"));
    assert!(!bundled_text.contains("Glossary"));
    assert!(!bundled_text.contains("Review Comments"));
    assert!(!bundled_text.contains("AI Provenance"));
    let manifest_json: Value =
        serde_json::from_str(&bundled_manifest).expect("bundle manifest json");
    assert_eq!(
        manifest_json
            .pointer("/export_target")
            .and_then(Value::as_str),
        Some("markdown-bundle")
    );
    assert_eq!(
        manifest_json
            .pointer("/export_options/includeStyles")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        manifest_json
            .pointer("/export_options/includeSyntaxHighlighting")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        manifest_json
            .pointer("/export_options/includeAgenda")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        manifest_json
            .pointer("/export_options/layoutPreset")
            .and_then(Value::as_str),
        Some("compact")
    );
}

#[test]
fn enabled_export_option_matrix_survives_cross_target_artifacts() {
    let response = compile(CompileRequest {
        text: r##"---
title: Enabled Option Matrix
status: approved
approvedBy: QA
approvedAt: 2026-05-21T10:00:00Z
version: 2.0.0
subtitle: Board-ready option proof
legalDisclaimer: "For enabled option proof only."
brand:
  name: Matrix Brand
  color: "#0F766E"
layout:
  header: "{{title}} | {{status}}"
  footer: "{{classification}} | Page {{page}} of {{pages}}"
classification: Internal
---

# Enabled Option Matrix

## Evidence

Enabled options should be visible in every target artifact family.

```js
const reviewed = true; // enabled proof
```

<!-- comment: author: QA | at: 2026-05-21 | resolved | Enabled comments appendix. -->
<!-- change: author: QA | at: 2026-05-21T10:10:00Z | Confirmed enabled matrix. -->

```glossary
SLA: Service-level agreement.
```

```ai-source
provider: OpenAI
model: gpt-5.4
date: 2026-05-21
promptSummary: enabled option matrix synthesis
reviewedBy: QA
reviewedAt: 2026-05-21T10:15:00Z
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
        "includeGlossary": true,
        "includeComments": true,
        "includeProvenance": true,
        "includeAgenda": true,
        "layoutPreset": "presentation",
        "watermark": "APPROVED"
    });

    let html = render_full_html(&response, &options);
    assert!(html.contains("<style>"));
    assert!(html.contains("class=\"cover\""));
    assert!(html.contains("Enabled Option Matrix | approved"));
    assert!(html.contains("Internal | Page 1 of 1"));
    assert!(html.contains("class=\"syn-keyword\""));
    assert!(html.contains("APPROVED"));
    assert!(html.contains("class=\"export-glossary\""));
    assert!(html.contains("class=\"export-comments\""));
    assert!(html.contains("class=\"export-provenance\""));
    assert!(html.contains("For enabled option proof only."));

    let pdf = String::from_utf8_lossy(&render_pdf_bytes(&response, &options)).into_owned();
    assert!(pdf.contains("Cover: Enabled Option Matrix"));
    assert!(pdf.contains("Page 1 of 1"));
    assert!(pdf.contains("Layout preset: presentation"));
    assert!(pdf.contains("Syntax highlighting: included"));
    assert!(pdf.contains("Watermark: APPROVED"));
    assert!(pdf.contains("Glossary"));
    assert!(pdf.contains("Review Comments"));
    assert!(pdf.contains("AI Provenance"));
    assert!(pdf.contains("For enabled option proof only."));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_footer = zip_entry_text(&docx, "word/footer1.xml");
    assert!(docx_document.contains("Cover: Enabled Option Matrix"));
    assert!(docx_document.contains("Page 1 of 1"));
    assert!(docx_document.contains("Layout preset: presentation"));
    assert!(docx_document.contains("Syntax highlighting: included"));
    assert!(docx_document.contains("Watermark: APPROVED"));
    assert!(docx_document.contains("Glossary"));
    assert!(docx_document.contains("Review Comments"));
    assert!(docx_document.contains("AI Provenance"));
    assert!(docx_document.contains("For enabled option proof only."));
    assert!(docx_footer.contains("PAGE"));
    assert!(docx_footer.contains("NUMPAGES"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
    assert!(pptx_slides.contains("Agenda"));
    assert!(pptx_slides.contains("Evidence"));
    assert!(pptx_slides.contains("Layout preset: presentation"));
    assert!(pptx_slides.contains("Syntax highlighting: included"));
    assert!(pptx_slides.contains("Watermark: APPROVED"));
    assert!(pptx_slides.contains("Glossary"));
    assert!(pptx_slides.contains("Review Comments"));
    assert!(pptx_slides.contains("AI Provenance"));
    assert!(pptx_slides.contains("For enabled option proof only."));

    let exported_text = export::export_text(&response, &options);
    assert!(exported_text.contains("Cover: Enabled Option Matrix"));
    assert!(exported_text.contains("Footer: Internal | Page 1 of 1"));
    assert!(exported_text.contains("Page 1 of 1"));
    assert!(exported_text.contains("Layout preset: presentation"));
    assert!(exported_text.contains("Syntax highlighting: included"));
    assert!(exported_text.contains("Watermark: APPROVED"));
    assert!(exported_text.contains("Glossary"));
    assert!(exported_text.contains("Review Comments"));
    assert!(exported_text.contains("AI Provenance"));
    assert!(exported_text.contains("For enabled option proof only."));

    let mut bundle_manifest = response.export_manifest.clone();
    bundle_manifest.export_target = "markdown-bundle".to_string();
    bundle_manifest.export_options = options.clone();
    let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
    let bundled_text = zip_entry_text(&bundle, "document.txt");
    let bundled_manifest = zip_entry_text(&bundle, "manifest.json");
    assert!(bundled_text.contains("Layout preset: presentation"));
    assert!(bundled_text.contains("Syntax highlighting: included"));
    assert!(bundled_text.contains("Watermark: APPROVED"));
    assert!(bundled_text.contains("Review Comments"));
    assert!(bundled_text.contains("AI Provenance"));
    let manifest_json: Value =
        serde_json::from_str(&bundled_manifest).expect("bundle manifest json");
    assert_eq!(
        manifest_json
            .pointer("/export_options/includeStyles")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        manifest_json
            .pointer("/export_options/includeSyntaxHighlighting")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        manifest_json
            .pointer("/export_options/includeAgenda")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        manifest_json
            .pointer("/export_options/layoutPreset")
            .and_then(Value::as_str),
        Some("presentation")
    );
    assert_eq!(
        manifest_json
            .pointer("/export_options/watermark")
            .and_then(Value::as_str),
        Some("APPROVED")
    );
}
