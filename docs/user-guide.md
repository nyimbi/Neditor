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

These fixtures are not static samples: automated export tests prove their
audience, local-first delivery model, and Markdown source-of-truth metadata stay
intact in reviewable artifacts.

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
index terms, open documents, and workspace files. It also accepts typed
AI-first instructions such as "revise this for the board and prepare PDF" and
previews the detected workflow lanes, output targets, and missing inputs before
you choose **Plan first** or **Generate Packet**. It also exposes recent Docs
Live draft recovery actions, including opening the history, appending the
latest draft, copying the latest draft, and inserting or copying the latest
review packet.

The Outline sidebar includes a **Document Map** that combines headings, review
comments, citation TODOs, inline citation references, bibliography entries,
includes, glossary terms, index terms, figures, tables, equations, and compiler
diagnostics. Use its filters, search, and **Blockers** view to jump directly to
missing bibliography entries, unresolved comments, unlabeled media, broken
includes, or other review issues without hunting across separate panels.

The top command bar is grouped around document, management, writing, navigation,
insertion, review, and view workflows. Use Navigate for search, previous/next
matches, the left document outline, and fold/unfold section commands. In
Settings or the command bar's Buttons selector, choose icons and text, icons
only, or text only to match the density you want. Buttons show contextual help
on hover and keyboard focus, including disabled-state guidance when an action
needs a document, selection, outline, or export-ready state first.

## AI-First Workflows

For the full product direction, see the
[AI-first platform roadmap](ai-first-platform-roadmap.md). It captures 50
concrete changes across agentic creation, outline-first composition, revision,
source grounding, review, distribution, provider operations, UI, governance,
and verification.

Use **Agent** when you know the outcome but not the exact sequence of document
operations. Describe the goal in ordinary business language, for example:

> Create a board memo for the executive team, revise it for the CFO, check
> citations and risks, then distribute as PDF and Google Docs. audience:
> executive team owner: Strategy deadline: June 1

The AI Agent Workspace converts that instruction into a visible plan:

- searchable and filterable workflow playbooks for board approvals, client
  proposals, SOPs, strategy memos, policies, release notes, grant applications,
  technical papers, blog/Substack publishing, and executive revision passes;
- creation, composition, editing, revision, review, and distribution lanes;
- missing inputs such as audience, owner, deadline, tone, evidence, reviewer,
  or approval status;
- placeholder values to pass into Docs Live;
- a suggested outline or the current document outline;
- revision instructions for selected text or the whole document;
- planned revision passes such as clarity, brevity, tone, evidence, legal
  caution, executive summary, accessibility, and humanization;
- reusable document memory for terminology, style, accepted decisions,
  rejected directions, reviewer preferences, and distribution preferences;
- requested export targets such as PDF, Substack, LaTeX, Google Docs, or EPUB;
- runnable next steps that open Docs Live, AI Paste cleanup, Review, or Export
  readiness.

When the plan lists missing inputs, answer them in **Context answers and
constraints** and choose **Replan with answers**. Those answers become part of
the context pack, placeholder resolution, generated agent packet, Docs Live
handoff, provider request package, and saved run history instead of living only
in a separate chat note. Use **Source Pack Builder** to add source notes, claims,
URLs, file paths, references, and reviewer comments before planning or provider
handoff; those managed items become part of the context pack, generated packet,
reviewer tasks, provider source evidence pack, and saved run history. Use
**Document memory** in the same workspace to add or capture terminology, house
style, accepted decisions, rejected directions, reviewer preferences, and
distribution preferences; memory packs can be inserted into the Markdown source
for human review and reused by future agent plans. Choose **Save memory** to
persist the managed memory library with the workspace, **Reload saved** to
restore it after experimentation, or **Append saved memory** to merge the saved
library into a new agent planning context. The run
history can be searched and filtered by status, workflow lane, or output target
so generated packets, applied revisions, provider-applied drafts, evidence
blockers, and distribution handoffs remain recoverable after the workspace
accumulates multiple agent runs. Use **Insert audit** or **Copy audit** to turn
the filtered history into a Markdown evidence package for reviewers, release
notes, or customer handoff records. The plan also scores context completeness across
audience, evidence, constraints, examples, tone, and approval context, with
recommendations for missing grounding before users generate a first draft.

