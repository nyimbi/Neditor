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
  evidenceReports?: SupportBundleEvidenceReport[];
  evidenceReportSummary?: {
    ready?: number;
    attention?: number;
    missing?: number;
    failed?: number;
    total?: number;
  };
  recommendations?: string[];
}

export interface SupportBundleEvidenceReport {
  id?: string;
  label?: string;
  reportPath?: string;
  status?: string;
  bucket?: string;
  generatedAt?: string;
  error?: string;
  summary?: Record<string, unknown>;
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

export function supportBundleEvidenceReturnPacketMarkdown(report: SupportBundleHandoffReport, generatedAt = new Date().toISOString()) {
  const releaseItems = report.releaseActionPlan?.workItems || [];
  const evidenceReports = report.evidenceReports || [];
  const sourceCommit = report.releaseCandidate?.currentSourceCommit || "<current git commit>";
  const openEvidenceGaps = (report.releaseReadiness?.evidenceGaps || []).map((gap) => evidenceGapId(gap)).filter(Boolean);
  const commands = uniqueStrings([
    ...releaseItems.flatMap((item) => [...(item.validatorCommands || []), item.ingestCommand || "", item.finalReadinessCommand || ""]),
    "pnpm run ingest:evidence -- --source <returned-evidence-dir>",
    "pnpm run check:release-readiness",
    "pnpm run check:spec-completion",
  ]);

  return [
    "## Release Evidence Return Packet",
    "",
    `Generated: ${generatedAt}`,
    `Workspace: ${report.workspace || "current workspace"}`,
    `Source commit: ${sourceCommit}`,
    `Release readiness: ${report.releaseReadiness?.status || "unknown"}`,
    `Evidence gaps: ${openEvidenceGaps.length || report.releaseReadiness?.evidenceGaps?.length || 0}`,
    "",
    "Use this packet to collect release evidence from platform owners, credentialed operators, human reviewers, and external-device testers. Return files under one folder, then ingest them with `pnpm run ingest:evidence -- --source <returned-evidence-dir>`.",
    "",
    "### Redaction Rules",
    "",
    "- Return only validator reports, screenshots needed for release evidence, signed review JSON, package artifacts, cask files, and command-output logs.",
    "- Do not include secrets, customer documents, API keys, OAuth tokens, raw audio, private clipboard contents, or unrelated user files.",
    "- Keep credentialed proof descriptive unless the validator schema explicitly requires a non-secret identifier.",
    "",
    "### Open Evidence Assignments",
    "",
    "| Work item | Owner lane | What to return | Recognized ingest candidates | Validators | Runbooks |",
    "| --- | --- | --- | --- | --- | --- |",
    ...(releaseItems.length
      ? releaseItems.map((item) => {
          const guide = evidenceReturnGuide(item.id || "");
          const returns = item.returns?.length ? item.returns : guide.returns;
          const validators = item.validatorCommands?.length ? item.validatorCommands : guide.validators;
          const runbook = item.runbooks?.length ? runbookText(item.runbooks) : guide.runbook;
          return `| ${cell(item.id || "release-evidence")} | ${cell(guide.owner)} | ${cell(returns.join("; ") || "Evidence report or artifact path")} | ${cell(returnCandidates(returns, guide.candidates).join("; ") || "Use paths listed in the evidence kit")} | ${cell(validators.join("; ") || "pnpm run check:release-readiness")} | ${cell(runbook)} |`;
        })
      : ["| - | No open release work items | Run `pnpm run collect:evidence-kit` then preview the support bundle again | - | `pnpm run check:release-readiness` | release evidence kit |"]),
    "",
    "### Evidence Report Status",
    "",
    "| Report | Bucket | Status | Path | Detail |",
    "| --- | --- | --- | --- | --- |",
    ...(evidenceReports.length
      ? evidenceReports.map((item) => `| ${cell(item.label || item.id || "evidence report")} | ${cell(item.bucket || "unknown")} | ${cell(item.status || "unknown")} | ${cell(item.reportPath || "not reported")} | ${cell(evidenceReportDetail(item))} |`)
      : ["| - | missing | No evidence reports attached to this support bundle | Run `ned support-bundle --workspace . --json` after `pnpm run check:release-readiness` | - |"]),
    "",
    "### Return Folder Layout",
    "",
    "```text",
    "returned-evidence/",
    "  platform-evidence/external/win32/package-artifacts.json",
    "  platform-evidence/external/win32/tauri-webdriver-report.json",
    "  platform-evidence/external/linux/package-artifacts.json",
    "  platform-evidence/external/linux/tauri-webdriver-report.json",
    "  release-signing/external/darwin/signing-evidence.json",
    "  release-signing/external/win32/signing-evidence.json",
    "  release-signing/external/linux/signing-evidence.json",
    "  homebrew/neditor.rb",
    "  homebrew/materialize-cask-report.json",
    "  google-docs-import/external/import-evidence.json",
    "  ai-provider-evidence/external/provider-evidence.json",
    "  ai-runtime-evidence/external/runtime-evidence.json",
    "  security-review/external/security-review.json",
    "  performance-profile/external/native-profile.json",
    "  rendered-export/visual-review-signoff.json",
    "  table-editor/manual-review-signoff.json",
    "  accessibility/manual-review-signoff.json",
    "  manual-review/<work-order-id>/signoff.json",
    "  manual-review/<work-order-id>/artifacts/",
    "```",
    "",
    "### Closure Commands",
    "",
    ...commands.slice(0, 18).map((command) => `- \`${command}\``),
    "",
    "### Acceptance Gate",
    "",
    "- `pnpm run ingest:evidence -- --source <returned-evidence-dir>` copies recognized returns into the local evidence cache.",
    "- `pnpm run check:release-readiness` must report zero failed checks before release packaging is considered complete.",
    "- `pnpm run check:spec-completion` must no longer list manual or credentialed closure gaps for returned work orders.",
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

function evidenceGapId(gap: unknown) {
  if (typeof gap === "string") return gap;
  if (!gap || typeof gap !== "object") return "";
  const value = gap as Record<string, unknown>;
  return String(value.id || value.check || value.name || "").trim();
}

function evidenceReturnGuide(id: string) {
  const normalized = id.toLowerCase();
  if (normalized.includes("windows") || normalized.includes("win32")) {
    return {
      owner: "Windows platform owner",
      returns: [".tmp/platform-evidence/external/win32/package-artifacts.json", ".tmp/platform-evidence/external/win32/tauri-webdriver-report.json"],
      candidates: ["win32/package-artifacts.json", "win32/tauri-webdriver-report.json"],
      validators: ["pnpm run check:platform-evidence"],
      runbook: "runbooks/windows-platform.md",
    };
  }
  if (normalized.includes("linux")) {
    return {
      owner: "Linux platform owner",
      returns: [".tmp/platform-evidence/external/linux/package-artifacts.json", ".tmp/platform-evidence/external/linux/tauri-webdriver-report.json"],
      candidates: ["linux/package-artifacts.json", "linux/tauri-webdriver-report.json"],
      validators: ["pnpm run check:platform-evidence"],
      runbook: "runbooks/linux-platform.md",
    };
  }
  if (normalized.includes("signing") || normalized.includes("notarization")) {
    return {
      owner: "Signing and notarization owner",
      returns: [".tmp/release-signing/external/darwin/signing-evidence.json", ".tmp/release-signing/external/win32/signing-evidence.json", ".tmp/release-signing/external/linux/signing-evidence.json"],
      candidates: ["darwin-signing-evidence.json", "win32-signing-evidence.json", "linux-signing-evidence.json"],
      validators: ["pnpm run check:release-signing"],
      runbook: "runbooks/release-signing.md",
    };
  }
  if (normalized.includes("homebrew")) {
    return {
      owner: "Homebrew release owner",
      returns: [".tmp/homebrew/external/neditor.rb", ".tmp/homebrew/external/materialize-cask-report.json", ".tmp/homebrew/external/neditor-release-artifact"],
      candidates: ["homebrew/neditor.rb", "homebrew/materialize-cask-report.json", "NEditor-macos.dmg"],
      validators: ["pnpm run check:homebrew"],
      runbook: "runbooks/homebrew-release.md",
    };
  }
  if (normalized.includes("google")) {
    return {
      owner: "Google Docs credentialed operator",
      returns: [".tmp/google-docs-import/external/import-evidence.json"],
      candidates: ["google-docs/import-evidence.json", "import-evidence.json"],
      validators: ["pnpm run check:google-docs-import"],
      runbook: "runbooks/google-docs-import.md",
    };
  }
  if (normalized.includes("ai-provider")) {
    return {
      owner: "AI provider operator",
      returns: [".tmp/ai-provider-evidence/external/provider-evidence.json"],
      candidates: ["ai-provider/provider-evidence.json", "provider-evidence.json"],
      validators: ["pnpm run check:ai-provider"],
      runbook: "runbooks/ai-provider-endpoint.md",
    };
  }
  if (normalized.includes("ai-runtime") || normalized.includes("ollama")) {
    return {
      owner: "AI runtime device owner",
      returns: [".tmp/ai-runtime-evidence/external/runtime-evidence.json"],
      candidates: ["ai-runtime/runtime-evidence.json", "runtime-evidence.json"],
      validators: ["pnpm run check:ai-runtime"],
      runbook: "runbooks/ai-runtime-device.md",
    };
  }
  if (normalized.includes("security")) {
    return {
      owner: "Independent security reviewer",
      returns: [".tmp/security-review/external/security-review.json"],
      candidates: ["security-review.json", "security/security-review.json"],
      validators: ["pnpm run check:security-review"],
      runbook: "runbooks/independent-security-review.md",
    };
  }
  if (normalized.includes("performance")) {
    return {
      owner: "Release-device performance tester",
      returns: [".tmp/performance-profile/external/native-profile.json"],
      candidates: ["performance/native-profile.json", "native-profile.json"],
      validators: ["pnpm run check:performance-profile"],
      runbook: "runbooks/release-device-performance-profile.md",
    };
  }
  if (normalized.includes("rendered")) {
    return {
      owner: "Rendered export human reviewer",
      returns: [".tmp/rendered-export-audit/external/visual-review-signoff.json"],
      candidates: ["rendered-export/visual-review-signoff.json", "visual-review-signoff.json"],
      validators: ["pnpm run test:rendered-exports -- --validate-signoff-only"],
      runbook: "runbooks/rendered-export-human-review.md",
    };
  }
  if (normalized.includes("accessibility")) {
    return {
      owner: "Accessibility reviewer",
      returns: [".tmp/accessibility/external/manual-review-signoff.json"],
      candidates: ["accessibility/manual-review-signoff.json", "manual-review-signoff.json"],
      validators: ["pnpm run check:a11y:manual"],
      runbook: "runbooks/accessibility-human-review.md",
    };
  }
  if (normalized.includes("table")) {
    return {
      owner: "Table editor manual reviewer",
      returns: [".tmp/table-editor/external/manual-review-signoff.json"],
      candidates: ["table-editor/manual-review-signoff.json", "table-editor-signoff.json"],
      validators: ["pnpm run check:tables:manual"],
      runbook: "runbooks/table-editor-human-review.md",
    };
  }
  return {
    owner: "Release evidence owner",
    returns: [] as string[],
    candidates: [] as string[],
    validators: [] as string[],
    runbook: "release evidence kit",
  };
}

function returnCandidates(returns: string[], extraCandidates: string[]) {
  return uniqueStrings(
    [
      ...returns,
      ...returns.map((item) => item.replace(/^\.tmp\//, "")),
      ...returns.map((item) => item.split(/[\\/]/).pop() || ""),
      ...extraCandidates,
    ].filter(Boolean),
  );
}

function evidenceReportDetail(report: SupportBundleEvidenceReport) {
  const details = [
    report.generatedAt ? `generated ${report.generatedAt}` : "",
    report.error || "",
    summaryDetail(report.summary),
  ].filter(Boolean);
  return details.join("; ") || "No detail reported";
}

function summaryDetail(summary: SupportBundleEvidenceReport["summary"]) {
  if (!summary) return "";
  return Object.entries(summary)
    .filter(([, value]) => typeof value === "string" || typeof value === "number" || typeof value === "boolean")
    .slice(0, 4)
    .map(([key, value]) => `${key}: ${String(value)}`)
    .join(", ");
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
