import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const kitDir = resolve(process.env.NEDITOR_RELEASE_EVIDENCE_KIT_DIR || join(root, ".tmp", "release-evidence-kit"));
const manifestPath = join(kitDir, "manifest.json");
const reportPath = join(kitDir, "report.json");
const readinessPath = join(root, ".tmp", "release-readiness", "report.json");
const currentSourceCommit = gitCommit();
const currentSourceTreeClean = gitTreeClean();
const expectedTemplateCount = 18;
const expectedRunbooks = [
  "runbooks/windows-platform.md",
  "runbooks/linux-platform.md",
  "runbooks/release-signing.md",
  "runbooks/homebrew-release.md",
  "runbooks/ai-provider-endpoint.md",
  "runbooks/ai-runtime-device.md",
  "runbooks/independent-security-review.md",
  "runbooks/google-docs-import.md",
  "runbooks/rendered-export-human-review.md",
  "runbooks/release-device-performance-profile.md",
  "runbooks/table-editor-human-review.md",
  "runbooks/accessibility-human-review.md",
  "runbooks/manual-review.md",
  "runbooks/optional-external-engines.md",
  "runbooks/spec-completion-closure.md",
];
const issues = [];

const manifest = readJson(manifestPath, "release evidence kit manifest");
const readiness = readJson(readinessPath, "release readiness report");
const readinessStatus = effectiveReadinessStatus(readiness);

if (manifest && readiness) {
  validateManifest(manifest, readiness, readinessStatus);
}

writeReport(manifest, readiness, readinessStatus);

