use super::*;

pub(crate) fn render_blog_publish_package_bytes(
    response: &CompileResponse,
    manifest: &ExportManifest,
) -> Result<Vec<u8>, String> {
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let slug = publish_slug(response);
    let publish_metadata = publish_metadata(response, manifest, &slug);

    zip.start_file("post.md", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(response.compiled_markdown.as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("post.html", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_blog_html(response, manifest).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("substack-copy.html", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_substack_copy_html(response).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("post.txt", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(export_text(response, &manifest.export_options).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("metadata.json", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(
        serde_json::to_string_pretty(&publish_metadata)
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

    zip.start_file("rss-item.xml", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_rss_item(response, &slug).as_bytes())
        .map_err(|err| err.to_string())?;

    zip.start_file("README.md", options)
        .map_err(|err| err.to_string())?;
    zip.write_all(render_publish_readme(response, manifest).as_bytes())
        .map_err(|err| err.to_string())?;

    for item in collect_docx_media(response) {
        zip.start_file(format!("assets/{}", safe_bundle_path(&item.path)), options)
            .map_err(|err| err.to_string())?;
        zip.write_all(&item.bytes).map_err(|err| err.to_string())?;
    }

    zip.finish().map_err(|err| err.to_string())?;
    Ok(cursor.into_inner())
}

fn publish_metadata(response: &CompileResponse, manifest: &ExportManifest, slug: &str) -> Value {
    json!({
        "title": response.semantic.title.clone(),
        "subtitle": metadata_string(&response.metadata, "subtitle"),
        "slug": slug,
        "author": metadata_string(&response.metadata, "author")
            .or_else(|| metadata_string(&response.metadata, "approvedBy")),
        "date": metadata_string(&response.metadata, "date")
            .or_else(|| metadata_string(&response.metadata, "publishedAt"))
            .or_else(|| metadata_string(&response.metadata, "approvedAt")),
        "status": response.semantic.status.clone(),
        "canonicalUrl": metadata_string(&response.metadata, "canonicalUrl")
            .or_else(|| metadata_string(&response.metadata, "canonical_url")),
        "tags": publish_tags(response),
        "audience": target_persona_summary(&response.metadata),
        "sourceHash": manifest.source_hash.clone(),
        "appVersion": manifest.app_version.clone(),
        "exportTarget": manifest.export_target.clone(),
        "readiness": manifest.readiness.clone(),
        "packageType": "publishing-handoff",
        "primaryPublishFile": primary_publish_file(&manifest.export_target),
        "fallbackFiles": ["post.md", "post.txt"],
        "publishingSteps": publishing_steps(&manifest.export_target),
    })
}

fn render_blog_html(response: &CompileResponse, manifest: &ExportManifest) -> String {
    let subtitle = metadata_string(&response.metadata, "subtitle")
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("<p class=\"subtitle\">{}</p>", escape_html(&value)))
        .unwrap_or_default();
    let author = metadata_string(&response.metadata, "author")
        .map(|value| format!("<span>{}</span>", escape_html(&value)))
        .unwrap_or_default();
    let date = metadata_string(&response.metadata, "date")
        .map(|value| format!("<span>{}</span>", escape_html(&value)))
        .unwrap_or_default();
    let tags = publish_tags(response)
        .into_iter()
        .map(|tag| format!("<span>{}</span>", escape_html(&tag)))
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{}</title><meta name=\"generator\" content=\"NEditor\"><meta name=\"neditor-source-hash\" content=\"{}\"><style>{}</style></head><body><article><header><h1>{}</h1>{}<p class=\"byline\">{}{}</p><p class=\"tags\">{}</p></header>{}</article></body></html>",
        escape_html(&response.semantic.title),
        escape_html(&manifest.source_hash),
        blog_css(),
        escape_html(&response.semantic.title),
        subtitle,
        author,
        date,
        tags,
        response.html
    )
}

fn render_substack_copy_html(response: &CompileResponse) -> String {
    let subtitle = metadata_string(&response.metadata, "subtitle")
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("<p>{}</p>", escape_html(&value)))
        .unwrap_or_default();
    format!(
        "<article><h1>{}</h1>{}{}</article>",
        escape_html(&response.semantic.title),
        subtitle,
        response.html
    )
}

fn render_rss_item(response: &CompileResponse, slug: &str) -> String {
    let canonical = metadata_string(&response.metadata, "canonicalUrl")
        .or_else(|| metadata_string(&response.metadata, "canonical_url"))
        .unwrap_or_else(|| slug.to_string());
    let description = metadata_string(&response.metadata, "subtitle")
        .unwrap_or_else(|| response.semantic.title.clone());
    format!(
        "<item><title>{}</title><link>{}</link><guid>{}</guid><description>{}</description></item>",
        escape_xml(&response.semantic.title),
        escape_xml(&canonical),
        escape_xml(&canonical),
        escape_xml(&description)
    )
}

fn render_publish_readme(response: &CompileResponse, manifest: &ExportManifest) -> String {
    format!(
        "# {}\n\nThis NEditor publish package contains copy-ready blog artifacts.\n\n- `post.md`: compiled Markdown source.\n- `post.html`: standalone blog HTML preview and static-site publishing source.\n- `substack-copy.html`: minimal HTML fragment for browser copy/paste into Substack's editor.\n- `post.txt`: plain-text fallback.\n- `metadata.json`: title, slug, author, tags, status, readiness, and publish workflow metadata.\n- `rss-item.xml`: feed item seed for static blog generators.\n- `manifest.json`: NEditor export audit manifest.\n\n## Publish Workflow\n\n1. Review `metadata.json` and confirm readiness is true.\n2. Use `{}` as the primary publish file for this target.\n3. Keep `manifest.json` with the published record for auditability.\n\nExport target: `{}`\nSource hash: `{}`\n",
        response.semantic.title,
        primary_publish_file(&manifest.export_target),
        manifest.export_target,
        manifest.source_hash
    )
}

fn primary_publish_file(target: &str) -> &'static str {
    if target == "substack" {
        "substack-copy.html"
    } else {
        "post.html"
    }
}

fn publishing_steps(target: &str) -> Vec<&'static str> {
    if target == "substack" {
        vec![
            "Open substack-copy.html in a browser or editor.",
            "Copy the article fragment into Substack's editor.",
            "Use metadata.json for title, slug, tags, and audit status.",
            "Retain manifest.json with the published record.",
        ]
    } else {
        vec![
            "Use post.html for static-site or CMS publishing.",
            "Use post.md when the destination accepts Markdown.",
            "Use rss-item.xml as a feed item seed when publishing through a static blog generator.",
            "Retain manifest.json with the published record.",
        ]
    }
}

fn publish_tags(response: &CompileResponse) -> Vec<String> {
    let mut tags = metadata_string_list(&response.metadata, "tags");
    if tags.is_empty() {
        tags = metadata_string_list(&response.metadata, "keywords");
    }
    tags.into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn publish_slug(response: &CompileResponse) -> String {
    metadata_string(&response.metadata, "slug")
        .filter(|value| !value.trim().is_empty())
        .map(|value| slugify(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| slugify(&response.semantic.title))
}

fn slugify(value: &str) -> String {
    let mut output = String::new();
    let mut previous_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
            previous_dash = false;
        } else if !previous_dash && !output.is_empty() {
            output.push('-');
            previous_dash = true;
        }
    }
    output.trim_matches('-').to_string()
}

fn blog_css() -> &'static str {
    "body{font-family:Inter,Arial,sans-serif;line-height:1.65;margin:0;color:#17202a;background:#fff}article{max-width:760px;margin:0 auto;padding:48px 24px}h1,h2,h3{line-height:1.2}pre{overflow:auto;padding:16px;background:#f4f6f8}code{font-family:Menlo,Consolas,monospace}.subtitle{font-size:1.15rem;color:#4a5565}.byline,.tags{display:flex;gap:12px;color:#5d6978}"
}
