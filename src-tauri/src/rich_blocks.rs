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
                "<figure class=\"equation\" id=\"{}\" data-caption=\"{}\"><div class=\"math-rendered math-display\" data-katex=\"{}\" data-katex-display role=\"math\" aria-label=\"{}\">{}</div><details class=\"math-source\"><summary>LaTeX</summary><pre><code>{}</code></pre></details><figcaption>{}</figcaption></figure>\n",
                escape_html(&id),
                escape_html(caption.trim()),
                escape_html(latex),
                escape_html(latex),
                latex_to_html(latex),
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
                latex_to_html(math),
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

// ─── LaTeX → HTML renderer ────────────────────────────────────────────────────

/// Convert a LaTeX math expression to semantic HTML with structured span/table markup.
pub(crate) fn latex_to_html(latex: &str) -> String {
    let chars: Vec<char> = latex.chars().collect();
    let mut pos = 0;
    latex_render(&chars, &mut pos)
}

fn latex_render(chars: &[char], pos: &mut usize) -> String {
    let mut out = String::new();
    while *pos < chars.len() {
        match chars[*pos] {
            '}' => break,
            '{' => {
                *pos += 1;
                let inner = latex_render(chars, pos);
                if chars.get(*pos) == Some(&'}') {
                    *pos += 1;
                }
                out.push_str(&inner);
            }
            '\\' => {
                *pos += 1;
                if chars.get(*pos) == Some(&'\\') {
                    *pos += 1; // row break — skip in non-matrix context
                } else {
                    let cmd = latex_read_cmd(chars, pos);
                    out.push_str(&latex_cmd(&cmd, chars, pos));
                }
            }
            '^' => {
                *pos += 1;
                out.push_str(&format!("<sup>{}</sup>", latex_group(chars, pos)));
            }
            '_' => {
                *pos += 1;
                out.push_str(&format!("<sub>{}</sub>", latex_group(chars, pos)));
            }
            '&' => {
                *pos += 1;
            }
            '<' => {
                out.push_str("&lt;");
                *pos += 1;
            }
            '>' => {
                out.push_str("&gt;");
                *pos += 1;
            }
            '\n' | '\r' => {
                out.push(' ');
                *pos += 1;
            }
            ch => {
                out.push(ch);
                *pos += 1;
            }
        }
    }
    out
}

fn latex_read_cmd(chars: &[char], pos: &mut usize) -> String {
    if *pos >= chars.len() {
        return String::new();
    }
    if chars[*pos].is_ascii_alphabetic() {
        let mut name = String::new();
        while chars
            .get(*pos)
            .map(|c| c.is_ascii_alphabetic())
            .unwrap_or(false)
        {
            name.push(chars[*pos]);
            *pos += 1;
        }
        // Do NOT skip trailing spaces — the space after a word command is part
        // of the surrounding content (e.g. `x \ge 0` → `x ≥ 0`).
        name
    } else {
        let c = chars[*pos];
        *pos += 1;
        c.to_string()
    }
}

fn latex_skip_ws(chars: &[char], pos: &mut usize) {
    while matches!(
        chars.get(*pos),
        Some(' ') | Some('\t') | Some('\n') | Some('\r')
    ) {
        *pos += 1;
    }
}

/// Parse a {group} or a single char/command, return rendered HTML.
fn latex_group(chars: &[char], pos: &mut usize) -> String {
    latex_skip_ws(chars, pos);
    if chars.get(*pos) == Some(&'{') {
        *pos += 1;
        let content = latex_render(chars, pos);
        if chars.get(*pos) == Some(&'}') {
            *pos += 1;
        }
        content
    } else if chars.get(*pos) == Some(&'\\') {
        *pos += 1;
        let cmd = latex_read_cmd(chars, pos);
        latex_cmd(&cmd, chars, pos)
    } else if *pos < chars.len() {
        let ch = chars[*pos];
        *pos += 1;
        match ch {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            other => other.to_string(),
        }
    } else {
        String::new()
    }
}

