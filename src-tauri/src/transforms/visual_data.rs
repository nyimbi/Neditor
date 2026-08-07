use crate::{diag, escape_html, value_to_string, DocumentDiagnostic};
use serde_json::Value;

pub(crate) fn render_vega_lite_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let spec = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid Vega-Lite JSON: {err}"),
                None,
                None,
                Some("Provide a JSON Vega-Lite spec with data.values and x/y encodings."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-vega-lite transform-error\">Invalid Vega-Lite JSON</section>".to_string();
        }
    };
    let mark = vega_lite_mark(&spec);
    if !matches!(
        mark.as_str(),
        "bar" | "line" | "point" | "circle" | "square" | "area" | "tick" | "text" | "rule" | "rect"
    ) {
        let diagnostic = diag(
            "warning",
            format!("Unsupported Vega-Lite mark for native preview: {mark}"),
            None,
            None,
            Some("Use bar, line, point, circle, square, area, tick, rect, text, or rule marks for the native static preview."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">Unsupported Vega-Lite mark</section>".to_string();
    }
    let color_field = vega_lite_encoding_field(&spec, "color");
    let text_field = vega_lite_encoding_field(&spec, "text");
    if mark == "rule" {
        return render_vega_lite_rule_svg(
            &spec,
            color_field.as_deref(),
            text_field.as_deref(),
            artifact_diags,
            diagnostics,
        );
    }
    // rect heatmap: y is categorical (nominal/ordinal) — 2-D grid path
    if mark == "rect" {
        let y_type = vega_lite_encoding_scale_type(&spec, "y");
        if y_type
            .as_deref()
            .is_some_and(|t| matches!(t, "nominal" | "ordinal"))
        {
            return render_vega_lite_rect_heatmap_path(
                &spec,
                color_field.as_deref(),
                artifact_diags,
                diagnostics,
            );
        }
    }
    let Some(x_field) = vega_lite_encoding_field(&spec, "x") else {
        return vega_lite_missing_field("x", artifact_diags, diagnostics);
    };
    let y_field = vega_lite_encoding_field(&spec, "y");
    let size_field = vega_lite_encoding_field(&spec, "size");
    let opacity_field = vega_lite_encoding_field(&spec, "opacity");
    let y_aggregate = vega_lite_encoding_aggregate(&spec, "y");
    if y_field.is_none() && !vega_lite_is_count_aggregate(y_aggregate.as_deref()) {
        return vega_lite_missing_field("y", artifact_diags, diagnostics);
    }
    let values = vega_lite_values(
        &spec,
        &x_field,
        y_field.as_deref(),
        color_field.as_deref(),
        text_field.as_deref(),
        size_field.as_deref(),
        opacity_field.as_deref(),
        y_aggregate.as_deref(),
    );
    if values.is_empty() {
        let diagnostic = diag(
            "warning",
            "Vega-Lite native preview did not find numeric data.values rows.",
            None,
            None,
            Some("Use inline data.values with a numeric y encoding."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">No drawable Vega-Lite rows</section>".to_string();
    }
    let title = spec
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Vega-Lite chart");
    let x_title = vega_lite_encoding_title(&spec, "x").unwrap_or(x_field);
    let y_title = vega_lite_encoding_title(&spec, "y")
        .or(y_field)
        .unwrap_or_else(|| "Count".to_string());
    render_vega_lite_chart_svg(
        title,
        &mark,
        &values,
        &x_title,
        &y_title,
        y_aggregate.as_deref(),
    )
}

fn render_vega_lite_rect_heatmap_path(
    spec: &Value,
    color_field: Option<&str>,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let Some(x_field) = vega_lite_encoding_field(spec, "x") else {
        return vega_lite_missing_field("x", artifact_diags, diagnostics);
    };
    let Some(y_field) = vega_lite_encoding_field(spec, "y") else {
        return vega_lite_missing_field("y", artifact_diags, diagnostics);
    };
    // value dimension: color field if present, else fall back to a field named "value"
    let value_field = color_field.unwrap_or("value");
    let cells = vega_lite_heatmap_values(spec, &x_field, &y_field, value_field);
    if cells.is_empty() {
        let diagnostic = diag(
            "warning",
            "Vega-Lite rect heatmap native preview did not find any data.values rows.",
            None,
            None,
            Some("Provide data.values rows with x, y, and a color/value field for the rect heatmap preview."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">No drawable Vega-Lite rect rows</section>".to_string();
    }
    let title = spec
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Vega-Lite heatmap");
    let x_title = vega_lite_encoding_title(spec, "x").unwrap_or(x_field);
    let y_title = vega_lite_encoding_title(spec, "y").unwrap_or(y_field);
    render_vega_lite_heatmap_svg(title, &cells, &x_title, &y_title)
}

pub(crate) fn render_geojson_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid GeoJSON document: {err}"),
                None,
                None,
                Some("Provide valid GeoJSON Feature, FeatureCollection, or Geometry JSON."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-geojson transform-error\">Invalid GeoJSON document</section>".to_string();
        }
    };
    let mut shapes = Vec::new();
    collect_geojson_shapes(&value, &mut shapes);
    let positions = geo_shapes_positions(&shapes);
    warn_geojson_projection_assumptions(&value, &positions, artifact_diags, diagnostics);
    if positions.is_empty() {
        let diagnostic = diag(
            "warning",
            "GeoJSON transform did not contain drawable coordinates.",
            None,
            None,
            Some("Add Point, LineString, Polygon, or Multi* coordinates."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-geojson transform-error\">No GeoJSON coordinates found</section>".to_string();
    }
    render_geo_shapes_svg(
        "geojson",
        "#ecfeff",
        "#67e8f9",
        "#134e4a",
        "linear-wgs84-fit",
        "longitude-latitude",
        &shapes,
    )
}

pub(crate) fn render_topojson_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid TopoJSON document: {err}"),
                None,
                None,
                Some("Provide valid TopoJSON with an arcs array."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-topojson transform-error\">Invalid TopoJSON document</section>".to_string();
        }
    };
    let shapes = decode_topojson_shapes(&value);
    if shapes.is_empty() {
        let diagnostic = diag(
            "warning",
            "TopoJSON transform did not contain drawable arcs.",
            None,
            None,
            Some("Add a Topology arcs array or verify the TopoJSON source."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-topojson transform-error\">No TopoJSON arcs found</section>".to_string();
    }
    render_geo_shapes_svg(
        "topojson",
        "#f8fafc",
        "#94a3b8",
        "#334155",
        "linear-topology-fit",
        "topology-coordinates",
        &shapes,
    )
}

pub(crate) fn render_stl_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let Some(parsed) = parse_stl_facets(body) else {
        let diagnostic = diag(
            "warning",
            "STL transform did not contain ASCII vertices or base64-encoded binary STL data.",
            None,
            None,
            Some("Use ASCII STL vertex records, or paste a base64-encoded binary STL payload or data:application/sla;base64 URI."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-stl transform-error\">No STL vertices found</section>".to_string();
    };
    let StlParsedGeometry {
        facets,
        source_kind,
        coordinate_assumption,
    } = parsed;
    if facets.is_empty() {
        let diagnostic = diag(
            "warning",
            "STL transform did not contain drawable vertex data.",
            None,
            None,
            Some("Provide triangle facets with finite XYZ vertex coordinates."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-stl transform-error\">No STL vertices found</section>".to_string();
    }
    let triangles = facets
        .iter()
        .map(|facet| StlTrianglePreview {
            projected: facet
                .vertices
                .iter()
                .map(|&vertex| stl_isometric_position(vertex))
                .collect::<Vec<_>>(),
            average_z: facet.vertices.iter().map(|(_, _, z)| *z).sum::<f64>() / 3.0,
            shade: stl_three_point_shade(facet.normal),
        })
        .collect::<Vec<_>>();
    let positions = triangles
        .iter()
        .flat_map(|triangle| triangle.projected.iter().copied())
        .collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let all_vertices = facets
        .iter()
        .flat_map(|facet| facet.vertices.iter().copied())
        .collect::<Vec<_>>();
    let (min_z, max_z) = stl_depth_bounds(&all_vertices);
    let mut sorted_triangles = triangles;
    sorted_triangles.sort_by(|left, right| {
        left.average_z
            .partial_cmp(&right.average_z)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let triangle_polygons = sorted_triangles
        .iter()
        .map(|triangle| {
            let points = projected_points(&triangle.projected, min_x, max_x, min_y, max_y);
            let (r, g, b) = stl_shade_to_rgb(triangle.shade);
            format!(
                "<polygon points=\"{points}\" data-depth=\"{:.2}\" data-shade=\"{:.2}\" fill=\"rgb({r},{g},{b})\" stroke=\"#1e3a5f\" stroke-width=\"2\" stroke-linejoin=\"round\"/>",
                triangle.average_z,
                triangle.shade,
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let z_summary = stl_depth_summary(min_z, max_z);
    let vertex_count = facets.len() * 3;
    format!(
        "<svg class=\"transform transform-stl\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\" data-projection=\"isometric-depth-fit\" data-stl-source=\"{source_kind}\" data-coordinate-assumption=\"{coordinate_assumption}\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"#f8fafc\" stroke=\"#cbd5e1\"/>{triangle_polygons}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"#334155\">{} triangles / {} vertices{z_summary}</text></svg>",
        sorted_triangles.len(),
        vertex_count,
    )
}

fn vega_lite_mark(spec: &Value) -> String {
    spec.get("mark")
        .and_then(|mark| {
            mark.as_str().map(ToString::to_string).or_else(|| {
                mark.get("type")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
        })
        .unwrap_or_else(|| "bar".to_string())
}

fn vega_lite_encoding_field(spec: &Value, channel: &str) -> Option<String> {
    spec.pointer(&format!("/encoding/{channel}/field"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn vega_lite_encoding_title(spec: &Value, channel: &str) -> Option<String> {
    spec.pointer(&format!("/encoding/{channel}/title"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
}

fn vega_lite_encoding_scale_type(spec: &Value, channel: &str) -> Option<String> {
    spec.pointer(&format!("/encoding/{channel}/type"))
        .and_then(Value::as_str)
        .map(|s| s.trim().to_ascii_lowercase())
}

fn vega_lite_encoding_aggregate(spec: &Value, channel: &str) -> Option<String> {
    spec.pointer(&format!("/encoding/{channel}/aggregate"))
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| {
            matches!(
                value.as_str(),
                "sum" | "mean" | "average" | "avg" | "min" | "max" | "count"
            )
        })
}

fn vega_lite_is_count_aggregate(aggregate: Option<&str>) -> bool {
    aggregate.is_some_and(|value| value == "count")
}

fn vega_lite_encoding_numeric_datum(spec: &Value, channel: &str) -> Option<f64> {
    spec.pointer(&format!("/encoding/{channel}/datum"))
        .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))
        .filter(|value| value.is_finite())
}

#[derive(Clone, Debug)]
struct VegaLiteDatum {
    label: String,
    value: f64,
    series: Option<String>,
    text: Option<String>,
    size: Option<f64>,
    opacity: Option<f64>,
}

#[derive(Clone, Debug)]
struct VegaLiteHeatmapDatum {
    x_label: String,
    y_label: String,
    value: f64,
}

#[derive(Clone, Debug)]
struct VegaLiteRuleDatum {
    label: String,
    value: f64,
    series: Option<String>,
    text: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum VegaLiteRuleOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
enum VegaLiteRuleSource {
    Field(String),
    Datum(f64),
}

fn vega_lite_values(
    spec: &Value,
    x_field: &str,
    y_field: Option<&str>,
    color_field: Option<&str>,
    text_field: Option<&str>,
    size_field: Option<&str>,
    opacity_field: Option<&str>,
    y_aggregate: Option<&str>,
) -> Vec<VegaLiteDatum> {
    let values = spec
        .pointer("/data/values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let x = row.get(x_field).map(value_to_axis_label)?;
            let y = if vega_lite_is_count_aggregate(y_aggregate) {
                1.0
            } else {
                row.get(y_field?)
                    .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))?
            };
            let series = color_field
                .and_then(|field| row.get(field))
                .map(value_to_axis_label)
                .filter(|value| !value.trim().is_empty());
            let text = text_field
                .and_then(|field| row.get(field))
                .map(value_to_axis_label)
                .filter(|value| !value.trim().is_empty());
            let size = size_field.and_then(|field| {
                row.get(field)
                    .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))
                    .filter(|value| value.is_finite())
            });
            let opacity = opacity_field.and_then(|field| {
                row.get(field)
                    .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))
                    .filter(|value| value.is_finite())
            });
            Some(VegaLiteDatum {
                label: x,
                value: y,
                series,
                text,
                size,
                opacity,
            })
        })
        .collect::<Vec<_>>();
    match y_aggregate {
        Some(aggregate) => aggregate_vega_lite_values(values, aggregate),
        None => values,
    }
}

fn vega_lite_heatmap_values(
    spec: &Value,
    x_field: &str,
    y_field: &str,
    value_field: &str,
) -> Vec<VegaLiteHeatmapDatum> {
    spec.pointer("/data/values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let x_label = row.get(x_field).map(value_to_axis_label)?;
            let y_label = row.get(y_field).map(value_to_axis_label)?;
            let value = row
                .get(value_field)
                .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok()))
                .unwrap_or(1.0);
            Some(VegaLiteHeatmapDatum {
                x_label,
                y_label,
                value,
            })
        })
        .collect()
}

fn render_vega_lite_rule_svg(
    spec: &Value,
    color_field: Option<&str>,
    text_field: Option<&str>,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let title = spec
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Vega-Lite rule");
    let Some((orientation, source, axis_title)) = vega_lite_rule_source(spec) else {
        let diagnostic = diag(
            "warning",
            "Vega-Lite rule native preview is missing a numeric x or y field/datum.",
            None,
            None,
            Some("Set encoding.y.field, encoding.y.datum, encoding.x.field, or encoding.x.datum for rule marks."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">Missing rule encoding</section>".to_string();
    };
    let values = vega_lite_rule_values(spec, &source, color_field, text_field);
    if values.is_empty() {
        let diagnostic = diag(
            "warning",
            "Vega-Lite rule native preview did not find numeric rule values.",
            None,
            None,
            Some("Use numeric rule field values in data.values, or a numeric encoding datum."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">No drawable Vega-Lite rules</section>".to_string();
    }
    render_vega_lite_rule_chart_svg(title, orientation, &values, &axis_title)
}

fn vega_lite_rule_source(
    spec: &Value,
) -> Option<(VegaLiteRuleOrientation, VegaLiteRuleSource, String)> {
    if let Some(field) = vega_lite_encoding_field(spec, "y") {
        let title = vega_lite_encoding_title(spec, "y").unwrap_or_else(|| field.clone());
        return Some((
            VegaLiteRuleOrientation::Horizontal,
            VegaLiteRuleSource::Field(field),
            title,
        ));
    }
    if let Some(value) = vega_lite_encoding_numeric_datum(spec, "y") {
        let title = vega_lite_encoding_title(spec, "y").unwrap_or_else(|| "y datum".to_string());
        return Some((
            VegaLiteRuleOrientation::Horizontal,
            VegaLiteRuleSource::Datum(value),
            title,
        ));
    }
    if let Some(field) = vega_lite_encoding_field(spec, "x") {
        let title = vega_lite_encoding_title(spec, "x").unwrap_or_else(|| field.clone());
        return Some((
            VegaLiteRuleOrientation::Vertical,
            VegaLiteRuleSource::Field(field),
            title,
        ));
    }
    if let Some(value) = vega_lite_encoding_numeric_datum(spec, "x") {
        let title = vega_lite_encoding_title(spec, "x").unwrap_or_else(|| "x datum".to_string());
        return Some((
            VegaLiteRuleOrientation::Vertical,
            VegaLiteRuleSource::Datum(value),
            title,
        ));
    }
    None
}

fn vega_lite_rule_values(
    spec: &Value,
    source: &VegaLiteRuleSource,
    color_field: Option<&str>,
    text_field: Option<&str>,
) -> Vec<VegaLiteRuleDatum> {
    match source {
        VegaLiteRuleSource::Datum(value) if value.is_finite() => vec![VegaLiteRuleDatum {
            label: format_vega_value(*value),
            value: *value,
            series: None,
            text: None,
        }],
        VegaLiteRuleSource::Datum(_) => Vec::new(),
        VegaLiteRuleSource::Field(field) => spec
            .pointer("/data/values")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|row| {
                let value = row
                    .get(field)
                    .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))?;
                let series = color_field
                    .and_then(|field| row.get(field))
                    .map(value_to_axis_label)
                    .filter(|value| !value.trim().is_empty());
                let text = text_field
                    .and_then(|field| row.get(field))
                    .map(value_to_axis_label)
                    .filter(|value| !value.trim().is_empty());
                let label = text
                    .clone()
                    .or_else(|| series.clone())
                    .unwrap_or_else(|| format!("{} {}", field, format_vega_value(value)));
                Some(VegaLiteRuleDatum {
                    label,
                    value,
                    series,
                    text,
                })
            })
            .collect(),
    }
}

#[derive(Clone, Debug)]
struct VegaLiteAggregateBucket {
    label: String,
    series: Option<String>,
    values: Vec<f64>,
}

fn aggregate_vega_lite_values(values: Vec<VegaLiteDatum>, aggregate: &str) -> Vec<VegaLiteDatum> {
    let mut buckets: Vec<VegaLiteAggregateBucket> = Vec::new();
    for value in values {
        if let Some(bucket) = buckets
            .iter_mut()
            .find(|bucket| bucket.label == value.label && bucket.series == value.series)
        {
            bucket.values.push(value.value);
        } else {
            buckets.push(VegaLiteAggregateBucket {
                label: value.label,
                series: value.series,
                values: vec![value.value],
            });
        }
    }
    buckets
        .into_iter()
        .filter_map(|bucket| {
            let value = aggregate_vega_bucket(&bucket.values, aggregate)?;
            Some(VegaLiteDatum {
                label: bucket.label,
                value,
                series: bucket.series,
                text: None,
                size: None,
                opacity: None,
            })
        })
        .collect()
}

fn aggregate_vega_bucket(values: &[f64], aggregate: &str) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    match aggregate {
        "sum" => Some(values.iter().sum()),
        "mean" | "average" | "avg" => Some(values.iter().sum::<f64>() / values.len() as f64),
        "min" => values.iter().copied().reduce(f64::min),
        "max" => values.iter().copied().reduce(f64::max),
        "count" => Some(values.len() as f64),
        _ => None,
    }
}

fn value_to_axis_label(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value_to_string(value))
}

fn vega_lite_missing_field(
    channel: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let diagnostic = diag(
        "warning",
        format!("Vega-Lite native preview is missing {channel} field encoding."),
        None,
        None,
        Some("Set encoding.x.field and encoding.y.field."),
    );
    artifact_diags.push(diagnostic.clone());
    diagnostics.push(diagnostic);
    format!(
        "<section class=\"transform transform-vega-lite transform-error\">Missing {channel} encoding</section>"
    )
}

fn render_vega_lite_chart_svg(
    title: &str,
    mark: &str,
    values: &[VegaLiteDatum],
    x_title: &str,
    y_title: &str,
    y_aggregate: Option<&str>,
) -> String {
    let domain = VegaLiteDomain::from_values(values);
    let width = 820usize;
    let height = 340usize;
    let plot_left = 72usize;
    let plot_width = 680usize;
    let labels = unique_vega_labels(values);
    let series = unique_vega_series(values);
    let size_domain = VegaLiteSizeDomain::from_values(values);
    let opacity_domain = VegaLiteOpacityDomain::from_values(values);
    let step = plot_width / labels.len().max(1);
    let aggregate_attr = y_aggregate
        .map(|aggregate| format!(" data-vega-aggregate=\"{}\"", escape_html(aggregate)))
        .unwrap_or_default();
    // Build SVG: axes first, then grid (behind marks), then zero-line
    let mut svg = format!(
        "<svg class=\"transform transform-vega-lite\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\" data-vega-mark=\"{}\"{aggregate_attr}><text x=\"72\" y=\"34\" font-size=\"18\" fill=\"#111827\">{}</text><line x1=\"72\" y1=\"262\" x2=\"770\" y2=\"262\" stroke=\"#94a3b8\"/><line x1=\"72\" y1=\"54\" x2=\"72\" y2=\"262\" stroke=\"#94a3b8\"/>",
        escape_html(mark),
        escape_html(title),
    );
    render_vega_y_axis_grid(&mut svg, &domain);
    svg.push_str(&format!(
        "<line class=\"vega-zero-line\" x1=\"72\" y1=\"{:.1}\" x2=\"770\" y2=\"{:.1}\" stroke=\"#64748b\" stroke-dasharray=\"4 3\"/>",
        domain.zero_y, domain.zero_y
    ));
    render_vega_axis_labels(&mut svg, &labels, plot_left, step);
    render_vega_axis_titles(&mut svg, x_title, y_title);
    if mark == "bar" || mark == "rect" {
        let mark_class = if mark == "rect" {
            "vega-rect-mark"
        } else {
            "vega-bar-mark"
        };
        let series_count = series.len().max(1);
        let bar_width = ((step.saturating_sub(16)) / series_count).max(4);
        for datum in values {
            let label_index = labels
                .iter()
                .position(|label| label == &datum.label)
                .unwrap_or(0);
            let series_index = datum
                .series
                .as_ref()
                .and_then(|name| series.iter().position(|series| series == name))
                .unwrap_or(0);
            let x = plot_left + label_index * step + 8 + series_index * bar_width;
            let color = VEGA_SERIES_COLORS[series_index % VEGA_SERIES_COLORS.len()];
            let series_attr = datum
                .series
                .as_deref()
                .map(|series| format!(" data-series=\"{}\"", escape_html(series)))
                .unwrap_or_default();
            let opacity_attr = opacity_attr_for(&opacity_domain, datum.opacity);
            let y = domain.y(datum.value);
            let bar_y = y.min(domain.zero_y);
            let bar_height = (y - domain.zero_y).abs().max(1.0);
            let value_label = format_vega_value(datum.value);
            svg.push_str(&format!(
                "<rect class=\"{mark_class}\" x=\"{x}\" y=\"{bar_y:.1}\" width=\"{bar_width}\" height=\"{bar_height:.1}\" fill=\"{color}\" data-label=\"{}\" data-value=\"{}\"{series_attr}{opacity_attr}/>",
                escape_html(&datum.label),
                escape_html(&value_label)
            ));
        }
    } else {
        let series_names = if series.is_empty() {
            vec![String::new()]
        } else {
            series.clone()
        };
        for (series_index, series_name) in series_names.iter().enumerate() {
            let points = values
                .iter()
                .filter(|datum| match (&datum.series, series_name.is_empty()) {
                    (None, true) => true,
                    (Some(name), false) => name == series_name,
                    _ => false,
                })
                .filter_map(|datum| {
                    let label_index = labels.iter().position(|label| label == &datum.label)?;
                    let x = plot_left + label_index * step + step / 2;
                    let y = domain.y(datum.value);
                    Some((
                        x,
                        y,
                        datum.label.as_str(),
                        datum.value,
                        datum.text.as_deref(),
                        datum.size,
                        datum.opacity,
                    ))
                })
                .collect::<Vec<_>>();
            let color = VEGA_SERIES_COLORS[series_index % VEGA_SERIES_COLORS.len()];
            let series_attr = if series_name.is_empty() {
                String::new()
            } else {
                format!(" data-series=\"{}\"", escape_html(series_name))
            };
            if mark == "area" {
                let baseline = domain.zero_y;
                let area_points = points
                    .iter()
                    .map(|(x, y, _, _, _, _, _)| format!("{x},{y:.1}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let area = match (points.first(), points.last()) {
                    (Some((first_x, _, _, _, _, _, _)), Some((last_x, _, _, _, _, _, _))) => {
                        format!("{first_x},{baseline:.1} {area_points} {last_x},{baseline:.1}")
                    }
                    _ => String::new(),
                };
                svg.push_str(&format!(
                    "<polygon points=\"{area}\" fill=\"{}\" fill-opacity=\"0.18\" stroke=\"{color}\" stroke-width=\"3\"{series_attr}/>",
                    color
                ));
            } else if mark == "line" {
                let polyline = points
                    .iter()
                    .map(|(x, y, _, _, _, _, _)| format!("{x},{y:.1}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                svg.push_str(&format!(
                    "<polyline points=\"{polyline}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"3\"{series_attr}/>"
                ));
            }
            for (x, y, label, value, text, size, opacity) in points {
                let label = if series_name.is_empty() {
                    label.to_string()
                } else {
                    format!("{label}: {series_name}")
                };
                let value_label = format_vega_value(value);
                let size_attr = size
                    .map(|size| format!(" data-size=\"{}\"", escape_html(&format_vega_value(size))))
                    .unwrap_or_default();
                let opacity_attr = opacity_attr_for(&opacity_domain, opacity);
                if mark == "text" {
                    let text = text.unwrap_or(&value_label);
                    svg.push_str(&format!(
                        "<text class=\"vega-text-mark\" x=\"{x}\" y=\"{:.1}\" font-size=\"13\" text-anchor=\"middle\" fill=\"{color}\" aria-label=\"{} {}\" data-value=\"{}\"{size_attr}{opacity_attr}{series_attr}>{}</text>",
                        y - 7.0,
                        escape_html(&label),
                        escape_html(&value_label),
                        escape_html(&value_label),
                        escape_html(text)
                    ));
                } else if mark == "tick" {
                    let x1 = x.saturating_sub(10);
                    let x2 = x + 10;
                    svg.push_str(&format!(
                        "<line class=\"vega-tick-mark\" x1=\"{x1}\" y1=\"{y:.1}\" x2=\"{x2}\" y2=\"{y:.1}\" stroke=\"{color}\" stroke-width=\"4\" stroke-linecap=\"round\" aria-label=\"{} {}\" data-value=\"{}\"{size_attr}{opacity_attr}{series_attr}/>",
                        escape_html(&label),
                        escape_html(&value_label),
                        escape_html(&value_label)
                    ));
                } else if mark == "square" {
                    let radius = size_domain
                        .as_ref()
                        .map(|domain| domain.radius(size))
                        .unwrap_or(5.0);
                    let side = radius * 2.0;
                    let square_x = x as f64 - radius;
                    let square_y = y - radius;
                    svg.push_str(&format!(
                        "<rect class=\"vega-square-mark\" x=\"{square_x:.1}\" y=\"{square_y:.1}\" width=\"{side:.1}\" height=\"{side:.1}\" fill=\"{color}\" aria-label=\"{} {}\" data-value=\"{}\"{size_attr}{opacity_attr}{series_attr}/>",
                        escape_html(&label),
                        escape_html(&value_label),
                        escape_html(&value_label)
                    ));
                } else {
                    let class_attr = if mark == "circle" {
                        " class=\"vega-circle-mark\""
                    } else {
                        " class=\"vega-point-mark\""
                    };
                    let radius = size_domain
                        .as_ref()
                        .map(|domain| domain.radius(size))
                        .unwrap_or(5.0);
                    svg.push_str(&format!(
                        "<circle{class_attr} cx=\"{x}\" cy=\"{y:.1}\" r=\"{radius:.1}\" fill=\"{color}\" aria-label=\"{} {}\" data-value=\"{}\"{size_attr}{opacity_attr}{series_attr}/>",
                        escape_html(&label),
                        escape_html(&value_label),
                        escape_html(&value_label)
                    ));
                }
            }
        }
    }
    render_vega_legend(&mut svg, &series);
    svg.push_str("</svg>");
    svg
}

fn render_vega_lite_rule_chart_svg(
    title: &str,
    orientation: VegaLiteRuleOrientation,
    values: &[VegaLiteRuleDatum],
    axis_title: &str,
) -> String {
    let min = values
        .iter()
        .map(|datum| datum.value)
        .filter(|value| value.is_finite())
        .fold(0.0_f64, f64::min);
    let mut max = values
        .iter()
        .map(|datum| datum.value)
        .filter(|value| value.is_finite())
        .fold(0.0_f64, f64::max);
    let mut min = min;
    if (max - min).abs() < f64::EPSILON {
        max = (max + 1.0).max(1.0);
        min = min.min(0.0);
    }
    let series = unique_vega_rule_series(values);
    let orientation_attr = match orientation {
        VegaLiteRuleOrientation::Horizontal => "horizontal",
        VegaLiteRuleOrientation::Vertical => "vertical",
    };
    let mut svg = format!(
        "<svg class=\"transform transform-vega-lite\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 820 340\" role=\"img\" data-vega-mark=\"rule\" data-rule-orientation=\"{orientation_attr}\"><text x=\"72\" y=\"34\" font-size=\"18\" fill=\"#111827\">{}</text><rect x=\"72\" y=\"54\" width=\"698\" height=\"208\" fill=\"#f8fafc\" stroke=\"#cbd5e1\"/><line x1=\"72\" y1=\"262\" x2=\"770\" y2=\"262\" stroke=\"#94a3b8\"/><line x1=\"72\" y1=\"54\" x2=\"72\" y2=\"262\" stroke=\"#94a3b8\"/>",
        escape_html(title),
    );
    match orientation {
        VegaLiteRuleOrientation::Horizontal => {
            let zero_y = chart_value_y(0.0, min, max);
            svg.push_str(&format!(
                "<line class=\"vega-zero-line\" x1=\"72\" y1=\"{zero_y:.1}\" x2=\"770\" y2=\"{zero_y:.1}\" stroke=\"#64748b\" stroke-dasharray=\"4 3\"/>"
            ));
            render_vega_axis_titles(&mut svg, "", axis_title);
            for (index, datum) in values.iter().enumerate() {
                let y = chart_value_y(datum.value, min, max);
                let color = VEGA_SERIES_COLORS[index % VEGA_SERIES_COLORS.len()];
                let label = datum.text.as_deref().unwrap_or(&datum.label);
                let value_label = format_vega_value(datum.value);
                let series_attr = datum
                    .series
                    .as_deref()
                    .map(|series| format!(" data-series=\"{}\"", escape_html(series)))
                    .unwrap_or_default();
                svg.push_str(&format!(
                    "<line class=\"vega-rule-mark\" x1=\"72\" y1=\"{y:.1}\" x2=\"770\" y2=\"{y:.1}\" stroke=\"{color}\" stroke-width=\"3\" stroke-dasharray=\"8 5\" aria-label=\"{} {}\" data-value=\"{}\"{series_attr}/><text class=\"vega-rule-label\" x=\"762\" y=\"{:.1}\" font-size=\"12\" text-anchor=\"end\" fill=\"{color}\">{}</text>",
                    y - 6.0,
                    escape_html(label),
                    escape_html(&value_label),
                    escape_html(&value_label),
                    escape_html(label)
                ));
            }
        }
        VegaLiteRuleOrientation::Vertical => {
            let zero_x = chart_value_x(0.0, min, max);
            svg.push_str(&format!(
                "<line class=\"vega-zero-line\" x1=\"{zero_x:.1}\" y1=\"54\" x2=\"{zero_x:.1}\" y2=\"262\" stroke=\"#64748b\" stroke-dasharray=\"4 3\"/>"
            ));
            render_vega_axis_titles(&mut svg, axis_title, "");
            for (index, datum) in values.iter().enumerate() {
                let x = chart_value_x(datum.value, min, max);
                let color = VEGA_SERIES_COLORS[index % VEGA_SERIES_COLORS.len()];
                let label = datum.text.as_deref().unwrap_or(&datum.label);
                let value_label = format_vega_value(datum.value);
                let series_attr = datum
                    .series
                    .as_deref()
                    .map(|series| format!(" data-series=\"{}\"", escape_html(series)))
                    .unwrap_or_default();
                svg.push_str(&format!(
                    "<line class=\"vega-rule-mark\" x1=\"{x:.1}\" y1=\"54\" x2=\"{x:.1}\" y2=\"262\" stroke=\"{color}\" stroke-width=\"3\" stroke-dasharray=\"8 5\" aria-label=\"{} {}\" data-value=\"{}\"{series_attr}/><text class=\"vega-rule-label\" x=\"{:.1}\" y=\"72\" font-size=\"12\" text-anchor=\"middle\" fill=\"{color}\">{}</text>",
                    x + 4.0,
                    escape_html(label),
                    escape_html(&value_label),
                    escape_html(&value_label),
                    escape_html(label)
                ));
            }
        }
    }
    render_vega_legend(&mut svg, &series);
    svg.push_str("</svg>");
    svg
}

#[derive(Clone, Debug)]
struct VegaLiteSizeDomain {
    min: f64,
    max: f64,
}

impl VegaLiteSizeDomain {
    fn from_values(values: &[VegaLiteDatum]) -> Option<Self> {
        let mut sizes = values
            .iter()
            .filter_map(|datum| datum.size)
            .filter(|size| size.is_finite());
        let first = sizes.next()?;
        let (min, max) = sizes.fold((first, first), |(min, max), size| {
            (min.min(size), max.max(size))
        });
        Some(Self { min, max })
    }

    fn radius(&self, size: Option<f64>) -> f64 {
        let Some(size) = size.filter(|value| value.is_finite()) else {
            return 5.0;
        };
        let range = (self.max - self.min).abs();
        if range < f64::EPSILON {
            return 8.0;
        }
        let normalized = ((size - self.min) / range).clamp(0.0, 1.0);
        4.0 + normalized * 8.0
    }
}

#[derive(Clone, Debug)]
struct VegaLiteDomain {
    min: f64,
    max: f64,
    zero_y: f64,
}

impl VegaLiteDomain {
    fn from_values(values: &[VegaLiteDatum]) -> Self {
        let mut min = 0.0_f64;
        let mut max = 0.0_f64;
        for value in values
            .iter()
            .map(|datum| datum.value)
            .filter(|value| value.is_finite())
        {
            min = min.min(value);
            max = max.max(value);
        }
        if (max - min).abs() < f64::EPSILON {
            max = (max + 1.0).max(1.0);
            min = min.min(0.0);
        }
        let zero_y = chart_value_y(0.0, min, max);
        Self { min, max, zero_y }
    }

    fn y(&self, value: f64) -> f64 {
        chart_value_y(value, self.min, self.max)
    }
}

fn chart_value_y(value: f64, min: f64, max: f64) -> f64 {
    let plot_top = 54.0_f64;
    let plot_bottom = 262.0_f64;
    let range = (max - min).abs().max(1.0);
    plot_top + ((max - value) / range) * (plot_bottom - plot_top)
}

fn chart_value_x(value: f64, min: f64, max: f64) -> f64 {
    let plot_left = 72.0_f64;
    let plot_right = 770.0_f64;
    let range = (max - min).abs().max(1.0);
    plot_left + ((value - min) / range) * (plot_right - plot_left)
}

fn format_vega_value(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if (rounded.fract()).abs() < 0.001 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.2}")
    }
}

const VEGA_SERIES_COLORS: [&str; 8] = [
    "#275DA8", "#0f766e", "#b45309", "#7c3aed", "#be123c", "#15803d", "#0369a1", "#a21caf",
];

fn unique_vega_labels(values: &[VegaLiteDatum]) -> Vec<String> {
    let mut labels = Vec::new();
    for datum in values {
        if !labels.iter().any(|label| label == &datum.label) {
            labels.push(datum.label.clone());
        }
    }
    labels
}

fn unique_vega_series(values: &[VegaLiteDatum]) -> Vec<String> {
    let mut series = Vec::new();
    for datum in values {
        if let Some(name) = &datum.series {
            if !series.iter().any(|series| series == name) {
                series.push(name.clone());
            }
        }
    }
    series
}

fn unique_vega_rule_series(values: &[VegaLiteRuleDatum]) -> Vec<String> {
    let mut series = Vec::new();
    for datum in values {
        if let Some(name) = &datum.series {
            if !series.iter().any(|series| series == name) {
                series.push(name.clone());
            }
        }
    }
    series
}

fn render_vega_axis_labels(svg: &mut String, labels: &[String], plot_left: usize, step: usize) {
    for (index, label) in labels.iter().enumerate() {
        let x = plot_left + index * step + step / 2;
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"286\" font-size=\"12\" text-anchor=\"middle\">{}</text>",
            x,
            escape_html(label)
        ));
    }
}

fn render_vega_axis_titles(svg: &mut String, x_title: &str, y_title: &str) {
    if !x_title.trim().is_empty() {
        svg.push_str(&format!(
            "<text class=\"vega-axis-title vega-x-title\" x=\"410\" y=\"326\" font-size=\"12\" text-anchor=\"middle\" fill=\"#475569\">{}</text>",
            escape_html(x_title)
        ));
    }
    if !y_title.trim().is_empty() {
        svg.push_str(&format!(
            "<text class=\"vega-axis-title vega-y-title\" x=\"20\" y=\"160\" font-size=\"12\" text-anchor=\"middle\" fill=\"#475569\" transform=\"rotate(-90 20 160)\">{}</text>",
            escape_html(y_title)
        ));
    }
}

fn render_vega_legend(svg: &mut String, series: &[String]) {
    if series.is_empty() {
        return;
    }
    for (index, series) in series.iter().enumerate() {
        let x = 600usize;
        let y = 60 + index * 20;
        let color = VEGA_SERIES_COLORS[index % VEGA_SERIES_COLORS.len()];
        svg.push_str(&format!(
            "<g class=\"vega-legend-item\"><rect x=\"{x}\" y=\"{}\" width=\"12\" height=\"12\" fill=\"{color}\"/><text x=\"{}\" y=\"{y}\" font-size=\"12\" fill=\"#334155\">{}</text></g>",
            y.saturating_sub(10),
            x + 18,
            escape_html(series)
        ));
    }
}

// ── Opacity domain ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct VegaLiteOpacityDomain {
    min: f64,
    max: f64,
}

impl VegaLiteOpacityDomain {
    fn from_values(values: &[VegaLiteDatum]) -> Option<Self> {
        let mut opacities = values
            .iter()
            .filter_map(|d| d.opacity)
            .filter(|v| v.is_finite());
        let first = opacities.next()?;
        let (min, max) = opacities.fold((first, first), |(mn, mx), v| (mn.min(v), mx.max(v)));
        Some(Self { min, max })
    }

    /// Map raw field value to an SVG opacity in [0.15, 1.0].
    fn opacity(&self, raw: f64) -> f64 {
        let range = (self.max - self.min).abs();
        if range < f64::EPSILON {
            return 0.8;
        }
        let t = ((raw - self.min) / range).clamp(0.0, 1.0);
        0.15 + t * 0.85
    }
}

/// Build an `opacity="…"` attribute string, or empty if no opacity data.
fn opacity_attr_for(domain: &Option<VegaLiteOpacityDomain>, raw: Option<f64>) -> String {
    let Some(domain) = domain else {
        return String::new();
    };
    let Some(raw) = raw else {
        return String::new();
    };
    format!(" opacity=\"{:.2}\"", domain.opacity(raw))
}

// ── Y-axis grid + tick labels ─────────────────────────────────────────────────

fn render_vega_y_axis_grid(svg: &mut String, domain: &VegaLiteDomain) {
    let n_ticks = 5usize;
    for i in 0..=n_ticks {
        let frac = i as f64 / n_ticks as f64;
        let value = domain.min + frac * (domain.max - domain.min);
        let y = domain.y(value);
        let label = format_vega_value(value);
        svg.push_str(&format!(
            "<line class=\"vega-grid-line\" x1=\"72\" y1=\"{y:.1}\" x2=\"770\" y2=\"{y:.1}\" stroke=\"#e2e8f0\"/><text class=\"vega-y-tick\" x=\"66\" y=\"{:.1}\" font-size=\"11\" text-anchor=\"end\" fill=\"#64748b\">{}</text>",
            y + 4.0,
            escape_html(&label)
        ));
    }
}

// ── Heatmap renderer (rect mark, y = nominal/ordinal) ────────────────────────

fn render_vega_lite_heatmap_svg(
    title: &str,
    cells: &[VegaLiteHeatmapDatum],
    x_title: &str,
    y_title: &str,
) -> String {
    let x_labels = unique_ordered_string_labels(cells.iter().map(|d| d.x_label.as_str()));
    let y_labels = unique_ordered_string_labels(cells.iter().map(|d| d.y_label.as_str()));
    let min_val = cells.iter().map(|d| d.value).fold(f64::INFINITY, f64::min);
    let max_val = cells
        .iter()
        .map(|d| d.value)
        .fold(f64::NEG_INFINITY, f64::max);
    let val_range = (max_val - min_val).abs().max(f64::EPSILON);

    let width = 820usize;
    let height = 340usize;
    let plot_left = 80usize;
    let plot_top = 54usize;
    let plot_width = 670usize;
    let plot_height = 220usize;
    let cell_w = plot_width / x_labels.len().max(1);
    let cell_h = plot_height / y_labels.len().max(1);

    let mut svg = format!(
        "<svg class=\"transform transform-vega-lite\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\" data-vega-mark=\"rect\" data-heatmap=\"true\"><text x=\"80\" y=\"34\" font-size=\"18\" fill=\"#111827\">{}</text><line x1=\"80\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#94a3b8\"/><line x1=\"80\" y1=\"{plot_top}\" x2=\"80\" y2=\"{}\" stroke=\"#94a3b8\"/>",
        escape_html(title),
        plot_top + plot_height,
        plot_left + plot_width,
        plot_top + plot_height,
        plot_top + plot_height,
    );

    // cells
    for datum in cells {
        let xi = x_labels
            .iter()
            .position(|l| l == &datum.x_label)
            .unwrap_or(0);
        let yi = y_labels
            .iter()
            .position(|l| l == &datum.y_label)
            .unwrap_or(0);
        let x = plot_left + xi * cell_w;
        let y = plot_top + yi * cell_h;
        let opacity = 0.15 + ((datum.value - min_val) / val_range) * 0.85;
        let value_label = format_vega_value(datum.value);
        svg.push_str(&format!(
            "<rect class=\"vega-rect-mark\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#275DA8\" fill-opacity=\"{opacity:.2}\" data-x=\"{}\" data-y=\"{}\" data-value=\"{}\"/>",
            x + 1,
            y + 1,
            cell_w.saturating_sub(2),
            cell_h.saturating_sub(2),
            escape_html(&datum.x_label),
            escape_html(&datum.y_label),
            escape_html(&value_label),
        ));
    }

    // x-axis labels
    for (i, label) in x_labels.iter().enumerate() {
        let lx = plot_left + i * cell_w + cell_w / 2;
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"11\" text-anchor=\"middle\" fill=\"#334155\">{}</text>",
            lx,
            plot_top + plot_height + 16,
            escape_html(label)
        ));
    }
    // y-axis labels
    for (i, label) in y_labels.iter().enumerate() {
        let ly = plot_top + i * cell_h + cell_h / 2 + 4;
        svg.push_str(&format!(
            "<text x=\"74\" y=\"{ly}\" font-size=\"11\" text-anchor=\"end\" fill=\"#334155\">{}</text>",
            escape_html(label)
        ));
    }
    // color scale legend (min→max strip on right)
    let legend_x = plot_left + plot_width + 16;
    for step in 0..=10usize {
        let frac = step as f64 / 10.0;
        let ly = plot_top + (frac * plot_height as f64) as usize;
        let lh = plot_height / 10 + 1;
        let op = 0.15 + frac * 0.85;
        svg.push_str(&format!(
            "<rect x=\"{legend_x}\" y=\"{ly}\" width=\"14\" height=\"{lh}\" fill=\"#275DA8\" fill-opacity=\"{op:.2}\"/>",
        ));
    }
    let min_label = format_vega_value(min_val);
    let max_label = format_vega_value(max_val);
    svg.push_str(&format!(
        "<text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#64748b\">{}</text><text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"#64748b\">{}</text>",
        legend_x + 18, plot_top + plot_height,
        escape_html(&min_label),
        legend_x + 18, plot_top,
        escape_html(&max_label),
    ));

    let mut svg_mut = svg;
    render_vega_axis_titles(&mut svg_mut, x_title, y_title);
    svg_mut.push_str("</svg>");
    svg_mut
}

fn unique_ordered_string_labels<'a>(iter: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut seen = Vec::new();
    for label in iter {
        let s = label.to_string();
        if !seen.iter().any(|l| l == &s) {
            seen.push(s);
        }
    }
    seen
}

