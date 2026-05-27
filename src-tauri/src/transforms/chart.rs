use crate::escape_html;

pub(crate) fn render_chart_svg(body: &str) -> String {
    let chart = parse_chart_spec(body);
    let title = chart.title.unwrap_or_else(|| "Chart".to_string());
    let chart_type = chart.chart_type.unwrap_or_else(|| "bar".to_string());
    let domain = ChartDomain::from_series(&chart.series, chart.target);
    let height = 300;
    let width = 760;
    let mut svg = format!(
        "<svg class=\"transform transform-chart chart\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\"><title>{}</title><text x=\"80\" y=\"32\" font-size=\"18\" font-weight=\"700\" fill=\"#1f2937\">{}</text>",
        escape_html(&title),
        escape_html(&title)
    );
    match chart_type.to_ascii_lowercase().as_str() {
        "horizontal-bar" | "bar-horizontal" | "barh" => {
            svg.push_str(&render_horizontal_bar_chart_svg(
                &chart.series,
                &domain,
                chart.target,
                chart.target_label.as_deref(),
                chart.value_prefix.as_deref().unwrap_or_default(),
                chart.value_suffix.as_deref().unwrap_or_default(),
            ))
        }
        "line" => svg.push_str(&render_line_chart_svg(
            &chart.series,
            &domain,
            chart.target,
            chart.target_label.as_deref(),
            chart.value_prefix.as_deref().unwrap_or_default(),
            chart.value_suffix.as_deref().unwrap_or_default(),
        )),
        "area" => svg.push_str(&render_area_chart_svg(
            &chart.series,
            &domain,
            chart.target,
            chart.target_label.as_deref(),
            chart.value_prefix.as_deref().unwrap_or_default(),
            chart.value_suffix.as_deref().unwrap_or_default(),
        )),
        "pie" => svg.push_str(&render_pie_chart_svg(
            &chart.values,
            chart.value_prefix.as_deref().unwrap_or_default(),
            chart.value_suffix.as_deref().unwrap_or_default(),
        )),
        "kpi" => svg.push_str(&render_kpi_chart_svg(
            &chart.values,
            chart.target,
            chart.target_label.as_deref(),
            chart.value_prefix.as_deref().unwrap_or_default(),
            chart.value_suffix.as_deref().unwrap_or_default(),
        )),
        _ => svg.push_str(&render_bar_chart_svg(
            &chart.series,
            &domain,
            chart.target,
            chart.target_label.as_deref(),
            chart.value_prefix.as_deref().unwrap_or_default(),
            chart.value_suffix.as_deref().unwrap_or_default(),
        )),
    }
    svg.push_str("</svg>");
    svg
}

#[derive(Debug)]
struct ChartSpec {
    title: Option<String>,
    chart_type: Option<String>,
    values: Vec<(String, f64)>,
    series: Vec<ChartSeries>,
    target: Option<f64>,
    target_label: Option<String>,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
}

#[derive(Clone, Debug)]
struct ChartSeries {
    label: String,
    values: Vec<(String, f64)>,
}

