# NEditor Current Completion Backlog

Updated: 2026-05-21

This is the active, evidence-based backlog for finishing NEditor against
`docs/specification.md` and `docs/external-transforms.md`. It replaces the stale
todo state from earlier implementation phases. Do not treat broad implemented
surfaces as complete until the current code, tests, artifacts, local workflow
checks prove the exact requirement.

## Survey Basis

Current survey inputs:

- Product scope: `docs/specification.md`
- External transform setup and safety model: `docs/external-transforms.md`
- User-facing status and commands: `README.md`
- Frontend workbench: `src/App.vue`
- Frontend document/workspace store: `src/stores/documents.ts`
- Frontend helpers: `src/lib/`
- Browser workflow harness: `e2e/app-workflows.spec.ts`
- Frontend unit tests: `tests/frontend-unit.test.ts`
- Backend compiler, export, transform, file, Git, snapshot, validation, and
  watcher modules under `src-tauri/src/`
- Backend tests under `src-tauri/src/tests/`
- Local verification scripts in `package.json`, `playwright.config.ts`, and
  Rust/Cargo checks under `src-tauri`

Status vocabulary:

- Complete: current code plus current direct verification proves the requirement.
- Partial: implementation exists, but scope, platform coverage, or proof is
  incomplete.
- Unverified: implementation appears to exist, but direct proof is missing.
- Missing: no implementation evidence was found in the current survey.
- Deferred: explicitly later/non-goal for the current release.

## Current High-Level State

NEditor is no longer a basic scaffold. The repository currently contains:

- Tauri 2, Vue 3, Pinia, Vite, CodeMirror 6, vanilla CSS, and Rust IPC.
- A large frontend workbench with toolbar, status bar, document tabs, sidebars,
  editor, live preview, export/review/versioning/settings panels, command
  palette, table editor, AI paste cleanup modal, transform settings, and
  external conflict UI.
- A centralized document/workspace store covering local files, open tabs,
  recents, pinned tabs, workspace folders, compile state, exports, readiness,
  Git/snapshots, external watcher state, transform settings, AI cleanup, and
  review governance.
- Rust compiler support for front matter, includes, source maps, variables,
  transforms, formulas, citations, bibliography, glossary, index, cross
  references, review/provenance metadata, document AST output, paged document
  output, diagnostics, export manifests, and multiple export targets.
- Native Rust renderers or fallbacks for the required transform family, plus
  trust-gated external adapters for Graphviz/DOT, D2, PlantUML, and Pikchr.
- Richer Rust-native OpenAPI and JSON Schema transform rendering for API
  operations, parameters, request/response bodies, component schemas, nested
  schema fields, array items, refs, enums, formats, defaults, and constraints.
- Export modules for HTML, PDF, DOCX, PPTX, and Markdown bundles.
- Backend tests across compiler, exports, transforms, tables, media packaging,
  validation, file commands, Git workflows, snapshots, review/provenance, and
  external engines.
- An executable IPC coverage guard for the initial command surface in
  specification section 25.4, plus `docs/ipc-command-coverage.md` as the
  human-readable command evidence table.
- MIT licensing in `LICENSE`, `package.json`, `src-tauri/Cargo.toml`, and
  the Tauri desktop bundle metadata.
- Frontend unit tests for table parsing/serialization, conflict diff alignment,
  AI paste insertion modes, conflict merge-line composition, explicit conflict
  merge composition ordering, and local verification script coverage.
- A Playwright browser workflow harness for the Vite-rendered workbench with a
  browser-side Tauri IPC mock.
- References sidebar include graph navigation for parent-child include edges,
  open-child actions, directive jumps, and command-palette include entries.
- Local verification commands for frontend logic, browser workflows, Rust
  checks/tests, native-watch compilation, static analysis, frontend builds, and
  Tauri desktop compilation.

The remaining work is primarily about workflow proof, artifact fidelity,
cross-platform validation, and reducing risk in oversized modules after
behavior is locked.

## Current Verification Snapshot

Latest pushed code commit inspected before this update:

- `25bc28f Prove snapshot and release workflows in the workbench`

Remote GitHub Actions are not an active verification surface for this project.
Older run references below are retained only as historical debugging context and
must not be used as completion evidence for new work. Current completion proof
comes from local command output, committed artifacts, rendered/manual QA, and
explicit platform checks run outside GitHub Actions.

Most recent local verification evidence:

- `cargo fmt --check`: passed in `src-tauri`.
- `cargo check --locked`: passed in `src-tauri`.
- `cargo check --locked --features native-watch`: passed in `src-tauri`.
- `cargo clippy --locked --all-targets -- -D warnings`: passed in
  `src-tauri`.
- `cargo test --locked`: passed locally with 169 Rust tests on this Unix host.
- `cargo test --locked compiler_stress_handles_large_documents_with_many_artifacts --lib`:
  passed and stress-compiles a large Markdown source with nested includes, 80
  tables, 80 CSV transform artifacts, 120 formula definitions, many source-map
  entries, and many broken link/media diagnostics.
- `cargo test --locked repeated_export_loop_keeps_large_artifacts_stable --lib`:
  passed and repeatedly renders a large compiled document through HTML, PDF,
  DOCX, PPTX, and Markdown bundle outputs with structural and artifact-size
  assertions.
- `cargo test --locked repeated_editing_sessions_reuse_external_transform_cache --lib`:
  passed and repeatedly recompiles edited document text while proving a stable
  trusted external DOT transform is served from cache after the first run.
- `cargo test --locked export_readiness_and_manifest_report_progress_steps --lib`:
  passed and proves readiness/export manifests carry compile, transform,
  readiness, render, and manifest progress steps.
- `cargo test --locked repeated_compile_export_cycles_keep_memory_growth_bounded --lib`:
  passed and repeatedly compiles/exports large documents while bounding retained
  artifact summaries and process RSS growth on macOS/Linux hosts.
- `cargo test --locked git_restore_and_tag_reject_option_shaped_refs --lib`:
  passed and proves release tags and restore revisions reject option-shaped and
  unsupported ref syntax before invoking Git.
- `cargo test --locked git_restore_refuses_symlink_targets --lib`: passed and
  proves restore refuses symlinked worktree files without modifying the outside
  symlink target.
- `cargo test --locked snapshot_restore --lib -- --nocapture`: passed and
  proves snapshot restore is scoped to Markdown snapshot files inside the
  configured active-document store, requires matching metadata, and rejects
  snapshots for another source document.
- `cargo test --locked git_history_diff_commit_tag_and_restore_workflow --lib`:
  passed after Git hardening and proves normal history, diff, commit, tag, and
  restore behavior still works.
- `cargo test --locked export_command_tests --lib`: passed 15 export command
  tests, including direct-export dirty-Git warnings copied into response,
  sidecar manifests, structured export progress-step reporting, and precise
  no-bibliography citation readiness ranges.
- `cargo test --locked prepare_for_export_reports_missing_citation_sources_with_precise_ranges --lib -- --nocapture`:
  passed and proves no-bibliography citation readiness emits a broad source
  warning plus precise per-key missing citation diagnostics that are copied
  into the export manifest.
- `cargo test --locked compiler_reports_duplicate_bibliography_keys --lib -- --nocapture`:
  passed and proves duplicate bibliography diagnostics point at the duplicate
  entry while recording the first occurrence in related context.
- `cargo test --locked compiler_reports_csl_and_hayagriva_duplicate_key_locations --lib -- --nocapture`:
  passed and proves duplicate-key source ranges for CSL JSON array entries and
  Hayagriva YAML top-level entries, including duplicate Hayagriva keys that
  would otherwise be collapsed by YAML map parsing.
- `cargo test --locked compiler_reports_duplicate_keys_across_external_bibliography_files --lib -- --nocapture`:
  passed and proves multiple bibliography front matter paths can load separate
  BibTeX and CSL JSON files, with duplicate-key diagnostics pointing at the
  duplicate key in the second external file and related context for the first
  file.
- `cargo test --locked citation_tests --lib`: passed 14 citation tests,
  including BibTeX, CSL JSON, Hayagriva, external-file duplicate-key source
  ranges, numeric citation rendering, unsupported CSL-style fallback
  diagnostics, and citation export conformance.
- `cargo test --locked compiler_generates_glossary_sections_from_marker_and_metadata --lib -- --nocapture`:
  passed and proves `[GLOSSARY]` marker replacement, front matter-driven
  generated glossary insertion, preview hover preservation, and DOCX glossary
  artifact text.
- `cargo test --locked heading_appendix_and_decision_references_survive_cross_target_exports --lib -- --nocapture`:
  passed and proves section, appendix, and decision cross-reference labels
  survive HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts.
- `cargo test --locked cross_references_resolve_heading_appendix_and_decision_anchors --lib -- --nocapture`:
  passed and proves unprefixed appendix/decision anchors render as Appendix
  and Decision labels instead of generic section labels.
- `cargo test --locked compiler_generates_index_from_front_matter_without_marker --lib -- --nocapture`:
  passed and proves `index.enabled: true` front matter generates an index
  without `[INDEX]`, preserves linked terms, honors exclusion settings, and
  strips inline index markers.
- `cargo test --locked front_matter_index_survives_cross_target_exports --lib -- --nocapture`:
  passed and proves `index: true` front matter-generated index content
  survives HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts.
