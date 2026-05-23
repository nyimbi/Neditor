<template>
  <div
    class="app-shell"
    :class="{ 'has-trust-prompt': externalTransformTrustPrompts.length }"
    :data-theme="store.theme"
    :data-toolbar-display="store.toolbarDisplay"
    :data-high-contrast="store.highContrast ? 'true' : 'false'"
    :data-reduced-motion="store.reducedMotion ? 'true' : 'false'"
    :style="appShellStyle"
  >
    <nav class="skip-links" aria-label="Keyboard shortcuts">
      <a href="#main-commands" @click="focusSkipTarget">Skip to commands</a>
      <a href="#document-workspace" @click="focusSkipTarget">Skip to workspace</a>
      <a href="#document-sidebar" @click="focusSkipTarget">Skip to sidebar</a>
      <a href="#markdown-source" @click="focusSkipTarget">Skip to source</a>
      <a href="#live-preview" @click="focusSkipTarget">Skip to preview</a>
      <a href="#document-status" @click="focusSkipTarget">Skip to status</a>
    </nav>

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
            <span class="tab-group-title">
              <span>{{ group.label }}</span>
              <small>{{ group.documents.length }}</small>
            </span>
            <button class="tab-icon-button" type="button" aria-label="Close tab group" title="Close tab group" @click="closeTabGroup(group)">
              <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
                <path v-for="path in toolbarIconPaths('close')" :key="path" :d="path"></path>
              </svg>
            </button>
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
            <button
              class="tab-main"
              type="button"
              :aria-label="`${document.dirty ? 'Unsaved ' : ''}${document.title}`"
              @click="activate(document.id)"
            >
              <span v-if="document.dirty" class="tab-dirty" aria-hidden="true"></span>
              <span>{{ document.title }}</span>
            </button>
            <button
              class="tab-icon-button"
              :class="{ active: document.pinned }"
              type="button"
              :aria-label="document.pinned ? 'Unpin document' : 'Pin document'"
              :title="document.pinned ? 'Unpin document' : 'Pin document'"
              @click="store.togglePin(document.id)"
            >
              <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
                <path v-for="path in toolbarIconPaths('pin')" :key="path" :d="path"></path>
              </svg>
            </button>
            <button class="tab-icon-button" type="button" aria-label="Close document" title="Close document" @click="closeDocument(document.id)">
              <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
                <path v-for="path in toolbarIconPaths('close')" :key="path" :d="path"></path>
              </svg>
            </button>
          </div>
        </section>
      </section>

      <section class="window-meta" aria-label="Document status">
        <span role="status" class="release-badge" :class="releaseStatusClass" :aria-label="`Release status ${releaseStatus}`">{{ releaseStatus }}</span>
        <span v-if="store.gitStatus?.inside_repo">{{ store.gitStatus.branch || "detached" }}{{ store.gitStatus.dirty ? " dirty" : " clean" }}</span>
      </section>
    </header>

    <nav id="main-commands" class="command-bar" aria-label="Main commands" tabindex="-1">
      <section v-for="row in commandToolbarRows" :key="row.id" class="command-toolbar-row" :aria-label="`${row.label} toolbar`">
        <span class="command-toolbar-title">{{ row.label }}</span>
        <section v-for="group in row.groups" :key="group.id" class="command-group" :aria-label="`${group.label} commands`">
          <span class="command-group-label">{{ group.label }}</span>
          <div class="command-group-actions">
            <button
              v-for="action in group.actions"
              :key="action.id"
              type="button"
              class="icon-command"
              :class="{ primary: action.primary }"
              :disabled="action.disabled"
              :aria-label="action.label"
              :title="action.title || action.label"
              @click="runCommandBarAction(action)"
            >
              <span class="command-icon" aria-hidden="true">
                <svg viewBox="0 0 24 24" focusable="false">
                  <path v-for="path in toolbarIconPaths(action.icon)" :key="path" :d="path"></path>
                </svg>
              </span>
              <span class="command-label">{{ action.label }}</span>
            </button>
          </div>
        </section>
      </section>
      <section class="command-toolbar-row command-toolbar-row-view" aria-label="View toolbar">
        <span class="command-toolbar-title">View</span>
        <label class="compact-field">
          <span>Mode</span>
          <select v-model="store.mode" aria-label="View mode">
            <option value="split">Split</option>
            <option value="source">Source</option>
            <option value="preview">Preview</option>
            <option value="focus">Focus</option>
            <option value="export">Export</option>
            <option value="review">Review</option>
            <option value="presentation">Presentation</option>
          </select>
        </label>
        <label class="compact-field">
          <span>Panel</span>
          <select v-model="store.sidebar" aria-label="Sidebar panel">
            <option value="files">Files</option>
            <option value="outline">Outline</option>
            <option value="diagnostics">Diagnostics</option>
            <option value="tables">Tables</option>
            <option value="templates">Templates</option>
            <option value="references">References</option>
            <option value="exports">Exports</option>
            <option value="versioning">Versioning</option>
            <option value="review">Review</option>
            <option value="settings">Settings</option>
          </select>
        </label>
        <label class="compact-field">
          <span>Buttons</span>
          <select v-model="store.toolbarDisplay" aria-label="Toolbar button display">
            <option value="both">Icons and text</option>
            <option value="icons">Icons only</option>
            <option value="text">Text only</option>
          </select>
        </label>
        <label class="compact-field compact-field-range">
          <span>Text</span>
          <input
            v-model.number="store.toolbarTextSize"
            aria-label="Toolbar text size"
            type="range"
            min="9"
            max="15"
            step="1"
          />
          <output aria-label="Current toolbar text size">{{ store.toolbarTextSize }}px</output>
        </label>
      </section>
    </nav>

    <section v-if="externalTransformTrustPrompts.length" class="trust-prompt" aria-label="External transform trust prompts">
      <article v-for="prompt in externalTransformTrustPrompts" :key="prompt.name" class="trust-prompt-item">
        <div>
          <strong>{{ prompt.name }} transform</strong>
          <span>{{ prompt.path }}</span>
          <small>{{ prompt.inputMode }} | {{ prompt.securitySummary }}</small>
        </div>
        <div class="trust-prompt-actions">
          <button type="button" @click="trustTransformEngine(prompt.name)">Trust</button>
          <button type="button" @click="reviewTransformEngineSettings(prompt.name)">Settings</button>
        </div>
      </article>
    </section>

    <section v-if="store.missingWorkspaceFiles.length" class="restore-warning" aria-label="Missing restored documents">
      <strong>Missing restored documents</strong>
      <p>These files were skipped during workspace restore.</p>
      <ul>
        <li v-for="path in store.missingWorkspaceFiles" :key="path">{{ path }}</li>
      </ul>
    </section>

    <main id="document-workspace" ref="workspacePane" class="workspace" :class="`mode-${store.mode}`" :style="workspaceStyle" tabindex="-1">
      <aside id="document-sidebar" class="sidebar" aria-label="Document workspace" tabindex="-1">
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
          <h2>Outline <small>{{ outlineHeadings.length }}</small></h2>
          <p v-if="!outlineHeadings.length" class="sidebar-hint">Add Markdown headings to build a document outline.</p>
          <button
            v-for="heading in outlineHeadings"
            :key="`${heading.line}-${heading.anchor}`"
            class="outline-row"
            :style="{ paddingLeft: `${heading.level * 10}px` }"
            type="button"
            :aria-label="`Go to ${heading.text}, line ${heading.line}`"
            @click="goToSourceTarget(heading)"
          >
            <span>{{ heading.text }}</span>
            <small>Line {{ heading.line }}</small>
          </button>
        </template>

        <template v-else-if="store.sidebar === 'diagnostics'">
          <h2>Diagnostics</h2>
          <section role="list" aria-label="Compiler diagnostics">
            <article
              v-for="diagnostic in active.compile?.diagnostics || []"
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
              <button type="button" :disabled="tableDraftHasErrors" @click="applyTableDraft">{{ isNewTableDraft ? "Insert table" : "Apply" }}</button>
              <button type="button" @click="cancelTableDraft">Cancel table edit</button>
              <button type="button" @click="addTableRow">Add row</button>
              <button type="button" @click="addTableColumn">Add column</button>
              <button type="button" @click="addTableTotalsRow">Add totals row</button>
              <button type="button" @click="addTableFormulaRow('AVG')">AVG row</button>
              <button type="button" @click="addTableFormulaRow('MIN')">MIN row</button>
              <button type="button" @click="addTableFormulaRow('MAX')">MAX row</button>
              <button type="button" @click="addTableFormulaRow('COUNT')">COUNT row</button>
            </div>
            <section class="table-formula-builder" aria-label="Table formula builder">
              <label>
                Function
                <select v-model="tableFormulaFunction">
                  <option value="SUM">SUM</option>
                  <option value="AVG">AVG</option>
                  <option value="MIN">MIN</option>
                  <option value="MAX">MAX</option>
                  <option value="COUNT">COUNT</option>
                </select>
              </label>
              <label>
                Target
                <select v-model.number="tableFormulaTargetColumn">
                  <option v-for="option in tableFormulaTargetColumns" :key="option.index" :value="option.index">
                    {{ option.label }}
                  </option>
                </select>
              </label>
              <label>
                From row
                <input v-model.number="tableFormulaStartRow" type="number" min="1" :max="tableDataRowCount" />
              </label>
              <label>
                To row
                <input v-model.number="tableFormulaEndRow" type="number" min="1" :max="tableDataRowCount" />
              </label>
              <label>
                Label
                <input v-model="tableFormulaLabel" />
              </label>
              <output>{{ tableFormulaPreview || "-" }}</output>
              <button type="button" :disabled="!tableFormulaPreview" @click="appendCustomTableFormulaRow">Add formula row</button>
            </section>
            <section class="table-span-builder" aria-label="Merged table cells">
              <label>
                Cell
                <select v-model="selectedTableSpanCell">
                  <option v-for="option in tableSpanCellOptions" :key="option.value" :value="option.value">
                    {{ option.label }}
                  </option>
                </select>
              </label>
              <label>
                Columns
                <input v-model.number="tableSpanColspan" type="number" min="1" :max="tableSpanMaxColspan" />
              </label>
              <label>
                Rows
                <input v-model.number="tableSpanRowspan" type="number" min="1" :max="tableSpanMaxRowspan" />
              </label>
              <output>{{ tableSpanPreview || "-" }}</output>
              <button type="button" :disabled="!tableSpanPreview" @click="applyTableCellSpan">Merge cell</button>
              <button type="button" @click="clearTableCellSpan">Clear merge</button>
            </section>
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
            <section v-if="tableDraftIssues.length" class="table-issues" aria-label="Table validation">
              <p v-for="issue in tableDraftIssues" :key="issue.message" :class="issue.severity">{{ issue.message }}</p>
            </section>
            <div
              class="table-editor-grid"
              role="group"
              aria-label="Table editor grid"
              :style="{ gridTemplateColumns: `220px repeat(${tableDraft.headers.length}, minmax(132px, 1fr)) 44px` }"
            >
              <span></span>
              <input
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`header-${columnIndex}`"
                v-model="tableDraft.headers[columnIndex]"
                :aria-label="tableHeaderLabel(columnIndex)"
              />
              <span></span>
              <span>Align</span>
              <select
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`align-${columnIndex}`"
                v-model="tableDraft.alignments[columnIndex]"
                :aria-label="`Column ${spreadsheetColumnName(columnIndex + 1)} alignment`"
              >
                <option value="left">Left</option>
                <option value="center">Center</option>
                <option value="right">Right</option>
              </select>
              <span></span>
              <span>Format</span>
              <select
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`format-${columnIndex}`"
                v-model="tableDraft.formats[columnIndex]"
                :aria-label="`Column ${spreadsheetColumnName(columnIndex + 1)} format`"
              >
                <option value="text">Text</option>
                <option value="number">Number</option>
                <option value="currency">Currency</option>
                <option value="percent">Percent</option>
                <option value="date">Date</option>
              </select>
              <span></span>
              <span>Sort</span>
              <span
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`sort-${columnIndex}`"
                class="column-actions"
                role="group"
                :aria-label="`Sort controls for column ${spreadsheetColumnName(columnIndex + 1)}`"
              >
                <button type="button" :aria-label="`Sort column ${spreadsheetColumnName(columnIndex + 1)} ascending`" @click="sortTableRows(columnIndex, 'asc')">Asc</button>
                <button type="button" :aria-label="`Sort column ${spreadsheetColumnName(columnIndex + 1)} descending`" @click="sortTableRows(columnIndex, 'desc')">Desc</button>
              </span>
              <span></span>
              <template v-for="(row, rowIndex) in tableDraft.rows" :key="`row-${rowIndex}`">
                <span class="row-actions" role="group" :aria-label="`Row ${rowIndex + 1} controls`">
                  <button type="button" :disabled="rowIndex === 0" :aria-label="`Move row ${rowIndex + 1} up`" @click="moveTableRow(rowIndex, -1)">Up</button>
                  <button type="button" :disabled="rowIndex === tableDraft.rows.length - 1" :aria-label="`Move row ${rowIndex + 1} down`" @click="moveTableRow(rowIndex, 1)">Down</button>
                  <button type="button" :aria-label="`Copy row ${rowIndex + 1}`" @click="duplicateTableRow(rowIndex)">Copy</button>
                  <button type="button" :aria-label="`Remove row ${rowIndex + 1}`" @click="removeTableRow(rowIndex)">Remove</button>
                </span>
                <input
                  v-for="(_, columnIndex) in tableDraft.headers"
                  :key="`cell-${rowIndex}-${columnIndex}`"
                  v-model="row[columnIndex]"
                  :class="{ 'formula-cell': isFormulaCell(row[columnIndex]) }"
                  :aria-label="tableCellLabel(rowIndex, columnIndex)"
                />
                <span></span>
              </template>
              <span>Totals</span>
              <output
                v-for="(total, columnIndex) in tableColumnTotals"
                :key="`total-${columnIndex}`"
                :aria-label="tableTotalLabel(columnIndex)"
              >
                {{ total || "-" }}
              </output>
              <span></span>
              <span>Move column</span>
              <span
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`move-col-${columnIndex}`"
                class="column-actions"
                role="group"
                :aria-label="`Move controls for column ${spreadsheetColumnName(columnIndex + 1)}`"
              >
                <button type="button" :disabled="columnIndex === 0" :aria-label="`Move column ${spreadsheetColumnName(columnIndex + 1)} left`" @click="moveTableColumn(columnIndex, -1)">Left</button>
                <button type="button" :disabled="columnIndex === tableDraft.headers.length - 1" :aria-label="`Move column ${spreadsheetColumnName(columnIndex + 1)} right`" @click="moveTableColumn(columnIndex, 1)">Right</button>
              </span>
              <span></span>
              <span>Duplicate column</span>
              <button
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`duplicate-col-${columnIndex}`"
                type="button"
                :aria-label="`Copy column ${spreadsheetColumnName(columnIndex + 1)}`"
                @click="duplicateTableColumn(columnIndex)"
              >
                Copy
              </button>
              <span></span>
              <span>Remove column</span>
              <button
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`remove-col-${columnIndex}`"
                type="button"
                :aria-label="`Remove column ${spreadsheetColumnName(columnIndex + 1)}`"
                @click="removeTableColumn(columnIndex)"
              >
                Remove
              </button>
              <span></span>
            </div>
            <label class="table-preview">
              Markdown preview
              <textarea :value="tableDraftMarkdownPreview" rows="7" readonly></textarea>
            </label>
          </template>
          <p v-else>No Markdown table selected.</p>
        </template>

        <template v-else-if="store.sidebar === 'templates'">
          <h2>Templates <small>{{ filteredTransformTemplates.length }}</small></h2>
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
        </template>

        <template v-else-if="store.sidebar === 'references'">
          <h2>References</h2>
          <label>
            Citation style
            <select :value="citationStyle" @change="setCitationStyle(eventValue($event))">
              <option value="title">Title</option>
              <option value="author-year">Author-year</option>
              <option value="key">Key</option>
              <option value="numeric">Numeric</option>
              <option value="apa">APA</option>
              <option value="chicago-author-date">Chicago author-date</option>
              <option value="harvard">Harvard</option>
              <option value="ieee">IEEE</option>
              <option value="vancouver">Vancouver</option>
              <option value="nature">Nature</option>
              <option value="ama">AMA</option>
            </select>
          </label>
          <h3>Citations</h3>
          <section class="reference-manager" aria-label="Citation manager">
            <div class="reference-actions">
              <button type="button" @click="insertBlock(bibliographySnippet)">Insert bibliography marker</button>
              <button type="button" @click="insertBlock(bibliographyTemplateSnippet)">Insert BibTeX template</button>
              <button type="button" :disabled="!missingCitationKeys.length" @click="insertMissingCitationStubs">Insert missing key stubs</button>
            </div>
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
            <div class="reference-actions">
              <button type="button" @click="insertBlock(glossarySectionSnippet)">Insert generated glossary</button>
              <button type="button" @click="insertBlock(glossarySnippet)">Insert glossary definitions</button>
              <button type="button" @click="store.exportDefaults.includeGlossary = true">Include glossary in exports</button>
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
            <div class="reference-actions">
              <button type="button" @click="insertBlock(indexSnippet)">Insert generated index</button>
              <button type="button" @click="setFrontMatterField('index', 'true')">Enable front matter index</button>
            </div>
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
          <button
            v-for="reference in active.compile?.semantic.cross_references || []"
            :key="`${reference.key}-${reference.line}`"
            class="outline-row"
            type="button"
            @click="goToCrossReference(reference)"
          >
            {{ reference.key }}: {{ reference.resolved ? "resolved" : "missing" }}
          </button>
          <h3>Labels</h3>
          <p v-for="label in active.compile?.semantic.labels || []" :key="label">{{ label }}</p>
        </template>

        <template v-else-if="store.sidebar === 'exports'">
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
            </select>
          </label>
          <section v-if="store.exportTarget === 'html'" class="export-target-options" aria-label="HTML export options">
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
          <label><input v-model="store.exportDefaults.includeComments" type="checkbox" /> Include comments</label>
          <label><input v-model="store.exportDefaults.includeProvenance" type="checkbox" /> Include AI provenance</label>
          <label><input v-model="store.exportDefaults.includeGlossary" type="checkbox" /> Include glossary</label>
          <label><input v-model="store.exportDefaults.includeAgenda" type="checkbox" /> PPTX agenda</label>
          <div class="export-actions">
            <button class="template-action-primary" type="button" :disabled="store.exportBusy" @click="exportDocumentAs('html')">
              <span class="button-icon" aria-hidden="true">
                <svg viewBox="0 0 24 24" focusable="false">
                  <path v-for="path in toolbarIconPaths('html')" :key="path" :d="path"></path>
                </svg>
              </span>
              Export HTML
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
          <h3>Snapshots</h3>
          <button type="button" @click="snapshotActive">Create snapshot</button>
          <button type="button" @click="store.listSnapshots">Refresh snapshots</button>
          <article v-for="snapshot in store.snapshots" :key="`version-${snapshot.snapshot_path}`" class="snapshot-row">
            <p>{{ snapshot.label || "snapshot" }}</p>
            <small>{{ snapshot.created_at || snapshot.snapshot_path }}</small>
            <small>{{ snapshot.document_version || "unversioned" }} | {{ snapshot.status || "unknown" }} | {{ snapshot.author || "unknown author" }}</small>
            <button type="button" @click="restoreSnapshot(snapshot.snapshot_path)">Restore snapshot</button>
          </article>
        </template>

        <template v-else-if="store.sidebar === 'review'">
          <h2>Review</h2>
          <h3>Summary</h3>
          <article class="snapshot-row">
            <p>{{ reviewSummary.status }} | {{ reviewSummary.unresolved }} unresolved | {{ reviewSummary.resolved }} resolved</p>
            <small>{{ reviewSummary.changeNotes }} change notes | {{ reviewSummary.aiPending }} AI review pending | {{ reviewSummary.aiReviewed }} AI reviewed</small>
          </article>
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
            Document set
            <input :value="String(active.compile?.metadata.documentSet || '')" @change="setFrontMatterField('documentSet', inputValue($event))" />
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
          <label>
            Change note
            <textarea v-model="changeNoteText" rows="3" placeholder="Change summary"></textarea>
          </label>
          <button type="button" @click="insertChangeNote">Add change note</button>
          <h3>Comments</h3>
          <article v-for="comment in active.compile?.semantic.comments || []" :key="String(comment.line)" class="snapshot-row">
            <p>{{ comment.text }}</p>
            <small>Line {{ comment.line }} | {{ comment.state }} | {{ comment.author || "local" }}{{ comment.created_at ? ` | ${comment.created_at}` : "" }}</small>
            <button v-if="comment.state !== 'resolved'" type="button" @click="store.resolveReviewComment(Number(comment.line))">Resolve</button>
          </article>
          <h3>Change notes</h3>
          <article v-for="note in active.compile?.semantic.change_notes || []" :key="`change-${note.line}`" class="snapshot-row">
            <p>{{ note.text }}</p>
            <small>Line {{ note.line }} | {{ note.author || "local" }}{{ note.created_at ? ` | ${note.created_at}` : "" }}</small>
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
          <label>
            Preview theme
            <select v-model="store.previewTheme">
              <option value="match">Match app</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
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
          <label><input v-model="store.wordWrap" type="checkbox" /> Word wrap</label>
          <label><input v-model="store.lineNumbers" type="checkbox" /> Line numbers</label>
          <label><input v-model="store.codeFolding" type="checkbox" /> Code folding</label>
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
          <label>
            Snapshot storage
            <select v-model="store.snapshotStorage">
              <option value="app-data">App data</option>
              <option value="project-local">Project local</option>
            </select>
          </label>
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
              <option value="harvard">Harvard</option>
              <option value="ieee">IEEE</option>
              <option value="vancouver">Vancouver</option>
              <option value="nature">Nature</option>
              <option value="ama">AMA</option>
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
          <h3>Git integration</h3>
          <label><input v-model="store.gitIntegration.enabled" type="checkbox" /> Enable Git status</label>
          <label><input v-model="store.gitIntegration.warnOnDirtyExport" type="checkbox" /> Warn on dirty export</label>
          <h3>AI paste cleanup defaults</h3>
          <label><input v-model="store.aiCleanupDefaults.markAsDraft" type="checkbox" /> Mark as draft</label>
          <label><input v-model="store.aiCleanupDefaults.addProvenance" type="checkbox" /> Add provenance block</label>
          <label><input v-model="store.aiCleanupDefaults.preserveHeadings" type="checkbox" /> Preserve original headings</label>
          <label><input v-model="store.aiCleanupDefaults.convertNumberedLists" type="checkbox" /> Convert numbered lists</label>
          <label><input v-model="store.aiCleanupDefaults.convertTables" type="checkbox" /> Convert tables</label>
          <label><input v-model="store.aiCleanupDefaults.insertCitationTodos" type="checkbox" /> Insert citation TODOs</label>
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
            <small>{{ engine.installationLabel }}</small>
            <small>{{ engine.setupHint }}</small>
            <small>{{ engine.adapterProfile }} Default command: {{ engine.defaultCommand }}</small>
            <small v-if="engine.diagnosticProfile.versionProbe">Version probe: {{ engine.diagnosticProfile.versionProbe }}</small>
            <small v-if="engine.diagnosticProfile.failureHint">Failure hint: {{ engine.diagnosticProfile.failureHint }}</small>
            <small>{{ engine.securitySummary }}</small>
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
        </template>
      </aside>

      <section id="markdown-source" v-show="store.mode !== 'preview' && store.mode !== 'export' && store.mode !== 'presentation'" class="editor-pane" aria-label="Markdown source" tabindex="-1">
        <div ref="editorHost" class="editor-host"></div>
      </section>

      <button
        v-show="paneSplitterVisible"
        class="pane-splitter"
        type="button"
        role="separator"
        aria-label="Resize editor and preview panes"
        aria-orientation="vertical"
        :aria-valuenow="Math.round(store.editorPaneRatio * 100)"
        aria-valuemin="25"
        aria-valuemax="75"
        @pointerdown="startPaneResize"
        @keydown="handlePaneSplitterKeydown"
      ></button>

      <section
        ref="previewPane"
        id="live-preview"
        v-show="store.mode !== 'source' && store.mode !== 'focus'"
        class="preview-pane"
        :data-preview-theme="store.previewTheme"
        aria-label="Live preview"
        tabindex="-1"
        @scroll="syncEditorScrollFromPreview"
      >
        <section v-if="store.mode === 'export'" class="export-preview-summary" aria-label="Export preview summary">
          <div>
            <strong>{{ exportPreviewSummary.targetLabel }}</strong>
            <span>{{ exportPreviewSummary.readinessLabel }}</span>
          </div>
          <p>{{ exportPreviewSummary.manifestLabel }}</p>
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
        <article
          class="preview-document"
          role="document"
          :aria-label="previewDocumentLabel"
          tabindex="0"
          :style="previewDocumentStyle"
          @click="handlePreviewClick"
          v-html="previewHtmlWithDiagnostics"
        ></article>
      </section>
    </main>

    <footer id="document-status" class="status-bar" aria-label="Document status and progress" tabindex="-1">
      <span
        class="status-message"
        role="status"
        aria-live="polite"
        aria-atomic="true"
        :aria-label="`Status message: ${store.statusMessage || 'No status message'}`"
      >
        {{ store.statusMessage }}
      </span>
      <span v-if="store.externalConflict" class="conflict-actions">
        <button type="button" @click="conflictOpen = true">Compare</button>
        <button type="button" @click="store.acceptExternalChanges">Accept external</button>
        <button type="button" @click="store.keepLocalChanges">Keep local</button>
        <button type="button" @click="saveConflictCopy">Save copy</button>
      </span>
      <span class="word-stats" :aria-label="`Document statistics: ${wordStats}`">{{ wordStats }}</span>
      <span
        v-if="watchStatus"
        class="watch-status"
        role="status"
        aria-live="polite"
        aria-atomic="true"
        :aria-label="`File watch status: ${watchStatus}`"
      >
        {{ watchStatus }}
      </span>
      <span
        v-if="store.compileProgress"
        class="compile-actions"
        role="status"
        aria-live="polite"
        aria-atomic="true"
        :aria-label="`Compile progress: ${store.compileProgress}`"
      >
        {{ store.compileProgress }}
        <button type="button" @click="store.cancelActiveCompile">Cancel compile</button>
      </span>
      <span
        v-if="store.exportProgress"
        class="export-progress"
        role="status"
        aria-live="polite"
        aria-atomic="true"
        :aria-label="`Export progress: ${store.exportProgress}`"
      >
        {{ store.exportProgress }}
      </span>
      <span
        v-if="store.lastError"
        class="error"
        role="alert"
        aria-live="assertive"
        aria-atomic="true"
        :aria-label="`Error: ${store.lastError}`"
      >
        {{ store.lastError }}
      </span>
    </footer>

    <section
      v-if="aiPasteOpen"
      ref="aiPasteDialog"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="AI paste cleanup"
      tabindex="-1"
      @keydown="handleModalKeydown('ai-paste', $event)"
    >
      <form class="modal" @submit.prevent="cleanAiPaste">
        <header>
          <h2>Paste from AI Chat</h2>
          <button type="button" aria-label="Close AI paste cleanup" @click="closeAiPaste">x</button>
        </header>
        <section class="compare-grid ai-paste-grid">
          <label>
            Original
            <textarea v-model="aiPasteText" rows="12" placeholder="Paste AI chat output here" data-initial-focus></textarea>
          </label>
          <label>
            Cleaned preview
            <textarea :value="store.aiCleanupPreview?.cleaned_markdown || ''" rows="12" readonly placeholder="Preview cleaned Markdown"></textarea>
          </label>
        </section>
        <label><input v-model="aiMarkAsDraft" type="checkbox" /> Mark as draft</label>
        <label><input v-model="aiAddProvenance" type="checkbox" /> Add provenance block</label>
        <label><input v-model="aiPreserveHeadings" type="checkbox" /> Preserve original headings</label>
        <label><input v-model="aiConvertNumberedLists" type="checkbox" /> Convert numbered lists</label>
        <label><input v-model="aiConvertTables" type="checkbox" /> Convert tables</label>
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

    <section
      v-if="commandPaletteOpen"
      ref="commandPaletteDialog"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
      tabindex="-1"
      @keydown="handleModalKeydown('command-palette', $event)"
    >
      <div class="modal command-modal">
        <header>
          <h2>Command Palette</h2>
          <button type="button" aria-label="Close command palette" @click="closeCommandPalette">x</button>
        </header>
        <input v-model="commandQuery" autofocus data-initial-focus aria-label="Search commands, headings, citations, glossary, and index terms" placeholder="Search commands, headings, citations, glossary, index terms" />
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

    <section
      v-if="conflictOpen && store.externalConflict"
      ref="conflictDialog"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="External file conflict"
      tabindex="-1"
      @keydown="handleModalKeydown('conflict', $event)"
    >
      <div class="modal conflict-modal">
        <header>
          <h2>External Changes</h2>
          <button type="button" aria-label="Close external file conflict" @click="closeConflictDialog">x</button>
        </header>
        <p>{{ store.externalConflict.message }}</p>
        <p class="conflict-path">{{ store.externalConflict.path }}</p>
        <section v-if="rootConflictCanMerge" class="conflict-merge">
          <div class="conflict-toolbar">
            <button type="button" @click="seedConflictMerge('local')">Use local as merge base</button>
            <button type="button" @click="seedConflictMerge('external')">Use external as merge base</button>
            <button type="button" @click="clearConflictMerge">Clear merge</button>
            <button type="button" :disabled="!mergedConflictText.trim()" @click="applyConflictMerge">Apply merged text</button>
          </div>
          <section class="conflict-diff" aria-label="Conflict line diff">
            <div class="conflict-diff-head">Local</div>
            <div class="conflict-diff-head">External</div>
            <template v-for="row in conflictDiffRows" :key="row.key">
              <div
                :class="['conflict-diff-cell', `is-${row.kind}`]"
                role="group"
                :aria-label="conflictDiffCellLabel(row, 'local')"
              >
                <button
                  type="button"
                  :disabled="row.localLine === null || isConflictMergePartSelected(row, 'local')"
                  :aria-label="row.localLine === null ? 'Add local line unavailable' : `Add local line ${row.localLine} to merge`"
                  @click="addConflictMergeLine(row, 'local')"
                >
                  Add
                </button>
                <pre><span>{{ row.localLine || "" }}</span>{{ row.local }}</pre>
              </div>
              <div
                :class="['conflict-diff-cell', `is-${row.kind}`]"
                role="group"
                :aria-label="conflictDiffCellLabel(row, 'external')"
              >
                <button
                  type="button"
                  :disabled="row.externalLine === null || isConflictMergePartSelected(row, 'external')"
                  :aria-label="row.externalLine === null ? 'Add external line unavailable' : `Add external line ${row.externalLine} to merge`"
                  @click="addConflictMergeLine(row, 'external')"
                >
                  Add
                </button>
                <pre><span>{{ row.externalLine || "" }}</span>{{ row.external }}</pre>
              </div>
            </template>
          </section>
          <section class="merge-composition" aria-label="Merge composition">
            <header>
              <strong>Merge composition</strong>
              <span>{{ conflictMergeParts.length }} selected lines</span>
            </header>
            <p v-if="!conflictMergeParts.length" class="sidebar-hint">Add local and external lines in the order they should appear, then review the merged result before applying.</p>
            <ol v-else>
              <li v-for="(part, index) in conflictMergeParts" :key="part.id">
                <span class="merge-source">{{ part.source }} {{ part.line }}</span>
                <code>{{ part.text || "blank line" }}</code>
                <button type="button" :disabled="index === 0" :aria-label="`Move ${part.source} line ${part.line} up`" @click="moveConflictLine(part.id, -1)">Up</button>
                <button type="button" :disabled="index === conflictMergeParts.length - 1" :aria-label="`Move ${part.source} line ${part.line} down`" @click="moveConflictLine(part.id, 1)">Down</button>
                <button type="button" :aria-label="`Remove ${part.source} line ${part.line}`" @click="removeConflictLine(part.id)">Remove</button>
              </li>
            </ol>
          </section>
          <label class="merge-editor">
            Merged result
            <textarea v-model="mergedConflictText" rows="12"></textarea>
          </label>
        </section>
        <section v-else class="compare-grid">
          <article>
            <h3>Local document</h3>
            <pre>{{ conflictDocument.text }}</pre>
          </article>
          <article>
            <h3>Changed file</h3>
            <pre>{{ store.externalConflict.externalText || "Included file changed. Recompile to update the preview." }}</pre>
          </article>
        </section>
        <footer>
          <button type="button" @click="store.keepLocalChanges(); closeConflictDialog()">Keep local</button>
          <button type="button" @click="saveConflictCopy">Save copy</button>
          <button type="button" @click="store.acceptExternalChanges(); closeConflictDialog()">Accept external</button>
        </footer>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch, type CSSProperties } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { EditorState, RangeSetBuilder } from "@codemirror/state";