fn parse_chart_spec(body: &str) -> ChartSpec {
    if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(body) {
        let title = yaml_get(&value, "title").and_then(yaml_scalar_string);
        let chart_type = yaml_get(&value, "type").and_then(yaml_scalar_string);
        let target = yaml_get_any(&value, &["target", "goal", "benchmark"]).and_then(yaml_number);
        let target_label = yaml_get_any(&value, &["targetLabel", "goalLabel", "benchmarkLabel"])
            .and_then(yaml_scalar_string);
        let value_prefix =
            yaml_get_any(&value, &["valuePrefix", "prefix"]).and_then(yaml_scalar_string);
        let value_suffix =
            yaml_get_any(&value, &["valueSuffix", "suffix", "unit"]).and_then(yaml_scalar_string);
        let x_key = yaml_get(&value, "x")
            .and_then(yaml_scalar_string)
            .unwrap_or_else(|| "label".to_string());
        let y_key = yaml_get(&value, "y")
            .and_then(yaml_scalar_string)
            .unwrap_or_else(|| "value".to_string());
        let series_defs = chart_series_defs(&value, &y_key);
        let series = yaml_get(&value, "data")
            .and_then(serde_yaml::Value::as_sequence)
            .map(|rows| chart_series_values(rows, &x_key, &series_defs))
            .unwrap_or_default();
        let values = series
            .first()
            .map(|series| series.values.clone())
            .unwrap_or_default();
        if !series.is_empty() {
            return ChartSpec {
                title,
                chart_type,
                values,
                series,
                target,
                target_label,
                value_prefix,
                value_suffix,
            };
        }
    }

    let values = body_values(body);
    ChartSpec {
        title: None,
        chart_type: Some("bar".to_string()),
        target: None,
        target_label: None,
        value_prefix: None,
        value_suffix: None,
        values: values.clone(),
        series: vec![ChartSeries {
            label: "Value".to_string(),
            values,
        }],
    }
}

fn body_values(body: &str) -> Vec<(String, f64)> {
    body.lines()
        .filter_map(|line| line.split_once(':'))
        .filter_map(|(label, value)| {
            value
                .trim()
                .parse::<f64>()
                .ok()
                .map(|value| (label.trim().to_string(), value))
        })
        .collect()
}

fn chart_series_defs(value: &serde_yaml::Value, default_y_key: &str) -> Vec<(String, String)> {
    let defs = yaml_get(value, "series")
        .and_then(serde_yaml::Value::as_sequence)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| match item {
                    serde_yaml::Value::String(key) => Some((key.clone(), key.clone())),
                    serde_yaml::Value::Mapping(_) => {
                        let key = yaml_get_any(item, &["key", "field", "y", "name"])
                            .and_then(yaml_scalar_string)?;
                        let label = yaml_get_any(item, &["label", "title", "name"])
                            .and_then(yaml_scalar_string)
                            .unwrap_or_else(|| key.clone());
                        Some((key, label))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if defs.is_empty() {
        vec![(default_y_key.to_string(), default_y_key.to_string())]
    } else {
        defs
    }
}

fn chart_series_values(
    rows: &[serde_yaml::Value],
    x_key: &str,
    series_defs: &[(String, String)],
) -> Vec<ChartSeries> {
    series_defs
        .iter()
        .filter_map(|(key, label)| {
            let values = rows
                .iter()
                .filter_map(|row| {
                    let label = yaml_get(row, x_key).and_then(yaml_scalar_string)?;
                    let value = yaml_get(row, key).and_then(yaml_number)?;
                    Some((label, value))
                })
                .collect::<Vec<_>>();
            (!values.is_empty()).then_some(ChartSeries {
                label: label.clone(),
                values,
            })
        })
        .collect()
}

fn yaml_get<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    match value {
        serde_yaml::Value::Mapping(map) => {
            let key = serde_yaml::Value::String(key.to_string());
            map.get(&key)
        }
        _ => None,
    }
}

fn yaml_get_any<'a>(value: &'a serde_yaml::Value, keys: &[&str]) -> Option<&'a serde_yaml::Value> {
    keys.iter().find_map(|key| yaml_get(value, key))
}

