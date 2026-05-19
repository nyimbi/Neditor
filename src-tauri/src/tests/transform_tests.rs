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

    for name in [
        "calc",
        "mermaid",
        "pikchr",
        "dot",
        "graphviz",
        "plantuml",
        "d2",
        "vega-lite",
        "chart",
        "geojson",
        "topojson",
        "stl",
        "csv",
        "tsv",
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
        "@book{porter1985, title={Competitive Advantage}}".to_string(),
    )
    .expect("bibtex transform");

    assert_eq!(artifact.output_kind, "html");
    assert!(artifact.html.contains("transform-bibtex"));
    assert!(artifact.html.contains("<dt>porter1985</dt>"));
    assert!(artifact.html.contains("<dd>Competitive Advantage</dd>"));
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
        "Now: Drafting\nNext: Review\nLater: Publish".to_string(),
    )
    .expect("roadmap transform");
    assert_eq!(roadmap.output_kind, "html");
    assert!(roadmap.html.contains("transform-roadmap"));
    assert!(roadmap.html.contains("Review"));

    let adr = run_transform(
        "adr".to_string(),
        "Status: accepted\nDecision: Use local-first exports".to_string(),
    )
    .expect("adr transform");
    assert_eq!(adr.output_kind, "html");
    assert!(adr.html.contains("transform-adr"));
    assert!(adr.html.contains("Use local-first exports"));

    let diff = run_transform("diff".to_string(), "@@ -1 +1 @@\n-old\n+new".to_string())
        .expect("diff transform");
    assert_eq!(diff.output_kind, "html");
    assert!(diff.html.contains("transform-diff"));
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
fn topojson_transform_renders_static_svg_preview() {
    let artifact = run_transform(
            "topojson".to_string(),
            r#"{"type":"Topology","transform":{"scale":[0.01,0.01],"translate":[36.8,-1.3]},"objects":{},"arcs":[[[0,0],[5,4],[5,-2]]]}"#.to_string(),
        )
        .expect("topojson transform");

    assert_eq!(artifact.output_kind, "svg");
    assert!(artifact.html.contains("transform-topojson"));
    assert!(artifact.html.contains("<polyline"));
    assert!(artifact.html.contains("1 arcs"));
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

#[cfg(unix)]
#[test]
fn external_transforms_are_trust_gated_and_limited() {
    let graphviz = write_executable_script(
            "graphviz-adapter",
            "#!/bin/sh\nprintf '<svg data-args=\"%s\">' \"$*\"\nfor arg in \"$@\"; do if [ -f \"$arg\" ]; then cat \"$arg\"; fi; done\ncat\nprintf '</svg>'\n",
        );
    let graphviz_path = path_to_string(&graphviz);
    let trust_error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "digraph {}".to_string(),
        engine_path: Some(graphviz_path.clone()),
        trusted: false,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();
    assert!(trust_error.contains("explicit trust"));

    let unique_body = format!(
        "<svg>{}</svg>",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    );
    let limit_error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "1234".to_string(),
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(3),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();
    assert!(limit_error.contains("above the 3 byte limit"));

    let output_limit_error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "1234".to_string(),
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(3),
    })
    .unwrap_err();
    assert!(output_limit_error.contains("output is"));
    assert!(output_limit_error.contains("above the 3 byte limit"));

    let stdin_artifact = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: unique_body.clone(),
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .expect("stdin external transform");
    assert_eq!(stdin_artifact.execution_kind, "external");
    assert_eq!(stdin_artifact.input_mode, "stdin");
    assert!(stdin_artifact.html.contains(&unique_body));
    assert!(!stdin_artifact.cache_key.is_empty());
    let success_diagnostic = stdin_artifact
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("completed"))
        .expect("success diagnostic");
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == &format!("cache_key: {}", stdin_artifact.cache_key)));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "input_mode: stdin"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "adapter: graphviz"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "adapter_args: -Tsvg"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == &format!("input_bytes: {}", unique_body.len())));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related.starts_with("output_bytes: ")));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "timeout_ms: 1000"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "output_channel: stdout"));
    assert!(success_diagnostic
        .related
        .iter()
        .any(|related| related == "status: 0"));
    assert!(stdin_artifact
        .engine_version
        .as_deref()
        .is_some_and(|version| version.contains("file-size:")));
    let cached_artifact = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: unique_body.clone(),
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .expect("cached stdin external transform");
    assert_eq!(cached_artifact.cache_key, stdin_artifact.cache_key);
    assert_eq!(cached_artifact.output_hash, stdin_artifact.output_hash);
    assert_eq!(
        cached_artifact.engine_version,
        stdin_artifact.engine_version
    );
    assert_eq!(cached_artifact.duration_ms, Some(0));
    assert!(cached_artifact
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("served from cache")));
    assert!(cached_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == &format!("cache_key: {}", cached_artifact.cache_key))
    }));
    transforms::external::clear_external_transform_memory_cache_for_tests();
    let persistent_cached_artifact = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: unique_body,
        engine_path: Some(graphviz_path.clone()),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .expect("persistent cached stdin external transform");
    assert_eq!(
        persistent_cached_artifact.cache_key,
        stdin_artifact.cache_key
    );
    assert_eq!(persistent_cached_artifact.duration_ms, Some(0));
    assert!(persistent_cached_artifact
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("persistent cache")));
    assert!(persistent_cached_artifact
        .diagnostics
        .iter()
        .any(|diagnostic| {
            diagnostic
                .related
                .iter()
                .any(|related| related.starts_with("cached_output_bytes: "))
        }));

    let file_artifact = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "digraph {}".to_string(),
        engine_path: Some(graphviz_path),
        trusted: true,
        input_mode: Some("file".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .expect("file external transform");
    assert_eq!(file_artifact.input_mode, "file");
    assert!(file_artifact.html.contains("digraph"));
    assert!(file_artifact.html.contains("-Tsvg"));
    let _ = fs::remove_file(graphviz);
}

