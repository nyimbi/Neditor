import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const currentSourceCommit = gitCommit();
const currentSourceTreeClean = gitTreeClean();
const outputDir = join(root, ".tmp", "manual-review");
const templateDir = join(outputDir, "templates");
const reportPath = join(outputDir, "report.json");
const dashboardMarkdownPath = join(outputDir, "dashboard.md");
const dashboardHtmlPath = join(outputDir, "dashboard.html");
const assignmentsCsvPath = join(outputDir, "assignments.csv");
const specWorkOrdersPath = join(root, ".tmp", "spec-completion", "work-orders.json");
const signoffDir = resolve(process.env.NEDITOR_MANUAL_REVIEW_DIR || join(outputDir, "external"));
const issues = [];

mkdirSync(templateDir, { recursive: true });

const specWorkOrders = readJson(specWorkOrdersPath, "spec-completion work-orders");
const manualOrders = (Array.isArray(specWorkOrders?.workOrders) ? specWorkOrders.workOrders : []).filter(
  (order) => order.classification === "manual-review",
);
const signoffs = collectSignoffs(signoffDir);
const accepted = [];
const invalid = [];
const pending = [];

for (const order of manualOrders) {
  writeTemplate(order);
  const signoff = signoffs.get(order.id);
  if (!signoff) {
    pending.push({
      workOrderId: order.id,
      requirement: order.requirement,
      template: relativePath(join(templateDir, `${order.id}.template.json`)),
      expectedSignoff: relativePath(join(signoffDir, order.id, "signoff.json")),
    });
    continue;
  }
  const validationIssues = validateSignoff(order, signoff);
  if (validationIssues.length > 0) {
    invalid.push({
      workOrderId: order.id,
      path: relativePath(signoff.path),
      issues: validationIssues,
    });
    issues.push(...validationIssues.map((issue) => `${order.id}: ${issue}`));
    continue;
  }
  accepted.push({
    workOrderId: order.id,
    requirement: order.requirement,
    path: relativePath(signoff.path),
    reviewer: reviewerName(signoff.data),
    reviewedAt: signoff.data.reviewedAt,
    artifacts: signoff.data.artifacts,
  });
}

for (const [workOrderId, signoff] of signoffs.entries()) {
  if (!manualOrders.some((order) => order.id === workOrderId)) {
    const issue = `signoff references unknown or non-manual work order ${workOrderId}: ${relativePath(signoff.path)}`;
    invalid.push({ workOrderId, path: relativePath(signoff.path), issues: [issue] });
    issues.push(issue);
  }
}

const status = issues.length > 0 ? "failed" : pending.length > 0 ? "pending-human-review" : "human-reviewed";
const report = {
  schema: "neditor.manual-review.report.v1",
  generatedAt: new Date().toISOString(),
  status,
  appVersion: packageJson.version,
  sourceCommit: currentSourceCommit,
  sourceTreeClean: currentSourceTreeClean,
  specWorkOrdersPath: relativePath(specWorkOrdersPath),
  signoffDir: relativePath(signoffDir),
  templateDir: relativePath(templateDir),
  dashboardMarkdown: relativePath(dashboardMarkdownPath),
  dashboardHtml: relativePath(dashboardHtmlPath),
  assignmentsCsv: relativePath(assignmentsCsvPath),
  summary: {
    manualWorkOrders: manualOrders.length,
    accepted: accepted.length,
    pending: pending.length,
    invalid: invalid.length,
    issues: issues.length,
    bySpecSection: summarizeBySpecSection(manualOrders, accepted, pending, invalid),
  },
  accepted,
  pending,
  invalid,
  issues,
};

mkdirSync(dirname(reportPath), { recursive: true });
writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
writeFileSync(dashboardMarkdownPath, renderDashboardMarkdown(report));
writeFileSync(dashboardHtmlPath, renderDashboardHtml(report));
writeFileSync(assignmentsCsvPath, renderAssignmentsCsv(report));

