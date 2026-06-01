# NEditor Vision, Capabilities, Business Affordances, And Future Direction

Updated: 2026-06-01

## Executive Summary

NEditor is a local-first, AI-first business document workbench. Its purpose is
to help serious document creators move from raw ideas, source material, RFPs,
research, spreadsheets, dictated instructions, and reusable business knowledge
to polished, governed, export-ready documents.

NEditor is not merely a Markdown editor. It uses Markdown as the durable source
format, then layers on the capabilities that business documents need but plain
Markdown normally lacks: outlines, document composition, rich tables,
calculations, citations, equations, multi-column layouts, reusable business
profiles, AI-guided drafting, compliance checklists, RFP response workflows,
deep research, review governance, publishing packages, export manifests,
release evidence, and a command-line interface for automation.

The strategic product thesis is simple:

> A modern business document should be a living, evidence-backed, reusable,
> governed artifact, not a fragile binary file assembled manually at the last
> minute.

NEditor gives users the speed and portability of Markdown while adding the
business affordances of a proposal factory, research assistant, compliance
analyst, data workbench, document design system, export pipeline, and release
control room.

## Product Vision

NEditor's long-term vision is to become the primary local-first environment for
creating high-value business, technical, research, procurement, educational,
and publishing documents.

The product should make the following outcomes routine:

- A consultant can create a proposal, RFP response, compliance matrix, budget,
  win-theme narrative, and final submission package from one source workspace.
- A business analyst can combine narrative, tables, SQL results, charts,
  equations, citations, source documents, and export-ready appendices without
  leaving the document.
- A non-technical executive can dictate what they want, answer guided business
  questions, approve an outline, and receive a first draft that is structured,
  reviewable, and tied to evidence.
- A technical writer can maintain large modular manuals from many included
  Markdown files with table of contents, glossary, index, cross references,
  export profiles, and release manifests.
- A researcher can run deep research, collect sources into a document-associated
  directory, populate a bibliography, track claims, and produce a report whose
  factual assertions are auditable.
- A procurement team can ingest an RFP from PDF, DOCX, Markdown, text, or URL,
  extract mandatory requirements, surface stated and implied buyer intent, build
  a compliance checklist, generate a proposal outline, draft section by
  section, and verify requirement coverage before submission.
- A publishing user can prepare HTML, PDF, DOCX, PPTX, EPUB, LaTeX, blog,
  Substack, Google Docs, and Markdown bundle outputs from one governed source.

The core promise is not "write Markdown faster." The core promise is:

> Create better business documents with less coordination drag, stronger
> evidence, clearer review, safer AI assistance, and more reliable export.

## Guiding Principles

### 1. Local-First Ownership

NEditor should preserve user ownership of files. The Markdown source, includes,
assets, downloaded sources, bibliography files, data sources, export manifests,
and evidence packets should remain understandable and movable. Cloud services
may assist, but they should not become the source of truth.

### 2. Markdown As Durable Source

The authoring format should remain readable plain text. NEditor extensions
should degrade safely in other editors through front matter, fenced blocks,
explicit markers, comments, and ordinary Markdown patterns.

### 3. AI As Governed Co-Worker

AI should help users think, structure, draft, revise, research, cite, and
prepare documents. It should not silently replace human ownership. AI output
should carry provenance, assumptions, source references, review state, and
quality gates.

### 4. Evidence Over Assertion

Important document claims, release claims, compliance claims, and export claims
should be backed by inspectable evidence: source files, citations, manifests,
hashes, diagnostics, review notes, or signoffs.

### 5. Documents As Business Workflows

A proposal, board paper, RFP response, lesson plan, textbook, novel, or policy
brief is not just text. It has intent, audience, deadlines, owners, source
confidence, reviewers, approvals, distribution targets, formats, and reuse
patterns. NEditor should model those workflows explicitly.

### 6. Professional Output Without Fragility

Beautiful documents should not depend on manual copy-paste. Layout presets,
brand kits, covers, headers, footers, page numbers, multi-column sections,
callouts, citations, equations, figures, captions, tables, and export profiles
should travel through a repeatable pipeline.

### 7. Automation Without Excluding Business Users

Power users should have the `ned` CLI for automation, CI, support bundles,
conversion, validation, and release evidence. Non-technical users should access
the same capabilities through menus, buttons, wizards, help overlays, and
guided setup.

