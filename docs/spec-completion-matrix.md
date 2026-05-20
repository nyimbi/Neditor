# NEditor Specification Completion Matrix

Updated: 2026-05-20

Status vocabulary:

- Complete: current code and tests provide direct evidence.
- Partial: implementation exists, but scope or proof is incomplete.
- Unverified: implementation appears to exist, but there is no current direct
  workflow, artifact, or platform proof.
- Missing: no current implementation evidence was found.
- Deferred: the specification marks the item as later/non-goal, or the project
  has intentionally deferred it.

This matrix is intentionally conservative. If evidence is indirect, the status
is not "Complete".

## Core Product Scope

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 1 Purpose | Local-first business Markdown workbench | Partial | `README.md`; Tauri/Vue/Rust project layout; compiler/export modules | Full buildout still needs requirement-by-requirement verification and workflow tests. |
| 2 Source Prompt Extension | Tauri 2, Vue 3, Pinia, Vite, vanilla CSS | Complete | `package.json`; `src/main.ts`; `src/App.vue`; `src-tauri/Cargo.toml` | Keep dependency admission docs current as test dependencies are added. |
| 2 Source Prompt Extension | Side-by-side Markdown editor and live preview | Partial | `src/App.vue` editor/preview panes; CodeMirror setup | Browser workflow proof for live typing, scroll sync, and preview navigation. |
| 2 Source Prompt Extension | Split, preview-only, focus/source modes | Partial | `src/App.vue` mode controls; `e2e/app-workflows.spec.ts` covers split/source/preview modes; GitHub Actions browser job passed on commit `420af08` | Focus/export/review/presentation modes and desktop execution still need proof. |
| 2 Source Prompt Extension | Local new/open/save/save-as flows | Partial | `src/stores/documents.ts`; `src-tauri/src/filesystem.rs`; file command tests | Browser/desktop workflow tests for dialogs and state transitions. |
| 2 Source Prompt Extension | Window title reflects file and dirty state | Partial | `windowTitle` getter and `setWindowTitle` in `src/App.vue` | Desktop/browser assertion for title updates. |
| 2 Source Prompt Extension | Toolbar commands and keyboard shortcuts | Partial | `src/App.vue` toolbar, command palette, shortcut handler | Test coverage for command execution and keybindings. |
| 2 Source Prompt Extension | Light/dark/system theme and typography | Partial | Settings in `src/App.vue`; persisted preferences in store | Visual/accessibility verification across themes. |
| 2 Source Prompt Extension | Cross-platform packaging | Partial | `.github/workflows/ci.yml`; `README.md` packaging note | Full bundle evidence for Windows/Linux and refreshed macOS bundle/DMG status. |
| 3 Product Positioning | Local-first document workbench, not cloud suite | Partial | Local file/snapshot/Git/export design | User documentation and fixture workflows need to prove positioning. |
| 4 Target Users | Business, technical, research, product users | Unverified | Feature set aligns with these personas | Realistic example projects and workflows needed. |

