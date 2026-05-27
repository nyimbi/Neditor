use super::*;
use crate::table_cells::{
    html_table_cell, markdown_table_rows, normalize_table_cell_rows, plain_table_cells,
    table_cell_texts,
};

#[derive(Debug)]
pub(super) struct ExportMedia {
    pub(super) source: String,
    pub(super) source_file: Option<String>,
    pub(super) float: Option<String>,
    pub(super) fit: Option<String>,
    pub(super) position: Option<String>,
    pub(super) relationship_id: String,
    pub(super) path: String,
    pub(super) extension: String,
    pub(super) content_type: String,
    pub(super) bytes: Vec<u8>,
    pub(super) dimensions: Option<ExportImageDimensions>,
}

#[derive(Clone, Debug)]
pub(super) struct ExportHyperlink {
    pub(super) url: String,
    pub(super) relationship_id: String,
}

pub(super) fn export_media_emu_size(
    media: &ExportMedia,
    max_width: i64,
    max_height: i64,
    fallback: (i64, i64),
) -> (i64, i64) {
    export_dimensions_emu_size(
        media.dimensions,
        media.fit.as_deref(),
        max_width,
        max_height,
        fallback,
    )
}

pub(super) fn render_root_relationships(office_document_target: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="{}"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/><Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties" Target="docProps/custom.xml"/></Relationships>"#,
        escape_xml(office_document_target)
    )
}

pub(super) fn render_core_properties(response: &CompileResponse) -> String {
    let author = metadata_string(&response.metadata, "author")
        .or_else(|| metadata_string(&response.metadata, "approvedBy"))
        .unwrap_or_else(|| "NEditor".to_string());
    let version = metadata_string(&response.metadata, "version").unwrap_or_default();
    let classification = metadata_string(&response.metadata, "classification").unwrap_or_default();
    let keywords = [
        response.semantic.status.as_str(),
        version.as_str(),
        classification.as_str(),
    ]
    .into_iter()
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join("; ");
    let description = format!(
        "Status: {}; Version: {}",
        response.semantic.status,
        if version.is_empty() {
            "unversioned"
        } else {
            version.as_str()
        }
    );
    let timestamp = Utc::now().to_rfc3339();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>{}</dc:title><dc:creator>{}</dc:creator><dc:description>{}</dc:description><cp:keywords>{}</cp:keywords><cp:category>{}</cp:category><cp:lastModifiedBy>NEditor</cp:lastModifiedBy><dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified></cp:coreProperties>"#,
        escape_xml(&response.semantic.title),
        escape_xml(&author),
        escape_xml(&description),
        escape_xml(&keywords),
        escape_xml(&response.semantic.status),
        escape_xml(&timestamp),
        escape_xml(&timestamp)
    )
}

pub(super) fn render_custom_properties(response: &CompileResponse) -> String {
    let mut properties = Vec::new();
    push_custom_property(&mut properties, "NEditorStatus", &response.semantic.status);
    push_custom_property(
        &mut properties,
        "NEditorVersion",
        &metadata_string(&response.metadata, "version").unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorClassification",
        &metadata_string(&response.metadata, "classification").unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorClient",
        &metadata_string(&response.metadata, "client").unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorApprovedBy",
        &metadata_string(&response.metadata, "approvedBy").unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorApprovedAt",
        &metadata_string(&response.metadata, "approvedAt").unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorLegalDisclaimer",
        &metadata_string(&response.metadata, "legalDisclaimer").unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorTargetPersona",
        &target_persona_summary(&response.metadata).unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorDeliveryModel",
        &metadata_string(&response.metadata, "positioning.model").unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorSourceOfTruth",
        &metadata_string(&response.metadata, "positioning.sourceOfTruth").unwrap_or_default(),
    );
    push_custom_property(
        &mut properties,
        "NEditorSourceHash",
        &response.export_manifest.source_hash,
    );
    push_custom_property(
        &mut properties,
        "NEditorAppVersion",
        env!("CARGO_PKG_VERSION"),
    );
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">{}</Properties>"#,
        properties.join("")
    )
}

