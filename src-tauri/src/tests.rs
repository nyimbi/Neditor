use super::*;
use std::io::{Cursor, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

#[cfg(unix)]
fn write_executable_script(prefix: &str, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("neditor-{prefix}-{unique}.sh"));
    fs::write(&path, body).expect("write executable test script");
    let mut permissions = fs::metadata(&path).expect("script metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("make script executable");
    path
}

fn installed_command_path(env_var: &str, command: &str) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(env_var)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute() && path.is_file())
    {
        return Some(path);
    }

    let path_value = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path_value) {
        let direct = directory.join(command);
        if direct.is_file() {
            return Some(direct);
        }
        #[cfg(windows)]
        {
            for extension in ["exe", "bat", "cmd"] {
                let candidate = directory.join(format!("{command}.{extension}"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn sample_document() -> String {
    r#"---
title: Test Report
version: 1.2.0
status: approved
approvedBy: QA
toc: true
client: Acme
brand:
  name: Acme
  logo: "data:image/svg+xml;base64,PHN2Zy8+"
---

# Test Report

[TOC]

Prepared for {{client}}.

```calc
revenue = 100
cost = 40
profit = revenue - cost
margin = profit / revenue
healthy = IF(revenue > cost, 1, 0)
target_met = IF(margin >= 0.60, 1, 0)
cost_match = IF(cost == 40, 1, 0)
spread = IF(revenue != cost, 1, 0)
discount = 12.5%
```

Margin: {{=margin | percent}}
After tax: {{=profit * 0.70 | currency}}
Healthy score: {{=IF(revenue > cost, profit, 0) | round}}
Discount: {{=discount | percent}}

```csv caption="Regional revenue" audited
Region,Revenue
East,100
West,80
```

```glossary
ARR: Annual recurring revenue.
```

[INDEX]
"#
    .to_string()
}

#[test]
fn compiler_resolves_metadata_variables_transforms_and_manifest() {
    let response = compile(CompileRequest {
        text: sample_document(),
        file_path: None,
    });

    assert_eq!(response.semantic.title, "Test Report");
    assert_eq!(response.semantic.status, "approved");
    assert!(response.compiled_markdown.contains("Prepared for Acme."));
    assert!(response.compiled_markdown.contains("Margin: 60.00%"));
    assert!(response.compiled_markdown.contains("After tax: $42.00"));
    assert!(response.compiled_markdown.contains("Healthy score: 60"));
    assert!(response.compiled_markdown.contains("Discount: 12.50%"));
    assert!(response.html.contains("Table of Contents"));
    assert!(response.html.contains("transform-table"));
    assert!(response.html.contains("<h1 id=\"test-report\">"));
    assert!(response.html.contains("href=\"#test-report\""));
    assert!(response.index_terms.iter().any(|term| term == "ARR"));
    assert_eq!(response.export_manifest.document_version, "1.2.0");
    let csv_artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "csv")
        .expect("csv transform artifact");
    assert!(!csv_artifact.output_hash.is_empty());
    assert!(csv_artifact.source.contains("Region,Revenue"));
    assert!(csv_artifact.source_line.is_some_and(|line| line > 1));
    assert!(csv_artifact.end_source_line >= csv_artifact.source_line);
    assert_eq!(
        csv_artifact.options.get("caption").and_then(Value::as_str),
        Some("Regional revenue")
    );
    assert_eq!(
        csv_artifact.options.get("audited").and_then(Value::as_bool),
        Some(true)
    );
    let manifest_csv_artifact = response
        .export_manifest
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.get("name").and_then(Value::as_str) == Some("csv"))
        .expect("csv manifest artifact");
    assert_eq!(
        manifest_csv_artifact
            .get("sourceHash")
            .and_then(Value::as_str),
        Some(csv_artifact.source_hash.as_str())
    );
    assert_eq!(
        manifest_csv_artifact.get("source").and_then(Value::as_str),
        Some(csv_artifact.source.as_str())
    );
    assert_eq!(
        manifest_csv_artifact
            .get("sourceFile")
            .and_then(Value::as_str),
        csv_artifact.source_file.as_deref()
    );
    assert_eq!(
        manifest_csv_artifact
            .get("sourceLine")
            .and_then(Value::as_u64),
        csv_artifact.source_line.map(|line| line as u64)
    );
    assert_eq!(
        manifest_csv_artifact
            .get("endSourceLine")
            .and_then(Value::as_u64),
        csv_artifact.end_source_line.map(|line| line as u64)
    );
    assert_eq!(
        manifest_csv_artifact
            .get("options")
            .and_then(|options| options.get("caption"))
            .and_then(Value::as_str),
        Some("Regional revenue")
    );
    assert_eq!(
        manifest_csv_artifact
            .get("outputHash")
            .and_then(Value::as_str),
        Some(csv_artifact.output_hash.as_str())
    );
    assert!(manifest_csv_artifact
        .get("cacheKey")
        .and_then(Value::as_str)
        .is_some_and(|cache_key| !cache_key.is_empty()));
    assert_eq!(
        manifest_csv_artifact
            .get("executionKind")
            .and_then(Value::as_str),
        Some("embedded")
    );
    assert_eq!(
        manifest_csv_artifact
            .get("inputMode")
            .and_then(Value::as_str),
        Some("embedded")
    );
    assert_eq!(
        manifest_csv_artifact
            .get("engineVersion")
            .and_then(Value::as_str),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "profit" && formula.value == Some(60.0)));
    let profit_formula = response
        .formula_graph
        .iter()
        .find(|formula| formula.name == "profit")
        .expect("profit formula");
    assert!(matches!(
        profit_formula.ast.as_ref(),
        Some(calculations::FormulaAstNode::Binary { op, .. }) if op == "-"
    ));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "healthy" && formula.value == Some(1.0)));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "target_met" && formula.value == Some(1.0)));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "cost_match" && formula.value == Some(1.0)));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "spread" && formula.value == Some(1.0)));
    assert!(response.formula_graph.iter().any(|formula| {
        formula.name == "discount"
            && (formula.value.unwrap_or_default() - 0.125).abs() < f64::EPSILON
    }));
}

