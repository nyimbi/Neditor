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

pub(crate) fn render_pdf_bytes(response: &CompileResponse, options: &Value) -> Vec<u8> {
    let pages = build_pdf_pages(response, options);
    let (page_width, page_height, margin_top, margin_left) = pdf_page_layout(response, options);
    let total_pages = pages.len().max(1);
    let mut objects = vec![
        String::new(),
        String::new(),
        "3 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n".to_string(),
    ];
    let mut page_ids = Vec::new();

    for (page_index, page) in pages.iter().enumerate() {
        let page_id = objects.len() + 1;
        let content_id = page_id + 1;
        page_ids.push(page_id);
        let (mut header, mut footer) =
            export_header_footer_for_page(response, options, page_index + 1, total_pages);
        if let Some(section_header) = &page.header {
            header = render_section_template(response, section_header, page_index + 1, total_pages);
        }
        if let Some(section_footer) = &page.footer {
            footer = render_section_template(response, section_footer, page_index + 1, total_pages);
        }
        let mut stream = String::new();
        let mut y = page_height.saturating_sub(margin_top) as i32;
        if !header.trim().is_empty() {
            stream.push_str(&pdf_text_line(
                9,
                margin_left,
                page_height.saturating_sub((margin_top / 2).max(12)) as i32,
                &header,
            ));
        }
        for item in page.items.iter().take(60) {
            match item {
                PdfPageItem::Text(line) => {
                    stream.push_str(&pdf_text_line(10, margin_left, y, line));
                    y -= 12;
                }
                PdfPageItem::Table(table) => {
                    let (table_stream, consumed_height) =
                        pdf_table_stream(table, margin_left, y, page_width - margin_left * 2);
                    stream.push_str(&table_stream);
                    y -= consumed_height;
                }
                PdfPageItem::Figure(figure) => {
                    let (figure_stream, consumed_height) =
                        pdf_figure_stream(figure, margin_left, y, page_width - margin_left * 2);
                    stream.push_str(&figure_stream);
                    y -= consumed_height;
                }
            }
        }
        if !footer.trim().is_empty() {
            stream.push_str(&pdf_text_line(
                9,
                margin_left,
                (margin_top / 2).max(12) as i32,
                &footer,
            ));
        }
        objects.push(format!(
            "{page_id} 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 {page_width} {page_height}] /Resources << /Font << /F1 3 0 R >> >> /Contents {content_id} 0 R >> endobj\n"
        ));
        objects.push(format!(
            "{content_id} 0 obj << /Length {} >> stream\n{}endstream endobj\n",
            stream.len(),
            stream
        ));
    }

    let kids = page_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    objects[0] = "1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n".to_string();
    objects[1] = format!(
        "2 0 obj << /Type /Pages /Kids [{kids}] /Count {} >> endobj\n",
        page_ids.len()
    );
    let info_id = objects.len() + 1;
    objects.push(render_pdf_info_object(info_id, response));

    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for object in &objects {
        offsets.push(pdf.len());
        pdf.extend_from_slice(object.as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R /Info {info_id} 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref
        )
        .as_bytes(),
    );
    pdf
}

fn pdf_text_line(font_size: u8, x: u32, y: i32, text: &str) -> String {
    format!(
        "BT /F1 {font_size} Tf {x} {y} Td ({}) Tj ET\n",
        escape_pdf(text)
    )
}

fn pdf_table_stream(table: &PdfTable, x: u32, top_y: i32, width: u32) -> (String, i32) {
    let mut stream = String::new();
    let caption = table_export_line(&table.id, &table.caption, &table.headers);
    stream.push_str(&pdf_text_line(10, x, top_y, &caption));
    let mut current_y = top_y - 18;
    let column_count = table.headers.len().max(1);
    let column_width = (width / column_count as u32).max(48);
    let row_height = 18i32;

    stream.push_str(&pdf_table_row_stream(
        &table.headers,
        &table.alignments,
        x,
        current_y,
        column_width,
        row_height,
    ));
    current_y -= row_height;
    for row in &table.rows {
        stream.push_str(&pdf_table_row_stream(
            row,
            &table.alignments,
            x,
            current_y,
            column_width,
            row_height,
        ));
        current_y -= row_height;
    }

    let consumed = (top_y - current_y + 10).max(28);
    (stream, consumed)
}

fn pdf_figure_stream(figure: &PdfFigure, x: u32, top_y: i32, max_width: u32) -> (String, i32) {
    let (width, height) =
        pdf_figure_box_size(figure.dimensions, max_width, 180, figure.fit.as_deref());
    let x = pdf_aligned_x(x, max_width, width, figure.float.as_deref());
    let bottom_y = top_y - height;
    let label = figure
        .alt
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Figure");
    let mut stream = format!("{x} {bottom_y} {width} {height} re S\n");
    stream.push_str(&pdf_text_line(
        8,
        (x + 8).max(0) as u32,
        bottom_y + (height / 2),
        label,
    ));
    stream.push_str(&pdf_text_line(
        10,
        x.max(0) as u32,
        bottom_y - 14,
        &figure.caption_line,
    ));
    (stream, height + pdf_figure_caption_height())
}

fn pdf_aligned_x(base_x: u32, max_width: u32, width: i32, float: Option<&str>) -> i32 {
    let base_x = base_x as i32;
    let remaining = max_width as i32 - width;
    match normalized_float(float) {
        Some("right") => base_x + remaining.max(0),
        Some("center") => base_x + (remaining.max(0) / 2),
        _ => base_x,
    }
}

fn pdf_figure_box_size(
    dimensions: Option<ExportImageDimensions>,
    max_width: u32,
    max_height: i32,
    fit: Option<&str>,
) -> (i32, i32) {
    let fallback = (240, 135);
    if normalized_fit(fit) == Some("cover") {
        return fallback;
    }
    let Some(dimensions) = dimensions else {
        return fallback;
    };
    let width = dimensions.width_px * 72.0 / 96.0;
    let height = dimensions.height_px * 72.0 / 96.0;
    if width <= 0.0 || height <= 0.0 {
        return fallback;
    }
    let scale = (max_width as f64 / width)
        .min(max_height as f64 / height)
        .min(1.0);
    let scaled_width = (width * scale).round() as i32;
    let scaled_height = (height * scale).round() as i32;
    if scaled_width <= 0 || scaled_height <= 0 {
        fallback
    } else {
        (scaled_width, scaled_height)
    }
}

fn pdf_table_row_stream(
    cells: &[String],
    alignments: &[String],
    x: u32,
    y: i32,
    column_width: u32,
    row_height: i32,
) -> String {
    let mut stream = String::new();
    let column_count = cells.len().max(1);
    for index in 0..column_count {
        let cell_x = x + (index as u32 * column_width);
        let cell = cells.get(index).map(String::as_str).unwrap_or("");
        stream.push_str(&format!("{cell_x} {y} {column_width} {row_height} re S\n"));
        let text_x = pdf_table_cell_text_x(
            cell_x,
            column_width,
            cell,
            alignments.get(index).map(String::as_str),
        );
        stream.push_str(&pdf_text_line(8, text_x, y + 6, cell));
    }
    stream
}

fn pdf_table_cell_text_x(
    cell_x: u32,
    column_width: u32,
    text: &str,
    alignment: Option<&str>,
) -> u32 {
    match alignment {
        Some("center") => {
            let text_width = approximate_pdf_text_width(text, 8);
            cell_x + column_width.saturating_sub(text_width) / 2
        }
        Some("right") => {
            let text_width = approximate_pdf_text_width(text, 8);
            cell_x + column_width.saturating_sub(text_width + 4)
        }
        _ => cell_x + 4,
    }
}

fn approximate_pdf_text_width(text: &str, font_size: u32) -> u32 {
    (text.chars().count() as u32)
        .saturating_mul(font_size)
        .saturating_div(2)
        .max(4)
}

fn render_pdf_info_object(object_id: usize, response: &CompileResponse) -> String {
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
    format!(
        "{object_id} 0 obj << /Title ({}) /Author ({}) /Subject ({}) /Keywords ({}) /Producer (NEditor) >> endobj\n",
        escape_pdf(&response.semantic.title),
        escape_pdf(&author),
        escape_pdf(&format!("Status: {}", response.semantic.status)),
        escape_pdf(&keywords)
    )
}