pub(super) fn push_custom_property(properties: &mut Vec<String>, name: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    let pid = properties.len() + 2;
    properties.push(format!(
        r#"<property fmtid="{{D5CDD505-2E9C-101B-9397-08002B2CF9AE}}" pid="{pid}" name="{}"><vt:lpwstr>{}</vt:lpwstr></property>"#,
        escape_xml(name),
        escape_xml(value)
    ));
}

pub(crate) fn export_text(response: &CompileResponse, options: &Value) -> String {
    let mut lines = export_metadata_lines(response, options);
    lines.push(String::new());
    lines.push(export_body_text_from_ast(&response.document_ast));
    lines.extend(appendix_export_lines(response, options));
    lines.join("\n")
}

pub(super) fn export_metadata_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
    let (header, footer) = export_header_footer(response, options);
    let watermark = options
        .get("watermark")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut lines = Vec::new();
    if include_cover_page(options) {
        lines.push(format!("Cover: {}", response.semantic.title));
    }
    lines.push(format!("Status: {}", response.semantic.status));
    lines.push(format!("Header: {header}"));
    if !footer.is_empty() {
        lines.push(format!("Footer: {footer}"));
    }
    if include_page_numbers(options) {
        lines.push("Page 1 of 1".to_string());
    }
    lines.push(format!("Layout preset: {}", layout_preset(options)));
    lines.push(format!(
        "Syntax highlighting: {}",
        if include_syntax_highlighting(options) {
            "included"
        } else {
            "omitted"
        }
    ));
    for path in ["subtitle", "author", "date", "version", "brand.name"] {
        if let Some(value) = metadata_string(&response.metadata, path) {
            lines.push(value);
        }
    }
    if let Some(personas) = target_persona_summary(&response.metadata) {
        lines.push(format!("Audience: {personas}"));
    }
    if let Some(model) = metadata_string(&response.metadata, "positioning.model") {
        lines.push(format!("Delivery model: {model}"));
    }
    if let Some(source) = metadata_string(&response.metadata, "positioning.sourceOfTruth") {
        lines.push(format!("Source of truth: {source}"));
    }
    if let Some(logo) = export_logo(&response.metadata) {
        lines.push(format!("Logo: {logo}"));
    }
    if !watermark.is_empty() {
        lines.push(format!("Watermark: {watermark}"));
    }
    lines
}

pub(super) fn export_logo(metadata: &Value) -> Option<String> {
    metadata_string(metadata, "brand.logo")
        .or_else(|| metadata_string(metadata, "layout.logo"))
        .or_else(|| metadata_string(metadata, "logo"))
        .filter(|value| !value.trim().is_empty())
}

pub(super) fn target_persona_summary(metadata: &Value) -> Option<String> {
    let personas = metadata_string_list(metadata, "targetPersona");
    if personas.is_empty() {
        None
    } else {
        Some(personas.join(", "))
    }
}

#[derive(Clone, Debug)]
pub(super) struct ExportTable {
    pub(super) headers: Vec<String>,
    pub(super) alignments: Vec<String>,
    pub(super) header_cells: Vec<TableCell>,
    pub(super) rows: Vec<Vec<String>>,
    pub(super) row_cells: Vec<Vec<TableCell>>,
}

pub(super) fn export_table_from_delimited_code(
    language: Option<&str>,
    code: &str,
) -> Option<ExportTable> {
    let delimiter = match language.unwrap_or("").trim().to_ascii_lowercase().as_str() {
        "csv" => ',',
        "tsv" => '\t',
        _ => return None,
    };
    let mut rows = delimited_rows_for_export(code, delimiter);
    if rows.is_empty() {
        return None;
    }
    let headers = rows.remove(0);
    if headers.is_empty() {
        return None;
    }
    let alignments = headers.iter().map(|_| "left".to_string()).collect();
    let rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|row| {
            (0..headers.len())
                .map(|index| row.get(index).cloned().unwrap_or_default())
                .collect::<Vec<_>>()
        })
        .collect();
    let header_cells = plain_export_table_cells(&headers);
    let row_cells = rows
        .iter()
        .map(|row| plain_export_table_cells(row))
        .collect();
    Some(ExportTable {
        headers,
        alignments,
        header_cells,
        rows,
        row_cells,
    })
}

