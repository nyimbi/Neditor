import type { AccessibilityQaReport } from "./accessibilityQa.js";
import type { ExportVisualQaDashboard } from "./exportVisualQa.js";
import type { ReleaseChecklistItem } from "./releaseReadiness.js";

export type ReleaseEvidenceLane = "complete" | "blocked" | "manual" | "credentialed" | "cross-platform" | "stale" | "ready-to-send";

export interface ReleaseEvidenceDashboardInput {
  releaseChecklist: ReleaseChecklistItem[];
  exportVisualQa?: Pick<ExportVisualQaDashboard, "status" | "summary"> | null;
  accessibilityQa?: Pick<AccessibilityQaReport, "status" | "summary"> | null;
  sourceCount?: number;
  sourceIntegrityIssueCount?: number;
  unresolvedCitationTodoCount?: number;
  diagnosticsErrorCount?: number;
  exportReadinessErrorCount?: number;
  exportReadinessWarningCount?: number;
  gitDirty?: boolean;
  googleAuthenticated?: boolean;
  releaseTarget?: string;
  homebrewEvidenceReady?: boolean;
  signingEvidenceReady?: boolean;
  platformEvidenceReady?: boolean;
}

export interface ReleaseEvidenceDashboardItem {
  id: string;
  lane: ReleaseEvidenceLane;
  label: string;
  detail: string;
  action: string;
}

export interface ReleaseEvidenceDashboard {
  status: "ready" | "needs-work" | "blocked";
  summary: string;
  counts: Record<ReleaseEvidenceLane, number>;
  items: ReleaseEvidenceDashboardItem[];
}

const lanes: ReleaseEvidenceLane[] = ["complete", "blocked", "manual", "credentialed", "cross-platform", "stale", "ready-to-send"];

