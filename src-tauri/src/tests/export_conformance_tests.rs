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
