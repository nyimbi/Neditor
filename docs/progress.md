# NEditor Goal Progress Log

Updated: 2026-05-21

## Active Goal

Complete the full development of NEditor against:

- `docs/specification.md`
- `docs/external-transforms.md`

The goal is not complete until the implementation, documentation, tests,
workflow verification, export artifacts, platform evidence, and committed
progress records prove the requested end state.

## Current Repository State

- Branch: `main`
- Latest inspected committed baseline before this update: `5542f8f Block
  ambiguous reference labels before export`
- Remote alignment at inspection time: `main...origin/main`
- Worktree before this log update: clean

## Durable Planning Artifacts

- `docs/todo.md`: current prioritized completion backlog.
- `docs/spec-completion-matrix.md`: conservative spec-to-evidence matrix.
- `docs/progress.md`: this committed progress log.

## Completed Recently

Recent pushed checkpoints visible in current git history:

- `7aff68f` announced workbench status changes accessibly.
- `8d73d69` managed focus across workbench modals.
- `4ce66b5` improved keyboard access to workbench regions.
- This update labels compiler, readiness, and export diagnostics as
  screen-reader list items, labels conflict diff cells as side-specific groups,
  and extends the static accessibility guard plus browser harness assertions for
  those names.
- This update also names the table editor grid, row/column control groups, and
  totals so screen-reader users can navigate the dense table editing surface by
  structure rather than visual position alone.
- This update adds `pnpm run test:desktop-smoke`, a local desktop artifact smoke
  check for the Vite build, Tauri config, package/license metadata, and release
  desktop binary produced by `./node_modules/.bin/tauri build --no-bundle`. The
  same harness has an opt-in bounded GUI launch mode for hosts that permit
  desktop app startup.
- This update hardens high-contrast and reduced-motion accessibility by adding
  static CSS guardrails for black-on-white high-contrast colors, focus outlines,
  and zero-duration motion, plus browser harness assertions for the computed
  settings state.
- This update gives the primary source and preview surfaces deeper
  screen-reader semantics: CodeMirror content is now a named multiline textbox,
  and the rendered preview article is a focusable named document.
- This update blocks duplicate reference labels across headings, figures,
  tables, equations, appendices, and decisions. The compiler now emits
  source-ranged duplicate-label diagnostics with first-occurrence context, and
  export readiness copies those diagnostics into manifests so ambiguous cross
  references cannot leave the editor as audit-ready deliverables.
- This update also hardens export option readiness: malformed default citation
  styles, non-hex brand colors, malformed default brand profile fields,
  dirty-Git warning flags, and legacy cover/page-number aliases now fail
  readiness with manifest-backed diagnostics instead of being ignored.
- This update also makes appendix export options content-aware: enabled
  glossary, review-comment, or AI-provenance appendices now produce
  manifest-backed info diagnostics when the document has no matching content,
  while populated appendices remain clean.
- This update tightens the dependency-admission guard so NEditor stays MIT
  licensed in the root `LICENSE`, npm package metadata, Cargo crate metadata,
  and Tauri desktop bundle metadata.
- `25bc28f` added titlebar release status visibility, Versioning-panel snapshot
  create/list/restore controls, and browser harness coverage for snapshot
  restore plus release tagging workflows.
- `929043c` constrained snapshot restores to active-document snapshot stores and
  matching metadata.
- `a57767b` surfaced malformed front matter diagnostics with source ranges.
- `2231776` added the first-user guide, Markdown extensions reference, README
  links, and automatic top-level docs link checking.
- `1c532ee` constrained Git restore/tag inputs to explicit safe refs, blocked
  symlink restore targets, and recorded the security evidence.
- `3c10a2c` documented storage and security boundaries, linked them from the
  README, and added markdown link checking for the new docs.
- `2b252c5` versioned workspace persistence migrations and covered legacy
  settings normalization in frontend unit tests.
- `c10da06` bounded large-document performance evidence with repeated
  compile/export memory-growth coverage.
- `ca94c90` proved preview debounce timing and coalescing behavior.
- `3214d1f` refreshed `docs/todo.md` from current evidence.
- `a93a974` recorded the spec completion matrix and durable progress log.
- `237f68c` logged the fresh verification baseline.
- `dee18fc` extracted shared workflow insertion helpers and added frontend unit
  coverage for AI paste insertion modes and conflict merge-line composition.
- `d94dc6c` added the first Playwright browser workflow harness, admitted the
  test dependency, wired the browser workflow CI lane, and refreshed this
  backlog/progress evidence.
- `6489162` installed pnpm before `actions/setup-node` enables `cache: pnpm` in
  CI.
- `420af08` kept the readiness browser workflow assertion strict by scoping the
  exact `Ready` match to the readiness panel.
- `9a6d52e` recorded the first passing Linux browser workflow CI lane in the
  durable docs.
- `5ce5b99` rewrote the active backlog around the latest survey and CI evidence.
- `25f7b04` Unix-gated the external transform test cache helper and attempted a
  `pikchr-cli` positional source compatibility path.
- `5c29914` normalized serialized file/export paths to `/` separators and
  changed the `pikchr-cli` adapter to pass a temporary `.pikchr` source file.
- `33ee6a9` made the fake `d2` adapter fixture drain stdin on Linux and
  refreshed the backlog, progress log, and completion matrix from current CI
  evidence.
- `443515b` added advanced table editor browser workflow coverage for Markdown
  paste import, numeric sorting, formula rows, merged-cell metadata, and
  apply-back behavior, with local unit/build proof and green CI in run
  `26134248308`.
- `e2f22d5` added a mocked Tauri file/dialog layer and browser workflow proof
  for open, edit, save, duplicate, rename, pin, reveal, workspace listing, and
  revert.
- `f5d5e9a`, `5b7a756`, `85ad6db`, and `613d880` tightened that workflow after
  CI exposed brittle selectors and edit-state assumptions.
- `11dafc3`, `a55970d`, `b7534f6`, and `12cd667` stabilized the mocked file
  lifecycle browser proof by checking saved mock contents, activating the files
  sidebar before workspace-row assertions, and pinning from the active document
  tab.
- `b2ccf83` refreshed the backlog, progress log, and completion matrix from
  the then-current survey before the final browser proof fixes.
- `61eff87`, `44c9639`, and `138bf5d` added and stabilized browser workflow
  proof for save-as to a new path and reopening that saved document from the
  recently closed list. The settings recent-path lists now have accessible
  section labels so repeated path buttons are unambiguous.
- `3cb1b84` and `25c7d1e` added and stabilized browser workflow proof for
  stale-save conflict recovery: save blocking when disk content changed, compare
  dialog visibility, local conflict-copy preservation, and merge-back recovery
  into the original file.
- `4eb1d2c` added browser workflow proof for the remaining stale-save conflict
  action buttons: keep-local preserves editor edits without overwriting the
  changed disk file, and accept-external replaces the active document with the
  external disk content.
- `e0ac31c` added browser workflow proof for watcher-originated root-file
  changes: clean documents reload external edits automatically and dirty
  documents open the non-destructive compare flow.
- `1ac72c1` preserved the clean reload status message after watcher-triggered
  compilation, fixing the CI regression from the first watcher workflow proof.
- This update adds table-draft cancellation, accessible table row/column
  controls, and browser workflow coverage for column formats, add/remove row,
  add/remove column, cancel-without-applying, and apply-back behavior.
- `15b7df6` kept fenced citation examples literal.
- `58ae0fd` shared table cell span normalization.
- `f157fbf` let the table editor author merged cells.
- `2c0253e` preserved imported table spans in semantic exports.
- `ae90d2c` carried merged table semantics through exports.
- `76fe169` forced optional diagram conformance in Linux CI.
- `51071fd` split tall PDF table cells through row fragments.
- `42f4298` covered PDF table continuation across page geometries.
- `45ff66e` split PDF paragraphs with widow and orphan control.
- `1f7c576` let PDF text flow beside figure floats.
- `d20cfde` shared fenced-code detection across compiler scanners.
- `a712da9` kept headings and references out of fenced examples.
- `3a3e009`, `5a13fe2`, and `7b549c0` restored workspace scroll positions,
  warned clearly about missing restored files, and recorded the green CI proof.
- `44c49f5`, `3fe0c78`, `39a3286`, and `bf60405` added and stabilized browser
  workflow proof for tab activation, dirty-close confirmation, renamed recent
  cleanup, and deleted recently-closed pruning.
- `13b3086` added browser workflow proof for reopening recent folders, pruning
  missing workspace roots from recent folders, and pruning externally moved
  recently closed document paths while the moved target remains visible through
  workspace refresh.
- `7702e89` added browser workflow proof for editor-to-preview scroll sync,
  preview-to-editor scroll sync, and rendered preview heading click-through to
  the CodeMirror source line.
- `f13c3f3` added browser workflow proof for persisted editor word-wrap and
  line-number settings, CodeMirror find/replace, smart list continuation,
  bracket auto-pairing, and command-palette heading search/navigation.
- `b881246` recorded the editor ergonomics workflow proof in the durable docs.
- `5f75e44` added browser workflow proof for command-palette citation,
  glossary, and index navigation, and made glossary palette entries jump to the
  matching source term while opening the References panel.
- `145942a` added browser workflow proof for command-palette open-document
  switching and workspace-file opening, including active tab, editor, and file
  sidebar state.
- `3b7a2fe` and `976016c` added and stabilized browser workflow proof for
  external transform engine settings: path-change trust clearing, trust
  prompts, denied trust reset, input mode and timeout persistence, successful
  probe details, cache identity, and missing-executable diagnostics.
- `03bda13`, `18ff4a4`, `e1e1b2a`, and `02e832a` added and stabilized export
  readiness/result workflow proof: target-specific readiness manifest preview,
  output and manifest path reporting, success and writer-failure diagnostics,
  blocked export diagnostics before file write, and an immediate editor-state
  flush before export readiness runs.
- This update adds `docs/ipc-command-coverage.md` and
  `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented`,
  which parse `docs/specification.md` section 25.4, the coverage table, and the
  Tauri `generate_handler!` registration so required IPC commands cannot drift
  out of the app shell unnoticed.
- This update also records the project license as MIT in `LICENSE`,
  `package.json`, `src-tauri/Cargo.toml`, and the Tauri desktop bundle
  metadata, and removes GitHub Actions as an active verification surface.
- This update expands the example fixtures with explicit target-persona front
  matter and strengthens
  `example_fixture_tests::example_project_fixtures_compile_and_export` so the
  target-user requirement is backed by executable compile/export proof.
- This update strengthens the Playwright AI paste workflow harness so
  citation TODO insertion, draft review markers, and `ai-source` provenance
  blocks are covered through the preview and insertion UI path.
- This update makes multi-cursor editing explicit through command-palette
  actions for add-cursor-above, add-cursor-below, and select-next-occurrence,
  plus a Playwright workflow harness case for editing two lines through
  multiple cursors.
- This update extends the editor ergonomics workflow harness with an exact
  status-bar assertion for word count, character count, and reading-time
  metrics.
- This update adds outline-sidebar navigation coverage, proving the sidebar
  can focus a deep heading back in the Markdown source outside the command
  palette path.
- This update adds editor workflow harness assertions for the CodeMirror
  `spellcheck` and `autocapitalize` content attributes.
- This update adds explicit Markdown format commands for bold, italic, inline
  code, and code fences, with workflow harness coverage for quote pairing,
  emphasis pairing, inline-code pairing, and fence insertion.
- This update adds diagnostics-panel workflow coverage for a compiler
  diagnostic with line/column metadata and source navigation back into the
  editor.
- This update wires export, review, and presentation modes to their expected
  sidebars and hides the source pane for presentation outline mode, with
  workflow harness coverage for split, source, preview, focus, export, review,
  and presentation modes.
- This update adds workflow harness coverage for browser window title updates
  across clean opened files, dirty edits, and post-save clean state.