/// Parse optional [bracket] and return its raw text.
fn latex_bracket(chars: &[char], pos: &mut usize) -> Option<String> {
    latex_skip_ws(chars, pos);
    if chars.get(*pos) != Some(&'[') {
        return None;
    }
    *pos += 1;
    let mut content = String::new();
    let mut depth = 1i32;
    while *pos < chars.len() {
        match chars[*pos] {
            '[' => {
                depth += 1;
                content.push('[');
                *pos += 1;
            }
            ']' => {
                depth -= 1;
                *pos += 1;
                if depth == 0 {
                    break;
                }
                content.push(']');
            }
            ch => {
                content.push(ch);
                *pos += 1;
            }
        }
    }
    Some(content)
}

/// Read a raw {group} without rendering (for environment names).
fn latex_group_raw(chars: &[char], pos: &mut usize) -> String {
    latex_skip_ws(chars, pos);
    if chars.get(*pos) != Some(&'{') {
        return String::new();
    }
    *pos += 1;
    let mut content = String::new();
    let mut depth = 1i32;
    while *pos < chars.len() {
        match chars[*pos] {
            '{' => {
                depth += 1;
                content.push('{');
                *pos += 1;
            }
            '}' => {
                depth -= 1;
                *pos += 1;
                if depth == 0 {
                    break;
                }
                content.push('}');
            }
            ch => {
                content.push(ch);
                *pos += 1;
            }
        }
    }
    content
}

/// Collect raw char tokens for a matrix body, splitting rows and cells.
fn latex_matrix_body(chars: &[char], pos: &mut usize) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cell: Vec<char> = Vec::new();

    while *pos < chars.len() {
        if chars[*pos] == '\\' {
            // Check for \end{
            if *pos + 4 < chars.len() {
                let snip: String = chars[*pos..*pos + 5].iter().collect();
                if snip == "\\end{" {
                    *pos += 5;
                    while *pos < chars.len() && chars[*pos] != '}' {
                        *pos += 1;
                    }
                    if *pos < chars.len() {
                        *pos += 1;
                    }
                    let s: String = cell.iter().collect();
                    row.push(s.trim().to_string());
                    if !row.iter().all(|c| c.is_empty()) {
                        rows.push(row);
                    }
                    return rows;
                }
            }
            // Check for \\ (row separator)
            if chars.get(*pos + 1) == Some(&'\\') {
                let s: String = cell.iter().collect();
                row.push(s.trim().to_string());
                rows.push(std::mem::take(&mut row));
                cell.clear();
                *pos += 2;
                latex_skip_ws(chars, pos);
                continue;
            }
            cell.push(chars[*pos]);
            *pos += 1;
        } else if chars[*pos] == '&' {
            let s: String = cell.iter().collect();
            row.push(s.trim().to_string());
            cell.clear();
            *pos += 1;
        } else {
            cell.push(chars[*pos]);
            *pos += 1;
        }
    }
    rows
}

fn latex_render_matrix(rows: Vec<Vec<String>>, kind: &str) -> String {
    let mut html = format!("<span class=\"math-matrix {kind}\"><table>");
    for row in rows {
        html.push_str("<tr>");
        for cell in row {
            html.push_str(&format!("<td>{}</td>", latex_to_html(cell.trim())));
        }
        html.push_str("</tr>");
    }
    html.push_str("</table></span>");
    html
}