pub(super) fn export_table_from_transform_html(html: &str) -> Option<ExportTable> {
    if !html.contains("<table") || !html.contains("transform-table") {
        return None;
    }
    let header_section = html_between(html, "<thead", "</thead>")?;
    let header_cells =
        normalize_table_cell_rows(&[html_table_semantic_cells(header_section, "th")])
            .into_iter()
            .next()
            .unwrap_or_default();
    if header_cells.is_empty() {
        return None;
    }
    let headers = table_cell_texts(&header_cells);
    let body_section = html_between(html, "<tbody", "</tbody>").unwrap_or("");
    let mut raw_row_cells = Vec::new();
    let mut rest = body_section;
    while let Some((row_html, next)) = next_html_tag_block(rest, "tr") {
        let row = html_table_semantic_cells(row_html, "td");
        if !row.is_empty() {
            raw_row_cells.push(row);
        }
        rest = next;
    }
    let row_cells = normalize_table_cell_rows(&raw_row_cells);
    let rows: Vec<Vec<String>> = row_cells
        .iter()
        .map(|row| {
            (0..headers.len())
                .map(|index| {
                    row.get(index)
                        .map(|cell| cell.text.clone())
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect();
    let alignments = headers.iter().map(|_| "left".to_string()).collect();
    Some(ExportTable {
        headers,
        alignments,
        header_cells,
        rows,
        row_cells,
    })
}

pub(super) fn plain_export_table_cells(cells: &[String]) -> Vec<TableCell> {
    plain_table_cells(cells)
}

pub(super) fn populated_table_cells(cells: &[TableCell], fallback: &[String]) -> Vec<TableCell> {
    if cells.is_empty() {
        plain_export_table_cells(fallback)
    } else {
        cells.to_vec()
    }
}

pub(super) fn populated_table_row_cells(
    cells: &[Vec<TableCell>],
    fallback: &[Vec<String>],
) -> Vec<Vec<TableCell>> {
    if cells.is_empty() {
        fallback
            .iter()
            .map(|row| plain_export_table_cells(row))
            .collect()
    } else {
        cells.to_vec()
    }
}

pub(super) fn html_between<'a>(
    html: &'a str,
    open_prefix: &str,
    close_tag: &str,
) -> Option<&'a str> {
    let open_start = html.find(open_prefix)?;
    let open_end = html[open_start..].find('>')? + open_start + 1;
    let close_start = html[open_end..].find(close_tag)? + open_end;
    Some(&html[open_end..close_start])
}

pub(super) fn next_html_tag_block<'a>(html: &'a str, tag: &str) -> Option<(&'a str, &'a str)> {
    let (_, body, rest) = next_html_tag_block_with_open(html, tag)?;
    Some((body, rest))
}

fn next_html_tag_block_with_open<'a>(
    html: &'a str,
    tag: &str,
) -> Option<(&'a str, &'a str, &'a str)> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let open_start = html.find(&open)?;
    let open_end = html[open_start..].find('>')? + open_start + 1;
    let close_start = html[open_end..].find(&close)? + open_end;
    let close_end = close_start + close.len();
    Some((
        &html[open_start..open_end],
        &html[open_end..close_start],
        &html[close_end..],
    ))
}

fn html_table_semantic_cells(row_html: &str, tag: &str) -> Vec<TableCell> {
    let mut cells = Vec::new();
    let mut rest = row_html;
    while let Some((open_tag, cell_html, next)) = next_html_tag_block_with_open(rest, tag) {
        let text = decode_export_html_entities(&strip_export_html_tags(cell_html))
            .trim()
            .to_string();
        cells.push(html_table_cell(text, open_tag, "colspan", "rowspan"));
        rest = next;
    }
    cells
}

pub(super) fn export_header_footer(
    response: &CompileResponse,
    options: &Value,
) -> (String, String) {
    export_header_footer_for_page(response, options, 1, 1)
}

