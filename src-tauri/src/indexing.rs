use crate::{compiler_support::fenced_code_marker, metadata_lookup, Heading};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(crate) struct IndexEntry {
    pub(crate) term: String,
    pub(crate) anchor: Option<String>,
}

struct MetadataIndexTerm {
    term: String,
    anchor: Option<String>,
}

pub(crate) fn collect_index_entries(
    text: &str,
    metadata: &Value,
    headings: &[Heading],
    glossary: &BTreeMap<String, String>,
) -> Vec<IndexEntry> {
    let excluded = index_exclude_terms(metadata);
    let mut entries: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut proper_nouns: BTreeMap<String, (usize, Option<String>)> = BTreeMap::new();
    let mut heading_index = 0usize;
    let mut current_anchor = headings.first().map(|heading| heading.anchor.clone());
    let mut fence_marker: Option<String> = None;

    for (zero_index, line) in text.lines().enumerate() {
        let line_number = zero_index + 1;
        while heading_index < headings.len() && headings[heading_index].line <= line_number {
            current_anchor = Some(headings[heading_index].anchor.clone());
            heading_index += 1;
        }
        if let Some(ref marker) = fence_marker {
            if line.trim_start().starts_with(marker.as_str()) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            fence_marker = Some(marker);
            continue;
        }
        for term in explicit_index_terms(line) {
            insert_index_entry(&mut entries, &excluded, term, current_anchor.clone());
        }
        for term in bold_index_terms(line) {
            insert_index_entry(&mut entries, &excluded, term, current_anchor.clone());
        }
        for term in proper_noun_candidates(line) {
            if excluded.contains(&index_exclude_key(&term)) {
                continue;
            }
            let entry = proper_nouns
                .entry(term)
                .or_insert_with(|| (0, current_anchor.clone()));
            entry.0 += 1;
            if entry.1.is_none() {
                entry.1 = current_anchor.clone();
            }
        }
    }

    for heading in headings {
        insert_index_entry(
            &mut entries,
            &excluded,
            heading.text.clone(),
            Some(heading.anchor.clone()),
        );
    }
    for term in glossary.keys() {
        let anchor = first_term_anchor(text, headings, term).or_else(|| current_anchor.clone());
        insert_index_entry(&mut entries, &excluded, term.clone(), anchor);
    }
    for term in metadata_index_terms(metadata) {
        let anchor = term
            .anchor
            .or_else(|| first_term_anchor(text, headings, &term.term));
        insert_index_entry(&mut entries, &excluded, term.term, anchor);
    }
    for (term, (count, anchor)) in proper_nouns {
        if count >= 2 {
            insert_index_entry(&mut entries, &excluded, term, anchor);
        }
    }

    entries
        .into_iter()
        .map(|(term, anchor)| IndexEntry { term, anchor })
        .collect()
}