#[derive(Clone, Debug)]
enum GeoShape {
    Point((f64, f64)),
    Line(Vec<(f64, f64)>),
    Polygon(Vec<Vec<(f64, f64)>>),
}

fn render_geo_shapes_svg(
    class_name: &str,
    fill: &str,
    stroke: &str,
    text_color: &str,
    projection: &str,
    coordinate_assumption: &str,
    shapes: &[GeoShape],
) -> String {
    let positions = geo_shapes_positions(shapes);
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let mut point_count = 0usize;
    let mut line_count = 0usize;
    let mut polygon_count = 0usize;
    let mut coordinate_count = 0usize;
    let mut body = String::new();

    for shape in shapes.iter().take(500) {
        match shape {
            GeoShape::Point(position) => {
                point_count += 1;
                coordinate_count += 1;
                let (x, y) = project_geojson_position(*position, min_x, max_x, min_y, max_y);
                body.push_str(&format!(
                    "<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"4\" fill=\"#0f766e\"/>"
                ));
            }
            GeoShape::Line(line) => {
                if line.len() < 2 {
                    continue;
                }
                line_count += 1;
                coordinate_count += line.len();
                let points = projected_points(line, min_x, max_x, min_y, max_y);
                body.push_str(&format!(
                    "<polyline points=\"{points}\" fill=\"none\" stroke=\"#275DA8\" stroke-width=\"3\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>"
                ));
            }
            GeoShape::Polygon(rings) => {
                for ring in rings {
                    if ring.len() < 3 {
                        continue;
                    }
                    polygon_count += 1;
                    coordinate_count += ring.len();
                    let points = projected_points(ring, min_x, max_x, min_y, max_y);
                    body.push_str(&format!(
                        "<polygon points=\"{points}\" fill=\"rgba(39,93,168,.18)\" stroke=\"#275DA8\" stroke-width=\"2\" stroke-linejoin=\"round\"/>"
                    ));
                }
            }
        }
    }

    let mut summary_parts = Vec::new();
    if polygon_count > 0 {
        summary_parts.push(format!("{polygon_count} polygons"));
    }
    if line_count > 0 {
        summary_parts.push(format!("{line_count} lines"));
    }
    if point_count > 0 {
        summary_parts.push(format!("{point_count} points"));
    }
    summary_parts.push(format!("{coordinate_count} coordinates"));
    let summary = summary_parts.join(" / ");

    format!(
        "<svg class=\"transform transform-{class_name}\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\" data-projection=\"{}\" data-coordinate-assumption=\"{}\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"{fill}\" stroke=\"{stroke}\"/>{body}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"{text_color}\">{summary}</text></svg>",
        escape_html(projection),
        escape_html(coordinate_assumption)
    )
}