fn yaml_scalar_string(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(text) => Some(text.clone()),
        serde_yaml::Value::Number(number) => Some(number.to_string()),
        serde_yaml::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn yaml_number(value: &serde_yaml::Value) -> Option<f64> {
    match value {
        serde_yaml::Value::Number(number) => number.as_f64(),
        serde_yaml::Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
}

#[derive(Debug)]
struct ChartDomain {
    min: f64,
    max: f64,
    zero_y: f64,
}

impl ChartDomain {
    fn from_series(series: &[ChartSeries], target: Option<f64>) -> Self {
        let values = series
            .iter()
            .flat_map(|series| series.values.iter().map(|(_, value)| *value))
            .collect::<Vec<_>>();
        let mut min = values
            .iter()
            .copied()
            .reduce(f64::min)
            .unwrap_or(0.0)
            .min(0.0);
        let mut max = values
            .iter()
            .copied()
            .reduce(f64::max)
            .unwrap_or(1.0)
            .max(0.0);
        if let Some(target) = target {
            min = min.min(target);
            max = max.max(target);
        }
        if (max - min).abs() < f64::EPSILON {
            max = min + 1.0;
        }
        Self {
            min,
            max,
            zero_y: chart_value_y(0.0, min, max),
        }
    }

    fn y(&self, value: f64) -> f64 {
        chart_value_y(value, self.min, self.max)
    }
}

fn chart_value_y(value: f64, min: f64, max: f64) -> f64 {
    let plot_top = 70.0;
    let plot_bottom = 240.0;
    let span = (max - min).max(1.0);
    plot_bottom - ((value - min) / span) * (plot_bottom - plot_top)
}

fn chart_value_x(value: f64, min: f64, max: f64) -> f64 {
    let plot_left = 180.0;
    let plot_right = 710.0;
    let span = (max - min).max(1.0);
    plot_left + ((value - min) / span) * (plot_right - plot_left)
}

fn render_bar_chart_svg(
    series: &[ChartSeries],
    domain: &ChartDomain,
    target: Option<f64>,
    target_label: Option<&str>,
    value_prefix: &str,
    value_suffix: &str,
) -> String {
    let labels = chart_labels(series);
    let series_count = series.len().max(1);
    let group_width = if labels.is_empty() {
        1
    } else {
        600 / labels.len().max(1)
    };
    let bar_width = if series_count <= 1 {
        group_width.saturating_sub(10).max(4)
    } else {
        (group_width.saturating_sub(18) / series_count).max(4)
    };
    let mut svg = format!(
        "<line x1=\"70\" y1=\"{:.1}\" x2=\"710\" y2=\"{:.1}\" stroke=\"#94a3b8\"/><text x=\"72\" y=\"{:.1}\" font-size=\"11\" fill=\"#64748b\">0</text>",
        domain.zero_y,
        domain.zero_y,
        (domain.zero_y - 6.0).clamp(66.0, 252.0)
    );
    if let Some(target) = target {
        svg.push_str(&render_chart_target_line(
            domain,
            target,
            target_label,
            value_prefix,
            value_suffix,
        ));
    }
    for (label_index, label) in labels.iter().enumerate() {
        let label_x = 80 + label_index * group_width;
        svg.push_str(&format!(
            "<text x=\"{}\" y=\"264\" font-size=\"12\">{}</text>",
            label_x,
            escape_html(label)
        ));
        for (series_index, chart_series) in series.iter().enumerate() {
            let Some(value) = chart_value_for_label(&chart_series.values, label) else {
                continue;
            };
            let value_y = domain.y(value);
            let y = value_y.min(domain.zero_y);
            let bar_height = (domain.zero_y - value_y).abs().max(1.0);
            let x = if series_count <= 1 {
                label_x
            } else {
                label_x + 8 + series_index * bar_width
            };
            let value_label = format_chart_value(value, value_prefix, value_suffix);
            let value_label_y = if value >= 0.0 {
                y - 8.0
            } else {
                y + bar_height + 16.0
            };
            let color = if value < 0.0 && series_count == 1 {
                "#be123c"
            } else {
                chart_color(series_index)
            };
            let series_attr = if series_count <= 1 {
                String::new()
            } else {
                format!(" data-series=\"{}\"", escape_html(&chart_series.label))
            };
            svg.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y:.1}\" width=\"{bar_width}\" height=\"{bar_height:.1}\" fill=\"{color}\"{series_attr}/><text x=\"{x}\" y=\"{value_label_y:.1}\" font-size=\"12\">{}</text>",
                escape_html(&value_label)
            ));
        }
    }
    render_chart_series_legend(&mut svg, series);
    svg
}

