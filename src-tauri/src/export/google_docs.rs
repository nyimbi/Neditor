use super::*;

pub(crate) fn render_google_docs_package_bytes(
    response: &CompileResponse,
    manifest: &ExportManifest,
) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("document.docx", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(&render_docx_bytes(response, &manifest.export_options)?)
        .map_err(|err| err.to_string())?;

    zip.start_file("document.html", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_full_html(response, &manifest.export_options).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("document.md", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(response.compiled_markdown.as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("document.txt", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(export_text(response, &manifest.export_options).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("metadata.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&google_docs_metadata(response, manifest))
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;

    zip.start_file("manifest.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(manifest)
            .map_err(|err| err.to_string())?
            .as_bytes(),
    )
    .map_err(|err| err.to_string())?;

    zip.start_file("README.md", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_google_docs_readme(response, manifest).as_bytes())
        .map_err(|err| err.to_string())?;

    for item in collect_docx_media(response) {
        zip.start_file(format!("assets/{}", safe_bundle_path(&item.path)), options)
            .map_err(|err| err.to_string())?;
        zip.write_all(&item.bytes).map_err(|err| err.to_string())?;
    }

    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}

fn google_docs_metadata(response: &CompileResponse, manifest: &ExportManifest) -> Value {
    json!({
        "title": response.semantic.title.clone(),
        "subtitle": metadata_string(&response.metadata, "subtitle"),
        "author": metadata_string(&response.metadata, "author")
            .or_else(|| metadata_string(&response.metadata, "approvedBy")),
        "date": metadata_string(&response.metadata, "date")
            .or_else(|| metadata_string(&response.metadata, "approvedAt")),
        "status": response.semantic.status.clone(),
        "version": metadata_string(&response.metadata, "version"),
        "sourceHash": manifest.source_hash.clone(),
        "appVersion": manifest.app_version.clone(),
        "exportTarget": manifest.export_target.clone(),
        "readiness": manifest.readiness.clone(),
        "importHint": "Upload document.docx to Google Docs, or unzip and use document.html/document.md as fallback sources."
    })
}

fn render_google_docs_readme(response: &CompileResponse, manifest: &ExportManifest) -> String {
    format!(
        "# {}\n\nThis NEditor Google Docs package is a local-first import handoff.\n\n- `document.docx`: primary file to upload or convert in Google Docs.\n- `document.html`: standalone HTML fallback for copy/import workflows.\n- `document.md`: compiled Markdown source.\n- `document.txt`: plain-text fallback.\n- `metadata.json`: title, author, status, version, readiness, and source hash metadata.\n- `manifest.json`: NEditor export audit manifest.\n- `assets/`: embedded media extracted for audit when present.\n\nExport target: `{}`\nSource hash: `{}`\n",
        response.semantic.title,
        manifest.export_target,
        manifest.source_hash
    )
}