pub(super) fn export_header_footer_for_page(
    response: &CompileResponse,
    options: &Value,
    page: usize,
    pages: usize,
) -> (String, String) {
    let classification = metadata_string(&response.metadata, "classification").unwrap_or_default();
    let header = metadata_string(&response.metadata, "layout.header")
        .map(|template| {
            render_export_template_for_page(&template, response, &classification, page, pages)
        })
        .unwrap_or_else(|| response.semantic.title.clone());
    let footer = metadata_string(&response.metadata, "layout.footer")
        .map(|template| {
            render_export_template_for_page(&template, response, &classification, page, pages)
        })
        .unwrap_or_else(|| {
            if include_page_numbers(options) {
                format!("Page {page} of {pages}")
            } else {
                String::new()
            }
        });
    (header, footer)
}

pub(super) fn render_section_template(
    response: &CompileResponse,
    template: &str,
    page: usize,
    pages: usize,
) -> String {
    let classification = metadata_string(&response.metadata, "classification").unwrap_or_default();
    render_export_template_for_page(template, response, &classification, page, pages)
}

pub(super) fn render_export_template_for_page(
    template: &str,
    response: &CompileResponse,
    classification: &str,
    page: usize,
    pages: usize,
) -> String {
    template
        .replace("{{title}}", &response.semantic.title)
        .replace("{{status}}", &response.semantic.status)
        .replace("{{classification}}", classification)
        .replace("{{page}}", &page.to_string())
        .replace("{{pages}}", &pages.to_string())
}

pub(super) fn boolean_option(options: &Value, name: &str, aliases: &[&str], default: bool) -> bool {
    std::iter::once(name)
        .chain(aliases.iter().copied())
        .find_map(|key| options.get(key).and_then(Value::as_bool))
        .unwrap_or(default)
}

pub(super) fn include_styles(options: &Value) -> bool {
    boolean_option(options, "includeStyles", &[], true)
}

pub(super) fn include_syntax_highlighting(options: &Value) -> bool {
    boolean_option(options, "includeSyntaxHighlighting", &[], true)
}

pub(super) fn include_cover_page(options: &Value) -> bool {
    boolean_option(options, "coverPage", &["includeCoverPage"], true)
}

pub(super) fn include_page_numbers(options: &Value) -> bool {
    boolean_option(options, "pageNumbers", &["includePageNumbers"], true)
}

pub(super) fn layout_preset(options: &Value) -> &str {
    match options.get("layoutPreset").and_then(Value::as_str) {
        Some("compact") => "compact",
        Some("presentation") => "presentation",
        _ => "business",
    }
}

pub(super) fn layout_page_size(metadata: &Value) -> String {
    metadata_string(metadata, "layout.pageSize")
        .or_else(|| metadata_string(metadata, "pageSize"))
        .map(|value| value.to_ascii_lowercase().replace([' ', '-'], ""))
        .and_then(|value| match value.as_str() {
            "letter" | "usletter" => Some("letter".to_string()),
            "legal" | "uslegal" => Some("legal".to_string()),
            "a4" => Some("a4".to_string()),
            _ => None,
        })
        .unwrap_or_else(|| "a4".to_string())
}

pub(super) fn layout_orientation(metadata: &Value) -> &'static str {
    metadata_string(metadata, "layout.orientation")
        .or_else(|| metadata_string(metadata, "orientation"))
        .map(|value| value.to_ascii_lowercase().replace([' ', '-'], ""))
        .and_then(|value| match value.as_str() {
            "landscape" => Some("landscape"),
            "portrait" => Some("portrait"),
            _ => None,
        })
        .unwrap_or("portrait")
}

pub(super) fn explicit_layout_margins(metadata: &Value) -> Option<String> {
    metadata_string(metadata, "layout.margins")
        .or_else(|| metadata_string(metadata, "margins"))
        .map(|value| value.to_ascii_lowercase().replace([' ', '-'], ""))
        .filter(|value| matches!(value.as_str(), "narrow" | "compact" | "normal" | "wide"))
}

