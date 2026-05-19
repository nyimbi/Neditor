# NEditor Current Completion Backlog

Updated: 2026-05-20

This file replaces the previous stale todo list. It is an evidence-based backlog
derived from the current repository state, `docs/specification.md`, and
`docs/external-transforms.md`. Treat this as the active planning source until it
is superseded by a newer audit.

## Audit Basis

Sources inspected in the current survey:

- Product and acceptance scope: `docs/specification.md`
- Optional external engine setup and safety model: `docs/external-transforms.md`
- User-visible status and developer commands: `README.md`
- Frontend workbench and UI implementation: `src/App.vue`
- Frontend document/workspace store: `src/stores/documents.ts`
- Backend compiler/export/transform/file/versioning modules under
  `src-tauri/src/`
- Existing backend tests under `src-tauri/src/tests/`
- Existing frontend unit tests under `tests/frontend-unit.test.ts`
- Current CI workflow: `.github/workflows/ci.yml`

Important interpretation rule: broad claims below are not considered complete
unless the current code and verification evidence prove them. Implemented
surfaces still need browser/desktop workflow verification where noted.

## Current High-Level State

NEditor is well past basic scaffolding. The current codebase contains:

- Tauri 2, Vue 3, Pinia, Vite, CodeMirror 6, vanilla CSS, and Rust IPC wiring.
- A large single-file frontend workbench with tabs, sidebars, command palette,
  settings, CodeMirror editing, live preview, export/review/versioning panels,
  AI paste cleanup, table editing controls, transform engine settings, and
  conflict UI.
- A Rust compiler pipeline with front matter, include expansion, source maps,
  variables, transforms, formulas, citations, bibliography, glossary, index,
  review/provenance parsing, document AST output, paged-document output,
  diagnostics, export manifests, and multiple export targets.
- Native Rust renderers or fallbacks for the required transform family, plus
  trust-gated external adapters for Graphviz/DOT, D2, PlantUML, and Pikchr.
- Backend unit/conformance tests across compiler, exports, transforms, tables,
  validation, file commands, snapshots, Git workflows, review/provenance, and
  media packaging.
- CI for macOS, Ubuntu, and Windows that runs Rust formatting/checks/tests,
  native-watch check, clippy, frontend unit tests, frontend build, and Tauri
  no-bundle desktop compilation.

The remaining work is therefore mostly about proving real workflows,
hardening incomplete edges, shrinking risk in oversized modules, and closing
spec details that are currently only partially satisfied.

## P0 - Must Finish Before Calling The Full Buildout Complete

### 1. Full Requirement-by-Requirement Completion Matrix

Build a maintained completion matrix from `docs/specification.md` section by
section.

For each requirement, record:

- Status: complete, partial, missing, non-goal, or unverified.
- Evidence: code path, test path, CI job, manual run, rendered artifact, or
  screenshot.
- Gaps: exact work needed to move partial/unverified items to complete.
- Verification command or manual workflow.

Why this is P0:

- The spec is large enough that informal "mostly done" status is unreliable.
- The user asked for full buildout, not a narrowed first-release subset.
- A completion claim needs auditable evidence for every explicit requirement.

Suggested artifact:

- Add `docs/spec-completion-matrix.md`.
- Keep `docs/todo.md` as the execution backlog and link matrix rows from it.

### 2. Browser/Desktop Workflow Tests

Add real interaction coverage for the most important frontend workflows.

Current evidence:

- Frontend logic unit tests exist for table parsing/serialization and conflict
  diff alignment.
- No Playwright/WebDriver/Tauri-driver e2e harness is currently present in
  `package.json`, CI, or the test tree.

Required workflow coverage:

- Open/edit/save/save-as/revert document flows.
- Workspace folder browsing, tab activation, pinning, recently closed reopen,
  and workspace restore.
- Split/source/preview/focus/export/review/presentation mode switching.
- Preview heading click-to-source and synchronized scrolling.
- Command palette search and command execution.
- Table editor interactions: create table, paste CSV/Markdown, add/remove rows
  and columns, sort, format columns, add totals, author merged cells, apply.
- Conflict modal interactions: compare, compose line merge, keep local, accept
  external, save local copy.
- AI paste cleanup modal: preview, options, insert modes, provenance output.
- Export readiness and export flow progress.
- Transform engine settings: path change clears trust, trust prompt, probe
  success/failure UI.

Implementation notes:

- Prefer Playwright for browser-level frontend checks if it can run against Vite
  without adding brittle desktop automation.
- Add Tauri-driver/WebDriver desktop smoke tests only after the browser-level
  harness is stable.
- Keep no-new-dependency discipline in mind, but this is one place where a test
  dependency may be justified because the spec requires workflow confidence.

### 3. Current Progress Log