## Carry-Forward MacDown Improvements

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 5.1 External File Refresh | Watch open files | Partial | `src-tauri/src/filesystem_watch.rs`; `src/stores/documents.ts`; native-watch CI check; stale-save conflict recovery is browser-proven in CI run `26139678118`; clean root-file watcher reload is browser-proven in CI run `26140882880`; clean included-file recompile and dirty included-file conflict flows are browser-proven in CI run `26145509141` | Desktop workflow tests. |
| 5.1 External File Refresh | Non-destructive conflicts: compare, accept external, keep local, save copy | Partial | Conflict modal in `src/App.vue`; `src/lib/conflict.ts`; frontend unit diff test; browser CI run `26139678118` covers stale-save compare visibility, local-copy preservation, merge-back recovery, keep-local, and accept-external; browser CI run `26140882880` covers watcher-originated dirty root-file conflict compare visibility | Included-file conflict resolution tests and more granular line-compose controls. |
| 5.1 External File Refresh | Watch included files and recompile master docs | Partial | Watch setup includes manifest included files; include tests; browser CI run `26145509141` covers clean included-file recompile and dirty included-file conflict workflows | Deeper include graph navigation edits. |
| 5.2 Build Toolchain Lessons | Pinned local JS/Rust dependencies | Partial | `pnpm-lock.yaml`; `src-tauri/Cargo.lock`; README commands | Confirm no hidden global dependency remains in packaging and optional engines. |
| 5.2 Build Toolchain Lessons | Diagnostics for missing external engines | Partial | `transforms/external.rs`; external transform tests | Cross-platform diagnostic proof and UI workflow coverage. |
| 5.3 Business Export Customization | HTML/PDF/DOCX/PPTX exports | Partial | `src-tauri/src/export/*`; export tests | Artifact-level conformance and rendered/manual checks for each target. |
| 5.3 Business Export Customization | Page numbering, headers/footers, logo, brand color, cover, watermark, presets, metadata | Partial | Export option tests; export shared helpers; frontend settings | Completion matrix rows for each option and visual artifact proof. |
| 5.4 Master Documents And Includes | Include syntaxes, relative resolution, strip child front matter, circular diagnostics | Partial | Compiler/document structure tests | Add completion evidence for max depth, invalid include files, navigation, and UI include graph. |
| 5.5 Table Of Contents | `[TOC]`, front matter TOC, depth, numbering, export formatting | Partial | Generated sections and export tests | Page-numbered TOC proof for PDF/DOCX where supported; browser preview tests. |
| 5.6 Native Tab Organization | Multiple docs, tabs, pinning, recents, restore | Partial | `src/App.vue`; `src/stores/documents.ts` | Document-set grouping, restart restore workflow, and tab behavior tests. |
| 5.6 Native Tab Organization | Split editor panes later | Deferred | Spec says "later versions" | Record as deferred/non-goal for current release. |

## Core Application Requirements

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 6.1 Application Shell | Tauri IPC commands exposed from Rust | Partial | `src-tauri/src/lib.rs` command registrations | Verify command coverage against 25.4 and UI workflow. |
| 6.1 Application Shell | Vue SFC block order template/script/style | Partial | `src/App.vue` follows order | Preserve when modularizing; add lint/check if practical. |
| 6.2 Primary Layout | Toolbar, status bar, sidebar, editor, preview | Partial | `src/App.vue` | Browser layout/accessibility checks across viewport sizes. |
| 6.2 Primary Layout | Export preview, review, presentation outline modes | Partial | Mode controls and sidebars in `src/App.vue` | Interaction tests and product QA for each mode. |
| 6.3 Editor | CodeMirror 6 engine | Complete | CodeMirror dependencies and imports in `src/App.vue` | None beyond ongoing behavior tests. |
| 6.3 Editor | Markdown highlighting, diagnostics gutter, decorations | Partial | CodeMirror markdown/linter/lintGutter/decorations | Tests for precise diagnostic navigation and visual states. |
| 6.3 Editor | Line numbers, word wrap, spellcheck, find/replace, word count | Partial | Settings and CodeMirror extensions; status bar stats | Browser tests for toggles and find/replace. |
| 6.3 Editor | Smart list continuation and auto-pairing | Partial | `continueMarkdownList`; `closeBrackets` | Unit/browser tests for editor behavior. |
| 6.3 Editor | Multi-cursor support | Unverified | CodeMirror may support it by default | Decide complete/deferred and document. |
| 6.3 Editor | Vim/emacs keybindings | Deferred | Spec says later if feasible | Record deferred unless implemented later. |
| 6.4 Preview | Live debounced preview | Partial | Store compile on editor update; debounce in `src/App.vue` | Large-document behavior and timing tests. |
| 6.4 Preview | Scroll sync and heading click-to-source | Partial | Preview/editor scroll handlers; click handler | Browser tests. |
| 6.4 Preview | Separate preview theme, inline warnings, transform blocks, export preview | Partial | Preview theme setting; diagnostics; transform rendering; modes | UI verification and visual tests. |
| 6.5 File Operations | New, open file, open folder, save, save as, revert, rename, duplicate, reveal | Partial | Store actions and Rust file commands; file command tests; Playwright mocked workflows cover open/save/save-as/duplicate/rename/pin/reveal/revert in CI run `26137556147`; stale-save conflict copy/merge/keep-local/accept-external recovery in CI run `26139678118`; watcher-originated root reload/conflict proof in CI run `26140882880`; included-file watcher conflict proof in CI run `26145509141` | Native desktop dialog workflow tests. |
| 6.5 File Operations | Recent docs/folders, workspace restore | Partial | Persisted workspace store; Playwright mocked workflows cover workspace listing after open and recently closed reopening in CI run `26137556147`; local Playwright discovery now includes restart-style restore for open tabs, active tab, pinned state, mode/sidebar, workspace root, and recent files | Pushed restart restore CI proof, missing/moved/deleted restore workflow tests, scroll restore, and fuller recent folder behavior. |
| 6.5 File Operations | External change detection/conflict handling | Partial | Watch/conflict code and tests; browser CI run `26139678118` covers stale-save conflict blocking, compare, save-copy preservation, merge-back recovery, keep-local, and accept-external; browser CI run `26140882880` covers clean root reload and watcher-originated dirty root conflict; browser CI run `26145509141` covers clean included-file recompile and dirty included-file conflict flows | Desktop workflow proof. |

