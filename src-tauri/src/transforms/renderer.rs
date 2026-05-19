use super::{
    business::{
        render_adr_html, render_bibtex_html, render_diff_html, render_glossary_html,
        render_roadmap_html, render_timeline_svg,
    },
    chart::render_chart_svg,
    diagram,
    external::{run_external_transform, ExternalTransformRequest},
    options::TransformExecutionOptions,
    qr, structured, transform_cache_key,
    visual_data::{render_geojson_svg, render_stl_svg, render_topojson_svg, render_vega_lite_svg},
    TransformArtifact,
};
use crate::{
    diag, escape_html, rich_blocks::render_layout_block_html, sha256_hex,
    tables::render_delimited_table, DocumentDiagnostic,
};
use serde_json::Value;

pub(crate) fn render_transform(
    name: &str,
    body: &str,
    fence_options: &Value,
    options: &TransformExecutionOptions,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> TransformArtifact {
    if external_transform_supported(name) {
        if let Some(artifact) =
            render_external_transform(name, body, fence_options, options, diagnostics)
        {
            return artifact;
        }
    }

    let source_hash = sha256_hex(body.as_bytes());
    let mut artifact_diags = Vec::new();
    let html = match name {
        "calc" => "<aside class=\"transform transform-calc\">Calculations resolved into document variables.</aside>".to_string(),
        "csv" => render_delimited_table(body, ',', &mut artifact_diags, diagnostics),
        "tsv" => render_delimited_table(body, '\t', &mut artifact_diags, diagnostics),
        "json" => structured::render_structured_data_html("json", body, &mut artifact_diags, diagnostics),
        "yaml" => structured::render_structured_data_html("yaml", body, &mut artifact_diags, diagnostics),
        "glossary" => render_glossary_html(body),
        "layout" => render_layout_block_html(body),
        "timeline" => render_timeline_svg(body),
        "roadmap" => render_roadmap_html(body),
        "adr" => render_adr_html(body),
        "diff" => render_diff_html(body),
        "qr" => qr::render_qr_svg(body, &mut artifact_diags, diagnostics),
        "chart" => render_chart_svg(body),
        "openapi" => structured::render_openapi_html(body, &mut artifact_diags, diagnostics),
        "json-schema" => structured::render_json_schema_html(body, &mut artifact_diags, diagnostics),
        "bibtex" => render_bibtex_html(body, &mut artifact_diags, diagnostics),
        "geojson" => render_geojson_svg(body, &mut artifact_diags, diagnostics),
        "topojson" => render_topojson_svg(body, &mut artifact_diags, diagnostics),
        "stl" => render_stl_svg(body, &mut artifact_diags, diagnostics),
        "vega-lite" => render_vega_lite_svg(body, &mut artifact_diags, diagnostics),
        "mermaid" => diagram::render_mermaid_svg(body, &mut artifact_diags, diagnostics),
        "pikchr" => diagram::render_pikchr_svg(body, &mut artifact_diags, diagnostics),
        "dot" | "graphviz" => diagram::render_dot_svg(name, body, &mut artifact_diags, diagnostics),
        "plantuml" => diagram::render_plantuml_svg(body, &mut artifact_diags, diagnostics),
        "d2" => diagram::render_d2_svg(body, &mut artifact_diags, diagnostics),
        _ => format!("<pre>{}</pre>", escape_html(body)),
    };
    let output_hash = sha256_hex(html.as_bytes());
    TransformArtifact {
        id: format!("{name}-{source_hash}"),
        name: name.to_string(),
        output_kind: if html.contains("<svg") { "svg" } else { "html" }.to_string(),
        output_hash,
        cache_key: transform_cache_key(name, "embedded", "rust-native", &source_hash),
        execution_kind: "embedded".to_string(),
        engine_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        engine_path: None,
        input_mode: "embedded".to_string(),
        duration_ms: None,
        source_hash,
        source: body.to_string(),
        source_file: None,
        source_line: None,
        end_source_line: None,
        options: fence_options.clone(),
        html,
        diagnostics: artifact_diags,
    }
}

pub(crate) fn supported_transform(name: &str) -> bool {
    matches!(
        name,
        "calc"
            | "csv"
            | "tsv"
            | "json"
            | "yaml"
            | "glossary"
            | "layout"
            | "timeline"
            | "roadmap"
            | "adr"
            | "diff"
            | "qr"
            | "chart"
            | "mermaid"
            | "pikchr"
            | "dot"
            | "graphviz"
            | "plantuml"
            | "d2"
            | "vega-lite"
            | "geojson"
            | "topojson"
            | "stl"
            | "openapi"
            | "json-schema"
            | "bibtex"
    )
}

fn render_external_transform(
    name: &str,
    body: &str,
    fence_options: &Value,
    options: &TransformExecutionOptions,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Option<TransformArtifact> {
    let engine_path = options.engine_path(name)?;
    let request = ExternalTransformRequest {
        name: name.to_string(),
        body: body.to_string(),
        engine_path: Some(engine_path),
        trusted: options.trusted(name),
        input_mode: options.input_mode(name),
        timeout_ms: options.timeout_ms,
        max_input_bytes: None,
        max_output_bytes: None,
    };
    match run_external_transform(request) {
        Ok(mut artifact) => {
            artifact.source = body.to_string();
            artifact.options = fence_options.clone();
            diagnostics.extend(artifact.diagnostics.iter().cloned());
            Some(artifact)
        }
        Err(error) => {
            diagnostics.push(diag(
                "warning",
                format!("{name} external transform failed: {error}"),
                None,
                None,
                Some("Check transform trust, engine path, input mode, and timeout settings."),
            ));
            None
        }
    }
}

fn external_transform_supported(name: &str) -> bool {
    matches!(name, "pikchr" | "dot" | "graphviz" | "plantuml" | "d2")
}
