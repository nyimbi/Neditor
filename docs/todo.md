# NEditor Current Completion Backlog

Updated: 2026-05-20

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
  AI paste insertion modes, conflict merge-line composition, and local
  verification script coverage.
- A Playwright browser workflow harness for the Vite-rendered workbench with a
  browser-side Tauri IPC mock.
- Local verification commands for frontend logic, browser workflows, Rust
  checks/tests, native-watch compilation, static analysis, frontend builds, and
  Tauri desktop compilation.

The remaining work is primarily about workflow proof, artifact fidelity,
cross-platform validation, and reducing risk in oversized modules after
behavior is locked.

## Current Verification Snapshot

Latest pushed code commit inspected:

- `c95ef42 Prove live preview updates and package MIT licensing`

Remote GitHub Actions are not an active verification surface for this project.
Older run references below are retained only as historical debugging context and
must not be used as completion evidence for new work. Current completion proof
comes from local command output, committed artifacts, rendered/manual QA, and
explicit platform checks run outside GitHub Actions.

Most recent local verification evidence:

- `pnpm exec playwright test --list`: listed the browser workflow harness with
  command-palette insertion proof for `[TOC]`, `[INDEX]`, `[BIBLIOGRAPHY]`,
  `[LIST_OF_FIGURES]`, and `[LIST_OF_TABLES]`.
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
  file, keep local editor edits while leaving the external disk edit untouched,
  and accept external disk content into the active document.
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
- External conflict modal: more granular line-compose controls.
- AI paste cleanup remaining proof: clipboard and richer review-state flows.
- Export artifact fidelity, target-specific option matrices,
  progress/cancellation behavior if needed, and rendered/manual proof.
- Large-document preview debounce timing proof.
- Remaining transform engine settings: disabled-engine and cross-platform
  executable edge cases beyond the mocked browser workflow.

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

Audit HTML, PDF, DOCX, PPTX, and Markdown bundle outputs for:

- Headings, paragraphs, lists, nested lists, block quotes, callouts, code blocks.
- Tables, merged cells, alignment, formulas, totals, captions, and large table
  pagination/splitting.
- Figures, captions, cover crop/fit, relative media packaging, duplicate media
  names, and missing media diagnostics.
- Generated `[LIST_OF_FIGURES]` and `[LIST_OF_TABLES]` sections now have a
  focused compiler/export artifact test for numbering, anchors, fenced-example
  exclusion, preview HTML, and DOCX text.
- Equations, numbering, references, and cross-target rendering.
- Citations, bibliography, locators, missing keys, duplicate keys, and CSL
  behavior.
- Cross references to headings, figures, tables, equations, appendices, and
  decision records.
- Glossary, index, TOC, generated sections, and appendices.
- Review comments, change notes, release metadata, AI provenance, legal
  disclaimers, draft warnings, and approval metadata.
- Page size, orientation, margins, columns, breaks, keep-with-next,
  keep-together, headers, footers, watermarks, cover pages, page numbers,
  brand profile, logo, colors, and fonts.
- Transform artifacts from native renderers and external engines.

Needed proof:

- Package/text assertions where appropriate.
- Rendered or manually inspected representative PDF/DOCX/PPTX artifacts.
- Fixture exports tied back to matrix rows.

### 6. Export Readiness Completeness

Status: implemented in part; requirement coverage needs audit.

Current evidence:

- Readiness now reports malformed review comments and change notes when author,
  timestamp, or body text is missing, and the diagnostics are copied into the
  export manifest.

Readiness should validate and report:

- Required metadata.
- Release status and approval metadata.
- Draft/export warnings.
- Includes and include graph.
- Broken local links and missing media.
- Citations, bibliography files, missing keys, duplicate keys, and style issues.
- Formulas, table formulas, dependencies, circular references, and invalid
  expressions.
- Figures, captions, references, glossary, and index.
- Transform engines, trust state, executable paths, adapter input mode,
  timeouts, stderr, missing output, output limits, and cache identity.
- Export target options and target-specific blockers.
- Unresolved comments and malformed comment/change-note audit metadata.
- AI provenance that is not human reviewed.
- Dirty Git state and export manifest state.

Completion criteria:

- Readiness UI exposes actionable diagnostics.
- Export commands block or warn consistently with readiness results.
- Export manifests include enough readiness context for auditability.

### 7. External Transform Platform Evidence

Status: Linux installed-engine evidence now passes in CI; macOS and Windows
optional-engine evidence is incomplete.

Finish:

- Preserve installed-engine conformance locally where engines are available
  while expanding optional engine proof beyond Linux.
- Keep Graphviz/DOT, D2, PlantUML, and Pikchr as real optional-engine proof
  where available.
