<template>
<h2>Settings</h2>
<section class="configuration-center" aria-label="NEditor configuration center">
  <nav class="configuration-center-nav" aria-label="Configuration sections">
    <button
      v-for="section in configurationCenterSections"
      :key="section.id"
      type="button"
      :class="{ active: selectedConfigurationSection === section.id }"
      @click="selectConfigurationSection(section.id)"
    >
      <strong>{{ section.label }}</strong>
      <small>{{ section.summary }}</small>
    </button>
  </nav>
  <p class="sidebar-hint">
    One configuration center controls setup, appearance, files, export, AI, voice, transforms, and release readiness.
  </p>
</section>
<section v-show="selectedConfigurationSection === 'overview'" class="configuration-center-panel" aria-label="Setup overview">
  <section class="configuration-setup-card" aria-label="NEditor configuration setup wizard">
  <header>
    <div>
      <strong>Setup wizard</strong>
      <span>{{ configurationSetupSummary }}</span>
    </div>
    <button type="button" @click="openConfigurationSetup()">Open setup</button>
  </header>
  <div class="configuration-status-grid">
    <button
      v-for="item in configurationSetupStatus.items"
      :key="item.id"
      type="button"
      :class="['configuration-status-chip', item.done ? 'ready' : 'needs-work']"
      @click="openConfigurationSetup(item.id)"
    >
      <strong>{{ item.label }}</strong>
      <small>{{ item.detail }}</small>
    </button>
  </div>
  <div style="margin-top:10px;padding-top:10px;border-top:1px solid #dce6f0">
    <p class="sidebar-hint">Reset the guided walkthrough so it replays on next open.</p>
    <button type="button" @click="store.resetGuidedDemoProgress(); void store.persistWorkspace()">Reset guided demo ({{ store.guidedDemoCompletedStepIds.length }} steps done)</button>
  </div>
</section>
</section>
<section v-show="selectedConfigurationSection === 'appearance'" class="configuration-center-panel" aria-label="Appearance and editor configuration">
<label>
  Theme
  <select v-model="store.theme">
    <option value="system">System</option>
    <option value="light">Light</option>
    <option value="paper">Paper ☕</option>
    <option value="dark">Dark</option>
  </select>
</label>
<label>
  Preview theme
  <div class="preview-theme-picker">
    <button type="button" @click="openPreviewThemeGallery()" :title="`Current theme: ${store.previewTheme}`">
      {{ store.previewTheme }} ▾
    </button>
  </div>
</label>
<label>
  Toolbar buttons
  <select v-model="store.toolbarDisplay">
    <option value="both">Icons and text</option>
    <option value="icons">Icons only</option>
    <option value="text">Text only</option>
  </select>
</label>
<label>
  Toolbar text size
  <input v-model.number="store.toolbarTextSize" type="range" min="9" max="15" step="1" />
  <output>{{ store.toolbarTextSize }}px</output>
</label>
<fieldset class="sidebar-layout-fieldset">
  <legend>Sidebar layout</legend>
  <label>
    <input type="radio" v-model="store.sidebarLayout" value="tabs" @change="void store.persistWorkspace()" />
    Tabs
    <small>Vertical tab strip inside the sidebar — one panel active at a time.</small>
  </label>
  <label>
    <input type="radio" v-model="store.sidebarLayout" value="activity-bar" @change="void store.persistWorkspace()" />
    Activity bar
    <small>Classic icon bar on the left edge selects the active panel.</small>
  </label>
</fieldset>
<label><input v-model="store.wordWrap" type="checkbox" /> Word wrap</label>
<label><input v-model="store.lineNumbers" type="checkbox" /> Line numbers</label>
<label><input v-model="store.codeFolding" type="checkbox" /> Code folding</label>
<label><input v-model="store.splitSourcePanes" type="checkbox" /> Split source editor panes</label>
<label>
  Editor keybindings
  <select v-model="store.editorKeymapMode">
    <option value="default">Default</option>
    <option value="emacs">Emacs-style navigation</option>
    <option value="vim">Vim-style modal navigation</option>
  </select>
</label>
<label><input v-model="store.highContrast" type="checkbox" /> High contrast</label>
<label><input v-model="store.reducedMotion" type="checkbox" /> Reduced motion</label>
<label><input v-model="store.keepInMenuBar" type="checkbox" @change="void store.persistWorkspace()" /> Keep in menu bar (hide window instead of quitting when closed — macOS only)</label>
<section class="accessibility-qa-panel" :data-status="accessibilityQaReport.status" aria-label="Screen-reader and accessibility QA">
  <header>
    <div>
      <h3>Accessibility QA</h3>
      <span>{{ accessibilityQaReport.summary }}</span>
    </div>
    <strong>{{ accessibilityQaReport.status }}</strong>
  </header>
  <div class="accessibility-qa-metrics" aria-label="Accessibility QA status counts">
    <span><strong>{{ accessibilityQaReport.counts.ready }}</strong> ready</span>
    <span><strong>{{ accessibilityQaReport.counts["needs-review"] }}</strong> review</span>
    <span><strong>{{ accessibilityQaReport.counts.blocked }}</strong> blocked</span>
  </div>
  <article
    v-for="item in accessibilityQaReport.items"
    :key="item.id"
    class="snapshot-row"
    :data-status="item.status"
  >
    <strong>{{ item.label }}</strong>
    <p>{{ item.detail }}</p>
    <small>{{ item.action }}</small>
  </article>
  <div class="reference-actions">
    <button type="button" title="Switch on high contrast for screen-reader and low-vision QA review" @click="store.highContrast = true">Use high contrast</button>
    <button type="button" title="Switch on reduced motion for users who prefer less animation" @click="store.reducedMotion = true">Reduce motion</button>
    <button type="button" title="Insert this accessibility QA report into the active Markdown document" @click="insertAccessibilityQaReport">Insert QA report</button>
  </div>
</section>
</section>
<section v-show="selectedConfigurationSection === 'files'" class="configuration-center-panel" aria-label="Files and history configuration">
<label><input v-model="store.autosave" type="checkbox" /> Autosave existing files</label>
<label>
  Autosave delay
  <input v-model.number="store.autosaveDelayMs" type="number" min="500" max="30000" step="250" />
</label>
<label><input v-model="store.autoSnapshot" type="checkbox" /> Automatic snapshots</label>
<label>
  Snapshot interval
  <input v-model.number="store.snapshotIntervalMs" type="number" min="30000" max="3600000" step="30000" />
</label>
<label>
  Snapshot storage
  <select v-model="store.snapshotStorage">
    <option value="app-data">App data</option>
    <option value="project-local">Project local</option>
  </select>
</label>
</section>
<section v-show="selectedConfigurationSection === 'exports' || selectedConfigurationSection === 'google-auth'" class="configuration-center-panel" aria-label="Export and brand configuration">
<h3>Export defaults</h3>
<label><input v-model="store.exportDefaults.includeManifest" type="checkbox" /> Manifest next to export</label>
<label><input v-model="store.exportDefaults.includeStyles" type="checkbox" /> Styles</label>
<label><input v-model="store.exportDefaults.includeSyntaxHighlighting" type="checkbox" /> Syntax highlighting</label>
<h3>HTML delivery</h3>
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
<h3>Document layout</h3>
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
<label><input v-model="store.exportDefaults.includeComments" type="checkbox" /> Comments</label>
<label><input v-model="store.exportDefaults.includeProvenance" type="checkbox" /> AI provenance</label>
<label><input v-model="store.exportDefaults.includeGlossary" type="checkbox" /> Glossary</label>
<label><input v-model="store.exportDefaults.includeAgenda" type="checkbox" /> PPTX agenda</label>
<section class="agent-provider-panel" aria-label="Google Docs authorization">
  <header>
    <div>
      <strong>Google Docs sign-in</strong>
      <span>Use a desktop OAuth client and keep access tokens session-only.</span>
    </div>
    <button type="button" @click="saveGoogleIntegrationSetup">Save Google setup</button>
  </header>
  <section class="agent-provider-grid">
    <label>
      OAuth client ID
      <input v-model="googleClientId" placeholder="Desktop app client ID from Google Cloud" />
    </label>
    <label>
      Account hint
      <input v-model="googleAccountHint" placeholder="name@example.com" />
    </label>
    <label class="wide-field">
      Google scopes
      <textarea v-model="googleScopesText" rows="3"></textarea>
    </label>
    <label class="wide-field">
      <input v-model="googleOfflineAccess" type="checkbox" />
      Request session refresh so Google Docs actions can renew expired access tokens without storing secrets.
    </label>
  </section>
  <div class="reference-actions">
    <button type="button" :disabled="googleAuthBusy || !googleClientId.trim()" @click="startGoogleSignIn">
      {{ googleAuthBusy ? "Waiting..." : "Sign in with Google" }}
    </button>
    <button type="button" :disabled="!googleAuthSession" @click="pollGoogleSignIn">Check callback</button>
    <button type="button" :disabled="!googleAccessToken || googleDocsImportBusy" @click="importCurrentDocumentToGoogleDocs">
      {{ googleDocsImportBusy ? "Importing..." : "Import current document" }}
    </button>
    <button type="button" :disabled="!googleAccessToken || !googleDocsLiveDocumentId || googleDocsImportBusy" @click="readBackGoogleDocsImport">
      Read back
    </button>
    <button type="button" :disabled="!googleRefreshToken || googleAuthBusy" @click="refreshGoogleAccessTokenNow">Refresh token</button>
    <button type="button" :disabled="!googleAccessToken" @click="copyGoogleAccessToken">Copy session token</button>
    <button type="button" :disabled="!googleAccessToken && !googleAuthSession" @click="clearGoogleSession">Clear session</button>
  </div>
  <p class="sidebar-hint">{{ googleDocsImportStatus || googleAuthStatus || googleAuthSummary }}</p>
  <div class="agent-cli-list" aria-label="Google authorization status">
    <span>
      Token storage
      <code>session-only</code>
    </span>
    <span>
      Scopes
      <code>{{ googleScopeList.length }}</code>
    </span>
    <span v-if="googleAuthSession">
      Redirect URI
      <code>{{ googleAuthSession.redirect_uri }}</code>
    </span>
    <span v-if="googleAuthPollStartedAt">
      Login started
      <code>{{ googleAuthPollStartedAt }}</code>
    </span>
    <span v-if="googleTokenScope">
      Granted scope
      <code>{{ googleTokenScope }}</code>
    </span>
    <span>
      Session refresh
      <code>{{ googleRefreshToken ? "available in memory" : googleOfflineAccess ? "requested" : "off" }}</code>
    </span>
    <span v-if="googleDocsLiveDocumentId">
      Google Doc ID
      <code>{{ googleDocsLiveDocumentId }}</code>
    </span>
    <span v-if="googleDocsLiveDocumentUrl">
      Google Doc URL
      <code>{{ googleDocsLiveDocumentUrl }}</code>
    </span>
  </div>
  <textarea
    v-if="googleDocsReadbackPreview"
    :value="googleDocsReadbackPreview"
    rows="5"
    readonly
    aria-label="Google Docs readback preview"
  ></textarea>