Create or update a committed progress log that is separate from stale todo
history.

Required log content:

- Objective: full NEditor buildout against `docs/specification.md` and
  `docs/external-transforms.md`.
- Current audit date.
- Completed capability areas with evidence.
- Active known gaps.
- Verification commands last run, with pass/fail status.
- Commit hashes for completed slices.
- Known environment-specific limitations, such as macOS DMG creation failure.

Suggested artifact:

- `docs/progress.md`

Why this is P0:

- The user explicitly requested detailed logs of the goal and progress.
- `.omx/` logs are ignored by git and are not enough as durable project
  documentation.

### 4. Fresh Verification Baseline

Run and record a fresh verification baseline after the current audit.

Minimum commands:

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

Optional when environment supports it:

```sh
./node_modules/.bin/tauri build --bundles app
./node_modules/.bin/tauri build --bundles dmg --verbose
```

Record results in `docs/progress.md`. Do not claim completion from older
session output.

## P1 - Product Gaps And Hardening

### 5. Export Fidelity Completion Audit

The export stack is substantial, but the spec asks for professional business
exports, and current evidence still needs a stricter artifact-level audit.

Audit and finish:

- HTML, PDF, DOCX, PPTX, and Markdown bundle parity for headings, paragraphs,
  lists, tables, merged table cells, figures, captions, equations, footnotes,
  citations, cross references, glossary, index, review comments, change notes,
  AI provenance, legal disclaimers, and transform artifacts.
- Page numbering, headers, footers, watermarks, cover pages, logo, brand color,
  layout presets, front matter metadata, and section-level layout overrides.
- TOC behavior across preview/export, including depth, numbering, links, and
  page-numbered TOC where supported by the target format.
- PDF limitations: real layout quality, text wrapping, table pagination, page
  geometry, figure floats, widow/orphan behavior, column flow, and overflow.
- DOCX limitations: field codes, native tables, comments/change notes,
  bookmarks/internal links, media sizing/cropping, and compatibility with Word.
- PPTX limitations: slide layout selection, notes, agenda, section dividers,
  table-heavy slides, media sizing/cropping, and compatibility with PowerPoint.

Needed evidence:

- Package inspection tests are useful but insufficient alone.
- Add fixture exports and, where practical, rendered visual comparisons or at
  least text/package assertions tied to each spec requirement.

### 6. Export Readiness Report Completeness

The spec requires one-click "Prepare for export" confidence before sending a
document externally.

Audit and finish validation for:

- Required metadata.
- Includes and include graph.
- Citations and bibliography files.
- Formulas and formula dependency graph.
- Figures and image files.
- Transform engines, trust, input mode, timeout, output limits.
- Export settings.
- Broken links.
- Unresolved comments.
- AI provenance that is not human-reviewed.
- Draft status warnings and Git dirty-state warnings.

Make sure readiness reports expose actionable diagnostics in the UI and are
also represented in export manifests.

### 7. External Transform Platform Evidence

Current implementation has safety primitives and adapters for external
transforms, with Linux CI installing optional engines. The spec and external
setup doc are cross-platform.

Finish:

- Add macOS optional-engine CI or documented manual evidence for Graphviz, D2,
  PlantUML, and Pikchr.
- Add Windows optional-engine CI or documented manual evidence for the same
  engines.
- Confirm Windows executable path behavior, especially `.exe` paths and
  package-manager shims.
- Confirm PlantUML file mode on all platforms.
- Confirm cache invalidation includes enough adapter/engine identity for real
  upgrades, not only file size/mtime.
- Add diagnostics that distinguish missing executable, non-executable path,
  timeout, bad syntax, sidecar not produced, output limit, and stderr warnings.

### 8. File Watcher And Conflict Workflow Hardening

Backend watcher support and conflict UI exist, but workflow proof is still
needed.

Finish:

- Browser/desktop tests for clean external reload.
- Dirty root-file conflict tests through the UI.
- Dirty included-file conflict/recompile tests through the UI.
- Save-race conflict path, where a file changes after last watcher event but
  before save.
- Multi-tab watcher switching and stale watcher cleanup.
- Include graph changes after editing include directives.
- Save-copy conflict path with expected file content and active tab behavior.

### 9. Workspace And Tab Groups

Tabs, pinned tabs, recently closed items, workspace folder browsing, and grouped
display exist. The spec also asks for native tab organization and grouping by
folder/workspace/project/document set.

Audit and finish:

- Explicit document set grouping from front matter metadata.
- Folder/workspace/project grouping behavior with tests.
- Restore previous workspace after app restart, including active tab and pinned
  state.
- Recently closed behavior for unsaved and moved files.
- Clear UX for documents missing from restored workspace.
- Later split editor panes are explicitly not first-release critical, but the
  decision should be captured in the completion matrix as deferred/non-goal.