fn render_horizontal_bar_chart_svg(
    series: &[ChartSeries],
    domain: &ChartDomain,
    target: Option<f64>,
    target_label: Option<&str>,
    value_prefix: &str,
    value_suffix: &str,
) -> String {
    let labels = chart_labels(series);
    let series_count = series.len().max(1);
    let plot_top = 70.0;
    let plot_bottom = 240.0;
    let plot_height = plot_bottom - plot_top;
    let zero_x = chart_value_x(0.0, domain.min, domain.max);
    let row_height = if labels.is_empty() {
        plot_height
    } else {
        (plot_height / labels.len().max(1) as f64).max(18.0)
    };
    let bar_height = if series_count <= 1 {
        (row_height - 10.0).max(6.0)
    } else {
        ((row_height - 12.0) / series_count as f64).max(5.0)
    };
    let mut svg = format!(
        "<line class=\"chart-horizontal-zero\" x1=\"{zero_x:.1}\" y1=\"60\" x2=\"{zero_x:.1}\" y2=\"248\" stroke=\"#94a3b8\"/><text x=\"{:.1}\" y=\"258\" font-size=\"11\" fill=\"#64748b\">0</text>",
        (zero_x + 4.0).clamp(184.0, 690.0)
    );
    if let Some(target) = target {
        svg.push_str(&render_chart_target_vertical_line(
            domain,
            target,
            target_label,
            value_prefix,
            value_suffix,
        ));
    }
    for (label_index, label) in labels.iter().enumerate() {
        let row_y = plot_top + label_index as f64 * row_height;
        let label_y = row_y + row_height / 2.0 + 4.0;
        svg.push_str(&format!(
            "<text class=\"chart-horizontal-label\" x=\"30\" y=\"{label_y:.1}\" font-size=\"12\" fill=\"#334155\">{}</text>",
            escape_html(label)
        ));
        for (series_index, chart_series) in series.iter().enumerate() {
            let Some(value) = chart_value_for_label(&chart_series.values, label) else {
                continue;
            };
            let value_x = chart_value_x(value, domain.min, domain.max);
            let x = value_x.min(zero_x);
            let width = (value_x - zero_x).abs().max(1.0);
            let y = if series_count <= 1 {
                row_y + (row_height - bar_height) / 2.0
            } else {
                row_y + 6.0 + series_index as f64 * bar_height
            };
            let value_label = format_chart_value(value, value_prefix, value_suffix);
            let value_label_x = if value >= 0.0 {
                (x + width + 6.0).clamp(184.0, 722.0)
            } else {
                (x - 48.0).clamp(184.0, 722.0)
            };
            let color = if value < 0.0 && series_count == 1 {
                "#be123c"
            } else {
                chart_color(series_index)
            };
            let series_attr = if series_count <= 1 {
                String::new()
            } else {
                format!(" data-series=\"{}\"", escape_html(&chart_series.label))
            };
            svg.push_str(&format!(
                "<rect class=\"chart-horizontal-bar\" x=\"{x:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{bar_height:.1}\" fill=\"{color}\"{series_attr}/><text x=\"{value_label_x:.1}\" y=\"{:.1}\" font-size=\"12\" fill=\"#1f2937\">{}</text>",
                y + bar_height - 2.0,
                escape_html(&value_label)
            ));
        }
    }
    render_chart_series_legend(&mut svg, series);
    svg
}

