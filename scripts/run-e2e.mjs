import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { resolvePlaywrightBrowserEnv, writePlaywrightBrowserReport } from "./playwright-browser-env.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const binary = join(
  root,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "playwright.cmd" : "playwright",
);
const args = ["test", ...process.argv.slice(2)];
const reportPath = join(root, ".tmp", "e2e-browser", "report.json");
const browserResolution = resolvePlaywrightBrowserEnv(process.env);

if (!browserResolution.ok) {
  writePlaywrightBrowserReport(reportPath, browserResolution, "blocked");
  console.error(browserResolution.message);
  process.exit(1);
}

console.error(browserResolution.message);
const result = spawnSync(binary, args, {
  cwd: root,
  env: browserResolution.env,
  shell: process.platform === "win32",
  stdio: "pipe",
  encoding: "utf8",
});

if (result.stdout) process.stdout.write(result.stdout);
if (result.stderr) process.stderr.write(result.stderr);

writePlaywrightBrowserReport(reportPath, browserResolution, result.status === 0 ? "passed" : "failed", {
  schema: "neditor.e2e-browser-workflow.v1",
  command: [binary, ...args],
  exitStatus: result.status,
  signal: result.signal,
  error: result.error ? String(result.error) : undefined,
  summary: summarizePlaywrightOutput(result.stdout || "", result.stderr || ""),
  workflowEvidence: workflowEvidence([binary, ...args], result.stdout || ""),
  stdoutTail: tail(result.stdout || ""),
  stderrTail: tail(result.stderr || ""),
});

process.exit(result.status ?? 1);

function tail(output) {
  return output
    .split(/\r?\n/)
    .filter(Boolean)
    .slice(-80);
}

function stripAnsi(value) {
  return value.replace(/\u001b\[[0-9;]*m/g, "");
}

function summarizePlaywrightOutput(stdout, stderr) {
  const output = stripAnsi(`${stdout}\n${stderr}`);
  const tests = Number(output.match(/Running\s+(\d+)\s+tests?\s+using/i)?.[1] || 0);
  const passed = Number(output.match(/(\d+)\s+passed\b/i)?.[1] || 0);
  const failed = Number(output.match(/(\d+)\s+failed\b/i)?.[1] || 0);
  const flaky = Number(output.match(/(\d+)\s+flaky\b/i)?.[1] || 0);
  const skipped = Number(output.match(/(\d+)\s+skipped\b/i)?.[1] || 0);
  const timedOut = Number(output.match(/(\d+)\s+timed out\b/i)?.[1] || 0);
  return {
    tests,
    passed,
    failed,
    flaky,
    skipped,
    timedOut,
  };
}

function workflowEvidence(command, stdout) {
  const commandText = command.join(" ");
  const output = stripAnsi(stdout);
  return {
    docsLiveDraft:
      /Docs Live/i.test(commandText) ||
      output.includes("generates a Docs Live draft from outline, context, and placeholders"),
    outlineModeCrud:
      /outline/i.test(commandText) ||
      output.includes("edits document structure from outline mode"),
    editableOutlinePlanning:
      /outline/i.test(commandText) ||
      output.includes("creates a document skeleton from an editable outline plan"),
    exportWorkflows:
      /export/i.test(commandText) ||
      output.includes("runs export readiness, success, and failure workflows"),
  };
}
