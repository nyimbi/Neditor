# NEditor

NEditor is a local-first Markdown workbench for serious business documents:
board papers, consulting reports, technical architecture notes, research
briefs, proposals, export packs, and AI-assisted drafts that still need human
governance.

It keeps the speed and readability of Markdown, then adds the things business
documents usually force into office suites: structured metadata, includes,
tables, calculations, citations, diagrams, review comments, release status,
brand-aware exports, and reproducible export manifests. Your source stays local
and inspectable. The app helps turn it into polished HTML, PDF, DOCX, PPTX, or
Markdown bundle outputs.

![NEditor workbench showing Markdown source, live preview, outline, diagnostics, and status](docs/screenshots/workbench.svg)

## Why NEditor

Most Markdown editors are great for notes and weak for deliverables. Most office
suites are strong at formatting but poor at repeatability, review evidence, and
version control. NEditor is built for the middle ground: documents that need to
be written quickly, reviewed carefully, exported consistently, and audited later.

- **Local-first by design:** files, workspace state, snapshots, transform
  settings, and export outputs stay on your machine.
- **Markdown as source of truth:** readable text remains the canonical document,
  while the Rust compiler builds the semantic model used by preview and export.
- **Business-document aware:** front matter, release status, approval metadata,
  comments, citations, glossary terms, index terms, variables, formulas, and
  layout directives are first-class concepts.
- **Repeatable exports:** every export can carry a manifest with source hashes,
  include hashes, options, diagnostics, app version, output path, and output
  hash evidence.
- **Safe extensibility:** diagram and data transforms use native renderers where
  possible, and external engines are trust-gated with timeouts, output limits,
  diagnostics, and deterministic cache keys.

## Screenshots

| Writing workbench | Export readiness |
| --- | --- |
| ![Split editor and preview with outline and diagnostics](docs/screenshots/workbench.svg) | ![Export readiness panel with target options, diagnostics, and manifest preview](docs/screenshots/export-readiness.svg) |

| Review and AI governance | Tables and transforms |
| --- | --- |
| ![Review panel with release status, unresolved comments, AI provenance, and change notes](docs/screenshots/review-governance.svg) | ![Business table editor and transform-aware document preview](docs/screenshots/tables-transforms.svg) |

## Feature Tour

### Native Document Workbench

- Tauri 2 desktop shell with Vue 3, Pinia, Vite, CodeMirror 6, vanilla CSS, and
  a Rust command backend.
- Split source and preview, source-only, preview-only, focus, review,
  presentation, and export modes.
- Tabs, pinned documents, recent files, recently closed documents, recent
  folders, workspace folder browsing, and restart-style workspace restore.
- CodeMirror editing with Markdown highlighting, diagnostics gutter, line
  numbers, word wrap, find/replace, smart list continuation, bracket pairing,
  word count, character count, and reading-time status.
- Command palette for app actions plus document-aware navigation across
  headings, citations, glossary terms, index terms, open documents, and
  workspace files.
- Separate preferences for theme, preview theme, typography, editor behavior,
  autosave, snapshots, export defaults, bibliography defaults, brand defaults,
  Git integration, AI cleanup, and transform engines.

### Compiler And Document Intelligence

- YAML front matter for title, subtitle, author, client, date, classification,
  status, version, release metadata, brand settings, layout settings, TOC,
  citation style, variables, and export controls.
- Include expansion for modular master documents, with relative resolution,
  child front-matter stripping, include graph tracking, source/include hashing,
  and diagnostics for missing or circular includes.
- Document variables, default variables, project variables, inline formulas,
  calculation blocks, table formulas, and dependency diagnostics.
- Automatic table of contents, glossary, index, bibliography, source maps,
  export manifests, semantic AST, paged-document model, diagnostics, and
  transform artifacts.
- Cross references for headings, figures, tables, equations, appendices, and
  decision records, with broken-reference diagnostics.
- Rich document blocks for callouts, equations, figures, captions, layout
  sections, review comments, change notes, AI provenance, and approval metadata.

### Tables, Data, And Calculations

- Visual Markdown table editor with paste import, row and column editing,
  alignment, numeric sorting, column formats, totals, formula rows, merged-cell
  metadata, cancel/apply workflows, and readable Markdown output.
- Markdown tables, CSV fences, and TSV fences can carry formulas such as
  `=10+15`, `=SUM(2,3)`, and named table references.
- Data transforms for CSV, TSV, JSON, YAML, OpenAPI, and JSON Schema produce
  preview/export-safe artifacts.

### Diagrams And Transform Blocks

NEditor includes a fenced-code transform registry for static, export-safe
artifacts:

- `calc` for document calculations.
- `chart` for bar, line, pie, area, and KPI charts.
- `mermaid`, `pikchr`, DOT/Graphviz, D2, and PlantUML for diagrams.
- `vega-lite`, GeoJSON, TopoJSON, STL, `timeline`, `glossary`, `bibtex`,
  OpenAPI, JSON Schema, JSON, YAML, CSV, and TSV.
- Native Rust renderers or fallbacks are used where practical.
- External executable engines require explicit trust, bounded execution, and
  clear setup/probe diagnostics.

### Review, Release, And AI Governance

- Release statuses such as draft, in-review, approved, published, and archived.
- Approval metadata validation before release-grade exports.
- Inline review comments, unresolved-comment validation, change notes, and
  export appendix options.
- AI paste cleanup for chat output, code fences, bullets, tables, links,
  citation TODOs, insertion modes, and provenance markers.
- AI-source and AI-assisted section tracking so generated material can be
  marked as needing review or human-reviewed before export.

### Local Files, Safety, And Versioning

- New, open, open folder, save, save as, revert, rename, duplicate, reveal,
  recent documents, recent folders, and workspace restore flows.
- On-disk hash guards prevent stale saves from silently overwriting external
  edits.
- File watchers track the root document and included files. Clean documents
  reload or recompile; dirty documents open a conflict workflow with compare,
  accept external, keep local, and save-copy actions.
- Git integration for repository detection, branch/dirty state, commit, history,
  diff, restore, tag, and dirty-export warnings.
- Git-free snapshots for non-Git users, including manual, automatic, and
  pre-export snapshot paths.

### Export System

NEditor exports from the compiled semantic document model rather than treating
preview HTML as the only source of truth.

| Target | What it is for | Current support |
| --- | --- | --- |
| HTML | Web previews, review copies, static publishing | Styles, syntax highlighting, semantic sections, transform artifacts, manifests |
| PDF | Board papers, reports, proposals, release packs | Paged layout model, page options, headers/footers, watermarks, tables, figures |
| DOCX | Word-compatible client deliverables | Document structure, paragraphs, tables, comments/provenance appendices, manifests |
| PPTX | Presentation-style summaries and executive decks | Slide packaging, agenda option, table splitting, manifest sidecars |
| Markdown bundle | Portable source handoff | Source document plus manifest and packaged evidence |

Export defaults include manifests, styles, syntax highlighting, cover page, page
numbers, layout preset, comments appendix, AI provenance appendix, glossary
appendix, PPTX agenda, citation style, brand profile, dirty-Git warnings,
transform engine settings, and draft watermark behavior.

## Example Projects

The repository ships example documents that compile and export through the
supported targets:

- [Board paper](examples/board-paper.md)
- [Consulting report with includes](examples/consulting-report.md)
- [Technical architecture document](examples/technical-architecture.md)
- [Research report with bibliography](examples/research-report.md)
- [Proposal with budget tables and formulas](examples/proposal-budget.md)
- [AI-assisted draft with review provenance](examples/ai-assisted-draft.md)

These examples are covered by Rust fixture tests so they stay executable instead
of becoming stale marketing samples.

## Quick Start

```sh
pnpm install
pnpm run build
./node_modules/.bin/tauri dev
```

Use the project-local Tauri binary for desktop commands. It avoids package
manager fetches in restricted-network environments.

## Verification

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

NEditor uses local verification rather than GitHub Actions. Run the commands
above before publishing a slice. Browser workflow tests are available through
`pnpm run test:e2e` when Playwright's local browser dependencies are installed
and the host allows Chromium to launch.

`cargo check` and `cargo test` require crates.io access the first time Rust
dependencies are resolved. After dependencies are present, the project is
designed to verify without global tools beyond the checked-in package managers
and local toolchains.

## Packaging Notes

```sh
pnpm run build
./node_modules/.bin/tauri build --no-bundle
./node_modules/.bin/tauri build --bundles app
```

On macOS, `.app` bundle creation is known to work in this workspace. DMG
creation can fail in restricted execution environments at the `hdiutil create`
step with `Device not configured`; that environment failure does not block app
bundle compilation.

## Project Status

NEditor is under active development and already has a broad implemented surface:
compiler, workbench UI, local file flows, conflict handling, transform registry,
review governance, AI cleanup, export readiness, export targets, example
fixtures, and local verification coverage. The remaining hardening work is
tracked conservatively in:

- [Specification](docs/specification.md)
- [External transform setup](docs/external-transforms.md)
- [Completion matrix](docs/spec-completion-matrix.md)
- [Current backlog](docs/todo.md)
- [Progress log](docs/progress.md)

Rows in the completion matrix move to complete only when current code, tests,
workflow evidence, artifacts, and platform checks prove the requirement.

## License

NEditor is released under the [MIT License](LICENSE). The npm package,
Cargo crate, and desktop bundle metadata all declare MIT licensing.
