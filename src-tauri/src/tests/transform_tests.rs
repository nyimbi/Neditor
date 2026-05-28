use super::*;

#[test]
fn transform_registry_covers_required_first_release_transforms() {
    let engines = list_transform_engines();
    let names = engines
        .iter()
        .filter_map(|engine| engine.get("name").and_then(Value::as_str))
        .collect::<BTreeSet<_>>();
    let pikchr = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("pikchr"))
        .expect("pikchr engine metadata");
    let vega_lite = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("vega-lite"))
        .expect("vega-lite engine metadata");
    let json_schema = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("json-schema"))
        .expect("json-schema engine metadata");
    let plantuml = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("plantuml"))
        .expect("plantuml engine metadata");
    let sql = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("sql"))
        .expect("sql engine metadata");
    assert_eq!(
        pikchr.get("trustRequired").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(pikchr.get("bundled").and_then(Value::as_bool), Some(false));
    assert!(pikchr
        .get("installationLabel")
        .and_then(Value::as_str)
        .is_some_and(|label| label.contains("not bundled")));
    assert!(pikchr
        .get("setupHint")
        .and_then(Value::as_str)
        .is_some_and(|hint| hint.contains("Pikchr executable")));
    assert!(pikchr
        .get("securitySummary")
        .and_then(Value::as_str)
        .is_some_and(|summary| summary.contains("no shell interpolation")));
    assert_eq!(
        pikchr.get("preferenceKey").and_then(Value::as_str),
        Some("transforms.pikchr.path")
    );
    assert!(pikchr
        .pointer("/diagnosticProfile/versionProbe")
        .and_then(Value::as_str)
        .is_some_and(|probe| probe.contains("pikchr --version")));
    assert!(pikchr
        .pointer("/diagnosticProfile/successRelated")
        .and_then(Value::as_array)
        .is_some_and(|fields| fields.iter().any(|field| field == "output_channel")));
    assert!(pikchr
        .pointer("/diagnosticProfile/failureHint")
        .and_then(Value::as_str)
        .is_some_and(|hint| hint.contains("Pikchr syntax")));
    assert!(pikchr
        .pointer("/diagnosticProfile/stderrHint")
        .and_then(Value::as_str)
        .is_some_and(|hint| hint.contains("Pikchr stderr")));
    assert_eq!(pikchr.get("output").and_then(Value::as_str), Some("svg"));
    assert_eq!(
        plantuml.get("output").and_then(Value::as_str),
        Some("svg-or-png")
    );
    assert_eq!(sql.get("output").and_then(Value::as_str), Some("table"));
    assert_eq!(
        sql.get("trustRequired").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        sql.get("defaultCommand").and_then(Value::as_str),
        Some("sqlite3")
    );
    assert!(sql
        .get("setupHint")
        .and_then(Value::as_str)
        .is_some_and(|hint| hint.contains("sqlite3") && hint.contains("read-only")));
    assert!(sql
        .pointer("/diagnosticProfile/versionProbe")
        .and_then(Value::as_str)
        .is_some_and(|probe| probe.contains("sqlite3 --version")));
    assert!(vega_lite
        .get("aliases")
        .and_then(Value::as_array)
        .is_some_and(|aliases| aliases.iter().any(|alias| alias == "vegalite")));
    assert_eq!(vega_lite.get("output").and_then(Value::as_str), Some("svg"));
    assert!(json_schema
        .get("aliases")
        .and_then(Value::as_array)
        .is_some_and(|aliases| aliases.iter().any(|alias| alias == "jsonschema")
            && aliases.iter().any(|alias| alias == "schema")));

    for name in [
        "calc",
        "mermaid",
        "pikchr",
        "dot",
        "graphviz",
        "circo",
        "neato",
        "fdp",
        "osage",
        "twopi",
        "plantuml",
        "d2",
        "vega-lite",
        "chart",
        "geojson",
        "topojson",
        "stl",
        "csv",
        "tsv",
        "sql",
        "json",
        "yaml",
        "openapi",
        "json-schema",
        "bibtex",
        "glossary",
        "layout",
        "timeline",
        "roadmap",
        "adr",
        "diff",
        "qr",
    ] {
        assert!(
            names.contains(name),
            "missing transform registry entry: {name}"
        );
        assert!(supported_transform(name), "unsupported transform: {name}");
    }
    for alias in ["vegalite", "jsonschema", "schema", "yml", "graph"] {
        assert!(
            supported_transform(alias),
            "unsupported transform alias: {alias}"
        );
    }

    let response = compile(CompileRequest {
        text: "---\ntitle: Diagram\n---\n# Diagram\n```pikchr\nbox \"A\"\narrow\nbox \"B\"\n```\n"
            .to_string(),
        file_path: None,
    });
    assert!(response.html.contains("transform-pikchr"));
    assert!(response.html.contains("pikchr-arrow"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Pikchr native preview")));
}

#[test]
fn transform_aliases_render_through_canonical_artifacts() {
    let vega_lite = run_transform(
            "vegalite".to_string(),
            r#"{"mark":"bar","title":"Alias Revenue","data":{"values":[{"region":"East","revenue":120}]},"encoding":{"x":{"field":"region","type":"nominal"},"y":{"field":"revenue","type":"quantitative"}}}"#.to_string(),
        )
        .expect("vegalite alias transform");
    assert_eq!(vega_lite.name, "vega-lite");
    assert_eq!(vega_lite.output_kind, "svg");
    assert!(vega_lite.html.contains("transform-vega-lite"));

    let schema = run_transform(
        "jsonschema".to_string(),
        r#"{"title":"Alias Schema","type":"object","properties":{"id":{"type":"string"}}}"#
            .to_string(),
    )
    .expect("jsonschema alias transform");
    assert_eq!(schema.name, "json-schema");
    assert_eq!(schema.output_kind, "html");
    assert!(schema.html.contains("Alias Schema"));

    let yaml =
        run_transform("yml".to_string(), "name: Alias\n".to_string()).expect("yml alias transform");
    assert_eq!(yaml.name, "yaml");
    assert!(yaml.html.contains("Alias"));

    let dot = run_transform("graph".to_string(), "digraph { a -> b }".to_string())
        .expect("graph alias transform");
    assert_eq!(dot.name, "dot");
    assert_eq!(dot.output_kind, "svg");
    assert!(dot.html.contains("transform-dot"));

    let response = compile(CompileRequest {
            text: "---\ntitle: Alias Fences\n---\n# Alias Fences\n```vegalite\n{\"mark\":\"bar\",\"title\":\"Fence Alias\",\"data\":{\"values\":[{\"region\":\"East\",\"revenue\":120}]},\"encoding\":{\"x\":{\"field\":\"region\"},\"y\":{\"field\":\"revenue\"}}}\n```\n"
                .to_string(),
            file_path: None,
        });
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "vega-lite"));
    assert!(response.html.contains("transform-vega-lite"));
}