#[test]
fn compiler_supports_default_document_variables() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Defaults\nstatus: approved\napprovedBy: QA\nclient: Acme\n---\n# Defaults\nPrepared for {{client | default:Fallback}} in {{region | default:\"East Africa\"}}.\nStill missing {{owner}}.\n".to_string(),
            file_path: None,
        });

    assert!(response
        .compiled_markdown
        .contains("Prepared for Acme in East Africa."));
    let missing_owner = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .message
                .contains("Missing document variable: owner")
        })
        .expect("missing owner diagnostic");
    assert_eq!(missing_owner.line, Some(9));
    assert_eq!(missing_owner.column, Some(15));
    assert_eq!(missing_owner.end_line, Some(9));
    assert_eq!(missing_owner.end_column, Some(24));
    assert_eq!(missing_owner.source_file.as_deref(), Some("untitled.md"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("region")));
}

#[test]
fn calc_blocks_resolve_forward_refs_and_report_cycles() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Calc Graph\nstatus: approved\napprovedBy: QA\n---\n# Calc Graph\n```calc\nprofit = revenue - cost\ncost = 40\nrevenue = 100\ncycle_a = cycle_b + 1\ncycle_b = cycle_a + 1\n```\n\nProfit: {{=profit | round}}\n".to_string(),
            file_path: None,
        });

    assert!(response.compiled_markdown.contains("Profit: 60"));
    assert!(response
        .formula_graph
        .iter()
        .any(|formula| formula.name == "profit" && formula.value == Some(60.0)));
    assert!(response
        .formula_dependency_edges
        .iter()
        .any(|edge| edge.from == "profit" && edge.to == "revenue"));
    assert!(response
        .formula_dependency_edges
        .iter()
        .any(|edge| edge.from == "profit" && edge.to == "cost"));
    assert!(response.formula_graph.iter().any(|formula| {
        formula.name == "cycle_a"
            && formula
                .error
                .as_deref()
                .is_some_and(|error| error.contains("#CYCLE? cycle_a -> cycle_b -> cycle_a"))
    }));
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("#CYCLE? cycle_a -> cycle_b -> cycle_a")));
}

