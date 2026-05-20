use crate::{
    compiler_support::fenced_code_marker,
    diagnostics::{diag, with_range},
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

pub(crate) fn collect_labels(text: &str, heading_anchors: &[&str]) -> Vec<String> {
    let mut labels = BTreeSet::new();
    for anchor in heading_anchors {
        labels.insert((*anchor).to_string());
    }
    let mut fence_marker = None;
    for line in text.lines() {
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
        for segment in line.split("{#").skip(1) {
            if let Some((label, _)) = segment.split_once('}') {
                let label = label.split_whitespace().next().unwrap_or("").trim();
                if !label.is_empty() {
                    labels.insert(label.to_string());
                }
            }
        }
    }
    labels.into_iter().collect()
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
                break;
            };
            let key_end = key_start + relative_end;
            let key = line[key_start..key_end].trim().to_string();
            if key.is_empty() {
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
