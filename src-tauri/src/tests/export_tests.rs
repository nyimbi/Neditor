use super::*;

#[test]
fn export_renderers_return_non_empty_artifacts() {
    let response = compile(CompileRequest {
        text: sample_document(),
        file_path: None,
    });

    let html = render_full_html(&response, &json!({ "watermark": "DRAFT" }));
    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("<html lang=\"en\">"));
    assert!(
        html.contains(r#"<meta name="viewport" content="width=device-width, initial-scale=1">"#)
    );
    assert!(html.contains(r#"<meta name="generator" content="NEditor">"#));
    assert!(html.contains(r#"<meta name="neditor-status" content="approved">"#));
    assert!(html.contains(r#"<meta name="neditor-version" content="1.2.0">"#));
    assert!(html.contains(r#"<meta name="author" content="QA">"#));
    assert!(html.contains(r#"<meta name="neditor-source-hash" content="sha256:"#));
    assert!(html.contains("class=\"cover\""));
    assert!(html.contains("class=\"cover-logo\""));
    assert!(html.contains("Page {{page}} of {{pages}}") || html.contains("Page 1 of 1"));
    assert!(html.contains("DRAFT"));
    let options = json!({ "watermark": "DRAFT" });
    let pdf = render_pdf_bytes(&response, &options);
    assert!(pdf.starts_with(b"%PDF-1.4"));
    assert!(String::from_utf8_lossy(&pdf).contains("Page 1 of 1"));
    assert!(String::from_utf8_lossy(&pdf).contains("/Title (Test Report)"));
    assert!(String::from_utf8_lossy(&pdf).contains("/Info "));
    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    assert!(docx.len() > 100);
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Cover: Test Report"));
    assert!(docx_document.contains("Logo: data:image/svg+xml"));
    assert!(docx_document.contains("Watermark: DRAFT"));
    let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
    let docx_core = zip_entry_text(&docx, "docProps/core.xml");
    let docx_app = zip_entry_text(&docx, "docProps/app.xml");
    let docx_custom = zip_entry_text(&docx, "docProps/custom.xml");
    assert!(docx_content_types.contains("custom-properties"));
    assert!(docx_core.contains("<dc:title>Test Report</dc:title>"));
    assert!(docx_core.contains("<cp:category>approved</cp:category>"));
    assert!(docx_app.contains("<Application>NEditor</Application>"));
    assert!(docx_app.contains("<Words>"));
    assert!(docx_app.contains("<AppVersion>"));
    assert!(docx_custom.contains(r#"name="NEditorStatus""#));
    assert!(docx_custom.contains("<vt:lpwstr>approved</vt:lpwstr>"));
    assert!(docx_custom.contains(r#"name="NEditorVersion""#));
    assert!(docx_custom.contains("<vt:lpwstr>1.2.0</vt:lpwstr>"));
    assert!(docx_custom.contains(r#"name="NEditorSourceHash""#));
    let docx_relationships = zip_entry_text(&docx, "_rels/.rels");
    assert!(docx_relationships.contains("metadata/core-properties"));
    assert!(docx_relationships.contains("extended-properties"));
    assert!(docx_relationships.contains("custom-properties"));
    let docx_document_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
    assert!(docx_document_relationships.contains("relationships/header"));
    assert!(docx_document_relationships.contains("relationships/footer"));
    assert!(zip_entry_text(&docx, "word/header1.xml").contains("Test Report"));
    let docx_footer = zip_entry_text(&docx, "word/footer1.xml");
    assert!(docx_footer.contains(r#"w:instr="PAGE""#));
    assert!(docx_footer.contains(r#"w:instr="NUMPAGES""#));
    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    assert!(pptx.len() > 100);
    let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide1.xml");
    assert!(pptx_slide.contains("Test Report"));
    assert!(pptx_slide.contains(r#"name="Header""#));
    assert!(pptx_slide.contains("Page 1 of 1"));
    let pptx_content_types = zip_entry_text(&pptx, "[Content_Types].xml");
    let pptx_core = zip_entry_text(&pptx, "docProps/core.xml");
    let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
    let pptx_custom = zip_entry_text(&pptx, "docProps/custom.xml");
    assert!(pptx_content_types.contains("custom-properties"));
    assert!(pptx_core.contains("<dc:title>Test Report</dc:title>"));
    assert!(pptx_core.contains("<cp:category>approved</cp:category>"));
    assert!(pptx_app.contains("<Application>NEditor</Application>"));
    assert!(pptx_app.contains("<Slides>"));
    assert!(pptx_app.contains("<Notes>0</Notes>"));
    assert!(pptx_custom.contains(r#"name="NEditorClient""#));
    assert!(pptx_custom.contains("<vt:lpwstr>Acme</vt:lpwstr>"));
    assert!(pptx_custom.contains(r#"name="NEditorSourceHash""#));
    assert!(
        render_markdown_bundle_bytes(&response, &response.export_manifest)
            .expect("bundle bytes")
            .starts_with(b"PK")
    );
}

#[test]
fn html_export_writes_standalone_web_metadata() {
    let response = compile(CompileRequest {
        text: r#"---
title: Public Brief
subtitle: Market entry update
author: Strategy Office
language: en-GB
canonicalUrl: https://example.com/public-brief
status: published
approvedBy: QA
---

# Public Brief

Ship this as a standalone HTML review copy.
"#
        .to_string(),
        file_path: None,
    });

    let metadata_language_html = render_full_html(&response, &json!({}));
    assert!(metadata_language_html.contains(r#"<html lang="en-GB">"#));

    let html = render_full_html(
        &response,
        &json!({
            "htmlLanguage": "fr-CA",
            "htmlDescription": "Option-level web description",
            "canonicalUrl": "https://example.com/option-brief"
        }),
    );
    assert!(html.contains(r#"<html lang="fr-CA">"#));
    assert!(html.contains(r#"<meta name="description" content="Option-level web description">"#));
    assert!(html.contains(r#"<meta name="author" content="Strategy Office">"#));
    assert!(html.contains(r#"<meta name="neditor-status" content="published">"#));
    assert!(html.contains(r#"<link rel="canonical" href="https://example.com/option-brief">"#));
}

#[test]
fn semantic_exporters_map_ast_blocks() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Semantic Export\nstatus: approved\napprovedBy: QA\n---\n# Semantic Exports\nBusiness paragraph with [source](https://example.com/report).\n\n- [x] Confirm controls\n- [ ] Final approval\n\n| Metric | Value |\n| --- | ---: |\n| Total | =SUM(1,2) |\n\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:diagram caption=\"System diagram\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n{{page-break}}\n{{section-break columns=2 header=\"Section Header\" footer=\"Section {{page}}/{{pages}}\"}}\n\n{{slide title=\"Board Review\" layout=\"two-column\" header=\"Slide Header\" footer=\"Slide {{page}}/{{pages}}\" notes=\"Open with risk summary\\nClose with decision ask\"}}\nSlide-specific body.\nSecond column body.\n\n## Appendix\nAfter the break.\n".to_string(),
            file_path: None,
        });
    let options = json!({ "watermark": "DRAFT" });

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
    let docx_section_header = zip_entry_text(&docx, "word/header2.xml");
    let docx_section_footer = zip_entry_text(&docx, "word/footer2.xml");
    let docx_svg = zip_entry_text(&docx, "word/media/image1.svg");
    assert!(docx_content_types.contains(r#"ContentType="image/svg+xml""#));
    assert!(docx_content_types.contains(r#"PartName="/word/header2.xml""#));
    assert!(docx_content_types.contains(r#"PartName="/word/footer2.xml""#));
    assert!(docx_relationships.contains(r#"Id="rIdImage1""#));
    assert!(docx_relationships.contains(r#"Target="media/image1.svg""#));
    assert!(docx_relationships.contains(r#"Id="rIdHyperlink1""#));
    assert!(docx_relationships.contains(r#"Target="https://example.com/report""#));
    assert!(docx_relationships.contains(r#"TargetMode="External""#));
    assert!(docx_relationships.contains(r#"Id="rIdHeader2""#));
    assert!(docx_relationships.contains(r#"Target="header2.xml""#));
    assert!(docx_relationships.contains(r#"Id="rIdFooter2""#));
    assert!(docx_relationships.contains(r#"Target="footer2.xml""#));
    assert!(docx_document.contains(r#"r:embed="rIdImage1""#));
    assert!(docx_document.contains(r#"<w:hyperlink r:id="rIdHyperlink1""#));
    assert!(docx_document.contains(r#"<w:headerReference w:type="default" r:id="rIdHeader2""#));
    assert!(docx_document.contains(r#"<w:footerReference w:type="default" r:id="rIdFooter2""#));
    assert!(docx_section_header.contains("Section Header"));
    assert!(docx_section_footer.contains("Section "));
    assert!(docx_section_footer.contains(r#"<w:fldSimple w:instr="PAGE">"#));
    assert!(docx_section_footer.contains(r#"<w:fldSimple w:instr="NUMPAGES">"#));
    assert_eq!(docx_svg, "<svg/>");
    assert!(docx_document.contains(r#"<w:pStyle w:val="Heading1""#));
    assert!(docx_document.contains(r#"<w:pStyle w:val="Heading2""#));
    assert!(docx_document.contains("<w:tbl>"));
    assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));
    assert!(docx_document.contains("[x] Confirm controls"));
    assert!(docx_document.contains("[ ] Final approval"));
    assert!(docx_document.contains(r#"<w:br w:type="page""#));
    assert!(docx_document.contains(r#"<w:cols w:num="2""#));
    assert!(docx_document.contains("System diagram"));
    assert!(docx_document.contains("ROI = Gain / Cost"));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_content_types = zip_entry_text(&pptx, "[Content_Types].xml");
    let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
    let presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
    let slide_two_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide2.xml.rels");
    let slide_four_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide4.xml.rels");
    let slide_four_notes = zip_entry_text(&pptx, "ppt/notesSlides/notesSlide4.xml");
    let pptx_svg = zip_entry_text(&pptx, "ppt/media/image1.svg");
    assert!(pptx_content_types.contains(r#"ContentType="image/svg+xml""#));
    assert!(pptx_content_types.contains("presentationml.notesSlide+xml"));
    assert!(pptx_app.contains("<Notes>1</Notes>"));
    assert!(presentation.contains(r#"r:id="rId2""#));
    let slide_two = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
    assert!(slide_two.contains("Semantic Exports"));
    assert!(slide_two.contains("- [x] Confirm controls"));
    assert!(slide_two.contains("- [ ] Final approval"));
    assert!(slide_two.contains("Table: Metric | Value"));
    assert!(slide_two.contains("<a:tbl>"));
    assert!(slide_two.contains(r#"firstRow="1""#));
    assert!(slide_two.contains(r#"<a:pPr algn="r"/>"#));
    assert!(slide_two.contains("<a:t>Total</a:t>"));
    assert!(slide_two.contains("System diagram"));
    assert!(slide_two.contains("Business paragraph with source (https://example.com/report)."));
    assert!(slide_two.contains(r#"<a:hlinkClick r:id="rIdHyperlink1""#));
    assert!(slide_two.contains(r#"name="Footer""#));
    assert!(slide_two.contains("Page 2 of 5"));
    assert!(slide_two.contains(r#"r:embed="rIdImage1""#));
    assert!(slide_two_relationships.contains(r#"Target="../media/image1.svg""#));
    assert!(slide_two_relationships.contains(r#"Target="https://example.com/report""#));
    assert!(slide_two_relationships.contains(r#"TargetMode="External""#));
    assert_eq!(pptx_svg, "<svg/>");
    let slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
    assert!(slide_three.contains("Section"));
    assert!(slide_three.contains("Section break: columns=2"));
    assert!(slide_three.contains("Section Header"));
    assert!(slide_three.contains("Section 3/5"));
    let slide_four = zip_entry_text(&pptx, "ppt/slides/slide4.xml");
    assert!(slide_four.contains("Board Review"));
    assert!(slide_four.contains("Slide-specific body."));
    assert!(slide_four.contains("Second column body."));
    assert!(slide_four.contains(r#"name="Left Column""#));
    assert!(slide_four.contains(r#"name="Right Column""#));
    assert!(slide_four.contains("Slide Header"));
    assert!(slide_four.contains("Slide 4/5"));
    assert!(slide_four_relationships.contains(
        r#"Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide""#
    ));
    assert!(slide_four_relationships.contains(r#"Target="../notesSlides/notesSlide4.xml""#));
    assert!(slide_four_notes.contains("Open with risk summary"));
    assert!(slide_four_notes.contains("Close with decision ask"));
    let slide_five = zip_entry_text(&pptx, "ppt/slides/slide5.xml");
    assert!(slide_five.contains("Appendix"));

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("/Count 3"));
    assert!(pdf_text.contains("Page 1 of 3"));
    assert!(pdf_text.contains("Page 2 of 3"));
    assert!(pdf_text.contains(" re S"));
    assert!(pdf_text.contains("(Metric) Tj"));
    assert!(pdf_text.contains("(Total) Tj"));
    assert!(pdf_text.contains("- [x] Confirm controls"));
    assert!(pdf_text.contains("- [ ] Final approval"));
    assert!(pdf_text.contains("Section break: columns=2"));
    assert!(pdf_text.contains("Section Header"));
    assert!(pdf_text.contains("Section 3/3"));
    assert!(pdf_text.contains("System diagram"));
    assert!(pdf_text.contains("After the break."));
}