## Target Constituents

NEditor creates value for several distinct user groups.

### Consultants And Professional Services Teams

Consultants need to produce proposals, statements of work, RFP responses,
client reports, strategy papers, implementation plans, financial models, and
board-ready deliverables. They need speed, reuse, brand consistency, proposal
discipline, compliance confidence, and export polish.

### Procurement And Business Development Teams

Business development teams need RFP ingestion, checklist extraction, compliance
matrices, buyer intent analysis, win-theme development, response outlines,
attachment tracking, deadline visibility, and requirement coverage validation.

### Executives And Managers

Executives need decision papers, operating reviews, strategy memos, policy
briefs, risk registers, investor updates, and board packs. They need documents
that are concise, structured, well-evidenced, easy to review, and easy to
export.

### Technical Writers And Product Teams

Technical writers need modular documentation, includes, cross references,
tables of contents, glossaries, indexes, diagrams, versioned snippets, release
notes, architecture docs, API docs, and export bundles.

### Researchers And Analysts

Researchers need citation management, bibliography generation, source
collection, claim inventories, evidence conflict detection, deep research,
tables, equations, charts, and reproducible analysis.

### Educators And Content Creators

Educators and creators need lesson plans, course content, tutorials, technical
textbooks, novels, podcast scripts, movie scripts, outlines, plots, sequential
drafting, editorial review, and export to reader-friendly formats such as EPUB,
PDF, DOCX, HTML, and publishing packages.

### Support, IT, And Release Operators

Operational teams need installability, diagnostics, CLI deployment,
configuration guidance, support bundles, release evidence dashboards, platform
proof, signing gates, Homebrew packaging, and reproducible builds.

## Capability Architecture

NEditor can be understood as a set of connected capability layers.

### 1. Authoring Workbench

The authoring workbench provides the day-to-day writing surface:

- Markdown source editor.
- Live preview.
- Split, source, preview, focus, outline, export, review, and presentation
  modes.
- Folding for headings, lists, fenced blocks, tables, includes, comments, and
  generated sections.
- Search and replace.
- Document outline.
- Editable outline mode for chapter, section, subsection, and subsubsection
  CRUD.
- Multi-toolbar command groups.
- Collapsible toolbars that recover vertical writing space.
- Restore controls after collapsed toolbar and maximized writing states.
- Menus and command palette for every major capability.
- Button display preferences for icons, text, or both.
- Resizable and density-aware button labels.
- Keyboard-first access to major workflows.
- Status, diagnostics, word count, preview timing, and readiness signals.

### 2. Document Model And Markdown Extensions

NEditor extends Markdown while keeping source files readable:

- YAML front matter for identity, metadata, governance, brand, layout, export,
  citation style, and variables.
- Variables such as `{{client}}` and formatted values.
- Inline formulas such as `{{=profit / revenue | percent}}`.
- Master-document includes.
- Generated sections such as `[TOC]`, `[BIBLIOGRAPHY]`, `[INDEX]`, glossary,
  figure lists, and table lists.
- Layout directives for columns, page breaks, orientation, margins, and section
  flow.
- Callouts for decisions, recommendations, risks, warnings, assumptions,
  evidence, actions, and notes.
- Calculation fences.
- CSV, TSV, JSON, YAML, and XLSX data flows.
- SQL transforms.
- Mermaid, PlantUML, D2, Graphviz, and related diagrams.
- Chart and visual-data fences.
- Citations and bibliography blocks.
- Cross references.
- Captions and labels.
- Equations with labels and captions.
- Glossary and index terms.
- Review comments and change notes.
- AI provenance blocks.

### 3. AI-First Creation System

The AI creation system turns NEditor from a blank-page editor into an adaptive
document partner:

- Intent-first document launcher.
- Adaptive document creation wizard.
- AI-generated questionnaires.
- Suggested optimal answers for non-expert users.
- Document-type playbooks for proposals, RFPs, RFQs, tenders, tutorials,
  lesson plans, lesson content, technical textbooks, novels, podcast scripts,
  movie scripts, business cases, executive briefs, policies, SOPs, and more.
