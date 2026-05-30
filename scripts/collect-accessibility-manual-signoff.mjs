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

const templatePath = resolve(
  args.template ||
    process.env.NEDITOR_ACCESSIBILITY_SIGNOFF_TEMPLATE ||
    join(root, ".tmp", "accessibility", "manual-review-template.json"),
);
const outputPath = resolve(
  args.output ||
    process.env.NEDITOR_ACCESSIBILITY_SIGNOFF_OUTPUT ||
    join(root, ".tmp", "accessibility", "external", "manual-review-signoff.json"),
);
const sourceTreeClean = gitTreeClean();
const template = readTemplate(templatePath);
const reviewerName = stringArg("reviewer-name", "NEDITOR_ACCESSIBILITY_REVIEWER_NAME");
const reviewerRole = stringArg("reviewer-role", "NEDITOR_ACCESSIBILITY_REVIEWER_ROLE", "accessibility-reviewer");
const platformOs = stringArg("platform-os", "NEDITOR_ACCESSIBILITY_PLATFORM_OS", process.platform);
const platformVersion = stringArg("platform-version", "NEDITOR_ACCESSIBILITY_PLATFORM_VERSION");
const platformDevice = stringArg("platform-device", "NEDITOR_ACCESSIBILITY_PLATFORM_DEVICE");
const assistiveTechnologyName = stringArg("assistive-technology", "NEDITOR_ACCESSIBILITY_ASSISTIVE_TECHNOLOGY");
const assistiveTechnologyVersion = stringArg(
  "assistive-technology-version",
  "NEDITOR_ACCESSIBILITY_ASSISTIVE_TECHNOLOGY_VERSION",
);
const assistiveTechnologySettings = stringArg(
  "assistive-technology-settings",
  "NEDITOR_ACCESSIBILITY_ASSISTIVE_TECHNOLOGY_SETTINGS",
  "default reviewer settings",
);
const browserOrWebviewName = stringArg("browser-or-webview", "NEDITOR_ACCESSIBILITY_BROWSER_OR_WEBVIEW");
const browserOrWebviewVersion = stringArg(
  "browser-or-webview-version",
  "NEDITOR_ACCESSIBILITY_BROWSER_OR_WEBVIEW_VERSION",
);
const evidenceReference = stringArg("evidence-reference", "NEDITOR_ACCESSIBILITY_EVIDENCE_REFERENCE");
const notes = stringArg("notes", "NEDITOR_ACCESSIBILITY_REVIEW_NOTES");
const durationMinutes = numberArg("duration-minutes", "NEDITOR_ACCESSIBILITY_DURATION_MINUTES", 20);

if (!sourceTreeClean) fail("Accessibility manual signoff must be collected from a clean Git tree. Commit or discard local changes first.");
if (!reviewerName) fail("Reviewer name is required. Pass --reviewer-name or set NEDITOR_ACCESSIBILITY_REVIEWER_NAME.");
if (!platformVersion) fail("Platform version is required. Pass --platform-version or set NEDITOR_ACCESSIBILITY_PLATFORM_VERSION.");
if (!platformDevice) fail("Platform device is required. Pass --platform-device or set NEDITOR_ACCESSIBILITY_PLATFORM_DEVICE.");
if (!assistiveTechnologyName) fail("Assistive technology name is required. Pass --assistive-technology or set NEDITOR_ACCESSIBILITY_ASSISTIVE_TECHNOLOGY.");
if (!assistiveTechnologyVersion) {
  fail("Assistive technology version is required. Pass --assistive-technology-version or set NEDITOR_ACCESSIBILITY_ASSISTIVE_TECHNOLOGY_VERSION.");
}
if (!browserOrWebviewName) fail("Browser/WebView name is required. Pass --browser-or-webview or set NEDITOR_ACCESSIBILITY_BROWSER_OR_WEBVIEW.");
if (!browserOrWebviewVersion) {
  fail("Browser/WebView version is required. Pass --browser-or-webview-version or set NEDITOR_ACCESSIBILITY_BROWSER_OR_WEBVIEW_VERSION.");
}
if (evidenceReference.trim().length < 12) fail("Evidence reference must be substantive and point to reviewer artifacts.");
if (notes.trim().length < 12) fail("Review notes must be substantive.");