#[cfg(unix)]
#[test]
fn external_transform_adapters_shape_engine_specific_invocations() {
    let d2 = write_executable_script(
        "d2-adapter",
        "#!/bin/sh\nprintf '<svg data-args=\"%s\">d2</svg>' \"$*\"\n",
    );
    let d2_artifact = run_external_transform(ExternalTransformRequest {
        name: "d2".to_string(),
        body: "source -> target".to_string(),
        engine_path: Some(path_to_string(&d2)),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(2048),
    })
    .expect("d2 adapter transform");
    assert!(d2_artifact.html.contains("data-args=\"- -\""));
    assert!(d2_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "adapter: d2")
    }));

    let plantuml = write_executable_script(
            "plantuml-adapter",
            "#!/bin/sh\nlast=\"\"\nfor arg in \"$@\"; do last=\"$arg\"; done\nout=\"${last%.*}.svg\"\nprintf '<svg data-args=\"%s\">plantuml sidecar</svg>' \"$*\" > \"$out\"\n",
        );
    let plantuml_artifact = run_external_transform(ExternalTransformRequest {
        name: "plantuml".to_string(),
        body: "@startuml\nAlice -> Bob: hi\n@enduml".to_string(),
        engine_path: Some(path_to_string(&plantuml)),
        trusted: true,
        input_mode: Some("file".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(2048),
    })
    .expect("plantuml file adapter transform");
    assert!(plantuml_artifact.html.contains("plantuml sidecar"));
    assert!(plantuml_artifact.html.contains("-tsvg"));
    assert_eq!(plantuml_artifact.input_mode, "file");
    assert!(plantuml_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "adapter: plantuml")
    }));
    assert!(plantuml_artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .related
            .iter()
            .any(|related| related == "output_channel: sidecar svg")
    }));

    let engines = list_transform_engines();
    let graphviz = engines
        .iter()
        .find(|engine| engine.get("name").and_then(Value::as_str) == Some("graphviz"))
        .expect("graphviz metadata");
    assert_eq!(
        graphviz.get("defaultCommand").and_then(Value::as_str),
        Some("dot")
    );
    assert!(graphviz
        .get("adapterProfile")
        .and_then(Value::as_str)
        .is_some_and(|profile| profile.contains("Graphviz DOT adapter")));
    assert_eq!(
        graphviz
            .pointer("/diagnosticProfile/versionProbe")
            .and_then(Value::as_str),
        Some("dot -V")
    );
    assert!(graphviz
        .pointer("/diagnosticProfile/failureHint")
        .and_then(Value::as_str)
        .is_some_and(|hint| hint.contains("Graphviz dot")));

    let _ = fs::remove_file(d2);
    let _ = fs::remove_file(plantuml);
}

