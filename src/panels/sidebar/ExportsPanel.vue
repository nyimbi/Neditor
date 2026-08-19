<template>
  <h2>Export</h2>
  <section class="export-profile-manager" aria-label="Export profiles">
    <h3>Profiles</h3>
    <label>
      Saved profile
      <select :value="store.activeExportProfileId" @change="selectExportProfile(inputValue($event))">
        <option value="">Current settings</option>
        <option v-for="profile in store.exportProfiles" :key="profile.id" :value="profile.id">
          {{ profile.name }}
        </option>
      </select>
    </label>
    <label>
      Profile name
      <input v-model="exportProfileName" type="text" />
    </label>
    <div class="export-actions">
      <button class="template-action-primary" type="button" @click="saveExportProfileFromPanel">Save profile</button>
      <button type="button" :disabled="!store.activeExportProfileId" @click="deleteActiveExportProfile">Delete profile</button>
    </div>
    <p v-if="activeExportProfile" class="sidebar-hint">{{ exportProfileSummary }}</p>
    <p v-else class="sidebar-hint">Save reusable HTML, PDF, Office, publishing, and brand settings for repeat exports.</p>
  </section>
  <label>
    Target
    <select v-model="store.exportTarget">
      <option value="html">HTML</option>
      <option value="pdf">PDF</option>
      <option value="docx">DOCX</option>
      <option value="pptx">PPTX</option>
      <option value="markdown-bundle">Markdown bundle</option>
      <option value="blog">Blog package</option>
      <option value="substack">Substack package</option>
      <option value="latex">LaTeX</option>
      <option value="google-docs">Google Docs package</option>
      <option value="epub">EPUB ebook</option>
    </select>
  </label>
  <section v-if="publicMetadataOptionsVisible" class="export-target-options" aria-label="Public export metadata options">
    <h3>{{ publicMetadataOptionsTitle }}</h3>
    <label>
      Language
      <input v-model="store.exportDefaults.htmlLanguage" type="text" placeholder="en" />
    </label>
    <label>
      Description
      <input v-model="store.exportDefaults.htmlDescription" type="text" />
    </label>
    <label>
      Canonical URL
      <input v-model="store.exportDefaults.canonicalUrl" type="url" />
    </label>
  </section>
  <section v-if="store.exportTarget === 'latex'" class="export-target-options" aria-label="LaTeX template options">
    <h3>LaTeX template</h3>
    <label>
      Template profile
      <select v-model="store.exportDefaults.latexTemplate">
        <option v-for="profile in availableLatexTemplateProfiles" :key="profile.id" :value="profile.id">
          {{ profile.label }}{{ profile.source === "custom" ? " (custom)" : "" }}
        </option>
      </select>
    </label>
    <p class="sidebar-hint">{{ activeLatexTemplateProfile.summary }}</p>
    <ul class="template-meta-summary" aria-label="LaTeX template best fit">
      <li v-for="item in activeLatexTemplateProfile.bestFor" :key="item">{{ item }}</li>
    </ul>
    <details class="custom-template-editor">
      <summary>Manage company LaTeX templates</summary>
      <section class="template-library-sync" aria-label="Portable LaTeX template library">
        <header>
          <h4>Portable library</h4>
          <span>{{ store.customLatexTemplates.length }} custom</span>
        </header>
        <p class="sidebar-hint">
          {{ latexTemplateWorkspaceSyncStatus || (store.workspaceRoot ? `Syncs with ${workspaceLatexTemplateLibraryPath(store.workspaceRoot)} for CLI and app reuse.` : "Open a workspace folder to sync .neditor/latex-templates.json.") }}
        </p>
        <div class="template-actions">
          <button type="button" :disabled="!store.workspaceRoot || latexTemplateWorkspaceSyncBusy" @click="syncWorkspaceLatexTemplates">
            {{ latexTemplateWorkspaceSyncBusy ? "Syncing..." : "Sync workspace library" }}
          </button>
          <button type="button" :disabled="!store.workspaceRoot || latexTemplateWorkspaceSyncBusy" @click="saveLatexTemplatesToWorkspace">
            Save library to workspace
          </button>
          <button type="button" @click="previewLatexTemplateLibraryJson">Show library JSON</button>
          <button type="button" :disabled="!latexTemplateLibraryJsonText.trim()" @click="importLatexTemplateLibraryJson">Import JSON below</button>
        </div>
        <textarea v-model="latexTemplateLibraryJsonText" rows="5" aria-label="Portable LaTeX template library JSON" placeholder="{ &quot;schema&quot;: &quot;neditor.workspace-latex-templates.v1&quot;, &quot;templates&quot;: [] }"></textarea>
      </section>
      <label>
        Name
        <input v-model="latexTemplateDraft.name" type="text" />
      </label>
      <label>
        Document class
        <input v-model="latexTemplateDraft.documentClass" type="text" placeholder="article, report, book, memoir" />
      </label>
      <label>
        Class options
        <input v-model="latexTemplateDraft.classOptions" type="text" placeholder="11pt,oneside" />
      </label>
      <label>
        Packages and preamble package lines
        <textarea v-model="latexTemplatePackagesDraft" rows="5" aria-label="LaTeX package lines"></textarea>
      </label>
      <label>
        Geometry
        <input v-model="latexTemplateDraft.geometry" type="text" placeholder="margin=1in" />
      </label>
      <label>
        Hyperref setup
        <input v-model="latexTemplateDraft.hypersetup" type="text" placeholder="colorlinks=true,linkcolor=blue,urlcolor=blue" />
      </label>
      <label>
        Header or house-style preamble
        <textarea v-model="latexTemplateDraft.header" rows="4" aria-label="LaTeX header preamble"></textarea>
      </label>
      <label>
        Best for
        <textarea v-model="latexTemplateBestForDraft" rows="3" aria-label="LaTeX template best for"></textarea>
      </label>
      <label><input v-model="latexTemplateDraft.chapterStyle" type="checkbox" /> Use chapter-style headings</label>
      <div class="template-actions">
        <button class="template-action-primary" type="button" @click="saveLatexTemplateDraft">Save template</button>
        <button type="button" @click="resetLatexTemplateDraft">Reset draft</button>
      </div>
      <article v-for="template in store.customLatexTemplates" :key="template.id" class="template-card">
        <header class="template-card-header">
          <div>
            <strong>{{ template.name }}</strong>
            <small>{{ template.documentClass }} / {{ template.classOptions }}</small>
          </div>
        </header>
        <p>{{ template.summary || "Custom LaTeX template profile." }}</p>
        <div class="template-actions">
          <button type="button" @click="editLatexTemplate(template.id)">Edit</button>
          <button type="button" @click="store.deleteCustomLatexTemplate(template.id)">Delete</button>
        </div>
      </article>
    </details>
  </section>
  <section
    v-if="exportDistributionChecklist.length"
    class="export-metadata-checklist"
    aria-label="Distribution metadata checklist"
  >
    <header>
      <h3>Target checklist</h3>
      <span>{{ exportDistributionChecklistSummary }}</span>
    </header>
    <p>{{ exportDistributionChecklistHelp }}</p>
    <button type="button" @click="applyExportMetadataScaffold">Add suggested metadata</button>
    <article
      v-for="item in exportDistributionChecklist"
      :key="item.id"
      class="snapshot-row"
      :data-status="item.status"
    >
      <strong>{{ item.label }}</strong>
      <p>{{ item.detail }}</p>
      <small>{{ item.suggestion }}</small>
    </article>
  </section>
  <section class="export-assistance-panel" aria-label="AI export readiness assistance">
    <header>
      <div>
        <h3>AI Export Assistance</h3>
        <span>Suggested next answers for metadata, readiness diagnostics, and artifact evidence.</span>
      </div>
      <button type="button" @click="appendAllExportStepAssistance">Use all</button>
    </header>
    <article v-for="item in exportStepAssistance" :key="item.stepId">
      <div>
        <small>{{ item.stepLabel }}</small>
        <p>{{ item.suggestedAnswer }}</p>
        <p class="sidebar-hint">{{ item.rationale }}</p>
        <ul>
          <li v-for="signal in item.contextSignals" :key="signal">{{ signal }}</li>
        </ul>
      </div>
      <button type="button" @click="appendExportStepAssistance(item)">{{ item.actionLabel }}</button>
    </article>
    <label>
      Export readiness notes
      <textarea v-model="exportReadinessNotes" rows="4" aria-label="Export readiness notes"></textarea>
    </label>
    <div class="reference-actions">
      <button type="button" :disabled="!exportReadinessNotes.trim()" @click="insertExportReadinessNotes">Insert notes</button>
      <button type="button" @click="prepareForExport">Run readiness</button>
    </div>
  </section>
  <section v-if="publishingPanelVisible" class="publishing-handoff-panel" aria-label="Publishing handoff">
    <header>
      <div>
        <h3>Publish and distribute</h3>
        <span>{{ publishingTargetHelpText }} Uses dry-run previews and session-only endpoint tokens.</span>
      </div>
      <button type="button" :disabled="store.exportBusy" @click="preparePublishingHandoff">Prepare</button>
    </header>
    <div class="publishing-profile-row">
      <label>
        Saved destination
        <select :value="store.activePublishingDestinationId" @change="selectPublishingDestination(inputValue($event))">
          <option value="">Current destination</option>
          <option v-for="profile in store.publishingDestinationProfiles" :key="profile.id" :value="profile.id">{{ profile.name }}</option>
        </select>
      </label>
      <label>
        Destination name
        <input v-model="publishingDestinationName" type="text" />
      </label>
      <button type="button" @click="savePublishingDestinationProfile">Save destination</button>
      <button type="button" :disabled="!activePublishingDestination" @click="deleteActivePublishingDestination">Delete</button>
    </div>
    <div class="publishing-grid">
      <label>
        Destination
        <select v-model="publishingTargetKind">
          <option v-for="target in publishingTargetOptions" :key="target.value" :value="target.value">{{ target.label }}</option>
        </select>
      </label>
      <label>
        Content
        <select v-model="publishingContentFormat">
          <option value="html">HTML</option>
          <option value="markdown">Markdown</option>
          <option value="text">Plain text</option>
        </select>
      </label>
      <label>
        Endpoint URL
        <input v-model="publishingEndpointUrl" type="url" placeholder="https://cms.example.com/webhook/neditor" />
      </label>
      <label>
        Auth header
        <input v-model="publishingAuthHeaderName" type="text" placeholder="Authorization" />
      </label>
      <label>
        Session token
        <input v-model="publishingAuthToken" type="password" autocomplete="off" placeholder="Stored only in this session" />
      </label>
      <label class="checkbox-row"><input v-model="publishingDryRun" type="checkbox" /> Dry run until I explicitly send</label>
    </div>
    <article class="publishing-summary" :data-status="publishingRequestPreview.canSend ? 'ready' : 'needs-review'">
      <strong>{{ publishingHandoff.title }}</strong>
      <p>{{ publishingHandoff.description || "No public summary yet." }}</p>
      <small>Slug {{ publishingHandoff.slug }} | {{ publishingHandoff.readinessLabel }} | {{ publishingHandoff.tags.length ? publishingHandoff.tags.join(", ") : "no tags" }}</small>
    </article>
    <div class="publishing-checklist">
      <article v-for="item in publishingHandoff.checklist" :key="item.id" class="snapshot-row" :data-status="item.status">
        <strong>{{ item.label }}</strong>
        <p>{{ item.detail }}</p>
      </article>
    </div>
    <section class="publishing-preflight" aria-label="Publishing preflight audit">
      <header>
        <div>
          <strong>Publishing preflight</strong>
          <span>{{ publishingPreflightReport.blockers.length }} blocker{{ publishingPreflightReport.blockers.length === 1 ? "" : "s" }} | {{ publishingPreflightReport.needsReview.length }} review item{{ publishingPreflightReport.needsReview.length === 1 ? "" : "s" }}</span>
        </div>
      </header>
      <article v-for="item in publishingPreflightReport.items" :key="item.id" class="snapshot-row" :data-status="item.status">
        <strong>{{ item.label }}</strong>
        <p>{{ item.status }} | {{ item.detail }}</p>
      </article>
    </section>
    <div class="reference-actions">
      <button type="button" @click="copyPublishingPayload">Copy payload</button>
      <button type="button" @click="copyPublishingContent">Copy content</button>
      <button type="button" @click="insertPublishingPreflightAudit">Insert preflight</button>
      <button type="button" @click="copyPublishingPreflightAudit">Copy preflight</button>
      <button
        type="button"
        :disabled="publishingBusy || publishingDryRun || !publishingRequestPreview.canSend"
        @click="sendPublishingPayload"
      >
        Send to endpoint
      </button>
    </div>
    <p v-for="warning in publishingRequestPreview.warnings" :key="warning" class="sidebar-hint">{{ warning }}</p>
    <textarea :value="publishingRequestPreview.bodyText" rows="7" readonly aria-label="Publishing payload preview"></textarea>
  </section>
  <label><input v-model="store.exportDefaults.includeManifest" type="checkbox" /> Export manifest</label>
  <label><input v-model="store.exportDefaults.includeStyles" type="checkbox" /> Include styles</label>
  <label><input v-model="store.exportDefaults.includeSyntaxHighlighting" type="checkbox" /> Syntax highlighting</label>
  <label><input v-model="store.exportDefaults.coverPage" type="checkbox" /> Cover page</label>
  <label><input v-model="store.exportDefaults.pageNumbers" type="checkbox" /> Page numbers</label>
  <label>
    Layout preset
    <select v-model="store.exportDefaults.layoutPreset">
      <option value="business">Business</option>
      <option value="compact">Compact</option>
      <option value="presentation">Presentation</option>
    </select>
  </label>
  <section class="print-preview-card" aria-label="Print preview controls">
    <header>
      <div>
        <strong>Print preview</strong>
        <span>{{ printPreviewReport.summary }}</span>
      </div>
      <button type="button" :class="{ active: printPreviewEnabled }" title="Show approximate page geometry, pagination, margins, columns, and page-break warnings in the preview" @click="() => togglePrintPreview()">
        {{ printPreviewEnabled ? "Hide" : "Show" }}
      </button>
    </header>
    <div class="print-preview-metrics" aria-label="Print preview export metrics">
      <span><strong>{{ printPreviewReport.estimatedPages }}</strong> pages</span>
      <span><strong>{{ printPreviewReport.wordCount }}</strong> words</span>
      <span><strong>{{ printPreviewReport.columns }}</strong> columns</span>
      <span><strong>{{ printPreviewReport.margins }}</strong> margins</span>
    </div>
    <ul v-if="printPreviewReport.warnings.length" aria-label="Print preview export warnings">
      <li v-for="warning in printPreviewReport.warnings" :key="warning">{{ warning }}</li>
    </ul>
    <p v-else class="sidebar-hint">No print-flow warnings from the approximate preview model.</p>
  </section>
  <section class="export-visual-qa-dashboard" :data-status="exportVisualQaDashboard.status" aria-label="Export visual QA dashboard">
    <header>
      <div>
        <h3>Visual QA</h3>
        <span>{{ exportVisualQaDashboard.summary }}</span>
      </div>
      <strong>{{ exportVisualQaDashboard.status }}</strong>
    </header>
    <div class="export-visual-qa-metrics" aria-label="Export visual QA status counts">
      <span><strong>{{ exportVisualQaDashboard.counts.ready }}</strong> ready</span>
      <span><strong>{{ exportVisualQaDashboard.counts["needs-review"] }}</strong> review</span>
      <span><strong>{{ exportVisualQaDashboard.counts.blocked }}</strong> blocked</span>
      <span><strong>{{ exportVisualQaDashboard.counts["not-run"] }}</strong> not run</span>
    </div>
    <article class="export-visual-qa-current" :data-status="exportVisualQaCurrentRow.status">
      <strong>{{ exportVisualQaCurrentRow.label }}</strong>
      <p>{{ exportVisualQaCurrentRow.nextAction }}</p>
      <small>{{ exportVisualQaCurrentRow.evidence.join(" | ") || exportVisualQaCurrentRow.blockers.join(" | ") || "No current output proof yet." }}</small>
    </article>
    <details>
      <summary>Target evidence</summary>
      <article
        v-for="row in exportVisualQaDashboard.rows"
        :key="row.target"
        class="export-visual-qa-row"
        :data-status="row.status"
      >
        <header>
          <strong>{{ row.label }}</strong>
          <span>{{ row.status }}</span>
        </header>
        <p>{{ row.nextAction }}</p>
        <ul v-if="row.blockers.length">
          <li v-for="blocker in row.blockers" :key="blocker">{{ blocker }}</li>
        </ul>
        <small>{{ row.evidence.join(" | ") || row.checks.slice(0, 2).join(" | ") }}</small>
      </article>
    </details>
    <div class="reference-actions">
      <button type="button" :disabled="store.exportBusy" title="Run target-aware export readiness before judging the selected output target" @click="prepareForExport">Run readiness</button>
      <button type="button" title="Insert this export visual QA dashboard into the Markdown document for reviewer handoff" @click="insertExportVisualQaReport">Insert QA report</button>
      <button type="button" title="Show approximate page geometry, pagination, margins, columns, and print-flow warnings" @click="togglePrintPreview(true)">Show print preview</button>
    </div>
  </section>
  <label><input v-model="store.exportDefaults.includeComments" type="checkbox" /> Include comments</label>
  <label><input v-model="store.exportDefaults.includeProvenance" type="checkbox" /> Include AI provenance</label>
  <label><input v-model="store.exportDefaults.includeGlossary" type="checkbox" /> Include glossary</label>
  <label><input v-model="store.exportDefaults.includeAgenda" type="checkbox" /> PPTX agenda</label>
  <h3>Presentation settings</h3>
  <div class="pres-theme-row">
    <span class="compact-label">Theme</span>
    <div class="pres-theme-grid">
      <button
        v-for="theme in PRESENTATION_THEMES" :key="theme.id"
        type="button" class="pres-theme-btn"
        :class="{ 'pres-theme-active': store.presentationTheme === theme.id }"
        :title="theme.label"
        :style="{ background: 'linear-gradient(135deg,' + theme.previewStart + ',' + theme.previewEnd + ')' }"
        @click="store.presentationTheme = theme.id; void store.persistWorkspace()"
      ><span class="pres-theme-lbl">{{ theme.label }}</span></button>
    </div>
  </div>
  <label>Transition<select v-model="store.presentationTransition" @change="void store.persistWorkspace()"><option v-for="t in PRESENTATION_TRANSITIONS" :key="t.id" :value="t.id">{{ t.label }}</option></select></label>
  <div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:8px">
    <button type="button" @click="openPresenterView">Presenter view</button>
    <button type="button" :disabled="store.exportBusy" @click="exportDocumentAs('html-slides')">Export HTML slides</button>
  </div>
  <div class="export-actions">
    <button class="template-action-primary" type="button" :disabled="store.exportBusy" @click="exportDocumentAs('html')">
      <span class="button-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" focusable="false">
          <path v-for="path in toolbarIconPaths('html')" :key="path" :d="path"></path>
        </svg>
      </span>
      Export HTML
    </button>
    <button type="button" :disabled="store.exportBusy" @click="exportDocumentAs('epub')">
      <span class="button-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" focusable="false">
          <path v-for="path in toolbarIconPaths('epub')" :key="path" :d="path"></path>
        </svg>
      </span>
      Export EPUB
    </button>
    <button type="button" :disabled="store.exportBusy" @click="prepareForExport">Prepare for export</button>
    <button type="button" :disabled="store.exportBusy" @click="exportDocument">Export document</button>
  </div>
  <article v-if="store.exportReadiness" class="readiness" :class="{ ready: store.exportReadiness.ready }">
    <strong>{{ store.exportReadiness.ready ? "Ready" : "Needs attention" }}</strong>
    <p>{{ store.exportReadiness.error_count }} errors, {{ store.exportReadiness.warning_count }} warnings, {{ store.exportReadiness.info_count }} info</p>
    <p>{{ readinessLayoutSummary }}</p>
    <ol v-if="store.exportReadiness.progress_steps.length" class="progress-steps" aria-label="Export readiness progress">
      <li v-for="step in store.exportReadiness.progress_steps" :key="`readiness-${step.id}`">
        <strong>{{ step.label }}</strong>
        <span>{{ step.state }}</span>
        <small>{{ step.detail }}</small>
      </li>
    </ol>
  </article>
  <section v-if="store.exportReadiness?.diagnostics.length" class="export-diagnostic-report" role="list" aria-label="Export readiness diagnostics">
    <article
      v-for="diagnostic in store.exportReadiness.diagnostics"
      :key="`${diagnostic.severity}-${diagnostic.source_file || ''}-${diagnostic.line || ''}-${diagnostic.message}`"
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
  <section v-if="store.lastExportOutputPath || store.lastExportDiagnostics.length" class="export-result" aria-label="Export result">
    <h3>Last export</h3>
    <p v-if="store.lastExportOutputPath">Output: {{ store.lastExportOutputPath }}</p>
    <p v-if="store.lastExportManifestPath">Manifest: {{ store.lastExportManifestPath }}</p>
    <ol v-if="store.lastExportProgressSteps.length" class="progress-steps" aria-label="Last export progress">
      <li v-for="step in store.lastExportProgressSteps" :key="`export-${step.id}`">
        <strong>{{ step.label }}</strong>
        <span>{{ step.state }}</span>
        <small>{{ step.detail }}</small>
      </li>
    </ol>
    <section v-if="store.lastExportDiagnostics.length" class="export-diagnostic-report" role="list" aria-label="Last export diagnostics">
      <article
        v-for="diagnostic in store.lastExportDiagnostics"
        :key="`export-${diagnostic.severity}-${diagnostic.source_file || ''}-${diagnostic.line || ''}-${diagnostic.message}`"
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
      </article>
    </section>
  </section>
  <h3>Manifest</h3>
  <pre>{{ manifestPreview }}</pre>
  <h3>Snapshots</h3>
  <button type="button" @click="store.listSnapshots">Refresh snapshots</button>
  <article v-for="snapshot in store.snapshots" :key="snapshot.snapshot_path" class="snapshot-row">
    <p>{{ snapshot.label || "snapshot" }}</p>
    <small>{{ snapshot.created_at || snapshot.snapshot_path }}</small>
    <small>{{ snapshot.document_version || "unversioned" }} | {{ snapshot.status || "unknown" }} | {{ snapshot.author || "unknown author" }}</small>
    <button type="button" @click="restoreSnapshot(snapshot.snapshot_path)">Restore</button>
  </article>