## Compiler And Document Model

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 7.1 Pipeline | Load, front matter, includes, source map, variables, transforms, formulas, citations, refs, generated sections, semantic model, preview/export | Partial | `src-tauri/src/compiler.rs`; supporting modules; many backend tests | Replace remaining string-heavy paths where correctness depends on semantic structure. |
| 7.2 Outputs | Compiled Markdown, HTML, semantic model, diagnostics, include graph, source map, metadata, bibliography, index, formula graph, transform artifacts, manifest | Partial | `CompileResponse` types and compiler tests | Completion evidence for every output in matrix and UI consumers. |
| 7.3 Diagnostics | Severity/message/source/range/suggested fix/related items | Partial | `diagnostics.rs`; diagnostic tests | Ensure all diagnostic families have precise ranges and suggested fixes where possible. |
| 8 Front Matter And Variables | YAML metadata, layout/export controls, variable resolution, filters | Partial | `front_matter.rs`; `variables.rs`; export options | Variable filter completeness, UI docs, malformed front matter diagnostics. |

## Twenty High-Value Business Improvements

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 9.1 Git versioning | Detect repo, branch/dirty, commit, history, diff, restore, tag | Partial | `git.rs`; `git_support.rs`; versioning UI; git workflow test | Browser/desktop workflow tests and Git-free UX validation. |
| 9.1 Git-free snapshots | Local snapshots for non-Git users | Partial | `snapshot.rs`; snapshot UI/tests | Restore workflow through UI and storage-mode docs. |
| 9.2 Export snapshots | Manifest with hashes/options/app/version/status/timestamp | Partial | `manifest.rs`; `export_commands.rs`; export command tests | Confirm all export targets write/consume manifests consistently. |
| 9.3 Release workflow | Status values, badge, draft export warning, approval metadata, release tagging | Partial | Review UI; validation; Git tag command | Visual badge proof and workflow tests. |
| 9.4 AI paste cleanup | Normalize chat output, code fences, bullets, tables, links, citations, insert modes | Partial | `ai_cleanup.rs`; AI modal; AI cleanup tests; shared insertion helper tests; Playwright coverage for insert, quote, appendix, replace document, merge into section, and replace selection modes | Clipboard behavior, provenance workflows, citation TODOs, and non-sandboxed execution. |
| 9.5 AI provenance | `ai-source`, AI-assisted sections, export appendix | Partial | `provenance.rs`; review/provenance tests; UI toggles | Workflow and export readiness proof for unreviewed content. |
| 9.6 Includes | Include graph, diagnostics, re-render, export single doc | Partial | Document structure tests; compiler support | UI include graph/navigation and watcher workflow proof. |
| 9.7 Business table editor | Visual editor, rows/cols, alignment, paste, sort, formats, readable Markdown, export | Partial | `src/lib/tables.ts`; table UI; frontend/backend tests; Playwright coverage for table insertion, pasted table import, numeric sorting, formula rows, merged-cell metadata, row/column add-remove behavior, column format totals, cancel-without-applying behavior, and apply-back-to-editor behavior | Remaining non-sandboxed browser execution and more export fixtures. |
| 9.8 Calculations | Calc blocks, inline formulas, table formulas, named values/tables, dependency graph | Partial | `calculations.rs`; table tests; compiler tests | Named table/range coverage, circular dependency behavior, UI proof. |
| 9.9 Equations | Inline/display math, numbering, references, export support | Partial | `rich_blocks.rs`; export tests | Cross-target artifact proof and editor UX tests. |
| 9.10 TOC | Automatic TOC with marker/front matter | Partial | `generated_sections.rs`; document/export tests | Page-numbered target proof and UI controls. |
| 9.11 Index | Automatic index and exclusions | Partial | `indexing.rs`; compiler tests | UI/export proof for index options. |
| 9.12 Bibliography | BibTeX/CSL JSON, citation syntax, rendered bibliography | Partial | `bibliography.rs`; citation tests | CSL style fidelity, UI manager workflow, duplicate-key UI. |
| 9.13 Cross references | Figures/tables/equations/headings, broken ref diagnostics, export links | Partial | `references.rs`; citation/export tests | Full cross-target proof. |
| 9.14 Captions | Figures/table captions, numbering, list support | Partial | AST/export/table/figure tests | List of figures/tables proof and UI workflow. |
| 9.15 Advanced layout | Page size/orientation/margins/columns/breaks/headers/footers/keeps/floats | Partial | `layout.rs`; `paged_document.rs`; export tests | Rendered/manual QA and overflow hardening. |
| 9.16 Brand templates | Brand name/color/logo/font/header/footer/watermark/legal disclaimer | Partial | Settings UI; export option tests | Template/profile persistence and cross-target visual proof. |
| 9.17 Review comments/change notes | Comments, unresolved validation, exports | Partial | `review.rs`; review UI; export conformance tests | UI workflow tests and native target fidelity. |
| 9.18 Document variables | Front matter/project/data variables | Partial | `variables.rs`; project variable tests | Filter coverage and docs. |
| 9.19 Pikchr diagrams | Native fallback/external setup and diagnostics | Partial | `transforms/diagram.rs`; `external.rs`; transform tests | Cross-platform optional engine proof. |
| 9.20 Validation | One-click prepare report across metadata/includes/citations/formulas/figures/transforms/settings/links/comments | Partial | `validation.rs`; `export_commands.rs`; readiness UI; initial Playwright readiness test | Completeness audit, target-specific readiness cases, and non-sandboxed browser execution. |

