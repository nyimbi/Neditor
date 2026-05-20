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

GitHub Actions is not an active verification surface for this project. Any
older run IDs in this matrix are historical context only; current completion
requires local command evidence, committed artifacts, rendered/manual QA, or
explicit platform checks.

## Core Product Scope

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 1 Purpose | Local-first business Markdown workbench | Partial | `README.md`; Tauri/Vue/Rust project layout; compiler/export modules | Full buildout still needs requirement-by-requirement verification and workflow tests. |
| 2 Source Prompt Extension | Tauri 2, Vue 3, Pinia, Vite, vanilla CSS | Complete | `package.json`; `src/main.ts`; `src/App.vue`; `src-tauri/Cargo.toml` | Keep dependency admission docs current as test dependencies are added. |
| 2 Source Prompt Extension | Side-by-side Markdown editor and live preview | Partial | `src/App.vue` editor/preview panes; CodeMirror setup; Playwright harness covers source edit to live preview update; archived browser workflow run `26153224371` covers synchronized editor/preview scrolling and preview heading click-to-source | Desktop execution and large-document debounce timing proof. |
| 2 Source Prompt Extension | Split, preview-only, focus/source modes | Partial | `src/App.vue` mode controls; `e2e/app-workflows.spec.ts` covers split/source/preview/focus/export/review/presentation modes with sidebar routing for export/review/presentation; archived browser workflow evidence passed on commit `420af08` | Desktop execution still needs proof. |
| 2 Source Prompt Extension | Local new/open/save/save-as flows | Partial | `src/stores/documents.ts`; `src-tauri/src/filesystem.rs`; file command tests | Browser/desktop workflow tests for dialogs and state transitions. |
| 2 Source Prompt Extension | Window title reflects file and dirty state | Partial | `windowTitle` getter and `setWindowTitle` in `src/App.vue`; Playwright harness asserts clean, dirty, and saved browser document titles | Native desktop title assertion. |
| 2 Source Prompt Extension | Toolbar commands and keyboard shortcuts | Partial | `src/App.vue` toolbar, command palette, shortcut handler; archived browser workflow run `26154535588` covers CodeMirror search keybinding and command-palette heading search/navigation; archived browser workflow run `26155535210` covers command-palette citation, glossary, and index navigation; archived browser workflow run `26156393184` covers open-document switching and workspace-file command navigation | Broader shortcut coverage and native desktop command execution proof. |
| 2 Source Prompt Extension | Light/dark/system theme and typography | Partial | Settings in `src/App.vue`; persisted preferences in store; Playwright harness asserts theme, preview theme, high contrast, reduced motion, and editor/preview typography application plus reload persistence | Visual/accessibility verification across themes in a real browser/native runtime. |
| 2 Source Prompt Extension | Cross-platform packaging | Partial | `README.md` packaging note; local Tauri build commands | Full local/manual bundle evidence for Windows/Linux and refreshed macOS bundle/DMG status. |
| 3 Product Positioning | Local-first document workbench, not cloud suite | Partial | Local file/snapshot/Git/export design | User documentation and fixture workflows need to prove positioning. |
| 4 Target Users | Business, technical, research, product users | Complete | Example project front matter declares consultants, technical writers, researchers and analysts, product and engineering teams, executives and managers, students and academics, developers, and teams using AI chat output; `example_fixture_tests::example_project_fixtures_compile_and_export` verifies each persona compiles and exports across supported targets | Keep new personas represented by executable example fixtures. |

