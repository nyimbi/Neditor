use crate::{
    bibliography::{BibliographyEntry, CitationReference},
    diagnostics::{diag, with_range, DocumentDiagnostic},
    document_ast::DocumentBlock,
    layout::{
        layout_margins_option, layout_option_text_any, layout_orientation_option,
        layout_page_size_option,
    },
    metadata_string,
    provenance::{AiAssistedSection, AiSource},
    review::ReviewComment,
    source_mapping::diagnostic_location_for_generated_line,
    SourceMapEntry,
};
use serde_json::Value;
use std::collections::HashSet;

pub(crate) struct DocumentValidationInput<'a> {
    pub(crate) metadata: &'a Value,
    pub(crate) citation_references: &'a [CitationReference],
    pub(crate) bibliography: &'a [BibliographyEntry],
    pub(crate) duplicate_bibliography_keys: &'a [String],
    pub(crate) comments: &'a [ReviewComment],
    pub(crate) ai_sources: &'a [AiSource],
    pub(crate) ai_assisted_sections: &'a [AiAssistedSection],
    pub(crate) has_bibliography_source: bool,
    pub(crate) source_map: &'a [SourceMapEntry],
}

pub(crate) fn validate_document(
    input: DocumentValidationInput<'_>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let metadata = input.metadata;
    if metadata
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .is_empty()
    {
        diagnostics.push(diag(
            "warning",
            "Missing title metadata.",
            None,
            None,
            Some("Add title to YAML front matter."),
        ));
    }
    if metadata
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("")
        .is_empty()
    {
        diagnostics.push(diag(
            "warning",
            "Missing version metadata.",
            None,
            None,
            Some("Add version to YAML front matter for export traceability."),
        ));
    }
    if metadata
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft")
        == "draft"
    {
        diagnostics.push(diag(
            "warning",
            "Document status is draft.",
            None,
            None,
            Some("Set status to approved or published before final export."),
        ));
    }
    if let Some(status) = metadata.get("status").and_then(Value::as_str) {
        if !matches!(
            status,
            "draft" | "in-review" | "approved" | "published" | "archived"
        ) {
            diagnostics.push(diag(
                "warning",
                format!("Invalid document status: {status}"),
                None,
                None,
                Some("Use draft, in-review, approved, published, or archived."),
            ));
        }
    }
    let release_status = matches!(
        metadata.get("status").and_then(Value::as_str),
        Some("approved" | "published")
    );
    if release_status {
        let approved_by_missing = metadata
            .get("approvedBy")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty();
        let approved_at_missing = metadata
            .get("approvedAt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty();
        if approved_by_missing || approved_at_missing {
            diagnostics.push(diag(
                "warning",
                "Approved or published document is missing approval metadata.",
                None,
                None,
                Some("Add approvedBy and approvedAt front matter for release auditability."),
            ));
        }
    }
    validate_layout_metadata(metadata, diagnostics);
    let known_keys = input
        .bibliography
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<HashSet<_>>();
    for key in input.duplicate_bibliography_keys {
        diagnostics.push(diag(
            "error",
            format!("Duplicate bibliography key: {key}"),
            None,
            None,
            Some("Keep bibliography keys unique so citations resolve deterministically."),
        ));
    }
    if !input.citation_references.is_empty() && !input.has_bibliography_source {
        let first = &input.citation_references[0];
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, first.line);
        let mut diagnostic = with_range(
            diag(
                "warning",
                "Document contains citations but no bibliography source.",
                source_file,
                line,
                Some("Add bibliography front matter, a bibtex fence, or a bibliography marker."),
            ),
            first.column,
            line,
            first.end_column,
        );
        diagnostic
            .related
            .push(format!("First citation: {}", first.raw));
        diagnostics.push(diagnostic);
    }
    let mut reported_broken_citations = HashSet::new();
    for reference in input.citation_references {
        if !known_keys.is_empty()
            && !known_keys.contains(reference.key.as_str())
            && reported_broken_citations.insert(reference.key.as_str())
        {
            let (source_file, line) =
                diagnostic_location_for_generated_line(input.source_map, reference.line);
            let mut diagnostic = with_range(
                diag(
                    "error",
                    format!("Broken citation: {}", reference.key),
                    source_file,
                    line,
                    Some("Add the key to a BibTeX or CSL bibliography source."),
                ),
                reference.column,
                line,
                reference.end_column,
            );
            diagnostic
                .related
                .push(format!("Citation syntax: {}", reference.raw));
            diagnostics.push(diagnostic);
        }
    }
    if let Some(comment) = input
        .comments
        .iter()
        .find(|comment| comment.state != "resolved")
    {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, comment.line);
        let mut diagnostic = diag(
            if release_status { "error" } else { "warning" },
            "Document has unresolved review comments.",
            source_file,
            line,
            Some("Resolve comments before publishing."),
        );
        diagnostic.related.push(format!(
            "First unresolved comment by {}: {}",
            comment.author, comment.text
        ));
        diagnostics.push(diagnostic);
    }
    if let Some(line_number) = first_pending_ai_review_line(&input) {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, line_number);
        diagnostics.push(diag(
            if release_status { "error" } else { "warning" },
            "Document has AI-assisted sections that are not human-reviewed.",
            source_file,
            line,
            Some("Mark AI source blocks and AI-assisted section markers as human-reviewed after review."),
        ));
    }
}

