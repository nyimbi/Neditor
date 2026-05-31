import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, extname, join, resolve, sep } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = parseArgs(process.argv.slice(2));
const bundles = String(args.bundles || args.bundle || "all")
  .split(/[,\s]+/)
  .map((entry) => entry.trim())
  .filter(Boolean);
const timeoutMs = positiveInteger(args.timeout || process.env.NEDITOR_TAURI_BUILD_TIMEOUT_MS, 40 * 60 * 1000);
const progressMs = positiveInteger(args["progress-ms"] || process.env.NEDITOR_TAURI_BUILD_PROGRESS_MS, 60 * 1000);
const reportPath = resolve(args["report"] || join(root, ".tmp", "tauri-build", "report.json"));
const bundleRoot = resolve(args["bundle-root"] || join(root, "src-tauri", "target", "release", "bundle"));
const startedAt = new Date();
const command = {
  executable: process.platform === "win32" ? "pnpm.cmd" : "pnpm",
  args: ["tauri", "build", "--bundles", ...bundles],
};
const outputTail = [];
let child;
let timedOut = false;
let lastProgressAt = Date.now();
let finished = false;
let progressTimer;
let timeoutTimer;

writeReport({
  status: "running",
  startedAt: startedAt.toISOString(),
  completedAt: null,
  durationMs: 0,
  timeoutMs,
  bundles,
  command: renderCommand(command),
  artifacts: [],
  outputTail: [],
});

console.log(`[tauri-build] starting ${renderCommand(command)} with timeout ${timeoutMs}ms`);

child = spawn(command.executable, command.args, {
  cwd: root,
  env: process.env,
  stdio: ["ignore", "pipe", "pipe"],
});

child.stdout.on("data", (chunk) => forwardOutput("stdout", chunk));
child.stderr.on("data", (chunk) => forwardOutput("stderr", chunk));
child.on("error", (error) => {
  finish({
    status: "failed",
    exitCode: null,
    signal: null,
    error: error.message,
  });
});

progressTimer = setInterval(() => {
  const elapsedMs = Date.now() - startedAt.getTime();
  console.log(`[tauri-build] still running after ${Math.round(elapsedMs / 1000)}s; last output ${Math.round((Date.now() - lastProgressAt) / 1000)}s ago`);
  writeReport({
    status: "running",
    startedAt: startedAt.toISOString(),
    completedAt: null,
    durationMs: elapsedMs,
    timeoutMs,
    bundles,
    command: renderCommand(command),
    artifacts: collectArtifacts(),
    outputTail,
  });
}, progressMs);

timeoutTimer = setTimeout(() => {
  timedOut = true;
  console.error(`[tauri-build] timeout after ${timeoutMs}ms; terminating build`);
  child.kill("SIGTERM");
  setTimeout(() => {
    if (!child.killed) child.kill("SIGKILL");
  }, 10_000).unref();
}, timeoutMs);

child.on("close", (exitCode, signal) => {
  clearInterval(progressTimer);
  clearTimeout(timeoutTimer);
  const status = exitCode === 0 && !timedOut ? "passed" : timedOut ? "timed-out" : "failed";
  finish({ status, exitCode, signal, error: null });
});

function finish({ status, exitCode, signal, error }) {
  if (finished) return;
  finished = true;
  clearInterval(progressTimer);
  clearTimeout(timeoutTimer);
  const completedAt = new Date();
  const report = {
    status,
    startedAt: startedAt.toISOString(),
    completedAt: completedAt.toISOString(),
    durationMs: completedAt.getTime() - startedAt.getTime(),
    timeoutMs,
    bundles,
    command: renderCommand(command),
    exitCode,
    signal,
    error,
    artifacts: collectArtifacts(),
    outputTail,
  };
  writeReport(report);
  if (status === "passed") {
    console.log(`[tauri-build] passed; wrote ${relative(reportPath)}`);
    return;
  }
  console.error(`[tauri-build] ${status}; wrote ${relative(reportPath)}`);
  process.exitCode = 1;
}

function forwardOutput(stream, chunk) {
  lastProgressAt = Date.now();
  const text = chunk.toString();
  process[stream].write(text);
  for (const line of text.split(/\r?\n/)) {
    if (!line.trim()) continue;
    outputTail.push({ stream, line: line.slice(0, 500) });
  }
  while (outputTail.length > 120) outputTail.shift();
}

function collectArtifacts() {
  if (!existsSync(bundleRoot)) return [];
  return walk(bundleRoot)
    .filter((path) => [".appimage", ".deb", ".rpm", ".exe", ".msi"].includes(extname(path.toLowerCase())))
    .map((path) => {
      const stats = statSync(path);
      return {
        path: relative(path),
        bytes: stats.size,
        sha256: createHash("sha256").update(readFileSync(path)).digest("hex"),
      };
    })
    .sort((a, b) => a.path.localeCompare(b.path));
}

function walk(directory) {
  const entries = [];
  for (const name of readdirSync(directory)) {
    const path = join(directory, name);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      entries.push(...walk(path));
    } else {
      entries.push(path);
    }
  }
  return entries;
}

function writeReport(report) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        schema: "neditor.tauri-build-report.v1",
        generatedAt: new Date().toISOString(),
        platform: process.platform,
        arch: process.arch,
        ...report,
      },
      null,
      2,
    )}\n`,
  );
}

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (!arg.startsWith("--")) continue;
    const key = arg.slice(2);
    const next = argv[index + 1];
    if (!next || next.startsWith("--")) {
      parsed[key] = true;
    } else {
      parsed[key] = next;
      index += 1;
    }
  }
  return parsed;
}

function positiveInteger(value, fallback) {
  const number = Number(value);
  return Number.isInteger(number) && number > 0 ? number : fallback;
}

function renderCommand(command) {
  return [command.executable, ...command.args.map(quoteArg)].join(" ");
}

function quoteArg(arg) {
  return /\s/.test(arg) ? JSON.stringify(arg) : arg;
}

function relative(path) {
  const normalized = path.split(sep).join("/");
  const normalizedRoot = root.split(sep).join("/");
  return normalized.startsWith(normalizedRoot) ? normalized.slice(normalizedRoot.length + 1) : path;
}
