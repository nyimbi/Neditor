<template>
  <div
    class="app-shell"
    :data-theme="store.theme"
    :data-high-contrast="store.highContrast ? 'true' : 'false'"
    :data-reduced-motion="store.reducedMotion ? 'true' : 'false'"
  >
    <header class="titlebar">
      <section class="document-tabs" aria-label="Open documents">
        <section
          v-for="group in groupedDocuments"
          :key="group.key"
          class="tab-group"
          :aria-label="`${group.label} tabs`"
          @dragover.prevent
          @drop="dropTabOnGroup(group)"
        >
          <header class="tab-group-header" :title="group.title">
            <span>{{ group.label }}</span>
            <small>{{ group.documents.length }}</small>
            <button type="button" aria-label="Close tab group" @click="closeTabGroup(group)">x</button>
          </header>
          <div
            v-for="document in group.documents"
            :key="document.id"
            class="tab"
            :class="{ active: document.id === store.activeId }"
            draggable="true"
            @dragstart="draggedTabId = document.id"
            @dragend="draggedTabId = ''"
          >
            <button class="tab-main" type="button" @click="activate(document.id)">
              <span>{{ document.dirty ? "*" : "" }}{{ document.title }}</span>
            </button>
            <button
              class="tab-close"
              type="button"
              :aria-label="document.pinned ? 'Unpin document' : 'Pin document'"
              @click="store.togglePin(document.id)"
            >
              {{ document.pinned ? "!" : "^" }}
            </button>
            <button class="tab-close" type="button" aria-label="Close document" @click="store.closeDocument(document.id)">x</button>
          </div>
        </section>
      </section>

      <section class="window-meta" aria-label="Document status">
        <span>{{ active.compile?.semantic.status || "draft" }}</span>
        <span v-if="store.gitStatus?.inside_repo">{{ store.gitStatus.branch || "detached" }}{{ store.gitStatus.dirty ? " dirty" : " clean" }}</span>
      </section>
    </header>

    <nav class="command-bar" aria-label="Main commands">
      <button type="button" @click="store.newDocument">New</button>
      <button type="button" @click="openDocument">Open</button>
      <button type="button" @click="openFolder">Open Folder</button>
      <button type="button" @click="saveWorkspace">Save Workspace</button>
      <button type="button" @click="saveDocument">Save</button>
      <button type="button" @click="saveDocumentAs">Save As</button>
      <button type="button" @click="store.revertActive">Revert</button>
      <button type="button" @click="renameDocument">Rename</button>
      <button type="button" @click="duplicateDocument">Duplicate</button>
      <button type="button" @click="store.revealActive">Reveal</button>
      <button type="button" @click="store.snapshotActive()">Snapshot</button>
      <button type="button" @click="exportDocument">Export</button>
      <button type="button" @click="openAiPaste">AI Paste</button>
      <button type="button" @click="commandPaletteOpen = true">Commands</button>
      <span class="divider"></span>
      <button type="button" title="Bold" @click="wrapSelection('**')"><strong>B</strong></button>
      <button type="button" title="Italic" @click="wrapSelection('*')"><em>I</em></button>
      <button type="button" title="Code" @click="wrapSelection('`')">Code</button>
      <button type="button" title="Heading" @click="insertAtLineStart('## ')">H2</button>
      <button type="button" title="Link" @click="wrapSelection('[', '](https://)')">Link</button>
      <button type="button" title="Table" @click="insertBlock(tableSnippet)">Table</button>
      <button type="button" title="Calculation" @click="insertBlock(calcSnippet)">Calc</button>
      <button type="button" title="AI source" @click="insertBlock(aiSnippet)">AI</button>
      <button type="button" title="Find and replace" @click="runEditorCommand(openSearchPanel)">Find</button>
      <button type="button" title="Find next" @click="runEditorCommand(findNext)">Next</button>
      <span class="divider"></span>
      <select v-model="store.mode" aria-label="View mode">
        <option value="split">Split</option>
        <option value="source">Source</option>
        <option value="preview">Preview</option>
        <option value="focus">Focus</option>
        <option value="export">Export</option>
        <option value="review">Review</option>
        <option value="presentation">Presentation</option>
      </select>
      <select v-model="store.sidebar" aria-label="Sidebar panel">
        <option value="files">Files</option>
        <option value="outline">Outline</option>
        <option value="diagnostics">Diagnostics</option>
        <option value="tables">Tables</option>
        <option value="references">References</option>
        <option value="exports">Exports</option>
        <option value="versioning">Versioning</option>
        <option value="review">Review</option>
        <option value="settings">Settings</option>
      </select>
    </nav>

    <main class="workspace" :class="`mode-${store.mode}`">
      <aside class="sidebar" aria-label="Document workspace">
        <template v-if="store.sidebar === 'files'">
          <h2>Workspace</h2>
          <button type="button" @click="openFolder">Open folder</button>
          <button v-if="store.workspaceRoot" type="button" @click="store.refreshWorkspace">Refresh</button>
          <p v-if="store.workspaceRoot" class="workspace-root">{{ store.workspaceRoot }}</p>
          <p v-else>Open a folder to browse project files.</p>
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

        <template v-else-if="store.sidebar === 'outline'">
          <h2>Outline</h2>
          <button
            v-for="heading in active.compile?.semantic.outline || []"
            :key="`${heading.line}-${heading.anchor}`"
            class="outline-row"
            :style="{ paddingLeft: `${heading.level * 10}px` }"
            type="button"
            @click="goToLine(heading.line)"
          >
            {{ heading.text }}
          </button>
        </template>

        <template v-else-if="store.sidebar === 'diagnostics'">
          <h2>Diagnostics</h2>
          <article
            v-for="diagnostic in active.compile?.diagnostics || []"
            :key="`${diagnostic.severity}-${diagnostic.message}-${diagnostic.line}`"
            class="diagnostic"
            :class="diagnostic.severity"
          >
            <strong>{{ diagnostic.severity }}</strong>
            <p>{{ diagnostic.message }}</p>
            <small v-if="diagnostic.suggestion">{{ diagnostic.suggestion }}</small>
          </article>
        </template>

        <template v-else-if="store.sidebar === 'tables'">
          <h2>Tables</h2>
          <label>
            Table
            <select v-model.number="selectedTableIndex" @change="loadSelectedTable">
              <option v-for="(table, index) in markdownTables" :key="`${table.startLine}-${index}`" :value="index">
                Line {{ table.startLine }} - {{ table.caption || table.headers.join(", ") }}
              </option>
            </select>
          </label>
          <button type="button" @click="createTableDraft">New table</button>
          <template v-if="tableDraft">
            <div class="table-actions">
              <button type="button" @click="applyTableDraft">{{ isNewTableDraft ? "Insert table" : "Apply" }}</button>
              <button type="button" @click="addTableRow">Add row</button>
              <button type="button" @click="addTableColumn">Add column</button>
              <button type="button" @click="addTableTotalsRow">Add totals row</button>
            </div>
            <div class="table-metadata">
              <label>
                Table id
                <input v-model="tableDraft.id" placeholder="tbl:revenue" />
              </label>
              <label>
                Caption
                <input v-model="tableDraft.caption" placeholder="Revenue by region" />
              </label>
            </div>
            <label>
              CSV/TSV paste
              <textarea v-model="tablePasteText" rows="4"></textarea>
            </label>
            <button type="button" @click="replaceTableFromPaste">Replace from paste</button>
            <div class="table-editor-grid" :style="{ gridTemplateColumns: `110px repeat(${tableDraft.headers.length}, minmax(120px, 1fr)) 44px` }">
              <span></span>
              <input
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`header-${columnIndex}`"
                v-model="tableDraft.headers[columnIndex]"
                aria-label="Column header"
              />
              <span></span>
              <span>Align</span>
              <select v-for="(_, columnIndex) in tableDraft.headers" :key="`align-${columnIndex}`" v-model="tableDraft.alignments[columnIndex]">
                <option value="left">Left</option>
                <option value="center">Center</option>
                <option value="right">Right</option>
              </select>
              <span></span>
              <span>Format</span>
              <select v-for="(_, columnIndex) in tableDraft.headers" :key="`format-${columnIndex}`" v-model="tableDraft.formats[columnIndex]">
                <option value="text">Text</option>
                <option value="number">Number</option>
                <option value="currency">Currency</option>
                <option value="percent">Percent</option>
                <option value="date">Date</option>
              </select>
              <span></span>
              <span>Sort</span>
              <button v-for="(_, columnIndex) in tableDraft.headers" :key="`sort-${columnIndex}`" type="button" @click="sortTableRows(columnIndex)">Sort</button>
              <span></span>
              <template v-for="(row, rowIndex) in tableDraft.rows" :key="`row-${rowIndex}`">
                <button type="button" @click="removeTableRow(rowIndex)">Remove</button>
                <input
                  v-for="(_, columnIndex) in tableDraft.headers"
                  :key="`cell-${rowIndex}-${columnIndex}`"
                  v-model="row[columnIndex]"
                  aria-label="Table cell"
                />
                <span></span>
              </template>
              <span>Totals</span>
              <output v-for="(total, columnIndex) in tableColumnTotals" :key="`total-${columnIndex}`">
                {{ total || "-" }}
              </output>
              <span></span>
              <span>Remove column</span>
              <button v-for="(_, columnIndex) in tableDraft.headers" :key="`remove-col-${columnIndex}`" type="button" @click="removeTableColumn(columnIndex)">
                Remove
              </button>
              <span></span>
            </div>
          </template>
          <p v-else>No Markdown table selected.</p>
        </template>

        <template v-else-if="store.sidebar === 'references'">
          <h2>References</h2>
          <label>
            Citation style
            <select :value="citationStyle" @change="setCitationStyle(eventValue($event))">
              <option value="title">Title</option>
              <option value="author-year">Author-year</option>
              <option value="key">Key</option>
            </select>
          </label>
          <h3>Citations</h3>
          <p v-for="citation in active.compile?.semantic.citation_references || []" :key="`${citation.key}-${citation.locator || ''}`">
            [@{{ citation.key }}<template v-if="citation.locator">, {{ citation.locator }}</template>]
            <small>{{ bibliographyByKey.get(citation.key) || "Missing bibliography entry" }}</small>
          </p>
          <template v-if="active.compile?.semantic.duplicate_bibliography_keys.length">
            <h3>Duplicate keys</h3>
            <p v-for="key in active.compile.semantic.duplicate_bibliography_keys" :key="key" class="error">{{ key }}</p>
          </template>
          <h3>Glossary</h3>
          <dl>
            <template v-for="(definition, term) in active.compile?.semantic.glossary || {}" :key="term">
              <dt>{{ term }}</dt>
              <dd>{{ definition }}</dd>
            </template>
          </dl>
          <h3>Index</h3>
          <button v-for="term in active.compile?.index_terms || []" :key="term" class="outline-row" type="button" @click="goToSearchTerm(term)">
            {{ term }}
          </button>
          <h3>Tables</h3>
          <article v-for="table in active.compile?.semantic.table_summaries || []" :key="table.line" class="snapshot-row">
            <p>{{ table.rows }} rows | {{ table.columns.join(", ") }}</p>
            <small v-for="(total, column) in table.numeric_columns" :key="column">{{ column }} total: {{ total }} </small>
          </article>
          <h3>Includes</h3>
          <p v-for="edge in active.compile?.include_graph || []" :key="`${edge.parent}-${edge.child}`">{{ edge.child }}</p>
          <h3>Cross references</h3>
          <p v-for="reference in active.compile?.semantic.cross_references || []" :key="reference.key">
            {{ reference.key }}: {{ reference.resolved ? "resolved" : "missing" }}
          </p>
          <h3>Labels</h3>
          <p v-for="label in active.compile?.semantic.labels || []" :key="label">{{ label }}</p>
        </template>

        <template v-else-if="store.sidebar === 'exports'">
          <h2>Export</h2>
          <label>
            Target
            <select v-model="store.exportTarget">
              <option value="html">HTML</option>
              <option value="pdf">PDF</option>
              <option value="docx">DOCX</option>
              <option value="pptx">PPTX</option>
              <option value="markdown-bundle">Markdown bundle</option>
            </select>
          </label>
          <label><input v-model="store.exportDefaults.includeManifest" type="checkbox" /> Export manifest</label>
          <label><input v-model="store.exportDefaults.includeComments" type="checkbox" /> Include comments</label>
          <label><input v-model="store.exportDefaults.includeProvenance" type="checkbox" /> Include AI provenance</label>
          <label><input v-model="store.exportDefaults.includeGlossary" type="checkbox" /> Include glossary</label>
          <button type="button" @click="store.prepareForExport">Prepare for export</button>
          <button type="button" @click="exportDocument">Export document</button>
          <article v-if="store.exportReadiness" class="readiness" :class="{ ready: store.exportReadiness.ready }">
            <strong>{{ store.exportReadiness.ready ? "Ready" : "Needs attention" }}</strong>
            <p>{{ store.exportReadiness.error_count }} errors, {{ store.exportReadiness.warning_count }} warnings, {{ store.exportReadiness.info_count }} info</p>
          </article>
          <h3>Manifest</h3>
          <pre>{{ manifestPreview }}</pre>
          <h3>Snapshots</h3>
          <button type="button" @click="store.listSnapshots">Refresh snapshots</button>
          <article v-for="snapshot in store.snapshots" :key="snapshot.snapshot_path" class="snapshot-row">
            <p>{{ snapshot.label || "snapshot" }}</p>
            <small>{{ snapshot.created_at || snapshot.snapshot_path }}</small>
            <small>{{ snapshot.document_version || "unversioned" }} | {{ snapshot.status || "unknown" }} | {{ snapshot.author || "unknown author" }}</small>
            <button type="button" @click="store.restoreSnapshot(snapshot.snapshot_path)">Restore</button>
          </article>
        </template>

        <template v-else-if="store.sidebar === 'versioning'">
          <h2>Versioning</h2>
          <article v-if="store.gitStatus?.inside_repo" class="snapshot-row">
            <p>{{ store.gitStatus.branch || "detached" }} | {{ store.gitStatus.dirty ? "dirty" : "clean" }}</p>
            <small v-for="line in store.gitStatus.summary" :key="line">{{ line }}</small>
          </article>
          <p v-else>Current document is not inside a Git repository.</p>
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

        <template v-else-if="store.sidebar === 'review'">
          <h2>Review</h2>
          <h3>Release</h3>
          <label>
            Status
            <select :value="String(active.compile?.semantic.status || 'draft')" @change="setDocumentStatus(inputValue($event))">
              <option v-for="status in releaseStatuses" :key="status" :value="status">{{ status }}</option>
            </select>
          </label>
          <label>
            Version
            <input :value="String(active.compile?.metadata.version || '')" @change="setFrontMatterField('version', inputValue($event))" />
          </label>
          <label>
            Approved by
            <input :value="String(active.compile?.metadata.approvedBy || '')" @change="setFrontMatterField('approvedBy', inputValue($event))" />
          </label>
          <label>
            Approved at
            <input :value="String(active.compile?.metadata.approvedAt || '')" @change="setFrontMatterField('approvedAt', inputValue($event))" />
          </label>
          <button type="button" @click="setApprovalTimestampNow">Set approval time</button>
          <label>
            New comment
            <textarea v-model="reviewCommentText" rows="4" placeholder="Review note"></textarea>
          </label>
          <button type="button" @click="insertReviewComment">Add comment</button>
          <h3>Comments</h3>
          <article v-for="comment in active.compile?.semantic.comments || []" :key="String(comment.line)" class="snapshot-row">
            <p>{{ comment.text }}</p>
            <small>Line {{ comment.line }} | {{ comment.state }} | {{ comment.author || "local" }}{{ comment.created_at ? ` | ${comment.created_at}` : "" }}</small>
            <button v-if="comment.state !== 'resolved'" type="button" @click="store.resolveReviewComment(Number(comment.line))">Resolve</button>
          </article>
          <h3>AI provenance</h3>
          <article v-for="source in active.compile?.semantic.ai_sources || []" :key="`ai-source-${source.line}`" class="snapshot-row">
            <p>{{ source.provider || "unknown" }} / {{ source.model || "unknown" }}</p>
            <small>{{ source.status }} | {{ source.reviewed_by || "unreviewed" }}{{ source.reviewed_at ? ` | ${source.reviewed_at}` : "" }}{{ source.prompt_summary ? ` | ${source.prompt_summary}` : "" }}</small>
            <label>
              <input
                type="checkbox"
                :checked="source.status === 'human-reviewed'"
                @change="toggleAiSourceReview(Number(source.line), $event)"
              />
              Human reviewed
            </label>
          </article>
          <article v-for="section in active.compile?.semantic.ai_assisted_sections || []" :key="`ai-section-${section.line}`" class="snapshot-row">
            <p>{{ section.heading || "Document body" }}</p>
            <small>Line {{ section.line }} | {{ section.status }} | {{ section.reviewed_by || "unreviewed" }}{{ section.reviewed_at ? ` | ${section.reviewed_at}` : "" }}</small>
            <label>
              <input
                type="checkbox"
                :checked="section.status === 'human-reviewed'"
                @change="toggleAiSectionReview(Number(section.line), $event)"
              />
              Human reviewed
            </label>
          </article>
        </template>

        <template v-else>
          <h2>Settings</h2>
          <label>
            Theme
            <select v-model="store.theme">
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </label>
          <label><input v-model="store.wordWrap" type="checkbox" /> Word wrap</label>
          <label><input v-model="store.lineNumbers" type="checkbox" /> Line numbers</label>
          <label><input v-model="store.highContrast" type="checkbox" /> High contrast</label>
          <label><input v-model="store.reducedMotion" type="checkbox" /> Reduced motion</label>
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
          <h3>Export defaults</h3>
          <label><input v-model="store.exportDefaults.includeManifest" type="checkbox" /> Manifest next to export</label>
          <label><input v-model="store.exportDefaults.includeComments" type="checkbox" /> Comments</label>
          <label><input v-model="store.exportDefaults.includeProvenance" type="checkbox" /> AI provenance</label>
          <label><input v-model="store.exportDefaults.includeGlossary" type="checkbox" /> Glossary</label>
          <h3>Bibliography defaults</h3>
          <label>
            Citation style
            <select v-model="store.bibliographyDefaults.citationStyle">
              <option value="title">Title</option>
              <option value="author-year">Author-year</option>
              <option value="key">Key</option>
            </select>
          </label>
          <h3>Brand profile defaults</h3>
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
          <h3>Git integration</h3>
          <label><input v-model="store.gitIntegration.enabled" type="checkbox" /> Enable Git status</label>
          <label><input v-model="store.gitIntegration.warnOnDirtyExport" type="checkbox" /> Warn on dirty export</label>
          <h3>AI paste cleanup defaults</h3>
          <label><input v-model="store.aiCleanupDefaults.markAsDraft" type="checkbox" /> Mark as draft</label>
          <label><input v-model="store.aiCleanupDefaults.addProvenance" type="checkbox" /> Add provenance block</label>
          <label><input v-model="store.aiCleanupDefaults.insertCitationTodos" type="checkbox" /> Insert citation TODOs</label>
          <h3>Typography</h3>
          <label>
            Editor font
            <input v-model="store.editorFont" />
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
            Preview line height
            <input v-model.number="store.previewLineHeight" type="number" min="1" max="2.4" step="0.05" />
          </label>
          <h3>Recent files</h3>
          <button v-for="path in store.recentFiles" :key="path" class="outline-row" type="button" @click="store.openPath(path)">
            {{ path }}
          </button>
          <h3>Recent folders</h3>
          <button v-for="path in store.recentFolders" :key="path" class="outline-row" type="button" @click="store.openFolder(path)">
            {{ path }}
          </button>
          <h3>Recently closed</h3>
          <button v-for="path in store.recentlyClosed" :key="path" class="outline-row" type="button" @click="store.openPath(path)">
            {{ path }}
          </button>
          <h3>Transform engines</h3>
          <label>
            Timeout
            <input
              :value="store.transformTimeoutMs"
              type="number"
              min="1"
              max="30000"
              step="250"
              @change="store.setTransformTimeout(Number(eventValue($event)))"
            />
          </label>
          <article v-for="engine in store.externalTransformEngines" :key="engine.name" class="engine-row">
            <h4>{{ engine.name }}</h4>
            <small>{{ engine.execution }}</small>
            <label>
              Engine path
              <span class="path-picker">
                <input :value="store.transformEnginePaths[engine.name] || ''" @change="store.setTransformEnginePath(engine.name, eventValue($event))" />
                <button type="button" @click="chooseTransformEngine(engine.name)">Choose</button>
              </span>
            </label>
            <label><input :checked="Boolean(store.trustedTransformEngines[engine.name])" type="checkbox" @change="store.setTransformTrust(engine.name, eventChecked($event))" /> Trusted</label>
            <label>
              Input
              <select :value="store.transformInputModes[engine.name] || 'stdin'" @change="store.setTransformInputMode(engine.name, eventValue($event) === 'file' ? 'file' : 'stdin')">
                <option v-for="mode in engine.inputModes" :key="mode" :value="mode">{{ mode }}</option>
              </select>
            </label>
            <button type="button" @click="store.testExternalTransform(engine.name)">Probe</button>
          </article>
          <p v-for="engine in store.transformEngines.filter((candidate) => !candidate.requiresExecution)" :key="engine.name" class="engine-summary">
            {{ engine.name }}: {{ engine.execution }}
          </p>
        </template>
      </aside>

      <section v-show="store.mode !== 'preview' && store.mode !== 'export'" class="editor-pane" aria-label="Markdown source">
        <div ref="editorHost" class="editor-host"></div>
      </section>

      <section
        ref="previewPane"
        v-show="store.mode !== 'source' && store.mode !== 'focus'"
        class="preview-pane"
        aria-label="Live preview"
        @scroll="syncEditorScrollFromPreview"
      >
        <article class="preview-document" :style="previewDocumentStyle" @click="handlePreviewClick" v-html="active.compile?.html || ''"></article>
      </section>
    </main>

    <footer class="status-bar">
      <span>{{ store.statusMessage }}</span>
      <span v-if="store.externalConflict" class="conflict-actions">
        <button type="button" @click="conflictOpen = true">Compare</button>
        <button type="button" @click="store.acceptExternalChanges">Accept external</button>
        <button type="button" @click="store.keepLocalChanges">Keep local</button>
        <button type="button" @click="saveConflictCopy">Save copy</button>
      </span>
      <span>{{ wordStats }}</span>
      <span v-if="store.lastError" class="error">{{ store.lastError }}</span>
    </footer>

    <section v-if="aiPasteOpen" class="modal-backdrop" role="dialog" aria-modal="true" aria-label="AI paste cleanup">
      <form class="modal" @submit.prevent="cleanAiPaste">
        <header>
          <h2>Paste from AI Chat</h2>
          <button type="button" @click="closeAiPaste">x</button>
        </header>
        <section class="compare-grid ai-paste-grid">
          <label>
            Original
            <textarea v-model="aiPasteText" rows="12" placeholder="Paste AI chat output here"></textarea>
          </label>
          <label>
            Cleaned preview
            <textarea :value="store.aiCleanupPreview?.cleaned_markdown || ''" rows="12" readonly placeholder="Preview cleaned Markdown"></textarea>
          </label>
        </section>
        <label><input v-model="aiMarkAsDraft" type="checkbox" /> Mark as draft</label>
        <label><input v-model="aiAddProvenance" type="checkbox" /> Add provenance block</label>
        <label><input v-model="aiInsertCitationTodos" type="checkbox" /> Insert citation TODOs</label>
        <label>
          Insert mode
          <select v-model="aiInsertMode">
            <option value="insert">Insert after document</option>
            <option value="quote">Quote</option>
            <option value="appendix">Appendix</option>
            <option value="selection">Replace selection</option>
            <option value="section">Merge into section</option>
            <option value="replace">Replace document</option>
          </select>
        </label>
        <section v-if="store.aiCleanupIssues.length" class="issue-list">
          <p v-for="issue in store.aiCleanupIssues" :key="issue">{{ issue }}</p>
        </section>
        <footer>
          <button type="button" @click="closeAiPaste">Cancel</button>
          <button type="button" :disabled="aiPreviewBusy || !aiPasteText.trim()" @click="previewAiPaste">
            {{ aiPreviewBusy ? "Cleaning" : "Preview cleanup" }}
          </button>
          <button type="submit" :disabled="aiPreviewBusy || !aiPasteText.trim()">Insert cleaned</button>
        </footer>
      </form>
    </section>

    <section v-if="commandPaletteOpen" class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Command palette">
      <div class="modal command-modal">
        <header>
          <h2>Command Palette</h2>
          <button type="button" @click="commandPaletteOpen = false">x</button>
        </header>
        <input v-model="commandQuery" autofocus placeholder="Search commands, headings, citations, glossary, index terms" />
        <button
          v-for="command in filteredCommands"
          :key="command.name"
          class="command-row"
          type="button"
          @click="runCommand(command.run)"
        >
          <strong>{{ command.name }}</strong>
          <span>{{ command.group }}</span>
        </button>
      </div>
    </section>

    <section v-if="conflictOpen && store.externalConflict" class="modal-backdrop" role="dialog" aria-modal="true" aria-label="External file conflict">
      <div class="modal conflict-modal">
        <header>
          <h2>External Changes</h2>
          <button type="button" @click="conflictOpen = false">x</button>
        </header>
        <p>{{ store.externalConflict.message }}</p>
        <p class="conflict-path">{{ store.externalConflict.path }}</p>
        <section v-if="rootConflictCanMerge" class="conflict-merge">
          <div class="conflict-toolbar">
            <button type="button" @click="seedConflictMerge('local')">Use local as merge base</button>
            <button type="button" @click="seedConflictMerge('external')">Use external as merge base</button>
            <button type="button" :disabled="!mergedConflictText.trim()" @click="applyConflictMerge">Apply merged text</button>
          </div>
          <section class="conflict-diff" aria-label="Conflict line diff">
            <div class="conflict-diff-head">Local</div>
            <div class="conflict-diff-head">External</div>
            <template v-for="row in conflictDiffRows" :key="row.key">
              <pre :class="['conflict-diff-cell', `is-${row.kind}`]"><span>{{ row.localLine || "" }}</span>{{ row.local }}</pre>
              <pre :class="['conflict-diff-cell', `is-${row.kind}`]"><span>{{ row.externalLine || "" }}</span>{{ row.external }}</pre>
            </template>
          </section>
          <label class="merge-editor">
            Merged result
            <textarea v-model="mergedConflictText" rows="12"></textarea>
          </label>
        </section>
        <section v-else class="compare-grid">
          <article>
            <h3>Local document</h3>
            <pre>{{ active.text }}</pre>
          </article>
          <article>
            <h3>Changed file</h3>
            <pre>{{ store.externalConflict.externalText || "Included file changed. Recompile to update the preview." }}</pre>
          </article>
        </section>
        <footer>
          <button type="button" @click="store.keepLocalChanges(); conflictOpen = false">Keep local</button>
          <button type="button" @click="saveConflictCopy">Save copy</button>
          <button type="button" @click="store.acceptExternalChanges(); conflictOpen = false">Accept external</button>
        </footer>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { EditorState, RangeSetBuilder } from "@codemirror/state";