fn render_line_chart_svg(
    series: &[ChartSeries],
    domain: &ChartDomain,
    target: Option<f64>,
    target_label: Option<&str>,
    value_prefix: &str,
    value_suffix: &str,
) -> String {
    let mut svg = format!(
        "<line x1=\"70\" y1=\"{:.1}\" x2=\"710\" y2=\"{:.1}\" stroke=\"#94a3b8\"/>",
        domain.zero_y, domain.zero_y
    );
    if let Some(target) = target {
        svg.push_str(&render_chart_target_line(
            domain,
            target,
            target_label,
            value_prefix,
            value_suffix,
        ));
    }
    let labels = chart_labels(series);
    for (series_index, chart_series) in series.iter().enumerate() {
        let plotted = chart_plot_points_for_labels(&chart_series.values, &labels, domain);
        let points = plotted
            .iter()
            .map(|(_, _, x, y)| format!("{x:.1},{y:.1}"))
            .collect::<Vec<_>>()
            .join(" ");
        let color = chart_color(series_index);
        let series_attr = if series.len() <= 1 {
            String::new()
        } else {
            format!(" data-series=\"{}\"", escape_html(&chart_series.label))
        };
        svg.push_str(&format!(
            "<polyline fill=\"none\" stroke=\"{color}\" stroke-width=\"3\" points=\"{points}\"{series_attr}/>"
        ));
        for (label, value, x, y) in plotted {
            let value_label = format_chart_value(value, value_prefix, value_suffix);
            svg.push_str(&format!(
                "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"{color}\" aria-label=\"{} {}\"{series_attr}/><text x=\"{x:.1}\" y=\"264\" font-size=\"12\">{}</text><text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"12\">{}</text>",
                escape_html(&chart_series.label),
                escape_html(label),
                escape_html(label),
                y - 10.0,
                escape_html(&value_label)
            ));
        }
    }
    render_chart_series_legend(&mut svg, series);
    svg
}

fn render_area_chart_svg(
    series: &[ChartSeries],
    domain: &ChartDomain,
    target: Option<f64>,
    target_label: Option<&str>,
    value_prefix: &str,
    value_suffix: &str,
) -> String {
    if series.iter().all(|series| series.values.is_empty()) {
        return "<line x1=\"70\" y1=\"240\" x2=\"710\" y2=\"240\" stroke=\"#94a3b8\"/>".to_string();
    }
    let baseline = domain.zero_y;
    let mut svg = format!(
        "<line x1=\"70\" y1=\"{baseline:.1}\" x2=\"710\" y2=\"{baseline:.1}\" stroke=\"#94a3b8\"/>"
    );
    if let Some(target) = target {
        svg.push_str(&render_chart_target_line(
            domain,
            target,
            target_label,
            value_prefix,
            value_suffix,
        ));
    }
    let labels = chart_labels(series);
    for (series_index, chart_series) in series.iter().enumerate() {
        let plotted = chart_plot_points_for_labels(&chart_series.values, &labels, domain);
        if plotted.is_empty() {
            continue;
        }
        let points = plotted
            .iter()
            .map(|(_, _, x, y)| format!("{x:.1},{y:.1}"))
            .collect::<Vec<_>>()
            .join(" ");
        let first_x = plotted.first().map(|(_, _, x, _)| *x).unwrap_or(80.0);
        let last_x = plotted.last().map(|(_, _, x, _)| *x).unwrap_or(first_x);
        let area_points = format!("{first_x:.1},{baseline:.1} {points} {last_x:.1},{baseline:.1}");
        let color = chart_color(series_index);
        let series_attr = if series.len() <= 1 {
            String::new()
        } else {
            format!(" data-series=\"{}\"", escape_html(&chart_series.label))
        };
        svg.push_str(&format!(
            "<polygon points=\"{area_points}\" fill=\"{color}\" opacity=\"0.16\"{series_attr}/><polyline fill=\"none\" stroke=\"{color}\" stroke-width=\"3\" points=\"{points}\"{series_attr}/>"
        ));
        for (label, value, x, y) in plotted {
            let value_label = format_chart_value(value, value_prefix, value_suffix);
            svg.push_str(&format!(
                "<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"{color}\" aria-label=\"{} {}\"{series_attr}/><text x=\"{x:.1}\" y=\"264\" font-size=\"12\">{}</text><text x=\"{x:.1}\" y=\"{:.1}\" font-size=\"12\">{}</text>",
                escape_html(&chart_series.label),
                escape_html(label),
                escape_html(label),
                y - 10.0,
                escape_html(&value_label)
            ));
        }
    }
    render_chart_series_legend(&mut svg, series);
    svg
}

