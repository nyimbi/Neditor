<template>
  <h2>References</h2>
  <label>
    Citation style
    <select
      :value="citationStyle"
      @change="(e) => { const v = eventValue(e); if (v.endsWith('.csl') || v.startsWith('/')) applyCslFilePath(v); else setCitationStyle(v); }"
    >
      <optgroup label="Built-in styles">
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
      </optgroup>
      <optgroup v-if="installedCslStyles.length" label="Installed CSL files">
        <option v-for="style in installedCslStyles" :key="style.id" :value="style.path">{{ style.title }}</option>
      </optgroup>
    </select>
  </label>
  <p v-if="installedCslStyles.length" class="sidebar-hint">{{ installedCslStyles.length }} CSL file(s) found in ~/.pandoc/csl/. Select one to apply via Pandoc.</p>
  <p v-else class="sidebar-hint">Install .csl files in <code>~/.pandoc/csl/</code> to use custom citation styles via Pandoc.</p>
  <h3>Citations</h3>
  <section class="reference-manager" aria-label="Citation manager">
    <div class="reference-actions">
      <button type="button" @click="insertBlock(bibliographySnippet)">Insert bibliography marker</button>
      <button type="button" @click="insertBlock(bibliographyTemplateSnippet)">Insert BibTeX template</button>
      <button type="button" :disabled="!missingCitationKeys.length" @click="insertMissingCitationStubs">Insert missing key stubs</button>
    </div>
  </section>
  <section class="reference-manager" aria-label="Citation TODO workflow">
    <header>
      <div>
        <strong>Citation TODO Workflow</strong>
        <span>{{ openCitationTodoCount }} open | {{ deferredCitationTodoCount }} deferred</span>
      </div>
    </header>
    <label>
      Source key or citation
      <input v-model="citationTodoKey" placeholder="@source2026 or [@source2026, p. 12]" />
    </label>
    <label>
      Resolution or deferral note
      <input v-model="citationTodoNote" placeholder="Source, page, owner, or deferral reason" />
    </label>
    <div class="reference-actions">
      <button type="button" @click="insertCitationTodo">Add TODO</button>
      <button type="button" :disabled="!citationTodoItems.length" @click="insertCitationTodoAudit">Insert audit</button>
      <button type="button" :disabled="!citationTodoItems.length" @click="copyCitationTodoAudit">Copy audit</button>
    </div>
    <article v-for="todo in citationTodoItems" :key="todo.id" class="snapshot-row" :data-status="todo.status">
      <p>{{ todo.excerpt }}</p>
      <small>Line {{ todo.line }} | {{ todo.status }}{{ todo.note ? ` | ${todo.note}` : "" }}</small>
      <div class="reference-actions">
        <button type="button" @click="goToCitationTodo(todo)">Go to TODO</button>
        <button type="button" :disabled="!citationTodoKey.trim()" @click="resolveCitationTodoItem(todo)">Resolve</button>
        <button type="button" @click="deferCitationTodoItem(todo)">Defer</button>
      </div>
    </article>
    <p v-if="!citationTodoItems.length" class="sidebar-hint">No citation TODOs detected.</p>
  </section>
  <section class="reference-manager" aria-label="Citation source search and deep research">
    <header>
      <div>
        <strong>Source Search & Deep Research</strong>
        <span>Search, download source documents, and draft sourced reports with Ollama or another configured provider.</span>
      </div>
    </header>
    <label>
      Search provider
      <select v-model="citationSearchProvider">
        <option value="duckduckgo">DuckDuckGo</option>
        <option value="searxng">SearXNG</option>
        <option value="tavily">Tavily</option>
        <option value="local-library">Local source library</option>
      </select>
    </label>
    <p v-if="citationSearchProvider === 'local-library'" class="sidebar-hint">
      Searches the saved source library associated with this document. Save the document and download sources first to build a reusable local research base.
    </p>
    <label v-if="citationSearchProvider === 'searxng'">
      SearXNG URL
      <input v-model="citationSearxngUrl" placeholder="http://127.0.0.1:8080" />
    </label>
    <label v-if="citationSearchProvider === 'tavily'">
      Tavily session key
      <input v-model="citationTavilyApiKey" type="password" autocomplete="off" placeholder="TAVILY_API_KEY or session key" />
    </label>
    <label>
      Citation/source query
      <input v-model="citationSearchQuery" type="search" placeholder="market sizing public sector ERP Kenya 2026" />
    </label>
    <div class="reference-actions">
      <button type="button" :disabled="citationSearchBusy || !citationSearchQuery.trim()" @click="searchCitationSources()">
        {{ citationSearchBusy ? "Searching..." : "Find sources" }}
      </button>
      <button
        type="button"
        :disabled="citationSourceLibraryBusy || !active.path"
        title="Refresh the saved source manifest for this document."
        @click="refreshCitationSourceLibrary()"
      >
        {{ citationSourceLibraryBusy ? "Refreshing..." : "Refresh source library" }}
      </button>
      <button
        type="button"
        :disabled="citationSourceBulkBusy || citationSearchProvider === 'local-library' || !active.path || !citationSearchResults.length"
        title="Save every visible search result into this document's local source library."
        @click="downloadAllCitationSources()"
      >
        {{ citationSearchProvider === "local-library" ? "Already local" : citationSourceBulkBusy ? "Saving sources..." : "Save all found sources" }}
      </button>
    </div>
    <p v-if="citationSourceLibraryDir" class="sidebar-hint">Saved source library: {{ citationSourceLibraryDir }}</p>
    <article v-for="source in citationSearchResults" :key="source.url" class="snapshot-row">
      <p>{{ source.title }}</p>
      <small>{{ source.source }}<template v-if="source.fitScore !== undefined"> | fit {{ source.fitScore }}/100 {{ source.fitLabel }}</template> | {{ source.url }}</small>
      <small v-if="source.fitReasons?.length">{{ source.fitReasons.join(" | ") }}</small>
      <small v-if="source.snippet">{{ source.snippet }}</small>
      <div class="reference-actions">
        <button type="button" :disabled="citationSourceBusyUrl === source.url || !active.path" @click="downloadCitationSource(source)">
          {{ citationSourceBusyUrl === source.url ? "Downloading..." : citationSearchProvider === "local-library" ? "Re-download source" : "Download source" }}
        </button>
        <button type="button" @click="insertBlock(`[${source.title}](${source.url})`)">Insert link</button>
      </div>
    </article>
    <div v-if="citationSourceLibrary.length" class="reference-manager" aria-label="Downloaded citation source library">
      <header>
        <div>
          <strong>Downloaded Source Library</strong>
          <span>{{ citationSourceLibrary.length }} saved source{{ citationSourceLibrary.length === 1 ? "" : "s" }} for this document</span>
        </div>
        <div class="reference-actions">
          <button type="button" @click="insertCitationSourceLibraryAudit">Insert audit</button>
          <button type="button" @click="copyCitationSourceLibraryAudit">Copy audit</button>
        </div>
      </header>
      <article v-for="source in citationSourceLibrary" :key="`${source.citation_key}-${source.sha256}`" class="snapshot-row">
        <p>{{ source.title }}</p>
        <small>
          @{{ source.citation_key }} | {{ source.source || "saved source" }}<template v-if="source.fit_score !== undefined"> | fit {{ source.fit_score }}/100 {{ source.fit_label }}</template> | {{ source.media_type || "source file" }} | {{ formatCitationSourceBytes(source.bytes) }}
        </small>
        <small v-if="source.file_exists === false" class="source-integrity-warning">
          Local file missing; re-download before review, export, or evidence handoff.
        </small>
        <small v-else-if="source.hash_matches === false" class="source-integrity-warning">
          Local file changed after download; re-download or verify the modified evidence before review.
        </small>
        <small v-if="source.fit_reasons?.length">{{ source.fit_reasons.join(" | ") }}</small>
        <small>{{ source.relative_path }}</small>
        <small>{{ source.url }}</small>
        <div class="reference-actions">
          <button type="button" @click="insertCitationSourceReference(source)">Cite</button>
          <button type="button" @click="insertCitationSourceBibliography(source)">Insert bibliography</button>
          <button type="button" @click="insertBlock(`[${source.title}](${source.relative_path})`)">Insert local link</button>
          <button v-if="citationSourceNeedsRecovery(source)" type="button" :disabled="citationSourceBusyUrl === source.url" @click="redownloadCitationSource(source)">
            {{ citationSourceBusyUrl === source.url ? "Re-downloading..." : "Re-download" }}
          </button>
          <button type="button" @click="copyCitationSourcePath(source)">Copy path</button>
          <button type="button" :disabled="source.file_exists === false" @click="revealCitationSource(source)">Reveal file</button>
        </div>
      </article>
    </div>
    <div class="reference-inline-form">
      <label>
        Deep research topic
        <input v-model="deepResearchTopic" placeholder="Topic for a sourced report" />
      </label>
      <label>
        Document type
        <input v-model="deepResearchDocumentType" placeholder="research brief, market report, policy memo" />
      </label>
      <label>
        Audience
        <input v-model="deepResearchAudience" placeholder="board, client, technical reviewers" />
      </label>
    </div>
    <label class="deep-research-length-control">
      <span>Report length: {{ deepResearchTargetPages }} page{{ deepResearchTargetPages === 1 ? "" : "s" }}</span>
      <input
        v-model.number="deepResearchTargetPages"
        aria-label="Deep research target report pages"
        type="range"
        min="1"
        max="200"
        step="1"
        @change="setDeepResearchTargetPages(deepResearchTargetPages)"
      />
      <input
        v-model.number="deepResearchTargetPages"
        aria-label="Exact deep research target pages"
        type="number"
        min="1"
        max="200"
        step="1"
        @change="setDeepResearchTargetPages(deepResearchTargetPages)"
      />
      <small>{{ deepResearchLengthSummary }}</small>
    </label>
    <div class="reference-inline-form">
      <label>
        Research loops
        <input v-model.number="deepResearchIterationsTarget" type="number" min="1" max="5" />
      </label>
      <label>
        Results per loop
        <input v-model.number="deepResearchResultsPerIteration" type="number" min="3" max="12" />
      </label>
    </div>
    <label class="reference-checkbox">
      <input v-model="deepResearchSaveSources" :disabled="!active.path" type="checkbox" />
      Save research source documents to this document's source library
    </label>
    <p v-if="deepResearchSaveSources && !active.path" class="sidebar-hint">Save the document first to preserve Deep Research sources locally.</p>
    <p v-if="deepResearchSavedSourceCount" class="sidebar-hint">
      Saved or reused {{ deepResearchSavedSourceCount }} Deep Research source{{ deepResearchSavedSourceCount === 1 ? "" : "s" }} in the local library.
    </p>
    <div class="reference-actions">
      <button type="button" :disabled="deepResearchBusy || !deepResearchTopic.trim()" @click="runDeepResearchDocumentCreation">
        {{ deepResearchBusy ? "Researching..." : "Create deep research draft" }}
      </button>
      <button type="button" :disabled="!deepResearchDraft" @click="insertDeepResearchDraft">Insert draft</button>
      <button type="button" :disabled="!deepResearchDraft" @click="openDeepResearchDraftAsDocument">Open as document</button>
      <button type="button" :disabled="!deepResearchIterations.length" @click="insertDeepResearchLog">Insert research log</button>
      <button type="button" :disabled="!deepResearchIterations.length" @click="insertDeepResearchSourceQualityReview">Insert source quality</button>
      <button type="button" :disabled="!deepResearchIterations.length" @click="insertDeepResearchConflictReview">Insert conflict review</button>
      <button type="button" :disabled="!deepResearchIterations.length" @click="insertDeepResearchAuditPacket">Insert audit packet</button>
    </div>
    <p class="sidebar-hint">
      {{ deepResearchStatus || "Deep research plans queries, searches, reflects on gaps, writes, and iterates expansion passes until it reaches the requested page count or the provider stops adding useful length." }}
    </p>
    <section v-if="deepResearchIterations.length" class="snapshot-row" aria-label="Deep research source quality review">
      <p>{{ deepResearchSourceQualitySummary }}</p>
      <small v-for="item in deepResearchSourceQualityItems.slice(0, 6)" :key="`${item.iteration}-${item.url}`">
        {{ item.fitLabel }} | {{ item.fitScore }}/100 | {{ item.title }} | {{ _formatQualityDimensions(item.qualityDimensions) }} | {{ item.reviewAction }}
      </small>
    </section>
    <section v-if="deepResearchIterations.length" class="snapshot-row" aria-label="Deep research evidence conflict review">
      <p>{{ deepResearchConflictSummary }}</p>
      <small v-for="conflict in deepResearchEvidenceConflicts.slice(0, 4)" :key="conflict.id">
        {{ conflict.id }} | {{ conflict.severity }} | {{ conflict.topic }} | {{ conflict.signals.join(" versus ") }}
      </small>
    </section>
    <textarea v-if="deepResearchDraft" :value="deepResearchDraft" rows="8" readonly aria-label="Deep research draft preview"></textarea>
  </section>
  <button
    v-for="citation in active.compile?.semantic.citation_references || []"
    :key="`${citation.key}-${citation.line}-${citation.column}`"
    class="outline-row"
    type="button"
    @click="goToSourceTarget(citation)"
  >
    <span>[@{{ citation.key }}<template v-if="citation.locator">, {{ citation.locator }}</template>]</span>
    <small>{{ bibliographyByKey.get(citation.key) || "Missing bibliography entry" }}</small>
  </button>
  <h3>Table of Contents</h3>
  <section class="reference-manager" aria-label="Table of contents manager">
    <div class="reference-actions">
      <button type="button" @click="insertBlock(tocSnippet)">Insert TOC marker</button>
      <button type="button" @click="enableFrontMatterToc">Enable front matter TOC</button>
    </div>
    <div class="reference-inline-form">
      <label>
        TOC depth
        <select v-model.number="tocDepthDraft">
          <option v-for="depth in tocDepthOptions" :key="depth" :value="depth">H1-H{{ depth }}</option>
        </select>
      </label>
      <button type="button" @click="applyTocSettings">Apply TOC settings</button>
    </div>
    <label class="reference-checkbox">
      <input v-model="tocNumberedDraft" type="checkbox" />
      Number TOC entries
    </label>
    <p class="sidebar-hint">{{ tocManagerSummary }}</p>
  </section>
  <h3>Local data sources</h3>
  <section class="reference-manager" aria-label="Local data source manager">
    <p class="sidebar-hint">{{ dataSourceManagerSummary }}</p>
    <div class="reference-inline-form">
      <label>
        Source name
        <input v-model="dataSourceNameDraft" placeholder="Revenue, Accounts, Settings" />
      </label>
      <label>
        File path
        <input v-model="dataSourcePathDraft" placeholder="data/revenue.csv" />
      </label>
      <button type="button" @click="chooseFrontMatterDataSourceFile">Choose file</button>
      <button type="button" :disabled="!dataSourcePathDraft.trim() || dataSourceCopyBusy" @click="copyFrontMatterDataSourceFile">
        {{ dataSourceCopyBusy ? "Copying..." : "Copy to data folder" }}
      </button>
      <label>
        Type
        <select v-model="dataSourceTypeDraft" aria-label="Data source type">
          <option v-for="type in dataSourceTypeOptions" :key="type" :value="type">{{ type.toUpperCase() }}</option>
        </select>
      </label>
      <label v-if="dataSourceTypeDraft === 'xlsx'">
        Worksheet
        <input v-model="dataSourceSheetNameDraft" placeholder="Pipeline Forecast" />
      </label>
      <label v-if="dataSourceTypeDraft === 'xlsx'">
        Sheet index
        <input v-model="dataSourceSheetIndexDraft" inputmode="numeric" placeholder="2" />
      </label>
      <button type="button" :disabled="!dataSourcePathDraft.trim()" @click="addFrontMatterDataSource">Add data source</button>
    </div>
    <div class="reference-actions">
      <button type="button" @click="insertDataSourceTemplate">Insert data source template</button>
    </div>
    <section class="data-refresh-workflow" aria-label="Data refresh workflow">
      <header>
        <strong>Data refresh workflow</strong>
        <span>{{ dataRefreshWorkflowSummary }}</span>
      </header>
      <div class="reference-actions">
        <button type="button" :disabled="!frontMatterDataSourceRows.length || store.compileBusy" @click="refreshDataSourcesPreview">
          {{ store.compileBusy ? "Refreshing..." : "Refresh preview imports" }}
        </button>
        <button type="button" :disabled="!frontMatterDataSourceRows.length" @click="insertDataRefreshAudit">Insert refresh audit</button>
        <button type="button" :disabled="!frontMatterDataSourceRows.length || store.compileBusy" @click="refreshDataSourcesAndInsertAudit">Refresh and audit</button>
      </div>
      <article v-for="row in dataRefreshPlan.rows" :key="row.id" class="snapshot-row" :data-status="row.status">
        <p>{{ row.source.name || row.source.path || "Unnamed data source" }}</p>
        <small>{{ row.label }} | {{ row.verification }}</small>
        <small v-for="item in row.evidence" :key="item">{{ item }}</small>
        <div class="reference-actions">
          <button v-if="row.source.line" type="button" @click="goToSourceTarget({ line: row.source.line })">Go to source</button>
          <button v-if="row.importableTable" type="button" :disabled="tableDataBusy" @click="importDataSourceAsEditableTable(row.source)">Import as editable table</button>
        </div>
      </article>
      <p v-if="!dataRefreshPlan.rows.length" class="sidebar-hint">Declare local data sources first, then refresh preview imports and record an audit before distribution.</p>
    </section>
    <article v-for="source in frontMatterDataSourceRows" :key="source.id" class="snapshot-row" :data-status="source.status">
      <p>{{ source.name || source.path || "Unnamed data source" }}</p>
      <small>{{ source.kind.toUpperCase() }} | {{ source.status }} | {{ source.source }}{{ source.line ? ` | line ${source.line}` : "" }}</small>
      <small v-if="source.kind === 'xlsx' && (source.sheetName || source.sheetIndex)">
        Worksheet: {{ source.sheetName || `#${source.sheetIndex}` }}
      </small>
      <small v-if="source.path">{{ source.path }}</small>
      <small v-if="source.detail">{{ source.detail }}</small>
      <div class="reference-actions">
        <button v-if="source.line" type="button" @click="goToSourceTarget({ line: source.line })">Go to source</button>
      </div>
    </article>
    <p v-if="!frontMatterDataSourceRows.length" class="sidebar-hint">No local CSV, TSV, JSON, YAML, or XLSX data sources declared in front matter.</p>
  </section>
  <h3>Document variables</h3>
  <section class="reference-manager" aria-label="Document variable manager">
    <p class="sidebar-hint">{{ documentVariableManagerSummary }}</p>
    <div class="reference-inline-form">
      <label>
        Variable name
        <input v-model="documentVariableNameDraft" placeholder="client, owner, budget" />
      </label>
      <label>
        Value
        <input v-model="documentVariableValueDraft" placeholder="Example Corp, Strategy Office, 125000" />
      </label>
      <button type="button" :disabled="!documentVariableNameDraft.trim()" @click="addDocumentVariable">Add variable</button>
    </div>
    <label>
      Insert filter
      <select v-model="documentVariableFilterDraft" aria-label="Document variable insert filter">
        <option v-for="filter in documentVariableFilterOptions" :key="filter.value" :value="filter.value">{{ filter.label }}</option>
      </select>
    </label>
    <article v-for="variable in frontMatterVariableRows" :key="variable.key" class="snapshot-row" :data-status="variable.status">
      <p>{{ variable.key }}</p>
      <small>{{ variable.status }} | {{ variable.value || "empty" }}{{ variable.line ? ` | line ${variable.line}` : "" }}</small>
      <div class="reference-actions">
        <button type="button" @click="insertDocumentVariable(variable.key)">Insert variable</button>
        <button type="button" @click="goToSourceTarget({ line: variable.line })">Go to variable</button>
      </div>
    </article>
    <article v-for="variable in mergedMetadataVariableRows" :key="variable.key" class="snapshot-row" :data-status="variable.status">
      <p>{{ variable.key }}</p>
      <small>{{ variable.status }} | project/merged metadata | {{ variable.value || "empty" }}</small>
      <div class="reference-actions">
        <button type="button" @click="insertDocumentVariable(variable.key)">Insert variable</button>
      </div>
    </article>
    <p v-if="!frontMatterVariableRows.length && !mergedMetadataVariableRows.length" class="sidebar-hint">
      No scalar front matter or merged project variables are available for placeholder insertion.
    </p>
  </section>
  <h3>Captions and Lists</h3>
  <section class="reference-manager" aria-label="Captions and generated lists manager">
    <div class="reference-actions">
      <button type="button" @click="insertBlock(listOfFiguresSnippet)">Insert list of figures</button>
      <button type="button" @click="insertBlock(listOfTablesSnippet)">Insert list of tables</button>
    </div>
    <p class="sidebar-hint">{{ captionManagerSummary }}</p>
    <article v-for="item in captionedReferenceItems" :key="`${item.kind}-${item.line}-${item.label}`" class="snapshot-row" :data-status="item.status">
      <p>{{ item.label }}</p>
      <small>{{ item.kind }} | {{ item.status }} | line {{ item.line }}</small>
      <div class="reference-actions">
        <button type="button" @click="goToSourceTarget(item)">Go to source</button>
        <button v-if="item.id" type="button" @click="insertBlock(`See {@${item.id}}.`)">Insert reference</button>
      </div>
    </article>
    <p v-if="!captionedReferenceItems.length" class="sidebar-hint">No tables, figures, or equations detected for generated lists.</p>
  </section>
  <template v-if="resolvedCitationEntries.length">
    <h3>Resolved references</h3>
    <article v-for="entry in resolvedCitationEntries" :key="entry.key" class="snapshot-row">
      <p>@{{ entry.key }}</p>
      <small>{{ entry.title }}</small>
      <small>{{ [entry.author, entry.issued].filter(Boolean).join(" | ") }}</small>
      <div class="reference-actions">
        <button type="button" @click="insertCitationReference(entry.key)">Cite again</button>
        <button type="button" @click="insertBlock(bibliographyEntryStub(entry))">Insert entry copy</button>
      </div>
    </article>
  </template>
  <template v-if="missingCitationKeys.length">
    <h3>Missing keys</h3>
    <article v-for="key in missingCitationKeys" :key="key" class="snapshot-row">
      <p class="error">@{{ key }}</p>
      <div class="reference-actions">
        <button type="button" @click="insertBlock(bibliographyEntryStub({ key }))">Insert stub</button>
        <button type="button" @click="insertCitationReference(key)">Cite again</button>
      </div>
    </article>
  </template>
  <template v-if="active.compile?.semantic.duplicate_bibliography_keys.length">
    <h3>Duplicate keys</h3>
    <article v-for="(entry, index) in duplicateBibliographyEntries" :key="`${entry.key}-${entry.line || index}`" class="snapshot-row">
      <button class="outline-row" type="button" @click="goToSourceTarget(entry)">
        @{{ entry.key }}
      </button>
      <small>{{ entry.locationLabel }}</small>
      <small>{{ entry.title }}</small>
    </article>
  </template>
  <h3>Glossary</h3>
  <section class="reference-manager" aria-label="Glossary manager">
    <p class="sidebar-hint">{{ glossaryManagerSummary }}</p>
    <div class="reference-actions">
      <button type="button" @click="insertBlock(glossarySectionSnippet)">Insert generated glossary</button>
      <button type="button" @click="insertBlock(glossarySnippet)">Insert glossary definitions</button>
      <button type="button" @click="store.exportDefaults.includeGlossary = true">Include glossary in exports</button>
      <button type="button" @click="insertGlossaryAuditTable">Insert glossary audit</button>
    </div>
    <p v-if="!glossaryEntries.length" class="sidebar-hint">No glossary terms detected.</p>
    <article v-for="entry in glossaryEntries" :key="entry.term" class="snapshot-row">
      <p>{{ entry.term }}</p>
      <small>{{ entry.definition }}</small>
      <div class="reference-actions">
        <button type="button" @click="goToSearchTerm(entry.term)">Find term</button>
        <button type="button" :aria-label="`Add ${entry.term} to index`" @click="insertIndexMarkerForTerm(entry.term)">Add to index</button>
      </div>
    </article>
  </section>
  <h3>Index</h3>
  <section class="reference-manager" aria-label="Index manager">
    <p class="sidebar-hint">{{ indexManagerSummary }}</p>
    <div class="reference-actions">
      <button type="button" @click="insertBlock(indexSnippet)">Insert generated index</button>
      <button type="button" @click="setFrontMatterField('index', 'true')">Enable front matter index</button>
      <button type="button" @click="insertIndexAuditTable">Insert index audit</button>
    </div>
    <div class="reference-inline-form">
      <label>
        Add index term
        <input v-model="indexTermDraft" placeholder="Liquidity, Working Capital, Client Name" />
      </label>
      <button type="button" :disabled="!indexTermDraft.trim()" @click="insertIndexMarkerFromDraft">Add marker</button>
    </div>
    <div class="reference-inline-form">
      <label>
        Exclude term
        <input v-model="indexExcludeDraft" placeholder="Internal Draft, Secret Plan" />
      </label>
      <button type="button" :disabled="!indexExcludeDraft.trim()" @click="addIndexExclusion">Exclude term</button>
    </div>
    <section v-if="indexExclusionTerms.length" class="reference-chip-list" aria-label="Index exclusions">
      <span v-for="term in indexExclusionTerms" :key="term">
        {{ term }}
        <button type="button" :aria-label="`Remove ${term} from index exclusions`" @click="removeIndexExclusion(term)">Remove</button>
      </span>
    </section>
    <p v-if="!indexTerms.length" class="sidebar-hint">No index terms detected.</p>
    <button v-for="term in indexTerms" :key="term" class="outline-row" type="button" @click="goToSearchTerm(term)">
      {{ term }}
    </button>
  </section>
  <h3>Tables</h3>
  <article v-for="table in active.compile?.semantic.table_summaries || []" :key="table.line" class="snapshot-row">
    <p>{{ table.rows }} rows | {{ table.columns.join(", ") }}</p>
    <small v-for="(total, column) in table.numeric_columns" :key="column">{{ column }} total: {{ total }} </small>
  </article>
  <h3>Figures</h3>
  <article v-for="figure in figureBlocks" :key="`${figure.id || figure.src}-${figure.line}`" class="snapshot-row">
    <p>{{ figure.caption || figure.alt || figure.id || figure.src || "Figure" }}</p>
    <small>{{ figure.fit || "default" }} | {{ figure.position || "center" }}</small>
    <button type="button" @click="goToSourceTarget(figure)">Go to source</button>
    <label>
      Crop focus
      <select :value="figure.position || 'center'" :disabled="!canEditFigureSource(figure)" @change="onFigureCropPositionChange(figure, $event)">
        <option v-for="position in figureCropPositions" :key="position" :value="position">{{ position }}</option>
      </select>
    </label>
    <div
      class="crop-focus-pad"
      :class="{ disabled: !canEditFigureSource(figure) }"
      :style="figureCropPreviewStyle(figure)"
      :data-position="figure.position || 'center'"
      role="slider"
      tabindex="0"
      aria-label="Crop focus"
      :aria-valuetext="figure.position || 'center'"
      :aria-disabled="!canEditFigureSource(figure)"
      @pointerdown.prevent="onFigureCropPointerDown(figure, $event)"
      @pointermove.prevent="onFigureCropPointerMove(figure, $event)"
      @keydown="onFigureCropKeydown(figure, $event)"
    >
      <span v-for="position in figureCropPositions" :key="position" class="crop-focus-point" :style="figureCropPointStyle(position)"></span>
      <span class="crop-focus-reticle" :style="figureCropReticleStyle(normalizeFigureCropPosition(figure.position))"></span>
    </div>
  </article>
  <h3>Formula graph</h3>
  <article v-for="formula in active.compile?.formula_graph || []" :key="formula.name" class="snapshot-row">
    <p>{{ formula.name }} = {{ formula.expression }}</p>
    <small>{{ formula.error || (formula.value ?? "unresolved") }}</small>
    <small v-if="formula.dependencies.length">depends on {{ formula.dependencies.join(", ") }}</small>
  </article>
  <p v-for="edge in active.compile?.formula_dependency_edges || []" :key="`${edge.from}-${edge.to}`">
    {{ edge.from }} -> {{ edge.to }}
  </p>
  <h3>Includes</h3>
  <section class="reference-manager include-builder" aria-label="Include document builder">
    <p class="sidebar-hint">Insert another Markdown file into this document. Include paths resolve relative to the saved parent document.</p>
    <label>
      Child document path
      <input v-model="includeTargetDraft" type="text" placeholder="chapters/introduction.md" aria-label="Included document path" />
    </label>
    <label>
      Include syntax
      <select v-model="includeSyntaxDraft" aria-label="Include directive syntax">
        <option v-for="option in includeDirectiveSyntaxOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </label>
    <p class="sidebar-hint">{{ includeDirectiveSyntaxHelp }}</p>
    <code class="include-directive-preview">{{ includeDirectivePreview || "Enter a child document path" }}</code>
    <p class="sidebar-hint">{{ includeChildCreateHelp }}</p>
    <div class="include-actions">
      <button type="button" :disabled="!includeDirectivePreview" @click="insertIncludeDirectiveFromBuilder">Insert include</button>
      <button type="button" :disabled="includeChildCreateBusy || Boolean(includeChildPathResolution.error)" @click="createIncludeChildDocument">
        {{ includeChildCreateBusy ? "Creating..." : "Create child document" }}
      </button>
    </div>
  </section>
  <p v-if="!includeGraphItems.length" class="sidebar-hint">No included files in this document.</p>
  <section v-else class="include-graph" aria-label="Include graph">
    <article
      v-for="edge in includeGraphItems"
      :key="`${edge.parent}-${edge.child}`"
      class="include-edge"
      :style="{ marginLeft: `${Math.max(0, edge.depth - 1) * 12}px` }"
    >
      <small>Depth {{ edge.depth }}</small>
      <p>
        <span>{{ edge.parentLabel }}</span>
        <span aria-hidden="true"> -&gt; </span>
        <strong>{{ edge.childLabel }}</strong>
      </p>
      <div class="include-actions">
        <button type="button" :aria-label="`Open include ${edge.child}`" @click="openIncludeChild(edge)">Open include</button>
        <button type="button" :aria-label="`Go to include directive for ${edge.child}`" @click="goToIncludeDirective(edge)">Go to directive</button>
      </div>
    </article>
  </section>
  <h3>Cross references</h3>
  <section class="reference-manager" aria-label="Cross reference manager">
    <p class="sidebar-hint">{{ crossReferenceManagerSummary }}</p>
    <article v-for="reference in crossReferenceRows" :key="`${reference.key}-${reference.line}-${reference.column}`" class="snapshot-row" :data-status="reference.resolved ? 'ready' : 'missing'">
      <p>{{ reference.key }}</p>
      <small>{{ reference.target_kind }} | {{ reference.resolved ? "resolved" : "missing" }} | line {{ reference.line }}</small>
      <div class="reference-actions">
        <button type="button" @click="goToCrossReference(reference)">Go to reference</button>
        <button type="button" @click="insertCrossReferenceForLabel(reference.key)">Insert another</button>
      </div>
    </article>
    <p v-if="!crossReferenceRows.length" class="sidebar-hint">No cross references detected.</p>
  </section>
  <h3>Labels</h3>
  <section class="reference-manager" aria-label="Reference label inventory">
    <p class="sidebar-hint">{{ referenceLabelManagerSummary }}</p>
    <article v-for="label in referenceLabelRows" :key="`${label.kind}-${label.key}`" class="snapshot-row">
      <p>{{ label.key }}</p>
      <small>{{ label.kind }} | {{ label.title }}{{ label.line ? ` | line ${label.line}` : "" }}</small>
      <div class="reference-actions">
        <button type="button" @click="goToReferenceLabel(label)">Go to label</button>
        <button type="button" @click="insertCrossReferenceForLabel(label.key)">Insert reference</button>
      </div>
    </article>
    <p v-if="!referenceLabelRows.length" class="sidebar-hint">No labels detected.</p>
  </section>