pub(super) fn include_glossary(options: &Value) -> bool {
    options
        .get("includeGlossary")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(super) fn include_comments(options: &Value) -> bool {
    options
        .get("includeComments")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(super) fn include_provenance(options: &Value) -> bool {
    options
        .get("includeProvenance")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(super) fn appendix_export_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
    let mut lines = glossary_export_lines(response, options);
    lines.extend(comment_export_lines(response, options));
    lines.extend(provenance_export_lines(response, options));
    lines.extend(legal_disclaimer_export_lines(response));
    lines
}

pub(super) fn glossary_export_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
    if !include_glossary(options) || response.semantic.glossary.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![String::new(), "Glossary".to_string()];
    lines.extend(
        response
            .semantic
            .glossary
            .iter()
            .map(|(term, definition)| format!("{term}: {definition}")),
    );
    lines
}

pub(super) fn comment_export_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
    if !include_comments(options)
        || (response.semantic.comments.is_empty() && response.semantic.change_notes.is_empty())
    {
        return Vec::new();
    }
    let mut lines = vec![String::new(), "Review Comments".to_string()];
    lines.extend(response.semantic.comments.iter().map(|comment| {
        let created_at = if comment.created_at.is_empty() {
            "undated"
        } else {
            comment.created_at.as_str()
        };
        let author = if comment.author.is_empty() {
            "local"
        } else {
            comment.author.as_str()
        };
        format!(
            "Line {} [{}] {} at {}: {}",
            comment.line, comment.state, author, created_at, comment.text
        )
    }));
    if !response.semantic.change_notes.is_empty() {
        lines.push(String::new());
        lines.push("Change Notes".to_string());
        lines.extend(response.semantic.change_notes.iter().map(|note| {
            let created_at = if note.created_at.is_empty() {
                "undated"
            } else {
                note.created_at.as_str()
            };
            let author = if note.author.is_empty() {
                "local"
            } else {
                note.author.as_str()
            };
            format!(
                "Line {} {} at {}: {}",
                note.line, author, created_at, note.text
            )
        }));
    }
    lines
}

pub(super) fn provenance_export_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
    if !include_provenance(options)
        || (response.semantic.ai_sources.is_empty()
            && response.semantic.ai_assisted_sections.is_empty())
    {
        return Vec::new();
    }
    let mut lines = vec![String::new(), "AI Provenance".to_string()];
    lines.extend(response.semantic.ai_sources.iter().map(|source| {
        let provider = empty_as(source.provider.as_str(), "unknown provider");
        let model = empty_as(source.model.as_str(), "unknown model");
        let date = empty_as(source.date.as_str(), "undated");
        let reviewer = empty_as(source.reviewed_by.as_str(), "unreviewed");
        let reviewed_at = empty_as(source.reviewed_at.as_str(), "undated");
        let summary = empty_as(source.prompt_summary.as_str(), "no prompt summary");
        format!(
            "{provider} / {model} on {date}; status: {}; reviewed by: {reviewer} on {reviewed_at}; prompt: {summary}",
            source.status
        )
    }));
    lines.extend(response.semantic.ai_assisted_sections.iter().map(|section| {
        let reviewer = empty_as(section.reviewed_by.as_str(), "unreviewed");
        let reviewed_at = empty_as(section.reviewed_at.as_str(), "undated");
        let source = empty_as(section.source.as_str(), "unspecified source");
        let summary = empty_as(section.prompt_summary.as_str(), "no prompt summary");
        format!(
            "Section '{}' at line {}: status {}; reviewed by {reviewer} on {reviewed_at}; source: {source}; prompt: {summary}",
            section.heading, section.line, section.status
        )
    }));
    lines
}

pub(super) fn legal_disclaimer_export_lines(response: &CompileResponse) -> Vec<String> {
    let Some(disclaimer) = metadata_string(&response.metadata, "legalDisclaimer")
        .filter(|value| !value.trim().is_empty())
    else {
        return Vec::new();
    };
    vec![String::new(), "Legal Disclaimer".to_string(), disclaimer]
}

pub(super) fn empty_as<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

pub(super) fn is_generated_toc_heading(block: &DocumentBlock) -> bool {
    matches!(
        block,
        DocumentBlock::Heading { level, text, .. }
            if *level == 2 && text == "Table of Contents"
    )
}