pub(crate) fn validate_layout_directives(
    blocks: &[DocumentBlock],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    for block in blocks {
        let DocumentBlock::Layout {
            options, source, ..
        } = block
        else {
            continue;
        };
        let source_file = source.as_ref().map(|range| range.source_file.clone());
        let line = source.as_ref().map(|range| range.source_line);
        let cases = [
            (
                &["pageSize", "page-size", "page_size", "paper", "size"] as &[&str],
                layout_page_size_option(options),
                "pageSize",
                "Use A4, Letter, or Legal.",
            ),
            (
                &["orientation", "pageOrientation", "page_orientation"] as &[&str],
                layout_orientation_option(options),
                "orientation",
                "Use portrait or landscape.",
            ),
            (
                &["margins", "margin", "pageMargins", "page_margins"] as &[&str],
                layout_margins_option(options),
                "margins",
                "Use narrow, normal, wide, or compact.",
            ),
        ];
        for (keys, normalized, label, suggestion) in cases {
            if normalized.is_some() {
                continue;
            }
            let Some(raw) = layout_option_text_any(options, keys) else {
                continue;
            };
            diagnostics.push(diag(
                "warning",
                format!("Unsupported layout directive {label}: {raw}"),
                source_file.clone(),
                line,
                Some(suggestion),
            ));
        }
    }
}

fn first_pending_ai_review_line(input: &DocumentValidationInput<'_>) -> Option<usize> {
    input
        .ai_sources
        .iter()
        .find(|source| source.status != "human-reviewed")
        .map(|source| source.line)
        .or_else(|| {
            input
                .ai_assisted_sections
                .iter()
                .find(|section| section.status != "human-reviewed")
                .map(|section| section.line)
        })
}

fn validate_layout_metadata(metadata: &Value, diagnostics: &mut Vec<DocumentDiagnostic>) {
    if let Some(page_size) = metadata_string(metadata, "layout.pageSize")
        .or_else(|| metadata_string(metadata, "pageSize"))
    {
        let normalized = page_size.to_ascii_lowercase().replace([' ', '-'], "");
        if !matches!(
            normalized.as_str(),
            "a4" | "letter" | "usletter" | "legal" | "uslegal"
        ) {
            diagnostics.push(diag(
                "warning",
                format!("Unsupported layout pageSize: {page_size}"),
                None,
                None,
                Some("Use A4, Letter, or Legal."),
            ));
        }
    }
    if let Some(margins) =
        metadata_string(metadata, "layout.margins").or_else(|| metadata_string(metadata, "margins"))
    {
        let normalized = margins.to_ascii_lowercase().replace([' ', '-'], "");
        if !matches!(
            normalized.as_str(),
            "narrow" | "compact" | "normal" | "wide"
        ) {
            diagnostics.push(diag(
                "warning",
                format!("Unsupported layout margins: {margins}"),
                None,
                None,
                Some("Use narrow, normal, wide, or compact."),
            ));
        }
    }
    if let Some(orientation) = metadata_string(metadata, "layout.orientation")
        .or_else(|| metadata_string(metadata, "orientation"))
    {
        let normalized = orientation.to_ascii_lowercase().replace([' ', '-'], "");
        if !matches!(normalized.as_str(), "portrait" | "landscape") {
            diagnostics.push(diag(
                "warning",
                format!("Unsupported layout orientation: {orientation}"),
                None,
                None,
                Some("Use portrait or landscape."),
            ));
        }
    }
}
