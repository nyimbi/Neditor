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
  stdio: "inherit",
});

writePlaywrightBrowserReport(reportPath, browserResolution, result.status === 0 ? "passed" : "failed", {
  command: [binary, ...args],
  exitStatus: result.status,
  signal: result.signal,
  error: result.error ? String(result.error) : undefined,
});

process.exit(result.status ?? 1);