import { Decoration, EditorView, keymap, lineNumbers, ViewPlugin, type DecorationSet, type ViewUpdate } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { findNext, findPrevious, openSearchPanel, replaceAll, replaceNext, searchKeymap } from "@codemirror/search";
import { closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import { forceLinting, linter, lintGutter, type Diagnostic as CodeMirrorDiagnostic } from "@codemirror/lint";
import { useDocumentsStore } from "./stores/documents";
import type { AiCleanupResponse, DocumentDiagnostic, OpenDocument } from "./types";

const store = useDocumentsStore();
const editorHost = ref<HTMLElement | null>(null);
const previewPane = ref<HTMLElement | null>(null);
let editorView: EditorView | null = null;
let debounceHandle = 0;
let autosaveHandle = 0;
let autoSnapshotHandle = 0;
let lastAutoSnapshotSignature = "";
let syncingScroll = false;
const aiPasteOpen = ref(false);
const aiPasteText = ref("");
const aiInsertMode = ref<"insert" | "quote" | "replace" | "appendix" | "selection" | "section">("insert");
const aiAddProvenance = ref(true);
const aiMarkAsDraft = ref(true);
const aiInsertCitationTodos = ref(true);
const aiPreviewBusy = ref(false);
const aiPreviewSignature = ref("");
const commandPaletteOpen = ref(false);
const conflictOpen = ref(false);
const mergedConflictText = ref("");
const commandQuery = ref("");
const reviewCommentText = ref("");
const selectedTableIndex = ref(0);
const tablePasteText = ref("");
const tableDraft = ref<TableDraft | null>(null);
const isNewTableDraft = ref(false);
const draggedTabId = ref("");

interface MarkdownTable {
  startLine: number;
  endLine: number;
  captionLine?: number;
  id: string;
  caption: string;
  headers: string[];
  alignments: TableAlignment[];
  rows: string[][];
}

type TableAlignment = "left" | "center" | "right";
type TableFormat = "text" | "number" | "currency" | "percent" | "date";

interface TableDraft {
  id: string;
  caption: string;
  headers: string[];
  alignments: TableAlignment[];
  formats: TableFormat[];
  rows: string[][];
}

interface ConflictDiffRow {
  key: string;
  kind: "equal" | "local" | "external";
  local: string;
  external: string;
  localLine: number | null;
  externalLine: number | null;
}

interface DocumentTabGroup {
  key: string;
  label: string;
  title: string;
  documents: OpenDocument[];
}

const tableSnippet = `| Item | Value |\n| --- | ---: |\n| Revenue | 125000 |\n`;
const calcSnippet = "```calc\nrevenue = 125000\ncost = 74000\nprofit = revenue - cost\n```\n";
const aiSnippet =
  "```ai-source\nprovider: OpenAI\nmodel: ChatGPT\ndate: 2026-05-18\npromptSummary: \nreviewedBy: \nreviewedAt: \nstatus: needs-review\n```\n";
const releaseStatuses = ["draft", "in-review", "approved", "published", "archived"];

const active = computed(() => store.activeDocument);
const previewDocumentStyle = computed(() => ({
  fontFamily: store.previewFont,
  lineHeight: String(clampUiLineHeight(store.previewLineHeight)),
}));
const wordStats = computed(() => {
  const text = active.value?.text || "";
  const words = text.trim().split(/\s+/).filter(Boolean).length;
  const minutes = words ? Math.max(1, Math.ceil(words / 220)) : 0;
  return `${words} words | ${text.length} characters | ${minutes} min read`;
});
const manifestPreview = computed(() => JSON.stringify(active.value.compile?.export_manifest || {}, null, 2));
const bibliographyByKey = computed(() => new Map((active.value.compile?.bibliography || []).map((entry) => [entry.key, entry.title])));
const citationStyle = computed(() =>
  String(active.value.compile?.metadata.citationStyle || active.value.compile?.metadata.cslStyle || store.bibliographyDefaults.citationStyle),
);
const markdownTables = computed(() => parseMarkdownTables(active.value?.text || ""));
const selectedTable = computed(() => markdownTables.value[selectedTableIndex.value] || null);
const groupedDocuments = computed<DocumentTabGroup[]>(() => {
  const groups = new Map<string, DocumentTabGroup>();
  for (const document of store.documents) {
    const descriptor = tabGroupDescriptor(document);
    let group = groups.get(descriptor.key);
    if (!group) {
      group = { ...descriptor, documents: [] };
      groups.set(descriptor.key, group);
    }
    group.documents.push(document);
  }
  return Array.from(groups.values());
});
const rootConflictCanMerge = computed(
  () => store.externalConflict?.reason === "root" && typeof store.externalConflict.externalText === "string",
);
const conflictDiffRows = computed(() => buildConflictDiff(active.value.text, store.externalConflict?.externalText || ""));
const tableColumnTotals = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return [];
  return draft.headers.map((_, columnIndex) => formatTableTotal(draft, columnIndex));
});
const diagnosticSignature = computed(() =>
  (active.value.compile?.diagnostics || [])
    .map((diagnostic) => [diagnostic.severity, diagnostic.source_file || "", diagnostic.line || "", diagnostic.message].join(":"))
    .join("\n"),
);
const commands = computed(() => [
  { name: "New document", group: "File", run: () => store.newDocument() },
  { name: "Open document", group: "File", run: () => void openDocument() },
  { name: "Open folder", group: "Workspace", run: () => void openFolder() },
  { name: "Save workspace", group: "Workspace", run: () => void saveWorkspace() },
  { name: "Save document", group: "File", run: () => void saveDocument() },
  { name: "Save as", group: "File", run: () => void saveDocumentAs() },
  { name: "Revert to saved", group: "File", run: () => void store.revertActive() },
  { name: "Rename document", group: "File", run: () => void renameDocument() },
  { name: "Duplicate document", group: "File", run: () => void duplicateDocument() },
  { name: "Prepare for export", group: "Export", run: () => void store.prepareForExport() },
  { name: "Export document", group: "Export", run: () => void exportDocument() },
  { name: "Create snapshot", group: "Versioning", run: () => void store.snapshotActive() },
  { name: "Refresh Git diff", group: "Versioning", run: () => void store.refreshGitDiff() },
  { name: "Commit document", group: "Versioning", run: () => void store.commitActive() },
  { name: "Tag release", group: "Versioning", run: () => void store.tagActiveRelease() },
  { name: "Paste from AI chat", group: "AI", run: () => openAiPaste() },
  { name: "Run transforms", group: "Transforms", run: () => void store.compileActive() },
  { name: "Find and replace", group: "Edit", run: () => runEditorCommand(openSearchPanel) },
  { name: "Find next", group: "Edit", run: () => runEditorCommand(findNext) },
  { name: "Find previous", group: "Edit", run: () => runEditorCommand(findPrevious) },
  { name: "Replace next", group: "Edit", run: () => runEditorCommand(replaceNext) },
  { name: "Replace all", group: "Edit", run: () => runEditorCommand(replaceAll) },
  { name: "Add review comment", group: "Review", run: () => (store.sidebar = "review") },
  { name: "Open table editor", group: "Tables", run: () => openTableEditor() },
  { name: "Insert table", group: "Snippet", run: () => insertBlock(tableSnippet) },
  { name: "Insert calculation", group: "Snippet", run: () => insertBlock(calcSnippet) },
  { name: "Insert AI source", group: "Snippet", run: () => insertBlock(aiSnippet) },
  {
    name: active.value.pinned ? "Unpin active tab" : "Pin active tab",
    group: "Workspace",
    run: () => store.togglePin(active.value.id),
  },
  ...store.documents.map((document) => ({
    name: document.title,
    group: "Open document",
    run: () => {
      store.activeId = document.id;
    },
  })),
  ...store.workspaceFiles
    .filter((entry) => entry.kind !== "directory")
    .map((entry) => ({
      name: entry.relative_path,
      group: "Workspace file",
      run: () => void store.openPath(entry.path),
    })),
  ...((active.value.compile?.semantic.outline || []).map((heading) => ({
    name: heading.text,
    group: `Heading line ${heading.line}`,
    run: () => goToLine(heading.line),
  }))),
  ...((active.value.compile?.semantic.citations || []).map((citation) => ({
    name: `[@${citation}]`,
    group: "Citation",
    run: () => {
      store.sidebar = "references";
    },
  }))),
  ...Object.keys(active.value.compile?.semantic.glossary || {}).map((term) => ({
    name: term,
    group: "Glossary",
    run: () => {
      store.sidebar = "references";
    },
  })),
  ...((active.value.compile?.index_terms || []).map((term) => ({
    name: term,
    group: "Index",
    run: () => {
      store.sidebar = "references";
      goToSearchTerm(term);
    },
  }))),
  ...((active.value.compile?.diagnostics || []).map((diagnostic) => ({
    name: diagnostic.message,
    group: `Diagnostic ${diagnostic.severity}`,
    run: () => {
      store.sidebar = "diagnostics";
      if (diagnostic.line) goToLine(diagnostic.line);
    },
  }))),
]);
const filteredCommands = computed(() => {
  const query = commandQuery.value.trim().toLowerCase();
  if (!query) return commands.value;
  return commands.value.filter((command) => `${command.name} ${command.group}`.toLowerCase().includes(query));
});

