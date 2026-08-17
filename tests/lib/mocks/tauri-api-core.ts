/** Mock for @tauri-apps/api/core — used by the Vite snapshot bundle. */

const EMPTY_COMPILE = {
  compiled_markdown: "",
  html: "",
  semantic: {
    title: "",
    status: "",
    headings: [],
    outline: [],
    tables: 0,
    table_summaries: [],
    figures: 0,
    equations: 0,
    citations: [],
    citation_references: [],
    duplicate_bibliography_keys: [],
    glossary: {},
    layout_directives: [],
    comments: [],
    change_notes: [],
    ai_sources: [],
    ai_assisted_sections: [],
    labels: [],
    cross_references: [],
  },
  document_ast: { blocks: [] },
  paged_document: { pages: [] },
  diagnostics: [],
  include_graph: [],
  source_map: [],
  metadata: {},
  bibliography: [],
  index_terms: [],
  formula_graph: [],
  formula_dependency_edges: [],
  transform_artifacts: [],
  knowledge_graph_nodes: [],
  knowledge_graph_edges: [],
  canvas_nodes: [],
  block_references: [],
  abscribe_branches: [],
};

const RESPONSES: Record<string, () => unknown> = {
  compile_document_with_options: () => EMPTY_COMPILE,
  compile_document: () => EMPTY_COMPILE,
  list_workspace_files: () => [],
  list_transform_engines: () => [],
  list_snapshots: () => [],
  get_git_status: () => null,
  drain_cli_open_queue: () => [],
  warmup_transforms: () => null,
  register_instance: () => null,
  list_preview_themes: () => [],
  read_workspace_settings: () => ({ theme: null, preview_theme: null }),
  run_external_transform: () => ({ html: "", diagnostics: [] }),
  list_installed_csl_styles: () => [],
  list_suggestions: () => [],
  build_workspace_link_graph: () => ({ nodes: [], edges: [] }),
  collect_workspace_tasks: () => [],
  read_audit_log: () => ({ entries: [] }),
  list_transform_handler_installers: () => [],
  load_default_markdown_reader_plan: () => null,
  load_cli_deploy_plan: () => null,
  get_tts_model_storage_location: () => null,
  bind_native_menu_commands: () => null,
  get_pending_cli_paths: () => [],
  stop_file_watcher: () => null,
  start_file_watcher: () => null,
  sync_file_watcher: () => null,
  list_trusted_engines: () => [],
  get_engine_probe_results: () => ({}),
};

export async function invoke<T>(cmd: string, _args?: Record<string, unknown>): Promise<T> {
  const handler = RESPONSES[cmd];
  return (handler ? handler() : null) as T;
}

export function convertFileSrc(path: string): string {
  return `asset://localhost/${path}`;
}

export function transformCallback(_callback: (response: unknown) => void, _once?: boolean): number {
  return 0;
}
