use super::*;

#[test]
fn compiler_loads_external_bibliography_and_validates_cross_refs() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-bib-test-{unique}"));
    fs::create_dir_all(&root).expect("create bib test dir");
    fs::write(
            root.join("refs.bib"),
            "@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n@article{doe2026,\n title={Evidence Based Reports},\n author={Doe},\n date={2026-04-01}\n}",
        )
        .expect("write bibliography");
    fs::write(root.join("diagram.svg"), "<svg></svg>").expect("write figure");

    let response = compile(CompileRequest {
            text: "---\ntitle: Cited\nstatus: approved\napprovedBy: QA\nbibliography: refs.bib\ncitationStyle: author-year\n---\n# Cited\nClaim [@porter1985, p. 42; @doe2026].\n\n![Diagram](diagram.svg){#fig:diagram caption=\"System diagram\"}\nSee {@fig:diagram} and {@fig:missing}.\n\n![Missing](missing.png){#fig:missing-image caption=\"Missing image\"}".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

    assert_eq!(response.bibliography.len(), 2);
    assert!(response
        .bibliography
        .iter()
        .any(|entry| entry.key == "doe2026" && entry.issued.as_deref() == Some("2026")));
    assert_eq!(response.semantic.citations, vec!["doe2026", "porter1985"]);
    assert!(response
        .semantic
        .citation_references
        .iter()
        .any(|citation| {
            citation.key == "porter1985"
                && citation.locator.as_deref() == Some("p. 42")
                && citation.column == 8
                && citation.end_column > citation.column
        }));
    assert!(response.html.contains("Porter 1985, p. 42; Doe 2026"));
    assert!(response.html.contains(
        "title=\"@porter1985 (p. 42): Competitive Advantage; @doe2026: Evidence Based Reports\""
    ));
    assert!(response
            .html
            .contains("aria-label=\"Citation: @porter1985 (p. 42): Competitive Advantage; @doe2026: Evidence Based Reports\""));
    assert!(response.html.contains("<figure"));
    assert!(response.html.contains("System diagram"));
    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Broken image path")));
    assert!(response
        .semantic
        .cross_references
        .iter()
        .any(|reference| reference.key == "fig:diagram"
            && reference.resolved
            && reference.line == 12));
    let broken_cross_reference = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .contains("Broken cross reference: fig:missing")
        })
        .expect("broken cross-reference diagnostic");
    assert_eq!(broken_cross_reference.line, Some(12));
    assert!(broken_cross_reference.column.is_some());
    assert!(broken_cross_reference.end_column > broken_cross_reference.column);
    assert!(broken_cross_reference
        .related
        .iter()
        .any(|related| related.contains("{@fig:missing}")));
    fs::remove_dir_all(root).expect("clean bib test dir");
}

#[test]
fn citation_references_ignore_fenced_examples() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Fenced Citations\nstatus: approved\napprovedBy: QA\n---\n# Fenced Citations\nClaim [@real2026].\n\n```md\nExample [@not-real] should stay literal.\n```\n\n```bibtex\n@article{real2026,\n title={Real Evidence},\n author={Doe},\n year={2026}\n}\n```\n"
            .to_string(),
        file_path: None,
    });

    assert_eq!(response.semantic.citations, vec!["real2026"]);
    assert!(response
        .semantic
        .citation_references
        .iter()
        .all(|reference| reference.key != "not-real"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("not-real")));
    assert!(response
        .compiled_markdown
        .contains("Example [@not-real] should stay literal."));
}

#[test]
fn compile_options_supply_default_citation_style() {
    let response = compile_with_options(
            CompileRequest {
                text: "Claim [@porter1985].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n```".to_string(),
                file_path: None,
            },
            &json!({ "defaultCitationStyle": "author-year" }),
        );

    assert_eq!(
        response
            .metadata
            .get("citationStyle")
            .and_then(Value::as_str),
        Some("author-year")
    );
    assert!(response.html.contains("Porter 1985"));
}

#[test]
fn compile_options_do_not_override_document_citation_style() {
    let response = compile_with_options(
            CompileRequest {
                text: "---\ncitationStyle: key\n---\nClaim [@porter1985].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n```".to_string(),
                file_path: None,
            },
            &json!({ "defaultCitationStyle": "author-year" }),
        );

    assert_eq!(
        response
            .metadata
            .get("citationStyle")
            .and_then(Value::as_str),
        Some("key")
    );
    assert!(response.html.contains("@porter1985"));
}