fn projected_points(
    positions: &[(f64, f64)],
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> String {
    positions
        .iter()
        .map(|position| {
            let (x, y) = project_geojson_position(*position, min_x, max_x, min_y, max_y);
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn geo_shapes_positions(shapes: &[GeoShape]) -> Vec<(f64, f64)> {
    shapes
        .iter()
        .flat_map(|shape| match shape {
            GeoShape::Point(position) => vec![*position],
            GeoShape::Line(line) => line.clone(),
            GeoShape::Polygon(rings) => rings.iter().flatten().copied().collect(),
        })
        .take(4000)
        .collect()
}

fn warn_geojson_projection_assumptions(
    value: &Value,
    positions: &[(f64, f64)],
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if value.get("crs").is_some() {
        push_visual_data_warning(
            artifact_diags,
            diagnostics,
            "GeoJSON native preview ignores legacy crs metadata and assumes WGS84 longitude/latitude coordinates.",
            "Reproject the source to WGS84 longitude/latitude before relying on the static map preview.",
        );
    }

    if positions
        .iter()
        .any(|(longitude, latitude)| longitude.abs() > 180.0 || latitude.abs() > 90.0)
    {
        push_visual_data_warning(
            artifact_diags,
            diagnostics,
            "GeoJSON native preview detected coordinates outside normal WGS84 longitude/latitude ranges.",
            "Verify the coordinate reference system or reproject projected coordinates to WGS84 for the native preview.",
        );
    }
}

fn push_visual_data_warning(
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
    message: &str,
    suggestion: &str,
) {
    let diagnostic = diag("warning", message, None, None, Some(suggestion));
    artifact_diags.push(diagnostic.clone());
    diagnostics.push(diagnostic);
}

fn collect_geojson_shapes(value: &Value, shapes: &mut Vec<GeoShape>) {
    let Some(kind) = value.get("type").and_then(Value::as_str) else {
        return;
    };
    match kind {
        "FeatureCollection" => {
            if let Some(features) = value.get("features").and_then(Value::as_array) {
                for feature in features {
                    collect_geojson_shapes(feature, shapes);
                }
            }
        }
        "Feature" => {
            if let Some(geometry) = value.get("geometry") {
                collect_geojson_shapes(geometry, shapes);
            }
        }
        "GeometryCollection" => {
            if let Some(geometries) = value.get("geometries").and_then(Value::as_array) {
                for geometry in geometries {
                    collect_geojson_shapes(geometry, shapes);
                }
            }
        }
        "Point" => {
            if let Some(point) = value.get("coordinates").and_then(position_from_value) {
                shapes.push(GeoShape::Point(point));
            }
        }
        "MultiPoint" => {
            if let Some(points) = value.get("coordinates").and_then(line_from_value) {
                shapes.extend(points.into_iter().map(GeoShape::Point));
            }
        }
        "LineString" => {
            if let Some(line) = value.get("coordinates").and_then(line_from_value) {
                shapes.push(GeoShape::Line(line));
            }
        }
        "MultiLineString" => {
            if let Some(lines) = value
                .get("coordinates")
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(line_from_value))
            {
                shapes.extend(lines.map(GeoShape::Line));
            }
        }
        "Polygon" => {
            if let Some(rings) = value.get("coordinates").and_then(rings_from_value) {
                shapes.push(GeoShape::Polygon(rings));
            }
        }
        "MultiPolygon" => {
            if let Some(polygons) = value
                .get("coordinates")
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(rings_from_value))
            {
                shapes.extend(polygons.map(GeoShape::Polygon));
            }
        }
        _ => {}
    }
}

fn position_from_value(value: &Value) -> Option<(f64, f64)> {
    let coordinates = value.as_array()?;
    Some((
        coordinates.first()?.as_f64()?,
        coordinates.get(1)?.as_f64()?,
    ))
}

fn line_from_value(value: &Value) -> Option<Vec<(f64, f64)>> {
    let positions = value
        .as_array()?
        .iter()
        .filter_map(position_from_value)
        .collect::<Vec<_>>();
    (!positions.is_empty()).then_some(positions)
}

fn rings_from_value(value: &Value) -> Option<Vec<Vec<(f64, f64)>>> {
    let rings = value
        .as_array()?
        .iter()
        .filter_map(line_from_value)
        .collect::<Vec<_>>();
    (!rings.is_empty()).then_some(rings)
}

fn decode_topojson_shapes(value: &Value) -> Vec<GeoShape> {
    let transform = decode_topojson_transform(value);
    let decoded_arcs = decode_topojson_arcs(value, transform);
    let mut shapes = Vec::new();
    if let Some(objects) = value.get("objects").and_then(Value::as_object) {
        for object in objects.values() {
            collect_topojson_object_shapes(object, &decoded_arcs, transform, &mut shapes);
        }
    }
    if shapes.is_empty() {
        shapes.extend(decoded_arcs.into_iter().map(GeoShape::Line));
    }
    shapes
}

#[derive(Clone, Copy, Debug)]
struct TopojsonTransform {
    scale: (f64, f64),
    translate: (f64, f64),
}

fn decode_topojson_transform(value: &Value) -> TopojsonTransform {
    let scale = value
        .pointer("/transform/scale")
        .and_then(Value::as_array)
        .and_then(|items| Some((items.first()?.as_f64()?, items.get(1)?.as_f64()?)))
        .unwrap_or((1.0, 1.0));
    let translate = value
        .pointer("/transform/translate")
        .and_then(Value::as_array)
        .and_then(|items| Some((items.first()?.as_f64()?, items.get(1)?.as_f64()?)))
        .unwrap_or((0.0, 0.0));
    TopojsonTransform { scale, translate }
}

fn decode_topojson_arcs(value: &Value, transform: TopojsonTransform) -> Vec<Vec<(f64, f64)>> {
    value
        .get("arcs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|arc| decode_topojson_arc(arc, transform))
        .collect()
}

fn collect_topojson_object_shapes(
    object: &Value,
    decoded_arcs: &[Vec<(f64, f64)>],
    transform: TopojsonTransform,
    shapes: &mut Vec<GeoShape>,
) {
    let Some(kind) = object.get("type").and_then(Value::as_str) else {
        return;
    };
    match kind {
        "GeometryCollection" => {
            if let Some(geometries) = object.get("geometries").and_then(Value::as_array) {
                for geometry in geometries {
                    collect_topojson_object_shapes(geometry, decoded_arcs, transform, shapes);
                }
            }
        }
        "LineString" => {
            if let Some(line) = object
                .get("arcs")
                .and_then(|arcs| topojson_line_from_arc_refs(arcs, decoded_arcs))
            {
                shapes.extend(
                    split_line_at_antimeridian(line)
                        .into_iter()
                        .filter(|seg| seg.len() >= 2)
                        .map(GeoShape::Line),
                );
            }
        }
        "MultiLineString" => {
            if let Some(lines) = object.get("arcs").and_then(Value::as_array) {
                for item in lines {
                    if let Some(line) = topojson_line_from_arc_refs(item, decoded_arcs) {
                        shapes.extend(
                            split_line_at_antimeridian(line)
                                .into_iter()
                                .filter(|seg| seg.len() >= 2)
                                .map(GeoShape::Line),
                        );
                    }
                }
            }
        }
        "Polygon" => {
            if let Some(rings) = object.get("arcs").and_then(Value::as_array).map(|items| {
                items
                    .iter()
                    .filter_map(|ring| topojson_line_from_arc_refs(ring, decoded_arcs))
                    .flat_map(split_line_at_antimeridian)
                    .filter(|seg| seg.len() >= 3)
                    .collect::<Vec<_>>()
            }) {
                if !rings.is_empty() {
                    shapes.push(GeoShape::Polygon(rings));
                }
            }
        }
        "MultiPolygon" => {
            if let Some(polygons) = object.get("arcs").and_then(Value::as_array) {
                for polygon in polygons {
                    if let Some(rings) = polygon.as_array().map(|items| {
                        items
                            .iter()
                            .filter_map(|ring| topojson_line_from_arc_refs(ring, decoded_arcs))
                            .flat_map(split_line_at_antimeridian)
                            .filter(|seg| seg.len() >= 3)
                            .collect::<Vec<_>>()
                    }) {
                        if !rings.is_empty() {
                            shapes.push(GeoShape::Polygon(rings));
                        }
                    }
                }
            }
        }
        "Point" => {
            if let Some(point) = object
                .get("coordinates")
                .and_then(position_from_value)
                .map(|point| apply_topojson_transform(point, transform))
            {
                shapes.push(GeoShape::Point(point));
            }
        }
        "MultiPoint" => {
            if let Some(points) =
                object
                    .get("coordinates")
                    .and_then(line_from_value)
                    .map(|points| {
                        points
                            .into_iter()
                            .map(|point| apply_topojson_transform(point, transform))
                    })
            {
                shapes.extend(points.into_iter().map(GeoShape::Point));
            }
        }
        _ => {}
    }
}

/// Split a decoded geographic arc at antimeridian crossings (|Δlon| > 180°).
/// Each sub-segment is returned as its own vec; callers filter by minimum length.
fn split_line_at_antimeridian(line: Vec<(f64, f64)>) -> Vec<Vec<(f64, f64)>> {
    if line.len() < 2 {
        return vec![line];
    }
    let mut result: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();
    for point in line {
        if let Some(&prev) = current.last() {
            if (point.0 - prev.0).abs() > 180.0 {
                result.push(std::mem::take(&mut current));
            }
        }
        current.push(point);
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

fn topojson_line_from_arc_refs(
    arc_refs: &Value,
    decoded_arcs: &[Vec<(f64, f64)>],
) -> Option<Vec<(f64, f64)>> {
    let mut line = Vec::new();
    for arc_ref in arc_refs.as_array()? {
        let arc_index = arc_ref.as_i64()?;
        let mut arc = topojson_arc_by_ref(arc_index, decoded_arcs)?;
        if !line.is_empty() && !arc.is_empty() {
            arc.remove(0);
        }
        line.extend(arc);
    }
    (!line.is_empty()).then_some(line)
}

fn topojson_arc_by_ref(arc_ref: i64, decoded_arcs: &[Vec<(f64, f64)>]) -> Option<Vec<(f64, f64)>> {
    if arc_ref >= 0 {
        return decoded_arcs.get(arc_ref as usize).cloned();
    }
    let index = (-arc_ref - 1) as usize;
    let mut arc = decoded_arcs.get(index)?.clone();
    arc.reverse();
    Some(arc)
}

fn decode_topojson_arc(arc: &Value, transform: TopojsonTransform) -> Option<Vec<(f64, f64)>> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut positions = Vec::new();
    for point in arc.as_array()? {
        let coordinates = point.as_array()?;
        x += coordinates.first()?.as_f64()?;
        y += coordinates.get(1)?.as_f64()?;
        positions.push(apply_topojson_transform((x, y), transform));
    }
    (!positions.is_empty()).then_some(positions)
}

fn apply_topojson_transform(
    (x, y): (f64, f64),
    TopojsonTransform {
        scale: (scale_x, scale_y),
        translate: (translate_x, translate_y),
    }: TopojsonTransform,
) -> (f64, f64) {
    (x * scale_x + translate_x, y * scale_y + translate_y)
}

#[derive(Clone, Debug)]
struct StlFacet {
    /// Unit surface normal (computed from cross product if file normal is degenerate).
    normal: (f64, f64, f64),
    vertices: [(f64, f64, f64); 3],
}

#[derive(Clone, Debug)]
struct StlTrianglePreview {
    projected: Vec<(f64, f64)>,
    average_z: f64,
    shade: f64,
}

#[derive(Clone, Debug)]
struct StlParsedGeometry {
    facets: Vec<StlFacet>,
    source_kind: &'static str,
    coordinate_assumption: &'static str,
}

// --- STL 3-point Lambertian lighting helpers ---

fn stl_normalize3((x, y, z): (f64, f64, f64)) -> (f64, f64, f64) {
    let len = (x * x + y * y + z * z).sqrt();
    if len < f64::EPSILON {
        (0.0, 0.0, 1.0)
    } else {
        (x / len, y / len, z / len)
    }
}

fn stl_dot3((ax, ay, az): (f64, f64, f64), (bx, by, bz): (f64, f64, f64)) -> f64 {
    ax * bx + ay * by + az * bz
}

fn stl_cross3((ax, ay, az): (f64, f64, f64), (bx, by, bz): (f64, f64, f64)) -> (f64, f64, f64) {
    (ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx)
}

fn compute_stl_face_normal(
    v0: (f64, f64, f64),
    v1: (f64, f64, f64),
    v2: (f64, f64, f64),
) -> (f64, f64, f64) {
    let e1 = (v1.0 - v0.0, v1.1 - v0.1, v1.2 - v0.2);
    let e2 = (v2.0 - v0.0, v2.1 - v0.1, v2.2 - v0.2);
    stl_normalize3(stl_cross3(e1, e2))
}

/// Returns a unit normal: uses the provided `n` if it has non-negligible length,
/// otherwise falls back to the cross product of the triangle edges.
fn stl_validated_normal(
    n: (f64, f64, f64),
    v0: (f64, f64, f64),
    v1: (f64, f64, f64),
    v2: (f64, f64, f64),
) -> (f64, f64, f64) {
    let len_sq = n.0 * n.0 + n.1 * n.1 + n.2 * n.2;
    if len_sq < 1e-8 {
        compute_stl_face_normal(v0, v1, v2)
    } else {
        let len = len_sq.sqrt();
        (n.0 / len, n.1 / len, n.2 / len)
    }
}

/// Three-point Lambertian shading: key (upper-right-front), fill (left), rim (behind-top).
/// Returns a [0,1] brightness value.
fn stl_three_point_shade(normal: (f64, f64, f64)) -> f64 {
    let key_dir = stl_normalize3((0.6, 0.8, 0.5));
    let fill_dir = stl_normalize3((-0.8, 0.2, 0.4));
    let rim_dir = stl_normalize3((0.0, -0.5, -0.8));
    let key = stl_dot3(normal, key_dir).max(0.0) * 0.65;
    let fill = stl_dot3(normal, fill_dir).max(0.0) * 0.25;
    let rim = stl_dot3(normal, rim_dir).max(0.0) * 0.35;
    (0.12_f64 + key + fill + rim).clamp(0.0, 1.0)
}

/// Map a [0,1] shade value to an RGB blue-palette color (dark navy → bright periwinkle).
fn stl_shade_to_rgb(shade: f64) -> (u8, u8, u8) {
    let r = (15.0 + shade * 115.0).round() as u8;
    let g = (40.0 + shade * 135.0).round() as u8;
    let b = (90.0 + shade * 150.0).round() as u8;
    (r, g, b)
}

fn stl_isometric_position((x, y, z): (f64, f64, f64)) -> (f64, f64) {
    (x - (z * 0.45), y + (z * 0.35))
}

fn stl_depth_bounds(vertices: &[(f64, f64, f64)]) -> (f64, f64) {
    vertices
        .iter()
        .map(|(_, _, z)| *z)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min_z, max_z), z| {
            (min_z.min(z), max_z.max(z))
        })
}

fn stl_depth_opacity(value: f64, min_z: f64, max_z: f64) -> f64 {
    let range = (max_z - min_z).abs();
    if range < f64::EPSILON {
        return 0.24;
    }
    let normalized = ((value - min_z) / range).clamp(0.0, 1.0);
    0.18 + (normalized * 0.18)
}

fn stl_depth_summary(min_z: f64, max_z: f64) -> String {
    let depth = max_z - min_z;
    if depth.abs() < f64::EPSILON {
        return String::new();
    }
    format!(" / z-depth {}", format_stl_number(depth))
}

fn format_stl_number(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract().abs() < 0.001 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.2}")
    }
}

