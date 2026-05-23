# NEditor

NEditor is a local-first Markdown workbench for serious business documents:
board papers, consulting reports, technical architecture notes, research
briefs, proposals, export packs, and AI-assisted drafts that still need human
governance.

It keeps the speed and readability of Markdown, then adds the things business
documents usually force into office suites: structured metadata, includes,
tables, calculations, citations, diagrams, review comments, release status,
brand-aware exports, and reproducible export manifests. Your source stays local
and inspectable. The app helps turn it into polished HTML, PDF, DOCX, PPTX,
Markdown bundle, blog, Substack, LaTeX, or Google Docs package outputs.

![NEditor workbench showing Markdown source, live preview, outline, diagnostics, and status](docs/screenshots/workbench.svg)

## Start Here If You Just Want To Use NEditor

NEditor is meant to feel like a document app, not a developer tool. A normal
business user should be able to open the desktop app, create or open a document,
write in plain text, preview the formatted result, and export a polished file
without touching a terminal.

### Download And Install

Use the installer or app package supplied by your organization, project team, or
release manager.

| Platform | What to use | What happens |
| --- | --- | --- |
| macOS | `NEditor.app`, usually delivered in a `.dmg` or `.zip` | Drag the app to Applications, then open it like any other Mac app. |
| Windows | Windows installer or packaged `NEditor` app | Install it, then launch NEditor from the Start menu. |
| Linux | AppImage, Debian package, RPM package, or packaged desktop app | Mark it executable or install the package, then launch NEditor from your app menu. |

If you are reading this README from a source-code checkout and do not see a
ready-made installer, ask a technical teammate to build or provide the desktop
package for your platform. The developer build commands are later in this file.

### Your First Document

1. Open NEditor.
2. Choose **New** for a blank document, or **Open** to work on an existing
   Markdown file.
3. Use the **Outline** mode or Outline panel to sketch chapters, sections,
   subsections, and subsubsections before writing the full text.
4. Write in the source editor. The preview shows how the document will read.
5. Use **Save** or **Save As** so the document lives in a normal folder you
   control.
6. Use **Export** or **HTML Export** when you need a polished handoff.

NEditor documents are plain Markdown files. That means your work is not locked
inside a cloud account or proprietary database. You can store the files in a
normal project folder, a shared drive, or a Git repository if your team uses one.

### Common Business Workflows

| I need to... | Use this in NEditor |
| --- | --- |
| Draft a board paper, proposal, report, or briefing note | Start with Outline mode, then fill in the generated sections. |
| Reuse a standard company structure | Keep a Markdown template or example project and duplicate it for new work. |
| Send a web-ready review copy | Use **HTML Export**. |
| Send a client-facing document | Export to PDF or DOCX. |
| Prepare a presentation handoff | Export to PPTX or use presentation mode to review the structure. |
| Publish to a blog or Substack | Export the Blog or Substack package and copy the prepared content into the publishing tool. |
| Hand off to Google Docs | Export the Google Docs package and import the included DOCX into Google Docs. |
| Keep an audit trail | Turn on export manifests and use snapshots or Git tagging if your team needs formal release evidence. |
| Add calculations, charts, or diagrams | Use the built-in transform templates, then fill in the values. |

### What To Ask IT Or A Technical Teammate For

Most writing, review, file, and export workflows work inside the NEditor app.
Some optional capabilities may need setup by a technical teammate:

- A ready-made app package for your operating system.
- Optional diagram engines such as Graphviz, D2, PlantUML, or Pikchr.
- Company brand profiles, export defaults, and approved document templates.
- A shared folder, Git repository, or backup location for important documents.
- Release signing or notarization if your organization distributes NEditor
  internally.

### Where To Learn More

- [User guide](docs/user-guide.md): practical writing, file, export, review,
  and troubleshooting guidance.
- [Markdown extensions](docs/markdown-extensions.md): the extra document
  features NEditor understands.
- [External transform setup](docs/external-transforms.md): optional diagram and
  data engine setup for technical users.

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

## What NEditor Is Not

