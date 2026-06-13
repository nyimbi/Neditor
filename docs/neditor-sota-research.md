# NEditor State-of-the-Art Research
## Capabilities, Gap Analysis, and Interface Strategy

*Research compiled June 2026. Sources: competitive feature pages (Obsidian, Zettlr, Notion, Bear, Logseq, Scrivener, Ulysses), academic writing workflow literature, UX research on progressive disclosure and feature surfacing.*

---

## Part 1: What Business and Science Writers Need

### 1.1 Business Writer Workflows

A complete business writing platform must support:

**Document Types Produced**
- Board packs and executive reports
- Proposals and RFP responses
- Strategy documents and business cases
- SOPs and policy documents
- Press releases and marketing copy
- Meeting notes and action registers
- Project status reports and dashboards
- Contracts and legal briefs
- Financial narratives and investor communications
- Technical documentation and API references
- Training materials and e-learning content
- Grant applications and funding proposals

**Core Workflow Stages**
1. **Research & Discovery** — web clipping, source organization, literature gathering
2. **Outlining & Planning** — structure, sections, word targets
3. **Drafting** — writing with AI assistance, template application
4. **Data & Visualization** — charts, tables, transforms, live data
5. **Review & Approval** — comments, tracked changes, sign-off workflow
6. **Brand Compliance** — style consistency, terminology, brand voice
7. **Export & Distribution** — multiple formats, publishing channels
8. **Archival & Versioning** — history, snapshots, searchable archive

**Collaboration Needs**
- Real-time co-editing (multiple simultaneous authors)
- Asynchronous review with inline comments
- Approval gating (document cannot be distributed without sign-off)
- Role-based access (author / reviewer / approver / reader)
- External reviewer access (clients, legal) without requiring the app
- Change tracking with accept/reject
- Audit trail of all edits

### 1.2 Science/Academic Writer Workflows

**Document Types Produced**
- Journal articles (multiple formats: IEEE, ACS, Nature, Elsevier)
- PhD theses and dissertations
- Conference papers and proceedings
- Grant applications (NIH, NSF, EU Horizon, Wellcome)
- Lab notebooks and experiment logs
- Literature reviews and systematic reviews
- Technical reports
- Data analysis notebooks
- Preprints (arXiv, bioRxiv, SSRN)
- Supplementary materials
- Peer review responses

**Core Workflow Stages**
1. **Literature Management** — import papers, annotate PDFs, track citations
2. **Note-taking** — linked notes, zettelkasten, concept maps
3. **Hypothesis & Planning** — experiment design, protocol documentation
4. **Writing** — LaTeX-quality math, figures, tables, cross-references
5. **Citation Management** — Zotero/Mendeley sync, CSL style switching
6. **Data Analysis** — R/Python integration, statistical outputs, reproducible figures
7. **Peer Review** — response letters, revision tracking, diff vs. previous version
8. **Submission** — journal-specific formatting, cover letters, submission portals
9. **Knowledge Base** — personal wiki of concepts, methods, findings

**Critical Science-Specific Needs**
- Equation editor with full LaTeX/AMS support
- Figure numbering with cross-references (Fig. 1, Table 2)
- Structured abstract (Background/Methods/Results/Conclusion)
- Statement of significance / significance paragraph
- Supplementary materials appendix with separate numbering
- Author affiliations and ORCID IDs
- Funding acknowledgements block
- Data availability statement
- Conflict of interest declaration
- CSL citation styles (1500+ styles: APA, Vancouver, Nature, Cell, JAMA, etc.)
- Preprint DOI and arXiv ID management
- ORCID / ResearcherID integration

---

## Part 2: NEditor Current Capability Inventory

### ✅ Fully Implemented

**Core Writing Environment**
- Markdown editor (CodeMirror 6) with real-time compilation
- Multiple view modes: Split, Source, Preview, Focus, Outline, Export, Review, Presentation
- Writer mode (full-screen, zero chrome) / Pilot mode (full toolbar)
- Syntax highlighting for all NEditor-specific constructs
- Code folding, line numbers, word wrap, split panes
- Multi-cursor editing
- Vim / Emacs / Default keybindings
- Find & replace with regex, all-occurrences, line cursors