#[test]
fn pikchr_native_fallback_handles_semicolon_shapes_and_arrow_labels() {
    let response = compile(CompileRequest {
        text: "---\ntitle: Pikchr Shapes\nstatus: approved\napprovedBy: QA\n---\n# Pikchr Shapes\n```pikchr\nbox \"Intake\"; arrow \"approve\"; diamond \"Gate\"; arrow; cylinder \"Store\"; arrow; file \"Export\"\n```\n"
            .to_string(),
        file_path: None,
    });

    assert!(response.html.contains("transform-pikchr"));
    assert!(response.html.contains("pikchr-box"));
    assert!(response.html.contains("pikchr-diamond"));
    assert!(response.html.contains("pikchr-cylinder"));
    assert!(response.html.contains("pikchr-file"));
    assert!(response.html.contains("approve"));
    assert!(response.html.contains("Intake"));
    assert!(response.html.contains("Export"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("Pikchr native preview")));
}

#[test]
fn external_diagram_fallbacks_render_simple_native_svgs() {
    for (name, body, expected) in [
        (
            "dot",
            "digraph { Start -> Review; Review -> Done; }",
            "transform-dot",
        ),
        (
            "graphviz",
            "digraph { a [label=\"Alpha\"]; a -> b; }",
            "Alpha",
        ),
        (
            "neato",
            "graph { Start -- Review; Review -- Done; }",
            "transform-neato",
        ),
        ("d2", "source -> target: request", "transform-d2"),
        (
            "plantuml",
            "@startuml\nAlice -> Bob: approve\n@enduml\n",
            "transform-plantuml",
        ),
    ] {
        let artifact = run_transform(name.to_string(), body.to_string())
            .unwrap_or_else(|err| panic!("{name} transform failed: {err}"));
        assert_eq!(artifact.output_kind, "svg", "{name} should render SVG");
        assert_eq!(artifact.execution_kind, "embedded");
        assert!(!artifact.html.contains("transform-pending"));
        assert!(artifact.html.contains(expected));
        assert!(artifact.diagnostics.is_empty());
    }
}

#[test]
fn d2_native_fallback_handles_labels_attributes_and_semicolons() {
    let artifact = run_transform(
        "d2".to_string(),
        r#"direction: right
customer: Customer
crm: CRM System {
  shape: rectangle
}
customer -> crm: submits RFP; crm <-> review: clarifies requirements
review: Review Board
review.shape: diamond
"#
        .to_string(),
    )
    .expect("d2 native fallback");

    assert_eq!(artifact.output_kind, "svg");
    assert_eq!(artifact.execution_kind, "embedded");
    assert!(artifact.html.contains("transform-d2"));
    assert!(artifact.html.contains("Customer"));
    assert!(artifact.html.contains("CRM System"));
    assert!(artifact.html.contains("Review Board"));
    assert!(artifact.html.contains("submits RFP"));
    assert!(artifact.html.contains("clarifies requirements"));
    assert!(!artifact.html.contains(">right<"));
    assert!(!artifact.html.contains(">rectangle<"));
    assert!(!artifact.html.contains(">diamond<"));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn document_ast_models_transform_artifacts_semantically() {
    let response = compile(CompileRequest {
        text: r#"---
title: Transform AST
status: approved
approvedBy: QA
---
# Transform AST

```roadmap
Q1: Launch beta
Q2: Expand exports
```

```timeline
2026-05-19: Semantic AST
```

```mermaid
flowchart LR
  A[Start] --> B[Done]
```
"#
        .to_string(),
        file_path: None,
    });

    let roadmap = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| match block {
            DocumentBlock::Transform {
                name,
                output_kind,
                text,
                html,
                source_hash,
                output_hash,
                cache_key,
                execution_kind,
                ..
            } if name == "roadmap" => Some((
                output_kind,
                text,
                html,
                source_hash,
                output_hash,
                cache_key,
                execution_kind,
            )),
            _ => None,
        })
        .expect("roadmap transform block");
    assert_eq!(roadmap.0, "html");
    assert!(roadmap.1.contains("Launch beta"));
    assert!(roadmap.2.contains("transform-roadmap"));
    assert!(roadmap.3.as_deref().is_some_and(|hash| hash.len() == 64));
    assert!(roadmap.4.as_deref().is_some_and(|hash| hash.len() == 64));
    assert!(roadmap.5.as_deref().is_some_and(|key| key.len() == 64));
    assert_eq!(roadmap.6.as_deref(), Some("embedded"));

    let timeline = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| match block {
            DocumentBlock::Transform {
                name,
                output_kind,
                text,
                ..
            } if name == "timeline" => Some((output_kind, text)),
            _ => None,
        })
        .expect("timeline transform block");
    assert_eq!(timeline.0, "svg");
    assert!(timeline.1.contains("Semantic AST"));

    let mermaid = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| match block {
            DocumentBlock::Transform {
                name,
                output_kind,
                text,
                html,
                ..
            } if name == "mermaid" => Some((output_kind, text, html)),
            _ => None,
        })
        .expect("mermaid transform block");
    assert_eq!(mermaid.0, "svg");
    assert!(mermaid.1.contains("Start"));
    assert!(mermaid.2.contains("transform-mermaid"));

    let roadmap_artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "roadmap")
        .expect("roadmap transform artifact");
    assert_eq!(roadmap_artifact.source_file.as_deref(), Some("untitled.md"));
    assert_eq!(roadmap_artifact.source_line, Some(8));
    assert_eq!(roadmap_artifact.end_source_line, Some(11));
    let mermaid_artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "mermaid")
        .expect("mermaid transform artifact");
    assert_eq!(mermaid_artifact.source_line, Some(17));
    assert_eq!(mermaid_artifact.end_source_line, Some(20));

    let exported = export::export_text(&response, &json!({}));
    assert!(exported.contains("Transform: roadmap"));
    assert!(exported.contains("Transform: mermaid"));

    let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
        .expect("transform metadata bundle");
    let bundled_ast: Value = serde_json::from_str(&zip_entry_text(&bundle, "document-ast.json"))
        .expect("document ast json");
    let bundled_roadmap = bundled_ast
        .get("blocks")
        .and_then(Value::as_array)
        .and_then(|blocks| {
            blocks.iter().find(|block| {
                block.get("kind").and_then(Value::as_str) == Some("transform")
                    && block.get("name").and_then(Value::as_str) == Some("roadmap")
            })
        })
        .expect("bundled roadmap transform block");
    assert_eq!(
        bundled_roadmap.get("output_kind").and_then(Value::as_str),
        Some("html")
    );
    assert_eq!(
        bundled_roadmap
            .get("execution_kind")
            .and_then(Value::as_str),
        Some("embedded")
    );
    assert_eq!(
        bundled_roadmap.get("output_hash").and_then(Value::as_str),
        Some(roadmap_artifact.output_hash.as_str())
    );
    assert!(bundled_roadmap
        .get("cache_key")
        .and_then(Value::as_str)
        .is_some_and(|key| key.len() == 64));

    let bundled_artifacts: Value =
        serde_json::from_str(&zip_entry_text(&bundle, "transform-artifacts.json"))
            .expect("transform artifacts json");
    let bundled_mermaid = bundled_artifacts
        .as_array()
        .and_then(|artifacts| {
            artifacts
                .iter()
                .find(|artifact| artifact.get("name").and_then(Value::as_str) == Some("mermaid"))
        })
        .expect("bundled mermaid artifact");
    assert_eq!(
        bundled_mermaid.get("output_kind").and_then(Value::as_str),
        Some("svg")
    );
    assert_eq!(
        bundled_mermaid.get("source_line").and_then(Value::as_u64),
        mermaid_artifact.source_line.map(|line| line as u64)
    );
    assert_eq!(
        bundled_mermaid
            .get("end_source_line")
            .and_then(Value::as_u64),
        mermaid_artifact.end_source_line.map(|line| line as u64)
    );
    assert_eq!(
        bundled_mermaid.get("output_hash").and_then(Value::as_str),
        Some(mermaid_artifact.output_hash.as_str())
    );
}

