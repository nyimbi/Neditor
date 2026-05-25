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
            :title="document.path || document.title"
            :data-document-path="document.path || ''"
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
      <section
        v-for="row in commandToolbarRows"
        :key="row.id"
        class="command-toolbar-row"
        :class="{ collapsed: isToolbarCollapsed(row.id) }"
        :aria-label="`${row.label} toolbar`"
      >
        <button
          class="command-toolbar-heading"
          type="button"
          :aria-label="`${isToolbarCollapsed(row.id) ? 'Expand' : 'Collapse'} ${row.label} toolbar`"
          :aria-expanded="!isToolbarCollapsed(row.id)"
          @click="toggleToolbarRow(row.id)"
        >
          <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
            <path v-for="path in toolbarIconPaths(isToolbarCollapsed(row.id) ? 'expand' : 'collapse')" :key="path" :d="path"></path>
          </svg>
          <span>{{ row.label }}</span>
        </button>
        <section v-for="group in row.groups" v-show="!isToolbarCollapsed(row.id)" :key="group.id" class="command-group" :aria-label="`${group.label} commands`">
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
      <section class="command-toolbar-row command-toolbar-row-view" :class="{ collapsed: isToolbarCollapsed('view') }" aria-label="View toolbar">
        <button
          class="command-toolbar-heading"
          type="button"
          :aria-label="`${isToolbarCollapsed('view') ? 'Expand' : 'Collapse'} View toolbar`"
          :aria-expanded="!isToolbarCollapsed('view')"
          @click="toggleToolbarRow('view')"
        >
          <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
            <path v-for="path in toolbarIconPaths(isToolbarCollapsed('view') ? 'expand' : 'collapse')" :key="path" :d="path"></path>
          </svg>
          <span>View</span>
        </button>
        <label class="compact-field">
          <span>Mode</span>
          <select v-show="!isToolbarCollapsed('view')" v-model="store.mode" aria-label="View mode">
            <option value="split">Split</option>
            <option value="source">Source</option>
            <option value="preview">Preview</option>
            <option value="focus">Focus</option>
            <option value="outline">Outline</option>
            <option value="export">Export</option>
            <option value="review">Review</option>
            <option value="presentation">Presentation</option>
          </select>
        </label>
        <label class="compact-field">
          <span>Panel</span>
          <select v-show="!isToolbarCollapsed('view')" v-model="store.sidebar" aria-label="Sidebar panel">
            <option value="files">Files</option>
            <option value="outline">Outline</option>
            <option value="diagnostics">Diagnostics</option>
            <option value="tables">Tables</option>
            <option value="templates">Templates</option>
            <option value="references">References</option>
            <option value="exports">Exports</option>
            <option value="versioning">Versioning</option>
            <option value="review">Review</option>
            <option value="help">Help</option>
            <option value="settings">Settings</option>
          </select>
        </label>
        <label class="compact-field">
          <span>Buttons</span>
          <select v-show="!isToolbarCollapsed('view')" v-model="store.toolbarDisplay" aria-label="Toolbar button display">
            <option value="both">Icons and text</option>
            <option value="icons">Icons only</option>
            <option value="text">Text only</option>
          </select>
        </label>
        <label class="compact-field compact-field-range">
          <span>Text</span>
          <input
            v-show="!isToolbarCollapsed('view')"
            v-model.number="store.toolbarTextSize"
            aria-label="Toolbar text size"
            type="range"
            min="9"
            max="15"
            step="1"
          />
          <output v-show="!isToolbarCollapsed('view')" aria-label="Current toolbar text size">{{ store.toolbarTextSize }}px</output>
        </label>
        <button v-show="!isToolbarCollapsed('view')" class="compact-toolbar-toggle" type="button" @click="setAllCommandToolbarsCollapsed(!anyCommandToolbarsCollapsed)">
          <span class="command-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" focusable="false">
              <path v-for="path in toolbarIconPaths(anyCommandToolbarsCollapsed ? 'expand' : 'collapse')" :key="path" :d="path"></path>
            </svg>
          </span>
          <span>{{ anyCommandToolbarsCollapsed ? "Expand all" : "Collapse all" }}</span>
        </button>
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
      <section v-if="store.mode === 'outline'" id="outline-mode" class="outline-mode-pane" aria-label="Document outline mode" tabindex="-1">
        <header class="outline-mode-header">
          <div>
            <h2>Outline</h2>
            <p>{{ outlineModeHeadings.length }} chapters, sections, subsections, and subsubsections.</p>
          </div>
          <button type="button" :disabled="!outlineModeHeadings.length" @click="openDocsLiveFromDocumentOutline">
            Flesh out with Docs Live
          </button>
          <div class="outline-mode-create">
            <label>
              Title
              <input v-model="outlineModeNewTitle" aria-label="New outline heading title" />
            </label>
            <label>
              Level
              <select v-model.number="outlineModeNewLevel" aria-label="New outline heading level">
                <option :value="1">Chapter</option>
                <option :value="2">Section</option>
                <option :value="3">Subsection</option>
                <option :value="4">Subsubsection</option>
              </select>
            </label>
            <button type="button" @click="createOutlineHeading()">Add heading</button>
          </div>
        </header>
        <section v-if="outlineModeHeadings.length" class="outline-mode-list" aria-label="Editable document outline">
          <article
            v-for="heading in outlineModeHeadings"
            :key="`${heading.line}-${heading.anchor}`"
            class="outline-mode-row"
            :style="{ '--outline-depth': String(heading.level - 1) }"
          >
            <span class="outline-mode-kind">{{ outlineHeadingKind(heading.level) }}</span>
            <input
              :value="heading.text"
              :aria-label="`Outline title ${heading.text}`"
              @change="renameOutlineHeading(heading, eventValue($event))"
              @keydown.enter.prevent="renameOutlineHeading(heading, eventValue($event))"
            />
            <select :value="heading.level" :aria-label="`Outline level ${heading.text}`" @change="setOutlineHeadingLevel(heading, Number(eventValue($event)))">
              <option :value="1">Chapter</option>
              <option :value="2">Section</option>
              <option :value="3">Subsection</option>
              <option :value="4">Subsubsection</option>
            </select>
            <div class="outline-mode-actions">
              <button type="button" @click="createOutlineHeading(heading, Math.min(4, heading.level + 1))">Add child</button>
              <button type="button" @click="createOutlineHeading(heading, heading.level)">Add sibling</button>
              <button type="button" @click="goToSourceTarget(heading)">Go</button>
              <button type="button" @click="deleteOutlineHeading(heading)">Delete</button>
            </div>
          </article>
        </section>
        <section v-else class="outline-mode-empty" aria-label="Empty outline">
          <h3>No outline yet</h3>
          <p>Create a chapter to start structuring the document before drafting the body.</p>
          <button type="button" @click="createOutlineHeading()">Create first chapter</button>
          <button type="button" :disabled="!outlineDraftItems.length" @click="openDocsLiveFromOutline">Use planner outline in Docs Live</button>
        </section>
      </section>

      <aside v-show="store.mode !== 'outline'" id="document-sidebar" class="sidebar" aria-label="Document workspace" tabindex="-1">
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
          <section class="outline-planner" aria-label="Outline planner">
            <h3>Plan</h3>
            <label>
              Document title
              <input v-model="outlineDraftTitle" placeholder="Board Brief" />
            </label>
            <label>
              Outline draft
              <textarea
                v-model="outlineDraftText"
                rows="9"
                aria-label="Editable document outline"
                placeholder="- Executive Summary&#10;  - Decision Needed&#10;  - Key Risks&#10;- Financial Case"
              ></textarea>
            </label>
            <label><input v-model="outlineDraftIncludeToc" type="checkbox" /> Include table of contents</label>
            <div class="outline-planner-actions">
              <button type="button" @click="loadOutlineDraftFromDocument">Load from document</button>
              <button type="button" :disabled="!outlineDraftItems.length" @click="openDocsLiveFromOutline">Flesh out with Docs Live</button>
              <button type="button" :disabled="!outlineDraftItems.length" @click="createDocumentFromOutline">Create document from outline</button>
              <button type="button" :disabled="!outlineDraftItems.length" @click="appendOutlineToDocument">Append outline</button>
            </div>
            <p class="sidebar-hint">{{ outlineDraftItems.length }} planned sections. Use indentation, bullets, numbers, or Markdown heading marks.</p>
          </section>
          <p v-if="!outlineHeadings.length" class="sidebar-hint">Add headings directly or create a document from an outline plan.</p>
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
          <section v-if="activeAgentControlCenter" class="agent-control-center persistent-agent-control" :data-status="activeAgentControlCenter.status" aria-label="Persistent AI control center">
            <header>
              <div>
                <strong>AI Control Center</strong>
                <span>{{ activeAgentControlCenter.summary }}</span>
              </div>
              <small>{{ activeAgentControlCenter.readinessScore }}/100 readiness</small>
            </header>
            <section class="agent-control-grid">
              <article>
                <h3>Next actions</h3>
                <ul>
                  <li v-for="action in activeAgentControlCenter.nextActions" :key="`persistent-${action.lane}-${action.label}`">
                    <strong>{{ action.label }}</strong>
                    <span>{{ action.lane }} | {{ action.status }}</span>
                    <p>{{ action.detail }}</p>
                  </li>
                </ul>
              </article>
              <article>
                <h3>Source grounding</h3>
                <ul>
                  <li v-for="item in activeAgentControlCenter.sourceGrounding" :key="`persistent-source-${item.label}`" :data-status="item.status">
                    <strong>{{ item.label }}</strong>
                    <span>{{ item.status }}</span>
                    <p>{{ item.detail }}</p>
                  </li>
                </ul>
              </article>
              <article>
                <h3>Governance</h3>
                <ul>
                  <li v-for="item in activeAgentControlCenter.governance" :key="`persistent-governance-${item.label}`" :data-status="item.status">
                    <strong>{{ item.label }}</strong>
                    <span>{{ item.status }}</span>
                    <p>{{ item.detail }}</p>
                  </li>
                </ul>
              </article>
              <article>
                <h3>Distribution state</h3>
                <ul>
                  <li v-for="item in activeAgentControlCenter.distribution" :key="`persistent-distribution-${item.label}`" :data-status="item.status">
                    <strong>{{ item.label }}</strong>
                    <span>{{ item.status }}</span>
                    <p>{{ item.detail }}</p>
                  </li>
                </ul>
              </article>
            </section>
            <div class="agent-section-actions">
              <button type="button" @click="openAgentWorkspace()">Open agent workspace</button>
              <button type="button" @click="runAgentPlanReview">Review readiness</button>
              <button type="button" @click="runAgentPlanDistribution">Distribution prep</button>
            </div>
          </section>
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
            Owner
            <input :value="String(active.compile?.metadata.owner || '')" @change="setFrontMatterField('owner', inputValue($event))" />
          </label>
          <label>
            Release target
            <input :value="String(active.compile?.metadata.releaseTarget || '')" @change="setFrontMatterField('releaseTarget', inputValue($event))" />
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

        <template v-else-if="store.sidebar === 'help'">
          <h2>Help Center</h2>
          <section class="help-center" aria-label="Help center">
            <div class="help-controls">
              <label>
                Search help
                <input v-model="helpQuery" type="search" placeholder="export, outline, voice, shortcut" />
              </label>
              <label>
                Area
                <select v-model="helpCategory">
                  <option value="all">All areas</option>
                  <option v-for="category in helpCategoryOptions" :key="category.id" :value="category.id">{{ category.label }}</option>
                </select>
              </label>
            </div>
            <div class="help-quick-actions" aria-label="Popular help actions">
              <button type="button" @click="openHelp('getting-started')">Start</button>
              <button type="button" @click="openHelp('docs-live')">Docs Live</button>
              <button type="button" @click="openHelp('agent-lifecycle-governance')">AI Governance</button>
              <button type="button" @click="openGuidedDemo()">Guided demo</button>
              <button type="button" @click="openHelp('export-publishing')">Export</button>
              <button type="button" @click="openHelp('keyboard-shortcuts')">Shortcuts</button>
            </div>
            <section class="help-topic-list" role="list" aria-label="Help topics">
              <div v-for="topic in filteredHelpTopics" :key="topic.id" role="listitem">
                <button
                  class="help-topic-button"
                  :class="{ active: topic.id === selectedHelpTopic?.id }"
                  type="button"
                  @click="selectHelpTopic(topic.id)"
                >
                  <strong>{{ topic.title }}</strong>
                  <small>{{ topic.summary }}</small>
                </button>
              </div>
            </section>
            <p v-if="!filteredHelpTopics.length" class="sidebar-hint">No help topics matched that search.</p>
            <article v-if="selectedHelpTopic" class="help-topic-detail" aria-label="Selected help topic">
              <div class="help-topic-header">
                <small>{{ helpCategoryLabel(selectedHelpTopic.category) }}</small>
                <h3>{{ selectedHelpTopic.title }}</h3>
                <p>{{ selectedHelpTopic.summary }}</p>
              </div>
              <p class="help-when">{{ selectedHelpTopic.when }}</p>
              <ol class="help-steps">
                <li v-for="step in selectedHelpTopic.steps" :key="step">{{ step }}</li>
              </ol>
              <ul class="help-tips">
                <li v-for="tip in selectedHelpTopic.tips" :key="tip">{{ tip }}</li>
              </ul>
              <div class="help-action-row">
                <button v-for="action in selectedHelpTopic.actions" :key="action.label" type="button" @click="runHelpAction(action)">
                  {{ action.label }}
                </button>
              </div>
              <div class="help-keywords" aria-label="Topic keywords">
                <span v-for="keyword in selectedHelpTopic.keywords" :key="keyword">{{ keyword }}</span>
              </div>
            </article>
          </section>
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

      <section id="markdown-source" v-show="store.mode !== 'preview' && store.mode !== 'export' && store.mode !== 'presentation' && store.mode !== 'outline'" class="editor-pane" aria-label="Markdown source" tabindex="-1">
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
        v-show="store.mode !== 'source' && store.mode !== 'focus' && store.mode !== 'outline'"
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

    <div v-if="buttonHelp.visible" class="button-help-tooltip" role="tooltip" :style="buttonHelpStyle">
      {{ buttonHelp.text }}
    </div>

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
      v-if="docsLiveOpen"
      ref="docsLiveDialog"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="Docs Live voice drafting"
      tabindex="-1"
      @keydown="handleModalKeydown('docs-live', $event)"
    >
      <form class="modal docs-live-modal" @submit.prevent="generateDocsLiveDraft">
        <header>
          <h2>Docs Live</h2>
          <button type="button" aria-label="Close Docs Live" @click="closeDocsLive">x</button>
        </header>

        <section class="docs-live-grid">
          <label>
            Document type
            <select v-model="docsLiveDocumentType" data-initial-focus @change="refreshDocsLiveQuestionnaire">
              <option v-for="type in docsLiveDocumentTypes" :key="type.id" :value="type.id">{{ type.label }}</option>
            </select>
          </label>
          <label>
            Document title
            <input v-model="docsLiveTitle" placeholder="Board brief, proposal, report" />
          </label>
          <label>
            Drafting depth
            <select v-model="docsLiveDraftingDepth">
              <option v-for="depth in docsLiveDraftingDepthOptions" :key="depth.value" :value="depth.value">{{ depth.label }}</option>
            </select>
          </label>
          <section class="docs-live-intent-brief docs-live-wide" aria-label="AI Create intent brief">
            <header>
              <div>
                <strong>Intent Brief</strong>
                <span>Business context required before a responsible first draft.</span>
              </div>
              <small>{{ docsLiveIntentCompletion }}</small>
            </header>
            <div class="docs-live-intent-grid">
              <label v-for="field in docsLiveIntentFields" :key="field.key">
                {{ field.label }}
                <input
                  :value="docsLivePlaceholderValue(field.key)"
                  :placeholder="field.placeholder"
                  @change="updateDocsLiveIntentField(field.key, inputValue($event))"
                />
              </label>
            </div>
          </section>
          <label class="docs-live-wide">
            Outline
            <textarea v-model="docsLiveOutlineText" rows="7" placeholder="- Executive Summary&#10;- Recommendation&#10;- Next Steps"></textarea>
          </label>
          <section class="docs-live-voice docs-live-wide" aria-label="Voice dictation">
            <div class="docs-live-voice-actions">
              <button type="button" :disabled="!docsLiveSpeechAvailable" @click="toggleDocsLiveDictation">
                {{ docsLiveListening ? "Stop dictation" : "Start dictation" }}
              </button>
              <button type="button" :disabled="docsLiveRuntimeChecking" @click="checkDocsLiveRuntime">
                {{ docsLiveRuntimeChecking ? "Checking runtime..." : "Check AI runtime" }}
              </button>
              <span role="status">{{ docsLiveSpeechStatus }}</span>
            </div>
            <section v-if="docsLiveRuntimeReport" class="docs-live-runtime" aria-label="AI runtime readiness">
              <header>
                <strong>Runtime readiness</strong>
                <span>{{ docsLiveRuntimeReport.issues.length }} issues</span>
              </header>
              <ul>
                <li>Speech: {{ docsLiveRuntimeReport.speechRecognition.state }} - {{ docsLiveRuntimeReport.speechRecognition.detail }}</li>
                <li>Microphone: {{ docsLiveRuntimeReport.microphonePermission.state }} - {{ docsLiveRuntimeReport.microphonePermission.detail }}</li>
                <li>Clipboard read: {{ docsLiveRuntimeReport.clipboardRead.state }} - {{ docsLiveRuntimeReport.clipboardRead.detail }}</li>
                <li>Clipboard write: {{ docsLiveRuntimeReport.clipboardWrite.state }} - {{ docsLiveRuntimeReport.clipboardWrite.detail }}</li>
              </ul>
              <textarea :value="docsLiveRuntimeReport.markdown" rows="7" readonly aria-label="AI runtime readiness report"></textarea>
            </section>
            <label>
              Spoken direction
              <textarea v-model="docsLiveTranscript" rows="6" placeholder="Dictate what should change, who it is for, and the outcome you need."></textarea>
            </label>
            <p v-if="docsLiveInterimTranscript" class="sidebar-hint">{{ docsLiveInterimTranscript }}</p>
          </section>
          <label>
            Context and answers
            <textarea v-model="docsLiveContext" rows="8" placeholder="Answer the questionnaire or add freeform context, constraints, examples, evidence, tone, and review expectations."></textarea>
          </label>
          <label>
            Placeholder values
            <textarea v-model="docsLivePlaceholderText" rows="8" placeholder="client: Acme&#10;audience: executive team&#10;deadline: June 1&#10;owner: Finance"></textarea>
          </label>
          <section class="docs-live-placeholder-manager docs-live-wide" aria-label="Docs Live placeholder manager">
            <header>
              <div>
                <strong>Placeholder Manager</strong>
                <span>{{ docsLivePlaceholderRows.length }} values | Missing {{ docsLiveMissingPlaceholderKeys.join(", ") || "none" }}</span>
              </div>
            </header>
            <div class="docs-live-placeholder-add">
              <label>
                Key
                <input v-model="docsLivePlaceholderKey" placeholder="client, amount, source" />
              </label>
              <label>
                Value
                <input v-model="docsLivePlaceholderDraftValue" placeholder="Acme, $250K, audited forecast" />
              </label>
              <label>
                Type
                <select v-model="docsLivePlaceholderDraftKind">
                  <option v-for="kind in docsLivePlaceholderKindOptions" :key="kind.value" :value="kind.value">{{ kind.label }}</option>
                </select>
              </label>
              <label>
                Source
                <input v-model="docsLivePlaceholderDraftSource" placeholder="Finance workbook, GC review, customer brief" />
              </label>
              <label>
                Review
                <select v-model="docsLivePlaceholderDraftStatus">
                  <option v-for="status in docsLivePlaceholderReviewStatusOptions" :key="status.value" :value="status.value">{{ status.label }}</option>
                </select>
              </label>
              <button type="button" :disabled="!docsLivePlaceholderKey.trim() || !docsLivePlaceholderDraftValue.trim()" @click="addDocsLivePlaceholder">
                Add value
              </button>
            </div>
            <div class="docs-live-placeholder-grid" role="table" aria-label="Managed placeholder values">
              <div class="docs-live-placeholder-head" role="row">
                <span role="columnheader">Key</span>
                <span role="columnheader">Value</span>
                <span role="columnheader">Type</span>
                <span role="columnheader">Source</span>
                <span role="columnheader">Review</span>
                <span role="columnheader">Action</span>
              </div>
              <div v-for="entry in docsLivePlaceholderRows" :key="entry.key" role="row">
                <span role="cell">{{ entry.key }}</span>
                <input
                  role="cell"
                  :value="entry.value"
                  :aria-label="`Value for ${entry.key}`"
                  @change="updateDocsLivePlaceholder(entry.key, inputValue($event))"
                />
                <select
                  role="cell"
                  :value="entry.kind"
                  :aria-label="`Type for ${entry.key}`"
                  @change="updateDocsLivePlaceholderKind(entry.key, inputValue($event))"
                >
                  <option v-for="kind in docsLivePlaceholderKindOptions" :key="kind.value" :value="kind.value">{{ kind.label }}</option>
                </select>
                <input
                  role="cell"
                  :value="entry.source"
                  :aria-label="`Source for ${entry.key}`"
                  placeholder="source or evidence"
                  @change="updateDocsLivePlaceholderMetadata(entry.key, { source: inputValue($event) })"
                />
                <select
                  role="cell"
                  :value="entry.reviewStatus"
                  :aria-label="`Review status for ${entry.key}`"
                  @change="updateDocsLivePlaceholderReviewStatus(entry.key, inputValue($event))"
                >
                  <option v-for="status in docsLivePlaceholderReviewStatusOptions" :key="status.value" :value="status.value">{{ status.label }}</option>
                </select>
                <button type="button" role="cell" @click="removeDocsLivePlaceholderValue(entry.key)">Remove</button>
              </div>
            </div>
          </section>
          <label>
            AI-created questionnaire
            <textarea v-model="docsLiveQuestionnaireText" rows="7" readonly></textarea>
          </label>
          <label>
            Questionnaire answers
            <textarea
              v-model="docsLiveQuestionnaireAnswerText"
              rows="7"
              placeholder="1. The reader should approve renewal.&#10;2. Include usage growth, budget, risks, and named owner.&#10;3. Leave financial assumptions marked for review."
            ></textarea>
          </label>
          <label>
            Apply result
            <select v-model="docsLiveInsertMode">
              <option value="replace">Replace document</option>
              <option value="append">Append to document</option>
              <option value="selection">Replace selection</option>
              <option value="section">Replace matching section</option>
            </select>
          </label>
          <p v-if="docsLiveTargetSection" class="sidebar-hint">
            Target section: {{ docsLiveTargetSection.heading }}. Apply draft will replace that matching Markdown section when it exists, or append the generated section when it does not.
          </p>
        </section>

        <section v-if="docsLiveDraft?.issues.length" class="issue-list">
          <p v-for="issue in docsLiveDraft.issues" :key="issue">{{ issue }}</p>
        </section>

        <section v-if="docsLiveDraft" class="docs-live-workflow" aria-label="Docs Live section drafting workflow">
          <header>
            <strong>Systematic drafting workflow</strong>
            <span>{{ docsLiveDraft.sections.length }} sections prepared for review</span>
          </header>
          <ol>
            <li v-for="step in docsLiveDraft.workflow" :key="step.id" :data-status="step.status">
              <strong>{{ step.label }}</strong>
              <small>{{ step.status }}</small>
              <span>{{ step.detail }}</span>
            </li>
          </ol>
          <div class="docs-live-section-cards">
            <article v-for="section in docsLiveDraft.sections" :key="section.title">
              <strong>{{ section.title }}</strong>
              <span>{{ section.qaFocus }}</span>
              <p>{{ section.draftingBrief }}</p>
              <ol class="docs-live-section-stage-list" :aria-label="`${section.title} drafting stages`">
                <li v-for="stage in section.stagePlan" :key="`${section.title}-${stage.id}`" :data-status="stage.status">
                  <strong>{{ stage.label }}</strong>
                  <small>{{ stage.status }}</small>
                  <span>{{ stage.detail }}</span>
                </li>
              </ol>
            </article>
          </div>
          <div class="docs-live-review-packet" aria-label="Docs Live review preparation packet">
            <header class="docs-live-review-packet-header">
              <div>
                <strong>Review preparation packet</strong>
                <span>Export the AI runbook, QA register, cleanup tasks, and reviewer prompts without replacing the draft.</span>
              </div>
              <div class="docs-live-review-actions">
                <button type="button" @click="insertDocsLiveReviewPacket">Insert packet</button>
                <button type="button" @click="copyDocsLiveReviewPacket">Copy packet</button>
              </div>
            </header>
            <section>
              <strong>Context package</strong>
              <ul>
                <li v-for="source in docsLiveDraft.reviewPacket.contextSources" :key="source">{{ source }}</li>
              </ul>
            </section>
            <section>
              <strong>Section runbook</strong>
              <ol>
                <li v-for="item in docsLiveDraft.reviewPacket.sectionRunbook" :key="item">{{ item }}</li>
              </ol>
            </section>
            <section>
              <strong>QA register</strong>
              <ul>
                <li v-for="item in docsLiveDraft.reviewPacket.qaRegister" :key="item">{{ item }}</li>
              </ul>
            </section>
            <section>
              <strong>Humanization checklist</strong>
              <ul>
                <li v-for="item in docsLiveDraft.reviewPacket.humanizationChecklist" :key="item">{{ item }}</li>
              </ul>
            </section>
            <section>
              <strong>Review packet</strong>
              <ul>
                <li v-for="item in docsLiveDraft.reviewPacket.reviewerHandoff" :key="item">{{ item }}</li>
              </ul>
            </section>
          </div>
        </section>

        <section v-if="docsLiveGeneratedMarkdown" class="docs-live-preview" aria-label="Docs Live generated draft">
          <header>
            <strong>{{ docsLiveDraft?.sections.length || 0 }} drafted sections</strong>
            <span>{{ docsLiveDraft?.title }}</span>
            <div class="docs-live-draft-actions">
              <button type="button" @click="appendDocsLiveDraftForReview">Append for review</button>
              <button type="button" @click="copyDocsLiveDraft">Copy draft</button>
            </div>
          </header>
          <textarea :value="docsLiveGeneratedMarkdown" rows="12" readonly aria-label="Docs Live generated Markdown"></textarea>
        </section>

        <section v-if="store.docsLiveDraftHistory.length" class="docs-live-history" aria-label="Docs Live draft history">
          <header>
            <div>
              <strong>Recent Docs Live drafts</strong>
              <span>{{ store.docsLiveDraftHistory.length }} saved locally for reuse</span>
            </div>
            <button type="button" @click="clearDocsLiveDraftHistory">Clear history</button>
          </header>
          <article v-for="item in store.docsLiveDraftHistory.slice(0, 6)" :key="item.draftId">
            <div>
              <strong>{{ item.title }}</strong>
              <span>{{ item.sectionCount }} sections / {{ item.documentType }}</span>
              <p>{{ item.markdownPreview }}</p>
            </div>
            <div class="docs-live-history-actions">
              <button type="button" @click="appendDocsLiveHistoryDraft(item)">Append draft</button>
              <button type="button" @click="copyDocsLiveHistoryDraft(item)">Copy draft</button>
              <button type="button" @click="insertDocsLiveHistoryReviewPacket(item)">Insert packet</button>
              <button type="button" @click="copyDocsLiveHistoryReviewPacket(item)">Copy packet</button>
              <button type="button" @click="removeDocsLiveHistoryDraft(item)">Remove</button>
            </div>
          </article>
        </section>

        <footer>
          <button type="button" @click="closeDocsLive">Cancel</button>
          <button type="button" @click="refreshDocsLiveQuestionnaire">Build questionnaire</button>
          <button type="button" @click="loadDocsLiveOutlineFromDocument">Use document outline</button>
          <button type="submit">Generate draft</button>
          <button type="button" :disabled="!docsLiveGeneratedMarkdown" @click="applyDocsLiveDraft">Apply draft</button>
        </footer>
      </form>
    </section>

    <section
      v-if="agentWorkspaceOpen"
      ref="agentWorkspaceDialog"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="AI agent workspace"
      tabindex="-1"
      @keydown="handleModalKeydown('agent-workspace', $event)"
    >
      <form class="modal agent-workspace-modal" @submit.prevent="buildAgentWorkspacePlan">
        <header>
          <div>
            <h2>AI Agent Workspace</h2>
            <p>Plan creation, composition, editing, revision, review, and distribution from one instruction.</p>
          </div>
          <button type="button" aria-label="Close AI agent workspace" @click="closeAgentWorkspace">x</button>
        </header>
        <label>
          What should NEditor do?
          <textarea
            v-model="agentInstruction"
            rows="5"
            data-initial-focus
            placeholder="Create a board memo for the executive team, revise it for the CFO, check evidence gaps, and prepare PDF plus Google Docs distribution."
          ></textarea>
        </label>
        <label>
          Context answers and constraints
          <textarea
            v-model="agentContextAnswers"
            rows="4"
            placeholder="Answer missing inputs, add source facts, target reviewer, approvals, distribution constraints, tone, deadlines, or placeholder values. These answers feed the next plan, packet, Docs Live handoff, and provider request."
          ></textarea>
        </label>
        <section class="agent-source-pack-builder" aria-label="Agent source pack builder">
          <header>
            <div>
              <strong>Source Pack Builder</strong>
              <span>
                {{ agentSourcePackPreview.items.length }} items |
                {{ agentSourcePackPreview.claims.length }} claims |
                {{ agentSourcePackPreview.urls.length }} URLs |
                {{ agentSourcePackPreview.files.length }} files |
                {{ agentSourcePackPreview.reviewerComments.length }} reviewer comments
              </span>
            </div>
          </header>
          <div class="agent-source-pack-add">
            <label>
              Type
              <select v-model="agentSourcePackKind">
                <option value="note">Note</option>
                <option value="claim">Claim</option>
                <option value="url">URL</option>
                <option value="file">File</option>
                <option value="reference">Reference</option>
                <option value="reviewer-comment">Reviewer comment</option>
              </select>
            </label>
            <label>
              Label
              <input v-model="agentSourcePackLabel" placeholder="Q2 forecast, CFO comment, research URL" />
            </label>
            <label>
              Detail
              <textarea v-model="agentSourcePackDetail" rows="3" placeholder="Paste the fact, link, file path, reviewer note, or citation detail."></textarea>
            </label>
            <button type="button" :disabled="!agentSourcePackLabel.trim() && !agentSourcePackDetail.trim()" @click="addAgentSourcePackItem">Add source</button>
          </div>
          <label>
            Managed source pack
            <textarea
              v-model="agentSourcePackText"
              rows="6"
              placeholder="[claim] ARR forecast: ARR grows 18% in Q2 according to finance workbook&#10;[url] Pricing source: https://example.com/pricing&#10;[reviewer-comment] CFO: Check renewal risk before board review"
            ></textarea>
          </label>
          <ul v-if="agentSourcePackPreview.items.length" class="agent-source-pack-list">
            <li v-for="item in agentSourcePackPreview.items" :key="item.id">
              <strong>{{ item.kind }} | {{ item.label }}</strong>
              <span>{{ item.detail }}</span>
              <button type="button" @click="removeAgentSourcePackItem(item.id)">Remove</button>
            </li>
          </ul>
        </section>
        <section class="agent-playbooks" aria-label="Agent workflow playbooks">
          <header>
            <div>
              <strong>Workflow Playbooks</strong>
              <span>{{ filteredAgenticWorkflowPlaybooks.length }} of {{ agenticWorkflowPlaybooks.length }} governed starts match the current filters.</span>
            </div>
          </header>
          <section class="agent-playbook-filters" aria-label="Filter agent workflow playbooks">
            <label>
              Search
              <input v-model="agentPlaybookQuery" type="search" placeholder="board, grant, policy, Substack, LaTeX" />
            </label>
            <label>
              Focus
              <select v-model="agentPlaybookFocusFilter">
                <option v-for="focus in agentPlaybookFocusOptions" :key="focus.value" :value="focus.value">{{ focus.label }}</option>
              </select>
            </label>
            <label>
              Output target
              <select v-model="agentPlaybookTargetFilter">
                <option v-for="target in agentPlaybookTargetOptions" :key="target.value" :value="target.value">{{ target.label }}</option>
              </select>
            </label>
          </section>
          <p v-if="!filteredAgenticWorkflowPlaybooks.length" class="sidebar-hint">No playbooks match the current filters.</p>
          <div class="agent-playbook-grid">
            <article v-for="playbook in filteredAgenticWorkflowPlaybooks" :key="playbook.id">
              <header>
                <div>
                  <strong>{{ playbook.label }}</strong>
                  <span>{{ playbook.summary }}</span>
                </div>
                <button type="button" @click="applyAgentWorkflowPlaybook(playbook)">Use</button>
              </header>
              <p class="agent-playbook-meta">
                {{ agentPlaybookFocusLabel(playbook) }} | {{ agentPlaybookTargets(playbook).map((target) => target.toUpperCase()).join(", ") || "No fixed export target" }}
              </p>
              <dl>
                <div>
                  <dt>Best for</dt>
                  <dd>{{ playbook.bestFor.join(", ") }}</dd>
                </div>
                <div>
                  <dt>Outputs</dt>
                  <dd>{{ playbook.expectedOutputs.join(", ") }}</dd>
                </div>
              </dl>
            </article>
          </div>
        </section>
        <div class="agent-workspace-actions">
          <button type="submit">Plan agent workflow</button>
          <button type="button" :disabled="!agentPlan" @click="generateAgentWorkspaceRun">Generate agent packet</button>
          <button type="button" :disabled="!agentRun" @click="applyAgentWorkspaceRun">Apply agent output</button>
          <button type="button" :disabled="!agentRun" @click="buildAgentProviderPackage">Build provider request</button>
          <button type="button" :disabled="!agentProviderPackage" @click="copyAgentProviderPackage">Copy provider package</button>
          <button type="button" :disabled="!agentProviderPackage" @click="copyAgentProviderSourcePack">Copy source pack</button>
          <button type="button" :disabled="!canRunAgentProvider" @click="runAgentProviderRequest">
            {{ agentProviderBusy ? "Running provider..." : "Run provider request" }}
          </button>
          <button type="button" :disabled="!agentPlan" @click="hydrateDocsLiveFromAgentPlan">Send to Docs Live</button>
          <button type="button" :disabled="!agentPlan" @click="runAgentPlanReview">Review readiness</button>
          <button type="button" :disabled="!agentPlan" @click="runAgentPlanDistribution">Distribution prep</button>
        </div>
        <section v-if="agentPlan" class="agent-plan" aria-label="Agent workflow plan">
          <header>
            <div>
              <strong>{{ agentPlan.title }}</strong>
              <span>{{ agentPlan.documentType }} | {{ agentPlan.lanes.join(" -> ") }}</span>
            </div>
            <small>{{ agentPlan.steps.length }} steps</small>
          </header>
          <section class="agent-plan-grid">
            <article class="agent-context-score" :data-status="agentPlan.contextCompleteness.status">
              <h3>Context completeness</h3>
              <strong>{{ agentPlan.contextCompleteness.score }}/100 {{ agentPlan.contextCompleteness.status }}</strong>
              <p>Present: {{ agentPlan.contextCompleteness.present.join(", ") || "none" }}</p>
              <p>Missing: {{ agentPlan.contextCompleteness.missing.join(", ") || "none" }}</p>
              <ul>
                <li v-for="item in agentPlan.contextCompleteness.recommendations" :key="item">{{ item }}</li>
              </ul>
            </article>
            <article>
              <h3>Context pack</h3>
              <pre>{{ agentPlan.context }}</pre>
            </article>
            <article>
              <h3>Placeholders</h3>
              <pre>{{ agentPlan.placeholderText }}</pre>
            </article>
            <article class="agent-plan-source-pack">
              <h3>Source pack</h3>
              <p>{{ agentPlan.sourcePack.items.length }} managed source items</p>
              <ul>
                <li v-for="item in agentPlan.sourcePack.items.slice(0, 6)" :key="item.id">{{ item.kind }}: {{ item.label }}</li>
              </ul>
            </article>
            <article class="agent-quality-gates">
              <h3>Quality gates</h3>
              <p>{{ agentPlan.qualityGates.length }} document-type gates</p>
              <ul>
                <li v-for="gate in agentPlan.qualityGates" :key="gate.id">{{ gate.label }}</li>
              </ul>
            </article>
            <article>
              <h3>Suggested outline</h3>
              <pre>{{ agentPlan.suggestedOutline }}</pre>
            </article>
            <article>
              <h3>Revision instruction</h3>
              <p>{{ agentPlan.revisionInstruction }}</p>
            </article>
            <article v-if="agentPlan.revisionModes.length" class="agent-revision-modes">
              <h3>Revision passes</h3>
              <ul>
                <li v-for="mode in agentPlan.revisionModes" :key="mode">{{ mode }}</li>
              </ul>
            </article>
          </section>
          <section v-if="agentPlan.missingInputs.length" class="agent-missing-inputs" aria-label="Agent missing inputs">
            <strong>Missing inputs</strong>
            <ul>
              <li v-for="input in agentPlan.missingInputs" :key="input">{{ input }}</li>
            </ul>
            <button type="button" @click="buildAgentWorkspacePlan">Replan with answers</button>
          </section>
          <ol class="agent-step-list" aria-label="Agent workflow steps">
            <li v-for="step in agentPlan.steps" :key="step.id" :data-lane="step.lane">
              <div>
                <small>{{ step.lane }} | {{ step.status }}</small>
                <strong>{{ step.title }}</strong>
                <p>{{ step.detail }}</p>
              </div>
              <button type="button" @click="runAgenticStep(step)">Run step</button>
            </li>
          </ol>
          <section v-if="agentRun" class="agent-run-output" aria-label="Agent generated output">
            <header>
              <div>
                <strong>{{ agentRun.summary }}</strong>
                <span>Apply mode: {{ agentRun.applicationMode }}</span>
              </div>
              <small>{{ agentRun.blockers.length }} blockers</small>
              <div class="agent-run-packet-actions">
                <button type="button" @click="appendAgentWorkspacePacket">Append packet</button>
                <button type="button" @click="copyAgentWorkspacePacket">Copy packet</button>
              </div>
            </header>
            <section class="agent-control-center" :data-status="agentRun.controlCenter.status" aria-label="AI control center">
              <header>
                <div>
                  <strong>AI Control Center</strong>
                  <span>{{ agentRun.controlCenter.summary }}</span>
                </div>
                <small>{{ agentRun.controlCenter.readinessScore }}/100 readiness</small>
              </header>
              <section class="agent-control-grid">
                <article>
                  <h3>Next actions</h3>
                  <ul>
                    <li v-for="action in agentRun.controlCenter.nextActions" :key="`${action.lane}-${action.label}`">
                      <strong>{{ action.label }}</strong>
                      <span>{{ action.lane }} | {{ action.status }}</span>
                      <p>{{ action.detail }}</p>
                    </li>
                  </ul>
                </article>
                <article>
                  <h3>Source grounding</h3>
                  <ul>
                    <li v-for="item in agentRun.controlCenter.sourceGrounding" :key="item.label" :data-status="item.status">
                      <strong>{{ item.label }}</strong>
                      <span>{{ item.status }}</span>
                      <p>{{ item.detail }}</p>
                    </li>
                  </ul>
                </article>
                <article>
                  <h3>Governance</h3>
                  <ul>
                    <li v-for="item in agentRun.controlCenter.governance" :key="item.label" :data-status="item.status">
                      <strong>{{ item.label }}</strong>
                      <span>{{ item.status }}</span>
                      <p>{{ item.detail }}</p>
                    </li>
                  </ul>
                </article>
                <article>
                  <h3>Distribution state</h3>
                  <ul>
                    <li v-for="item in agentRun.controlCenter.distribution" :key="item.label" :data-status="item.status">
                      <strong>{{ item.label }}</strong>
                      <span>{{ item.status }}</span>
                      <p>{{ item.detail }}</p>
                    </li>
                  </ul>
                </article>
              </section>
            </section>
            <section
              v-if="agentRun.documentEvidence.reviewCommentResolutions.length"
              class="agent-review-comment-queue"
              aria-label="Review comment resolution queue"
            >
              <header>
                <div>
                  <strong>Review Comment Resolution Queue</strong>
                  <span>Turn unresolved comments into reviewer-owned decisions with notes before release.</span>
                </div>
                <small>{{ agentRun.documentEvidence.reviewCommentResolutions.length }} unresolved</small>
              </header>
              <ol>
                <li
                  v-for="comment in agentRun.documentEvidence.reviewCommentResolutions"
                  :key="comment.id"
                  :data-blocker="comment.blocker"
                  :data-status="agentReviewCommentState(comment)?.status || 'queued'"
                >
                  <div>
                    <small>
                      Line {{ comment.line }} | {{ comment.author }} | {{ agentReviewCommentState(comment)?.status || "queued" }}
                    </small>
                    <strong>{{ comment.excerpt }}</strong>
                    <p>{{ comment.requiredAction }}</p>
                    <p v-if="agentReviewCommentState(comment)?.note" class="sidebar-hint">
                      Resolution note: {{ agentReviewCommentState(comment)?.note }}
                    </p>
                  </div>
                  <ul>
                    <li v-for="option in comment.resolutionOptions" :key="option">{{ option }}</li>
                  </ul>
                  <div class="agent-lifecycle-actions">
                    <button type="button" @click="setAgentReviewCommentStatus(comment, 'in-progress')">Start</button>
                    <button type="button" @click="setAgentReviewCommentStatus(comment, 'needs-review')">Carry forward</button>
                    <button type="button" @click="setAgentReviewCommentStatus(comment, 'complete')">Resolve</button>
                  </div>
                  <label>
                    Resolution note
                    <input
                      :value="agentReviewCommentState(comment)?.note || ''"
                      placeholder="Decision, source, owner, date, or carry-forward reason"
                      @change="setAgentReviewCommentNote(comment, inputValue($event))"
                    />
                  </label>
                </li>
              </ol>
            </section>
            <section v-if="agentRun.editAcceptanceQueue.length" class="agent-edit-acceptance-queue" aria-label="Agent edit acceptance queue">
              <header>
                <div>
                  <strong>Edit Acceptance Queue</strong>
                  <span>Review generated edits one item at a time before applying accepted changes.</span>
                </div>
                <small>{{ acceptedAgentEditCount }} accepted of {{ agentRun.editAcceptanceQueue.length }}</small>
              </header>
              <ol>
                <li v-for="row in agentEditAcceptanceRows" :key="row.item.id" :data-scope="row.item.scope" :data-status="row.state.status">
                  <div>
                    <small>{{ row.item.scope }} | {{ row.state.status }}</small>
                    <strong>{{ row.item.heading }}</strong>
                    <p>{{ row.item.recommendation }}</p>
                    <p v-if="row.state.note" class="sidebar-hint">Acceptance note: {{ row.state.note }}</p>
                  </div>
                  <section class="agent-edit-acceptance-compare">
                    <article>
                      <h3>Original</h3>
                      <pre>{{ row.item.originalText }}</pre>
                    </article>
                    <article>
                      <h3>Proposed</h3>
                      <pre>{{ row.item.proposedText }}</pre>
                    </article>
                  </section>
                  <div>
                    <h3>Risk notes</h3>
                    <ul>
                      <li v-for="note in row.item.riskNotes" :key="note">{{ note }}</li>
                    </ul>
                  </div>
                  <div class="agent-lifecycle-actions">
                    <button type="button" @click="setAgentEditAcceptanceStatus(row.item, 'accepted')">Accept</button>
                    <button type="button" @click="setAgentEditAcceptanceStatus(row.item, 'rejected')">Reject</button>
                    <button type="button" @click="reviseAgentAcceptanceItem(row.item)">Revise</button>
                  </div>
                  <label>
                    Acceptance note
                    <input
                      :value="row.state.note || ''"
                      placeholder="Reason accepted, rejected, or sent for another pass"
                      @change="setAgentEditAcceptanceNote(row.item, inputValue($event))"
                    />
                  </label>
                </li>
              </ol>
              <button type="button" :disabled="acceptedAgentEditCount === 0" @click="applyAcceptedAgentEdits">Apply accepted edits</button>
            </section>
            <section class="agent-lifecycle-board" aria-label="Agent lifecycle task board">
              <header>
                <div>
                  <strong>Lifecycle Task Board</strong>
                  <span>Operational tasks for creating, composing, editing, revising, reviewing, and distributing the document.</span>
                </div>
                <small>{{ agentLifecycleTaskRows.length }} of {{ agentLifecycleTaskTotal }} tasks</small>
              </header>
              <section class="agent-lifecycle-filters" aria-label="Filter agent lifecycle tasks">
                <label>
                  Lane
                  <select v-model="agentTaskLaneFilter">
                    <option v-for="lane in agentTaskLaneOptions" :key="lane" :value="lane">{{ lane === "all" ? "All lanes" : lane }}</option>
                  </select>
                </label>
                <label>
                  Status
                  <select v-model="agentTaskStatusFilter">
                    <option v-for="status in agentTaskStatusOptions" :key="status" :value="status">{{ status === "all" ? "All statuses" : status }}</option>
                  </select>
                </label>
                <label>
                  Owner
                  <select v-model="agentTaskOwnerFilter">
                    <option v-for="owner in agentTaskOwnerOptions" :key="owner" :value="owner">{{ owner === "all" ? "All owners" : owner }}</option>
                  </select>
                </label>
                <label>
                  Section
                  <select v-model="agentTaskSectionFilter">
                    <option v-for="section in agentTaskSectionOptions" :key="section.value" :value="section.value">{{ section.label }}</option>
                  </select>
                </label>
                <label>
                  Target
                  <select v-model="agentTaskTargetFilter">
                    <option v-for="target in agentTaskTargetOptions" :key="target" :value="target">{{ target === "all" ? "All targets" : target }}</option>
                  </select>
                </label>
                <label>
                  Evidence
                  <select v-model="agentTaskEvidenceFilter">
                    <option value="all">All evidence states</option>
                    <option value="has-evidence">Has evidence</option>
                    <option value="missing-evidence">Missing evidence</option>
                    <option value="release-blocker">Release blockers</option>
                  </select>
                </label>
                <label>
                  Search tasks
                  <input v-model="agentTaskQuery" placeholder="search title, note, evidence, or next step" />
                </label>
              </section>
              <p v-if="!agentLifecycleTaskRows.length" class="sidebar-hint">No lifecycle tasks match the current filters.</p>
              <ol v-else>
                <li v-for="row in agentLifecycleTaskRows" :key="row.task.id" :data-lane="row.task.lane" :data-status="row.state.status">
                  <div>
                    <small>{{ row.task.lane }} | {{ row.state.status }} | {{ row.task.owner }}</small>
                    <strong>{{ row.task.title }}</strong>
                    <p>{{ row.task.nextStep }}</p>
                    <p v-if="row.state.note" class="sidebar-hint">Execution note: {{ row.state.note }}</p>
                    <div class="agent-lifecycle-actions">
                      <button type="button" @click="runAgentLifecycleTask(row.task)">Run task</button>
                      <button type="button" @click="setAgentLifecycleTaskStatus(row.task, 'in-progress')">Start</button>
                      <button type="button" @click="setAgentLifecycleTaskStatus(row.task, 'needs-review')">Needs review</button>
                      <button type="button" @click="setAgentLifecycleTaskStatus(row.task, 'complete')">Complete</button>
                      <button type="button" @click="insertAgentLifecycleTaskBrief(row.task)">Insert brief</button>
                      <button type="button" @click="copyAgentLifecycleTaskBrief(row.task)">Copy brief</button>
                    </div>
                    <label>
                      Task note
                      <input
                        :value="row.state.note || ''"
                        placeholder="Evidence, blocker, reviewer, or completion note"
                        @change="setAgentLifecycleTaskNote(row.task, inputValue($event))"
                      />
                    </label>
                  </div>
                  <ul>
                    <li v-for="item in row.task.evidence" :key="item">{{ item }}</li>
                  </ul>
                </li>
              </ol>
            </section>
            <section class="agent-reviewer-agents" aria-label="Agent reviewer agents">
              <header>
                <div>
                  <strong>Review Agents</strong>
                  <span>Specialized agent checks for editorial quality, evidence, risk, citations, governance, and export readiness.</span>
                </div>
                <small>{{ agentRun.reviewerAgents.length }} reviewers</small>
              </header>
              <section class="agent-reviewer-grid">
                <article v-for="reviewer in agentRun.reviewerAgents" :key="reviewer.id" :data-status="reviewer.status">
                  <header>
                    <div>
                      <strong>{{ reviewer.label }}</strong>
                      <span>{{ reviewer.status }}</span>
                    </div>
                  </header>
                  <p>{{ reviewer.mandate }}</p>
                  <div>
                    <h3>Findings</h3>
                    <ul>
                      <li v-for="item in reviewer.findings" :key="item">{{ item }}</li>
                    </ul>
                  </div>
                  <div>
                    <h3>Required actions</h3>
                    <ul>
                      <li v-for="item in reviewer.requiredActions" :key="item">{{ item }}</li>
                    </ul>
                  </div>
                </article>
              </section>
            </section>
            <section class="agent-section-workqueue" aria-label="Agent section work queue">
              <header>
                <div>
                  <strong>Section Work Queue</strong>
                  <span>Draft and review the document section by section with assigned reviewer agents.</span>
                </div>
                <small>{{ agentRun.sectionWorkQueue.length }} sections</small>
              </header>
              <ol>
                <li v-for="section in agentRun.sectionWorkQueue" :key="section.id">
                  <div>
                    <small>Level {{ section.level }} | {{ section.lane }} | {{ section.draftingDepth }} depth</small>
                    <strong>{{ section.heading }}</strong>
                    <label class="agent-section-depth">
                      Depth
                      <select v-model="section.draftingDepth">
                        <option v-for="depth in agentSectionDraftingDepthOptions" :key="depth.value" :value="depth.value">{{ depth.label }}</option>
                      </select>
                    </label>
                    <p>{{ section.draftingInstruction }}</p>
                    <span>Reviewers: {{ section.reviewerAgentIds.join(", ") }}</span>
                    <div class="agent-section-actions">
                      <button type="button" @click="insertAgentSectionBrief(section)">Insert brief</button>
                      <button type="button" @click="draftAgentSectionWithDocsLive(section)">Draft in Docs Live</button>
                    </div>
                  </div>
                  <ul>
                    <li v-for="item in section.completionCriteria" :key="item">{{ item }}</li>
                  </ul>
                </li>
              </ol>
            </section>
            <section class="agent-audit-trail" aria-label="Agent audit trail">
              <header>
                <div>
                  <strong>Agent Audit Trail</strong>
                  <span>{{ agentRun.auditTrail.runId }}</span>
                </div>
                <small>{{ agentRun.auditTrail.plannerVersion }}</small>
              </header>
              <section class="agent-audit-grid">
                <article>
                  <h3>Fingerprints</h3>
                  <dl>
                    <div>
                      <dt>Instruction</dt>
                      <dd>{{ agentRun.auditTrail.instructionFingerprint }}</dd>
                    </div>
                    <div>
                      <dt>Context</dt>
                      <dd>{{ agentRun.auditTrail.contextFingerprint }}</dd>
                    </div>
                    <div>
                      <dt>Source</dt>
                      <dd>{{ agentRun.auditTrail.sourceFingerprint }}</dd>
                    </div>
                    <div>
                      <dt>Output</dt>
                      <dd>{{ agentRun.auditTrail.outputFingerprint }}</dd>
                    </div>
                  </dl>
                </article>
                <article>
                  <h3>Rollback plan</h3>
                  <ul>
                    <li v-for="item in agentRun.auditTrail.rollbackPlan" :key="item">{{ item }}</li>
                  </ul>
                </article>
                <article>
                  <h3>Review events</h3>
                  <ul>
                    <li v-for="item in agentRun.auditTrail.reviewEvents" :key="item">{{ item }}</li>
                  </ul>
                </article>
              </section>
            </section>
            <section class="agent-release-evidence" aria-label="Agent release evidence bundle">
              <header>
                <div>
                  <strong>Release Evidence Bundle</strong>
                  <span>{{ agentRun.releaseEvidenceBundle.summary }}</span>
                </div>
                <small>{{ agentRun.releaseEvidenceBundle.blockers.length }} blockers</small>
                <div class="agent-release-evidence-actions">
                  <button type="button" @click="insertAgentReleaseEvidenceAuditPackage">Insert audit package</button>
                  <button type="button" @click="copyAgentReleaseEvidenceAuditPackage">Copy audit package</button>
                </div>
              </header>
              <section class="agent-release-evidence-grid">
                <article
                  v-for="item in agentRun.releaseEvidenceBundle.items"
                  :key="item.label"
                  :data-status="item.status"
                >
                  <small>{{ item.owner }} | {{ item.requiredBeforeRelease ? "required" : "optional" }}</small>
                  <strong>{{ item.label }}</strong>
                  <p>{{ item.detail }}</p>
                </article>
              </section>
            </section>
            <section v-if="agentRun.blockers.length" class="agent-missing-inputs" aria-label="Agent run blockers">
              <strong>Resolve before final release</strong>
              <ul>
                <li v-for="blocker in agentRun.blockers" :key="blocker">{{ blocker }}</li>
              </ul>
            </section>
            <section class="agent-run-columns">
              <article>
                <h3>QA gates</h3>
                <ul>
                  <li v-for="item in agentRun.reviewChecklist" :key="item">{{ item }}</li>
                </ul>
              </article>
              <article>
                <h3>Distribution gates</h3>
                <ul>
                  <li v-for="item in agentRun.distributionChecklist" :key="item">{{ item }}</li>
                </ul>
              </article>
            </section>
            <section v-if="agentRun.distributionTargetPlans.length" class="agent-distribution-runbooks" aria-label="Agent distribution target runbooks">
              <article v-for="targetPlan in agentRun.distributionTargetPlans" :key="targetPlan.target">
                <header>
                  <strong>{{ targetPlan.label }}</strong>
                  <span>{{ targetPlan.purpose }}</span>
                </header>
                <div>
                  <h3>Preflight</h3>
                  <ul>
                    <li v-for="item in targetPlan.preflightChecks" :key="item">{{ item }}</li>
                  </ul>
                </div>
                <div>
                  <h3>Handoff</h3>
                  <ul>
                    <li v-for="item in targetPlan.handoffSteps" :key="item">{{ item }}</li>
                  </ul>
                </div>
                <div>
                  <h3>Evidence</h3>
                  <ul>
                    <li v-for="item in targetPlan.evidenceRequired" :key="item">{{ item }}</li>
                  </ul>
                </div>
              </article>
            </section>
            <textarea :value="agentRun.markdown" rows="12" readonly aria-label="Agent generated Markdown"></textarea>
          </section>
          <section v-if="store.agentRunHistory.length" class="agent-history" aria-label="Agent run history">
            <header>
              <div>
                <strong>Agent Run History</strong>
                <span>Local audit records for generated and applied agent work.</span>
              </div>
              <small>{{ filteredAgentRunHistory.length }} of {{ store.agentRunHistory.length }} saved</small>
              <div class="agent-history-audit-actions">
                <button type="button" :disabled="!filteredAgentRunHistory.length" @click="insertAgentHistoryAudit">Insert audit</button>
                <button type="button" :disabled="!filteredAgentRunHistory.length" @click="copyAgentHistoryAudit">Copy audit</button>
                <button type="button" @click="clearAgentHistory">Clear history</button>
              </div>
            </header>
            <section class="agent-history-filters" aria-label="Filter agent run history">
              <label>
                Search
                <input v-model="agentHistoryQuery" type="search" placeholder="Instruction, evidence, provider, blocker" />
              </label>
              <label>
                Status
                <select v-model="agentHistoryStatusFilter">
                  <option value="all">All statuses</option>
                  <option value="generated">Generated</option>
                  <option value="applied">Applied</option>
                  <option value="provider-applied">Provider applied</option>
                </select>
              </label>
              <label>
                Lane
                <select v-model="agentHistoryLaneFilter">
                  <option v-for="lane in agentTaskLaneOptions" :key="lane" :value="lane">
                    {{ lane === "all" ? "All lanes" : lane }}
                  </option>
                </select>
              </label>
              <label>
                Target
                <select v-model="agentHistoryTargetFilter">
                  <option value="all">All targets</option>
                  <option v-for="option in agentPlaybookTargetOptions.filter((item) => item.value !== 'all')" :key="option.value" :value="option.value">
                    {{ option.label }}
                  </option>
                </select>
              </label>
            </section>
            <p v-if="!filteredAgentRunHistory.length" class="sidebar-hint">No agent runs match the current history filters.</p>
            <ol>
              <li v-for="item in filteredAgentRunHistory.slice(0, 12)" :key="item.runId">
                <div>
                  <strong>{{ item.title }}</strong>
                  <span>{{ item.status }} | {{ item.applicationMode }} | {{ item.readinessScore }}/100</span>
                  <small>{{ item.runId }} | {{ item.updatedAt }}</small>
                  <p v-if="item.packetPreview">{{ item.packetPreview }}</p>
                  <p v-if="item.controlCenter">Control: {{ item.controlCenter.status }} | {{ item.controlCenter.summary }}</p>
                  <p v-if="item.documentEvidence">Evidence: {{ agentRunHistoryEvidenceSummary(item) }}</p>
                  <p v-if="item.outlineCritique?.length">Outline: {{ agentRunHistoryOutlineSummary(item) }}</p>
                  <p v-if="item.sourcePack">Source pack: {{ agentRunHistorySourcePackSummary(item) }}</p>
                  <p v-if="item.lifecycleTaskStates?.length">Task states: {{ agentRunHistoryTaskStateSummary(item) }}</p>
                  <div class="agent-history-actions">
                    <button type="button" @click="replanAgentHistoryRun(item)">Replan</button>
                    <button type="button" :disabled="!item.packetMarkdown" @click="appendAgentHistoryPacket(item)">Append packet</button>
                    <button type="button" :disabled="!item.packetMarkdown" @click="copyAgentHistoryPacket(item)">Copy packet</button>
                    <button type="button" @click="removeAgentHistoryRun(item)">Remove</button>
                  </div>
                </div>
                <dl>
                  <div>
                    <dt>Output</dt>
                    <dd>{{ item.outputFingerprint }}</dd>
                  </div>
                  <div>
                    <dt>Source</dt>
                    <dd>{{ item.sourceFingerprint }}</dd>
                  </div>
                  <div>
                    <dt>Provider</dt>
                    <dd>{{ item.providerProfile || "local planner" }}</dd>
                  </div>
                  <div>
                    <dt>Sections</dt>
                    <dd>{{ item.sectionCount || 0 }}</dd>
                  </div>
                  <div>
                    <dt>Reviewers</dt>
                    <dd>{{ item.reviewerCount || 0 }}</dd>
                  </div>
                  <div>
                    <dt>Tasks</dt>
                    <dd>{{ item.taskCount || 0 }}</dd>
                  </div>
                </dl>
              </li>
            </ol>
          </section>
          <section class="agent-provider-panel" aria-label="AI provider handoff">
            <header>
              <div>
                <strong>Provider handoff</strong>
                <span>Generate a redacted request package for an approved AI provider or local model gateway.</span>
              </div>
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
                <input v-model="agentProviderModel" placeholder="Approved model or deployment name" />
              </label>
              <label>
                Endpoint
                <input v-model="agentProviderEndpoint" placeholder="https://provider.example/v1/messages" />
              </label>
              <label>
                API key environment variable
                <input v-model="agentProviderKeyEnv" placeholder="NEDITOR_AI_API_KEY" />
              </label>
              <label>
                Session API key
                <input v-model="agentProviderApiKey" type="password" autocomplete="off" placeholder="Used once, never saved" />
              </label>
            </section>
            <section v-if="agentProviderPackage" class="agent-provider-output" aria-label="AI provider request package">
              <header>
                <div>
                  <strong>{{ agentProviderPackage.profile.label }}</strong>
                  <span>{{ agentProviderPackage.profile.summary }}</span>
                </div>
              </header>
              <ul>
                <li v-for="item in agentProviderPackage.checklist" :key="item">{{ item }}</li>
              </ul>
              <label>
                Source evidence pack
                <textarea :value="agentProviderSourcePackMarkdown" rows="8" readonly aria-label="AI provider source evidence pack"></textarea>
              </label>
              <textarea :value="agentProviderPackage.markdown" rows="12" readonly aria-label="AI provider request Markdown"></textarea>
            </section>
            <section v-if="agentProviderResult" class="agent-provider-output" aria-label="AI provider response">
              <header>
                <div>
                  <strong>Provider response</strong>
                  <span>{{ agentProviderResult.status }} {{ agentProviderResult.statusText }} | Apply wraps this output in needs-review provenance.</span>
                </div>
                <button type="button" @click="applyAgentProviderResponse">Apply response</button>
              </header>
              <textarea :value="agentProviderResult.markdown" rows="12" readonly aria-label="AI provider response Markdown"></textarea>
            </section>
          </section>
        </section>
      </form>
    </section>

    <section
      v-if="guidedDemoOpen"
      ref="guidedDemoDialog"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="NEditor guided demo"
      tabindex="-1"
      @keydown="handleModalKeydown('guided-demo', $event)"
    >
      <div class="modal guided-demo-modal">
        <header>
          <div>
            <h2>NEditor Guided Demo</h2>
            <p>{{ currentDemoStep?.summary }}</p>
            <small>{{ guidedDemoCompletionSummary }}</small>
          </div>
          <button type="button" aria-label="Close guided demo" @click="closeGuidedDemo">x</button>
        </header>
        <section class="guided-demo-progress" aria-label="Guided demo progress">
          <div>
            <strong>{{ guidedDemoCompletedCount }} of {{ guidedDemoSteps.length }} completed</strong>
            <span>{{ guidedDemoCompletionPercent }}%</span>
          </div>
          <progress :value="guidedDemoCompletedCount" :max="guidedDemoSteps.length">{{ guidedDemoCompletionPercent }}%</progress>
        </section>
        <section class="guided-demo-layout">
          <ol class="guided-demo-steps" aria-label="Guided demo steps">
            <li
              v-for="(step, index) in guidedDemoSteps"
              :key="step.id"
              :class="{ active: index === guidedDemoStepIndex, complete: guidedDemoStepIsComplete(step.id) }"
            >
              <button type="button" @click="selectGuidedDemoStep(index)">
                <span>{{ index + 1 }}</span>
                <strong>{{ step.title }}</strong>
                <small>{{ guidedDemoStepIsComplete(step.id) ? "Done" : "Open" }}</small>
              </button>
            </li>
          </ol>
          <article v-if="currentDemoStep" class="guided-demo-card" aria-live="polite">
            <small>{{ currentDemoStep.mode }}</small>
            <h3>{{ currentDemoStep.title }}</h3>
            <p>{{ currentDemoStep.detail }}</p>
            <ul>
              <li v-for="point in currentDemoStep.points" :key="point">{{ point }}</li>
            </ul>
            <div class="guided-demo-actions">
              <button type="button" :disabled="guidedDemoStepIndex === 0" @click="previousGuidedDemoStep">Previous</button>
              <button type="button" @click="runGuidedDemoStep(currentDemoStep)">Try this step</button>
              <button type="button" @click="markGuidedDemoStepComplete(currentDemoStep.id)">Mark done</button>
              <button type="button" :disabled="guidedDemoStepIndex === guidedDemoSteps.length - 1" @click="nextGuidedDemoStep">Next</button>
            </div>
            <div class="guided-demo-evidence-actions">
              <button type="button" @click="insertGuidedDemoChecklist">Insert checklist</button>
              <button type="button" @click="copyGuidedDemoChecklist">Copy checklist</button>
              <button type="button" @click="resetGuidedDemoProgress">Reset progress</button>
            </div>
          </article>
        </section>
      </div>
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
        <input
          v-model="commandQuery"
          autofocus
          data-initial-focus
          aria-label="Search commands, headings, citations, glossary, index terms, or enter an AI instruction"
          placeholder="Search commands or tell NEditor what to do"
          @keydown.enter.prevent="runCommandPaletteAgentInstruction"
        />
        <section v-if="commandAgentInstructionAvailable" class="command-agent-route" aria-label="AI command route">
          <div>
            <strong>Generate with AI agent</strong>
            <span>Plan the workflow, create a governed packet, and keep it ready for review or distribution.</span>
            <dl v-if="commandAgentPlanPreview" class="command-agent-preview" aria-label="AI command plan preview">
              <div>
                <dt>Lanes</dt>
                <dd>{{ commandAgentPlanPreview.lanes.join(", ") }}</dd>
              </div>
              <div>
                <dt>Targets</dt>
                <dd>{{ commandAgentPlanPreview.distributionTargets.length ? commandAgentPlanPreview.distributionTargets.join(", ") : "Review packet" }}</dd>
              </div>
              <div>
                <dt>Missing</dt>
                <dd>{{ commandAgentPlanPreview.missingInputs.length ? commandAgentPlanPreview.missingInputs.slice(0, 4).join(", ") : "Ready to draft" }}</dd>
              </div>
            </dl>
          </div>
          <div class="command-agent-actions">
            <button type="button" @click="openCommandPaletteAgentPlan">Plan first</button>
            <button type="button" @click="runCommandPaletteAgentInstruction">Generate Packet</button>
          </div>
        </section>
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
import { EditorSelection, EditorState, RangeSetBuilder } from "@codemirror/state";
import { Decoration, EditorView, keymap, lineNumbers, ViewPlugin, type DecorationSet, type ViewUpdate } from "@codemirror/view";
import { addCursorAbove, addCursorBelow, defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { codeFolding, foldAll, foldGutter, foldKeymap, unfoldAll } from "@codemirror/language";
import { markdown } from "@codemirror/lang-markdown";
import { findNext, findPrevious, openSearchPanel, replaceAll, replaceNext, searchKeymap, selectNextOccurrence } from "@codemirror/search";
import { closeBrackets, closeBracketsKeymap, insertBracket } from "@codemirror/autocomplete";
import { forceLinting, linter, lintGutter, type Diagnostic as CodeMirrorDiagnostic } from "@codemirror/lint";
import {
  aiProviderProfiles,
  buildAiProviderRequestPackage,
  buildAiProviderResponseReviewMarkdown,
  executeAiProviderRequestPackage,
  formatAiProviderSourcePack,
  type AiProviderExecutionResult,
  type AiProviderProfileId,
  type AiProviderRequestPackage,
  type AiProviderSourcePack,
} from "./lib/aiProviderPackages";
import { inspectAiRuntimeReadiness, type AiRuntimeReadinessReport } from "./lib/aiRuntimeReadiness";
import { bibliographyEntryStub, bibliographyStubsForMissingKeys, citationReferenceSnippet } from "./lib/bibliographyManager";
import {
  agenticWorkflowPlaybooks,
  buildAgenticLifecycleTaskBrief,
  buildAgenticReleaseEvidenceAuditPackage,
  buildAgenticSectionWorkBrief,
  buildAgenticSourcePack,
  buildAgenticWorkflowPlan,
  buildAgenticWorkflowRun,
  serializeAgenticSourcePackItem,
  stableFingerprint,
  type AgenticSourcePackItemKind,
  type AgenticWorkflowPlaybook,
  type AgenticWorkflowLane,
  type AgenticWorkflowPlan,
  type AgenticWorkflowRun,
  type AgenticLifecycleTask,
  type AgenticEditAcceptanceItem,
  type AgenticReviewCommentResolution,
  type AgenticSectionWorkItem,
  type AgenticWorkflowStep,
} from "./lib/agenticWorkflows";
import { buildConflictDiff, type ConflictDiffRow } from "./lib/conflict";
import {
  citationTodoAuditMarkdown,
  citationTodoComment,
  deferCitationTodo,
  extractCitationTodoItems,
  resolveCitationTodo,
  type CitationTodoItem,
} from "./lib/citationTodoWorkflow";
import { createDebouncedTextCommit } from "./lib/debounce";
import {
  buildDocsLiveDraft,
  buildDocsLiveQuestionnaire,
  docsLivePlaceholderEntries,
  docsLiveDocumentTypes,
  removeDocsLivePlaceholder,
  upsertDocsLivePlaceholder,
  type DocsLiveDocumentType,
  type DocsLiveDraft,
  type DocsLiveDraftDepth,
  type DocsLivePlaceholderEntry,
  type DocsLivePlaceholderKind,
  type DocsLivePlaceholderReviewStatus,
} from "./lib/docsLive";
import { outlinePlanFromMarkdown, outlinePlanToMarkdown, parseOutlinePlan } from "./lib/documentOutline";
import { markdownListContinuation } from "./lib/markdownEditing";
import { replaceOrAppendMarkdownSection } from "./lib/markdownSectionMerge";
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
import {
  SUPPORTED_CITATION_STYLES,
  type AgentEditAcceptanceState,
  type AgentEditAcceptanceStatus,
  type AgentLifecycleExecutionStatus,
  type AgentLifecycleTaskState,
  type AgentRunHistoryItem,
  type DocsLiveDraftHistoryItem,
} from "./lib/workspacePersistence";
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
type DesktopWorkflowTestHooks = {
  activeDocumentPath(): string | null;
  activeDocumentText(): string;
  activeDocumentTitle(): string;
};

declare global {
  interface Window {
    __NEDITOR_DESKTOP_WORKFLOW__?: DesktopWorkflowTestHooks;
  }
}

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
const docsLiveDialog = ref<HTMLElement | null>(null);
const agentWorkspaceDialog = ref<HTMLElement | null>(null);
const guidedDemoDialog = ref<HTMLElement | null>(null);
const commandPaletteDialog = ref<HTMLElement | null>(null);
const conflictDialog = ref<HTMLElement | null>(null);
let editorView: EditorView | null = null;
let syncingEditorFromStore = false;
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
const agentWorkspaceOpen = ref(false);
const agentInstruction = ref("");
const agentContextAnswers = ref("");
const agentSourcePackText = ref("");
const agentSourcePackKind = ref<AgenticSourcePackItemKind>("note");
const agentSourcePackLabel = ref("");
const agentSourcePackDetail = ref("");
const agentPlaybookQuery = ref("");
const agentPlaybookFocusFilter = ref("all");
const agentPlaybookTargetFilter = ref<"all" | ExportTarget>("all");
const agentPlan = ref<AgenticWorkflowPlan | null>(null);
const agentRun = ref<AgenticWorkflowRun | null>(null);
const agentLifecycleTaskStates = ref<Record<string, AgentLifecycleTaskState>>({});
const agentEditAcceptanceStates = ref<Record<string, AgentEditAcceptanceState>>({});
const defaultAgentProviderProfile = aiProviderProfiles[0];
const agentProviderId = ref<AiProviderProfileId>("manual-review");
const agentProviderEndpoint = ref(defaultAgentProviderProfile.endpoint);
const agentProviderModel = ref(defaultAgentProviderProfile.model);
const agentProviderKeyEnv = ref("NEDITOR_AI_API_KEY");
const agentProviderPackage = ref<AiProviderRequestPackage | null>(null);
const agentProviderApiKey = ref("");
const agentProviderBusy = ref(false);
const agentProviderResult = ref<AiProviderExecutionResult | null>(null);
const agentTaskLaneFilter = ref<"all" | AgenticWorkflowLane>("all");
const agentTaskStatusFilter = ref<"all" | AgentLifecycleExecutionStatus>("all");
const agentTaskOwnerFilter = ref("all");
const agentTaskSectionFilter = ref("all");
const agentTaskTargetFilter = ref("all");
const agentTaskEvidenceFilter = ref<"all" | "has-evidence" | "missing-evidence" | "release-blocker">("all");
const agentTaskQuery = ref("");
const agentHistoryQuery = ref("");
const agentHistoryStatusFilter = ref<"all" | AgentRunHistoryItem["status"]>("all");
const agentHistoryLaneFilter = ref<"all" | AgenticWorkflowLane>("all");
const agentHistoryTargetFilter = ref<"all" | ExportTarget>("all");
const docsLiveOpen = ref(false);
const guidedDemoOpen = ref(false);
const guidedDemoStepIndex = ref(0);
const docsLiveDocumentType = ref<DocsLiveDocumentType>("business-brief");
const docsLiveTitle = ref("");
const docsLiveOutlineText = ref("");
const docsLiveTranscript = ref("");
const docsLiveInterimTranscript = ref("");
const docsLiveContext = ref("");
const docsLivePlaceholderText = ref("");
const docsLivePlaceholderKey = ref("");
const docsLivePlaceholderDraftValue = ref("");
const docsLivePlaceholderDraftKind = ref<DocsLivePlaceholderKind>("text");
const docsLivePlaceholderDraftSource = ref("");
const docsLivePlaceholderDraftStatus = ref<DocsLivePlaceholderReviewStatus>("provided");
const docsLivePlaceholderKindOptions: Array<{ value: DocsLivePlaceholderKind; label: string }> = [
  { value: "text", label: "Text" },
  { value: "client", label: "Client" },
  { value: "person", label: "Person" },
  { value: "reviewer", label: "Reviewer" },
  { value: "date", label: "Date" },
  { value: "money", label: "Money" },
  { value: "number", label: "Number" },
  { value: "source", label: "Source" },
  { value: "decision", label: "Decision" },
  { value: "channel", label: "Channel" },
];
const docsLivePlaceholderReviewStatusOptions: Array<{ value: DocsLivePlaceholderReviewStatus; label: string }> = [
  { value: "provided", label: "Provided" },
  { value: "needs-review", label: "Needs review" },
  { value: "verified", label: "Verified" },
];
const docsLiveIntentFields = [
  { key: "audience", label: "Audience", placeholder: "executive team, board, customers" },
  { key: "outcome", label: "Outcome", placeholder: "approve renewal, align launch plan" },
  { key: "owner", label: "Owner", placeholder: "Finance, Product, Legal" },
  { key: "deadline", label: "Deadline", placeholder: "June 1, end of Q2" },
  { key: "distribution target", label: "Distribution target", placeholder: "PDF, Google Docs, Substack" },
];
const docsLiveQuestionnaireText = ref(buildDocsLiveQuestionnaire("business-brief"));
const docsLiveQuestionnaireAnswerText = ref("");
const docsLiveGeneratedMarkdown = ref("");
const docsLiveDraft = ref<DocsLiveDraft | null>(null);
const docsLiveDraftingDepth = ref<DocsLiveDraftDepth>("standard");
const docsLiveDraftingDepthOptions: Array<{ value: DocsLiveDraftDepth; label: string }> = [
  { value: "summary", label: "Summary" },
  { value: "standard", label: "Standard" },
  { value: "detailed", label: "Detailed" },
  { value: "technical", label: "Technical" },
  { value: "legal", label: "Legal" },
  { value: "executive", label: "Executive" },
];
const agentSectionDraftingDepthOptions = docsLiveDraftingDepthOptions;
const docsLiveInsertMode = ref<"replace" | "append" | "selection" | "section">("replace");
const docsLiveTargetSection = ref<AgenticSectionWorkItem | null>(null);
const docsLiveListening = ref(false);
const docsLiveSpeechStatus = ref("Voice ready");
const docsLiveRuntimeChecking = ref(false);
const docsLiveRuntimeReport = ref<AiRuntimeReadinessReport | null>(null);
const desktopWorkflowSmokeActive = ref(false);
const commandPaletteOpen = ref(false);
const conflictOpen = ref(false);
const mergedConflictText = ref("");
const conflictMergeParts = ref<ConflictMergePart[]>([]);
const commandQuery = ref("");
const reviewCommentText = ref("");
const changeNoteText = ref("");
const citationTodoKey = ref("");
const citationTodoNote = ref("");
const selectedTableIndex = ref(0);
const outlineDraftText = ref("- Executive Summary\n  - Decision Needed\n  - Key Risks\n- Financial Case\n- Next Steps");
const outlineDraftTitle = ref("");
const outlineDraftIncludeToc = ref(true);
const outlineModeNewTitle = ref("New chapter");
const outlineModeNewLevel = ref(1);
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
const helpQuery = ref("");
const helpCategory = ref<"all" | HelpCategory>("all");
const selectedHelpTopicId = ref("getting-started");
const buttonHelp = ref({ visible: false, text: "", x: 0, y: 0, placement: "bottom" as "top" | "bottom" });

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

interface SpeechRecognitionAlternativeLike {
  transcript?: string;
}

interface SpeechRecognitionResultLike {
  isFinal?: boolean;
  length: number;
  [index: number]: SpeechRecognitionAlternativeLike;
}

interface SpeechRecognitionResultListLike {
  length: number;
  [index: number]: SpeechRecognitionResultLike;
}

interface SpeechRecognitionEventLike extends Event {
  resultIndex?: number;
  results?: SpeechRecognitionResultListLike;
}

interface SpeechRecognitionErrorEventLike extends Event {
  error?: string;
}

interface SpeechRecognitionLike {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  onresult: ((event: SpeechRecognitionEventLike) => void) | null;
  onerror: ((event: SpeechRecognitionErrorEventLike) => void) | null;
  onend: (() => void) | null;
  start: () => void;
  stop: () => void;
  abort?: () => void;
}

type SpeechRecognitionConstructor = new () => SpeechRecognitionLike;
type SpeechRecognitionWindow = Window & {
  SpeechRecognition?: SpeechRecognitionConstructor;
  webkitSpeechRecognition?: SpeechRecognitionConstructor;
};

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

type HelpCategory = "basics" | "writing" | "structure" | "content" | "review" | "export" | "settings";

interface HelpTopicAction {
  label: string;
  run: () => unknown;
}

interface HelpTopic {
  id: string;
  title: string;
  category: HelpCategory;
  summary: string;
  when: string;
  steps: string[];
  tips: string[];
  actions: HelpTopicAction[];
  keywords: string[];
}

interface GuidedDemoStep {
  id: string;
  title: string;
  mode: string;
  summary: string;
  detail: string;
  points: string[];
  run: () => unknown;
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
  | "mic"
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
  | "collapse"
  | "expand"
  | "help"
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

interface OutlineModeHeading {
  text: string;
  anchor: string;
  level: number;
  line: number;
  end_line?: number | null;
  source_file?: string | null;
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
  mic: ["M12 4a3 3 0 0 0-3 3v5a3 3 0 0 0 6 0V7a3 3 0 0 0-3-3z", "M5 11a7 7 0 0 0 14 0", "M12 18v3", "M8 21h8"],
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
  collapse: ["M7 9l5 5 5-5"],
  expand: ["M9 7l5 5-5 5"],
  help: ["M9 9a3 3 0 1 1 5.4 1.8c-.7 1.1-2.4 1.5-2.4 3.2", "M12 18h.01", "M12 3a9 9 0 1 1 0 18 9 9 0 0 1 0-18z"],
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
let docsLiveRecognition: SpeechRecognitionLike | null = null;

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
const buttonHelpStyle = computed<CSSProperties>(() => ({
  left: `${buttonHelp.value.x}px`,
  top: `${buttonHelp.value.y}px`,
  transform: buttonHelp.value.placement === "top" ? "translate(-50%, -100%)" : "translate(-50%, 0)",
}));
const docsLiveSpeechAvailable = computed(() => Boolean(speechRecognitionConstructor()));
const docsLivePlaceholderRows = computed(() => docsLivePlaceholderEntries(docsLivePlaceholderText.value));
const docsLiveRequiredPlaceholderKeys = ["audience", "outcome", "owner", "deadline", "distribution target", "evidence", "tone", "reviewer"];
const docsLiveMissingPlaceholderKeys = computed(() => {
  const present = new Set(docsLivePlaceholderRows.value.map((entry) => entry.key));
  return docsLiveRequiredPlaceholderKeys.filter((key) => !present.has(key));
});
const docsLiveIntentCompletion = computed(() => {
  const present = docsLiveIntentFields.filter((field) => Boolean(docsLivePlaceholderValue(field.key))).length;
  return `${present}/${docsLiveIntentFields.length} intent fields`;
});
const canRunAgentProvider = computed(() => {
  if (agentProviderBusy.value || !agentProviderPackage.value?.profile.endpoint) return false;
  return !agentProviderPackage.value.profile.authHeader || Boolean(agentProviderApiKey.value.trim());
});
const agentSourcePackPreview = computed(() => buildAgenticSourcePack(agentSourcePackText.value));
const agentPlaybookFocusOptions = [
  { value: "all", label: "All workflows" },
  { value: "approval", label: "Approval and governance" },
  { value: "proposal", label: "Proposals and funding" },
  { value: "operations", label: "Operations and policy" },
  { value: "strategy", label: "Strategy and research" },
  { value: "technical", label: "Technical and LaTeX" },
  { value: "publishing", label: "Publishing and release" },
  { value: "revision", label: "Revision and polishing" },
] as const;
const agentPlaybookTargetOptions: Array<{ value: "all" | ExportTarget; label: string }> = [
  { value: "all", label: "All output targets" },
  { value: "pdf", label: "PDF" },
  { value: "docx", label: "DOCX" },
  { value: "html", label: "HTML" },
  { value: "blog", label: "Blog" },
  { value: "substack", label: "Substack" },
  { value: "latex", label: "LaTeX" },
  { value: "google-docs", label: "Google Docs" },
];
const filteredAgenticWorkflowPlaybooks = computed(() => {
  const query = agentPlaybookQuery.value.trim().toLowerCase();
  return agenticWorkflowPlaybooks.filter((playbook) => {
    const targetMatches = agentPlaybookTargetFilter.value === "all" || agentPlaybookTargets(playbook).includes(agentPlaybookTargetFilter.value);
    const focusMatches = agentPlaybookFocusFilter.value === "all" || agentPlaybookFocus(playbook) === agentPlaybookFocusFilter.value;
    const searchable = [
      playbook.label,
      playbook.summary,
      playbook.instruction,
      agentPlaybookFocusLabel(playbook),
      ...playbook.bestFor,
      ...playbook.expectedOutputs,
      ...agentPlaybookTargets(playbook),
    ]
      .join(" ")
      .toLowerCase();
    return targetMatches && focusMatches && (!query || searchable.includes(query));
  });
});
const agentProviderSourcePackMarkdown = computed(() =>
  agentProviderPackage.value ? formatAiProviderSourcePack(agentProviderPackage.value.sourcePack) : "",
);
const latestAgentRunHistory = computed(() => store.agentRunHistory[0] || null);
const latestDocsLiveDraftHistory = computed(() => store.docsLiveDraftHistory[0] || null);
const activeAgentControlCenter = computed(() => agentRun.value?.controlCenter || latestAgentRunHistory.value?.controlCenter || null);
const agentTaskLaneOptions: Array<"all" | AgenticWorkflowLane> = ["all", "create", "compose", "edit", "revise", "review", "distribute"];
const agentTaskStatusOptions: Array<"all" | AgentLifecycleExecutionStatus> = ["all", "queued", "in-progress", "needs-review", "complete", "blocked"];
const agentTaskOwnerOptions = computed(() => [
  "all",
  ...Array.from(new Set((agentRun.value?.lifecycleTasks || []).map((task) => task.owner).filter(Boolean))).sort(),
]);
const agentTaskSectionOptions = computed(() => {
  const sections = new Map((agentRun.value?.sectionWorkQueue || []).map((section) => [section.id, section.heading]));
  const taskSections = (agentRun.value?.lifecycleTasks || [])
    .map((task) => task.sectionId)
    .filter((sectionId): sectionId is string => Boolean(sectionId));
  return [
    { value: "all", label: "All sections" },
    ...Array.from(new Set(taskSections)).map((sectionId) => ({
      value: sectionId,
      label: sections.get(sectionId) || sectionId,
    })),
  ];
});
const agentTaskTargetOptions = computed(() => [
  "all",
  ...Array.from(new Set((agentRun.value?.lifecycleTasks || []).map((task) => task.target).filter((target): target is ExportTarget => Boolean(target)))).sort(),
]);
const agentEditAcceptanceRows = computed(() =>
  (agentRun.value?.editAcceptanceQueue || []).map((item) => ({
    item,
    state: agentEditAcceptanceStates.value[item.id] || defaultAgentEditAcceptanceState(item),
  })),
);
const acceptedAgentEditCount = computed(() =>
  agentEditAcceptanceRows.value.filter((row) => row.state.status === "accepted").length,
);
const agentLifecycleTaskRows = computed(() =>
  (agentRun.value?.lifecycleTasks || [])
    .map((task) => ({
      task,
      state: agentLifecycleTaskStates.value[task.id] || defaultAgentLifecycleTaskState(task),
    }))
    .filter((row) => {
      const query = agentTaskQuery.value.trim().toLowerCase();
      const laneMatches = agentTaskLaneFilter.value === "all" || row.task.lane === agentTaskLaneFilter.value;
      const statusMatches = agentTaskStatusFilter.value === "all" || row.state.status === agentTaskStatusFilter.value;
      const ownerMatches = agentTaskOwnerFilter.value === "all" || row.task.owner === agentTaskOwnerFilter.value;
      const sectionMatches = agentTaskSectionFilter.value === "all" || row.task.sectionId === agentTaskSectionFilter.value;
      const targetMatches = agentTaskTargetFilter.value === "all" || row.task.target === agentTaskTargetFilter.value;
      const evidenceText = row.task.evidence.join(" ").toLowerCase();
      const evidenceMatches =
        agentTaskEvidenceFilter.value === "all" ||
        (agentTaskEvidenceFilter.value === "has-evidence" && row.task.evidence.length > 0) ||
        (agentTaskEvidenceFilter.value === "missing-evidence" && row.task.evidence.length === 0) ||
        (agentTaskEvidenceFilter.value === "release-blocker" && /\b(blocked|blocker|missing|unresolved|required|approval|release)\b/i.test(`${row.task.title} ${row.task.nextStep} ${evidenceText}`));
      const searchable = [
        row.task.title,
        row.task.owner,
        row.task.lane,
        row.state.status,
        row.task.sectionId || "",
        row.task.target || "",
        row.task.nextStep,
        row.state.note || "",
        ...row.task.evidence,
      ]
        .join(" ")
        .toLowerCase();
      return laneMatches && statusMatches && ownerMatches && sectionMatches && targetMatches && evidenceMatches && (!query || searchable.includes(query));
    }),
);
const agentLifecycleTaskTotal = computed(() => agentRun.value?.lifecycleTasks.length || 0);
const filteredAgentRunHistory = computed(() => {
  const query = agentHistoryQuery.value.trim().toLowerCase();
  return store.agentRunHistory.filter((item) => {
    const statusMatches = agentHistoryStatusFilter.value === "all" || item.status === agentHistoryStatusFilter.value;
    const laneMatches = agentHistoryLaneFilter.value === "all" || item.lanes.includes(agentHistoryLaneFilter.value);
    const targetMatches = agentHistoryTargetFilter.value === "all" || item.distributionTargets.includes(agentHistoryTargetFilter.value);
    const searchable = [
      item.title,
      item.instruction,
      item.contextAnswers || "",
      item.sourcePackText || "",
      item.packetPreview || "",
      item.status,
      item.applicationMode,
      item.providerProfile || "",
      item.documentType,
      item.controlCenter?.summary || "",
      ...(item.lanes || []),
      ...(item.distributionTargets || []),
      ...(item.documentEvidence?.unresolvedPlaceholders || []),
      ...(item.documentEvidence?.citationTodos || []),
      ...(item.outlineCritique || []).map((critique) => `${critique.area} ${critique.heading} ${critique.detail} ${critique.recommendation}`),
      ...(item.sourcePack?.claimReview || []),
      ...(item.sourcePack?.cleanupBlockers || []),
      ...(item.sourcePack?.governanceBlockers || []),
      ...(item.sourcePack?.distributionBlockers || []),
      ...(item.sourcePack?.releaseEvidence || []),
    ]
      .join(" ")
      .toLowerCase();
    return statusMatches && laneMatches && targetMatches && (!query || searchable.includes(query));
  });
});
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
const paneSplitterVisible = computed(() => !["source", "focus", "preview", "export", "outline"].includes(store.mode));
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
const citationTodoItems = computed(() => extractCitationTodoItems(active.value.text));
const openCitationTodoCount = computed(() => citationTodoItems.value.filter((item) => item.status === "open").length);
const deferredCitationTodoCount = computed(() => citationTodoItems.value.filter((item) => item.status === "deferred").length);
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
const outlineModeHeadings = computed<OutlineModeHeading[]>(() =>
  outlineHeadings.value.filter((heading) => heading.level <= 4 && (!heading.source_file || !active.value.path || heading.source_file === active.value.path)),
);
const outlineDraftItems = computed(() => parseOutlinePlan(outlineDraftText.value));
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
const helpCategoryOptions: { id: HelpCategory; label: string }[] = [
  { id: "basics", label: "Basics" },
  { id: "writing", label: "Writing" },
  { id: "structure", label: "Structure" },
  { id: "content", label: "Content blocks" },
  { id: "review", label: "Review" },
  { id: "export", label: "Export" },
  { id: "settings", label: "Settings" },
];
const helpTopics = computed<HelpTopic[]>(() => [
  {
    id: "ai-first-composition",
    title: "AI-first document creation",
    category: "writing",
    summary: "Use Docs Live and Agent Workspace playbooks for outlines, context gathering, drafting, QA, review, and distribution preparation.",
    when: "Use this when you want to start from a business outcome instead of a blank page.",
    steps: [
      "Open AI Create, Docs Live, or Agent Workspace and choose a workflow playbook when the work matches a common business pattern.",
      "Give the agent the audience, outcome, constraints, source facts, tone, and placeholder values.",
      "Let the AI-created questionnaire expose missing context before drafting.",
      "Generate the draft section by section, then use the Lifecycle Task Board, QA register, and humanization checklist before applying it.",
    ],
    tips: [
      "AI-first does not mean unreviewed: keep provenance, comments, and human review status visible.",
      "Outline-first inputs produce better drafts because sections have a clear job before prose is generated.",
      "Use the lifecycle board when composition, editing, revision, review, and distribution need different owners or evidence.",
    ],
    actions: [
      { label: "AI Create", run: () => startAiDocumentCreation() },
      { label: "Agent workspace", run: () => openAgentWorkspace() },
      { label: "Guided demo", run: () => openGuidedDemo() },
      { label: "Review AI governance", run: () => (store.sidebar = "review") },
    ],
    keywords: ["AI first", "agentic", "create", "compose", "questionnaire", "QA", "humanize"],
  },
  {
    id: "agent-workspace",
    title: "Agent Workspace playbooks",
    category: "writing",
    summary: "Start complex document work from reusable AI playbooks, then generate governed packets with reviewers, section work queues, and export runbooks.",
    when: "Use this when the document needs more than one step, such as creation plus revision, evidence review, approval, provider handoff, and distribution.",
    steps: [
      "Open Agent Workspace and filter playbooks by focus or output target so business users can find board, proposal, SOP, technical, publishing, strategy, policy, release-notes, grant, or executive revision workflows.",
      "Edit the generated instruction so it names the audience, owner, deadline, evidence, reviewer, and target delivery channels.",
      "Plan the workflow to inspect lanes, missing inputs, placeholders, outline, and next actions.",
      "Generate the agent packet, then review the AI Control Center, reviewer agents, section work queue, audit trail, and distribution runbooks before applying or sending to a provider.",
      "Use the Lifecycle Task Board to run, insert, or copy owned task briefs for creation, composition, editing, revision, review, and distribution.",
    ],
    tips: [
      "Playbooks are starting points, not hidden automation: the full instruction remains editable before generation.",
      "Provider handoff packages include reviewer agents and section work queues so an approved model can continue the same governed workflow.",
      "Provider responses are applied as needs-review material, then tracked in run history with the exact wrapped review packet.",
    ],
    actions: [
      { label: "Open Agent Workspace", run: () => openAgentWorkspace() },
      { label: "Board memo playbook", run: () => openAgentWorkspace(agenticWorkflowPlaybooks[0]?.instruction || "") },
      { label: "Publishing playbook", run: () => openAgentWorkspace(agenticWorkflowPlaybooks.find((playbook) => playbook.id === "publish-to-blog-and-substack")?.instruction || "") },
      { label: "Policy playbook", run: () => openAgentWorkspace(agenticWorkflowPlaybooks.find((playbook) => playbook.id === "policy-to-approval")?.instruction || "") },
      { label: "Grant playbook", run: () => openAgentWorkspace(agenticWorkflowPlaybooks.find((playbook) => playbook.id === "grant-application-review")?.instruction || "") },
    ],
    keywords: ["agent", "playbook", "workflow", "board memo", "proposal", "SOP", "substack", "policy", "grant", "strategy", "release notes", "provider"],
  },
  {
    id: "agent-lifecycle-governance",
    title: "Agent lifecycle governance",
    category: "review",
    summary: "Turn AI plans into owned tasks, governed provider handoffs, review evidence, and reusable run history.",
    when: "Use this when AI work needs to move from idea to draft to review to distribution without becoming an untracked chat transcript.",
    steps: [
      "Open Agent Workspace, load a playbook, and generate the agent packet.",
      "Read the AI Control Center to decide which next action is safe: gather context, draft, revise, review, or prepare export.",
      "Use the Lifecycle Task Board to run the right workspace surface or insert/copy a durable task brief for another owner.",
      "Build a provider request only after reviewing the redacted prompt, lifecycle context, reviewer assignments, section queue, and safety checklist.",
      "Apply provider responses only through Apply response so NEditor wraps them in needs-review provenance and saves the wrapped packet to history.",
    ],
    tips: [
      "Run task is for immediate routing; Insert brief is for document-visible work orders; Copy brief is for external reviewers or delivery owners.",
      "Run history makes the agent workflow reusable: replan from the same instruction, append the saved packet, or copy the exact governed material later.",
      "Keep human review status separate from provider output so distribution readiness can prove what was inspected.",
    ],
    actions: [
      {
        label: "Open lifecycle board",
        run: () => {
          openAgentWorkspace(agenticWorkflowPlaybooks[0]?.instruction || "");
          generateAgentWorkspaceRun();
        },
      },
      {
        label: "Build provider package",
        run: () => {
          openAgentWorkspace(agenticWorkflowPlaybooks.find((playbook) => playbook.id === "publish-to-blog-and-substack")?.instruction || "");
          generateAgentWorkspaceRun();
          buildAgentProviderPackage();
        },
      },
      { label: "Review provenance", run: () => (store.sidebar = "review") },
    ],
    keywords: ["lifecycle", "governance", "provider", "audit", "task board", "provenance", "history"],
  },
  {
    id: "guided-demo",
    title: "Guided product demo",
    category: "basics",
    summary: "Walk through NEditor capabilities from AI creation to lifecycle governance, outline planning, review, templates, and export.",
    when: "Use this when onboarding a new user or evaluating what NEditor can do.",
    steps: [
      "Start the guided demo from Help or the command palette.",
      "Move step by step through AI creation, lifecycle task ownership, provider governance, outline planning, systematic composition, templates, review, and export.",
      "Use Try this step to route the workbench to the relevant real feature.",
      "Return to Help at any time for deeper workflow guidance.",
    ],
    tips: [
      "The demo is interactive: every step points at the actual product surface.",
      "It is designed for non-technical business users who need a quick capability tour.",
    ],
    actions: [
      { label: "Start guided demo", run: () => openGuidedDemo() },
      { label: "Open agent workspace", run: () => openAgentWorkspace() },
      { label: "AI Create", run: () => startAiDocumentCreation() },
      { label: "Help Center", run: () => openHelp("getting-started") },
    ],
    keywords: ["demo", "tour", "onboarding", "walkthrough", "capabilities"],
  },
  {
    id: "getting-started",
    title: "Getting started",
    category: "basics",
    summary: "Create, open, save, and orient yourself in the writing workspace.",
    when: "Use this when you are new to NEditor or returning to a document set after time away.",
    steps: [
      "Use New, Open, Save, and Save As from the File toolbar or File menu.",
      "Open a folder when you want a file browser, recent files, and project-relative includes.",
      "Use the View toolbar to choose Split, Source, Preview, Focus, Outline, Export, or Review mode.",
      "Keep the sidebar on Outline while drafting, Exports while preparing delivery, or Help while learning a workflow.",
    ],
    tips: [
      "Split view is best for normal writing because source and preview stay side by side.",
      "Focus mode hides nonessential panes when you only need to write.",
      "The status strip at the bottom reports saves, exports, diagnostics, and Docs Live progress.",
    ],
    actions: [
      { label: "New document", run: () => store.newDocument() },
      { label: "Open file", run: () => openDocument() },
      { label: "Show outline", run: () => showOutline() },
    ],
    keywords: ["new", "open", "save", "workspace", "mode", "sidebar"],
  },
  {
    id: "file-management",
    title: "File and workspace management",
    category: "basics",
    summary: "Manage individual documents, folders, recent files, snapshots, and disk changes.",
    when: "Use this when you need predictable document handling for business files and client deliverables.",
    steps: [
      "Save new documents with Save As so NEditor can track on-disk changes.",
      "Use Open Folder to browse a working folder and keep includes close to the main document.",
      "Use Rename, Duplicate, Reveal, Revert, and Snapshot from the File toolbar when managing versions.",
      "Use recently closed and pinned tabs to recover work without hunting through folders.",
    ],
    tips: [
      "Snapshots are useful before large AI-assisted rewrites or export cleanup passes.",
      "NEditor warns when a watched file changes outside the app so you can resolve conflicts before saving.",
    ],
    actions: [
      { label: "Open folder", run: () => openFolder() },
      { label: "Save workspace", run: () => saveWorkspace() },
      { label: "Show versioning", run: () => (store.sidebar = "versioning") },
    ],
    keywords: ["folder", "recent", "snapshot", "rename", "duplicate", "conflict"],
  },
  {
    id: "editing-markdown",
    title: "Writing and Markdown editing",
    category: "writing",
    summary: "Use rich toolbar commands while keeping the document portable Markdown.",
    when: "Use this for everyday drafting, formatting, and source editing.",
    steps: [
      "Select text and use Bold, Italic, Code, Link, Heading, Fence, Table, Figure, Equation, or TOC.",
      "Use Find and Replace for targeted edits across the current document.",
      "Fold all sections when you want to reduce visual noise in a long Markdown file.",
      "Use the command palette for commands that are not visible in the current toolbar layout.",
    ],
    tips: [
      "Markdown source remains readable, so files stay usable in Git, docs pipelines, and plain text tools.",
      "Line numbers and code folding can be toggled in Settings.",
    ],
    actions: [
      { label: "Find and replace", run: () => runEditorCommand(openSearchPanel) },
      { label: "Command palette", run: () => (commandPaletteOpen.value = true) },
      { label: "Settings", run: () => (store.sidebar = "settings") },
    ],
    keywords: ["markdown", "format", "find", "replace", "fold", "toolbar"],
  },
  {
    id: "outline-first",
    title: "Outline-first drafting",
    category: "structure",
    summary: "Plan chapters, sections, subsections, and subsubsections before writing body text.",
    when: "Use this when the structure matters before the prose, such as reports, proposals, policies, and board papers.",
    steps: [
      "Open the Outline sidebar to sketch a document plan using indented bullets or Markdown heading marks.",
      "Create a document from the outline or append the outline to the current document.",
      "Switch to Outline mode to CRUD the actual document headings without body text in the way.",
      "Use Docs Live from the outline when you are ready to flesh out sections systematically.",
    ],
    tips: [
      "Outline mode shows only chapter-level structure through subsubsections.",
      "Use Add child and Add sibling to keep document hierarchy consistent.",
    ],
    actions: [
      { label: "Plan outline", run: () => planDocumentOutline() },
      { label: "Outline mode", run: () => (store.mode = "outline") },
      { label: "Docs Live from outline", run: () => openDocsLiveFromOutline() },
    ],
    keywords: ["outline", "chapters", "sections", "plan", "CRUD", "structure"],
  },
  {
    id: "docs-live",
    title: "Docs Live voice drafting",
    category: "writing",
    summary: "Dictate the document type, context, outline, and placeholders, then generate a structured first draft.",
    when: "Use this when you want NEditor to act as a thought partner and co-writer from spoken or typed context.",
    steps: [
      "Open Docs Live and choose the document type and drafting depth.",
      "Load the current document outline or paste a planned outline.",
      "Dictate or type context, placeholders, constraints, and known facts.",
      "Generate the draft, review the section runbook, then insert or append the result.",
    ],
    tips: [
      "Docs Live creates a questionnaire when the context is thin so missing details are explicit.",
      "Generated drafts include QA, humanization, and review preparation blocks so humans can audit them.",
    ],
    actions: [
      { label: "Open Docs Live", run: () => openDocsLive() },
      { label: "Load outline", run: () => loadDocsLiveOutlineFromDocument() },
      { label: "Review panel", run: () => (store.sidebar = "review") },
    ],
    keywords: ["voice", "dictation", "AI", "draft", "questionnaire", "humanize"],
  },
  {
    id: "tables-calculations-templates",
    title: "Tables, calculations, and templates",
    category: "content",
    summary: "Insert structured tables, calculation blocks, and reusable business or scientific templates.",
    when: "Use this for financial models, metrics tables, formulas, calculations, and repeatable transform snippets.",
    steps: [
      "Open Templates to browse built-in calculation and transform templates by category.",
      "Use Create custom template for reusable organization-specific blocks.",
      "Open the table editor to normalize pasted spreadsheet data before inserting it.",
      "Run transforms after inserting calculation or external transform blocks.",
    ],
    tips: [
      "Template fill fields show which placeholder values you need to replace.",
      "External transform engines require trust before NEditor executes them.",
    ],
    actions: [
      { label: "Open templates", run: () => openTransformTemplates() },
      { label: "Open table editor", run: () => openTableEditor() },
      { label: "Run transforms", run: () => store.compileActive() },
    ],
    keywords: ["calc", "tables", "templates", "transform", "formula", "spreadsheet"],
  },
  {
    id: "references-citations",
    title: "References, citations, glossary, and index",
    category: "content",
    summary: "Keep citations, bibliography entries, cross references, glossary terms, and index terms visible.",
    when: "Use this when the document needs traceable sources, labeled figures, terms, or formal references.",
    steps: [
      "Open References to inspect resolved and missing citations.",
      "Add bibliography, glossary, index, list of figures, and list of tables snippets when needed.",
      "Use cross reference diagnostics to find broken labels before export.",
      "Choose citation style defaults in Settings for repeat exports.",
    ],
    tips: [
      "Missing citation keys are easier to fix before the export readiness pass.",
      "Glossary and index output can be included or excluded in export defaults.",
    ],
    actions: [
      { label: "Open references", run: () => (store.sidebar = "references") },
      { label: "Insert bibliography", run: () => insertBlock(bibliographySnippet) },
      { label: "Citation settings", run: () => (store.sidebar = "settings") },
    ],
    keywords: ["citation", "bibliography", "glossary", "index", "cross reference"],
  },
  {
    id: "review-provenance",
    title: "Review, comments, and AI provenance",
    category: "review",
    summary: "Track review comments, change notes, release status, and human review of AI-assisted sections.",
    when: "Use this before handing a document to reviewers, clients, managers, or compliance stakeholders.",
    steps: [
      "Open Review to see unresolved comments, release state, change notes, and AI provenance.",
      "Insert review comments or change notes directly into the Markdown source.",
      "Mark AI sources and AI-assisted sections as human reviewed after checking the content.",
      "Run AI Paste cleanup for text copied from chat tools before it enters the document.",
    ],
    tips: [
      "Review metadata travels with the Markdown, so provenance does not depend on local app state.",
      "Export options can include comments and AI provenance when an audit trail is required.",
    ],
    actions: [
      { label: "Review panel", run: () => (store.sidebar = "review") },
      { label: "Clean AI paste", run: () => openAiPaste() },
      { label: "Insert AI source", run: () => insertBlock(aiSnippet) },
    ],
    keywords: ["review", "comment", "provenance", "AI source", "approval", "human reviewed"],
  },
  {
    id: "export-publishing",
    title: "Export and publishing",
    category: "export",
    summary: "Prepare and export HTML, PDF, DOCX, PPTX, Markdown bundles, blog packages, Substack, LaTeX, and Google Docs packages.",
    when: "Use this when the document needs to leave NEditor as a deliverable or publishing package.",
    steps: [
      "Open Exports and choose the target format.",
      "Set delivery options such as HTML language, canonical URL, layout preset, comments, provenance, glossary, and agenda.",
      "Run Prepare for export to see readiness diagnostics before generating files.",
      "Save export profiles for client, internal, blog, Substack, LaTeX, or Google Docs delivery settings.",
    ],
    tips: [
      "Prepare for export is the safest first step when a document has references, figures, transforms, or layout directives.",
      "Export profiles reduce repeated setup for business users who publish the same way each week.",
    ],
    actions: [
      { label: "Export panel", run: () => (store.sidebar = "exports") },
      { label: "Prepare export", run: () => prepareForExport() },
      { label: "Export HTML", run: () => exportDocumentAs("html") },
    ],
    keywords: ["html", "pdf", "docx", "pptx", "blog", "substack", "latex", "google docs"],
  },
  {
    id: "keyboard-shortcuts",
    title: "Keyboard shortcuts",
    category: "settings",
    summary: "Use common shortcuts for save, open, new, export, formatting, and command discovery.",
    when: "Use this when you want to move quickly without relying on toolbar visibility.",
    steps: [
      "Use Cmd or Ctrl plus S to save, Shift plus Cmd or Ctrl plus S for Save As.",
      "Use Cmd or Ctrl plus O to open, N for new, E for export, B for bold, and I for italic.",
      "Use Cmd or Ctrl plus K, or Shift plus Cmd or Ctrl plus P, to open the command palette.",
      "Use the View toolbar to collapse toolbar rows when you need more writing space.",
    ],
    tips: [
      "The command palette is the fastest way to find actions while learning the app.",
      "Toolbar text can be resized or hidden if you prefer icons only.",
    ],
    actions: [
      { label: "Command palette", run: () => (commandPaletteOpen.value = true) },
      { label: "Collapse toolbars", run: () => setAllCommandToolbarsCollapsed(true) },
      { label: "Toolbar settings", run: () => (store.sidebar = "settings") },
    ],
    keywords: ["shortcut", "keyboard", "command palette", "collapse", "toolbar"],
  },
  {
    id: "display-accessibility",
    title: "Display and accessibility settings",
    category: "settings",
    summary: "Tune toolbar density, text size, theme, editor fonts, line height, and motion preferences.",
    when: "Use this when the interface feels too dense, too large, too small, or visually uncomfortable.",
    steps: [
      "Choose icons and text, icons only, or text only for toolbar buttons.",
      "Resize toolbar text and editor or preview fonts to match your workspace.",
      "Toggle high contrast, reduced motion, word wrap, line numbers, and code folding.",
      "Collapse toolbar rows when vertical screen space matters.",
    ],
    tips: [
      "Icons plus text is clearest while learning; icons only saves the most space once commands are familiar.",
      "Preview theme can match the app or stay fixed for export-like review.",
    ],
    actions: [
      { label: "Open settings", run: () => (store.sidebar = "settings") },
      { label: "Icons and text", run: () => (store.toolbarDisplay = "both") },
      { label: "Icons only", run: () => (store.toolbarDisplay = "icons") },
    ],
    keywords: ["accessibility", "theme", "font", "toolbar", "text size", "contrast"],
  },
  {
    id: "troubleshooting",
    title: "Troubleshooting",
    category: "basics",
    summary: "Find diagnostics, export readiness problems, transform trust prompts, and external file conflicts.",
    when: "Use this when preview, transforms, export, or saving does not behave as expected.",
    steps: [
      "Open Diagnostics to inspect compile errors and warnings.",
      "Use Go to source from diagnostics whenever line information is available.",
      "Run Prepare for export for delivery-specific checks.",
      "Review transform trust prompts before enabling external renderers.",
    ],
    tips: [
      "Most export issues are easiest to fix from readiness diagnostics rather than from the generated file.",
      "If a file changed on disk, resolve the conflict before saving to avoid losing external edits.",
    ],
    actions: [
      { label: "Diagnostics", run: () => (store.sidebar = "diagnostics") },
      { label: "Prepare export", run: () => prepareForExport() },
      { label: "Exports", run: () => (store.sidebar = "exports") },
    ],
    keywords: ["diagnostics", "error", "warning", "export readiness", "conflict", "trust"],
  },
]);
const filteredHelpTopics = computed(() => {
  const query = helpQuery.value.trim().toLowerCase();
  return helpTopics.value.filter((topic) => {
    if (helpCategory.value !== "all" && topic.category !== helpCategory.value) return false;
    if (!query) return true;
    return [topic.title, topic.summary, topic.when, ...topic.steps, ...topic.tips, ...topic.keywords].join(" ").toLowerCase().includes(query);
  });
});
const selectedHelpTopic = computed(() => {
  const selected = helpTopics.value.find((topic) => topic.id === selectedHelpTopicId.value);
  if (selected && filteredHelpTopics.value.includes(selected)) return selected;
  return filteredHelpTopics.value[0] || helpTopics.value[0] || null;
});
const guidedDemoSteps = computed<GuidedDemoStep[]>(() => [
  {
    id: "ai-create",
    title: "Create with AI",
    mode: "Agentic creation",
    summary: "Start with intent, audience, outline, context, and placeholders instead of a blank page.",
    detail: "Docs Live is the AI-first composition surface. It asks for missing context, builds a section plan, drafts systematically, and prepares review notes.",
    points: [
      "Choose the document type and drafting depth.",
      "Describe the business goal in speech or text.",
      "Add placeholders such as client, audience, owner, deadline, and required evidence.",
    ],
    run: () => startAiDocumentCreation(),
  },
  {
    id: "agent-playbooks",
    title: "Run a workflow playbook",
    mode: "Agent Workspace",
    summary: "Start common business workflows from reusable agent instructions.",
    detail: "Agent Workspace playbooks turn board approvals, proposals, SOPs, strategy memos, policies, release notes, grant applications, technical papers, publishing packages, and executive revision passes into editable governed workflows.",
    points: [
      "Filter playbooks by focus, output target, or search term, then adjust the instruction for the current document.",
      "Inspect missing inputs, reviewer agents, and section work queue before applying output.",
      "Build a provider package when an approved model should continue the workflow.",
    ],
    run: () => openAgentWorkspace(agenticWorkflowPlaybooks[0]?.instruction || ""),
  },
  {
    id: "lifecycle-tasks",
    title: "Turn plans into owned tasks",
    mode: "Lifecycle Task Board",
    summary: "Route creation, composition, editing, revision, review, and distribution through visible task briefs.",
    detail: "The Lifecycle Task Board converts an AI workflow into owned actions with evidence, next steps, and controls to run the task, insert a brief, or copy it for another stakeholder.",
    points: [
      "Review task lane, status, owner, next step, and evidence before anyone starts work.",
      "Use Run task to jump to Docs Live, Outline, Review, AI Paste, or Export readiness.",
      "Use Insert brief or Copy brief when a task should become a documented handoff.",
    ],
    run: () => {
      openAgentWorkspace(agenticWorkflowPlaybooks[0]?.instruction || "");
      generateAgentWorkspaceRun();
    },
  },
  {
    id: "provider-governance",
    title: "Govern provider handoffs",
    mode: "Provider review",
    summary: "Send only reviewed packages to approved AI providers and apply responses as needs-review material.",
    detail: "Provider handoff builds a redacted request package with lifecycle context, reviewer assignments, section work queues, and safety checks; Apply response wraps returned Markdown in AI provenance before it reaches the document.",
    points: [
      "Choose the approved provider profile, model, endpoint, and session-only key.",
      "Inspect the request package before any direct provider execution.",
      "Apply provider output only after previewing the response and preserving needs-review provenance.",
    ],
    run: () => {
      openAgentWorkspace(agenticWorkflowPlaybooks.find((playbook) => playbook.id === "publish-to-blog-and-substack")?.instruction || "");
      generateAgentWorkspaceRun();
      buildAgentProviderPackage();
    },
  },
  {
    id: "outline",
    title: "Plan the structure",
    mode: "Outline-first work",
    summary: "Create the document architecture before drafting prose.",
    detail: "The outline planner and Outline mode let users build chapters, sections, subsections, and subsubsections as a first-class document artifact.",
    points: [
      "Sketch a hierarchy in the Outline sidebar.",
      "Create or append the planned document skeleton.",
      "Switch to Outline mode to CRUD headings without body text in the way.",
    ],
    run: () => planDocumentOutline(),
  },
  {
    id: "compose",
    title: "Compose section by section",
    mode: "Systematic drafting",
    summary: "Generate a draft from outline plus context and inspect the runbook.",
    detail: "Docs Live produces a workflow, section cards, QA register, humanization checklist, and reviewer handoff so generated text is easier to evaluate.",
    points: [
      "Load the current document outline.",
      "Generate the draft after context and questionnaire answers are ready.",
      "Review section QA notes before applying the draft.",
    ],
    run: () => openDocsLiveFromDocumentOutline(),
  },
  {
    id: "templates",
    title: "Insert smart building blocks",
    mode: "Templates and transforms",
    summary: "Use reusable calculation, table, business, scientific, and transform templates.",
    detail: "Templates expose fill values and reusable snippets so non-technical users can insert structured document logic without writing syntax from scratch.",
    points: [
      "Search templates by category or transform type.",
      "Duplicate useful examples into custom templates.",
      "Run transforms after inserting calculations or diagrams.",
    ],
    run: () => openTransformTemplates(),
  },
  {
    id: "review",
    title: "Govern AI output",
    mode: "Review and provenance",
    summary: "Track comments, changes, AI sources, and human review status.",
    detail: "The Review panel keeps AI provenance visible and lets users mark AI-assisted material as human reviewed before export.",
    points: [
      "Clean pasted AI chat output before inserting it.",
      "Add comments and change notes.",
      "Mark AI sources and sections as human reviewed after inspection.",
    ],
    run: () => {
      store.sidebar = "review";
      openAiPaste();
    },
  },
  {
    id: "export",
    title: "Prepare delivery",
    mode: "Export readiness",
    summary: "Validate and export to business and publishing targets.",
    detail: "Export readiness checks diagnostics, metadata, references, layout, and target-specific requirements before creating deliverables.",
    points: [
      "Choose HTML, PDF, DOCX, PPTX, Markdown bundle, blog, Substack, LaTeX, or Google Docs package.",
      "Run Prepare for export before generating files.",
      "Save export profiles for repeated client or publishing workflows.",
    ],
    run: () => {
      store.sidebar = "exports";
      void prepareForExport();
    },
  },
]);
const currentDemoStep = computed(() => guidedDemoSteps.value[guidedDemoStepIndex.value] || guidedDemoSteps.value[0] || null);
const guidedDemoCompletedCount = computed(() => guidedDemoSteps.value.filter((step) => store.guidedDemoCompletedStepIds.includes(step.id)).length);
const guidedDemoCompletionPercent = computed(() =>
  guidedDemoSteps.value.length ? Math.round((guidedDemoCompletedCount.value / guidedDemoSteps.value.length) * 100) : 0,
);
const guidedDemoCompletionSummary = computed(() =>
  `${guidedDemoCompletedCount.value}/${guidedDemoSteps.value.length} demo capabilities completed: AI creation, playbooks, lifecycle tasks, provider governance, outline, composition, templates, review, and export.`,
);
const commandBarGroups = computed<CommandBarGroup[]>(() => [
  {
    id: "document",
    label: "Document",
    actions: [
      { id: "ai-create", label: "AI Create", title: "Create a document with the agentic Docs Live composer", icon: "ai", primary: true, run: () => startAiDocumentCreation() },
      { id: "agent", label: "Agent", title: "Plan creation, editing, revision, review, and distribution with the AI agent workspace", icon: "ai", primary: true, run: () => openAgentWorkspace() },
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
      { id: "docs-live", label: "Docs Live", title: "Open voice-guided document drafting", icon: "mic", primary: true, run: () => openDocsLive() },
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
      { id: "plan-outline", label: "Plan", title: "Plan document from outline", icon: "outline", run: () => planDocumentOutline() },
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
      { id: "help", label: "Help", title: "Open Help Center", icon: "help", run: () => openHelp() },
      { id: "demo", label: "Demo", title: "Start guided product demo", icon: "help", run: () => openGuidedDemo() },
    ],
  },
]);
const commandToolbarDefinitions = [
  { id: "file", label: "File", groupIds: ["document", "manage"] },
  { id: "writing", label: "Writing", groupIds: ["write", "insert"] },
  { id: "review-navigation", label: "Review & Navigate", groupIds: ["navigate", "review"] },
];
const toolbarCollapseRowIds = [...commandToolbarDefinitions.map((row) => row.id), "view"];
const commandToolbarRows = computed<CommandToolbarRow[]>(() => {
  const byId = new Map(commandBarGroups.value.map((group) => [group.id, group]));
  return commandToolbarDefinitions.map((row) => ({
    id: row.id,
    label: row.label,
    groups: row.groupIds.flatMap((id) => {
      const group = byId.get(id);
      return group ? [group] : [];
    }),
  }));
});
const normalizedToolbarCollapsedRows = (ids: string[]) =>
  Array.from(new Set(ids.filter((id) => toolbarCollapseRowIds.includes(id))));
const anyCommandToolbarsCollapsed = computed(() => toolbarCollapseRowIds.some((id) => store.toolbarCollapsedRows.includes(id)));
function isToolbarCollapsed(id: string) {
  return store.toolbarCollapsedRows.includes(id);
}
function toggleToolbarRow(id: string) {
  const current = new Set(store.toolbarCollapsedRows);
  if (current.has(id)) {
    current.delete(id);
  } else {
    current.add(id);
  }
  store.toolbarCollapsedRows = normalizedToolbarCollapsedRows([...current]);
}
function setAllCommandToolbarsCollapsed(collapsed: boolean) {
  store.toolbarCollapsedRows = collapsed ? [...toolbarCollapseRowIds] : [];
}
function helpCategoryLabel(category: HelpCategory) {
  return helpCategoryOptions.find((option) => option.id === category)?.label || "Help";
}
function selectHelpTopic(topicId: string) {
  selectedHelpTopicId.value = topicId;
}
function openHelp(topicId = "getting-started") {
  if (store.mode === "outline") store.mode = "split";
  store.sidebar = "help";
  selectedHelpTopicId.value = topicId;
  store.statusMessage = "Opened Help Center";
}
function runHelpAction(action: HelpTopicAction) {
  void action.run();
}
function currentEditorSelectionText() {
  const selection = editorView?.state.selection.main;
  if (!selection || selection.empty) return "";
  return editorView?.state.sliceDoc(selection.from, selection.to) || "";
}
function openAgentWorkspace(seedInstruction = "") {
  if (seedInstruction.trim()) {
    agentInstruction.value = seedInstruction.trim();
    agentContextAnswers.value = "";
  } else if (!agentInstruction.value.trim()) {
    agentInstruction.value = "Create or improve this document, revise it for the audience, run review readiness, and prepare the right distribution package.";
  }
  buildAgentWorkspacePlan();
  agentWorkspaceOpen.value = true;
  store.statusMessage = "Opened AI agent workspace";
}
function closeAgentWorkspace() {
  agentWorkspaceOpen.value = false;
}
function applyAgentWorkflowPlaybook(playbook: AgenticWorkflowPlaybook) {
  agentInstruction.value = playbook.instruction;
  agentContextAnswers.value = "";
  buildAgentWorkspacePlan();
  store.statusMessage = `Loaded ${playbook.label} playbook`;
}
function agentPlaybookTargets(playbook: AgenticWorkflowPlaybook): ExportTarget[] {
  const text = [playbook.instruction, playbook.summary, ...playbook.expectedOutputs].join(" ").toLowerCase();
  const targets: ExportTarget[] = [];
  const patterns: Array<[ExportTarget, RegExp]> = [
    ["pdf", /\bpdf\b/],
    ["docx", /\bdocx|word\b/],
    ["html", /\bhtml|web\b/],
    ["blog", /\bblog\b/],
    ["substack", /\bsubstack|newsletter\b/],
    ["latex", /\blatex|tex\b/],
    ["google-docs", /\bgoogle docs?\b/],
    ["pptx", /\bpptx|slides?|deck\b/],
    ["markdown-bundle", /\bmarkdown bundle|source package\b/],
  ];
  for (const [target, pattern] of patterns) {
    if (pattern.test(text)) targets.push(target);
  }
  return targets;
}
function agentPlaybookFocus(playbook: AgenticWorkflowPlaybook) {
  const text = [playbook.id, playbook.label, playbook.summary, playbook.instruction, ...playbook.bestFor, ...playbook.expectedOutputs]
    .join(" ")
    .toLowerCase();
  if (/\b(substack|blog|publish|release notes|announcement|newsletter)\b/.test(text)) return "publishing";
  if (/\b(technical|latex|research|architecture|academic|paper)\b/.test(text)) return "technical";
  if (/\b(strategy|market|portfolio|research notes)\b/.test(text)) return "strategy";
  if (/\b(policy|sop|operating procedure|control|compliance|governance)\b/.test(text)) return "operations";
  if (/\b(proposal|grant|funding|client|statement of work)\b/.test(text)) return "proposal";
  if (/\b(revision|revise|rewrite|executive audience|humanization)\b/.test(text)) return "revision";
  if (/\b(approval|board|decision|risk|sign-off)\b/.test(text)) return "approval";
  return "approval";
}
function agentPlaybookFocusLabel(playbook: AgenticWorkflowPlaybook) {
  return agentPlaybookFocusOptions.find((option) => option.value === agentPlaybookFocus(playbook))?.label || "Workflow";
}
function syncAgentProviderProfile() {
  const profile = aiProviderProfiles.find((item) => item.id === agentProviderId.value) || aiProviderProfiles[0];
  agentProviderEndpoint.value = profile.endpoint;
  agentProviderModel.value = profile.model;
  agentProviderPackage.value = null;
  agentProviderResult.value = null;
}
function buildAgentWorkspacePlan() {
  flushEditorTextToStore();
  agentPlan.value = buildAgenticWorkflowPlan({
    instruction: agentInstruction.value,
    contextAnswers: agentContextAnswers.value,
    sourcePackText: agentSourcePackText.value,
    documentTitle: active.value.compile?.semantic.title || active.value.title,
    documentText: active.value.text,
    selectedText: currentEditorSelectionText(),
  });
  agentRun.value = null;
  agentLifecycleTaskStates.value = {};
  agentEditAcceptanceStates.value = {};
  agentProviderPackage.value = null;
  agentProviderResult.value = null;
  store.statusMessage = `Planned ${agentPlan.value.steps.length} agent workflow steps`;
}
function generateAgentWorkspaceRun() {
  flushEditorTextToStore();
  if (!agentPlan.value) buildAgentWorkspacePlan();
  agentRun.value = buildAgenticWorkflowRun({
    instruction: agentInstruction.value,
    contextAnswers: agentContextAnswers.value,
    sourcePackText: agentSourcePackText.value,
    documentTitle: active.value.compile?.semantic.title || active.value.title,
    documentText: active.value.text,
    selectedText: currentEditorSelectionText(),
  });
  agentLifecycleTaskStates.value = Object.fromEntries(
    agentRun.value.lifecycleTasks.map((task) => [task.id, defaultAgentLifecycleTaskState(task)]),
  );
  agentEditAcceptanceStates.value = Object.fromEntries(
    agentRun.value.editAcceptanceQueue.map((item) => [item.id, defaultAgentEditAcceptanceState(item)]),
  );
  agentProviderPackage.value = null;
  agentProviderResult.value = null;
  recordAgentRunHistory(agentRun.value, "generated");
  store.statusMessage = `Generated agent packet for ${agentRun.value.plan.lanes.length} workflow lanes`;
}
function agentRunHistoryItem(
  run: AgenticWorkflowRun,
  status: AgentRunHistoryItem["status"],
  providerProfile = "",
  packetMarkdownOverride = "",
  sourcePack?: AiProviderSourcePack,
): AgentRunHistoryItem {
  const now = new Date().toISOString();
  const packetMarkdown = packetMarkdownOverride || run.markdown;
  return {
    runId: run.auditTrail.runId,
    title: run.plan.title,
    generatedAt: run.auditTrail.generatedAt,
    updatedAt: now,
    instruction: run.plan.instruction,
    contextAnswers: run.plan.contextAnswers,
    sourcePackText: run.plan.sourcePackText,
    documentType: run.plan.documentType,
    lanes: run.plan.lanes,
    distributionTargets: run.plan.distributionTargets,
    status,
    applicationMode: run.applicationMode,
    readinessScore: run.controlCenter.readinessScore,
    outputFingerprint: packetMarkdownOverride ? stableFingerprint(packetMarkdownOverride) : run.auditTrail.outputFingerprint,
    sourceFingerprint: run.auditTrail.sourceFingerprint,
    contextFingerprint: run.auditTrail.contextFingerprint,
    instructionFingerprint: run.auditTrail.instructionFingerprint,
    packetMarkdown: packetMarkdown.slice(0, 24_000),
    packetPreview: packetMarkdownOverride ? agentPacketPreview(packetMarkdownOverride) : run.summary.slice(0, 260),
    sectionCount: run.sectionWorkQueue.length,
    reviewerCount: run.reviewerAgents.length,
    taskCount: run.lifecycleTasks.length,
    lifecycleTaskStates: agentLifecycleTaskStateList(),
    editAcceptanceStates: agentEditAcceptanceStateList(),
    controlCenter: run.controlCenter,
    documentEvidence: run.documentEvidence,
    outlineCritique: run.outlineCritique,
    sourcePack,
    appliedAt: status === "generated" ? undefined : now,
    providerProfile: providerProfile || undefined,
  };
}
function agentPacketPreview(markdown: string) {
  return markdown
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/[#>*_`[\]-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .slice(0, 260);
}
function defaultAgentLifecycleTaskState(task: AgenticLifecycleTask): AgentLifecycleTaskState {
  return {
    taskId: task.id,
    title: task.title,
    lane: task.lane,
    status: task.status === "blocked" ? "blocked" : task.status === "needs-input" ? "needs-review" : "queued",
    updatedAt: agentRun.value?.auditTrail.generatedAt || new Date(0).toISOString(),
  };
}
function defaultAgentEditAcceptanceState(item: AgenticEditAcceptanceItem): AgentEditAcceptanceState {
  return {
    itemId: item.id,
    heading: item.heading,
    scope: item.scope,
    status: "queued",
    updatedAt: agentRun.value?.auditTrail.generatedAt || new Date(0).toISOString(),
  };
}
function agentLifecycleTaskStateList() {
  if (!agentRun.value) return [];
  return agentRun.value.lifecycleTasks.map((task) => agentLifecycleTaskStates.value[task.id] || defaultAgentLifecycleTaskState(task));
}
function agentEditAcceptanceStateList() {
  if (!agentRun.value) return [];
  return agentRun.value.editAcceptanceQueue.map((item) => agentEditAcceptanceStates.value[item.id] || defaultAgentEditAcceptanceState(item));
}
function persistAgentLifecycleTaskStates() {
  if (!agentRun.value) return;
  const existing = store.agentRunHistory.find((item) => item.runId === agentRun.value?.auditTrail.runId);
  const packetMarkdownOverride = existing?.packetMarkdown && existing.packetMarkdown !== agentRun.value.markdown ? existing.packetMarkdown : "";
  recordAgentRunHistory(agentRun.value, existing?.status || "generated", existing?.providerProfile || "", packetMarkdownOverride, existing?.sourcePack);
}
function persistAgentEditAcceptanceStates() {
  if (!agentRun.value) return;
  const existing = store.agentRunHistory.find((item) => item.runId === agentRun.value?.auditTrail.runId);
  const packetMarkdownOverride = existing?.packetMarkdown && existing.packetMarkdown !== agentRun.value.markdown ? existing.packetMarkdown : "";
  recordAgentRunHistory(agentRun.value, existing?.status || "generated", existing?.providerProfile || "", packetMarkdownOverride, existing?.sourcePack);
}
function recordAgentRunHistory(
  run: AgenticWorkflowRun,
  status: AgentRunHistoryItem["status"],
  providerProfile = "",
  packetMarkdownOverride = "",
  sourcePack?: AiProviderSourcePack,
) {
  store.recordAgentRunHistory(agentRunHistoryItem(run, status, providerProfile, packetMarkdownOverride, sourcePack));
}
function setAgentLifecycleTaskStatus(task: AgenticLifecycleTask, status: AgentLifecycleExecutionStatus) {
  const now = new Date().toISOString();
  agentLifecycleTaskStates.value = {
    ...agentLifecycleTaskStates.value,
    [task.id]: {
      ...(agentLifecycleTaskStates.value[task.id] || defaultAgentLifecycleTaskState(task)),
      status,
      updatedAt: now,
      completedAt: status === "complete" ? now : undefined,
    },
  };
  persistAgentLifecycleTaskStates();
  store.statusMessage = `Marked ${task.title} ${status}`;
}
function setAgentLifecycleTaskNote(task: AgenticLifecycleTask, note: string) {
  const now = new Date().toISOString();
  agentLifecycleTaskStates.value = {
    ...agentLifecycleTaskStates.value,
    [task.id]: {
      ...(agentLifecycleTaskStates.value[task.id] || defaultAgentLifecycleTaskState(task)),
      note: note.trim() || undefined,
      updatedAt: now,
    },
  };
  persistAgentLifecycleTaskStates();
  store.statusMessage = `Updated ${task.title} task note`;
}
function replanAgentHistoryRun(item: AgentRunHistoryItem) {
  agentInstruction.value = item.instruction;
  agentContextAnswers.value = item.contextAnswers || "";
  agentSourcePackText.value = item.sourcePackText || "";
  agentRun.value = null;
  agentLifecycleTaskStates.value = Object.fromEntries((item.lifecycleTaskStates || []).map((state) => [state.taskId, state]));
  agentEditAcceptanceStates.value = Object.fromEntries((item.editAcceptanceStates || []).map((state) => [state.itemId, state]));
  agentProviderPackage.value = null;
  agentProviderResult.value = null;
  buildAgentWorkspacePlan();
  store.statusMessage = `Replanned saved agent run ${item.runId}`;
}
function addAgentSourcePackItem() {
  const serialized = serializeAgenticSourcePackItem(agentSourcePackKind.value, agentSourcePackLabel.value, agentSourcePackDetail.value);
  agentSourcePackText.value = [agentSourcePackText.value.trim(), serialized].filter(Boolean).join("\n");
  agentSourcePackLabel.value = "";
  agentSourcePackDetail.value = "";
  buildAgentWorkspacePlan();
  store.statusMessage = "Added item to agent source pack";
}
function removeAgentSourcePackItem(itemId: string) {
  const kept = agentSourcePackPreview.value.items.filter((item) => item.id !== itemId);
  agentSourcePackText.value = kept.map((item) => serializeAgenticSourcePackItem(item.kind, item.label, item.detail)).join("\n");
  buildAgentWorkspacePlan();
  store.statusMessage = "Removed item from agent source pack";
}
function setAgentEditAcceptanceStatus(item: AgenticEditAcceptanceItem, status: AgentEditAcceptanceStatus) {
  const now = new Date().toISOString();
  agentEditAcceptanceStates.value = {
    ...agentEditAcceptanceStates.value,
    [item.id]: {
      ...(agentEditAcceptanceStates.value[item.id] || defaultAgentEditAcceptanceState(item)),
      status,
      updatedAt: now,
      appliedAt: status === "accepted" ? agentEditAcceptanceStates.value[item.id]?.appliedAt : undefined,
    },
  };
  persistAgentEditAcceptanceStates();
  store.statusMessage = `Marked ${item.heading} ${status}`;
}
function setAgentEditAcceptanceNote(item: AgenticEditAcceptanceItem, note: string) {
  const now = new Date().toISOString();
  agentEditAcceptanceStates.value = {
    ...agentEditAcceptanceStates.value,
    [item.id]: {
      ...(agentEditAcceptanceStates.value[item.id] || defaultAgentEditAcceptanceState(item)),
      note: note.trim() || undefined,
      updatedAt: now,
    },
  };
  persistAgentEditAcceptanceStates();
  store.statusMessage = `Updated ${item.heading} acceptance note`;
}
function agentReviewCommentTask(comment: AgenticReviewCommentResolution) {
  return agentRun.value?.lifecycleTasks.find((task) => task.id === `task-${comment.id}`) || null;
}
function agentReviewCommentState(comment: AgenticReviewCommentResolution) {
  const task = agentReviewCommentTask(comment);
  if (!task) return null;
  return agentLifecycleTaskStates.value[task.id] || defaultAgentLifecycleTaskState(task);
}
function setAgentReviewCommentStatus(comment: AgenticReviewCommentResolution, status: AgentLifecycleExecutionStatus) {
  const task = agentReviewCommentTask(comment);
  if (!task) return;
  setAgentLifecycleTaskStatus(task, status);
}
function setAgentReviewCommentNote(comment: AgenticReviewCommentResolution, note: string) {
  const task = agentReviewCommentTask(comment);
  if (!task) return;
  setAgentLifecycleTaskNote(task, note);
}
function appendAgentHistoryPacket(item: AgentRunHistoryItem) {
  if (!item.packetMarkdown) return;
  applyAgentMarkdown(item.packetMarkdown, "append-packet");
  store.statusMessage = `Appended saved agent packet ${item.runId}`;
}
async function copyAgentHistoryPacket(item: AgentRunHistoryItem) {
  if (!item.packetMarkdown) return;
  try {
    await navigator.clipboard?.writeText(item.packetMarkdown);
    store.statusMessage = `Copied saved agent packet ${item.runId}`;
  } catch {
    store.statusMessage = `Saved agent packet ${item.runId} is ready to copy`;
  }
}
function removeAgentHistoryRun(item: AgentRunHistoryItem) {
  store.removeAgentRunHistory(item.runId);
  store.statusMessage = `Removed saved agent run ${item.runId}`;
}
function clearAgentHistory() {
  store.clearAgentRunHistory();
  store.statusMessage = "Cleared saved agent run history";
}
function agentHistoryAuditMarkdown() {
  const runs = filteredAgentRunHistory.value;
  const generatedAt = new Date().toISOString();
  const filters = [
    agentHistoryQuery.value.trim() ? `query=${agentHistoryQuery.value.trim()}` : "",
    agentHistoryStatusFilter.value !== "all" ? `status=${agentHistoryStatusFilter.value}` : "",
    agentHistoryLaneFilter.value !== "all" ? `lane=${agentHistoryLaneFilter.value}` : "",
    agentHistoryTargetFilter.value !== "all" ? `target=${agentHistoryTargetFilter.value}` : "",
  ].filter(Boolean);
  const lines = [
    "## Agent Run History Audit",
    "",
    "```ai-audit",
    "type: agent-run-history",
    `generatedAt: ${generatedAt}`,
    `runs: ${runs.length}`,
    `filters: ${filters.join("; ") || "all"}`,
    "source: NEditor Agent Workspace",
    "```",
    "",
    "| Run | Status | Lanes | Targets | Readiness | Provider | Evidence | Tasks |",
    "| --- | --- | --- | --- | ---: | --- | --- | --- |",
    ...runs.slice(0, 24).map((item) =>
      [
        agentAuditTableCell(`${item.title} (${item.runId})`),
        agentAuditTableCell(item.status),
        agentAuditTableCell(item.lanes.join(", ")),
        agentAuditTableCell(item.distributionTargets.join(", ") || "review"),
        `${item.readinessScore}`,
        agentAuditTableCell(item.providerProfile || "local planner"),
        agentAuditTableCell(agentRunHistoryEvidenceSummary(item)),
        agentAuditTableCell(agentRunHistoryTaskStateSummary(item) || `${item.taskCount || 0} tasks`),
      ].join(" | ").replace(/^/, "| ").replace(/$/, " |"),
    ),
    "",
    "### Run Notes",
    "",
    ...runs.slice(0, 24).flatMap((item) => [
      `- **${agentAuditInline(item.title)}** (${agentAuditInline(item.runId)}): ${agentAuditInline(item.controlCenter?.summary || item.packetPreview || "No summary captured.")}`,
      item.outlineCritique?.length ? `  - Outline: ${agentAuditInline(agentRunHistoryOutlineSummary(item))}` : "",
      item.sourcePack ? `  - Source pack: ${agentAuditInline(agentRunHistorySourcePackSummary(item))}` : "",
    ].filter(Boolean)),
  ];
  return lines.join("\n");
}
function agentAuditTableCell(value: string) {
  return (value || "").replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}
function agentAuditInline(value: string) {
  return (value || "").replace(/\r?\n/g, " ").trim();
}
function insertAgentHistoryAudit() {
  if (!filteredAgentRunHistory.value.length) return;
  insertBlock(agentHistoryAuditMarkdown());
  store.statusMessage = `Inserted agent history audit for ${filteredAgentRunHistory.value.length} runs`;
}
async function copyAgentHistoryAudit() {
  if (!filteredAgentRunHistory.value.length) return;
  const audit = agentHistoryAuditMarkdown();
  try {
    await navigator.clipboard?.writeText(audit);
    store.statusMessage = `Copied agent history audit for ${filteredAgentRunHistory.value.length} runs`;
  } catch {
    store.statusMessage = "Agent history audit is ready to copy";
  }
}
function agentRunHistoryTaskStateSummary(item: AgentRunHistoryItem) {
  const counts = new Map<AgentLifecycleExecutionStatus, number>();
  for (const state of item.lifecycleTaskStates || []) {
    counts.set(state.status, (counts.get(state.status) || 0) + 1);
  }
  return (["queued", "in-progress", "needs-review", "complete", "blocked"] as AgentLifecycleExecutionStatus[])
    .filter((status) => counts.has(status))
    .map((status) => `${counts.get(status)} ${status}`)
    .join(", ");
}
function agentRunHistoryEvidenceSummary(item: AgentRunHistoryItem) {
  const evidence = item.documentEvidence;
  if (!evidence) return "none captured";
  const parts = [
    evidence.unresolvedPlaceholders.length ? `${evidence.unresolvedPlaceholders.length} placeholders` : "",
    evidence.citationTodos.length ? `${evidence.citationTodos.length} citation TODOs` : "",
    evidence.claimInventory.length ? `${evidence.claimInventory.length} claims` : "",
    evidence.humanizationFindings.length ? `${evidence.humanizationFindings.length} humanization notes` : "",
    evidence.reviewCommentResolutions.length ? `${evidence.reviewCommentResolutions.length} comment queue items` : evidence.unresolvedComments ? `${evidence.unresolvedComments} comments` : "",
    evidence.unreviewedAiMarkers ? `${evidence.unreviewedAiMarkers} AI markers` : "",
    evidence.brokenLinkHints.length ? `${evidence.brokenLinkHints.length} link checks` : "",
  ].filter(Boolean);
  return parts.join(", ") || "no blockers";
}
function agentRunHistoryOutlineSummary(item: AgentRunHistoryItem) {
  const counts = new Map<string, number>();
  for (const critique of item.outlineCritique || []) {
    counts.set(critique.severity, (counts.get(critique.severity) || 0) + 1);
  }
  return ["blocker", "warning", "info"]
    .filter((severity) => counts.has(severity))
    .map((severity) => `${counts.get(severity)} ${severity}`)
    .join(", ");
}
function agentRunHistorySourcePackSummary(item: AgentRunHistoryItem) {
  const sourcePack = item.sourcePack;
  if (!sourcePack) return "none captured";
  const count =
    sourcePack.contextSources.length +
    (sourcePack.userSources?.length || 0) +
    sourcePack.claimReview.length +
    sourcePack.cleanupBlockers.length +
    sourcePack.governanceBlockers.length +
    sourcePack.distributionBlockers.length +
    sourcePack.releaseEvidence.length;
  return `${count} provider handoff item${count === 1 ? "" : "s"}`;
}
function buildAgentProviderPackage() {
  if (!agentRun.value) generateAgentWorkspaceRun();
  if (!agentRun.value) return;
  agentProviderPackage.value = buildAiProviderRequestPackage(agentRun.value, {
    profileId: agentProviderId.value,
    endpoint: agentProviderEndpoint.value,
    model: agentProviderModel.value,
    keyEnv: agentProviderKeyEnv.value,
  });
  const existing = store.agentRunHistory.find((item) => item.runId === agentRun.value?.auditTrail.runId);
  recordAgentRunHistory(
    agentRun.value,
    existing?.status || "generated",
    existing?.providerProfile || agentProviderPackage.value.profile.label,
    existing?.packetMarkdown && existing.packetMarkdown !== agentRun.value.markdown ? existing.packetMarkdown : "",
    agentProviderPackage.value.sourcePack,
  );
  agentProviderResult.value = null;
  store.statusMessage = `Built ${agentProviderPackage.value.profile.label} request package`;
}
async function runAgentProviderRequest() {
  if (!agentProviderPackage.value || agentProviderBusy.value) return;
  agentProviderBusy.value = true;
  agentProviderResult.value = null;
  try {
    agentProviderResult.value = await executeAiProviderRequestPackage(agentProviderPackage.value, agentProviderApiKey.value);
    store.statusMessage = `Provider returned ${agentProviderResult.value.markdown.length} Markdown characters for review`;
  } catch (error) {
    store.lastError = error instanceof Error ? error.message : String(error);
    store.statusMessage = "Provider request failed";
  } finally {
    agentProviderBusy.value = false;
  }
}
function applyAgentProviderResponse() {
  if (!agentProviderResult.value) return;
  const reviewMarkdown = buildAiProviderResponseReviewMarkdown(agentProviderResult.value.markdown, {
    profileLabel: agentProviderPackage.value?.profile.label,
    model: agentProviderPackage.value?.profile.model,
    runId: agentRun.value?.auditTrail.runId,
  });
  applyAgentMarkdown(reviewMarkdown, agentRun.value?.applicationMode || "append-packet");
  if (agentRun.value) {
    recordAgentRunHistory(agentRun.value, "provider-applied", agentProviderPackage.value?.profile.label || "", reviewMarkdown, agentProviderPackage.value?.sourcePack);
  }
  store.statusMessage = "Applied provider response for human review";
  closeAgentWorkspace();
}
async function copyAgentProviderPackage() {
  if (!agentProviderPackage.value) return;
  try {
    await navigator.clipboard?.writeText(agentProviderPackage.value.markdown);
    store.statusMessage = "Copied provider request package";
  } catch {
    store.statusMessage = "Provider request package is ready to copy";
  }
}
async function copyAgentProviderSourcePack() {
  if (!agentProviderPackage.value) return;
  try {
    await navigator.clipboard?.writeText(agentProviderSourcePackMarkdown.value);
    store.statusMessage = "Copied provider source evidence pack";
  } catch {
    store.statusMessage = "Provider source evidence pack is ready to copy";
  }
}
function applyAgentWorkspaceRun() {
  const run = agentRun.value;
  if (!run) return;
  if (run.editAcceptanceQueue.length) {
    if (!acceptedAgentEditCount.value) {
      store.statusMessage = "Accept at least one queued edit before applying agent revisions";
      return;
    }
    applyAcceptedAgentEdits();
    return;
  }
  applyAgentMarkdown(run.revision?.proposedText || run.markdown, run.applicationMode);
  recordAgentRunHistory(run, "applied");
  store.statusMessage = "Applied agent output for human review";
  closeAgentWorkspace();
}
function appendAgentWorkspacePacket() {
  const run = agentRun.value;
  if (!run) return;
  applyAgentMarkdown(run.markdown, "append-packet");
  recordAgentRunHistory(run, "applied");
  store.statusMessage = "Appended agent packet for review";
}
async function copyAgentWorkspacePacket() {
  const run = agentRun.value;
  if (!run) return;
  try {
    await navigator.clipboard?.writeText(run.markdown);
    store.statusMessage = "Copied current agent packet";
  } catch {
    store.statusMessage = "Current agent packet is ready to copy";
  }
}
function applyAcceptedAgentEdits() {
  const run = agentRun.value;
  if (!run) return;
  const acceptedRows = agentEditAcceptanceRows.value.filter((row) => row.state.status === "accepted");
  if (!acceptedRows.length) {
    store.statusMessage = "No accepted agent edits to apply";
    return;
  }
  let documentText = active.value.text;
  let selectionHandled = false;
  for (const row of acceptedRows) {
    const item = row.item;
    if (item.scope === "selection" && editorView && !selectionHandled) {
      const range = editorView.state.selection.main;
      if (!range.empty) {
        editorView.dispatch({
          changes: { from: range.from, to: range.to, insert: item.proposedText },
          selection: { anchor: range.from + item.proposedText.length },
        });
        documentText = editorView.state.doc.toString();
        selectionHandled = true;
      } else if (documentText.includes(item.originalText)) {
        documentText = documentText.replace(item.originalText, item.proposedText);
      } else {
        documentText = `${documentText.trimEnd()}\n\n${item.proposedText}`;
      }
    } else if (item.scope === "section") {
      documentText = replaceOrAppendMarkdownSection(documentText, item.proposedText, item.heading);
    } else if (item.scope === "document") {
      documentText = item.proposedText;
    }
    const now = new Date().toISOString();
    agentEditAcceptanceStates.value = {
      ...agentEditAcceptanceStates.value,
      [item.id]: {
        ...(agentEditAcceptanceStates.value[item.id] || defaultAgentEditAcceptanceState(item)),
        status: "accepted",
        updatedAt: now,
        appliedAt: now,
      },
    };
  }
  store.updateText(documentText);
  editorView?.focus();
  store.sidebar = "review";
  persistAgentEditAcceptanceStates();
  recordAgentRunHistory(run, "applied");
  store.statusMessage = `Applied ${acceptedRows.length} accepted agent edit${acceptedRows.length === 1 ? "" : "s"}`;
  closeAgentWorkspace();
}
function reviseAgentAcceptanceItem(item: AgenticEditAcceptanceItem) {
  setAgentEditAcceptanceStatus(item, "needs-revision");
  aiPasteText.value = item.proposedText;
  aiInsertMode.value = item.scope === "selection" ? "selection" : item.scope === "document" ? "replace" : "section";
  closeAgentWorkspace();
  openAiPaste();
  store.statusMessage = `Opened AI Paste to revise ${item.heading}`;
}
function applyAgentMarkdown(markdown: string, mode: AgenticWorkflowRun["applicationMode"]) {
  if (mode === "replace-selection" && editorView) {
    const range = editorView.state.selection.main;
    editorView.dispatch({
      changes: { from: range.from, to: range.to, insert: markdown },
      selection: { anchor: range.from + markdown.length },
    });
    store.updateText(editorView.state.doc.toString());
    editorView.focus();
  } else if (mode === "replace-document") {
    store.updateText(markdown);
  } else {
    store.updateText(`${active.value.text.trimEnd()}\n\n${markdown}`);
  }
  store.sidebar = "review";
}
function hydrateDocsLiveFromAgentPlan() {
  const plan = agentPlan.value;
  if (!plan) return;
  docsLiveTargetSection.value = null;
  docsLiveDocumentType.value = plan.documentType;
  docsLiveTitle.value = plan.title;
  docsLiveOutlineText.value = plan.suggestedOutline;
  docsLiveContext.value = [plan.context, plan.sourcePack.markdown ? `\nManaged source pack:\n${plan.sourcePack.markdown}` : ""].filter(Boolean).join("\n");
  docsLivePlaceholderText.value = plan.placeholderText;
  docsLiveQuestionnaireAnswerText.value = plan.contextAnswers
    ? `Agent context answers:\n${plan.contextAnswers}`
    : plan.missingInputs.length
      ? `Missing inputs to resolve:\n${plan.missingInputs.map((input) => `- ${input}`).join("\n")}`
      : docsLiveQuestionnaireAnswerText.value;
  refreshDocsLiveQuestionnaire();
  closeAgentWorkspace();
  docsLiveOpen.value = true;
  store.statusMessage = "Sent agent plan to Docs Live";
}
function insertAgentSectionBrief(section: AgenticSectionWorkItem) {
  const run = agentRun.value;
  if (!run) return;
  insertBlock(buildAgenticSectionWorkBrief(section, run.reviewerAgents));
  store.updateText(editorView?.state.doc.toString() || active.value.text);
  store.sidebar = "review";
  store.statusMessage = `Inserted ${section.heading} section brief`;
}
function draftAgentSectionWithDocsLive(section: AgenticSectionWorkItem) {
  const run = agentRun.value;
  if (!run) return;
  const reviewerLines = section.reviewerAgentIds
    .map((id) => run.reviewerAgents.find((agent) => agent.id === id))
    .filter((agent): agent is NonNullable<typeof agent> => Boolean(agent))
    .map((agent) => `- ${agent.label} [${agent.status}]: ${agent.mandate}`);
  docsLiveDocumentType.value = run.plan.documentType;
  docsLiveTitle.value = `${run.plan.title} - ${section.heading}`;
  docsLiveOutlineText.value = `${"  ".repeat(Math.max(0, section.level - 1))}- ${section.heading}`;
  docsLiveContext.value = [
    run.plan.context,
    "",
    `Section drafting instruction: ${section.draftingInstruction}`,
    "",
    "Completion criteria:",
    ...section.completionCriteria.map((item) => `- ${item}`),
    "",
    "Assigned reviewer agents:",
    ...reviewerLines,
  ].join("\n");
  docsLivePlaceholderText.value = run.plan.placeholderText;
  docsLiveQuestionnaireAnswerText.value = `Draft only this section first: ${section.heading}. Keep unresolved facts visible and preserve reviewer handoff notes.`;
  docsLiveDraftingDepth.value = section.draftingDepth;
  docsLiveInsertMode.value = "section";
  docsLiveTargetSection.value = section;
  refreshDocsLiveQuestionnaire();
  closeAgentWorkspace();
  docsLiveOpen.value = true;
  store.statusMessage = `Sent ${section.heading} to Docs Live`;
}
function runAgentLifecycleTask(task: AgenticLifecycleTask) {
  setAgentLifecycleTaskStatus(task, "in-progress");
  const section = task.sectionId ? agentRun.value?.sectionWorkQueue.find((item) => item.id === task.sectionId) : null;
  if (section) {
    draftAgentSectionWithDocsLive(section);
    return;
  }
  if (task.target) {
    store.exportTarget = task.target;
  }
  runAgenticStep({
    id: task.id,
    lane: task.lane,
    title: task.title,
    detail: task.nextStep,
    action: task.action,
    status: task.status === "ready" ? "ready" : "needs-input",
  });
}
function insertAgentLifecycleTaskBrief(task: AgenticLifecycleTask) {
  insertBlock(buildAgenticLifecycleTaskBrief(task));
  store.updateText(editorView?.state.doc.toString() || active.value.text);
  store.sidebar = "review";
  store.statusMessage = `Inserted ${task.title} task brief`;
}
async function copyAgentLifecycleTaskBrief(task: AgenticLifecycleTask) {
  const brief = buildAgenticLifecycleTaskBrief(task);
  try {
    await navigator.clipboard?.writeText(brief);
    store.statusMessage = `Copied ${task.title} task brief`;
  } catch {
    store.statusMessage = `${task.title} task brief is ready to copy`;
  }
}
function insertAgentReleaseEvidenceAuditPackage() {
  const run = agentRun.value;
  if (!run) return;
  insertBlock(buildAgenticReleaseEvidenceAuditPackage(run));
  store.updateText(editorView?.state.doc.toString() || active.value.text);
  store.sidebar = "review";
  store.statusMessage = "Inserted release evidence audit package";
}
async function copyAgentReleaseEvidenceAuditPackage() {
  const run = agentRun.value;
  if (!run) return;
  const auditPackage = buildAgenticReleaseEvidenceAuditPackage(run);
  try {
    await navigator.clipboard?.writeText(auditPackage);
    store.statusMessage = "Copied release evidence audit package";
  } catch {
    store.statusMessage = "Release evidence audit package is ready to copy";
  }
}
function runAgentPlanReview() {
  closeAgentWorkspace();
  store.mode = "review";
  store.sidebar = "review";
  store.statusMessage = "Agent routed document to review readiness";
}
function runAgentPlanDistribution() {
  const plan = agentPlan.value;
  closeAgentWorkspace();
  if (plan?.distributionTargets[0]) store.exportTarget = plan.distributionTargets[0];
  store.mode = "export";
  store.sidebar = "exports";
  void prepareForExport();
}
function runAgenticStep(step: AgenticWorkflowStep) {
  switch (step.action) {
    case "open-docs-live":
      hydrateDocsLiveFromAgentPlan();
      break;
    case "generate-docs-live-draft":
      hydrateDocsLiveFromAgentPlan();
      generateDocsLiveDraft();
      break;
    case "open-outline":
      closeAgentWorkspace();
      store.sidebar = "outline";
      break;
    case "open-ai-paste":
      closeAgentWorkspace();
      aiPasteText.value = currentEditorSelectionText() || agentPlan.value?.revisionInstruction || agentInstruction.value;
      aiInsertMode.value = currentEditorSelectionText() ? "selection" : "section";
      openAiPaste();
      break;
    case "open-review":
      runAgentPlanReview();
      break;
    case "prepare-export":
      runAgentPlanDistribution();
      break;
    case "open-exports":
      closeAgentWorkspace();
      store.sidebar = "exports";
      break;
  }
}
function openGuidedDemo(stepId = "ai-create") {
  const stepIndex = guidedDemoSteps.value.findIndex((step) => step.id === stepId);
  guidedDemoStepIndex.value = stepIndex >= 0 ? stepIndex : 0;
  guidedDemoOpen.value = true;
  store.statusMessage = "Started guided product demo";
}
function closeGuidedDemo() {
  guidedDemoOpen.value = false;
}
function selectGuidedDemoStep(index: number) {
  guidedDemoStepIndex.value = Math.min(Math.max(index, 0), Math.max(0, guidedDemoSteps.value.length - 1));
}
function previousGuidedDemoStep() {
  selectGuidedDemoStep(guidedDemoStepIndex.value - 1);
}
function nextGuidedDemoStep() {
  selectGuidedDemoStep(guidedDemoStepIndex.value + 1);
}
function guidedDemoStepIsComplete(stepId: string) {
  return store.guidedDemoCompletedStepIds.includes(stepId);
}
function markGuidedDemoStepComplete(stepId: string) {
  store.recordGuidedDemoStepComplete(stepId);
  store.statusMessage = `Marked guided demo step complete: ${stepId}`;
}
function resetGuidedDemoProgress() {
  store.resetGuidedDemoProgress();
  store.statusMessage = "Reset guided demo progress";
}
function guidedDemoTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}
async function runGuidedDemoStep(step: GuidedDemoStep) {
  markGuidedDemoStepComplete(step.id);
  closeGuidedDemo();
  await nextTick();
  void step.run();
}
function guidedDemoChecklistMarkdown() {
  const generatedAt = new Date().toISOString();
  return [
    "## NEditor Guided Demo Checklist",
    "",
    `Generated: ${generatedAt}`,
    `Progress: ${guidedDemoCompletedCount.value}/${guidedDemoSteps.value.length} (${guidedDemoCompletionPercent.value}%)`,
    "",
    "| Done | Capability | Surface | Evidence to inspect |",
    "| --- | --- | --- | --- |",
    ...guidedDemoSteps.value.map((step) => {
      const done = guidedDemoStepIsComplete(step.id) ? "x" : " ";
      return `| [${done}] | ${guidedDemoTableCell(step.title)} | ${guidedDemoTableCell(step.mode)} | ${guidedDemoTableCell(step.points.join("; "))} |`;
    }),
    "",
    "### Trainer Notes",
    "",
    "- Complete every step before onboarding a team to AI-first document creation.",
    "- Confirm provider outputs remain needs-review until a human accepts them.",
    "- Confirm export readiness is run before distributing external deliverables.",
    "",
  ].join("\n");
}
function insertGuidedDemoChecklist() {
  insertBlock(guidedDemoChecklistMarkdown());
  store.updateText(editorView?.state.doc.toString() || active.value.text);
  store.sidebar = "review";
  store.statusMessage = "Inserted guided demo checklist";
}
async function copyGuidedDemoChecklist() {
  const checklist = guidedDemoChecklistMarkdown();
  try {
    await navigator.clipboard?.writeText(checklist);
    store.statusMessage = "Copied guided demo checklist";
  } catch {
    store.statusMessage = "Guided demo checklist is ready to copy";
  }
}
function startAiDocumentCreation() {
  if (store.mode === "outline") store.mode = "split";
  docsLiveDocumentType.value = docsLiveDocumentType.value || "business-brief";
  docsLiveDraftingDepth.value = "standard";
  docsLiveInsertMode.value = "replace";
  docsLiveTargetSection.value = null;
  if (!docsLiveContext.value.trim()) {
    docsLiveContext.value = "Describe the outcome, audience, decision needed, evidence, constraints, tone, and review expectations.";
  }
  openDocsLive();
  refreshDocsLiveQuestionnaire();
  store.statusMessage = "AI-first document creation ready in Docs Live";
}
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
  { name: "Open AI agent workspace", group: "AI", run: () => openAgentWorkspace() },
  { name: "AI: Create document", group: "AI", run: () => startAiDocumentCreation() },
  { name: "AI: Compose from outline", group: "AI", run: () => openDocsLiveFromOutline() },
  { name: "AI: Review and clean pasted text", group: "AI", run: () => openAiPaste() },
  { name: "Open Docs Live", group: "AI", run: () => openDocsLive() },
  { name: "Open Docs Live draft history", group: "AI", run: () => openDocsLiveHistory() },
  { name: "Append latest Docs Live draft", group: "AI", run: () => appendLatestDocsLiveDraft() },
  { name: "Copy latest Docs Live draft", group: "AI", run: () => void copyLatestDocsLiveDraft() },
  { name: "Insert latest Docs Live review packet", group: "AI", run: () => insertLatestDocsLiveReviewPacket() },
  { name: "Copy latest Docs Live review packet", group: "AI", run: () => void copyLatestDocsLiveReviewPacket() },
  { name: "Paste from AI chat", group: "AI", run: () => openAiPaste() },
  { name: "Open Help Center", group: "Help", run: () => openHelp() },
  { name: "Start guided demo", group: "Help", run: () => openGuidedDemo() },
  { name: "Help: AI-first composition", group: "Help", run: () => openHelp("ai-first-composition") },
  { name: "Help: Getting started", group: "Help", run: () => openHelp("getting-started") },
  { name: "Help: Docs Live", group: "Help", run: () => openHelp("docs-live") },
  { name: "Help: Export and publishing", group: "Help", run: () => openHelp("export-publishing") },
  { name: "Help: Keyboard shortcuts", group: "Help", run: () => openHelp("keyboard-shortcuts") },
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
  { name: "Open outline mode", group: "Navigate", run: () => (store.mode = "outline") },
  { name: "Plan document from outline", group: "Navigate", run: () => planDocumentOutline() },
  { name: "Fold all sections", group: "Navigate", run: () => runEditorCommand(foldAll) },
  { name: "Unfold all sections", group: "Navigate", run: () => runEditorCommand(unfoldAll) },
  { name: "Show toolbar icons and text", group: "View", run: () => (store.toolbarDisplay = "both") },
  { name: "Show toolbar icons only", group: "View", run: () => (store.toolbarDisplay = "icons") },
  { name: "Show toolbar text only", group: "View", run: () => (store.toolbarDisplay = "text") },
  { name: "Collapse all toolbars", group: "View", run: () => setAllCommandToolbarsCollapsed(true) },
  { name: "Expand all toolbars", group: "View", run: () => setAllCommandToolbarsCollapsed(false) },
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
const commandAgentInstructionAvailable = computed(() => {
  const query = commandQuery.value.trim();
  if (query.length < 8) return false;
  return /\b(ai|agent|create|draft|write|revise|edit|review|summari[sz]e|publish|export|prepare|make|turn|improve|humanize|outline|compose)\b/i.test(query);
});
const commandAgentPlanPreview = computed(() => {
  const instruction = commandQuery.value.trim();
  if (!commandAgentInstructionAvailable.value) return null;
  return buildAgenticWorkflowPlan({
    instruction,
    documentTitle: active.value.compile?.semantic.title || active.value.title,
    documentText: active.value.text,
    selectedText: currentEditorSelectionText(),
  });
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
    case "neditor-mode-outline":
      store.mode = "outline";
      store.sidebar = "outline";
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
    case "neditor-open-docs-live":
      openDocsLive();
      break;
    case "neditor-open-agent-workspace":
      openAgentWorkspace();
      break;
    case "neditor-ai-create-document":
      startAiDocumentCreation();
      break;
    case "neditor-clean-ai-paste":
      openAiPaste();
      break;
    case "neditor-open-help":
      openHelp();
      break;
    case "neditor-guided-demo":
      openGuidedDemo();
      break;
    case "neditor-help-getting-started":
      openHelp("getting-started");
      break;
    case "neditor-help-docs-live":
      openHelp("docs-live");
      break;
    case "neditor-help-exports":
      openHelp("export-publishing");
      break;
    case "neditor-help-shortcuts":
      openHelp("keyboard-shortcuts");
      break;
  }
}

async function installDesktopWorkflowTestHooks() {
  const enabled = await invoke<boolean>("desktop_workflow_smoke_enabled").catch(() => false);
  if (!enabled) return;
  window.__NEDITOR_DESKTOP_WORKFLOW__ = {
    activeDocumentPath: () => active.value.path,
    activeDocumentText: () => active.value.text,
    activeDocumentTitle: () => active.value.title,
  };
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
  void installDesktopWorkflowTestHooks();
  window.addEventListener("keydown", handleShortcut);
  window.addEventListener("mouseover", handleButtonHelpEnter);
  window.addEventListener("focusin", handleButtonHelpEnter);
  window.addEventListener("mouseout", handleButtonHelpLeave);
  window.addEventListener("focusout", handleButtonHelpLeave);
  window.addEventListener("scroll", hideButtonHelp, true);
});

onBeforeUnmount(() => {
  recordActiveScrollPosition(true);
  editorView?.destroy();
  previewTextCommit.cancel();
  window.clearTimeout(autosaveHandle);
  window.clearTimeout(autoSnapshotHandle);
  window.clearTimeout(scrollPersistHandle);
  window.removeEventListener("keydown", handleShortcut);
  window.removeEventListener("mouseover", handleButtonHelpEnter);
  window.removeEventListener("focusin", handleButtonHelpEnter);
  window.removeEventListener("mouseout", handleButtonHelpLeave);
  window.removeEventListener("focusout", handleButtonHelpLeave);
  window.removeEventListener("scroll", hideButtonHelp, true);
  delete window.__NEDITOR_DESKTOP_WORKFLOW__;
  unlistenNativeMenuCommand?.();
  unlistenNativeMenuCommand = null;
  stopDocsLiveDictation();
  stopPaneResize();
});

watch(aiPasteOpen, (open) => handleModalStateChange(open, aiPasteDialog));
watch(agentWorkspaceOpen, (open) => handleModalStateChange(open, agentWorkspaceDialog));
watch(docsLiveOpen, (open) => handleModalStateChange(open, docsLiveDialog));
watch(guidedDemoOpen, (open) => handleModalStateChange(open, guidedDemoDialog));
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
    syncingEditorFromStore = true;
    try {
      editorView.dispatch({
        changes: { from: 0, to: editorView.state.doc.length, insert: text },
      });
    } finally {
      syncingEditorFromStore = false;
    }
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
    store.toolbarCollapsedRows.join("|"),
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
  async (mode) => {
    if (mode === "export") {
      store.sidebar = "exports";
    } else if (mode === "review") {
      store.sidebar = "review";
    } else if (mode === "outline") {
      store.sidebar = "outline";
    } else if (mode === "presentation") {
      store.sidebar = "outline";
    }
    if (["split", "source", "focus"].includes(mode)) {
      await nextTick();
      syncEditorViewFromActiveDocument();
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
      toolbarCollapsedRows: store.toolbarCollapsedRows,
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
  const enabled = await invoke<boolean>("desktop_workflow_smoke_autorun_enabled").catch(() => false);
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
    const editorErgonomicsEvidence = await collectNativeEditorErgonomicsEvidence(record);
    smokePhase = "editor-ergonomics";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, editorErgonomicsEvidence });
    const outlineNavigationEvidence = await collectNativeOutlineNavigationEvidence(record);
    smokePhase = "outline-navigation";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, editorErgonomicsEvidence, outlineNavigationEvidence });

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
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, editorErgonomicsEvidence, outlineNavigationEvidence, exportResult, nativeMenuExportResult });
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
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, editorErgonomicsEvidence, outlineNavigationEvidence, exportResult, nativeMenuExportResult });
    const exportProfileEvidence = await collectNativeExportProfileEvidence(record);
    smokePhase = "export-profile";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, editorErgonomicsEvidence, outlineNavigationEvidence, exportResult, nativeMenuExportResult, exportProfileEvidence });
    smokePhase = "theme-accessibility-start";
    await writeNativeWorkflowProgress(smokePhase, assertions, { fileWorkflow, snapshotEvidence, modeEvidence, editorErgonomicsEvidence, outlineNavigationEvidence, exportResult, nativeMenuExportResult, exportProfileEvidence });
    const themeAccessibility = await collectNativeThemeAccessibilityEvidence(record);
    smokePhase = "theme-accessibility";
    await writeNativeWorkflowProgress(smokePhase, assertions, {
      fileWorkflow,
      snapshotEvidence,
      modeEvidence,
      editorErgonomicsEvidence,
      outlineNavigationEvidence,
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
      editorErgonomicsEvidence,
      outlineNavigationEvidence,
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
  const visibleText = (selector: string) => document.querySelector(selector)?.textContent?.replace(/\s+/g, " ").trim() || "";
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

  await runMenuCommand("neditor-open-docs-live", "native-menu-command-docs-live");
  await waitForNativeWorkflowCondition(() => docsLiveOpen.value, 1000);
  const docsLiveOpened = docsLiveOpen.value;
  docsLiveDocumentType.value = "proposal";
  docsLiveTitle.value = "Native Docs Live Proposal";
  docsLiveDraftingDepth.value = "detailed";
  docsLiveOutlineText.value = "- Executive Summary\n- Recommendation\n- Review Plan";
  docsLiveTranscript.value = "Create a native desktop proposal draft from the outline and prepare it for review.";
  docsLiveContext.value = "The document should renew an enterprise contract, name the executive audience, and include QA and humanization steps.";
  docsLivePlaceholderText.value = "client: Native Acme\naudience: executive team\nowner: Desktop workflow\ndeadline: June 1";
  docsLiveQuestionnaireAnswerText.value =
    "1. The reader should approve the renewal path.\n2. Include commercial evidence, timeline risk, and a named reviewer.\n3. Keep pricing assumptions marked for review.";
  generateDocsLiveDraft();
  await nextTick();
  const generated = {
    markdown: docsLiveGeneratedMarkdown.value.includes("## Drafting Plan"),
    workflow: Boolean(docsLiveDraft.value?.workflow.some((step) => step.id === "humanize")),
    sections: docsLiveDraft.value?.sections.length || 0,
    previewVisible: Boolean(document.querySelector('[aria-label="Docs Live section drafting workflow"]')),
  };
  record(
    "native workflow generated Docs Live section draft from native writing tools menu",
    generated.markdown && generated.workflow && generated.sections === 3 && generated.previewVisible,
    JSON.stringify(generated),
  );
  applyDocsLiveDraft();
  await store.compileActive();
  await nextTick();
  const applied = {
    sidebar: store.sidebar,
    title: active.value.compile?.semantic.title || "",
    hasDraftingPlan: active.value.text.includes("## Drafting Plan"),
    hasSectionQa: active.value.text.includes("### Section QA"),
    hasReviewPreparation: active.value.text.includes("## Review Preparation"),
    hasHumanizeWorkflow: active.value.text.includes("workflow: outline-to-section-draft-qa-humanize-review"),
    previewText: visibleText("#live-preview").slice(0, 240),
  };
  evidence.docsLive = {
    open: docsLiveOpened,
    speechStatus: docsLiveSpeechStatus.value,
    title: docsLiveTitle.value,
    generated,
    applied,
  };
  record("native workflow opened Docs Live from native writing tools menu", docsLiveOpened, JSON.stringify(evidence.docsLive));
  record(
    "native workflow applied Docs Live section draft for review",
    applied.sidebar === "review" &&
      applied.title === "Native Docs Live Proposal" &&
      applied.hasDraftingPlan &&
      applied.hasSectionQa &&
      applied.hasReviewPreparation &&
      applied.hasHumanizeWorkflow,
    JSON.stringify(applied),
  );
  await checkpoint?.("native-menu-command-docs-live-recorded");

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
  const modes: Array<typeof store.mode> = ["split", "source", "preview", "focus", "outline", "export", "review", "presentation"];
  type NativeModeEvidence = {
    mode: typeof store.mode;
    workspaceClass: string;
    sidebar: string;
    sourceVisible: boolean;
    previewVisible: boolean;
    sidebarText: string;
    previewText: string;
    outlineVisible: boolean;
    outlineText: string;
    outlineTitles: string[];
  };
  const expectedSidebar: Partial<Record<typeof store.mode, string>> = {
    outline: "outline",
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
      const outlineText = surfaceText("#outline-mode").slice(0, 1400);
      const outlineVisible = surfaceVisible("#outline-mode");
      const outlineTitles = Array.from(document.querySelectorAll<HTMLInputElement>("#outline-mode .outline-mode-row input")).map((input) => input.value);
      const entry = { mode, workspaceClass, sidebar, sourceVisible, previewVisible, sidebarText, previewText, outlineVisible, outlineText, outlineTitles };
      const passed = workspaceClass.includes(`mode-${mode}`) && (!expectedSidebar[mode] || sidebar === expectedSidebar[mode]);
      record(`native workflow switched ${mode} mode`, passed, JSON.stringify(entry));
      evidence.push(entry);
    }
    const byMode = (mode: typeof store.mode) => evidence.find((entry) => entry.mode === mode);
    const exportMode = byMode("export");
    const reviewMode = byMode("review");
    const outlineMode = byMode("outline");
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
      "native workflow rendered outline mode structure only",
      Boolean(
        outlineMode?.outlineVisible &&
          !outlineMode.sourceVisible &&
          !outlineMode.previewVisible &&
          outlineMode.outlineTitles.includes("Market Entry Report") &&
          outlineMode.outlineTitles.includes("Executive Summary") &&
          outlineMode.outlineText.includes("Add heading"),
      ),
      JSON.stringify(outlineMode),
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

async function collectNativeEditorErgonomicsEvidence(record: (name: string, passed: boolean, detail?: string) => void) {
  const original = {
    text: active.value.text,
    wordWrap: store.wordWrap,
    lineNumbers: store.lineNumbers,
    codeFolding: store.codeFolding,
  };
  const evidence: Record<string, unknown> = {};
  try {
    await setNativeWorkflowText(
      [
        "---",
        "title: Native Editor Ergonomics",
        "status: draft",
        "---",
        "",
        "# Native Editor Ergonomics",
        "",
        "Find target Acme should be replaced from the native smoke.",
        "",
        "## Metrics",
        "",
        "- First item",
      ].join("\n"),
    );
    await store.compileActive();
    store.wordWrap = true;
    store.lineNumbers = true;
    store.codeFolding = true;
    await nextTick();
    await nextTick();

    const editorContent = document.querySelector(".cm-content") as HTMLElement | null;
    const wordStats = document.querySelector(".word-stats")?.textContent?.replace(/\s+/g, " ").trim() || "";
    evidence.settings = {
      wordWrapEnabled: editorContent?.classList.contains("cm-lineWrapping") || false,
      lineNumbersVisible: document.querySelectorAll(".cm-lineNumbers").length > 0,
      foldGutterVisible: document.querySelectorAll(".cm-foldGutter").length > 0,
      spellcheck: editorContent?.getAttribute("spellcheck") || "",
      autocapitalize: editorContent?.getAttribute("autocapitalize") || "",
      role: editorContent?.getAttribute("role") || "",
      ariaLabel: editorContent?.getAttribute("aria-label") || "",
      wordStats,
    };
    record(
      "native workflow reported editor word statistics",
      wordStats.includes("words") && wordStats.includes("characters") && wordStats.includes("min read"),
      wordStats,
    );
    record(
      "native workflow exposed spellcheck editor attributes",
      Boolean(
          (evidence.settings as { spellcheck: string }).spellcheck === "true" &&
          (evidence.settings as { autocapitalize: string }).autocapitalize === "sentences" &&
          (evidence.settings as { role: string }).role === "textbox" &&
          (evidence.settings as { ariaLabel: string }).ariaLabel.includes("Markdown"),
      ),
      JSON.stringify(evidence.settings),
    );
    record(
      "native workflow rendered line numbers word wrap and folding gutter",
      Boolean(
        (evidence.settings as { wordWrapEnabled: boolean }).wordWrapEnabled &&
          (evidence.settings as { lineNumbersVisible: boolean }).lineNumbersVisible &&
          (evidence.settings as { foldGutterVisible: boolean }).foldGutterVisible,
      ),
      JSON.stringify(evidence.settings),
    );

    runEditorCommand(openSearchPanel);
    await nextTick();
    const searchPanel = document.querySelector(".cm-search");
    if (editorView) {
      const text = editorView.state.doc.toString();
      const from = text.indexOf("Acme");
      if (from >= 0) {
        editorView.dispatch({ changes: { from, to: from + "Acme".length, insert: "Globex" } });
        flushEditorTextToStore();
      }
    }
    await nextTick();
    evidence.searchReplace = {
      searchPanelOpen: Boolean(searchPanel),
      containsReplacement: active.value.text.includes("Find target Globex"),
      containsOriginal: active.value.text.includes("Acme"),
    };
    record(
      "native workflow opened editor search panel",
      (evidence.searchReplace as { searchPanelOpen: boolean }).searchPanelOpen,
      JSON.stringify(evidence.searchReplace),
    );
    record(
      "native workflow replaced editor search target",
      Boolean(
        (evidence.searchReplace as { containsReplacement: boolean }).containsReplacement &&
          !(evidence.searchReplace as { containsOriginal: boolean }).containsOriginal,
      ),
      JSON.stringify(evidence.searchReplace),
    );

    await setNativeWorkflowText("- First item");
    if (editorView) {
      editorView.dispatch({ selection: { anchor: editorView.state.doc.length } });
      continueMarkdownList(editorView);
      editorView.dispatch({ changes: { from: editorView.state.selection.main.head, insert: "Second item" } });
      flushEditorTextToStore();
    }
    await nextTick();
    evidence.listContinuation = { text: active.value.text };
    record(
      "native workflow continued markdown list in editor",
      active.value.text.includes("- First item\n- Second item"),
      JSON.stringify(evidence.listContinuation),
    );

    await setNativeWorkflowText("");
    if (editorView) {
      editorView.focus();
      const paired = insertBracket(editorView.state, "(");
      if (paired) {
        editorView.dispatch(paired);
        flushEditorTextToStore();
      }
    }
    await nextTick();
    evidence.pairing = { text: active.value.text };
    record("native workflow inserted paired bracket in editor", active.value.text.includes("()"), JSON.stringify(evidence.pairing));

    await setNativeWorkflowText("Alpha\nBeta");
    if (editorView) {
      editorView.dispatch({
        selection: EditorSelection.create([EditorSelection.cursor(0), EditorSelection.cursor(6)]),
      });
      const transaction = editorView.state.changeByRange((range) => ({
        changes: { from: range.from, to: range.to, insert: "Native " },
        range: EditorSelection.cursor(range.from + "Native ".length),
      }));
      editorView.dispatch(transaction);
      flushEditorTextToStore();
    }
    await nextTick();
    evidence.multiCursor = {
      text: active.value.text,
      inserted: active.value.text.includes("Native Alpha") && active.value.text.includes("Native Beta"),
    };
    record(
      "native workflow edited multiple cursors in editor",
      (evidence.multiCursor as { inserted: boolean }).inserted,
      JSON.stringify(evidence.multiCursor),
    );
    return evidence;
  } finally {
    store.wordWrap = original.wordWrap;
    store.lineNumbers = original.lineNumbers;
    store.codeFolding = original.codeFolding;
    await setNativeWorkflowText(original.text);
    await store.compileActive();
    await nextTick();
  }
}

async function collectNativeOutlineNavigationEvidence(record: (name: string, passed: boolean, detail?: string) => void) {
  const original = {
    text: active.value.text,
    mode: store.mode,
    sidebar: store.sidebar,
  };
  const evidence: Record<string, unknown> = {};
  try {
    await setNativeWorkflowText(
      [
        "---",
        "title: Native Outline Navigation",
        "status: draft",
        "---",
        "",
        "# Native Outline Navigation",
        "",
        "Introductory section.",
        "",
        "## Native Outline Target",
        "",
        "This heading should be selected from the launched Tauri outline panel.",
        "",
        "## Native Outline Follow-up",
        "",
        "Follow-up text.",
      ].join("\n"),
    );
    await store.compileActive();
    store.mode = "split";
    store.sidebar = "outline";
    await nextTick();
    await nextTick();

    const target = outlineHeadings.value.find((heading) => heading.text === "Native Outline Target");
    const outlineButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("#document-sidebar .outline-row"));
    const targetButton = outlineButtons.find((button) => button.textContent?.replace(/\s+/g, " ").includes("Native Outline Target"));
    targetButton?.click();
    await nextTick();
    await nextTick();

    const selectionLine = editorView ? editorView.state.doc.lineAt(editorView.state.selection.main.from) : null;
    evidence.outline = {
      sidebar: store.sidebar,
      mode: store.mode,
      buttonFound: Boolean(targetButton),
      buttonLabel: targetButton?.textContent?.replace(/\s+/g, " ").trim() || "",
      targetLine: target?.line || 0,
      selectedLine: selectionLine?.number || 0,
      selectedText: selectionLine?.text || "",
      editorFocused: editorView?.hasFocus || false,
      sidebarText: document.querySelector("#document-sidebar")?.textContent?.replace(/\s+/g, " ").trim().slice(0, 600) || "",
    };
    record(
      "native workflow navigated outline heading to source",
      Boolean(
        target &&
          targetButton &&
          store.sidebar === "outline" &&
          store.mode === "split" &&
          selectionLine?.number === target.line &&
          selectionLine.text.includes("## Native Outline Target"),
      ),
      JSON.stringify(evidence.outline),
    );
    return evidence;
  } finally {
    store.mode = original.mode;
    store.sidebar = original.sidebar;
    await setNativeWorkflowText(original.text);
    await store.compileActive();
    await nextTick();
  }
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
      if (syncingEditorFromStore) return;
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

function syncEditorViewFromActiveDocument() {
  if (!editorView || editorView.state.doc.toString() === active.value.text) return;
  syncingEditorFromStore = true;
  try {
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: active.value.text },
    });
  } finally {
    syncingEditorFromStore = false;
  }
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

