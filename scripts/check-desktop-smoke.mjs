import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const launchRequested =
  process.argv.includes("--launch") || process.env.NEDITOR_DESKTOP_SMOKE_LAUNCH === "1";
const launchTimeoutMs = Number(process.env.NEDITOR_DESKTOP_SMOKE_TIMEOUT_MS || 3000);
const nativeWindowReportPath = join(root, ".tmp", "desktop-smoke", "native-window-report.json");
const issues = [];

const tauriConfig = readJson("src-tauri/tauri.conf.json");
const packageJson = readJson("package.json");
const cargoToml = readText("src-tauri/Cargo.toml");
const binaryPath = desktopBinaryPath();
const smokeReport = {
  generatedAt: new Date().toISOString(),
  platform: process.platform,
  arch: process.arch,
  binary: relative(binaryPath),
  binarySize: null,
  frontendAssets: {
    jsBundles: 0,
  },
  nativeCommandWorkflow: null,
  nativeWindow: null,
};

requireEqual(packageJson.license, "MIT", "package.json must declare MIT license");
requireEqual(tauriConfig.productName, "NEditor", "Tauri productName must remain NEditor");
requireEqual(tauriConfig.identifier, "com.neditor.desktop", "Tauri bundle identifier must remain stable");
requireEqual(tauriConfig.build?.frontendDist, "../dist", "Tauri frontendDist must point at the built Vite dist");
requireEqual(tauriConfig.bundle?.license, "MIT", "Tauri bundle must declare MIT license");
requireEqual(tauriConfig.bundle?.licenseFile, "../LICENSE", "Tauri bundle must point at the repository license file");
requireIncludes(cargoToml, 'name = "neditor"', "Cargo package must remain named neditor");
requireIncludes(cargoToml, 'license = "MIT"', "Cargo package must declare MIT license");

requireFile("dist/index.html", "frontend build output is missing; run pnpm run build first");
const assetDir = join(root, "dist", "assets");
const assetNames = existsSync(assetDir) ? readdirSync(assetDir) : [];
smokeReport.frontendAssets.jsBundles = assetNames.filter((name) => name.endsWith(".js")).length;
if (!existsSync(assetDir) || smokeReport.frontendAssets.jsBundles === 0) {
  issues.push("frontend asset bundle is missing from dist/assets");
}
requireExecutable(binaryPath, "desktop release binary is missing; run ./node_modules/.bin/tauri build --no-bundle first");
if (existsSync(binaryPath)) {
  smokeReport.binarySize = statSync(binaryPath).size;
}

if (issues.length === 0) {
  runNativeCommandWorkflowSmoke();
}

if (issues.length === 0 && launchRequested) {
  await launchDesktop(binaryPath);
}

if (issues.length > 0) {
  writeSmokeReport();
  console.error("Desktop smoke check failed:");
  for (const issue of issues) {
    console.error(`- ${issue}`);
  }
  process.exit(1);
}

writeSmokeReport();
console.log(
  launchRequested
    ? "Checked NEditor desktop build artifacts, native command workflow smoke, and bounded launch smoke."
    : "Checked NEditor desktop build artifacts and native command workflow smoke. Set NEDITOR_DESKTOP_SMOKE_LAUNCH=1 for bounded GUI launch smoke.",
);

function readJson(relativePath) {
  return JSON.parse(readText(relativePath));
}

function readText(relativePath) {
  return readFileSync(join(root, relativePath), "utf8");
}