if (issues.length > 0) {
  console.error("Release evidence kit failed validation:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

console.log(`Release evidence kit is current; checked ${relative(manifestPath)}.`);

function validateManifest(manifest, readiness, readinessStatus) {
  requireValue(manifest.schema === "neditor.release-evidence-kit.v1", "schema must be neditor.release-evidence-kit.v1");
  requireValue(manifest.appVersion === packageJson.version, `appVersion must match package.json version ${packageJson.version}`);
  requireValue(manifest.sourceCommit === currentSourceCommit, `sourceCommit must match current git commit ${currentSourceCommit}`);
  requireValue(manifest.sourceTreeClean === true, "sourceTreeClean must be true");
  requireValue(currentSourceTreeClean === true, "current source tree must be clean");
  requireValue(isIsoDate(manifest.generatedAt), "generatedAt must be an ISO timestamp");
  requireValue(manifest.releaseReadinessReport === relative(readinessPath), "releaseReadinessReport must point to the current readiness report");
  requireValue(manifest.readinessStatus === readinessStatus, "readinessStatus must match the current release readiness report");

  const readinessGaps = gaps(readiness);
  const manifestGaps = Array.isArray(manifest.gaps) ? manifest.gaps : [];
  requireValue(manifestGaps.length === readinessGaps.length, "manifest gaps must mirror the release readiness report");
  const manifestGapIds = new Set(manifestGaps.map((gap) => gap.id));
  for (const gap of readinessGaps) {
    requireValue(manifestGapIds.has(gap.id), `manifest is missing readiness gap ${gap.id}`);
  }

  const copiedTemplates = Array.isArray(manifest.copiedTemplates) ? manifest.copiedTemplates : [];
  requireValue(copiedTemplates.length === expectedTemplateCount, `copiedTemplates must include ${expectedTemplateCount} entries`);
  requireValue(Array.isArray(manifest.missingTemplates) && manifest.missingTemplates.length === 0, "missingTemplates must be empty");
  requireValue(Array.isArray(manifest.staleTemplates) && manifest.staleTemplates.length === 0, "staleTemplates must be empty");
  validateSpecCompletionWorkOrders(manifest.specCompletionWorkOrders);
  validateSpecCompletionRunbooks(manifest.specCompletionRunbooks, manifest.specCompletionWorkOrders);
  validateManualReviewAssets(manifest.manualReviewAssets);
  for (const template of copiedTemplates) {
    requireValue(template.copied === true, `template must be copied: ${template.source || template.path}`);
    requireValue(template.freshness?.status === "current", `template freshness must be current: ${template.source || template.path}`);
    requireFile(join(kitDir, template.path), `copied template ${template.path}`, 10);
  }

  requireFile(join(kitDir, "README.md"), "release evidence kit README", 100);
  const runbooks = Array.isArray(manifest.runbooks) ? manifest.runbooks : [];
  const runbookPaths = new Set(runbooks.map((runbook) => runbook.path));
  for (const runbook of expectedRunbooks) {
    requireValue(runbookPaths.has(runbook), `manifest is missing runbook ${runbook}`);
    requireFile(join(kitDir, runbook), `runbook ${runbook}`, 100);
  }
  for (const runbook of runbooks) {
    requireValue(Array.isArray(runbook.validatorCommands) && runbook.validatorCommands.length > 0, `runbook ${runbook.path} must list validator commands`);
    requireValue(String(runbook.ingestCommand || "").includes("pnpm run ingest:evidence"), `runbook ${runbook.path} must list ingest command`);
    requireValue(String(runbook.finalReadinessCommand || "").includes("pnpm run check:release-readiness"), `runbook ${runbook.path} must list final readiness command`);
  }

  const gapWorkItems = Array.isArray(manifest.gapWorkItems) ? manifest.gapWorkItems : [];
  requireValue(gapWorkItems.length === readinessGaps.length, "gapWorkItems must mirror the release readiness report");
  const readinessGapIds = new Set(readinessGaps.map((gap) => gap.id));
  for (const item of gapWorkItems) {
    requireValue(readinessGapIds.has(item.id), `gapWorkItems contains unknown readiness gap ${item.id}`);
    requireValue(item.readyToSend === true, `gap work item ${item.id} must be ready to send`);
    requireValue(Array.isArray(item.runbooks) && item.runbooks.length > 0, `gap work item ${item.id} must list at least one runbook`);
    requireValue(Array.isArray(item.returns) && item.returns.length > 0, `gap work item ${item.id} must list returned evidence paths`);
    requireValue(Array.isArray(item.validatorCommands) && item.validatorCommands.length > 0, `gap work item ${item.id} must list validator commands`);
    requireValue(String(item.ingestCommand || "").includes("pnpm run ingest:evidence"), `gap work item ${item.id} must list ingest command`);
    requireValue(String(item.finalReadinessCommand || "").includes("pnpm run check:release-readiness"), `gap work item ${item.id} must list final readiness command`);
  }
}

function validateManualReviewAssets(manualReviewAssets) {
  requireValue(manualReviewAssets && typeof manualReviewAssets === "object", "manifest must include manualReviewAssets");
  const expectedManualWorkOrders = Number(manualReviewAssets?.expectedManualWorkOrders || 0);
  const templates = manualReviewAssets?.templates || {};
  requireValue(Number.isFinite(expectedManualWorkOrders) && expectedManualWorkOrders >= 0, "manualReviewAssets expectedManualWorkOrders must be numeric");
  if (expectedManualWorkOrders === 0) return;
  requireValue(templates.copied === true, "manual-review work-order templates must be copied");
  requireValue(Number(templates.total || 0) === expectedManualWorkOrders, "manual-review template count must match manual work-order count");
  requireValue(Array.isArray(templates.files) && templates.files.length === expectedManualWorkOrders, "manual-review template files must list every work order");
  for (const templatePath of Array.isArray(templates.files) ? templates.files : []) {
    requireFile(join(kitDir, templatePath), `manual-review template ${templatePath}`, 100);
  }
  requireValue(manualReviewAssets.dashboardMarkdown?.copied === true, "manual-review Markdown dashboard must be copied");
  requireValue(manualReviewAssets.dashboardHtml?.copied === true, "manual-review HTML dashboard must be copied");
  requireValue(manualReviewAssets.assignmentsCsv?.copied === true, "manual-review assignments CSV must be copied");
  requireFile(join(kitDir, manualReviewAssets.dashboardMarkdown.path || ""), "manual-review Markdown dashboard", 100);
  requireFile(join(kitDir, manualReviewAssets.dashboardHtml.path || ""), "manual-review HTML dashboard", 100);
  requireFile(join(kitDir, manualReviewAssets.assignmentsCsv.path || ""), "manual-review assignments CSV", 50);
}

function validateSpecCompletionWorkOrders(workOrders) {
  requireValue(workOrders && typeof workOrders === "object", "manifest must include specCompletionWorkOrders");
  requireValue(workOrders.jsonCopied === true, "spec completion work-orders JSON must be copied");
  requireValue(workOrders.markdownCopied === true, "spec completion work-orders Markdown must be copied");
  requireValue(workOrders.schema === "neditor.spec-completion-work-orders.v1", "spec completion work-orders schema must be neditor.spec-completion-work-orders.v1");
  requireValue(Number(workOrders.total || 0) >= 0, "spec completion work-orders total must be numeric");
  requireValue(Number(workOrders.readyToSend || 0) === Number(workOrders.total || 0), "all spec completion work orders must be ready to send");
  requireFile(join(kitDir, workOrders.jsonPath || ""), "spec completion work-orders JSON", 100);
  requireFile(join(kitDir, workOrders.markdownPath || ""), "spec completion work-orders Markdown", 100);
}

function validateSpecCompletionRunbooks(runbooks, workOrders) {
  requireValue(Array.isArray(runbooks), "manifest must include specCompletionRunbooks");
  requireValue(Array.isArray(runbooks) && runbooks.length > 0, "specCompletionRunbooks must include referenced work-order runbooks");
  const runbookPaths = new Set((Array.isArray(runbooks) ? runbooks : []).map((runbook) => runbook.path));
  const workOrderJson = readJson(join(kitDir, workOrders?.jsonPath || ""), "spec completion work-orders JSON for runbook validation");
  const referencedRunbooks = new Set(
    (Array.isArray(workOrderJson?.workOrders) ? workOrderJson.workOrders : [])
      .flatMap((order) => (Array.isArray(order.runbooks) ? order.runbooks : []))
      .filter(Boolean),
  );
  for (const referenced of referencedRunbooks) {
    requireValue(runbookPaths.has(referenced), `specCompletionRunbooks is missing referenced runbook ${referenced}`);
  }
  for (const runbook of Array.isArray(runbooks) ? runbooks : []) {
    requireValue(runbook.available === true, `spec-completion runbook must be available: ${runbook.path}`);
    requireFile(join(kitDir, runbook.path || ""), `spec-completion runbook ${runbook.path}`, 100);
  }
}

function writeReport(manifest, readiness, readinessStatus) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        schema: "neditor.release-evidence-kit-report.v1",
        generatedAt: new Date().toISOString(),
        status: issues.length === 0 ? "passed" : "failed",
        manifestPath: relative(manifestPath),
        releaseReadinessReport: relative(readinessPath),
        sourceCommit: manifest?.sourceCommit || null,
        currentSourceCommit,
        sourceTreeClean: manifest?.sourceTreeClean ?? null,
        currentSourceTreeClean,
        appVersion: manifest?.appVersion || null,
        currentAppVersion: packageJson.version,
        readinessStatus: manifest?.readinessStatus || null,
        currentReadinessStatus: readinessStatus,
        summary: {
          gaps: Array.isArray(manifest?.gaps) ? manifest.gaps.length : 0,
          copiedTemplates: Array.isArray(manifest?.copiedTemplates) ? manifest.copiedTemplates.length : 0,
          missingTemplates: Array.isArray(manifest?.missingTemplates) ? manifest.missingTemplates.length : 0,
          missingSpecCompletionRunbooks: Array.isArray(manifest?.missingSpecCompletionRunbooks) ? manifest.missingSpecCompletionRunbooks.length : 0,
          staleTemplates: Array.isArray(manifest?.staleTemplates) ? manifest.staleTemplates.length : 0,
          runbooks: Array.isArray(manifest?.runbooks) ? manifest.runbooks.length : 0,
          specCompletionRunbooks: Array.isArray(manifest?.specCompletionRunbooks) ? manifest.specCompletionRunbooks.length : 0,
          manualReviewTemplates: Number(manifest?.manualReviewAssets?.templates?.total || 0),
          manualReviewDashboardCopied: Boolean(
            manifest?.manualReviewAssets?.dashboardMarkdown?.copied || manifest?.manualReviewAssets?.dashboardHtml?.copied,
          ),
          specWorkOrders: Number(manifest?.specCompletionWorkOrders?.total || 0),
          specWorkOrdersReady: Number(manifest?.specCompletionWorkOrders?.readyToSend || 0),
          issues: issues.length,
        },
        gapIds: Array.isArray(manifest?.gaps) ? manifest.gaps.map((gap) => gap.id) : [],
        issues,
      },
      null,
      2,
    )}\n`,
  );
}

function gaps(readiness) {
  const values = Array.isArray(readiness?.evidenceGaps)
    ? readiness.evidenceGaps
    : Array.isArray(readiness?.gaps)
      ? readiness.gaps
      : [];
  return values.map((gap) => ({
    id: gap.id || gap.check || gap.name,
  }));
}

function effectiveReadinessStatus(readinessReport) {
  if (!readinessReport) return null;
  const status = readinessReport.status || "unknown";
  const failures = Array.isArray(readinessReport.failures) ? readinessReport.failures : [];
  const onlyEvidenceKitBootstrapFailure =
    status === "failed" && failures.length > 0 && failures.every((failure) => String(failure).startsWith("release-evidence-kit "));
  if (!onlyEvidenceKitBootstrapFailure) return status;
  return Number(readinessReport.summary?.evidenceGaps || 0) > 0 ? "current-host-ready-with-external-gaps" : "ready";
}

function requireFile(path, label, minBytes) {
  if (!existsSync(path)) {
    issues.push(`missing ${label}: ${relative(path)}`);
    return;
  }
  const bytes = statSync(path).size;
  if (bytes < minBytes) issues.push(`${label} is unexpectedly small: ${bytes} bytes`);
}

function readJson(path, label) {
  if (!existsSync(path)) {
    issues.push(`missing ${label}: ${relative(path)}`);
    return null;
  }
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    issues.push(`${label} is not valid JSON: ${String(error)}`);
    return null;
  }
}

function requireValue(condition, message) {
  if (!condition) issues.push(message);
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0) return "";
  return result.stdout.trim();
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 && result.stdout.trim() === "";
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
