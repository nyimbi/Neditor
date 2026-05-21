use crate::{
    compiler_support::fenced_code_marker,
    document_ast::{extract_label, extract_quoted_attribute},
    escape_html,
    layout::layout_css_style,
};

pub(crate) fn render_figures(markdown: &str) -> String {
    let mut fence_marker = None;
    markdown
        .lines()
        .map(|line| {
            if let Some(marker) = fence_marker {
                if line.trim_start().starts_with(marker) {
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
    let mut fence_marker = None;
    while index < lines.len() {
        let line = lines[index];
        if let Some(marker) = fence_marker {
            output.push_str(line);
            output.push('\n');
            if line.trim_start().starts_with(marker) {
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
                "<figure class=\"equation\" id=\"{}\" data-caption=\"{}\"><div class=\"math-rendered math-display\" role=\"math\" aria-label=\"{}\">{}</div><details class=\"math-source\"><summary>LaTeX</summary><pre><code>{}</code></pre></details><figcaption>{}</figcaption></figure>\n",
                escape_html(&id),
                escape_html(caption.trim()),
                escape_html(latex),
                render_latex_visual(latex),
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
                "<span class=\"math math-inline\" role=\"math\" aria-label=\"{}\"><span class=\"math-rendered\">{}</span><code class=\"math-source-inline\">{}</code></span>",
                escape_html(math),
                render_latex_visual(math),
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

fn render_latex_visual(input: &str) -> String {
    render_latex_expr(input.trim())
}

fn render_latex_expr(input: &str) -> String {
    if let Some(matrix) = render_latex_matrix(input) {
        return matrix;
    }
    let mut output = String::new();
    let mut index = 0usize;
    while index < input.len() {
        let rest = &input[index..];
        if rest.starts_with("\\frac") {
            if let Some((numerator, after_numerator)) =
                parse_latex_braced(input, index + "\\frac".len())
            {
                if let Some((denominator, after_denominator)) =
                    parse_latex_braced(input, after_numerator)
                {
                    output.push_str("<span class=\"math-frac\"><span>");
                    output.push_str(&render_latex_expr(&numerator));
                    output.push_str("</span><span>");
                    output.push_str(&render_latex_expr(&denominator));
                    output.push_str("</span></span>");
                    index = after_denominator;
                    continue;
                }
            }
        } else if rest.starts_with("\\begin{") {
            if let Some((matrix, after_matrix)) = render_latex_matrix_prefix(input, index) {
                output.push_str(&matrix);
                index = after_matrix;
                continue;
            }
        } else if rest.starts_with("\\sqrt") {
            if let Some((radicand, after_radicand)) =
                parse_latex_braced(input, index + "\\sqrt".len())
            {
                output.push_str("<span class=\"math-sqrt\">");
                output.push_str(&render_latex_expr(&radicand));
                output.push_str("</span>");
                index = after_radicand;
                continue;
            }
        } else if let Some(command) = rest.strip_prefix('\\').and_then(latex_command) {
            output.push_str(&escape_html(command.symbol));
            index += command.consumed + 1;
            continue;
        }

        let Some(ch) = rest.chars().next() else {
            break;
        };
        if ch == '&' {
            output.push_str("<span class=\"math-align-separator\"> </span>");
            index += ch.len_utf8();
            continue;
        }
        if ch == '^' || ch == '_' {
            if let Some((script, after_script)) = parse_latex_script(input, index + ch.len_utf8()) {
                output.push_str(if ch == '^' { "<sup>" } else { "<sub>" });
                output.push_str(&render_latex_expr(&script));
                output.push_str(if ch == '^' { "</sup>" } else { "</sub>" });
                index = after_script;
                continue;
            }
        }
        output.push_str(&escape_html(&ch.to_string()));
        index += ch.len_utf8();
    }
    output
}

fn render_latex_matrix(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let (environment, body) = parse_latex_environment(trimmed)?;
    let bracket_class = matrix_bracket_class(environment)?;
    Some(render_latex_matrix_body(bracket_class, body))
}

fn render_latex_matrix_prefix(input: &str, start: usize) -> Option<(String, usize)> {
    let (environment, body, after_environment) = parse_latex_environment_prefix(input, start)?;
    let bracket_class = matrix_bracket_class(environment)?;
    Some((
        render_latex_matrix_body(bracket_class, body),
        after_environment,
    ))
}

fn matrix_bracket_class(environment: &str) -> Option<&'static str> {
    match environment {
        "matrix" => Some("matrix-none"),
        "pmatrix" => Some("matrix-round"),
        "bmatrix" => Some("matrix-square"),
        "vmatrix" => Some("matrix-vertical"),
        _ => None,
    }
}

fn render_latex_matrix_body(bracket_class: &str, body: &str) -> String {
    let rows = body
        .split("\\\\")
        .map(str::trim)
        .filter(|row| !row.is_empty())
        .map(|row| {
            row.split('&')
                .map(str::trim)
                .map(render_latex_expr)
                .map(|cell| format!("<td>{cell}</td>"))
                .collect::<Vec<_>>()
                .join("")
        })
        .map(|row| format!("<tr>{row}</tr>"))
        .collect::<Vec<_>>();
    if rows.is_empty() {
        String::new()
    } else {
        format!(
            "<span class=\"math-matrix {bracket_class}\"><table><tbody>{}</tbody></table></span>",
            rows.join("")
        )
    }
}

fn parse_latex_environment(input: &str) -> Option<(&str, &str)> {
    let begin_marker = "\\begin{";
    let after_begin = input.strip_prefix(begin_marker)?;
    let (environment, after_environment) = after_begin.split_once('}')?;
    let end_marker = format!("\\end{{{environment}}}");
    let body = after_environment.strip_suffix(&end_marker)?;
    Some((environment, body))
}

fn parse_latex_environment_prefix(input: &str, start: usize) -> Option<(&str, &str, usize)> {
    let begin_marker = "\\begin{";
    let after_begin_index = start + begin_marker.len();
    let after_begin = input.get(after_begin_index..)?;
    let (environment, after_environment) = after_begin.split_once('}')?;
    let body_start = after_begin_index + environment.len() + 1;
    let end_marker = format!("\\end{{{environment}}}");
    let relative_end = after_environment.find(&end_marker)?;
    let body = &after_environment[..relative_end];
    let after_environment_index = body_start + relative_end + end_marker.len();
    Some((environment, body, after_environment_index))
}

struct LatexCommand<'a> {
    symbol: &'a str,
    consumed: usize,
}

fn latex_command(input: &str) -> Option<LatexCommand<'_>> {
    let name = input
        .chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .collect::<String>();
    let symbol = match name.as_str() {
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ε",
        "lambda" => "λ",
        "mu" => "μ",
        "pi" => "π",
        "sigma" => "σ",
        "theta" => "θ",
        "omega" => "ω",
        "Omega" => "Ω",
        "Delta" => "Δ",
        "Sigma" => "Σ",
        "sum" => "∑",
        "prod" => "∏",
        "int" => "∫",
        "infty" => "∞",
        "partial" => "∂",
        "nabla" => "∇",
        "times" => "×",
        "cdot" => "·",
        "pm" => "±",
        "to" => "→",
        "rightarrow" => "→",
        "leftarrow" => "←",
        "Rightarrow" => "⇒",
        "Leftarrow" => "⇐",
        "approx" => "≈",
        "equiv" => "≡",
        "lt" => "&lt;",
        "gt" => "&gt;",
        "le" => "≤",
        "leq" => "≤",
        "ge" => "≥",
        "geq" => "≥",
        "neq" => "≠",
        _ => return None,
    };
    Some(LatexCommand {
        symbol,
        consumed: name.len(),
    })
}

fn parse_latex_braced(input: &str, start: usize) -> Option<(String, usize)> {
    let mut index = skip_latex_whitespace(input, start);
    if input[index..].chars().next()? != '{' {
        return None;
    }
    index += 1;
    let content_start = index;
    let mut depth = 1usize;
    while index < input.len() {
        let ch = input[index..].chars().next()?;
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some((
                    input[content_start..index].to_string(),
                    index + ch.len_utf8(),
                ));
            }
        }
        index += ch.len_utf8();
    }
    None
}

fn parse_latex_script(input: &str, start: usize) -> Option<(String, usize)> {
    let index = skip_latex_whitespace(input, start);
    if input[index..].starts_with('{') {
        return parse_latex_braced(input, index);
    }
    let ch = input[index..].chars().next()?;
    if ch == '\\' {
        let command_len = input[index + 1..]
            .chars()
            .take_while(|candidate| candidate.is_ascii_alphabetic())
            .map(char::len_utf8)
            .sum::<usize>();
        if command_len > 0 {
            return Some((
                input[index..index + 1 + command_len].to_string(),
                index + 1 + command_len,
            ));
        }
    }
    Some((ch.to_string(), index + ch.len_utf8()))
}

fn skip_latex_whitespace(input: &str, start: usize) -> usize {
    let mut index = start.min(input.len());
    while index < input.len() {
        let Some(ch) = input[index..].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

pub(crate) fn render_callouts(markdown: &str) -> String {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut output = Vec::new();
    let mut index = 0;
    let mut fence_marker = None;
    while index < lines.len() {
        let line = lines[index];
        if let Some(marker) = fence_marker {
            output.push(line.to_string());
            if line.trim_start().starts_with(marker) {
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
    let mut fence_marker = None;
    markdown
        .lines()
        .map(|line| {
            if let Some(marker) = fence_marker {
                if line.trim_start().starts_with(marker) {
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