- Add macOS manual evidence for all optional engines.
- Add Windows manual evidence for all optional engines.
- Confirm Windows `.exe` paths and package-manager shims.
- Confirm PlantUML file mode on all platforms.
- Confirm Pikchr stdin/file/argument mode for each supported executable shape.
- Confirm cache invalidation includes adapter identity, executable identity,
  arguments, file size/mtime, and relevant version data.
- Preserve diagnostics for missing executable, non-executable path, timeout,
  bad syntax, sidecar not produced, output limit, and stderr warnings.

### 8. File Watcher And Conflict Workflows

Status: backend and UI exist; stale-save conflict copy/merge/keep-local/
accept-external workflow proof exists in archived browser workflow evidence.
Clean watcher reload and watcher-originated dirty root-file conflict proof are
also present in archived browser workflow evidence. Clean included-file
recompile and dirty included-file conflict proof are also present in archived
browser workflow run `26145509141`.

Finish:

- Clean external reload for unchanged local documents. Browser archived workflow run
  `26140882880` covers this for root-file changes.
- Dirty root-file conflict through UI. Browser archived workflow run `26140882880` covers this
  for watcher-originated root-file changes.
- Dirty included-file conflict and master recompilation through UI. Browser CI
  run `26145509141` covers clean included-file recompile and dirty
  included-file conflict handling.
- Save-race conflict when a file changes after the last watcher event but
  before save. Browser archived workflow run `26139678118` covers the stale-save conflict path
  through compare, save-copy preservation, merge-back recovery, keep-local, and
  accept-external.
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
- Inline warning rendering.
- Transform-aware preview artifacts.
- Print/export preview mode behavior.
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
- AI-assisted section review-state toggles.
- Readiness/export warnings for unreviewed AI content.
- Export appendix behavior for AI provenance.

### 13. Tables, Calculations, And Data Sources

Status: broad implementation exists; workflow and artifact proof need expansion.

Finish:

- Table editor workflows for paste, add/remove rows/columns, sort, format,
  totals, merged cells, readable Markdown output, and cancellation.
- Named table/range references and formula dependency graph proof.
- Inline formulas and table-cell formulas in preview/export/readiness.
- Data sources from front matter and external CSV/TSV/JSON/YAML paths.
- Validation for malformed data source paths, broken formulas, circular or
  unsupported dependencies, and mixed span/formula tables.
- Export parity for large, merged, formatted, summarized, sorted, and
  formula-driven tables.

### 14. Bibliography, Citations, Index, Glossary, And Cross References

Status: core support exists; UI and cross-target proof remain incomplete.

Finish:

- BibTeX and CSL JSON import edge cases.
- Duplicate bibliography key UI and readiness reporting.
- Citation styles: title, author-year, key, and CSL-driven choices.
- Missing citation diagnostics with precise ranges.
- Cross-reference links across preview, HTML, PDF, DOCX, PPTX, and bundle
  outputs.
- Automatic index inclusion/exclusion.
- Glossary definition preview, hover behavior, export appendix behavior, and
  command-palette navigation.

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

Status: many controls have labels/roles; no full accessibility audit exists.

Finish:

- Keyboard-only navigation through toolbar, sidebar, tabs, editor, preview,
  modals, command palette, table editor, and conflict UI.
- Focus management in all modals.
- ARIA labels and roles for custom controls.
- High contrast and reduced motion behavior.
- Screen-reader labels for diagnostics, status messages, table cells, conflict
  diff rows, and export progress.
- Automated checks where practical plus manual checklist evidence.

### 17. Performance And Large Documents

Status: debounce/cache/progress code exists; stress evidence is missing.

Finish:

- Benchmarks or stress tests for large Markdown documents, deep include graphs,
  many diagnostics, many tables, and many transform artifacts.
- Debounce and cancellation behavior for compile/preview updates.
- Progress reporting for expensive transforms and exports.
- Cache behavior for repeated transform execution.
- Memory growth checks for long editing sessions and repeated exports.

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

Constraints:

- Preserve persisted workspace schema or add migrations.
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

Needed docs:

- Getting started.
- Markdown extensions supported by NEditor.
- Includes and master documents.
- Export options and manifests.
- Review and release workflow.
- AI paste cleanup and governance.
- Tables, formulas, and data sources.
- Bibliography, citations, glossary, index, and cross references.
- Transform engine setup and trust model.
- Troubleshooting build, packaging, and engine issues.

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
6. Modularize frontend/store/backend code after behavior is locked.
7. Complete packaging evidence, user docs, and example projects.
8. Run a final requirement-by-requirement audit and fresh verification baseline.

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
