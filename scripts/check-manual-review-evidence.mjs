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
  summary: {
    manualWorkOrders: manualOrders.length,
    accepted: accepted.length,
    pending: pending.length,
    invalid: invalid.length,
    issues: issues.length,
  },
  accepted,
  pending,
  invalid,
  issues,
};

mkdirSync(dirname(reportPath), { recursive: true });
writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);

if (issues.length > 0) {
  console.error("Manual review evidence validation failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  console.error(`Wrote ${relativePath(reportPath)}.`);
  process.exit(1);
}

console.log(
  `Manual review evidence is ${status}; ${accepted.length}/${manualOrders.length} work-order signoff(s) accepted, ${pending.length} pending. Wrote ${relativePath(reportPath)}.`,
);

function writeTemplate(order) {
  const template = {
    schema: "neditor.manual-review.signoff.v1",
    workOrderId: order.id,
    requirement: order.requirement,
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
    artifacts: ["artifacts/screenshot-or-export-proof.png"],
    checklist: checklistForOrder(order),
    unresolvedBlockers: [],
    notes: "",
  };
  writeFileSync(join(templateDir, `${order.id}.template.json`), `${JSON.stringify(template, null, 2)}\n`);
}

function checklistForOrder(order) {
  const base = [
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
  return base.concat(
    (Array.isArray(order.validatorCommands) ? order.validatorCommands : []).map((command, index) => ({
      id: `validator-${String(index + 1).padStart(2, "0")}`,
      label: `Validator command passed: ${command}`,
      status: "pending",
      evidence: "artifacts/validator-output.txt",
      notes: "",
    })),
  );
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
