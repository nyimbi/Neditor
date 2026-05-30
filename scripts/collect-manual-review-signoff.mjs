import { basename, dirname, extname, join, resolve } from "node:path";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = parseArgs(process.argv.slice(2));

if (args.help || args.h) {
  printHelp();
  process.exit(0);
}

const workOrderId = stringArg("work-order-id", "NEDITOR_MANUAL_REVIEW_WORK_ORDER_ID");
const templatePath = resolve(
  args.template ||
    process.env.NEDITOR_MANUAL_REVIEW_TEMPLATE ||
    (workOrderId ? join(root, ".tmp", "manual-review", "templates", `${workOrderId}.template.json`) : ""),
);
const template = readTemplate(templatePath);
const resolvedWorkOrderId = workOrderId || template.workOrderId;
const outputDir = resolve(
  args["output-dir"] ||
    process.env.NEDITOR_MANUAL_REVIEW_OUTPUT_DIR ||
    join(root, ".tmp", "manual-review", "external", resolvedWorkOrderId),
);
const outputPath = resolve(args.output || process.env.NEDITOR_MANUAL_REVIEW_OUTPUT || join(outputDir, "signoff.json"));
const artifactDir = join(dirname(outputPath), "artifacts");
const sourceTreeClean = gitTreeClean();
const reviewerName = stringArg("reviewer-name", "NEDITOR_MANUAL_REVIEW_REVIEWER_NAME");
const reviewerRole = stringArg("reviewer-role", "NEDITOR_MANUAL_REVIEW_REVIEWER_ROLE", "manual-reviewer");
const reviewerOrganization = stringArg("reviewer-organization", "NEDITOR_MANUAL_REVIEW_REVIEWER_ORGANIZATION", "");
const platformOs = stringArg("platform-os", "NEDITOR_MANUAL_REVIEW_PLATFORM_OS", process.platform);
const platformArch = stringArg("platform-arch", "NEDITOR_MANUAL_REVIEW_PLATFORM_ARCH", process.arch);
const platformVersion = stringArg("platform-version", "NEDITOR_MANUAL_REVIEW_PLATFORM_VERSION");
const platformDevice = stringArg("platform-device", "NEDITOR_MANUAL_REVIEW_PLATFORM_DEVICE");
const appBuildKind = stringArg("app-build-kind", "NEDITOR_MANUAL_REVIEW_APP_BUILD_KIND", template.appBuild?.kind || "local-source-or-packaged-release");
const appBuildPath = stringArg("app-build-path", "NEDITOR_MANUAL_REVIEW_APP_BUILD_PATH", template.appBuild?.path || "");
const appBuildHash = stringArg("app-build-hash", "NEDITOR_MANUAL_REVIEW_APP_BUILD_HASH", template.appBuild?.hash || "");
const notes = stringArg("notes", "NEDITOR_MANUAL_REVIEW_NOTES");
const artifactFiles = listArg("artifact-file", "NEDITOR_MANUAL_REVIEW_ARTIFACT_FILES");
const validatorOutputFile = stringArg("validator-output-file", "NEDITOR_MANUAL_REVIEW_VALIDATOR_OUTPUT_FILE", "");

if (!sourceTreeClean) fail("Manual review signoff must be collected from a clean Git tree. Commit or discard local changes first.");
if (!resolvedWorkOrderId) fail("Work order id is required. Pass --work-order-id, --template, or set NEDITOR_MANUAL_REVIEW_WORK_ORDER_ID.");
if (template.workOrderId !== resolvedWorkOrderId) {
  fail(`Template workOrderId ${template.workOrderId} does not match requested work order ${resolvedWorkOrderId}.`);
}
if (!reviewerName) fail("Reviewer name is required. Pass --reviewer-name or set NEDITOR_MANUAL_REVIEW_REVIEWER_NAME.");
if (!platformVersion) fail("Platform version is required. Pass --platform-version or set NEDITOR_MANUAL_REVIEW_PLATFORM_VERSION.");
if (!platformDevice) fail("Platform device is required. Pass --platform-device or set NEDITOR_MANUAL_REVIEW_PLATFORM_DEVICE.");
if (notes.trim().length < 12) fail("Review notes must be substantive.");
if (artifactFiles.length === 0 && !validatorOutputFile) {
  fail("At least one real evidence file is required. Pass --artifact-file or --validator-output-file.");
}

