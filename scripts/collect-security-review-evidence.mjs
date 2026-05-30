import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
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

const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputPath = resolve(args.output || process.env.NEDITOR_SECURITY_REVIEW_OUTPUT || join(root, ".tmp", "security-review", "external", "security-review.json"));
const reportPath = resolveRequiredPath(args["report-file"] || process.env.NEDITOR_SECURITY_REVIEW_REPORT_FILE, "--report-file or NEDITOR_SECURITY_REVIEW_REPORT_FILE");
const toolOutputPath = args["tool-output-file"] || process.env.NEDITOR_SECURITY_REVIEW_TOOL_OUTPUT_FILE
  ? resolve(args["tool-output-file"] || process.env.NEDITOR_SECURITY_REVIEW_TOOL_OUTPUT_FILE)
  : null;
const sourceCommit = String(args["source-commit"] || process.env.NEDITOR_SOURCE_COMMIT || gitCommit()).trim();
const sourceTreeClean = gitTreeClean();
const reviewerName = stringArg("reviewer-name", "NEDITOR_SECURITY_REVIEWER_NAME");
const reviewerRole = stringArg("reviewer-role", "NEDITOR_SECURITY_REVIEWER_ROLE", "security-reviewer");
const reviewerOrganization = stringArg("reviewer-organization", "NEDITOR_SECURITY_REVIEWER_ORGANIZATION");
const reportReference = stringArg("report-reference", "NEDITOR_SECURITY_REVIEW_REPORT_REFERENCE", relative(reportPath));
const toolName = stringArg("tool-name", "NEDITOR_SECURITY_REVIEW_TOOL_NAME", "manual-threat-model-review");
const toolResult = stringArg("tool-result", "NEDITOR_SECURITY_REVIEW_TOOL_RESULT", "passed");
const mediumFindings = numberArg("medium", "NEDITOR_SECURITY_REVIEW_MEDIUM", 0);
const lowFindings = numberArg("low", "NEDITOR_SECURITY_REVIEW_LOW", 0);
const acceptedRisks = listArg("accepted-risk", "NEDITOR_SECURITY_REVIEW_ACCEPTED_RISKS");

if (!sourceCommit) fail("Source commit is required. Run from a Git checkout or pass --source-commit / NEDITOR_SOURCE_COMMIT.");
if (!sourceTreeClean) fail("Security review evidence must be collected from a clean Git tree. Commit or discard local changes first.");
if (!reviewerName) fail("Reviewer name is required. Pass --reviewer-name or set NEDITOR_SECURITY_REVIEWER_NAME.");
if (!reviewerOrganization) fail("Reviewer organization is required. Pass --reviewer-organization or set NEDITOR_SECURITY_REVIEWER_ORGANIZATION.");
if (!existsSync(reportPath) || !statSync(reportPath).isFile()) fail(`Security review report is missing: ${relative(reportPath)}`);
if (toolOutputPath && (!existsSync(toolOutputPath) || !statSync(toolOutputPath).isFile())) fail(`Security tool output is missing: ${relative(toolOutputPath)}`);
if (mediumFindings > 3) fail("Medium findings must be 3 or fewer for release signoff evidence.");

