use crate::{
    compiler_support::fenced_code_marker,
    compiler_types::Heading,
    diagnostics::{diag, with_range},
    document_ast::is_valid_reference_key,
    source_mapping::diagnostic_location_for_generated_line,
    DocumentDiagnostic, SourceMapEntry,
};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Serialize)]
pub(crate) struct CrossReference {
    pub(crate) key: String,
    pub(crate) target_kind: String,
    pub(crate) resolved: bool,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) end_column: usize,
    pub(crate) source_file: Option<String>,
}

#[derive(Clone)]
struct LabelOccurrence {
    key: String,
    source_file: Option<String>,
    line: Option<usize>,
    column: usize,
    end_column: usize,
    origin: &'static str,
}

pub(crate) fn collect_labels(
    text: &str,
    headings: &[Heading],
    source_map: &[SourceMapEntry],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<String> {
    let mut labels = BTreeSet::new();
    let mut occurrences = Vec::new();
    for heading in headings {
        labels.insert(heading.anchor.clone());
        let (source_file, source_line) =
            diagnostic_location_for_generated_line(source_map, heading.line);
        occurrences.push(LabelOccurrence {
            key: heading.anchor.clone(),
            source_file,
            line: source_line,
            column: 1,
            end_column: heading.text.len().max(1),
            origin: "heading",
        });
    }
    let mut fence_marker = None;
    for (line_index, line) in text.lines().enumerate() {
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
        let generated_line = line_index + 1;
        let (source_file, source_line) =
            diagnostic_location_for_generated_line(source_map, generated_line);
        let collect_occurrences = !line_is_markdown_heading(line);
        scan_reference_labels(
            line,
            source_file,
            source_line,
            collect_occurrences,
            &mut labels,
            &mut occurrences,
            diagnostics,
        );
    }
    validate_unique_labels(&occurrences, diagnostics);
    labels.into_iter().collect()
}

fn scan_reference_labels(
    line: &str,
    source_file: Option<String>,
    source_line: Option<usize>,
    collect_occurrences: bool,
    labels: &mut BTreeSet<String>,
    occurrences: &mut Vec<LabelOccurrence>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let mut search_from = 0usize;
    while let Some(relative_start) = line[search_from..].find("{#") {
        let start = search_from + relative_start;
        let key_start = start + 2;
        let Some(relative_end) = line[key_start..].find('}') else {
            push_unclosed_reference_marker_diagnostic(
                "label",
                "{#",
                line,
                start,
                source_file.clone(),
                source_line,
                diagnostics,
            );
            break;
        };
        let raw_key_end = key_start + relative_end;
        let raw_label = &line[key_start..raw_key_end];
        let parsed_label = parse_reference_label(raw_label);
        if let Some(display_label) = parsed_label.invalid_display {
            push_malformed_reference_key_diagnostic(
                "label",
                display_label,
                start,
                raw_key_end + 1,
                source_file.clone(),
                source_line,
                diagnostics,
            );
        } else if let Some(label) = parsed_label.key {
            labels.insert(label.to_string());
            if collect_occurrences {
                occurrences.push(LabelOccurrence {
                    key: label.to_string(),
                    source_file: source_file.clone(),
                    line: source_line,
                    column: start + 1,
                    end_column: key_start + parsed_label.key_end + 1,
                    origin: "label",
                });
            }
        }
        search_from = raw_key_end + 1;
    }
}

struct ParsedLabel<'a> {
    key: Option<&'a str>,
    key_end: usize,
    invalid_display: Option<&'a str>,
}

fn parse_reference_label(raw_label: &str) -> ParsedLabel<'_> {
    let trimmed = raw_label.trim();
    if trimmed.is_empty() {
        return ParsedLabel {
            key: None,
            key_end: 0,
            invalid_display: Some(""),
        };
    }
    let leading_whitespace = raw_label.len() - raw_label.trim_start().len();
    let mut split = trimmed.splitn(2, char::is_whitespace);
    let key = split.next().unwrap_or("");
    let rest = split.next().unwrap_or("").trim();
    let key_end = leading_whitespace + key.len();
    let rest_is_attributes = rest.is_empty() || rest.contains('=');
    if !rest_is_attributes {
        return ParsedLabel {
            key: None,
            key_end,
            invalid_display: Some(trimmed),
        };
    }
    if !is_valid_reference_key(key) {
        return ParsedLabel {
            key: None,
            key_end,
            invalid_display: Some(key),
        };
    }
    ParsedLabel {
        key: Some(key),
        key_end,
        invalid_display: None,
    }
}