</section>
<h3>Bibliography defaults</h3>
<label>
  Citation style
  <select v-model="store.bibliographyDefaults.citationStyle">
    <option value="title">Title</option>
    <option value="author-year">Author-year</option>
    <option value="key">Key</option>
    <option value="numeric">Numeric</option>
    <option value="apa">APA</option>
    <option value="chicago-author-date">Chicago author-date</option>
    <option value="mla">MLA</option>
    <option value="harvard">Harvard</option>
    <option value="ieee">IEEE</option>
    <option value="vancouver">Vancouver</option>
    <option value="nature">Nature</option>
    <option value="ama">AMA</option>
  </select>
</label>
<h3>Brand profile defaults</h3>
<section class="brand-kit-manager" tabindex="-1" aria-label="Brand kit and page design presets">
  <header>
    <div>
      <strong>Brand kit and page design presets</strong>
      <span>Apply a coherent business-document identity, layout, cover, watermark, and export profile in one step.</span>
    </div>
    <button type="button" :disabled="!selectedBrandKitPreset" @click="applySelectedBrandKitPreset">Apply selected</button>
  </header>
  <section class="brand-kit-selector">
    <label>
      Preset
      <select v-model="selectedBrandKitPresetId">
        <option v-for="preset in brandKitPresets" :key="preset.id" :value="preset.id">
          {{ preset.label }}
        </option>
      </select>
    </label>
    <p v-if="selectedBrandKitPreset" class="sidebar-hint">{{ selectedBrandKitPreset.summary }}</p>
  </section>
  <section class="brand-kit-grid" aria-label="Available brand kit presets">
    <article v-for="preset in brandKitPresets" :key="preset.id" class="brand-kit-card">
      <header>
        <span class="brand-kit-swatch" :style="{ backgroundColor: preset.brand.color || store.brandProfileDefaults.color }"></span>
        <div>
          <strong>{{ preset.label }}</strong>
          <small>{{ preset.summary }}</small>
        </div>
      </header>
      <div class="template-meta">
        <span v-for="item in preset.bestFor" :key="`${preset.id}-${item}`">{{ item }}</span>
      </div>
      <ul>
        <li v-for="note in preset.designNotes" :key="`${preset.id}-${note}`">{{ note }}</li>
      </ul>
      <button type="button" @click="applyBrandKitPreset(preset)">Apply {{ preset.label }}</button>
    </article>
  </section>
  <section class="brand-kit-preview" aria-label="Current brand kit preview">
    <header :style="{ borderColor: store.brandProfileDefaults.color }">
      <div>
        <strong>{{ store.brandProfileDefaults.name || "Current brand" }}</strong>
        <span>{{ store.brandProfileDefaults.header || "No header template" }}</span>
      </div>
      <small>{{ store.exportDefaults.layoutPreset }} layout</small>
    </header>
    <dl>
      <div v-for="row in currentBrandKitPreviewRows" :key="row.label">
        <dt>{{ row.label }}</dt>
        <dd>{{ row.value }}</dd>
      </div>
    </dl>
  </section>
</section>
<label>
  Brand name
  <input v-model="store.brandProfileDefaults.name" />
</label>
<label>
  Brand color
  <input v-model="store.brandProfileDefaults.color" type="color" />
</label>
<label>
  Logo path
  <input v-model="store.brandProfileDefaults.logo" />
</label>
<label>
  Brand font
  <input v-model="store.brandProfileDefaults.font" />
</label>
<label>
  Header template
  <input v-model="store.brandProfileDefaults.header" />
</label>
<label>
  Footer template
  <input v-model="store.brandProfileDefaults.footer" />
</label>
<label>
  Watermark preset
  <input v-model="store.brandProfileDefaults.watermark" />
</label>
<label>
  Legal disclaimer
  <textarea v-model="store.brandProfileDefaults.legalDisclaimer" rows="3"></textarea>
</label>
</section>
<section v-show="selectedConfigurationSection === 'files'" class="configuration-center-panel" aria-label="Versioning configuration">
<h3>Git integration</h3>
<label><input v-model="store.gitIntegration.enabled" type="checkbox" /> Enable Git status</label>
<label><input v-model="store.gitIntegration.warnOnDirtyExport" type="checkbox" /> Warn on dirty export</label>
</section>
<section v-show="selectedConfigurationSection === 'ai'" class="configuration-center-panel" aria-label="AI agents and voice configuration">
<h3>AI paste cleanup defaults</h3>
<section class="agent-provider-panel" aria-label="LLM access defaults">
  <header>
    <div>
      <strong>LLM access defaults</strong>
      <span>Saved defaults do not store API keys; use environment variables or session-only keys.</span>
    </div>
    <button type="button" @click="saveAgentProviderDefaults">Save defaults</button>
  </header>
  <section class="agent-provider-grid">
    <label>
      Provider profile
      <select v-model="agentProviderId" @change="syncAgentProviderProfile">
        <option v-for="profile in aiProviderProfiles" :key="profile.id" :value="profile.id">
          {{ profile.label }}
        </option>
      </select>
    </label>
    <label>
      Model
      <select
        v-if="isOllamaProvider"
        v-model="agentProviderModel"
        aria-label="Ollama model"
        :disabled="ollamaModelBusy || (!ollamaModelOptions.length && !agentProviderModel.trim())"
        @change="confirmOllamaModelSelection"
      >
        <option v-if="!ollamaModelOptions.length && !agentProviderModel.trim()" value="">Refresh Ollama models to choose</option>
        <option v-if="currentModelMissingFromOllamaList || (!ollamaModelOptions.length && agentProviderModel.trim())" :value="agentProviderModel">
          {{ agentProviderModel }} (current)
        </option>
        <option v-for="model in ollamaModelOptions" :key="model.name" :value="model.name">
          {{ formatOllamaModelOption(model) }}
        </option>
      </select>
      <input v-else v-model="agentProviderModel" placeholder="Approved model or deployment name" />
      <small v-if="isOllamaProvider && ollamaSelectedModelMetadata" class="model-selection-note">{{ ollamaSelectedModelMetadata }}</small>
    </label>
    <div v-if="isOllamaProvider" class="ollama-model-picker" aria-label="Ollama model discovery">
      <button type="button" :disabled="ollamaModelBusy || !agentProviderEndpoint.trim()" @click="refreshOllamaModels">
        {{ ollamaModelBusy ? "Loading models..." : "Refresh from Ollama" }}
      </button>
      <small>{{ ollamaModelPickerHelp }}</small>
    </div>
    <label>
      Endpoint
      <input v-model="agentProviderEndpoint" placeholder="https://provider.example/v1/messages" />
    </label>
    <label>
      API key environment variable
      <input v-model="agentProviderKeyEnv" placeholder="OPENAI_API_KEY or NEDITOR_AI_API_KEY" />
    </label>
    <label v-if="isOllamaProvider && currentAgentProviderProfile.authHeader">
      Session API key for model discovery
      <input v-model="agentProviderApiKey" type="password" autocomplete="off" placeholder="Used once, never saved" />
    </label>
  </section>
  <div class="agent-cli-list" aria-label="Configured local agent options">
    <span v-for="profile in localAgentCliProfiles" :key="profile.id">
      {{ profile.label }}
      <code>{{ profile.command }}</code>
    </span>
  </div>
  <label class="ai-timeout-row">
    Request timeout (seconds)
    <input
      type="number"
      min="10"
      max="600"
      step="5"
      :value="store.aiProviderDefaults.aiTimeoutSeconds"
      @change="store.saveAiProviderDefaults({ ...store.aiProviderDefaults, aiTimeoutSeconds: Math.min(600, Math.max(10, Number(($event.target as HTMLInputElement).value) || 60)) })"
      aria-label="AI provider request timeout in seconds (10–600)"
    />
    <small>10–600 s · default 60. Applies to all AI provider HTTP requests.</small>
  </label>
</section>
<!-- Ollama status + model catalog -->
<template v-if="isOllamaProfile">
  <h3>Ollama status</h3>
  <div class="ollama-health-row">
    <div class="ollama-health-badge" :class="ollamaHealth ? (ollamaHealth.running ? 'health-ok' : 'health-err') : 'health-unknown'">
      <span class="health-dot"></span>
      <span v-if="!ollamaHealth">not checked</span>
      <span v-else-if="ollamaHealth.running">running · {{ ollamaHealth.modelCount }} model{{ ollamaHealth.modelCount !== 1 ? 's' : '' }} installed{{ ollamaHealth.version ? ' · v' + ollamaHealth.version : '' }}</span>
      <span v-else>not running — {{ ollamaHealth.error || 'check endpoint' }}</span>
    </div>
    <button type="button" :disabled="ollamaHealthBusy" @click="probeOllamaHealth">{{ ollamaHealthBusy ? 'Checking…' : 'Check' }}</button>
  </div>

  <div v-if="contextBudgetInfo && activeModelCard" class="context-budget">
    <div class="context-budget-label">
      <span>Context budget — {{ activeModelCard.label }}</span>
      <span>{{ contextBudgetInfo.totalUsed.toLocaleString() }} / {{ activeModelCard.numCtx.toLocaleString() }} est. tokens ({{ contextBudgetInfo.utilizationPct }}% used)</span>
    </div>
    <div class="context-budget-bar">
      <div class="context-budget-fill"
        :class="contextBudgetInfo.utilizationPct > 85 ? 'budget-danger' : contextBudgetInfo.utilizationPct > 65 ? 'budget-warn' : 'budget-ok'"
        :style="{ width: contextBudgetInfo.utilizationPct + '%' }"
      ></div>
    </div>
    <p v-if="contextBudgetInfo.overBudget" class="sidebar-hint" style="color:#b91c1c">Document exceeds context window. NEditor will trim context automatically.</p>
  </div>

  <template v-if="ollamaInstalledModels.length">
    <h3>Installed models</h3>
    <div class="ollama-installed-list">
      <div v-for="m in ollamaInstalledModels" :key="m" class="ollama-installed-row">
        <span class="ollama-installed-name">{{ m }}</span>
        <div class="ollama-installed-actions">
          <button type="button" :class="{ primary: store.aiProviderDefaults.model === m }" @click="useOllamaModel(m)">
            {{ store.aiProviderDefaults.model === m ? '✓ Active' : 'Use' }}
          </button>
          <button type="button" :disabled="ollamaDeleteBusy" @click="deleteOllamaModel(m)">Delete</button>
        </div>
      </div>
    </div>
  </template>

  <h3>Recommended models (≤9B, local)</h3>
  <p class="sidebar-hint">Pull downloads the model to your machine — no API key needed. All models below are ≤9B parameters.</p>
  <div v-if="ollamaPullError" class="sidebar-hint" style="color:#b91c1c;margin-top:4px">{{ ollamaPullError }}</div>
  <div v-if="ollamaPullSuccess" class="sidebar-hint" style="color:#1a6b48;margin-top:4px">✓ {{ ollamaPullSuccess }}</div>
  <div class="ollama-model-grid">
    <div
      v-for="card in OLLAMA_MODEL_CATALOG.filter(m => m.recommended)"
      :key="card.id"
      class="ollama-model-card"
      :class="{
        'model-active': store.aiProviderDefaults.model === card.id,
        'model-installed': ollamaInstalledModels.some(n => n.startsWith(card.id.split(':')[0]))
      }"
    >
      <div class="model-card-header">
        <span class="model-badge-family">{{ card.family }}</span>
        <span class="model-badge-params">{{ card.params }}</span>
        <span class="model-badge-rec">{{ card.badge }}</span>
      </div>
      <div class="model-card-name">{{ card.label }}</div>
      <div class="model-card-meta">
        <span title="Context window">{{ (card.contextTokens / 1000).toFixed(0) }}k ctx</span>
        <span title="VRAM">~{{ card.vramGb }}GB</span>
        <span title="Disk">{{ card.diskGb }}GB dl</span>
        <span class="model-speed" :class="'speed-' + card.speed">{{ card.speed }}</span>
      </div>
      <div class="model-card-why">{{ card.whyRecommended }}</div>
      <div class="model-card-tags">
        <span v-for="tag in card.tags.slice(0, 3)" :key="tag" class="model-tag">{{ tag }}</span>
      </div>
      <div class="model-card-actions">
        <button
          v-if="!ollamaInstalledModels.some(n => n.startsWith(card.id.split(':')[0]))"
          type="button"
          class="model-pull-btn"
          :disabled="ollamaPullBusy"
          @click="pullOllamaModel(card.id)"
        >{{ ollamaPullBusy && ollamaPullModelId === card.id ? 'Pulling…' : 'Pull ↓' }}</button>
        <button
          v-else
          type="button"
          :class="{ primary: store.aiProviderDefaults.model === card.id }"
          @click="useOllamaModel(card.id)"
        >{{ store.aiProviderDefaults.model === card.id ? '✓ Active' : 'Use' }}</button>
      </div>
    </div>
  </div>