onMounted(async () => {
  await store.boot();
  applyAiPasteDefaults();
  buildEditor();
  scheduleAutosave();
  scheduleAutoSnapshot();
  document.title = store.windowTitle;
  window.addEventListener("keydown", handleShortcut);
});

onBeforeUnmount(() => {
  editorView?.destroy();
  window.clearTimeout(autosaveHandle);
  window.clearTimeout(autoSnapshotHandle);
  window.removeEventListener("keydown", handleShortcut);
});

watch(
  () => active.value.id,
  async () => {
    await nextTick();
    selectedTableIndex.value = 0;
    loadSelectedTable();
    buildEditor();
  },
);

watch(
  () => active.value.text,
  (text) => {
    if (!editorView || editorView.state.doc.toString() === text) return;
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: text },
    });
  },
);

watch(
  () => [active.value.id, active.value.text, active.value.path, active.value.dirty, store.autosave, store.autosaveDelayMs, store.externalConflict?.externalHash],
  () => {
    scheduleAutosave();
  },
);

watch(
  () => [active.value.id, active.value.text, active.value.dirty, store.autoSnapshot, store.snapshotIntervalMs, store.externalConflict?.externalHash],
  () => {
    scheduleAutoSnapshot();
  },
);

watch(
  () => [active.value.id, store.externalConflict?.externalHash, store.externalConflict?.externalText],
  () => {
    if (store.externalConflict?.reason === "root") {
      mergedConflictText.value = active.value.text;
    } else {
      mergedConflictText.value = "";
    }
  },
);