fn parse_ascii_stl_facets(body: &str) -> Vec<StlFacet> {
    let mut facets = Vec::new();
    let mut current_normal: Option<(f64, f64, f64)> = None;
    let mut current_vertices: Vec<(f64, f64, f64)> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("facet normal ") {
            let mut parts = rest.split_whitespace();
            if let (Some(nx), Some(ny), Some(nz)) = (parts.next(), parts.next(), parts.next()) {
                if let (Ok(nx), Ok(ny), Ok(nz)) =
                    (nx.parse::<f64>(), ny.parse::<f64>(), nz.parse::<f64>())
                {
                    if nx.is_finite() && ny.is_finite() && nz.is_finite() {
                        current_normal = Some((nx, ny, nz));
                    }
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("vertex ") {
            let mut parts = rest.split_whitespace();
            if let (Some(x), Some(y), Some(z)) = (parts.next(), parts.next(), parts.next()) {
                if let (Ok(x), Ok(y), Ok(z)) =
                    (x.parse::<f64>(), y.parse::<f64>(), z.parse::<f64>())
                {
                    if x.is_finite() && y.is_finite() && z.is_finite() {
                        current_vertices.push((x, y, z));
                    }
                }
            }
        } else if trimmed == "endfacet" {
            if current_vertices.len() == 3 {
                let [v0, v1, v2] = [
                    current_vertices[0],
                    current_vertices[1],
                    current_vertices[2],
                ];
                let normal = match current_normal {
                    Some(n) => stl_validated_normal(n, v0, v1, v2),
                    None => compute_stl_face_normal(v0, v1, v2),
                };
                facets.push(StlFacet {
                    normal,
                    vertices: [v0, v1, v2],
                });
            }
            current_vertices.clear();
            current_normal = None;
        }
    }
    facets
}

fn parse_stl_facets(body: &str) -> Option<StlParsedGeometry> {
    let ascii_facets = parse_ascii_stl_facets(body);
    if !ascii_facets.is_empty() {
        return Some(StlParsedGeometry {
            facets: ascii_facets,
            source_kind: "ascii",
            coordinate_assumption: "ascii-stl-xyz",
        });
    }
    let facets = parse_base64_binary_stl_facets(body)?;
    Some(StlParsedGeometry {
        facets,
        source_kind: "binary-base64",
        coordinate_assumption: "binary-stl-base64-xyz",
    })
}

fn parse_base64_binary_stl_facets(body: &str) -> Option<Vec<StlFacet>> {
    let payload = stl_base64_payload(body)?;
    let bytes = decode_base64_payload(payload)?;
    parse_binary_stl_facets(&bytes)
}

fn stl_base64_payload(body: &str) -> Option<&str> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("data:"))
    {
        return trimmed.split_once(',').map(|(_, payload)| payload.trim());
    }
    if let Some((prefix, payload)) = trimmed.split_once(':') {
        if matches!(
            prefix.trim().to_ascii_lowercase().as_str(),
            "base64" | "binary" | "binary-stl" | "stl-base64"
        ) {
            return Some(payload.trim());
        }
    }
    Some(trimmed)
}