Choose **Generate agent packet** when the instruction should become a reviewable
artifact immediately. NEditor creates an auditable packet with AI provenance,
the generated draft when creation or composition is requested, a
selection-aware revision proposal when editing or revision is requested,
the planned revision pass checklist, meaning-drift findings for changed or
removed numbers, dates, commitments, and caveats, and an edit acceptance queue
where each selected-text, document, or section edit can be accepted, rejected,
sent for another revision pass, and applied only after approval. The packet also
includes document-type quality gates for board memos, proposals, policies, SOPs,
research/technical papers, publishing drafts, and other supported documents, plus
distribution gates, a review comment resolution queue with per-comment required
actions, resolution options, task notes, and blocker status, and missing-input blockers. It
shows an **AI Control Center** with readiness score, next actions, source
grounding, governance state, and distribution state so reviewers can see what is
safe to do next before applying generated text. It also generates a **Release
Evidence Bundle** covering audit trail, source grounding, review closure,
approval metadata, provider proof, distribution artifact requirements, and
per-target export or publishing evidence obligations. The control center and reviewer
agents inspect the current Markdown for unresolved placeholders, citation TODOs,
candidate claims with line numbers, unreviewed AI provenance markers,
humanization findings for generic or overconfident phrasing, unresolved
comments, missing approval metadata, and placeholder links, so the agent review
reflects the document in front of the user instead of only the prompt. Those
findings also become
concrete next actions and lifecycle tasks, so evidence cleanup can be assigned,
run, noted, and completed from the same agent board. After a packet is generated,
the latest AI Control Center also appears in the Review sidebar so readiness,
next actions, grounding, governance, and distribution state remain visible while
the user edits the document. The Review sidebar also includes **AI quality
assistance** that turns current QA findings into suggested triage, evidence,
humanization, and reviewer-handoff answers with visible rationale and context
signals; accept the guidance into editable review notes, revise it, then insert
the notes as a quality handoff artifact. When the agent detects numbers, dates,
commitments, quotes, or unsupported claims, the **Claim Inventory** panel gives
each finding a source jump, an insertable/copyable audit table, and a one-click
citation TODO action. The same packet includes named
reviewer agents for editorial quality, evidence grounding, risk, citations,
governance, and export readiness so each review lane has findings and required
actions instead of one undifferentiated checklist. It also includes deterministic
outline critique for coverage, sequencing, duplicated headings, excessive depth,
and generic section names before drafting starts, deterministic humanization
findings for stale AI phrasing and vague transitions, plus a section-by-section
work queue with drafting instructions, completion criteria, and assigned
reviewer agents so a team can build the document systematically
from outline to reviewed sections. Each queued section can insert a visible work
brief into the Markdown source or open Docs Live preloaded with that section's
instruction, criteria, placeholders, and reviewer agents for focused drafting.
When the section title already exists in the Markdown source, applying that
focused Docs Live draft replaces the matching section for review instead of
duplicating the work at the end of the document; if the section does not exist,
NEditor appends the generated section as a new review block.
The same packet includes a **Lifecycle Task Board** that turns the whole
document workflow into owned tasks across creation, composition, editing,
revision, review, and distribution. Natural-language instructions in the
Command Palette now surface route buttons for Docs Live, AI Paste cleanup,
Review governance, Export readiness, Outline mode, and provider handoff. Use
**Run task** when a task should route you to Docs Live, Outline, AI Paste,
Review, or Export readiness from the
agent's plan. Filter the task board by lane, status, or freeform search when a
packet has many section, reviewer, evidence, and distribution tasks. Use
**Start**, **Needs review**, and **Complete** to persist each
task's execution state in local run history, and use the task note field for
evidence, blockers, reviewers, or completion notes. Use **Insert brief** or
**Copy brief** when a task should become a durable Markdown work order for a
reviewer, writer, or distribution owner.
The evidence scan also checks label and cross-reference integrity, turning
missing labels, malformed reference syntax, and unmatched `{@label}` links into
review tasks, Control Center findings, provider handoff evidence, and release
evidence blockers when distribution is in scope.
The packet also includes an
**Agent Audit Trail** with run ID, deterministic fingerprints for instruction,
context, source, and output payloads, rollback guidance, and review events.
Generated and applied runs are also saved in local workspace history so a user
can reopen the Agent Workspace later and inspect recent run IDs, readiness,
provider, apply mode, fingerprints, section/reviewer/task counts, and a bounded
packet snapshot that can be appended back into the document or copied for
review. The current live packet can also be copied or appended immediately from
the generated output header when the team wants review material without using
the packet's replacement mode. Use **Replan** on a saved run when you want to
restart from the same instruction with the current document context. Remove a
single saved run or clear the local run history when old AI artifacts should no
longer remain in workspace preferences.
The AI Control Center's next actions are directly runnable from Agent Workspace
and from the Review sidebar, so a recommendation can open Docs Live, start a
section draft, route to review readiness, prepare export, or launch AI Paste
without translating the agent's instruction into separate UI steps.
**Apply agent output** uses the packet's safe apply mode: replace a new
document, replace the selected text, or append a review packet to the current
source.

Use **Build provider request** when your team wants a credentialed model to
continue the work outside the local deterministic planner. Choose the approved
provider profile, model, endpoint, and API-key environment variable. NEditor
creates a redacted handoff package with system prompt, user prompt, JSON body,
headers, cURL starter, source evidence pack, lifecycle task-board context,
reviewer assignments, section work queue, and a safety checklist. The source
pack carries extracted claims, citation TODOs, humanization findings, outline
critique, governance blockers, and distribution blockers so the provider sees
the same local review evidence as the human reviewer. The package is designed
for review before it leaves the local document workspace; it does not place API
keys or provider secrets into Markdown.

If policy allows direct execution, enter the API key in **Session API key** and
choose **Run provider request**. The key is held only in the open dialog, is not
written to the document, and is cleared when the session ends. Review the
provider response before using **Apply response**. NEditor wraps imported
provider output in local `ai-source` and `ai-assisted` needs-review provenance,
so a provider response cannot enter the document as approved final copy. The
saved run history stores that wrapped review draft, so later append/copy actions
reproduce the same governed material that was applied.