## Fenced-Code Transform System

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 10.1 Architecture | Registry, option validation, artifact cache, diagnostics | Partial | `transforms/renderer.rs`; `transforms/options.rs`; `transforms/external.rs` | More option validation coverage and structured artifact use in exports. |
| 10.2 Safety | No network, no shell, trust, timeout, output limits, source hash cache | Partial | External transform runner and tests | Cross-platform process behavior and cache identity hardening. |
| 10.3 Core transforms | `calc`, `mermaid`, `pikchr` | Partial | Renderer and transform tests | Fidelity limitations documented; optional engine proof for Pikchr. |
| 10.4.1 DOT/Graphviz | SVG diagrams and engines | Partial | Native fallback; external adapter; Linux CI conformance | Engine variants and Windows/macOS proof. |
| 10.4.2 PlantUML | SVG/PNG enterprise diagrams | Partial | External adapter and fallback | Real PlantUML cross-platform proof; PNG support decision. |
| 10.4.3 D2 | SVG diagrams | Partial | External adapter and fallback | Cross-platform proof. |
| 10.4.4 Vega-Lite | Charts | Partial | Native SVG subset in `visual_data.rs` | Document supported subset; broader Vega-Lite support or clear diagnostics. |
| 10.4.5 Chart | Bar/line/pie/area/KPI | Partial | `chart.rs`; transform tests | Export/visual fixture coverage. |
| 10.4.6 GeoJSON | Static map preview | Partial | `visual_data.rs`; transform tests | More geometry support or explicit subset docs. |
| 10.4.7 TopoJSON | Static map preview | Partial | `visual_data.rs`; transform tests | More topology support or explicit subset docs. |
| 10.4.8 STL | Static preview | Partial | `visual_data.rs`; transform tests | 3D fidelity limits and export proof. |
| 10.4.9 CSV | Tables and formulas | Partial | `tables.rs`; transform/table tests | Data source workflow and UI proof. |
| 10.4.10 TSV | Tables and formulas | Partial | `tables.rs`; transform tests | Data source workflow and UI proof. |
| 10.4.11 JSON | Structured rendering | Partial | `structured.rs`; transform tests | Large/nested docs and schema interactions. |
| 10.4.12 YAML | Structured rendering | Partial | `structured.rs`; transform tests | Large/nested docs and schema interactions. |
| 10.4.13 OpenAPI | API docs | Partial | `structured.rs`; transform tests | Richer endpoint/schema rendering. |
| 10.4.14 JSON Schema | Schema docs | Partial | `structured.rs`; transform tests | Richer schema rendering. |
| 10.4.15 BibTeX | Bibliography rendering | Partial | `business.rs`; transform tests | Edge-case parsing and UI integration. |
| 10.4.16 Glossary | Definitions and term rendering | Partial | `business.rs`; glossary/index tests | Hover/UI/export proof. |
| 10.4.17 Timeline | Timeline SVG | Partial | `business.rs`; transform tests | Visual/export fixture coverage. |
| 10.5 Later transforms | roadmap, ADR, diff, QR, etc. | Partial | Several implemented in transform registry | Decide which are first-release, document supported syntax. |

