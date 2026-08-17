<template>
  <section v-if="open" class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Preview theme gallery">
    <div class="modal">
      <header class="modal-header">
        <h2>Preview Themes</h2>
        <button type="button" class="modal-close" aria-label="Close theme gallery" @click="$emit('update:open', false)">×</button>
      </header>
      <div class="modal-body preview-theme-gallery-body">
        <div v-if="!themeList.length" class="preview-theme-empty">
          <p>No themes found. Add .css files to your user themes folder.</p>
          <button type="button" @click="$emit('open-themes-dir')">Open user themes folder</button>
        </div>
        <ul v-else class="preview-theme-list">
          <li
            v-for="theme in themeList"
            :key="theme.id"
            class="preview-theme-item"
            :class="{ active: store.previewTheme === theme.id }"
            @click="$emit('select-theme', theme.id); $emit('update:open', false)"
          >
            <div class="preview-theme-swatch" :data-theme-source="theme.source"></div>
            <span class="preview-theme-name">{{ theme.name }}</span>
            <small class="preview-theme-source">{{ theme.source }}</small>
          </li>
        </ul>
        <footer class="preview-theme-gallery-footer">
          <button type="button" @click="$emit('open-themes-dir')">Open user themes folder</button>
          <button type="button" @click="$emit('update:open', false)">Close</button>
        </footer>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useDocumentsStore } from "../stores/documents";

const store = useDocumentsStore();

defineProps<{
  open: boolean;
  themeList: Array<{ id: string; name: string; source: "bundled" | "user" }>;
}>();

defineEmits<{
  'update:open': [value: boolean];
  'select-theme': [id: string];
  'open-themes-dir': [];
}>();
</script>
