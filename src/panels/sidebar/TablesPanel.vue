<template>
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

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  addTableColumn,
  addTableFormulaRow,
  addTableRow,
  addTableTotalsRow,
  appendCustomTableFormulaRow,
  applyTableCellSpan,
  applyTableDraft,
  applyTableSourceEdit,
  applyTableTextCellEdit,
  canEditMarkdownTableText,
  canGoToTableSource,
  cancelTableDraft,
  clearTableCellSpan,
  createTableDraft,
  duplicateTableColumn,
  duplicateTableRow,
  editSelectedTableInMarkdownText,
  exportSelectedTable,
  focusTableGrid,
  focusTableSourceEditor,
  goToSelectedTableSource,
  goToTableTextCellSource,
  handleButtonHelpHitboxEnter,
  hideButtonHelp,
  importSelectedSpreadsheetWorksheet,
  importTableFromSpreadsheet,
  inputValue,
  insertSqlTransformTemplate,
  insertTableDraftInMarkdownText,
  isFormulaCell,
  isNewTableDraft,
  loadTableAtCursor,
  loadTableTextCellAtCursor,
  markTableSourceEditDirty,
  markdownTables,
  moveTableColumn,
  moveTableRow,
  refreshTableSourceEditFromDraft,
  reloadTableDraftFromSource,
  removeTableColumn,
  removeTableRow,
  replaceTableFromPaste,
  selectTableForEditing,
  selectedTableEditSummary,
  selectedTableIndex,
  selectedTableSpanCell,
  sortTableRows,
  spreadsheetColumnName,
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
  updateTableDraftFromSourceText,
} = _ctx;
</script>