NEditor is deliberately scoped as a local-first, document-file-centered desktop
workbench. The first release is not trying to be a real-time multiplayer suite,
cloud document store, mobile app, full WYSIWYG editor, arbitrary plugin
marketplace, server-side renderer, enterprise identity layer, or browser-based
web app.

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
- First-class outline planning in the Outline panel: draft a document structure
  with indented bullets, numbers, or Markdown headings, then create or append a
  Markdown document skeleton before writing the body text.
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
HTML export is available as a first-class File menu action (`File` -> `Export`
-> `HTML Export`), a toolbar command, a command-palette command, and the default
Export panel target.

| Target | What it is for | Current support |
| --- | --- | --- |
| HTML | Web previews, review copies, static publishing | Standalone document metadata, language, canonical links, styles, syntax highlighting, semantic sections, transform artifacts, manifests |
| PDF | Board papers, reports, proposals, release packs | Paged layout model, page options, headers/footers, watermarks, tables, figures |
| DOCX | Word-compatible client deliverables | Document structure, paragraphs, tables, comments/provenance appendices, manifests |
| PPTX | Presentation-style summaries and executive decks | Slide packaging, agenda option, table splitting, manifest sidecars |
| Markdown bundle | Portable source handoff | Source document plus manifest and packaged evidence |
| Blog package | Blog publishing handoff | Copy-ready Markdown, HTML, text, publish workflow metadata, RSS seed, assets, and manifest |
| Substack package | Substack publishing handoff | Substack copy HTML, Markdown, text, publish workflow metadata, RSS seed, assets, and manifest |
| LaTeX | Academic or technical handoff | `.tex` source with metadata, headings, tables, figures, equations, links, and labels |
| Google Docs package | Google Docs import handoff | DOCX, HTML, Markdown, text, import workflow metadata, assets, and manifest |

Export defaults include manifests, styles, syntax highlighting, HTML language,
HTML description, canonical URL, cover page, page numbers, layout preset,
comments appendix, AI provenance appendix, glossary appendix, PPTX agenda,
citation style, brand profile, dirty-Git warnings, transform engine settings,
and draft watermark behavior.

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
of becoming stale marketing samples. The tests also keep the README links in
sync and prove each example's audience metadata and representative features
survive HTML, PDF, DOCX, PPTX, and Markdown bundle exports.

## Developer Quick Start

```sh
pnpm install
pnpm run build
./node_modules/.bin/tauri dev
```

Use the project-local Tauri binary for desktop commands. It avoids package
manager fetches in restricted-network environments.

## Verification

```sh
pnpm run verify:local
pnpm run verify:local:full
```

NEditor uses local verification rather than GitHub Actions. Run
`pnpm run verify:local` before publishing a normal slice. Run
`pnpm run verify:local:full` for a release-grade baseline; it extends the quick
checks with the production build, the full browser workflow suite, the focused
runtime accessibility audit, the manual accessibility review contract, optional
engine probe, native-watch check, clippy, full Rust tests, rendered export audit,
platform package configuration audit, Tauri no-bundle release compile, macOS
`.app` bundle build/smoke and DMG classification on macOS, desktop artifact
smoke, bounded macOS GUI launch smoke on macOS, and the desktop WebDriver smoke
or platform skip evidence.

Use `pnpm run verify:local -- --list` or
`pnpm run verify:local:full -- --list` to print the exact command sequence
without running it. Quick verification now includes the same browser environment
preflight exposed by `pnpm run check:e2e-env`,
which proves the current host can launch Chromium through NEditor's Playwright
wrapper. The full baseline runs the same browser suite exposed by
`pnpm run test:e2e`; the runner prefers
Playwright's workspace-local browser cache at `.tmp/ms-playwright` and falls
back to an installed Chrome-compatible browser when that cache is missing. To
refresh that local browser cache, run:

```sh
PLAYWRIGHT_BROWSERS_PATH=.tmp/ms-playwright pnpm exec playwright install chromium
```

Use `pnpm run check:e2e-env` before browser workflow runs. It defaults to the
project-local Playwright browser cache, records the browser source under
`.tmp/e2e-environment/report.json`, and runs the focused workbench boot workflow
through the same Playwright CLI path as the full browser suite. The check retries
transient browser launch failures, such as macOS headless Chrome closing before
app assertions, while still failing immediately for real workflow assertion
failures.