function requireEqual(actual, expected, message) {
  if (actual !== expected) {
    issues.push(`${message}: expected ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function requireIncludes(text, needle, message) {
  if (!text.includes(needle)) {
    issues.push(message);
  }
}

function requireFile(relativePath, message) {
  const path = join(root, relativePath);
  if (!existsSync(path) || !statSync(path).isFile()) {
    issues.push(message);
  }
}

function requireExecutable(path, message) {
  if (!existsSync(path) || !statSync(path).isFile()) {
    issues.push(message);
    return;
  }
  if (process.platform !== "win32" && (statSync(path).mode & 0o111) === 0) {
    issues.push(`desktop release binary is not executable: ${relative(path)}`);
  }
}

function desktopBinaryPath() {
  const name = process.platform === "win32" ? "neditor.exe" : "neditor";
  return join(root, "src-tauri", "target", "release", name);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}

function runNativeCommandWorkflowSmoke() {
  const startedAt = Date.now();
  const result = spawnSync(
    "cargo",
    ["test", "--locked", "desktop_native_command_workflow_smoke", "--lib"],
    {
      cwd: join(root, "src-tauri"),
      encoding: "utf8",
      shell: process.platform === "win32",
    },
  );
  smokeReport.nativeCommandWorkflow = {
    command: "cargo test --locked desktop_native_command_workflow_smoke --lib",
    status: result.status ?? 1,
    durationMs: Date.now() - startedAt,
    stdoutTail: tail(result.stdout),
    stderrTail: tail(result.stderr),
  };
  if (result.status !== 0) {
    const detail = [result.stdout?.trim(), result.stderr?.trim()].filter(Boolean).join("\n");
    issues.push(
      `native command workflow smoke failed with exit code ${result.status ?? 1}${
        detail ? `:\n${detail}` : ""
      }`,
    );
  }
}

function writeSmokeReport() {
  const directory = join(root, ".tmp", "desktop-smoke");
  mkdirSync(directory, { recursive: true });
  writeFileSync(join(directory, "native-command-report.json"), `${JSON.stringify(smokeReport, null, 2)}\n`);
}

async function launchDesktop(path) {
  await new Promise((resolveLaunch) => {
    const startedAt = Date.now();
    rmSync(nativeWindowReportPath, { force: true });
    const child = spawn(path, [], {
      cwd: root,
      env: {
        ...process.env,
        RUST_BACKTRACE: "1",
        NEDITOR_DESKTOP_SMOKE_REPORT: nativeWindowReportPath,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    const report = {
      platform: process.platform,
      binary: relative(path),
      pid: child.pid,
      timeoutMs: launchTimeoutMs,
      status: "started",
      observedMs: 0,
      processAlive: false,
      stdout: "",
      stderr: "",
    };
    let stdout = "";
    let stderr = "";
    let settled = false;
    const timeout = setTimeout(() => {
      if (settled) return;
      settled = true;
      report.observedMs = Date.now() - startedAt;
      report.processAlive = isProcessAlive(child.pid);
      report.stdout = stdout.trim();
      report.stderr = stderr.trim();
      if (!report.processAlive) {
        report.status = "not-running-at-timeout";
        issues.push(`desktop launch process was not alive after ${report.observedMs}ms`);
      } else {
        report.status = "survived-until-timeout";
        validateNativeWindowReport(report);
      }
      writeLaunchReport(report);
      child.kill("SIGTERM");
      resolveLaunch();
    }, launchTimeoutMs);
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.on("error", (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      report.status = "spawn-error";
      report.observedMs = Date.now() - startedAt;
      report.stderr = error.message;
      writeLaunchReport(report);
      issues.push(`desktop launch failed: ${error.message}`);
      resolveLaunch();
    });
    child.on("exit", (code, signal) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      report.status = "exited-early";
      report.observedMs = Date.now() - startedAt;
      report.processAlive = false;
      report.stdout = stdout.trim();
      report.stderr = stderr.trim();
      writeLaunchReport(report);
      const detail = [stdout.trim(), stderr.trim()].filter(Boolean).join("\n");
      issues.push(
        `desktop launch exited before the ${launchTimeoutMs}ms smoke window with code ${
          code ?? "none"
        } signal ${signal ?? "none"}${detail ? `: ${detail}` : ""}`,
      );
      resolveLaunch();
    });
  });
}

function isProcessAlive(pid) {
  if (!pid) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function validateNativeWindowReport(launchReport) {
  if (!existsSync(nativeWindowReportPath)) {
    issues.push(`native launch report was not written: ${relative(nativeWindowReportPath)}`);
    return;
  }
  let report;
  try {
    report = JSON.parse(readFileSync(nativeWindowReportPath, "utf8"));
  } catch (error) {
    issues.push(`native launch report is not valid JSON: ${error.message}`);
    return;
  }
  smokeReport.nativeWindow = report;
  launchReport.nativeWindow = report;
  if (report.packageName !== "NEditor") {
    issues.push(`native launch package name changed: expected "NEditor", found ${JSON.stringify(report.packageName)}`);
  }
  if (report.identifier !== "com.neditor.desktop") {
    issues.push(
      `native launch bundle identifier changed: expected "com.neditor.desktop", found ${JSON.stringify(
        report.identifier,
      )}`,
    );
  }
  if (report.window?.label !== "main") {
    issues.push(`native launch did not report the main window label: ${JSON.stringify(report.window?.label)}`);
  }
  if (report.window?.title !== "NEditor") {
    issues.push(`native launch title changed: expected "NEditor", found ${JSON.stringify(report.window?.title)}`);
  }
  if (report.window?.visible !== true) {
    issues.push(`native launch did not report a visible window: ${JSON.stringify(report.window?.visible)}`);
  }
  if (!Number.isFinite(report.window?.innerSize?.width) || !Number.isFinite(report.window?.innerSize?.height)) {
    issues.push(`native launch did not report a usable window size: ${JSON.stringify(report.window?.innerSize)}`);
  }
}

function writeLaunchReport(report) {
  const directory = join(root, ".tmp", "desktop-smoke");
  mkdirSync(directory, { recursive: true });
  writeFileSync(join(directory, "launch-report.json"), `${JSON.stringify(report, null, 2)}\n`);
}

function tail(value) {
  const lines = value.trim().split(/\r?\n/).filter(Boolean);
  return lines.slice(-12);
}