import { Decoration, EditorView, keymap, lineNumbers, ViewPlugin, type DecorationSet, type ViewUpdate } from "@codemirror/view";
import { addCursorAbove, addCursorBelow, defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { codeFolding, foldAll, foldGutter, foldKeymap, unfoldAll } from "@codemirror/language";
import { markdown } from "@codemirror/lang-markdown";
import { findNext, findPrevious, openSearchPanel, replaceAll, replaceNext, searchKeymap, selectNextOccurrence } from "@codemirror/search";
import { closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import { forceLinting, linter, lintGutter, type Diagnostic as CodeMirrorDiagnostic } from "@codemirror/lint";
import { bibliographyEntryStub, bibliographyStubsForMissingKeys, citationReferenceSnippet } from "./lib/bibliographyManager";
import { buildConflictDiff, type ConflictDiffRow } from "./lib/conflict";
import { createDebouncedTextCommit } from "./lib/debounce";
import { markdownListContinuation } from "./lib/markdownEditing";
import {
  blankCustomTransformTemplate,
  builtinTransformTemplates,
  createCustomTransformTemplateId,
  transformTemplateFillFields,
  transformTemplateCategories,
  transformTemplateKinds,
  transformTemplateMarkdown,
  type CustomTransformTemplate,
  type TransformTemplate,
} from "./lib/transformTemplates";
import { SUPPORTED_CITATION_STYLES } from "./lib/workspacePersistence";
import {
  appendConflictMergePart,
  moveConflictMergePart,
  removeConflictMergePart,
  renderConflictMergeParts,
  type ConflictMergePart,
  type ConflictMergeSource,
} from "./lib/workflows";
import {
  formatTableTotal,
  inferTableFormat,
  isFormulaCell,
  isTableSummaryRow,
  normalizeTableDraft,
  padAlignments,
  padTableRow,
  parseMarkdownTables,
  parseTablePaste,
  parseTableCellSpan,
  serializeMarkdownTable,
  setTableCellSpan,
  sortTableDraftRows,
  spreadsheetColumnName,
  tableColumnRange,
  validateTableDraft,
  type TableDraft,
  type TableFormulaFunction,
  type TableSortDirection,
} from "./lib/tables";
import { useDocumentsStore } from "./stores/documents";
import type { AiCleanupResponse, DocumentBlock, DocumentDiagnostic, OpenDocument } from "./types";

const store = useDocumentsStore();
type ExportTarget = typeof store.exportTarget;
type WindowTitleTarget = {
  setTitle(title: string): Promise<void>;
};

function getWindowTitleTarget(): WindowTitleTarget | null {
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

const appWindow = getWindowTitleTarget();
const editorHost = ref<HTMLElement | null>(null);
const workspacePane = ref<HTMLElement | null>(null);
const previewPane = ref<HTMLElement | null>(null);
const aiPasteDialog = ref<HTMLElement | null>(null);
const commandPaletteDialog = ref<HTMLElement | null>(null);
const conflictDialog = ref<HTMLElement | null>(null);
let editorView: EditorView | null = null;
const previewTextCommit = createDebouncedTextCommit((text) => store.updateText(text), {
  setTimeout: (callback, delayMs) => window.setTimeout(callback, delayMs),
  clearTimeout: (handle) => window.clearTimeout(handle),
});
let autosaveHandle = 0;
let autoSnapshotHandle = 0;
let scrollPersistHandle = 0;
let lastAutoSnapshotSignature = "";
let syncingScroll = false;
let restoringScroll = false;
let modalReturnFocus: HTMLElement | null = null;
const aiPasteOpen = ref(false);
const aiPasteText = ref("");
const aiInsertMode = ref<"insert" | "quote" | "replace" | "appendix" | "selection" | "section">("insert");
const aiAddProvenance = ref(true);
const aiMarkAsDraft = ref(true);
const aiInsertCitationTodos = ref(true);
const aiPreserveHeadings = ref(false);
const aiConvertNumberedLists = ref(true);
const aiConvertTables = ref(true);
const aiPreviewBusy = ref(false);
const aiPreviewSignature = ref("");
const desktopWorkflowSmokeActive = ref(false);
const commandPaletteOpen = ref(false);
const conflictOpen = ref(false);
const mergedConflictText = ref("");
const conflictMergeParts = ref<ConflictMergePart[]>([]);
const commandQuery = ref("");
const reviewCommentText = ref("");
const changeNoteText = ref("");
const selectedTableIndex = ref(0);
const tablePasteText = ref("");
const tableDraft = ref<TableDraft | null>(null);
const isNewTableDraft = ref(false);
const tableFormulaFunction = ref<TableFormulaFunction>("SUM");
const tableFormulaTargetColumn = ref(1);
const tableFormulaStartRow = ref(1);
const tableFormulaEndRow = ref(2);
const tableFormulaLabel = ref("Total");
const tableSpanRow = ref(0);
const tableSpanColumn = ref(0);
const tableSpanColspan = ref(1);
const tableSpanRowspan = ref(1);
const templateQuery = ref("");
const templateCategory = ref("all");
const templateTransform = ref("all");
const customTemplateDraft = ref<CustomTransformTemplate>(blankCustomTransformTemplate());
const editingCustomTemplateId = ref("");
const draggedTabId = ref("");
const exportProfileName = ref("Client delivery");

type FigureCropPosition = "center" | "top" | "bottom" | "left" | "right" | "top-left" | "top-right" | "bottom-left" | "bottom-right";

type ClipboardItemLike = {
  types: string[];
  getType: (type: string) => Promise<Blob>;
};

type RichClipboard = Clipboard & {
  read?: () => Promise<ClipboardItemLike[]>;
};

interface ClipboardTextRead {
  text: string;
  kind: "rich" | "plain";
}

interface DocumentTabGroup {
  key: string;
  label: string;
  title: string;
  documents: OpenDocument[];
}

interface FigureListItem {
  id?: string | null;
  src?: string | null;
  alt?: string | null;
  caption?: string | null;
  fit?: string | null;
  position?: string | null;
  line: number;
  end_line: number;
  source_file?: string | null;
}

interface TransformTrustPrompt {
  name: string;
  path: string;
  inputMode: string;
  securitySummary: string;
}

interface IncludeGraphItem {
  parent: string;
  child: string;
  depth: number;
  parentLabel: string;
  childLabel: string;
  commandLabel: string;
}

interface PreviewDiagnosticItem extends DocumentDiagnostic {
  generatedLine: number;
}

interface TransformPreviewItem {
  id: string;
  name: string;
  sourceFile?: string | null;
  sourceLine?: number | null;
  endSourceLine?: number | null;
  diagnostics: DocumentDiagnostic[];
  outputLabel: string;
  cacheLabel: string;
  locationLabel: string;
}

type ToolbarIconName =
  | "new"
  | "open"
  | "folder"
  | "workspace"
  | "save"
  | "saveAs"
  | "revert"
  | "rename"
  | "duplicate"
  | "reveal"
  | "snapshot"
  | "export"
  | "ai"
  | "commands"
  | "bold"
  | "italic"
  | "code"
  | "fence"
  | "heading"
  | "link"
  | "table"
  | "figure"
  | "calc"
  | "templates"
  | "equation"
  | "toc"
  | "outline"
  | "comment"
  | "find"
  | "previous"
  | "next"
  | "fold"
  | "unfold"
  | "html"
  | "pin"
  | "close";

interface CommandBarAction {
  id: string;
  label: string;
  title?: string;
  icon: ToolbarIconName;
  primary?: boolean;
  disabled?: boolean;
  run: () => unknown;
}

interface CommandBarGroup {
  id: string;
  label: string;
  actions: CommandBarAction[];
}

interface CommandToolbarRow {
  id: string;
  label: string;
  groups: CommandBarGroup[];
}

const toolbarIconPathMap: Record<ToolbarIconName, string[]> = {
  new: ["M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z", "M14 3v6h6", "M12 12v6", "M9 15h6"],
  open: ["M4 6h6l2 2h8v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z", "M8 14h8", "M12 10v8"],
  folder: ["M3 7h7l2 2h9l-2 10H5z", "M3 7v10a2 2 0 0 0 2 2"],
  workspace: ["M4 5h7v7H4z", "M13 5h7v7h-7z", "M4 14h7v5H4z", "M13 14h7v5h-7z"],
  save: ["M5 4h12l2 2v14H5z", "M8 4v6h8V4", "M8 16h8"],
  saveAs: ["M5 4h11l3 3v13H5z", "M8 4v6h7V4", "M8 16h5", "M15 17l3 3", "M18 14l-3 3"],
  revert: ["M9 7H4v5", "M4 12a8 8 0 1 0 2.3-5.7L4 8.5"],
  rename: ["M4 20h4l11-11a2.8 2.8 0 0 0-4-4L4 16z", "M13 6l4 4"],
  duplicate: ["M8 8h11v11H8z", "M5 16H4a1 1 0 0 1-1-1V5h10v1"],
  reveal: ["M3 12s3.5-6 9-6 9 6 9 6-3.5 6-9 6-9-6-9-6z", "M12 9a3 3 0 1 1 0 6 3 3 0 0 1 0-6z"],
  snapshot: ["M4 7h3l2-2h6l2 2h3v12H4z", "M12 10a4 4 0 1 1 0 8 4 4 0 0 1 0-8z"],
  export: ["M12 3v12", "M7 8l5-5 5 5", "M5 15v4h14v-4"],
  ai: ["M12 3l1.6 4.4L18 9l-4.4 1.6L12 15l-1.6-4.4L6 9l4.4-1.6z", "M5 14l.8 2.2L8 17l-2.2.8L5 20l-.8-2.2L2 17l2.2-.8z"],
  commands: ["M4 7h16", "M4 12h16", "M4 17h10", "M17 15l3 2-3 2"],
  bold: ["M8 5h5a3 3 0 0 1 0 6H8z", "M8 11h6a3 3 0 0 1 0 6H8z", "M8 5v12"],
  italic: ["M10 5h8", "M6 19h8", "M14 5l-4 14"],
  code: ["M9 18l-6-6 6-6", "M15 6l6 6-6 6"],
  fence: ["M5 6h14", "M5 12h14", "M5 18h14"],
  heading: ["M5 5v14", "M19 5v14", "M5 12h14", "M14 19h5"],
  link: ["M10 13a5 5 0 0 0 7 0l2-2a5 5 0 0 0-7-7l-1 1", "M14 11a5 5 0 0 0-7 0l-2 2a5 5 0 0 0 7 7l1-1"],
  table: ["M4 5h16v14H4z", "M4 10h16", "M4 15h16", "M10 5v14", "M15 5v14"],
  figure: ["M4 5h16v14H4z", "M8 13l3-3 3 4 2-2 4 5", "M8 8h.01"],
  calc: ["M7 4h10v16H7z", "M10 8h4", "M10 12h1", "M14 12h1", "M10 16h1", "M14 16h1"],
  templates: ["M4 5h7v6H4z", "M13 5h7v6h-7z", "M4 13h7v6H4z", "M13 13h7v6h-7z"],
  equation: ["M4 8h16", "M4 16h16", "M8 12h8"],
  toc: ["M5 7h2", "M10 7h9", "M5 12h2", "M10 12h9", "M5 17h2", "M10 17h9"],
  outline: ["M5 5h14", "M5 10h10", "M5 15h14", "M5 20h8"],
  comment: ["M5 5h14v10H9l-4 4z"],
  find: ["M11 5a6 6 0 1 1 0 12 6 6 0 0 1 0-12z", "M16 16l4 4"],
  previous: ["M15 6l-6 6 6 6"],
  next: ["M9 6l6 6-6 6"],
  fold: ["M5 7h14", "M8 12h8", "M11 17h2"],
  unfold: ["M5 7h14", "M5 12h14", "M5 17h14"],
  html: ["M8 8l-4 4 4 4", "M16 8l4 4-4 4", "M14 5l-4 14"],
  pin: ["M14 4l6 6-4 1-4 6-1 3-1-1-3-3-3-3-1-1 3-1 6-4z", "M9 15l-5 5"],
  close: ["M6 6l12 12", "M18 6L6 18"],
};

const tableSnippet = `| Item | Value |\n| --- | ---: |\n| Revenue | 125000 |\n`;
const codeFenceSnippet = "```markdown\n\n```\n";
const figureCropPositions: FigureCropPosition[] = ["center", "top", "bottom", "left", "right", "top-left", "top-right", "bottom-left", "bottom-right"];
const figureCropPositionGrid: Record<FigureCropPosition, { x: -1 | 0 | 1; y: -1 | 0 | 1 }> = {
  center: { x: 0, y: 0 },
  top: { x: 0, y: -1 },
  bottom: { x: 0, y: 1 },
  left: { x: -1, y: 0 },
  right: { x: 1, y: 0 },
  "top-left": { x: -1, y: -1 },
  "top-right": { x: 1, y: -1 },
  "bottom-left": { x: -1, y: 1 },
  "bottom-right": { x: 1, y: 1 },
};
const figureCropPositionPoints: Record<FigureCropPosition, { x: number; y: number }> = {
  center: { x: 50, y: 50 },
  top: { x: 50, y: 12 },
  bottom: { x: 50, y: 88 },
  left: { x: 12, y: 50 },
  right: { x: 88, y: 50 },
  "top-left": { x: 12, y: 12 },
  "top-right": { x: 88, y: 12 },
  "bottom-left": { x: 12, y: 88 },
  "bottom-right": { x: 88, y: 88 },
};
const calcSnippet = "```calc\nrevenue = 125000\ncost = 74000\nprofit = revenue - cost\n```\n";
const equationSnippet = "$$\nE = mc^2\n$$ {#eq:energy}\n";
const tocSnippet = "[TOC]\n";
const indexSnippet = "[INDEX]\n";
const bibliographySnippet = "[BIBLIOGRAPHY]\n";
const bibliographyTemplateSnippet = "```bibtex\n@misc{source2026,\n  title = {Source title},\n  author = {Author},\n  year = {2026}\n}\n```\n";
const listOfFiguresSnippet = "[LIST_OF_FIGURES]\n";
const listOfTablesSnippet = "[LIST_OF_TABLES]\n";
const glossarySectionSnippet = "[GLOSSARY]\n";
const glossarySnippet = "```glossary\nARR: Annual recurring revenue.\nCAC: Customer acquisition cost.\n```\n";
const layoutSnippet = "```layout\ncolumns: 2\nsection: market-analysis\n```\n";
const commentSnippet = "<!-- comment: unresolved | author: local | at: 2026-05-18T00:00:00Z | Review note. -->\n";
const aiSnippet =
  "```ai-source\nprovider: OpenAI\nmodel: ChatGPT\ndate: 2026-05-18\npromptSummary: \nreviewedBy: \nreviewedAt: \nstatus: needs-review\n```\n";
const releaseStatuses = ["draft", "in-review", "approved", "published", "archived"];
const nativeMenuExportTargets: Record<string, ExportTarget> = {
  "neditor-export-html": "html",
  "neditor-export-pdf": "pdf",
  "neditor-export-docx": "docx",
  "neditor-export-pptx": "pptx",
  "neditor-export-markdown-bundle": "markdown-bundle",
  "neditor-export-blog": "blog",
  "neditor-export-substack": "substack",
  "neditor-export-latex": "latex",
  "neditor-export-google-docs": "google-docs",
};
let unlistenNativeMenuCommand: UnlistenFn | null = null;

const active = computed(() => store.activeDocument);
const activeExportProfile = computed(() => store.exportProfiles.find((profile) => profile.id === store.activeExportProfileId) || null);
const exportProfileSummary = computed(() => {
  const profile = activeExportProfile.value;
  if (!profile) return "";
  const enabled = [
    profile.exportDefaults.includeManifest && "manifest",
    profile.exportDefaults.coverPage && "cover",
    profile.exportDefaults.pageNumbers && "page numbers",
    profile.exportDefaults.includeComments && "comments",
    profile.exportDefaults.includeProvenance && "AI provenance",
    profile.exportDefaults.includeGlossary && "glossary",
  ].filter(Boolean);
  const brand = profile.brandProfileDefaults.name || profile.brandProfileDefaults.color;
  return `${profile.exportTarget.toUpperCase()} / ${profile.exportDefaults.layoutPreset}${brand ? ` / ${brand}` : ""}${enabled.length ? ` / ${enabled.join(", ")}` : ""}`;
});
const previewDocumentStyle = computed(() => ({
  fontFamily: store.previewFont,
  fontSize: `${clampUiFontSize(store.previewFontSize)}px`,
  lineHeight: String(clampUiLineHeight(store.previewLineHeight)),
}));
const appShellStyle = computed(() => ({
  "--toolbar-font-size": `${clampToolbarTextSize(store.toolbarTextSize)}px`,
}));
const previewDocumentLabel = computed(() => {
  const title = active.value.compile?.semantic.title || active.value.title || "Untitled document";
  const status = active.value.compile?.semantic.status || "draft";
  return `Rendered preview for ${title}, ${status}`;
});
const previewDiagnostics = computed<PreviewDiagnosticItem[]>(() => {
  const diagnostics = active.value.compile?.diagnostics || [];
  return diagnostics
    .filter((diagnostic) => Boolean(diagnostic.line))
    .map((diagnostic) => ({
      ...diagnostic,
      generatedLine: previewGeneratedLineForDiagnostic(diagnostic),
    }))
    .sort((left, right) => left.generatedLine - right.generatedLine || (left.line || 0) - (right.line || 0));
});
const previewHtmlWithDiagnostics = computed(() => inlinePreviewDiagnostics(active.value.compile?.html || "", previewDiagnostics.value));
const exportPreviewSummary = computed(() => {
  const manifest = store.exportReadiness?.manifest || active.value.compile?.export_manifest;
  const readiness = store.exportReadiness;
  const options = [
    store.exportDefaults.includeManifest ? "Manifest" : "No manifest",
    store.exportDefaults.coverPage ? "Cover" : "No cover",
    store.exportDefaults.pageNumbers ? "Page numbers" : "No page numbers",
    store.exportDefaults.includeComments ? "Comments" : "No comments",
    store.exportDefaults.includeProvenance ? "AI provenance" : "No AI provenance",
    store.exportDefaults.includeGlossary ? "Glossary" : "No glossary",
    ...(store.exportTarget === "html" && store.exportDefaults.htmlLanguage ? [`HTML ${store.exportDefaults.htmlLanguage}`] : []),
    ...(store.exportTarget === "html" && store.exportDefaults.canonicalUrl ? ["Canonical URL"] : []),
    store.exportDefaults.layoutPreset,
  ];
  return {
    targetLabel: `${store.exportTarget.toUpperCase()} export preview`,
    readinessLabel: readiness ? (readiness.ready ? "ready" : `${readiness.error_count} errors, ${readiness.warning_count} warnings`) : "readiness not run",
    manifestLabel: manifest
      ? `${manifest.included_files.length} included files, ${manifest.transform_artifacts.length} transform artifacts, ${manifest.layout_sections.length} layout sections`
      : "No export manifest yet",
    options,
  };
});
const transformPreviewItems = computed<TransformPreviewItem[]>(() =>
  (active.value.compile?.transform_artifacts || []).map((artifact, index) => {
    const locationLabel = artifact.source_line
      ? `${artifact.source_file || active.value.path || "document"}: line ${artifact.source_line}`
      : "";
    return {
      id: artifact.id || `${artifact.name}-${index}`,
      name: artifact.name,
      sourceFile: artifact.source_file || null,
      sourceLine: artifact.source_line || null,
      endSourceLine: artifact.end_source_line || artifact.source_line || null,
      diagnostics: artifact.diagnostics || [],
      outputLabel: `${artifact.output_kind} via ${artifact.execution_kind || "native"}${artifact.duration_ms ? ` in ${artifact.duration_ms} ms` : ""}`,
      cacheLabel: artifact.cache_key ? `Cache ${artifact.cache_key}` : `Output ${artifact.output_hash}`,
      locationLabel,
    };
  }),
);
const workspaceStyle = computed(() => ({ "--editor-ratio": String(store.editorPaneRatio) }));
const paneSplitterVisible = computed(() => !["source", "focus", "preview", "export"].includes(store.mode));
const wordStats = computed(() => {
  const text = active.value?.text || "";
  const words = text.trim().split(/\s+/).filter(Boolean).length;
  const minutes = words ? Math.max(1, Math.ceil(words / 220)) : 0;
  return `${words} words | ${text.length} characters | ${minutes} min read`;
});
const releaseStatus = computed(() => active.value.compile?.semantic.status || "draft");
const releaseStatusClass = computed(() => `release-${releaseStatus.value.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}`);
const watchStatus = computed(() => {
  if (store.watchDriver === "off" || !store.watchedPaths.length) return "";
  const label = store.watchDriver === "native" ? "Native watch" : "Plugin watch";
  const suffix = store.watchedPaths.length === 1 ? "path" : "paths";
  return `${label}: ${store.watchedPaths.length} ${suffix}`;
});
const externalTransformTrustPrompts = computed<TransformTrustPrompt[]>(() => {
  const text = active.value?.text || "";
  return store.externalTransformEngines
    .filter((engine) => documentUsesTransformFence(text, engine.name))
    .map((engine) => ({
      engine,
      path: store.transformEnginePaths[engine.name]?.trim() || "",
    }))
    .filter(({ engine, path }) => Boolean(path) && !store.trustedTransformEngines[engine.name])
    .map(({ engine, path }) => ({
      name: engine.name,
      path,
      inputMode: store.transformInputModes[engine.name] || "stdin",
      securitySummary: engine.securitySummary,
    }));
});
const manifestPreview = computed(() => JSON.stringify(store.exportReadiness?.manifest || active.value.compile?.export_manifest || {}, null, 2));
const readinessLayoutSummary = computed(() => {
  const sections = store.exportReadiness?.paged_document.sections || [];
  const columnedSections = sections.filter((section) => (section.layout.columns || 1) > 1).length;
  return `${sections.length} layout sections, ${columnedSections} columned`;
});
const bibliographyByKey = computed(() => new Map((active.value.compile?.bibliography || []).map((entry) => [entry.key, entry.title])));
const missingCitationKeys = computed(() => {
  const byKey = bibliographyByKey.value;
  const keys = (active.value.compile?.semantic.citation_references || [])
    .map((citation) => citation.key)
    .filter((key) => !byKey.has(key));
  return Array.from(new Set(keys)).sort();
});
const resolvedCitationEntries = computed(() => {
  const citedKeys = new Set((active.value.compile?.semantic.citation_references || []).map((citation) => citation.key));
  return (active.value.compile?.bibliography || []).filter((entry) => citedKeys.has(entry.key));
});
const duplicateBibliographyEntries = computed(() => {
  const duplicateKeys = new Set(active.value.compile?.semantic.duplicate_bibliography_keys || []);
  return (active.value.compile?.bibliography || [])
    .filter((entry) => duplicateKeys.has(entry.key))
    .map((entry) => ({
      ...entry,
      locationLabel: entry.line ? `${entry.source_file || active.value.path || "document"}:${entry.line}` : "Source location unavailable",
    }));
});
const glossaryEntries = computed(() =>
  Object.entries(active.value.compile?.semantic.glossary || {})
    .map(([term, definition]) => ({ term, definition }))
    .sort((left, right) => left.term.localeCompare(right.term)),
);
const indexTerms = computed(() => [...(active.value.compile?.index_terms || [])].sort((left, right) => left.localeCompare(right)));
const reviewSummary = computed(() => {
  const semantic = active.value.compile?.semantic;
  const comments = semantic?.comments || [];
  const aiSources = semantic?.ai_sources || [];
  const aiSections = semantic?.ai_assisted_sections || [];
  const aiItems = [...aiSources, ...aiSections];
  return {
    status: semantic?.status || "draft",
    unresolved: comments.filter((comment) => comment.state !== "resolved").length,
    resolved: comments.filter((comment) => comment.state === "resolved").length,
    changeNotes: semantic?.change_notes.length || 0,
    aiPending: aiItems.filter((item) => item.status !== "human-reviewed").length,
    aiReviewed: aiItems.filter((item) => item.status === "human-reviewed").length,
  };
});
const citationStyle = computed(() =>
  String(active.value.compile?.metadata.citationStyle || active.value.compile?.metadata.cslStyle || store.bibliographyDefaults.citationStyle),
);
const markdownTables = computed(() => parseMarkdownTables(active.value?.text || ""));
const selectedTable = computed(() => markdownTables.value[selectedTableIndex.value] || null);
const outlineHeadings = computed(() =>
  (active.value.compile?.document_ast.blocks || []).flatMap((block) => {
    if (block.kind !== "heading") return [];
    return [
      {
        text: block.text,
        anchor: block.anchor,
        level: block.level,
        line: block.source?.source_line || block.line,
        end_line: block.source?.end_source_line || block.end_line,
        source_file: block.source?.source_file || null,
      },
    ];
  }),
);
const figureBlocks = computed<FigureListItem[]>(() =>
  (active.value.compile?.document_ast.blocks || []).flatMap((block: DocumentBlock) => {
    if (block.kind !== "figure") return [];
    return [
      {
        id: block.id || null,
        src: block.src || null,
        alt: block.alt || null,
        caption: block.caption || null,
        fit: block.fit || null,
        position: block.position || null,
        line: block.source?.source_line || block.line,
        end_line: block.source?.end_source_line || block.end_line,
        source_file: block.source?.source_file || null,
      },
    ];
  }),
);
const includeGraphItems = computed<IncludeGraphItem[]>(() => {
  const seen = new Set<string>();
  return (active.value.compile?.include_graph || [])
    .filter((edge) => {
      const key = `${edge.parent}\n${edge.child}\n${edge.depth}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    })
    .sort((left, right) => left.depth - right.depth || left.parent.localeCompare(right.parent) || left.child.localeCompare(right.child))
    .map((edge) => {
      const parentLabel = displayDocumentPath(edge.parent);
      const childLabel = displayDocumentPath(edge.child);
      return {
        ...edge,
        parentLabel,
        childLabel,
        commandLabel: `Open include ${childLabel}`,
      };
    });
});
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
const conflictDocument = computed(() => {
  const conflict = store.externalConflict;
  if (!conflict) return active.value;
  return store.documents.find((document) => document.id === conflict.documentId) || active.value;
});
const conflictDiffRows = computed(() => buildConflictDiff(conflictDocument.value.text, store.externalConflict?.externalText || ""));
const tableColumnTotals = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return [];
  return draft.headers.map((_, columnIndex) => formatTableTotal(draft, columnIndex));
});
const tableDraftIssues = computed(() => (tableDraft.value ? validateTableDraft(tableDraft.value) : []));
const tableDraftHasErrors = computed(() => tableDraftIssues.value.some((issue) => issue.severity === "error"));
const tableDraftMarkdownPreview = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return "";
  return serializeMarkdownTable(normalizeTableDraft(draft)).join("\n");
});
const tableDataRowCount = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return 1;
  return Math.max(1, draft.rows.filter((row) => !isTableSummaryRow(row)).length);
});
const tableFormulaTargetColumns = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return [];
  const options = draft.headers.map((header, index) => ({
    index,
    label: `${spreadsheetColumnName(index + 1)} - ${header || `Column ${index + 1}`}`,
  }));
  return options.length > 1 ? options.slice(1) : options;
});
const tableFormulaPreview = computed(() => {
  const row = buildCustomTableFormulaRow();
  if (!row) return "";
  return row.find(isFormulaCell) || "";
});
const selectedTableSpanCell = computed({
  get: () => `${tableSpanRow.value}:${tableSpanColumn.value}`,
  set: (value: string) => {
    const [row, column] = value.split(":").map((part) => Number(part));
    tableSpanRow.value = Number.isInteger(row) ? row : 0;
    tableSpanColumn.value = Number.isInteger(column) ? column : 0;
    syncTableSpanControlsFromCell();
  },
});
const tableSpanCellOptions = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return [];
  return draft.rows.flatMap((row, rowIndex) =>
    draft.headers.map((header, columnIndex) => ({
      value: `${rowIndex}:${columnIndex}`,
      label: `${spreadsheetColumnName(columnIndex + 1)}${rowIndex + 1} - ${header || `Column ${columnIndex + 1}`} - ${row[columnIndex] || "blank"}`,
    })),
  );
});
const tableSpanMaxColspan = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return 1;
  return Math.max(1, draft.headers.length - tableSpanColumn.value);
});
const tableSpanMaxRowspan = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return 1;
  return Math.max(1, draft.rows.length - tableSpanRow.value);
});
const tableSpanPreview = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return "";
  const row = draft.rows[tableSpanRow.value];
  const value = row?.[tableSpanColumn.value];
  if (value === undefined) return "";
  const colspan = clampInteger(tableSpanColspan.value, 1, tableSpanMaxColspan.value);
  const rowspan = clampInteger(tableSpanRowspan.value, 1, tableSpanMaxRowspan.value);
  return setTableCellSpan(value, colspan, rowspan);
});
const diagnosticSignature = computed(() =>
  (active.value.compile?.diagnostics || [])
    .map((diagnostic) =>
      [
        diagnostic.severity,
        diagnostic.source_file || "",
        diagnostic.line || "",
        diagnostic.column || "",
        diagnostic.end_line || "",
        diagnostic.end_column || "",
        diagnostic.message,
        diagnostic.related.join("|"),
      ].join(":"),
    )
    .join("\n"),
);
const allTransformTemplates = computed<TransformTemplate[]>(() => [
  ...builtinTransformTemplates,
  ...store.customTransformTemplates.map((template) => ({ ...template, source: "custom" as const })),
]);
const transformTemplateCategoryOptions = computed(() =>
  [...new Set([...transformTemplateCategories, ...store.customTransformTemplates.map((template) => template.category).filter(Boolean)])].sort(),
);
const transformTemplateKindOptions = computed(() =>
  [...new Set([...transformTemplateKinds, ...store.customTransformTemplates.map((template) => template.transform).filter(Boolean)])].sort(),
);
const filteredTransformTemplates = computed(() => {
  const query = templateQuery.value.trim().toLowerCase();
  return allTransformTemplates.value.filter((template) => {
    if (templateCategory.value !== "all" && template.category !== templateCategory.value) return false;
    if (templateTransform.value !== "all" && template.transform !== templateTransform.value) return false;
    if (!query) return true;
    return [template.name, template.category, template.transform, template.summary, ...template.tags].join(" ").toLowerCase().includes(query);
  });
});
const customTemplateTags = computed({
  get: () => customTemplateDraft.value.tags.join(", "),
  set: (value: string) => {
    customTemplateDraft.value.tags = value
      .split(",")
      .map((tag) => tag.trim())
      .filter(Boolean);
  },
});
const customTemplateIsValid = computed(
  () => Boolean(customTemplateDraft.value.name.trim() && customTemplateDraft.value.transform.trim() && customTemplateDraft.value.body.trim()),
);
const customTemplateFillFields = computed(() => transformTemplateFillFields(customTemplateDraft.value));
const commandBarGroups = computed<CommandBarGroup[]>(() => [
  {
    id: "document",
    label: "Document",
    actions: [
      { id: "new", label: "New", title: "New document", icon: "new", primary: true, run: () => store.newDocument() },
      { id: "open", label: "Open", title: "Open document", icon: "open", run: () => openDocument() },
      { id: "save", label: "Save", title: "Save document", icon: "save", primary: true, run: () => saveDocument() },
      { id: "save-as", label: "Save As", title: "Save document as", icon: "saveAs", run: () => saveDocumentAs() },
      { id: "export-html", label: "HTML Export", title: "Export standalone HTML", icon: "html", run: () => exportDocumentAs("html") },
      { id: "export", label: "Export", title: "Export document", icon: "export", disabled: store.exportBusy, run: () => exportDocument() },
    ],
  },
  {
    id: "manage",
    label: "Manage",
    actions: [
      { id: "open-folder", label: "Open Folder", title: "Open folder", icon: "folder", run: () => openFolder() },
      { id: "save-workspace", label: "Save Workspace", title: "Save workspace", icon: "workspace", run: () => saveWorkspace() },
      { id: "revert", label: "Revert", title: "Revert to saved", icon: "revert", run: () => store.revertActive() },
      { id: "rename", label: "Rename", title: "Rename document", icon: "rename", run: () => renameDocument() },
      { id: "duplicate", label: "Duplicate", title: "Duplicate document", icon: "duplicate", run: () => duplicateDocument() },
      { id: "reveal", label: "Reveal", title: "Reveal in file manager", icon: "reveal", run: () => store.revealActive() },
      { id: "snapshot", label: "Snapshot", title: "Create snapshot", icon: "snapshot", run: () => snapshotActive() },
    ],
  },
  {
    id: "write",
    label: "Write",
    actions: [
      { id: "bold", label: "Bold", title: "Bold selection", icon: "bold", run: () => wrapSelection("**") },
      { id: "italic", label: "Italic", title: "Italic selection", icon: "italic", run: () => wrapSelection("*") },
      { id: "code", label: "Code", title: "Inline code selection", icon: "code", run: () => wrapSelection("`") },
      { id: "link", label: "Link", title: "Insert link", icon: "link", run: () => wrapSelection("[", "](https://)") },
      { id: "heading", label: "Heading", title: "Insert second-level heading", icon: "heading", run: () => insertAtLineStart("## ") },
      { id: "fence", label: "Fence", title: "Insert code fence", icon: "fence", run: () => insertBlock(codeFenceSnippet) },
    ],
  },
  {
    id: "navigate",
    label: "Navigate",
    actions: [
      { id: "search", label: "Find", title: "Find and replace", icon: "find", run: () => runEditorCommand(openSearchPanel) },
      { id: "find-previous", label: "Prev", title: "Find previous match", icon: "previous", run: () => runEditorCommand(findPrevious) },
      { id: "find-next", label: "Next", title: "Find next match", icon: "next", run: () => runEditorCommand(findNext) },
      { id: "outline", label: "Outline", title: "Show document outline", icon: "outline", run: () => showOutline() },
      { id: "fold-all", label: "Fold", title: "Fold all Markdown sections", icon: "fold", run: () => runEditorCommand(foldAll) },
      { id: "unfold-all", label: "Unfold", title: "Unfold all Markdown sections", icon: "unfold", run: () => runEditorCommand(unfoldAll) },
    ],
  },
  {
    id: "insert",
    label: "Insert",
    actions: [
      { id: "table", label: "Table", title: "Insert table", icon: "table", run: () => insertBlock(tableSnippet) },
      { id: "figure", label: "Figure", title: "Insert figure", icon: "figure", run: () => insertFigureSnippet() },
      { id: "calc", label: "Calc", title: "Insert calculation block", icon: "calc", run: () => insertBlock(calcSnippet) },
      { id: "templates", label: "Templates", title: "Open transform templates", icon: "templates", run: () => openTransformTemplates() },
      { id: "equation", label: "Equation", title: "Insert equation", icon: "equation", run: () => insertBlock(equationSnippet) },
      { id: "toc", label: "TOC", title: "Insert table of contents", icon: "toc", run: () => insertBlock(tocSnippet) },
      { id: "ai-source", label: "AI Source", title: "Insert AI source block", icon: "ai", run: () => insertBlock(aiSnippet) },
    ],
  },
  {
    id: "review",
    label: "Review",
    actions: [
      { id: "ai-paste", label: "AI Paste", title: "Paste from AI chat", icon: "ai", run: () => openAiPaste() },
      { id: "comment", label: "Comment", title: "Insert review comment", icon: "comment", run: () => insertBlock(commentSnippet) },
      { id: "commands", label: "Commands", title: "Open command palette", icon: "commands", run: () => (commandPaletteOpen.value = true) },
    ],
  },
]);
const commandToolbarRows = computed<CommandToolbarRow[]>(() => {
  const byId = new Map(commandBarGroups.value.map((group) => [group.id, group]));
  const rows = [
    { id: "file", label: "File", groupIds: ["document", "manage"] },
    { id: "writing", label: "Writing", groupIds: ["write", "insert"] },
    { id: "review-navigation", label: "Review & Navigate", groupIds: ["navigate", "review"] },
  ];
  return rows.map((row) => ({
    id: row.id,
    label: row.label,
    groups: row.groupIds.flatMap((id) => {
      const group = byId.get(id);
      return group ? [group] : [];
    }),
  }));
});
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
  { name: "Prepare for export", group: "Export", run: () => void prepareForExport() },
  { name: "Export HTML", group: "Export", run: () => void exportDocumentAs("html") },
  { name: "Export document", group: "Export", run: () => void exportDocument() },
  { name: "Create snapshot", group: "Versioning", run: () => void snapshotActive() },
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
  { name: "Select next occurrence", group: "Edit", run: () => runEditorCommand(selectNextOccurrence) },
  { name: "Add cursor above", group: "Edit", run: () => runEditorCommand(addCursorAbove) },
  { name: "Add cursor below", group: "Edit", run: () => runEditorCommand(addCursorBelow) },
  { name: "Show document outline", group: "Navigate", run: () => showOutline() },
  { name: "Fold all sections", group: "Navigate", run: () => runEditorCommand(foldAll) },
  { name: "Unfold all sections", group: "Navigate", run: () => runEditorCommand(unfoldAll) },
  { name: "Show toolbar icons and text", group: "View", run: () => (store.toolbarDisplay = "both") },
  { name: "Show toolbar icons only", group: "View", run: () => (store.toolbarDisplay = "icons") },
  { name: "Show toolbar text only", group: "View", run: () => (store.toolbarDisplay = "text") },
  { name: "Bold selection", group: "Markdown", run: () => wrapSelection("**") },
  { name: "Italic selection", group: "Markdown", run: () => wrapSelection("*") },
  { name: "Inline code selection", group: "Markdown", run: () => wrapSelection("`") },
  { name: "Add review comment", group: "Review", run: () => (store.sidebar = "review") },
  { name: "Open table editor", group: "Tables", run: () => openTableEditor() },
  { name: "Open transform templates", group: "Transforms", run: () => openTransformTemplates() },
  { name: "Insert code fence", group: "Snippet", run: () => insertBlock(codeFenceSnippet) },
  { name: "Insert table", group: "Snippet", run: () => insertBlock(tableSnippet) },
  { name: "Insert cover figure", group: "Snippet", run: () => insertFigureSnippet() },
  ...figureCropPositions
    .filter((position) => position !== "center")
    .map((position) => ({
      name: `Insert ${position} crop figure`,
      group: "Snippet",
      run: () => insertFigureSnippet(position),
    })),
  { name: "Insert calculation", group: "Snippet", run: () => insertBlock(calcSnippet) },
  { name: "Insert equation", group: "Snippet", run: () => insertBlock(equationSnippet) },
  { name: "Insert table of contents", group: "Snippet", run: () => insertBlock(tocSnippet) },
  { name: "Insert index", group: "Snippet", run: () => insertBlock(indexSnippet) },
  { name: "Insert bibliography", group: "Snippet", run: () => insertBlock(bibliographySnippet) },
  { name: "Insert list of figures", group: "Snippet", run: () => insertBlock(listOfFiguresSnippet) },
  { name: "Insert list of tables", group: "Snippet", run: () => insertBlock(listOfTablesSnippet) },
  { name: "Insert glossary section", group: "Snippet", run: () => insertBlock(glossarySectionSnippet) },
  { name: "Insert glossary", group: "Snippet", run: () => insertBlock(glossarySnippet) },
  { name: "Insert layout directive", group: "Snippet", run: () => insertBlock(layoutSnippet) },
  { name: "Insert review comment", group: "Snippet", run: () => insertBlock(commentSnippet) },
  { name: "Insert AI source", group: "Snippet", run: () => insertBlock(aiSnippet) },
  ...allTransformTemplates.value.map((template) => ({
    name: `Insert ${template.name} template`,
    group: `Template ${template.category}`,
    run: () => insertTransformTemplate(template),
  })),
  {
    name: active.value.pinned ? "Unpin active tab" : "Pin active tab",
    group: "Workspace",
    run: () => store.togglePin(active.value.id),
  },
  ...store.documents.map((document) => ({
    name: document.title,
    group: "Open document",
    run: () => activate(document.id),
  })),
  ...store.workspaceFiles
    .filter((entry) => entry.kind !== "directory")
    .map((entry) => ({
      name: entry.relative_path,
      group: "Workspace file",
      run: () => void store.openPath(entry.path),
    })),
  ...includeGraphItems.value.map((edge) => ({
    name: edge.commandLabel,
    group: `Include depth ${edge.depth}`,
    run: () => void openIncludeChild(edge),
  })),
  ...((active.value.compile?.document_ast.blocks || []).flatMap((block) => {
    if (block.kind !== "heading") return [];
    const line = block.source?.source_line || block.line;
    return [
      {
        name: block.text,
        group: `Heading line ${line}`,
        run: () =>
          void goToSourceTarget({
            line,
            end_line: block.source?.end_source_line || block.end_line,
            source_file: block.source?.source_file || null,
          }),
      },
    ];
  })),
  ...((active.value.compile?.semantic.citation_references || []).map((citation) => ({
    name: `[@${citation.key}]`,
    group: "Citation",
    run: () => {
      store.sidebar = "references";
      void goToSourceTarget(citation);
    },
  }))),
  ...Object.keys(active.value.compile?.semantic.glossary || {}).map((term) => ({
    name: term,
    group: "Glossary",
    run: () => {
      store.sidebar = "references";
      goToSearchTerm(term);
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
      if (diagnostic.line) void goToSourceTarget(diagnostic);
    },
  }))),
]);
const filteredCommands = computed(() => {
  const query = commandQuery.value.trim().toLowerCase();
  if (!query) return commands.value;
  return commands.value.filter((command) => `${command.name} ${command.group}`.toLowerCase().includes(query));
});

