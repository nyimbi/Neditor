use super::*;

#[test]
fn compiler_renders_block_and_inline_equations() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Math\nstatus: approved\napprovedBy: QA\n---\n# Math\nInline \\(ROI = x\\).\n\n$$\nROI = \\frac{Gain - Cost}{Cost}\n$$ {#eq:roi}\n\nSee {@eq:roi}.".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("class=\"equation\""));
    assert!(response.html.contains("id=\"eq:roi\""));
    assert!(response.html.contains("Equation 1"));
    assert!(response.html.contains("class=\"math math-inline\""));
    assert!(response.html.contains("class=\"math-frac\""));
    assert!(response.html.contains("role=\"math\""));
    assert!(response.html.contains("<summary>LaTeX</summary>"));
    assert!(response
        .compiled_markdown
        .contains("See [Equation roi](#eq:roi)."));
    assert!(response
        .html
        .contains(r##"<a href="#eq:roi">Equation roi</a>"##));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Equation { text, .. } if text.contains("\\frac")
        )
    }));
    assert!(response
        .semantic
        .cross_references
        .iter()
        .any(|reference| reference.key == "eq:roi" && reference.resolved));
}

#[test]
fn compiler_renders_markdown_footnotes() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Footnotes\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-18\n---\n# Footnotes\nA governed claim.[^risk]\n\n[^risk]: Reviewed by compliance.\n    Includes second-line evidence.\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("role=\"doc-endnotes\""));
    assert!(response.html.contains("id=\"fn:risk\""));
    assert!(response.html.contains("Reviewed by compliance."));
    assert!(response.html.contains("Includes second-line evidence."));
    assert!(!response.compiled_markdown.contains("[^risk]:"));
    assert!(!response
        .compiled_markdown
        .contains("    Includes second-line evidence."));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Footnotes { entries, .. }
                if entries.len() == 1
                    && entries[0].key == "risk"
                    && entries[0].text.contains("Reviewed by compliance.")
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Paragraph { inlines, .. }
                if inlines.iter().any(|node| matches!(
                    node,
                    document_ast::InlineNode::FootnoteReference { key, number, .. }
                        if key == "risk" && *number == 1
                ))
        )
    }));

    let options = json!({});
    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Footnotes"));
    assert!(pdf_text.contains("Reviewed by compliance."));
    assert!(!pdf_text.contains("<section"));

    let docx = render_docx_bytes(&response, &options).expect("docx footnotes");
    let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
    let docx_footnotes = zip_entry_text(&docx, "word/footnotes.xml");
    assert!(docx_content_types.contains("wordprocessingml.footnotes+xml"));
    assert!(docx_relationships.contains(r#"Target="footnotes.xml""#));
    assert!(docx_document.contains("Footnotes"));
    assert!(docx_document.contains("A governed claim."));
    assert!(docx_document.contains(r#"<w:footnoteReference w:id="1""#));
    assert!(!docx_document.contains("Footnote 1"));
    assert!(docx_footnotes.contains(r#"<w:footnote w:id="1""#));
    assert!(docx_footnotes.contains("Reviewed by compliance."));
    assert!(docx_footnotes.contains("Includes second-line evidence."));
    assert!(!docx_document.contains("&lt;section"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx footnotes");
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(pptx_slide.contains("Footnotes"));
    assert!(pptx_slide.contains("Reviewed by compliance."));
    assert!(!pptx_slide.contains("&lt;section"));
}

#[test]
fn cross_references_resolve_heading_appendix_and_decision_anchors() {
    let response = compile(CompileRequest {
            text: "---\ntitle: References\nstatus: approved\napprovedBy: QA\n---\n# Strategy {#sec:strategy}\nSee {@sec:strategy}, {@appendix-a}, and {@decision-record}.\n\n```md\n# Example {#code-label}\nSee {@missing-code} and {@sec:strategy}.\n```\n\n## Appendix A\nSupporting detail.\n\n## Decision Record\nUse local-first exports.\n".to_string(),
            file_path: None,
        });

    assert!(response
        .semantic
        .headings
        .iter()
        .any(|heading| heading.text == "Strategy" && heading.anchor == "sec:strategy"));
    for key in ["sec:strategy", "appendix-a", "decision-record"] {
        assert!(response
            .semantic
            .cross_references
            .iter()
            .any(|reference| reference.key == key && reference.resolved));
    }
    assert!(!response
        .semantic
        .labels
        .iter()
        .any(|label| label == "code-label"));
    assert!(!response
        .semantic
        .cross_references
        .iter()
        .any(|reference| reference.key == "missing-code"));
    assert!(response.compiled_markdown.contains(
            "See [Section strategy](#sec:strategy), [Section appendix a](#appendix-a), and [Section decision record](#decision-record)."
        ));
    assert!(response
        .compiled_markdown
        .contains("See {@missing-code} and {@sec:strategy}."));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Broken cross reference")));
}

#[test]
fn compiler_renders_layout_break_directives() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Layout\nstatus: approved\napprovedBy: QA\n---\n# Layout\n{{page-break}}\n{{section-break columns=1}}\n\n```layout\ncolumns: 2\n```\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("data-layout=\"page-break\""));
    assert!(response.html.contains("data-layout=\"section-break\""));
    assert!(response.html.contains("columns=1"));
    assert!(response.html.contains("data-layout=\"layout\""));
    assert!(response.html.contains("column-count:2"));
    assert!(!response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Missing document variable: section-break")));
}

#[test]
fn layout_pagination_controls_flow_through_exports() {
    let column_lines = (1..=46)
        .map(|index| format!("Column flow line {index}."))
        .collect::<Vec<_>>()
        .join("\n\n");
    let response = compile(CompileRequest {
            text: format!("---\ntitle: Flow Layout\nstatus: approved\napprovedBy: QA\n---\n# Flow Layout\n\n```layout\nbreakBefore: page\nkeepWithNext: true\nkeepTogether: true\n```\n## Kept Heading\nKept paragraph.\n\n{{{{section-break columns=2 pageSize=letter orientation=landscape margins=narrow breakAfter=page header=\"Flow Header\" footer=\"Flow {{{{page}}}}/{{{{pages}}}}\"}}}}\nAfter section.\n\n{column_lines}\n\nSecond column marker.\n"),
            file_path: None,
        });
    let options = json!({});

    assert!(response.html.contains("break-before:page"));
    assert!(response.html.contains("page-break-before:always"));
    assert!(response.html.contains("break-after:avoid"));
    assert!(response.html.contains("break-inside:avoid"));
    assert!(response.html.contains("page:neditor-landscape"));
    assert!(response.html.contains("--neditor-page-size:letter"));
    assert!(response.html.contains("--neditor-page-margins:narrow"));
    assert!(response.document_ast.blocks.iter().any(|block| matches!(
        block,
        DocumentBlock::Layout {
            directive,
            settings,
            ..
        } if directive == "layout"
            && settings.break_before.as_deref() == Some("page")
            && settings.keep_with_next
            && settings.keep_together
    )));
    assert!(response.document_ast.blocks.iter().any(|block| matches!(
        block,
        DocumentBlock::Layout {
            directive,
            settings,
            ..
        } if directive == "section-break"
            && settings.columns == Some(2)
            && settings.page_size.as_deref() == Some("letter")
            && settings.orientation.as_deref() == Some("landscape")
            && settings.margins.as_deref() == Some("narrow")
            && settings.break_after.as_deref() == Some("page")
    )));
    assert_eq!(response.paged_document.sections.len(), 2);
    let flow_section = response
        .paged_document
        .sections
        .iter()
        .find(|section| section.layout.columns == Some(2))
        .expect("section-level paged layout");
    assert_eq!(flow_section.layout.page_size.as_deref(), Some("letter"));
    assert_eq!(
        flow_section.layout.orientation.as_deref(),
        Some("landscape")
    );
    assert_eq!(flow_section.layout.margins.as_deref(), Some("narrow"));
    assert!(flow_section
        .blocks
        .iter()
        .any(|block| block.kind == "layout" && block.source.is_some()));
    let manifest_flow_section = response
        .export_manifest
        .layout_sections
        .iter()
        .find(|section| section.id == flow_section.id)
        .expect("manifest layout section");
    assert_eq!(manifest_flow_section.columns, Some(2));
    assert_eq!(manifest_flow_section.page_size.as_deref(), Some("letter"));
    assert_eq!(
        manifest_flow_section.orientation.as_deref(),
        Some("landscape")
    );
    assert_eq!(manifest_flow_section.margins.as_deref(), Some("narrow"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_header = zip_entry_text(&docx, "word/header2.xml");
    let docx_footer = zip_entry_text(&docx, "word/footer2.xml");
    assert!(docx_document.contains("<w:pageBreakBefore/>"));
    assert!(docx_document.contains("<w:keepNext/>"));
    assert!(docx_document.contains("<w:keepLines/>"));
    assert!(docx_document.contains(r#"<w:cols w:num="2""#));
    assert!(docx_document.contains(r#"<w:pgSz w:w="15840" w:h="12240" w:orient="landscape"/>"#));
    assert!(docx_document
        .contains(r#"<w:pgMar w:top="720" w:right="720" w:bottom="720" w:left="720"/>"#));
    assert!(docx_header.contains("Flow Header"));
    assert!(docx_footer.contains(r#"<w:fldSimple w:instr="PAGE">"#));
    assert!(docx_footer.contains(r#"<w:fldSimple w:instr="NUMPAGES">"#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
    let slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/slide");
    assert!(pptx_app.contains("<Slides>"));
    assert!(slides.iter().any(|slide| slide.contains("Flow Header")));
    assert!(slides
        .iter()
        .any(|slide| slide.contains("Section break: columns=2, pageSize=letter, orientation=landscape, margins=narrow, breakAfter=page")));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Layout: breakBefore=page, keepWithNext=true, keepTogether=true"));
    assert!(pdf_text.contains(
        "Section break: columns=2, pageSize=letter, orientation=landscape, margins=narrow, breakAfter=page"
    ));
    assert!(pdf_text.contains("Flow Header"));
    assert!(pdf_text.contains("/MediaBox [0 0 595 842]"));
    assert!(pdf_text.contains("/MediaBox [0 0 792 612]"));
    assert!(pdf_text.contains("BT /F1 10 Tf 408 "));
    assert!(pdf_text.contains("(Second column marker.) Tj"));

    let bundle =
        render_markdown_bundle_bytes(&response, &response.export_manifest).expect("layout bundle");
    let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
    assert!(bundled_ast.contains(r#""break_before": "page""#));
    assert!(bundled_ast.contains(r#""keep_with_next": true"#));
    assert!(bundled_ast.contains(r#""keep_together": true"#));
    assert!(bundled_ast.contains(r#""page_size": "letter""#));
    assert!(bundled_ast.contains(r#""orientation": "landscape""#));
    assert!(bundled_ast.contains(r#""margins": "narrow""#));
    let bundled_paged_document = zip_entry_text(&bundle, "paged-document.json");
    assert!(bundled_paged_document.contains(r#""id": "section-2""#));
    assert!(bundled_paged_document.contains(r#""columns": 2"#));
    assert!(bundled_paged_document.contains(r#""page_size": "letter""#));
}

#[test]
fn pdf_layout_keep_with_next_moves_following_block_as_group() {
    let filler = (1..=57)
        .map(|index| format!("Filler paragraph {index}."))
        .collect::<Vec<_>>()
        .join("\n\n");
    let response = compile(CompileRequest {
        text: format!(
            "---\ntitle: PDF Keep\nstatus: approved\napprovedBy: QA\n---\n# PDF Keep\n\n{filler}\n\n```layout\nkeepWithNext: true\nkeepTogether: true\n```\n## Kept Heading\nKept paragraph.\n"
        ),
        file_path: None,
    });

    let pdf = render_pdf_bytes(&response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text
        .contains("BT /F1 10 Tf 68 774 Td (Layout: keepWithNext=true, keepTogether=true) Tj"));
    assert!(pdf_text.contains("BT /F1 10 Tf 68 762 Td (## Kept Heading) Tj"));
    assert!(!pdf_text
        .contains("BT /F1 10 Tf 68 78 Td (Layout: keepWithNext=true, keepTogether=true) Tj"));
}

#[test]
fn pdf_left_floats_allow_text_to_wrap_alongside_figures() {
    let left_response = compile(CompileRequest {
            text: "---\ntitle: Float PDF\nstatus: approved\napprovedBy: QA\n---\n# Float PDF\n![Float](data:image/svg+xml;base64,PHN2Zy8+){#fig:left caption=\"Left float\" float=\"left\"}\nParagraph after the floated figure should begin beside the reserved figure area and continue wrapping in the remaining column width.\n".to_string(),
            file_path: None,
        });

    let pdf = render_pdf_bytes(&left_response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);

    assert!(pdf_text.contains("68 627 240 135 re S"));
    assert!(pdf_text.contains("BT /F1 10 Tf 320 762 Td (Paragraph after the floated figure"));
    assert!(!pdf_text.contains("BT /F1 10 Tf 68 601 Td (Paragraph after the floated figure"));

    let right_response = compile(CompileRequest {
            text: "---\ntitle: Float PDF\nstatus: approved\napprovedBy: QA\n---\n# Float PDF\n![Float](data:image/svg+xml;base64,PHN2Zy8+){#fig:right caption=\"Right float\" float=\"right\"}\nParagraph after the right floated figure should stay in the left text column while the figure occupies the right side.\n".to_string(),
            file_path: None,
        });
    let pdf = render_pdf_bytes(&right_response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("287 627 240 135 re S"));
    assert!(pdf_text.contains("BT /F1 10 Tf 68 762 Td (Paragraph after the right floated figure"));
}

#[test]
fn pdf_wraps_long_paragraphs_and_avoids_single_line_widows() {
    let filler = (1..=57)
        .map(|index| format!("Filler paragraph {index}."))
        .collect::<Vec<_>>()
        .join("\n\n");
    let long_paragraph = (1..=25)
        .map(|index| format!("alpha{index:02}"))
        .collect::<Vec<_>>()
        .join(" ");
    let first_wrapped_line = (1..=11)
        .map(|index| format!("alpha{index:02}"))
        .collect::<Vec<_>>()
        .join(" ");
    let second_wrapped_line = (12..=22)
        .map(|index| format!("alpha{index:02}"))
        .collect::<Vec<_>>()
        .join(" ");
    let response = compile(CompileRequest {
        text: format!(
            "---\ntitle: PDF Widows\nstatus: approved\napprovedBy: QA\n---\n# PDF Widows\n\n{filler}\n\n{long_paragraph}\n"
        ),
        file_path: None,
    });

    let pdf = render_pdf_bytes(&response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains(&format!("BT /F1 10 Tf 68 774 Td ({first_wrapped_line}) Tj")));
    assert!(pdf_text.contains(&format!(
        "BT /F1 10 Tf 68 762 Td ({second_wrapped_line}) Tj"
    )));
    assert!(!pdf_text.contains(&format!("BT /F1 10 Tf 68 78 Td ({first_wrapped_line}) Tj")));
}

#[test]
fn pdf_section_columns_split_large_tables_across_columns() {
    let table_rows = (1..=34)
        .map(|index| format!("| R{index} | {index} |"))
        .collect::<Vec<_>>()
        .join("\n");
    let response = compile(CompileRequest {
        text: format!(
            "---\ntitle: Column Table\nstatus: archived\nversion: 1.0.0\n---\n# Column Table\n\n{{{{section-break columns=2 pageSize=letter orientation=landscape margins=narrow}}}}\n\nTable: Column flow\n\n| Item | Value |\n| --- | ---: |\n{table_rows}\n"
        ),
        file_path: None,
    });

    assert!(response.diagnostics.is_empty());
    assert!(response
        .paged_document
        .sections
        .iter()
        .any(|section| section.layout.columns == Some(2)));

    let pdf = render_pdf_bytes(&response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("Table \\(continued\\)"));
    assert!(pdf_text.contains("BT /F1 8 Tf 412 "));
    assert!(pdf_text.contains("(R30) Tj"));
}

#[test]
fn pdf_section_columns_continue_oversized_tables_across_pages() {
    let table_rows = (1..=72)
        .map(|index| format!("| R{index} | {index} |"))
        .collect::<Vec<_>>()
        .join("\n");
    let response = compile(CompileRequest {
        text: format!(
            "---\ntitle: Column Page Table\nstatus: archived\nversion: 1.0.0\n---\n# Column Page Table\n\n{{{{section-break columns=2 pageSize=letter orientation=landscape margins=narrow}}}}\n\nTable: Column page flow\n| Item | Value |\n| --- | ---: |\n{table_rows}\n"
        ),
        file_path: None,
    });

    let pdf = render_pdf_bytes(&response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("/Count 4"));
    assert!(pdf_text.contains("Table: Column page flow \\(continued\\)"));
    assert!(pdf_text.contains("BT /F1 8 Tf 412 "));
    assert!(pdf_text.contains("(R30) Tj"));
    assert!(pdf_text.contains("BT /F1 8 Tf 38 "));
    assert!(pdf_text.contains("(R60) Tj"));
}

#[test]
fn compiler_renders_callouts_as_semantic_blocks() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Callouts\nstatus: approved\napprovedBy: QA\n---\n# Callouts\n> [!NOTE] Board review\n> Confirm the launch criteria.\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("class=\"callout callout-note\""));
    assert!(response.html.contains("Board review"));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Callout { callout_type, title, text, .. }
                if callout_type == "note"
                    && title == "Board review"
                    && text.contains("Confirm the launch criteria")
        )
    }));

    let options = json!({});
    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    assert!(zip_entry_text(&docx, "word/document.xml")
        .contains("Callout: note: Board review: Confirm the launch criteria."));
    let pdf = render_pdf_bytes(&response, &options);
    assert!(String::from_utf8_lossy(&pdf).contains("Callout: note: Board review"));
}

#[test]
fn compiler_builds_document_ast_blocks_for_exports() {
    let response = compile(CompileRequest {
            text: "---\ntitle: AST\nstatus: approved\napprovedBy: QA\n---\n# AST\nBusiness paragraph with **margin** and [source](https://example.com) [@doe2024] {@missing-ref}.\n\n> Quoted evidence\n> with continuation\n\n```js\nconst total = 42;\nconst literal = \"![Not a figure](asset.png)\";\nconst math = \"$$not an equation$$\";\n| Literal | Value |\n| --- | ---: |\n| Not a table | =SUM(1,2) |\n![Code figure](asset.png){#fig:code caption=\"Code figure\"}\n$$\ncode equation\n$$ {#eq:code}\n> [!note] Code callout\n> Keep literal\n{{page-break}}\n```\n\n- First decision\n- Second decision\n\n- [x] Reviewed by finance\n- [ ] Attach signed approval\n\n| Metric | Value |\n| --- | ---: |\n| Total | =SUM(1,2) |\n\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:diagram caption=\"System diagram\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n{{page-break}}\n".to_string(),
            file_path: None,
        });

    assert_eq!(response.document_ast.metadata.title, "AST");
    assert_eq!(response.document_ast.metadata.status, "approved");
    assert!(response
        .document_ast
        .metadata
        .source_hash
        .starts_with("sha256:"));
    assert!(response
            .document_ast
            .blocks
            .iter()
            .any(|block| matches!(block, DocumentBlock::Heading { text, anchor, .. } if text == "AST" && anchor == "ast")));
    assert!(response
            .document_ast
            .blocks
            .iter()
            .any(|block| matches!(block, DocumentBlock::Paragraph { text, inlines, line, end_line, .. }
                if text.contains("Business paragraph with margin")
                    && line == end_line
                    && inlines.iter().any(|node| matches!(node, document_ast::InlineNode::Strong { text } if text == "margin"))
                    && inlines.iter().any(|node| matches!(node, document_ast::InlineNode::Link { text, url } if text == "source" && url == "https://example.com"))
                    && inlines.iter().any(|node| matches!(node, document_ast::InlineNode::Citation { key, .. } if key == "doe2024"))
                    && inlines.iter().any(|node| matches!(node, document_ast::InlineNode::CrossReference { key, .. } if key == "missing-ref"))
            )));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::BlockQuote { text, .. }
                if text == "Quoted evidence\nwith continuation"
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::CodeBlock { language, code, .. }
                if language.as_deref() == Some("js") && code.contains("const total = 42;")
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::List { ordered, items, .. }
                if !ordered
                    && items == &vec![
                        "First decision".to_string(),
                        "Second decision".to_string()
                    ]
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::TaskList { items, .. }
                if items.len() == 2
                    && items[0].checked
                    && items[0].text == "Reviewed by finance"
                    && !items[1].checked
                    && items[1].text == "Attach signed approval"
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Table { line, end_line, headers, alignments, rows, .. }
                if headers == &vec!["Metric".to_string(), "Value".to_string()]
                    && alignments == &vec!["left".to_string(), "right".to_string()]
                    && *end_line == *line + 2
                    && rows.iter().any(|row| row == &vec!["Total".to_string(), "3".to_string()])
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Figure { id, caption, .. }
                if id.as_deref() == Some("fig:diagram")
                    && caption.as_deref() == Some("System diagram")
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Equation { id, text, .. }
                if id.as_deref() == Some("eq:roi") && text.contains("ROI")
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Layout { directive, .. } if directive == "page-break"
        )
    }));
    assert_eq!(response.semantic.tables, 1);
    assert_eq!(response.semantic.figures, 1);
    assert_eq!(response.semantic.equations, 1);
    assert!(response
        .compiled_markdown
        .contains("| Not a table | =SUM(1,2) |"));
    assert!(response
        .compiled_markdown
        .contains("![Code figure](asset.png){#fig:code caption=\"Code figure\"}"));
    assert!(response.compiled_markdown.contains("$$ {#eq:code}"));
    assert!(response
        .compiled_markdown
        .contains("> [!note] Code callout"));
    assert!(response.compiled_markdown.contains("{{page-break}}"));

    let exported = export::export_text(&response, &json!({}));
    assert!(exported.contains("> Quoted evidence\n> with continuation"));
    assert!(exported.contains("```js\nconst total = 42;"));
    assert!(exported.contains("![Not a figure](asset.png)"));
    assert!(exported.contains("$$not an equation$$"));
    assert!(exported.contains("- First decision\n- Second decision"));
    assert!(exported.contains("- [x] Reviewed by finance\n- [ ] Attach signed approval"));
    assert!(exported.contains("Table: Metric | Value"));
    assert!(exported.contains("Figure: fig:diagram: System diagram"));
    assert!(exported.contains("Equation: eq:roi: ROI = Gain / Cost"));
}

#[test]
fn compiler_renders_openapi_and_json_schema_tables() {
    let response = compile(CompileRequest {
        text: r#"---
title: API
status: approved
approvedBy: QA
---
# API

```openapi
openapi: 3.1.0
paths:
  /accounts:
    get:
      summary: List accounts
```

```json-schema
{
  "type": "object",
  "required": ["id"],
  "properties": {
    "id": { "type": "string", "description": "Account id" },
    "balance": { "type": "number" }
  }
}
```
"#
        .to_string(),
        file_path: None,
    });

    assert!(response.html.contains("List accounts"));
    assert!(response.html.contains("Account id"));
    assert!(response.html.contains("<td>yes</td>"));
}

#[test]
fn include_expansion_strips_child_front_matter() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-test-{unique}"));
    let chapter_dir = root.join("chapters");
    fs::create_dir_all(&chapter_dir).expect("create test dirs");
    fs::write(
        chapter_dir.join("intro.md"),
        "---\ntitle: Child\n---\n\n## Included\n\nBody",
    )
    .expect("write include");

    let response = compile(CompileRequest {
        text: "---\ntitle: Root\n---\n\n!include chapters/intro.md\n".to_string(),
        file_path: Some(path_to_string(&root.join("root.md"))),
    });

    assert!(response.compiled_markdown.contains("## Included"));
    assert!(!response.compiled_markdown.contains("title: Child"));
    assert_eq!(response.include_graph.len(), 1);
    let included_line = response
        .compiled_markdown
        .lines()
        .position(|line| line == "## Included")
        .map(|index| index + 1)
        .expect("included heading line");
    assert!(response.source_map.iter().any(|entry| {
        entry.generated_line == included_line
            && entry.source_file.ends_with("chapters/intro.md")
            && entry.source_line == 2
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Heading { text, source: Some(source), .. }
                if text == "Included"
                    && source.source_file.ends_with("chapters/intro.md")
                    && source.source_line == 2
                    && source.end_source_line == 2
        )
    }));
    fs::remove_dir_all(root).expect("clean test dirs");
}

#[test]
fn include_expansion_supports_documented_directive_forms() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-include-forms-test-{unique}"));
    fs::create_dir_all(root.join("chapters")).expect("create include forms dirs");
    fs::create_dir_all(root.join("appendices")).expect("create appendices dir");
    fs::write(root.join("chapters").join("intro.md"), "## Bang Include\n")
        .expect("write bang include");
    fs::write(
        root.join("chapters").join("market.md"),
        "## Brace Include\n",
    )
    .expect("write brace include");
    fs::write(
        root.join("appendices").join("financials.md"),
        "## Comment Include\n",
    )
    .expect("write comment include");

    let response = compile(CompileRequest {
            text: "!include chapters/intro.md\n{{include chapters/market.md}}\n<!-- include: appendices/financials.md -->\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

    assert!(response.compiled_markdown.contains("## Bang Include"));
    assert!(response.compiled_markdown.contains("## Brace Include"));
    assert!(response.compiled_markdown.contains("## Comment Include"));
    assert_eq!(response.include_graph.len(), 3);
    assert!(response
        .export_manifest
        .included_files
        .iter()
        .any(|file| file.path.ends_with("chapters/market.md")));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Missing include")));
    fs::remove_dir_all(root).expect("clean include forms test dir");
}