#[test]
fn inline_formula_diagnostics_include_source_ranges() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Formula Diagnostics\nstatus: approved\napprovedBy: QA\n---\n# Formula Diagnostics\nBad: {{=missing + 1}}\n"
                .to_string(),
            file_path: None,
        });

    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Inline formula error"))
        .expect("inline formula diagnostic");
    assert_eq!(diagnostic.line, Some(7));
    assert_eq!(diagnostic.column, Some(6));
    assert_eq!(diagnostic.end_line, Some(7));
    assert_eq!(diagnostic.end_column, Some(22));
    assert_eq!(diagnostic.source_file.as_deref(), Some("untitled.md"));
}

#[test]
fn compiler_loads_project_level_variables_without_overriding_front_matter() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-project-vars-test-{unique}"));
    fs::create_dir_all(root.join(".neditor")).expect("create project vars dir");
    fs::write(
        root.join(".neditor").join("variables.yaml"),
        "client: Project Client\nregion: West\nowner: Strategy Office\n",
    )
    .expect("write project variables");
    let doc = root.join("docs").join("report.md");
    fs::create_dir_all(doc.parent().expect("doc parent")).expect("create docs dir");
    fs::write(&doc, "# Report").expect("write doc");

    let response = compile(CompileRequest {
            text: "---\ntitle: Project Vars\nstatus: approved\napprovedBy: QA\nclient: Front Matter Client\n---\n# Project Vars\nPrepared for {{client}} in {{region}} by {{owner}}.\n".to_string(),
            file_path: Some(path_to_string(&doc)),
        });

    assert!(response
        .compiled_markdown
        .contains("Prepared for Front Matter Client in West by Strategy Office."));
    assert_eq!(response.metadata["client"], "Front Matter Client");
    assert_eq!(response.metadata["region"], "West");
    fs::remove_dir_all(root).expect("clean project vars test dir");
}

#[test]
fn compiler_loads_front_matter_csv_data_sources() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-data-source-test-{unique}"));
    fs::create_dir_all(root.join("data")).expect("create data dir");
    fs::write(
        root.join("data").join("revenue.csv"),
        "Region,Revenue\n\"East\nCoast\",100\nWest,\"=SUM(B1,80)\"\n",
    )
    .expect("write csv data source");

    let response = compile(CompileRequest {
            text: "---\ntitle: Data Source\nstatus: approved\napprovedBy: QA\ndataSources:\n  - name: Revenue\n    path: data/revenue.csv\n    type: csv\n---\n# Data Source\n".to_string(),
            file_path: Some(path_to_string(&root.join("report.md"))),
        });

    assert!(response
        .compiled_markdown
        .contains("## Data Source: Revenue"));
    assert!(response.html.contains("<td>180</td>"));
    assert!(response.html.contains("East\nCoast"));
    assert!(response
        .include_graph
        .iter()
        .any(|edge| edge.child.ends_with("data/revenue.csv")));
    assert!(response
        .export_manifest
        .included_files
        .iter()
        .any(|file| file.path.ends_with("data/revenue.csv")));
    assert!(response.export_manifest.source_hash.starts_with("sha256:"));
    assert!(response
        .export_manifest
        .included_files
        .iter()
        .all(|file| file.hash.starts_with("sha256:")));
    fs::remove_dir_all(root).expect("clean data source test dir");
}

#[test]
fn compiler_honors_toc_depth_and_numbering() {
    let response = compile(CompileRequest {
            text: "---\ntitle: TOC\nstatus: approved\napprovedBy: QA\ntoc: true\ntocDepth: 2\ntocNumbered: true\n---\n# Alpha\n## Beta\n### Gamma\n## Delta\n".to_string(),
            file_path: None,
        });

    assert!(response.compiled_markdown.contains("- [1 Alpha](#alpha)"));
    assert!(response.compiled_markdown.contains("  - [1.1 Beta](#beta)"));
    assert!(response
        .compiled_markdown
        .contains("  - [1.2 Delta](#delta)"));
    assert!(!response.compiled_markdown.contains("[1.1.1 Gamma](#gamma)"));
    let docx = render_docx_bytes(&response, &json!({})).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains(r#"w:instr="TOC \o &quot;1-2&quot; \h \z \u""#));
    assert!(!docx_document.contains("#alpha"));
}

#[test]
fn compiler_adds_glossary_hover_terms_to_preview_html() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Glossary Hover\nstatus: approved\napprovedBy: QA\n---\n# Glossary Hover\nARR informs planning.\n\n```glossary\nARR: Annual recurring revenue.\n```\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("class=\"glossary-term\""));
    assert!(response
        .html
        .contains("title=\"Annual recurring revenue.\""));
    assert!(response.html.contains(">ARR</span> informs planning"));
}