## Carry-Forward MacDown Improvements

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 5.1 External File Refresh | Watch open files | Partial | `src-tauri/src/filesystem_watch.rs`; `src/stores/documents.ts`; native-watch local check; stale-save conflict recovery is browser-proven in archived workflow run `26139678118`; clean root-file watcher reload is browser-proven in archived workflow run `26140882880`; clean included-file recompile and dirty included-file conflict flows are browser-proven in archived workflow run `26145509141` | Desktop workflow tests. |
| 5.1 External File Refresh | Non-destructive conflicts: compare, accept external, keep local, save copy | Partial | Conflict modal in `src/App.vue`; `src/lib/conflict.ts`; frontend unit diff test; archived browser workflow run `26139678118` covers stale-save compare visibility, local-copy preservation, merge-back recovery, keep-local, and accept-external; archived browser workflow run `26140882880` covers watcher-originated dirty root-file conflict compare visibility | Included-file conflict resolution tests and more granular line-compose controls. |
| 5.1 External File Refresh | Watch included files and recompile master docs | Partial | Watch setup includes manifest included files; include tests; archived browser workflow run `26145509141` covers clean included-file recompile and dirty included-file conflict workflows | Deeper include graph navigation edits. |
| 5.2 Build Toolchain Lessons | Pinned local JS/Rust dependencies | Partial | `pnpm-lock.yaml`; `src-tauri/Cargo.lock`; README commands | Confirm no hidden global dependency remains in packaging and optional engines. |
| 5.2 Build Toolchain Lessons | Diagnostics for missing external engines | Partial | `transforms/external.rs`; external transform tests | Cross-platform diagnostic proof and UI workflow coverage. |
| 5.3 Business Export Customization | HTML/PDF/DOCX/PPTX exports | Partial | `src-tauri/src/export/*`; export tests; archived browser workflow run `26159396761` covers export invocation success/failure, output and manifest path UI reporting, and readiness-blocked export before write | Artifact-level conformance, target-specific option matrices, and rendered/manual checks for each target. |
| 5.3 Business Export Customization | Page numbering, headers/footers, logo, brand color, cover, watermark, presets, metadata | Partial | Export option tests; export shared helpers; frontend settings | Completion matrix rows for each option and visual artifact proof. |
| 5.4 Master Documents And Includes | Include syntaxes, relative resolution, strip child front matter, circular diagnostics | Partial | Compiler/document structure tests | Add completion evidence for max depth, invalid include files, navigation, and UI include graph. |
| 5.5 Table Of Contents | `[TOC]`, front matter TOC, depth, numbering, export formatting | Partial | Generated sections and export tests; Playwright harness covers command-palette insertion of `[TOC]` | Page-numbered TOC proof for PDF/DOCX where supported; browser preview tests. |
| 5.6 Native Tab Organization | Multiple docs, tabs, pinning, recents, restore | Partial | `src/App.vue`; `src/stores/documents.ts`; Playwright harness covers document-set grouping, folder grouping, dragging a document into a set, closing a group, tab activation, dirty-close confirmation, stale recent cleanup, workspace restore, scroll restore, and missing restored-file warnings | Native desktop proof and deeper drag/reorder edge cases. |
| 5.6 Native Tab Organization | Split editor panes later | Deferred | Spec says "later versions" | Record as deferred/non-goal for current release. |