#[test]
fn transform_diagnostics_resolve_to_source_fence_ranges() {
    let response = compile(CompileRequest {
        text: r#"---
title: Bad Transform
---
# Bad Transform

```json
{bad
```
"#
        .to_string(),
        file_path: None,
    });

    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("Invalid JSON transform input"))
        .expect("json transform diagnostic");
    assert_eq!(diagnostic.source_file.as_deref(), Some("untitled.md"));
    assert_eq!(diagnostic.line, Some(6));
    assert_eq!(diagnostic.end_line, Some(8));
    assert_eq!(diagnostic.column, Some(1));
    assert_eq!(diagnostic.end_column, Some(4));
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related == "transform: json"));
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related == "source range: 6-8"));

    let artifact_diagnostic = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "json")
        .and_then(|artifact| artifact.diagnostics.first())
        .expect("json artifact diagnostic");
    assert_eq!(
        artifact_diagnostic.source_file.as_deref(),
        Some("untitled.md")
    );
    assert_eq!(artifact_diagnostic.line, Some(6));
    assert_eq!(artifact_diagnostic.end_line, Some(8));
    assert_eq!(artifact_diagnostic.column, Some(1));
    assert_eq!(artifact_diagnostic.end_column, Some(4));
    assert!(artifact_diagnostic
        .related
        .iter()
        .any(|related| related == "transform: json"));
    assert!(artifact_diagnostic
        .related
        .iter()
        .any(|related| related == "source range: 6-8"));
}

#[test]
fn unknown_transform_attempts_report_source_ranged_diagnostics() {
    let response = compile(CompileRequest {
        text: "# Unknown Transform\n\n```notebook output=html\nvalue: 42\n```\n\n```python\nprint(42)\n```\n"
            .to_string(),
        file_path: None,
    });

    let diagnostic = response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message == "Unknown fenced transform: notebook")
        .expect("unknown transform diagnostic");
    assert_eq!(diagnostic.severity, "warning");
    assert_eq!(diagnostic.source_file.as_deref(), Some("untitled.md"));
    assert_eq!(diagnostic.line, Some(3));
    assert_eq!(diagnostic.end_line, Some(3));
    assert_eq!(diagnostic.column, Some(4));
    assert_eq!(diagnostic.end_column, Some(12));
    assert!(diagnostic
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion.contains("supported transform name")));
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related == "transform: notebook"));
    assert!(diagnostic
        .related
        .iter()
        .any(|related| related == "fence info: notebook output=html"));
    assert!(!response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("python")));
    assert!(response
        .compiled_markdown
        .contains("```notebook output=html"));
    assert!(response.compiled_markdown.contains("```python"));
}

#[test]
fn document_ast_parses_multiline_semantic_html_blocks() {
    let response = compile(CompileRequest {
        text: r#"---
title: Multiline HTML AST
---
# Multiline HTML AST

<figure class="figure" id="fig:multi">
<img src="diagram.svg" alt="Diagram">
<figcaption>Multiline caption</figcaption>
</figure>

<section class="transform transform-custom">
<pre>alpha
beta</pre>
</section>
"#
        .to_string(),
        file_path: None,
    });

    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Figure { id, caption, line, end_line, .. }
                if id.as_deref() == Some("fig:multi")
                    && caption.as_deref() == Some("Multiline caption")
                    && *end_line > *line
        )
    }));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Transform { name, text, line, end_line, .. }
                if name == "custom"
                    && text.contains("alpha")
                    && text.contains("beta")
                    && *end_line > *line
        )
    }));
}

#[test]
fn qr_transform_renders_static_svg_preview() {
    let artifact =
        run_transform("qr".to_string(), "https://example.com".to_string()).expect("qr transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-qr"));
    assert!(artifact.html.contains("<rect"));
    assert!(artifact.html.contains("QR code for https://example.com"));
    assert!(artifact.diagnostics.is_empty());

    let response = compile(CompileRequest {
        text: "---\ntitle: QR\n---\n# QR\n```qr\nhttps://example.com\n```\n".to_string(),
        file_path: None,
    });
    assert!(response.html.contains("transform-qr"));
    assert!(response
        .transform_artifacts
        .iter()
        .any(|artifact| artifact.name == "qr" && artifact.output_kind == "svg"));
}

#[test]
fn qr_matrix_reserves_finder_separators() {
    let matrix = transforms::qr::render_qr_matrix(b"HELLO").expect("qr matrix");
    assert_eq!(matrix.len(), 21);

    for (row, cells) in matrix.iter().enumerate().take(8) {
        assert!(!cells[13], "top-right finder separator row {row}");
    }
    for (column, cell) in matrix[13].iter().enumerate().take(8) {
        assert!(!cell, "bottom-left finder separator column {column}");
    }
    assert!(transforms::qr::render_qr_matrix(&[b'x'; 79]).is_err());
}

#[test]
fn bibtex_transform_renders_bibliography_preview() {
    let artifact = run_transform(
        "bibtex".to_string(),
        "@book{porter1985, title={Competitive Advantage}, author={Michael Porter}, year={1985}, publisher={Free Press}, edition={2}, isbn={978-0-684-84146-5}}\n@article{doe2026, title=\"Evidence Based Reports\", author=\"Jane Doe\", date=\"2026-05-21\", journal=\"Business Evidence Review\", volume={4}, number={2}, pages={10--18}, doi={10.1000/example}, url={https://example.test/evidence}, issn={2049-3630}, abstract={Practical evidence review for business documents.}}".to_string(),
    )
    .expect("bibtex transform");

    assert_eq!(artifact.output_kind, "html");
    assert!(artifact.html.contains("transform-bibtex"));
    assert!(artifact.html.contains("<dt>porter1985</dt>"));
    assert!(artifact.html.contains("<cite>Competitive Advantage</cite>"));
    assert!(artifact.html.contains("Michael Porter"));
    assert!(artifact.html.contains("1985"));
    assert!(artifact.html.contains("Publisher"));
    assert!(artifact.html.contains("Free Press"));
    assert!(artifact.html.contains("Edition"));
    assert!(artifact.html.contains("978-0-684-84146-5"));
    assert!(artifact.html.contains("<dt>doe2026</dt>"));
    assert!(artifact
        .html
        .contains("<cite>Evidence Based Reports</cite>"));
    assert!(artifact.html.contains("Jane Doe"));
    assert!(artifact.html.contains("2026"));
    assert!(artifact.html.contains("Business Evidence Review"));
    assert!(artifact.html.contains("10.1000/example"));
    assert!(artifact.html.contains("https://example.test/evidence"));
    assert!(artifact.html.contains("2049-3630"));
    assert!(artifact
        .html
        .contains("Practical evidence review for business documents."));
    assert!(artifact.html.contains("bibtex-entry-metadata"));
    assert!(artifact.diagnostics.is_empty());

    let engines = list_transform_engines();
    let bibtex = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("bibtex"))
        .expect("bibtex engine metadata");
    assert_eq!(
        bibtex.get("execution").and_then(Value::as_str),
        Some("rust-native")
    );
}