watch(
  () => [
    store.wordWrap,
    store.lineNumbers,
    store.theme,
    store.highContrast,
    store.reducedMotion,
    store.editorFont,
    store.editorLineHeight,
    store.previewFont,
    store.previewLineHeight,
  ],
  () => {
    buildEditor();
    void store.persistWorkspace();
  },
);

watch(
  () => [store.autosave, store.autosaveDelayMs, store.autoSnapshot, store.snapshotIntervalMs],
  () => {
    scheduleAutosave();
    scheduleAutoSnapshot();
    void store.persistWorkspace();
  },
);

watch(
  () => store.exportTarget,
  () => {
    void store.persistWorkspace();
  },
);

watch(
  () => [
    store.exportDefaults.includeManifest,
    store.exportDefaults.includeComments,
    store.exportDefaults.includeProvenance,
    store.exportDefaults.includeGlossary,
  ],
  () => {
    void store.persistWorkspace();
  },
);

watch(
  () => store.bibliographyDefaults.citationStyle,
  () => {
    void store.compileActive();
    void store.persistWorkspace();
  },
);

watch(
  () => [store.brandProfileDefaults.name, store.brandProfileDefaults.color, store.brandProfileDefaults.logo],
  () => {
    void store.compileActive();
    void store.persistWorkspace();
  },
);

watch(
  () => [store.gitIntegration.enabled, store.gitIntegration.warnOnDirtyExport],
  () => {
    void store.refreshGitStatus();
    void store.persistWorkspace();
  },
);

watch(
  () => [
    store.aiCleanupDefaults.addProvenance,
    store.aiCleanupDefaults.markAsDraft,
    store.aiCleanupDefaults.insertCitationTodos,
  ],
  () => {
    void store.persistWorkspace();
  },
);

watch(
  () => store.windowTitle,
  (title) => {
    document.title = title;
  },
);

watch(diagnosticSignature, () => {
  if (editorView) forceLinting(editorView);
});

watch(
  markdownTables,
  (tables) => {
    if (!tables.length) {
      if (!isNewTableDraft.value) tableDraft.value = null;
      selectedTableIndex.value = 0;
      return;
    }
    if (isNewTableDraft.value) return;
    if (selectedTableIndex.value >= tables.length) {
      selectedTableIndex.value = tables.length - 1;
    }
    if (!tableDraft.value) loadSelectedTable();
  },
  { immediate: true },
);

function editorExtensions() {
  return [
    ...(store.lineNumbers ? [lineNumbers()] : []),
    lintGutter(),
    history(),
    markdown(),
    linter(editorDiagnostics, { delay: 150 }),
    semanticEditorDecorations,
    closeBrackets(),
    EditorView.contentAttributes.of({ spellcheck: "true", autocapitalize: "sentences" }),
    keymap.of([{ key: "Enter", run: continueMarkdownList }, ...closeBracketsKeymap, ...defaultKeymap, ...historyKeymap, ...searchKeymap]),
    EditorView.domEventHandlers({
      scroll: () => {
        syncPreviewScrollFromEditor();
      },
    }),
    ...(store.wordWrap ? [EditorView.lineWrapping] : []),
    EditorView.updateListener.of((update) => {
      if (!update.docChanged) return;
      window.clearTimeout(debounceHandle);
      debounceHandle = window.setTimeout(() => {
        store.updateText(update.state.doc.toString());
      }, 120);
    }),
    EditorView.theme({
      "&": {
        height: "100%",
        fontSize: "14px",
      },
      ".cm-scroller": {
        fontFamily: store.editorFont,
        lineHeight: String(clampUiLineHeight(store.editorLineHeight)),
      },
      ".cm-neditor-citation": {
        color: "#1f6f55",
        fontWeight: "700",
      },
      ".cm-neditor-variable": {
        color: "#6d28d9",
        backgroundColor: "rgba(109, 40, 217, 0.08)",
      },
      ".cm-neditor-front-matter": {
        color: "#334155",
        backgroundColor: "rgba(148, 163, 184, 0.16)",
      },
      ".cm-neditor-formula": {
        color: "#7c2d12",
        backgroundColor: "rgba(251, 146, 60, 0.16)",
      },
      ".cm-neditor-unresolved-reference": {
        color: "#991b1b",
        textDecoration: "underline wavy #dc2626",
      },
      ".cm-neditor-transform-fence": {
        color: "#1d4ed8",
        backgroundColor: "rgba(59, 130, 246, 0.12)",
      },
      ".cm-neditor-layout-token": {
        color: "#92400e",
        backgroundColor: "rgba(245, 158, 11, 0.14)",
      },
      ".cm-neditor-review-comment": {
        color: "#991b1b",
        backgroundColor: "rgba(248, 113, 113, 0.14)",
      },
      ".cm-neditor-ai-source": {
        color: "#155e75",
        backgroundColor: "rgba(14, 116, 144, 0.12)",
      },
      ".cm-neditor-ai-assisted": {
        color: "#166534",
        backgroundColor: "rgba(34, 197, 94, 0.12)",
      },
    }),
  ];
}

const semanticEditorDecorations = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = buildSemanticEditorDecorations(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = buildSemanticEditorDecorations(update.view);
      }
    }
  },
  {
    decorations: (plugin) => plugin.decorations,
  },
);

