use crate::{
    compile_with_options,
    compiler_support::supported_citation_style,
    compiler_types::{
        export_progress_steps, export_readiness_summary, ExportProgressStep,
        ExportReadinessSummary, SemanticDocument,
    },
    diagnostics::{diag, DocumentDiagnostic},
    export::{
        render_blog_publish_package_bytes, render_docx_bytes, render_epub_bytes, render_full_html,
        render_google_docs_package_bytes, render_latex_bytes, render_markdown_bundle_bytes,
        render_pdf_bytes, render_pptx_bytes,
    },
    git::get_git_status,
    metadata_string,
    paged_document::PagedDocument,
    path_to_string, sha256_uri,
    validation::validate_captioned_business_objects,
    CompileRequest, ExportManifest, SourceMapEntry,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub(crate) struct ExportRequest {
    pub(crate) text: String,
    pub(crate) file_path: Option<String>,
    pub(crate) target: String,
    pub(crate) output_path: String,
    pub(crate) options: Value,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PrepareExportRequest {
    pub(crate) text: String,
    pub(crate) file_path: Option<String>,
    pub(crate) target: String,
    pub(crate) options: Value,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportResponse {
    pub(crate) output_path: String,
    pub(crate) manifest_path: Option<String>,
    pub(crate) manifest: ExportManifest,
    pub(crate) diagnostics: Vec<DocumentDiagnostic>,
    pub(crate) progress_steps: Vec<ExportProgressStep>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportReadinessReport {
    pub(crate) ready: bool,
    pub(crate) error_count: usize,
    pub(crate) warning_count: usize,
    pub(crate) info_count: usize,
    pub(crate) readiness: ExportReadinessSummary,
    pub(crate) source_map: Vec<SourceMapEntry>,
    pub(crate) paged_document: PagedDocument,
    pub(crate) diagnostics: Vec<DocumentDiagnostic>,
    pub(crate) manifest: ExportManifest,
    pub(crate) progress_steps: Vec<ExportProgressStep>,
}

#[tauri::command]
pub(crate) fn export_document(request: ExportRequest) -> Result<ExportResponse, String> {
    let file_path = request.file_path.clone();
    let compile_response = compile_with_options(
        CompileRequest {
            text: request.text,
            file_path: request.file_path,
        },
        &request.options,
    );
    let mut manifest = compile_response.export_manifest.clone();
    manifest.export_target = request.target.clone();
    manifest.export_options = request.options.clone();
    if let Some(error) = compile_response
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == "error")
    {
        return Err(format!(
            "Export blocked by compiler error: {}",
            error.message
        ));
    }
    let mut diagnostics = compile_response.diagnostics.clone();
    validate_export_settings(&request.target, &request.options, &mut diagnostics);
    validate_export_output_path(&request.target, &request.output_path, &mut diagnostics);
    validate_target_specific_export_readiness(
        &request.target,
        &compile_response.metadata,
        &mut diagnostics,
    );
    validate_captioned_business_objects(&compile_response.document_ast.blocks, &mut diagnostics);
    validate_content_sensitive_export_options(
        &request.target,
        &request.options,
        &compile_response.semantic,
        &mut diagnostics,
    );
    if git_export_warnings_enabled(&request.options) {
        validate_git_export_cleanliness(file_path.as_deref(), &mut diagnostics);
    }
    if let Some(error) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == "error")
    {
        return Err(format!(
            "Export blocked by validation error: {}",
            error.message
        ));
    }
    manifest.readiness = export_readiness_summary(&diagnostics);
    manifest.diagnostics = diagnostics.clone();

    let output_path = PathBuf::from(&request.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    match request.target.as_str() {
        "html" => fs::write(
            &output_path,
            render_full_html(&compile_response, &request.options),
        )
        .map_err(|err| err.to_string())?,
        "pdf" => fs::write(
            &output_path,
            render_pdf_bytes(&compile_response, &request.options),
        )
        .map_err(|err| err.to_string())?,
        "docx" => fs::write(
            &output_path,
            render_docx_bytes(&compile_response, &request.options)?,
        )
        .map_err(|err| err.to_string())?,
        "pptx" => fs::write(
            &output_path,
            render_pptx_bytes(&compile_response, &request.options)?,
        )
        .map_err(|err| err.to_string())?,
        "markdown-bundle" | "markdown" => fs::write(
            &output_path,
            render_markdown_bundle_bytes(&compile_response, &manifest)?,
        )
        .map_err(|err| err.to_string())?,
        "blog" | "substack" => fs::write(
            &output_path,
            render_blog_publish_package_bytes(&compile_response, &manifest)?,
        )
        .map_err(|err| err.to_string())?,
        "latex" => fs::write(
            &output_path,
            render_latex_bytes(&compile_response, &manifest)?,
        )
        .map_err(|err| err.to_string())?,
        "google-docs" => fs::write(
            &output_path,
            render_google_docs_package_bytes(&compile_response, &manifest)?,
        )
        .map_err(|err| err.to_string())?,
        "epub" => fs::write(
            &output_path,
            render_epub_bytes(&compile_response, &manifest)?,
        )
        .map_err(|err| err.to_string())?,
        other => {
            return Err(format!(
                "Unsupported export target '{other}'. Use html, pdf, docx, pptx, markdown-bundle, blog, substack, latex, google-docs, or epub."
            ));
        }
    }
    let output_bytes = fs::read(&output_path).map_err(|err| err.to_string())?;
    manifest.output_path = Some(path_to_string(&output_path));
    manifest.output_hash = Some(sha256_uri(&output_bytes));
    manifest.progress_steps = export_progress_steps(
        &request.target,
        compile_response.transform_artifacts.len(),
        request
            .options
            .get("includeManifest")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        manifest.output_path.as_deref(),
        true,
    );

    let manifest_path = if request
        .options
        .get("includeManifest")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        let manifest_path = PathBuf::from(format!("{}.manifest.json", output_path.display()));
        let manifest_json =
            serde_json::to_string_pretty(&manifest).map_err(|err| err.to_string())?;
        fs::write(&manifest_path, manifest_json).map_err(|err| err.to_string())?;
        Some(path_to_string(&manifest_path))
    } else {
        None
    };

    Ok(ExportResponse {
        output_path: path_to_string(&output_path),
        manifest_path,
        progress_steps: manifest.progress_steps.clone(),
        manifest,
        diagnostics,
    })
}

#[tauri::command]
pub(crate) fn prepare_for_export(request: PrepareExportRequest) -> ExportReadinessReport {
    let file_path = request.file_path.clone();
    let mut response = compile_with_options(
        CompileRequest {
            text: request.text,
            file_path,
        },
        &request.options,
    );
    response.export_manifest.export_target = request.target.clone();
    response.export_manifest.export_options = request.options.clone();
    validate_export_settings(&request.target, &request.options, &mut response.diagnostics);
    validate_target_specific_export_readiness(
        &request.target,
        &response.metadata,
        &mut response.diagnostics,
    );
    validate_captioned_business_objects(&response.document_ast.blocks, &mut response.diagnostics);
    validate_content_sensitive_export_options(
        &request.target,
        &request.options,
        &response.semantic,
        &mut response.diagnostics,
    );
    if git_export_warnings_enabled(&request.options) {
        validate_git_export_cleanliness(request.file_path.as_deref(), &mut response.diagnostics);
    }
    response.export_manifest.readiness = export_readiness_summary(&response.diagnostics);
    response.export_manifest.diagnostics = response.diagnostics.clone();
    response.export_manifest.progress_steps = export_progress_steps(
        &request.target,
        response.transform_artifacts.len(),
        request
            .options
            .get("includeManifest")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        None,
        false,
    );
    let readiness = response.export_manifest.readiness.clone();
    let progress_steps = response.export_manifest.progress_steps.clone();
    ExportReadinessReport {
        ready: readiness.ready,
        error_count: readiness.error_count,
        warning_count: readiness.warning_count,
        info_count: readiness.info_count,
        readiness,
        source_map: response.source_map,
        paged_document: response.paged_document,
        diagnostics: response.diagnostics,
        manifest: response.export_manifest,
        progress_steps,
    }
}

fn git_export_warnings_enabled(options: &Value) -> bool {
    options
        .get("warnOnDirtyGit")
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

fn validate_git_export_cleanliness(
    file_path: Option<&str>,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(path) = file_path else {
        return;
    };
    if !Path::new(path).exists() {
        return;
    }
    let Ok(status) = get_git_status(Some(path.to_string())) else {
        return;
    };
    if status.inside_repo && status.dirty {
        let summary = if status.summary.is_empty() {
            "working tree has uncommitted changes".to_string()
        } else {
            status.summary.join("; ")
        };
        diagnostics.push(diag(
            "warning",
            format!("Git working tree is dirty before export: {summary}"),
            Some(path.to_string()),
            None,
            Some("Commit, stash, or intentionally document the dirty state before exporting."),
        ));
    }
}

fn validate_export_settings(
    target: &str,
    options: &Value,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if !matches!(
        target,
        "html"
            | "pdf"
            | "docx"
            | "pptx"
            | "markdown-bundle"
            | "markdown"
            | "blog"
            | "substack"
            | "latex"
            | "google-docs"
            | "epub"
    ) {
        diagnostics.push(diag(
            "error",
            format!("Unsupported export target: {target}"),
            None,
            None,
            Some("Use html, pdf, docx, pptx, markdown-bundle, blog, substack, latex, google-docs, or epub."),
        ));
    }
    validate_optional_string(options, "watermark", "Export watermark", diagnostics);
    validate_optional_string(options, "htmlDescription", "htmlDescription", diagnostics);
    validate_url_option(options, "canonicalUrl", "canonicalUrl", diagnostics);
    validate_language_option(options, "htmlLanguage", "htmlLanguage", diagnostics);
    validate_language_option(options, "language", "language", diagnostics);
    validate_brand_color_option(options, diagnostics);
    validate_default_citation_style_option(options, diagnostics);
    validate_default_brand_profile_option(options, diagnostics);
    for option in [
        "includeManifest",
        "includeStyles",
        "includeSyntaxHighlighting",
        "coverPage",
        "pageNumbers",
        "includeGlossary",
        "includeComments",
        "includeProvenance",
        "includeAgenda",
        "warnOnDirtyGit",
        "includeCoverPage",
        "includePageNumbers",
    ] {
        if options.get(option).is_some_and(|value| !value.is_boolean()) {
            diagnostics.push(diag(
                "error",
                format!("{option} must be true or false."),
                None,
                None,
                Some("Use boolean values for export options."),
            ));
        }
    }
    if let Some(layout_preset) = options.get("layoutPreset") {
        let valid = layout_preset
            .as_str()
            .is_some_and(|value| matches!(value, "business" | "compact" | "presentation"));
        if !valid {
            diagnostics.push(diag(
                "error",
                "layoutPreset must be business, compact, or presentation.",
                None,
                None,
                Some("Use one of the supported layout preset names."),
            ));
        }
    }
    validate_transform_export_settings(options, diagnostics);
    validate_target_specific_export_options(target, options, diagnostics);
}

fn validate_optional_string(
    options: &Value,
    key: &str,
    label: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) -> Option<String> {
    let value = options.get(key)?;
    let Some(text) = value.as_str() else {
        diagnostics.push(diag(
            "error",
            format!("{label} must be a string."),
            None,
            None,
            Some("Use a text value or remove the option."),
        ));
        return None;
    };
    Some(text.to_string())
}

fn validate_url_option(
    options: &Value,
    key: &str,
    label: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(value) = validate_optional_string(options, key, label, diagnostics) else {
        return;
    };
    if !value.trim().is_empty() && !is_http_url(&value) {
        diagnostics.push(diag(
            "error",
            format!("{label} must be an absolute http:// or https:// URL."),
            None,
            None,
            Some(
                "Use the final public URL for this document or remove the field until it is known.",
            ),
        ));
    }
}

fn validate_language_option(
    options: &Value,
    key: &str,
    label: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(value) = validate_optional_string(options, key, label, diagnostics) else {
        return;
    };
    if !value.trim().is_empty() && !is_language_tag(&value) {
        diagnostics.push(diag(
            "error",
            format!("{label} must be a valid BCP-47-style language tag."),
            None,
            None,
            Some("Use values such as en, en-US, fr, or pt-BR."),
        ));
    }
}

fn validate_brand_color_option(options: &Value, diagnostics: &mut Vec<DocumentDiagnostic>) {
    let Some(color) = validate_optional_string(options, "brandColor", "brandColor", diagnostics)
    else {
        return;
    };
    if !color.trim().is_empty() && !is_hex_color(&color) {
        diagnostics.push(diag(
            "error",
            "brandColor must be a hex color such as #275DA8.",
            None,
            None,
            Some("Use the color picker value or remove the option."),
        ));
    }
}

fn validate_default_citation_style_option(
    options: &Value,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(value) = options.get("defaultCitationStyle") else {
        return;
    };
    let Some(style) = value.as_str() else {
        diagnostics.push(diag(
            "error",
            "defaultCitationStyle must be a string.",
            None,
            None,
            Some("Use title, author-year, key, numeric, or a supported CSL alias."),
        ));
        return;
    };
    if !supported_citation_style(style) {
        diagnostics.push(diag(
            "error",
            "defaultCitationStyle must be a supported citation style.",
            None,
            None,
            Some("Choose title, author-year, key, numeric, apa, chicago-author-date, ieee, vancouver, or remove the default."),
        ));
    }
}

fn validate_default_brand_profile_option(
    options: &Value,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(value) = options.get("defaultBrandProfile") else {
        return;
    };
    let Some(fields) = value.as_object() else {
        diagnostics.push(diag(
            "error",
            "defaultBrandProfile must be an object.",
            None,
            None,
            Some("Use brand profile fields such as name, color, logo, font, header, and footer."),
        ));
        return;
    };
    for key in [
        "name",
        "logo",
        "font",
        "header",
        "footer",
        "watermark",
        "legalDisclaimer",
    ] {
        if fields.get(key).is_some_and(|field| !field.is_string()) {
            diagnostics.push(diag(
                "error",
                format!("defaultBrandProfile.{key} must be a string."),
                None,
                None,
                Some("Use string values for brand profile defaults."),
            ));
        }
    }
    if let Some(color) = fields.get("color") {
        let Some(color) = color.as_str() else {
            diagnostics.push(diag(
                "error",
                "defaultBrandProfile.color must be a string.",
                None,
                None,
                Some("Use a hex color string such as #275DA8."),
            ));
            return;
        };
        if !color.trim().is_empty() && !is_hex_color(color) {
            diagnostics.push(diag(
                "error",
                "defaultBrandProfile.color must be a hex color such as #275DA8.",
                None,
                None,
                Some("Use the brand color picker value or remove the default color."),
            ));
        }
    }
}

fn is_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    matches!(bytes.len(), 4 | 7)
        && bytes.first() == Some(&b'#')
        && bytes[1..].iter().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_target_specific_export_options(
    target: &str,
    options: &Value,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let sidecar_manifest_disabled =
        options.get("includeManifest").and_then(Value::as_bool) == Some(false);

    if target != "pptx" && bool_option_enabled(options, "includeAgenda") {
        push_option_info(
            target,
            "includeAgenda",
            "includeAgenda is only used for PPTX exports.",
            "Disable includeAgenda for this target or switch the export target to pptx.",
            diagnostics,
        );
    }

    if matches!(
        target,
        "markdown-bundle" | "markdown" | "blog" | "substack" | "google-docs" | "epub"
    ) {
        if sidecar_manifest_disabled {
            push_option_info(
                target,
                "includeManifest",
                "includeManifest=false disables the sidecar manifest, but package exports still embed manifest.json.",
                "Enable includeManifest when you also need a sidecar manifest with final output path and hash evidence.",
                diagnostics,
            );
        }
        for option in [
            "includeStyles",
            "includeSyntaxHighlighting",
            "coverPage",
            "pageNumbers",
        ] {
            if bool_option_enabled(options, option) {
                push_option_info(
                    target,
                    option,
                    &format!("{option} is recorded in the package manifest but does not render package content."),
                    "Keep the option for manifest parity or disable it to reduce package export noise.",
                    diagnostics,
                );
            }
        }
    } else if sidecar_manifest_disabled {
        push_option_info(
            target,
            "includeManifest",
            "includeManifest=false disables the sidecar audit manifest for this export target.",
            "Enable includeManifest when the exported artifact needs separate source hash, option, diagnostic, output path, and output hash evidence.",
            diagnostics,
        );
    }
}

fn validate_content_sensitive_export_options(
    target: &str,
    options: &Value,
    semantic: &SemanticDocument,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if bool_option_enabled(options, "includeGlossary") && semantic.glossary.is_empty() {
        push_option_info(
            target,
            "includeGlossary",
            "includeGlossary is enabled but the document has no glossary entries.",
            "Add glossary entries or disable includeGlossary for this export.",
            diagnostics,
        );
    }
    if bool_option_enabled(options, "includeComments")
        && semantic.comments.is_empty()
        && semantic.change_notes.is_empty()
    {
        push_option_info(
            target,
            "includeComments",
            "includeComments is enabled but the document has no review comments or change notes.",
            "Add review comments/change notes or disable includeComments for this export.",
            diagnostics,
        );
    }
    if bool_option_enabled(options, "includeProvenance")
        && semantic.ai_sources.is_empty()
        && semantic.ai_assisted_sections.is_empty()
    {
        push_option_info(
            target,
            "includeProvenance",
            "includeProvenance is enabled but the document has no AI provenance entries.",
            "Add ai-source or ai-assisted metadata, or disable includeProvenance for this export.",
            diagnostics,
        );
    }
}

fn push_option_info(
    target: &str,
    option: &str,
    message: &str,
    suggestion: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let mut diagnostic = diag("info", message, None, None, Some(suggestion));
    diagnostic.related.push(format!("target:{target}"));
    diagnostic.related.push(format!("option:{option}"));
    diagnostics.push(diagnostic);
}

fn bool_option_enabled(options: &Value, option: &str) -> bool {
    options
        .get(option)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn validate_export_output_path(
    target: &str,
    output_path: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(expected_extension) = expected_export_extension(target) else {
        return;
    };
    let actual_extension = Path::new(output_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if actual_extension == expected_extension {
        return;
    }
    let suggestion = format!(
        "Choose an output path ending in .{} or change the export target.",
        expected_extension
    );
    diagnostics.push(diag(
        "error",
        format!(
            "{} export target must write to .{} files.",
            target.to_ascii_uppercase(),
            expected_extension
        ),
        Some(output_path.to_string()),
        None,
        Some(&suggestion),
    ));
}

fn expected_export_extension(target: &str) -> Option<&'static str> {
    match target {
        "html" => Some("html"),
        "pdf" => Some("pdf"),
        "docx" => Some("docx"),
        "pptx" => Some("pptx"),
        "latex" => Some("tex"),
        "markdown-bundle" | "markdown" | "blog" | "substack" | "google-docs" => Some("zip"),
        "epub" => Some("epub"),
        _ => None,
    }
}

fn validate_target_specific_export_readiness(
    target: &str,
    metadata: &Value,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    validate_distribution_metadata(target, metadata, diagnostics);

    if !release_metadata_required_for_target(target) {
        return;
    }

    let status = metadata_string(metadata, "status").unwrap_or_else(|| "draft".to_string());
    let normalized_status = status.trim().to_ascii_lowercase();
    let approval_reviewer_missing = metadata_string(metadata, "approvedBy")
        .or_else(|| metadata_string(metadata, "reviewer"))
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    let approved_at_missing = metadata_string(metadata, "approvedAt")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    let owner_missing = metadata_string(metadata, "owner")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    let release_target_missing = metadata_string(metadata, "releaseTarget")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    if !matches!(normalized_status.as_str(), "approved" | "published")
        || approval_reviewer_missing
        || approved_at_missing
        || owner_missing
        || release_target_missing
    {
        let mut diagnostic = diag(
            "error",
            &format!("{} export requires release approval metadata before writing.", target.to_ascii_uppercase()),
            None,
            None,
            Some("Set status to approved or published and add approvedBy or reviewer, approvedAt, owner, and releaseTarget before distribution."),
        );
        diagnostic.related.push(format!("target:{target}"));
        diagnostic.related.push(format!("status:{status}"));
        if approval_reviewer_missing {
            diagnostic
                .related
                .push("missing:approvedBy-or-reviewer".to_string());
        }
        if approved_at_missing {
            diagnostic.related.push("missing:approvedAt".to_string());
        }
        if owner_missing {
            diagnostic.related.push("missing:owner".to_string());
        }
        if release_target_missing {
            diagnostic.related.push("missing:releaseTarget".to_string());
        }
        diagnostics.push(diagnostic);
    }
}

fn validate_distribution_metadata(
    target: &str,
    metadata: &Value,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if matches!(target, "html" | "blog" | "substack") {
        validate_metadata_url(target, metadata, "canonicalUrl", diagnostics);
        validate_metadata_url(target, metadata, "canonical_url", diagnostics);
    }

    if matches!(target, "html" | "blog" | "substack" | "epub") {
        for key in ["language", "lang", "locale"] {
            validate_metadata_language(target, metadata, key, diagnostics);
        }
    }

    if matches!(target, "blog" | "substack") {
        let description_missing =
            first_metadata_string(metadata, &["description", "summary", "subtitle", "excerpt"])
                .map(|value| value.trim().is_empty())
                .unwrap_or(true);
        if description_missing {
            push_distribution_warning(
                target,
                "missing:description-or-excerpt",
                "Publishing exports should include a description, summary, subtitle, or excerpt for previews and RSS handoff.",
                "Add front matter such as description, summary, subtitle, or excerpt before publishing.",
                diagnostics,
            );
        }

        let tags_missing = metadata_string_list(metadata, "tags").is_empty()
            && metadata_string_list(metadata, "keywords").is_empty();
        if tags_missing {
            push_distribution_warning(
                target,
                "missing:tags-or-keywords",
                "Publishing exports should include tags or keywords for platform discovery and archive management.",
                "Add tags or keywords front matter before publishing.",
                diagnostics,
            );
        }
    }

    if target == "epub" {
        let creator_missing =
            first_metadata_string(metadata, &["author", "approvedBy", "reviewer"])
                .map(|value| value.trim().is_empty())
                .unwrap_or(true);
        if creator_missing {
            push_distribution_warning(
                target,
                "missing:author-or-reviewer",
                "EPUB exports should include an author, approver, or reviewer for reader metadata.",
                "Add author, approvedBy, or reviewer front matter before distributing the ebook.",
                diagnostics,
            );
        }
    }
}

fn validate_metadata_url(
    target: &str,
    metadata: &Value,
    key: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(value) = metadata_string(metadata, key).filter(|value| !value.trim().is_empty())
    else {
        return;
    };
    if is_http_url(&value) {
        return;
    }
    let mut diagnostic = diag(
        "error",
        format!("{key} metadata must be an absolute http:// or https:// URL for {target} export."),
        None,
        None,
        Some("Use the final public URL or remove the canonical URL until it is known."),
    );
    diagnostic.related.push(format!("target:{target}"));
    diagnostic.related.push(format!("metadata:{key}"));
    diagnostics.push(diagnostic);
}

fn validate_metadata_language(
    target: &str,
    metadata: &Value,
    key: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let Some(value) = metadata_string(metadata, key).filter(|value| !value.trim().is_empty())
    else {
        return;
    };
    if is_language_tag(&value) {
        return;
    }
    let mut diagnostic = diag(
        "error",
        format!("{key} metadata must be a valid BCP-47-style language tag for {target} export."),
        None,
        None,
        Some("Use values such as en, en-US, fr, or pt-BR."),
    );
    diagnostic.related.push(format!("target:{target}"));
    diagnostic.related.push(format!("metadata:{key}"));
    diagnostics.push(diagnostic);
}

fn push_distribution_warning(
    target: &str,
    related: &str,
    message: &str,
    suggestion: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    let mut diagnostic = diag("warning", message, None, None, Some(suggestion));
    diagnostic.related.push(format!("target:{target}"));
    diagnostic.related.push(related.to_string());
    diagnostics.push(diagnostic);
}

fn first_metadata_string(metadata: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| metadata_string(metadata, key))
}

fn metadata_string_list(metadata: &Value, key: &str) -> Vec<String> {
    let Some(value) = metadata.get(key) else {
        return Vec::new();
    };
    if let Some(items) = value.as_array() {
        return items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect();
    }
    value
        .as_str()
        .map(|text| {
            text.split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn is_http_url(value: &str) -> bool {
    let trimmed = value.trim();
    let Some(rest) = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim();
    !host.is_empty()
        && !host.contains(char::is_whitespace)
        && !host.starts_with('.')
        && !host.ends_with('.')
}

fn is_language_tag(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 35 {
        return false;
    }
    trimmed.split('-').all(|part| {
        !part.is_empty() && part.len() <= 8 && part.chars().all(|ch| ch.is_ascii_alphanumeric())
    })
}

fn release_metadata_required_for_target(target: &str) -> bool {
    matches!(
        target,
        "pptx" | "blog" | "substack" | "google-docs" | "epub"
    )
}

fn validate_transform_export_settings(options: &Value, diagnostics: &mut Vec<DocumentDiagnostic>) {
    if let Some(timeout) = options.get("transformTimeoutMs") {
        let valid = timeout
            .as_u64()
            .is_some_and(|value| (1..=30_000).contains(&value));
        if !valid {
            diagnostics.push(diag(
                "error",
                "transformTimeoutMs must be between 1 and 30000.",
                None,
                None,
                Some("Use a millisecond timeout within the supported external engine limit."),
            ));
        }
    }
    validate_transform_engine_paths(options, diagnostics);
    validate_bool_map(options, "trustedTransformEngines", diagnostics);
    validate_bool_map(options, "disabledTransformEngines", diagnostics);
    validate_string_map(
        options,
        "transformInputModes",
        diagnostics,
        Some(validate_transform_input_mode),
    );
}

fn validate_string_map(
    options: &Value,
    key: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
    entry_validator: Option<fn(&str, &str, &mut Vec<DocumentDiagnostic>)>,
) {
    let Some(value) = options.get(key) else {
        return;
    };
    let Some(fields) = value.as_object() else {
        diagnostics.push(diag(
            "error",
            format!("{key} must be an object."),
            None,
            None,
            Some("Use transform names as keys."),
        ));
        return;
    };
    for (name, field) in fields {
        let Some(field) = field.as_str() else {
            diagnostics.push(diag(
                "error",
                format!("{key}.{name} must be a string."),
                None,
                None,
                Some("Use string values for transform engine settings."),
            ));
            continue;
        };
        if let Some(validator) = entry_validator {
            validator(name, field, diagnostics);
        }
    }
}

fn validate_bool_map(options: &Value, key: &str, diagnostics: &mut Vec<DocumentDiagnostic>) {
    let Some(value) = options.get(key) else {
        return;
    };
    let Some(fields) = value.as_object() else {
        diagnostics.push(diag(
            "error",
            format!("{key} must be an object."),
            None,
            None,
            Some("Use transform names as keys."),
        ));
        return;
    };
    for (name, field) in fields {
        if !field.is_boolean() {
            diagnostics.push(diag(
                "error",
                format!("{key}.{name} must be true or false."),
                None,
                None,
                Some("Use boolean trust values for each transform engine."),
            ));
        }
    }
}

fn validate_transform_engine_paths(options: &Value, diagnostics: &mut Vec<DocumentDiagnostic>) {
    let Some(value) = options.get("transformEnginePaths") else {
        return;
    };
    let Some(fields) = value.as_object() else {
        diagnostics.push(diag(
            "error",
            "transformEnginePaths must be an object.",
            None,
            None,
            Some("Use transform names as keys."),
        ));
        return;
    };
    for (name, field) in fields {
        let Some(path) = field.as_str() else {
            diagnostics.push(diag(
                "error",
                format!("transformEnginePaths.{name} must be a string."),
                None,
                None,
                Some("Use string values for transform engine settings."),
            ));
            continue;
        };
        validate_transform_engine_path(name, path, options, diagnostics);
    }
}

fn validate_transform_engine_path(
    name: &str,
    path: &str,
    options: &Value,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if path.trim().is_empty() || transform_engine_disabled(options, name) {
        return;
    }
    let path = Path::new(path);
    if !path.is_absolute() {
        diagnostics.push(diag(
            "error",
            format!("transformEnginePaths.{name} must be an absolute path."),
            None,
            None,
            Some("Use an absolute executable path; shell lookup is disabled."),
        ));
        return;
    }
    if !path.is_file() {
        diagnostics.push(diag(
            "error",
            format!("transformEnginePaths.{name} does not point to an executable file."),
            Some(path.display().to_string()),
            None,
            Some("Choose the actual engine binary path or disable this transform engine."),
        ));
        return;
    }
    validate_transform_engine_executable(name, path, diagnostics);
}

fn transform_engine_disabled(options: &Value, name: &str) -> bool {
    options
        .get("disabledTransformEngines")
        .and_then(Value::as_object)
        .and_then(|fields| fields.get(name))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

#[cfg(unix)]
fn validate_transform_engine_executable(
    name: &str,
    path: &Path,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    use std::os::unix::fs::PermissionsExt;

    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if metadata.permissions().mode() & 0o111 != 0 {
        return;
    }
    diagnostics.push(diag(
        "error",
        format!("transformEnginePaths.{name} is not executable."),
        Some(path.display().to_string()),
        None,
        Some("Make the selected engine executable or choose a different binary path."),
    ));
}

#[cfg(not(unix))]
fn validate_transform_engine_executable(
    _name: &str,
    _path: &Path,
    _diagnostics: &mut Vec<DocumentDiagnostic>,
) {
}

fn validate_transform_input_mode(
    name: &str,
    mode: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if !matches!(mode, "stdin" | "file") {
        diagnostics.push(diag(
            "error",
            format!("transformInputModes.{name} must be stdin or file."),
            None,
            None,
            Some("Choose one of the supported external transform input modes."),
        ));
    }
}
