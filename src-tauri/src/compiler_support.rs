use crate::{
    compiler_types::Heading,
    document_ast::{extract_label, slugify},
};
use serde_json::Value;
use std::collections::BTreeMap;

pub(crate) fn extract_headings(text: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut fence_marker = None;
    for (index, line) in text.lines().enumerate() {
        if let Some(marker) = fence_marker {
            if line.trim_start().starts_with(marker) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            fence_marker = Some(marker);
            continue;
        }
        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|ch| *ch == '#').count();
        if !(1..=6).contains(&level) || trimmed.chars().nth(level) != Some(' ') {
            continue;
        }
        let raw_text = trimmed[level..].trim();
        let text = strip_heading_attributes(raw_text).to_string();
        if text.is_empty() {
            continue;
        }
        headings.push(Heading {
            level,
            anchor: extract_label(raw_text).unwrap_or_else(|| slugify(&text)),
            text,
            line: index + 1,
        });
    }
    headings
}

pub(crate) fn collect_glossary(text: &str) -> BTreeMap<String, String> {
    let mut glossary = BTreeMap::new();
    for body in collect_fence_bodies(text, "glossary") {
        for line in body.lines() {
            if let Some((term, definition)) = line.split_once(':') {
                glossary.insert(term.trim().to_string(), definition.trim().to_string());
            }
        }
    }
    glossary
}

pub(crate) fn citation_style(metadata: &Value) -> &str {
    metadata
        .get("citationStyle")
        .or_else(|| metadata.get("cslStyle"))
        .or_else(|| metadata.get("citation_style"))
        .and_then(Value::as_str)
        .unwrap_or("title")
}

pub(crate) fn collect_fence_bodies(text: &str, target: &str) -> Vec<String> {
    collect_fence_bodies_with_lines(text, target)
        .into_iter()
        .map(|(_, body)| body)
        .collect()
}

fn strip_heading_attributes(text: &str) -> &str {
    text.split("{#").next().unwrap_or(text).trim()
}

pub(crate) fn fenced_code_marker(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

pub(crate) fn collect_fence_bodies_with_lines(text: &str, target: &str) -> Vec<(usize, String)> {
    let mut bodies = Vec::new();
    let mut lines = text.lines().enumerate();
    while let Some((line_index, line)) = lines.next() {
        if line
            .trim()
            .strip_prefix("```")
            .map(|info| info.split_whitespace().next().unwrap_or("") == target)
            .unwrap_or(false)
        {
            let mut body = String::new();
            for (_, body_line) in lines.by_ref() {
                if body_line.trim() == "```" {
                    break;
                }
                body.push_str(body_line);
                body.push('\n');
            }
            bodies.push((line_index + 1, body));
        }
    }
    bodies
}
