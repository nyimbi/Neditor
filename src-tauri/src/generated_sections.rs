use crate::{
    bibliography::BibliographyEntry,
    compiler_support::{citation_style, fenced_code_marker},
    document_ast::{extract_label, extract_quoted_attribute},
    indexing::{render_index_entries, IndexEntry},
    metadata_lookup, Heading,
};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug)]
struct CaptionEntry {
    id: Option<String>,
    label: String,
}

pub(crate) fn inject_generated_sections(
    text: &str,
    metadata: &Value,
    headings: &[Heading],
    index_entries: &[IndexEntry],
    bibliography: &[BibliographyEntry],
    glossary: &BTreeMap<String, String>,
) -> String {
    let wants_toc = text.contains("[TOC]")
        || metadata_bool(
            metadata,
            &[
                "toc",
                "toc.enabled",
                "tableOfContents",
                "tableOfContents.enabled",
            ],
        );
    let mut output = text.to_string();
    if wants_toc {
        let toc = render_toc(
            headings,
            toc_depth(metadata),
            toc_numbering_enabled(metadata),
        );
        output = output.replace("[TOC]", &format!("## Table of Contents\n\n{toc}"));
        if !text.contains("[TOC]") {
            output = format!("## Table of Contents\n\n{toc}\n\n{output}");
        }
    }
    if output.contains("[LIST_OF_FIGURES]")
        || metadata_bool(metadata, &["listOfFigures", "list_of_figures"])
    {
        let list = render_caption_list("Figure", &collect_figure_entries(&output));
        output = output.replace(
            "[LIST_OF_FIGURES]",
            &format!("## List of Figures\n\n{list}"),
        );
        if !text.contains("[LIST_OF_FIGURES]") {
            output = format!("## List of Figures\n\n{list}\n\n{output}");
        }
    }
    if output.contains("[LIST_OF_TABLES]")
        || metadata_bool(metadata, &["listOfTables", "list_of_tables"])
    {
        let list = render_caption_list("Table", &collect_table_entries(&output));
        output = output.replace("[LIST_OF_TABLES]", &format!("## List of Tables\n\n{list}"));
        if !text.contains("[LIST_OF_TABLES]") {
            output = format!("## List of Tables\n\n{list}\n\n{output}");
        }
    }
    if output.contains("[INDEX]") || generated_index_section_requested(metadata) {
        let index = render_index_entries(index_entries);
        output = output.replace("[INDEX]", &format!("## Index\n\n{index}"));
        if !text.contains("[INDEX]") {
            output = format!("## Index\n\n{index}\n\n{output}");
        }
    }
    if output.contains("[GLOSSARY]") || generated_glossary_section_requested(metadata) {
        let section = render_glossary_entries(glossary);
        output = output.replace("[GLOSSARY]", &format!("## Glossary\n\n{section}"));
        if !text.contains("[GLOSSARY]") {
            output = format!("## Glossary\n\n{section}\n\n{output}");
        }
    }
    if output.contains("[BIBLIOGRAPHY]") {
        let references = render_bibliography_entries(bibliography, citation_style(metadata));
        output = output.replace(
            "[BIBLIOGRAPHY]",
            &format!("## Bibliography\n\n{references}"),
        );
    }
    output
}