async function bindNativeMenuCommands() {
  try {
    unlistenNativeMenuCommand = await listen<string>("neditor-menu-command", (event) => {
      void runNativeMenuCommand(event.payload);
    });
  } catch {
    unlistenNativeMenuCommand = null;
  }
}

async function runNativeMenuCommand(command: string) {
  const exportTarget = nativeMenuExportTargets[command];
  if (exportTarget) {
    await exportDocumentAs(exportTarget);
    return;
  }

  switch (command) {
    case "neditor-new-document":
      store.newDocument();
      break;
    case "neditor-open-document":
      await openDocument();
      break;
    case "neditor-save-document":
      await saveDocument();
      break;
    case "neditor-save-document-as":
      await saveDocumentAs();
      break;
    case "neditor-prepare-export":
      await prepareForExport();
      store.sidebar = "exports";
      break;
    case "neditor-export-current":
      await exportDocument();
      break;
    case "neditor-open-folder":
      await openFolder();
      break;
    case "neditor-save-workspace":
      await saveWorkspace();
      break;
    case "neditor-open-search":
      runEditorCommand(openSearchPanel);
      break;
    case "neditor-mode-split":
      store.mode = "split";
      break;
    case "neditor-mode-source":
      store.mode = "source";
      break;
    case "neditor-mode-preview":
      store.mode = "preview";
      break;
    case "neditor-mode-focus":
      store.mode = "focus";
      break;
    case "neditor-mode-export":
      store.mode = "export";
      store.sidebar = "exports";
      break;
    case "neditor-show-outline":
      store.sidebar = "outline";
      break;
    case "neditor-show-exports":
      store.sidebar = "exports";
      break;
    case "neditor-insert-table":
      insertBlock(tableSnippet);
      flushEditorTextToStore();
      break;
    case "neditor-insert-code-fence":
      insertBlock(codeFenceSnippet);
      flushEditorTextToStore();
      break;
    case "neditor-insert-equation":
      insertBlock(equationSnippet);
      flushEditorTextToStore();
      break;
    case "neditor-insert-toc":
      insertBlock(tocSnippet);
      flushEditorTextToStore();
      break;
    case "neditor-open-templates":
      store.sidebar = "templates";
      break;
    case "neditor-clean-ai-paste":
      openAiPaste();
      break;
  }
}

onMounted(async () => {
  await store.boot();
  await bindNativeMenuCommands();
  applyAiPasteDefaults();
  buildEditor();
  scheduleAutosave();
  scheduleAutoSnapshot();
  setWindowTitle(store.windowTitle);
  void nextTick().then(async () => {
    await reportDesktopUiSmoke();
    await runDesktopWorkflowSmokeIfEnabled();
  });
  window.addEventListener("keydown", handleShortcut);
});

onBeforeUnmount(() => {
  recordActiveScrollPosition(true);
  editorView?.destroy();
  previewTextCommit.cancel();
  window.clearTimeout(autosaveHandle);
  window.clearTimeout(autoSnapshotHandle);
  window.clearTimeout(scrollPersistHandle);
  window.removeEventListener("keydown", handleShortcut);
  unlistenNativeMenuCommand?.();
  unlistenNativeMenuCommand = null;
  stopPaneResize();
});

watch(aiPasteOpen, (open) => handleModalStateChange(open, aiPasteDialog));
watch(commandPaletteOpen, (open) => handleModalStateChange(open, commandPaletteDialog));
watch(conflictOpen, (open) => handleModalStateChange(open, conflictDialog));

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
  () => store.externalConflict?.path || "",
  () => {
    conflictMergeParts.value = [];
    mergedConflictText.value = "";
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
      mergedConflictText.value = conflictDocument.value.text;
    } else {
      mergedConflictText.value = "";
    }
  },
);

watch(
  () => [
    store.wordWrap,
    store.lineNumbers,
    store.codeFolding,
    store.theme,
    store.previewTheme,
    store.toolbarDisplay,
    store.toolbarTextSize,
    store.highContrast,
    store.reducedMotion,
    store.editorFont,
    store.editorFontSize,
    store.editorLineHeight,
    store.previewFont,
    store.previewFontSize,
    store.previewLineHeight,
  ],
  () => {
    buildEditor();
    void store.persistWorkspace();
  },
);

watch(
  () => [store.autosave, store.autosaveDelayMs, store.autoSnapshot, store.snapshotIntervalMs, store.snapshotStorage],
  () => {
    scheduleAutosave();
    scheduleAutoSnapshot();
    void store.persistWorkspace();
  },
);

watch(
  () => store.exportTarget,
  () => {
    store.exportReadiness = null;
    void store.persistWorkspace();
  },
);

watch(
  () => store.mode,
  (mode) => {
    if (mode === "export") {
      store.sidebar = "exports";
    } else if (mode === "review") {
      store.sidebar = "review";
    } else if (mode === "presentation") {
      store.sidebar = "outline";
    }
  },
);

watch(
  () => [store.mode, store.sidebar],
  () => {
    void store.persistWorkspace();
  },
);

watch(
  () => [
    store.exportDefaults.includeManifest,
    store.exportDefaults.includeStyles,
    store.exportDefaults.includeSyntaxHighlighting,
    store.exportDefaults.htmlLanguage,
    store.exportDefaults.htmlDescription,
    store.exportDefaults.canonicalUrl,
    store.exportDefaults.coverPage,
    store.exportDefaults.pageNumbers,
    store.exportDefaults.layoutPreset,
    store.exportDefaults.includeComments,
    store.exportDefaults.includeProvenance,
    store.exportDefaults.includeGlossary,
    store.exportDefaults.includeAgenda,
  ],
  () => {
    void store.persistWorkspace();
  },
);