pub(super) fn is_generated_toc_body(block: &DocumentBlock) -> bool {
    matches!(
        block,
        DocumentBlock::Paragraph { text, .. }
            if text.trim_start().starts_with("- [") && text.contains("](#")
    ) || matches!(
        block,
        DocumentBlock::List { items, .. }
            if items.iter().any(|item| item.contains("](#"))
    )
}

pub(super) fn collect_docx_media(response: &CompileResponse) -> Vec<ExportMedia> {
    let mut media = Vec::new();
    for block in &response.document_ast.blocks {
        let DocumentBlock::Figure {
            src: Some(src),
            source,
            float,
            fit,
            position,
            ..
        } = block
        else {
            continue;
        };
        let source_file = source.as_ref().map(|range| range.source_file.clone());
        if media
            .iter()
            .any(|item: &ExportMedia| item.source == *src && item.source_file == source_file)
        {
            continue;
        }
        let Some(parsed) = parse_export_image(src, source.as_ref()) else {
            continue;
        };
        let index = media.len() + 1;
        let path = format!("media/image{index}.{}", parsed.extension);
        media.push(ExportMedia {
            source: src.clone(),
            source_file,
            float: normalized_float(float.as_deref()).map(str::to_string),
            fit: normalized_fit(fit.as_deref()).map(str::to_string),
            position: normalized_position(position.as_deref()).map(str::to_string),
            relationship_id: format!("rIdImage{index}"),
            path,
            extension: parsed.extension,
            content_type: parsed.content_type,
            bytes: parsed.bytes,
            dimensions: parsed.dimensions,
        });
    }
    media
}

pub(super) fn is_external_hyperlink(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
}

pub(super) fn appendix_pages(response: &CompileResponse, options: &Value) -> Vec<Vec<String>> {
    [
        glossary_export_lines(response, options),
        comment_export_lines(response, options),
        provenance_export_lines(response, options),
        legal_disclaimer_export_lines(response),
    ]
    .into_iter()
    .filter(|lines| !lines.is_empty())
    .collect()
}

pub(super) fn block_export_lines(block: &DocumentBlock) -> Vec<String> {
    match block {
        DocumentBlock::Heading { level, text, .. } => {
            vec![format!("{} {text}", "#".repeat(*level))]
        }
        DocumentBlock::Paragraph { text, inlines, .. } => {
            vec![paragraph_export_line(text, inlines)]
        }
        DocumentBlock::BlockQuote { text, .. } => {
            text.lines().map(|line| format!("> {line}")).collect()
        }
        DocumentBlock::CodeBlock { language, code, .. } => {
            if let Some(table) = export_table_from_delimited_code(language.as_deref(), code) {
                return table_export_lines(
                    &None,
                    &None,
                    &table.headers,
                    &table.alignments,
                    &table.header_cells,
                    &table.rows,
                    &table.row_cells,
                );
            }
            let mut lines = vec![format!("```{}", language.as_deref().unwrap_or(""))];
            lines.extend(code.lines().map(ToString::to_string));
            lines.push("```".to_string());
            lines
        }
        DocumentBlock::List { ordered, items, .. } => items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if *ordered {
                    format!("{}. {item}", index + 1)
                } else {
                    format!("- {item}")
                }
            })
            .collect(),
        DocumentBlock::TaskList { items, .. } => items
            .iter()
            .map(|item| format!("- [{}] {}", if item.checked { "x" } else { " " }, item.text))
            .collect(),
        DocumentBlock::Table {
            id,
            caption,
            headers,
            alignments,
            header_cells,
            rows,
            row_cells,
            ..
        } => table_export_lines(
            id,
            caption,
            headers,
            alignments,
            header_cells,
            rows,
            row_cells,
        ),
        DocumentBlock::Figure {
            id,
            src,
            alt,
            caption,
            float,
            fit,
            position,
            ..
        } => vec![figure_export_line(
            id, src, alt, caption, float, fit, position,
        )],
        DocumentBlock::Equation {
            id, caption, text, ..
        } => vec![equation_export_line(id, text, caption)],
        DocumentBlock::Layout {
            directive,
            options,
            settings,
            ..
        } => layout_export_lines(directive, options, settings),
        DocumentBlock::Callout {
            callout_type,
            title,
            text,
            ..
        } => vec![callout_export_line(callout_type, title, text)],
        DocumentBlock::Footnotes { entries, .. } => {
            let mut lines = vec!["Footnotes".to_string()];
            lines.extend(
                entries
                    .iter()
                    .map(|entry| format!("{}. {}", entry.number, entry.text)),
            );
            lines
        }
        DocumentBlock::ReviewComment { comment, .. } => vec![format!(
            "Review comment: {} | {} | {}",
            comment.state, comment.author, comment.text
        )],
        DocumentBlock::ChangeNote { note, .. } => {
            vec![format!("Change note: {} | {}", note.author, note.text)]
        }
        DocumentBlock::AiSource { provenance, .. } => vec![format!(
            "AI source: {} / {} | {}",
            empty_as(&provenance.provider, "unknown"),
            empty_as(&provenance.model, "unknown"),
            empty_as(&provenance.status, "unreviewed")
        )],
        DocumentBlock::Transform { name, text, .. } => vec![transform_export_line(name, text)],
        DocumentBlock::RawHtml { html, .. } => {
            if let Some(table) = export_table_from_transform_html(html) {
                return table_export_lines(
                    &None,
                    &None,
                    &table.headers,
                    &table.alignments,
                    &table.header_cells,
                    &table.rows,
                    &table.row_cells,
                );
            }
            raw_html_export_lines(html)
        }
    }
}

