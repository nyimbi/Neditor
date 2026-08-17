<template>
  <footer v-show="!writingSpaceMaximized && store.uiMode !== 'writer'" id="document-status" class="status-bar" :class="{ 'zen-mode-status': store.zenMode }" aria-label="Document status and progress" tabindex="-1">
    <span
      class="status-message"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      :aria-label="`Status message: ${store.statusMessage || 'No status message'}`"
    >
      {{ store.statusMessage }}
    </span>
    <span v-if="store.externalConflict" class="conflict-actions">
      <button type="button" @click="$emit('open-conflict')">Compare</button>
      <button type="button" @click="store.acceptExternalChanges">Accept external</button>
      <button type="button" @click="store.keepLocalChanges">Keep local</button>
      <button type="button" @click="$emit('save-conflict-copy')">Save copy</button>
    </span>
    <span class="word-stats" :aria-label="`Document statistics: ${wordStats}`" @click="$emit('open-word-goal-dialog')" style="cursor:pointer" title="Click to set word count goal">{{ wordStats }}</span>
    <span v-if="wordGoalProgress" class="word-goal-progress" :title="`Word goal: ${wordGoalProgress.current} / ${wordGoalProgress.target}`">
      <span class="word-goal-bar" :style="{ width: wordGoalProgress.pct + '%' }" :class="{ done: wordGoalProgress.done }"></span>
      <small>{{ wordGoalProgress.pct }}%</small>
    </span>
    <span v-if="activeNudge" class="feature-nudge" role="status" aria-live="polite">
      {{ activeNudge }}
      <button type="button" @click="$emit('dismiss-nudge')" aria-label="Dismiss tip">✕</button>
    </span>
    <span class="keymap-status" :aria-label="`Editor keybinding mode: ${editorKeymapStatus}`">{{ editorKeymapStatus }}</span>
    <span
      v-if="previewTimingStatus"
      class="preview-timing"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      :aria-label="`Preview timing: ${previewTimingStatus}`"
    >
      {{ previewTimingStatus }}
    </span>
    <span
      v-if="watchStatus"
      class="watch-status"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      :aria-label="`File watch status: ${watchStatus}`"
    >
      {{ watchStatus }}
    </span>
    <span
      v-if="store.compileProgress"
      class="compile-actions"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      :aria-label="`Compile progress: ${store.compileProgress}`"
    >
      {{ store.compileProgress }}
      <button type="button" @click="store.cancelActiveCompile">Cancel compile</button>
    </span>
    <span
      v-if="store.exportProgress"
      class="export-progress"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      :aria-label="`Export progress: ${store.exportProgress}`"
    >
      {{ store.exportProgress }}
    </span>
    <span
      v-if="store.aiRun"
      class="ai-run-pill"
      role="status"
      aria-live="polite"
      aria-atomic="true"
      :aria-label="`AI run: ${store.aiRun.label}, ${aiElapsedSeconds}s elapsed`"
    >
      <span class="ai-run-label">{{ store.aiRun.label }}</span>
      <span class="ai-run-timer">{{ aiElapsedSeconds }}s</span>
      <button type="button" class="ai-run-cancel" @click="store.cancelAiRun()">Cancel</button>
    </span>
    <span
      v-if="store.lastError"
      class="error"
      role="alert"
      aria-live="assertive"
      aria-atomic="true"
      :aria-label="`Error: ${store.lastError}`"
    >
      {{ store.lastError }}
    </span>
  </footer>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useDocumentsStore } from "../stores/documents";

const store = useDocumentsStore();

const props = defineProps<{
  writingSpaceMaximized: boolean;
  activeNudge: string | null;
  aiElapsedSeconds: number;
  editorKeymapStatus: string;
}>();

defineEmits<{
  'open-word-goal-dialog': [];
  'open-conflict': [];
  'save-conflict-copy': [];
  'dismiss-nudge': [];
}>();

const activeWordCount = computed(() => {
  const text = store.activeDocument?.text || "";
  return text.trim().split(/\s+/).filter(Boolean).length;
});

const wordGoalProgress = computed(() => {
  const target = store.wordCountTarget;
  if (!target || target <= 0) return null;
  const current = activeWordCount.value;
  const pct = Math.min(100, Math.round((current / target) * 100));
  return { target, current, pct, done: current >= target };
});

const wordStats = computed(() => {
  const words = activeWordCount.value;
  const text = store.activeDocument?.text || "";
  const rs = store.lastReadabilityStats;
  const minutes = rs ? rs.readingTimeMinutes.toFixed(1) : (words ? Math.max(1, Math.ceil(words / 220)) : 0);
  const grade = rs ? ` | FK ${rs.fleschKincaidGrade.toFixed(1)}` : '';
  return `${words} words | ${text.length} chars | ${minutes} min read${grade}`;
});

const previewTimingStatus = computed(() => {
  if (store.lastPreviewCompileDurationMs === null) return "";
  return `Preview updated in ${store.lastPreviewCompileDurationMs} ms for ${store.lastPreviewCompiledCharacters} characters`;
});

const watchStatus = computed(() => {
  if (store.watchDriver === "off" || !store.watchedPaths.length) return "";
  const label = store.watchDriver === "native" ? "Native watch" : "Plugin watch";
  const suffix = store.watchedPaths.length === 1 ? "path" : "paths";
  return `${label}: ${store.watchedPaths.length} ${suffix}`;
});
</script>
