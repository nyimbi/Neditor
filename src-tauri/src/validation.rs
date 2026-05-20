use crate::{
    bibliography::{BibliographyEntry, CitationReference},
    compiler_support::{citation_style_value, supported_citation_style},
    diagnostics::{diag, with_range, DocumentDiagnostic},
    document_ast::DocumentBlock,
    layout::{
        layout_margins_option, layout_option_text_any, layout_orientation_option,
        layout_page_size_option,
    },
    metadata_string,
    provenance::{AiAssistedSection, AiSource},
    review::{ChangeNote, ReviewComment},
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
    pub(crate) generated_index_requested: bool,
    pub(crate) index_terms: &'a [String],
    pub(crate) generated_glossary_requested: bool,
    pub(crate) glossary_term_count: usize,
    pub(crate) comments: &'a [ReviewComment],
    pub(crate) change_notes: &'a [ChangeNote],
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
    validate_citation_style_metadata(metadata, diagnostics);
    let known_keys = input
        .bibliography
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<HashSet<_>>();
    for key in input.duplicate_bibliography_keys {
        push_duplicate_bibliography_diagnostic(&input, key, diagnostics);
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
        push_missing_citation_source_diagnostics(&input, diagnostics);
    }
    validate_generated_reference_sections(&input, diagnostics);
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
    if let Some(comment) = input.comments.iter().find(|comment| {
        !comment.has_author || !comment.has_created_at || comment.text.trim().is_empty()
    }) {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, comment.line);
        let mut diagnostic = diag(
            "warning",
            "Review comment is missing audit metadata.",
            source_file,
            line,
            Some("Add author, at timestamp, and comment text before export."),
        );
        diagnostic.related.push(format!(
            "Comment metadata: author={}, at={}",
            if comment.has_author {
                "present"
            } else {
                "missing"
            },
            if comment.has_created_at {
                "present"
            } else {
                "missing"
            }
        ));
        diagnostics.push(diagnostic);
    }
    if let Some(note) = input
        .change_notes
        .iter()
        .find(|note| !note.has_author || !note.has_created_at || note.text.trim().is_empty())
    {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, note.line);
        let mut diagnostic = diag(
            "warning",
            "Change note is missing audit metadata.",
            source_file,
            line,
            Some("Add author, at timestamp, and change note text before export."),
        );
        diagnostic.related.push(format!(
            "Change note metadata: author={}, at={}",
            if note.has_author {
                "present"
            } else {
                "missing"
            },
            if note.has_created_at {
                "present"
            } else {
                "missing"
            }
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
    validate_ai_provenance_metadata(&input, diagnostics);
}

fn validate_generated_reference_sections(
    input: &DocumentValidationInput<'_>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if input.generated_index_requested && input.index_terms.is_empty() {
        let mut diagnostic = diag(
            "warning",
            "Generated index was requested but no index terms were found.",
            None,
            None,
            Some("Add headings, glossary terms, bold key terms, or explicit {#index:Term} markers before final export."),
        );
        diagnostic.related.push("index terms: 0".to_string());
        diagnostics.push(diagnostic);
    }
    if input.generated_glossary_requested && input.glossary_term_count == 0 {
        let mut diagnostic = diag(
            "warning",
            "Generated glossary was requested but no glossary entries were found.",
            None,
            None,
            Some("Add a glossary fenced block or disable the generated glossary section before final export."),
        );
        diagnostic.related.push("glossary entries: 0".to_string());
        diagnostics.push(diagnostic);
    }
}