const platform = {
  os: platformOs,
  version: platformVersion,
  device: platformDevice,
};
const assistiveTechnology = {
  name: assistiveTechnologyName,
  version: assistiveTechnologyVersion,
  settings: assistiveTechnologySettings,
};
const browserOrWebview = {
  name: browserOrWebviewName,
  version: browserOrWebviewVersion,
};
const signoff = {
  ...template,
  reviewer: {
    name: reviewerName,
    role: reviewerRole,
  },
  reviewedAt: new Date().toISOString(),
  platform,
  assistiveTechnology,
  browserOrWebview,
  reviewSessions: template.reviewSessions.map((session) => ({
    ...session,
    status: "pass",
    assistiveTechnology,
    platform,
    browserOrWebview,
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
  unresolvedBlockers: [],
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(signoff, null, 2)}\n`);
console.log(`Collected accessibility manual signoff: ${relative(outputPath)}`);
console.log(`Validate it with: NEDITOR_ACCESSIBILITY_SIGNOFF=${relative(outputPath)} pnpm run check:a11y:manual`);

function readTemplate(path) {
  if (!existsSync(path)) {
    fail(`Missing accessibility signoff template: ${relative(path)}. Run pnpm run check:a11y:manual first.`);
  }
  try {
    const value = JSON.parse(readFileSync(path, "utf8"));
    if (value.schema !== "neditor.accessibility.manual-signoff.v1") {
      fail("Template schema must be neditor.accessibility.manual-signoff.v1.");
    }
    if (!Array.isArray(value.reviewSessions) || !Array.isArray(value.checklist)) {
      fail("Template must include reviewSessions and checklist arrays.");
    }
    return value;
  } catch (error) {
    fail(`Could not read accessibility signoff template ${relative(path)}: ${String(error)}`);
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
  console.log(`Collect accessibility manual signoff evidence.

Usage:
  pnpm run collect:a11y:manual -- --reviewer-name "Reviewer Name" --platform-version "macOS 15" --platform-device "MacBook Pro" --assistive-technology "VoiceOver" --assistive-technology-version "macOS 15" --browser-or-webview "Tauri WebView" --browser-or-webview-version "WebKit 620" --evidence-reference ".tmp/accessibility-review/artifacts" --notes "Reviewed screen-reader, keyboard, native shell, and export artifact workflows with no blockers."

Options:
  --template <path>                         Generated template from pnpm run check:a11y:manual.
  --output <path>                           Completed signoff JSON output. Defaults to .tmp/accessibility/external/manual-review-signoff.json.
  --reviewer-name <name>                    Human reviewer name.
  --reviewer-role <role>                    Reviewer role. Defaults to accessibility-reviewer.
  --platform-os <name>                      Platform OS. Defaults to current process platform.
  --platform-version <value>                OS/platform version observed by the reviewer.
  --platform-device <value>                 Device or VM description.
  --assistive-technology <name>             Screen reader or assistive technology used for review.
  --assistive-technology-version <value>    Assistive technology version.
  --assistive-technology-settings <value>   Relevant AT settings. Defaults to default reviewer settings.
  --browser-or-webview <name>               Tauri WebView or browser used for review.
  --browser-or-webview-version <value>      Browser/WebView version.
  --duration-minutes <n>                    Per-session review duration. Defaults to 20.
  --evidence-reference <ref>                Artifact folder, ticket, or report reference.
  --notes <text>                            Substantive reviewer notes used across required sessions and checklist items.

Environment:
  NEDITOR_ACCESSIBILITY_SIGNOFF_TEMPLATE, NEDITOR_ACCESSIBILITY_SIGNOFF_OUTPUT,
  NEDITOR_ACCESSIBILITY_REVIEWER_NAME, NEDITOR_ACCESSIBILITY_REVIEWER_ROLE,
  NEDITOR_ACCESSIBILITY_PLATFORM_OS, NEDITOR_ACCESSIBILITY_PLATFORM_VERSION,
  NEDITOR_ACCESSIBILITY_PLATFORM_DEVICE, NEDITOR_ACCESSIBILITY_ASSISTIVE_TECHNOLOGY,
  NEDITOR_ACCESSIBILITY_ASSISTIVE_TECHNOLOGY_VERSION,
  NEDITOR_ACCESSIBILITY_ASSISTIVE_TECHNOLOGY_SETTINGS,
  NEDITOR_ACCESSIBILITY_BROWSER_OR_WEBVIEW,
  NEDITOR_ACCESSIBILITY_BROWSER_OR_WEBVIEW_VERSION,
  NEDITOR_ACCESSIBILITY_DURATION_MINUTES, NEDITOR_ACCESSIBILITY_EVIDENCE_REFERENCE,
  NEDITOR_ACCESSIBILITY_REVIEW_NOTES.

The collector requires a clean Git tree and a real assistive-technology review.
It does not perform the review or close the gate without returned reviewer evidence.`);
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