## AI, Versioning, Tables, Equations, Citations

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 11 AI workflow | Paste cleanup preview and governance | Partial | AI modal, store actions, backend tests | Browser tests and docs. |
| 12 Versioning | Snapshots, Git integration, version metadata | Partial | Snapshot/Git modules and UI | End-to-end workflow tests and docs. |
| 13 Tables/data | Table editing, formulas, data sources | Partial | Table lib/UI/backend tests | Interaction tests and more data-source validation. |
| 14 Equations | Math authoring/render/export | Partial | Rich block parsing/export tests | Formula/equation references across targets. |
| 15 Bibliography/citations | BibTeX/CSL/citation rendering | Partial | Bibliography/citation tests | Citation manager UX and style fidelity. |
| 16 Index/glossary | Index and glossary generation | Partial | Indexing/glossary modules/tests | UI and export appendix proof. |
| 17 Layout/reflow | Layout model/directives/export mapping | Partial | Layout, paged document, export tests | Rendered visual QA and overflow/performance work. |
| 18 Export system | Targets, options, manifest | Partial | Export modules/tests and UI | Artifact-level completion audit and cross-platform packaging proof. |

## Workspace, Commands, Preferences, Security, Accessibility, Performance

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 19 Workspace/tab groups | Tabs, groups, pinning, recents, restore | Partial | `src/App.vue`; store persistence | Document-set grouping and workflow tests. |
| 20 Command palette | Search commands/headings/citations/glossary/index | Partial | `commands` computed in `src/App.vue`; initial Playwright table insertion command test | Browser tests for heading/citation/glossary/index navigation, keyboard shortcuts, and non-sandboxed execution. |
| 21 Preferences | Theme, typography, export, Git, AI, transforms, recents | Partial | Settings UI; persisted workspace | Migration/schema tests and UI workflow. |
| 22 Security/privacy | Local-first, trust-gated executable transforms, no shell | Partial | External transform runner/tests; local file design | Security review, platform proof, threat-model docs. |
| 23 Accessibility | Keyboard, ARIA, contrast, reduced motion | Partial | Some labels/roles/settings in UI | Accessibility audit and automated/manual checks. |
| 24 Performance | Large docs, debounced preview, transform cache, progress | Partial | Debounce/cache/progress code | Benchmarks/stress tests and cancellation behavior. |

## Architecture, Storage, Phases, Acceptance

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 25.1 Frontend architecture | Views/stores/components | Partial | App/store exist; few extracted components | Modularize oversized frontend after workflow tests. |
| 25.1.1 Tauri contract | File ops, compile, export, snapshots, Git, transform commands | Partial | `src-tauri/src/lib.rs`; store invokes commands | Command matrix and UI workflow proof. |
| 25.2 State stores | Documents/workspace/editor/preview/compiler/diagnostics/exports/preferences/versioning/transforms/bibliography/AI | Partial | Mostly centralized in `documents.ts` | Split store by domain after behavior is locked. |
| 25.3 Rust backend | Filesystem, watcher, compiler, transform, export, Git, snapshot, diagnostics, external runner | Partial | Rust modules exist | Continue modularization and command coverage audit. |
| 25.4 IPC commands | Required command list | Partial | Command registrations include required list and more | Produce explicit command coverage table. |
| 26 Data storage | Preferences, recents, brand profiles, transform paths, snapshots, sidecars | Partial | Store plugin usage, snapshot/export manifest modules | Storage docs, migration tests, sidecar behavior proof. |
| 27 Implementation phases | Phase deliverables | Partial | Most phase surfaces exist | Phase completion must be proven by matrix and tests. |
| 28 Acceptance criteria | Concrete app acceptance | Partial | Current build/test surfaces | Fresh baseline plus workflow/export proof required. |
| 29 Non-goals | Cloud collaboration and overreach controls | Partial | Local-first design | Document current non-goals in user docs. |
| 30 Architecture decisions | Licensing, editor, parser, PDF/DOCX/PPTX/citations/formulas/transforms/snapshots/dependency gate | Partial | Implementation follows many decisions; dependency doc exists | Keep dependency admission current and document deviations. |
| 31 First milestone | Prove architecture with representative features | Partial | Current code strongly exceeds first milestone in breadth | Still unverified until fresh baseline and workflow tests pass. |

