use crate::{
    compiler_support::fenced_code_marker,
    document_ast::{extract_label, extract_quoted_attribute},
    escape_html,
    layout::layout_css_style,
};

pub(crate) fn render_figures(markdown: &str) -> String {
    let mut fence_marker: Option<String> = None;
    markdown
        .lines()
        .map(|line| {
            if let Some(ref marker) = fence_marker {
                if line.trim_start().starts_with(marker.as_str()) {
                    fence_marker = None;
                }
                return line.to_string();
            }
            if let Some(marker) = fenced_code_marker(line) {
                fence_marker = Some(marker);
                return line.to_string();
            }
            render_figure_line(line).unwrap_or_else(|| line.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_figure_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let alt_start = trimmed.strip_prefix("![")?;
    let (alt, after_alt) = alt_start.split_once("](")?;
    let (src, after_src) = after_alt.split_once(')')?;
    let attrs = after_src.trim();
    if !attrs.starts_with("{#fig:") || !attrs.ends_with('}') {
        return None;
    }
    let id = extract_label(attrs)?;
    let caption = extract_quoted_attribute(attrs, "caption").unwrap_or_else(|| alt.to_string());
    let float = figure_float(attrs);
    let fit = figure_fit(attrs);
    let position = figure_position(attrs);
    let float_class = float
        .as_deref()
        .map(|value| format!(" figure-float-{value}"))
        .unwrap_or_default();
    let fit_class = fit
        .as_deref()
        .map(|value| format!(" figure-fit-{value}"))
        .unwrap_or_default();
    let position_class = position
        .as_deref()
        .map(|value| format!(" figure-position-{value}"))
        .unwrap_or_default();
    let float_attr = float
        .as_deref()
        .map(|value| format!(" data-float=\"{}\"", escape_html(value)))
        .unwrap_or_default();
    let fit_attr = fit
        .as_deref()
        .map(|value| format!(" data-fit=\"{}\"", escape_html(value)))
        .unwrap_or_default();
    let position_attr = position
        .as_deref()
        .map(|value| format!(" data-position=\"{}\"", escape_html(value)))
        .unwrap_or_default();
    Some(format!(
        "<figure id=\"{}\" class=\"figure{}{}{}\"{}{}{}><img src=\"{}\" alt=\"{}\"/><figcaption>{}</figcaption></figure>",
        escape_html(&id),
        float_class,
        fit_class,
        position_class,
        float_attr,
        fit_attr,
        position_attr,
        escape_html(src),
        escape_html(alt),
        escape_html(&caption)
    ))
}

fn figure_float(attrs: &str) -> Option<String> {
    extract_quoted_attribute(attrs, "float")
        .or_else(|| extract_quoted_attribute(attrs, "align"))
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| matches!(value.as_str(), "left" | "right"))
}

fn figure_fit(attrs: &str) -> Option<String> {
    extract_quoted_attribute(attrs, "fit")
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| matches!(value.as_str(), "cover" | "contain"))
}

fn figure_position(attrs: &str) -> Option<String> {
    extract_quoted_attribute(attrs, "position")
        .or_else(|| extract_quoted_attribute(attrs, "focus"))
        .map(|value| value.trim().to_ascii_lowercase().replace('_', "-"))
        .filter(|value| {
            matches!(
                value.as_str(),
                "center"
                    | "top"
                    | "bottom"
                    | "left"
                    | "right"
                    | "top-left"
                    | "top-right"
                    | "bottom-left"
                    | "bottom-right"
            )
        })
}

pub(crate) fn render_equations(markdown: &str) -> String {
    let mut output = String::new();
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut equation_number = 1usize;
    let mut fence_marker: Option<String> = None;
    while index < lines.len() {
        let line = lines[index];
        if let Some(ref marker) = fence_marker {
            output.push_str(line);
            output.push('\n');
            if line.trim_start().starts_with(marker.as_str()) {
                fence_marker = None;
            }
            index += 1;
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            output.push_str(line);
            output.push('\n');
            fence_marker = Some(marker);
            index += 1;
            continue;
        }
        let trimmed = line.trim();
        if trimmed == "$$" || trimmed.starts_with("$$ ") {
            let mut body = String::new();
            let mut label = String::new();
            let mut caption = String::new();
            index += 1;
            while index < lines.len() {
                let equation_line = lines[index];
                let equation_trimmed = equation_line.trim();
                if equation_trimmed.starts_with("$$") {
                    label = extract_label(equation_trimmed).unwrap_or_default();
                    caption =
                        extract_quoted_attribute(equation_trimmed, "caption").unwrap_or_default();
                    index += 1;
                    break;
                }
                body.push_str(equation_line);
                body.push('\n');
                index += 1;
            }
            let id = if label.is_empty() {
                format!("eq:{equation_number}")
            } else {
                label
            };
            let latex = body.trim();
            let rendered_caption = if caption.trim().is_empty() {
                format!("Equation {equation_number}")
            } else {
                format!("Equation {equation_number}: {}", caption.trim())
            };
            output.push_str(&format!(
                "<figure class=\"equation\" id=\"{}\" data-caption=\"{}\"><div class=\"math-rendered math-display\" data-katex=\"{}\" data-katex-display role=\"math\" aria-label=\"{}\">{}</div><details class=\"math-source\"><summary>LaTeX source</summary><pre><code>{}</code></pre></details><figcaption>{}</figcaption></figure>\n",
                escape_html(&id),
                escape_html(caption.trim()),
                escape_html(latex),
                escape_html(latex),
                escape_html(latex),
                escape_html(latex),
                escape_html(&rendered_caption)
            ));
            equation_number += 1;
        } else {
            output.push_str(&render_inline_math(line));
            output.push('\n');
            index += 1;
        }
    }
    output
}

fn render_inline_math(line: &str) -> String {
    let mut output = String::new();
    let mut rest = line;
    while let Some(start) = rest.find("\\(") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        if let Some(end) = after_start.find("\\)") {
            let math = &after_start[..end];
            output.push_str(&format!(
                "<span class=\"math math-inline\" role=\"math\" aria-label=\"{}\"><span class=\"math-rendered\" data-katex=\"{}\">{}</span><code class=\"math-source-inline\">{}</code></span>",
                escape_html(math),
                escape_html(math),
                escape_html(math),
                escape_html(math)
            ));
            rest = &after_start[end + 2..];
        } else {
            output.push_str(&rest[start..]);
            rest = "";
        }
    }
    output.push_str(rest);
    output
}