</template>

<section class="agent-provider-panel" aria-label="Text to speech setup">
  <header>
    <div>
      <strong>Read aloud</strong>
      <span>Read selected text or the full Markdown document with browser speech, macOS Say, or Supertonic.</span>
    </div>
    <button type="button" @click="store.saveTtsPreferences(store.ttsPreferences)">Save TTS</button>
  </header>
  <section class="agent-provider-grid">
    <label>
      TTS engine
      <select v-model="store.ttsPreferences.engine">
        <option v-for="option in ttsEngineOptions" :key="option.id" :value="option.id">{{ option.label }}</option>
      </select>
    </label>
    <label>
      Voice
      <input v-model="store.ttsPreferences.voice" placeholder="Browser voice, macOS voice, or Supertonic voice" />
    </label>
    <label>
      Language
      <input v-model="store.ttsPreferences.language" placeholder="en-US" />
    </label>
    <label>
      Rate
      <input v-model.number="store.ttsPreferences.rate" type="number" min="0.5" max="2" step="0.1" />
    </label>
    <label>
      Supertonic command
      <input v-model="store.ttsPreferences.supertonicCommand" placeholder="supertonic or /path/to/supertonic" />
    </label>
    <label>
      Supertonic voice
      <input v-model="store.ttsPreferences.supertonicVoice" placeholder="F1, M1, or approved voice" />
    </label>
    <label>
      Model storage path
      <input
        v-model="store.ttsPreferences.supertonicModelStoragePath"
        placeholder="~/.cache/supertonic/models"
      />
    </label>
  </section>
  <section v-if="ttsModelDownloadPlan" class="tts-model-download-notice" aria-label="TTS model download notice">
    <header>
      <div>
        <strong>Model download required before Supertonic speech</strong>
        <span>NEditor will not start a model-backed Supertonic run until you acknowledge this download.</span>
      </div>
    </header>
    <dl>
      <div>
        <dt>Model</dt>
        <dd>{{ ttsModelDownloadPlan.model }}</dd>
      </div>
      <div>
        <dt>Size</dt>
        <dd>{{ ttsModelDownloadPlan.approximateSize }}</dd>
      </div>
      <div>
        <dt>Storage location</dt>
        <dd>{{ ttsModelDownloadPlan.storagePath }}</dd>
      </div>
      <div>
        <dt>Download source</dt>
        <dd>{{ ttsModelDownloadPlan.source }}</dd>
      </div>
    </dl>
    <label class="tts-model-consent">
      <input v-model="store.ttsPreferences.supertonicModelDownloadAcknowledged" type="checkbox" @change="saveTtsModelDownloadAcknowledgement" />
      I understand Supertonic may download {{ ttsModelDownloadPlan.model }} ({{ ttsModelDownloadPlan.approximateSize }}) to {{ ttsModelDownloadPlan.storagePath }} before first use.
    </label>
    <div class="reference-actions">
      <button type="button" :disabled="!ttsModelDownloadPlan.acknowledged || ttsModelDownloadBusy" @click="downloadSelectedTtsModel">
        {{ ttsModelDownloadBusy ? "Starting..." : "Download model" }}
      </button>
      <button type="button" @click="copyTtsModelDownloadCommand">Copy command</button>
    </div>
    <p class="sidebar-hint">Download command: <code>{{ ttsModelDownloadPlan.command }}</code></p>
  </section>
  <div class="reference-actions">
    <button type="button" :disabled="ttsInspectionBusy" @click="checkTtsRuntime">
      {{ ttsInspectionBusy ? "Checking..." : "Check TTS" }}
    </button>
    <button type="button" :disabled="ttsReadDisabled" @click="readSelectionAloud">Read selection</button>
    <button type="button" :disabled="ttsReadDisabled" @click="readDocumentAloud">Read document</button>
    <button type="button" @click="stopReadingAloud">Stop</button>
  </div>
  <p class="sidebar-hint">{{ ttsStatus || ttsRuntimeSummary || ttsSetupSummary }}</p>
  <div v-if="ttsInspectionReport" class="agent-cli-list" aria-label="Text to speech runtime status">
    <span v-for="engine in ttsInspectionReport.engines" :key="engine.id">
      {{ engine.label }}
      <code>{{ engine.available ? "available" : "needs setup" }}</code>
    </span>
  </div>
</section>
<label><input v-model="store.aiCleanupDefaults.markAsDraft" type="checkbox" /> Mark as draft</label>
<label><input v-model="store.aiCleanupDefaults.addProvenance" type="checkbox" /> Add provenance block</label>
<label><input v-model="store.aiCleanupDefaults.preserveHeadings" type="checkbox" /> Preserve original headings</label>
<label><input v-model="store.aiCleanupDefaults.convertNumberedLists" type="checkbox" /> Convert numbered lists</label>
<label><input v-model="store.aiCleanupDefaults.convertTables" type="checkbox" /> Convert tables</label>
<label><input v-model="store.aiCleanupDefaults.insertCitationTodos" type="checkbox" /> Insert citation TODOs</label>
<h3>Document memory</h3>
<p class="sidebar-hint">Shared context injected into every AI session. Keep to 3-5 lines of audience, tone, and standing instructions.</p>
<label><textarea v-model="store.documentMemoryText" rows="4" placeholder="Audience: senior finance executives.&#10;Tone: formal, direct." @blur="void store.persistWorkspace()"></textarea></label>
</section>
<section v-show="selectedConfigurationSection === 'appearance'" class="configuration-center-panel" aria-label="Typography configuration">
<h3>Typography</h3>
<label>
  Editor font
  <input v-model="store.editorFont" />
</label>
<label>
  Editor font size
  <input v-model.number="store.editorFontSize" type="number" min="12" max="22" step="1" />
</label>
<label>
  Editor line height
  <input v-model.number="store.editorLineHeight" type="number" min="1" max="2.4" step="0.05" />
</label>
<label>
  Preview font
  <input v-model="store.previewFont" />
</label>
<label>
  Preview font size
  <input v-model.number="store.previewFontSize" type="number" min="12" max="22" step="1" />
</label>
<label>
  Preview line height
  <input v-model.number="store.previewLineHeight" type="number" min="1" max="2.4" step="0.05" />
