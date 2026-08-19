<template>
  <h2>Workspace</h2>
  <!-- Pinned files -->
  <section v-if="store.pinnedFiles.length" class="pinned-files" aria-label="Pinned files">
    <h3>Pinned</h3>
    <div class="pinned-files-list">
      <div v-for="path in store.pinnedFiles" :key="path" class="pinned-file-item">
        <button type="button" class="pinned-file-name" @click="store.openPath(path)" :title="path">
          📌 {{ path.split('/').pop() }}
        </button>
        <button type="button" class="pin-remove" @click="unpinFile(path)" title="Unpin">×</button>
      </div>
    </div>
  </section>
  <!-- PARA setup -->
  <div class="files-actions" style="display:flex;gap:4px;flex-wrap:wrap;margin-bottom:6px">
    <button type="button" @click="setupPARAWorkspace" title="Create PARA folder structure (Projects/Areas/Resources/Archives)">PARA setup</button>
    <button type="button" @click="active?.path && pinFile(active.path)" :disabled="!active?.path || store.pinnedFiles.includes(active?.path || '')">📌 Pin current</button>
  </div>
  <div class="workspace-search-box">
    <input
      v-model="workspaceSearchQuery"
      type="search"
      placeholder="Search workspace..."
      class="workspace-search-input"
      @keydown.enter="runWorkspaceSearch"
      aria-label="Search workspace files"
    />
    <button type="button" :disabled="workspaceSearchBusy" @click="runWorkspaceSearch">
      {{ workspaceSearchBusy ? '…' : '↵' }}
    </button>
  </div>
  <div v-if="workspaceSearchResults.length" class="workspace-search-results">
    <div
      v-for="result in workspaceSearchResults.slice(0, 50)"
      :key="result.path + result.line"
      class="ws-search-result"
      role="button"
      tabindex="0"
      @click="openSearchResult(result)"
      @keydown.enter="openSearchResult(result)"
    >
      <span class="ws-sr-path">{{ result.path }}</span>
      <span class="ws-sr-line">:{{ result.line }}</span>
      <span class="ws-sr-excerpt">{{ result.excerpt }}</span>
    </div>
  </div>
  <button type="button" @click="openFolder">Open folder</button>
  <button v-if="store.workspaceRoot" type="button" @click="store.refreshWorkspace">Refresh</button>
  <p v-if="store.workspaceRoot" class="workspace-root">{{ store.workspaceRoot }}</p>
  <p v-else>Open a folder to browse project files.</p>
  <section class="document-set-manager" aria-label="Document set manager">
    <header>
      <div>
        <h3>Document Sets</h3>
        <small>{{ documentSetGroups.length }} open sets</small>
      </div>
    </header>
    <label>
      Active document set
      <input v-model="documentSetDraft" aria-label="Active document set" placeholder="Board Pack, Client Deliverable" />
    </label>
    <div class="document-set-actions">
      <button type="button" :disabled="!documentSetDraft.trim()" @click="assignActiveDocumentSet(documentSetDraft)">Assign active</button>
      <button type="button" :disabled="!activeDocumentSet" @click="clearActiveDocumentSet">Remove active</button>
    </div>
    <label v-if="activeDocumentSet">
      Rename open set
      <input v-model="documentSetRenameDraft" aria-label="Rename active document set" placeholder="New set name" />
    </label>
    <button
      v-if="activeDocumentSet"
      type="button"
      :disabled="!documentSetRenameDraft.trim() || documentSetRenameDraft.trim() === activeDocumentSet"
      @click="renameActiveDocumentSet"
    >
      Rename all open set tabs
    </button>
    <div v-if="activeDocumentSet" class="document-set-actions">
      <button type="button" @click="insertActiveDocumentSetManifest">Insert manifest</button>
      <button type="button" @click="copyActiveDocumentSetManifest">Copy manifest</button>
    </div>
    <div v-if="documentSetGroups.length" class="document-set-list" role="list" aria-label="Open document sets">
      <article v-for="group in documentSetGroups" :key="group.key" role="listitem">
        <div>
          <strong>{{ group.label }}</strong>
          <small>{{ group.documents.length }} open document{{ group.documents.length === 1 ? "" : "s" }}</small>
        </div>
        <button type="button" :disabled="!active.path || activeDocumentSet === group.label" @click="assignActiveDocumentSet(group.label)">Add active</button>
      </article>
    </div>
    <p v-else class="sidebar-hint">Use document sets to keep board packs, proposals, appendices, and review bundles grouped together.</p>
  </section>
  <button
    v-for="entry in store.workspaceFiles"
    :key="entry.path"
    class="file-row"
    :class="{ directory: entry.kind === 'directory', active: entry.path === active.path }"
    :style="{ paddingLeft: `${entry.depth * 12 + 8}px` }"
    type="button"
    @click="entry.kind === 'directory' ? undefined : store.openPath(entry.path)"
  >
    <span>{{ entry.kind === "directory" ? ">" : "-" }}</span>
    <span>{{ entry.name }}</span>
  </button>
</template>

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  active,
  activeDocumentSet,
  assignActiveDocumentSet,
  clearActiveDocumentSet,
  copyActiveDocumentSetManifest,
  documentSetDraft,
  documentSetGroups,
  documentSetRenameDraft,
  insertActiveDocumentSetManifest,
  openFolder,
  openSearchResult,
  pinFile,
  renameActiveDocumentSet,
  runWorkspaceSearch,
  setupPARAWorkspace,
  unpinFile,
  workspaceSearchBusy,
  workspaceSearchQuery,
  workspaceSearchResults,
} = _ctx;
</script>