Open **Help > Open Capability Showcase** before onboarding new users. NEditor
loads the packaged showcase document with real headings, tables, equations,
images, citations, includes, AI provenance, review comments, release metadata,
and export settings. Then open **Help > Guided Demo**. Each demo step now points
to evidence in the showcase so business users can practice routing tasks,
inserting briefs, copying handoffs, building provider packages, governing AI
output, reusing local source libraries, mapping claims to saved source
evidence, preflighting publishing targets, completing setup, and preparing
exports against a real document instead of a blank page. Trainers can mark
steps complete, reset locally persisted progress, or insert/copy the generated
Markdown checklist as onboarding evidence.

Use **AI Create** when the next action is clearly to create a first draft. It
opens Docs Live with an intent-first workflow: document type, outline, spoken or
typed context, managed placeholder values, AI-created questionnaire,
section-by-section drafting, QA, humanization, and review handoff. Use the
Placeholder Manager when you want a business-friendly table for clients,
owners, dates, evidence, reviewers, numbers, sources, and repeated variables;
each row can carry a type, source or evidence note, and review status while the
underlying plain-text placeholder block stays synchronized for Docs Live, agent
packets, and provider handoffs.
When dictating revision requests, review the **Voice command plan** and use
**Use commands** to convert phrases such as "expand the executive summary" or
"turn pricing into a table" into scoped drafting actions before generation.
Use **Run drafting** or a per-command **Draft with agent** button when those
voice edits should become a governed Agent Workspace packet. NEditor includes
the current editor selection when one exists, otherwise the current Markdown
section, so the agent receives the spoken command, target, rationale, and
source context before proposing reviewed Markdown.
Workflow phrases such as "open Deep Research", "run QA", "prepare export", or
"read selected text aloud" appear in the same plan with **Run workflows** and
per-command run buttons, so voice input can navigate creation, review,
research, export, table, provider, help, and read-aloud workflows without
forcing users back to the command palette.

Docs Live includes blueprints for business briefs, board memos, proposals,
strategy plans, project plans, research briefs, policies, meeting briefs,
business cases, operating procedures, technical architecture documents, ADRs,
release notes, contract briefs, marketing briefs, and customer case studies.
Each blueprint supplies a default outline, context questions, section focus, QA
prompts, humanization tasks, and reviewer handoff notes. After generation, copy
the draft for external review, insert or copy the standalone review packet,
append it as review material, or apply it with the selected
replace/append/selection/section mode. NEditor keeps recent Docs Live drafts in
a bounded local history so you can append or copy a saved draft, or insert or
copy its review packet later without rebuilding the first draft.
Remove stale saved drafts or clear the local Docs Live history when a draft is
no longer useful or should not remain in workspace preferences.

Before relying on dictation or clipboard-assisted cleanup, use **Check AI
runtime** in Docs Live. The report checks secure runtime context, Web Speech,
microphone permission, and clipboard read/write support, and records only
capability status plus clipboard character counts rather than clipboard
content.

For read-aloud workflows, open **Settings -> AI, agents, and voice**. Browser
speech and macOS Say do not require model downloads. If you choose
**Supertonic CLI**, NEditor shows the model name, approximate size, editable
storage path, download source, and command before any model-backed speech can
start. The **Download model** and **Read document** actions stay disabled until
you acknowledge the download notice.

The packaged CLI exposes the same consent gate for scripted review sessions.
Use `ned read-aloud report.md --engine macos-say --dry-run --json` to inspect
the OS-native command without starting playback. For Supertonic, use
`ned read-aloud report.md --engine supertonic-cli --model-storage ~/.cache/supertonic/models --dry-run --json`
first; playback is blocked until you rerun with
`--acknowledge-model-download`, after reviewing the model name, approximate
size, and storage location. Add `--script-output read-aloud.sh` when a reviewer
needs an auditable local handoff script.

Use **AI Paste** when text came from a chat tool and needs cleanup before it
enters the source document. Keep provenance enabled when the document needs an
audit trail.

Use the Templates panel or Insert > Templates to browse reusable transform
blocks. The built-in library includes business, scientific, and mathematical
`calc` blocks, plus chart, diagram, timeline, roadmap, ADR, CSV, OpenAPI, JSON
Schema, and QR templates. Built-ins can be inserted directly or duplicated into
custom templates that are saved with the workspace. Template cards show the
detected fill values for calculation and structured transform bodies before
insertion. The same panel includes **AI template assistance** for the full
template workflow: choose the right template from the current filters and
document context, replace sample values with sourced inputs, preview and verify
the result, and write a reviewer handoff note when the transform supports a
financial, scientific, compliance, or client-facing claim.

The Templates panel also includes **Versioned clauses** for approved language
such as confidentiality, commercial validity, AI review disclosure, and
workspace-specific clauses. Create or edit a custom clause with a label, kind,
current version, stale markers, and Markdown body; NEditor fills saved business
profile placeholders when inserting it, then audits the active document for
current, stale, and missing clause versions before external review.

Use **Reusable document parts** when a full document template is too heavy.
Built-in parts cover contact blocks, company overviews, executive summaries,
scope, pricing assumptions, compliance matrices, tender checklists, tutorial
steps, risk registers, and review handoffs. You can also save custom parts with
a label, kind, summary, and Markdown body; custom parts are profile-aware,
searchable beside built-ins, persisted with the workspace, and installable from
template packs.

