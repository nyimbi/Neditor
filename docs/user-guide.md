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

The editor supports Markdown highlighting, line numbers, word wrap, code
folding, diagnostics, find and replace, smart list continuation, bracket
pairing, word count, character count, reading-time status, and source navigation
from diagnostics.

The command palette searches app actions, headings, citations, glossary terms,
index terms, open documents, and workspace files.

The top command bar is grouped around document, management, writing, navigation,
insertion, review, and view workflows. Use Navigate for search, previous/next
matches, the left document outline, and fold/unfold section commands. In
Settings or the command bar's Buttons selector, choose icons and text, icons
only, or text only to match the density you want.

## AI-First Workflows

Use **Agent** when you know the outcome but not the exact sequence of document
operations. Describe the goal in ordinary business language, for example:

> Create a board memo for the executive team, revise it for the CFO, check
> citations and risks, then distribute as PDF and Google Docs. audience:
> executive team owner: Strategy deadline: June 1

The AI Agent Workspace converts that instruction into a visible plan:

- creation, composition, editing, revision, review, and distribution lanes;
- missing inputs such as audience, owner, deadline, tone, evidence, reviewer,
  or approval status;
- placeholder values to pass into Docs Live;
- a suggested outline or the current document outline;
- revision instructions for selected text or the whole document;
- requested export targets such as PDF, Substack, LaTeX, or Google Docs;
- runnable next steps that open Docs Live, AI Paste cleanup, Review, or Export
  readiness.

Choose **Generate agent packet** when the instruction should become a reviewable
artifact immediately. NEditor creates an auditable packet with AI provenance,
the generated draft when creation or composition is requested, a
selection-aware revision proposal when editing or revision is requested, QA
gates, distribution gates, and blockers for missing inputs. The packet also
shows an **AI Control Center** with readiness score, next actions, source
grounding, governance state, and distribution state so reviewers can see what is
safe to do next before applying generated text. The same packet includes an
**Agent Audit Trail** with run ID, deterministic fingerprints for instruction,
context, source, and output payloads, rollback guidance, and review events.
**Apply agent output** uses the packet's safe apply mode: replace a new
document, replace the selected text, or append a review packet to the current
source.

Use **Build provider request** when your team wants a credentialed model to
continue the work outside the local deterministic planner. Choose the approved
provider profile, model, endpoint, and API-key environment variable. NEditor
creates a redacted handoff package with system prompt, user prompt, JSON body,
headers, cURL starter, and a safety checklist. The package is designed for
review before it leaves the local document workspace; it does not place API
keys or provider secrets into Markdown.

If policy allows direct execution, enter the API key in **Session API key** and
choose **Run provider request**. The key is held only in the open dialog, is not
written to the document, and is cleared when the session ends. Review the
provider response before using **Apply response**, because the imported text is
still marked for human review.

Use **AI Create** when the next action is clearly to create a first draft. It
opens Docs Live with an intent-first workflow: document type, outline, spoken or
typed context, placeholder values, AI-created questionnaire, section-by-section
drafting, QA, humanization, and review handoff.

Docs Live includes blueprints for business briefs, board memos, proposals,
strategy plans, project plans, research briefs, policies, meeting briefs,
business cases, operating procedures, technical architecture documents, ADRs,
release notes, contract briefs, marketing briefs, and customer case studies.
Each blueprint supplies a default outline, context questions, section focus, QA
prompts, humanization tasks, and reviewer handoff notes.

Before relying on dictation or clipboard-assisted cleanup, use **Check AI
runtime** in Docs Live. The report checks secure runtime context, Web Speech,
microphone permission, and clipboard read/write support, and records only
capability status plus clipboard character counts rather than clipboard
content.

Use **AI Paste** when text came from a chat tool and needs cleanup before it
enters the source document. Keep provenance enabled when the document needs an
audit trail.

Use the Templates panel or Insert > Templates to browse reusable transform
blocks. The built-in library includes business, scientific, and mathematical
`calc` blocks, plus chart, diagram, timeline, roadmap, ADR, CSV, OpenAPI, JSON
Schema, and QR templates. Built-ins can be inserted directly or duplicated into
custom templates that are saved with the workspace. Template cards show the
detected fill values for calculation and structured transform bodies before
insertion.

## Product Boundaries

NEditor is a local-first desktop workbench for document files. It does not
upload documents, synchronize drafts through a cloud service, or provide
real-time multiplayer collaboration.

The first release is not a mobile app, browser-hosted web app, server-side
rendering service, full WYSIWYG editor, enterprise identity layer, or arbitrary
plugin marketplace. Those may become separate product lines later, but the
current app stays focused on local Markdown source, deterministic compilation,
review evidence, and exportable deliverables.

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
- Import BibTeX with brace or parenthesis entries, Zotero/Better BibTeX-style
  metadata records, dotted keys, and `@` characters inside field values.