</template>

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  PRESENTATION_THEMES,
  PRESENTATION_TRANSITIONS,
  activeExportProfile,
  activeLatexTemplateProfile,
  activePublishingDestination,
  appendAllExportStepAssistance,
  appendExportStepAssistance,
  applyExportMetadataScaffold,
  availableLatexTemplateProfiles,
  canNavigateDiagnostic,
  copyPublishingContent,
  copyPublishingPayload,
  copyPublishingPreflightAudit,
  deleteActiveExportProfile,
  deleteActivePublishingDestination,
  diagnosticAnnouncementLabel,
  diagnosticLocation,
  editLatexTemplate,
  exportDistributionChecklist,
  exportDistributionChecklistHelp,
  exportDistributionChecklistSummary,
  exportDocument,
  exportDocumentAs,
  exportProfileName,
  exportProfileSummary,
  exportReadinessNotes,
  exportStepAssistance,
  exportVisualQaCurrentRow,
  exportVisualQaDashboard,
  goToSourceTarget,
  importLatexTemplateLibraryJson,
  inputValue,
  insertExportReadinessNotes,
  insertExportVisualQaReport,
  insertPublishingPreflightAudit,
  latexTemplateBestForDraft,
  latexTemplateDraft,
  latexTemplateLibraryJsonText,
  latexTemplatePackagesDraft,
  latexTemplateWorkspaceSyncBusy,
  latexTemplateWorkspaceSyncStatus,
  manifestPreview,
  openPresenterView,
  prepareForExport,
  preparePublishingHandoff,
  previewLatexTemplateLibraryJson,
  printPreviewEnabled,
  printPreviewReport,
  publicMetadataOptionsTitle,
  publicMetadataOptionsVisible,
  publishingAuthHeaderName,
  publishingAuthToken,
  publishingBusy,
  publishingContentFormat,
  publishingDestinationName,
  publishingDryRun,
  publishingEndpointUrl,
  publishingHandoff,
  publishingPanelVisible,
  publishingPreflightReport,
  publishingRequestPreview,
  publishingTargetHelpText,
  publishingTargetKind,
  publishingTargetOptions,
  readinessLayoutSummary,
  resetLatexTemplateDraft,
  restoreSnapshot,
  saveExportProfileFromPanel,
  saveLatexTemplateDraft,
  saveLatexTemplatesToWorkspace,
  savePublishingDestinationProfile,
  selectExportProfile,
  selectPublishingDestination,
  sendPublishingPayload,
  syncWorkspaceLatexTemplates,
  togglePrintPreview,
  toolbarIconPaths,
  workspaceLatexTemplateLibraryPath,
} = _ctx;
</script>
