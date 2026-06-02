import type { FrontMatterDataSourceRow } from "./frontMatterManagers.js";

export type DataRefreshRowStatus = "current" | "needs-refresh" | "stale-audit" | "blocked" | "missing-compile";

export interface DataRefreshDiagnosticLike {
  severity?: string;
  message: string;
  source_file?: string | null;
}

export interface DataRefreshIncludeEdgeLike {
  child: string;
}

export interface DataRefreshAuditEntry {
  refreshedAt: string;
  signature: string;
}

export interface DataRefreshPlanRow {
  id: string;
  source: FrontMatterDataSourceRow;
  status: DataRefreshRowStatus;
  label: string;
  verification: string;
  evidence: string[];
  compiled: boolean;
  importableTable: boolean;
  audit?: DataRefreshAuditEntry;
}

export interface DataRefreshPlan {
  signature: string;
  generatedAt: string;
  rows: DataRefreshPlanRow[];
  summary: {
    total: number;
    current: number;
    needsRefresh: number;
    staleAudit: number;
    blocked: number;
    missingCompile: number;
    importableTables: number;
  };
}

export interface BuildDataRefreshPlanInput {
  sources: FrontMatterDataSourceRow[];
  includeGraph?: DataRefreshIncludeEdgeLike[];
  diagnostics?: DataRefreshDiagnosticLike[];
  documentText?: string;
  generatedAt?: string;
}

export function buildDataRefreshPlan(input: BuildDataRefreshPlanInput): DataRefreshPlan {
  const includeGraph = input.includeGraph || [];
  const diagnostics = input.diagnostics || [];
  const audits = extractDataRefreshAudits(input.documentText || "");
  const signature = dataRefreshSignature(input.sources, includeGraph, diagnostics);
  const generatedAt = input.generatedAt || new Date().toISOString();
  const rows = input.sources.map((source) => {
    const compiled = dataSourceCompiled(source, includeGraph);
    const sourceDiagnostics = diagnosticsForSource(source, diagnostics);
    const audit = audits.get(source.path) || audits.get("*");
    const importableTable = dataSourceCanImportAsEditableTable(source);
    const status = dataRefreshRowStatus(source, compiled, sourceDiagnostics, audit, signature, includeGraph.length);
    return {
      id: source.id,
      source,
      status,
      label: dataRefreshStatusLabel(status),
      verification: dataRefreshVerification(source, status, compiled, audit),
      evidence: dataRefreshEvidence(source, compiled, sourceDiagnostics, audit),
      compiled,
      importableTable,
      ...(audit ? { audit } : {}),
    };
  });
  return {
    signature,
    generatedAt,
    rows,
    summary: {
      total: rows.length,
      current: rows.filter((row) => row.status === "current").length,
      needsRefresh: rows.filter((row) => row.status === "needs-refresh").length,
      staleAudit: rows.filter((row) => row.status === "stale-audit").length,
      blocked: rows.filter((row) => row.status === "blocked").length,
      missingCompile: rows.filter((row) => row.status === "missing-compile").length,
      importableTables: rows.filter((row) => row.importableTable).length,
    },
  };
}

export function dataRefreshSummary(plan: DataRefreshPlan) {
  const pending = plan.summary.needsRefresh + plan.summary.staleAudit + plan.summary.missingCompile;
  return `${plan.summary.total} data source${plan.summary.total === 1 ? "" : "s"} | ${plan.summary.current} current | ${pending} need refresh | ${plan.summary.blocked} blocked | ${plan.summary.importableTables} table-import ready`;
}

export function dataRefreshAuditMarkdown(plan: DataRefreshPlan, refreshedAt = plan.generatedAt) {
  const rows = plan.rows.length
    ? plan.rows.map((row) => `| ${escapeTableCell(row.source.name)} | ${escapeTableCell(row.source.kind)} | ${escapeTableCell(row.label)} | ${escapeTableCell(row.verification)} | ${escapeTableCell(row.source.path)} |`)
    : ["| No data sources declared | - | Missing | Add front matter dataSources, csvFiles, or xlsxFiles before refreshing. | - |"];
  return [
    "## Data Refresh Audit",
    "",
    `<!-- neditor:data-refresh refreshedAt="${escapeAttribute(refreshedAt)}" signature="${escapeAttribute(plan.signature)}" -->`,
    "",
    `Refreshed: ${refreshedAt}`,
    "",
    "| Source | Type | Status | Verification | Path |",
    "| --- | --- | --- | --- | --- |",
    ...rows,
    "",
    "### Refresh Notes",
    "",
    "- Recompile preview after source files change.",
    "- Import CSV, TSV, or XLSX rows as editable Markdown tables before hand-editing table data.",
    "- Treat blocked paths, unsupported types, and missing files as export blockers.",
  ].join("\n");
}

export function extractDataRefreshAudits(text: string): Map<string, DataRefreshAuditEntry> {
  const audits = new Map<string, DataRefreshAuditEntry>();
  for (const match of text.matchAll(/<!--\s*neditor:data-refresh\s+([^>]*)-->/g)) {
    const attrs = parseHtmlCommentAttributes(match[1] || "");
    const refreshedAt = attrs.get("refreshedAt") || attrs.get("refreshedat") || "";
    const signature = attrs.get("signature") || "";
    if (!signature) continue;
    audits.set("*", { refreshedAt, signature });
  }
  for (const match of text.matchAll(/<!--\s*neditor:data-source-refresh\s+([^>]*)-->/g)) {
    const attrs = parseHtmlCommentAttributes(match[1] || "");
    const path = attrs.get("path") || "";
    const refreshedAt = attrs.get("refreshedAt") || attrs.get("refreshedat") || "";
    const signature = attrs.get("signature") || "";
    if (path && signature) audits.set(path, { refreshedAt, signature });
  }
  return audits;
}

