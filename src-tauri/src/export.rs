use crate::{
    document_ast::{export_body_text_from_ast, DocumentBlock, FootnoteEntry, InlineNode},
    escape_css, escape_html, escape_pdf, escape_xml,
    export_media::{
        drawingml_source_crop, export_dimensions_emu_size, normalized_fit, normalized_position,
        parse_export_image, safe_bundle_path, ExportImageDimensions,
    },
    generated_sections::toc_depth,
    layout::{matches_layout_break, LayoutSettings},
    metadata_string, render_export_template, sha256_uri,
    tables::delimited_rows_for_export,
    CompileResponse, ExportManifest,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    fs,
    io::{Cursor, Write},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[derive(Debug)]
struct ExportMedia {
    source: String,
    source_file: Option<String>,
    float: Option<String>,
    fit: Option<String>,
    position: Option<String>,
    relationship_id: String,
    path: String,
    extension: String,
    content_type: String,
    bytes: Vec<u8>,
    dimensions: Option<ExportImageDimensions>,
}

#[derive(Clone, Debug)]
struct ExportHyperlink {
    url: String,
    relationship_id: String,
}

struct BundleInclude {
    source_path: String,
    bundle_path: String,
    hash: String,
    bytes: Vec<u8>,
}

pub(crate) fn render_full_html(response: &CompileResponse, options: &Value) -> String {
    let brand_color = options
        .get("brandColor")
        .and_then(Value::as_str)
        .or_else(|| {
            response
                .metadata
                .pointer("/brand/color")
                .and_then(Value::as_str)
        })
        .unwrap_or("#275DA8");
    let watermark = options
        .get("watermark")
        .and_then(Value::as_str)
        .unwrap_or("");
    let brand_font = metadata_string(&response.metadata, "brand.font")
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Inter,Arial,sans-serif".to_string());
    let subtitle = metadata_string(&response.metadata, "subtitle");
    let author = metadata_string(&response.metadata, "author");
    let date = metadata_string(&response.metadata, "date");
    let version = metadata_string(&response.metadata, "version");
    let classification = metadata_string(&response.metadata, "classification");
    let brand = metadata_string(&response.metadata, "brand.name");
    let logo = export_logo(&response.metadata);
    let header_template = metadata_string(&response.metadata, "layout.header")
        .or_else(|| Some(response.semantic.title.clone()));
    let footer_template = metadata_string(&response.metadata, "layout.footer").or_else(|| {
        include_page_numbers(options).then(|| "Page {{page}} of {{pages}}".to_string())
    });
    let running_header = render_export_template(
        header_template.as_deref().unwrap_or(""),
        response,
        classification.as_deref().unwrap_or(""),
    );
    let running_footer = render_export_template(
        footer_template.as_deref().unwrap_or(""),
        response,
        classification.as_deref().unwrap_or(""),
    );
    let cover_meta = [author, date, version, classification, brand]
        .into_iter()
        .flatten()
        .map(|value| format!("<p>{}</p>", escape_html(&value)))
        .collect::<String>();
    let appendix_sections = html_appendix_sections(response, options);
    let style_tag = if include_styles(options) {
        format!(
            "<style>{}</style>",
            export_css(
                brand_color,
                watermark,
                &brand_font,
                include_page_numbers(options),
                layout_preset(options),
                include_syntax_highlighting(options),
                &response.metadata,
            )
        )
    } else {
        String::new()
    };
    let cover_section = if include_cover_page(options) {
        format!(
            "<section class=\"cover\">{}<h1>{}</h1>{}<p class=\"status\">{}</p>{}</section>",
            logo.as_ref()
                .map(|src| format!(
                    "<img class=\"cover-logo\" src=\"{}\" alt=\"{} logo\"/>",
                    escape_html(src),
                    escape_html(&response.semantic.title)
                ))
                .unwrap_or_default(),
            escape_html(&response.semantic.title),
            subtitle
                .map(|value| format!("<p class=\"subtitle\">{}</p>", escape_html(&value)))
                .unwrap_or_default(),
            escape_html(&response.semantic.status),
            cover_meta
        )
    } else {
        String::new()
    };
    let body_html = if include_syntax_highlighting(options) {
        highlight_code_blocks(&response.html)
    } else {
        response.html.clone()
    };
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{}</title>{}</head><body><div class=\"running-header\">{}</div>{}<main>{}{}</main><footer><strong>{}</strong><span>{}</span><small>{}</small></footer></body></html>",
        escape_html(&response.semantic.title),
        style_tag,
        escape_html(&running_header),
        cover_section,
        body_html,
        appendix_sections,
        escape_html(&running_footer),
        escape_html("Generated by NEditor"),
        escape_html(&Utc::now().to_rfc3339())
    )
}

