use chrono::Utc;
use serde_json::{json, Value};
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::Path;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env,
    path::PathBuf,
};

mod ai_cleanup;
mod bibliography;
mod calculations;
mod compile_options;
mod compiler_types;
mod diagnostics;
mod document_ast;
mod export;
mod export_commands;
mod export_media;
mod filesystem;
mod footnotes;
mod front_matter;
mod generated_sections;
mod git;
mod html_preview;
mod indexing;
mod layout;
mod link_validation;
mod manifest;
mod markdown_tables;
mod provenance;
mod references;
mod review;
mod rich_blocks;
mod snapshot;
mod source_mapping;
mod tables;
mod transforms;
mod utils;
mod validation;
mod variables;

use ai_cleanup::cleanup_ai_paste;
#[cfg(test)]
use ai_cleanup::AiCleanupRequest;
use bibliography::{
    citation_keys_from_references, collect_bibliography, collect_citation_references,
    duplicate_bibliography_keys, parse_bibliography_source, render_citations,
};
use calculations::{collect_calculations, formula_dependency_edges};
use compile_options::apply_compile_options;
use compiler_types::{
    CompileRequest, CompileResponse, CompileWithOptionsRequest, ExportManifest, Heading,
    IncludeEdge, SemanticDocument, SourceMapEntry,
};
use diagnostics::{diag, DocumentDiagnostic};
#[cfg(test)]
use document_ast::DocumentBlock;
use document_ast::{
    attach_source_ranges, attach_transform_artifacts, build_document_ast, extract_label, slugify,
    AstDocumentMetadata,
};
#[cfg(test)]
use export::{
    render_docx_bytes, render_full_html, render_markdown_bundle_bytes, render_pdf_bytes,
    render_pptx_bytes,
};
use export_commands::{export_document, prepare_for_export};
#[cfg(test)]
use export_commands::{ExportRequest, PrepareExportRequest};
#[cfg(all(test, feature = "native-watch"))]
use filesystem::notify_event_should_emit;
use filesystem::{
    duplicate_file, file_metadata, list_workspace_files, open_file, read_file, rename_file,
    reveal_path, save_file, save_file_as, start_file_watcher, stop_file_watcher, watch_file,
    FileResponse, FileWatcherState,
};
#[cfg(test)]
use filesystem::{
    DuplicateFileRequest, RenameFileRequest, SaveFileRequest, WatchFileRequest,
    WorkspaceFileRequest,
};
use footnotes::render_footnotes;
use front_matter::{merge_project_variables, parse_front_matter, render_front_matter_data_sources};
use generated_sections::inject_generated_sections;
use git::{
    commit_document_changes, get_git_status, git_diff, git_history, restore_git_revision,
    tag_release,
};
#[cfg(test)]
use git::{run_git, GitCommitRequest, GitPathRequest, GitRestoreRequest, GitTagRequest};
use html_preview::markdown_to_html;
use indexing::{collect_index_entries, strip_index_markers};
use link_validation::{validate_image_paths, validate_link_paths, validate_logo_path};
use manifest::{count_equations, count_figures, manifest_file, manifest_media_files};
use provenance::{collect_ai_assisted_sections, collect_ai_sources};
use references::{collect_cross_references, collect_labels, render_cross_references};
use review::{collect_change_notes, collect_comments};
use rich_blocks::{
    render_callouts, render_equations, render_figures, render_layout_block_html,
    render_layout_tokens,
};
use snapshot::{create_snapshot, list_snapshots, restore_snapshot};
use source_mapping::{
    ast_source_range_for_generated_lines, expand_includes, normalize_source_map_after_front_matter,
};
use tables::{collect_table_summaries, evaluate_markdown_table_formulas, render_delimited_table};
use transforms::external::ExternalTransformRequest;
use transforms::options::TransformExecutionOptions;
use transforms::{
    chart::render_chart_svg,
    external::{list_transform_engines, run_external_transform},
    transform_cache_key, TransformArtifact,
};
pub(crate) use utils::{
    escape_css, escape_html, escape_pdf, escape_xml, format_value, metadata_lookup,
    metadata_string, path_to_string, render_export_template, sha256_hex, sha256_uri,
    value_to_string,
};
use validation::{validate_document, DocumentValidationInput};
use variables::interpolate_variables;

#[tauri::command]
fn compile_document(request: CompileRequest) -> Result<CompileResponse, String> {
    Ok(compile(request))
}

#[tauri::command]
fn compile_document_with_options(
    request: CompileWithOptionsRequest,
) -> Result<CompileResponse, String> {
    Ok(compile_with_options(
        CompileRequest {
            text: request.text,
            file_path: request.file_path,
        },
        &request.options,
    ))
}

#[tauri::command]
fn run_transform(name: String, body: String) -> Result<TransformArtifact, String> {
    if !supported_transform(&name) {
        return Err(format!("Unknown transform: {name}"));
    }
    let mut diagnostics = Vec::new();
    Ok(render_transform(
        &name,
        &body,
        &json!({}),
        &TransformExecutionOptions::default(),
        &mut diagnostics,
    ))
}

fn compile(request: CompileRequest) -> CompileResponse {
    compile_inner(request, None)
}

pub(crate) fn compile_with_options(request: CompileRequest, options: &Value) -> CompileResponse {
    compile_inner(request, Some(options))
}

