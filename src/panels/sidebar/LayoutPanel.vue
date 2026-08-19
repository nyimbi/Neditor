<template>
  <h2>Layout Advisor</h2>
  <p class="sidebar-hint">Create polished sections, catch export-risky layout choices, and keep page flow intentional before PDF/DOCX delivery.</p>
  <section class="layout-advisor-summary" aria-label="Layout advisor summary">
    <article class="snapshot-row" :data-status="layoutAdvisorStatus">
      <strong>{{ layoutAdvisorHeadline }}</strong>
      <p>{{ layoutAdvisorDetail }}</p>
      <small>{{ readinessLayoutSummary }}</small>
    </article>
    <div class="release-readiness-actions">
      <button type="button" @click="runLayoutQualityReview">Review layout</button>
      <button type="button" @click="insertDocumentLayoutPreset('two-column-section')">Insert two-column section</button>
      <button type="button" @click="insertDocumentLayoutPreset('wide-landscape-section')">Insert wide section</button>
    </div>
  </section>
  <section class="cover-builder" aria-label="Professional cover builder">
    <header>
      <div>
        <h3>Cover Builder</h3>
        <small>{{ coverBuilderSummary }}</small>
      </div>
      <button type="button" title="Reload cover fields from current metadata and business identity" @click="resetCoverBuilderDraft">
        Load defaults
      </button>
    </header>
    <div class="cover-builder-grid">
      <label>
        Title
        <input v-model="coverBuilderDraft.title" :placeholder="coverBuilderDefaults.title" />
      </label>
      <label>
        Subtitle
        <input v-model="coverBuilderDraft.subtitle" :placeholder="coverBuilderDefaults.subtitle" />
      </label>
      <label>
        Client
        <input v-model="coverBuilderDraft.client" :placeholder="coverBuilderDefaults.client" />
      </label>
      <label>
        Prepared by
        <input v-model="coverBuilderDraft.preparedBy" :placeholder="coverBuilderDefaults.preparedBy" />
      </label>
      <label>
        Date
        <input v-model="coverBuilderDraft.date" type="date" :placeholder="coverBuilderDefaults.date" />
      </label>
      <label>
        Confidentiality
        <input v-model="coverBuilderDraft.confidentiality" :placeholder="coverBuilderDefaults.confidentiality" />
      </label>
      <label>
        Status
        <select v-model="coverBuilderDraft.status">
          <option v-for="status in releaseStatuses" :key="status" :value="status">{{ status }}</option>
        </select>
      </label>
      <label>
        Version
        <input v-model="coverBuilderDraft.version" :placeholder="coverBuilderDefaults.version" />
      </label>
    </div>
    <div class="cover-builder-actions">
      <button type="button" title="Write cover metadata to front matter and enable cover page export" @click="applyCoverBuilderMetadata">Apply metadata</button>
      <button type="button" title="Insert a Markdown cover section with the current cover values" @click="insertCoverBuilderSection">Insert cover section</button>
      <button type="button" title="Apply metadata and insert the cover section in one step" @click="applyCoverBuilderPackage">Apply and insert</button>
    </div>
  </section>
  <section class="layout-preset-grid" aria-label="Business layout presets">
    <article v-for="preset in documentLayoutPresets" :key="preset.id" class="snapshot-row" data-status="improve">
      <strong>{{ preset.label }}</strong>
      <p>{{ preset.summary }}</p>
      <small>{{ preset.commandName }}</small>
      <button type="button" @click="insertDocumentLayoutPreset(preset.id)">Insert</button>
    </article>
  </section>
  <section class="layout-advisor-findings" aria-label="Layout quality findings">
    <header>
      <h3>Layout quality</h3>
      <span>{{ layoutQualityRecommendations.length }} finding{{ layoutQualityRecommendations.length === 1 ? "" : "s" }}</span>
    </header>
    <article v-if="!layoutQualityRecommendations.length" class="snapshot-row" data-status="pass">
      <strong>No deterministic layout risks</strong>
      <p>Wide tables, column gutters, and dense-section resets look intentional from the current Markdown.</p>
      <small>Run rendered export review for final page geometry.</small>
    </article>
    <article
      v-for="item in layoutQualityRecommendations"
      :key="item.id"
      class="snapshot-row"
      :data-status="item.severity"
    >
      <strong>{{ item.label }}</strong>
      <p>{{ item.recommendation }}</p>
      <small>{{ item.action }}</small>
    </article>
  </section>
</template>

<script setup lang="ts">
import { inject } from 'vue';

const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  applyCoverBuilderMetadata,
  applyCoverBuilderPackage,
  coverBuilderDefaults,
  coverBuilderDraft,
  coverBuilderSummary,
  documentLayoutPresets,
  insertCoverBuilderSection,
  insertDocumentLayoutPreset,
  layoutAdvisorDetail,
  layoutAdvisorHeadline,
  layoutAdvisorStatus,
  layoutQualityRecommendations,
  readinessLayoutSummary,
  releaseStatuses,
  resetCoverBuilderDraft,
  runLayoutQualityReview,
} = _ctx;
</script>