function buildSemanticEditorDecorations(view: EditorView) {
  const builder = new RangeSetBuilder<Decoration>();
  const source = view.state.doc.toString();
  const knownReferences = collectKnownReferenceAnchors(source);
  const frontMatterEndLine = frontMatterBoundaryLine(source);
  for (let lineNumber = 1; lineNumber <= view.state.doc.lines; lineNumber += 1) {
    const line = view.state.doc.line(lineNumber);
    const text = line.text;
    if (frontMatterEndLine && lineNumber <= frontMatterEndLine) {
      builder.add(line.from, line.to, Decoration.mark({ class: "cm-neditor-front-matter" }));
      continue;
    }
    if (/^\s*\{\{(?:page-break|section-break|slide)\b/.test(text) || /^\s*```layout\b/.test(text)) {
      builder.add(line.from, line.to, Decoration.mark({ class: "cm-neditor-layout-token" }));
      continue;
    }
    if (/^\s*<!--\s*comment:/.test(text)) {
      builder.add(line.from, line.to, Decoration.mark({ class: "cm-neditor-review-comment" }));
      continue;
    }
    if (/^\s*<!--\s*(?:ai-assisted:|draft:\s*AI paste cleanup review required)/.test(text)) {
      builder.add(line.from, line.to, Decoration.mark({ class: "cm-neditor-ai-assisted" }));
      continue;
    }
    if (/^\s*```ai-source\b/.test(text)) {
      builder.add(line.from, line.to, Decoration.mark({ class: "cm-neditor-ai-source" }));
      continue;
    }
    if (/^\s*```[A-Za-z0-9_-]+\b/.test(text)) {
      builder.add(line.from, line.to, Decoration.mark({ class: "cm-neditor-transform-fence" }));
      continue;
    }
    const inlineDecorations: Array<{ start: number; end: number; className: string }> = [];
    collectRegexDecorations(inlineDecorations, text, /\[@[A-Za-z0-9_:-]+(?:[^\]]*)\]/g, "cm-neditor-citation");
    collectRegexDecorations(inlineDecorations, text, /\{\{=[^}\n]+\}\}/g, "cm-neditor-formula");
    collectReferenceDecorations(inlineDecorations, text, knownReferences);
    collectRegexDecorations(inlineDecorations, text, /\{\{[^}\n]+\}\}/g, "cm-neditor-variable");
    inlineDecorations.sort((left, right) => left.start - right.start || left.end - right.end);
    for (const decoration of inlineDecorations) {
      builder.add(
        line.from + decoration.start,
        line.from + decoration.end,
        Decoration.mark({ class: decoration.className }),
      );
    }
  }
  return builder.finish();
}

function frontMatterBoundaryLine(source: string) {
  if (!source.startsWith("---\n")) return 0;
  const lines = source.split("\n");
  const endIndex = lines.findIndex((line, index) => index > 0 && line.trim() === "---");
  return endIndex > 0 ? endIndex + 1 : 0;
}

function collectKnownReferenceAnchors(text: string) {
  const anchors = new Set<string>();
  for (const match of text.matchAll(/\{#([^}\s]+)[^}]*\}/g)) {
    anchors.add(match[1]);
  }
  for (const line of text.split("\n")) {
    const match = line.trimStart().match(/^(#{1,6})\s+(.+)$/);
    if (!match) continue;
    const raw = match[2].trim();
    const explicit = raw.match(/\{#([^}\s]+)[^}]*\}/);
    anchors.add(explicit?.[1] || slugifyAnchor(raw.split("{#")[0].trim()));
  }
  return anchors;
}

function collectReferenceDecorations(
  decorations: Array<{ start: number; end: number; className: string }>,
  text: string,
  knownReferences: Set<string>,
) {
  for (const match of text.matchAll(/\{@([^}\s]+)\}/g)) {
    if (match.index === undefined || knownReferences.has(match[1])) continue;
    decorations.push({
      start: match.index,
      end: match.index + match[0].length,
      className: "cm-neditor-unresolved-reference",
    });
  }
}

function slugifyAnchor(text: string) {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");
}

function collectRegexDecorations(
  decorations: Array<{ start: number; end: number; className: string }>,
  text: string,
  pattern: RegExp,
  className: string,
) {
  for (const match of text.matchAll(pattern)) {
    const start = match.index ?? 0;
    const end = start + match[0].length;
    if (end > start) {
      decorations.push({ start, end, className });
    }
  }
}

function continueMarkdownList(view: EditorView) {
  const selection = view.state.selection.main;
  if (!selection.empty) return false;
  const line = view.state.doc.lineAt(selection.head);
  const beforeCursor = line.text.slice(0, selection.head - line.from);
  const bullet = beforeCursor.match(/^(\s*)([-+*])\s+(.*)$/);
  const numbered = beforeCursor.match(/^(\s*)(\d+)([.)])\s+(.*)$/);
  if (!bullet && !numbered) return false;

  const indent = (bullet?.[1] || numbered?.[1] || "");
  const marker = bullet ? bullet[2] : `${Number(numbered?.[2] || "0") + 1}${numbered?.[3] || "."}`;
  const content = (bullet?.[3] || numbered?.[4] || "").trim();
  if (!content) {
    view.dispatch({
      changes: { from: line.from, to: selection.head, insert: indent },
      selection: { anchor: line.from + indent.length },
      scrollIntoView: true,
    });
    return true;
  }

  const insert = `\n${indent}${marker} `;
  view.dispatch({
    changes: { from: selection.head, insert },
    selection: { anchor: selection.head + insert.length },
    scrollIntoView: true,
  });
  return true;
}

function editorDiagnostics(view: EditorView): CodeMirrorDiagnostic[] {
  return (active.value.compile?.diagnostics || []).flatMap((diagnostic) => codeMirrorDiagnostic(view, diagnostic));
}

function codeMirrorDiagnostic(view: EditorView, diagnostic: DocumentDiagnostic): CodeMirrorDiagnostic[] {
  if (!diagnostic.line || diagnosticAppliesToIncludedFile(diagnostic)) return [];
  const line = view.state.doc.line(Math.max(1, Math.min(diagnostic.line, view.state.doc.lines)));
  const message = diagnostic.suggestion ? `${diagnostic.message}\n${diagnostic.suggestion}` : diagnostic.message;
  return [
    {
      from: line.from,
      to: Math.max(line.from, line.to),
      severity: diagnostic.severity,
      message,
      source: diagnostic.source_file || "compiler",
    },
  ];
}

function diagnosticAppliesToIncludedFile(diagnostic: DocumentDiagnostic) {
  const sourceFile = diagnostic.source_file;
  const activePath = active.value.path;
  return Boolean(sourceFile && activePath && sourceFile !== activePath);
}

function buildEditor() {
  if (!editorHost.value) return;
  editorView?.destroy();
  editorView = new EditorView({
    state: EditorState.create({
      doc: active.value.text,
      extensions: editorExtensions(),
    }),
    parent: editorHost.value,
  });
}

function syncPreviewScrollFromEditor() {
  if (!editorView || !previewPane.value || syncingScroll) return;
  syncingScroll = true;
  syncScrollPosition(editorView.scrollDOM, previewPane.value);
  window.requestAnimationFrame(() => {
    syncingScroll = false;
  });
}

function syncEditorScrollFromPreview() {
  if (!editorView || !previewPane.value || syncingScroll) return;
  syncingScroll = true;
  syncScrollPosition(previewPane.value, editorView.scrollDOM);
  window.requestAnimationFrame(() => {
    syncingScroll = false;
  });
}

function syncScrollPosition(source: HTMLElement, target: HTMLElement) {
  const sourceRange = Math.max(1, source.scrollHeight - source.clientHeight);
  const targetRange = Math.max(0, target.scrollHeight - target.clientHeight);
  target.scrollTop = (source.scrollTop / sourceRange) * targetRange;
}

function runEditorCommand(command: (view: EditorView) => boolean) {
  if (!editorView) return;
  command(editorView);
  editorView.focus();
}

function activate(id: string) {
  store.activeId = id;
}

function closeTabGroup(group: DocumentTabGroup) {
  for (const document of [...group.documents]) {
    store.closeDocument(document.id);
  }
}

function dropTabOnGroup(group: DocumentTabGroup) {
  if (!draggedTabId.value) return;
  store.setPinned(draggedTabId.value, group.key === "pinned");
  draggedTabId.value = "";
}

function tabGroupDescriptor(document: OpenDocument): Omit<DocumentTabGroup, "documents"> {
  if (document.pinned) {
    return {
      key: "pinned",
      label: "Pinned",
      title: "Pinned documents",
    };
  }
  if (!document.path) {
    return {
      key: "drafts",
      label: "Drafts",
      title: "Unsaved documents",
    };
  }
  const folder = folderFromDocumentPath(document.path);
  const label = folderLabel(folder);
  return {
    key: `folder:${folder}`,
    label,
    title: folder || "Workspace root",
  };
}

function folderFromDocumentPath(path: string) {
  const normalized = normalizeDocumentPath(path);
  const index = normalized.lastIndexOf("/");
  return index > 0 ? normalized.slice(0, index) : "";
}

function folderLabel(folder: string) {
  const workspaceRoot = store.workspaceRoot ? normalizeDocumentPath(store.workspaceRoot) : "";
  if (!folder || (workspaceRoot && folder === workspaceRoot)) return "Workspace";
  const parts = folder.split("/").filter(Boolean);
  return parts[parts.length - 1] || folder;
}

function normalizeDocumentPath(path: string) {
  return path.replace(/\\/g, "/");
}

async function openDocument() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Markdown", extensions: ["md", "markdown", "mdown", "txt"] }],
  });
  if (typeof selected === "string") await store.openPath(selected);
}

async function openFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (typeof selected === "string") await store.openFolder(selected);
}

async function saveWorkspace() {
  await store.persistWorkspace();
  store.statusMessage = "Saved workspace";
}

function eventValue(event: Event) {
  return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement ? event.target.value : "";
}

function eventChecked(event: Event) {
  return event.target instanceof HTMLInputElement ? event.target.checked : false;
}

function inputValue(event: Event) {
  return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement ? event.target.value : "";
}

function setCitationStyle(style: string) {
  const supported = new Set(["title", "author-year", "key"]);
  if (!supported.has(style)) return;
  store.updateText(upsertFrontMatterField(active.value.text, "citationStyle", style));
}

function setFrontMatterField(key: string, value: string) {
  store.updateText(upsertFrontMatterField(active.value.text, key, value.trim()));
}

function setDocumentStatus(status: string) {
  if (!releaseStatuses.includes(status)) return;
  setFrontMatterField("status", status);
}

function setApprovalTimestampNow() {
  setFrontMatterField("approvedAt", new Date().toISOString());
}

function upsertFrontMatterField(text: string, key: string, value: string) {
  const line = `${key}: ${value}`;
  if (!text.startsWith("---\n")) {
    return `---\n${line}\n---\n\n${text}`;
  }
  const lines = text.split("\n");
  const endIndex = lines.findIndex((candidate, index) => index > 0 && candidate.trim() === "---");
  if (endIndex <= 0) {
    return `---\n${line}\n---\n\n${text}`;
  }
  const existingIndex = lines.findIndex((candidate, index) => index > 0 && index < endIndex && candidate.trimStart().startsWith(`${key}:`));
  if (existingIndex > 0) {
    lines[existingIndex] = line;
  } else {
    lines.splice(endIndex, 0, line);
  }
  return lines.join("\n");
}

function clampUiLineHeight(value: number) {
  return Math.min(Math.max(Number(value) || 1.55, 1), 2.4);
}

function clampAutosaveDelay(value: number) {
  return Math.min(Math.max(Number(value) || 1500, 500), 30000);
}

function clampSnapshotInterval(value: number) {
  return Math.min(Math.max(Number(value) || 300000, 30000), 3600000);
}

function scheduleAutosave() {
  window.clearTimeout(autosaveHandle);
  const doc = active.value;
  if (!store.autosave || !doc.path || !doc.dirty || store.externalConflict) return;
  autosaveHandle = window.setTimeout(() => {
    void store.saveActive().catch((error) => {
      store.lastError = error instanceof Error ? error.message : String(error);
      store.statusMessage = "Autosave failed";
    });
  }, clampAutosaveDelay(store.autosaveDelayMs));
}

function scheduleAutoSnapshot() {
  window.clearTimeout(autoSnapshotHandle);
  const doc = active.value;
  const signature = `${doc.id}:${doc.text}`;
  if (!store.autoSnapshot || !doc.dirty || store.externalConflict || lastAutoSnapshotSignature === signature) return;
  autoSnapshotHandle = window.setTimeout(() => {
    const current = active.value;
    const currentSignature = `${current.id}:${current.text}`;
    if (lastAutoSnapshotSignature === currentSignature || store.externalConflict) return;
    void store
      .createSnapshot("auto")
      .then(() => {
        lastAutoSnapshotSignature = currentSignature;
        void store.listSnapshots();
      })
      .catch((error) => {
        store.lastError = error instanceof Error ? error.message : String(error);
        store.statusMessage = "Automatic snapshot failed";
      });
  }, clampSnapshotInterval(store.snapshotIntervalMs));
}

async function chooseTransformEngine(name: string) {
  const selected = await open({
    multiple: false,
  });
  if (typeof selected === "string") await store.setTransformEnginePath(name, selected);
}

async function saveDocument() {
  if (!active.value.path) {
    await saveDocumentAs();
    return;
  }
  await store.saveActive();
}

async function saveDocumentAs() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: active.value.title.endsWith(".md") ? active.value.title : `${active.value.title}.md`,
  });
  if (path) await store.saveActive(path);
}

async function renameDocument() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: active.value.title.endsWith(".md") ? active.value.title : `${active.value.title}.md`,
  });
  if (path) await store.renameActive(path);
}

async function duplicateDocument() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: `${active.value.title.replace(/\.[^.]+$/, "")} copy.md`,
  });
  if (path) await store.duplicateActive(path);
}

async function saveConflictCopy() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: `${active.value.title.replace(/\.[^.]+$/, "")} local copy.md`,
  });
  if (path) {
    await store.saveLocalConflictCopy(path);
    conflictOpen.value = false;
  }
}

function seedConflictMerge(source: "local" | "external") {
  mergedConflictText.value = source === "external" ? store.externalConflict?.externalText || "" : active.value.text;
}

async function applyConflictMerge() {
  await store.applyConflictMerge(mergedConflictText.value);
  conflictOpen.value = false;
}

async function exportDocument() {
  const extensions: Record<typeof store.exportTarget, string> = {
    html: "html",
    pdf: "pdf",
    docx: "docx",
    pptx: "pptx",
    "markdown-bundle": "zip",
  };
  const extension = extensions[store.exportTarget];
  const path = await save({
    filters: [{ name: store.exportTarget.toUpperCase(), extensions: [extension] }],
    defaultPath: `${active.value.title.replace(/\.[^.]+$/, "")}.${extension}`,
  });
  if (path) await store.exportActive(path);
}

async function cleanAiPaste() {
  if (!aiPasteText.value.trim()) return;
  if (aiPreviewSignature.value !== aiCleanupSignature() || !store.aiCleanupPreview) {
    await previewAiPaste();
  }
  if (!store.aiCleanupPreview) return;
  if (aiInsertMode.value === "selection") {
    replaceSelectionWithAiPaste(store.aiCleanupPreview);
  } else if (aiInsertMode.value === "section") {
    mergeAiPasteIntoCurrentSection(store.aiCleanupPreview);
  } else {
    store.insertAiPaste(store.aiCleanupPreview, aiInsertMode.value);
  }
  closeAiPaste();
}

function replaceSelectionWithAiPaste(response: AiCleanupResponse) {
  if (!editorView) {
    store.insertAiPaste(response, "insert");
    return;
  }
  const markdown = response.cleaned_markdown;
  const range = editorView.state.selection.main;
  editorView.dispatch({
    changes: { from: range.from, to: range.to, insert: markdown },
    selection: { anchor: range.from + markdown.length },
  });
  store.updateText(editorView.state.doc.toString());
  editorView.focus();
  store.statusMessage = "Inserted cleaned AI paste into selection";
}

function mergeAiPasteIntoCurrentSection(response: AiCleanupResponse) {
  if (!editorView) {
    store.insertAiPaste(response, "insert");
    return;
  }
  const doc = editorView.state.doc;
  const position = findCurrentSectionEnd(doc, editorView.state.selection.main.from);
  const insertion = formatSectionInsertion(doc.toString(), position, response.cleaned_markdown);
  editorView.dispatch({
    changes: { from: position, insert: insertion },
    selection: { anchor: position + insertion.length },
  });
  store.updateText(editorView.state.doc.toString());
  editorView.focus();
  store.statusMessage = `Merged cleaned AI paste into current section with ${response.issues.length} issue notes`;
}

function findCurrentSectionEnd(doc: EditorState["doc"], position: number) {
  const cursorLine = doc.lineAt(position);
  let headingLevel = 0;
  for (let lineNumber = cursorLine.number; lineNumber >= 1; lineNumber -= 1) {
    const match = doc.line(lineNumber).text.match(/^(#{1,6})\s+\S/);
    if (match) {
      headingLevel = match[1].length;
      break;
    }
  }
  for (let lineNumber = cursorLine.number + 1; lineNumber <= doc.lines; lineNumber += 1) {
    const line = doc.line(lineNumber);
    const match = line.text.match(/^(#{1,6})\s+\S/);
    if (match && (!headingLevel || match[1].length <= headingLevel)) {
      return line.from;
    }
  }
  return doc.length;
}

function formatSectionInsertion(text: string, position: number, markdown: string) {
  const cleaned = markdown.trim();
  if (!cleaned) return "";
  const before = text.slice(0, position);
  const after = text.slice(position);
  const prefix = before.length ? (before.endsWith("\n\n") ? "" : before.endsWith("\n") ? "\n" : "\n\n") : "";
  const suffix = after.length ? (after.startsWith("\n\n") ? "" : after.startsWith("\n") ? "\n" : "\n\n") : "\n";
  return `${prefix}${cleaned}${suffix}`;
}

async function previewAiPaste() {
  if (!aiPasteText.value.trim()) return;
  aiPreviewBusy.value = true;
  try {
    await store.previewAiPaste(aiPasteText.value, {
      addProvenance: aiAddProvenance.value,
      markAsDraft: aiMarkAsDraft.value,
      insertCitationTodos: aiInsertCitationTodos.value,
    });
    aiPreviewSignature.value = aiCleanupSignature();
  } finally {
    aiPreviewBusy.value = false;
  }
}

function applyAiPasteDefaults() {
  aiAddProvenance.value = store.aiCleanupDefaults.addProvenance;
  aiMarkAsDraft.value = store.aiCleanupDefaults.markAsDraft;
  aiInsertCitationTodos.value = store.aiCleanupDefaults.insertCitationTodos;
}

function openAiPaste() {
  applyAiPasteDefaults();
  aiPasteOpen.value = true;
}

function closeAiPaste() {
  aiPasteText.value = "";
  aiPreviewSignature.value = "";
  store.aiCleanupPreview = null;
  store.aiCleanupIssues = [];
  aiPasteOpen.value = false;
}

function aiCleanupSignature() {
  return JSON.stringify({
    text: aiPasteText.value,
    addProvenance: aiAddProvenance.value,
    markAsDraft: aiMarkAsDraft.value,
    insertCitationTodos: aiInsertCitationTodos.value,
  });
}

function runCommand(run: () => unknown) {
  run();
  commandPaletteOpen.value = false;
  commandQuery.value = "";
}

function insertReviewComment() {
  store.insertReviewComment(reviewCommentText.value);
  reviewCommentText.value = "";
}

function toggleAiSectionReview(line: number, event: Event) {
  store.setAiAssistedSectionReviewed(line, Boolean((event.target as HTMLInputElement | null)?.checked));
}

function toggleAiSourceReview(line: number, event: Event) {
  store.setAiSourceReviewed(line, Boolean((event.target as HTMLInputElement | null)?.checked));
}

function wrapSelection(prefix: string, suffix = prefix) {
  if (!editorView) return;
  const range = editorView.state.selection.main;
  const selected = editorView.state.sliceDoc(range.from, range.to);
  editorView.dispatch({
    changes: { from: range.from, to: range.to, insert: `${prefix}${selected}${suffix}` },
    selection: { anchor: range.from + prefix.length, head: range.to + prefix.length },
  });
  editorView.focus();
}

function insertAtLineStart(prefix: string) {
  if (!editorView) return;
  const line = editorView.state.doc.lineAt(editorView.state.selection.main.from);
  editorView.dispatch({ changes: { from: line.from, insert: prefix } });
  editorView.focus();
}

function insertBlock(block: string) {
  if (!editorView) return;
  const position = editorView.state.selection.main.to;
  editorView.dispatch({ changes: { from: position, insert: `\n${block}\n` } });
  editorView.focus();
}

function openTableEditor() {
  store.sidebar = "tables";
  if (markdownTables.value.length && !tableDraft.value) loadSelectedTable();
  if (!markdownTables.value.length && !tableDraft.value) createTableDraft();
}

function loadSelectedTable() {
  const table = selectedTable.value;
  isNewTableDraft.value = false;
  if (!table) {
    tableDraft.value = null;
    return;
  }
  tableDraft.value = {
    id: table.id,
    caption: table.caption,
    headers: [...table.headers],
    alignments: [...table.alignments],
    formats: table.headers.map((_, columnIndex) => inferTableFormat(table.rows.map((row) => row[columnIndex] || ""))),
    rows: table.rows.map((row) => padTableRow(row, table.headers.length)),
  };
}

function createTableDraft() {
  isNewTableDraft.value = true;
  tableDraft.value = {
    id: "",
    caption: "",
    headers: ["Item", "Value"],
    alignments: ["left", "right"],
    formats: ["text", "number"],
    rows: [
      ["Revenue", "125000"],
      ["Cost", "74000"],
    ],
  };
}

function applyTableDraft() {
  const table = selectedTable.value;
  const draft = tableDraft.value;
  if (!draft) return;
  const normalizedDraft = normalizeTableDraft(draft);
  const serialized = serializeMarkdownTable(normalizedDraft);
  if (table && !isNewTableDraft.value) {
    const lines = active.value.text.split("\n");
    const replaceStart = table.captionLine || table.startLine;
    lines.splice(replaceStart - 1, table.endLine - replaceStart + 1, ...serialized);
    store.updateText(lines.join("\n"));
  } else {
    insertTableAtCursor(serialized);
    const nextTableIndex = markdownTables.value.length;
    void nextTick().then(() => {
      selectedTableIndex.value = Math.min(nextTableIndex, Math.max(0, markdownTables.value.length - 1));
      loadSelectedTable();
    });
  }
  isNewTableDraft.value = false;
  tableDraft.value = normalizedDraft;
}

function insertTableAtCursor(lines: string[]) {
  const text = active.value.text;
  const position = editorView?.state.selection.main.to ?? text.length;
  const before = text.slice(0, position);
  const after = text.slice(position);
  const block = lines.join("\n");
  const prefix = !before ? "" : before.endsWith("\n\n") ? "" : before.endsWith("\n") ? "\n" : "\n\n";
  const suffix = !after ? "\n" : after.startsWith("\n") ? "\n" : "\n\n";
  store.updateText(`${before}${prefix}${block}${suffix}${after}`);
}

function addTableRow() {
  if (!tableDraft.value) return;
  tableDraft.value.rows.push(tableDraft.value.headers.map(() => ""));
}

function removeTableRow(rowIndex: number) {
  if (!tableDraft.value) return;
  tableDraft.value.rows.splice(rowIndex, 1);
}

function addTableColumn() {
  if (!tableDraft.value) return;
  const nextColumn = tableDraft.value.headers.length + 1;
  tableDraft.value.headers.push(`Column ${nextColumn}`);
  tableDraft.value.alignments.push("left");
  tableDraft.value.formats.push("text");
  for (const row of tableDraft.value.rows) row.push("");
}

function removeTableColumn(columnIndex: number) {
  if (!tableDraft.value || tableDraft.value.headers.length <= 1) return;
  tableDraft.value.headers.splice(columnIndex, 1);
  tableDraft.value.alignments.splice(columnIndex, 1);
  tableDraft.value.formats.splice(columnIndex, 1);
  for (const row of tableDraft.value.rows) row.splice(columnIndex, 1);
}

function addTableTotalsRow() {
  const draft = tableDraft.value;
  if (!draft) return;
  const totals = draft.headers.map((_, columnIndex) => {
    if (columnIndex === 0) return "Total";
    const values = numericColumnValues(draft, columnIndex);
    if (!values.length) return "";
    return `=SUM(${values.map(formatFormulaNumber).join(",")})`;
  });
  draft.rows.push(totals);
}

function replaceTableFromPaste() {
  const rows = parseDelimitedRows(tablePasteText.value);
  if (!rows.length) return;
  const current = tableDraft.value;
  const headers = rows[0].map((cell, index) => cell.trim() || `Column ${index + 1}`);
  const bodyRows = rows.slice(1).map((row) => padTableRow(row, headers.length));
  tableDraft.value = {
    id: current?.id || "",
    caption: current?.caption || "",
    headers,
    alignments: headers.map(() => "left"),
    formats: headers.map((_, columnIndex) => inferTableFormat(bodyRows.map((row) => row[columnIndex] || ""))),
    rows: bodyRows.length ? bodyRows : [headers.map(() => "")],
  };
}

function sortTableRows(columnIndex: number) {
  if (!tableDraft.value) return;
  const format = tableDraft.value.formats[columnIndex];
  tableDraft.value.rows.sort((left, right) => compareTableCells(left[columnIndex] || "", right[columnIndex] || "", format));
}

function buildConflictDiff(localText: string, externalText: string): ConflictDiffRow[] {
  const localLines = localText.split(/\r?\n/);
  const externalLines = externalText.split(/\r?\n/);
  if (localLines.length * externalLines.length > 250_000) {
    return [
      {
        key: "large-conflict",
        kind: "local",
        local: `${localLines.length} local lines`,
        external: `${externalLines.length} external lines`,
        localLine: null,
        externalLine: null,
      },
    ];
  }

  const scores = Array.from({ length: localLines.length + 1 }, () => Array(externalLines.length + 1).fill(0));
  for (let localIndex = localLines.length - 1; localIndex >= 0; localIndex -= 1) {
    for (let externalIndex = externalLines.length - 1; externalIndex >= 0; externalIndex -= 1) {
      scores[localIndex][externalIndex] =
        localLines[localIndex] === externalLines[externalIndex]
          ? scores[localIndex + 1][externalIndex + 1] + 1
          : Math.max(scores[localIndex + 1][externalIndex], scores[localIndex][externalIndex + 1]);
    }
  }

  const rows: ConflictDiffRow[] = [];
  let localIndex = 0;
  let externalIndex = 0;
  while (localIndex < localLines.length || externalIndex < externalLines.length) {
    const key = `${localIndex}:${externalIndex}:${rows.length}`;
    if (localIndex < localLines.length && externalIndex < externalLines.length && localLines[localIndex] === externalLines[externalIndex]) {
      rows.push({
        key,
        kind: "equal",
        local: localLines[localIndex],
        external: externalLines[externalIndex],
        localLine: localIndex + 1,
        externalLine: externalIndex + 1,
      });
      localIndex += 1;
      externalIndex += 1;
    } else if (localIndex >= localLines.length) {
      rows.push({
        key,
        kind: "external",
        local: "",
        external: externalLines[externalIndex],
        localLine: null,
        externalLine: externalIndex + 1,
      });
      externalIndex += 1;
    } else if (externalIndex >= externalLines.length || scores[localIndex + 1][externalIndex] >= scores[localIndex][externalIndex + 1]) {
      rows.push({
        key,
        kind: "local",
        local: localLines[localIndex],
        external: "",
        localLine: localIndex + 1,
        externalLine: null,
      });
      localIndex += 1;
    } else {
      rows.push({
        key,
        kind: "external",
        local: "",
        external: externalLines[externalIndex],
        localLine: null,
        externalLine: externalIndex + 1,
      });
      externalIndex += 1;
    }
  }
  return rows;
}

function parseMarkdownTables(text: string): MarkdownTable[] {
  const lines = text.split("\n");
  const tables: MarkdownTable[] = [];
  let index = 0;
  while (index + 1 < lines.length) {
    const header = lines[index].trim();
    const separator = lines[index + 1].trim();
    if (!isMarkdownTableRow(header) || !isMarkdownTableSeparator(separator)) {
      index += 1;
      continue;
    }
    const caption = index > 0 ? parseTableCaption(lines[index - 1].trim()) : null;
    const headers = splitMarkdownTableRow(header);
    const alignments = splitMarkdownTableRow(separator).map(alignmentFromSeparator);
    const rows: string[][] = [];
    let nextIndex = index + 2;
    while (nextIndex < lines.length && isMarkdownTableRow(lines[nextIndex].trim())) {
      rows.push(padTableRow(splitMarkdownTableRow(lines[nextIndex].trim()), headers.length));
      nextIndex += 1;
    }
    tables.push({
      startLine: index + 1,
      endLine: nextIndex,
      captionLine: caption ? index : undefined,
      id: caption?.id || "",
      caption: caption?.caption || "",
      headers,
      alignments: padAlignments(alignments, headers.length),
      rows,
    });
    index = nextIndex;
  }
  return tables;
}

function parseTableCaption(line: string) {
  if (!line.toLowerCase().startsWith("table:")) return null;
  const id = line.match(/\{#([^}\s]+)(?:\s+[^}]*)?\}/)?.[1] || "";
  const captionAttribute = line.match(/\bcaption="([^"]*)"/)?.[1] || "";
  const captionText = line
    .replace(/^table:/i, "")
    .replace(/\{#[^}]+\}/g, "")
    .trim();
  const caption = captionAttribute || captionText;
  if (!id && !caption) return null;
  return { id, caption };
}

function isMarkdownTableRow(line: string) {
  return line.startsWith("|") && line.endsWith("|") && (line.match(/\|/g) || []).length >= 2;
}

function isMarkdownTableSeparator(line: string) {
  return isMarkdownTableRow(line) && splitMarkdownTableRow(line).every((cell) => /^:?-{3,}:?$/.test(cell.replace(/\s/g, "")));
}

function splitMarkdownTableRow(line: string) {
  return line
    .trim()
    .slice(1, -1)
    .split("|")
    .map((cell) => cell.replace(/\\\|/g, "|").trim());
}

function alignmentFromSeparator(cell: string): TableAlignment {
  const compact = cell.replace(/\s/g, "");
  if (compact.startsWith(":") && compact.endsWith(":")) return "center";
  if (compact.endsWith(":")) return "right";
  return "left";
}

function padAlignments(alignments: TableAlignment[], length: number) {
  return Array.from({ length }, (_, index) => alignments[index] || "left");
}

function padTableRow(row: string[], length: number) {
  return Array.from({ length }, (_, index) => row[index] || "");
}

function normalizeTableDraft(draft: TableDraft): TableDraft {
  const headers = draft.headers.map((header, index) => header.trim() || `Column ${index + 1}`);
  return {
    id: normalizeTableId(draft.id),
    caption: draft.caption.trim(),
    headers,
    alignments: padAlignments(draft.alignments, headers.length),
    formats: Array.from({ length: headers.length }, (_, index) => draft.formats[index] || "text"),
    rows: draft.rows.map((row) => padTableRow(row, headers.length)),
  };
}

function serializeMarkdownTable(draft: TableDraft) {
  const headers = draft.headers.map(escapeTableCell);
  const separator = draft.alignments.map(separatorForAlignment);
  const rows = draft.rows.map((row) =>
    row.map((cell, columnIndex) => escapeTableCell(formatTableCell(cell, draft.formats[columnIndex]))),
  );
  const table = [`| ${headers.join(" | ")} |`, `| ${separator.join(" | ")} |`, ...rows.map((row) => `| ${row.join(" | ")} |`)];
  const caption = serializeTableCaption(draft);
  return caption ? [caption, ...table] : table;
}

function normalizeTableId(id: string) {
  return id.trim().replace(/^\{?#?/, "").replace(/\}?$/, "");
}

function serializeTableCaption(draft: TableDraft) {
  if (!draft.id && !draft.caption) return "";
  const caption = draft.caption || "Untitled table";
  const id = draft.id ? ` {#${draft.id}}` : "";
  return `Table: ${caption}${id}`;
}

function separatorForAlignment(alignment: TableAlignment) {
  if (alignment === "center") return ":---:";
  if (alignment === "right") return "---:";
  return "---";
}

function escapeTableCell(cell: string) {
  return cell.replace(/\r?\n/g, " ").replace(/\|/g, "\\|").trim();
}

function parseDelimitedRows(text: string) {
  const source = text.trim();
  const rows = parseDelimitedText(source, detectDelimitedPasteDelimiter(source));
  const width = Math.max(0, ...rows.map((row) => row.length));
  return rows.map((row) => padTableRow(row, width));
}

function parseDelimitedText(text: string, delimiter: "," | "\t") {
  const rows: string[][] = [];
  let row: string[] = [];
  let cell = "";
  let quoted = false;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    const next = text[index + 1];
    if (char === '"' && quoted && next === '"') {
      cell += '"';
      index += 1;
    } else if (char === '"') {
      quoted = !quoted;
    } else if (char === delimiter && !quoted) {
      row.push(cell.trim());
      cell = "";
    } else if ((char === "\n" || char === "\r") && !quoted) {
      if (char === "\r" && next === "\n") index += 1;
      pushDelimitedPasteRow(rows, row, cell);
      row = [];
      cell = "";
    } else {
      cell += char;
    }
  }
  pushDelimitedPasteRow(rows, row, cell);
  return rows;
}

function pushDelimitedPasteRow(rows: string[][], row: string[], cell: string) {
  const nextRow = [...row, cell.trim()];
  if (nextRow.some((value) => value.trim())) rows.push(nextRow);
}

function detectDelimitedPasteDelimiter(text: string): "," | "\t" {
  let quoted = false;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    const next = text[index + 1];
    if (char === '"' && quoted && next === '"') {
      index += 1;
    } else if (char === '"') {
      quoted = !quoted;
    } else if (char === "\t" && !quoted) {
      return "\t";
    } else if (char === "," && !quoted) {
      return ",";
    }
  }
  return ",";
}

function inferTableFormat(values: string[]): TableFormat {
  const filled = values.map((value) => value.trim()).filter(Boolean);
  if (!filled.length) return "text";
  if (filled.every((value) => /^\$?-?\d[\d,]*(\.\d+)?$/.test(value))) {
    return filled.some((value) => value.startsWith("$")) ? "currency" : "number";
  }
  if (filled.every((value) => /^-?\d+(\.\d+)?%$/.test(value))) return "percent";
  if (filled.every((value) => !Number.isNaN(Date.parse(value)))) return "date";
  return "text";
}

function compareTableCells(left: string, right: string, format: TableFormat) {
  if (format === "number" || format === "currency" || format === "percent") {
    return parseCellNumber(left) - parseCellNumber(right);
  }
  if (format === "date") {
    return Date.parse(left) - Date.parse(right);
  }
  return left.localeCompare(right);
}

function formatTableCell(value: string, format: TableFormat) {
  const trimmed = value.trim();
  if (!trimmed || format === "text") return trimmed;
  if (format === "date") {
    const time = Date.parse(trimmed);
    return Number.isNaN(time) ? trimmed : new Date(time).toISOString().slice(0, 10);
  }
  const number = parseCellNumber(trimmed);
  if (Number.isNaN(number)) return trimmed;
  if (format === "currency") return `$${trimFixed(number, 2)}`;
  if (format === "percent") {
    const percent = trimmed.includes("%") || Math.abs(number) > 1 ? number : number * 100;
    return `${trimFixed(percent, 2)}%`;
  }
  return trimFixed(number, 2);
}

function formatTableTotal(draft: TableDraft, columnIndex: number) {
  const values = numericColumnValues(draft, columnIndex);
  if (!values.length) return "";
  const total = values.reduce((sum, value) => sum + value, 0);
  return formatTableCell(String(total), draft.formats[columnIndex]);
}

function numericColumnValues(draft: TableDraft, columnIndex: number) {
  return draft.rows
    .map((row) => parseEditableTableNumber(row[columnIndex] || ""))
    .filter((value): value is number => Number.isFinite(value));
}

function parseEditableTableNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed || trimmed.startsWith("=")) return Number.NaN;
  return parseCellNumber(trimmed);
}

function formatFormulaNumber(value: number) {
  return Number.isInteger(value) ? String(value) : trimFixed(value, 6);
}

function parseCellNumber(value: string) {
  return Number(value.replace(/[$,%]/g, ""));
}

function trimFixed(value: number, places: number) {
  return value.toFixed(places).replace(/\.?0+$/, "");
}

function goToLine(lineNumber: number) {
  if (!editorView) return;
  const line = editorView.state.doc.line(Math.max(1, Math.min(lineNumber, editorView.state.doc.lines)));
  editorView.dispatch({ selection: { anchor: line.from }, effects: EditorView.scrollIntoView(line.from, { y: "center" }) });
  editorView.focus();
}

function goToSearchTerm(term: string) {
  if (!editorView || !term.trim()) return;
  const text = editorView.state.doc.toString();
  const index = text.toLowerCase().indexOf(term.toLowerCase());
  if (index < 0) return;
  editorView.dispatch({
    selection: { anchor: index, head: index + term.length },
    effects: EditorView.scrollIntoView(index, { y: "center" }),
  });
  editorView.focus();
}

function handlePreviewClick(event: MouseEvent) {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const link = target.closest("a[href^='#']");
  const heading = target.closest("h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]");
  const anchor = heading?.id || link?.getAttribute("href")?.slice(1) || "";
  if (!anchor) return;
  const headingEntry = active.value.compile?.semantic.outline.find((item) => item.anchor === anchor);
  if (!headingEntry) return;
  event.preventDefault();
  goToLine(headingEntry.line);
}

function handleShortcut(event: KeyboardEvent) {
  if (!(event.metaKey || event.ctrlKey)) return;
  if (event.key === "s") {
    event.preventDefault();
    void saveDocument();
  } else if (event.key === "o") {
    event.preventDefault();
    void openDocument();
  } else if (event.key === "n") {
    event.preventDefault();
    store.newDocument();
  } else if (event.key === "b") {
    event.preventDefault();
    wrapSelection("**");
  } else if (event.key === "i") {
    event.preventDefault();
    wrapSelection("*");
  } else if (event.key.toLowerCase() === "p" && event.shiftKey) {
    event.preventDefault();
    commandPaletteOpen.value = true;
  }
}
</script>

<style>
:root {
  color: #18212f;
  background: #edf1f5;
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 14px;
  font-synthesis: none;
  line-height: 1.45;
  text-rendering: optimizeLegibility;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
}

button,
select,
input {
  font: inherit;
}

button,
select {
  border: 1px solid #bac4d1;
  border-radius: 6px;
  background: #f9fbfc;
  color: #18212f;
  min-height: 30px;
}

button {
  padding: 4px 10px;
  cursor: pointer;
}

button:hover,
select:hover {
  border-color: #6386b4;
}

.app-shell {
  display: grid;
  grid-template-rows: 38px 42px minmax(0, 1fr) 28px;
  width: 100vw;
  height: 100vh;
  color: #18212f;
  background: #edf1f5;
}

.app-shell[data-theme="dark"] {
  color: #e6edf5;
  background: #111821;
}

@media (prefers-color-scheme: dark) {
  .app-shell[data-theme="system"] {
    color: #e6edf5;
    background: #111821;
  }
}

.app-shell[data-high-contrast="true"] {
  color: #000000;
  background: #ffffff;
}

.app-shell[data-high-contrast="true"] .titlebar,
.app-shell[data-high-contrast="true"] .command-bar,
.app-shell[data-high-contrast="true"] .status-bar,
.app-shell[data-high-contrast="true"] .sidebar,
.app-shell[data-high-contrast="true"] button,
.app-shell[data-high-contrast="true"] select,
.app-shell[data-high-contrast="true"] input,
.app-shell[data-high-contrast="true"] textarea {
  border-color: #000000;
  color: #000000;
  background: #ffffff;
}

.app-shell[data-high-contrast="true"] .tab.active,
.app-shell[data-high-contrast="true"] .file-row.active {
  outline: 2px solid #000000;
  background: #fff6a3;
}

.app-shell[data-reduced-motion="true"] * {
  scroll-behavior: auto;
  transition-duration: 0s;
  animation-duration: 0s;
  animation-iteration-count: 1;
}

@media (prefers-reduced-motion: reduce) {
  .app-shell * {
    scroll-behavior: auto;
    transition-duration: 0s;
    animation-duration: 0s;
    animation-iteration-count: 1;
  }
}

.titlebar,
.command-bar,
.status-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  border-bottom: 1px solid #c9d2dc;
  background: #f7f9fb;
}