`pnpm run check:engines` probes optional external transform engines and reports
installed/missing Graphviz/DOT variants, D2, PlantUML, Java-backed PlantUML, and
Pikchr paths without failing just because an optional engine is absent. For each
installed engine it also renders a small SVG smoke artifact through the adapter
shape NEditor uses, writes those files under `.tmp/external-engines/artifacts/`,
and records paths, versions, byte counts, and compatibility status in
`.tmp/external-engines/probe-report.json` for local platform evidence.

`pnpm run check:deps` verifies that every JavaScript and Rust dependency in the
project manifests has an entry in the dependency admission record and that
NEditor package metadata still declares MIT licensing.

`pnpm run check:platform-packaging` verifies cross-platform package
configuration without requiring a Windows or Linux host. It checks synchronized
npm/Cargo/Tauri version and license metadata, all-platform Tauri bundle targets,
macOS/Windows/Linux icon coverage, production desktop window dimensions, CSP
guardrails, root MIT license linkage, and writes
`.tmp/desktop-bundle/platform-package-config-report.json` with the current
`unsigned-local-builds` signing stance. Release distribution signing,
notarization, and installer attestation remain credentialed release steps outside
local verification.

`pnpm run check:platform-evidence` writes
`.tmp/platform-evidence/report.json` and creates JSON templates under
`.tmp/platform-evidence/templates/` for Windows and Linux package/WebDriver
evidence. Missing supported-host evidence remains a release gap, but malformed
evidence copied back from another host fails the check instead of being silently
accepted.

`pnpm run check:release-readiness` aggregates the current local proof set into
`.tmp/release-readiness/report.json`. It fails if required current-host reports
are missing or failed, and otherwise records remaining external evidence gaps
such as Windows/Linux package artifacts, Windows/Linux WebDriver execution,
release signing/notarization, optional missing engines, and human reviewer
sign-off for accessibility or native-viewer export review.

`pnpm run test:rendered-exports` runs the representative rendered export audit
and writes local review artifacts to `.tmp/rendered-export-audit`: HTML, PDF,
DOCX, PPTX, Markdown bundle, blog package, Substack package, LaTeX, Google Docs
package, hashes, a manual visual-review checklist, and `viewer-proof.json` with
executable HTML/PDF/DOCX/PPTX/package assertions, publishing handoff workflow
metadata checks, LaTeX source checks, Google Docs import workflow checks, and
nested Google Docs DOCX checks. It also writes
`visual-review-summary.json`, which maps export targets and review cases to
browser screenshots, PDF raster thumbnails, generated DOCX/PPTX Office preview
dashboards, native tool proof, skipped host-limitation records, and human
sign-off state. It also writes `automated-visual-review.json`, a current-host
automated visual review report that requires browser-rendered HTML screenshots,
PDF raster proof, Office XML preview extraction, Office preview screenshots, and
mapped proof for every primary and review-case target before marking the audit
`automated-reviewed`. The audit also writes
`visual-review-signoff.template.json`; fill a copy and rerun with
`NEDITOR_RENDERED_EXPORT_SIGNOFF=/path/to/signoff.json` to validate completed
manual native-viewer review. On macOS it also extracts DOCX text through
`textutil` and attempts bounded Quick Look PDF/DOCX/PPTX thumbnail proof,
recording either thumbnail assertions or host sandbox limitations in
`viewer-proof.json`. When `pdflatex` is installed, it also compiles the
generated `.tex` file into `.tmp/rendered-export-audit/latex-compile/`.

`pnpm run test:performance-audit` writes `.tmp/performance-audit/report.json`
after running the Rust performance stress tests and the focused browser
large-document workflow through the project-local Playwright browser cache.

`pnpm run test:desktop-smoke` verifies the local Vite build, Tauri
configuration, package metadata, MIT license metadata, release desktop binary
produced by `./node_modules/.bin/tauri build --no-bundle`, and a native
command workflow smoke that opens, watches, compiles, checks readiness, exports,
and reveals real local files through the Rust command surface. The command writes
`.tmp/desktop-smoke/native-command-report.json` with binary/build metadata and
native command workflow duration. On machines that allow GUI app startup, run
`NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` for a bounded
native launch smoke. This launch smoke is part of the full local baseline on
macOS, so release-grade verification proves the app-authored Tauri window and
workflow reports instead of relying only on artifact inspection. The launch
smoke writes
`.tmp/desktop-smoke/launch-report.json` with the binary path, PID, elapsed
window, captured output, and `processAlive: true` evidence when the app remains
running until the timeout, plus `.tmp/desktop-smoke/native-window-report.json`
with app-authored package, identifier, main-window title, visibility, size, and
scale-factor evidence, `.tmp/desktop-smoke/native-ui-report.json` with rendered
workbench surface evidence, and `.tmp/desktop-smoke/native-workflow-report.json`
with native mode, real-file, conflict, transform-template, readiness, HTML
export, and guarded native menu-event export evidence.