- `cargo test --locked compiler_renders_block_and_inline_equations --lib -- --nocapture`:
  passed and proves documented equation captions render as numbered captions,
  flow into the AST, and satisfy equation caption-label readiness.
- `cargo test --locked captioned_equations_survive_cross_target_exports --lib -- --nocapture`:
  passed and proves captioned equations survive HTML, PDF, DOCX, PPTX, and
  Markdown bundle text/AST outputs.
- `cargo test --locked generated_toc_exports_page_numbers_for_pdf_and_docx --lib -- --nocapture`:
  passed and proves front matter-generated TOCs keep depth/numbering, PDF
  exports render page-numbered TOC entries, and DOCX exports emit a TOC field
  instead of leaking Markdown TOC links.
- `cargo test --locked compiler_reports_circular_and_too_deep_includes --lib -- --nocapture`:
  passed and proves circular include diagnostics, include-chain graph depth, and
  maximum include depth enforcement.
- `cargo test --locked compiler_reports_unreadable_include_targets_with_context --lib -- --nocapture`:
  passed and proves unreadable/invalid include target diagnostics carry source
  file, line, suggestion, include target, and resolved path context.
- `cargo test --locked named_table_formulas_survive_cross_target_exports --lib -- --nocapture`:
  passed and proves named table/range formulas survive HTML, PDF, DOCX, PPTX,
  Markdown bundle text, and Markdown bundle AST outputs.
- `cargo test --locked compiler_loads_front_matter_json_and_yaml_data_sources --lib -- --nocapture`:
  passed and proves JSON/YAML files referenced from front matter render through
  structured transform artifacts, preview HTML, include graph, and manifest
  included-file evidence.
- `cargo test --locked compiler_reports_malformed_front_matter_data_sources --lib -- --nocapture`:
  passed and proves data source entries with missing paths, unsupported types,
  and unreadable local files produce actionable diagnostics.
- `cargo test --locked front_matter_data_sources_survive_cross_target_exports --lib -- --nocapture`:
  passed and proves front matter CSV/TSV/JSON/YAML data sources survive HTML,
  PDF, DOCX, PPTX, Markdown bundle text, and Markdown bundle manifest outputs.
- `cargo test --locked compiler_formats_document_variables_and_reports_bad_filters --lib -- --nocapture`:
  passed and proves document variable defaults, chained text filters, numeric
  filters, unsupported-filter diagnostics, and nonnumeric-filter diagnostics.
- `cargo test --locked formatted_document_variables_survive_cross_target_exports --lib -- --nocapture`:
  passed and proves formatted document variables survive HTML, PDF, DOCX, PPTX,
  and Markdown bundle text outputs.
- `cargo test --locked compiler_reports_malformed_front_matter_with_source_ranges --lib -- --nocapture`:
  passed and proves invalid YAML front matter and non-mapping YAML front matter
  produce source-ranged diagnostics with actionable suggestions.
- `cargo test --locked`: passed locally with 167 Rust tests on this Unix host.
- `pnpm run test:unit`: passed with 11 frontend unit tests, including latest
  document task cancellation/stale-result guard coverage, preview debounce
  timing/coalescing coverage, and workspace persistence migration/schema
  normalization.
- `pnpm run build`: passed with `vue-tsc --noEmit` and Vite production build.
- `pnpm exec playwright test --list`: listed 36 Chromium workflow tests,
  including the new snapshot restore and release tagging workflow.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "snapshot restore" --project chromium`:
  blocked before assertions because the local Playwright Chromium headless-shell
  executable is missing from this host cache.
- `./node_modules/.bin/tauri build --no-bundle`: passed and built the release
  desktop binary after the snapshot restore IPC request update.
- `pnpm run check:a11y`: passed and checked the Vue template for accessible
  button names, form-control labels, and dialog labeling.
- `pnpm run check:docs`: passed and checked README plus all docs for missing
  local links, including storage model and security threat model docs.
- `npx playwright test e2e/app-workflows.spec.ts -g "keeps large document editing"`:
  not run to completion on this host because the Playwright Chromium binary is
  not installed; the large-document browser workflow is present in the harness.
- `pnpm exec playwright test --list`: listed 36 Chromium workflow tests,
  including the new large-document browser interaction workflow plus the
  existing harness proof for command-palette insertion of `[TOC]`, `[INDEX]`,
  `[GLOSSARY]`, `[BIBLIOGRAPHY]`, `[LIST_OF_FIGURES]`, and
  `[LIST_OF_TABLES]`, and the snapshot restore/release tagging workflow.
- `./node_modules/.bin/tauri build --no-bundle`: passed and built the release
  desktop binary at `src-tauri/target/release/neditor`.
- `git diff --check`: passed.
- Repo-wide markdown local link resolution: passed after adding
  `docs/architecture.svg` and the `check:docs` guard.
- `pnpm exec playwright test --list`: listed the browser workflow harness with
  references-panel proof for resolved bibliography entries, missing citation
  keys, and duplicate bibliography keys.
- `pnpm run test:unit`: passed with 8 frontend unit tests.
- `pnpm run build`: passed with `vue-tsc --noEmit` and Vite production build.
- `cargo test --locked compiler_generates_lists_of_figures_and_tables --lib`:
  passed.
- `cargo fmt --check`: passed in `src-tauri`.
- `pnpm exec playwright test --list`: listed 34 Chromium workflow tests,
  including document-set and folder tab grouping.
- `pnpm run test:unit`: passed with 8 frontend unit tests.
- `pnpm run build`: passed with `vue-tsc --noEmit` and Vite production build.
- `git diff --check`: passed.
- Markdown local link resolution for updated docs: passed.
- `cargo test --locked spec_25_4_ipc_commands_are_registered_and_documented --lib`: passed.
- `cargo test --locked`: passed locally with 129 Rust tests.
- `cargo fmt --check`: passed in `src-tauri`.
- `cargo clippy --locked --all-targets -- -D warnings`: passed in `src-tauri`.
- `git diff --check`: passed.
- Markdown local link resolution for README and updated docs: passed.

Archived remote workflow evidence from retired GitHub Actions:

- Browser workflow tests passed after pnpm setup, Node setup, dependency
  install, Playwright Chromium install, and `pnpm run test:e2e`. The suite now
  includes 28 Chromium workflow tests, including advanced table paste import,
  numeric sorting, formula rows, merged-cell metadata, apply-back-to-editor
  behavior, row/column structure editing, column format totals,
  cancel-without-applying behavior, AI paste insert/quote/appendix/replace
  document/section merge/replace selection modes, the mocked file lifecycle
  flow, save-as to a new path, and reopening that saved document from the
  recently closed list. It also now covers stale-save conflict blocking,
  conflict compare visibility, saving unsaved local edits to a copy without
  overwriting the disk edit, merging external conflict text back into the
  original file, keeping local editor edits while leaving the external disk edit
  untouched, accepting external disk content into the active document, clean
  watcher-originated reload, watcher-originated dirty root-file conflicts,
  clean included-file recompile, dirty included-file conflicts, restart-style
  workspace restore, restored scroll positions, missing restored-file warnings,
  tab activation, dirty close confirmation, renamed recent cleanup, deleted
  recently-closed pruning, recent folder reopen/prune behavior, and externally
  moved recently-closed path pruning. It also covers synchronized scrolling in
  both directions between editor and preview, clicking a rendered preview
  heading to jump back to the source line, persisted editor word-wrap and
  line-number settings, CodeMirror find/replace, smart list continuation,
  bracket auto-pairing, command-palette heading navigation, command-palette
  citation navigation, command-palette glossary navigation, command-palette
  index navigation, command-palette open-document switching, command-palette
  workspace-file opening, transform engine settings trust/probe diagnostics,
  target-specific export readiness status and manifest preview, export output
  and manifest path reporting, success and writer-failure diagnostics, and
  blocked-readiness diagnostics before file write.
- Ubuntu desktop passed setup, Linux optional transform installation, Rust
  formatting, Rust check, native-watch check, clippy, Rust tests, frontend unit
  tests, frontend build, and Tauri `--no-bundle` desktop build.
- macOS desktop passed Rust formatting, Rust check, native-watch check, clippy,
  Rust tests, frontend unit tests, frontend build, and Tauri `--no-bundle`
  desktop build.
- Windows desktop passed setup, Rust formatting, Rust check, native-watch check,
  clippy, Rust tests, frontend unit tests, frontend build, and Tauri
  `--no-bundle` desktop build.

Recent local verification evidence from this buildout:

- `pnpm run test:unit`: passed.
- `pnpm run build`: passed.
- `cargo fmt --check`: passed in `src-tauri`.
- `cargo check --locked`: passed in `src-tauri`.
- `cargo check --locked --features native-watch`: passed in `src-tauri`.
- `cargo clippy --locked --all-targets -- -D warnings`: passed locally.
- `cargo test --locked`: passed locally with 126 Rust tests.
- `pnpm tauri build --no-bundle`: passed locally.
- `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test --list`: listed the
  current browser tests.
- `pnpm run test:unit`: passed after table-draft cancellation and accessible
  row/column controls.
- `pnpm run build`: passed after table-draft cancellation and accessible
  row/column controls.
- `pnpm exec playwright test --list`: listed 14 Chromium workflow tests after
  adding table structure/format/cancel coverage.
- `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts
  --grep "edits table structure" --project chromium`: blocked locally by the
  macOS Chromium Mach bootstrap permission failure before app assertions;
  escalated local browser execution was rejected by the approval reviewer.
- `gh run watch 26143491444 --exit-status`: diagnosed the first pushed
  table-structure workflow failure; 13 browser tests passed and the new test
  expected raw `74000` after currency formatting had correctly rendered
  `$74000`.
- `pnpm run test:unit`, `pnpm run build`, `pnpm exec playwright test --list`,
  and `git diff --check`: passed again after tightening the table-structure
  assertion to match currency formatting.
- `gh run view 26143632239 --json status,conclusion,headSha,jobs,url`: passed
  for `dbd440d` across browser workflow tests and Ubuntu/macOS/Windows desktop
  builds.
- `pnpm run test:unit`: passed after adding AI paste quote, appendix, replace
  document, section-merge, and replace-selection browser workflow coverage.
- `pnpm run build`: passed after adding the AI paste mode browser workflow
  coverage.
- `pnpm exec playwright test --list`: listed 17 Chromium workflow tests after
  adding the AI paste mode coverage.
- `gh run view 26144290812 --job 76896091795 --log`: diagnosed the first
  pushed AI paste mode workflow failure; 15 browser tests passed and the two
  new mode tests only failed because compile statistics replaced transient
  status-bar messages before assertion.
- `gh run view 26144430209 --json status,conclusion,headSha,jobs,url`: passed
  for `3b17c03` across 17 browser workflow tests and Ubuntu/macOS/Windows
  desktop builds.
- `pnpm run test:unit`: passed after adding include-aware browser mock
  compilation and included-file watcher workflow tests.
- `pnpm run build`: passed after adding include-aware browser mock compilation
  and included-file watcher workflow tests.
- `pnpm exec playwright test --list`: listed 19 Chromium workflow tests after
  adding clean included-file recompile and dirty included-file conflict
  coverage.
- `gh run watch 26145509141 --exit-status`: passed for commit `c0cefd1` across
  19 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `pnpm run test:unit`: passed after adding workspace mode/sidebar persistence.
- `pnpm run build`: passed after adding workspace mode/sidebar persistence and
  the restart-style browser workflow mock support.
- `pnpm exec playwright test --list`: listed 20 Chromium workflow tests after
  adding restart-style workspace restore coverage.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "restores
  workspace" --project chromium`: blocked locally because the Chromium
  headless-shell executable is missing from the Playwright cache.