#[test]
fn compiler_preserves_figure_float_semantics() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Floating Figure\nstatus: approved\napprovedBy: QA\n---\n# Floating Figure\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:float caption=\"Floating diagram\" float=\"right\"}\n".to_string(),
            file_path: None,
        });

    assert!(response.html.contains("figure-float-right"));
    assert!(response.html.contains("data-float=\"right\""));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Figure {
                id,
                caption,
                float,
                ..
            } if id.as_deref() == Some("fig:float")
                && caption.as_deref() == Some("Floating diagram")
                && float.as_deref() == Some("right")
        )
    }));

    let exported = export::export_text(&response, &json!({}));
    assert!(exported.contains("float=right"));

    let full_html = render_full_html(&response, &json!({}));
    assert!(full_html.contains("figure-float-right"));
    assert!(full_html.contains("float:right"));

    let docx = render_docx_bytes(&response, &json!({})).expect("docx bytes");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    assert!(docx_document.contains("float=right"));
    assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));

    let pptx = render_pptx_bytes(&response, &json!({})).expect("pptx bytes");
    let floating_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
        .into_iter()
        .find(|slide| slide.contains(r#"r:embed="rIdImage1""#))
        .expect("floating figure slide");
    assert!(floating_slide.contains(r#"<a:off x="5029200""#));

    let pdf = render_pdf_bytes(&response, &json!({}));
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf_text.contains("287 627 240 135 re S"));
}

#[test]
fn compiler_generates_linked_index_with_exclusions_and_proper_terms() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Index\nstatus: approved\napprovedBy: QA\nindexExclude:\n  - internal draft\n---\n# Market Analysis\nAcme Strategy appears here. **Working Capital** matters.\n\n## Follow Up\nAcme Strategy returns. Internal Draft should stay out. Working capital{#index:Liquidity} marker.\n\n[INDEX]\n".to_string(),
            file_path: None,
        });

    assert!(response
        .index_terms
        .iter()
        .any(|term| term == "Acme Strategy"));
    assert!(response.index_terms.iter().any(|term| term == "Liquidity"));
    assert!(response
        .index_terms
        .iter()
        .any(|term| term == "Working Capital"));
    assert!(!response
        .index_terms
        .iter()
        .any(|term| term == "Internal Draft"));
    assert!(response.html.contains("href=\"#market-analysis\""));
    assert!(response.html.contains("Acme Strategy"));
    assert!(response.html.contains("Liquidity"));
    assert!(!response.html.contains("{#index:Liquidity}"));
}

#[test]
fn compiler_parses_review_comment_metadata() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Review\nstatus: approved\napprovedBy: QA\n---\n# Review\n<!-- comment: unresolved | author: Dana | at: 2026-05-18T10:00:00Z | Clarify the risk note. -->\n<!-- change: author: Dana | at: 2026-05-18T11:00:00Z | Updated the risk note. -->\n".to_string(),
            file_path: None,
        });
    let comment = response.semantic.comments.first().expect("review comment");
    let change_note = response.semantic.change_notes.first().expect("change note");

    assert_eq!(comment.state, "unresolved");
    assert_eq!(comment.author, "Dana");
    assert_eq!(comment.created_at, "2026-05-18T10:00:00Z");
    assert_eq!(comment.text, "Clarify the risk note.");
    assert_eq!(change_note.author, "Dana");
    assert_eq!(change_note.created_at, "2026-05-18T11:00:00Z");
    assert_eq!(change_note.text, "Updated the risk note.");
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::ReviewComment { comment, .. }
                if comment.author == "Dana"
                    && comment.state == "unresolved"
                    && comment.text == "Clarify the risk note."
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::ChangeNote { note, .. }
                if note.author == "Dana" && note.text == "Updated the risk note."
        )
    }));
    let unresolved_comment_diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("unresolved review comments"))
        .expect("unresolved comment diagnostic");
    assert_eq!(unresolved_comment_diagnostic.severity, "error");
    assert_eq!(unresolved_comment_diagnostic.line, Some(7));
    assert_eq!(
        unresolved_comment_diagnostic.source_file.as_deref(),
        Some("untitled.md")
    );
    assert!(unresolved_comment_diagnostic
        .related
        .iter()
        .any(|related| related.contains("Clarify the risk note")));
}