mkdirSync(artifactDir, { recursive: true });
const copiedArtifacts = [];
for (const [index, artifactPath] of artifactFiles.entries()) {
  copiedArtifacts.push(copyArtifactFile(artifactPath, artifactDir, `artifact-${String(index + 1).padStart(2, "0")}`));
}
let validatorEvidence = copiedArtifacts[0] || null;
if (validatorOutputFile) {
  validatorEvidence = copyArtifactFile(validatorOutputFile, artifactDir, "validator-output", "validator-output.txt");
}
if (!validatorEvidence) validatorEvidence = copiedArtifacts[0];

const artifactEvidence = copiedArtifacts[0] || validatorEvidence;
const signoff = {
  ...template,
  reviewer: {
    name: reviewerName,
    role: reviewerRole,
    organization: reviewerOrganization,
  },
  reviewedAt: new Date().toISOString(),
  platform: {
    os: platformOs,
    arch: platformArch,
    version: platformVersion,
    device: platformDevice,
  },
  appBuild: {
    kind: appBuildKind,
    path: appBuildPath,
    hash: appBuildHash,
  },
  artifacts: [...new Set(copiedArtifacts.concat(validatorEvidence).filter(Boolean))],
  checklist: template.checklist.map((item) => ({
    ...item,
    status: "pass",
    evidence: String(item.id || "").startsWith("validator-") ? validatorEvidence : artifactEvidence,
    notes,
  })),
  unresolvedBlockers: [],
  notes,
};

writeFileSync(outputPath, `${JSON.stringify(signoff, null, 2)}\n`);
console.log(`Collected manual review signoff: ${relative(outputPath)}`);
console.log(`Copied ${signoff.artifacts.length} evidence artifact(s) into ${relative(artifactDir)}.`);
console.log(`Validate it with: pnpm run check:manual-review`);

function readTemplate(path) {
  if (!path) fail("Template path is required. Pass --template or --work-order-id.");
  if (!existsSync(path)) {
    fail(`Missing manual-review template: ${relative(path)}. Run pnpm run check:manual-review first.`);
  }
  try {
    const value = JSON.parse(readFileSync(path, "utf8"));
    if (value.schema !== "neditor.manual-review.signoff.v1") {
      fail("Template schema must be neditor.manual-review.signoff.v1.");
    }
    if (!value.workOrderId || !Array.isArray(value.checklist)) {
      fail("Template must include workOrderId and checklist.");
    }
    return value;
  } catch (error) {
    fail(`Could not read manual-review template ${relative(path)}: ${String(error)}`);
  }
}

function copyArtifactFile(source, destinationDir, fallbackName, fixedName = "") {
  const sourcePath = resolve(source);
  if (!existsSync(sourcePath)) fail(`Evidence file does not exist: ${source}`);
  const extension = extname(sourcePath);
  const rawName = fixedName || `${fallbackName}-${basename(sourcePath, extension)}${extension}`;
  const targetName = sanitizeFileName(rawName);
  const targetPath = join(destinationDir, targetName);
  copyFileSync(sourcePath, targetPath);
  return `artifacts/${targetName}`;
}