`pnpm run test:desktop-bundle` verifies the current host's packaged desktop
bundle evidence. On macOS it checks `NEditor.app` Info.plist metadata, bundle
identifier, version, executable, icon, copyright, and high-resolution flag after
`./node_modules/.bin/tauri build --bundles app`.

`pnpm run test:desktop-dmg` verifies macOS DMG packaging when this host can run
`hdiutil`. In restricted execution environments it records the known sandboxed
`hdiutil create` failure as `.tmp/desktop-bundle/macos-dmg-report.json` instead
of treating it as app-bundle or metadata regression.

`pnpm run test:tauri-webdriver` runs the Tauri WebDriver desktop smoke on
Windows and Linux hosts with `tauri-driver` plus the platform WebDriver
installed. The harness starts the built desktop binary, checks the native title,
switches view mode, opens the command palette, creates a dirty document and
checks the native dirty-title marker, inserts a Science calc template through
the Templates panel, saves and reopens a real Markdown file through the guarded
dialog-free smoke path, renames, duplicates, and reveals deterministic Markdown
files, runs export readiness through the desktop command path, writes a real HTML
export, validates the sidecar manifest/output hash, and verifies selected
preferences survive a desktop session restart before restoring them. It writes
`.tmp/desktop-webdriver/report.json` with dependency, assertion, file artifact,
export artifact, pass, or skip evidence. On macOS, official Tauri WebDriver is
skipped because the supported stack does not provide a WKWebView driver; use the
bounded desktop launch smoke there. When the bounded launch smoke has already
run on macOS, the WebDriver report also records `fallbackProof` from
`.tmp/desktop-smoke/native-command-report.json` so the skip still points to
current app-authored native file, export, workspace, snapshot, and UI evidence.
The fallback proof is freshness-checked against the built desktop binary and
the bounded launch report so stale `.tmp` native reports cannot satisfy the
WebDriver evidence contract.

`cargo check` and `cargo test` require crates.io access the first time Rust
dependencies are resolved. After dependencies are present, the project is
designed to verify without global tools beyond the checked-in package managers
and local toolchains.

## Packaging Notes

```sh
pnpm run build
./node_modules/.bin/tauri build --no-bundle
./node_modules/.bin/tauri build --bundles app
pnpm run test:desktop-bundle
pnpm run test:desktop-dmg
```

On macOS, `.app` bundle creation is part of `pnpm run verify:local:full` and the
bundle checker writes `.tmp/desktop-bundle/macos-app-report.json`. The DMG
checker writes `.tmp/desktop-bundle/macos-dmg-report.json`; on this sandboxed
host, `hdiutil create` cannot start `hdiejectd` and returns `Device not
configured`, which classifies the failure as host-specific rather than an app
bundle regression.

## Project Status

NEditor is under active development and already has a broad implemented surface:
compiler, workbench UI, local file flows, conflict handling, transform registry,
review governance, AI cleanup, export readiness, export targets, example
fixtures, and local verification coverage. The remaining hardening work is
tracked conservatively in:

- [Specification](docs/specification.md)
- [User guide](docs/user-guide.md)
- [Markdown extensions](docs/markdown-extensions.md)
- [External transform setup](docs/external-transforms.md)
- [Storage model](docs/storage-model.md)
- [Security threat model](docs/security-threat-model.md)
- [Completion matrix](docs/spec-completion-matrix.md)
- [Current backlog](docs/todo.md)
- [Progress log](docs/progress.md)

Rows in the completion matrix move to complete only when current code, tests,
workflow evidence, artifacts, and platform checks prove the requirement.

## License

NEditor is released under the [MIT License](LICENSE). The npm package,
Cargo crate, and desktop bundle metadata all declare MIT licensing.
