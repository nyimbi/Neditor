use crate::{
    bibliography::{
        citation_keys_from_references, collect_bibliography, collect_citation_references,
        duplicate_bibliography_keys, render_citations,
    },
    calculations::{collect_calculations, formula_dependency_edges},
    compile_options::apply_compile_options,
    compiler_support::{citation_style, collect_fence_bodies, collect_glossary, extract_headings},
    compiler_types::{
        export_progress_steps, export_readiness_summary, CompileRequest, CompileResponse,
        CompileWithOptionsRequest, ExportManifest, SemanticDocument, SourceMapEntry,
    },
    diagnostics::DocumentDiagnostic,
    document_ast::{
        attach_source_ranges, attach_transform_artifacts, build_document_ast, AstDocumentMetadata,
        DocumentBlock,
    },
    footnotes::render_footnotes,
    front_matter::{merge_project_variables, parse_front_matter, render_front_matter_data_sources},
    generated_sections::{
        generated_figure_list_requested as figure_list_requested_from_metadata,
        generated_glossary_section_requested, generated_index_section_requested,
        generated_table_list_requested as table_list_requested_from_metadata,
        generated_toc_requested as toc_requested_from_metadata, inject_generated_sections,
    },
    html_preview::markdown_to_html,
    indexing::{collect_index_entries, strip_index_markers},
    link_validation::{validate_image_paths, validate_link_paths, validate_logo_path},
    manifest::{
        count_ast_equations, count_ast_figures, manifest_file, manifest_layout_sections,
        manifest_media_files,
    },
    metadata_string,
    paged_document::build_paged_document,
    path_to_string,
    provenance::{collect_ai_assisted_sections, collect_ai_sources},
    references::{collect_cross_references, collect_labels, render_cross_references},
    review::{collect_change_notes, collect_comments},
    rich_blocks::{render_callouts, render_equations, render_figures, render_layout_tokens},
    sha256_uri,
    source_mapping::{
        ast_source_range_for_generated_lines, expand_includes,
        normalize_source_map_after_front_matter,
    },
    tables::{collect_table_summaries, evaluate_markdown_table_formulas},
    transforms::{
        options::TransformExecutionOptions,
        pipeline::apply_transform_fences,
        renderer::{render_transform, supported_transform},
        TransformArtifact,
    },
    validation::{validate_document, validate_layout_directives, DocumentValidationInput},
    variables::interpolate_variables,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
};

#[tauri::command]
pub(crate) fn compile_document(request: CompileRequest) -> Result<CompileResponse, String> {
    Ok(compile(request))
}

#[tauri::command]
pub(crate) fn compile_document_with_options(
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
pub(crate) fn run_transform(name: String, body: String) -> Result<TransformArtifact, String> {
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

pub(crate) fn compile(request: CompileRequest) -> CompileResponse {
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
    let formula_graph = collect_calculations(
        &body,
        &mut calculation_context,
        &source_map,
        &mut diagnostics,
    );
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
    let labels = collect_labels(&interpolated, &headings, &source_map, &mut diagnostics);
    let cross_references =
        collect_cross_references(&interpolated, &labels, &source_map, &mut diagnostics);
    let reference_markdown = render_cross_references(&interpolated, &cross_references);
    let index_entries = collect_index_entries(&interpolated, &metadata, &headings, &glossary);
    let index_terms = index_entries
        .iter()
        .map(|entry| entry.term.clone())
        .collect::<Vec<_>>();
    let generated_figure_list_requested = reference_markdown.contains("[LIST_OF_FIGURES]")
        || figure_list_requested_from_metadata(&metadata);
    let generated_table_list_requested = reference_markdown.contains("[LIST_OF_TABLES]")
        || table_list_requested_from_metadata(&metadata);
    let generated_toc_requested =
        reference_markdown.contains("[TOC]") || toc_requested_from_metadata(&metadata);
    let generated_index_requested =
        reference_markdown.contains("[INDEX]") || generated_index_section_requested(&metadata);
    let generated_glossary_requested = reference_markdown.contains("[GLOSSARY]")
        || generated_glossary_section_requested(&metadata);
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
        &glossary,
    );
    let index_marker_markdown = strip_index_markers(&with_toc);
    let transform_options = TransformExecutionOptions::from_compile_options(options);
    let (transformed_markdown, transform_artifacts) = apply_transforms(
        &index_marker_markdown,
        &source_map,
        &transform_options,
        &mut diagnostics,
    );
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
    validate_layout_directives(&document_ast.blocks, &mut diagnostics);
    let figure_count = document_ast
        .blocks
        .iter()
        .filter(|block| matches!(block, DocumentBlock::Figure { .. }))
        .count();
    let table_count = document_ast
        .blocks
        .iter()
        .filter(|block| matches!(block, DocumentBlock::Table { .. }))
        .count();
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
    let paged_document = build_paged_document(&document_ast);
    validate_document(
        DocumentValidationInput {
            metadata: &metadata,
            citation_references: &citation_references,
            bibliography: &bibliography,
            duplicate_bibliography_keys: &duplicate_bibliography_keys,
            generated_toc_requested,
            heading_count: headings.len(),
            generated_index_requested,
            index_terms: &index_terms,
            generated_glossary_requested,
            glossary_term_count: glossary.len(),
            generated_figure_list_requested,
            figure_count,
            generated_table_list_requested,
            table_count,
            comments: &comments,
            change_notes: &change_notes,
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
    let layout_sections = manifest_layout_sections(&paged_document);
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
        include_graph: include_graph.clone(),
        media_files,
        layout_sections,
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
                    "sourceFile": artifact.source_file.clone(),
                    "sourceLine": artifact.source_line,
                    "endSourceLine": artifact.end_source_line,
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
        progress_steps: export_progress_steps(
            "preview",
            transform_artifacts.len(),
            false,
            None,
            true,
        ),
        readiness: export_readiness_summary(&diagnostics),
        diagnostics: diagnostics.clone(),
        source_map: source_map.clone(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let table_summaries = collect_table_summaries(&document_ast);
    let semantic = SemanticDocument {
        title,
        status,
        headings: headings.clone(),
        outline: headings,
        tables: table_summaries.len(),
        table_summaries,
        figures: count_ast_figures(&document_ast),
        equations: count_ast_equations(&document_ast),
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
        paged_document,
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
    source_map: &[SourceMapEntry],
    options: &TransformExecutionOptions,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> (String, Vec<TransformArtifact>) {
    apply_transform_fences(
        text,
        source_map,
        diagnostics,
        supported_transform,
        |name, body, fence_options, diagnostics| {
            render_transform(name, body, fence_options, options, diagnostics)
        },
    )
}
