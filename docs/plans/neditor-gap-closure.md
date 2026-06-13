# NEditor: Full Gap Closure Implementation Plan

## Context

`docs/neditor-sota-research.md` documents a comprehensive gap analysis comparing NEditor against Obsidian, Zettlr, Notion, Bear, Logseq, and Scrivener. The research identified ~40 missing features across knowledge management, scientific writing, interface design, and AI workflows. This plan closes every achievable gap — deferring only features requiring architectural overhauls (CRDT real-time collaboration, browser extension web clipper, mobile app, cloud sync, plugin API).

**First task of implementation:** Copy the final plan into `docs/plans/neditor-gap-closure.md`.

---

## Current State Summary (from exploration)

- **Frontend:** Single `src/App.vue` (37,562 lines), Vue 3 + Pinia, CodeMirror 6, no component files
- **Backend:** Tauri 2 + Rust, ~103 modules, rich transform/export/citation infrastructure
- **Existing infrastructure to reuse:**
  - `backlinks.rs` — `find_backlinks(target, workspace_root)` exists (full directory scan)
  - `bibliography.rs` — 11 built-in citation styles, CSL-JSON parser, `cslStyle` front-matter alias already wired
  - `qualityRecommendations.ts` — readability stub (long-paragraph heuristic only)
  - `cli_ipc.rs` — IPC queue for single-instance routing
  - `snapshot.rs` — atomic write, 200-snapshot LRU
- **Dependencies installed:** CodeMirror 6, KaTeX 0.17, Pinia 3, Vue 3. No d3, no citeproc-js, no readability library.

---

## Sprint 1 — High Value, No New Dependencies

### 1.1 Readability Analysis (Flesch-Kincaid)

**New file:** `src-tauri/src/readability.rs`

Implement pure-Rust Flesch-Kincaid scoring:
```
FK Reading Ease = 206.835 - 1.015*(words/sentences) - 84.6*(syllables/words)
FK Grade Level  = 0.39*(words/sentences) + 11.8*(syllables/words) - 15.59
```

Syllable counting algorithm: count vowel groups (aeiou) in each word, handle silent-e, -le endings, minimum 1 per word.

**Struct to return:**
```rust
pub struct ReadabilityStats {
    pub word_count: usize,
    pub sentence_count: usize,
    pub paragraph_count: usize,
    pub syllable_count: usize,
    pub avg_words_per_sentence: f64,
    pub flesch_reading_ease: f64,   // 0–100 (higher = easier)
    pub flesch_kincaid_grade: f64,  // US school grade level
    pub gunning_fog: f64,           // optional
    pub reading_time_minutes: f64,  // word_count / 250
    pub long_sentence_count: usize, // sentences > 30 words
    pub complex_word_count: usize,  // words > 3 syllables
}
```

**Register command in `lib.rs`:** `analyze_readability(text: String) -> ReadabilityStats`

**Frontend changes (App.vue):**
- Status bar: show FK grade level and reading time alongside word count
- Inspector panel: when no specific block is at cursor, show full readability card
- Update `src/lib/qualityRecommendations.ts` `buildQualityRecommendations()` to call `analyze_readability` and use real FK score (warn when FK grade > 12 or reading ease < 50)
- Add `analyzeReadability` action to command palette

---

### 1.2 Daily Notes

**New Rust command in `lib.rs`:** `open_daily_note(workspace_root: String, date: String) -> FileResponse`

Implementation in new `src-tauri/src/daily_notes.rs`:
- Creates `<workspace_root>/daily-notes/<YYYY-MM-DD>.md` if not exists
- Default template front-matter: `date`, `type: daily-note`, `tags: [journal]`
- Body: `# Journal — YYYY-MM-DD\n\n## Today\n\n\n## Notes\n\n\n## Tasks\n- [ ] \n`
- Links to previous/next day in front-matter as `previous:` / `next:`
- Returns the file path so frontend can open it as a tab