- This update adds workflow harness coverage for app theme, preview theme, high
  contrast, reduced motion, editor typography, preview typography, and reload
  persistence.
- This update adds workflow harness coverage for live preview updates after
  typing new source content.
- This update adds workflow harness coverage for tab grouping by folder and
  front matter document set, dragging a loose tab into an existing document
  set, saving the generated `documentSet` front matter, and closing that group
  without disturbing other open groups.
- This update adds generated `[LIST_OF_FIGURES]` and `[LIST_OF_TABLES]`
  support, including front matter aliases, fence-aware caption scanning,
  numbered anchor links, and Rust compiler/export proof for compiled Markdown,
  preview HTML, and DOCX artifact text.
- This update adds command-palette insertion paths for all generated-section
  markers: `[TOC]`, `[INDEX]`, `[BIBLIOGRAPHY]`, `[LIST_OF_FIGURES]`, and
  `[LIST_OF_TABLES]`, with workflow harness coverage in the snippet insertion
  flow.
- This update strengthens the references workflow harness with mock BibTeX
  parsing so the References panel proves resolved bibliography entries,
  missing citation keys, and duplicate bibliography-key reporting.
- This update extends export readiness validation so malformed review comments
  and change notes missing author, timestamp, or text emit actionable
  diagnostics, mark readiness as not ready, and copy those diagnostics into the
  export manifest for auditability.
- This update extends AI provenance readiness validation so incomplete
  `ai-source` blocks, incomplete AI-assisted section markers, missing
  human-review metadata, and invalid AI review statuses produce actionable
  diagnostics that are also copied into the export manifest.
- This update extends export readiness validation so figures, tables, and
  equations missing stable labels or captions produce actionable diagnostics
  that are copied into the export manifest before artifact writes.
- This update adds an explicit export manifest readiness summary with ready,
  error, warning, and info counts so sidecar manifests carry portable
  preflight evidence without requiring consumers to recount diagnostics.
- This update makes direct exports run the same dirty-Git readiness check as
  preflight export reports, records that warning in the response and sidecar
  manifest, and avoids falling back to the repo cwd when an unsaved/nonexistent
  source path is exported.
- This update restores the `docs/specification.md` architecture figure target
  with `docs/architecture.svg` and adds `pnpm run check:docs` as a repeatable
  repo-wide local markdown link guard.
- This update adds `pnpm run check:a11y` as a static accessibility guard for
  the Vue workbench template and fixes exposed labeling gaps in modal close
  buttons, command-palette search, and conflict merge-line controls.
- This update adds visible-on-focus skip links for keyboard users, makes the
  command bar, workspace, sidebar, Markdown source, live preview, and status
  bar programmatically focusable skip targets, extends `pnpm run check:a11y` to
  guard those targets, and adds a Playwright workflow for skip-link focus.
- This update adds modal focus management for the AI paste, command palette,
  and external-conflict dialogs: initial focus, edge Tab trapping, Escape close,
  and focus return to the invoking control. The static accessibility guard now
  requires dialog focusability and keyboard handling, and the browser harness
  lists a modal focus/Escape workflow.
- This update makes status-bar messages, file-watch state, compile progress,
  export progress, and errors screen-reader visible with labeled live regions.
  The static accessibility guard now requires those live regions, and the
  browser harness lists a status/progress live-region workflow.
- This update adds
  `performance_tests::compiler_stress_handles_large_documents_with_many_artifacts`,
  which stress-compiles a large Markdown source with nested includes, many
  tables, formulas, transform artifacts, source-map entries, and diagnostics.
- This update adds
  `performance_tests::repeated_export_loop_keeps_large_artifacts_stable`,
  which repeatedly renders a large compiled document through HTML, PDF, DOCX,
  PPTX, and Markdown bundle outputs with structural and artifact-size
  assertions.
- This update adds
  `performance_tests::repeated_editing_sessions_reuse_external_transform_cache`,
  which repeatedly recompiles edited document text while proving a stable
  trusted external DOT transform is served from cache after the first run.
- This update adds structured export progress steps to readiness/export
  manifests and the export UI. The new
  `export_command_tests::export_readiness_and_manifest_report_progress_steps`
  test proves compile, transform, readiness, render, and manifest progress
  stages are carried in backend evidence.
- This update adds latest-document task guards for preview compilation, a
  status-bar cancel action while compilation is pending, and frontend unit
  coverage proving stale or cancelled compile results are rejected.
- This update extracts preview text commit debounce into a tested helper with
  an explicit `PREVIEW_DEBOUNCE_MS` budget and frontend unit coverage for
  rapid-edit coalescing plus immediate flush behavior.
- This update adds
  `performance_tests::repeated_compile_export_cycles_keep_memory_growth_bounded`,
  which repeatedly compiles and exports large documents while retaining only
  summary data and bounding process RSS growth on macOS/Linux hosts.
- This update adds a Playwright large-document interaction workflow that edits
  a 120-section document, waits for preview update, checks elapsed browser
  time, and verifies editor-to-preview scroll sync. This host still needs the
  Playwright Chromium binary before that browser workflow can execute locally.
- This update extracts workspace persistence migration into
  `src/lib/workspacePersistence.ts` with an explicit schema version, normalized
  settings, legacy key aliases, clamped persisted values, and frontend unit
  coverage for preferences, recents, transform settings, AI cleanup defaults,
  and scroll positions.
- This update adds `docs/storage-model.md` and
  `docs/security-threat-model.md`, links them from the README, and folds them
  into the markdown link checker so local-first storage, sidecar evidence,
  trust boundaries, threat mitigations, and non-goals are auditable alongside
  the specification matrix.
- This update hardens the Git versioning command surface: release tags and
  restore revisions now reject option-shaped or unsupported ref syntax before
  invoking Git, restore refuses symlink targets, and restore paths must resolve
  inside the repository root before the document is overwritten.
- This update adds first-user documentation in `docs/user-guide.md`, a
  readable syntax reference in `docs/markdown-extensions.md`, README links to
  both, and a docs link checker that automatically discovers top-level docs
  Markdown files.
- This update extends export conformance proof for release-grade business
  metadata: the fixture now carries a legal disclaimer, exported DOCX/PPTX
  packages carry approval and legal-disclaimer custom properties, and
  `export_conformance_fixture_maps_business_features` asserts legal
  disclaimer plus approval metadata across HTML, PDF, DOCX, PPTX, plain text,
  and Markdown bundle outputs.
- This update adds
  `prepare_for_export_carries_broad_readiness_audit_to_manifest`, which proves
  one export readiness report and its manifest carry diagnostics for metadata,
  approval, layout, citation source gaps, comments, change notes, AI
  provenance, AI review status, broken links/media, inline and table formulas,
  caption/label checks, transform settings, and pending progress state.
- This update carries the compiler include graph into export manifests and
  Markdown bundles, adding `include_graph` manifest evidence plus an
  `include-graph.json` bundle entry so exported audit packs preserve parent,
  child, and depth relationships instead of only flat included-file hashes.
- This update expands the Rust-native OpenAPI and JSON Schema transforms from
  minimal tables into richer reference renderers: OpenAPI output now includes
  API title/version, servers, operation IDs, tags, path and operation
  parameters, request bodies, responses, media types, and component schemas;
  JSON Schema output now includes root metadata, nested field paths, array item
  paths, required flags, descriptions, refs, enums, formats, defaults, and
  numeric/string constraints.
- This update adds
  `export_option_matrix_is_preserved_across_targets_and_bundle_evidence`,
  which runs one explicit export option matrix through HTML, PDF, DOCX, PPTX,
  plain text, and Markdown bundle evidence. It proves styles, syntax
  highlighting, cover pages, page numbers, glossary/comment/provenance
  appendices, PPTX agenda generation, layout preset, watermark, legal
  disclaimer output, and Markdown bundle `export_options` are honored from the
  same compiled document.
- This update adds
  `rich_markdown_blocks_survive_cross_target_exports`, which pushes a richer
  Markdown business fixture through HTML, PDF, DOCX, PPTX, plain text, and
  Markdown bundle evidence. It covers block quotes, callouts, unordered,
  nested, ordered, and task lists, JavaScript code blocks, tables, figures,
  equations, generated lists of figures/tables, and cross references.
- This update adds
  `external_transform_cache_invalidates_when_trusted_executable_changes`, which
  rewrites a trusted external executable at the same absolute path and proves
  the cache key changes, the output hash changes, and the new engine output is
  executed instead of serving stale memory or disk cache. The transform
  metadata and setup docs now describe engine file size, modified time,
  adapter arguments, input mode, renderer version, and source hash as part of
  cache identity.
- This update adds `pnpm run check:engines` and
  `docs/external-transform-platform-evidence.md`. The current macOS Darwin
  arm64 evidence verifies Graphviz/DOT, D2, and PlantUML through the optional
  engine probe and `external_transform_conformance_runs_installed_engines`
  with `--nocapture`; Pikchr is explicitly recorded as missing on this host.
- This update adds per-key missing-citation readiness diagnostics with precise
  source ranges when citations are present but no bibliography source is
  available. The new readiness proof verifies line, column, end-line, and
  end-column details for each missing key and confirms those diagnostics are
  copied into export manifests.
- This update carries source locations through bibliography entries, points
  duplicate-key readiness diagnostics at the duplicate entry with the first
  occurrence in related context, and shows duplicate bibliography source
  locations in the References panel.

## Current Capability Snapshot

Implemented or substantially present, pending the conservative caveats in
`docs/spec-completion-matrix.md`:

- Tauri 2 desktop app scaffold with Vue 3, Pinia, Vite, CodeMirror 6, vanilla
  CSS, and Rust IPC commands.
- Markdown workbench UI with editor, live preview, sidebars, tabs, status bar,
  command palette, review/versioning/export/settings panels, and conflict UI.
- File operations for local documents, workspace folder browsing, recent files,
  recently closed documents, pinned tabs, workspace restore, snapshots,
  validated Git history/diff/commit/tag/restore, and guarded saves.
- Compiler pipeline for front matter, includes, variables, transforms,
  formulas, citations, bibliography, glossary, index, cross references, review
  comments, AI provenance, generated lists of figures/tables, semantic AST,
  paged document model, diagnostics, source maps, and export manifests.
- Export modules for HTML, PDF, DOCX, PPTX, and Markdown bundle outputs.
- Export package metadata now records status, version, classification, client,
  approval metadata, legal disclaimer, source hash, and app version where the
  target format supports custom properties.
- Export manifests now include included-file hashes, media hashes, source maps,
  diagnostics, readiness summaries, progress steps, layout sections, transform
  metadata, and the include graph used to build the deliverable.
- Export option fidelity now has backend package/text proof across HTML, PDF,
  DOCX, PPTX, plain text, and Markdown bundle manifests for one target-specific
  option matrix.
- Rich Markdown block fidelity now has backend package/text proof across HTML,
  PDF, DOCX, PPTX, plain text, and Markdown bundle AST/text evidence for common
  business-document blocks and generated cross-reference sections.
- Transform registry with Rust-native renderers/fallbacks and trust-gated
  external adapters for Graphviz/DOT, D2, PlantUML, and Pikchr.
- External transform cache identity includes executable file identity
  (path/size/modified time), adapter arguments, input mode, renderer version,
  and source hash, with a regression test proving same-path executable rewrites
  do not serve stale cached output.
- Optional external engine platform evidence can now be refreshed locally with
  `pnpm run check:engines`; the latest macOS record proves dot, d2, and
  PlantUML execution paths and records the missing Pikchr gap.
- Rust-native structured-document transforms for JSON, YAML, OpenAPI, and JSON
  Schema, including nested schema/reference output for API and schema docs.
- Business features including AI paste cleanup, table editor logic, formula
  handling, diagrams, citations, captions, layout directives, export readiness,
  and review/release metadata.
