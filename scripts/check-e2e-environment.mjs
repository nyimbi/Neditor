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
const reportPath = join(root, ".tmp", "e2e-environment", "report.json");
const browserResolution = resolvePlaywrightBrowserEnv(process.env);
const maxAttempts = Math.max(1, Number(process.env.NEDITOR_E2E_ENV_ATTEMPTS || 5));
const retryBackoffMs = Math.max(0, Number(process.env.NEDITOR_E2E_ENV_RETRY_BACKOFF_MS || 1500));

if (!browserResolution.ok) {
  writePlaywrightBrowserReport(reportPath, browserResolution, "blocked");
  console.error(browserResolution.message);
  process.exit(1);
}

let result = null;
const attempts = [];
for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
  result = spawnSync(binary, ["test", "--grep", "boots the workbench"], {
    cwd: root,
    env: browserResolution.env,
    shell: process.platform === "win32",
    stdio: "pipe",
    encoding: "utf8",
  });
  attempts.push({
    attempt,
    exitStatus: result.status,
    signal: result.signal,
    stdoutTail: tail(result.stdout),
    stderrTail: tail(result.stderr),
  });
  if (result.status === 0 || !isTransientBrowserLaunchFailure(result) || attempt === maxAttempts) break;
  const delayMs = retryBackoffMs * attempt;
  console.error(`E2E environment check hit a transient Chromium launch failure on attempt ${attempt}; retrying in ${delayMs}ms.`);
  sleep(delayMs);
}

writePlaywrightBrowserReport(reportPath, browserResolution, result.status === 0 ? "passed" : "failed", {
  command: [binary, "test", "--grep", "boots the workbench"],
  exitStatus: result.status,
  signal: result.signal,
  attempts,
  stdoutTail: tail(result.stdout),
  stderrTail: tail(result.stderr),
});

if (result.status !== 0) {
  console.error("E2E environment check failed: the focused Chromium boot workflow did not pass.");
  if (result.stdout.trim()) console.error(result.stdout.trim());
  if (result.stderr.trim()) console.error(result.stderr.trim());
  process.exit(result.status ?? 1);
}

console.log(`${browserResolution.message} Playwright Chromium launch preflight passed through the focused workbench boot workflow.`);

function tail(output) {
  return output
    .split(/\r?\n/)
    .filter(Boolean)
    .slice(-80);
}

function isTransientBrowserLaunchFailure(result) {
  const output = `${result.stdout || ""}\n${result.stderr || ""}`;
  return [
    "browserType.launch",
    "Target page, context or browser has been closed",
    "kill EPERM",
    "signal=SIGABRT",
    "bootstrap_check_in",
  ].some((needle) => output.includes(needle));
}

function sleep(ms) {
  if (ms <= 0) return;
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}