fn push_missing_citation_source_diagnostics(
    input: &DocumentValidationInput<'_>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let mut reported_missing_citations = HashSet::new();
    for reference in input.citation_references {
        if !reported_missing_citations.insert(reference.key.as_str()) {
            continue;
        }
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, reference.line);
        let mut diagnostic = with_range(
            diag(
                "warning",
                format!("Missing citation bibliography entry: {}", reference.key),
                source_file,
                line,
                Some(
                    "Add a bibliography source that defines this key, or remove the citation before export.",
                ),
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

fn validate_citation_style_metadata(metadata: &Value, diagnostics: &mut Vec<DocumentDiagnostic>) {
    let Some(style) = citation_style_value(metadata) else {
        return;
    };
    if supported_citation_style(style) {
        return;
    }
    diagnostics.push(diag(
        "warning",
        format!("Unsupported citation style: {style}"),
        None,
        None,
        Some("Use title, author-year, key, or numeric; unsupported CSL style names fall back to title rendering."),
    ));
}

fn push_duplicate_bibliography_diagnostic(
    input: &DocumentValidationInput<'_>,
    key: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let entries = input
        .bibliography
        .iter()
        .filter(|entry| entry.key == key)
        .collect::<Vec<_>>();
    let primary = entries.get(1).or_else(|| entries.first());
    let (source_file, line) = primary
        .and_then(|entry| bibliography_entry_location(entry, input.source_map))
        .unwrap_or((None, None));
    let mut diagnostic = diag(
        "error",
        format!("Duplicate bibliography key: {key}"),
        source_file,
        line,
        Some("Keep bibliography keys unique so citations resolve deterministically."),
    );
    if let Some(entry) = primary {
        if let (Some(column), Some(end_column)) = (entry.column, entry.end_column) {
            let end_line = diagnostic.line;
            diagnostic = with_range(diagnostic, column, end_line, end_column);
        }
    }
    diagnostic
        .related
        .push(format!("Duplicate occurrences: {}", entries.len()));
    if let Some(first) = entries.first() {
        if let Some((source_file, line)) = bibliography_entry_location(first, input.source_map) {
            diagnostic.related.push(format!(
                "First occurrence: {}:{}",
                source_file.unwrap_or_else(|| "document".to_string()),
                line.unwrap_or(first.line.unwrap_or_default())
            ));
        }
    }
    diagnostics.push(diagnostic);
}

fn bibliography_entry_location(
    entry: &BibliographyEntry,
    source_map: &[SourceMapEntry],
) -> Option<(Option<String>, Option<usize>)> {
    if entry.source_file.is_some() {
        return Some((entry.source_file.clone(), entry.line));
    }
    entry
        .line
        .map(|line| diagnostic_location_for_generated_line(source_map, line))
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

pub(crate) fn validate_captioned_business_objects(
    blocks: &[DocumentBlock],
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    for block in blocks {
        match block {
            DocumentBlock::Figure {
                id,
                caption,
                line,
                source,
                ..
            } if missing_label_or_caption(id, caption) => {
                push_caption_diagnostic(
                    diagnostics,
                    CaptionDiagnosticInput {
                        kind: "Figure",
                        message: "Figure is missing a stable label or caption.",
                        suggestion:
                            "Add a figure label such as {#fig:name} and a caption attribute before export.",
                        id,
                        caption,
                        fallback_line: *line,
                        source: source.as_ref(),
                    },
                );
            }
            DocumentBlock::Table {
                id,
                caption,
                line,
                source,
                ..
            } if missing_label_or_caption(id, caption) => {
                push_caption_diagnostic(
                    diagnostics,
                    CaptionDiagnosticInput {
                        kind: "Table",
                        message: "Table is missing a stable label or caption.",
                        suggestion:
                            "Add a table caption such as Table: Caption {{#tbl:name}} before export.",
                        id,
                        caption,
                        fallback_line: *line,
                        source: source.as_ref(),
                    },
                );
            }
            DocumentBlock::Equation {
                id,
                caption,
                line,
                source,
                ..
            } if missing_label_or_caption(id, caption) => {
                push_caption_diagnostic(
                    diagnostics,
                    CaptionDiagnosticInput {
                        kind: "Equation",
                        message: "Equation is missing a stable label or caption.",
                        suggestion:
                            "Add an equation label such as {#eq:name} and a caption before export.",
                        id,
                        caption,
                        fallback_line: *line,
                        source: source.as_ref(),
                    },
                );
            }
            _ => {}
        }
    }
}

fn missing_label_or_caption(id: &Option<String>, caption: &Option<String>) -> bool {
    id.as_deref().unwrap_or("").trim().is_empty()
        || caption.as_deref().unwrap_or("").trim().is_empty()
}

struct CaptionDiagnosticInput<'a> {
    kind: &'a str,
    message: &'a str,
    suggestion: &'a str,
    id: &'a Option<String>,
    caption: &'a Option<String>,
    fallback_line: usize,
    source: Option<&'a crate::document_ast::AstSourceRange>,
}

fn push_caption_diagnostic(
    diagnostics: &mut Vec<DocumentDiagnostic>,
    input: CaptionDiagnosticInput<'_>,
) {
    let (source_file, line) = input
        .source
        .map(|source| (Some(source.source_file.clone()), Some(source.source_line)))
        .unwrap_or((None, Some(input.fallback_line)));
    let mut diagnostic = diag(
        "warning",
        input.message,
        source_file,
        line,
        Some(input.suggestion),
    );
    diagnostic.related.push(format!(
        "{} metadata: label={}, caption={}",
        input.kind,
        present_or_missing(input.id.as_deref().unwrap_or_default()),
        present_or_missing(input.caption.as_deref().unwrap_or_default())
    ));
    diagnostics.push(diagnostic);
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

fn validate_ai_provenance_metadata(
    input: &DocumentValidationInput<'_>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if let Some(source) = input.ai_sources.iter().find(|source| {
        source.provider.trim().is_empty()
            || source.model.trim().is_empty()
            || source.date.trim().is_empty()
            || source.prompt_summary.trim().is_empty()
    }) {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, source.line);
        let mut diagnostic = diag(
            "warning",
            "AI source block is missing provenance metadata.",
            source_file,
            line,
            Some("Record provider, model, date, and promptSummary in ai-source blocks."),
        );
        diagnostic
            .related
            .push(missing_ai_source_metadata_summary(source));
        diagnostics.push(diagnostic);
    }

    if let Some(source) = input
        .ai_sources
        .iter()
        .find(|source| !is_known_ai_review_status(&source.status))
    {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, source.line);
        diagnostics.push(diag(
            "warning",
            format!("Invalid AI source review status: {}", source.status),
            source_file,
            line,
            Some("Use unreviewed, needs-review, or human-reviewed."),
        ));
    }

    if let Some(source) = input.ai_sources.iter().find(|source| {
        source.status == "human-reviewed"
            && (source.reviewed_by.trim().is_empty() || source.reviewed_at.trim().is_empty())
    }) {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, source.line);
        let mut diagnostic = diag(
            "warning",
            "AI source is marked human-reviewed without reviewer metadata.",
            source_file,
            line,
            Some("Add reviewedBy and reviewedAt to human-reviewed ai-source blocks."),
        );
        diagnostic.related.push(format!(
            "AI source review metadata: reviewedBy={}, reviewedAt={}",
            present_or_missing(&source.reviewed_by),
            present_or_missing(&source.reviewed_at)
        ));
        diagnostics.push(diagnostic);
    }

    if let Some(section) = input.ai_assisted_sections.iter().find(|section| {
        section.source.trim().is_empty() || section.prompt_summary.trim().is_empty()
    }) {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, section.line);
        let mut diagnostic = diag(
            "warning",
            "AI-assisted section marker is missing provenance metadata.",
            source_file,
            line,
            Some("Record source and promptSummary on AI-assisted section markers."),
        );
        diagnostic.related.push(format!(
            "AI-assisted section metadata: source={}, promptSummary={}",
            present_or_missing(&section.source),
            present_or_missing(&section.prompt_summary)
        ));
        diagnostics.push(diagnostic);
    }

    if let Some(section) = input
        .ai_assisted_sections
        .iter()
        .find(|section| !is_known_ai_review_status(&section.status))
    {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, section.line);
        diagnostics.push(diag(
            "warning",
            format!(
                "Invalid AI-assisted section review status: {}",
                section.status
            ),
            source_file,
            line,
            Some("Use unreviewed, needs-review, or human-reviewed."),
        ));
    }

    if let Some(section) = input.ai_assisted_sections.iter().find(|section| {
        section.status == "human-reviewed"
            && (section.reviewed_by.trim().is_empty() || section.reviewed_at.trim().is_empty())
    }) {
        let (source_file, line) =
            diagnostic_location_for_generated_line(input.source_map, section.line);
        let mut diagnostic = diag(
            "warning",
            "AI-assisted section is marked human-reviewed without reviewer metadata.",
            source_file,
            line,
            Some("Add reviewedBy and reviewedAt to human-reviewed AI-assisted section markers."),
        );
        diagnostic.related.push(format!(
            "AI-assisted review metadata: reviewedBy={}, reviewedAt={}",
            present_or_missing(&section.reviewed_by),
            present_or_missing(&section.reviewed_at)
        ));
        diagnostics.push(diagnostic);
    }
}

fn is_known_ai_review_status(status: &str) -> bool {
    matches!(status, "unreviewed" | "needs-review" | "human-reviewed")
}

fn missing_ai_source_metadata_summary(source: &AiSource) -> String {
    format!(
        "AI source metadata: provider={}, model={}, date={}, promptSummary={}",
        present_or_missing(&source.provider),
        present_or_missing(&source.model),
        present_or_missing(&source.date),
        present_or_missing(&source.prompt_summary)
    )
}

fn present_or_missing(value: &str) -> &'static str {
    if value.trim().is_empty() {
        "missing"
    } else {
        "present"
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