</label>
</section>
<section v-show="selectedConfigurationSection === 'files'" class="configuration-center-panel" aria-label="Recent documents configuration">
<section aria-label="Command line and default reader setup">
  <h3>Command line and default reader</h3>
  <p class="sidebar-hint">Use <code>ned file.md</code> to open Markdown, <code>ned open file.md --dry-run --json</code> to verify file handoff, <code>ned init . --json</code> to create a reusable <code>.neditor</code> project scaffold, <code>ned profile --workspace . --set companyName=Acme --json</code> to set reusable business identity, <code>ned profile --fields --json</code> to list identity fields and aliases, <code>ned profile --workspace . --get companyName</code> to print one value for scripts, <code>ned rfp-response rfp.pdf --output response.md --matrix-output matrix.md --checklist-output checklist.md --json</code> to analyze an RFP and write a response, compliance matrix, and front-of-proposal checklist, <code>ned analyze-rfp - --matrix</code> to turn piped RFP text into a matrix, <code>ned analyze-rfp - --checklist</code> to turn piped RFP text into a checklist, <code>ned templates --category Procurement --json</code> to discover filtered starters, <code>ned templates --markdown report --workspace . --fill-profile</code> to preview profile-aware starter Markdown, <code>ned transform-templates --category Business --transform calc --query ROI --json</code> to discover reusable calc/chart/diagram/data blocks, <code>ned outlines --category Procurement --query RFP --json</code> to discover reusable document outlines, <code>ned outlines --markdown business-report</code> to print an outline for the planner or Docs Live, <code>ned outlines --workspace . --save board-pack --docs-live-type board-memo --section "Decision Requested" --section "Recommendation"</code> to add a reusable workspace outline, <code>ned latex-templates --query proposal --json</code> to discover reusable LaTeX profiles, <code>ned latex-templates --preamble rfp-response</code> to review a profile preamble, <code>ned snippets --workspace . --markdown business-contact-block --fill-profile</code> to print profile-aware reusable document parts, <code>ned new tender.md --template tender --workspace . --fill-profile --json</code> or <code>ned new podcast.md --template podcast-script --workspace . --fill-profile --json</code> to start from business and publishing scaffolds, <code>ned inspect file.md --json</code> for no-write document inventory, <code>ned validate file.md --to pdf --json</code> for no-write readiness checks, <code>ned convert file.md --to pdf,docx,html --output-dir exports</code> for headless delivery packs, <code>ned convert - --to html --stdout</code> for pipe automation, <code>ned targets</code> or <code>ned handlers --commands-only</code> for setup discovery, <code>ned readiness --json</code> for release gap summaries, <code>ned readiness --action-plan</code> for assignable release evidence work items, <code>ned evidence --json</code> for release evidence report status, <code>ned default-reader --status --json</code> for default Markdown reader setup, <code>ned support-bundle --output support.json</code> for help desk handoffs, <code>ned completions zsh</code> for shell setup, and <code>ned doctor --workspace . --json</code> for setup checks.</p>
  <div class="support-bundle-actions">
    <button type="button" :disabled="cliDeployBusy || cliDeployPlan?.supported === false" title="Install the packaged ned helper into a user-level command directory" @click="deployCliGlobally">
      {{ cliDeployBusy ? "Deploying..." : "Deploy CLI" }}
    </button>
    <button type="button" :disabled="cliDeployBusy" title="Refresh the CLI deployment plan without changing files" @click="loadCliDeployPlan">Refresh</button>
    <span>{{ cliDeployStatus || cliDeployPlan?.message || "Deploy CLI makes the packaged ned command available from new terminal windows." }}</span>
  </div>
  <p v-if="cliDeployPlan" class="engine-setup-status" :class="cliDeployPlan.applied && cliDeployPlan.pathReady ? 'ok' : cliDeployPlan.supported ? '' : 'failed'" role="status">
    Target: <code>{{ cliDeployPlan.deployedPath }}</code>
  </p>
  <pre v-if="cliDeployPlan?.commands?.length" class="transform-installer-commands">{{ cliDeployPlan.commands.join("\n") }}</pre>
  <ul v-if="cliDeployPlan?.manualSteps?.length" class="transform-installer-handlers">
    <li v-for="step in cliDeployPlan.manualSteps" :key="step">{{ step }}</li>
  </ul>
  <label>
    <input :checked="defaultMarkdownReaderEnabled" type="checkbox" :disabled="defaultMarkdownReaderBusy" @change="toggleDefaultMarkdownReader($event)" />
    Make NEditor the default Markdown reader
  </label>
  <p class="engine-setup-status" :class="defaultMarkdownReaderPlan?.applied ? 'ok' : defaultMarkdownReaderPlan?.supported ? '' : 'failed'" role="status">
    {{ defaultMarkdownReaderStatus || defaultMarkdownReaderPlan?.message || "Check default Markdown reader setup before changing OS file associations." }}
  </p>
  <pre v-if="defaultMarkdownReaderPlan?.commands?.length" class="transform-installer-commands">{{ defaultMarkdownReaderPlan.commands.join("\n") }}</pre>
  <ul v-if="defaultMarkdownReaderPlan?.manual_steps?.length" class="transform-installer-handlers">
    <li v-for="step in defaultMarkdownReaderPlan.manual_steps" :key="step">{{ step }}</li>
  </ul>
</section>
<section class="transform-handler-installer" aria-label="Support bundle">
  <header>
    <div>
      <h4>Support bundle</h4>
      <span>Create a redaction-safe setup and release-readiness handoff for help desks, release managers, or internal IT.</span>
    </div>
    <button type="button" :disabled="supportBundleBusy" title="Preview support diagnostics without writing a file" @click="previewSupportBundle">Preview</button>
  </header>
  <div class="support-bundle-actions">
    <button type="button" :disabled="supportBundleBusy" title="Choose where to write the support bundle JSON" @click="saveSupportBundle">Save JSON</button>
    <button type="button" :disabled="!supportBundleReport" title="Insert a redaction-safe setup, release, and spec-closure handoff into the active document" @click="insertSupportBundleHandoff">Insert handoff</button>
    <button type="button" :disabled="!supportBundleReport" title="Insert exact return paths, ingest candidates, validators, and redaction rules for release evidence owners" @click="insertEvidenceReturnPacket">Insert evidence return packet</button>
    <button type="button" :disabled="!supportBundleManualReviewWorkOrders.length" title="Insert reviewer-ready manual sign-off templates for spec-closure work orders" @click="insertManualReviewSignoffKit">Insert manual review kit</button>
    <span>{{ supportBundleStatus || "The bundle contains setup status and evidence summaries, not document content or secrets." }}</span>
  </div>
  <dl v-if="supportBundleReport" class="transform-installer-summary">
    <div>
      <dt>Doctor</dt>
      <dd>{{ supportBundleReport.doctor?.status || "unknown" }}</dd>
    </div>
    <div>
      <dt>Release</dt>
      <dd>{{ supportBundleReport.releaseReadiness?.status || "unknown" }}</dd>
    </div>
    <div>
      <dt>Gaps</dt>
      <dd>{{ supportBundleReport.releaseReadiness?.evidenceGaps?.length || 0 }}</dd>
    </div>
    <div>
      <dt>Spec rows</dt>
      <dd>
        {{ supportBundleReport.specCompletion?.summary?.openRows || 0 }} open,
        {{ supportBundleReport.specActionPlan?.readyToSendCount || 0 }}/{{ supportBundleReport.specActionPlan?.workOrders?.length || 0 }} work orders ready
      </dd>
    </div>
    <div>
      <dt>Engines</dt>
      <dd>
        {{ supportBundleReport.engineProbe?.summary?.installed || 0 }} installed,
        {{ supportBundleReport.engineProbe?.summary?.missingLocal || 0 }} missing
      </dd>
    </div>
    <div>
      <dt>Evidence reports</dt>
      <dd>
        {{ supportBundleReport.evidenceReportSummary?.ready || 0 }} ready,
        {{ supportBundleReport.evidenceReportSummary?.attention || 0 }} attention,
        {{ supportBundleReport.evidenceReportSummary?.missing || 0 }} missing
      </dd>
    </div>
    <div>
      <dt>Action plan</dt>
      <dd>
        {{ supportBundleReport.releaseActionPlan?.status || "unknown" }}
        ({{ supportBundleReport.releaseActionPlan?.readyToSendCount || 0 }}/{{ supportBundleReport.releaseActionPlan?.workItems?.length || 0 }} ready)
      </dd>
    </div>
    <div>
      <dt>Candidate</dt>
      <dd>
        {{ supportBundleReport.releaseCandidate?.status || "unknown" }},
        {{ supportBundleReport.releaseCandidate?.releaseable ? "releaseable" : "not releaseable" }},
        {{ supportBundleReport.releaseCandidate?.summary?.artifacts || 0 }} artifacts
      </dd>
    </div>
    <div>
      <dt>100 improvements</dt>
      <dd>
        {{ supportBundleReport.improvementAudit?.summary?.implementedEvidencePresent || 0 }}/{{ supportBundleReport.improvementAudit?.total || 0 }} evidenced,
        {{ supportBundleReport.improvementAudit?.summary?.open || 0 }} open
      </dd>
    </div>
    <div>
      <dt>Output</dt>
      <dd>{{ supportBundleReport.writtenTo || "preview only" }}</dd>
    </div>
  </dl>
  <section v-if="supportBundleRecommendationGroups.length" class="support-bundle-recommendations" aria-label="Support bundle recommendations">
    <article v-for="group in supportBundleRecommendationGroups" :key="group.id" :data-priority="group.priority">
      <header>
        <span>{{ group.label }}</span>
        <small>{{ group.items.length }} item{{ group.items.length === 1 ? "" : "s" }}</small>
      </header>
      <ul>
        <li v-for="recommendation in group.items" :key="recommendation">{{ recommendation }}</li>
      </ul>
    </article>
  </section>
  <section v-if="supportBundleReport?.releaseActionPlan?.workItems?.length" class="support-bundle-action-plan" aria-label="Release evidence action plan">
    <h5>Release evidence action plan</h5>
    <article v-for="item in supportBundleReport.releaseActionPlan.workItems.slice(0, 6)" :key="item.id">
      <strong>{{ item.id }}</strong>
      <span>{{ item.detail }}</span>
      <small>
        Runbook:
        {{ (item.runbooks as any[])?.map((runbook) => runbook.path || runbook.title).filter(Boolean).join(", ") || "not mapped" }}
      </small>
    </article>
    <p v-if="supportBundleReport.releaseActionPlan.workItems.length > 6" class="sidebar-hint">
      {{ supportBundleReport.releaseActionPlan.workItems.length - 6 }} more action item(s) are included in the saved JSON.
    </p>
  </section>
  <section v-if="supportBundleReport?.specActionPlan?.workOrders?.length" class="support-bundle-action-plan" aria-label="Specification work orders">
    <h5>Specification work orders</h5>
    <article v-for="order in supportBundleReport.specActionPlan.workOrders.slice(0, 6)" :key="order.id">
      <strong>{{ order.id }}</strong>
      <span>{{ order.specSection }} / {{ order.requirementArea }}</span>
      <small>
        {{ order.classification || "evidence" }}:
        {{ order.runbooks?.join(", ") || "runbook not mapped" }}
      </small>
    </article>
    <p v-if="supportBundleReport.specActionPlan.workOrders.length > 6" class="sidebar-hint">
      {{ supportBundleReport.specActionPlan.workOrders.length - 6 }} more work order(s) are included in the saved JSON.
    </p>
  </section>
  <section v-if="supportBundleReport?.releaseCandidate" class="support-bundle-action-plan" aria-label="Release candidate status">
    <h5>Release candidate</h5>
    <article>
      <strong>{{ supportBundleReport.releaseCandidate.status || "unknown" }}</strong>
      <span>
        {{ supportBundleReport.releaseCandidate.releaseable ? "Final-releaseable on this host" : "Not final-releaseable on this host" }}
      </span>
      <small>
        {{ supportBundleReport.releaseCandidate.candidateDir || ".tmp/release-candidate" }} |
        {{ supportBundleReport.releaseCandidate.summary?.checkStatus || "missing" }} checker,
        {{ supportBundleReport.releaseCandidate.summary?.evidenceGaps || 0 }} evidence gap(s)
      </small>
    </article>
  </section>
  <section v-if="supportBundleReport?.improvementAudit" class="support-bundle-action-plan" aria-label="100 improvements coverage">
    <h5>100 improvements coverage</h5>
    <article>
      <strong>{{ supportBundleReport.improvementAudit.implementationReady ? "implementation-ready" : "open roadmap work" }}</strong>
      <span>
        {{ supportBundleReport.improvementAudit.summary?.implementedEvidencePresent || 0 }} evidenced,
        {{ supportBundleReport.improvementAudit.summary?.partialOrExternal || 0 }} partial/external,
        {{ supportBundleReport.improvementAudit.summary?.needsImplementationEvidence || 0 }} need implementation evidence,
        production {{ supportBundleReport.improvementAudit.productionReady ? "ready" : "blocked" }}
      </span>
      <small>{{ supportBundleReport.improvementAudit.releaseReadiness?.status || "release readiness not checked" }} | {{ supportBundleReport.improvementAudit.releaseReadiness?.evidenceGaps || 0 }} release evidence gap(s) | <code>ned improvements --json</code> | <code>ned improvements --output improvement-coverage.md</code></small>
    </article>
    <article v-for="item in ((supportBundleReport.improvementAudit.items as any[]) || []).filter((entry) => entry.status !== 'implemented-evidence-present').slice(0, 5)" :key="item.number">
      <strong>#{{ item.number }} {{ item.title }}</strong>
      <span>{{ item.status }} | {{ item.lane }}</span>
      <small>{{ item.nextAction }}</small>
    </article>
  </section>
