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

#[derive(Debug, Serialize)]
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
    pub(crate) media_files: Vec<ManifestFile>,
    pub(crate) layout_sections: Vec<ManifestLayoutSection>,
    pub(crate) export_target: String,
    pub(crate) export_options: Value,
    pub(crate) transform_artifacts: Vec<Value>,
    pub(crate) diagnostics: Vec<DocumentDiagnostic>,
    pub(crate) source_map: Vec<SourceMapEntry>,
    pub(crate) app_version: String,
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
    pub(crate) page_size: Option<String>,
    pub(crate) orientation: Option<String>,
    pub(crate) margins: Option<String>,
    pub(crate) header: Option<String>,
    pub(crate) footer: Option<String>,
}
