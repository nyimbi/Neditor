<template>
  <h2>Templates <small>{{ filteredTransformTemplates.length }}</small></h2>
  <section class="business-template-hub" aria-label="Business document creation">
    <header>
      <div>
        <strong>Business identity</strong>
        <span>Saved sender, company, address, website, and voice values for repeatable documents.</span>
      </div>
      <small>{{ businessProfileCompletion }}</small>
    </header>
    <div class="template-actions">
      <button type="button" title="Set up the business identity values reused in proposals, tenders, RFQs, and snippets" @click="openBusinessProfile">
        <span class="button-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" focusable="false">
            <path v-for="path in toolbarIconPaths('settings')" :key="path" :d="path"></path>
          </svg>
        </span>
        Business info
      </button>
      <button
        type="button"
        title="Insert the saved contact block into the current document"
        @click="insertBusinessSnippet(businessDocumentSnippets[0])"
      >
        <span class="button-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" focusable="false">
            <path v-for="path in toolbarIconPaths('templates')" :key="path" :d="path"></path>
          </svg>
        </span>
        Contact block
      </button>
    </div>
    <p class="sidebar-hint">
      {{ store.businessProfile.companyName || "No company saved yet" }}
      <span v-if="store.businessProfile.email"> | {{ store.businessProfile.email }}</span>
      <span v-if="store.businessProfile.website"> | {{ store.businessProfile.website }}</span>
    </p>
  </section>
  <section class="business-template-hub" aria-label="AI document creation wizard">
    <header>
      <div>
        <strong>Document creation wizard</strong>
        <span>Start common business documents with identity, placeholders, outline, QA, humanization, and agent handoff.</span>
      </div>
    </header>
    <label>
      Find a document type
      <input v-model="businessTemplateQuery" placeholder="proposal, RFP, RFQ, tender, tutorial" />
    </label>
    <div class="business-template-list" role="list" aria-label="Business development document templates">
      <article v-for="template in filteredBusinessTemplates" :key="template.id" class="template-card business-document-card" role="listitem">
        <header class="template-card-header">
          <div>
            <strong>{{ template.label }}</strong>
            <small>{{ template.summary }}</small>
          </div>
          <span class="template-source">wizard</span>
        </header>
        <div class="template-tags" aria-label="Best-fit uses">
          <small v-for="item in template.bestFor" :key="`${template.id}-${item}`">{{ item }}</small>
        </div>
        <details>
          <summary>Outline</summary>
          <ol>
            <li v-for="heading in template.outline" :key="`${template.id}-${heading}`">{{ heading }}</li>
          </ol>
        </details>
        <details class="business-wizard-assistance">
          <summary>AI step assistance</summary>
          <ol>
            <li v-for="item in businessWizardStepAssistance(template)" :key="`${template.id}-${item.stepId}`">
              <strong>{{ item.stepLabel }}</strong>
              <p>{{ item.suggestedAnswer }}</p>
              <small>{{ item.contextSignals.join(" | ") }}</small>
            </li>
          </ol>
        </details>
        <div class="template-actions">
          <button type="button" :title="`Insert a fillable ${template.label} Markdown template`" @click="insertBusinessTemplate(template)">
            <span class="button-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" focusable="false">
                <path v-for="path in toolbarIconPaths('new')" :key="path" :d="path"></path>
              </svg>
            </span>
            Insert
          </button>
          <button type="button" :title="`Open Docs Live wizard for ${template.label}`" @click="startBusinessDocumentWizard(template)">
            <span class="button-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" focusable="false">
                <path v-for="path in toolbarIconPaths('ai')" :key="path" :d="path"></path>
              </svg>
            </span>
            AI wizard
          </button>
          <button type="button" :title="`Prepare Claude Code, Codex, OpenCode, or Google Antigravity handoff for ${template.label}`" @click="openAgentWorkspaceForBusinessTemplate(template)">
            <span class="button-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" focusable="false">
                <path v-for="path in toolbarIconPaths('agent')" :key="path" :d="path"></path>
              </svg>
            </span>
            Agent handoff
          </button>
        </div>
      </article>
    </div>
  </section>
  <section class="business-template-hub rfp-response-wizard" aria-label="Native RFP response wizard">
    <header>
      <div>
        <strong>RFP response wizard</strong>
        <span>Import an RFP, create the compliance checklist, build the proposal outline, and then draft the response section by section.</span>
      </div>
      <small>{{ rfpAnalysisSummary }}</small>
    </header>
    <div class="rfp-source-grid">
      <label>
        Source type
        <select v-model="rfpSourceKind" aria-label="RFP source type">
          <option value="markdown">Markdown or pasted text</option>
          <option value="pdf">PDF</option>
          <option value="docx">DOCX</option>
          <option value="url">URL</option>
        </select>
      </label>
      <label>
        RFP URL
        <input v-model="rfpSourceUrl" type="url" placeholder="https://buyer.example/rfp" aria-label="RFP URL" />
      </label>
    </div>
    <label>
      RFP source text
      <textarea
        v-model="rfpSourceText"
        rows="7"
        aria-label="RFP source text"
        placeholder="Paste RFP text, extracted PDF/DOCX content, or use Import RFP file / Fetch URL."
      ></textarea>
    </label>
    <label>
      Response context and decision notes
      <textarea
        v-model="rfpResponseContextNotes"
        rows="5"
        aria-label="RFP response context notes"
        placeholder="Add win themes, known differentiators, red-team concerns, pricing caveats, reviewer instructions, or accept AI step guidance here."
      ></textarea>
    </label>
    <details class="business-wizard-assistance rfp-step-assistance" open>
      <summary>AI RFP step assistance</summary>
      <ol>
        <li v-for="item in rfpWizardStepAssistance" :key="item.stepId">
          <strong>{{ item.stepLabel }}</strong>
          <p>{{ item.suggestedAnswer }}</p>
          <p class="sidebar-hint">{{ item.rationale }}</p>
          <small>{{ item.contextSignals.join(" | ") }}</small>
          <button type="button" @click="appendRfpWizardSuggestion(item)">{{ item.actionLabel }}</button>
        </li>
      </ol>
    </details>
    <div class="template-actions">
      <button type="button" title="Import a PDF, DOCX, Markdown, or text RFP source through the native file picker" :disabled="rfpImportBusy" @click="importRfpSourceFile">
        <span class="button-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" focusable="false">
            <path v-for="path in toolbarIconPaths('open')" :key="path" :d="path"></path>
          </svg>
        </span>
        Import RFP file
      </button>
      <button type="button" title="Fetch and analyze a public RFP URL" :disabled="rfpImportBusy || !rfpSourceUrl.trim()" @click="importRfpSourceUrl">
        <span class="button-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" focusable="false">
            <path v-for="path in toolbarIconPaths('link')" :key="path" :d="path"></path>
          </svg>
        </span>
        Fetch URL
      </button>
      <button type="button" title="Use the active Markdown document as the RFP source" @click="loadActiveDocumentAsRfpSource">Use active doc</button>
      <button type="button" title="Analyze requirements, capabilities, timelines, budget hints, stated intent, and implied intent" @click="analyzeCurrentRfpSource">Analyze RFP</button>
    </div>
    <p v-if="rfpImportMessage" class="sidebar-hint">{{ rfpImportMessage }}</p>
    <section v-if="rfpAnalysis" class="rfp-analysis-panel" aria-label="RFP analysis results">
      <div class="rfp-analysis-metrics">
        <span><strong>{{ rfpAnalysis.requirements.length }}</strong> requirements</span>
        <span><strong>{{ rfpAnalysis.capabilities.length }}</strong> capabilities</span>
        <span><strong>{{ rfpAnalysis.timelines.length }}</strong> timeline hints</span>
        <span><strong>{{ rfpAnalysis.budgetHints.length }}</strong> budget hints</span>
        <span><strong>{{ rfpAnalysis.complianceChecklist.length }}</strong> checklist items</span>
        <span><strong>{{ rfpAnalysis.criticalDisqualifiers.length }}</strong> critical traps</span>
        <span><strong>{{ rfpAnalysis.proposalOutline.activities.length }}</strong> outline activities</span>
        <span><strong>{{ rfpAnalysis.winThemes.length }}</strong> win themes</span>
        <span><strong>{{ rfpAnalysis.verificationSummary.rowsNeedingEvidence }}</strong> evidence checks</span>
      </div>
      <details open>
        <summary>Submission package checklist</summary>
        <ul>
          <li>Deadline: {{ rfpAnalysis.proposalOutline.metadata.submissionDeadline }}</li>
          <li>Page limit: {{ rfpAnalysis.proposalOutline.metadata.pageLimitSource }}</li>
          <li>{{ rfpAnalysis.mandatoryAttachments.length }} attachment hint(s), {{ rfpAnalysis.annexReferences.length }} annex reference(s), {{ rfpAnalysis.bilingualRequirements.length }} language obligation(s)</li>
          <li>{{ rfpAnalysis.placeholderRisks.length }} placeholder trap(s), {{ rfpAnalysis.warnings.length }} source-capture warning(s)</li>
        </ul>
      </details>
      <details open>
        <summary>Proposal outline planner</summary>
        <ul>
          <li>Deadline: {{ rfpAnalysis.proposalOutline.metadata.submissionDeadline }}</li>
          <li>Page limit: {{ rfpAnalysis.proposalOutline.metadata.pageLimitSource }}</li>
          <li>Evaluation model: {{ rfpAnalysis.proposalOutline.metadata.evaluationModel }}</li>
          <li>Currency: {{ rfpAnalysis.proposalOutline.metadata.currency }}</li>
        </ul>
        <ol>
          <li v-for="item in rfpAnalysis.proposalOutline.activities.slice(0, 10)" :key="`${item.sourceLine}-${item.label}`">
            {{ item.label }}
            <small>Source line {{ item.sourceLine }} | {{ item.placeholder }}</small>
          </li>
        </ol>
      </details>
      <details open>
        <summary>Compliance checklist extractor</summary>
        <ul>
          <li v-for="item in rfpAnalysis.complianceChecklist.slice(0, 16)" :key="item.id">
            <strong>{{ item.id }}</strong> {{ item.requirement }}
            <small>{{ item.section }} | {{ item.risk }} | {{ item.owner }} | {{ item.reference }}</small>
            <p>{{ item.verification }}</p>
          </li>
        </ul>
      </details>
      <details v-if="rfpAnalysis.criticalDisqualifiers.length" open>
        <summary>Critical disqualification traps</summary>
        <ul>
          <li v-for="item in rfpAnalysis.criticalDisqualifiers" :key="item">{{ item }}</li>
        </ul>
      </details>
      <details open>
        <summary>Stated buyer intent</summary>
        <ul>
          <li v-for="item in rfpAnalysis.statedIntent" :key="`stated-${item}`">{{ item }}</li>
        </ul>
      </details>
      <details open>
        <summary>Implied buyer intent</summary>
        <ul>
          <li v-for="item in rfpAnalysis.impliedIntent" :key="`implied-${item}`">{{ item }}</li>
        </ul>
      </details>
      <details open>
        <summary>Win theme builder</summary>
        <ol>
          <li v-for="theme in rfpAnalysis.winThemes" :key="theme.id">
            <strong>{{ theme.id }}: {{ theme.title }}</strong>
            <p>{{ theme.buyerSignal }}</p>
            <small>{{ theme.proposalPlacement }} | {{ theme.proofPoint }}</small>
            <p class="sidebar-hint">{{ theme.riskToAvoid }}</p>
          </li>
        </ol>
      </details>
      <details open>
        <summary>Requirement verification</summary>
        <ul>
          <li v-for="item in rfpAnalysis.verificationSummary.checklist" :key="`verify-${item}`">{{ item }}</li>
        </ul>
        <ol>
          <li v-for="row in rfpAnalysis.complianceRows.slice(0, 12)" :key="row.id">
            <strong>{{ row.id }}</strong> {{ row.text }}
            <small>{{ row.category }} | {{ row.responseSection }} | {{ row.complianceStatus }} | {{ row.verification }}</small>
            <p>{{ row.suggestedResponse }}</p>
          </li>
        </ol>
      </details>
      <div class="template-actions">
        <button type="button" title="Insert only the generated compliance matrix into the active document" @click="insertRfpComplianceMatrix">Insert matrix</button>
        <button type="button" title="Insert deadline, page-limit, attachment, annex, language, placeholder, and evidence gates into the active document" @click="insertRfpSubmissionChecklist">Insert submission checklist</button>
        <button type="button" title="Insert evaluator-facing win themes into the active document" @click="insertRfpWinThemes">Insert win themes</button>
        <button type="button" title="Replace the active document with the compliance checklist followed by a scored proposal outline" @click="createRfpProposalOutline">Create outline</button>
        <button type="button" title="Replace the active document with a full responsive RFP response draft" @click="createResponsiveRfpResponse">Create response</button>
        <button type="button" title="Send the analyzed RFP to Docs Live for section-by-section drafting" @click="sendRfpResponseToDocsLive">Docs Live</button>
        <button type="button" title="Prepare a Claude Code, Codex, OpenCode, or Google Antigravity handoff for the analyzed RFP" @click="openAgentWorkspaceForRfpAnalysis">Agent handoff</button>
      </div>
    </section>
  </section>
  <section class="business-template-hub" aria-label="Reusable document parts">
    <header>
      <div>
        <strong>Reusable document parts</strong>
        <span>Insert standard sections without starting a full template.</span>
      </div>
    </header>
    <label>
      Find a part
      <input v-model="businessSnippetQuery" placeholder="scope, pricing, compliance, risk, review" />
    </label>
    <div class="versioned-clause-editor" aria-label="Custom reusable document part editor">
      <label>
        Part label
        <input v-model="customSnippetDraft.label" placeholder="Client onboarding checklist" />
      </label>
      <label>
        Kind
        <select v-model="customSnippetDraft.kind">
          <option value="identity">Identity</option>
          <option value="proposal">Proposal</option>
          <option value="procurement">Procurement</option>
          <option value="delivery">Delivery</option>
          <option value="governance">Governance</option>
          <option value="review">Review</option>
        </select>
      </label>
      <label>
        Summary
        <input v-model="customSnippetDraft.summary" placeholder="Where this reusable section should be used" />
      </label>
      <label>
        Part Markdown
        <textarea v-model="customSnippetDraft.body" rows="5" aria-label="Custom reusable document part Markdown"></textarea>
      </label>
      <div class="template-actions">
        <button type="button" title="Start a new reusable document part" @click="resetCustomSnippetDraft">New part</button>
        <button type="button" title="Save this reusable document part in the workspace library" @click="saveCustomBusinessSnippet">Save custom part</button>
        <button v-if="editingCustomSnippetId" type="button" title="Delete this reusable document part from the workspace library" @click="deleteEditingCustomBusinessSnippet">Delete custom part</button>
      </div>
      <p class="sidebar-hint">Custom document parts are profile-aware Markdown snippets. Use saved business-profile fields such as <code v-pre>{{companyName}}</code> and <code v-pre>{{defaultClientName}}</code> to keep repeated language consistent.</p>
    </div>
    <div class="snippet-list" role="list" aria-label="Standard document snippets">
      <article v-for="snippet in filteredBusinessSnippets" :key="snippet.id" class="snippet-card" role="listitem">
        <div>
          <strong>{{ snippet.label }}</strong>
          <small>{{ snippet.kind }} | {{ snippet.summary }}{{ store.customBusinessSnippets.some((item) => item.id === snippet.id) ? " | custom" : "" }}</small>
        </div>
        <button type="button" :title="`Insert ${snippet.label} into the document`" @click="insertBusinessSnippet(snippet)">Insert</button>
        <button v-if="store.customBusinessSnippets.some((item) => item.id === snippet.id)" type="button" :title="`Edit ${snippet.label}`" @click="editCustomBusinessSnippet(snippet)">Edit</button>
      </article>
    </div>
  </section>
  <section class="business-template-hub callout-palette" aria-label="Business callout and admonition styles">
    <header>
      <div>
        <strong>Business callouts</strong>
        <span>Insert styled decision, risk, evidence, warning, recommendation, assumption, action, and note boxes.</span>
      </div>
      <button type="button" :disabled="!selectedCalloutPreset" @click="insertSelectedCalloutPreset">Insert selected</button>
    </header>
    <section class="callout-selector">
      <label>
        Callout style
        <select v-model="selectedCalloutPresetId">
          <option v-for="preset in calloutPresets" :key="preset.id" :value="preset.id">{{ preset.label }}</option>
        </select>
      </label>
      <p v-if="selectedCalloutPreset" class="sidebar-hint">{{ selectedCalloutPreset.summary }}</p>
    </section>
    <div class="callout-grid" role="list" aria-label="Callout style library">
      <article v-for="preset in calloutPresets" :key="preset.id" class="callout-card" :data-tone="preset.tone" role="listitem">
        <div>
          <strong>{{ preset.label }}</strong>
          <small>{{ preset.summary }}</small>
        </div>
        <div class="template-meta" aria-label="Callout best uses">
          <span v-for="item in preset.bestFor" :key="`${preset.id}-${item}`">{{ item }}</span>
        </div>
        <pre>{{ calloutPresetMarkdown(preset) }}</pre>
        <button type="button" :title="`Insert ${preset.label} callout`" @click="insertCalloutPreset(preset)">Insert</button>
      </article>
    </div>
  </section>
  <section class="business-template-hub chart-designer" aria-label="Chart designer">
    <header>
      <div>
        <strong>Chart designer</strong>
        <span>Create board-ready chart blocks from a few fields or from a selected Markdown table.</span>
      </div>
      <button type="button" title="Insert the designed chart block into the active document" @click="insertDesignedChart">Insert chart</button>
    </header>
    <div class="chart-designer-grid">
      <label>
        Chart type
        <select v-model="chartDesignerDraft.kind" @change="resetChartDesignerForType(chartDesignerDraft.kind)">
          <option v-for="kind in chartDesignerKindOptions" :key="kind.id" :value="kind.id">{{ kind.label }}</option>
        </select>
      </label>
      <label>
        Title
        <input v-model="chartDesignerDraft.title" placeholder="Executive pipeline coverage" />
      </label>
      <label>
        Subtitle
        <input v-model="chartDesignerDraft.subtitle" placeholder="Weighted qualified pipeline by segment" />
      </label>
      <label>
        Source note
        <input v-model="chartDesignerDraft.source" placeholder="CRM export, May 2026" />
      </label>
      <label>
        Category field
        <input v-model="chartDesignerDraft.xField" placeholder="Segment" />
      </label>
      <label>
        Value field
        <input v-model="chartDesignerDraft.yField" placeholder="Coverage" />
      </label>
      <label>
        Target value
        <input v-model="chartDesignerDraft.target" placeholder="85" />
      </label>
      <label>
        Target label
        <input v-model="chartDesignerDraft.targetLabel" placeholder="Board plan" />
      </label>
      <label>
        Value suffix
        <input v-model="chartDesignerDraft.valueSuffix" placeholder="%" />
      </label>
      <label><input v-model="chartDesignerDraft.showValues" type="checkbox" /> Show value labels</label>
    </div>
    <label>
      Chart data
      <textarea v-model="chartDesignerDraft.dataText" rows="5" aria-label="Chart designer data" placeholder="Segment, Coverage&#10;Enterprise, 112&#10;Mid-market, 78"></textarea>
    </label>
    <label>
      Palette
      <textarea v-model="chartDesignerDraft.paletteText" rows="3" aria-label="Chart designer palette" placeholder="#2563eb&#10;#16a34a&#10;#f59e0b"></textarea>
    </label>
    <div class="template-actions">
      <button type="button" title="Load sample data for the selected chart type" @click="resetChartDesignerForType(chartDesignerDraft.kind)">Reset sample</button>
      <button type="button" title="Convert the selected or current Markdown table into chart data" @click="loadSelectedTableIntoChartDesigner">Use selected table</button>
      <button type="button" title="Insert the designed chart block into the active document" @click="insertDesignedChart">Insert chart</button>
    </div>
    <details>
      <summary>Generated chart Markdown</summary>
      <pre>{{ chartDesignerPreviewMarkdown }}</pre>
    </details>
  </section>
  <section class="business-template-hub" aria-label="Versioned reusable clauses">
    <header>
      <div>
        <strong>Versioned clauses</strong>
        <span>Insert approved language and detect stale clauses before sending client-facing work.</span>
      </div>
      <small>{{ versionedClauseAuditSummary }}</small>
    </header>
    <div class="versioned-clause-editor" aria-label="Custom versioned clause editor">
      <label>
        Clause label
        <input v-model="customClauseDraft.label" placeholder="Mutual confidentiality" />
      </label>
      <label>
        Kind
        <select v-model="customClauseDraft.kind">
          <option value="identity">Identity</option>
          <option value="proposal">Proposal</option>
          <option value="procurement">Procurement</option>
          <option value="delivery">Delivery</option>
          <option value="governance">Governance</option>
          <option value="review">Review</option>
        </select>
      </label>
      <label>
        Current version
        <input v-model="customClauseDraft.currentVersion" placeholder="2026.05" />
      </label>
      <label>
        Summary
        <input v-model="customClauseDraft.summary" placeholder="Where this approved language should be used" />
      </label>
      <label>
        Stale markers
        <textarea v-model="customClauseStaleMarkersText" rows="2" aria-label="Custom clause stale markers" placeholder="legacy confidentiality clause&#10;clause:confidentiality version=2025"></textarea>
      </label>
      <label>
        Clause Markdown
        <textarea v-model="customClauseDraft.body" rows="5" aria-label="Custom versioned clause Markdown"></textarea>
      </label>
      <div class="template-actions">
        <button type="button" title="Start a new custom clause draft" @click="resetCustomClauseDraft">New clause</button>
        <button type="button" title="Save this custom clause in the workspace library" @click="saveCustomVersionedClause">Save custom clause</button>
        <button v-if="editingCustomClauseId" type="button" title="Delete this custom clause from the workspace library" @click="deleteEditingCustomVersionedClause">Delete custom clause</button>
      </div>
      <p class="sidebar-hint">Custom clauses are profile-aware Markdown parts with explicit version markers. Insert the current version, and NEditor will flag stale markers before external review.</p>
    </div>
    <div class="snippet-list" role="list" aria-label="Approved reusable clauses">
      <article v-for="clause in allVersionedBusinessClauses" :key="clause.id" class="snippet-card" role="listitem">
        <div>
          <strong>{{ clause.label }}</strong>
          <small>{{ clause.kind }} | v{{ clause.currentVersion }} | {{ clause.summary }}{{ store.customVersionedClauses.some((item) => item.id === clause.id) ? " | custom" : "" }}</small>
        </div>
        <button type="button" :title="`Insert ${clause.label} version ${clause.currentVersion}`" @click="insertVersionedClause(clause)">Insert current</button>
        <button v-if="store.customVersionedClauses.some((item) => item.id === clause.id)" type="button" :title="`Edit ${clause.label}`" @click="editCustomVersionedClause(clause)">Edit</button>
      </article>
    </div>
    <div class="snippet-list" role="list" aria-label="Versioned clause audit">
      <article v-for="item in versionedClauseAuditItems" :key="item.id" class="snippet-card" :data-status="item.status" role="listitem">
        <div>
          <strong>{{ item.label }}</strong>
          <small>{{ item.status }} | current v{{ item.currentVersion }} | {{ item.detail }}</small>
        </div>
        <button v-if="item.line" type="button" title="Go to the detected clause marker" @click="goToSourceTarget({ line: item.line })">Go</button>
      </article>
    </div>
  </section>
  <!-- Academic / Science Templates -->
  <h3>Academic &amp; Science templates</h3>
  <section class="academic-templates" aria-label="Academic and science document templates">
    <div v-for="(templates, category) in academicTemplatesByCategory()" :key="String(category)" class="academic-template-group">
      <h4>{{ category }}</h4>
      <div class="academic-template-list">
        <article v-for="tmpl in templates" :key="tmpl.id" class="template-card">
          <header class="template-card-header">
            <div>
              <strong>{{ tmpl.label }}</strong>
              <small>{{ tmpl.description }}</small>
            </div>
          </header>
          <div class="template-card-actions">
            <button type="button" @click="insertMarkdownAtCursor('\n' + tmpl.content + '\n')">Insert</button>
            <button type="button" @click="store.newDocument(); store.updateText(tmpl.content)">New doc</button>
          </div>
        </article>
      </div>
    </div>
  </section>

  <h3>Calculation and transform templates</h3>
  <section class="template-filters" aria-label="Transform template filters">
    <label>
      Search
      <input v-model="templateQuery" placeholder="margin, dose, roadmap" />
    </label>
    <label>
      Category
      <select v-model="templateCategory">
        <option value="all">All</option>
        <option v-for="category in transformTemplateCategoryOptions" :key="category" :value="category">{{ category }}</option>
      </select>
    </label>
    <label>
      Transform
      <select v-model="templateTransform">
        <option value="all">All</option>
        <option v-for="transform in transformTemplateKindOptions" :key="transform" :value="transform">{{ transform }}</option>
      </select>
    </label>
  </section>
  <section class="transform-template-assistance" aria-label="AI transform template assistance">
    <header>
      <div>
        <h3>AI template assistance</h3>
        <span>{{ filteredTransformTemplates.length }} matching templates</span>
      </div>
      <button type="button" @click="appendAllTransformTemplateAssistance">Use all</button>
    </header>
    <p>Context-aware guidance helps choose the right calculation or transform, replace sample values responsibly, preview results, and prepare a review handoff.</p>
    <article
      v-for="item in transformTemplateAssistance"
      :key="item.stepId"
      class="snapshot-row"
      data-status="improve"
    >
      <strong>{{ item.stepLabel }}</strong>
      <p>{{ item.suggestedAnswer }}</p>
      <small>{{ item.rationale }}</small>
      <ul class="signal-list">
        <li v-for="signal in item.contextSignals" :key="`${item.stepId}-${signal}`">{{ signal }}</li>
      </ul>
      <button type="button" @click="appendTransformTemplateAssistance(item)">{{ item.actionLabel }}</button>
    </article>
    <label>
      Transform assistance notes
      <textarea v-model="transformTemplateAssistanceNotes" aria-label="Transform assistance notes" rows="6" placeholder="Accept guidance, record source values, owners, preview findings, and review questions here."></textarea>
    </label>
    <button type="button" @click="insertTransformTemplateAssistanceNotes">Insert transform notes</button>
  </section>
  <section class="template-list" role="list" aria-label="Transform templates">
    <article
      v-for="template in filteredTransformTemplates"
      :key="`${template.source}-${template.id}`"
      class="template-card"
      role="listitem"
    >
      <header class="template-card-header">
        <div>
          <strong>{{ template.name }}</strong>
          <small>{{ template.summary }}</small>
        </div>
        <span class="template-source">{{ template.source }}</span>
      </header>
      <div class="template-meta" aria-label="Template metadata">
        <small class="template-meta-summary">{{ template.category }} | {{ template.transform }} | {{ template.source }}</small>
        <span>{{ template.category }}</span>
        <span>{{ template.transform }}</span>
      </div>
      <div v-if="templateFillFields(template).length" class="template-fill-fields" aria-label="Template fill values">
        <span>Fill</span>
        <code v-for="field in templateFillFields(template)" :key="`${template.id}-${field.name}`" :title="`${field.name} = ${field.value}`">
          {{ field.name }}
        </code>
      </div>
      <div class="template-tags" aria-label="Template tags">
        <small v-for="tag in template.tags" :key="`${template.id}-${tag}`">{{ tag }}</small>
      </div>
      <details>
        <summary>Preview</summary>
        <pre>{{ template.body }}</pre>
      </details>
      <div class="template-actions">
        <button class="template-action-primary" type="button" @click="insertTransformTemplate(template)">
          <span class="button-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" focusable="false">
              <path v-for="path in toolbarIconPaths('templates')" :key="path" :d="path"></path>
            </svg>
          </span>
          Insert
        </button>
        <button type="button" @click="duplicateTransformTemplate(template)">
          <span class="button-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" focusable="false">
              <path v-for="path in toolbarIconPaths('duplicate')" :key="path" :d="path"></path>
            </svg>
          </span>
          Duplicate
        </button>
        <button v-if="template.source === 'custom'" type="button" @click="editCustomTransformTemplate(template)">
          <span class="button-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" focusable="false">
              <path v-for="path in toolbarIconPaths('rename')" :key="path" :d="path"></path>
            </svg>
          </span>
          Edit
        </button>
        <button v-if="template.source === 'custom'" class="danger-action" type="button" @click="store.deleteCustomTransformTemplate(template.id)">
          <span class="button-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" focusable="false">
              <path v-for="path in toolbarIconPaths('close')" :key="path" :d="path"></path>
            </svg>
          </span>
          Delete
        </button>
      </div>
    </article>
  </section>
  <section class="custom-template-editor" aria-label="Custom transform template editor">
    <h3>Custom template</h3>
    <label>
      Name
      <input v-model="customTemplateDraft.name" />
    </label>
    <label>
      Category
      <input v-model="customTemplateDraft.category" />
    </label>
    <label>
      Transform
      <select v-model="customTemplateDraft.transform">
        <option v-for="transform in transformTemplateKindOptions" :key="transform" :value="transform">{{ transform }}</option>
      </select>
    </label>
    <label>
      Summary
      <input v-model="customTemplateDraft.summary" />
    </label>
    <label>
      Tags
      <input v-model="customTemplateTags" placeholder="finance, kpi" />
    </label>
    <label>
      Body
      <textarea v-model="customTemplateDraft.body" rows="10"></textarea>
    </label>
    <div v-if="customTemplateFillFields.length" class="template-fill-fields" aria-label="Detected template fill values">
      <span>Fill</span>
      <code
        v-for="field in customTemplateFillFields"
        :key="`${customTemplateDraft.id}-${field.name}`"
        :title="`${field.name} = ${field.value}`"
      >
        {{ field.name }}
      </code>
    </div>
    <div class="template-actions">
      <button type="button" @click="startNewCustomTemplate">New custom</button>
      <button type="button" :disabled="!customTemplateIsValid" @click="saveCustomTransformTemplate">
        {{ editingCustomTemplateId ? "Save custom" : "Create custom" }}
      </button>
    </div>
  </section>
  <section class="template-pack-manager" aria-label="Template pack marketplace">
    <header>
      <div>
        <h3>Template pack marketplace</h3>
        <span>Package filtered templates into a portable pack with metadata, placeholders, examples, outline rules, and usage guidance.</span>
      </div>
      <button type="button" @click="copyCurrentTemplatePack">Copy pack</button>
    </header>
    <div class="template-pack-fields">
      <label>
        Pack name
        <input v-model="templatePackName" />
      </label>
      <label>
        Publisher
        <input v-model="templatePackPublisher" placeholder="Company or author" />
      </label>
      <label>
        Version
        <input v-model="templatePackVersion" />
      </label>
      <label>
        License
        <input v-model="templatePackLicense" />
      </label>
    </div>
    <label>
      Summary
      <input v-model="templatePackSummary" placeholder="Portable pack for proposals, board papers, or research reports" />
    </label>
    <label>
      Tags
      <input v-model="templatePackTags" placeholder="proposal, board, research" />
    </label>
    <label>
      Usage guidance
      <textarea v-model="templatePackUsageGuidance" rows="4" aria-label="Template pack usage guidance"></textarea>
    </label>
    <div class="template-pack-counts" aria-label="Current template pack contents">
      <span v-for="row in currentTemplatePackRows" :key="row.label">
        <strong>{{ row.value }}</strong>
        {{ row.label }}
      </span>
    </div>
    <details>
      <summary>Preview pack JSON</summary>
      <pre>{{ currentTemplatePackJson }}</pre>
    </details>
    <div class="template-actions">
      <button type="button" @click="insertCurrentTemplatePackManifest">Insert manifest</button>
      <button type="button" @click="copyCurrentTemplatePack">Copy pack JSON</button>
    </div>
    <label>
      Install pasted pack
      <textarea v-model="templatePackImportText" rows="6" aria-label="Template pack import JSON" placeholder="{ &quot;schema&quot;: &quot;neditor.template-pack.v1&quot;, ... }"></textarea>
    </label>
    <div v-if="importedTemplatePack" class="template-pack-counts" aria-label="Imported template pack contents">
      <span v-for="row in importedTemplatePackRows" :key="`import-${row.label}`">
        <strong>{{ row.value }}</strong>
        {{ row.label }}
      </span>
    </div>
    <p v-if="templatePackStatus" class="sidebar-hint">{{ templatePackStatus }}</p>
    <div class="template-actions">
      <button type="button" :disabled="!importedTemplatePack" @click="installPastedTemplatePack">Install reusable items</button>
      <button type="button" :disabled="!importedTemplatePack" @click="insertImportedTemplatePackManifest">Insert imported manifest</button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  academicTemplatesByCategory,
  allVersionedBusinessClauses,
  analyzeCurrentRfpSource,
  appendAllTransformTemplateAssistance,
  appendRfpWizardSuggestion,
  appendTransformTemplateAssistance,
  businessDocumentSnippets,
  businessProfileCompletion,
  businessSnippetQuery,
  businessTemplateQuery,
  businessWizardStepAssistance,
  calloutPresetMarkdown,
  calloutPresets,
  chartDesignerDraft,
  chartDesignerKindOptions,
  chartDesignerPreviewMarkdown,
  copyCurrentTemplatePack,
  createResponsiveRfpResponse,
  createRfpProposalOutline,
  currentTemplatePackJson,
  currentTemplatePackRows,
  customClauseDraft,
  customClauseStaleMarkersText,
  customSnippetDraft,
  customTemplateDraft,
  customTemplateFillFields,
  customTemplateIsValid,
  customTemplateTags,
  deleteEditingCustomBusinessSnippet,
  deleteEditingCustomVersionedClause,
  duplicateTransformTemplate,
  editCustomBusinessSnippet,
  editCustomTransformTemplate,
  editCustomVersionedClause,
  editingCustomClauseId,
  editingCustomSnippetId,
  editingCustomTemplateId,
  filteredBusinessSnippets,
  filteredBusinessTemplates,
  filteredTransformTemplates,
  goToSourceTarget,
  importRfpSourceFile,
  importRfpSourceUrl,
  importedTemplatePack,
  importedTemplatePackRows,
  insertBusinessSnippet,
  insertBusinessTemplate,
  insertCalloutPreset,
  insertCurrentTemplatePackManifest,
  insertDesignedChart,
  insertImportedTemplatePackManifest,
  insertMarkdownAtCursor,
  insertRfpComplianceMatrix,
  insertRfpSubmissionChecklist,
  insertRfpWinThemes,
  insertSelectedCalloutPreset,
  insertTransformTemplate,
  insertTransformTemplateAssistanceNotes,
  insertVersionedClause,
  installPastedTemplatePack,
  loadActiveDocumentAsRfpSource,
  loadSelectedTableIntoChartDesigner,
  openAgentWorkspaceForBusinessTemplate,
  openAgentWorkspaceForRfpAnalysis,
  openBusinessProfile,
  resetChartDesignerForType,
  resetCustomClauseDraft,
  resetCustomSnippetDraft,
  rfpAnalysis,
  rfpAnalysisSummary,
  rfpImportBusy,
  rfpImportMessage,
  rfpResponseContextNotes,
  rfpSourceKind,
  rfpSourceText,
  rfpSourceUrl,
  rfpWizardStepAssistance,
  saveCustomBusinessSnippet,
  saveCustomTransformTemplate,
  saveCustomVersionedClause,
  selectedCalloutPreset,
  selectedCalloutPresetId,
  sendRfpResponseToDocsLive,
  startBusinessDocumentWizard,
  startNewCustomTemplate,
  templateCategory,
  templateFillFields,
  templatePackImportText,
  templatePackLicense,
  templatePackName,
  templatePackPublisher,
  templatePackStatus,
  templatePackSummary,
  templatePackTags,
  templatePackUsageGuidance,
  templatePackVersion,
  templateQuery,
  templateTransform,
  toolbarIconPaths,
  transformTemplateAssistance,
  transformTemplateAssistanceNotes,
  transformTemplateCategoryOptions,
  transformTemplateKindOptions,
  versionedClauseAuditItems,
  versionedClauseAuditSummary,
} = _ctx;
</script>
