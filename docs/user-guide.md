# NEditor User Guide

This guide is for people using NEditor to write and ship local-first business
Markdown documents. It complements the implementation specification with
practical workflows and links to the syntax reference.

## Start A Workspace

1. Install dependencies:

   ```sh
   pnpm install
   ```

2. Run the desktop app during development:

   ```sh
   ./node_modules/.bin/tauri dev
   ```

3. Open a Markdown file or a project folder from the workbench.

4. Keep source files in a normal folder or Git repository. NEditor stores
   preferences, recent paths, and automatic snapshots locally; it does not
   upload documents or sync them in the background.

Good first examples:

- [Board paper](../examples/board-paper.md)
- [Consulting report with includes](../examples/consulting-report.md)
- [Research report with bibliography](../examples/research-report.md)
- [Proposal with budget tables and formulas](../examples/proposal-budget.md)
- [AI-assisted draft with review provenance](../examples/ai-assisted-draft.md)

## Workbench Basics

The main screen is a document workbench rather than a note-only editor.

- Use split mode for source and preview together.
- Use source mode for focused authoring.
- Use preview mode to read the compiled document.
- Use focus mode for distraction-light drafting.
- Use export, review, and presentation modes when preparing a deliverable.

The editor supports Markdown highlighting, line numbers, word wrap, diagnostics,
find and replace, smart list continuation, bracket pairing, word count,
character count, reading-time status, and source navigation from diagnostics.

The command palette searches app actions, headings, citations, glossary terms,
index terms, open documents, and workspace files.

## Files And Workspaces

NEditor treats Markdown files as the source of truth.

- Open, save, save as, duplicate, rename, reveal, and revert local files.
- Open folders to browse project Markdown documents.
- Pin frequently used tabs.
- Reopen recent files, recent folders, and recently closed documents.
- Restore previous workspace state after restart, including active tab and
  scroll position where available.

When a file changes on disk, NEditor protects local edits:

- Clean documents can reload automatically.
- Dirty documents open a conflict workflow.
- Conflict actions include compare, accept external, keep local, save copy, and
  merge selected lines.
- Saves use file hashes so a stale save cannot silently overwrite a newer disk
  edit.

## Master Documents And Includes

Large deliverables can be split into smaller files and compiled as one document.

Use any supported include form:

```md
!include chapters/introduction.md
{{include chapters/market-analysis.md}}
<!-- include: appendices/financials.md -->
```

Includes resolve relative to the current document. Child front matter is
stripped when included into a master document. Missing includes, circular
includes, and unsafe include depth emit diagnostics.

NEditor tracks the include graph for preview, export manifests, and snapshot
metadata. The References sidebar shows parent-to-child include edges with
depth, opens included files directly, and jumps back to the include directive in
the parent document. File watchers recompile master documents when included
files change.

## Markdown Extensions

NEditor supports standard Markdown plus business-document extensions:

- YAML front matter for metadata, layout, brand, release, citation, and export
  settings.
- Generated sections such as `[TOC]`, `[INDEX]`, `[GLOSSARY]`,
  `[BIBLIOGRAPHY]`, `[LIST_OF_FIGURES]`, and `[LIST_OF_TABLES]`.
- Variables such as `{{client}}` and formula expressions such as
  `{{=margin | percent}}`.
- Calculation blocks, tables, table formulas, and data transforms.
- Citations, bibliography entries, glossary entries, cross references,
  captions, equations, review comments, change notes, and AI provenance.
- Fenced-code transforms for charts, diagrams, data, timelines, APIs, schemas,
  maps, and static artifacts.

Use [Markdown extensions](markdown-extensions.md) as the syntax reference.

## Tables, Data, And Calculations

Use normal Markdown tables for readable source. The table editor can import
pasted tables, add or remove rows and columns, sort numeric data, apply
alignment, add totals, preserve merged-cell metadata, and write clean Markdown
back to the document.

Use `calc` blocks for document-level calculations:

````md
```calc
revenue = 100
cost = 40
profit = revenue - cost
margin = profit / revenue
```

Margin: {{=margin | percent}}
````

CSV and TSV transform blocks can also evaluate formula cells and feed export
artifacts.

## Citations, Glossary, Index, And References

Research and business documents can keep reference metadata near the source.

