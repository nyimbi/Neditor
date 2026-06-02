import type { PrintPreviewReport } from "./printPreview.js";

export type ExportVisualQaTarget =
  | "html"
  | "pdf"
  | "docx"
  | "pptx"
  | "markdown-bundle"
  | "blog"
  | "substack"
  | "latex"
  | "google-docs"
  | "epub";

export type ExportVisualQaStatus = "ready" | "needs-review" | "blocked" | "not-run";

export interface ExportVisualQaDiagnosticLike {
  severity: "info" | "warning" | "error" | string;
  message: string;
}

export interface ExportVisualQaManifestLike {
  export_target?: string;
  output_path?: string | null;
  output_hash?: string | null;
  source_hash?: string | null;
  app_version?: string | null;
  included_files?: unknown[];
  media_files?: unknown[];
  transform_artifacts?: unknown[];
  layout_sections?: unknown[];
  readiness?: { ready?: boolean; error_count?: number; warning_count?: number; info_count?: number };
}

export interface ExportVisualQaInput {
  currentTarget: ExportVisualQaTarget;
  manifest?: ExportVisualQaManifestLike | null;
  readiness?: { ready: boolean; error_count: number; warning_count: number; info_count: number } | null;
  diagnostics?: ExportVisualQaDiagnosticLike[];
  printPreview?: PrintPreviewReport;
  outlineCount?: number;
  bibliographyCount?: number;
  citationCount?: number;
  figureCount?: number;
  tableCount?: number;
  equationCount?: number;
  transformCount?: number;
  includeCount?: number;
  metadata?: Record<string, unknown>;
}

export interface ExportVisualQaTargetRow {
  target: ExportVisualQaTarget;
  label: string;
  status: ExportVisualQaStatus;
  checks: string[];
  blockers: string[];
  evidence: string[];
  nextAction: string;
}

export interface ExportVisualQaDashboard {
  status: ExportVisualQaStatus;
  summary: string;
  generatedAt: string;
  rows: ExportVisualQaTargetRow[];
  counts: Record<ExportVisualQaStatus, number>;
  primaryEvidence: string[];
}

const targetLabels: Record<ExportVisualQaTarget, string> = {
  html: "HTML",
  pdf: "PDF",
  docx: "DOCX",
  pptx: "PPTX",
  "markdown-bundle": "Markdown bundle",
  blog: "Blog package",
  substack: "Substack package",
  latex: "LaTeX",
  "google-docs": "Google Docs",
  epub: "EPUB",
};

const targetChecks: Record<ExportVisualQaTarget, string[]> = {
  html: ["Rendered HTML opens", "Styles and syntax highlighting present", "Language and description metadata set", "Images have inspectable paths"],
  pdf: ["Pagination reviewed", "Margins and headers checked", "Wide tables fit", "Page numbers and cover settings verified"],
  docx: ["Heading styles map cleanly", "Tables and captions survive", "Comments/provenance policy checked", "DOCX import target reviewed"],
  pptx: ["Agenda and slide sections checked", "Speaker notes reviewed", "Figures and tables fit slides", "Decision slides obvious"],
  "markdown-bundle": ["Source included", "Includes and assets packaged", "Manifest hashes present", "Transform artifacts packaged"],
  blog: ["Slug, excerpt, tags, and canonical URL checked", "Copy-ready HTML reviewed", "Images and alt text checked", "Publish checklist complete"],
  substack: ["Subject/preview text checked", "Copy-ready HTML and plaintext reviewed", "Links and calls to action checked", "Subscriber context reviewed"],
  latex: ["TeX source generated", "Equations and labels reviewed", "Bibliography and cross references checked", "Local compiler/toolchain result recorded"],
  "google-docs": ["DOCX/HTML handoff generated", "OAuth session ready if importing live", "Readback or manual import proof recorded", "Comments and collaboration state checked"],
  epub: ["Navigation document present", "Metadata, language, and author checked", "Cover/stylesheet reviewed", "Reader table, figure, and equation flow checked"],
};

export function buildExportVisualQaDashboard(input: ExportVisualQaInput, generatedAt = new Date().toISOString()): ExportVisualQaDashboard {
  const diagnostics = input.diagnostics || [];
  const manifestReadiness = input.manifest?.readiness || null;
  const readiness =
    input.readiness ||
    (manifestReadiness
      ? {
          ready: Boolean(manifestReadiness.ready),
          error_count: manifestReadiness.error_count || 0,
          warning_count: manifestReadiness.warning_count || 0,
          info_count: manifestReadiness.info_count || 0,
        }
      : null);
  const rows = (Object.keys(targetLabels) as ExportVisualQaTarget[]).map((target) => buildTargetRow(target, input, diagnostics, readiness));
  const counts = {
    ready: rows.filter((row) => row.status === "ready").length,
    "needs-review": rows.filter((row) => row.status === "needs-review").length,
    blocked: rows.filter((row) => row.status === "blocked").length,
    "not-run": rows.filter((row) => row.status === "not-run").length,
  };
  const status: ExportVisualQaStatus = counts.blocked ? "blocked" : counts["needs-review"] ? "needs-review" : counts.ready ? "ready" : "not-run";
  const primaryEvidence = [
    input.printPreview?.summary ? `Print preview: ${input.printPreview.summary}` : "Print preview not generated.",
    readiness ? `Export readiness: ${readiness.ready ? "ready" : "needs attention"} (${readiness.error_count || 0} errors, ${readiness.warning_count || 0} warnings).` : "Export readiness has not run.",
    input.manifest?.source_hash ? `Source hash: ${input.manifest.source_hash}` : "No export manifest source hash.",
    `${input.outlineCount || 0} headings, ${input.figureCount || 0} figures, ${input.tableCount || 0} tables, ${input.equationCount || 0} equations, ${input.transformCount || 0} transforms.`,
  ];
  return {
    status,
    summary: `${counts.ready} ready | ${counts["needs-review"]} needs review | ${counts.blocked} blocked | ${counts["not-run"]} not run`,
    generatedAt,
    rows,
    counts,
    primaryEvidence,
  };
}