**Frontend changes (App.vue):**
- Toolbar button (calendar icon, ⌘⇧D): calls `open_daily_note` for today, opens result as tab
- Add `'daily-notes'` panel to sidebar (new `v-else-if="store.sidebar === 'daily-notes'"` block)
  - Mini calendar grid (pure CSS, no library): show month view, days with notes highlighted
  - Click day → opens that day's note
  - Shows streak counter (consecutive days with notes)
- Activity bar: add to "document" group alongside outline/diagnostics
- Slash command: `/journal` or `/daily-note` → opens today's note
- Command palette: "Open today's note", "Open daily note for date…"

**Store additions (`documents.ts`):**
- `dailyNotesEnabled: boolean` (default true)
- `dailyNotesFolder: string` (default `"daily-notes"`)

---

### 1.3 Backlink Panel

Rust `find_backlinks(target_path, workspace_root)` already exists in `backlinks.rs`. Just need the frontend panel.

**Frontend changes (App.vue):**
- New sidebar panel `'backlinks'` (`v-else-if="store.sidebar === 'backlinks'"`)
  - Header: "Links to this document (N)"
  - Calls `invoke('find_backlinks', { targetPath: activeDoc.path, workspaceRoot: store.workspaceRoot })` on `watch(activeId, ...)`
  - Renders grouped list: each item shows filename, line number, excerpt
  - Click → opens file at that line
  - "Unlinked mentions" sub-section: call new `find_unlinked_mentions` command (see §1.4)
- Activity bar: add `'backlinks'` to `ACTIVITY_GROUPS` "document" group

**New Rust command:** `find_unlinked_mentions(title: String, workspace_root: String) -> Vec<BacklinkResult>` — same scan as `find_backlinks` but matches the document title string (not `[[...]]` syntax) in plain text that is NOT already wrapped in `[[]]`.

---

### 1.4 Cross-Document Task Aggregation

**New Rust command:** `collect_workspace_tasks(workspace_root: String) -> Vec<WorkspaceTask>` in new `src-tauri/src/task_aggregator.rs`

```rust
pub struct WorkspaceTask {
    pub file_path: String,
    pub line: usize,
    pub text: String,
    pub done: bool,
    pub tags: Vec<String>,    // #tag patterns in the line
    pub due_date: Option<String>,   // @YYYY-MM-DD or due:YYYY-MM-DD
    pub heading_context: String,    // nearest ## heading above
}
```

Scans all `.md` files for `- [ ]` and `- [x]` lines; extracts tags and dates from the text.

**Frontend changes (App.vue):**
- New sidebar panel `'tasks'`
- Filter bar: All / Todo / Done; filter by #tag; filter by file; sort by due date
- Group by: File, Tag, Due Date
- Click → navigate to file+line
- "Mark done" toggle button per item
- Activity bar: add to "document" group

---

### 1.5 Workflow Preset Selector

**New file:** `src/lib/workflowPresets.ts`

```typescript
export interface WorkflowPreset {
  id: string
  label: string
  description: string
  icon: string
  applyFn: (store: DocumentsStore) => void
}

export const WORKFLOW_PRESETS: WorkflowPreset[] = [
  {
    id: 'academic',
    label: 'Academic / Research',
    description: 'Journal articles, theses, lab notebooks',
    icon: '⚗️',
    applyFn: (store) => {
      store.sidebar = 'references'
      store.exportTarget = 'pdf'
      store.bibliographyDefaults.citationStyle = 'apa'
      store.toolbarCollapsedRows = ['row-business', 'row-presentation']
    }
  },
  {
    id: 'business',
    label: 'Business Writing',
    description: 'Proposals, reports, board packs',
    applyFn: (store) => {
      store.sidebar = 'templates'
      store.exportTarget = 'docx'
      store.toolbarCollapsedRows = ['row-equations', 'row-citations']
    }
  },
  {
    id: 'lab-notebook',
    label: 'Lab Notebook',
    description: 'Experiments, protocols, observations',
    applyFn: ...
  },
  {
    id: 'presentation',
    label: 'Presentation',
    description: 'Slides, decks, speaker notes',
    applyFn: (store) => { store.mode = 'presentation'; store.exportTarget = 'pptx' }
  },
  {
    id: 'daily-journal',
    label: 'Daily Journal',
    description: 'Notes, diary, daily review',
    applyFn: (store) => { store.uiMode = 'writer'; store.sidebar = 'daily-notes' }
  },
  {
    id: 'technical',
    label: 'Technical Documentation',
    description: 'API docs, READMEs, specs',
    applyFn: (store) => { store.exportTarget = 'html'; store.sidebar = 'outline' }
  },
]
```