- Backend test coverage across many compiler, export, transform, table,
  validation, media, file, Git, snapshot, review, and provenance paths.
- Executable coverage for the initial Tauri IPC command list in specification
  section 25.4, backed by the human-readable
  `docs/ipc-command-coverage.md` table.
- MIT license metadata in root, npm, Cargo, and Tauri desktop bundle package
  surfaces.
- Realistic example fixtures now cover consultants, technical writers,
  researchers and analysts, product and engineering teams, executives and
  managers, students and academics, developers, and teams using AI chat output,
  with local Rust coverage across HTML, PDF, DOCX, PPTX, and Markdown bundle
  outputs.
- User-facing docs now include the README feature tour, a first-user guide, a
  Markdown extensions syntax reference, transform setup, storage model,
  security threat model, completion matrix, backlog, and progress log.
- Frontend unit coverage for table logic and conflict diff alignment.
- Playwright browser workflow harness for Vite with mocked Tauri IPC,
  covering split/source/preview/focus/export/review/presentation view mode
  switching, clean/dirty/saved window titles, theme/typography persistence,
  command palette table insertion, table editor insertion, mocked file
  lifecycle operations, advanced table
  paste/sort/formula/merge/apply behavior, row/column structure editing,
  column format totals, cancel behavior, save-as plus recently closed
  reopening, stale-save conflict copy/merge/keep-local/accept-external
  recovery, watcher-originated root reload/conflict behavior, AI paste cleanup
  insertion plus quote/appendix/replace-document/section/selection modes,
  citation TODO insertion, draft markers, provenance blocks,
  clean included-file recompile, dirty included-file conflict handling, and
  export readiness, restart-style workspace restore, folder/document-set tab
  grouping, drag-to-document-set front matter updates, close-group behavior,
  restored scroll position handling, missing restored-file warnings, tab activation, dirty-close
  confirmation, renamed recent cleanup, deleted recently-closed pruning, recent
  folder reopen/prune behavior, moved recently-closed path pruning,
  live preview updates from source edits, synchronized editor/preview scrolling,
  preview heading click-to-source, persisted editor word-wrap and line-number
  settings, spellcheck/autocapitalize editor attributes, CodeMirror find/replace, status-bar
  word/character/reading-time metrics, explicit multi-cursor command coverage,
  smart list continuation, bracket/quote/emphasis/inline-code pairing,
  code-fence insertion, command-palette heading navigation, command-palette
  citation, glossary, and index navigation, outline-sidebar heading navigation,
  diagnostics-panel source navigation, command-palette open-document switching
  and workspace-file opening, plus
  transform engine settings trust/probe diagnostics, target-specific export
  readiness manifest preview, export output/manifest path reporting, export
  success/failure diagnostics, and blocked-readiness diagnostics before file
  write.
- Local verification scripts for Rust formatting/check/test, native-watch
  compilation, clippy, frontend unit tests, frontend build, Playwright browser
  workflows, and Tauri no-bundle desktop compilation.

## Active Known Gaps

P0 gaps:

- GitHub Actions is not an active verification surface for NEditor. The
  `.github/workflows/ci.yml` workflow has been removed, and completion gates now
  rely on local verification evidence plus explicit rendered/manual/platform
  proof where required.

- Archived remote workflow evidence for commit `02e832a` was green: browser workflow, Ubuntu
  desktop, macOS desktop, and Windows desktop all passed in run `26159396761`.
  The prior Windows path-sensitive Rust-test failures, Ubuntu installed Pikchr
  conformance failure, and Ubuntu fake-`d2` stdin fixture failure are resolved
  in that retired workflow.
- Browser-level workflow tests previously passed in Linux Actions with 28 Chromium tests in
  run `26159396761`, including mocked file lifecycle coverage, save-as plus
  recently closed reopening, stale-save conflict copy/merge/keep-local/
  accept-external recovery, clean watcher reload, watcher-originated dirty
  root-file conflicts, advanced table structure/format/cancel coverage, and AI
  paste insert/quote/appendix/replace-document/section-merge/replace-selection
  workflows, clean included-file recompilation, and dirty included-file
  conflict handling, plus restart-style workspace restore for open tabs, active
  tab, pinned state, mode/sidebar persistence, workspace root, and recent
  files, scroll-position restore, and missing-restored-file warning coverage.
  Local focused Playwright execution remains blocked by sandboxed Playwright
  browser cache installation: `pnpm exec playwright install chromium` failed
  with `EPERM` while creating
  `/Users/nyimbiodero/Library/Caches/ms-playwright/__dirlock`.
- The same green browser run now also covers tab activation, dirty close
  confirmation, renamed recent cleanup, deleted recently-closed pruning, recent
  folder reopen/prune behavior, moved recently-closed path pruning,
  synchronized editor/preview scrolling, preview heading click-to-source,
  editor word-wrap and line-number persistence, CodeMirror find/replace, smart
  list continuation, bracket auto-pairing, command-palette heading navigation,
  citation navigation, glossary navigation, index navigation, open-document
  switching, workspace-file opening, transform engine settings trust/probe
  diagnostics, target-specific export readiness manifest preview, export
  output/manifest path reporting, export success/failure diagnostics, and
  blocked export diagnostics before file write.
- Desktop WebDriver/Tauri-driver workflow tests are missing.
- Current progress/matrix/docs need to be kept updated as evidence changes.

P1 gaps:

- Export fidelity still requires rendered/manual proof, but target-specific
  option matrix preservation and rich Markdown block fidelity now have backend
  package/text evidence across every export target.
- Export readiness has browser workflow coverage for the target-specific
  status/diagnostic path, but still needs a requirement-by-requirement audit.
- Optional external transform evidence now includes current macOS Graphviz/DOT,
  D2, and PlantUML proof. macOS Pikchr and all Windows optional-engine evidence
  remain incomplete.
- File watcher/conflict flows need UI workflow tests.
- Workspace/tab-group behavior now has browser harness proof for restart
  restore and document-set grouping; native desktop proof and deeper drag/reorder
  edge cases remain.
- Remaining editor and preview ergonomics need browser interaction proof beyond
  covered scroll sync and heading click-to-source.
- AI paste, citations, layout, accessibility, performance, table export
  fixtures, and non-sandboxed table/browser execution need workflow, artifact,
  or benchmark evidence before they can be considered complete.

P2/P3 gaps:

- `src/App.vue` and `src/stores/documents.ts` remain large and should be split
  after workflow behavior is locked.
- Large backend/export/transform modules remain and should be split when
  boundaries are clear.
- User documentation, example project fixtures, and full cross-platform
  packaging evidence remain incomplete.

## Verification Status

Fresh baseline recorded on 2026-05-20:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 6 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 53 modules transformed. |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff. |
| `cargo check --locked` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 126 Rust tests passed; 0 failed. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary built at `src-tauri/target/release/neditor`. |

Additional export option matrix verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after adding the option-matrix conformance test. |
| `cargo test --locked export_option_tests --lib` in `src-tauri` | Pass | 9 export option tests passed, including `export_option_matrix_is_preserved_across_targets_and_bundle_evidence`. |
| `cargo test --locked` in `src-tauri` | Pass | 144 Rust tests passed; 0 failed. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 12 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional rich Markdown block export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after adding the rich-block conformance test. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 5 export conformance tests passed, including `rich_markdown_blocks_survive_cross_target_exports`. |
| `cargo test --locked` in `src-tauri` | Pass | 145 Rust tests passed; 0 failed. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 12 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional external transform cache identity verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after adding same-path executable cache invalidation coverage. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 9 external transform tests passed, including `external_transform_cache_invalidates_when_trusted_executable_changes`. |
| `cargo test --locked` in `src-tauri` | Pass | 146 Rust tests passed; 0 failed. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 12 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional external transform platform evidence:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:engines` | Partial pass | On Darwin arm64, Graphviz/DOT was installed at `/opt/homebrew/bin/dot` with version `14.1.5`, D2 was installed at `/opt/homebrew/bin/d2` with version `0.7.1`, PlantUML was installed at `/opt/homebrew/bin/plantuml` with version `1.2026.3`, and Pikchr was missing. The command exits successfully because optional engines may be absent. |
| `cargo test --locked external_transform_conformance_runs_installed_engines --lib -- --nocapture` in `src-tauri` | Partial pass | The installed-engine conformance test verified `dot`, `d2`, and `plantuml` through NEditor's external transform execution path and skipped `pikchr` because it is not installed on this host. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after adding `check:engines` to the local verification script contract. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 13 Markdown files, including the new platform evidence doc; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional citation readiness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_missing_citation_sources_with_precise_ranges --lib -- --nocapture` in `src-tauri` | Pass | Proved no-bibliography citation readiness emits the broad source warning plus one precise per-key diagnostic for each missing citation key, with line/column/end-column ranges and manifest parity. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 15 export command tests passed, including the new missing-citation readiness range coverage. |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after the validation change. |
| `cargo check --locked` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 147 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 13 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed, with Pikchr missing as an explicit optional-engine gap. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional citation and generated-section verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_duplicate_bibliography_keys --lib -- --nocapture` in `src-tauri` | Pass | Proved duplicate bibliography diagnostics point at the second duplicate entry and include the first occurrence in related context. |
| `cargo test --locked compiler_reports_csl_and_hayagriva_duplicate_key_locations --lib -- --nocapture` in `src-tauri` | Pass | Proved duplicate-key source ranges for CSL JSON array entries and Hayagriva YAML top-level entries, including preservation of duplicate Hayagriva keys before YAML map parsing can collapse them. |
| `cargo test --locked compiler_reports_duplicate_keys_across_external_bibliography_files --lib -- --nocapture` in `src-tauri` | Pass | Proved multiple bibliography front matter paths can load separate BibTeX and CSL JSON files, and duplicate-key diagnostics point at the duplicate key in the second external file with first-file context. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 14 citation tests passed, including BibTeX, CSL JSON, Hayagriva, external-file duplicate-key source ranges, numeric citation rendering, unsupported CSL-style fallback diagnostics, and citation export conformance. |
| `cargo test --locked compiler_generates_glossary_sections_from_marker_and_metadata --lib -- --nocapture` in `src-tauri` | Pass | Proved `[GLOSSARY]` marker replacement, front matter-driven generated glossary insertion, preview hover preservation, and DOCX glossary artifact text. |
| `cargo test --locked cross_references_resolve_heading_appendix_and_decision_anchors --lib -- --nocapture` in `src-tauri` | Pass | Proved unprefixed appendix and decision anchors render as Appendix and Decision labels rather than generic section labels. |
| `cargo test --locked heading_appendix_and_decision_references_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Proved section, appendix, and decision cross-reference labels survive HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked compiler_generates_index_from_front_matter_without_marker --lib -- --nocapture` in `src-tauri` | Pass | Proved `index.enabled: true` front matter generates an index without `[INDEX]`, preserves linked terms, honors exclusion settings, and strips inline index markers. |
| `cargo test --locked front_matter_index_survives_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Proved `index: true` front matter-generated index content survives HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after carrying bibliography source metadata. |
| `cargo check --locked` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 156 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after extending bibliography entry shape and numeric citation-style preference normalization. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed with duplicate source locations and numeric citation style controls in the References panel. |
| `pnpm run check:docs` | Pass | Checked 13 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed, with Pikchr missing as an explicit optional-engine gap. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests, including reference-navigation and command-palette insertion coverage for `[GLOSSARY]` and other generated sections. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional focused verification after workflow helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed, including AI paste insertion modes and conflict merge-line composition. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |

Additional browser workflow harness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright --version` | Pass | Playwright reported `Version 1.60.0`. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright install chromium` | Pass | Chromium, FFmpeg, and Chromium headless shell downloaded into Playwright's workspace-local browser cache. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test --list` | Pass | Listed 4 Chromium tests in `e2e/app-workflows.spec.ts`. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm run test:e2e` | Blocked by sandbox | Chromium launch failed before app assertions with `bootstrap_check_in ... Permission denied (1100)`. |

