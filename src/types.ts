export type DiagnosticSeverity = "info" | "warning" | "error";

export interface DocumentDiagnostic {
  severity: DiagnosticSeverity;
  message: string;
  source_file?: string | null;
  line?: number | null;
  column?: number | null;
  end_line?: number | null;
  end_column?: number | null;
  suggestion?: string | null;
  related: string[];
}

export interface Heading {
  level: number;
  text: string;
  anchor: string;
  line: number;
}

export interface SemanticDocument {
  title: string;
  status: string;
  headings: Heading[];
  outline: Heading[];
  tables: number;
  table_summaries: Array<{
    line: number;
    columns: string[];
    rows: number;
    numeric_columns: Record<string, number>;
  }>;
  figures: number;
  equations: number;
  citations: string[];
  citation_references: Array<{
    key: string;
    locator?: string | null;
    raw: string;
    line: number;
    column: number;
    end_column: number;
  }>;
  duplicate_bibliography_keys: string[];
  glossary: Record<string, string>;
  layout_directives: string[];
  comments: Array<{ line: number; author: string; created_at?: string | null; state: string; text: string }>;
  change_notes: Array<{ line: number; author: string; created_at?: string | null; text: string }>;
  ai_sources: Array<{
    line: number;
    provider: string;
    model: string;
    date: string;
    prompt_summary: string;
    reviewed_by: string;
    reviewed_at: string;
    status: string;
  }>;
  ai_assisted_sections: Array<{
    line: number;
    heading: string;
    status: string;
    reviewed_by: string;
    reviewed_at: string;
    source: string;
    prompt_summary: string;
  }>;
  labels: string[];
  cross_references: Array<{
    key: string;
    target_kind: string;
    resolved: boolean;
    line: number;
    column: number;
    end_column: number;
    source_file?: string | null;
  }>;
}

export interface DocumentSourceRange {
  source_file: string;
  source_line: number;
  end_source_line: number;
}

export interface TaskListItem {
  checked: boolean;
  text: string;
}

export interface FootnoteEntry {
  number: number;
  key: string;
  text: string;
}

export interface AstReviewComment {
  author: string;
  created_at: string;
  state: string;
  text: string;
}

export interface AstChangeNote {
  author: string;
  created_at: string;
  text: string;
}

export interface AstAiSource {
  provider: string;
  model: string;
  date: string;
  prompt_summary: string;
  reviewed_by: string;
  reviewed_at: string;
  status: string;
}

export interface LayoutSettings {
  columns?: number | null;
  break_before?: string | null;
  break_after?: string | null;
  keep_with_next: boolean;
  keep_together: boolean;
  header?: string | null;
  footer?: string | null;
  title?: string | null;
  layout?: string | null;
  notes?: string | null;
}

export type InlineNode =
  | { kind: "text"; text: string }
  | { kind: "strong"; text: string }
  | { kind: "emphasis"; text: string }
  | { kind: "code"; text: string }
  | { kind: "link"; text: string; url: string }
  | { kind: "citation"; key: string; keys: string[]; raw: string }
  | { kind: "cross_reference"; key: string; raw: string };

export type DocumentBlock =
  | { kind: "heading"; level: number; text: string; anchor: string; line: number; end_line: number; source?: DocumentSourceRange | null }
  | { kind: "paragraph"; text: string; inlines: InlineNode[]; line: number; end_line: number; source?: DocumentSourceRange | null }
  | { kind: "list"; ordered: boolean; items: string[]; line: number; end_line: number; source?: DocumentSourceRange | null }
  | { kind: "task_list"; items: TaskListItem[]; line: number; end_line: number; source?: DocumentSourceRange | null }
  | { kind: "block_quote"; text: string; line: number; end_line: number; source?: DocumentSourceRange | null }
  | { kind: "code_block"; language?: string | null; code: string; line: number; end_line: number; source?: DocumentSourceRange | null }
  | {
      kind: "table";
      line: number;
      end_line: number;
      id?: string | null;
      caption?: string | null;
      headers: string[];
      alignments: string[];
      rows: string[][];
      source?: DocumentSourceRange | null;
    }
  | {
      kind: "figure";
      line: number;
      end_line: number;
      id?: string | null;
      src?: string | null;
      alt?: string | null;
      caption?: string | null;
      float?: string | null;
      fit?: string | null;
      position?: string | null;
      source?: DocumentSourceRange | null;
    }
  | { kind: "equation"; line: number; end_line: number; id?: string | null; caption?: string | null; text: string; source?: DocumentSourceRange | null }
  | { kind: "layout"; line: number; end_line: number; directive: string; options: string; settings: LayoutSettings; source?: DocumentSourceRange | null }
  | { kind: "callout"; line: number; end_line: number; callout_type: string; title: string; text: string; source?: DocumentSourceRange | null }
  | { kind: "footnotes"; line: number; end_line: number; entries: FootnoteEntry[]; source?: DocumentSourceRange | null }
  | { kind: "review_comment"; line: number; end_line: number; comment: AstReviewComment; source?: DocumentSourceRange | null }
  | { kind: "change_note"; line: number; end_line: number; note: AstChangeNote; source?: DocumentSourceRange | null }
  | { kind: "ai_source"; line: number; end_line: number; provenance: AstAiSource; source?: DocumentSourceRange | null }
  | {
      kind: "transform";
      line: number;
      end_line: number;
      name: string;
      output_kind: string;
      text: string;
      html: string;
      source_hash?: string | null;
      output_hash?: string | null;
      cache_key?: string | null;
      execution_kind?: string | null;
      options?: Record<string, unknown> | null;
      source?: DocumentSourceRange | null;
    }
  | { kind: "raw_html"; line: number; end_line: number; html: string; source?: DocumentSourceRange | null };

