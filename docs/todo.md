What remains to achieve the full vision:

  1. Real semantic exporters
     DOCX/PPTX/PDF now map more business semantics, including native DOCX comments/change notes, footnotes with inline references, and DOCX bookmarks/internal
     hyperlinks for cross references, DOCX citation fields with bookmarked bibliography fallbacks, DOCX/PDF/PPTX section header/footer overrides from layout directives, and
     PPTX title, section-divider, two-column, and table-oriented slide layouts. Still needed: deeper page-aware behavior and stronger media/layout fidelity.
  2. Canonical document AST
     There is an AST foundation, but the compiler still has string-heavy paths. Export, diagnostics, source maps, references, transforms, and validation need to depend on
     a durable semantic model.
  3. Full visual table editor
     Core editor controls now cover table parsing/paste/captions, add/remove/duplicate/move rows and columns, alignment, sorting, column formats, totals, formula rows,
     validation, and Markdown preview before apply. Still needed: browser-level interaction tests, richer formula authoring, accessibility polish, and export fixture
     coverage for edited tables.
  4. External transform UX and adapters
     Safety primitives exist: trust, timeouts, size limits, stdin/file modes, caching, diagnostics, setup/security guidance, and adapter profiles for Graphviz, D2,
     PlantUML, and Pikchr. Platform setup guidance and per-engine diagnostic profiles now exist. Still needed: conformance against real installed binaries.
  5. Watcher/conflict hardening
     Native watcher support exists behind native-watch, with fallback now. Watch events are now scoped to the document that installed the watcher, tab activation
     recompiles/resyncs watched paths, conflict actions target the conflicted document instead of whichever tab is currently visible, and backend watch setup discovers nested
     include directives for stronger include graph refresh coverage. Still needed: verified native-watch builds after fsevent-sys downloads and richer merge workflow.
  6. E2E and export conformance tests
     Need Playwright/Tauri workflow tests, DOCX/PPTX/PDF package inspections, fixture comparisons, and confirmed macOS/Windows/Linux CI runs.
  7. Backend modularization
     Chart/native diagram renderers and manifest/media helpers have moved out of lib.rs, but src-tauri/src/lib.rs is still too large. Compiler orchestration, remaining
     transform families, export helpers, diagnostics, filesystem, git, snapshot, and test fixtures should be split further.