</section>
<section aria-label="Recent files">
  <h3>Recent files</h3>
  <button v-for="path in store.recentFiles" :key="path" class="outline-row" type="button" @click="store.openRecentPath(path)">
    {{ path }}
  </button>
</section>
<section aria-label="Recent folders">
  <h3>Recent folders</h3>
  <button v-for="path in store.recentFolders" :key="path" class="outline-row" type="button" @click="store.openRecentFolder(path)">
    {{ path }}
  </button>
</section>
<section aria-label="Recently closed documents">
  <h3>Recently closed</h3>
  <button v-for="path in store.recentlyClosed" :key="path" class="outline-row" type="button" @click="store.openRecentPath(path)">
    {{ path }}
  </button>
</section>
</section>
<section v-show="selectedConfigurationSection === 'release'" class="configuration-center-panel" aria-label="Release evidence configuration">
  <section class="release-evidence-dashboard" :data-status="releaseEvidenceDashboard.status" aria-label="Configurator release evidence dashboard">
    <header>
      <h3>Release evidence setup</h3>
      <span>{{ releaseEvidenceDashboard.summary }}</span>
    </header>
    <p>Use this setup area to keep release gates, credentialed workflows, cross-platform package proof, Homebrew evidence, signing, notarization, accessibility, and freshness visible before distribution.</p>
    <div class="release-evidence-metrics" aria-label="Configurator release evidence lane counts">
      <span><strong>{{ releaseEvidenceDashboard.counts.complete }}</strong> complete</span>
      <span><strong>{{ releaseEvidenceDashboard.counts.blocked }}</strong> blocked</span>
      <span><strong>{{ releaseEvidenceDashboard.counts.manual }}</strong> manual</span>
      <span><strong>{{ releaseEvidenceDashboard.counts.credentialed }}</strong> credentialed</span>
      <span><strong>{{ releaseEvidenceDashboard.counts["cross-platform"] }}</strong> cross-platform</span>
      <span><strong>{{ releaseEvidenceDashboard.counts.stale }}</strong> stale</span>
      <span><strong>{{ releaseEvidenceDashboard.counts["ready-to-send"] }}</strong> ready</span>
    </div>
    <div class="release-readiness-actions">
      <button type="button" @click="openConfigurationSetup('release')">Open release setup wizard</button>
      <button type="button" @click="insertReleaseEvidenceDashboard">Insert evidence dashboard</button>
      <button type="button" @click="insertReleaseReadinessAudit">Insert release audit</button>
      <button type="button" @click="insertProductionReadinessWorkOrders">Insert work orders</button>
    </div>
    <section class="production-readiness-work-orders" aria-label="Configurator production readiness work orders">
      <header>
        <h4>Open production work orders</h4>
        <span>{{ productionReadinessWorkOrders.length }} open</span>
      </header>
      <article v-for="workOrder in productionReadinessWorkOrders.slice(0, 4)" :key="workOrder.id" class="snapshot-row" :data-status="workOrder.priority">
        <strong>{{ workOrder.title }}</strong>
        <p>{{ workOrder.owner }} | {{ workOrder.command }}</p>
        <small>{{ workOrder.acceptanceEvidence }}</small>
      </article>
    </section>
    <article
      v-for="item in releaseEvidenceDashboard.items"
      :key="item.id"
      class="snapshot-row"
      :data-status="item.lane"
    >
      <strong>{{ item.label }}</strong>
      <p>{{ item.detail }}</p>
      <small>{{ item.action }}</small>
    </article>
  </section>
</section>
<section v-show="selectedConfigurationSection === 'imports'" class="configuration-center-panel" aria-label="Imports and data sources">
  <h3>Document import (pandoc)</h3>
  <p class="sidebar-hint">pandoc converts Word (.docx), PowerPoint (.pptx), OpenDocument, RTF, and HTML to Markdown. Install from <strong>pandoc.org</strong>.</p>
  <div class="config-probe-row">
    <div class="config-probe-status" :class="pandocAvailable ? 'probe-ok' : 'probe-missing'">
      <strong>pandoc</strong><span>{{ pandocProbeResult || "not checked" }}</span>
    </div>
    <div class="config-probe-status" :class="curlAvailable ? 'probe-ok' : 'probe-missing'">
      <strong>curl</strong><span>{{ curlProbeResult || "not checked" }}</span>
    </div>
    <button type="button" @click="probeImportTools">Check tools</button>
  </div>
  <label>pandoc path (blank = use PATH)<input v-model="store.pandocBinaryPath" type="text" placeholder="/usr/local/bin/pandoc" @blur="void store.persistWorkspace()" /></label>
  <label>curl path (blank = use PATH)<input v-model="store.curlBinaryPath" type="text" placeholder="/usr/bin/curl" @blur="void store.persistWorkspace()" /></label>
  <h3>REST data source security</h3>
  <p class="sidebar-hint">Allowed hosts for REST data fetches (one per line). Leave empty to allow any host.</p>
  <label>
    Allowed hosts
    <textarea :value="store.restFetchAllowedHosts.join('\n')" rows="4" placeholder="api.example.com" @blur="store.restFetchAllowedHosts = ($event.target as HTMLTextAreaElement).value.split('\n').map((s: string) => s.trim()).filter(Boolean); void store.persistWorkspace()"></textarea>
  </label>
  <label>REST fetch timeout (ms)<input v-model.number="store.restFetchTimeoutMs" type="number" min="1000" max="60000" step="1000" @blur="void store.persistWorkspace()" /></label>
  <h3>Mail merge</h3>
  <label class="compact-check"><input v-model="store.mailMergeRequireWorkspaceRoot" type="checkbox" @change="void store.persistWorkspace()" /><span>Require workspace root (restrict paths to open workspace)</span></label>
  <label>Max records per merge<input v-model.number="store.mailMergeMaxRecords" type="number" min="1" max="100000" step="100" @blur="void store.persistWorkspace()" /></label>
  <label>Default CSV delimiter<select v-model="store.mailMergeDefaultDelimiter" @change="void store.persistWorkspace()"><option value=",">Comma (,)</option><option value="&#9;">Tab</option></select></label>
</section>
<section v-show="selectedConfigurationSection === 'automation'" class="configuration-center-panel" aria-label="Automation and webhooks">
  <h3>Webhooks</h3>
  <p class="sidebar-hint">NEditor POSTs a JSON event payload to each enabled URL when the selected events occur. Requires curl on PATH.</p>
  <div v-if="!store.webhookConfigs.length" class="sidebar-hint">No webhooks configured.</div>
  <div v-for="webhook in store.webhookConfigs" :key="webhook.id" class="webhook-row">
    <div class="webhook-info">
      <strong>{{ webhook.name }}</strong>
      <span class="webhook-url">{{ webhook.url }}</span>
      <span class="webhook-events">{{ webhook.events.join(", ") }}</span>
    </div>
    <div class="webhook-actions">
      <label class="compact-check" style="margin:0"><input type="checkbox" :checked="webhook.enabled" @change="toggleWebhook(webhook.id)" /><span>{{ webhook.enabled ? "on" : "off" }}</span></label>
      <button type="button" @click="removeWebhook(webhook.id)">Remove</button>
    </div>
  </div>
  <section class="webhook-add-form" aria-label="Add webhook">
    <h4>Add webhook</h4>
    <label>Name<input v-model="newWebhookDraft.name" type="text" placeholder="Slack notify" /></label>
    <label>URL<input v-model="newWebhookDraft.url" type="url" placeholder="https://hooks.example.com/neditor" /></label>
    <div class="webhook-events-select">
      <strong>Fire on events</strong>
      <label v-for="ev in WEBHOOK_EVENTS" :key="ev.id" class="compact-check">
        <input type="checkbox" :value="ev.id" v-model="newWebhookDraft.events" /><span>{{ ev.label }}</span>
      </label>
    </div>
    <button type="button" :disabled="!newWebhookDraft.name.trim() || !newWebhookDraft.url.trim()" @click="addWebhook">Add webhook</button>
  </section>
</section>
<section v-show="selectedConfigurationSection === 'audit'" class="configuration-center-panel" aria-label="Audit and compliance">
  <h3>Document audit log</h3>
  <p class="sidebar-hint">Appends a tamper-evident JSONL entry to <code>.neditor/audit.jsonl</code> on save, export, status change, and approval.</p>
  <label class="compact-check"><input v-model="store.auditEnabled" type="checkbox" @change="void store.persistWorkspace()" /><span>Enable audit log for this workspace</span></label>
  <label>Author identity<input v-model="store.auditAuthor" type="text" :placeholder="store.businessProfile.fullName || 'Your name'" @blur="void store.persistWorkspace()" /></label>
  <label>Max log file size (bytes, 0 = unlimited)<input v-model.number="store.auditMaxBytes" type="number" min="0" max="100000000" step="1000000" @blur="void store.persistWorkspace()" /></label>
  <div style="margin-top:8px" v-if="store.workspaceRoot">
    <button type="button" @click="loadAuditLog()">View recent entries</button>
    <div v-if="auditLogEntries.length" class="audit-log-preview">
      <div v-for="entry in auditLogEntries.slice(0, 20)" :key="entry.timestamp" class="audit-entry">
        <span class="audit-ts">{{ entry.timestamp.slice(0, 19).replace('T', ' ') }}</span>
        <span class="audit-event">{{ entry.event }}</span>
        <span v-if="entry.document_title" class="audit-doc">{{ entry.document_title }}</span>
      </div>
    </div>
  </div>
  <h3>History retention</h3>
  <div style="display:flex;gap:8px;flex-wrap:wrap;margin-top:6px">
    <button type="button" @click="store.clearAgentRunHistory(); void store.persistWorkspace()">Clear agent history ({{ store.agentRunHistory.length }})</button>
    <button type="button" @click="store.clearDocsLiveDraftHistory(); void store.persistWorkspace()">Clear Docs Live history ({{ store.docsLiveDraftHistory.length }})</button>
  </div>
  <h3>AI humanizer</h3>
  <label>Default intensity<select v-model="store.humanizerDefaultMode" @change="void store.persistWorkspace()"><option value="light">Light</option><option value="standard">Standard (recommended)</option><option value="heavy">Heavy</option></select></label>
  <h3>Document comparison</h3>
  <label>Max lines (prevents OOM)<input v-model.number="store.compareMaxLines" type="number" min="100" max="50000" step="500" @blur="void store.persistWorkspace()" /></label>
  <label class="compact-check"><input v-model="store.compareIgnoreWhitespace" type="checkbox" @change="void store.persistWorkspace()" /><span>Ignore leading/trailing whitespace</span></label>
