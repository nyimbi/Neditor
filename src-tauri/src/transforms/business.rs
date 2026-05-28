use crate::{
    bibliography::{parse_bibliography_source, BibliographyEntry},
    diag, escape_html, DocumentDiagnostic,
};

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
        let metadata = bibtex_entry_metadata_html(&entry);
        html.push_str(&format!(
            "<dt>{}</dt><dd><cite>{}</cite>{metadata}</dd>",
            escape_html(&entry.key),
            escape_html(&entry.title)
        ));
    }
    html.push_str("</dl>");
    html
}

fn bibtex_entry_metadata_html(entry: &BibliographyEntry) -> String {
    let mut rows = Vec::new();
    push_bibtex_metadata_row(&mut rows, "Type", entry.entry_type.as_deref());
    push_bibtex_metadata_row(&mut rows, "Author", entry.author.as_deref());
    push_bibtex_metadata_row(&mut rows, "Issued", entry.issued.as_deref());
    for (label, fields) in [
        (
            "Journal",
            &["journal", "journaltitle", "container-title"][..],
        ),
        ("Book title", &["booktitle", "collection-title"][..]),
        ("Publisher", &["publisher"][..]),
        ("Volume", &["volume"][..]),
        ("Issue", &["number", "issue"][..]),
        ("Pages", &["pages", "page"][..]),
        ("DOI", &["doi"][..]),
        ("URL", &["url", "URL"][..]),
        ("Editor", &["editor"][..]),
        ("Organization", &["organization"][..]),
        ("Institution", &["institution"][..]),
        ("School", &["school"][..]),
        ("Series", &["series"][..]),
        ("Edition", &["edition"][..]),
        ("Address", &["address", "location"][..]),
        ("ISBN", &["isbn", "ISBN"][..]),
        ("ISSN", &["issn", "ISSN"][..]),
        (
            "Archive",
            &["archiveprefix", "archivePrefix", "archive"][..],
        ),
        ("Eprint", &["eprint"][..]),
        ("Note", &["note"][..]),
        ("Abstract", &["abstract"][..]),
    ] {
        let value = fields.iter().find_map(|field| entry.fields.get(*field));
        push_bibtex_metadata_row(&mut rows, label, value.map(String::as_str));
    }
    if rows.is_empty() {
        return String::new();
    }
    format!(
        "<table class=\"bibtex-entry-metadata\"><tbody>{}</tbody></table>",
        rows.join("")
    )
}

fn push_bibtex_metadata_row(rows: &mut Vec<String>, label: &str, value: Option<&str>) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    rows.push(format!(
        "<tr><th>{}</th><td>{}</td></tr>",
        escape_html(label),
        escape_html(value)
    ));
}

pub(crate) fn render_timeline_svg(body: &str) -> String {
    let items = body
        .lines()
        .map(parse_timeline_item)
        .filter(|item| !item.marker.is_empty() || !item.label.is_empty())
        .collect::<Vec<_>>();
    let height = 80 + items.len() * 76;
    let mut svg = format!("<svg class=\"transform transform-timeline timeline\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 {height}\" role=\"img\"><line x1=\"120\" y1=\"40\" x2=\"120\" y2=\"{}\" stroke=\"#275DA8\" stroke-width=\"3\"/>", height - 30);
    for (index, item) in items.iter().enumerate() {
        let y = 50 + index * 76;
        let metadata = timeline_metadata_svg(&item.metadata, y);
        svg.push_str(&format!(
            "<g class=\"timeline-item\"><circle cx=\"120\" cy=\"{y}\" r=\"8\" fill=\"#275DA8\"/><text class=\"timeline-marker\" x=\"150\" y=\"{}\" font-size=\"14\" font-weight=\"700\" fill=\"#275DA8\">{}</text><text class=\"timeline-label\" x=\"270\" y=\"{}\" font-size=\"18\" fill=\"#1f2937\">{}</text>{metadata}</g>",
            y + 5,
            escape_html(&item.marker),
            y + 5,
            escape_html(&item.label)
        ));
    }
    svg.push_str("</svg>");
    svg
}

struct TimelineItem {
    marker: String,
    label: String,
    metadata: Vec<(String, String)>,
}

fn parse_timeline_item(line: &str) -> TimelineItem {
    let parts = line
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let event = parts.first().copied().unwrap_or_default();
    let (marker, label) = event
        .split_once(':')
        .or_else(|| event.split_once(" - "))
        .map(|(marker, label)| (marker.trim(), label.trim()))
        .unwrap_or(("", event.trim()));
    let metadata = parts
        .iter()
        .skip(1)
        .filter_map(|part| {
            let (key, value) = part.split_once('=')?;
            let key = key.trim();
            let value = value.trim();
            if key.is_empty() || value.is_empty() {
                None
            } else {
                Some((key.to_string(), value.to_string()))
            }
        })
        .collect();
    TimelineItem {
        marker: marker.to_string(),
        label: label.to_string(),
        metadata,
    }
}

fn timeline_metadata_svg(metadata: &[(String, String)], y: usize) -> String {
    metadata
        .iter()
        .take(4)
        .enumerate()
        .map(|(index, (key, value))| {
            let x = 270 + index * 142;
            let class_key = metadata_class_key(key);
            format!(
                "<text class=\"timeline-meta timeline-meta-{class_key}\" x=\"{x}\" y=\"{}\" font-size=\"12\" fill=\"#64748b\"><tspan font-weight=\"700\">{}</tspan>: {}</text>",
                y + 28,
                escape_html(key),
                escape_html(value)
            )
        })
        .collect::<String>()
}

fn metadata_class_key(key: &str) -> String {
    let normalized = key
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if matches!(character, '-' | '_') {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>();
    if normalized.is_empty() {
        "item".to_string()
    } else {
        normalized
    }
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