Use **Template pack marketplace** in the same panel to package filtered
business templates, snippets, outlines, transform templates, company LaTeX
profiles, and custom versioned clauses into a portable
`neditor.template-pack.v1` JSON file. Packs include publisher metadata,
version, license, tags, placeholder inventory, examples, outline rules, and
usage guidance so another workspace can inspect the pack before installing it.
Pasted packs install reusable document parts, outlines, transform templates,
LaTeX templates, and custom clauses into the local workspace library; business
templates remain preserved in the pack manifest for review and copy-forward use.

Use **References > Local data sources** for CSV, TSV, JSON, YAML, and XLSX
inputs that should travel with a document. The **Data refresh workflow** shows
which sources are compiled, blocked, stale against the last audit, or ready to
import as editable Markdown tables. Choose **Refresh preview imports** after
source files change, **Import as editable table** for CSV/TSV/XLSX sources that
need hand editing in the table grid, and **Insert refresh audit** before review
or export so reviewers can see source status and verification notes.

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

Open References and use the Include document builder when you want to
incorporate one Markdown document inside another without memorizing directive
syntax. Enter the child document path, choose the directive style, review the
preview, then insert it into the parent document. If the child file does not
exist yet, choose **Create child document** to create a starter Markdown file
beside the saved parent document, insert the include directive, and open the
new child for editing.

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
back to the document. It is also a two-way source editor: place the cursor
inside an existing Markdown table, open Tables, choose **Edit table at cursor**,
change the grid, then choose **Apply** to replace that exact table in the text.
Choose **New table in text** or **Insert draft in text** when you want NEditor
to place a starter or visual draft table directly in the Markdown editor and
select it for immediate text editing.
Use the **Markdown source** box when you want a two-way source-and-grid view;
valid pipe-table text updates the visual grid as you type, and **Apply source
text** writes that edited source into the document. Use **Edit Markdown in
text** when you want to select the exact table source lines and revise the pipe
table directly in the editor; valid text edits sync back into the visual grid.
Use **Go to source table** when you only need to jump from the visual editor
back to the Markdown range being edited. The same table workflow is available
from the Writing Tools menu, command palette, toolbar, and native desktop
Writing Tools menu, so business users can reach it from whichever command
surface they are already using.

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
artifacts. Front matter data sources can reference local CSV, TSV, JSON, YAML,
or XLSX files; XLSX sources render a safe default worksheet as a Markdown table
and stay under the same document-folder path safety rules as other local
sources. For multi-sheet workbooks, add `sheet: Sheet Name` or a one-based
`sheetIndex: 2` to a `dataSources` XLSX entry to render a specific worksheet;
the References panel's local data-source manager exposes the same worksheet
fields when adding an XLSX source, and its file picker infers CSV, TSV, JSON,
YAML, or XLSX type from the selected local file. Use **Copy to data folder**
after choosing an external file to copy it into the active document or
workspace `data/` folder and replace the draft path with a portable
document-relative path.

For spreadsheet exchange, open the Tables panel and choose CSV, TSV, or XLSX
import. Review the detected headers, rows, alignment, and formula cells, then
insert the table as Markdown. Use table export when a reviewer needs CSV or
XLSX output from a Markdown table. XLSX import lists workbook worksheets in the
Tables panel so you can switch sheets before inserting or applying the Markdown
table, and uses cached formula values when the workbook provides them.

For database-backed tables, insert a SQL transform template or write a fenced
`sql` block with a `database` option. Configure and trust the local `sqlite3`
engine in Configuration Center before running it. NEditor accepts read-only
`SELECT` or `WITH` queries, rejects mutation statements and multi-statement
batches, resolves relative database paths such as `data/revenue.sqlite` from
the Markdown document's folder, blocks relative paths that escape that folder,
and renders the result as a Markdown table that can be reviewed, copied, and
exported with the rest of the document.

Use **Configuration Center > Transforms > Safe database profiles** to save
reusable database connection metadata without storing passwords or tokens in a
document. Profiles can point to a local SQLite file or to an environment
variable such as `NEDITOR_DATABASE_URL`, record owner/review notes, mark the
profile read-only, and insert a SQL transform scaffold that keeps credentials
in environment variables or session setup.

Before inserting a calculation, chart, diagram, SQL query, or other transform,
open Templates and review the AI template assistance cards. Accept the guidance
into transform assistance notes, add owners and source values, then insert the
notes into the document when the rendered output affects claims, budgets,
requirements, or review decisions.

## Citations, Glossary, Index, And References

Research and business documents can keep reference metadata near the source.

- Use citation syntax such as `[@porter1985]` or `[@porter1985, p. 42]`.
- Add `[BIBLIOGRAPHY]` where the rendered references should appear.
- Import BibTeX with brace or parenthesis entries, Zotero/Better BibTeX-style
  metadata records, dotted keys, and `@` characters inside field values.
- Use BibTeX transform previews when you need to inspect reference metadata:
  entry type, author, year/date, journal or book title, publisher, volume,
  issue, pages, DOI, and URL fields are shown before export.
- Import CSL JSON as a root array, a single item, or an object with `items`,
  `references`, `bibliography`, or `data`.
- Deep Research report handoffs append CSL JSON records for any missing source
  keys even when the AI draft already contains an empty or partial bibliography
  fence, complete empty or partial Source Citation Index sections with a
  missing-key addendum, complete empty or partial evidence logs with missing
  research iterations, complete empty or partial Source Library Audit sections
  with missing saved-source rows, add an evidence-conflict review when search
  snippets appear to disagree, then add the `[BIBLIOGRAPHY]` marker.