.document-tabs {
  display: flex;
  min-width: 0;
  flex: 1;
  gap: 8px;
  overflow-x: auto;
}

.tab-group {
  display: flex;
  align-items: stretch;
  flex: 0 0 auto;
  gap: 4px;
  min-width: 0;
  padding-right: 8px;
  border-right: 1px solid #d7dee7;
}

.tab-group:last-child {
  border-right: 0;
  padding-right: 0;
}

.tab-group-header {
  display: flex;
  min-width: 72px;
  max-width: 140px;
  flex-direction: column;
  justify-content: center;
  color: #526171;
  font-size: 11px;
  line-height: 1.2;
  text-transform: uppercase;
}

.tab-group-header span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-group-header small {
  color: #7b8794;
  font-size: 10px;
}

.tab-group-header button {
  width: 18px;
  height: 18px;
  margin-top: 3px;
  padding: 0;
  border: 1px solid #c5cfdb;
  border-radius: 4px;
  background: #fff;
  color: #526171;
  font-size: 11px;
  line-height: 1;
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  max-width: 220px;
  min-height: 30px;
  border: 1px solid #bac4d1;
  border-radius: 6px;
  background: #edf1f5;
}

.tab[draggable="true"] {
  cursor: grab;
}

.tab.active {
  border-color: #275da8;
  background: #ffffff;
}