- Import CSL JSON as a root array, a single item, or an object with `items`,
  `references`, `bibliography`, or `data`.
- Choose `title`, `author-year`, `key`, `numeric`, or a common CSL alias such
  as `apa`, `chicago-author-date`, `ieee`, or `vancouver` from front matter,
  preferences, or the references panel.
- Use `glossary` fenced blocks for terms and definitions.
- Add `[GLOSSARY]` where the generated glossary should appear, or set
  `glossarySection: true` in front matter.
- Add `[INDEX]` where the generated index should appear, or set `index: true`
  / `index.enabled: true` in front matter.
- Use labels and cross references for headings, figures, tables, equations,
  appendices, and decisions.

The references panel surfaces resolved bibliography entries, missing citation
keys, and duplicate bibliography keys. Unsupported citation-style warnings are
reported with the rest of the document diagnostics. The same panel can insert
the bibliography marker, add a BibTeX template, create stubs for missing keys,
reinsert citation references, and copy resolved entries as editable BibTeX
blocks. Reuse of the same section, figure, table, equation, appendix, or
decision label is also reported as an export-blocking reference diagnostic so
links stay deterministic. Malformed labels and cross references, including
empty keys, spaces, slash characters, and unclosed markers, are also
export-blocking diagnostics with source ranges.

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
- Use Docs Live after creating an outline to build an outline-aware
  questionnaire, combine the answers with dictated direction, freeform context,
  and placeholder values, then generate a section-by-section draft with a
  drafting plan, review packet, QA checks, humanization tasks, and reviewer
  handoff prompts. The review packet summarizes context sources, the section
  work queue, assumptions to verify, cleanup checks, and reviewer ownership.
- Use Agent Workspace packets for multi-step document work that combines
  drafting, revision, review, and distribution. The packet records blockers,
  QA gates, and distribution gates before the output is accepted.
- Use provider handoff packages when a governed AI provider or local model
  gateway should continue an agent run. Review the package, redact sensitive
  facts where required, and import the response as a review draft. If direct
  execution is allowed, use a session-only key and apply the response only after
  checking the provider output.
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
Use `File` -> `Export` -> `HTML Export` for a direct standalone HTML write, or
select HTML in the Export panel and run the same readiness-backed export
workflow from there.

Supported targets:

- HTML for web previews, review copies, and static publishing.
- PDF for board papers, reports, proposals, and release packs.
- DOCX for Word-compatible client deliverables.
- PPTX for presentation-style summaries and executive decks.
- Markdown bundle for portable source handoff.
- Blog and Substack packages for local-first publishing handoff.
- LaTeX source for academic and technical handoff.
- Google Docs package for local-first import through DOCX, HTML, Markdown, or
  text fallbacks.

Export options cover manifests, styles, syntax highlighting, HTML language,
HTML description, canonical URL, cover pages, page numbers, layout presets,
comments appendix, AI provenance appendix, glossary appendix, PPTX agenda,
citation style, brand profile, dirty-Git warnings, transform engine settings,
and draft watermark behavior.
Front matter `targetPersona` entries are carried into the HTML cover metadata,
plain text/Markdown bundle text, bundled metadata, and DOCX/PPTX custom
properties so exported examples keep their intended audience visible.
Readiness also audits those options before writing: invalid citation-style
defaults, non-hex brand colors, malformed brand profile fields, and non-boolean
export toggles are reported in the same manifest-backed diagnostics as document
content problems. When optional appendix exports are enabled but the document
has no matching glossary, review, or AI provenance content, readiness keeps the
export allowed and records an informational diagnostic so the manifest explains
the no-op.

When `includeManifest` is enabled, NEditor writes sidecar evidence beside the
artifact. The manifest records source hashes, include hashes, export options,
include graph edges, diagnostics, readiness summary, layout sections, progress
steps, output path, and output hash.
Markdown bundle exports always include an embedded `manifest.json` for portable
handoff. If `includeManifest` is disabled for a Markdown bundle, NEditor still
embeds that internal manifest and records an informational readiness diagnostic;
only the sidecar manifest with final output path/hash evidence is suppressed.
Blog and Substack package exports are ZIP files with copy-ready Markdown,
standalone HTML, plain text, metadata, an RSS item seed, packaged media assets,
and an embedded NEditor manifest. Substack packages also include a minimal
`substack-copy.html` fragment intended for paste/import workflows. Their
`metadata.json` files name the primary publish file, fallback files, and
target-specific publishing steps so the package remains self-describing outside
NEditor.
Google Docs package exports are ZIP files with `document.docx` as the primary
Google Docs upload/import file, plus HTML, Markdown, plain text, metadata,
assets, and an embedded manifest for auditability. Their metadata names
`document.docx` as the primary import file, records HTML/Markdown/plain-text
fallbacks, and includes the Google Docs import workflow. LaTeX exports write a
direct `.tex` file and use the regular sidecar manifest when `includeManifest`
is enabled.

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
