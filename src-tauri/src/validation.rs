use crate::{
    bibliography::{BibliographyEntry, CitationReference},
    diagnostic_location_for_generated_line,
    diagnostics::{diag, DocumentDiagnostic},
    metadata_string,
    provenance::{AiAssistedSection, AiSource},
    review::ReviewComment,
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
        let mut diagnostic = diag(
            "warning",
            "Document contains citations but no bibliography source.",
            source_file,
            line,
            Some("Add bibliography front matter, a bibtex fence, or a bibliography marker."),
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
            let mut diagnostic = diag(
                "error",
                format!("Broken citation: {}", reference.key),
                source_file,
                line,
                Some("Add the key to a BibTeX or CSL bibliography source."),
            );
            diagnostic
                .related
                .push(format!("Citation syntax: {}", reference.raw));
            diagnostics.push(diagnostic);
        }
    }
    if input
        .comments
        .iter()
        .any(|comment| comment.state != "resolved")
    {
        diagnostics.push(diag(
            if release_status { "error" } else { "warning" },
            "Document has unresolved review comments.",
            None,
            None,
            Some("Resolve comments before publishing."),
        ));
    }
    if input
        .ai_sources
        .iter()
        .any(|source| source.status != "human-reviewed")
        || input
            .ai_assisted_sections
            .iter()
            .any(|section| section.status != "human-reviewed")
    {
        diagnostics.push(diag(
            if release_status { "error" } else { "warning" },
            "Document has AI-assisted sections that are not human-reviewed.",
            None,
            None,
            Some("Mark AI source blocks and AI-assisted section markers as human-reviewed after review."),
        ));
    }
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
}
