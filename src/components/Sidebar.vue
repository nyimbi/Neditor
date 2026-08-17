<template>
  <aside
    v-show="store.mode !== 'outline' && !writingSpaceMaximized && !store.zenMode"
    id="document-sidebar"
    :key="store.sidebar"
    :data-sidebar="store.sidebar"
    class="sidebar"
    aria-label="Document workspace"
    tabindex="-1"
  >
    <div
      v-if="store.uiMode === 'pilot'"
      class="sidebar-resize-handle"
      title="Drag to resize"
      @mousedown.prevent="onSidebarResizeStart"
    ></div>
    <button
      type="button"
      class="sidebar-collapse-btn"
      :title="sidebarCollapsed ? 'Expand sidebar (⌘B)' : 'Collapse sidebar (⌘B)'"
      :aria-label="sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'"
      @click="toggleSidebarCollapsed()"
    >{{ sidebarCollapsed ? '▶' : '◀' }}</button>
    <header v-if="store.uiMode === 'pilot'" class="sidebar-panel-header" aria-label="Current panel">
      <span class="sidebar-panel-name">{{ currentPanelLabel }}</span>
    </header>
    <template v-if="store.sidebar === 'files'">
      <h2>Workspace</h2>
      <!-- Pinned files -->
      <section v-if="store.pinnedFiles.length" class="pinned-files" aria-label="Pinned files">
        <h3>Pinned</h3>
        <div class="pinned-files-list">
          <div v-for="path in store.pinnedFiles" :key="path" class="pinned-file-item">
            <button type="button" class="pinned-file-name" @click="store.openPath(path)" :title="path">
              📌 {{ path.split('/').pop() }}
            </button>
            <button type="button" class="pin-remove" @click="unpinFile(path)" title="Unpin">×</button>
          </div>
        </div>
      </section>
      <!-- PARA setup -->
      <div class="files-actions" style="display:flex;gap:4px;flex-wrap:wrap;margin-bottom:6px">
        <button type="button" @click="setupPARAWorkspace" title="Create PARA folder structure (Projects/Areas/Resources/Archives)">PARA setup</button>
        <button type="button" @click="active?.path && pinFile(active.path)" :disabled="!active?.path || store.pinnedFiles.includes(active?.path || '')">📌 Pin current</button>
      </div>
      <div class="workspace-search-box">
        <input
          v-model="workspaceSearchQuery"
          type="search"
          placeholder="Search workspace..."
          class="workspace-search-input"
          @keydown.enter="runWorkspaceSearch"
          aria-label="Search workspace files"
        />
        <button type="button" :disabled="workspaceSearchBusy" @click="runWorkspaceSearch">
          {{ workspaceSearchBusy ? '…' : '↵' }}
        </button>
      </div>
      <div v-if="workspaceSearchResults.length" class="workspace-search-results">
        <div
          v-for="result in workspaceSearchResults.slice(0, 50)"
          :key="result.path + result.line"
          class="ws-search-result"
          role="button"
          tabindex="0"
          @click="openSearchResult(result)"
          @keydown.enter="openSearchResult(result)"
        >
          <span class="ws-sr-path">{{ result.path }}</span>
          <span class="ws-sr-line">:{{ result.line }}</span>
          <span class="ws-sr-excerpt">{{ result.excerpt }}</span>
        </div>
      </div>
      <button type="button" @click="openFolder">Open folder</button>
      <button v-if="store.workspaceRoot" type="button" @click="store.refreshWorkspace">Refresh</button>
      <p v-if="store.workspaceRoot" class="workspace-root">{{ store.workspaceRoot }}</p>
      <p v-else>Open a folder to browse project files.</p>
      <section class="document-set-manager" aria-label="Document set manager">
        <header>
          <div>
            <h3>Document Sets</h3>
            <small>{{ documentSetGroups.length }} open sets</small>
          </div>
        </header>
        <label>
          Active document set
          <input v-model="documentSetDraft" aria-label="Active document set" placeholder="Board Pack, Client Deliverable" />
        </label>
        <div class="document-set-actions">
          <button type="button" :disabled="!documentSetDraft.trim()" @click="assignActiveDocumentSet(documentSetDraft)">Assign active</button>
          <button type="button" :disabled="!activeDocumentSet" @click="clearActiveDocumentSet">Remove active</button>
        </div>
        <label v-if="activeDocumentSet">
          Rename open set
          <input v-model="documentSetRenameDraft" aria-label="Rename active document set" placeholder="New set name" />
        </label>
        <button
          v-if="activeDocumentSet"
          type="button"
          :disabled="!documentSetRenameDraft.trim() || documentSetRenameDraft.trim() === activeDocumentSet"
          @click="renameActiveDocumentSet"
        >
          Rename all open set tabs
        </button>
        <div v-if="activeDocumentSet" class="document-set-actions">
          <button type="button" @click="insertActiveDocumentSetManifest">Insert manifest</button>
          <button type="button" @click="copyActiveDocumentSetManifest">Copy manifest</button>
        </div>
        <div v-if="documentSetGroups.length" class="document-set-list" role="list" aria-label="Open document sets">
          <article v-for="group in documentSetGroups" :key="group.key" role="listitem">
            <div>
              <strong>{{ group.label }}</strong>
              <small>{{ group.documents.length }} open document{{ group.documents.length === 1 ? "" : "s" }}</small>
            </div>
            <button type="button" :disabled="!active.path || activeDocumentSet === group.label" @click="assignActiveDocumentSet(group.label)">Add active</button>
          </article>
        </div>
        <p v-else class="sidebar-hint">Use document sets to keep board packs, proposals, appendices, and review bundles grouped together.</p>
      </section>
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
      <section class="document-map" aria-label="Document map">
        <header>
          <div>
            <h3>Document Map</h3>
            <small>{{ documentMapSummary }}</small>
          </div>
          <button
            type="button"
            :class="{ active: documentMapBlockersOnly }"
            title="Focus all unresolved review, citation, metadata, and diagnostic blockers"
            @click="focusDocumentMapBlockers"
          >
            {{ documentMapBlockersOnly ? "All items" : "Blockers" }}
          </button>
        </header>
        <div class="document-map-controls">
          <label>
            Search map
            <input v-model="documentMapQuery" type="search" placeholder="heading, comment, figure, warning" />
          </label>
          <label>
            Show
            <select v-model="documentMapFilter">
              <option v-for="filter in documentMapFilterOptions" :key="filter.id" :value="filter.id">{{ filter.label }}</option>
            </select>
          </label>
        </div>
        <div class="document-map-counts" aria-label="Document map counts">
          <button v-for="filter in documentMapFilterOptions" :key="filter.id" type="button" :class="{ active: documentMapFilter === filter.id }" @click="documentMapFilter = filter.id">
            <strong>{{ documentMapCountByFilter(filter.id) }}</strong>
            <span>{{ filter.shortLabel }}</span>
          </button>
        </div>
        <div v-if="filteredDocumentMapItems.length" class="document-map-list" role="list" aria-label="Navigable document map items">
          <button
            v-for="item in filteredDocumentMapItems"
            :key="item.id"
            type="button"
            role="listitem"
            class="document-map-row"
            :data-kind="item.kind"
            @click="goToDocumentMapItem(item)"
          >
            <span class="document-map-kind">{{ documentMapKindLabel(item.kind) }}</span>
            <span class="document-map-label">{{ item.label }}</span>
            <small>{{ item.detail }}</small>
            <span class="document-map-status">{{ item.status }}</span>
          </button>
        </div>
        <p v-else class="sidebar-hint">No document map items match the current filter.</p>
      </section>
      <section class="outline-library" aria-label="Document outline library">
        <header>
          <div>
            <h3>Outline Library</h3>
            <small>{{ filteredDocumentOutlineTemplates.length }} of {{ allDocumentOutlineTemplates.length }} outlines</small>
          </div>
        </header>
        <div class="outline-library-filters">
          <label>
            Search outlines
            <input v-model="outlineLibraryQuery" type="search" placeholder="RFP, board, textbook, policy" />
          </label>
          <label>
            Category
            <select v-model="outlineLibraryCategory">
              <option v-for="category in outlineLibraryCategories" :key="category" :value="category">{{ category === "all" ? "All categories" : category }}</option>
            </select>
          </label>
        </div>
        <div class="outline-library-actions">
          <button type="button" :disabled="!outlineDraftItems.length" @click="saveCurrentOutlineTemplate">Save planner outline</button>
          <button type="button" @click="resetCustomOutlineDraft">New custom outline</button>
          <button type="button" :disabled="!store.workspaceRoot || workspaceOutlineSyncBusy" @click="syncWorkspaceOutlines">
            {{ workspaceOutlineSyncBusy ? "Syncing..." : "Sync workspace outlines" }}
          </button>
        </div>
        <p class="sidebar-hint">{{ workspaceOutlineSyncStatus || (store.workspaceRoot ? "Workspace outlines sync with .neditor/outlines.json for CLI and app reuse." : "Open a workspace folder to sync outlines with .neditor/outlines.json.") }}</p>
        <label>
          Custom outline name
          <input v-model="customOutlineDraft.name" placeholder="Quarterly business review" />
        </label>
        <label>
          Custom category
          <input v-model="customOutlineDraft.category" placeholder="Executive" />
        </label>
        <label>
          Custom summary
          <input v-model="customOutlineDraft.summary" placeholder="Reusable outline for recurring documents" />
        </label>
        <label>
          Custom tags
          <input v-model="customOutlineTags" placeholder="board, decision, quarterly" />
        </label>
        <label>
          Best for
          <input v-model="customOutlineBestFor" placeholder="Board packs, client reviews, technical chapters" />
        </label>
        <label>
          Docs Live workflow
          <select v-model="customOutlineDraft.docsLiveType">
            <option v-for="type in docsLiveDocumentTypes" :key="type.id" :value="type.id">{{ type.label }}</option>
          </select>
        </label>
        <p class="sidebar-hint">Saving uses the current planner outline so users can adapt a built-in outline and keep it for future documents.</p>
        <div class="outline-template-list" role="list" aria-label="Selectable document outlines">
          <article v-for="template in filteredDocumentOutlineTemplates" :key="`${template.source}-${template.id}`" role="listitem" class="outline-template-card">
            <header>
              <div>
                <strong>{{ template.name }}</strong>
                <small>{{ template.category }} | {{ template.source }} | {{ outlineDocsLiveTypeLabel(template) }} | {{ template.outline.length }} sections</small>
              </div>
              <button type="button" title="Load this outline into the planner" @click="useDocumentOutlineTemplate(template)">Use</button>
            </header>
            <p>{{ template.summary }}</p>
            <pre>{{ documentOutlineTemplateToPlannerText(template) }}</pre>
            <div class="outline-template-actions">
              <button type="button" title="Send this outline to Docs Live" @click="sendOutlineTemplateToDocsLive(template)">Docs Live</button>
              <button type="button" title="Append this outline skeleton to the active document" @click="appendDocumentOutlineTemplate(template)">Append</button>
              <button v-if="template.source === 'custom'" type="button" title="Edit this custom outline's metadata using the current planner" @click="editCustomOutlineTemplate(template)">Edit</button>
              <button v-if="template.source === 'custom'" class="danger-action" type="button" title="Delete this custom outline" @click="store.deleteCustomDocumentOutlineTemplate(template.id)">Delete</button>
            </div>
          </article>
        </div>
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
      <section class="compiler-output-inventory" aria-label="Compiler output inventory">
        <article v-for="item in compilerOutputInventory" :key="item.label" class="snapshot-row" :data-status="item.status">
          <p>{{ item.label }}</p>
          <small>{{ item.status }} | {{ item.detail }}</small>
        </article>
      </section>
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

    <template v-else-if="store.sidebar === 'layout'">
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

    <template v-else-if="store.sidebar === 'tables'">
      <h2>Tables</h2>
      <p class="sidebar-hint">{{ selectedTableEditSummary }}</p>
      <section v-if="tableDraft" class="table-two-way-strip" aria-label="Two-way table editing">
        <header>
          <div>
            <strong>Two-way table editing</strong>
            <span>{{ tableTwoWayHint }}</span>
          </div>
          <span :class="['table-sync-chip', tableTwoWayStatusClass]" role="status">{{ tableTwoWayStatus }}</span>
        </header>
        <div class="table-two-way-actions" role="group" aria-label="Table text and grid synchronization">
          <button type="button" :disabled="!tableDraft" title="Focus the visual table grid" @click="focusTableGrid">Focus grid</button>
          <button type="button" :disabled="!tableDraft" title="Focus the editable Markdown source block in the Tables panel" @click="focusTableSourceEditor">
            Source block
          </button>
          <label class="compact-check table-follow-source-toggle" title="Automatically load the Markdown table under the source editor cursor when the Tables panel is open">
            <input v-model="tableFollowSourceCursor" type="checkbox" />
            Follow source cursor
          </label>
          <button
            type="button"
            :disabled="!canGoToTableSource"
            title="Select the table's Markdown source in the document editor so you can edit the table directly in text"
            @click="editSelectedTableInMarkdownText"
          >
            Edit table text
          </button>
          <button
            type="button"
            :disabled="(!isNewTableDraft && tableDraftDirty) || tableDraftHasErrors"
            title="Insert a Markdown table at the cursor and select it for direct text editing"
            @click="insertTableDraftInMarkdownText"
          >
            {{ isNewTableDraft ? "Insert draft as text" : "Create table in text" }}
          </button>
          <button
            type="button"
            :disabled="!tableDraft || !tableSourceEditDirty"
            title="Parse the edited Markdown source text and update the visual grid preview"
            @click="updateTableDraftFromSourceText"
          >
            Sync text to grid
          </button>
          <button
            type="button"
            :disabled="tableDraftHasErrors || tableDraftSourceChanged"
            title="Write the current visual grid back to the Markdown source table"
            @click="applyTableDraft()"
          >
            Apply grid to text
          </button>
          <button
            type="button"
            :disabled="!tableCursorCellPreview"
            :title="tableCursorCellPreview ? 'Load the Markdown table cell under the editor cursor for a precise text edit' : 'Place the editor cursor inside a Markdown table header or body cell'"
            @click="loadTableTextCellAtCursor"
          >
            Cell at cursor
          </button>
        </div>
      </section>
      <section class="table-cell-text-editor" aria-label="Text table cell editor">
        <p class="sidebar-hint table-cursor-cell">{{ tableCursorCellSummary }}</p>
        <label>
          Table cell text
          <input
            v-model="tableTextCellValue"
            :disabled="!tableTextCellEdit"
            :placeholder="tableTextCellEdit ? 'Cell value' : 'Place cursor in a table cell'"
            @keydown.enter.prevent="applyTableTextCellEdit"
          />
        </label>
        <div class="table-actions">
          <button
            type="button"
            :disabled="!tableCursorCellPreview"
            :title="tableCursorCellPreview ? 'Read the table cell at the current source cursor' : 'Place the editor cursor inside a Markdown table header or body cell'"
            @click="loadTableTextCellAtCursor"
          >
            Edit cell at cursor
          </button>
          <button type="button" :disabled="!tableTextCellEdit" title="Write this cell value directly into the Markdown table text" @click="applyTableTextCellEdit">
            Apply cell to text
          </button>
          <button type="button" :disabled="!tableTextCellEdit" title="Select the source row for this table cell" @click="goToTableTextCellSource">Go to cell text</button>
        </div>
        <p v-if="tableTextCellError" class="table-source-error" role="alert">{{ tableTextCellError }}</p>
        <p v-else class="sidebar-hint">{{ tableTextCellEditSummary }}</p>
      </section>
      <label>
        Table
        <select
          :value="selectedTableIndex"
          :disabled="tableDraftDirty"
          :title="tableDraftDirty ? 'Apply or cancel the current table edit before switching source tables' : 'Choose a Markdown source table to edit'"
          @change="selectTableForEditing(inputValue($event))"
        >
          <option v-for="(table, index) in markdownTables" :key="`${table.startLine}-${index}`" :value="index">
            Line {{ table.startLine }} - {{ table.caption || table.headers.join(", ") }}
          </option>
        </select>
      </label>
      <div class="table-actions">
        <button
          type="button"
          :disabled="tableDraftDirty"
          :title="tableDraftDirty ? 'Apply or cancel the current table edit before loading another source table' : 'Load the Markdown table at the editor cursor or selection'"
          @click="loadTableAtCursor()"
        >
          Edit table at cursor
        </button>
        <button
          type="button"
          :disabled="!canEditMarkdownTableText"
          title="Select the exact Markdown table lines in the editor so you can edit the table directly in text"
          @click="editSelectedTableInMarkdownText"
        >
          Edit Markdown in text
        </button>
        <button type="button" :disabled="!canGoToTableSource" @click="() => goToSelectedTableSource()">Go to source table</button>
        <button
          type="button"
          :disabled="tableDraftDirty"
          :title="tableDraftDirty ? 'Apply or cancel the current table edit before creating another table' : 'Create a new Markdown table draft'"
          @click="createTableDraft"
        >
          New table
        </button>
        <button
          type="button"
          :disabled="(!isNewTableDraft && tableDraftDirty) || tableDraftHasErrors"
          :title="isNewTableDraft ? 'Insert this draft as Markdown and select it in the document editor' : 'Insert a starter Markdown table at the cursor and select it for direct text editing'"
          @click="insertTableDraftInMarkdownText"
        >
          {{ isNewTableDraft ? "Insert draft in text" : "New table in text" }}
        </button>
      </div>
      <div class="table-actions">
        <button type="button" :disabled="tableDataBusy" @click="importTableFromSpreadsheet">
          {{ tableDataBusy ? "Working..." : "Import CSV/XLSX" }}
        </button>
        <span class="button-help-hitbox" @mouseenter="handleButtonHelpHitboxEnter" @mousemove="handleButtonHelpHitboxEnter" @mouseleave="hideButtonHelp">
          <button type="button" :disabled="tableDataBusy || !tableDraft" @click="exportSelectedTable('csv')">Export CSV</button>
          <span v-if="tableDataBusy || !tableDraft" class="button-help-hitbox-overlay" aria-hidden="true"></span>
        </span>
        <span class="button-help-hitbox" @mouseenter="handleButtonHelpHitboxEnter" @mousemove="handleButtonHelpHitboxEnter" @mouseleave="hideButtonHelp">
          <button type="button" :disabled="tableDataBusy || !tableDraft" @click="exportSelectedTable('xlsx')">Export XLSX</button>
          <span v-if="tableDataBusy || !tableDraft" class="button-help-hitbox-overlay" aria-hidden="true"></span>
        </span>
        <button type="button" @click="insertSqlTransformTemplate">Insert SQL transform</button>
      </div>
      <label v-if="tableImportSheetNames.length > 1">
        Workbook worksheet
        <select
          v-model.number="tableImportSelectedSheetIndex"
          :disabled="tableDataBusy"
          title="Choose which worksheet from the imported XLSX workbook should become the editable Markdown table"
          @change="importSelectedSpreadsheetWorksheet"
        >
          <option v-for="(sheet, index) in tableImportSheetNames" :key="`${sheet}-${index}`" :value="index">
            {{ index + 1 }}. {{ sheet }}
          </option>
        </select>
      </label>
      <p v-if="tableImportSheetNames.length > 1" class="sidebar-hint">
        Imported worksheet {{ tableImportSelectedSheetIndex + 1 }} of {{ tableImportSheetNames.length }} from {{ tableImportSourceLabel }}.
      </p>
      <template v-if="tableDraft">
        <div class="table-actions">
          <button type="button" :disabled="tableDraftHasErrors || tableDraftSourceChanged" title="Write this visual table draft back to the Markdown source" @click="applyTableDraft()">{{ isNewTableDraft ? "Insert table" : "Apply table" }}</button>
          <button type="button" title="Discard the visual table draft and return to the current source table" @click="cancelTableDraft">Cancel table edit</button>
          <button type="button" title="Add a blank row to the visual table draft" @click="addTableRow">Add row</button>
          <button type="button" title="Add a blank column to the visual table draft" @click="addTableColumn">Add column</button>
          <button type="button" title="Append a SUM formula row across numeric columns" @click="addTableTotalsRow">Add totals row</button>
          <button type="button" title="Append an AVG formula row across numeric columns" @click="addTableFormulaRow('AVG')">AVG row</button>
          <button type="button" title="Append a MIN formula row across numeric columns" @click="addTableFormulaRow('MIN')">MIN row</button>
          <button type="button" title="Append a MAX formula row across numeric columns" @click="addTableFormulaRow('MAX')">MAX row</button>
          <button type="button" title="Append a COUNT formula row across numeric columns" @click="addTableFormulaRow('COUNT')">COUNT row</button>
        </div>
        <section v-if="tableDraftSourceChanged" class="table-source-sync" aria-label="Table source synchronization">
          <strong>Source table changed</strong>
          <p>{{ tableSourceSyncMessage }}</p>
          <div class="table-actions">
            <button type="button" title="Reload the visual grid from the current Markdown source table" @click="reloadTableDraftFromSource">Reload from source</button>
            <button type="button" :disabled="tableDraftHasErrors" title="Replace the current Markdown table with this visual draft" @click="applyTableDraft(true)">Apply draft over source</button>
          </div>
        </section>
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
          ref="tableEditorGrid"
          class="table-editor-grid"
          role="group"
          aria-label="Table editor grid"
          tabindex="-1"
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
        <label class="table-preview table-source-editor">
          Markdown source
          <textarea
            ref="tableSourceEditor"
            v-model="tableSourceEditText"
            rows="7"
            spellcheck="false"
            title="Edit the Markdown pipe table directly; valid source updates the visual grid as you type"
            :aria-invalid="Boolean(tableSourceEditError)"
            @input="markTableSourceEditDirty"
          ></textarea>
        </label>
        <div class="table-actions">
          <button type="button" :disabled="!tableDraft || !tableSourceEditDirty" title="Canonicalize the Markdown source text and confirm the live visual grid preview" @click="updateTableDraftFromSourceText">
            Update grid from source
          </button>
          <button type="button" :disabled="!tableDraft" title="Regenerate Markdown source text from the current visual grid" @click="refreshTableSourceEditFromDraft">
            Refresh source from grid
          </button>
          <button
            type="button"
            :disabled="!tableDraft || (!isNewTableDraft && tableDraftSourceChanged)"
            title="Parse and write this Markdown source table into the document"
            @click="applyTableSourceEdit()"
          >
            {{ isNewTableDraft ? "Insert source text" : "Apply source text" }}
          </button>
        </div>
        <p v-if="tableSourceEditError" class="table-source-error" role="alert">{{ tableSourceEditError }}</p>
        <p v-else class="sidebar-hint">{{ tableSourceEditSummary }}</p>
      </template>
      <p v-else>No Markdown table selected.</p>
    </template>

    <template v-else-if="store.sidebar === 'templates'">
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

    <template v-else-if="store.sidebar === 'references'">
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
            {{ item.fitLabel }} | {{ item.fitScore }}/100 | {{ item.title }} | {{ item.qualityDimensions.map((dimension) => `${dimension.name} ${dimension.score}`).join(" / ") }} | {{ item.reviewAction }}
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

    <template v-else-if="store.sidebar === 'versioning'">
      <h2>Versioning</h2>
      <article v-if="store.gitStatus?.inside_repo" class="snapshot-row">
        <p>{{ store.gitStatus.branch || "detached" }} | {{ store.gitStatus.dirty ? "dirty" : "clean" }}</p>
        <small v-for="line in store.gitStatus.summary" :key="line">{{ line }}</small>
      </article>
      <section v-else class="git-free-versioning" aria-label="Git-free versioning guidance">
        <header>
          <strong>Snapshot-first document history</strong>
          <span>{{ versioningModeLabel }}</span>
        </header>
        <p>
          This document is outside Git, so NEditor keeps recovery points locally. Use snapshots for business drafts,
          approvals, and pre-export rollback without configuring developer tooling.
        </p>
        <ol>
          <li v-for="step in gitFreeVersioningPlan" :key="step">{{ step }}</li>
        </ol>
        <section class="git-free-controls" aria-label="Snapshot recovery controls">
          <label>
            Snapshot storage
            <select v-model="store.snapshotStorage" aria-label="Versioning snapshot storage">
              <option value="app-data">Private app data</option>
              <option value="project-local">Project .neditor folder</option>
            </select>
          </label>
          <label><input v-model="store.autoSnapshot" type="checkbox" /> Automatic recovery snapshots</label>
          <label>
            Recovery interval
            <input v-model.number="store.snapshotIntervalMs" type="number" min="30000" max="3600000" step="30000" />
          </label>
        </section>
        <button type="button" @click="createRecoverySnapshot">Create recovery snapshot</button>
      </section>
      <template v-if="store.gitStatus?.inside_repo">
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
      <h3>Snapshots</h3>
      <button type="button" @click="snapshotActive">Create snapshot</button>
      <button type="button" @click="store.listSnapshots">Refresh snapshots</button>
      <article v-for="snapshot in store.snapshots" :key="`version-${snapshot.snapshot_path}`" class="snapshot-row">
        <p>{{ snapshot.label || "snapshot" }}</p>
        <small>{{ snapshot.created_at || snapshot.snapshot_path }}</small>
        <small>{{ snapshot.snapshot_path }}</small>
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
                <div class="agent-lifecycle-actions">
                  <button type="button" @click="runAgentControlAction(action)">Run action</button>
                </div>
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
      <!-- Style guide findings -->
      <section class="style-guide-panel" aria-label="Style guide enforcement">
        <header>
          <h3>Style guide</h3>
          <label class="style-guide-toggle" :title="store.styleGuideEnabled ? 'Disable style guide' : 'Enable style guide'">
            <input type="checkbox" v-model="store.styleGuideEnabled" />
            {{ store.styleGuideEnabled ? 'On' : 'Off' }}
          </label>
        </header>
        <template v-if="store.styleGuideEnabled">
          <p v-if="!styleGuideFindings.length" class="sidebar-hint">No style issues found.</p>
          <article
            v-for="finding in styleGuideFindings.slice(0, 50)"
            :key="finding.ruleId + ':' + finding.line + ':' + finding.column"
            class="style-finding"
            :data-severity="finding.severity"
          >
            <div class="style-finding-header">
              <span class="style-badge" :class="finding.severity">{{ finding.severity }}</span>
              <small>line {{ finding.line }} · {{ finding.category }}</small>
            </div>
            <p>{{ finding.description }}: <em>"{{ finding.matchedText }}"</em></p>
            <small class="style-suggestion">{{ finding.suggestion }}</small>
          </article>
          <p v-if="styleGuideFindings.length > 50" class="sidebar-hint">{{ styleGuideFindings.length - 50 }} more findings not shown.</p>
        </template>
        <p v-else class="sidebar-hint">Enable the style guide to check for weak qualifiers, filler phrases, passive voice, and jargon.</p>
      </section>

      <section class="quality-recommendations" aria-label="Quality improvement recommendations">
        <header>
          <h3>Quality recommendations</h3>
          <span>{{ qualityRecommendationSummary }}</span>
        </header>
        <p>Deterministic QA scans surface evidence gaps, review risks, structure issues, and concrete quality-improvement actions before human review or export.</p>
        <div class="release-readiness-actions">
          <button type="button" @click="runQualityReview">Run QA review</button>
          <button type="button" @click="insertQualityImprovementReport">Insert QA report</button>
          <button type="button" @click="openQualityAgent">Improve with agent</button>
        </div>
        <article
          v-for="item in qualityImprovementRecommendations"
          :key="item.id"
          class="snapshot-row"
          :data-status="item.severity"
        >
          <strong>{{ item.label }}</strong>
          <p>{{ item.recommendation }}</p>
          <small>{{ item.action }}</small>
        </article>
        <section class="quality-step-assistance" aria-label="AI quality review assistance">
          <header>
            <h4>AI quality assistance</h4>
            <button type="button" @click="appendAllQualityStepAssistance">Use all</button>
          </header>
          <p>Context-aware suggestions turn the QA findings into triage, evidence, humanization, and handoff answers you can edit before inserting.</p>
          <article
            v-for="item in qualityStepAssistance"
            :key="item.id"
            class="snapshot-row"
            data-status="improve"
          >
            <strong>{{ item.label }}</strong>
            <p>{{ item.suggestedAnswer }}</p>
            <small>{{ item.rationale }}</small>
            <ul class="signal-list">
              <li v-for="signal in item.contextSignals" :key="`${item.id}-${signal}`">{{ signal }}</li>
            </ul>
            <button type="button" @click="appendQualityStepAssistance(item)">{{ item.actionLabel }}</button>
          </article>
          <label class="field">
            <span>Quality review notes</span>
            <textarea v-model="qualityReviewNotes" aria-label="Quality review notes" rows="6" placeholder="Accept guidance, add owner decisions, and record reviewer questions here."></textarea>
          </label>
          <button type="button" @click="insertQualityReviewNotes">Insert review notes</button>
        </section>
      </section>
      <section class="review-evidence-snapshot" aria-label="Document evidence and approval review">
        <header>
          <div>
            <h3>Evidence and approval review</h3>
            <span>{{ reviewEvidenceSnapshotSummary }}</span>
          </div>
          <button type="button" @click="refreshReviewEvidenceSnapshot">Refresh</button>
        </header>
        <p>Surface claims, missing citations, unresolved reviewer comments, approval metadata, and specialist reviewer actions before export.</p>
        <div class="release-readiness-actions">
          <button type="button" :disabled="!activeReviewEvidenceRun" @click="insertReviewEvidenceAudit">Insert evidence audit</button>
          <button type="button" :disabled="!activeReviewEvidenceRun" @click="insertClaimEvidenceMatrix">Insert claim-source matrix</button>
          <button type="button" @click="openAgentWorkspace('Review this document for claim inventory, citations, approval metadata, reviewer objections, and export release blockers.')">Open agent workspace</button>
        </div>
        <template v-if="activeReviewEvidenceRun">
          <section class="review-evidence-metrics" aria-label="Evidence review metrics">
            <span><strong>{{ activeReviewEvidenceRun.documentEvidence.claimInventory.length }}</strong> claims</span>
            <span><strong>{{ activeClaimSourceMatches.length }}</strong> source matches</span>
            <span><strong>{{ activeReviewEvidenceRun.documentEvidence.citationTodos.length }}</strong> citation TODOs</span>
            <span><strong>{{ activeReviewEvidenceRun.documentEvidence.reviewCommentResolutions.length }}</strong> comments</span>
            <span><strong>{{ activeReviewEvidenceRun.approvalGate.blockers.length }}</strong> approval blockers</span>
          </section>
          <details :open="Boolean(activeReviewEvidenceRun.documentEvidence.claimInventory.length)">
            <summary>Claim inventory</summary>
            <article
              v-for="claim in activeReviewEvidenceRun.documentEvidence.claimInventory.slice(0, 12)"
              :key="`${claim.sourceLine}-${claim.text}`"
              class="snapshot-row"
              :data-status="claim.kind"
            >
              <strong>Line {{ claim.sourceLine }} | {{ claim.kind }}</strong>
              <p>{{ claim.text }}</p>
              <small>{{ claim.reason }}</small>
              <small v-if="activeClaimSourceMatchByLine.get(claim.sourceLine)">
                Suggested source: @{{ activeClaimSourceMatchByLine.get(claim.sourceLine)?.source.citation_key }}
                | {{ activeClaimSourceMatchByLine.get(claim.sourceLine)?.reasons.join("; ") }}
              </small>
            </article>
            <p v-if="!activeReviewEvidenceRun.documentEvidence.claimInventory.length" class="sidebar-hint">No candidate claims detected in the current snapshot.</p>
          </details>
          <details :open="Boolean(activeReviewEvidenceRun.approvalGate.blockers.length)">
            <summary>Approval metadata gate</summary>
            <article v-for="field in activeReviewEvidenceRun.approvalGate.fields" :key="field.key" class="snapshot-row" :data-status="field.status">
              <strong>{{ field.label }}</strong>
              <p>{{ field.value || "Missing" }}</p>
              <small>{{ field.guidance }}</small>
            </article>
            <ul v-if="activeReviewEvidenceRun.approvalGate.blockers.length" class="signal-list">
              <li v-for="blocker in activeReviewEvidenceRun.approvalGate.blockers" :key="blocker">{{ blocker }}</li>
            </ul>
          </details>
          <details>
            <summary>Reviewer agents</summary>
            <article v-for="reviewer in activeReviewEvidenceRun.reviewerAgents" :key="reviewer.id" class="snapshot-row" :data-status="reviewer.status">
              <strong>{{ reviewer.label }}</strong>
              <p>{{ reviewer.mandate }}</p>
              <small>{{ reviewer.requiredActions.slice(0, 2).join(" | ") }}</small>
            </article>
          </details>
        </template>
        <p v-else class="sidebar-hint">Refresh to create an evidence review snapshot for the current document.</p>
      </section>
      <section class="release-evidence-dashboard" :data-status="releaseEvidenceDashboard.status" aria-label="Release evidence dashboard">
        <header>
          <h3>Release evidence</h3>
          <span>{{ releaseEvidenceDashboard.summary }}</span>
        </header>
        <p>Track complete, blocked, manual, credentialed, cross-platform, stale, and ready-to-send evidence before distribution.</p>
        <div class="release-evidence-metrics" aria-label="Release evidence lane counts">
          <span><strong>{{ releaseEvidenceDashboard.counts.complete }}</strong> complete</span>
          <span><strong>{{ releaseEvidenceDashboard.counts.blocked }}</strong> blocked</span>
          <span><strong>{{ releaseEvidenceDashboard.counts.manual }}</strong> manual</span>
          <span><strong>{{ releaseEvidenceDashboard.counts.credentialed }}</strong> credentialed</span>
          <span><strong>{{ releaseEvidenceDashboard.counts["cross-platform"] }}</strong> cross-platform</span>
          <span><strong>{{ releaseEvidenceDashboard.counts.stale }}</strong> stale</span>
          <span><strong>{{ releaseEvidenceDashboard.counts["ready-to-send"] }}</strong> ready</span>
        </div>
        <div class="release-readiness-actions">
          <button type="button" @click="openConfigurationSetup('release')">Setup release evidence</button>
          <button type="button" @click="insertReleaseEvidenceDashboard">Insert evidence dashboard</button>
          <button type="button" @click="insertProductionReadinessWorkOrders">Insert work orders</button>
        </div>
        <section class="production-readiness-work-orders" aria-label="Production readiness work orders">
          <header>
            <h4>Production readiness work orders</h4>
            <span>{{ productionReadinessWorkOrders.length }} open</span>
          </header>
          <article v-for="workOrder in productionReadinessWorkOrders.slice(0, 6)" :key="workOrder.id" class="snapshot-row" :data-status="workOrder.priority">
            <strong>{{ workOrder.title }}</strong>
            <p>{{ workOrder.owner }} | {{ workOrder.command }}</p>
            <small>{{ workOrder.priority }} | {{ workOrder.lane }} | {{ workOrder.acceptanceEvidence }}</small>
          </article>
          <p v-if="!productionReadinessWorkOrders.length" class="sidebar-hint">All production-readiness evidence lanes are closed or ready to send.</p>
        </section>
        <article
          v-for="item in releaseEvidenceDashboard.items"
          :key="item.id"
          class="snapshot-row"
          :data-status="item.lane"
        >
          <strong>{{ item.label }}</strong>
          <p>{{ item.detail }}</p>
          <small>{{ item.lane }} | {{ item.action }}</small>
        </article>
      </section>
      <section class="release-readiness-checklist" aria-label="Release readiness checklist">
        <header>
          <h3>Release readiness</h3>
          <span>{{ releaseChecklistSummary }}</span>
        </header>
        <p>{{ releaseChecklistHelp }}</p>
        <div class="release-readiness-actions">
          <button type="button" @click="applyReleaseMetadataScaffold">Prepare release metadata</button>
          <button type="button" @click="insertReleaseReadinessAudit">Insert release audit</button>
        </div>
        <article
          v-for="item in releaseReadinessChecklist"
          :key="item.id"
          class="snapshot-row"
          :data-status="item.status"
        >
          <strong>{{ item.label }}</strong>
          <p>{{ item.detail }}</p>
          <small>{{ item.action }}</small>
        </article>
      </section>
      <h3>Release</h3>
      <label>
        Status
        <select :value="String(active.compile?.semantic.status || 'draft')" @change="setDocumentStatus(inputValue($event))">
          <option v-for="status in releaseStatuses" :key="status" :value="status">{{ status }}</option>
        </select>
      </label>
      <label>
        Version
        <input :value="String(active.compile?.metadata.version || '')" @input="setFrontMatterField('version', inputValue($event))" @change="setFrontMatterField('version', inputValue($event))" />
      </label>
      <label>
        Document set
        <input :value="String(active.compile?.metadata.documentSet || '')" @input="setFrontMatterField('documentSet', inputValue($event))" @change="setFrontMatterField('documentSet', inputValue($event))" />
      </label>
      <label>
        Owner
        <input :value="String(active.compile?.metadata.owner || '')" @input="setFrontMatterField('owner', inputValue($event))" @change="setFrontMatterField('owner', inputValue($event))" />
      </label>
      <label>
        Release target
        <input :value="String(active.compile?.metadata.releaseTarget || '')" @input="setFrontMatterField('releaseTarget', inputValue($event))" @change="setFrontMatterField('releaseTarget', inputValue($event))" />
      </label>
      <label>
        Approved by
        <input :value="String(active.compile?.metadata.approvedBy || '')" @input="setFrontMatterField('approvedBy', inputValue($event))" @change="setFrontMatterField('approvedBy', inputValue($event))" />
      </label>
      <label>
        Approved at
        <input :value="String(active.compile?.metadata.approvedAt || '')" @input="setFrontMatterField('approvedAt', inputValue($event))" @change="setFrontMatterField('approvedAt', inputValue($event))" />
      </label>
      <label>
        Source confidence
        <input :value="String(active.compile?.metadata.sourceConfidence || active.compile?.metadata.source_confidence || '')" @input="setFrontMatterField('sourceConfidence', inputValue($event))" @change="setFrontMatterField('sourceConfidence', inputValue($event))" />
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

    <!-- ── Backlinks panel ──────────────────────────────────────────────── -->
    <template v-else-if="store.sidebar === 'backlinks'">
      <h2>Backlinks</h2>
      <div class="sidebar-toolbar">
        <button type="button" @click="refreshBacklinks" :disabled="backlinksLoading" title="Refresh backlinks">↻ Refresh</button>
      </div>
      <section class="backlinks-panel" aria-label="Documents linking to this document">
        <div v-if="backlinksLoading" class="sidebar-loading">Scanning workspace…</div>
        <template v-else>
          <div class="backlinks-group" v-if="backlinksData.length">
            <h3>Linked ({{ backlinksData.length }})</h3>
            <button
              v-for="bl in backlinksData"
              :key="bl.source_path + ':' + bl.line"
              type="button"
              class="backlink-item"
              :title="bl.excerpt"
              @click="store.openPath(bl.source_path)"
            >
              <span class="backlink-file">{{ bl.source_path.split('/').pop() }}</span>
              <small class="backlink-line">line {{ bl.line }}</small>
              <span class="backlink-excerpt">{{ bl.excerpt }}</span>
            </button>
          </div>
          <p v-else class="sidebar-hint">No documents in this workspace link to <em>{{ active?.title }}</em>.</p>
          <div class="backlinks-group" v-if="unlinkedMentionsData.length" style="margin-top:12px">
            <h3>Unlinked mentions ({{ unlinkedMentionsData.length }})</h3>
            <button
              v-for="bl in unlinkedMentionsData"
              :key="bl.source_path + ':' + bl.line + ':unlinked'"
              type="button"
              class="backlink-item"
              :title="bl.excerpt"
              @click="store.openPath(bl.source_path)"
            >
              <span class="backlink-file">{{ bl.source_path.split('/').pop() }}</span>
              <small class="backlink-line">line {{ bl.line }}</small>
              <span class="backlink-excerpt">{{ bl.excerpt }}</span>
            </button>
          </div>
        </template>
      </section>
    </template>

    <!-- ── Tasks panel ──────────────────────────────────────────────────── -->
    <template v-else-if="store.sidebar === 'tasks'">
      <h2>Tasks</h2>
      <div class="sidebar-toolbar tasks-toolbar">
        <button type="button" @click="refreshWorkspaceTasks" :disabled="tasksLoading" title="Refresh tasks">↻</button>
        <button type="button" :class="{ active: tasksFilterStatus === 'all' }" @click="tasksFilterStatus = 'all'">All</button>
        <button type="button" :class="{ active: tasksFilterStatus === 'todo' }" @click="tasksFilterStatus = 'todo'">Todo</button>
        <button type="button" :class="{ active: tasksFilterStatus === 'done' }" @click="tasksFilterStatus = 'done'">Done</button>
      </div>
      <div class="tasks-tag-filter" v-if="allTaskTags.length">
        <select v-model="tasksFilterTag" aria-label="Filter by tag">
          <option value="">All tags</option>
          <option v-for="tag in allTaskTags" :key="tag" :value="tag">#{{ tag }}</option>
        </select>
      </div>
      <section class="tasks-panel" aria-label="Workspace tasks">
        <div v-if="tasksLoading" class="sidebar-loading">Scanning workspace…</div>
        <template v-else>
          <p v-if="!filteredTasks.length" class="sidebar-hint">
            {{ workspaceTasks.length ? 'No tasks match the current filter.' : 'No checkboxes found in workspace. Use - [ ] to create tasks.' }}
          </p>
          <div class="task-group" v-for="(groupTasks, groupKey) in filteredTasks.reduce((acc: Record<string, typeof filteredTasks>, t) => { (acc[t.file_path] = acc[t.file_path] || []).push(t); return acc; }, {})" :key="String(groupKey)">
            <h4 class="task-group-header">{{ String(groupKey).split('/').pop() }}</h4>
            <label
              v-for="task in groupTasks as typeof filteredTasks"
              :key="task.file_path + ':' + task.line"
              class="task-item"
              :class="{ done: task.done }"
            >
              <input type="checkbox" :checked="task.done" disabled />
              <span class="task-text">{{ task.text }}</span>
              <span v-if="task.due_date" class="task-due">{{ task.due_date }}</span>
              <button type="button" class="task-goto" @click="store.openPath(task.file_path)" title="Open file">→</button>
            </label>
          </div>
        </template>
      </section>
    </template>

    <!-- ── Daily Notes panel ────────────────────────────────────────────── -->
    <template v-else-if="store.sidebar === 'daily-notes'">
      <h2>Daily Notes</h2>
      <div class="sidebar-toolbar">
        <button type="button" @click="openTodayNote" class="primary" title="Open today's note (⌘⇧D)">Today</button>
      </div>
      <section class="daily-notes-panel" aria-label="Daily notes calendar">
        <div class="daily-notes-calendar-nav">
          <button type="button" @click="if (dailyNotesCalendarMonth === 1) { dailyNotesCalendarYear--; dailyNotesCalendarMonth = 12; } else dailyNotesCalendarMonth--">‹</button>
          <span>{{ ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'][dailyNotesCalendarMonth - 1] }} {{ dailyNotesCalendarYear }}</span>
          <button type="button" @click="if (dailyNotesCalendarMonth === 12) { dailyNotesCalendarYear++; dailyNotesCalendarMonth = 1; } else dailyNotesCalendarMonth++">›</button>
        </div>
        <div class="daily-notes-calendar-grid" role="grid" aria-label="Daily notes calendar">
          <div class="cal-dow" v-for="d in ['Su','Mo','Tu','We','Th','Fr','Sa']" :key="d">{{ d }}</div>
          <button
            v-for="cell in dailyNotesCalendarGrid"
            :key="cell.date || 'empty-' + cell.day"
            type="button"
            :class="['cal-day', { 'cal-empty': cell.empty, 'cal-has-note': cell.hasNote, 'cal-today': cell.isToday }]"
            :disabled="cell.empty"
            :aria-label="cell.date ? `Open note for ${cell.date}` : undefined"
            @click="cell.date && openDailyNoteForDate(cell.date)"
          >{{ cell.day || '' }}</button>
        </div>
        <p class="sidebar-hint" v-if="!store.workspaceRoot">Open a workspace folder to enable daily notes.</p>
      </section>
    </template>
    <template v-if="store.sidebar === 'help'">
      <slot name="help-panel" />
    </template>
    <template v-else-if="store.sidebar === 'settings'">
      <slot name="settings-panel" />
    </template>
  </aside>
</template>

<script setup lang="ts">
import { computed, inject } from 'vue';
import { useDocumentsStore } from '../stores/documents';

const props = defineProps<{
  writingSpaceMaximized: boolean;
  sidebarCollapsed: boolean;
  sidebarWidth: number;
}>();

const emit = defineEmits<{
  'toggle-sidebar-collapsed': [];
  'update:sidebarWidth': [val: number];
}>();

const store = useDocumentsStore();

const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  PRESENTATION_THEMES,
  PRESENTATION_TRANSITIONS,
  academicTemplatesByCategory,
  active,
  activeAgentControlCenter,
  activeClaimSourceMatchByLine,
  activeClaimSourceMatches,
  activeDocumentSet,
  activeExportProfile,
  activeLatexTemplateProfile,
  activePublishingDestination,
  activeReviewEvidenceRun,
  addDocumentVariable,
  addFrontMatterDataSource,
  addIndexExclusion,
  addTableColumn,
  addTableFormulaRow,
  addTableRow,
  addTableTotalsRow,
  allDocumentOutlineTemplates,
  allTaskTags,
  allVersionedBusinessClauses,
  analyzeCurrentRfpSource,
  appendAllExportStepAssistance,
  appendAllQualityStepAssistance,
  appendAllTransformTemplateAssistance,
  appendCustomTableFormulaRow,
  appendDocumentOutlineTemplate,
  appendExportStepAssistance,
  appendOutlineToDocument,
  appendQualityStepAssistance,
  appendRfpWizardSuggestion,
  appendTransformTemplateAssistance,
  applyCoverBuilderMetadata,
  applyCoverBuilderPackage,
  applyCslFilePath,
  applyExportMetadataScaffold,
  applyReleaseMetadataScaffold,
  applyTableCellSpan,
  applyTableDraft,
  applyTableSourceEdit,
  applyTableTextCellEdit,
  applyTocSettings,
  assignActiveDocumentSet,
  availableLatexTemplateProfiles,
  backlinksData,
  backlinksLoading,
  bibliographyByKey,
  bibliographyEntryStub,
  bibliographySnippet,
  bibliographyTemplateSnippet,
  businessDocumentSnippets,
  businessProfileCompletion,
  businessSnippetQuery,
  businessTemplateQuery,
  businessWizardStepAssistance,
  calloutPresetMarkdown,
  calloutPresets,
  canEditFigureSource,
  canEditMarkdownTableText,
  canGoToTableSource,
  canNavigateDiagnostic,
  cancelTableDraft,
  captionManagerSummary,
  captionedReferenceItems,
  changeNoteText,
  chartDesignerDraft,
  chartDesignerKindOptions,
  chartDesignerPreviewMarkdown,
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
  clearActiveDocumentSet,
  clearTableCellSpan,
  compilerOutputInventory,
  confirm,
  copyActiveDocumentSetManifest,
  copyCitationSourceLibraryAudit,
  copyCitationSourcePath,
  copyCitationTodoAudit,
  copyCurrentTemplatePack,
  copyFrontMatterDataSourceFile,
  copyPublishingContent,
  copyPublishingPayload,
  copyPublishingPreflightAudit,
  coverBuilderDefaults,
  coverBuilderDraft,
  coverBuilderSummary,
  createDocumentFromOutline,
  createIncludeChildDocument,
  createRecoverySnapshot,
  createResponsiveRfpResponse,
  createRfpProposalOutline,
  createTableDraft,
  crossReferenceManagerSummary,
  crossReferenceRows,
  currentTemplatePackJson,
  currentTemplatePackRows,
  customClauseDraft,
  customClauseStaleMarkersText,
  customOutlineBestFor,
  customOutlineDraft,
  customOutlineTags,
  customSnippetDraft,
  customTemplateDraft,
  customTemplateFillFields,
  customTemplateIsValid,
  customTemplateTags,
  dailyNotesCalendarGrid,
  dailyNotesCalendarMonth,
  dailyNotesCalendarYear,
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
  deleteActiveExportProfile,
  deleteActivePublishingDestination,
  deleteEditingCustomBusinessSnippet,
  deleteEditingCustomVersionedClause,
  diagnosticAnnouncementLabel,
  diagnosticLocation,
  docsLiveDocumentTypes,
  documentLayoutPresets,
  documentMapBlockersOnly,
  documentMapCountByFilter,
  documentMapFilter,
  documentMapFilterOptions,
  documentMapKindLabel,
  documentMapQuery,
  documentMapSummary,
  documentOutlineTemplateToPlannerText,
  documentSetDraft,
  documentSetGroups,
  documentSetRenameDraft,
  documentVariableFilterDraft,
  documentVariableFilterOptions,
  documentVariableManagerSummary,
  documentVariableNameDraft,
  documentVariableValueDraft,
  downloadAllCitationSources,
  downloadCitationSource,
  duplicateBibliographyEntries,
  duplicateTableColumn,
  duplicateTableRow,
  duplicateTransformTemplate,
  editCustomBusinessSnippet,
  editCustomOutlineTemplate,
  editCustomTransformTemplate,
  editCustomVersionedClause,
  editLatexTemplate,
  editSelectedTableInMarkdownText,
  editingCustomClauseId,
  editingCustomSnippetId,
  editingCustomTemplateId,
  enableFrontMatterToc,
  eventValue,
  exportDistributionChecklist,
  exportDistributionChecklistHelp,
  exportDistributionChecklistSummary,
  exportDocument,
  exportDocumentAs,
  exportProfileName,
  exportProfileSummary,
  exportReadinessNotes,
  exportSelectedTable,
  exportStepAssistance,
  exportVisualQaCurrentRow,
  exportVisualQaDashboard,
  figureBlocks,
  figureCropPointStyle,
  figureCropPositions,
  figureCropPreviewStyle,
  figureCropReticleStyle,
  filteredBusinessSnippets,
  filteredBusinessTemplates,
  filteredDocumentMapItems,
  filteredDocumentOutlineTemplates,
  filteredTasks,
  filteredTransformTemplates,
  focusDocumentMapBlockers,
  focusTableGrid,
  focusTableSourceEditor,
  formatCitationSourceBytes,
  frontMatterDataSourceRows,
  frontMatterVariableRows,
  gitFreeVersioningPlan,
  glossaryEntries,
  glossaryManagerSummary,
  glossarySectionSnippet,
  glossarySnippet,
  goToCitationTodo,
  goToCrossReference,
  goToDocumentMapItem,
  goToIncludeDirective,
  goToReferenceLabel,
  goToSearchTerm,
  goToSelectedTableSource,
  goToSourceTarget,
  goToTableTextCellSource,
  handleButtonHelpHitboxEnter,
  hideButtonHelp,
  history,
  importDataSourceAsEditableTable,
  importLatexTemplateLibraryJson,
  importRfpSourceFile,
  importRfpSourceUrl,
  importSelectedSpreadsheetWorksheet,
  importTableFromSpreadsheet,
  importedTemplatePack,
  importedTemplatePackRows,
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
  inputValue,
  insertActiveDocumentSetManifest,
  insertBlock,
  insertBusinessSnippet,
  insertBusinessTemplate,
  insertCalloutPreset,
  insertChangeNote,
  insertCitationReference,
  insertCitationSourceBibliography,
  insertCitationSourceLibraryAudit,
  insertCitationSourceReference,
  insertCitationTodo,
  insertCitationTodoAudit,
  insertClaimEvidenceMatrix,
  insertCoverBuilderSection,
  insertCrossReferenceForLabel,
  insertCurrentTemplatePackManifest,
  insertDataRefreshAudit,
  insertDataSourceTemplate,
  insertDeepResearchAuditPacket,
  insertDeepResearchConflictReview,
  insertDeepResearchDraft,
  insertDeepResearchLog,
  insertDeepResearchSourceQualityReview,
  insertDesignedChart,
  insertDocumentLayoutPreset,
  insertDocumentVariable,
  insertExportReadinessNotes,
  insertExportVisualQaReport,
  insertGlossaryAuditTable,
  insertImportedTemplatePackManifest,
  insertIncludeDirectiveFromBuilder,
  insertIndexAuditTable,
  insertIndexMarkerForTerm,
  insertIndexMarkerFromDraft,
  insertMarkdownAtCursor,
  insertMissingCitationStubs,
  insertProductionReadinessWorkOrders,
  insertPublishingPreflightAudit,
  insertQualityImprovementReport,
  insertQualityReviewNotes,
  insertReleaseEvidenceDashboard,
  insertReleaseReadinessAudit,
  insertReviewComment,
  insertReviewEvidenceAudit,
  insertRfpComplianceMatrix,
  insertRfpSubmissionChecklist,
  insertRfpWinThemes,
  insertSelectedCalloutPreset,
  insertSqlTransformTemplate,
  insertTableDraftInMarkdownText,
  insertTransformTemplate,
  insertTransformTemplateAssistanceNotes,
  insertVersionedClause,
  installPastedTemplatePack,
  installedCslStyles,
  isFormulaCell,
  isNewTableDraft,
  latexTemplateBestForDraft,
  latexTemplateDraft,
  latexTemplateLibraryJsonText,
  latexTemplatePackagesDraft,
  latexTemplateWorkspaceSyncBusy,
  latexTemplateWorkspaceSyncStatus,
  layoutAdvisorDetail,
  layoutAdvisorHeadline,
  layoutAdvisorStatus,
  layoutQualityRecommendations,
  listOfFiguresSnippet,
  listOfTablesSnippet,
  loadActiveDocumentAsRfpSource,
  loadOutlineDraftFromDocument,
  loadSelectedTableIntoChartDesigner,
  loadTableAtCursor,
  loadTableTextCellAtCursor,
  manifestPreview,
  markTableSourceEditDirty,
  markdown,
  markdownTables,
  mergedMetadataVariableRows,
  missingCitationKeys,
  moveTableColumn,
  moveTableRow,
  normalizeFigureCropPosition,
  onFigureCropKeydown,
  onFigureCropPointerDown,
  onFigureCropPointerMove,
  onFigureCropPositionChange,
  open,
  openAgentWorkspace,
  openAgentWorkspaceForBusinessTemplate,
  openAgentWorkspaceForRfpAnalysis,
  openBusinessProfile,
  openCitationTodoCount,
  openConfigurationSetup,
  openDailyNoteForDate,
  openDeepResearchDraftAsDocument,
  openDocsLiveFromOutline,
  openFolder,
  openIncludeChild,
  openPresenterView,
  openQualityAgent,
  openSearchResult,
  openTodayNote,
  outlineDocsLiveTypeLabel,
  outlineDraftIncludeToc,
  outlineDraftItems,
  outlineDraftText,
  outlineDraftTitle,
  outlineHeadings,
  outlineLibraryCategories,
  outlineLibraryCategory,
  outlineLibraryQuery,
  pinFile,
  prepareForExport,
  preparePublishingHandoff,
  previewLatexTemplateLibraryJson,
  printPreviewEnabled,
  printPreviewReport,
  productionReadinessWorkOrders,
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
  qualityImprovementRecommendations,
  qualityRecommendationSummary,
  qualityReviewNotes,
  qualityStepAssistance,
  readinessLayoutSummary,
  redownloadCitationSource,
  ref,
  referenceLabelManagerSummary,
  referenceLabelRows,
  refreshBacklinks,
  refreshCitationSourceLibrary,
  refreshDataSourcesAndInsertAudit,
  refreshDataSourcesPreview,
  refreshReviewEvidenceSnapshot,
  refreshTableSourceEditFromDraft,
  refreshWorkspaceTasks,
  releaseChecklistHelp,
  releaseChecklistSummary,
  releaseEvidenceDashboard,
  releaseReadinessChecklist,
  releaseStatuses,
  reloadTableDraftFromSource,
  removeIndexExclusion,
  removeTableColumn,
  removeTableRow,
  renameActiveDocumentSet,
  replaceTableFromPaste,
  resetChartDesignerForType,
  resetCoverBuilderDraft,
  resetCustomClauseDraft,
  resetCustomOutlineDraft,
  resetCustomSnippetDraft,
  resetLatexTemplateDraft,
  resolveCitationTodoItem,
  resolvedCitationEntries,
  restoreSnapshot,
  revealCitationSource,
  reviewCommentText,
  reviewEvidenceSnapshotSummary,
  reviewSummary,
  rfpAnalysis,
  rfpAnalysisSummary,
  rfpImportBusy,
  rfpImportMessage,
  rfpResponseContextNotes,
  rfpSourceKind,
  rfpSourceText,
  rfpSourceUrl,
  rfpWizardStepAssistance,
  runAgentControlAction,
  runAgentPlanDistribution,
  runAgentPlanReview,
  runDeepResearchDocumentCreation,
  runLayoutQualityReview,
  runQualityReview,
  runWorkspaceSearch,
  saveCurrentOutlineTemplate,
  saveCustomBusinessSnippet,
  saveCustomTransformTemplate,
  saveCustomVersionedClause,
  saveExportProfileFromPanel,
  saveLatexTemplateDraft,
  saveLatexTemplatesToWorkspace,
  savePublishingDestinationProfile,
  searchCitationSources,
  selectExportProfile,
  selectPublishingDestination,
  selectTableForEditing,
  selectedCalloutPreset,
  selectedCalloutPresetId,
  selectedTableEditSummary,
  selectedTableIndex,
  selectedTableSpanCell,
  sendOutlineTemplateToDocsLive,
  sendPublishingPayload,
  sendRfpResponseToDocsLive,
  setApprovalTimestampNow,
  setCitationStyle,
  setDeepResearchTargetPages,
  setDocumentStatus,
  setFrontMatterField,
  setupPARAWorkspace,
  sidebarCollapsed,
  snapshotActive,
  sortTableRows,
  spreadsheetColumnName,
  startBusinessDocumentWizard,
  startNewCustomTemplate,
  styleGuideFindings,
  syncWorkspaceLatexTemplates,
  syncWorkspaceOutlines,
  tableCellLabel,
  tableColumnTotals,
  tableCursorCellPreview,
  tableCursorCellSummary,
  tableDataBusy,
  tableDataRowCount,
  tableDraft,
  tableDraftDirty,
  tableDraftHasErrors,
  tableDraftIssues,
  tableDraftSourceChanged,
  tableEditorGrid,
  tableFollowSourceCursor,
  tableFormulaEndRow,
  tableFormulaFunction,
  tableFormulaLabel,
  tableFormulaPreview,
  tableFormulaStartRow,
  tableFormulaTargetColumn,
  tableFormulaTargetColumns,
  tableHeaderLabel,
  tableImportSelectedSheetIndex,
  tableImportSheetNames,
  tableImportSourceLabel,
  tablePasteText,
  tableSourceEditDirty,
  tableSourceEditError,
  tableSourceEditSummary,
  tableSourceEditText,
  tableSourceEditor,
  tableSourceSyncMessage,
  tableSpanCellOptions,
  tableSpanColspan,
  tableSpanMaxColspan,
  tableSpanMaxRowspan,
  tableSpanPreview,
  tableSpanRowspan,
  tableTextCellEdit,
  tableTextCellEditSummary,
  tableTextCellError,
  tableTextCellValue,
  tableTotalLabel,
  tableTwoWayHint,
  tableTwoWayStatus,
  tableTwoWayStatusClass,
  tasksFilterStatus,
  tasksFilterTag,
  tasksLoading,
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
  tocDepthDraft,
  tocDepthOptions,
  tocManagerSummary,
  tocNumberedDraft,
  tocSnippet,
  toggleAiSectionReview,
  toggleAiSourceReview,
  togglePrintPreview,
  toolbarIconPaths,
  transformTemplateAssistance,
  transformTemplateAssistanceNotes,
  transformTemplateCategoryOptions,
  transformTemplateKindOptions,
  unlinkedMentionsData,
  unpinFile,
  updateTableDraftFromSourceText,
  useDocumentOutlineTemplate,
  versionedClauseAuditItems,
  versionedClauseAuditSummary,
  versioningModeLabel,
  workspaceLatexTemplateLibraryPath,
  workspaceOutlineSyncBusy,
  workspaceOutlineSyncStatus,
  workspaceSearchBusy,
  workspaceSearchQuery,
  workspaceSearchResults,
  workspaceTasks,
  writingSpaceMaximized,
} = _ctx;

const PANEL_LABELS: Record<string, string> = {
  files: 'Files', outline: 'Outline', diagnostics: 'Diagnostics', layout: 'Layout',
  tables: 'Tables', templates: 'Templates', references: 'References',
  exports: 'Export', versioning: 'Versioning', review: 'Review', help: 'Help', settings: 'Settings',
};

const currentPanelLabel = computed(() => PANEL_LABELS[store.sidebar as string] || String(store.sidebar));

function onSidebarResizeStart(event: MouseEvent): void {
  const startX = event.clientX;
  const startW = props.sidebarWidth;
  const onMove = (e: MouseEvent) => {
    emit('update:sidebarWidth', Math.max(160, Math.min(480, startW + (e.clientX - startX))));
  };
  const onUp = () => {
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  };
  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}

function toggleSidebarCollapsed(): void {
  emit('toggle-sidebar-collapsed');
}
</script>