export function dataSourceCanImportAsEditableTable(source: Pick<FrontMatterDataSourceRow, "kind" | "status">) {
  return source.status === "ready" && ["csv", "tsv", "xlsx"].includes(String(source.kind).toLowerCase());
}

function dataRefreshRowStatus(
  source: FrontMatterDataSourceRow,
  compiled: boolean,
  diagnostics: DataRefreshDiagnosticLike[],
  audit: DataRefreshAuditEntry | undefined,
  signature: string,
  includeGraphSize: number,
): DataRefreshRowStatus {
  if (source.status !== "ready" || diagnostics.some((diagnostic) => diagnostic.severity === "error")) return "blocked";
  if (!includeGraphSize) return "missing-compile";
  if (!compiled) return "needs-refresh";
  if (!audit) return "needs-refresh";
  if (audit.signature !== signature) return "stale-audit";
  return "current";
}

function dataRefreshStatusLabel(status: DataRefreshRowStatus) {
  const labels: Record<DataRefreshRowStatus, string> = {
    current: "Current",
    "needs-refresh": "Needs refresh",
    "stale-audit": "Stale audit",
    blocked: "Blocked",
    "missing-compile": "Compile needed",
  };
  return labels[status];
}

function dataRefreshVerification(
  source: FrontMatterDataSourceRow,
  status: DataRefreshRowStatus,
  compiled: boolean,
  audit?: DataRefreshAuditEntry,
) {
  if (source.status === "blocked-path") return "Move the file under the document folder or workspace before compiling.";
  if (source.status === "unsupported-type") return "Change the source type to CSV, TSV, JSON, YAML, or XLSX.";
  if (source.status === "missing-path") return "Add a local source path before refresh.";
  if (status === "missing-compile") return "Refresh the preview so the compiler imports front matter data sources.";
  if (!compiled) return "Refresh preview; the current compile output does not include this source path.";
  if (status === "stale-audit") return "Insert a new audit after source rows, diagnostics, or compile evidence change.";
  if (audit?.refreshedAt) return `Audit recorded at ${audit.refreshedAt}.`;
  return "Insert a data refresh audit after verifying imported rows and dependent narrative.";
}

function dataRefreshEvidence(
  source: FrontMatterDataSourceRow,
  compiled: boolean,
  diagnostics: DataRefreshDiagnosticLike[],
  audit?: DataRefreshAuditEntry,
) {
  return [
    source.detail,
    compiled ? "Included in current compile graph." : "Not found in current compile graph.",
    audit?.refreshedAt ? `Last audit: ${audit.refreshedAt}` : "No audit marker recorded.",
    ...diagnostics.slice(0, 3).map((diagnostic) => diagnostic.message),
  ].filter(Boolean);
}

function dataRefreshSignature(
  sources: FrontMatterDataSourceRow[],
  includeGraph: DataRefreshIncludeEdgeLike[],
  diagnostics: DataRefreshDiagnosticLike[],
) {
  const payload = JSON.stringify({
    sources: sources.map((source) => ({
      path: source.path,
      kind: source.kind,
      status: source.status,
      sheetName: source.sheetName || "",
      sheetIndex: source.sheetIndex || 0,
      compiled: dataSourceCompiled(source, includeGraph),
    })),
    diagnostics: diagnostics
      .filter((diagnostic) => sources.some((source) => diagnosticMentionsSource(diagnostic, source)))
      .map((diagnostic) => ({ severity: diagnostic.severity || "", message: diagnostic.message })),
  });
  return `dsr-${stableHash(payload)}`;
}

function dataSourceCompiled(source: FrontMatterDataSourceRow, includeGraph: DataRefreshIncludeEdgeLike[]) {
  return includeGraph.some((edge) => pathEndsWith(normalizePath(edge.child), normalizePath(source.path)));
}

function diagnosticsForSource(source: FrontMatterDataSourceRow, diagnostics: DataRefreshDiagnosticLike[]) {
  return diagnostics.filter((diagnostic) => diagnosticMentionsSource(diagnostic, source));
}

function diagnosticMentionsSource(diagnostic: DataRefreshDiagnosticLike, source: FrontMatterDataSourceRow) {
  const haystack = `${diagnostic.source_file || ""}\n${diagnostic.message}`.toLowerCase();
  return Boolean(source.path && haystack.includes(source.path.toLowerCase())) || Boolean(source.name && haystack.includes(source.name.toLowerCase()));
}

function pathEndsWith(fullPath: string, suffix: string) {
  if (!suffix) return false;
  return fullPath === suffix || fullPath.endsWith(`/${suffix}`);
}

function normalizePath(path: string) {
  return path.replace(/\\/g, "/").replace(/\/+/g, "/").replace(/^\.\//, "");
}

function parseHtmlCommentAttributes(text: string) {
  const attrs = new Map<string, string>();
  for (const match of text.matchAll(/([A-Za-z0-9_-]+)="([^"]*)"/g)) {
    attrs.set(match[1], match[2]);
  }
  return attrs;
}

function escapeTableCell(value: unknown) {
  return String(value ?? "").replace(/\|/g, "\\|").replace(/\n/g, " ").trim() || "-";
}

function escapeAttribute(value: string) {
  return value.replace(/&/g, "&amp;").replace(/"/g, "&quot;");
}

function stableHash(value: string) {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(36);
}