### 10. Editor Ergonomics

CodeMirror is integrated, but spec-level editor ergonomics need current proof.

Audit and finish:

- Markdown syntax highlighting.
- Diagnostics gutter accuracy and click/navigation behavior.
- Line numbers toggle and persistence.
- Word wrap toggle and persistence.
- Markdown shortcuts and smart list continuation.
- Auto-pairing for brackets, quotes, code fences, and emphasis markers.
- Find and replace.
- Spellcheck.
- Word/character/reading-time status.
- Outline navigation.
- Multi-cursor support if CodeMirror support is acceptable; otherwise mark
  deferred with rationale.
- Vim/emacs-style keybindings are listed as "later if feasible"; mark as
  deferred/non-goal unless implemented.

### 11. Preview Ergonomics

Live preview exists, but the spec requires several interaction details.

Audit and finish:

- Debounced rendering behavior on large documents.
- Synchronized editor/preview scrolling.
- Preview heading click jumps to source.
- Preview theme separate from editor theme.
- Inline warnings where appropriate.
- Transform-aware preview artifacts.
- Print/export preview mode behavior.
- Accessibility of preview content.

### 12. AI Paste And Governance

AI paste cleanup, provenance blocks, review status, and diagnostics exist. Close
the remaining workflow and policy gaps.

Finish:

- Browser-level tests for all insert modes: draft/insert, quote, appendix,
  replace selection, merge into current section, replace document.
- Clipboard-rich paste behavior across browsers/webviews where possible.
- Citation TODO insertion policy for unsupported factual claims.
- Provenance block alias handling and export behavior.
- Human-review toggles for AI source blocks and AI-assisted sections.
- Export warnings/readiness warnings for unreviewed AI content.

### 13. Tables, Calculations, And Data Sources

The table editor and formula engine are broad, but the full spec asks for a
business-grade table workflow.

Finish:

- Browser interaction tests for table editor workflows.
- Formula references to named tables and ranges, with dependency graph proof.
- Inline formulas and table-cell formulas across preview/export/readiness.
- Data sources from front matter and external CSV/TSV/JSON/YAML paths.
- Validation for malformed data source paths, broken formulas, circular or
  unsupported dependencies, and mixed span/formula tables.
- Export parity for large, merged, formatted, summarized, sorted, and
  formula-driven tables.

### 14. Bibliography, Citations, Index, Glossary, Cross References

Core support exists, but current completion needs artifact-level proof.

Finish:

- BibTeX and CSL JSON import edge cases.
- Duplicate bibliography key handling in UI and readiness report.
- Citation style behavior: title, author-year, key, and CSL-driven choices.
- Missing citation diagnostics with precise source ranges.
- Cross-reference links to headings, figures, tables, and equations across
  preview/export targets.
- Automatic index inclusion/exclusion and generated section behavior.
- Glossary definitions, hover/preview behavior, and export appendix behavior.

### 15. Layout And Reflow

Layout directives and paged-document output exist. The remaining risk is real
layout quality and proof.

Finish:

- Full layout directive parser coverage for page breaks, section breaks,
  columns, margins, orientation, page size, keep-with-next, keep-together,
  headers, footers, and slide directives.
- Paged-document model evidence for each directive.
- Export mapping evidence for HTML, PDF, DOCX, PPTX, and bundle outputs.
- Visual/manual review of representative PDF/DOCX/PPTX outputs.
- Better overflow behavior for large figures, equations, tables, code blocks,
  and long unbroken words.

### 16. Accessibility

The UI has many native controls and labels, but accessibility has not been
proven.

Finish:

- Keyboard-only navigation through toolbar, sidebar, tabs, editor, preview,
  modals, command palette, table editor, and conflict UI.
- Focus management in all modals.
- ARIA labels and roles for icon/tool buttons and custom separators.
- High contrast and reduced motion behavior.
- Screen-reader labels for diagnostics, status messages, table cells, conflict
  diff rows, and export progress.
- Automated checks where practical plus manual checklist evidence.

### 17. Performance And Large Documents

The spec calls out responsiveness for large documents and expensive transforms.

Finish:

- Benchmarks or stress tests for large Markdown documents, deep include graphs,
  many diagnostics, many tables, and many transform artifacts.
- Debounce and cancellation behavior for compile/preview updates.
- Progress reporting for expensive transforms and exports.
- Cache behavior for repeated transform execution.
- Memory growth checks for long editing sessions and repeated exports.

## P2 - Architecture And Maintainability

### 18. Frontend Modularization

`src/App.vue` is currently very large. This increases risk for any remaining
workflow work.

Refactor after locking behavior with tests:

- Extract toolbar/command palette.
- Extract sidebar panels: files, outline/references, diagnostics, export,
  snapshots, versioning, review, settings.
