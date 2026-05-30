export interface SupportBundleHandoffReport {
  schema?: string;
  workspace?: string;
  writtenTo?: string;
  privacy?: {
    documentContentIncluded?: boolean;
    secretsIncluded?: boolean;
    note?: string;
  };
  doctor?: {
    status?: string;
    warnings?: string[];
    workspaceScaffold?: {
      status?: string;
      recommended_command?: string | null;
    };
  };
  releaseReadiness?: {
    status?: string;
    releaseReady?: boolean;
    evidenceGaps?: unknown[];
    failures?: unknown[];
  };
  releaseActionPlan?: {
    status?: string;
    manifestPath?: string;
    readyToSendCount?: number;
    workItems?: SupportBundleReleaseWorkItem[];
  };
  specCompletion?: {
    status?: string;
    summary?: {
      openRows?: number;
      totalRows?: number;
      completeRows?: number;
    };
  };
  specActionPlan?: {
    status?: string;
    workOrdersPath?: string;
    readyToSendCount?: number;
    workOrders?: SupportBundleSpecWorkOrder[];
  };
  releaseCandidate?: {
    status?: string;
    releaseable?: boolean;
    sourceCurrent?: boolean;
    candidateDir?: string;
    manifestPath?: string;
    checkReportPath?: string;
    readmePath?: string;
    sha256SumsPath?: string;
    summary?: {
      artifacts?: number;
      evidenceGaps?: number;
      checkStatus?: string;
      checkIssues?: number;
      checkWarnings?: number;
    };
    issues?: string[];
    nextSteps?: string[];
  };
  engineProbe?: {
    status?: string;
    summary?: {
      installed?: number;
      missingLocal?: number;
      incompatible?: number;
      invalidExternalEvidence?: number;
    };
  };
  evidenceReportSummary?: {
    ready?: number;
    attention?: number;
    missing?: number;
    failed?: number;
    total?: number;
  };
  recommendations?: string[];
}

export interface SupportBundleReleaseWorkItem {
  id?: string;
  detail?: string;
  runbooks?: Array<{
    title?: string;
    path?: string;
  }>;
  returns?: string[];
  validatorCommands?: string[];
  ingestCommand?: string;
  finalReadinessCommand?: string;
  readyToSend?: boolean;
}

export interface SupportBundleSpecWorkOrder {
  id?: string;
  readyToSend?: boolean;
  owner?: string;
  specSection?: string;
  requirementArea?: string;
  classification?: string;
  remainingGap?: string;
  runbooks?: string[];
  returns?: string[];
  validatorCommands?: string[];
  ingestCommand?: string;
  matrixClosureCommand?: string;
}