pub(super) fn paragraph_export_line(text: &str, inlines: &[InlineNode]) -> String {
    let mut rendered = text.to_string();
    for inline in inlines {
        let InlineNode::Link { text: label, url } = inline else {
            continue;
        };
        if !is_external_hyperlink(url) {
            continue;
        }
        let replacement = format!("{label} ({url})");
        let markdown_link = format!("[{label}]({url})");
        if let Some(start) = rendered.find(&markdown_link) {
            rendered.replace_range(start..start + markdown_link.len(), &replacement);
        } else if let Some(start) = rendered.find(label) {
            rendered.replace_range(start..start + label.len(), &replacement);
        } else if !rendered.contains(url) {
            rendered.push_str(&format!(" ({url})"));
        }
    }
    rendered
}

pub(super) fn raw_html_export_lines(html: &str) -> Vec<String> {
    if html.contains("role=\"doc-endnotes\"") || html.contains("class=\"footnotes\"") {
        return vec!["Footnotes".to_string()];
    }
    let text = decode_export_html_entities(&strip_export_html_tags(html))
        .trim()
        .trim_end_matches(" back")
        .trim()
        .to_string();
    if text.is_empty() {
        Vec::new()
    } else {
        vec![text]
    }
}

pub(super) fn strip_export_html_tags(html: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn decode_export_html_entities(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

pub(super) fn layout_export_lines(
    directive: &str,
    options: &str,
    settings: &LayoutSettings,
) -> Vec<String> {
    let summary = layout_summary(options, settings);
    let label = match directive {
        "section-break" => "Section break",
        "slide" => "Slide",
        "layout" => "Layout",
        "page-break" => "Page break",
        _ => "Layout directive",
    };
    vec![format!("{label}{summary}")]
}

pub(super) fn layout_summary(options: &str, settings: &LayoutSettings) -> String {
    let mut parts = Vec::new();
    if let Some(columns) = settings.columns {
        parts.push(format!("columns={columns}"));
    }
    if let Some(column_gap) = &settings.column_gap {
        parts.push(format!("columnGap={column_gap}"));
    }
    if let Some(page_size) = &settings.page_size {
        parts.push(format!("pageSize={page_size}"));
    }
    if let Some(orientation) = &settings.orientation {
        parts.push(format!("orientation={orientation}"));
    }
    if let Some(margins) = &settings.margins {
        parts.push(format!("margins={margins}"));
    }
    if let Some(break_before) = &settings.break_before {
        parts.push(format!("breakBefore={break_before}"));
    }
    if let Some(break_after) = &settings.break_after {
        parts.push(format!("breakAfter={break_after}"));
    }
    if settings.keep_with_next {
        parts.push("keepWithNext=true".to_string());
    }
    if settings.keep_together {
        parts.push("keepTogether=true".to_string());
    }
    if parts.is_empty() {
        let trimmed = options.trim();
        if trimmed.is_empty() {
            String::new()
        } else {
            format!(": {trimmed}")
        }
    } else {
        format!(": {}", parts.join(", "))
    }
}

pub(super) fn slide_title_from_options(options: &str, settings: &LayoutSettings) -> String {
    settings.title.clone().unwrap_or_else(|| {
        let trimmed = options.trim();
        if trimmed.is_empty() {
            "Slide".to_string()
        } else {
            trimmed.to_string()
        }
    })
}

pub(super) fn slide_notes_from_options(settings: &LayoutSettings) -> Vec<String> {
    settings
        .notes
        .as_ref()
        .map(|value| {
            value
                .replace("\\n", "\n")
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn callout_export_line(callout_type: &str, title: &str, text: &str) -> String {
    let mut parts = vec![format!("Callout: {callout_type}")];
    if !title.is_empty() {
        parts.push(title.to_string());
    }
    if !text.is_empty() {
        parts.push(text.to_string());
    }
    parts.join(": ")
}

pub(super) fn transform_export_line(name: &str, text: &str) -> String {
    let label = format!("Transform: {name}");
    if text.is_empty() {
        label
    } else {
        format!("{label}: {text}")
    }
}

pub(super) fn figure_export_line(
    id: &Option<String>,
    src: &Option<String>,
    alt: &Option<String>,
    caption: &Option<String>,
    float: &Option<String>,
    fit: &Option<String>,
    position: &Option<String>,
) -> String {
    let mut parts = vec!["Figure".to_string()];
    if let Some(id) = id {
        parts.push(id.clone());
    }
    if let Some(caption) = caption {
        parts.push(caption.clone());
    }
    if let Some(alt) = alt {
        parts.push(alt.clone());
    }
    if let Some(float) = float {
        parts.push(format!("float={float}"));
    }
    if let Some(fit) = fit {
        parts.push(format!("fit={fit}"));
    }
    if let Some(position) = position {
        parts.push(format!("position={position}"));
    }
    if let Some(src) = src {
        parts.push(format!("({src})"));
    }
    parts.join(": ")
}

pub(super) fn normalized_float(float: Option<&str>) -> Option<&'static str> {
    match float?.trim().to_ascii_lowercase().as_str() {
        "left" => Some("left"),
        "right" => Some("right"),
        "center" | "centre" => Some("center"),
        _ => None,
    }
}

pub(super) fn table_export_line(
    id: &Option<String>,
    caption: &Option<String>,
    headers: &[String],
) -> String {
    let mut parts = vec!["Table".to_string()];
    if let Some(id) = id {
        parts.push(id.clone());
    }
    if let Some(caption) = caption {
        parts.push(caption.clone());
    }
    if parts.len() == 1 {
        parts.push(headers.join(" | "));
    }
    parts.join(": ")
}

pub(super) fn table_export_lines(
    id: &Option<String>,
    caption: &Option<String>,
    headers: &[String],
    alignments: &[String],
    header_cells: &[TableCell],
    rows: &[Vec<String>],
    row_cells: &[Vec<TableCell>],
) -> Vec<String> {
    let mut lines = vec![table_export_line(id, caption, headers)];
    lines.extend(markdown_table_rows(
        headers,
        alignments,
        header_cells,
        rows,
        row_cells,
    ));
    lines
}

pub(super) fn equation_export_line(
    id: &Option<String>,
    text: &str,
    caption: &Option<String>,
) -> String {
    let mut parts = vec!["Equation".to_string()];
    if let Some(id) = id {
        parts.push(id.clone());
    }
    if !text.is_empty() {
        parts.push(text.to_string());
    }
    if let Some(caption) = caption {
        parts.push(caption.clone());
    }
    parts.join(": ")
}