- Outline-first generation.
- Plot-first and architecture-first workflows for novels and textbooks.
- Section-by-section drafting queues.
- Context-aware placeholder extraction.
- Reusable business profile merge.
- AI quality passes for structure, evidence, clarity, tone, compliance, and
  export readiness.
- Humanization controls to remove generic AI phrasing.
- Reviewer-agent roles for strategy, editing, evidence, risk, citations,
  governance, and export readiness.
- Agent task board showing blocked, ready, needs-review, and completed work.
- Provider-agnostic runtime for approved LLM routes, including local Ollama
  endpoints and external providers.

### 4. Business Knowledge And Reuse

NEditor should reduce repeated typing and repeated decision-making:

- Reusable business profile for names, email, company, address, website, tax
  identifiers, credentials, legal disclaimers, brand voice, and common facts.
- Snippet library for executive summaries, contact blocks, scope sections,
  assumptions, pricing notes, risks, governance, review handoffs, bios,
  disclaimers, and procurement clauses.
- Versioned reusable clauses with stale-language warnings.
- Template catalog for standard business and creative documents.
- Template metadata, placeholders, outline rules, best-fit usage, and examples.
- Workspace memory for terminology, tone, accepted boilerplate, reviewers, and
  recurring claims.
- Brand kit manager for colors, fonts, logos, covers, watermarks, headers,
  footers, page numbers, and export defaults.
- Export profiles per client, workspace, document type, or target.

### 5. RFP And Procurement System

NEditor should be especially strong for procurement work:

- Native RFP ingestion from PDF, DOCX, Markdown, plain text, and URL sources.
- Extraction of deadlines, page limits, submission requirements, scoring
  weights, mandatory requirements, pass/fail gates, disqualification language,
  required attachments, annexes, team roles, language requirements, technical
  mandates, and verification methods.
- Compliance checklist at the front of the proposal after the cover and before
  the table of contents.
- Compliance matrix with requirement, response section, owner, status,
  evidence, verification method, and source reference.
- Critical disqualifier panel.
- Buyer stated and implied intent analysis.
- Score-maximizing proposal outline before drafting.
- Section-by-section response drafting.
- Attachment and annex tracker.
- Requirement coverage validator.
- Win-theme builder.
- Evidence and capability mapping.
- Final submission readiness checks.

### 6. Deep Research And Citation System

NEditor should support evidence-rich research and long-form reports:

- Deep research planning.
- Search provider choice, including SearXNG, DuckDuckGo, Tavily, and private or
  local indexes.
- Downloaded source-document vault stored beside the document.
- Claim inventory for unsupported assertions.
- Citation TODO workflow.
- Bibliography population from source documents and URLs.
- Research-length control from one-page brief to long report.
- Iterative expansion until target coverage and length are met.
- Source quality scoring.
- Evidence conflict detection.
- Research audit packet with queries, sources, downloaded files, decisions, and
  bibliography state.

### 7. Tables, Data, And Computation

NEditor should make business documents computational without turning them into
opaque spreadsheets:

- Two-way table editor that synchronizes Markdown source and visual grid.
- Table CRUD in source and grid.
- CSV and TSV import/export.
- XLSX import to Markdown tables.
- Markdown table export to CSV and XLSX.
- Formula-aware tables.
- Summary rows.
- Currency, percentage, date, and numeric formatting.
- Business calculation templates such as ROI, NPV, IRR, margin, CAGR,
  utilization, break-even, pricing, forecast, budget, and weighted scoring.
- Scientific and mathematical templates for matrices, statistics, regression,
  units, probability, physics, engineering, and optimization.
- SQL transform for safe read-only queries.
- Safe database profiles that avoid storing secrets in documents.
- Data refresh workflow with audit records.
- Chart generation from tables and local data sources.

### 8. Layout And Beautiful Documents

NEditor should help users craft documents that look professional without
manual layout surgery:

- Multi-column layouts.
- Section-specific page layout.
- Page breaks.
- Landscape appendices.
- Callout and admonition styles.
- Professional cover builder.
- Brand-aware headers, footers, watermarks, and page numbers.
- Figure, table, and equation numbering.
- Cross-referenceable labels.
- Equation editor with LaTeX source round-trip.
- LaTeX template integration.
- Page design presets for board memos, consulting reports, academic articles,
  book chapters, proposals, newsletters, and policy briefs.
- Print preview for pagination, margins, columns, headers, footers, and page
  breaks.