#[test]
fn structured_data_transforms_render_tables_and_trees() {
    let json_artifact = run_transform(
        "json".to_string(),
        r#"[{"region":"East","revenue":120},{"region":"West","revenue":98}]"#.to_string(),
    )
    .expect("json transform");
    assert_eq!(json_artifact.output_kind, "html");
    assert!(json_artifact.html.contains("transform-json"));
    assert!(json_artifact.html.contains("<th>region</th>"));
    assert!(json_artifact.html.contains("<td>East</td>"));
    assert!(json_artifact.diagnostics.is_empty());

    let yaml_artifact = run_transform(
        "yaml".to_string(),
        "api:\n  version: v1\n  endpoints:\n    - /accounts\n".to_string(),
    )
    .expect("yaml transform");
    assert_eq!(yaml_artifact.output_kind, "html");
    assert!(yaml_artifact.html.contains("structured-tree"));
    assert!(yaml_artifact.html.contains("<dt>version</dt>"));
    assert!(yaml_artifact.html.contains("/accounts"));
    assert!(yaml_artifact.diagnostics.is_empty());

    let nested_json_artifact = run_transform(
        "json".to_string(),
        r#"{"data":[{"region":"East","revenue":120},{"region":"West","revenue":98}],"generatedAt":"2026-05-27"}"#.to_string(),
    )
    .expect("nested json table transform");
    assert!(nested_json_artifact.html.contains("transform-json"));
    assert!(nested_json_artifact
        .html
        .contains("<caption>data</caption>"));
    assert!(nested_json_artifact.html.contains("<th>revenue</th>"));
    assert!(nested_json_artifact.html.contains("<td>98</td>"));
    assert!(nested_json_artifact.diagnostics.is_empty());

    let nested_yaml_artifact = run_transform(
        "yaml".to_string(),
        "records:\n  - account: Acme\n    status: active\n  - account: Beta\n    status: review\n"
            .to_string(),
    )
    .expect("nested yaml table transform");
    assert!(nested_yaml_artifact.html.contains("transform-yaml"));
    assert!(nested_yaml_artifact
        .html
        .contains("<caption>records</caption>"));
    assert!(nested_yaml_artifact.html.contains("<td>Beta</td>"));
    assert!(nested_yaml_artifact.diagnostics.is_empty());
}

#[test]
fn structured_data_tables_flatten_nested_business_rows() {
    let json_artifact = run_transform(
        "json".to_string(),
        r#"{"records":[{"account":{"name":"Acme","owner":"Mina"},"metrics":{"revenue":120,"margin":0.42},"tags":["strategic","renewal"],"risks":[]},{"account":{"name":"Beta","owner":"Sam"},"metrics":{"revenue":98,"margin":0.31},"tags":["watch"],"risks":[]}]}"#.to_string(),
    )
    .expect("nested business json transform");

    assert_eq!(json_artifact.output_kind, "html");
    assert!(json_artifact.html.contains("transform-json"));
    assert!(json_artifact.html.contains("<caption>records</caption>"));
    assert!(json_artifact.html.contains("<th>account.name</th>"));
    assert!(json_artifact.html.contains("<th>metrics.revenue</th>"));
    assert!(json_artifact.html.contains("<td>Acme</td>"));
    assert!(json_artifact.html.contains("<td>120</td>"));
    assert!(json_artifact.html.contains("<td>strategic, renewal</td>"));
    assert!(json_artifact.html.contains("<td>[]</td>"));
    assert!(json_artifact.diagnostics.is_empty());

    let yaml_artifact = run_transform(
        "yaml".to_string(),
        "items:\n  - customer:\n      name: Contoso\n      tier: enterprise\n    contract:\n      renewal: 2026-07-01\n      value: 250000\n  - customer:\n      name: Fabrikam\n      tier: growth\n    contract:\n      renewal: 2026-09-15\n      value: 175000\n".to_string(),
    )
    .expect("nested business yaml transform");

    assert_eq!(yaml_artifact.output_kind, "html");
    assert!(yaml_artifact.html.contains("transform-yaml"));
    assert!(yaml_artifact.html.contains("<caption>items</caption>"));
    assert!(yaml_artifact.html.contains("<th>contract.renewal</th>"));
    assert!(yaml_artifact.html.contains("<th>customer.tier</th>"));
    assert!(yaml_artifact.html.contains("<td>enterprise</td>"));
    assert!(yaml_artifact.html.contains("<td>175000</td>"));
    assert!(yaml_artifact.diagnostics.is_empty());
}

#[test]
fn structured_data_tables_render_keyed_maps_and_scalar_settings() {
    let json_artifact = run_transform(
        "json".to_string(),
        r#"{"accounts":{"acme":{"owner":"Mina","metrics":{"mrr":42000},"tags":["strategic","renewal"]},"beta":{"owner":"Sam","metrics":{"mrr":18000},"tags":["watch"]}}}"#.to_string(),
    )
    .expect("keyed json map transform");

    assert_eq!(json_artifact.output_kind, "html");
    assert!(json_artifact.html.contains("transform-json"));
    assert!(json_artifact.html.contains("<caption>accounts</caption>"));
    assert!(json_artifact.html.contains("<th>key</th>"));
    assert!(json_artifact.html.contains("<th>metrics.mrr</th>"));
    assert!(json_artifact.html.contains("<td>acme</td>"));
    assert!(json_artifact.html.contains("<td>42000</td>"));
    assert!(json_artifact.html.contains("<td>strategic, renewal</td>"));
    assert!(json_artifact.diagnostics.is_empty());

    let yaml_artifact = run_transform(
        "yaml".to_string(),
        "name: Launch Plan\nstatus: approved\naudiences:\n  - legal\n  - finance\n".to_string(),
    )
    .expect("scalar settings yaml transform");

    assert_eq!(yaml_artifact.output_kind, "html");
    assert!(yaml_artifact.html.contains("transform-yaml"));
    assert!(yaml_artifact.html.contains("<caption>fields</caption>"));
    assert!(yaml_artifact.html.contains("<th>key</th>"));
    assert!(yaml_artifact.html.contains("<th>value</th>"));
    assert!(yaml_artifact.html.contains("<td>audiences</td>"));
    assert!(yaml_artifact.html.contains("<td>legal, finance</td>"));
    assert!(yaml_artifact.html.contains("<td>Launch Plan</td>"));
    assert!(yaml_artifact.diagnostics.is_empty());
}

