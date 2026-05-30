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
    currentSourceCommit?: string;
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

export function supportBundleManualReviewKitMarkdown(report: SupportBundleHandoffReport, generatedAt = new Date().toISOString()) {
  const manualOrders = (report.specActionPlan?.workOrders || []).filter((order) => order.classification === "manual-review");
  const sourceCommit = report.releaseCandidate?.currentSourceCommit || "<current git commit>";
  const appVersion = "<package.json version>";
  return [
    "## Manual Review Sign-Off Kit",
    "",
    `Generated: ${generatedAt}`,
    `Workspace: ${report.workspace || "current workspace"}`,
    `Manual review work orders: ${manualOrders.length}`,
    "",
    "Use this kit with `pnpm run check:manual-review`. Complete one sign-off JSON file per work order, keep artifacts beside the sign-off, then ingest the returned evidence with `pnpm run ingest:evidence -- --source <returned-evidence-dir>`.",
    "",
    "### Assignment Index",
    "",
    "| Work order | Spec area | Reviewer owner | Return path | Validators |",
    "| --- | --- | --- | --- | --- |",
    ...(manualOrders.length
      ? manualOrders.map((order) => `| ${cell(order.id || "manual-review")} | ${cell([order.specSection, order.requirementArea].filter(Boolean).join(" / ") || "Manual review")} | ${cell(order.owner || "Named manual reviewer")} | ${cell((order.returns || [`.tmp/manual-review/${order.id || "<work-order-id>"}/signoff.json`]).join("; "))} | ${cell((order.validatorCommands || []).join("; ") || "pnpm run check:manual-review")} |`)
      : ["| - | No manual-review work orders | - | - | - |"]),
    "",
    "### Reviewer Instructions",
    "",
    "- Confirm `git status --short` is clean before signing.",
    "- Run the validator commands listed for the assigned work order.",
    "- Capture screenshots, exported files, native-viewer proof, or screen-reader notes under the returned `artifacts/` folder.",
    "- Mark every checklist item as `pass` or `exception`; exceptions need a non-release-blocking rationale.",
    "- Keep `unresolvedBlockers` empty before returning evidence.",
    "- Do not include secrets, customer documents, API keys, raw audio, or private clipboard contents.",
    "",
    "### Sign-Off Templates",
    "",
    ...(manualOrders.length
      ? manualOrders.flatMap((order) => manualReviewTemplateSection(order, sourceCommit, appVersion))
      : ["No manual-review sign-off templates are required for the current support bundle.", ""]),
    "### Closure Commands",
    "",
    "- `pnpm run check:manual-review`",
    "- `pnpm run ingest:evidence -- --source <returned-evidence-dir>`",
    "- `pnpm run check:release-readiness`",
    "- `pnpm run check:spec-completion`",
    "",
  ].join("\n");
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

function manualReviewTemplateSection(order: SupportBundleSpecWorkOrder, sourceCommit: string, appVersion: string) {
  const workOrderId = order.id || "manual-review-work-order";
  const checklist = [
    {
      id: "workflow-observed",
      label: `Exercise and observe: ${[order.specSection, order.requirementArea].filter(Boolean).join(" / ") || workOrderId}`,
      status: "pass",
      evidence: "artifacts/screenshot-or-export-proof.png",
      notes: "",
    },
    {
      id: "artifact-evidence",
      label: "Attach screenshots, exported files, native-viewer proof, or screen-reader notes as appropriate.",
      status: "pass",
      evidence: "artifacts/screenshot-or-export-proof.png",
      notes: "",
    },
    {
      id: "no-release-blockers",
      label: "Confirm no unresolved release blocker remains for this work order.",
      status: "pass",
      evidence: "artifacts/validator-output.txt",
      notes: "",
    },
    ...(order.validatorCommands || []).map((command, index) => ({
      id: `validator-${String(index + 1).padStart(2, "0")}`,
      label: `Validator command passed: ${command}`,
      status: "pass",
      evidence: "artifacts/validator-output.txt",
      notes: "",
    })),
  ];
  const template = {
    schema: "neditor.manual-review.signoff.v1",
    workOrderId,
    requirement: [order.specSection, order.requirementArea].filter(Boolean).join(" / ") || workOrderId,
    appVersion,
    sourceCommit,
    sourceTreeClean: true,
    reviewer: {
      name: "",
      role: "",
      organization: "",
    },
    reviewedAt: generatedIsoPlaceholder(),
    platform: {
      os: "",
      arch: "",
      version: "",
      device: "",
    },
    appBuild: {
      kind: "packaged-release-or-current-local-app",
      path: "",
      hash: "",
    },
    artifacts: ["artifacts/screenshot-or-export-proof.png", "artifacts/validator-output.txt"],
    checklist,
    unresolvedBlockers: [],
    notes: order.remainingGap || "",
  };
  return [
    `#### ${workOrderId}`,
    "",
    `Owner: ${order.owner || "Named manual reviewer"}`,
    `Spec area: ${[order.specSection, order.requirementArea].filter(Boolean).join(" / ") || "Manual review"}`,
    `Return path: ${(order.returns || [`.tmp/manual-review/${workOrderId}/signoff.json`]).join(", ")}`,
    "",
    "```json",
    JSON.stringify(template, null, 2),
    "```",
    "",
  ];
}

function generatedIsoPlaceholder() {
  return "YYYY-MM-DDTHH:mm:ss.sssZ";
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
