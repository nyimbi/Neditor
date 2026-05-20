use super::*;

struct BundleInclude {
    source_path: String,
    bundle_path: String,
    hash: String,
    bytes: Vec<u8>,
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
    zip.start_file("paged-document.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&response.paged_document)
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
    zip.start_file("include-graph.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&manifest.include_graph)
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