#[test]
fn chart_transform_renders_yaml_business_chart_specs() {
    let artifact = run_transform(
            "chart".to_string(),
            "type: bar\ntitle: Revenue by Region\ndata:\n  - region: East\n    revenue: 120\n  - region: West\n    revenue: 98\nx: region\ny: revenue\n".to_string(),
        )
        .expect("chart transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-chart"));
    assert!(artifact.html.contains("Revenue by Region"));
    assert!(artifact.html.contains(">East<"));
    assert!(artifact.html.contains(">120<"));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn chart_transform_renders_pie_area_and_kpi_specs() {
    let pie = run_transform(
            "chart".to_string(),
            "type: pie\ntitle: Revenue Mix\ndata:\n  - segment: Services\n    revenue: 120\n  - segment: Software\n    revenue: 80\nx: segment\ny: revenue\n".to_string(),
        )
        .expect("pie chart transform");
    assert_eq!(pie.output_kind, "svg");
    assert!(pie.html.contains("Revenue Mix"));
    assert!(pie.html.contains("<path d=\"M 260.0 154.0"));
    assert!(pie.html.contains("Services"));
    assert!(pie.html.contains("(60.0%)"));

    let area = run_transform(
            "chart".to_string(),
            "type: area\ntitle: Pipeline\ndata:\n  - month: May\n    value: 20\n  - month: Jun\n    value: 45\nx: month\ny: value\n".to_string(),
        )
        .expect("area chart transform");
    assert_eq!(area.output_kind, "svg");
    assert!(area.html.contains("<polygon"));
    assert!(area.html.contains("<polyline"));
    assert!(area.html.contains(">Jun<"));

    let kpi = run_transform(
            "chart".to_string(),
            "type: kpi\ntitle: Board KPI\ndata:\n  - metric: NDR\n    value: 118\n  - metric: Target\n    value: 110\nx: metric\ny: value\n".to_string(),
        )
        .expect("kpi chart transform");
    assert_eq!(kpi.output_kind, "svg");
    assert!(kpi.html.contains("Board KPI"));
    assert!(kpi.html.contains(">NDR<"));
    assert!(kpi.html.contains(">118<"));
    assert!(kpi.html.contains("Target: 110"));
}

#[test]
fn chart_transform_handles_negative_values_targets_and_value_units() {
    let artifact = run_transform(
            "chart".to_string(),
            "type: bar\ntitle: Profit Variance\ntarget: 25\ntargetLabel: Plan\nvaluePrefix: $\nvalueSuffix: k\ndata:\n  - quarter: Q1\n    profit: 42\n  - quarter: Q2\n    profit: -18\n  - quarter: Q3\n    profit: 30\nx: quarter\ny: profit\n".to_string(),
        )
        .expect("variance chart transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("Profit Variance"));
    assert!(artifact.html.contains("chart-target-line"));
    assert!(artifact.html.contains("Plan: $25k"));
    assert!(artifact.html.contains(">$42k<"));
    assert!(artifact.html.contains(">$-18k<"));
    assert!(artifact.html.contains("fill=\"#be123c\""));
    assert!(artifact.html.contains(">Q2<"));
    assert!(artifact.diagnostics.is_empty());

    let area = run_transform(
            "chart".to_string(),
            "type: area\ntitle: Cash Flow\ntarget: 6\ngoalLabel: Break-even\nunit: m\ndata:\n  - month: Jan\n    value: -4\n  - month: Feb\n    value: 8\nx: month\ny: value\n".to_string(),
        )
        .expect("cash-flow area chart transform");
    assert_eq!(area.output_kind, "svg");
    assert!(area.html.contains("Cash Flow"));
    assert!(area.html.contains(">-4m<"));
    assert!(area.html.contains(">8m<"));
    assert!(area.html.contains("Break-even: 6m"));
    assert!(area.html.contains("<polygon"));
    assert!(area.diagnostics.is_empty());
}

#[test]
fn chart_transform_renders_multi_series_business_specs() {
    let grouped = run_transform(
            "chart".to_string(),
            "type: bar\ntitle: Budget vs Actual\nunit: k\ndata:\n  - month: Jan\n    budget: 100\n    actual: 92\n  - month: Feb\n    budget: 110\n    actual: 118\nx: month\nseries:\n  - key: budget\n    label: Budget\n  - key: actual\n    label: Actual\n".to_string(),
        )
        .expect("grouped business chart transform");

    assert_eq!(grouped.output_kind, "svg");
    assert!(grouped.html.contains("Budget vs Actual"));
    assert!(grouped.html.contains("data-series=\"Budget\""));
    assert!(grouped.html.contains("data-series=\"Actual\""));
    assert!(grouped.html.contains("chart-legend-item"));
    assert!(grouped.html.contains(">100k<"));
    assert!(grouped.html.contains(">118k<"));
    assert!(grouped.html.contains(">Feb<"));
    assert!(grouped.diagnostics.is_empty());

    let line = run_transform(
            "chart".to_string(),
            "type: line\ntitle: Segment Growth\nvalueSuffix: \"%\"\ndata:\n  - quarter: Q1\n    enterprise: 12\n    smb: 8\n  - quarter: Q2\n    enterprise: 18\n    smb: 11\nx: quarter\nseries:\n  - enterprise\n  - smb\n".to_string(),
        )
        .expect("multi-series line chart transform");
    assert_eq!(line.output_kind, "svg");
    assert!(line.html.contains("Segment Growth"));
    assert!(line.html.contains("data-series=\"enterprise\""));
    assert!(line.html.contains("data-series=\"smb\""));
    assert!(line.html.contains(">18%<"));
    assert!(line.html.contains(">11%<"));
    assert!(line.html.contains("<polyline"));
    assert!(line.diagnostics.is_empty());
}

#[test]
fn chart_transform_renders_horizontal_business_comparisons() {
    let ranked = run_transform(
            "chart".to_string(),
            "type: horizontal-bar\ntitle: Renewal Risk by Account\ntarget: 40\ntargetLabel: Escalation\nvalueSuffix: \"%\"\ndata:\n  - account: Very Long Enterprise Account Name\n    risk: 72\n  - account: Growth Segment\n    risk: -12\nx: account\ny: risk\n".to_string(),
        )
        .expect("horizontal chart transform");

    assert_eq!(ranked.output_kind, "svg");
    assert!(ranked.html.contains("Renewal Risk by Account"));
    assert!(ranked.html.contains("chart-horizontal-bar"));
    assert!(ranked.html.contains("chart-horizontal-label"));
    assert!(ranked.html.contains("Very Long Enterprise Account Name"));
    assert!(ranked.html.contains(">72%<"));
    assert!(ranked.html.contains(">-12%<"));
    assert!(ranked.html.contains("fill=\"#be123c\""));
    assert!(ranked.html.contains("chart-target-vertical-line"));
    assert!(ranked.html.contains("Escalation: 40%"));
    assert!(ranked.diagnostics.is_empty());

    let grouped = run_transform(
            "chart".to_string(),
            "type: barh\ntitle: Proposal Scorecard\nunit: pts\ndata:\n  - criterion: Technical fit\n    incumbent: 62\n    challenger: 84\n  - criterion: Implementation risk\n    incumbent: 38\n    challenger: 24\nx: criterion\nseries:\n  - key: incumbent\n    label: Incumbent\n  - key: challenger\n    label: Challenger\n".to_string(),
        )
        .expect("grouped horizontal chart transform");
    assert_eq!(grouped.output_kind, "svg");
    assert!(grouped.html.contains("Proposal Scorecard"));
    assert!(grouped.html.contains("data-series=\"Incumbent\""));
    assert!(grouped.html.contains("data-series=\"Challenger\""));
    assert!(grouped.html.contains("chart-legend-item"));
    assert!(grouped.html.contains(">84pts<"));
    assert!(grouped.html.contains("Implementation risk"));
    assert!(grouped.diagnostics.is_empty());
}

#[test]
fn timeline_transform_renders_static_svg_preview() {
    let artifact = run_transform(
        "timeline".to_string(),
        "2026-05-18: Kickoff\n2026-06-01: Review\n2026-06-15: Release\n".to_string(),
    )
    .expect("timeline transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-timeline"));
    assert!(artifact.html.contains("Kickoff"));
    assert!(artifact.html.contains("Release"));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn business_workflow_transforms_render_static_html() {
    let roadmap = run_transform(
        "roadmap".to_string(),
        "Now: Drafting | status=active | owner=Docs\nNext: Review | due=2026-06-01\nLater: Publish"
            .to_string(),
    )
    .expect("roadmap transform");
    assert_eq!(roadmap.output_kind, "html");
    assert!(roadmap.html.contains("transform-roadmap"));
    assert!(roadmap.html.contains("roadmap-item"));
    assert!(roadmap.html.contains("roadmap-meta-status"));
    assert!(roadmap.html.contains("active"));
    assert!(roadmap.html.contains("Review"));

    let adr = run_transform(
        "adr".to_string(),
        "Status: accepted\nDecision: Use local-first exports".to_string(),
    )
    .expect("adr transform");
    assert_eq!(adr.output_kind, "html");
    assert!(adr.html.contains("transform-adr"));
    assert!(adr.html.contains("adr-status"));
    assert!(adr.html.contains("adr-decision"));
    assert!(adr.html.contains("Use local-first exports"));

    let diff = run_transform("diff".to_string(), "@@ -1 +1 @@\n-old\n+new".to_string())
        .expect("diff transform");
    assert_eq!(diff.output_kind, "html");
    assert!(diff.html.contains("transform-diff"));
    assert!(diff.html.contains("1 additions / 1 deletions / 1 hunks"));
    assert!(diff.html.contains("diff-del"));
    assert!(diff.html.contains("diff-add"));
}

#[test]
fn geojson_transform_renders_static_svg_preview() {
    let artifact = run_transform(
            "geojson".to_string(),
            r#"{"type":"Feature","geometry":{"type":"LineString","coordinates":[[36.80,-1.30],[36.85,-1.26],[36.90,-1.28]]}}"#.to_string(),
        )
        .expect("geojson transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-geojson"));
    assert!(artifact.html.contains("<polyline"));
    assert!(artifact.html.contains("3 coordinates"));
    assert!(artifact.diagnostics.is_empty());

    let engines = list_transform_engines();
    let geojson = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("geojson"))
        .expect("geojson engine metadata");
    assert_eq!(
        geojson.get("execution").and_then(Value::as_str),
        Some("rust-native-svg")
    );
}

