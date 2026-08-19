<template>
  <h2>Versioning</h2>
  <article v-if="store.gitStatus?.inside_repo" class="snapshot-row">
    <p>{{ store.gitStatus.branch || "detached" }} | {{ store.gitStatus.dirty ? "dirty" : "clean" }}</p>
    <small v-for="line in store.gitStatus.summary" :key="line">{{ line }}</small>
  </article>
  <section v-else class="git-free-versioning" aria-label="Git-free versioning guidance">
    <header>
      <strong>Snapshot-first document history</strong>
      <span>{{ versioningModeLabel }}</span>
    </header>
    <p>
      This document is outside Git, so NEditor keeps recovery points locally. Use snapshots for business drafts,
      approvals, and pre-export rollback without configuring developer tooling.
    </p>
    <ol>
      <li v-for="step in gitFreeVersioningPlan" :key="step">{{ step }}</li>
    </ol>
    <section class="git-free-controls" aria-label="Snapshot recovery controls">
      <label>
        Snapshot storage
        <select v-model="store.snapshotStorage" aria-label="Versioning snapshot storage">
          <option value="app-data">Private app data</option>
          <option value="project-local">Project .neditor folder</option>
        </select>
      </label>
      <label><input v-model="store.autoSnapshot" type="checkbox" /> Automatic recovery snapshots</label>
      <label>
        Recovery interval
        <input v-model.number="store.snapshotIntervalMs" type="number" min="30000" max="3600000" step="30000" />
      </label>
    </section>
    <button type="button" @click="createRecoverySnapshot">Create recovery snapshot</button>
  </section>
  <template v-if="store.gitStatus?.inside_repo">
    <label>
      Commit message
      <input v-model="store.commitMessage" placeholder="Update document" />
    </label>
    <button type="button" @click="store.commitActive()">Commit document</button>
    <label>
      Release tag
      <input v-model="store.releaseTag" placeholder="v1.0.0" />
    </label>
    <button type="button" @click="store.tagActiveRelease()">Tag release</button>
    <button type="button" @click="store.refreshGitDiff">Refresh diff</button>
    <h3>Diff</h3>
    <pre>{{ store.gitDiffText || "No uncommitted diff." }}</pre>
    <h3>History</h3>
    <article v-for="entry in store.gitHistory" :key="entry.revision" class="snapshot-row">
      <p>{{ entry.subject }}</p>
      <small>{{ entry.revision.slice(0, 12) }} | {{ entry.author }} | {{ entry.date }}</small>
      <button type="button" @click="store.restoreGitRevision(entry.revision)">Restore</button>
    </article>
  </template>
  <h3>Snapshots</h3>
  <button type="button" @click="snapshotActive">Create snapshot</button>
  <button type="button" @click="store.listSnapshots">Refresh snapshots</button>
  <article v-for="snapshot in store.snapshots" :key="`version-${snapshot.snapshot_path}`" class="snapshot-row">
    <p>{{ snapshot.label || "snapshot" }}</p>
    <small>{{ snapshot.created_at || snapshot.snapshot_path }}</small>
    <small>{{ snapshot.snapshot_path }}</small>
    <small>{{ snapshot.document_version || "unversioned" }} | {{ snapshot.status || "unknown" }} | {{ snapshot.author || "unknown author" }}</small>
    <button type="button" @click="restoreSnapshot(snapshot.snapshot_path)">Restore snapshot</button>
  </article>
</template>

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  createRecoverySnapshot,
  gitFreeVersioningPlan,
  restoreSnapshot,
  snapshotActive,
  versioningModeLabel,
} = _ctx;
</script>