pub(crate) fn render_index_entries(entries: &[IndexEntry]) -> String {
    if entries.is_empty() {
        return "_No index terms found._".to_string();
    }
    entries
        .iter()
        .map(|entry| {
            if let Some(anchor) = &entry.anchor {
                format!(
                    "- [{}](#{})",
                    escape_markdown_link_text(&entry.term),
                    anchor
                )
            } else {
                format!("- {}", entry.term)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn strip_index_markers(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{#index:") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + "{#index:".len()..];
        if let Some(end) = after_start.find('}') {
            rest = &after_start[end + 1..];
        } else {
            output.push_str("{#index:");
            output.push_str(after_start);
            return output;
        }
    }
    output.push_str(rest);
    output
}

fn insert_index_entry(
    entries: &mut BTreeMap<String, Option<String>>,
    excluded: &BTreeSet<String>,
    term: String,
    anchor: Option<String>,
) {
    let normalized = term.trim().trim_matches('"').trim_matches('\'').to_string();
    if normalized.is_empty()
        || normalized.len() > 100
        || excluded.contains(&index_exclude_key(&normalized))
    {
        return;
    }
    entries
        .entry(normalized)
        .and_modify(|existing| {
            if existing.is_none() {
                *existing = anchor.clone();
            }
        })
        .or_insert(anchor);
}

fn index_exclude_terms(metadata: &Value) -> BTreeSet<String> {
    let mut terms = BTreeSet::new();
    for term in metadata_string_values(metadata, &["indexExclude", "index.exclude"]) {
        terms.insert(index_exclude_key(&term));
    }
    terms
}

fn metadata_index_terms(metadata: &Value) -> Vec<MetadataIndexTerm> {
    let mut values = Vec::new();
    for key in ["indexTerms", "index_terms", "index.terms", "index.keywords"] {
        collect_metadata_index_terms(metadata_lookup(metadata, key), &mut values);
    }
    values
}

fn metadata_string_values(metadata: &Value, keys: &[&str]) -> Vec<String> {
    let mut values = Vec::new();
    for key in keys {
        collect_metadata_string_values(metadata_lookup(metadata, key), &mut values);
    }
    values
}

fn collect_metadata_string_values(value: Option<&Value>, values: &mut Vec<String>) {
    match value {
        Some(Value::Array(items)) => {
            for item in items {
                collect_metadata_string_values(Some(item), values);
            }
        }
        Some(Value::String(value)) => {
            for term in value.split(',') {
                let term = term.trim();
                if !term.is_empty() {
                    values.push(term.to_string());
                }
            }
        }
        _ => {}
    }
}

fn collect_metadata_index_terms(value: Option<&Value>, values: &mut Vec<MetadataIndexTerm>) {
    match value {
        Some(Value::Array(items)) => {
            for item in items {
                collect_metadata_index_terms(Some(item), values);
            }
        }
        Some(Value::String(value)) => {
            for term in value.split(',') {
                let term = term.trim();
                if !term.is_empty() {
                    values.push(MetadataIndexTerm {
                        term: term.to_string(),
                        anchor: None,
                    });
                }
            }
        }
        Some(Value::Object(object)) => {
            let term = ["term", "name", "title", "label", "value"]
                .iter()
                .find_map(|key| object.get(*key).and_then(Value::as_str))
                .map(str::trim)
                .filter(|term| !term.is_empty());
            if let Some(term) = term {
                let anchor = ["anchor", "target", "ref", "section"]
                    .iter()
                    .find_map(|key| object.get(*key).and_then(Value::as_str))
                    .map(|anchor| anchor.trim().trim_start_matches('#').to_string())
                    .filter(|anchor| !anchor.is_empty());
                values.push(MetadataIndexTerm {
                    term: term.to_string(),
                    anchor,
                });
            }
        }
        _ => {}
    }
}

fn index_exclude_key(term: &str) -> String {
    term.trim().to_ascii_lowercase()
}

fn explicit_index_terms(line: &str) -> Vec<String> {
    line.split("{#index:")
        .skip(1)
        .filter_map(|segment| {
            segment
                .split_once('}')
                .map(|(term, _)| term.trim().to_string())
        })
        .collect()
}

fn bold_index_terms(line: &str) -> Vec<String> {
    line.split("**")
        .skip(1)
        .step_by(2)
        .map(str::trim)
        .filter(|term| !term.is_empty() && term.len() <= 80)
        .map(ToString::to_string)
        .collect()
}

fn proper_noun_candidates(line: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut current = Vec::new();
    for raw in line.split_whitespace() {
        let token = raw
            .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-' && ch != '&')
            .trim();
        if token.len() > 2
            && token
                .chars()
                .next()
                .map(|ch| ch.is_uppercase())
                .unwrap_or(false)
            && !index_stop_word(token)
        {
            current.push(token.to_string());
        } else {
            push_proper_noun_candidate(&mut candidates, &mut current);
        }
    }
    push_proper_noun_candidate(&mut candidates, &mut current);
    candidates
}

fn push_proper_noun_candidate(candidates: &mut Vec<String>, current: &mut Vec<String>) {
    if current.is_empty() {
        return;
    }
    if current.len() == 1 || current.iter().map(String::len).sum::<usize>() <= 80 {
        candidates.push(current.join(" "));
    }
    current.clear();
}

fn index_stop_word(token: &str) -> bool {
    matches!(
        token,
        "The" | "This" | "That" | "These" | "Those" | "Prepared" | "Expected" | "Figure" | "Table"
    )
}

fn first_term_anchor(text: &str, headings: &[Heading], term: &str) -> Option<String> {
    let mut heading_index = 0usize;
    let mut current_anchor = headings.first().map(|heading| heading.anchor.clone());
    let mut fence_marker: Option<String> = None;
    for (zero_index, line) in text.lines().enumerate() {
        let line_number = zero_index + 1;
        while heading_index < headings.len() && headings[heading_index].line <= line_number {
            current_anchor = Some(headings[heading_index].anchor.clone());
            heading_index += 1;
        }
        if let Some(ref marker) = fence_marker {
            if line.trim_start().starts_with(marker.as_str()) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            fence_marker = Some(marker);
            continue;
        }
        if line.contains(term) {
            return current_anchor;
        }
    }
    None
}

fn escape_markdown_link_text(text: &str) -> String {
    text.replace('[', "\\[").replace(']', "\\]")
}