</template>

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  active,
  addDocumentVariable,
  addFrontMatterDataSource,
  addIndexExclusion,
  applyCslFilePath,
  applyTocSettings,
  bibliographyByKey,
  bibliographyEntryStub,
  bibliographySnippet,
  bibliographyTemplateSnippet,
  canEditFigureSource,
  captionManagerSummary,
  captionedReferenceItems,
  chooseFrontMatterDataSourceFile,
  citationSearchBusy,
  citationSearchProvider,
  citationSearchQuery,
  citationSearchResults,
  citationSearxngUrl,
  citationSourceBulkBusy,
  citationSourceBusyUrl,
  citationSourceLibrary,
  citationSourceLibraryBusy,
  citationSourceLibraryDir,
  citationSourceNeedsRecovery,
  citationStyle,
  citationTavilyApiKey,
  citationTodoItems,
  citationTodoKey,
  citationTodoNote,
  copyCitationSourceLibraryAudit,
  copyCitationSourcePath,
  copyCitationTodoAudit,
  copyFrontMatterDataSourceFile,
  createIncludeChildDocument,
  crossReferenceManagerSummary,
  crossReferenceRows,
  dataRefreshPlan,
  dataRefreshWorkflowSummary,
  dataSourceCopyBusy,
  dataSourceManagerSummary,
  dataSourceNameDraft,
  dataSourcePathDraft,
  dataSourceSheetIndexDraft,
  dataSourceSheetNameDraft,
  dataSourceTypeDraft,
  dataSourceTypeOptions,
  deepResearchAudience,
  deepResearchBusy,
  deepResearchConflictSummary,
  deepResearchDocumentType,
  deepResearchDraft,
  deepResearchEvidenceConflicts,
  deepResearchIterations,
  deepResearchIterationsTarget,
  deepResearchLengthSummary,
  deepResearchResultsPerIteration,
  deepResearchSaveSources,
  deepResearchSavedSourceCount,
  deepResearchSourceQualityItems,
  deepResearchSourceQualitySummary,
  deepResearchStatus,
  deepResearchTargetPages,
  deepResearchTopic,
  deferCitationTodoItem,
  deferredCitationTodoCount,
  documentVariableFilterDraft,
  documentVariableFilterOptions,
  documentVariableManagerSummary,
  documentVariableNameDraft,
  documentVariableValueDraft,
  downloadAllCitationSources,
  downloadCitationSource,
  duplicateBibliographyEntries,
  enableFrontMatterToc,
  eventValue,
  figureBlocks,
  figureCropPointStyle,
  figureCropPositions,
  figureCropPreviewStyle,
  figureCropReticleStyle,
  formatCitationSourceBytes,
  frontMatterDataSourceRows,
  frontMatterVariableRows,
  glossaryEntries,
  glossaryManagerSummary,
  glossarySectionSnippet,
  glossarySnippet,
  goToCitationTodo,
  goToCrossReference,
  goToIncludeDirective,
  goToReferenceLabel,
  goToSearchTerm,
  goToSourceTarget,
  importDataSourceAsEditableTable,
  includeChildCreateBusy,
  includeChildCreateHelp,
  includeChildPathResolution,
  includeDirectivePreview,
  includeDirectiveSyntaxHelp,
  includeDirectiveSyntaxOptions,
  includeGraphItems,
  includeSyntaxDraft,
  includeTargetDraft,
  indexExcludeDraft,
  indexExclusionTerms,
  indexManagerSummary,
  indexSnippet,
  indexTermDraft,
  indexTerms,
  insertBlock,
  insertCitationReference,
  insertCitationSourceBibliography,
  insertCitationSourceLibraryAudit,
  insertCitationSourceReference,
  insertCitationTodo,
  insertCitationTodoAudit,
  insertCrossReferenceForLabel,
  insertDataRefreshAudit,
  insertDataSourceTemplate,
  insertDeepResearchAuditPacket,
  insertDeepResearchConflictReview,
  insertDeepResearchDraft,
  insertDeepResearchLog,
  insertDeepResearchSourceQualityReview,
  insertDocumentVariable,
  insertGlossaryAuditTable,
  insertIncludeDirectiveFromBuilder,
  insertIndexAuditTable,
  insertIndexMarkerForTerm,
  insertIndexMarkerFromDraft,
  insertMissingCitationStubs,
  installedCslStyles,
  listOfFiguresSnippet,
  listOfTablesSnippet,
  mergedMetadataVariableRows,
  missingCitationKeys,
  normalizeFigureCropPosition,
  onFigureCropKeydown,
  onFigureCropPointerDown,
  onFigureCropPointerMove,
  onFigureCropPositionChange,
  openCitationTodoCount,
  openDeepResearchDraftAsDocument,
  openIncludeChild,
  redownloadCitationSource,
  referenceLabelManagerSummary,
  referenceLabelRows,
  refreshCitationSourceLibrary,
  refreshDataSourcesAndInsertAudit,
  refreshDataSourcesPreview,
  removeIndexExclusion,
  resolveCitationTodoItem,
  resolvedCitationEntries,
  revealCitationSource,
  runDeepResearchDocumentCreation,
  searchCitationSources,
  setCitationStyle,
  setDeepResearchTargetPages,
  setFrontMatterField,
  tableDataBusy,
  tocDepthDraft,
  tocDepthOptions,
  tocManagerSummary,
  tocNumberedDraft,
  tocSnippet,
} = _ctx;

function _formatQualityDimensions(dims: { name: string; score: number }[]): string {
  return dims.map(d => `${d.name} ${d.score}`).join(' / ');
}
</script>