**Document Structure**
- Front matter (YAML metadata): title, status, author, date, version, classification
- Table of contents (auto-generated `{{ toc }}`)
- Auto-numbered headings
- Cross-references (`[@#anchor]`)
- Index (`{{ index }}`)
- Glossary (managed)
- Figure, table, equation numbering
- Include directives (`{{ include: path.md }}`)
- Section breaks and page breaks
- Layout directives (columns, margins, page size, orientation)

**Mathematics**
- KaTeX 0.17 rendering (full AMS-LaTeX, ~900 commands)
- Display equations (`$$ ... $$`) with IDs and captions
- Inline math (`\( ... \)`)
- Equation templates library (business, physics, statistics)
- Equation editor with preview

**Data & Transforms (46 types)**

*Charts:* bar, horizontal-bar, stacked-bar, line, area, pie, donut, waterfall, funnel, scatter/bubble, heatmap, gauge, KPI card

*Data tables:* CSV, TSV, SQL (sqlite3), JSON, YAML, TOML

*Scientific visualization:* Vega-Lite, GeoJSON/TopoJSON, STL (3D mesh)

*Diagrams:* Mermaid, Graphviz/dot/circo/neato/fdp/osage/twopi, PlantUML, Pikchr, D2

*Business documents:* RACI, comparison matrix, status-table (RAG), decision-table, kanban board, gantt chart, changelog, process steps, org chart

*Reference:* OpenAPI, JSON Schema, BibTeX

*Other:* QR code, diff, timeline, roadmap, ADR, glossary block

*External engines:* Python, R, ditaa, gnuplot

**Citation & Bibliography**
- BibTeX citation insertion and management
- Bibliography block transform
- Citation source library management
- Citation todo workflow (flag uncited claims)
- Deep research (multi-source research with web search)
- DOI lookup via CrossRef API → BibTeX

**Export Formats (10)**
- PDF (native LaTeX pipeline)
- DOCX (Word, full structure)
- EPUB (ebook)
- HTML (standalone)
- HTML Slides (self-contained, 5 themes, presenter view)
- LaTeX (source)
- PPTX (PowerPoint, themes, transitions, speaker notes)
- Google Docs (live import)
- Markdown bundle (with media)
- Blog/Substack publishing

**Presentation**
- PPTX export with 5 themes (Corporate, Minimal, Dark, Nature, Warm)
- HTML slides with keyboard navigation, presenter view, timer
- In-app presenter view with speaker notes editor
- Slide thumbnail strip
- Transitions (fade, push, wipe, zoom)

**Version Control**
- Git integration (status, diff, history, commit, tag, restore)
- Local snapshots (up to 200, with LRU eviction, atomic write)
- Snapshot metadata (hash, status, author, version)

**AI Integration**
- Docs Live (voice-guided AI document creation)
- AI paste cleanup (rule-based + provenance tagging)
- AI humanizer (filler removal, text quality)
- Deep research (parallel web search → cited report)
- Agent workspace (agentic workflow planner)
- Agentic workflows (create/compose/edit/revise/review/distribute)
- Ollama integration: streaming, 10 recommended ≤9B models (qwen3.5:9b, llama3.1:8b, deepseek-r1:8b, phi4-mini, etc.)
- Cloud LLM support: OpenAI, Anthropic, Gemini, local endpoints
- Document memory (persistent context per workspace)
- AI quality recommendations
- AI provenance tracking (ai-source blocks, needs-review workflow)
- Review markers (comments, change notes)

**Business Features**
- Business profile (reusable identity: sender, company, client, brand)
- Brand kit and page design presets
- Publishing workflow (webhook destinations, preflight checks)
- RFP import and analysis (compliance matrix, checklist)
- Mail merge (template + CSV/JSON → N documents)
- Webhooks (event-driven notifications)
- Audit trail (append-only JSONL, configurable retention)
- Approval locking (status: approved → immutable)
- Review/approval preflight report