#[test]
fn geojson_transform_preserves_geometry_types_in_static_svg_preview() {
    let artifact = run_transform(
            "geojson".to_string(),
            r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"name":"District"},"geometry":{"type":"Polygon","coordinates":[[[36.80,-1.30],[36.86,-1.30],[36.86,-1.24],[36.80,-1.24],[36.80,-1.30]]]}},{"type":"Feature","geometry":{"type":"MultiPoint","coordinates":[[36.81,-1.29],[36.84,-1.26]]}}]}"#.to_string(),
        )
        .expect("geojson feature collection transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-geojson"));
    assert!(artifact.html.contains("<polygon"));
    assert!(artifact.html.contains("<circle"));
    assert!(artifact.html.contains("1 polygons"));
    assert!(artifact.html.contains("2 points"));
    assert!(artifact.html.contains("7 coordinates"));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn geojson_transform_warns_for_projection_assumptions() {
    let artifact = run_transform(
            "geojson".to_string(),
            r#"{"type":"FeatureCollection","crs":{"type":"name","properties":{"name":"EPSG:3857"}},"features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[4088910,-144715]}}]}"#.to_string(),
        )
        .expect("geojson projected transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-geojson"));
    assert!(artifact
        .html
        .contains("data-projection=\"linear-wgs84-fit\""));
    assert!(artifact
        .html
        .contains("data-coordinate-assumption=\"longitude-latitude\""));
    assert!(artifact.html.contains("<circle"));
    assert!(artifact
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("ignores legacy crs metadata")));
    assert!(artifact
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("outside normal WGS84")));
}

#[test]
fn topojson_transform_renders_static_svg_preview() {
    let artifact = run_transform(
            "topojson".to_string(),
            r#"{"type":"Topology","transform":{"scale":[0.01,0.01],"translate":[36.8,-1.3]},"objects":{},"arcs":[[[0,0],[5,4],[5,-2]]]}"#.to_string(),
        )
        .expect("topojson transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-topojson"));
    assert!(artifact.html.contains("<polyline"));
    assert!(artifact.html.contains("1 lines"));
    assert!(artifact.html.contains("3 coordinates"));
    assert!(artifact.diagnostics.is_empty());

    let engines = list_transform_engines();
    let topojson = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("topojson"))
        .expect("topojson engine metadata");
    assert_eq!(
        topojson.get("execution").and_then(Value::as_str),
        Some("rust-native-svg")
    );
}

#[test]
fn topojson_transform_resolves_object_arc_references() {
    let artifact = run_transform(
            "topojson".to_string(),
            r#"{"type":"Topology","transform":{"scale":[0.01,0.01],"translate":[36.8,-1.3]},"objects":{"district":{"type":"Polygon","arcs":[[0,-2]]}},"arcs":[[[0,0],[6,0],[0,6]],[[0,0],[0,6],[6,0]]]}"#.to_string(),
        )
        .expect("topojson object transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-topojson"));
    assert!(artifact.html.contains("<polygon"));
    assert!(artifact.html.contains("1 polygons"));
    assert!(artifact.html.contains("5 coordinates"));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn topojson_transform_applies_quantized_point_geometry_transform() {
    let artifact = run_transform(
            "topojson".to_string(),
            r#"{"type":"Topology","transform":{"scale":[0.1,0.1],"translate":[100,50]},"objects":{"route":{"type":"GeometryCollection","geometries":[{"type":"LineString","arcs":[0]},{"type":"Point","coordinates":[10,10]},{"type":"MultiPoint","coordinates":[[0,0],[5,5]]}]}},"arcs":[[[0,0],[10,0]]]}"#.to_string(),
        )
        .expect("topojson point transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-topojson"));
    assert!(artifact.html.contains("<polyline"));
    assert!(artifact.html.contains("<circle"));
    assert!(artifact.html.contains("1 lines / 3 points / 5 coordinates"));
    assert!(artifact.html.contains("<circle cx=\"852.00\" cy=\"48.00\""));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn stl_transform_renders_ascii_static_svg_preview() {
    let artifact = run_transform(
            "stl".to_string(),
            "solid test\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 10 0 0\nvertex 0 10 0\nendloop\nendfacet\nendsolid test".to_string(),
        )
        .expect("stl transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-stl"));
    assert!(artifact.html.contains("<polygon"));
    assert!(artifact.html.contains("1 triangles / 3 vertices"));
    assert!(artifact.diagnostics.is_empty());

    let engines = list_transform_engines();
    let stl = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("stl"))
        .expect("stl engine metadata");
    assert_eq!(
        stl.get("execution").and_then(Value::as_str),
        Some("rust-native-svg")
    );
}

