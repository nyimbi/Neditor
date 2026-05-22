import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const launchRequested =
  process.argv.includes("--launch") || process.env.NEDITOR_DESKTOP_SMOKE_LAUNCH === "1";
const launchTimeoutMs = Number(process.env.NEDITOR_DESKTOP_SMOKE_TIMEOUT_MS || 8000);
const nativeWindowReportPath = join(root, ".tmp", "desktop-smoke", "native-window-report.json");
const nativeUiReportPath = join(root, ".tmp", "desktop-smoke", "native-ui-report.json");
const nativeWorkflowReportPath = join(root, ".tmp", "desktop-smoke", "native-workflow-report.json");
const nativeWorkflowFilePath = join(root, ".tmp", "desktop-smoke", "native-workflow-file.md");
const nativeWorkflowExportPath = join(root, ".tmp", "desktop-smoke", "native-workflow-export.html");
const nativeWorkflowCopyPath = join(root, ".tmp", "desktop-smoke", "native-workflow-export.md");
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
  nativeUi: null,
  nativeWorkflow: null,
  nativeAutomation: null,
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
    rmSync(nativeUiReportPath, { force: true });
    rmSync(nativeWorkflowReportPath, { force: true });
    rmSync(nativeWorkflowFilePath, { force: true });
    rmSync(nativeWorkflowExportPath, { force: true });
    rmSync(nativeWorkflowCopyPath, { force: true });
    rmSync(`${nativeWorkflowExportPath}.manifest.json`, { force: true });
    const child = spawn(path, [], {
      cwd: root,
      env: {
        ...process.env,
        RUST_BACKTRACE: "1",
        NEDITOR_DESKTOP_SMOKE_REPORT: nativeWindowReportPath,
        NEDITOR_DESKTOP_UI_SMOKE_REPORT: nativeUiReportPath,
        NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT: nativeWorkflowReportPath,
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
        validateNativeUiReport(report);
        validateNativeWorkflowReport(report);
        collectNativeAutomationReport(report, child.pid);
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

function validateNativeUiReport(launchReport) {
  if (!existsSync(nativeUiReportPath)) {
    issues.push(`native UI smoke report was not written: ${relative(nativeUiReportPath)}`);
    return;
  }
  let report;
  try {
    report = JSON.parse(readFileSync(nativeUiReportPath, "utf8"));
  } catch (error) {
    issues.push(`native UI smoke report is not valid JSON: ${error.message}`);
    return;
  }
  smokeReport.nativeUi = report;
  launchReport.nativeUi = report;
  const payload = report.payload || {};
  if (!String(payload.title || "").includes("NEditor")) {
    issues.push(`native UI report did not include the NEditor window title: ${JSON.stringify(payload.title)}`);
  }
  for (const [surface, present] of Object.entries(payload.surfaces || {})) {
    if (present !== true) {
      issues.push(`native UI report did not render the ${surface} surface`);
    }
  }
  for (const command of ["New", "Open", "Save", "Templates", "Commands"]) {
    if (!Array.isArray(payload.commandLabels) || !payload.commandLabels.includes(command)) {
      issues.push(`native UI report did not include command button ${command}`);
    }
  }
  if (!String(payload.workspaceClass || "").includes("workspace")) {
    issues.push(`native UI report did not include the workspace class: ${JSON.stringify(payload.workspaceClass)}`);
  }
  if (
    !String(payload.previewLabel || "").includes("Market Entry Report") &&
    !String(payload.surfaceText?.preview || "").includes("Transform Artifacts")
  ) {
    issues.push("native UI report did not include rendered preview identity or content");
  }
  if (!String(payload.surfaceText?.status || "").includes("diagnostics")) {
    issues.push("native UI report did not include document status text");
  }
  if (!Number.isFinite(payload.viewport?.width) || !Number.isFinite(payload.viewport?.height)) {
    issues.push(`native UI report did not include viewport dimensions: ${JSON.stringify(payload.viewport)}`);
  }
}

function validateNativeWorkflowReport(launchReport) {
  if (!existsSync(nativeWorkflowReportPath)) {
    issues.push(`native workflow smoke report was not written: ${relative(nativeWorkflowReportPath)}`);
    return;
  }
  let report;
  try {
    report = JSON.parse(readFileSync(nativeWorkflowReportPath, "utf8"));
  } catch (error) {
    issues.push(`native workflow smoke report is not valid JSON: ${error.message}`);
    return;
  }
  smokeReport.nativeWorkflow = report;
  launchReport.nativeWorkflow = report;
  const payload = report.payload || {};
  if (payload.status !== "passed") {
    issues.push(`native workflow smoke did not pass: ${JSON.stringify(payload)}`);
  }
  const assertionNames = new Set((payload.assertions || []).filter((assertion) => assertion?.passed === true).map((assertion) => assertion.name));
  for (const assertion of [
    "native workflow starts with NEditor title",
    "native workflow saved document to real file",
    "native workflow created new document",
    "native workflow opened saved real file",
    "native workflow dirtied opened real file",
    "native workflow reverted saved real file",
    "native workflow blocked stale save with external conflict",
    "native workflow kept local conflict changes",
    "native workflow saved kept-local conflict changes",
    "native workflow saved local conflict copy",
    "native workflow merged external conflict changes",
    "native workflow accepted external conflict changes",
    "native workflow restored real file after conflict proof",
    "native workflow opened command palette",
    "native workflow found dose template",
    "native workflow inserted calc template into source",
    "native workflow rendered calc template preview",
    "native workflow exposed dirty title",
    "native workflow prepared html export readiness",
    "native workflow wrote html export artifact",
    "native workflow applied dark theme attribute",
    "native workflow applied high contrast attributes and colors",
    "native workflow applied reduced motion",
    "native workflow applied editor typography",
    "native workflow applied preview theme and typography",
  ]) {
    if (!assertionNames.has(assertion)) {
      issues.push(`native workflow report did not include passing assertion: ${assertion}`);
    }
  }
  for (const mode of ["split", "source", "preview", "focus", "export", "review", "presentation"]) {
    const assertion = `native workflow switched ${mode} mode`;
    if (!assertionNames.has(assertion)) {
      issues.push(`native workflow report did not include passing assertion: ${assertion}`);
    }
  }
  const modeEvidence = Array.isArray(payload.modeEvidence) ? payload.modeEvidence : [];
  const modeSet = new Set(modeEvidence.map((entry) => entry?.mode));
  for (const mode of ["split", "source", "preview", "focus", "export", "review", "presentation"]) {
    if (!modeSet.has(mode)) {
      issues.push(`native workflow report did not include mode evidence for ${mode}`);
    }
  }
  for (const [mode, sidebar] of [
    ["export", "exports"],
    ["review", "review"],
    ["presentation", "outline"],
  ]) {
    const entry = modeEvidence.find((candidate) => candidate?.mode === mode);
    if (entry?.sidebar !== sidebar) {
      issues.push(`native workflow report did not route ${mode} mode to ${sidebar} sidebar: ${JSON.stringify(entry)}`);
    }
  }
  const fileWorkflow = payload.fileWorkflow || {};
  const expectedFilePath = nativeWorkflowFilePath.replaceAll("\\", "/");
  if (String(fileWorkflow.filePath || "").replaceAll("\\", "/") !== expectedFilePath) {
    issues.push(`native workflow report did not include real file workflow path: ${JSON.stringify(fileWorkflow)}`);
  }
  const expectedCopyPath = nativeWorkflowCopyPath.replaceAll("\\", "/");
  if (String(fileWorkflow.copyPath || "").replaceAll("\\", "/") !== expectedCopyPath) {
    issues.push(`native workflow report did not include local conflict copy path: ${JSON.stringify(fileWorkflow)}`);
  }
  if (!existsSync(nativeWorkflowFilePath)) {
    issues.push(`native workflow saved Markdown file was not written: ${relative(nativeWorkflowFilePath)}`);
  } else {
    const markdown = readFileSync(nativeWorkflowFilePath, "utf8");
    if (
      !markdown.includes("Market Entry Report") ||
      markdown.includes("Native smoke revert marker") ||
      markdown.includes("External native conflict edit") ||
      markdown.includes("Local unsaved native conflict edit") ||
      markdown.includes("Keep-local native conflict edit") ||
      markdown.includes("Save-copy native conflict edit") ||
      markdown.includes("Merged native conflict edit")
    ) {
      issues.push("native workflow saved Markdown file did not preserve reverted document content");
    }
  }
  if (!existsSync(nativeWorkflowCopyPath)) {
    issues.push(`native workflow local conflict copy was not written: ${relative(nativeWorkflowCopyPath)}`);
  } else {
    const copy = readFileSync(nativeWorkflowCopyPath, "utf8");
    if (!copy.includes("Save-copy native conflict edit")) {
      issues.push("native workflow local conflict copy did not preserve local conflict text");
    }
  }
  if (!String(payload.editorSnippet || "").includes("weight_kg = 72")) {
    issues.push("native workflow report did not include inserted calc source");
  }
  if (!String(payload.previewSnippet || "").includes("Total dose")) {
    issues.push("native workflow report did not include rendered calc preview");
  }
  if (payload.exportReadiness?.target !== "html" || !Array.isArray(payload.exportReadiness?.progressSteps)) {
    issues.push(`native workflow report did not include HTML export readiness evidence: ${JSON.stringify(payload.exportReadiness)}`);
  }
  const exportResult = payload.exportResult || {};
  const expectedExportPath = nativeWorkflowExportPath.replaceAll("\\", "/");
  if (
    exportResult.target !== "html" ||
    String(exportResult.outputPath || "").replaceAll("\\", "/") !== expectedExportPath ||
    !Array.isArray(exportResult.progressSteps) ||
    !exportResult.progressSteps.includes("render")
  ) {
    issues.push(`native workflow report did not include HTML export write evidence: ${JSON.stringify(exportResult)}`);
  }
  if (!existsSync(nativeWorkflowExportPath)) {
    issues.push(`native workflow HTML export artifact was not written: ${relative(nativeWorkflowExportPath)}`);
  } else {
    const html = readFileSync(nativeWorkflowExportPath, "utf8");
    if (!html.includes("Total dose") || !html.includes("Market Entry Report")) {
      issues.push("native workflow HTML export artifact did not include rendered document content");
    }
  }
  if (!existsSync(`${nativeWorkflowExportPath}.manifest.json`)) {
    issues.push(`native workflow HTML export manifest was not written: ${relative(`${nativeWorkflowExportPath}.manifest.json`)}`);
  } else {
    const manifest = readFileSync(`${nativeWorkflowExportPath}.manifest.json`, "utf8");
    if (!manifest.includes('"export_target": "html"') || !manifest.includes('"output_hash"')) {
      issues.push("native workflow HTML export manifest did not include target/hash evidence");
    }
  }
  const theme = payload.themeAccessibility || {};
  if (
    theme.shellTheme !== "dark" ||
    theme.highContrast !== "true" ||
    theme.reducedMotion !== "true" ||
    theme.previewTheme !== "dark" ||
    theme.commandBorderColor !== "rgb(0, 0, 0)" ||
    theme.editorTransitionDuration !== "0s" ||
    theme.editorFontSize !== "18px" ||
    !String(theme.previewStyle || "").includes("font-size: 19px")
  ) {
    issues.push(`native workflow report did not include theme/accessibility evidence: ${JSON.stringify(theme)}`);
  }
}

function writeLaunchReport(report) {
  const directory = join(root, ".tmp", "desktop-smoke");
  mkdirSync(directory, { recursive: true });
  writeFileSync(join(directory, "launch-report.json"), `${JSON.stringify(report, null, 2)}\n`);
}

function collectNativeAutomationReport(launchReport, pid) {
  if (process.platform !== "darwin") return;
  const script = `
set targetPid to ${Number(pid) || 0}
tell application "System Events"
  set matches to every process whose unix id is targetPid
  if (count of matches) is 0 then error "process not found for pid " & targetPid
  set targetProcess to item 1 of matches
  set processName to name of targetProcess
  set processFrontmost to frontmost of targetProcess
  set windowCount to count of windows of targetProcess
  set windowName to ""
  set windowWidth to 0
  set windowHeight to 0
  if windowCount > 0 then
    set firstWindow to window 1 of targetProcess
    set windowName to name of firstWindow
    set windowSize to size of firstWindow
    set windowWidth to item 1 of windowSize
    set windowHeight to item 2 of windowSize
  end if
  return processName & linefeed & processFrontmost & linefeed & windowCount & linefeed & windowName & linefeed & windowWidth & linefeed & windowHeight
end tell
`;
  const result = spawnSync("osascript", ["-e", script], {
    encoding: "utf8",
    timeout: 5000,
  });
  const output = result.stdout.trim();
  const stderr = result.stderr.trim();
  if (result.status !== 0) {
    const detail = stderr || output || `osascript exited with code ${result.status ?? "unknown"}`;
    const automation = {
      tool: "osascript:System Events",
      status: "skipped",
      reason: detail,
    };
    launchReport.nativeAutomation = automation;
    smokeReport.nativeAutomation = automation;
    return;
  }
  const [processName, frontmost, windowCount, windowName, windowWidth, windowHeight] = output.split(/\r?\n/);
  const automation = {
    tool: "osascript:System Events",
    status: "passed",
    processName,
    frontmost: frontmost === "true",
    windowCount: Number(windowCount),
    window: {
      name: windowName,
      width: Number(windowWidth),
      height: Number(windowHeight),
    },
  };
  if (!Number.isFinite(automation.windowCount) || automation.windowCount < 1) {
    automation.status = "limited";
    automation.reason = "System Events found the NEditor process but did not expose a window for this launch; the app-authored native window report remains authoritative.";
  }
  launchReport.nativeAutomation = automation;
  smokeReport.nativeAutomation = automation;
  if (!processName || !processName.toLowerCase().includes("neditor")) {
    issues.push(`macOS native automation found an unexpected process name: ${JSON.stringify(processName)}`);
  }
  if (automation.status === "passed" && (!Number.isFinite(automation.window.width) || !Number.isFinite(automation.window.height))) {
    issues.push(`macOS native automation did not report a usable window size: ${JSON.stringify(automation.window)}`);
  }
}

function tail(value) {
  const lines = value.trim().split(/\r?\n/).filter(Boolean);
  return lines.slice(-12);
}