fn validate_unique_labels(
    occurrences: &[LabelOccurrence],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let mut first_by_key = HashMap::<&str, &LabelOccurrence>::new();
    for occurrence in occurrences {
        if let Some(first) = first_by_key.get(occurrence.key.as_str()) {
            let mut diagnostic = with_range(
                diag(
                    "error",
                    format!("Duplicate reference label: {}", occurrence.key),
                    occurrence.source_file.clone(),
                    occurrence.line,
                    Some("Rename one label so cross references resolve to one stable target."),
                ),
                occurrence.column,
                occurrence.line,
                occurrence.end_column,
            );
            diagnostic.related.push(format!(
                "First occurrence: {}:{}",
                first
                    .source_file
                    .clone()
                    .unwrap_or_else(|| "document".to_string()),
                first.line.unwrap_or_default()
            ));
            diagnostic
                .related
                .push(format!("First origin: {}", first.origin));
            diagnostic
                .related
                .push(format!("Duplicate origin: {}", occurrence.origin));
            diagnostics.push(diagnostic);
        } else {
            first_by_key.insert(occurrence.key.as_str(), occurrence);
        }
    }
}

fn line_is_markdown_heading(line: &str) -> bool {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    (1..=6).contains(&level) && trimmed.chars().nth(level) == Some(' ')
}