## External Transform Setup Matrix

| Setup doc section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| Safety model | Real executable paths, per-engine trust, bounded execution, cache keys, fallback | Partial | `transforms/external.rs`; external transform tests | Cross-platform process proof and UI workflow tests. |
| macOS setup | Graphviz, D2, Pikchr, Java/PlantUML paths | Unverified | Documentation exists | Add manual/CI evidence on macOS. |
| Linux setup | Packages and optional engines | Partial | CI installs Linux engines and sets env vars | Keep installed-engine conformance stable. |
| Windows setup | Winget paths and shim guidance | Unverified | Documentation exists | Add manual/CI evidence on Windows. |
| Engine defaults | stdin/file modes by engine | Partial | Adapter profiles and tests | Cross-platform confirmation, especially PlantUML file mode. |
| Troubleshooting | Permission, empty output, timeout, trust disabled, cache stale | Partial | Diagnostics/failure hints | UI docs and platform-specific cases. |

## Verification Coverage Summary

Current direct evidence:

- Backend Rust tests cover many compiler/export/transform/table/file/Git/snapshot
  behaviors under `src-tauri/src/tests/`.
- Frontend unit tests cover table parsing/serialization and conflict diff
  alignment under `tests/frontend-unit.test.ts`.
- CI runs on macOS, Ubuntu, and Windows and includes Rust checks/tests, frontend
  unit tests, frontend build, and Tauri no-bundle compile.

Current major verification gaps:

- Latest pushed CI on commit `c0cefd1` is green across browser workflows and
  Ubuntu/macOS/Windows desktop builds. The earlier Windows path-sensitive
  Rust-test failures, Ubuntu installed Pikchr conformance failure, and Ubuntu
  fake-`d2` stdin fixture failure are resolved in current CI.
- Browser-level workflow harness passes in Linux CI run `26145509141` with 19
  Chromium tests, including advanced table editor coverage, mocked file
  lifecycle coverage, save-as, recently closed reopening, and stale-save
  conflict copy/merge/keep-local/accept-external recovery plus watcher-originated
  clean reload, dirty root-conflict coverage, and AI paste insert/quote/
  appendix/replace-document/section-merge/replace-selection coverage, clean
  included-file recompile, and dirty included-file conflict handling. Local
  focused execution is currently blocked because the workspace-local Chromium
  headless-shell executable is missing from the Playwright cache.
- Local browser workflow discovery now lists 20 Chromium tests after adding
  restart-style workspace restore coverage for open tabs, active tab, pinned
  state, mode/sidebar, workspace root, and recent files; pushed CI proof is
  pending for that slice.
- No desktop WebDriver/Tauri-driver workflow test harness.
- Current committed browser workflow evidence exists, and the desktop CI matrix
  is currently green, but desktop user journeys are still not covered by a
  WebDriver/Tauri-driver harness.
- Export tests rely heavily on package/text assertions; visual/rendered quality
  remains under-proven.
- Optional external transform engines are proven most strongly on Linux; macOS
  and Windows evidence is missing or indirect.
- Accessibility and performance are not proven by dedicated tests or checklists.

## Next Matrix Work

1. Expand each "Partial" row with item-level checklist entries as the
   corresponding feature area is actively worked.
2. Link each row to exact test names once the fresh verification baseline is
   run.
3. Move rows to "Complete" only after direct evidence exists and the evidence is
   current.
4. Keep this matrix synchronized with `docs/todo.md` and `docs/progress.md`.