Additional table editor workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding table-draft cancellation and accessible row/column controls. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 14 Chromium workflow tests, including `edits table structure with formats and cancels draft changes`. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "edits table structure" --project chromium` | Blocked by sandbox | Chromium launched, then failed before app assertions with `bootstrap_check_in ... Permission denied (1100)`. Escalated local browser execution was rejected by the approval reviewer. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |
| `gh run watch 26143491444 --exit-status` | Failure diagnosed | For `5a97d86`, 13 browser tests passed and the new table-structure test failed because the assertion expected raw `74000` after the Value column had been switched to currency format; the preview correctly rendered `$74000`. |
| `pnpm run test:unit` | Pass | Re-run after fixing the table-structure assertion; 8 frontend unit tests passed. |
| `pnpm run build` | Pass | Re-run after fixing the table-structure assertion; `vue-tsc --noEmit` and Vite build passed. |
| `pnpm exec playwright test --list` | Pass | Re-run after fixing the table-structure assertion; still lists 14 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after fixing the table-structure assertion. |
| `gh run view 26143632239 --json status,conclusion,headSha,jobs,url` | Pass | Commit `dbd440d` passed browser workflow tests and Ubuntu/macOS/Windows desktop builds. |

Additional AI paste browser workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding AI paste quote, appendix, replace-document, section-merge, and replace-selection browser coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 17 Chromium workflow tests, including the AI paste mode workflows. |
| `gh run view 26144290812 --job 76896091795 --log` | Failure diagnosed | First pushed AI paste browser run had 15 passing tests; the two failures were transient status-bar assertions after the editor mutations had succeeded. |
| `gh run view 26144430209 --json status,conclusion,headSha,jobs,url` | Pass | Commit `3b17c03` passed 17 browser workflow tests and Ubuntu/macOS/Windows desktop builds. |

Additional included-file watcher workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding include-aware browser mock compilation and included-file watcher workflows. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 19 Chromium workflow tests, including clean included-file recompile and dirty included-file conflict workflows. |
| `git diff --check` | Pass | No whitespace errors after adding included-file watcher workflow coverage. |
| `gh run watch 26145509141 --exit-status` | Pass | Commit `c0cefd1` passed 19 browser workflow tests and Ubuntu/macOS/Windows desktop builds. |

Additional workspace restore workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after persisting mode/sidebar with the workspace restore state. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 20 Chromium workflow tests, including restart-style workspace restore for tabs, active document, pins, mode, sidebar, workspace root, and recent files. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "restores workspace" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the workspace cache. |
| `git diff --check` | Pass | No whitespace errors after adding workspace restore persistence coverage. |
| `gh run view 26147556750 --json status,conclusion,headSha,jobs,url` | Pass | Commit `655d65c` passed the 20-test browser workflow job plus Ubuntu/macOS/Windows desktop builds. |

Additional missing-file and scroll restore workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding per-document scroll persistence and missing-restore state. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 21 Chromium workflow tests, including missing restored-file warning coverage and scroll-position restore in the workspace reload workflow. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "restores workspace\|skips missing" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `gh run view 26148828614 --json status,conclusion,headSha,jobs,url` | Pass | Commit `5a13fe2` passed the 21-test browser workflow job plus Ubuntu, macOS, and Windows desktop builds. |
| `git diff --check` | Pass | No whitespace errors after adding scroll and missing-file restore coverage. |

Additional tab and stale-recent workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding dirty-close confirmation and stale recent cleanup paths. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 22 Chromium workflow tests, including tab activation, dirty close confirmation, renamed recent cleanup, and deleted recently-closed pruning coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "switches tabs" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `pnpm exec playwright install chromium` | Interrupted | The install attempt produced no output for several minutes, so the hung process tree was terminated; browser workflow proof still needs a local host where Playwright can launch Chromium. |
| `git diff --check` | Pass | No whitespace errors after adding tab and stale-recent coverage. |
| `gh run view 26150151349 --json status,conclusion,headSha,jobs` | Superseded failure | Commit `44c49f5` ran 20 passing browser workflows before the new tab/stale-recent test exposed title and dirty-close mock assumptions. |
| `gh run view 26150581320 --job 76916713881 --log` | Superseded failure diagnosed | Commit `3fe0c78` still failed because the Tauri dialog mock handled `plugin:dialog\|confirm`, while installed `@tauri-apps/plugin-dialog` implements `confirm()` through `plugin:dialog\|message`. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after correcting the dialog mock and new-document tab-count assertions. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after correcting the dialog mock and new-document tab-count assertions. |
| `pnpm exec playwright test --list` | Pass | Still listed 22 Chromium workflow tests after the dialog mock correction. |
| `git diff --check` | Pass | No whitespace errors after the dialog mock correction. |
| `gh run view 26151007765 --job 76918155717 --log` | Superseded failure diagnosed | Commit `39a3286` ran 21 passing browser workflows; the remaining failure was a broad `Rename` locator matching the `Rename Source` tab and file row. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after disambiguating the rename command locator. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after disambiguating the rename command locator. |
| `pnpm exec playwright test --list` | Pass | Listed the same 22 Chromium workflow tests after the exact rename locator fix. |
| `git diff --check` | Pass | No whitespace errors after the exact rename locator fix. |
| `gh run watch 26151184228 --exit-status` | Pass | Commit `bf60405` passed 22 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional recent-folder and moved-path workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding stale recent-folder cleanup and moved recently-closed path coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the recent-folder and moved-path workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 23 Chromium workflow tests, including recent folder reopen/prune behavior and moved recently-closed path pruning. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "reopens recent folders" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding recent-folder and moved-path coverage. |
| `gh run watch 26152255407 --exit-status` | Pass | Commit `13b3086` passed 23 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional preview navigation workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding preview scroll-sync and heading navigation browser coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the preview navigation workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 24 Chromium workflow tests, including preview scroll sync and heading click-to-source coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "syncs editor" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding preview navigation coverage. |
| `gh run watch 26153224371 --exit-status` | Pass | Commit `7702e89` passed 24 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional editor ergonomics workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding editor ergonomics browser coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the editor ergonomics workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 25 Chromium workflow tests, including persisted editor settings, find/replace, list continuation, bracket auto-pairing, and command-palette heading navigation coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "persists editor settings" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding editor ergonomics coverage. |
| `gh run watch 26154535588 --exit-status` | Pass | Commit `f13c3f3` passed 25 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional command palette reference navigation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding citation/glossary/index command palette workflow coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the command palette reference navigation update. |
| `pnpm exec playwright test --list` | Pass | Listed 26 Chromium workflow tests, including command palette citation, glossary, and index navigation coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "citation glossary" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding command palette reference navigation coverage. |
| `gh run watch 26155535210 --exit-status` | Pass | Commit `5f75e44` passed 26 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional command palette document/workspace navigation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding command palette open-document and workspace-file workflow coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the command palette document/workspace navigation update. |
| `pnpm exec playwright test --list` | Pass | Listed 27 Chromium workflow tests, including command palette open-document switching and workspace-file opening. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "open document and workspace file" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding command palette document/workspace navigation coverage. |
| `gh run watch 26156393184 --exit-status` | Pass | Commit `145942a` passed 27 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional transform settings workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding transform engine trust/probe browser workflow coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the transform settings workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 28 Chromium workflow tests, including transform engine trust/probe diagnostics coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "transform engine trust" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding and stabilizing transform settings workflow coverage. |
| `gh run watch 26157446711 --exit-status` | Pass | Commit `976016c` passed 28 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional export readiness/result workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after export result diagnostics and immediate-export editor-state flush. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the export readiness/result workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 28 Chromium workflow tests, including export readiness, success, failure, and blocked-readiness workflow coverage. |
| `pnpm exec playwright install chromium` | Blocked locally | Browser install is sandbox-blocked by `EPERM` while creating `/Users/nyimbiodero/Library/Caches/ms-playwright/__dirlock`. |
| `git diff --check` | Pass | No whitespace errors after the export workflow fixes. |
| `gh run watch 26159396761 --exit-status` | Pass | Commit `02e832a` passed 28 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional document grouping workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright test --list` | Pass | Listed 34 Chromium workflow tests, including `groups documents by document set and folder`. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding document-set metadata to the browser mock. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the grouping workflow. |
| `git diff --check` | Pass | No whitespace errors after adding the grouping workflow and docs. |
| Markdown local link resolution script | Pass | Updated markdown docs contain no broken local links. |

Additional caption-list export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after generated-section changes. |
| `cargo test --locked compiler_generates_lists_of_figures_and_tables --lib` in `src-tauri` | Pass | Focused test proves generated list markers, numbering, anchors, fenced-example exclusion, preview HTML, and DOCX artifact text. |

Additional generated-list insertion workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright test --list` | Pass | Browser harness lists the command-palette insertion workflow that now asserts `[TOC]`, `[INDEX]`, `[BIBLIOGRAPHY]`, `[LIST_OF_FIGURES]`, and `[LIST_OF_TABLES]` insertion. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding generated-section insertion commands. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding generated-section insertion commands. |

Additional references-panel bibliography verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright test --list` | Pass | Browser harness lists the citation/glossary/index workflow that now asserts resolved bibliography entries, missing citation keys, and duplicate bibliography keys. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding bibliography-aware browser mock compilation. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding bibliography-aware browser mock compilation. |