if (issues.length > 0) {
  console.error("Manual review evidence validation failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  console.error(`Wrote ${relativePath(reportPath)}.`);
  process.exit(1);
}

console.log(
  `Manual review evidence is ${status}; ${accepted.length}/${manualOrders.length} work-order signoff(s) accepted, ${pending.length} pending. Wrote ${relativePath(reportPath)}, ${relativePath(dashboardMarkdownPath)}, ${relativePath(dashboardHtmlPath)}, and ${relativePath(assignmentsCsvPath)}.`,
);

function writeTemplate(order) {
  const template = {
    schema: "neditor.manual-review.signoff.v1",
    workOrderId: order.id,
    requirement: order.requirement,
    specSection: order.specSection,
    requirementArea: order.requirementArea,
    objective: order.objective,
    remainingGap: order.remainingGap,
    acceptanceCriteria: Array.isArray(order.acceptanceCriteria) ? order.acceptanceCriteria : [],
    runbooks: Array.isArray(order.runbooks) ? order.runbooks : [],
    validatorCommands: Array.isArray(order.validatorCommands) ? order.validatorCommands : [],
    ingestCommand: order.ingestCommand || "pnpm run ingest:evidence -- --source <returned-evidence-dir>",
    finalReadinessCommand: order.finalReadinessCommand || "pnpm run check:release-readiness",
    matrixClosureCommand: order.matrixClosureCommand || "pnpm run check:spec-completion",
    appVersion: packageJson.version,
    sourceCommit: currentSourceCommit || "replace-with-current-git-commit",
    sourceTreeClean: currentSourceTreeClean,
    reviewer: {
      name: "",
      role: "",
      organization: "",
    },
    reviewedAt: new Date().toISOString(),
    platform: {
      os: process.platform,
      arch: process.arch,
      version: "",
      device: "",
    },
    appBuild: {
      kind: "local-source-or-packaged-release",
      path: "",
      hash: "",
    },
    artifacts: ["artifacts/screenshot-or-export-proof.png", "artifacts/validator-output.txt"],
    checklist: checklistForOrder(order),
    unresolvedBlockers: [],
    notes: "",
  };
  writeFileSync(join(templateDir, `${order.id}.template.json`), `${JSON.stringify(template, null, 2)}\n`);
}

function checklistForOrder(order) {
  const base = [
    {
      id: "current-source-identity",
      label: `Confirm app version ${packageJson.version}, Git commit ${currentSourceCommit || "<unknown>"}, and clean-source provenance before review.`,
      status: "pending",
      evidence: "artifacts/validator-output.txt",
      notes: "",
    },
    {
      id: "workflow-observed",
      label: `Exercise and observe: ${order.requirement}`,
      status: "pending",
      evidence: "artifacts/screenshot-or-export-proof.png",
      notes: "",
    },
    {
      id: "artifact-evidence",
      label: "Attach screenshots, exported files, native-viewer proof, or screen recordings as appropriate.",
      status: "pending",
      evidence: "artifacts/screenshot-or-export-proof.png",
      notes: "",
    },
    {
      id: "no-release-blockers",
      label: "Confirm no unresolved release blocker remains for this work order.",
      status: "pending",
      evidence: "artifacts/screenshot-or-export-proof.png",
      notes: "",
    },
  ];
  const acceptanceCriteria = (Array.isArray(order.acceptanceCriteria) ? order.acceptanceCriteria : []).map((criterion, index) => ({
    id: `acceptance-${String(index + 1).padStart(2, "0")}`,
    label: String(criterion),
    status: "pending",
    evidence: "artifacts/screenshot-or-export-proof.png",
    notes: "",
  }));
  return base.concat(
    acceptanceCriteria,
    (Array.isArray(order.validatorCommands) ? order.validatorCommands : []).map((command, index) => ({
      id: `validator-${String(index + 1).padStart(2, "0")}`,
      label: `Validator command passed: ${command}`,
      status: "pending",
      evidence: "artifacts/validator-output.txt",
      notes: "",
    })),
  );
}

function summarizeBySpecSection(manualOrders, accepted, pending, invalid) {
  const acceptedById = new Set(accepted.map((item) => item.workOrderId));
  const pendingById = new Set(pending.map((item) => item.workOrderId));
  const invalidById = new Set(invalid.map((item) => item.workOrderId));
  const bySection = new Map();
  for (const order of manualOrders) {
    const section = order.specSection || "Unspecified";
    const entry = bySection.get(section) || {
      total: 0,
      accepted: 0,
      pending: 0,
      invalid: 0,
    };
    entry.total += 1;
    if (acceptedById.has(order.id)) entry.accepted += 1;
    if (pendingById.has(order.id)) entry.pending += 1;
    if (invalidById.has(order.id)) entry.invalid += 1;
    bySection.set(section, entry);
  }
  return Object.fromEntries([...bySection.entries()].sort(([a], [b]) => a.localeCompare(b)));
}

function renderDashboardMarkdown(report) {
  const summary = report.summary || {};
  const sectionRows = Object.entries(summary.bySpecSection || {}).map(
    ([section, counts]) =>
      `| ${cell(section)} | ${counts.total || 0} | ${counts.accepted || 0} | ${counts.pending || 0} | ${counts.invalid || 0} |`,
  );
  const pendingRows = report.pending.map(
    (item) =>
      `| ${cell(item.workOrderId)} | ${cell(item.requirement)} | \`${item.template}\` | \`${item.expectedSignoff}\` |`,
  );
  const acceptedRows = report.accepted.map(
    (item) =>
      `| ${cell(item.workOrderId)} | ${cell(item.requirement)} | ${cell(item.reviewer)} | ${cell(item.reviewedAt)} | \`${item.path}\` |`,
  );
  const invalidRows = report.invalid.map(
    (item) => `| ${cell(item.workOrderId)} | \`${item.path}\` | ${cell(item.issues.join("; "))} |`,
  );
  return `${[
    "# NEditor Manual Review Dashboard",
    "",
    `Generated: ${report.generatedAt}`,
    `Status: **${report.status}**`,
    `App version: \`${report.appVersion}\``,
    `Source commit: \`${report.sourceCommit || "<unknown>"}\``,
    `Source tree clean: ${report.sourceTreeClean ? "yes" : "no"}`,
    "",
    "## Summary",
    "",
    `- Manual work orders: ${summary.manualWorkOrders || 0}`,
    `- Accepted: ${summary.accepted || 0}`,
    `- Pending: ${summary.pending || 0}`,
    `- Invalid: ${summary.invalid || 0}`,
    `- Template directory: \`${report.templateDir}\``,
    `- Sign-off directory: \`${report.signoffDir}\``,
    `- Assignment CSV: \`${report.assignmentsCsv}\``,
    "",
    "## Workflow",
    "",
    "1. Send each pending template plus its runbook and artifact folder to the named reviewer.",
    "2. Keep screenshots, native-viewer exports, screen recordings, and validator output beside the completed sign-off JSON.",
    "3. Ingest returned evidence with `pnpm run ingest:evidence -- --source <returned-evidence-dir>`.",
    "4. Rerun `pnpm run check:manual-review`, `pnpm run check:release-readiness`, and `pnpm run check:spec-completion`.",
    "",
    "## By Spec Section",
    "",
    "| Spec section | Total | Accepted | Pending | Invalid |",
    "| --- | ---: | ---: | ---: | ---: |",
    ...(sectionRows.length ? sectionRows : ["| - | 0 | 0 | 0 | 0 |"]),
    "",
    "## Pending Work Orders",
    "",
    "| Work order | Requirement | Template | Expected sign-off |",
    "| --- | --- | --- | --- |",
    ...(pendingRows.length ? pendingRows : ["| - | No pending manual-review work orders | - | - |"]),
    "",
    "## Accepted Sign-Offs",
    "",
    "| Work order | Requirement | Reviewer | Reviewed at | Path |",
    "| --- | --- | --- | --- | --- |",
    ...(acceptedRows.length ? acceptedRows : ["| - | No accepted manual-review sign-offs yet | - | - | - |"]),
    "",
    "## Invalid Sign-Offs",
    "",
    "| Work order | Path | Issues |",
    "| --- | --- | --- |",
    ...(invalidRows.length ? invalidRows : ["| - | No invalid manual-review sign-offs | - |"]),
    "",
  ].join("\n")}\n`;
}

function renderDashboardHtml(report) {
  const markdown = renderDashboardMarkdown(report);
  const body = markdown
    .split(/\r?\n/)
    .map((line) => {
      if (line.startsWith("# ")) return `<h1>${escapeHtml(line.slice(2))}</h1>`;
      if (line.startsWith("## ")) return `<h2>${escapeHtml(line.slice(3))}</h2>`;
      if (line.startsWith("- ")) return `<p class="bullet">${escapeHtml(line)}</p>`;
      if (/^\d+\.\s+/.test(line)) return `<p class="step">${escapeHtml(line)}</p>`;
      if (line.startsWith("|")) return `<pre>${escapeHtml(line)}</pre>`;
      if (!line.trim()) return "";
      return `<p>${escapeHtml(line).replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>").replace(/`([^`]+)`/g, "<code>$1</code>")}</p>`;
    })
    .join("\n");
  return `${[
    "<!doctype html>",
    '<html lang="en">',
    "<head>",
    '<meta charset="utf-8" />',
    "<title>NEditor Manual Review Dashboard</title>",
    "<style>",
    "body{font-family:Inter,Arial,sans-serif;margin:32px;line-height:1.45;color:#17202a;background:#fbfcfd}",
    "h1,h2{color:#111827}code{background:#eef2f7;padding:2px 4px;border-radius:4px}pre{white-space:pre-wrap;background:#fff;border:1px solid #d7dde6;border-radius:6px;padding:8px 10px;margin:6px 0}.bullet,.step{margin:4px 0}",
    "</style>",
    "</head>",
    "<body>",
    body,
    "</body>",
    "</html>",
  ].join("\n")}\n`;
}