</section>
<section v-show="selectedConfigurationSection === 'support'" class="configuration-center-panel" aria-label="Support and diagnostics configuration">
  <section class="transform-handler-installer" aria-label="Configurator support bundle">
    <header>
      <div>
        <h3>Support bundle</h3>
        <span>Create a redaction-safe setup and release-readiness handoff for help desks, release managers, or internal IT.</span>
      </div>
      <button type="button" :disabled="supportBundleBusy" @click="previewSupportBundle">
        {{ supportBundleBusy ? "Building..." : "Preview support bundle" }}
      </button>
    </header>
    <div class="support-bundle-actions">
      <button type="button" :disabled="supportBundleBusy" @click="saveSupportBundle">Save JSON</button>
      <button type="button" :disabled="!supportBundleReport" @click="insertSupportBundleHandoff">Insert handoff</button>
      <button type="button" :disabled="!supportBundleReport" @click="insertEvidenceReturnPacket">Insert evidence packet</button>
      <button type="button" :disabled="!supportBundleManualReviewWorkOrders.length" @click="insertManualReviewSignoffKit">Insert manual review kit</button>
      <button type="button" @click="openConfigurationSetup('support')">Open support setup wizard</button>
      <span>{{ supportBundleStatus || "The bundle contains setup status and evidence summaries, not document content or secrets." }}</span>
    </div>
    <dl v-if="supportBundleReport" class="transform-installer-summary">
      <div>
        <dt>Doctor</dt>
        <dd>{{ supportBundleReport.doctor?.status || "unknown" }}</dd>
      </div>
      <div>
        <dt>Release</dt>
        <dd>{{ supportBundleReport.releaseReadiness?.status || "unknown" }}</dd>
      </div>
      <div>
        <dt>Evidence attention</dt>
        <dd>{{ (supportBundleReport.evidenceReportSummary?.attention || 0) + (supportBundleReport.evidenceReportSummary?.missing || 0) }}</dd>
      </div>
      <div>
        <dt>Recommendations</dt>
        <dd>{{ supportBundleReport.recommendations?.length || 0 }}</dd>
      </div>
      <div>
        <dt>100 improvements</dt>
        <dd>
          {{ supportBundleReport.improvementAudit?.summary?.implementedEvidencePresent || 0 }}/{{ supportBundleReport.improvementAudit?.total || 0 }} evidenced,
          {{ supportBundleReport.improvementAudit?.summary?.open || 0 }} open
        </dd>
      </div>
      <div>
        <dt>Output</dt>
        <dd>{{ supportBundleReport.writtenTo || "preview only" }}</dd>
      </div>
    </dl>
    <section v-if="supportBundleRecommendationGroups.length" class="support-bundle-recommendations" aria-label="Configurator support bundle recommendations">
      <article v-for="group in supportBundleRecommendationGroups" :key="group.id" :data-priority="group.priority">
        <header>
          <span>{{ group.label }}</span>
          <small>{{ group.items.length }} item{{ group.items.length === 1 ? "" : "s" }}</small>
        </header>
        <ul>
          <li v-for="recommendation in group.items" :key="recommendation">{{ recommendation }}</li>
        </ul>
      </article>
    </section>
    <section v-if="supportBundleReport?.improvementAudit" class="support-bundle-action-plan" aria-label="Configurator 100 improvements coverage">
      <h5>100 improvements coverage</h5>
      <article>
        <strong>{{ supportBundleReport.improvementAudit.implementationReady ? "implementation-ready" : "not complete" }}</strong>
        <span>
          {{ supportBundleReport.improvementAudit.summary?.implementedEvidencePresent || 0 }} evidenced,
          {{ supportBundleReport.improvementAudit.summary?.partialOrExternal || 0 }} partial/external,
          {{ supportBundleReport.improvementAudit.summary?.needsImplementationEvidence || 0 }} need implementation evidence,
          production {{ supportBundleReport.improvementAudit.productionReady ? "ready" : "blocked" }}
        </span>
        <small>{{ supportBundleReport.improvementAudit.releaseReadiness?.status || "release readiness not checked" }} | {{ supportBundleReport.improvementAudit.releaseReadiness?.evidenceGaps || 0 }} release evidence gap(s) | <code>ned improvements --json</code> | <code>ned improvements --output improvement-coverage.md</code></small>
      </article>
    </section>
    <p class="sidebar-hint">Use this support artifact when a non-technical user needs help configuring NEditor, validating release readiness, or handing setup evidence to internal IT without sharing document content or secrets.</p>
  </section>
</section>
<section v-show="selectedConfigurationSection === 'transforms'" class="configuration-center-panel" aria-label="Transform engine configuration">
<h3>Transform engines</h3>
<section class="transform-handler-installer" aria-label="Transform handler installer">
  <header>
    <div>
      <h4>Download and install transform handlers</h4>
      <span>Use a managed setup plan for every external transform handler before choosing trusted executable paths.</span>
    </div>
    <button type="button" @click="loadTransformHandlerInstallers">Refresh installer options</button>
  </header>
  <label>
    Installer profile
    <select v-model="selectedTransformInstallerId">
      <option v-for="plan in transformInstallerPlans" :key="plan.id" :value="plan.id">
        {{ plan.label }}
      </option>
    </select>
  </label>
  <dl v-if="selectedTransformInstallerPlan" class="transform-installer-summary">
    <div>
      <dt>Platform</dt>
      <dd>{{ selectedTransformInstallerPlan.platform }}</dd>
    </div>
    <div>
      <dt>Manager</dt>
      <dd>{{ selectedTransformInstallerPlan.manager }}</dd>
    </div>
    <div>
      <dt>Mode</dt>
      <dd>{{ selectedTransformInstallerPlan.installable ? "Can start from NEditor" : "Copy commands and run in a terminal" }}</dd>
    </div>
    <div>
      <dt>Privilege</dt>
      <dd>{{ selectedTransformInstallerPlan.requires_admin ? "May ask for administrator access" : "No administrator prompt expected from NEditor" }}</dd>
    </div>
    <div>
      <dt>Coverage</dt>
      <dd>{{ transformInstallerCoverageSummary }}</dd>
    </div>
  </dl>
  <p v-if="missingTransformInstallerEngines.length" class="engine-setup-status failed" role="alert">
    Missing installer coverage for {{ missingTransformInstallerEngines.join(", ") }}.
  </p>
  <p v-else-if="selectedTransformInstallerPlan" class="engine-setup-status ok" role="note">
    Installer plan covers all external transform handlers currently registered by NEditor.
  </p>
  <p v-if="selectedTransformInstallerPlan" class="engine-summary">
    Engines: {{ selectedTransformInstallerPlan.engine_names?.join(", ") || "none" }}
  </p>
  <ul v-if="selectedTransformInstallerPlan" class="transform-installer-handlers">
    <li v-for="handler in selectedTransformInstallerPlan.handlers" :key="handler">{{ handler }}</li>
  </ul>
  <pre v-if="transformInstallerCommandText" class="transform-installer-commands">{{ transformInstallerCommandText }}</pre>
  <div class="reference-actions">
    <button
      type="button"
      :disabled="!selectedTransformInstallerPlan?.installable || transformInstallerBusy"
      @click="startTransformHandlerInstall"
    >
      {{ transformInstallerBusy ? "Starting..." : "Download/install all handlers" }}
    </button>
    <button type="button" :disabled="!transformInstallerCommandText" @click="copyTransformInstallerCommands">Copy commands</button>
  </div>
  <p class="engine-setup-status" role="status">
    {{ transformInstallerStatus || selectedTransformInstallerPlan?.notes?.join(" ") || "Installer options will appear after setup loads." }}
  </p>
</section>
<label>
  Timeout
  <input
    :value="store.transformTimeoutMs"
    type="number"
    min="1"
    max="30000"
    step="250"
    @input="store.setTransformTimeout(Number(eventValue($event)))"
    @change="store.setTransformTimeout(Number(eventValue($event)))"
  />
</label>
<section class="database-profile-manager" aria-label="Safe database profile manager">
  <header>
    <div>
      <h4>Safe database profiles</h4>
      <span>{{ databaseProfileSummaryText }}</span>
    </div>
    <button type="button" @click="resetDatabaseProfileDraft">New profile</button>
  </header>
  <section class="agent-provider-grid">
    <label>
      Profile name
      <input v-model="databaseProfileDraft.name" placeholder="Client reporting warehouse" />
    </label>
    <label>
      Driver
      <select v-model="databaseProfileDraft.driver">
        <option v-for="driver in databaseProfileDrivers" :key="driver.value" :value="driver.value">{{ driver.label }}</option>
      </select>
    </label>
    <label>
      Connection mode
      <select v-model="databaseProfileDraft.connectionMode">
        <option v-for="mode in databaseProfileConnectionModes" :key="mode.value" :value="mode.value">{{ mode.label }}</option>
      </select>
    </label>
    <label v-if="databaseProfileDraft.connectionMode === 'file'">
      Database path
      <input v-model="databaseProfileDraft.databasePath" placeholder="data/example.sqlite" />
    </label>
    <label v-if="databaseProfileDraft.connectionMode === 'environment'">
      DSN environment variable
      <input v-model="databaseProfileDraft.dsnEnv" placeholder="NEDITOR_DATABASE_URL" />
    </label>
    <label>
      Host
      <input v-model="databaseProfileDraft.host" placeholder="db.example.internal" />
    </label>
    <label>
      Port
      <input v-model="databaseProfileDraft.port" placeholder="5432" />
    </label>
    <label>
      Database
      <input v-model="databaseProfileDraft.databaseName" placeholder="analytics" />
    </label>
    <label>
      Username
      <input v-model="databaseProfileDraft.username" placeholder="readonly_user" />
    </label>
    <label>
      Secret environment variable
      <input v-model="databaseProfileDraft.secretEnv" placeholder="NEDITOR_DB_PASSWORD" />
    </label>
    <label>
      Tags
      <input :value="databaseProfileDraft.tags.join(', ')" placeholder="client, reporting, readonly" @input="databaseProfileDraft.tags = inputValue($event).split(',').map((item) => item.trim()).filter(Boolean)" />
    </label>
  </section>
  <label><input v-model="databaseProfileDraft.readonly" type="checkbox" /> Read-only profile</label>
  <label>
    Notes
    <textarea v-model="databaseProfileDraft.notes" rows="3" placeholder="Access scope, owner, allowed datasets, review notes"></textarea>
  </label>
  <dl class="database-profile-preview">
    <div v-for="row in databaseProfileDraftRows" :key="row.label">
      <dt>{{ row.label }}</dt>
      <dd>{{ row.value }}</dd>
    </div>
  </dl>
  <p v-for="warning in databaseProfileDraftWarnings" :key="warning" class="engine-setup-status failed" role="alert">{{ warning }}</p>
  <div class="reference-actions">
    <button type="button" @click="saveDatabaseProfileDraft">Save profile</button>
    <button type="button" :disabled="!activeDatabaseProfile" @click="insertActiveDatabaseProfileSqlTransform">Insert SQL from selected profile</button>
  </div>
  <label v-if="store.databaseProfiles.length">
    Saved profiles
    <select v-model="selectedDatabaseProfileId" @change="loadSelectedDatabaseProfile">
      <option value="">Choose a profile</option>
      <option v-for="profile in store.databaseProfiles" :key="profile.id" :value="profile.id">{{ profile.name }}</option>
    </select>
  </label>
  <article v-for="profile in store.databaseProfiles" :key="profile.id" class="engine-row">
    <h4>{{ profile.name }}</h4>
    <small>{{ databaseProfileSummary(profile) }}</small>
    <small>{{ databaseProfileWarnings(profile).join(' ') || "No secret or readiness warnings." }}</small>
    <div class="reference-actions">
      <button type="button" @click="editDatabaseProfile(profile)">Edit</button>
      <button type="button" @click="insertDatabaseProfileSqlTransform(profile)">Insert SQL</button>
      <button type="button" @click="store.deleteDatabaseProfile(profile.id)">Delete</button>
    </div>
  </article>