fn render_glossary_entries(glossary: &BTreeMap<String, String>) -> String {
    if glossary.is_empty() {
        return "_No glossary terms found._".to_string();
    }
    glossary
        .iter()
        .map(|(term, definition)| format!("- **{term}**: {definition}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_bibliography_entries(bibliography: &[BibliographyEntry], style: &str) -> String {
    bibliography
        .iter()
        .enumerate()
        .map(|(index, entry)| match style {
            "numeric" => format!("- [{}] **{}**. {}", index + 1, entry.key, entry.title),
            "author-year" => {
                let author_year = [entry.author.as_deref(), entry.issued.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ");
                if author_year.is_empty() {
                    format!("- **{}**. {}", entry.key, entry.title)
                } else {
                    format!("- **{}**. {}. {}", entry.key, author_year, entry.title)
                }
            }
            _ => format!("- **{}**. {}", entry.key, entry.title),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn metadata_bool(metadata: &Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        metadata_lookup(metadata, key)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    })
}

fn metadata_u64(metadata: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| metadata_lookup(metadata, key).and_then(Value::as_u64))
}

pub(crate) fn generated_index_section_requested(metadata: &Value) -> bool {
    metadata_bool(metadata, &["indexSection", "index_section"])
        || metadata_bool(metadata, &["index", "index.enabled", "index.section"])
}

pub(crate) fn generated_glossary_section_requested(metadata: &Value) -> bool {
    metadata_bool(
        metadata,
        &["glossary", "glossarySection", "glossary_section"],
    )
}

fn render_caption_list(kind: &str, entries: &[CaptionEntry]) -> String {
    if entries.is_empty() {
        return "- No entries.".to_string();
    }
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let text = format!("{kind} {}: {}", index + 1, entry.label);
            if let Some(id) = &entry.id {
                format!("- [{text}](#{id})")
            } else {
                format!("- {text}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_figure_entries(text: &str) -> Vec<CaptionEntry> {
    let mut entries = Vec::new();
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
        if let Some(entry) = figure_entry_from_line(line) {
            entries.push(entry);
        }
    }
    entries
}

fn figure_entry_from_line(line: &str) -> Option<CaptionEntry> {
    let image_start = line.find("![")?;
    let alt_start = image_start + 2;
    let alt_end = line[alt_start..].find(']')? + alt_start;
    let alt = clean_inline_text(&line[alt_start..alt_end]);
    let after_alt = line[alt_end..].trim_start();
    if !after_alt.starts_with("](") {
        return None;
    }
    let attrs = line[alt_end..].split_once('{').map(|(_, attrs)| attrs)?;
    let attrs = attrs.trim_end_matches('}').trim();
    let caption = extract_quoted_attribute(attrs, "caption")
        .map(|value| clean_inline_text(&value))
        .filter(|value| !value.is_empty())
        .or_else(|| (!alt.is_empty()).then_some(alt))?;
    let labelled_attrs = ["{", attrs, "}"].concat();
    Some(CaptionEntry {
        id: extract_label(&labelled_attrs),
        label: caption,
    })
}

fn collect_table_entries(text: &str) -> Vec<CaptionEntry> {
    let mut entries = Vec::new();
    let mut fence_marker = None;
    let mut pending_caption: Option<CaptionEntry> = None;
    for line in text.lines() {
        if let Some(marker) = fence_marker {
            if line.trim_start().starts_with(marker) {
                fence_marker = None;
            }
            continue;
        }
        if let Some(marker) = fenced_code_marker(line) {
            fence_marker = Some(marker);
            pending_caption = None;
            continue;
        }
        if let Some(caption) = table_caption_entry(line) {
            pending_caption = Some(caption);
            continue;
        }
        if line.trim_start().starts_with('|') {
            if let Some(caption) = pending_caption.take() {
                entries.push(caption);
            }
        } else if !line.trim().is_empty() {
            pending_caption = None;
        }
    }
    entries
}

fn table_caption_entry(line: &str) -> Option<CaptionEntry> {
    let trimmed = line.trim();
    if !trimmed.to_ascii_lowercase().starts_with("table:") {
        return None;
    }
    let id = extract_label(trimmed);
    let label = extract_quoted_attribute(trimmed, "caption")
        .map(|value| clean_inline_text(&value))
        .filter(|value| !value.is_empty())
        .or_else(|| {
            let without_prefix = trimmed.trim_start_matches(|ch: char| ch != ':');
            let without_prefix = without_prefix.trim_start_matches(':').trim();
            let before_attrs = without_prefix.split("{#").next().unwrap_or("").trim();
            (!before_attrs.is_empty()).then(|| clean_inline_text(before_attrs))
        })?;
    Some(CaptionEntry { id, label })
}

fn clean_inline_text(text: &str) -> String {
    strip_html_tags(
        text.split("{#")
            .next()
            .unwrap_or(text)
            .replace("**", "")
            .replace(['*', '`'], "")
            .trim(),
    )
    .trim()
    .to_string()
}

fn strip_html_tags(text: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn render_toc(headings: &[Heading], depth: usize, numbered: bool) -> String {
    let mut counters = [0usize; 6];
    headings
        .iter()
        .filter(|heading| heading.level <= depth)
        .map(|heading| {
            let label = if numbered {
                let number = toc_number_for_heading(heading.level, &mut counters);
                format!("{number} {}", heading.text)
            } else {
                heading.text.clone()
            };
            format!(
                "{}- [{}](#{})",
                "  ".repeat(heading.level.saturating_sub(1)),
                label,
                heading.anchor
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn toc_number_for_heading(level: usize, counters: &mut [usize; 6]) -> String {
    let index = level.saturating_sub(1).min(5);
    counters[index] += 1;
    for counter in counters.iter_mut().skip(index + 1) {
        *counter = 0;
    }
    counters[..=index]
        .iter()
        .copied()
        .filter(|value| *value > 0)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

pub(crate) fn toc_depth(metadata: &Value) -> usize {
    metadata_u64(
        metadata,
        &[
            "tocDepth",
            "toc_depth",
            "toc.depth",
            "tableOfContents.depth",
        ],
    )
    .map(|depth| depth.clamp(1, 6) as usize)
    .unwrap_or(6)
}

fn toc_numbering_enabled(metadata: &Value) -> bool {
    metadata_bool(
        metadata,
        &[
            "tocNumbered",
            "numberedHeadings",
            "toc.numbered",
            "toc.numberedHeadings",
            "tableOfContents.numbered",
        ],
    )
}