function renderAssignmentsCsv(report) {
  const header = ["workOrderId", "status", "requirement", "template", "expectedSignoff", "reviewer", "reviewedAt", "issues"];
  const rows = [
    ...report.pending.map((item) => [
      item.workOrderId,
      "pending",
      item.requirement,
      item.template,
      item.expectedSignoff,
      "",
      "",
      "",
    ]),
    ...report.accepted.map((item) => [
      item.workOrderId,
      "accepted",
      item.requirement,
      "",
      item.path,
      item.reviewer,
      item.reviewedAt,
      "",
    ]),
    ...report.invalid.map((item) => [item.workOrderId, "invalid", "", "", item.path, "", "", item.issues.join("; ")]),
  ];
  return `${[header, ...rows].map((row) => row.map(csvCell).join(",")).join("\n")}\n`;
}

function cell(value) {
  return String(value ?? "").replace(/\|/g, "\\|").replace(/\s+/g, " ").trim();
}

function csvCell(value) {
  const text = String(value ?? "");
  return `"${text.replace(/"/g, '""')}"`;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function collectSignoffs(dir) {
  const signoffs = new Map();
  if (!existsSync(dir)) return signoffs;
  for (const path of walkJson(dir)) {
    let data;
    try {
      data = JSON.parse(readFileSync(path, "utf8"));
    } catch {
      continue;
    }
    if (data?.schema !== "neditor.manual-review.signoff.v1") continue;
    const workOrderId = String(data.workOrderId || "").trim();
    if (!workOrderId) continue;
    signoffs.set(workOrderId, { path, dir: dirname(path), data });
  }
  return signoffs;
}

function walkJson(dir) {
  const entries = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      entries.push(...walkJson(path));
    } else if (entry.isFile() && entry.name.endsWith(".json")) {
      entries.push(path);
    }
  }
  return entries;
}