</section>
<article v-for="engine in store.externalTransformEngines" :key="engine.name" class="engine-row">
  <h4>{{ engine.name }}</h4>
  <small>{{ engine.execution }}</small>
  <small>{{ engine.installationLabel }}</small>
  <small>{{ engine.setupHint }}</small>
  <small>{{ engine.adapterProfile }} Default command: {{ engine.defaultCommand }}</small>
  <small v-if="engine.diagnosticProfile.versionProbe">Version probe: {{ engine.diagnosticProfile.versionProbe }}</small>
  <small v-if="engine.diagnosticProfile.failureHint">Failure hint: {{ engine.diagnosticProfile.failureHint }}</small>
  <small>{{ engine.securitySummary }}</small>
  <p :class="['engine-setup-status', externalEngineSetupStatus(engine).status]" role="note">
    <strong>Setup status:</strong> {{ externalEngineSetupStatus(engine).message }}
  </p>
  <label>
    Engine path
    <span class="path-picker">
      <input :value="store.transformEnginePaths[engine.name] || ''" @change="store.setTransformEnginePath(engine.name, eventValue($event))" />
      <button type="button" @click="chooseTransformEngine(engine.name)">Choose</button>
    </span>
  </label>
  <label><input :checked="Boolean(store.trustedTransformEngines[engine.name])" type="checkbox" @change="toggleTransformTrust(engine.name, $event)" /> Trusted</label>
  <small v-if="store.transformEnginePaths[engine.name] && !store.trustedTransformEngines[engine.name]" class="engine-trust-note">
    Trust was cleared because the executable path changed.
  </small>
  <label><input :checked="Boolean(store.disabledTransformEngines[engine.name])" type="checkbox" @change="store.setTransformDisabled(engine.name, eventChecked($event))" /> Disable external engine</label>
  <label>
    Input
    <select :value="store.transformInputModes[engine.name] || 'stdin'" @change="store.setTransformInputMode(engine.name, eventValue($event) === 'file' ? 'file' : 'stdin')">
      <option v-for="mode in engine.inputModes" :key="mode" :value="mode">{{ mode }}</option>
    </select>
  </label>
  <button type="button" @click="store.testExternalTransform(engine.name)">Probe</button>
  <article
    v-if="store.transformProbeResults[engine.name]"
    :class="['engine-probe', store.transformProbeResults[engine.name].ok ? 'ok' : 'failed']"
  >
    <strong>{{ store.transformProbeResults[engine.name].ok ? "Probe passed" : "Probe failed" }}</strong>
    <p>{{ store.transformProbeResults[engine.name].message }}</p>
    <small v-if="store.transformProbeResults[engine.name].cacheKey">Cache: {{ store.transformProbeResults[engine.name].cacheKey }}</small>
    <ul v-if="store.transformProbeResults[engine.name].diagnostics.length">
      <li v-for="diagnostic in store.transformProbeResults[engine.name].diagnostics" :key="diagnostic">{{ diagnostic }}</li>
    </ul>
  </article>
</article>
<p v-for="engine in store.transformEngines.filter((candidate) => !candidate.requiresExecution)" :key="engine.name" class="engine-summary">
  {{ engine.name }}: {{ engine.execution }} | {{ engine.installationLabel }} | {{ engine.securitySummary }}
</p>
<section class="trust-store-panel" aria-label="Backend trust store">
  <h4>Backend trust store <button type="button" @click="loadTrustStore" title="Refresh">↻</button></h4>
  <p v-if="!trustedEngineStore.length" class="sidebar-hint">No engines currently trusted in the backend store.</p>
  <article v-for="entry in trustedEngineStore" :key="entry.engine_path" class="trust-store-entry">
    <strong>{{ entry.transform_name }}</strong>
    <small :class="entry.valid ? 'trust-valid' : 'trust-invalid'">{{ entry.valid ? "fingerprint valid" : "fingerprint mismatch — re-trust required" }}</small>
    <code class="trust-path">{{ entry.engine_path }}</code>
    <button type="button" @click="revokeEngineFromStore(entry.engine_path)">Revoke</button>
  </article>
  <details class="trust-add-form">
    <summary>Trust an engine</summary>
    <label>Engine path <input v-model="trustEnginePath" type="text" placeholder="/absolute/path/to/engine" /></label>
    <label>Transform name <input v-model="trustEngineName" type="text" placeholder="engine-name" /></label>
    <button type="button" @click="trustExternalEngine">Trust engine</button>
    <p v-if="trustEngineError" class="sidebar-hint trust-error">{{ trustEngineError }}</p>
  </details>
</section>
</section>
</template>


<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDocumentsStore } from "../stores/documents";
import {
  aiProviderProfiles,
  localAgentCliProfiles,
} from "../lib/aiProviderPackages";
import {
  brandKitPresets,
} from "../lib/brandKitPresets";
import { ttsEngineOptions } from "../lib/ttsSetup";
import {
  databaseProfileConnectionModes,
  databaseProfileDrivers,
  databaseProfileSummary,
  databaseProfileWarnings,
} from "../lib/databaseProfiles";
import type { OllamaHealthResult } from "../lib/ollamaModels.js";
import { OLLAMA_MODEL_CATALOG, checkOllamaHealth, computeContextBudget } from "../lib/ollamaModels.js";

const store = useDocumentsStore();