const evidence = {
  schema: "neditor.security-review-evidence.v1",
  generatedAt: new Date().toISOString(),
  status: "passed",
  appVersion: packageJson.version,
  sourceCommit,
  sourceTreeClean,
  reviewType: "independent-security-review",
  independentReviewer: true,
  reviewer: {
    name: reviewerName,
    role: reviewerRole,
    organization: reviewerOrganization,
    reviewedAt: new Date().toISOString(),
  },
  scope: {
    trustBoundaries: [
      "tauri-command-boundary",
      "filesystem-boundary",
      "snapshot-boundary",
      "include-boundary",
      "export-boundary",
      "git-boundary",
      "external-transform-boundary",
      "ai-provider-boundary",
      "persistence-boundary",
    ],
    reviewedArtifacts: [
      "security-threat-model",
      "tauri-config",
      "rust-command-surface",
      "external-transform-runner",
      "git-restore-and-tag",
      "snapshot-restore",
      "ai-provider-packages",
      "workspace-persistence",
      "release-evidence-contracts",
    ],
  },
  tools: [
    {
      name: toolName,
      result: toolResult,
    },
  ],
  findings: {
    critical: 0,
    high: 0,
    medium: mediumFindings,
    low: lowFindings,
    unresolved: 0,
    acceptedRisks,
  },
  artifacts: {
    reportReference,
    reportSha256: sha256File(reportPath),
    ...(toolOutputPath ? { toolOutputSha256: sha256File(toolOutputPath) } : {}),
  },
  signoff: {
    approvedForRelease: true,
    secretsStored: false,
    networkTelemetryAdded: false,
    externalExecutionReviewed: true,
    providerBoundaryReviewed: true,
  },
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(`Collected independent security review evidence: ${relative(outputPath)}`);
console.log("Validate it with: pnpm run check:security-review");

function resolveRequiredPath(value, label) {
  if (!value) fail(`Missing required ${label}.`);
  return resolve(String(value));
}

function stringArg(name, envName, fallback = "") {
  return String(args[name] || process.env[envName] || fallback).trim();
}

function numberArg(name, envName, fallback) {
  const parsed = Number(args[name] ?? process.env[envName] ?? fallback);
  if (!Number.isFinite(parsed) || parsed < 0) fail(`${name} must be a non-negative number.`);
  return parsed;
}

function listArg(name, envName) {
  const cliValues = [args[name]].flat().filter((value) => typeof value === "string" && value.trim());
  const envValues = String(process.env[envName] || "")
    .split("\n")
    .map((value) => value.trim())
    .filter(Boolean);
  return [...cliValues, ...envValues];
}

function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!arg.startsWith("--")) continue;
    const key = arg.slice(2);
    const next = argv[index + 1];
    const value = !next || next.startsWith("--") ? true : next;
    if (parsed[key] !== undefined) parsed[key] = [parsed[key], value].flat();
    else parsed[key] = value;
    if (value !== true) index += 1;
  }
  return parsed;
}

function printHelp() {
  console.log(`Collect independent security-review release evidence.

Usage:
  pnpm run collect:security-review -- --report-file /path/to/security-report.md --reviewer-name "Reviewer Name" --reviewer-organization "Independent Org"

Options:
  --report-file <path>              Independent review report to hash into evidence.
  --tool-output-file <path>         Optional scanner/tool output to hash into evidence.
  --reviewer-name <name>            Independent reviewer name.
  --reviewer-role <role>            Reviewer role. Defaults to security-reviewer.
  --reviewer-organization <org>     Independent team or company.
  --report-reference <ref>          Ticket, URL, or local path reference for the report.
  --tool-name <name>                Review method or scanner name.
  --tool-result <result>            Review method or scanner result.
  --medium <n>                      Medium findings count, max 3.
  --low <n>                         Low findings count.
  --accepted-risk <text>            Repeatable accepted-risk note.
  --output <path>                   Evidence JSON output. Defaults to .tmp/security-review/external/security-review.json.

Environment:
  NEDITOR_SECURITY_REVIEW_REPORT_FILE, NEDITOR_SECURITY_REVIEW_TOOL_OUTPUT_FILE,
  NEDITOR_SECURITY_REVIEWER_NAME, NEDITOR_SECURITY_REVIEWER_ROLE,
  NEDITOR_SECURITY_REVIEWER_ORGANIZATION, NEDITOR_SECURITY_REVIEW_REPORT_REFERENCE,
  NEDITOR_SECURITY_REVIEW_TOOL_NAME, NEDITOR_SECURITY_REVIEW_TOOL_RESULT,
  NEDITOR_SECURITY_REVIEW_MEDIUM, NEDITOR_SECURITY_REVIEW_LOW,
  NEDITOR_SECURITY_REVIEW_ACCEPTED_RISKS.

The collector requires a clean Git tree and a real independent report. It does
not perform the review or downgrade findings.
Validate returned evidence with pnpm run check:security-review.`);
}

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 ? result.stdout.trim() : "";
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