#[test]
fn external_transform_conformance_runs_installed_engines() {
    struct EngineCase {
        name: &'static str,
        command: &'static str,
        env_var: &'static str,
        input_mode: &'static str,
        body: String,
    }

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let cases = [
        EngineCase {
            name: "dot",
            command: "dot",
            env_var: "NEDITOR_TEST_DOT",
            input_mode: "stdin",
            body: format!("digraph G {{ start -> done [label=\"{unique}\"]; }}"),
        },
        EngineCase {
            name: "d2",
            command: "d2",
            env_var: "NEDITOR_TEST_D2",
            input_mode: "stdin",
            body: format!("source -> target: {unique}"),
        },
        EngineCase {
            name: "plantuml",
            command: "plantuml",
            env_var: "NEDITOR_TEST_PLANTUML",
            input_mode: "file",
            body: format!("@startuml\nAlice -> Bob: {unique}\n@enduml\n"),
        },
        EngineCase {
            name: "pikchr",
            command: "pikchr",
            env_var: "NEDITOR_TEST_PIKCHR",
            input_mode: "stdin",
            body: format!("box \"{unique}\"; arrow; box \"Done\""),
        },
    ];

    let mut verified = Vec::new();
    let mut skipped = Vec::new();
    for case in cases {
        let Some(path) = installed_command_path(case.env_var, case.command) else {
            skipped.push(case.name);
            continue;
        };
        let artifact = run_external_transform(ExternalTransformRequest {
            name: case.name.to_string(),
            body: case.body,
            engine_path: Some(path_to_string(&path)),
            trusted: true,
            input_mode: Some(case.input_mode.to_string()),
            timeout_ms: Some(15_000),
            max_input_bytes: Some(16_384),
            max_output_bytes: Some(1_048_576),
        })
        .unwrap_or_else(|error| {
            panic!(
                "{} conformance failed with {}: {error}",
                case.name,
                path.display()
            )
        });

        assert_eq!(artifact.execution_kind, "external");
        assert_eq!(artifact.input_mode, case.input_mode);
        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("<svg"));
        let engine_path = path_to_string(&path);
        assert_eq!(artifact.engine_path.as_deref(), Some(engine_path.as_str()));
        assert!(artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic.related.iter().any(|related| {
                related == &format!("adapter: {}", external_conformance_adapter(case.name))
            })
        }));
        assert!(artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .related
                .iter()
                .any(|related| related.starts_with("engine_version: file-size:"))
        }));
        verified.push(case.name);
    }

    eprintln!(
        "external transform conformance verified: {}; skipped: {}",
        verified.join(", "),
        skipped.join(", ")
    );
    if verified.is_empty() {
        eprintln!("No optional external transform engines were installed; set NEDITOR_TEST_DOT, NEDITOR_TEST_D2, NEDITOR_TEST_PLANTUML, or NEDITOR_TEST_PIKCHR to force a conformance run.");
    }
}

fn external_conformance_adapter(name: &str) -> &'static str {
    match name {
        "dot" => "graphviz",
        "d2" => "d2",
        "plantuml" => "plantuml",
        "pikchr" => "pikchr",
        _ => "unknown",
    }
}

#[cfg(unix)]
#[test]
fn compiler_uses_trusted_external_transform_preferences() {
    let graphviz = write_executable_script(
        "compiler-graphviz-adapter",
        "#!/bin/sh\nprintf '<svg data-args=\"%s\">' \"$*\"\ncat\nprintf '</svg>'\n",
    );
    let response = compile_with_options(
        CompileRequest {
            text:
                "---\ntitle: External Dot\n---\n# External Dot\n```dot\ndigraph { a -> b }\n```\n"
                    .to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": { "dot": path_to_string(&graphviz) },
            "trustedTransformEngines": { "dot": true },
            "transformInputModes": { "dot": "stdin" },
            "transformTimeoutMs": 1000
        }),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "dot")
        .expect("dot artifact");
    assert_eq!(artifact.execution_kind, "external");
    assert_eq!(artifact.input_mode, "stdin");
    assert!(artifact
        .engine_path
        .as_deref()
        .is_some_and(|path| path == path_to_string(&graphviz)));
    assert!(artifact.html.contains("digraph { a -> b }"));
    assert!(artifact.html.contains("-Tsvg"));
    assert!(response.html.contains("transform-external"));
    assert!(response.html.contains("transform-dot"));
    let ast_transform = response
        .document_ast
        .blocks
        .iter()
        .find_map(|block| match block {
            DocumentBlock::Transform {
                name,
                execution_kind,
                ..
            } if name == "dot" => Some(execution_kind),
            _ => None,
        })
        .expect("dot AST transform");
    assert_eq!(ast_transform.as_deref(), Some("external"));
    assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("dot external transform completed")));
    let _ = fs::remove_file(graphviz);
}

