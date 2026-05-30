import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = parseArgs(process.argv.slice(2));

if (args.help || args.h) {
  printHelp();
  process.exit(0);
}

const auditDir = resolve(
  args["audit-dir"] ||
    process.env.NEDITOR_RENDERED_EXPORT_AUDIT_DIR ||
    join(root, ".tmp", "rendered-export-audit"),
);
const templatePath = resolve(
  args.template ||
    process.env.NEDITOR_RENDERED_EXPORT_SIGNOFF_TEMPLATE ||
    join(auditDir, "visual-review-signoff.template.json"),
);
const outputPath = resolve(
  args.output ||
    process.env.NEDITOR_RENDERED_EXPORT_SIGNOFF_OUTPUT ||
    join(root, ".tmp", "rendered-export-audit", "external", "visual-review-signoff.json"),
);
const sourceTreeClean = gitTreeClean();
const template = readTemplate(templatePath);
const reviewerName = stringArg("reviewer-name", "NEDITOR_RENDERED_EXPORT_REVIEWER_NAME");
const reviewerRole = stringArg("reviewer-role", "NEDITOR_RENDERED_EXPORT_REVIEWER_ROLE", "native-viewer-reviewer");
const reviewerPlatform = stringArg("reviewer-platform", "NEDITOR_RENDERED_EXPORT_REVIEWER_PLATFORM", process.platform);
const nativeViewers = listArg("native-viewer", "NEDITOR_RENDERED_EXPORT_NATIVE_VIEWERS");
const evidenceReference = stringArg("evidence-reference", "NEDITOR_RENDERED_EXPORT_EVIDENCE_REFERENCE");
const notes = stringArg("notes", "NEDITOR_RENDERED_EXPORT_REVIEW_NOTES");

if (!sourceTreeClean) fail("Rendered export signoff must be collected from a clean Git tree. Commit or discard local changes first.");
if (!reviewerName) fail("Reviewer name is required. Pass --reviewer-name or set NEDITOR_RENDERED_EXPORT_REVIEWER_NAME.");
if (!reviewerPlatform) fail("Reviewer platform is required. Pass --reviewer-platform or set NEDITOR_RENDERED_EXPORT_REVIEWER_PLATFORM.");
if (nativeViewers.length === 0) {
  fail("At least one native viewer is required. Pass --native-viewer one or more times or set NEDITOR_RENDERED_EXPORT_NATIVE_VIEWERS.");
}
if (evidenceReference.trim().length < 12) fail("Evidence reference must be substantive and point to reviewer artifacts.");
if (notes.trim().length < 12) fail("Review notes must be substantive.");

const signoff = {
  ...template,
  status: "human-reviewed",
  reviewer: {
    ...template.reviewer,
    name: reviewerName,
    role: reviewerRole,
    reviewedAt: new Date().toISOString(),
    platform: reviewerPlatform,
    nativeViewers,
  },
  primaryArtifacts: template.primaryArtifacts.map((artifact) => reviewedArtifact(artifact, nativeViewers, notes, evidenceReference)),
  reviewCases: template.reviewCases.map((reviewCase) => ({
    ...reviewCase,
    status: "passed",
    reviewerNotes: `${notes} Evidence: ${evidenceReference}`,
    targets: reviewCase.targets.map((artifact) => reviewedArtifact(artifact, nativeViewers, notes, evidenceReference)),
  })),
  checklist: template.checklist.map((item) => ({
    ...item,
    status: "passed",
    reviewerNotes: `${notes} Evidence: ${evidenceReference}`,
  })),
  acceptance: {
    allPrimaryArtifactsReviewed: true,
    allReviewCasesReviewed: true,
    allChecklistItemsReviewed: true,
    blockers: [],
    notes: `${notes} Evidence: ${evidenceReference}`,
  },
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(signoff, null, 2)}\n`);
console.log(`Collected rendered export visual signoff: ${relative(outputPath)}`);
console.log(
  `Validate it with: NEDITOR_RENDERED_EXPORT_SIGNOFF=${relative(outputPath)} pnpm run test:rendered-exports -- --validate-signoff-only`,
);

function reviewedArtifact(artifact, viewers, notes, evidenceReference) {
  return {
    ...artifact,
    status: "passed",
    viewer: viewers.join(", "),
    reviewerNotes: `${notes} Evidence: ${evidenceReference}`,
  };
}

function readTemplate(path) {
  if (!existsSync(path)) {
    fail(`Missing rendered export signoff template: ${relative(path)}. Run pnpm run test:rendered-exports first.`);
  }
  try {
    const value = JSON.parse(readFileSync(path, "utf8"));
    if (value.schema !== "neditor.rendered-export.visual-signoff.v1") {
      fail("Template schema must be neditor.rendered-export.visual-signoff.v1.");
    }
    if (!Array.isArray(value.primaryArtifacts) || !Array.isArray(value.reviewCases) || !Array.isArray(value.checklist)) {
      fail("Template must include primaryArtifacts, reviewCases, and checklist arrays.");
    }
    return value;
  } catch (error) {
    fail(`Could not read rendered export signoff template ${relative(path)}: ${String(error)}`);
  }
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
  console.log(`Collect rendered export native-viewer signoff evidence.

Usage:
  pnpm run collect:rendered-exports:manual -- --reviewer-name "Reviewer Name" --reviewer-platform "macOS 15" --native-viewer "Preview" --native-viewer "Microsoft Word" --evidence-reference ".tmp/rendered-export-review/artifacts" --notes "Reviewed primary and review-case export artifacts with no blockers."

Options:
  --audit-dir <path>             Rendered export audit directory. Defaults to .tmp/rendered-export-audit.
  --template <path>              Generated visual signoff template from pnpm run test:rendered-exports.
  --output <path>                Completed signoff JSON output. Defaults to .tmp/rendered-export-audit/external/visual-review-signoff.json.
  --reviewer-name <name>         Human reviewer name.
  --reviewer-role <role>         Reviewer role. Defaults to native-viewer-reviewer.
  --reviewer-platform <value>    OS/platform observed by the reviewer.
  --native-viewer <value>        Native/browser viewer used for review. Repeat for multiple viewers.
  --evidence-reference <ref>     Artifact folder, ticket, or report reference.
  --notes <text>                 Substantive reviewer notes used across artifacts, cases, and checklist items.

Environment:
  NEDITOR_RENDERED_EXPORT_AUDIT_DIR, NEDITOR_RENDERED_EXPORT_SIGNOFF_TEMPLATE,
  NEDITOR_RENDERED_EXPORT_SIGNOFF_OUTPUT, NEDITOR_RENDERED_EXPORT_REVIEWER_NAME,
  NEDITOR_RENDERED_EXPORT_REVIEWER_ROLE, NEDITOR_RENDERED_EXPORT_REVIEWER_PLATFORM,
  NEDITOR_RENDERED_EXPORT_NATIVE_VIEWERS, NEDITOR_RENDERED_EXPORT_EVIDENCE_REFERENCE,
  NEDITOR_RENDERED_EXPORT_REVIEW_NOTES.

The collector requires a clean Git tree and a real native-viewer review. It does
not perform the review or close the gate without returned reviewer evidence.`);
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

function fail(message) {
  console.error(message);
  process.exit(1);
}