#[test]
fn compile_options_supply_numeric_default_citation_style() {
    let response = compile_with_options(
            CompileRequest {
                text: "Claim [@porter1985].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n```".to_string(),
                file_path: None,
            },
            &json!({ "defaultCitationStyle": "numeric" }),
        );

    assert_eq!(
        response
            .metadata
            .get("citationStyle")
            .and_then(Value::as_str),
        Some("numeric")
    );
    assert!(response.html.contains("[1]"));
}

#[test]
fn compiler_renders_numeric_citations_and_numbered_bibliography() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Numeric Citations\nstatus: approved\napprovedBy: QA\ncitationStyle: numeric\n---\n# Numeric Citations\nSingle [@porter1985].\nMultiple [@porter1985, p. 42; @doe2026].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n@article{doe2026,\n title={Evidence Based Reports},\n author={Doe},\n year={2026}\n}\n```\n\n[BIBLIOGRAPHY]\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("[1]"));
    assert!(response.html.contains("[1, p. 42; 2]"));
    assert!(response.html.contains("[1] <strong>porter1985</strong>"));
    assert!(response.html.contains("[2] <strong>doe2026</strong>"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Unsupported citation style")));
}

#[test]
fn compiler_warns_and_falls_back_for_unsupported_csl_style() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Unsupported CSL\nstatus: approved\napprovedBy: QA\ncslStyle: apa\n---\n# Unsupported CSL\nClaim [@porter1985].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n```\n[BIBLIOGRAPHY]".to_string(),
            file_path: None,
        });

    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Unsupported citation style: apa")));
    assert!(response.html.contains("(Competitive Advantage)"));
    assert!(!response.html.contains("(Porter 1985)"));
}

#[test]
fn compiler_loads_csl_json_bibliography() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-csl-test-{unique}"));
    fs::create_dir_all(&root).expect("create csl test dir");
    fs::write(
        root.join("refs.json"),
        r#"[{"id":"doe2026","title":"Evidence Based Reports"}]"#,
    )
    .expect("write csl bibliography");

    let response = compile(CompileRequest {
            text: "---\ntitle: CSL\nstatus: approved\napprovedBy: QA\nbibliography: refs.json\n---\n# CSL\nClaim [@doe2026].\n[BIBLIOGRAPHY]".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

    assert_eq!(response.bibliography[0].key, "doe2026");
    assert!(response.html.contains("Evidence Based Reports"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Broken citation")));
    fs::remove_dir_all(root).expect("clean csl test dir");
}

#[test]
fn compiler_loads_hayagriva_yaml_bibliography() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Hayagriva\nstatus: approved\napprovedBy: QA\ncitationStyle: author-year\n---\n# Hayagriva\nClaim [@porter1985].\n\n```hayagriva\nporter1985:\n  type: book\n  title: Competitive Advantage\n  author: Porter\n  date: 1985\n```\n[BIBLIOGRAPHY]".to_string(),
            file_path: None,
        });

    assert_eq!(response.bibliography.len(), 1);
    assert_eq!(response.bibliography[0].key, "porter1985");
    assert_eq!(response.bibliography[0].author.as_deref(), Some("Porter"));
    assert_eq!(response.bibliography[0].issued.as_deref(), Some("1985"));
    assert!(response.html.contains("Porter 1985"));
    assert!(response.html.contains("Competitive Advantage"));
}

#[test]
fn compiler_reports_duplicate_bibliography_keys() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Duplicate Bibliography\nstatus: approved\napprovedBy: QA\n---\n# Duplicate Bibliography\nClaim [@porter1985].\n\n```bibtex\n@book{porter1985, title={Competitive Advantage}}\n@article{porter1985, title={Duplicate Entry}}\n```\n[BIBLIOGRAPHY]".to_string(),
            file_path: None,
        });

    assert_eq!(
        response.semantic.duplicate_bibliography_keys,
        vec!["porter1985".to_string()]
    );
    let duplicate = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Duplicate bibliography key"))
        .expect("duplicate bibliography diagnostic");
    assert_eq!(duplicate.source_file.as_deref(), Some("untitled.md"));
    assert_eq!(duplicate.line, Some(11));
    assert_eq!(duplicate.column, Some(10));
    assert_eq!(duplicate.end_column, Some(20));
    assert!(duplicate
        .related
        .iter()
        .any(|related| related.contains("First occurrence: untitled.md:10")));
}