export function supportBundleHandoffMarkdown(report: SupportBundleHandoffReport, generatedAt = new Date().toISOString()) {
  const releaseItems = report.releaseActionPlan?.workItems || [];
  const specOrders = report.specActionPlan?.workOrders || [];
  const evidenceSummary = report.evidenceReportSummary || {};
  const releaseCandidate = report.releaseCandidate;
  const recommendations = report.recommendations || [];
  const privacyNotes = [
    report.privacy?.documentContentIncluded === false ? "Document content is excluded." : "",
    report.privacy?.secretsIncluded === false ? "Stored secrets are excluded." : "",
    report.privacy?.note || "",
  ].filter(Boolean);
  const nextCommands = uniqueStrings([
    ...releaseItems.flatMap((item) => [...(item.validatorCommands || []), item.ingestCommand || "", item.finalReadinessCommand || ""]),
    ...specOrders.flatMap((item) => [...(item.validatorCommands || []), item.ingestCommand || "", item.matrixClosureCommand || ""]),
    ...(releaseCandidate?.nextSteps || []),
    report.doctor?.workspaceScaffold?.recommended_command || "",
  ]).slice(0, 12);

  return [
    "## NEditor Support And Release Handoff",
    "",
    `Generated: ${generatedAt}`,
    `Workspace: ${report.workspace || "current workspace"}`,
    `Support bundle: ${report.writtenTo || "preview only"}`,
    "",
    "### Readiness Summary",
    "",
    "| Area | Status | Detail |",
    "| --- | --- | --- |",
    `| Doctor | ${cell(report.doctor?.status || "unknown")} | ${cell((report.doctor?.warnings || []).join("; ") || "No warnings reported")} |`,
    `| Release readiness | ${cell(report.releaseReadiness?.status || "unknown")} | ${cell(`${report.releaseReadiness?.evidenceGaps?.length || 0} evidence gap(s), ${report.releaseReadiness?.failures?.length || 0} failure(s)`)} |`,
    `| Release action plan | ${cell(report.releaseActionPlan?.status || "unknown")} | ${cell(`${report.releaseActionPlan?.readyToSendCount || 0}/${releaseItems.length} work item(s) ready to send`)} |`,
    `| Spec closure | ${cell(report.specActionPlan?.status || report.specCompletion?.status || "unknown")} | ${cell(`${report.specCompletion?.summary?.openRows || 0} open spec row(s), ${report.specActionPlan?.readyToSendCount || 0}/${specOrders.length} work order(s) ready`)} |`,
    `| Evidence reports | ${cell(evidenceStatus(evidenceSummary))} | ${cell(`${evidenceSummary.ready || 0} ready, ${evidenceSummary.attention || 0} attention, ${evidenceSummary.missing || 0} missing, ${evidenceSummary.failed || 0} failed`)} |`,
    `| Transform engines | ${cell(report.engineProbe?.status || "unknown")} | ${cell(`${report.engineProbe?.summary?.installed || 0} installed, ${report.engineProbe?.summary?.missingLocal || 0} missing, ${report.engineProbe?.summary?.incompatible || 0} incompatible`)} |`,
    `| Release candidate | ${cell(releaseCandidate?.status || "unknown")} | ${cell(releaseCandidate ? `${releaseCandidate.releaseable ? "releaseable" : "not releaseable"}, ${releaseCandidate.summary?.artifacts || 0} artifact(s), ${releaseCandidate.summary?.evidenceGaps || 0} evidence gap(s)` : "No release candidate report attached")} |`,
    "",
    "### Recommended Actions",
    "",
    ...(recommendations.length ? recommendations.map((item) => `- ${item}`) : ["- No support-bundle recommendations were reported."]),
    "",
    "### Release Evidence Assignments",
    "",
    "| Ready | Work item | Detail | Runbook | Evidence to return |",
    "| --- | --- | --- | --- | --- |",
    ...(releaseItems.length
      ? releaseItems.slice(0, 12).map((item) => `| ${item.readyToSend ? "yes" : "no"} | ${cell(item.id || "release-work-item")} | ${cell(item.detail || "Release evidence work item")} | ${cell(runbookText(item.runbooks))} | ${cell((item.returns || []).join("; ") || "Evidence report or artifact path")} |`)
      : ["| - | No release work items | Release action plan is empty or missing | - | - |"]),
    releaseItems.length > 12 ? `\n_${releaseItems.length - 12} additional release work item(s) remain in the JSON support bundle._` : "",
    "",
    "### Specification Closure Work Orders",
    "",
    "| Ready | Owner | Spec area | Classification | Remaining gap |",
    "| --- | --- | --- | --- | --- |",
    ...(specOrders.length
      ? specOrders.slice(0, 12).map((item) => `| ${item.readyToSend ? "yes" : "no"} | ${cell(item.owner || "Release/spec owner")} | ${cell([item.specSection, item.requirementArea].filter(Boolean).join(" / ") || item.id || "spec work order")} | ${cell(item.classification || "evidence")} | ${cell(item.remainingGap || "Attach closure evidence")} |`)
      : ["| - | No spec work orders | Spec action plan is empty or missing | - | - |"]),
    specOrders.length > 12 ? `\n_${specOrders.length - 12} additional spec work order(s) remain in the JSON support bundle._` : "",
    "",
    "### Release Candidate",
    "",
    ...(releaseCandidate
      ? [
          `- Status: ${releaseCandidate.status || "unknown"}`,
          `- Source current: ${releaseCandidate.sourceCurrent ? "yes" : "no"}`,
          `- Releaseable on this host: ${releaseCandidate.releaseable ? "yes" : "no"}`,
          `- Candidate directory: ${releaseCandidate.candidateDir || ".tmp/release-candidate"}`,
          ...(releaseCandidate.issues || []).slice(0, 8).map((item) => `- Issue: ${item}`),
        ]
      : ["- No release candidate report is attached."]),
    "",
    "### Next Commands",
    "",
    ...(nextCommands.length ? nextCommands.map((command) => `- \`${command}\``) : ["- `ned support-bundle --workspace . --output support.json`"]),
    "",
    "### Privacy",
    "",
    ...(privacyNotes.length ? privacyNotes.map((item) => `- ${item}`) : ["- Review the generated support bundle before sharing outside the organization."]),
    "",
  ].filter((line) => line !== undefined).join("\n");
}

function runbookText(runbooks: SupportBundleReleaseWorkItem["runbooks"]) {
  return (runbooks || []).map((runbook) => runbook.path || runbook.title).filter(Boolean).join(", ") || "not mapped";
}

function evidenceStatus(summary: SupportBundleHandoffReport["evidenceReportSummary"]) {
  if (!summary) return "unknown";
  if ((summary.failed || 0) > 0) return "failed";
  if ((summary.attention || 0) > 0 || (summary.missing || 0) > 0) return "needs attention";
  if ((summary.ready || 0) > 0) return "ready";
  return "unknown";
}

function uniqueStrings(values: string[]) {
  return Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)));
}

function cell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}