fn parse_binary_stl_facets(bytes: &[u8]) -> Option<Vec<StlFacet>> {
    if bytes.len() < 84 {
        return None;
    }
    let triangle_count = u32::from_le_bytes(bytes.get(80..84)?.try_into().ok()?) as usize;
    let expected_len = 84usize.checked_add(triangle_count.checked_mul(50)?)?;
    if bytes.len() < expected_len {
        return None;
    }
    let mut facets = Vec::with_capacity(triangle_count);
    for triangle_index in 0..triangle_count {
        let base = 84 + triangle_index * 50;
        // Bytes 0-11 of each 50-byte record are the face normal (3 × f32 LE).
        let file_normal = (
            read_le_f32(bytes, base)? as f64,
            read_le_f32(bytes, base + 4)? as f64,
            read_le_f32(bytes, base + 8)? as f64,
        );
        let v0 = (
            read_le_f32(bytes, base + 12)? as f64,
            read_le_f32(bytes, base + 16)? as f64,
            read_le_f32(bytes, base + 20)? as f64,
        );
        let v1 = (
            read_le_f32(bytes, base + 24)? as f64,
            read_le_f32(bytes, base + 28)? as f64,
            read_le_f32(bytes, base + 32)? as f64,
        );
        let v2 = (
            read_le_f32(bytes, base + 36)? as f64,
            read_le_f32(bytes, base + 40)? as f64,
            read_le_f32(bytes, base + 44)? as f64,
        );
        if !(v0.0.is_finite()
            && v0.1.is_finite()
            && v0.2.is_finite()
            && v1.0.is_finite()
            && v1.1.is_finite()
            && v1.2.is_finite()
            && v2.0.is_finite()
            && v2.1.is_finite()
            && v2.2.is_finite())
        {
            return None;
        }
        let normal = stl_validated_normal(file_normal, v0, v1, v2);
        facets.push(StlFacet {
            normal,
            vertices: [v0, v1, v2],
        });
    }
    Some(facets)
}

