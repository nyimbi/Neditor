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
    if !matches!(mark.as_str(), "bar" | "line" | "point" | "area") {
        let diagnostic = diag(
            "warning",
            format!("Unsupported Vega-Lite mark for native preview: {mark}"),
            None,
            None,
            Some("Use bar, line, point, or area marks for the native static preview."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">Unsupported Vega-Lite mark</section>".to_string();
    }
    let Some(x_field) = vega_lite_encoding_field(&spec, "x") else {
        return vega_lite_missing_field("x", artifact_diags, diagnostics);
    };
    let Some(y_field) = vega_lite_encoding_field(&spec, "y") else {
        return vega_lite_missing_field("y", artifact_diags, diagnostics);
    };
    let color_field = vega_lite_encoding_field(&spec, "color");
    let y_aggregate = vega_lite_encoding_aggregate(&spec, "y");
    let values = vega_lite_values(
        &spec,
        &x_field,
        &y_field,
        color_field.as_deref(),
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
    let y_title = vega_lite_encoding_title(&spec, "y").unwrap_or(y_field);
    render_vega_lite_chart_svg(title, &mark, &values, &x_title, &y_title)
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
    let vertices = parse_ascii_stl_vertices(body);
    if vertices.is_empty() {
        let diagnostic = diag(
            "warning",
            "STL transform did not contain ASCII vertex data.",
            None,
            None,
            Some("Use ASCII STL fences for static previews, or configure an external STL renderer later."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-stl transform-error\">No ASCII STL vertices found</section>".to_string();
    }
    let positions = vertices
        .iter()
        .map(|(x, y, _)| (*x, *y))
        .collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let triangles = vertices
        .chunks(3)
        .filter(|triangle| triangle.len() == 3)
        .map(|triangle| {
            let points = triangle
                .iter()
                .map(|(x, y, _)| {
                    let (x, y) = project_geojson_position((*x, *y), min_x, max_x, min_y, max_y);
                    format!("{x:.2},{y:.2}")
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!("<polygon points=\"{points}\" fill=\"rgba(39,93,168,.18)\" stroke=\"#275DA8\" stroke-width=\"2\"/>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<svg class=\"transform transform-stl\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"#f8fafc\" stroke=\"#cbd5e1\"/>{triangles}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"#334155\">{} triangles / {} vertices</text></svg>",
        vertices.len() / 3,
        vertices.len()
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

fn vega_lite_encoding_aggregate(spec: &Value, channel: &str) -> Option<String> {
    spec.pointer(&format!("/encoding/{channel}/aggregate"))
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| matches!(value.as_str(), "sum" | "mean" | "average" | "min" | "max"))
}

#[derive(Clone, Debug)]
struct VegaLiteDatum {
    label: String,
    value: f64,
    series: Option<String>,
}

fn vega_lite_values(
    spec: &Value,
    x_field: &str,
    y_field: &str,
    color_field: Option<&str>,
    y_aggregate: Option<&str>,
) -> Vec<VegaLiteDatum> {
    let values = spec
        .pointer("/data/values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let x = row.get(x_field).map(value_to_axis_label)?;
            let y = row
                .get(y_field)
                .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))?;
            let series = color_field
                .and_then(|field| row.get(field))
                .map(value_to_axis_label)
                .filter(|value| !value.trim().is_empty());
            Some(VegaLiteDatum {
                label: x,
                value: y,
                series,
            })
        })
        .collect::<Vec<_>>();
    match y_aggregate {
        Some(aggregate) => aggregate_vega_lite_values(values, aggregate),
        None => values,
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
        "mean" | "average" => Some(values.iter().sum::<f64>() / values.len() as f64),
        "min" => values.iter().copied().reduce(f64::min),
        "max" => values.iter().copied().reduce(f64::max),
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
) -> String {
    let domain = VegaLiteDomain::from_values(values);
    let width = 820usize;
    let height = 340usize;
    let plot_left = 72usize;
    let plot_width = 680usize;
    let labels = unique_vega_labels(values);
    let series = unique_vega_series(values);
    let step = plot_width / labels.len().max(1);
    let mut svg = format!(
        "<svg class=\"transform transform-vega-lite\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\"><text x=\"72\" y=\"34\" font-size=\"18\" fill=\"#111827\">{}</text><line x1=\"72\" y1=\"262\" x2=\"770\" y2=\"262\" stroke=\"#94a3b8\"/><line x1=\"72\" y1=\"54\" x2=\"72\" y2=\"262\" stroke=\"#94a3b8\"/><line class=\"vega-zero-line\" x1=\"72\" y1=\"{:.1}\" x2=\"770\" y2=\"{:.1}\" stroke=\"#64748b\" stroke-dasharray=\"4 3\"/>",
        escape_html(title),
        domain.zero_y,
        domain.zero_y
    );
    render_vega_axis_labels(&mut svg, &labels, plot_left, step);
    render_vega_axis_titles(&mut svg, x_title, y_title);
    if mark == "bar" {
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
            let y = domain.y(datum.value);
            let bar_y = y.min(domain.zero_y);
            let bar_height = (y - domain.zero_y).abs().max(1.0);
            let value_label = format_vega_value(datum.value);
            svg.push_str(&format!(
                "<rect x=\"{x}\" y=\"{bar_y:.1}\" width=\"{bar_width}\" height=\"{bar_height:.1}\" fill=\"{color}\" data-label=\"{}\" data-value=\"{}\"{series_attr}/>",
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
                    Some((x, y, datum.label.as_str(), datum.value))
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
                    .map(|(x, y, _, _)| format!("{x},{y:.1}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let area = match (points.first(), points.last()) {
                    (Some((first_x, _, _, _)), Some((last_x, _, _, _))) => {
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
                    .map(|(x, y, _, _)| format!("{x},{y:.1}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                svg.push_str(&format!(
                    "<polyline points=\"{polyline}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"3\"{series_attr}/>"
                ));
            }
            for (x, y, label, value) in points {
                let label = if series_name.is_empty() {
                    label.to_string()
                } else {
                    format!("{label}: {series_name}")
                };
                let value_label = format_vega_value(value);
                svg.push_str(&format!(
                    "<circle cx=\"{x}\" cy=\"{y:.1}\" r=\"5\" fill=\"{color}\" aria-label=\"{} {}\" data-value=\"{}\"{series_attr}/>",
                    escape_html(&label),
                    escape_html(&value_label),
                    escape_html(&value_label)
                ));
            }
        }
    }
    render_vega_legend(&mut svg, &series);
    svg.push_str("</svg>");
    svg
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
                shapes.push(GeoShape::Line(line));
            }
        }
        "MultiLineString" => {
            if let Some(lines) = object.get("arcs").and_then(Value::as_array).map(|items| {
                items
                    .iter()
                    .filter_map(|line| topojson_line_from_arc_refs(line, decoded_arcs))
            }) {
                shapes.extend(lines.map(GeoShape::Line));
            }
        }
        "Polygon" => {
            if let Some(rings) = object.get("arcs").and_then(Value::as_array).map(|items| {
                items
                    .iter()
                    .filter_map(|ring| topojson_line_from_arc_refs(ring, decoded_arcs))
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

fn parse_ascii_stl_vertices(body: &str) -> Vec<(f64, f64, f64)> {
    body.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "vertex" {
                return None;
            }
            Some((
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
            ))
        })
        .collect()
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
