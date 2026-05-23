import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { findSystemChromium } from "./playwright-browser-env.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = join(root, ".tmp", "accessibility", "runtime-report.json");
const specPath = join(root, "e2e", "app-workflows.spec.ts");
const expectedWorkflows = [
  "exposes keyboard skip links to primary workbench regions",
  "keeps primary workbench regions accessible across desktop and narrow viewports",
  "manages modal focus and Escape return paths",
  "supports keyboard-only operation for deep workbench controls",
  "exposes status and progress messages as live regions",
  "persists editor settings and runs search plus heading commands",
];
const grep = expectedWorkflows.map(escapeRegExp).join("|");
const command = [
  process.execPath,
  "scripts/run-e2e.mjs",
  "e2e/app-workflows.spec.ts",
  "--grep",
  grep,
  "--project",
  "chromium",
];

const issues = [];
const specSource = readFileSync(specPath, "utf8");
for (const workflow of expectedWorkflows) {
  if (!specSource.includes(`test("${workflow}"`)) {
    issues.push(`missing runtime accessibility workflow: ${workflow}`);
  }
}

if (issues.length === 0) {
  const attempts = [];
  let result = runWorkflowAttempt("default", process.env, attempts);
  if (result.status !== 0 && shouldRetryWithSystemBrowser(result)) {
    const fallbackPath = findSystemChromium();
    if (fallbackPath) {
      console.error(`Runtime accessibility audit hit a transient bundled Chromium launch failure; retrying with ${fallbackPath}.`);
      result = runWorkflowAttempt(
        "system-chromium-fallback",
        {
          ...process.env,
          PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH: fallbackPath,
        },
        attempts,
      );
    }
  }
  if (result.status !== 0) issues.push(`runtime accessibility workflow command exited with ${result.status ?? 1}`);
  writeReport(result, issues, attempts);
  if (issues.length > 0) process.exit(result.status ?? 1);
} else {
  writeReport(null, issues, []);
  for (const issue of issues) console.error(issue);
  process.exit(1);
}

console.log(`Checked runtime accessibility workflows; wrote ${reportPath}.`);

function runWorkflowAttempt(label, env, attempts) {
  const result = spawnSync(command[0], command.slice(1), {
    cwd: root,
    encoding: "utf8",
    env,
  });
  process.stdout.write(result.stdout || "");
  process.stderr.write(result.stderr || "");
  attempts.push({
    label,
    exitStatus: result.status,
    signal: result.signal,
    browserExecutable: env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH || null,
    stdoutTail: tail(result.stdout || ""),
    stderrTail: tail(result.stderr || ""),
  });
  return result;
}

function writeReport(result, reportIssues, attempts) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        status: reportIssues.length === 0 ? "passed" : "failed",
        command,
        expectedWorkflows,
        source: "e2e/app-workflows.spec.ts",
        e2eReport: ".tmp/e2e-browser/report.json",
        exitStatus: result?.status ?? null,
        signal: result?.signal ?? null,
        attempts,
        stdoutTail: tail(result?.stdout || ""),
        stderrTail: tail(result?.stderr || ""),
        issues: reportIssues,
      },
      null,
      2,
    )}\n`,
  );
}

function tail(text) {
  return text.split(/\r?\n/).slice(-80).join("\n").trim();
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function shouldRetryWithSystemBrowser(result) {
  if (process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH) return false;
  const output = `${result.stdout || ""}\n${result.stderr || ""}`;
  return [
    "browserType.launch",
    "Target page, context or browser has been closed",
    "kill EPERM",
    "signal=SIGABRT",
    "bootstrap_check_in",
  ].some((needle) => output.includes(needle));
}