#[test]
fn compiler_reports_csl_and_hayagriva_duplicate_key_locations() {
    let csl = compile(CompileRequest {
        text: "---\ntitle: CSL Duplicate\nstatus: approved\napprovedBy: QA\n---\n# CSL Duplicate\nClaim [@dup2026].\n\n```bibliography\n[\n{\"id\":\"dup2026\",\"title\":\"First Entry\"},\n{\"id\":\"dup2026\",\"title\":\"Second Entry\"}\n]\n```\n[BIBLIOGRAPHY]"
            .to_string(),
        file_path: None,
    });
    let csl_duplicate = csl
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Duplicate bibliography key"))
        .expect("CSL duplicate diagnostic");
    assert_eq!(csl_duplicate.line, Some(12));
    assert_eq!(csl_duplicate.column, Some(8));
    assert_eq!(csl_duplicate.end_column, Some(15));
    assert!(csl_duplicate
        .related
        .iter()
        .any(|related| related.contains("First occurrence: untitled.md:11")));

    let hayagriva = compile(CompileRequest {
        text: "---\ntitle: Hayagriva Duplicate\nstatus: approved\napprovedBy: QA\n---\n# Hayagriva Duplicate\nClaim [@dup2026].\n\n```hayagriva\ndup2026:\n  type: article\n  title: First Entry\ndup2026:\n  type: article\n  title: Second Entry\n```\n[BIBLIOGRAPHY]"
            .to_string(),
        file_path: None,
    });
    assert_eq!(
        hayagriva.semantic.duplicate_bibliography_keys,
        vec!["dup2026".to_string()]
    );
    let hayagriva_duplicate = hayagriva
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Duplicate bibliography key"))
        .expect("Hayagriva duplicate diagnostic");
    assert_eq!(hayagriva_duplicate.line, Some(13));
    assert_eq!(hayagriva_duplicate.column, Some(1));
    assert_eq!(hayagriva_duplicate.end_column, Some(8));
    assert!(hayagriva_duplicate
        .related
        .iter()
        .any(|related| related.contains("First occurrence: untitled.md:10")));
}

#[test]
fn compiler_reports_duplicate_keys_across_external_bibliography_files() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-multi-bib-test-{unique}"));
    fs::create_dir_all(&root).expect("create multi bibliography test dir");
    let primary = root.join("refs-primary.bib");
    let secondary = root.join("refs-secondary.json");
    fs::write(
        &primary,
        "@book{dup2026,\n title={Primary Entry},\n author={One},\n year={2026}\n}",
    )
    .expect("write primary bibliography");
    fs::write(
        &secondary,
        "[\n{\"id\":\"dup2026\",\"title\":\"Secondary Entry\",\"author\":[{\"family\":\"Two\"}],\"issued\":{\"date-parts\":[[2026]]}}\n]",
    )
    .expect("write secondary bibliography");

    let response = compile(CompileRequest {
        text: "---\ntitle: Multi Bibliography\nstatus: approved\napprovedBy: QA\nbibliography:\n  - refs-primary.bib\n  - refs-secondary.json\ncitationStyle: author-year\n---\n# Multi Bibliography\nClaim [@dup2026].\n[BIBLIOGRAPHY]"
            .to_string(),
        file_path: Some(path_to_string(&root.join("root.md"))),
    });

    assert_eq!(response.bibliography.len(), 2);
    assert_eq!(
        response.semantic.duplicate_bibliography_keys,
        vec!["dup2026".to_string()]
    );
    let duplicate = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Duplicate bibliography key"))
        .expect("external duplicate bibliography diagnostic");
    assert_eq!(
        duplicate.source_file.as_deref(),
        Some(path_to_string(&secondary).as_str())
    );
    assert_eq!(duplicate.line, Some(2));
    assert_eq!(duplicate.column, Some(8));
    assert_eq!(duplicate.end_column, Some(15));
    assert!(duplicate.related.iter().any(|related| {
        related.contains(&format!("First occurrence: {}:1", path_to_string(&primary)))
    }));
    assert!(response.html.contains("Two 2026"));
    fs::remove_dir_all(root).expect("clean multi bibliography test dir");
}