#[test]
fn compiler_reports_missing_include_without_panicking() {
    let response = compile(CompileRequest {
        text: "!include missing/chapter.md\n".to_string(),
        file_path: None,
    });

    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.severity == "error" && diagnostic.message.contains("Missing include file")
        })
        .expect("missing include diagnostic");
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related.contains("missing/chapter.md")));
}

#[test]
fn compiler_reports_broken_local_markdown_links() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-link-test-{unique}"));
    fs::create_dir_all(root.join("docs")).expect("create link test dir");
    fs::write(root.join("docs").join("existing.md"), "# Existing").expect("write linked doc");

    let response = compile(CompileRequest {
            text: "---\ntitle: Links\nstatus: approved\napprovedBy: QA\nbrand:\n  logo: docs/missing-logo.svg\n---\n# Links\nRead [existing](docs/existing.md), [missing](docs/missing.md), [section](#links), and [web](https://example.com).\n![Missing image](docs/missing.png)\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });
    let root_doc = path_to_string(&root.join("root.md"));

    let broken_link = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Broken link path"))
        .expect("broken link diagnostic");
    assert_eq!(broken_link.line, Some(9));
    assert!(broken_link.column.is_some());
    assert!(broken_link.end_column > broken_link.column);
    assert_eq!(broken_link.source_file.as_deref(), Some(root_doc.as_str()));
    assert!(broken_link
        .related
        .iter()
        .any(|related| related.contains("docs/missing.md")));
    assert_eq!(
        response
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.message.contains("Broken link path"))
            .count(),
        1
    );
    let broken_image = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Broken image path"))
        .expect("broken image diagnostic");
    assert_eq!(broken_image.line, Some(10));
    assert!(broken_image.column.is_some());
    assert!(broken_image.end_column > broken_image.column);
    assert_eq!(broken_image.source_file.as_deref(), Some(root_doc.as_str()));
    assert!(broken_image
        .related
        .iter()
        .any(|related| related.contains("docs/missing.png")));
    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Broken logo path")));
    fs::remove_dir_all(root).expect("clean link test dir");
}

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
    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Duplicate bibliography key")));
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

mod table_tests;