- Export visual QA dashboard.

### 9. Export, Publishing, And Distribution

NEditor should make distribution repeatable:

- HTML export.
- PDF export.
- DOCX export.
- PPTX export.
- EPUB export.
- LaTeX export.
- Markdown bundle export.
- Google Docs import handoff and live readback evidence when authenticated.
- Blog and CMS publishing packages.
- Substack package export.
- Static site bundle.
- WordPress, Ghost, webhook, and manual publishing routes.
- Target-specific export readiness.
- Export manifests with hashes, options, source metadata, and provenance.
- Sidecar manifests.
- Brand-aware export profiles.
- Distribution preflight checks.
- Rendered export visual QA.

### 10. Voice, TTS, And Accessibility

NEditor should support users who think by speaking, users who review by
listening, and users who require keyboard or assistive technology access:

- Voice command interface for document changes.
- Voice-first document wizard.
- Dictation-driven outlines, drafts, revisions, and workflow commands.
- Voice correction loop.
- Read selected text or entire documents aloud.
- OS-native TTS such as macOS Say.
- Optional external TTS engines such as Supertonic.
- Consent-gated model downloads with model name, size, source, and storage
  location.
- Screen-reader QA mode.
- Accessible command palette.
- Keyboard-first table editing.
- High-contrast mode.
- Reduced-motion mode.
- Plain-language help overlays.
- Guided demo system.
- Hover and focus help for buttons and controls.

### 11. Productization, Setup, And Release

NEditor should be usable by non-technical users and supportable by operators:

- Unified configurator for AI providers, TTS, transforms, Google auth, CLI
  setup, templates, export defaults, external engines, and release gates.
- Guided provider setup for OpenAI-compatible endpoints, Ollama, Claude Code,
  Codex, OpenCode, Google integrations, local gateways, and private-network
  model gateways.
- Ollama endpoint and model selection.
- Transform handler installer.
- `ned` CLI for opening, creating, inspecting, validating, converting,
  exporting, publishing, profile management, template discovery, evidence
  packets, support bundles, and release readiness.
- Deploy CLI menu item.
- Default Markdown reader setup.
- Homebrew release path.
- Signed and notarized release workflow.
- Cross-platform package evidence.
- Release evidence dashboard.
- Evidence kit collection and ingestion.
- Support bundle generation.
- In-app tutorial and capability showcase.

## Specific Business Affordances

The following business affordances are the concrete value NEditor creates. They
are written as "NEditor lets the business..." statements because affordances
should describe what becomes possible or easier for the user.