#[test]
fn stl_transform_renders_depth_aware_isometric_preview() {
    let artifact = run_transform(
        "stl".to_string(),
        "solid depth
facet normal 0 0 1
outer loop
vertex 0 0 0
vertex 10 0 0
vertex 0 10 0
endloop
endfacet
facet normal 0 0 1
outer loop
vertex 0 0 10
vertex 10 0 10
vertex 0 10 10
endloop
endfacet
endsolid depth"
            .to_string(),
    )
    .expect("depth-aware stl transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-stl"));
    assert!(artifact
        .html
        .contains("data-projection=\"isometric-depth-fit\""));
    assert!(artifact
        .html
        .contains("data-coordinate-assumption=\"ascii-stl-xyz\""));
    assert!(artifact
        .html
        .contains("2 triangles / 6 vertices / z-depth 10"));
    assert!(artifact.html.contains("data-depth=\"0.00\""));
    assert!(artifact.html.contains("data-depth=\"10.00\""));
    assert!(artifact.html.contains("rgba(39,93,168,0.18)"));
    assert!(artifact.html.contains("rgba(39,93,168,0.36)"));
    assert!(artifact.html.find("data-depth=\"0.00\"") < artifact.html.find("data-depth=\"10.00\""));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn stl_transform_renders_base64_binary_static_svg_preview() {
    let artifact = run_transform(
        "stl".to_string(),
        "data:application/sla;base64,TkVkaXRvciBiaW5hcnkgU1RMIHRlc3QgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICACAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAgD8AAIA/AAAAAAAAgD8AAAAAAACAPwAAgD8AAA==".to_string(),
    )
    .expect("binary stl transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-stl"));
    assert!(artifact.html.contains("data-stl-source=\"binary-base64\""));
    assert!(artifact
        .html
        .contains("data-coordinate-assumption=\"binary-stl-base64-xyz\""));
    assert!(artifact
        .html
        .contains("2 triangles / 6 vertices / z-depth 1"));
    assert!(artifact.html.contains("data-depth=\"0.00\""));
    assert!(artifact.html.contains("data-depth=\"1.00\""));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn vega_lite_transform_renders_static_svg_preview() {
    let artifact = run_transform(
            "vega-lite".to_string(),
            r#"{"mark":"bar","title":"Revenue","data":{"values":[{"region":"East","revenue":120},{"region":"West","revenue":98}]},"encoding":{"x":{"field":"region","type":"nominal"},"y":{"field":"revenue","type":"quantitative"}}}"#.to_string(),
        )
        .expect("vega-lite transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-vega-lite"));
    assert!(artifact.html.contains("Revenue"));
    assert!(artifact.html.contains("<rect"));
    assert!(artifact.diagnostics.is_empty());

    let engines = list_transform_engines();
    let vega_lite = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("vega-lite"))
        .expect("vega-lite engine metadata");
    assert_eq!(
        vega_lite.get("execution").and_then(Value::as_str),
        Some("rust-native-svg")
    );
}