- Use citation syntax such as `[@porter1985]` or `[@porter1985, p. 42]`.
- Add `[BIBLIOGRAPHY]` where the rendered references should appear.
- Choose `title`, `author-year`, `key`, or `numeric` citation style from front
  matter, preferences, or the references panel.
- Use `glossary` fenced blocks for terms and definitions.
- Add `[GLOSSARY]` where the generated glossary should appear, or set
  `glossarySection: true` in front matter.
- Add `[INDEX]` where the generated index should appear, or set `index: true`
  / `index.enabled: true` in front matter.
- Use labels and cross references for headings, figures, tables, equations,
  appendices, and decisions.

The references panel surfaces resolved bibliography entries, missing citation
keys, and duplicate bibliography keys. Unsupported citation-style warnings are
reported with the rest of the document diagnostics.

## Review, Release, And AI Governance

NEditor is built for drafts that need evidence before publication.

- Set release status in front matter, for example `draft`, `in-review`,
  `approved`, `published`, or `archived`.
- Use the titlebar release badge and Review panel summary to see the current
  document status while editing.
- Add approval metadata before release-grade exports.
- Use inline review comments and change notes to keep audit context near the
  text.
- Use AI paste cleanup to normalize chat output, add citation TODOs, insert
  into a selected section, append as an appendix, or replace selected content.
- Use `ai-source` blocks and AI-assisted section metadata so generated content
  can be marked as needing review or human-reviewed.

Export readiness reports unresolved comments, incomplete AI provenance,
missing approval metadata, missing captions or labels, broken references, dirty
Git state, and other issues that affect deliverability.

## Transform Engines

NEditor uses native Rust renderers or static fallbacks where practical.
External diagram engines are optional and must be explicitly trusted.

External engines include Graphviz/DOT, D2, PlantUML, and Pikchr. Configure
exact executable paths in settings, probe the engine, and trust only the engine
you intend to use. Execution is bounded by timeout and output-size limits, and
no engine is invoked through interpolated shell strings.

See [External transform setup](external-transforms.md) for platform-specific
install paths and troubleshooting.

## Export Deliverables

Exports are generated from the compiled semantic document model.

Supported targets:

- HTML for web previews, review copies, and static publishing.
- PDF for board papers, reports, proposals, and release packs.
- DOCX for Word-compatible client deliverables.
- PPTX for presentation-style summaries and executive decks.
- Markdown bundle for portable source handoff.

Export options cover manifests, styles, syntax highlighting, cover pages, page
numbers, layout presets, comments appendix, AI provenance appendix, glossary
appendix, PPTX agenda, citation style, brand profile, dirty-Git warnings,
transform engine settings, and draft watermark behavior.

When `includeManifest` is enabled, NEditor writes sidecar evidence beside the
artifact. The manifest records source hashes, include hashes, export options,
include graph edges, diagnostics, readiness summary, layout sections, progress
steps, output path, and output hash.

## Versioning And Snapshots

If a document is inside a Git repository, NEditor can show repository status,
branch, dirty files, history, diffs, commits, restore actions, release tags,
and dirty-export warnings.

Git-free snapshots remain available for non-Git users. Automatic snapshots
default to app data. Project-local `.neditor/` snapshots are opt-in and are
added to `.gitignore` when enabled.

Snapshot restore is document-scoped. The app restores only Markdown snapshots
from the active document's configured snapshot store, and it checks matching
snapshot metadata before loading the older text back into the editor as an
unsaved change.

The Versioning panel shows Git status, diff, history, release tagging controls,
and the active document's snapshots together so teams can recover a draft,
inspect changes, and tag an approved release from one workflow.

See [Storage model](storage-model.md) and
[Security threat model](security-threat-model.md) for persistence and trust
boundaries.

## Troubleshooting

- If a local link or image is missing, check the diagnostics panel and export
  readiness report.
- If a save is blocked, the file changed on disk after the editor last read it.
  Use the conflict workflow instead of forcing an overwrite.
- If an external transform is disabled, configure the executable path, probe
  it, and trust the engine explicitly.
- If an export is blocked, open export readiness and fix errors before writing
  the artifact.
- If Playwright browser workflows cannot run locally, install the browser
  dependencies on a host that allows Chromium to launch. NEditor does not use
  GitHub Actions as the current verification surface.
- If macOS DMG creation fails with `Device not configured`, use the `.app`
  bundle or `tauri build --no-bundle` proof in restricted environments.
