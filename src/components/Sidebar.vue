<template>
  <aside
    v-show="store.mode !== 'outline' && !writingSpaceMaximized && !store.zenMode"
    id="document-sidebar"
    :key="store.sidebar"
    :data-sidebar="store.sidebar"
    class="sidebar"
    aria-label="Document workspace"
    tabindex="-1"
  >
    <div
      v-if="store.uiMode === 'pilot'"
      class="sidebar-resize-handle"
      title="Drag to resize"
      @mousedown.prevent="onSidebarResizeStart"
    ></div>
    <button
      type="button"
      class="sidebar-collapse-btn"
      :title="sidebarCollapsed ? 'Expand sidebar (⌘B)' : 'Collapse sidebar (⌘B)'"
      :aria-label="sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'"
      @click="toggleSidebarCollapsed()"
    >{{ sidebarCollapsed ? '▶' : '◀' }}</button>
    <header v-if="store.uiMode === 'pilot'" class="sidebar-panel-header" aria-label="Current panel">
      <span class="sidebar-panel-name">{{ currentPanelLabel }}</span>
    </header>
    <slot />
    <template v-if="store.sidebar === 'help'">
      <slot name="help-panel" />
    </template>
    <template v-else-if="store.sidebar === 'settings'">
      <slot name="settings-panel" />
    </template>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useDocumentsStore } from '../stores/documents';

const props = defineProps<{
  writingSpaceMaximized: boolean;
  sidebarCollapsed: boolean;
  sidebarWidth: number;
}>();

const emit = defineEmits<{
  'toggle-sidebar-collapsed': [];
  'update:sidebarWidth': [val: number];
}>();

const store = useDocumentsStore();

const PANEL_LABELS: Record<string, string> = {
  files: 'Files', outline: 'Outline', diagnostics: 'Diagnostics', layout: 'Layout',
  tables: 'Tables', templates: 'Templates', references: 'References',
  exports: 'Export', versioning: 'Versioning', review: 'Review', help: 'Help', settings: 'Settings',
};

const currentPanelLabel = computed(() => PANEL_LABELS[store.sidebar as string] || String(store.sidebar));

function onSidebarResizeStart(event: MouseEvent): void {
  const startX = event.clientX;
  const startW = props.sidebarWidth;
  const onMove = (e: MouseEvent) => {
    emit('update:sidebarWidth', Math.max(160, Math.min(480, startW + (e.clientX - startX))));
  };
  const onUp = () => {
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  };
  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}

function toggleSidebarCollapsed(): void {
  emit('toggle-sidebar-collapsed');
}
</script>