- Extract AI paste modal.
- Extract table editor.
- Extract conflict modal.
- Extract CodeMirror setup/decorations/navigation helpers.
- Keep Vue SFC block order as template, script, style.

Avoid broad styling churn while splitting. Preserve current behavior.

### 19. Frontend Store Modularization

`src/stores/documents.ts` owns too many domains.

Split cautiously after tests:

- Documents/tabs/workspace.
- Compile/diagnostics/preview.
- Export/readiness.
- Watch/conflict resolution.
- Git/snapshots/versioning.
- Transforms/preferences.
- AI cleanup/review governance.

Preserve persisted workspace schema or add migration handling.

### 20. Backend Modularization

The backend has already been split into many modules, but several files remain
large and high-risk.

Continue splitting:

- `document_ast.rs`
- `export/docx.rs`
- `export/pdf.rs`
- `export/pptx.rs`
- `export/shared.rs`
- `transforms/external.rs`
- Large nested test modules

Only split when tests are green and the boundary is obvious. Prefer moving
cohesive helpers over creating abstract layers.

### 21. String-Heavy Compiler And Export Paths

The compiler has a semantic AST foundation, but several export and validation
paths still parse rendered HTML or plain strings.

Reduce string scanning where it affects correctness:

- Use AST nodes for references, figures, tables, equations, layout, review
  comments, AI provenance, and transform artifacts.
- Use structured transform artifacts for export mapping.
- Avoid parsing generated HTML for table/media semantics when equivalent AST
  data is available.
- Keep raw string parsing only for Markdown syntax that has no structured model
  yet, and document why.

### 22. Dependency Admission Records

`docs/dependency-admission.md` exists, but it should stay current as test or
automation dependencies are added.

Finish:

- Add entries for any e2e/browser automation dependency if introduced.
- Revisit `@tauri-apps/plugin-shell` if external transforms stay purely Rust
  backend driven.
- Confirm licenses, runtime impact, alternatives considered, and security
  posture for each new dependency.

## P3 - Packaging, Release, And Documentation

### 23. Cross-Platform Packaging Evidence

CI proves desktop compilation with `tauri build --no-bundle`, but full packaging
evidence is incomplete.

Finish:

- macOS app bundle verification is documented; refresh it with current commit.
- macOS DMG currently has an environment-specific `hdiutil` failure documented
  in `README.md`; determine whether it is host-specific or config-specific.
- Windows packaging evidence for `.msi`/`.exe` or chosen bundle target.
- Linux packaging evidence for AppImage/deb/rpm or chosen bundle target.
- Confirm icons, app metadata, bundle identifier, signing/notarization notes,
  and updater stance.

### 24. User Documentation

Create practical docs for first users.

Needed docs:

- Getting started.
- Markdown extensions supported by NEditor.
- Includes and master documents.
- Export options and manifests.
- Review/release workflow.
- AI paste cleanup and governance.
- Tables/formulas/data sources.
- Bibliography/citations/cross references.
- Transform engine setup and trust model.
- Troubleshooting build/packaging/engine issues.

### 25. Example Project Fixtures

Create realistic sample documents that exercise the product.

Needed fixtures:

- Board paper.
- Consulting report with includes.
- Technical architecture doc with diagrams.
- Research report with bibliography and citations.
- Proposal with budget tables and formulas.
- AI-assisted draft with provenance and review workflow.

Each fixture should be exportable to all supported targets and usable in tests
or manual QA.

## Recommended Execution Order

1. Add `docs/spec-completion-matrix.md`.
2. Add `docs/progress.md` and record the fresh baseline.
3. Run the full verification baseline and update `docs/progress.md`.
4. Add browser-level frontend workflow tests.
5. Close gaps exposed by those workflow tests.
6. Audit export artifacts against the completion matrix and add missing
   conformance tests.
7. Harden cross-platform external transform evidence.
8. Modularize frontend and backend only after behavior is locked by tests.
9. Complete cross-platform packaging evidence.
10. Final pass: requirement-by-requirement audit, fresh verification, commit,
    push, and only then mark the goal complete.

## Completion Gate

Do not mark the full NEditor buildout complete until all of the following are
true:

- Every explicit requirement in `docs/specification.md` and
  `docs/external-transforms.md` is either implemented and verified or explicitly
  documented as a non-goal/deferred item with rationale.
- `docs/spec-completion-matrix.md` has current evidence for every requirement.
- `docs/progress.md` has current verification results.
- Backend tests, frontend tests, typecheck/build, clippy, native-watch check,
  and desktop compile pass on the current commit.
- Browser/desktop workflow coverage proves the main user journeys.
- Export fixtures prove the required business-document outputs.
- The worktree is clean and pushed.