## Core Application Requirements

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 6.1 Application Shell | Tauri IPC commands exposed from Rust | Complete | `src-tauri/src/lib.rs`; `docs/ipc-command-coverage.md`; `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` verifies every spec 25.4 command is registered and documented | Keep coverage synchronized as new required IPC commands are added. |
| 6.1 Application Shell | Vue SFC block order template/script/style | Partial | `src/App.vue` follows order | Preserve when modularizing; add lint/check if practical. |
| 6.2 Primary Layout | Toolbar, status bar, sidebar, editor, preview | Partial | `src/App.vue`; Playwright harness covers outline-sidebar heading navigation to source | Browser layout/accessibility checks across viewport sizes and native execution proof. |
| 6.2 Primary Layout | Export preview, review, presentation outline modes | Partial | Mode controls and sidebars in `src/App.vue`; Playwright harness covers export preview, review, and presentation outline routing | Local browser/native execution proof and product QA for each mode. |
| 6.3 Editor | CodeMirror 6 engine | Complete | CodeMirror dependencies and imports in `src/App.vue` | None beyond ongoing behavior tests. |
| 6.3 Editor | Markdown highlighting, diagnostics gutter, decorations | Partial | CodeMirror markdown/linter/lintGutter/decorations; Playwright harness covers diagnostics-panel source navigation with precise line/column metadata | Local browser/native execution proof for diagnostic range rendering and visual states. |
| 6.3 Editor | Line numbers, word wrap, spellcheck, find/replace, word count | Partial | Settings and CodeMirror extensions; status bar stats; Playwright harness covers spellcheck/autocapitalize editor attributes and word/character/reading-time status metrics; archived browser workflow run `26154535588` covers line-number and word-wrap persistence plus CodeMirror find/replace | Local browser/native execution proof for status metrics and spellcheck behavior. |
| 6.3 Editor | Smart list continuation and auto-pairing | Partial | `continueMarkdownList`; `closeBrackets`; Markdown format commands in `src/App.vue`; Playwright harness covers list continuation, bracket pairing, quote pairing, bold/italic/inline-code pairing, and code-fence insertion; archived browser workflow run `26154535588` covers basic Markdown list continuation and bracket auto-pairing | Local browser/native execution proof and deeper Markdown shortcut edge cases. |
| 6.3 Editor | Multi-cursor support | Partial | Explicit CodeMirror `addCursorAbove`, `addCursorBelow`, and `selectNextOccurrence` commands in `src/App.vue`; Playwright harness includes a multi-cursor edit workflow | Local browser/native execution proof. |
| 6.3 Editor | Vim/emacs keybindings | Deferred | Spec says later if feasible | Record deferred unless implemented later. |
| 6.4 Preview | Live debounced preview | Partial | Store compile on editor update; debounce in `src/App.vue`; Playwright harness covers preview update after source typing | Large-document behavior and timing tests. |
| 6.4 Preview | Scroll sync and heading click-to-source | Partial | Preview/editor scroll handlers; click handler; archived browser workflow run `26153224371` covers editor-to-preview scroll sync, preview-to-editor scroll sync, and rendered preview heading click-through to the source line | Desktop workflow proof and edge cases for non-heading anchors. |
| 6.4 Preview | Separate preview theme, inline warnings, transform blocks, export preview | Partial | Preview theme setting; diagnostics; transform rendering; modes; Playwright harness asserts preview theme and typography attributes | Inline warning rendering, transform-aware preview visual tests, and export preview proof. |
| 6.5 File Operations | New, open file, open folder, save, save as, revert, rename, duplicate, reveal | Partial | Store actions and Rust file commands; file command tests; Playwright mocked workflows cover open/save/save-as/duplicate/rename/pin/reveal/revert in archived workflow run `26137556147`; stale-save conflict copy/merge/keep-local/accept-external recovery in archived workflow run `26139678118`; watcher-originated root reload/conflict proof in archived workflow run `26140882880`; included-file watcher conflict proof in archived workflow run `26145509141` | Native desktop dialog workflow tests. |
| 6.5 File Operations | Recent docs/folders, workspace restore | Partial | Persisted workspace store; Playwright mocked workflows cover workspace listing after open and recently closed reopening in archived workflow run `26137556147`; archived browser workflow run `26147556750` covers restart-style restore for open tabs, active tab, pinned state, mode/sidebar, workspace root, and recent files; archived browser workflow run `26148828614` covers scroll restore and missing-restored-file warning coverage; archived browser workflow run `26151184228` covers tab activation, dirty close confirmation, renamed recent cleanup, and deleted recently-closed pruning coverage; archived browser workflow run `26152255407` covers recent folder reopen/prune behavior and externally moved recently-closed path pruning | Native desktop dialog/recent-folder proof and deeper folder/document-set grouping behavior. |
| 6.5 File Operations | External change detection/conflict handling | Partial | Watch/conflict code and tests; archived browser workflow run `26139678118` covers stale-save conflict blocking, compare, save-copy preservation, merge-back recovery, keep-local, and accept-external; archived browser workflow run `26140882880` covers clean root reload and watcher-originated dirty root conflict; archived browser workflow run `26145509141` covers clean included-file recompile and dirty included-file conflict flows | Desktop workflow proof. |

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
| 9.2 Export snapshots | Manifest with hashes/options/app/version/status/timestamp | Partial | `manifest.rs`; `export_commands.rs`; export command tests; archived browser workflow run `26159396761` covers manifest path reporting and target-specific readiness manifest preview; manifests now include an explicit readiness summary with ready/error/warning/info counts and direct-export dirty-Git readiness warnings | Confirm all export targets write/consume manifests consistently. |
| 9.3 Release workflow | Status values, badge, draft export warning, approval metadata, release tagging | Partial | Review UI; validation; Git tag command | Visual badge proof and workflow tests. |
| 9.4 AI paste cleanup | Normalize chat output, code fences, bullets, tables, links, citations, insert modes | Partial | `ai_cleanup.rs`; AI modal; AI cleanup tests; shared insertion helper tests; Playwright coverage for insert, quote, appendix, replace document, merge into section, replace selection, citation TODO preview/insertion, draft marking, and provenance toggle workflows | Rich clipboard behavior and non-sandboxed execution. |
| 9.5 AI provenance | `ai-source`, AI-assisted sections, export appendix | Partial | `provenance.rs`; review/provenance tests; UI toggles; Playwright AI paste governance workflow covers `ai-source` insertion and draft review markers; `prepare_for_export_reports_ai_provenance_audit_metadata` and `prepare_for_export_reports_invalid_ai_review_statuses` prove readiness/manifest diagnostics for missing provenance metadata and invalid AI review statuses | Appendix workflow coverage and native workflow execution. |
| 9.6 Includes | Include graph, diagnostics, re-render, export single doc | Partial | Document structure tests; compiler support | UI include graph/navigation and watcher workflow proof. |
| 9.7 Business table editor | Visual editor, rows/cols, alignment, paste, sort, formats, readable Markdown, export | Partial | `src/lib/tables.ts`; table UI; frontend/backend tests; Playwright coverage for table insertion, pasted table import, numeric sorting, formula rows, merged-cell metadata, row/column add-remove behavior, column format totals, cancel-without-applying behavior, and apply-back-to-editor behavior | Remaining non-sandboxed browser execution and more export fixtures. |
| 9.8 Calculations | Calc blocks, inline formulas, table formulas, named values/tables, dependency graph | Partial | `calculations.rs`; table tests; compiler tests | Named table/range coverage, circular dependency behavior, UI proof. |
| 9.9 Equations | Inline/display math, numbering, references, export support | Partial | `rich_blocks.rs`; export tests; `prepare_for_export_reports_missing_caption_labels` proves readiness/manifest diagnostics for equations missing stable labels or captions | Cross-target artifact proof and editor UX tests. |
| 9.10 TOC | Automatic TOC with marker/front matter | Partial | `generated_sections.rs`; document/export tests | Page-numbered target proof and UI controls. |
| 9.11 Index | Automatic index and exclusions | Partial | `indexing.rs`; compiler tests; Playwright harness covers command-palette insertion of `[INDEX]` | Export proof for index options and native workflow execution. |
| 9.12 Bibliography | BibTeX/CSL JSON, citation syntax, rendered bibliography | Partial | `bibliography.rs`; citation tests; Playwright harness covers command-palette insertion of `[BIBLIOGRAPHY]`, resolved reference display, missing-key UI, and duplicate-key UI | CSL style fidelity, richer manager workflows, and native workflow execution. |
| 9.13 Cross references | Figures/tables/equations/headings, broken ref diagnostics, export links | Partial | `references.rs`; citation/export tests; readiness tests now warn when figure/table/equation targets lack stable labels before export | Full cross-target proof. |
| 9.14 Captions | Figures/table captions, numbering, list support | Partial | AST/export/table/figure tests; `compiler_generates_lists_of_figures_and_tables` covers `[LIST_OF_FIGURES]`, `[LIST_OF_TABLES]`, caption numbering, anchor links, fenced-example exclusion, preview HTML, and DOCX artifact text; Playwright harness covers command-palette insertion of both caption-list markers; `prepare_for_export_reports_missing_caption_labels` proves readiness/manifest diagnostics for missing figure/table/equation labels or captions | Broader cross-target rendered proof and native workflow execution. |
| 9.15 Advanced layout | Page size/orientation/margins/columns/breaks/headers/footers/keeps/floats | Partial | `layout.rs`; `paged_document.rs`; export tests | Rendered/manual QA and overflow hardening. |
| 9.16 Brand templates | Brand name/color/logo/font/header/footer/watermark/legal disclaimer | Partial | Settings UI; export option tests | Template/profile persistence and cross-target visual proof. |
| 9.17 Review comments/change notes | Comments, unresolved validation, exports | Partial | `review.rs`; review UI; export conformance tests; `prepare_for_export_reports_review_change_note_audit_metadata` proves readiness/manifest diagnostics for missing review and change-note audit metadata | UI workflow tests and native target fidelity. |
| 9.18 Document variables | Front matter/project/data variables | Partial | `variables.rs`; project variable tests | Filter coverage and docs. |
| 9.19 Pikchr diagrams | Native fallback/external setup and diagnostics | Partial | `transforms/diagram.rs`; `external.rs`; transform tests | Cross-platform optional engine proof. |
| 9.20 Validation | One-click prepare report across metadata/includes/citations/formulas/figures/transforms/settings/links/comments | Partial | `validation.rs`; `export_commands.rs`; readiness UI; archived browser workflow run `26159396761` covers target-specific readiness diagnostics and blocked export behavior; Rust readiness tests cover malformed review/comment, AI provenance audit metadata, missing figure/table/equation caption labels, explicit manifest readiness summaries, and dirty-Git warnings in direct export manifests | Full readiness completeness audit and non-sandboxed/native execution. |

