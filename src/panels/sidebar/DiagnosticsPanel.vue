<template>
  <h2>Diagnostics</h2>

  <!-- ── Compile group ──────────────────────────────────────────────────── -->
  <section class="diagnostics-group" aria-label="Compile diagnostics">
    <header class="diagnostics-group-header">
      <h3>Compile</h3>
      <span v-if="compileDiagnostics.length" class="diagnostics-count" :data-severity="compileHasErrors ? 'error' : 'warning'">
        {{ compileDiagnostics.length }}
      </span>
    </header>
    <p v-if="!compileDiagnostics.length" class="sidebar-hint">No compile diagnostics.</p>
    <article
      v-for="diagnostic in compileDiagnostics"
      :key="`${diagnostic.severity}-${diagnostic.source_file || ''}-${diagnostic.line || ''}-${diagnostic.column || ''}-${diagnostic.message}`"
      class="diagnostic"
      :class="diagnostic.severity"
      role="listitem"
      :aria-label="diagnosticAnnouncementLabel(diagnostic)"
    >
      <strong>{{ diagnostic.severity }}</strong>
      <p>{{ diagnostic.message }}</p>
      <small v-if="diagnosticLocation(diagnostic)">{{ diagnosticLocation(diagnostic) }}</small>
      <small v-if="diagnostic.suggestion">{{ diagnostic.suggestion }}</small>
      <ul v-if="diagnostic.related.length" class="diagnostic-related">
        <li v-for="related in diagnostic.related" :key="related">{{ related }}</li>
      </ul>
      <button v-if="canNavigateDiagnostic(diagnostic)" type="button" @click="goToSourceTarget(diagnostic)">Go to source</button>
    </article>
  </section>

  <!-- ── Transform trust group ──────────────────────────────────────────── -->
  <section v-if="externalTransformTrustPrompts.length" class="diagnostics-group" aria-label="Transform trust prompts">
    <header class="diagnostics-group-header">
      <h3>Trust</h3>
      <span class="diagnostics-count" data-severity="warning">{{ externalTransformTrustPrompts.length }}</span>
    </header>
    <article
      v-for="prompt in externalTransformTrustPrompts"
      :key="prompt.name"
      class="diagnostic warning"
      role="listitem"
      :aria-label="`Trust required for ${prompt.name}`"
    >
      <strong>Trust required</strong>
      <p>Transform engine <strong>{{ prompt.name }}</strong> is used in this document but not yet trusted.</p>
      <small>{{ prompt.path }}</small>
      <small v-if="prompt.securitySummary">{{ prompt.securitySummary }}</small>
      <div class="diagnostic-actions">
        <button type="button" @click="trustTransformEngine(prompt.name)">Trust {{ prompt.name }}</button>
        <button type="button" @click="reviewTransformEngineSettings(prompt.name)">Review settings</button>
      </div>
    </article>
  </section>

  <!-- ── Engine group ───────────────────────────────────────────────────── -->
  <section class="diagnostics-group" aria-label="Compiler output inventory">
    <header class="diagnostics-group-header">
      <h3>Engine</h3>
      <span class="diagnostics-count" :data-severity="engineIssues.length ? 'warning' : 'ok'">{{ engineIssues.length }}</span>
    </header>
    <p v-if="!engineIssues.length" class="sidebar-hint">All compiler outputs present.</p>
    <article
      v-for="item in engineIssues"
      :key="item.label"
      class="diagnostic warning"
      role="listitem"
    >
      <strong>Missing output</strong>
      <p>{{ item.label }}</p>
      <small>{{ item.detail }}</small>
    </article>
  </section>

  <!-- ── Advanced ───────────────────────────────────────────────────────── -->
  <CollapsibleAdvanced panel-id="diagnostics" label="Advanced">
    <section class="compiler-output-inventory" aria-label="Compiler output inventory">
      <h4>Output inventory</h4>
      <article v-for="item in compilerOutputInventory" :key="item.label" class="snapshot-row" :data-status="item.status">
        <p>{{ item.label }}</p>
        <small>{{ item.status }} | {{ item.detail }}</small>
      </article>
    </section>
    <div class="reference-actions">
      <button type="button" @click="copyPreviewErrorForSupport">Copy error report</button>
    </div>
  </CollapsibleAdvanced>
</template>

<script setup lang="ts">
import { computed, inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';
import CollapsibleAdvanced from '../../components/CollapsibleAdvanced.vue';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  active,
  canNavigateDiagnostic,
  compilerOutputInventory,
  copyPreviewErrorForSupport,
  diagnosticAnnouncementLabel,
  diagnosticLocation,
  externalTransformTrustPrompts,
  goToSourceTarget,
  reviewTransformEngineSettings,
  trustTransformEngine,
} = _ctx;

const compileDiagnostics = computed(() => active.value?.compile?.diagnostics || []);
const compileHasErrors = computed(() => compileDiagnostics.value.some((d: { severity: string }) => d.severity === 'error'));
const engineIssues = computed(() =>
  (compilerOutputInventory.value || []).filter((item: { status: string }) => item.status === 'missing'),
);
</script>