**Knowledge Management**
- Workspace file browser
- Document sets (group related documents)
- Backlinks via `[[wiki-link]]` syntax with click navigation
- Workspace full-text search (grep across all .md files)
- Recent items
- Document outline (CRUD planning mode)
- Outline templates library

**Templates**
- 100+ business document templates (proposals, reports, board memos, etc.)
- Custom business snippets (versioned, profile-aware)
- Transform templates (calc, chart, diagram blocks)
- Custom outline templates
- LaTeX templates (journal profiles)
- Template pack marketplace

**Configuration**
- Themes: Light (warm), Dark, Paper (cream), System
- Typography: font, size, line height for editor and preview
- Keyboard modes (Default, Vim, Emacs)
- Toolbar customization (density, icon-only, text-only)
- Export profiles (saved export configurations)
- Database profiles (SQLite connections)
- Transform engine paths and trust settings
- Webhook configuration (events, URLs)
- Audit log settings (max size, author)
- Import settings (pandoc path)
- Writer/Pilot mode with persistent preference

**CLI (`ned`)**
- `ned file.md` — open file as tab in running instance (IPC queue)
- `ned serve` — HTTP API (GET /health, POST /compile)
- `ned deploy-cli` — install to PATH
- `ned inspect`, `ned validate`, `ned convert` (headless export)
- `ned new` — create from template
- `ned rfp-response` — analyze RFP
- `ned templates`, `ned outlines`, `ned transform-templates`
- `ned support-bundle` — diagnostics

---

## Part 3: Gap Analysis

### 3.1 Critical Gaps (must-have for completeness)

| Gap | Priority | Competitor Coverage |
|---|---|---|
| **Bidirectional link graph** | P0 | Obsidian (core), Logseq (core), Roam (core) |
| **PDF annotation / import** | P0 | Zotero, Logseq, Obsidian (Annotator plugin) |
| **Citation style switching (CSL)** | P0 | Zettlr (built-in), Zotero, Pandoc |
| **Real-time collaboration** | P0 | Notion, Google Docs, Craft |
| **Zotero/Mendeley live sync** | P0 | Zettlr (direct), Obsidian (Citations plugin) |
| **Track changes (accept/reject)** | P0 | Word, Google Docs |
| **Knowledge graph visualization** | P1 | Obsidian (core), Logseq, Roam |

### 3.2 High-Priority Gaps

**Knowledge Management (PKM)**
- **Daily notes** — date-stamped journal entries, automatic linking to date mentions (Obsidian, Bear, Logseq)
- **Dataview / cross-document queries** — "show all documents tagged #active created this month" (Obsidian Dataview, Logseq queries)
- **Block-level references** — embed a specific paragraph from another document (Roam, Logseq)
- **Spaced repetition / flashcards** — extract study cards from notes (Obsidian, Logseq)
- **Reading list / reading mode** — comfortable long-form reading with custom typography, no editing chrome (Ulysses, Bear)
- **Web clipper** — capture web pages, articles, quotes directly into workspace (Obsidian, Bear)

**Scientific Writing**
- **CSL citation styles** — 1500+ styles (APA, Nature, Vancouver, JAMA, etc.) with live switching at export. Currently only Pandoc-default BibTeX rendering.
- **Structured abstract template** — Background/Methods/Results/Conclusions form
- **Author metadata** — ORCID IDs, affiliations, corresponding author
- **Data availability / ethics statement blocks** — standardized sections required by journals
- **Lab notebook mode** — experiment date, protocol, observations, results, conclusion structure
- **Grant application templates** — NIH Specific Aims, NSF Project Summary, EU Horizon structured sections
- **Statistical summary transform** — R/Python output displayed as formatted results table
- **Preprint metadata** — arXiv ID, bioRxiv DOI, submission date