## Fenced-Code Transform System

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 10.1 Architecture | Registry, option validation, artifact cache, diagnostics | Partial | `transforms/renderer.rs`; `transforms/options.rs`; `transforms/external.rs` | More option validation coverage and structured artifact use in exports. |
| 10.2 Safety | No network, no shell, trust, timeout, output limits, source hash cache | Partial | External transform runner and tests | Cross-platform process behavior and cache identity hardening. |
| 10.3 Core transforms | `calc`, `mermaid`, `pikchr` | Partial | Renderer and transform tests | Fidelity limitations documented; optional engine proof for Pikchr. |
| 10.4.1 DOT/Graphviz | SVG diagrams and engines | Partial | Native fallback; external adapter; Linux installed-engine conformance | Engine variants and Windows/macOS proof. |
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
| 19 Workspace/tab groups | Tabs, groups, pinning, recents, restore | Partial | `src/App.vue`; store persistence; Playwright harness covers folder/document-set tab groups, drag-to-document-set front matter updates, close-group behavior, pinning, recents, and restore | Native desktop proof, drag/reorder edge cases, and saved workspace/package evidence. |
| 20 Command palette | Search commands/headings/citations/glossary/index/documents/workspace files | Partial | `commands` computed in `src/App.vue`; Playwright table insertion command test; archived browser workflow run `26154535588` covers heading command navigation; archived browser workflow run `26155535210` covers citation, glossary, and index command navigation; archived browser workflow run `26156393184` covers open-document switching and workspace-file opening | Broader keyboard shortcuts and non-sandboxed/native execution. |
| 21 Preferences | Theme, typography, export, Git, AI, transforms, recents | Partial | Settings UI; persisted workspace; archived browser workflow run `26157446711` covers external transform path changes, input mode, timeout, trust prompt, denied trust reset, successful probe UI, and missing-executable diagnostics | Migration/schema tests and broader settings workflow coverage. |
| 22 Security/privacy | Local-first, trust-gated executable transforms, no shell | Partial | External transform runner/tests; local file design; archived browser workflow run `26157446711` covers transform trust prompts, path-change trust clearing, denied trust reset, and missing executable diagnostics | Security review, platform proof, threat-model docs. |
| 23 Accessibility | Keyboard, ARIA, contrast, reduced motion | Partial | Some labels/roles/settings in UI; `pnpm run check:a11y` statically checks Vue template button names, form-control labels, and dialog labeling; modal close buttons, command-palette search, and conflict merge-line controls now have explicit labels | Broader automated checks, keyboard-only/manual audit, and native workflow proof. |
| 24 Performance | Large docs, debounced preview, transform cache, progress | Partial | Debounce/cache/progress code; `performance_tests::compiler_stress_handles_large_documents_with_many_artifacts` stress-compiles nested includes, many tables, formulas, transform artifacts, source maps, and diagnostics; `performance_tests::repeated_export_loop_keeps_large_artifacts_stable` repeatedly renders a large compiled document across HTML, PDF, DOCX, PPTX, and Markdown bundle outputs with structural and artifact-size assertions; `performance_tests::repeated_editing_sessions_reuse_external_transform_cache` repeatedly recompiles edited document text while proving a stable trusted external DOT transform is served from cache after the first run; `export_command_tests::export_readiness_and_manifest_report_progress_steps` proves readiness/export manifests carry compile, transform, readiness, render, and manifest progress steps; the export UI now displays readiness and last-export progress steps | Memory growth checks, UI debounce timing, cancellation behavior, and native/browser performance workflow proof. |