| # | Business affordance | What it enables |
| ---: | --- | --- |
| 1 | Start from business intent instead of a blank page | Documents begin with outcome, audience, owner, deadline, source confidence, approval path, and distribution target. |
| 2 | Reuse organizational identity | Company names, addresses, legal names, websites, tax IDs, contacts, credentials, disclaimers, and brand voice are inserted consistently. |
| 3 | Keep standard language current | Versioned snippets and clauses reduce stale boilerplate and inconsistent proposal language. |
| 4 | Turn outlines into execution plans | Users can create a document outline first, then use it as a drafting and review queue. |
| 5 | Create documents by voice | Non-technical users can dictate document intent, context, and desired changes. |
| 6 | Ask better setup questions | AI-generated questionnaires ask only missing high-value questions based on document type and context. |
| 7 | Reduce blank-page anxiety | Suggested optimal answers help users move forward when they do not know what to write. |
| 8 | Draft systematically | Section-by-section drafting avoids one-shot AI output that is hard to review. |
| 9 | Separate strategy from prose | Proposal strategy, evidence review, risk review, citation review, and export review become explicit lanes. |
| 10 | Humanize AI output | Generic AI language is detected and replaced with specific, evidence-grounded writing. |
| 11 | Preserve AI accountability | AI provenance, provider, prompt summary, assumptions, and review state remain visible. |
| 12 | Improve document quality before review | Quality recommendations catch weak structure, missing evidence, placeholders, long paragraphs, and unresolved comments. |
| 13 | Govern external distribution | Approval metadata gates prevent accidental distribution of draft or unapproved documents. |
| 14 | Build board-ready documents | Covers, headers, footers, page numbers, watermarks, callouts, and export profiles produce executive-ready packs. |
| 15 | Make RFP response work repeatable | RFP ingestion, checklist extraction, matrix generation, outline drafting, and coverage validation form a repeatable response process. |
| 16 | Avoid procurement disqualification | Mandatory requirements, automatic-exclusion language, deadlines, annexes, and attachments are surfaced early. |
| 17 | Align response pages to scoring | Scoring weights and page allocations guide proposal structure before prose is drafted. |
| 18 | Make buyer intent explicit | Stated and implied intent analysis helps teams respond to what the buyer values, not only what they wrote. |
| 19 | Track compliance requirement by requirement | Compliance matrices make every requirement assignable and verifiable. |
| 20 | Put the checklist where reviewers need it | The compliance checklist appears near the front of the proposal package. |
| 21 | Reuse proposal themes and differentiators | Win-theme building turns RFP priorities into executive-summary framing. |
| 22 | Manage attachments and annexes | CVs, declarations, schedules, audited accounts, certificates, and required forms are tracked. |
| 23 | Move faster on bids | Teams can go from RFP source to response outline and draft faster with less manual triage. |
| 24 | Create evidence-backed reports | Claims are linked to citations, source files, or unresolved citation TODOs. |
| 25 | Keep sources with the document | Downloaded source documents live beside the Markdown source, improving auditability and handoff. |
| 26 | Populate bibliographies automatically | Source metadata becomes BibTeX or CSL-style bibliography records. |
| 27 | Detect unsupported claims | Claim inventory makes unsupported assertions visible before publication. |
| 28 | Detect evidence conflicts | Conflicting sources or weak support can be flagged before the document reaches a decision-maker. |
| 29 | Scale research deliverables | Users can target anything from a one-page brief to a long-form research report. |
| 30 | Preserve research audit trails | Queries, sources, downloaded files, and evidence decisions are exportable. |
| 31 | Write modular long documents | Includes let teams compose master documents from chapters, appendices, and reusable components. |
| 32 | Recompile when included files change | External edits and included-file changes can refresh the master document safely. |
| 33 | Avoid destructive file conflicts | External file changes can be compared, accepted, kept local, merged, or saved as a copy. |
| 34 | Navigate large documents | Outlines, document maps, citations, figures, tables, equations, comments, includes, and TODOs become navigable. |
| 35 | Edit structure before prose exists | Outline mode lets users design the document skeleton first. |
| 36 | Manage multiple documents | Tabs, groups, pinned files, recents, and workspace restore support real project work. |
| 37 | Keep writing space available | Collapsible toolbars, focus mode, and restore controls protect screen real estate. |
| 38 | Serve different user preferences | Icons, text, both, density, button label size, themes, typography, and accessibility preferences adapt to the user. |
| 39 | Make every feature discoverable | Menus, buttons, command palette, hover help, guided demo, and help system expose capabilities. |
| 40 | Keep Markdown tables editable | Two-way table editing lets users work in source or grid without losing Markdown fidelity. |
| 41 | Bridge documents and spreadsheets | CSV/XLSX import and export connect business documents to spreadsheet workflows. |
| 42 | Insert database-backed evidence | SQL transforms can bring safe read-only query results into the document. |
| 43 | Avoid secret leakage in documents | Database and provider profiles avoid storing credentials in Markdown. |
| 44 | Make calculations reusable | Business and scientific calculation templates reduce manual formula errors. |
| 45 | Produce board charts from source data | Chart transforms and data-source workflows turn tables into visual evidence. |
| 46 | Refresh stale data with audit | Data refresh records help reviewers know what changed and when. |
| 47 | Support equations for technical work | Equation editing and LaTeX round-trip make math first-class. |
| 48 | Support cross references | Figures, tables, equations, headings, decisions, and appendices can be referenced reliably. |
| 49 | Create beautiful multi-column documents | Layout directives and export mapping enable newsletters, policy briefs, reports, and proposals. |
| 50 | Use professional LaTeX ecosystems | Markdown can be mapped into curated LaTeX templates and classes. |
| 51 | Preview print layout before export | Print preview reduces surprises in pagination, margins, columns, and page breaks. |
| 52 | Export to many targets | HTML, PDF, DOCX, PPTX, EPUB, LaTeX, Markdown bundles, blog, Substack, and Google Docs handoffs support real distribution. |
| 53 | Use one source for many outputs | Export profiles keep output-specific settings repeatable without copying content into separate files. |
| 54 | Package documents with manifests | Bundle exports include source, includes, assets, bibliography, hashes, and provenance. |
| 55 | Verify output visually | Export QA helps compare rendered artifacts against intended structure. |
| 56 | Publish without messy copy-paste | Publishing packages prepare metadata, HTML, plaintext, previews, and checklists. |
| 57 | Read documents aloud | TTS supports listening review, accessibility, and long-document proofreading. |
| 58 | Gate external model downloads | TTS models are downloaded only after explicit consent with clear size and storage details. |
| 59 | Support keyboard-only workflows | Command palette, shortcuts, focus management, and accessible controls support power and assistive users. |
| 60 | Help non-technical users install and configure | Unified setup guides AI providers, transforms, CLI deployment, default reader setup, and support bundles. |
| 61 | Automate from the terminal | `ned` enables open, create, convert, validate, export, profile, template, source, research, RFP, readiness, and support workflows. |
| 62 | Make support diagnosable | Support bundles collect setup diagnostics, evidence status, spec gaps, and environment details. |
| 63 | Make release readiness explicit | Release dashboards show complete, blocked, stale, manual, credentialed, and ready-to-send lanes. |
| 64 | Support cross-platform deployment | Tauri packaging, platform evidence, sidecar CLI, Homebrew path, and signing gates support real distribution. |
| 65 | Make demos concrete | A shipped capability showcase demonstrates tables, equations, images, citations, AI workflows, and export paths. |

