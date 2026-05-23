import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const kitDir = resolve(process.env.NEDITOR_RELEASE_EVIDENCE_KIT_DIR || join(root, ".tmp", "release-evidence-kit"));
const manifestPath = join(kitDir, "manifest.json");
const readinessPath = join(root, ".tmp", "release-readiness", "report.json");
const currentSourceCommit = gitCommit();
const expectedTemplateCount = 10;
const expectedRunbooks = [
  "runbooks/windows-platform.md",
  "runbooks/linux-platform.md",
  "runbooks/release-signing.md",
  "runbooks/google-docs-import.md",
  "runbooks/rendered-export-human-review.md",
  "runbooks/accessibility-human-review.md",
];
const issues = [];

const manifest = readJson(manifestPath, "release evidence kit manifest");
const readiness = readJson(readinessPath, "release readiness report");

if (manifest && readiness) {
  validateManifest(manifest, readiness);
}

if (issues.length > 0) {
  console.error("Release evidence kit failed validation:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

console.log(`Release evidence kit is current; checked ${relative(manifestPath)}.`);

function validateManifest(manifest, readiness) {
  requireValue(manifest.schema === "neditor.release-evidence-kit.v1", "schema must be neditor.release-evidence-kit.v1");
  requireValue(manifest.appVersion === packageJson.version, `appVersion must match package.json version ${packageJson.version}`);
  requireValue(manifest.sourceCommit === currentSourceCommit, `sourceCommit must match current git commit ${currentSourceCommit}`);
  requireValue(manifest.sourceTreeClean === true, "sourceTreeClean must be true");
  requireValue(isIsoDate(manifest.generatedAt), "generatedAt must be an ISO timestamp");
  requireValue(manifest.releaseReadinessReport === relative(readinessPath), "releaseReadinessReport must point to the current readiness report");
  requireValue(manifest.readinessStatus === readiness.status, "readinessStatus must match the current release readiness report");

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

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