**Frontend changes (App.vue):**
- "I'm writing a…" button in toolbar (Pilot mode) → opens preset picker modal
- Welcome screen shows preset picker as primary CTA (see §1.6)
- Command palette: "Switch workflow preset…" → shows picker
- Store: `activeWorkflowPreset: string | null` in `documents.ts`
- Slash command: `/preset` → opens picker

---

### 1.6 Welcome / Home Screen

**Frontend changes (App.vue):**
- Conditional render in main content area: `v-if="store.documents.length === 0"`
- Full-screen welcome panel replaces the empty editor
- Contents:
  - NEditor wordmark + version
  - "I'm writing a…" workflow preset grid (6 preset cards)  
  - Quick actions: New Document, Open File, Open Folder, Import Document
  - Recent files grid (last 8, from `store.recentFiles`)
  - Today's note shortcut (shows today's date, opens daily note)
  - "Continue where you left off" (restore last session)
- Also show when entering Writer mode with no document (not just on launch)

---

### 1.7 Breadcrumb Navigation

**Frontend changes (App.vue):**
- New computed `documentBreadcrumbs`: parses editor content, finds all headings, determines which heading precedes the cursor
- Returns array: `[{level, text, offset}, ...]` for current heading hierarchy
- Renders as narrow bar above editor (between toolbar and CodeMirror): `Document > Section > Subsection`
- Click on any crumb scrolls editor to that heading
- Only shown in Pilot mode; hidden in Writer mode

---

### 1.8 Focus Sentence / Paragraph Mode (iA Writer Style)

**Frontend changes (App.vue):**
- New store field: `focusMode: 'off' | 'paragraph' | 'sentence'`
- CodeMirror ViewPlugin: observes cursor position, computes which paragraph/sentence is "active"
- Applies `.cm-dim` class to all other paragraphs/sentences (CSS: `opacity: 0.25; transition: opacity 0.15s`)
- Toggle: ⌘⌥F cycles off → paragraph → sentence → off
- Button in View toolbar (Pilot mode)
- Command palette: "Focus mode: sentence", "Focus mode: paragraph", "Focus mode: off"

---

### 1.9 Context-Sensitive Slash Command Categories

**Frontend changes (App.vue) — modify `SLASH_CMDS` array and slash picker UI:**
- Add `category` field to each slash command: `'insert' | 'ai' | 'data' | 'template' | 'block'`
- Group the picker display by category with headers
- Context awareness: detect if cursor is in a table → show "Data" category first; in a code block → show "Code" first
- Keyboard: typing after `/` filters across all categories

---

### 1.10 Feature Discovery Nudges

**Store additions (`documents.ts`):**
- `featureUsageCounts: Record<string, number>` — persisted, incremented per feature use
- `dismissedNudges: string[]` — persisted

**Frontend changes (App.vue):**
- `checkNudge(feature: string)` — after incrementing count, check if a tip should surface
- Tip definitions:
  - After 3rd Transform Palette click: "Tip: ⌘⇧T opens the Transform Palette"
  - After 5th manual save: "Tip: Enable Autosave in Settings"
  - After 3rd AI paste cleanup: "Tip: ABScribe inline variations available — select text then ⌘⌥A"
  - After first daily note: "Tip: ⌘⇧D opens today's journal entry"
