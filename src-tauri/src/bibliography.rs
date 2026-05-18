use crate::diagnostics::DocumentDiagnostic;
use crate::{collect_fence_bodies, diagnostics::diag, escape_html, path_to_string};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize)]
pub(crate) struct BibliographyEntry {
    pub(crate) key: String,
    pub(crate) title: String,
    pub(crate) author: Option<String>,
    pub(crate) issued: Option<String>,
    pub(crate) raw: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CitationReference {
    pub(crate) key: String,
    pub(crate) locator: Option<String>,
    pub(crate) raw: String,
    pub(crate) line: usize,
}

pub(crate) fn collect_bibliography(
    text: &str,
    metadata: &Value,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<BibliographyEntry> {
    let mut sources = collect_fence_bodies(text, "bibtex");
    sources.extend(collect_fence_bodies(text, "hayagriva"));
    sources.extend(collect_fence_bodies(text, "bibliography"));
    if let Some(path) = metadata.get("bibliography").and_then(Value::as_str) {
        let base = root_path
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let bibliography_path = base.join(path);
        match fs::read_to_string(&bibliography_path) {
            Ok(contents) => sources.push(contents),
            Err(err) => diagnostics.push(diag(
                "error",
                format!(
                    "Missing bibliography file {}: {err}",
                    bibliography_path.display()
                ),
                Some(path_to_string(&bibliography_path)),
                None,
                Some("Create the bibliography file or update front matter."),
            )),
        }
    }

    sources
        .into_iter()
        .flat_map(|body| parse_bibliography_source(&body))
        .collect()
}

pub(crate) fn parse_bibliography_source(body: &str) -> Vec<BibliographyEntry> {
    if let Ok(entries) = parse_csl_json_bibliography(body) {
        return entries;
    }
    if let Ok(entries) = parse_hayagriva_yaml_bibliography(body) {
        return entries;
    }
    body.split('@')
        .filter_map(|entry| {
            let (kind_and_key, rest) = entry.split_once('{')?;
            let (key, raw) = rest.split_once(',')?;
            let title =
                bibtex_field(raw, "title").unwrap_or_else(|| kind_and_key.trim().to_string());
            Some(BibliographyEntry {
                key: key.trim().to_string(),
                title,
                author: bibtex_field(raw, "author"),
                issued: bibtex_issued_year(raw),
                raw: raw.to_string(),
            })
        })
        .collect()
}

pub(crate) fn duplicate_bibliography_keys(bibliography: &[BibliographyEntry]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut duplicates = BTreeSet::new();
    for entry in bibliography {
        if !seen.insert(entry.key.as_str()) {
            duplicates.insert(entry.key.clone());
        }
    }
    duplicates.into_iter().collect()
}

fn parse_hayagriva_yaml_bibliography(
    body: &str,
) -> Result<Vec<BibliographyEntry>, serde_yaml::Error> {
    let value = serde_yaml::from_str::<Value>(body)?;
    let entries = value
        .as_object()
        .into_iter()
        .flat_map(|entries| entries.iter())
        .filter_map(|(key, entry)| {
            let title = entry
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(key)
                .to_string();
            Some(BibliographyEntry {
                key: key.to_string(),
                title,
                author: yaml_author(entry),
                issued: yaml_issued_year(entry),
                raw: serde_yaml::to_string(entry).unwrap_or_default(),
            })
        })
        .collect();
    Ok(entries)
}

fn parse_csl_json_bibliography(body: &str) -> Result<Vec<BibliographyEntry>, serde_json::Error> {
    let value = serde_json::from_str::<Value>(body)?;
    let entries = value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let key = entry
                .get("id")
                .or_else(|| entry.get("citation-key"))
                .and_then(Value::as_str)?;
            let title = entry
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(key)
                .to_string();
            Some(BibliographyEntry {
                key: key.to_string(),
                title,
                author: csl_author(entry),
                issued: csl_issued_year(entry),
                raw: entry.to_string(),
            })
        })
        .collect();
    Ok(entries)
}

fn yaml_author(entry: &Value) -> Option<String> {
    entry.get("author").and_then(|author| match author {
        Value::String(value) => Some(value.clone()),
        Value::Array(values) => values.first().and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Object(author) => author
                .get("name")
                .or_else(|| author.get("family"))
                .or_else(|| author.get("literal"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            _ => None,
        }),
        _ => None,
    })
}

fn yaml_issued_year(entry: &Value) -> Option<String> {
    ["date", "year", "issued"]
        .into_iter()
        .find_map(|field| entry.get(field))
        .and_then(|value| match value {
            Value::String(value) => Some(value.chars().take(4).collect::<String>()),
            Value::Number(value) => Some(value.to_string()),
            _ => None,
        })
        .filter(|year| year.len() == 4 && year.chars().all(|ch| ch.is_ascii_digit()))
}

fn bibtex_field(raw: &str, field: &str) -> Option<String> {
    raw.lines()
        .find(|line| line.trim_start().starts_with(field))
        .and_then(|line| line.split_once('='))
        .map(|(_, value)| clean_bibliography_value(value))
        .filter(|value| !value.is_empty())
}