// ── Props ────────────────────────────────────────────────────────────────────
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const props = defineProps<{
  // v-model (writable via emit)
  agentProviderId: string;
  agentProviderModel: string;
  agentProviderApiKey: string;
  agentProviderEndpoint: string;
  agentProviderKeyEnv: string;
  googleClientId: string;
  googleAccountHint: string;
  googleScopesText: string;
  googleOfflineAccess: boolean;
  selectedBrandKitPresetId: string;
  selectedTransformInstallerId: string;
  selectedDatabaseProfileId: string;
  trustEnginePath: string;
  trustEngineName: string;
  // readonly state from App.vue
  agentProviderBusy: boolean;
  isOllamaProvider: boolean;
  ollamaModelBusy: boolean;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ollamaModelOptions: any[];
  ollamaSelectedModelMetadata: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  currentAgentProviderProfile: any;
  currentModelMissingFromOllamaList: boolean;
  ollamaModelPickerHelp: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ttsModelDownloadPlan: any;
  ttsModelDownloadBusy: boolean;
  ttsInspectionBusy: boolean;
  ttsReadDisabled: boolean;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ttsInspectionReport: any;
  ttsStatus: string;
  ttsSetupSummary: string;
  ttsRuntimeSummary: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  releaseEvidenceDashboard: any;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  productionReadinessWorkOrders: any[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  supportBundleReport: any;
  supportBundleStatus: string;
  supportBundleBusy: boolean;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  supportBundleManualReviewWorkOrders: any[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  supportBundleRecommendationGroups: any[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  configurationSetupStatus: any;
  configurationSetupSummary: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  configurationCenterSections: readonly any[];
  selectedConfigurationSection: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  selectedBrandKitPreset: any;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  currentBrandKitPreviewRows: any[];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  accessibilityQaReport: any;
  googleAuthBusy: boolean;
  googleAuthStatus: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  googleAuthSession: any;
  googleAccessToken: string;
  googleRefreshToken: string;
  googleTokenScope: string;
  googleTokenExpiresAt: string;
  googleAuthPollStartedAt: string;
  googleDocsImportBusy: boolean;
  googleDocsImportStatus: string;
  googleDocsLiveDocumentId: string;
  googleDocsLiveDocumentUrl: string;
  googleDocsReadbackPreview: string;
  googleScopeList: string[];
  googleAuthSummary: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  transformInstallerPlans: any[];
  transformInstallerBusy: boolean;
  transformInstallerStatus: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  selectedTransformInstallerPlan: any;
  missingTransformInstallerEngines: string[];
  transformInstallerCoverageSummary: string;
  transformInstallerCommandText: string;
  cliDeployBusy: boolean;
  cliDeployStatus: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  cliDeployPlan: any;
  defaultMarkdownReaderBusy: boolean;
  defaultMarkdownReaderStatus: string;
  defaultMarkdownReaderEnabled: boolean;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  defaultMarkdownReaderPlan: any;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  databaseProfileDraft: any;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  activeDatabaseProfile: any;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  databaseProfileDraftRows: any[];
  databaseProfileDraftWarnings: string[];
  databaseProfileSummaryText: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  trustedEngineStore: any[];
  trustEngineError: string;
  // callback function props
  selectConfigurationSection: (id: string) => void;
  openConfigurationSetup: (stepId?: string) => void;
  openPreviewThemeGallery: () => void;
  syncAgentProviderProfile: () => void;
  refreshOllamaModels: () => void;
  confirmOllamaModelSelection: () => void;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  formatOllamaModelOption: (model: any) => string;
  saveAgentProviderDefaults: () => void;
  saveGoogleIntegrationSetup: () => void;
  startGoogleSignIn: () => void;
  pollGoogleSignIn: () => void;
  importCurrentDocumentToGoogleDocs: () => void;
  readBackGoogleDocsImport: () => void;
  refreshGoogleAccessTokenNow: () => void;
  copyGoogleAccessToken: () => void;
  clearGoogleSession: () => void;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  applyBrandKitPreset: (preset: any) => void;
  applySelectedBrandKitPreset: () => void;
  insertAccessibilityQaReport: () => void;
  downloadSelectedTtsModel: () => void;
  checkTtsRuntime: () => void;
  readSelectionAloud: () => void;
  readDocumentAloud: () => void;
  stopReadingAloud: () => void;
  saveTtsModelDownloadAcknowledgement: () => void;
  copyTtsModelDownloadCommand: () => void;
  insertReleaseEvidenceDashboard: () => void;
  insertReleaseReadinessAudit: () => void;
  insertProductionReadinessWorkOrders: () => void;
  previewSupportBundle: () => void;
  saveSupportBundle: () => void;
  insertSupportBundleHandoff: () => void;
  insertEvidenceReturnPacket: () => void;
  insertManualReviewSignoffKit: () => void;
  deployCliGlobally: () => void;
  loadCliDeployPlan: () => void;
  toggleDefaultMarkdownReader: (event: Event) => void;
  loadTransformHandlerInstallers: () => void;
  startTransformHandlerInstall: () => void;
  copyTransformInstallerCommands: () => void;
  chooseTransformEngine: (name: string) => void;
  toggleTransformTrust: (name: string, event: Event) => void;
  loadTrustStore: () => void;
  revokeEngineFromStore: (enginePath: string) => void;
  trustExternalEngine: () => void;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  externalEngineSetupStatus: (engine: any) => { status: string; message: string };
  saveDatabaseProfileDraft: () => void;
  resetDatabaseProfileDraft: () => void;
  loadSelectedDatabaseProfile: () => void;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  editDatabaseProfile: (profile: any) => void;
  insertActiveDatabaseProfileSqlTransform: () => void;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  insertDatabaseProfileSqlTransform: (profile: any) => void;
  // Pandoc/curl probe — kept in App.vue to feed configurationSetupStatus
  pandocAvailable: boolean;
  curlAvailable: boolean;
  pandocProbeResult: string;
  curlProbeResult: string;
  probeImportTools: () => void;
}>();

const emit = defineEmits<{
  'update:agentProviderId': [v: string];
  'update:agentProviderModel': [v: string];
  'update:agentProviderApiKey': [v: string];
  'update:agentProviderEndpoint': [v: string];
  'update:agentProviderKeyEnv': [v: string];
  'update:googleClientId': [v: string];
  'update:googleAccountHint': [v: string];
  'update:googleScopesText': [v: string];
  'update:googleOfflineAccess': [v: boolean];
  'update:selectedBrandKitPresetId': [v: string];
  'update:selectedTransformInstallerId': [v: string];
  'update:selectedDatabaseProfileId': [v: string];
  'update:trustEnginePath': [v: string];
  'update:trustEngineName': [v: string];
}>();

// ── Writable computed refs for v-model props ─────────────────────────────────
const agentProviderId = computed({ get: () => props.agentProviderId, set: (v) => emit('update:agentProviderId', v) });
const agentProviderModel = computed({ get: () => props.agentProviderModel, set: (v) => emit('update:agentProviderModel', v) });
const agentProviderApiKey = computed({ get: () => props.agentProviderApiKey, set: (v) => emit('update:agentProviderApiKey', v) });
const agentProviderEndpoint = computed({ get: () => props.agentProviderEndpoint, set: (v) => emit('update:agentProviderEndpoint', v) });
const agentProviderKeyEnv = computed({ get: () => props.agentProviderKeyEnv, set: (v) => emit('update:agentProviderKeyEnv', v) });
const googleClientId = computed({ get: () => props.googleClientId, set: (v) => emit('update:googleClientId', v) });
const googleAccountHint = computed({ get: () => props.googleAccountHint, set: (v) => emit('update:googleAccountHint', v) });
const googleScopesText = computed({ get: () => props.googleScopesText, set: (v) => emit('update:googleScopesText', v) });
const googleOfflineAccess = computed({ get: () => props.googleOfflineAccess, set: (v) => emit('update:googleOfflineAccess', v) });
const selectedBrandKitPresetId = computed({ get: () => props.selectedBrandKitPresetId, set: (v) => emit('update:selectedBrandKitPresetId', v) });
const selectedTransformInstallerId = computed({ get: () => props.selectedTransformInstallerId, set: (v) => emit('update:selectedTransformInstallerId', v) });
const selectedDatabaseProfileId = computed({ get: () => props.selectedDatabaseProfileId, set: (v) => emit('update:selectedDatabaseProfileId', v) });
const trustEnginePath = computed({ get: () => props.trustEnginePath, set: (v) => emit('update:trustEnginePath', v) });
const trustEngineName = computed({ get: () => props.trustEngineName, set: (v) => emit('update:trustEngineName', v) });

// ── Internal state (settings-panel only) ─────────────────────────────────────

// Ollama health / model management
const ollamaHealth = ref<OllamaHealthResult | null>(null);
const ollamaHealthBusy = ref(false);
const ollamaPullModelId = ref("");
const ollamaPullBusy = ref(false);
const ollamaPullError = ref("");
const ollamaPullSuccess = ref("");
const ollamaDeleteBusy = ref(false);
const ollamaInstalledModels = ref<string[]>([]);

const isOllamaProfile = computed(() =>
  store.aiProviderDefaults.profileId === "ollama-local" ||
  store.aiProviderDefaults.profileId === "ollama-cloud"
);

const activeModelCard = computed(() =>
  OLLAMA_MODEL_CATALOG.find(m => m.id === store.aiProviderDefaults.model)
);

const contextBudgetInfo = computed(() => {
  if (!isOllamaProfile.value || !activeModelCard.value) return null;
  const docText = store.activeDocument?.text ?? "";
  const sysPrompt = "You are an expert document co-writer inside NEditor. Return only well-structured Markdown.";
  return computeContextBudget(sysPrompt, docText, activeModelCard.value.numCtx, 4096);
});

async function probeOllamaHealth(): Promise<void> {
  if (ollamaHealthBusy.value) return;
  ollamaHealthBusy.value = true;
  const endpoint = store.aiProviderDefaults.endpoint || "http://127.0.0.1:11434/api/chat";
  ollamaHealth.value = await checkOllamaHealth(endpoint);
  if (ollamaHealth.value.running) {
    try {
      const resp = await invoke<{ models: Array<{ name: string }> }>("list_ollama_models", {
        request: { endpoint, auth_header: null, api_key: null, key_env: null },
      });
      ollamaInstalledModels.value = resp.models.map((m) => m.name);
    } catch { ollamaInstalledModels.value = []; }
  }
  ollamaHealthBusy.value = false;
}

async function pullOllamaModel(modelId: string): Promise<void> {
  ollamaPullBusy.value = true;
  ollamaPullError.value = "";
  ollamaPullSuccess.value = "";
  ollamaPullModelId.value = modelId;
  try {
    const endpoint = store.aiProviderDefaults.endpoint || "http://127.0.0.1:11434/api/chat";
    const result = await invoke<{ success: boolean; error: string; status: string }>("pull_ollama_model", {
      request: { endpoint, model: modelId },
    });
    if (result.success) {
      ollamaPullSuccess.value = `${modelId} installed successfully`;
      await probeOllamaHealth();
    } else {
      ollamaPullError.value = result.error || "Pull failed";
    }
  } catch (e) {
    ollamaPullError.value = String(e);
  } finally {
    ollamaPullBusy.value = false;
    ollamaPullModelId.value = "";
  }
}

async function deleteOllamaModel(modelName: string): Promise<void> {
  ollamaDeleteBusy.value = true;
  try {
    const endpoint = store.aiProviderDefaults.endpoint || "http://127.0.0.1:11434/api/chat";
    await invoke("delete_ollama_model", { request: { endpoint, model: modelName } });
    await probeOllamaHealth();
  } catch (e) { store.statusMessage = `Delete failed: ${e}`; }
  finally { ollamaDeleteBusy.value = false; }
}

function useOllamaModel(modelId: string): void {
  store.aiProviderDefaults = { ...store.aiProviderDefaults, model: modelId };
  void store.persistWorkspace();
  store.statusMessage = `Switched to ${modelId}`;
}

watch(isOllamaProfile, (v) => { if (v) void probeOllamaHealth(); });

// pandocAvailable, curlAvailable, pandocProbeResult, curlProbeResult, probeImportTools
// stay in App.vue (used in configurationSetupStatus) and are passed as props above

// Webhook management
const newWebhookDraft = ref({ name: "", url: "", events: ["status-changed"] as string[], enabled: true });

const WEBHOOK_EVENTS = [
  { id: "status-changed", label: "Status changed" },
  { id: "exported", label: "Exported" },
  { id: "approved", label: "Approved / locked" },
  { id: "saved", label: "Saved" },
];

function addWebhook(): void {
  const draft = newWebhookDraft.value;
  if (!draft.name.trim() || !draft.url.trim()) return;
  const id = "wh-" + String(store.webhookConfigs.length + 1) + "-" + draft.name.trim().slice(0, 8).replace(/\s+/g, "");
  store.webhookConfigs = [...store.webhookConfigs, { id, name: draft.name.trim(), url: draft.url.trim(), events: [...draft.events], enabled: true }];
  newWebhookDraft.value = { name: "", url: "", events: ["status-changed"], enabled: true };
  void store.persistWorkspace();
}

function removeWebhook(id: string): void {
  store.webhookConfigs = store.webhookConfigs.filter(w => w.id !== id);
  void store.persistWorkspace();
}

function toggleWebhook(id: string): void {
  store.webhookConfigs = store.webhookConfigs.map(w => w.id === id ? { ...w, enabled: !w.enabled } : w);
  void store.persistWorkspace();
}

// Audit log
const auditLogEntries = ref<{ timestamp: string; event: string; document_title?: string }[]>([]);

async function loadAuditLog(): Promise<void> {
  try {
    const result = await invoke<{ entries: { timestamp: string; event: string; document_title?: string }[] }>("read_audit_log", { workspaceRoot: store.workspaceRoot }).catch(() => ({ entries: [] }));
    auditLogEntries.value = result.entries || [];
  } catch { auditLogEntries.value = []; }
}

// ── Template utilities ────────────────────────────────────────────────────────
function eventValue(event: Event): string {
  return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement ? event.target.value : "";
}

function eventChecked(event: Event): boolean {
  return event.target instanceof HTMLInputElement ? event.target.checked : false;
}

function inputValue(event: Event): string {
  return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement ? event.target.value : "";
}


</script>