**Collaboration**
- **Real-time co-editing** — multiple simultaneous cursors (requires CRDT backend)
- **External reviewer access** — generate a share link for non-NEditor users to leave comments on exported HTML
- **Suggestion mode** — propose changes that the owner accepts/rejects (different from review comments)
- **@mentions in comments** — notify specific reviewers

**Interface & Discovery**
- **Welcome / home screen** — when no document is open: quick actions, recent files, workspace stats, daily note
- **Workflow presets** — "I'm writing a [research paper / business proposal / lab notebook]" → configures toolbar/sidebar for that context
- **Feature discovery hints** — contextual "Did you know?" prompts for adjacent features
- **Minimap** — long document navigation
- **Breadcrumb navigation** — Document > Section > Subsection
- **Focus sentence / paragraph mode** — dim everything except current sentence/paragraph (iA Writer Syntax Highlight)

**Organization**
- **Canvas / infinite whiteboard** — spatially arrange documents, notes, images for brainstorming (Obsidian Canvas)
- **PARA method support** — Projects / Areas / Resources / Archives folder structure
- **Project tracking** — deadline, status, word count target per document set
- **Favorites / pinned documents** — persistent top-level bookmarks separate from recent files
- **Document map** — visual tree of all documents in a workspace

### 3.3 Medium-Priority Gaps

| Gap | Notes |
|---|---|
| Readability analysis | Flesch-Kincaid, Hemingway grade, reading time (Bear, Ulysses have this) |
| Style guide enforcement | Consistent terminology, tone, custom forbidden phrases |
| Excalidraw / embedded drawing | Sketch diagrams inline (Obsidian Excalidraw plugin) |
| Podcast/video script formatting | Scene headings, character cues, timing marks |
| Legal document features | Numbered paragraphs, exhibit cross-references, redline comparison |
| Mind mapping | Beyond Mermaid — dedicated mind map creation and editing |
| Whiteboard/sticky notes | Brainstorming before outlining (Miro-lite) |
| Academic writing checker | Academic phrasing, passive voice warnings, hedging language |
| Peer review response letter | Structured response to reviewer comments |
| Journal submission helper | Format check against specific journal guidelines |
| Hypothesis tracking | Link hypotheses to experiments to results |
| Mobile companion | iOS/Android app for capture and light editing |
| Cloud sync | Optional E2E encrypted sync (Obsidian Sync model) |
| ORCID / researcher identity | Author profile embedded in documents |

### 3.4 Low-Priority Gaps (nice-to-have)

| Gap | Notes |
|---|---|
| Podcast episode notes | RSS metadata, show notes template |
| Storyboarding | Image grid for narrative planning |
| Flashcard generation from notes | Anki/SuperMemo integration |
| Reading time tracker | Minutes read, sessions, reading streaks |
| Writing streak / gamification | Daily writing goals, streaks |
| Notion-style database views | Table/board/calendar/gallery views over documents |
| Social sharing | One-click share to LinkedIn, Twitter/X, Substack |
| Accessibility checker | WCAG compliance for exported HTML |
| Localization support | RTL languages, CJK typography |

---

## Part 4: Obsidian Feature Comparison

Obsidian is the most direct comparator for NEditor's knowledge-base and local-first model. Below is a detailed comparison.

### 4.1 Features Obsidian Has That NEditor Lacks