- `gh run view 26147556750 --json status,conclusion,headSha,jobs,url`: passed
  on commit `655d65c` across the 20-test browser workflow job and the
  Ubuntu/macOS/Windows desktop builds, including restart-style workspace
  restore.
- `pnpm run test:unit`: passed after adding per-document scroll persistence and
  missing-restored-file state.
- `pnpm run build`: passed after adding per-document scroll persistence and
  missing-restored-file warning UI.
- `pnpm exec playwright test --list`: listed 21 Chromium workflow tests after
  adding scroll restore and missing-restored-file warning coverage.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "restores
  workspace\|skips missing" --project chromium`: blocked locally because the
  Chromium headless-shell executable is missing from the Playwright cache.
- `gh run view 26148828614 --json status,conclusion,headSha,jobs,url`: passed
  on commit `5a13fe2` across the 21-test browser workflow job and the
  Ubuntu/macOS/Windows desktop builds, including scroll restore and
  missing-restored-file warning coverage.
- `pnpm run test:unit`: passed after adding dirty-close confirmation and stale
  recent cleanup paths.
- `pnpm run build`: passed after adding dirty-close confirmation and stale
  recent cleanup paths.
- `pnpm exec playwright test --list`: listed 22 Chromium workflow tests after
  adding tab activation, dirty close confirmation, renamed recent cleanup, and
  deleted recently-closed pruning coverage.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "switches tabs"
  --project chromium`: blocked locally because the Chromium headless-shell
  executable is missing from the Playwright cache.
- `pnpm exec playwright install chromium`: interrupted after producing no
  output for several minutes; the hung process tree was terminated.
- `gh run view 26150581320 --job 76916713881 --log`: diagnosed the pushed
  tab/stale-recent workflow failure; the Tauri dialog mock handled
  `plugin:dialog|confirm`, while installed `@tauri-apps/plugin-dialog`
  implements `confirm()` through `plugin:dialog|message`.
- `pnpm run test:unit`, `pnpm run build`, `pnpm exec playwright test --list`,
  and `git diff --check`: passed after correcting the dialog mock and
  new-document dirty-close assertions.
- `gh run view 26151007765 --job 76918155717 --log`: diagnosed the follow-up
  browser failure; 21 workflow tests passed and the remaining issue was an
  ambiguous `Rename` locator.
- `pnpm run test:unit`, `pnpm run build`, `pnpm exec playwright test --list`,
  and `git diff --check`: passed after disambiguating the rename command
  locator.
- `gh run watch 26151184228 --exit-status`: passed on commit `bf60405` across
  22 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `pnpm run test:unit`: passed after adding stale recent-folder cleanup and
  moved recently-closed path coverage.
- `pnpm run build`: passed after adding stale recent-folder cleanup and moved
  recently-closed path coverage.
- `pnpm exec playwright test --list`: listed 23 Chromium workflow tests after
  adding recent folder reopen/prune behavior and moved recently-closed path
  pruning.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "reopens recent
  folders" --project chromium`: blocked locally because the Chromium
  headless-shell executable is missing from the Playwright cache.
- `git diff --check`: passed after adding recent-folder and moved-path
  coverage.
- `gh run watch 26152255407 --exit-status`: passed on commit `13b3086` across
  23 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `pnpm run test:unit`: passed after adding preview scroll-sync and heading
  navigation browser coverage.
- `pnpm run build`: passed after adding preview scroll-sync and heading
  navigation browser coverage.
- `pnpm exec playwright test --list`: listed 24 Chromium workflow tests after
  adding preview scroll sync and heading click-to-source coverage.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "syncs editor"
  --project chromium`: blocked locally because the Chromium headless-shell
  executable is missing from the Playwright cache.
- `git diff --check`: passed after adding preview navigation coverage.
- `gh run watch 26153224371 --exit-status`: passed on commit `7702e89` across
  24 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `pnpm run test:unit`: passed after adding editor ergonomics browser coverage.
- `pnpm run build`: passed after adding editor ergonomics browser coverage.
- `pnpm exec playwright test --list`: listed 25 Chromium workflow tests after
  adding editor settings, find/replace, list continuation, bracket auto-pairing,
  and command-palette heading navigation coverage.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "persists editor
  settings" --project chromium`: blocked locally because the Chromium
  headless-shell executable is missing from the Playwright cache.
- `git diff --check`: passed after adding editor ergonomics coverage.
- `gh run watch 26154535588 --exit-status`: passed on commit `f13c3f3` across
  25 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `pnpm run test:unit`: passed after adding command palette citation/glossary/
  index navigation coverage.
- `pnpm run build`: passed after adding command palette citation/glossary/index
  navigation coverage.
- `pnpm exec playwright test --list`: listed 26 Chromium workflow tests after
  adding command palette citation, glossary, and index navigation coverage.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "citation
  glossary" --project chromium`: blocked locally because the Chromium
  headless-shell executable is missing from the Playwright cache.
- `git diff --check`: passed after adding command palette reference navigation
  coverage.
- `gh run watch 26155535210 --exit-status`: passed on commit `5f75e44` across
  26 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `pnpm run test:unit`: passed after adding command palette open-document and
  workspace-file navigation coverage.
- `pnpm run build`: passed after adding command palette open-document and
  workspace-file navigation coverage.
- `pnpm exec playwright test --list`: listed 27 Chromium workflow tests after
  adding command palette document/workspace navigation coverage.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "open document
  and workspace file" --project chromium`: blocked locally because the Chromium
  headless-shell executable is missing from the Playwright cache.
- `git diff --check`: passed after adding command palette document/workspace
  navigation coverage.
- `gh run watch 26156393184 --exit-status`: passed on commit `145942a` across
  27 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `pnpm run test:unit`: passed after adding transform engine trust/probe
  browser workflow coverage.
- `pnpm run build`: passed after adding transform engine trust/probe browser
  workflow coverage.
- `pnpm exec playwright test --list`: listed 28 Chromium workflow tests after
  adding transform settings trust/probe diagnostics coverage.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "transform
  engine trust" --project chromium`: blocked locally because the Chromium
  headless-shell executable is missing from the Playwright cache.
- `git diff --check`: passed after adding and stabilizing transform settings
  workflow coverage.
- `gh run watch 26157312901 --exit-status`: failed on commit `3b7a2fe` after
  27 browser workflows passed; the new transform-settings workflow reached the
  denied-trust branch, where Playwright `check()` conflicted with the expected
  checkbox reset.
- `gh run watch 26157446711 --exit-status`: passed on commit `976016c` across
  28 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `pnpm run test:unit`: passed after export result diagnostics and immediate
  export editor-state flushing.
- `pnpm run build`: passed after the export readiness/result workflow update.
- `pnpm exec playwright test --list`: listed 28 Chromium workflow tests after
  adding export readiness/result success, failure, and blocked-readiness
  workflow coverage.
- `pnpm exec playwright install chromium`: blocked locally by `EPERM` while
  creating `/Users/nyimbiodero/Library/Caches/ms-playwright/__dirlock`.