- Choose **Local source library** as the search provider when you want Deep
  Research or citation search to reuse downloaded sources associated with the
  current document. It ranks saved sources by title, snippet, path, provider
  metadata, and readable local file content without sending a search request to
  the web.
- From the terminal, use
  `ned sources --document report.md --query "market evidence"` to search
  DuckDuckGo, SearXNG, Tavily, or the local source library; use
  `ned sources --document report.md --download-url https://example.com/source.pdf`
  to save a source into the document-associated source vault; and use
  `ned sources --document report.md --audit --output source-audit.md` to
  create a reviewer-ready audit of saved files, hashes, citation keys, source
  quality notes, and missing or modified local evidence.
- Use
  `ned deep-research --topic "market evidence" --provider duckduckgo --pages 10 --output research-dossier.md`
  when a terminal workflow should create an editable source-backed research
  dossier with a page-budget table and section-by-section draft queue. Add
  `--document report.md --save-sources` to download returned source files into
  the document vault, or choose `--provider local-library` to build from
  already saved sources without a live web search.
- In **Review**, refresh **Evidence and approval review** to extract candidate
  claims, then use **Insert claim-source matrix** to map those claims to saved
  source-library citations. The matrix shows match scores, reasons, local file
  paths, and review actions before you accept a citation.
- Use **Insert conflict review** in Source Search & Deep Research to add a
  source-disagreement table before accepting contested findings or exporting a
  research report.
- Use **Insert source quality** to add a source review queue with fit scores,
  authority, recency, relevance, independence, and evidence-strength dimensions,
  ranking reasons, and recommended citation actions for every Deep Research
  source candidate before relying on generated claims.
- Use **Insert audit packet** to add one reviewer handoff containing run
  settings, search queries, source-quality bands, conflict review, generated
  bibliography records, citation index state, saved-source audit, and a final
  sign-off checklist.
- Choose `title`, `author-year`, `key`, `numeric`, or a common CSL alias such
  as `apa`, `chicago-author-date`, `ieee`, or `vancouver` from front matter,
  preferences, or the references panel.
- Use `glossary` fenced blocks for terms and definitions.
- Add `[GLOSSARY]` where the generated glossary should appear, or set
  `glossarySection: true` in front matter.
- Add `[INDEX]` where the generated index should appear, or set `index: true`
  / `index.enabled: true` in front matter.
- Use the Table of Contents manager to insert `[TOC]`, enable front matter TOC
  generation, choose the heading depth, and toggle numbered entries without
  editing YAML by hand.
- Use the Index manager to add explicit `{#index:Term}` markers, enable the
  generated index, exclude internal or irrelevant terms through `indexExclude`,
  and remove exclusions as the document moves toward release.
- Use labels and cross references for headings, figures, tables, equations,
  appendices, and decisions.
- Use the Cross References manager to review resolved and missing references,
  inspect the available label inventory, jump to source references or source
  labels, and insert reusable `{@label}` links without memorizing syntax.
- Use the Captions and Lists manager to insert lists of figures and tables,
  inspect tables, figures, and equations that need labels or captions, jump to
  their source, and insert cross-reference text for labeled items.
- Use the Document Variables manager to inspect scalar front matter values and
  merged project variables from `.neditor/variables.yaml`, insert filtered
  placeholders such as `{{projectLead | upper}}`, and add new front matter
  variables without editing YAML by hand.

The references panel surfaces resolved bibliography entries, missing citation
keys, duplicate bibliography keys, and citation TODO blockers. The Citation
TODO workflow can insert a source-review marker, jump to each unresolved item,
resolve it with a citation such as `[@porter1985]`, defer it with a reason, or
insert/copy a Markdown audit checklist for reviewers. It recognizes typed
phrases such as `needs citation` and cleanup comments such as
`TODO: citation needed`, so AI paste cleanup output flows into the same review
queue. Unsupported
citation-style warnings are reported with the rest of the document diagnostics.
The same panel can insert the bibliography marker, add a BibTeX template,
create stubs for missing keys, reinsert citation references, and copy resolved
entries as editable BibTeX blocks. Reuse of the same section, figure, table,
equation, appendix, or decision label is also reported as an export-blocking
reference diagnostic so links stay deterministic. Malformed labels and cross
references, including empty keys, spaces, slash characters, and unclosed
markers, are also export-blocking diagnostics with source ranges.

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
  handoff prompts. Suggested questionnaire answers now show the rationale and
  context signals behind each recommendation, and accepting a suggestion carries
  those notes into the editable answer block for review. The review packet
  summarizes context sources, the section work queue, assumptions to verify,
  cleanup checks, and reviewer ownership, and can be inserted or copied as a
  standalone audit handoff. Recent Docs Live drafts remain available locally for
  later append/copy and packet handoff.
- Use the Templates panel document builders when you want wizard assistance for
  proposals, RFPs, RFQs, tenders, tutorials, lesson content, textbooks, novels,
  podcast scripts, movie scripts, business cases, and executive briefs. Each
  builder now shows AI step assistance with suggested optimal answers, rationale,
  and context signals for identity, intent, outline approval, sequential
  drafting, QA, and humanization before the wizard opens Docs Live or prepares a
  local-agent handoff.
- Use Agent Workspace packets for multi-step document work that combines
  drafting, revision, review, and distribution. The packet records blockers,
  QA gates, and distribution gates before the output is accepted.
