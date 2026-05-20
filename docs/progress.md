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
- Latest inspected committed baseline before this update: `25c7d1e Stabilize
  conflict merge workflow proof`
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
  comments, AI provenance, semantic AST, paged document model, diagnostics,
  source maps, and export manifests.
- Export modules for HTML, PDF, DOCX, PPTX, and Markdown bundle outputs.
- Transform registry with Rust-native renderers/fallbacks and trust-gated
  external adapters for Graphviz/DOT, D2, PlantUML, and Pikchr.
- Business features including AI paste cleanup, table editor logic, formula
  handling, diagrams, citations, captions, layout directives, export readiness,
  and review/release metadata.
- Backend test coverage across many compiler, export, transform, table,
  validation, media, file, Git, snapshot, review, and provenance paths.
- Frontend unit coverage for table logic and conflict diff alignment.
- Playwright browser workflow harness for Vite with mocked Tauri IPC,
  covering view mode switching, command palette table insertion, table editor
  insertion, mocked file lifecycle operations, advanced table
  paste/sort/formula/merge/apply behavior, save-as plus recently closed
  reopening, stale-save conflict copy/merge recovery, AI paste cleanup
  insertion, and export readiness.
- CI matrix for macOS, Ubuntu, and Windows with Rust formatting/check/test,
  native-watch check, clippy, frontend unit tests, frontend build, and Tauri
  no-bundle compile.
- CI browser workflow job on Ubuntu with Playwright Chromium installation and
  `pnpm run test:e2e`.

## Active Known Gaps

P0 gaps:

- Latest pushed CI for commit `25c7d1e` is green: browser workflow, Ubuntu
  desktop, macOS desktop, and Windows desktop all passed in run `26138672512`.
  The prior Windows path-sensitive Rust-test failures, Ubuntu installed Pikchr
  conformance failure, and Ubuntu fake-`d2` stdin fixture failure are resolved
  in current CI.
- Browser-level workflow tests pass in Linux CI with nine Chromium tests in
  run `26138672512`, including mocked file lifecycle coverage, save-as plus
  recently closed reopening, and stale-save conflict copy/merge recovery.
  Coverage remains incomplete, and local focused Playwright execution is blocked
  by the missing/default macOS Playwright cache and the workspace-local Chromium
  Mach bootstrap permission failure.
- Desktop WebDriver/Tauri-driver workflow tests are missing.
- Current progress/matrix/docs need to be kept updated as evidence changes.

P1 gaps:

- Export fidelity requires stricter artifact-level and visual/manual proof.
- Export readiness coverage needs a requirement-by-requirement audit.
- Optional external transform evidence is strongest on Linux; macOS and Windows
  optional-engine evidence remains incomplete.
- File watcher/conflict flows need UI workflow tests.
- Workspace/tab-group behavior needs restart and document-set grouping proof.
- Editor and preview ergonomics need browser interaction proof.
- AI paste, citations, layout, accessibility, performance, and remaining table
  editor/export fixture paths need workflow, artifact, or benchmark evidence
  before they can be considered complete.

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

Current CI evidence log:

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

Relevant CI fixes already landed:

- `.github/workflows/ci.yml` now installs pnpm with `pnpm/action-setup@v4`
  before `actions/setup-node@v4` in both the desktop matrix and browser
  workflow job.
- `tests/frontend-unit.test.ts` now asserts that the CI workflow keeps the pnpm
  setup step and both frontend test commands.
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

1. Expand browser coverage for workspace restore, conflicts, preview
   navigation, scroll sync, transform settings, export progress, and the
   remaining AI/table modes.
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
