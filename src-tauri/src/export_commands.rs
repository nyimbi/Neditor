use crate::{
    compile_with_options,
    diagnostics::{diag, DocumentDiagnostic},
    export::{
        render_docx_bytes, render_full_html, render_markdown_bundle_bytes, render_pdf_bytes,
        render_pptx_bytes,
    },
    git::get_git_status,
    path_to_string, CompileRequest, ExportManifest,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub(crate) struct ExportRequest {
    text: String,
    file_path: Option<String>,
    target: String,
    output_path: String,
    options: Value,
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
    output_path: String,
    manifest_path: Option<String>,
    manifest: ExportManifest,
    diagnostics: Vec<DocumentDiagnostic>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportReadinessReport {
    pub(crate) ready: bool,
    pub(crate) error_count: usize,
    pub(crate) warning_count: usize,
    pub(crate) info_count: usize,
    pub(crate) diagnostics: Vec<DocumentDiagnostic>,
    pub(crate) manifest: ExportManifest,
}

#[tauri::command]
pub(crate) fn export_document(request: ExportRequest) -> Result<ExportResponse, String> {
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
        other => {
            return Err(format!(
                "Unsupported export target '{other}'. Use html, pdf, docx, pptx, or markdown-bundle."
            ));
        }
    }

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
        manifest,
        diagnostics: compile_response.diagnostics,
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
    if git_export_warnings_enabled(&request.options) {
        validate_git_export_cleanliness(request.file_path.as_deref(), &mut response.diagnostics);
    }
    let error_count = response
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "error")
        .count();
    let warning_count = response
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "warning")
        .count();
    let info_count = response
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "info")
        .count();
    ExportReadinessReport {
        ready: error_count == 0 && warning_count == 0,
        error_count,
        warning_count,
        info_count,
        diagnostics: response.diagnostics,
        manifest: response.export_manifest,
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
        "html" | "pdf" | "docx" | "pptx" | "markdown-bundle" | "markdown"
    ) {
        diagnostics.push(diag(
            "error",
            format!("Unsupported export target: {target}"),
            None,
            None,
            Some("Use html, pdf, docx, pptx, or markdown-bundle."),
        ));
    }
    if options
        .get("watermark")
        .is_some_and(|value| !value.is_string())
    {
        diagnostics.push(diag(
            "error",
            "Export watermark must be a string.",
            None,
            None,
            Some("Use a text watermark or remove the option."),
        ));
    }
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
    validate_string_map(
        options,
        "transformEnginePaths",
        diagnostics,
        Some(validate_transform_engine_path),
    );
    validate_bool_map(options, "trustedTransformEngines", diagnostics);
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

fn validate_transform_engine_path(
    name: &str,
    path: &str,
    diagnostics: &mut Vec<DocumentDiagnostic>,
) {
    if path.trim().is_empty() {
        return;
    }
    if !Path::new(path).is_absolute() {
        diagnostics.push(diag(
            "error",
            format!("transformEnginePaths.{name} must be an absolute path."),
            None,
            None,
            Some("Use an absolute executable path; shell lookup is disabled."),
        ));
    }
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