#[test]
fn cross_references_resolve_heading_appendix_and_decision_anchors() {
    let response = compile(CompileRequest {
            text: "---\ntitle: References\nstatus: approved\napprovedBy: QA\n---\n# Strategy {#sec:strategy}\nSee {@sec:strategy}, {@appendix-a}, and {@decision-record}.\n\n## Appendix A\nSupporting detail.\n\n## Decision Record\nUse local-first exports.\n".to_string(),
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
    assert!(response.compiled_markdown.contains(
            "See [Section strategy](#sec:strategy), [Section appendix a](#appendix-a), and [Section decision record](#decision-record)."
        ));
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
            text: "---\ntitle: AST\nstatus: approved\napprovedBy: QA\n---\n# AST\nBusiness paragraph with **margin** and [source](https://example.com) [@doe2024] {@missing-ref}.\n\n> Quoted evidence\n> with continuation\n\n```js\nconst total = 42;\n```\n\n- First decision\n- Second decision\n\n- [x] Reviewed by finance\n- [ ] Attach signed approval\n\n| Metric | Value |\n| --- | ---: |\n| Total | =SUM(1,2) |\n\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:diagram caption=\"System diagram\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n{{page-break}}\n".to_string(),
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

    let exported = export::export_text(&response, &json!({}));
    assert!(exported.contains("> Quoted evidence\n> with continuation"));
    assert!(exported.contains("```js\nconst total = 42;\n```"));
    assert!(exported.contains("- First decision\n- Second decision"));
    assert!(exported.contains("- [x] Reviewed by finance\n- [ ] Attach signed approval"));
    assert!(exported.contains("Table: Metric | Value"));
    assert!(exported.contains("Figure: fig:diagram: System diagram"));
    assert!(exported.contains("Equation: eq:roi: ROI = Gain / Cost"));
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

mod transform_tests;

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

mod export_tests;

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
fn export_conformance_fixture_maps_business_features() {
    let response = compile(CompileRequest {
        text: include_str!("../fixtures/export/business_report.md").to_string(),
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

    let pdf = render_pdf_bytes(&response, &options);
    let pdf_text = String::from_utf8_lossy(&pdf);
    assert!(pdf.starts_with(b"%PDF-1.4"));
    assert!(pdf_text.contains("/Count 6"));
    assert!(pdf_text.contains("/Title (Export Conformance Report)"));
    assert!(pdf_text.contains("/Keywords (approved; 2.0.0; restricted)"));
    assert!(pdf_text.contains("Export Conformance Report | restricted"));
    assert!(pdf_text.contains("Page 6 of 6"));
    assert!(pdf_text.contains("Export Conformance Report"));
    assert!(pdf_text.contains("Competitive Advantage, p. 42"));
    assert!(pdf_text.contains(" re S"));
    assert!(pdf_text.contains("(Region) Tj"));
    assert!(pdf_text.contains("Reference architecture"));
    assert!(pdf_text.contains("Figure architecture"));
    assert!(pdf_text.contains("Equation roi"));
    assert!(pdf_text.contains("Glossary"));
    assert!(pdf_text.contains("Review Comments"));
    assert!(pdf_text.contains("Change Notes"));
    assert!(pdf_text.contains("AI Provenance"));

    let docx = render_docx_bytes(&response, &options).expect("docx bytes");
    let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
    let docx_document = zip_entry_text(&docx, "word/document.xml");
    let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
    let docx_header = zip_entry_text(&docx, "word/header1.xml");
    let docx_footer = zip_entry_text(&docx, "word/footer1.xml");
    let docx_comments = zip_entry_text(&docx, "word/comments.xml");
    let docx_app = zip_entry_text(&docx, "docProps/app.xml");
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
    assert!(docx_app.contains("<Application>NEditor</Application>"));
    assert!(docx_app.contains("<Company>Acme Strategy</Company>"));
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

    let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
    let pptx_content_types = zip_entry_text(&pptx, "[Content_Types].xml");
    let pptx_presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
    let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
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
    assert!(pptx_slide_three.contains("Page 3 of 9"));
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

    let exported_text = export::export_text(&response, &options);
    assert!(exported_text.contains("Glossary"));
    assert!(exported_text.contains("ARR: Annual recurring revenue"));
    assert!(exported_text.contains("Review Comments"));
    assert!(exported_text.contains("Change Notes"));
    assert!(exported_text.contains("AI Provenance"));

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

mod media_export_tests;
mod review_provenance_tests;

fn zip_entry_text(bytes: &[u8], path: &str) -> String {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor).expect("zip archive");
    let mut entry = archive.by_name(path).expect("zip entry");
    let mut text = String::new();
    entry.read_to_string(&mut text).expect("zip text");
    text
}

fn zip_has_entry(bytes: &[u8], path: &str) -> bool {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor).expect("zip archive");
    let result = archive.by_name(path).is_ok();
    result
}

fn zip_entry_count_with_prefix(bytes: &[u8], prefix: &str, suffix: &str) -> usize {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor).expect("zip archive");
    (0..archive.len())
        .filter(|index| {
            let entry = archive.by_index(*index).expect("zip entry by index");
            entry.name().starts_with(prefix) && entry.name().ends_with(suffix)
        })
        .count()
}

fn zip_entry_texts_with_prefix(bytes: &[u8], prefix: &str) -> Vec<String> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor).expect("zip archive");
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).expect("zip entry by index");
        if !entry.name().starts_with(prefix) || !entry.name().ends_with(".xml") {
            continue;
        }
        let mut text = String::new();
        entry.read_to_string(&mut text).expect("zip text");
        entries.push(text);
    }
    entries
}