## Business Workflows NEditor Should Own

### Proposal And RFP Response Workflow

1. Ingest the RFP or opportunity brief.
2. Extract metadata, deadlines, page limits, evaluation model, scoring weights,
   mandatory gates, and attachments.
3. Extract compliance requirements from prose, tables, annexes, and
   disqualification language.
4. Analyze stated and implied buyer intent.
5. Build a critical disqualifier checklist.
6. Build a front-of-proposal compliance checklist.
7. Build a response compliance matrix.
8. Generate a score-optimized proposal outline.
9. Approve the outline before drafting.
10. Draft sections sequentially.
11. Insert required evidence, team qualifications, project examples, and
    attachments.
12. Run requirement coverage validation.
13. Run proposal quality review, humanization, and export readiness.
14. Export the final response, matrix, checklist, and attachments.

### Board Paper Workflow

1. Start from decision required, audience, meeting date, owner, and approval
   status.
2. Insert executive summary, recommendation, options, risks, financial impact,
   implementation plan, and decision request.
3. Pull data from tables, spreadsheets, SQL, or calculations.
4. Add evidence-backed claims and citations.
5. Apply board-paper layout preset and brand kit.
6. Run quality, risk, unresolved-comment, and approval metadata gates.
7. Export PDF/DOCX/HTML package with manifest.

### Research Report Workflow

1. Define topic, research questions, intended audience, confidence threshold,
   source providers, and target length.
2. Search through SearXNG, DuckDuckGo, Tavily, or private indexes.
3. Download sources into the document source vault.
4. Score sources.
5. Build claim inventory and bibliography.
6. Generate outline.
7. Draft section by section.
8. Detect unsupported claims and source conflicts.
9. Generate bibliography, appendices, audit packet, and final exports.

### Technical Manual Workflow

1. Define product, audience, version, release state, and export targets.
2. Create a master document with includes.
3. Maintain chapters, references, glossary, index, diagrams, equations, and
   screenshots.
4. Watch includes and recompile the master document.
5. Generate table of contents, glossary, index, and cross references.
6. Run diagnostics and export readiness.
7. Export HTML, PDF, DOCX, EPUB, and Markdown bundle artifacts.

### Educational And Creative Workflow

1. Select lesson plan, lesson content, tutorial, textbook, novel, podcast, or
   movie script.
2. Use a document-type wizard that asks the correct domain decisions.
3. Create outline, plot, textbook architecture, or episode structure first.
4. Approve the structure.
5. Draft sequentially by section, chapter, scene, episode, or lesson.
6. Run domain-specific quality review.
7. Humanize, revise, and package for review or publication.

## Strategic Differentiation

NEditor can be differentiated from ordinary document tools in several ways.

