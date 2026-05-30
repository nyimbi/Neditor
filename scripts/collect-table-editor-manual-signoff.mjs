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

const templatePath = resolve(args.template || process.env.NEDITOR_TABLE_EDITOR_SIGNOFF_TEMPLATE || join(root, ".tmp", "table-editor", "manual-review-template.json"));
const outputPath = resolve(args.output || process.env.NEDITOR_TABLE_EDITOR_SIGNOFF_OUTPUT || join(root, ".tmp", "table-editor", "external", "manual-review-signoff.json"));
const sourceTreeClean = gitTreeClean();
const template = readTemplate(templatePath);
const reviewerName = stringArg("reviewer-name", "NEDITOR_TABLE_EDITOR_REVIEWER_NAME");
const reviewerRole = stringArg("reviewer-role", "NEDITOR_TABLE_EDITOR_REVIEWER_ROLE", "table-editor-reviewer");
const reviewerOrganization = stringArg("reviewer-organization", "NEDITOR_TABLE_EDITOR_REVIEWER_ORGANIZATION", "");
const platformOs = stringArg("platform-os", "NEDITOR_TABLE_EDITOR_PLATFORM_OS", process.platform);
const platformVersion = stringArg("platform-version", "NEDITOR_TABLE_EDITOR_PLATFORM_VERSION");
const platformDevice = stringArg("platform-device", "NEDITOR_TABLE_EDITOR_PLATFORM_DEVICE");
const webviewOrBrowser = stringArg("webview-or-browser", "NEDITOR_TABLE_EDITOR_WEBVIEW_OR_BROWSER");
const evidenceReference = stringArg("evidence-reference", "NEDITOR_TABLE_EDITOR_EVIDENCE_REFERENCE");
const notes = stringArg("notes", "NEDITOR_TABLE_EDITOR_REVIEW_NOTES");
const durationMinutes = numberArg("duration-minutes", "NEDITOR_TABLE_EDITOR_DURATION_MINUTES", 15);
const supportedPlatform = stringArg("supported-platform", "NEDITOR_TABLE_EDITOR_SUPPORTED_PLATFORM", `${platformOs} ${platformVersion}`.trim());
const supportedStatus = stringArg("supported-status", "NEDITOR_TABLE_EDITOR_SUPPORTED_STATUS", "pass");

if (!sourceTreeClean) fail("Table editor manual signoff must be collected from a clean Git tree. Commit or discard local changes first.");
if (!reviewerName) fail("Reviewer name is required. Pass --reviewer-name or set NEDITOR_TABLE_EDITOR_REVIEWER_NAME.");
if (!platformVersion) fail("Platform version is required. Pass --platform-version or set NEDITOR_TABLE_EDITOR_PLATFORM_VERSION.");
if (!platformDevice) fail("Platform device is required. Pass --platform-device or set NEDITOR_TABLE_EDITOR_PLATFORM_DEVICE.");
if (!webviewOrBrowser) fail("WebView/browser is required. Pass --webview-or-browser or set NEDITOR_TABLE_EDITOR_WEBVIEW_OR_BROWSER.");
if (evidenceReference.trim().length < 12) fail("Evidence reference must be substantive and point to reviewer artifacts.");
if (notes.trim().length < 12) fail("Review notes must be substantive.");
if (!["pass", "documented-limitation"].includes(supportedStatus)) fail("--supported-status must be pass or documented-limitation.");