export function buildReleaseEvidenceDashboard(input: ReleaseEvidenceDashboardInput): ReleaseEvidenceDashboard {
  const releaseMissing = input.releaseChecklist.filter((item) => item.status === "missing").length;
  const releaseReview = input.releaseChecklist.filter((item) => item.status === "needs-review").length;
  const sourceIntegrityIssueCount = input.sourceIntegrityIssueCount || 0;
  const unresolvedCitationTodoCount = input.unresolvedCitationTodoCount || 0;
  const diagnosticsErrorCount = input.diagnosticsErrorCount || 0;
  const exportReadinessErrorCount = input.exportReadinessErrorCount || 0;
  const exportReadinessWarningCount = input.exportReadinessWarningCount || 0;
  const hasReleaseBlockers = releaseMissing > 0 || diagnosticsErrorCount > 0 || exportReadinessErrorCount > 0 || sourceIntegrityIssueCount > 0;
  const hasManualWork =
    releaseReview > 0 ||
    unresolvedCitationTodoCount > 0 ||
    exportReadinessWarningCount > 0 ||
    input.exportVisualQa?.status === "needs-review" ||
    input.accessibilityQa?.status === "needs-review";

  const items: ReleaseEvidenceDashboardItem[] = [
    {
      id: "local-release-metadata",
      lane: hasReleaseBlockers || releaseReview ? (releaseMissing ? "blocked" : "manual") : "complete",
      label: "Local release metadata",
      detail: `${input.releaseChecklist.length - releaseMissing - releaseReview}/${input.releaseChecklist.length} release checks complete.`,
      action: releaseMissing ? "Prepare release metadata and approval fields before external distribution." : releaseReview ? "Resolve review-only release checks before final approval." : "Archive the local release audit with the deliverable.",
    },
    {
      id: "export-readiness-evidence",
      lane: exportReadinessErrorCount ? "blocked" : input.exportVisualQa?.status === "ready" && !exportReadinessWarningCount ? "complete" : "manual",
      label: "Export and visual QA evidence",
      detail: input.exportVisualQa?.summary || "Export visual QA has not been generated.",
      action: exportReadinessErrorCount ? "Fix export readiness errors before writing release artifacts." : "Run export readiness, inspect target output, and insert/export visual QA proof.",
    },
    {
      id: "source-citation-evidence",
      lane: sourceIntegrityIssueCount ? "stale" : unresolvedCitationTodoCount ? "manual" : input.sourceCount ? "complete" : "manual",
      label: "Sources and citation vault",
      detail: `${input.sourceCount || 0} saved source(s), ${sourceIntegrityIssueCount} integrity issue(s), ${unresolvedCitationTodoCount} citation TODO(s).`,
      action: sourceIntegrityIssueCount ? "Re-download or verify modified local source documents." : unresolvedCitationTodoCount ? "Resolve citation TODOs or record review disposition." : "Insert the source library audit when sources support claims.",
    },
    {
      id: "accessibility-evidence",
      lane: input.accessibilityQa?.status === "blocked" ? "blocked" : input.accessibilityQa?.status === "ready" ? "complete" : "manual",
      label: "Accessibility and screen-reader evidence",
      detail: input.accessibilityQa?.summary || "Accessibility QA has not been reviewed in this session.",
      action: input.accessibilityQa?.status === "ready" ? "Archive accessibility QA with release evidence." : "Run Accessibility QA and complete manual assistive-technology sign-off when required.",
    },
    {
      id: "credentialed-integrations",
      lane: input.googleAuthenticated ? "credentialed" : "manual",
      label: "Credentialed integrations",
      detail: input.googleAuthenticated ? "Google session is available for authenticated import/readback workflows." : "Google Docs, AI provider, publishing, or signing credentials are not proven in this session.",
      action: input.googleAuthenticated ? "Run live readback where Google Docs delivery is required." : "Sign in or collect external credentialed evidence before claiming live integrations.",
    },
    {
      id: "cross-platform-packaging",
      lane: input.platformEvidenceReady ? "cross-platform" : "manual",
      label: "Cross-platform packaging evidence",
      detail: input.platformEvidenceReady ? "Platform package evidence is marked ready." : "Windows, Linux, and packaged macOS execution evidence still require external proof.",
      action: "Collect platform package artifacts, run package smoke checks, and ingest the evidence kit before release.",
    },
    {
      id: "homebrew-signing",
      lane: input.homebrewEvidenceReady && input.signingEvidenceReady ? "complete" : "credentialed",
      label: "Homebrew, signing, and notarization",
      detail: input.homebrewEvidenceReady && input.signingEvidenceReady ? "Homebrew and signing evidence are ready." : "Homebrew cask, checksum, signing, and notarization evidence require release artifacts and credentials.",
      action: "Materialize the cask from the signed artifact, verify checksum, run Homebrew audit, and archive signing/notarization proof.",
    },
    {
      id: "working-tree-release-state",
      lane: input.gitDirty ? "stale" : "complete",
      label: "Working tree and evidence freshness",
      detail: input.gitDirty ? "Working tree is dirty; release evidence may be stale." : "Working tree is clean for the current evidence snapshot.",
      action: input.gitDirty ? "Commit or discard unrelated changes, then refresh release evidence." : "Keep generated evidence tied to the current commit.",
    },
  ];

  if (!hasReleaseBlockers && !hasManualWork && !input.gitDirty && input.homebrewEvidenceReady && input.signingEvidenceReady && input.platformEvidenceReady) {
    items.push({
      id: "ready-to-send",
      lane: "ready-to-send",
      label: "Ready to send",
      detail: `Release target ${input.releaseTarget || "external distribution"} has complete local, visual, accessibility, platform, and signing evidence.`,
      action: "Export final artifacts, tag the release, and archive the evidence packet.",
    });
  }

  const counts = lanes.reduce<Record<ReleaseEvidenceLane, number>>((acc, lane) => {
    acc[lane] = items.filter((item) => item.lane === lane).length;
    return acc;
  }, { complete: 0, blocked: 0, manual: 0, credentialed: 0, "cross-platform": 0, stale: 0, "ready-to-send": 0 });
  const status = counts["ready-to-send"] ? "ready" : counts.blocked || counts.stale ? "blocked" : counts.manual || counts.credentialed || counts["cross-platform"] ? "needs-work" : "ready";
  return {
    status,
    counts,
    summary: lanes.map((lane) => `${counts[lane]} ${lane}`).join(" | "),
    items,
  };
}

export function releaseEvidenceDashboardMarkdown(dashboard: ReleaseEvidenceDashboard, generatedAt = new Date().toISOString()) {
  return [
    "## Release Evidence Dashboard",
    "",
    `Status: ${dashboard.status}`,
    `Generated: ${generatedAt}`,
    `Summary: ${dashboard.summary}`,
    "",
    "| Evidence lane | Item | Detail | Action |",
    "| --- | --- | --- | --- |",
    ...dashboard.items.map((item) => `| ${item.lane} | ${escapeTableCell(item.label)} | ${escapeTableCell(item.detail)} | ${escapeTableCell(item.action)} |`),
    "",
  ].join("\n");
}

function escapeTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}