- Use provider handoff packages when a governed AI provider or local model
  gateway should continue an agent run. Review the package, redact sensitive
  facts where required, and import the response as a review draft. If direct
  execution is allowed, use a session-only key and apply the response only after
  checking the provider output.
- Use **Settings -> Setup wizard** when configuring NEditor for a new user or
  workstation. Each setup area now includes AI setup assistance with a suggested
  next answer, rationale, and context signals for identity, LLM access, local
  agents, voice runtime, read-aloud, exports, transforms, and release gates.
  The release setup area now mirrors the Release Evidence Dashboard, so blocked,
  stale, manual, credentialed, cross-platform, and ready-to-send lanes are
  visible from the same configurator used for AI, TTS, transforms, Google Docs,
  and exports. The same configurator also exposes **Support and diagnostics**,
  where users can preview or save a redaction-safe support bundle with setup
  diagnostics, release evidence summaries, spec work orders, transform health,
  and recommendations for help desks or internal IT. After previewing the
  bundle, use **Insert handoff** to add a redaction-safe support and release
  handoff to the active document with recommendations, release assignments,
  spec-closure work orders, release-candidate status, next commands, and
  privacy notes. Use **Insert evidence return packet** when external platform
  owners, credentialed operators, or human reviewers need exact return paths,
  recognized ingest filenames, validator commands, redaction rules, and closure
  commands for release evidence collection. Release managers can generate the
  same packet outside the app with
  `ned evidence-packet --output release-evidence-return-packet.md`, which is
  useful for support tickets, QA handoffs, and credentialed-host assignments.
  If the bundle includes manual-review spec work orders, use **Insert manual
  review kit** to add reviewer-ready sign-off templates, expected artifact
  paths, validator commands, and closure commands to the active document. Add
  the suggestion to editable setup notes when a non-technical user needs a
  clear setup record before changing configuration.
- For Claude Code, Codex, OpenCode, and Google Antigravity profiles, use
  **Prepare local agent workspace** to write the governed prompt package under
  `.neditor/agent-handoffs` in the active document folder. NEditor reports the
  exact launch command, whether the selected CLI is present on `PATH`, and the
  handoff file path so the local agent can work from the same project context.
  Each handoff also includes an expected response Markdown file path. Ask the
  local agent to write its final Markdown there, then use **Import local
  response** so NEditor can wrap it as needs-review AI material before you
  accept or apply it.
- For RFP responses, use the native RFP wizard in Templates to import or paste
  the source RFP, inspect stated and implied buyer intent, and review a
  requirement-by-requirement compliance matrix. Each compliance row includes a
  suggested response answer, evidence owner, verification note, and target
  response section. The full response draft also groups those answers into a
  **Requirement Response Drafts** section, so each response area has usable
  starter prose while the evidence owner and proof requirements remain visible.
  Use **Insert submission checklist** to add a bid-coordinator gate table for
  deadline, page-limit, attachments, annexes, language obligations, placeholder
  traps, scoring alignment, and evidence ownership before final packaging.
  The same wizard now builds evaluator-facing win themes from stated intent,
  implied intent, scoring criteria, risks, and capability signals; use
  **Insert win themes** when reviewers need a compact strategy table before
  drafting or red-team review.
  From the terminal, `ned rfp-response buyer-rfp.pdf --outline-output proposal-outline.md`
  writes the same pre-drafting packet as a standalone artifact: compliance
  checklist first, evaluator planning prompt second, and score-aware proposal
  outline third. Use `ned analyze-rfp - --outline` to print that packet from
  pasted or piped RFP text.
  Add `--coverage-output coverage.md`, or use `ned analyze-rfp - --coverage`,
  to produce a requirement coverage validator that checks mandatory rows,
  disqualification traps, evidence ownership, attachments, annexes, language
  obligations, placeholders, and final verification tasks before export.
  The RFP wizard also shows AI step assistance for source intake, requirement
  analysis, buyer intent, response drafting, evidence QA, and handoff; accepting
  a suggestion puts the answer, rationale, and context signals into editable
  response-context notes that flow into Docs Live, local-agent handoff, and full
  response drafts.
- Use `ai-source` blocks and AI-assisted section metadata so generated content
  can be marked as needing review or human-reviewed.

Export readiness reports unresolved comments, incomplete AI provenance,
missing approval metadata, missing captions or labels, broken references, dirty
Git state, and other issues that affect deliverability. The Export panel also
shows AI Export Assistance for target metadata, readiness diagnostics, and
artifact evidence; accepted guidance can be collected as editable export
readiness notes and inserted into the document as an audit handoff.

External distribution targets such as Blog, Substack, Google Docs, EPUB, and
HTML require release-grade metadata before NEditor writes or publishes the
handoff package. Set `status` to `approved` or `published`, add `approvedBy` or
`reviewer`, `approvedAt`, `owner`, `releaseTarget`, and a release-ready
`sourceConfidence` value such as `verified`, then resolve every review comment.
The Review panel and export metadata scaffold insert these fields with
`sourceConfidence: needs-review` so the document remains blocked until a human
explicitly upgrades the evidence state.

From the terminal, run
`ned quality report.md --markdown --output quality-review.md` when a reviewer,
release manager, or automation job needs the same kind of deterministic QA
handoff outside the app. The report checks compiler diagnostics, unresolved
placeholders, citation evidence, citation TODOs, review comments, AI provenance,
document identity, heading structure, long paragraphs, generic AI phrasing,
wide tables, and multi-column layout reset risks. Add `--json` for automation
or `--strict` when risks and improvements should fail the command.