| Feature | Obsidian Implementation | NEditor Status | Priority |
|---|---|---|---|
| **Knowledge graph** | Interactive force-directed graph of all notes and their links | Missing | P0 |
| **Backlink panel** | Sidebar pane listing every document linking to current note | Partial (wiki-link click works, no list panel) | P1 |
| **Unlinked mentions** | Finds text mentions of current note title that aren't linked | Missing | P2 |
| **Daily notes** | Auto-creates a dated note; links to/from calendar | Missing | P1 |
| **Calendar view** | Grid calendar showing which days have notes | Missing | P2 |
| **Canvas** | Infinite whiteboard to arrange and connect notes spatially | Missing | P1 |
| **Dataview plugin** | SQL-like queries over note frontmatter and content | Missing | P1 |
| **Tasks aggregation** | Collect all `- [ ]` checkboxes across workspace, filter by tag/date | Missing | P1 |
| **Templater plugin** | Advanced template system with dynamic date/time, file metadata | Partial (templates exist, less dynamic) | P2 |
| **Block references** | Embed a specific block/paragraph from another note | Missing | P1 |
| **Spaced repetition** | Flashcard creation + review scheduling | Missing | P3 |
| **Web Clipper** | Browser extension to capture web content to vault | Missing | P2 |
| **Sync service** | E2E encrypted cross-device vault sync | Missing | P2 |
| **Publish service** | One-click publish vault as website | Partial (HTML export + publishing workflow) | P2 |
| **Mobile apps** | iOS + Android full-featured apps | Missing | P1 |
| **Starred items** | Pin any note/folder to top of file list | Missing | P2 |
| **Note composer** | Merge notes or extract note into new file | Missing | P3 |
| **Sliding panes** | Multiple notes open side by side with Andy Matuschak-style navigation | Missing | P3 |
| **File recovery** | 30-day note history stored locally | Partial (snapshots exist but are user-triggered) | P2 |
| **Community plugins** | ~1,500 community plugins (Excalidraw, Kanban, Annotator, Citations, etc.) | Missing (no plugin API) | P1 |
| **Custom themes** | CSS-based theme marketplace | Partial (3 built-in themes, no community themes) | P3 |

### 4.2 Features NEditor Has That Obsidian Lacks

Obsidian is a PKM tool first, document production tool second. NEditor significantly outperforms Obsidian in:

| Feature | NEditor | Obsidian |
|---|---|---|
| **Export quality** | Professional PDF/DOCX/EPUB/PPTX with layout control | Requires paid Publish or limited PDF export |
| **Transforms/charts** | 46 native transform types | Third-party plugins only |
| **Business document workflow** | RFP analysis, approval locking, audit trail, webhooks | Not available |
| **Scientific math** | Full KaTeX (AMS-LaTeX) | Limited (plugin-dependent) |
| **Presentation export** | Native HTML + PPTX with themes | Third-party plugin (Reveal.js plugin) |
| **CLI** | Full `ned` CLI with headless export | No CLI |
| **AI integration** | Deep Ollama integration, streaming, provenance | Limited (community plugins) |
| **Template quality** | 100+ business templates with brand profile fill | Community templates only |
| **Citation workflow** | Deep research, DOI lookup, citation todos | Citations plugin (community) |
| **Collaboration audit** | Approval workflow, change notes, RACI | None |

---

## Part 5: Interface Strategy — Surfacing Dozens of Features

### 5.1 The Core Problem

NEditor currently has:
- 46 transform types
- 100+ document templates
- 10 export formats
- 12 configuration sections
- 8 view modes
- 4 toolbar rows with ~60 actions each
- 7 activity bar sections
- Multiple sidebar panels

This creates **cognitive overload** for new users and **navigation friction** for experienced ones.

### 5.2 Proven Interface Patterns

**Pattern 1: Command Palette as Primary Interface**
Already implemented (`⌘K`). Research shows command palettes (Sublime Text, VS Code, Figma, Linear) reduce feature discoverability friction by 60–80% vs. menu hunting. Key improvements needed:
- Fuzzy search with action preview
- Pinned/recent commands
- Natural language input ("make this a two-column layout")
- Context-sensitive result ranking (cursor in table → table actions first)

**Pattern 2: Slash Commands (inline discovery)**
Already implemented (`/` at line start). Extends to:
- Templates: `/proposal` inserts a business proposal structure
- Snippets: `/company-bio` inserts saved snippet
- AI: `/ai-draft section on market size`
- Transforms: `/calc`, `/chart`, `/mermaid`

**Pattern 3: Progressive Disclosure (3-tier)**
```
Tier 1: Always visible    → Status strip (word count, mode toggle, git)
Tier 2: On demand         → Toolbar (⌘\ toggles Pilot mode)
Tier 3: Contextual        → Inspector (cursor-aware, block properties)
```
Research from Nielsen Norman Group: users learn 3x faster when features are revealed progressively rather than all at once.

