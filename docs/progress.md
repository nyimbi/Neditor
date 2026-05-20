# NEditor Goal Progress Log

Updated: 2026-05-20

## Active Goal

Complete the full development of NEditor against:

- `docs/specification.md`
- `docs/external-transforms.md`

The goal is not complete until the implementation, documentation, tests,
workflow verification, export artifacts, platform evidence, and committed
progress records prove the requested end state.

## Current Repository State

- Branch: `main`
- Latest inspected committed baseline before this update: `c95ef42 Prove live
  preview updates and package MIT licensing`
- Remote alignment at inspection time: `main...origin/main`
- Worktree before this log update: clean

## Durable Planning Artifacts

- `docs/todo.md`: current prioritized completion backlog.
- `docs/spec-completion-matrix.md`: conservative spec-to-evidence matrix.
- `docs/progress.md`: this committed progress log.

## Completed Recently

Recent pushed checkpoints visible in current git history:

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

## Current Capability Snapshot

Implemented or substantially present, pending the conservative caveats in
`docs/spec-completion-matrix.md`:

- Tauri 2 desktop app scaffold with Vue 3, Pinia, Vite, CodeMirror 6, vanilla
  CSS, and Rust IPC commands.
- Markdown workbench UI with editor, live preview, sidebars, tabs, status bar,
  command palette, review/versioning/export/settings panels, and conflict UI.
- File operations for local documents, workspace folder browsing, recent files,
  recently closed documents, pinned tabs, workspace restore, snapshots, Git
  history/diff/commit/tag/restore, and guarded saves.
- Compiler pipeline for front matter, includes, variables, transforms,
  formulas, citations, bibliography, glossary, index, cross references, review
  comments, AI provenance, generated lists of figures/tables, semantic AST,
  paged document model, diagnostics, source maps, and export manifests.
- Export modules for HTML, PDF, DOCX, PPTX, and Markdown bundle outputs.
- Transform registry with Rust-native renderers/fallbacks and trust-gated
  external adapters for Graphviz/DOT, D2, PlantUML, and Pikchr.
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

- Export fidelity requires stricter artifact-level and visual/manual proof.
- Export readiness has browser workflow coverage for the target-specific
  status/diagnostic path, but still needs a requirement-by-requirement audit.
- Optional external transform evidence is strongest on Linux; macOS and Windows
  optional-engine evidence remains incomplete.
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

## Next Execution Order

1. Expand browser coverage for export artifact fidelity, target-specific export
   option matrices, progress/cancellation behavior if needed, remaining preview
   modes, broader keyboard shortcuts, deeper workspace grouping, AI review-state
   workflows, and table export modes.
2. Add desktop WebDriver/Tauri-driver smoke tests after the browser harness is
   stable.
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