watch(activeExportProfile, (profile) => {
  if (profile) exportProfileName.value = profile.name;
});

watch(
  () => store.bibliographyDefaults.citationStyle,
  () => {
    void store.compileActive();
    void store.persistWorkspace();
  },
);

watch(
  () => [
    store.brandProfileDefaults.name,
    store.brandProfileDefaults.color,
    store.brandProfileDefaults.logo,
    store.brandProfileDefaults.font,
    store.brandProfileDefaults.header,
    store.brandProfileDefaults.footer,
    store.brandProfileDefaults.watermark,
    store.brandProfileDefaults.legalDisclaimer,
  ],
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
    store.aiCleanupDefaults.preserveHeadings,
    store.aiCleanupDefaults.convertNumberedLists,
    store.aiCleanupDefaults.convertTables,
  ],
  () => {
    void store.persistWorkspace();
  },
);

watch(
  () => store.windowTitle,
  (title) => {
    setWindowTitle(title);
  },
);

function setWindowTitle(title: string) {
  document.title = title;
  void appWindow?.setTitle(title).catch(() => undefined);
}

async function reportDesktopUiSmoke() {
  const text = (selector: string) => document.querySelector(selector)?.textContent?.replace(/\s+/g, " ").trim() || "";
  const commandLabels = Array.from(document.querySelectorAll("#main-commands button"))
    .map((button) => button.textContent?.replace(/\s+/g, " ").trim() || "")
    .filter(Boolean);
  await invoke("write_desktop_ui_smoke_report", {
    payload: {
      title: document.title,
      activeDocumentTitle: active.value.title,
      viewMode: store.mode,
      sidebarPanel: store.sidebar,
      toolbarDisplay: store.toolbarDisplay,
      workspaceClass: document.querySelector("#document-workspace")?.className || "",
      commandLabels,
      surfaces: {
        commands: Boolean(document.querySelector("#main-commands")),
        sidebar: Boolean(document.querySelector("#document-sidebar")),
        source: Boolean(document.querySelector("#markdown-source")),
        preview: Boolean(document.querySelector("#live-preview")),
        status: Boolean(document.querySelector("#document-status")),
      },
      surfaceText: {
        sidebar: text("#document-sidebar").slice(0, 240),
        preview: text("#live-preview").slice(0, 240),
        status: text("#document-status").slice(0, 240),
      },
      previewLabel: document.querySelector(".preview-document")?.getAttribute("aria-label") || "",
      viewport: {
        width: window.innerWidth,
        height: window.innerHeight,
        scrollWidth: document.documentElement.scrollWidth,
      },
    },
  }).catch(() => undefined);
}

async function runDesktopWorkflowSmokeIfEnabled() {
  const enabled = await invoke<boolean>("desktop_workflow_smoke_enabled").catch(() => false);
  if (!enabled) return;
  desktopWorkflowSmokeActive.value = true;

  const assertions: Array<{ name: string; passed: boolean; detail?: string }> = [];
  const record = (name: string, passed: boolean, detail?: string) => {
    assertions.push({ name, passed, ...(detail ? { detail } : {}) });
  };
  const text = (selector: string) => document.querySelector(selector)?.textContent?.replace(/\s+/g, " ").trim() || "";
  let smokePhase = "starting";

  try {
    smokePhase = "started";
    await writeNativeWorkflowProgress(smokePhase, assertions);
    record("native workflow starts with NEditor title", document.title.includes("NEditor"), document.title);

    const fileWorkflow = await collectNativeFileWorkflowEvidence(record);
    smokePhase = "file-workflow";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow });
    const snapshotEvidence = await collectNativeSnapshotEvidence(record);
    smokePhase = "snapshots";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence });
    const modeEvidence = await collectNativeModeEvidence(record);
    smokePhase = "modes";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence });

    commandPaletteOpen.value = true;
    await nextTick();
    record(
      "native workflow opened command palette",
      Boolean(document.querySelector('[role="dialog"][aria-label="Command palette"]')),
    );
    commandPaletteOpen.value = false;
    store.mode = "split";
    await nextTick();

    openTransformTemplates();
    templateCategory.value = "Science";
    templateTransform.value = "calc";
    templateQuery.value = "dose";
    await nextTick();
    const doseTemplate = filteredTransformTemplates.value.find((template) => template.id === "calc-science-dose");
    record("native workflow found dose template", Boolean(doseTemplate));
    if (doseTemplate) {
      insertTransformTemplate(doseTemplate);
      flushEditorTextToStore();
      await store.compileActive();
      await nextTick();
    }
    record("native workflow inserted calc template into source", active.value.text.includes("weight_kg = 72"));
    record("native workflow rendered calc template preview", text("#live-preview").includes("Total dose"));
    record("native workflow exposed dirty title", document.title.startsWith("* "), document.title);

    store.activeExportProfileId = "";
    store.exportTarget = "html";
    store.exportDefaults.includeManifest = true;
    await store.prepareForExport();
    await nextTick();
    const readinessTarget = store.exportReadiness?.manifest?.export_target;
    record("native workflow prepared html export readiness", readinessTarget === "html", JSON.stringify(readinessTarget));
    const exportOutputPath = await invoke<string | null>("desktop_workflow_smoke_export_path", { extension: "html" }).catch(() => null);
    if (exportOutputPath) {
      await store.exportActive(exportOutputPath);
      await nextTick();
    }
    const exportResult = {
      target: store.exportTarget,
      outputPath: store.lastExportOutputPath,
      manifestPath: store.lastExportManifestPath,
      progressSteps: store.lastExportProgressSteps.map((step) => step.id),
      diagnostics: store.lastExportDiagnostics.map((diagnostic) => diagnostic.severity),
    };
    record(
      "native workflow wrote html export artifact",
      Boolean(
        exportOutputPath &&
          store.lastExportOutputPath === exportOutputPath &&
          store.lastExportProgressSteps.some((step) => step.id === "render" && step.state === "complete") &&
          !store.lastExportDiagnostics.some((diagnostic) => diagnostic.severity === "error"),
      ),
      JSON.stringify(exportResult),
    );
    store.lastExportOutputPath = "";
    store.lastExportManifestPath = "";
    store.lastExportProgressSteps = [];
    store.lastExportDiagnostics = [];
    await emitNativeWorkflowMenuCommand("neditor-export-html", 500);
    await waitForNativeWorkflowCondition(
      () =>
        Boolean(
          exportOutputPath &&
            store.exportTarget === "html" &&
            store.sidebar === "exports" &&
            store.lastExportOutputPath === exportOutputPath &&
            store.lastExportProgressSteps.some((step) => step.id === "render" && step.state === "complete") &&
            !store.lastExportDiagnostics.some((diagnostic) => diagnostic.severity === "error"),
        ),
      2400,
    );
    const nativeMenuExportResult = {
      target: store.exportTarget,
      sidebar: store.sidebar,
      outputPath: store.lastExportOutputPath,
      manifestPath: store.lastExportManifestPath,
      progressSteps: store.lastExportProgressSteps.map((step) => `${step.id}:${step.state}`),
      diagnostics: store.lastExportDiagnostics.map((diagnostic) => diagnostic.severity),
    };
    record(
      "native workflow exported html from native menu command",
      Boolean(
        exportOutputPath &&
          store.lastExportOutputPath === exportOutputPath &&
          store.lastExportProgressSteps.some((step) => step.id === "render" && step.state === "complete") &&
          store.sidebar === "exports",
      ),
      JSON.stringify(nativeMenuExportResult),
    );
    smokePhase = "html-export";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, exportResult, nativeMenuExportResult });
    const editorSnippet = smokeSnippetAround(active.value.text, "weight_kg = 72");
    const previewSnippet = text("#live-preview").slice(0, 2000);
    const exportReadinessEvidence = store.exportReadiness
      ? {
          ready: store.exportReadiness.ready,
          errors: store.exportReadiness.error_count,
          warnings: store.exportReadiness.warning_count,
          target: store.exportReadiness.manifest?.export_target,
          progressSteps: store.exportReadiness.progress_steps.map((step) => step.id),
        }
      : null;
    smokePhase = "export-profile-start";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, exportResult, nativeMenuExportResult });
    const exportProfileEvidence = await collectNativeExportProfileEvidence(record);
    smokePhase = "export-profile";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, exportResult, nativeMenuExportResult, exportProfileEvidence });
    smokePhase = "theme-accessibility-start";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, exportResult, nativeMenuExportResult, exportProfileEvidence });
    const themeAccessibility = await collectNativeThemeAccessibilityEvidence(record);
    smokePhase = "theme-accessibility";
    await writeNativeWorkflowProgress(smokePhase, assertions, {
      fileWorkflow,
      snapshotEvidence,
      modeEvidence,
      exportResult,
      nativeMenuExportResult,
      exportProfileEvidence,
      themeAccessibility,
    });
    smokePhase = "native-menu-commands-start";
    await writeNativeWorkflowCheckpoint(smokePhase, assertions);
    const nativeMenuCommandEvidence = await collectNativeMenuCommandEvidence(record, (phase) => writeNativeWorkflowCheckpoint(phase, assertions));
    smokePhase = "native-menu-commands";
    await writeNativeWorkflowCheckpoint(smokePhase, assertions);
    smokePhase = "workspace-tabs-start";
    await writeNativeWorkflowCheckpoint(smokePhase, assertions);
    const workspaceTabEvidence = await collectNativeWorkspaceTabEvidence(record);
    smokePhase = "workspace-tabs";
    await writeNativeWorkflowCheckpoint(smokePhase, assertions);

    const passed = assertions.every((assertion) => assertion.passed);
    smokePhase = "final";
    await writeDesktopWorkflowSmokeReport({
      status: passed ? "passed" : "failed",
      phase: smokePhase,
      assertions,
      title: document.title,
      fileWorkflow,
      snapshotEvidence,
      mode: store.mode,
      sidebar: store.sidebar,
      modeEvidence,
      editorSnippet,
      previewSnippet,
      themeAccessibility,
      exportProfileEvidence,
      exportResult,
      nativeMenuExportResult,
      nativeMenuCommandEvidence,
      workspaceTabEvidence,
      exportReadiness: exportReadinessEvidence,
    });
  } catch (error) {
    await writeDesktopWorkflowSmokeReport({
      status: "failed",
      phase: smokePhase,
      assertions,
      error: error instanceof Error ? error.message : String(error),
      title: document.title,
    });
  }
}

async function collectNativeSnapshotEvidence(record: (name: string, passed: boolean, detail?: string) => void) {
  const filePath = active.value.path;
  const originalText = active.value.text;
  const originalStorage = store.snapshotStorage;
  if (!filePath) {
    record("native workflow resolved snapshot source file", false);
    return null;
  }
  store.snapshotStorage = "app-data";
  await store.persistWorkspace();
  const created = await store.createSnapshot("native-smoke");
  await store.listSnapshots();
  const listedSnapshot = store.snapshots.find((snapshot) => snapshot.snapshot_path === created.snapshot_path);
  const appDataCreated = {
    storage: store.snapshotStorage,
    snapshotPath: created.snapshot_path,
    listed: Boolean(listedSnapshot),
    label: listedSnapshot?.label || "",
    sourcePath: filePath,
    hash: listedSnapshot?.hash || "",
  };
  record(
    "native workflow created and listed app-data snapshot",
    Boolean(created.snapshot_path && listedSnapshot?.label === "native-smoke" && listedSnapshot.hash),
    JSON.stringify(appDataCreated),
  );

  const mutatedText = `${originalText}\n\nNative snapshot mutation.`;
  await setNativeWorkflowText(mutatedText);
  record("native workflow dirtied document before snapshot restore", active.value.text.includes("Native snapshot mutation"), active.value.title);
  await store.restoreSnapshot(created.snapshot_path);
  await nextTick();
  const appDataRestored = {
    restoredText: active.value.text.slice(0, 120),
    containsMutation: active.value.text.includes("Native snapshot mutation"),
    statusMessage: store.statusMessage,
    snapshotCount: store.snapshots.length,
  };
  record(
    "native workflow restored app-data snapshot",
    active.value.text === originalText && !active.value.text.includes("Native snapshot mutation"),
    JSON.stringify(appDataRestored),
  );
  await store.saveActive(filePath);

  store.snapshotStorage = "project-local";
  await store.persistWorkspace();
  const projectCreated = await store.createSnapshot("native-project-smoke");
  await store.listSnapshots();
  const listedProjectSnapshot = store.snapshots.find((snapshot) => snapshot.snapshot_path === projectCreated.snapshot_path);
  const projectSnapshotPath = projectCreated.snapshot_path.replace(/\\/g, "/");
  const projectLocalCreated = {
    storage: store.snapshotStorage,
    snapshotPath: projectCreated.snapshot_path,
    listed: Boolean(listedProjectSnapshot),
    label: listedProjectSnapshot?.label || "",
    sourcePath: filePath,
    hash: listedProjectSnapshot?.hash || "",
  };
  record(
    "native workflow created and listed project-local snapshot",
    Boolean(
      projectCreated.snapshot_path &&
        projectSnapshotPath.includes("/.neditor/snapshots/") &&
        listedProjectSnapshot?.label === "native-project-smoke" &&
        listedProjectSnapshot.hash,
    ),
    JSON.stringify(projectLocalCreated),
  );

  const projectMutatedText = `${originalText}\n\nNative project-local snapshot mutation.`;
  await setNativeWorkflowText(projectMutatedText);
  record(
    "native workflow dirtied document before project-local snapshot restore",
    active.value.text.includes("Native project-local snapshot mutation"),
    active.value.title,
  );
  await store.restoreSnapshot(projectCreated.snapshot_path);
  await nextTick();
  const projectLocalRestored = {
    restoredText: active.value.text.slice(0, 120),
    containsMutation: active.value.text.includes("Native project-local snapshot mutation"),
    statusMessage: store.statusMessage,
    snapshotCount: store.snapshots.length,
  };
  record(
    "native workflow restored project-local snapshot",
    active.value.text === originalText && !active.value.text.includes("Native project-local snapshot mutation"),
    JSON.stringify(projectLocalRestored),
  );
  await store.saveActive(filePath);

  store.snapshotStorage = originalStorage;
  await store.persistWorkspace();
  return {
    appData: { created: appDataCreated, restored: appDataRestored },
    projectLocal: { created: projectLocalCreated, restored: projectLocalRestored },
  };
}

async function writeNativeWorkflowProgress(
  phase: string,
  assertions: Array<{ name: string; passed: boolean; detail?: string }>,
  extra: Record<string, unknown> = {},
) {
  await writeDesktopWorkflowSmokeReport({
    status: "running",
    phase,
    assertions,
    title: document.title,
    mode: store.mode,
    sidebar: store.sidebar,
    ...extra,
  });
}

async function writeNativeWorkflowCheckpoint(
  phase: string,
  assertions: Array<{ name: string; passed: boolean; detail?: string }>,
) {
  await writeDesktopWorkflowSmokeReport({
    status: "running",
    phase,
    assertionCount: assertions.length,
    title: document.title,
    mode: store.mode,
    sidebar: store.sidebar,
  });
}

async function collectNativeExportProfileEvidence(record: (name: string, passed: boolean, detail?: string) => void) {
  store.sidebar = "exports";
  store.exportTarget = "pdf";
  store.exportDefaults.includeManifest = false;
  store.exportDefaults.coverPage = false;
  store.exportDefaults.pageNumbers = false;
  store.exportDefaults.layoutPreset = "compact";
  store.bibliographyDefaults.citationStyle = "ieee";
  store.brandProfileDefaults.name = "Native Board";
  store.brandProfileDefaults.color = "#006699";
  store.brandProfileDefaults.footer = "Native confidential";
  await nextTick();
  const profile = store.saveCurrentExportProfile("Native client PDF");
  await store.persistWorkspace();
  record(
    "native workflow saved export profile",
    Boolean(profile.id && profile.exportTarget === "pdf" && profile.brandProfileDefaults.name === "Native Board"),
    JSON.stringify({ id: profile.id, target: profile.exportTarget, brand: profile.brandProfileDefaults.name }),
  );

  store.exportTarget = "html";
  store.exportDefaults.includeManifest = true;
  store.exportDefaults.coverPage = true;
  store.exportDefaults.pageNumbers = true;
  store.exportDefaults.layoutPreset = "presentation";
  store.bibliographyDefaults.citationStyle = "title";
  store.brandProfileDefaults.name = "";
  store.brandProfileDefaults.footer = "";
  await store.applyExportProfile(profile.id);
  const applied = {
    id: store.activeExportProfileId,
    target: String(store.exportTarget),
    layoutPreset: String(store.exportDefaults.layoutPreset),
    includeManifest: store.exportDefaults.includeManifest,
    coverPage: store.exportDefaults.coverPage,
    pageNumbers: store.exportDefaults.pageNumbers,
    citationStyle: String(store.bibliographyDefaults.citationStyle),
    brandName: store.brandProfileDefaults.name,
    footer: store.brandProfileDefaults.footer,
  };
  record(
    "native workflow applied export profile",
    applied.id === profile.id &&
      applied.target === "pdf" &&
      applied.layoutPreset === "compact" &&
      applied.includeManifest === false &&
      applied.coverPage === false &&
      applied.pageNumbers === false &&
      applied.citationStyle === "ieee" &&
      applied.brandName === "Native Board",
    JSON.stringify(applied),
  );
  await store.persistWorkspace();

  store.exportProfiles = [];
  store.activeExportProfileId = "";
  await store.loadPreferences();
  const reloadedProfile = store.exportProfiles.find((item) => item.id === profile.id);
  const reloaded = {
    profileCount: store.exportProfiles.length,
    id: reloadedProfile?.id || "",
    activeExportProfileId: store.activeExportProfileId,
    target: String(store.exportTarget),
    layoutPreset: String(store.exportDefaults.layoutPreset),
    brandName: reloadedProfile?.brandProfileDefaults.name || "",
  };
  record(
    "native workflow reloaded export profile from settings store",
    Boolean(
      reloadedProfile &&
        store.activeExportProfileId === profile.id &&
        reloaded.target === "pdf" &&
        reloaded.layoutPreset === "compact" &&
        reloadedProfile.brandProfileDefaults.name === "Native Board",
    ),
    JSON.stringify(reloaded),
  );
  return { saved: profile, applied, reloaded };
}

async function collectNativeMenuCommandEvidence(record: (name: string, passed: boolean, detail?: string) => void, checkpoint?: (phase: string) => Promise<void>) {
  const evidence: Record<string, unknown> = {};
  const runMenuCommand = async (command: string, phase: string) => {
    await checkpoint?.(`${phase}-start`);
    await emitNativeWorkflowMenuCommand(command, 500);
    await nextTick();
    await checkpoint?.(`${phase}-emitted`);
  };
  const textCount = (needle: string) => active.value.text.split(needle).length - 1;
  const recordInsertion = async (command: string, key: string, assertion: string, needle: string) => {
    const before = textCount(needle);
    await runMenuCommand(command, `native-menu-command-${key}`);
    await waitForNativeWorkflowCondition(() => textCount(needle) > before, 1000);
    const inserted = textCount(needle) > before;
    evidence[key] = { inserted };
    record(assertion, inserted, JSON.stringify(evidence[key]));
    await checkpoint?.(`native-menu-command-${key}-recorded`);
  };

  await runMenuCommand("neditor-mode-export", "native-menu-command-export-mode");
  await waitForNativeWorkflowCondition(() => store.mode === "export" && store.sidebar === "exports", 1000);
  evidence.exportMode = { mode: store.mode, sidebar: store.sidebar };
  record("native workflow routed export preview from native view menu", store.mode === "export" && store.sidebar === "exports", JSON.stringify(evidence.exportMode));
  await checkpoint?.("native-menu-command-export-mode-recorded");

  await runMenuCommand("neditor-show-outline", "native-menu-command-outline");
  await waitForNativeWorkflowCondition(() => store.sidebar === "outline", 1000);
  evidence.outline = { sidebar: store.sidebar };
  record("native workflow routed outline from native view menu", store.sidebar === "outline", JSON.stringify(evidence.outline));
  await checkpoint?.("native-menu-command-outline-recorded");

  await runMenuCommand("neditor-show-exports", "native-menu-command-exports");
  await waitForNativeWorkflowCondition(() => store.sidebar === "exports", 1000);
  evidence.exports = { sidebar: store.sidebar };
  record("native workflow routed exports from native view menu", store.sidebar === "exports", JSON.stringify(evidence.exports));
  await checkpoint?.("native-menu-command-exports-recorded");

  await runMenuCommand("neditor-open-search", "native-menu-command-search");
  await waitForNativeWorkflowCondition(() => Boolean(document.querySelector(".cm-search")), 1000);
  evidence.search = { open: Boolean(document.querySelector(".cm-search")) };
  record("native workflow opened search from native menu command", Boolean(document.querySelector(".cm-search")), JSON.stringify(evidence.search));
  await checkpoint?.("native-menu-command-search-recorded");

  await recordInsertion("neditor-insert-toc", "toc", "native workflow inserted toc from native writing tools menu", "[TOC]");
  await recordInsertion("neditor-insert-equation", "equation", "native workflow inserted equation from native writing tools menu", "E = mc^2");
  await recordInsertion("neditor-insert-code-fence", "codeFence", "native workflow inserted code fence from native writing tools menu", "```markdown");
  await recordInsertion("neditor-insert-table", "table", "native workflow inserted table from native writing tools menu", "| Revenue | 125000 |");

  await runMenuCommand("neditor-open-templates", "native-menu-command-templates");
  await waitForNativeWorkflowCondition(() => store.sidebar === "templates", 1000);
  evidence.templates = { sidebar: store.sidebar };
  record("native workflow opened templates from native writing tools menu", store.sidebar === "templates", JSON.stringify(evidence.templates));
  await checkpoint?.("native-menu-command-templates-recorded");

  await runMenuCommand("neditor-clean-ai-paste", "native-menu-command-ai-paste");
  await waitForNativeWorkflowCondition(() => aiPasteOpen.value, 1000);
  evidence.aiPaste = { open: aiPasteOpen.value, statusMessage: store.statusMessage };
  record("native workflow opened AI paste from native writing tools menu", aiPasteOpen.value, JSON.stringify(evidence.aiPaste));
  await checkpoint?.("native-menu-command-ai-paste-recorded");
  aiPasteOpen.value = false;

  return evidence;
}

async function emitNativeWorkflowMenuCommand(command: string, timeoutMs: number) {
  void invoke("emit_desktop_workflow_smoke_menu_command", { command }).catch(() => undefined);
  await nativeWorkflowDelay(timeoutMs);
  await nextTick();
}

async function collectNativeWorkspaceTabEvidence(record: (name: string, passed: boolean, detail?: string) => void) {
  const boardOnePath = await invoke<string | null>("desktop_workflow_smoke_named_path", { fileStem: "native-workspace-board-one", extension: "md" }).catch(() => null);
  const boardTwoPath = await invoke<string | null>("desktop_workflow_smoke_named_path", { fileStem: "native-workspace-board-two", extension: "md" }).catch(() => null);
  const loosePath = await invoke<string | null>("desktop_workflow_smoke_named_path", { fileStem: "native-workspace-loose-note", extension: "md" }).catch(() => null);
  const boardSet = "Native Board Pack";
  const evidence: Record<string, unknown> = {
    boardOnePath,
    boardTwoPath,
    loosePath,
    boardSet,
  };
  if (!boardOnePath || !boardTwoPath || !loosePath) {
    record("native workflow resolved workspace tab proof paths", false, JSON.stringify(evidence));
    return evidence;
  }

  const boardOneText = `---
title: Native Board One
documentSet: ${boardSet}
status: draft
---

# Native Board One

Native board one body.
`;
  const boardTwoText = `---
title: Native Board Two
documentSet: ${boardSet}
status: draft
---

# Native Board Two

Native board two body.
`;
  const looseText = `---
title: Native Loose Note
status: draft
---

# Native Loose Note

Native loose note body.
`;
  await invoke("save_file", { request: { path: boardOnePath, text: boardOneText, expected_hash: null } });
  await invoke("save_file", { request: { path: boardTwoPath, text: boardTwoText, expected_hash: null } });
  await invoke("save_file", { request: { path: loosePath, text: looseText, expected_hash: null } });

  await store.openPath(boardOnePath);
  await store.openPath(boardTwoPath);
  await store.openPath(loosePath);
  await nextTick();
  const initialBoardGroup = groupedDocuments.value.find((group) => group.key === `set:${boardSet}`);
  evidence.initialBoardGroup = initialBoardGroup
    ? { key: initialBoardGroup.key, label: initialBoardGroup.label, count: initialBoardGroup.documents.length }
    : null;
  record(
    "native workflow grouped document-set tabs",
    Boolean(initialBoardGroup && initialBoardGroup.documents.length >= 2),
    JSON.stringify(evidence.initialBoardGroup),
  );

  const boardOne = store.documents.find((document) => document.path === boardOnePath);
  if (boardOne) {
    store.setPinned(boardOne.id, true);
    await waitForNativeWorkflowCondition(() => groupedDocuments.value.some((group) => group.key === "pinned" && group.documents.some((document) => document.path === boardOnePath)), 800);
  }
  const pinnedGroup = groupedDocuments.value.find((group) => group.key === "pinned");
  evidence.pinnedGroup = pinnedGroup ? { count: pinnedGroup.documents.length, paths: pinnedGroup.documents.map((document) => document.path) } : null;
  record(
    "native workflow pinned tab into pinned group",
    Boolean(pinnedGroup?.documents.some((document) => document.path === boardOnePath)),
    JSON.stringify(evidence.pinnedGroup),
  );

  const looseDocument = store.documents.find((document) => document.path === loosePath);
  const boardGroupForDrop = groupedDocuments.value.find((group) => group.key === `set:${boardSet}`);
  if (looseDocument && boardGroupForDrop) {
    draggedTabId.value = looseDocument.id;
    dropTabOnGroup(boardGroupForDrop);
    await waitForNativeWorkflowCondition(() => active.value.path === loosePath && active.value.text.includes(`documentSet: ${boardSet}`), 1000);
    await store.saveActive(loosePath);
    await nextTick();
    await store.compileActive();
  }
  const looseAssigned = {
    activePath: active.value.path,
    textHasDocumentSet: active.value.text.includes(`documentSet: ${boardSet}`),
    saved: !active.value.dirty,
  };
  evidence.looseAssigned = looseAssigned;
  record(
    "native workflow assigned loose tab to document set",
    active.value.path === loosePath && looseAssigned.textHasDocumentSet && looseAssigned.saved,
    JSON.stringify(looseAssigned),
  );

  const boardGroupAfterDrop = groupedDocuments.value.find((group) => group.key === `set:${boardSet}`);
  const closeGroupPaths = boardGroupAfterDrop?.documents.map((document) => document.path).filter(Boolean) || [];
  if (boardGroupAfterDrop) {
    closeTabGroup(boardGroupAfterDrop);
    await waitForNativeWorkflowCondition(() => closeGroupPaths.every((path) => !store.documents.some((document) => document.path === path)), 1000);
  }
  const closeGroupEvidence = {
    closedPaths: closeGroupPaths,
    openPaths: store.documents.map((document) => document.path).filter(Boolean),
    recentlyClosed: store.recentlyClosed.slice(0, 6),
  };
  evidence.closeGroup = closeGroupEvidence;
  record(
    "native workflow closed document-set tab group",
    closeGroupPaths.length >= 2 && closeGroupPaths.every((path) => !store.documents.some((document) => document.path === path)),
    JSON.stringify(closeGroupEvidence),
  );

  await store.openRecentPath(boardTwoPath);
  await nextTick();
  const recentReopen = {
    activePath: active.value.path,
    recentlyClosed: store.recentlyClosed.slice(0, 6),
  };
  evidence.recentReopen = recentReopen;
  record(
    "native workflow reopened recently closed tab",
    active.value.path === boardTwoPath && !store.recentlyClosed.includes(boardTwoPath),
    JSON.stringify(recentReopen),
  );

  await store.openPath(boardOnePath);
  await nextTick();
  const restoredBoardOne = store.documents.find((document) => document.path === boardOnePath);
  if (restoredBoardOne) {
    store.setPinned(restoredBoardOne.id, true);
    store.setDocumentScroll(restoredBoardOne.id, { editor: 0.42, preview: 0.58 }, true);
  }
  store.mode = "review";
  store.sidebar = "review";
  await store.persistWorkspace();
  await store.restoreWorkspace(
    [boardOnePath, boardTwoPath],
    boardTwoPath,
    [boardOnePath],
    {
      [boardOnePath]: { editor: 0.42, preview: 0.58 },
      [boardTwoPath]: { editor: 0.12, preview: 0.34 },
    },
  );
  await nextTick();
  await store.compileActive();
  const restoredPinned = store.documents.find((document) => document.path === boardOnePath);
  const restoredActive = active.value;
  const restoreEvidence = {
    paths: store.documents.map((document) => document.path).filter(Boolean),
    activePath: restoredActive.path,
    pinnedPath: restoredPinned?.path || "",
    pinned: restoredPinned?.pinned === true,
    editorScrollRatio: restoredPinned?.editorScrollRatio,
    previewScrollRatio: restoredPinned?.previewScrollRatio,
  };
  evidence.restore = restoreEvidence;
  record(
    "native workflow restored workspace tabs with active pinned and scroll state",
    restoreEvidence.activePath === boardTwoPath &&
      restoredPinned?.pinned === true &&
      Math.abs((restoredPinned.editorScrollRatio || 0) - 0.42) < 0.001 &&
      Math.abs((restoredPinned.previewScrollRatio || 0) - 0.58) < 0.001,
    JSON.stringify(restoreEvidence),
  );

  return evidence;
}