pub(crate) fn render_callouts(markdown: &str) -> String {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    let mut fence_marker: Option<String> = None;
    while index < lines.len() {
        let line = lines[index];
        if let Some(ref marker) = fence_marker {
            output.push(line.to_string());
            if line.trim_start().starts_with(marker.as_str()) {
                fence_marker = None;
            }
            index += 1;
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            output.push(line.to_string());
            fence_marker = Some(marker);
            index += 1;
            continue;
        }
        let trimmed = line.trim_start();
        let Some(after_marker) = trimmed.strip_prefix("> [!") else {
            output.push(line.to_string());
            index += 1;
            continue;
        };
        let Some(marker_end) = after_marker.find(']') else {
            output.push(line.to_string());
            index += 1;
            continue;
        };
        let callout_type = after_marker[..marker_end].trim().to_ascii_lowercase();
        if callout_type.is_empty()
            || !callout_type
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        {
            output.push(line.to_string());
            index += 1;
            continue;
        }
        let title = after_marker[marker_end + 1..].trim();
        let title = if title.is_empty() {
            callout_type.to_ascii_uppercase()
        } else {
            title.to_string()
        };
        index += 1;
        let mut body_lines = Vec::new();
        while index < lines.len() {
            let quoted = lines[index].trim_start();
            if !quoted.starts_with('>') {
                break;
            }
            body_lines.push(strip_callout_quote(quoted));
            index += 1;
        }
        let body = body_lines
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| escape_html(line.trim()))
            .collect::<Vec<_>>()
            .join("<br/>");
        output.push(format!(
            "<aside class=\"callout callout-{}\" data-callout=\"{}\"><strong>{}</strong><p>{}</p></aside>",
            escape_html(&callout_type),
            escape_html(&callout_type),
            escape_html(&title),
            body
        ));
    }
    output.join("\n")
}

fn strip_callout_quote(line: &str) -> String {
    line.strip_prefix('>')
        .map(str::trim_start)
        .unwrap_or(line)
        .to_string()
}

pub(crate) fn render_layout_tokens(markdown: &str) -> String {
    let mut fence_marker: Option<String> = None;
    markdown
        .lines()
        .map(|line| {
            if let Some(ref marker) = fence_marker {
                if line.trim_start().starts_with(marker.as_str()) {
                    fence_marker = None;
                }
                return line.to_string();
            }
            if let Some(marker) = fenced_code_marker(line) {
                fence_marker = Some(marker);
                return line.to_string();
            }
            let trimmed = line.trim();
            if trimmed == "{{page-break}}" {
                "<div class=\"page-break\" data-layout=\"page-break\"></div>".to_string()
            } else if let Some(rest) = trimmed.strip_prefix("{{section-break") {
                let attributes = rest.trim_end_matches("}}").trim();
                let style = layout_css_style(attributes);
                format!(
                    "<section class=\"section-break\" data-layout=\"section-break\" data-options=\"{}\"{}></section>",
                    escape_html(attributes),
                    style_attribute(&style)
                )
            } else if let Some(rest) = trimmed.strip_prefix("{{slide") {
                let attributes = rest.trim_end_matches("}}").trim();
                format!(
                    "<section class=\"slide-break\" data-layout=\"slide\" data-options=\"{}\"></section>",
                    escape_html(attributes)
                )
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn render_layout_block_html(body: &str) -> String {
    let style = layout_css_style(body);
    format!(
        "<section class=\"layout-directive\" data-layout=\"layout\" data-options=\"{}\"{}></section>",
        escape_html(body.trim()),
        style_attribute(&style)
    )
}

fn style_attribute(style: &str) -> String {
    if style.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", escape_html(style))
    }
}