function buttonFromEvent(event: Event) {
  const target = event.target instanceof Element ? event.target : null;
  return target?.closest("button") as HTMLButtonElement | null;
}

function buttonHelpText(button: HTMLButtonElement) {
  const explicit = button.getAttribute("data-help") || button.getAttribute("title") || button.getAttribute("aria-label");
  const visible = button.innerText.replace(/\s+/g, " ").trim();
  const base = (explicit || visible || "Button").replace(/\s+/g, " ").trim();
  if (!button.disabled) return base;
  const disabledReason = button.getAttribute("data-disabled-help") || "This action is unavailable until the required document state is ready.";
  return `${base}. ${disabledReason}`;
}

function handleButtonHelpEnter(event: Event) {
  const button = buttonFromEvent(event);
  if (!button || button.closest(".button-help-tooltip")) return;
  const text = buttonHelpText(button);
  if (!text) return;
  const rect = button.getBoundingClientRect();
  const placement = rect.bottom + 52 < window.innerHeight ? "bottom" : "top";
  const x = Math.min(Math.max(rect.left + rect.width / 2, 96), Math.max(96, window.innerWidth - 96));
  const y = placement === "bottom" ? rect.bottom + 8 : rect.top - 8;
  buttonHelp.value = { visible: true, text, x, y, placement };
}

