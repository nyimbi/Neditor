import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const matrixPath = join(root, "docs", "spec-completion-matrix.md");
const outputDir = join(root, ".tmp", "spec-completion");
const reportPath = join(outputDir, "report.json");
const allowedStatuses = new Set(["Complete", "Partial", "Unverified", "Missing", "Deferred"]);

const markdown = readFileSync(matrixPath, "utf8");
const rows = parseMatrixRows(markdown);
let issues = validateMatrix(markdown, rows);
const summary = summarizeRows(rows);
const openRows = rows.filter((row) => ["Partial", "Unverified", "Missing"].includes(row.status));
const openRowPlans = openRows.map((row) => ({
  specSection: row.specSection,
  requirementArea: row.requirementArea,
  status: row.status,
  classification: classifyOpenRow(row),
  nextAction: nextActionForOpenRow(row),
  remainingGap: row.remainingGap,
}));
const gapTriage = summarizeGapTriage(openRowPlans);
const gapPlanPath = join(outputDir, "gap-plan.md");
const workOrdersPath = join(outputDir, "work-orders.json");
const workOrdersMarkdownPath = join(outputDir, "work-orders.md");
const workOrders = openRowPlans.map((row, index) => buildWorkOrder(row, index));
issues = [...issues, ...validateWorkOrderRunbooks(workOrders)];
const status = issues.length ? "failed" : summary.openRows > 0 ? "partial-with-release-risks" : "complete";

mkdirSync(outputDir, { recursive: true });
writeFileSync(
  reportPath,
  `${JSON.stringify(
    {
      schema: "neditor.spec-completion-report.v1",
      generatedAt: new Date().toISOString(),
      status,
      matrixPath: relative(matrixPath),
      gapPlanPath: relative(gapPlanPath),
      workOrdersPath: relative(workOrdersPath),
      workOrdersMarkdownPath: relative(workOrdersMarkdownPath),
      summary,
      gapTriage,
      issues,
      openRows: openRowPlans,
      workOrders,
    },
    null,
    2,
  )}\n`,
);
writeFileSync(gapPlanPath, renderGapPlanMarkdown(openRowPlans, gapTriage, summary, status));
writeFileSync(
  workOrdersPath,
  `${JSON.stringify(
    {
      schema: "neditor.spec-completion-work-orders.v1",
      generatedAt: new Date().toISOString(),
      status,
      matrixPath: relative(matrixPath),
      reportPath: relative(reportPath),
      summary: {
        total: workOrders.length,
        readyToSend: workOrders.filter((order) => order.readyToSend).length,
        byClassification: gapTriage.byClassification,
      },
      workOrders,
    },
    null,
    2,
  )}\n`,
);
writeFileSync(workOrdersMarkdownPath, renderWorkOrdersMarkdown(workOrders, summary, status));