fn render_pie_chart_svg(
    values: &[(String, f64)],
    value_prefix: &str,
    value_suffix: &str,
) -> String {
    let positive_values = values
        .iter()
        .filter(|(_, value)| *value > 0.0)
        .collect::<Vec<_>>();
    let total = positive_values.iter().map(|(_, value)| *value).sum::<f64>();
    if total <= 0.0 {
        return "<text x=\"80\" y=\"160\" font-size=\"14\" fill=\"#64748b\">No positive values</text>"
            .to_string();
    }

    let cx = 260.0;
    let cy = 154.0;
    let radius = 88.0;
    let mut start = -std::f64::consts::FRAC_PI_2;
    let mut svg = String::new();
    for (index, (label, value)) in positive_values.iter().enumerate() {
        let span = (*value / total) * std::f64::consts::TAU;
        let end = start + span;
        let x1 = cx + radius * start.cos();
        let y1 = cy + radius * start.sin();
        let x2 = cx + radius * end.cos();
        let y2 = cy + radius * end.sin();
        let large_arc = if span > std::f64::consts::PI { 1 } else { 0 };
        let color = chart_color(index);
        svg.push_str(&format!(
            "<path d=\"M {cx:.1} {cy:.1} L {x1:.1} {y1:.1} A {radius:.1} {radius:.1} 0 {large_arc} 1 {x2:.1} {y2:.1} Z\" fill=\"{color}\" stroke=\"#ffffff\" stroke-width=\"2\"/>"
        ));
        let legend_y = 78 + index * 26;
        svg.push_str(&format!(
            "<rect x=\"420\" y=\"{}\" width=\"14\" height=\"14\" fill=\"{color}\"/><text x=\"442\" y=\"{}\" font-size=\"13\" fill=\"#1f2937\">{}: {} ({:.1}%)</text>",
            legend_y,
            legend_y + 12,
            escape_html(label),
            escape_html(&format_chart_value(*value, value_prefix, value_suffix)),
            (*value / total) * 100.0
        ));
        start = end;
    }
    svg
}

fn render_kpi_chart_svg(
    values: &[(String, f64)],
    target: Option<f64>,
    target_label: Option<&str>,
    value_prefix: &str,
    value_suffix: &str,
) -> String {
    let Some((label, value)) = values.first() else {
        return "<text x=\"80\" y=\"160\" font-size=\"14\" fill=\"#64748b\">No KPI value</text>"
            .to_string();
    };
    let secondary = values
        .get(1)
        .map(|(label, value)| {
            let value = format_chart_value(*value, value_prefix, value_suffix);
            format!(
                "<text x=\"84\" y=\"214\" font-size=\"16\" fill=\"#475569\">{}: {}</text>",
                escape_html(label),
                escape_html(&value)
            )
        })
        .unwrap_or_default();
    let target_note = target
        .map(|target| {
            let label = target_label.unwrap_or("Target");
            let value = format_chart_value(target, value_prefix, value_suffix);
            format!(
                "<text x=\"420\" y=\"214\" font-size=\"16\" fill=\"#334155\">{}: {}</text>",
                escape_html(label),
                escape_html(&value)
            )
        })
        .unwrap_or_default();
    let value = format_chart_value(*value, value_prefix, value_suffix);
    format!(
        "<rect x=\"72\" y=\"68\" width=\"620\" height=\"174\" rx=\"10\" fill=\"#ecfeff\" stroke=\"#67e8f9\"/><text x=\"84\" y=\"116\" font-size=\"16\" fill=\"#475569\">{}</text><text x=\"84\" y=\"178\" font-size=\"56\" font-weight=\"700\" fill=\"#0f766e\">{}</text>{secondary}{target_note}",
        escape_html(label),
        escape_html(&value)
    )
}