mod export_command_tests;

#[test]
fn approved_documents_require_approval_metadata() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Approved\nstatus: approved\n---\n# Approved\n".to_string(),
        file_path: None,
    });

    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("missing approval metadata")));
}

#[test]
fn validation_requires_version_metadata() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Versioned\nstatus: approved\napprovedBy: QA\n---\n# Versioned\n"
            .to_string(),
        file_path: None,
    });

    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Missing version metadata."));
}

#[test]
fn validation_rejects_unknown_release_status() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Status\nversion: 1.0.0\nstatus: final\n---\n# Status\n".to_string(),
        file_path: None,
    });

    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "Invalid document status: final"));
}

#[test]
fn approved_documents_require_approval_timestamp() {
    let response = compile(CompileRequest {
            text: "---\ntitle: Approved\nversion: 1.0.0\nstatus: published\napprovedBy: QA\n---\n# Approved\n".to_string(),
            file_path: None,
        });

    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("missing approval metadata")));
}

#[test]
fn file_duplicate_and_rename_commands_move_content() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-file-test-{unique}"));
    fs::create_dir_all(&root).expect("create test dir");
    let source = root.join("source.md");
    let copy = root.join("copy.md");
    let renamed = root.join("renamed.md");
    fs::write(&source, "hello").expect("write source");

    let duplicated = duplicate_file(DuplicateFileRequest {
        from: path_to_string(&source),
        to: path_to_string(&copy),
    })
    .expect("duplicate file");
    assert_eq!(duplicated.text, "hello");

    let metadata = rename_file(RenameFileRequest {
        from: path_to_string(&copy),
        to: path_to_string(&renamed),
    })
    .expect("rename file");
    assert!(metadata.exists);
    assert!(renamed.exists());
    fs::remove_dir_all(root).expect("clean test dir");
}

#[test]
fn save_file_rejects_stale_expected_hash() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-save-conflict-test-{unique}"));
    fs::create_dir_all(&root).expect("create test dir");
    let doc = root.join("doc.md");
    fs::write(&doc, "external").expect("write external content");

    let result = save_file(SaveFileRequest {
        path: path_to_string(&doc),
        text: "local".to_string(),
        expected_hash: Some(sha256_hex(b"old")),
    });

    assert!(result
        .expect_err("stale save should fail")
        .contains("File changed on disk"));
    assert_eq!(fs::read_to_string(&doc).expect("read doc"), "external");
    fs::remove_dir_all(root).expect("clean save conflict test dir");
}

#[test]
fn stable_file_ipc_aliases_open_save_as_and_watch_paths() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-ipc-alias-test-{unique}"));
    fs::create_dir_all(root.join("chapters")).expect("create chapters");
    fs::create_dir_all(root.join("appendices")).expect("create appendices");
    let doc = root.join("doc.md");
    let included = root.join("chapters").join("intro.md");
    let nested = root.join("appendices").join("risk.md");
    let copy = root.join("copy.md");
    fs::write(&doc, "# Root\n!include chapters/intro.md").expect("write root");
    fs::write(&included, "# Intro\n{{include ../appendices/risk.md}}").expect("write include");
    fs::write(&nested, "# Risk").expect("write nested include");

    let opened = open_file(path_to_string(&doc)).expect("open file alias");
    assert!(opened.text.contains("# Root"));

    let saved = save_file_as(SaveFileRequest {
        path: path_to_string(&copy),
        text: "# Copy".to_string(),
        expected_hash: Some("stale-hash-ignored-for-save-as".to_string()),
    })
    .expect("save file as alias");
    assert_eq!(saved.text, "# Copy");

    let watched = watch_file(WatchFileRequest {
        root: path_to_string(&doc),
        included: vec![path_to_string(&included), path_to_string(&included)],
    })
    .expect("watch file command");
    assert_eq!(watched.paths.len(), 3);
    assert!(watched.paths.iter().all(|metadata| metadata.exists));
    assert_eq!(watched.paths[0].role, "root");
    assert_eq!(watched.paths[1].role, "include");
    assert!(watched
        .paths
        .iter()
        .any(|metadata| metadata.path.ends_with("chapters/intro.md")));
    assert!(watched
        .paths
        .iter()
        .any(|metadata| metadata.path.ends_with("appendices/risk.md")));
    fs::remove_dir_all(root).expect("clean ipc alias test dir");
}