async function writeDesktopWorkflowSmokeReport(payload: Record<string, unknown>) {
  const written = invoke("write_desktop_workflow_smoke_report", { payload }).catch(() => undefined);
  await Promise.race([written, nativeWorkflowDelay(750)]);
}

function nativeWorkflowDelay(ms: number) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

async function collectNativeFileWorkflowEvidence(record: (name: string, passed: boolean, detail?: string) => void) {
  const filePath = await invoke<string | null>("desktop_workflow_smoke_file_path", { extension: "md" }).catch(() => null);
  if (!filePath) {
    record("native workflow resolved real file path", false);
    return null;
  }
  await store.saveActive(filePath);
  await nextTick();
  await waitForNativeWorkflowCondition(() => !document.title.startsWith("* ") && document.title.includes(active.value.title), 800);
  const savedDocumentId = active.value.id;
  const savedText = active.value.text;
  record(
    "native workflow saved document to real file",
    active.value.path === filePath && !active.value.dirty && active.value.savedHash.length > 0,
    JSON.stringify({ filePath, title: active.value.title, dirty: active.value.dirty }),
  );
  record(
    "native workflow save cleared native title",
    !document.title.startsWith("* ") && document.title.includes(active.value.title),
    JSON.stringify({ documentTitle: document.title, activeTitle: active.value.title, dirty: active.value.dirty }),
  );

  store.newDocument();
  await nextTick();
  record(
    "native workflow created new document",
    active.value.title === "Untitled" && active.value.dirty && active.value.text.includes("Market Entry Report"),
    JSON.stringify({ title: active.value.title, dirty: active.value.dirty }),
  );

  store.closeDocument(savedDocumentId);
  await nextTick();
  await store.openPath(filePath);
  await nextTick();
  record(
    "native workflow opened saved real file",
    active.value.path === filePath && active.value.text === savedText && !active.value.dirty,
    JSON.stringify({ filePath: active.value.path, title: active.value.title, dirty: active.value.dirty }),
  );

  await setNativeWorkflowText(`${active.value.text}\n\nNative smoke revert marker.`);
  record("native workflow dirtied opened real file", active.value.dirty, active.value.title);
  await waitForNativeWorkflowCondition(() => document.title.startsWith("* "), 800);
  record(
    "native workflow dirtied native title for opened real file",
    document.title.startsWith("* ") && document.title.includes(active.value.title),
    JSON.stringify({ documentTitle: document.title, activeTitle: active.value.title, dirty: active.value.dirty }),
  );
  await store.revertActive();
  await nextTick();
  previewTextCommit.cancel();
  await waitForNativeWorkflowCondition(
    () => active.value.path === filePath && active.value.text === savedText && !active.value.dirty && !document.title.startsWith("* "),
    800,
  );
  record(
    "native workflow reverted saved real file",
    active.value.path === filePath && active.value.text === savedText && !active.value.dirty,
    JSON.stringify({ filePath: active.value.path, title: active.value.title, dirty: active.value.dirty }),
  );
  record(
    "native workflow revert cleared native title",
    !document.title.startsWith("* ") && document.title.includes(active.value.title),
    JSON.stringify({ documentTitle: document.title, activeTitle: active.value.title, dirty: active.value.dirty }),
  );

  const watcherReloadText = `${savedText}\n\nNative clean watcher reload marker.`;
  await invoke("save_file", { request: { path: filePath, text: watcherReloadText, expected_hash: null } });
  await waitForNativeWorkflowCondition(
    () => active.value.path === filePath && active.value.text === watcherReloadText && !active.value.dirty && !store.externalConflict,
    2000,
  );
  record(
    "native workflow reloaded clean external watcher change",
    active.value.path === filePath && active.value.text === watcherReloadText && !active.value.dirty && !store.externalConflict,
    JSON.stringify({
      title: active.value.title,
      dirty: active.value.dirty,
      statusMessage: store.statusMessage,
      watchDriver: store.watchDriver,
    }),
  );
  await setNativeWorkflowText(savedText);
  await store.saveActive(filePath);
  await nextTick();
  await waitForNativeWorkflowCondition(
    () => active.value.path === filePath && active.value.text === savedText && !active.value.dirty && !store.externalConflict,
    800,
  );
  record(
    "native workflow restored clean watcher reload",
    active.value.path === filePath && active.value.text === savedText && !active.value.dirty && !store.externalConflict,
    JSON.stringify({ title: active.value.title, dirty: active.value.dirty, statusMessage: store.statusMessage }),
  );

  const includePath = await invoke<string | null>("desktop_workflow_smoke_file_path", { extension: "include" }).catch(() => null);
  if (includePath) {
    const includeInitialText = "## Native Include\n\nNative include watcher initial.";
    const includeUpdatedText = "## Native Include\n\nNative include watcher updated.";
    const rootWithInclude = `${savedText}\n\n!include ${includePath}\n`;
    await invoke("save_file", { request: { path: includePath, text: includeInitialText, expected_hash: null } });
    await setNativeWorkflowText(rootWithInclude);
    await store.saveActive(filePath);
    await nextTick();
    await store.compileActive();
    await waitForNativeWorkflowCondition(
      () =>
        active.value.path === filePath &&
        !active.value.dirty &&
        store.watchDriver === "native" &&
        store.watchedPaths.some((path) => path === includePath) &&
        Boolean(active.value.compile?.html.includes("Native include watcher initial")),
      2000,
    );
    record(
      "native workflow watched included file with native driver",
      active.value.path === filePath &&
        !active.value.dirty &&
        store.watchDriver === "native" &&
        store.watchedPaths.some((path) => path === includePath) &&
        Boolean(active.value.compile?.html.includes("Native include watcher initial")),
      JSON.stringify({
        includePath,
        watchDriver: store.watchDriver,
        watchedPaths: store.watchedPaths,
      }),
    );
    await invoke("save_file", { request: { path: includePath, text: includeUpdatedText, expected_hash: null } });
    await waitForNativeWorkflowCondition(
      () => Boolean(active.value.compile?.html.includes("Native include watcher updated")) && !active.value.dirty && !store.externalConflict,
      2000,
    );
    record(
      "native workflow recompiled clean included watcher change",
      Boolean(active.value.compile?.html.includes("Native include watcher updated")) && !active.value.dirty && !store.externalConflict,
      JSON.stringify({
        title: active.value.title,
        dirty: active.value.dirty,
        statusMessage: store.statusMessage,
        watchDriver: store.watchDriver,
      }),
    );
    await setNativeWorkflowText(savedText);
    await store.saveActive(filePath);
    await nextTick();
    await store.compileActive();
    await waitForNativeWorkflowCondition(
      () => active.value.path === filePath && active.value.text === savedText && !active.value.dirty && !store.externalConflict,
      800,
    );
    record(
      "native workflow restored included watcher root",
      active.value.path === filePath && active.value.text === savedText && !active.value.dirty && !store.externalConflict,
      JSON.stringify({ title: active.value.title, dirty: active.value.dirty, statusMessage: store.statusMessage }),
    );
  } else {
    record("native workflow resolved included watcher path", false);
  }

  const externalText = `${savedText}\n\nExternal native conflict edit.`;
  const localText = `${savedText}\n\nLocal unsaved native conflict edit.`;
  await invoke("save_file", { request: { path: filePath, text: externalText, expected_hash: null } });
  await setNativeWorkflowText(localText);
  await store.saveActive();
  await nextTick();
  record(
    "native workflow blocked stale save with external conflict",
    Boolean(
      store.externalConflict?.reason === "root" &&
        store.externalConflict.path === filePath &&
        Boolean(store.externalConflict.externalText?.includes("External native conflict edit")) &&
        active.value.text.includes("Local unsaved native conflict edit"),
    ),
    JSON.stringify({
      conflictPath: store.externalConflict?.path,
      reason: store.externalConflict?.reason,
      statusMessage: store.statusMessage,
    }),
  );

  conflictOpen.value = true;
  await nextTick();
  await waitForNativeWorkflowCondition(() => Boolean(document.querySelector('[aria-label="External file conflict"]')), 800);
  const conflictModal = document.querySelector('[aria-label="External file conflict"]') as HTMLElement | null;
  record(
    "native workflow rendered conflict modal controls",
    Boolean(
      conflictModal?.textContent?.includes("External Changes") &&
        conflictModal.textContent.includes(filePath) &&
        conflictModal.textContent.includes("Local unsaved native conflict edit") &&
        conflictModal.textContent.includes("External native conflict edit") &&
        nativeWorkflowButtonExists("Use local as merge base", conflictModal) &&
        nativeWorkflowButtonExists("Use external as merge base", conflictModal) &&
        nativeWorkflowButtonExists("Apply merged text", conflictModal) &&
        nativeWorkflowButtonExists("Keep local", conflictModal) &&
        nativeWorkflowButtonExists("Save copy", conflictModal) &&
        nativeWorkflowButtonExists("Accept external", conflictModal),
    ),
    JSON.stringify({
      hasDialog: Boolean(conflictModal),
      conflictText: conflictModal?.textContent?.slice(0, 300) || "",
    }),
  );
  await clickNativeWorkflowButton("Use local as merge base", conflictModal);
  await waitForNativeWorkflowCondition(() => mergedConflictText.value.includes("Local unsaved native conflict edit"), 800);
  record(
    "native workflow conflict modal seeded local merge base",
    mergedConflictText.value.includes("Local unsaved native conflict edit"),
    JSON.stringify({ mergedConflictText: mergedConflictText.value.slice(-160) }),
  );
  await clickNativeWorkflowButton("Use external as merge base", conflictModal);
  await waitForNativeWorkflowCondition(() => mergedConflictText.value.includes("External native conflict edit"), 800);
  record(
    "native workflow conflict modal seeded external merge base",
    mergedConflictText.value.includes("External native conflict edit"),
    JSON.stringify({ mergedConflictText: mergedConflictText.value.slice(-160) }),
  );
  closeConflictDialog();
  await nextTick();

  await store.keepLocalChanges();
  await nextTick();
  record(
    "native workflow kept local conflict changes",
    !store.externalConflict && active.value.dirty && active.value.text.includes("Local unsaved native conflict edit"),
    JSON.stringify({ title: active.value.title, dirty: active.value.dirty, statusMessage: store.statusMessage }),
  );
  await store.saveActive(filePath);
  await nextTick();
  await waitForNativeWorkflowCondition(
    () => active.value.path === filePath && !active.value.dirty && !store.externalConflict,
    800,
  );
  record(
    "native workflow saved kept-local conflict changes",
    active.value.path === filePath && active.value.text.includes("Local unsaved native conflict edit") && !active.value.dirty && !store.externalConflict,
    JSON.stringify({ title: active.value.title, dirty: active.value.dirty, statusMessage: store.statusMessage }),
  );

  const saveCopyExternalText = `${savedText}\n\nExternal save-copy native conflict edit.`;
  const saveCopyLocalText = `${savedText}\n\nSave-copy native conflict edit.`;
  await openNativeWorkflowConflict(filePath, saveCopyExternalText, saveCopyLocalText);
  const copyPath = await invoke<string | null>("desktop_workflow_smoke_export_path", { extension: "md" }).catch(() => null);
  if (copyPath) {
    await store.saveLocalConflictCopy(copyPath);
    await nextTick();
  }
  record(
    "native workflow saved local conflict copy",
    Boolean(copyPath && active.value.path === copyPath && active.value.text.includes("Save-copy native conflict edit") && !store.externalConflict && !active.value.dirty),
    JSON.stringify({ copyPath, activePath: active.value.path, dirty: active.value.dirty, statusMessage: store.statusMessage }),
  );

  const mergeExternalText = `${savedText}\n\nExternal merge native conflict edit.`;
  const mergeLocalText = `${savedText}\n\nLocal merge native conflict edit.`;
  const mergedText = `${savedText}\n\nMerged native conflict edit.\nExternal merge native conflict edit.\nLocal merge native conflict edit.`;
  await openNativeWorkflowConflict(filePath, mergeExternalText, mergeLocalText);
  await store.applyConflictMerge(mergedText);
  await nextTick();
  await store.saveActive(filePath);
  await nextTick();
  await waitForNativeWorkflowCondition(
    () => active.value.path === filePath && active.value.text.includes("Merged native conflict edit") && !active.value.dirty && !store.externalConflict,
    800,
  );
  record(
    "native workflow merged external conflict changes",
    active.value.path === filePath &&
      active.value.text.includes("Merged native conflict edit") &&
      active.value.text.includes("External merge native conflict edit") &&
      active.value.text.includes("Local merge native conflict edit") &&
      !active.value.dirty &&
      !store.externalConflict,
    JSON.stringify({ title: active.value.title, dirty: active.value.dirty, statusMessage: store.statusMessage }),
  );

  await openNativeWorkflowConflict(filePath, externalText, localText);
  await store.acceptExternalChanges();
  await nextTick();
  await waitForNativeWorkflowCondition(
    () => active.value.path === filePath && active.value.text.includes("External native conflict edit") && !active.value.dirty && !store.externalConflict,
    800,
  );
  record(
    "native workflow accepted external conflict changes",
    active.value.path === filePath && active.value.text.includes("External native conflict edit") && !active.value.dirty && !store.externalConflict,
    JSON.stringify({ title: active.value.title, dirty: active.value.dirty, statusMessage: store.statusMessage }),
  );

  await setNativeWorkflowText(savedText);
  await store.saveActive(filePath);
  await nextTick();
  record(
    "native workflow restored real file after conflict proof",
    active.value.path === filePath && active.value.text === savedText && !active.value.dirty && !store.externalConflict,
    JSON.stringify({ title: active.value.title, dirty: active.value.dirty, statusMessage: store.statusMessage }),
  );

  return {
    filePath,
    copyPath,
    includePath,
    title: active.value.title,
    recentFiles: store.recentFiles.slice(0, 5),
    recentlyClosed: store.recentlyClosed.slice(0, 5),
  };
}

async function openNativeWorkflowConflict(filePath: string, externalText: string, localText: string) {
  await store.openPath(filePath);
  await nextTick();
  await invoke("save_file", { request: { path: filePath, text: externalText, expected_hash: null } });
  await setNativeWorkflowText(localText);
  await store.saveActive();
  await nextTick();
  await waitForNativeWorkflowCondition(
    () => store.externalConflict?.reason === "root" && store.externalConflict.path === filePath && active.value.text === localText,
    800,
  );
}

async function setNativeWorkflowText(text: string) {
  previewTextCommit.cancel();
  store.updateText(text);
  await nextTick();
  previewTextCommit.cancel();
}

async function waitForNativeWorkflowCondition(check: () => boolean, timeoutMs: number) {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    await new Promise((resolve) => window.setTimeout(resolve, 120));
    await nextTick();
    if (check()) return true;
  }
  return check();
}

function nativeWorkflowButtonExists(label: string, root: ParentNode | null = document) {
  return Boolean(nativeWorkflowButton(label, root));
}

function nativeWorkflowButton(label: string, root: ParentNode | null = document) {
  return Array.from(root?.querySelectorAll("button") || []).find((button) => button.textContent?.trim() === label) as HTMLButtonElement | undefined;
}

async function clickNativeWorkflowButton(label: string, root: ParentNode | null = document) {
  const button = nativeWorkflowButton(label, root);
  if (!button) return false;
  button.click();
  await nextTick();
  return true;
}

async function collectNativeModeEvidence(record: (name: string, passed: boolean, detail?: string) => void) {
  const modes: Array<typeof store.mode> = ["split", "source", "preview", "focus", "export", "review", "presentation"];
  type NativeModeEvidence = {
    mode: typeof store.mode;
    workspaceClass: string;
    sidebar: string;
    sourceVisible: boolean;
    previewVisible: boolean;
    sidebarText: string;
    previewText: string;
  };
  const expectedSidebar: Partial<Record<typeof store.mode, string>> = {
    export: "exports",
    review: "review",
    presentation: "outline",
  };
  const originalTarget = store.exportTarget;
  const originalManifestDefault = store.exportDefaults.includeManifest;
  store.exportTarget = "html";
  store.exportDefaults.includeManifest = true;
  await store.compileActive();
  await nextTick();
  const surfaceText = (selector: string) => document.querySelector(selector)?.textContent?.replace(/\s+/g, " ").trim() || "";
  const surfaceVisible = (selector: string) => {
    const element = document.querySelector(selector) as HTMLElement | null;
    if (!element) return false;
    const style = window.getComputedStyle(element);
    return style.display !== "none" && style.visibility !== "hidden";
  };
  try {
    const evidence: NativeModeEvidence[] = [];
    for (const mode of modes) {
      store.mode = mode;
      await nextTick();
      const workspaceClass = document.querySelector("#document-workspace")?.className || "";
      const sidebar = store.sidebar;
      const sourceVisible = surfaceVisible("#markdown-source");
      const previewVisible = surfaceVisible("#live-preview");
      const sidebarText = surfaceText("#document-sidebar").slice(0, 900);
      const previewText = surfaceText("#live-preview").slice(0, 1400);
      const entry = { mode, workspaceClass, sidebar, sourceVisible, previewVisible, sidebarText, previewText };
      const passed = workspaceClass.includes(`mode-${mode}`) && (!expectedSidebar[mode] || sidebar === expectedSidebar[mode]);
      record(`native workflow switched ${mode} mode`, passed, JSON.stringify(entry));
      evidence.push(entry);
    }
    const byMode = (mode: typeof store.mode) => evidence.find((entry) => entry.mode === mode);
    const exportMode = byMode("export");
    const reviewMode = byMode("review");
    const presentationMode = byMode("presentation");
    record(
      "native workflow rendered export mode preview content",
      Boolean(
        exportMode?.previewVisible &&
          !exportMode.sourceVisible &&
          exportMode.previewText.includes("HTML export preview") &&
          exportMode.previewText.includes("Market Entry Report") &&
          exportMode.sidebarText.includes("HTML delivery"),
      ),
      JSON.stringify(exportMode),
    );
    record(
      "native workflow rendered review mode governance content",
      Boolean(
        reviewMode?.sourceVisible &&
          reviewMode.previewVisible &&
          reviewMode.sidebarText.includes("Review") &&
          reviewMode.sidebarText.includes("Summary") &&
          reviewMode.sidebarText.includes("Approved by"),
      ),
      JSON.stringify(reviewMode),
    );
    record(
      "native workflow rendered presentation outline content",
      Boolean(
        presentationMode?.previewVisible &&
          !presentationMode.sourceVisible &&
          presentationMode.sidebarText.includes("Outline") &&
          presentationMode.previewText.includes("Market Entry Report"),
      ),
      JSON.stringify(presentationMode),
    );
    return evidence;
  } finally {
    store.exportTarget = originalTarget;
    store.exportDefaults.includeManifest = originalManifestDefault;
    store.mode = "split";
    await nextTick();
  }
}

async function collectNativeThemeAccessibilityEvidence(record: (name: string, passed: boolean, detail?: string) => void) {
  const original = {
    theme: store.theme,
    previewTheme: store.previewTheme,
    highContrast: store.highContrast,
    reducedMotion: store.reducedMotion,
    editorFontSize: store.editorFontSize,
    previewFontSize: store.previewFontSize,
    previewLineHeight: store.previewLineHeight,
  };
  store.theme = "dark";
  store.previewTheme = "dark";
  store.highContrast = true;
  store.reducedMotion = true;
  store.editorFontSize = 18;
  store.previewFontSize = 19;
  store.previewLineHeight = 1.9;
  await nextTick();
  await nextTick();

  const shell = document.querySelector(".app-shell") as HTMLElement | null;
  const commandButton = Array.from(document.querySelectorAll("#main-commands button")).find((button) =>
    button.textContent?.replace(/\s+/g, " ").trim().includes("Commands"),
  ) as HTMLElement | undefined;
  const editorContent = document.querySelector(".cm-content") as HTMLElement | null;
  const previewPane = document.querySelector(".preview-pane") as HTMLElement | null;
  const previewDocument = document.querySelector(".preview-document") as HTMLElement | null;
  const shellStyle = shell ? getComputedStyle(shell) : null;
  const buttonStyle = commandButton ? getComputedStyle(commandButton) : null;
  const editorStyle = editorContent ? getComputedStyle(editorContent) : null;
  const evidence = {
    shellTheme: shell?.dataset.theme || "",
    highContrast: shell?.dataset.highContrast || "",
    reducedMotion: shell?.dataset.reducedMotion || "",
    previewTheme: previewPane?.dataset.previewTheme || "",
    shellBackgroundColor: shellStyle?.backgroundColor || "",
    commandBorderColor: buttonStyle?.borderTopColor || "",
    editorTransitionDuration: editorStyle?.transitionDuration || "",
    editorFontSize: editorStyle?.fontSize || "",
    previewStyle: previewDocument?.getAttribute("style") || "",
  };
  record("native workflow applied dark theme attribute", evidence.shellTheme === "dark", evidence.shellTheme);
  record("native workflow applied high contrast attributes and colors", evidence.highContrast === "true" && evidence.commandBorderColor === "rgb(0, 0, 0)", JSON.stringify(evidence));
  record("native workflow applied reduced motion", evidence.reducedMotion === "true" && evidence.editorTransitionDuration === "0s", evidence.editorTransitionDuration);
  record("native workflow applied editor typography", evidence.editorFontSize === "18px", evidence.editorFontSize);
  record(
    "native workflow applied preview theme and typography",
    evidence.previewTheme === "dark" && evidence.previewStyle.includes("font-size: 19px") && evidence.previewStyle.includes("line-height: 1.9"),
    evidence.previewStyle,
  );

  store.theme = original.theme;
  store.previewTheme = original.previewTheme;
  store.highContrast = original.highContrast;
  store.reducedMotion = original.reducedMotion;
  store.editorFontSize = original.editorFontSize;
  store.previewFontSize = original.previewFontSize;
  store.previewLineHeight = original.previewLineHeight;
  await nextTick();
  await store.persistWorkspace();
  return evidence;
}

function smokeSnippetAround(text: string, needle: string) {
  const index = text.indexOf(needle);
  if (index < 0) return text.slice(0, 800);
  const start = Math.max(0, index - 240);
  return text.slice(start, start + 800);
}

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
    ...(store.codeFolding ? [foldGutter(), codeFolding({ placeholderText: " folded " })] : []),
    lintGutter(),
    history(),
    EditorState.allowMultipleSelections.of(true),
    markdown(),
    linter(editorDiagnostics, { delay: 150 }),
    semanticEditorDecorations,
    closeBrackets(),
    EditorView.contentAttributes.of({
      role: "textbox",
      "aria-label": "Markdown editor",
      "aria-multiline": "true",
      spellcheck: "true",
      autocapitalize: "sentences",
    }),
    keymap.of([
      { key: "Enter", run: continueMarkdownList },
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...historyKeymap,
      ...searchKeymap,
      ...(store.codeFolding ? foldKeymap : []),
    ]),
    EditorView.domEventHandlers({
      scroll: () => {
        syncPreviewScrollFromEditor();
      },
    }),
    ...(store.wordWrap ? [EditorView.lineWrapping] : []),
    EditorView.updateListener.of((update) => {
      if (!update.docChanged) return;
      previewTextCommit.schedule(update.state.doc.toString());
    }),
    EditorView.theme({
      "&": {
        height: "100%",
        fontSize: `${clampUiFontSize(store.editorFontSize)}px`,
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
    collectRegexDecorations(inlineDecorations, text, /\[[^\]\n]*@[A-Za-z0-9_:-]+[^\]\n]*\]/g, "cm-neditor-citation");
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
  const continuation = markdownListContinuation(beforeCursor);
  if (!continuation) return false;

  if (continuation.kind === "exit") {
    view.dispatch({
      changes: {
        from: line.from + continuation.fromColumn,
        to: selection.head,
        insert: continuation.replacement,
      },
      selection: { anchor: line.from + continuation.fromColumn + continuation.replacement.length },
      scrollIntoView: true,
    });
    return true;
  }

  view.dispatch({
    changes: { from: selection.head, insert: continuation.insert },
    selection: { anchor: selection.head + continuation.insert.length },
    scrollIntoView: true,
  });
  return true;
}