- Display: transient status bar message (4 seconds, dismissible with ✕)

---

### 1.11 Academic / Science Templates

**New file:** `src/lib/academicTemplates.ts`

Templates to add:
- **IMRaD structured abstract** — Background / Methods / Results / Conclusions
- **Journal article shell** — Abstract, Keywords, Introduction, Methods, Results, Discussion, Conclusion, Acknowledgements, References
- **Lab notebook entry** — Date, Protocol, Hypothesis, Materials, Procedure, Observations, Results, Discussion
- **NIH Specific Aims page** — ~1 page with Opening Statement, Specific Aim 1/2/3, Innovation, Impact
- **NSF Project Summary** — Overview, Intellectual Merit, Broader Impacts
- **EU Horizon section** — Excellence, Impact, Implementation
- **Peer review response letter** — Dear Editors / Dear Reviewers, per-reviewer/per-comment accordion structure
- **Author metadata block** — transform: `author-block` with ORCID, affiliation, email, corresponding author flag
- **Data availability statement** — standardized boilerplate variants (open, restricted, on request)
- **Structured abstract** — 4-field form-style front-matter

Register all these in the existing templates sidebar panel and command palette.

---

### 1.12 Writing Goals / Word Count Targets

**Store additions (`documents.ts`):**
- `wordCountTarget: number | null` (per-document, stored in front-matter)
- `sessionWordCount: number` — computed from session start baseline
- `writingGoalDeadline: string | null`

**Frontend changes (App.vue):**
- Status bar: show `{wordCount} / {target} words` progress bar when target is set
- Click on word count in status bar → set word count target dialog
- Command palette: "Set word count target"
- Daily goal tracking: persist session stats in `featureUsageCounts`

---

## Sprint 2 — Moderate Complexity

### 2.1 Knowledge Graph Visualization

**New dependency:** `d3` (MIT, well-established — update `docs/dependency-admission.md`)
```
pnpm add d3 @types/d3
```

**New Rust command:** `build_workspace_link_graph(workspace_root: String) -> WorkspaceLinkGraph`

```rust
pub struct WorkspaceLinkGraph {
    pub nodes: Vec<GraphNode>,       // { id, title, path, tag_count }
    pub edges: Vec<GraphEdge>,       // { source: path, target: path, link_text }
}
```

Implementation: batch scan all `.md` files, extract `[[...]]` links → build adjacency list. Reuses `backlinks.rs` scan logic but constructs the full bidirectional graph in one pass.

**Frontend changes (App.vue):**
- New view mode: `'graph'` added to `store.mode` union
- Mode switcher in toolbar
- Content area when `store.mode === 'graph'`: renders a `<div ref="graphContainer">` 
- D3 force-directed simulation: nodes = documents, edges = wiki-links
- Node color by tag; node size by link count; hover shows title + link count
- Click node → opens document; double-click → opens + switches to split mode
- "Rebuild graph" button (calls command again)
- Sidebar panel `'graph'` as alternate (shows graph in sidebar, smaller)
- Command palette: "Open knowledge graph"

---

### 2.2 ABScribe Inline AI Variation Pattern

**Frontend changes (App.vue) — new `inlineVariations` system:**

State refs:
```typescript
const inlineVariationsActive = ref(false)
const inlineVariationAnchor = ref<{from: number, to: number} | null>(null)
const inlineVariationCandidates = ref<string[]>([])
const inlineVariationBusy = ref(false)
```

Flow:
1. User selects text in editor → a floating mini-toolbar appears (CSS positioned, no CodeMirror widget needed for trigger)
2. Mini-toolbar includes: Bold, Italic, AI Rewrite (✦ icon, ⌘⌥A)
3. On AI Rewrite: calls `executeStreamingOllamaPrompt` with prompt: "Rewrite the following text in {N=3} distinct variations. Each variation should preserve the meaning but differ in style, length, or framing. Return as JSON array."
4. Variations stored in `inlineVariationCandidates`
5. Popup displays variations as numbered cards below the selection
6. Click a card: replaces selection with that variation; popup closes
7. Escape: dismisses without change
8. Falls back to the existing AI paste modal if Ollama unavailable