## Architecture, Storage, Phases, Acceptance

| Spec section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| 25.1 Frontend architecture | Views/stores/components | Partial | App/store exist; few extracted components | Modularize oversized frontend after workflow tests. |
| 25.1.1 Tauri contract | File ops, compile, export, snapshots, Git, transform commands | Partial | `src-tauri/src/lib.rs`; `docs/ipc-command-coverage.md`; store invokes commands; `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` covers initial spec IPC registration | Broader UI workflow proof for every extended command path. |
| 25.2 State stores | Documents/workspace/editor/preview/compiler/diagnostics/exports/preferences/versioning/transforms/bibliography/AI | Partial | Mostly centralized in `documents.ts` | Split store by domain after behavior is locked. |
| 25.3 Rust backend | Filesystem, watcher, compiler, transform, export, Git, snapshot, diagnostics, external runner | Partial | Rust modules exist; initial IPC command coverage audit is executable via `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` | Continue modularization and deeper command behavior coverage. |
| 25.4 IPC commands | Required command list | Complete | `docs/ipc-command-coverage.md`; `src-tauri/src/lib.rs`; `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented` parses the spec, coverage table, and Tauri handler registration | Keep the table and guardrail test current if the spec command list changes. |
| 26 Data storage | Preferences, recents, brand profiles, transform paths, snapshots, sidecars | Partial | Store plugin usage, snapshot/export manifest modules | Storage docs, migration tests, sidecar behavior proof. |
| 27 Implementation phases | Phase deliverables | Partial | Most phase surfaces exist | Phase completion must be proven by matrix and tests. |
| 28 Acceptance criteria | Concrete app acceptance | Partial | Current build/test surfaces | Fresh baseline plus workflow/export proof required. |
| 29 Non-goals | Cloud collaboration and overreach controls | Partial | Local-first design | Document current non-goals in user docs. |
| 30 Architecture decisions | Licensing, editor, parser, PDF/DOCX/PPTX/citations/formulas/transforms/snapshots/dependency gate | Partial | `LICENSE`; `package.json`; `src-tauri/Cargo.toml`; implementation follows many decisions; dependency doc exists | Keep dependency admission current and document deviations. |
| 31 First milestone | Prove architecture with representative features | Partial | Current code strongly exceeds first milestone in breadth | Still unverified until fresh baseline and workflow tests pass. |