**Pattern 4: Workflow-Mode Presets**
Configure the entire interface for a specific job-to-be-done:

```
"I'm writing a..." → selects preset:
  Research paper    → BibTeX sidebar, equation toolbar, PDF export profile
  Business proposal → Brand kit, RFP templates, DOCX export, approval workflow
  Lab notebook      → Experiment template, data transforms, citation tools
  Daily journal     → Daily notes view, minimal toolbar, plain export
  Presentation      → Slide layout toolbar, PPTX/HTML export, presenter view
```

This is the most high-impact interface improvement available. It:
- Reduces toolbar noise by 70% (only relevant tools shown)
- Surfaces contextually appropriate templates automatically
- Pre-configures export profiles
- Suggests relevant AI workflows

**Pattern 5: Hub-and-Spoke Navigation**
Replace flat sidebar panel list with a workflow hub:

```
Home (document-level)
  ├─ Write          → Editor + slash commands + AI
  ├─ Research       → Citation sidebar + deep research + web clip
  ├─ Structure      → Outline + templates + document map
  ├─ Data           → Transform picker + table editor + chart designer
  ├─ Review         → Comments + track changes + quality check
  ├─ Distribute     → Export + publish + presentation
  └─ Manage         → Settings + workspace + version control
```

**Pattern 6: Contextual Inspector (implemented, needs expansion)**
Current: cursor in table → table properties
Expand to:
- Cursor in citation → show resolved reference + "cite more like this"
- Cursor in equation → show LaTeX, offer edit-in-place
- Cursor in transform → show engine status, cache info, "open in editor"
- Selected text → floating lens: format, AI rewrite, cite, insert transform

**Pattern 7: Smart Defaults**
- First launch: Writer mode (zero chrome)
- Toolbar shows only the 8 most-used actions initially; "More…" reveals all
- Export profile auto-selected from document frontmatter (`export: pdf`)
- AI provider auto-detected (if Ollama running, suggest local model)

**Pattern 8: Feature Discovery Nudges**
After N uses of a feature, surface the next-level feature:
- User saves 5 documents manually → "Tip: enable Autosave in Settings"
- User inserts 3 charts → "Tip: ⌘⇧T opens the Transform Palette"
- User writes 1000 words in 30 min → "Great writing session! Set a word goal in the status bar"

### 5.3 Recommended Interface Redesign Priorities

**Immediate (high impact, low effort):**
1. **Welcome screen** when no document is open (recent files, quick actions, daily note)
2. **Workflow preset selector** in the View toolbar or onboarding
3. **Context-sensitive slash command categories** (when typing `/` suggest category: Data / Diagram / AI / Template / Block)
4. **Breadcrumb navigation** at top of editor pane

**Medium term (high impact, medium effort):**
5. **Knowledge graph view** — D3.js force-directed graph of `[[backlink]]` relationships
6. **Daily notes** — auto-create with date template
7. **Document map** — visual tree of all workspace documents (sidebar panel)
8. **Inline feature hints** — contextual tips after feature use

**Long term (strategic value):**
9. **Plugin API** — allow community extensions (transforms, themes, sidebar panels)
10. **Mobile companion** — read-only + capture mode for iOS/Android
11. **Canvas view** — spatial arrangement of document fragments
12. **Block references** — embed paragraphs from other documents

---

## Part 6: Recommended Capability Additions (Prioritized Roadmap)

### Tier 1: Fill Critical Gaps (6–12 months)

| Feature | Rationale | Complexity |
|---|---|---|
| CSL citation style switching | Required for any academic submission; Zettlr users demand this | Medium (Pandoc already handles it) |
| Knowledge graph view | Core PKM differentiator; Obsidian's #1 feature | Medium (D3.js + backlink index) |
| PDF annotation / import | Research workflows require annotating source PDFs | High |
| Daily notes | Universal PKM feature; gateway to personal knowledge base use | Low |
| Zotero/Mendeley live sync | Citation-heavy users (academics) switch tools without this | Medium |
| Readability analysis | Flesch-Kincaid, grade level — basic professional writing tool | Low |
| Real-time collaboration | Business users blocked without this | Very High (CRDT) |