fn latex_cmd(cmd: &str, chars: &[char], pos: &mut usize) -> String {
    match cmd {
        // Greek uppercase
        "Gamma" => "Γ".into(),
        "Delta" => "Δ".into(),
        "Theta" => "Θ".into(),
        "Lambda" => "Λ".into(),
        "Pi" => "Π".into(),
        "Sigma" => "Σ".into(),
        "Phi" => "Φ".into(),
        "Psi" => "Ψ".into(),
        "Omega" => "Ω".into(),
        "Upsilon" => "Υ".into(),
        "Xi" => "Ξ".into(),
        // Greek lowercase
        "alpha" => "α".into(),
        "beta" => "β".into(),
        "gamma" => "γ".into(),
        "delta" => "δ".into(),
        "epsilon" | "varepsilon" => "ε".into(),
        "zeta" => "ζ".into(),
        "eta" => "η".into(),
        "theta" | "vartheta" => "θ".into(),
        "iota" => "ι".into(),
        "kappa" => "κ".into(),
        "lambda" => "λ".into(),
        "mu" => "μ".into(),
        "nu" => "ν".into(),
        "xi" => "ξ".into(),
        "pi" | "varpi" => "π".into(),
        "rho" | "varrho" => "ρ".into(),
        "sigma" | "varsigma" => "σ".into(),
        "tau" => "τ".into(),
        "upsilon" => "υ".into(),
        "phi" | "varphi" => "φ".into(),
        "chi" => "χ".into(),
        "psi" => "ψ".into(),
        "omega" => "ω".into(),
        // Math operators
        "sum" => "∑".into(),
        "prod" => "∏".into(),
        "int" => "∫".into(),
        "partial" => "∂".into(),
        "nabla" => "∇".into(),
        "infty" => "∞".into(),
        "pm" => "±".into(),
        "mp" => "∓".into(),
        "cdot" => "·".into(),
        "times" => "×".into(),
        "div" => "÷".into(),
        "cdots" => "⋯".into(),
        "vdots" => "⋮".into(),
        "ddots" => "⋱".into(),
        "ldots" | "dots" => "…".into(),
        // Relations
        "approx" => "≈".into(),
        "sim" => "∼".into(),
        "equiv" => "≡".into(),
        "ne" | "neq" => "≠".into(),
        "le" | "leq" => "≤".into(),
        "ge" | "geq" => "≥".into(),
        "ll" => "≪".into(),
        "gg" => "≫".into(),
        // Set / logic
        "in" => "∈".into(),
        "notin" => "∉".into(),
        "subset" => "⊂".into(),
        "supset" => "⊃".into(),
        "subseteq" => "⊆".into(),
        "supseteq" => "⊇".into(),
        "cup" => "∪".into(),
        "cap" => "∩".into(),
        "emptyset" => "∅".into(),
        "forall" => "∀".into(),
        "exists" => "∃".into(),
        "land" | "wedge" => "∧".into(),
        "lor" | "vee" => "∨".into(),
        "lnot" | "neg" => "¬".into(),
        // Arrows
        "to" | "rightarrow" => "→".into(),
        "leftarrow" | "gets" => "←".into(),
        "Rightarrow" => "⇒".into(),
        "Leftarrow" => "⟸".into(),
        "Leftrightarrow" | "iff" => "⟺".into(),
        "leftrightarrow" => "↔".into(),
        "mapsto" => "↦".into(),
        // Functions
        "lim" => "lim".into(),
        "limsup" => "lim sup".into(),
        "liminf" => "lim inf".into(),
        "sup" => "sup".into(),
        "inf" => "inf".into(),
        "max" => "max".into(),
        "min" => "min".into(),
        "det" => "det".into(),
        "dim" => "dim".into(),
        "exp" => "exp".into(),
        "gcd" => "gcd".into(),
        "ker" => "ker".into(),
        "log" => "log".into(),
        "ln" => "ln".into(),
        "sin" => "sin".into(),
        "cos" => "cos".into(),
        "tan" => "tan".into(),
        "arcsin" => "arcsin".into(),
        "arccos" => "arccos".into(),
        "arctan" => "arctan".into(),
        "Pr" => "Pr".into(),
        "Re" => "ℜ".into(),
        "Im" => "ℑ".into(),
        // Misc symbols
        "ell" => "ℓ".into(),
        "hbar" => "ℏ".into(),
        "angle" => "∠".into(),
        "perp" => "⊥".into(),
        "mid" => "∣".into(),
        "therefore" => "∴".into(),
        "dag" | "dagger" => "†".into(),
        // Delimiters (consume next char)
        "left" => {
            latex_skip_ws(chars, pos);
            if *pos < chars.len() {
                let d = chars[*pos];
                *pos += 1;
                if d == '\\' {
                    let cmd2 = latex_read_cmd(chars, pos);
                    match cmd2.as_str() {
                        "langle" => "⟨".into(),
                        "rangle" => "⟩".into(),
                        "lfloor" => "⌊".into(),
                        "rfloor" => "⌋".into(),
                        "lceil" => "⌈".into(),
                        "rceil" => "⌉".into(),
                        "." => String::new(),
                        other => other.into(),
                    }
                } else {
                    match d {
                        '<' => "&lt;".into(),
                        '>' => "&gt;".into(),
                        other => other.to_string(),
                    }
                }
            } else {
                String::new()
            }
        }
        "right" => {
            latex_skip_ws(chars, pos);
            if *pos < chars.len() {
                let d = chars[*pos];
                *pos += 1;
                if d == '\\' {
                    let cmd2 = latex_read_cmd(chars, pos);
                    match cmd2.as_str() {
                        "rangle" => "⟩".into(),
                        "langle" => "⟨".into(),
                        "rfloor" => "⌋".into(),
                        "lfloor" => "⌊".into(),
                        "rceil" => "⌉".into(),
                        "lceil" => "⌈".into(),
                        "." => String::new(),
                        other => other.into(),
                    }
                } else {
                    match d {
                        '<' => "&lt;".into(),
                        '>' => "&gt;".into(),
                        other => other.to_string(),
                    }
                }
            } else {
                String::new()
            }
        }
        // One-arg with optional bracket
        "sqrt" => {
            let idx = latex_bracket(chars, pos);
            let arg = latex_group(chars, pos);
            match idx {
                Some(i) => format!("<span class=\"math-root-index\">{i}</span>√{arg}"),
                None => format!("√{arg}"),
            }
        }
        // Two-arg
        "frac" => {
            let num = latex_group(chars, pos);
            let den = latex_group(chars, pos);
            format!(
                "<span class=\"math-frac\"><span class=\"math-num\">{num}</span><span class=\"math-den\">{den}</span></span>"
            )
        }
        // One-arg wrappers
        "overline" | "bar" => {
            format!(
                "<span class=\"math-overline\">{}</span>",
                latex_group(chars, pos)
            )
        }
        "underline" => {
            format!(
                "<span class=\"math-underline\">{}</span>",
                latex_group(chars, pos)
            )
        }
        "hat" | "widehat" => {
            format!(
                "<span class=\"math-hat\">{}</span>",
                latex_group(chars, pos)
            )
        }
        "vec" | "overrightarrow" => {
            format!(
                "<span class=\"math-vec\">{}</span>",
                latex_group(chars, pos)
            )
        }
        "text" | "textrm" | "textbf" | "textit" | "textsf" => {
            format!(
                "<span class=\"math-text\">{}</span>",
                latex_group(chars, pos)
            )
        }
        "mathbb" => {
            format!(
                "<span class=\"math-blackboard\">{}</span>",
                latex_group(chars, pos)
            )
        }
        "mathcal" => {
            format!(
                "<span class=\"math-calligraphic\">{}</span>",
                latex_group(chars, pos)
            )
        }
        "mathrm" | "operatorname" => {
            format!(
                "<span class=\"math-roman\">{}</span>",
                latex_group(chars, pos)
            )
        }
        "mathbf" | "boldsymbol" => {
            format!("<strong>{}</strong>", latex_group(chars, pos))
        }
        "mathit" => {
            format!("<em>{}</em>", latex_group(chars, pos))
        }
        "widetilde" | "tilde" => {
            format!(
                "<span class=\"math-hat\" title=\"tilde\">{}</span>",
                latex_group(chars, pos)
            )
        }
        // Matrix environments
        "begin" => {
            let env = latex_group_raw(chars, pos);
            let rows = latex_matrix_body(chars, pos);
            let kind = match env.trim() {
                "bmatrix" => "matrix-square",
                "Bmatrix" => "matrix-curly",
                "pmatrix" => "matrix-round",
                "vmatrix" => "matrix-vertical",
                "cases" | "cases*" => "matrix-cases",
                _ => "matrix-plain",
            };
            latex_render_matrix(rows, kind)
        }
        // Spacing
        "," | ";" | ":" | "!" => "\u{200B}".into(),
        "quad" => "&ensp;".into(),
        "qquad" => "&emsp;".into(),
        // Fallback: emit command name
        other => other.into(),
    }
}

// ──────────────────────────────────────────────────────────────────────────────

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
