# NEditor UI/UX Analysis and Recommendations

This document captures a review of the current NEditor user interface, the problems observed, and a prioritized set of recommended changes. The goal is to make NEditor fit for intense daily use by business people, analysts, developers, and casual Markdown readers.

The analysis was produced by running the built application in a browser, capturing screenshots of the default state, Pilot mode, toolbars, AI panels, command palette, and export/review sidebars, and then reading the relevant code paths in `src/App.vue`, `src/stores/documents.ts`, and `src/lib/aiProviderPackages.ts`.

## TL;DR

The application currently oscillates between two extremes:

- **Writer mode** (the default) hides almost everything and leaves new users staring at a near-blank page.
- **Pilot mode** dumps roughly 90 unlabeled toolbar icons on screen at once.

The AI flows are powerful but modal-heavy, lack progress/cancellation feedback, and can hang because provider requests have no timeout. The single 39,000-line `App.vue` file makes all of this hard to iterate on.

The highest-impact fixes are:

1. Make the default landing a balanced, labelled workbench instead of Writer mode.
2. Show text labels on toolbar buttons by default.
3. Add a real empty-state / onboarding screen.
4. Put the AI requests behind timeouts, progress indicators, and clear error states.
5. Reduce duplicate navigation surfaces and progressively disclose advanced features.
6. Start componentizing `App.vue` so UX changes become maintainable.

## What was observed

### 1. Default Writer mode is disorienting

- `store.uiMode` defaults to `"writer"` (`src/stores/documents.ts:361`).
- The CSS for Writer mode hides the command bar, activity bar, sidebar, preview pane, inspector, and status bar (`src/App.vue:37383-37416`).
- The first-run screen shows only a menu bar, a tab, and a thin status strip. There is no visible toolbar, no outline, no preview, and the editor workspace can render blank when the preview backend is unavailable.
- A new user has no obvious place to click to start typing.

### 2. Pilot mode is overwhelming

- Switching to Pilot reveals the full command bar, which currently renders roughly **90 icon-only buttons** across the File, Writing, and Review & Navigate rows.
- The toolbar label setting (`store.toolbarDisplay`) is effectively ignored in Pilot mode because the CSS explicitly hides group labels (`src/App.vue:29908`).
- Many actions are duplicated across the top menu bar, the command bar, the activity bar, the sidebar, and the command palette, so every feature is reachable in five different places.

### 3. Sidebar panels show everything simultaneously

- The left sidebar crams outline, outline planner, document map, blockers, file search, and document sets into one scroll.
- Switching sidebar activity buttons opens very different panels (Exports, Versioning, Review) but there is no clear hierarchy or "advanced" section.
- Casual users see enterprise-grade export QA and snapshot recovery options on first contact.

### 4. AI workflows look powerful but fragile

- **Docs Live** is a 5-step wizard inside a modal with dense copy and many fields. It is unclear what happens when the user presses *Generate draft* or what the expected wait time is.
- **Agent Workspace** is another modal with a long instruction box, source-pack builder, document memory, and provider handoff controls all competing for attention.
- The actual provider request (`src/lib/aiProviderPackages.ts:284-314`) uses `globalThis.fetch` with **no timeout, no `AbortController`, and no streaming progress**. If an endpoint is unreachable, the UI's only feedback is a busy flag and a status-bar message, which feels like a hang.

### 5. Status and error communication is too subtle

- Most feedback goes to the bottom status bar or the thin writer-status-strip.
- There are no non-blocking toasts, no inline empty-state illustrations, and no "what went wrong and how to fix it" messages when the preview backend or AI provider is unavailable.

### 6. The codebase is not structured for rapid UX iteration

- `src/App.vue` is approximately 39,000 lines.
- The entire UI, state wiring, and dozens of modal panels live in one file.
- This makes it risky to move, hide, or reorganize elements without side effects.

## Recommended changes

### Immediate (do first)

1. **Change the default landing experience**
   - Replace the default `uiMode: "writer"` with a new balanced "Workbench" mode:
     - Split source/preview by default.
     - One visible toolbar row with text labels.
     - Left sidebar showing the document outline.
     - Right inspector collapsed but discoverable.
   - Keep Writer mode, but make it an explicit focus-mode toggle (e.g., a prominent "Focus" button or `Esc` to enter/exit), not the first impression.

2. **Show labels by default**
   - Remove or fix the CSS that hides command-group labels in Pilot mode (`src/App.vue:29908`).
   - Default `toolbarDisplay` to a labelled style and make icon-only an opt-in compact setting.
   - Every toolbar icon should have a persistent visible label or, at minimum, a tooltip on hover/focus.

3. **Add a first-run / empty-state screen**
   - For a new untitled document, show a centered overlay with:
     - "Start typing" hint with a big cursor/placeholder.
     - Quick actions: New, Open, Open recent folder, Open showcase, Open guided demo.
     - A one-line explanation of where the document lives (local Markdown file).
   - Hide the overlay as soon as the user types or dismisses it.

4. **Make the source editor resilient**
   - The Markdown source editor should mount and remain editable even when the Rust compile backend is unavailable (e.g., in a browser preview).
   - The preview pane can show a "Preview unavailable" placeholder instead of taking the whole workspace down.

