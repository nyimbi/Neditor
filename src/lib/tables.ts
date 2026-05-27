export interface MarkdownTable {
  startLine: number;
  endLine: number;
  captionLine?: number;
  id: string;
  caption: string;
  headers: string[];
  alignments: TableAlignment[];
  rows: string[][];
}

export type TableAlignment = "left" | "center" | "right";
export type TableFormat = "text" | "number" | "currency" | "percent" | "date";
export type TableSortDirection = "asc" | "desc";
export type TableFormulaFunction = "SUM" | "AVG" | "MIN" | "MAX" | "COUNT";

export interface TableDraft {
  id: string;
  caption: string;
  headers: string[];
  alignments: TableAlignment[];
  formats: TableFormat[];
  rows: string[][];
}

export interface TableSourceSnapshot {
  documentId: string;
  tableIndex: number;
  startLine: number;
  endLine: number;
  sourceText: string;
  draftMarkdown: string;
}

export interface MarkdownTableSnapshotMatch {
  table: MarkdownTable;
  index: number;
}

export interface TableDocumentTextSyncOptions {
  text: string;
  documentId: string;
  tables?: MarkdownTable[];
  snapshot?: TableSourceSnapshot | null;
  fallbackIndex?: number;
}

export interface TableDocumentTextSyncResult {
  table: MarkdownTable;
  index: number;
  draft: TableDraft;
  sourceText: string;
  snapshot: TableSourceSnapshot;
}

export interface TableDraftIssue {
  severity: "warning" | "error";
  message: string;
}

export interface ParsedTablePaste {
  rows: string[][];
  alignments?: TableAlignment[];
  id?: string;
  caption?: string;
}

export interface ParsedTableSourceDraft {
  draft: TableDraft;
  sourceText: string;
}

export interface TableExportMarkdownOptions {
  draftMarkdown?: string | null;
  documentText?: string | null;
  sourceEditDirty?: boolean;
  sourceEditText?: string | null;
}

export interface TableExportMarkdownSelection {
  markdown: string;
  source: "source-edit" | "draft" | "document" | "none";
  sourceEditValid: boolean;
}

export interface TableDraftFromRowsOptions {
  id?: string;
  caption?: string;
  fallbackId?: string;
  fallbackCaption?: string;
  alignments?: TableAlignment[];
}

export interface TableDraftFromPasteOptions {
  fallbackId?: string;
  fallbackCaption?: string;
  caption?: string;
}

export interface TableFormulaRowOptions {
  formula: TableFormulaFunction;
  targetColumn: number;
  startRow: number;
  endRow: number;
  label?: string;
}

export interface TableFormulaTargetOption {
  index: number;
  label: string;
}

export interface TableCellSpan {
  text: string;
  colspan: number;
  rowspan: number;
}

export interface TableCellSpanSelection {
  rowIndex: number;
  columnIndex: number;
  colspan: number;
  rowspan: number;
}

export interface TableSpanCellOption {
  value: string;
  label: string;
}