pub(crate) fn render_docx_bytes(
    response: &CompileResponse,
    options_value: &Value,
) -> Result<Vec<u8>, String> {
    let media = collect_docx_media(response);
    let hyperlinks = collect_docx_hyperlinks(response);
    let section_overrides = collect_docx_section_overrides(response);
    let include_native_comments = docx_has_native_comments(response, options_value);
    let include_native_footnotes = docx_has_native_footnotes(response);
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("[Content_Types].xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        render_docx_content_types(
            &media,
            &section_overrides,
            include_native_comments,
            include_native_footnotes,
        )
        .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.add_directory("_rels/", options)
        .map_err(|err| err.to_string())?;
    zip.start_file("_rels/.rels", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_root_relationships("word/document.xml").as_bytes())
        .map_err(|err| err.to_string())?;
    zip.add_directory("docProps/", options)
        .map_err(|err| err.to_string())?;
    zip.start_file("docProps/core.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_core_properties(response).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("docProps/app.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_docx_app_properties(response).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("docProps/custom.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_custom_properties(response).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.add_directory("word/", options)
        .map_err(|err| err.to_string())?;
    zip.add_directory("word/_rels/", options)
        .map_err(|err| err.to_string())?;
    zip.start_file("word/_rels/document.xml.rels", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        render_docx_document_relationships(
            &media,
            &hyperlinks,
            &section_overrides,
            include_native_comments,
            include_native_footnotes,
        )
        .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    if !media.is_empty() {
        zip.add_directory("word/media/", options)
            .map_err(|err| err.to_string())?;
        for item in &media {
            zip.start_file(format!("word/{}", item.path), options)
                .map_err(|err| err.to_string())?;
            zip.write_all(&item.bytes).map_err(|err| err.to_string())?;
        }
    }
    zip.start_file("word/header1.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_docx_header(response).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("word/footer1.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_docx_footer(response, options_value).as_bytes())
        .map_err(|err| err.to_string())?;
    for part in docx_section_parts(&section_overrides) {
        zip.start_file(format!("word/{}", part.target), options)
            .map_err(|err| err.to_string())?;
        zip.write_all(render_docx_header_footer_part(response, part).as_bytes())
            .map_err(|err| err.to_string())?;
    }
    if include_native_comments {
        zip.start_file("word/comments.xml", options)
            .map_err(|err| err.to_string())?;
        zip.write_all(render_docx_comments(response).as_bytes())
            .map_err(|err| err.to_string())?;
    }
    if include_native_footnotes {
        zip.start_file("word/footnotes.xml", options)
            .map_err(|err| err.to_string())?;
        zip.write_all(render_docx_footnotes(response).as_bytes())
            .map_err(|err| err.to_string())?;
    }
    zip.start_file("word/document.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        render_docx_document(
            response,
            options_value,
            &media,
            &hyperlinks,
            &section_overrides,
        )
        .as_bytes(),
    )
    .map_err(|err| err.to_string())?;
    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}

pub(crate) fn render_pptx_bytes(
    response: &CompileResponse,
    options_value: &Value,
) -> Result<Vec<u8>, String> {
    let slides = build_pptx_slides(response, options_value);
    let media = collect_pptx_media(response);
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("[Content_Types].xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_pptx_content_types(&slides, &media).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.add_directory("_rels/", options)
        .map_err(|err| err.to_string())?;
    zip.start_file("_rels/.rels", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_root_relationships("ppt/presentation.xml").as_bytes())
        .map_err(|err| err.to_string())?;
    zip.add_directory("docProps/", options)
        .map_err(|err| err.to_string())?;
    zip.start_file("docProps/core.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_core_properties(response).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("docProps/app.xml", options)
        .map_err(|err| err.to_string())?;
    let notes_count = slides
        .iter()
        .filter(|slide| !slide.notes.is_empty())
        .count();
    zip.write_all(render_pptx_app_properties(response, slides.len(), notes_count).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.start_file("docProps/custom.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_custom_properties(response).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.add_directory("ppt/_rels/", options)
        .map_err(|err| err.to_string())?;
    zip.start_file("ppt/_rels/presentation.xml.rels", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_pptx_relationships(slides.len()).as_bytes())
        .map_err(|err| err.to_string())?;
    zip.add_directory("ppt/slides/", options)
        .map_err(|err| err.to_string())?;
    let has_slide_relationships = !media.is_empty()
        || slides
            .iter()
            .any(|slide| !slide.notes.is_empty() || !slide.hyperlinks.is_empty());
    if has_slide_relationships {
        zip.add_directory("ppt/slides/_rels/", options)
            .map_err(|err| err.to_string())?;
    }
    if slides.iter().any(|slide| !slide.notes.is_empty()) {
        zip.add_directory("ppt/notesSlides/", options)
            .map_err(|err| err.to_string())?;
    }
    if !media.is_empty() {
        zip.add_directory("ppt/media/", options)
            .map_err(|err| err.to_string())?;
        for item in &media {
            zip.start_file(format!("ppt/{}", item.path), options)
                .map_err(|err| err.to_string())?;
            zip.write_all(&item.bytes).map_err(|err| err.to_string())?;
        }
    }
    zip.start_file("ppt/presentation.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_pptx_presentation(slides.len()).as_bytes())
        .map_err(|err| err.to_string())?;
    for (index, slide) in slides.iter().enumerate() {
        let slide_relationship_media = pptx_slide_relationship_media(slide, &media);
        let slide_pictures = pptx_slide_pictures(slide, &media);
        if !slide_relationship_media.is_empty()
            || !slide.notes.is_empty()
            || !slide.hyperlinks.is_empty()
        {
            zip.start_file(
                format!("ppt/slides/_rels/slide{}.xml.rels", index + 1),
                options,
            )
            .map_err(|err| err.to_string())?;
            zip.write_all(
                render_pptx_slide_relationships(
                    &slide_relationship_media,
                    &slide.hyperlinks,
                    (!slide.notes.is_empty()).then_some(index + 1),
                )
                .as_bytes(),
            )
            .map_err(|err| err.to_string())?;
        }
        zip.start_file(format!("ppt/slides/slide{}.xml", index + 1), options)
            .map_err(|err| err.to_string())?;
        zip.write_all(render_pptx_slide(slide, &slide_pictures).as_bytes())
            .map_err(|err| err.to_string())?;
        if !slide.notes.is_empty() {
            zip.start_file(
                format!("ppt/notesSlides/notesSlide{}.xml", index + 1),
                options,
            )
            .map_err(|err| err.to_string())?;
            zip.write_all(render_pptx_notes_slide(slide).as_bytes())
                .map_err(|err| err.to_string())?;
        }
    }
    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}

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

fn render_docx_content_types(
    media: &[ExportMedia],
    section_overrides: &[DocxSectionOverride],
    include_comments: bool,
    include_footnotes: bool,
) -> String {
    let mut defaults = vec![
        (
            "rels".to_string(),
            "application/vnd.openxmlformats-package.relationships+xml".to_string(),
        ),
        ("xml".to_string(), "application/xml".to_string()),
    ];
    for item in media {
        if !defaults
            .iter()
            .any(|(extension, _)| extension == &item.extension)
        {
            defaults.push((item.extension.clone(), item.content_type.clone()));
        }
    }
    let default_xml = defaults
        .iter()
        .map(|(extension, content_type)| {
            format!(
                r#"<Default Extension="{}" ContentType="{}"/>"#,
                escape_xml(extension),
                escape_xml(content_type)
            )
        })
        .collect::<String>();
    let comments_override = if include_comments {
        r#"<Override PartName="/word/comments.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml"/>"#
    } else {
        ""
    };
    let footnotes_override = if include_footnotes {
        r#"<Override PartName="/word/footnotes.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml"/>"#
    } else {
        ""
    };
    let section_part_overrides = docx_section_parts(section_overrides)
        .into_iter()
        .map(|part| {
            let content_type = match part.kind {
                DocxHeaderFooterKind::Header => {
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"
                }
                DocxHeaderFooterKind::Footer => {
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"
                }
            };
            format!(
                r#"<Override PartName="/word/{}" ContentType="{}"/>"#,
                escape_xml(&part.target),
                content_type
            )
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">{default_xml}<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/><Override PartName="/docProps/custom.xml" ContentType="application/vnd.openxmlformats-officedocument.custom-properties+xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/header1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/><Override PartName="/word/footer1.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>{section_part_overrides}{comments_override}{footnotes_override}</Types>"#
    )
}

fn render_root_relationships(office_document_target: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="{}"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/><Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties" Target="docProps/custom.xml"/></Relationships>"#,
        escape_xml(office_document_target)
    )
}

fn render_docx_document_relationships(
    media: &[ExportMedia],
    hyperlinks: &[ExportHyperlink],
    section_overrides: &[DocxSectionOverride],
    include_comments: bool,
    include_footnotes: bool,
) -> String {
    let media_relationships = media
        .iter()
        .map(|item| {
            format!(
                r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="{}"/>"#,
                escape_xml(&item.relationship_id),
                escape_xml(&item.path)
            )
        })
        .collect::<String>();
    let hyperlink_relationships = hyperlinks
        .iter()
        .map(|item| {
            format!(
                r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{}" TargetMode="External"/>"#,
                escape_xml(&item.relationship_id),
                escape_xml(&item.url)
            )
        })
        .collect::<String>();
    let section_part_relationships = docx_section_parts(section_overrides)
        .into_iter()
        .map(|part| {
            let relationship_type = match part.kind {
                DocxHeaderFooterKind::Header => {
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header"
                }
                DocxHeaderFooterKind::Footer => {
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"
                }
            };
            format!(
                r#"<Relationship Id="{}" Type="{}" Target="{}"/>"#,
                escape_xml(&part.relationship_id),
                relationship_type,
                escape_xml(&part.target)
            )
        })
        .collect::<String>();
    let comments_relationship = if include_comments {
        r#"<Relationship Id="rIdComments" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/>"#
    } else {
        ""
    };
    let footnotes_relationship = if include_footnotes {
        r#"<Relationship Id="rIdFootnotes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="footnotes.xml"/>"#
    } else {
        ""
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdHeader1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header1.xml"/><Relationship Id="rIdFooter1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer1.xml"/>{section_part_relationships}{comments_relationship}{footnotes_relationship}{media_relationships}{hyperlink_relationships}</Relationships>"#
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

fn render_docx_app_properties(response: &CompileResponse) -> String {
    let body_text = export_body_text_from_ast(&response.document_ast);
    let words = body_text.split_whitespace().count();
    let characters = body_text.chars().filter(|ch| !ch.is_whitespace()).count();
    let company = metadata_string(&response.metadata, "brand.name")
        .or_else(|| metadata_string(&response.metadata, "client"))
        .unwrap_or_else(|| "NEditor".to_string());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"><Application>NEditor</Application><DocSecurity>0</DocSecurity><ScaleCrop>false</ScaleCrop><Pages>1</Pages><Words>{words}</Words><Characters>{characters}</Characters><Company>{}</Company><AppVersion>{}</AppVersion></Properties>"#,
        escape_xml(&company),
        escape_xml(env!("CARGO_PKG_VERSION"))
    )
}

#[derive(Clone, Debug)]
struct DocxSectionOverride {
    header: Option<DocxHeaderFooterPart>,
    footer: Option<DocxHeaderFooterPart>,
}

#[derive(Clone, Debug)]
struct DocxHeaderFooterPart {
    kind: DocxHeaderFooterKind,
    relationship_id: String,
    target: String,
    template: String,
}

#[derive(Clone, Copy, Debug)]
enum DocxHeaderFooterKind {
    Header,
    Footer,
}

#[derive(Clone, Debug)]
struct DocxSectionProperties {
    header_relationship_id: String,
    footer_relationship_id: String,
    columns: Option<usize>,
}

impl Default for DocxSectionProperties {
    fn default() -> Self {
        Self {
            header_relationship_id: "rIdHeader1".to_string(),
            footer_relationship_id: "rIdFooter1".to_string(),
            columns: None,
        }
    }
}

impl DocxSectionProperties {
    fn apply_override(&mut self, section_override: &DocxSectionOverride) {
        if let Some(header) = &section_override.header {
            self.header_relationship_id = header.relationship_id.clone();
        }
        if let Some(footer) = &section_override.footer {
            self.footer_relationship_id = footer.relationship_id.clone();
        }
    }
}

fn collect_docx_section_overrides(response: &CompileResponse) -> Vec<DocxSectionOverride> {
    let mut header_index = 2;
    let mut footer_index = 2;
    response
        .document_ast
        .blocks
        .iter()
        .filter_map(|block| {
            let DocumentBlock::Layout {
                directive,
                settings,
                ..
            } = block
            else {
                return None;
            };
            if !is_docx_section_layout(directive, settings) {
                return None;
            }
            let header = settings.header.clone().map(|template| {
                let part = DocxHeaderFooterPart {
                    kind: DocxHeaderFooterKind::Header,
                    relationship_id: format!("rIdHeader{header_index}"),
                    target: format!("header{header_index}.xml"),
                    template,
                };
                header_index += 1;
                part
            });
            let footer = settings.footer.clone().map(|template| {
                let part = DocxHeaderFooterPart {
                    kind: DocxHeaderFooterKind::Footer,
                    relationship_id: format!("rIdFooter{footer_index}"),
                    target: format!("footer{footer_index}.xml"),
                    template,
                };
                footer_index += 1;
                part
            });
            Some(DocxSectionOverride { header, footer })
        })
        .collect()
}

fn docx_section_parts(section_overrides: &[DocxSectionOverride]) -> Vec<&DocxHeaderFooterPart> {
    section_overrides
        .iter()
        .flat_map(|section_override| {
            [
                section_override.header.as_ref(),
                section_override.footer.as_ref(),
            ]
            .into_iter()
            .flatten()
        })
        .collect()
}

fn is_docx_section_layout(directive: &str, settings: &LayoutSettings) -> bool {
    directive == "section-break"
        || settings.columns.is_some()
        || settings.header.is_some()
        || settings.footer.is_some()
}

fn render_pptx_app_properties(
    response: &CompileResponse,
    slide_count: usize,
    notes_count: usize,
) -> String {
    let company = metadata_string(&response.metadata, "brand.name")
        .or_else(|| metadata_string(&response.metadata, "client"))
        .unwrap_or_else(|| "NEditor".to_string());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"><Application>NEditor</Application><PresentationFormat>On-screen Show (16:9)</PresentationFormat><Slides>{slide_count}</Slides><Notes>{notes_count}</Notes><HiddenSlides>0</HiddenSlides><Company>{}</Company><AppVersion>{}</AppVersion></Properties>"#,
        escape_xml(&company),
        escape_xml(env!("CARGO_PKG_VERSION"))
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

fn render_docx_document(
    response: &CompileResponse,
    options: &Value,
    media: &[ExportMedia],
    hyperlinks: &[ExportHyperlink],
    section_overrides: &[DocxSectionOverride],
) -> String {
    let mut body = String::new();
    for line in export_metadata_lines(response, options) {
        body.push_str(&docx_paragraph(&line));
    }
    body.push_str(&docx_page_break());
    let mut skip_next_toc_body = false;
    let bibliography_keys = response
        .bibliography
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<Vec<_>>();
    let mut next_list_is_bibliography = false;
    let page_layout = docx_page_layout(response, options);
    let mut section_index = 0;
    let mut current_section = DocxSectionProperties::default();
    let mut pending_flow: Option<LayoutSettings> = None;
    for block in &response.document_ast.blocks {
        if skip_next_toc_body {
            skip_next_toc_body = false;
            if is_generated_toc_body(block) {
                continue;
            }
        }
        if is_generated_toc_heading(block) {
            body.push_str(&docx_generated_toc(response));
            skip_next_toc_body = true;
            continue;
        }
        if next_list_is_bibliography {
            if let DocumentBlock::List { ordered, items, .. } = block {
                body.push_str(&docx_bibliography_list(*ordered, items));
                next_list_is_bibliography = false;
                continue;
            }
            next_list_is_bibliography = false;
        }
        if matches!(block, DocumentBlock::Heading { text, .. } if text == "Bibliography") {
            next_list_is_bibliography = true;
        }
        if let DocumentBlock::Layout {
            directive,
            settings,
            ..
        } = block
        {
            if matches_layout_break(settings.break_before.as_deref()) {
                pending_flow = Some(settings.clone());
            }
            if is_docx_section_layout(directive, settings) {
                body.push_str(&docx_section_break(&current_section, page_layout));
                if let Some(section_override) = section_overrides.get(section_index) {
                    current_section.apply_override(section_override);
                }
                current_section.columns = settings.columns;
                section_index += 1;
                if matches_layout_break(settings.break_after.as_deref()) {
                    body.push_str(&docx_page_break());
                }
                if settings.has_pagination_controls() {
                    pending_flow = Some(settings.clone());
                }
                continue;
            }
        }
        let block_xml = render_docx_block(block, media, hyperlinks, &bibliography_keys);
        if let Some(flow) = pending_flow.take() {
            body.push_str(&docx_apply_flow_properties(block_xml, &flow));
        } else {
            body.push_str(&block_xml);
        }
    }
    if docx_has_native_comments(response, options) {
        body.push_str(&render_docx_comment_references(response));
    }
    for line in appendix_export_lines(response, options) {
        if matches!(
            line.as_str(),
            "Glossary" | "Review Comments" | "AI Provenance" | "Legal Disclaimer"
        ) {
            body.push_str(&docx_heading(1, &line));
        } else {
            body.push_str(&docx_paragraph(&line));
        }
    }
    let final_section = docx_section_properties(&current_section, page_layout);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><w:body>{body}{final_section}</w:body></w:document>"#
    )
}

fn docx_has_native_comments(response: &CompileResponse, options: &Value) -> bool {
    include_comments(options)
        && (!response.semantic.comments.is_empty() || !response.semantic.change_notes.is_empty())
}

fn docx_has_native_footnotes(response: &CompileResponse) -> bool {
    !docx_footnote_entries(response).is_empty()
}

fn docx_footnote_entries(response: &CompileResponse) -> Vec<&FootnoteEntry> {
    response
        .document_ast
        .blocks
        .iter()
        .filter_map(|block| match block {
            DocumentBlock::Footnotes { entries, .. } => Some(entries.iter()),
            _ => None,
        })
        .flatten()
        .collect()
}

fn render_docx_comments(response: &CompileResponse) -> String {
    let mut comments = response
        .semantic
        .comments
        .iter()
        .enumerate()
        .map(|(index, comment)| {
            let author = empty_as(comment.author.as_str(), "local");
            let created_at = if comment.created_at.is_empty() {
                Utc::now().to_rfc3339()
            } else {
                comment.created_at.clone()
            };
            format!(
                r#"<w:comment w:id="{index}" w:author="{}" w:date="{}"><w:p><w:r><w:t>{}</w:t></w:r></w:p></w:comment>"#,
                escape_xml(author),
                escape_xml(&created_at),
                escape_xml(&comment.text)
            )
        })
        .collect::<String>();
    let change_note_offset = response.semantic.comments.len();
    comments.push_str(
        &response
            .semantic
            .change_notes
            .iter()
            .enumerate()
            .map(|(index, note)| {
                let comment_id = change_note_offset + index;
                let author = empty_as(note.author.as_str(), "local");
                let created_at = if note.created_at.is_empty() {
                    Utc::now().to_rfc3339()
                } else {
                    note.created_at.clone()
                };
                format!(
                    r#"<w:comment w:id="{comment_id}" w:author="{}" w:date="{}"><w:p><w:r><w:t>Change note: {}</w:t></w:r></w:p></w:comment>"#,
                    escape_xml(author),
                    escape_xml(&created_at),
                    escape_xml(&note.text)
                )
            })
            .collect::<String>(),
    );
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{comments}</w:comments>"#
    )
}

fn render_docx_footnotes(response: &CompileResponse) -> String {
    let entries = docx_footnote_entries(response)
        .into_iter()
        .map(|entry| {
            format!(
                r#"<w:footnote w:id="{}"><w:p><w:pPr><w:pStyle w:val="FootnoteText"/></w:pPr><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:t>{}</w:t></w:r><w:r><w:t xml:space="preserve"> {}</w:t></w:r></w:p></w:footnote>"#,
                entry.number,
                entry.number,
                escape_xml(&entry.text)
            )
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>{entries}</w:footnotes>"#
    )
}

fn render_docx_comment_references(response: &CompileResponse) -> String {
    let mut references = response
        .semantic
        .comments
        .iter()
        .enumerate()
        .map(|(index, comment)| {
            let label = format!("Comment on source line {}", comment.line);
            format!(
                r#"<w:p><w:commentRangeStart w:id="{index}"/><w:r><w:t>{}</w:t></w:r><w:commentRangeEnd w:id="{index}"/><w:r><w:commentReference w:id="{index}"/></w:r></w:p>"#,
                escape_xml(&label)
            )
        })
        .collect::<String>();
    let change_note_offset = response.semantic.comments.len();
    references.push_str(
        &response
            .semantic
            .change_notes
            .iter()
            .enumerate()
            .map(|(index, note)| {
                let comment_id = change_note_offset + index;
                let label = format!("Change note on source line {}", note.line);
                format!(
                    r#"<w:p><w:commentRangeStart w:id="{comment_id}"/><w:r><w:t>{}</w:t></w:r><w:commentRangeEnd w:id="{comment_id}"/><w:r><w:commentReference w:id="{comment_id}"/></w:r></w:p>"#,
                    escape_xml(&label)
                )
            })
            .collect::<String>(),
    );
    references
}

fn render_docx_header(response: &CompileResponse) -> String {
    let header = metadata_string(&response.metadata, "layout.header")
        .unwrap_or_else(|| response.semantic.title.clone());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{}</w:hdr>"#,
        render_docx_header_footer_paragraph(response, &header)
    )
}

fn render_docx_footer(response: &CompileResponse, options: &Value) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{}</w:ftr>"#,
        render_docx_header_footer_paragraph(response, &docx_footer_template(response, options))
    )
}

fn render_docx_header_footer_part(
    response: &CompileResponse,
    part: &DocxHeaderFooterPart,
) -> String {
    let body = render_docx_header_footer_paragraph(response, &part.template);
    match part.kind {
        DocxHeaderFooterKind::Header => {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{body}</w:hdr>"#
            )
        }
        DocxHeaderFooterKind::Footer => {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?><w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{body}</w:ftr>"#
            )
        }
    }
}

fn docx_footer_template(response: &CompileResponse, options: &Value) -> String {
    metadata_string(&response.metadata, "layout.footer").unwrap_or_else(|| {
        if include_page_numbers(options) {
            "Page {{page}} of {{pages}}".to_string()
        } else {
            String::new()
        }
    })
}

fn render_docx_header_footer_paragraph(response: &CompileResponse, template: &str) -> String {
    let classification = metadata_string(&response.metadata, "classification").unwrap_or_default();
    if template.contains("{{page}}") || template.contains("{{pages}}") {
        let template = template
            .replace("{{title}}", &response.semantic.title)
            .replace("{{status}}", &response.semantic.status)
            .replace("{{classification}}", &classification);
        return docx_paragraph_with_page_fields(&template);
    }
    docx_paragraph(&render_export_template_for_page(
        template,
        response,
        &classification,
        1,
        1,
    ))
}

fn docx_paragraph_with_page_fields(template: &str) -> String {
    let mut runs = String::new();
    let mut remaining = template;
    while !remaining.is_empty() {
        let page_pos = remaining.find("{{page}}");
        let pages_pos = remaining.find("{{pages}}");
        let next = match (page_pos, pages_pos) {
            (Some(page), Some(pages)) if page < pages => Some((page, "PAGE", "{{page}}", "1")),
            (Some(_), Some(pages)) => Some((pages, "NUMPAGES", "{{pages}}", "1")),
            (Some(page), None) => Some((page, "PAGE", "{{page}}", "1")),
            (None, Some(pages)) => Some((pages, "NUMPAGES", "{{pages}}", "1")),
            (None, None) => None,
        };
        let Some((index, instruction, marker, fallback)) = next else {
            runs.push_str(&docx_text_run(remaining));
            break;
        };
        runs.push_str(&docx_text_run(&remaining[..index]));
        runs.push_str(&format!(
            r#"<w:fldSimple w:instr="{instruction}"><w:r><w:t>{fallback}</w:t></w:r></w:fldSimple>"#
        ));
        remaining = &remaining[index + marker.len()..];
    }
    format!("<w:p>{runs}</w:p>")
}

fn docx_text_run(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    format!(
        r#"<w:r><w:t xml:space="preserve">{}</w:t></w:r>"#,
        escape_xml(text)
    )
}

fn render_docx_block(
    block: &DocumentBlock,
    media: &[ExportMedia],
    hyperlinks: &[ExportHyperlink],
    bibliography_keys: &[&str],
) -> String {
    match block {
        DocumentBlock::Heading {
            level,
            text,
            anchor,
            ..
        } => docx_heading_with_bookmark(*level, text, Some(anchor)),
        DocumentBlock::Paragraph { text, inlines, .. } => {
            docx_paragraph_from_inlines(text, inlines, hyperlinks, bibliography_keys)
        }
        DocumentBlock::List { ordered, items, .. } => docx_list(*ordered, items),
        DocumentBlock::TaskList { items, .. } => docx_task_list(items),
        DocumentBlock::BlockQuote { text, .. } => docx_paragraph(&format!("Quote: {text}")),
        DocumentBlock::CodeBlock { language, code, .. } => {
            if let Some(table) = export_table_from_delimited_code(language.as_deref(), code) {
                let mut output = docx_paragraph(&table_export_line(&None, &None, &table.headers));
                output.push_str(&docx_table(&table.headers, &table.alignments, &table.rows));
                return output;
            }
            let label = language
                .as_deref()
                .filter(|value| !value.is_empty())
                .map(|value| format!("Code ({value})"))
                .unwrap_or_else(|| "Code".to_string());
            format!("{}{}", docx_paragraph(&label), docx_paragraph(code))
        }
        DocumentBlock::Table {
            id,
            caption,
            headers,
            alignments,
            rows,
            ..
        } => {
            let mut output = String::new();
            if id.is_some() || caption.is_some() {
                output.push_str(&docx_bookmarked_paragraph(
                    &table_export_line(id, caption, headers),
                    id.as_deref(),
                ));
            }
            output.push_str(&docx_table(headers, alignments, rows));
            output
        }
        DocumentBlock::Figure { .. } => docx_figure(block, media),
        DocumentBlock::Equation {
            id, caption, text, ..
        } => docx_bookmarked_paragraph(&equation_export_line(id, text, caption), id.as_deref()),
        DocumentBlock::Layout { directive, .. } if directive == "page-break" => docx_page_break(),
        DocumentBlock::Layout {
            directive,
            settings,
            ..
        } if is_docx_section_layout(directive, settings) => String::new(),
        DocumentBlock::Layout {
            directive, options, ..
        } => docx_paragraph(format!("Layout: {directive} {options}").trim()),
        DocumentBlock::Callout {
            callout_type,
            title,
            text,
            ..
        } => docx_paragraph(&callout_export_line(callout_type, title, text)),
        DocumentBlock::Footnotes { .. } => String::new(),
        DocumentBlock::ReviewComment { comment, .. } => docx_paragraph(&format!(
            "Review comment: {} | {} | {}",
            comment.state, comment.author, comment.text
        )),
        DocumentBlock::ChangeNote { note, .. } => {
            docx_paragraph(&format!("Change note: {} | {}", note.author, note.text))
        }
        DocumentBlock::AiSource { provenance, .. } => docx_paragraph(&format!(
            "AI source: {} / {} | {}",
            empty_as(&provenance.provider, "unknown"),
            empty_as(&provenance.model, "unknown"),
            empty_as(&provenance.status, "unreviewed")
        )),
        DocumentBlock::Transform { name, text, .. } => {
            docx_paragraph(&transform_export_line(name, text))
        }
        DocumentBlock::RawHtml { html, .. } => {
            if let Some(table) = export_table_from_transform_html(html) {
                let mut output = docx_paragraph(&table_export_line(&None, &None, &table.headers));
                output.push_str(&docx_table(&table.headers, &table.alignments, &table.rows));
                return output;
            }
            raw_html_export_lines(html)
                .into_iter()
                .map(|line| docx_paragraph(&line))
                .collect::<String>()
        }
    }
}

fn docx_generated_toc(response: &CompileResponse) -> String {
    format!(
        "{}{}",
        docx_heading(2, "Table of Contents"),
        docx_toc_field(toc_depth(&response.metadata))
    )
}

fn docx_heading(level: usize, text: &str) -> String {
    docx_heading_with_bookmark(level, text, None)
}

fn docx_heading_with_bookmark(level: usize, text: &str, bookmark: Option<&str>) -> String {
    let style = format!("Heading{}", level.clamp(1, 6));
    let bookmark_start = bookmark.map(docx_bookmark_start).unwrap_or_default();
    let bookmark_end = bookmark.map(docx_bookmark_end).unwrap_or_default();
    format!(
        r#"<w:p><w:pPr><w:pStyle w:val="{style}"/><w:widowControl/></w:pPr>{bookmark_start}<w:r><w:t>{}</w:t></w:r>{bookmark_end}</w:p>"#,
        escape_xml(text)
    )
}

fn docx_toc_field(depth: usize) -> String {
    let depth = depth.clamp(1, 6);
    format!(
        r#"<w:p><w:fldSimple w:instr="TOC \o &quot;1-{depth}&quot; \h \z \u"><w:r><w:t>Update table of contents in Word to refresh page numbers.</w:t></w:r></w:fldSimple></w:p>"#
    )
}

fn docx_paragraph(text: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:widowControl/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>"#,
        escape_xml(text)
    )
}

fn docx_bookmarked_paragraph(text: &str, bookmark: Option<&str>) -> String {
    let bookmark_start = bookmark.map(docx_bookmark_start).unwrap_or_default();
    let bookmark_end = bookmark.map(docx_bookmark_end).unwrap_or_default();
    format!(
        r#"<w:p><w:pPr><w:widowControl/></w:pPr>{bookmark_start}<w:r><w:t>{}</w:t></w:r>{bookmark_end}</w:p>"#,
        escape_xml(text)
    )
}

fn docx_apply_flow_properties(xml: String, settings: &LayoutSettings) -> String {
    let properties = docx_flow_properties(settings);
    if properties.is_empty() || xml.is_empty() {
        return xml;
    }
    if let Some(index) = xml.find("<w:pPr>") {
        let insert_at = index + "<w:pPr>".len();
        let mut output = String::with_capacity(xml.len() + properties.len());
        output.push_str(&xml[..insert_at]);
        output.push_str(&properties);
        output.push_str(&xml[insert_at..]);
        return output;
    }
    if let Some(index) = xml.find("<w:p>") {
        let insert_at = index + "<w:p>".len();
        let mut output = String::with_capacity(xml.len() + properties.len() + 15);
        output.push_str(&xml[..insert_at]);
        output.push_str("<w:pPr>");
        output.push_str(&properties);
        output.push_str("</w:pPr>");
        output.push_str(&xml[insert_at..]);
        return output;
    }
    xml
}

fn docx_flow_properties(settings: &LayoutSettings) -> String {
    let mut properties = Vec::new();
    if matches_layout_break(settings.break_before.as_deref()) {
        properties.push("<w:pageBreakBefore/>");
    }
    if settings.keep_with_next {
        properties.push("<w:keepNext/>");
    }
    if settings.keep_together {
        properties.push("<w:keepLines/>");
    }
    properties.join("")
}

fn docx_paragraph_from_inlines(
    fallback_text: &str,
    inlines: &[InlineNode],
    hyperlinks: &[ExportHyperlink],
    bibliography_keys: &[&str],
) -> String {
    if inlines.is_empty() {
        return docx_paragraph(fallback_text);
    }
    let runs = inlines
        .iter()
        .map(|inline| match inline {
            InlineNode::Text { text } => docx_text_run(text),
            InlineNode::CrossReference { raw, .. } => docx_text_run(&inline_export_text(raw)),
            InlineNode::Citation { raw, keys, .. } => {
                docx_citation_run(raw, keys, bibliography_keys)
            }
            InlineNode::FootnoteReference { number, .. } => docx_footnote_reference_run(*number),
            InlineNode::Strong { text } => docx_text_run_with_properties(text, "<w:b/>"),
            InlineNode::Emphasis { text } => docx_text_run_with_properties(text, "<w:i/>"),
            InlineNode::Code { text } => {
                docx_text_run_with_properties(text, r#"<w:rStyle w:val="Code"/>"#)
            }
            InlineNode::Link { text, url } => {
                if let Some(anchor) = url.strip_prefix('#') {
                    docx_internal_hyperlink_run(text, anchor)
                } else if let Some(link) = hyperlinks.iter().find(|item| item.url == *url) {
                    docx_hyperlink_run(text, &link.relationship_id)
                } else {
                    docx_text_run(text)
                }
            }
        })
        .collect::<String>();
    format!("<w:p><w:pPr><w:widowControl/></w:pPr>{runs}</w:p>")
}

fn inline_export_text(text: &str) -> String {
    decode_export_html_entities(&strip_export_html_tags(text))
}

fn docx_text_run_with_properties(text: &str, properties: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    format!(
        r#"<w:r><w:rPr>{properties}</w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
        escape_xml(text)
    )
}

fn docx_hyperlink_run(text: &str, relationship_id: &str) -> String {
    format!(
        r#"<w:hyperlink r:id="{}" w:history="1"><w:r><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r></w:hyperlink>"#,
        escape_xml(relationship_id),
        escape_xml(text)
    )
}

fn docx_internal_hyperlink_run(text: &str, anchor: &str) -> String {
    let run = format!(
        r#"<w:r><w:rPr><w:color w:val="0563C1"/><w:u w:val="single"/></w:rPr><w:t xml:space="preserve">{}</w:t></w:r>"#,
        escape_xml(text)
    );
    docx_internal_hyperlink_content(&run, anchor)
}

fn docx_internal_hyperlink_content(content: &str, anchor: &str) -> String {
    format!(
        r#"<w:hyperlink w:anchor="{}" w:history="1">{content}</w:hyperlink>"#,
        escape_xml(&docx_bookmark_name(anchor)),
    )
}

fn docx_citation_run(raw: &str, keys: &[String], bibliography_keys: &[&str]) -> String {
    let label = inline_export_text(raw);
    let matched_keys = keys
        .iter()
        .filter(|key| bibliography_keys.contains(&key.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    let Some(key) = matched_keys.first() else {
        return docx_text_run(&label);
    };
    let citation_field = docx_citation_field(&label, &matched_keys);
    docx_internal_hyperlink_content(&citation_field, &format!("bib:{key}"))
}

fn docx_citation_field(label: &str, keys: &[&str]) -> String {
    let Some(first_key) = keys.first() else {
        return docx_text_run(label);
    };
    let mut instruction = format!("CITATION {first_key}");
    for key in keys.iter().skip(1) {
        instruction.push_str(" \\m ");
        instruction.push_str(key);
    }
    instruction.push_str(" \\l 1033");
    format!(
        r#"<w:fldSimple w:instr="{}"><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:fldSimple>"#,
        escape_xml(&instruction),
        escape_xml(label)
    )
}

fn docx_footnote_reference_run(number: usize) -> String {
    format!(
        r#"<w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:footnoteReference w:id="{number}"/></w:r>"#
    )
}

fn docx_bookmark_start(anchor: &str) -> String {
    let id = docx_bookmark_id(anchor);
    format!(
        r#"<w:bookmarkStart w:id="{id}" w:name="{}"/>"#,
        escape_xml(&docx_bookmark_name(anchor))
    )
}

fn docx_bookmark_end(anchor: &str) -> String {
    format!(r#"<w:bookmarkEnd w:id="{}"/>"#, docx_bookmark_id(anchor))
}

fn docx_bookmark_name(anchor: &str) -> String {
    let mut name = anchor
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if name.is_empty()
        || !name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
    {
        name.insert_str(0, "neditor_");
    }
    name
}

fn docx_bookmark_id(anchor: &str) -> usize {
    anchor.bytes().fold(17usize, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(byte as usize)
    }) % 2_000_000
        + 10
}

fn docx_list(ordered: bool, items: &[String]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let marker = if ordered {
                format!("{}.", index + 1)
            } else {
                "-".to_string()
            };
            docx_paragraph(&format!("{marker} {item}"))
        })
        .collect::<String>()
}

fn docx_bibliography_list(ordered: bool, items: &[String]) -> String {
    let mut output = docx_bibliography_field();
    output.push_str(
        &items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let marker = if ordered {
                    format!("{}.", index + 1)
                } else {
                    "-".to_string()
                };
                let text = format!("{marker} {item}");
                let bookmark = bibliography_key_from_item(item).map(|key| format!("bib:{key}"));
                docx_bookmarked_paragraph(&text, bookmark.as_deref())
            })
            .collect::<String>(),
    );
    output
}

fn docx_bibliography_field() -> String {
    r#"<w:p><w:fldSimple w:instr="BIBLIOGRAPHY \l 1033"><w:r><w:t>Bibliography</w:t></w:r></w:fldSimple></w:p>"#
        .to_string()
}

fn bibliography_key_from_item(item: &str) -> Option<&str> {
    let (key, _) = item.split_once('.')?;
    let key = key.trim();
    (!key.is_empty()
        && key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':')))
    .then_some(key)
}

