use super::*;

#[test]
fn example_project_fixtures_compile_and_export() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let examples: [(&str, &[&str], &[&str]); 7] = [
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
        (
            "examples/showcase/neditor-capability-showcase.md",
            &[
                "Business writers",
                "Proposal teams",
                "Analysts and reviewers",
                "Release managers",
            ][..],
            &[
                "include",
                "chart",
                "csv",
                "tsv",
                "mermaid",
                "d2",
                "plantuml",
                "timeline",
                "roadmap",
                "vega-lite",
                "geojson",
                "topojson",
                "stl",
                "qr",
                "json",
                "yaml",
                "openapi",
                "json-schema",
                "bibtex",
                "citation",
                "equation",
                "ai-source",
                "comment",
                "review",
            ],
        ),
    ];
    let example_paths = examples
        .iter()
        .map(|(relative_path, _, _)| (*relative_path).to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        readme_example_links(&root),
        example_paths,
        "README example links should stay synchronized with executable example fixtures"
    );
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
        assert_eq!(
            metadata_string_value(&response.metadata, "positioning.model").as_deref(),
            Some("local-first document-file workbench"),
            "{relative_path} should declare local-first product positioning"
        );
        assert_eq!(
            metadata_string_value(&response.metadata, "positioning.sourceOfTruth").as_deref(),
            Some("Markdown source file"),
            "{relative_path} should declare Markdown source of truth"
        );
        assert_eq!(
            metadata_string_value(&response.metadata, "positioning.cloudSync").as_deref(),
            Some("false"),
            "{relative_path} should explicitly reject background cloud sync"
        );
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
        assert!(
            expected_personas
                .iter()
                .any(|persona| html.contains(persona)),
            "{relative_path} should carry target persona metadata into HTML"
        );
        assert!(
            html.contains(r#"<meta name="neditor-delivery-model" content="local-first document-file workbench">"#)
                && html.contains(r#"<meta name="neditor-source-of-truth" content="Markdown source file">"#),
            "{relative_path} should carry local-first positioning into HTML metadata"
        );
        for feature_marker in feature_markers {
            assert!(
                exported_html_contains_feature(&html, feature_marker),
                "{relative_path} should carry feature marker {feature_marker:?} into HTML"
            );
        }

        let pdf = render_pdf_bytes(&response, &options);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(
            pdf_text.contains(&format!("/Title ({})", response.semantic.title)),
            "{relative_path} should carry title metadata into PDF"
        );
        assert!(
            pdf_text.contains(&response.semantic.title),
            "{relative_path} should carry title text into PDF"
        );

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        assert!(docx.starts_with(b"PK"));
        assert!(zip_has_entry(&docx, "word/document.xml"));
        assert!(zip_has_entry(&docx, "docProps/core.xml"));
        assert!(zip_has_entry(&docx, "docProps/custom.xml"));
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        let docx_core = zip_entry_text(&docx, "docProps/core.xml");
        let docx_custom = zip_entry_text(&docx, "docProps/custom.xml");
        assert!(
            docx_document.contains(&response.semantic.title),
            "{relative_path} should carry title text into DOCX"
        );
        assert!(
            docx_core.contains(&format!("<dc:title>{}</dc:title>", response.semantic.title)),
            "{relative_path} should carry title metadata into DOCX core properties"
        );
        assert!(
            docx_custom.contains(r#"name="NEditorSourceHash""#),
            "{relative_path} should carry source hash evidence into DOCX custom properties"
        );
        assert!(
            docx_custom.contains(r#"name="NEditorTargetPersona""#),
            "{relative_path} should carry audience evidence into DOCX custom properties"
        );
        assert!(
            docx_custom.contains(r#"name="NEditorDeliveryModel""#)
                && docx_custom.contains("local-first document-file workbench")
                && docx_custom.contains(r#"name="NEditorSourceOfTruth""#)
                && docx_custom.contains("Markdown source file"),
            "{relative_path} should carry local-first positioning into DOCX custom properties"
        );
        for expected_persona in expected_personas {
            assert!(
                docx_custom.contains(expected_persona),
                "{relative_path} should carry target persona {expected_persona:?} into DOCX custom properties"
            );
        }

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        assert!(pptx.starts_with(b"PK"));
        assert!(zip_has_entry(&pptx, "ppt/presentation.xml"));
        assert!(zip_has_entry(&pptx, "docProps/core.xml"));
        assert!(zip_has_entry(&pptx, "docProps/custom.xml"));
        let pptx_slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/").join("\n");
        let pptx_core = zip_entry_text(&pptx, "docProps/core.xml");
        let pptx_custom = zip_entry_text(&pptx, "docProps/custom.xml");
        assert!(
            pptx_slides.contains(&response.semantic.title),
            "{relative_path} should carry title text into PPTX slides"
        );
        assert!(
            pptx_core.contains(&format!("<dc:title>{}</dc:title>", response.semantic.title)),
            "{relative_path} should carry title metadata into PPTX core properties"
        );
        assert!(
            pptx_custom.contains(r#"name="NEditorSourceHash""#),
            "{relative_path} should carry source hash evidence into PPTX custom properties"
        );
        assert!(
            pptx_custom.contains(r#"name="NEditorTargetPersona""#),
            "{relative_path} should carry audience evidence into PPTX custom properties"
        );
        assert!(
            pptx_custom.contains(r#"name="NEditorDeliveryModel""#)
                && pptx_custom.contains("local-first document-file workbench")
                && pptx_custom.contains(r#"name="NEditorSourceOfTruth""#)
                && pptx_custom.contains("Markdown source file"),
            "{relative_path} should carry local-first positioning into PPTX custom properties"
        );
        for expected_persona in expected_personas {
            assert!(
                pptx_custom.contains(expected_persona),
                "{relative_path} should carry target persona {expected_persona:?} into PPTX custom properties"
            );
        }

        let mut manifest = response.export_manifest.clone();
        manifest.export_target = "markdown-bundle".to_string();
        manifest.export_options = options;
        let bundle = render_markdown_bundle_bytes(&response, &manifest).expect("bundle bytes");
        assert!(bundle.starts_with(b"PK"));
        assert!(zip_has_entry(&bundle, "document.md"));
        assert!(zip_has_entry(&bundle, "document.txt"));
        assert!(zip_has_entry(&bundle, "manifest.json"));
        assert!(zip_has_entry(&bundle, "semantic.json"));
        assert!(zip_has_entry(&bundle, "metadata.json"));
        assert!(zip_has_entry(&bundle, "source-map.json"));
        assert!(zip_has_entry(&bundle, "diagnostics.json"));
        assert!(zip_has_entry(&bundle, "transform-artifacts.json"));
        let bundled_text = zip_entry_text(&bundle, "document.txt");
        let bundled_manifest = zip_entry_text(&bundle, "manifest.json");
        let bundled_semantic = zip_entry_text(&bundle, "semantic.json");
        let bundled_metadata = zip_entry_text(&bundle, "metadata.json");
        assert!(
            bundled_text.contains(&response.semantic.title),
            "{relative_path} should carry title text into Markdown bundle text"
        );
        assert!(
            bundled_manifest.contains(&format!(
                "\"document_title\": \"{}\"",
                response.semantic.title
            )),
            "{relative_path} should carry title metadata into Markdown bundle manifest"
        );
        assert!(
            bundled_semantic.contains(&format!("\"title\": \"{}\"", response.semantic.title)),
            "{relative_path} should carry title metadata into semantic bundle evidence"
        );
        for expected_persona in expected_personas {
            assert!(
                bundled_metadata.contains(expected_persona),
                "{relative_path} should carry target persona metadata into bundled metadata"
            );
        }
        assert!(
            bundled_metadata.contains("local-first document-file workbench")
                && bundled_metadata.contains("Markdown source file")
                && bundled_metadata.contains("\"cloudSync\": false"),
            "{relative_path} should carry local-first positioning into bundled metadata"
        );
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

fn readme_example_links(root: &Path) -> BTreeSet<String> {
    let readme = fs::read_to_string(root.join("README.md")).expect("README should be readable");
    readme
        .lines()
        .filter_map(|line| {
            let start = line.find("](examples/")?;
            let link = &line[start + 2..];
            let end = link.find(')')?;
            Some(link[..end].to_string())
        })
        .collect()
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

fn metadata_string_value(metadata: &Value, path: &str) -> Option<String> {
    let mut current = metadata;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    match current {
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        other => Some(other.to_string()),
    }
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

fn exported_html_contains_feature(html: &str, marker: &str) -> bool {
    match marker {
        "include" => true,
        "comment" | "review" => html.contains("export-comments") || html.contains("comment"),
        "ai-source" => html.contains("export-provenance") || html.contains("ai-source"),
        "citation" => html.contains("citation"),
        "equation" => html.contains("equation"),
        "formula" => html.contains("Net total") || html.contains("$67500.00"),
        "csv" => html.contains("Quarterly rollout budget") || html.contains("Q1"),
        other => html.contains(&format!("transform-{other}")) || html.contains(other),
    }
}
