What remains to achieve the full vision:

  1. Real semantic exporters
     DOCX/PPTX/PDF now map more business semantics, including native DOCX comments/change notes, footnotes with inline references, and DOCX bookmarks/internal
     hyperlinks for cross references, DOCX citation fields with bookmarked bibliography fallbacks, DOCX/PDF/PPTX section header/footer overrides from layout directives, and
     PPTX title, section-divider, two-column, and table-oriented slide layouts. SVG/PNG/JPEG dimensions now drive DOCX/PPTX image boxes and proportionate PDF
     figure boxes, figure float semantics affect DOCX/PPTX/PDF positioning, fit="cover" plus crop position maps to preview/export metadata and DOCX/PPTX
     crop rectangles, the toolbar/command palette can insert cover figures with crop focus, and the References sidebar can update figure crop focus in-place with
     a pointer/keyboard crop focus pad.
     Layout directives now carry page break, keep-with-next, keep-together, section column, section page-size/orientation/margin, and header/footer semantics through HTML,
     AST, DOCX, PDF, PPTX, and bundle conformance fixtures, DOCX native TOC fields now honor front matter TOC depth, layout orientation metadata maps to
     HTML/PDF/DOCX page geometry, section-level PDF page controls now produce per-page MediaBox geometry, and PDF section columns now reflow text/table/figure items
     through real column slots instead of only recording column metadata. PDF fixture coverage now proves keep-with-next/keep-together grouping moves controlled blocks as a
     unit, long paragraphs wrap within the available column width with two-line widow/orphan splitting, left/right figure floats reserve side width for adjacent wrapped text,
     large tables continue across section columns and onto following pages, large table continuation is covered inside mixed page-size/orientation sections, tall PDF table cells
     split into continued row fragments instead of overflowing a single flow slot, and table cell span metadata now carries Markdown and imported HTML colspan/rowspan semantics through
     the AST, DOCX gridSpan/vMerge, PPTX table span metadata, PDF cell geometry, and bundle conformance fixtures. Still needed: visual merged-cell editing.
  2. Canonical document AST
     There is an AST foundation, with normalized layout settings now attached to layout blocks, and transform artifacts now preserve source-file/range provenance
     through the compiler, manifests, bundle exports, frontend types, and transform diagnostics. The compiler now emits a source-mapped semantic paged-document model,
     includes it in Markdown bundles, stores compact layout-section summaries in export manifests, and returns it from prepare-for-export so readiness checks and the export UI
     can inspect section layout before files are written. Semantic table/figure/equation inventory now counts AST blocks instead of scanning raw Markdown strings, heading/rich block/table
     scanners, cross-reference handling, and local link validators now preserve fenced-code examples, and layout shortcode tokens now pass through interpolation without false
     missing-variable warnings, so semantic layout directives do not make readiness reports noisy.
     The compiler still has string-heavy paths. Export, references, transforms, and validation need to depend more fully on durable semantic models.
  3. Full visual table editor
     Core editor controls now cover table parsing/paste/captions, add/remove/duplicate/move rows and columns, alignment, sorting, column formats, totals, formula rows,
     validation, Markdown preview before apply, target/range formula authoring, and cell/header accessibility labels. Export fixtures now cover broader edited-table
     permutations, and no-new-dependency frontend unit coverage now locks table parsing/paste/validation/serialization behavior in CI. Still needed: browser-level interaction tests.
  4. External transform UX and adapters
     Safety primitives exist: trust, active document trust prompts, timeouts, size limits, stdin/file modes, caching, diagnostics, setup/security guidance, and adapter
     profiles for Graphviz, D2, PlantUML, and Pikchr. Platform setup guidance, per-engine diagnostic profiles and failure hints, an installed-engine conformance harness,
     and Linux CI coverage for Graphviz, D2, PlantUML, and Pikchr real binaries now exist. Still needed: broader Windows/macOS optional-engine evidence.
  5. Watcher/conflict hardening
     Native watcher support exists behind native-watch, with fallback now. Watch events are now scoped to the document that installed the watcher, tab activation
     recompiles/resyncs watched paths, conflict actions target the conflicted document instead of whichever tab is currently visible, and backend watch setup discovers nested
     include directives for stronger include graph refresh coverage. The native-watch build has been verified after the optional watcher crates were available. The conflict
     modal now supports line-by-line merge composition from the local/external diff, with frontend unit coverage for local/external diff alignment in CI. Still needed: browser-level
     workflow coverage for conflict resolution.
  6. E2E and export conformance tests
     Need Playwright/Tauri workflow tests and broader fixture comparisons. The desktop CI matrix covers macOS/Windows/Linux, and the frontend unit harness now locks that
     matrix plus the frontend unit-test step. DOCX sidecar manifests and PPTX slide/media package inventories now have direct package-inspection coverage.
  7. Backend modularization
     Chart/native diagram renderers, structured data/API transforms, visual data transforms, business transforms, transform renderer dispatch, compiler support helpers, layout parsing,
     transform fence/source-provenance plumbing, and manifest/media helpers have moved out of lib.rs; export media parsing, sizing, and crop helpers have moved out of export.rs.
     The root Tauri module is now focused on command/runtime wiring, compiler orchestration lives in src-tauri/src/compiler.rs, test fixtures live in src-tauri/src/tests.rs,
     and the export package writers are split into HTML, DOCX, PDF, PPTX, and Markdown bundle modules, with shared export text/table/layout helpers isolated in
     export/shared.rs. Filesystem watch/include-graph logic and workspace tree scanning now live outside the low-level file command module, snapshot metadata/storage
     helpers are isolated from snapshot commands, Git IPC types/process helpers now live outside Git command routing, and diagnostic payload types are split from diagnostic
     builder helpers. The transform renderer, external transform, semantic export renderer, export command/readiness, AI cleanup, table/formula export, review/provenance,
     media package, file/Git command, compiler core, bibliography/citation, export/layout option, document structure/include, export conformance, and validation test clusters
     now live in dedicated nested test modules instead of the root backend fixture file. Still needed: continue splitting oversized backend modules and the largest nested test
     modules as the feature set grows.