function handleButtonHelpLeave(event: Event) {
  const button = buttonFromEvent(event);
  if (!button) return;
  const related = "relatedTarget" in event && event.relatedTarget instanceof Node ? event.relatedTarget : null;
  if (related && button.contains(related)) return;
  hideButtonHelp();
}

function hideButtonHelp() {
  buttonHelp.value = { ...buttonHelp.value, visible: false };
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

function handleModalKeydown(kind: "ai-paste" | "agent-workspace" | "docs-live" | "guided-demo" | "command-palette" | "conflict", event: KeyboardEvent) {
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

function closeModal(kind: "ai-paste" | "agent-workspace" | "docs-live" | "guided-demo" | "command-palette" | "conflict") {
  if (kind === "ai-paste") {
    closeAiPaste();
  } else if (kind === "agent-workspace") {
    closeAgentWorkspace();
  } else if (kind === "docs-live") {
    closeDocsLive();
  } else if (kind === "guided-demo") {
    closeGuidedDemo();
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

function planDocumentOutline() {
  store.sidebar = "outline";
  if (!outlineDraftTitle.value.trim()) outlineDraftTitle.value = active.value.title.replace(/\.[^.]+$/, "");
  void nextTick(() => {
    document.querySelector<HTMLTextAreaElement>('[aria-label="Editable document outline"]')?.focus();
  });
}

function loadOutlineDraftFromDocument() {
  const draft = outlinePlanFromMarkdown(active.value.text);
  outlineDraftText.value = draft || outlineDraftText.value;
  outlineDraftTitle.value = active.value.compile?.semantic.title || active.value.title.replace(/\.[^.]+$/, "");
  store.statusMessage = draft ? "Loaded outline from document headings" : "No headings found to load into outline planner";
}

function createDocumentFromOutline() {
  const markdown = outlinePlanToMarkdown(outlineDraftText.value, {
    title: outlineDraftTitle.value,
    includeToc: outlineDraftIncludeToc.value,
  });
  if (!markdown) return;
  store.updateText(markdown);
  store.sidebar = "outline";
  store.statusMessage = "Created document skeleton from outline";
}

function appendOutlineToDocument() {
  const markdown = outlinePlanToMarkdown(outlineDraftText.value, {
    title: outlineDraftTitle.value || active.value.title.replace(/\.[^.]+$/, ""),
    includeToc: false,
  });
  if (!markdown) return;
  const body = markdown.replace(/^---[\s\S]*?---\n+#[^\n]+\n+/, "").trim();
  store.updateText(`${active.value.text.trimEnd()}\n\n${body}\n`);
  store.sidebar = "outline";
  store.statusMessage = "Appended outline skeleton to document";
}

function openDocsLiveFromOutline() {
  docsLiveTargetSection.value = null;
  docsLiveOutlineText.value = outlineDraftText.value;
  docsLiveTitle.value = outlineDraftTitle.value || active.value.title.replace(/\.[^.]+$/, "");
  openDocsLive();
}

function openDocsLiveFromDocumentOutline() {
  docsLiveTargetSection.value = null;
  docsLiveOutlineText.value = outlinePlanFromMarkdown(active.value.text) || outlineDraftText.value;
  docsLiveTitle.value = active.value.compile?.semantic.title || active.value.title.replace(/\.[^.]+$/, "");
  openDocsLive();
}

function outlineHeadingKind(level: number) {
  if (level === 1) return "Chapter";
  if (level === 2) return "Section";
  if (level === 3) return "Subsection";
  return "Subsubsection";
}

function outlineHeadingMarker(level: number) {
  return "#".repeat(Math.max(1, Math.min(4, Math.trunc(level) || 1)));
}

function activeDocumentLines() {
  return active.value.text.split("\n");
}

function sectionEndLineIndex(heading: OutlineModeHeading, lines: string[]) {
  const next = outlineModeHeadings.value.find((candidate) => candidate.line > heading.line && candidate.level <= heading.level);
  return next ? Math.max(heading.line - 1, next.line - 2) : lines.length - 1;
}

function applyOutlineModeText(lines: string[], statusMessage: string) {
  previewTextCommit.cancel();
  store.updateText(lines.join("\n").replace(/\n{4,}/g, "\n\n\n"));
  store.statusMessage = statusMessage;
  void nextTick(() => syncEditorViewFromActiveDocument());
}

function renameOutlineHeading(heading: OutlineModeHeading, title: string) {
  const cleanTitle = title.trim() || "Untitled section";
  const lines = activeDocumentLines();
  const index = heading.line - 1;
  if (!lines[index]) return;
  lines[index] = `${outlineHeadingMarker(heading.level)} ${cleanTitle}`;
  applyOutlineModeText(lines, `Renamed outline heading to ${cleanTitle}`);
}

function setOutlineHeadingLevel(heading: OutlineModeHeading, level: number) {
  const nextLevel = Math.max(1, Math.min(4, Math.trunc(level) || heading.level));
  const lines = activeDocumentLines();
  const index = heading.line - 1;
  if (!lines[index]) return;
  lines[index] = `${outlineHeadingMarker(nextLevel)} ${heading.text || "Untitled section"}`;
  applyOutlineModeText(lines, `Changed ${heading.text} to ${outlineHeadingKind(nextLevel).toLowerCase()}`);
}

function createOutlineHeading(after?: OutlineModeHeading, level = outlineModeNewLevel.value) {
  const nextLevel = Math.max(1, Math.min(4, Math.trunc(level) || 1));
  const title = (after ? `New ${outlineHeadingKind(nextLevel).toLowerCase()}` : outlineModeNewTitle.value).trim() || `New ${outlineHeadingKind(nextLevel).toLowerCase()}`;
  const lines = activeDocumentLines();
  const insertAt = after ? sectionEndLineIndex(after, lines) + 1 : lines.length;
  const block = ["", `${outlineHeadingMarker(nextLevel)} ${title}`, "", "<!-- Draft this section. -->"];
  lines.splice(insertAt, 0, ...block);
  applyOutlineModeText(lines, `Added ${outlineHeadingKind(nextLevel).toLowerCase()} ${title}`);
  outlineModeNewTitle.value = nextLevel === 1 ? "New chapter" : `New ${outlineHeadingKind(nextLevel).toLowerCase()}`;
}

function deleteOutlineHeading(heading: OutlineModeHeading) {
  const lines = activeDocumentLines();
  const start = heading.line - 1;
  if (start < 0 || start >= lines.length) return;
  let end = sectionEndLineIndex(heading, lines);
  while (end + 1 < lines.length && lines[end + 1] === "") end += 1;
  lines.splice(start, Math.max(1, end - start + 1));
  applyOutlineModeText(lines, `Deleted outline section ${heading.text}`);
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

function insertCitationTodo() {
  flushEditorTextToStore();
  insertBlock(citationTodoComment(citationTodoNote.value));
  store.updateText(editorView?.state.doc.toString() || active.value.text);
  store.statusMessage = "Inserted citation TODO";
}

function citationTodoReference() {
  const value = citationTodoKey.value.trim();
  if (!value) return "";
  return /^\[\s*@/.test(value) ? value : citationReferenceSnippet(value);
}

function resolveCitationTodoItem(todo: CitationTodoItem) {
  flushEditorTextToStore();
  const reference = citationTodoReference();
  if (!reference) return;
  store.updateText(resolveCitationTodo(active.value.text, todo, reference, citationTodoNote.value));
  store.statusMessage = `Resolved citation TODO on line ${todo.line}`;
}

function deferCitationTodoItem(todo: CitationTodoItem) {
  flushEditorTextToStore();
  store.updateText(deferCitationTodo(active.value.text, todo, citationTodoNote.value));
  store.statusMessage = `Deferred citation TODO on line ${todo.line}`;
}

function insertCitationTodoAudit() {
  flushEditorTextToStore();
  insertBlock(citationTodoAuditMarkdown(citationTodoItems.value));
  store.updateText(editorView?.state.doc.toString() || active.value.text);
  store.statusMessage = "Inserted citation TODO audit";
}

async function copyCitationTodoAudit() {
  flushEditorTextToStore();
  const audit = citationTodoAuditMarkdown(citationTodoItems.value);
  try {
    await navigator.clipboard?.writeText(audit);
    store.statusMessage = "Copied citation TODO audit";
  } catch {
    store.statusMessage = "Citation TODO audit is ready to copy";
  }
}

function goToCitationTodo(todo: CitationTodoItem) {
  void goToSourceTarget({ line: todo.line, column: todo.column, end_column: todo.column + todo.marker.length });
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

function openDocsLive() {
  flushEditorTextToStore();
  if (!docsLiveTitle.value.trim()) docsLiveTitle.value = active.value.compile?.semantic.title || active.value.title.replace(/\.[^.]+$/, "");
  if (!docsLiveOutlineText.value.trim()) {
    docsLiveOutlineText.value = outlinePlanFromMarkdown(active.value.text) || outlineDraftText.value;
  }
  refreshDocsLiveQuestionnaire();
  docsLiveOpen.value = true;
  docsLiveSpeechStatus.value = docsLiveSpeechAvailable.value ? "Voice ready" : "Voice unavailable in this WebView";
}

function closeDocsLive() {
  stopDocsLiveDictation();
  docsLiveOpen.value = false;
  docsLiveInterimTranscript.value = "";
  docsLiveTargetSection.value = null;
  if (docsLiveInsertMode.value === "section") docsLiveInsertMode.value = "append";
}

async function checkDocsLiveRuntime() {
  docsLiveRuntimeChecking.value = true;
  try {
    docsLiveRuntimeReport.value = await inspectAiRuntimeReadiness({
      readClipboard: async () => {
        const clipboardText = await readClipboardText();
        return clipboardText ? { kind: clipboardText.kind, length: clipboardText.text.length } : null;
      },
    });
    docsLiveSpeechStatus.value = docsLiveRuntimeReport.value.issues.length
      ? `Runtime check found ${docsLiveRuntimeReport.value.issues.length} issue(s)`
      : "AI runtime ready";
    store.statusMessage = "Checked AI runtime readiness";
  } finally {
    docsLiveRuntimeChecking.value = false;
  }
}

function refreshDocsLiveQuestionnaire() {
  docsLiveQuestionnaireText.value = buildDocsLiveQuestionnaire(docsLiveDocumentType.value, {
    title: docsLiveTitle.value,
    outline: docsLiveOutlineText.value,
    context: docsLiveContext.value,
    transcript: docsLiveTranscript.value,
    placeholders: docsLivePlaceholderText.value,
  });
}

function addDocsLivePlaceholder() {
  docsLivePlaceholderText.value = upsertDocsLivePlaceholder(
    docsLivePlaceholderText.value,
    docsLivePlaceholderKey.value,
    docsLivePlaceholderDraftValue.value,
    {
      kind: docsLivePlaceholderDraftKind.value,
      source: docsLivePlaceholderDraftSource.value,
      reviewStatus: docsLivePlaceholderDraftStatus.value,
    },
  );
  docsLivePlaceholderKey.value = "";
  docsLivePlaceholderDraftValue.value = "";
  docsLivePlaceholderDraftKind.value = "text";
  docsLivePlaceholderDraftSource.value = "";
  docsLivePlaceholderDraftStatus.value = "provided";
  refreshDocsLiveQuestionnaire();
  store.statusMessage = "Added Docs Live placeholder value";
}

function docsLivePlaceholderValue(key: string) {
  return docsLivePlaceholderRows.value.find((entry) => entry.key === key)?.value || "";
}

function updateDocsLiveIntentField(key: string, value: string) {
  docsLivePlaceholderText.value = value.trim()
    ? upsertDocsLivePlaceholder(docsLivePlaceholderText.value, key, value)
    : removeDocsLivePlaceholder(docsLivePlaceholderText.value, key);
  refreshDocsLiveQuestionnaire();
  store.statusMessage = `Updated ${key} intent value`;
}

function updateDocsLivePlaceholder(key: string, value: string) {
  const existing = docsLivePlaceholderRows.value.find((entry) => entry.key === key);
  docsLivePlaceholderText.value = upsertDocsLivePlaceholder(docsLivePlaceholderText.value, key, value, {
    kind: existing?.kind,
    source: existing?.source,
    reviewStatus: existing?.reviewStatus,
  });
  refreshDocsLiveQuestionnaire();
  store.statusMessage = `Updated ${key} placeholder value`;
}

function updateDocsLivePlaceholderMetadata(key: string, metadata: Partial<Pick<DocsLivePlaceholderEntry, "kind" | "source" | "reviewStatus">>) {
  const existing = docsLivePlaceholderRows.value.find((entry) => entry.key === key);
  if (!existing) return;
  docsLivePlaceholderText.value = upsertDocsLivePlaceholder(docsLivePlaceholderText.value, key, existing.value, {
    kind: (metadata.kind as DocsLivePlaceholderKind | undefined) || existing.kind,
    source: metadata.source ?? existing.source,
    reviewStatus: (metadata.reviewStatus as DocsLivePlaceholderReviewStatus | undefined) || existing.reviewStatus,
  });
  refreshDocsLiveQuestionnaire();
  store.statusMessage = `Updated ${key} placeholder metadata`;
}

function updateDocsLivePlaceholderKind(key: string, kind: string) {
  const allowed = docsLivePlaceholderKindOptions.some((option) => option.value === kind);
  updateDocsLivePlaceholderMetadata(key, { kind: allowed ? (kind as DocsLivePlaceholderKind) : "text" });
}

function updateDocsLivePlaceholderReviewStatus(key: string, reviewStatus: string) {
  const allowed = docsLivePlaceholderReviewStatusOptions.some((option) => option.value === reviewStatus);
  updateDocsLivePlaceholderMetadata(key, {
    reviewStatus: allowed ? (reviewStatus as DocsLivePlaceholderReviewStatus) : "provided",
  });
}

function removeDocsLivePlaceholderValue(key: string) {
  docsLivePlaceholderText.value = removeDocsLivePlaceholder(docsLivePlaceholderText.value, key);
  refreshDocsLiveQuestionnaire();
  store.statusMessage = `Removed ${key} placeholder value`;
}

function loadDocsLiveOutlineFromDocument() {
  docsLiveOutlineText.value = outlinePlanFromMarkdown(active.value.text) || outlineDraftText.value;
  docsLiveTitle.value = active.value.compile?.semantic.title || active.value.title.replace(/\.[^.]+$/, "");
  store.statusMessage = docsLiveOutlineText.value.trim() ? "Loaded document outline for Docs Live" : "No outline found for Docs Live";
}

function generateDocsLiveDraft() {
  const draft = buildDocsLiveDraft({
    documentType: docsLiveDocumentType.value,
    title: docsLiveTitle.value,
    outline: docsLiveOutlineText.value,
    transcript: docsLiveTranscript.value,
    context: docsLiveContext.value,
    questionnaireAnswers: docsLiveQuestionnaireAnswerText.value,
    placeholders: docsLivePlaceholderText.value,
    draftingDepth: docsLiveDraftingDepth.value,
  });
  docsLiveDraft.value = draft;
  docsLiveGeneratedMarkdown.value = draft.markdown;
  docsLiveOutlineText.value = draft.outlineText;
  docsLiveTitle.value = draft.title;
  store.recordDocsLiveDraftHistory(docsLiveDraftHistoryItem(draft));
  store.statusMessage = `Docs Live generated ${draft.sections.length} section draft with QA and humanization`;
}

function applyDocsLiveDraft() {
  if (!docsLiveGeneratedMarkdown.value.trim()) return;
  const markdown = docsLiveGeneratedMarkdown.value;
  if (docsLiveInsertMode.value === "selection" && editorView) {
    const range = editorView.state.selection.main;
    editorView.dispatch({
      changes: { from: range.from, to: range.to, insert: markdown },
      selection: { anchor: range.from + markdown.length },
    });
    store.updateText(editorView.state.doc.toString());
    editorView.focus();
  } else if (docsLiveInsertMode.value === "append") {
    store.updateText(`${active.value.text.trimEnd()}\n\n${markdown}`);
  } else if (docsLiveInsertMode.value === "section") {
    const target = docsLiveTargetSection.value;
    const fallbackSection = docsLiveDraft.value?.sections[0];
    const heading = target?.heading || fallbackSection?.title || docsLiveTitle.value;
    const preferredLevel = target ? target.level + 1 : fallbackSection ? fallbackSection.level + 1 : undefined;
    const nextText = replaceOrAppendMarkdownSection(active.value.text, markdown, heading, preferredLevel);
    store.updateText(nextText);
  } else {
    store.updateText(markdown);
  }
  store.sidebar = "review";
  store.statusMessage = docsLiveInsertMode.value === "section"
    ? `Applied Docs Live draft to ${docsLiveTargetSection.value?.heading || docsLiveDraft.value?.sections[0]?.title || "matching section"} for review`
    : "Applied Docs Live draft for review";
  closeDocsLive();
}

function appendDocsLiveDraftForReview() {
  if (!docsLiveGeneratedMarkdown.value.trim()) return;
  store.updateText(`${active.value.text.trimEnd()}\n\n${docsLiveGeneratedMarkdown.value}`);
  store.sidebar = "review";
  store.statusMessage = "Appended Docs Live draft for review";
}

async function copyDocsLiveDraft() {
  if (!docsLiveGeneratedMarkdown.value.trim()) return;
  try {
    await navigator.clipboard?.writeText(docsLiveGeneratedMarkdown.value);
    store.statusMessage = "Copied Docs Live draft";
  } catch {
    store.statusMessage = "Docs Live draft is ready to copy";
  }
}

function docsLiveDraftHistoryItem(draft: DocsLiveDraft): DocsLiveDraftHistoryItem {
  const generatedAt = new Date().toISOString();
  const outputFingerprint = stableFingerprint(draft.markdown);
  const reviewPacketMarkdown = docsLiveReviewPacketMarkdownFor(draft, generatedAt);
  return {
    draftId: `docs-live-${outputFingerprint.slice(0, 16)}`,
    title: draft.title,
    generatedAt,
    updatedAt: generatedAt,
    documentType: draft.documentType,
    sectionCount: draft.sections.length,
    issueCount: draft.issues.length,
    outlineText: draft.outlineText,
    instruction: docsLiveAuditInline(
      [docsLiveContext.value, docsLiveTranscript.value, docsLiveQuestionnaireAnswerText.value].filter(Boolean).join("\n\n"),
    ).slice(0, 4_000),
    markdown: draft.markdown,
    markdownPreview: docsLiveHistoryPreview(draft.markdown),
    reviewPacketMarkdown,
    reviewPacketPreview: docsLiveHistoryPreview(reviewPacketMarkdown),
    outputFingerprint,
  };
}

function docsLiveHistoryPreview(value: string) {
  return (
    value
      .replace(/```[\s\S]*?```/g, " ")
      .replace(/[#>*_`[\]-]/g, " ")
      .replace(/\s+/g, " ")
      .trim()
      .slice(0, 220) || "No preview captured."
  );
}

function docsLiveReviewPacketMarkdown() {
  const draft = docsLiveDraft.value;
  return draft ? docsLiveReviewPacketMarkdownFor(draft, new Date().toISOString()) : "";
}

function docsLiveReviewPacketMarkdownFor(draft: DocsLiveDraft, generatedAt: string) {
  const packet = draft.reviewPacket;
  const lines = [
    "## Docs Live Review Packet",
    "",
    "```ai-audit",
    "type: docs-live-review-packet",
    `generatedAt: ${generatedAt}`,
    `title: ${docsLiveAuditInline(draft.title)}`,
    `documentType: ${docsLiveAuditInline(draft.documentType)}`,
    `sections: ${draft.sections.length}`,
    "source: NEditor Docs Live",
    "```",
    "",
    "### Context Package",
    "",
    ...packet.contextSources.map((source) => `- ${docsLiveAuditInline(source)}`),
    "",
    "### Section Work Queue",
    "",
    ...packet.sectionRunbook.map((item) => `- ${docsLiveAuditInline(item)}`),
    "",
    "### Assumption Register",
    "",
    ...packet.qaRegister.map((item) => `- [ ] ${docsLiveAuditInline(item)}`),
    "",
    "### Humanization Checklist",
    "",
    ...packet.humanizationChecklist.map((item) => `- [ ] ${docsLiveAuditInline(item)}`),
    "",
    "### Reviewer Handoff",
    "",
    ...packet.reviewerHandoff.map((item) => `- [ ] ${docsLiveAuditInline(item)}`),
  ];
  return lines.join("\n");
}

function docsLiveAuditInline(value: string) {
  return (value || "").replace(/\r?\n/g, " ").trim();
}

function appendDocsLiveHistoryDraft(item: DocsLiveDraftHistoryItem) {
  if (!item.markdown.trim()) return;
  store.updateText(`${active.value.text.trimEnd()}\n\n${item.markdown}`);
  store.sidebar = "review";
  store.statusMessage = `Appended saved Docs Live draft ${item.title}`;
}

async function copyDocsLiveHistoryDraft(item: DocsLiveDraftHistoryItem) {
  if (!item.markdown.trim()) return;
  try {
    await navigator.clipboard?.writeText(item.markdown);
    store.statusMessage = `Copied saved Docs Live draft ${item.title}`;
  } catch {
    store.statusMessage = `Saved Docs Live draft ${item.title} is ready to copy`;
  }
}

function insertDocsLiveHistoryReviewPacket(item: DocsLiveDraftHistoryItem) {
  if (!item.reviewPacketMarkdown?.trim()) return;
  insertBlock(item.reviewPacketMarkdown);
  store.sidebar = "review";
  store.statusMessage = `Inserted saved Docs Live review packet ${item.title}`;
}

async function copyDocsLiveHistoryReviewPacket(item: DocsLiveDraftHistoryItem) {
  if (!item.reviewPacketMarkdown?.trim()) return;
  try {
    await navigator.clipboard?.writeText(item.reviewPacketMarkdown);
    store.statusMessage = `Copied saved Docs Live review packet ${item.title}`;
  } catch {
    store.statusMessage = `Saved Docs Live review packet ${item.title} is ready to copy`;
  }
}

function removeDocsLiveHistoryDraft(item: DocsLiveDraftHistoryItem) {
  store.removeDocsLiveDraftHistory(item.draftId);
  store.statusMessage = `Removed saved Docs Live draft ${item.title}`;
}

function clearDocsLiveDraftHistory() {
  store.clearDocsLiveDraftHistory();
  store.statusMessage = "Cleared saved Docs Live draft history";
}

function openDocsLiveHistory() {
  openDocsLive();
  store.statusMessage = store.docsLiveDraftHistory.length
    ? "Opened Docs Live draft history"
    : "No Docs Live draft history yet";
}

function appendLatestDocsLiveDraft() {
  const item = latestDocsLiveDraftHistory.value;
  if (!item) {
    store.statusMessage = "No saved Docs Live draft yet";
    return;
  }
  appendDocsLiveHistoryDraft(item);
}

async function copyLatestDocsLiveDraft() {
  const item = latestDocsLiveDraftHistory.value;
  if (!item) {
    store.statusMessage = "No saved Docs Live draft yet";
    return;
  }
  await copyDocsLiveHistoryDraft(item);
}

function insertLatestDocsLiveReviewPacket() {
  const item = latestDocsLiveDraftHistory.value;
  if (!item) {
    store.statusMessage = "No saved Docs Live review packet yet";
    return;
  }
  insertDocsLiveHistoryReviewPacket(item);
}

async function copyLatestDocsLiveReviewPacket() {
  const item = latestDocsLiveDraftHistory.value;
  if (!item) {
    store.statusMessage = "No saved Docs Live review packet yet";
    return;
  }
  await copyDocsLiveHistoryReviewPacket(item);
}

function insertDocsLiveReviewPacket() {
  const packet = docsLiveReviewPacketMarkdown();
  if (!packet) return;
  insertBlock(packet);
  store.sidebar = "review";
  store.statusMessage = "Inserted Docs Live review packet";
}

async function copyDocsLiveReviewPacket() {
  const packet = docsLiveReviewPacketMarkdown();
  if (!packet) return;
  try {
    await navigator.clipboard?.writeText(packet);
    store.statusMessage = "Copied Docs Live review packet";
  } catch {
    store.statusMessage = "Docs Live review packet is ready to copy";
  }
}

function toggleDocsLiveDictation() {
  if (docsLiveListening.value) {
    stopDocsLiveDictation();
  } else {
    startDocsLiveDictation();
  }
}

function startDocsLiveDictation() {
  const Recognition = speechRecognitionConstructor();
  if (!Recognition) {
    docsLiveSpeechStatus.value = "Voice unavailable in this WebView";
    return;
  }
  stopDocsLiveDictation();
  const recognition = new Recognition();
  docsLiveRecognition = recognition;
  recognition.continuous = true;
  recognition.interimResults = true;
  recognition.lang = "en-US";
  recognition.onresult = (event) => {
    const results = event.results;
    if (!results) return;
    let finalTranscript = "";
    let interimTranscript = "";
    for (let index = event.resultIndex || 0; index < results.length; index += 1) {
      const result = results[index];
      const transcript = result?.[0]?.transcript?.trim();
      if (!transcript) continue;
      if (result.isFinal) {
        finalTranscript = `${finalTranscript} ${transcript}`.trim();
      } else {
        interimTranscript = `${interimTranscript} ${transcript}`.trim();
      }
    }
    if (finalTranscript) docsLiveTranscript.value = `${docsLiveTranscript.value.trimEnd()} ${finalTranscript}`.trim();
    docsLiveInterimTranscript.value = interimTranscript;
  };
  recognition.onerror = (event) => {
    docsLiveSpeechStatus.value = event.error ? `Voice error: ${event.error}` : "Voice dictation stopped";
    docsLiveListening.value = false;
  };
  recognition.onend = () => {
    docsLiveListening.value = false;
    docsLiveInterimTranscript.value = "";
    if (docsLiveSpeechStatus.value === "Listening") docsLiveSpeechStatus.value = "Voice stopped";
  };
  try {
    recognition.start();
    docsLiveListening.value = true;
    docsLiveSpeechStatus.value = "Listening";
  } catch (error) {
    docsLiveSpeechStatus.value = error instanceof Error ? error.message : "Voice dictation could not start";
    docsLiveListening.value = false;
  }
}

function stopDocsLiveDictation() {
  const recognition = docsLiveRecognition;
  docsLiveRecognition = null;
  if (!recognition) return;
  recognition.onend = null;
  recognition.onerror = null;
  recognition.onresult = null;
  try {
    recognition.stop();
  } catch {
    recognition.abort?.();
  }
  docsLiveListening.value = false;
  docsLiveInterimTranscript.value = "";
}

function speechRecognitionConstructor(): SpeechRecognitionConstructor | null {
  if (typeof window === "undefined") return null;
  const speechWindow = window as SpeechRecognitionWindow;
  return speechWindow.SpeechRecognition || speechWindow.webkitSpeechRecognition || null;
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

async function openCommandPaletteAgentPlan() {
  const instruction = commandQuery.value.trim();
  if (!instruction) return;
  closeCommandPalette();
  await nextTick();
  openAgentWorkspace(instruction);
  store.statusMessage = "Planned agent workflow from command palette instruction";
}

async function runCommandPaletteAgentInstruction() {
  const instruction = commandQuery.value.trim();
  if (!instruction) return;
  closeCommandPalette();
  await nextTick();
  openAgentWorkspace(instruction);
  generateAgentWorkspaceRun();
  store.statusMessage = "Generated agent packet from command palette instruction";
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
  const text = editorView?.state.doc.toString() ?? active.value.text;
  const position = editorView?.state.selection.main.to ?? text.length;
  const before = text.slice(0, position);
  const after = text.slice(position);
  const block = lines.join("\n");
  const prefix = !before ? "" : before.endsWith("\n\n") ? "" : before.endsWith("\n") ? "\n" : "\n\n";
  const suffix = !after ? "\n" : after.startsWith("\n") ? "\n" : "\n\n";
  previewTextCommit.cancel();
  store.updateText(`${before}${prefix}${block}${suffix}${after}`);
  void nextTick(() => syncEditorViewFromActiveDocument());
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
  if (["preview", "export", "presentation", "outline"].includes(store.mode)) {
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
.app-shell[data-theme="dark"] .help-topic-button,
.app-shell[data-theme="dark"] .help-topic-header small,
.app-shell[data-theme="dark"] .help-keywords span,
.app-shell[data-theme="dark"] .guided-demo-progress,
.app-shell[data-theme="dark"] .guided-demo-card,
.app-shell[data-theme="dark"] .guided-demo-steps span,
.app-shell[data-theme="dark"] .agent-playbooks,
.app-shell[data-theme="dark"] .agent-source-pack-builder,
.app-shell[data-theme="dark"] .agent-source-pack-list li,
.app-shell[data-theme="dark"] .agent-playbook-grid article,
.app-shell[data-theme="dark"] .agent-plan > header,
.app-shell[data-theme="dark"] .agent-plan-grid article,
.app-shell[data-theme="dark"] .agent-missing-inputs,
.app-shell[data-theme="dark"] .agent-step-list li,
.app-shell[data-theme="dark"] .agent-missing-inputs li,
.app-shell[data-theme="dark"] .agent-run-output,
.app-shell[data-theme="dark"] .agent-control-center,
.app-shell[data-theme="dark"] .agent-control-grid article,
.app-shell[data-theme="dark"] .agent-review-comment-queue,
.app-shell[data-theme="dark"] .agent-review-comment-queue li,
.app-shell[data-theme="dark"] .agent-reviewer-agents,
.app-shell[data-theme="dark"] .agent-reviewer-grid article,
.app-shell[data-theme="dark"] .agent-section-workqueue,
.app-shell[data-theme="dark"] .agent-section-workqueue li,
.app-shell[data-theme="dark"] .agent-audit-trail,
.app-shell[data-theme="dark"] .agent-audit-grid article,
.app-shell[data-theme="dark"] .agent-release-evidence,
.app-shell[data-theme="dark"] .agent-release-evidence-grid article,
.app-shell[data-theme="dark"] .agent-history,
.app-shell[data-theme="dark"] .agent-history li,
.app-shell[data-theme="dark"] .agent-run-columns article,
.app-shell[data-theme="dark"] .agent-distribution-runbooks article,
.app-shell[data-theme="dark"] .agent-provider-panel,
.app-shell[data-theme="dark"] .agent-provider-output,
.app-shell[data-theme="dark"] .docs-live-runtime,
.app-shell[data-theme="dark"] .docs-live-intent-brief,
.app-shell[data-theme="dark"] .docs-live-placeholder-manager,
.app-shell[data-theme="dark"] .docs-live-workflow,
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
.app-shell[data-theme="dark"] .help-topic-button small,
.app-shell[data-theme="dark"] .help-topic-header p,
.app-shell[data-theme="dark"] .help-when,
.app-shell[data-theme="dark"] .help-tips,
.app-shell[data-theme="dark"] .guided-demo-modal header p,
.app-shell[data-theme="dark"] .guided-demo-modal header small,
.app-shell[data-theme="dark"] .guided-demo-progress div,
.app-shell[data-theme="dark"] .guided-demo-steps small,
.app-shell[data-theme="dark"] .guided-demo-card small,
.app-shell[data-theme="dark"] .agent-workspace-modal header p,
.app-shell[data-theme="dark"] .agent-playbooks > header span,
.app-shell[data-theme="dark"] .agent-source-pack-builder > header span,
.app-shell[data-theme="dark"] .agent-source-pack-list span,
.app-shell[data-theme="dark"] .agent-playbook-grid header span,
.app-shell[data-theme="dark"] .agent-playbook-grid dt,
.app-shell[data-theme="dark"] .agent-playbook-grid dd,
.app-shell[data-theme="dark"] .agent-plan > header span,
.app-shell[data-theme="dark"] .agent-plan > header small,
.app-shell[data-theme="dark"] .agent-step-list small,
.app-shell[data-theme="dark"] .agent-run-output > header span,
.app-shell[data-theme="dark"] .agent-run-output > header small,
.app-shell[data-theme="dark"] .agent-review-comment-queue > header span,
.app-shell[data-theme="dark"] .agent-review-comment-queue > header small,
.app-shell[data-theme="dark"] .agent-review-comment-queue small,
.app-shell[data-theme="dark"] .agent-review-comment-queue ul,
.app-shell[data-theme="dark"] .agent-reviewer-agents > header span,
.app-shell[data-theme="dark"] .agent-reviewer-agents > header small,
.app-shell[data-theme="dark"] .agent-reviewer-grid article header span,
.app-shell[data-theme="dark"] .agent-reviewer-grid ul,
.app-shell[data-theme="dark"] .agent-section-workqueue > header span,
.app-shell[data-theme="dark"] .agent-section-workqueue > header small,
.app-shell[data-theme="dark"] .agent-section-workqueue small,
.app-shell[data-theme="dark"] .agent-section-workqueue span,
.app-shell[data-theme="dark"] .agent-section-workqueue ul,
.app-shell[data-theme="dark"] .agent-release-evidence > header span,
.app-shell[data-theme="dark"] .agent-release-evidence > header small,
.app-shell[data-theme="dark"] .agent-release-evidence-grid small,
.app-shell[data-theme="dark"] .agent-history p,
.app-shell[data-theme="dark"] .agent-run-columns ul,
.app-shell[data-theme="dark"] .agent-distribution-runbooks ul,
.app-shell[data-theme="dark"] .agent-provider-panel header span,
.app-shell[data-theme="dark"] .agent-provider-output header span,
.app-shell[data-theme="dark"] .agent-provider-output ul,
.app-shell[data-theme="dark"] .docs-live-runtime header span,
.app-shell[data-theme="dark"] .docs-live-runtime li,
.app-shell[data-theme="dark"] .docs-live-workflow header span,
.app-shell[data-theme="dark"] .docs-live-section-cards span,
.app-shell[data-theme="dark"] .sidebar-hint {
  color: #aebdcc;
}

.app-shell[data-theme="dark"] .help-topic-button:hover,
.app-shell[data-theme="dark"] .help-topic-button:focus-visible,
.app-shell[data-theme="dark"] .help-topic-button.active {
  background: #203247;
}

.app-shell[data-theme="dark"] .guided-demo-steps .active button {
  background: #203247;
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
  .app-shell[data-theme="system"] .help-topic-button,
  .app-shell[data-theme="system"] .help-topic-header small,
  .app-shell[data-theme="system"] .help-keywords span,
  .app-shell[data-theme="system"] .guided-demo-progress,
  .app-shell[data-theme="system"] .guided-demo-card,
  .app-shell[data-theme="system"] .guided-demo-steps span,
  .app-shell[data-theme="system"] .agent-playbooks,
  .app-shell[data-theme="system"] .agent-source-pack-builder,
  .app-shell[data-theme="system"] .agent-source-pack-list li,
  .app-shell[data-theme="system"] .agent-playbook-grid article,
  .app-shell[data-theme="system"] .agent-plan > header,
  .app-shell[data-theme="system"] .agent-plan-grid article,
  .app-shell[data-theme="system"] .agent-missing-inputs,
  .app-shell[data-theme="system"] .agent-step-list li,
  .app-shell[data-theme="system"] .agent-missing-inputs li,
  .app-shell[data-theme="system"] .agent-run-output,
  .app-shell[data-theme="system"] .agent-control-center,
  .app-shell[data-theme="system"] .agent-control-grid article,
  .app-shell[data-theme="system"] .agent-review-comment-queue,
  .app-shell[data-theme="system"] .agent-review-comment-queue li,
  .app-shell[data-theme="system"] .agent-reviewer-agents,
  .app-shell[data-theme="system"] .agent-reviewer-grid article,
  .app-shell[data-theme="system"] .agent-section-workqueue,
  .app-shell[data-theme="system"] .agent-section-workqueue li,
  .app-shell[data-theme="system"] .agent-audit-trail,
  .app-shell[data-theme="system"] .agent-audit-grid article,
  .app-shell[data-theme="system"] .agent-release-evidence,
  .app-shell[data-theme="system"] .agent-release-evidence-grid article,
  .app-shell[data-theme="system"] .agent-history,
  .app-shell[data-theme="system"] .agent-history li,
  .app-shell[data-theme="system"] .agent-run-columns article,
  .app-shell[data-theme="system"] .agent-distribution-runbooks article,
  .app-shell[data-theme="system"] .agent-provider-panel,
  .app-shell[data-theme="system"] .agent-provider-output,
  .app-shell[data-theme="system"] .docs-live-runtime,
  .app-shell[data-theme="system"] .docs-live-intent-brief,
  .app-shell[data-theme="system"] .docs-live-placeholder-manager,
  .app-shell[data-theme="system"] .docs-live-workflow,
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
  .app-shell[data-theme="system"] .help-topic-button small,
  .app-shell[data-theme="system"] .help-topic-header p,
  .app-shell[data-theme="system"] .help-when,
  .app-shell[data-theme="system"] .help-tips,
  .app-shell[data-theme="system"] .guided-demo-modal header p,
  .app-shell[data-theme="system"] .guided-demo-modal header small,
  .app-shell[data-theme="system"] .guided-demo-progress div,
  .app-shell[data-theme="system"] .guided-demo-steps small,
  .app-shell[data-theme="system"] .guided-demo-card small,
  .app-shell[data-theme="system"] .agent-workspace-modal header p,
  .app-shell[data-theme="system"] .agent-playbooks > header span,
  .app-shell[data-theme="system"] .agent-source-pack-builder > header span,
  .app-shell[data-theme="system"] .agent-source-pack-list span,
  .app-shell[data-theme="system"] .agent-playbook-grid header span,
  .app-shell[data-theme="system"] .agent-playbook-grid dt,
  .app-shell[data-theme="system"] .agent-playbook-grid dd,
  .app-shell[data-theme="system"] .agent-plan > header span,
  .app-shell[data-theme="system"] .agent-plan > header small,
  .app-shell[data-theme="system"] .agent-step-list small,
  .app-shell[data-theme="system"] .agent-run-output > header span,
  .app-shell[data-theme="system"] .agent-run-output > header small,
  .app-shell[data-theme="system"] .agent-review-comment-queue > header span,
  .app-shell[data-theme="system"] .agent-review-comment-queue > header small,
  .app-shell[data-theme="system"] .agent-review-comment-queue small,
  .app-shell[data-theme="system"] .agent-review-comment-queue ul,
  .app-shell[data-theme="system"] .agent-reviewer-agents > header span,
  .app-shell[data-theme="system"] .agent-reviewer-agents > header small,
  .app-shell[data-theme="system"] .agent-reviewer-grid article header span,
  .app-shell[data-theme="system"] .agent-reviewer-grid ul,
  .app-shell[data-theme="system"] .agent-section-workqueue > header span,
  .app-shell[data-theme="system"] .agent-section-workqueue > header small,
  .app-shell[data-theme="system"] .agent-section-workqueue small,
  .app-shell[data-theme="system"] .agent-section-workqueue span,
  .app-shell[data-theme="system"] .agent-section-workqueue ul,
  .app-shell[data-theme="system"] .agent-release-evidence > header span,
  .app-shell[data-theme="system"] .agent-release-evidence > header small,
  .app-shell[data-theme="system"] .agent-release-evidence-grid small,
  .app-shell[data-theme="system"] .agent-history p,
  .app-shell[data-theme="system"] .agent-run-columns ul,
  .app-shell[data-theme="system"] .agent-distribution-runbooks ul,
  .app-shell[data-theme="system"] .agent-provider-panel header span,
  .app-shell[data-theme="system"] .agent-provider-output header span,
  .app-shell[data-theme="system"] .agent-provider-output ul,
  .app-shell[data-theme="system"] .docs-live-runtime header span,
  .app-shell[data-theme="system"] .docs-live-runtime li,
  .app-shell[data-theme="system"] .docs-live-workflow header span,
  .app-shell[data-theme="system"] .docs-live-section-cards span,
  .app-shell[data-theme="system"] .sidebar-hint {
    color: #aebdcc;
  }

  .app-shell[data-theme="system"] .help-topic-button:hover,
  .app-shell[data-theme="system"] .help-topic-button:focus-visible,
  .app-shell[data-theme="system"] .help-topic-button.active {
    background: #203247;
  }

  .app-shell[data-theme="system"] .guided-demo-steps .active button {
    background: #203247;
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
.app-shell[data-high-contrast="true"] .help-topic-button,
.app-shell[data-high-contrast="true"] .help-topic-header small,
.app-shell[data-high-contrast="true"] .help-keywords span,
.app-shell[data-high-contrast="true"] .guided-demo-progress,
.app-shell[data-high-contrast="true"] .guided-demo-card,
.app-shell[data-high-contrast="true"] .guided-demo-steps span,
.app-shell[data-high-contrast="true"] .agent-playbooks,
.app-shell[data-high-contrast="true"] .agent-source-pack-builder,
.app-shell[data-high-contrast="true"] .agent-source-pack-list li,
.app-shell[data-high-contrast="true"] .agent-playbook-grid article,
.app-shell[data-high-contrast="true"] .agent-plan > header,
.app-shell[data-high-contrast="true"] .agent-plan-grid article,
.app-shell[data-high-contrast="true"] .agent-missing-inputs,
.app-shell[data-high-contrast="true"] .agent-step-list li,
.app-shell[data-high-contrast="true"] .agent-missing-inputs li,
.app-shell[data-high-contrast="true"] .agent-run-output,
.app-shell[data-high-contrast="true"] .agent-control-center,
.app-shell[data-high-contrast="true"] .agent-control-grid article,
.app-shell[data-high-contrast="true"] .agent-review-comment-queue,
.app-shell[data-high-contrast="true"] .agent-review-comment-queue li,
.app-shell[data-high-contrast="true"] .agent-reviewer-agents,
.app-shell[data-high-contrast="true"] .agent-reviewer-grid article,
.app-shell[data-high-contrast="true"] .agent-section-workqueue,
.app-shell[data-high-contrast="true"] .agent-section-workqueue li,
.app-shell[data-high-contrast="true"] .agent-audit-trail,
.app-shell[data-high-contrast="true"] .agent-audit-grid article,
.app-shell[data-high-contrast="true"] .agent-release-evidence,
.app-shell[data-high-contrast="true"] .agent-release-evidence-grid article,
.app-shell[data-high-contrast="true"] .agent-history,
.app-shell[data-high-contrast="true"] .agent-history li,
.app-shell[data-high-contrast="true"] .agent-run-columns article,
.app-shell[data-high-contrast="true"] .agent-distribution-runbooks article,
.app-shell[data-high-contrast="true"] .agent-provider-panel,
.app-shell[data-high-contrast="true"] .agent-provider-output,
.app-shell[data-high-contrast="true"] .docs-live-runtime,
.app-shell[data-high-contrast="true"] .docs-live-workflow,
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
.app-shell[data-high-contrast="true"] .file-row.active,
.app-shell[data-high-contrast="true"] .help-topic-button.active {
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

.command-toolbar-row.collapsed {
  min-height: 30px;
}

.command-toolbar-row.collapsed .command-group,
.command-toolbar-row.collapsed .compact-field,
.command-toolbar-row.collapsed .compact-toolbar-toggle {
  display: none;
}

.command-toolbar-heading {
  display: inline-flex;
  align-items: center;
  justify-content: flex-start;
  gap: 5px;
  width: 92px;
  flex: 0 0 92px;
  min-height: 28px;
  padding: 0 6px;
  border: 1px solid transparent;
  background: transparent;
  color: #5b6c80;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0;
  line-height: 1;
  text-align: left;
  text-transform: uppercase;
}

.command-toolbar-heading:hover,
.command-toolbar-heading:focus-visible {
  border-color: #c7d2df;
  background: #ffffff;
}

.command-toolbar-heading svg {
  width: 13px;
  height: 13px;
  flex: 0 0 13px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 2;
}

.command-toolbar-heading span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
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

.compact-toolbar-toggle {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  min-height: 28px;
  padding: 0 7px;
  border-color: #c5d0dc;
  background: #f8fafc;
  color: #203044;
  font-size: var(--toolbar-font-size, 10px);
  font-weight: 700;
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

.workspace.mode-outline {
  grid-template-columns: minmax(0, 1fr);
}

.sidebar,
.editor-pane,
.preview-pane,
.outline-mode-pane {
  min-height: 0;
  overflow: auto;
  border-right: 1px solid #c9d2dc;
}

.outline-mode-pane {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 12px;
  padding: 16px;
  background: #f7f9fc;
}

.outline-mode-header {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #d8e1eb;
}

.outline-mode-header h2 {
  margin: 0 0 4px;
  font-size: 20px;
}

.outline-mode-header p {
  margin: 0;
  color: #526171;
}

.outline-mode-create {
  display: flex;
  align-items: end;
  flex-wrap: wrap;
  gap: 8px;
}

.outline-mode-create label {
  display: grid;
  gap: 4px;
  color: #526171;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
}

.outline-mode-create input {
  min-width: 220px;
}

.outline-mode-list {
  display: grid;
  align-content: start;
  gap: 8px;
  overflow: auto;
}

.outline-mode-row {
  display: grid;
  grid-template-columns: 112px minmax(180px, 1fr) 150px auto;
  align-items: center;
  gap: 8px;
  margin-left: calc(var(--outline-depth, 0) * 22px);
  padding: 8px;
  border: 1px solid #d9e1ea;
  border-radius: 6px;
  background: #ffffff;
}

.outline-mode-kind {
  color: #5b6c80;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0;
  text-transform: uppercase;
}

.outline-mode-row input,
.outline-mode-row select {
  width: 100%;
}

.outline-mode-actions {
  display: inline-flex;
  justify-content: flex-end;
  gap: 6px;
  white-space: nowrap;
}

.outline-mode-empty {
  display: grid;
  place-content: center;
  gap: 8px;
  min-height: 280px;
  text-align: center;
}

.outline-mode-empty h3,
.outline-mode-empty p {
  margin: 0;
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

.outline-planner {
  display: grid;
  gap: 8px;
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid #d7dee7;
}

.outline-planner textarea {
  resize: vertical;
  min-height: 136px;
  font-family: var(--editor-font, "SFMono-Regular", Consolas, monospace);
}

.outline-planner-actions {
  display: grid;
  grid-template-columns: 1fr;
  gap: 6px;
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

.help-center {
  display: grid;
  gap: 12px;
}

.help-controls {
  display: grid;
  gap: 8px;
}

.help-quick-actions,
.help-action-row,
.help-keywords {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.help-quick-actions button,
.help-action-row button {
  min-height: 28px;
  padding: 4px 8px;
}

.help-topic-list {
  display: grid;
  gap: 6px;
}

.help-topic-button {
  display: grid;
  gap: 3px;
  width: 100%;
  padding: 8px;
  border: 1px solid #d8e0e8;
  border-left: 3px solid transparent;
  border-radius: 6px;
  background: #ffffff;
  color: inherit;
  text-align: left;
}

.help-topic-button:hover,
.help-topic-button:focus-visible,
.help-topic-button.active {
  border-left-color: #2f6f9f;
  background: #f1f6fb;
}

.help-topic-button strong,
.help-topic-button small {
  min-width: 0;
  overflow-wrap: anywhere;
}

.help-topic-button small {
  color: #526171;
  font-size: 11px;
  line-height: 1.35;
}

.help-topic-detail {
  display: grid;
  gap: 10px;
  padding-top: 10px;
  border-top: 1px solid #d8e0e8;
}

.help-topic-header {
  display: grid;
  gap: 4px;
}

.help-topic-header h3 {
  margin: 0;
  font-size: 15px;
}

.help-topic-header p,
.help-when {
  margin: 0;
  color: #44566a;
}

.help-topic-header small {
  width: fit-content;
  padding: 2px 7px;
  border: 1px solid #c7d5e5;
  border-radius: 999px;
  background: #f2f7fc;
  color: #31516f;
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
}

.help-steps,
.help-tips {
  display: grid;
  gap: 6px;
  margin: 0;
  padding-left: 18px;
}

.help-steps li,
.help-tips li {
  line-height: 1.4;
}

.help-tips {
  color: #526171;
}

.help-keywords span {
  padding: 2px 6px;
  border: 1px solid #d8e0e8;
  background: #f8fafc;
  color: #44566a;
  font-size: 11px;
}

.button-help-tooltip {
  position: fixed;
  z-index: 1200;
  max-width: min(320px, calc(100vw - 24px));
  padding: 7px 9px;
  border: 1px solid #1f3147;
  border-radius: 6px;
  background: #172231;
  color: #ffffff;
  box-shadow: 0 8px 20px rgba(15, 23, 42, 0.24);
  font-size: 12px;
  line-height: 1.35;
  pointer-events: none;
}

.guided-demo-modal {
  width: min(860px, calc(100vw - 32px));
}

.guided-demo-modal header {
  align-items: start;
}

.guided-demo-modal header p {
  margin: 4px 0 0;
  color: #526171;
}

.guided-demo-modal header small {
  color: #526171;
  font-size: 12px;
}

.guided-demo-progress {
  display: grid;
  gap: 6px;
  padding: 10px;
  border: 1px solid #d8e0e8;
  background: #f8fafc;
}

.guided-demo-progress div {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  color: #31516f;
  font-size: 12px;
}

.guided-demo-progress progress {
  width: 100%;
  height: 10px;
}

.guided-demo-layout {
  display: grid;
  grid-template-columns: minmax(180px, 0.45fr) minmax(0, 1fr);
  gap: 16px;
}

.guided-demo-steps {
  display: grid;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.guided-demo-steps button {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  width: 100%;
  min-height: 36px;
  text-align: left;
}

.guided-demo-steps span {
  display: inline-grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border-radius: 999px;
  background: #e5edf7;
  color: #174a88;
  font-weight: 800;
  font-size: 11px;
}

.guided-demo-steps .active button {
  border-color: #7fa2cd;
  background: #eef6ff;
}

.guided-demo-steps .complete button {
  border-color: #88b99a;
  background: #f0faf3;
}

.guided-demo-steps small {
  color: #526171;
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.guided-demo-card {
  display: grid;
  gap: 10px;
  padding: 14px;
  border: 1px solid #d8e0e8;
  border-left: 3px solid #2f6f9f;
  background: #ffffff;
}

.guided-demo-card small {
  color: #31516f;
  font-weight: 800;
  text-transform: uppercase;
}

.guided-demo-card h3,
.guided-demo-card p,
.guided-demo-card ul {
  margin: 0;
}

.guided-demo-card ul {
  display: grid;
  gap: 6px;
  padding-left: 18px;
}

.guided-demo-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.guided-demo-evidence-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid #d8e0e8;
}

.agent-workspace-modal {
  width: min(980px, calc(100vw - 32px));
}

.agent-workspace-modal header {
  align-items: start;
}

.agent-workspace-modal header p {
  margin: 4px 0 0;
  color: #526171;
}

.agent-workspace-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.agent-run-packet-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: flex-end;
}

.agent-run-packet-actions button {
  min-height: 28px;
  padding: 4px 8px;
  font-size: 11px;
}

.agent-playbooks {
  display: grid;
  gap: 10px;
  padding: 10px;
  border: 1px solid #d8e0e8;
  border-left: 3px solid #6857a8;
  background: #fbfaff;
}

.agent-source-pack-builder {
  display: grid;
  gap: 10px;
  padding: 10px;
  border: 1px solid #d8e0e8;
  border-left: 3px solid #526f4f;
  background: #f8fcf7;
}

.agent-source-pack-builder > header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.agent-source-pack-builder > header div,
.agent-source-pack-add,
.agent-source-pack-list {
  display: grid;
  gap: 8px;
}

.agent-source-pack-builder > header span {
  color: #526171;
  font-size: 12px;
}

.agent-source-pack-add {
  grid-template-columns: minmax(120px, 0.3fr) minmax(160px, 0.45fr) minmax(220px, 1fr) auto;
  align-items: end;
}

.agent-source-pack-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.agent-source-pack-list li {
  display: grid;
  grid-template-columns: minmax(180px, 0.45fr) minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
  border: 1px solid #d8e0e8;
  background: #ffffff;
}

.agent-source-pack-list span {
  color: #2d3746;
  font-size: 12px;
}

.agent-playbooks > header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.agent-playbooks > header div {
  display: grid;
  gap: 2px;
}

.agent-playbooks > header span {
  color: #526171;
  font-size: 12px;
}

.agent-playbook-filters {
  display: grid;
  grid-template-columns: minmax(180px, 1fr) minmax(150px, 0.55fr) minmax(150px, 0.55fr);
  gap: 8px;
  align-items: end;
}

.agent-playbook-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.agent-playbook-grid article {
  display: grid;
  gap: 8px;
  padding: 10px;
  border: 1px solid #d8e0e8;
  background: #ffffff;
}

.agent-playbook-grid header {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
}

.agent-playbook-grid header div {
  display: grid;
  gap: 2px;
}

.agent-playbook-grid header span {
  color: #526171;
  font-size: 12px;
}

.agent-playbook-meta {
  margin: 0;
  color: #526171;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.agent-playbook-grid dl {
  display: grid;
  gap: 5px;
  margin: 0;
}

.agent-playbook-grid dl div {
  display: grid;
  gap: 2px;
}

.agent-playbook-grid dt {
  color: #526171;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.agent-playbook-grid dd {
  margin: 0;
  color: #2d3746;
  font-size: 12px;
}

.agent-plan {
  display: grid;
  gap: 12px;
}

.agent-plan > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px;
  border: 1px solid #d8e0e8;
  border-left: 3px solid #2f6f9f;
  background: #ffffff;
}

.agent-plan > header div {
  display: grid;
  gap: 2px;
}

.agent-plan > header span,
.agent-plan > header small {
  color: #526171;
}

.agent-plan-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.agent-plan-grid article,
.agent-missing-inputs,
.agent-step-list li,
.agent-run-output,
.agent-control-center,
.agent-control-grid article,
.agent-review-comment-queue,
.agent-review-comment-queue li,
.agent-reviewer-agents,
.agent-reviewer-grid article,
.agent-section-workqueue,
.agent-section-workqueue li,
.agent-audit-trail,
.agent-audit-grid article,
.agent-release-evidence,
.agent-release-evidence-grid article,
.agent-history,
.agent-history li,
.agent-run-columns article,
.agent-distribution-runbooks article,
.agent-provider-panel,
.agent-provider-output {
  padding: 10px;
  border: 1px solid #d8e0e8;
  background: #ffffff;
}

.agent-plan-grid h3,
.agent-plan-grid p,
.agent-missing-inputs ul,
.agent-step-list p {
  margin: 0;
}

.agent-plan-grid pre {
  max-height: 180px;
  overflow: auto;
  margin: 6px 0 0;
  white-space: pre-wrap;
}

.agent-context-score[data-status="thin"] {
  border-left: 3px solid #ba5c4b;
}

.agent-context-score[data-status="usable"] {
  border-left: 3px solid #c38a22;
}

.agent-context-score[data-status="strong"] {
  border-left: 3px solid #4f7f55;
}

.agent-missing-inputs {
  border-left: 3px solid #c68a1a;
}

.agent-missing-inputs ul {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 0;
  list-style: none;
}

.agent-missing-inputs li {
  padding: 2px 7px;
  border: 1px solid #e2c582;
  background: #fff9e8;
  font-size: 11px;
}

.agent-step-list {
  display: grid;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.agent-step-list li {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
  border-left: 3px solid #2f6f7e;
}

.agent-step-list small {
  color: #526171;
  font-weight: 800;
  text-transform: uppercase;
}

.agent-run-output {
  display: grid;
  gap: 10px;
  border-left: 3px solid #4f7f55;
}

.agent-run-output > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.agent-run-output > header div {
  display: grid;
  gap: 2px;
}

.agent-run-output > header span,
.agent-run-output > header small {
  color: #526171;
  font-size: 12px;
}

.agent-control-center {
  display: grid;
  gap: 10px;
  border-left: 3px solid #275da8;
  background: #f7fbff;
}

.agent-control-center[data-status="needs-input"] {
  border-left-color: #c68a1a;
}

.agent-control-center[data-status="blocked"] {
  border-left-color: #b34040;
}

.persistent-agent-control .agent-control-grid {
  grid-template-columns: 1fr;
}

.agent-control-center > header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.agent-control-center > header div {
  display: grid;
  gap: 2px;
}

.agent-control-center > header span,
.agent-control-center > header small {
  color: #526171;
  font-size: 12px;
}

.agent-control-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.agent-control-grid article {
  display: grid;
  gap: 6px;
}

.agent-control-grid h3,
.agent-control-grid ul,
.agent-control-grid p {
  margin: 0;
}

.agent-control-grid ul {
  display: grid;
  gap: 6px;
  padding: 0;
  list-style: none;
}

.agent-control-grid li {
  display: grid;
  gap: 2px;
  padding: 6px;
  border: 1px solid #d8e0e8;
  background: #ffffff;
}

.agent-control-grid li[data-status="missing"] {
  border-color: #e4aaaa;
}

.agent-control-grid li[data-status="needs-review"] {
  border-color: #e2c582;
}

.agent-control-grid li span {
  color: #526171;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.agent-control-grid li p {
  color: #2d3746;
  font-size: 12px;
}

.agent-reviewer-agents {
  display: grid;
  gap: 10px;
  border-left: 3px solid #6d668d;
  background: #fbfaff;
}

.agent-reviewer-agents > header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.agent-reviewer-agents > header div,
.agent-reviewer-grid article,
.agent-reviewer-grid article header div {
  display: grid;
  gap: 2px;
}

.agent-reviewer-agents > header span,
.agent-reviewer-agents > header small,
.agent-reviewer-grid article header span {
  color: #526171;
  font-size: 12px;
}

.agent-reviewer-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.agent-reviewer-grid article {
  gap: 8px;
  border-left: 3px solid #7fa2cd;
}

.agent-reviewer-grid article[data-status="needs-review"] {
  border-left-color: #c68a1a;
}

.agent-reviewer-grid article[data-status="blocked"] {
  border-left-color: #b34040;
}

.agent-reviewer-grid article header {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.agent-reviewer-grid h3,
.agent-reviewer-grid p,
.agent-reviewer-grid ul {
  margin: 0;
}

.agent-reviewer-grid h3 {
  font-size: 12px;
}

.agent-reviewer-grid ul {
  display: grid;
  gap: 4px;
  padding-left: 18px;
  color: #2d3746;
  font-size: 12px;
}

.agent-section-workqueue {
  display: grid;
  gap: 10px;
  border-left: 3px solid #2f6f7e;
  background: #f8fcfb;
}

.agent-edit-acceptance-queue {
  display: grid;
  gap: 10px;
  border-left: 3px solid #5f6b2f;
  background: #fbfcf4;
}

.agent-review-comment-queue {
  display: grid;
  gap: 10px;
  border-left: 3px solid #8c5a2f;
  background: #fff8f1;
}

.agent-lifecycle-board {
  display: grid;
  gap: 10px;
  border-left: 3px solid #7d5a28;
  background: #fffaf2;
}

.agent-section-workqueue > header,
.agent-review-comment-queue > header,
.agent-edit-acceptance-queue > header,
.agent-lifecycle-board > header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.agent-section-workqueue > header div,
.agent-section-workqueue li > div,
.agent-review-comment-queue > header div,
.agent-review-comment-queue li > div,
.agent-edit-acceptance-queue > header div,
.agent-edit-acceptance-queue li > div,
.agent-lifecycle-board > header div,
.agent-lifecycle-board li > div {
  display: grid;
  gap: 2px;
}

.agent-section-workqueue > header span,
.agent-section-workqueue > header small,
.agent-section-workqueue small,
.agent-section-workqueue span,
.agent-review-comment-queue > header span,
.agent-review-comment-queue > header small,
.agent-review-comment-queue small,
.agent-edit-acceptance-queue > header span,
.agent-edit-acceptance-queue > header small,
.agent-edit-acceptance-queue small,
.agent-lifecycle-board > header span,
.agent-lifecycle-board > header small,
.agent-lifecycle-board small {
  color: #526171;
  font-size: 12px;
}

.agent-lifecycle-filters {
  display: grid;
  grid-template-columns: repeat(3, minmax(150px, 1fr));
  gap: 8px;
  align-items: end;
}

.agent-lifecycle-filters label {
  display: grid;
  gap: 4px;
  color: #526171;
  font-size: 12px;
  font-weight: 800;
}

.agent-lifecycle-filters select,
.agent-lifecycle-filters input {
  width: 100%;
  min-height: 32px;
  border: 1px solid #cbd5df;
  border-radius: 6px;
  padding: 4px 8px;
  background: #ffffff;
  color: #182230;
  font: inherit;
}

.agent-section-workqueue ol,
.agent-review-comment-queue ol,
.agent-edit-acceptance-queue ol,
.agent-lifecycle-board ol {
  display: grid;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.agent-section-workqueue li {
  display: grid;
  grid-template-columns: minmax(220px, 0.6fr) minmax(0, 1fr);
  gap: 10px;
  border-left: 3px solid #7fa2cd;
}

.agent-lifecycle-board li {
  display: grid;
  grid-template-columns: minmax(220px, 0.62fr) minmax(0, 1fr);
  gap: 10px;
  border-left: 3px solid #c09a55;
}

.agent-review-comment-queue li {
  display: grid;
  grid-template-columns: minmax(240px, 0.7fr) minmax(0, 1fr);
  gap: 10px;
  border-left: 3px solid #b5854f;
}

.agent-review-comment-queue li[data-blocker="true"] {
  border-left-color: #b34040;
}

.agent-review-comment-queue li[data-status="complete"] {
  border-left-color: #2f7d4c;
}

.agent-edit-acceptance-queue li {
  display: grid;
  gap: 10px;
  border-left: 3px solid #98a454;
}

.agent-edit-acceptance-queue li[data-status="accepted"] {
  border-left-color: #2f7d4c;
}

.agent-edit-acceptance-queue li[data-status="rejected"] {
  border-left-color: #9d3d3d;
}

.agent-edit-acceptance-queue li[data-status="needs-revision"] {
  border-left-color: #b18127;
}

.agent-section-workqueue p,
.agent-section-workqueue ul,
.agent-review-comment-queue p,
.agent-review-comment-queue ul,
.agent-edit-acceptance-queue p,
.agent-edit-acceptance-queue ul,
.agent-lifecycle-board p,
.agent-lifecycle-board ul {
  margin: 0;
}

.agent-edit-acceptance-compare {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.agent-edit-acceptance-compare article {
  min-width: 0;
  border: 1px solid #d8e0e8;
  background: #ffffff;
}

.agent-edit-acceptance-compare h3 {
  margin: 0 0 6px;
  font-size: 12px;
}

.agent-edit-acceptance-compare pre {
  max-height: 180px;
  margin: 0;
  overflow: auto;
  white-space: pre-wrap;
  font-size: 12px;
}

.agent-section-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
}

.agent-release-evidence-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.agent-release-evidence-actions button {
  min-height: 28px;
  padding: 4px 8px;
  font-size: 11px;
}

.agent-section-depth {
  max-width: 180px;
  margin-top: 6px;
}

.agent-section-depth select {
  min-height: 30px;
  font-size: 12px;
}

.agent-section-actions button {
  min-height: 28px;
  padding: 4px 8px;
  font-size: 11px;
}

.agent-lifecycle-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
}

.agent-lifecycle-actions button {
  width: fit-content;
  min-height: 28px;
  padding: 4px 8px;
  font-size: 11px;
}

.agent-section-workqueue small,
.agent-review-comment-queue small,
.agent-edit-acceptance-queue small,
.agent-lifecycle-board small {
  font-weight: 800;
  text-transform: uppercase;
}

.agent-section-workqueue ul,
.agent-review-comment-queue ul,
.agent-edit-acceptance-queue ul,
.agent-lifecycle-board ul {
  display: grid;
  gap: 4px;
  padding-left: 18px;
  color: #2d3746;
  font-size: 12px;
}

.agent-audit-trail {
  display: grid;
  gap: 10px;
  border-left: 3px solid #596b7f;
  background: #f8fafc;
}

.agent-audit-trail > header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.agent-audit-trail > header div {
  display: grid;
  gap: 2px;
}

.agent-audit-trail > header span,
.agent-audit-trail > header small {
  color: #526171;
  font-size: 12px;
}

.agent-audit-grid {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.agent-audit-grid article {
  display: grid;
  gap: 6px;
}

.agent-audit-grid h3,
.agent-audit-grid dl,
.agent-audit-grid ul {
  margin: 0;
}

.agent-audit-grid dl,
.agent-audit-grid ul {
  display: grid;
  gap: 5px;
}

.agent-audit-grid dl div {
  display: grid;
  grid-template-columns: 82px minmax(0, 1fr);
  gap: 6px;
}

.agent-audit-grid dt {
  color: #526171;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.agent-audit-grid dd {
  margin: 0;
  overflow-wrap: anywhere;
  font-family: "SFMono-Regular", Consolas, monospace;
  font-size: 11px;
}

.agent-audit-grid ul {
  padding-left: 18px;
  color: #2d3746;
  font-size: 12px;
}

.agent-release-evidence {
  display: grid;
  gap: 10px;
  border-left: 3px solid #4d6f8f;
  background: #f7fbff;
}

.agent-release-evidence > header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.agent-release-evidence > header div,
.agent-release-evidence-grid article {
  display: grid;
  gap: 3px;
}

.agent-release-evidence > header span,
.agent-release-evidence > header small,
.agent-release-evidence-grid small {
  color: #526171;
  font-size: 12px;
}

.agent-release-evidence-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.agent-release-evidence-grid article[data-status="missing"] {
  border-left: 3px solid #b34040;
}

.agent-release-evidence-grid article[data-status="needs-review"] {
  border-left: 3px solid #c68a1a;
}

.agent-release-evidence-grid article[data-status="available"] {
  border-left: 3px solid #2f7d4c;
}

.agent-release-evidence-grid p {
  margin: 0;
  color: #2d3746;
  font-size: 12px;
}

.agent-history {
  display: grid;
  gap: 10px;
  border-left: 3px solid #3d7160;
  background: #f8fbfa;
}

.agent-history > header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.agent-history-audit-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: flex-end;
}

.agent-history-audit-actions button {
  min-height: 28px;
  padding: 4px 8px;
  font-size: 11px;
}

.agent-history > header div:not(.agent-history-audit-actions),
.agent-history li > div {
  display: grid;
  gap: 2px;
}

.agent-history > header span,
.agent-history > header small,
.agent-history li span,
.agent-history li small {
  color: #526171;
  font-size: 12px;
}

.agent-history-filters {
  display: grid;
  grid-template-columns: minmax(160px, 1.5fr) repeat(3, minmax(120px, 1fr));
  gap: 8px;
}

.agent-history ol {
  display: grid;
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.agent-history li {
  display: grid;
  grid-template-columns: minmax(180px, 0.7fr) minmax(0, 1fr);
  gap: 10px;
}

.agent-history dl {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 6px;
  margin: 0;
}

.agent-history dt {
  color: #526171;
  font-size: 11px;
  font-weight: 800;
  text-transform: uppercase;
}

.agent-history dd {
  margin: 0;
  overflow-wrap: anywhere;
  font-family: "SFMono-Regular", Consolas, monospace;
  font-size: 11px;
}

.agent-history p {
  margin: 4px 0 0;
  color: #2d3746;
  font-size: 12px;
}

.agent-history-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
}

.agent-history-actions button {
  min-height: 28px;
  padding: 4px 8px;
  font-size: 11px;
}

.agent-run-columns {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.agent-distribution-runbooks {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.agent-run-columns article {
  display: grid;
  gap: 6px;
}

.agent-distribution-runbooks article {
  display: grid;
  gap: 8px;
  border-left: 3px solid #7a5b2e;
}

.agent-distribution-runbooks header {
  display: grid;
  gap: 2px;
}

.agent-distribution-runbooks header span {
  color: #526171;
  font-size: 12px;
}

.agent-run-columns h3,
.agent-run-columns ul,
.agent-distribution-runbooks h3,
.agent-distribution-runbooks ul {
  margin: 0;
}

.agent-run-columns ul,
.agent-distribution-runbooks ul {
  display: grid;
  gap: 4px;
  padding-left: 18px;
  color: #2d3746;
  font-size: 12px;
}

.agent-provider-panel {
  display: grid;
  gap: 10px;
  border-left: 3px solid #6857a8;
}

.agent-provider-panel > header,
.agent-provider-output > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.agent-provider-panel > header div,
.agent-provider-output > header div {
  display: grid;
  gap: 2px;
}

.agent-provider-panel header span,
.agent-provider-output header span {
  color: #526171;
  font-size: 12px;
}

.agent-provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
  gap: 10px;
}

.agent-provider-output {
  display: grid;
  gap: 10px;
  background: #f8fafc;
}

.agent-provider-output ul {
  display: grid;
  gap: 4px;
  margin: 0;
  padding-left: 18px;
  color: #2d3746;
  font-size: 12px;
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

.docs-live-modal {
  width: min(1120px, 100%);
}

.docs-live-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 12px;
}

.docs-live-grid label,
.docs-live-voice {
  display: grid;
  gap: 6px;
  min-width: 0;
}

.docs-live-wide,
.docs-live-preview {
  grid-column: 1 / -1;
}

.docs-live-voice {
  padding: 10px;
  border: 1px solid #d7dee7;
  border-radius: 8px;
  background: #eef3f8;
}

.docs-live-voice-actions,
.docs-live-preview header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.docs-live-draft-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: flex-end;
}

.docs-live-draft-actions button {
  min-height: 28px;
  padding: 4px 8px;
  font-size: 11px;
}

.docs-live-voice-actions span,
.docs-live-preview span {
  color: #526171;
  font-size: 12px;
}

.docs-live-runtime {
  display: grid;
  gap: 8px;
  padding: 8px;
  border: 1px solid #d7dee7;
  border-left: 3px solid #4f7f55;
  border-radius: 6px;
  background: #ffffff;
}

.docs-live-runtime header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.docs-live-runtime header span,
.docs-live-runtime li {
  color: #526171;
  font-size: 12px;
}

.docs-live-runtime ul {
  display: grid;
  gap: 4px;
  margin: 0;
  padding-left: 18px;
}

.docs-live-intent-brief,
.docs-live-placeholder-manager {
  display: grid;
  gap: 8px;
  padding: 10px;
  border: 1px solid #d7dee7;
  border-radius: 8px;
  background: #ffffff;
}

.docs-live-intent-brief header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: start;
}

.docs-live-intent-brief header span,
.docs-live-intent-brief header small {
  color: #526171;
  font-size: 12px;
}

.docs-live-intent-grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(130px, 1fr));
  gap: 8px;
}

.docs-live-placeholder-manager header,
.docs-live-placeholder-add,
.docs-live-placeholder-grid [role="row"] {
  display: grid;
  gap: 8px;
  align-items: end;
}

.docs-live-placeholder-manager header {
  grid-template-columns: 1fr;
}

.docs-live-placeholder-manager header span {
  color: #526171;
  font-size: 12px;
}

.docs-live-placeholder-add {
  grid-template-columns: minmax(105px, 0.55fr) minmax(150px, 1fr) minmax(110px, 0.55fr) minmax(150px, 1fr) minmax(120px, 0.6fr) auto;
}

.docs-live-placeholder-grid {
  display: grid;
  gap: 4px;
}

.docs-live-placeholder-grid [role="row"] {
  grid-template-columns: minmax(90px, 0.45fr) minmax(150px, 1fr) minmax(105px, 0.55fr) minmax(150px, 1fr) minmax(115px, 0.6fr) auto;
}

.docs-live-placeholder-head {
  color: #526171;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
}

.docs-live-workflow {
  display: grid;
  gap: 10px;
  padding: 10px;
  border: 1px solid #cbd5df;
  border-radius: 8px;
  background: #ffffff;
}

.docs-live-workflow header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.docs-live-workflow header span,
.docs-live-section-cards span {
  color: #526171;
  font-size: 12px;
}

.docs-live-workflow > ol {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 8px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.docs-live-workflow > ol > li,
.docs-live-section-cards article {
  display: grid;
  gap: 4px;
  min-width: 0;
  padding: 8px;
  border: 1px solid #d7dee7;
  border-radius: 6px;
  background: #f7f9fb;
}

.docs-live-workflow > ol > li[data-status="needs-input"] {
  border-color: #c58a18;
  background: #fff8e8;
}

.docs-live-workflow li small {
  color: #526171;
  font-size: 11px;
  text-transform: uppercase;
}

.docs-live-workflow li span,
.docs-live-section-cards p {
  margin: 0;
  color: #2d3746;
  font-size: 12px;
  line-height: 1.4;
}

.docs-live-section-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 8px;
}

.docs-live-section-stage-list {
  display: grid;
  gap: 6px;
  margin: 4px 0 0;
  padding: 0;
  list-style: none;
}

.docs-live-section-stage-list li {
  display: grid;
  gap: 2px;
  padding-top: 6px;
  border-top: 1px solid #d7dee7;
}

.docs-live-section-stage-list li[data-status="needs-review"] strong {
  color: #7a5308;
}

.docs-live-review-packet {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
  gap: 8px;
}

.docs-live-review-packet-header {
  grid-column: 1 / -1;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 8px;
  border: 1px solid #d7dee7;
  border-radius: 6px;
  background: #f8fafc;
}

.docs-live-review-packet-header div:first-child {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.docs-live-review-packet-header span {
  color: #526071;
  font-size: 12px;
  line-height: 1.35;
}

.docs-live-review-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 6px;
  flex: 0 0 auto;
}

.docs-live-review-packet section {
  display: grid;
  gap: 6px;
  min-width: 0;
  padding: 8px;
  border: 1px solid #d7dee7;
  border-radius: 6px;
  background: #ffffff;
}

.docs-live-review-packet ul,
.docs-live-review-packet ol {
  display: grid;
  gap: 4px;
  margin: 0;
  padding-left: 18px;
  color: #2d3746;
  font-size: 12px;
  line-height: 1.4;
}

.docs-live-preview {
  display: grid;
  gap: 8px;
}

.docs-live-history {
  display: grid;
  gap: 8px;
}

.docs-live-history > header,
.docs-live-history article {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 8px;
  border: 1px solid #d7dee7;
  border-radius: 6px;
  background: #ffffff;
}

.docs-live-history > header {
  background: #f8fafc;
}

.docs-live-history article > div:first-child,
.docs-live-history > header > div {
  display: grid;
  gap: 3px;
  min-width: 0;
}

.docs-live-history span,
.docs-live-history p {
  margin: 0;
  color: #526071;
  font-size: 12px;
  line-height: 1.35;
}

.docs-live-history-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 6px;
  flex: 0 0 min(260px, 40%);
}

.modal textarea,
.modal input,
.modal select {
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

.command-agent-route {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  border: 1px solid #b9d4ec;
  border-left: 3px solid #275da8;
  border-radius: 6px;
  padding: 10px;
  background: #f5faff;
}

.command-agent-route > div,
.command-agent-actions {
  display: grid;
  gap: 6px;
}

.command-agent-route span {
  color: #526171;
  font-size: 12px;
}

.command-agent-preview {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 6px;
  margin: 4px 0 0;
}

.command-agent-preview div {
  min-width: 0;
}

.command-agent-preview dt {
  color: #526171;
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.command-agent-preview dd {
  margin: 0;
  overflow-wrap: anywhere;
  font-size: 12px;
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

  .command-agent-route,
  .command-agent-preview {
    grid-template-columns: 1fr;
  }

  .workspace,
  .workspace.mode-source,
  .workspace.mode-focus,
  .workspace.mode-preview,
  .workspace.mode-export,
  .workspace.mode-outline,
  .workspace.mode-presentation {
    grid-template-columns: 1fr;
  }

  .outline-mode-header,
  .outline-mode-row {
    grid-template-columns: 1fr;
  }

  .outline-mode-header {
    display: grid;
  }

  .outline-mode-row {
    margin-left: 0;
  }

  .outline-mode-actions {
    justify-content: flex-start;
    flex-wrap: wrap;
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

  .docs-live-grid {
    grid-template-columns: 1fr;
  }

  .docs-live-intent-grid {
    grid-template-columns: 1fr;
  }

  .docs-live-placeholder-add,
  .docs-live-placeholder-grid [role="row"] {
    grid-template-columns: 1fr;
  }

  .agent-playbook-filters {
    grid-template-columns: 1fr;
  }

  .agent-history-filters {
    grid-template-columns: 1fr;
  }

  .guided-demo-layout {
    grid-template-columns: 1fr;
  }

  .agent-plan-grid,
  .agent-playbook-grid,
  .agent-source-pack-add,
  .agent-source-pack-list li,
  .agent-control-grid,
  .agent-reviewer-grid,
  .agent-audit-grid,
  .agent-edit-acceptance-compare,
  .agent-review-comment-queue li,
  .agent-section-workqueue li,
  .agent-release-evidence-grid,
  .agent-history li,
  .agent-history dl,
  .agent-run-columns,
  .agent-distribution-runbooks,
  .agent-provider-grid,
  .agent-step-list li {
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

  .command-toolbar-heading {
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