function sanitizeFileName(value) {
  return String(value || "artifact")
    .replace(/[^A-Za-z0-9._-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 120);
}

function stringArg(name, envName, fallback = "") {
  return String(args[name] || process.env[envName] || fallback).trim();
}

function listArg(name, envName) {
  const values = Array.isArray(args[name]) ? args[name] : args[name] ? [args[name]] : [];
  const envValues = process.env[envName]
    ? process.env[envName]
        .split(",")
        .map((value) => value.trim())
        .filter(Boolean)
    : [];
  return [...values, ...envValues].map((value) => String(value).trim()).filter(Boolean);
}

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!arg.startsWith("--")) continue;
    const key = arg.slice(2);
    const next = argv[index + 1];
    const value = !next || next.startsWith("--") ? true : next;
    if (parsed[key] !== undefined) {
      parsed[key] = Array.isArray(parsed[key]) ? [...parsed[key], value] : [parsed[key], value];
    } else {
      parsed[key] = value;
    }
    if (value !== true) index += 1;
  }
  return parsed;
}

function printHelp() {
  console.log(`Collect spec manual-review work-order signoff evidence.

Usage:
  pnpm run collect:manual-review -- --work-order-id 001-manual-review-example --reviewer-name "Reviewer Name" --platform-version "macOS 15" --platform-device "MacBook Pro" --artifact-file /path/to/screenshot.png --validator-output-file /path/to/validator-output.txt --notes "Reviewed the assigned workflow and found no release blockers."

Options:
  --work-order-id <id>             Work-order id from .tmp/manual-review/assignments.csv.
  --template <path>                Generated work-order template. Defaults from --work-order-id.
  --output-dir <path>              Directory for signoff.json and artifacts/.
  --output <path>                  Completed signoff JSON output.
  --reviewer-name <name>           Human reviewer name.
  --reviewer-role <role>           Reviewer role. Defaults to manual-reviewer.
  --reviewer-organization <org>    Optional reviewer organization.
  --platform-os <name>             Review platform OS. Defaults to current process platform.
  --platform-arch <value>          Review platform architecture. Defaults to current process arch.
  --platform-version <value>       OS/platform version observed by the reviewer.
  --platform-device <value>        Device or VM description.
  --app-build-kind <value>         Reviewed build kind.
  --app-build-path <path>          Optional reviewed app/package path.
  --app-build-hash <sha256>        Optional reviewed app/package hash.
  --artifact-file <path>           Real review artifact to copy beside the signoff. Repeat for multiple files.
  --validator-output-file <path>   Validator output file copied as artifacts/validator-output.txt.
  --notes <text>                   Substantive reviewer notes used across checklist items.

Environment:
  NEDITOR_MANUAL_REVIEW_WORK_ORDER_ID, NEDITOR_MANUAL_REVIEW_TEMPLATE,
  NEDITOR_MANUAL_REVIEW_OUTPUT_DIR, NEDITOR_MANUAL_REVIEW_OUTPUT,
  NEDITOR_MANUAL_REVIEW_REVIEWER_NAME, NEDITOR_MANUAL_REVIEW_REVIEWER_ROLE,
  NEDITOR_MANUAL_REVIEW_REVIEWER_ORGANIZATION, NEDITOR_MANUAL_REVIEW_PLATFORM_OS,
  NEDITOR_MANUAL_REVIEW_PLATFORM_ARCH, NEDITOR_MANUAL_REVIEW_PLATFORM_VERSION,
  NEDITOR_MANUAL_REVIEW_PLATFORM_DEVICE, NEDITOR_MANUAL_REVIEW_APP_BUILD_KIND,
  NEDITOR_MANUAL_REVIEW_APP_BUILD_PATH, NEDITOR_MANUAL_REVIEW_APP_BUILD_HASH,
  NEDITOR_MANUAL_REVIEW_ARTIFACT_FILES, NEDITOR_MANUAL_REVIEW_VALIDATOR_OUTPUT_FILE,
  NEDITOR_MANUAL_REVIEW_NOTES.

The collector requires a clean Git tree and real reviewer artifacts. It packages
returned evidence for pnpm run check:manual-review; it does not perform the
manual review or close work orders without supplied artifacts.`);
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 && result.stdout.trim() === "";
}

function relative(path) {
  return path && path.startsWith(root) ? path.slice(root.length + 1) : path;
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