- `git diff --check`: passed after the export workflow fixes.
- `gh run watch 26158664142 --exit-status`: failed on commit `03bda13` because
  the UI preview still showed the active compile manifest instead of the
  target-specific readiness manifest.
- `gh run watch 26158831828 --exit-status`: failed on commit `18ff4a4` because
  immediate export after typing could still read stale debounced store text.
- `gh run watch 26159218728 --exit-status`: failed on commit `e1e1b2a` with
  the same stale-store export readiness issue after the test insertion path was
  stabilized.
- `gh run watch 26159396761 --exit-status`: passed on commit `02e832a` across
  28 browser workflow tests and Ubuntu/macOS/Windows desktop builds.
- `cargo test --locked external_transform_tests --lib`: passed after the
  `pikchr-cli` temporary source path fix.
- `cargo test --locked file_command_tests --lib`: passed after slash-normalized
  path serialization.
- `cargo test --locked media_export_tests --lib`: passed after slash-normalized
  path serialization.
- `cargo clippy --locked --all-targets -- -D warnings`: passed after the
  latest Pikchr/path fixes.
- `cargo test --locked`: passed after the latest Pikchr/path and fake-`d2`
  fixture fixes with 126 Rust tests.
- `git diff --check`: passed after the latest documentation and fake-`d2`
  fixture edits.
- `pnpm exec playwright test --list`: passed after adding the mocked file
  lifecycle workflow, listing six Chromium workflow tests.
- `pnpm exec playwright test --list`: passed after adding the mocked save-as
  and recently closed workflow, listing seven Chromium workflow tests.
- `pnpm run build`: passed after adding accessible recent-file list sections and
  the new workflow proof.
- `gh run watch 26137556147 --exit-status`: passed for commit `138bf5d` across
  browser workflows and Ubuntu/macOS/Windows desktop builds.
- `pnpm exec playwright test --list`: passed after adding stale-save conflict
  copy and merge workflows, listing nine Chromium workflow tests.
- `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts
  --grep "stale saves|external conflict" --project chromium`: blocked locally
  by the existing macOS Chromium Mach bootstrap permission failure before app
  assertions.
- `pnpm run build`: passed after the stale-save conflict browser workflow tests.
- `git diff --check`: passed after the stale-save conflict browser workflow
  tests.
- `gh run watch 26138478934 --exit-status`: browser job failed for commit
  `3cb1b84` because the merge test asserted a transient status message that
  compile refresh could overwrite; eight browser tests passed before the fix.
- `gh run watch 26138672512 --exit-status`: passed for commit `25c7d1e` across
  nine browser workflows and Ubuntu/macOS/Windows desktop builds.
- `pnpm exec playwright test --list`: passed after adding keep-local and
  accept-external stale-save conflict workflows, listing 11 Chromium workflow
  tests.
- `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts
  --grep "keeps local|accepts external" --project chromium`: blocked locally
  by the existing macOS Chromium Mach bootstrap permission failure before app
  assertions.
- `pnpm run build`: passed after the keep-local and accept-external conflict
  workflow tests.
- `git diff --check`: passed after the keep-local and accept-external conflict
  workflow tests.
- `gh run watch 26139678118 --exit-status`: passed for commit `4eb1d2c` across
  11 browser workflows and Ubuntu/macOS/Windows desktop builds.
- `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "opens, saves,
  duplicates" --project chromium`: blocked locally because the macOS
  Playwright cache is missing the Chromium headless-shell executable.
- `git diff --check`: passed after stabilizing the file lifecycle workflow
  assertions.

Known local environment caveat:

- Full Playwright execution in this macOS sandbox reaches Chromium launch, then
  fails before app assertions because Chromium cannot register its Mach
  bootstrap port. This remains a local host limitation, not a reason to use
  GitHub Actions as a source of truth.

## P0 - Immediate Blockers

### 1. Keep Local Verification Green

Status: active.

GitHub Actions has been removed from the project. The local baseline must stay
green before claiming a slice is complete:

- `pnpm run test:unit`
- `pnpm run build`
- `cd src-tauri && cargo fmt --check`
- `cd src-tauri && cargo check --locked`
- `cd src-tauri && cargo check --locked --features native-watch`
- `cd src-tauri && cargo clippy --locked --all-targets -- -D warnings`
- `cd src-tauri && cargo test --locked`
- `./node_modules/.bin/tauri build --no-bundle`

Older remote run notes in this file explain past fixes only; do not use them as
current completion evidence.

Resolved previous Windows clippy failure:

- Command: `cargo clippy --locked --all-targets -- -D warnings`
- Job: `Desktop build (windows-latest)`
- Run: `26131929125`
- Failure: `clear_external_transform_memory_cache_for_tests` is dead code in
  `src/transforms/external.rs:444` for the Windows lib-test target.

Local fix:

- `clear_external_transform_memory_cache_for_tests` is now compiled only for
  Unix tests, matching its Unix-only call site.

Completion criteria:

- The helper is compiled only where it is used, used by tests on every target,
  or annotated with a narrowly justified lint expectation.
- `cargo clippy --locked --all-targets -- -D warnings` passes locally.
- Archived Windows remote workflow reached Rust backend tests in run `26132634911`.

Resolved Windows path failure:

- Command: `cargo test --locked`
- Job: `Desktop build (windows-latest)`
- Run: `26132634911`
- Failing tests:
  `stable_file_ipc_aliases_open_save_as_and_watch_paths`,
  `workspace_listing_skips_hidden_and_build_artifacts`,
  `export_packages_local_figure_media_relative_to_source_file`, and
  `markdown_bundle_keeps_duplicate_include_basenames_distinct`.
- Failure shape: native Windows path separators caused unstable IPC/export
  path identity and relative path assertions.

Local fix:

- `path_to_string` now serializes paths with `/` separators so file IPC,
  workspace listings, manifests, include maps, media maps, and tests share the
  same cross-platform path representation.

Completion criteria:

- Archived Windows remote workflow passes the four previously failing Rust tests and the full desktop
  job in runs `26133595556` and `26134248308`.
- The path normalization does not break local file commands, Git commands,
  external engine execution, or export artifact references.

Resolved Ubuntu installed-Pikchr failure:

- Command: `cargo test --locked`
- Job: `Desktop build (ubuntu-22.04)`
- Run: `26132634911`
- Failing test:
  `external_transform_conformance_runs_installed_engines`
- Failure: `pikchr-cli` exits with status 1 and reports `No such file or
  directory` after receiving the raw source as its positional argument. This
  means CI's installed CLI contract is positional source file path, not
  positional source text.

Local fix:

- The external Pikchr adapter now detects `pikchr-cli` executables and passes
  a temporary `.pikchr` source file path while preserving stdin/file behavior
  for other Pikchr executables.
- The adapter cache identity and diagnostics include the selected adapter args,
  with runtime diagnostics exposing the temporary source path.

Completion criteria:

- The Pikchr external adapter detects the installed CLI contract correctly.
  Run `26133136580` reached and passed
  `external_transform_conformance_runs_installed_engines`.
- The test still proves real installed-engine conformance; do not weaken it
  into a mock-only pass.
- Local installed-engine checks pass where the optional engines are installed;
  otherwise record the missing engine as an explicit platform gap.

Resolved Ubuntu fixture failure:

- Command: `cargo test --locked`
- Job: `Desktop build (ubuntu-22.04)`
- Run: `26133136580`
- Failing test:
  `external_transform_adapters_shape_engine_specific_invocations`
- Failure: the fake `d2` adapter exited successfully without consuming stdin,
  so Linux reported `Broken pipe (os error 32)` while the test writer thread
  was still sending source input.

Local fix:

- The fake `d2` adapter now drains stdin with `cat >/dev/null` before printing
  the SVG fixture output. This preserves real adapter stdin behavior while
  making the fixture portable on Linux.

Completion criteria:

- Local Ubuntu/Linux proof passes `external_transform_adapters_shape_engine_specific_invocations` where optional engines are installed.
  Verified in runs `26133595556` and `26134248308`.
- The real installed-engine conformance test remains enabled and passing.
- Do not mask stdin write failures in production adapter code to compensate for
  a mock that ignores stdin.

### 2. Keep The Browser Workflow Lane Passing While Expanding It

Status: current browser workflow lane is green in CI; coverage remains partial.

Current browser coverage in `e2e/app-workflows.spec.ts`:

- Workbench boot.
- Split, preview, and source view mode switching.
- Command palette table snippet insertion.
- Table editor insertion.
- Mocked file lifecycle workflow: open, edit, save, duplicate, rename, pin,
  reveal, workspace listing, and revert against an in-memory Tauri file/dialog
  mock.
- Mocked save-as and recently closed workflow: save the active document to a new
  path, prove the saved contents, close the tab, reopen from the recently closed
  list, clear that list entry, and prove the reopened path through the active
  workspace row.
- Stale-save conflict workflow: block a save when disk content changed, expose
  Compare/Accept external/Keep local/Save copy actions, show local and external
  content in the compare dialog, preserve local edits as a separate copy without
  overwriting the disk edit, merge external conflict text back into the original
  file, compose merge output from selected local/external lines with reorder and
  remove controls, keep local editor edits while leaving the external disk edit
  untouched, and accept external disk content into the active document.