if (issues.length) {
  console.error("Spec completion matrix validation failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  console.error(`Wrote ${relative(reportPath)}.`);
  process.exit(1);
}

console.log(
  `Spec completion matrix is ${status}; wrote ${relative(reportPath)}, ${relative(gapPlanPath)}, ${relative(workOrdersPath)}, and ${relative(workOrdersMarkdownPath)}.`,
);

function parseMatrixRows(text) {
  const parsed = [];
  let section = "";
  const lines = text.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const heading = line.match(/^##\s+(.+)$/);
    if (heading) section = heading[1].trim();
    if (!line.startsWith("|")) continue;
    if (/^\|\s*-+/.test(line)) continue;
    if (line.includes("Requirement area") && line.includes("Current status")) continue;

    const cells = splitTableRow(line);
    if (cells.length !== 5) continue;
    parsed.push({
      section,
      line: index + 1,
      specSection: cleanCell(cells[0]),
      requirementArea: cleanCell(cells[1]),
      status: cleanCell(cells[2]),
      evidence: cleanCell(cells[3]),
      remainingGap: cleanCell(cells[4]),
    });
  }
  return parsed;
}

function validateMatrix(text, rows) {
  const foundIssues = [];
  if (!/^# NEditor Specification Completion Matrix/m.test(text)) {
    foundIssues.push("matrix title is missing");
  }
  if (!/^Updated:\s+\d{4}-\d{2}-\d{2}$/m.test(text)) {
    foundIssues.push("Updated date must use YYYY-MM-DD");
  }
  if (!text.includes("Status vocabulary:")) {
    foundIssues.push("status vocabulary section is missing");
  }
  if (!text.includes("Current major verification gaps:")) {
    foundIssues.push("Current major verification gaps section is missing");
  }
  if (!text.includes("## Next Matrix Work")) {
    foundIssues.push("Next Matrix Work section is missing");
  }
  if (rows.length < 45) {
    foundIssues.push(`matrix has too few requirement rows: ${rows.length}`);
  }

  const duplicateKeys = new Set();
  const seenKeys = new Set();
  for (const row of rows) {
    const key = `${row.specSection}::${row.requirementArea}`;
    if (seenKeys.has(key)) duplicateKeys.add(key);
    seenKeys.add(key);
    if (!row.specSection) foundIssues.push(`line ${row.line}: spec section is empty`);
    if (!row.requirementArea) foundIssues.push(`line ${row.line}: requirement area is empty`);
    if (!allowedStatuses.has(row.status)) foundIssues.push(`line ${row.line}: invalid status ${JSON.stringify(row.status)}`);
    if (!row.evidence || isPlaceholder(row.evidence)) foundIssues.push(`line ${row.line}: evidence is empty or placeholder`);
    if (!row.remainingGap || isPlaceholder(row.remainingGap)) foundIssues.push(`line ${row.line}: remaining gap is empty or placeholder`);
    if (["Partial", "Unverified", "Missing"].includes(row.status) && !hasSubstantiveGap(row.remainingGap)) {
      foundIssues.push(`line ${row.line}: ${row.status} row must name a substantive remaining gap`);
    }
    if (row.status === "Complete" && /\b(missing|unverified|not implemented|todo|tbd)\b/i.test(row.evidence)) {
      foundIssues.push(`line ${row.line}: Complete row evidence contains unresolved language`);
    }
  }

  for (const duplicate of duplicateKeys) {
    foundIssues.push(`duplicate requirement row: ${duplicate}`);
  }

  return foundIssues;
}

function validateWorkOrderRunbooks(workOrders) {
  const foundIssues = [];
  const runbooks = new Set(workOrders.flatMap((order) => order.runbooks || []));
  for (const runbook of runbooks) {
    const path = join(root, runbook);
    if (!existsSync(path)) {
      foundIssues.push(`work order runbook is missing: ${runbook}`);
    }
  }
  return foundIssues;
}

function summarizeRows(rows) {
  const byStatus = Object.fromEntries([...allowedStatuses].map((status) => [status, 0]));
  for (const row of rows) byStatus[row.status] = (byStatus[row.status] || 0) + 1;
  const openRows = Number(byStatus.Partial || 0) + Number(byStatus.Unverified || 0) + Number(byStatus.Missing || 0);
  return {
    totalRows: rows.length,
    completeRows: Number(byStatus.Complete || 0),
    partialRows: Number(byStatus.Partial || 0),
    unverifiedRows: Number(byStatus.Unverified || 0),
    missingRows: Number(byStatus.Missing || 0),
    deferredRows: Number(byStatus.Deferred || 0),
    openRows,
    sections: Array.from(new Set(rows.map((row) => row.section))).filter(Boolean),
  };
}

function classifyOpenRow(row) {
  const remainingGap = row.remainingGap.toLowerCase();
  const evidence = row.evidence.toLowerCase();
  const classification = classifyGapText(remainingGap);
  if (classification) return classification;
  const fallbackClassification = classifyGapText(`${remainingGap} ${evidence}`);
  if (fallbackClassification) return fallbackClassification;
  return "needs-triage";
}

function classifyGapText(text) {
  if (/\b(signing|notarization|notarized|credential|certificate|attestation)\b/.test(text)) return "release-credentials";
  if (/\b(homebrew|cask|sha256|artifact proof)\b/.test(text)) return "distribution-artifacts";
  if (/\b(human|manual|screen-reader|assistive|native viewer|visual qa|review sign-off|sign-off|signoff)\b/.test(text)) {
    return "manual-review";
  }
  if (
    /\b(live provider|live audible|actual model download|model download|real device|native runtime|google docs live|authorized drive|external evidence|independent security|release-device|credentialed)\b/.test(
      text,
    )
  ) {
    return "external-evidence";
  }
  if (/\b(windows|linux|cross-platform|supported host|supported-host|other os|platform evidence|package artifact)\b/.test(text)) {
    return "cross-platform-evidence";
  }
  if (/\b(document|sync|matrix|todo|progress|docs|runbook|guide)\b/.test(text)) return "documentation-proof";
  if (/\b(test|workflow|coverage|proof|verify|native proof|browser proof|evidence)\b/.test(text)) return "local-proof";
  if (/\b(modular|split|refactor|implementation|implement|deeper|broader|edge case)\b/.test(text)) return "local-implementation";
  return "";
}

function nextActionForOpenRow(row) {
  switch (classifyOpenRow(row)) {
    case "release-credentials":
      return "Collect credentialed signing/notarization evidence on a clean release host and ingest it into the release evidence kit.";
    case "distribution-artifacts":
      return "Build the final distributable artifact, pin its SHA-256 in the release/Homebrew evidence, and rerun readiness gates.";
    case "cross-platform-evidence":
      return "Run the packaged workflow or WebDriver proof on the named supported host and copy the validator-shaped report into the evidence directory.";
    case "manual-review":
      return "Complete the generated reviewer checklist with named reviewer, platform, evidence references, passing checklist items, and zero unresolved blockers.";
    case "external-evidence":
      return "Collect validator-shaped external proof with current app version, current Git commit, clean source tree, artifact hashes, and no stored secrets.";
    case "documentation-proof":
      return "Replace broad prose with exact command, artifact, and source references, then rerun the docs and spec completion checks.";
    case "local-proof":
      return "Add or refresh direct local tests, browser workflows, native smoke assertions, or artifact inspection that proves this row end to end.";
    case "local-implementation":
      return "Implement the missing behavior in the smallest owned module, then add direct tests and update the matrix evidence.";
    default:
      return "Classify the gap, identify the authoritative proof required, and add an exact command or artifact reference before claiming closure.";
  }
}

function summarizeGapTriage(openRowPlans) {
  const categories = [
    "local-implementation",
    "local-proof",
    "documentation-proof",
    "manual-review",
    "external-evidence",
    "cross-platform-evidence",
    "release-credentials",
    "distribution-artifacts",
    "needs-triage",
  ];
  const byClassification = Object.fromEntries(categories.map((category) => [category, 0]));
  for (const row of openRowPlans) {
    byClassification[row.classification] = (byClassification[row.classification] || 0) + 1;
  }
  const locallyClosableRows =
    Number(byClassification["local-implementation"] || 0) +
    Number(byClassification["local-proof"] || 0) +
    Number(byClassification["documentation-proof"] || 0) +
    Number(byClassification["needs-triage"] || 0);
  const evidenceBlockedRows = openRowPlans.length - locallyClosableRows;
  return {
    byClassification,
    locallyClosableRows,
    evidenceBlockedRows,
    firstLocalActions: openRowPlans
      .filter((row) => ["local-implementation", "local-proof", "documentation-proof", "needs-triage"].includes(row.classification))
      .slice(0, 12)
      .map((row) => ({
        specSection: row.specSection,
        requirementArea: row.requirementArea,
        classification: row.classification,
        nextAction: row.nextAction,
      })),
  };
}

function buildWorkOrder(row, index) {
  const id = `${String(index + 1).padStart(3, "0")}-${slugify(row.classification)}-${slugify(row.requirementArea).slice(0, 48)}`;
  const owner = ownerForClassification(row.classification);
  const runbooks = runbooksForClassification(row.classification);
  const validatorCommands = validatorCommandsForClassification(row.classification);
  const returns = returnedEvidenceForClassification(row.classification);
  return {
    id,
    readyToSend: true,
    owner,
    specSection: row.specSection,
    requirementArea: row.requirementArea,
    status: row.status,
    classification: row.classification,
    remainingGap: row.remainingGap,
    objective: objectiveForWorkOrder(row),
    acceptanceCriteria: acceptanceCriteriaForClassification(row.classification),
    runbooks,
    returns,
    validatorCommands,
    ingestCommand: "pnpm run ingest:evidence -- --source <returned-evidence-dir>",
    finalReadinessCommand: "pnpm run check:release-readiness",
    matrixClosureCommand: "pnpm run check:spec-completion",
    notes: notesForClassification(row.classification),
  };
}

function ownerForClassification(classification) {
  switch (classification) {
    case "manual-review":
      return "Named manual reviewer";
    case "external-evidence":
      return "Credentialed evidence collector";
    case "cross-platform-evidence":
      return "Supported-host QA owner";
    case "release-credentials":
      return "Release engineering owner";
    case "distribution-artifacts":
      return "Distribution packaging owner";
    case "local-implementation":
      return "NEditor implementation owner";
    case "local-proof":
      return "NEditor verification owner";
    case "documentation-proof":
      return "Documentation and evidence owner";
    default:
      return "Release readiness owner";
  }
}

function runbooksForClassification(classification) {
  switch (classification) {
    case "manual-review":
      return ["runbooks/manual-review.md", "runbooks/rendered-export-review.md"];
    case "external-evidence":
      return ["runbooks/external-evidence.md", "runbooks/performance-profile.md"];
    case "cross-platform-evidence":
      return ["runbooks/platform-evidence.md", "runbooks/external-engine-evidence.md"];
    case "release-credentials":
      return ["runbooks/release-signing.md", "runbooks/platform-evidence.md"];
    case "distribution-artifacts":
      return ["runbooks/homebrew-release.md", "runbooks/release-signing.md"];
    case "local-implementation":
      return ["runbooks/spec-completion-closure.md"];
    case "local-proof":
      return ["runbooks/spec-completion-closure.md"];
    case "documentation-proof":
      return ["runbooks/spec-completion-closure.md"];
    default:
      return ["runbooks/spec-completion-closure.md"];
  }
}

function returnedEvidenceForClassification(classification) {
  switch (classification) {
    case "manual-review":
      return [".tmp/manual-review/<work-order-id>/signoff.json", ".tmp/manual-review/<work-order-id>/artifacts/"];
    case "external-evidence":
      return [".tmp/performance-profile/external/<host>/performance-profile-evidence.json"];
    case "cross-platform-evidence":
      return [".tmp/platform-evidence/external/<platform>/platform-evidence.json", ".tmp/external-engines/external/<platform>/external-engine-evidence.json"];
    case "release-credentials":
      return [".tmp/release-signing/external/<platform>/signing-evidence.json"];
    case "distribution-artifacts":
      return [".tmp/homebrew/release-cask-evidence.json", ".tmp/release-signing/external/macos/signing-evidence.json"];
    case "local-implementation":
      return ["committed source changes", "test output pasted into docs/progress.md if the matrix row changes"];
    case "local-proof":
      return [".tmp/<focused-proof>/report.json", "test output pasted into docs/progress.md if the matrix row changes"];
    case "documentation-proof":
      return ["updated docs/spec-completion-matrix.md row with exact command and artifact evidence"];
    default:
      return [".tmp/spec-completion/<work-order-id>/evidence.json"];
  }
}

function validatorCommandsForClassification(classification) {
  switch (classification) {
    case "manual-review":
      return ["pnpm run test:rendered-exports", "pnpm run check:a11y:manual", "pnpm run check:tables:manual"];
    case "external-evidence":
      return ["pnpm run check:performance-profile", "pnpm run check:ai-runtime", "pnpm run check:ai-provider"];
    case "cross-platform-evidence":
      return ["pnpm run check:platform-evidence", "pnpm run check:engines", "pnpm run test:tauri-webdriver"];
    case "release-credentials":
      return ["pnpm run check:release-signing", "pnpm run check:platform-evidence"];
    case "distribution-artifacts":
      return ["pnpm run check:homebrew", "pnpm run check:release-signing"];
    case "local-implementation":
      return ["pnpm run test:unit", "pnpm run check", "pnpm run check:spec-completion"];
    case "local-proof":
      return ["pnpm run test:unit", "pnpm run check:spec-completion"];
    case "documentation-proof":
      return ["pnpm run check:docs", "pnpm run check:spec-completion"];
    default:
      return ["pnpm run check:spec-completion"];
  }
}

function objectiveForWorkOrder(row) {
  return `Close the ${row.classification} evidence gap for ${row.specSection} / ${row.requirementArea}: ${row.remainingGap}`;
}

function acceptanceCriteriaForClassification(classification) {
  const common = [
    "Evidence names the reviewer or host, app version, Git commit, platform, and generated artifact paths.",
    "Evidence is current for the source tree under review and contains no secrets, document content, API keys, or raw audio samples.",
    "The listed validator commands pass after the evidence is copied or ingested.",
    "pnpm run check:release-readiness and pnpm run check:spec-completion are rerun after ingestion.",
  ];
  switch (classification) {
    case "manual-review":
      return [
        "A named reviewer signs every checklist item as pass or explicitly records a non-release-blocking exception.",
        "Screenshots, exported files, or native-viewer artifacts are referenced by path.",
        "No unresolved blocker remains in the sign-off JSON.",
        ...common,
      ];
    case "cross-platform-evidence":
      return [
        "Evidence comes from the named supported host rather than the current host guessing on its behalf.",
        "Package, WebDriver, or engine proof matches the current package version and Git commit.",
        "Validator-shaped JSON is returned under the expected external evidence directory.",
        ...common,
      ];
    case "release-credentials":
      return [
        "Signing, notarization, or attestation evidence is collected on a clean credentialed release host.",
        "Artifact hashes match the distributable files used for publication.",
        "No private certificate, token, or signing secret is stored in the evidence.",
        ...common,
      ];
    case "distribution-artifacts":
      return [
        "The final distributable artifact exists and has a pinned SHA-256.",
        "Homebrew or release metadata references the same signed/notarized artifact.",
        "Installation instructions and verification commands match the published artifact.",
        ...common,
      ];
    default:
      return common;
  }
}

function notesForClassification(classification) {
  switch (classification) {
    case "manual-review":
      return "Use this as a reviewer assignment. Attach the completed sign-off to the evidence kit before marking the matrix row complete.";
    case "cross-platform-evidence":
      return "Run on the actual Windows/Linux/macOS host named by the gap; do not synthesize platform proof locally.";
    case "release-credentials":
      return "Requires release credentials. Do not check in signing secrets or unredacted notarization logs.";
    case "distribution-artifacts":
      return "Requires final release artifacts. Do not close from template cask or placeholder SHA evidence.";
    default:
      return "Close with direct code, command, artifact, and matrix evidence before claiming completion.";
  }
}

function renderGapPlanMarkdown(openRowPlans, gapTriage, summary, status) {
  const lines = [
    "# NEditor Spec Gap Plan",
    "",
    `Generated: ${new Date().toISOString()}`,
    `Status: ${status}`,
    "",
    "## Summary",
    "",
    `- Total matrix rows: ${summary.totalRows}`,
    `- Open rows: ${summary.openRows}`,
    `- Locally closable rows: ${gapTriage.locallyClosableRows}`,
    `- External/manual/distribution evidence rows: ${gapTriage.evidenceBlockedRows}`,
    "",
    "## Triage",
    "",
    "| Classification | Rows |",
    "| --- | ---: |",
    ...Object.entries(gapTriage.byClassification).map(([classification, count]) => `| ${classification} | ${count} |`),
    "",
    "## First Local Actions",
    "",
    "| Spec section | Requirement area | Classification | Next action |",
    "| --- | --- | --- | --- |",
    ...gapTriage.firstLocalActions.map(
      (row) =>
        `| ${escapeMarkdownTableCell(row.specSection)} | ${escapeMarkdownTableCell(row.requirementArea)} | ${row.classification} | ${escapeMarkdownTableCell(row.nextAction)} |`,
    ),
    "",
    "## All Open Rows",
    "",
    "| Spec section | Requirement area | Status | Classification | Remaining gap | Next action |",
    "| --- | --- | --- | --- | --- | --- |",
    ...openRowPlans.map(
      (row) =>
        `| ${escapeMarkdownTableCell(row.specSection)} | ${escapeMarkdownTableCell(row.requirementArea)} | ${row.status} | ${row.classification} | ${escapeMarkdownTableCell(row.remainingGap)} | ${escapeMarkdownTableCell(row.nextAction)} |`,
    ),
    "",
  ];
  return lines.join("\n");
}

function renderWorkOrdersMarkdown(workOrders, summary, status) {
  const lines = [
    "# NEditor Spec Completion Work Orders",
    "",
    `Generated: ${new Date().toISOString()}`,
    `Status: ${status}`,
    "",
    "## Summary",
    "",
    `- Total matrix rows: ${summary.totalRows}`,
    `- Work orders: ${workOrders.length}`,
    `- Ready to send: ${workOrders.filter((order) => order.readyToSend).length}`,
    "",
    "## Work Order Index",
    "",
    "| ID | Owner | Classification | Requirement |",
    "| --- | --- | --- | --- |",
    ...workOrders.map(
      (order) =>
        `| ${order.id} | ${escapeMarkdownTableCell(order.owner)} | ${order.classification} | ${escapeMarkdownTableCell(`${order.specSection} / ${order.requirementArea}`)} |`,
    ),
    "",
  ];
  for (const order of workOrders) {
    lines.push(
      `## ${order.id}`,
      "",
      `Owner: ${order.owner}`,
      `Classification: ${order.classification}`,
      `Requirement: ${order.specSection} / ${order.requirementArea}`,
      "",
      "Objective:",
      "",
      order.objective,
      "",
      "Acceptance criteria:",
      "",
      ...order.acceptanceCriteria.map((item) => `- ${item}`),
      "",
      "Runbooks:",
      "",
      ...order.runbooks.map((item) => `- ${item}`),
      "",
      "Return evidence:",
      "",
      ...order.returns.map((item) => `- ${item}`),
      "",
      "Validator commands:",
      "",
      ...order.validatorCommands.map((item) => `- \`${item}\``),
      "",
      `Ingest: \`${order.ingestCommand}\``,
      `Final readiness: \`${order.finalReadinessCommand}\``,
      `Matrix closure: \`${order.matrixClosureCommand}\``,
      "",
      `Notes: ${order.notes}`,
      "",
    );
  }
  return lines.join("\n");
}

function escapeMarkdownTableCell(value) {
  return String(value ?? "")
    .replace(/\\/g, "\\\\")
    .replace(/\|/g, "\\|")
    .replace(/\r?\n/g, " ")
    .trim();
}

function slugify(value) {
  return String(value ?? "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 64) || "work-order";
}

function splitTableRow(line) {
  const cells = [];
  let cell = "";
  let escaped = false;
  const trimmed = line.trim().replace(/^\|/, "").replace(/\|$/, "");
  for (const char of trimmed) {
    if (char === "|" && !escaped) {
      cells.push(cell);
      cell = "";
    } else {
      cell += char;
    }
    escaped = char === "\\" && !escaped;
    if (char !== "\\") escaped = false;
  }
  cells.push(cell);
  return cells;
}

function cleanCell(value) {
  return value.replace(/\\\|/g, "|").replace(/<br\s*\/?>/gi, " ").replace(/\s+/g, " ").trim();
}

function isPlaceholder(value) {
  return /^(?:n\/a|none|tbd|todo|-|\.)$/i.test(value.trim());
}

function hasSubstantiveGap(value) {
  return value.trim().length >= 12 && !/^(?:keep current|revisit only if|no gap)$/i.test(value.trim());
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