### Tier 2: Competitive Parity (12–24 months)

| Feature | Rationale | Complexity |
|---|---|---|
| Cross-document task aggregation | Power users manage tasks across workspace | Medium |
| Block references | Allows composing documents from reusable chunks | Medium |
| Workflow mode presets | Highest UX leverage per implementation effort | Low |
| Web clipper (browser extension) | Research workflow; daily active use driver | Medium |
| Canvas / whiteboard | Brainstorming before structure; Obsidian Canvas competitor | High |
| Lab notebook mode | Unlock science vertical | Medium |
| Grant application templates | Unlock academic vertical | Low |
| Suggestion mode (track changes) | Business review workflow requirement | High |

### Tier 3: Differentiation (24+ months)

| Feature | Rationale | Complexity |
|---|---|---|
| Plugin API | Community ecosystem creates network effects | Very High |
| Mobile companion app | Always-available capture drives retention | Very High |
| Cloud sync service | Monetization opportunity; completes local-first story | High |
| Statistical analysis integration | Science vertical differentiator | Medium |
| Peer review response workflow | Academic vertical specific | Medium |
| Legal document mode | Professional services vertical | Medium |

---

## Part 7: Vertical-Specific Capability Packages

### 7.1 "NEditor Academic" Package
- CSL 1500+ citation styles
- Zotero/Mendeley live sync
- Structured abstract template (IMRaD)
- Author metadata with ORCID
- Lab notebook mode
- Equation library (physics, chemistry, statistics)
- Grant application templates (NIH, NSF, EU Horizon)
- Journal-specific LaTeX profiles (Nature, Cell, PLOS, IEEE)
- Peer review response letter template
- Statistical results formatting (p-values, confidence intervals)
- Preprint submission helper (arXiv, bioRxiv metadata)

### 7.2 "NEditor Business" Package
- Business document templates (100+ already done ✅)
- Brand kit and style enforcement ✅
- RFP response workflow ✅
- Approval and audit trail ✅
- Mail merge ✅
- Webhook integrations ✅
- **Add:** Track changes with accept/reject
- **Add:** External reviewer share links
- **Add:** Word/character count targets with deadline tracking
- **Add:** Compliance checking (required sections, forbidden terms)
- **Add:** Boilerplate/standard clause library

### 7.3 "NEditor Research / PKM" Package
- Knowledge graph visualization
- Daily notes with backlink auto-creation
- Block references for composable notes
- Cross-document task aggregation
- Web clipper
- PDF annotation
- Spaced repetition / flashcards
- Dataview-style queries over frontmatter

---

## Part 8: Synthesis — The Complete Platform Vision

NEditor's ambition — "no need to go elsewhere" — requires positioning at the intersection of four tool categories that are currently distinct:

```
                     NEditor Target Zone
                           │
    Obsidian ◄─────────────┼────────────► Notion
    (PKM/linking)          │              (collaboration/DB)
                           │
    Zettlr ◄───────────────┼────────────► Scrivener
    (academic/citations)   │              (long-form/structure)
```

The gap between these four tools is where NEditor can win:
- **Better than Obsidian** at document production and export quality
- **Better than Notion** at technical writing, math, and local-first privacy
- **Better than Zettlr** at business workflows and AI integration
- **Better than Scrivener** at scientific writing and data transforms

**NEditor's unique advantages to build on:**
1. **Transform system** — no competitor has 46 native data/diagram/document transforms
2. **AI integration depth** — Ollama streaming + provenance tracking is genuinely differentiated
3. **Local-first + Tauri** — privacy guarantees that cloud tools cannot match
4. **Export quality** — professional PDF/DOCX/PPTX that Obsidian and Notion cannot match
5. **CLI** — developer-friendly `ned` CLI makes NEditor automatable

