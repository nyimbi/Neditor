import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = parseArgs(process.argv.slice(2));
const readinessTemplatePath = resolve(
  args["readiness-template-output"] ||
    process.env.NEDITOR_AI_RUNTIME_READINESS_TEMPLATE ||
    join(root, ".tmp", "ai-runtime-evidence", "templates", "runtime-readiness.template.json"),
);

if (args.help || args.h) {
  printHelp();
  process.exit(0);
}

if (args["write-template"]) {
  writeReadinessTemplate(readinessTemplatePath);
  console.log(`Wrote AI runtime readiness template: ${relative(readinessTemplatePath)}`);
  process.exit(0);
}

const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputPath = resolve(args.output || process.env.NEDITOR_AI_RUNTIME_OUTPUT || join(root, ".tmp", "ai-runtime-evidence", "external", "runtime-evidence.json"));
const readinessPath = resolveRequiredPath(
  args["readiness-json"] || process.env.NEDITOR_AI_RUNTIME_READINESS_JSON,
  "--readiness-json or NEDITOR_AI_RUNTIME_READINESS_JSON",
);
const sourceCommit = String(args["source-commit"] || process.env.NEDITOR_SOURCE_COMMIT || gitCommit()).trim();
const sourceTreeClean = gitTreeClean();
const readiness = readReadiness(readinessPath);
const microphoneResult = String(args["microphone-result"] || process.env.NEDITOR_AI_RUNTIME_MICROPHONE_RESULT || "").trim();
const reviewerNotes = String(args.notes || process.env.NEDITOR_AI_RUNTIME_REVIEW_NOTES || "Collected from a real Docs Live runtime readiness session.").trim();
const clipboardKind = String(args["clipboard-kind"] || process.env.NEDITOR_AI_RUNTIME_CLIPBOARD_KIND || inferClipboardKind(readiness) || "").trim();
const clipboardCharacters = numberValue(
  args["clipboard-characters"] || process.env.NEDITOR_AI_RUNTIME_CLIPBOARD_CHARACTERS || inferClipboardCharacters(readiness),
  "clipboard characters",
);
const clipboardWriteSucceeded = booleanValue(args["clipboard-write-succeeded"] ?? process.env.NEDITOR_AI_RUNTIME_CLIPBOARD_WRITE_SUCCEEDED, "clipboard write succeeded");

writeReadinessTemplate(readinessTemplatePath);
validateReadinessShape(readiness);

if (!sourceCommit) fail("Source commit is required. Run from a Git checkout or pass --source-commit / NEDITOR_SOURCE_COMMIT.");
if (!sourceTreeClean) fail("AI runtime evidence must be collected from a clean Git tree. Commit or discard local changes first.");
if (!["permission-granted", "stream-opened"].includes(microphoneResult)) {
  fail("Pass --microphone-result permission-granted or --microphone-result stream-opened after a real device/browser session.");
}
if (!["plain", "rich"].includes(clipboardKind)) fail("Pass --clipboard-kind plain or rich.");
if (clipboardCharacters <= 0) fail("Clipboard characters must be greater than zero without storing clipboard content.");

const evidence = {
  schema: "neditor.ai-runtime-evidence.v1",
  generatedAt: new Date().toISOString(),
  status: "passed",
  appVersion: packageJson.version,
  sourceCommit,
  sourceTreeClean,
  platform: String(args.platform || readiness.platform || process.platform),
  arch: String(args.arch || readiness.arch || process.arch),
  runtime: String(args.runtime || readiness.runtime || "tauri-webview"),
  secureContext: Boolean(readiness.secureContext),
  speechRecognition: {
    supported: readiness.speechRecognition.supported === true,
    state: String(readiness.speechRecognition.state || ""),
  },
  microphonePermission: {
    state: String(readiness.microphonePermission.state || ""),
  },
  microphoneProbe: {
    attempted: true,
    result: microphoneResult,
    audioStored: false,
    notes: reviewerNotes,
  },
  clipboardRead: {
    supported: readiness.clipboardRead.supported === true,
    state: String(readiness.clipboardRead.state || ""),
    kind: clipboardKind,
    charactersDetected: clipboardCharacters,
    contentStored: false,
  },
  clipboardWrite: {
    supported: readiness.clipboardWrite.supported === true,
    writeSucceeded: clipboardWriteSucceeded,
    contentStored: false,
  },
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(`Collected AI runtime evidence: ${relative(outputPath)}`);
console.log("Validate it with: pnpm run check:ai-runtime");

function readReadiness(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`Could not read AI runtime readiness JSON ${relative(path)}: ${String(error)}`);
  }
}