function validateSignoff(order, signoff) {
  const foundIssues = [];
  const data = signoff.data;
  if (data.schema !== "neditor.manual-review.signoff.v1") foundIssues.push("schema must be neditor.manual-review.signoff.v1");
  if (data.workOrderId !== order.id) foundIssues.push(`workOrderId must be ${order.id}`);
  if (data.appVersion !== packageJson.version) foundIssues.push(`appVersion must match package.json version ${packageJson.version}`);
  if (data.sourceCommit !== currentSourceCommit) foundIssues.push(`sourceCommit must match current git commit ${currentSourceCommit}`);
  if (data.sourceTreeClean !== true) foundIssues.push("sourceTreeClean must be true in completed signoff");
  if (!currentSourceTreeClean) foundIssues.push("current source tree must be clean before accepting completed signoff");
  if (!reviewerName(data)) foundIssues.push("reviewer.name or reviewer string is required");
  if (!isIsoDate(data.reviewedAt)) foundIssues.push("reviewedAt must be an ISO timestamp");
  if (!platformValue(data.platform)) foundIssues.push("platform must name the review platform");
  if (!Array.isArray(data.unresolvedBlockers)) foundIssues.push("unresolvedBlockers must be an array");
  if (Array.isArray(data.unresolvedBlockers) && data.unresolvedBlockers.length > 0) foundIssues.push("unresolvedBlockers must be empty");
  if (!Array.isArray(data.artifacts) || data.artifacts.length === 0) foundIssues.push("artifacts must list at least one evidence file");
  for (const artifact of Array.isArray(data.artifacts) ? data.artifacts : []) {
    validateRelativeEvidencePath(signoff.dir, artifact, "artifact", foundIssues);
  }

  if (!Array.isArray(data.checklist) || data.checklist.length === 0) {
    foundIssues.push("checklist must contain at least one item");
  } else {
    for (const item of data.checklist) {
      const id = String(item?.id || "").trim();
      const status = String(item?.status || "").trim();
      if (!id) foundIssues.push("every checklist item must have an id");
      if (!["pass", "exception"].includes(status)) {
        foundIssues.push(`checklist item ${id || "<missing-id>"} status must be pass or exception`);
      }
      if (!String(item?.evidence || "").trim()) {
        foundIssues.push(`checklist item ${id || "<missing-id>"} must include evidence`);
      } else {
        validateRelativeEvidencePath(signoff.dir, item.evidence, `checklist item ${id || "<missing-id>"} evidence`, foundIssues);
      }
      if (status === "exception" && !String(item?.notes || "").trim()) {
        foundIssues.push(`checklist item ${id || "<missing-id>"} exception must include notes`);
      }
    }
  }

  const serialized = JSON.stringify(data);
  if (/(api[_-]?key|secret|token|password)\s*[:=]\s*["']?[A-Za-z0-9_\-.]{12,}/i.test(serialized)) {
    foundIssues.push("signoff appears to contain secret-like material");
  }
  return foundIssues;
}

function validateRelativeEvidencePath(baseDir, value, label, foundIssues) {
  const relativeEvidence = String(value || "").trim();
  if (!relativeEvidence) {
    foundIssues.push(`${label} path is empty`);
    return;
  }
  if (relativeEvidence.startsWith("/") || relativeEvidence.includes("..")) {
    foundIssues.push(`${label} must be a relative path inside the signoff directory: ${relativeEvidence}`);
    return;
  }
  const path = resolve(baseDir, relativeEvidence);
  if (!path.startsWith(resolve(baseDir))) {
    foundIssues.push(`${label} escapes the signoff directory: ${relativeEvidence}`);
    return;
  }
  if (!existsSync(path) || !statSync(path).isFile()) {
    foundIssues.push(`${label} file is missing: ${relativeEvidence}`);
  }
}

function reviewerName(data) {
  if (typeof data.reviewer === "string") return data.reviewer.trim();
  return String(data.reviewer?.name || "").trim();
}

function platformValue(value) {
  if (typeof value === "string") return value.trim();
  return [value?.os, value?.arch, value?.version, value?.device].filter(Boolean).join(" ").trim();
}

function readJson(path, label) {
  if (!existsSync(path)) {
    issues.push(`${label} is missing: ${relativePath(path)}; run pnpm run check:spec-completion first`);
    return null;
  }
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    issues.push(`${label} is not valid JSON: ${String(error)}`);
    return null;
  }
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value)) && /\d{4}-\d{2}-\d{2}T/.test(value);
}

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  return result.status === 0 ? result.stdout.trim() : null;
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--short"], { cwd: root, encoding: "utf8" });
  return result.status === 0 && result.stdout.trim() === "";
}

function relativePath(path) {
  return path && path.startsWith(root) ? relative(root, path) : path;
}