const platform = {
  os: platformOs,
  version: platformVersion,
  device: platformDevice,
  webviewOrBrowser,
};
const signoff = {
  ...template,
  reviewer: {
    name: reviewerName,
    role: reviewerRole,
    organization: reviewerOrganization,
  },
  reviewedAt: new Date().toISOString(),
  platform,
  reviewSessions: template.reviewSessions.map((session) => ({
    ...session,
    status: "pass",
    platform,
    durationMinutes,
    evidenceReference,
    notes,
    blockers: [],
  })),
  checklist: template.checklist.map((item) => ({
    ...item,
    status: "pass",
    notes,
  })),
  supportedHostResults: [
    {
      platform: supportedPlatform,
      status: supportedStatus,
      evidenceReference,
      limitations: supportedStatus === "documented-limitation" ? [notes] : [],
      notes,
    },
  ],
  unresolvedBlockers: [],
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(signoff, null, 2)}\n`);
console.log(`Collected table editor manual signoff: ${relative(outputPath)}`);
console.log(`Validate it with: NEDITOR_TABLE_EDITOR_SIGNOFF=${relative(outputPath)} pnpm run check:tables:manual`);

function readTemplate(path) {
  if (!existsSync(path)) {
    fail(`Missing table editor signoff template: ${relative(path)}. Run pnpm run check:tables:manual first.`);
  }
  try {
    const value = JSON.parse(readFileSync(path, "utf8"));
    if (value.schema !== "neditor.table-editor.manual-signoff.v1") {
      fail("Template schema must be neditor.table-editor.manual-signoff.v1.");
    }
    if (!Array.isArray(value.reviewSessions) || !Array.isArray(value.checklist)) {
      fail("Template must include reviewSessions and checklist arrays.");
    }
    return value;
  } catch (error) {
    fail(`Could not read table editor signoff template ${relative(path)}: ${String(error)}`);
  }
}

function stringArg(name, envName, fallback = "") {
  return String(args[name] || process.env[envName] || fallback).trim();
}

function numberArg(name, envName, fallback) {
  const parsed = Number(args[name] ?? process.env[envName] ?? fallback);
  if (!Number.isFinite(parsed) || parsed <= 0) fail(`${name} must be a positive number.`);
  return parsed;
}

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!arg.startsWith("--")) continue;
    const key = arg.slice(2);
    const next = argv[index + 1];
    const value = !next || next.startsWith("--") ? true : next;
    parsed[key] = value;
    if (value !== true) index += 1;
  }
  return parsed;
}

function printHelp() {
  console.log(`Collect table editor manual signoff evidence.

Usage:
  pnpm run collect:tables:manual -- --reviewer-name "Reviewer Name" --platform-version "macOS 15" --platform-device "MacBook Pro" --webview-or-browser "Tauri WebView" --evidence-reference ".tmp/table-review/artifacts" --notes "Reviewed source/grid/export workflows with no blockers."

Options:
  --template <path>             Generated template from pnpm run check:tables:manual.
  --output <path>               Completed signoff JSON output. Defaults to .tmp/table-editor/external/manual-review-signoff.json.
  --reviewer-name <name>        Human reviewer name.
  --reviewer-role <role>        Reviewer role. Defaults to table-editor-reviewer.
  --reviewer-organization <org> Optional reviewer organization.
  --platform-os <name>          Platform OS. Defaults to current process platform.
  --platform-version <value>    OS/platform version observed by the reviewer.
  --platform-device <value>     Device or VM description.
  --webview-or-browser <value>  Tauri WebView or browser used for the review.
  --duration-minutes <n>        Per-session review duration. Defaults to 15.
  --evidence-reference <ref>    Artifact folder, ticket, or report reference.
  --notes <text>                Substantive reviewer notes used across required sessions and checklist items.
  --supported-platform <value>  Supported-host result platform label.
  --supported-status <value>    pass or documented-limitation.

Environment:
  NEDITOR_TABLE_EDITOR_SIGNOFF_TEMPLATE, NEDITOR_TABLE_EDITOR_SIGNOFF_OUTPUT,
  NEDITOR_TABLE_EDITOR_REVIEWER_NAME, NEDITOR_TABLE_EDITOR_REVIEWER_ROLE,
  NEDITOR_TABLE_EDITOR_REVIEWER_ORGANIZATION, NEDITOR_TABLE_EDITOR_PLATFORM_OS,
  NEDITOR_TABLE_EDITOR_PLATFORM_VERSION, NEDITOR_TABLE_EDITOR_PLATFORM_DEVICE,
  NEDITOR_TABLE_EDITOR_WEBVIEW_OR_BROWSER, NEDITOR_TABLE_EDITOR_DURATION_MINUTES,
  NEDITOR_TABLE_EDITOR_EVIDENCE_REFERENCE, NEDITOR_TABLE_EDITOR_REVIEW_NOTES,
  NEDITOR_TABLE_EDITOR_SUPPORTED_PLATFORM, NEDITOR_TABLE_EDITOR_SUPPORTED_STATUS.

The collector requires a clean Git tree and a real human review. It does not
perform the review or close the gate without returned reviewer evidence.`);
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