fn compile_inner(request: CompileRequest, options: Option<&Value>) -> CompileResponse {
    let mut diagnostics = Vec::new();
    let mut include_graph = Vec::new();
    let root_path = request.file_path.as_deref().map(PathBuf::from);
    let root_file = root_path
        .as_ref()
        .map(|path| path_to_string(path.as_path()))
        .unwrap_or_else(|| "untitled.md".to_string());
    let mut visited = HashSet::new();
    let mut source_map = Vec::new();
    let mut generated_line_count = 0usize;
    let source = expand_includes(
        &request.text,
        root_path.as_deref(),
        &root_file,
        0,
        &mut visited,
        &mut include_graph,
        &mut source_map,
        &mut generated_line_count,
        &mut diagnostics,
    );
    let (mut metadata, body, body_start_line) =
        parse_front_matter(&source, &mut diagnostics, Some(root_file.clone()));
    merge_project_variables(&mut metadata, root_path.as_deref(), &mut diagnostics);
    apply_compile_options(&mut metadata, options);
    let mut body = body;
    let data_source_markdown = render_front_matter_data_sources(
        &metadata,
        root_path.as_deref(),
        &root_file,
        &mut include_graph,
        &mut diagnostics,
    );
    if !data_source_markdown.is_empty() {
        body.push_str("\n\n");
        body.push_str(&data_source_markdown);
    }
    normalize_source_map_after_front_matter(&mut source_map, body_start_line);
    let mut calculation_context = HashMap::new();
    let formula_graph = collect_calculations(&body, &mut calculation_context, &mut diagnostics);
    let formula_edges = formula_dependency_edges(&formula_graph);
    let interpolated = interpolate_variables(
        &body,
        &metadata,
        &calculation_context,
        &source_map,
        &mut diagnostics,
    );
    let headings = extract_headings(&interpolated);
    let bibliography = collect_bibliography(
        &interpolated,
        &metadata,
        root_path.as_deref(),
        &mut diagnostics,
    );
    let duplicate_bibliography_keys = duplicate_bibliography_keys(&bibliography);
    let glossary = collect_glossary(&interpolated);
    let citation_references = collect_citation_references(&interpolated);
    let citations = citation_keys_from_references(&citation_references);
    let semantic_heading_anchors = headings
        .iter()
        .map(|heading| heading.anchor.as_str())
        .collect::<Vec<_>>();
    let labels = collect_labels(&interpolated, &semantic_heading_anchors);
    let cross_references =
        collect_cross_references(&interpolated, &labels, &source_map, &mut diagnostics);
    let reference_markdown = render_cross_references(&interpolated, &cross_references);
    let index_entries = collect_index_entries(&interpolated, &metadata, &headings, &glossary);
    let index_terms = index_entries
        .iter()
        .map(|entry| entry.term.clone())
        .collect::<Vec<_>>();
    let layout_directives = collect_fence_bodies(&interpolated, "layout");
    let comments = collect_comments(&interpolated);
    let change_notes = collect_change_notes(&interpolated);
    let ai_sources = collect_ai_sources(&interpolated);
    let ai_assisted_sections = collect_ai_assisted_sections(&interpolated, &headings);
    let with_toc = inject_generated_sections(
        &reference_markdown,
        &metadata,
        &headings,
        &index_entries,
        &bibliography,
    );
    let index_marker_markdown = strip_index_markers(&with_toc);
    let transform_options = TransformExecutionOptions::from_compile_options(options);
    let (transformed_markdown, transform_artifacts) =
        apply_transforms(&index_marker_markdown, &transform_options, &mut diagnostics);
    let citation_markdown = render_citations(
        &transformed_markdown,
        &bibliography,
        citation_style(&metadata),
    );
    let table_formula_markdown =
        evaluate_markdown_table_formulas(&citation_markdown, &mut diagnostics);
    validate_image_paths(
        &table_formula_markdown,
        root_path.as_deref(),
        &source_map,
        &mut diagnostics,
    );
    validate_logo_path(&metadata, root_path.as_deref(), &mut diagnostics);
    validate_link_paths(
        &table_formula_markdown,
        root_path.as_deref(),
        &source_map,
        &mut diagnostics,
    );
    let footnote_markdown = render_footnotes(&table_formula_markdown);
    let figure_markdown = render_figures(&footnote_markdown);
    let equation_markdown = render_equations(&figure_markdown);
    let callout_markdown = render_callouts(&equation_markdown);
    let layout_markdown = render_layout_tokens(&callout_markdown);
    let mut document_ast = build_document_ast(&layout_markdown);
    attach_source_ranges(&mut document_ast, |line, end_line| {
        ast_source_range_for_generated_lines(&source_map, line, end_line)
    });
    attach_transform_artifacts(&mut document_ast, &transform_artifacts);
    let preview_headings = extract_headings(&layout_markdown);
    let heading_anchors = preview_headings
        .iter()
        .map(|heading| heading.anchor.as_str())
        .collect::<Vec<_>>();
    let html = markdown_to_html(&layout_markdown, &heading_anchors, &glossary);
    let title = metadata
        .get("title")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| headings.first().map(|heading| heading.text.clone()))
        .unwrap_or_else(|| "Untitled Document".to_string());
    let status = metadata
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("draft")
        .to_string();
    document_ast.metadata = AstDocumentMetadata {
        title: title.clone(),
        status: status.clone(),
        version: metadata_string(&metadata, "version").unwrap_or_default(),
        source_hash: sha256_uri(layout_markdown.as_bytes()),
    };
    validate_document(
        DocumentValidationInput {
            metadata: &metadata,
            citation_references: &citation_references,
            bibliography: &bibliography,
            duplicate_bibliography_keys: &duplicate_bibliography_keys,
            comments: &comments,
            ai_sources: &ai_sources,
            ai_assisted_sections: &ai_assisted_sections,
            has_bibliography_source: !bibliography.is_empty(),
            source_map: &source_map,
        },
        &mut diagnostics,
    );
    let included_files = include_graph
        .iter()
        .filter_map(|edge| manifest_file(&edge.child))
        .collect::<Vec<_>>();
    let media_files = manifest_media_files(&document_ast);
    let manifest = ExportManifest {
        document_title: title.clone(),
        document_version: metadata
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("0.1.0")
            .to_string(),
        status: status.clone(),
        exported_at: Utc::now().to_rfc3339(),
        source_hash: sha256_uri(source.as_bytes()),
        output_path: None,
        output_hash: None,
        included_files,
        media_files,
        export_target: "preview".to_string(),
        export_options: json!({}),
        transform_artifacts: transform_artifacts
            .iter()
            .map(|artifact| {
                json!({
                    "id": artifact.id,
                    "name": artifact.name,
                    "outputKind": artifact.output_kind,
                    "sourceHash": artifact.source_hash,
                    "source": artifact.source.clone(),
                    "options": artifact.options.clone(),
                    "outputHash": artifact.output_hash,
                    "cacheKey": artifact.cache_key,
                    "executionKind": artifact.execution_kind,
                    "engineVersion": artifact.engine_version,
                    "enginePath": artifact.engine_path,
                    "inputMode": artifact.input_mode,
                    "durationMs": artifact.duration_ms,
                    "diagnostics": artifact.diagnostics
                })
            })
            .collect(),
        diagnostics: diagnostics.clone(),
        source_map: source_map.clone(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let table_summaries = collect_table_summaries(&table_formula_markdown);
    let semantic = SemanticDocument {
        title,
        status,
        headings: headings.clone(),
        outline: headings,
        tables: table_summaries.len(),
        table_summaries,
        figures: count_figures(&transformed_markdown),
        equations: count_equations(&transformed_markdown),
        citations,
        citation_references,
        duplicate_bibliography_keys,
        glossary,
        layout_directives,
        comments,
        change_notes,
        ai_sources,
        ai_assisted_sections,
        labels,
        cross_references,
    };
    CompileResponse {
        compiled_markdown: layout_markdown,
        html,
        semantic,
        document_ast,
        diagnostics,
        include_graph,
        source_map,
        metadata,
        bibliography,
        index_terms,
        formula_graph,
        formula_dependency_edges: formula_edges,
        transform_artifacts,
        export_manifest: manifest,
    }
}

fn apply_transforms(
    text: &str,
    options: &TransformExecutionOptions,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> (String, Vec<TransformArtifact>) {
    let mut output = String::new();
    let mut artifacts = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        if let Some(info) = line.trim().strip_prefix("```") {
            let name = info.split_whitespace().next().unwrap_or("");
            if supported_transform(name) {
                let fence_options = transform_fence_options(info);
                let mut body = String::new();
                for body_line in lines.by_ref() {
                    if body_line.trim() == "```" {
                        break;
                    }
                    body.push_str(body_line);
                    body.push('\n');
                }
                let artifact = render_transform(name, &body, &fence_options, options, diagnostics);
                output.push_str(&artifact.html);
                output.push('\n');
                artifacts.push(artifact);
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }
    (output, artifacts)
}

fn transform_fence_options(info: &str) -> Value {
    let mut fields = serde_json::Map::new();
    for token in transform_info_tokens(info).into_iter().skip(1) {
        if let Some((key, value)) = token.split_once('=') {
            let value = value.trim_matches(|ch| ch == '"' || ch == '\'');
            fields.insert(key.to_string(), Value::String(value.to_string()));
        } else if !token.is_empty() {
            fields.insert(token.to_string(), Value::Bool(true));
        }
    }
    Value::Object(fields)
}

fn transform_info_tokens(info: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote = None::<char>;
    for ch in info.chars() {
        if let Some(quote_ch) = quote {
            if ch == quote_ch {
                quote = None;
            } else {
                token.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch.is_whitespace() {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
        } else {
            token.push(ch);
        }
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}

fn render_transform(
    name: &str,
    body: &str,
    fence_options: &Value,
    options: &TransformExecutionOptions,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> TransformArtifact {
    if external_transform_supported(name) {
        if let Some(artifact) =
            render_external_transform(name, body, fence_options, options, diagnostics)
        {
            return artifact;
        }
    }

    let source_hash = sha256_hex(body.as_bytes());
    let mut artifact_diags = Vec::new();
    let html = match name {
        "calc" => "<aside class=\"transform transform-calc\">Calculations resolved into document variables.</aside>".to_string(),
        "csv" => render_delimited_table(body, ',', &mut artifact_diags, diagnostics),
        "tsv" => render_delimited_table(body, '\t', &mut artifact_diags, diagnostics),
        "json" => render_structured_data_html("json", body, &mut artifact_diags, diagnostics),
        "yaml" => render_structured_data_html("yaml", body, &mut artifact_diags, diagnostics),
        "glossary" => render_glossary_html(body),
        "layout" => render_layout_block_html(body),
        "timeline" => render_timeline_svg(body),
        "roadmap" => render_roadmap_html(body),
        "adr" => render_adr_html(body),
        "diff" => render_diff_html(body),
        "qr" => transforms::qr::render_qr_svg(body, &mut artifact_diags, diagnostics),
        "chart" => render_chart_svg(body),
        "openapi" => render_openapi_html(body, &mut artifact_diags, diagnostics),
        "json-schema" => render_json_schema_html(body, &mut artifact_diags, diagnostics),
        "bibtex" => render_bibtex_html(body, &mut artifact_diags, diagnostics),
        "geojson" => render_geojson_svg(body, &mut artifact_diags, diagnostics),
        "topojson" => render_topojson_svg(body, &mut artifact_diags, diagnostics),
        "stl" => render_stl_svg(body, &mut artifact_diags, diagnostics),
        "vega-lite" => render_vega_lite_svg(body, &mut artifact_diags, diagnostics),
        "mermaid" => transforms::diagram::render_mermaid_svg(body, &mut artifact_diags, diagnostics),
        "pikchr" => transforms::diagram::render_pikchr_svg(body, &mut artifact_diags, diagnostics),
        "dot" | "graphviz" => transforms::diagram::render_dot_svg(name, body, &mut artifact_diags, diagnostics),
        "plantuml" => transforms::diagram::render_plantuml_svg(body, &mut artifact_diags, diagnostics),
        "d2" => transforms::diagram::render_d2_svg(body, &mut artifact_diags, diagnostics),
        _ => format!("<pre>{}</pre>", escape_html(body)),
    };
    let output_hash = sha256_hex(html.as_bytes());
    TransformArtifact {
        id: format!("{name}-{source_hash}"),
        name: name.to_string(),
        output_kind: if html.contains("<svg") { "svg" } else { "html" }.to_string(),
        output_hash,
        cache_key: transform_cache_key(name, "embedded", "rust-native", &source_hash),
        execution_kind: "embedded".to_string(),
        engine_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        engine_path: None,
        input_mode: "embedded".to_string(),
        duration_ms: None,
        source_hash,
        source: body.to_string(),
        options: fence_options.clone(),
        html,
        diagnostics: artifact_diags,
    }
}

fn render_external_transform(
    name: &str,
    body: &str,
    fence_options: &Value,
    options: &TransformExecutionOptions,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Option<TransformArtifact> {
    let engine_path = options.engine_path(name)?;
    let request = ExternalTransformRequest {
        name: name.to_string(),
        body: body.to_string(),
        engine_path: Some(engine_path),
        trusted: options.trusted(name),
        input_mode: options.input_mode(name),
        timeout_ms: options.timeout_ms,
        max_input_bytes: None,
        max_output_bytes: None,
    };
    match run_external_transform(request) {
        Ok(mut artifact) => {
            artifact.source = body.to_string();
            artifact.options = fence_options.clone();
            diagnostics.extend(artifact.diagnostics.iter().cloned());
            Some(artifact)
        }
        Err(error) => {
            diagnostics.push(diag(
                "warning",
                format!("{name} external transform failed: {error}"),
                None,
                None,
                Some("Check transform trust, engine path, input mode, and timeout settings."),
            ));
            None
        }
    }
}

fn supported_transform(name: &str) -> bool {
    matches!(
        name,
        "calc"
            | "csv"
            | "tsv"
            | "json"
            | "yaml"
            | "glossary"
            | "layout"
            | "timeline"
            | "roadmap"
            | "adr"
            | "diff"
            | "qr"
            | "chart"
            | "mermaid"
            | "pikchr"
            | "dot"
            | "graphviz"
            | "plantuml"
            | "d2"
            | "vega-lite"
            | "geojson"
            | "topojson"
            | "stl"
            | "openapi"
            | "json-schema"
            | "bibtex"
    )
}

fn external_transform_supported(name: &str) -> bool {
    matches!(name, "pikchr" | "dot" | "graphviz" | "plantuml" | "d2")
}

fn extract_headings(text: &str) -> Vec<Heading> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim_start();
            let level = trimmed.chars().take_while(|ch| *ch == '#').count();
            if (1..=6).contains(&level) && trimmed.chars().nth(level) == Some(' ') {
                let raw_text = trimmed[level..].trim();
                let text = strip_heading_attributes(raw_text).to_string();
                if text.is_empty() {
                    return None;
                }
                Some(Heading {
                    level,
                    anchor: extract_label(raw_text).unwrap_or_else(|| slugify(&text)),
                    text,
                    line: index + 1,
                })
            } else {
                None
            }
        })
        .collect()
}

fn strip_heading_attributes(text: &str) -> &str {
    text.split("{#").next().unwrap_or(text).trim()
}

fn collect_glossary(text: &str) -> BTreeMap<String, String> {
    let mut glossary = BTreeMap::new();
    for body in collect_fence_bodies(text, "glossary") {
        for line in body.lines() {
            if let Some((term, definition)) = line.split_once(':') {
                glossary.insert(term.trim().to_string(), definition.trim().to_string());
            }
        }
    }
    glossary
}

fn citation_style(metadata: &Value) -> &str {
    metadata
        .get("citationStyle")
        .or_else(|| metadata.get("cslStyle"))
        .or_else(|| metadata.get("citation_style"))
        .and_then(Value::as_str)
        .unwrap_or("title")
}

fn collect_fence_bodies(text: &str, target: &str) -> Vec<String> {
    collect_fence_bodies_with_lines(text, target)
        .into_iter()
        .map(|(_, body)| body)
        .collect()
}

fn collect_fence_bodies_with_lines(text: &str, target: &str) -> Vec<(usize, String)> {
    let mut bodies = Vec::new();
    let mut lines = text.lines().enumerate();
    while let Some((line_index, line)) = lines.next() {
        if line
            .trim()
            .strip_prefix("```")
            .map(|info| info.split_whitespace().next().unwrap_or("") == target)
            .unwrap_or(false)
        {
            let mut body = String::new();
            for (_, body_line) in lines.by_ref() {
                if body_line.trim() == "```" {
                    break;
                }
                body.push_str(body_line);
                body.push('\n');
            }
            bodies.push((line_index + 1, body));
        }
    }
    bodies
}

fn render_structured_data_html(
    format: &str,
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let parsed = if format == "json" {
        serde_json::from_str::<Value>(body).map_err(|err| err.to_string())
    } else {
        serde_yaml::from_str::<Value>(body).map_err(|err| err.to_string())
    };
    match parsed {
        Ok(value) => {
            if let Some(table) = render_structured_table(format, &value) {
                table
            } else {
                format!(
                    "<section class=\"transform transform-{format} structured-tree\">{}</section>",
                    render_structured_tree("root", &value)
                )
            }
        }
        Err(error) => {
            let diagnostic = diag(
                "error",
                format!(
                    "Invalid {} transform input: {error}",
                    format.to_ascii_uppercase()
                ),
                None,
                None,
                Some("Check the structured data syntax."),
            );
            diagnostics.push(diagnostic.clone());
            artifact_diags.push(diagnostic);
            format!(
                "<pre class=\"transform transform-{format} transform-error\">{}</pre>",
                escape_html(body)
            )
        }
    }
}

fn render_structured_table(format: &str, value: &Value) -> Option<String> {
    let rows = value.as_array()?;
    if rows.is_empty() || !rows.iter().all(Value::is_object) {
        return None;
    }
    let headers = rows
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|object| object.keys().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if headers.is_empty() {
        return None;
    }
    let mut html = format!("<table class=\"transform-table transform-{format}\"><thead><tr>");
    for header in &headers {
        html.push_str(&format!("<th>{}</th>", escape_html(header)));
    }
    html.push_str("</tr></thead><tbody>");
    for row in rows {
        let object = row.as_object()?;
        html.push_str("<tr>");
        for header in &headers {
            let cell = object
                .get(header)
                .map(structured_value_summary)
                .unwrap_or_default();
            html.push_str(&format!("<td>{}</td>", escape_html(&cell)));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    Some(html)
}

fn render_structured_tree(label: &str, value: &Value) -> String {
    match value {
        Value::Object(object) => {
            let mut html = format!(
                "<details open><summary>{}</summary><dl>",
                escape_html(label)
            );
            for (key, value) in object {
                html.push_str("<dt>");
                html.push_str(&escape_html(key));
                html.push_str("</dt><dd>");
                html.push_str(&render_structured_tree(key, value));
                html.push_str("</dd>");
            }
            html.push_str("</dl></details>");
            html
        }
        Value::Array(values) => {
            let mut html = format!(
                "<details open><summary>{} [{}]</summary><ol>",
                escape_html(label),
                values.len()
            );
            for value in values {
                html.push_str("<li>");
                html.push_str(&render_structured_tree("item", value));
                html.push_str("</li>");
            }
            html.push_str("</ol></details>");
            html
        }
        _ => escape_html(&structured_value_summary(value)),
    }
}

fn structured_value_summary(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => format!("[{} items]", values.len()),
        Value::Object(object) => format!("{{{} fields}}", object.len()),
    }
}

fn render_glossary_html(body: &str) -> String {
    let mut html = String::from("<dl class=\"glossary\">");
    for line in body.lines() {
        if let Some((term, definition)) = line.split_once(':') {
            html.push_str(&format!(
                "<dt>{}</dt><dd>{}</dd>",
                escape_html(term.trim()),
                escape_html(definition.trim())
            ));
        }
    }
    html.push_str("</dl>");
    html
}

fn render_bibtex_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let entries = parse_bibliography_source(body);
    if entries.is_empty() {
        let diagnostic = diag(
            "warning",
            "BibTeX transform did not contain any bibliography entries.",
            None,
            None,
            Some("Add BibTeX entries such as @book{key, title={Title}}."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-bibtex transform-error\">No bibliography entries found.</section>".to_string();
    }
    let mut html = String::from("<dl class=\"transform transform-bibtex\">");
    for entry in entries {
        html.push_str(&format!(
            "<dt>{}</dt><dd>{}</dd>",
            escape_html(&entry.key),
            escape_html(&entry.title)
        ));
    }
    html.push_str("</dl>");
    html
}

fn render_timeline_svg(body: &str) -> String {
    let items = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let height = 80 + items.len() * 54;
    let mut svg = format!("<svg class=\"transform transform-timeline timeline\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 {height}\" role=\"img\"><line x1=\"120\" y1=\"40\" x2=\"120\" y2=\"{}\" stroke=\"#275DA8\" stroke-width=\"3\"/>", height - 30);
    for (index, item) in items.iter().enumerate() {
        let y = 50 + index * 54;
        svg.push_str(&format!("<circle cx=\"120\" cy=\"{y}\" r=\"8\" fill=\"#275DA8\"/><text x=\"150\" y=\"{}\" font-size=\"18\" fill=\"#1f2937\">{}</text>", y + 6, escape_html(item)));
    }
    svg.push_str("</svg>");
    svg
}

fn render_roadmap_html(body: &str) -> String {
    let items = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (stage, text) = line
                .split_once(':')
                .or_else(|| line.split_once('-'))
                .map(|(stage, text)| (stage.trim(), text.trim()))
                .unwrap_or(("Item", line));
            format!(
                "<article><strong>{}</strong><p>{}</p></article>",
                escape_html(stage),
                escape_html(text)
            )
        })
        .collect::<String>();
    format!(
        "<section class=\"transform transform-roadmap\"><h3>Roadmap</h3><div>{items}</div></section>"
    )
}

fn render_adr_html(body: &str) -> String {
    let rows = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (key, value) = line
                .split_once(':')
                .map(|(key, value)| (key.trim(), value.trim()))
                .unwrap_or(("Note", line));
            format!(
                "<tr><th>{}</th><td>{}</td></tr>",
                escape_html(key),
                escape_html(value)
            )
        })
        .collect::<String>();
    format!(
        "<section class=\"transform transform-adr\"><h3>Architecture Decision Record</h3><table><tbody>{rows}</tbody></table></section>"
    )
}

fn render_diff_html(body: &str) -> String {
    let lines = body
        .lines()
        .map(|line| {
            let class = if line.starts_with('+') && !line.starts_with("+++") {
                "add"
            } else if line.starts_with('-') && !line.starts_with("---") {
                "del"
            } else if line.starts_with("@@") {
                "hunk"
            } else {
                "ctx"
            };
            format!("<code class=\"diff-{class}\">{}</code>", escape_html(line))
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<pre class=\"transform transform-diff\">{lines}</pre>")
}

fn render_vega_lite_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let spec = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid Vega-Lite JSON: {err}"),
                None,
                None,
                Some("Provide a JSON Vega-Lite spec with data.values and x/y encodings."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-vega-lite transform-error\">Invalid Vega-Lite JSON</section>".to_string();
        }
    };
    let mark = vega_lite_mark(&spec);
    if !matches!(mark.as_str(), "bar" | "line" | "point") {
        let diagnostic = diag(
            "warning",
            format!("Unsupported Vega-Lite mark for native preview: {mark}"),
            None,
            None,
            Some("Use bar, line, or point marks for the native static preview."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">Unsupported Vega-Lite mark</section>".to_string();
    }
    let Some(x_field) = vega_lite_encoding_field(&spec, "x") else {
        return vega_lite_missing_field("x", artifact_diags, diagnostics);
    };
    let Some(y_field) = vega_lite_encoding_field(&spec, "y") else {
        return vega_lite_missing_field("y", artifact_diags, diagnostics);
    };
    let values = vega_lite_values(&spec, &x_field, &y_field);
    if values.is_empty() {
        let diagnostic = diag(
            "warning",
            "Vega-Lite native preview did not find numeric data.values rows.",
            None,
            None,
            Some("Use inline data.values with a numeric y encoding."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-vega-lite transform-error\">No drawable Vega-Lite rows</section>".to_string();
    }
    let title = spec
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Vega-Lite chart");
    render_vega_lite_chart_svg(title, &mark, &values)
}

fn vega_lite_mark(spec: &Value) -> String {
    spec.get("mark")
        .and_then(|mark| {
            mark.as_str().map(ToString::to_string).or_else(|| {
                mark.get("type")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
        })
        .unwrap_or_else(|| "bar".to_string())
}

fn vega_lite_encoding_field(spec: &Value, channel: &str) -> Option<String> {
    spec.pointer(&format!("/encoding/{channel}/field"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn vega_lite_values(spec: &Value, x_field: &str, y_field: &str) -> Vec<(String, f64)> {
    spec.pointer("/data/values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let x = row.get(x_field).map(value_to_axis_label)?;
            let y = row
                .get(y_field)
                .and_then(|value| value.as_f64().or_else(|| value.as_str()?.parse().ok()))?;
            Some((x, y))
        })
        .collect()
}

fn value_to_axis_label(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value_to_string(value))
}

fn vega_lite_missing_field(
    channel: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let diagnostic = diag(
        "warning",
        format!("Vega-Lite native preview is missing {channel} field encoding."),
        None,
        None,
        Some("Set encoding.x.field and encoding.y.field."),
    );
    artifact_diags.push(diagnostic.clone());
    diagnostics.push(diagnostic);
    format!(
        "<section class=\"transform transform-vega-lite transform-error\">Missing {channel} encoding</section>"
    )
}

fn render_vega_lite_chart_svg(title: &str, mark: &str, values: &[(String, f64)]) -> String {
    let max = values
        .iter()
        .map(|(_, value)| *value)
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(1.0);
    let width = 820usize;
    let height = 320usize;
    let plot_left = 72usize;
    let plot_bottom = 262usize;
    let plot_width = 680usize;
    let step = plot_width / values.len().max(1);
    let mut svg = format!(
        "<svg class=\"transform transform-vega-lite\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" role=\"img\"><text x=\"72\" y=\"34\" font-size=\"18\" fill=\"#111827\">{}</text><line x1=\"72\" y1=\"262\" x2=\"770\" y2=\"262\" stroke=\"#94a3b8\"/><line x1=\"72\" y1=\"54\" x2=\"72\" y2=\"262\" stroke=\"#94a3b8\"/>",
        escape_html(title)
    );
    let points = values
        .iter()
        .enumerate()
        .map(|(index, (_, value))| {
            let x = plot_left + index * step + step / 2;
            let y = plot_bottom - ((*value / max) * 190.0) as usize;
            (x, y)
        })
        .collect::<Vec<_>>();
    if mark == "bar" {
        for (index, (label, value)) in values.iter().enumerate() {
            let bar_height = ((*value / max) * 190.0) as usize;
            let x = plot_left + index * step + 8;
            let y = plot_bottom - bar_height;
            svg.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{}\" height=\"{bar_height}\" fill=\"#275DA8\"/><text x=\"{x}\" y=\"286\" font-size=\"12\">{}</text>",
                step.saturating_sub(16),
                escape_html(label)
            ));
        }
    } else {
        if mark == "line" {
            let polyline = points
                .iter()
                .map(|(x, y)| format!("{x},{y}"))
                .collect::<Vec<_>>()
                .join(" ");
            svg.push_str(&format!(
                "<polyline points=\"{polyline}\" fill=\"none\" stroke=\"#275DA8\" stroke-width=\"3\"/>"
            ));
        }
        for ((x, y), (label, _)) in points.iter().zip(values.iter()) {
            svg.push_str(&format!(
                "<circle cx=\"{x}\" cy=\"{y}\" r=\"5\" fill=\"#275DA8\"/><text x=\"{}\" y=\"286\" font-size=\"12\">{}</text>",
                x.saturating_sub(12),
                escape_html(label)
            ));
        }
    }
    svg.push_str("</svg>");
    svg
}

fn render_geojson_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid GeoJSON document: {err}"),
                None,
                None,
                Some("Provide valid GeoJSON Feature, FeatureCollection, or Geometry JSON."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-geojson transform-error\">Invalid GeoJSON document</section>".to_string();
        }
    };
    let mut positions = Vec::new();
    collect_geojson_positions(&value, &mut positions);
    if positions.is_empty() {
        let diagnostic = diag(
            "warning",
            "GeoJSON transform did not contain drawable coordinates.",
            None,
            None,
            Some("Add Point, LineString, Polygon, or Multi* coordinates."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-geojson transform-error\">No GeoJSON coordinates found</section>".to_string();
    }
    let positions = positions.into_iter().take(2000).collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let points = positions
        .iter()
        .map(|position| {
            let (x, y) = project_geojson_position(*position, min_x, max_x, min_y, max_y);
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>();
    let markers = points
        .iter()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap_or(("0", "0"));
            format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"3\" fill=\"#0f766e\"/>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<svg class=\"transform transform-geojson\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"#ecfeff\" stroke=\"#67e8f9\"/><polyline points=\"{}\" fill=\"none\" stroke=\"#275DA8\" stroke-width=\"3\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>{markers}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"#134e4a\">{} coordinates</text></svg>",
        points.join(" "),
        positions.len()
    )
}

fn render_topojson_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match serde_json::from_str::<Value>(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid TopoJSON document: {err}"),
                None,
                None,
                Some("Provide valid TopoJSON with an arcs array."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-topojson transform-error\">Invalid TopoJSON document</section>".to_string();
        }
    };
    let arcs = decode_topojson_arcs(&value);
    if arcs.is_empty() {
        let diagnostic = diag(
            "warning",
            "TopoJSON transform did not contain drawable arcs.",
            None,
            None,
            Some("Add a Topology arcs array or verify the TopoJSON source."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-topojson transform-error\">No TopoJSON arcs found</section>".to_string();
    }
    let positions = arcs
        .iter()
        .flatten()
        .copied()
        .take(4000)
        .collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let polylines = arcs
        .iter()
        .map(|arc| {
            let points = arc
                .iter()
                .map(|position| {
                    let (x, y) = project_geojson_position(*position, min_x, max_x, min_y, max_y);
                    format!("{x:.2},{y:.2}")
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!(
                "<polyline points=\"{points}\" fill=\"none\" stroke=\"#275DA8\" stroke-width=\"3\" stroke-linejoin=\"round\" stroke-linecap=\"round\"/>"
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<svg class=\"transform transform-topojson\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"#f8fafc\" stroke=\"#94a3b8\"/>{polylines}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"#334155\">{} arcs</text></svg>",
        arcs.len()
    )
}

fn decode_topojson_arcs(value: &Value) -> Vec<Vec<(f64, f64)>> {
    let scale = value
        .pointer("/transform/scale")
        .and_then(Value::as_array)
        .and_then(|items| Some((items.first()?.as_f64()?, items.get(1)?.as_f64()?)))
        .unwrap_or((1.0, 1.0));
    let translate = value
        .pointer("/transform/translate")
        .and_then(Value::as_array)
        .and_then(|items| Some((items.first()?.as_f64()?, items.get(1)?.as_f64()?)))
        .unwrap_or((0.0, 0.0));
    value
        .get("arcs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|arc| decode_topojson_arc(arc, scale, translate))
        .collect()
}

fn decode_topojson_arc(
    arc: &Value,
    (scale_x, scale_y): (f64, f64),
    (translate_x, translate_y): (f64, f64),
) -> Option<Vec<(f64, f64)>> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut positions = Vec::new();
    for point in arc.as_array()? {
        let coordinates = point.as_array()?;
        x += coordinates.first()?.as_f64()?;
        y += coordinates.get(1)?.as_f64()?;
        positions.push((x * scale_x + translate_x, y * scale_y + translate_y));
    }
    (!positions.is_empty()).then_some(positions)
}

fn render_stl_svg(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let vertices = parse_ascii_stl_vertices(body);
    if vertices.is_empty() {
        let diagnostic = diag(
            "warning",
            "STL transform did not contain ASCII vertex data.",
            None,
            None,
            Some("Use ASCII STL fences for static previews, or configure an external STL renderer later."),
        );
        artifact_diags.push(diagnostic.clone());
        diagnostics.push(diagnostic);
        return "<section class=\"transform transform-stl transform-error\">No ASCII STL vertices found</section>".to_string();
    }
    let positions = vertices
        .iter()
        .map(|(x, y, _)| (*x, *y))
        .collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = geojson_bounds(&positions);
    let triangles = vertices
        .chunks(3)
        .filter(|triangle| triangle.len() == 3)
        .map(|triangle| {
            let points = triangle
                .iter()
                .map(|(x, y, _)| {
                    let (x, y) = project_geojson_position((*x, *y), min_x, max_x, min_y, max_y);
                    format!("{x:.2},{y:.2}")
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!("<polygon points=\"{points}\" fill=\"rgba(39,93,168,.18)\" stroke=\"#275DA8\" stroke-width=\"2\"/>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        "<svg class=\"transform transform-stl\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 900 460\" role=\"img\"><rect x=\"24\" y=\"24\" width=\"852\" height=\"412\" rx=\"8\" fill=\"#f8fafc\" stroke=\"#cbd5e1\"/>{triangles}<text x=\"34\" y=\"52\" font-size=\"16\" fill=\"#334155\">{} triangles / {} vertices</text></svg>",
        vertices.len() / 3,
        vertices.len()
    )
}

fn parse_ascii_stl_vertices(body: &str) -> Vec<(f64, f64, f64)> {
    body.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            if parts.next()? != "vertex" {
                return None;
            }
            Some((
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
                parts.next()?.parse().ok()?,
            ))
        })
        .collect()
}

fn collect_geojson_positions(value: &Value, positions: &mut Vec<(f64, f64)>) {
    match value {
        Value::Array(items) => {
            if items.len() >= 2 {
                if let (Some(x), Some(y)) = (items[0].as_f64(), items[1].as_f64()) {
                    positions.push((x, y));
                    return;
                }
            }
            for item in items {
                collect_geojson_positions(item, positions);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                collect_geojson_positions(value, positions);
            }
        }
        _ => {}
    }
}

fn geojson_bounds(positions: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    positions.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), (x, y)| {
            (min_x.min(*x), max_x.max(*x), min_y.min(*y), max_y.max(*y))
        },
    )
}

fn project_geojson_position(
    (x, y): (f64, f64),
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) -> (f64, f64) {
    let width = (max_x - min_x).abs().max(0.000_001);
    let height = (max_y - min_y).abs().max(0.000_001);
    let projected_x = 48.0 + ((x - min_x) / width) * 804.0;
    let projected_y = 412.0 - ((y - min_y) / height) * 364.0;
    (projected_x, projected_y)
}

fn render_openapi_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match parse_json_or_yaml(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid OpenAPI document: {err}"),
                None,
                None,
                Some("Provide valid JSON or YAML OpenAPI content."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-error\">Invalid OpenAPI document</section>"
                .to_string();
        }
    };
    let mut html = String::from(
        "<table class=\"transform-table openapi\"><thead><tr><th>Method</th><th>Path</th><th>Summary</th></tr></thead><tbody>",
    );
    if let Some(paths) = value.get("paths").and_then(Value::as_object) {
        for (path, methods) in paths {
            if let Some(methods) = methods.as_object() {
                for (method, operation) in methods {
                    let summary = operation
                        .get("summary")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                        escape_html(&method.to_ascii_uppercase()),
                        escape_html(path),
                        escape_html(summary)
                    ));
                }
            }
        }
    }
    html.push_str("</tbody></table>");
    html
}

fn render_json_schema_html(
    body: &str,
    artifact_diags: &mut Vec<DocumentDiagnostic>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> String {
    let value = match parse_json_or_yaml(body) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = diag(
                "error",
                format!("Invalid JSON Schema document: {err}"),
                None,
                None,
                Some("Provide valid JSON or YAML JSON Schema content."),
            );
            artifact_diags.push(diagnostic.clone());
            diagnostics.push(diagnostic);
            return "<section class=\"transform transform-error\">Invalid JSON Schema document</section>"
                .to_string();
        }
    };
    let required = value
        .get("required")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let mut html = String::from(
        "<table class=\"transform-table json-schema\"><thead><tr><th>Field</th><th>Type</th><th>Required</th><th>Description</th></tr></thead><tbody>",
    );
    if let Some(properties) = value.get("properties").and_then(Value::as_object) {
        for (field, schema) in properties {
            let kind = schema.get("type").and_then(Value::as_str).unwrap_or("");
            let description = schema
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape_html(field),
                escape_html(kind),
                if required.contains(field.as_str()) {
                    "yes"
                } else {
                    "no"
                },
                escape_html(description)
            ));
        }
    }
    html.push_str("</tbody></table>");
    html
}

fn parse_json_or_yaml(body: &str) -> Result<Value, String> {
    serde_json::from_str::<Value>(body)
        .or_else(|_| serde_yaml::from_str::<Value>(body))
        .map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(FileWatcherState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            open_file,
            read_file,
            save_file_as,
            save_file,
            watch_file,
            start_file_watcher,
            stop_file_watcher,
            rename_file,
            duplicate_file,
            reveal_path,
            file_metadata,
            list_workspace_files,
            compile_document,
            compile_document_with_options,
            export_document,
            prepare_for_export,
            create_snapshot,
            list_snapshots,
            restore_snapshot,
            get_git_status,
            git_history,
            git_diff,
            commit_document_changes,
            tag_release,
            restore_git_revision,
            list_transform_engines,
            run_transform,
            run_external_transform,
            cleanup_ai_paste
        ])
        .run(tauri::generate_context!())
        .expect("error while running NEditor");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::ZipArchive;

    #[cfg(unix)]
    fn write_executable_script(prefix: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("neditor-{prefix}-{unique}.sh"));
        fs::write(&path, body).expect("write executable test script");
        let mut permissions = fs::metadata(&path).expect("script metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make script executable");
        path
    }

    fn installed_command_path(env_var: &str, command: &str) -> Option<PathBuf> {
        if let Some(path) = std::env::var_os(env_var)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && path.is_file())
        {
            return Some(path);
        }

        let path_value = std::env::var_os("PATH")?;
        for directory in std::env::split_paths(&path_value) {
            let direct = directory.join(command);
            if direct.is_file() {
                return Some(direct);
            }
            #[cfg(windows)]
            {
                for extension in ["exe", "bat", "cmd"] {
                    let candidate = directory.join(format!("{command}.{extension}"));
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }
        None
    }

    fn sample_document() -> String {
        r#"---
title: Test Report
version: 1.2.0
status: approved
approvedBy: QA
toc: true
client: Acme
brand:
  name: Acme
  logo: "data:image/svg+xml;base64,PHN2Zy8+"
---

# Test Report

[TOC]

Prepared for {{client}}.

```calc
revenue = 100
cost = 40
profit = revenue - cost
margin = profit / revenue
healthy = IF(revenue > cost, 1, 0)
target_met = IF(margin >= 0.60, 1, 0)
cost_match = IF(cost == 40, 1, 0)
spread = IF(revenue != cost, 1, 0)
discount = 12.5%
```

Margin: {{=margin | percent}}
After tax: {{=profit * 0.70 | currency}}
Healthy score: {{=IF(revenue > cost, profit, 0) | round}}
Discount: {{=discount | percent}}

```csv caption="Regional revenue" audited
Region,Revenue
East,100
West,80
```

```glossary
ARR: Annual recurring revenue.
```

[INDEX]
"#
        .to_string()
    }

    #[test]
    fn compiler_resolves_metadata_variables_transforms_and_manifest() {
        let response = compile(CompileRequest {
            text: sample_document(),
            file_path: None,
        });

        assert_eq!(response.semantic.title, "Test Report");
        assert_eq!(response.semantic.status, "approved");
        assert!(response.compiled_markdown.contains("Prepared for Acme."));
        assert!(response.compiled_markdown.contains("Margin: 60.00%"));
        assert!(response.compiled_markdown.contains("After tax: $42.00"));
        assert!(response.compiled_markdown.contains("Healthy score: 60"));
        assert!(response.compiled_markdown.contains("Discount: 12.50%"));
        assert!(response.html.contains("Table of Contents"));
        assert!(response.html.contains("transform-table"));
        assert!(response.html.contains("<h1 id=\"test-report\">"));
        assert!(response.html.contains("href=\"#test-report\""));
        assert!(response.index_terms.iter().any(|term| term == "ARR"));
        assert_eq!(response.export_manifest.document_version, "1.2.0");
        let csv_artifact = response
            .transform_artifacts
            .iter()
            .find(|artifact| artifact.name == "csv")
            .expect("csv transform artifact");
        assert!(!csv_artifact.output_hash.is_empty());
        assert!(csv_artifact.source.contains("Region,Revenue"));
        assert_eq!(
            csv_artifact.options.get("caption").and_then(Value::as_str),
            Some("Regional revenue")
        );
        assert_eq!(
            csv_artifact.options.get("audited").and_then(Value::as_bool),
            Some(true)
        );
        let manifest_csv_artifact = response
            .export_manifest
            .transform_artifacts
            .iter()
            .find(|artifact| artifact.get("name").and_then(Value::as_str) == Some("csv"))
            .expect("csv manifest artifact");
        assert_eq!(
            manifest_csv_artifact
                .get("sourceHash")
                .and_then(Value::as_str),
            Some(csv_artifact.source_hash.as_str())
        );
        assert_eq!(
            manifest_csv_artifact.get("source").and_then(Value::as_str),
            Some(csv_artifact.source.as_str())
        );
        assert_eq!(
            manifest_csv_artifact
                .get("options")
                .and_then(|options| options.get("caption"))
                .and_then(Value::as_str),
            Some("Regional revenue")
        );
        assert_eq!(
            manifest_csv_artifact
                .get("outputHash")
                .and_then(Value::as_str),
            Some(csv_artifact.output_hash.as_str())
        );
        assert!(manifest_csv_artifact
            .get("cacheKey")
            .and_then(Value::as_str)
            .is_some_and(|cache_key| !cache_key.is_empty()));
        assert_eq!(
            manifest_csv_artifact
                .get("executionKind")
                .and_then(Value::as_str),
            Some("embedded")
        );
        assert_eq!(
            manifest_csv_artifact
                .get("inputMode")
                .and_then(Value::as_str),
            Some("embedded")
        );
        assert_eq!(
            manifest_csv_artifact
                .get("engineVersion")
                .and_then(Value::as_str),
            Some(env!("CARGO_PKG_VERSION"))
        );
        assert!(response
            .formula_graph
            .iter()
            .any(|formula| formula.name == "profit" && formula.value == Some(60.0)));
        let profit_formula = response
            .formula_graph
            .iter()
            .find(|formula| formula.name == "profit")
            .expect("profit formula");
        assert!(matches!(
            profit_formula.ast.as_ref(),
            Some(calculations::FormulaAstNode::Binary { op, .. }) if op == "-"
        ));
        assert!(response
            .formula_graph
            .iter()
            .any(|formula| formula.name == "healthy" && formula.value == Some(1.0)));
        assert!(response
            .formula_graph
            .iter()
            .any(|formula| formula.name == "target_met" && formula.value == Some(1.0)));
        assert!(response
            .formula_graph
            .iter()
            .any(|formula| formula.name == "cost_match" && formula.value == Some(1.0)));
        assert!(response
            .formula_graph
            .iter()
            .any(|formula| formula.name == "spread" && formula.value == Some(1.0)));
        assert!(response.formula_graph.iter().any(|formula| {
            formula.name == "discount"
                && (formula.value.unwrap_or_default() - 0.125).abs() < f64::EPSILON
        }));
    }

    #[test]
    fn compiler_supports_default_document_variables() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Defaults\nstatus: approved\napprovedBy: QA\nclient: Acme\n---\n# Defaults\nPrepared for {{client | default:Fallback}} in {{region | default:\"East Africa\"}}.\nStill missing {{owner}}.\n".to_string(),
            file_path: None,
        });

        assert!(response
            .compiled_markdown
            .contains("Prepared for Acme in East Africa."));
        let missing_owner = response
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic
                    .message
                    .contains("Missing document variable: owner")
            })
            .expect("missing owner diagnostic");
        assert_eq!(missing_owner.line, Some(9));
        assert_eq!(missing_owner.column, Some(15));
        assert_eq!(missing_owner.end_line, Some(9));
        assert_eq!(missing_owner.end_column, Some(24));
        assert_eq!(missing_owner.source_file.as_deref(), Some("untitled.md"));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("region")));
    }

    #[test]
    fn calc_blocks_resolve_forward_refs_and_report_cycles() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Calc Graph\nstatus: approved\napprovedBy: QA\n---\n# Calc Graph\n```calc\nprofit = revenue - cost\ncost = 40\nrevenue = 100\ncycle_a = cycle_b + 1\ncycle_b = cycle_a + 1\n```\n\nProfit: {{=profit | round}}\n".to_string(),
            file_path: None,
        });

        assert!(response.compiled_markdown.contains("Profit: 60"));
        assert!(response
            .formula_graph
            .iter()
            .any(|formula| formula.name == "profit" && formula.value == Some(60.0)));
        assert!(response
            .formula_dependency_edges
            .iter()
            .any(|edge| edge.from == "profit" && edge.to == "revenue"));
        assert!(response
            .formula_dependency_edges
            .iter()
            .any(|edge| edge.from == "profit" && edge.to == "cost"));
        assert!(response.formula_graph.iter().any(|formula| {
            formula.name == "cycle_a"
                && formula
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("#CYCLE? cycle_a -> cycle_b -> cycle_a"))
        }));
        assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("#CYCLE? cycle_a -> cycle_b -> cycle_a")));
    }

    #[test]
    fn inline_formula_diagnostics_include_source_ranges() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Formula Diagnostics\nstatus: approved\napprovedBy: QA\n---\n# Formula Diagnostics\nBad: {{=missing + 1}}\n"
                .to_string(),
            file_path: None,
        });

        let diagnostic = response
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("Inline formula error"))
            .expect("inline formula diagnostic");
        assert_eq!(diagnostic.line, Some(7));
        assert_eq!(diagnostic.column, Some(6));
        assert_eq!(diagnostic.end_line, Some(7));
        assert_eq!(diagnostic.end_column, Some(22));
        assert_eq!(diagnostic.source_file.as_deref(), Some("untitled.md"));
    }

    #[test]
    fn compiler_loads_project_level_variables_without_overriding_front_matter() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-project-vars-test-{unique}"));
        fs::create_dir_all(root.join(".neditor")).expect("create project vars dir");
        fs::write(
            root.join(".neditor").join("variables.yaml"),
            "client: Project Client\nregion: West\nowner: Strategy Office\n",
        )
        .expect("write project variables");
        let doc = root.join("docs").join("report.md");
        fs::create_dir_all(doc.parent().expect("doc parent")).expect("create docs dir");
        fs::write(&doc, "# Report").expect("write doc");

        let response = compile(CompileRequest {
            text: "---\ntitle: Project Vars\nstatus: approved\napprovedBy: QA\nclient: Front Matter Client\n---\n# Project Vars\nPrepared for {{client}} in {{region}} by {{owner}}.\n".to_string(),
            file_path: Some(path_to_string(&doc)),
        });

        assert!(response
            .compiled_markdown
            .contains("Prepared for Front Matter Client in West by Strategy Office."));
        assert_eq!(response.metadata["client"], "Front Matter Client");
        assert_eq!(response.metadata["region"], "West");
        fs::remove_dir_all(root).expect("clean project vars test dir");
    }

    #[test]
    fn compiler_loads_front_matter_csv_data_sources() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-data-source-test-{unique}"));
        fs::create_dir_all(root.join("data")).expect("create data dir");
        fs::write(
            root.join("data").join("revenue.csv"),
            "Region,Revenue\n\"East\nCoast\",100\nWest,\"=SUM(B1,80)\"\n",
        )
        .expect("write csv data source");

        let response = compile(CompileRequest {
            text: "---\ntitle: Data Source\nstatus: approved\napprovedBy: QA\ndataSources:\n  - name: Revenue\n    path: data/revenue.csv\n    type: csv\n---\n# Data Source\n".to_string(),
            file_path: Some(path_to_string(&root.join("report.md"))),
        });

        assert!(response
            .compiled_markdown
            .contains("## Data Source: Revenue"));
        assert!(response.html.contains("<td>180</td>"));
        assert!(response.html.contains("East\nCoast"));
        assert!(response
            .include_graph
            .iter()
            .any(|edge| edge.child.ends_with("data/revenue.csv")));
        assert!(response
            .export_manifest
            .included_files
            .iter()
            .any(|file| file.path.ends_with("data/revenue.csv")));
        assert!(response.export_manifest.source_hash.starts_with("sha256:"));
        assert!(response
            .export_manifest
            .included_files
            .iter()
            .all(|file| file.hash.starts_with("sha256:")));
        fs::remove_dir_all(root).expect("clean data source test dir");
    }

    #[test]
    fn compiler_honors_toc_depth_and_numbering() {
        let response = compile(CompileRequest {
            text: "---\ntitle: TOC\nstatus: approved\napprovedBy: QA\ntoc: true\ntocDepth: 2\ntocNumbered: true\n---\n# Alpha\n## Beta\n### Gamma\n## Delta\n".to_string(),
            file_path: None,
        });

        assert!(response.compiled_markdown.contains("- [1 Alpha](#alpha)"));
        assert!(response.compiled_markdown.contains("  - [1.1 Beta](#beta)"));
        assert!(response
            .compiled_markdown
            .contains("  - [1.2 Delta](#delta)"));
        assert!(!response.compiled_markdown.contains("[1.1.1 Gamma](#gamma)"));
        let docx = render_docx_bytes(&response, &json!({})).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains(r#"w:instr="TOC \o &quot;1-2&quot; \h \z \u""#));
        assert!(!docx_document.contains("#alpha"));
    }

    #[test]
    fn compiler_adds_glossary_hover_terms_to_preview_html() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Glossary Hover\nstatus: approved\napprovedBy: QA\n---\n# Glossary Hover\nARR informs planning.\n\n```glossary\nARR: Annual recurring revenue.\n```\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("class=\"glossary-term\""));
        assert!(response
            .html
            .contains("title=\"Annual recurring revenue.\""));
        assert!(response.html.contains(">ARR</span> informs planning"));
    }

    #[test]
    fn compiler_preserves_figure_float_semantics() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Floating Figure\nstatus: approved\napprovedBy: QA\n---\n# Floating Figure\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:float caption=\"Floating diagram\" float=\"right\"}\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("figure-float-right"));
        assert!(response.html.contains("data-float=\"right\""));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Figure {
                    id,
                    caption,
                    float,
                    ..
                } if id.as_deref() == Some("fig:float")
                    && caption.as_deref() == Some("Floating diagram")
                    && float.as_deref() == Some("right")
            )
        }));

        let exported = export::export_text(&response, &json!({}));
        assert!(exported.contains("float=right"));

        let full_html = render_full_html(&response, &json!({}));
        assert!(full_html.contains("figure-float-right"));
        assert!(full_html.contains("float:right"));

        let docx = render_docx_bytes(&response, &json!({})).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains("float=right"));
        assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));

        let pptx = render_pptx_bytes(&response, &json!({})).expect("pptx bytes");
        let floating_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
            .into_iter()
            .find(|slide| slide.contains(r#"r:embed="rIdImage1""#))
            .expect("floating figure slide");
        assert!(floating_slide.contains(r#"<a:off x="5029200""#));

        let pdf = render_pdf_bytes(&response, &json!({}));
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("287 627 240 135 re S"));
    }

    #[test]
    fn compiler_generates_linked_index_with_exclusions_and_proper_terms() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Index\nstatus: approved\napprovedBy: QA\nindexExclude:\n  - internal draft\n---\n# Market Analysis\nAcme Strategy appears here. **Working Capital** matters.\n\n## Follow Up\nAcme Strategy returns. Internal Draft should stay out. Working capital{#index:Liquidity} marker.\n\n[INDEX]\n".to_string(),
            file_path: None,
        });

        assert!(response
            .index_terms
            .iter()
            .any(|term| term == "Acme Strategy"));
        assert!(response.index_terms.iter().any(|term| term == "Liquidity"));
        assert!(response
            .index_terms
            .iter()
            .any(|term| term == "Working Capital"));
        assert!(!response
            .index_terms
            .iter()
            .any(|term| term == "Internal Draft"));
        assert!(response.html.contains("href=\"#market-analysis\""));
        assert!(response.html.contains("Acme Strategy"));
        assert!(response.html.contains("Liquidity"));
        assert!(!response.html.contains("{#index:Liquidity}"));
    }

    #[test]
    fn compiler_parses_review_comment_metadata() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Review\nstatus: approved\napprovedBy: QA\n---\n# Review\n<!-- comment: unresolved | author: Dana | at: 2026-05-18T10:00:00Z | Clarify the risk note. -->\n<!-- change: author: Dana | at: 2026-05-18T11:00:00Z | Updated the risk note. -->\n".to_string(),
            file_path: None,
        });
        let comment = response.semantic.comments.first().expect("review comment");
        let change_note = response.semantic.change_notes.first().expect("change note");

        assert_eq!(comment.state, "unresolved");
        assert_eq!(comment.author, "Dana");
        assert_eq!(comment.created_at, "2026-05-18T10:00:00Z");
        assert_eq!(comment.text, "Clarify the risk note.");
        assert_eq!(change_note.author, "Dana");
        assert_eq!(change_note.created_at, "2026-05-18T11:00:00Z");
        assert_eq!(change_note.text, "Updated the risk note.");
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::ReviewComment { comment, .. }
                    if comment.author == "Dana"
                        && comment.state == "unresolved"
                        && comment.text == "Clarify the risk note."
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::ChangeNote { note, .. }
                    if note.author == "Dana" && note.text == "Updated the risk note."
            )
        }));
        let unresolved_comment_diagnostic = response
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("unresolved review comments"))
            .expect("unresolved comment diagnostic");
        assert_eq!(unresolved_comment_diagnostic.severity, "error");
        assert_eq!(unresolved_comment_diagnostic.line, Some(7));
        assert_eq!(
            unresolved_comment_diagnostic.source_file.as_deref(),
            Some("untitled.md")
        );
        assert!(unresolved_comment_diagnostic
            .related
            .iter()
            .any(|related| related.contains("Clarify the risk note")));
    }

    #[test]
    fn compiler_reports_missing_include_without_panicking() {
        let response = compile(CompileRequest {
            text: "!include missing/chapter.md\n".to_string(),
            file_path: None,
        });

        let diagnostic = response
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.severity == "error"
                    && diagnostic.message.contains("Missing include file")
            })
            .expect("missing include diagnostic");
        assert!(diagnostic
            .related
            .iter()
            .any(|related| related.contains("missing/chapter.md")));
    }

    #[test]
    fn compiler_reports_broken_local_markdown_links() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-link-test-{unique}"));
        fs::create_dir_all(root.join("docs")).expect("create link test dir");
        fs::write(root.join("docs").join("existing.md"), "# Existing").expect("write linked doc");

        let response = compile(CompileRequest {
            text: "---\ntitle: Links\nstatus: approved\napprovedBy: QA\nbrand:\n  logo: docs/missing-logo.svg\n---\n# Links\nRead [existing](docs/existing.md), [missing](docs/missing.md), [section](#links), and [web](https://example.com).\n![Missing image](docs/missing.png)\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });
        let root_doc = path_to_string(&root.join("root.md"));

        let broken_link = response
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("Broken link path"))
            .expect("broken link diagnostic");
        assert_eq!(broken_link.line, Some(9));
        assert!(broken_link.column.is_some());
        assert!(broken_link.end_column > broken_link.column);
        assert_eq!(broken_link.source_file.as_deref(), Some(root_doc.as_str()));
        assert!(broken_link
            .related
            .iter()
            .any(|related| related.contains("docs/missing.md")));
        assert_eq!(
            response
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.message.contains("Broken link path"))
                .count(),
            1
        );
        let broken_image = response
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("Broken image path"))
            .expect("broken image diagnostic");
        assert_eq!(broken_image.line, Some(10));
        assert!(broken_image.column.is_some());
        assert!(broken_image.end_column > broken_image.column);
        assert_eq!(broken_image.source_file.as_deref(), Some(root_doc.as_str()));
        assert!(broken_image
            .related
            .iter()
            .any(|related| related.contains("docs/missing.png")));
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Broken logo path")));
        fs::remove_dir_all(root).expect("clean link test dir");
    }

    #[test]
    fn compiler_loads_external_bibliography_and_validates_cross_refs() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-bib-test-{unique}"));
        fs::create_dir_all(&root).expect("create bib test dir");
        fs::write(
            root.join("refs.bib"),
            "@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n@article{doe2026,\n title={Evidence Based Reports},\n author={Doe},\n date={2026-04-01}\n}",
        )
        .expect("write bibliography");
        fs::write(root.join("diagram.svg"), "<svg></svg>").expect("write figure");

        let response = compile(CompileRequest {
            text: "---\ntitle: Cited\nstatus: approved\napprovedBy: QA\nbibliography: refs.bib\ncitationStyle: author-year\n---\n# Cited\nClaim [@porter1985, p. 42; @doe2026].\n\n![Diagram](diagram.svg){#fig:diagram caption=\"System diagram\"}\nSee {@fig:diagram} and {@fig:missing}.\n\n![Missing](missing.png){#fig:missing-image caption=\"Missing image\"}".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert_eq!(response.bibliography.len(), 2);
        assert!(response
            .bibliography
            .iter()
            .any(|entry| entry.key == "doe2026" && entry.issued.as_deref() == Some("2026")));
        assert_eq!(response.semantic.citations, vec!["doe2026", "porter1985"]);
        assert!(response
            .semantic
            .citation_references
            .iter()
            .any(|citation| {
                citation.key == "porter1985"
                    && citation.locator.as_deref() == Some("p. 42")
                    && citation.column == 8
                    && citation.end_column > citation.column
            }));
        assert!(response.html.contains("Porter 1985, p. 42; Doe 2026"));
        assert!(response
            .html
            .contains("title=\"@porter1985 (p. 42): Competitive Advantage; @doe2026: Evidence Based Reports\""));
        assert!(response
            .html
            .contains("aria-label=\"Citation: @porter1985 (p. 42): Competitive Advantage; @doe2026: Evidence Based Reports\""));
        assert!(response.html.contains("<figure"));
        assert!(response.html.contains("System diagram"));
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Broken image path")));
        assert!(response
            .semantic
            .cross_references
            .iter()
            .any(|reference| reference.key == "fig:diagram"
                && reference.resolved
                && reference.line == 12));
        let broken_cross_reference = response
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic
                    .message
                    .contains("Broken cross reference: fig:missing")
            })
            .expect("broken cross-reference diagnostic");
        assert_eq!(broken_cross_reference.line, Some(12));
        assert!(broken_cross_reference.column.is_some());
        assert!(broken_cross_reference.end_column > broken_cross_reference.column);
        assert!(broken_cross_reference
            .related
            .iter()
            .any(|related| related.contains("{@fig:missing}")));
        fs::remove_dir_all(root).expect("clean bib test dir");
    }

    #[test]
    fn compile_options_supply_default_citation_style() {
        let response = compile_with_options(
            CompileRequest {
                text: "Claim [@porter1985].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n```".to_string(),
                file_path: None,
            },
            &json!({ "defaultCitationStyle": "author-year" }),
        );

        assert_eq!(
            response
                .metadata
                .get("citationStyle")
                .and_then(Value::as_str),
            Some("author-year")
        );
        assert!(response.html.contains("Porter 1985"));
    }

    #[test]
    fn compile_options_do_not_override_document_citation_style() {
        let response = compile_with_options(
            CompileRequest {
                text: "---\ncitationStyle: key\n---\nClaim [@porter1985].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n```".to_string(),
                file_path: None,
            },
            &json!({ "defaultCitationStyle": "author-year" }),
        );

        assert_eq!(
            response
                .metadata
                .get("citationStyle")
                .and_then(Value::as_str),
            Some("key")
        );
        assert!(response.html.contains("@porter1985"));
    }

    #[test]
    fn compile_options_supply_brand_profile_defaults() {
        let response = compile_with_options(
            CompileRequest {
                text: "# Branded\n".to_string(),
                file_path: None,
            },
            &json!({
                "defaultBrandProfile": {
                    "name": "Acme Strategy",
                    "color": "#0F766E",
                    "logo": "brand/acme.svg",
                    "font": "Aptos",
                    "header": "{{title}}",
                    "footer": "Confidential | Page {{page}}",
                    "legalDisclaimer": "Internal use only."
                }
            }),
        );

        assert_eq!(
            response
                .metadata
                .pointer("/brand/name")
                .and_then(Value::as_str),
            Some("Acme Strategy")
        );
        assert_eq!(
            response
                .metadata
                .pointer("/brand/color")
                .and_then(Value::as_str),
            Some("#0F766E")
        );
        assert_eq!(
            response
                .metadata
                .pointer("/brand/logo")
                .and_then(Value::as_str),
            Some("brand/acme.svg")
        );
        assert_eq!(
            response
                .metadata
                .pointer("/brand/font")
                .and_then(Value::as_str),
            Some("Aptos")
        );
        assert_eq!(
            response
                .metadata
                .pointer("/layout/header")
                .and_then(Value::as_str),
            Some("{{title}}")
        );
        assert_eq!(
            response
                .metadata
                .pointer("/layout/footer")
                .and_then(Value::as_str),
            Some("Confidential | Page {{page}}")
        );
        assert_eq!(
            response
                .metadata
                .get("legalDisclaimer")
                .and_then(Value::as_str),
            Some("Internal use only.")
        );
        let options = json!({ "watermark": "BOARD" });
        let html = render_full_html(&response, &options);
        assert!(html.contains("font-family:Aptos"));
        assert!(html.contains("Legal Disclaimer"));
        assert!(html.contains("Internal use only."));
        let exported_text = export::export_text(&response, &options);
        assert!(exported_text.contains("Header: Branded"));
        assert!(exported_text.contains("Footer: Confidential | Page 1"));
        assert!(exported_text.contains("Watermark: BOARD"));
        assert!(exported_text.contains("Legal Disclaimer"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let legal_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
            .into_iter()
            .find(|slide| slide.contains("Legal Disclaimer"))
            .expect("legal disclaimer slide");
        assert!(legal_slide.contains("<a:t>Legal Disclaimer</a:t>"));
        assert!(legal_slide.contains("Internal use only."));
    }

    #[test]
    fn compile_options_do_not_override_document_brand_profile() {
        let response = compile_with_options(
            CompileRequest {
                text: "---\nbrand:\n  name: Document Brand\n  color: \"#111111\"\n---\n# Branded\n"
                    .to_string(),
                file_path: None,
            },
            &json!({
                "defaultBrandProfile": {
                    "name": "Acme Strategy",
                    "color": "#0F766E",
                    "logo": "brand/acme.svg"
                }
            }),
        );

        assert_eq!(
            response
                .metadata
                .pointer("/brand/name")
                .and_then(Value::as_str),
            Some("Document Brand")
        );
        assert_eq!(
            response
                .metadata
                .pointer("/brand/color")
                .and_then(Value::as_str),
            Some("#111111")
        );
        assert_eq!(
            response
                .metadata
                .pointer("/brand/logo")
                .and_then(Value::as_str),
            Some("brand/acme.svg")
        );
    }

    #[test]
    fn export_options_control_cover_styles_and_page_numbers() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Export Options\nstatus: approved\napprovedBy: QA\n---\n# Export Options\n\nBody."
                .to_string(),
            file_path: None,
        });
        let options = json!({
            "includeStyles": false,
            "coverPage": false,
            "pageNumbers": false
        });

        let html = render_full_html(&response, &options);
        assert!(!html.contains("<style>"));
        assert!(!html.contains("class=\"cover\""));
        assert!(!html.contains("Page 1 of 1"));
        assert!(html.contains("<main>"));

        let exported_text = export::export_text(&response, &options);
        assert!(!exported_text.contains("Cover: Export Options"));
        assert!(!exported_text.contains("Page 1 of 1"));
        assert!(exported_text.contains("Status: approved"));
    }

    #[test]
    fn export_layout_preset_controls_html_css_and_metadata() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Layout Options\nstatus: approved\napprovedBy: QA\n---\n# Layout Options\n\nBody."
                .to_string(),
            file_path: None,
        });
        let options = json!({ "layoutPreset": "compact" });

        let html = render_full_html(&response, &options);
        assert!(html.contains("margin:32px"));
        assert!(html.contains("line-height:1.42"));
        assert!(html.contains("p,li,blockquote{orphans:2;widows:2}"));
        assert!(html.contains("@page{size:A4;margin:18mm"));

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let document = zip_entry_text(&docx, "word/document.xml");
        assert!(document.contains("<w:widowControl/>"));

        let exported_text = export::export_text(&response, &options);
        assert!(exported_text.contains("Layout preset: compact"));
    }

    #[test]
    fn export_layout_metadata_controls_page_size_and_margins() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Page Layout\nstatus: approved\napprovedBy: QA\nlayout:\n  pageSize: Letter\n  margins: wide\n---\n# Page Layout\n\nBody.".to_string(),
            file_path: None,
        });
        let options = json!({});

        let html = render_full_html(&response, &options);
        assert!(html.contains("@page{size:Letter;margin:32mm"));

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let document = zip_entry_text(&docx, "word/document.xml");
        assert!(document.contains(r#"<w:pgSz w:w="12240" w:h="15840"/>"#));
        assert!(document
            .contains(r#"<w:pgMar w:top="1800" w:right="1800" w:bottom="1800" w:left="1800"/>"#));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("/MediaBox [0 0 612 792]"));
        assert!(pdf_text.contains("BT /F1 10 Tf 91 701 Td"));
    }

    #[test]
    fn compiler_validates_layout_page_metadata() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Bad Layout\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-18\nlayout:\n  pageSize: Tabloid\n  margins: huge\n---\n# Bad Layout\n".to_string(),
            file_path: None,
        });

        assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("Unsupported layout pageSize: Tabloid")));
        assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("Unsupported layout margins: huge")));
    }

    #[test]
    fn export_syntax_highlighting_can_be_included_or_omitted() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Syntax Options\nstatus: approved\napprovedBy: QA\n---\n# Syntax Options\n\n```js\nconst total = 42; // amount\n```\n"
                .to_string(),
            file_path: None,
        });

        let highlighted = render_full_html(&response, &json!({}));
        assert!(highlighted.contains("class=\"syn-keyword\""));
        assert!(highlighted.contains("class=\"syn-number\""));
        assert!(highlighted.contains("class=\"syn-comment\""));
        assert!(highlighted.contains(".syn-keyword"));

        let plain = render_full_html(&response, &json!({ "includeSyntaxHighlighting": false }));
        assert!(!plain.contains("class=\"syn-keyword\""));
        assert!(!plain.contains(".syn-keyword"));
        assert!(plain.contains("const total = 42; // amount"));

        let exported_text =
            export::export_text(&response, &json!({ "includeSyntaxHighlighting": false }));
        assert!(exported_text.contains("Syntax highlighting: omitted"));
    }

    #[test]
    fn compiler_loads_csl_json_bibliography() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-csl-test-{unique}"));
        fs::create_dir_all(&root).expect("create csl test dir");
        fs::write(
            root.join("refs.json"),
            r#"[{"id":"doe2026","title":"Evidence Based Reports"}]"#,
        )
        .expect("write csl bibliography");

        let response = compile(CompileRequest {
            text: "---\ntitle: CSL\nstatus: approved\napprovedBy: QA\nbibliography: refs.json\n---\n# CSL\nClaim [@doe2026].\n[BIBLIOGRAPHY]".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert_eq!(response.bibliography[0].key, "doe2026");
        assert!(response.html.contains("Evidence Based Reports"));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Broken citation")));
        fs::remove_dir_all(root).expect("clean csl test dir");
    }

    #[test]
    fn compiler_loads_hayagriva_yaml_bibliography() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Hayagriva\nstatus: approved\napprovedBy: QA\ncitationStyle: author-year\n---\n# Hayagriva\nClaim [@porter1985].\n\n```hayagriva\nporter1985:\n  type: book\n  title: Competitive Advantage\n  author: Porter\n  date: 1985\n```\n[BIBLIOGRAPHY]".to_string(),
            file_path: None,
        });

        assert_eq!(response.bibliography.len(), 1);
        assert_eq!(response.bibliography[0].key, "porter1985");
        assert_eq!(response.bibliography[0].author.as_deref(), Some("Porter"));
        assert_eq!(response.bibliography[0].issued.as_deref(), Some("1985"));
        assert!(response.html.contains("Porter 1985"));
        assert!(response.html.contains("Competitive Advantage"));
    }

    #[test]
    fn compiler_reports_duplicate_bibliography_keys() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Duplicate Bibliography\nstatus: approved\napprovedBy: QA\n---\n# Duplicate Bibliography\nClaim [@porter1985].\n\n```bibtex\n@book{porter1985, title={Competitive Advantage}}\n@article{porter1985, title={Duplicate Entry}}\n```\n[BIBLIOGRAPHY]".to_string(),
            file_path: None,
        });

        assert_eq!(
            response.semantic.duplicate_bibliography_keys,
            vec!["porter1985".to_string()]
        );
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Duplicate bibliography key")));
    }

    #[test]
    fn citation_export_conformance_covers_required_cases() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Citation Export\nstatus: approved\napprovedBy: QA\ncitationStyle: author-year\n---\n# Citation Export\nSingle [@porter1985].\nMultiple [@porter1985; @doe2026].\nLocator [@porter1985, p. 42].\nMissing [@missing2026].\nSecond [@doe2026].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n@article{doe2026,\n title={Evidence Based Reports},\n author={Doe},\n year={2026}\n}\n```\n\n[BIBLIOGRAPHY]\n".to_string(),
            file_path: None,
        });
        let options = json!({});

        assert_eq!(
            response.semantic.citations,
            vec!["doe2026", "missing2026", "porter1985"]
        );
        let broken_citation = response
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("Broken citation: missing2026"))
            .expect("broken citation diagnostic");
        assert_eq!(broken_citation.line, Some(11));
        assert_eq!(broken_citation.column, Some(10));
        assert!(broken_citation.end_column > broken_citation.column);
        assert!(broken_citation
            .related
            .iter()
            .any(|related| related.contains("@missing2026")));

        let html = render_full_html(&response, &options);
        assert!(html.contains("Porter 1985"));
        assert!(html.contains("Porter 1985; Doe 2026"));
        assert!(html.contains("Porter 1985, p. 42"));
        assert!(html.contains("missing bibliography entry"));
        assert!(html.contains("Bibliography"));
        assert!(html.contains("Competitive Advantage"));
        assert!(html.contains("Evidence Based Reports"));

        let docx = render_docx_bytes(&response, &options).expect("docx citation bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains("Porter 1985"));
        assert!(docx_document.contains("Porter 1985; Doe 2026"));
        assert!(docx_document.contains("Porter 1985, p. 42"));
        assert!(docx_document.contains("missing2026"));
        assert!(docx_document.contains("Competitive Advantage"));
        assert!(docx_document.contains("Evidence Based Reports"));
        assert!(docx_document.contains(r#"w:name="bib_porter1985""#));
        assert!(docx_document.contains(r#"w:name="bib_doe2026""#));
        assert!(docx_document.contains(r#"w:instr="CITATION porter1985 \l 1033""#));
        assert!(docx_document.contains(r#"w:instr="CITATION porter1985 \m doe2026 \l 1033""#));
        assert!(docx_document.contains(r#"w:instr="BIBLIOGRAPHY \l 1033""#));
        assert!(docx_document.contains(r#"<w:hyperlink w:anchor="bib_porter1985""#));
        assert!(docx_document.contains(r#"<w:hyperlink w:anchor="bib_doe2026""#));
        assert!(!docx_document.contains(r#"w:anchor="bib_missing2026""#));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx citation bytes");
        let slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/");
        assert!(slides.iter().any(|slide| slide.contains("Porter 1985")));
        assert!(slides
            .iter()
            .any(|slide| slide.contains("Porter 1985; Doe 2026")));
        assert!(slides
            .iter()
            .any(|slide| slide.contains("Porter 1985, p. 42")));
        assert!(slides.iter().any(|slide| slide.contains("missing2026")));
        assert!(slides
            .iter()
            .any(|slide| slide.contains("Competitive Advantage")));
        assert!(slides
            .iter()
            .any(|slide| slide.contains("Evidence Based Reports")));
    }

    #[test]
    fn compiler_renders_block_and_inline_equations() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Math\nstatus: approved\napprovedBy: QA\n---\n# Math\nInline \\(ROI = x\\).\n\n$$\nROI = \\frac{Gain - Cost}{Cost}\n$$ {#eq:roi}\n\nSee {@eq:roi}.".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("class=\"equation\""));
        assert!(response.html.contains("id=\"eq:roi\""));
        assert!(response.html.contains("Equation 1"));
        assert!(response.html.contains("class=\"math math-inline\""));
        assert!(response.html.contains("class=\"math-frac\""));
        assert!(response.html.contains("role=\"math\""));
        assert!(response.html.contains("<summary>LaTeX</summary>"));
        assert!(response
            .compiled_markdown
            .contains("See [Equation roi](#eq:roi)."));
        assert!(response
            .html
            .contains(r##"<a href="#eq:roi">Equation roi</a>"##));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Equation { text, .. } if text.contains("\\frac")
            )
        }));
        assert!(response
            .semantic
            .cross_references
            .iter()
            .any(|reference| reference.key == "eq:roi" && reference.resolved));
    }

    #[test]
    fn compiler_renders_markdown_footnotes() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Footnotes\nversion: 1.0.0\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-18\n---\n# Footnotes\nA governed claim.[^risk]\n\n[^risk]: Reviewed by compliance.\n    Includes second-line evidence.\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("role=\"doc-endnotes\""));
        assert!(response.html.contains("id=\"fn:risk\""));
        assert!(response.html.contains("Reviewed by compliance."));
        assert!(response.html.contains("Includes second-line evidence."));
        assert!(!response.compiled_markdown.contains("[^risk]:"));
        assert!(!response
            .compiled_markdown
            .contains("    Includes second-line evidence."));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Footnotes { entries, .. }
                    if entries.len() == 1
                        && entries[0].key == "risk"
                        && entries[0].text.contains("Reviewed by compliance.")
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Paragraph { inlines, .. }
                    if inlines.iter().any(|node| matches!(
                        node,
                        document_ast::InlineNode::FootnoteReference { key, number, .. }
                            if key == "risk" && *number == 1
                    ))
            )
        }));

        let options = json!({});
        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("Footnotes"));
        assert!(pdf_text.contains("Reviewed by compliance."));
        assert!(!pdf_text.contains("<section"));

        let docx = render_docx_bytes(&response, &options).expect("docx footnotes");
        let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
        let docx_footnotes = zip_entry_text(&docx, "word/footnotes.xml");
        assert!(docx_content_types.contains("wordprocessingml.footnotes+xml"));
        assert!(docx_relationships.contains(r#"Target="footnotes.xml""#));
        assert!(docx_document.contains("Footnotes"));
        assert!(docx_document.contains("A governed claim."));
        assert!(docx_document.contains(r#"<w:footnoteReference w:id="1""#));
        assert!(!docx_document.contains("Footnote 1"));
        assert!(docx_footnotes.contains(r#"<w:footnote w:id="1""#));
        assert!(docx_footnotes.contains("Reviewed by compliance."));
        assert!(docx_footnotes.contains("Includes second-line evidence."));
        assert!(!docx_document.contains("&lt;section"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx footnotes");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert!(pptx_slide.contains("Footnotes"));
        assert!(pptx_slide.contains("Reviewed by compliance."));
        assert!(!pptx_slide.contains("&lt;section"));
    }

    #[test]
    fn compiler_summarizes_markdown_tables() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Tables\nstatus: approved\napprovedBy: QA\n---\n# Tables\nTable: Revenue by region {#tbl:revenue}\n| Region | Revenue |\n| --- | ---: |\n| East | 100 |\n| West | =SUM(B1,80) |\n| Total | =SUM(B1:B2) |\n\nSee {@tbl:revenue}.\n".to_string(),
            file_path: None,
        });

        assert!(response.compiled_markdown.contains("| West | 180 |"));
        assert!(response.compiled_markdown.contains("| Total | 280 |"));
        assert!(response.html.contains(">280</td>"));
        assert_eq!(response.semantic.tables, 1);
        assert_eq!(response.semantic.table_summaries[0].rows, 3);
        assert_eq!(
            response.semantic.table_summaries[0]
                .numeric_columns
                .get("Revenue"),
            Some(&560.0)
        );
        assert!(response
            .semantic
            .cross_references
            .iter()
            .any(|reference| reference.key == "tbl:revenue" && reference.resolved));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Table { id, caption, .. }
                    if id.as_deref() == Some("tbl:revenue")
                        && caption.as_deref() == Some("Revenue by region")
            )
        }));
    }

    #[test]
    fn cross_references_resolve_heading_appendix_and_decision_anchors() {
        let response = compile(CompileRequest {
            text: "---\ntitle: References\nstatus: approved\napprovedBy: QA\n---\n# Strategy {#sec:strategy}\nSee {@sec:strategy}, {@appendix-a}, and {@decision-record}.\n\n## Appendix A\nSupporting detail.\n\n## Decision Record\nUse local-first exports.\n".to_string(),
            file_path: None,
        });

        assert!(response
            .semantic
            .headings
            .iter()
            .any(|heading| heading.text == "Strategy" && heading.anchor == "sec:strategy"));
        for key in ["sec:strategy", "appendix-a", "decision-record"] {
            assert!(response
                .semantic
                .cross_references
                .iter()
                .any(|reference| reference.key == key && reference.resolved));
        }
        assert!(response.compiled_markdown.contains(
            "See [Section strategy](#sec:strategy), [Section appendix a](#appendix-a), and [Section decision record](#decision-record)."
        ));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Broken cross reference")));
    }

    #[test]
    fn csv_and_tsv_transforms_evaluate_table_formula_cells() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Formula Tables\nstatus: approved\napprovedBy: QA\n---\n# Formula Tables\n```csv\nMetric,Value\nTotal,=10+15\nRounded,=ROUND(2.6)\nRange,=SUM(B1:B2)\n```\n\n```tsv\nMetric\tValue\nAbs\t=ABS(-5)\nSum\t=SUM(2,3)\nProfitable\t=IF(10>5,1,0)\nEqual\t=IF(ROUND(2.6)=3,1,0)\nRange\t=SUM(B1:B4)\n```\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("<td>25</td>"));
        assert!(response.html.contains("<td>3</td>"));
        assert!(response.html.contains("<td>1</td>"));
        assert!(response.html.contains("<td>5</td>"));
        assert!(response.html.contains("<td>28</td>"));
        assert!(response.html.contains("<td>12</td>"));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Table formula error")));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Table { headers, rows, .. }
                    if headers == &vec!["Metric".to_string(), "Value".to_string()]
                        && rows.iter().any(|row| row == &vec![
                            "Total".to_string(),
                            "25".to_string()
                        ])
            )
        }));

        let options = json!({});
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains("<w:tbl>"));
        assert!(docx_document.contains(">25<"));
        assert!(!docx_document.contains("```csv"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert!(pptx_slide.contains("<a:tbl>"));
        assert!(pptx_slide.contains("25"));
        assert!(!pptx_slide.contains("```csv"));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains(" re S"));
        assert!(pdf_text.contains("(25) Tj"));
    }

    #[test]
    fn table_formulas_resolve_forward_refs_and_report_cycles() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Formula Cycles\nstatus: approved\napprovedBy: QA\n---\n# Formula Cycles\n| Metric | Value |\n| --- | ---: |\n| Forward | =B2 |\n| Source | 42 |\n| Cycle A | =B4 |\n| Cycle B | =B3 |\n".to_string(),
            file_path: None,
        });

        assert!(response.compiled_markdown.contains("| Forward | 42 |"));
        assert!(response.compiled_markdown.contains("| Cycle A | #ERROR |"));
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("#CYCLE? B3 -> B4 -> B3")));
    }

    #[test]
    fn table_formulas_reference_named_tables() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Named Tables\nstatus: approved\napprovedBy: QA\n---\n# Named Tables\nTable: Revenue {#tbl:revenue}\n| Region | Revenue |\n| --- | ---: |\n| East | 100 |\n| West | 180 |\n| Total | =SUM(B1:B2) |\n\nTable: Summary {#tbl:summary}\n| Metric | Value |\n| --- | ---: |\n| Revenue rollup | =SUM(tbl:revenue!B1:B3) |\n| Reported total | =revenue!B3 |\n".to_string(),
            file_path: None,
        });

        assert!(response
            .compiled_markdown
            .contains("| Revenue rollup | 560 |"));
        assert!(response
            .compiled_markdown
            .contains("| Reported total | 280 |"));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("#NAME?")));
    }

    #[test]
    fn markdown_tables_preserve_escaped_pipes_across_ast_and_formulas() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Escaped Tables\nstatus: approved\napprovedBy: QA\n---\n# Escaped Tables\nTable: Pricing notes {#tbl:pricing}\n| Product | Notes | Value |\n| --- | --- | ---: |\n| A \\| B | keep literal pipe | 10 |\n| Total | formula keeps source readable | =SUM(C1,5) |\n".to_string(),
            file_path: None,
        });

        assert!(response.compiled_markdown.contains("| A \\| B |"));
        assert!(response
            .compiled_markdown
            .contains("| Total | formula keeps source readable | 15 |"));
        assert_eq!(response.semantic.table_summaries[0].columns.len(), 3);
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Table { headers, rows, .. }
                    if headers == &vec![
                        "Product".to_string(),
                        "Notes".to_string(),
                        "Value".to_string()
                    ]
                    && rows.iter().any(|row| row == &vec![
                        "A | B".to_string(),
                        "keep literal pipe".to_string(),
                        "10".to_string()
                    ])
                    && rows.iter().any(|row| row == &vec![
                        "Total".to_string(),
                        "formula keeps source readable".to_string(),
                        "15".to_string()
                    ])
            )
        }));
    }

    #[test]
    fn edited_table_fixture_exports_to_all_packages() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Edited Table Export\nstatus: approved\napprovedBy: QA\n---\n# Edited Table Export\nTable: Edited revenue {#tbl:edited}\n| Region | Revenue | Margin |\n| --- | ---: | ---: |\n| East | $125,000 | 42% |\n| West | $98,000 | 38% |\n| Total | =SUM(B1:B2) | =AVG(C1:C2) |\n".to_string(),
            file_path: None,
        });

        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "error"));
        assert!(response
            .compiled_markdown
            .contains("| Total | 223000 | 40 |"));
        assert_eq!(response.semantic.tables, 1);
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Table {
                    id,
                    caption,
                    headers,
                    rows,
                    ..
                } if id.as_deref() == Some("tbl:edited")
                    && caption.as_deref() == Some("Edited revenue")
                    && headers == &vec!["Region".to_string(), "Revenue".to_string(), "Margin".to_string()]
                    && rows.iter().any(|row| row == &vec![
                        "Total".to_string(),
                        "223000".to_string(),
                        "40".to_string()
                    ])
            )
        }));

        let options = json!({});
        let docx = render_docx_bytes(&response, &options).expect("docx edited table");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains("Edited revenue"));
        assert!(docx_document.contains("223000"));
        assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx edited table");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert!(pptx_slide.contains("Edited revenue"));
        assert!(pptx_slide.contains("<a:tbl>"));
        assert!(pptx_slide.contains("<a:t>223000</a:t>"));
        assert!(pptx_slide.contains(r#"<a:pPr algn="r"/>"#));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("Edited revenue"));
        assert!(pdf_text.contains("(223000) Tj"));
        assert!(pdf_text.contains("(40) Tj"));

        let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
            .expect("edited table bundle");
        let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
        assert!(bundled_ast.contains("\"kind\": \"table\""));
        assert!(bundled_ast.contains("Edited revenue"));
        assert!(bundled_ast.contains("223000"));
    }

    #[test]
    fn edited_table_permutation_exports_alignment_escapes_and_formula_rows() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Edited Table Permutations\nstatus: approved\napprovedBy: QA\n---\n# Edited Table Permutations\nTable: Scenario grid {#tbl:scenario}\n| Scenario | Owner | Score | Status |\n| :--- | :---: | ---: | --- |\n| Base \\| Case | Finance | $1,200.50 | Ready |\n| Stretch | Ops | 75% | Watch |\n| Floor | Risk | 20 | Hold |\n| Min | Summary | =MIN(C1:C3) | Formula |\n| Max | Summary | =MAX(C1:C3) | Formula |\n| Count | Summary | =COUNT(C1:C3) | Formula |\n".to_string(),
            file_path: None,
        });

        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "error"));
        assert!(response.compiled_markdown.contains("Base \\| Case"));
        assert!(response
            .compiled_markdown
            .contains("| Min | Summary | 20 | Formula |"));
        assert!(
            response
                .compiled_markdown
                .contains("| Max | Summary | 1200 | Formula |"),
            "{}",
            response.compiled_markdown
        );
        assert!(response
            .compiled_markdown
            .contains("| Count | Summary | 3 | Formula |"));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Table {
                    id,
                    caption,
                    headers,
                    alignments,
                    rows,
                    ..
                } if id.as_deref() == Some("tbl:scenario")
                    && caption.as_deref() == Some("Scenario grid")
                    && headers == &vec![
                        "Scenario".to_string(),
                        "Owner".to_string(),
                        "Score".to_string(),
                        "Status".to_string()
                    ]
                    && alignments == &vec![
                        "left".to_string(),
                        "center".to_string(),
                        "right".to_string(),
                        "left".to_string()
                    ]
                    && rows.iter().any(|row| row == &vec![
                        "Base | Case".to_string(),
                        "Finance".to_string(),
                        "$1,200.50".to_string(),
                        "Ready".to_string()
                    ])
                    && rows.iter().any(|row| row == &vec![
                        "Max".to_string(),
                        "Summary".to_string(),
                        "1200".to_string(),
                        "Formula".to_string()
                    ])
            )
        }));

        let options = json!({});
        let docx = render_docx_bytes(&response, &options).expect("docx edited table permutation");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains("Scenario grid"));
        assert!(docx_document.contains("Base | Case"));
        assert!(docx_document.contains("1200"));
        assert!(docx_document.contains(r#"<w:jc w:val="center"/>"#));
        assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx edited table permutation");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert!(pptx_slide.contains("Scenario grid"));
        assert!(pptx_slide.contains("<a:t>Base | Case</a:t>"));
        assert!(pptx_slide.contains("<a:t>1200</a:t>"));
        assert!(pptx_slide.contains(r#"<a:pPr algn="ctr"/>"#));
        assert!(pptx_slide.contains(r#"<a:pPr algn="r"/>"#));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("Scenario grid"));
        assert!(pdf_text.contains("(Base | Case) Tj"));
        assert!(pdf_text.contains("(1200) Tj"));

        let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
            .expect("edited table permutation bundle");
        let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
        assert!(bundled_ast.contains("\"id\": \"tbl:scenario\""));
        assert!(bundled_ast.contains("Base | Case"));
        assert!(bundled_ast.contains("1200"));
    }

    #[test]
    fn compiler_renders_layout_break_directives() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Layout\nstatus: approved\napprovedBy: QA\n---\n# Layout\n{{page-break}}\n{{section-break columns=1}}\n\n```layout\ncolumns: 2\n```\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("data-layout=\"page-break\""));
        assert!(response.html.contains("data-layout=\"section-break\""));
        assert!(response.html.contains("columns=1"));
        assert!(response.html.contains("data-layout=\"layout\""));
        assert!(response.html.contains("column-count:2"));
    }

    #[test]
    fn layout_pagination_controls_flow_through_exports() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Flow Layout\nstatus: approved\napprovedBy: QA\n---\n# Flow Layout\n\n```layout\nbreakBefore: page\nkeepWithNext: true\nkeepTogether: true\n```\n## Kept Heading\nKept paragraph.\n\n{{section-break columns=2 breakAfter=page header=\"Flow Header\" footer=\"Flow {{page}}/{{pages}}\"}}\nAfter section.\n".to_string(),
            file_path: None,
        });
        let options = json!({});

        assert!(response.html.contains("break-before:page"));
        assert!(response.html.contains("page-break-before:always"));
        assert!(response.html.contains("break-after:avoid"));
        assert!(response.html.contains("break-inside:avoid"));
        assert!(response.document_ast.blocks.iter().any(|block| matches!(
            block,
            DocumentBlock::Layout {
                directive,
                settings,
                ..
            } if directive == "layout"
                && settings.break_before.as_deref() == Some("page")
                && settings.keep_with_next
                && settings.keep_together
        )));
        assert!(response.document_ast.blocks.iter().any(|block| matches!(
            block,
            DocumentBlock::Layout {
                directive,
                settings,
                ..
            } if directive == "section-break"
                && settings.columns == Some(2)
                && settings.break_after.as_deref() == Some("page")
        )));

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        let docx_header = zip_entry_text(&docx, "word/header2.xml");
        let docx_footer = zip_entry_text(&docx, "word/footer2.xml");
        assert!(docx_document.contains("<w:pageBreakBefore/>"));
        assert!(docx_document.contains("<w:keepNext/>"));
        assert!(docx_document.contains("<w:keepLines/>"));
        assert!(docx_document.contains(r#"<w:cols w:num="2""#));
        assert!(docx_header.contains("Flow Header"));
        assert!(docx_footer.contains(r#"<w:fldSimple w:instr="PAGE">"#));
        assert!(docx_footer.contains(r#"<w:fldSimple w:instr="NUMPAGES">"#));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
        let slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/slide");
        assert!(pptx_app.contains("<Slides>"));
        assert!(slides.iter().any(|slide| slide.contains("Flow Header")));
        assert!(slides
            .iter()
            .any(|slide| slide.contains("Section break: columns=2, breakAfter=page")));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("Layout: breakBefore=page, keepWithNext=true, keepTogether=true"));
        assert!(pdf_text.contains("Section break: columns=2, breakAfter=page"));
        assert!(pdf_text.contains("Flow Header"));

        let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
            .expect("layout bundle");
        let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
        assert!(bundled_ast.contains(r#""break_before": "page""#));
        assert!(bundled_ast.contains(r#""keep_with_next": true"#));
        assert!(bundled_ast.contains(r#""keep_together": true"#));
    }

    #[test]
    fn compiler_renders_callouts_as_semantic_blocks() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Callouts\nstatus: approved\napprovedBy: QA\n---\n# Callouts\n> [!NOTE] Board review\n> Confirm the launch criteria.\n".to_string(),
            file_path: None,
        });

        assert!(response.html.contains("class=\"callout callout-note\""));
        assert!(response.html.contains("Board review"));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Callout { callout_type, title, text, .. }
                    if callout_type == "note"
                        && title == "Board review"
                        && text.contains("Confirm the launch criteria")
            )
        }));

        let options = json!({});
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        assert!(zip_entry_text(&docx, "word/document.xml")
            .contains("Callout: note: Board review: Confirm the launch criteria."));
        let pdf = render_pdf_bytes(&response, &options);
        assert!(String::from_utf8_lossy(&pdf).contains("Callout: note: Board review"));
    }

    #[test]
    fn compiler_builds_document_ast_blocks_for_exports() {
        let response = compile(CompileRequest {
            text: "---\ntitle: AST\nstatus: approved\napprovedBy: QA\n---\n# AST\nBusiness paragraph with **margin** and [source](https://example.com) [@doe2024] {@missing-ref}.\n\n> Quoted evidence\n> with continuation\n\n```js\nconst total = 42;\n```\n\n- First decision\n- Second decision\n\n- [x] Reviewed by finance\n- [ ] Attach signed approval\n\n| Metric | Value |\n| --- | ---: |\n| Total | =SUM(1,2) |\n\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:diagram caption=\"System diagram\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n{{page-break}}\n".to_string(),
            file_path: None,
        });

        assert_eq!(response.document_ast.metadata.title, "AST");
        assert_eq!(response.document_ast.metadata.status, "approved");
        assert!(response
            .document_ast
            .metadata
            .source_hash
            .starts_with("sha256:"));
        assert!(response
            .document_ast
            .blocks
            .iter()
            .any(|block| matches!(block, DocumentBlock::Heading { text, anchor, .. } if text == "AST" && anchor == "ast")));
        assert!(response
            .document_ast
            .blocks
            .iter()
            .any(|block| matches!(block, DocumentBlock::Paragraph { text, inlines, line, end_line, .. }
                if text.contains("Business paragraph with margin")
                    && line == end_line
                    && inlines.iter().any(|node| matches!(node, document_ast::InlineNode::Strong { text } if text == "margin"))
                    && inlines.iter().any(|node| matches!(node, document_ast::InlineNode::Link { text, url } if text == "source" && url == "https://example.com"))
                    && inlines.iter().any(|node| matches!(node, document_ast::InlineNode::Citation { key, .. } if key == "doe2024"))
                    && inlines.iter().any(|node| matches!(node, document_ast::InlineNode::CrossReference { key, .. } if key == "missing-ref"))
            )));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::BlockQuote { text, .. }
                    if text == "Quoted evidence\nwith continuation"
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::CodeBlock { language, code, .. }
                    if language.as_deref() == Some("js") && code.contains("const total = 42;")
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::List { ordered, items, .. }
                    if !ordered
                        && items == &vec![
                            "First decision".to_string(),
                            "Second decision".to_string()
                        ]
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::TaskList { items, .. }
                    if items.len() == 2
                        && items[0].checked
                        && items[0].text == "Reviewed by finance"
                        && !items[1].checked
                        && items[1].text == "Attach signed approval"
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Table { line, end_line, headers, alignments, rows, .. }
                    if headers == &vec!["Metric".to_string(), "Value".to_string()]
                        && alignments == &vec!["left".to_string(), "right".to_string()]
                        && *end_line == *line + 2
                        && rows.iter().any(|row| row == &vec!["Total".to_string(), "3".to_string()])
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Figure { id, caption, .. }
                    if id.as_deref() == Some("fig:diagram")
                        && caption.as_deref() == Some("System diagram")
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Equation { id, text, .. }
                    if id.as_deref() == Some("eq:roi") && text.contains("ROI")
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Layout { directive, .. } if directive == "page-break"
            )
        }));

        let exported = export::export_text(&response, &json!({}));
        assert!(exported.contains("> Quoted evidence\n> with continuation"));
        assert!(exported.contains("```js\nconst total = 42;\n```"));
        assert!(exported.contains("- First decision\n- Second decision"));
        assert!(exported.contains("- [x] Reviewed by finance\n- [ ] Attach signed approval"));
        assert!(exported.contains("Table: Metric | Value"));
        assert!(exported.contains("Figure: fig:diagram: System diagram"));
        assert!(exported.contains("Equation: eq:roi: ROI = Gain / Cost"));
    }

    #[test]
    fn document_ast_preserves_multiple_citation_keys() {
        let response = compile(CompileRequest {
            text: "---\ntitle: AST Citations\nstatus: approved\napprovedBy: QA\ncitationStyle: key\n---\n# AST Citations\nClaim [@porter1985, p. 42; @doe2026].\n\n```bibtex\n@book{porter1985,\n title={Competitive Advantage},\n author={Porter},\n year={1985}\n}\n@article{doe2026,\n title={Evidence Based Reports},\n author={Doe},\n year={2026}\n}\n```\n"
                .to_string(),
            file_path: None,
        });

        let citation = response
            .document_ast
            .blocks
            .iter()
            .find_map(|block| match block {
                DocumentBlock::Paragraph { inlines, .. } => {
                    inlines.iter().find_map(|inline| match inline {
                        document_ast::InlineNode::Citation { key, keys, raw } => {
                            Some((key, keys, raw))
                        }
                        _ => None,
                    })
                }
                _ => None,
            })
            .expect("AST citation inline");

        assert_eq!(citation.0, "porter1985");
        assert_eq!(citation.1.as_slice(), ["porter1985", "doe2026"]);
        assert!(citation
            .2
            .contains("data-citation-keys=\"porter1985 doe2026\""));
    }

    #[test]
    fn compiler_renders_openapi_and_json_schema_tables() {
        let response = compile(CompileRequest {
            text: r#"---
title: API
status: approved
approvedBy: QA
---
# API

```openapi
openapi: 3.1.0
paths:
  /accounts:
    get:
      summary: List accounts
```

```json-schema
{
  "type": "object",
  "required": ["id"],
  "properties": {
    "id": { "type": "string", "description": "Account id" },
    "balance": { "type": "number" }
  }
}
```
"#
            .to_string(),
            file_path: None,
        });

        assert!(response.html.contains("List accounts"));
        assert!(response.html.contains("Account id"));
        assert!(response.html.contains("<td>yes</td>"));
    }

    #[test]
    fn transform_registry_covers_required_first_release_transforms() {
        let engines = list_transform_engines();
        let names = engines
            .iter()
            .filter_map(|engine| engine.get("name").and_then(Value::as_str))
            .collect::<BTreeSet<_>>();
        let pikchr = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("pikchr"))
            .expect("pikchr engine metadata");
        assert_eq!(
            pikchr.get("trustRequired").and_then(Value::as_bool),
            Some(true)
        );
        assert_eq!(pikchr.get("bundled").and_then(Value::as_bool), Some(false));
        assert!(pikchr
            .get("installationLabel")
            .and_then(Value::as_str)
            .is_some_and(|label| label.contains("not bundled")));
        assert!(pikchr
            .get("setupHint")
            .and_then(Value::as_str)
            .is_some_and(|hint| hint.contains("Pikchr executable")));
        assert!(pikchr
            .get("securitySummary")
            .and_then(Value::as_str)
            .is_some_and(|summary| summary.contains("no shell interpolation")));
        assert_eq!(
            pikchr.get("preferenceKey").and_then(Value::as_str),
            Some("transforms.pikchr.path")
        );
        assert!(pikchr
            .pointer("/diagnosticProfile/versionProbe")
            .and_then(Value::as_str)
            .is_some_and(|probe| probe.contains("pikchr --version")));
        assert!(pikchr
            .pointer("/diagnosticProfile/successRelated")
            .and_then(Value::as_array)
            .is_some_and(|fields| fields.iter().any(|field| field == "output_channel")));

        for name in [
            "calc",
            "mermaid",
            "pikchr",
            "dot",
            "graphviz",
            "plantuml",
            "d2",
            "vega-lite",
            "chart",
            "geojson",
            "topojson",
            "stl",
            "csv",
            "tsv",
            "json",
            "yaml",
            "openapi",
            "json-schema",
            "bibtex",
            "glossary",
            "layout",
            "timeline",
            "roadmap",
            "adr",
            "diff",
            "qr",
        ] {
            assert!(
                names.contains(name),
                "missing transform registry entry: {name}"
            );
            assert!(supported_transform(name), "unsupported transform: {name}");
        }

        let response = compile(CompileRequest {
            text:
                "---\ntitle: Diagram\n---\n# Diagram\n```pikchr\nbox \"A\"\narrow\nbox \"B\"\n```\n"
                    .to_string(),
            file_path: None,
        });
        assert!(response.html.contains("transform-pikchr"));
        assert!(response.html.contains("pikchr-arrow"));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Pikchr native preview")));
    }

    #[test]
    fn external_diagram_fallbacks_render_simple_native_svgs() {
        for (name, body, expected) in [
            (
                "dot",
                "digraph { Start -> Review; Review -> Done; }",
                "transform-dot",
            ),
            (
                "graphviz",
                "digraph { a [label=\"Alpha\"]; a -> b; }",
                "Alpha",
            ),
            ("d2", "source -> target: request", "transform-d2"),
            (
                "plantuml",
                "@startuml\nAlice -> Bob: approve\n@enduml\n",
                "transform-plantuml",
            ),
        ] {
            let artifact = run_transform(name.to_string(), body.to_string())
                .unwrap_or_else(|err| panic!("{name} transform failed: {err}"));
            assert_eq!(artifact.output_kind, "svg", "{name} should render SVG");
            assert_eq!(artifact.execution_kind, "embedded");
            assert!(!artifact.html.contains("transform-pending"));
            assert!(artifact.html.contains(expected));
            assert!(artifact.diagnostics.is_empty());
        }
    }

    #[test]
    fn document_ast_models_transform_artifacts_semantically() {
        let response = compile(CompileRequest {
            text: r#"---
title: Transform AST
status: approved
approvedBy: QA
---
# Transform AST

```roadmap
Q1: Launch beta
Q2: Expand exports
```

```timeline
2026-05-19: Semantic AST
```

```mermaid
flowchart LR
  A[Start] --> B[Done]
```
"#
            .to_string(),
            file_path: None,
        });

        let roadmap = response
            .document_ast
            .blocks
            .iter()
            .find_map(|block| match block {
                DocumentBlock::Transform {
                    name,
                    output_kind,
                    text,
                    html,
                    source_hash,
                    output_hash,
                    cache_key,
                    execution_kind,
                    ..
                } if name == "roadmap" => Some((
                    output_kind,
                    text,
                    html,
                    source_hash,
                    output_hash,
                    cache_key,
                    execution_kind,
                )),
                _ => None,
            })
            .expect("roadmap transform block");
        assert_eq!(roadmap.0, "html");
        assert!(roadmap.1.contains("Launch beta"));
        assert!(roadmap.2.contains("transform-roadmap"));
        assert!(roadmap.3.as_deref().is_some_and(|hash| hash.len() == 64));
        assert!(roadmap.4.as_deref().is_some_and(|hash| hash.len() == 64));
        assert!(roadmap.5.as_deref().is_some_and(|key| key.len() == 64));
        assert_eq!(roadmap.6.as_deref(), Some("embedded"));

        let timeline = response
            .document_ast
            .blocks
            .iter()
            .find_map(|block| match block {
                DocumentBlock::Transform {
                    name,
                    output_kind,
                    text,
                    ..
                } if name == "timeline" => Some((output_kind, text)),
                _ => None,
            })
            .expect("timeline transform block");
        assert_eq!(timeline.0, "svg");
        assert!(timeline.1.contains("Semantic AST"));

        let mermaid = response
            .document_ast
            .blocks
            .iter()
            .find_map(|block| match block {
                DocumentBlock::Transform {
                    name,
                    output_kind,
                    text,
                    html,
                    ..
                } if name == "mermaid" => Some((output_kind, text, html)),
                _ => None,
            })
            .expect("mermaid transform block");
        assert_eq!(mermaid.0, "svg");
        assert!(mermaid.1.contains("Start"));
        assert!(mermaid.2.contains("transform-mermaid"));

        let exported = export::export_text(&response, &json!({}));
        assert!(exported.contains("Transform: roadmap"));
        assert!(exported.contains("Transform: mermaid"));
    }

    #[test]
    fn document_ast_parses_multiline_semantic_html_blocks() {
        let response = compile(CompileRequest {
            text: r#"---
title: Multiline HTML AST
---
# Multiline HTML AST

<figure class="figure" id="fig:multi">
<img src="diagram.svg" alt="Diagram">
<figcaption>Multiline caption</figcaption>
</figure>

<section class="transform transform-custom">
<pre>alpha
beta</pre>
</section>
"#
            .to_string(),
            file_path: None,
        });

        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Figure { id, caption, line, end_line, .. }
                    if id.as_deref() == Some("fig:multi")
                        && caption.as_deref() == Some("Multiline caption")
                        && *end_line > *line
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Transform { name, text, line, end_line, .. }
                    if name == "custom"
                        && text.contains("alpha")
                        && text.contains("beta")
                        && *end_line > *line
            )
        }));
    }

    #[test]
    fn qr_transform_renders_static_svg_preview() {
        let artifact = run_transform("qr".to_string(), "https://example.com".to_string())
            .expect("qr transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-qr"));
        assert!(artifact.html.contains("<rect"));
        assert!(artifact.html.contains("QR code for https://example.com"));
        assert!(artifact.diagnostics.is_empty());

        let response = compile(CompileRequest {
            text: "---\ntitle: QR\n---\n# QR\n```qr\nhttps://example.com\n```\n".to_string(),
            file_path: None,
        });
        assert!(response.html.contains("transform-qr"));
        assert!(response
            .transform_artifacts
            .iter()
            .any(|artifact| artifact.name == "qr" && artifact.output_kind == "svg"));
    }

    #[test]
    fn qr_matrix_reserves_finder_separators() {
        let matrix = transforms::qr::render_qr_matrix(b"HELLO").expect("qr matrix");
        assert_eq!(matrix.len(), 21);

        for (row, cells) in matrix.iter().enumerate().take(8) {
            assert!(!cells[13], "top-right finder separator row {row}");
        }
        for (column, cell) in matrix[13].iter().enumerate().take(8) {
            assert!(!cell, "bottom-left finder separator column {column}");
        }
        assert!(transforms::qr::render_qr_matrix(&[b'x'; 79]).is_err());
    }

    #[test]
    fn bibtex_transform_renders_bibliography_preview() {
        let artifact = run_transform(
            "bibtex".to_string(),
            "@book{porter1985, title={Competitive Advantage}}".to_string(),
        )
        .expect("bibtex transform");

        assert_eq!(artifact.output_kind, "html");
        assert!(artifact.html.contains("transform-bibtex"));
        assert!(artifact.html.contains("<dt>porter1985</dt>"));
        assert!(artifact.html.contains("<dd>Competitive Advantage</dd>"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let bibtex = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("bibtex"))
            .expect("bibtex engine metadata");
        assert_eq!(
            bibtex.get("execution").and_then(Value::as_str),
            Some("rust-native")
        );
    }

    #[test]
    fn structured_data_transforms_render_tables_and_trees() {
        let json_artifact = run_transform(
            "json".to_string(),
            r#"[{"region":"East","revenue":120},{"region":"West","revenue":98}]"#.to_string(),
        )
        .expect("json transform");
        assert_eq!(json_artifact.output_kind, "html");
        assert!(json_artifact.html.contains("transform-json"));
        assert!(json_artifact.html.contains("<th>region</th>"));
        assert!(json_artifact.html.contains("<td>East</td>"));
        assert!(json_artifact.diagnostics.is_empty());

        let yaml_artifact = run_transform(
            "yaml".to_string(),
            "api:\n  version: v1\n  endpoints:\n    - /accounts\n".to_string(),
        )
        .expect("yaml transform");
        assert_eq!(yaml_artifact.output_kind, "html");
        assert!(yaml_artifact.html.contains("structured-tree"));
        assert!(yaml_artifact.html.contains("<dt>version</dt>"));
        assert!(yaml_artifact.html.contains("/accounts"));
        assert!(yaml_artifact.diagnostics.is_empty());
    }

    #[test]
    fn chart_transform_renders_yaml_business_chart_specs() {
        let artifact = run_transform(
            "chart".to_string(),
            "type: bar\ntitle: Revenue by Region\ndata:\n  - region: East\n    revenue: 120\n  - region: West\n    revenue: 98\nx: region\ny: revenue\n".to_string(),
        )
        .expect("chart transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-chart"));
        assert!(artifact.html.contains("Revenue by Region"));
        assert!(artifact.html.contains(">East<"));
        assert!(artifact.html.contains(">120<"));
        assert!(artifact.diagnostics.is_empty());
    }

    #[test]
    fn chart_transform_renders_pie_area_and_kpi_specs() {
        let pie = run_transform(
            "chart".to_string(),
            "type: pie\ntitle: Revenue Mix\ndata:\n  - segment: Services\n    revenue: 120\n  - segment: Software\n    revenue: 80\nx: segment\ny: revenue\n".to_string(),
        )
        .expect("pie chart transform");
        assert_eq!(pie.output_kind, "svg");
        assert!(pie.html.contains("Revenue Mix"));
        assert!(pie.html.contains("<path d=\"M 260.0 154.0"));
        assert!(pie.html.contains("Services"));
        assert!(pie.html.contains("(60.0%)"));

        let area = run_transform(
            "chart".to_string(),
            "type: area\ntitle: Pipeline\ndata:\n  - month: May\n    value: 20\n  - month: Jun\n    value: 45\nx: month\ny: value\n".to_string(),
        )
        .expect("area chart transform");
        assert_eq!(area.output_kind, "svg");
        assert!(area.html.contains("<polygon"));
        assert!(area.html.contains("<polyline"));
        assert!(area.html.contains(">Jun<"));

        let kpi = run_transform(
            "chart".to_string(),
            "type: kpi\ntitle: Board KPI\ndata:\n  - metric: NDR\n    value: 118\n  - metric: Target\n    value: 110\nx: metric\ny: value\n".to_string(),
        )
        .expect("kpi chart transform");
        assert_eq!(kpi.output_kind, "svg");
        assert!(kpi.html.contains("Board KPI"));
        assert!(kpi.html.contains(">NDR<"));
        assert!(kpi.html.contains(">118<"));
        assert!(kpi.html.contains("Target: 110"));
    }

    #[test]
    fn timeline_transform_renders_static_svg_preview() {
        let artifact = run_transform(
            "timeline".to_string(),
            "2026-05-18: Kickoff\n2026-06-01: Review\n2026-06-15: Release\n".to_string(),
        )
        .expect("timeline transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-timeline"));
        assert!(artifact.html.contains("Kickoff"));
        assert!(artifact.html.contains("Release"));
        assert!(artifact.diagnostics.is_empty());
    }

    #[test]
    fn business_workflow_transforms_render_static_html() {
        let roadmap = run_transform(
            "roadmap".to_string(),
            "Now: Drafting\nNext: Review\nLater: Publish".to_string(),
        )
        .expect("roadmap transform");
        assert_eq!(roadmap.output_kind, "html");
        assert!(roadmap.html.contains("transform-roadmap"));
        assert!(roadmap.html.contains("Review"));

        let adr = run_transform(
            "adr".to_string(),
            "Status: accepted\nDecision: Use local-first exports".to_string(),
        )
        .expect("adr transform");
        assert_eq!(adr.output_kind, "html");
        assert!(adr.html.contains("transform-adr"));
        assert!(adr.html.contains("Use local-first exports"));

        let diff = run_transform("diff".to_string(), "@@ -1 +1 @@\n-old\n+new".to_string())
            .expect("diff transform");
        assert_eq!(diff.output_kind, "html");
        assert!(diff.html.contains("transform-diff"));
        assert!(diff.html.contains("diff-del"));
        assert!(diff.html.contains("diff-add"));
    }

    #[test]
    fn geojson_transform_renders_static_svg_preview() {
        let artifact = run_transform(
            "geojson".to_string(),
            r#"{"type":"Feature","geometry":{"type":"LineString","coordinates":[[36.80,-1.30],[36.85,-1.26],[36.90,-1.28]]}}"#.to_string(),
        )
        .expect("geojson transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-geojson"));
        assert!(artifact.html.contains("<polyline"));
        assert!(artifact.html.contains("3 coordinates"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let geojson = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("geojson"))
            .expect("geojson engine metadata");
        assert_eq!(
            geojson.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn topojson_transform_renders_static_svg_preview() {
        let artifact = run_transform(
            "topojson".to_string(),
            r#"{"type":"Topology","transform":{"scale":[0.01,0.01],"translate":[36.8,-1.3]},"objects":{},"arcs":[[[0,0],[5,4],[5,-2]]]}"#.to_string(),
        )
        .expect("topojson transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-topojson"));
        assert!(artifact.html.contains("<polyline"));
        assert!(artifact.html.contains("1 arcs"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let topojson = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("topojson"))
            .expect("topojson engine metadata");
        assert_eq!(
            topojson.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn stl_transform_renders_ascii_static_svg_preview() {
        let artifact = run_transform(
            "stl".to_string(),
            "solid test\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 10 0 0\nvertex 0 10 0\nendloop\nendfacet\nendsolid test".to_string(),
        )
        .expect("stl transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-stl"));
        assert!(artifact.html.contains("<polygon"));
        assert!(artifact.html.contains("1 triangles / 3 vertices"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let stl = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("stl"))
            .expect("stl engine metadata");
        assert_eq!(
            stl.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn vega_lite_transform_renders_static_svg_preview() {
        let artifact = run_transform(
            "vega-lite".to_string(),
            r#"{"mark":"bar","title":"Revenue","data":{"values":[{"region":"East","revenue":120},{"region":"West","revenue":98}]},"encoding":{"x":{"field":"region","type":"nominal"},"y":{"field":"revenue","type":"quantitative"}}}"#.to_string(),
        )
        .expect("vega-lite transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-vega-lite"));
        assert!(artifact.html.contains("Revenue"));
        assert!(artifact.html.contains("<rect"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let vega_lite = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("vega-lite"))
            .expect("vega-lite engine metadata");
        assert_eq!(
            vega_lite.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[test]
    fn mermaid_transform_renders_simple_flowchart_svg() {
        let artifact = run_transform(
            "mermaid".to_string(),
            "flowchart TD\nA[Start] --> B{Review}\nB -->|Approve| C[Publish]".to_string(),
        )
        .expect("mermaid transform");

        assert_eq!(artifact.output_kind, "svg");
        assert!(artifact.html.contains("transform-mermaid"));
        assert!(artifact.html.contains("Start"));
        assert!(artifact.html.contains("Publish"));
        assert!(artifact.html.contains("marker-end"));
        assert!(artifact.diagnostics.is_empty());

        let engines = list_transform_engines();
        let mermaid = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("mermaid"))
            .expect("mermaid engine metadata");
        assert_eq!(
            mermaid.get("execution").and_then(Value::as_str),
            Some("rust-native-svg")
        );
    }

    #[cfg(unix)]
    #[test]
    fn external_transforms_are_trust_gated_and_limited() {
        let graphviz = write_executable_script(
            "graphviz-adapter",
            "#!/bin/sh\nprintf '<svg data-args=\"%s\">' \"$*\"\nfor arg in \"$@\"; do if [ -f \"$arg\" ]; then cat \"$arg\"; fi; done\ncat\nprintf '</svg>'\n",
        );
        let graphviz_path = path_to_string(&graphviz);
        let trust_error = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "digraph {}".to_string(),
            engine_path: Some(graphviz_path.clone()),
            trusted: false,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(1024),
        })
        .unwrap_err();
        assert!(trust_error.contains("explicit trust"));

        let unique_body = format!(
            "<svg>{}</svg>",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        );
        let limit_error = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "1234".to_string(),
            engine_path: Some(graphviz_path.clone()),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(3),
            max_output_bytes: Some(1024),
        })
        .unwrap_err();
        assert!(limit_error.contains("above the 3 byte limit"));

        let output_limit_error = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "1234".to_string(),
            engine_path: Some(graphviz_path.clone()),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(3),
        })
        .unwrap_err();
        assert!(output_limit_error.contains("output is"));
        assert!(output_limit_error.contains("above the 3 byte limit"));

        let stdin_artifact = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: unique_body.clone(),
            engine_path: Some(graphviz_path.clone()),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(1024),
        })
        .expect("stdin external transform");
        assert_eq!(stdin_artifact.execution_kind, "external");
        assert_eq!(stdin_artifact.input_mode, "stdin");
        assert!(stdin_artifact.html.contains(&unique_body));
        assert!(!stdin_artifact.cache_key.is_empty());
        let success_diagnostic = stdin_artifact
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("completed"))
            .expect("success diagnostic");
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related == &format!("cache_key: {}", stdin_artifact.cache_key)));
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related == "input_mode: stdin"));
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related == "adapter: graphviz"));
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related == "adapter_args: -Tsvg"));
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related == &format!("input_bytes: {}", unique_body.len())));
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related.starts_with("output_bytes: ")));
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related == "timeout_ms: 1000"));
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related == "output_channel: stdout"));
        assert!(success_diagnostic
            .related
            .iter()
            .any(|related| related == "status: 0"));
        assert!(stdin_artifact
            .engine_version
            .as_deref()
            .is_some_and(|version| version.contains("file-size:")));
        let cached_artifact = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: unique_body.clone(),
            engine_path: Some(graphviz_path.clone()),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(1024),
        })
        .expect("cached stdin external transform");
        assert_eq!(cached_artifact.cache_key, stdin_artifact.cache_key);
        assert_eq!(cached_artifact.output_hash, stdin_artifact.output_hash);
        assert_eq!(
            cached_artifact.engine_version,
            stdin_artifact.engine_version
        );
        assert_eq!(cached_artifact.duration_ms, Some(0));
        assert!(cached_artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("served from cache")));
        assert!(cached_artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .related
                .iter()
                .any(|related| related == &format!("cache_key: {}", cached_artifact.cache_key))
        }));
        transforms::external::clear_external_transform_memory_cache_for_tests();
        let persistent_cached_artifact = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: unique_body,
            engine_path: Some(graphviz_path.clone()),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(1024),
        })
        .expect("persistent cached stdin external transform");
        assert_eq!(
            persistent_cached_artifact.cache_key,
            stdin_artifact.cache_key
        );
        assert_eq!(persistent_cached_artifact.duration_ms, Some(0));
        assert!(persistent_cached_artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("persistent cache")));
        assert!(persistent_cached_artifact
            .diagnostics
            .iter()
            .any(|diagnostic| {
                diagnostic
                    .related
                    .iter()
                    .any(|related| related.starts_with("cached_output_bytes: "))
            }));

        let file_artifact = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "digraph {}".to_string(),
            engine_path: Some(graphviz_path),
            trusted: true,
            input_mode: Some("file".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(1024),
        })
        .expect("file external transform");
        assert_eq!(file_artifact.input_mode, "file");
        assert!(file_artifact.html.contains("digraph"));
        assert!(file_artifact.html.contains("-Tsvg"));
        let _ = fs::remove_file(graphviz);
    }

    #[cfg(unix)]
    #[test]
    fn external_transform_adapters_shape_engine_specific_invocations() {
        let d2 = write_executable_script(
            "d2-adapter",
            "#!/bin/sh\nprintf '<svg data-args=\"%s\">d2</svg>' \"$*\"\n",
        );
        let d2_artifact = run_external_transform(ExternalTransformRequest {
            name: "d2".to_string(),
            body: "source -> target".to_string(),
            engine_path: Some(path_to_string(&d2)),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(2048),
        })
        .expect("d2 adapter transform");
        assert!(d2_artifact.html.contains("data-args=\"- -\""));
        assert!(d2_artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .related
                .iter()
                .any(|related| related == "adapter: d2")
        }));

        let plantuml = write_executable_script(
            "plantuml-adapter",
            "#!/bin/sh\nlast=\"\"\nfor arg in \"$@\"; do last=\"$arg\"; done\nout=\"${last%.*}.svg\"\nprintf '<svg data-args=\"%s\">plantuml sidecar</svg>' \"$*\" > \"$out\"\n",
        );
        let plantuml_artifact = run_external_transform(ExternalTransformRequest {
            name: "plantuml".to_string(),
            body: "@startuml\nAlice -> Bob: hi\n@enduml".to_string(),
            engine_path: Some(path_to_string(&plantuml)),
            trusted: true,
            input_mode: Some("file".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(2048),
        })
        .expect("plantuml file adapter transform");
        assert!(plantuml_artifact.html.contains("plantuml sidecar"));
        assert!(plantuml_artifact.html.contains("-tsvg"));
        assert_eq!(plantuml_artifact.input_mode, "file");
        assert!(plantuml_artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .related
                .iter()
                .any(|related| related == "adapter: plantuml")
        }));
        assert!(plantuml_artifact.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .related
                .iter()
                .any(|related| related == "output_channel: sidecar svg")
        }));

        let engines = list_transform_engines();
        let graphviz = engines
            .iter()
            .find(|engine| engine.get("name").and_then(Value::as_str) == Some("graphviz"))
            .expect("graphviz metadata");
        assert_eq!(
            graphviz.get("defaultCommand").and_then(Value::as_str),
            Some("dot")
        );
        assert!(graphviz
            .get("adapterProfile")
            .and_then(Value::as_str)
            .is_some_and(|profile| profile.contains("Graphviz DOT adapter")));
        assert_eq!(
            graphviz
                .pointer("/diagnosticProfile/versionProbe")
                .and_then(Value::as_str),
            Some("dot -V")
        );

        let _ = fs::remove_file(d2);
        let _ = fs::remove_file(plantuml);
    }

    #[test]
    fn external_transform_conformance_runs_installed_engines() {
        struct EngineCase {
            name: &'static str,
            command: &'static str,
            env_var: &'static str,
            input_mode: &'static str,
            body: String,
        }

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let cases = [
            EngineCase {
                name: "dot",
                command: "dot",
                env_var: "NEDITOR_TEST_DOT",
                input_mode: "stdin",
                body: format!("digraph G {{ start -> done [label=\"{unique}\"]; }}"),
            },
            EngineCase {
                name: "d2",
                command: "d2",
                env_var: "NEDITOR_TEST_D2",
                input_mode: "stdin",
                body: format!("source -> target: {unique}"),
            },
            EngineCase {
                name: "plantuml",
                command: "plantuml",
                env_var: "NEDITOR_TEST_PLANTUML",
                input_mode: "file",
                body: format!("@startuml\nAlice -> Bob: {unique}\n@enduml\n"),
            },
            EngineCase {
                name: "pikchr",
                command: "pikchr",
                env_var: "NEDITOR_TEST_PIKCHR",
                input_mode: "stdin",
                body: format!("box \"{unique}\"; arrow; box \"Done\""),
            },
        ];

        let mut verified = Vec::new();
        let mut skipped = Vec::new();
        for case in cases {
            let Some(path) = installed_command_path(case.env_var, case.command) else {
                skipped.push(case.name);
                continue;
            };
            let artifact = run_external_transform(ExternalTransformRequest {
                name: case.name.to_string(),
                body: case.body,
                engine_path: Some(path_to_string(&path)),
                trusted: true,
                input_mode: Some(case.input_mode.to_string()),
                timeout_ms: Some(15_000),
                max_input_bytes: Some(16_384),
                max_output_bytes: Some(1_048_576),
            })
            .unwrap_or_else(|error| {
                panic!(
                    "{} conformance failed with {}: {error}",
                    case.name,
                    path.display()
                )
            });

            assert_eq!(artifact.execution_kind, "external");
            assert_eq!(artifact.input_mode, case.input_mode);
            assert_eq!(artifact.output_kind, "svg");
            assert!(artifact.html.contains("<svg"));
            let engine_path = path_to_string(&path);
            assert_eq!(artifact.engine_path.as_deref(), Some(engine_path.as_str()));
            assert!(artifact.diagnostics.iter().any(|diagnostic| {
                diagnostic.related.iter().any(|related| {
                    related == &format!("adapter: {}", external_conformance_adapter(case.name))
                })
            }));
            assert!(artifact.diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .related
                    .iter()
                    .any(|related| related.starts_with("engine_version: file-size:"))
            }));
            verified.push(case.name);
        }

        eprintln!(
            "external transform conformance verified: {}; skipped: {}",
            verified.join(", "),
            skipped.join(", ")
        );
        if verified.is_empty() {
            eprintln!("No optional external transform engines were installed; set NEDITOR_TEST_DOT, NEDITOR_TEST_D2, NEDITOR_TEST_PLANTUML, or NEDITOR_TEST_PIKCHR to force a conformance run.");
        }
    }

    fn external_conformance_adapter(name: &str) -> &'static str {
        match name {
            "dot" => "graphviz",
            "d2" => "d2",
            "plantuml" => "plantuml",
            "pikchr" => "pikchr",
            _ => "unknown",
        }
    }

    #[cfg(unix)]
    #[test]
    fn compiler_uses_trusted_external_transform_preferences() {
        let graphviz = write_executable_script(
            "compiler-graphviz-adapter",
            "#!/bin/sh\nprintf '<svg data-args=\"%s\">' \"$*\"\ncat\nprintf '</svg>'\n",
        );
        let response = compile_with_options(
            CompileRequest {
                text: "---\ntitle: External Dot\n---\n# External Dot\n```dot\ndigraph { a -> b }\n```\n"
                    .to_string(),
                file_path: None,
            },
            &json!({
                "transformEnginePaths": { "dot": path_to_string(&graphviz) },
                "trustedTransformEngines": { "dot": true },
                "transformInputModes": { "dot": "stdin" },
                "transformTimeoutMs": 1000
            }),
        );

        let artifact = response
            .transform_artifacts
            .iter()
            .find(|artifact| artifact.name == "dot")
            .expect("dot artifact");
        assert_eq!(artifact.execution_kind, "external");
        assert_eq!(artifact.input_mode, "stdin");
        assert!(artifact
            .engine_path
            .as_deref()
            .is_some_and(|path| path == path_to_string(&graphviz)));
        assert!(artifact.html.contains("digraph { a -> b }"));
        assert!(artifact.html.contains("-Tsvg"));
        assert!(response.html.contains("transform-external"));
        assert!(response.html.contains("transform-dot"));
        let ast_transform = response
            .document_ast
            .blocks
            .iter()
            .find_map(|block| match block {
                DocumentBlock::Transform {
                    name,
                    execution_kind,
                    ..
                } if name == "dot" => Some(execution_kind),
                _ => None,
            })
            .expect("dot AST transform");
        assert_eq!(ast_transform.as_deref(), Some("external"));
        assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("dot external transform completed")));
        let _ = fs::remove_file(graphviz);
    }

    #[test]
    fn compiler_falls_back_when_external_transform_is_untrusted() {
        let cat = Path::new("/bin/cat");
        if !cat.exists() {
            return;
        }
        let response = compile_with_options(
            CompileRequest {
                text: "---\ntitle: Untrusted Dot\n---\n# Untrusted Dot\n```dot\ndigraph { a -> b }\n```\n"
                    .to_string(),
                file_path: None,
            },
            &json!({
                "transformEnginePaths": { "dot": path_to_string(cat) },
                "trustedTransformEngines": { "dot": false }
            }),
        );

        let artifact = response
            .transform_artifacts
            .iter()
            .find(|artifact| artifact.name == "dot")
            .expect("dot artifact");
        assert_eq!(artifact.execution_kind, "embedded");
        assert_eq!(artifact.output_kind, "svg");
        assert!(!artifact.html.contains("transform-pending"));
        assert!(artifact.html.contains("transform-dot"));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Transform { name, execution_kind, .. }
                    if name == "dot" && execution_kind.as_deref() == Some("embedded")
            )
        }));
        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("dot external transform failed")));
    }

    #[cfg(unix)]
    #[test]
    fn external_transform_rejects_non_executable_engine_path() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let script = std::env::temp_dir().join(format!("neditor-not-executable-{unique}.sh"));
        fs::write(&script, "#!/bin/sh\ncat\n").expect("write non-executable script");

        let error = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "digraph {}".to_string(),
            engine_path: Some(path_to_string(&script)),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(1024),
        })
        .unwrap_err();

        let _ = fs::remove_file(script);
        assert!(error.contains("not executable"));
    }

    #[cfg(unix)]
    #[test]
    fn external_transform_timeout_covers_blocked_stdin() {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let script = std::env::temp_dir().join(format!("neditor-blocked-stdin-{unique}.sh"));
        fs::write(&script, "#!/bin/sh\nsleep 2\n").expect("write blocked stdin script");
        let mut permissions = fs::metadata(&script)
            .expect("script metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).expect("make script executable");

        let started = std::time::Instant::now();
        let error = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "x".repeat(512 * 1024),
            engine_path: Some(path_to_string(&script)),
            trusted: true,
            input_mode: Some("stdin".to_string()),
            timeout_ms: Some(50),
            max_input_bytes: Some(1024 * 1024),
            max_output_bytes: Some(1024),
        })
        .unwrap_err();

        let _ = fs::remove_file(script);
        assert!(error.contains("timed out"));
        assert!(
            started.elapsed() < std::time::Duration::from_secs(1),
            "blocked stdin write should not bypass the timeout"
        );
    }

    #[cfg(unix)]
    #[test]
    fn external_transform_exit_errors_include_stderr() {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let script = std::env::temp_dir().join(format!("neditor-stderr-exit-{unique}.sh"));
        fs::write(&script, "#!/bin/sh\necho engine exploded >&2\nexit 7\n")
            .expect("write stderr script");
        let mut permissions = fs::metadata(&script)
            .expect("script metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script, permissions).expect("make script executable");

        let error = run_external_transform(ExternalTransformRequest {
            name: "dot".to_string(),
            body: "digraph {}".to_string(),
            engine_path: Some(path_to_string(&script)),
            trusted: true,
            input_mode: Some("file".to_string()),
            timeout_ms: Some(1000),
            max_input_bytes: Some(1024),
            max_output_bytes: Some(1024),
        })
        .unwrap_err();

        let _ = fs::remove_file(script);
        assert!(error.contains("status 7"));
        assert!(error.contains("engine exploded"));
    }

    #[test]
    fn include_expansion_strips_child_front_matter() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-test-{unique}"));
        let chapter_dir = root.join("chapters");
        fs::create_dir_all(&chapter_dir).expect("create test dirs");
        fs::write(
            chapter_dir.join("intro.md"),
            "---\ntitle: Child\n---\n\n## Included\n\nBody",
        )
        .expect("write include");

        let response = compile(CompileRequest {
            text: "---\ntitle: Root\n---\n\n!include chapters/intro.md\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert!(response.compiled_markdown.contains("## Included"));
        assert!(!response.compiled_markdown.contains("title: Child"));
        assert_eq!(response.include_graph.len(), 1);
        let included_line = response
            .compiled_markdown
            .lines()
            .position(|line| line == "## Included")
            .map(|index| index + 1)
            .expect("included heading line");
        assert!(response.source_map.iter().any(|entry| {
            entry.generated_line == included_line
                && entry.source_file.ends_with("chapters/intro.md")
                && entry.source_line == 2
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Heading { text, source: Some(source), .. }
                    if text == "Included"
                        && source.source_file.ends_with("chapters/intro.md")
                        && source.source_line == 2
                        && source.end_source_line == 2
            )
        }));
        fs::remove_dir_all(root).expect("clean test dirs");
    }

    #[test]
    fn include_expansion_supports_documented_directive_forms() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-include-forms-test-{unique}"));
        fs::create_dir_all(root.join("chapters")).expect("create include forms dirs");
        fs::create_dir_all(root.join("appendices")).expect("create appendices dir");
        fs::write(root.join("chapters").join("intro.md"), "## Bang Include\n")
            .expect("write bang include");
        fs::write(
            root.join("chapters").join("market.md"),
            "## Brace Include\n",
        )
        .expect("write brace include");
        fs::write(
            root.join("appendices").join("financials.md"),
            "## Comment Include\n",
        )
        .expect("write comment include");

        let response = compile(CompileRequest {
            text: "!include chapters/intro.md\n{{include chapters/market.md}}\n<!-- include: appendices/financials.md -->\n".to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
        });

        assert!(response.compiled_markdown.contains("## Bang Include"));
        assert!(response.compiled_markdown.contains("## Brace Include"));
        assert!(response.compiled_markdown.contains("## Comment Include"));
        assert_eq!(response.include_graph.len(), 3);
        assert!(response
            .export_manifest
            .included_files
            .iter()
            .any(|file| file.path.ends_with("chapters/market.md")));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Missing include")));
        fs::remove_dir_all(root).expect("clean include forms test dir");
    }

    #[test]
    fn export_renderers_return_non_empty_artifacts() {
        let response = compile(CompileRequest {
            text: sample_document(),
            file_path: None,
        });

        let html = render_full_html(&response, &json!({ "watermark": "DRAFT" }));
        assert!(html.contains("<!doctype html>"));
        assert!(html.contains("class=\"cover\""));
        assert!(html.contains("class=\"cover-logo\""));
        assert!(html.contains("Page {{page}} of {{pages}}") || html.contains("Page 1 of 1"));
        assert!(html.contains("DRAFT"));
        let options = json!({ "watermark": "DRAFT" });
        let pdf = render_pdf_bytes(&response, &options);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(String::from_utf8_lossy(&pdf).contains("Page 1 of 1"));
        assert!(String::from_utf8_lossy(&pdf).contains("/Title (Test Report)"));
        assert!(String::from_utf8_lossy(&pdf).contains("/Info "));
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        assert!(docx.len() > 100);
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains("Cover: Test Report"));
        assert!(docx_document.contains("Logo: data:image/svg+xml"));
        assert!(docx_document.contains("Watermark: DRAFT"));
        let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
        let docx_core = zip_entry_text(&docx, "docProps/core.xml");
        let docx_app = zip_entry_text(&docx, "docProps/app.xml");
        let docx_custom = zip_entry_text(&docx, "docProps/custom.xml");
        assert!(docx_content_types.contains("custom-properties"));
        assert!(docx_core.contains("<dc:title>Test Report</dc:title>"));
        assert!(docx_core.contains("<cp:category>approved</cp:category>"));
        assert!(docx_app.contains("<Application>NEditor</Application>"));
        assert!(docx_app.contains("<Words>"));
        assert!(docx_app.contains("<AppVersion>"));
        assert!(docx_custom.contains(r#"name="NEditorStatus""#));
        assert!(docx_custom.contains("<vt:lpwstr>approved</vt:lpwstr>"));
        assert!(docx_custom.contains(r#"name="NEditorVersion""#));
        assert!(docx_custom.contains("<vt:lpwstr>1.2.0</vt:lpwstr>"));
        assert!(docx_custom.contains(r#"name="NEditorSourceHash""#));
        let docx_relationships = zip_entry_text(&docx, "_rels/.rels");
        assert!(docx_relationships.contains("metadata/core-properties"));
        assert!(docx_relationships.contains("extended-properties"));
        assert!(docx_relationships.contains("custom-properties"));
        let docx_document_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
        assert!(docx_document_relationships.contains("relationships/header"));
        assert!(docx_document_relationships.contains("relationships/footer"));
        assert!(zip_entry_text(&docx, "word/header1.xml").contains("Test Report"));
        let docx_footer = zip_entry_text(&docx, "word/footer1.xml");
        assert!(docx_footer.contains(r#"w:instr="PAGE""#));
        assert!(docx_footer.contains(r#"w:instr="NUMPAGES""#));
        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        assert!(pptx.len() > 100);
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide1.xml");
        assert!(pptx_slide.contains("Test Report"));
        assert!(pptx_slide.contains(r#"name="Header""#));
        assert!(pptx_slide.contains("Page 1 of 1"));
        let pptx_content_types = zip_entry_text(&pptx, "[Content_Types].xml");
        let pptx_core = zip_entry_text(&pptx, "docProps/core.xml");
        let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
        let pptx_custom = zip_entry_text(&pptx, "docProps/custom.xml");
        assert!(pptx_content_types.contains("custom-properties"));
        assert!(pptx_core.contains("<dc:title>Test Report</dc:title>"));
        assert!(pptx_core.contains("<cp:category>approved</cp:category>"));
        assert!(pptx_app.contains("<Application>NEditor</Application>"));
        assert!(pptx_app.contains("<Slides>"));
        assert!(pptx_app.contains("<Notes>0</Notes>"));
        assert!(pptx_custom.contains(r#"name="NEditorClient""#));
        assert!(pptx_custom.contains("<vt:lpwstr>Acme</vt:lpwstr>"));
        assert!(pptx_custom.contains(r#"name="NEditorSourceHash""#));
        assert!(
            render_markdown_bundle_bytes(&response, &response.export_manifest)
                .expect("bundle bytes")
                .starts_with(b"PK")
        );
    }

    #[test]
    fn semantic_exporters_map_ast_blocks() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Semantic Export\nstatus: approved\napprovedBy: QA\n---\n# Semantic Exports\nBusiness paragraph with [source](https://example.com/report).\n\n- [x] Confirm controls\n- [ ] Final approval\n\n| Metric | Value |\n| --- | ---: |\n| Total | =SUM(1,2) |\n\n![Diagram](data:image/svg+xml;base64,PHN2Zy8+){#fig:diagram caption=\"System diagram\"}\n\n$$\nROI = Gain / Cost\n$$ {#eq:roi}\n\n{{page-break}}\n{{section-break columns=2 header=\"Section Header\" footer=\"Section {{page}}/{{pages}}\"}}\n\n{{slide title=\"Board Review\" layout=\"two-column\" header=\"Slide Header\" footer=\"Slide {{page}}/{{pages}}\" notes=\"Open with risk summary\\nClose with decision ask\"}}\nSlide-specific body.\nSecond column body.\n\n## Appendix\nAfter the break.\n".to_string(),
            file_path: None,
        });
        let options = json!({ "watermark": "DRAFT" });

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
        let docx_section_header = zip_entry_text(&docx, "word/header2.xml");
        let docx_section_footer = zip_entry_text(&docx, "word/footer2.xml");
        let docx_svg = zip_entry_text(&docx, "word/media/image1.svg");
        assert!(docx_content_types.contains(r#"ContentType="image/svg+xml""#));
        assert!(docx_content_types.contains(r#"PartName="/word/header2.xml""#));
        assert!(docx_content_types.contains(r#"PartName="/word/footer2.xml""#));
        assert!(docx_relationships.contains(r#"Id="rIdImage1""#));
        assert!(docx_relationships.contains(r#"Target="media/image1.svg""#));
        assert!(docx_relationships.contains(r#"Id="rIdHyperlink1""#));
        assert!(docx_relationships.contains(r#"Target="https://example.com/report""#));
        assert!(docx_relationships.contains(r#"TargetMode="External""#));
        assert!(docx_relationships.contains(r#"Id="rIdHeader2""#));
        assert!(docx_relationships.contains(r#"Target="header2.xml""#));
        assert!(docx_relationships.contains(r#"Id="rIdFooter2""#));
        assert!(docx_relationships.contains(r#"Target="footer2.xml""#));
        assert!(docx_document.contains(r#"r:embed="rIdImage1""#));
        assert!(docx_document.contains(r#"<w:hyperlink r:id="rIdHyperlink1""#));
        assert!(docx_document.contains(r#"<w:headerReference w:type="default" r:id="rIdHeader2""#));
        assert!(docx_document.contains(r#"<w:footerReference w:type="default" r:id="rIdFooter2""#));
        assert!(docx_section_header.contains("Section Header"));
        assert!(docx_section_footer.contains("Section "));
        assert!(docx_section_footer.contains(r#"<w:fldSimple w:instr="PAGE">"#));
        assert!(docx_section_footer.contains(r#"<w:fldSimple w:instr="NUMPAGES">"#));
        assert_eq!(docx_svg, "<svg/>");
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading1""#));
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading2""#));
        assert!(docx_document.contains("<w:tbl>"));
        assert!(docx_document.contains(r#"<w:jc w:val="right"/>"#));
        assert!(docx_document.contains("[x] Confirm controls"));
        assert!(docx_document.contains("[ ] Final approval"));
        assert!(docx_document.contains(r#"<w:br w:type="page""#));
        assert!(docx_document.contains(r#"<w:cols w:num="2""#));
        assert!(docx_document.contains("System diagram"));
        assert!(docx_document.contains("ROI = Gain / Cost"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let pptx_content_types = zip_entry_text(&pptx, "[Content_Types].xml");
        let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
        let presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
        let slide_two_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide2.xml.rels");
        let slide_four_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide4.xml.rels");
        let slide_four_notes = zip_entry_text(&pptx, "ppt/notesSlides/notesSlide4.xml");
        let pptx_svg = zip_entry_text(&pptx, "ppt/media/image1.svg");
        assert!(pptx_content_types.contains(r#"ContentType="image/svg+xml""#));
        assert!(pptx_content_types.contains("presentationml.notesSlide+xml"));
        assert!(pptx_app.contains("<Notes>1</Notes>"));
        assert!(presentation.contains(r#"r:id="rId2""#));
        let slide_two = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert!(slide_two.contains("Semantic Exports"));
        assert!(slide_two.contains("- [x] Confirm controls"));
        assert!(slide_two.contains("- [ ] Final approval"));
        assert!(slide_two.contains("Table: Metric | Value"));
        assert!(slide_two.contains("<a:tbl>"));
        assert!(slide_two.contains(r#"firstRow="1""#));
        assert!(slide_two.contains(r#"<a:pPr algn="r"/>"#));
        assert!(slide_two.contains("<a:t>Total</a:t>"));
        assert!(slide_two.contains("System diagram"));
        assert!(slide_two.contains("Business paragraph with source (https://example.com/report)."));
        assert!(slide_two.contains(r#"<a:hlinkClick r:id="rIdHyperlink1""#));
        assert!(slide_two.contains(r#"name="Footer""#));
        assert!(slide_two.contains("Page 2 of 5"));
        assert!(slide_two.contains(r#"r:embed="rIdImage1""#));
        assert!(slide_two_relationships.contains(r#"Target="../media/image1.svg""#));
        assert!(slide_two_relationships.contains(r#"Target="https://example.com/report""#));
        assert!(slide_two_relationships.contains(r#"TargetMode="External""#));
        assert_eq!(pptx_svg, "<svg/>");
        let slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
        assert!(slide_three.contains("Section"));
        assert!(slide_three.contains("Section break: columns=2"));
        assert!(slide_three.contains("Section Header"));
        assert!(slide_three.contains("Section 3/5"));
        let slide_four = zip_entry_text(&pptx, "ppt/slides/slide4.xml");
        assert!(slide_four.contains("Board Review"));
        assert!(slide_four.contains("Slide-specific body."));
        assert!(slide_four.contains("Second column body."));
        assert!(slide_four.contains(r#"name="Left Column""#));
        assert!(slide_four.contains(r#"name="Right Column""#));
        assert!(slide_four.contains("Slide Header"));
        assert!(slide_four.contains("Slide 4/5"));
        assert!(slide_four_relationships.contains(r#"Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide""#));
        assert!(slide_four_relationships.contains(r#"Target="../notesSlides/notesSlide4.xml""#));
        assert!(slide_four_notes.contains("Open with risk summary"));
        assert!(slide_four_notes.contains("Close with decision ask"));
        let slide_five = zip_entry_text(&pptx, "ppt/slides/slide5.xml");
        assert!(slide_five.contains("Appendix"));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("/Count 3"));
        assert!(pdf_text.contains("Page 1 of 3"));
        assert!(pdf_text.contains("Page 2 of 3"));
        assert!(pdf_text.contains(" re S"));
        assert!(pdf_text.contains("(Metric) Tj"));
        assert!(pdf_text.contains("(Total) Tj"));
        assert!(pdf_text.contains("- [x] Confirm controls"));
        assert!(pdf_text.contains("- [ ] Final approval"));
        assert!(pdf_text.contains("Section break: columns=2"));
        assert!(pdf_text.contains("Section Header"));
        assert!(pdf_text.contains("Section 3/3"));
        assert!(pdf_text.contains("System diagram"));
        assert!(pdf_text.contains("After the break."));
    }

    #[test]
    fn pdf_export_splits_large_tables_across_pages() {
        let rows = (1..=60)
            .map(|index| format!("| Row {index} | {index} |"))
            .collect::<Vec<_>>()
            .join("\n");
        let response = compile(CompileRequest {
            text: format!(
                "---\ntitle: Large Table\nstatus: approved\napprovedBy: QA\n---\n# Large Table\n\nTable: Row audit {{#tbl:rows}}\n| Label | Value |\n| --- | ---: |\n{rows}\n"
            ),
            file_path: None,
        });

        let pdf = render_pdf_bytes(&response, &json!({}));
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains("/Count 3"));
        assert!(pdf_text.contains("Row audit"));
        assert!(pdf_text.contains("Row audit \\(continued\\)"));
        assert!(pdf_text.contains("(Row 1) Tj"));
        assert!(pdf_text.contains("(Row 60) Tj"));
        assert!(pdf_text.contains("Page 3 of 3"));
    }

    #[test]
    fn pptx_export_can_include_an_agenda_from_options() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Agenda Export\nstatus: approved\napprovedBy: QA\n---\n# Agenda Export\nIntro.\n\n## Market\nBody.\n\n## Finance\nBody.\n".to_string(),
            file_path: None,
        });

        let pptx =
            render_pptx_bytes(&response, &json!({ "includeAgenda": true })).expect("pptx bytes");
        let agenda_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        let body_slide = zip_entry_text(&pptx, "ppt/slides/slide3.xml");

        assert!(agenda_slide.contains("Agenda"));
        assert!(agenda_slide.contains("Agenda Export"));
        assert!(agenda_slide.contains("Market"));
        assert!(agenda_slide.contains("Finance"));
        assert!(body_slide.contains("Agenda Export"));
    }

    #[test]
    fn pptx_export_splits_large_tables_across_slides() {
        let rows = (1..=20)
            .map(|index| format!("| Row {index} | {index} |"))
            .collect::<Vec<_>>()
            .join("\n");
        let response = compile(CompileRequest {
            text: format!(
                "---\ntitle: Large Table Deck\nstatus: approved\napprovedBy: QA\n---\n# Large Table Deck\n\nTable: Row audit {{#tbl:rows}}\n| Label | Value |\n| --- | ---: |\n{rows}\n"
            ),
            file_path: None,
        });

        let pptx = render_pptx_bytes(&response, &json!({})).expect("pptx bytes");
        let presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
        let slide_two = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        let slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
        let slide_four = zip_entry_text(&pptx, "ppt/slides/slide4.xml");
        assert!(presentation.contains(r#"r:id="rId4""#));
        assert!(slide_two.contains("<a:tbl>"));
        assert!(slide_two.contains("Row 1"));
        assert!(slide_two.contains("Row 8"));
        assert!(!slide_two.contains("Row 9"));
        assert!(slide_three.contains("Row audit (continued)"));
        assert!(slide_three.contains("Row 9"));
        assert!(slide_three.contains("Row 16"));
        assert!(slide_four.contains("Row audit (continued)"));
        assert!(slide_four.contains("Row 20"));
    }

    #[test]
    fn export_conformance_fixture_maps_business_features() {
        let response = compile(CompileRequest {
            text: include_str!("../fixtures/export/business_report.md").to_string(),
            file_path: None,
        });
        let options = json!({
            "watermark": "APPROVED",
            "includeGlossary": true,
            "includeComments": true,
            "includeProvenance": true
        });

        assert_eq!(response.semantic.title, "Export Conformance Report");
        assert_eq!(response.semantic.status, "approved");
        assert_eq!(response.export_manifest.document_version, "2.0.0");
        assert!(response
            .semantic
            .citations
            .iter()
            .any(|citation| citation == "porter1985"));
        assert!(response.semantic.glossary.contains_key("ARR"));
        assert!(response.semantic.comments.iter().any(|comment| comment
            .text
            .contains("board-pack export fidelity")
            && comment.state == "resolved"));
        assert!(response
            .semantic
            .change_notes
            .iter()
            .any(|note| note.text.contains("export conformance evidence")));
        assert!(response
            .semantic
            .ai_sources
            .iter()
            .any(|source| source.provider == "OpenAI" && source.status == "human-reviewed"));
        assert!(response
            .semantic
            .ai_sources
            .iter()
            .any(|source| source.prompt_summary == "board-pack synthesis"));
        assert!(response
            .semantic
            .ai_sources
            .iter()
            .any(|source| source.line > 0 && source.reviewed_at == "2026-05-18T12:00:00Z"));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::ReviewComment { comment, .. }
                    if comment.text.contains("board-pack export fidelity")
                        && comment.state == "resolved"
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::ChangeNote { note, .. }
                    if note.text.contains("export conformance evidence")
            )
        }));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::AiSource { provenance, .. }
                    if provenance.provider == "OpenAI"
                        && provenance.model == "gpt-5.4"
                        && provenance.status == "human-reviewed"
            )
        }));
        assert_eq!(response.semantic.tables, 1);
        assert_eq!(response.semantic.figures, 1);
        assert_eq!(response.semantic.equations, 1);
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Layout { directive, .. } if directive == "page-break"
            )
        }));
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "error"));

        let html = render_full_html(&response, &options);
        assert!(html.contains("Board Pack Fixture"));
        assert!(html.contains("APPROVED"));
        assert!(html.contains("Competitive Advantage, p. 42"));
        assert!(html.contains("Reference architecture"));
        assert!(html.contains(r##"<a href="#fig:architecture">Figure architecture</a>"##));
        assert!(html.contains(r##"<a href="#eq:roi">Equation roi</a>"##));
        assert!(html.contains("Competitive Advantage"));
        assert!(html.contains("class=\"export-glossary\""));
        assert!(html.contains("<dt>ARR</dt>"));
        assert!(html.contains("class=\"export-comments\""));
        assert!(html.contains("Verify board-pack export fidelity."));
        assert!(html.contains("Added export conformance evidence."));
        assert!(html.contains("class=\"export-provenance\""));
        assert!(html.contains("gpt-5.4"));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf_text.contains("/Count 6"));
        assert!(pdf_text.contains("/Title (Export Conformance Report)"));
        assert!(pdf_text.contains("/Keywords (approved; 2.0.0; restricted)"));
        assert!(pdf_text.contains("Export Conformance Report | restricted"));
        assert!(pdf_text.contains("Page 6 of 6"));
        assert!(pdf_text.contains("Export Conformance Report"));
        assert!(pdf_text.contains("Competitive Advantage, p. 42"));
        assert!(pdf_text.contains(" re S"));
        assert!(pdf_text.contains("(Region) Tj"));
        assert!(pdf_text.contains("Reference architecture"));
        assert!(pdf_text.contains("Figure architecture"));
        assert!(pdf_text.contains("Equation roi"));
        assert!(pdf_text.contains("Glossary"));
        assert!(pdf_text.contains("Review Comments"));
        assert!(pdf_text.contains("Change Notes"));
        assert!(pdf_text.contains("AI Provenance"));

        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_content_types = zip_entry_text(&docx, "[Content_Types].xml");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
        let docx_header = zip_entry_text(&docx, "word/header1.xml");
        let docx_footer = zip_entry_text(&docx, "word/footer1.xml");
        let docx_comments = zip_entry_text(&docx, "word/comments.xml");
        let docx_app = zip_entry_text(&docx, "docProps/app.xml");
        let docx_svg = zip_entry_text(&docx, "word/media/image1.svg");
        assert!(docx_content_types.contains(r#"ContentType="image/svg+xml""#));
        assert!(docx_content_types.contains(
            r#"ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml""#
        ));
        assert!(docx_content_types.contains(
            r#"ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml""#
        ));
        assert!(docx_relationships.contains(r#"Id="rIdImage1""#));
        assert!(docx_relationships.contains(r#"Target="media/image1.svg""#));
        assert!(docx_relationships.contains(
            r#"Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments""#
        ));
        assert!(docx_document.contains(r#"r:embed="rIdImage1""#));
        assert_eq!(docx_svg, "<svg/>");
        assert!(docx_document.contains(r#"<w:pStyle w:val="Heading1""#));
        assert!(docx_document.contains("w:headerReference"));
        assert!(docx_document.contains("w:footerReference"));
        assert!(docx_document.contains(r#"<w:commentRangeStart w:id="0""#));
        assert!(docx_document.contains(r#"<w:commentReference w:id="0""#));
        assert!(docx_document.contains(r#"<w:commentRangeStart w:id="1""#));
        assert!(docx_document.contains(r#"<w:commentReference w:id="1""#));
        assert!(docx_comments.contains(r#"<w:comment w:id="0" w:author="QA""#));
        assert!(docx_comments.contains("Verify board-pack export fidelity."));
        assert!(docx_comments.contains(r#"<w:comment w:id="1" w:author="QA""#));
        assert!(docx_comments.contains("Change note: Added export conformance evidence."));
        assert!(docx_app.contains("<Application>NEditor</Application>"));
        assert!(docx_app.contains("<Company>Acme Strategy</Company>"));
        let docx_without_comments = render_docx_bytes(
            &response,
            &json!({
                "watermark": "APPROVED",
                "includeGlossary": true,
                "includeComments": false,
                "includeProvenance": true
            }),
        )
        .expect("docx bytes without comments");
        assert!(!zip_has_entry(&docx_without_comments, "word/comments.xml"));
        assert!(
            !zip_entry_text(&docx_without_comments, "[Content_Types].xml")
                .contains("wordprocessingml.comments+xml")
        );
        assert!(docx_header.contains("Export Conformance Report | restricted"));
        assert!(docx_footer.contains(r#"w:instr="PAGE""#));
        assert!(docx_footer.contains(r#"w:instr="NUMPAGES""#));
        assert!(docx_document.contains("<w:tbl>"));
        assert!(docx_document.contains(r#"<w:br w:type="page""#));
        assert!(docx_document.contains("Competitive Advantage, p. 42"));
        assert!(docx_document.contains("Reference architecture"));
        assert!(docx_document.contains("Figure architecture"));
        assert!(docx_document.contains("Equation roi"));
        assert!(docx_document.contains(r#"w:name="fig_architecture""#));
        assert!(docx_document.contains(r#"w:name="eq_roi""#));
        assert!(docx_document.contains(r#"<w:hyperlink w:anchor="fig_architecture""#));
        assert!(docx_document.contains(r#"<w:hyperlink w:anchor="eq_roi""#));
        assert!(docx_document.contains("Competitive Advantage"));
        assert!(docx_document.contains("Annual recurring revenue"));
        assert!(docx_document.contains("Review Comments"));
        assert!(docx_document.contains("Verify board-pack export fidelity."));
        assert!(docx_document.contains("Change Notes"));
        assert!(docx_document.contains("Added export conformance evidence."));
        assert!(docx_document.contains("AI Provenance"));
        assert!(docx_document.contains("gpt-5.4"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let pptx_content_types = zip_entry_text(&pptx, "[Content_Types].xml");
        let pptx_presentation = zip_entry_text(&pptx, "ppt/presentation.xml");
        let pptx_app = zip_entry_text(&pptx, "docProps/app.xml");
        let pptx_agenda_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        let pptx_slide_three = zip_entry_text(&pptx, "ppt/slides/slide3.xml");
        let pptx_slide_three_relationships =
            zip_entry_text(&pptx, "ppt/slides/_rels/slide3.xml.rels");
        let pptx_svg = zip_entry_text(&pptx, "ppt/media/image1.svg");
        let pptx_slide_part_count = zip_entry_count_with_prefix(&pptx, "ppt/slides/slide", ".xml");
        let pptx_media_part_count = zip_entry_count_with_prefix(&pptx, "ppt/media/", "");
        assert!(pptx_content_types.contains(r#"ContentType="image/svg+xml""#));
        assert!(pptx_content_types.contains(
            r#"ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml""#
        ));
        assert!(pptx_presentation.contains(r#"r:id="rId2""#));
        assert!(pptx_app.contains("<Application>NEditor</Application>"));
        assert!(pptx_app.contains(&format!("<Slides>{pptx_slide_part_count}</Slides>")));
        assert_eq!(pptx_media_part_count, 2);
        assert!(pptx_agenda_slide.contains("Agenda"));
        assert!(pptx_agenda_slide.contains("Export Conformance Report"));
        assert!(pptx_agenda_slide.contains("Appendix"));
        assert!(pptx_slide_three.contains("Export Conformance Report"));
        assert!(pptx_slide_three.contains("Competitive Advantage, p. 42"));
        assert!(pptx_slide_three.contains("Figure architecture"));
        assert!(pptx_slide_three.contains("Equation roi"));
        assert!(pptx_slide_three.contains("Table: Region | Revenue | Margin"));
        assert!(pptx_slide_three.contains("<a:tbl>"));
        assert!(pptx_slide_three.contains(r#"<a:pPr algn="r"/>"#));
        assert!(pptx_slide_three.contains("Reference architecture"));
        assert!(pptx_slide_three.contains(r#"name="Header""#));
        assert!(pptx_slide_three.contains(r#"name="Footer""#));
        assert!(pptx_slide_three.contains("Page 3 of 9"));
        assert!(pptx_slide_three.contains(r#"r:embed="rIdImage1""#));
        assert!(pptx_slide_three_relationships.contains(r#"Target="../media/image1.svg""#));
        assert_eq!(pptx_svg, "<svg/>");
        let pptx_glossary_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
            .into_iter()
            .find(|slide| slide.contains("Glossary"))
            .expect("glossary slide");
        assert!(pptx_glossary_slide.contains("Annual recurring revenue"));
        let pptx_comments_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
            .into_iter()
            .find(|slide| slide.contains("Review Comments"))
            .expect("comments slide");
        assert!(pptx_comments_slide.contains("Verify board-pack export fidelity."));
        assert!(pptx_comments_slide.contains("Change Notes"));
        assert!(pptx_comments_slide.contains("Added export conformance evidence."));
        let pptx_provenance_slide = zip_entry_texts_with_prefix(&pptx, "ppt/slides/")
            .into_iter()
            .find(|slide| slide.contains("AI Provenance"))
            .expect("provenance slide");
        assert!(pptx_provenance_slide.contains("gpt-5.4"));

        let exported_text = export::export_text(&response, &options);
        assert!(exported_text.contains("Glossary"));
        assert!(exported_text.contains("ARR: Annual recurring revenue"));
        assert!(exported_text.contains("Review Comments"));
        assert!(exported_text.contains("Change Notes"));
        assert!(exported_text.contains("AI Provenance"));

        let mut bundle_manifest = response.export_manifest.clone();
        bundle_manifest.export_options = options.clone();
        let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
        let bundled_markdown = zip_entry_text(&bundle, "document.md");
        let bundled_text = zip_entry_text(&bundle, "document.txt");
        let bundled_manifest = zip_entry_text(&bundle, "manifest.json");
        let bundled_semantic = zip_entry_text(&bundle, "semantic.json");
        let bundled_metadata = zip_entry_text(&bundle, "metadata.json");
        let bundled_ast = zip_entry_text(&bundle, "document-ast.json");
        let bundled_source_map = zip_entry_text(&bundle, "source-map.json");
        let bundled_diagnostics = zip_entry_text(&bundle, "diagnostics.json");
        let bundled_bibliography = zip_entry_text(&bundle, "bibliography.json");
        let bundled_formula_graph = zip_entry_text(&bundle, "formula-graph.json");
        let bundled_transform_artifacts = zip_entry_text(&bundle, "transform-artifacts.json");
        let bundled_media_map = zip_entry_text(&bundle, "media-map.json");
        let bundled_svg = zip_entry_text(&bundle, "media/image1.svg");
        assert!(bundled_markdown.contains("Competitive Advantage"));
        assert!(bundled_text.contains("Figure: fig:architecture: Reference architecture"));
        assert!(bundled_text.contains("Verify board-pack export fidelity."));
        assert!(bundled_text.contains("OpenAI / gpt-5.4"));
        assert!(bundled_manifest.contains("\"document_title\": \"Export Conformance Report\""));
        assert!(bundled_semantic.contains("\"title\": \"Export Conformance Report\""));
        assert!(bundled_semantic.contains("\"comments\""));
        assert!(bundled_metadata.contains("\"classification\": \"restricted\""));
        assert!(bundled_ast.contains("\"kind\": \"figure\""));
        assert!(bundled_ast.contains("\"source_file\""));
        assert!(bundled_source_map.contains("\"generated_line\""));
        assert!(bundled_source_map.contains("\"source_line\""));
        assert!(bundled_diagnostics.starts_with('['));
        assert!(bundled_bibliography.contains("\"key\": \"porter1985\""));
        assert!(bundled_formula_graph.contains("\"formulas\""));
        assert!(bundled_formula_graph.contains("\"dependencies\""));
        assert!(bundled_transform_artifacts.contains("\"name\": \"glossary\""));
        assert!(bundled_transform_artifacts.contains("\"output_hash\""));
        assert!(bundled_media_map.contains("\"bundle_path\": \"media/image1.svg\""));
        assert!(bundled_media_map.contains("\"hash\": \"sha256:"));
        assert_eq!(bundled_svg, "<svg/>");
    }

    #[test]
    fn markdown_bundle_keeps_duplicate_include_basenames_distinct() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-bundle-includes-{unique}"));
        let north = root.join("north");
        let south = root.join("south");
        fs::create_dir_all(&north).expect("create north include dir");
        fs::create_dir_all(&south).expect("create south include dir");
        let north_section = north.join("section.md");
        let south_section = south.join("section.md");
        fs::write(&north_section, "North section").expect("write north include");
        fs::write(&south_section, "South section").expect("write south include");
        let root_doc = root.join("root.md");
        let response = compile(CompileRequest {
            text: "---\ntitle: Bundle Includes\nstatus: approved\napprovedBy: QA\n---\n# Root\n!include north/section.md\n!include south/section.md\n"
                .to_string(),
            file_path: Some(path_to_string(&root_doc)),
        });

        let bundle =
            render_markdown_bundle_bytes(&response, &response.export_manifest).expect("bundle");
        let north_bundle_path = format!(
            "includes/{}-section.md",
            &sha256_hex(path_to_string(&north_section).as_bytes())[..12]
        );
        let south_bundle_path = format!(
            "includes/{}-section.md",
            &sha256_hex(path_to_string(&south_section).as_bytes())[..12]
        );
        assert_ne!(north_bundle_path, south_bundle_path);
        assert_eq!(zip_entry_text(&bundle, &north_bundle_path), "North section");
        assert_eq!(zip_entry_text(&bundle, &south_bundle_path), "South section");
        let include_map = zip_entry_text(&bundle, "include-map.json");
        assert!(include_map.contains(&format!("\"bundle_path\": \"{north_bundle_path}\"")));
        assert!(include_map.contains(&format!("\"bundle_path\": \"{south_bundle_path}\"")));
        assert!(include_map.contains(&path_to_string(&north_section)));
        assert!(include_map.contains(&path_to_string(&south_section)));

        fs::remove_dir_all(root).expect("clean bundle include fixture");
    }

    #[test]
    fn compiler_tracks_ai_assisted_section_review_status() {
        let source = "---\ntitle: AI Review\nstatus: approved\napprovedBy: QA\n---\n<!-- ai-assisted: status=needs-review | source=ChatGPT | promptSummary=Drafted risk language -->\n# Risk Review\nBody.\n\n<!-- ai-assisted: status=human-reviewed | reviewedBy=Jane Doe | reviewedAt=2026-05-18 | source=Claude | promptSummary=Edited executive summary -->\n## Executive Summary\nReviewed body.\n";
        let response = compile(CompileRequest {
            text: source.to_string(),
            file_path: None,
        });

        assert_eq!(response.semantic.ai_assisted_sections.len(), 2);
        assert_eq!(
            response.semantic.ai_assisted_sections[0].heading,
            "Risk Review"
        );
        assert_eq!(
            response.semantic.ai_assisted_sections[0].prompt_summary,
            "Drafted risk language"
        );
        assert_eq!(
            response.semantic.ai_assisted_sections[1].reviewed_by,
            "Jane Doe"
        );
        assert_eq!(
            response.semantic.ai_assisted_sections[1].heading,
            "Executive Summary"
        );
        assert!(response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("AI-assisted sections that are not human-reviewed")));
        let ai_review_diagnostic = response
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic
                    .message
                    .contains("AI-assisted sections that are not human-reviewed")
            })
            .expect("AI review diagnostic");
        assert_eq!(ai_review_diagnostic.severity, "error");
        assert_eq!(ai_review_diagnostic.line, Some(6));
        assert_eq!(
            ai_review_diagnostic.source_file.as_deref(),
            Some("untitled.md")
        );

        let report = prepare_for_export(PrepareExportRequest {
            text: source.to_string(),
            file_path: None,
            target: "pdf".to_string(),
            options: json!({ "includeProvenance": true }),
        });
        assert!(!report.ready);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("AI-assisted sections that are not human-reviewed")));
    }

    #[test]
    fn compiler_accepts_ai_assisted_section_metadata_aliases() {
        let response = compile(CompileRequest {
            text: "---\ntitle: AI Section Aliases\nstatus: approved\napprovedBy: QA\n---\n<!-- ai-assisted: status=human-reviewed | reviewed_by=Jane Doe | reviewed_at=2026-05-19 | source=OpenAI | prompt_summary=Alias section prompt -->\n# AI Section Aliases\nReviewed body.\n"
                .to_string(),
            file_path: None,
        });

        let section = response
            .semantic
            .ai_assisted_sections
            .first()
            .expect("ai-assisted section");
        assert_eq!(section.reviewed_by, "Jane Doe");
        assert_eq!(section.reviewed_at, "2026-05-19");
        assert_eq!(section.prompt_summary, "Alias section prompt");
        assert!(!response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("AI-assisted sections that are not human-reviewed")));
    }

    #[test]
    fn compiler_accepts_ai_source_metadata_aliases() {
        let response = compile(CompileRequest {
            text: "---\ntitle: AI Source Aliases\nstatus: approved\napprovedBy: QA\n---\n# AI Source Aliases\n```ai-source\nprovider: OpenAI\nmodel: ChatGPT\ndate: 2026-05-18\nprompt_summary: Alias prompt\nreviewer: Jane Doe\nreviewed_at: 2026-05-19T09:00:00Z\nstatus: human-reviewed\n```\n"
                .to_string(),
            file_path: None,
        });

        let source = response
            .semantic
            .ai_sources
            .first()
            .expect("ai source metadata");
        assert_eq!(source.prompt_summary, "Alias prompt");
        assert_eq!(source.reviewed_by, "Jane Doe");
        assert_eq!(source.reviewed_at, "2026-05-19T09:00:00Z");
        assert!(!response.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("AI-assisted sections that are not human-reviewed")));
    }

    #[test]
    fn document_ast_accepts_ai_source_metadata_aliases() {
        let response = compile(CompileRequest {
            text: "---\ntitle: AI AST Aliases\nstatus: approved\napprovedBy: QA\n---\n# AI AST Aliases\n```ai-source\nprovider: OpenAI\nmodel: ChatGPT\ndate: 2026-05-18\nprompt: Alias prompt\nreviewer: Jane Doe\nreviewDate: 2026-05-19T09:00:00Z\nstatus: human-reviewed\n```\n"
                .to_string(),
            file_path: None,
        });

        let ast_source = response
            .document_ast
            .blocks
            .iter()
            .find_map(|block| match block {
                DocumentBlock::AiSource { provenance, .. } => Some(provenance),
                _ => None,
            })
            .expect("ai source AST block");
        assert_eq!(ast_source.prompt_summary, "Alias prompt");
        assert_eq!(ast_source.reviewed_by, "Jane Doe");
        assert_eq!(ast_source.reviewed_at, "2026-05-19T09:00:00Z");
    }

    #[test]
    fn export_packages_local_figure_media_relative_to_source_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-local-media-export-{unique}"));
        let assets = root.join("assets");
        fs::create_dir_all(&assets).expect("create media fixture dir");
        let image = assets.join("diagram.svg");
        fs::write(
            &image,
            "<svg width=\"320\" height=\"180\" viewBox=\"0 0 320 180\"><rect width=\"320\" height=\"180\"/></svg>",
        )
        .expect("write svg");
        let doc = root.join("report.md");
        fs::write(
            &doc,
            "---\ntitle: Local Media\nstatus: approved\napprovedBy: QA\n---\n# Local Media\n![Diagram](assets/diagram.svg){#fig:local caption=\"Local diagram\"}\n",
        )
        .expect("write document");

        let response = compile(CompileRequest {
            text: fs::read_to_string(&doc).expect("read document"),
            file_path: Some(path_to_string(&doc)),
        });
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "error"));
        assert!(response.export_manifest.media_files.iter().any(|file| {
            file.path == path_to_string(&image) && file.hash.starts_with("sha256:")
        }));

        let options = json!({});
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
        let docx_svg = zip_entry_text(&docx, "word/media/image1.svg");
        assert!(docx_document.contains(r#"r:embed="rIdImage1""#));
        assert!(docx_document.contains(r#"<wp:extent cx="3048000" cy="1714500""#));
        assert!(docx_document.contains(r#"<a:ext cx="3048000" cy="1714500""#));
        assert!(docx_relationships.contains(r#"Target="media/image1.svg""#));
        assert!(docx_svg.contains("<rect"));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        let pptx_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide2.xml.rels");
        let pptx_svg = zip_entry_text(&pptx, "ppt/media/image1.svg");
        assert!(pptx_slide.contains(r#"r:embed="rIdImage1""#));
        assert!(pptx_slide.contains(r#"<a:ext cx="3048000" cy="1714500""#));
        assert!(pptx_relationships.contains(r#"Target="../media/image1.svg""#));
        assert!(pptx_svg.contains("<rect"));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains(" 240 135 re S"));
        assert!(pdf_text.contains("Local diagram"));

        let mut bundle_manifest = response.export_manifest.clone();
        bundle_manifest.export_options = options;
        let bundle = render_markdown_bundle_bytes(&response, &bundle_manifest).expect("bundle");
        let media_map = zip_entry_text(&bundle, "media-map.json");
        assert!(media_map.contains(r#""width_px": 320.0"#));
        assert!(media_map.contains(r#""height_px": 180.0"#));

        fs::remove_dir_all(root).expect("clean media export fixture");
    }

    #[test]
    fn export_packages_preserve_figure_cover_fit_crop() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-cover-fit-export-{unique}"));
        let assets = root.join("assets");
        fs::create_dir_all(&assets).expect("create cover fit fixture dir");
        let image = assets.join("square.svg");
        fs::write(
            &image,
            "<svg width=\"320\" height=\"320\" viewBox=\"0 0 320 320\"><rect width=\"320\" height=\"320\"/></svg>",
        )
        .expect("write square svg");
        let doc = root.join("report.md");
        fs::write(
            &doc,
            "---\ntitle: Cover Fit\nstatus: approved\napprovedBy: QA\n---\n# Cover Fit\n![Square](assets/square.svg){#fig:square caption=\"Square crop\" fit=\"cover\"}\n",
        )
        .expect("write document");

        let response = compile(CompileRequest {
            text: fs::read_to_string(&doc).expect("read document"),
            file_path: Some(path_to_string(&doc)),
        });
        assert!(response.html.contains("figure-fit-cover"));
        assert!(response.html.contains("data-fit=\"cover\""));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Figure { id, fit, .. }
                    if id.as_deref() == Some("fig:square")
                        && fit.as_deref() == Some("cover")
            )
        }));
        assert!(export::export_text(&response, &json!({})).contains("fit=cover"));

        let options = json!({});
        let full_html = render_full_html(&response, &options);
        assert!(full_html.contains("figure[data-fit='cover'] img"));

        let docx = render_docx_bytes(&response, &options).expect("docx cover fit");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains(r#"<wp:extent cx="4320000" cy="3240000""#));
        assert!(docx_document.contains(r#"<a:srcRect t="12500" b="12500"/>"#));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx cover fit");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert!(pptx_slide.contains(r#"<a:ext cx="3657600" cy="2057400""#));
        assert!(pptx_slide.contains(r#"<a:srcRect t="21875" b="21875"/>"#));

        let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
            .expect("cover fit bundle");
        let media_map = zip_entry_text(&bundle, "media-map.json");
        assert!(media_map.contains(r#""fit": "cover""#));

        fs::remove_dir_all(root).expect("clean cover fit fixture");
    }

    #[test]
    fn export_packages_preserve_figure_cover_crop_position() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-cover-position-export-{unique}"));
        let assets = root.join("assets");
        fs::create_dir_all(&assets).expect("create cover position fixture dir");
        let image = assets.join("square.svg");
        fs::write(
            &image,
            "<svg width=\"320\" height=\"320\" viewBox=\"0 0 320 320\"><rect width=\"320\" height=\"320\"/></svg>",
        )
        .expect("write square svg");
        let doc = root.join("report.md");
        fs::write(
            &doc,
            "---\ntitle: Cover Position\nstatus: approved\napprovedBy: QA\n---\n# Cover Position\n![Square](assets/square.svg){#fig:square-top caption=\"Top crop\" fit=\"cover\" position=\"top\"}\n",
        )
        .expect("write document");

        let response = compile(CompileRequest {
            text: fs::read_to_string(&doc).expect("read document"),
            file_path: Some(path_to_string(&doc)),
        });
        assert!(response.html.contains("figure-position-top"));
        assert!(response.html.contains("data-position=\"top\""));
        assert!(response.document_ast.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Figure { id, fit, position, .. }
                    if id.as_deref() == Some("fig:square-top")
                        && fit.as_deref() == Some("cover")
                        && position.as_deref() == Some("top")
            )
        }));
        let export_text = export::export_text(&response, &json!({}));
        assert!(export_text.contains("fit=cover"));
        assert!(export_text.contains("position=top"));

        let options = json!({});
        let full_html = render_full_html(&response, &options);
        assert!(full_html.contains("figure[data-position='top'] img"));

        let docx = render_docx_bytes(&response, &options).expect("docx cover position");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains(r#"<a:srcRect t="0" b="25000"/>"#));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx cover position");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert!(pptx_slide.contains(r#"<a:srcRect t="0" b="43750"/>"#));

        let bundle = render_markdown_bundle_bytes(&response, &response.export_manifest)
            .expect("cover position bundle");
        let media_map = zip_entry_text(&bundle, "media-map.json");
        assert!(media_map.contains(r#""position": "top""#));

        fs::remove_dir_all(root).expect("clean cover position fixture");
    }

    #[test]
    fn pptx_repeated_media_keeps_per_figure_crop_settings() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-pptx-repeated-media-{unique}"));
        let assets = root.join("assets");
        fs::create_dir_all(&assets).expect("create repeated media fixture dir");
        let image = assets.join("square.svg");
        fs::write(
            &image,
            "<svg width=\"320\" height=\"320\" viewBox=\"0 0 320 320\"><rect width=\"320\" height=\"320\"/></svg>",
        )
        .expect("write square svg");
        let doc = root.join("report.md");
        fs::write(
            &doc,
            "---\ntitle: Reused Media\nstatus: approved\napprovedBy: QA\n---\n# Reused Media\n![Contain](assets/square.svg){#fig:contain caption=\"Contain\" fit=\"contain\"}\n![Cover](assets/square.svg){#fig:cover caption=\"Cover\" fit=\"cover\" position=\"top\"}\n",
        )
        .expect("write document");

        let response = compile(CompileRequest {
            text: fs::read_to_string(&doc).expect("read document"),
            file_path: Some(path_to_string(&doc)),
        });
        let pptx = render_pptx_bytes(&response, &json!({})).expect("pptx repeated media");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        assert_eq!(pptx_slide.matches("<p:pic>").count(), 2);
        assert!(pptx_slide.contains(r#"<a:srcRect t="0" b="43750"/>"#));

        let pptx_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide2.xml.rels");
        assert_eq!(
            pptx_relationships
                .matches(r#"Target="../media/image1.svg""#)
                .count(),
            1
        );

        fs::remove_dir_all(root).expect("clean repeated media fixture");
    }

    #[test]
    fn export_packages_raster_media_intrinsic_dimensions() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-raster-media-export-{unique}"));
        let assets = root.join("assets");
        fs::create_dir_all(&assets).expect("create media fixture dir");
        let png = assets.join("chart.png");
        fs::write(
            &png,
            [
                0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, b'I', b'H',
                b'D', b'R', 0x00, 0x00, 0x00, 0xc8, 0x00, 0x00, 0x00, 0x64, 0x08, 0x02, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
        )
        .expect("write png");
        let jpg = assets.join("photo.jpg");
        fs::write(
            &jpg,
            [
                0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x01,
                0x00, 0x60, 0x00, 0x60, 0x00, 0x00, 0xff, 0xc0, 0x00, 0x11, 0x08, 0x00, 0x78, 0x00,
                0xf0, 0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00, 0xff, 0xd9,
            ],
        )
        .expect("write jpg");
        let doc = root.join("report.md");
        fs::write(
            &doc,
            "---\ntitle: Raster Media\nstatus: approved\napprovedBy: QA\n---\n# Raster Media\n![Chart](assets/chart.png){#fig:chart caption=\"PNG chart\"}\n\n![Photo](assets/photo.jpg){#fig:photo caption=\"JPEG photo\"}\n",
        )
        .expect("write document");

        let response = compile(CompileRequest {
            text: fs::read_to_string(&doc).expect("read document"),
            file_path: Some(path_to_string(&doc)),
        });
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "error"));

        let options = json!({});
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        let docx_relationships = zip_entry_text(&docx, "word/_rels/document.xml.rels");
        assert!(docx_relationships.contains(r#"Target="media/image1.png""#));
        assert!(docx_relationships.contains(r#"Target="media/image2.jpg""#));
        assert!(docx_document.contains(r#"<wp:extent cx="1905000" cy="952500""#));
        assert!(docx_document.contains(r#"<wp:extent cx="2286000" cy="1143000""#));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        let pptx_slide = zip_entry_text(&pptx, "ppt/slides/slide2.xml");
        let pptx_relationships = zip_entry_text(&pptx, "ppt/slides/_rels/slide2.xml.rels");
        assert!(pptx_relationships.contains(r#"Target="../media/image1.png""#));
        assert!(pptx_relationships.contains(r#"Target="../media/image2.jpg""#));
        assert!(pptx_slide.contains(r#"<a:ext cx="1905000" cy="952500""#));
        assert!(pptx_slide.contains(r#"<a:ext cx="2286000" cy="1143000""#));

        let pdf = render_pdf_bytes(&response, &options);
        let pdf_text = String::from_utf8_lossy(&pdf);
        assert!(pdf_text.contains(" 150 75 re S"));
        assert!(pdf_text.contains(" 180 90 re S"));

        fs::remove_dir_all(root).expect("clean media export fixture");
    }

    #[test]
    fn export_keeps_duplicate_relative_media_from_includes_distinct() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-include-media-export-{unique}"));
        let chapter_a = root.join("a");
        let chapter_b = root.join("b");
        fs::create_dir_all(chapter_a.join("assets")).expect("create chapter a assets");
        fs::create_dir_all(chapter_b.join("assets")).expect("create chapter b assets");
        fs::write(
            chapter_a.join("assets").join("diagram.svg"),
            "<svg><text>A</text></svg>",
        )
        .expect("write a svg");
        fs::write(
            chapter_b.join("assets").join("diagram.svg"),
            "<svg><text>B</text></svg>",
        )
        .expect("write b svg");
        fs::write(
            chapter_a.join("section.md"),
            "## A\n![Diagram](assets/diagram.svg){#fig:a caption=\"A diagram\"}\n",
        )
        .expect("write a section");
        fs::write(
            chapter_b.join("section.md"),
            "## B\n![Diagram](assets/diagram.svg){#fig:b caption=\"B diagram\"}\n",
        )
        .expect("write b section");
        let doc = root.join("root.md");
        fs::write(
            &doc,
            "---\ntitle: Include Media\nstatus: approved\napprovedBy: QA\n---\n# Include Media\n!include a/section.md\n!include b/section.md\n",
        )
        .expect("write root document");

        let response = compile(CompileRequest {
            text: fs::read_to_string(&doc).expect("read root document"),
            file_path: Some(path_to_string(&doc)),
        });
        assert_eq!(response.semantic.figures, 2);
        assert!(!response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "error"));

        let options = json!({});
        let docx = render_docx_bytes(&response, &options).expect("docx bytes");
        assert!(zip_entry_text(&docx, "word/media/image1.svg").contains("<text>A</text>"));
        assert!(zip_entry_text(&docx, "word/media/image2.svg").contains("<text>B</text>"));
        let docx_document = zip_entry_text(&docx, "word/document.xml");
        assert!(docx_document.contains(r#"r:embed="rIdImage1""#));
        assert!(docx_document.contains(r#"r:embed="rIdImage2""#));

        let pptx = render_pptx_bytes(&response, &options).expect("pptx bytes");
        assert!(zip_entry_text(&pptx, "ppt/media/image1.svg").contains("<text>A</text>"));
        assert!(zip_entry_text(&pptx, "ppt/media/image2.svg").contains("<text>B</text>"));
        let slides = zip_entry_texts_with_prefix(&pptx, "ppt/slides/");
        assert!(slides
            .iter()
            .any(|slide| slide.contains(r#"r:embed="rIdImage1""#)));
        assert!(slides
            .iter()
            .any(|slide| slide.contains(r#"r:embed="rIdImage2""#)));

        fs::remove_dir_all(root).expect("clean include media export fixture");
    }

    fn zip_entry_text(bytes: &[u8], path: &str) -> String {
        let cursor = Cursor::new(bytes.to_vec());
        let mut archive = ZipArchive::new(cursor).expect("zip archive");
        let mut entry = archive.by_name(path).expect("zip entry");
        let mut text = String::new();
        entry.read_to_string(&mut text).expect("zip text");
        text
    }

    fn zip_has_entry(bytes: &[u8], path: &str) -> bool {
        let cursor = Cursor::new(bytes.to_vec());
        let mut archive = ZipArchive::new(cursor).expect("zip archive");
        let result = archive.by_name(path).is_ok();
        result
    }

    fn zip_entry_count_with_prefix(bytes: &[u8], prefix: &str, suffix: &str) -> usize {
        let cursor = Cursor::new(bytes.to_vec());
        let mut archive = ZipArchive::new(cursor).expect("zip archive");
        (0..archive.len())
            .filter(|index| {
                let entry = archive.by_index(*index).expect("zip entry by index");
                entry.name().starts_with(prefix) && entry.name().ends_with(suffix)
            })
            .count()
    }

    fn zip_entry_texts_with_prefix(bytes: &[u8], prefix: &str) -> Vec<String> {
        let cursor = Cursor::new(bytes.to_vec());
        let mut archive = ZipArchive::new(cursor).expect("zip archive");
        let mut entries = Vec::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).expect("zip entry by index");
            if !entry.name().starts_with(prefix) || !entry.name().ends_with(".xml") {
                continue;
            }
            let mut text = String::new();
            entry.read_to_string(&mut text).expect("zip text");
            entries.push(text);
        }
        entries
    }

    #[test]
    fn prepare_for_export_blocks_warning_cleanliness() {
        let report = prepare_for_export(PrepareExportRequest {
            text: "---\ntitle: Draft\nstatus: draft\n---\n# Draft".to_string(),
            file_path: None,
            target: "pdf".to_string(),
            options: json!({ "watermark": "DRAFT", "includeManifest": true }),
        });

        assert!(!report.ready);
        assert_eq!(report.error_count, 0);
        assert!(report.warning_count > 0);
    }

    #[test]
    fn export_document_blocks_compiler_errors_before_writing() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-export-block-test-{unique}"));
        fs::create_dir_all(&root).expect("create export block dir");
        let output = root.join("broken.pdf");

        let error = export_document(ExportRequest {
            text:
                "---\ntitle: Broken\nstatus: approved\napprovedBy: QA\n---\n!include missing.md\n"
                    .to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
            target: "pdf".to_string(),
            output_path: path_to_string(&output),
            options: json!({ "includeManifest": true }),
        })
        .expect_err("compiler errors should block export");

        assert!(error.contains("Export blocked by compiler error"));
        assert!(error.contains("Missing include"));
        assert!(!output.exists());
        assert!(!PathBuf::from(format!("{}.manifest.json", output.display())).exists());
        fs::remove_dir_all(root).expect("clean export block dir");
    }

    #[test]
    fn export_document_blocks_invalid_options_before_writing() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-export-options-test-{unique}"));
        fs::create_dir_all(&root).expect("create export options dir");
        let output = root.join("invalid.pdf");

        let error = export_document(ExportRequest {
            text: "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-19\n---\n# Ready\n"
                .to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
            target: "pdf".to_string(),
            output_path: path_to_string(&output),
            options: json!({ "includeManifest": "yes" }),
        })
        .expect_err("invalid export options should block export");

        assert!(error.contains("Export blocked by validation error"));
        assert!(error.contains("includeManifest must be true or false"));
        assert!(!output.exists());
        assert!(!PathBuf::from(format!("{}.manifest.json", output.display())).exists());
        fs::remove_dir_all(root).expect("clean export options dir");
    }

    #[test]
    fn export_document_writes_optional_sidecar_manifest() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-export-manifest-test-{unique}"));
        fs::create_dir_all(&root).expect("create export manifest dir");
        let output = root.join("ready.html");
        let source =
            "---\ntitle: Manifest Ready\nstatus: approved\napprovedBy: QA\napprovedAt: 2026-05-19\nversion: 1.0.0\n---\n# Ready\n";

        let response = export_document(ExportRequest {
            text: source.to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
            target: "html".to_string(),
            output_path: path_to_string(&output),
            options: json!({ "includeManifest": true }),
        })
        .expect("successful html export");

        let manifest_path = response.manifest_path.expect("manifest path");
        let manifest_text = fs::read_to_string(&manifest_path).expect("manifest file");
        assert!(output.exists());
        assert!(manifest_text.contains("\"document_title\": \"Manifest Ready\""));
        assert!(manifest_text.contains("\"document_version\": \"1.0.0\""));
        assert!(manifest_text.contains("\"export_target\": \"html\""));
        assert!(manifest_text.contains("\"source_hash\": \"sha256:"));
        assert!(manifest_text.contains("\"output_path\": "));
        assert!(manifest_text.contains("\"output_hash\": \"sha256:"));
        assert!(manifest_text.contains("\"diagnostics\": []"));
        assert!(manifest_text.contains("\"source_map\": ["));
        assert_eq!(response.manifest.document_title, "Manifest Ready");
        assert_eq!(response.manifest.export_target, "html");
        let output_string = path_to_string(&output);
        assert_eq!(
            response.manifest.output_path.as_deref(),
            Some(output_string.as_str())
        );
        assert!(response
            .manifest
            .output_hash
            .as_deref()
            .is_some_and(|hash| hash.starts_with("sha256:")));
        assert!(response.manifest.diagnostics.is_empty());
        assert!(!response.manifest.source_map.is_empty());

        let docx_output = root.join("ready.docx");
        let docx_response = export_document(ExportRequest {
            text: source.to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
            target: "docx".to_string(),
            output_path: path_to_string(&docx_output),
            options: json!({ "includeManifest": true }),
        })
        .expect("successful docx export with manifest");
        let docx_manifest_path = docx_response.manifest_path.expect("docx manifest path");
        let docx_manifest_text =
            fs::read_to_string(&docx_manifest_path).expect("docx manifest file");
        let docx_bytes = fs::read(&docx_output).expect("docx output bytes");
        assert!(docx_output.exists());
        assert!(docx_bytes.starts_with(b"PK"));
        assert!(zip_has_entry(&docx_bytes, "word/document.xml"));
        assert!(docx_manifest_text.contains("\"export_target\": \"docx\""));
        assert!(docx_manifest_text.contains("\"document_title\": \"Manifest Ready\""));
        assert!(docx_manifest_text.contains("\"output_hash\": \"sha256:"));
        assert_eq!(docx_response.manifest.export_target, "docx");
        assert_eq!(
            docx_response.manifest.output_path.as_deref(),
            Some(path_to_string(&docx_output).as_str())
        );
        assert!(docx_response
            .manifest
            .output_hash
            .as_deref()
            .is_some_and(|hash| hash.starts_with("sha256:")));

        let no_manifest_output = root.join("ready-no-manifest.html");
        let no_manifest = export_document(ExportRequest {
            text: source.to_string(),
            file_path: Some(path_to_string(&root.join("root.md"))),
            target: "html".to_string(),
            output_path: path_to_string(&no_manifest_output),
            options: json!({ "includeManifest": false }),
        })
        .expect("successful html export without manifest");
        assert!(no_manifest_output.exists());
        assert!(no_manifest.manifest_path.is_none());
        assert!(!PathBuf::from(format!("{}.manifest.json", no_manifest_output.display())).exists());

        fs::remove_dir_all(root).expect("clean export manifest dir");
    }

    #[test]
    fn prepare_for_export_validates_target_and_options() {
        let report = prepare_for_export(PrepareExportRequest {
            text: "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\n---\n# Ready".to_string(),
            file_path: None,
            target: "rtf".to_string(),
            options: json!({
                "watermark": 42,
                "includeManifest": "yes",
                "includeStyles": "yes",
                "includeSyntaxHighlighting": "yes",
                "coverPage": "yes",
                "pageNumbers": "yes",
                "layoutPreset": "dense",
                "includeGlossary": "yes",
                "includeComments": "yes",
                "includeProvenance": "yes",
                "includeAgenda": "yes"
            }),
        });

        assert!(!report.ready);
        assert_eq!(report.error_count, 12);
        assert_eq!(report.manifest.export_target, "rtf");
        assert!(report.manifest.output_path.is_none());
        assert!(report.manifest.output_hash.is_none());
        assert_eq!(report.manifest.diagnostics.len(), report.diagnostics.len());
        assert_eq!(report.manifest.source_map.len(), report.source_map.len());
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Unsupported export target")));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("watermark must be a string")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeManifest must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeStyles must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeSyntaxHighlighting must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("coverPage must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("pageNumbers must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("layoutPreset must be business, compact, or presentation")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeGlossary must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeComments must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeProvenance must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("includeAgenda must be true or false")));
    }

    #[test]
    fn prepare_for_export_validates_transform_engine_options() {
        let report = prepare_for_export(PrepareExportRequest {
            text: "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\n---\n# Ready".to_string(),
            file_path: None,
            target: "pdf".to_string(),
            options: json!({
                "transformTimeoutMs": 50000,
                "transformEnginePaths": { "dot": "dot" },
                "trustedTransformEngines": { "dot": "yes" },
                "transformInputModes": { "dot": "pipe" }
            }),
        });

        assert!(!report.ready);
        assert_eq!(report.error_count, 4);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("transformTimeoutMs must be between 1 and 30000")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("transformEnginePaths.dot must be an absolute path")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("trustedTransformEngines.dot must be true or false")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("transformInputModes.dot must be stdin or file")));
    }

    #[test]
    fn prepare_for_export_warns_on_dirty_git_tree() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-export-git-test-{unique}"));
        fs::create_dir_all(&root).expect("create export git test dir");
        run_git(&root, &["init"]).expect("init git repo");
        let doc = root.join("doc.md");
        fs::write(
            &doc,
            "---\ntitle: Ready\nstatus: approved\napprovedBy: QA\n---\n# Ready",
        )
        .expect("write doc");

        let report = prepare_for_export(PrepareExportRequest {
            text: fs::read_to_string(&doc).expect("read doc"),
            file_path: Some(path_to_string(&doc)),
            target: "pdf".to_string(),
            options: json!({ "includeManifest": true }),
        });

        assert!(!report.ready);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("Git working tree is dirty before export")));

        let suppressed = prepare_for_export(PrepareExportRequest {
            text: fs::read_to_string(&doc).expect("read doc"),
            file_path: Some(path_to_string(&doc)),
            target: "pdf".to_string(),
            options: json!({ "includeManifest": true, "warnOnDirtyGit": false }),
        });
        assert!(!suppressed.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("Git working tree is dirty before export")));
        fs::remove_dir_all(root).expect("clean export git test dir");
    }

    #[test]
    fn approved_documents_require_approval_metadata() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Approved\nstatus: approved\n---\n# Approved\n".to_string(),
            file_path: None,
        });

        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("missing approval metadata")));
    }

    #[test]
    fn validation_requires_version_metadata() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Versioned\nstatus: approved\napprovedBy: QA\n---\n# Versioned\n"
                .to_string(),
            file_path: None,
        });

        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Missing version metadata."));
    }

    #[test]
    fn validation_rejects_unknown_release_status() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Status\nversion: 1.0.0\nstatus: final\n---\n# Status\n".to_string(),
            file_path: None,
        });

        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "Invalid document status: final"));
    }

    #[test]
    fn approved_documents_require_approval_timestamp() {
        let response = compile(CompileRequest {
            text: "---\ntitle: Approved\nversion: 1.0.0\nstatus: published\napprovedBy: QA\n---\n# Approved\n".to_string(),
            file_path: None,
        });

        assert!(response
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("missing approval metadata")));
    }

    #[test]
    fn file_duplicate_and_rename_commands_move_content() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-file-test-{unique}"));
        fs::create_dir_all(&root).expect("create test dir");
        let source = root.join("source.md");
        let copy = root.join("copy.md");
        let renamed = root.join("renamed.md");
        fs::write(&source, "hello").expect("write source");

        let duplicated = duplicate_file(DuplicateFileRequest {
            from: path_to_string(&source),
            to: path_to_string(&copy),
        })
        .expect("duplicate file");
        assert_eq!(duplicated.text, "hello");

        let metadata = rename_file(RenameFileRequest {
            from: path_to_string(&copy),
            to: path_to_string(&renamed),
        })
        .expect("rename file");
        assert!(metadata.exists);
        assert!(renamed.exists());
        fs::remove_dir_all(root).expect("clean test dir");
    }

    #[test]
    fn save_file_rejects_stale_expected_hash() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-save-conflict-test-{unique}"));
        fs::create_dir_all(&root).expect("create test dir");
        let doc = root.join("doc.md");
        fs::write(&doc, "external").expect("write external content");

        let result = save_file(SaveFileRequest {
            path: path_to_string(&doc),
            text: "local".to_string(),
            expected_hash: Some(sha256_hex(b"old")),
        });

        assert!(result
            .expect_err("stale save should fail")
            .contains("File changed on disk"));
        assert_eq!(fs::read_to_string(&doc).expect("read doc"), "external");
        fs::remove_dir_all(root).expect("clean save conflict test dir");
    }

    #[test]
    fn stable_file_ipc_aliases_open_save_as_and_watch_paths() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-ipc-alias-test-{unique}"));
        fs::create_dir_all(root.join("chapters")).expect("create chapters");
        fs::create_dir_all(root.join("appendices")).expect("create appendices");
        let doc = root.join("doc.md");
        let included = root.join("chapters").join("intro.md");
        let nested = root.join("appendices").join("risk.md");
        let copy = root.join("copy.md");
        fs::write(&doc, "# Root\n!include chapters/intro.md").expect("write root");
        fs::write(&included, "# Intro\n{{include ../appendices/risk.md}}").expect("write include");
        fs::write(&nested, "# Risk").expect("write nested include");

        let opened = open_file(path_to_string(&doc)).expect("open file alias");
        assert!(opened.text.contains("# Root"));

        let saved = save_file_as(SaveFileRequest {
            path: path_to_string(&copy),
            text: "# Copy".to_string(),
            expected_hash: Some("stale-hash-ignored-for-save-as".to_string()),
        })
        .expect("save file as alias");
        assert_eq!(saved.text, "# Copy");

        let watched = watch_file(WatchFileRequest {
            root: path_to_string(&doc),
            included: vec![path_to_string(&included), path_to_string(&included)],
        })
        .expect("watch file command");
        assert_eq!(watched.paths.len(), 3);
        assert!(watched.paths.iter().all(|metadata| metadata.exists));
        assert_eq!(watched.paths[0].role, "root");
        assert_eq!(watched.paths[1].role, "include");
        assert!(watched
            .paths
            .iter()
            .any(|metadata| metadata.path.ends_with("chapters/intro.md")));
        assert!(watched
            .paths
            .iter()
            .any(|metadata| metadata.path.ends_with("appendices/risk.md")));
        fs::remove_dir_all(root).expect("clean ipc alias test dir");
    }

    #[cfg(feature = "native-watch")]
    #[test]
    fn notify_watcher_ignores_access_only_events() {
        assert!(!notify_event_should_emit(&notify::EventKind::Access(
            notify::event::AccessKind::Any
        )));
        assert!(notify_event_should_emit(&notify::EventKind::Modify(
            notify::event::ModifyKind::Any
        )));
    }

    #[test]
    fn workspace_listing_skips_hidden_and_build_artifacts() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-workspace-test-{unique}"));
        fs::create_dir_all(root.join("chapters")).expect("create chapters");
        fs::create_dir_all(root.join(".git")).expect("create hidden dir");
        fs::create_dir_all(root.join("node_modules")).expect("create node modules");
        fs::write(root.join("root.md"), "# Root").expect("write root doc");
        fs::write(root.join("chapters").join("intro.md"), "# Intro").expect("write child doc");
        fs::write(root.join(".secret.md"), "# Secret").expect("write hidden file");
        fs::write(root.join("node_modules").join("package.md"), "# Dependency")
            .expect("write ignored dependency doc");
        fs::write(root.join("binary.bin"), [0, 1, 2, 3]).expect("write binary");

        let entries = list_workspace_files(WorkspaceFileRequest {
            root: path_to_string(&root),
        })
        .expect("workspace listing");
        let paths = entries
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .collect::<Vec<_>>();

        assert!(paths.contains(&"root.md"));
        assert!(paths.contains(&"chapters"));
        assert!(paths.contains(&"chapters/intro.md"));
        assert!(!paths.iter().any(|path| path.contains(".secret")));
        assert!(!paths.iter().any(|path| path.contains("node_modules")));
        assert!(!paths.iter().any(|path| path.contains("binary.bin")));
        fs::remove_dir_all(root).expect("clean workspace test dir");
    }

    #[test]
    fn git_history_diff_commit_tag_and_restore_workflow() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("neditor-git-test-{unique}"));
        fs::create_dir_all(&root).expect("create git test dir");
        run_git(&root, &["init"]).expect("git init");
        run_git(&root, &["config", "user.email", "neditor@example.test"]).expect("git email");
        run_git(&root, &["config", "user.name", "NEditor Test"]).expect("git name");
        let doc = root.join("doc.md");
        fs::write(&doc, "one\n").expect("write initial doc");
        run_git(&root, &["add", "doc.md"]).expect("git add");
        run_git(&root, &["commit", "-m", "Initial document"]).expect("git commit");
        fs::write(&doc, "two\n").expect("write changed doc");

        let diff = git_diff(GitPathRequest {
            path: path_to_string(&doc),
        })
        .expect("git diff");
        assert!(diff.contains("-one"));
        assert!(diff.contains("+two"));

        commit_document_changes(GitCommitRequest {
            path: path_to_string(&doc),
            message: "Update document".to_string(),
        })
        .expect("commit command");
        let history = git_history(GitPathRequest {
            path: path_to_string(&doc),
        })
        .expect("history command");
        assert!(history.len() >= 2);

        let tag = tag_release(GitTagRequest {
            path: path_to_string(&doc),
            tag: format!("test-{unique}"),
            message: "Test release".to_string(),
        })
        .expect("tag command");
        assert!(tag.starts_with("test-"));

        let restored = restore_git_revision(GitRestoreRequest {
            path: path_to_string(&doc),
            revision: history
                .last()
                .expect("initial history entry")
                .revision
                .clone(),
        })
        .expect("restore revision");
        assert_eq!(restored.text, "one\n");
        fs::remove_dir_all(root).expect("clean git test dir");
    }

    #[test]
    fn ai_cleanup_normalizes_chat_artifacts() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "ChatGPT said:\n• First\tSecond\nA\tB\nRevenue grew 24%.".to_string(),
            add_provenance: true,
            mark_as_draft: true,
            insert_citation_todos: true,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

        assert!(response.cleaned_markdown.contains("- First"));
        assert!(response.cleaned_markdown.contains("| A | B |"));
        assert!(response
            .cleaned_markdown
            .contains("Revenue grew 24%. <!-- TODO: citation needed -->"));
        assert!(response.cleaned_markdown.contains("```ai-source"));
        assert!(response
            .cleaned_markdown
            .contains("ai-assisted: status=needs-review"));
        assert!(response
            .cleaned_markdown
            .contains("promptSummary: AI paste cleanup"));
        assert!(response.issues.len() >= 4);
    }

    #[test]
    fn ai_cleanup_respects_preview_options() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "Assistant:\nClean paragraph.\n```text\nRevenue grew 24%.\n```".to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

        assert!(!response.cleaned_markdown.contains("draft: AI paste"));
        assert!(!response.cleaned_markdown.contains("```ai-source"));
        assert!(!response.cleaned_markdown.contains("TODO: citation needed"));
        assert!(response.provenance_block.is_none());
    }

    #[test]
    fn ai_cleanup_normalizes_chat_list_numbering() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "1) First action\n  ◦ Nested action\n2) Second action\n```text\n1) literal\n◦ literal\n```"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

        assert!(response.cleaned_markdown.contains("1. First action"));
        assert!(response.cleaned_markdown.contains("  - Nested action"));
        assert!(response.cleaned_markdown.contains("2. Second action"));
        assert!(response
            .cleaned_markdown
            .contains("```text\n1) literal\n◦ literal\n```"));
    }

    #[test]
    fn ai_cleanup_removes_chat_labels_without_touching_code_fences() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "DeepSeek said:\nAssistant: Revenue grew 24%.\n```text\nAssistant: literal\nChatGPT said: literal\n```\nYou: ignore this prompt"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

        assert!(!response.cleaned_markdown.contains("DeepSeek said:"));
        assert!(response.cleaned_markdown.contains("Revenue grew 24%."));
        assert!(response
            .cleaned_markdown
            .contains("```text\nAssistant: literal\nChatGPT said: literal\n```"));
        assert!(response.cleaned_markdown.contains("ignore this prompt"));
        assert!(response
            .issues
            .iter()
            .any(|issue| issue.contains("Removed chat labels")));
    }

    #[test]
    fn ai_cleanup_removes_duplicate_markdown_headings() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "## Market Update\n\n## Market Update\nRevenue grew 24%.\n\n```markdown\n## Market Update\n## Market Update\n```"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

        assert_eq!(
            response
                .cleaned_markdown
                .matches("## Market Update")
                .count(),
            3
        );
        assert!(response
            .cleaned_markdown
            .contains("```markdown\n## Market Update\n## Market Update\n```"));
        assert!(response
            .issues
            .iter()
            .any(|issue| issue.contains("duplicated heading")));
    }

    #[test]
    fn ai_cleanup_converts_csv_table_blocks_conservatively() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "Region,Revenue,Growth\nEMEA,1200,24%\nAMER,950,12%\n\nThis sentence, with a comma, should stay prose.\n```csv\nRegion,Revenue\nEMEA,1200\n```"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

        assert!(response
            .cleaned_markdown
            .contains("| Region | Revenue | Growth |\n| --- | --- | --- |\n| EMEA | 1200 | 24% |"));
        assert!(response
            .cleaned_markdown
            .contains("This sentence, with a comma, should stay prose."));
        assert!(response
            .cleaned_markdown
            .contains("```csv\nRegion,Revenue\nEMEA,1200\n```"));
        assert!(response
            .issues
            .iter()
            .any(|issue| issue.contains("comma-separated table")));
    }

    #[test]
    fn ai_cleanup_respects_structure_conversion_options() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text:
                "## Market Update\n\n## Market Update\n1) Review revenue\nRegion,Revenue\nEMEA,1200"
                    .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: false,
            preserve_headings: true,
            convert_numbered_lists: false,
            convert_tables: false,
        });

        assert_eq!(
            response
                .cleaned_markdown
                .matches("## Market Update")
                .count(),
            2
        );
        assert!(response.cleaned_markdown.contains("1) Review revenue"));
        assert!(response.cleaned_markdown.contains("Region,Revenue"));
        assert!(!response.cleaned_markdown.contains("| Region | Revenue |"));
    }

    #[test]
    fn ai_cleanup_normalizes_rich_html_clipboard_content() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "<h2>Board Update</h2><p>Revenue grew 24%. <a href=\"https://example.com/report?x=1&amp;y=2\">Source report</a></p><ul><li>Approve budget</li></ul><table><tr><th>Region</th><th>Revenue</th></tr><tr><td>EMEA</td><td>24</td></tr></table>"
                .to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: true,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

        assert!(response.cleaned_markdown.contains("## Board Update"));
        assert!(response.cleaned_markdown.contains("Revenue grew 24%."));
        assert!(response
            .cleaned_markdown
            .contains("[Source report](https://example.com/report?x=1&y=2)"));
        assert!(response.cleaned_markdown.contains("- Approve budget"));
        assert!(response.cleaned_markdown.contains("| Region | Revenue |"));
        assert!(response.cleaned_markdown.contains("| --- | --- |"));
        assert!(response.cleaned_markdown.contains("| EMEA | 24 |"));
        assert!(response
            .issues
            .iter()
            .any(|issue| issue.contains("Converted rich HTML clipboard")));
    }

    #[test]
    fn ai_cleanup_preserves_code_fence_content() {
        let response = cleanup_ai_paste(AiCleanupRequest {
            text: "Assistant:\n```text\n• literal bullet\nA\tB\nRevenue grew 24%.\n```\n\n• Real bullet\nA\tB\nRevenue grew 24%.".to_string(),
            add_provenance: false,
            mark_as_draft: false,
            insert_citation_todos: true,
            preserve_headings: false,
            convert_numbered_lists: true,
            convert_tables: true,
        });

        assert!(response
            .cleaned_markdown
            .contains("```text\n• literal bullet\nA\tB\nRevenue grew 24%.\n```"));
        assert!(response.cleaned_markdown.contains("- Real bullet"));
        assert!(response.cleaned_markdown.contains("| A | B |"));
        assert!(response
            .cleaned_markdown
            .contains("Revenue grew 24%. <!-- TODO: citation needed -->"));
    }
}