#[test]
fn compiler_falls_back_when_external_transform_is_untrusted() {
    let cat = Path::new("/bin/cat");
    if !cat.exists() {
        return;
    }
    let response = compile_with_options(
        CompileRequest {
            text:
                "---\ntitle: Untrusted Dot\n---\n# Untrusted Dot\n```dot\ndigraph { a -> b }\n```\n"
                    .to_string(),
            file_path: None,
        },
        &json!({
            "transformEnginePaths": { "dot": path_to_string(cat) },
            "trustedTransformEngines": { "dot": false }
        }),
    );

    let artifact = response
        .transform_artifacts
        .iter()
        .find(|artifact| artifact.name == "dot")
        .expect("dot artifact");
    assert_eq!(artifact.execution_kind, "embedded");
    assert_eq!(artifact.output_kind, "svg");
    assert!(!artifact.html.contains("transform-pending"));
    assert!(artifact.html.contains("transform-dot"));
    assert!(response.document_ast.blocks.iter().any(|block| {
        matches!(
            block,
            DocumentBlock::Transform { name, execution_kind, .. }
                if name == "dot" && execution_kind.as_deref() == Some("embedded")
        )
    }));
    assert!(response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("dot external transform failed")));
}

#[cfg(unix)]
#[test]
fn external_transform_rejects_non_executable_engine_path() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let script = std::env::temp_dir().join(format!("neditor-not-executable-{unique}.sh"));
    fs::write(&script, "#!/bin/sh\ncat\n").expect("write non-executable script");

    let error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "digraph {}".to_string(),
        engine_path: Some(path_to_string(&script)),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();

    let _ = fs::remove_file(script);
    assert!(error.contains("not executable"));
}

#[cfg(unix)]
#[test]
fn external_transform_timeout_covers_blocked_stdin() {
    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let script = std::env::temp_dir().join(format!("neditor-blocked-stdin-{unique}.sh"));
    fs::write(&script, "#!/bin/sh\nsleep 2\n").expect("write blocked stdin script");
    let mut permissions = fs::metadata(&script)
        .expect("script metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make script executable");

    let started = std::time::Instant::now();
    let error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "x".repeat(512 * 1024),
        engine_path: Some(path_to_string(&script)),
        trusted: true,
        input_mode: Some("stdin".to_string()),
        timeout_ms: Some(50),
        max_input_bytes: Some(1024 * 1024),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();

    let _ = fs::remove_file(script);
    assert!(error.contains("timed out"));
    assert!(
        started.elapsed() < std::time::Duration::from_secs(1),
        "blocked stdin write should not bypass the timeout"
    );
}

#[cfg(unix)]
#[test]
fn external_transform_exit_errors_include_stderr() {
    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let script = std::env::temp_dir().join(format!("neditor-stderr-exit-{unique}.sh"));
    fs::write(&script, "#!/bin/sh\necho engine exploded >&2\nexit 7\n")
        .expect("write stderr script");
    let mut permissions = fs::metadata(&script)
        .expect("script metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions).expect("make script executable");

    let error = run_external_transform(ExternalTransformRequest {
        name: "dot".to_string(),
        body: "digraph {}".to_string(),
        engine_path: Some(path_to_string(&script)),
        trusted: true,
        input_mode: Some("file".to_string()),
        timeout_ms: Some(1000),
        max_input_bytes: Some(1024),
        max_output_bytes: Some(1024),
    })
    .unwrap_err();

    let _ = fs::remove_file(script);
    assert!(error.contains("status 7"));
    assert!(error.contains("engine exploded"));
    assert!(error.contains("Check DOT syntax"));
    assert!(error.contains("-Tsvg"));
}