function resolveRequiredPath(value, label) {
  if (!value) {
    writeReadinessTemplate(readinessTemplatePath);
    fail(`Missing required ${label}. A readiness template was written to ${relative(readinessTemplatePath)}.`);
  }
  return resolve(String(value));
}

function validateReadinessShape(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) fail("Readiness JSON must be an object.");
  requireCapability(value.speechRecognition, "speechRecognition");
  requireCapability(value.microphonePermission, "microphonePermission");
  requireCapability(value.clipboardRead, "clipboardRead");
  requireCapability(value.clipboardWrite, "clipboardWrite");
  if (typeof value.secureContext !== "boolean") fail("readiness.secureContext must be a boolean.");
}

function requireCapability(value, label) {
  if (!value || typeof value !== "object" || Array.isArray(value)) fail(`readiness.${label} must be an object.`);
  if (typeof value.supported !== "boolean") fail(`readiness.${label}.supported must be a boolean.`);
  if (typeof value.state !== "string") fail(`readiness.${label}.state must be supplied.`);
}

function inferClipboardKind(readiness) {
  const detail = String(readiness?.clipboardRead?.detail || "");
  if (/\brich\b/i.test(detail)) return "rich";
  if (/\bplain\b/i.test(detail)) return "plain";
  return "";
}

function inferClipboardCharacters(readiness) {
  const detail = String(readiness?.clipboardRead?.detail || "");
  return Number(detail.match(/(\d+)\s+characters?/i)?.[1] || 0);
}

function numberValue(value, label) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) fail(`${label} must be numeric.`);
  return parsed;
}

function booleanValue(value, label) {
  if (value === true || value === "true" || value === "1" || value === "yes") return true;
  if (value === false || value === "false" || value === "0" || value === "no") return false;
  fail(`Pass ${label} as true/false.`);
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

function writeReadinessTemplate(path) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(
    path,
    `${JSON.stringify(
      {
        schema: "neditor.ai-runtime-readiness.v1",
        generatedAt: new Date().toISOString(),
        platform: process.platform,
        arch: process.arch,
        runtime: "tauri-webview",
        secureContext: true,
        speechRecognition: {
          supported: true,
          state: "available",
          detail: "SpeechRecognition API is available.",
        },
        microphonePermission: {
          supported: true,
          state: "granted",
          detail: "microphone permission is granted.",
        },
        clipboardRead: {
          supported: true,
          state: "granted",
          detail: "Clipboard rich read succeeded (30 characters detected, content not stored).",
        },
        clipboardWrite: {
          supported: true,
          state: "granted",
          detail: "clipboard-write permission is granted.",
        },
        issues: [],
      },
      null,
      2,
    )}\n`,
  );
}

function printHelp() {
  console.log(`Collect real-device AI runtime evidence.

Usage:
  pnpm run collect:ai-runtime -- --readiness-json /path/to/readiness.json --microphone-result stream-opened --clipboard-write-succeeded true

Options:
  --readiness-json <path>         JSON from the Docs Live AI runtime readiness check.
  --microphone-result <value>     permission-granted or stream-opened from the real runtime session.
  --clipboard-kind <value>        plain or rich. Inferred from readiness detail when possible.
  --clipboard-characters <n>      Positive character count detected during read; content is not stored.
  --clipboard-write-succeeded     true only after write proof in the real runtime session.
  --runtime <value>               tauri-webview or browser. Defaults to readiness value or tauri-webview.
  --output <path>                 Evidence JSON output. Defaults to .tmp/ai-runtime-evidence/external/runtime-evidence.json.
  --write-template                Write a readiness JSON template and exit without collecting evidence.

Environment:
  NEDITOR_AI_RUNTIME_READINESS_JSON, NEDITOR_AI_RUNTIME_OUTPUT,
  NEDITOR_AI_RUNTIME_MICROPHONE_RESULT, NEDITOR_AI_RUNTIME_CLIPBOARD_KIND,
  NEDITOR_AI_RUNTIME_CLIPBOARD_CHARACTERS,
  NEDITOR_AI_RUNTIME_CLIPBOARD_WRITE_SUCCEEDED.

The collector requires a clean Git tree and never stores audio or clipboard content.
Validate returned evidence with pnpm run check:ai-runtime.`);
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
