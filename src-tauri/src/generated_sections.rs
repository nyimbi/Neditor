use crate::{
    bibliography::BibliographyEntry,
    indexing::{render_index_entries, IndexEntry},
    Heading,
};
use serde_json::Value;

pub(crate) fn inject_generated_sections(
    text: &str,
    metadata: &Value,
    headings: &[Heading],
    index_entries: &[IndexEntry],
    bibliography: &[BibliographyEntry],
) -> String {
    let wants_toc = text.contains("[TOC]")
        || metadata
            .get("toc")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        || metadata
            .get("tableOfContents")
            .and_then(Value::as_bool)
            .unwrap_or(false);
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
    if output.contains("[INDEX]") {
        let index = render_index_entries(index_entries);
        output = output.replace("[INDEX]", &format!("## Index\n\n{index}"));
    }
    if output.contains("[BIBLIOGRAPHY]") {
        let references = bibliography
            .iter()
            .map(|entry| format!("- **{}**. {}", entry.key, entry.title))
            .collect::<Vec<_>>()
            .join("\n");
        output = output.replace(
            "[BIBLIOGRAPHY]",
            &format!("## Bibliography\n\n{references}"),
        );
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

fn toc_depth(metadata: &Value) -> usize {
    metadata
        .get("tocDepth")
        .or_else(|| metadata.get("toc_depth"))
        .and_then(Value::as_u64)
        .map(|depth| depth.clamp(1, 6) as usize)
        .unwrap_or(6)
}

fn toc_numbering_enabled(metadata: &Value) -> bool {
    metadata
        .get("tocNumbered")
        .or_else(|| metadata.get("numberedHeadings"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