function editorDiagnostics(view: EditorView): CodeMirrorDiagnostic[] {
  return (active.value.compile?.diagnostics || []).flatMap((diagnostic) => codeMirrorDiagnostic(view, diagnostic));
}

function codeMirrorDiagnostic(view: EditorView, diagnostic: DocumentDiagnostic): CodeMirrorDiagnostic[] {
  if (!diagnostic.line || diagnosticAppliesToIncludedFile(diagnostic)) return [];
  const range = diagnosticEditorRange(view, diagnostic);
  const message = [diagnostic.message, diagnostic.suggestion, ...diagnostic.related].filter(Boolean).join("\n");
  return [
    {
      from: range.from,
      to: range.to,
      severity: diagnostic.severity,
      message,
      source: diagnostic.source_file || "compiler",
    },
  ];
}

function diagnosticEditorRange(view: EditorView, diagnostic: DocumentDiagnostic) {
  const startLine = view.state.doc.line(Math.max(1, Math.min(diagnostic.line || 1, view.state.doc.lines)));
  if (!diagnostic.column) {
    return { from: startLine.from, to: Math.max(startLine.from + 1, startLine.to) };
  }
  const endLine = view.state.doc.line(
    Math.max(1, Math.min(diagnostic.end_line || diagnostic.line || 1, view.state.doc.lines)),
  );
  const from = startLine.from + clampColumnOffset(diagnostic.column, startLine.length);
  const to = endLine.from + clampColumnOffset(diagnostic.end_column, endLine.length);
  return { from, to: Math.max(from + 1, to) };
}

function clampColumnOffset(column: number | null | undefined, lineLength: number) {
  if (!column || column < 1) return 0;
  return Math.max(0, Math.min(column - 1, lineLength));
}

function diagnosticAppliesToIncludedFile(diagnostic: DocumentDiagnostic) {
  const sourceFile = diagnostic.source_file;
  const activePath = active.value.path;
  return Boolean(sourceFile && activePath && sourceFile !== activePath);
}

function canNavigateDiagnostic(diagnostic: DocumentDiagnostic) {
  return Boolean(diagnostic.line);
}

function diagnosticLocation(diagnostic: DocumentDiagnostic) {
  const parts = [diagnostic.source_file, diagnostic.line ? `line ${diagnostic.line}` : ""].filter(Boolean);
  return parts.join(": ");
}

function diagnosticAnnouncementLabel(diagnostic: DocumentDiagnostic) {
  const location = diagnosticLocation(diagnostic);
  const suggestion = diagnostic.suggestion ? ` Suggested fix: ${diagnostic.suggestion}` : "";
  return `${diagnostic.severity} diagnostic: ${diagnostic.message}${location ? ` at ${location}` : ""}${suggestion}`;
}

function conflictDiffCellLabel(row: ConflictDiffRow, source: ConflictMergeSource) {
  const line = source === "local" ? row.localLine : row.externalLine;
  const text = source === "local" ? row.local : row.external;
  const side = source === "local" ? "Local" : "External";
  const change = row.kind === "equal" ? "unchanged" : row.kind === source ? "changed" : "empty";
  const location = line === null ? "no matching line" : `line ${line}`;
  return `${side} ${change} ${location}: ${text.trim() || "blank line"}`;
}

function previewGeneratedLineForDiagnostic(diagnostic: DocumentDiagnostic) {
  const compile = active.value.compile;
  const sourceLine = diagnostic.line || 1;
  const sourceFile = diagnostic.source_file ? normalizeDocumentPath(diagnostic.source_file) : "";
  const sourceMap = compile?.source_map || [];
  const exact = sourceMap.find((entry) => {
    const fileMatches = !sourceFile || normalizeDocumentPath(entry.source_file) === sourceFile;
    return fileMatches && entry.source_line === sourceLine;
  });
  if (exact) return Math.max(1, exact.generated_line);
  const nearest = sourceMap
    .filter((entry) => !sourceFile || normalizeDocumentPath(entry.source_file) === sourceFile)
    .filter((entry) => entry.source_line >= sourceLine)
    .sort((left, right) => left.source_line - right.source_line)[0];
  return Math.max(1, nearest?.generated_line || sourceLine);
}

function inlinePreviewDiagnostics(html: string, diagnostics: PreviewDiagnosticItem[]) {
  if (!diagnostics.length) return html;
  const lines = html.split("\n");
  const diagnosticsByLine = new Map<number, PreviewDiagnosticItem[]>();
  const maxLine = Math.max(1, lines.length);
  for (const diagnostic of diagnostics) {
    const line = Math.max(1, Math.min(diagnostic.generatedLine || 1, maxLine + 1));
    diagnosticsByLine.set(line, [...(diagnosticsByLine.get(line) || []), diagnostic]);
  }
  const output: string[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    output.push(...(diagnosticsByLine.get(index + 1) || []).map(renderPreviewDiagnostic));
    output.push(lines[index]);
  }
  output.push(...(diagnosticsByLine.get(maxLine + 1) || []).map(renderPreviewDiagnostic));
  return output.join("\n");
}

function renderPreviewDiagnostic(diagnostic: PreviewDiagnosticItem) {
  const location = diagnosticLocation(diagnostic);
  const related = diagnostic.related.length
    ? `<ul>${diagnostic.related.map((item) => `<li>${escapePreviewHtml(item)}</li>`).join("")}</ul>`
    : "";
  const sourceFile = diagnostic.source_file || active.value.path || "";
  return [
    `<aside class="preview-diagnostic ${escapePreviewAttribute(diagnostic.severity)}" role="note" aria-label="${escapePreviewAttribute(
      `${diagnostic.severity} preview diagnostic`,
    )}">`,
    `<strong>${escapePreviewHtml(diagnostic.severity)}</strong>`,
    `<p>${escapePreviewHtml(diagnostic.message)}</p>`,
    location ? `<small>${escapePreviewHtml(location)}</small>` : "",
    diagnostic.suggestion ? `<small>${escapePreviewHtml(diagnostic.suggestion)}</small>` : "",
    related,
    `<button type="button" class="preview-diagnostic-jump" data-source-file="${escapePreviewAttribute(sourceFile)}" data-line="${escapePreviewAttribute(
      String(diagnostic.line || ""),
    )}" data-column="${escapePreviewAttribute(String(diagnostic.column || ""))}" data-end-line="${escapePreviewAttribute(
      String(diagnostic.end_line || diagnostic.line || ""),
    )}" data-end-column="${escapePreviewAttribute(String(diagnostic.end_column || ""))}">Go to source</button>`,
    "</aside>",
  ].join("");
}

function escapePreviewHtml(value: string) {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function escapePreviewAttribute(value: string) {
  return escapePreviewHtml(value).replace(/"/g, "&quot;");
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
  void nextTick(() => restoreActiveScrollPosition());
}

function syncPreviewScrollFromEditor() {
  if (!editorView || !previewPane.value || syncingScroll) return;
  syncingScroll = true;
  syncScrollPosition(editorView.scrollDOM, previewPane.value);
  recordActiveScrollPosition();
  scheduleScrollPositionPersist();
  window.requestAnimationFrame(() => {
    syncingScroll = false;
  });
}

function syncEditorScrollFromPreview() {
  if (!editorView || !previewPane.value || syncingScroll) return;
  syncingScroll = true;
  syncScrollPosition(previewPane.value, editorView.scrollDOM);
  recordActiveScrollPosition();
  scheduleScrollPositionPersist();
  window.requestAnimationFrame(() => {
    syncingScroll = false;
  });
}

function syncScrollPosition(source: HTMLElement, target: HTMLElement) {
  applyScrollRatio(target, readScrollRatio(source));
}

function readScrollRatio(element: HTMLElement) {
  const range = element.scrollHeight - element.clientHeight;
  if (range <= 0) return 0;
  return Math.min(Math.max(element.scrollTop / range, 0), 1);
}

function applyScrollRatio(element: HTMLElement, ratio = 0) {
  const range = Math.max(0, element.scrollHeight - element.clientHeight);
  element.scrollTop = range * Math.min(Math.max(ratio, 0), 1);
}

function restoreActiveScrollPosition() {
  if (!editorView) return;
  restoringScroll = true;
  applyScrollRatio(editorView.scrollDOM, active.value.editorScrollRatio);
  if (previewPane.value) {
    applyScrollRatio(previewPane.value, active.value.previewScrollRatio ?? active.value.editorScrollRatio);
  }
  window.requestAnimationFrame(() => {
    restoringScroll = false;
  });
}

function recordActiveScrollPosition(persist = false) {
  if (!editorView || restoringScroll) return;
  store.setDocumentScroll(
    active.value.id,
    {
      editor: readScrollRatio(editorView.scrollDOM),
      preview: previewPane.value ? readScrollRatio(previewPane.value) : undefined,
    },
    persist,
  );
}

function scheduleScrollPositionPersist() {
  if (restoringScroll) return;
  window.clearTimeout(scrollPersistHandle);
  scrollPersistHandle = window.setTimeout(() => {
    recordActiveScrollPosition(true);
  }, 250);
}

function startPaneResize(event: PointerEvent) {
  event.preventDefault();
  resizeEditorPane(event);
  window.addEventListener("pointermove", resizeEditorPane);
  window.addEventListener("pointerup", stopPaneResize, { once: true });
}

function stopPaneResize() {
  window.removeEventListener("pointermove", resizeEditorPane);
  void store.persistWorkspace();
}

function resizeEditorPane(event: PointerEvent) {
  const workspace = workspacePane.value;
  if (!workspace) return;
  const rect = workspace.getBoundingClientRect();
  const sidebarWidth = window.matchMedia("(max-width: 900px)").matches ? 0 : 260;
  const splitterWidth = 8;
  const availableWidth = rect.width - sidebarWidth - splitterWidth;
  if (availableWidth <= 0) return;
  const x = event.clientX - rect.left - sidebarWidth;
  store.setEditorPaneRatio(x / availableWidth, false);
}

function handlePaneSplitterKeydown(event: KeyboardEvent) {
  const keyStep = event.shiftKey ? 0.1 : 0.025;
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    store.setEditorPaneRatio(store.editorPaneRatio - keyStep);
  } else if (event.key === "ArrowRight") {
    event.preventDefault();
    store.setEditorPaneRatio(store.editorPaneRatio + keyStep);
  } else if (event.key === "Home") {
    event.preventDefault();
    store.setEditorPaneRatio(0.25);
  } else if (event.key === "End") {
    event.preventDefault();
    store.setEditorPaneRatio(0.75);
  }
}

function focusSkipTarget(event: Event) {
  const link = event.currentTarget as HTMLAnchorElement | null;
  const targetId = link?.hash?.slice(1);
  if (!targetId) return;
  const target = document.getElementById(targetId);
  if (!target) return;
  event.preventDefault();
  target.scrollIntoView({ block: "nearest", inline: "nearest" });
  target.focus({ preventScroll: true });
}

async function handleModalStateChange(open: boolean, dialogRef: { value: HTMLElement | null }) {
  if (open) {
    modalReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    await nextTick();
    focusFirstModalControl(dialogRef.value);
  } else {
    restoreModalFocus();
  }
}

function focusFirstModalControl(dialog: HTMLElement | null) {
  if (!dialog) return;
  const initial = dialog.querySelector<HTMLElement>("[data-initial-focus]");
  const target = initial || modalFocusableElements(dialog)[0] || dialog;
  target.focus({ preventScroll: true });
}

function modalFocusableElements(dialog: HTMLElement) {
  return Array.from(
    dialog.querySelectorAll<HTMLElement>(
      [
        "a[href]",
        "button:not([disabled])",
        "input:not([disabled])",
        "select:not([disabled])",
        "textarea:not([disabled])",
        "[tabindex]:not([tabindex='-1'])",
      ].join(","),
    ),
  ).filter((element) => !element.hasAttribute("disabled") && element.offsetParent !== null);
}

function restoreModalFocus() {
  const target = modalReturnFocus;
  modalReturnFocus = null;
  if (target?.isConnected) {
    target.focus({ preventScroll: true });
  } else {
    editorView?.focus();
  }
}

function handleModalKeydown(kind: "ai-paste" | "command-palette" | "conflict", event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    closeModal(kind);
    return;
  }
  if (event.key !== "Tab") return;
  const dialog = event.currentTarget as HTMLElement;
  const focusable = modalFocusableElements(dialog);
  if (!focusable.length) {
    event.preventDefault();
    dialog.focus({ preventScroll: true });
    return;
  }
  const first = focusable[0];
  const last = focusable[focusable.length - 1];
  const activeElement = document.activeElement;
  if (event.shiftKey && activeElement === first) {
    event.preventDefault();
    last.focus({ preventScroll: true });
  } else if (!event.shiftKey && activeElement === last) {
    event.preventDefault();
    first.focus({ preventScroll: true });
  }
}

function closeModal(kind: "ai-paste" | "command-palette" | "conflict") {
  if (kind === "ai-paste") {
    closeAiPaste();
  } else if (kind === "command-palette") {
    closeCommandPalette();
  } else {
    closeConflictDialog();
  }
}

function closeCommandPalette() {
  commandPaletteOpen.value = false;
  commandQuery.value = "";
}

function closeConflictDialog() {
  conflictOpen.value = false;
}

function runEditorCommand(command: (view: EditorView) => boolean) {
  if (!editorView) return;
  command(editorView);
  editorView.focus();
}

function showOutline() {
  store.sidebar = "outline";
  void nextTick(() => {
    workspacePane.value?.focus();
  });
}

function openTransformTemplates() {
  store.sidebar = "templates";
  void nextTick(() => {
    workspacePane.value?.focus();
  });
}

function templateFillFields(template: Pick<TransformTemplate, "body" | "transform">) {
  return transformTemplateFillFields(template);
}

function insertTransformTemplate(template: TransformTemplate) {
  insertBlock(transformTemplateMarkdown(template));
  store.statusMessage = `Inserted ${template.name} template`;
}

function startNewCustomTemplate() {
  customTemplateDraft.value = blankCustomTransformTemplate();
  editingCustomTemplateId.value = "";
}

function duplicateTransformTemplate(template: TransformTemplate) {
  customTemplateDraft.value = {
    id: createCustomTransformTemplateId(),
    name: `${template.name} copy`,
    category: template.category,
    transform: template.transform,
    summary: template.summary,
    body: template.body,
    tags: [...template.tags],
  };
  editingCustomTemplateId.value = "";
}

function editCustomTransformTemplate(template: TransformTemplate) {
  customTemplateDraft.value = {
    id: template.id,
    name: template.name,
    category: template.category,
    transform: template.transform,
    summary: template.summary,
    body: template.body,
    tags: [...template.tags],
  };
  editingCustomTemplateId.value = template.id;
}

async function saveCustomTransformTemplate() {
  if (!customTemplateIsValid.value) return;
  await store.saveCustomTransformTemplate(customTemplateDraft.value);
  editingCustomTemplateId.value = customTemplateDraft.value.id;
  store.statusMessage = `Saved ${customTemplateDraft.value.name} template`;
}

function activate(id: string) {
  recordActiveScrollPosition(true);
  void store.activateDocument(id);
}

function closeTabGroup(group: DocumentTabGroup) {
  for (const document of [...group.documents]) {
    void closeDocument(document.id);
  }
}

async function closeDocument(id: string) {
  const document = store.documents.find((item) => item.id === id);
  if (!document) return;
  if (document.dirty) {
    const discard = await confirm(`Close ${document.title} and discard unsaved changes?`, {
      title: "Discard unsaved changes",
      kind: "warning",
    });
    if (!discard) {
      store.statusMessage = `Kept ${document.title} open`;
      return;
    }
  }
  store.closeDocument(id);
}

