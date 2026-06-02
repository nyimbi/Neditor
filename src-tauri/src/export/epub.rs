use super::*;

pub(crate) fn render_epub_bytes(
    response: &CompileResponse,
    manifest: &ExportManifest,
) -> Result<Vec<u8>, String> {
    let media = collect_docx_media(response);
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("mimetype", stored)
        .map_err(|err| err.to_string())?;
    zip.write_all(b"application/epub+zip")
        .map_err(|err| err.to_string())?;

    zip.start_file("META-INF/container.xml", deflated)
        .map_err(|err| err.to_string())?;
    zip.write_all(epub_container_xml().as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("OEBPS/content.opf", deflated)
        .map_err(|err| err.to_string())?;
    zip.write_all(epub_content_opf(response, manifest, &media).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("OEBPS/nav.xhtml", deflated)
        .map_err(|err| err.to_string())?;
    zip.write_all(epub_nav_xhtml(response, manifest).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("OEBPS/document.xhtml", deflated)
        .map_err(|err| err.to_string())?;
    zip.write_all(epub_document_xhtml(response, manifest, &media).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("OEBPS/styles/neditor.css", deflated)
        .map_err(|err| err.to_string())?;
    zip.write_all(epub_css().as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("OEBPS/metadata/manifest.json", deflated)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(manifest)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;

    zip.start_file("OEBPS/metadata/document.txt", deflated)
        .map_err(|err| err.to_string())?;
    zip.write_all(export_text(response, &manifest.export_options).as_bytes())
        .map_err(|err| err.to_string())?;

    for item in &media {
        zip.start_file(
            format!("OEBPS/assets/{}", safe_bundle_path(&item.source)),
            deflated,
        )
        .map_err(|err| err.to_string())?;
        zip.write_all(&item.bytes).map_err(|err| err.to_string())?;
    }

    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}

fn epub_container_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#
}

fn epub_content_opf(
    response: &CompileResponse,
    manifest: &ExportManifest,
    media: &[ExportMedia],
) -> String {
    let identifier = format!("urn:sha256:{}", manifest.source_hash.replace("sha256:", ""));
    let author = metadata_string(&response.metadata, "author")
        .or_else(|| metadata_string(&response.metadata, "approvedBy"))
        .unwrap_or_else(|| "NEditor".to_string());
    let language = epub_language(response, manifest);
    let media_items = media
        .iter()
        .enumerate()
        .map(|(index, item)| {
            format!(
                r#"<item id="asset-{}" href="assets/{}" media-type="{}"/>"#,
                index + 1,
                escape_xml(&safe_bundle_path(&item.source)),
                escape_xml(&item.content_type)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="pub-id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="pub-id">{}</dc:identifier><dc:title>{}</dc:title><dc:creator>{}</dc:creator><dc:language>{}</dc:language><dc:publisher>NEditor</dc:publisher><meta property="dcterms:modified">{}</meta><meta property="neditor:status">{}</meta><meta property="neditor:sourceHash">{}</meta></metadata><manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="doc" href="document.xhtml" media-type="application/xhtml+xml"/><item id="css" href="styles/neditor.css" media-type="text/css"/><item id="manifest-json" href="metadata/manifest.json" media-type="application/json"/><item id="document-text" href="metadata/document.txt" media-type="text/plain"/>{}</manifest><spine><itemref idref="doc"/></spine></package>"#,
        escape_xml(&identifier),
        escape_xml(&response.semantic.title),
        escape_xml(&author),
        escape_xml(&language),
        escape_xml(&Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        escape_xml(&response.semantic.status),
        escape_xml(&manifest.source_hash),
        media_items
    )
}

fn epub_nav_xhtml(response: &CompileResponse, manifest: &ExportManifest) -> String {
    let language = epub_language(response, manifest);
    let items = response
        .semantic
        .headings
        .iter()
        .filter(|heading| heading.level <= 3)
        .map(|heading| {
            format!(
                r#"<li class="level-{}"><a href="document.xhtml#{}">{}</a></li>"#,
                heading.level,
                escape_xml(&heading.anchor),
                escape_xml(&heading.text)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="{}"><head><title>{}</title><link rel="stylesheet" type="text/css" href="styles/neditor.css"/></head><body><nav epub:type="toc" id="toc"><h1>Table of contents</h1><ol>{}</ol></nav></body></html>"#,
        escape_xml(&language),
        escape_xml(&response.semantic.title),
        items
    )
}

fn epub_document_xhtml(
    response: &CompileResponse,
    manifest: &ExportManifest,
    media: &[ExportMedia],
) -> String {
    let body_html = epub_body_html(response, manifest, media);
    let language = epub_language(response, manifest);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="{}"><head><title>{}</title><link rel="stylesheet" type="text/css" href="styles/neditor.css"/></head><body><section epub:type="titlepage" class="cover"><h1>{}</h1><p>Status: {}</p><p>Source hash: {}</p></section><main>{}</main></body></html>"#,
        escape_xml(&language),
        escape_xml(&response.semantic.title),
        escape_xml(&response.semantic.title),
        escape_xml(&response.semantic.status),
        escape_xml(&manifest.source_hash),
        body_html
    )
}

fn epub_language(response: &CompileResponse, manifest: &ExportManifest) -> String {
    manifest
        .export_options
        .get("language")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            manifest
                .export_options
                .get("htmlLanguage")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| metadata_string(&response.metadata, "language"))
        .or_else(|| metadata_string(&response.metadata, "lang"))
        .unwrap_or_else(|| "en".to_string())
}

fn epub_body_html(
    response: &CompileResponse,
    manifest: &ExportManifest,
    media: &[ExportMedia],
) -> String {
    let mut body = format!(
        "{}{}",
        response.html,
        html_appendix_sections(response, &manifest.export_options)
    );
    for item in media {
        body = body.replace(
            &format!(r#"src="{}""#, escape_html(&item.source)),
            &format!(
                r#"src="assets/{}""#,
                escape_xml(&safe_bundle_path(&item.source))
            ),
        );
    }
    body
}

fn epub_css() -> &'static str {
    "body{font-family:serif;line-height:1.55;margin:1.2em;color:#111}h1,h2,h3{font-family:sans-serif;line-height:1.2}table{border-collapse:collapse;width:100%;margin:1em 0}td,th{border:1px solid #999;padding:.35em}pre{white-space:pre-wrap;background:#f4f4f4;padding:.75em}.cover{border-bottom:1px solid #ddd;margin-bottom:1.5em}.level-2{margin-left:1em}.level-3{margin-left:2em}.equation{margin:1em 0}"
}
