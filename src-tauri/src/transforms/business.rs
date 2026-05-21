use crate::{bibliography::parse_bibliography_source, diag, escape_html, DocumentDiagnostic};

pub(crate) fn render_glossary_html(body: &str) -> String {
    let mut html = String::from("<dl class=\"glossary\">");
    for line in body.lines() {
        if let Some((term, definition)) = line.split_once(':') {
            html.push_str(&format!(
                "<dt>{}</dt><dd>{}</dd>",
                escape_html(term.trim()),
                escape_html(definition.trim())
            ));
        }
    }
    html.push_str("</dl>");
    html
}

pub(crate) fn render_bibtex_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let entries = parse_bibliography_source(body);
    if entries.is_empty() {
        let diagnostic = diag(
            "warning",
            "BibTeX transform did not contain any bibliography entries.",
            None,
            None,
            Some("Add BibTeX entries such as @book{key, title={Title}}."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-bibtex transform-error\">No bibliography entries found.</section>".to_string();
    }
    let mut html = String::from("<dl class=\"transform transform-bibtex\">");
    for entry in entries {
        let metadata = [entry.author.as_deref(), entry.issued.as_deref()]
            .into_iter()
            .flatten()
            .filter(|value| !value.trim().is_empty())
            .map(escape_html)
            .collect::<Vec<_>>()
            .join(" | ");
        let metadata = if metadata.is_empty() {
            String::new()
        } else {
            format!("<small>{metadata}</small>")
        };
        html.push_str(&format!(
            "<dt>{}</dt><dd><cite>{}</cite>{metadata}</dd>",
            escape_html(&entry.key),
            escape_html(&entry.title)
        ));
    }
    html.push_str("</dl>");
    html
}

pub(crate) fn render_timeline_svg(body: &str) -> String {
    let items = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let height = 80 + items.len() * 54;
    let mut svg = format!("<svg class=\"transform transform-timeline timeline\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 {height}\" role=\"img\"><line x1=\"120\" y1=\"40\" x2=\"120\" y2=\"{}\" stroke=\"#275DA8\" stroke-width=\"3\"/>", height - 30);
    for (index, item) in items.iter().enumerate() {
        let y = 50 + index * 54;
        svg.push_str(&format!("<circle cx=\"120\" cy=\"{y}\" r=\"8\" fill=\"#275DA8\"/><text x=\"150\" y=\"{}\" font-size=\"18\" fill=\"#1f2937\">{}</text>", y + 6, escape_html(item)));
    }
    svg.push_str("</svg>");
    svg
}

pub(crate) fn render_roadmap_html(body: &str) -> String {
    let items = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (stage, remainder) = line
                .split_once(':')
                .or_else(|| line.split_once('-'))
                .map(|(stage, text)| (stage.trim(), text.trim()))
                .unwrap_or(("Item", line));
            let parts = remainder
                .split('|')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>();
            let text = parts.first().copied().unwrap_or(remainder);
            let metadata = roadmap_metadata(&parts[1..]);
            format!(
                "<article class=\"roadmap-item\"><strong>{}</strong><p>{}</p>{metadata}</article>",
                escape_html(stage),
                escape_html(text)
            )
        })
        .collect::<String>();
    format!(
        "<section class=\"transform transform-roadmap\"><h3>Roadmap</h3><div>{items}</div></section>"
    )
}

pub(crate) fn render_adr_html(body: &str) -> String {
    let rows = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (key, value) = line
                .split_once(':')
                .map(|(key, value)| (key.trim(), value.trim()))
                .unwrap_or(("Note", line));
            let class = business_key_class(key);
            format!(
                "<tr class=\"adr-{class}\"><th>{}</th><td>{}</td></tr>",
                escape_html(key),
                escape_html(value)
            )
        })
        .collect::<String>();
    format!(
        "<section class=\"transform transform-adr\"><h3>Architecture Decision Record</h3><table><tbody>{rows}</tbody></table></section>"
    )
}

pub(crate) fn render_diff_html(body: &str) -> String {
    let mut additions = 0usize;
    let mut deletions = 0usize;
    let mut hunks = 0usize;
    let lines = body
        .lines()
        .map(|line| {
            let class = if line.starts_with('+') && !line.starts_with("+++") {
                additions += 1;
                "add"
            } else if line.starts_with('-') && !line.starts_with("---") {
                deletions += 1;
                "del"
            } else if line.starts_with("@@") {
                hunks += 1;
                "hunk"
            } else {
                "ctx"
            };
            format!("<code class=\"diff-{class}\">{}</code>", escape_html(line))
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<section class=\"transform transform-diff\"><p class=\"diff-summary\">{additions} additions / {deletions} deletions / {hunks} hunks</p><pre>{lines}</pre></section>"
    )
}

fn roadmap_metadata(parts: &[&str]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    let items = parts
        .iter()
        .map(|part| {
            let (key, value) = part
                .split_once('=')
                .or_else(|| part.split_once(':'))
                .map(|(key, value)| (key.trim(), value.trim()))
                .unwrap_or(("detail", *part));
            format!(
                "<span class=\"roadmap-meta roadmap-meta-{}\"><b>{}</b>: {}</span>",
                business_key_class(key),
                escape_html(key),
                escape_html(value)
            )
        })
        .collect::<String>();
    format!("<small>{items}</small>")
}

fn business_key_class(value: &str) -> String {
    let class = value
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if character.is_whitespace() || matches!(character, '-' | '_' | '.') {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if class.is_empty() {
        "field".to_string()
    } else {
        class
    }
}