#[cfg(feature = "native-watch")]
#[test]
fn notify_watcher_ignores_access_only_events() {
    assert!(!notify_event_should_emit(&notify::EventKind::Access(
        notify::event::AccessKind::Any
    )));
    assert!(notify_event_should_emit(&notify::EventKind::Modify(
        notify::event::ModifyKind::Any
    )));
}

#[test]
fn workspace_listing_skips_hidden_and_build_artifacts() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-workspace-test-{unique}"));
    fs::create_dir_all(root.join("chapters")).expect("create chapters");
    fs::create_dir_all(root.join(".git")).expect("create hidden dir");
    fs::create_dir_all(root.join("node_modules")).expect("create node modules");
    fs::write(root.join("root.md"), "# Root").expect("write root doc");
    fs::write(root.join("chapters").join("intro.md"), "# Intro").expect("write child doc");
    fs::write(root.join(".secret.md"), "# Secret").expect("write hidden file");
    fs::write(root.join("node_modules").join("package.md"), "# Dependency")
        .expect("write ignored dependency doc");
    fs::write(root.join("binary.bin"), [0, 1, 2, 3]).expect("write binary");

    let entries = list_workspace_files(WorkspaceFileRequest {
        root: path_to_string(&root),
    })
    .expect("workspace listing");
    let paths = entries
        .iter()
        .map(|entry| entry.relative_path.as_str())
        .collect::<Vec<_>>();

    assert!(paths.contains(&"root.md"));
    assert!(paths.contains(&"chapters"));
    assert!(paths.contains(&"chapters/intro.md"));
    assert!(!paths.iter().any(|path| path.contains(".secret")));
    assert!(!paths.iter().any(|path| path.contains("node_modules")));
    assert!(!paths.iter().any(|path| path.contains("binary.bin")));
    fs::remove_dir_all(root).expect("clean workspace test dir");
}

#[test]
fn git_history_diff_commit_tag_and_restore_workflow() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("neditor-git-test-{unique}"));
    fs::create_dir_all(&root).expect("create git test dir");
    run_git(&root, &["init"]).expect("git init");
    run_git(&root, &["config", "user.email", "neditor@example.test"]).expect("git email");
    run_git(&root, &["config", "user.name", "NEditor Test"]).expect("git name");
    let doc = root.join("doc.md");
    fs::write(&doc, "one\n").expect("write initial doc");
    run_git(&root, &["add", "doc.md"]).expect("git add");
    run_git(&root, &["commit", "-m", "Initial document"]).expect("git commit");
    fs::write(&doc, "two\n").expect("write changed doc");

    let diff = git_diff(GitPathRequest {
        path: path_to_string(&doc),
    })
    .expect("git diff");
    assert!(diff.contains("-one"));
    assert!(diff.contains("+two"));

    commit_document_changes(GitCommitRequest {
        path: path_to_string(&doc),
        message: "Update document".to_string(),
    })
    .expect("commit command");
    let history = git_history(GitPathRequest {
        path: path_to_string(&doc),
    })
    .expect("history command");
    assert!(history.len() >= 2);

    let tag = tag_release(GitTagRequest {
        path: path_to_string(&doc),
        tag: format!("test-{unique}"),
        message: "Test release".to_string(),
    })
    .expect("tag command");
    assert!(tag.starts_with("test-"));

    let restored = restore_git_revision(GitRestoreRequest {
        path: path_to_string(&doc),
        revision: history
            .last()
            .expect("initial history entry")
            .revision
            .clone(),
    })
    .expect("restore revision");
    assert_eq!(restored.text, "one\n");
    fs::remove_dir_all(root).expect("clean git test dir");
}

mod ai_cleanup_tests;
