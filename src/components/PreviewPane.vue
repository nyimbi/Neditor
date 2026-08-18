<template>
  <section
    ref="rootEl"
    id="live-preview"
    v-show="store.mode !== 'source' && store.mode !== 'focus' && store.mode !== 'outline'"
    class="preview-pane"
    :data-preview-theme="store.previewTheme"
    aria-label="Live preview"
    tabindex="-1"
    @scroll="syncEditorScrollFromPreview"
    @click="(e: MouseEvent) => { const a = (e.target as HTMLElement).closest('a[data-wiki-target]') as HTMLAnchorElement | null; if (a) { const target = a.getAttribute('data-wiki-target') || ''; const hashIdx = target.indexOf('#'); if (hashIdx !== -1) { e.preventDefault(); void navigateToBlockRef(target.slice(0, hashIdx), target.slice(hashIdx + 1)); } } }"
  >
    <!-- Item C: degraded preview state -->
    <div v-if="store.previewFailed" class="preview-degraded-banner" role="status" aria-live="polite">
      <div class="preview-degraded-icon" aria-hidden="true">&#9888;</div>
      <div class="preview-degraded-body">
        <strong class="preview-degraded-heading">Preview unavailable</strong>
        <span class="preview-degraded-kind">{{ ({ 'compile-failed': 'Compile failed', 'backend-unavailable': 'Backend not responding', 'transform-error': 'Transform engine missing' } as Record<string, string>)[store.lastCompileErrorKind] ?? 'Compile failed' }}</span>
        <span v-if="store.lastCompileErrorMessage" class="preview-degraded-msg">{{ store.lastCompileErrorMessage }}</span>
      </div>
      <div class="preview-degraded-actions">
        <button type="button" class="preview-degraded-retry" @click="store.compileActive()">Retry</button>
        <button
          v-if="store.consecutiveCompileFailures >= 3"
          type="button"
          class="preview-degraded-report"
          @click="copyPreviewErrorForSupport()"
        >Copy error for support</button>
      </div>
    </div>
    <div v-if="store.previewFailed && active?.compile?.html" class="preview-stale-ribbon" aria-label="Stale preview shown below">Stale</div>

    <section v-if="store.mode === 'export'" class="export-preview-summary" aria-label="Export preview summary">
      <div>
        <strong>{{ exportPreviewSummary.targetLabel }}</strong>
        <span>{{ exportPreviewSummary.readinessLabel }}</span>
      </div>
      <p>{{ exportPreviewSummary.manifestLabel }}</p>
      <p v-if="exportPreviewSummary.releaseLabel">{{ exportPreviewSummary.releaseLabel }}</p>
      <ul aria-label="Export preview options">
        <li v-for="option in exportPreviewSummary.options" :key="option">{{ option }}</li>
      </ul>
    </section>
    <section v-if="transformPreviewItems.length" class="transform-preview-summary" aria-label="Transform artifact preview">
      <h2>Transform Artifacts</h2>
      <article v-for="artifact in transformPreviewItems" :key="artifact.id">
        <strong>{{ artifact.name }}</strong>
        <p>{{ artifact.outputLabel }}</p>
        <small>{{ artifact.cacheLabel }}</small>
        <small v-if="artifact.locationLabel">{{ artifact.locationLabel }}</small>
        <button v-if="artifact.sourceLine" type="button" @click="goToTransformArtifact(artifact)">Go to source</button>
        <ul v-if="artifact.diagnostics.length" class="diagnostic-related">
          <li v-for="diagnostic in artifact.diagnostics" :key="diagnostic.message">{{ diagnostic.message }}</li>
        </ul>
      </article>
    </section>
    <section v-if="printPreviewEnabled" class="print-preview-summary" aria-label="Print preview summary">
      <header>
        <div>
          <strong>Print preview</strong>
          <span>{{ printPreviewReport.summary }}</span>
        </div>
        <button type="button" title="Leave print preview mode" @click="printPreviewEnabled = false">Exit</button>
      </header>
      <div class="print-preview-metrics" aria-label="Print preview metrics">
        <span><strong>{{ printPreviewReport.estimatedPages }}</strong> pages</span>
        <span><strong>{{ printPreviewReport.wordCount }}</strong> words</span>
        <span><strong>{{ printPreviewReport.pageBreaks }}</strong> page breaks</span>
        <span><strong>{{ printPreviewReport.sectionBreaks.length }}</strong> section breaks</span>
      </div>
      <ul v-if="printPreviewReport.warnings.length" aria-label="Print preview warnings">
        <li v-for="warning in printPreviewReport.warnings" :key="warning">{{ warning }}</li>
      </ul>
    </section>
    <article
      class="preview-document"
      :class="{ 'print-preview-document': printPreviewEnabled, 'preview-document-stale': store.previewFailed }"
      role="document"
      :aria-label="previewDocumentLabel"
      tabindex="0"
      :style="previewDocumentStyle"
      @click="handlePreviewClick"
      @keydown="handlePreviewKeydown"
      v-html="previewHtmlWithDiagnostics"
    ></article>
  </section>
</template>

<script setup lang="ts">
import { inject, onMounted, ref, type ComputedRef, type CSSProperties, type Ref } from 'vue'
import { useDocumentsStore } from '../stores/documents'

interface PreviewPaneCtx {
  previewHtmlWithDiagnostics: ComputedRef<string>
  previewDocumentLabel: ComputedRef<string>
  previewDocumentStyle: ComputedRef<CSSProperties>
  handlePreviewClick: (e: MouseEvent) => void
  handlePreviewKeydown: (e: KeyboardEvent) => void
  syncEditorScrollFromPreview: () => void
  navigateToBlockRef: (refPath: string, headingId: string) => Promise<void>
  copyPreviewErrorForSupport: () => void
  exportPreviewSummary: ComputedRef<any>
  transformPreviewItems: ComputedRef<any[]>
  goToTransformArtifact: (artifact: any) => void
  printPreviewEnabled: Ref<boolean>
  printPreviewReport: ComputedRef<any>
  active: ComputedRef<any>
}

const emit = defineEmits<{
  scroll: [payload: { ratio: number; scrollTop: number }]
}>()

const store = useDocumentsStore()
const ctx = inject<PreviewPaneCtx>('previewPaneCtx')!
const registerPaneEl = inject<(el: HTMLElement | null) => void>('registerPreviewPaneEl')

const {
  previewHtmlWithDiagnostics,
  previewDocumentLabel,
  previewDocumentStyle,
  handlePreviewClick,
  handlePreviewKeydown,
  navigateToBlockRef,
  copyPreviewErrorForSupport,
  exportPreviewSummary,
  transformPreviewItems,
  goToTransformArtifact,
  printPreviewEnabled,
  printPreviewReport,
  active,
} = ctx

const rootEl = ref<HTMLElement | null>(null)

function syncEditorScrollFromPreview(): void {
  ctx.syncEditorScrollFromPreview()
  if (rootEl.value) {
    const el = rootEl.value
    const scrollHeight = el.scrollHeight - el.clientHeight
    const ratio = scrollHeight > 0 ? el.scrollTop / scrollHeight : 0
    emit('scroll', { ratio, scrollTop: el.scrollTop })
  }
}

onMounted(() => {
  registerPaneEl?.(rootEl.value)
})

defineExpose({
  paneEl: () => rootEl.value,
})
</script>
