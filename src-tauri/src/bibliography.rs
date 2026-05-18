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
    pub(crate) raw: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CitationReference {
    pub(crate) key: String,
    pub(crate) locator: Option<String>,
    pub(crate) raw: String,
}

pub(crate) fn collect_bibliography(
    text: &str,
    metadata: &Value,
    root_path: Option<&Path>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<BibliographyEntry> {
    let mut sources = collect_fence_bodies(text, "bibtex");
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
    body.split('@')
        .filter_map(|entry| {
            let (kind_and_key, rest) = entry.split_once('{')?;
            let (key, raw) = rest.split_once(',')?;
            let title = raw
                .lines()
                .find(|line| line.trim_start().starts_with("title"))
                .and_then(|line| line.split_once('='))
                .map(|(_, value)| {
                    value
                        .trim()
                        .trim_matches(&['{', '}', ',', '"'][..])
                        .to_string()
                })
                .unwrap_or_else(|| kind_and_key.trim().to_string());
            Some(BibliographyEntry {
                key: key.trim().to_string(),
                title,
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
                raw: entry.to_string(),
            })
        })
        .collect();
    Ok(entries)
}

pub(crate) fn collect_citation_references(text: &str) -> Vec<CitationReference> {
    let mut citations = Vec::new();
    for segment in text.split('[').skip(1) {
        if let Some((inside, _)) = segment.split_once(']') {
            if !inside.contains('@') {
                continue;
            }
            citations.extend(citation_references_from_bracket(inside));
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

fn citation_references_from_bracket(text: &str) -> Vec<CitationReference> {
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
            });
        }
        rest = &after_at[key_len..];
    }
    references
}

pub(crate) fn render_citations(markdown: &str, bibliography: &[BibliographyEntry]) -> String {
    let titles = bibliography
        .iter()
        .map(|entry| (entry.key.as_str(), entry.title.as_str()))
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
            let references = citation_references_from_bracket(inside);
            output.push_str(&render_citation_span(&references, &titles));
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

fn render_citation_span(references: &[CitationReference], titles: &HashMap<&str, &str>) -> String {
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
            let mut label = titles
                .get(reference.key.as_str())
                .copied()
                .unwrap_or(reference.key.as_str())
                .to_string();
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
            let title = titles
                .get(reference.key.as_str())
                .copied()
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