Use **Print Preview** from the document toolbar, View menu, command palette, or
Export panel when page geometry matters. The preview estimates page count,
word count, page breaks, section breaks, paper size, orientation, margins, and
column flow. It also warns about wide portrait tables, long documents without
explicit page breaks, and multi-column layouts that need final PDF or DOCX
review before distribution.

The Export panel includes an **Export Visual QA Dashboard** that turns current
readiness, manifest, diagnostics, print-preview, asset, include, transform, and
output-proof signals into target-by-target status rows. Use it before external
handoff to see which targets are ready, blocked, still need visual review, or
have not been run. The same dashboard can be inserted into the document as a
Markdown reviewer handoff from the Export menu, Quality menu, command palette,
or the dashboard itself.

## Transform Engines

NEditor uses native Rust renderers or static fallbacks where practical.
External diagram and database engines are optional and must be explicitly
trusted.

External engines include Graphviz/DOT, D2, PlantUML, Pikchr, and SQLite
`sqlite3` for read-only SQL table transforms. Configure exact executable paths
in settings, probe the engine, and trust only the engine you intend to use.
Execution is bounded by timeout and output-size limits, and no engine is
invoked through interpolated shell strings.

Open **Settings -> Transforms -> Download and install transform handlers** to
install or copy setup commands for the full handler set. The configurator shows
the package manager, platform, exact commands, covered handlers, and privilege
expectations before it starts an allowlisted installer.

For release evidence, optional engines can be proven on another workstation.
Run `pnpm run collect:engine-evidence` on that workstation, using
`NEDITOR_TEST_*=/absolute/path/to/engine` when an executable is not on `PATH`.
Then return the generated engine proof such as
`external-engines/external/pikchr.json` or
`external-engines/external/sqlite.json`, and import it with
`pnpm run ingest:evidence`; NEditor validates the returned proof through the
same external-engine checker before release readiness accepts it.

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
- LaTeX source for academic, technical, business report, proposal, RFP
  response, textbook, and long-form manuscript handoff.
- Google Docs package for local-first import through DOCX, HTML, Markdown, or
  text fallbacks.
- EPUB ebook for portable long-form reader distribution.

Export options cover manifests, styles, syntax highlighting, HTML language,
HTML description, canonical URL, cover pages, page numbers, layout presets,
LaTeX template profiles, comments appendix, AI provenance appendix, glossary
appendix, PPTX agenda, citation style, brand profile, dirty-Git warnings,
transform engine settings, and draft watermark behavior.

When the target is LaTeX, choose a template profile in the Export panel before
writing the `.tex` file. The current built-ins cover article, business report,
proposal, RFP response, technical report, academic paper, textbook, and book
layouts. Use **Manage company LaTeX templates** in the same panel to add
organization, publisher, or client house styles from a larger LaTeX template
library. A saved custom profile records the document class, class options,
package/preamble lines, geometry, hyperref setup, optional header preamble,
chapter-style heading preference, and best-fit notes. NEditor stores the
profile locally, makes it available in the same template picker as the built-ins,
records the selected profile in saved export profiles, validates it before
export, and emits template-specific document classes, packages, margins,
headers, and heading levels in the generated `.tex` file.

For large external LaTeX libraries, create one custom profile per reusable
house style instead of pasting an entire archive into a document. Keep class
names in the class field, one package or preamble line per package row, and
house-style page/header commands in the header field. This keeps Markdown
portable while still letting the export stage produce TeX that matches client,
publisher, proposal, textbook, or internal report conventions.
The same panel can sync custom profiles with `.neditor/latex-templates.json`,
show the portable library JSON, and import JSON libraries created by
`ned latex-templates --export-library`. This lets proposal, publishing, and
technical-writing teams keep a reviewed LaTeX template library under project
control while still selecting those profiles from the app.
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
NEditor. For teams that publish through an internal CMS bridge, static-site
automation, WordPress REST endpoint, or Ghost Admin proxy, the toolbar
**Publish** action and the **Export** -> **Publishing** menu open the same
**Publish and distribute** handoff. It builds a reviewable JSON payload, keeps
endpoint tokens session-only, supports dry-run preview before sending, and
blocks unsafe non-local HTTP endpoints so business users can post through an
approved publishing bridge without copying files by hand.
Use **Insert preflight** to add a publishing audit section before sending. It
records endpoint safety, dry-run state, public metadata, content payload
availability, secret handling, target workflow caveats, and the exact blockers
or review items that remain.
The packaged CLI exposes the same handoff for automation:

```sh
ned publish document.md --target blog --destination wordpress-rest --endpoint https://cms.example.com/wp-json/wp/v2/posts --token-env CMS_TOKEN --output publish-payload.json --json
```

Reusable business identity values live in `.neditor/business-profile.json`.
Use the **Business Identity** dialog or this CLI form:

```sh
ned profile --workspace . --set companyName=Acme --set companyLegalName="Acme LLC" --set taxIdentifier=VAT-999 --set credentialsSummary="Approved supplier" --json
```