export function parseMarkdownTables(text: string): MarkdownTable[] {
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

export function findMarkdownTableIndexForLineRange(tables: MarkdownTable[], fromLine: number, toLine = fromLine) {
  const start = Math.max(1, Math.min(fromLine, toLine));
  const end = Math.max(1, Math.max(fromLine, toLine));
  return tables.findIndex((table) => {
    const tableStart = table.captionLine || table.startLine;
    return end >= tableStart && start <= table.endLine;
  });
}

export function markdownTableToDraft(table: MarkdownTable): TableDraft {
  return {
    id: table.id,
    caption: table.caption,
    headers: [...table.headers],
    alignments: [...table.alignments],
    formats: table.headers.map((_, columnIndex) => inferTableFormat(table.rows.map((row) => row[columnIndex] || ""))),
    rows: table.rows.map((row) => padTableRow(row, table.headers.length)),
  };
}

export function replaceMarkdownTableInText(text: string, table: MarkdownTable, draft: TableDraft) {
  const lines = text.split("\n");
  const replaceStart = table.captionLine || table.startLine;
  const normalizedDraft = normalizeTableDraft(draft);
  const serialized = serializeMarkdownTable(normalizedDraft);
  lines.splice(replaceStart - 1, table.endLine - replaceStart + 1, ...serialized);
  return {
    text: lines.join("\n"),
    draft: normalizedDraft,
    startLine: replaceStart,
    endLine: replaceStart + serialized.length - 1,
  };
}

export function replaceMarkdownTableSnapshotInText(text: string, snapshot: TableSourceSnapshot, draft: TableDraft) {
  const lines = text.split("\n");
  const replaceStart = clampInteger(snapshot.startLine, 1, Math.max(1, lines.length + 1));
  const replaceEnd = clampInteger(snapshot.endLine, replaceStart, Math.max(replaceStart, lines.length));
  const normalizedDraft = normalizeTableDraft(draft);
  const serialized = serializeMarkdownTable(normalizedDraft);
  lines.splice(replaceStart - 1, replaceEnd - replaceStart + 1, ...serialized);
  return {
    text: lines.join("\n"),
    draft: normalizedDraft,
    startLine: replaceStart,
    endLine: replaceStart + serialized.length - 1,
  };
}

export function tableSourceText(text: string, table: MarkdownTable) {
  const lines = text.split("\n");
  const startLine = table.captionLine || table.startLine;
  return lines.slice(startLine - 1, table.endLine).join("\n");
}

export function tableDraftMarkdown(draft: TableDraft) {
  return serializeMarkdownTable(normalizeTableDraft(draft)).join("\n");
}

export function createTableSourceSnapshot(
  text: string,
  documentId: string,
  tableIndex: number,
  table: MarkdownTable,
  draft: TableDraft,
): TableSourceSnapshot {
  return {
    documentId,
    tableIndex,
    startLine: table.captionLine || table.startLine,
    endLine: table.endLine,
    sourceText: tableSourceText(text, table),
    draftMarkdown: tableDraftMarkdown(draft),
  };
}

export function tableSourceChanged(
  text: string,
  table: MarkdownTable | null | undefined,
  snapshot: TableSourceSnapshot | null | undefined,
  documentId: string,
  isNewDraft = false,
) {
  if (!snapshot || isNewDraft || snapshot.documentId !== documentId) return false;
  if (!table) return true;
  return tableSourceText(text, table) !== snapshot.sourceText;
}

export function tableOverlapsSourceSnapshot(
  table: MarkdownTable | null | undefined,
  snapshot: TableSourceSnapshot | null | undefined,
  documentId: string,
) {
  if (!table || !snapshot || snapshot.documentId !== documentId) return false;
  const tableStart = table.captionLine || table.startLine;
  return tableStart <= snapshot.endLine && table.endLine >= snapshot.startLine;
}

export function findMarkdownTableForSourceSnapshot(
  tables: MarkdownTable[],
  text: string,
  snapshot: TableSourceSnapshot | null | undefined,
  documentId: string,
): MarkdownTableSnapshotMatch | null {
  if (!snapshot || snapshot.documentId !== documentId) return null;

  const indexed = tables.map((table, index) => ({ table, index }));
  const [snapshotTable] = parseMarkdownTables(snapshot.sourceText);
  const indexedTable = indexed[snapshot.tableIndex];
  if (
    indexedTable &&
    tableOverlapsSourceSnapshot(indexedTable.table, snapshot, documentId) &&
    tableMatchesSnapshotIdentity(indexedTable.table, snapshotTable)
  ) {
    return indexedTable;
  }

  const overlapping = indexed.find(
    ({ table }) => tableOverlapsSourceSnapshot(table, snapshot, documentId) && tableMatchesSnapshotIdentity(table, snapshotTable),
  );
  if (overlapping) return overlapping;

  const snapshotId = snapshotTable?.id || "";
  if (snapshotId) {
    const byId = indexed.find(({ table }) => table.id === snapshotId);
    if (byId) return byId;
  }

  const bySource = indexed.find(({ table }) => tableSourceText(text, table) === snapshot.sourceText);
  if (bySource) return bySource;

  if (snapshotTable?.caption) {
    const byCaptionAndHeaders = indexed.find(
      ({ table }) =>
        table.caption === snapshotTable.caption &&
        table.headers.length === snapshotTable.headers.length &&
        table.headers.every((header, index) => header === snapshotTable.headers[index]),
    );
    if (byCaptionAndHeaders) return byCaptionAndHeaders;
  }

  const editedInPlace = indexed.find(({ table }) => {
    const tableStart = table.captionLine || table.startLine;
    return tableStart === snapshot.startLine && tableOverlapsSourceSnapshot(table, snapshot, documentId);
  });
  if (editedInPlace) return editedInPlace;

  return null;
}

export function syncTableDraftFromDocumentText(options: TableDocumentTextSyncOptions): TableDocumentTextSyncResult | null {
  const tables = options.tables ?? parseMarkdownTables(options.text);
  const fallbackIndex = clampInteger(options.fallbackIndex ?? 0, 0, Math.max(0, tables.length - 1));
  const match = options.snapshot
    ? findMarkdownTableForSourceSnapshot(tables, options.text, options.snapshot, options.documentId)
    : tables[fallbackIndex]
      ? { table: tables[fallbackIndex], index: fallbackIndex }
      : null;
  if (!match) return null;
  const draft = markdownTableToDraft(match.table);
  return {
    table: match.table,
    index: match.index,
    draft,
    sourceText: tableSourceText(options.text, match.table),
    snapshot: createTableSourceSnapshot(options.text, options.documentId, match.index, match.table, draft),
  };
}

function tableMatchesSnapshotIdentity(table: MarkdownTable, snapshotTable: MarkdownTable | undefined) {
  if (!snapshotTable) return true;
  if (snapshotTable.id) return table.id === snapshotTable.id;
  if (!snapshotTable.caption) return true;
  return (
    table.caption === snapshotTable.caption &&
    table.headers.length === snapshotTable.headers.length &&
    table.headers.every((header, index) => header === snapshotTable.headers[index])
  );
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
  const trimmed = line.trim();
  if (!trimmed) return false;
  const pipeCount = unescapedPipeCount(trimmed);
  const cells = splitMarkdownTableRow(trimmed);
  return cells.length >= 2 || (trimmed.startsWith("|") && hasUnescapedTrailingPipe(trimmed) && pipeCount >= 2 && cells.length >= 1);
}

function isMarkdownTableSeparator(line: string) {
  return isMarkdownTableRow(line) && splitMarkdownTableRow(line).every((cell) => /^:?-{3,}:?$/.test(cell.replace(/\s/g, "")));
}

function splitMarkdownTableRow(line: string) {
  const cells: string[] = [];
  let cell = "";
  let escaped = false;
  for (const char of stripOptionalOuterPipes(line.trim())) {
    if (escaped) {
      cell += char === "|" ? "|" : `\\${char}`;
      escaped = false;
    } else if (char === "\\") {
      escaped = true;
    } else if (char === "|") {
      cells.push(cell.trim());
      cell = "";
    } else {
      cell += char;
    }
  }
  if (escaped) cell += "\\";
  cells.push(cell.trim());
  return cells;
}

function stripOptionalOuterPipes(line: string) {
  let inner = line.startsWith("|") ? line.slice(1) : line;
  if (hasUnescapedTrailingPipe(inner)) inner = inner.slice(0, -1);
  return inner;
}

function hasUnescapedTrailingPipe(line: string) {
  if (!line.endsWith("|")) return false;
  let slashCount = 0;
  for (let index = line.length - 2; index >= 0 && line[index] === "\\"; index -= 1) {
    slashCount += 1;
  }
  return slashCount % 2 === 0;
}

function alignmentFromSeparator(cell: string): TableAlignment {
  const compact = cell.replace(/\s/g, "");
  if (compact.startsWith(":") && compact.endsWith(":")) return "center";
  if (compact.endsWith(":")) return "right";
  return "left";
}

function unescapedPipeCount(line: string) {
  let count = 0;
  let escaped = false;
  for (const char of line) {
    if (escaped) {
      escaped = false;
    } else if (char === "\\") {
      escaped = true;
    } else if (char === "|") {
      count += 1;
    }
  }
  return count;
}

export function padAlignments(alignments: TableAlignment[], length: number) {
  return Array.from({ length }, (_, index) => alignments[index] || "left");
}

export function padTableRow(row: string[], length: number) {
  return Array.from({ length }, (_, index) => row[index] || "");
}

export function normalizeTableDraft(draft: TableDraft): TableDraft {
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

export function createDefaultTableDraft(): TableDraft {
  return {
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

export function tableDraftFromMarkdownSource(sourceText: string): ParsedTableSourceDraft | null {
  const [table] = parseMarkdownTables(sourceText);
  if (!table) return null;
  const draft = markdownTableToDraft(table);
  return {
    draft,
    sourceText: tableDraftMarkdown(draft),
  };
}

export function tableMarkdownForExport(options: TableExportMarkdownOptions): TableExportMarkdownSelection {
  const sourceEditText = options.sourceEditText?.trim() || "";
  if (options.sourceEditDirty) {
    return tableDraftFromMarkdownSource(sourceEditText)
      ? { markdown: sourceEditText, source: "source-edit", sourceEditValid: true }
      : { markdown: "", source: "source-edit", sourceEditValid: false };
  }
  const draftMarkdown = options.draftMarkdown?.trim() || "";
  if (draftMarkdown) return { markdown: draftMarkdown, source: "draft", sourceEditValid: true };
  const documentText = options.documentText?.trim() || "";
  if (documentText) return { markdown: documentText, source: "document", sourceEditValid: true };
  return { markdown: "", source: "none", sourceEditValid: true };
}

export function tableDraftFromRows(rows: string[][], options: TableDraftFromRowsOptions = {}): TableDraft | null {
  if (!rows.length) return null;
  const headers = rows[0].map((cell, index) => cell.trim() || `Column ${index + 1}`);
  const bodyRows = rows.slice(1).map((row) => padTableRow(row, headers.length));
  const draftRows = bodyRows.length ? bodyRows : [headers.map(() => "")];
  return {
    id: options.id ?? options.fallbackId ?? "",
    caption: options.caption ?? options.fallbackCaption ?? "",
    headers,
    alignments: options.alignments ? padAlignments(options.alignments, headers.length) : headers.map(() => "left"),
    formats: headers.map((_, columnIndex) => inferTableFormat(draftRows.map((row) => row[columnIndex] || ""))),
    rows: draftRows,
  };
}

export function tableDraftFromPasteText(text: string, options: TableDraftFromPasteOptions = {}) {
  const parsed = parseTablePaste(text);
  return tableDraftFromRows(parsed.rows, {
    id: parsed.id,
    caption: options.caption ?? parsed.caption,
    fallbackId: options.fallbackId,
    fallbackCaption: options.fallbackCaption,
    alignments: parsed.alignments,
  });
}

export function tableDraftDataRowCount(draft: TableDraft | null | undefined) {
  if (!draft) return 1;
  return Math.max(1, draft.rows.filter((row) => !isTableSummaryRow(row)).length);
}

export function tableFormulaTargetOptions(draft: TableDraft | null | undefined): TableFormulaTargetOption[] {
  if (!draft) return [];
  const options = draft.headers.map((header, index) => ({
    index,
    label: `${spreadsheetColumnName(index + 1)} - ${header || `Column ${index + 1}`}`,
  }));
  return options.length > 1 ? options.slice(1) : options;
}

export function tableSpanCellOptions(draft: TableDraft | null | undefined): TableSpanCellOption[] {
  if (!draft) return [];
  return draft.rows.flatMap((row, rowIndex) =>
    draft.headers.map((header, columnIndex) => ({
      value: `${rowIndex}:${columnIndex}`,
      label: `${spreadsheetColumnName(columnIndex + 1)}${rowIndex + 1} - ${header || `Column ${columnIndex + 1}`} - ${row[columnIndex] || "blank"}`,
    })),
  );
}

export function tableHeaderLabel(columnIndex: number) {
  return `Column ${spreadsheetColumnName(columnIndex + 1)} header`;
}

export function tableCellLabel(draft: TableDraft | null | undefined, rowIndex: number, columnIndex: number) {
  const header = draft?.headers[columnIndex]?.trim();
  const column = spreadsheetColumnName(columnIndex + 1);
  return header ? `${header}, row ${rowIndex + 1}, column ${column}` : `Row ${rowIndex + 1}, column ${column}`;
}

export function tableTotalLabel(draft: TableDraft | null | undefined, columnIndex: number) {
  const header = draft?.headers[columnIndex]?.trim();
  const column = spreadsheetColumnName(columnIndex + 1);
  return header ? `Total for ${header}, column ${column}` : `Total for column ${column}`;
}

export function addTableDraftRow(draft: TableDraft) {
  draft.rows.push(draft.headers.map(() => ""));
  return draft;
}

export function removeTableDraftRow(draft: TableDraft, rowIndex: number) {
  draft.rows.splice(rowIndex, 1);
  return draft;
}

export function duplicateTableDraftRow(draft: TableDraft, rowIndex: number) {
  const source = draft.rows[rowIndex] || draft.headers.map(() => "");
  draft.rows.splice(rowIndex + 1, 0, padTableRow([...source], draft.headers.length));
  return draft;
}

export function moveTableDraftRow(draft: TableDraft, rowIndex: number, direction: -1 | 1) {
  moveArrayItem(draft.rows, rowIndex, rowIndex + direction);
  return draft;
}

export function addTableDraftColumn(draft: TableDraft) {
  const nextColumn = draft.headers.length + 1;
  draft.headers.push(`Column ${nextColumn}`);
  draft.alignments.push("left");
  draft.formats.push("text");
  for (const row of draft.rows) row.push("");
  return draft;
}

export function removeTableDraftColumn(draft: TableDraft, columnIndex: number) {
  if (draft.headers.length <= 1) return draft;
  draft.headers.splice(columnIndex, 1);
  draft.alignments.splice(columnIndex, 1);
  draft.formats.splice(columnIndex, 1);
  for (const row of draft.rows) row.splice(columnIndex, 1);
  return draft;
}

export function duplicateTableDraftColumn(draft: TableDraft, columnIndex: number) {
  const header = draft.headers[columnIndex] || `Column ${columnIndex + 1}`;
  draft.headers.splice(columnIndex + 1, 0, `${header} copy`);
  draft.alignments.splice(columnIndex + 1, 0, draft.alignments[columnIndex] || "left");
  draft.formats.splice(columnIndex + 1, 0, draft.formats[columnIndex] || "text");
  for (const row of draft.rows) {
    row.splice(columnIndex + 1, 0, row[columnIndex] || "");
  }
  return draft;
}

export function moveTableDraftColumn(draft: TableDraft, columnIndex: number, direction: -1 | 1) {
  const targetIndex = columnIndex + direction;
  moveArrayItem(draft.headers, columnIndex, targetIndex);
  moveArrayItem(draft.alignments, columnIndex, targetIndex);
  moveArrayItem(draft.formats, columnIndex, targetIndex);
  for (const row of draft.rows) moveArrayItem(row, columnIndex, targetIndex);
  return draft;
}

export function appendTableSummaryFormulaRow(draft: TableDraft, formula: TableFormulaFunction, label: string = formula) {
  const dataRowCount = draft.rows.filter((row) => !isTableSummaryRow(row)).length;
  const row = draft.headers.map((_, columnIndex) => {
    if (columnIndex === 0) return label;
    if (!dataRowCount) return "";
    return `=${formula}(${tableColumnRange(columnIndex, dataRowCount)})`;
  });
  draft.rows.push(row);
  return draft;
}

export function buildTableFormulaRow(draft: TableDraft, options: TableFormulaRowOptions) {
  if (!draft.headers.length) return null;
  const firstFormulaColumn = draft.headers.length > 1 ? 1 : 0;
  const targetColumn = clampInteger(options.targetColumn, firstFormulaColumn, draft.headers.length - 1);
  const dataRowCount = Math.max(1, draft.rows.filter((row) => !isTableSummaryRow(row)).length);
  const startRow = clampInteger(options.startRow, 1, dataRowCount);
  const endRow = clampInteger(options.endRow, 1, dataRowCount);
  const [fromRow, toRow] = startRow <= endRow ? [startRow, endRow] : [endRow, startRow];
  const column = spreadsheetColumnName(targetColumn + 1);
  const row = draft.headers.map(() => "");
  const label = options.label?.trim() || options.formula;
  if (targetColumn > 0) row[0] = label;
  row[targetColumn] = `=${options.formula}(${column}${fromRow}:${column}${toRow})`;
  return row;
}

export function validateTableDraft(draft: TableDraft): TableDraftIssue[] {
  const normalized = normalizeTableDraft(draft);
  const issues: TableDraftIssue[] = [];
  if (normalized.id && !/^[A-Za-z][A-Za-z0-9_.:-]*$/.test(normalized.id)) {
    issues.push({
      severity: "error",
      message: "Table id must start with a letter and contain only letters, numbers, dots, colons, underscores, or hyphens.",
    });
  }

  const headerCounts = new Map<string, number>();
  for (const header of normalized.headers) {
    const key = header.toLowerCase();
    headerCounts.set(key, (headerCounts.get(key) || 0) + 1);
  }
  for (const [header, count] of headerCounts) {
    if (count > 1) {
      issues.push({
        severity: "warning",
        message: `Duplicate header "${header}" can make formulas and exports ambiguous.`,
      });
    }
  }

  const dataRowCount = normalized.rows.filter((row) => !isTableSummaryRow(row)).length;
  for (const [rowIndex, row] of normalized.rows.entries()) {
    for (const [columnIndex, cell] of row.entries()) {
      const address = `${spreadsheetColumnName(columnIndex + 1)}${rowIndex + 1}`;
      const span = parseTableCellSpan(cell);
      if (span.colspan > normalized.headers.length - columnIndex) {
        issues.push({
          severity: "error",
          message: `${address} colspan exceeds the available table columns.`,
        });
      }
      if (span.rowspan > normalized.rows.length - rowIndex) {
        issues.push({
          severity: "error",
          message: `${address} rowspan exceeds the available table rows.`,
        });
      }
      issues.push(...validateTableCell(cell, normalized.formats[columnIndex], address, normalized.headers.length, dataRowCount));
    }
  }
  return issues;
}

function validateTableCell(
  value: string,
  format: TableFormat,
  address: string,
  columnCount: number,
  dataRowCount: number,
): TableDraftIssue[] {
  const trimmed = parseTableCellSpan(value).text.trim();
  if (!trimmed) return [];
  if (isFormulaCell(trimmed)) return validateTableFormula(trimmed, address, columnCount, dataRowCount);
  if (format === "date" && Number.isNaN(Date.parse(trimmed))) {
    return [{ severity: "warning", message: `${address} is marked as a date but cannot be parsed.` }];
  }
  if ((format === "number" || format === "currency" || format === "percent") && Number.isNaN(parseCellNumber(trimmed))) {
    return [{ severity: "warning", message: `${address} is marked as ${format} but is not numeric.` }];
  }
  return [];
}

function validateTableFormula(formula: string, address: string, columnCount: number, dataRowCount: number): TableDraftIssue[] {
  const issues: TableDraftIssue[] = [];
  const expression = formula.trim().slice(1).trim();
  if (!/^(SUM|AVG|MIN|MAX|COUNT)\s*\(/i.test(expression)) {
    issues.push({
      severity: "warning",
      message: `${address} uses a formula outside the table editor's supported summary functions.`,
    });
  }
  const references = [...expression.matchAll(/\b([A-Z]+)(\d+)(?::([A-Z]+)(\d+))?\b/gi)];
  if (!references.length) {
    issues.push({
      severity: "warning",
      message: `${address} formula does not reference any table cells.`,
    });
  }
  for (const reference of references) {
    const fromColumn = spreadsheetColumnIndex(reference[1]);
    const fromRow = Number(reference[2]);
    const toColumn = reference[3] ? spreadsheetColumnIndex(reference[3]) : fromColumn;
    const toRow = reference[4] ? Number(reference[4]) : fromRow;
    if (fromColumn > toColumn || fromRow > toRow) {
      issues.push({
        severity: "error",
        message: `${address} formula reference ${reference[0]} has an invalid range order.`,
      });
    } else if (fromColumn < 1 || toColumn > columnCount || fromRow < 1 || toRow > dataRowCount) {
      issues.push({
        severity: "error",
        message: `${address} formula reference ${reference[0]} is outside the editable data range.`,
      });
    }
  }
  return issues;
}

export function serializeMarkdownTable(draft: TableDraft) {
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

export function parseTableCellSpan(value: string): TableCellSpan {
  const trimmed = value.trim();
  const match = trimmed.match(/\s*\{(?=[^{}]*(?:colspan|rowspan)=)([^{}]*)\}\s*$/);
  if (!match || match.index === undefined) {
    return { text: trimmed, colspan: 1, rowspan: 1 };
  }
  return {
    text: trimmed.slice(0, match.index).trim(),
    colspan: spanAttribute(match[1], "colspan"),
    rowspan: spanAttribute(match[1], "rowspan"),
  };
}

export function setTableCellSpan(value: string, colspan: number, rowspan: number) {
  const current = parseTableCellSpan(value);
  const attrs = [
    colspan > 1 ? `colspan=${Math.trunc(colspan)}` : "",
    rowspan > 1 ? `rowspan=${Math.trunc(rowspan)}` : "",
  ].filter(Boolean);
  return attrs.length ? `${current.text} {${attrs.join(" ")}}`.trim() : current.text;
}

export function tableCellSpanPreview(draft: TableDraft, selection: TableCellSpanSelection) {
  const resolved = resolveTableCellSpanSelection(draft, selection);
  const row = draft.rows[resolved.rowIndex];
  const value = row?.[resolved.columnIndex];
  if (value === undefined) return "";
  return setTableCellSpan(value, resolved.colspan, resolved.rowspan);
}

export function applyTableCellSpanToDraft(draft: TableDraft, selection: TableCellSpanSelection) {
  const resolved = resolveTableCellSpanSelection(draft, selection);
  const row = draft.rows[resolved.rowIndex];
  if (!row || row[resolved.columnIndex] === undefined) return draft;
  row[resolved.columnIndex] = setTableCellSpan(row[resolved.columnIndex] || "", resolved.colspan, resolved.rowspan);
  return draft;
}

export function clearTableCellSpanFromDraft(draft: TableDraft, rowIndex: number, columnIndex: number) {
  const row = draft.rows[rowIndex];
  if (!row || row[columnIndex] === undefined) return null;
  const span = parseTableCellSpan(row[columnIndex]);
  row[columnIndex] = span.text;
  return span;
}

function resolveTableCellSpanSelection(draft: TableDraft, selection: TableCellSpanSelection) {
  const rowIndex = clampInteger(selection.rowIndex, 0, Math.max(0, draft.rows.length - 1));
  const columnIndex = clampInteger(selection.columnIndex, 0, Math.max(0, draft.headers.length - 1));
  const colspan = clampInteger(selection.colspan, 1, Math.max(1, draft.headers.length - columnIndex));
  const rowspan = clampInteger(selection.rowspan, 1, Math.max(1, draft.rows.length - rowIndex));
  return { rowIndex, columnIndex, colspan, rowspan };
}

function spanAttribute(attrs: string, name: "colspan" | "rowspan") {
  const match = attrs.match(new RegExp(`(?:^|\\s)${name}=(?:"(\\d+)"|(\\d+))`));
  return Math.max(1, Number(match?.[1] || match?.[2] || 1));
}

export function parseTablePaste(text: string): ParsedTablePaste {
  const source = text.trim();
  const markdownTable = parseMarkdownTablePaste(source);
  if (markdownTable) return markdownTable;
  const rows = parseDelimitedText(source, detectDelimitedPasteDelimiter(source));
  const width = Math.max(0, ...rows.map((row) => row.length));
  return { rows: rows.map((row) => padTableRow(row, width)) };
}

function parseMarkdownTablePaste(text: string) {
  const lines = text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  for (let index = 0; index + 1 < lines.length; index += 1) {
    const header = lines[index];
    const separator = lines[index + 1];
    if (!isMarkdownTableRow(header) || !isMarkdownTableSeparator(separator)) continue;
    const caption = index > 0 ? parseMarkdownTableCaption(lines[index - 1]) : null;
    const rows = [splitMarkdownTableRow(header)];
    const alignments = splitMarkdownTableRow(separator).map(alignmentFromSeparator);
    let nextIndex = index + 2;
    while (nextIndex < lines.length && isMarkdownTableRow(lines[nextIndex])) {
      rows.push(splitMarkdownTableRow(lines[nextIndex]));
      nextIndex += 1;
    }
    const width = Math.max(0, ...rows.map((row) => row.length));
    return {
      rows: rows.map((row) => padTableRow(row, width)),
      alignments: padAlignments(alignments, width),
      id: caption?.id,
      caption: caption?.caption,
    };
  }
  return null;
}

function parseMarkdownTableCaption(line: string) {
  const match = line.match(/^Table:\s*(.*?)\s*$/i);
  if (!match) return null;
  const rawCaption = match[1] || "";
  const idMatch = rawCaption.match(/\s*\{#([^}]+)\}\s*$/);
  const caption = (idMatch ? rawCaption.slice(0, idMatch.index).trim() : rawCaption.trim()) || "Untitled table";
  return {
    id: idMatch?.[1],
    caption,
  };
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

export function inferTableFormat(values: string[]): TableFormat {
  const filled = values.map((value) => value.trim()).filter(Boolean);
  if (!filled.length) return "text";
  if (filled.every((value) => /^\$?-?\d[\d,]*(\.\d+)?$/.test(value))) {
    return filled.some((value) => value.startsWith("$")) ? "currency" : "number";
  }
  if (filled.every((value) => /^-?\d+(\.\d+)?%$/.test(value))) return "percent";
  if (filled.every((value) => !Number.isNaN(Date.parse(value)))) return "date";
  return "text";
}

export function compareTableCells(left: string, right: string, format: TableFormat) {
  if (format === "number" || format === "currency" || format === "percent") {
    return parseCellNumber(left) - parseCellNumber(right);
  }
  if (format === "date") {
    return Date.parse(left) - Date.parse(right);
  }
  return left.localeCompare(right);
}

export function sortTableDraftRows(
  draft: TableDraft,
  columnIndex: number,
  direction: TableSortDirection,
): TableDraft {
  const normalized = normalizeTableDraft(draft);
  const format = normalized.formats[columnIndex] || "text";
  const multiplier = direction === "asc" ? 1 : -1;
  const sortableRows = normalized.rows.filter((row) => !isTableSummaryRow(row));
  const summaryRows = normalized.rows.filter(isTableSummaryRow);
  const rows = [
    ...sortableRows.sort(
      (left, right) => multiplier * compareTableCells(left[columnIndex] || "", right[columnIndex] || "", format),
    ),
    ...summaryRows,
  ];
  return { ...normalized, rows };
}

function moveArrayItem<T>(items: T[], from: number, to: number) {
  if (from === to || from < 0 || to < 0 || from >= items.length || to >= items.length) return;
  const [item] = items.splice(from, 1);
  items.splice(to, 0, item);
}

function formatTableCell(value: string, format: TableFormat) {
  const span = parseTableCellSpan(value);
  const trimmed = span.text.trim();
  if (!trimmed || format === "text") return setTableCellSpan(trimmed, span.colspan, span.rowspan);
  if (isFormulaCell(trimmed)) return setTableCellSpan(trimmed, span.colspan, span.rowspan);
  let formatted = trimmed;
  if (format === "date") {
    const time = Date.parse(trimmed);
    formatted = Number.isNaN(time) ? trimmed : new Date(time).toISOString().slice(0, 10);
    return setTableCellSpan(formatted, span.colspan, span.rowspan);
  }
  const number = parseCellNumber(trimmed);
  if (Number.isNaN(number)) return setTableCellSpan(trimmed, span.colspan, span.rowspan);
  if (format === "currency") formatted = `$${trimFixed(number, 2)}`;
  if (format === "percent") {
    const percent = trimmed.includes("%") || Math.abs(number) > 1 ? number : number * 100;
    formatted = `${trimFixed(percent, 2)}%`;
  }
  if (format === "number") formatted = trimFixed(number, 2);
  return setTableCellSpan(formatted, span.colspan, span.rowspan);
}

export function formatTableTotal(draft: TableDraft, columnIndex: number) {
  const values = numericColumnValues(draft, columnIndex);
  if (!values.length) return "";
  const total = values.reduce((sum, value) => sum + value, 0);
  return formatTableCell(String(total), draft.formats[columnIndex]);
}

export function numericColumnValues(draft: TableDraft, columnIndex: number) {
  return draft.rows
    .map((row) => parseEditableTableNumber(row[columnIndex] || ""))
    .filter((value): value is number => Number.isFinite(value));
}

export function parseEditableTableNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed || trimmed.startsWith("=")) return Number.NaN;
  return parseCellNumber(trimmed);
}

export function isTableSummaryRow(row: string[]) {
  const firstCell = (row[0] || "").trim().toLowerCase();
  if (["total", "subtotal", "grand total"].includes(firstCell)) return true;
  return row.slice(1).some((cell) => cell.trim().startsWith("="));
}

export function isFormulaCell(value = "") {
  return value.trim().startsWith("=");
}

export function tableColumnRange(columnIndex: number, rowCount: number) {
  const column = spreadsheetColumnName(columnIndex + 1);
  return `${column}1:${column}${rowCount}`;
}

export function spreadsheetColumnName(index: number) {
  let value = index;
  let name = "";
  while (value > 0) {
    value -= 1;
    name = String.fromCharCode(65 + (value % 26)) + name;
    value = Math.floor(value / 26);
  }
  return name || "A";
}

function clampInteger(value: number, min: number, max: number) {
  if (!Number.isFinite(value)) return min;
  return Math.min(Math.max(Math.trunc(value), min), max);
}

function spreadsheetColumnIndex(name: string) {
  return name
    .toUpperCase()
    .split("")
    .reduce((value, char) => value * 26 + char.charCodeAt(0) - 64, 0);
}

function parseCellNumber(value: string) {
  return Number(value.replace(/[$,%]/g, ""));
}

function trimFixed(value: number, places: number) {
  return value.toFixed(places).replace(/\.?0+$/, "");
}