export interface DocumentAst {
  metadata: {
    title: string;
    status: string;
    version: string;
    source_hash: string;
  };
  blocks: DocumentBlock[];
}

export interface CompileResponse {
  compiled_markdown: string;
  html: string;
  semantic: SemanticDocument;
  document_ast: DocumentAst;
  diagnostics: DocumentDiagnostic[];
  include_graph: Array<{ parent: string; child: string; depth: number }>;
  source_map: Array<{ generated_line: number; source_file: string; source_line: number }>;
  metadata: Record<string, unknown>;
  bibliography: Array<{ key: string; title: string; author?: string | null; issued?: string | null; raw: string }>;
  index_terms: string[];
  formula_graph: Array<{
    name: string;
    expression: string;
    value?: number | null;
    error?: string | null;
    dependencies: string[];
    ast?: unknown;
  }>;
  formula_dependency_edges: Array<{ from: string; to: string }>;
  transform_artifacts: Array<{
    id: string;
    name: string;
    output_kind: string;
    source_hash: string;
    source: string;
    source_file?: string | null;
    source_line?: number | null;
    end_source_line?: number | null;
    options: Record<string, unknown>;
    output_hash: string;
    cache_key: string;
    execution_kind: string;
    engine_version?: string | null;
    engine_path?: string | null;
    input_mode: string;
    duration_ms?: number | null;
    html: string;
    diagnostics: DocumentDiagnostic[];
  }>;
  export_manifest: ExportManifest;
}

export interface ManifestFile {
  path: string;
  hash: string;
}

export interface ExportManifest {
  document_title: string;
  document_version: string;
  status: string;
  exported_at: string;
  source_hash: string;
  output_path?: string | null;
  output_hash?: string | null;
  included_files: ManifestFile[];
  media_files: ManifestFile[];
  export_target: string;
  export_options: Record<string, unknown>;
  transform_artifacts: Array<Record<string, unknown>>;
  diagnostics: DocumentDiagnostic[];
  source_map: Array<{ generated_line: number; source_file: string; source_line: number }>;
  app_version: string;
}

export interface TransformEngineMetadata {
  name: string;
  execution: string;
  available: boolean;
  bundled: boolean;
  installationLabel: string;
  setupHint: string;
  securitySummary: string;
  adapterProfile: string;
  defaultCommand: string;
  requiresExecution: boolean;
  preferenceKey: string;
  inputModes: Array<"stdin" | "file" | string>;
  limits: {
    timeoutMs: number;
    maxTimeoutMs: number;
    maxInputBytes: number;
    maxOutputBytes: number;
  };
  cacheScope: string;
  exportTargets: string[];
}

export interface OpenDocument {
  id: string;
  path: string | null;
  title: string;
  text: string;
  savedHash: string;
  dirty: boolean;
  pinned?: boolean;
  modified?: string | null;
  compile?: CompileResponse;
}

export interface GitStatus {
  inside_repo: boolean;
  branch?: string | null;
  dirty: boolean;
  summary: string[];
}

export interface GitHistoryEntry {
  revision: string;
  author: string;
  date: string;
  subject: string;
}

export interface SnapshotListItem {
  snapshot_path: string;
  metadata_path: string;
  hash?: string | null;
  created_at?: string | null;
  label?: string | null;
  document_version?: string | null;
  status?: string | null;
  author?: string | null;
  include_graph_hash?: string | null;
}

export interface WorkspaceFileEntry {
  path: string;
  name: string;
  relative_path: string;
  kind: string;
  depth: number;
}

export interface WatchFileResponse {
  paths: Array<{
    path: string;
    exists: boolean;
    hash?: string | null;
    modified?: string | null;
    role?: "root" | "include" | string;
  }>;
  native_watcher?: boolean;
  watcher_error?: string | null;
}

export interface AiCleanupResponse {
  cleaned_markdown: string;
  issues: string[];
  provenance_block?: string | null;
}

export interface AiCleanupOptions {
  addProvenance: boolean;
  markAsDraft: boolean;
  insertCitationTodos: boolean;
  preserveHeadings: boolean;
  convertNumberedLists: boolean;
  convertTables: boolean;
}

export interface ExportReadinessReport {
  ready: boolean;
  error_count: number;
  warning_count: number;
  info_count: number;
  source_map: Array<{ generated_line: number; source_file: string; source_line: number }>;
  diagnostics: DocumentDiagnostic[];
  manifest: ExportManifest;
}