Command palette: "Rewrite selection (3 variations)"

---

### 2.3 Calendar View for Daily Notes

Built as part of the daily notes sidebar panel (§1.2):
- Month grid: 7 columns × 5-6 rows
- Days with `.md` files in `daily-notes/` folder are highlighted
- Today is outlined in accent color
- Prev/Next month navigation
- No new dependency — pure CSS grid

Rust: `list_daily_notes(workspace_root: String, year: u16, month: u8) -> Vec<String>` — returns list of dates (YYYY-MM-DD) that have note files.

---

### 2.4 Document Favorites / Pinned Files

**Store additions (`documents.ts`):**
- `pinnedFiles: string[]` — list of absolute paths, persisted

**Frontend changes (App.vue):**
- Files panel: "Pinned" section at top of workspace browser
- Right-click menu on any file → "Pin / Unpin"
- Command palette: "Pin current document", "Unpin current document"

---

### 2.5 PARA Method Folder Structure

Command palette action: "Set up PARA workspace"
- Creates `Projects/`, `Areas/`, `Resources/`, `Archives/` folders in workspace root
- Creates a `_PARA-INDEX.md` document explaining each folder
- Optionally moves existing files: asks user which category

Frontend only — uses existing `invoke('save_file_as', ...)` to create index docs.

---

### 2.6 Project Tracking per Document Set

**Store additions:**
- Per-document: `wordCountTarget`, `deadline`, `projectStatus` — stored in front-matter
- `projectDashboard: boolean` flag

**New Rust command:** `collect_document_set_stats(paths: Vec<String>) -> DocumentSetStats`
Returns: total words, done count, deadline nearest, status distribution (draft/review/approved).

**Frontend changes:**
- Document set manifest view shows project stats bar
- Command palette: "Set document deadline", "Set word target"

---

### 2.7 CSL Arbitrary Style Support via Pandoc

The `cslStyle` front-matter key is already wired in `compiler_support.rs`. Extension:

**New Rust command:** `list_installed_csl_styles(csl_dir: Option<String>) -> Vec<CslStyleInfo>` — scans Pandoc's `~/.pandoc/csl/` (or user-configured path) for `.csl` files, returns `{id, title, filename}`.

**New Rust command:** `download_csl_style(url: String, dest_dir: String) -> String` — downloads a `.csl` file from a URL the user provides (not guessed).

**Frontend changes (App.vue):**
- References sidebar: replace the 12-option `<select>` with a searchable dropdown that includes:
  - Built-in styles (existing 11)
  - Installed `.csl` files from Pandoc's data dir
  - "Browse installed styles…" option → shows file picker
- When a `.csl` file path is selected, it is written to front-matter as `cslStyle: /path/to/file.csl`

**Backend integration:** In `compiler.rs`, when `citation_style` starts with `/` or `~/` (is a file path), invoke Pandoc with `--csl=<path>` instead of built-in renderer. This requires the existing Pandoc import path to be configured.

---

### 2.8 Style Guide Enforcement

**New file:** `src/lib/styleGuide.ts`

- `StyleGuideRule`: `{ id, description, pattern: RegExp | string[], severity: 'error'|'warn'|'info', suggestion }`
- Built-in rules: passive voice, hedge words, filler phrases, sentence length, jargon
- User-configurable: `store.styleGuideRules: StyleGuideRule[]`

**Frontend changes:**
- Review sidebar: new "Style Guide" sub-panel
- Runs client-side (JS regex scan of document text)
- Shows flagged phrases with suggestions inline
- New Tauri command not needed — pure frontend

---

### 2.9 Unlinked Mentions (Already Planned in §1.3)

