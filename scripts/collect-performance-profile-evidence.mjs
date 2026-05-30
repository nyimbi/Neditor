import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { cpus, totalmem } from "node:os";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = parseArgs(process.argv.slice(2));
const metricsTemplatePath = resolve(
  args["metrics-template-output"] ||
    process.env.NEDITOR_PERFORMANCE_PROFILE_METRICS_TEMPLATE ||
    join(root, ".tmp", "performance-profile", "templates", "native-profile-metrics.template.json"),
);

if (args.help || args.h) {
  printHelp();
  process.exit(0);
}

if (args["write-template"]) {
  writeMetricsTemplate(metricsTemplatePath);
  console.log(`Wrote performance metrics template: ${relative(metricsTemplatePath)}`);
  process.exit(0);
}

const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputPath = resolve(args.output || process.env.NEDITOR_PERFORMANCE_PROFILE_OUTPUT || join(root, ".tmp", "performance-profile", "external", "native-profile.json"));
const metricsPath = resolveRequiredPath(
  args.metrics || process.env.NEDITOR_PERFORMANCE_PROFILE_METRICS,
  "--metrics or NEDITOR_PERFORMANCE_PROFILE_METRICS",
);
const summaryArtifactPath = resolveRequiredPath(
  args["summary-artifact"] || process.env.NEDITOR_PERFORMANCE_PROFILE_SUMMARY,
  "--summary-artifact or NEDITOR_PERFORMANCE_PROFILE_SUMMARY",
);
const traceArtifactPath = args["trace-artifact"] || process.env.NEDITOR_PERFORMANCE_PROFILE_TRACE
  ? resolve(args["trace-artifact"] || process.env.NEDITOR_PERFORMANCE_PROFILE_TRACE)
  : null;
const binaryPath = resolve(args.binary || process.env.NEDITOR_PERFORMANCE_PROFILE_BINARY || join(root, "src-tauri", "target", "release", process.platform === "win32" ? "neditor.exe" : "neditor"));
const sourceCommit = String(args["source-commit"] || process.env.NEDITOR_SOURCE_COMMIT || gitCommit()).trim();
const sourceTreeClean = gitTreeClean();
const reviewerName = String(args["reviewer-name"] || process.env.NEDITOR_PERFORMANCE_PROFILE_REVIEWER || "").trim();
const reviewerRole = String(args["reviewer-role"] || process.env.NEDITOR_PERFORMANCE_PROFILE_REVIEWER_ROLE || "release-performance-reviewer").trim();
const metrics = readMetrics(metricsPath);

writeMetricsTemplate(metricsTemplatePath);
validateMetricsShape(metrics);

if (!sourceCommit) fail("Source commit is required. Run from a Git checkout or pass --source-commit / NEDITOR_SOURCE_COMMIT.");
if (!sourceTreeClean) fail("Performance profile evidence must be collected from a clean Git tree. Commit or discard local changes first.");
if (!reviewerName) fail("Reviewer name is required. Pass --reviewer-name or set NEDITOR_PERFORMANCE_PROFILE_REVIEWER.");
if (!existsSync(binaryPath) || !statSync(binaryPath).isFile()) fail(`Release binary is missing: ${relative(binaryPath)}`);
if (!existsSync(summaryArtifactPath) || !statSync(summaryArtifactPath).isFile()) fail(`Profiler summary artifact is missing: ${relative(summaryArtifactPath)}`);
if (traceArtifactPath && (!existsSync(traceArtifactPath) || !statSync(traceArtifactPath).isFile())) fail(`Profiler trace artifact is missing: ${relative(traceArtifactPath)}`);

const binaryStats = statSync(binaryPath);
const evidence = {
  schema: "neditor.performance-profile-evidence.v1",
  generatedAt: new Date().toISOString(),
  status: "passed",
  appVersion: packageJson.version,
  sourceCommit,
  sourceTreeClean,
  platform: String(args.platform || metrics.platform || process.platform),
  arch: String(args.arch || metrics.arch || process.arch),
  runtime: "tauri-release",
  deviceClass: String(args["device-class"] || metrics.deviceClass || "release-laptop-or-workstation"),
  durationMinutes: numberValue(args["duration-minutes"] ?? metrics.durationMinutes, "durationMinutes"),
  environment: {
    osVersion: String(args["os-version"] || metrics.environment?.osVersion || hostOsVersion()),
    cpu: String(args.cpu || metrics.environment?.cpu || cpus()[0]?.model || "unknown-cpu"),
    memoryGb: numberValue(args["memory-gb"] ?? metrics.environment?.memoryGb ?? Math.round(totalmem() / 1024 / 1024 / 1024), "environment.memoryGb"),
    powerMode: String(args["power-mode"] || metrics.environment?.powerMode || "unknown"),
  },
  binary: {
    target: "release",
    appVersion: packageJson.version,
    path: relative(binaryPath),
    bytes: binaryStats.size,
    sha256: sha256File(binaryPath),
  },
  scenarios: requiredArray(metrics.scenarios, "metrics.scenarios"),
  soak: requiredObject(metrics.soak, "metrics.soak"),
  exports: requiredObject(metrics.exports, "metrics.exports"),
  artifacts: {
    summaryPath: relative(summaryArtifactPath),
    summarySha256: sha256File(summaryArtifactPath),
    ...(traceArtifactPath
      ? {
          tracePath: relative(traceArtifactPath),
          traceSha256: sha256File(traceArtifactPath),
        }
      : {}),
    notes: String(metrics.artifacts?.notes || "Collected by scripts/collect-performance-profile-evidence.mjs."),
  },
  reviewer: {
    name: reviewerName,
    role: reviewerRole,
    reviewedAt: new Date().toISOString(),
  },
};

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(`Collected release-device performance profile evidence: ${relative(outputPath)}`);
console.log("Validate it with: pnpm run check:performance-profile");