fn chart_plot_points_for_labels<'a>(
    values: &'a [(String, f64)],
    labels: &[String],
    domain: &ChartDomain,
) -> Vec<(&'a str, f64, f64, f64)> {
    let step = if labels.len() <= 1 {
        1.0
    } else {
        600.0 / (labels.len() - 1) as f64
    };
    values
        .iter()
        .filter_map(|(label, value)| {
            let index = labels.iter().position(|candidate| candidate == label)?;
            let x = 80.0 + index as f64 * step;
            let y = domain.y(*value);
            Some((label.as_str(), *value, x, y))
        })
        .collect()
}

fn chart_labels(series: &[ChartSeries]) -> Vec<String> {
    let mut labels = Vec::new();
    for chart_series in series {
        for (label, _) in &chart_series.values {
            if !labels.iter().any(|candidate| candidate == label) {
                labels.push(label.clone());
            }
        }
    }
    labels
}

fn chart_value_for_label(values: &[(String, f64)], label: &str) -> Option<f64> {
    values
        .iter()
        .find_map(|(candidate, value)| (candidate == label).then_some(*value))
}

fn render_chart_series_legend(svg: &mut String, series: &[ChartSeries]) {
    if series.len() <= 1 {
        return;
    }
    for (index, chart_series) in series.iter().enumerate() {
        let x = 540usize;
        let y = 54 + index * 20;
        let color = chart_color(index);
        svg.push_str(&format!(
            "<g class=\"chart-legend-item\"><rect x=\"{x}\" y=\"{}\" width=\"12\" height=\"12\" fill=\"{color}\"/><text x=\"{}\" y=\"{y}\" font-size=\"12\" fill=\"#334155\">{}</text></g>",
            y.saturating_sub(10),
            x + 18,
            escape_html(&chart_series.label)
        ));
    }
}

fn render_chart_target_line(
    domain: &ChartDomain,
    target: f64,
    target_label: Option<&str>,
    value_prefix: &str,
    value_suffix: &str,
) -> String {
    let y = domain.y(target);
    let label = target_label.unwrap_or("Target");
    let value = format_chart_value(target, value_prefix, value_suffix);
    format!(
        "<line class=\"chart-target-line\" x1=\"70\" y1=\"{y:.1}\" x2=\"710\" y2=\"{y:.1}\" stroke=\"#b45309\" stroke-dasharray=\"6 4\"/><text class=\"chart-target-label\" x=\"540\" y=\"{:.1}\" font-size=\"12\" fill=\"#92400e\">{}: {}</text>",
        (y - 8.0).clamp(66.0, 252.0),
        escape_html(label),
        escape_html(&value)
    )
}

fn render_chart_target_vertical_line(
    domain: &ChartDomain,
    target: f64,
    target_label: Option<&str>,
    value_prefix: &str,
    value_suffix: &str,
) -> String {
    let x = chart_value_x(target, domain.min, domain.max);
    let label = target_label.unwrap_or("Target");
    let value = format_chart_value(target, value_prefix, value_suffix);
    format!(
        "<line class=\"chart-target-line chart-target-vertical-line\" x1=\"{x:.1}\" y1=\"60\" x2=\"{x:.1}\" y2=\"248\" stroke=\"#b45309\" stroke-dasharray=\"6 4\"/><text class=\"chart-target-label\" x=\"{:.1}\" y=\"62\" font-size=\"12\" fill=\"#92400e\">{}: {}</text>",
        (x + 6.0).clamp(184.0, 590.0),
        escape_html(label),
        escape_html(&value)
    )
}

fn format_chart_value(value: f64, prefix: &str, suffix: &str) -> String {
    let number = if value.fract().abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        let formatted = format!("{value:.2}");
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    };
    format!("{prefix}{number}{suffix}")
}

fn chart_color(index: usize) -> &'static str {
    const COLORS: [&str; 8] = [
        "#275DA8", "#0f766e", "#b45309", "#7c3aed", "#be123c", "#047857", "#0369a1", "#a16207",
    ];
    COLORS[index % COLORS.len()]
}