- Watcher-originated root-file workflow: clean documents reload external edits
  automatically, while dirty documents open the non-destructive compare flow.
- Included-file watcher workflow: clean master documents recompile when an
  included file changes, and dirty master documents open the non-destructive
  included-file compare flow before accepting the updated include.
- Workspace restore workflow: restart-style restore of open tabs, active tab,
  pinned state, mode/sidebar state, workspace root, recent files, restored
  scroll positions, and clear warnings for missing restored files.
- Tab/recent workflow: tab activation, dirty close confirmation, renamed recent
  cleanup, and deleted recently-closed pruning.
- Recent folder and moved-path workflow: reopen a recent folder, prune a missing
  workspace root from recent folders, prune an externally moved recently closed
  path, and show the moved target through workspace refresh.
- Preview navigation workflow: synchronize scroll from editor to preview,
  synchronize scroll from preview back to editor, and click a rendered preview
  heading to jump to the source line.
- Live preview update workflow: type a heading and paragraph in the source and
  verify the preview renders the new content.
- Editor ergonomics workflow: persist word-wrap and line-number settings,
  execute CodeMirror find/replace through the search keybinding, continue a
  Markdown list item, auto-pair brackets, and navigate a heading through the
  command palette.
- Command palette reference navigation workflow: search and execute citation,
  glossary, and index commands, open the References panel, and jump to the
  matching source locations.
- Command palette document/workspace navigation workflow: switch to an
  already-open document from the palette, open a workspace file by relative
  path, and verify active tab, editor, and file-sidebar state.
- Transform engine settings workflow: path changes clear trust, trust prompts
  appear for used external transform fences, denied trust resets the checkbox,
  input mode and timeout feed probe requests, successful probes show diagnostics
  and cache identity, and missing executables surface failure diagnostics.
- Export readiness/result workflow: target-specific readiness status and
  manifest preview, export output and manifest path reporting, success
  diagnostics, writer-failure diagnostics, blocked-readiness diagnostics before
  file write, and immediate editor edits flushed before readiness checks.
- View mode workflow: split, source, preview, focus, export preview, review,
  and presentation outline modes, including sidebar routing for export, review,
  and presentation.
- Window title workflow: clean opened document title, dirty title marker, and
  return to clean title after save.
- Theme and typography workflow: app theme, preview theme, high contrast,
  reduced motion, editor typography, preview typography, and reload
  persistence.
- Document grouping workflow: group tabs by folder and front matter document
  set, drag a loose document into an existing document set, save the generated
  `documentSet` front matter, and close a tab group without disturbing other
  groups.
- Generated-section insertion workflow: command-palette insertion for `[TOC]`,
  `[INDEX]`, `[BIBLIOGRAPHY]`, `[LIST_OF_FIGURES]`, and `[LIST_OF_TABLES]`.
- References workflow: citation command navigation plus resolved bibliography
  entries, missing citation keys, and duplicate bibliography-key reporting.
- Table editor Markdown paste import, numeric sorting, custom formula rows,
  merged-cell metadata, row and column add/remove behavior, column format
  totals, cancel-without-applying behavior, and apply-back-to-editor behavior.
- AI paste cleanup workflow: preview, insertion, quote, appendix,
  replace-selection, merge-into-section, replace-document, citation TODO,
  draft-marker, and provenance-block modes.

Required next coverage:

- Remaining file/workspace flows: multi-tab watcher switching and native
  desktop dialog behavior.
- Deeper workspace folder browsing, native document-set proof, and remaining
  tab drag/reorder edge cases.
- Focus, export, review, and presentation mode local browser/native execution proof.
- Broader keyboard shortcut coverage.
- Theme/typography visual accessibility proof in a real browser/native runtime.
- Remaining table editor flows: non-sandboxed browser execution and export
  fixture proof for edited tables.
- External conflict modal: native desktop proof and deeper manual UX QA for
  the new line-composition tray.
- AI paste cleanup remaining proof: clipboard and richer review-state flows.
- Export artifact fidelity, target-specific option matrices,
  progress/cancellation behavior if needed, and rendered/manual proof.
- Run the large-document browser performance workflow on a host with Playwright
  Chromium installed, plus native desktop performance proof.
- Remaining transform engine settings: cross-platform executable edge cases
  beyond the mocked browser workflow.

Completion criteria:

- Browser workflow tests run locally with `pnpm run test:e2e` on hosts where
  Playwright browser installation and Chromium launch are available.
- Local sandbox limitations remain documented but are not used as completion
  evidence.
- Browser coverage failures drive implementation fixes rather than broad
  selector weakening.

### 3. Add Desktop WebDriver/Tauri Workflow Smoke Tests

Status: missing.

Browser tests prove the Vue workbench under mocked Tauri IPC. They do not prove
native dialogs, real file permissions, window title behavior, real Tauri
command wiring, or desktop shell lifecycle.

Needed desktop smoke coverage:

- App boots as a Tauri desktop shell.
- New/open/save/save-as with real local files.
- Dirty title/status behavior.
- External file watcher and conflict flow with real file changes.
- Export readiness and one real export invocation.
- Preferences persistence across restart.

Completion criteria:

- Use Tauri-driver/WebDriver or another project-appropriate desktop smoke
  harness without weakening the browser harness.
- Do not add GitHub Actions for this smoke harness; keep it local/manual unless
  the project policy changes.

### 4. Maintain The Requirement Matrix And Progress Log

Status: artifacts exist; must stay synchronized.

Current artifacts:

- `docs/spec-completion-matrix.md`
- `docs/progress.md`
- `docs/todo.md`

Required maintenance:

- Update all three whenever a verified slice changes the evidence.
- Keep completion claims conservative.
- Link requirement rows to exact code paths, tests, local command output, manual artifacts,
  or screenshots.
- Record failed verification as directly as passed verification.

Completion criteria:

- No TODO item claims a feature is complete unless the matrix and progress log
  have matching current evidence.

## P1 - Product Completion And Hardening

### 5. Export Fidelity Audit

Status: broad implementation exists; artifact-level proof is incomplete.

Current evidence:

- `export_conformance_fixture_maps_business_features` now asserts release
  approval metadata and legal disclaimer fidelity across HTML, PDF, DOCX
  package properties/body text, PPTX package properties/slides, plain text, and
  Markdown bundle metadata.
- DOCX/PPTX custom package properties now include approval metadata and legal
  disclaimer fields alongside status, version, classification, client, source
  hash, and app version.
- `export_option_matrix_is_preserved_across_targets_and_bundle_evidence` now
  proves one explicit target option matrix across HTML, PDF, DOCX, PPTX, plain
  text, and Markdown bundle outputs. The matrix covers omitted styles, syntax
  highlighting, cover page, page numbers, glossary/comment/provenance
  appendices, PPTX agenda, compact layout preset, watermark, legal disclaimer
  carry-through, and exact Markdown bundle `export_options` evidence.
- `rich_markdown_blocks_survive_cross_target_exports` now proves common
  Markdown/business blocks across HTML, PDF, DOCX, PPTX, plain text, and
  Markdown bundle evidence: block quotes, callouts, unordered/nested/ordered
  lists, task lists, code blocks, tables, figures, equations, generated lists
  of figures/tables, and table/figure/equation cross references.

Audit HTML, PDF, DOCX, PPTX, and Markdown bundle outputs for:

- Headings, paragraphs, lists, nested lists, block quotes, callouts, code blocks.
  Common block evidence now exists in
  `rich_markdown_blocks_survive_cross_target_exports`; remaining work is
  rendered/manual QA and more edge-case permutations.
- Tables, merged cells, alignment, formulas, totals, captions, and large table
  pagination/splitting.
- Figures, captions, cover crop/fit, relative media packaging, duplicate media
  names, and missing media diagnostics.
- Generated `[LIST_OF_FIGURES]` and `[LIST_OF_TABLES]` sections now have a
  focused compiler/export artifact test for numbering, anchors, fenced-example
  exclusion, preview HTML, and DOCX text.
- Equations, numbering, references, and cross-target rendering.
  A focused rich-block export test now proves equation text and equation
  references across the artifact family, and a focused captioned-equation
  export test now proves equation captions survive HTML, PDF, DOCX, PPTX, and
  Markdown bundle outputs. Remaining work is rendered visual QA and more math
  syntax permutations.
- Citations, bibliography, locators, missing keys, duplicate keys, and CSL
  behavior.
- Cross references to headings, figures, tables, equations, appendices, and
  decision records.
- Glossary, index, TOC, generated sections, and appendices.
  A focused TOC export conformance test now proves front matter TOC
  depth/numbering plus page-numbered PDF lines and DOCX TOC field output.
- Review comments, change notes, release metadata, AI provenance, legal
  disclaimers, draft warnings, and approval metadata. The legal-disclaimer and
  approval-metadata path is now covered by the export conformance fixture.
- Page size, orientation, margins, columns, breaks, keep-with-next,
  keep-together, headers, footers, watermarks, cover pages, page numbers,
  brand profile, logo, colors, and fonts.
- Transform artifacts from native renderers and external engines.

Needed proof:

- Package/text assertions where appropriate.
- More target-specific option combinations beyond the current focused matrix.
- Rendered or manually inspected representative PDF/DOCX/PPTX artifacts.
- Fixture exports tied back to matrix rows.

### 6. Export Readiness Completeness

Status: implemented in part; requirement coverage needs audit.

Current evidence:

- Readiness now reports malformed review comments and change notes when author,
  timestamp, or body text is missing, and the diagnostics are copied into the
  export manifest.
- Readiness now reports incomplete AI provenance blocks and AI-assisted section
  markers when required provenance fields, human-review metadata, or known
  review statuses are missing, and the diagnostics are copied into the export
  manifest.
- Readiness now reports figures, tables, and equations that are missing stable
  labels or captions, and the diagnostics are copied into the export manifest.
- Export manifests now include an explicit readiness summary with ready, error,
  warning, and info counts.
- Direct exports now run the same dirty-Git readiness check as preflight
  reports, copy the warning into the response and sidecar manifest, and skip
  Git inspection for nonexistent source paths so unsaved/export-only documents
  do not inherit the app process working tree state.
- Direct exports now block target/output extension mismatches before writing,
  preventing artifacts and sidecar manifests from being created under the wrong
  target extension.
- `prepare_for_export_carries_broad_readiness_audit_to_manifest` now exercises
  a broad readiness audit in one report: missing title/version, missing
  approval metadata, unsupported layout metadata, citation source gaps,
  unresolved comments, malformed change-note metadata, incomplete AI
  provenance, pending AI review, broken image and link paths, inline and table
  formula errors, figure/table/equation caption-label warnings, transform
  timeout/path/trust/input-mode errors, and pending render/manifest progress.
- Export manifests now carry the include graph as first-class evidence, and
  Markdown bundles include `include-graph.json` in addition to `manifest.json`
  and `include-map.json`; focused Rust coverage proves both include directives
  and front matter data-source edges appear in export evidence.
- PPTX readiness now has target-specific validation: unapproved presentation
  exports are blocked unless status is approved/published and `approvedBy` plus
  `approvedAt` are present, and the diagnostic is copied into manifest
  readiness.
- Target-specific option audits now report non-blocking info diagnostics when
  valid options are ignored by the selected target, including non-PPTX agenda
  options and Markdown bundle render-only options.
- Readiness now warns when generated index or glossary sections are requested
  but no index terms or glossary entries exist, and the diagnostics are copied
  into export manifest readiness.

Readiness should validate and report:

- Required metadata. Broad readiness audit test coverage exists.
- Release status and approval metadata. Broad readiness audit test coverage
  exists.
- Draft/export warnings.
- Includes and include graph. Export manifests and Markdown bundles now carry
  include graph evidence, focused compiler coverage proves missing include,
  circular include, maximum include depth, and invalid/unreadable include target
  diagnostics, and the workbench now exposes parent-child include graph
  navigation from the References sidebar and command palette. Remaining work is
  current-host browser execution, native workflow proof, and broader platform
  coverage.
- Broken local links and missing media. Broad readiness audit test coverage
  exists.
- Citations, bibliography files, missing keys, duplicate keys, and style
  issues. Broad readiness audit test coverage now covers missing bibliography
  source; separate citation tests cover missing and duplicate keys.
- Formulas, table formulas, dependencies, circular references, and invalid
  expressions. Broad readiness audit test coverage exists for inline and table
  formula errors.
- Figures, captions, references, glossary, and index. Broad readiness audit
  test coverage exists for figure/table/equation caption-label warnings, and
  focused readiness coverage now reports requested-but-empty generated index
  and glossary sections.
- Transform engines, trust state, executable paths, adapter input mode,
  timeouts, stderr, missing output, output limits, and cache identity. Broad
  readiness audit test coverage exists for export settings; external transform
  tests cover runtime stderr, missing output, output limits, and cache identity.
- Export target options and target-specific blockers. Focused Rust coverage now
  proves the PPTX approved-metadata blocker; more target-specific option
  combinations remain.
- Target-specific option no-op visibility. Focused Rust coverage now proves
  valid-but-ignored options are surfaced as info diagnostics without blocking
  readiness.
- Target/output extension consistency. Focused Rust coverage now proves direct
  exports refuse mismatched target extensions before writing artifacts.
- Unresolved comments and malformed comment/change-note audit metadata. Broad
  readiness audit test coverage exists.
- AI provenance that is not human reviewed or lacks required audit metadata.
  Broad readiness audit test coverage exists.
- Dirty Git state and export manifest state. Direct export and preflight export
  paths now both cover dirty Git; remaining work is broader readiness coverage
  and UI/native proof.

Completion criteria:

- Readiness UI exposes actionable diagnostics.
- Export commands block or warn consistently with readiness results.
- Export manifests include enough readiness context for auditability.

### 7. External Transform Platform Evidence

Status: Linux installed-engine evidence passed historically, this macOS host
has current local Graphviz/DOT, D2, and PlantUML evidence, and the Graphviz
adapter now exposes the requested `dot`, `graphviz`, `circo`, `neato`, `fdp`,
`osage`, and `twopi` transform names. Pikchr on macOS and all Windows
optional-engine evidence remain incomplete.

Finish:

- Preserve installed-engine conformance locally where engines are available
  while expanding optional engine proof beyond Linux.
- Keep Graphviz/DOT variants, D2, PlantUML, and Pikchr as real optional-engine
  proof where available.
- macOS evidence now verifies Graphviz `dot`, `circo`, `neato`, `fdp`,
  `osage`, `twopi`, D2, and PlantUML through `pnpm run check:engines` and
  `cargo test --locked external_transform_conformance_runs_installed_engines --lib -- --nocapture`;
  Pikchr remains missing on this host.
- PlantUML now supports fence-level PNG output selection through `format=png`,
  `output=png`, or the `png` flag, with `compiler_uses_plantuml_png_fence_output_format`
  proving the trusted file-mode sidecar path and PNG data URL artifact.
- Add Windows manual evidence for all optional engines.
- Confirm Windows `.exe` paths and package-manager shims.
- Confirm PlantUML SVG and PNG file mode on all platforms.
- Confirm Pikchr stdin/file/argument mode for each supported executable shape.
- Cache invalidation now includes adapter identity, executable path, executable
  file size/mtime, adapter arguments, input mode, renderer version, and source
  hash; `external_transform_cache_invalidates_when_trusted_executable_changes`
  proves same-path executable rewrites do not serve stale cached output.
- External engines now have an explicit per-engine disabled setting. Disabled
  engines are skipped before trust/path execution checks, fall back to embedded
  rendering when available, and avoid trust-failure noise.
- Preserve diagnostics for missing executable, non-executable path, timeout,
  bad syntax, sidecar not produced, output limit, and stderr warnings.
- Keep `docs/external-transform-platform-evidence.md` current when optional
  engines are installed, upgraded, or verified on another platform.

### 8. File Watcher And Conflict Workflows

Status: backend and UI exist; stale-save conflict copy/merge/keep-local/
accept-external workflows are covered by the local Playwright harness, with
merge composition add/reorder/remove assertions now listed in the conflict
merge workflow. Clean watcher reload and watcher-originated dirty root-file
conflict proof are also present in archived browser workflow evidence. Clean
included-file recompile and dirty included-file conflict proof are also present
in archived browser workflow run `26145509141`.

Finish:

- Clean external reload for unchanged local documents. Browser archived workflow run
  `26140882880` covers this for root-file changes.
- Dirty root-file conflict through UI. Browser archived workflow run `26140882880` covers this
  for watcher-originated root-file changes.
- Dirty included-file conflict and master recompilation through UI. Browser CI
  run `26145509141` covers clean included-file recompile and dirty
  included-file conflict handling.
- Save-race conflict when a file changes after the last watcher event but
  before save. The local Playwright harness lists the stale-save conflict path
  through compare, save-copy preservation, merge-back recovery with explicit
  line composition controls, keep-local, and accept-external; current-host
  execution remains blocked by the missing Playwright Chromium binary.
- Multi-tab watcher switching beyond the current tab-activation proof.
- Stale watcher cleanup when tabs close or paths move beyond current
  recent-path cleanup coverage.
- Include graph changes after editing include directives.

### 9. Workspace, Tabs, And Document Sets

Status: tabs, pinned tabs, recents, recently closed items, workspace browsing,
and restore logic exist; spec-level proof is incomplete.

Finish:

- Folder/workspace/project grouping behavior. The browser harness now covers
  folder grouping for open documents and document-set grouping from front
  matter metadata.
- Remaining document-set edge cases: native execution proof, saved workspace
  package evidence, and deeper drag/reorder behavior beyond moving a loose tab
  into an existing set.
- Restart restore of previous workspace, active tab, mode/sidebar state, and
  pinned state. Browser archived workflow run `26147556750` covers this workflow.
- Scroll position restore. Browser archived workflow run `26148828614` covers this workflow.
- Recently closed behavior for renamed and deleted files, plus dirty unsaved
  close confirmation. Browser archived workflow run `26151184228` covers this workflow.
- Recent folder reopening and stale recent-folder pruning, plus externally
  moved recently-closed file pruning. Browser archived workflow run `26152255407` covers this
  workflow.
- Clear UX for missing documents during restore. Browser archived workflow run `26148828614`
  covers this workflow.
- Matrix entry that split editor panes are deferred/later if not implemented.

### 10. Editor Ergonomics

Status: CodeMirror is integrated; browser proof now covers the first
interaction slice in archived workflow run `26154535588`.

Covered:

