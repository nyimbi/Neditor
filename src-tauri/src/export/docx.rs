use super::*;

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
struct DocxPageLayout {
    page_size: String,
    orientation: String,
    width: u32,
    height: u32,
    margin_top: u32,
    margin_right: u32,
    margin_bottom: u32,
    margin_left: u32,
}

#[derive(Clone, Debug)]
struct DocxSectionProperties {
    header_relationship_id: String,
    footer_relationship_id: String,
    columns: Option<usize>,
    page_size: Option<String>,
    orientation: Option<String>,
    margins: Option<String>,
}

impl Default for DocxSectionProperties {
    fn default() -> Self {
        Self {
            header_relationship_id: "rIdHeader1".to_string(),
            footer_relationship_id: "rIdFooter1".to_string(),
            columns: None,
            page_size: None,
            orientation: None,
            margins: None,
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

    fn apply_layout(&mut self, settings: &LayoutSettings) {
        self.columns = settings.columns;
        if settings.page_size.is_some() {
            self.page_size = settings.page_size.clone();
        }
        if settings.orientation.is_some() {
            self.orientation = settings.orientation.clone();
        }
        if settings.margins.is_some() {
            self.margins = settings.margins.clone();
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
        || settings.has_page_model_controls()
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
                body.push_str(&docx_section_break(&current_section, &page_layout));
                if let Some(section_override) = section_overrides.get(section_index) {
                    current_section.apply_override(section_override);
                }
                current_section.apply_layout(settings);
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
    let final_section = docx_section_properties(&current_section, &page_layout);
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
                output.push_str(&docx_table(
                    &table.header_cells,
                    &table.alignments,
                    &table.row_cells,
                ));
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
            header_cells,
            rows,
            row_cells,
            ..
        } => {
            let mut output = String::new();
            if id.is_some() || caption.is_some() {
                output.push_str(&docx_bookmarked_paragraph(
                    &table_export_line(id, caption, headers),
                    id.as_deref(),
                ));
            }
            let header_cells = populated_table_cells(header_cells, headers);
            let row_cells = populated_table_row_cells(row_cells, rows);
            output.push_str(&docx_table(&header_cells, alignments, &row_cells));
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
                output.push_str(&docx_table(
                    &table.header_cells,
                    &table.alignments,
                    &table.row_cells,
                ));
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

fn docx_section_break(section: &DocxSectionProperties, page_layout: &DocxPageLayout) -> String {
    format!(
        r#"<w:p><w:pPr>{}</w:pPr></w:p>"#,
        docx_section_properties(section, page_layout)
    )
}

fn docx_section_properties(
    section: &DocxSectionProperties,
    page_layout: &DocxPageLayout,
) -> String {
    let page_layout = section.resolve_page_layout(page_layout);
    let orientation_attr = if page_layout.orientation == "landscape" {
        r#" w:orient="landscape""#
    } else {
        ""
    };
    let columns = section
        .columns
        .map(|columns| format!(r#"<w:cols w:num="{columns}" w:space="720"/>"#))
        .unwrap_or_default();
    format!(
        r#"<w:sectPr><w:headerReference w:type="default" r:id="{}"/><w:footerReference w:type="default" r:id="{}"/><w:pgSz w:w="{page_width}" w:h="{page_height}"{orientation_attr}/><w:pgMar w:top="{margin_top}" w:right="{margin_right}" w:bottom="{margin_bottom}" w:left="{margin_left}"/>{columns}</w:sectPr>"#,
        escape_xml(&section.header_relationship_id),
        escape_xml(&section.footer_relationship_id),
        page_width = page_layout.width,
        page_height = page_layout.height,
        margin_top = page_layout.margin_top,
        margin_right = page_layout.margin_right,
        margin_bottom = page_layout.margin_bottom,
        margin_left = page_layout.margin_left
    )
}

fn docx_page_layout(response: &CompileResponse, options: &Value) -> DocxPageLayout {
    let page_size = layout_page_size(&response.metadata);
    let orientation = layout_orientation(&response.metadata).to_string();
    let (width, height) = docx_page_dimensions(&page_size, &orientation);
    let margin = match explicit_layout_margins(&response.metadata).as_deref() {
        Some(margins) => docx_margin_for_preset(margins),
        None => match layout_preset(options) {
            "compact" => 1080,
            "presentation" => 1200,
            _ => 1440,
        },
    };
    DocxPageLayout {
        page_size,
        orientation,
        width,
        height,
        margin_top: margin,
        margin_right: margin,
        margin_bottom: margin,
        margin_left: margin,
    }
}

impl DocxSectionProperties {
    fn resolve_page_layout(&self, base: &DocxPageLayout) -> DocxPageLayout {
        let page_size = self
            .page_size
            .clone()
            .unwrap_or_else(|| base.page_size.clone());
        let orientation = self
            .orientation
            .clone()
            .unwrap_or_else(|| base.orientation.clone());
        let (width, height) = docx_page_dimensions(&page_size, &orientation);
        let margin = self.margins.as_deref().map(docx_margin_for_preset);
        DocxPageLayout {
            page_size,
            orientation,
            width,
            height,
            margin_top: margin.unwrap_or(base.margin_top),
            margin_right: margin.unwrap_or(base.margin_right),
            margin_bottom: margin.unwrap_or(base.margin_bottom),
            margin_left: margin.unwrap_or(base.margin_left),
        }
    }
}

fn docx_page_dimensions(page_size: &str, orientation: &str) -> (u32, u32) {
    let (mut width, mut height) = match page_size {
        "letter" => (12240, 15840),
        "legal" => (12240, 20160),
        _ => (11906, 16838),
    };
    if orientation == "landscape" {
        std::mem::swap(&mut width, &mut height);
    }
    (width, height)
}

fn docx_margin_for_preset(margins: &str) -> u32 {
    match margins {
        "narrow" | "compact" => 720,
        "wide" => 1800,
        "normal" => 1440,
        _ => 1440,
    }
}

fn docx_table(headers: &[TableCell], alignments: &[String], rows: &[Vec<TableCell>]) -> String {
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

fn docx_table_row(cells: &[TableCell], alignments: &[String]) -> String {
    let cells = cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| docx_cell(cell, alignments.get(index).map(String::as_str)))
        .collect::<String>();
    format!("<w:tr>{cells}</w:tr>")
}

fn docx_cell(cell: &TableCell, alignment: Option<&str>) -> Option<String> {
    if cell.covered && !cell.continues_rowspan {
        return None;
    }
    let alignment = match alignment {
        Some("center") => r#"<w:pPr><w:jc w:val="center"/></w:pPr>"#,
        Some("right") => r#"<w:pPr><w:jc w:val="right"/></w:pPr>"#,
        _ => "",
    };
    let grid_span = if cell.colspan > 1 {
        format!(r#"<w:gridSpan w:val="{}"/>"#, cell.colspan)
    } else {
        String::new()
    };
    let vmerge = if cell.covered && cell.continues_rowspan {
        r#"<w:vMerge/>"#.to_string()
    } else if cell.rowspan > 1 {
        r#"<w:vMerge w:val="restart"/>"#.to_string()
    } else {
        String::new()
    };
    Some(format!(
        r#"<w:tc><w:tcPr><w:tcW w:w="2400" w:type="dxa"/>{grid_span}{vmerge}</w:tcPr><w:p>{alignment}<w:r><w:t>{}</w:t></w:r></w:p></w:tc>"#,
        escape_xml(&cell.text)
    ))
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