fn docx_task_list(items: &[crate::document_ast::TaskListItem]) -> String {
    items
        .iter()
        .map(|item| {
            docx_paragraph(&format!(
                "[{}] {}",
                if item.checked { "x" } else { " " },
                item.text
            ))
        })
        .collect::<String>()
}

fn docx_page_break() -> String {
    r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#.to_string()
}

fn docx_section_break(
    section: &DocxSectionProperties,
    page_layout: (u32, u32, u32, u32, u32, u32),
) -> String {
    format!(
        r#"<w:p><w:pPr>{}</w:pPr></w:p>"#,
        docx_section_properties(section, page_layout)
    )
}

fn docx_section_properties(
    section: &DocxSectionProperties,
    page_layout: (u32, u32, u32, u32, u32, u32),
) -> String {
    let (page_width, page_height, margin_top, margin_right, margin_bottom, margin_left) =
        page_layout;
    let columns = section
        .columns
        .map(|columns| format!(r#"<w:cols w:num="{columns}" w:space="720"/>"#))
        .unwrap_or_default();
    format!(
        r#"<w:sectPr><w:headerReference w:type="default" r:id="{}"/><w:footerReference w:type="default" r:id="{}"/><w:pgSz w:w="{page_width}" w:h="{page_height}"/><w:pgMar w:top="{margin_top}" w:right="{margin_right}" w:bottom="{margin_bottom}" w:left="{margin_left}"/>{columns}</w:sectPr>"#,
        escape_xml(&section.header_relationship_id),
        escape_xml(&section.footer_relationship_id)
    )
}

fn docx_page_layout(response: &CompileResponse, options: &Value) -> (u32, u32, u32, u32, u32, u32) {
    let (width, height) = match layout_page_size(&response.metadata).as_str() {
        "letter" => (12240, 15840),
        "legal" => (12240, 20160),
        _ => (11906, 16838),
    };
    let margin = match explicit_layout_margins(&response.metadata).as_deref() {
        Some("narrow") | Some("compact") => 720,
        Some("wide") => 1800,
        Some("normal") => 1440,
        _ => match layout_preset(options) {
            "compact" => 1080,
            "presentation" => 1200,
            _ => 1440,
        },
    };
    (width, height, margin, margin, margin, margin)
}

fn docx_table(headers: &[String], alignments: &[String], rows: &[Vec<String>]) -> String {
    let mut table = String::from(
        r#"<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="0" w:type="auto"/></w:tblPr>"#,
    );
    table.push_str(&docx_table_row(headers, alignments));
    for row in rows {
        table.push_str(&docx_table_row(row, alignments));
    }
    table.push_str("</w:tbl>");
    table
}

fn docx_table_row(cells: &[String], alignments: &[String]) -> String {
    let cells = cells
        .iter()
        .enumerate()
        .map(|(index, cell)| docx_cell(cell, alignments.get(index).map(String::as_str)))
        .collect::<String>();
    format!("<w:tr>{cells}</w:tr>")
}

fn docx_cell(text: &str, alignment: Option<&str>) -> String {
    let alignment = match alignment {
        Some("center") => r#"<w:pPr><w:jc w:val="center"/></w:pPr>"#,
        Some("right") => r#"<w:pPr><w:jc w:val="right"/></w:pPr>"#,
        _ => "",
    };
    format!(
        r#"<w:tc><w:tcPr><w:tcW w:w="2400" w:type="dxa"/></w:tcPr><w:p>{alignment}<w:r><w:t>{}</w:t></w:r></w:p></w:tc>"#,
        escape_xml(text)
    )
}

fn docx_figure(block: &DocumentBlock, media: &[ExportMedia]) -> String {
    let DocumentBlock::Figure {
        id,
        src,
        alt,
        caption,
        float,
        fit,
        position,
        source,
        ..
    } = block
    else {
        return String::new();
    };
    let caption_text = figure_export_line(id, src, alt, caption, float, fit, position);
    let Some(src) = src else {
        return docx_paragraph(&caption_text);
    };
    let Some((media_index, item)) = media.iter().enumerate().find(|(_, item)| {
        item.source == *src
            && item.source_file.as_deref()
                == source.as_ref().map(|range| range.source_file.as_str())
    }) else {
        return docx_paragraph(&caption_text);
    };
    let doc_pr_id = media_index + 1;
    let name = caption
        .as_deref()
        .or(alt.as_deref())
        .or(id.as_deref())
        .unwrap_or("Figure");
    let (image_width, image_height) =
        export_media_emu_size(item, 4_320_000, 3_240_000, (4_320_000, 2_430_000));
    let src_rect = drawingml_source_crop(
        item.dimensions,
        image_width,
        image_height,
        fit.as_deref(),
        position.as_deref(),
    );
    let paragraph_props = docx_figure_paragraph_props(float.as_deref());
    let drawing = format!(
        r#"<w:p>{paragraph_props}<w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{image_width}" cy="{image_height}"/><wp:docPr id="{doc_pr_id}" name="{}"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="{doc_pr_id}" name="{}"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{}"/>{src_rect}<a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{image_width}" cy="{image_height}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#,
        escape_xml(name),
        escape_xml(name),
        escape_xml(&item.relationship_id)
    );
    format!(
        "{drawing}{}",
        docx_bookmarked_paragraph(&caption_text, id.as_deref())
    )
}

fn docx_figure_paragraph_props(float: Option<&str>) -> &'static str {
    match normalized_float(float) {
        Some("right") => r#"<w:pPr><w:jc w:val="right"/></w:pPr>"#,
        Some("center") => r#"<w:pPr><w:jc w:val="center"/></w:pPr>"#,
        Some("left") => r#"<w:pPr><w:jc w:val="left"/></w:pPr>"#,
        _ => "",
    }
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

fn collect_docx_hyperlinks(response: &CompileResponse) -> Vec<ExportHyperlink> {
    let mut hyperlinks = Vec::new();
    for block in &response.document_ast.blocks {
        let DocumentBlock::Paragraph { inlines, .. } = block else {
            continue;
        };
        for inline in inlines {
            let InlineNode::Link { url, .. } = inline else {
                continue;
            };
            if !is_external_hyperlink(url)
                || hyperlinks
                    .iter()
                    .any(|item: &ExportHyperlink| item.url == *url)
            {
                continue;
            }
            let index = hyperlinks.len() + 1;
            hyperlinks.push(ExportHyperlink {
                url: url.clone(),
                relationship_id: format!("rIdHyperlink{index}"),
            });
        }
    }
    hyperlinks
}

fn is_external_hyperlink(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("mailto:")
}

fn collect_pptx_media(response: &CompileResponse) -> Vec<ExportMedia> {
    collect_docx_media(response)
}

#[derive(Clone, Debug)]
enum PdfPageItem {
    Text(String),
    Table(PdfTable),
    Figure(PdfFigure),
}

#[derive(Clone, Debug)]
struct PdfPage {
    items: Vec<PdfPageItem>,
    header: Option<String>,
    footer: Option<String>,
}

#[derive(Clone, Debug)]
struct PdfTable {
    id: Option<String>,
    caption: Option<String>,
    headers: Vec<String>,
    alignments: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
struct PdfFigure {
    caption_line: String,
    alt: Option<String>,
    float: Option<String>,
    fit: Option<String>,
    dimensions: Option<ExportImageDimensions>,
}

fn build_pdf_pages(response: &CompileResponse, options: &Value) -> Vec<PdfPage> {
    let (_, page_height, margin_top, _) = pdf_page_layout(response, options);
    let footer_reserved = (margin_top / 2).max(12) + 24;
    let available_height =
        (page_height as i32 - margin_top as i32 - footer_reserved as i32).max(120);
    let mut paginator = PdfPaginator::new(available_height);
    paginator.extend_text(export_metadata_lines(response, options));
    paginator.finish_page();

    for block in &response.document_ast.blocks {
        match block {
            DocumentBlock::Layout { directive, .. } if directive == "page-break" => {
                paginator.finish_page();
            }
            DocumentBlock::Layout {
                directive,
                options,
                settings,
                ..
            } if directive == "section-break" => {
                paginator.finish_page();
                paginator.apply_section_options(settings);
                paginator.extend_text(layout_export_lines(directive, options, settings));
                if matches_layout_break(settings.break_after.as_deref()) {
                    paginator.finish_page();
                }
            }
            DocumentBlock::Layout {
                directive,
                options,
                settings,
                ..
            } if directive == "layout" => {
                if matches_layout_break(settings.break_before.as_deref()) {
                    paginator.finish_page();
                }
                paginator.apply_section_options(settings);
                paginator.extend_text(layout_export_lines(directive, options, settings));
                if matches_layout_break(settings.break_after.as_deref()) {
                    paginator.finish_page();
                }
            }
            _ => paginator.extend_items(block_pdf_items(block)),
        }
    }
    paginator.finish_page();
    for appendix in appendix_pages(response, options) {
        paginator.extend_text(appendix);
        paginator.finish_page();
    }
    paginator.into_pages()
}

fn block_pdf_items(block: &DocumentBlock) -> Vec<PdfPageItem> {
    if let DocumentBlock::RawHtml { html, .. } = block {
        if let Some(table) = export_table_from_transform_html(html) {
            return vec![PdfPageItem::Table(PdfTable {
                id: None,
                caption: None,
                headers: table.headers,
                alignments: table.alignments,
                rows: table.rows,
            })];
        }
    }
    if let DocumentBlock::CodeBlock { language, code, .. } = block {
        if let Some(table) = export_table_from_delimited_code(language.as_deref(), code) {
            return vec![PdfPageItem::Table(PdfTable {
                id: None,
                caption: None,
                headers: table.headers,
                alignments: table.alignments,
                rows: table.rows,
            })];
        }
    }
    if let DocumentBlock::Table {
        id,
        caption,
        headers,
        alignments,
        rows,
        ..
    } = block
    {
        return vec![PdfPageItem::Table(PdfTable {
            id: id.clone(),
            caption: caption.clone(),
            headers: headers.clone(),
            alignments: alignments.clone(),
            rows: rows.clone(),
        })];
    }
    if let DocumentBlock::Figure {
        id,
        src,
        alt,
        caption,
        float,
        fit,
        position,
        source,
        ..
    } = block
    {
        let dimensions = src.as_deref().and_then(|src| {
            parse_export_image(src, source.as_ref()).and_then(|image| image.dimensions)
        });
        return vec![PdfPageItem::Figure(PdfFigure {
            caption_line: figure_export_line(id, src, alt, caption, float, fit, position),
            alt: alt.clone(),
            float: normalized_float(float.as_deref()).map(str::to_string),
            fit: normalized_fit(fit.as_deref()).map(str::to_string),
            dimensions,
        })];
    }
    pdf_text_items(block_export_lines(block))
}

fn pdf_text_items(lines: Vec<String>) -> Vec<PdfPageItem> {
    lines.into_iter().map(PdfPageItem::Text).collect()
}

struct PdfPaginator {
    pages: Vec<PdfPage>,
    current: Vec<PdfPageItem>,
    current_header: Option<String>,
    current_footer: Option<String>,
    used_height: i32,
    available_height: i32,
}

impl PdfPaginator {
    fn new(available_height: i32) -> Self {
        Self {
            pages: Vec::new(),
            current: Vec::new(),
            current_header: None,
            current_footer: None,
            used_height: 0,
            available_height,
        }
    }

    fn apply_section_options(&mut self, settings: &LayoutSettings) {
        if let Some(header) = &settings.header {
            self.current_header = Some(header.clone());
        }
        if let Some(footer) = &settings.footer {
            self.current_footer = Some(footer.clone());
        }
    }

    fn extend_text(&mut self, lines: Vec<String>) {
        self.extend_items(pdf_text_items(lines));
    }

    fn extend_items(&mut self, items: Vec<PdfPageItem>) {
        for item in items {
            match item {
                PdfPageItem::Text(line) => self.push_text(line),
                PdfPageItem::Table(table) => self.push_table(table),
                PdfPageItem::Figure(figure) => self.push_figure(figure),
            }
        }
    }

    fn push_text(&mut self, line: String) {
        let height = pdf_text_item_height();
        if self.used_height + height > self.available_height {
            self.finish_page();
        }
        self.used_height += height;
        self.current.push(PdfPageItem::Text(line));
    }

    fn push_table(&mut self, table: PdfTable) {
        let mut remaining_rows = table.rows.as_slice();
        let mut continued = false;
        while !remaining_rows.is_empty() {
            let available_rows = self.available_table_rows(continued);
            if available_rows == 0 {
                self.finish_page();
                continue;
            }
            let take_count = remaining_rows.len().min(available_rows);
            let chunk = pdf_table_chunk(&table, remaining_rows[..take_count].to_vec(), continued);
            let height = pdf_table_height(&chunk);
            if self.used_height + height > self.available_height && !self.current.is_empty() {
                self.finish_page();
                continue;
            }
            self.used_height += height;
            self.current.push(PdfPageItem::Table(chunk));
            remaining_rows = &remaining_rows[take_count..];
            continued = true;
        }

        if table.rows.is_empty() {
            let height = pdf_table_height(&table);
            if self.used_height + height > self.available_height {
                self.finish_page();
            }
            self.used_height += height;
            self.current.push(PdfPageItem::Table(table));
        }
    }

    fn push_figure(&mut self, figure: PdfFigure) {
        let height = pdf_figure_height(figure.dimensions, figure.fit.as_deref());
        if self.used_height + height > self.available_height && !self.current.is_empty() {
            self.finish_page();
        }
        self.used_height += height.min(self.available_height);
        self.current.push(PdfPageItem::Figure(figure));
    }

    fn available_table_rows(&self, continued: bool) -> usize {
        let remaining = self.available_height - self.used_height;
        let caption_height = if continued {
            pdf_table_continued_caption_height()
        } else {
            pdf_table_caption_height()
        };
        let available_for_rows = remaining - caption_height - pdf_table_header_height() - 10;
        if available_for_rows < pdf_table_row_height() {
            return 0;
        }
        (available_for_rows / pdf_table_row_height()) as usize
    }

    fn finish_page(&mut self) {
        if self.current.is_empty() {
            return;
        }
        self.pages.push(PdfPage {
            items: std::mem::take(&mut self.current),
            header: self.current_header.clone(),
            footer: self.current_footer.clone(),
        });
        self.used_height = 0;
    }

    fn into_pages(mut self) -> Vec<PdfPage> {
        self.finish_page();
        if self.pages.is_empty() {
            self.pages.push(PdfPage {
                items: Vec::new(),
                header: self.current_header.clone(),
                footer: self.current_footer.clone(),
            });
        }
        self.pages
    }
}

fn pdf_table_chunk(table: &PdfTable, rows: Vec<Vec<String>>, continued: bool) -> PdfTable {
    let caption = if continued {
        Some(table.caption.clone().unwrap_or_else(|| "Table".to_string()) + " (continued)")
    } else {
        table.caption.clone()
    };
    PdfTable {
        id: table.id.clone(),
        caption,
        headers: table.headers.clone(),
        alignments: table.alignments.clone(),
        rows,
    }
}

fn pdf_text_item_height() -> i32 {
    12
}

fn pdf_table_caption_height() -> i32 {
    18
}

fn pdf_table_continued_caption_height() -> i32 {
    18
}

fn pdf_figure_caption_height() -> i32 {
    26
}

fn pdf_table_header_height() -> i32 {
    18
}

fn pdf_table_row_height() -> i32 {
    18
}

fn pdf_table_height(table: &PdfTable) -> i32 {
    pdf_table_caption_height()
        + pdf_table_header_height()
        + (table.rows.len() as i32 * pdf_table_row_height())
        + 10
}

fn pdf_figure_height(dimensions: Option<ExportImageDimensions>, fit: Option<&str>) -> i32 {
    let (_, height) = pdf_figure_box_size(dimensions, 468, 180, fit);
    height + pdf_figure_caption_height()
}

fn pdf_page_layout(response: &CompileResponse, options: &Value) -> (u32, u32, u32, u32) {
    let (width, height) = match layout_page_size(&response.metadata).as_str() {
        "letter" => (612, 792),
        "legal" => (612, 1008),
        _ => (595, 842),
    };
    let margin = match explicit_layout_margins(&response.metadata).as_deref() {
        Some("narrow") | Some("compact") => 34,
        Some("wide") => 91,
        Some("normal") => 68,
        _ => match layout_preset(options) {
            "compact" => 51,
            "presentation" => 57,
            _ => 68,
        },
    };
    (width, height, margin, margin)
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

const PPTX_TABLE_ROWS_PER_SLIDE: usize = 8;

#[derive(Clone, Debug)]
struct PptxSlide {
    title: String,
    lines: Vec<String>,
    layout: PptxLayout,
    header: String,
    footer: String,
    header_override: Option<String>,
    footer_override: Option<String>,
    tables: Vec<PptxTable>,
    media_refs: Vec<MediaRef>,
    hyperlinks: Vec<ExportHyperlink>,
    notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PptxLayout {
    Title,
    Section,
    Content,
    TwoColumn,
    Table,
}

#[derive(Clone, Debug)]
struct PptxTable {
    id: Option<String>,
    caption: Option<String>,
    headers: Vec<String>,
    alignments: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Clone, Debug)]
struct MediaRef {
    source: String,
    source_file: Option<String>,
    float: Option<String>,
    fit: Option<String>,
    position: Option<String>,
}

struct PptxSlidePicture<'a> {
    media: &'a ExportMedia,
    float: Option<&'a str>,
    fit: Option<&'a str>,
    position: Option<&'a str>,
}

impl PptxSlide {
    fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
            layout: PptxLayout::Content,
            header: String::new(),
            footer: String::new(),
            header_override: None,
            footer_override: None,
            tables: Vec::new(),
            media_refs: Vec::new(),
            hyperlinks: Vec::new(),
            notes: Vec::new(),
        }
    }

    fn with_lines(title: impl Into<String>, lines: Vec<String>) -> Self {
        Self {
            title: title.into(),
            lines,
            layout: PptxLayout::Content,
            header: String::new(),
            footer: String::new(),
            header_override: None,
            footer_override: None,
            tables: Vec::new(),
            media_refs: Vec::new(),
            hyperlinks: Vec::new(),
            notes: Vec::new(),
        }
    }
}

fn build_pptx_slides(response: &CompileResponse, options: &Value) -> Vec<PptxSlide> {
    let mut title_slide = PptxSlide::with_lines(
        response.semantic.title.clone(),
        export_metadata_lines(response, options)
            .into_iter()
            .filter(|line| !line.starts_with("Cover: "))
            .collect(),
    );
    title_slide.layout = PptxLayout::Title;
    let mut slides = vec![title_slide];
    let include_agenda = include_pptx_agenda(response, options);
    if include_agenda {
        let lines = response
            .semantic
            .outline
            .iter()
            .filter(|heading| heading.level <= 2)
            .map(|heading| {
                format!(
                    "{}{}",
                    "  ".repeat(heading.level.saturating_sub(1)),
                    heading.text
                )
            })
            .collect::<Vec<_>>();
        if !lines.is_empty() {
            slides.push(PptxSlide::with_lines("Agenda", lines));
        }
    }
    let mut current: Option<PptxSlide> = None;
    let mut skip_next_toc_body = false;

    for block in &response.document_ast.blocks {
        if skip_next_toc_body {
            skip_next_toc_body = false;
            if is_generated_toc_body(block) {
                continue;
            }
        }
        if include_agenda && is_generated_toc_heading(block) {
            skip_next_toc_body = true;
            continue;
        }
        match block {
            DocumentBlock::Heading { level, text, .. } if *level <= 2 => {
                if let Some(slide) = current.take() {
                    if !slide.lines.is_empty() || slide.title != "Continued" {
                        slides.push(slide);
                    }
                }
                current = Some(PptxSlide::new(text.clone()));
            }
            DocumentBlock::Layout { directive, .. } if directive == "page-break" => {
                if let Some(slide) = current.take() {
                    slides.push(slide);
                }
                current = Some(PptxSlide::new("Continued"));
            }
            DocumentBlock::Layout {
                directive,
                options,
                settings,
                ..
            } if directive == "section-break" => {
                if let Some(slide) = current.take() {
                    if !slide.lines.is_empty() || slide.title != "Continued" {
                        slides.push(slide);
                    }
                }
                let mut slide = PptxSlide::with_lines(
                    "Section",
                    layout_export_lines(directive, options, settings),
                );
                slide.layout = PptxLayout::Section;
                apply_pptx_section_options(&mut slide, settings);
                current = Some(slide);
                if matches_layout_break(settings.break_after.as_deref()) {
                    if let Some(slide) = current.take() {
                        slides.push(slide);
                    }
                    current = Some(PptxSlide::new("Continued"));
                }
            }
            DocumentBlock::Layout {
                directive,
                options,
                settings,
                ..
            } if directive == "slide" => {
                if let Some(slide) = current.take() {
                    if !slide.lines.is_empty() || slide.title != "Continued" {
                        slides.push(slide);
                    }
                }
                let mut slide = PptxSlide::new(slide_title_from_options(options, settings));
                slide.layout = pptx_layout_from_options(settings);
                slide.notes = slide_notes_from_options(settings);
                apply_pptx_section_options(&mut slide, settings);
                current = Some(slide);
            }
            DocumentBlock::Layout {
                directive,
                options,
                settings,
                ..
            } if directive == "layout" => {
                if matches_layout_break(settings.break_before.as_deref()) {
                    if let Some(slide) = current.take() {
                        if !slide.lines.is_empty() || slide.title != "Continued" {
                            slides.push(slide);
                        }
                    }
                    current = Some(PptxSlide::new("Continued"));
                }
                if current.is_none() {
                    current = Some(PptxSlide::new("Document"));
                }
                if let Some(slide) = current.as_mut() {
                    apply_pptx_section_options(slide, settings);
                    slide
                        .lines
                        .extend(layout_export_lines(directive, options, settings));
                }
                if matches_layout_break(settings.break_after.as_deref()) {
                    if let Some(slide) = current.take() {
                        slides.push(slide);
                    }
                    current = Some(PptxSlide::new("Continued"));
                }
            }
            _ => {
                if current.is_none() {
                    current = Some(PptxSlide::new("Document"));
                }
                if let Some(slide) = current.as_mut() {
                    add_block_to_pptx_slide(slide, block);
                }
            }
        }
    }

    if let Some(slide) = current {
        if !slide.lines.is_empty() || slide.title != "Continued" {
            slides.push(slide);
        }
    }
    for appendix in appendix_pages(response, options) {
        let title = appendix
            .iter()
            .find(|line| is_appendix_heading(line))
            .cloned()
            .unwrap_or_else(|| "Appendix".to_string());
        slides.push(PptxSlide::with_lines(
            title,
            appendix
                .into_iter()
                .filter(|line| !line.is_empty())
                .filter(|line| !is_appendix_heading(line))
                .collect(),
        ));
    }
    let slides = expand_pptx_table_slides(slides);
    let total_slides = slides.len().max(1);
    slides
        .into_iter()
        .enumerate()
        .map(|(index, mut slide)| {
            if slide.lines.is_empty() {
                slide.lines.push("No body content".to_string());
            }
            let (header, footer) =
                export_header_footer_for_page(response, options, index + 1, total_slides);
            slide.header = slide
                .header_override
                .as_deref()
                .map(|template| {
                    render_section_template(response, template, index + 1, total_slides)
                })
                .unwrap_or(header);
            slide.footer = slide
                .footer_override
                .as_deref()
                .map(|template| {
                    render_section_template(response, template, index + 1, total_slides)
                })
                .unwrap_or(footer);
            slide.lines.truncate(14);
            slide
        })
        .collect()
}

fn apply_pptx_section_options(slide: &mut PptxSlide, settings: &LayoutSettings) {
    slide.header_override = settings.header.clone();
    slide.footer_override = settings.footer.clone();
}

fn pptx_layout_from_options(settings: &LayoutSettings) -> PptxLayout {
    settings
        .layout
        .as_deref()
        .map(|value| match value.trim().to_ascii_lowercase().as_str() {
            "title" | "title-slide" | "cover" => PptxLayout::Title,
            "section" | "section-divider" | "divider" => PptxLayout::Section,
            "two-column" | "two_columns" | "columns" => PptxLayout::TwoColumn,
            "table" | "table-slide" => PptxLayout::Table,
            _ => PptxLayout::Content,
        })
        .unwrap_or(PptxLayout::Content)
}

fn is_appendix_heading(line: &str) -> bool {
    matches!(
        line,
        "Glossary" | "Review Comments" | "AI Provenance" | "Legal Disclaimer"
    )
}

fn expand_pptx_table_slides(slides: Vec<PptxSlide>) -> Vec<PptxSlide> {
    let mut expanded = Vec::new();
    for slide in slides {
        if slide
            .tables
            .iter()
            .all(|table| table.rows.len() <= PPTX_TABLE_ROWS_PER_SLIDE)
        {
            expanded.push(slide);
            continue;
        }

        let mut base = slide.clone();
        base.tables.clear();
        let mut emitted_any = false;
        for table in &slide.tables {
            for (chunk_index, table_chunk) in pptx_table_chunks(table).into_iter().enumerate() {
                let mut next_slide = if !emitted_any {
                    let mut slide = base.clone();
                    slide.lines = vec![table_export_line(
                        &table_chunk.id,
                        &table_chunk.caption,
                        &table_chunk.headers,
                    )];
                    slide
                } else {
                    let mut continued = PptxSlide::with_lines(
                        format!("{} (continued)", slide.title),
                        vec![table_export_line(
                            &table_chunk.id,
                            &table_chunk.caption,
                            &table_chunk.headers,
                        )],
                    );
                    continued.layout = slide.layout;
                    continued.header_override = slide.header_override.clone();
                    continued.footer_override = slide.footer_override.clone();
                    continued
                };
                if chunk_index > 0 && next_slide.lines.is_empty() {
                    next_slide.lines.push(table_export_line(
                        &table_chunk.id,
                        &table_chunk.caption,
                        &table_chunk.headers,
                    ));
                }
                next_slide.tables.push(table_chunk);
                expanded.push(next_slide);
                emitted_any = true;
            }
        }
    }
    expanded
}

fn pptx_table_chunks(table: &PptxTable) -> Vec<PptxTable> {
    if table.rows.len() <= PPTX_TABLE_ROWS_PER_SLIDE {
        return vec![table.clone()];
    }
    table
        .rows
        .chunks(PPTX_TABLE_ROWS_PER_SLIDE)
        .enumerate()
        .map(|(index, rows)| pptx_table_chunk(table, rows.to_vec(), index > 0))
        .collect()
}

fn pptx_table_chunk(table: &PptxTable, rows: Vec<Vec<String>>, continued: bool) -> PptxTable {
    let caption = if continued {
        Some(table.caption.clone().unwrap_or_else(|| "Table".to_string()) + " (continued)")
    } else {
        table.caption.clone()
    };
    PptxTable {
        id: table.id.clone(),
        caption,
        headers: table.headers.clone(),
        alignments: table.alignments.clone(),
        rows,
    }
}

fn include_pptx_agenda(response: &CompileResponse, options: &Value) -> bool {
    options
        .get("includeAgenda")
        .and_then(Value::as_bool)
        .or_else(|| response.metadata.get("toc").and_then(Value::as_bool))
        .unwrap_or(false)
}

fn add_block_to_pptx_slide(slide: &mut PptxSlide, block: &DocumentBlock) {
    slide.lines.extend(block_export_lines(block));
    collect_pptx_block_hyperlinks(slide, block);
    if let DocumentBlock::RawHtml { html, .. } = block {
        if let Some(table) = export_table_from_transform_html(html) {
            if slide.layout == PptxLayout::Content {
                slide.layout = PptxLayout::Table;
            }
            slide.tables.push(PptxTable {
                id: None,
                caption: None,
                headers: table.headers,
                alignments: table.alignments,
                rows: table.rows,
            });
        }
    }
    if let DocumentBlock::CodeBlock { language, code, .. } = block {
        if let Some(table) = export_table_from_delimited_code(language.as_deref(), code) {
            if slide.layout == PptxLayout::Content {
                slide.layout = PptxLayout::Table;
            }
            slide.tables.push(PptxTable {
                id: None,
                caption: None,
                headers: table.headers,
                alignments: table.alignments,
                rows: table.rows,
            });
        }
    }
    if let DocumentBlock::Table {
        id,
        caption,
        headers,
        alignments,
        rows,
        ..
    } = block
    {
        if slide.layout == PptxLayout::Content {
            slide.layout = PptxLayout::Table;
        }
        slide.tables.push(PptxTable {
            id: id.clone(),
            caption: caption.clone(),
            headers: headers.clone(),
            alignments: alignments.clone(),
            rows: rows.clone(),
        });
    }
    if let DocumentBlock::Figure {
        src: Some(src),
        float,
        fit,
        position,
        source,
        ..
    } = block
    {
        slide.media_refs.push(MediaRef {
            source: src.clone(),
            source_file: source.as_ref().map(|range| range.source_file.clone()),
            float: normalized_float(float.as_deref()).map(str::to_string),
            fit: normalized_fit(fit.as_deref()).map(str::to_string),
            position: normalized_position(position.as_deref()).map(str::to_string),
        });
    }
}

fn collect_pptx_block_hyperlinks(slide: &mut PptxSlide, block: &DocumentBlock) {
    let DocumentBlock::Paragraph { inlines, .. } = block else {
        return;
    };
    for inline in inlines {
        let InlineNode::Link { url, .. } = inline else {
            continue;
        };
        if !is_external_hyperlink(url) || slide.hyperlinks.iter().any(|item| item.url == *url) {
            continue;
        }
        let index = slide.hyperlinks.len() + 1;
        slide.hyperlinks.push(ExportHyperlink {
            url: url.clone(),
            relationship_id: format!("rIdHyperlink{index}"),
        });
    }
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

fn render_pptx_content_types(slides: &[PptxSlide], media: &[ExportMedia]) -> String {
    let mut defaults = vec![
        (
            "rels".to_string(),
            "application/vnd.openxmlformats-package.relationships+xml".to_string(),
        ),
        ("xml".to_string(), "application/xml".to_string()),
    ];
    for item in media {
        if !defaults
            .iter()
            .any(|(extension, _)| extension == &item.extension)
        {
            defaults.push((item.extension.clone(), item.content_type.clone()));
        }
    }
    let default_xml = defaults
        .iter()
        .map(|(extension, content_type)| {
            format!(
                r#"<Default Extension="{}" ContentType="{}"/>"#,
                escape_xml(extension),
                escape_xml(content_type)
            )
        })
        .collect::<String>();
    let slide_overrides = (1..=slides.len())
        .map(|index| format!(r#"<Override PartName="/ppt/slides/slide{index}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#))
        .collect::<String>();
    let notes_overrides = slides
        .iter()
        .enumerate()
        .filter(|(_, slide)| !slide.notes.is_empty())
        .map(|(index, _)| {
            let note_index = index + 1;
            format!(r#"<Override PartName="/ppt/notesSlides/notesSlide{note_index}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"/>"#)
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">{default_xml}<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/><Override PartName="/docProps/custom.xml" ContentType="application/vnd.openxmlformats-officedocument.custom-properties+xml"/><Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>{slide_overrides}{notes_overrides}</Types>"#
    )
}

fn render_pptx_relationships(slide_count: usize) -> String {
    let relationships = (1..=slide_count)
        .map(|index| format!(r#"<Relationship Id="rId{index}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{index}.xml"/>"#))
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{relationships}</Relationships>"#
    )
}

fn render_pptx_slide_relationships(
    media: &[&ExportMedia],
    hyperlinks: &[ExportHyperlink],
    notes_slide_index: Option<usize>,
) -> String {
    let mut relationships = media
        .iter()
        .map(|item| {
            format!(
                r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../{}"/>"#,
                escape_xml(&item.relationship_id),
                escape_xml(&item.path)
            )
        })
        .collect::<Vec<_>>();
    relationships.extend(hyperlinks.iter().map(|item| {
        format!(
            r#"<Relationship Id="{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{}" TargetMode="External"/>"#,
            escape_xml(&item.relationship_id),
            escape_xml(&item.url)
        )
    }));
    if let Some(index) = notes_slide_index {
        relationships.push(format!(
            r#"<Relationship Id="rIdNotes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide{index}.xml"/>"#
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{}</Relationships>"#,
        relationships.join("")
    )
}

fn render_pptx_presentation(slide_count: usize) -> String {
    let slide_ids = (1..=slide_count)
        .map(|index| {
            let id = 255 + index;
            format!(r#"<p:sldId id="{id}" r:id="rId{index}"/>"#)
        })
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldIdLst>{slide_ids}</p:sldIdLst><p:sldSz cx="9144000" cy="5143500"/></p:presentation>"#
    )
}

fn render_pptx_slide(slide: &PptxSlide, pictures: &[PptxSlidePicture]) -> String {
    let content_shapes = render_pptx_content_shapes(slide);
    let picture_shapes = pictures
        .iter()
        .enumerate()
        .map(|(index, item)| render_pptx_picture(item, index))
        .collect::<String>();
    let tables = slide
        .tables
        .iter()
        .enumerate()
        .map(|(index, table)| render_pptx_table(table, index, slide.lines.len()))
        .collect::<String>();
    let header = if slide.header.trim().is_empty() {
        String::new()
    } else {
        format!(
            r#"<p:sp><p:nvSpPr><p:cNvPr id="97" name="Header"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="457200" y="171450"/><a:ext cx="8229600" cy="274320"/></a:xfrm></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr sz="900"/><a:t>{}</a:t></a:r></a:p></p:txBody></p:sp>"#,
            escape_xml(&slide.header)
        )
    };
    let footer = if slide.footer.trim().is_empty() {
        String::new()
    } else {
        format!(
            r#"<p:sp><p:nvSpPr><p:cNvPr id="98" name="Footer"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="457200" y="4800600"/><a:ext cx="8229600" cy="274320"/></a:xfrm></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr sz="900"/><a:t>{}</a:t></a:r></a:p></p:txBody></p:sp>"#,
            escape_xml(&slide.footer)
        )
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>{header}{content_shapes}{tables}{pictures}{footer}</p:spTree></p:cSld></p:sld>"#,
        pictures = picture_shapes
    )
}

fn render_pptx_content_shapes(slide: &PptxSlide) -> String {
    match slide.layout {
        PptxLayout::Title => {
            let title = render_pptx_text_shape(
                2,
                "Title",
                685_800,
                1_250_000,
                7_772_400,
                900_000,
                std::slice::from_ref(&slide.title),
                2800,
                &slide.hyperlinks,
            );
            let body = render_pptx_text_shape(
                3,
                "Subtitle",
                914_400,
                2_300_000,
                7_315_200,
                1_600_000,
                &slide.lines,
                1300,
                &slide.hyperlinks,
            );
            format!("{title}{body}")
        }
        PptxLayout::Section => {
            let mut lines = vec![slide.title.clone()];
            lines.extend(slide.lines.clone());
            render_pptx_text_shape(
                2,
                "Section Divider",
                685_800,
                1_600_000,
                7_772_400,
                1_700_000,
                &lines,
                2200,
                &slide.hyperlinks,
            )
        }
        PptxLayout::TwoColumn => {
            let split = slide.lines.len().div_ceil(2);
            let title = render_pptx_text_shape(
                2,
                "Title",
                571_500,
                600_000,
                8_001_000,
                520_000,
                std::slice::from_ref(&slide.title),
                1800,
                &slide.hyperlinks,
            );
            let left = render_pptx_text_shape(
                3,
                "Left Column",
                571_500,
                1_250_000,
                3_720_000,
                3_250_000,
                &slide.lines[..split],
                1100,
                &slide.hyperlinks,
            );
            let right = render_pptx_text_shape(
                4,
                "Right Column",
                4_852_500,
                1_250_000,
                3_720_000,
                3_250_000,
                &slide.lines[split..],
                1100,
                &slide.hyperlinks,
            );
            format!("{title}{left}{right}")
        }
        PptxLayout::Content | PptxLayout::Table => {
            let mut lines = vec![slide.title.clone()];
            lines.extend(slide.lines.clone());
            render_pptx_text_shape(
                2,
                if slide.layout == PptxLayout::Table {
                    "Table Slide"
                } else {
                    "Title"
                },
                571_500,
                600_000,
                8_001_000,
                3_950_000,
                &lines,
                1200,
                &slide.hyperlinks,
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_pptx_text_shape(
    id: usize,
    name: &str,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    lines: &[String],
    font_size: usize,
    hyperlinks: &[ExportHyperlink],
) -> String {
    let paragraphs = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| render_pptx_body_line_with_size(line, hyperlinks, font_size))
        .collect::<String>();
    format!(
        r#"<p:sp><p:nvSpPr><p:cNvPr id="{id}" name="{}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{width}" cy="{height}"/></a:xfrm></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/>{paragraphs}</p:txBody></p:sp>"#,
        escape_xml(name)
    )
}

fn render_pptx_body_line_with_size(
    line: &str,
    hyperlinks: &[ExportHyperlink],
    font_size: usize,
) -> String {
    let hyperlink = hyperlinks.iter().find(|item| line.contains(&item.url));
    match hyperlink {
        Some(item) => format!(
            r#"<a:p><a:r><a:rPr sz="{font_size}"><a:hlinkClick r:id="{}"/></a:rPr><a:t>{}</a:t></a:r></a:p>"#,
            escape_xml(&item.relationship_id),
            escape_xml(line)
        ),
        None => format!(
            r#"<a:p><a:r><a:rPr sz="{font_size}"/><a:t>{}</a:t></a:r></a:p>"#,
            escape_xml(line)
        ),
    }
}

fn render_pptx_table(table: &PptxTable, index: usize, body_line_count: usize) -> String {
    let shape_id = 100 + index;
    let column_count = table.headers.len().max(1);
    let row_count = table.rows.len() + 1;
    let width = 8_000_000i64;
    let height = (row_count as i64 * 280_000).clamp(560_000, 2_900_000);
    let column_width = width / column_count as i64;
    let y = 1_400_000 + (body_line_count.min(8) as i64 * 155_000) + (index as i64 * 1_250_000);
    let name = table
        .caption
        .as_deref()
        .or(table.id.as_deref())
        .unwrap_or("Table");
    let grid = (0..column_count)
        .map(|_| format!(r#"<a:gridCol w="{column_width}"/>"#))
        .collect::<String>();
    let header = render_pptx_table_row(&table.headers, &table.alignments, true);
    let rows = table
        .rows
        .iter()
        .map(|row| render_pptx_table_row(row, &table.alignments, false))
        .collect::<String>();

    format!(
        r#"<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{shape_id}" name="{}"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="571500" y="{y}"/><a:ext cx="{width}" cy="{height}"/></p:xfrm><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl><a:tblPr firstRow="1" bandRow="1"><a:tableStyleId>{{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}}</a:tableStyleId></a:tblPr><a:tblGrid>{grid}</a:tblGrid>{header}{rows}</a:tbl></a:graphicData></a:graphic></p:graphicFrame>"#,
        escape_xml(name)
    )
}

fn render_pptx_table_row(cells: &[String], alignments: &[String], header: bool) -> String {
    let height = if header { 320_000 } else { 280_000 };
    let cells = cells
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            render_pptx_table_cell(cell, alignments.get(index).map(String::as_str), header)
        })
        .collect::<String>();
    format!(r#"<a:tr h="{height}">{cells}</a:tr>"#)
}

fn render_pptx_table_cell(text: &str, alignment: Option<&str>, header: bool) -> String {
    let alignment = match alignment {
        Some("center") => "ctr",
        Some("right") => "r",
        _ => "l",
    };
    let bold_start = if header { "<a:rPr b=\"1\"/>" } else { "" };
    format!(
        r#"<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:pPr algn="{alignment}"/><a:r>{bold_start}<a:t>{}</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc>"#,
        escape_xml(text)
    )
}

fn render_pptx_notes_slide(slide: &PptxSlide) -> String {
    let title = escape_xml(&format!("Notes: {}", slide.title));
    let notes = slide
        .notes
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| format!(r#"<a:p><a:r><a:t>{}</a:t></a:r></a:p>"#, escape_xml(line)))
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="Notes"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>{title}</a:t></a:r></a:p>{notes}</p:txBody></p:sp></p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:notes>"#
    )
}

fn render_pptx_picture(picture: &PptxSlidePicture, index: usize) -> String {
    let item = picture.media;
    let shape_id = 20 + index;
    let y = 2_850_000 + (index as i64 * 1_000_000);
    let (image_width, image_height) = export_dimensions_emu_size(
        item.dimensions,
        picture.fit,
        3_657_600,
        2_057_400,
        (3_657_600, 1_371_600),
    );
    let x = pptx_aligned_x(image_width, picture.float);
    let src_rect = drawingml_source_crop(
        item.dimensions,
        image_width,
        image_height,
        picture.fit,
        picture.position,
    );
    format!(
        r#"<p:pic><p:nvPicPr><p:cNvPr id="{shape_id}" name="{}"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="{}"/>{src_rect}<a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{image_width}" cy="{image_height}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr></p:pic>"#,
        escape_xml(&item.path),
        escape_xml(&item.relationship_id)
    )
}

fn pptx_aligned_x(width: i64, float: Option<&str>) -> i64 {
    const SLIDE_WIDTH: i64 = 9_144_000;
    const SLIDE_MARGIN: i64 = 457_200;
    match normalized_float(float) {
        Some("right") => SLIDE_WIDTH - SLIDE_MARGIN - width,
        Some("center") => (SLIDE_WIDTH - width) / 2,
        _ => SLIDE_MARGIN,
    }
}

fn pptx_slide_relationship_media<'a>(
    slide: &PptxSlide,
    media: &'a [ExportMedia],
) -> Vec<&'a ExportMedia> {
    let mut slide_media = Vec::new();
    for media_ref in &slide.media_refs {
        let Some(item) = export_media_for_ref(media_ref, media) else {
            continue;
        };
        if !slide_media
            .iter()
            .any(|existing: &&ExportMedia| existing.relationship_id == item.relationship_id)
        {
            slide_media.push(item);
        }
    }
    slide_media
}

fn pptx_slide_pictures<'a>(
    slide: &'a PptxSlide,
    media: &'a [ExportMedia],
) -> Vec<PptxSlidePicture<'a>> {
    slide
        .media_refs
        .iter()
        .filter_map(|media_ref| {
            let item = export_media_for_ref(media_ref, media)?;
            Some(PptxSlidePicture {
                media: item,
                float: media_ref.float.as_deref(),
                fit: media_ref.fit.as_deref(),
                position: media_ref.position.as_deref(),
            })
        })
        .collect()
}

fn export_media_for_ref<'a>(
    media_ref: &MediaRef,
    media: &'a [ExportMedia],
) -> Option<&'a ExportMedia> {
    media
        .iter()
        .find(|item| media_ref.source == item.source && media_ref.source_file == item.source_file)
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