Additional review/provenance readiness metadata verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after adding review/change-note, AI provenance, caption/label readiness diagnostics, and direct-export dirty-Git manifest warnings. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the direct-export readiness update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native file watcher feature still compiles after the direct-export readiness update. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 13 export command tests passed, including readiness and manifest diagnostics for malformed review comments, change notes, incomplete AI provenance metadata, invalid AI review statuses, missing figure/table/equation labels or captions, explicit manifest readiness summaries, direct-export dirty-Git manifest warnings, and structured export progress-step reporting. |
| `cargo test --locked review_provenance_tests --lib` in `src-tauri` | Pass | 4 review/provenance tests passed after the stricter AI provenance validation. |
| `cargo test --locked validation_tests --lib` in `src-tauri` | Pass | 4 validation tests passed after the readiness validation extension. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the shared parser and export-readiness validation changes. |
| `cargo test --locked compiler_stress_handles_large_documents_with_many_artifacts --lib` in `src-tauri` | Pass | Large-document compiler stress test passed with nested includes, 80 tables, 80 CSV transform artifacts, 120 formula definitions, many source-map entries, and many broken link/media diagnostics. |
| `cargo test --locked repeated_export_loop_keeps_large_artifacts_stable --lib` in `src-tauri` | Pass | Repeated export-loop stress test passed across HTML, PDF, DOCX, PPTX, and Markdown bundle rendering for a large compiled document. |
| `cargo test --locked repeated_editing_sessions_reuse_external_transform_cache --lib` in `src-tauri` | Pass | Repeated editing/cache stress test passed with a trusted external DOT transform executed once and served from cache during subsequent edits. |
| `cargo test --locked export_readiness_and_manifest_report_progress_steps --lib` in `src-tauri` | Pass | Export progress-step test passed, proving readiness/export manifests expose compile, transform, readiness, render, and manifest stages. |
| `cargo test --locked prepare_for_export_carries_broad_readiness_audit_to_manifest --lib` in `src-tauri` | Pass | Broad readiness audit test passed, proving report/manifest parity for metadata, approval, layout, citations, comments, AI provenance, links/media, formulas, captions, transform settings, and progress state. |
| `cargo test --locked compiler_loads_front_matter_csv_data_sources --lib` and `cargo test --locked markdown_bundle_keeps_duplicate_include_basenames_distinct --lib` in `src-tauri` | Pass | Include-graph manifest tests passed, proving data-source/include edges appear in export manifests and Markdown bundles include both `manifest.json` include graph data and `include-graph.json`. |
| `cargo test --locked compiler_renders_openapi_and_json_schema_tables --lib` in `src-tauri` | Pass | Structured transform test passed, proving richer OpenAPI operation/server/parameter/request/response/component rendering and nested JSON Schema field/constraint rendering. |
| `cargo test --locked export_conformance_fixture_maps_business_features --lib` in `src-tauri` | Pass | Export conformance fixture now proves legal disclaimer and approval metadata across HTML, PDF, DOCX package properties/body, PPTX package properties/slides, plain text, and Markdown bundle metadata. |
| `cargo test --locked repeated_compile_export_cycles_keep_memory_growth_bounded --lib` in `src-tauri` | Pass | Repeated compile/export memory-growth stress passed with bounded retained summaries and process RSS growth sampling. |
| `cargo test --locked git_restore_and_tag_reject_option_shaped_refs --lib` in `src-tauri` | Pass | Git tag/revision option-injection regression passed. |
| `cargo test --locked git_restore_refuses_symlink_targets --lib` in `src-tauri` | Pass | Git restore refused a symlinked worktree file and left the outside target unchanged. |
| `cargo test --locked git_history_diff_commit_tag_and_restore_workflow --lib` in `src-tauri` | Pass | Existing Git history, diff, commit, tag, and restore workflow still passed after the new guards. |
| `cargo test --locked` in `src-tauri` | Pass | 143 Rust tests passed plus main/doc test targets with 0 tests on this Unix host. |
| `npx playwright test e2e/app-workflows.spec.ts -g "keeps large document editing"` | Blocked | The large-document browser workflow is present, but this host is missing Playwright Chromium at `~/Library/Caches/ms-playwright/.../chrome-headless-shell`. |
| `pnpm exec playwright test --list` | Pass | Browser harness lists 35 Chromium workflow tests, including the large-document interaction workflow. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed, including latest-document task cancellation/stale-result guard coverage, preview debounce timing/coalescing coverage, and workspace persistence migration/schema normalization. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guard passed for button names, form-control labels, and dialog labeling. |
| `pnpm run check:docs` | Pass | Markdown link guard now discovers README plus all top-level docs Markdown files, including the user guide and Markdown extensions reference. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary built at `src-tauri/target/release/neditor`. |
| `git diff --check` | Pass | No whitespace errors after the readiness metadata, dirty-Git manifest, and documentation updates. |
| `pnpm run check:docs` | Pass | Repo-wide markdown link guard checked README plus all docs and found no missing local links after adding `docs/architecture.svg`. |

Archived remote workflow evidence log:

| Command | Result | Evidence |
| --- | --- | --- |
| `gh run view 26131218116 --json jobs,conclusion,status,headSha,url` | Failure diagnosed | All jobs for `d94dc6c` failed at `Set up Node.js` before project commands ran. |
| `XDG_CACHE_HOME=.cache gh run view 26131218116 --log-failed` | Failure diagnosed | `actions/setup-node` could not locate `pnpm` while `cache: pnpm` was enabled before pnpm setup. |
| `gh run view 26131619095 --json jobs,conclusion,status,headSha,url` | Partial improvement | For `6489162`, pnpm and Node setup succeeded in every job; the browser job reached Playwright execution. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76857769468/logs` | Browser failure diagnosed | Playwright ran 4 tests in CI; 3 passed and the readiness test failed because `getByText("Ready")` matched both `Ready` and `Document is ready for export`. |
| `gh run view 26131805873 --json jobs,conclusion,status,headSha,url` | Browser workflow pass | For `420af08`, the `Browser workflow tests` job passed through Playwright Chromium installation and `pnpm run test:e2e`. |
| `gh run view 26131929125 --json jobs,conclusion,status,headSha,url` | Mixed CI result | For `9a6d52e`, browser workflows and macOS desktop passed; Windows desktop failed clippy, Ubuntu desktop failed Rust tests. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76858711971/logs` | Windows failure diagnosed | Clippy failed because `clear_external_transform_memory_cache_for_tests` in `src/transforms/external.rs:444` was dead code in the Windows lib-test target. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76858711986/logs` | Ubuntu failure diagnosed | `external_transform_conformance_runs_installed_engines` failed because the installed `pikchr-cli` requires a positional `<PIKCHR>` argument and exited with status 2. |
| `cargo fmt --check` in `src-tauri` | Pass | Re-run after the external transform adapter fix. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib` in `src-tauri` | Pass | Covered the first attempted `pikchr-cli` positional raw-source adapter path. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 8 external transform tests passed, including installed-engine conformance path and trust/timeout/error behavior. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Re-run after Unix-gating the test cache helper and adding the Pikchr adapter path. |
| `cargo test --locked` in `src-tauri` | Pass | 126 Rust tests passed plus main/doc test targets with 0 tests. |
| `gh run view 26132634911 --json status,conclusion,headSha,jobs` | Mixed CI result | For `25f7b04`, browser workflow and macOS desktop passed; Windows desktop failed Rust tests; Ubuntu desktop failed Rust tests. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76860838872/logs` | Windows failure diagnosed | Four Rust tests failed because serialized paths were not stable across Windows separators: file watch aliases, workspace listing, local figure media, and duplicate include bundle tests. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76860838881/logs` | Ubuntu follow-up diagnosed | The no-argument Pikchr failure moved forward, but `pikchr-cli` treated the raw source argument as a missing file path and exited with status 1. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib` in `src-tauri` | Pass | Covers `pikchr-cli` receiving a temporary `.pikchr` source path. |
| `cargo test --locked file_command_tests --lib` in `src-tauri` | Pass | Covers slash-normalized watched paths and workspace listing on the local platform. |
| `cargo test --locked media_export_tests --lib` in `src-tauri` | Pass | Covers slash-normalized manifest/include/media paths on the local platform. |
| `cargo fmt --check` in `src-tauri` | Pass | Re-run after the latest Pikchr/path fixes; formatting is clean. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 8 external transform tests passed with temporary source path delivery for `pikchr-cli`. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Re-run after the latest Pikchr/path fixes; no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 126 Rust tests passed plus main/doc test targets with 0 tests. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |
| `gh run view 26133136580 --json status,conclusion,headSha,jobs` | Mixed CI result | For `5c29914`, browser workflow and macOS desktop passed; Windows passed Rust tests, frontend unit tests, and frontend build before continuing the Tauri shell build; Ubuntu failed one Rust fixture test. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76862375005/logs` | Ubuntu fixture failure diagnosed | `external_transform_conformance_runs_installed_engines` passed, but `external_transform_adapters_shape_engine_specific_invocations` failed because the fake `d2` script did not consume stdin before exiting. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib` in `src-tauri` | Pass | Re-run after making the fake `d2` adapter consume stdin. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 8 external transform tests passed after the fake `d2` stdin fixture fix. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Re-run after the fake `d2` stdin fixture fix; no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 126 Rust tests passed plus main/doc test targets with 0 tests after the fake `d2` stdin fixture fix. |
| `git diff --check` | Pass | No whitespace errors after the latest documentation and fake-`d2` fixture edits. |
| `gh run watch 26133595556 --exit-status` | Pass | For `33ee6a9`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Ubuntu passed optional engine install, Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle build. |
| `git diff --check` | Pass | No whitespace errors after refreshing docs with the green CI result. |
| `pnpm exec playwright test --list` | Pass | Listed 5 Chromium browser workflow tests, including the new advanced table paste/sort/formula/merge/apply flow. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding the advanced table browser workflow. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding the advanced table browser workflow. |
| `gh run watch 26134248308 --exit-status` | Pass | For `443515b`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser workflow tests ran through Chromium installation and `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `pnpm exec playwright test --list` | Pass | Listed 6 Chromium browser workflow tests, including the new mocked file open/save/duplicate/rename/pin/reveal/revert flow. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding the mocked file lifecycle browser workflow. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding the mocked file lifecycle browser workflow. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "opens, saves, duplicates" --project chromium` | Blocked locally | Playwright could not launch because `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/chrome-headless-shell-mac-arm64/chrome-headless-shell` is missing. |
| `git diff --check` | Pass | No whitespace errors after stabilizing the mocked file lifecycle workflow assertions. |
| `gh run view 26135510740 --json status,conclusion,jobs` | Superseded failure | For `11dafc3`, the mocked lifecycle browser test still asserted the active tab text as a path even when front matter made the tab title `*Market Entry Report`. |
| `gh run view 26135873103 --json status,conclusion,jobs` | Superseded failure | For `b2ccf83`, the same browser assertion shape remained brittle before the file-row assertion fix. |
| `gh run view 26136003362 --json status,conclusion,jobs` | Superseded failure | For `a55970d`, the workflow tried to click workspace file rows while the sidebar was still in the review panel. |
| `gh run view 26136129630 --json status,conclusion,jobs` | Superseded failure | For `b7534f6`, `getByLabel("Pin document")` matched multiple tab buttons. |
| `pnpm exec playwright test --list` | Pass | Listed 6 Chromium browser workflow tests after the final active-tab pin selector fix. |
| `git diff --check` | Pass | No whitespace errors after the final browser selector fix. |
| `gh run watch 26136223804 --exit-status` | Pass | For `12cd667`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser CI installed Chromium and ran `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `pnpm exec playwright test --list` | Pass | Listed 7 Chromium browser workflow tests after adding save-as and recently closed reopening. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "saves a document as a new file" --project chromium` | Blocked locally | Default Playwright cache still lacks the Chromium headless-shell executable. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "saves a document as a new file" --project chromium` | Blocked locally | Workspace-local Chromium launched, then failed before assertions with `bootstrap_check_in ... Permission denied (1100)`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding accessible recent-path sections. |
| `git diff --check` | Pass | No whitespace errors after the save-as/recently closed workflow edits. |
| `gh run view 26137235763 --json status,conclusion,jobs` | Superseded failure | For `61eff87`, the new path appeared in both Recent files and Recently closed, so the browser test needed a scoped recently-closed selector. |
| `gh run view 26137384613 --json status,conclusion,jobs` | Superseded failure | For `44c9639`, reopened tab text reflected front matter title rather than the filesystem path; the test now proves path identity via the active workspace row. |
| `gh run watch 26137556147 --exit-status` | Pass | For `138bf5d`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser CI installed Chromium and ran 7 tests through `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `pnpm exec playwright test --list` | Pass | Listed 9 Chromium browser workflow tests after adding stale-save conflict copy and merge recovery workflows. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "stale saves\|external conflict" --project chromium` | Blocked locally | Workspace-local Chromium launched, then failed before assertions with `bootstrap_check_in ... Permission denied (1100)`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding stale-save conflict browser workflows. |
| `git diff --check` | Pass | No whitespace errors after the stale-save conflict browser workflow edits. |
| `gh run watch 26138478934 --exit-status` | Superseded failure | For `3cb1b84`, 8 browser tests passed but the merge test asserted a transient status message that compile refresh could overwrite. |
| `pnpm exec playwright test --list` | Pass | Listed 9 Chromium browser workflow tests after stabilizing the conflict merge assertion. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after removing the transient status assertion. |
| `git diff --check` | Pass | No whitespace errors after stabilizing the conflict merge workflow proof. |
| `gh run watch 26138672512 --exit-status` | Pass | For `25c7d1e`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser CI installed Chromium and ran 9 tests through `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `pnpm exec playwright test --list` | Pass | Listed 11 Chromium browser workflow tests after adding keep-local and accept-external stale-save conflict workflows. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "keeps local\|accepts external" --project chromium` | Blocked locally | Workspace-local Chromium launched, then failed before assertions with `bootstrap_check_in ... Permission denied (1100)`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding keep-local and accept-external conflict workflows. |
| `git diff --check` | Pass | No whitespace errors after the keep-local and accept-external conflict workflow edits. |
| `gh run watch 26139678118 --exit-status` | Pass | For `4eb1d2c`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser CI installed Chromium and ran 11 tests through `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `gh run list --branch main --limit 5 --json databaseId,headSha,status,conclusion,displayTitle,createdAt,url` | Pass | Latest pushed run for `1ac72c1` completed successfully as run `26140882880`. |
| `gh run view 26140882880 --json jobs,conclusion,status,headSha,url` | Pass | Browser workflow tests and Ubuntu/macOS/Windows desktop builds all passed for `1ac72c1`. |

Archived remote-workflow fixes from before GitHub Actions was retired:

- The removed workflow had installed pnpm before Node setup after earlier
  cache failures.
- `tests/frontend-unit.test.ts` now verifies local package scripts instead of
  enforcing any GitHub Actions workflow.
- `e2e/app-workflows.spec.ts` now scopes the readiness `Ready` assertion to
  `article.readiness` with an exact text match so the browser workflow remains
  strict without matching the status-bar message.

Baseline command set:

```sh
pnpm run test:unit
pnpm run build
cd src-tauri && cargo fmt --check
cd src-tauri && cargo check --locked
cd src-tauri && cargo check --locked --features native-watch
cd src-tauri && cargo clippy --locked --all-targets -- -D warnings
cd src-tauri && cargo test --locked
./node_modules/.bin/tauri build --no-bundle
```

Baseline gaps:

- The baseline includes an initial browser workflow harness, but it does not yet
  include a passing browser execution result from this sandbox.
- The baseline does not include desktop WebDriver/Tauri-driver workflow tests.
- The baseline does not include rendered visual QA for PDF/DOCX/PPTX outputs.
- The baseline does not include full macOS/Windows/Linux package bundle
  creation.
- The baseline does not include macOS/Windows optional external transform
  engines.

Optional packaging checks:

```sh
./node_modules/.bin/tauri build --bundles app
./node_modules/.bin/tauri build --bundles dmg --verbose
```

Known packaging note from `README.md`:

- macOS app bundle builds have previously succeeded.
- DMG bundling previously reached app bundle creation and then failed in
  `hdiutil create` with `Device not configured`; this needs refreshed
  classification as host-specific or config-specific.

Additional equation caption verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_block_and_inline_equations --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves the documented equation `caption` attribute renders as `Equation N: Caption`, is retained as `data-caption`, is captured in the document AST, and satisfies caption-label readiness for the equation. |
| `cargo test --locked captioned_equations_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves captioned equation text, stable equation references, and the human caption survive HTML, PDF, DOCX, PPTX, Markdown bundle text, and Markdown bundle AST outputs. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after the equation caption parser, AST, and tests were added. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the equation caption update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the equation caption update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after preserving equation captions through rich-block rendering and AST parsing. |
| `cargo test --locked` in `src-tauri` | Pass | 157 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the documentation and backend update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional TOC export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked generated_toc_exports_page_numbers_for_pdf_and_docx --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves front matter TOC depth/numbering, page-numbered PDF TOC entries such as `1 Alpha .... 2`, DOCX TOC field output, and suppression of raw Markdown TOC links in DOCX. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after the PDF TOC page-numbering update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the PDF TOC page-numbering update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the PDF TOC page-numbering update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after adding PDF TOC post-pagination page-number rendering. |
| `cargo test --locked` in `src-tauri` | Pass | 158 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional include guard verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_circular_and_too_deep_includes --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves circular include diagnostics, related include-target context, include graph depth entries for the resolved chain, maximum include depth enforcement, and suppression of content beyond the depth limit. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after adding include guard regression coverage. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the include guard coverage update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the include guard coverage update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after adding circular/max-depth include guard coverage. |
| `cargo test --locked` in `src-tauri` | Pass | 159 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional invalid-include and table-formula export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_unreadable_include_targets_with_context --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves unreadable/invalid include target diagnostics include source file, source line, suggestion, original include target, and resolved path context. |
| `cargo test --locked named_table_formulas_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves named table/range formulas resolve and survive HTML, PDF, DOCX, PPTX, Markdown bundle text, and Markdown bundle AST outputs. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after the invalid-include diagnostic and named table formula export updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the invalid-include diagnostic and named table formula export updates. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the invalid-include diagnostic and named table formula export updates. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the invalid-include diagnostic and named table formula export updates. |
| `cargo test --locked` in `src-tauri` | Pass | 161 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional front matter data source verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_loads_front_matter_json_and_yaml_data_sources --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves JSON and YAML files referenced from front matter render through structured transform artifacts, preview HTML, include graph, and manifest included-file evidence. |
| `cargo test --locked compiler_reports_malformed_front_matter_data_sources --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves missing data source paths, unsupported data source types, and unreadable local files produce actionable diagnostics. |
| `cargo test --locked front_matter_data_sources_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves front matter CSV/TSV/JSON/YAML data source content survives HTML, PDF, DOCX, PPTX, Markdown bundle text, and Markdown bundle manifest outputs. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after extending front matter data source support. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the front matter data source update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the front matter data source update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the front matter data source update. |
| `cargo test --locked` in `src-tauri` | Pass | 164 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional document variable filter verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_formats_document_variables_and_reports_bad_filters --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves document variable defaults, chained text filters, numeric filters, unsupported-filter diagnostics, nonnumeric-filter diagnostics, and source-ranged warnings. |
| `cargo test --locked formatted_document_variables_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves formatted document variables survive HTML, PDF, DOCX, PPTX, and Markdown bundle text outputs. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after extending document variable filters. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the document variable filter update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the document variable filter update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the document variable filter update. |
| `cargo test --locked` in `src-tauri` | Pass | 166 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional malformed front matter verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_malformed_front_matter_with_source_ranges --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves invalid YAML front matter and non-mapping YAML front matter produce source-ranged diagnostics with actionable suggestions. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after tightening front matter diagnostics. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the front matter diagnostic update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the front matter diagnostic update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the front matter diagnostic update. |
| `cargo test --locked` in `src-tauri` | Pass | 167 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional snapshot restore hardening verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked snapshot_restore --lib -- --nocapture` in `src-tauri` | Pass | Focused snapshot tests prove restore succeeds for matching active-document metadata and rejects out-of-store snapshots, non-Markdown restore targets, and snapshots whose source metadata belongs to another document. |
| `cargo test --locked spec_25_4_ipc_commands_are_registered_and_documented --lib -- --nocapture` in `src-tauri` | Pass | IPC command coverage stayed synchronized after changing `restore_snapshot` to a typed restore request. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after the snapshot restore hardening. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the snapshot restore hardening. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the snapshot restore hardening. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the snapshot restore hardening. |
| `cargo test --locked` in `src-tauri` | Pass | 169 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the store restore request update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the store restore request update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary built at `src-tauri/target/release/neditor`, proving the updated Tauri command shape compiles in the desktop app. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional versioning workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the release badge, Versioning-panel snapshot controls, and browser workflow mock coverage. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the versioning workflow UI and mock harness update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve after documenting the versioning workflow update. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed after adding the release status badge and Versioning-panel snapshot controls. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 36 Chromium workflow tests, including `runs snapshot restore and release tagging workflows`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "snapshot restore" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional include graph navigation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding References-sidebar include graph navigation and command-palette include entries. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the include navigation changes. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding the include graph region and navigation buttons. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 37 Chromium workflow tests, including `navigates include graph entries from references and commands`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "include graph" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. Attempting `pnpm exec playwright install chromium` with escalation was rejected before the command ran. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting remained clean; this slice did not edit Rust sources. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional preview diagnostic rendering verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding source-map-backed preview diagnostic callouts and source-jump delegation. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the preview diagnostic rendering change. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding generated preview diagnostic callouts. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 37 Chromium workflow tests; `navigates compiler diagnostics to the source range` now contains preview diagnostic assertions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "diagnostics to the source range" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional export and transform preview verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the export preview summary and transform artifact preview inventory. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the export/transform preview change. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding export/transform preview regions. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 37 Chromium workflow tests; `runs export readiness, success, and failure workflows` now contains export preview and transform artifact assertions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "export readiness" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional AI review governance verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after extending the browser harness for AI review-state toggles and readiness warnings. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the AI governance workflow proof update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the workflow proof update. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 38 Chromium workflow tests, including `toggles AI review state and clears provenance readiness warnings`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "AI review state" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional index and glossary manager verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding References-sidebar Index and Glossary manager controls. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the index/glossary manager update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding manager regions and controls. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 38 Chromium workflow tests; the citation/glossary/index workflow now contains manager assertions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "citation glossary" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional external conflict composition verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the conflict merge composition tray and controls. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including merge composition ordering, duplicate prevention, blank-line preservation, remove, and reorder helpers. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding labeled merge composition controls. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests; the external conflict merge workflow now contains merge composition add/reorder/remove assertions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "merges external conflict" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional target-specific PPTX readiness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_target_specific_pptx_blockers --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust readiness proof blocks PPTX export for in-review documents missing `approvedBy` and `approvedAt`, records `target:pptx` related context, copies readiness into the manifest, and keeps the same source ready for PDF. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 16 export command tests passed after adding the PPTX target-specific blocker. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the export readiness update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after aligning the browser export workflow mock with the PPTX approved-metadata blocker. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests still pass after the export workflow update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the readiness matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the export workflow update. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests; the export readiness workflow now exercises the PPTX approved-metadata blocker instead of a synthetic token. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "export readiness" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional direct-export target extension verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked export_document_blocks_target_extension_mismatches_before_writing --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust export proof blocks a `target: "pptx"` export to `board-deck.pdf`, returns a target-extension validation error, and writes neither artifact nor sidecar manifest. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 17 export command tests passed after adding pre-write target/output extension validation. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the extension validation update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding output extension validation. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional target-specific option audit verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_target_specific_option_info --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust readiness proof reports non-blocking info diagnostics for `includeAgenda` on PDF and render-only options on Markdown bundle exports, keeps readiness true, and copies info counts into manifest readiness. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 18 export command tests passed after adding target-specific option info diagnostics. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after adding target-specific option audits. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding target-specific option audits. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional generated reference-section readiness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_empty_generated_reference_sections --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust readiness proof reports warnings when `index: true` and `glossarySection: true` request generated sections without any index terms or glossary entries, and copies both diagnostics into manifest readiness. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 19 export command tests passed after adding generated index/glossary readiness diagnostics. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 173 Rust library tests passed, covering compiler, export, transform, table, citation, validation, performance, and file command paths after the shared validation update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after adding generated reference-section validation. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after wiring generated section request detection into compiler validation. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional disabled external-engine verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_skips_disabled_external_transform_without_trust_warning --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust compiler proof skips a configured-but-disabled DOT external engine before trust/path execution, falls back to the embedded renderer, and avoids external trust-failure diagnostics. |
| `cargo test --locked prepare_for_export_validates_transform_engine_options --lib -- --nocapture` in `src-tauri` | Pass | Focused readiness proof now validates `disabledTransformEngines` as a boolean map alongside engine paths, trust, input mode, and timeout settings. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 10 external transform tests passed, including trust gating, adapter invocation, disabled-engine fallback, non-executable paths, timeout, stderr, cache invalidation, and installed-engine conformance. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 19 export command tests passed after adding disabled-engine option validation. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 174 Rust library tests passed after adding the disabled external-engine setting and compiler fallback behavior. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the disabled-engine option update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding disabled external-engine option plumbing. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the Settings UI toggle and store state. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after adding disabled external-engine persistence migration coverage. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding the disabled-engine checkbox. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the transform setup doc, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests, including the external transform settings workflow. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "external transform engine" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused external transform settings workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional Vega-Lite native subset verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked vega_lite_area_mark_renders_static_svg_preview --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust transform proof renders native static Vega-Lite `area` marks as SVG polygons with no diagnostics. |
| `cargo test --locked vega_lite_unsupported_marks_report_supported_static_subset --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust transform proof reports unsupported Vega-Lite marks with a suggestion naming the supported native subset: bar, line, point, or area. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 30 transform tests passed after expanding the Vega-Lite native static renderer. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the visual-data renderer update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding native Vega-Lite area rendering. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs and backend transform update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting native visual-data transform subsets; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional Graphviz variant external-engine verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked transform_registry_covers_required_first_release_transforms --lib -- --nocapture` in `src-tauri` | Pass | Focused registry proof includes `dot`, `graphviz`, `circo`, `neato`, `fdp`, `osage`, and `twopi` as supported first-release transforms. |
| `cargo test --locked external_diagram_fallbacks_render_simple_native_svgs --lib -- --nocapture` in `src-tauri` | Pass | Native fallback proof renders a Graphviz variant fence as static SVG when no trusted executable is configured. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib -- --nocapture` in `src-tauri` | Pass | Adapter metadata proof exposes Graphviz variant default commands, version probes, setup hints, and no-shell `-Tsvg` adapter profiles. |
| `cargo test --locked compiler_uses_trusted_graphviz_variant_transform_preferences --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof routes a trusted `neato` transform through external execution with the configured executable path and transform artifact metadata. |
| `cargo test --locked external_transform_tests --lib -- --nocapture` in `src-tauri` | Pass | 11 external transform tests passed; installed-engine conformance verified `dot`, `circo`, `neato`, `fdp`, `osage`, `twopi`, D2, and PlantUML, with Pikchr skipped because it is not installed. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 31 transform tests passed after adding Graphviz variants to the registry and native fallback surface. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 177 Rust library tests passed after widening Graphviz transform support. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the Graphviz variant update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after widening external transform support. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding Graphviz variant probe settings. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the transform settings and docs update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the Settings transform list expansion. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating external transform docs, matrix, TODO, platform evidence, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 reports Graphviz `dot`, `circo`, `neato`, `fdp`, `osage`, `twopi`, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests, including the external transform settings workflow. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional PlantUML PNG output verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib -- --nocapture` in `src-tauri` | Pass | Adapter proof covers PlantUML SVG sidecars and PNG sidecars, including `-tpng`, sidecar `png` output, `output_format: png`, and PNG data URL embedding. |
| `cargo test --locked compiler_uses_plantuml_png_fence_output_format --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof routes a trusted ````plantuml format=png```` fence through file-mode external execution and emits a PNG transform artifact. |
| `cargo test --locked external_transform_tests --lib -- --nocapture` in `src-tauri` | Pass | 12 external transform tests passed after adding PlantUML PNG output selection; installed-engine conformance still verified Graphviz variants, D2, and PlantUML SVG with Pikchr skipped because it is not installed. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 32 transform tests passed after adding PlantUML PNG fence support. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 178 Rust library tests passed after adding PlantUML PNG output selection. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the PlantUML PNG update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding external transform output-format plumbing. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the transform and docs update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the docs and transform update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the PlantUML PNG output update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating external transform docs, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional visual-data map geometry verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked geojson_transform_preserves_geometry_types_in_static_svg_preview --lib -- --nocapture` in `src-tauri` | Pass | Focused GeoJSON proof renders FeatureCollection Polygon and MultiPoint inputs as separate static SVG polygon and point shapes. |
| `cargo test --locked topojson_transform_resolves_object_arc_references --lib -- --nocapture` in `src-tauri` | Pass | Focused TopoJSON proof resolves object-level Polygon arc references, including a reversed arc reference, into a static SVG polygon. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests passed after replacing flattened GeoJSON/TopoJSON coordinate previews with typed map geometry rendering. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 180 Rust library tests passed after the visual-data map geometry update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the visual-data renderer update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding typed GeoJSON and TopoJSON rendering helpers. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed after replacing flattened map previews with typed geometry rendering. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the visual-data and docs update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the docs and renderer update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the visual-data renderer update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating visual-data docs, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Safe business transform completion verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked business_workflow_transforms_render_static_html --lib -- --nocapture` in `src-tauri` | Pass | Focused transform proof covers roadmap metadata, ADR key classes, diff summaries, and static HTML classes for safe business workflow transforms. |
| `cargo test --locked safe_business_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof covers roadmap, ADR, diff, and QR transform artifacts, source ranges, output hashes, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests passed after improving first-release native business transforms. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 13 export conformance tests passed after adding safe business transform cross-target artifact proof. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 181 Rust library tests passed after the safe business transform and export evidence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the renderer and test updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the business transform renderer update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs and renderer update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the safe business transform update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting safe first-release and deferred second-wave transforms; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Visual/data transform export-fidelity verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked visual_data_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof covers chart, Vega-Lite, GeoJSON, TopoJSON, STL, and timeline static SVG artifacts, output hashes, source ranges, direct export-manifest entries, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 14 export conformance tests passed after adding the visual/data transform export fixture. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests still pass after adding export-fidelity coverage for visual/data transforms. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 182 Rust library tests passed after the visual/data transform export evidence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the export conformance fixture update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the visual/data transform export fixture update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the matrix/log/test update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the visual/data transform export evidence update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

