use crate::{
    bibliography::{BibliographyEntry, CitationReference},
    calculations::{FormulaDependencyEdge, FormulaValue},
    diagnostics::DocumentDiagnostic,
    document_ast::DocumentAst,
    paged_document::PagedDocument,
    provenance::{AiAssistedSection, AiSource},
    review::{ChangeNote, ReviewComment},
    tables::TableSummary,
    transforms::TransformArtifact,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct CompileRequest {
    pub(crate) text: String,
    pub(crate) file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CompileWithOptionsRequest {
    pub(crate) text: String,
    pub(crate) file_path: Option<String>,
    pub(crate) options: Value,
}

#[derive(Debug, Serialize)]
pub(crate) struct CompileResponse {
    pub(crate) compiled_markdown: String,
    pub(crate) html: String,
    pub(crate) semantic: SemanticDocument,
    pub(crate) document_ast: DocumentAst,
    pub(crate) paged_document: PagedDocument,
    pub(crate) diagnostics: Vec<DocumentDiagnostic>,
    pub(crate) include_graph: Vec<IncludeEdge>,
    pub(crate) source_map: Vec<SourceMapEntry>,
    pub(crate) metadata: Value,
    pub(crate) bibliography: Vec<BibliographyEntry>,
    pub(crate) index_terms: Vec<String>,
    pub(crate) formula_graph: Vec<FormulaValue>,
    pub(crate) formula_dependency_edges: Vec<FormulaDependencyEdge>,
    pub(crate) transform_artifacts: Vec<TransformArtifact>,
    pub(crate) export_manifest: ExportManifest,
}

#[derive(Debug, Serialize)]
pub(crate) struct SemanticDocument {
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) headings: Vec<Heading>,
    pub(crate) outline: Vec<Heading>,
    pub(crate) tables: usize,
    pub(crate) table_summaries: Vec<TableSummary>,
    pub(crate) figures: usize,
    pub(crate) equations: usize,
    pub(crate) citations: Vec<String>,
    pub(crate) citation_references: Vec<CitationReference>,
    pub(crate) duplicate_bibliography_keys: Vec<String>,
    pub(crate) glossary: std::collections::BTreeMap<String, String>,
    pub(crate) layout_directives: Vec<String>,
    pub(crate) comments: Vec<ReviewComment>,
    pub(crate) change_notes: Vec<ChangeNote>,
    pub(crate) ai_sources: Vec<AiSource>,
    pub(crate) ai_assisted_sections: Vec<AiAssistedSection>,
    pub(crate) labels: Vec<String>,
    pub(crate) cross_references: Vec<crate::references::CrossReference>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Heading {
    pub(crate) level: usize,
    pub(crate) text: String,
    pub(crate) anchor: String,
    pub(crate) line: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct IncludeEdge {
    pub(crate) parent: String,
    pub(crate) child: String,
    pub(crate) depth: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SourceMapEntry {
    pub(crate) generated_line: usize,
    pub(crate) source_file: String,
    pub(crate) source_line: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ExportManifest {
    pub(crate) document_title: String,
    pub(crate) document_version: String,
    pub(crate) status: String,
    pub(crate) exported_at: String,
    pub(crate) source_hash: String,
    pub(crate) output_path: Option<String>,
    pub(crate) output_hash: Option<String>,
    pub(crate) included_files: Vec<ManifestFile>,
    pub(crate) include_graph: Vec<IncludeEdge>,
    pub(crate) media_files: Vec<ManifestFile>,
    pub(crate) layout_sections: Vec<ManifestLayoutSection>,
    pub(crate) export_target: String,
    pub(crate) export_options: Value,
    pub(crate) transform_artifacts: Vec<Value>,
    pub(crate) progress_steps: Vec<ExportProgressStep>,
    pub(crate) readiness: ExportReadinessSummary,
    pub(crate) diagnostics: Vec<DocumentDiagnostic>,
    pub(crate) source_map: Vec<SourceMapEntry>,
    pub(crate) app_version: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ExportProgressStep {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) state: String,
    pub(crate) detail: String,
    pub(crate) work_units: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ExportReadinessSummary {
    pub(crate) ready: bool,
    pub(crate) error_count: usize,
    pub(crate) warning_count: usize,
    pub(crate) info_count: usize,
}

pub(crate) fn export_readiness_summary(
    diagnostics: &[DocumentDiagnostic],
) -> ExportReadinessSummary {
    let error_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "error")
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "warning")
        .count();
    let info_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "info")
        .count();
    ExportReadinessSummary {
        ready: error_count == 0 && warning_count == 0,
        error_count,
        warning_count,
        info_count,
    }
}

pub(crate) fn export_progress_steps(
    target: &str,
    transform_count: usize,
    include_manifest: bool,
    output_path: Option<&str>,
    output_written: bool,
) -> Vec<ExportProgressStep> {
    let render_state = if output_written {
        "complete"
    } else {
        "pending"
    };
    let render_detail = output_path
        .map(|path| format!("Target artifact path: {path}"))
        .unwrap_or_else(|| "Target artifact has not been written yet.".to_string());
    let mut steps = vec![
        ExportProgressStep {
            id: "compile".to_string(),
            label: "Compile document model".to_string(),
            state: "complete".to_string(),
            detail: "Source, includes, semantic model, diagnostics, and preview HTML are available."
                .to_string(),
            work_units: 1,
        },
        ExportProgressStep {
            id: "transforms".to_string(),
            label: "Render transform artifacts".to_string(),
            state: "complete".to_string(),
            detail: format!("{transform_count} transform artifact(s) rendered or cache-resolved."),
            work_units: transform_count,
        },
        ExportProgressStep {
            id: "readiness".to_string(),
            label: "Validate export readiness".to_string(),
            state: "complete".to_string(),
            detail: "Metadata, diagnostics, target settings, captions, provenance, and Git cleanliness checks completed.".to_string(),
            work_units: 1,
        },
        ExportProgressStep {
            id: "render".to_string(),
            label: format!("Render {target} artifact"),
            state: render_state.to_string(),
            detail: render_detail,
            work_units: 1,
        },
    ];
    let bundle_embeds_manifest = matches!(
        target,
        "markdown-bundle" | "markdown" | "blog" | "substack" | "google-docs" | "epub"
    );
    if include_manifest || bundle_embeds_manifest {
        let (label, detail) = if bundle_embeds_manifest && !include_manifest {
            (
                "Embed package manifest".to_string(),
                if output_written {
                    "Package export embeds manifest.json; sidecar manifest output is disabled."
                        .to_string()
                } else {
                    "Package export will embed manifest.json; sidecar manifest output is disabled."
                        .to_string()
                },
            )
        } else if bundle_embeds_manifest {
            (
                "Write export manifests".to_string(),
                if output_written {
                    "Package export embeds manifest.json and the sidecar manifest includes source, output, readiness, diagnostics, and progress evidence.".to_string()
                } else {
                    "Package manifest.json and sidecar manifest will be written after the target artifact succeeds.".to_string()
                },
            )
        } else {
            (
                "Write export manifest".to_string(),
                if output_written {
                    "Sidecar manifest includes source, output, readiness, diagnostics, and progress evidence.".to_string()
                } else {
                    "Sidecar manifest will be written after the target artifact succeeds."
                        .to_string()
                },
            )
        };
        steps.push(ExportProgressStep {
            id: "manifest".to_string(),
            label,
            state: render_state.to_string(),
            detail,
            work_units: 1,
        });
    }
    steps
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ManifestFile {
    pub(crate) path: String,
    pub(crate) hash: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ManifestLayoutSection {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) columns: Option<usize>,
    pub(crate) column_gap: Option<String>,
    pub(crate) page_size: Option<String>,
    pub(crate) orientation: Option<String>,
    pub(crate) margins: Option<String>,
    pub(crate) header: Option<String>,
    pub(crate) footer: Option<String>,
}
