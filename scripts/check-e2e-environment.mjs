import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const binary = join(
  root,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "playwright.cmd" : "playwright",
);
const env = {
  ...process.env,
  PLAYWRIGHT_BROWSERS_PATH: process.env.PLAYWRIGHT_BROWSERS_PATH ?? "0",
};

const result = spawnSync(binary, ["test", "--grep", "boots the workbench"], {
  cwd: root,
  env,
  shell: process.platform === "win32",
  stdio: "pipe",
  encoding: "utf8",
});

if (result.status !== 0) {
  console.error("E2E environment check failed: the focused Chromium boot workflow did not pass.");
  if (result.stdout.trim()) console.error(result.stdout.trim());
  if (result.stderr.trim()) console.error(result.stderr.trim());
  process.exit(result.status ?? 1);
}

console.log("Playwright Chromium launch preflight passed through the focused workbench boot workflow.");