### Compared With Word Processors

NEditor keeps the source plain, portable, diffable, scriptable, and compatible
with Git while still providing business document affordances such as brand
profiles, review gates, citations, tables, and export packages.

### Compared With Markdown Editors

NEditor treats documents as business workflows, not just Markdown preview
buffers. It adds AI drafting, RFP compliance, deep research, snippets,
variables, data transforms, multi-target exports, release evidence, and guided
setup.

### Compared With AI Chat

NEditor gives AI a governed workspace: outlines, sources, placeholders,
section queues, provenance, citation tasks, review gates, and export readiness.
It avoids the common failure mode where AI produces an impressive but
unverifiable blob of text.

### Compared With Notebooks

NEditor supports calculations, tables, SQL, charts, diagrams, and data sources
inside a document that remains readable as business prose. It is document-first
rather than code-cell-first.

### Compared With Cloud Knowledge Tools

NEditor remains local-first. The user's files, sources, evidence, and exports
can live in normal folders and repositories. Integrations are useful but not
mandatory for ownership.

## Future Directions

The future direction should be ambitious but disciplined. The highest-value
path is to deepen NEditor's strengths instead of becoming a generic office
suite.

### Direction 1: Make AI Document Generation Truly Agentic

NEditor should evolve from "AI-assisted drafting" to a governed document
production system.

Priorities:

- Stronger multi-agent orchestration for strategist, drafter, editor, evidence
  reviewer, compliance reviewer, citation reviewer, and export reviewer.
- Better context packing for large documents and large source packs.
- Explicit per-section acceptance criteria.
- Automated section queue execution with pause, resume, rollback, and human
  approval.
- Reviewable AI run history.
- Provider interchangeability across local and cloud endpoints.
- Local model workflows that do not require cloud services for sensitive
  documents.
- Better natural-language command routing from command palette and voice.

### Direction 2: Build The Best RFP Response System Available

RFP response is one of the strongest business opportunities.

Priorities:

- Robust extraction from real-world PDFs, DOCX files, scanned documents, and
  complex tables.
- Better table parsing for roles, requirements, scoring, attachments, and
  compliance gates.
- Stronger implied-intent analysis.
- Requirement deduplication and cross-reference consolidation.
- Automated compliance checklist placement near the front of the proposal.
- Requirement-to-outline mapping.
- Requirement-to-draft coverage tracking.
- Proposal red-team review.
- Attachment completeness workflow.
- Submission package preflight.
- Reusable company capability library.
- Past-performance project library.
- Partner and consortium response workflows.

### Direction 3: Make Deep Research Auditable And Enterprise-Grade

Deep research should not be a black box.

Priorities:

- Search plan editing before execution.
- Multiple search-provider comparison.
- Source vault deduplication.
- Source quality scores.
- Source quote extraction with strict copyright-safe summaries.
- Claim-to-source verification.
- Contradiction and uncertainty tracking.
- Long-report planning for 50, 100, and 200 page outputs.
- Bibliography style fidelity.
- Research audit packet export.
- Team review workflow for contentious claims.

### Direction 4: Turn Layout Into A Real Document Design System

NEditor should produce documents that business users are proud to send.

Priorities:

- More page design presets.
- Stronger cover builder.
- Multi-column layout inspector.
- Sidebar, pull quote, and margin note support.
- Better table fitting and overflow diagnostics.
- Better figure placement.
- More robust PDF/DOCX/LaTeX parity.
- Template library import for LaTeX classes.
- Visual regression baselines for exported documents.
- Native viewer signoff workflows.

### Direction 5: Make Tables, Data, And Computation Safer And More Powerful

Business documents often fail because numbers are copied manually.

Priorities:

- Richer formula language.
- More spreadsheet-like editing.
- Safer database profiles for PostgreSQL, MySQL, DuckDB, and SQLite.
- Query parameter prompts.
- Data lineage panel.
- Automatic stale-output warnings.
- Table-to-chart workflows.
- Scenario and sensitivity analysis templates.
- Reviewable calculation audit trails.
- Stronger currency, unit, and date handling.

### Direction 6: Build A Complete Template And Snippet Ecosystem

Templates should become a major product surface.

Priorities:

- Portable template pack format.
- Template validation.
- Template marketplace or curated library.
- Business development packs.
- Procurement packs.
- Education packs.
- Research packs.
- Creative writing packs.
- Industry-specific packs for consulting, climate, geospatial, legal,
  engineering, education, finance, nonprofits, and government.
- Snippet versioning and approval.
- Team-shared template repositories.

### Direction 7: Make Setup Effortless For Non-Technical Users

NEditor should be installable and understandable by business users.

Priorities:

- Polished first-run setup.
- Guided AI provider setup.
- Guided Ollama setup.
- Google login flow where possible.
- Guided transform engine installation.
- TTS setup with clear model-download consent.
- CLI deployment from menu.
- Default Markdown reader setup.
- Support bundle wizard.
- In-app diagnostics with plain-language remediation.
- Help system that explains workflows without overwhelming the user.

### Direction 8: Harden Release, Security, And Enterprise Readiness

NEditor should become deployable in serious environments.

Priorities:

- Signed and notarized macOS builds.
- Windows signing.
- Linux package verification.
- Homebrew cask publication.
- SBOM generation.
- Dependency audit.
- Security threat model updates.
- Strict CSP and provider endpoint governance.
- Secret redaction checks.
- Evidence kit validation.
- Release-device performance profiles.
- Accessibility signoff.
- Table editor manual supported-host signoff.
- Native viewer export signoff.
- Enterprise configuration policy.

### Direction 9: Support Collaboration Without Becoming Cloud-First

NEditor should support review and collaboration while preserving local-first
ownership.

Priorities:

- Better review comment lifecycle.
- Change-note and decision-log workflows.
- Git-backed document review.
- Comment export and import.
- Reviewer packets.
- Approval workflows.
- Redline-style export handoffs.
- Workspace role metadata.
- Optional integration with external review systems.

### Direction 10: Make NEditor Extensible

NEditor should allow advanced users and organizations to extend it safely.

Priorities:

- Transform handler plugin model.
- Template pack validation and installation.
- Custom export profile packs.
- Custom document wizards.
- Custom quality rules.
- Custom citation/source providers.
- Local agent integration contracts.
- Policy-controlled extension installation.

## Product Success Metrics

NEditor should be judged by business outcomes, not only feature count.

Suggested metrics:

- Time from blank page to approved outline.
- Time from RFP upload to compliance matrix.
- Percentage of mandatory RFP requirements mapped to response sections.
- Number of unsupported claims remaining before export.
- Number of unresolved placeholders before review.
- Export success rate across target formats.
- Manual formatting time avoided.
- Reused snippets and profile values per document.
- Number of source documents collected into the vault.
- Reviewer issues caught before external distribution.
- Support tickets resolved with support bundles.
- Release evidence gates accepted per release candidate.
- User confidence score after guided setup and first export.

## Risks And Product Discipline

The product should avoid several traps.

### Avoid Becoming A Generic Cloud Suite

Local-first file ownership is a core differentiator. Collaboration features
should enhance document workflows without requiring cloud lock-in.

### Avoid Treating AI Output As Automatically Finished

AI should accelerate drafting and thinking, but the product should preserve
review, evidence, provenance, and human approval.

### Avoid Hidden Complexity

Powerful features should remain discoverable through menus, buttons, command
palette, wizards, and help overlays. Advanced syntax should be optional and
explained.

### Avoid Manual-Only Release Claims

Release readiness should remain evidence-driven. If a claim requires a host,
credential, reviewer, or artifact, the system should say so clearly.

### Avoid Template Sprawl Without Quality

Templates should be curated, validated, searchable, and tied to real workflows.
Quantity matters less than usefulness, clarity, and business fit.

## The North Star

NEditor should become the place where important documents are created because
it understands the entire lifecycle:

1. Intent.
2. Sources.
3. Structure.
4. Drafting.
5. Evidence.
6. Data.
7. Review.
8. Design.
9. Export.
10. Distribution.
11. Release evidence.
12. Reuse.

The product should help a user answer these questions at any point:

- What am I trying to create?
- Who is it for?
- What evidence supports it?
- What is missing?
- What must be reviewed?
- What could disqualify or embarrass us?
- What sections remain unfinished?
- What output targets are ready?
- What has changed since the last version?
- What can be reused next time?

If NEditor keeps those questions visible and actionable, it can become a
world-class business document creation platform rather than just another editor.