## External Transform Setup Matrix

| Setup doc section | Requirement area | Current status | Evidence | Remaining gap |
| --- | --- | --- | --- | --- |
| Safety model | Real executable paths, per-engine trust, bounded execution, cache keys, fallback | Partial | `transforms/external.rs`; external transform tests; archived browser workflow run `26157446711` covers settings-level path change trust clearing, trust prompts, denied trust reset, input mode/timeout probe details, cache identity display, and missing-executable diagnostics | Cross-platform process proof and deeper executable edge cases. |
| macOS setup | Graphviz, D2, Pikchr, Java/PlantUML paths | Unverified | Documentation exists | Add manual evidence on macOS. |
| Linux setup | Packages and optional engines | Partial | Historical workflow installed Linux engines; current proof requires local installed-engine checks | Keep installed-engine conformance stable. |
| Windows setup | Winget paths and shim guidance | Unverified | Documentation exists | Add manual evidence on Windows. |
| Engine defaults | stdin/file modes by engine | Partial | Adapter profiles and tests | Cross-platform confirmation, especially PlantUML file mode. |
| Troubleshooting | Permission, empty output, timeout, trust disabled, cache stale | Partial | Diagnostics/failure hints | UI docs and platform-specific cases. |

## Verification Coverage Summary

Current direct evidence:

- Backend Rust tests cover many compiler/export/transform/table/file/Git/snapshot
  behaviors under `src-tauri/src/tests/`.
- Backend Rust stress coverage now includes a large-document compiler stress
  case for nested includes, many tables, formulas, transform artifacts, source
  maps, and diagnostics.
- Frontend unit tests cover table parsing/serialization and conflict diff
  alignment under `tests/frontend-unit.test.ts`.
- Local verification covers Rust checks/tests, frontend unit tests, frontend
  build, and Tauri no-bundle compile when run on the current host.
- `pnpm run check:docs` now checks README plus the docs set for missing local
  links, including the `docs/specification.md` architecture figure target.
- `pnpm run check:a11y` now checks static Vue template accessibility guardrails
  for accessible control names and dialog labeling.

Current major verification gaps:

- Archived remote workflow evidence on commit `02e832a` was green across browser workflows and
  Ubuntu/macOS/Windows desktop builds. The earlier Windows path-sensitive
  Rust-test failures, Ubuntu installed Pikchr conformance failure, and Ubuntu
  fake-`d2` stdin fixture failure were resolved there. This is historical
  evidence only, not the current verification source.
- Browser-level workflow harness previously passed in Linux Actions run `26159396761` with 28
  Chromium tests, including advanced table editor coverage, mocked file
  lifecycle coverage, save-as, recently closed reopening, and stale-save
  conflict copy/merge/keep-local/accept-external recovery plus watcher-originated
  clean reload, dirty root-conflict coverage, and AI paste insert/quote/
  appendix/replace-document/section-merge/replace-selection coverage, clean
  included-file recompile, dirty included-file conflict handling, and
  restart-style workspace restore coverage for open tabs, active tab, pinned
  state, mode/sidebar, workspace root, recent files, scroll-position restore,
  missing-restored-file warning coverage, tab activation, dirty close
  confirmation, renamed recent cleanup, deleted recently-closed pruning, recent
  folder reopen/prune behavior, and externally moved recently-closed path
  pruning, synchronized editor/preview scrolling, preview heading
  click-to-source, persisted editor word-wrap and line-number settings,
  CodeMirror find/replace, smart list continuation, bracket auto-pairing, and
  command-palette heading, citation, glossary, index, open-document, and
  workspace-file navigation, plus transform engine settings trust/probe
  diagnostics, target-specific export readiness manifest preview, export
  output/manifest path reporting, export success/failure diagnostics, and
  blocked export diagnostics before file write.
  Local focused execution is currently blocked because Playwright browser
  installation in the sandbox fails with `EPERM` while creating
  `/Users/nyimbiodero/Library/Caches/ms-playwright/__dirlock`.
- No desktop WebDriver/Tauri-driver workflow test harness.
- Current committed browser workflow tests exist, but local browser execution
  depends on Playwright browser installation and host permissions; desktop user
  journeys are still not covered by a WebDriver/Tauri-driver harness.
- Export tests rely heavily on package/text assertions; visual/rendered quality
  remains under-proven.
- Optional external transform engines are proven most strongly on Linux; macOS
  and Windows evidence is missing or indirect.
- Accessibility has an initial static guard, but full keyboard/manual audit
  evidence is still missing. Performance now has compiler, repeated export
  loop, repeated edit/cache, and export progress reporting coverage, but memory
  growth, UI debounce timing, cancellation behavior, and native/browser
  performance workflow proof remain under-proven.

## Next Matrix Work

1. Expand each "Partial" row with item-level checklist entries as the
   corresponding feature area is actively worked.
2. Link each row to exact test names once the fresh verification baseline is
   run.
3. Move rows to "Complete" only after direct evidence exists and the evidence is
   current.
4. Keep this matrix synchronized with `docs/todo.md` and `docs/progress.md`.
