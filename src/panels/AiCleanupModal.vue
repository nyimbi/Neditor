<template>
  <section
    v-if="aiPasteOpen"
    :ref="(el) => (aiPasteDialog.value = el as HTMLElement | null)"
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="AI paste cleanup"
    tabindex="-1"
    @keydown="handleModalKeydown('ai-paste', $event)"
  >
    <form class="modal" @submit.prevent="cleanAiPaste">
      <header>
        <h2>Paste from AI Chat</h2>
        <button type="button" aria-label="Close AI paste cleanup" @click="closeAiPaste">x</button>
      </header>
      <section class="compare-grid ai-paste-grid">
        <div class="field-with-action">
          <div class="field-action-header">
            <label for="ai-paste-original">Original</label>
            <button
              type="button"
              :disabled="aiClipboardBusy"
              title="Read the current clipboard into the AI cleanup input"
              @click="loadAiPasteFromClipboard"
            >
              {{ aiClipboardBusy ? "Reading..." : "Load clipboard" }}
            </button>
          </div>
          <textarea
            id="ai-paste-original"
            v-model="aiPasteText"
            rows="12"
            placeholder="Paste AI chat output here"
            aria-label="Original"
            data-initial-focus
          ></textarea>
        </div>
        <label>
          Cleaned preview
          <textarea :value="store.aiCleanupPreview?.cleaned_markdown || ''" rows="12" readonly placeholder="Preview cleaned Markdown"></textarea>
        </label>
      </section>
      <label><input v-model="aiMarkAsDraft" type="checkbox" /> Mark as draft</label>
      <label><input v-model="aiAddProvenance" type="checkbox" /> Add provenance block</label>
      <label><input v-model="aiPreserveHeadings" type="checkbox" /> Preserve original headings</label>
      <label><input v-model="aiConvertNumberedLists" type="checkbox" /> Convert numbered lists</label>
      <label><input v-model="aiConvertTables" type="checkbox" /> Convert tables</label>
      <label><input v-model="aiInsertCitationTodos" type="checkbox" /> Insert citation TODOs</label>
      <label>
        Insert mode
        <select v-model="aiInsertMode">
          <option value="insert">Insert after document</option>
          <option value="quote">Quote</option>
          <option value="appendix">Appendix</option>
          <option value="selection">Replace selection</option>
          <option value="section">Merge into section</option>
          <option value="replace">Replace document</option>
        </select>
      </label>
      <section v-if="store.aiCleanupIssues.length" class="issue-list">
        <p v-for="issue in store.aiCleanupIssues" :key="issue">{{ issue }}</p>
      </section>
      <footer>
        <button type="button" @click="closeAiPaste">Cancel</button>
        <button type="button" :disabled="aiPreviewBusy || !aiPasteText.trim()" @click="previewAiPaste">
          {{ aiPreviewBusy ? "Cleaning" : "Preview cleanup" }}
        </button>
        <button type="submit" :disabled="aiPreviewBusy || !aiPasteText.trim()">Insert cleaned</button>
      </footer>
    </form>
  </section>
</template>

<script setup lang="ts">
import { inject } from 'vue';

const _ctx = inject('aiCleanupCtx') as Record<string, any>;
const {
  store,
  aiPasteDialog,
  aiPasteOpen,
  aiPasteText,
  aiClipboardBusy,
  aiInsertMode,
  aiAddProvenance,
  aiMarkAsDraft,
  aiInsertCitationTodos,
  aiPreserveHeadings,
  aiConvertNumberedLists,
  aiConvertTables,
  aiPreviewBusy,
  cleanAiPaste,
  previewAiPaste,
  loadAiPasteFromClipboard,
  closeAiPaste,
  handleModalKeydown,
} = _ctx;
</script>