pub(crate) fn collect_cross_references(
    text: &str,
    labels: &[String],
    source_map: &[SourceMapEntry],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Vec<CrossReference> {
    let known = labels.iter().map(String::as_str).collect::<HashSet<_>>();
    let mut references = Vec::new();
    let mut fence_marker = None;
    for (line_index, line) in text.lines().enumerate() {
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
        let generated_line = line_index + 1;
        let mut search_from = 0usize;
        while let Some(relative_start) = line[search_from..].find("{@") {
            let start = search_from + relative_start;
            let key_start = start + 2;
            let Some(relative_end) = line[key_start..].find('}') else {
                let (source_file, source_line) =
                    diagnostic_location_for_generated_line(source_map, generated_line);
                push_unclosed_reference_marker_diagnostic(
                    "cross reference",
                    "{@",
                    line,
                    start,
                    source_file,
                    source_line,
                    diagnostics,
                );
                break;
            };
            let key_end = key_start + relative_end;
            let key = line[key_start..key_end].trim().to_string();
            if !is_valid_reference_key(&key) {
                let (source_file, source_line) =
                    diagnostic_location_for_generated_line(source_map, generated_line);
                push_malformed_reference_key_diagnostic(
                    "cross reference",
                    &key,
                    start,
                    key_end + 1,
                    source_file,
                    source_line,
                    diagnostics,
                );
                search_from = key_end + 1;
                continue;
            }
            let column = start + 1;
            let end_column = key_end + 2;
            let resolved = known.contains(key.as_str());
            let (source_file, source_line) =
                diagnostic_location_for_generated_line(source_map, generated_line);
            if !resolved {
                let mut diagnostic = with_range(
                    diag(
                        "error",
                        format!("Broken cross reference: {key}"),
                        source_file.clone(),
                        source_line,
                        Some(
                            "Add a matching label such as {#fig:name}, {#tbl:name}, or {#eq:name}.",
                        ),
                    ),
                    column,
                    source_line,
                    end_column,
                );
                diagnostic
                    .related
                    .push(format!("Reference syntax: {{@{key}}}"));
                diagnostics.push(diagnostic);
            }
            references.push(CrossReference {
                target_kind: key
                    .split_once(':')
                    .map(|(kind, _)| kind.to_string())
                    .unwrap_or_else(|| "section".to_string()),
                key,
                resolved,
                line: source_line.unwrap_or(generated_line),
                column,
                end_column,
                source_file,
            });
            search_from = key_end + 1;
        }
    }
    references
}

fn push_malformed_reference_key_diagnostic(
    marker_type: &str,
    key: &str,
    start: usize,
    raw_end: usize,
    source_file: Option<String>,
    source_line: Option<usize>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let display_key = if key.is_empty() { "<empty>" } else { key };
    let mut diagnostic = with_range(
        diag(
            "error",
            format!("Malformed reference {marker_type}: {display_key}"),
            source_file,
            source_line,
            Some(
                "Use only letters, numbers, colon, underscore, dash, or period in reference keys.",
            ),
        ),
        start + 1,
        source_line,
        raw_end + 1,
    );
    diagnostic.related.push(format!(
        "Allowed reference key pattern: {}",
        "A-Z a-z 0-9 : _ - ."
    ));
    diagnostics.push(diagnostic);
}

fn push_unclosed_reference_marker_diagnostic(
    marker_type: &str,
    marker: &str,
    line: &str,
    start: usize,
    source_file: Option<String>,
    source_line: Option<usize>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let mut diagnostic = with_range(
        diag(
            "error",
            format!("Unclosed reference {marker_type} marker: {marker}"),
            source_file,
            source_line,
            Some("Close the reference marker with } or remove the incomplete marker."),
        ),
        start + 1,
        source_line,
        line.len() + 1,
    );
    diagnostic
        .related
        .push(format!("Marker starts at column {}", start + 1));
    diagnostics.push(diagnostic);
}

pub(crate) fn render_cross_references(markdown: &str, references: &[CrossReference]) -> String {
    if references.is_empty() {
        return markdown.to_string();
    }
    let reference_map = references
        .iter()
        .map(|reference| (reference.key.as_str(), reference))
        .collect::<HashMap<_, _>>();
    let mut fence_marker = None;
    let mut lines = Vec::new();
    for line in markdown.lines() {
        if let Some(marker) = fence_marker {
            lines.push(line.to_string());
            if line.trim_start().starts_with(marker) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            lines.push(line.to_string());
            fence_marker = Some(marker);
            continue;
        }
        lines.push(render_cross_reference_line(line, &reference_map));
    }
    let mut output = lines.join("\n");
    if markdown.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn render_cross_reference_line(
    line: &str,
    reference_map: &HashMap<&str, &CrossReference>,
) -> String {
    let mut output = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find("{@") {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find('}') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let key = after_start[..end].trim();
        if let Some(reference) = reference_map
            .get(key)
            .filter(|reference| reference.resolved)
        {
            output.push_str(&format!(
                "[{}](#{})",
                escape_markdown_link_text(&reference_display_text(reference)),
                escape_markdown_link_target(&reference.key)
            ));
        } else {
            output.push_str(&rest[start..start + 2 + end + 1]);
        }
        rest = &after_start[end + 1..];
    }
    output.push_str(rest);
    output
}

fn reference_display_text(reference: &CrossReference) -> String {
    if !reference.key.contains(':') {
        return unprefixed_reference_display_text(&reference.key);
    }
    let (label, suffix) = reference
        .key
        .split_once(':')
        .map(|(kind, suffix)| (reference_kind_label(kind), suffix))
        .unwrap_or(("Section", reference.key.as_str()));
    if suffix.is_empty() {
        label.to_string()
    } else {
        format!("{label} {}", suffix.replace(['-', '_'], " "))
    }
}

fn unprefixed_reference_display_text(key: &str) -> String {
    let normalized = key.replace(['-', '_'], " ");
    for (prefix, label) in [("appendix ", "Appendix"), ("decision ", "Decision")] {
        if let Some(suffix) = normalized.strip_prefix(prefix) {
            return format!("{label} {}", title_case_reference_suffix(suffix));
        }
    }
    format!("Section {normalized}")
}

fn title_case_reference_suffix(suffix: &str) -> String {
    suffix
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn reference_kind_label(kind: &str) -> &'static str {
    match kind {
        "fig" | "figure" => "Figure",
        "tbl" | "table" => "Table",
        "eq" | "equation" => "Equation",
        "app" | "appendix" => "Appendix",
        "dec" | "decision" => "Decision",
        "sec" | "section" => "Section",
        _ => "Reference",
    }
}

fn escape_markdown_link_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

fn escape_markdown_link_target(target: &str) -> String {
    target.replace(')', "%29").replace(' ', "%20")
}