#[test]
fn citation_export_conformance_covers_required_cases() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Citation Export\nstatus: approved\napprovedBy: QA\ncitationStyle: author-year\n---\n# Citation Export\nSingle [@porter1985].\nMultiple [@porter1985; @doe2026].\nLocator [@porter1985, p. 42].\nMissing [@missing2026].\nSecond [@doe2026].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n@article{doe2026,\n title={Evidence Based Reports},\n author={Doe},\n year={2026}\n}\n```\n\n[BIBLIOGRAPHY]\n".to_string(),
            file_path: None,
        });
    let options = json!({});

    assert_eq!(
        response.semantic.citations,
        vec!["doe2026", "missing2026", "porter1985"]
    );
    let broken_citation = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Broken citation: missing2026"))
        .expect("broken citation diagnostic");
    assert_eq!(broken_citation.line, Some(11));
    assert_eq!(broken_citation.column, Some(10));
    assert!(broken_citation.end_column > broken_citation.column);
    assert!(broken_citation
        .related
        .iter()
        .any(|related| related.contains("@missing2026")));

    let html = render_full_html(&response, &options);
    assert!(html.contains("Porter 1985"));
    assert!(html.contains("Porter 1985; Doe 2026"));
    assert!(html.contains("Porter 1985, p. 42"));
    assert!(html.contains("missing bibliography entry"));
    assert!(html.contains("Bibliography"));
    assert!(html.contains("Competitive Advantage"));
    assert!(html.contains("Evidence Based Reports"));

    let docx = render_docx_bytes(&response, &options).expect("docx citation bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("Porter 1985"));
    assert!(docx_document.contains("Porter 1985; Doe 2026"));
    assert!(docx_document.contains("Porter 1985, p. 42"));
    assert!(docx_document.contains("missing2026"));
    assert!(docx_document.contains("Competitive Advantage"));
    assert!(docx_document.contains("Evidence Based Reports"));
    assert!(docx_document.contains(r#"w:name="bib_porter1985""#));
    assert!(docx_document.contains(r#"w:name="bib_doe2026""#));
    assert!(docx_document.contains(r#"w:instr="CITATION porter1985 \l 1033""#));
    assert!(docx_document.contains(r#"w:instr="CITATION porter1985 \m doe2026 \l 1033""#));
    assert!(docx_document.contains(r#"w:instr="BIBLIOGRAPHY \l 1033""#));
    assert!(docx_document.contains(r#"<w:hyperlink w:anchor="bib_porter1985""#));
    assert!(docx_document.contains(r#"<w:hyperlink w:anchor="bib_doe2026""#));
    assert!(!docx_document.contains(r#"w:anchor="bib_missing2026""#));

    let pptx = render_pptx_bytes(&response, &options).expect("pptx citation bytes");
    let slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/");
    assert!(slides.iter().any(|slide| slide.contains("Porter 1985")));
    assert!(slides
        .iter()
        .any(|slide| slide.contains("Porter 1985; Doe 2026")));
    assert!(slides
        .iter()
        .any(|slide| slide.contains("Porter 1985, p. 42")));
    assert!(slides.iter().any(|slide| slide.contains("missing2026")));
    assert!(slides
        .iter()
        .any(|slide| slide.contains("Competitive Advantage")));
    assert!(slides
        .iter()
        .any(|slide| slide.contains("Evidence Based Reports")));
}

#[test]
fn document_ast_preserves_multiple_citation_keys() {
    let response = compile(CompileRequest {
            text: "---\ntitle: AST Citations\nstatus: approved\napprovedBy: QA\ncitationStyle: key\n---\n# AST Citations\nClaim [@porter1985, p. 42; @doe2026].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n@article{doe2026,\n title={Evidence Based Reports},\n author={Doe},\n year={2026}\n}\n```\n"
                .to_string(),
            file_path: None,
        });

    let citation = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| match block {
            DocumentBlock::Paragraph { inlines, .. } => {
                inlines.iter().find_map(|inline| match inline {
                    document_ast::InlineNode::Citation { key, keys, raw } => Some((key, keys, raw)),
                    _ => None,
                })
            }
            _ => None,
        })
        .expect("AST citation inline");

    assert_eq!(citation.0, "porter1985");
    assert_eq!(citation.1.as_slice(), ["porter1985", "doe2026"]);
    assert!(citation
        .2
        .contains("data-citation-keys=\"porter1985 doe2026\""));
}
