<template>
  <section v-show="!store.zenMode" class="document-tabs" aria-label="Open documents">
    <section
      v-for="group in groupedDocuments"
      :key="group.key"
      class="tab-group"
      :aria-label="`${group.label} tabs`"
      @dragover.prevent
      @drop="$emit('drop-on-group', group, $event)"
    >
      <header class="tab-group-header" :title="group.title">
        <span class="tab-group-title">
          <span>{{ group.label }}</span>
          <small>{{ group.documents.length }}</small>
        </span>
        <button class="tab-icon-button" type="button" aria-label="Close tab group" title="Close tab group" @click="$emit('close-tab-group', group)">
          <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
            <path v-for="path in toolbarIconPaths('close')" :key="path" :d="path"></path>
          </svg>
        </button>
      </header>
      <div
        v-for="(document, tabIndex) in group.documents"
        :key="document.id"
        class="tab"
        :class="{ active: document.id === store.activeId }"
        :title="document.path || document.title"
        :data-document-path="document.path || ''"
        draggable="true"
        @pointerdown="$emit('update:draggedTabId', document.id)"
        @dragstart="startTabDrag(document.id, $event)"
        @dragover.prevent
        @drop="$emit('drop-on-document', document, $event)"
        @dragend="$emit('update:draggedTabId', '')"
      >
        <span
          class="tab-drag-handle"
          draggable="true"
          title="Drag tab"
          aria-hidden="true"
          @dragstart="startTabDrag(document.id, $event)"
        >::</span>
        <button
          class="tab-main"
          type="button"
          :aria-label="documentTabAriaLabel(document)"
          @click="$emit('activate', document.id)"
        >
          <span v-if="document.dirty" class="tab-dirty" aria-hidden="true"></span>
          <span class="tab-title">{{ document.title }}</span>
          <small v-if="documentTabFileName(document)" class="tab-file-name">{{ documentTabFileName(document) }}</small>
        </button>
        <button
          class="tab-icon-button"
          type="button"
          aria-label="Move tab left"
          :title="`Move ${document.title} tab left`"
          :disabled="tabIndex === 0"
          @click="$emit('move-within-group', group, document.id, -1)"
        >
          <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
            <path v-for="path in toolbarIconPaths('previous')" :key="path" :d="path"></path>
          </svg>
        </button>
        <button
          class="tab-icon-button"
          type="button"
          aria-label="Move tab right"
          :title="`Move ${document.title} tab right`"
          :disabled="tabIndex === group.documents.length - 1"
          @click="$emit('move-within-group', group, document.id, 1)"
        >
          <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
            <path v-for="path in toolbarIconPaths('next')" :key="path" :d="path"></path>
          </svg>
        </button>
        <button
          class="tab-icon-button"
          :class="{ active: document.pinned }"
          type="button"
          :aria-label="document.pinned ? 'Unpin document' : 'Pin document'"
          :title="document.pinned ? 'Unpin document' : 'Pin document'"
          @click="$emit('toggle-pin', document.id)"
        >
          <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
            <path v-for="path in toolbarIconPaths('pin')" :key="path" :d="path"></path>
          </svg>
        </button>
        <button class="tab-icon-button" type="button" aria-label="Close document" title="Close document" @click="$emit('close-document', document.id)">
          <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
            <path v-for="path in toolbarIconPaths('close')" :key="path" :d="path"></path>
          </svg>
        </button>
      </div>
    </section>
  </section>
</template>

<script setup lang="ts">
import { useDocumentsStore } from "../stores/documents";
import type { OpenDocument } from "../types";

interface DocumentTabGroup {
  key: string;
  label: string;
  title: string;
  documents: OpenDocument[];
}

const props = defineProps<{
  groupedDocuments: DocumentTabGroup[];
  draggedTabId: string;
  toolbarIconPaths: (icon: string) => string[];
}>();

defineEmits<{
  "update:draggedTabId": [id: string];
  activate: [id: string];
  "close-document": [id: string];
  "close-tab-group": [group: DocumentTabGroup];
  "drop-on-group": [group: DocumentTabGroup, event: DragEvent];
  "drop-on-document": [doc: OpenDocument, event: DragEvent];
  "move-within-group": [group: DocumentTabGroup, id: string, dir: -1 | 1];
  "toggle-pin": [id: string];
}>();

const store = useDocumentsStore();

function documentTabFileName(document: OpenDocument): string {
  if (!document.path) return "";
  const fileName = document.path.split(/[\\/]/).pop() || "";
  return fileName && fileName !== document.title ? fileName : "";
}

function documentTabAriaLabel(document: OpenDocument): string {
  const fileName = documentTabFileName(document);
  const parts = [document.dirty ? "Unsaved" : "", document.title, fileName].filter(Boolean);
  return parts.join(" ");
}

function startTabDrag(documentId: string, event: DragEvent): void {
  event.dataTransfer?.setData("text/plain", documentId);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}
</script>