fn clean_bibliography_value(value: &str) -> String {
    value
        .trim()
        .trim_matches(&['{', '}', ',', '"'][..])
        .trim()
        .to_string()
}

fn bibtex_issued_year(raw: &str) -> Option<String> {
    bibtex_field(raw, "year").or_else(|| {
        bibtex_field(raw, "date").and_then(|date| {
            let year = date.chars().take(4).collect::<String>();
            (year.len() == 4 && year.chars().all(|ch| ch.is_ascii_digit())).then_some(year)
        })
    })
}

fn csl_author(entry: &Value) -> Option<String> {
    entry
        .get("author")
        .and_then(Value::as_array)
        .and_then(|authors| authors.first())
        .and_then(|author| {
            author
                .get("literal")
                .and_then(Value::as_str)
                .or_else(|| author.get("family").and_then(Value::as_str))
                .or_else(|| author.get("name").and_then(Value::as_str))
        })
        .map(ToString::to_string)
}

fn csl_issued_year(entry: &Value) -> Option<String> {
    entry
        .get("issued")
        .and_then(|issued| issued.get("date-parts"))
        .and_then(Value::as_array)
        .and_then(|date_parts| date_parts.first())
        .and_then(Value::as_array)
        .and_then(|first_date| first_date.first())
        .and_then(Value::as_i64)
        .map(|year| year.to_string())
}

pub(crate) fn collect_citation_references(text: &str) -> Vec<CitationReference> {
    let mut citations = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for segment in line.split('[').skip(1) {
            if let Some((inside, _)) = segment.split_once(']') {
                if !inside.contains('@') {
                    continue;
                }
                citations.extend(citation_references_from_bracket(inside, index + 1));
            }
        }
    }
    citations
}

pub(crate) fn citation_keys_from_references(references: &[CitationReference]) -> Vec<String> {
    let mut citations = BTreeSet::new();
    for reference in references {
        citations.insert(reference.key.clone());
    }
    citations.into_iter().collect()
}

fn citation_references_from_bracket(text: &str, line: usize) -> Vec<CitationReference> {
    let mut references = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find('@') {
        let after_at = &rest[index + 1..];
        let key = after_at
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':'))
            .collect::<String>();
        let key_len = key.len();
        if !key.is_empty() {
            let after_key = &after_at[key_len..];
            let locator_end = after_key.find('@').unwrap_or(after_key.len());
            let locator = after_key[..locator_end]
                .trim()
                .trim_start_matches(|ch| ch == ',' || ch == ';')
                .trim()
                .trim_end_matches(';')
                .trim();
            references.push(CitationReference {
                key,
                locator: (!locator.is_empty()).then(|| locator.to_string()),
                raw: text.to_string(),
                line,
            });
        }
        rest = &after_at[key_len..];
    }
    references
}

pub(crate) fn render_citations(
    markdown: &str,
    bibliography: &[BibliographyEntry],
    style: &str,
) -> String {
    let entries = bibliography
        .iter()
        .map(|entry| (entry.key.as_str(), entry))
        .collect::<HashMap<_, _>>();
    let mut output = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(start) = rest.find('[') {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find(']') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let inside = &after_start[..end];
        if inside.contains('@') {
            let references = citation_references_from_bracket(inside, 0);
            output.push_str(&render_citation_span(&references, &entries, style));
        } else {
            output.push('[');
            output.push_str(inside);
            output.push(']');
        }
        rest = &after_start[end + 1..];
    }
    output.push_str(rest);
    output
}

fn render_citation_span(
    references: &[CitationReference],
    entries: &HashMap<&str, &BibliographyEntry>,
    style: &str,
) -> String {
    if references.is_empty() {
        return String::new();
    }
    let keys = references
        .iter()
        .map(|reference| reference.key.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let label = references
        .iter()
        .map(|reference| {
            let mut label = citation_label(
                reference,
                entries.get(reference.key.as_str()).copied(),
                style,
            );
            if let Some(locator) = &reference.locator {
                label.push_str(", ");
                label.push_str(locator);
            }
            label
        })
        .collect::<Vec<_>>()
        .join("; ");
    let details = references
        .iter()
        .map(|reference| {
            let title = entries
                .get(reference.key.as_str())
                .map(|entry| entry.title.as_str())
                .unwrap_or("missing bibliography entry");
            match &reference.locator {
                Some(locator) => format!("@{} ({locator}): {title}", reference.key),
                None => format!("@{}: {title}", reference.key),
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "<span class=\"citation\" tabindex=\"0\" title=\"{}\" aria-label=\"Citation: {}\" data-citation-keys=\"{}\">({})</span>",
        escape_html(&details),
        escape_html(&details),
        escape_html(&keys),
        escape_html(&label)
    )
}

fn citation_label(
    reference: &CitationReference,
    entry: Option<&BibliographyEntry>,
    style: &str,
) -> String {
    let Some(entry) = entry else {
        return reference.key.clone();
    };
    match style {
        "key" => format!("@{}", reference.key),
        "author-year" => match (&entry.author, &entry.issued) {
            (Some(author), Some(year)) => format!("{author} {year}"),
            (Some(author), None) => author.clone(),
            (None, Some(year)) => year.clone(),
            (None, None) => entry.title.clone(),
        },
        _ => entry.title.clone(),
    }
}