fn read_le_f32(bytes: &[u8], offset: usize) -> Option<f32> {
    Some(f32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn decode_base64_payload(input: &str) -> Option<Vec<u8>> {
    let mut cleaned = input
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if cleaned.is_empty() {
        return None;
    }
    let pad = (4 - cleaned.len() % 4) % 4;
    cleaned.extend(std::iter::repeat(b'=').take(pad));
    let mut output = Vec::with_capacity(cleaned.len() / 4 * 3);
    for chunk in cleaned.chunks(4) {
        let a = base64_value(chunk[0])?;
        let b = base64_value(chunk[1])?;
        let c_is_padding = chunk[2] == b'=';
        let d_is_padding = chunk[3] == b'=';
        if c_is_padding && !d_is_padding {
            return None;
        }
        let c = if c_is_padding {
            0
        } else {
            base64_value(chunk[2])?
        };
        let d = if d_is_padding {
            0
        } else {
            base64_value(chunk[3])?
        };
        output.push((a << 2) | (b >> 4));
        if !c_is_padding {
            output.push(((b & 0x0f) << 4) | (c >> 2));
        }
        if !d_is_padding {
            output.push(((c & 0x03) << 6) | d);
        }
    }
    Some(output)
}

fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn geojson_bounds(positions: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    positions.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), (x, y)| {
            (min_x.min(*x), max_x.max(*x), min_y.min(*y), max_y.max(*y))
        },
    )
}

fn project_geojson_position(
    (x, y): (f64, f64),
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> (f64, f64) {
    let width = (max_x - min_x).abs().max(0.000_001);
    let height = (max_y - min_y).abs().max(0.000_001);
    let projected_x = 48.0 + ((x - min_x) / width) * 804.0;
    let projected_y = 412.0 - ((y - min_y) / height) * 364.0;
    (projected_x, projected_y)
}
