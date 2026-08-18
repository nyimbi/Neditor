<template>
  <section id="markdown-source" v-show="store.mode !== 'preview' && store.mode !== 'export' && store.mode !== 'presentation' && store.mode !== 'outline'" class="editor-pane" :class="{ 'focus-mode': writerFocusMode && store.uiMode === 'writer' }" aria-label="Markdown source" tabindex="-1" @pointerup="handleEditorPointerUp" @keydown.escape="selectionToolbarVisible = false" style="position:relative">
    <!-- ── Breadcrumb navigation ── -->
    <nav
      v-if="store.uiMode === 'pilot' && documentBreadcrumbs.length"
      class="editor-breadcrumbs"
      aria-label="Document section breadcrumbs"
    >
      <span class="breadcrumb-doc">{{ active?.title || 'Document' }}</span>
      <template v-for="crumb in documentBreadcrumbs" :key="crumb.line">
        <span class="breadcrumb-sep">›</span>
        <button type="button" class="breadcrumb-item" @click="scrollEditorToLine(crumb.line)">{{ crumb.text }}</button>
      </template>
    </nav>
    <div v-if="store.uiMode === 'writer'" class="writer-doc-title" aria-hidden="true">
      {{ active?.compile?.semantic.title || active?.title || '' }}
    </div>
    <div v-if="docLocked" class="doc-locked-banner" aria-live="polite">
      🔒 This document is approved/locked. Editing is disabled. Change status in front matter to unlock.
    </div>
    <!-- Item B: first-run empty-state overlay -->
    <div
      v-if="emptyStateOverlayVisible"
      class="empty-state-overlay"
      role="dialog"
      aria-modal="true"
      aria-label="Welcome to NEditor"
      @keydown.escape.stop="dismissEmptyState(false)"
    >
      <div class="empty-state-card" @keydown.tab.stop>
        <div class="empty-state-cursor" aria-hidden="true">|</div>
        <h2 class="empty-state-heading">Start typing</h2>
        <p class="empty-state-hint">NEditor saves your work as a normal Markdown file on your Mac. Nothing leaves your device unless you export it.</p>
        <div class="empty-state-actions">
          <button type="button" class="primary" @click="store.newDocument(); dismissEmptyState(true)">New document</button>
          <button type="button" @click="openDocument(); dismissEmptyState(true)">Open file</button>
          <button type="button" @click="openFolder(); dismissEmptyState(true)">Open folder</button>
        </div>
        <label class="empty-state-dismiss-label">
          <input type="checkbox" @change="(e) => { if ((e.target as HTMLInputElement).checked) dismissEmptyState(true); }" />
          Do not show again
        </label>
        <button type="button" class="empty-state-close" aria-label="Close welcome overlay" @click="dismissEmptyState(false)">x</button>
      </div>
    </div>

    <div class="editor-split-grid" :data-split-source="store.splitSourcePanes ? 'true' : 'false'" :class="{ 'has-minimap': store.showMinimap }">
      <div ref="editorHost" class="editor-host editor-host-primary" :class="{ 'editor-locked': docLocked }" aria-label="Primary Markdown source pane"></div>
      <div v-if="store.splitSourcePanes" ref="secondaryEditorHost" class="editor-host editor-host-secondary" aria-label="Secondary Markdown source pane"></div>
      <!-- Minimap: document structure navigator -->
      <nav v-if="store.showMinimap" class="editor-minimap" aria-label="Document minimap">
        <div class="minimap-header">
          <span>Map</span>
          <button type="button" @click="store.showMinimap = false" aria-label="Close minimap">×</button>
        </div>
        <div class="minimap-headings">
          <button
            v-for="heading in (active?.compile?.document_ast?.blocks?.filter((b: any) => b.kind === 'heading') || [])"
            :key="(heading as any).line"
            type="button"
            class="minimap-heading"
            :class="`minimap-h${(heading as any).level}`"
            :title="(heading as any).text"
            @click="scrollEditorToLine((heading as any).line)"
          >{{ (heading as any).text }}</button>
        </div>
      </nav>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, inject, ref, type Ref } from 'vue'
import { useDocumentsStore } from '../stores/documents'

interface EditorPaneCtx {
  writerFocusMode: Ref<boolean>
  handleEditorPointerUp: () => void
  selectionToolbarVisible: Ref<boolean>
  openDocument: () => Promise<void>
  openFolder: () => Promise<void>
  editorCursorLine: Ref<number>
  scrollEditorToLine: (line: number) => void
  docLocked: Ref<boolean>
}

defineEmits<{
  change: [text: string]
  cursor: [line: number]
}>()

const store = useDocumentsStore()
const ctx = inject<EditorPaneCtx>('editorPaneCtx')!

const { writerFocusMode, handleEditorPointerUp, selectionToolbarVisible, openDocument, openFolder, editorCursorLine, scrollEditorToLine, docLocked } = ctx

const editorHost = ref<HTMLElement | null>(null)
const secondaryEditorHost = ref<HTMLElement | null>(null)

const active = computed(() => store.activeDocument)

const emptyStateOverlayVisible = computed(
  () =>
    !store.hasSeenEmptyState &&
    active.value.title === "Untitled" &&
    !active.value.text.trim() &&
    store.uiMode !== 'writer',
)

function dismissEmptyState(permanent: boolean): void {
  if (permanent) {
    store.hasSeenEmptyState = true
    void store.persistWorkspace()
  }
}

const documentBreadcrumbs = computed(() => {
  const doc = active.value
  if (!doc?.compile) return []
  const line = editorCursorLine.value
  const headings = doc.compile.document_ast?.blocks?.filter((b: any) => b.kind === 'heading') || []
  const active_headings: Array<{ level: number; text: string; line: number }> = []
  for (const h of headings as Array<{ kind: string; level: number; text: string; line: number; end_line: number }>) {
    if (h.line > line) break
    while (active_headings.length && active_headings[active_headings.length - 1].level >= h.level) {
      active_headings.pop()
    }
    active_headings.push({ level: h.level, text: h.text, line: h.line })
  }
  return active_headings
})

defineExpose({
  editorHostEl: computed(() => editorHost.value),
  secondaryEditorHostEl: computed(() => secondaryEditorHost.value),
})
</script>
