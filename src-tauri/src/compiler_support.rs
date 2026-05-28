use crate::{
    compiler_types::Heading,
    document_ast::{containing_byte_range, extract_label, inline_code_spans, slugify},
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

pub(crate) fn citation_style(metadata: &Value) -> &'static str {
    let style = citation_style_value(metadata).unwrap_or("title");
    canonical_citation_style(style).unwrap_or("title")
}

pub(crate) fn citation_style_value(metadata: &Value) -> Option<&str> {
    metadata
        .get("citationStyle")
        .or_else(|| metadata.get("cslStyle"))
        .or_else(|| metadata.get("citation_style"))
        .and_then(Value::as_str)
}

pub(crate) fn supported_citation_style(style: &str) -> bool {
    canonical_citation_style(style).is_some()
}

pub(crate) fn canonical_citation_style(style: &str) -> Option<&'static str> {
    let normalized = style.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "title" | "neditor-title" => Some("title"),
        "author-year" | "author_year" | "harvard" | "council-of-science-editors-author-date" => {
            Some("author-year")
        }
        "apa" | "american-psychological-association" => Some("apa"),
        "chicago-author-date" | "chicago" => Some("chicago-author-date"),
        "mla" | "modern-language-association" => Some("mla"),
        "key" | "citation-key" | "citation_key" => Some("key"),
        "numeric" => Some("numeric"),
        "ieee" => Some("ieee"),
        "vancouver" => Some("vancouver"),
        "nature" => Some("nature"),
        "american-medical-association" | "ama" => Some("ama"),
        "elsevier-vancouver" => Some("elsevier-vancouver"),
        _ => None,
    }
}

pub(crate) fn collect_fence_bodies(text: &str, target: &str) -> Vec<String> {
    collect_fence_bodies_with_lines(text, target)
        .into_iter()
        .map(|(_, body)| body)
        .collect()
}

fn strip_heading_attributes(text: &str) -> &str {
    let code_spans = inline_code_spans(text);
    let mut last = None;
    let mut search_from = 0usize;
    while let Some(relative_start) = text[search_from..].find("{#") {
        let start = search_from + relative_start;
        if let Some((_, end)) = containing_byte_range(start, &code_spans) {
            search_from = end;
            continue;
        }
        let key_start = start + 2;
        let Some(relative_end) = text[key_start..].find('}') else {
            break;
        };
        last = Some(start);
        search_from = key_start + relative_end + 1;
    }
    last.map(|index| text[..index].trim())
        .unwrap_or_else(|| text.trim())
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
        if let Some(marker) = fenced_code_marker(line) {
            let info = line.trim_start().strip_prefix(marker).unwrap_or("").trim();
            if info.split_whitespace().next().unwrap_or("") != target {
                for (_, body_line) in lines.by_ref() {
                    if body_line.trim_start().starts_with(marker) {
                        break;
                    }
                }
                continue;
            }
            let mut body = String::new();
            for (_, body_line) in lines.by_ref() {
                if body_line.trim_start().starts_with(marker) {
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