.tab-main {
  min-width: 0;
  flex: 1;
  border: 0;
  background: transparent;
  text-align: left;
}

.tab-main span {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-close {
  min-width: 20px;
  min-height: 20px;
  padding: 0;
  border: 0;
  background: transparent;
}

.window-meta,
.status-bar {
  color: #526171;
  font-size: 12px;
}

.command-bar {
  overflow-x: auto;
}

.divider {
  width: 1px;
  height: 22px;
  background: #c9d2dc;
}

.workspace {
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr) minmax(0, 1fr);
  min-height: 0;
}

.workspace.mode-source,
.workspace.mode-focus {
  grid-template-columns: 260px minmax(0, 1fr);
}

.workspace.mode-preview,
.workspace.mode-export {
  grid-template-columns: 260px minmax(0, 1fr);
}

.sidebar,
.editor-pane,
.preview-pane {
  min-height: 0;
  overflow: auto;
  border-right: 1px solid #c9d2dc;
}

.sidebar {
  padding: 12px;
  background: #f7f9fb;
}

.sidebar h2 {
  margin: 0 0 12px;
  font-size: 13px;
  text-transform: uppercase;
  color: #526171;
}

.sidebar h3 {
  margin: 16px 0 8px;
  font-size: 13px;
}

.sidebar label {
  display: grid;
  gap: 6px;
  margin-bottom: 12px;
}

