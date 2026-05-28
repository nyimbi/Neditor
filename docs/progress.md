# NEditor Goal Progress Log

Updated: 2026-05-28

## Active Goal

Complete the full development of NEditor against:

- `docs/specification.md`
- `docs/external-transforms.md`

The goal is not complete until the implementation, documentation, tests,
workflow verification, export artifacts, platform evidence, and committed
progress records prove the requested end state.

## Current Repository State

- Branch: `main`
- Latest inspected committed baseline before this update: `439de25 Prove
  advanced multi-cursor commands`
- Remote alignment at inspection time: `main...origin/main`
- Worktree before this log update: clean and aligned with `origin/main`.

## Durable Planning Artifacts

- `docs/todo.md`: current prioritized completion backlog.
- `docs/spec-completion-matrix.md`: conservative spec-to-evidence matrix.
- `docs/progress.md`: this committed progress log.

## 2026-05-28 Verification Update

Table cursor-follow source editing verification:

| Check | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 87 frontend unit/static tests passed, including source-cursor sync gating, row cell-count tracking for author-added table cells, and static UI guards for **Follow source cursor** wiring. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the table source cursor-follow wiring. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting cursor-follow table editing. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording the table editor cursor-follow evidence. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow still passes for the broader two-way table workflow after the cursor-follow change. |

AI provenance native-smoke proof update:

| Check | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 87 frontend unit/static tests passed, including static guards that the launched native workflow collects reviewed AI source provenance, reviewed AI-assisted section provenance, Review-sidebar evidence, provenance export readiness, and exported HTML appendix evidence. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the native workflow started authoring reviewed AI provenance, preparing HTML provenance export, and carrying the evidence through smoke progress reports. |
| `pnpm run build` | Pass | Vite production build completed after the AI provenance workflow wiring. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Rebuilt the release binary at `src-tauri/target/release/neditor` with the updated native workflow smoke. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked native_workflow_ai_provenance_fixture_exports_reviewed_appendix --lib` | Pass | Focused native artifact proof compiles the same reviewed AI provenance fixture, verifies export readiness with `includeProvenance`, checks semantic source/section records, and asserts HTML/text export appendices carry audit-readable source, section, reviewer, timestamp, and prompt summary evidence. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 NEDITOR_DESKTOP_SMOKE_TIMEOUT_MS=120000 NEDITOR_DESKTOP_SMOKE_ATTEMPTS=1 pnpm run test:desktop-smoke` | Host launch failure before in-app report | The command path passed the native command workflow, but the launched app survived until timeout without writing `.tmp/desktop-smoke/native-window-report.json`, `.tmp/desktop-smoke/native-ui-report.json`, or `.tmp/desktop-smoke/native-workflow-report.json`; launch stderr showed macOS `-10827` and `com.apple.hiservices-xpcservice` connection errors before the WebView smoke code produced an accepted report. |

BibTeX and CSL style fidelity:

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked compiler_preserves_scientific_and_medical_csl_alias_intent --lib` | Pass | Nature, AMA, and Elsevier-Vancouver aliases now stay distinct numeric citation styles and render bibliography entries with journal, volume, issue, page, year, and DOI metadata instead of collapsing to generic Vancouver output. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked bibtex_transform_renders_bibliography_preview --lib` | Pass | BibTeX transform previews now surface richer real-world metadata including edition, ISBN, ISSN, and abstract in addition to existing author/year/publisher/journal/volume/issue/pages/DOI/URL fields. |
| `pnpm run test:unit` | Pass | 87 frontend unit/static tests passed, including citation-style preference normalization for `nature`, `AMA`, and `elsevier-vancouver`. |

Transform option alias architecture:

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked compiler_resolves_transform_option_aliases_for_graph_fences --lib` | Pass | Transform engine path, trust, and input-mode settings keyed as `graph` now resolve through the canonical `dot` artifact for a `graph` fence, proving preference aliases follow the same transform alias model as rendered fences. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked external_transform_tests --lib` | Pass | All 16 external-transform tests passed, including trust gating, cache reuse/invalidation, Graphviz alias inheritance, graph option aliases, adapter argument shaping, missing/disabled engine diagnostics, timeout handling, and installed-engine conformance when available. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked transform_tests --lib` | Pass | All 54 transform tests passed after the option alias change, covering first-release transform registry behavior, aliases, embedded fallbacks, structured data, diagrams, charts, visual data, and business transforms. |

Core transform fidelity documentation:

| Check | Result | Evidence |
| --- | --- | --- |
| `docs/markdown-extensions.md` | Updated | The author-facing transform documentation now states the first-release native Mermaid and Pikchr subsets explicitly: simple Mermaid `flowchart`/`graph` edges and labels, simple Pikchr business shapes/arrows/connector labels, warnings for unsupported syntax, and the need for a trusted Pikchr executable or external workflow when exact grammar/layout fidelity is required. |

## Completed Recently

Recent pushed checkpoints visible in current git history:

- Table editing now follows the Markdown source cursor by default when the
  Tables panel is open and no dirty table draft is waiting. This closes a
  practical two-way editing gap: a user can type or select a Markdown pipe
  table in the document text and have the visual table editor load that source
  table automatically, while a visible **Follow source cursor** toggle lets
  users opt out when they want manual table selection. Direct source-cell edits
  also track the actual cell count on the row, so author-added extra cells can
  be edited without being rejected as outside the normalized header count.
- Editor keybinding proof now covers more of the Emacs/Vim muscle-memory
  surface. Emacs `Ctrl+K`/`Ctrl+Y`/`Ctrl+W` editor chords are reserved from the
  global command-palette shortcut while focus is in CodeMirror, and the focused
  Chromium workflow proves Emacs kill/yank plus Vim `yy`, `p`, `P`, `b`, and
  `e` behavior through the real editor.
- Multi-cursor browser proof now covers the full command family. The focused
  Chromium workflow discovers Add Cursor Above/Below, Select Next Occurrence,
  Select All Occurrences, and Split Selection Into Line Cursors through the
  command palette, then proves simultaneous term replacement and selection
  splitting across every selected line in the editor.
- External file watching now treats every open saved document as a watched root,
  not only the active editor tab. The native watch request accepts `open_roots`,
  inactive clean root edits are reloaded into their matching open document
  without disrupting the active editor, and focused Chromium proof covers
  inactive-root reload, Save As watcher-root moves, and closing the active
  watched tab.
- Table editing is now exposed as a two-way workflow from the main toolbar as
  well as the Tables panel, Writing Tools menu, and command palette. Users can
  open the table editor, load the table at the cursor, select the Markdown table
  source for direct text editing, edit a specific source cell, apply grid
  changes back to text, and sync edited Markdown source back into the grid. The
  Tables sidebar now also tracks the current Markdown editor cursor cell so
  users can see the exact source cell they are about to edit before applying a
  direct text change.
- Button help now gives disabled table/export actions an explicit hover hitbox
  while still describing the real disabled button with `aria-describedby`.
  Chromium workflow proof now covers the delegated tooltip path and the two-way
  table editing path.
- Multi-cursor editing now includes first-class select-all-occurrences and
  split-selection-into-line-cursors commands in the Edit menu, command palette,
  and writing toolbar, backed by pure helper tests for repeated-term selection
  and per-line cursor ranges.
- Vim-style editing now has first-class yank/paste register behavior. The
  editor supports pending `y` operators for `yy`, word motions, and line
  motions, plus `p`/`P` paste placement with linewise register semantics, and
  focused frontend unit coverage proves the word, line, and paste helpers that
  keep modal edits predictable.
- Spec-completion evidence now produces a machine-readable closure plan instead
  of only a flat partial-row count. `pnpm run check:spec-completion` writes
  `.tmp/spec-completion/report.json` plus `.tmp/spec-completion/gap-plan.md`,
  classifies each open matrix row as local implementation/proof,
  documentation-proof, manual review, external evidence, cross-platform
  evidence, release credentials, or distribution artifacts, and lists the first
  local actions so release gap closure can proceed row by row.
- BibTeX transform previews now preserve and show richer reference metadata.
  The bibliography parser records entry type plus common scalar fields, and the
  embedded BibTeX transform renders author/year alongside publisher, journal or
  book title, volume, issue, pages, DOI, and URL rows so reference packs are
  inspectable before export.
- D2 native diagram previews now handle more realistic business diagrams without
  requiring an external engine. The fallback renders declared node labels, edge
  labels, and semicolon-separated edge statements, while ignoring common D2
  layout/style attributes such as `direction`, `shape`, and `style.*` so those
  settings do not appear as bogus diagram nodes. Cross-target export conformance
  now carries the same D2 artifact through HTML, PDF, DOCX, PPTX, and Markdown
  bundle evidence.
- Vega-Lite native static previews now support `tick` marks for compact
  distribution, risk-score, and QA dashboards. Tick marks preserve the existing
  x/y field handling, color-series legend metadata, axis titles, and
  export-safe SVG data values instead of forcing users to install a full Vega
  renderer for common score-strip previews, and the Templates panel now includes
  a ready-made risk-score tick plot starter.
- Vega-Lite native static previews now also support `text` marks with
  `encoding.text.field`, so milestone/readiness plots can place short labels at
  x/y positions while retaining color-series metadata and export-safe values.
  The Templates panel includes a matching readiness label plot starter.
- Vega-Lite native static previews now accept common scatter aliases `circle`
  and `square`. The renderer preserves color-series metadata and axis titles,
  and the Templates panel includes an opportunity scatter starter for account
  and portfolio scoring workflows.
- Vega-Lite static scatter previews now honor `encoding.size.field` for
  bubble/symbol sizing, preserving the authored size as export-safe SVG
  metadata while scaling circle and square marks for portfolio comparison.
- Table editing now has a text-cell path in addition to the visual grid and
  editable source block. **Edit Table Cell at Cursor** locates the Markdown
  header/body cell under the source cursor, loads its value into the Tables
  panel, and writes the edited value directly back into the Markdown row while
  preserving escaped pipe cells and refreshing the visual table draft.
- Release evidence validators now classify prior-commit external evidence as
  `stale` instead of hard-invalid while still rejecting malformed or bad
  current-commit evidence. Platform, AI provider/runtime, security review,
  performance profile, and Google Docs import checks now keep local readiness
  refreshes usable after source commits without accepting stale proof. The tab
  bar also shows the file name beneath semantic document titles when they
  differ, so Save As/Rename/Pin workflows keep file identity visible, and the
  visual table action is labeled **Apply table** to avoid ambiguity with
  **Apply source text**.
- Release-state export manifests now carry approval metadata as first-class
  audit fields: `approved_by`, `approved_at`, `owner`, and `release_target`.
  Export preview surfaces that release/audit line before writing artifacts, and
  focused export workflow proof checks that approved PPTX readiness exposes the
  release target and approver in both the manifest preview and visible export
  preview summary.
- Table editor manual QA now has an executable sign-off contract. `pnpm run
  check:tables:manual` writes `.tmp/table-editor/manual-review-template.json`
  and `.tmp/table-editor/manual-review-summary.json`, validates completed
  reviewer sign-off with current commit/source cleanliness plus prerequisite
  report hashes, and surfaces the remaining supported-host/human-review work
  through release readiness and the release evidence kit.
- Rendered export audit now includes an edited-table review case. The generated
  `review-cases/edited-tables` artifacts cover table-editor style output with
  edited source text, formula rows, escaped pipes, and alignment across HTML,
  PDF, DOCX, PPTX, and Markdown bundle targets, and the audit validator now
  requires that case.
- Rendered export audit now includes a generated TOC/page-number review case.
  The generated `review-cases/toc-page-numbers` artifacts cover numbered
  generated TOC entries, PDF page-number leader text, native DOCX TOC-field
  output, and manual review links across HTML, PDF, DOCX, PPTX, and Markdown
  bundle targets, and the audit/release-readiness validators now require that
  case.
- Rendered export audit now includes a brand/layout review case. The generated
  `review-cases/brand-layout` artifacts cover cover-logo rendering, brand
  color styling, compact landscape Letter page layout, header/footer template
  expansion, page-number fields, watermark metadata, legal disclaimer text, and
  brand metadata across HTML, PDF, DOCX, PPTX, and Markdown bundle targets, and
  the audit/release-readiness validators now require that case.
- Rendered export audit now includes a business-transform review case. The
  generated `review-cases/business-transforms` artifacts cover first-release
  safe `roadmap`, `adr`, `diff`, and `qr` transform output, embedded artifact
  hashes, ADR decision text, diff additions, and QR target text across HTML,
  PDF, DOCX, PPTX, and Markdown bundle targets, and the
  audit/release-readiness validators now require that case.
- Rendered export audit now includes an equation review case. The generated
  `review-cases/equations` artifacts cover inline math, matrix equations,
  extended notation, piecewise equations, equation captions, and equation
  cross references across HTML, PDF, DOCX, PPTX, and Markdown bundle targets,
  and the audit/release-readiness validators now require that case.
- Include watcher state now has focused browser proof for directive edits. The
  workflow edits a master document from `!include chapters/risk.md` to
  `!include chapters/ops.md`, proves the include graph and watched path set
  drop the old child and add the new child, ignores a stale watch event from
  the old include, and recompiles when the new include changes.
- File watcher roots now resync immediately after path-changing saves and
  renames. `saveActive` and `renameActive` recompile after the document path
  changes so the watch root moves to the current file, and the focused Chromium
  workflow proves Save As drops the old root, watches the new root, ignores
  stale old-path events, and reloads clean changes from the new path.
- Close-tab watcher cleanup now has focused browser proof. Closing the active
  watched tab switches to the previous tab, recompiles the new active document,
  moves the watcher root to that file, ignores stale events from the closed tab,
  and still reloads clean external changes from the active tab.
- Supertonic read-aloud setup now has browser workflow proof for the
  consent-gated model download path. Settings exposes an editable model storage
  path, the model notice shows name, size, storage location, source, and
  command, and the focused Chromium workflow proves download/playback stay
  disabled until acknowledgement before running mock download, read-aloud, and
  runtime inspection paths.
- Text-to-speech setup logic is now extracted into `src/lib/ttsSetup.ts` with
  direct unit coverage for engine labels, runtime summaries, Supertonic model
  download plans, copyable download details, acknowledgement prompts, and native
  read-aloud request values.
- Docs Live review packet Markdown now lives in `src/lib/docsLive.ts` instead
  of local Vue glue. Draft history, copy, and insertion paths share the same
  helper, and direct frontend unit coverage proves the audit fence, generated
  metadata, review queues, assumption register, humanization checklist, and
  reviewer handoff sections are emitted consistently.
- Business-document and RFP local-agent handoffs now include Google Antigravity
  alongside Claude Code, Codex, and OpenCode. The Templates wizard, RFP wizard,
  wizard context package, README, user guide, and spec matrix now describe the
  same governed local-agent set already supported by the AI provider
  configuration surface.
- Native local-agent handoff preparation now allowlists Google Antigravity as
  well. The Tauri command writes Antigravity handoff Markdown under
  `.neditor/agent-handoffs`, returns the `antigravity` launch command, and keeps
  the same reviewed-Markdown-only instructions used by the other local-agent
  profiles.
- Table editing now exposes direct Markdown-text editing as a first-class
  two-way action. The Tables sidebar has an explicit **Edit Markdown in text**
  control that can load the table at the editor cursor, select the exact source
  table range, and the focused Chromium workflow proves replacing that text
  with a valid pipe table refreshes the visual grid cells.
- Table editing source-block edits now preview into the visual grid
  immediately when the typed Markdown is a valid pipe table, while invalid or
  partial source remains editable until the user fixes, applies, or cancels it.
- Advanced layout directives now support explicit `columnGap`/`column-gap`
  control for multi-column sections, carrying the value through preview CSS,
  PDF reflow, DOCX section column spacing, PPTX/PDF summaries, manifests, and
  Markdown bundle evidence.
- Native `chart` transforms now handle business variance charts with negative
  values, true zero baselines, target/goal/benchmark lines, and value
  prefix/suffix/unit labels so profit/loss, budget, and cash-flow charts remain
  readable without requiring Vega-Lite. The same native chart syntax now also
  supports multi-series bar, line, and area charts for budget-vs-actual,
  revenue-vs-cost, segment, and scenario comparisons.
- CSV and TSV transforms now tag numeric columns and cells with export-safe
  alignment and `data-format`/`data-value` metadata for plain numbers,
  currencies, percentages, and negative values while preserving the authored
  table text.
- Direct text-edited tables now accept common Markdown pipe-table syntax with
  or without outer pipes. The frontend table editor parses `A | B` / `--- |
  ---` source edits into the visual grid, and the backend compiler normalizes
  those text-edited tables into canonical pipe tables before formula
  evaluation, preview rendering, and export.
- Docs Live production-script creation now has browser workflow proof for
  structure-first podcast and movie-script gates. Podcast drafts block prose
  until episode architecture is approved, movie scripts block prose until
  screen-story architecture is approved, and both switch to sequential segment
  or beat drafting after the structure is supplied.
- Docs Live long-form creation now has browser workflow proof for the outline
  and plot-first gates. The focused Chromium workflow opens Docs Live in
  technical-textbook and novel modes, verifies prose is blocked while the
  textbook architecture or plot architecture is only suggested, then supplies
  an approved structure and verifies sequential chapter drafting becomes ready.
- OpenAPI transform rendering now shows more of the server contract reviewers
  need to see. Top-level servers include server-variable defaults, enums, and
  descriptions, while operation labels surface operation-level server overrides
  and path-level server fallbacks so generated API references do not hide
  tenant, region, or read-replica routing details.
- Release readiness now rejects stale release-evidence-kit reports instead of
  accepting a kit generated for an older commit. The aggregator recomputes the
  current Git HEAD and clean-tree state, then fails the evidence-kit check if
  the embedded kit commit does not match or the current checkout is dirty.
- Two-way table editing now protects unsaved source-block and visual-grid edits
  from context switches. While a table edit is dirty, table selection, edit at
  cursor, and new-table actions are disabled or guarded until the user applies,
  updates, reloads, or cancels the current table edit.
- Agent Workspace AI step assistance now has browser workflow proof. The
  focused help/workflow test opens the Agent Workspace, plans a multi-lane
  workflow, clicks **Add answer and replan**, verifies the editable context
  answers receive the suggested answer, rationale, and context signals, and
  then continues through packet generation and provider handoff.
- Versioning release tags now refresh the Git status panel immediately after
  tagging. The focused browser workflow also proves snapshot restore, refreshed
  release-tag summary, cleared tag input, Git-history restore, and the
  pre-restore safety snapshot row.
- The packaged `ned` CLI now has a headless RFP analysis and response workflow
  for procurement teams. `ned rfp-response` / `ned analyze-rfp` accepts
  Markdown/stdin, PDF, DOCX, or URL sources through native RFP intake, extracts
  requirements plus stated and implied buyer intent, timelines, budget hints,
  evaluation criteria, mandatory attachments, risks, and verification
  questions, then writes a response draft, a compliance matrix, or schema
  `neditor.ned-rfp-response.v1` JSON output for automation.
- Table editing is now explicitly two-way inside the Tables sidebar, not only
  through the main Markdown editor. The loaded or newly created table has an
  editable **Markdown source** block; users can type pipe-table text, parse it
  back into the visual grid, regenerate source text from grid edits, and apply
  the source text into the document.
- Direct in-text table editing is now more forgiving while users are still
  typing. If the original pipe-table range temporarily stops parsing, the
  Tables panel keeps the visual draft, labels the source as not currently
  parseable, prevents accidental context switches, and lets users either fix
  the Markdown text or explicitly apply the draft over the original range.
- Delegated button help now has runtime proof, not only static guardrails. The
  focused browser accessibility workflow verifies tooltip text on hover,
  keyboard focus handoff between toolbar buttons, hiding after focus leaves
  buttons, and disabled-button guidance for table actions.
- Delegated button help now exposes the active tooltip through a stable
  `aria-describedby` relationship on the hovered or focused button, clears that
  relationship when focus/hover leaves the control, and keeps static guardrails
  aligned with the dynamic CodeMirror editor label pattern.
- Table editing now has source-sync protection beyond command discoverability.
  The visual table editor records the Markdown source range it loaded, auto
  reloads clean visual drafts when the user edits the table directly in source
  text, and warns before a dirty visual draft can overwrite a concurrently
  changed source table. Users get explicit **Reload from source** and **Apply
  draft over source** choices when both sides diverge.
- The table source synchronization behavior has been moved into direct-tested
  helpers in `src/lib/tables.ts`: source extraction, draft Markdown
  serialization, source snapshot creation, and source-change detection now have
  reusable unit coverage instead of living only in the workbench component.
- Table editing now works as a two-way source workflow. The Tables panel can
  load the Markdown table at the current source cursor or selection, show the
  exact source-line range being edited, jump back to that source table, and
  apply visual-grid edits back over the original Markdown table range instead
  of only inserting new tables.
- The same two-way table editing workflow is now first-class in the command
  surface. Users can open the visual table editor, edit the table at the
  current cursor or selection, jump back to the source Markdown table, import
  CSV/XLSX data, export CSV/XLSX, and insert SQL table transforms from the
  Tables panel, Writing Tools menu, command palette, toolbar, and native
  desktop Writing Tools menu instead of hunting for a single sidebar path.
- The Templates sidebar now provides **AI template assistance** for calculation
  and transform work. `buildTransformTemplateAssistance` uses the current
  template filters, document text, custom-template draft, and assistance notes
  to suggest a template choice, fill-value plan, preview/verification pass, and
  reviewer handoff. Users can accept one or all suggestions into editable
  transform notes and insert those notes into the document before relying on a
  calculation, chart, diagram, SQL query, or other transform in business prose.
- The Review sidebar now provides **AI quality assistance** on top of the
  deterministic QA/QI recommendations. `buildQualityStepAssistance` turns the
  current findings, document title, export target, document length, and review
  notes into suggested triage, evidence-review, humanization, and reviewer
  handoff answers with rationale and context signals. Users can accept one or
  all suggestions into editable quality review notes and insert those notes into
  the document as a review handoff artifact.

- Quality recommendations now have a dedicated typed implementation module
  instead of living only inside the workbench component. `src/lib/qualityRecommendations.ts`
  builds deterministic QA/QI findings for diagnostics, placeholders, citation
  evidence, unresolved comments, AI provenance, missing document identity, weak
  structure, readability, and generic AI wording; formats the summary; and
  generates the insertable QA report Markdown. Frontend unit coverage directly
  verifies blocker/risk/improvement classification, pass-state behavior, and
  report generation, reducing the QA feature's coupling to `src/App.vue`.
- Release readiness checks now have a dedicated typed implementation module
  instead of living only inside the workbench component. `src/lib/releaseReadiness.ts`
  builds local publish/release gate checks for status, ownership metadata,
  approval audit fields, unresolved review comments, change notes, and
  unreviewed AI provenance; formats the status summary and Help guidance; and
  generates the insertable release audit Markdown. Frontend unit coverage
  directly verifies both missing-governance and approved-document paths.
- Export distribution metadata checks now have a dedicated typed implementation
  module instead of living only inside the workbench component.
  `src/lib/exportMetadataChecklist.ts` builds target-specific publishing,
  release-approval, canonical URL, language, tag, and EPUB creator/outline
  preflight items; formats checklist summaries; and supplies Help guidance for
  the Export sidebar. Frontend unit coverage directly verifies blog publishing
  metadata and EPUB handoff readiness decisions without mounting the UI.
- Front matter reading and update helpers now have a shared typed
  implementation module. `src/lib/frontMatter.ts` owns scalar/list reads,
  multi-key lookup, exact-key matching, scalar upsert/removal, and list upsert
  behavior used by the workbench, quality recommendations, release readiness,
  and export metadata preflight. Frontend unit coverage verifies exact-key
  safety so fields such as `statusNote` are not accidentally treated as
  `status`.
- Docs Live textbook and novel wizards now start with structure instead of
  prose. Technical textbooks lock textbook architecture, chapter order,
  prerequisites, learning outcomes, examples, exercises, and assessment logic
  before sequential chapter drafting and instructional quality review. Novels
  lock plot architecture, character arcs, world rules, chapter order, and
  continuity promises before sequential chapter drafting and narrative quality
  review. The business-document templates and Agent Workspace quality gates use
  the same outline/plot-first contract, and frontend unit coverage verifies the
  generated drafts, template metadata, and quality gates.
- The long-form textbook and novel wizard output now includes first-class
  architecture or plot approval gates, required planning artifacts, sequential
  chapter draft queues, per-chapter acceptance criteria, and final instructional
  or narrative quality review checklists so users can plan first, approve the
  structure, then flesh out chapters in order.
- Podcast and movie script wizards now use the same staged creation contract:
  podcast scripts lock episode architecture and a segment rundown before
  sequential segment drafting and audio production quality review, while movie
  scripts lock screen story architecture and a beat sheet before sequential beat
  drafting and screenplay quality review. Agent Workspace quality gates,
  business templates, Docs Live generated drafts, and focused frontend unit
  coverage now prove those creative wizards are no longer simple outline-only
  starters.
- Docs Live now adds AI assistance to every questionnaire step by generating
  context-aware suggested answers from the selected document type, title,
  outline, placeholders, dictated direction, and existing context. Users can add
  one suggested answer or append the full set before generating, keeping the AI
  guidance editable and reviewable instead of silently filling the document.
- The packaged `ned` CLI now has a first-class reusable business profile setup
  surface. `ned init` scaffolds `.neditor/business-profile.json`, and
  `ned profile` / `ned business-profile` can initialize, update, dry-run, and
  print repeated sender, company, client, website, industry, and brand-voice
  values as JSON, Markdown, or Docs Live placeholder text. The field parser
  accepts documented camelCase keys plus spaced, dashed, underscored, and common
  business aliases so non-technical setup scripts can reuse the same profile in
  templates, snippets, Docs Live, and local-agent handoffs.
- The same packaged profile setup surface now supports script-friendly field
  discovery and single-value retrieval. `ned profile --fields --json` lists the
  reusable identity fields with aliases and intended uses, while
  `ned profile --workspace . --get companyName` returns just one value for help
  desk scripts, setup wizards, document generators, and local-agent handoffs.
- Agent Workspace plans now include AI assistance at every planned workflow
  step. Each creation, composition, edit/revision, review, and distribution
  step receives a context-aware suggested optimal answer, rationale, and context
  signals from the document intent sheet, source pack, reusable document memory,
  outline variants, quality gates, missing inputs, and export targets. The UI
  renders the suggestions as an editable assistance layer so users can add one
  into the context answers and replan instead of letting the system silently
  invent business facts, and generated agent packets carry the same
  `AI Step Assistance` evidence table. Accepting a suggestion now uses a shared
  `appendAgenticStepAssistanceContext` helper that preserves existing context,
  appends the suggested answer, and carries rationale plus context signals into
  the next replan.
- Docs Live drafting workflows now carry AI next-best-action assistance on every
  systematic step: outline approval, context capture, sequential drafting, QA,
  humanization, and review handoff. The same stage assistance appears in the UI
  and generated Drafting Plan table so users can see the suggested next action
  and context signals while moving through the document creation process.
- Docs Live questionnaire suggestions now expose the rationale and context
  signals behind each suggested optimal answer, and accepting a suggestion
  carries the answer, rationale, and signals into the editable answer block so
  users can review the AI advice before drafting.
- Agent Workspace step assistance now makes the accept/replan loop one action:
  **Add answer and replan** appends the selected context-aware optimal answer
  with rationale and context signals, immediately rebuilds the plan from the
  updated answers, clears stale run/provider state, and reports the replanned
  step count. Focused frontend unit/static coverage proves the appended answer
  becomes the next plan context and that every replanned step still receives
  context-aware assistance.
- Business document builders now surface AI assistance at each wizard step
  before a user opens Docs Live or prepares a local-agent handoff. Proposal,
  RFP, RFQ, tender, tutorial, lesson, textbook, novel, podcast, movie, business
  case, and executive brief cards show context-aware suggested optimal answers
  with rationale and context signals for identity, intent, outline approval,
  sequential drafting, QA, and humanization; the same assistance is included in
  the Docs Live wizard context so non-technical users can adapt the suggestions
  instead of starting from blank prompts.
- The References sidebar front-matter manager parser is now more tolerant of
  business-document YAML edge cases: CRLF front matter, quoted `#` characters,
  comma-containing inline data-source lists, `yml` aliases, URL/Windows/parent
  traversal path blocking, filenames that legitimately contain `..`, compact
  `dataSources` objects such as `{name, path, type}`, and top-level inline
  `dataSources: {...}` objects, `dataSources: [{...}]` lists, or inline scalar
  path lists such as `dataSources: [data/customers.csv]`; single-file scalar
  declarations such as `dataSources: data/customers.csv` are also inventoried.
  Compact data-source objects can also reuse scalar anchors plus inline or
  block-map `<<: *defaults` merge maps for repeated `name`, `type`, or `kind`
  values, and expanded data-source rows can start with `- <<: *defaults` before
  overriding fields such as `path` or `name`; expanded row fields also resolve
  scalar aliases such as `path: *sourcePath`. Custom tags, including
  tag-handle forms such as `!docs!sources`, on compact data-source objects,
  inline lists, and alias lists are ignored for inventory purposes, and
  anchored compact objects inside inline lists can be reused by later compact
  data-source rows. Top-level `dataSources: *alias` declarations can now reuse
  anchored source maps, source paths, inline source lists, or block source
  lists, and legacy alias sections such as `csvFiles`,
  `jsonFiles`, and `ymlFiles` resolve scalar, inline-list, and block-list
  aliases plus direct single-file scalar declarations. Anchored source maps with
  nested lists remain map defaults instead of being misclassified as
  source-list aliases, and aliased block-list source rows can start with
  `<<: *defaults` merge defaults. Dotted, slash, and URI-style namespaced
  anchor names such as `&source.defaults`, `&business/client.defaults`, and
  `*urn:source/path.default` now resolve in data-source defaults and scalar
  source paths.
- The same front-matter manager now resolves simple and namespaced scalar
  anchors and aliases and surfaces folded/literal block scalars as usable
  document-variable values, so repeated owners/reviewers and short multiline
  summaries can be inserted through the References sidebar instead of
  disappearing from the variable list. Dotted metadata keys such as
  `client.name`, nested `account.owner.name`, and inline map keys such as
  `{proposal.dueDate: 2026-07-01}` now surface as the same dotted variables
  users insert in document templates, while excluded roots such as `brand.*`
  remain hidden from the document-variable manager.
- The Rust compiler now resolves literal dotted front-matter keys through the
  same `{{client.name}}` variable syntax and export metadata lookup path. True
  nested maps still take precedence when both `profile.name` and
  `profile: {name: ...}` forms exist, and sibling dotted fallback keys such as
  `profile.owner` still resolve when the nested `profile` map does not contain
  that leaf, so document authors can use compact dotted metadata without
  breaking explicit structured YAML. The same lookup now has project-variable
  coverage, so `.neditor/variables.yaml` can supply compact dotted defaults
  like `profile.owner` and `layout.header` without overriding explicit nested
  front matter.
- Simple front-matter merge defaults and tagged scalars are now handled in the
  document-variable inventory. Common YAML such as `<<: *defaults`, `!!str`,
  `!custom`, `!docs!channel`, and `!<tag:yaml.org,2002:str>` can populate
  repeated client, owner, reviewer, budget, or region variables while explicit
  values still win.
- Front matter typed scalar handling now does more than strip tags: `!!null`
  and `!<tag:yaml.org,2002:null>` inventory as empty values, `!!bool` and
  `!<tag:yaml.org,2002:bool>` normalize common yes/no/on/off forms to
  `true`/`false`, and `!!str null` remains the literal string `null`. Anchored
  typed defaults also merge into downstream document-variable rows.
- Generated table-of-contents controls now accept structured metadata such as
  `toc: {enabled: true, depth: 2, numbered: true}` in addition to the legacy
  flat `tocDepth` and `tocNumbered` keys, and the DOCX TOC field uses the same
  shared depth lookup.
- Generated-section markers are now fence-aware: Markdown examples containing
  `[TOC]`, `[INDEX]`, `[GLOSSARY]`, `[BIBLIOGRAPHY]`, `[LIST_OF_FIGURES]`, or
  `[LIST_OF_TABLES]` stay intact inside code fences while real outside markers
  still render.
- Shared fenced-body collection now accepts both backtick and tilde fences for
  extension blocks such as `glossary`, `bibtex`, `hayagriva`, `bibliography`,
  and `layout`, keeping author Markdown syntax choices consistent across
  generated sections and citation workflows. Non-target fenced examples are now
  consumed as examples, so nested `glossary` or `bibtex` samples cannot leak
  fake terms or references into the compiled document model.
- Transform fences now use the same shared fence detector: supported transforms
  such as `chart` render from either backtick or tilde fences, while unsupported
  or documentation/example fences are preserved inertly instead of allowing
  nested transform samples to execute.
- Transform template fill-field detection now recognizes tilde-fenced `calc`
  and structured transform templates, so custom templates expose editable values
  consistently with compiler-supported fence syntax.
- The editor UI now uses the shared frontend fence opener for source
  decorations, transform-engine detection, and compact AI/Docs Live previews,
  so tilde-fenced transform and provenance blocks receive the same ergonomic
  treatment as backtick-fenced blocks.
- Agentic workflow evidence now uses the shared frontend fence opener too:
  tilde-fenced `llm-source` provenance blocks count toward human-review
  governance, and tilde-fenced Markdown examples stay out of claim and
  humanization scans. The scan now also skips nested `ai-source` examples inside
  non-provenance fenced examples, avoiding false governance blockers from
  documentation snippets.
- AI provenance collection now follows the same fence policy: `ai-source`,
  `ai-provenance`, and `llm-source` blocks can use backtick or tilde fences,
  while nested provenance examples remain inert and do not create semantic
  provenance records.
- Review actions now use the same frontend provenance fence helper, so marking
  AI source blocks as reviewed works for tilde-fenced `llm-source` and other
  supported provenance aliases without mutating fenced Markdown examples.
- Quality recommendations now recognize tilde-fenced `bibtex`, `hayagriva`,
  and `bibliography` source blocks, keeping deterministic citation-evidence
  warnings aligned with compiler-supported bibliography fences.
- Quality recommendations now strip backtick and tilde fenced examples before
  counting placeholders, citation markers, headings, long paragraphs, and
  generic AI phrasing, so documentation snippets do not create false QA risks.
- Bibliography evidence detection in the same QA pass is now top-level
  fence-aware: real `[BIBLIOGRAPHY]`, raw BibTeX entries, and supported
  bibliography fences still satisfy citation evidence, while bibliography
  examples nested inside fenced Markdown do not.
- Release readiness, backend validation, and backend export readiness now treat
  front matter and semantic release states case-insensitively, so
  business-authored `Approved`, `approved`, `PUBLISHED`, or `published` status
  values are classified consistently before UI checks or target-specific export
  gates run.
- Docs Live now distinguishes user-supplied outlines from generated fallback
  outlines: supplied outlines are marked locked, while document-type fallback
  outlines are marked suggested and must be reviewed before prose is accepted.
- Native RFP DOCX import now scans every Word header/footer part plus footnotes,
  endnotes, and comments in addition to the main document body, so requirement
  language and buyer clarifications outside `word/document.xml` are not silently
  omitted from the RFP response wizard intake.
- The native RFP response wizard now gives each extracted requirement a
  context-aware suggested response answer, including response section, evidence
  owner, proof needed, and reviewer caveat. Suggested answers appear in the
  Templates UI, compliance matrix export, full RFP response draft, verification
  checklist, and local-agent handoff brief.
- Full RFP responses now include a dedicated **Requirement Response Drafts**
  section that groups the suggested requirement answers by target response
  section, keeps evidence owners and proof needs attached to each answer, and
  gives reviewers usable draft prose before the verification checklist.
- The native RFP response wizard now has its own AI step-assistance layer for
  source intake, requirement analysis, buyer intent, response drafting, evidence
  QA, and handoff. Users can accept guidance into editable response-context
  notes, and those notes flow into generated full responses, Docs Live context,
  and local-agent handoff context without altering the source RFP.
- The Configuration Center setup wizard now provides AI setup assistance for
  identity, LLM access, local agents, voice runtime, read-aloud, exports,
  transforms, and release gates. Each setup area shows a context-aware suggested
  answer, rationale, and context signals, and accepted guidance is appended to
  editable setup notes for non-technical setup handoff.
- The Export panel now includes AI Export Assistance for target metadata,
  backend readiness diagnostics, and artifact evidence. Users can accept the
  suggested answer, rationale, and context signals into editable export
  readiness notes, insert those notes into the document as an audit handoff, and
  rerun readiness from the same panel.
- Export readiness now warns when a generated table of contents is requested
  but the document has no headings, and copies that diagnostic into manifest
  readiness with the other empty generated-section warnings.
- Export readiness now also warns when `[BIBLIOGRAPHY]` is present but no
  bibliography entries are available, preventing an empty bibliography section
  from quietly passing release checks.
- Empty generated bibliography markers now render a visible
  `_No bibliography entries found._` placeholder in compiled Markdown/HTML, so
  the readiness warning is also visible at the marker location.
- PPTX agenda generation now follows the same structured TOC metadata path:
  `toc.enabled` and `tableOfContents.enabled` can automatically add an agenda
  slide without requiring the explicit `includeAgenda` export option.
- Generated glossary sections now accept structured `glossary.enabled: true`
  metadata in addition to `[GLOSSARY]`, `glossary: true`, and the legacy
  `glossarySection` aliases.
- Generated caption lists now accept structured front matter such as
  `captionLists.figures: true`, `captionLists.tables: true`,
  `figures.list.enabled`, and `tables.list.enabled`, so business templates can
  request lists of figures/tables without inserting explicit markers.
- Export readiness now warns when generated lists of figures or tables are
  requested but the document has no figures or tables, and carries those
  diagnostics into manifest readiness alongside empty index/glossary warnings.
- Automatic indexes now honor metadata-defined terms from `index.terms`,
  `indexTerms`, `index_terms`, and `index.keywords`, including comma-separated
  strings and structured entries such as `{term, anchor}`, while
  `index.exclude` and `indexExclude` use the same shared metadata lookup and
  continue to suppress forbidden terms. The cross-target export proof now carries
  both prose-linked and explicitly anchored metadata-defined index terms through
  HTML, PDF, DOCX, PPTX, and Markdown bundle outputs.
- Automatic index extraction now uses the shared Markdown fence detector, so
  both backtick and tilde fenced examples are ignored for bold terms, repeated
  proper nouns, explicit `{#index:...}` markers, and metadata-defined term
  anchor lookup.
  Nested scalar defaults such as `address.city` and `delivery.timezone` also
  flow through simple merges as dotted variable names, and anchored maps,
  including dotted or slash-namespaced anchors such as `&client.defaults` and
  `&business/client.defaults`, can inherit dotted-key defaults from other
  anchored maps before being reused by a client or partner block.
- The Rust compiler front-matter path now treats custom YAML tags and
  tag-handle wrappers as metadata decorators too. Tagged scalars, tagged inline
  maps, and tagged `dataSources` lists such as `!docs!sources` are normalized
  before JSON metadata conversion, so compiled document variables, currency
  filters, rendered data-source tables, and include-graph/export-manifest
  evidence still work when business templates use typed YAML annotations.
  Project-level `.neditor/variables.yaml` files now use the same normalization
  path, so tagged `variables:` wrappers and tagged project values merge into
  document metadata without overriding explicit front matter. Project variables
  now merge recursively too: a document can override `profile.name` and
  `profile.address.city` while still inheriting `profile.address.country` and
  `profile.owner` from reusable project defaults.
- Simple inline YAML maps now expand into dotted document variables as well.
  Compact business front matter such as `client: {name: Acme, tier:
  Enterprise}` and anchored defaults such as `defaults: &clientDefaults
  {owner: Strategy}` surface through the References sidebar, including simple
  scalar aliases, nested inline maps such as `address: {city: Nairobi}`, merge
  defaults, direct map aliases such as `client: *clientDefaults`, and
  explicit-value overrides. Inline sequences now surface as indexed dotted
  variables too, so `reviewers: [CFO, Legal]` and `contacts: [{name: Jane}]`
  become insertable values such as `reviewers.0` and `contacts.0.name`. The
  parser also inventories standard `milestones` block-list entries, preserving
  scalar aliases inside list-item fields. Sequence items can now start with
  `<<: *defaults` merge keys, so repeated client/contact rows inherit standard
  defaults while explicit list-item fields override them. Anchored block-list
  rows such as stakeholder records can also be reused elsewhere as direct map
  aliases. Nested inline sequences inside compact maps now expand through
  original declarations, merge defaults, and copied aliases, compact territory
  or approval matrices such as `[[North, South], [East, West]]` are inventoried
  as indexed dotted variables, and compact objects anchored inside inline
  sequence items can be reused later as direct map aliases. Custom tags on
  inline maps and block-list rows are ignored for inventory purposes while
  their scalar fields remain available as document variables.
- Application navigation now exposes NEditor's capabilities through both menus
  and buttons. The workbench header has visible File, Edit, View, Writing
  Tools, Quality, Export, and Help menus that mirror the native desktop menu
  surface, while the command bar groups file operations, writing/insert
  actions, review/navigation, quality, creation wizards, and view controls into
  multiple collapsible toolbars. The new Quality surface scans diagnostics,
  placeholders, citation gaps, unresolved comments, AI provenance, weak
  structure, long paragraphs, and generic AI phrasing, then offers QA review,
  report insertion, and agentic improvement actions from the Review sidebar,
  in-app menus, native menus, and toolbar buttons. Docs Live and the document
  wizard catalog now include lesson plans, lesson content, technical textbooks,
  novels, podcast scripts, and movie scripts, and save/snapshot flows now flush
  current editor text before persisting so menu-driven file actions operate on
  the latest document state.
- Split editor panes are no longer deferred. The View toolbar and Settings now
  expose a persisted Dual source toggle that mounts two synchronized
  CodeMirror source panes for the active document. Edits in either pane use the
  same document store, preview compile, save, watcher, and export path. A
  focused Chromium workflow now opens a long document, edits from both panes,
  verifies peer-pane/live-preview synchronization, and checks that primary-pane
  scroll drives preview sync without secondary-pane scroll hijacking the
  preview position. The launched desktop smoke now records the same class of
  proof in `.tmp/desktop-smoke/native-workflow-report.json` with mounted
  primary/secondary CodeMirror panes, bidirectional pane sync, live-preview
  sync, and scroll-isolation evidence.
- Editor keybinding modes are no longer deferred. Settings now persists
  Default, Emacs-style, and Vim-style modes; Emacs mode uses CodeMirror's
  native Emacs keymap, and Vim-style mode adds visible insert/normal state plus
  common normal-mode navigation/edit commands without adding a dependency.
  The status bar reports the active keybinding mode, and the Help Center
  documents the supported key sets for keyboard-first writers. A focused
  Chromium workflow now proves Emacs line-start/line-end editing, Vim
  insert/normal status transitions, normal-mode character blocking,
  line-start insertion, line-end append editing, and persisted Vim mode after
  reload. The launched desktop smoke now records Emacs and Vim mode
  attributes/status, Emacs line editing, Vim normal-mode blocking, Vim
  insert/append editing, and persisted Vim preference reload.
- Smart Markdown list continuation now covers more business-writing edge cases:
  tab-indented lists, zero-padded ordered lists, uppercase checked task lists,
  and nested blockquoted numbered lists, with width-preserving ordered-list
  continuation in `src/lib/markdownEditing.ts`.
- Windows external-transform setup is now guarded by
  `pnpm run check:external-transform-docs`: the Windows docs cover Graphviz,
  D2, Java/PlantUML, Rust/Pikchr CLI, explicit `.exe` paths, shim guidance, and
  the PowerShell `NEDITOR_TEST_PIKCHR` probe path. The matrix now treats that
  row as Partial pending actual Windows-host evidence.
- Current macOS optional-engine evidence now includes Pikchr again: the
  workspace-built executable under `.tmp/pikchr-build/` passes
  `NEDITOR_TEST_PIKCHR=... pnpm run check:engines`, writes `pikchr.svg`, and
  passes Rust installed-engine conformance through NEditor's external transform
  execution path.
- The specification matrix now treats the recommended first milestone as
  complete with requirement-mapped evidence for the Tauri shell, editor/preview,
  file operations, front matter/includes/TOC, transform registry, Mermaid,
  Graphviz, CSV tables, HTML/PDF exports, export manifests, and external file
  refresh. Later release-host and manual sign-off gaps remain visible
  separately.
- The Templates browser workflow now proves safe business and structured API
  transform insertion beyond calculations and charts: timeline, roadmap, ADR,
  QR, OpenAPI endpoint, and JSON Schema object built-in templates are inserted
  from the Templates sidebar and verified in the transform artifact preview.
- Help now includes an External transform troubleshooting topic for Graphviz,
  D2, Pikchr, PlantUML, and executable transform failures, including
  platform-specific path/permission guidance, timeout/empty-output/trust/cache
  recovery steps, and direct routing to Engine settings, Diagnostics,
  Templates, and Export readiness.
- The Review sidebar now has browser workflow proof for adding unresolved
  review comments, adding change notes, updating the semantic review summary,
  inserting audit markers into Markdown, and resolving comments from the UI.
- Open document tabs now have accessible move-left and move-right controls in
  addition to pointer drag behavior. Reordering is persisted through workspace
  restore, and dragging one tab onto another aligns document-set/pinned grouping
  before placing the moved tab next to the target.
- The workbench now has a richer global shortcut layer for collapsed-toolbar
  and keyboard-first use. Cmd/Ctrl shortcuts cover save/save-as, new/open,
  open-folder, search, generic export, direct HTML export, formatting,
  command-palette discovery, Docs Live, AI Agent Workspace, Review readiness,
  Export readiness, and shortcut help while leaving ordinary form fields alone.
- Open document sets are now first-class workspace objects instead of only tab
  group labels. The Files sidebar includes a Document Sets manager that shows
  open sets, assigns the active document to a set, renames every open document
  in the active set, removes the active document from a set, normalizes legacy
  `document_set`/`set` aliases to `documentSet`, and groups tabs immediately
  from current front matter text so inactive documents do not wait for a
  recompilation before moving. Document sets can now also insert or copy a
  Markdown manifest with source, generated timestamp, workspace, active
  document, open document count, document paths, statuses, save state, pinned
  state, and review handoff instructions.
- The Agent Workspace now builds a structured document intent sheet before
  drafting or distribution. The sheet extracts document type, working title,
  audience, outcome, owner, deadline, tone, evidence, reviewer, approval
  status, distribution targets, and constraints from the instruction, context
  answers, source pack, and current document; surfaces missing fields in the
  plan UI; feeds the AI Control Center and release evidence bundle; and is
  included in generated agent packets, audit fingerprints, persisted run
  history, and filtered run-history audits.
- The Agent Workspace also accepts reusable document memory for terminology,
  style, accepted decisions, rejected directions, reviewer preferences, and
  distribution preferences. Memory items are parsed deterministically, included
  in the context pack, surfaced in the plan UI, carried into lifecycle tasks,
  AI Control Center grounding, release evidence, and run-history replanning.
- Agent section work now includes first-class contract cards. Each outline item
  carries purpose, target reader, desired outcome, evidence expectations,
  accountable owner, risk level, and done criteria into the Agent Workspace UI,
  Docs Live section brief, generated packet, lifecycle task evidence, release
  evidence bundle, and audit fingerprints.
- Agent plans now generate outline variants for comparison before drafting.
  Executive-first, problem-solution, evidence-led, risk-first, and
  channel-specific structures include summaries, best-fit use cases, tradeoffs,
  risks, full outline text, release evidence, lifecycle tasks, AI Control Center
  grounding, generated packet content, and UI actions to load a variant into
  Docs Live or the outline planner.
- Agent runs now include a pre-review rehearsal before formal review. The run
  simulates likely reviewer questions, objections, redlines, and missing
  evidence requests; marks release blockers; feeds lifecycle tasks, release
  evidence, audit fingerprints, generated packets, and the Agent Workspace UI;
  and persists a prompt count into local run history.
- Agent runs now preserve composable section draft history. Each generated
  section version carries a prompt summary, rationale, reviewer notes, section
  and source fingerprints, acceptance status, and reusable restore-point
  Markdown; the Agent Workspace exposes insert/copy/Docs Live reuse actions and
  local run history persists bounded draft-version evidence.
- Agent runs now include a first-class automation scheduler for safe local
  checks. The queue stages evidence scan, outline critique, transform
  validation, export preflight, accessibility review, and readiness refresh
  tasks with owners, triggers, evidence inputs, non-destructive action routing,
  lifecycle/release/audit evidence, and Agent Workspace controls. The scheduler
  now also keeps visible queued/running/complete/blocked state per check, runs
  the safe queue in place without dismissing the agent workspace, records
  per-check results, separates execution from "open surface" navigation, and
  persists the per-check queued/running/complete/blocked breakdown into local
  run history so replanned runs, history summaries, and inserted/copied audit
  packages keep durable automation evidence.
- Clean multi-tab external refresh now catches changes that happen while a file
  tab is inactive and not the current watcher root. When the user switches back,
  the active-tab watcher setup immediately compares the saved root-file hash
  against disk, reloads clean documents, clears the dirty marker, and reports
  `Reloaded external changes`; the full browser suite now passes 64 Chromium
  workflows including the inactive-tab watcher case.
- Preview click-to-source navigation now covers non-heading anchored artifacts.
  Clicking a rendered figure, table, or equation with a stable preview anchor
  uses the same source-target resolver as headings and cross-reference links,
  and the focused browser workflow now proves direct figure, table, and equation
  click-through to the Markdown source lines.
- The AI-first platform roadmap is now an executable release contract. The
  roadmap still names 50 concrete changes, and `pnpm run check:ai-roadmap`
  verifies the item count, ten five-item sections, README/spec-matrix linkage,
  and app/test evidence for Docs Live, Agent Workspace, workflow playbooks,
  source packs, lifecycle tasks, provider handoff, AI runtime readiness, and the
  guided demo.
- The roadmap has been expanded from a checklist into an AI-first product
  blueprint. It now defines a durable capability model covering understanding,
  planning, composition, revision, verification, distribution, and learning,
  then maps 50 concrete changes across intent capture, outline architecture,
  section drafting, editing, evidence, transforms, QA, publishing packages,
  provider operations, governance, reusable memory, and enterprise readiness.
- The Help Center now exposes an in-app "AI-first platform roadmap" topic so
  users can discover how intent capture, outline-first planning, agentic
  editing, grounding, review, provider handoff, distribution, and release
  evidence fit together without leaving NEditor.
- Transform engine settings now show a setup status for native fallback
  diagnostics, untrusted configured paths, trusted paths, and disabled external
  execution. The browser workflow covers these states alongside probe success
  and missing-executable diagnostics so optional engine setup is clearer for
  non-technical users.
- Transform diagnostics now attach tighter source metadata to the fenced block.
  Global compiler diagnostics and per-artifact diagnostics both carry the
  transform name, source-range breadcrumb, start/end lines, and column bounds,
  so preview/sidebar jumps and export manifests have a more precise repair
  target. Row-level CSV/TSV formula diagnostics are now translated from
  transform-body row numbers to absolute Markdown source lines, with the
  diagnostic ending on the exact bad data row instead of the whole fence.
- Front matter data-source diagnostics now carry structured related context for
  the declared source name, path, type, and resolved path where available,
  improving sidebar/export-manifest repair handoffs for missing paths,
  unsupported types, unreadable files, and unsafe path escapes.
- The status bar now reports live preview update telemetry, including elapsed
  compile time and compiled character count. The large-document browser
  workflow verifies that a 120-section edit reaches the preview within the
  timing budget and exposes that timing signal to users.
- The Outline panel is now an editable planning surface, not only a generated
  heading navigator. Users can draft an outline with indented bullets, numbered
  lines, or Markdown heading marks, then create a document skeleton with front
  matter, optional `[TOC]`, heading hierarchy, and draft placeholders before
  writing body content. The command bar and palette expose `Plan document from
  outline`, and the browser workflow now proves the outline-first creation path.
- This update deepens native editor-ergonomics proof in the launched Tauri
  smoke. The app-authored workflow now records structured evidence for
  CodeMirror word statistics, spellcheck/autocapitalize/role attributes, line
  numbers, word wrap, the folding gutter, search-panel opening, replacement
  mutation, Markdown list continuation, bracket-pair insertion, and a
  multi-cursor edit in the native webview.
- Toolbar rows can now be collapsed individually to recover vertical writing
  space. The File, Writing, Review & Navigate, and View toolbars keep compact
  expander buttons when collapsed, the View toolbar exposes Collapse all/Expand
  all, and the collapsed row state persists with the existing toolbar display
  and text-size preferences.
- Native outline navigation is now enforced in the launched Tauri smoke. The
  app-authored workflow opens the Outline sidebar, clicks a real outline-row
  button, and records CodeMirror selection/focus evidence for the matching
  source heading.
- Outline is now also a first-class workbench mode for document planning. It
  hides the source, preview, and sidebar panes, shows only chapter, section,
  subsection, and subsubsection rows, and lets users add, rename, re-level,
  navigate to, and delete document sections before or during drafting.
- The supported-platform Tauri WebDriver harness now includes outline-mode
  structural editing. On Windows/Linux the workflow switches to Outline mode,
  verifies source/preview are hidden, renames, adds, re-levels, and deletes
  headings through the desktop DOM, switches back to source, and records
  `outlineArtifacts` in `.tmp/desktop-webdriver/report.json`.
- This update adds `pnpm run check:a11y:runtime` as a focused runtime
  accessibility audit. It validates that the expected keyboard, landmark,
  modal-focus, live-region, high-contrast/reduced-motion, search, and heading
  workflows exist in `e2e/app-workflows.spec.ts`, executes those six Chromium
  workflows through `scripts/run-e2e.mjs`, writes
  `.tmp/accessibility/runtime-report.json`, and can retry through an installed
  system Chromium-compatible browser if the bundled cache aborts before any app
  assertion runs.
- This update adds `pnpm run check:a11y:manual` as the manual accessibility
  review contract. It writes
  `.tmp/accessibility/manual-review-template.json`, records pending or accepted
  sign-off state in `.tmp/accessibility/manual-review-summary.json`, requires
  the static and runtime accessibility reports before accepting a completed
  reviewer file, and validates reviewer metadata, platform, assistive
  technology, checklist notes, unresolved blockers, and prerequisite report
  SHA-256 identity through `NEDITOR_ACCESSIBILITY_SIGNOFF=/path/to/signoff.json`.
  Stale sign-off files from older static/runtime accessibility reports fail
  validation instead of closing the assistive-technology readiness gap.
- Docs Live now gives the workbench a voice-guided first-draft path. The
  Writing toolbar, Outline panel, command palette, and native Writing Tools menu
  can open a Docs Live dialog that accepts dictation where Web Speech is
  available, freeform document context, placeholder values, generated
  questionnaire prompts, and an existing or planned outline, then creates
  section-by-section Markdown with an explicit drafting workflow table,
  selectable per-section drafting depth, AI provenance, needs-review section
  markers, per-section quality-assurance checks, humanization tasks, and
  reviewer handoff questions.
  The focused browser workflow now exercises the dictation path through a
  SpeechRecognition-compatible harness, and the bounded native launch smoke
  records `nativeMenuCommandEvidence.docsLive.open: true` from the desktop
  Writing Tools menu.
- The browser workflow runner now emits structured release evidence instead of
  relying on exit status alone. `.tmp/e2e-browser/report.json` records schema
  `neditor.e2e-browser-workflow.v1`, parsed Playwright test totals, stdout and
  stderr tails, browser source, command, and workflow flags for Docs Live,
  outline planning, outline CRUD, and export workflows. Release readiness now
  rejects stale browser reports, missing test counts, failed/timed-out tests,
  or reports that do not prove the Docs Live drafting workflow.
- Focused browser audits no longer replace the canonical full-suite browser
  proof. `scripts/run-e2e.mjs` writes `.tmp/e2e-browser/report.json` only for
  the no-argument full suite, marks that report with `scope: "full-suite"`, and
  writes focused runs to either `.tmp/e2e-browser/focused-report.json` or the
  caller's `NEDITOR_E2E_REPORT_PATH`. Runtime accessibility now writes
  `.tmp/accessibility/e2e-runtime-report.json`, performance audit writes
  `.tmp/performance-audit/e2e-large-document-report.json`, and release
  readiness validates those linked focused reports before accepting the
  accessibility and performance checks.
- Native macOS release artifacts now have structured readiness validation.
  Release readiness checks the app-bundle report against the current executable
  and icon files, validates required Info.plist identity/version/icon/high-DPI
  fields, and accepts the DMG host-limitation report only when the hdiutil
  sandbox classification is newer than the app-bundle report and includes app
  bundle fallback proof.
- Native desktop smoke and macOS WebDriver fallback proof now have
  readiness-side freshness checks against the current desktop binary. Release
  readiness re-stats `src-tauri/target/release/neditor` and rejects stale
  `.tmp/desktop-smoke/native-command-report.json`,
  `.tmp/desktop-smoke/launch-report.json`, and
  `.tmp/desktop-webdriver/report.json` artifacts even if an older WebDriver
  fallback report self-reported `freshForBinary: true`.
- The release-grade local baseline now runs both
  `pnpm run check:a11y:runtime` and `pnpm run check:a11y:manual` after the full
  browser workflow suite. This keeps the focused runtime accessibility report
  and manual review template/summary current whenever `pnpm run
  verify:local:full` is used for release evidence.
- This update adds `pnpm run check:release-readiness` as the final
  release-evidence aggregation step. It writes
  `.tmp/release-readiness/report.json`, fails on missing or failed current-host
  reports, and records remaining external gaps such as Windows/Linux package
  artifact proof, Windows/Linux WebDriver execution, signing/notarization,
  optional missing engines, and human reviewer sign-off.
- This update adds `pnpm run check:platform-evidence` as the supported-host
  evidence contract for Windows/Linux package artifacts and Tauri WebDriver
  execution. It writes `.tmp/platform-evidence/report.json`, generates fillable
  JSON templates, treats absent reports as pending external evidence, and fails
  malformed supplied evidence so release readiness cannot accept vague platform
  claims.
- This update adds `pnpm run collect:platform-evidence` for supported
  Windows/Linux hosts. After package generation and `pnpm run
  test:tauri-webdriver`, it scans the real installer/package files, records
  SHA-256 and byte evidence, copies the passing WebDriver report, and writes
  the exact external JSON files consumed by `pnpm run check:platform-evidence`.
- This update adds `pnpm run check:release-signing` as the credentialed release
  evidence contract. It writes `.tmp/release-signing/report.json`, generates
  macOS/Windows/Linux signing templates, treats absent signing/notarization
  reports as pending release credentials, and fails malformed supplied evidence
  so release readiness cannot accept unsigned distribution claims.
- This update adds `pnpm run collect:release-signing` for credentialed release
  hosts. It records signed artifact bytes and SHA-256 hashes, runs required
  verifier commands for the target platform, captures proof summaries, and
  writes the exact `neditor.release-signing-evidence.v1` JSON accepted by
  `pnpm run check:release-signing`.
- This update adds `pnpm run check:google-docs-import` as the live Google Docs
  import/readback evidence contract. It writes
  `.tmp/google-docs-import/report.json`, verifies local rendered Google Docs
  handoff artifact hashes, generates a Drive import evidence template, treats
  absent Drive proof as pending Google Drive authorization, and fails malformed
  supplied import/readback evidence.
- This update moves bounded macOS GUI launch proof into
  `pnpm run verify:local:full` after desktop artifact smoke. On macOS the full
  baseline now runs `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke`
  before the WebDriver step, so release-grade local verification collects
  app-authored Tauri window, UI, and native workflow reports.
- The Tauri WebDriver harness now records macOS fallback proof from
  `.tmp/desktop-smoke/native-command-report.json` when official WebDriver is
  unavailable for WKWebView. The skip remains explicit, but the report now
  carries assertion counts plus HTML export, real-file, workspace restore, and
  project-local snapshot evidence from the bounded native launch smoke.
- This update promotes browser workflow execution into the ordinary local gates.
  `pnpm run verify:local` now runs `node scripts/check-e2e-environment.mjs`, and
  `pnpm run verify:local:full` now runs the full `node scripts/run-e2e.mjs` browser
  workflow suite after the production frontend build. The runner now defaults to the workspace-local Playwright cache under
  `.tmp/ms-playwright` before falling back to a system Chrome-compatible
  browser when that cache is missing.
- The browser environment preflight now records per-attempt output and retries
  transient Chromium launch failures, such as macOS headless Chrome closing
  before app assertions, while preserving immediate failures for real workflow
  assertion errors.
- This update deepens native primary-layout proof for the launched Tauri smoke.
  Mode evidence now records source/preview pane visibility and rendered sidebar
  or preview text, and the smoke validator requires concrete content proof for
  export-preview, review-governance, and presentation-outline modes rather than
  accepting only a workspace CSS class and sidebar route. The bounded launch
  window is now 60 seconds by default so the expanded end-to-end workflow can
  reach the final report instead of being cut off at an intermediate checkpoint.
- This update keeps external transform trust diagnostics synchronized with the
  effective trust state. A configured but untrusted executable path now shows
  the trust-cleared warning directly in the settings row, even if an older probe
  result was restored before the path field changed again.
- This update makes the already-wired direct HTML exporter unmistakable in the
  primary File toolbar by labeling the command `HTML Export` and preserving the
  existing Export sidebar, command-palette, native `File` -> `Export` ->
  `HTML Export`, rendered artifact, and sidecar manifest paths.
- This update broadens native workspace/tab proof in the launched Tauri smoke.
  The app-authored workflow now creates deterministic real Markdown files under
  `.tmp/desktop-smoke/native-workspace-*`, proves document-set grouping,
  pinning into the pinned group, assigning a loose note into `Native Board Pack`
  through the production drop handler, closing the document-set tab group,
  reopening a recently closed tab, and restoring active/pinned/editor-scroll/
  preview-scroll state. The smoke validator now requires structured
  `workspaceTabEvidence` in the final report.
- The smoke-only native menu emitter now queues the Tauri event asynchronously
  before returning. This keeps the native event route effect-based while
  avoiding a webview IPC promise stall that could leave a valid event delivery
  hidden behind a timeout.
- This update broadens native menu-command proof in the launched Tauri smoke.
  The desktop smoke now retries only transient no-report macOS launch failures,
  isolates the launch home/config/cache directories, records strict progress
  phases while the app-authored workflow is running, and requires final
  `status: passed` evidence for native View/Writing Tools menu routing. The
  launched workflow now proves export-preview, outline, and exports sidebar
  routing, CodeMirror search opening, TOC/equation/code-fence/table insertion,
  Templates panel opening, Docs Live section-draft generation and application
  for review, AI Paste modal opening, and guarded native `File` -> `Export` ->
  `HTML Export` execution in the Tauri webview.
- This update repairs the reveal-path file operation slice after the
  in-progress `filesystem.rs` change left the Rust crate uncompilable. The
  command builder now has focused regression coverage for existing paths,
  missing paths, platform-specific file-manager arguments, and path arguments
  containing spaces.
- This update extends desktop proof beyond artifact smoke. The desktop smoke
  script now also runs a native command workflow against real local files,
  covering save-as, open, watch, compile, export readiness, HTML/PDF/DOCX/PPTX/
  Markdown bundle export, sidecar manifests, and reveal command construction.
- This update adds app-authored native UI launch proof. During bounded desktop
  launch smoke, the Vue workbench sends a guarded Tauri IPC report that proves
  the native webview rendered primary commands, source/preview/sidebar/status
  surfaces, the active document identity, preview label, toolbar display, and
  viewport dimensions; the smoke records it in
  `.tmp/desktop-smoke/native-ui-report.json`.
- This update adds a representative rendered export audit for a board-style
  fixture, checking inspectable HTML, PDF object structure, DOCX/PPTX package
  anatomy, core/custom properties, comments and AI provenance appendices,
  transform artifacts, and Markdown bundle manifest evidence.
- This update makes the rendered export audit directly executable through
  `pnpm run test:rendered-exports`. The command generates local HTML, PDF,
  DOCX, PPTX, Markdown bundle, blog package, Substack package, LaTeX, and
  Google Docs package artifacts under `.tmp/rendered-export-audit`, plus a JSON
  report with sizes, hashes, a manual visual-review checklist, and
  `viewer-proof.json` with executable HTML/PDF/DOCX/PPTX/package assertions. On
  hosts with `pdflatex`, it also compiles the generated `.tex` artifact into a
  proof PDF under `.tmp/rendered-export-audit/latex-compile/`.
- This update also writes `.tmp/rendered-export-audit/manual-review.html`, a
  linked manual QA dashboard for the primary artifacts, review cases, checklist,
  hashes, and executable viewer/package proof.
- Completed rendered-export native-viewer sign-off now has to use the
  `neditor.rendered-export.visual-signoff.v1` schema, include reviewer and
  native-viewer metadata, use an ISO review timestamp, and preserve the exact
  byte counts plus SHA-256 hashes for every primary and review-case artifact in
  the current rendered export audit. Use
  `NEDITOR_RENDERED_EXPORT_SIGNOFF=/path/to/signoff.json pnpm run test:rendered-exports -- --validate-signoff-only`
  to validate a completed review against an existing audit bundle without
  regenerating artifacts. Stale sign-off files from older artifacts fail
  validation instead of closing the readiness gap.
- This update adds `.tmp/rendered-export-audit/automated-visual-review.json`.
  The report marks the current host `automated-reviewed` only when browser
  screenshots, PDF raster thumbnails, DOCX/PPTX Office preview extraction,
  Office preview screenshots, and mapped proof for every primary and review-case
  target are complete; human sign-off remains a separate optional reviewer
  contract.
- Release readiness now validates the rendered export audit structure instead
  of accepting the primary JSON report merely because it exists. The aggregator
  requires all primary export targets, the rich-blocks and option-heavy review
  cases, per-case HTML/PDF/DOCX/PPTX/Markdown-bundle artifacts, and the manual
  checklist before accepting the rendered export audit check.
- This update adds local-first blog and Substack publishing packages as export
  targets. `blog` and `substack` exports write ZIP packages containing
  compiled Markdown, standalone blog HTML, a minimal Substack copy/paste HTML
  fragment, plain text, metadata, RSS item seed, README, and the embedded
  NEditor manifest.
- This update adds LaTeX and Google Docs exports. `latex` writes a `.tex`
  document with metadata, headings, tables, figures, equations, links, and
  labels. `google-docs` writes a local ZIP handoff with `document.docx`,
  standalone HTML, Markdown, plain text, metadata, assets, README, and embedded
  manifest evidence for Google Docs import workflows.
- The current UI also exposes direct HTML export as a first-class action in the
  grouped File toolbar and Export sidebar. Browser workflow coverage verifies
  export readiness, success/failure diagnostics, output paths, manifest paths,
  and target handoffs; the native workflow smoke writes a real HTML export with
  sidecar manifest and output-hash proof from the launched Tauri webview.
- This update hardens HTML export as a standalone web document. Rendered HTML
  now carries a language attribute, viewport and generator metadata, status,
  version, source-hash, author/approval metadata, description fallback, and
  canonical links when present; focused Rust export tests and the rendered
  export audit cover the contract.
- This update deepens accessibility proof with a focused keyboard-only browser
  workflow. The workflow covers document-tab switching, command-palette result
  selection, diagnostics source jumps, preview document focus, table-editor
  cells/actions, stale-save conflict compare, conflict-modal Tab wrapping,
  merge-line add/reorder/remove controls, Escape close, and focus restoration
  through the system-Chrome Playwright fallback.
- This update fixes the native keep-local conflict path. Choosing keep-local for
  a root-file conflict now acknowledges the exact external hash that was
  reviewed, so the next save deliberately overwrites that known revision instead
  of being blocked as a stale save. The launched native workflow smoke now
  proves keep-local plus save, save-copy, merge, accept-external, and final file
  restoration against real files in the Tauri webview.
- This update also enables the existing Rust native file watcher feature for
  normal desktop builds. The launched workflow smoke now proves a clean external
  file write is delivered through `watchDriver: "native"`, reloads the open
  document without dirtying it, proves the native watcher tracks an included
  file and recompiles the master document after the include changes, and then
  restores the original source before continuing through the conflict/export
  workflow.
- This update strengthens the opt-in macOS desktop launch proof. The bounded
  launch smoke now writes `.tmp/desktop-smoke/launch-report.json` with the
  binary path, PID, observed launch window, captured output, and
  `processAlive: true` evidence when NEditor remains running until timeout.
- This update deepens the supported Windows/Linux Tauri WebDriver harness. In
  addition to native launch/title/shell checks, it now creates a dirty document
  and checks the native dirty-title marker, filters and inserts the built-in
  "Dose by weight" calc template through the Templates panel and verifies the
  inserted source reaches the desktop preview, saves and reopens a real Markdown
  file through the guarded dialog-free smoke path, renames, duplicates, and
  reveals deterministic Markdown files, runs export readiness through the desktop
  UI/command path, writes a real HTML export, validates the output artifact plus
  sidecar manifest hash, and verifies selected preferences persist across a
  desktop session restart before restoring them.
- This update makes macOS app bundle proof executable. `pnpm run verify:local:full`
  now builds `NEditor.app` on macOS and `pnpm run test:desktop-bundle` verifies
  the bundle Info.plist metadata, identifier, version, executable, icon, copyright,
  and high-resolution flag while writing
  `.tmp/desktop-bundle/macos-app-report.json`.
- This update makes the macOS DMG packaging limitation executable. `pnpm run
  test:desktop-dmg` creates a DMG when `hdiutil` works, or records the current
  sandboxed-host `hdiutil create` failure as
  `.tmp/desktop-bundle/macos-dmg-report.json` without masking unrelated
  packaging errors.
- This update adds host-independent package configuration proof. `pnpm run
  check:platform-packaging` verifies synchronized npm/Cargo/Tauri metadata,
  all-platform bundle targets, macOS/Windows/Linux icon coverage, production
  desktop window sizing, CSP guardrails, MIT license linkage, and the explicit
  `unsigned-local-builds` signing stance, then writes
  `.tmp/desktop-bundle/platform-package-config-report.json`.
- This update makes optional external-engine platform evidence inspectable.
  `pnpm run check:engines` now writes
  `.tmp/external-engines/probe-report.json` with platform, architecture,
  installed Graphviz/DOT variants, D2, PlantUML, missing optional engines, and
  SVG smoke artifacts under `.tmp/external-engines/artifacts/` for every
  installed engine. Installed engines now have to pass the adapter-shaped smoke
  render before the probe reports them compatible. The probe also writes
  external evidence templates under `.tmp/external-engines/templates/`, accepts
  validated copied `neditor.external-engine-evidence.v1` reports from
  `NEDITOR_EXTERNAL_ENGINE_EVIDENCE_DIR` or `.tmp/external-engines/external/`,
  fails malformed supplied evidence, and lets release readiness close missing
  optional engines only when accepted external proof exists.
- This update adds `pnpm run collect:engine-evidence`, which reuses the same
  bounded external-engine smoke probes and writes accepted
  `neditor.external-engine-evidence.v1` files under
  `.tmp/external-engines/external/` for every installed compatible engine. Use
  `NEDITOR_TEST_*=/absolute/path/to/engine pnpm run collect:engine-evidence`
  when a verifier host needs an explicit executable path such as a locally
  built Pikchr binary.
- This update adds native command workflow timing evidence to the desktop
  smoke. `pnpm run test:desktop-smoke` now writes
  `.tmp/desktop-smoke/native-command-report.json` with binary/build metadata,
  native command workflow status, and duration.
- This update deepens the bounded macOS desktop launch smoke. With
  `NEDITOR_DESKTOP_SMOKE_LAUNCH=1`, `pnpm run test:desktop-smoke` now also
  records System Events process/window evidence in
  `.tmp/desktop-smoke/launch-report.json` when macOS automation is available,
  including the native process name, window count, window name, and window size.
- This update makes desktop WebDriver evidence durable. `pnpm run
  test:tauri-webdriver` now writes `.tmp/desktop-webdriver/report.json` with
  the supported workflow plan plus dependency and assertion evidence on
  Windows/Linux, or the official macOS unsupported-platform reason plus the
  launch-smoke fallback.
- This update deepens rendered export handoff proof. `pnpm run
  test:rendered-exports` now asserts blog/Substack metadata and copy artifacts,
  LaTeX source structure, Google Docs import metadata, and the nested DOCX
  package inside the Google Docs handoff ZIP.
- This update expands macOS native DOCX extraction proof. The rendered export
  audit now runs `textutil` against the primary DOCX, the Google Docs handoff
  package's nested DOCX, and each rendered review-case DOCX, with all assertions
  recorded in `viewer-proof.json` and `manual-review.html`.
- This update adds macOS native rendered-viewer classification. The rendered
  export audit now attempts Quick Look thumbnails for the generated PDF, DOCX,
  and PPTX artifacts and records either thumbnail evidence, sandbox limits, or
  format-specific host limitations in
  `.tmp/rendered-export-audit/viewer-proof.json`.
- This update adds Chromium-rendered visual proof for export review. The
  rendered export audit now opens the primary HTML export, `manual-review.html`,
  and the rich-block/option-heavy HTML review cases in the resolved Playwright
  browser, captures screenshots under
  `.tmp/rendered-export-audit/browser-visual-proof/`, records dimensions,
  scroll metrics, expected evidence, and browser source in `viewer-proof.json`,
  and links the screenshots from the manual review dashboard.
- This update adds generated Office visual proof for DOCX/PPTX exports. The
  rendered export audit now derives reviewable HTML dashboards from the actual
  Office package XML for primary and review-case DOCX/PPTX artifacts, screenshots
  those dashboards through the resolved Chromium browser when available, records
  the evidence under `.tmp/rendered-export-audit/office-preview/`, and maps it
  into `viewer-proof.json`, `manual-review.html`, and `visual-review-summary.json`.
- This update makes browser workflow execution current-host evidence instead
  of stale archived evidence. `pnpm run test:e2e` now passes all 53 Chromium
  workbench workflows locally, including first-class outline mode CRUD,
  editor/preview typing, settings persistence, command palette navigation,
  collapsible toolbars, responsive desktop/narrow layout proof, fold/unfold
  controls, file/save/rename/reveal flows, snapshots, workspace restore,
  stale-save conflicts, include watchers, AI governance, and export
  readiness/success/failure, transform template management, deep keyboard-only
  workbench operation, plus browser UI handoff coverage for blog,
  Substack, LaTeX, and Google Docs package targets. The browser report now
  includes structured schema, test totals, output tails, and workflow flags so
  release readiness cannot accept a stale exit-status-only artifact.
- The checked-in browser harness now includes a Docs Live workflow in addition
  to the prior 52-test baseline. The focused
  `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "Docs Live" --project chromium`
  run passed on 2026-05-23, proving the modal opens from the Writing toolbar,
  accepts document type, outline, spoken direction, context, and placeholder
  values, generates provenance-bearing Markdown, applies it to the editor, and
  renders the review-preparation section in preview.
- This update fixes workflow bugs found by that browser run: command-palette
  editor commands now regain editor focus after the modal closes, CodeMirror
  enables multiple selections for multi-cursor commands, prepare-for-export and
  snapshot/restore actions flush pending editor text first, and source
  navigation from export/preview/presentation modes returns the workbench to a
  source-visible split view.
- `7aff68f` announced workbench status changes accessibly.
- `8d73d69` managed focus across workbench modals.
- `4ce66b5` improved keyboard access to workbench regions.
- This update labels compiler, readiness, and export diagnostics as
  screen-reader list items, labels conflict diff cells as side-specific groups,
  and extends the static accessibility guard plus browser harness assertions for
  those names.
- This update also names the table editor grid, row/column control groups, and
  totals so screen-reader users can navigate the dense table editing surface by
  structure rather than visual position alone.
- This update adds `pnpm run test:desktop-smoke`, a local desktop artifact smoke
  check for the Vite build, Tauri config, package/license metadata, and release
  desktop binary produced by `./node_modules/.bin/tauri build --no-bundle`. The
  same harness has an opt-in bounded GUI launch mode for hosts that permit
  desktop app startup.
- This update hardens high-contrast and reduced-motion accessibility by adding
  static CSS guardrails for black-on-white high-contrast colors, focus outlines,
  and zero-duration motion, plus browser harness assertions for the computed
  settings state.
- This update gives the primary source and preview surfaces deeper
  screen-reader semantics: CodeMirror content is now a named multiline textbox,
  and the rendered preview article is a focusable named document.
- This update blocks duplicate reference labels across headings, figures,
  tables, equations, appendices, and decisions. The compiler now emits
  source-ranged duplicate-label diagnostics with first-occurrence context, and
  export readiness copies those diagnostics into manifests so ambiguous cross
  references cannot leave the editor as audit-ready deliverables.
- This update also hardens export option readiness: malformed default citation
  styles, non-hex brand colors, malformed default brand profile fields,
  dirty-Git warning flags, and legacy cover/page-number aliases now fail
  readiness with manifest-backed diagnostics instead of being ignored.
- This update also makes appendix export options content-aware: enabled
  glossary, review-comment, or AI-provenance appendices now produce
  manifest-backed info diagnostics when the document has no matching content,
  while populated appendices remain clean.
- This update tightens the dependency-admission guard so NEditor stays MIT
  licensed in the root `LICENSE`, npm package metadata, Cargo crate metadata,
  and Tauri desktop bundle metadata.
- This update improves the native Pikchr fallback so compact
  semicolon-separated diagrams such as `box "A"; arrow "approve"; diamond
  "Gate"` render without requiring an external executable. The fallback now
  recognizes connector labels and common business shapes: box, circle/ellipse,
  diamond, cylinder, and file.
- Follow-up export evidence extends the visual transform conformance fixture so
  that the same native Pikchr artifact survives HTML, PDF, DOCX, PPTX, and
  Markdown bundle outputs.
- This update improves shared text and Markdown bundle table exports. Tables
  now keep readable Markdown-style header rows, alignment separator rows,
  escaped pipe cells, formula output rows, and merged-cell span annotations
  instead of collapsing to caption plus data rows.
- This update adds `media-uses.json` to Markdown bundle exports. Bundles now
  preserve per-figure audit evidence for each packaged media use, including
  figure IDs, captions, alt text, source files, source ranges, bundle paths,
  hashes, and per-use float/fit/position metadata while keeping shared media
  bytes deduplicated.
- `25bc28f` added titlebar release status visibility, Versioning-panel snapshot
  create/list/restore controls, and browser harness coverage for snapshot
  restore plus release tagging workflows.
- `929043c` constrained snapshot restores to active-document snapshot stores and
  matching metadata.
- `a57767b` surfaced malformed front matter diagnostics with source ranges.
- `2231776` added the first-user guide, Markdown extensions reference, README
  links, and automatic top-level docs link checking.
- `1c532ee` constrained Git restore/tag inputs to explicit safe refs, blocked
  symlink restore targets, and recorded the security evidence.
- `3c10a2c` documented storage and security boundaries, linked them from the
  README, and added markdown link checking for the new docs.
- `2b252c5` versioned workspace persistence migrations and covered legacy
  settings normalization in frontend unit tests.
- `c10da06` bounded large-document performance evidence with repeated
  compile/export memory-growth coverage.
- `ca94c90` proved preview debounce timing and coalescing behavior.
- `3214d1f` refreshed `docs/todo.md` from current evidence.
- `a93a974` recorded the spec completion matrix and durable progress log.
- `237f68c` logged the fresh verification baseline.
- `dee18fc` extracted shared workflow insertion helpers and added frontend unit
  coverage for AI paste insertion modes and conflict merge-line composition.
- `d94dc6c` added the first Playwright browser workflow harness, admitted the
  test dependency, wired the browser workflow CI lane, and refreshed this
  backlog/progress evidence.
- `6489162` installed pnpm before `actions/setup-node` enables `cache: pnpm` in
  CI.
- `420af08` kept the readiness browser workflow assertion strict by scoping the
  exact `Ready` match to the readiness panel.
- `9a6d52e` recorded the first passing Linux browser workflow CI lane in the
  durable docs.
- `5ce5b99` rewrote the active backlog around the latest survey and CI evidence.
- `25f7b04` Unix-gated the external transform test cache helper and attempted a
  `pikchr-cli` positional source compatibility path.
- `5c29914` normalized serialized file/export paths to `/` separators and
  changed the `pikchr-cli` adapter to pass a temporary `.pikchr` source file.
- `33ee6a9` made the fake `d2` adapter fixture drain stdin on Linux and
  refreshed the backlog, progress log, and completion matrix from current CI
  evidence.
- `443515b` added advanced table editor browser workflow coverage for Markdown
  paste import, numeric sorting, formula rows, merged-cell metadata, and
  apply-back behavior, with local unit/build proof and green CI in run
  `26134248308`.
- `e2f22d5` added a mocked Tauri file/dialog layer and browser workflow proof
  for open, edit, save, duplicate, rename, pin, reveal, workspace listing, and
  revert.
- `f5d5e9a`, `5b7a756`, `85ad6db`, and `613d880` tightened that workflow after
  CI exposed brittle selectors and edit-state assumptions.
- `11dafc3`, `a55970d`, `b7534f6`, and `12cd667` stabilized the mocked file
  lifecycle browser proof by checking saved mock contents, activating the files
  sidebar before workspace-row assertions, and pinning from the active document
  tab.
- `b2ccf83` refreshed the backlog, progress log, and completion matrix from
  the then-current survey before the final browser proof fixes.
- `61eff87`, `44c9639`, and `138bf5d` added and stabilized browser workflow
  proof for save-as to a new path and reopening that saved document from the
  recently closed list. The settings recent-path lists now have accessible
  section labels so repeated path buttons are unambiguous.
- `3cb1b84` and `25c7d1e` added and stabilized browser workflow proof for
  stale-save conflict recovery: save blocking when disk content changed, compare
  dialog visibility, local conflict-copy preservation, and merge-back recovery
  into the original file.
- `4eb1d2c` added browser workflow proof for the remaining stale-save conflict
  action buttons: keep-local preserves editor edits without overwriting the
  changed disk file, and accept-external replaces the active document with the
  external disk content.
- `e0ac31c` added browser workflow proof for watcher-originated root-file
  changes: clean documents reload external edits automatically and dirty
  documents open the non-destructive compare flow.
- `1ac72c1` preserved the clean reload status message after watcher-triggered
  compilation, fixing the CI regression from the first watcher workflow proof.
- This update adds table-draft cancellation, accessible table row/column
  controls, and browser workflow coverage for column formats, add/remove row,
  add/remove column, cancel-without-applying, and apply-back behavior.
- `15b7df6` kept fenced citation examples literal.
- `58ae0fd` shared table cell span normalization.
- `f157fbf` let the table editor author merged cells.
- `2c0253e` preserved imported table spans in semantic exports.
- `ae90d2c` carried merged table semantics through exports.
- `76fe169` forced optional diagram conformance in Linux CI.
- `51071fd` split tall PDF table cells through row fragments.
- `42f4298` covered PDF table continuation across page geometries.
- `45ff66e` split PDF paragraphs with widow and orphan control.
- `1f7c576` let PDF text flow beside figure floats.
- `d20cfde` shared fenced-code detection across compiler scanners.
- `a712da9` kept headings and references out of fenced examples.
- `3a3e009`, `5a13fe2`, and `7b549c0` restored workspace scroll positions,
  warned clearly about missing restored files, and recorded the green CI proof.
- `44c49f5`, `3fe0c78`, `39a3286`, and `bf60405` added and stabilized browser
  workflow proof for tab activation, dirty-close confirmation, renamed recent
  cleanup, and deleted recently-closed pruning.
- `13b3086` added browser workflow proof for reopening recent folders, pruning
  missing workspace roots from recent folders, and pruning externally moved
  recently closed document paths while the moved target remains visible through
  workspace refresh.
- `7702e89` added browser workflow proof for editor-to-preview scroll sync,
  preview-to-editor scroll sync, and rendered preview heading click-through to
  the CodeMirror source line.
- `f13c3f3` added browser workflow proof for persisted editor word-wrap and
  line-number settings, CodeMirror find/replace, smart list continuation,
  bracket auto-pairing, and command-palette heading search/navigation.
- `b881246` recorded the editor ergonomics workflow proof in the durable docs.
- `5f75e44` added browser workflow proof for command-palette citation,
  glossary, and index navigation, and made glossary palette entries jump to the
  matching source term while opening the References panel.
- `145942a` added browser workflow proof for command-palette open-document
  switching and workspace-file opening, including active tab, editor, and file
  sidebar state.
- `3b7a2fe` and `976016c` added and stabilized browser workflow proof for
  external transform engine settings: path-change trust clearing, trust
  prompts, denied trust reset, input mode and timeout persistence, successful
  probe details, cache identity, and missing-executable diagnostics.
- `03bda13`, `18ff4a4`, `e1e1b2a`, and `02e832a` added and stabilized export
  readiness/result workflow proof: target-specific readiness manifest preview,
  output and manifest path reporting, success and writer-failure diagnostics,
  blocked export diagnostics before file write, and an immediate editor-state
  flush before export readiness runs.
- This update adds `docs/ipc-command-coverage.md` and
  `ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented`,
  which parse `docs/specification.md` section 25.4, the coverage table, and the
  Tauri `generate_handler!` registration so required IPC commands cannot drift
  out of the app shell unnoticed.
- This update also records the project license as MIT in `LICENSE`,
  `package.json`, `src-tauri/Cargo.toml`, and the Tauri desktop bundle
  metadata, and removes GitHub Actions as an active verification surface.
- This update expands the example fixtures with explicit target-persona front
  matter and strengthens
  `example_fixture_tests::example_project_fixtures_compile_and_export` so the
  target-user requirement is backed by executable compile/export proof.
- This update strengthens the Playwright AI paste workflow harness so
  citation TODO insertion, draft review markers, and `ai-source` provenance
  blocks are covered through the preview and insertion UI path.
- This update makes multi-cursor editing explicit through command-palette
  actions for add-cursor-above, add-cursor-below, and select-next-occurrence,
  plus a Playwright workflow harness case for editing two lines through
  multiple cursors.
- This update adds command-palette descriptions and intent-search aliases for
  multi-cursor editing, so a non-technical query such as "multi cursor" exposes
  add-cursor-above, add-cursor-below, and select-next-occurrence before the
  browser workflow edits two lines through multiple cursors.
- This update adds first-class Glossary and Index manager summaries plus
  insertable audit tables, letting reviewers verify detected terms,
  generated-section markers, export inclusion, and index exclusions from the
  References panel.
- This update extends the editor ergonomics workflow harness with an exact
  status-bar assertion for word count, character count, and reading-time
  metrics.
- This update adds outline-sidebar navigation coverage, proving the sidebar
  can focus a deep heading back in the Markdown source outside the command
  palette path.
- This update adds editor workflow harness assertions for the CodeMirror
  `spellcheck` and `autocapitalize` content attributes.
- This update adds explicit Markdown format commands for bold, italic, inline
  code, and code fences, with workflow harness coverage for quote pairing,
  emphasis pairing, inline-code pairing, and fence insertion.
- This update adds diagnostics-panel workflow coverage for a compiler
  diagnostic with line/column metadata and source navigation back into the
  editor.
- This update wires export, review, and presentation modes to their expected
  sidebars and hides the source pane for presentation outline mode, with
  workflow harness coverage for split, source, preview, focus, export, review,
  and presentation modes.
- This update adds workflow harness coverage for browser window title updates
  across clean opened files, dirty edits, and post-save clean state.
- This update adds workflow harness coverage for app theme, preview theme, high
  contrast, reduced motion, editor typography, preview typography, and reload
  persistence.
- This update adds workflow harness coverage for live preview updates after
  typing new source content.
- This update adds workflow harness coverage for tab grouping by folder and
  front matter document set, dragging a loose tab into an existing document
  set, saving the generated `documentSet` front matter, and closing that group
  without disturbing other open groups.
- This update adds generated `[LIST_OF_FIGURES]` and `[LIST_OF_TABLES]`
  support, including front matter aliases, fence-aware caption scanning,
  numbered anchor links, and Rust compiler/export proof for compiled Markdown,
  preview HTML, and DOCX artifact text.
- This update adds command-palette insertion paths for all generated-section
  markers: `[TOC]`, `[INDEX]`, `[BIBLIOGRAPHY]`, `[LIST_OF_FIGURES]`, and
  `[LIST_OF_TABLES]`, with workflow harness coverage in the snippet insertion
  flow.
- This update strengthens the references workflow harness with mock BibTeX
  parsing so the References panel proves resolved bibliography entries,
  missing citation keys, and duplicate bibliography-key reporting.
- This update extends export readiness validation so malformed review comments
  and change notes missing author, timestamp, or text emit actionable
  diagnostics, mark readiness as not ready, and copy those diagnostics into the
  export manifest for auditability.
- This update extends AI provenance readiness validation so incomplete
  `ai-source` blocks, incomplete AI-assisted section markers, missing
  human-review metadata, and invalid AI review statuses produce actionable
  diagnostics that are also copied into the export manifest.
- This update extends export readiness validation so figures, tables, and
  equations missing stable labels or captions produce actionable diagnostics
  that are copied into the export manifest before artifact writes.
- This update adds an explicit export manifest readiness summary with ready,
  error, warning, and info counts so sidecar manifests carry portable
  preflight evidence without requiring consumers to recount diagnostics.
- This update makes direct exports run the same dirty-Git readiness check as
  preflight export reports, records that warning in the response and sidecar
  manifest, and avoids falling back to the repo cwd when an unsaved/nonexistent
  source path is exported.
- This update restores the `docs/specification.md` architecture figure target
  with `docs/architecture.svg` and adds `pnpm run check:docs` as a repeatable
  repo-wide local markdown link guard.
- This update adds `pnpm run check:a11y` as a static accessibility guard for
  the Vue workbench template and fixes exposed labeling gaps in modal close
  buttons, command-palette search, and conflict merge-line controls.
- This update adds visible-on-focus skip links for keyboard users, makes the
  command bar, workspace, sidebar, Markdown source, live preview, and status
  bar programmatically focusable skip targets, extends `pnpm run check:a11y` to
  guard those targets, and adds a Playwright workflow for skip-link focus.
- This update adds modal focus management for the AI paste, command palette,
  and external-conflict dialogs: initial focus, edge Tab trapping, Escape close,
  and focus return to the invoking control. The static accessibility guard now
  requires dialog focusability and keyboard handling, and the browser harness
  lists a modal focus/Escape workflow.
- This update makes status-bar messages, file-watch state, compile progress,
  export progress, and errors screen-reader visible with labeled live regions.
  The static accessibility guard now requires those live regions, and the
  browser harness lists a status/progress live-region workflow.
- This update adds
  `performance_tests::compiler_stress_handles_large_documents_with_many_artifacts`,
  which stress-compiles a large Markdown source with nested includes, many
  tables, formulas, transform artifacts, source-map entries, and diagnostics.
- This update adds
  `performance_tests::repeated_export_loop_keeps_large_artifacts_stable`,
  which repeatedly renders a large compiled document through HTML, PDF, DOCX,
  PPTX, and Markdown bundle outputs with structural and artifact-size
  assertions.
- This update adds
  `performance_tests::repeated_editing_sessions_reuse_external_transform_cache`,
  which repeatedly recompiles edited document text while proving a stable
  trusted external DOT transform is served from cache after the first run.
- This update adds structured export progress steps to readiness/export
  manifests and the export UI. The new
  `export_command_tests::export_readiness_and_manifest_report_progress_steps`
  test proves compile, transform, readiness, render, and manifest progress
  stages are carried in backend evidence.
- This update adds latest-document task guards for preview compilation, a
  status-bar cancel action while compilation is pending, and frontend unit
  coverage proving stale or cancelled compile results are rejected.
- This update adds browser workflow proof for compile cancellation. The
  Playwright harness can delay mocked preview compilation, click the status-bar
  Cancel compile action while work is pending, assert the cancellation status,
  then resume editing and confirm the live preview updates again.
- This update adds browser preview proof for generated tables of contents. The
  browser harness now renders a generated `[TOC]`, checks nested heading links,
  and clicks a TOC link back to the source heading.
- This update extracts preview text commit debounce into a tested helper with
  an explicit `PREVIEW_DEBOUNCE_MS` budget and frontend unit coverage for
  rapid-edit coalescing plus immediate flush behavior.
- This update adds
  `performance_tests::repeated_compile_export_cycles_keep_memory_growth_bounded`,
  which repeatedly compiles and exports large documents while retaining only
  summary data and bounding process RSS growth on macOS/Linux hosts.
- This update adds a Playwright large-document interaction workflow that edits
  a 120-section document, waits for preview update, checks elapsed browser
  time, and verifies editor-to-preview scroll sync. This host still needs the
  Playwright Chromium binary before that browser workflow can execute locally.
- This update extracts workspace persistence migration into
  `src/lib/workspacePersistence.ts` with an explicit schema version, normalized
  settings, legacy key aliases, clamped persisted values, and frontend unit
  coverage for preferences, recents, transform settings, AI cleanup defaults,
  and scroll positions.
- This update adds `docs/storage-model.md` and
  `docs/security-threat-model.md`, links them from the README, and folds them
  into the markdown link checker so local-first storage, sidecar evidence,
  trust boundaries, threat mitigations, and non-goals are auditable alongside
  the specification matrix.
- This update hardens the Git versioning command surface: release tags and
  restore revisions now reject option-shaped or unsupported ref syntax before
  invoking Git, restore refuses symlink targets, and restore paths must resolve
  inside the repository root before the document is overwritten.
- This update adds first-user documentation in `docs/user-guide.md`, a
  readable syntax reference in `docs/markdown-extensions.md`, README links to
  both, and a docs link checker that automatically discovers top-level docs
  Markdown files.
- This update extends export conformance proof for release-grade business
  metadata: the fixture now carries a legal disclaimer, exported DOCX/PPTX
  packages carry approval and legal-disclaimer custom properties, and
  `export_conformance_fixture_maps_business_features` asserts legal
  disclaimer plus approval metadata across HTML, PDF, DOCX, PPTX, plain text,
  and Markdown bundle outputs.
- This update adds
  `prepare_for_export_carries_broad_readiness_audit_to_manifest`, which proves
  one export readiness report and its manifest carry diagnostics for metadata,
  approval, layout, citation source gaps, comments, change notes, AI
  provenance, AI review status, broken links/media, inline and table formulas,
  caption/label checks, transform settings, and pending progress state.
- This update carries the compiler include graph into export manifests and
  Markdown bundles, adding `include_graph` manifest evidence plus an
  `include-graph.json` bundle entry so exported audit packs preserve parent,
  child, and depth relationships instead of only flat included-file hashes.
- This update expands the Rust-native OpenAPI and JSON Schema transforms from
  minimal tables into richer reference renderers: OpenAPI output now includes
  API title/version, servers, operation IDs, tags, path and operation
  parameters, request bodies, responses, media types, and component schemas;
  JSON Schema output now includes root metadata, nested field paths, array item
  paths, required flags, descriptions, refs, enums, formats, defaults, and
  numeric/string constraints.
- This update adds
  `export_option_matrix_is_preserved_across_targets_and_bundle_evidence`,
  which runs one explicit export option matrix through HTML, PDF, DOCX, PPTX,
  plain text, and Markdown bundle evidence. It proves styles, syntax
  highlighting, cover pages, page numbers, glossary/comment/provenance
  appendices, PPTX agenda generation, layout preset, watermark, legal
  disclaimer output, and Markdown bundle `export_options` are honored from the
  same compiled document.
- This update adds
  `rich_markdown_blocks_survive_cross_target_exports`, which pushes a richer
  Markdown business fixture through HTML, PDF, DOCX, PPTX, plain text, and
  Markdown bundle evidence. It covers block quotes, callouts, unordered,
  nested, ordered, and task lists, JavaScript code blocks, tables, figures,
  equations, generated lists of figures/tables, and cross references.
- This update adds
  `external_transform_cache_invalidates_when_trusted_executable_changes`, which
  rewrites a trusted external executable at the same absolute path and proves
  the cache key changes, the output hash changes, and the new engine output is
  executed instead of serving stale memory or disk cache. The transform
  metadata and setup docs now describe engine file size, modified time,
  adapter arguments, input mode, renderer version, and source hash as part of
  cache identity.
- This update adds `pnpm run check:engines` and
  `docs/external-transform-platform-evidence.md`. The current macOS Darwin
  arm64 evidence verifies Graphviz/DOT, D2, and PlantUML through the optional
  engine probe and `external_transform_conformance_runs_installed_engines`
  with `--nocapture`; Pikchr is explicitly recorded as missing on this host.
- This update adds per-key missing-citation readiness diagnostics with precise
  source ranges when citations are present but no bibliography source is
  available. The new readiness proof verifies line, column, end-line, and
  end-column details for each missing key and confirms those diagnostics are
  copied into export manifests.
- This update carries source locations through bibliography entries, points
  duplicate-key readiness diagnostics at the duplicate entry with the first
  occurrence in related context, and shows duplicate bibliography source
  locations in the References panel.

## Latest Focused Verification

| Command | Result | Notes |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked representative_rendered_export_artifacts_are_package_inspectable --lib` | Pass | Focused Rust audit generation test writes the rendered export audit with the new equation review case and asserts inline/styled math, matrix rendering, extended notation, piecewise rendering, captions, and cross-target equation evidence. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit now verifies `review-cases/equations` across HTML/PDF/DOCX/PPTX/Markdown-bundle artifacts, browser visual proof, Office preview dashboards, Poppler proof where available, and manual-review dashboard links. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked compiler_accepts_text_edited_tables_without_outer_pipes --lib` | Pass | Focused Rust compiler proof accepts a directly typed Markdown table without outer pipes, evaluates formulas, resolves table references, preserves escaped literal pipes, and renders the table into HTML/AST output. |
| `pnpm run test:unit` | Pass | 80 frontend unit/static tests passed, including table editor parsing for directly typed Markdown tables without outer pipes and canonical source regeneration for the visual grid. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked representative_rendered_export_artifacts_are_package_inspectable --lib` | Pass | Focused Rust audit generation test writes the rendered export audit with the new business-transform review case and asserts roadmap, ADR, diff, QR, embedded artifact hashes, and cross-target ADR/diff text. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit now verifies `review-cases/business-transforms` across HTML/PDF/DOCX/PPTX/Markdown-bundle artifacts, browser visual proof, Office preview dashboards, Poppler proof where available, and manual-review dashboard links. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked representative_rendered_export_artifacts_are_package_inspectable --lib` | Pass | Focused Rust audit generation test writes the rendered export audit with the new brand/layout review case and asserts logo, brand color, page layout CSS, header/footer templates, watermark metadata, and DOCX page-number fields. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit now verifies `review-cases/brand-layout` across HTML/PDF/DOCX/PPTX/Markdown-bundle artifacts, browser visual proof, Office preview dashboards, Poppler proof where available, manual-review dashboard links, and release-readiness-required review-case coverage. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked representative_rendered_export_artifacts_are_package_inspectable --lib` | Pass | Focused Rust audit generation test writes the rendered export audit with the new generated TOC/page-number review case. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit now verifies `review-cases/toc-page-numbers` across HTML/PDF/DOCX/PPTX/Markdown-bundle artifacts, including numbered generated TOC links, PDF page-number leaders, native DOCX TOC-field guidance, browser visual proof, Office preview dashboards, Poppler proof where available, and manual-review/sign-off metadata. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting remains clean after the OpenAPI server-rendering update. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked compiler_renders_openapi_and_json_schema_tables --lib` | Pass | Focused transform test proves OpenAPI server variables plus operation/path server overrides render in generated API docs. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked api_schema_transforms_survive_cross_target_exports --lib` | Pass | Cross-target API/schema export conformance remains intact. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked representative_rendered_export_artifacts_are_package_inspectable --lib` | Pass | Focused Rust audit generation test writes the rendered export audit with the new edited-table review case. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit now verifies `review-cases/edited-tables` across HTML/PDF/DOCX/PPTX/Markdown-bundle artifacts, browser visual proof, Office preview dashboards, Poppler proof where available, and manual-review/sign-off metadata. |
| `pnpm run check:tables:manual` | Pass | Table editor manual-review contract now writes the reviewer template and pending summary, validates prerequisite report identity, and provides a strict `NEDITOR_TABLE_EDITOR_SIGNOFF` path for source/grid/spreadsheet/export/supported-host sign-off. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "opens, saves|saves a document|switches tabs|edits table structure" --project chromium` | Pass | Focused rerun proves Save/Open/Rename/Pin/Recently Closed tab identity and the table structure cancel/apply workflow after adding file-name subtitles and the explicit **Apply table** action. |
| `pnpm run test:e2e` | Pass | Full Chromium browser workflow suite refreshed the authoritative `.tmp/e2e-browser/report.json` with 72 passed workflows and zero failures. |
| `pnpm run check:platform-evidence` | Pass with pending external evidence | Stale Windows/Linux external platform evidence is now classified as stale/pending instead of hard-invalid after a new source commit. |
| `pnpm run check:ai-provider` | Pass with pending live evidence | Prior-commit provider evidence is stale/pending while malformed current evidence remains invalid. |
| `pnpm run check:security-review` | Pass with pending independent review | Prior-commit security review evidence is stale/pending while malformed current evidence remains invalid. |
| `pnpm run check:performance-profile` | Pass with pending release-device profile | Prior-commit performance profile evidence is stale/pending while malformed current evidence remains invalid. |
| `pnpm run check:ai-runtime` | Pass with pending real-device evidence | AI runtime evidence template/report refreshed for the current commit. |
| `pnpm run check:google-docs-import` | Pass with pending Google Drive authorization | Prior-commit Google Docs import evidence is stale/pending while local rendered package proof remains current. |
| `pnpm run check:release-ci` | Pass | Release CI workflow report refreshed after package script changes. |
| `pnpm run verify:local -- --list` | Pass | Quick local verification list now includes the table editor manual-review contract so the release baseline keeps the new table QA gate visible. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "recomputes watched include paths" --project chromium` | Pass | Focused browser workflow proves editing include directives recomputes the include graph and watched paths, removes the old include, watches the new include, ignores stale old-include watch events, and recompiles after the new include changes. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "moves clean watcher roots" --project chromium` | Pass | Focused browser workflow proves Save As path changes resync the clean root watcher from the old path to the new path, ignore stale old-root events, and reload clean external changes from the moved path. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "moves watcher roots after closing" --project chromium` | Pass | Focused browser workflow proves closing the active watched tab moves the watcher root to the newly active tab, ignores stale closed-tab events, and reloads clean external changes from the active file. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "gates Supertonic TTS model downloads" --project chromium` | Pass | Focused browser workflow proves Settings exposes Supertonic model size, editable storage path, download source, and command; model download and read-aloud are disabled until acknowledgement; acknowledged setup can copy details, queue the model download, read the document with Supertonic, and inspect runtime status. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused browser workflow proves two-way table editing: visual creation, direct in-text Markdown table edits syncing into the grid, invalid in-text table protection, editable source text export, source-to-grid parsing, applying source text back into the document, and the explicit **Edit Markdown in text** action selecting a source table that can be replaced in the editor and synced back into visual cells. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "keeps production Docs Live wizards structure-first" --project chromium` | Pass | Focused browser workflow proves podcast and movie-script Docs Live modes block prose while production structure is only suggested, then unlock sequential segment or beat drafting after the approved architecture is supplied. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "keeps long-form Docs Live wizards outline-first" --project chromium` | Pass | Focused browser workflow proves Docs Live textbook and novel modes block prose until the architecture/plot is approved, then switch to sequential chapter drafting after an approved outline/plot is supplied. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording Supertonic model-download browser proof, direct table-text editing, and production-script Docs Live evidence. |
| `git diff --check` | Pass | No whitespace errors are present. |

## Current Capability Snapshot

Implemented or substantially present, pending the conservative caveats in
`docs/spec-completion-matrix.md`:

- Tauri 2 desktop app scaffold with Vue 3, Pinia, Vite, CodeMirror 6, vanilla
  CSS, and Rust IPC commands.
- Markdown workbench UI with editor, live preview, sidebars, tabs, status bar,
  command palette, review/versioning/export/settings panels, and conflict UI.
- File operations for local documents, workspace folder browsing, recent files,
  recently closed documents, pinned tabs, workspace restore, snapshots,
  validated Git history/diff/commit/tag/restore, and guarded saves.
- Compiler pipeline for front matter, includes, variables, transforms,
  formulas, citations, bibliography, glossary, index, cross references, review
  comments, AI provenance, generated lists of figures/tables, semantic AST,
  paged document model, diagnostics, source maps, and export manifests.
- Export modules for HTML, PDF, DOCX, PPTX, and Markdown bundle outputs.
- Export package metadata now records status, version, classification, client,
  approval metadata, legal disclaimer, source hash, and app version where the
  target format supports custom properties.
- Export manifests now include included-file hashes, media hashes, source maps,
  diagnostics, readiness summaries, progress steps, layout sections, transform
  metadata, and the include graph used to build the deliverable.
- Export option fidelity now has backend package/text proof across HTML, PDF,
  DOCX, PPTX, plain text, and Markdown bundle manifests for one target-specific
  option matrix.
- Rich Markdown block fidelity now has backend package/text proof across HTML,
  PDF, DOCX, PPTX, plain text, and Markdown bundle AST/text evidence for common
  business-document blocks and generated cross-reference sections.
- Transform registry with Rust-native renderers/fallbacks and trust-gated
  external adapters for Graphviz/DOT, D2, PlantUML, and Pikchr.
- External transform cache identity includes executable file identity
  (path/size/modified time), adapter arguments, input mode, renderer version,
  and source hash, with a regression test proving same-path executable rewrites
  do not serve stale cached output.
- Optional external engine platform evidence can now be refreshed locally with
  `pnpm run check:engines`; the latest macOS record proves Graphviz variants,
  D2, and PlantUML version plus SVG smoke-render paths and records the missing
  Pikchr gap.
- Rust-native structured-document transforms for JSON, YAML, OpenAPI, and JSON
  Schema, including nested schema/reference output for API and schema docs.
- Business features including AI paste cleanup, table editor logic, formula
  handling, diagrams, citations, captions, layout directives, export readiness,
  and review/release metadata.
- Backend test coverage across many compiler, export, transform, table,
  validation, media, file, Git, snapshot, review, and provenance paths.
- Executable coverage for the initial Tauri IPC command list in specification
  section 25.4, backed by the human-readable
  `docs/ipc-command-coverage.md` table.
- MIT license metadata in root, npm, Cargo, and Tauri desktop bundle package
  surfaces.
- Realistic example fixtures now cover consultants, technical writers,
  researchers and analysts, product and engineering teams, executives and
  managers, students and academics, developers, and teams using AI chat output,
  with local Rust coverage across HTML, PDF, DOCX, PPTX, and Markdown bundle
  outputs.
- User-facing docs now include the README feature tour, a first-user guide, a
  Markdown extensions syntax reference, transform setup, storage model,
  security threat model, completion matrix, backlog, and progress log.
- Frontend unit coverage for table logic and conflict diff alignment.
- Playwright browser workflow harness for Vite with mocked Tauri IPC,
  covering split/source/preview/focus/export/review/presentation view mode
  switching, clean/dirty/saved window titles, theme/typography persistence,
  command palette table insertion, table editor insertion, mocked file
  lifecycle operations, advanced table
  paste/sort/formula/merge/apply behavior, row/column structure editing,
  column format totals, cancel behavior, save-as plus recently closed
  reopening, stale-save conflict copy/merge/keep-local/accept-external
  recovery, watcher-originated root reload/conflict behavior, AI paste cleanup
  insertion plus quote/appendix/replace-document/section/selection modes,
  citation TODO insertion, draft markers, provenance blocks,
  clean included-file recompile, dirty included-file conflict handling, and
  export readiness, restart-style workspace restore, folder/document-set tab
  grouping, drag-to-document-set front matter updates, close-group behavior,
  restored scroll position handling, missing restored-file warnings, tab activation, dirty-close
  confirmation, renamed recent cleanup, deleted recently-closed pruning, recent
  folder reopen/prune behavior, moved recently-closed path pruning,
  live preview updates from source edits, synchronized editor/preview scrolling,
  preview heading click-to-source, persisted editor word-wrap and line-number
  settings, spellcheck/autocapitalize editor attributes, CodeMirror find/replace, status-bar
  word/character/reading-time metrics, explicit multi-cursor command coverage,
  smart list continuation, bracket/quote/emphasis/inline-code pairing,
  code-fence insertion, command-palette heading navigation, command-palette
  citation, glossary, and index navigation, outline-sidebar heading navigation,
  diagnostics-panel source navigation, command-palette open-document switching
  and workspace-file opening, plus
  transform engine settings trust/probe diagnostics, target-specific export
  readiness manifest preview, export output/manifest path reporting, export
  success/failure diagnostics, and blocked-readiness diagnostics before file
  write.
- Local verification scripts for Rust formatting/check/test, native-watch
  compilation, clippy, frontend unit tests, frontend build, Playwright browser
  workflows, and Tauri no-bundle desktop compilation.

## Active Known Gaps

P0 gaps:

- GitHub Actions is not an active verification surface for NEditor. The
  `.github/workflows/ci.yml` workflow has been removed, and completion gates now
  rely on local verification evidence plus explicit rendered/manual/platform
  proof where required.

- Archived remote workflow evidence for commit `02e832a` was green: browser workflow, Ubuntu
  desktop, macOS desktop, and Windows desktop all passed in run `26159396761`.
  The prior Windows path-sensitive Rust-test failures, Ubuntu installed Pikchr
  conformance failure, and Ubuntu fake-`d2` stdin fixture failure are resolved
  in that retired workflow.
- Browser-level workflow tests now pass locally with 53 Chromium tests through
  `pnpm run test:e2e`. The current macOS proof used the workspace-local
  Playwright Chromium cache at `.tmp/ms-playwright`, and
  `.tmp/e2e-browser/report.json` records `source: playwright-bundled`, the
  cache executable path, schema `neditor.e2e-browser-workflow.v1`, 53 tests
  run, 53 passed, zero failures/timeouts, and explicit workflow evidence for
  Docs Live, editable outline planning, outline mode CRUD, and export
  workflows. This closes the prior local browser launch blocker and covers
  mocked file
  lifecycle, save-as/recently-closed flows, stale-save conflict copy/merge/
  keep-local/accept-external recovery, clean watcher reload, watcher-originated
  dirty root-file conflicts, included-file recompilation/conflicts, restart
  workspace restore, tab activation, recent cleanup, synchronized editor/
  preview scrolling, preview heading click-to-source, settings persistence,
  CodeMirror find/replace, fold/unfold controls, editing helpers,
  command-palette navigation, generated TOC preview/source navigation,
  transform engine diagnostics, pending preview compile cancellation/resume,
  responsive desktop/narrow layout proof, transform template management, table workflows, AI governance, export
  readiness/success/failure diagnostics, and blog/Substack/LaTeX/Google Docs
  export-target UI handoffs.
- Desktop proof now includes the native command workflow smoke plus a bounded
  macOS GUI launch smoke. A Tauri-driver/WebDriver harness also exists and is
  wired into full local verification; on this host it records the official
  macOS skip. The supported-platform harness covers dirty native titles, export
  readiness, and preference persistence across restart, but supported
  Windows/Linux execution is still needed.
- Current progress/matrix/docs need to be kept updated as evidence changes.

P1 gaps:

- Export fidelity now has a local rendered-export audit artifact bundle,
  generated rich-block and option-heavy review cases, and a manual checklist.
  Live Google Docs import, cross-platform native viewer checks, and human visual
  review outside package/text inspection remain before the entire export surface
  can be complete.
- Export readiness has browser workflow coverage for the target-specific
  status/diagnostic path, but still needs a requirement-by-requirement audit.
- Optional external transform evidence now includes current macOS Graphviz/DOT,
  D2, and PlantUML proof. macOS Pikchr and all Windows optional-engine evidence
  remain incomplete.
- File watcher/conflict flows now have browser workflow proof; native desktop
  execution proof remains missing.
- Workspace/tab-group behavior now has browser harness proof for restart
  restore and document-set grouping; native desktop proof and deeper drag/reorder
  edge cases remain.
- Remaining editor and preview ergonomics need native desktop proof and broader
  visual QA beyond the current browser interaction coverage.
- AI paste, citations, layout, accessibility, performance, table export
  fixtures, and non-sandboxed table/browser execution need workflow, artifact,
  or benchmark evidence before they can be considered complete.

P2/P3 gaps:

- `src/App.vue` and `src/stores/documents.ts` remain large and should be split
  after workflow behavior is locked.
- Large backend/export/transform modules remain and should be split when
  boundaries are clear.
- User documentation, example project fixtures, and full cross-platform
  packaging evidence remain incomplete.

## Verification Status

Current verification recorded on 2026-05-21 through 2026-05-27:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed with 22 focused tests and rebuilt `src-tauri/target/debug/ned`; coverage now includes `ned rfp-response` JSON/schema output, response and compliance-matrix file writing, stdin `ned analyze-rfp - --matrix`, shell completions, and help text. |
| `src-tauri/target/debug/ned analyze-rfp - --matrix` with piped RFP text | Pass | Direct packaged-binary smoke read stdin and printed a compliance matrix with pricing and implementation timeline requirements mapped to response sections and evidence prompts. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting stayed clean after adding the RFP CLI parser, analyzer, response renderer, and tests. |
| `pnpm run test:unit` | Pass | 76 frontend unit/static tests passed after adding Settings guidance and static guards for `ned rfp-response`, `ned analyze-rfp`, schema `neditor.ned-rfp-response.v1`, and the command dispatcher. |
| `pnpm run check` | Pass | Vue typecheck passed after adding RFP CLI examples to the Configuration Center copy. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting headless RFP response/matrix generation. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording headless RFP CLI setup guidance. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration contract still accepts the packaged `ned` sidecar after adding the new RFP CLI commands. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract still passes after the `ned` command surface was extended. |
| `git diff --check` | Pass | No whitespace errors are present after the headless RFP CLI update. |
| `pnpm run test:unit` | Pass | 76 frontend unit/static tests passed after adding the editable table Markdown source block, including static guards for source-to-grid parsing, grid-to-source refresh, source-text apply, and source-edit dirty tracking. |
| `pnpm run check` | Pass | Vue typecheck passed after wiring the editable table source textarea and its parse/apply controls into the Tables sidebar. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting explicit source-text table editing. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording editable source block evidence for table editing. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "shows delegated button help on hover and focus"` | Pass | Focused Chromium workflow proved delegated button help appears on pointer hover, follows keyboard focus from Commands to Help, hides when focus leaves button controls, and includes disabled-state guidance for the table Export CSV action. |
| `pnpm run check:a11y:runtime` | Pass | Runtime accessibility audit now runs seven focused Chromium workflows and includes delegated button-help proof for hover, keyboard focus handoff, tooltip hiding, and disabled-button guidance. |
| `pnpm run test:unit` | Pass | 76 frontend unit/static tests passed after updating the runtime accessibility workflow guard to require delegated button-help coverage. |
| `pnpm run check` | Pass | Vue typecheck passed after adding the button-help workflow and runtime audit entry. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after updating progress and the spec matrix. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording the accessibility evidence update. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed 66 tests after the focused accessibility run refreshed the browser evidence report; the suite now includes delegated button-help behavior. |
| `pnpm run check:release-readiness` | Pass with external gaps | Wrote `.tmp/release-readiness/report.json` with status `current-host-ready-with-external-gaps` after refreshing full browser workflow evidence. |
| `git diff --check` | Pass | No whitespace errors are present after adding delegated button-help runtime proof. |
| `pnpm run test:unit` | Pass | 76 frontend unit/static tests passed after extracting table source-sync helpers into `src/lib/tables.ts`, including direct source snapshot, source extraction, draft Markdown serialization, changed-source detection, deleted-table detection, other-document suppression, and new-draft suppression coverage. |
| `pnpm run check` | Pass | Vue typecheck passed after moving table source synchronization helpers out of the workbench component. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after updating progress and the spec matrix for the helper extraction. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording frontend architecture and table editor evidence updates. |
| `git diff --check` | Pass | No whitespace errors are present after extracting table source synchronization helpers. |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding source-sync protection for the table editor, including static guards for the source-change warning, reload action, explicit overwrite action, and synchronization status. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows"` | Pass | Focused Chromium workflow proved a visual table can be inserted, then a direct Markdown source edit from `Revenue` to `Pipeline` syncs back into the visual table grid cell without requiring a new table. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed 65 tests after the focused table run refreshed the full-suite browser evidence required by release readiness; the table workflow includes source-to-grid synchronization. |
| `pnpm run check` | Pass | Vue typecheck passed after adding table source snapshots, clean-draft auto-sync, dirty-draft overwrite protection, and the source synchronization UI. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting table source synchronization. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording source-synced two-way table editing. |
| `pnpm run check:release-readiness` | Pass with external gaps | Wrote `.tmp/release-readiness/report.json` with status `current-host-ready-with-external-gaps` after refreshing full browser workflow evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the table source synchronization diff. |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding cursor-aware two-way Markdown table editing, including direct proof for table line-range matching and static wiring for source-table load/jump/apply controls. |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding AI transform template assistance, including direct proof that ROI/payback context generates four template-workflow suggestions and static wiring for the Templates sidebar notes workflow. |
| `pnpm run check` | Pass | Vue typecheck passed after adding two-way source table editing and AI transform template assistance. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting two-way table editing and AI transform template assistance. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording the table and transform workflow updates. |
| `pnpm run check:release-readiness` | Pass with external gaps | Wrote `.tmp/release-readiness/report.json` with status `current-host-ready-with-external-gaps`; external release evidence gaps remain unchanged. |
| `git diff --check` | Pass | No whitespace errors are present in the table editing and transform assistance diff. |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding AI quality review assistance, including direct proof that quality findings produce four context-aware suggestions and static wiring for the Review sidebar notes workflow. |
| `pnpm run check` | Pass | Vue typecheck passed after wiring quality step assistance into the Review sidebar and editable quality review notes. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting AI quality assistance in the README, user guide, progress log, and spec completion matrix. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording the AI quality assistance coverage. |
| `pnpm run check:release-readiness` | Pass with external gaps | Wrote `.tmp/release-readiness/report.json` with status `current-host-ready-with-external-gaps`; external release evidence gaps remain unchanged. |
| `git diff --check` | Pass | No whitespace errors are present in the AI quality assistance diff. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting stayed clean after adding the `ned profile` command surface and profile-field normalizer. |
| `pnpm run check:cli` | Pass | 21 focused CLI tests passed and the `ned` binary rebuilt. Coverage includes `.neditor/business-profile.json` scaffold creation, repeated `--set` updates, camelCase business profile fields, schema `neditor.ned-profile.v1`, Markdown identity output, Docs Live placeholder output, dry-run protection, alias command support, shell-completion generation, and unknown-field validation. |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding Settings CLI copy and guards that the CLI source exposes the profile command, profile schema, and scaffold file. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting scriptable business profile setup and agentic step assistance in the README, specification, completion matrix, and progress log. |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding Agent Workspace `stepAssistance`, generated `AI Step Assistance` packet evidence, UI controls for adding suggested answers into context, the shared `appendAgenticStepAssistanceContext` helper that carries rationale/context signals into replanning, Docs Live workflow next-best-action assistance for every systematic stage, and static UI guards for the assistance section. |
| `pnpm run check:ai-roadmap` | Pass | AI-first roadmap contract passed for 50 changes across 10 sections and wrote `.tmp/ai-first-roadmap/report.json` after the agentic step-assistance update. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks`: 116 matrix rows, 9 complete, and 107 still partial/open after adding the explicit 11.4 step-assistance row. |
| `pnpm run check:release-readiness` | Pass with external gaps | Wrote `.tmp/release-readiness/report.json` with status `current-host-ready-with-external-gaps` and 15 evidence gaps; current host checks remain accepted, but release evidence still needs signing/notarization, Windows/Linux package and WebDriver proof, live Google Docs import/readback, final Homebrew cask/artifact proof, live AI provider proof, real-device AI runtime proof, release-device performance profiling, independent security review, native-viewer/export human sign-off, and assistive-technology sign-off. |
| `cargo test --locked file_command_tests --lib` in `src-tauri` | Pass | 8 file command tests passed, including `reveal_command_for_existing_path_is_platform_specific_and_argument_safe`. |
| `cargo test --locked desktop_native_command_workflow_smoke --lib` in `src-tauri` | Pass | Native command workflow smoke passed against real local files and direct export outputs. |
| `cargo test --locked representative_rendered_export_artifacts_are_package_inspectable --lib` in `src-tauri` | Pass | Representative rendered/package export audit passed across HTML, PDF, DOCX, PPTX, Markdown bundle, blog, Substack, LaTeX, and Google Docs package evidence. |
| `pnpm run test:rendered-exports` | Pass | Generated and verified `.tmp/rendered-export-audit` artifacts for HTML, PDF, DOCX, PPTX, Markdown bundle, blog, Substack, LaTeX, and Google Docs package outputs, including hashes, a manual checklist report, `manual-review.html`, `viewer-proof.json` executable viewer/package assertions, Poppler `pdfinfo`/`pdftotext` PDF metadata/text proof on this host, publishing handoff metadata checks, LaTeX source checks, nested Google Docs DOCX checks, macOS `textutil` extraction for primary/nested/review-case DOCX files, macOS Quick Look PDF/DOCX/PPTX thumbnail classification, Chromium-rendered screenshots and DOM/scroll metrics for the primary HTML export, the manual dashboard, and rich-block/option-heavy HTML review cases under `browser-visual-proof/`, `review-cases/rich-blocks` and `review-cases/option-heavy` artifact proof, and a `pdflatex` compile proof on this host. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 28 export command tests passed, including blog/Substack publish packages, LaTeX export, Google Docs package export, sidecar manifests, readiness diagnostics, progress steps, and native command workflow smoke. |
| `pnpm run verify:local` | Pass | Quick local verification passed: frontend typecheck, frontend unit tests, project structure, accessibility, dependency admission, Markdown links, Rust formatting, Rust `cargo check --locked`, and `git diff --check`. |
| `pnpm run verify:local:full` | Pass | Full local verification passed: quick checks, production build, optional engine probe, native-watch check, clippy, 213 Rust tests, rendered export audit, Tauri no-bundle release compile, macOS `.app` bundle build/smoke plus DMG classification on this host, desktop artifact/native-command smoke, and the desktop WebDriver harness step. Optional engine probe writes `.tmp/external-engines/probe-report.json`; configured runs now also accept the local `.tmp` Pikchr executable through `NEDITOR_TEST_PIKCHR`. |
| `pnpm run check:google-docs-import` | Pass with pending Google Drive authorization | Wrote `.tmp/google-docs-import/report.json` and `.tmp/google-docs-import/import-evidence.template.json`, verified the local rendered Google Docs DOCX/package hashes from `.tmp/rendered-export-audit`, and recorded `pending-google-drive-authorization` until live Drive import/readback evidence is supplied. |
| `pnpm run verify:local:full -- --list` | Pass | The full local verification plan now includes `Google Docs import evidence contract: pnpm run check:google-docs-import` immediately after rendered export audit. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration audit passed and wrote `.tmp/desktop-bundle/platform-package-config-report.json` with synchronized npm/Cargo/Tauri metadata, `bundle.targets: "all"`, macOS/Windows/Linux icon coverage, production window dimensions, CSP guardrails, MIT license linkage, and `unsigned-local-builds` signing stance. |
| `pnpm run verify:local:full -- --list` | Pass | The full local verification plan now includes `Platform package configuration: pnpm run check:platform-packaging` before the desktop release compile and host bundle checks. |
| `pnpm run check:platform-evidence` | Pass with pending external evidence | Wrote `.tmp/platform-evidence/report.json` and templates under `.tmp/platform-evidence/templates/`. The report records `pending-external-evidence` with four missing supported-host items: Windows package artifacts, Windows Tauri WebDriver, Linux package artifacts, and Linux Tauri WebDriver, and zero invalid supplied reports. |
| `node scripts/collect-platform-evidence.mjs --platform linux/win32 ...` plus `NEDITOR_PLATFORM_EVIDENCE_DIR=.tmp/platform-collector-fixture/evidence pnpm run check:platform-evidence` | Pass | Synthetic supported-host fixtures proved the collector writes validator-compatible `package-artifacts.json` and `tauri-webdriver-report.json` files for both Linux and Windows. The default `pnpm run check:platform-evidence` was then rerun to restore the real current-host `pending-external-evidence` report. |
| `pnpm run verify:local:full -- --list` | Pass | The full local verification plan now includes `External platform evidence contract: pnpm run check:platform-evidence` before desktop release compile and host bundle checks. |
| `pnpm run check:release-signing` | Pass with pending release credentials | Wrote `.tmp/release-signing/report.json` and templates under `.tmp/release-signing/templates/`. The report records `pending-release-credentials` with three missing signing evidence items: macOS codesign/notarization/spctl, Windows Authenticode/timestamp, and Linux package signature/checksum, and zero invalid supplied reports. |
| `node scripts/collect-release-signing-evidence.mjs --platform darwin/win32/linux ...` plus `NEDITOR_RELEASE_SIGNING_DIR=.tmp/release-signing-collector-fixture/evidence pnpm run check:release-signing` | Pass | Synthetic credentialed-host fixtures proved the collector writes validator-compatible macOS, Windows, and Linux `signing-evidence.json` files with artifact hashes and required proof summaries. The default `pnpm run check:release-signing` was then rerun to restore the real current-host `pending-release-credentials` report. |
| `pnpm run verify:local:full -- --list` | Pass | The full local verification plan now includes `Release signing evidence contract: pnpm run check:release-signing` before desktop release compile and host bundle checks. |
| `pnpm run check:ai-provider` | Pass with pending provider credentials | Writes `.tmp/ai-provider-evidence/report.json` and `.tmp/ai-provider-evidence/templates/provider-evidence.template.json`, records `pending-live-provider-evidence` when no credentialed proof is supplied, and rejects malformed, stale, marker-missing, or secret-bearing provider evidence. |
| `pnpm run check:ai-runtime` | Pass with pending real-device evidence | Writes `.tmp/ai-runtime-evidence/report.json` and `.tmp/ai-runtime-evidence/templates/runtime-evidence.template.json`, records `pending-real-runtime-evidence` when no real microphone/clipboard proof is supplied, and rejects malformed, stale, audio-recording, or clipboard-content evidence. |
| `pnpm run verify:local -- --list` / `pnpm run verify:local:full -- --list` | Pass | The quick and full local verification plans now include `AI provider evidence contract: pnpm run check:ai-provider` and `AI runtime evidence contract: pnpm run check:ai-runtime`, and the full plan keeps the provider/runtime contracts near release evidence collection before release readiness aggregation. |
| `pnpm run verify:local:full -- --list` | Pass | The full local verification plan now includes `Accessibility runtime audit: pnpm run check:a11y:runtime` and `Accessibility manual review contract: pnpm run check:a11y:manual` after the browser workflow suite. |
| `pnpm run verify:local:full -- --list` | Pass | The full local verification plan now ends with `Release readiness aggregation: pnpm run check:release-readiness`. |
| `pnpm run check:release-readiness` | Pass with external gaps | Aggregates current local reports into `.tmp/release-readiness/report.json`, including structured full-suite browser workflow proof, linked focused e2e evidence for runtime accessibility and performance, current macOS app-bundle/DMG classification proof, current-binary native desktop smoke and macOS WebDriver fallback proof, the Google Docs import contract, AI provider/runtime evidence contracts, external platform contract, and release signing evidence contract. It reports zero failed required checks when the current-host proof set is fresh, with explicit external evidence gaps for signing/notarization, Windows/Linux WebDriver execution, Windows package artifacts, Linux package artifacts, live AI provider endpoint proof, real-device AI runtime proof, Google Docs live import/readback, rendered export native-viewer sign-off, and accessibility assistive-technology sign-off. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 53 Chromium workflow tests in `e2e/app-workflows.spec.ts`, including first-class outline mode CRUD and the Docs Live draft workflow. |
| `pnpm run check:e2e-env` | Pass | Focused workbench boot workflow passed on this host through the workspace-local Playwright Chromium cache at `.tmp/ms-playwright`; `.tmp/e2e-environment/report.json` records `source: playwright-bundled`, the cache executable path, and the passing command tail. |
| `pnpm run check:a11y:runtime` | Pass | Focused runtime accessibility audit passed 6 Chromium workflows on this host with browser-launch permission, covering skip links, primary regions across desktop/narrow viewports, modal focus/Escape return, keyboard-only deep controls, status/progress live regions, and editor settings/search/heading commands; `.tmp/accessibility/runtime-report.json` records the expected workflows, command, browser output tail, zero issues, and links `.tmp/accessibility/e2e-runtime-report.json` so the canonical full-suite e2e report is not overwritten. |
| `pnpm run check:a11y:manual` | Pass | Wrote `.tmp/accessibility/manual-review-template.json` and `.tmp/accessibility/manual-review-summary.json` with `pending-human-review` status for the required manual screen-reader/native assistive-technology checklist. |
| `NEDITOR_ACCESSIBILITY_SIGNOFF=.tmp/accessibility/completed-review-smoke.json pnpm run check:a11y:manual` | Pass | Validated the completed-signoff path against a local smoke reviewer file, requiring static/runtime accessibility reports, reviewer metadata, platform, assistive technology, passing checklist notes, and zero unresolved blockers before marking the summary `human-reviewed`. This proves the validator contract, not a completed real manual review. |
| `pnpm run test:e2e` | Pass | Re-run on 2026-05-23 passed 53 Chromium browser workbench workflows locally on this host through the workspace-local Playwright Chromium cache. `.tmp/e2e-browser/report.json` now records schema `neditor.e2e-browser-workflow.v1`, `tests: 53`, `passed: 53`, zero failures/timeouts, and workflow evidence for Docs Live, editable outline planning, outline mode CRUD, and export workflows. |
| `pnpm run test:performance-audit` | Pass | Re-run on 2026-05-23 passed 4 Rust performance stress tests and the focused large-document Chromium workflow. `.tmp/performance-audit/report.json` now links `.tmp/performance-audit/e2e-large-document-report.json`, which records a focused e2e report without replacing the full-suite browser readiness proof. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "Docs Live" --project chromium` | Pass | Focused Docs Live browser workflow passed on this host, covering Writing-toolbar launch, document type, outline, mocked Web Speech dictation into spoken direction, context, placeholder values, generated Markdown preview, draft application, and rendered review-preparation output. |
| `pnpm run test:desktop-smoke` | Pass | Checked NEditor desktop build artifacts and native command workflow smoke; wrote `.tmp/desktop-smoke/native-command-report.json` with binary/build metadata and native command workflow duration. |
| `./node_modules/.bin/tauri build --bundles app` | Pass | Rebuilt `src-tauri/target/release/bundle/macos/NEditor.app` on this macOS host. |
| `pnpm run test:desktop-bundle` | Pass | Verified `NEditor.app` Info.plist metadata, bundle identifier, version, executable, icon, copyright, and high-resolution flag; wrote `.tmp/desktop-bundle/macos-app-report.json` with the current executable size `16329616` and icon size `98451`. |
| `pnpm run test:desktop-dmg` | Pass | Reclassified this sandboxed macOS host's DMG limitation after the fresh app-bundle report: `hdiutil create` cannot start `hdiejectd` because the process is sandboxed and returns `Device not configured`; wrote `.tmp/desktop-bundle/macos-dmg-report.json`, and release readiness now requires the classification to be newer than the app-bundle report. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Re-run after the fresh desktop binary checked NEditor desktop build artifacts, native command workflow smoke, and bounded native GUI launch on this macOS host; the run writes `.tmp/desktop-smoke/launch-report.json` with PID, elapsed window, captured output, `processAlive: true`, app-authored native window evidence, and app-authored native UI evidence. `.tmp/desktop-smoke/native-command-report.json` now records binary size `16329616`, 86 passing native workflow assertions, and current report mtimes that release readiness recomputes against the binary before accepting. |
| `pnpm run test:tauri-webdriver` | Skipped on macOS with current native fallback proof | Official Tauri WebDriver remains unsupported for macOS WKWebView, but the report was refreshed after the native launch smoke. `.tmp/desktop-webdriver/report.json` now records `fallbackProof.freshForBinary: true`, binary mtime `2026-05-23T15:20:51.748Z`, smoke report mtime `2026-05-23T15:28:37.244Z`, launch report mtime `2026-05-23T15:28:37.239Z`, 86/86 passing assertions, and the HTML export output hash. Release readiness independently re-stats those files before accepting the fallback. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "manages external transform engine trust" --project chromium` | Pass | Focused system-Chrome fallback workflow passed after keeping transform trust diagnostics visible when a configured path is untrusted. |
| `pnpm run test:e2e` | Pass | Re-run on 2026-05-23 passed all 53 Chromium workflows through the workspace-local Playwright Chromium cache and wrote structured browser evidence with Docs Live, outline planning, outline CRUD, and export workflow flags. |
| `pnpm run verify:local -- --list` | Pass | Quick local verification now lists the browser workflow environment preflight, so Chromium launch readiness is part of routine completed-slice verification. |
| `pnpm run verify:local:full -- --list` | Pass | Full local verification now lists the full browser workflow suite after the production frontend build, making browser workflow execution part of the release-grade local baseline. |
| `pnpm run verify:local` | Pass | Quick local verification passed with the new browser workflow environment gate included; the gate used the workspace-local Playwright Chromium cache and would retry transient browser-launch failures before failing the baseline. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "collapses and restores command toolbars" --project chromium` | Pass | Focused Chromium workflow proved individual toolbar collapse/expand, Collapse all, recovery through the compact View expander, and command-bar height reduction. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed from the updated source; 62 modules transformed. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary rebuilt from the updated frontend bundle at `src-tauri/target/release/neditor`. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Re-run on 2026-05-23 validated desktop build artifacts, native command workflow smoke, bounded launch smoke, mode pane visibility, rendered outline/HTML export/review/presentation content, guarded native `File` -> `Export` -> `HTML Export` routing, native editor ergonomics evidence for word stats, spellcheck attributes, line numbers, word wrap, folding gutter, search/replace, list continuation, bracket pairing, multi-cursor editing, native Outline sidebar click-to-source navigation, and native Writing Tools -> Docs Live draft generation/application. The workflow report recorded `status: passed`, 86 passing assertions, `nativeMenuCommandEvidence.docsLive.generated.workflow: true`, `sections: 3`, and `applied.sidebar: "review"` with drafting-plan, Section QA, Review Preparation, and humanization workflow markers present. |
| `pnpm run test:tauri-webdriver` | Skipped on macOS with native proof | The Tauri WebDriver harness is present and runs on Windows/Linux with `tauri-driver`; it now covers native title/shell, mode switching, command palette, outline-mode structural editing, dirty-title state, Templates-panel calc insertion to source/preview, real Markdown save/open/rename/duplicate/reveal, export readiness, real HTML export writing through the guarded dialog-free smoke path with sidecar manifest/output-hash validation, and preference restart persistence. This macOS host records the official unsupported WKWebView-driver platform skip, the supported workflow plan, and `fallbackProof.status: "passed"` with outline mode titles plus outline navigation line evidence in `.tmp/desktop-webdriver/report.json`. |

Fresh baseline recorded on 2026-05-20:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 6 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 53 modules transformed. |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff. |
| `cargo check --locked` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 126 Rust tests passed; 0 failed. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary built at `src-tauri/target/release/neditor`. |

Additional export option matrix verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after adding the option-matrix conformance test. |
| `cargo test --locked export_option_tests --lib` in `src-tauri` | Pass | 9 export option tests passed, including `export_option_matrix_is_preserved_across_targets_and_bundle_evidence`. |
| `cargo test --locked` in `src-tauri` | Pass | 144 Rust tests passed; 0 failed. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 12 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional rich Markdown block export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after adding the rich-block conformance test. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 5 export conformance tests passed, including `rich_markdown_blocks_survive_cross_target_exports`. |
| `cargo test --locked` in `src-tauri` | Pass | 145 Rust tests passed; 0 failed. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 12 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional external transform cache identity verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after adding same-path executable cache invalidation coverage. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 9 external transform tests passed, including `external_transform_cache_invalidates_when_trusted_executable_changes`. |
| `cargo test --locked` in `src-tauri` | Pass | 146 Rust tests passed; 0 failed. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 12 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional external transform platform evidence:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:engines` | Partial pass | On Darwin arm64, Graphviz/DOT was installed at `/opt/homebrew/bin/dot` with version `14.1.5`, D2 was installed at `/opt/homebrew/bin/d2` with version `0.7.1`, PlantUML was installed at `/opt/homebrew/bin/plantuml` with version `1.2026.3`, and Pikchr was missing. The command exits successfully because optional engines may be absent. |
| `cargo test --locked external_transform_conformance_runs_installed_engines --lib -- --nocapture` in `src-tauri` | Partial pass | The installed-engine conformance test verified `dot`, `d2`, and `plantuml` through NEditor's external transform execution path and skipped `pikchr` because it is not installed on this host. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after adding `check:engines` to the local verification script contract. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 13 Markdown files, including the new platform evidence doc; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional citation readiness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_missing_citation_sources_with_precise_ranges --lib -- --nocapture` in `src-tauri` | Pass | Proved no-bibliography citation readiness emits the broad source warning plus one precise per-key diagnostic for each missing citation key, with line/column/end-column ranges and manifest parity. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 15 export command tests passed, including the new missing-citation readiness range coverage. |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after the validation change. |
| `cargo check --locked` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 147 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 57 modules transformed. |
| `pnpm run check:docs` | Pass | Checked 13 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed, with Pikchr missing as an explicit optional-engine gap. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests; execution still depends on local Chromium availability and host permissions. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional citation and generated-section verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_duplicate_bibliography_keys --lib -- --nocapture` in `src-tauri` | Pass | Proved duplicate bibliography diagnostics point at the second duplicate entry and include the first occurrence in related context. |
| `cargo test --locked compiler_reports_csl_and_hayagriva_duplicate_key_locations --lib -- --nocapture` in `src-tauri` | Pass | Proved duplicate-key source ranges for CSL JSON array entries and Hayagriva YAML top-level entries, including preservation of duplicate Hayagriva keys before YAML map parsing can collapse them. |
| `cargo test --locked compiler_reports_duplicate_keys_across_external_bibliography_files --lib -- --nocapture` in `src-tauri` | Pass | Proved multiple bibliography front matter paths can load separate BibTeX and CSL JSON files, and duplicate-key diagnostics point at the duplicate key in the second external file with first-file context. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 14 citation tests passed, including BibTeX, CSL JSON, Hayagriva, external-file duplicate-key source ranges, numeric citation rendering, unsupported CSL-style fallback diagnostics, and citation export conformance. |
| `cargo test --locked compiler_generates_glossary_sections_from_marker_and_metadata --lib -- --nocapture` in `src-tauri` | Pass | Proved `[GLOSSARY]` marker replacement, front matter-driven generated glossary insertion, preview hover preservation, and DOCX glossary artifact text. |
| `cargo test --locked cross_references_resolve_heading_appendix_and_decision_anchors --lib -- --nocapture` in `src-tauri` | Pass | Proved unprefixed appendix and decision anchors render as Appendix and Decision labels rather than generic section labels. |
| `cargo test --locked heading_appendix_and_decision_references_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Proved section, appendix, and decision cross-reference labels survive HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked compiler_generates_index_from_front_matter_without_marker --lib -- --nocapture` in `src-tauri` | Pass | Proved `index.enabled: true` front matter generates an index without `[INDEX]`, preserves linked terms, honors exclusion settings, and strips inline index markers. |
| `cargo test --locked front_matter_index_survives_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Proved `index: true` front matter-generated index content survives HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo fmt --check` in `src-tauri` | Pass | Completed with no formatting diff after carrying bibliography source metadata. |
| `cargo check --locked` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Finished dev profile successfully. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Finished with no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 156 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after extending bibliography entry shape and numeric citation-style preference normalization. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed with duplicate source locations and numeric citation style controls in the References panel. |
| `pnpm run check:docs` | Pass | Checked 13 Markdown files; local links resolve. |
| `pnpm run check:a11y` | Pass | Checked `App.vue` template accessibility guardrails. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed, with Pikchr missing as an explicit optional-engine gap. |
| `pnpm exec playwright test --list` | Pass | Listed 35 Chromium workflow tests, including reference-navigation and command-palette insertion coverage for `[GLOSSARY]` and other generated sections. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |

Additional focused verification after workflow helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed, including AI paste insertion modes and conflict merge-line composition. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |

Additional browser workflow harness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright --version` | Pass | Playwright reported `Version 1.60.0`. |
| `PLAYWRIGHT_BROWSERS_PATH=.tmp/ms-playwright pnpm exec playwright install chromium` | Pass | Chromium, FFmpeg, and Chromium headless shell downloaded into Playwright's workspace-local browser cache. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test --list` | Pass | Listed 4 Chromium tests in `e2e/app-workflows.spec.ts`. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm run test:e2e` | Blocked by sandbox | Chromium launch failed before app assertions with `bootstrap_check_in ... Permission denied (1100)`. |

Additional table editor workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding table-draft cancellation and accessible row/column controls. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 14 Chromium workflow tests, including `edits table structure with formats and cancels draft changes`. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "edits table structure" --project chromium` | Blocked by sandbox | Chromium launched, then failed before app assertions with `bootstrap_check_in ... Permission denied (1100)`. Escalated local browser execution was rejected by the approval reviewer. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |
| `gh run watch 26143491444 --exit-status` | Failure diagnosed | For `5a97d86`, 13 browser tests passed and the new table-structure test failed because the assertion expected raw `74000` after the Value column had been switched to currency format; the preview correctly rendered `$74000`. |
| `pnpm run test:unit` | Pass | Re-run after fixing the table-structure assertion; 8 frontend unit tests passed. |
| `pnpm run build` | Pass | Re-run after fixing the table-structure assertion; `vue-tsc --noEmit` and Vite build passed. |
| `pnpm exec playwright test --list` | Pass | Re-run after fixing the table-structure assertion; still lists 14 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after fixing the table-structure assertion. |
| `gh run view 26143632239 --json status,conclusion,headSha,jobs,url` | Pass | Commit `dbd440d` passed browser workflow tests and Ubuntu/macOS/Windows desktop builds. |

Additional AI paste browser workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding AI paste quote, appendix, replace-document, section-merge, and replace-selection browser coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 17 Chromium workflow tests, including the AI paste mode workflows. |
| `gh run view 26144290812 --job 76896091795 --log` | Failure diagnosed | First pushed AI paste browser run had 15 passing tests; the two failures were transient status-bar assertions after the editor mutations had succeeded. |
| `gh run view 26144430209 --json status,conclusion,headSha,jobs,url` | Pass | Commit `3b17c03` passed 17 browser workflow tests and Ubuntu/macOS/Windows desktop builds. |

Additional included-file watcher workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding include-aware browser mock compilation and included-file watcher workflows. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 19 Chromium workflow tests, including clean included-file recompile and dirty included-file conflict workflows. |
| `git diff --check` | Pass | No whitespace errors after adding included-file watcher workflow coverage. |
| `gh run watch 26145509141 --exit-status` | Pass | Commit `c0cefd1` passed 19 browser workflow tests and Ubuntu/macOS/Windows desktop builds. |

Additional workspace restore workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after persisting mode/sidebar with the workspace restore state. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 20 Chromium workflow tests, including restart-style workspace restore for tabs, active document, pins, mode, sidebar, workspace root, and recent files. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "restores workspace" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the workspace cache. |
| `git diff --check` | Pass | No whitespace errors after adding workspace restore persistence coverage. |
| `gh run view 26147556750 --json status,conclusion,headSha,jobs,url` | Pass | Commit `655d65c` passed the 20-test browser workflow job plus Ubuntu/macOS/Windows desktop builds. |

Additional missing-file and scroll restore workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding per-document scroll persistence and missing-restore state. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 21 Chromium workflow tests, including missing restored-file warning coverage and scroll-position restore in the workspace reload workflow. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "restores workspace\|skips missing" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `gh run view 26148828614 --json status,conclusion,headSha,jobs,url` | Pass | Commit `5a13fe2` passed the 21-test browser workflow job plus Ubuntu, macOS, and Windows desktop builds. |
| `git diff --check` | Pass | No whitespace errors after adding scroll and missing-file restore coverage. |

Additional tab and stale-recent workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding dirty-close confirmation and stale recent cleanup paths. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed; 54 modules transformed. |
| `pnpm exec playwright test --list` | Pass | Listed 22 Chromium workflow tests, including tab activation, dirty close confirmation, renamed recent cleanup, and deleted recently-closed pruning coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "switches tabs" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `pnpm exec playwright install chromium` | Interrupted | The install attempt produced no output for several minutes, so the hung process tree was terminated; browser workflow proof still needs a local host where Playwright can launch Chromium. |
| `git diff --check` | Pass | No whitespace errors after adding tab and stale-recent coverage. |
| `gh run view 26150151349 --json status,conclusion,headSha,jobs` | Superseded failure | Commit `44c49f5` ran 20 passing browser workflows before the new tab/stale-recent test exposed title and dirty-close mock assumptions. |
| `gh run view 26150581320 --job 76916713881 --log` | Superseded failure diagnosed | Commit `3fe0c78` still failed because the Tauri dialog mock handled `plugin:dialog\|confirm`, while installed `@tauri-apps/plugin-dialog` implements `confirm()` through `plugin:dialog\|message`. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after correcting the dialog mock and new-document tab-count assertions. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after correcting the dialog mock and new-document tab-count assertions. |
| `pnpm exec playwright test --list` | Pass | Still listed 22 Chromium workflow tests after the dialog mock correction. |
| `git diff --check` | Pass | No whitespace errors after the dialog mock correction. |
| `gh run view 26151007765 --job 76918155717 --log` | Superseded failure diagnosed | Commit `39a3286` ran 21 passing browser workflows; the remaining failure was a broad `Rename` locator matching the `Rename Source` tab and file row. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after disambiguating the rename command locator. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after disambiguating the rename command locator. |
| `pnpm exec playwright test --list` | Pass | Listed the same 22 Chromium workflow tests after the exact rename locator fix. |
| `git diff --check` | Pass | No whitespace errors after the exact rename locator fix. |
| `gh run watch 26151184228 --exit-status` | Pass | Commit `bf60405` passed 22 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional recent-folder and moved-path workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding stale recent-folder cleanup and moved recently-closed path coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the recent-folder and moved-path workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 23 Chromium workflow tests, including recent folder reopen/prune behavior and moved recently-closed path pruning. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "reopens recent folders" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding recent-folder and moved-path coverage. |
| `gh run watch 26152255407 --exit-status` | Pass | Commit `13b3086` passed 23 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional preview navigation workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding preview scroll-sync and heading navigation browser coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the preview navigation workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 24 Chromium workflow tests, including preview scroll sync and heading click-to-source coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "syncs editor" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding preview navigation coverage. |
| `gh run watch 26153224371 --exit-status` | Pass | Commit `7702e89` passed 24 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional editor ergonomics workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding editor ergonomics browser coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the editor ergonomics workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 25 Chromium workflow tests, including persisted editor settings, find/replace, list continuation, bracket auto-pairing, and command-palette heading navigation coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "persists editor settings" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding editor ergonomics coverage. |
| `gh run watch 26154535588 --exit-status` | Pass | Commit `f13c3f3` passed 25 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional command palette reference navigation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding citation/glossary/index command palette workflow coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the command palette reference navigation update. |
| `pnpm exec playwright test --list` | Pass | Listed 26 Chromium workflow tests, including command palette citation, glossary, and index navigation coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "citation glossary" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding command palette reference navigation coverage. |
| `gh run watch 26155535210 --exit-status` | Pass | Commit `5f75e44` passed 26 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional command palette document/workspace navigation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding command palette open-document and workspace-file workflow coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the command palette document/workspace navigation update. |
| `pnpm exec playwright test --list` | Pass | Listed 27 Chromium workflow tests, including command palette open-document switching and workspace-file opening. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "open document and workspace file" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding command palette document/workspace navigation coverage. |
| `gh run watch 26156393184 --exit-status` | Pass | Commit `145942a` passed 27 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional transform settings workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding transform engine trust/probe browser workflow coverage. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the transform settings workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 28 Chromium workflow tests, including transform engine trust/probe diagnostics coverage. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "transform engine trust" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache. |
| `git diff --check` | Pass | No whitespace errors after adding and stabilizing transform settings workflow coverage. |
| `gh run watch 26157446711 --exit-status` | Pass | Commit `976016c` passed 28 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional export readiness/result workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after export result diagnostics and immediate-export editor-state flush. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the export readiness/result workflow update. |
| `pnpm exec playwright test --list` | Pass | Listed 28 Chromium workflow tests, including export readiness, success, failure, and blocked-readiness workflow coverage. |
| `pnpm exec playwright install chromium` | Blocked locally | Browser install is sandbox-blocked by `EPERM` while creating `/Users/nyimbiodero/Library/Caches/ms-playwright/__dirlock`. |
| `git diff --check` | Pass | No whitespace errors after the export workflow fixes. |
| `gh run watch 26159396761 --exit-status` | Pass | Commit `02e832a` passed 28 browser workflow tests plus Ubuntu, macOS, and Windows desktop builds. |

Additional document grouping workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright test --list` | Pass | Listed 34 Chromium workflow tests, including `groups documents by document set and folder`. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding document-set metadata to the browser mock. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the grouping workflow. |
| `git diff --check` | Pass | No whitespace errors after adding the grouping workflow and docs. |
| Markdown local link resolution script | Pass | Updated markdown docs contain no broken local links. |

Additional caption-list export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after generated-section changes. |
| `cargo test --locked compiler_generates_lists_of_figures_and_tables --lib` in `src-tauri` | Pass | Focused test proves generated list markers, numbering, anchors, fenced-example exclusion, preview HTML, and DOCX artifact text. |

Additional generated-list insertion workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright test --list` | Pass | Browser harness lists the command-palette insertion workflow that now asserts `[TOC]`, `[INDEX]`, `[BIBLIOGRAPHY]`, `[LIST_OF_FIGURES]`, and `[LIST_OF_TABLES]` insertion. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding generated-section insertion commands. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding generated-section insertion commands. |

Additional references-panel bibliography verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright test --list` | Pass | Browser harness lists the citation/glossary/index workflow that now asserts resolved bibliography entries, missing citation keys, and duplicate bibliography keys. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding bibliography-aware browser mock compilation. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding bibliography-aware browser mock compilation. |

Additional review/provenance readiness metadata verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after adding review/change-note, AI provenance, caption/label readiness diagnostics, and direct-export dirty-Git manifest warnings. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the direct-export readiness update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native file watcher feature still compiles after the direct-export readiness update. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 13 export command tests passed, including readiness and manifest diagnostics for malformed review comments, change notes, incomplete AI provenance metadata, invalid AI review statuses, missing figure/table/equation labels or captions, explicit manifest readiness summaries, direct-export dirty-Git manifest warnings, and structured export progress-step reporting. |
| `cargo test --locked review_provenance_tests --lib` in `src-tauri` | Pass | 4 review/provenance tests passed after the stricter AI provenance validation. |
| `cargo test --locked validation_tests --lib` in `src-tauri` | Pass | 4 validation tests passed after the readiness validation extension. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the shared parser and export-readiness validation changes. |
| `cargo test --locked compiler_stress_handles_large_documents_with_many_artifacts --lib` in `src-tauri` | Pass | Large-document compiler stress test passed with nested includes, 80 tables, 80 CSV transform artifacts, 120 formula definitions, many source-map entries, and many broken link/media diagnostics. |
| `cargo test --locked repeated_export_loop_keeps_large_artifacts_stable --lib` in `src-tauri` | Pass | Repeated export-loop stress test passed across HTML, PDF, DOCX, PPTX, and Markdown bundle rendering for a large compiled document. |
| `cargo test --locked repeated_editing_sessions_reuse_external_transform_cache --lib` in `src-tauri` | Pass | Repeated editing/cache stress test passed with a trusted external DOT transform executed once and served from cache during subsequent edits. |
| `cargo test --locked export_readiness_and_manifest_report_progress_steps --lib` in `src-tauri` | Pass | Export progress-step test passed, proving readiness/export manifests expose compile, transform, readiness, render, and manifest stages. |
| `cargo test --locked prepare_for_export_carries_broad_readiness_audit_to_manifest --lib` in `src-tauri` | Pass | Broad readiness audit test passed, proving report/manifest parity for metadata, approval, layout, citations, comments, AI provenance, links/media, formulas, captions, transform settings, and progress state. |
| `cargo test --locked compiler_loads_front_matter_csv_data_sources --lib` and `cargo test --locked markdown_bundle_keeps_duplicate_include_basenames_distinct --lib` in `src-tauri` | Pass | Include-graph manifest tests passed, proving data-source/include edges appear in export manifests and Markdown bundles include both `manifest.json` include graph data and `include-graph.json`. |
| `cargo test --locked compiler_renders_openapi_and_json_schema_tables --lib` in `src-tauri` | Pass | Structured transform test passed, proving richer OpenAPI operation/server/parameter/request/response/component rendering and nested JSON Schema field/constraint rendering. |
| `cargo test --locked export_conformance_fixture_maps_business_features --lib` in `src-tauri` | Pass | Export conformance fixture now proves legal disclaimer and approval metadata across HTML, PDF, DOCX package properties/body, PPTX package properties/slides, plain text, and Markdown bundle metadata. |
| `cargo test --locked repeated_compile_export_cycles_keep_memory_growth_bounded --lib` in `src-tauri` | Pass | Repeated compile/export memory-growth stress passed with bounded retained summaries and process RSS growth sampling. |
| `cargo test --locked git_restore_and_tag_reject_option_shaped_refs --lib` in `src-tauri` | Pass | Git tag/revision option-injection regression passed. |
| `cargo test --locked git_restore_refuses_symlink_targets --lib` in `src-tauri` | Pass | Git restore refused a symlinked worktree file and left the outside target unchanged. |
| `cargo test --locked git_history_diff_commit_tag_and_restore_workflow --lib` in `src-tauri` | Pass | Existing Git history, diff, commit, tag, and restore workflow still passed after the new guards. |
| `cargo test --locked` in `src-tauri` | Pass | 143 Rust tests passed plus main/doc test targets with 0 tests on this Unix host. |
| `npx playwright test e2e/app-workflows.spec.ts -g "keeps large document editing"` | Blocked | The large-document browser workflow is present, but this host is missing Playwright Chromium at `~/Library/Caches/ms-playwright/.../chrome-headless-shell`. |
| `pnpm exec playwright test --list` | Pass | Browser harness lists 35 Chromium workflow tests, including the large-document interaction workflow. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed, including latest-document task cancellation/stale-result guard coverage, preview debounce timing/coalescing coverage, and workspace persistence migration/schema normalization. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guard passed for button names, form-control labels, and dialog labeling. |
| `pnpm run check:docs` | Pass | Markdown link guard now discovers README plus all top-level docs Markdown files, including the user guide and Markdown extensions reference. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary built at `src-tauri/target/release/neditor`. |
| `git diff --check` | Pass | No whitespace errors after the readiness metadata, dirty-Git manifest, and documentation updates. |
| `pnpm run check:docs` | Pass | Repo-wide markdown link guard checked README plus all docs and found no missing local links after adding `docs/architecture.svg`. |

Archived remote workflow evidence log:

| Command | Result | Evidence |
| --- | --- | --- |
| `gh run view 26131218116 --json jobs,conclusion,status,headSha,url` | Failure diagnosed | All jobs for `d94dc6c` failed at `Set up Node.js` before project commands ran. |
| `XDG_CACHE_HOME=.cache gh run view 26131218116 --log-failed` | Failure diagnosed | `actions/setup-node` could not locate `pnpm` while `cache: pnpm` was enabled before pnpm setup. |
| `gh run view 26131619095 --json jobs,conclusion,status,headSha,url` | Partial improvement | For `6489162`, pnpm and Node setup succeeded in every job; the browser job reached Playwright execution. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76857769468/logs` | Browser failure diagnosed | Playwright ran 4 tests in CI; 3 passed and the readiness test failed because `getByText("Ready")` matched both `Ready` and `Document is ready for export`. |
| `gh run view 26131805873 --json jobs,conclusion,status,headSha,url` | Browser workflow pass | For `420af08`, the `Browser workflow tests` job passed through Playwright Chromium installation and `pnpm run test:e2e`. |
| `gh run view 26131929125 --json jobs,conclusion,status,headSha,url` | Mixed CI result | For `9a6d52e`, browser workflows and macOS desktop passed; Windows desktop failed clippy, Ubuntu desktop failed Rust tests. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76858711971/logs` | Windows failure diagnosed | Clippy failed because `clear_external_transform_memory_cache_for_tests` in `src/transforms/external.rs:444` was dead code in the Windows lib-test target. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76858711986/logs` | Ubuntu failure diagnosed | `external_transform_conformance_runs_installed_engines` failed because the installed `pikchr-cli` requires a positional `<PIKCHR>` argument and exited with status 2. |
| `cargo fmt --check` in `src-tauri` | Pass | Re-run after the external transform adapter fix. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib` in `src-tauri` | Pass | Covered the first attempted `pikchr-cli` positional raw-source adapter path. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 8 external transform tests passed, including installed-engine conformance path and trust/timeout/error behavior. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Re-run after Unix-gating the test cache helper and adding the Pikchr adapter path. |
| `cargo test --locked` in `src-tauri` | Pass | 126 Rust tests passed plus main/doc test targets with 0 tests. |
| `gh run view 26132634911 --json status,conclusion,headSha,jobs` | Mixed CI result | For `25f7b04`, browser workflow and macOS desktop passed; Windows desktop failed Rust tests; Ubuntu desktop failed Rust tests. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76860838872/logs` | Windows failure diagnosed | Four Rust tests failed because serialized paths were not stable across Windows separators: file watch aliases, workspace listing, local figure media, and duplicate include bundle tests. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76860838881/logs` | Ubuntu follow-up diagnosed | The no-argument Pikchr failure moved forward, but `pikchr-cli` treated the raw source argument as a missing file path and exited with status 1. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib` in `src-tauri` | Pass | Covers `pikchr-cli` receiving a temporary `.pikchr` source path. |
| `cargo test --locked file_command_tests --lib` in `src-tauri` | Pass | Covers slash-normalized watched paths and workspace listing on the local platform. |
| `cargo test --locked media_export_tests --lib` in `src-tauri` | Pass | Covers slash-normalized manifest/include/media paths on the local platform. |
| `cargo fmt --check` in `src-tauri` | Pass | Re-run after the latest Pikchr/path fixes; formatting is clean. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 8 external transform tests passed with temporary source path delivery for `pikchr-cli`. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Re-run after the latest Pikchr/path fixes; no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 126 Rust tests passed plus main/doc test targets with 0 tests. |
| `git diff --check` | Pass | No whitespace errors in the current diff. |
| `gh run view 26133136580 --json status,conclusion,headSha,jobs` | Mixed CI result | For `5c29914`, browser workflow and macOS desktop passed; Windows passed Rust tests, frontend unit tests, and frontend build before continuing the Tauri shell build; Ubuntu failed one Rust fixture test. |
| `XDG_CACHE_HOME=.cache gh api repos/nyimbi/Neditor/actions/jobs/76862375005/logs` | Ubuntu fixture failure diagnosed | `external_transform_conformance_runs_installed_engines` passed, but `external_transform_adapters_shape_engine_specific_invocations` failed because the fake `d2` script did not consume stdin before exiting. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib` in `src-tauri` | Pass | Re-run after making the fake `d2` adapter consume stdin. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 8 external transform tests passed after the fake `d2` stdin fixture fix. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Re-run after the fake `d2` stdin fixture fix; no warnings. |
| `cargo test --locked` in `src-tauri` | Pass | 126 Rust tests passed plus main/doc test targets with 0 tests after the fake `d2` stdin fixture fix. |
| `git diff --check` | Pass | No whitespace errors after the latest documentation and fake-`d2` fixture edits. |
| `gh run watch 26133595556 --exit-status` | Pass | For `33ee6a9`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Ubuntu passed optional engine install, Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle build. |
| `git diff --check` | Pass | No whitespace errors after refreshing docs with the green CI result. |
| `pnpm exec playwright test --list` | Pass | Listed 5 Chromium browser workflow tests, including the new advanced table paste/sort/formula/merge/apply flow. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding the advanced table browser workflow. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding the advanced table browser workflow. |
| `gh run watch 26134248308 --exit-status` | Pass | For `443515b`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser workflow tests ran through Chromium installation and `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `pnpm exec playwright test --list` | Pass | Listed 6 Chromium browser workflow tests, including the new mocked file open/save/duplicate/rename/pin/reveal/revert flow. |
| `pnpm run test:unit` | Pass | 8 frontend unit tests passed after adding the mocked file lifecycle browser workflow. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding the mocked file lifecycle browser workflow. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "opens, saves, duplicates" --project chromium` | Blocked locally | Playwright could not launch because `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/chrome-headless-shell-mac-arm64/chrome-headless-shell` is missing. |
| `git diff --check` | Pass | No whitespace errors after stabilizing the mocked file lifecycle workflow assertions. |
| `gh run view 26135510740 --json status,conclusion,jobs` | Superseded failure | For `11dafc3`, the mocked lifecycle browser test still asserted the active tab text as a path even when front matter made the tab title `*Market Entry Report`. |
| `gh run view 26135873103 --json status,conclusion,jobs` | Superseded failure | For `b2ccf83`, the same browser assertion shape remained brittle before the file-row assertion fix. |
| `gh run view 26136003362 --json status,conclusion,jobs` | Superseded failure | For `a55970d`, the workflow tried to click workspace file rows while the sidebar was still in the review panel. |
| `gh run view 26136129630 --json status,conclusion,jobs` | Superseded failure | For `b7534f6`, `getByLabel("Pin document")` matched multiple tab buttons. |
| `pnpm exec playwright test --list` | Pass | Listed 6 Chromium browser workflow tests after the final active-tab pin selector fix. |
| `git diff --check` | Pass | No whitespace errors after the final browser selector fix. |
| `gh run watch 26136223804 --exit-status` | Pass | For `12cd667`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser CI installed Chromium and ran `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `pnpm exec playwright test --list` | Pass | Listed 7 Chromium browser workflow tests after adding save-as and recently closed reopening. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "saves a document as a new file" --project chromium` | Blocked locally | Default Playwright cache still lacks the Chromium headless-shell executable. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "saves a document as a new file" --project chromium` | Blocked locally | Workspace-local Chromium launched, then failed before assertions with `bootstrap_check_in ... Permission denied (1100)`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding accessible recent-path sections. |
| `git diff --check` | Pass | No whitespace errors after the save-as/recently closed workflow edits. |
| `gh run view 26137235763 --json status,conclusion,jobs` | Superseded failure | For `61eff87`, the new path appeared in both Recent files and Recently closed, so the browser test needed a scoped recently-closed selector. |
| `gh run view 26137384613 --json status,conclusion,jobs` | Superseded failure | For `44c9639`, reopened tab text reflected front matter title rather than the filesystem path; the test now proves path identity via the active workspace row. |
| `gh run watch 26137556147 --exit-status` | Pass | For `138bf5d`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser CI installed Chromium and ran 7 tests through `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `pnpm exec playwright test --list` | Pass | Listed 9 Chromium browser workflow tests after adding stale-save conflict copy and merge recovery workflows. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "stale saves\|external conflict" --project chromium` | Blocked locally | Workspace-local Chromium launched, then failed before assertions with `bootstrap_check_in ... Permission denied (1100)`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding stale-save conflict browser workflows. |
| `git diff --check` | Pass | No whitespace errors after the stale-save conflict browser workflow edits. |
| `gh run watch 26138478934 --exit-status` | Superseded failure | For `3cb1b84`, 8 browser tests passed but the merge test asserted a transient status message that compile refresh could overwrite. |
| `pnpm exec playwright test --list` | Pass | Listed 9 Chromium browser workflow tests after stabilizing the conflict merge assertion. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after removing the transient status assertion. |
| `git diff --check` | Pass | No whitespace errors after stabilizing the conflict merge workflow proof. |
| `gh run watch 26138672512 --exit-status` | Pass | For `25c7d1e`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser CI installed Chromium and ran 9 tests through `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `pnpm exec playwright test --list` | Pass | Listed 11 Chromium browser workflow tests after adding keep-local and accept-external stale-save conflict workflows. |
| `PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright test e2e/app-workflows.spec.ts --grep "keeps local\|accepts external" --project chromium` | Blocked locally | Workspace-local Chromium launched, then failed before assertions with `bootstrap_check_in ... Permission denied (1100)`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build completed after adding keep-local and accept-external conflict workflows. |
| `git diff --check` | Pass | No whitespace errors after the keep-local and accept-external conflict workflow edits. |
| `gh run watch 26139678118 --exit-status` | Pass | For `4eb1d2c`, browser workflows and Ubuntu/macOS/Windows desktop builds all passed. Browser CI installed Chromium and ran 11 tests through `pnpm run test:e2e`; desktop jobs passed Rust formatting/check/native-watch/clippy/tests, frontend tests/build, and Tauri no-bundle builds. |
| `gh run list --branch main --limit 5 --json databaseId,headSha,status,conclusion,displayTitle,createdAt,url` | Pass | Latest pushed run for `1ac72c1` completed successfully as run `26140882880`. |
| `gh run view 26140882880 --json jobs,conclusion,status,headSha,url` | Pass | Browser workflow tests and Ubuntu/macOS/Windows desktop builds all passed for `1ac72c1`. |

Archived remote-workflow fixes from before GitHub Actions was retired:

- The removed workflow had installed pnpm before Node setup after earlier
  cache failures.
- `tests/frontend-unit.test.ts` now verifies local package scripts instead of
  enforcing any GitHub Actions workflow.
- `e2e/app-workflows.spec.ts` now scopes the readiness `Ready` assertion to
  `article.readiness` with an exact text match so the browser workflow remains
  strict without matching the status-bar message.

Baseline command set:

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

Baseline gaps:

- The baseline includes an initial browser workflow harness, but it does not yet
  include a passing browser execution result from this sandbox.
- The baseline does not include desktop WebDriver/Tauri-driver workflow tests.
- The baseline does not include rendered visual QA for PDF/DOCX/PPTX outputs.
- The baseline now includes macOS `.app` bundle creation/smoke evidence and
  sandboxed-host DMG classification on the current host; Windows/Linux package
  bundle creation remains open.
- The baseline includes macOS optional external transform engine evidence for
  Graphviz/DOT variants, D2, and PlantUML; Pikchr and Windows optional-engine
  evidence remain open.

Optional packaging checks:

```sh
./node_modules/.bin/tauri build --bundles app
pnpm run test:desktop-dmg
```

Known packaging note from `README.md`:

- macOS app bundle builds are now current-host verified by
  `./node_modules/.bin/tauri build --bundles app` plus
  `pnpm run test:desktop-bundle`.
- DMG bundling reaches app bundle creation on this host, then `hdiutil create`
  fails because sandboxing prevents `hdiejectd` startup and returns `Device not
  configured`; `pnpm run test:desktop-dmg` records this as a host-specific
  limitation.

Additional equation caption verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_block_and_inline_equations --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves the documented equation `caption` attribute renders as `Equation N: Caption`, is retained as `data-caption`, is captured in the document AST, and satisfies caption-label readiness for the equation. |
| `cargo test --locked captioned_equations_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves captioned equation text, stable equation references, and the human caption survive HTML, PDF, DOCX, PPTX, Markdown bundle text, and Markdown bundle AST outputs. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after the equation caption parser, AST, and tests were added. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the equation caption update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the equation caption update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after preserving equation captions through rich-block rendering and AST parsing. |
| `cargo test --locked` in `src-tauri` | Pass | 157 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the documentation and backend update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional TOC export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked generated_toc_exports_page_numbers_for_pdf_and_docx --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves front matter TOC depth/numbering, page-numbered PDF TOC entries such as `1 Alpha .... 2`, DOCX TOC field output, and suppression of raw Markdown TOC links in DOCX. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after the PDF TOC page-numbering update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the PDF TOC page-numbering update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the PDF TOC page-numbering update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after adding PDF TOC post-pagination page-number rendering. |
| `cargo test --locked` in `src-tauri` | Pass | 158 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional include guard verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_circular_and_too_deep_includes --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves circular include diagnostics, related include-target context, include graph depth entries for the resolved chain, maximum include depth enforcement, and suppression of content beyond the depth limit. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after adding include guard regression coverage. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the include guard coverage update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the include guard coverage update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after adding circular/max-depth include guard coverage. |
| `cargo test --locked` in `src-tauri` | Pass | 159 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional invalid-include and table-formula export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_unreadable_include_targets_with_context --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves unreadable/invalid include target diagnostics include source file, source line, suggestion, original include target, and resolved path context. |
| `cargo test --locked named_table_formulas_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves named table/range formulas resolve and survive HTML, PDF, DOCX, PPTX, Markdown bundle text, and Markdown bundle AST outputs. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after the invalid-include diagnostic and named table formula export updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the invalid-include diagnostic and named table formula export updates. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the invalid-include diagnostic and named table formula export updates. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the invalid-include diagnostic and named table formula export updates. |
| `cargo test --locked` in `src-tauri` | Pass | 161 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional front matter data source verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_loads_front_matter_json_and_yaml_data_sources --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves JSON and YAML files referenced from front matter render through structured transform artifacts, preview HTML, include graph, and manifest included-file evidence. |
| `cargo test --locked compiler_reports_malformed_front_matter_data_sources --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves missing data source paths, unsupported data source types, and unreadable local files produce actionable diagnostics. |
| `cargo test --locked front_matter_data_sources_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves front matter CSV/TSV/JSON/YAML data source content survives HTML, PDF, DOCX, PPTX, Markdown bundle text, and Markdown bundle manifest outputs. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after extending front matter data source support. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the front matter data source update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the front matter data source update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the front matter data source update. |
| `cargo test --locked` in `src-tauri` | Pass | 164 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional document variable filter verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_formats_document_variables_and_reports_bad_filters --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves document variable defaults, chained text filters, numeric filters, unsupported-filter diagnostics, nonnumeric-filter diagnostics, and source-ranged warnings. |
| `cargo test --locked formatted_document_variables_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export-conformance test proves formatted document variables survive HTML, PDF, DOCX, PPTX, and Markdown bundle text outputs. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "manages front matter data sources" --project chromium` | Pass | Focused browser workflow now proves the References-panel variable manager surfaces nested YAML metadata as dotted paths such as `account.ownerName` and `account.renewalValue`, then inserts a filtered `{{account.ownerName \| title}}` placeholder. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after extending document variable filters. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the document variable filter update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the document variable filter update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the document variable filter update. |
| `cargo test --locked` in `src-tauri` | Pass | 166 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional malformed front matter verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_malformed_front_matter_with_source_ranges --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler test proves invalid YAML front matter and non-mapping YAML front matter produce source-ranged diagnostics with actionable suggestions. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after tightening front matter diagnostics. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the front matter diagnostic update. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the front matter diagnostic update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the front matter diagnostic update. |
| `cargo test --locked` in `src-tauri` | Pass | 167 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the backend/documentation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend/documentation update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional snapshot restore hardening verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked snapshot_restore --lib -- --nocapture` in `src-tauri` | Pass | Focused snapshot tests prove restore succeeds for matching active-document metadata and rejects out-of-store snapshots, non-Markdown restore targets, and snapshots whose source metadata belongs to another document. |
| `cargo test --locked spec_25_4_ipc_commands_are_registered_and_documented --lib -- --nocapture` in `src-tauri` | Pass | IPC command coverage stayed synchronized after changing `restore_snapshot` to a typed restore request. |
| `cargo fmt --check` in `src-tauri` | Pass | Formatting remained clean after the snapshot restore hardening. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the snapshot restore hardening. |
| `cargo check --locked --features native-watch` in `src-tauri` | Pass | Native watcher feature still compiles after the snapshot restore hardening. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | No clippy warnings after the snapshot restore hardening. |
| `cargo test --locked` in `src-tauri` | Pass | 169 Rust tests passed; 0 failed. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the store restore request update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the store restore request update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 35 Chromium workflow tests. Full browser execution remains host-limited in this workspace. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary built at `src-tauri/target/release/neditor`, proving the updated Tauri command shape compiles in the desktop app. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional versioning workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the release badge, Versioning-panel snapshot controls, and browser workflow mock coverage. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests still passed after the versioning workflow UI and mock harness update. |
| `pnpm run check:docs` | Pass | 13 Markdown files had all local links resolve after documenting the versioning workflow update. |
| `pnpm run check:a11y` | Pass | Vue template accessibility guardrails still passed after adding the release status badge and Versioning-panel snapshot controls. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 36 Chromium workflow tests, including `runs snapshot restore and release tagging workflows`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "snapshot restore" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional include graph navigation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding References-sidebar include graph navigation and command-palette include entries. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the include navigation changes. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding the include graph region and navigation buttons. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 37 Chromium workflow tests, including `navigates include graph entries from references and commands`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "include graph" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. Attempting `pnpm exec playwright install chromium` with escalation was rejected before the command ran. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting remained clean; this slice did not edit Rust sources. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional preview diagnostic rendering verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding source-map-backed preview diagnostic callouts and source-jump delegation. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the preview diagnostic rendering change. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding generated preview diagnostic callouts. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 37 Chromium workflow tests; `navigates compiler diagnostics to the source range` now contains preview diagnostic assertions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "diagnostics to the source range" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional export and transform preview verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the export preview summary and transform artifact preview inventory. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the export/transform preview change. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding export/transform preview regions. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 37 Chromium workflow tests; `runs export readiness, success, and failure workflows` now contains export preview and transform artifact assertions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "export readiness" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional AI review governance verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after extending the browser harness for AI review-state toggles and readiness warnings. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the AI governance workflow proof update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the workflow proof update. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 38 Chromium workflow tests, including `toggles AI review state and clears provenance readiness warnings`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "AI review state" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional index and glossary manager verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding References-sidebar Index and Glossary manager controls. |
| `pnpm run test:unit` | Pass | 11 frontend unit tests passed after the index/glossary manager update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked and all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding manager regions and controls. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery lists 38 Chromium workflow tests; the citation/glossary/index workflow now contains manager assertions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "citation glossary" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional external conflict composition verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the conflict merge composition tray and controls. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including merge composition ordering, duplicate prevention, blank-line preservation, remove, and reorder helpers. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding labeled merge composition controls. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests; the external conflict merge workflow now contains merge composition add/reorder/remove assertions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "merges external conflict" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional target-specific external release readiness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_target_specific_release_metadata_blockers --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust readiness proof blocks PPTX, blog, Substack, and Google Docs export for in-review documents missing approver/reviewer, `approvedAt`, `owner`, and `releaseTarget`, records target-specific related context, copies readiness into the manifest, and keeps the same source ready for PDF. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | Export command tests passed after expanding target-specific release metadata blockers across PPTX, blog, Substack, and Google Docs. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the export readiness update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after aligning the browser export workflow mock with the PPTX approved-metadata blocker. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests still pass after the export workflow update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the readiness matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the export workflow update. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests; the export readiness workflow now exercises the PPTX approved-metadata blocker instead of a synthetic token. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "export readiness" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional direct-export target extension verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked export_document_blocks_target_extension_mismatches_before_writing --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust export proof blocks a `target: "pptx"` export to `board-deck.pdf`, returns a target-extension validation error, and writes neither artifact nor sidecar manifest. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 17 export command tests passed after adding pre-write target/output extension validation. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the extension validation update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding output extension validation. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional target-specific option audit verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_target_specific_option_info --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust readiness proof reports non-blocking info diagnostics for `includeAgenda` on PDF and render-only options on Markdown bundle exports, keeps readiness true, and copies info counts into manifest readiness. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 18 export command tests passed after adding target-specific option info diagnostics. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after adding target-specific option audits. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding target-specific option audits. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional generated reference-section readiness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_empty_generated_reference_sections --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust readiness proof reports warnings when `index: true` and `glossarySection: true` request generated sections without any index terms or glossary entries, and copies both diagnostics into manifest readiness. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 19 export command tests passed after adding generated index/glossary readiness diagnostics. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 173 Rust library tests passed, covering compiler, export, transform, table, citation, validation, performance, and file command paths after the shared validation update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after adding generated reference-section validation. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after wiring generated section request detection into compiler validation. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional disabled external-engine verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_skips_disabled_external_transform_without_trust_warning --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust compiler proof skips a configured-but-disabled DOT external engine before trust/path execution, falls back to the embedded renderer, and avoids external trust-failure diagnostics. |
| `cargo test --locked prepare_for_export_validates_transform_engine_options --lib -- --nocapture` in `src-tauri` | Pass | Focused readiness proof now validates `disabledTransformEngines` as a boolean map alongside engine paths, trust, input mode, and timeout settings. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 10 external transform tests passed, including trust gating, adapter invocation, disabled-engine fallback, non-executable paths, timeout, stderr, cache invalidation, and installed-engine conformance. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 19 export command tests passed after adding disabled-engine option validation. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 174 Rust library tests passed after adding the disabled external-engine setting and compiler fallback behavior. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the disabled-engine option update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding disabled external-engine option plumbing. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the Settings UI toggle and store state. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after adding disabled external-engine persistence migration coverage. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after adding the disabled-engine checkbox. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the transform setup doc, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests, including the external transform settings workflow. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "external transform engine" --project chromium` | Blocked locally | Playwright could not launch because the local Chromium headless-shell executable is missing from the Playwright cache; the focused external transform settings workflow was discovered but did not execute assertions. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional Vega-Lite native subset verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked vega_lite_area_mark_renders_static_svg_preview --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust transform proof renders native static Vega-Lite `area` marks as SVG polygons with no diagnostics. |
| `cargo test --locked vega_lite_unsupported_marks_report_supported_static_subset --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust transform proof reports unsupported Vega-Lite marks with a suggestion naming the supported native subset: bar, line, point, or area. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 30 transform tests passed after expanding the Vega-Lite native static renderer. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the visual-data renderer update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding native Vega-Lite area rendering. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs and backend transform update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting native visual-data transform subsets; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz/DOT, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional Graphviz variant external-engine verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked transform_registry_covers_required_first_release_transforms --lib -- --nocapture` in `src-tauri` | Pass | Focused registry proof includes `dot`, `graphviz`, `circo`, `neato`, `fdp`, `osage`, and `twopi` as supported first-release transforms. |
| `cargo test --locked external_diagram_fallbacks_render_simple_native_svgs --lib -- --nocapture` in `src-tauri` | Pass | Native fallback proof renders a Graphviz variant fence as static SVG when no trusted executable is configured. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib -- --nocapture` in `src-tauri` | Pass | Adapter metadata proof exposes Graphviz variant default commands, version probes, setup hints, and no-shell `-Tsvg` adapter profiles. |
| `cargo test --locked compiler_uses_trusted_graphviz_variant_transform_preferences --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof routes a trusted `neato` transform through external execution with the configured executable path and transform artifact metadata. |
| `cargo test --locked external_transform_tests --lib -- --nocapture` in `src-tauri` | Pass | 11 external transform tests passed; installed-engine conformance verified `dot`, `circo`, `neato`, `fdp`, `osage`, `twopi`, D2, and PlantUML, with Pikchr skipped because it is not installed. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 31 transform tests passed after adding Graphviz variants to the registry and native fallback surface. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 177 Rust library tests passed after widening Graphviz transform support. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the Graphviz variant update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after widening external transform support. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding Graphviz variant probe settings. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the transform settings and docs update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the Settings transform list expansion. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating external transform docs, matrix, TODO, platform evidence, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 reports Graphviz `dot`, `circo`, `neato`, `fdp`, `osage`, `twopi`, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests, including the external transform settings workflow. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional PlantUML PNG output verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib -- --nocapture` in `src-tauri` | Pass | Adapter proof covers PlantUML SVG sidecars and PNG sidecars, including `-tpng`, sidecar `png` output, `output_format: png`, and PNG data URL embedding. |
| `cargo test --locked compiler_uses_plantuml_png_fence_output_format --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof routes a trusted ````plantuml format=png```` fence through file-mode external execution and emits a PNG transform artifact. |
| `cargo test --locked external_transform_tests --lib -- --nocapture` in `src-tauri` | Pass | 12 external transform tests passed after adding PlantUML PNG output selection; installed-engine conformance still verified Graphviz variants, D2, and PlantUML SVG with Pikchr skipped because it is not installed. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 32 transform tests passed after adding PlantUML PNG fence support. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 178 Rust library tests passed after adding PlantUML PNG output selection. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the PlantUML PNG update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding external transform output-format plumbing. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the transform and docs update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the docs and transform update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the PlantUML PNG output update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating external transform docs, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Additional visual-data map geometry verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked geojson_transform_preserves_geometry_types_in_static_svg_preview --lib -- --nocapture` in `src-tauri` | Pass | Focused GeoJSON proof renders FeatureCollection Polygon and MultiPoint inputs as separate static SVG polygon and point shapes. |
| `cargo test --locked topojson_transform_resolves_object_arc_references --lib -- --nocapture` in `src-tauri` | Pass | Focused TopoJSON proof resolves object-level Polygon arc references, including a reversed arc reference, into a static SVG polygon. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests passed after replacing flattened GeoJSON/TopoJSON coordinate previews with typed map geometry rendering. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 180 Rust library tests passed after the visual-data map geometry update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the visual-data renderer update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding typed GeoJSON and TopoJSON rendering helpers. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed after replacing flattened map previews with typed geometry rendering. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the visual-data and docs update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the docs and renderer update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the visual-data renderer update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating visual-data docs, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Safe business transform completion verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked business_workflow_transforms_render_static_html --lib -- --nocapture` in `src-tauri` | Pass | Focused transform proof covers roadmap metadata, ADR key classes, diff summaries, and static HTML classes for safe business workflow transforms. |
| `cargo test --locked safe_business_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof covers roadmap, ADR, diff, and QR transform artifacts, source ranges, output hashes, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests passed after improving first-release native business transforms. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 13 export conformance tests passed after adding safe business transform cross-target artifact proof. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 181 Rust library tests passed after the safe business transform and export evidence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the renderer and test updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the business transform renderer update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs and renderer update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the safe business transform update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting safe first-release and deferred second-wave transforms; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Visual/data transform export-fidelity verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked visual_data_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof covers chart, Vega-Lite, GeoJSON, TopoJSON, STL, and timeline static SVG artifacts, output hashes, source ranges, direct export-manifest entries, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 14 export conformance tests passed after adding the visual/data transform export fixture. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests still pass after adding export-fidelity coverage for visual/data transforms. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 182 Rust library tests passed after the visual/data transform export evidence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the export conformance fixture update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the visual/data transform export fixture update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the matrix/log/test update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the visual/data transform export evidence update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

BibTeX transform metadata/export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked bibtex_transform_renders_bibliography_preview --lib -- --nocapture` in `src-tauri` | Pass | Focused transform proof covers inline BibTeX title, author, year, and date metadata parsing plus richer transform preview rendering. |
| `cargo test --locked bibtex_transform_survives_cross_target_exports_with_metadata --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof covers BibTeX transform artifacts, source ranges, output hashes, bibliography metadata, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests passed after hardening inline BibTeX field parsing and preview output. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 14 citation tests passed after the BibTeX parser update. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 15 export conformance tests passed after adding the BibTeX transform export fixture. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 183 Rust library tests passed after the BibTeX transform metadata/export evidence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the parser, renderer, and test updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the BibTeX parser and renderer update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs/parser/test update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the BibTeX transform evidence update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating bibliography docs, matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

OpenAPI and JSON Schema export-fidelity verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked api_schema_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof covers OpenAPI and JSON Schema transform artifacts, output hashes, source ranges, semantic table caption preservation, HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests passed after adding API/schema transform proof. |
| `cargo test --locked document_structure_tests --lib` in `src-tauri` | Pass | 18 document-structure tests passed after preserving transform table captions through the semantic AST/export path. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests still pass after adding API/schema export proof. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 184 Rust library tests passed after the API/schema transform export evidence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the AST and export fixture updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the AST and export fixture updates. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the matrix/log/test update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the API/schema transform evidence update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

OpenAPI and JSON Schema semantic-depth verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_openapi_and_json_schema_tables --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof now covers OpenAPI global security requirements, deprecated operations, external docs, response media examples, response headers, links, security schemes, plus JSON Schema pattern properties, tuple `prefixItems`, dependent required/dependent schemas, object `additionalProperties`, `oneOf`, refs, and numeric constraints. |
| `cargo test --locked api_schema_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof carries the richer OpenAPI security/header/link/example rows and JSON Schema composition/object-keyword rows through HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked document_structure_tests --lib` in `src-tauri` | Pass | 18 document-structure tests passed after widening API/schema semantic rendering. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests passed after widening API/schema export artifact assertions. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests still pass after the API/schema renderer expansion. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 184 Rust library tests passed after the API/schema semantic-depth update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the renderer and fixture updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the renderer and fixture updates. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs and renderer update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the API/schema semantic-depth update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating transform docs, matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

OpenAPI callbacks/webhooks and JSON Schema dialect verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_openapi_and_json_schema_tables --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof now covers OpenAPI callbacks, webhooks, discriminator summaries and mappings, plus JSON Schema conditional branches and `$defs` rows. |
| `cargo test --locked api_schema_transforms_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Focused export proof carries callback, webhook, discriminator, conditional, and `$defs` evidence through HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked document_structure_tests --lib` in `src-tauri` | Pass | 18 document-structure tests passed after adding OpenAPI callback/webhook/discriminator and JSON Schema dialect coverage. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests passed after widening API/schema artifact assertions. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 34 transform tests still pass after the renderer update. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 184 Rust library tests passed after the callback/webhook/schema-dialect update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the renderer and fixture updates. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the renderer and fixture updates. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs and renderer update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the API/schema dialect update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating transform docs, matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Direct export sidecar manifest verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked export_document_writes_optional_sidecar_manifest --lib -- --nocapture` in `src-tauri` | Pass | Focused direct-export proof covers HTML, PDF, DOCX, PPTX, and Markdown bundle sidecar manifests, structured sidecar/response manifest equality, exact output paths, actual output SHA-256 hashes, readiness, and progress-step parity. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 19 export-command tests passed after hardening direct sidecar manifest parity assertions. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 184 Rust library tests passed after the direct export sidecar manifest proof update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the export command test update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the export command test update. |
| `cargo clippy --locked --all-targets -- -D warnings` in `src-tauri` | Pass | Rust static analysis passed with no warnings. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the docs/test update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the direct sidecar manifest evidence update. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails passed after the update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm run check:engines` | Partial pass | Darwin arm64 still reports Graphviz variants, D2, and PlantUML installed; Pikchr remains a missing optional engine. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 38 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors in the slice. |

Keyboard skip-link accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify accessible controls, dialog labeling, and required skip-link targets for commands, workspace, sidebar, source, preview, and status. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding skip-link focus handling and target landmarks. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the accessibility update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery now lists 39 Chromium workflow tests, including `exposes keyboard skip links to primary workbench regions`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "skip links" --project chromium` | Blocked locally | The Vite server started, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence used. |

Modal focus accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify accessible controls, dialog labels/modal state, dialog focusability/keyboard handling, and required skip-link targets. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding modal focus/Tab/Escape handling. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the modal focus update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery now lists 40 Chromium workflow tests, including `manages modal focus and Escape return paths`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "modal focus" --project chromium` | Blocked locally | The Vite server started, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence used. |

Status/progress live-region accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify accessible controls, dialog focus handling, required skip-link targets, and status/progress live regions. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding labeled status, watch, compile, export, and error live regions. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the status/progress live-region update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery now lists 41 Chromium workflow tests, including `exposes status and progress messages as live regions`. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "status and progress" --project chromium` | Blocked locally | The Vite server started, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence used. |

Diagnostic/conflict screen-reader accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify diagnostic article labels, named diagnostic lists, and conflict diff cell group labels. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding diagnostic and conflict diff accessible names. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the diagnostic/conflict accessibility update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; diagnostic navigation and external conflict merge workflows now assert accessible list/listitem/group names. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "diagnostics\|merges external conflict" --project chromium` | Blocked locally | The Vite server started and selected the relevant workflows, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; the escalated browser install request was rejected by the auto-review layer, so no workaround was attempted and no GitHub Actions evidence was used. |

Table-editor screen-reader accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails now verify the table editor grid name, header/cell/total label helpers, and row/sort/move control group labels. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding table editor grid/action/total labels. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the table editor accessibility update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; the table workflows now assert table grid/action/total accessible names. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "pasted tables\|table structure" --project chromium` | Blocked locally | The Vite server started and selected the two table workflows, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence was used. |
| `git diff --check` | Pass | No whitespace errors after the diagnostic/conflict/table accessibility updates. |

Desktop artifact smoke verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build created the frontend artifact consumed by the Tauri build. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release desktop binary built at `src-tauri/target/release/neditor`. |
| `pnpm run test:desktop-smoke` | Pass | Checked `dist/index.html`, bundled JS assets, Tauri product/config/license metadata, npm/Cargo MIT metadata, and the executable release binary. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including the package-script guard for `test:desktop-smoke`. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the desktop smoke harness; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after adding the desktop smoke script and docs. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Blocked locally | Bounded GUI launch smoke requires elevated desktop app launch permission; the escalation request was rejected by the auto-review layer, so no workaround was attempted. |
| `git diff --check` | Pass | No whitespace errors after the desktop smoke harness update. |

High-contrast/reduced-motion accessibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue/CSS accessibility guardrails now verify high-contrast shell/control/focus styles and reduced-motion data/media-query styles. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding high-contrast focus CSS and settings workflow assertions. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the accessibility guard update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; the settings workflow now asserts high-contrast computed colors and reduced-motion zero-duration transitions. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "persists editor settings" --project chromium` | Blocked locally | The Vite server started and selected the settings workflow, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence was used. |
| `git diff --check` | Pass | No whitespace errors after the high-contrast/reduced-motion accessibility update. |

Editor/preview screen-reader surface verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static Vue accessibility guardrails now verify the Markdown source and live preview region labels, the focusable `role=document` preview article, and CodeMirror `role=textbox`/`aria-label`/`aria-multiline` content attributes. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding editor and preview document accessibility semantics. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the editor/preview accessibility update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating matrix, TODO, and progress logs; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; the boot workflow now asserts the named Markdown editor textbox and rendered preview document. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts --grep "boots the workbench" --project chromium` | Blocked locally | The Vite server started and selected the boot workflow, but Playwright could not launch because the Chromium headless-shell executable is missing from `/Users/nyimbiodero/Library/Caches/ms-playwright/chromium_headless_shell-1223/...`; no GitHub Actions evidence was used. |
| `git diff --check` | Pass | No whitespace errors after the editor/preview accessibility update. |

Malformed reference marker validation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked malformed_reference_markers_are_reported_with_source_ranges --lib -- --nocapture` in `src-tauri` | Pass | Focused compiler proof rejects malformed heading labels, figure labels, empty cross references, slash-containing keys, and unclosed cross-reference markers with source-ranged errors while preserving generated heading anchors. |
| `cargo test --locked prepare_for_export_blocks_malformed_reference_markers_in_manifest --lib -- --nocapture` in `src-tauri` | Pass | Focused readiness proof blocks export for malformed labels/references and copies the source-ranged diagnostics into the export manifest readiness evidence. |
| `cargo test --locked duplicate_reference_labels_are_reported_with_source_ranges --lib -- --nocapture` in `src-tauri` | Pass | Existing duplicate-label regression still reports the duplicate anchor with first/duplicate origin context after tightening label parsing. |
| `cargo test --locked prepare_for_export_blocks_duplicate_reference_labels_in_manifest --lib -- --nocapture` in `src-tauri` | Pass | Existing export-readiness duplicate-label regression still copies duplicate-anchor diagnostics into the manifest. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 190 Rust library tests passed after strict reference marker validation and the shared label extractor fix for labels with attributes. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the reference validation update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the compiler/reference validation update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend and docs update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the reference validation slice. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting strict reference key rules; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the reference validation slice. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after the reference validation update. |

Bibliography import compatibility verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_parses_better_bibtex_edge_cases_without_splitting_at_symbols --lib -- --nocapture` in `src-tauri` | Pass | Focused BibTeX proof skips `@string`/`@comment` metadata records, handles parenthesis-delimited entries, dotted keys, protected title braces, locators, and `@` characters inside field values without splitting the entry. |
| `cargo test --locked compiler_loads_csl_json_object_variants_with_full_author_dates --lib -- --nocapture` in `src-tauri` | Pass | Focused CSL JSON proof accepts wrapper objects with `items`, full given/family authors, literal authors, raw date strings, string `date-parts`, and DOI-style slash keys. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 16 citation tests passed after extending BibTeX/CSL import compatibility and citation key parsing. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 192 Rust library tests passed after the bibliography parser update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the bibliography parser update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the bibliography parser update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the bibliography/parser docs update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the bibliography/parser update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting expanded BibTeX/CSL compatibility; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the bibliography/parser update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the bibliography/parser update. |

CSL alias rendering verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_maps_common_csl_style_aliases_to_native_renderers --lib -- --nocapture` in `src-tauri` | Pass | Focused citation proof maps `apa` to author-year output and `ieee` to numeric output without unsupported-style warnings. |
| `cargo test --locked compile_options_supply_csl_alias_default_citation_style --lib -- --nocapture` in `src-tauri` | Pass | Compile-option defaults now accept supported CSL aliases such as `apa` and render through the native author-year path. |
| `cargo test --locked compiler_warns_and_falls_back_for_unsupported_csl_style --lib -- --nocapture` in `src-tauri` | Pass | Unknown CSL style names still warn and fall back to title rendering. |
| `cargo test --locked prepare_for_export_validates_brand_and_default_style_options --lib -- --nocapture` in `src-tauri` | Pass | Export readiness accepts supported CSL aliases as default citation styles and still blocks unknown default styles before writing. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 18 citation tests passed after adding deterministic CSL alias rendering. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 23 export command tests passed after updating default citation-style validation for CSL aliases. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 194 Rust library tests passed after adding deterministic CSL alias support. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the CSL alias update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the CSL alias update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the CSL alias docs/update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the CSL alias update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting CSL aliases; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the CSL alias update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the CSL alias update. |

Broader equation syntax verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_broader_latex_equation_syntax --lib -- --nocapture` in `src-tauri` | Pass | Focused equation proof renders sums, superscripts, subscripts, approximation/infinity/arrows, Omega, and a `bmatrix` environment, preserves AST equation metadata, emits HTML/export CSS, and keeps DOCX/PPTX text evidence. |
| `cargo test --locked compiler_renders_block_and_inline_equations --lib -- --nocapture` in `src-tauri` | Pass | Existing equation regression still proves inline/display math, fractions, captions, labels, references, AST capture, and readiness cleanliness. |
| `cargo test --locked captioned_equations_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Existing cross-target equation regression still proves captioned equation text and references across HTML, PDF, DOCX, PPTX, and Markdown bundle outputs. |
| `cargo test --locked --lib` in `src-tauri` | Pass | 195 Rust library tests passed after adding the broader native equation syntax coverage. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the equation renderer update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the equation renderer update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the preview/export math styling update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed after the equation renderer update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the broader equation subset; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the math preview styling update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the equation renderer update. |

Dependency admission guard verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:deps` | Pass | Checked dependency admission records for 39 manifest dependencies across `package.json` and `src-tauri/Cargo.toml`, and verified project MIT license metadata. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the dependency admission guard; all local links resolved. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including the package-script guard for `check:deps`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the dependency admission command. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after the dependency admission update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the dependency admission guard update. |

Citation alias preference verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed; workspace persistence now preserves supported CSL aliases such as `APA` as `apa` and still falls back to `title` for unknown styles. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding CSL alias options to References and Settings selectors. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting citation alias preference support; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after adding citation alias options. |
| `pnpm run check:deps` | Pass | Dependency admission guard still passes after the citation preference update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the citation alias preference update. |

Project structure guard verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:structure` | Pass | Verified `src/App.vue` keeps the required top-level `template`, `<script setup>`, and `style` block order. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including the package-script guard for `check:structure`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the project structure guard. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the structure guard; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue template accessibility guardrails still pass after adding the structure guard. |
| `pnpm run check:deps` | Pass | Dependency admission guard still passes after adding the structure guard script. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the structure guard update. |

Non-goal documentation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after adding first-release non-goals to the README and user guide; all local links resolved. |

Markdown bundle manifest sidecar verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_reports_markdown_bundle_manifest_sidecar_info --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust readiness proof reports `includeManifest=false` on Markdown bundles as a non-blocking info diagnostic, keeps readiness true, and shows progress as embedded bundle-manifest work instead of sidecar output. |
| `cargo test --locked export_document_writes_optional_sidecar_manifest --lib -- --nocapture` in `src-tauri` | Pass | Direct export proof now verifies Markdown bundles with sidecar manifests disabled still write a ZIP containing `manifest.json`, suppress the sidecar path, keep final output path/hash evidence in the response manifest, and record embedded-manifest progress. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 24 export command tests passed after adding Markdown bundle sidecar/embedded manifest distinction coverage. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the Markdown bundle manifest update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the Markdown bundle manifest update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the user guide, matrix, TODO, and progress log; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests; full browser execution remains dependent on a locally installed Playwright Chromium. |
| `git diff --check` | Pass | No whitespace errors after the Markdown bundle manifest update. |

Browser workflow environment preflight verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `PLAYWRIGHT_BROWSERS_PATH=.tmp/ms-playwright pnpm exec playwright install chromium` | Pass | Installed/confirmed workspace-local Playwright Chromium without touching tracked files. |
| `pnpm run check:e2e-env` | Pass | Project-local Playwright Chromium launch preflight passed on this host. |
| `pnpm run test:e2e` | Pass | Full browser workflow execution passed all 41 Chromium tests locally. |
| `node --check scripts/check-e2e-environment.mjs` | Pass | The E2E environment preflight script parses successfully. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests passed, including the package-script guard for `check:e2e-env`. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding the preflight script. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the preflight script; all local links resolved. |
| `pnpm run check:deps` | Pass | Dependency admission guard still passes after adding the package script. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after the E2E preflight update. |

Enabled export option matrix verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked enabled_export_option_matrix_survives_cross_target_artifacts --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust artifact proof covers enabled styles, syntax highlighting, cover pages, page numbers, glossary/comments/provenance appendices, PPTX agenda, presentation layout preset, watermark, text output, and Markdown bundle manifest evidence across HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked export_option_tests --lib` in `src-tauri` | Pass | 10 export option tests passed after adding the enabled cross-target option matrix fixture. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests still passed after adding the enabled option matrix proof. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the enabled option matrix fixture. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the enabled option matrix fixture. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after the enabled option matrix fixture. |

Extended LaTeX equation notation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_extended_latex_equation_notation --lib -- --nocapture` in `src-tauri` | Pass | Focused Rust proof covers optional-index roots, `\text{}` labels, overline/underline/hat/vector wrappers, left/right delimiters, set/logic symbols, ellipses, preview/export CSS, AST capture, and DOCX/PPTX text evidence. |
| `cargo test --locked document_structure_tests --lib` in `src-tauri` | Pass | 22 document structure tests passed after extending native LaTeX notation rendering. |
| `cargo test --locked captioned_equations_survive_cross_target_exports --lib -- --nocapture` in `src-tauri` | Pass | Captioned equation export conformance still passes across target artifacts. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the extended LaTeX renderer update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the extended LaTeX renderer update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after adding preview CSS for the new math notation wrappers. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue accessibility guardrails still pass after the preview CSS update. |
| `pnpm run check:structure` | Pass | The Vue single-file component structure guard still passes after the preview CSS update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests still pass after the preview CSS update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests. |
| `git diff --check` | Pass | No whitespace errors after the extended LaTeX renderer update. |

Piecewise and styled LaTeX equation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked compiler_renders_piecewise_and_styled_latex_equations --lib` | Pass | Focused compiler proof covers piecewise `cases` equations, blackboard/calligraphic/roman identifiers, probability and limit operators, equation captions/references, preview/export CSS, AST capture, and DOCX/PPTX text fallback evidence. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting is clean after the piecewise/styled LaTeX renderer update. |
| `pnpm run check:docs` | Pass | 15 Markdown files were checked after updating the equation docs, matrix, TODO, and progress log; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with the current equation evidence recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the equation renderer update. |
| `git diff --check` | Pass | No whitespace errors after the piecewise/styled LaTeX renderer update. |

Agentic step assistance accept-and-replan verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after making Agent Workspace step assistance accept and immediately replan from the appended suggested answer, including proof that the accepted answer becomes next-plan context and that replanned steps retain context-aware assistance. |
| `pnpm run check:docs` | Pass | 15 Markdown files were checked after updating the step-assistance evidence; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with the accept-and-replan proof recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the Agent Workspace step-assistance update. |
| `git diff --check` | Pass | No whitespace errors after the Agent Workspace step-assistance update. |

Docs Live suggested-answer rationale verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding rationale and context signals to Docs Live suggested questionnaire answers, including static UI proof and typed helper coverage. |
| `pnpm run check` | Pass | Vue typecheck passed after rendering Docs Live suggestion rationale and context-signal lists. |
| `pnpm run check:docs` | Pass | Markdown docs were checked after documenting rationale-visible Docs Live suggested answers; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with Docs Live suggestion rationale evidence recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the Docs Live suggested-answer rationale update. |
| `git diff --check` | Pass | No whitespace errors after the Docs Live suggested-answer rationale update. |

Business wizard step-assistance verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding business wizard step assistance, including direct proof that a tender builder gets six context-aware suggested answers, procurement QA guidance, brand-voice humanization guidance, Docs Live context evidence, and Templates UI wiring. |
| `pnpm run check` | Pass | Vue typecheck passed after wiring business wizard step assistance into the Templates panel. |
| `pnpm run check:docs` | Pass | 15 Markdown files were checked after documenting business wizard step assistance; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with business wizard step-assistance evidence recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the business wizard step-assistance update. |
| `git diff --check` | Pass | No whitespace errors after the business wizard step-assistance update. |

RFP suggested response verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding context-aware suggested responses to every RFP compliance row, including matrix columns, full response draft text, verification checklist entries, UI wiring, and local-agent handoff context. |
| `pnpm run check` | Pass | Vue typecheck passed after adding suggested RFP response answers to the RFP analysis UI. |
| `pnpm run check:docs` | Pass | 15 Markdown files were checked after documenting RFP suggested response answers; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with RFP suggested response evidence recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the RFP suggested response update. |
| `git diff --check` | Pass | No whitespace errors after the RFP suggested response update. |

RFP requirement response draft verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding the grouped Requirement Response Drafts section to full RFP response output, including evidence owner and proof-needed prompts for each requirement answer. |
| `pnpm run check` | Pass | Vue typecheck passed after wiring the grouped RFP requirement response drafts into generated Markdown. |
| `pnpm run check:docs` | Pass | Markdown docs were checked after documenting grouped RFP requirement response drafts; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with grouped RFP response draft evidence recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the grouped RFP response draft update. |
| `git diff --check` | Pass | No whitespace errors after the grouped RFP response draft update. |

RFP wizard step-assistance verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding native RFP wizard step assistance, editable response-context notes, generated-response note carry-through, and static UI wiring. |
| `pnpm run check` | Pass | Vue typecheck passed after rendering RFP step assistance and response-context note acceptance controls. |
| `pnpm run check:docs` | Pass | Markdown docs were checked after documenting RFP wizard step assistance; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with RFP wizard step-assistance evidence recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the RFP wizard step-assistance update. |
| `git diff --check` | Pass | No whitespace errors after the RFP wizard step-assistance update. |

AI paste code fence cleanup verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked ai_cleanup_tests --lib` in `src-tauri` | Pass | 11 AI cleanup tests passed after adding code-fence normalization and rich HTML `<pre><code>` conversion coverage. |
| `cargo test --locked --lib` in `src-tauri` | Pass | Full Rust library suite passed with 200 tests after the AI cleanup code-fence update. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the AI cleanup code-fence update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the AI cleanup code-fence update. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after the backend AI cleanup update and documentation changes. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue accessibility guardrails still pass after the backend AI cleanup update. |
| `pnpm run check:structure` | Pass | The project structure guard still passes after the backend AI cleanup update. |
| `pnpm run test:unit` | Pass | 12 frontend unit tests still pass after the backend AI cleanup update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests, including the existing AI paste workflows. |
| `git diff --check` | Pass | No whitespace errors after the AI cleanup code-fence update. |

Smart Markdown list continuation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 13 frontend unit tests passed after extracting Markdown list continuation logic and covering bullets, numbered lists, task lists, blockquoted lists, quoted numbered lists, empty-marker exit behavior, and plain-paragraph no-op behavior. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after wiring the extracted Markdown list continuation helper into the CodeMirror Enter key handler. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue accessibility guardrails still pass after the editor continuation helper extraction. |
| `pnpm run check:structure` | Pass | The project structure guard still passes after adding `src/lib/markdownEditing.ts`. |
| `cargo check --locked` in `src-tauri` | Pass | Backend dev-profile check remains clean after the frontend editor ergonomics update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests, including the existing editor ergonomics workflow. |
| `git diff --check` | Pass | No whitespace errors after the smart Markdown list continuation update. |

Table editor typed sorting verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 14 frontend unit tests passed after extracting table draft sorting and covering text, currency, and date ordering while retaining summary/formula rows at the bottom. |
| `pnpm run build` | Pass | `vue-tsc --noEmit` and Vite production build passed after wiring the extracted table draft sorting helper into the table editor. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `pnpm run check:a11y` | Pass | Static Vue accessibility guardrails still pass after the table sorting helper extraction. |
| `pnpm run check:structure` | Pass | The project structure guard still passes after the table sorting helper extraction. |
| `cargo check --locked` in `src-tauri` | Pass | Backend dev-profile check remains clean after the frontend table editor update. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery still lists 41 Chromium workflow tests, including the existing table editor workflows. |
| `git diff --check` | Pass | No whitespace errors after the table editor typed sorting update. |

External transform engine path readiness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_validates_transform_engine --lib` in `src-tauri` | Pass | Two focused readiness tests passed, proving transform option shape checks plus real executable-path validation for configured external engines; missing files, directories, and non-executable paths fail readiness while disabled engine paths are ignored. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 25 export command tests passed after adding executable-path readiness validation. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the external engine path readiness update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the external engine path readiness update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the external engine path readiness update. |

Graphviz alias external transform settings verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_uses_dot_settings_for_graphviz_alias --lib` in `src-tauri` | Pass | Focused compiler proof confirms `graphviz` fences inherit the configured `dot` executable path, trust state, and stdin input mode, then execute through the external Graphviz adapter. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 13 external transform tests passed after the Graphviz alias settings fix, including adapter invocation, disabled fallback, cache behavior, PlantUML PNG, and installed-engine conformance. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the Graphviz alias settings update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the Graphviz alias settings update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the Graphviz alias settings update. |

Missing external transform engine path diagnostics verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_missing_external_engine_path_before_embedded_fallback --lib` in `src-tauri` | Pass | Focused compiler proof confirms an external-capable `dot` fence with no configured engine path emits a non-blocking setup diagnostic before using the embedded renderer. |
| `cargo test --locked external_transform_tests --lib` in `src-tauri` | Pass | 14 external transform tests passed after adding the missing-path diagnostic proof. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the missing external engine path diagnostic update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the missing external engine path diagnostic update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the missing external engine path diagnostic update. |

Transform registry alias metadata verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 37 transform tests passed after adding registry alias/output metadata plus canonical command and fenced-rendering support for `vegalite`, `jsonschema`, `schema`, `yml`, and `graph`. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the transform registry alias metadata update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the transform registry alias metadata update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating transform docs, matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the transform registry alias metadata update. |

Vega-Lite grouped-series static preview verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked vega_lite --lib` in `src-tauri` | Pass | Four Vega-Lite tests passed, including `encoding.color.field` grouped/series rendering with per-series marks and legends. |
| `cargo test --locked transform_tests --lib` in `src-tauri` | Pass | 38 transform tests passed after adding Vega-Lite grouped-series rendering. |
| `cargo test --locked visual_data_transforms_survive_cross_target_exports --lib` in `src-tauri` | Pass | Existing visual-data cross-target export proof remains clean for HTML, PDF, DOCX, PPTX, and Markdown bundle evidence. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the Vega-Lite grouped-series update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the Vega-Lite grouped-series update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating transform docs, matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the Vega-Lite grouped-series update. |

Citation preview popover verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked citation_export_conformance_covers_required_cases --lib` in `src-tauri` | Pass | Focused citation export proof passed after adding compiler-rendered citation detail attributes and preview/full-HTML popover CSS. |
| `cargo test --locked citation_tests --lib` in `src-tauri` | Pass | 18 citation tests passed, covering BibTeX/CSL/Hayagriva import, duplicate diagnostics, style aliases, numeric rendering, AST citation preservation, and the new citation popover evidence. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the citation popover update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the citation popover update. |
| `pnpm run check:a11y` | Pass | Static App.vue accessibility guardrails passed after adding focus-visible citation popovers. |
| `pnpm run build` | Pass | Vue typecheck and Vite production build passed after adding preview citation popover CSS. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating citation docs, matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the citation popover update. |

Citation manager repair workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 15 frontend unit tests passed, including new bibliography-manager helper proof for citation-key normalization, locator citation insertion, resolved BibTeX copy generation, and deduplicated missing-key stubs. |
| `pnpm run check:a11y` | Pass | Static App.vue accessibility guardrails passed after adding Citation manager actions in the References sidebar. |
| `pnpm run check:structure` | Pass | Project structure guardrails passed with the new bibliography manager helper module. |
| `pnpm run build` | Pass | Vue typecheck and Vite production build passed after wiring Citation manager actions into the workbench. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating citation manager docs, matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the citation manager update. |

Table text and Markdown bundle export verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked table_tests --lib` in `src-tauri` | Pass | 9 table tests passed after making text/Markdown bundle table exports preserve header rows, alignment separators, escaped pipes, formula output rows, and merged-cell span annotations. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests still pass after the shared table text export change. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the table text export update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the table text export update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the table text export update. |

Markdown bundle figure/media audit verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked media_export_tests --lib` in `src-tauri` | Pass | 7 media export tests passed after adding `media-uses.json`, proving local media source ranges, reused-media per-figure crop metadata, and include-relative duplicate media traceability in Markdown bundles. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests still pass after adding the Markdown bundle media-use map. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the bundle media-use update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the bundle media-use update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the bundle media-use update. |

Direct export sidecar-manifest readiness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 25 export command tests passed after making `includeManifest=false` visible for non-bundle targets; the focused readiness test now proves HTML exports report a non-blocking sidecar audit manifest diagnostic, and the direct export manifest test proves the sidecar file is suppressed while the diagnostic is retained. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the sidecar-manifest readiness update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the sidecar-manifest readiness update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the sidecar-manifest readiness update. |

Local verification runner verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run verify:local` | Pass | New quick local baseline passed all 9 steps: frontend typecheck, frontend unit tests, project structure, accessibility, dependency/license admission, Markdown links, Rust formatting, Rust dev check, and whitespace checks. |
| `pnpm run verify:local -- --list` | Pass | Printed the quick local baseline command plan without running it. |
| `pnpm run verify:local:full -- --list` | Pass | Printed the release-grade local baseline command plan, including build, optional engine probe, native-watch, clippy, full Rust tests, Tauri no-bundle compile, desktop artifact smoke, and desktop WebDriver smoke. |
| `pnpm run test:unit` | Pass | 15 frontend unit tests passed after adding package-script coverage for `verify:local` and `verify:local:full`. |
| `git diff --check` | Pass | No whitespace errors after adding the local verification runner. |

Front matter data-source boundary verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_core_tests --lib` in `src-tauri` | Pass | 24 compiler core tests passed after blocking unsafe front matter data-source imports. New coverage proves absolute paths, `..` parent-directory escapes, and Unix symlink escapes are rejected before reading files outside the document folder, while safe sibling data sources still render. |
| `cargo test --locked front_matter_data_sources_survive_cross_target_exports --lib` in `src-tauri` | Pass | Existing cross-target data-source export proof still passes for CSV, TSV, JSON, and YAML front matter data sources after the path-boundary guard. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the front matter data-source boundary update. |
| `pnpm run verify:local` | Pass | The quick local verification baseline passed after the data-source boundary update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the Markdown extension docs, matrix, TODO, and progress log; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the data-source boundary update. |

JSON Schema dialect transform verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_renders_openapi_and_json_schema_tables --lib` in `src-tauri` | Pass | Focused compiler proof now covers JSON Schema `propertyNames`, `contains` with `minContains`/`maxContains`, `unevaluatedItems`/`unevaluatedProperties`, content annotations, and nested `contentSchema` rows. |
| `cargo test --locked api_schema_transforms_survive_cross_target_exports --lib` in `src-tauri` | Pass | Focused cross-target proof carries the richer JSON Schema dialect rows and constraint summaries through HTML, PDF summary text, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the schema dialect update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the schema dialect update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating transform docs, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run verify:local` | Pass | The quick local verification baseline passed after the schema dialect update. |
| `git diff --check` | Pass | No whitespace errors after the schema dialect update. |

Nested JSON/YAML table transforms:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked structured_data_transforms_render_tables_and_trees --lib` | Pass | Focused transform coverage now proves top-level arrays, `{ "data": [...] }` JSON objects, and `records:` YAML objects with row arrays render as inspectable `transform-json`/`transform-yaml` tables with field captions instead of falling back to tree-only summaries. |

Full registered IPC command coverage verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked spec_25_4_ipc_commands_are_registered_and_documented --lib` in `src-tauri` | Pass | IPC coverage now proves the spec 25.4 command subset is registered and documented, the coverage table exactly matches every registered Tauri command, and each row has implementation/evidence columns. |
| `cargo test --locked git_history_diff_commit_tag_and_restore_workflow --lib` in `src-tauri` | Pass | Git workflow proof now directly exercises `get_git_status` before diff, commit, history, tag, and restore operations. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the IPC coverage audit update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the IPC coverage audit update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating IPC coverage docs, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run verify:local` | Pass | The quick local verification baseline passed after the IPC coverage audit update. |
| `git diff --check` | Pass | No whitespace errors after the IPC coverage audit update. |

Example project export-audience verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked example_project_fixtures_compile_and_export --lib` in `src-tauri` | Pass | Example fixture proof now keeps README example links synchronized with executable fixtures and proves each example's title, audience metadata, representative feature markers, package metadata, source hash custom properties, semantic metadata, source map, diagnostics, and Markdown bundle evidence across HTML, PDF, DOCX, PPTX, and Markdown bundle artifacts. |
| `cargo test --locked export_conformance_tests --lib` in `src-tauri` | Pass | 16 export conformance tests still pass after carrying `targetPersona` audience metadata into export metadata. |
| `cargo test --locked export_option_tests --lib` in `src-tauri` | Pass | 10 export option tests still pass after adding audience metadata to HTML, plain text, DOCX/PPTX custom properties, and bundle text. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the audience metadata export update. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after the audience metadata export update. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating README, user guide, Markdown extension docs, matrix, TODO, and progress log; all local links resolved. |
| `pnpm run verify:local` | Pass | The quick local verification baseline passed after the audience metadata export update. |
| `git diff --check` | Pass | No whitespace errors after the audience metadata export update. |

Accessibility report and browser-performance evidence verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run verify:local` | Pass | Quick local verification passed after making `pnpm run check:a11y` emit `.tmp/accessibility/report.json`; the report recorded 10 named checks, 10 passed, 0 failed, and 0 issues. |
| `pnpm run test:e2e` | Pass | 47 Chromium browser workflows passed locally through the system-Chrome fallback, including skip links, modal focus, status/progress live regions, desktop/narrow responsive workbench layout, fold/unfold controls, pending preview compile cancellation/resume, generated TOC preview/source navigation, accessible diagnostics/conflict/table/source/preview semantics, large-document edit/preview responsiveness, transform template management, and blog/Substack/LaTeX/Google Docs export handoffs. |

Publishing and Google Docs handoff workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 28 export command tests passed after adding machine-readable publish/import workflow metadata to blog, Substack, and Google Docs packages. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit passed with package assertions for blog/Substack publish workflow metadata, Google Docs import workflow metadata, LaTeX source checks, nested Google Docs DOCX package checks, macOS `textutil` extraction for primary/nested DOCX files, and `pdflatex` compile proof. |

Performance audit report verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:performance-audit` | Pass | Wrote `.tmp/performance-audit/report.json` with 2 passing checks: 4 Rust performance stress tests and the focused Chromium large-document edit/preview workflow. The first sandboxed browser attempt failed on macOS Mach port permissions, then the approved run passed outside the sandbox. |

Command bar UI polish verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after restructuring the command bar and adding the persisted toolbar display setting. |
| `pnpm run test:unit` | Pass | 17 frontend unit tests passed, including workspace migration for `toolbarDisplay` and a static guard proving the command bar exposes workflow groups, icon paths, and display controls. |
| `pnpm run check:a11y` | Pass | Static accessibility guardrails passed and rewrote `.tmp/accessibility/report.json`; command bar buttons keep accessible names through icon-only mode. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the grouped icon command bar and toolbar display preference; all local links resolved. |
| `pnpm run build` | Pass | Vue typecheck and Vite production build passed after the command-bar UI polish. |
| `pnpm run verify:local` | Pass | Quick local verification passed all 9 steps after the command-bar UI polish. |
| `git diff --check` | Pass | No whitespace errors after the command-bar UI polish. |

Editor ergonomics verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after enabling CodeMirror folding, persisted code-folding settings, and direct navigation controls. |
| `pnpm run test:unit` | Pass | 17 frontend unit tests passed, including workspace migration for `codeFolding` and static guards for folding plus the Navigate command-bar group. |
| `pnpm run check:a11y` | Pass | Static accessibility guardrails passed and rewrote `.tmp/accessibility/report.json` after adding search, outline, fold, and unfold controls. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting editor folding/navigation ergonomics; all local links resolved. |
| `pnpm run build` | Pass | Vue typecheck and Vite production build passed after the editor ergonomics update. |
| `pnpm run verify:local` | Pass | Quick local verification passed all 9 steps after the editor ergonomics update. |
| `git diff --check` | Pass | No whitespace errors after the editor ergonomics update. |

Transform template library verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding the Templates sidebar, template command actions, custom template editor, and workspace persistence. |
| `pnpm run test:unit` | Pass | 18 frontend unit tests passed, including built-in transform template breadth, custom template normalization, and persistence migration coverage. |
| `pnpm run check:a11y` | Pass | Static accessibility guardrails passed after adding the template filters, list, preview details, and custom editor controls. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting transform templates; all local links resolved. |
| `pnpm run build` | Pass | Vue typecheck and Vite production build passed after the transform template library update. |
| `pnpm run verify:local` | Pass | Quick local verification passed all 9 steps after the transform template update. |
| `git diff --check` | Pass | No whitespace errors after the transform template update. |

Browser workflow fallback verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:e2e-env` | Pass | Focused workbench boot workflow passed on this host by falling back from the missing Playwright bundled Chromium path to `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`; `.tmp/e2e-environment/report.json` records the browser source and command tail. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "modal focus\|syncs editor\|persists editor settings" --project chromium` | Pass | Escalated macOS run proved the default system-Chrome fallback can run the editor/modal/search workflow subset after restoring explicit command labels. |
| `node scripts/run-e2e.mjs` | Pass | That macOS run passed all 47 Chromium browser workflows with the default system-Chrome fallback; `.tmp/e2e-browser/report.json` recorded `source: system-chromium`, the Chrome executable path, and exit status 0. |
| `pnpm run check`, `pnpm run test:unit`, `pnpm run check:a11y` | Pass | Typecheck, 18 frontend unit tests, and static accessibility guardrails passed after restoring explicit Find, Open Folder, and Save Workspace command labels. |

Preview compile cancellation workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "cancels a pending preview compile" --project chromium` | Pass | Focused Chromium workflow passed on this host with the system-Chrome fallback, proving a delayed preview compile exposes the live Cancel compile action, records the cancellation status, hides the pending action, and resumes live preview updates after editing continues. |
| `node scripts/run-e2e.mjs` | Pass | Full Chromium browser workflow suite passed all 46 tests after adding pending preview compile cancellation/resume coverage. |

Generated TOC preview workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "renders generated table of contents" --project chromium` | Pass | Focused Chromium workflow passed on this host with the system-Chrome fallback, proving generated Table of Contents preview rendering, nested heading links, and link-to-source navigation for a generated TOC entry. |
| `node scripts/run-e2e.mjs` | Pass | Full Chromium browser workflow suite passed all 46 tests after adding generated TOC preview/source-navigation coverage. |

Markdown folding workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "folds and unfolds Markdown sections" --project chromium` | Pass | Focused Chromium workflow passed on this host with the system-Chrome fallback, proving toolbar and command-palette fold/unfold controls create and remove CodeMirror fold placeholders for Markdown content. |
| `node scripts/run-e2e.mjs` | Pass | Full Chromium browser workflow suite passed all 46 tests after adding Markdown fold/unfold coverage. |

Native desktop window smoke verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo check --locked` in `src-tauri` | Pass | Tauri setup hook for the desktop smoke report compiled cleanly. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Rebuilt the release desktop binary consumed by the launch smoke after adding the app-authored native window report. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Bounded macOS launch smoke verified the release binary, native command workflow, process survival, app-authored native window report, and app-authored native UI report. `.tmp/desktop-smoke/native-window-report.json` records package `NEditor`, identifier `com.neditor.desktop`, main-window title `NEditor`, visible `true`, size `2880x1840`, and scale factor `2`; `.tmp/desktop-smoke/native-ui-report.json` records rendered workbench commands, primary surfaces, document identity, preview label, status text, and viewport dimensions. |
| `node --check scripts/check-desktop-smoke.mjs`, `cargo fmt --check` in `src-tauri`, `pnpm run build`, `pnpm run check` | Pass | Script syntax, Rust formatting, production frontend build, and Vue typecheck passed after the desktop UI smoke hardening. |

Responsive layout workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "keeps primary workbench regions accessible" --project chromium` | Pass | Focused Chromium workflow passed on this host with the system-Chrome fallback, proving command, sidebar, source, preview, and status regions remain rendered across desktop and 390px narrow viewports, the narrow layout stacks the sidebar above editor/preview instead of hiding it, and the page does not introduce horizontal overflow. |
| `pnpm exec playwright test --list` | Pass | Browser harness discovery now lists 47 Chromium workflow tests after adding the responsive layout proof. |
| `node scripts/run-e2e.mjs` | Pass | Full Chromium browser workflow suite passed all 47 tests after adding responsive layout coverage. |

Transform template browser workflow verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "manages transform templates" --project chromium` | Pass | Focused Chromium workflow passed on this host with the system-Chrome fallback, proving the Templates panel filters the Science `calc` library, exposes fill values for the built-in "Dose by weight" template, previews and inserts it, inserts a built-in chart transform template, creates/edits/duplicates/deletes a custom `calc` template, persists the custom template library, and inserts the custom template from the command palette. |
| `node scripts/run-e2e.mjs` | Pass | Full Chromium browser workflow suite passed all 47 tests after broadening transform template management coverage. |

Transform template fill-field verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding derived fill-field metadata to template cards and the custom template editor. |
| `pnpm run test:unit` | Pass | 19 frontend unit tests passed, including template-library breadth, calc fill-field extraction for the "Dose by weight" template, structured transform fill-field extraction, and custom template normalization. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "manages transform templates" --project chromium` | Pass | Focused Chromium workflow passed with system Chrome and verified fill-value chips before template insertion. |

Rendered export review-case verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:rendered-exports` | Pass | The rendered export audit now writes `manual-review.html` plus `review-cases/rich-blocks` and `review-cases/option-heavy` artifacts for HTML, PDF, DOCX, PPTX, and Markdown bundle targets, records them in `rendered-export-audit-report.json`, checks required evidence through the Node viewer/package proof, runs Poppler `pdfinfo`/`pdftotext` proof against the primary and review-case PDFs when available, renders Poppler `pdftoppm` PNG thumbnails for representative PDF pages under `raster-proof/`, runs macOS `textutil` extraction against the primary DOCX, nested Google Docs DOCX, and both review-case DOCX files, attempts macOS Quick Look thumbnails for primary PDF/DOCX/PPTX artifacts, and captures Chromium-rendered screenshots for the primary HTML export, the manual dashboard, and both HTML review cases under `browser-visual-proof/`. |
| `node --check scripts/check-rendered-export-audit.mjs` | Pass | The rendered export audit verifier remains syntactically valid after adding Chromium visual proof and host-limited Quick Look classification. |
| `rg -n "pdftoppm\|browser-visual\|Visual Review Thumbnails" .tmp/rendered-export-audit/viewer-proof.json .tmp/rendered-export-audit/manual-review.html` | Pass | `viewer-proof.json` and `manual-review.html` include PDF raster proof rows plus `browser-visual-primary-html`, `browser-visual-manual-dashboard`, `browser-visual-review-rich-blocks`, and `browser-visual-review-option-heavy` rows with linked PNG screenshots. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting remains clean after adding generated review-case fixtures. |
| `cargo check --locked` in `src-tauri` | Pass | Dev-profile Rust check passed after adding generated review-case fixtures. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after recording the rendered export review-case evidence; all local links resolved. |
| `pnpm run verify:local` | Pass | Quick local verification passed all 9 steps after the rendered export review-case update. |
| `pnpm run build` | Pass | Vue typecheck and Vite production build passed after the rendered export review-case update. |
| `git diff --check` | Pass | No whitespace errors after the rendered export review-case update. |

Desktop native automation smoke verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node --check scripts/check-desktop-smoke.mjs` | Pass | Desktop smoke verifier syntax remained valid after adding macOS System Events evidence capture. |
| `pnpm run test:desktop-smoke` | Pass | Desktop artifact and native command workflow smoke still pass without GUI launch enabled. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Bounded macOS GUI launch smoke passed and `.tmp/desktop-smoke/launch-report.json` recorded app-authored Tauri window metadata, app-authored Vue workbench UI evidence, and `.tmp/desktop-smoke/native-workflow-report.json` proof. The workflow report asserts real-file save/new/open/revert flow, clean native root-file watcher reload with `watchDriver: "native"`, included-file native watcher tracking/recompile, stale-save conflict blocking, keep-local plus save, save-copy to `.tmp/desktop-smoke/native-workflow-export.md`, merge-back recovery, accept-external recovery, and final file restoration through the launched webview. It also proves native split/source/preview/focus/export/review/presentation mode switching, export/review/presentation sidebar routing, command-palette opening, Science `calc` template insertion into source, rendered preview output (`Total dose: 360 mg`), dirty title mutation, HTML export readiness progress evidence, app-initiated HTML export output at `.tmp/desktop-smoke/native-workflow-export.html` with `.manifest.json` sidecar proof, and native webview theme/accessibility evidence for dark theme, high contrast, reduced motion, editor font size, preview theme, preview font size, and preview line height; System Events evidence remains classified as limited on this host because it exposed the `neditor` process but not a window. |

Desktop app-authored workflow smoke verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding native menu-command evidence and strict progress reports to the guarded desktop workflow smoke path. |
| `pnpm run test:unit` | Pass | 21 frontend unit tests passed, including static coverage that the native workflow smoke gate, native menu-command evidence collector, report validator, and expected workflow assertions remain wired. |
| `pnpm run test:e2e` | Pass | 49 Chromium browser workflows passed locally through the system-Chrome fallback after the native menu and workspace/tab proof changes, covering editor commands, search, transforms/templates, file workflows, workspace grouping/restore, conflicts, tables, AI paste, export readiness, export profiles, and extended export targets. |
| `node --check scripts/check-desktop-smoke.mjs` | Pass | Desktop smoke verifier syntax remained valid after adding native workflow report validation. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting remained clean after registering the guarded workflow smoke commands. |
| `cargo check --locked` in `src-tauri` | Pass | Tauri command registration for the workflow smoke gate/report compiled cleanly. |
| `pnpm run check:deps` | Pass | Dependency admission remains synchronized after making the existing `native-watch` feature part of the default desktop build. |
| `pnpm run build` | Pass | Vue typecheck and Vite production build passed before rebuilding the release desktop binary. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Rebuilt the release desktop binary consumed by the native workflow launch smoke. |
| `pnpm run test:desktop-smoke` | Pass | Desktop artifact and native command workflow smoke passed without GUI launch enabled after adding the conflict-branch validator checks. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Launch smoke validated `.tmp/desktop-smoke/native-workflow-report.json` with passing assertions for real-file save/new/open/revert, clean native root-file watcher reload/restoration, native included-file watcher tracking/recompile/restoration, stale-save conflict blocking, rendered native conflict modal controls, local/external merge-base seeding in the modal, keep-local plus save, save-copy, merge-back recovery, accept-external recovery, split/source/preview/focus/export/review/presentation native mode switching, export/review/presentation sidebar routing, command palette, calc-template source insertion, rendered preview, dirty title, HTML export readiness, real HTML export writing, dark theme, high contrast, reduced motion, editor typography, and preview typography. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Rebuilt launch smoke now also validates native export-profile settings persistence: the launched Tauri webview saves a branded PDF profile, reapplies it after option drift, reloads it from the Tauri settings store, records `exportProfileEvidence`, and keeps the HTML sidecar manifest proof deterministic by clearing the active profile before direct HTML export. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Rebuilt launch smoke now also validates native View/Writing Tools menu command execution in the launched Tauri webview: export-preview, outline, and exports sidebar routing, CodeMirror search opening, TOC/equation/code-fence/table insertion, Templates panel opening, Docs Live modal opening, AI Paste modal opening, and guarded native `File` -> `Export` -> `HTML Export` execution. The smoke writes strict progress phases while running but the verifier still requires final `status: passed` evidence. |
| `python3 -c 'import json; ... nativeMenuCommandEvidence ...'` | Pass | The native workflow report records `status: passed`, `phase: final`, 84 passing assertions, and `nativeMenuCommandEvidence` with `exportMode`, `outline`, `exports`, `search`, `toc`, `equation`, `codeFence`, `table`, `templates`, `docsLive.open: true`, and `aiPaste` proof. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Rebuilt launch smoke now also validates native workspace/tab organization: deterministic real Markdown files under `.tmp/desktop-smoke/native-workspace-*`, document-set grouping for `Native Board Pack`, pinned-tab grouping, production drop-handler assignment of a loose note into the document set, close-group behavior, recently closed reopening, and restore of active/pinned/editor-scroll/preview-scroll state. |
| `node -e 'const r=require("./.tmp/desktop-smoke/native-workflow-report.json").payload; ... workspaceTabEvidence ...'` | Pass | The native workflow report records `workspaceTabEvidence` with `initialBoardGroup.count: 2`, `pinnedGroup.paths`, `looseAssigned.textHasDocumentSet: true`, closed group paths, `recentReopen.activePath`, and restored active/pinned/scroll ratios. |
| `node -e 'const r=require("./.tmp/desktop-smoke/native-workflow-report.json"); ...'` | Pass | The native workflow report records `status: passed`, `watchDriver: native` for clean root-file reload and included-file recompile assertions, `fileWorkflow.includePath` at `.tmp/desktop-smoke/native-workflow-file.include`, `fileWorkflow.copyPath` at `.tmp/desktop-smoke/native-workflow-export.md`, HTML `exportResult` progress steps `compile/transforms/readiness/render/manifest`, and passing assertions for blocked stale save, keep-local, saved kept-local conflict changes, saved local conflict copy, merged external conflict changes, accepted external conflict changes, and restored file content. |
| `rg -n "native workflow created and listed app-data snapshot\|native workflow restored app-data snapshot\|snapshotEvidence\|native-smoke\|snapshotPath\|containsMutation" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json` | Pass | Native desktop workflow evidence records app-data snapshot creation/listing with label `native-smoke`, snapshot path, source hash, and restoration after a mutation, with `containsMutation: false` in the restored evidence. |
| `rg -n "native workflow saved export profile\|native workflow applied export profile\|native workflow reloaded export profile\|exportProfileEvidence\|Native Board\|activeExportProfileId" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json` | Pass | Native desktop workflow evidence records the saved, applied, and reloaded profile assertions plus `exportProfileEvidence` with target `pdf`, compact layout, disabled manifest/cover/page-number flags, IEEE citation style, and the `Native Board` brand profile restored from settings. |
| `rg -n "Native include watcher\|native workflow watched included\|native workflow recompiled clean included\|watchDriver" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json .tmp/desktop-smoke/native-workflow-file.include .tmp/desktop-smoke/native-workflow-file.md` | Pass | Native desktop workflow evidence now includes the included watcher file at `.tmp/desktop-smoke/native-workflow-file.include`, watched paths for both root and include files, `watchDriver: native`, `Recompiled after included file changed: native-workflow-file.include`, and root restoration without persisting the include directive in `.tmp/desktop-smoke/native-workflow-file.md`. |
| `node -e 'const r=require("./.tmp/desktop-smoke/native-workflow-report.json"); ... conflict modal ...'` | Pass | The native workflow report records `native workflow rendered conflict modal controls`, `native workflow conflict modal seeded local merge base`, and `native workflow conflict modal seeded external merge base`, including modal text with the conflict path, local/external conflict markers, merge-base controls, resolution controls, and merged text previews. |
| `rg -n "blocked stale save\|kept local conflict\|saved kept-local\|saved local conflict copy\|merged external conflict\|accepted external conflict\|restored real file after conflict\|External native conflict edit\|Local unsaved native conflict edit\|native-workflow-file\|Save blocked" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json .tmp/desktop-smoke/native-workflow-file.md` | Pass | Native desktop workflow evidence now includes a stale-save conflict assertion with `reason: root` and `Save blocked; resolve external changes first`, keep-local/save-copy/merge/accept-external recovery, and restoration of `.tmp/desktop-smoke/native-workflow-file.md` without persisting the external, local, save-copy, or merged conflict markers. |
| `rg -n "native workflow saved document\|native workflow created new document\|native workflow opened saved real file\|native workflow reverted saved real file\|fileWorkflow\|native-workflow-file\|Market Entry Report\|Native smoke revert" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json .tmp/desktop-smoke/native-workflow-file.md` | Pass | Native desktop workflow evidence now includes saved-file, new-document, reopened-file, dirty-edit, and revert assertions; `.tmp/desktop-smoke/native-workflow-file.md` contains `Market Entry Report` and the validator rejects a persisted `Native smoke revert marker`. |
| `rg -n "native workflow wrote html export artifact\|exportResult\|native-workflow-export\|output_hash\|Total dose" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json .tmp/desktop-smoke/native-workflow-export.html .tmp/desktop-smoke/native-workflow-export.html.manifest.json` | Pass | Native desktop workflow evidence now includes the export assertion, `exportResult` progress steps `compile/transforms/readiness/render/manifest`, `.tmp/desktop-smoke/native-workflow-export.html` containing `Total dose: 360 mg`, and the sidecar manifest recording `export_target: html`, the output path, and a SHA-256 output hash. |
| `rg -n "native workflow switched\|modeEvidence\|themeAccessibility" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json` | Pass | The native workflow reports include per-mode desktop-webview evidence for `workspace mode-split`, `workspace mode-source`, `workspace mode-preview`, `workspace mode-focus`, `workspace mode-export`, `workspace mode-review`, and `workspace mode-presentation`, with export/review/presentation routed to the expected sidebars, plus computed desktop-webview evidence for `shellTheme: dark`, `highContrast: true`, `reducedMotion: true`, `commandBorderColor: rgb(0, 0, 0)`, `editorTransitionDuration: 0s`, `editorFontSize: 18px`, and preview `font-size: 19px; line-height: 1.9`. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "boots the workbench\|manages transform templates" --project chromium` | Pass | Focused browser workflows still passed with the system-Chrome fallback after adding the guarded native workflow IPC calls to app boot. |
| `pnpm run verify:local` | Pass | Quick local verification passed all 9 steps after adding the native workflow smoke report. |

Desktop WebDriver harness verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node --check scripts/run-tauri-webdriver.mjs` | Pass | Desktop WebDriver harness syntax remains valid after adding outline-mode structural editing plus guarded dialog-free Markdown save/open/rename/duplicate/reveal and HTML export write workflows. |
| `pnpm run check` | Pass | Vue typecheck passed after extending the desktop smoke path used by the WebDriver harness. |
| `pnpm run test:unit` | Pass | 22 frontend unit tests passed, including static guards that the desktop WebDriver harness covers outline-mode structural editing, dirty title, transform-template insertion, real Markdown save/open/rename/duplicate/reveal, export readiness, real HTML export writing, manifest hash validation, preference restart persistence, and the supported workflow plan. |
| `pnpm run test:tauri-webdriver` | Skipped on macOS with fresh native fallback proof | This host still records the official unsupported WKWebView-driver skip, but `.tmp/desktop-webdriver/report.json` now includes the supported Windows/Linux workflow plan with outline-mode structural editing and `fallbackProof.status: "passed"` from `.tmp/desktop-smoke/native-command-report.json`. The fallback records `freshForBinary: true`, `launchStatus: "survived-until-timeout"`, `processAlive: true`, outline title/navigation evidence, and HTML export output/manifest/hash evidence. |
| `rg -n "freshForBinary\|binaryMtime\|launchStatus\|processAlive" .tmp/desktop-webdriver/report.json` | Pass | The WebDriver skip report records desktop-binary, native-smoke, and launch-report freshness fields so stale `.tmp` native proof cannot silently satisfy the macOS fallback evidence contract. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after updating the WebDriver file/export evidence notes; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors after the desktop WebDriver file/export harness update. |

Rendered export manual sign-off verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node --check scripts/check-rendered-export-audit.mjs` | Pass | Rendered export audit verifier syntax remains valid after adding the structured manual sign-off template and completed-signoff validator. |
| `pnpm run test:unit` | Pass | 20 frontend unit tests passed, including the static guard that the rendered export audit exposes `visual-review-signoff.template.json`, `NEDITOR_RENDERED_EXPORT_SIGNOFF`, pending/human-reviewed states, and acceptance flags. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit passed and now writes `.tmp/rendered-export-audit/visual-review-signoff.template.json`; `viewer-proof.json` records passing `manual-signoff-template` evidence and a skipped `human-signoff` row until a completed reviewer file is supplied; `visual-review-summary.json` links the template and keeps `humanSignoff.status` at `pending-human-review` on this host. |
| `rg -n "visual-review-signoff\|humanSignoff\|human-signoff\|manual-signoff-template\|NEDITOR_RENDERED_EXPORT_SIGNOFF" .tmp/rendered-export-audit/visual-review-summary.json .tmp/rendered-export-audit/viewer-proof.json .tmp/rendered-export-audit/manual-review.html .tmp/rendered-export-audit/visual-review-signoff.template.json` | Pass | Generated audit artifacts include the sign-off template, manual dashboard instructions, summary sign-off link, passing template proof, and skipped completed-signoff proof. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the rendered export manual sign-off workflow; all local links resolved. |

Rendered export browser visual-proof verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit now captures the rendered `body` element for primary HTML, manual dashboard, and review-case screenshots, avoiding browser-page compositor artifacts while keeping DOM evidence, dimensions, and required text checks. |
| Visual inspection of `.tmp/rendered-export-audit/browser-visual-proof/*.png` | Pass | Representative screenshots show the expected NEditor export pages: primary HTML with cover/table/chart/comments/provenance/legal sections, manual review dashboard with proof tables, rich-block review, and option-heavy review. |

Rendered export Office preview verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node --check scripts/check-rendered-export-audit.mjs` | Pass | Rendered export audit verifier syntax remains valid after adding generated Office preview extraction and screenshot proof. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit passed and now writes `.tmp/rendered-export-audit/office-preview/office-preview-docx.html`, `.tmp/rendered-export-audit/office-preview/office-preview-pptx.html`, review-case DOCX/PPTX preview dashboards, and Chromium screenshots for each preview when the browser fallback is available. |
| `pnpm run test:unit` | Pass | 20 frontend unit tests passed, including static guards that the rendered export audit keeps the Office preview proof, screenshots, and summary mapping wired. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the generated Office preview export proof; all local links resolved. |

Rendered export automated visual-review verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node --check scripts/check-rendered-export-audit.mjs` | Pass | Rendered export audit verifier syntax remains valid after adding the current-host automated visual-review report. |
| `pnpm run test:unit` | Pass | 22 frontend unit tests passed, including the static guard that the rendered export audit writes `automated-visual-review.json`, reports `automated-reviewed`, and maps `automatedVisualReview` into the summary. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit passed and `.tmp/rendered-export-audit/automated-visual-review.json` recorded `status: "automated-reviewed"` with complete browser visual screenshots, Office preview extraction, Office preview screenshots, PDF raster proof, primary-target proof, and review-case proof on this host. |
| `rg -n "automated-visual-review\|automatedVisualReview\|automated-reviewed" .tmp/rendered-export-audit/viewer-proof.json .tmp/rendered-export-audit/visual-review-summary.json .tmp/rendered-export-audit/manual-review.html .tmp/rendered-export-audit/automated-visual-review.json` | Pass | The generated proof set links the automated review report from `viewer-proof.json`, `visual-review-summary.json`, and `manual-review.html`, and the report contains no blockers. |

External transform engine smoke verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node --check scripts/check-external-engines.mjs` | Pass | Optional engine probe syntax remains valid after adding adapter-shaped smoke artifact generation. |
| `NEDITOR_TEST_PIKCHR=/Users/nyimbiodero/src/pjs/tooling/neditor/.tmp/pikchr-build/pikchr-trunk/pikchr pnpm run check:engines` | Pass | Darwin arm64 reports Graphviz `dot`, `circo`, `neato`, `fdp`, `osage`, `twopi`, D2, PlantUML, and the locally built Pikchr executable installed and smoke-compatible; `.tmp/external-engines/probe-report.json` records SVG smoke artifacts under `.tmp/external-engines/artifacts/`, including `pikchr.svg`. |
| `node -e 'const r=require("./.tmp/external-engines/probe-report.json"); ... smoke ...'` | Pass | Probe report contains 9 installed engines, 9 passing smoke artifacts, zero incompatible engines, and zero unresolved missing optional-engine evidence on this host. |
| `cargo test --locked external_transform_adapters_shape_engine_specific_invocations --lib` in `src-tauri` | Pass | Focused adapter coverage proves modern Pikchr stdin execution passes the `-` stdin marker and keeps the `pikchr-cli` source-file argument shape working. |
| `cargo test --locked external_transform_conformance_runs_installed_engines --lib -- --nocapture` in `src-tauri` | Pass | Rust conformance continues to verify installed Graphviz variants, D2, PlantUML, and any configured Pikchr path through NEditor's external transform execution path. |
| `pnpm run test:unit` | Pass | 21 frontend unit tests passed, including static guards that `check:engines` keeps smoke artifacts, incompatible-engine failure reporting, PlantUML file-mode proof, and Pikchr CLI detection wired. |
| `pnpm run check:docs` | Pass | 13 Markdown files were checked after documenting the stronger external-engine platform evidence; all local links resolved. |

Export profile persistence verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 21 frontend unit tests passed, including workspace migration coverage for named export profiles and static guards for Export sidebar profile controls plus store save/apply/delete actions. |
| `pnpm run check` | Pass | Vue typecheck passed after adding normalized export profile persistence and profile manager UI. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "saves and reapplies reusable export profiles" --project chromium` | Pass | Browser workflow passed with the system-Chrome fallback, proving a branded PDF export profile can be saved, export settings can drift, the profile can be reapplied, and the saved profile survives reload. |

Native project-local snapshot verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Launched native smoke validates both app-data and project-local snapshot create/list/restore flows, including `.neditor/snapshots` path evidence for the project-local snapshot and mutation removal after restore. |
| `rg -n "native workflow created and listed project-local snapshot\|native workflow restored project-local snapshot\|native-project-smoke\|/.neditor/snapshots" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json` | Pass | Native desktop workflow evidence records project-local snapshot creation/listing with label `native-project-smoke`, `.neditor/snapshots` storage, and restoration after a mutation. |

Native title-state verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 22 frontend unit tests passed, including `src/lib/documentOutline.ts` coverage for parsing indented/numbered/bulleted outline drafts and rendering a document skeleton before body content exists. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "creates a document skeleton" --project chromium` | Pass | Focused browser workflow passed through the workspace-local Playwright Chromium cache, proving the Outline panel can accept an editable plan, generate front matter, `[TOC]`, nested headings, draft placeholders, and update source/preview/sidebar outline evidence. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Launched native smoke validates title state across real file save, dirty edit, revert, and later template insertion, so save/revert clear the native dirty marker while edit/template mutations add it. |
| `rg -n "native workflow save cleared native title\|native workflow dirtied native title for opened real file\|native workflow revert cleared native title\|native workflow exposed dirty title" .tmp/desktop-smoke/native-workflow-report.json .tmp/desktop-smoke/launch-report.json` | Pass | Native workflow reports include clean-title evidence after save, dirty-title evidence after editing an opened real file, clean-title evidence after revert, and dirty-title evidence after inserting a calc template. |
| `pnpm run test:tauri-webdriver` | Skipped on macOS with native proof | Official Tauri WebDriver remains unavailable for macOS WKWebView, but `.tmp/desktop-webdriver/report.json` now records `fallbackProof.status: "passed"` from `.tmp/desktop-smoke/native-command-report.json`, including 72/72 native workflow assertions, visible NEditor window evidence, the real Markdown workflow file path, HTML export output/manifest paths, and the sidecar output hash. |
| `pnpm run verify:local:full -- --list` | Pass | Full local verification now lists `Desktop macOS GUI launch smoke: NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` before the desktop WebDriver step on macOS. |

Live Google Docs import attempt:

| Command | Result | Evidence |
| --- | --- | --- |
| Google Drive `_import_document` with `.tmp/rendered-export-audit/rendered-export-audit.docx` | Blocked by connector authorization | The current connector call returned `token_expired` before upload/conversion; an earlier attempt reached the Drive upload-conversion endpoint and returned `403 Forbidden`. The exported DOCX/package remains locally verified, and live import proof needs a refreshed Google Drive OAuth scope or another authorized Drive session. |
| Google Drive `_import_document` with `.tmp/rendered-export-audit/rendered-export-audit.docx` on 2026-05-23 | Blocked by connector authorization | The connector still returns `token_expired`, so no live upload/conversion/readback evidence can be generated from this session until Drive authorization is refreshed. |

AI lifecycle task preservation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 45 frontend unit tests passed, including a long-outline, evidence-heavy, nine-target agentic release run proving the bounded lifecycle task board preserves every target-specific distribution task plus final release readiness while keeping section drafting and evidence tasks present. |

Compiler media inventory verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | Frontend static coverage now requires the Compiler output inventory to expose explicit Media map and Figure media uses rows. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "navigates compiler diagnostics" --project chromium` | Pass | The browser workflow opens a document with a figure and diagnostic target, then proves the Diagnostics sidebar reports `Media map` with one media file and `Figure media uses` with one figure use beside the source-ranged diagnostic inventory. |

Agent-selected transform recommendation verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 46 frontend unit tests passed, including agentic runs that produce source-grounded calc, chart, table, timeline/schema/equation/publishing recommendations with suggested Markdown, lifecycle tasks, audit events, release evidence, UI insertion/copy actions, and persisted run-history counts. |
| `pnpm run check:ai-roadmap` | Pass | The AI-first roadmap verifier treats Agent-Selected Transforms as a required product surface for roadmap items 29 and 30 and passed the 50-change roadmap contract. |

Data-to-narrative bridge verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 46 frontend unit tests passed, including agentic runs that link claims, calculations, charts, tables, timelines, schemas, equations, and publishing metadata to affected narrative sections with review actions, release evidence, audit events, UI insert/copy actions, and persisted run-history counts. |
| `pnpm run check:ai-roadmap` | Pass | The AI-first roadmap verifier treats the Data-to-Narrative Bridge as a required product surface for roadmap item 30 and passed the 50-change roadmap contract. |

Approval metadata gate verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding the shared Approval Metadata Gate run model, UI surface, release evidence, audit markdown, and persisted run-history status. |
| `pnpm run test:unit` | Pass | 47 frontend unit tests passed, including blocked distribution runs with missing approval/source-confidence metadata and a ready run with complete status, reviewer, approvedAt, owner, releaseTarget, and sourceConfidence metadata. |
| `pnpm run check:ai-roadmap` | Pass | The AI-first roadmap verifier now treats Approval Metadata Gate as a required product surface for roadmap item 48 and passed the 50-change roadmap contract. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting the distribution gate in README and the spec completion matrix; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | The spec completion matrix remains partial-with-release-risks while recording Approval Metadata Gate evidence for distribution blockers. |
| `pnpm run check:a11y` | Pass | Static accessibility guardrails passed with the new Agent Workspace approval gate section and controls. |
| `git diff --check` | Pass | The approval gate diff has no whitespace errors. |

AI control-center approval gate wiring verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after moving Approval Metadata Gate ahead of control-center construction and threading it into governance, distribution, next-action, and automation-preflight decisions. |
| `pnpm run test:unit` | Pass | 47 frontend unit tests passed, including blocked agent runs whose AI Control Center summary, readiness state, next actions, governance rows, distribution rows, and automation preflight now reflect the approval gate instead of scattered approval hints. |
| `git diff --check` | Pass | The control-center gate-wiring diff has no whitespace errors. |

Release evidence kit closure-plan verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `node --check scripts/collect-release-evidence-kit.mjs` | Pass | Evidence-kit collector syntax remains valid after adding per-gap validator commands, ingest commands, final readiness commands, and `readyToSend` closure metadata. |
| `node --check scripts/check-release-evidence-kit.mjs` | Pass | Evidence-kit checker syntax remains valid after making it reject gap work items without runbooks, returned evidence paths, validator commands, ingest commands, and final release-readiness commands. |
| `pnpm run test:unit` | Pass | 47 frontend unit tests passed, including static guards for the strengthened release evidence kit closure-plan and ingest contract. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting the self-validating release evidence kit work items; all local links resolved. |

Agent automation scheduler execution verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding in-place automation queue execution state, run-safe-queue controls, per-check result reporting, surface navigation, and Markdown audit insertion/copy actions. |
| `pnpm run test:unit` | Pass | 47 frontend unit tests passed, including static guards and migration coverage for persisted Automation Scheduler task states in local run history. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "offers searchable contextual help"` | Pass | Focused Chromium workflow generated an agent packet, ran the safe automation queue without leaving the Agent Workspace, and verified persisted run-history automation summaries plus the refreshed evidence scan. |

Document-set manager verification:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding the Files-sidebar Document Sets manager, current-text front matter grouping, set assignment, set renaming, active-document removal controls, and Markdown manifest generation. |
| `pnpm run test:unit` | Pass | 47 frontend unit tests passed, including static guards for the Document Sets manager, current front matter grouping helpers, and manifest command surface. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "groups documents by document set"` | Pass | Focused Chromium workflow opened folder/document-set groups, dragged a loose document into a set, saved generated `documentSet` front matter, renamed the open set, verified immediate regrouping, inserted a Document Set Manifest with set members and review handoff, removed the active document from the set, and closed the remaining group without disturbing another folder group. |

Native diagnostic and preview source-map proof:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding preview source-anchor annotation and native diagnostic/source-map smoke collection. |
| `pnpm run test:unit` | Pass | 47 frontend unit tests passed, including static guards for the new native diagnostic and preview source-map smoke evidence. |
| `pnpm run build` | Pass | Production Vite build passed after preview tables gained generated IDs/captions for source-map navigation. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release-profile Tauri binary rebuilt with the updated webview bundle. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | App-authored launched-webview smoke passed with 102 assertions. The report records `diagnosticNavigationEvidence` for a real broken-image compiler diagnostic rendered as one CodeMirror lint range and one lint gutter marker, preview diagnostic `Go to source`, and Diagnostics sidebar `Go to source`; it also records `previewSourceMapEvidence` for rendered table/equation caption clicks that land on the Markdown table source and nearby equation source block. |
| `pnpm run test:tauri-webdriver` | Pass | macOS WebDriver remains an official skip, and the script refreshed the native launch fallback proof against the current smoke artifacts. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` after the native fallback refresh. |

Native fold visual-state proof:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding native fold placeholder evidence to the editor ergonomics smoke. |
| `pnpm run test:unit` | Pass | 47 frontend unit tests passed, including static guards for the new fold visual-state assertion and validator coverage. |
| `pnpm run build` | Pass | Production Vite build passed after threading fold visual-state evidence through the app-authored smoke. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release-profile Tauri binary rebuilt with the updated smoke path. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | App-authored launched-webview smoke passed with 103 assertions. The report records `editorErgonomicsEvidence.foldState` with `beforeFold: 0`, `foldedPlaceholderCount: 2`, `afterUnfold: 0`, placeholder text visible while folded, and restored Metrics/list text after unfold. |
| `pnpm run test:tauri-webdriver` | Pass | macOS WebDriver remains an official skip, and the script refreshed the native launch fallback proof against the current smoke artifacts. |

Business identity, document wizard, snippets, and local-agent handoff:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 48 frontend unit tests passed after adding `src/lib/businessDocuments.ts`, persistent business profile normalization, business-development templates for tutorials/proposals/RFPs/RFQs/tenders and related document types, reusable document-part snippets, Docs Live blueprint support for procurement/tutorial documents, and Claude Code/Codex/OpenCode provider handoff profiles. |
| `pnpm run check` | Pass | Vue typecheck passed after wiring the business identity modal, Templates-sidebar document wizard/snippets, Docs Live wizard stages, and provider handoff command routing. |
| `pnpm run build` | Pass | Production Vite build passed after the UI, store, provider profile, and shared library updates. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting business identity, document builders, reusable parts, and Claude Code/Codex/OpenCode handoff; all local links resolved. |
| `./node_modules/.bin/tauri build --no-bundle` | Pass | Release-profile Tauri binary rebuilt with the updated workbench bundle and native smoke path. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | App-authored launched-webview smoke passed after adding `tocNavigationEvidence`; the validator now requires numbered/depth-limited TOC rendering plus TOC preview link-to-source navigation. |
| `pnpm run test:tauri-webdriver` | Pass | macOS WebDriver remains an official skip, and the script refreshed the native launch fallback proof against the current binary and smoke artifacts. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` after the native fallback refresh. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` with current evidence rows for this slice. |
| `git diff --check` | Pass | No whitespace errors in the business wizard, native TOC, docs, tests, and validator diff. |

Native RFP response wizard:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding the native Templates-sidebar RFP response wizard, RFP analysis state, full response generation, Docs Live handoff, and local-agent handoff wiring. |
| `pnpm run test:unit` | Pass | 49 frontend unit tests passed, including RFP source analysis for requirements, stated buyer intent, implied buyer intent, timelines, budget hints, mandatory attachments, compliance matrix rows, and generated review-ready Markdown. |
| `cargo test --locked rfp_import --lib` in `src-tauri` | Pass | Native importer unit tests passed for DOCX XML text extraction and URL HTML-to-text cleanup. The new Tauri command accepts Markdown/text, DOCX packages, PDF via local `pdftotext`, and URL fetch via local `curl`, returning warnings when source capture needs manual verification. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "builds business documents from saved identity snippets and local-agent handoff"` | Pass | Focused Chromium workflow passed outside the sandbox on this host and wrote `.tmp/e2e-browser/business-document-wizard-report.json` with `businessDocumentWizard: true` and `rfpResponseWizard: true`. The workflow now proves pasted RFP analysis, stated/implied intent surfacing, requirement verification, full response creation, and compliance matrix rendering. |

Equation editor and EPUB export:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked export_document_writes_epub_package --lib` in `src-tauri` | Pass | Direct native EPUB export writes `.epub` output with `mimetype`, `META-INF/container.xml`, `OEBPS/content.opf`, `OEBPS/nav.xhtml`, `OEBPS/document.xhtml`, stylesheet, embedded manifest, and no sidecar when package-embedded manifests are requested. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | 29 export command tests passed, including EPUB extension validation, release metadata gating, optional manifest diagnostics, and desktop native command export through every target including EPUB. |
| `pnpm run test:unit` | Pass | 49 frontend unit tests passed, including static UI guards for the Equation Editor, RFP response wizard, EPUB target persistence, agentic EPUB target detection, and release-readiness target lists. |
| `pnpm run test:rendered-exports` | Pass | Rendered export audit passed and now writes `.tmp/rendered-export-audit/rendered-export-audit.epub` with executable checks for EPUB container, OPF metadata, navigation, XHTML body, text fallback, packaged media, embedded NEditor manifest, visual review summary mapping, and package evidence. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "publishes and hands off extended export targets"` | Pass | Focused Chromium workflow passed outside the sandbox and wrote `.tmp/e2e-browser/extended-export-targets-report.json` with `exportWorkflows: true` and `epubExport: true`, proving the Export panel exposes EPUB, prepares readiness, writes the selected `.epub` output path, and reports manifest evidence. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "builds business documents from saved identity snippets and local-agent handoff"` | Pass | Focused Chromium workflow passed outside the sandbox and wrote `.tmp/e2e-browser/business-document-wizard-report.json` with `businessDocumentWizard: true`, `rfpResponseWizard: true`, and `equationEditor: true`, proving the equation editor templates and insertion flow in the same business-document workflow. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed outside the sandbox with 64 tests. `.tmp/e2e-browser/report.json` records `businessDocumentWizard`, `rfpResponseWizard`, `equationEditor`, `exportWorkflows`, and `epubExport` as true. |
| `pnpm run build` | Pass | Production Vite build passed after EPUB UI, native command wiring, RFP wizard, and equation editor additions. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting EPUB in the README, user guide, progress log, and spec completion matrix; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` with current evidence rows for EPUB export and the native Equation Editor. |
| `cargo check --locked` in `src-tauri` | Pass | Rust command/backend code compiles with the EPUB renderer, native RFP importer, and updated export command target set. |
| `git diff --check` and `cargo fmt --check` | Pass | No whitespace errors and Rust formatting is clean. |

Calc block diagnostic precision:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked calc_blocks_resolve_forward_refs_and_report_cycles --lib` in `src-tauri` | Pass | Focused compiler proof now checks cyclic calc block diagnostics include the Markdown source file, exact formula source line, start/end columns, formula-name related context, and dependency related context. |

Direct EPUB export affordances:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding the direct EPUB toolbar, Export panel, command-palette, and help-action routes. |
| `pnpm run test:unit` | Pass | 50 frontend unit tests passed, including static guards for direct EPUB export commands, toolbar metadata, native menu mapping, and command wiring. |
| `cargo test --locked export_document_writes_epub_package --lib` in `src-tauri` | Pass | Focused native export proof still writes a valid `.epub` package with EPUB container, OPF metadata, navigation, XHTML body, stylesheet, embedded manifest, and packaged text fallback. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "publishes and hands off extended export targets"` | Pass | Focused Chromium workflow passed and now uses the direct **Export EPUB** button for the EPUB target while preserving readiness, output-path, manifest-path, and status evidence. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after the direct EPUB flow change, restoring the full-suite browser evidence expected by release readiness. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting direct EPUB access in the README, user guide, progress log, and spec completion matrix; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the EPUB affordance evidence update. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` after refreshing the full-suite browser workflow evidence. |

Target-specific public metadata readiness:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked prepare_for_export_validates_public_distribution_metadata --lib -- --nocapture` in `src-tauri` | Pass | Blog readiness now blocks malformed front matter `canonicalUrl` values and invalid `language` tags before writing public publishing packages, and copies both diagnostics into manifest readiness. |
| `cargo test --locked prepare_for_export_reports_public_distribution_metadata_warnings --lib -- --nocapture` in `src-tauri` | Pass | Substack/blog readiness now warns when a publishing handoff lacks description/summary/subtitle/excerpt metadata or tags/keywords for previews, RSS, discovery, and archive management. |
| `cargo test --locked prepare_for_export_validates_public_distribution_options --lib -- --nocapture` in `src-tauri` | Pass | HTML readiness now rejects malformed `canonicalUrl`, invalid `htmlLanguage`, non-string `language`, and non-string `htmlDescription` options before rendering. |
| `cargo test --locked prepare_for_export_reports_target_specific_release_metadata_blockers --lib -- --nocapture` in `src-tauri` | Pass | Existing target-specific release blockers still pass after adding public metadata diagnostics, with publishing and EPUB metadata warnings now counted explicitly. |
| `cargo test --locked export_document_applies_public_metadata_options_to_publish_packages --lib -- --nocapture` in `src-tauri` | Pass | Blog package rendering now applies export-level canonical URL, description, and language options to `metadata.json`, `post.html`, and `rss-item.xml`, keeping public metadata controls aligned with generated artifacts. |
| `cargo test --locked export_document_writes_epub_package --lib -- --nocapture` in `src-tauri` | Pass | EPUB rendering now carries export-option language into OPF metadata plus nav/document XHTML language attributes. |
| `cargo test --locked export_command_tests --lib` in `src-tauri` | Pass | Full export command suite passed with 33 tests after the public metadata readiness and renderer additions, covering package exports, manifests, dirty-Git warnings, output-path checks, target-specific readiness diagnostics, and public metadata package output. |
| `cargo check --locked` in `src-tauri` | Pass | Rust backend check passed after adding public metadata validators, renderer metadata plumbing, and readiness diagnostics. |
| `pnpm run check` | Pass | Vue typecheck passed after adding the Distribution metadata checklist and target-aware public metadata controls in the Export sidebar. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "publishes and hands off extended export targets"` | Pass | Focused Chromium workflow proves the Export sidebar checklist surfaces release and publishing metadata needs, writes suggested owner/releaseTarget/description/tags/language front matter, then completes blog, Substack, LaTeX, Google Docs, and EPUB handoffs. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after refreshing browser evidence for the current source files. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting public metadata readiness evidence; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the validation evidence update. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` after refreshing browser workflow evidence. |

Equation editor template depth:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding categorized/searchable Equation Editor templates and a larger business/science/mathematics template library. |
| `pnpm run test:unit` | Pass | 50 frontend unit tests passed, including static guards for Equation Editor filters, template categories, and the expanded matrix-template surface. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "builds business documents from saved identity snippets and local-agent handoff"` | Pass | Focused Chromium workflow proves users can filter Equation Editor templates by category/search, load total-cost and molarity templates, see Markdown preview updates, switch to inline mode, and insert an inline equation into the document. |

AI paste clipboard import:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding the explicit AI Paste **Load clipboard** action and guarded busy/status state. |
| `pnpm run test:unit` | Pass | 50 frontend unit tests passed, including static guards for the Load clipboard control and handler. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "previews and inserts cleaned AI paste through the modal"` | Pass | Focused Chromium workflow proves rich HTML clipboard import, status feedback, cleanup preview, and insertion through the AI paste modal. |

Toolbar space recovery and Vim operator parity:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after replacing stacked collapsed toolbar rows with a single collapsed-toolbar tray and expanding Vim normal mode with change/delete operator motions. |
| `pnpm run test:unit` | Pass | 50 frontend unit tests passed, including static guards for the collapsed toolbar tray and direct unit coverage for Vim operator-motion ranges. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "collapses and restores command toolbars"` | Pass | Focused Chromium workflow proves collapsed toolbars move into one tray, the command bar height drops below 45% of its expanded height, and the document workspace gains more than 40px of vertical writing space. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "runs configurable Emacs and Vim-style editor keybinding modes"` | Pass | Focused Chromium workflow proves the expanded Vim operator set, including `db`, `C`, `J`, and `cw`, while preserving Emacs mode and persisted Vim settings. |
| `pnpm run build` | Pass | Production Vue/Vite build passed after the toolbar tray and Vim operator changes. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after the toolbar and Vim updates, preserving the full-suite browser evidence needed by release readiness. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after updating progress and spec evidence for the toolbar and Vim improvements; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the updated evidence. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` with all current-host checks accepted. |

Local agent CLI workspace preparation:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked local_agent --lib` in `src-tauri` | Pass | Native local-agent tests now prove Claude Code/Codex/OpenCode remain allowlisted and Google Antigravity writes a governed handoff file under `.neditor/agent-handoffs` with the `antigravity` launch command. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting passed after adding the Google Antigravity native local-agent profile. |
| `cargo check --locked` in `src-tauri` | Pass | The registered `prepare_local_agent_handoff` command compiles with the expanded local-agent allowlist. |
| `pnpm run test:unit -- --runInBand` | Pass | 85 frontend unit/static tests passed, including the static business-document guard that keeps Google Antigravity visible with Claude Code, Codex, and OpenCode in the wizard handoff catalog. |
| `pnpm run check` | Pass | Vue typecheck passed after the native/frontend local-agent catalog alignment. |
| `pnpm run check:docs` | Pass | Markdown docs were checked after documenting the native Google Antigravity handoff alignment; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with native Google Antigravity handoff evidence recorded under AI workflow. |
| `git diff --check` | Pass | No whitespace errors are present in the native Antigravity handoff diff. |
| `cargo test --locked local_agent --lib` in `src-tauri` | Pass | Native local-agent tests prove unsupported profiles are rejected, Codex handoff Markdown is written under `.neditor/agent-handoffs`, and executable discovery is resolved through an allowlisted command path. |
| `cargo check --locked` in `src-tauri` | Pass | The new `prepare_local_agent_handoff` Tauri command compiles with the registered app invoke handler. |
| `pnpm run check` | Pass | Vue typecheck passed after wiring local-agent handoff state, result rendering, and command invocation. |
| `pnpm run test:unit` | Pass | 49 frontend unit tests passed, including guards for Claude Code, Codex, OpenCode local-agent profile metadata and the new local-agent workspace UI. |
| `pnpm run build` | Pass | Production Vite build passed with the local-agent handoff panel and native invoke wiring. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "routes natural language command palette instructions to AI workflow surfaces"` | Pass | Focused Chromium workflow proves command-palette provider routing, switching to the Codex CLI profile, preparing the local-agent workspace, and displaying the `.neditor/agent-handoffs` file path. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after adding local-agent workspace preparation to the Agent Workspace provider flow. |

Editor keybinding parity:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after expanding Vim normal mode with explicit word-start/word-end motion, line-start/line-end insert/append, and `dd` pending-operator deletion. |
| `pnpm run test:unit` | Pass | 49 frontend unit tests passed, including static guards for the Vim pending operator, word motions, line deletion, and keybinding wiring. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "runs configurable Emacs and Vim-style editor keybinding modes"` | Pass | Focused Chromium workflow proves Emacs line commands plus Vim normal-mode blocking, insert/append, `I`/`A`, `dd`, `w`, and persisted Vim settings. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after extending Vim normal-mode parity. |

Frontend architecture modularization:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after moving Docs Live review-packet Markdown formatting and audit-inline sanitization from `src/App.vue` into direct helpers in `src/lib/docsLive.ts`. |
| `pnpm run test:unit -- --runInBand` | Pass | 85 focused frontend unit/static tests passed, including direct Docs Live review-packet Markdown coverage and static wiring proof that the UI uses the extracted helper. |
| `pnpm run check:docs` | Pass | Markdown docs were checked after documenting the Docs Live review-packet helper extraction; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with the Docs Live helper extraction recorded under frontend architecture evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the Docs Live helper extraction diff. |
| `pnpm run check` | Pass | Vue typecheck passed after moving Vim normal-mode command handling and word-motion helpers from `src/App.vue` into `src/lib/vimKeybindings.ts`. |
| `pnpm run test:unit` | Pass | 50 frontend unit tests passed, including direct coverage for pure Vim word-start/word-end helper semantics and static wiring guards for the extracted module. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "runs configurable Emacs and Vim-style editor keybinding modes"` | Pass | Focused Chromium workflow re-proved the extracted keybinding module through the real editor UI. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after extracting the Vim keybinding module. |

Business wizard local-agent coverage:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit -- --runInBand` | Pass | Focused frontend unit/static coverage proves business-document local-agent handoff metadata now includes Claude Code, Codex, OpenCode, and Google Antigravity, and that generated wizard context exposes Google Antigravity to non-technical business users. |
| `pnpm run check` | Pass | Vue typecheck passed after making Templates and RFP handoff copy consistent with the expanded governed local-agent set. |
| `pnpm run check:docs` | Pass | README and user-guide links/checks passed after documenting Google Antigravity in business-document and setup guidance. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with AI workflow evidence updated for Google Antigravity handoffs. |
| `git diff --check` | Pass | No whitespace errors are present in the local-agent handoff coverage diff. |

Two-way table source editing:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | Frontend unit coverage now proves Markdown source tables can be converted into table-editor drafts, replaced over the original source range, and reparsed with edited values plus escaped pipe cells intact. |
| `pnpm run check` | Pass | Vue typecheck passed after moving source table draft conversion and source-range replacement helpers into `src/lib/tables.ts`. |
| `pnpm run test:unit` | Pass | Frontend unit coverage now proves pasted Markdown/CSV rows and spreadsheet-import rows share one draft construction helper for headers, fallback captions, alignments, inferred formats, and empty data rows. |

Menus, quality recommendations, and expanded document wizards:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after adding visible application menus, expanded toolbar groups, quality recommendation actions, wizard buttons, and current-text save/snapshot flushing. |
| `pnpm run test:unit` | Pass | 50 frontend unit tests passed, including static guards for the in-app menus, native menu command mapping, quality recommendation UI/actions, and the lesson/textbook/novel/podcast/movie wizard metadata. |
| `cargo fmt --check` in `src-tauri` | Pass | Native menu additions are formatted after applying `cargo fmt`. |
| `cargo check --locked` in `src-tauri` | Pass | Rust backend compiled after adding native File, Document Wizards, and Quality menu entries. |
| `git diff --check` | Pass | No whitespace errors are present in the current diff. |
| `pnpm run build` | Pass | Production Vue/Vite build passed with the visible menu bar, expanded command groups, and quality recommendation sidebar. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting the menu, QA, and wizard exposure; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the evidence refresh. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests, including visible File/Writing Tools/Quality menus, menu-routed QA review, expanded wizard access, save/snapshot restore, and extended export workflows. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` after refreshing full-suite browser evidence. |

Quality recommendation modularization:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after extracting deterministic QA/QI recommendation logic from `src/App.vue` into `src/lib/qualityRecommendations.ts`. |
| `pnpm run test:unit` | Pass | 52 frontend unit tests passed, including direct coverage for quality recommendation classification, summary formatting, pass-state behavior, and insertable report Markdown. |
| `git diff --check` | Pass | No whitespace errors are present in the QA extraction diff. |
| `pnpm run build` | Pass | Production Vue/Vite build passed with the extracted QA recommendation module. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting the QA extraction; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the evidence refresh. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after the QA extraction, preserving the visible menu and Review sidebar QA workflow proof. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` after refreshing browser evidence. |

Release readiness extraction and long-form wizard sequencing:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after extracting release-readiness checklist logic and making textbook/novel Docs Live workflows outline-or-plot-first. |
| `pnpm run test:unit` | Pass | 55 frontend unit tests passed, including direct release-readiness missing/approved-state coverage and textbook/novel wizard sequencing coverage for generated drafts, business templates, and Agent Workspace quality gates. |
| `git diff --check` | Pass | No whitespace errors are present in the release-readiness and long-form wizard sequencing diff. |
| `pnpm run build` | Pass | Production Vue/Vite build passed with the extracted release-readiness module and updated long-form wizard behavior. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting release-readiness extraction and long-form wizard sequencing; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the evidence refresh. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after the release-readiness extraction and textbook/novel wizard sequencing changes. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` with refreshed browser workflow evidence. |

Long-form wizard planning gates:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 58 frontend unit tests passed after strengthening textbook and novel wizard generation with architecture/plot approval gates, planning-artifact tables, sequential chapter draft queues, per-chapter acceptance criteria, final instructional/narrative quality review checklists, and template gate coverage. |

Front matter manager YAML edge cases:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec tsc -p tsconfig.test.json` | Pass | Test TypeScript compilation passed after hardening `src/lib/frontMatterManagers.ts`. |
| `node --test --test-name-pattern "front matter managers" .tmp-tests/tests/frontend-unit.test.js` | Pass | Focused frontend unit coverage passed for CRLF front matter, quote-aware YAML comments, comma-containing inline lists, compact inline `dataSources` block objects plus top-level object/list declarations, scalar path entries inside inline `dataSources` lists and direct scalar `dataSources` declarations, scalar aliases and inline/block-map `<<: *defaults` inside compact data-source objects, expanded data-source row scalar aliases such as `path: *sourcePath`, top-level `dataSources: *alias` reuse of anchored maps, scalar paths, inline source lists, and block source lists, legacy alias-section scalar, inline-list, block-list aliases, and direct scalar declarations, anchored compact objects reused by later inline-list rows, anchored source maps with nested lists preserved as map defaults, aliased block-list source rows beginning with `<<: *defaults`, expanded `dataSources` rows beginning with `- <<: *defaults`, custom tags on compact data-source objects, inline lists, and alias lists, `yml` alias normalization, URL/Windows/parent traversal path blocking, safe filenames containing `..`, scalar anchors/aliases, and folded/literal block scalar variables. |
| `pnpm run check` | Pass | Vue typecheck passed after the front-matter parser hardening. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting front-matter manager edge-case coverage; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the front-matter addendum evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the parser, tests, or docs diff. |

Front matter manager anchors and multiline variables:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec tsc -p tsconfig.test.json` | Pass | Test TypeScript compilation passed after adding simple scalar anchor/alias and block-scalar handling. |
| `node --test --test-name-pattern "front matter managers" .tmp-tests/tests/frontend-unit.test.js` | Pass | Three focused frontend unit tests passed, including scalar anchors, aliases, nested aliases, quoted `#` in anchored values, literal block scalar summaries, and folded block scalar excerpts. |

Front matter manager merge keys and tags:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec tsc -p tsconfig.test.json` | Pass | Test TypeScript compilation passed after adding conservative tagged-scalar and simple merge-key handling. |
| `node --test --test-name-pattern "front matter managers" .tmp-tests/tests/frontend-unit.test.js` | Pass | Four focused frontend unit tests passed, including `!!str`, `!custom`, `!<tag:yaml.org,2002:str>`, simple `<<: *defaults` merges, list-form merge aliases, nested scalar defaults such as `address.city`/`delivery.timezone`, inherited anchored-map defaults, and explicit value override of merged defaults. |

Front matter manager typed tags:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec tsc -p tsconfig.test.json && node --test --test-name-pattern "front matter managers" .tmp-tests/tests/frontend-unit.test.js` | Pass | Seven focused front-matter manager tests passed after adding typed scalar behavior: `!!null` and URI null tags become empty rows, `!!bool` and URI bool tags normalize yes/no/on/off forms, `!!str null` stays a literal string, and anchored typed defaults merge into downstream variables. |

Front matter manager inline maps:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec tsc -p tsconfig.test.json` | Pass | Test TypeScript compilation passed after adding conservative inline YAML map expansion. |
| `node --test --test-name-pattern "inline object" .tmp-tests/tests/frontend-unit.test.js` | Pass | Focused frontend unit coverage proves simple inline maps expand into dotted variables, scalar aliases inside inline maps resolve, nested inline maps become dotted variables, inline sequences and block sequences produce indexed dotted variables for scalar and object entries, block-list items can start with `<<: *defaults` merge keys without surfacing literal merge text, direct `client: *clientDefaults` map aliases expand instead of rendering as literal alias text, aliased inline sequences can be reused, anchored block-list rows can be reused as direct map aliases, custom tags on inline maps and block-list rows do not hide scalar fields, merge defaults populate compact client metadata, explicit nested values override merged values while preserving sibling defaults, quoted `#` text survives, and nested anchored-parent inline maps can be reused through later merges. |

Export metadata checklist modularization:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after moving Export sidebar distribution metadata checklist logic from `src/App.vue` into `src/lib/exportMetadataChecklist.ts`. |
| `pnpm run test:unit` | Pass | 56 frontend unit tests passed, including direct blog publishing metadata and EPUB handoff readiness checklist coverage. |
| `git diff --check` | Pass | No whitespace errors are present in the export metadata checklist extraction diff. |
| `pnpm run build` | Pass | Production Vue/Vite build passed with the extracted Export sidebar checklist module. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting export metadata checklist modularization; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the frontend architecture evidence refresh. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after the Export sidebar checklist extraction. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` with refreshed browser workflow evidence. |

Shared front matter helper modularization:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue typecheck passed after moving shared front matter scalar/list read and upsert helpers into `src/lib/frontMatter.ts`. |
| `pnpm run test:unit` | Pass | 57 frontend unit tests passed, including direct exact-key front matter helper coverage for scalar reads, list reads, scalar upsert/removal, and list upsert. |
| `git diff --check` | Pass | No whitespace errors are present in the shared front matter extraction diff. |
| `pnpm run build` | Pass | Production Vue/Vite build passed with the shared front matter helper module. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting shared front matter helper modularization; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the frontend architecture evidence refresh. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 64 tests after the shared front matter helper extraction. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` with refreshed browser workflow evidence. |

RFP compliance verification hardening:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 68 frontend unit tests passed, including requirement-level evidence checklists, compliance coverage, and generated response verification sections. |
| `pnpm run check` | Pass | Vue typecheck passed for the expanded RFP analysis shape and sidebar display. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after recording the RFP verification evidence; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after strengthening native RFP response verification. |

Long-form wizard planning gate hardening:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 68 frontend unit tests passed, including textbook and novel outline/plot-first behavior. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed for the Docs Live long-form planning gate changes. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after recording the long-form wizard evidence; all local links resolved. |
| `git diff --check` | Pass | No whitespace errors are present in the Docs Live, tests, or progress diff. |

Emacs editor keybinding hardening:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 69 frontend unit tests passed, including Emacs kill-line, word-range, kill/yank wiring, and existing editor behavior. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed for the supplemental Emacs keymap module and editor wiring. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after updating spec evidence for Emacs editing parity; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after refreshing the Vim/Emacs evidence row. |
| `git diff --check` | Pass | No whitespace errors are present in the editor keybinding diff. |

Logo metadata diagnostic source ranges:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compiler_reports_broken_local_markdown_links --lib` in `src-tauri` | Pass | Focused compiler diagnostic verification passed for broken Markdown links, images, and front-matter logo source ranges. |
| `cargo fmt --check` in `src-tauri` | Pass | Rust formatting is clean after the logo diagnostic range hardening. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after refreshing diagnostic evidence; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after documenting the richer diagnostics proof. |
| `git diff --check` | Pass | No whitespace errors are present in the diagnostic and docs diff. |

Command palette discoverability hardening:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 69 frontend unit/static tests passed, including evidence that command-palette search includes richer open-document, workspace, include, heading, citation, glossary, index, and diagnostic metadata. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed for dynamic palette metadata and computed bibliography reference access. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after updating command-palette evidence; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the command-palette evidence refresh. |
| `git diff --check` | Pass | No whitespace errors are present in the UI, test, or docs diff. |

Command palette helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after extracting command-palette metadata compaction and search text helpers to `src/lib/commandPalette.ts`. |
| `pnpm run test:unit` | Pass | 70 frontend unit/static tests passed, including direct command-palette metadata compaction, description joining, and searchable text coverage. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting command-palette helper extraction; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the frontend architecture evidence refresh. |
| `git diff --check` | Pass | No whitespace errors are present in the command-palette helper extraction diff. |

Recent item store helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after extracting recent file/folder/list dedupe and forget helpers to `src/lib/recentItems.ts`. |
| `pnpm run test:unit` | Pass | 71 frontend unit/static tests passed, including direct recent-item deduplication, limit enforcement, whitespace trimming, and forget behavior. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting recent-item helper extraction; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the state-store evidence refresh. |
| `git diff --check` | Pass | No whitespace errors are present in the recent-item helper extraction diff. |

Watch path helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after extracting watcher path normalization, path equivalence, and watched-role map construction to `src/lib/watchPaths.ts`. |
| `pnpm run test:unit` | Pass | 72 frontend unit/static tests passed, including direct Windows/POSIX watch-path normalization, equivalence, and role lookup key coverage. |
| `pnpm run check:docs` | Pass | 14 Markdown files were checked after documenting watch-path helper extraction; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the state-store evidence refresh. |
| `git diff --check` | Pass | No whitespace errors are present in the watch-path helper extraction diff. |

Homebrew distribution packaging gate:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:homebrew` | Pass | Homebrew cask contract passed with zero configuration issues and wrote `.tmp/homebrew/homebrew-packaging-report.json`; the report correctly keeps release blockers open for final cask replacement, signed/notarized macOS artifact proof, and cask/artifact SHA evidence. |
| `pnpm run check:docs` | Pass | 15 Markdown files were checked after adding the Homebrew distribution runbook and README links; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the Homebrew packaging evidence refresh. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding the Homebrew packaging scripts and package metadata. |
| `pnpm run test:unit` | Pass | 72 frontend unit/static tests passed after adding the Homebrew gate to local verification. |
| `pnpm run check:release-ci` | Pass | Release CI workflow guard refreshed after `package.json` changed and wrote `.tmp/release-ci/workflow-report.json`. |
| `pnpm run check:release-readiness` | Pass | Release readiness aggregation returned `current-host-ready-with-external-gaps`, now including Homebrew blockers as explicit release risks rather than accepting an unsigned placeholder cask. |
| `git diff --check` | Pass | No whitespace errors are present in packaging, docs, or scripts. |

Configuration setup, LLM defaults, and read-aloud:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding the configuration setup wizard, persisted non-secret LLM defaults, Google Antigravity handoff routing, and TTS settings/actions. |
| `pnpm run test:unit` | Pass | 74 frontend unit/static tests passed, including AI provider defaults, TTS preference normalization, setup wizard wiring, read-aloud controls, native menu wiring, and persisted migration coverage. |
| `cargo test --locked tts --lib` | Pass | Rust TTS command-builder tests passed for Supertonic CLI argument shaping and macOS Say stdin-based speech input. |
| `cargo test --locked ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented --lib` | Pass | IPC registration and documentation coverage passed after adding `read_text_aloud` and `stop_text_aloud` plus the previously registered RFP/local-agent/native-smoke commands to the coverage ledger. |
| `pnpm run check:docs` | Pass | 15 Markdown files were checked after documenting setup/read-aloud behavior and IPC coverage; all local links resolved. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the preferences and command-palette evidence refresh. |
| `git diff --check` | Pass | No whitespace errors are present in the setup, TTS, tests, or docs diff. |

Native TTS runtime inspection:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding native TTS runtime inspection status to Settings, the setup wizard, and command palette. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with guards for `inspect_native_tts`, Check TTS runtime commands, and the text-to-speech runtime report UI. |
| `cargo test --locked tts --lib` | Pass | Rust TTS tests passed for browser/native inspection, Supertonic command availability reporting without process launch, macOS Say stdin handling, and Supertonic argument shaping. |
| `cargo test --locked ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented --lib` | Pass | IPC registration and documentation coverage passed after adding `inspect_native_tts` to the coverage ledger. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the TTS runtime inspection proof. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after refreshing Preferences evidence for native TTS inspection. |
| `git diff --check` | Pass | No whitespace errors are present in the native TTS inspection diff. |

Configuration Center consolidation:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after consolidating Settings behind a single Configuration Center with overview, appearance/editor, files/history, exports/brand, AI/agents/voice, and transforms sections. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with guards for Configuration Center navigation, section labels, setup wiring, and the existing AI/TTS configuration controls. |
| `pnpm run check:docs` | Pass | Markdown links resolved after refreshing the Preferences completion evidence for the consolidated setup surface. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after adding Configuration Center evidence to Preferences. |
| `git diff --check` | Pass | No whitespace errors are present in the Configuration Center diff. |

Configuration setup assistance:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding AI setup assistance, editable setup notes, and static UI wiring to the configuration setup wizard. |
| `pnpm run check` | Pass | Vue typecheck passed after rendering setup assistance and setup-note acceptance controls across configuration areas. |
| `pnpm run check:docs` | Pass | Markdown docs were checked after documenting configuration setup assistance; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with configuration setup assistance evidence recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the configuration setup assistance update. |
| `git diff --check` | Pass | No whitespace errors after the configuration setup assistance update. |

Export readiness assistance:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 75 frontend unit/static tests passed after adding export step assistance, editable export readiness notes, note insertion, and static UI wiring. |
| `pnpm run check` | Pass | Vue typecheck passed after rendering export assistance and note acceptance controls in the Export panel. |
| `pnpm run check:docs` | Pass | Markdown docs were checked after documenting export readiness assistance; local links resolve. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` with export readiness assistance evidence recorded. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` after the export readiness assistance update. |
| `git diff --check` | Pass | No whitespace errors after the export readiness assistance update. |

Consent-gated TTS model downloads:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding the Supertonic-only model download panel, explicit acknowledgement state, model size/location disclosure, and command-palette entries. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with guards for TTS model download UI, persisted acknowledgement/storage fields, download command wiring, and existing read-aloud setup controls. |
| `cargo test --locked tts --lib` | Pass | Rust TTS tests passed for argument-safe native engines, Supertonic playback refusal before model-download acknowledgement, explicit model download command shaping, native inspection, and macOS Say stdin handling. |
| `cargo test --locked ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented --lib` | Pass | IPC registration and documentation coverage passed after adding `download_tts_model` to the coverage ledger. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting TTS model download consent gating. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after refreshing Preferences evidence for model-backed TTS consent. |
| `git diff --check` | Pass | No whitespace errors are present in the TTS model download consent diff. |

Spreadsheet table exchange and SQL transform:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding table import/export controls, command-palette entries, SQL transform insertion, and spreadsheet exchange IPC wiring. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with guards for CSV/XLSX import/export actions, SQL transform insertion, and SQL template registration. |
| `cargo test --locked table_tests::spreadsheet_table_import_export_round_trips_csv_and_xlsx --lib` | Pass | Rust table tests proved native CSV import, XLSX export, XLSX import, and CSV export through the new spreadsheet exchange commands. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked sql_transform_requires_read_only_trusted_queries --lib` | Pass | Rust transform tests proved SQL mutation blocking, stacked `SELECT; DELETE` and `WITH ... SELECT; UPDATE` rejection, quoted-keyword and identifier false-positive avoidance, and trust gating for sqlite-backed SQL transforms. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked sql_transform_runs_trusted_sqlite_query_against_document_relative_database --lib` | Pass | Focused Rust test created a real SQLite database under the document folder, configured the installed `sqlite3` executable as a trusted SQL engine, resolved `database="data/revenue.sqlite"` relative to the Markdown file, ran a read-only `SELECT`, and rendered the live query result as a `transform-sql` table without error diagnostics. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked sql_transform --lib` | Pass | Focused SQL transform suite now also proves document-relative SQL database paths that escape with `..` are blocked before sqlite3 can be invoked, while the live trusted query path still renders successfully. |
| `cargo test --locked ipc_command_tests::spec_25_4_ipc_commands_are_registered_and_documented --lib` | Pass | IPC registration and documentation coverage passed after adding `import_spreadsheet_table` and `export_markdown_tables` to the coverage ledger. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting spreadsheet exchange and SQL transform evidence. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after refreshing Tables/data and Later transforms evidence. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting remains clean after resolving document-relative SQL database paths. |
| `git diff --check` | Pass | No whitespace errors are present in the spreadsheet exchange and SQL transform diff. |

`ned` document inspection:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed for `ned inspect` JSON document inventory, stdin inspection, shell completion discovery, and help text coverage while preserving open/new/convert/validate/doctor workflows. |
| `src-tauri/target/debug/ned inspect README.md --json` | Pass | Direct smoke returned `neditor.ned-inspect.v1` with title/status, word and line counts, outline headings, diagnostics, export targets, and no artifact write. |
| `printf '# Inspect Pipe\n\nHello from stdin.\n' \| src-tauri/target/debug/ned inspect - --json` | Pass | Direct pipe smoke returned stdin inspection with `sourcePath: null`, heading inventory, diagnostics, and export target discovery. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding Settings copy for the no-write document inventory command. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Settings command-line guidance refresh. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting `ned inspect` in the README. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after adding CLI inspection evidence to local file and Settings rows. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after the CLI extension. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after the CLI extension. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`, preserving external signing, notarization, and cross-host proof blockers. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the CLI implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the `ned inspect` diff. |

`ned` project workspace bootstrap:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed for `ned init` `.neditor` workspace scaffolding, idempotent reruns, dry-run behavior, shell completions, and help text coverage. |
| `src-tauri/target/debug/ned init /private/tmp/neditor-ned-init-smoke-codex --force --json` | Pass | Direct smoke created `.neditor/README.md`, `.neditor/variables.yaml`, `.neditor/snippets/business.md`, and `.neditor/agent-handoffs/.gitkeep` with `neditor.ned-init.v1` JSON evidence. |
| `src-tauri/target/debug/ned init /private/tmp/neditor-ned-init-smoke-dry --dry-run` | Pass | Direct dry-run smoke reported planned scaffold files without creating the target workspace. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding Settings copy for `ned init`. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Settings command-line guidance refresh. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting `ned init` in the README. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after adding CLI bootstrap evidence to local file and Settings rows. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after the CLI bootstrap extension. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after the CLI bootstrap extension. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`, preserving external signing, notarization, and cross-host proof blockers. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the CLI bootstrap implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the `ned init` diff. |

`ned` transform handler setup discovery:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed for `ned handlers` JSON setup reports, Windows copyable command output, Linux copy-only guidance, completions, and help text coverage. |
| `src-tauri/target/debug/ned handlers --platform macos --json` | Pass | Direct smoke returned `neditor.ned-handlers.v1` with every registered external engine covered by the Homebrew setup plan. |
| `src-tauri/target/debug/ned handlers --platform windows --commands-only` | Pass | Direct smoke returned copyable winget commands plus `cargo install pikchr-cli --locked` without starting installers. |
| `src-tauri/target/debug/ned handlers --platform linux` | Pass | Direct smoke returned copy-only Linux package guidance and explicit notes about terminal execution, trust, and probe follow-up. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding Settings copy for transform handler CLI setup discovery. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Settings command-line guidance refresh. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting `ned handlers` in the README. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after adding CLI handler discovery evidence to local file, Settings, and Windows setup rows. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after the CLI handler discovery extension. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after the CLI handler discovery extension. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`, preserving external signing, notarization, and cross-host proof blockers. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the CLI handler discovery implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the `ned handlers` diff. |

`ned` workspace-aware setup doctor:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed for `ned doctor --workspace` JSON setup reporting, uninitialized scaffold recommendations, initialized scaffold readiness, transform-handler coverage, completions, and help text coverage. |
| `src-tauri/target/debug/ned doctor --workspace /private/tmp/neditor-doctor-uninitialized --json` | Pass | Direct smoke returned `neditor.ned-doctor.v1` with app binary/default-reader details, transform handler coverage, workspace scaffold status `workspace-missing`, missing `.neditor` files, and the exact `ned init ... --json` recommendation. |
| `src-tauri/target/debug/ned init /private/tmp/neditor-doctor-ready --force --json` | Pass | Direct setup smoke created the expected `.neditor` workspace scaffold for the ready-doctor proof. |
| `src-tauri/target/debug/ned doctor --workspace /private/tmp/neditor-doctor-ready` | Pass | Direct text smoke reported the workspace scaffold as ready and transform handler setup coverage as complete while preserving host-specific default-reader warning details. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after updating Settings copy for workspace-aware doctor diagnostics. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Settings command-line guidance refresh. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting `ned doctor --workspace . --json` in the README. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after adding workspace-aware doctor evidence to local file and Settings rows. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after the CLI doctor extension. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after the CLI doctor extension. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`, preserving external signing, notarization, and cross-host proof blockers. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the CLI doctor implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the `ned doctor` diff. |

`ned doctor --strict` workspace gate:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after promoting missing/incomplete workspace scaffolds into doctor warnings and preserving ready-workspace output without scaffold warnings. |
| `src-tauri/target/debug/ned doctor --workspace /private/tmp/neditor-doctor-strict-missing --strict --json` | Pass | Direct smoke exited non-zero and returned `neditor.ned-doctor.v1` with `Workspace scaffold is workspace-missing` plus the exact `ned init ... --json` remediation command. |
| `src-tauri/target/debug/ned init /private/tmp/neditor-doctor-strict-ready --force --json` | Pass | Direct setup smoke created the expected `.neditor` scaffold for the ready strict-doctor proof. |
| `src-tauri/target/debug/ned doctor --workspace /private/tmp/neditor-doctor-strict-ready --json` | Pass | Direct smoke reported the workspace scaffold as ready and omitted workspace-scaffold warnings while preserving host-specific default-reader warning details. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after documenting strict workspace setup behavior. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the README-only strict-mode clarification. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting strict workspace setup behavior in the README. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after the strict-doctor code and docs update. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after the strict doctor gate. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after the strict doctor gate. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`, preserving external signing, notarization, and cross-host proof blockers. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the strict doctor implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the strict doctor diff. |

`ned readiness` release report inspection:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed for `ned readiness` and `ned release-readiness` JSON/text report reading, custom `--report` fixtures, strict non-zero behavior when evidence gaps remain, ready-report strict pass behavior, shell completions, and help text coverage. |
| `src-tauri/target/debug/ned readiness --json` | Pass | Direct smoke returned `neditor.ned-readiness.v1` from `.tmp/release-readiness/report.json` with `releaseReady: false`, 26 accepted required checks, zero failed checks, 16 evidence gaps, and next commands for release-readiness refresh and evidence-kit collection. |
| `src-tauri/target/debug/ned readiness --strict` | Expected non-zero | Direct strict smoke exited `1` because the current report remains `current-host-ready-with-external-gaps` and lists the 16 external evidence gaps that still block publication. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding Settings copy for release-readiness CLI report inspection. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Settings command-line guidance refresh. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting `ned readiness` in the README and specification. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after adding CLI release-readiness evidence to the packaging contract. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after the CLI readiness report extension. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after the CLI readiness report extension. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`, preserving external signing, notarization, Google Docs, AI provider/runtime, cross-host, human sign-off, and optional engine evidence gaps. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the readiness CLI implementation. |

`ned support-bundle` help desk handoff:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed for `ned support` and `ned support-bundle` JSON output, file-output mode, redaction-safe privacy fields, embedded doctor reports, embedded release-readiness summaries, shell completions, and help text coverage. |
| `src-tauri/target/debug/ned support-bundle --workspace . --json` | Pass | Direct smoke returned `neditor.ned-support-bundle.v1` with no document content or secrets, `ned doctor` setup diagnostics, the current release-readiness summary, 16 external evidence gaps, and actionable support recommendations. |
| `src-tauri/target/debug/ned support-bundle --workspace . --output /private/tmp/neditor-support-bundle-smoke.json` | Pass | Direct smoke wrote a machine-readable support bundle and printed a concise support handoff summary with doctor status, release-readiness status, evidence-gap count, and privacy statement. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding Settings copy for support-bundle help desk handoffs. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Settings command-line guidance refresh. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting `ned support-bundle` in the README and specification. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after adding the CLI support-bundle packaging contract. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after the support-bundle CLI extension. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after the support-bundle CLI extension. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` with the external proof blockers preserved. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the support-bundle CLI implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the support-bundle diff. |

Settings support-bundle action:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after exposing the support-bundle builder as the `create_support_bundle` Tauri command and covering IPC-style file output through the same redaction-safe contract. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked spec_25_4_ipc_commands_are_registered_and_documented --lib` | Pass | IPC coverage ledger now documents `pending_cli_open_paths`, default-reader setup commands, and `create_support_bundle`, and remains synchronized with `tauri::generate_handler!`. |
| `src-tauri/target/debug/ned support-bundle --workspace . --json` | Pass | Direct smoke still returns `neditor.ned-support-bundle.v1` with setup diagnostics, release-readiness status, 16 external evidence gaps, recommendations, and no document content or secrets. |
| `src-tauri/target/debug/ned support-bundle --workspace . --output /private/tmp/neditor-settings-support-bundle-smoke.json` | Pass | Direct smoke wrote the support bundle JSON after the IPC refactor and printed the concise handoff summary used by Settings. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "creates support bundle handoff from settings"` | Pass | Focused Chromium smoke verified Settings -> Files and history exposes Support bundle, Preview, Save JSON, privacy copy, preview status, recommendations, and saved output path rendering. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding Settings preview/save controls for support bundles. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed for the Settings support-bundle UI, request typing, and status/report rendering. |
| `pnpm run check:docs` | Pass | Markdown links resolved after updating the IPC coverage ledger and specification. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` after adding the app-facing support-bundle IPC contract. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after exposing support bundle creation from Settings. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after exposing support bundle creation from Settings. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` with all current-host checks accepted and external proof blockers preserved. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the support-bundle IPC refactor. |
| `git diff --check` | Pass | No whitespace errors are present in the Settings support-bundle diff. |

Support bundle spec-completion summary:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after adding `--spec-report` and `spec_report` request support to `ned support-bundle` and `create_support_bundle`, including fixture coverage for open spec rows and text output. |
| `src-tauri/target/debug/ned support-bundle --workspace . --json` | Pass | Direct smoke returned `specCompletion` with `.tmp/spec-completion/report.json`, status `partial-with-release-risks`, 106 open rows, and capped open-row details while preserving redaction-safe privacy fields. |
| `src-tauri/target/debug/ned support-bundle --workspace . --output /private/tmp/neditor-spec-support-bundle-smoke.json` | Pass | Direct text smoke reported `Spec completion: partial-with-release-risks (106 open rows)` and wrote the support bundle JSON. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "creates support bundle handoff from settings"` | Pass | Focused Chromium smoke verified Settings support-bundle preview and saved-output rendering now include the 106 open spec rows count. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding spec-completion support bundle UI and static wiring checks. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed for support-bundle spec-completion report typing and rendering. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 65 tests after stabilizing Settings navigation and support-bundle mocks for the spec-completion handoff. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting spec-completion summaries in support bundles. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator returned `partial-with-release-risks` and refreshed `.tmp/spec-completion/report.json` for support-bundle ingestion. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after extending support bundles with spec-completion summaries. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after extending support bundles with spec-completion summaries. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` with current-host evidence accepted and external proof blockers preserved. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the support-bundle spec-completion implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the support-bundle spec-completion diff. |

Support bundle transform-engine summary:

| Command | Result | Evidence |
| --- | --- | --- |
| `NEDITOR_TEST_PIKCHR=/Users/nyimbiodero/src/pjs/tooling/neditor/.tmp/pikchr-build/pikchr-trunk/pikchr pnpm run check:engines` | Pass | External transform probe now reports Graphviz, D2, PlantUML, Pikchr, and SQLite installed on this Darwin arm64 host with smoke artifacts; the optional Pikchr gap is closed in the current release-readiness report. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`, but evidence gaps dropped from 17 to 15 after current-host Pikchr proof and the dependent Homebrew release-readiness gate were refreshed. |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after adding `--engine-report` and `engine_report` request support to `ned support-bundle` and `create_support_bundle`, including fixture coverage for transform-engine summary rows, text output, and shell completions. |
| `src-tauri/target/debug/ned support-bundle --workspace . --json` | Pass | Direct smoke returned `engineProbe` with `.tmp/external-engines/probe-report.json`, status `complete`, 10 installed engines, zero missing or incompatible engines, and bounded engine details while preserving the redaction-safe support-bundle contract. |
| `src-tauri/target/debug/ned support-bundle --workspace . --output /private/tmp/neditor-engine-support-bundle-smoke.json` | Pass | Direct text smoke reported 15 release evidence gaps plus `Transform engines: complete (10 installed, 0 missing, 0 incompatible)` and wrote the support bundle JSON. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked spec_25_4_ipc_commands_are_registered_and_documented --lib` | Pass | IPC coverage ledger remains synchronized after documenting the expanded `create_support_bundle` evidence contract. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "creates support bundle handoff from settings"` | Pass | Focused Chromium smoke verified Settings support-bundle preview and saved-output rendering now include transform-engine health next to release and spec-completion status. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding transform-engine support bundle UI and static wiring checks. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed for support-bundle transform-engine report typing and rendering. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 65 tests after refreshing the shared browser workflow report for release-readiness acceptance. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting transform-engine summaries in support bundles. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while this slice improves support and release evidence visibility. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the support-bundle transform-engine implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the support-bundle transform-engine diff. |

Support bundle release-evidence report summary:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after adding `--evidence-root` and `evidence_root` request support to `ned support-bundle` and `create_support_bundle`, including fixture coverage for ready, attention, and missing evidence-report buckets plus text output and shell completions. |
| `src-tauri/target/debug/ned support-bundle --workspace . --json` | Pass | Direct smoke returned `evidenceReports` and `evidenceReportSummary` for platform, signing, Google Docs, Homebrew, AI provider, AI runtime, performance profile, security review, rendered-export sign-off, and accessibility sign-off reports without including document bodies or secrets. |
| `src-tauri/target/debug/ned support-bundle --workspace . --output /private/tmp/neditor-evidence-support-bundle-smoke.json` | Pass | Direct text smoke reported 15 release evidence gaps, `Evidence reports: 0 ready, 10 need attention, 0 missing`, and the recommendation to collect or refresh 10 release evidence reports. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked spec_25_4_ipc_commands_are_registered_and_documented --lib` | Pass | IPC coverage ledger remains synchronized after documenting the release evidence report summary in the `create_support_bundle` contract. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "creates support bundle handoff from settings"` | Pass | Focused Chromium smoke verified Settings support-bundle preview now displays release evidence report readiness next to release, spec, and transform-engine health. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed after adding release evidence report summary UI and static wiring checks. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed for support-bundle release evidence report typing and rendering. |
| `pnpm run test:e2e` | Pass | Full Chromium workflow suite passed with 65 tests after refreshing the shared browser workflow report for release-readiness acceptance. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting release evidence report summaries in support bundles. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while this slice improves support and release evidence visibility. |
| `pnpm run check:platform-packaging` | Pass | Platform package configuration remains valid after extending support bundles with release evidence report summaries. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after extending support bundles with release evidence report summaries. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps` with 15 evidence gaps and zero failed checks. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the support-bundle release evidence report implementation. |
| `git diff --check` | Pass | No whitespace errors are present in the support-bundle release evidence report diff. |

`ned evidence` release-evidence report inspection:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after adding `ned evidence` / `ned evidence-status`, `--json`, `--strict`, and `--evidence-root` support, including fixture coverage for ready, attention, missing, strict-failure, completion, help, and nested rendered-export human sign-off status handling. |
| `src-tauri/target/debug/ned evidence --json` | Pass | Direct smoke returned schema `neditor.ned-evidence-status.v1`, status `needs-attention`, 10 tracked evidence reports, and the nested rendered-export status `pending-human-review` instead of a generic present marker. |
| `src-tauri/target/debug/ned evidence --strict` | Pass with expected nonzero exit | Direct strict smoke exited `1` because the current host still has 10 release evidence reports needing attention; text output included next commands for collecting/ingesting evidence and rerunning release readiness. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the dedicated evidence command implementation. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding Settings guidance for `ned evidence --json`. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 74 tests, including static assertions that CLI docs and Settings expose the evidence command contract. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the evidence command in business-friendly README guidance. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while the CLI now exposes release-evidence status directly. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration remains valid after extending the packaged `ned` surface. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after extending the packaged `ned` surface. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`; this slice improves evidence inspection without claiming external proof completion. |
| Browser e2e suite | Not rerun | Skipped to conserve battery because this slice adds terminal behavior, completion/help text, README/spec copy, and static Settings guidance; no interactive workflow behavior changed. |

`ned default-reader` scriptable setup reporting:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after adding `ned default-reader --status --json`, schema `neditor.ned-default-reader.v1`, unsupported-option validation, status text coverage, completion/help updates, and rebuilt the `ned` binary. |
| `src-tauri/target/debug/ned default-reader --status --json` | Pass | Direct smoke returned platform, automation support, copyable commands, manual setup steps, next commands, and status `manual-setup-required` on this macOS host where `duti` is not installed. |
| `src-tauri/target/debug/ned default-reader --status` | Pass | Direct text smoke now starts with `Default Markdown reader: manual-setup-required` and lists the same safe commands/manual steps for non-technical setup. |
| `src-tauri/target/debug/ned default-reader --mystery` | Pass with expected CLI error | Direct smoke returned an unsupported-option error instead of silently ignoring an invalid default-reader flag. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the scriptable default-reader reporting implementation. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding Settings guidance for `ned default-reader --status --json`. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 74 tests, including static assertions that Settings and CLI source expose the default-reader JSON contract. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting scriptable default-reader setup status in the business-friendly README. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while packaged CLI default-reader setup reporting is now machine-readable. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration remains valid after extending the packaged `ned` default-reader surface. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after extending the packaged `ned` default-reader surface. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`; this slice improves local setup reporting without claiming external release proof completion. |
| Browser e2e suite | Not rerun | Skipped to conserve battery because this slice adds terminal behavior, completion/help text, README/spec copy, and static Settings guidance; no interactive workflow behavior changed. |

Scriptable `ned open` and `ned new` status:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after adding `--json` to `ned open` and `ned new`, schema checks for `neditor.ned-open.v1` and `neditor.ned-new.v1`, dry-run file handoff coverage, creation status coverage, unsupported open-option validation, completion/help updates, and rebuilt the `ned` binary. |
| `src-tauri/target/debug/ned open README.md --dry-run --json` | Pass | Direct smoke returned schema `neditor.ned-open.v1`, one canonical Markdown path, `dryRun: true`, and `opened: false` without launching the app. |
| `src-tauri/target/debug/ned new /private/tmp/neditor-ned-json-smoke.md --template proposal --title "CLI JSON Smoke" --force --json` | Pass | Direct smoke created a proposal document in `/private/tmp` and returned schema `neditor.ned-new.v1`, selected template, resolved title, output path, `created: true`, and `opened: false`. |
| `src-tauri/target/debug/ned open README.md --mystery` | Pass with expected CLI error | Direct smoke returned an unsupported-option error instead of treating an invalid flag as a path. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the scriptable open/new CLI implementation. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding Settings guidance for scriptable `ned open` and `ned new` usage. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 74 tests, including static assertions that Settings and CLI source expose open/new JSON schemas. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting scriptable file handoff and document creation in the README. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while packaged CLI file open and business-template creation are now machine-readable. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration remains valid after extending the packaged `ned` open/new surface. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after extending the packaged `ned` open/new surface. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`; this slice improves local CLI automation without claiming external release proof completion. |
| Browser e2e suite | Not rerun | Skipped to conserve battery because this slice adds terminal behavior, completion/help text, README/spec copy, and static Settings guidance; no interactive workflow behavior changed. |

Expanded `ned new` business and publishing starters:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after expanding `ned new --template` to 17 starters, including RFP, RFQ, tender, tutorial, lesson content, technical textbook, podcast script, movie script, business case, and executive brief coverage with JSON creation assertions and placeholder checks. |
| `src-tauri/target/debug/ned templates --json` | Pass | Direct smoke returned schema `neditor.ned-templates.v1` with the expanded starter catalog: blank, proposal, RFP, RFP response, RFQ, tender, report, tutorial, lesson plan, lesson content, textbook, technical textbook, novel, podcast script, movie script, business case, and executive brief. |
| `src-tauri/target/debug/ned new /private/tmp/neditor-tender-smoke.md --template tender --title "Tender Smoke" --force --json` | Pass | Direct smoke created a tender starter and returned schema `neditor.ned-new.v1`, selected template, resolved title, output path, `created: true`, and `opened: false`. |
| `rg -n "documentType: tender\|Evaluation Method\|Instructions To Tenderers" /private/tmp/neditor-tender-smoke.md` | Pass | Generated tender Markdown contains the expected front matter and procurement sections. |
| `src-tauri/target/debug/ned new /private/tmp/neditor-podcast-smoke.md --template podcast-script --title "Podcast Smoke" --force --json` | Pass | Direct smoke created a podcast script starter and returned schema `neditor.ned-new.v1`, selected template, resolved title, output path, `created: true`, and `opened: false`. |
| `rg -n "documentType: podcast-script\|Segment Rundown\|Production Notes" /private/tmp/neditor-podcast-smoke.md` | Pass | Generated podcast Markdown contains the expected front matter and production sections. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for the expanded template catalog. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after updating Settings CLI examples for tender and podcast starters. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 74 tests, including static assertions that Settings points users at the broader CLI starter catalog. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the expanded starter catalog in README and specification. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while packaged CLI business-template creation now covers more requested document types. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration remains valid after expanding the packaged `ned` template catalog. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after expanding the packaged `ned` template catalog. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`; this slice improves local document-creation capability without claiming external release proof completion. |
| Browser e2e suite | Not rerun | Skipped to conserve battery because this slice adds terminal behavior, completion/help text, README/spec copy, and static Settings guidance; no interactive workflow behavior changed. |

Rich `ned templates` discovery:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after replacing the generic template list with rich metadata, category/query filters, `--ids-only`, JSON `templateDetails`, completion/help updates, and filtered discovery assertions. |
| `src-tauri/target/debug/ned templates --category Procurement --query quote --json` | Pass | Direct smoke returned schema `neditor.ned-templates.v1`, normalized filters, `count: 1`, template id `rfq`, and metadata with label, category, summary, and best-fit uses. |
| `src-tauri/target/debug/ned templates --category Media --ids-only` | Pass | Direct smoke returned only `podcast-script` and `movie-script` for script-friendly filtered discovery. |
| `src-tauri/target/debug/ned templates --query executive` | Pass | Direct text smoke returned a readable filtered catalog with template IDs, categories, labels, summaries, and the `ned new <file.md> --template <id> --json` next step. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for rich template discovery. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after updating Settings CLI guidance for filtered template discovery. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 74 tests, including static assertions that Settings and CLI source expose filtered template discovery and metadata. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting filtered template discovery in README and specification. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while packaged CLI template discovery is now stronger for non-technical users and support scripts. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration remains valid after extending the packaged `ned templates` surface. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after extending the packaged `ned templates` surface. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`; this slice improves local template discovery without claiming external release proof completion. |
| Browser e2e suite | Not rerun | Skipped to conserve battery because this slice adds terminal behavior, completion/help text, README/spec copy, and static Settings guidance; no interactive workflow behavior changed. |

Packaged `ned snippets` reusable document parts:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:cli` | Pass | Rust CLI tests passed after adding `ned snippets` / `ned parts`, kind/query filters, `--ids-only`, `--markdown`, JSON `snippetDetails`, completion/help updates, and body-output assertions for reusable document parts. |
| `src-tauri/target/debug/ned snippets --kind procurement --json` | Pass | Direct smoke returned schema `neditor.ned-snippets.v1`, `count: 2`, procurement snippet metadata, and copyable bodies for RFP compliance matrix and tender submission checklist. |
| `src-tauri/target/debug/ned snippets --query risk --ids-only` | Pass | Direct smoke returned only `risk-register` for script-friendly snippet lookup. |
| `src-tauri/target/debug/ned snippets --markdown review-handoff` | Pass | Direct smoke printed the reusable review handoff Markdown with placeholders preserved. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed for packaged snippet discovery and Markdown output. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after updating Settings CLI guidance for snippet output. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 74 tests, including static assertions that Settings and CLI source expose snippet discovery and metadata. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting packaged snippet discovery and Markdown output in README and specification. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while reusable document parts are now available from the packaged CLI. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration remains valid after extending the packaged `ned snippets` surface. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after extending the packaged `ned snippets` surface. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`; this slice improves local document-part reuse without claiming external release proof completion. |
| Browser e2e suite | Not rerun | Skipped to conserve battery because this slice adds terminal behavior, completion/help text, README/spec copy, and static Settings guidance; no interactive workflow behavior changed. |

Creative content wizard staging:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 75 tests after making podcast and movie script wizards lock episode/screen-story architecture before prose, generate sequential segment/beat draft queues, add final audio-production/screenplay quality review gates, update business-template approval gates, expose matching Agent Workspace quality gates, and add context-aware suggested answers for every Docs Live questionnaire step. |
| Browser e2e suite | Not rerun | Skipped to conserve battery because this slice changes shared Docs Live draft generation, business-template metadata, agentic workflow planning logic, and static Docs Live UI wiring covered by the focused frontend unit suite. |

Table command-surface completion:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after exposing source-table editing through toolbar, visible Writing Tools menu, command palette, and native menu bridge. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 75 tests, including static guards for Edit Table at Cursor, Open Table Editor, Go to Source Table, CSV/XLSX table export, new table drafts, and matching native menu command handlers. |
| `pnpm run test:e2e` | Pass | Full Chromium browser workflow suite passed with 65 tests after adding Writing Tools menu assertions for Edit Table at Cursor and Go to Source Table and tightening strict selectors for exact headings, readiness diagnostics, and template cards. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting check passed after adding native Writing Tools table-edit menu items. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting table command-surface access in README, user guide, and Markdown extensions reference. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while recording that two-way table workflows are now discoverable from all main command surfaces. |
| `pnpm run check:release-readiness` | Pass | Release readiness is `current-host-ready-with-external-gaps` after refreshing the full browser workflow evidence for the changed UI/spec sources. |
| `git diff --check` | Pass | No whitespace errors are present in the table command-surface diff. |

Native table menu workflow proof:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run build` | Pass | Vue/TypeScript validation and production Vite build passed before rebuilding the Tauri binary for native smoke proof. |
| `pnpm tauri build --no-bundle` | Pass | Rebuilt `src-tauri/target/release/neditor` with the updated frontend and native menu workflow hooks. |
| `NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke` | Pass | Bounded native launch smoke passed outside the sandbox and `.tmp/desktop-smoke/native-workflow-report.json` now records native Writing Tools menu evidence for opening the table editor, loading the source table at the cursor into a draft containing the Revenue row, and jumping back to the source Markdown table range. |
| `pnpm run test:desktop-smoke` | Pass | Non-launch desktop smoke validates the refreshed native workflow report and command workflow after adding the table-menu assertions. |
| `pnpm run test:tauri-webdriver` | Pass | macOS WebDriver harness correctly skipped with fresh native fallback proof for the rebuilt binary. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after adding native table menu smoke evidence and the review-mode sentinel update. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 75 tests, including static guards for the native table-menu smoke evidence and desktop smoke validator expectations. |
| `pnpm run check:release-readiness` | Pass | Release readiness returned `current-host-ready-with-external-gaps` after refreshing native smoke and WebDriver fallback evidence for the rebuilt binary. |
| `git diff --check` | Pass | No whitespace errors are present in the native table menu proof diff. |

Button-help tooltip semantics:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:a11y` | Pass | Static accessibility guardrails passed after requiring the delegated button-help tooltip id, `aria-describedby` relationship creation, relationship cleanup, dynamic CodeMirror default editor label recognition, and labels for the business-placeholder preview plus Docs Live suggested-answer controls. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "shows delegated button help on hover and focus"` | Pass | Focused Chromium workflow passed and now proves the New, Commands, Help, and disabled Export CSV buttons receive `aria-describedby="button-help-tooltip"` while their help is visible and lose it when the help is hidden or handed off. |
| `pnpm run check:a11y:runtime` | Pass | Runtime accessibility audit passed all 7 focused Chromium workflows, including the delegated button-help workflow, and rewrote `.tmp/accessibility/runtime-report.json`. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 76 tests, including table source-sync helpers and accessibility guard assertions for button-help relationship creation/cleanup. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after correcting the Docs Live suggested-answer control labels. |
| `pnpm run check:docs` | Pass | Markdown links resolved after the progress and spec evidence updates. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while recording the stronger button-help accessibility proof. |
| `pnpm run test:e2e` | Pass | Full Chromium browser workflow suite passed with 66 tests after adding the tooltip relationship assertions to the runtime accessibility workflow. |
| `git diff --check` | Pass | No whitespace errors are present in the button-help semantics diff. |

Scriptable business profile field discovery:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting passed after adding `ned profile --fields` and `--get`. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked cli_tests --lib` | Pass | Focused Rust CLI test module passed with 21 tests, including reusable business profile field discovery, alias metadata, single-value retrieval, placeholder fallback for unset values, completions, and existing profile update/dry-run behavior. |
| `pnpm run check:cli` | Pass | CLI gate passed and rebuilt the `ned` binary after adding `neditor.ned-profile-fields.v1` and `neditor.ned-profile-value.v1` outputs. |
| `src-tauri/target/debug/ned profile --fields --json` | Pass | Direct smoke returned all reusable profile fields, aliases, labels, intended uses, and schema `neditor.ned-profile-fields.v1`. |
| `src-tauri/target/debug/ned profile --workspace /private/tmp/neditor-profile-smoke --init --set fullName='Jane Doe' --set companyName='Acme Advisory' --json` | Pass | Direct smoke initialized `.neditor/business-profile.json` and returned schema `neditor.ned-profile.v1` with reusable identity JSON, Markdown, and placeholder text. |
| `src-tauri/target/debug/ned profile --workspace /private/tmp/neditor-profile-smoke --get companyName --json` | Pass | Direct smoke returned schema `neditor.ned-profile-value.v1`, canonical field `companyName`, value `Acme Advisory`, and matching placeholder output for scriptable setup checks. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after updating Settings command-line guidance. |
| `pnpm run test:unit` | Pass | Frontend unit/static tests passed with 76 tests, including Settings guidance guards for `ned profile --fields --json` and `ned profile --workspace . --get companyName`. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting profile field discovery and single-value retrieval in README. |
| `pnpm run check:spec-completion` | Pass | Spec completion matrix validator remains `partial-with-release-risks` while recording stronger CLI setup evidence in the Settings row. |
| `pnpm run check:platform-packaging` | Pass | Cross-platform package configuration remains valid after extending the packaged `ned` profile command. |
| `pnpm run check:homebrew` | Pass | Homebrew cask packaging contract remains valid after extending the packaged `ned` profile command. |
| `pnpm run check:release-readiness` | Pass | Release readiness remains `current-host-ready-with-external-gaps`; this slice improves local setup automation without claiming external release proof completion. |
| Browser e2e suite | Not rerun | Skipped to conserve battery because this slice changes terminal behavior, completion/help text, README/spec copy, and static Settings guidance rather than interactive workbench behavior. |
| `git diff --check` | Pass | No whitespace errors are present in the profile field-discovery diff. |

Two-way table source editing guard:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 76 frontend unit/static tests passed after guarding table context switches and adding static checks for the table source-switch guard and selector handler. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after flushing editor source before table apply, preventing existing-table source edits from falling through to new-table insertion, and snapshotting newly inserted tables immediately. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow now proves editable Markdown table source disables context-switch actions while dirty, parses typed source into the visual grid, applies that source back into the existing table, and avoids leaving the replaced captioned table behind. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting guarded two-way table source editing. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording the guarded two-way table editing evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the guarded table source editing diff. |

Agent Workspace AI step assistance browser proof:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "offers searchable contextual help with workflow actions" --project chromium` | Pass | Focused Chromium workflow now proves the visible Agent Workspace **Add answer and replan** action appends the AI suggested answer into editable context answers with rationale and context signals, immediately replans, and keeps the generated agent packet/provider workflow usable. |
| `pnpm run test:unit` | Pass | 76 frontend unit/static tests passed after strengthening browser proof for Agent Workspace step assistance. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the Agent Workspace step-assistance proof. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after marking 11.4 Agentic step assistance complete with direct unit/static and focused browser evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the Agent Workspace step-assistance proof diff. |

Versioning release-tag and Git-history browser proof:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs snapshot restore and release tagging workflows" --project chromium` | Pass | Focused Chromium workflow now proves approved release metadata, manual snapshot creation, snapshot restore, release tag submission, immediate Git status refresh with `tag v2.0.0`, cleared release-tag input, Git-history restore from `001122334455`, and the safety `pre-git-restore` snapshot row before revision restore. |
| `pnpm run test:unit` | Pass | 76 frontend unit/static tests passed after refreshing Git status from the release-tag action and extending versioning browser proof. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after making release tagging refresh the Versioning panel Git state immediately. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting stronger versioning workflow evidence. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording snapshot, release tag, and Git-history restore evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the versioning workflow diff. |

Table editor architecture extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 77 frontend unit/static tests passed after moving table row/column mutation, summary formula insertion, and custom formula-row construction into `src/lib/tables.ts`; the new direct test proves add/remove/duplicate/move row and column behavior plus formula row clamping. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after `App.vue` was reduced to delegating table editor actions through the extracted helpers. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow passed after the UI table editor actions were routed through extracted mutation and formula helpers. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the table editor architecture extraction. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording the added table helper extraction evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the table architecture extraction diff. |

Table merged-cell helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 77 frontend unit/static tests passed after moving merged-cell span preview, apply, and clear behavior into `src/lib/tables.ts`; the table span helper test now proves bounded span preview, draft application, and span clearing. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after `App.vue` delegated merged-cell controls through the table helper module. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "edits pasted tables with sorting, formulas, and merged cells" --project chromium` | Pass | Focused Chromium workflow passed after merged-cell controls delegated through table helpers and the workflow targeted the current editable Markdown source area. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the merged-cell helper extraction. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording the table span helper evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the merged-cell helper extraction diff. |

Table source/default helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 78 frontend unit/static tests passed after moving default table draft creation and editable Markdown source parsing into `src/lib/tables.ts`; the new direct test proves the default scaffold, parsed source draft metadata, inferred numeric format, normalized source text, and invalid-source rejection. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after `App.vue` delegated new-table defaults and Markdown source parsing through table helpers. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow passed after the table editor routed new-table creation and source-text parsing through extracted helpers. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the table source/default helper extraction. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording editable source/default draft helper evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the table source/default helper extraction diff. |

Table paste/import helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 78 frontend unit/static tests passed after routing CSV/TSV/Markdown paste and spreadsheet import Markdown through `tableDraftFromPasteText`; unit coverage now proves TSV fallback metadata and imported Markdown-table caption override behavior. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after `App.vue` delegated paste/import draft creation through the table helper module. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow passed for table creation and editable source parsing after paste/import helper extraction. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "edits pasted tables with sorting, formulas, and merged cells" --project chromium` | Pass | Focused Chromium workflow passed for pasted Markdown tables, sorting, formula rows, and merged cells after paste/import helper extraction. A first parallel attempt collided on the shared Vite port and was rerun sequentially. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the table paste/import helper extraction. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording paste/import draft helper evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the table paste/import helper extraction diff. |

Table accessibility label helper extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 79 frontend unit/static tests passed after moving table data-row counts, formula target options, span-cell options, and table grid header/cell/total label text into `src/lib/tables.ts`; the new direct test proves generated labels for named and unnamed columns. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after `App.vue` delegated table accessibility labels and table option lists through table helpers. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium table workflow passed after table labels and options delegated through helpers. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "supports keyboard-only operation for deep workbench controls" --project chromium` | Pass | Focused Chromium keyboard workflow passed after label extraction, proving table-editor cells/actions remain keyboard reachable. A first parallel attempt collided on the shared Vite port and was rerun sequentially. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting the table accessibility label helper extraction. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording table accessibility label helper evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the table accessibility label helper extraction diff. |

Table dirty source export guard:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 80 frontend unit/static tests passed after adding `tableMarkdownForExport`; direct coverage proves dirty valid Markdown source is selected for CSV/XLSX export, dirty invalid or blank source is blocked, and clean exports still fall back to the visual draft or full document text. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after `App.vue` routed table export through the shared table export-source selector. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium table workflow now proves a user can type a new Markdown table in the editable source block, export CSV before applying/updating the grid, and receive the edited source table rather than the stale visual draft. |
| `pnpm run check:docs` | Pass | Markdown links resolved after documenting dirty editable-source table export behavior. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording the dirty source export guard. |
| `git diff --check` | Pass | No whitespace errors are present in the dirty source export guard diff. |

Direct text table editing hardening:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 80 frontend unit/static tests passed after adding source-snapshot overlap and replacement recovery helpers; direct coverage proves a temporarily invalid first table is not confused with a later parsed table and can be recovered by replacing the original source range. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after routing table source synchronization through `selectedTableForDraft`, overlap checks, invalid-source messaging, and snapshot-range recovery. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow proves direct in-text source editing still syncs clean table edits into the visual grid, keeps the visual draft visible when the separator row is temporarily invalid, and resumes the workflow after the text is fixed. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting in-text table editing recovery. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording source-snapshot overlap and invalid in-text table recovery evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the direct text table editing hardening diff. |

Moved-source table tracking:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 80 frontend unit/static tests passed after adding source-snapshot identity recovery; direct coverage proves a loaded table can be recovered by label/caption identity after another Markdown table is inserted above the original source range. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after routing the visual table editor through identity-based source snapshot matching and synchronizing the selected table index before source reloads. |
| `pnpm exec playwright test e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow now creates a table, inserts another Markdown table above it directly in the text source, keeps the original table loaded in the visual editor, applies source edits back to the intended table, and preserves the inserted table. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting identity-based moved-table tracking. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording moved-table tracking evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the moved-source table tracking diff. |

Inline-code reference readiness hardening:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting is clean after aligning semantic heading extraction, AST label extraction, and cross-reference scanning around inline code spans. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked inline_code_reference_markers_are_treated_as_literal_examples --lib` | Pass | Focused regression proves inline-code examples such as `{@missing}` and `{#fig:literal}` stay literal, do not create labels or broken-reference diagnostics, headings containing code-form label examples keep a slug anchor instead of adopting the example label, and image alt text containing a literal label does not override the real figure label. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked reference --lib` | Pass | 13 reference-related Rust tests passed, covering inline-code literal markers, duplicate labels, malformed labels, prepare-for-export manifest blockers, resolved heading/appendix/decision references, citation fenced examples, named table formulas, topology references, and cross-target reference exports. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked prepare_for_export_blocks_malformed_reference_markers_in_manifest --lib` | Pass | Focused readiness regression still blocks malformed real reference markers and copies diagnostics into export manifests. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked prepare_for_export_blocks_duplicate_reference_labels_in_manifest --lib` | Pass | Focused readiness regression still blocks duplicate real reference labels and preserves manifest audit context. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after documenting inline-code reference readiness hardening. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after recording inline-code reference edge coverage. |
| `git diff --check` | Pass | No whitespace errors are present in the inline-code reference readiness diff. |

OpenAPI/JSON Schema nullable fields:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting is clean after adding nullable schema type and constraint summaries. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked compiler_renders_openapi_and_json_schema_tables --lib` | Pass | Focused schema renderer test proves OpenAPI/JSON Schema tables now surface `nullable: true` and summarize nullable fields as `string | null`. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked json_schema --lib` | Pass | JSON Schema focused test selection passed, covering the renderer path with nullable field evidence. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked api_schema_transforms_survive_cross_target_exports --lib` | Pass | Cross-target API/schema export conformance still passes across HTML, PDF, DOCX, PPTX, and Markdown bundle evidence after nullable support. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after recording nullable schema evidence. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after refreshing OpenAPI/JSON Schema nullable evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the nullable schema diff. |

RFP URL and DOCX source cleanup hardening:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Pass | Rust formatting is clean after hardening native RFP source cleanup. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked rfp_import --lib` | Pass | 5 focused RFP importer tests passed, proving DOCX entity decoding, URL script/style/noise stripping, table/list text preservation, numeric entity decoding, page-title extraction, and non-ASCII-safe case-insensitive tag cleanup for URL-sourced RFPs. |
| `pnpm run check:docs` | Pass | Checked 15 Markdown files; local links resolve after recording non-ASCII-safe RFP URL cleanup evidence. |
| `pnpm run check:spec-completion` | Pass with release risks | Wrote `.tmp/spec-completion/report.json` with status `partial-with-release-risks` after refreshing native RFP URL cleanup evidence. |
| `git diff --check` | Pass | No whitespace errors are present in the RFP URL cleanup hardening diff. |

XLSX front matter data sources:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked data_sources --lib` | Pass | Seven focused Rust data-source tests passed, including `compiler_loads_front_matter_xlsx_data_sources`, proving front matter `dataSources` and `xlsxFiles` can import a local XLSX first worksheet as Markdown table output while preserving include graph and export-manifest evidence. |
| `pnpm run test:unit` | Pass | 80 frontend unit/static tests passed after adding XLSX as a supported local data-source kind in the front matter manager. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after exposing XLSX in data-source UI type options. |

Two-way table text editing:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 80 frontend unit/static tests passed after adding the two-way table strip, **New table in text**/**Insert draft in text** actions, and same-start source snapshot recovery for in-place text edits that change headers and data. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after wiring the table grid/source/document-text focus controls and direct Markdown insertion path. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts -g "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow now proves a user can create a visual draft, insert it as selected Markdown in the document editor, replace that selected table text with edited headers/rows, and see the visual grid sync back from the in-text Markdown table. |

JSON Schema dialect coverage:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked compiler_renders_openapi_and_json_schema_tables --lib` | Pass | Focused compiler transform test passed after adding JSON Schema rendering for boolean schemas, draft-07 tuple `items: [...]`, and boolean `additionalItems`; the generated schema table now shows `any`/`never` rows and explicit boolean-schema constraints instead of blank entries. |

Optional engine evidence collection:

| Command | Result | Evidence |
| --- | --- | --- |
| `NEDITOR_TEST_PIKCHR=.tmp/pikchr-build/pikchr-trunk/pikchr pnpm run collect:engine-evidence` | Pass | The new collector reused the bounded engine smoke probes and wrote accepted `neditor.external-engine-evidence.v1` JSON under `.tmp/external-engines/external/` for Graphviz layout engines, D2, PlantUML, Pikchr, and SQLite. |
| `pnpm run check:engines` | Pass | The normal probe accepted the generated evidence, including `.tmp/external-engines/external/pikchr.json`, so release readiness no longer reports a current-host optional-engine evidence gap when Pikchr is not on `PATH`. |
| `pnpm run test:unit` | Pass | 80 frontend/static tests passed after exposing `collect:engine-evidence` and guarding the evidence-writer path. |
| `pnpm run check:external-transform-docs` | Pass | External transform documentation checks now require the evidence collector instructions. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the package-script and test updates. |

Homebrew readiness gap de-cycling:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run check:homebrew` | Pass with release blockers | The refreshed Homebrew report now carries only the real pending blockers: final cask, signed/notarized macOS artifact, and macOS signing evidence. |
| `pnpm run check:release-readiness` | Pass with external gaps | The readiness aggregator no longer re-imports the self-referential `homebrew-release-readiness` blocker from the Homebrew report; actual readiness failures still fail the aggregation directly. |

Citation style fidelity:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked citation --lib` | Pass | 20 focused Rust citation tests passed after preserving common citation-style intent. APA, Chicago author-date, MLA, IEEE, and Vancouver now keep distinct generated bibliography formatting and citation labels instead of collapsing every CSL alias to generic author-year or numeric output. |
| `pnpm run test:unit` | Pass | 80 frontend unit/static tests passed after adding MLA to supported persisted bibliography defaults and exposing the style in the References and Settings citation-style selectors. |

Vega-Lite native preview fidelity:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked vega_lite --lib` | Pass | Focused transform tests passed after the native Vega-Lite SVG renderer learned object-form marks, y-axis `sum`/`mean`/`average`/`min`/`max` aggregates, axis titles, negative values with a real zero baseline, grouped-series metadata, and export-safe `data-value` labels for variance charts. |

GeoJSON projection honesty:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked geojson_transform_warns_for_projection_assumptions --lib` | Pass | Focused transform proof shows GeoJSON static SVG previews declare their native `linear-wgs84-fit` longitude-latitude projection assumption and warn on legacy CRS/projected-coordinate inputs. |

Nested structured data tables:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked structured_data_tables_flatten_nested_business_rows --lib` | Pass | Focused transform proof shows nested JSON/YAML business row objects flatten into dot-path table columns instead of opaque object summaries, including scalar-array cells. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked front_matter_data_sources_survive_cross_target_exports --lib` | Pass | Cross-target data-source proof remains clean after nested structured-row flattening and now asserts export-safe numeric `data-value` cells for CSV/TSV data-source tables. |

External transform executable validation:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked external_transform_rejects_directory_engine_path_before_spawn --lib` | Pass | Focused external-transform safety proof shows configured engine paths must be regular executable files and directories are rejected before process spawn. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked external_transform --lib` | Pass | 16 external-transform tests passed, including trust gating, disabled fallback, missing path diagnostics, non-executable rejection, directory-as-engine rejection, timeout, stderr, adapter shapes, installed-engine conformance, and cache invalidation. |

Two-way table text editing:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | Frontend unit coverage now includes `syncTableDraftFromDocumentText`, proving a visually created table can be inserted as Markdown, edited directly in the document text, resynced into the visual grid, written back from the grid, and protected while the text edit is temporarily not parseable. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after routing clean in-document table source edits through the explicit table text-sync helper instead of an implicit watcher reload. |
| `pnpm run check:tables:manual` | Pass with pending human sign-off | Table editor manual review template and pending summary were regenerated so source-to-grid, grid-to-source, spreadsheet exchange, rendered export, keyboard/accessibility, and supported-host review remain visible release gates. |

Horizontal business charts:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked chart_transform_renders_horizontal_business_comparisons --lib` | Pass | Focused transform proof shows native chart rendering now supports `horizontal-bar`/`barh` for long-label ranked comparisons, including negative values, target lines, units, and grouped multi-series bars. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked chart_transform --lib` | Pass | All five chart transform tests passed, covering YAML bars, pie, area, KPI, negative variance/target/unit charts, grouped bar/line charts, and horizontal business comparisons. |
| `pnpm run test:unit` | Pass | Frontend template coverage now proves the built-in chart library includes a horizontal risk-comparison starter template with fill fields. |

TopoJSON mixed geometry fidelity:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked topojson_transform_applies_quantized_point_geometry_transform --lib` | Pass | Focused transform proof shows TopoJSON `Point` and `MultiPoint` object coordinates use the same `transform.scale`/`transform.translate` decoding as arcs, keeping markers aligned with mixed line/point static SVG previews. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked visual_data_transforms_survive_cross_target_exports --lib` | Pass | Cross-target visual-data export proof remains clean for static SVG artifacts after the TopoJSON point-transform decoder update. |

STL depth-aware static previews:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked stl_transform_renders_depth_aware_isometric_preview --lib` | Pass | Focused transform proof shows native STL previews preserve XYZ input through an isometric projection, draw lower-depth triangles first, expose per-triangle `data-depth`, vary opacity by depth, and report `z-depth` summary metadata. |

Structured JSON/YAML map tables:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked structured_data_tables_render_keyed_maps_and_scalar_settings --lib` | Pass | Focused structured-data proof shows keyed JSON object maps render as captioned tables with stable key columns, scalar YAML settings maps render as two-column field tables, scalar arrays remain readable, and mixed hierarchical documents continue using structured trees. |

OpenAPI metadata completeness:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked compiler_renders_openapi_and_json_schema_tables --lib` | Pass | Focused OpenAPI proof shows native API references now preserve terms of service, contact, license, root external docs, tag descriptions, and tag external docs alongside existing operation, callback, webhook, schema, security, and parameter tables. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked api_schema_transforms_survive_cross_target_exports --lib` | Pass | Cross-target API/schema export proof remains clean after adding API metadata to the OpenAPI native HTML artifact fixture. |

Profile-filled CLI snippets:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked ned_cli --lib` | Pass | Rust CLI coverage now proves `ned snippets --workspace <dir> --fill-profile --markdown company-contact-block` merges saved `.neditor/business-profile.json` identity values into reusable document parts, preserves raw snippet Markdown in JSON, and leaves unresolved non-profile placeholders visible. |
| `src-tauri/target/debug/ned snippets --workspace /private/tmp/neditor-profile-snippet-smoke --fill-profile --markdown company-contact-block` | Pass | Direct binary smoke printed a reusable contact snippet with saved full name, role, company, email, and website merged from `.neditor/business-profile.json` while leaving unset address and phone placeholders visible. |
| `pnpm run check:docs` | Pass | Markdown documentation links remain valid after documenting profile-filled snippet rendering for non-technical CLI use. |

Configuration setup and two-way table polish:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit -- --runInBand` | Pass | 84 frontend unit/static tests passed after extracting configuration setup readiness, suggested-answer, note-block, and section metadata into `src/lib/configurationSetup.ts`; the same static coverage now guards the explicit two-way table action panel labels for creating tables in Markdown text, editing table text, syncing text to the grid, and applying grid edits back to text. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Tables panel gained a dedicated two-way status hint and the configuration setup wizard delegated setup scoring and assistance copy through the extracted helper module. |

Table two-way state extraction:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit -- --runInBand` | Pass | 85 frontend unit/static tests passed, including direct `buildTableTwoWayState` coverage for synced, invalid source, valid dirty-source preview, still-invalid dirty source, text-repair, document-changed, and new-draft states so table text/grid round-trip guidance is behavior-tested instead of only embedded in `App.vue`. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Tables panel delegated its two-way status, status class, and hint through the extracted table helper. |

AI provenance appendix auditability:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked ai_provenance_appendix_is_audit_readable --manifest-path src-tauri/Cargo.toml` | Pass | Focused export proof now checks that enabled HTML provenance appendices distinguish `data-kind="source"` records from `data-kind="section"` records, carry `data-status` and section `data-line` audit hooks, render reviewer-readable source/section labels, escape prompt summaries, preserve reviewer timestamps with ISO colons, and disappear when `includeProvenance` is false. |
| `cargo test --locked review_provenance_tests --manifest-path src-tauri/Cargo.toml` | Pass | All eight provenance parser tests remain clean after fixing inline AI-assisted metadata parsing so `key=value` fields with ISO timestamp colons keep the full `reviewedAt` value. |

Mixed calculation and table workflow proof:

| Command | Result | Evidence |
| --- | --- | --- |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow now inserts a calc block from the command palette, verifies the live preview renders the formula output, then continues through the same document's table workflow with visual table creation, totals/formula rows, direct Markdown source edits, source-to-grid sync, edited-source CSV export, and cell-at-cursor text editing. |

Brand profile cross-target artifact proof:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked compile_options_supply_brand_profile_defaults --manifest-path src-tauri/Cargo.toml` | Pass | Default brand profiles now have direct artifact-family proof across HTML, PDF/text metadata, DOCX headers/footers/custom properties, PPTX company/header/footer/legal slides, and Markdown bundle `document.txt` plus `metadata.json`, including brand name/color/logo/font, header/footer templates, watermark, and legal disclaimer. |

CSV/TSV ragged-row validation:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked csv_and_tsv_transforms_warn_and_normalize_ragged_rows --manifest-path src-tauri/Cargo.toml` | Pass | Focused table-transform proof now covers ragged CSV and TSV inputs: missing header/data cells produce artifact and document diagnostics, placeholder headers such as `Column 3` are generated, short rows are padded, extra cells remain exportable, and numeric metadata still survives normalized rows. |
| `cargo test --locked table_tests --manifest-path src-tauri/Cargo.toml` | Pass | All 17 table tests passed after adding CSV/TSV row-width normalization, preserving existing Markdown table formulas, CSV/TSV formula diagnostics, spreadsheet exchange, named tables, SQL transform, merged cells, and cross-target table export behavior. |

Text-preserving table cell edits:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 86 frontend unit/static tests passed after tightening Markdown table cell text edits so the cursor-cell workflow preserves outer-pipe-free rows, indented pipe rows, escaped literal pipes, and extra author-typed cells while still reparsing the changed table into the visual grid. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the table text-edit helper started preserving the source row style during two-way table edits. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "runs command palette insertion and table editor workflows" --project chromium` | Pass | Focused Chromium workflow still proves table creation, direct Markdown source replacement, source-to-grid sync, edited-source CSV export, and cursor-cell text editing after the source-preserving cell edit change. |

RFP import command coverage:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked rfp_import --lib` | Pass | 7 RFP import tests passed, including direct command-level Markdown/text alias import, pasted-source normalization, metadata trimming, unsupported source-type rejection, DOCX XML text/table handling, and URL HTML cleanup/title extraction. |

Local-agent handoff command coverage:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked local_agents --lib` | Pass | Local-agent handoff command tests now prove the profile allowlist, unsupported-profile rejection, empty-prompt rejection, workspace-contained handoff file creation, document-path-to-parent workspace resolution, Google Antigravity profile support, and PATH executable probing. |

File watcher shutdown command coverage:

| Command | Result | Evidence |
| --- | --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked stop_file_watcher_clears_active_watcher_state --lib` | Pass | Focused Rust proof now constructs active watcher state for the default native-watch build, clears it through the same helper used by the `stop_file_watcher` IPC command, and verifies the active watcher slot is released. |

Vim punctuation word-motion parity:

| Command | Result | Evidence |
| --- | --- | --- |
| `pnpm run test:unit` | Pass | 87 frontend unit/static tests passed after Vim `w`/`e`/`b` and operator-motion ranges started treating punctuation runs as word boundaries, covering `client.name/value next` alongside existing word, linewise yank, paste, and Emacs kill/yank semantics. |
| `pnpm run check` | Pass | Vue/TypeScript validation passed after the Vim motion helper started using punctuation-aware character classes. |
| `node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "runs configurable Emacs and Vim-style editor keybinding modes" --project chromium` | Pass | Focused Chromium workflow now proves punctuation-aware Vim operator motions and word motions in the actual CodeMirror editor by deleting `client`, deleting `.`, and inserting before `/` in `client.name/value next`. |

## Next Execution Order

1. Refresh Google Drive connector authorization for document upload/conversion,
   then re-run live Google Docs import proof for the rendered export package.
2. Execute the Windows/Linux Tauri-driver workflow harness on supported hosts.
   The macOS full baseline now runs the app-authored launch smoke first, and
   the WebDriver report attaches that native fallback proof where WebDriver is
   officially unavailable.
3. Use failures from workflow tests to drive implementation fixes.
4. Continue expanding browser coverage for remaining preview modes, AI
   review-state workflows, export progress edge cases, table export modes, and
   cross-platform shortcut/tab-pointer accelerators.
5. Add macOS/Windows optional transform engine evidence.
6. Only after behavior is locked, modularize oversized frontend/store/backend
   modules.

## Completion Gate

Do not mark the thread goal complete until:

- `docs/spec-completion-matrix.md` has current evidence for every explicit spec
  requirement.
- Every non-deferred row in the matrix is complete.
- The fresh verification baseline passes on the current commit.
- Browser/desktop workflow tests prove the main user journeys.
- Export artifacts are proven for business-document fidelity.
- Platform packaging and optional-engine evidence are documented.
- The worktree is clean and pushed.
