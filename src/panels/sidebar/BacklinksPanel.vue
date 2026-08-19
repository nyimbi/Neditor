<template>
  <h2>Backlinks</h2>
  <div class="sidebar-toolbar">
    <button type="button" @click="refreshBacklinks" :disabled="backlinksLoading" title="Refresh backlinks">↻ Refresh</button>
  </div>
  <section class="backlinks-panel" aria-label="Documents linking to this document">
    <div v-if="backlinksLoading" class="sidebar-loading">Scanning workspace…</div>
    <template v-else>
      <div class="backlinks-group" v-if="backlinksData.length">
        <h3>Linked ({{ backlinksData.length }})</h3>
        <button
          v-for="bl in backlinksData"
          :key="bl.source_path + ':' + bl.line"
          type="button"
          class="backlink-item"
          :title="bl.excerpt"
          @click="store.openPath(bl.source_path)"
        >
          <span class="backlink-file">{{ bl.source_path.split('/').pop() }}</span>
          <small class="backlink-line">line {{ bl.line }}</small>
          <span class="backlink-excerpt">{{ bl.excerpt }}</span>
        </button>
      </div>
      <p v-else class="sidebar-hint">No documents in this workspace link to <em>{{ active?.title }}</em>.</p>
      <div class="backlinks-group" v-if="unlinkedMentionsData.length" style="margin-top:12px">
        <h3>Unlinked mentions ({{ unlinkedMentionsData.length }})</h3>
        <button
          v-for="bl in unlinkedMentionsData"
          :key="bl.source_path + ':' + bl.line + ':unlinked'"
          type="button"
          class="backlink-item"
          :title="bl.excerpt"
          @click="store.openPath(bl.source_path)"
        >
          <span class="backlink-file">{{ bl.source_path.split('/').pop() }}</span>
          <small class="backlink-line">line {{ bl.line }}</small>
          <span class="backlink-excerpt">{{ bl.excerpt }}</span>
        </button>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  active,
  backlinksData,
  backlinksLoading,
  refreshBacklinks,
  unlinkedMentionsData,
} = _ctx;
</script>
