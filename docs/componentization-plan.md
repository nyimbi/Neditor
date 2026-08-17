# App.vue Componentization Plan

Phase 0 goal: DOM snapshot safety net that proves zero-behaviour-change refactors.

## Phase 0 Architecture

App.vue (39 k lines) is a single `<script setup lang="ts">` SFC. Mounting it in
a non-Tauri environment requires three layers of isolation:

### 1. Tauri IPC mocking (solved)

`@tauri-apps/api/core`, `@tauri-apps/api/event`, `@tauri-apps/api/path`,
`@tauri-apps/api/window`, and the five plugin packages are aliased to
`tests/lib/mocks/tauri-*.ts` stubs via `vite.snapshot.config.ts`.  All invoke
commands that fire during the boot critical path (`compile_document_with_options`,
`list_workspace_files`, `list_transform_engines`, `get_git_status`,
`list_snapshots`, `drain_cli_open_queue`, `list_preview_themes`, …) return
empty-success shapes so the store can complete without an error.

`@tauri-apps/plugin-store`'s `Store.load()` returns a mock instance whose
`get("workspace")` returns `null`, causing `loadPreferences()` to fall back to
factory defaults.  Store actions `bootCritical` and `bootBackground` are stubbed
in each test to prevent the factory-default fall-back from overwriting the
pre-set snapshot state.

### 2. .vue SFC compilation (solved)

`vite build --config vite.snapshot.config.ts` pre-compiles `src/App.vue` (via
the existing `@vitejs/plugin-vue`) and emits `.tmp-tests/AppBundle.js` as an ES
module.  `vue` and `pinia` are externalized so the test runner and the bundle
share one Pinia instance, enabling store patching from outside the bundle.

### 3. DOM environment (solved with caveats)

`happy-dom` provides `document`, `window`, `MutationObserver`,
`requestAnimationFrame`, `getComputedStyle`, `IntersectionObserver`, and a
no-op `ResizeObserver` polyfill.  This is sufficient for the Vue template
renderer.  CodeMirror's `EditorView` constructor is invoked in `onMounted` and
may emit layout errors (e.g. `getBoundingClientRect` returns zero dimensions in
happy-dom).  These are caught by `config.global.config.errorHandler` and do not
prevent template capture.

## Phase 0 Blockers

| # | Symptom | Root cause | Mitigation status |
|---|---------|------------|-------------------|
| B1 | `window.__neditor_boot` TypeError on first line of `onMounted` | Property does not exist on happy-dom Window | Fixed: test setup injects `globalThis.__neditor_boot = {}` before mount |
| B2 | `EditorView` layout measurement returns 0 in happy-dom | `getBoundingClientRect` is a no-op in non-rendering DOM | Mitigated: `errorHandler` swallows CM errors; template still captured |
| B3 | Dynamic `import("@tauri-apps/plugin-updater")` at runtime | Vite alias applies to static imports; dynamic imports in the bundle also go through Vite's rollup plugin so the alias IS applied | Resolved: alias covers dynamic imports in the bundle |
| B4 | `commandPaletteOpen` is a `ref` local to `<script setup>`; cannot be set from outside | No public accessor exists | Mitigated: test dispatches `keydown` (⌘K) to trigger it; if the shortcut handler is removed during refactor, snapshot 08 will diverge and surface the regression |
| B5 | `@tauri-apps/plugin-fs` `watch` is top-level imported in `documents.ts` | All Tauri sub-packages must be aliased individually | Fixed: alias covers `plugin-fs` |
| B6 | `tsc -p tsconfig.test.json` stack-overflows on `frontend-snapshots.test.ts` | happy-dom's `Window` type has circular self-references; TypeScript's control-flow type solver recurses until Node.js blows the call stack | Mitigated: `// @ts-nocheck` on the test file; tsc still emits JS output without type-checking that file; Vite build provides integrity for the App compilation |

## Extraction Sequence (post Phase 0)

Recommended extraction order (each step is a zero-behaviour-change refactor
that the snapshot suite verifies):

1. **Toast host** — `<div class="toast-host">` + `useToasts()` → `ToastHost.vue`
2. **Command palette** — `commandPaletteOpen` ref + `<CommandPalette>` template block → `CommandPalette.vue`
3. **Sidebar** — `store.sidebar` + all `<template v-else-if="store.sidebar === ...">` → `Sidebar.vue` + `SidebarPanel.vue` children
4. **Status bar** — `<footer id="document-status">` → `StatusBar.vue`
5. **Toolbar** — `<nav id="main-commands">` → `Toolbar.vue`
6. **Document tabs** — `<section class="document-tabs">` → `DocumentTabs.vue`
7. **Preview pane** — `<section id="preview-pane">` → `PreviewPane.vue`
8. **Editor pane** — CodeMirror mount + `buildEditor()` → `EditorPane.vue` (highest risk; extract last)
9. **AI overlays** — error card, progress pill, ai-paste dialog → `AiOverlay.vue`

After each extraction, run `pnpm run test:unit` to verify all 13 snapshots match.

## Snapshot Coverage

| # | File | State exercised |
|---|------|-----------------|
| 01 | `01-workbench-default.html` | uiMode=workbench, no docs open |
| 02 | `02-writer-mode.html` | uiMode=writer |
| 03 | `03-zen-mode.html` | zenMode=true |
| 04 | `04-pilot-mode.html` | uiMode=pilot |
| 05 | `05-empty-state-visible.html` | hasSeenEmptyState=false |
| 06 | `06-empty-state-dismissed.html` | hasSeenEmptyState=true |
| 07a | `07a-sidebar-outline.html` | sidebar=outline |
| 07b | `07b-sidebar-files.html` | sidebar=files |
| 07c | `07c-sidebar-exports.html` | sidebar=exports |
| 07d | `07d-sidebar-versioning.html` | sidebar=versioning |
| 07e | `07e-sidebar-review.html` | sidebar=review |
| 07f | `07f-sidebar-references.html` | sidebar=references |
| 08 | `08-command-palette-open.html` | commandPaletteOpen via ⌘K |
| 09 | `09-settings-panel.html` | sidebar=settings |
| 10 | `10-toast-host.html` | 3 toasts (info/warning/error) |
| 11 | `11-preview-failed.html` | previewFailed=true |
| 12 | `12-ai-progress-pill.html` | aiRun set |
| 13 | `13-ai-error-card.html` | aiLastError set |