.outline-row {
  display: block;
  width: 100%;
  margin-bottom: 2px;
  border: 0;
  background: transparent;
  text-align: left;
}

.file-row {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  width: 100%;
  margin: 1px 0;
  border: 0;
  background: transparent;
  text-align: left;
}

.file-row span:last-child,
.workspace-root {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-row.directory {
  color: #526171;
  font-weight: 600;
}

.file-row.active {
  background: #e5edf7;
  color: #174a88;
}

.workspace-root {
  margin: 8px 0;
  color: #526171;
  font-size: 12px;
}

.diagnostic {
  margin-bottom: 8px;
  padding: 8px;
  border-left: 3px solid #6386b4;
  background: #ffffff;
}

.diagnostic.warning {
  border-color: #c68a1a;
}

.diagnostic.error {
  border-color: #c24141;
}

.diagnostic p {
  margin: 4px 0;
}

.readiness,
.snapshot-row,
.issue-list {
  margin: 8px 0;
  padding: 8px;
  border: 1px solid #c9d2dc;
  background: #ffffff;
}

.readiness {
  border-left: 3px solid #c68a1a;
}

.readiness.ready {
  border-left-color: #2f855a;
}

.snapshot-row p {
  margin: 0 0 4px;
}

.engine-row {
  display: grid;
  gap: 8px;
  margin: 8px 0;
  padding: 8px;
  border: 1px solid #c9d2dc;
  background: #ffffff;
}

.engine-row h4 {
  margin: 0;
  font-size: 13px;
}

.path-picker {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 6px;
}

.path-picker input {
  min-width: 0;
}

.engine-summary {
  margin: 4px 0;
  color: #526171;
  font-size: 12px;
}

.table-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}

.table-metadata {
  display: grid;
  grid-template-columns: minmax(150px, 220px) minmax(220px, 1fr);
  gap: 8px;
}

.table-editor-grid {
  display: grid;
  gap: 4px;
  min-width: 680px;
  align-items: center;
}

.table-editor-grid input,
.table-editor-grid select,
.table-editor-grid button,
.table-editor-grid output {
  width: 100%;
  min-width: 0;
}

.table-editor-grid output {
  min-height: 30px;
  padding: 6px 8px;
  border: 1px solid #d8e0e8;
  background: #f5f7fa;
  color: #1b2733;
  font-variant-numeric: tabular-nums;
}

.table-editor-grid span {
  color: #526171;
  font-size: 12px;
}

.editor-pane {
  background: #ffffff;
}

.editor-host {
  height: 100%;
}

.preview-pane {
  background: #ffffff;
}

.preview-document {
  max-width: 900px;
  margin: 0 auto;
  padding: 48px;
  color: #1f2937;
  line-height: 1.65;
}

.preview-document h1,
.preview-document h2,
.preview-document h3 {
  color: #111827;
  line-height: 1.2;
}

.preview-document table,
.transform-table {
  width: 100%;
  border-collapse: collapse;
}

.preview-document td,
.preview-document th,
.transform-table td,
.transform-table th {
  border: 1px solid #c9d2dc;
  padding: 6px 8px;
}

.preview-document .glossary-term {
  border-bottom: 1px dotted #275da8;
  color: #174a8c;
  cursor: help;
  text-decoration: none;
}

.preview-document .glossary-term:focus {
  outline: 2px solid #275da8;
  outline-offset: 2px;
}

.preview-document .citation {
  color: #174a8c;
  cursor: help;
  font-weight: 600;
}

.preview-document .citation:focus {
  outline: 2px solid #275da8;
  outline-offset: 2px;
}

.preview-document .callout {
  border-left: 4px solid #1f6f55;
  background: #eefaf4;
  padding: 10px 12px;
  margin: 14px 0;
}

.preview-document .callout strong {
  display: block;
  color: #0f5132;
  margin-bottom: 4px;
}

.preview-document pre,
.sidebar pre {
  overflow: auto;
  padding: 10px;
  background: #edf1f5;
}

.transform-pending,
.transform-calc {
  padding: 10px;
  border: 1px solid #c9d2dc;
  background: #f7f9fb;
}

.status-bar {
  justify-content: space-between;
  border-top: 1px solid #c9d2dc;
  border-bottom: 0;
}

.conflict-actions {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.error {
  color: #c24141;
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgba(15, 23, 42, 0.38);
  z-index: 20;
}

.modal {
  display: grid;
  gap: 12px;
  width: min(720px, 100%);
  max-height: min(760px, 92vh);
  overflow: auto;
  padding: 16px;
  border: 1px solid #bac4d1;
  border-radius: 8px;
  background: #f7f9fb;
  box-shadow: 0 20px 60px rgba(15, 23, 42, 0.28);
}

.modal header,
.modal footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.modal h2 {
  margin: 0;
  font-size: 18px;
}

.modal textarea,
.modal input {
  width: 100%;
  border: 1px solid #bac4d1;
  border-radius: 6px;
  padding: 10px;
  color: #18212f;
  background: #ffffff;
}

.command-modal {
  align-content: start;
}

.command-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 16px;
  width: 100%;
  padding: 8px 10px;
  text-align: left;
}

.command-row span {
  color: #526171;
  font-size: 12px;
}

.conflict-modal {
  width: min(1100px, 100%);
}

.conflict-path {
  margin: -4px 0 0;
  overflow-wrap: anywhere;
  color: #526171;
  font-family: "SFMono-Regular", Consolas, monospace;
  font-size: 12px;
}

.compare-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 12px;
  min-height: 0;
}

.compare-grid pre {
  max-height: 420px;
  overflow: auto;
  padding: 10px;
  background: #edf1f5;
}

.conflict-merge {
  display: grid;
  gap: 12px;
}

.conflict-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.conflict-diff {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  max-height: 320px;
  overflow: auto;
  border: 1px solid #c9d2dc;
  background: #ffffff;
}

.conflict-diff-head {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 8px 10px;
  border-bottom: 1px solid #c9d2dc;
  background: #edf1f5;
  font-weight: 700;
}

.conflict-diff-cell {
  min-height: 26px;
  margin: 0;
  padding: 6px 10px;
  border-bottom: 1px solid #e2e8f0;
  white-space: pre-wrap;
  word-break: break-word;
}

.conflict-diff-cell span {
  display: inline-block;
  width: 36px;
  margin-right: 8px;
  color: #64748b;
  user-select: none;
}

.conflict-diff-cell.is-local {
  background: #fff3e8;
}

.conflict-diff-cell.is-external {
  background: #eaf6f0;
}

.merge-editor {
  display: grid;
  gap: 6px;
}

.merge-editor textarea {
  min-height: 240px;
  resize: vertical;
  font-family: Menlo, Consolas, monospace;
}

@media (max-width: 900px) {
  .workspace,
  .workspace.mode-source,
  .workspace.mode-focus,
  .workspace.mode-preview,
  .workspace.mode-export {
    grid-template-columns: 1fr;
  }

  .sidebar {
    display: none;
  }

  .preview-document {
    padding: 24px;
  }

  .compare-grid {
    grid-template-columns: 1fr;
  }
}
</style>
