use super::*;

#[test]
fn example_project_fixtures_compile_and_export() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let examples: [(&str, &[&str], &[&str]); 6] = [
        (
            "examples/board-paper.md",
            &["Executives and managers"][..],
            &["chart", "glossary", "comment"],
        ),
        (
            "examples/consulting-report.md",
            &["Consultants"][..],
            &["include", "roadmap"],
        ),
        (
            "examples/technical-architecture.md",
            &[
                "Technical writers",
                "Product and engineering teams",
                "Developers",
            ][..],
            &["mermaid", "timeline", "adr"],
        ),
        (
            "examples/research-report.md",
            &["Researchers and analysts", "Students and academics"][..],
            &["bibtex", "citation", "equation"],
        ),
        (
            "examples/proposal-budget.md",
            &["Consultants", "Product and engineering teams"][..],
            &["calc", "csv", "formula"],
        ),
        (
            "examples/ai-assisted-draft.md",
            &[
                "Teams using AI chat output",
                "Product and engineering teams",
            ][..],
            &["ai-source", "comment", "review"],
        ),
    ];
    let mut covered_personas = BTreeSet::new();

    for (relative_path, expected_personas, feature_markers) in examples {
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
        let personas = metadata_string_array(&response.metadata, "targetPersona");
        for expected_persona in expected_personas {
            assert!(
                personas.iter().any(|persona| persona == expected_persona),
                "{relative_path} should declare target persona {expected_persona:?}: {personas:?}"
            );
            covered_personas.insert((*expected_persona).to_string());
        }
        for feature_marker in feature_markers {
            assert!(
                example_contains_feature(&response, feature_marker),
                "{relative_path} should exercise feature marker {feature_marker:?}"
            );
        }

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

    for required_persona in [
        "Consultants",
        "Technical writers",
        "Researchers and analysts",
        "Product and engineering teams",
        "Executives and managers",
        "Students and academics",
        "Developers",
        "Teams using AI chat output",
    ] {
        assert!(
            covered_personas.contains(required_persona),
            "example fixtures should cover target persona {required_persona:?}"
        );
    }
}

fn metadata_string_array(metadata: &Value, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn example_contains_feature(response: &CompileResponse, marker: &str) -> bool {
    match marker {
        "include" => !response.include_graph.is_empty(),
        "comment" => !response.semantic.comments.is_empty(),
        "review" => {
            !response.semantic.comments.is_empty()
                || !response.semantic.change_notes.is_empty()
                || !response.semantic.ai_sources.is_empty()
                || !response.semantic.ai_assisted_sections.is_empty()
        }
        "ai-source" => !response.semantic.ai_sources.is_empty(),
        "citation" => !response.semantic.citation_references.is_empty(),
        "equation" => response.semantic.equations > 0,
        "formula" => !response.formula_graph.is_empty(),
        other => response
            .transform_artifacts
            .iter()
            .any(|artifact| artifact.name == other),
    }
}
