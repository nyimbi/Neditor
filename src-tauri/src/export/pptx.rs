use super::*;

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

fn collect_pptx_media(response: &CompileResponse) -> Vec<ExportMedia> {
    collect_docx_media(response)
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
    header_cells: Vec<TableCell>,
    rows: Vec<Vec<String>>,
    row_cells: Vec<Vec<TableCell>>,
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
    let start = table
        .rows
        .windows(rows.len().max(1))
        .position(|window| window == rows.as_slice())
        .unwrap_or(0);
    let row_cells = table
        .row_cells
        .get(start..start + rows.len())
        .map(|slice| slice.to_vec())
        .unwrap_or_else(|| {
            rows.iter()
                .map(|row| plain_export_table_cells(row))
                .collect()
        });
    PptxTable {
        id: table.id.clone(),
        caption,
        headers: table.headers.clone(),
        alignments: table.alignments.clone(),
        header_cells: table.header_cells.clone(),
        rows,
        row_cells,
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
                header_cells: table.header_cells,
                rows: table.rows,
                row_cells: table.row_cells,
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
                header_cells: table.header_cells,
                rows: table.rows,
                row_cells: table.row_cells,
            });
        }
    }
    if let DocumentBlock::Table {
        id,
        caption,
        headers,
        alignments,
        header_cells,
        rows,
        row_cells,
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
            header_cells: populated_table_cells(header_cells, headers),
            rows: rows.clone(),
            row_cells: populated_table_row_cells(row_cells, rows),
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
    let header_cells = populated_table_cells(&table.header_cells, &table.headers);
    let header = render_pptx_table_row(&header_cells, &table.alignments, true);
    let row_cells = populated_table_row_cells(&table.row_cells, &table.rows);
    let rows = row_cells
        .iter()
        .map(|row| render_pptx_table_row(row, &table.alignments, false))
        .collect::<String>();

    format!(
        r#"<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{shape_id}" name="{}"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="571500" y="{y}"/><a:ext cx="{width}" cy="{height}"/></p:xfrm><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl><a:tblPr firstRow="1" bandRow="1"><a:tableStyleId>{{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}}</a:tableStyleId></a:tblPr><a:tblGrid>{grid}</a:tblGrid>{header}{rows}</a:tbl></a:graphicData></a:graphic></p:graphicFrame>"#,
        escape_xml(name)
    )
}

fn render_pptx_table_row(cells: &[TableCell], alignments: &[String], header: bool) -> String {
    let height = if header { 320_000 } else { 280_000 };
    let cells = cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            render_pptx_table_cell(cell, alignments.get(index).map(String::as_str), header)
        })
        .collect::<String>();
    format!(r#"<a:tr h="{height}">{cells}</a:tr>"#)
}

fn render_pptx_table_cell(
    cell: &TableCell,
    alignment: Option<&str>,
    header: bool,
) -> Option<String> {
    if cell.covered && !cell.continues_rowspan {
        return None;
    }
    let alignment = match alignment {
        Some("center") => "ctr",
        Some("right") => "r",
        _ => "l",
    };
    let bold_start = if header { "<a:rPr b=\"1\"/>" } else { "" };
    let grid_span = if cell.colspan > 1 {
        format!(r#" gridSpan="{}""#, cell.colspan)
    } else {
        String::new()
    };
    let row_span = if cell.rowspan > 1 {
        format!(r#" rowSpan="{}""#, cell.rowspan)
    } else {
        String::new()
    };
    let merge = if cell.covered && cell.continues_rowspan {
        r#" vMerge="1""#
    } else {
        ""
    };
    Some(format!(
        r#"<a:tc{grid_span}{row_span}{merge}><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:pPr algn="{alignment}"/><a:r>{bold_start}<a:t>{}</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc>"#,
        escape_xml(&cell.text)
    ))
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
