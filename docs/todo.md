What remains to achieve the full vision:

  1. Real semantic exporters
     DOCX/PPTX/PDF still need proper semantic mapping for headings, tables, captions, citations, media, comments, provenance, headers/footers, layout sections, and page-
     aware behavior.
  2. Canonical document AST
     There is an AST foundation, but the compiler still has string-heavy paths. Export, diagnostics, source maps, references, transforms, and validation need to depend on
     a durable semantic model.
  3. Full visual table editor
     Core editor controls now cover table parsing/paste/captions, add/remove/duplicate/move rows and columns, alignment, sorting, column formats, totals, formula rows,
     validation, and Markdown preview before apply. Still needed: browser-level interaction tests, richer formula authoring, accessibility polish, and export fixture
     coverage for edited tables.
  4. External transform UX and adapters
     Safety primitives exist: trust, timeouts, size limits, stdin/file modes, caching, diagnostics, setup/security guidance, and adapter profiles for Graphviz, D2,
     PlantUML, and Pikchr. Still needed: conformance against real installed binaries, richer per-engine diagnostics, and packaged setup docs for each platform.
  5. Watcher/conflict hardening
     Native watcher support exists behind native-watch, with fallback now. Still needed: verified native-watch builds after fsevent-sys downloads, richer merge workflow,
     multi-document watcher behavior, and stronger include graph refresh coverage.
  6. E2E and export conformance tests
     Need Playwright/Tauri workflow tests, DOCX/PPTX/PDF package inspections, fixture comparisons, and confirmed macOS/Windows/Linux CI runs.
  7. Backend modularization
     Chart and native diagram renderers have moved into transform modules, but src-tauri/src/lib.rs is still too large. Compiler orchestration, remaining transform
     families, export helpers, diagnostics, filesystem, git, snapshot, and test fixtures should be split further.