export function exportVisualQaMarkdown(dashboard: ExportVisualQaDashboard) {
  return [
    "## Export Visual QA Dashboard",
    "",
    `Status: ${dashboard.status}`,
    `Generated: ${dashboard.generatedAt}`,
    `Summary: ${dashboard.summary}`,
    "",
    "### Primary Evidence",
    "",
    ...dashboard.primaryEvidence.map((item) => `- ${item}`),
    "",
    "| Target | Status | Next action | Evidence |",
    "| --- | --- | --- | --- |",
    ...dashboard.rows.map((row) => `| ${row.label} | ${row.status} | ${escapeTableCell(row.nextAction)} | ${escapeTableCell(row.evidence.join("; ") || row.blockers.join("; "))} |`),
  ].join("\n");
}

function buildTargetRow(
  target: ExportVisualQaTarget,
  input: ExportVisualQaInput,
  diagnostics: ExportVisualQaDiagnosticLike[],
  readiness: ExportVisualQaInput["readiness"],
): ExportVisualQaTargetRow {
  const isCurrent = input.currentTarget ? input.currentTarget === target : input.manifest?.export_target === target;
  const outputProof = Boolean(isCurrent && (input.manifest?.output_hash || input.manifest?.output_path));
  const readinessRan = Boolean(isCurrent && readiness);
  const errorCount = readinessRan ? (readiness?.error_count ?? 0) : diagnostics.filter((diagnostic) => diagnostic.severity === "error").length;
  const warningCount = readinessRan ? (readiness?.warning_count ?? 0) : diagnostics.filter((diagnostic) => diagnostic.severity === "warning").length;
  const blockers = [
    readinessRan && errorCount ? `${errorCount} readiness error${errorCount === 1 ? "" : "s"}` : "",
    isCurrent && !readinessRan && errorCount ? `${errorCount} compiler error${errorCount === 1 ? "" : "s"}` : "",
    target === "google-docs" && isCurrent && !outputProof ? "No Google Docs handoff package proof yet" : "",
    target === "latex" && isCurrent && !outputProof ? "No LaTeX output path or hash yet" : "",
  ].filter(Boolean);
  const reviewNotes = [
    readinessRan && warningCount ? `${warningCount} readiness warning${warningCount === 1 ? "" : "s"}` : "",
    isCurrent && input.printPreview?.warnings.length && targetNeedsPageReview(target) ? `${input.printPreview.warnings.length} print-flow warning${input.printPreview.warnings.length === 1 ? "" : "s"}` : "",
  ].filter(Boolean);
  const evidence = [
    isCurrent ? "Selected export target." : "",
    readinessRan ? `Readiness ${readiness?.ready ? "ready" : "needs attention"}: ${errorCount} errors, ${warningCount} warnings.` : "",
    outputProof ? `Output proof ${input.manifest?.output_path || input.manifest?.output_hash}.` : "",
    isCurrent && input.manifest?.included_files?.length ? `${input.manifest.included_files.length} included file(s).` : "",
    isCurrent && input.manifest?.media_files?.length ? `${input.manifest.media_files.length} media file(s).` : "",
    isCurrent && input.manifest?.transform_artifacts?.length ? `${input.manifest.transform_artifacts.length} transform artifact(s).` : "",
    isCurrent && targetNeedsPageReview(target) && input.printPreview ? input.printPreview.summary : "",
    ...reviewNotes,
  ].filter(Boolean);
  let status: ExportVisualQaStatus = "not-run";
  if (blockers.length) status = "blocked";
  else if (isCurrent && readinessRan && readiness?.ready && outputProof && !reviewNotes.length) status = "ready";
  else if (isCurrent && readinessRan) status = "needs-review";
  else if (isCurrent) status = "needs-review";
  return {
    target,
    label: targetLabels[target],
    status,
    checks: targetChecks[target],
    blockers,
    evidence,
    nextAction: nextActionForStatus(status, target, outputProof),
  };
}

function targetNeedsPageReview(target: ExportVisualQaTarget) {
  return ["pdf", "docx", "pptx", "latex", "epub"].includes(target);
}

function nextActionForStatus(status: ExportVisualQaStatus, target: ExportVisualQaTarget, outputProof: boolean) {
  if (status === "ready") return "Archive output proof and reviewer sign-off.";
  if (status === "blocked") return "Fix blockers, rerun export readiness, then export again.";
  if (status === "needs-review" && !outputProof) return `Export ${targetLabels[target]} and capture output proof.`;
  if (status === "needs-review") return "Open the exported artifact and record visual review notes.";
  return `Switch to ${targetLabels[target]} when this distribution target is required.`;
}

function escapeTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\n/g, " ").trim() || "-";
}
