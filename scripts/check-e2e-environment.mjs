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

if (!browserResolution.ok) {
  writePlaywrightBrowserReport(reportPath, browserResolution, "blocked");
  console.error(browserResolution.message);
  process.exit(1);
}

const result = spawnSync(binary, ["test", "--grep", "boots the workbench"], {
  cwd: root,
  env: browserResolution.env,
  shell: process.platform === "win32",
  stdio: "pipe",
  encoding: "utf8",
});

writePlaywrightBrowserReport(reportPath, browserResolution, result.status === 0 ? "passed" : "failed", {
  command: [binary, "test", "--grep", "boots the workbench"],
  exitStatus: result.status,
  signal: result.signal,
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
