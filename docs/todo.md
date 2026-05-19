What remains to achieve the full vision:

  1. Real semantic exporters
     DOCX/PPTX/PDF now map more business semantics, including native DOCX comments/change notes, footnotes with inline references, and DOCX bookmarks/internal
     hyperlinks for cross references, DOCX citation fields with bookmarked bibliography fallbacks, DOCX/PDF/PPTX section header/footer overrides from layout directives, and
     PPTX title, section-divider, two-column, and table-oriented slide layouts. SVG/PNG/JPEG dimensions now drive DOCX/PPTX image boxes and proportionate PDF
     figure boxes, figure float semantics affect DOCX/PPTX/PDF positioning, fit="cover" plus crop position maps to preview/export metadata and DOCX/PPTX
     crop rectangles, the toolbar/command palette can insert cover figures with crop focus, and the References sidebar can update figure crop focus in-place with
     a pointer/keyboard crop focus pad.
     Layout directives now carry page break, keep-with-next, keep-together, section column, and header/footer semantics through HTML, AST, DOCX, PDF, PPTX,
     and bundle conformance fixtures, and DOCX native TOC fields now honor front matter TOC depth. Still needed: deeper page model controls beyond directive-level pagination.
  2. Canonical document AST
     There is an AST foundation, with normalized layout settings now attached to layout blocks, and transform artifacts now preserve source-file/range provenance
     through the compiler, manifests, bundle exports, frontend types, and transform diagnostics. The compiler still has string-heavy paths. Export, source maps,
     references, transforms, and validation need to depend more fully on a durable semantic model.
  3. Full visual table editor
     Core editor controls now cover table parsing/paste/captions, add/remove/duplicate/move rows and columns, alignment, sorting, column formats, totals, formula rows,
     validation, Markdown preview before apply, target/range formula authoring, and cell/header accessibility labels. Export fixtures now cover broader edited-table
     permutations. Still needed: browser-level interaction tests.
  4. External transform UX and adapters
     Safety primitives exist: trust, timeouts, size limits, stdin/file modes, caching, diagnostics, setup/security guidance, and adapter profiles for Graphviz, D2,
     PlantUML, and Pikchr. Platform setup guidance, per-engine diagnostic profiles, an installed-engine conformance harness, and Linux CI coverage for Graphviz/PlantUML
     real binaries now exist. Still needed: broader Windows/macOS optional-engine evidence and D2/Pikchr CI coverage.
  5. Watcher/conflict hardening
     Native watcher support exists behind native-watch, with fallback now. Watch events are now scoped to the document that installed the watcher, tab activation
     recompiles/resyncs watched paths, conflict actions target the conflicted document instead of whichever tab is currently visible, and backend watch setup discovers nested
     include directives for stronger include graph refresh coverage. The native-watch build has been verified after the optional watcher crates were available. The conflict
     modal now supports line-by-line merge composition from the local/external diff. Still needed: browser-level workflow coverage for conflict resolution.
  6. E2E and export conformance tests
     Need Playwright/Tauri workflow tests, broader fixture comparisons, and confirmed macOS/Windows/Linux CI runs. DOCX sidecar manifests and PPTX slide/media
     package inventories now have direct package-inspection coverage.
  7. Backend modularization
     Chart/native diagram renderers, structured data/API transforms, visual data transforms, business transforms, transform renderer dispatch, compiler support helpers, layout parsing,
     transform fence/source-provenance plumbing, and manifest/media helpers have moved out of lib.rs; export media parsing, sizing, and crop helpers have moved out of export.rs.
     The root Tauri module is now focused on command/runtime wiring, test fixtures live in src-tauri/src/tests.rs, and the export package writers are split into HTML,
     DOCX, PDF, PPTX, and Markdown bundle modules. Still needed: split compiler orchestration, shared export text/helpers, diagnostics, filesystem, git, snapshot,
     and remaining test fixture clusters further as the feature set grows.