#[test]
fn vega_lite_area_mark_renders_static_svg_preview() {
    let artifact = run_transform(
            "vega-lite".to_string(),
            r#"{"mark":"area","title":"Pipeline","data":{"values":[{"month":"Jan","value":10},{"month":"Feb","value":18},{"month":"Mar","value":14}]},"encoding":{"x":{"field":"month","type":"ordinal"},"y":{"field":"value","type":"quantitative"}}}"#.to_string(),
        )
        .expect("vega-lite area transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-vega-lite"));
    assert!(artifact.html.contains("Pipeline"));
    assert!(artifact.html.contains("<polygon"));
    assert!(artifact.html.contains("fill-opacity=\"0.18\""));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn vega_lite_color_encoding_renders_grouped_series_preview() {
    let artifact = run_transform(
            "vega-lite".to_string(),
            r##"{"mark":"line","title":"Revenue by Region","data":{"values":[{"month":"Jan","region":"East","revenue":120},{"month":"Feb","region":"East","revenue":135},{"month":"Jan","region":"West","revenue":98},{"month":"Feb","region":"West","revenue":112}]},"encoding":{"x":{"field":"month","type":"ordinal"},"y":{"field":"revenue","type":"quantitative"},"color":{"field":"region","type":"nominal"}}}"##.to_string(),
        )
        .expect("vega-lite color transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("Revenue by Region"));
    assert!(artifact.html.contains("data-series=\"East\""));
    assert!(artifact.html.contains("data-series=\"West\""));
    assert!(artifact.html.contains("vega-legend-item"));
    assert!(artifact.html.contains("Jan: East"));
    assert!(artifact.html.contains("<polyline"));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn vega_lite_tick_mark_renders_static_distribution_preview() {
    let artifact = run_transform(
            "vega-lite".to_string(),
            r##"{"mark":{"type":"tick"},"title":"Risk Score Distribution","data":{"values":[{"team":"Legal","risk":72,"lane":"Review"},{"team":"Finance","risk":58,"lane":"Review"},{"team":"Operations","risk":41,"lane":"Ready"}]},"encoding":{"x":{"field":"team","type":"nominal","title":"Team"},"y":{"field":"risk","type":"quantitative","title":"Risk score"},"color":{"field":"lane","type":"nominal"}}}"##.to_string(),
        )
        .expect("vega-lite tick transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("Risk Score Distribution"));
    assert!(artifact.html.contains("vega-tick-mark"));
    assert!(artifact.html.contains("data-series=\"Review\""));
    assert!(artifact.html.contains("data-series=\"Ready\""));
    assert!(artifact.html.contains("aria-label=\"Legal: Review 72\""));
    assert!(artifact.html.contains("vega-axis-title vega-x-title"));
    assert!(artifact.html.contains(">Risk score<"));
    assert!(!artifact.html.contains("Unsupported Vega-Lite mark"));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn vega_lite_text_mark_renders_static_labels_preview() {
    let artifact = run_transform(
            "vega-lite".to_string(),
            r##"{"mark":"text","title":"Milestone Readiness Labels","data":{"values":[{"stage":"Security","score":92,"label":"Ready","lane":"Release"},{"stage":"Accessibility","score":74,"label":"Review","lane":"Release"},{"stage":"Evidence","score":58,"label":"Blocked","lane":"Proof"}]},"encoding":{"x":{"field":"stage","type":"nominal","title":"Milestone"},"y":{"field":"score","type":"quantitative","title":"Readiness"},"text":{"field":"label"},"color":{"field":"lane","type":"nominal"}}}"##.to_string(),
        )
        .expect("vega-lite text transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("Milestone Readiness Labels"));
    assert!(artifact.html.contains("vega-text-mark"));
    assert!(artifact.html.contains(">Ready<"));
    assert!(artifact.html.contains(">Review<"));
    assert!(artifact.html.contains(">Blocked<"));
    assert!(artifact.html.contains("data-series=\"Release\""));
    assert!(artifact.html.contains("data-series=\"Proof\""));
    assert!(artifact
        .html
        .contains("aria-label=\"Security: Release 92\""));
    assert!(!artifact.html.contains("Unsupported Vega-Lite mark"));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn vega_lite_circle_and_square_marks_render_static_scatter_previews() {
    let circle = run_transform(
            "vega-lite".to_string(),
            r##"{"mark":{"type":"circle"},"title":"Opportunity Scatter","data":{"values":[{"account":"Acme","value":82,"segment":"Enterprise","dealSize":40},{"account":"Beta","value":55,"segment":"Growth","dealSize":10}]},"encoding":{"x":{"field":"account","title":"Account"},"y":{"field":"value","title":"Opportunity score"},"color":{"field":"segment"},"size":{"field":"dealSize"}}}"##.to_string(),
        )
        .expect("vega-lite circle transform");

    assert_eq!(circle.output_kind, "svg");
    assert!(circle.html.contains("Opportunity Scatter"));
    assert!(circle.html.contains("vega-circle-mark"));
    assert!(circle.html.contains("data-series=\"Enterprise\""));
    assert!(circle.html.contains("data-size=\"40\""));
    assert!(circle.html.contains("r=\"12.0\""));
    assert!(circle.html.contains("r=\"4.0\""));
    assert!(circle.html.contains("aria-label=\"Acme: Enterprise 82\""));
    assert!(!circle.html.contains("Unsupported Vega-Lite mark"));
    assert!(circle.diagnostics.is_empty());

    let square = run_transform(
            "vega-lite".to_string(),
            r##"{"mark":"square","title":"Control Status Matrix","data":{"values":[{"control":"Access","score":88,"status":"Ready"},{"control":"Logging","score":67,"status":"Review"}]},"encoding":{"x":{"field":"control","title":"Control"},"y":{"field":"score","title":"Readiness"},"color":{"field":"status"}}}"##.to_string(),
        )
        .expect("vega-lite square transform");

    assert_eq!(square.output_kind, "svg");
    assert!(square.html.contains("Control Status Matrix"));
    assert!(square.html.contains("vega-square-mark"));
    assert!(square.html.contains("data-series=\"Review\""));
    assert!(square.html.contains("aria-label=\"Logging: Review 67\""));
    assert!(!square.html.contains("Unsupported Vega-Lite mark"));
    assert!(square.diagnostics.is_empty());
}

#[test]
fn vega_lite_preview_preserves_negative_values_aggregation_and_axis_titles() {
    let artifact = run_transform(
            "vega-lite".to_string(),
            r##"{"mark":{"type":"bar"},"title":"Net Revenue Variance","data":{"values":[{"month":"Jan","region":"East","net":120},{"month":"Jan","region":"East","net":-30},{"month":"Jan","region":"West","net":-45},{"month":"Feb","region":"East","net":75},{"month":"Feb","region":"West","net":30}]},"encoding":{"x":{"field":"month","type":"ordinal","title":"Month"},"y":{"field":"net","type":"quantitative","aggregate":"sum","title":"Net revenue"},"color":{"field":"region","type":"nominal"}}}"##.to_string(),
        )
        .expect("vega-lite variance transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("Net Revenue Variance"));
    assert!(artifact.html.contains("vega-zero-line"));
    assert!(artifact.html.contains("vega-axis-title vega-x-title"));
    assert!(artifact.html.contains(">Month<"));
    assert!(artifact.html.contains(">Net revenue<"));
    assert!(artifact.html.contains("data-series=\"East\""));
    assert!(artifact.html.contains("data-series=\"West\""));
    assert!(artifact.html.contains("data-value=\"90\""));
    assert!(artifact.html.contains("data-value=\"-45\""));
    assert!(artifact.diagnostics.is_empty());
}

#[test]
fn vega_lite_rule_mark_renders_reference_lines() {
    let horizontal = run_transform(
            "vega-lite".to_string(),
            r##"{"mark":"rule","title":"SLA Thresholds","data":{"values":[{"threshold":95,"lane":"Target","label":"Committed SLA"},{"threshold":80,"lane":"Watch","label":"Review floor"}]},"encoding":{"y":{"field":"threshold","type":"quantitative","title":"Score"},"color":{"field":"lane"},"text":{"field":"label"}}}"##.to_string(),
        )
        .expect("vega-lite horizontal rule transform");

    assert_eq!(horizontal.output_kind, "svg");
    assert!(horizontal.html.contains("SLA Thresholds"));
    assert!(horizontal.html.contains("data-vega-mark=\"rule\""));
    assert!(horizontal
        .html
        .contains("data-rule-orientation=\"horizontal\""));
    assert!(horizontal.html.contains("vega-rule-mark"));
    assert!(horizontal.html.contains("Committed SLA"));
    assert!(horizontal.html.contains("Review floor"));
    assert!(horizontal.html.contains("data-series=\"Target\""));
    assert!(horizontal.html.contains("data-value=\"95\""));
    assert!(horizontal.diagnostics.is_empty());

    let vertical = run_transform(
            "vega-lite".to_string(),
            r##"{"mark":{"type":"rule"},"title":"Budget Gate","encoding":{"x":{"datum":120000,"type":"quantitative","title":"Budget"}}}"##.to_string(),
        )
        .expect("vega-lite vertical datum rule transform");

    assert_eq!(vertical.output_kind, "svg");
    assert!(vertical.html.contains("Budget Gate"));
    assert!(vertical.html.contains("data-rule-orientation=\"vertical\""));
    assert!(vertical.html.contains("data-value=\"120000\""));
    assert!(vertical.html.contains(">120000<"));
    assert!(vertical.html.contains(">Budget<"));
    assert!(vertical.diagnostics.is_empty());
}

#[test]
fn vega_lite_unsupported_marks_report_supported_static_subset() {
    let artifact = run_transform(
            "vega-lite".to_string(),
            r#"{"mark":"arc","data":{"values":[{"label":"A","value":10}]},"encoding":{"x":{"field":"label"},"y":{"field":"value"}}}"#.to_string(),
        )
        .expect("vega-lite unsupported transform");

    assert_eq!(artifact.output_kind, "html");
    assert!(artifact.html.contains("Unsupported Vega-Lite mark"));
    assert!(artifact.diagnostics.iter().any(|diagnostic| diagnostic
        .suggestion
        .as_deref()
        .is_some_and(|suggestion| suggestion
            .contains("bar, line, point, circle, square, area, tick, text, or rule"))));
}

#[test]
fn mermaid_transform_renders_simple_flowchart_svg() {
    let artifact = run_transform(
        "mermaid".to_string(),
        "flowchart TD\nA[Start] --> B{Review}\nB -->|Approve| C[Publish]".to_string(),
    )
    .expect("mermaid transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-mermaid"));
    assert!(artifact.html.contains("Start"));
    assert!(artifact.html.contains("Publish"));
    assert!(artifact.html.contains("marker-end"));
    assert!(artifact.diagnostics.is_empty());

    let engines = list_transform_engines();
    let mermaid = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("mermaid"))
        .expect("mermaid engine metadata");
    assert_eq!(
        mermaid.get("execution").and_then(Value::as_str),
        Some("rust-native-svg")
    );
}