function dropTabOnGroup(group: DocumentTabGroup) {
  if (!draggedTabId.value) return;
  const document = store.documents.find((candidate) => candidate.id === draggedTabId.value);
  if (group.key.startsWith("set:") && document) {
    store.setPinned(document.id, false);
    store.setActiveDocument(document.id);
    store.updateText(upsertFrontMatterField(document.text, "documentSet", group.label));
  } else {
    store.setPinned(draggedTabId.value, group.key === "pinned");
  }
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
  const documentSet = documentSetName(document);
  if (documentSet) {
    return {
      key: `set:${documentSet}`,
      label: documentSet,
      title: `Document set: ${documentSet}`,
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

function documentSetName(document: OpenDocument) {
  const metadata = document.compile?.metadata || {};
  const value = metadata.documentSet || metadata.document_set || metadata.set;
  return typeof value === "string" && value.trim() ? value.trim() : "";
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

function displayDocumentPath(path: string) {
  const normalized = normalizeDocumentPath(path);
  const workspaceRoot = store.workspaceRoot ? normalizeDocumentPath(store.workspaceRoot) : "";
  if (workspaceRoot && normalized === workspaceRoot) return folderLabel(workspaceRoot);
  if (workspaceRoot && normalized.startsWith(`${workspaceRoot}/`)) return normalized.slice(workspaceRoot.length + 1);
  return normalized;
}

async function openIncludeChild(edge: IncludeGraphItem) {
  await store.openPath(edge.child);
}

async function goToIncludeDirective(edge: IncludeGraphItem) {
  await store.openPath(edge.parent);
  await nextTick();
  const line = findIncludeDirectiveLine(active.value.text, edge.parent, edge.child);
  if (!line) return;
  await goToSourceTarget({
    source_file: edge.parent,
    line,
    column: 1,
    end_line: line,
    end_column: Math.max(2, includeDirectiveLineText(active.value.text, line).length + 1),
  });
}

function findIncludeDirectiveLine(text: string, parentPath: string, childPath: string) {
  const normalizedChild = normalizeDocumentPath(childPath);
  const lines = text.split(/\r?\n/);
  return (
    lines.findIndex((line) => {
      const target = includeDirectiveTarget(line);
      return Boolean(target && normalizeDocumentPath(resolveIncludePath(parentPath, target)) === normalizedChild);
    }) + 1
  );
}

function includeDirectiveLineText(text: string, line: number) {
  return text.split(/\r?\n/)[line - 1] || "";
}

function includeDirectiveTarget(line: string) {
  const trimmed = line.trim();
  const bang = trimmed.match(/^!include\s+(.+)$/);
  if (bang) return unquoteIncludeTarget(bang[1]);
  const braces = trimmed.match(/^\{\{\s*include\s+(.+?)\s*\}\}$/);
  if (braces) return unquoteIncludeTarget(braces[1]);
  const comment = trimmed.match(/^<!--\s*include:\s*(.+?)\s*-->$/);
  if (comment) return unquoteIncludeTarget(comment[1]);
  return "";
}

function unquoteIncludeTarget(target: string) {
  return target.trim().replace(/^["']|["']$/g, "");
}

function resolveIncludePath(parentPath: string, target: string) {
  if (target.startsWith("/")) return normalizeDocumentPath(target);
  const parentFolder = folderFromDocumentPath(parentPath);
  const stack: string[] = [];
  for (const part of `${parentFolder}/${target}`.split("/")) {
    if (!part || part === ".") continue;
    if (part === "..") {
      stack.pop();
    } else {
      stack.push(part);
    }
  }
  return `/${stack.join("/")}`;
}

async function openDocument() {
  const smokePath = await desktopWorkflowSmokeMarkdownPath();
  if (smokePath) {
    await store.openPath(smokePath);
    return;
  }
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
  const supported = new Set<string>(SUPPORTED_CITATION_STYLES);
  if (!supported.has(style)) return;
  store.updateText(upsertFrontMatterField(active.value.text, "citationStyle", style));
}

function insertCitationReference(key: string) {
  const snippet = citationReferenceSnippet(key);
  if (snippet) insertBlock(snippet);
}

function insertMissingCitationStubs() {
  const snippet = bibliographyStubsForMissingKeys(missingCitationKeys.value);
  if (snippet) insertBlock(snippet);
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

function clampUiFontSize(value: number) {
  return Math.min(Math.max(Number(value) || 14, 12), 22);
}

function clampToolbarTextSize(value: number) {
  return Math.min(Math.max(Number(value) || 10, 9), 15);
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

function documentUsesTransformFence(text: string, name: string) {
  const fencePrefix = new RegExp(`^\\s*\`\`\`${escapeRegExp(name)}(?:\\s|$)`, "i");
  return text.split("\n").some((line) => fencePrefix.test(line));
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

async function confirmTransformEngineTrust(name: string) {
  const enginePath = store.transformEnginePaths[name]?.trim();
  if (!enginePath) {
    store.statusMessage = `Choose a ${name} engine path before trusting it`;
    return false;
  }
  return confirm(
    `Trust ${name} external transform engine?\n\nNEditor will be allowed to run ${enginePath} for this transform with timeout, size, and no-shell execution limits.`,
    { title: "Trust external transform engine", kind: "warning" },
  );
}

async function trustTransformEngine(name: string) {
  if (!(await confirmTransformEngineTrust(name))) return;
  await store.setTransformTrust(name, true);
  await store.compileActive();
  store.statusMessage = `${name} external transform trusted`;
}

function reviewTransformEngineSettings(name: string) {
  store.sidebar = "settings";
  store.statusMessage = `Review ${name} external transform settings`;
}

async function toggleTransformTrust(name: string, event: Event) {
  const trusted = eventChecked(event);
  if (!trusted) {
    await store.setTransformTrust(name, false);
    return;
  }
  const allowed = await confirmTransformEngineTrust(name);
  if (!allowed) {
    if (event.target instanceof HTMLInputElement) event.target.checked = false;
    await store.setTransformTrust(name, false);
    return;
  }
  await store.setTransformTrust(name, true);
}

async function saveDocument() {
  if (!active.value.path) {
    await saveDocumentAs();
    return;
  }
  await store.saveActive();
}

async function saveDocumentAs() {
  const path =
    (await desktopWorkflowSmokeMarkdownPath()) ||
    (await save({
      filters: [{ name: "Markdown", extensions: ["md"] }],
      defaultPath: active.value.title.endsWith(".md") ? active.value.title : `${active.value.title}.md`,
    }));
  if (path) await store.saveActive(path);
}

async function desktopWorkflowSmokeMarkdownPath() {
  return invoke<string | null>("desktop_workflow_smoke_file_path", { extension: "md" }).catch(() => null);
}

async function desktopWorkflowSmokeNamedMarkdownPath(fileStem: string) {
  return invoke<string | null>("desktop_workflow_smoke_named_path", { fileStem, extension: "md" }).catch(() => null);
}

async function renameDocument() {
  const path =
    (await desktopWorkflowSmokeNamedMarkdownPath("native-workflow-renamed")) ||
    (await save({
      filters: [{ name: "Markdown", extensions: ["md"] }],
      defaultPath: active.value.title.endsWith(".md") ? active.value.title : `${active.value.title}.md`,
    }));
  if (path) await store.renameActive(path);
}

async function duplicateDocument() {
  const path =
    (await desktopWorkflowSmokeNamedMarkdownPath("native-workflow-duplicate")) ||
    (await save({
      filters: [{ name: "Markdown", extensions: ["md"] }],
      defaultPath: `${active.value.title.replace(/\.[^.]+$/, "")} copy.md`,
    }));
  if (path) await store.duplicateActive(path);
}

async function saveConflictCopy() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: `${conflictDocument.value.title.replace(/\.[^.]+$/, "")} local copy.md`,
  });
  if (path) {
    await store.saveLocalConflictCopy(path);
    closeConflictDialog();
  }
}

function seedConflictMerge(source: ConflictMergeSource) {
  conflictMergeParts.value = [];
  mergedConflictText.value = source === "external" ? store.externalConflict?.externalText || "" : conflictDocument.value.text;
}

function clearConflictMerge() {
  conflictMergeParts.value = [];
  mergedConflictText.value = "";
}

function isConflictMergePartSelected(row: ConflictDiffRow, source: ConflictMergeSource) {
  const line = source === "local" ? row.localLine : row.externalLine;
  if (line === null) return false;
  return conflictMergeParts.value.some((part) => part.id === `${source}:${line}:${row.key}`);
}

function addConflictMergeLine(row: ConflictDiffRow, source: ConflictMergeSource) {
  conflictMergeParts.value = appendConflictMergePart(conflictMergeParts.value, row, source);
  mergedConflictText.value = renderConflictMergeParts(conflictMergeParts.value);
}

function removeConflictLine(id: string) {
  conflictMergeParts.value = removeConflictMergePart(conflictMergeParts.value, id);
  mergedConflictText.value = renderConflictMergeParts(conflictMergeParts.value);
}

function moveConflictLine(id: string, direction: -1 | 1) {
  conflictMergeParts.value = moveConflictMergePart(conflictMergeParts.value, id, direction);
  mergedConflictText.value = renderConflictMergeParts(conflictMergeParts.value);
}

async function applyConflictMerge() {
  await store.applyConflictMerge(mergedConflictText.value);
  conflictMergeParts.value = [];
  closeConflictDialog();
}

function flushEditorTextToStore() {
  if (!editorView) return;
  const text = editorView.state.doc.toString();
  if (active.value.text === text) return;
  previewTextCommit.flush(text);
}

async function exportDocument() {
  if (store.exportBusy) return;
  await prepareForExport();
  if (store.exportReadiness && store.exportReadiness.error_count > 0) {
    store.sidebar = "exports";
    store.statusMessage = `${store.exportReadiness.error_count} errors block export`;
    return;
  }
  const extensions: Record<typeof store.exportTarget, string> = {
    html: "html",
    pdf: "pdf",
    docx: "docx",
    pptx: "pptx",
    "markdown-bundle": "zip",
    blog: "zip",
    substack: "zip",
    latex: "tex",
    "google-docs": "zip",
  };
  const extension = extensions[store.exportTarget];
  const smokeExportPath = await invoke<string | null>("desktop_workflow_smoke_export_path", { extension }).catch(() => null);
  const path =
    smokeExportPath ||
    (await save({
      filters: [{ name: store.exportTarget.toUpperCase(), extensions: [extension] }],
      defaultPath: `${active.value.title.replace(/\.[^.]+$/, "")}.${extension}`,
    }));
  if (path) await store.exportActive(path);
}

async function exportDocumentAs(target: typeof store.exportTarget) {
  store.exportTarget = target;
  store.sidebar = "exports";
  await nextTick();
  await exportDocument();
}

function selectExportProfile(id: string) {
  if (id) {
    void store.applyExportProfile(id);
    return;
  }
  store.activeExportProfileId = "";
  void store.persistWorkspace();
}

function saveExportProfileFromPanel() {
  const profile = store.saveCurrentExportProfile(exportProfileName.value);
  exportProfileName.value = profile.name;
}

function deleteActiveExportProfile() {
  if (!store.activeExportProfileId) return;
  store.deleteExportProfile(store.activeExportProfileId);
}

async function prepareForExport() {
  flushEditorTextToStore();
  await store.prepareForExport();
}

async function snapshotActive() {
  flushEditorTextToStore();
  await store.snapshotActive();
}

async function restoreSnapshot(path: string) {
  flushEditorTextToStore();
  await store.restoreSnapshot(path);
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
      preserveHeadings: aiPreserveHeadings.value,
      convertNumberedLists: aiConvertNumberedLists.value,
      convertTables: aiConvertTables.value,
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
  aiPreserveHeadings.value = store.aiCleanupDefaults.preserveHeadings;
  aiConvertNumberedLists.value = store.aiCleanupDefaults.convertNumberedLists;
  aiConvertTables.value = store.aiCleanupDefaults.convertTables;
}

async function readClipboardText(): Promise<ClipboardTextRead | null> {
  const clipboard = navigator.clipboard as RichClipboard | undefined;
  if (!clipboard) return null;

  if (clipboard.read) {
    try {
      const items = await boundedClipboardRead(clipboard.read());
      if (!items) return null;
      for (const item of items) {
        const preferredType = ["text/html", "text/plain"].find((type) => item.types.includes(type));
        if (!preferredType) continue;
        const text = await (await item.getType(preferredType)).text();
        if (text.trim()) {
          return { text, kind: preferredType === "text/html" ? "rich" : "plain" };
        }
      }
    } catch {
      // WebViews may expose readText but deny rich clipboard reads.
    }
  }

  if (!clipboard.readText) return null;
  try {
    const text = await boundedClipboardRead(clipboard.readText());
    if (!text) return null;
    return text.trim() ? { text, kind: "plain" } : null;
  } catch {
    return null;
  }
}

async function boundedClipboardRead<T>(read: Promise<T>) {
  return Promise.race<T | null>([read, new Promise((resolve) => window.setTimeout(() => resolve(null), 350))]);
}

async function openAiPaste() {
  applyAiPasteDefaults();
  aiPasteOpen.value = true;
  if (desktopWorkflowSmokeActive.value) {
    store.statusMessage = "Paste AI chat text to preview cleanup";
    return;
  }
  if (aiPasteText.value.trim()) return;
  const clipboardText = await readClipboardText();
  if (clipboardText) {
    aiPasteText.value = clipboardText.text;
    store.statusMessage =
      clipboardText.kind === "rich" ? "Loaded rich clipboard text for AI cleanup" : "Loaded clipboard text for AI cleanup";
  } else {
    store.statusMessage = "Paste AI chat text to preview cleanup";
  }
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
    preserveHeadings: aiPreserveHeadings.value,
    convertNumberedLists: aiConvertNumberedLists.value,
    convertTables: aiConvertTables.value,
  });
}

async function runCommand(run: () => unknown) {
  closeCommandPalette();
  await nextTick();
  run();
}

function toolbarIconPaths(icon: ToolbarIconName) {
  return toolbarIconPathMap[icon] || toolbarIconPathMap.commands;
}

async function runCommandBarAction(action: CommandBarAction) {
  if (action.disabled) return;
  try {
    await action.run();
  } catch (error) {
    store.lastError = error instanceof Error ? error.message : String(error);
    store.statusMessage = `${action.label} failed`;
  }
}

function insertReviewComment() {
  store.insertReviewComment(reviewCommentText.value);
  reviewCommentText.value = "";
}

function insertChangeNote() {
  store.insertChangeNote(changeNoteText.value);
  changeNoteText.value = "";
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

function insertFigureSnippet(position: FigureCropPosition = "center") {
  insertBlock(formatFigureSnippet(position));
}

function insertIndexMarkerForTerm(term: string) {
  insertBlock(`#index:${term}`);
}

function formatFigureSnippet(position: FigureCropPosition) {
  return `![Figure alt](assets/figure.png){#fig:figure caption="Figure caption" fit="cover" position="${position}"}`;
}

function onFigureCropPositionChange(figure: FigureListItem, event: Event) {
  const value = (event.target as HTMLSelectElement | null)?.value;
  if (!isFigureCropPosition(value)) return;
  setFigureCropPosition(figure, value);
}

function setFigureCropPosition(figure: FigureListItem, position: FigureCropPosition) {
  if (!editorView || !canEditFigureSource(figure)) return;
  const line = editorView.state.doc.line(Math.max(1, Math.min(figure.line, editorView.state.doc.lines)));
  const withFit = upsertMarkdownAttribute(line.text, "fit", "cover");
  const updated = upsertMarkdownAttribute(withFit, "position", position);
  editorView.dispatch({
    changes: { from: line.from, to: line.to, insert: updated },
    selection: { anchor: line.from, head: line.from + updated.length },
  });
  editorView.focus();
}

function onFigureCropPointerDown(figure: FigureListItem, event: PointerEvent) {
  const element = event.currentTarget as HTMLElement | null;
  if (!element || !canEditFigureSource(figure)) return;
  element.setPointerCapture?.(event.pointerId);
  setFigureCropPosition(figure, figureCropPositionFromPointer(element, event));
}

function onFigureCropPointerMove(figure: FigureListItem, event: PointerEvent) {
  if (event.buttons !== 1) return;
  const element = event.currentTarget as HTMLElement | null;
  if (!element || !canEditFigureSource(figure)) return;
  setFigureCropPosition(figure, figureCropPositionFromPointer(element, event));
}

function onFigureCropKeydown(figure: FigureListItem, event: KeyboardEvent) {
  if (!canEditFigureSource(figure)) return;
  const current = normalizeFigureCropPosition(figure.position);
  const grid = figureCropPositionGrid[current];
  let next = grid;
  if (event.key === "ArrowUp") next = { ...grid, y: clampGridValue(grid.y - 1) };
  else if (event.key === "ArrowDown") next = { ...grid, y: clampGridValue(grid.y + 1) };
  else if (event.key === "ArrowLeft") next = { ...grid, x: clampGridValue(grid.x - 1) };
  else if (event.key === "ArrowRight") next = { ...grid, x: clampGridValue(grid.x + 1) };
  else if (event.key === "Home") next = { x: 0, y: 0 };
  else return;
  event.preventDefault();
  setFigureCropPosition(figure, figureCropPositionFromGrid(next.x, next.y));
}

function figureCropPositionFromPointer(element: HTMLElement, event: PointerEvent): FigureCropPosition {
  const rect = element.getBoundingClientRect();
  const x = rect.width > 0 ? (event.clientX - rect.left) / rect.width : 0.5;
  const y = rect.height > 0 ? (event.clientY - rect.top) / rect.height : 0.5;
  return figureCropPositionFromGrid(pointerGridValue(x), pointerGridValue(y));
}

function pointerGridValue(value: number): -1 | 0 | 1 {
  if (value < 1 / 3) return -1;
  if (value > 2 / 3) return 1;
  return 0;
}

function clampGridValue(value: number): -1 | 0 | 1 {
  if (value < 0) return -1;
  if (value > 0) return 1;
  return 0;
}

function figureCropPositionFromGrid(x: -1 | 0 | 1, y: -1 | 0 | 1): FigureCropPosition {
  const match = figureCropPositions.find((position) => {
    const point = figureCropPositionGrid[position];
    return point.x === x && point.y === y;
  });
  return match || "center";
}

function figureCropPreviewStyle(figure: FigureListItem): CSSProperties {
  const position = normalizeFigureCropPosition(figure.position);
  const point = figureCropPositionPoints[position];
  const style: CSSProperties = {
    backgroundPosition: `${point.x}% ${point.y}%`,
  };
  if (figure.src) {
    style.backgroundImage = `linear-gradient(rgba(15, 23, 42, 0.18), rgba(15, 23, 42, 0.18)), url("${escapeCssUrl(figure.src)}")`;
  }
  return style;
}

function figureCropPointStyle(position: FigureCropPosition): CSSProperties {
  const point = figureCropPositionPoints[position];
  return { left: `${point.x}%`, top: `${point.y}%` };
}

function figureCropReticleStyle(position: FigureCropPosition): CSSProperties {
  const point = figureCropPositionPoints[position];
  return { left: `${point.x}%`, top: `${point.y}%` };
}

function normalizeFigureCropPosition(value: string | null | undefined): FigureCropPosition {
  if (isFigureCropPosition(value || undefined)) return value as FigureCropPosition;
  return "center";
}

function escapeCssUrl(value: string) {
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function canEditFigureSource(figure: FigureListItem) {
  return !figure.source_file || !active.value.path || figure.source_file === active.value.path;
}

function upsertMarkdownAttribute(line: string, key: string, value: string) {
  const attribute = `${key}="${value}"`;
  const match = line.match(/\{([^{}]*)\}\s*$/);
  if (!match || match.index === undefined) return `${line}{${attribute}}`;
  const attrs = match[1];
  const pattern = new RegExp(`(^|\\s)${key}="[^"]*"`);
  const updatedAttrs = pattern.test(attrs)
    ? attrs.replace(pattern, (_token, prefix: string) => `${prefix}${attribute}`)
    : `${attrs.trim()} ${attribute}`.trim();
  return `${line.slice(0, match.index)}{${updatedAttrs}}`;
}

function isFigureCropPosition(value: string | undefined): value is FigureCropPosition {
  return Boolean(value && figureCropPositions.includes(value as FigureCropPosition));
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
  const issues = validateTableDraft(draft);
  if (issues.some((issue) => issue.severity === "error")) {
    store.statusMessage = "Fix table validation errors before applying";
    return;
  }
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

function cancelTableDraft() {
  if (isNewTableDraft.value) {
    tableDraft.value = null;
    isNewTableDraft.value = false;
    if (selectedTable.value) loadSelectedTable();
    store.statusMessage = "Cancelled new table";
    return;
  }
  loadSelectedTable();
  store.statusMessage = "Discarded table draft changes";
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

function duplicateTableRow(rowIndex: number) {
  const draft = tableDraft.value;
  if (!draft) return;
  const source = draft.rows[rowIndex] || draft.headers.map(() => "");
  draft.rows.splice(rowIndex + 1, 0, padTableRow([...source], draft.headers.length));
}

function moveTableRow(rowIndex: number, direction: -1 | 1) {
  const draft = tableDraft.value;
  if (!draft) return;
  moveArrayItem(draft.rows, rowIndex, rowIndex + direction);
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

function duplicateTableColumn(columnIndex: number) {
  const draft = tableDraft.value;
  if (!draft) return;
  const header = draft.headers[columnIndex] || `Column ${columnIndex + 1}`;
  draft.headers.splice(columnIndex + 1, 0, `${header} copy`);
  draft.alignments.splice(columnIndex + 1, 0, draft.alignments[columnIndex] || "left");
  draft.formats.splice(columnIndex + 1, 0, draft.formats[columnIndex] || "text");
  for (const row of draft.rows) {
    row.splice(columnIndex + 1, 0, row[columnIndex] || "");
  }
}

function moveTableColumn(columnIndex: number, direction: -1 | 1) {
  const draft = tableDraft.value;
  if (!draft) return;
  const targetIndex = columnIndex + direction;
  moveArrayItem(draft.headers, columnIndex, targetIndex);
  moveArrayItem(draft.alignments, columnIndex, targetIndex);
  moveArrayItem(draft.formats, columnIndex, targetIndex);
  for (const row of draft.rows) moveArrayItem(row, columnIndex, targetIndex);
}

function moveArrayItem<T>(items: T[], from: number, to: number) {
  if (from === to || from < 0 || to < 0 || from >= items.length || to >= items.length) return;
  const [item] = items.splice(from, 1);
  items.splice(to, 0, item);
}

function addTableTotalsRow() {
  addTableFormulaRow("SUM", "Total");
}

function addTableFormulaRow(formula: TableFormulaFunction, label: string = formula) {
  const draft = tableDraft.value;
  if (!draft) return;
  const dataRowCount = draft.rows.filter((row) => !isTableSummaryRow(row)).length;
  const totals = draft.headers.map((_, columnIndex) => {
    if (columnIndex === 0) return label;
    if (!dataRowCount) return "";
    return `=${formula}(${tableColumnRange(columnIndex, dataRowCount)})`;
  });
  draft.rows.push(totals);
}

function appendCustomTableFormulaRow() {
  const draft = tableDraft.value;
  const row = buildCustomTableFormulaRow();
  if (!draft || !row) return;
  draft.rows.push(row);
}

function applyTableCellSpan() {
  const draft = tableDraft.value;
  if (!draft) return;
  const rowIndex = clampInteger(tableSpanRow.value, 0, Math.max(0, draft.rows.length - 1));
  const columnIndex = clampInteger(tableSpanColumn.value, 0, Math.max(0, draft.headers.length - 1));
  const row = draft.rows[rowIndex];
  if (!row) return;
  const colspan = clampInteger(tableSpanColspan.value, 1, Math.max(1, draft.headers.length - columnIndex));
  const rowspan = clampInteger(tableSpanRowspan.value, 1, Math.max(1, draft.rows.length - rowIndex));
  row[columnIndex] = setTableCellSpan(row[columnIndex] || "", colspan, rowspan);
}

function clearTableCellSpan() {
  const draft = tableDraft.value;
  if (!draft) return;
  const row = draft.rows[tableSpanRow.value];
  if (!row || row[tableSpanColumn.value] === undefined) return;
  const span = parseTableCellSpan(row[tableSpanColumn.value]);
  row[tableSpanColumn.value] = span.text;
  tableSpanColspan.value = 1;
  tableSpanRowspan.value = 1;
}

function syncTableSpanControlsFromCell() {
  const draft = tableDraft.value;
  const row = draft?.rows[tableSpanRow.value];
  const value = row?.[tableSpanColumn.value];
  if (value === undefined) {
    tableSpanColspan.value = 1;
    tableSpanRowspan.value = 1;
    return;
  }
  const span = parseTableCellSpan(value);
  tableSpanColspan.value = span.colspan;
  tableSpanRowspan.value = span.rowspan;
}

function buildCustomTableFormulaRow() {
  const draft = tableDraft.value;
  if (!draft || !draft.headers.length) return null;
  const targetColumn = resolvedFormulaTargetColumn(draft);
  const [startRow, endRow] = resolvedFormulaRows();
  const column = spreadsheetColumnName(targetColumn + 1);
  const row = draft.headers.map(() => "");
  const label = tableFormulaLabel.value.trim() || tableFormulaFunction.value;
  if (targetColumn > 0) row[0] = label;
  row[targetColumn] = `=${tableFormulaFunction.value}(${column}${startRow}:${column}${endRow})`;
  return row;
}

function resolvedFormulaTargetColumn(draft: TableDraft) {
  const preferred = Number(tableFormulaTargetColumn.value);
  const firstFormulaColumn = draft.headers.length > 1 ? 1 : 0;
  if (!Number.isInteger(preferred)) return firstFormulaColumn;
  return Math.min(Math.max(preferred, firstFormulaColumn), draft.headers.length - 1);
}

function resolvedFormulaRows() {
  const maxRow = tableDataRowCount.value;
  const start = clampInteger(tableFormulaStartRow.value, 1, maxRow);
  const end = clampInteger(tableFormulaEndRow.value, 1, maxRow);
  return start <= end ? [start, end] : [end, start];
}

function clampInteger(value: number, min: number, max: number) {
  if (!Number.isFinite(value)) return min;
  return Math.min(Math.max(Math.trunc(value), min), max);
}

function replaceTableFromPaste() {
  const parsed = parseTablePaste(tablePasteText.value);
  const rows = parsed.rows;
  if (!rows.length) return;
  const current = tableDraft.value;
  const headers = rows[0].map((cell, index) => cell.trim() || `Column ${index + 1}`);
  const bodyRows = rows.slice(1).map((row) => padTableRow(row, headers.length));
  tableDraft.value = {
    id: parsed.id ?? current?.id ?? "",
    caption: parsed.caption ?? current?.caption ?? "",
    headers,
    alignments: parsed.alignments
      ? padAlignments(parsed.alignments, headers.length)
      : headers.map(() => "left"),
    formats: headers.map((_, columnIndex) => inferTableFormat(bodyRows.map((row) => row[columnIndex] || ""))),
    rows: bodyRows.length ? bodyRows : [headers.map(() => "")],
  };
}

function sortTableRows(columnIndex: number, direction: TableSortDirection) {
  const draft = tableDraft.value;
  if (!draft) return;
  tableDraft.value = sortTableDraftRows(draft, columnIndex, direction);
}

function tableHeaderLabel(columnIndex: number) {
  return `Column ${spreadsheetColumnName(columnIndex + 1)} header`;
}

function tableCellLabel(rowIndex: number, columnIndex: number) {
  const draft = tableDraft.value;
  const header = draft?.headers[columnIndex]?.trim();
  const column = spreadsheetColumnName(columnIndex + 1);
  return header ? `${header}, row ${rowIndex + 1}, column ${column}` : `Row ${rowIndex + 1}, column ${column}`;
}

function tableTotalLabel(columnIndex: number) {
  const draft = tableDraft.value;
  const header = draft?.headers[columnIndex]?.trim();
  const column = spreadsheetColumnName(columnIndex + 1);
  return header ? `Total for ${header}, column ${column}` : `Total for column ${column}`;
}

async function goToSourceTarget(target: {
  line?: number | null;
  column?: number | null;
  end_line?: number | null;
  end_column?: number | null;
  source_file?: string | null;
}) {
  if (target.source_file && active.value.path && target.source_file !== active.value.path) {
    await store.openPath(target.source_file);
    await nextTick();
  }
  if (["preview", "export", "presentation"].includes(store.mode)) {
    store.mode = "split";
    await nextTick();
  }
  if (!editorView || !target.line) return;
  const startLine = editorView.state.doc.line(Math.max(1, Math.min(target.line, editorView.state.doc.lines)));
  const endLine = editorView.state.doc.line(Math.max(1, Math.min(target.end_line || target.line, editorView.state.doc.lines)));
  const from = startLine.from + clampColumnOffset(target.column, startLine.length);
  const to = endLine.from + clampColumnOffset(target.end_column, endLine.length);
  editorView.dispatch({
    selection: { anchor: from, head: Math.max(from + 1, to) },
    effects: EditorView.scrollIntoView(from, { y: "center" }),
  });
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

function goToCrossReference(reference: { line: number; column?: number | null; end_column?: number | null; source_file?: string | null }) {
  void goToSourceTarget(reference);
}

function goToTransformArtifact(artifact: TransformPreviewItem) {
  if (!artifact.sourceLine) return;
  void goToSourceTarget({
    source_file: artifact.sourceFile || null,
    line: artifact.sourceLine,
    end_line: artifact.endSourceLine || artifact.sourceLine,
  });
}

function handlePreviewClick(event: MouseEvent) {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const diagnosticJump = target.closest<HTMLButtonElement>("button.preview-diagnostic-jump");
  if (diagnosticJump) {
    event.preventDefault();
    const line = Number(diagnosticJump.dataset.line || 0);
    if (line) {
      void goToSourceTarget({
        source_file: diagnosticJump.dataset.sourceFile || null,
        line,
        column: Number(diagnosticJump.dataset.column || 0) || null,
        end_line: Number(diagnosticJump.dataset.endLine || 0) || line,
        end_column: Number(diagnosticJump.dataset.endColumn || 0) || null,
      });
    }
    return;
  }
  const link = target.closest("a[href^='#']");
  const heading = target.closest("h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]");
  const anchor = heading?.id || link?.getAttribute("href")?.slice(1) || "";
  if (!anchor) return;
  const sourceTarget = sourceTargetForAnchor(anchor);
  if (!sourceTarget?.line) return;
  event.preventDefault();
  void goToSourceTarget(sourceTarget);
}

function sourceTargetForAnchor(anchor: string) {
  const compile = active.value.compile;
  if (!compile) return null;
  const headingBlock = compile.document_ast.blocks.find((block) => block.kind === "heading" && block.anchor === anchor);
  if (headingBlock?.kind === "heading") {
    return {
      line: headingBlock.source?.source_line || headingBlock.line,
      end_line: headingBlock.source?.end_source_line || headingBlock.end_line,
      source_file: headingBlock.source?.source_file || null,
    };
  }
  for (const block of compile.document_ast.blocks) {
    if (
      (block.kind === "table" || block.kind === "figure" || block.kind === "equation") &&
      block.id === anchor
    ) {
      return {
        line: block.source?.source_line || block.line,
        end_line: block.source?.end_source_line || block.end_line,
        source_file: block.source?.source_file || null,
      };
    }
  }
  const headingEntry = compile.semantic.outline.find((item) => item.anchor === anchor);
  if (headingEntry) return { line: headingEntry.line };
  return null;
}

function handleShortcut(event: KeyboardEvent) {
  if (!(event.metaKey || event.ctrlKey)) return;
  if (event.key === "s") {
    event.preventDefault();
    if (event.shiftKey) {
      void saveDocumentAs();
    } else {
      void saveDocument();
    }
  } else if (event.key === "o") {
    event.preventDefault();
    void openDocument();
  } else if (event.key === "n") {
    event.preventDefault();
    store.newDocument();
  } else if (event.key.toLowerCase() === "e") {
    event.preventDefault();
    void exportDocument();
  } else if (event.key === "b") {
    event.preventDefault();
    wrapSelection("**");
  } else if (event.key === "i") {
    event.preventDefault();
    wrapSelection("*");
  } else if (event.key.toLowerCase() === "k" || (event.key.toLowerCase() === "p" && event.shiftKey)) {
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
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 4px 10px;
  cursor: pointer;
}

button svg {
  width: 16px;
  height: 16px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.9;
}

button:hover,
select:hover {
  border-color: #6386b4;
}

.app-shell {
  display: grid;
  grid-template-rows: 38px auto minmax(0, 1fr) 28px;
  width: 100vw;
  height: 100vh;
  color: #18212f;
  background: #edf1f5;
}

.app-shell.has-trust-prompt {
  grid-template-rows: 38px auto auto minmax(0, 1fr) 28px;
}

.app-shell[data-theme="dark"] {
  color: #e6edf5;
  background: #111821;
}

.app-shell[data-theme="dark"] .titlebar,
.app-shell[data-theme="dark"] .command-bar,
.app-shell[data-theme="dark"] .status-bar {
  border-color: #29384a;
  background: #172231;
}

.app-shell[data-theme="dark"] button,
.app-shell[data-theme="dark"] select,
.app-shell[data-theme="dark"] input,
.app-shell[data-theme="dark"] textarea {
  border-color: #405267;
  background: #202c3b;
  color: #e6edf5;
}

.app-shell[data-theme="dark"] .icon-command.primary {
  border-color: #587ea9;
  background: #203b58;
}

.app-shell[data-theme="dark"] .tab,
.app-shell[data-theme="dark"] .tab-group-header,
.app-shell[data-theme="dark"] .command-group,
.app-shell[data-theme="dark"] .template-card,
.app-shell[data-theme="dark"] .template-source,
.app-shell[data-theme="dark"] .template-meta span,
.app-shell[data-theme="dark"] .status-message,
.app-shell[data-theme="dark"] .word-stats,
.app-shell[data-theme="dark"] .watch-status,
.app-shell[data-theme="dark"] .export-progress {
  border-color: #34465a;
  background: #1b2736;
  color: #dce7f3;
}

.app-shell[data-theme="dark"] .template-card-header small,
.app-shell[data-theme="dark"] .template-fill-fields,
.app-shell[data-theme="dark"] .sidebar-hint {
  color: #aebdcc;
}

@media (prefers-color-scheme: dark) {
  .app-shell[data-theme="system"] {
    color: #e6edf5;
    background: #111821;
  }

  .app-shell[data-theme="system"] .titlebar,
  .app-shell[data-theme="system"] .command-bar,
  .app-shell[data-theme="system"] .status-bar {
    border-color: #29384a;
    background: #172231;
  }

  .app-shell[data-theme="system"] button,
  .app-shell[data-theme="system"] select,
  .app-shell[data-theme="system"] input,
  .app-shell[data-theme="system"] textarea {
    border-color: #405267;
    background: #202c3b;
    color: #e6edf5;
  }

  .app-shell[data-theme="system"] .icon-command.primary {
    border-color: #587ea9;
    background: #203b58;
  }

  .app-shell[data-theme="system"] .tab,
  .app-shell[data-theme="system"] .tab-group-header,
  .app-shell[data-theme="system"] .command-group,
  .app-shell[data-theme="system"] .template-card,
  .app-shell[data-theme="system"] .template-source,
  .app-shell[data-theme="system"] .template-meta span,
  .app-shell[data-theme="system"] .status-message,
  .app-shell[data-theme="system"] .word-stats,
  .app-shell[data-theme="system"] .watch-status,
  .app-shell[data-theme="system"] .export-progress {
    border-color: #34465a;
    background: #1b2736;
    color: #dce7f3;
  }

  .app-shell[data-theme="system"] .template-card-header small,
  .app-shell[data-theme="system"] .template-fill-fields,
  .app-shell[data-theme="system"] .sidebar-hint {
    color: #aebdcc;
  }
}

.app-shell[data-high-contrast="true"] {
  color: #000000;
  background: #ffffff;
}

.app-shell[data-high-contrast="true"] .titlebar,
.app-shell[data-high-contrast="true"] .command-bar,
.app-shell[data-high-contrast="true"] .trust-prompt,
.app-shell[data-high-contrast="true"] .status-bar,
.app-shell[data-high-contrast="true"] .sidebar,
.app-shell[data-high-contrast="true"] .release-badge,
.app-shell[data-high-contrast="true"] .tab,
.app-shell[data-high-contrast="true"] .tab-group-header,
.app-shell[data-high-contrast="true"] .command-group,
.app-shell[data-high-contrast="true"] .template-card,
.app-shell[data-high-contrast="true"] .template-source,
.app-shell[data-high-contrast="true"] .template-meta span,
.app-shell[data-high-contrast="true"] .status-message,
.app-shell[data-high-contrast="true"] .word-stats,
.app-shell[data-high-contrast="true"] .watch-status,
.app-shell[data-high-contrast="true"] .export-progress,
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

.app-shell[data-high-contrast="true"] :focus-visible,
.app-shell[data-high-contrast="true"] .skip-links a:focus {
  outline: 3px solid #000000;
  outline-offset: 2px;
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

.skip-links {
  position: fixed;
  top: 8px;
  left: 8px;
  z-index: 1000;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  pointer-events: none;
}

.skip-links a {
  position: absolute;
  top: -999px;
  left: 0;
  border: 2px solid #18212f;
  border-radius: 6px;
  padding: 6px 10px;
  background: #ffffff;
  color: #18212f;
  font-weight: 700;
  text-decoration: none;
  pointer-events: auto;
}

.skip-links a:focus {
  position: static;
  transform: none;
  outline: 3px solid #f6c343;
  outline-offset: 2px;
}

.titlebar,
.command-bar,
.status-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
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
  align-items: center;
  flex: 0 0 auto;
  gap: 6px;
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
  align-items: center;
  gap: 5px;
  min-width: 76px;
  max-width: 140px;
  min-height: 28px;
  padding: 2px 4px 2px 6px;
  border: 1px solid #d7dee7;
  border-radius: 6px;
  background: #ffffff;
  color: #526171;
  font-size: 11px;
  line-height: 1.2;
  text-transform: uppercase;
}

.tab-group-title {
  display: grid;
  min-width: 0;
  flex: 1;
  gap: 1px;
}

.tab-group-title span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-group-title small {
  color: #7b8794;
  font-size: 10px;
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  max-width: 220px;
  min-height: 30px;
  padding: 0 3px 0 0;
  border: 1px solid #bac4d1;
  border-radius: 6px;
  background: #f3f6fa;
}

.tab[draggable="true"] {
  cursor: grab;
}

.tab.active {
  border-color: #275da8;
  background: #ffffff;
  box-shadow: inset 0 -2px 0 #275da8;
}

.tab-main {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  flex: 1;
  min-height: 28px;
  padding: 4px 8px;
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

.tab-dirty {
  width: 6px;
  height: 6px;
  flex: 0 0 6px;
  border-radius: 50%;
  background: #c68a1a;
}

.tab-icon-button {
  width: 24px;
  min-width: 24px;
  height: 24px;
  min-height: 24px;
  padding: 0;
  border-color: transparent;
  background: transparent;
  color: #607083;
}

.tab-icon-button:hover,
.tab-icon-button.active {
  border-color: #c5cfdb;
  background: #eef4fb;
  color: #174a88;
}

.window-meta,
.status-bar {
  color: #526171;
  font-size: 12px;
}

.window-meta {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
}

.release-badge {
  display: inline-flex;
  align-items: center;
  min-height: 22px;
  padding: 0 8px;
  border: 1px solid #9fb0c2;
  border-radius: 6px;
  background: #ffffff;
  color: #253142;
  font-weight: 700;
  line-height: 1;
  text-transform: uppercase;
}

.release-draft {
  border-color: #c68a1a;
  background: #fff7df;
  color: #714b00;
}

.release-in-review {
  border-color: #4575b4;
  background: #e8f1ff;
  color: #164071;
}

.release-approved {
  border-color: #2f855a;
  background: #e7f7ed;
  color: #19543a;
}

.release-published {
  border-color: #5d55a5;
  background: #f0edff;
  color: #3b3474;
}

.release-archived {
  border-color: #7b8794;
  background: #eef1f4;
  color: #3d4852;
}

.command-bar {
  display: grid;
  align-items: stretch;
  gap: 4px;
  min-height: 0;
  overflow: visible;
  padding: 6px 8px;
  border-bottom-color: #b9c6d4;
  background: #f7f9fc;
}

.command-toolbar-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  padding: 2px 0;
}

.command-toolbar-title {
  width: 92px;
  flex: 0 0 92px;
  color: #5b6c80;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0;
  line-height: 1;
  text-transform: uppercase;
}

.command-toolbar-row-view {
  align-items: center;
}

.command-group {
  display: inline-flex;
  align-items: center;
  flex: 0 0 auto;
  gap: 5px;
  min-height: 34px;
  padding: 3px 5px;
  border: 1px solid #d9e1ea;
  border-radius: 6px;
  background: #ffffff;
}

.command-group:first-child {
  padding-left: 7px;
}

.command-group:last-child {
  padding-right: 7px;
}

.command-group-label {
  min-width: auto;
  color: #607083;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0;
  line-height: 1.1;
  text-transform: uppercase;
}

.command-group-actions {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.icon-command {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-width: 34px;
  height: 28px;
  padding: 0 6px;
  border-color: #c5d0dc;
  background: #f8fafc;
  color: #203044;
  font-size: var(--toolbar-font-size, 10px);
  line-height: 1.1;
  white-space: nowrap;
}

.icon-command.primary {
  border-color: #7fa2cd;
  background: #eaf3ff;
  color: #143f70;
}

.icon-command:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.command-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  flex: 0 0 16px;
}

.command-icon svg {
  width: 16px;
  height: 16px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.9;
}

.command-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.app-shell[data-toolbar-display="icons"] .icon-command {
  width: 36px;
  padding: 0;
}

.app-shell[data-toolbar-display="icons"] .command-label,
.app-shell[data-toolbar-display="text"] .command-icon {
  display: none;
}

.app-shell[data-toolbar-display="text"] .icon-command {
  min-width: auto;
}

.command-group-view {
  grid-template-columns: auto repeat(3, minmax(104px, max-content));
}

.compact-field {
  display: inline-grid;
  flex: 0 0 auto;
  grid-template-columns: auto minmax(84px, max-content);
  align-items: center;
  gap: 5px;
  color: #526171;
  font-size: var(--toolbar-font-size, 10px);
  font-weight: 700;
}

.compact-field span {
  text-transform: uppercase;
}

.compact-field select {
  height: 28px;
  min-width: 96px;
  font-size: var(--toolbar-font-size, 10px);
}

.compact-field input[type="range"] {
  width: 92px;
}

.compact-field output {
  min-width: 32px;
  color: #526171;
  font-variant-numeric: tabular-nums;
}

.compact-field-range {
  grid-template-columns: auto 92px 32px;
}

.trust-prompt {
  display: flex;
  align-items: stretch;
  gap: 8px;
  overflow-x: auto;
  padding: 8px 10px;
  border-bottom: 1px solid #d8b66d;
  background: #fff8e8;
}

.trust-prompt-item {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) auto;
  align-items: center;
  min-width: min(100%, 560px);
  gap: 12px;
  padding: 8px;
  border: 1px solid #d8b66d;
  border-radius: 6px;
  background: #ffffff;
}

.trust-prompt-item strong,
.trust-prompt-item span,
.trust-prompt-item small {
  display: block;
}

.trust-prompt-item span {
  overflow-wrap: anywhere;
  color: #374151;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
  font-size: 12px;
}

.trust-prompt-item small {
  color: #526171;
}

.trust-prompt-actions {
  display: inline-flex;
  gap: 6px;
}

.divider {
  width: 1px;
  height: 22px;
  background: #c9d2dc;
}

.workspace {
  display: grid;
  grid-template-columns:
    260px minmax(260px, calc((100vw - 268px) * var(--editor-ratio, 0.5))) 8px
    minmax(260px, 1fr);
  min-height: 0;
}

.workspace.mode-source,
.workspace.mode-focus {
  grid-template-columns: 260px minmax(0, 1fr);
}

.workspace.mode-preview,
.workspace.mode-export,
.workspace.mode-presentation {
  grid-template-columns: 260px minmax(0, 1fr);
}

.sidebar,
.editor-pane,
.preview-pane {
  min-height: 0;
  overflow: auto;
  border-right: 1px solid #c9d2dc;
}

.pane-splitter {
  width: 8px;
  min-width: 8px;
  padding: 0;
  border: 0;
  border-right: 1px solid #c9d2dc;
  border-left: 1px solid #d7dee7;
  background: #e4eaf1;
  cursor: col-resize;
}

.pane-splitter:hover,
.pane-splitter:focus-visible {
  background: #2f6f9f;
  outline: none;
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
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: baseline;
  gap: 8px;
  width: 100%;
  margin-bottom: 2px;
  border: 0;
  background: transparent;
  text-align: left;
}

.outline-row span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.outline-row small {
  color: #526171;
  font-size: 11px;
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

.sidebar-hint {
  margin: 6px 0 12px;
  color: #526171;
  font-size: 12px;
}

.include-graph {
  display: grid;
  gap: 8px;
}

.include-edge {
  display: grid;
  gap: 6px;
  padding: 8px;
  border-left: 3px solid #2f6f7e;
  background: #ffffff;
}

.include-edge p {
  margin: 0;
  min-width: 0;
  overflow-wrap: anywhere;
}

.include-edge small {
  color: #526171;
}

.include-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.reference-manager {
  display: grid;
  gap: 8px;
  margin: 6px 0 12px;
}

.reference-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.workspace-root {
  margin: 8px 0;
  color: #526171;
  font-size: 12px;
}

.restore-warning {
  margin: 10px 0;
  padding: 8px;
  border: 1px solid #c88a1d;
  background: #fff7e6;
  color: #5b3a04;
}

.restore-warning p {
  margin: 4px 0;
}

.restore-warning ul {
  margin: 6px 0 0;
  padding-left: 16px;
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

.diagnostic-related {
  margin: 6px 0 0;
  padding-left: 18px;
  color: #526171;
  font-size: 12px;
}

.preview-diagnostic {
  display: grid;
  gap: 4px;
  margin: 10px 0;
  padding: 8px 10px;
  border-left: 4px solid #6386b4;
  background: #f7f9fc;
}

.preview-diagnostic.warning {
  border-color: #c68a1a;
  background: #fff7e6;
}

.preview-diagnostic.error {
  border-color: #c24141;
  background: #fff1f1;
}

.preview-diagnostic p,
.preview-diagnostic ul {
  margin: 0;
}

.preview-diagnostic ul {
  padding-left: 18px;
}

.preview-diagnostic small {
  color: #526171;
}

.preview-diagnostic button {
  justify-self: start;
}

.export-preview-summary,
.transform-preview-summary {
  display: grid;
  gap: 8px;
  margin: 0 0 16px;
  padding: 10px;
  border: 1px solid #b9c6d4;
  border-left: 4px solid #2f6f7e;
  background: #f8fbfc;
}

.export-preview-summary div {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: baseline;
}

.export-preview-summary p,
.transform-preview-summary p {
  margin: 0;
}

.export-preview-summary ul {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.export-preview-summary li {
  padding: 2px 6px;
  border: 1px solid #c9d2dc;
  background: #ffffff;
  font-size: 12px;
}

.transform-preview-summary h2 {
  margin: 0;
  font-size: 15px;
}

.transform-preview-summary article {
  display: grid;
  gap: 4px;
  padding: 8px;
  border: 1px solid #d6dde6;
  background: #ffffff;
}

.transform-preview-summary button {
  justify-self: start;
}

.progress-steps {
  display: grid;
  gap: 6px;
  margin: 8px 0 0;
  padding-left: 18px;
}

.progress-steps li {
  display: grid;
  gap: 2px;
}

.progress-steps span,
.progress-steps small {
  color: #526171;
  font-size: 12px;
}

.export-diagnostic-report {
  max-height: 280px;
  overflow: auto;
  padding-right: 4px;
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

.template-filters,
.custom-template-editor {
  display: grid;
  gap: 8px;
  margin-bottom: 12px;
}

.template-filters {
  grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
}

.template-list {
  display: grid;
  gap: 10px;
}

.template-card {
  display: grid;
  gap: 8px;
  padding: 10px;
  border: 1px solid #c9d2dc;
  border-left: 3px solid #2f6f7e;
  border-radius: 7px;
  background: #ffffff;
}

.template-card-header {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: start;
  gap: 8px;
}

.template-card-header div {
  display: grid;
  min-width: 0;
  gap: 2px;
}

.template-card-header strong,
.template-card-header small {
  min-width: 0;
  overflow-wrap: anywhere;
}

.template-card-header small {
  color: #526171;
  line-height: 1.35;
}

.template-source {
  align-self: start;
  padding: 2px 6px;
  border: 1px solid #c7d5e5;
  border-radius: 999px;
  background: #f2f7fc;
  color: #31516f;
  font-size: 10px;
  font-weight: 700;
  line-height: 1.3;
  text-transform: uppercase;
}

.template-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.template-meta-summary {
  flex: 1 0 100%;
  color: #526171;
  font-size: 11px;
}

.template-meta span {
  padding: 2px 7px;
  border: 1px solid #d8e0e8;
  border-radius: 999px;
  background: #f8fafc;
  color: #44566a;
  font-size: 11px;
  font-weight: 650;
}

.template-card pre {
  max-height: 220px;
  overflow: auto;
  margin: 6px 0 0;
  white-space: pre-wrap;
}

.template-tags,
.template-fill-fields,
.template-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.template-fill-fields {
  align-items: center;
  color: #4a5b6d;
}

.template-fill-fields span {
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
}

.template-fill-fields code {
  padding: 2px 6px;
  border: 1px solid #bfcedc;
  background: #f2f6fa;
  color: #183247;
  font-family: inherit;
  font-size: 0.78rem;
}

.template-tags small {
  padding: 2px 6px;
  border: 1px solid #d8e0e8;
  background: #f8fafc;
}

.template-actions button {
  min-height: 28px;
  padding: 3px 8px;
}

.export-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin: 8px 0 10px;
}

.export-profile-manager {
  margin-bottom: 12px;
  padding-bottom: 10px;
  border-bottom: 1px solid #d8e0e8;
}

.export-profile-manager h3 {
  margin: 0 0 8px;
}

.export-target-options {
  display: grid;
  gap: 8px;
  margin: 4px 0 12px;
  padding: 10px;
  border: 1px solid #d8e0e8;
  border-left: 3px solid #2f6f7e;
  background: #ffffff;
}

.export-target-options h3 {
  margin: 0;
}

.export-target-options label {
  margin-bottom: 0;
}

.template-action-primary {
  border-color: #7fa2cd;
  background: #eaf3ff;
  color: #143f70;
  font-weight: 700;
}

.danger-action {
  border-color: #e3b7b7;
  background: #fff5f5;
  color: #9b1c1c;
}

.button-icon {
  display: inline-flex;
  width: 16px;
  height: 16px;
  flex: 0 0 16px;
}

.button-icon svg {
  width: 16px;
  height: 16px;
}

.custom-template-editor {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid #d8e0e8;
}

.crop-focus-pad {
  position: relative;
  width: 100%;
  min-height: 128px;
  overflow: hidden;
  border: 1px solid #9fb0c2;
  background:
    linear-gradient(rgba(255, 255, 255, 0.72), rgba(255, 255, 255, 0.72)),
    repeating-linear-gradient(45deg, #e6ecf3 0 8px, #f7f9fb 8px 16px);
  background-repeat: no-repeat;
  background-size: cover;
  cursor: crosshair;
  touch-action: none;
}

.crop-focus-pad::before {
  position: absolute;
  inset: 0;
  background:
    linear-gradient(to right, transparent 33%, rgba(15, 23, 42, 0.32) 33% 34%, transparent 34% 66%, rgba(15, 23, 42, 0.32) 66% 67%, transparent 67%),
    linear-gradient(to bottom, transparent 33%, rgba(15, 23, 42, 0.32) 33% 34%, transparent 34% 66%, rgba(15, 23, 42, 0.32) 66% 67%, transparent 67%);
  content: "";
}

.crop-focus-pad:focus-visible {
  outline: 2px solid #2f6f9f;
  outline-offset: 2px;
}

.crop-focus-pad.disabled {
  cursor: not-allowed;
  opacity: 0.58;
}

.crop-focus-point,
.crop-focus-reticle {
  position: absolute;
  z-index: 1;
  transform: translate(-50%, -50%);
  pointer-events: none;
}

.crop-focus-point {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: rgba(15, 23, 42, 0.55);
}

.crop-focus-reticle {
  width: 20px;
  height: 20px;
  border: 2px solid #ffffff;
  border-radius: 50%;
  background: #2f6f9f;
  box-shadow: 0 0 0 2px rgba(15, 23, 42, 0.72);
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

.engine-probe {
  border-left: 3px solid #64748b;
  padding: 6px 8px;
  background: #f8fafc;
}

.engine-probe.ok {
  border-left-color: #2f855a;
}

.engine-probe.failed {
  border-left-color: #c2410c;
}

.engine-probe p,
.engine-probe ul {
  margin: 4px 0 0;
}

.path-picker {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 6px;
}

.path-picker input {
  min-width: 0;
}

.formula-cell {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  color: #7c2d12;
  background: #fff7ed;
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

.table-formula-builder,
.table-span-builder {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(104px, 1fr));
  gap: 8px;
  align-items: end;
  margin-bottom: 12px;
  padding: 8px;
  border: 1px solid #d8e0e8;
  background: #f8fafc;
}

.table-formula-builder output,
.table-span-builder output {
  min-height: 32px;
  padding: 7px 9px;
  border: 1px solid #d8e0e8;
  background: #fff;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  color: #7c2d12;
}

.table-formula-builder button,
.table-formula-builder input,
.table-formula-builder select,
.table-span-builder button,
.table-span-builder input,
.table-span-builder select {
  width: 100%;
  min-width: 0;
}

.table-metadata {
  display: grid;
  grid-template-columns: minmax(150px, 220px) minmax(220px, 1fr);
  gap: 8px;
}

.table-issues {
  display: grid;
  gap: 6px;
  margin: 10px 0 12px;
  padding: 8px 10px;
  border: 1px solid #d8e0e8;
  background: #f8fafc;
}

.table-issues p {
  margin: 0;
  font-size: 12px;
}

.table-issues .error {
  color: #b91c1c;
}

.table-issues .warning {
  color: #92400e;
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

.table-editor-grid .row-actions,
.table-editor-grid .column-actions {
  display: grid;
  gap: 4px;
}

.table-editor-grid .row-actions {
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.table-editor-grid .column-actions {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.table-preview {
  margin-top: 12px;
}

.table-preview textarea {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  white-space: pre;
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

.preview-pane[data-preview-theme="dark"] {
  background: #0f172a;
}

.preview-pane[data-preview-theme="dark"] .preview-document {
  color: #dbeafe;
  background: #111827;
}

.preview-pane[data-preview-theme="dark"] .preview-document h1,
.preview-pane[data-preview-theme="dark"] .preview-document h2,
.preview-pane[data-preview-theme="dark"] .preview-document h3 {
  color: #f8fafc;
}

.preview-pane[data-preview-theme="dark"] .preview-document pre {
  background: #0b1220;
  color: #e5e7eb;
}

.preview-pane[data-preview-theme="light"] {
  background: #ffffff;
}

.preview-pane[data-preview-theme="light"] .preview-document {
  color: #1f2937;
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

.preview-document p,
.preview-document li,
.preview-document blockquote {
  orphans: 2;
  widows: 2;
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

.preview-document figure[data-float="right"],
.preview-document .figure-float-right {
  float: right;
  max-width: 45%;
  margin: 0 0 16px 24px;
}

.preview-document figure[data-float="left"],
.preview-document .figure-float-left {
  float: left;
  max-width: 45%;
  margin: 0 24px 16px 0;
}

.preview-document figure[data-fit="cover"] img,
.preview-document .figure-fit-cover img {
  width: 100%;
  aspect-ratio: 16 / 9;
  object-fit: cover;
}

.preview-document figure[data-position="top"] img,
.preview-document .figure-position-top img {
  object-position: center top;
}

.preview-document figure[data-position="bottom"] img,
.preview-document .figure-position-bottom img {
  object-position: center bottom;
}

.preview-document figure[data-position="left"] img,
.preview-document .figure-position-left img {
  object-position: left center;
}

.preview-document figure[data-position="right"] img,
.preview-document .figure-position-right img {
  object-position: right center;
}

.preview-document figure[data-position="top-left"] img,
.preview-document .figure-position-top-left img {
  object-position: left top;
}

.preview-document figure[data-position="top-right"] img,
.preview-document .figure-position-top-right img {
  object-position: right top;
}

.preview-document figure[data-position="bottom-left"] img,
.preview-document .figure-position-bottom-left img {
  object-position: left bottom;
}

.preview-document figure[data-position="bottom-right"] img,
.preview-document .figure-position-bottom-right img {
  object-position: right bottom;
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
  position: relative;
}

.preview-document .citation:focus {
  outline: 2px solid #275da8;
  outline-offset: 2px;
}

.preview-document .citation[data-citation-detail]:hover::after,
.preview-document .citation[data-citation-detail]:focus::after {
  content: attr(data-citation-detail);
  position: absolute;
  left: 0;
  bottom: calc(100% + 6px);
  z-index: 20;
  width: max-content;
  max-width: min(320px, 80vw);
  white-space: normal;
  background: #111827;
  color: #ffffff;
  border: 1px solid #374151;
  border-radius: 4px;
  box-shadow: 0 12px 24px rgba(15, 23, 42, 0.22);
  padding: 8px 10px;
  font-size: 0.84rem;
  line-height: 1.35;
  font-weight: 500;
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

.preview-document .equation {
  margin: 18px 0;
}

.preview-document .math-rendered {
  font-family: Georgia, "Times New Roman", serif;
  font-size: 1.08em;
}

.preview-document .math-display {
  padding: 12px;
  border: 1px solid #d8e0e8;
  background: #f8fafc;
  text-align: center;
}

.preview-document .math-frac {
  display: inline-grid;
  grid-template-rows: auto auto;
  vertical-align: middle;
  text-align: center;
}

.preview-document .math-frac span:first-child {
  border-bottom: 1px solid currentColor;
}

.preview-document .math-sqrt::before {
  content: "√";
}

.preview-document .math-root-index {
  font-size: 0.65em;
  vertical-align: super;
}

.preview-document .math-text {
  font-family: inherit;
}

.preview-document .math-hat::before,
.preview-document .math-vec::before {
  display: block;
  height: 0;
  line-height: 0;
  text-align: center;
}

.preview-document .math-hat::before {
  content: "^";
}

.preview-document .math-vec::before {
  content: "→";
}

.preview-document .math-overline {
  border-top: 1px solid currentColor;
}

.preview-document .math-underline {
  border-bottom: 1px solid currentColor;
}

.preview-document .math-align-separator {
  display: inline-block;
  min-width: 1ch;
}

.preview-document .math-matrix {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  vertical-align: middle;
}

.preview-document .math-matrix::before,
.preview-document .math-matrix::after {
  font-size: 1.8em;
  line-height: 1;
}

.preview-document .math-matrix.matrix-round::before {
  content: "(";
}

.preview-document .math-matrix.matrix-round::after {
  content: ")";
}

.preview-document .math-matrix.matrix-square::before {
  content: "[";
}

.preview-document .math-matrix.matrix-square::after {
  content: "]";
}

.preview-document .math-matrix.matrix-vertical::before,
.preview-document .math-matrix.matrix-vertical::after {
  content: "|";
}

.preview-document .math-matrix table {
  width: auto;
  border-collapse: collapse;
  display: inline-table;
}

.preview-document .math-matrix td {
  border: 0;
  padding: 0 0.45ch;
  text-align: center;
}

.preview-document .math-source-inline {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
}

.preview-document pre,
.sidebar pre {
  overflow: auto;
  padding: 10px;
  background: #edf1f5;
}

.transform-calc {
  padding: 10px;
  border: 1px solid #c9d2dc;
  background: #f7f9fb;
}

.status-bar {
  justify-content: space-between;
  gap: 8px;
  overflow-x: auto;
  overflow-y: hidden;
  border-top: 1px solid #c9d2dc;
  border-bottom: 0;
  background: #f8fafc;
  white-space: nowrap;
}

.status-message,
.word-stats,
.watch-status,
.export-progress,
.error {
  display: inline-flex;
  align-items: center;
  min-height: 20px;
  min-width: 0;
  padding: 1px 7px;
  border: 1px solid transparent;
  border-radius: 999px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-message {
  max-width: 36vw;
  border-color: #d9e1ea;
  background: #ffffff;
  color: #324156;
}

.word-stats,
.watch-status,
.export-progress {
  background: #eef3f8;
  color: #4a5b6d;
}

.conflict-actions,
.compile-actions {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.error {
  border-color: #efc7c7;
  background: #fff5f5;
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
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 6px;
  align-items: start;
  min-height: 26px;
  margin: 0;
  padding: 6px 10px;
  border-bottom: 1px solid #e2e8f0;
  white-space: pre-wrap;
  word-break: break-word;
}

.conflict-diff-cell button {
  min-width: 48px;
  padding: 4px 6px;
  font-size: 12px;
}

.conflict-diff-cell pre {
  margin: 0;
  white-space: pre-wrap;
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

.merge-composition {
  display: grid;
  gap: 8px;
  padding: 10px;
  border: 1px solid #c9d2dc;
  background: #f8fafc;
}

.merge-composition header,
.merge-composition li {
  display: flex;
  align-items: center;
  gap: 8px;
}

.merge-composition header {
  justify-content: space-between;
}

.merge-composition ol {
  display: grid;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.merge-composition li {
  padding: 6px;
  border: 1px solid #dbe3ec;
  background: #ffffff;
}

.merge-composition code {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.merge-composition button {
  padding: 4px 6px;
  font-size: 12px;
}

.merge-source {
  min-width: 74px;
  color: #475569;
  font-size: 12px;
  font-weight: 700;
  text-transform: capitalize;
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
  .command-bar {
    max-height: none;
    overflow-y: visible;
  }

  .command-toolbar-row {
    overflow-x: auto;
    overflow-y: hidden;
    padding-bottom: 3px;
  }

  .command-group,
  .compact-field {
    max-width: 100%;
  }

  .workspace,
  .workspace.mode-source,
  .workspace.mode-focus,
  .workspace.mode-preview,
  .workspace.mode-export,
  .workspace.mode-presentation {
    grid-template-columns: 1fr;
  }

  .sidebar {
    display: block;
    max-height: 220px;
    border-right: 0;
    border-bottom: 1px solid #c9d2dc;
  }

  .pane-splitter {
    display: none;
  }

  .editor-pane,
  .preview-pane {
    min-height: 320px;
    border-right: 0;
    border-bottom: 1px solid #c9d2dc;
  }

  .preview-document {
    padding: 24px;
  }

  .compare-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 600px) {
  .app-shell {
    grid-template-rows: 38px auto minmax(0, 1fr) 34px;
  }

  .command-group,
  .command-toolbar-row-view {
    width: 100%;
  }

  .command-toolbar-title {
    width: 72px;
    flex-basis: 72px;
  }

  .command-group-actions {
    flex-wrap: nowrap;
  }

  .compact-field {
    grid-template-columns: auto max-content;
  }

  .compact-field select {
    min-width: 104px;
  }

  .compact-field-range {
    grid-template-columns: auto 92px 32px;
  }

  .status-bar {
    justify-content: start;
  }
}
</style>