Implemented as part of the backlinks panel — see §1.3 `find_unlinked_mentions` command.

---

### 2.10 Minimap

**Frontend changes (App.vue):**
- Toggle via ⌘⌥M or View toolbar button
- CSS-based minimap (not a library): thin right-side strip showing `transform: scale(0.15)` clone of the document in a scrollable container
- Viewport indicator overlay: draggable, synced bidirectionally with editor scroll position
- Store: `showMinimap: boolean`

---

## Sprint 3 — High Complexity

### 3.1 Track Changes / Suggestion Mode

**New Rust command:** `apply_suggestion(original_text: String, suggestion_patch: String) -> String`
**New Rust command:** `reject_suggestion(original_text: String, suggestion_id: String) -> String`

**Suggestion storage:** inline in document as HTML comments:
```markdown
Normal text <!--suggest-delete:id123-->deleted text<!--/suggest-delete--> more text
<!--suggest-insert:id124-->inserted text<!--/suggest-insert-->
```

**Frontend changes (App.vue):**
- New store field: `suggestionMode: boolean`
- When `suggestionMode` is on: all deletions/insertions are wrapped in suggestion markers instead of applied directly
- CodeMirror decoration: renders suggestion markers as strikethrough red (delete) or underline green (insert)
- Accept/Reject per-suggestion via floating controls on hover
- "Accept all" / "Reject all" buttons
- Reviewer name from `store.auditAuthor` tagged in suggestion markers
- Mode toggle: ⌘⌥S

---

### 3.2 Block References (`![[doc#heading]]`)

**Syntax:** `![[path/to/doc.md#heading-id]]` — embed the content of a specific heading section.

**New Rust command:** `resolve_block_reference(workspace_root: String, ref_path: String, heading_id: String) -> String`
- Reads target file, finds heading, extracts content until next same-level heading
- Returns rendered HTML

**Frontend changes (App.vue):**
- Compiler pipeline: pre-pass over Markdown text, replace `![[...]]` references with fetched content (or a placeholder + async fetch)
- Inspector: cursor in block reference → shows source doc link
- Click → navigate to source

---

### 3.3 External Reviewer Share Links

**New Rust command:** `export_html_with_comment_layer(doc_path: String, export_opts: ExportOptions) -> String`
- Generates standalone HTML with embedded CSS for a reviewer overlay
- Reviewer can highlight and annotate using a self-contained JS annotation layer
- Annotations exported as JSON, importable back via new `import_review_annotations` command

**Frontend changes:**
- Export panel: "Share for review" button → generates HTML → prompts save location
- Import annotations: drag JSON file onto review sidebar

---

### 3.4 Canvas / Whiteboard View

**New view mode:** `'canvas'` added to `store.mode`

**New store fields:**
- `canvasNodes: CanvasNode[]` — `{id, type: 'document'|'note'|'image', x, y, w, h, content}`
- `canvasEdges: CanvasEdge[]` — `{id, source, target, label}`
- Persisted to `<workspace>.neditor-canvas.json`

**Frontend changes:**
- Canvas mode: renders SVG/div-based spatial layout
- Drag nodes, resize, connect with edges
- Double-click document node → opens that document in split pane
- "Add current document to canvas" button
- New Tauri command: `save_canvas_state`, `load_canvas_state`

No new JS library needed — pure CSS/SVG positioning.

---

### 3.5 Peer Review Response Letter Workflow

**New template** (in `src/lib/academicTemplates.ts`): structured response letter template (already in Sprint 1.11).

**New functionality:** "Import reviewer comments" — pastes raw reviewer text, parser extracts numbered comments, creates response scaffold.

**New Rust command:** `parse_reviewer_comments(text: String) -> Vec<ReviewerComment>`
- Identifies common reviewer comment patterns: "Comment 1:", "Reviewer 1:", "1.", "RC1:", etc.
- Returns `{reviewer_id, comment_id, text}`

---