function readMetrics(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`Could not read performance metrics JSON ${relative(path)}: ${String(error)}`);
  }
}

function resolveRequiredPath(value, label) {
  if (!value) {
    writeMetricsTemplate(metricsTemplatePath);
    fail(`Missing required ${label}. A metrics template was written to ${relative(metricsTemplatePath)}.`);
  }
  return resolve(String(value));
}

function requiredArray(value, label) {
  if (!Array.isArray(value)) fail(`${label} must be an array in the metrics JSON.`);
  return value;
}

function requiredObject(value, label) {
  if (!value || typeof value !== "object" || Array.isArray(value)) fail(`${label} must be an object in the metrics JSON.`);
  return value;
}

function numberValue(value, label) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) fail(`${label} must be numeric.`);
  return parsed;
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
    parsed[key] = value;
    if (value !== true) index += 1;
  }
  return parsed;
}

function validateMetricsShape(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) fail("Metrics JSON must be an object.");
  numberValue(value.durationMinutes, "metrics.durationMinutes");
  requiredArray(value.scenarios, "metrics.scenarios");
  requiredObject(value.soak, "metrics.soak");
  requiredObject(value.exports, "metrics.exports");
}

function writeMetricsTemplate(path) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(
    path,
    `${JSON.stringify(
      {
        schema: "neditor.performance-profile-metrics.v1",
        durationMinutes: 45,
        platform: process.platform,
        arch: process.arch,
        deviceClass: "release-laptop-or-workstation",
        environment: {
          osVersion: "replace-with-profiled-device-os-version",
          cpu: "replace-with-profiled-device-cpu-model",
          memoryGb: 16,
          powerMode: "plugged-in",
        },
        scenarios: [
          "startup-open-document",
          "large-document-edit-preview",
          "export-suite",
          "native-file-watch-conflict",
          "agent-workflow-review",
        ].map((id) => ({
          id,
          status: "passed",
          samples: 5,
          p95InteractionMs: 250,
          maxInteractionMs: 750,
          peakRssMb: 900,
          memoryGrowthMb: 80,
          notes: "Replace with measurement method, profiler name, and observed behavior.",
        })),
        soak: {
          durationMinutes: 45,
          crashes: 0,
          hangs: 0,
          errorDialogs: 0,
          memoryGrowthMb: 80,
        },
        exports: {
          completed: 5,
          failed: 0,
          targets: ["html", "pdf", "docx", "pptx", "markdown-bundle"],
          p95Ms: 3000,
        },
        artifacts: {
          notes: "Summarize profiler artifact paths and measurement context. The collector records hashes from --summary-artifact and optional --trace-artifact.",
        },
      },
      null,
      2,
    )}\n`,
  );
}

function printHelp() {
  console.log(`Collect release-device performance evidence.

Usage:
  pnpm run collect:performance-profile -- --metrics /path/to/metrics.json --summary-artifact /path/to/summary.txt --reviewer-name "Reviewer Name"

Options:
  --metrics <path>             JSON metrics captured during a real 30+ minute release-device profile.
  --summary-artifact <path>    Profiler summary file to hash into the evidence.
  --trace-artifact <path>      Optional profiler trace file to hash into the evidence.
  --binary <path>              Profiled release binary. Defaults to src-tauri/target/release/neditor.
  --output <path>              Evidence JSON output. Defaults to .tmp/performance-profile/external/native-profile.json.
  --reviewer-name <name>       Required named reviewer/operator for the release-device session.
  --reviewer-role <role>       Reviewer role. Defaults to release-performance-reviewer.
  --write-template             Write a metrics template and exit without collecting evidence.

Environment:
  NEDITOR_PERFORMANCE_PROFILE_METRICS, NEDITOR_PERFORMANCE_PROFILE_SUMMARY,
  NEDITOR_PERFORMANCE_PROFILE_TRACE, NEDITOR_PERFORMANCE_PROFILE_BINARY,
  NEDITOR_PERFORMANCE_PROFILE_OUTPUT, NEDITOR_PERFORMANCE_PROFILE_REVIEWER,
  NEDITOR_PERFORMANCE_PROFILE_REVIEWER_ROLE.

The collector requires a clean Git tree and does not generate synthetic evidence.
Validate returned evidence with pnpm run check:performance-profile.`);
}

function hostOsVersion() {
  if (process.platform === "darwin") {
    const result = spawnSync("sw_vers", ["-productVersion"], { encoding: "utf8" });
    if (result.status === 0 && result.stdout.trim()) return `macOS ${result.stdout.trim()}`;
  }
  const result = spawnSync("uname", ["-a"], { encoding: "utf8" });
  return result.status === 0 && result.stdout.trim() ? result.stdout.trim() : process.platform;
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