BibTeX transform metadata/export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked bibtex_transform_renders_bibliography_preview --lib -- --nocapture` in `src-tauri` | Pass | Focused transform proof covers inline BibTeX title, author, year, and date metadata parsing plus richer transform preview rendering. |
| `cargo test --locked bibtex_transform_survives_cross_target_exports_with_metadata --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof covers BibTeX transform artifacts, source ranges, output hashes, bibliography metadata, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests passed after hardening inline BibTeX field parsing and preview output. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 14 citation tests passed after the BibTeX parser update. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 15 export conformance tests passed after adding the BibTeX transform export fixture. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 183 Rust library tests passed after the BibTeX transform metadata/export evidence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the parser, renderer, and test updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the BibTeX parser and renderer update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs/parser/test update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the BibTeX transform evidence update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating bibliography docs, matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

OpenAPI and JSON Schema export-fidelity verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked api_schema_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof covers OpenAPI and JSON Schema transform artifacts, output hashes, source ranges, semantic table caption preservation, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests passed after adding API/schema transform proof. |
| `cargo test --locked document_structure_tests --lib` in `src-tauri` | Pass | 18 document-structure tests passed after preserving transform table captions through the semantic AST/export path. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests still pass after adding API/schema export proof. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 184 Rust library tests passed after the API/schema transform export evidence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the AST and export fixture updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the AST and export fixture updates. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the matrix/log/test update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the API/schema transform evidence update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

OpenAPI and JSON Schema semantic-depth verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_openapi_and_json_schema_tables --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof now covers OpenAPI global security requirements, deprecated operations, external docs, response media examples, response headers, links, security schemes, plus JSON Schema pattern properties, tuple `prefixItems`, dependent required/dependent schemas, object `additionalProperties`, `oneOf`, refs, and numeric constraints. |
| `cargo test --locked api_schema_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof carries the richer OpenAPI security/header/link/example rows and JSON Schema composition/object-keyword rows through HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked document_structure_tests --lib` in `src-tauri` | Pass | 18 document-structure tests passed after widening API/schema semantic rendering. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests passed after widening API/schema export artifact assertions. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests still pass after the API/schema renderer expansion. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 184 Rust library tests passed after the API/schema semantic-depth update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the renderer and fixture updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the renderer and fixture updates. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs and renderer update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the API/schema semantic-depth update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating transform docs, matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

OpenAPI callbacks/webhooks and JSON Schema dialect verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_openapi_and_json_schema_tables --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof now covers OpenAPI callbacks, webhooks, discriminator summaries and mappings, plus JSON Schema conditional branches and `$defs` rows. |
| `cargo test --locked api_schema_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof carries callback, webhook, discriminator, conditional, and `$defs` evidence through HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked document_structure_tests --lib` in `src-tauri` | Pass | 18 document-structure tests passed after adding OpenAPI callback/webhook/discriminator and JSON Schema dialect coverage. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests passed after widening API/schema artifact assertions. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests still pass after the renderer update. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 184 Rust library tests passed after the callback/webhook/schema-dialect update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the renderer and fixture updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the renderer and fixture updates. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs and renderer update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the API/schema dialect update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating transform docs, matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Direct export sidecar manifest verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked export_document_writes_optional_sidecar_manifest --lib -- --nocapture` in `src-tauri` | Pass | Focused direct-export proof covers HTML, PDF, DOCX, PPTX, and Markdown bundle sidecar manifests, structured sidecar/response manifest equality, exact output paths, actual output SHA-256 hashes, readiness, and progress-step parity. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 19 export-command tests passed after hardening direct sidecar manifest parity assertions. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 184 Rust library tests passed after the direct export sidecar manifest proof update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the export command test update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the export command test update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs/test update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the direct sidecar manifest evidence update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Keyboard skip-link accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify accessible controls, dialog labeling, and required skip-link targets for commands, workspace, sidebar, source, preview, and status. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding skip-link focus handling and target landmarks. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the accessibility update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery now lists 39 Chromium workflow tests, including `exposes keyboard skip links to primary workbench regions`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "skip links" --project chromium` | Blocked locally | The Vite server started, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence used. |

Modal focus accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify accessible controls, dialog labels/modal state, dialog focusability/keyboard handling, and required skip-link targets. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding modal focus/Tab/Escape handling. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the modal focus update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery now lists 40 Chromium workflow tests, including `manages modal focus and Escape return paths`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "modal focus" --project chromium` | Blocked locally | The Vite server started, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence used. |

Status/progress live-region accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify accessible controls, dialog focus handling, required skip-link targets, and status/progress live regions. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding labeled status, watch, compile, export, and error live regions. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the status/progress live-region update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery now lists 41 Chromium workflow tests, including `exposes status and progress messages as live regions`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "status and progress" --project chromium` | Blocked locally | The Vite server started, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence used. |

Diagnostic/conflict screen-reader accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify diagnostic article labels, named diagnostic lists, and conflict diff cell group labels. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding diagnostic and conflict diff accessible names. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the diagnostic/conflict accessibility update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; diagnostic navigation and external conflict merge workflows now assert accessible list/listitem/group names. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "diagnostics\|merges external conflict" --project chromium` | Blocked locally | The Vite server started and selected the relevant workflows, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; the escalated browser install request was rejected by the auto-review layer, so no workaround was attempted and no GitHub Actions evidence was used. |

Table-editor screen-reader accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify the table editor grid name, header/cell/total label helpers, and row/sort/move control group labels. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding table editor grid/action/total labels. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the table editor accessibility update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; the table workflows now assert table grid/action/total accessible names. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "pasted tables\|table structure" --project chromium` | Blocked locally | The Vite server started and selected the two table workflows, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence was used. |
| `git diff --check` | Pass | No whitespace errors after the diagnostic/conflict/table accessibility updates. |

Desktop artifact smoke verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build created the frontend artifact consumed by the Tauri build. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary built at `src-tauri/target/release/neditor`. |
| `pnpm run test:desktop-smoke` | Pass | Checked `dist/index.html`, bundled JS assets, Tauri product/config/license metadata, npm/Cargo MIT metadata, and the executable release binary. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including the package-script guard for `test:desktop-smoke`. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the desktop smoke harness; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after adding the desktop smoke script and docs. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Blocked locally | Bounded GUI launch smoke requires elevated desktop app launch permission; the escalation request was rejected by the auto-review layer, so no workaround was attempted. |
| `git diff --check` | Pass | No whitespace errors after the desktop smoke harness update. |