## Deferred (Out of Scope for This Plan)

| Feature | Reason Deferred |
|---|---|
| Real-time collaboration | Requires CRDT backend (Automerge/Yjs) — architectural overhaul |
| PDF annotation | Requires PDF.js or native PDF renderer — new product surface |
| Plugin API | Requires stable internal API design — multi-quarter effort |
| Mobile companion | Separate Tauri Mobile or React Native app |
| Cloud sync | Requires server infrastructure, encryption key management |
| Web clipper | Browser extension = separate product build |
| Zotero/Mendeley live sync | Requires Zotero API OAuth + CSL-JSON sync protocol |
| Spaced repetition / flashcards | Nice-to-have, low-leverage for primary workflows |

---

## Implementation Order (Sequenced)

### Phase 1 (Sprint 1 — implement in this session)

Ordered by dependency and value:

1. `docs/plans/neditor-gap-closure.md` — copy this plan (first task)
2. `src-tauri/src/readability.rs` + register command + status bar display
3. `src-tauri/src/daily_notes.rs` + register command + sidebar panel + calendar
4. Backlink panel (frontend only — command exists)
5. `src-tauri/src/task_aggregator.rs` + register command + tasks sidebar panel
6. `src/lib/workflowPresets.ts` + preset picker modal + welcome screen
7. Breadcrumb navigation
8. Focus sentence/paragraph mode (CodeMirror decorator)
9. Context-sensitive slash command categories
10. Feature discovery nudges (store + App.vue)
11. `src/lib/academicTemplates.ts` + register in sidebar
12. Writing goals / word count targets

### Phase 2 (Sprint 2)

13. Knowledge graph: `pnpm add d3 @types/d3` + Rust command + view mode
14. ABScribe inline AI variations
15. Calendar view for daily notes (completes §2.3 alongside §1.2)
16. Pinned favorites
17. PARA workspace setup command
18. CSL arbitrary style support
19. Style guide enforcement
20. Minimap

### Phase 3 (Sprint 3)

21. Track changes / suggestion mode
22. Block references
23. External reviewer share link export
24. Canvas view
25. Peer review response workflow

---

## Files to Create

| File | Purpose |
|---|---|
| `src-tauri/src/readability.rs` | Flesch-Kincaid computation |
| `src-tauri/src/daily_notes.rs` | Daily note file creation |
| `src-tauri/src/task_aggregator.rs` | Cross-document task scan |
| `src/lib/workflowPresets.ts` | Workflow preset definitions |
| `src/lib/academicTemplates.ts` | Academic/science templates |
| `src/lib/styleGuide.ts` | Style guide rules engine |
| `docs/plans/neditor-gap-closure.md` | Copy of this plan for project reference |

---

## Files to Modify Heavily

| File | Changes |
|---|---|
| `src-tauri/src/lib.rs` | Register ~8 new Tauri commands |
| `src-tauri/src/bibliography.rs` | Add CSL file path support |
| `src-tauri/src/compiler_support.rs` | Route CSL file paths to Pandoc |
| `src/App.vue` | ~15 new sidebar panels, welcome screen, breadcrumbs, focus mode, graph view, ABScribe, nudges |
| `src/stores/documents.ts` | ~20 new state fields |
| `src/lib/qualityRecommendations.ts` | Replace heuristic with real FK scoring |
| `package.json` | Add `d3`, `@types/d3` |
| `docs/dependency-admission.md` | Record d3 admission |

---

## Verification

After each sprint, verify:
1. `pnpm build` completes without TypeScript errors
2. `cd src-tauri && cargo build` compiles without warnings
3. Manual smoke test: open NEditor, trigger each new feature
4. Regression check: Writer/Pilot mode toggle, transform rendering, AI paste, export to PDF

For readability: validate FK scores against known reference texts (8th-grade text should score ~60 ease / grade 8).

For knowledge graph: open a workspace with 5+ linked documents, verify graph renders with correct edges.