This saves repeated sender, company, legal-name, registration, tax, DUNS, country,
website, LinkedIn, credential, certification, disclaimer, client, and brand
voice facts. Templates and snippets can then use placeholders such as
`{{companyLegalName}}`, `{{taxIdentifier}}`, `{{credentialsSummary}}`,
`{{certifications}}`, and `{{legalDisclaimer}}`; run
`ned profile --workspace . --placeholders` to inspect the available values.

Repeated delivery settings can be saved as project-local export profiles under
`.neditor/export-profiles.json`. Use
`ned export-profiles --workspace . --save client-html --target html --layout-preset business --brand name=Client --citation-style apa --json`
to create a reusable client or channel profile, then run
`ned export-profiles --workspace . --apply client-html --document document.md --output-dir exports`
to export with the saved target, layout, citation style, manifest setting, and
brand defaults. Add `--dry-run --json` to preview the resolved options and
output path before writing an artifact.

The payload references the token environment variable but never stores the
token value.
Google Docs package exports are ZIP files with `document.docx` as the primary
Google Docs upload/import file, plus HTML, Markdown, plain text, metadata,
assets, and an embedded manifest for auditability. Their metadata names
`document.docx` as the primary import file, records HTML/Markdown/plain-text
fallbacks, and includes the Google Docs import workflow. LaTeX exports write a
direct `.tex` file and use the regular sidecar manifest when `includeManifest`
is enabled.
For live Google Docs actions, open **Settings** -> **Google Docs**, add a
desktop OAuth client ID, and choose **Sign in with Google**. NEditor opens the
system browser, receives the loopback callback locally, and keeps Google tokens
in session memory rather than writing them to workspace preferences. Leave
**Request session refresh** enabled when longer review sessions may need to
renew an expired access token without another browser sign-in. After sign-in,
choose **Import current document** to upload the active document as a Google
Docs file, then choose **Read back** to export plain text from the created Google
Doc for verification.
EPUB exports write a direct `.epub` package with the required `mimetype`,
container file, OPF metadata, navigation document, XHTML body, stylesheet, text
fallback, packaged media, and embedded NEditor manifest. Use **Export EPUB**
from the Export panel, document toolbar, command palette, or native
`File` -> `Export` menu when the document is ready for ebook review.

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

## Release Evidence Dashboard

Open the **Review** panel or choose **Quality** -> **Insert Release Evidence
Dashboard** to review whether a document is complete, blocked, manual,
credentialed, cross-platform, stale, or ready to send.

The dashboard combines release metadata, review comments, AI provenance,
export readiness, visual QA, citation vault integrity, unresolved citation
TODOs, accessibility QA, Google credential state, platform package evidence,
Homebrew evidence, signing evidence, notarization evidence, Git cleanliness,
and evidence freshness into one release-manager view. Use **Setup release
evidence** to open the consolidated configurator, or **Insert evidence
dashboard** to add the current release evidence table to the active Markdown
document for review packets and stakeholder handoff.
The same release view now derives **Production readiness work orders** with
priority, owner, command, and acceptance evidence for every open lane. Use
**Insert work orders** when the release owner needs assignable next steps
instead of a passive status summary. From the terminal, use
`ned evidence-packet --output release-evidence-return-packet.md` to generate a
Markdown packet that combines release work items, spec/manual work orders,
recognized ingest paths, validator commands, redaction rules, and final closure
commands.

Use `ned improvements --output improvement-coverage.md` to turn
`docs/100-improve.md` into a conservative item-by-item coverage audit. The same
100-improvement summary appears in support bundles and the Configuration
Center support panels, so release managers can see how many product promises
have implementation evidence, how many still depend on external/manual proof,
and which items still need implementation evidence before anyone claims the
platform is complete.

NEditor stays conservative: credentialed integrations, cross-platform package
smoke checks, Homebrew audit output, signing proof, and notarization proof must
be backed by explicit evidence before the dashboard can mark a package ready to
send.

## Accessibility QA

Open **Settings** -> **Appearance** to review the in-app **Accessibility QA**
panel. It summarizes keyboard command access, skip links, screen-reader
landmarks, hover/focus button help, status feedback, plain-language help, high
contrast, reduced motion, and manual assistive-technology sign-off readiness.

Use **Open accessibility QA** from the command palette or Quality menu when a
document is heading to external review. The panel can switch on high contrast,
switch on reduced motion, and insert an **Accessibility QA Report** into the
active Markdown document so a reviewer or release manager can see exactly what
has been checked and what still requires manual screen-reader sign-off.

## Troubleshooting

- If a local link or image is missing, check the diagnostics panel and export
  readiness report.
- If a save is blocked, the file changed on disk after the editor last read it.
  Use the conflict workflow instead of forcing an overwrite.
- If an external transform is disabled, configure the executable path, probe
  it, and trust the engine explicitly.
- If an external transform fails differently by platform, open Help and search
  for "external transform troubleshooting". The in-app guide covers macOS
  quarantine/executable permissions, Windows `.exe` paths and package-manager
  shims, Linux executable bits/package paths, PlantUML file mode, timeouts,
  empty output, disabled trust, and stale cache evidence.
- If an export is blocked, open export readiness and fix errors before writing
  the artifact.
- If Playwright browser workflows cannot run locally, install the browser
  dependencies on a host that allows Chromium to launch. NEditor does not use
  GitHub Actions as the current verification surface.
- If macOS DMG creation fails with `Device not configured`, use the `.app`
  bundle or `tauri build --no-bundle` proof in restricted environments.