5. **Fix the AI hang**
   - Wrap all provider fetches in an `AbortController` with a user-visible timeout (default ~60 seconds, configurable).
   - While waiting, show a real progress indicator (spinner + "Contacting provider…") and a **Cancel** button.
   - On failure, show a non-blocking error card with the exact error and a "Check provider settings" link, not just a status-bar message.

6. **Consolidate the navigation model**
   - Top menu bar: File / Edit / View (classic, predictable).
   - Toolbar: context-aware formatting and insertion tools only.
   - Activity bar / sidebar: document structure, diagnostics, and exports as distinct tabs, not one long scroll.
   - Command palette: the catch-all for everything else.
   - Remove duplicate buttons where the same action appears in three places (e.g., Export, AI Create, Agent).

### Short term (next 1-2 months)

7. **Redesign the toolbar as a contextual ribbon**
   - Three collapsible sections by default: File/Edit, Insert/AI, Review/Export.
   - Move advanced actions (Visual QA, Release evidence, A11y QA, multiple export formats, LaTeX templates, handlers) into a "More" overflow or into relevant sidebar panels.
   - Highlight primary actions (Save, AI Create, Export) with a stronger visual style; do not make every button look identical.

8. **Restructure the sidebar**
   - Use clearly named tabs: Outline, Files, Diagnostics, References, Exports, AI, Settings.
   - Each tab should have a primary panel and an expandable "Advanced" section.
   - Combine outline planner, document map, and heading navigation into a single navigable outline.

9. **Simplify AI modals**
   - **Docs Live**: turn the 5-step header into a real stepper with Next/Back flow, and show a preview of the generated outline before drafting.
   - **Agent Workspace**: separate planning, execution, and review into distinct views; show a task board with states (queued / running / needs approval / done) so the user always knows what the AI is doing.

10. **Add a global notification / toast system**
    - Replace the status bar as the primary feedback channel for user actions.
    - Use toasts for save, compile errors, AI progress, and export completion.
    - Keep the status bar for persistent metadata (word count, file path, git branch).

11. **Improve keyboard discoverability**
    - Show a keyboard shortcut hint in the command palette footer.
    - Add a "Keyboard shortcuts" item under Help.
    - Ensure focus rings are visible and focus order is logical through modals.

12. **Run accessibility and UX smoke tests**
    - Execute the existing runtime accessibility checks (`pnpm run check:a11y:runtime`) after changes.
    - Verify tab order, visible labels, and reduced-motion behavior.

### Longer term (2-6 months)

13. **Componentize `App.vue`**
    - Split into lazy-loaded panels: `EditorPane`, `PreviewPane`, `Sidebar`, `Toolbar`, `CommandPalette`, `AiWorkspace`, `ExportPanel`, `ReviewPanel`.
    - Move AI logic into a dedicated `useAi` composable or Pinia store module.
    - This is a prerequisite for safely iterating on the UX without regressions.

14. **Adaptive UI by document type / workflow**
    - When the user picks a workflow preset (business brief, lab notebook, RFP response, etc.), dynamically surface relevant tools:
      - Business brief: AI Create, Brand Kit, Export.
      - Lab notebook: Tables, Figures, Citations, Templates.
      - RFP response: Compliance matrix, Review, Export.
    - Hide irrelevant toolbar sections by default for the chosen workflow.

15. **Density and customization presets**
    - Offer Compact / Comfortable / Spacious density modes.
    - Let users pin/unpin toolbar sections and remember the layout per workspace.

16. **Onboarding tutorial**
    - A short, skippable interactive tour that opens the showcase document and demonstrates: typing Markdown, seeing live preview, inserting a table, using the command palette, and exporting.

17. **Establish UX metrics**
    - Track which toolbar buttons and sidebar panels are used, how long users stay in Writer vs Pilot mode, and where AI flows are abandoned.
    - Use that data to trim unused surface area.

## Persona lens

- **Business person / analyst** needs a clean default workbench, labelled tools, easy tables/charts, guided AI drafting, and one-click export. Current Pilot mode is too noisy; Writer mode is too empty.
- **Developer / power user** needs the command palette, keyboard shortcuts, source-first mode, and Git-aware status. They will tolerate density but need predictable shortcuts and no modal traps.
- **Casual Markdown reader** needs a simple preview/reading mode with minimal chrome. The current app forces them through a complex multi-panel interface.

## Key files to look at first

- `src/App.vue:37383-37416` — Writer-mode CSS that hides too much.
- `src/App.vue:29908` — CSS hiding toolbar labels in Pilot mode.
- `src/App.vue:14100-14117` — toolbar row definitions.
- `src/stores/documents.ts:361` — default `uiMode`.
- `src/lib/aiProviderPackages.ts:284-314` — provider fetch with no timeout or cancellation.

## Supporting evidence

During the review, screenshots were captured and saved under `.tmp/screenshots/` in the repo root. Example files include:

- `00-initial-load.png` — blank default state.
- `04-pilot-mode.png` — full Pilot workbench.
- `12-ai-create.png` — AI Create modal.
- `14-docs-live.png` — Docs Live wizard.
- `15-command-palette.png` — Command Palette.
- `16-export-panel.png` — Export & Publish sidebar.
- `17-review-panel.png` — Review & History sidebar.

These can be referenced when discussing or implementing specific changes.