- Line numbers toggle and persistence.
- Word wrap toggle and persistence.
- Basic Markdown list continuation.
- Basic bracket auto-pairing.
- Quote, bold, italic, inline-code pairing, and code-fence insertion through
  explicit Markdown commands.
- CodeMirror find and replace.
- Spellcheck and sentence autocapitalization editor attributes.
- Word count, character count, and reading-time status-bar metrics.
- Command-palette heading navigation.
- Outline sidebar heading navigation to source.
- Diagnostics-panel source navigation with line/column metadata.
- Explicit multi-cursor command palette actions for add-cursor-above,
  add-cursor-below, and select-next-occurrence, with a browser harness workflow
  that edits matching lines through multiple cursors.

Finish:

- Markdown syntax highlighting.
- Diagnostics gutter/range visual-state local browser/native execution proof.
- Broader Markdown shortcut edge cases.
- Markdown shortcut local browser/native execution proof.
- Spellcheck behavior local browser/native execution proof.
- Word/character/reading-time local browser/native execution proof.
- Outline navigation local browser/native execution proof.
- Multi-cursor support local browser/native execution proof.
- Vim/emacs keybindings classification: deferred unless intentionally added.

### 11. Preview Ergonomics

Status: live preview exists; detailed workflow proof is incomplete.

Finish:

- Debounced rendering behavior on large documents.
- Editor/preview synchronized scrolling.
- Preview heading click jumps to source.
- Separate preview theme behavior.
- Inline warning rendering. The preview now injects escaped diagnostic callouts
  into the rendered flow and delegates source jumps back to the editor.
- Transform-aware preview artifacts. The preview pane now summarizes transform
  artifacts, cache keys, execution mode, diagnostics, and source jumps.
- Print/export preview mode behavior. Export mode now adds an export preview
  summary for the selected target, readiness state, manifest counts, and export
  options.
- Preview accessibility and keyboard navigation.

### 12. AI Paste Cleanup And Governance

Status: backend cleanup and UI exist; governance workflows need proof.

Finish:

- Browser tests for clipboard/rich paste and provenance toggles. Insert, quote,
  appendix, replace document, merge into section, replace selection, citation
  TODO, draft marker, and provenance block workflows are now covered in the
  browser harness locally.
- Rich clipboard paste behavior where the runtime supports it.
- Provenance block aliases.
- AI-assisted section review-state toggles. The Review sidebar toggles
  `ai-source` blocks and AI-assisted section markers between needs-review and
  human-reviewed states.
- Readiness/export warnings for unreviewed AI content. The browser harness now
  covers an unreviewed AI provenance warning and verifies it clears after
  review-state toggles.
- Export appendix behavior for AI provenance. Backend export option tests
  prove appendix inclusion/exclusion across targets; remaining work is richer
  rendered/native QA.

### 13. Tables, Calculations, And Data Sources

Status: broad implementation exists; workflow and artifact proof need expansion.

Finish:

- Table editor workflows for paste, add/remove rows/columns, sort, format,
  totals, merged cells, readable Markdown output, and cancellation.
- Named table/range references and formula dependency graph proof. Focused Rust
  coverage now proves named table/range formulas compile and survive HTML, PDF,
  DOCX, PPTX, Markdown bundle text, and Markdown bundle AST outputs.
- Inline formulas and table-cell formulas in preview/export/readiness.
- Data sources from front matter and external CSV/TSV/JSON/YAML paths. Backend
  coverage now proves CSV, TSV, JSON, and YAML local file sources through
  compile, transform artifacts, manifests, and cross-target export outputs.
- Validation for malformed data source paths, broken formulas, circular or
  unsupported dependencies, and mixed span/formula tables. Data source coverage
  now proves missing path, unsupported type, and unreadable file diagnostics.
- Export parity for large, merged, formatted, summarized, sorted, and
  formula-driven tables.

### 14. Bibliography, Citations, Index, Glossary, And Cross References

Status: core support exists; UI and cross-target proof remain incomplete.
Readiness now reports missing citation bibliography entries with precise
source ranges when citations are present but no bibliography source is
available. Duplicate bibliography entries now carry source locations, readiness
points at duplicate entries for BibTeX, CSL JSON, Hayagriva YAML, and multiple
external bibliography files, and the References panel displays duplicate entry
locations. Citation style handling now supports title, author-year, key, and
numeric styles, and unsupported CSL/style names produce a warning before
falling back to title rendering.

Finish:

- BibTeX and CSL JSON import edge cases.
- Duplicate bibliography key UI and readiness reporting. Current coverage shows
  duplicate entry locations and source-range readiness diagnostics for BibTeX,
  CSL JSON, Hayagriva YAML, and separate external bibliography files; remaining
  work is richer citation manager UX.
- Citation styles: title, author-year, key, numeric, and CSL-driven choices.
- Remaining citation diagnostics: richer CSL style validation and a future
  Rust-native CSL/Hayagriva adapter for named CSL styles beyond the built-in
  options.
- Cross-reference links across preview, HTML, PDF, DOCX, PPTX, and bundle
  outputs. Current focused coverage proves table, figure, equation, section,
  appendix, and decision references across target artifacts, with appendix and
  decision anchors rendered as semantic labels instead of generic sections.
- Automatic index inclusion/exclusion. Current focused coverage proves
  `[INDEX]`, `index: true`, `index.enabled: true`, linked heading/glossary/bold/
  proper-noun/explicit terms, exclusion settings, marker stripping, and
  cross-target export artifacts. The References sidebar now provides an Index
  manager for generated index insertion, front matter index enablement, and
  detected-term navigation. Readiness now warns when a generated index is
  requested without any terms, and copies the warning into export manifests;
  remaining work is native workflow execution and rendered/manual UX QA.
- Glossary definition preview, hover behavior, generated-section marker,
  front matter insertion, export appendix behavior, and command-palette
  navigation. Current focused coverage proves preview hover, `[GLOSSARY]`,
  front matter insertion, DOCX artifact text, export appendices, and command
  palette listing. The References sidebar now provides a Glossary manager for
  generated glossary insertion, definition templates, export glossary enablement,
  term navigation, and adding glossary terms to the index. Readiness now warns
  when a generated glossary is requested without entries, and copies the warning
  into export manifests; remaining work is native workflow execution and
  rendered/manual UX QA.

### 15. Layout And Reflow

Status: layout directives and paged-document output exist; real quality proof is
incomplete.

Finish:

- Parser coverage for page breaks, section breaks, columns, margins,
  orientation, page size, keep-with-next, keep-together, headers, footers, and
  slide directives.
- Paged-document model evidence for each directive.
- Export mapping evidence for HTML, PDF, DOCX, PPTX, and Markdown bundle.
- Visual/manual review of representative PDF/DOCX/PPTX outputs.
- Overflow handling for large figures, equations, tables, code blocks, and long
  unbroken words.

### 16. Accessibility

Status: partial automated guard exists; full accessibility audit remains open.

Current evidence:

- `pnpm run check:a11y` checks `src/App.vue` for accessible button names,
  form-control labels, and dialog labels/modal state.
- Modal close buttons, command-palette search, and conflict merge-line controls
  now have explicit labels exposed to assistive technology.

Finish:

- Keyboard-only navigation through toolbar, sidebar, tabs, editor, preview,
  modals, command palette, table editor, and conflict UI.
- Focus management in all modals.
- ARIA labels and roles for custom controls.
- High contrast and reduced motion behavior.
- Screen-reader labels for diagnostics, status messages, table cells, conflict
  diff rows, and export progress.
- Broader automated checks where practical plus manual checklist evidence.

### 17. Performance And Large Documents

Status: compiler, repeated export-loop, repeated edit/cache, export progress,
compile-result cancellation, preview debounce timing, and repeated
compile/export memory-growth evidence exists; broader performance proof remains
open.

Current evidence:

- `performance_tests::compiler_stress_handles_large_documents_with_many_artifacts`
  stress-compiles a large Markdown document with nested includes, many tables,
  formulas, native transform artifacts, source-map entries, and broken
  link/media diagnostics.
- `performance_tests::repeated_export_loop_keeps_large_artifacts_stable`
  repeatedly renders a large compiled document through HTML, PDF, DOCX, PPTX,
  and Markdown bundle outputs, asserting target structure, meaningful content,
  non-trivial artifact sizes, and bounded size drift across three export loops.
- `performance_tests::repeated_editing_sessions_reuse_external_transform_cache`
  repeatedly recompiles edited document text while keeping a trusted external
  DOT transform stable, proving memory and persistent cache reuse without
  re-running the external engine after the first compile.
- `export_command_tests::export_readiness_and_manifest_report_progress_steps`
  proves readiness/export manifests carry compile, transform, readiness,
  render, and manifest progress steps, and the export UI displays readiness and
  last-export progress steps.
- `tests/frontend-unit.test.ts` covers the latest-document task guard used by
  preview compilation so stale or cancelled compile results cannot overwrite
  newer editor state. The workbench also exposes a status-bar cancel action
  while preview compilation is pending.
- `tests/frontend-unit.test.ts` covers the preview debounce helper, asserting
  `PREVIEW_DEBOUNCE_MS` remains within the 100 ms small-document preview budget
  and proving rapid edits coalesce before commit while explicit flushes commit
  immediately.
- `performance_tests::repeated_compile_export_cycles_keep_memory_growth_bounded`
  repeatedly compiles and renders large documents through HTML, PDF, DOCX,
  PPTX, and Markdown bundle paths while retaining only summaries and asserting
  process RSS growth remains bounded on macOS/Linux hosts.