mod pdf;
pub(crate) use pdf::render_pdf_bytes;

mod docx;
pub(crate) use docx::render_docx_bytes;

mod pptx;
pub(crate) use pptx::render_pptx_bytes;

pub(crate) fn render_markdown_bundle_bytes(
    response: &CompileResponse,
    manifest: &ExportManifest,
) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("document.md", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(response.compiled_markdown.as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("document.txt", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(export_text(response, &manifest.export_options).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("manifest.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(manifest)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.start_file("semantic.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&response.semantic)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.start_file("metadata.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&response.metadata)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.start_file("document-ast.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&response.document_ast)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.start_file("source-map.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&response.source_map)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.start_file("diagnostics.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&response.diagnostics)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.start_file("bibliography.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&response.bibliography)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.start_file("formula-graph.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&json!({
            "formulas": &response.formula_graph,
            "dependencies": &response.formula_dependency_edges,
        }))
        .map_err(|err| err.to_string())?
        .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.start_file("transform-artifacts.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&response.transform_artifacts)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    let media = collect_docx_media(response);
    if !media.is_empty() {
        zip.start_file("media-map.json", options)
            .map_err(|err| err.to_string())?;
        zip.write_all(render_bundle_media_map(&media)?.as_bytes())
            .map_err(|err| err.to_string())?;
    }
    let bundled_includes = collect_bundle_includes(manifest);
    if !bundled_includes.is_empty() {
        zip.start_file("include-map.json", options)
            .map_err(|err| err.to_string())?;
        zip.write_all(render_bundle_include_map(&bundled_includes)?.as_bytes())
            .map_err(|err| err.to_string())?;
    }
    for included in bundled_includes {
        zip.start_file(included.bundle_path, options)
            .map_err(|err| err.to_string())?;
        zip.write_all(&included.bytes)
            .map_err(|err| err.to_string())?;
    }
    for item in media {
        zip.start_file(item.path, options)
            .map_err(|err| err.to_string())?;
        zip.write_all(&item.bytes).map_err(|err| err.to_string())?;
    }
    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}

fn collect_bundle_includes(manifest: &ExportManifest) -> Vec<BundleInclude> {
    manifest
        .included_files
        .iter()
        .filter_map(|included| {
            let bytes = fs::read(&included.path).ok()?;
            Some(BundleInclude {
                source_path: included.path.clone(),
                bundle_path: format!("includes/{}", safe_bundle_path(&included.path)),
                hash: included.hash.clone(),
                bytes,
            })
        })
        .collect()
}

fn render_bundle_include_map(includes: &[BundleInclude]) -> Result<String, String> {
    let entries = includes
        .iter()
        .map(|item| {
            json!({
                "source_path": item.source_path,
                "bundle_path": item.bundle_path,
                "hash": item.hash,
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&entries).map_err(|err| err.to_string())
}

fn render_bundle_media_map(media: &[ExportMedia]) -> Result<String, String> {
    let entries = media
        .iter()
        .map(|item| {
            let mut entry = json!({
                "source": item.source,
                "source_file": item.source_file,
                "float": item.float,
                "fit": item.fit,
                "position": item.position,
                "bundle_path": item.path,
                "content_type": item.content_type,
                "hash": sha256_uri(&item.bytes),
            });
            if let Some(dimensions) = item.dimensions {
                entry["width_px"] = json!(dimensions.width_px);
                entry["height_px"] = json!(dimensions.height_px);
            }
            entry
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&entries).map_err(|err| err.to_string())
}

fn export_media_emu_size(
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

fn render_root_relationships(office_document_target: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="{}"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/><Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties" Target="docProps/custom.xml"/></Relationships>"#,
        escape_xml(office_document_target)
    )
}

fn render_core_properties(response: &CompileResponse) -> String {
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

fn render_custom_properties(response: &CompileResponse) -> String {
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

fn push_custom_property(properties: &mut Vec<String>, name: &str, value: &str) {
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

fn export_metadata_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
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
    if let Some(logo) = export_logo(&response.metadata) {
        lines.push(format!("Logo: {logo}"));
    }
    if !watermark.is_empty() {
        lines.push(format!("Watermark: {watermark}"));
    }
    lines
}

fn export_logo(metadata: &Value) -> Option<String> {
    metadata_string(metadata, "brand.logo")
        .or_else(|| metadata_string(metadata, "layout.logo"))
        .or_else(|| metadata_string(metadata, "logo"))
        .filter(|value| !value.trim().is_empty())
}

#[derive(Clone, Debug)]
struct ExportTable {
    headers: Vec<String>,
    alignments: Vec<String>,
    rows: Vec<Vec<String>>,
}

fn export_table_from_delimited_code(language: Option<&str>, code: &str) -> Option<ExportTable> {
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
    let rows = rows
        .into_iter()
        .map(|row| {
            (0..headers.len())
                .map(|index| row.get(index).cloned().unwrap_or_default())
                .collect()
        })
        .collect();
    Some(ExportTable {
        headers,
        alignments,
        rows,
    })
}

fn export_table_from_transform_html(html: &str) -> Option<ExportTable> {
    if !html.contains("<table") || !html.contains("transform-table") {
        return None;
    }
    let header_section = html_between(html, "<thead", "</thead>")?;
    let headers = html_table_cells(header_section, "th");
    if headers.is_empty() {
        return None;
    }
    let body_section = html_between(html, "<tbody", "</tbody>").unwrap_or("");
    let mut rows = Vec::new();
    let mut rest = body_section;
    while let Some((row_html, next)) = next_html_tag_block(rest, "tr") {
        let row = html_table_cells(row_html, "td");
        if !row.is_empty() {
            rows.push(
                (0..headers.len())
                    .map(|index| row.get(index).cloned().unwrap_or_default())
                    .collect(),
            );
        }
        rest = next;
    }
    let alignments = headers.iter().map(|_| "left".to_string()).collect();
    Some(ExportTable {
        headers,
        alignments,
        rows,
    })
}

fn html_between<'a>(html: &'a str, open_prefix: &str, close_tag: &str) -> Option<&'a str> {
    let open_start = html.find(open_prefix)?;
    let open_end = html[open_start..].find('>')? + open_start + 1;
    let close_start = html[open_end..].find(close_tag)? + open_end;
    Some(&html[open_end..close_start])
}

fn next_html_tag_block<'a>(html: &'a str, tag: &str) -> Option<(&'a str, &'a str)> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let open_start = html.find(&open)?;
    let open_end = html[open_start..].find('>')? + open_start + 1;
    let close_start = html[open_end..].find(&close)? + open_end;
    let close_end = close_start + close.len();
    Some((&html[open_end..close_start], &html[close_end..]))
}

fn html_table_cells(row_html: &str, tag: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut rest = row_html;
    while let Some((cell_html, next)) = next_html_tag_block(rest, tag) {
        let text = decode_export_html_entities(&strip_export_html_tags(cell_html))
            .trim()
            .to_string();
        cells.push(text);
        rest = next;
    }
    cells
}

fn export_header_footer(response: &CompileResponse, options: &Value) -> (String, String) {
    export_header_footer_for_page(response, options, 1, 1)
}

fn export_header_footer_for_page(
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

fn render_section_template(
    response: &CompileResponse,
    template: &str,
    page: usize,
    pages: usize,
) -> String {
    let classification = metadata_string(&response.metadata, "classification").unwrap_or_default();
    render_export_template_for_page(template, response, &classification, page, pages)
}

fn render_export_template_for_page(
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

fn boolean_option(options: &Value, name: &str, aliases: &[&str], default: bool) -> bool {
    std::iter::once(name)
        .chain(aliases.iter().copied())
        .find_map(|key| options.get(key).and_then(Value::as_bool))
        .unwrap_or(default)
}

fn include_styles(options: &Value) -> bool {
    boolean_option(options, "includeStyles", &[], true)
}

fn include_syntax_highlighting(options: &Value) -> bool {
    boolean_option(options, "includeSyntaxHighlighting", &[], true)
}

fn include_cover_page(options: &Value) -> bool {
    boolean_option(options, "coverPage", &["includeCoverPage"], true)
}

fn include_page_numbers(options: &Value) -> bool {
    boolean_option(options, "pageNumbers", &["includePageNumbers"], true)
}

fn layout_preset(options: &Value) -> &str {
    match options.get("layoutPreset").and_then(Value::as_str) {
        Some("compact") => "compact",
        Some("presentation") => "presentation",
        _ => "business",
    }
}

fn layout_page_size(metadata: &Value) -> String {
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

fn explicit_layout_margins(metadata: &Value) -> Option<String> {
    metadata_string(metadata, "layout.margins")
        .or_else(|| metadata_string(metadata, "margins"))
        .map(|value| value.to_ascii_lowercase().replace([' ', '-'], ""))
        .filter(|value| matches!(value.as_str(), "narrow" | "compact" | "normal" | "wide"))
}

fn highlight_code_blocks(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(pre_start) = rest.find("<pre><code") {
        output.push_str(&rest[..pre_start]);
        let candidate = &rest[pre_start..];
        let Some(open_end) = candidate.find('>') else {
            output.push_str(candidate);
            return output;
        };
        let content_start = open_end + 1;
        let Some(close_start) = candidate[content_start..].find("</code></pre>") else {
            output.push_str(candidate);
            return output;
        };
        let close_start = content_start + close_start;
        let close_end = close_start + "</code></pre>".len();
        let open_tag = &candidate[..content_start];
        let code = &candidate[content_start..close_start];
        output.push_str(open_tag);
        output.push_str(&highlight_code_content(
            &decode_export_html_entities(code),
            code_language(open_tag),
        ));
        output.push_str("</code></pre>");
        rest = &candidate[close_end..];
    }
    output.push_str(rest);
    output
}

fn code_language(open_tag: &str) -> Option<&str> {
    let marker = "language-";
    let start = open_tag.find(marker)? + marker.len();
    let language = open_tag[start..]
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
        .next()
        .filter(|value| !value.is_empty())?;
    Some(language)
}

fn highlight_code_content(code: &str, language: Option<&str>) -> String {
    let mut output = String::with_capacity(code.len());
    let bytes = code.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if is_comment_start(code, index, language) {
            let end = code[index..]
                .find('\n')
                .map(|offset| index + offset)
                .unwrap_or(code.len());
            push_span(&mut output, "syn-comment", &code[index..end]);
            index = end;
        } else if matches!(bytes[index], b'\'' | b'"') {
            let end = quoted_literal_end(code, index);
            push_span(&mut output, "syn-string", &code[index..end]);
            index = end;
        } else if bytes[index].is_ascii_digit() {
            let end = scan_while(code, index, |byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_')
            });
            push_span(&mut output, "syn-number", &code[index..end]);
            index = end;
        } else if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let end = scan_while(code, index, |byte| {
                byte.is_ascii_alphanumeric() || byte == b'_'
            });
            let token = &code[index..end];
            if is_keyword(token) {
                push_span(&mut output, "syn-keyword", token);
            } else {
                output.push_str(&escape_html(token));
            }
            index = end;
        } else {
            let ch = code[index..]
                .chars()
                .next()
                .expect("index should point at a char boundary");
            output.push_str(&escape_html(&ch.to_string()));
            index += ch.len_utf8();
        }
    }
    output
}

fn is_comment_start(code: &str, index: usize, language: Option<&str>) -> bool {
    let rest = &code[index..];
    if rest.starts_with("//") {
        return true;
    }
    if rest.starts_with('#') {
        return matches!(
            language,
            Some("py" | "python" | "sh" | "bash" | "zsh" | "yaml" | "yml" | "toml")
        );
    }
    false
}

fn quoted_literal_end(code: &str, start: usize) -> usize {
    let quote = code.as_bytes()[start];
    let mut index = start + 1;
    let bytes = code.as_bytes();
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == quote {
            return index + 1;
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn scan_while(code: &str, start: usize, predicate: impl Fn(u8) -> bool) -> usize {
    let mut index = start;
    let bytes = code.as_bytes();
    while index < bytes.len() && predicate(bytes[index]) {
        index += 1;
    }
    index
}

fn is_keyword(token: &str) -> bool {
    matches!(
        token,
        "as" | "async"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "def"
            | "else"
            | "enum"
            | "export"
            | "false"
            | "fn"
            | "for"
            | "from"
            | "function"
            | "if"
            | "impl"
            | "import"
            | "in"
            | "let"
            | "match"
            | "mod"
            | "mut"
            | "return"
            | "self"
            | "struct"
            | "true"
            | "type"
            | "use"
            | "var"
            | "while"
    )
}

fn push_span(output: &mut String, class_name: &str, text: &str) {
    output.push_str("<span class=\"");
    output.push_str(class_name);
    output.push_str("\">");
    output.push_str(&escape_html(text));
    output.push_str("</span>");
}

fn include_glossary(options: &Value) -> bool {
    options
        .get("includeGlossary")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn include_comments(options: &Value) -> bool {
    options
        .get("includeComments")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn include_provenance(options: &Value) -> bool {
    options
        .get("includeProvenance")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn appendix_export_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
    let mut lines = glossary_export_lines(response, options);
    lines.extend(comment_export_lines(response, options));
    lines.extend(provenance_export_lines(response, options));
    lines.extend(legal_disclaimer_export_lines(response));
    lines
}

fn glossary_export_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
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

fn comment_export_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
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

fn provenance_export_lines(response: &CompileResponse, options: &Value) -> Vec<String> {
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

fn legal_disclaimer_export_lines(response: &CompileResponse) -> Vec<String> {
    let Some(disclaimer) = metadata_string(&response.metadata, "legalDisclaimer")
        .filter(|value| !value.trim().is_empty())
    else {
        return Vec::new();
    };
    vec![String::new(), "Legal Disclaimer".to_string(), disclaimer]
}

fn empty_as<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

fn html_appendix_sections(response: &CompileResponse, options: &Value) -> String {
    [
        html_glossary_section(response, options),
        html_comments_section(response, options),
        html_provenance_section(response, options),
        html_legal_disclaimer_section(response),
    ]
    .join("")
}

fn html_glossary_section(response: &CompileResponse, options: &Value) -> String {
    if !include_glossary(options) || response.semantic.glossary.is_empty() {
        return String::new();
    }
    let entries = response
        .semantic
        .glossary
        .iter()
        .map(|(term, definition)| {
            format!(
                "<dt>{}</dt><dd>{}</dd>",
                escape_html(term),
                escape_html(definition)
            )
        })
        .collect::<String>();
    format!("<section class=\"export-glossary\"><h2>Glossary</h2><dl>{entries}</dl></section>")
}

fn html_comments_section(response: &CompileResponse, options: &Value) -> String {
    if !include_comments(options)
        || (response.semantic.comments.is_empty() && response.semantic.change_notes.is_empty())
    {
        return String::new();
    }
    let entries = response
        .semantic
        .comments
        .iter()
        .map(|comment| {
            let created_at = empty_as(comment.created_at.as_str(), "undated");
            let author = empty_as(comment.author.as_str(), "local");
            format!(
                "<li><strong>Line {}</strong> <span>{}</span> <em>{} at {}</em><p>{}</p></li>",
                comment.line,
                escape_html(&comment.state),
                escape_html(author),
                escape_html(created_at),
                escape_html(&comment.text)
            )
        })
        .collect::<String>();
    let change_entries = response
        .semantic
        .change_notes
        .iter()
        .map(|note| {
            let created_at = empty_as(note.created_at.as_str(), "undated");
            let author = empty_as(note.author.as_str(), "local");
            format!(
                "<li><strong>Line {}</strong> <em>{} at {}</em><p>{}</p></li>",
                note.line,
                escape_html(author),
                escape_html(created_at),
                escape_html(&note.text)
            )
        })
        .collect::<String>();
    let changes = if change_entries.is_empty() {
        String::new()
    } else {
        format!("<h3>Change Notes</h3><ol>{change_entries}</ol>")
    };
    format!(
        "<section class=\"export-comments\"><h2>Review Comments</h2><ol>{entries}</ol>{changes}</section>"
    )
}

fn html_provenance_section(response: &CompileResponse, options: &Value) -> String {
    if !include_provenance(options)
        || (response.semantic.ai_sources.is_empty()
            && response.semantic.ai_assisted_sections.is_empty())
    {
        return String::new();
    }
    let source_entries = response
        .semantic
        .ai_sources
        .iter()
        .map(|source| {
            format!(
                "<li><strong>{}</strong> <span>{}</span><p>{}; reviewed by {} on {}; {}; prompt: {}</p></li>",
                escape_html(empty_as(source.provider.as_str(), "unknown provider")),
                escape_html(empty_as(source.model.as_str(), "unknown model")),
                escape_html(empty_as(source.date.as_str(), "undated")),
                escape_html(empty_as(source.reviewed_by.as_str(), "unreviewed")),
                escape_html(empty_as(source.reviewed_at.as_str(), "undated")),
                escape_html(&source.status),
                escape_html(empty_as(
                    source.prompt_summary.as_str(),
                    "no prompt summary"
                ))
            )
        })
        .collect::<String>();
    let section_entries = response
        .semantic
        .ai_assisted_sections
        .iter()
        .map(|section| {
            format!(
                "<li><strong>{}</strong> <span>line {}</span><p>{}; reviewed by {} on {}; source: {}; prompt: {}</p></li>",
                escape_html(&section.heading),
                section.line,
                escape_html(&section.status),
                escape_html(empty_as(section.reviewed_by.as_str(), "unreviewed")),
                escape_html(empty_as(section.reviewed_at.as_str(), "undated")),
                escape_html(empty_as(section.source.as_str(), "unspecified source")),
                escape_html(empty_as(section.prompt_summary.as_str(), "no prompt summary"))
            )
        })
        .collect::<String>();
    let entries = format!("{source_entries}{section_entries}");
    format!(
        "<section class=\"export-provenance\"><h2>AI Provenance</h2><ol>{entries}</ol></section>"
    )
}

fn html_legal_disclaimer_section(response: &CompileResponse) -> String {
    let Some(disclaimer) = metadata_string(&response.metadata, "legalDisclaimer")
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    format!(
        "<section class=\"export-legal\"><h2>Legal Disclaimer</h2><p>{}</p></section>",
        escape_html(&disclaimer)
    )
}

fn is_generated_toc_heading(block: &DocumentBlock) -> bool {
    matches!(
        block,
        DocumentBlock::Heading { level, text, .. }
            if *level == 2 && text == "Table of Contents"
    )
}

fn is_generated_toc_body(block: &DocumentBlock) -> bool {
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

fn collect_docx_media(response: &CompileResponse) -> Vec<ExportMedia> {
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

fn is_external_hyperlink(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
}

fn appendix_pages(response: &CompileResponse, options: &Value) -> Vec<Vec<String>> {
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

fn block_export_lines(block: &DocumentBlock) -> Vec<String> {
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
                let mut lines = vec![table_export_line(&None, &None, &table.headers)];
                lines.extend(table.rows.iter().map(|row| row.join(" | ")));
                return lines;
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
            alignments: _,
            rows,
            ..
        } => {
            let mut lines = vec![table_export_line(id, caption, headers)];
            lines.extend(rows.iter().map(|row| row.join(" | ")));
            lines
        }
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
                let mut lines = vec![table_export_line(&None, &None, &table.headers)];
                lines.extend(table.rows.iter().map(|row| row.join(" | ")));
                return lines;
            }
            raw_html_export_lines(html)
        }
    }
}

fn paragraph_export_line(text: &str, inlines: &[InlineNode]) -> String {
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

fn raw_html_export_lines(html: &str) -> Vec<String> {
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

fn strip_export_html_tags(html: &str) -> String {
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

fn decode_export_html_entities(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn layout_export_lines(directive: &str, options: &str, settings: &LayoutSettings) -> Vec<String> {
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

fn layout_summary(options: &str, settings: &LayoutSettings) -> String {
    let mut parts = Vec::new();
    if let Some(columns) = settings.columns {
        parts.push(format!("columns={columns}"));
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

fn slide_title_from_options(options: &str, settings: &LayoutSettings) -> String {
    settings.title.clone().unwrap_or_else(|| {
        let trimmed = options.trim();
        if trimmed.is_empty() {
            "Slide".to_string()
        } else {
            trimmed.to_string()
        }
    })
}

fn slide_notes_from_options(settings: &LayoutSettings) -> Vec<String> {
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

fn callout_export_line(callout_type: &str, title: &str, text: &str) -> String {
    let mut parts = vec![format!("Callout: {callout_type}")];
    if !title.is_empty() {
        parts.push(title.to_string());
    }
    if !text.is_empty() {
        parts.push(text.to_string());
    }
    parts.join(": ")
}

fn transform_export_line(name: &str, text: &str) -> String {
    let label = format!("Transform: {name}");
    if text.is_empty() {
        label
    } else {
        format!("{label}: {text}")
    }
}

fn figure_export_line(
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

fn normalized_float(float: Option<&str>) -> Option<&'static str> {
    match float?.trim().to_ascii_lowercase().as_str() {
        "left" => Some("left"),
        "right" => Some("right"),
        "center" | "centre" => Some("center"),
        _ => None,
    }
}

fn table_export_line(id: &Option<String>, caption: &Option<String>, headers: &[String]) -> String {
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

fn equation_export_line(id: &Option<String>, text: &str, caption: &Option<String>) -> String {
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

fn export_css(
    brand_color: &str,
    watermark: &str,
    brand_font: &str,
    page_numbers: bool,
    layout_preset: &str,
    syntax_highlighting: bool,
    metadata: &Value,
) -> String {
    let page_counter_rule = if page_numbers {
        "@bottom-center{content:'Page ' counter(page) ' of ' counter(pages)}"
    } else {
        ""
    };
    let (body_margin, body_line_height, cover_min_height, heading_size, page_margin) =
        match layout_preset {
            "compact" => ("32px", "1.42", "72vh", "36px", "18mm"),
            "presentation" => ("64px", "1.7", "78vh", "54px", "20mm"),
            _ => ("48px", "1.55", "85vh", "44px", "24mm"),
        };
    let page_size = match layout_page_size(metadata).as_str() {
        "letter" => "Letter",
        "legal" => "Legal",
        _ => "A4",
    };
    let page_margin = match explicit_layout_margins(metadata).as_deref() {
        Some("narrow") | Some("compact") => "12mm",
        Some("wide") => "32mm",
        Some("normal") => "24mm",
        _ => page_margin,
    };
    let syntax_rules = if syntax_highlighting {
        ".syn-keyword{color:#7c3aed;font-weight:700}.syn-string{color:#047857}.syn-number{color:#b45309}.syn-comment{color:#64748b;font-style:italic}"
    } else {
        ""
    };
    let figure_position_rules = "figure[data-position='top'] img,.figure-position-top img{object-position:center top}figure[data-position='bottom'] img,.figure-position-bottom img{object-position:center bottom}figure[data-position='left'] img,.figure-position-left img{object-position:left center}figure[data-position='right'] img,.figure-position-right img{object-position:right center}figure[data-position='top-left'] img,.figure-position-top-left img{object-position:left top}figure[data-position='top-right'] img,.figure-position-top-right img{object-position:right top}figure[data-position='bottom-left'] img,.figure-position-bottom-left img{object-position:left bottom}figure[data-position='bottom-right'] img,.figure-position-bottom-right img{object-position:right bottom}";
    format!(
        "body{{font-family:{};margin:{body_margin};color:#1f2937;line-height:{body_line_height}}}.running-header{{position:running(header);border-bottom:3px solid {brand_color};padding-bottom:8px;color:#475569}}.cover{{min-height:{cover_min_height};display:flex;flex-direction:column;justify-content:center;border-left:10px solid {brand_color};padding-left:32px;page-break-after:always}}.cover-logo{{max-width:160px;max-height:80px;object-fit:contain;margin-bottom:24px}}.cover h1{{font-size:{heading_size};margin:0 0 12px}}.subtitle{{font-size:22px;color:#475569}}.status{{display:inline-block;color:{brand_color};font-weight:700;text-transform:uppercase}}footer{{display:flex;justify-content:space-between;gap:16px;margin-top:40px;border-top:1px solid #cbd5e1;padding-top:12px;color:#475569}}h1,h2,h3{{color:#111827}}p,li,blockquote{{orphans:2;widows:2}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #cbd5e1;padding:6px 8px}}figure[data-float='right'],.figure-float-right{{float:right;max-width:45%;margin:0 0 16px 24px}}figure[data-float='left'],.figure-float-left{{float:left;max-width:45%;margin:0 24px 16px 0}}figure[data-fit='cover'] img,.figure-fit-cover img{{width:100%;aspect-ratio:16/9;object-fit:cover}}{figure_position_rules}.citation{{color:{brand_color};font-weight:700}}.glossary-term{{border-bottom:1px dotted {brand_color};color:{brand_color};cursor:help}}.callout{{border-left:4px solid {brand_color};background:#eefaf4;padding:10px 12px;margin:14px 0}}.callout strong{{display:block;color:#0f5132;margin-bottom:4px}}.equation{{margin:18px 0}}.math-rendered{{font-family:Georgia,'Times New Roman',serif;font-size:1.08em}}.math-display{{padding:12px;border:1px solid #d8e0e8;background:#f8fafc;text-align:center}}.math-frac{{display:inline-grid;grid-template-rows:auto auto;vertical-align:middle;text-align:center}}.math-frac span:first-child{{border-bottom:1px solid currentColor}}.math-sqrt::before{{content:'√'}}.math-source-inline{{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}}.export-glossary,.export-comments,.export-provenance,.export-legal{{page-break-before:always;border-top:3px solid {brand_color};margin-top:40px;padding-top:16px}}.export-glossary dt{{font-weight:700;color:#111827}}.export-glossary dd{{margin:0 0 10px 0}}.export-comments li,.export-provenance li{{margin-bottom:12px}}.export-comments p,.export-provenance p{{margin:4px 0 0}}{syntax_rules}main::before{{content:'{}';position:fixed;inset:35% auto auto 20%;font-size:64px;color:rgba(0,0,0,.06);transform:rotate(-25deg);z-index:-1}}.page-break{{page-break-after:always}}@page{{size:{page_size};margin:{page_margin};@top-center{{content:element(header)}}{page_counter_rule}}}",
        escape_css(brand_font),
        escape_css(watermark)
    )
}
