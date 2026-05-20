use super::*;

#[test]
fn example_project_fixtures_compile_and_export() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let examples = [
        "examples/board-paper.md",
        "examples/consulting-report.md",
        "examples/technical-architecture.md",
        "examples/research-report.md",
        "examples/proposal-budget.md",
        "examples/ai-assisted-draft.md",
    ];

    for relative_path in examples {
        let path = root.join(relative_path);
        let source = fs::read_to_string(&path).expect("example fixture should be readable");
        let response = compile(CompileRequest {
            text: source,
            file_path: Some(path_to_string(&path)),
        });

        assert!(
            !response
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == "error"),
            "{relative_path} should compile without errors: {:?}",
            response.diagnostics
        );
        assert!(
            !response.semantic.title.trim().is_empty(),
            "{relative_path} should provide a title"
        );
        assert!(
            !response.source_map.is_empty(),
            "{relative_path} should produce source-map entries"
        );
        assert_eq!(
            response.export_manifest.document_title,
            response.semantic.title
        );

        let options = json!({
            "includeGlossary": true,
            "includeComments": true,
            "includeProvenance": true,
            "includeAgenda": true,
            "watermark": "SAMPLE"
        });
        let html = render_full_html(&response, &options);
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains(&response.semantic.title));

        let pdf = render_pdf_bytes(&response, &options);
        assert!(pdf.starts_with(b"%PDF-1.4"));

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        assert!(docx.starts_with(b"PK"));
        assert!(zip_has_entry(&docx, "word/document.xml"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        assert!(pptx.starts_with(b"PK"));
        assert!(zip_has_entry(&pptx, "ppt/presentation.xml"));

        let mut manifest = response.export_manifest.clone();
        manifest.export_target = "markdown-bundle".to_string();
        manifest.export_options = options;
        let bundle = render_markdown_bundle_bytes(&response, &manifest).expect("bundle bytes");
        assert!(bundle.starts_with(b"PK"));
        assert!(zip_has_entry(&bundle, "document.md"));
        assert!(zip_has_entry(&bundle, "manifest.json"));
    }
}