- `e2e/app-workflows.spec.ts` includes a large-document editing workflow that
  appends source text, waits for preview update, checks elapsed browser time,
  and verifies editor-to-preview scroll sync. Local execution still requires
  installing the Playwright Chromium binary.

Finish:

- Deeper long-running memory profiling beyond bounded test loops.
- Run the large-document browser workflow where Playwright Chromium is
  installed, and add native desktop performance proof.

## P2 - Architecture And Maintainability

### 18. Frontend Modularization

Status: needed after workflow behavior is locked.

`src/App.vue` is too large for low-risk continued feature work. Do not split it
before the important workflows have browser tests.

Candidate extraction targets:

- Toolbar and command palette.
- File/workspace sidebar.
- Outline/references/sidebar navigation.
- Diagnostics panel.
- Export and readiness panel.
- Snapshot/versioning/review panels.
- Settings and transform settings.
- AI paste cleanup modal.
- Table editor modal.
- External conflict modal.
- CodeMirror setup, decorations, linting, and navigation helpers.

Constraints:

- Preserve Vue SFC block order: template, script, style.
- Keep diffs small and behavior-preserving.
- Avoid styling churn during structural splits.

### 19. Frontend Store Modularization

Status: needed after workflow behavior is locked.

`src/stores/documents.ts` owns too many domains.

Candidate split:

- Documents, tabs, recents, workspace restore.
- Compile, diagnostics, preview state.
- Export, readiness, manifests.
- Watcher and conflict resolution.
- Git, snapshots, versioning.
- Transforms and preferences.
- AI cleanup and review governance.

- Current migration evidence: `src/lib/workspacePersistence.ts` owns the
  schema version and normalizes/clamps persisted settings before the store uses
  them; `tests/frontend-unit.test.ts` covers legacy aliases, limits, records,
  transform settings, AI cleanup defaults, and scroll-position normalization.
- Preserve persisted workspace schema and keep migrations explicit as domains
  are split.
- Split only with regression tests around the moved behavior.

### 20. Backend Modularization

Status: continue opportunistically after local verification is green.

Large/high-risk files:

- `document_ast.rs`
- `export/docx.rs`
- `export/pdf.rs`
- `export/pptx.rs`
- `export/shared.rs`
- `transforms/external.rs`
- Large nested test modules

Guidance:

- Move cohesive helpers rather than adding abstract layers.
- Prefer deletion and simplification over new framework code.
- Keep backend tests green after each split.

### 21. Reduce String-Heavy Compiler And Export Paths

Status: partial semantic AST exists; string scanning remains in correctness
paths.

Improve:

- Use AST nodes for references, figures, tables, equations, layout, review
  comments, AI provenance, and transform artifacts.
- Use structured transform artifacts for export mapping.
- Avoid parsing generated HTML for table/media semantics where AST data exists.
- Keep raw string parsing only where Markdown syntax has no structured model
  yet, and document the reason.

### 22. Dependency Admission Records

Status: exists; keep current.

Finish:

- Keep `docs/dependency-admission.md` updated for Playwright and any future
  test/runtime dependencies.
- Revisit `@tauri-apps/plugin-shell` if external transforms remain entirely
  Rust-backend driven.
- Confirm licenses, runtime impact, alternatives, and security posture for all
  new dependencies.

## P3 - Packaging, Release, And User Documentation

### 23. Cross-Platform Packaging Evidence

Status: desktop shell compile is tested; full bundle evidence is incomplete.

Finish:

- Refresh macOS app bundle evidence on the current commit.
- Classify the documented macOS DMG `hdiutil create` failure as host-specific
  or config-specific.
- Add Windows package evidence for the chosen `.msi`/`.exe` target.
- Add Linux package evidence for AppImage/deb/rpm or the chosen bundle target.
- Confirm icons, bundle identifier, app metadata, signing/notarization stance,
  and updater stance.

### 24. User Documentation

Status: README and technical docs exist; first-user docs are incomplete.

Current evidence:

- `docs/specification.md` now has a checked-in `docs/architecture.svg` target
  for the figure/caption example.
- `README.md` links the user-facing documentation set.
- `docs/user-guide.md` now covers first-run setup, workbench modes, local file
  safety, includes, Markdown extensions, tables/calculations, citations,
  review and AI governance, transform trust, export manifests, versioning, and
  troubleshooting.
- `docs/markdown-extensions.md` now provides a readable syntax reference for
  front matter, includes, generated sections, variables, variable filters,
  formulas, tables, figures, equations, citations, glossary/index, comments,
  AI provenance, transforms, native visual-data transform subsets, and export
  readiness markers.
- Native visual-data subset documentation now reflects typed GeoJSON geometry
  previews and TopoJSON object arc-reference resolution, including reversed
  arcs.
- First-release safe business transforms are now explicit: `roadmap`, `adr`,
  `diff`, and `qr` are documented native transforms with cross-target artifact
  proof, while execution-heavy second-wave transforms remain deferred until
  each has a safe sandbox or static renderer.
- Visual/data transform export proof now covers `chart`, `vega-lite`,
  `geojson`, `topojson`, `stl`, and `timeline` static SVG artifacts, output
  hashes, source ranges, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence.
- BibTeX transform proof now covers inline title/author/year/date metadata
  parsing, richer preview rendering, cross-target export artifacts, bibliography
  metadata JSON, source ranges, and output hashes.
- OpenAPI and JSON Schema transform proof now covers structured documentation
  artifacts, semantic table caption preservation, output hashes, source ranges,
  HTML, PDF, DOCX, PPTX, and Markdown bundle evidence.
- OpenAPI rendering now covers security requirements, deprecated operations,
  external docs, response media examples, response headers, response links, and
  security schemes; JSON Schema rendering now covers pattern properties, tuple
  `prefixItems`, dependent required/dependent schemas, `additionalProperties`,
  and `oneOf` composition across the export artifact family.
- OpenAPI callback, webhook, and discriminator details now flow through the
  native renderer and export artifacts; JSON Schema conditional branches and
  `$defs` rows now flow through the same semantic-table export path.
- Front matter documentation now calls out that metadata must be a YAML mapping
  and that invalid YAML/list/scalar metadata produces source-ranged diagnostics.
- `pnpm run check:docs` discovers README plus all top-level docs Markdown files
  and checks missing local links.

Needed docs:

- Expand target-specific export option examples and rendered artifact
  screenshots as export fidelity proof grows.
- Add desktop/native workflow screenshots after the native smoke harness exists.
- Keep user docs synchronized with matrix rows as features move from partial to
  complete.

### 25. Example Project Fixtures

Status: complete for current fixture set.

Current realistic sample projects:

- `examples/board-paper.md`: executives and managers; board paper, review
  comment, chart, glossary, index.
- `examples/consulting-report.md`: consultants; consulting report with includes
  and roadmap.
- `examples/technical-architecture.md`: technical writers, product and
  engineering teams, developers; architecture document with diagrams, timeline,
  and ADR.
- `examples/research-report.md`: researchers and analysts, students and
  academics; bibliography, citations, equation, and cross-reference.
- `examples/proposal-budget.md`: consultants plus product and engineering
  teams; budget table, `calc`, CSV, and formulas.
- `examples/ai-assisted-draft.md`: teams using AI chat output plus product and
  engineering teams; AI provenance and review workflow.

Completion criteria:

- Keep every target persona in `docs/specification.md` section 4 represented by
  at least one executable fixture.
- Keep every fixture exporting to HTML, PDF, DOCX, PPTX, and Markdown bundle.
- Keep README links to the examples current.

## Recommended Execution Order

1. Expand browser workflow coverage for export artifact fidelity,
   target-specific option matrices, progress/cancellation behavior if needed,
   broader keyboard shortcuts, deeper workspace grouping, remaining preview
   modes, and remaining AI provenance/table export modes.
2. Add desktop WebDriver/Tauri smoke tests.
3. Use workflow failures to close real implementation gaps.
4. Audit export artifacts and add conformance fixtures.
5. Harden cross-platform external transform evidence.
6. Run an independent security review against `docs/security-threat-model.md`.
7. Prove native settings/snapshot storage and sidecar workflows in a real
   desktop run.
8. Modularize frontend/store/backend code after behavior is locked.
9. Complete packaging evidence, user docs, and example projects.
10. Run a final requirement-by-requirement audit and fresh verification baseline.

## Completion Gate

Do not mark the full NEditor buildout complete until all of the following are
true:

- Every explicit requirement in `docs/specification.md` and
  `docs/external-transforms.md` is implemented and verified, or explicitly
  documented as deferred/non-goal with rationale.
- `docs/spec-completion-matrix.md` has current evidence for every requirement.
- `docs/progress.md` has current verification results.
- Backend tests, frontend tests, typecheck/build, clippy, native-watch check,
  browser workflow tests where locally runnable, and desktop compile pass on
  the current commit through local verification.
- Browser workflow coverage proves the main mocked-IPC frontend journeys.
- Desktop workflow coverage proves the main native Tauri journeys.
- Export fixtures prove the required business-document outputs.
- Cross-platform optional engine behavior is documented and tested where
  practical.
- Accessibility, performance, and packaging have explicit evidence or
  documented release limitations.
- The worktree is clean and pushed.