High-contrast/reduced-motion accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue/CSS accessibility guardrails now verify high-contrast shell/control/focus styles and reduced-motion data/media-query styles. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding high-contrast focus CSS and settings workflow assertions. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the accessibility guard update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; the settings workflow now asserts high-contrast computed colors and reduced-motion zero-duration transitions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "persists editor settings" --project chromium` | Blocked locally | The Vite server started and selected the settings workflow, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence was used. |
| `git diff --check` | Pass | No whitespace errors after the high-contrast/reduced-motion accessibility update. |

Editor/preview screen-reader surface verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue accessibility guardrails now verify the Markdown source and live preview region labels, the focusable `role=document` preview article, and CodeMirror `role=textbox`/`aria-label`/`aria-multiline` content attributes. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding editor and preview document accessibility semantics. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the editor/preview accessibility update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; the boot workflow now asserts the named Markdown editor textbox and rendered preview document. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "boots the workbench" --project chromium` | Blocked locally | The Vite server started and selected the boot workflow, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence was used. |
| `git diff --check` | Pass | No whitespace errors after the editor/preview accessibility update. |

Malformed reference marker validation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked malformed_reference_markers_are_reported_with_source_ranges --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof rejects malformed heading labels, figure labels, empty cross references, slash-containing keys, and unclosed cross-reference markers with source-ranged errors while preserving generated heading anchors. |
| `cargo test --locked prepare_for_export_blocks_malformed_reference_markers_in_manifest --lib -- --nocapture` in `src-tauri` | Pass | Focused readiness proof blocks export for malformed labels/references and copies the source-ranged diagnostics into the export manifest readiness evidence. |
| `cargo test --locked duplicate_reference_labels_are_reported_with_source_ranges --lib -- --nocapture` in `src-tauri` | Pass | Existing duplicate-label regression still reports the duplicate anchor with first/duplicate origin context after tightening label parsing. |
| `cargo test --locked prepare_for_export_blocks_duplicate_reference_labels_in_manifest --lib -- --nocapture` in `src-tauri` | Pass | Existing export-readiness duplicate-label regression still copies duplicate-anchor diagnostics into the manifest. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 190 Rust library tests passed after strict reference marker validation and the shared label extractor fix for labels with attributes. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the reference validation update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the compiler/reference validation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend and docs update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the reference validation slice. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting strict reference key rules; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the reference validation slice. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the reference validation update. |

Bibliography import compatibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_parses_better_bibtex_edge_cases_without_splitting_at_symbols --lib -- --nocapture` in `src-tauri` | Pass | Focused BibTeX proof skips `@string`/`@comment` metadata records, handles parenthesis-delimited entries, dotted keys, protected title braces, locators, and `@` characters inside field values without splitting the entry. |
| `cargo test --locked compiler_loads_csl_json_object_variants_with_full_author_dates --lib -- --nocapture` in `src-tauri` | Pass | Focused CSL JSON proof accepts wrapper objects with `items`, full given/family authors, literal authors, raw date strings, string `date-parts`, and DOI-style slash keys. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 16 citation tests passed after extending BibTeX/CSL import compatibility and citation key parsing. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 192 Rust library tests passed after the bibliography parser update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the bibliography parser update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the bibliography parser update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the bibliography/parser docs update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the bibliography/parser update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting expanded BibTeX/CSL compatibility; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the bibliography/parser update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the bibliography/parser update. |

CSL alias rendering verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_maps_common_csl_style_aliases_to_native_renderers --lib -- --nocapture` in `src-tauri` | Pass | Focused citation proof maps `apa` to author-year output and `ieee` to numeric output without unsupported-style warnings. |
| `cargo test --locked compile_options_supply_csl_alias_default_citation_style --lib -- --nocapture` in `src-tauri` | Pass | Compile-option defaults now accept supported CSL aliases such as `apa` and render through the native author-year path. |
| `cargo test --locked compiler_warns_and_falls_back_for_unsupported_csl_style --lib -- --nocapture` in `src-tauri` | Pass | Unknown CSL style names still warn and fall back to title rendering. |
| `cargo test --locked prepare_for_export_validates_brand_and_default_style_options --lib -- --nocapture` in `src-tauri` | Pass | Export readiness accepts supported CSL aliases as default citation styles and still blocks unknown default styles before writing. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 18 citation tests passed after adding deterministic CSL alias rendering. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 23 export command tests passed after updating default citation-style validation for CSL aliases. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 194 Rust library tests passed after adding deterministic CSL alias support. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the CSL alias update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the CSL alias update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the CSL alias docs/update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the CSL alias update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting CSL aliases; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the CSL alias update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the CSL alias update. |

Broader equation syntax verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_broader_latex_equation_syntax --lib -- --nocapture` in `src-tauri` | Pass | Focused equation proof renders sums, superscripts, subscripts, approximation/infinity/arrows, Omega, and a `bmatrix` environment, preserves AST equation metadata, emits HTML/export CSS, and keeps DOCX/PPTX text evidence. |
| `cargo test --locked compiler_renders_block_and_inline_equations --lib -- --nocapture` in `src-tauri` | Pass | Existing equation regression still proves inline/display math, fractions, captions, labels, references, AST capture, and readiness cleanliness. |
| `cargo test --locked captioned_equations_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Existing cross-target equation regression still proves captioned equation text and references across HTML, PDF, DOCX, PPTX, and Markdown bundle outputs. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 195 Rust library tests passed after adding the broader native equation syntax coverage. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the equation renderer update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the equation renderer update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the preview/export math styling update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the equation renderer update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the broader equation subset; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the math preview styling update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the equation renderer update. |

Dependency admission guard verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:deps` | Pass | Checked dependency admission records for 39 manifest dependencies across `package.json` and `src-tauri/Cargo.toml`, and verified project MIT license metadata. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the dependency admission guard; all local links resolved. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including the package-script guard for `check:deps`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the dependency admission command. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the dependency admission update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the dependency admission guard update. |

Citation alias preference verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed; workspace persistence now preserves supported CSL aliases such as `APA` as `apa` and still falls back to `title` for unknown styles. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding CSL alias options to References and Settings selectors. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting citation alias preference support; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after adding citation alias options. |
| `pnpm run check:deps` | Pass | Dependency admission guard still passes after the citation preference update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the citation alias preference update. |

Project structure guard verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:structure` | Pass | Verified `src/App.vue` keeps the required top-level `template`, `<script setup>`, and `style` block order. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including the package-script guard for `check:structure`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the project structure guard. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the structure guard; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after adding the structure guard. |
| `pnpm run check:deps` | Pass | Dependency admission guard still passes after adding the structure guard script. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the structure guard update. |

Non-goal documentation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after adding first-release non-goals to the README and user guide; all local links resolved. |

Markdown bundle manifest sidecar verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_markdown_bundle_manifest_sidecar_info --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust readiness proof reports `includeManifest=false` on Markdown bundles as a non-blocking info diagnostic, keeps readiness true, and shows progress as embedded bundle-manifest work instead of sidecar output. |
| `cargo test --locked export_document_writes_optional_sidecar_manifest --lib -- --nocapture` in `src-tauri` | Pass | Direct export proof now verifies Markdown bundles with sidecar manifests disabled still write a ZIP containing `manifest.json`, suppress the sidecar path, keep final output path/hash evidence in the response manifest, and record embedded-manifest progress. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 24 export command tests passed after adding Markdown bundle sidecar/embedded manifest distinction coverage. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the Markdown bundle manifest update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the Markdown bundle manifest update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the user guide, matrix, TODO, and progress log; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the Markdown bundle manifest update. |

Browser workflow environment preflight verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright install chromium` | Pass | Installed/confirmed project-local Playwright Chromium without touching tracked files. |
| `pnpm run check:e2e-env` | Blocked locally | The new preflight found project-local Playwright Chromium, then classified the remaining failure as a macOS Mach bootstrap permission denial rather than a missing browser. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "boots the workbench" --project chromium` | Blocked locally | The focused workflow reached the installed browser executable and failed before app assertions with `bootstrap_check_in ... Permission denied (1100)`. |
| `node --check scripts/check-e2e-environment.mjs` | Pass | The E2E environment preflight script parses successfully. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including the package-script guard for `check:e2e-env`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the preflight script. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the preflight script; all local links resolved. |
| `pnpm run check:deps` | Pass | Dependency admission guard still passes after adding the package script. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after the E2E preflight update. |

Enabled export option matrix verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked enabled_export_option_matrix_survives_cross_target_artifacts --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust artifact proof covers enabled styles, syntax highlighting, cover pages, page numbers, glossary/comments/provenance appendices, PPTX agenda, presentation layout preset, watermark, text output, and Markdown bundle manifest evidence across HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked export_option_tests --lib` in `src-tauri` | Pass | 10 export option tests passed after adding the enabled cross-target option matrix fixture. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests still passed after adding the enabled option matrix proof. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the enabled option matrix fixture. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the enabled option matrix fixture. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after the enabled option matrix fixture. |

Extended LaTeX equation notation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_extended_latex_equation_notation --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust proof covers optional-index roots, `\text{}` labels, overline/underline/hat/vector wrappers, left/right delimiters, set/logic symbols, ellipses, preview/export CSS, AST capture, and DOCX/PPTX text evidence. |
| `cargo test --locked document_structure_tests --lib` in `src-tauri` | Pass | 22 document structure tests passed after extending native LaTeX notation rendering. |
| `cargo test --locked captioned_equations_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Captioned equation export conformance still passes across target artifacts. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the extended LaTeX renderer update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the extended LaTeX renderer update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding preview CSS for the new math notation wrappers. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue accessibility guardrails still pass after the preview CSS update. |
| `pnpm run check:structure` | Pass | The Vue single-file component structure guard still passes after the preview CSS update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests still pass after the preview CSS update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after the extended LaTeX renderer update. |

AI paste code fence cleanup verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked ai_cleanup_tests --lib` in `src-tauri` | Pass | 11 AI cleanup tests passed after adding code-fence normalization and rich HTML `<pre><code>` conversion coverage. |
| `cargo test --locked --lib` in `src-tauri` | Pass | Full Rust library suite passed with 200 tests after the AI cleanup code-fence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the AI cleanup code-fence update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the AI cleanup code-fence update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend AI cleanup update and documentation changes. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue accessibility guardrails still pass after the backend AI cleanup update. |
| `pnpm run check:structure` | Pass | The project structure guard still passes after the backend AI cleanup update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests still pass after the backend AI cleanup update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests, including the existing AI paste workflows. |
| `git diff --check` | Pass | No whitespace errors after the AI cleanup code-fence update. |

## Next Execution Order

1. Expand browser coverage for export artifact fidelity, target-specific export
   option matrices, progress/cancellation behavior if needed, remaining preview
   modes, broader keyboard shortcuts, deeper workspace grouping, AI review-state
   workflows, and table export modes.
2. Extend the new desktop smoke harness with WebDriver/Tauri-driver workflows
   for real local file, watcher, export, title, and restart behavior.
3. Use failures from workflow tests to drive implementation fixes.
4. Expand export fixture proof for HTML/PDF/DOCX/PPTX/Markdown bundle parity.
5. Add macOS/Windows optional transform engine evidence.
6. Only after behavior is locked, modularize oversized frontend/store/backend
   modules.

## Completion Gate

Do not mark the thread goal complete until:

- `docs/spec-completion-matrix.md` has current evidence for every explicit spec
  requirement.
- Every non-deferred row in the matrix is complete.
- The fresh verification baseline passes on the current commit.
- Browser/desktop workflow tests prove the main user journeys.
- Export artifacts are proven for business-document fidelity.
- Platform packaging and optional-engine evidence are documented.
- The worktree is clean and pushed.