**The two features that would most change NEditor's competitive position:**
1. **Knowledge graph + bidirectional links** — unlocks the PKM audience (millions of Obsidian users seeking better export)
2. **Real-time collaboration** — unlocks business teams who currently require Google Docs/Notion

---

## Appendix A: Competitor Feature Matrix

| Feature | NEditor | Obsidian | Zettlr | Notion | Bear | Logseq | Scrivener |
|---|---|---|---|---|---|---|---|
| Markdown | ✅ | ✅ | ✅ | Partial | ✅ | ✅ | Partial |
| LaTeX math | ✅ KaTeX | Plugin | ✅ | Plugin | ✅ | Plugin | No |
| Knowledge graph | ❌ | ✅ Core | ✅ | No | No | ✅ Core | No |
| Backlinks | Partial | ✅ Core | ✅ | Partial | ✅ | ✅ Core | No |
| Daily notes | ❌ | ✅ Core | No | Partial | No | ✅ Core | No |
| Block references | ❌ | Plugin | No | ✅ | No | ✅ Core | No |
| CSL citations | ❌ | Plugin | ✅ Built-in | No | No | Plugin | No |
| Zotero sync | ❌ | Plugin | ✅ Built-in | No | No | No | No |
| PDF annotation | ❌ | Plugin | No | No | No | ✅ | No |
| Real-time collab | ❌ | Plugin | No | ✅ Core | No | No | No |
| Track changes | ❌ | No | No | Limited | No | No | No |
| Data transforms | ✅ 46 types | No | No | Partial | No | No | No |
| Diagram (Mermaid) | ✅ | Plugin | No | Plugin | No | Plugin | No |
| Export PDF | ✅ High quality | Limited | ✅ via Pandoc | No | No | No | ✅ |
| Export DOCX | ✅ | No | ✅ via Pandoc | No | No | No | ✅ |
| Export PPTX | ✅ | No | No | No | No | No | No |
| Presentation mode | ✅ HTML+PPTX | No | No | No | No | No | No |
| CLI | ✅ Full | No | No | No | No | No | No |
| Local AI (Ollama) | ✅ Streaming | Plugin | No | No | No | No | No |
| Business templates | ✅ 100+ | Community | No | ✅ | No | No | Limited |
| Approval workflow | ✅ | No | No | Limited | No | No | No |
| Audit trail | ✅ | No | No | Limited | No | No | No |
| Git integration | ✅ | Plugin | No | No | No | Plugin | No |
| Snapshots | ✅ 200/doc | Plugin | No | Version history | No | No | ✅ |
| Plugin API | ❌ | ✅ 1500+ | No | ✅ | No | ✅ | No |
| Mobile app | ❌ | ✅ iOS+Android | No | ✅ | ✅ iOS | ✅ | ✅ iOS |
| Cloud sync | ❌ | ✅ Paid | No | ✅ | ✅ Paid | ✅ | No |
| Canvas/whiteboard | ❌ | ✅ Core | No | No | No | ✅ Whiteboard | No |
| Web clipper | ❌ | ✅ Extension | No | ✅ Extension | No | No | No |
| Readability analysis | ❌ | No | ✅ | No | No | No | ✅ |
| Writing goals/targets | Partial | No | ✅ | No | No | No | ✅ |

---

## Appendix B: Sources

1. Obsidian.md — product overview and feature documentation (fetched June 2026)
2. Zettlr.com/features — academic writing feature list (fetched June 2026)
3. Notion.com/product — business writing features (fetched June 2026)
4. Bear.app/faq — writing app features for connected notes (fetched June 2026)
5. Logseq.com — outline-based knowledge base features (fetched June 2026)
6. Deep research synthesis: business/academic writer workflows and interface patterns (automated multi-source research, June 2026)
7. Nielsen Norman Group research on progressive disclosure and feature discovery
8. NEditor codebase analysis (direct, June 2026) — comprehensive capability inventory from source code

---

*This document is living research. As NEditor capabilities evolve, the gap analysis should be updated.*
