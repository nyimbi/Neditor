import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const launchRequested =
  process.argv.includes("--launch") || process.env.NEDITOR_DESKTOP_SMOKE_LAUNCH === "1";
const launchTimeoutMs = Number(process.env.NEDITOR_DESKTOP_SMOKE_TIMEOUT_MS || 60000);
const launchAttempts = Math.max(1, Number(process.env.NEDITOR_DESKTOP_SMOKE_ATTEMPTS || 3));
const nativeWindowReportPath = join(root, ".tmp", "desktop-smoke", "native-window-report.json");
const nativeUiReportPath = join(root, ".tmp", "desktop-smoke", "native-ui-report.json");
const nativeWorkflowReportPath = join(root, ".tmp", "desktop-smoke", "native-workflow-report.json");
const nativeWorkflowFilePath = join(root, ".tmp", "desktop-smoke", "native-workflow-file.md");
const nativeWorkflowIncludePath = join(root, ".tmp", "desktop-smoke", "native-workflow-file.include");
const nativeWorkflowExportPath = join(root, ".tmp", "desktop-smoke", "native-workflow-export.html");
const nativeWorkflowCopyPath = join(root, ".tmp", "desktop-smoke", "native-workflow-export.md");
const nativeSmokeHomePath = join(root, ".tmp", "desktop-smoke", "home");
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
  for (let attempt = 1; attempt <= launchAttempts; attempt += 1) {
    const issueStart = issues.length;
    const report = await launchDesktopAttempt(path, attempt);
    const newIssues = issues.slice(issueStart);
    if (newIssues.length === 0) return;
    if (attempt < launchAttempts && shouldRetryLaunchAttempt(report)) {
      issues.splice(issueStart);
      await sleep(750);
      continue;
    }
    return;
  }
}

async function launchDesktopAttempt(path, attempt) {
  return new Promise((resolveLaunch) => {
    const startedAt = Date.now();
    rmSync(nativeWindowReportPath, { force: true });
    rmSync(nativeUiReportPath, { force: true });
    rmSync(nativeWorkflowReportPath, { force: true });
    rmSync(nativeWorkflowFilePath, { force: true });
    rmSync(nativeWorkflowIncludePath, { force: true });
    rmSync(nativeWorkflowExportPath, { force: true });
    rmSync(nativeWorkflowCopyPath, { force: true });
    rmSync(`${nativeWorkflowExportPath}.manifest.json`, { force: true });
    rmSync(nativeSmokeHomePath, { recursive: true, force: true });
    mkdirSync(nativeSmokeHomePath, { recursive: true });
    const child = spawn(path, [], {
      cwd: root,
      env: {
        ...process.env,
        HOME: nativeSmokeHomePath,
        XDG_CONFIG_HOME: join(nativeSmokeHomePath, ".config"),
        XDG_DATA_HOME: join(nativeSmokeHomePath, ".local", "share"),
        XDG_CACHE_HOME: join(nativeSmokeHomePath, ".cache"),
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
      attempt,
      maxAttempts: launchAttempts,
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
    const timeout = setTimeout(async () => {
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
      await terminateLaunchedProcess(child);
      resolveLaunch(report);
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
      resolveLaunch(report);
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
      resolveLaunch(report);
    });
  });
}

async function terminateLaunchedProcess(child) {
  if (!child.pid || child.exitCode !== null || child.signalCode !== null) return;
  child.kill("SIGTERM");
  await new Promise((resolve) => {
    const fallback = setTimeout(() => {
      if (isProcessAlive(child.pid)) child.kill("SIGKILL");
      resolve();
    }, 2000);
    child.once("exit", () => {
      clearTimeout(fallback);
      resolve();
    });
  });
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function shouldRetryLaunchAttempt(report) {
  if (report?.status !== "survived-until-timeout") return false;
  if (existsSync(nativeWindowReportPath) || existsSync(nativeUiReportPath) || existsSync(nativeWorkflowReportPath)) {
    return false;
  }
  return /Connection invalid|hiservices-xpcservice|scheduleApplicationNotification|-10827/i.test(
    `${report.stderr || ""}\n${report.nativeAutomation?.reason || ""}`,
  );
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
    "native workflow save cleared native title",
    "native workflow created new document",
    "native workflow opened saved real file",
    "native workflow dirtied opened real file",
    "native workflow dirtied native title for opened real file",
    "native workflow reverted saved real file",
    "native workflow revert cleared native title",
    "native workflow created and listed app-data snapshot",
    "native workflow dirtied document before snapshot restore",
    "native workflow restored app-data snapshot",
    "native workflow created and listed project-local snapshot",
    "native workflow dirtied document before project-local snapshot restore",
    "native workflow restored project-local snapshot",
    "native workflow reloaded clean external watcher change",
    "native workflow restored clean watcher reload",
    "native workflow watched included file with native driver",
    "native workflow recompiled clean included watcher change",
    "native workflow restored included watcher root",
    "native workflow blocked stale save with external conflict",
    "native workflow rendered conflict modal controls",
    "native workflow conflict modal seeded local merge base",
    "native workflow conflict modal seeded external merge base",
    "native workflow kept local conflict changes",
    "native workflow saved kept-local conflict changes",
    "native workflow saved local conflict copy",
    "native workflow merged external conflict changes",
    "native workflow accepted external conflict changes",
    "native workflow restored real file after conflict proof",
    "native workflow reported editor word statistics",
    "native workflow exposed spellcheck editor attributes",
    "native workflow rendered line numbers word wrap and folding gutter",
    "native workflow opened editor search panel",
    "native workflow replaced editor search target",
    "native workflow continued markdown list in editor",
    "native workflow inserted paired bracket in editor",
    "native workflow edited multiple cursors in editor",
    "native workflow navigated outline heading to source",
    "native workflow opened command palette",
    "native workflow found dose template",
    "native workflow inserted calc template into source",
    "native workflow rendered calc template preview",
    "native workflow exposed dirty title",
    "native workflow prepared html export readiness",
    "native workflow wrote html export artifact",
    "native workflow exported html from native menu command",
    "native workflow rendered outline mode structure only",
    "native workflow rendered export mode preview content",
    "native workflow rendered review mode governance content",
    "native workflow rendered presentation outline content",
    "native workflow routed export preview from native view menu",
    "native workflow routed outline from native view menu",
    "native workflow routed exports from native view menu",
    "native workflow opened search from native menu command",
    "native workflow inserted toc from native writing tools menu",
    "native workflow inserted equation from native writing tools menu",
    "native workflow inserted code fence from native writing tools menu",
    "native workflow inserted table from native writing tools menu",
    "native workflow opened templates from native writing tools menu",
    "native workflow opened AI paste from native writing tools menu",
    "native workflow grouped document-set tabs",
    "native workflow pinned tab into pinned group",
    "native workflow assigned loose tab to document set",
    "native workflow closed document-set tab group",
    "native workflow reopened recently closed tab",
    "native workflow restored workspace tabs with active pinned and scroll state",
    "native workflow saved export profile",
    "native workflow applied export profile",
    "native workflow reloaded export profile from settings store",
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
  for (const mode of ["split", "source", "preview", "focus", "outline", "export", "review", "presentation"]) {
    const assertion = `native workflow switched ${mode} mode`;
    if (!assertionNames.has(assertion)) {
      issues.push(`native workflow report did not include passing assertion: ${assertion}`);
    }
  }
  const modeEvidence = Array.isArray(payload.modeEvidence) ? payload.modeEvidence : [];
  const modeSet = new Set(modeEvidence.map((entry) => entry?.mode));
  for (const mode of ["split", "source", "preview", "focus", "outline", "export", "review", "presentation"]) {
    if (!modeSet.has(mode)) {
      issues.push(`native workflow report did not include mode evidence for ${mode}`);
    }
  }
  for (const [mode, sidebar] of [
    ["outline", "outline"],
    ["export", "exports"],
    ["review", "review"],
    ["presentation", "outline"],
  ]) {
    const entry = modeEvidence.find((candidate) => candidate?.mode === mode);
    if (entry?.sidebar !== sidebar) {
      issues.push(`native workflow report did not route ${mode} mode to ${sidebar} sidebar: ${JSON.stringify(entry)}`);
    }
  }
  const modeEntry = (mode) => modeEvidence.find((candidate) => candidate?.mode === mode) || {};
  const expectedModeVisibility = {
    split: { sourceVisible: true, previewVisible: true },
    source: { sourceVisible: true, previewVisible: false },
    preview: { sourceVisible: false, previewVisible: true },
    focus: { sourceVisible: true, previewVisible: false },
    outline: { sourceVisible: false, previewVisible: false },
    export: { sourceVisible: false, previewVisible: true },
    review: { sourceVisible: true, previewVisible: true },
    presentation: { sourceVisible: false, previewVisible: true },
  };
  for (const [mode, expected] of Object.entries(expectedModeVisibility)) {
    const entry = modeEntry(mode);
    if (entry.sourceVisible !== expected.sourceVisible || entry.previewVisible !== expected.previewVisible) {
      issues.push(`native workflow report did not include correct pane visibility for ${mode}: ${JSON.stringify(entry)}`);
    }
  }
  const outlineModeEntry = modeEntry("outline");
  const outlineModeTitles = Array.isArray(outlineModeEntry.outlineTitles) ? outlineModeEntry.outlineTitles : [];
  if (
    outlineModeEntry.outlineVisible !== true ||
    !outlineModeTitles.includes("Market Entry Report") ||
    !outlineModeTitles.includes("Executive Summary") ||
    !String(outlineModeEntry.outlineText || "").includes("Add heading")
  ) {
    issues.push(`native workflow report did not include rendered outline-mode structure: ${JSON.stringify(outlineModeEntry)}`);
  }
  const exportModeEntry = modeEntry("export");
  if (
    !String(exportModeEntry.previewText || "").includes("HTML export preview") ||
    !String(exportModeEntry.previewText || "").includes("Market Entry Report") ||
    !String(exportModeEntry.sidebarText || "").includes("HTML delivery")
  ) {
    issues.push(`native workflow report did not include rendered export-mode content: ${JSON.stringify(exportModeEntry)}`);
  }
  const reviewModeEntry = modeEntry("review");
  if (
    !String(reviewModeEntry.sidebarText || "").includes("Review") ||
    !String(reviewModeEntry.sidebarText || "").includes("Summary") ||
    !String(reviewModeEntry.sidebarText || "").includes("Approved by")
  ) {
    issues.push(`native workflow report did not include rendered review-mode governance content: ${JSON.stringify(reviewModeEntry)}`);
  }
  const presentationModeEntry = modeEntry("presentation");
  if (!String(presentationModeEntry.sidebarText || "").includes("Outline") || !String(presentationModeEntry.previewText || "").includes("Market Entry Report")) {
    issues.push(`native workflow report did not include rendered presentation outline content: ${JSON.stringify(presentationModeEntry)}`);
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
  const expectedIncludePath = nativeWorkflowIncludePath.replaceAll("\\", "/");
  if (String(fileWorkflow.includePath || "").replaceAll("\\", "/") !== expectedIncludePath) {
    issues.push(`native workflow report did not include included watcher path: ${JSON.stringify(fileWorkflow)}`);
  }
  if (!existsSync(nativeWorkflowFilePath)) {
    issues.push(`native workflow saved Markdown file was not written: ${relative(nativeWorkflowFilePath)}`);
  } else {
    const markdown = readFileSync(nativeWorkflowFilePath, "utf8");
    if (
      !markdown.includes("Market Entry Report") ||
      markdown.includes("Native smoke revert marker") ||
      markdown.includes("Native clean watcher reload marker") ||
      markdown.includes("Native include watcher") ||
      markdown.includes("!include") ||
      markdown.includes("External native conflict edit") ||
      markdown.includes("Local unsaved native conflict edit") ||
      markdown.includes("Keep-local native conflict edit") ||
      markdown.includes("Save-copy native conflict edit") ||
      markdown.includes("Merged native conflict edit")
    ) {
      issues.push("native workflow saved Markdown file did not preserve reverted document content");
    }
  }
  if (!existsSync(nativeWorkflowIncludePath)) {
    issues.push(`native workflow included watcher file was not written: ${relative(nativeWorkflowIncludePath)}`);
  } else {
    const included = readFileSync(nativeWorkflowIncludePath, "utf8");
    if (!included.includes("Native include watcher updated")) {
      issues.push("native workflow included watcher file did not preserve updated include text");
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
  const editorErgonomicsEvidence = payload.editorErgonomicsEvidence || {};
  if (
    editorErgonomicsEvidence.settings?.wordWrapEnabled !== true ||
    editorErgonomicsEvidence.settings?.lineNumbersVisible !== true ||
    editorErgonomicsEvidence.settings?.foldGutterVisible !== true ||
    editorErgonomicsEvidence.settings?.spellcheck !== "true" ||
    editorErgonomicsEvidence.settings?.autocapitalize !== "sentences" ||
    editorErgonomicsEvidence.settings?.role !== "textbox" ||
    !String(editorErgonomicsEvidence.settings?.ariaLabel || "").includes("Markdown") ||
    !String(editorErgonomicsEvidence.settings?.wordStats || "").includes("words") ||
    !String(editorErgonomicsEvidence.settings?.wordStats || "").includes("characters") ||
    editorErgonomicsEvidence.searchReplace?.searchPanelOpen !== true ||
    editorErgonomicsEvidence.searchReplace?.containsReplacement !== true ||
    editorErgonomicsEvidence.searchReplace?.containsOriginal !== false ||
    !String(editorErgonomicsEvidence.listContinuation?.text || "").includes("- First item\n- Second item") ||
    !String(editorErgonomicsEvidence.pairing?.text || "").includes("()") ||
    editorErgonomicsEvidence.multiCursor?.inserted !== true
  ) {
    issues.push(`native workflow report did not include editor ergonomics evidence: ${JSON.stringify(editorErgonomicsEvidence)}`);
  }
  const outlineNavigationEvidence = payload.outlineNavigationEvidence?.outline || {};
  if (
    outlineNavigationEvidence.sidebar !== "outline" ||
    outlineNavigationEvidence.mode !== "split" ||
    outlineNavigationEvidence.buttonFound !== true ||
    !String(outlineNavigationEvidence.buttonLabel || "").includes("Native Outline Target") ||
    !Number.isFinite(outlineNavigationEvidence.targetLine) ||
    outlineNavigationEvidence.targetLine < 1 ||
    outlineNavigationEvidence.selectedLine !== outlineNavigationEvidence.targetLine ||
    !String(outlineNavigationEvidence.selectedText || "").includes("## Native Outline Target") ||
    !String(outlineNavigationEvidence.sidebarText || "").includes("Native Outline Target")
  ) {
    issues.push(`native workflow report did not include outline navigation evidence: ${JSON.stringify(outlineNavigationEvidence)}`);
  }
  const snapshotEvidence = payload.snapshotEvidence || {};
  if (
    snapshotEvidence.appData?.created?.storage !== "app-data" ||
    snapshotEvidence.appData?.created?.listed !== true ||
    snapshotEvidence.appData?.created?.label !== "native-smoke" ||
    !String(snapshotEvidence.appData?.created?.snapshotPath || "").endsWith(".md") ||
    snapshotEvidence.appData?.restored?.containsMutation !== false ||
    !String(snapshotEvidence.appData?.restored?.restoredText || "").includes("Market Entry Report")
  ) {
    issues.push(`native workflow report did not include app-data snapshot restore evidence: ${JSON.stringify(snapshotEvidence)}`);
  }
  const projectLocalSnapshotPath = String(snapshotEvidence.projectLocal?.created?.snapshotPath || "").replaceAll("\\", "/");
  if (
    snapshotEvidence.projectLocal?.created?.storage !== "project-local" ||
    snapshotEvidence.projectLocal?.created?.listed !== true ||
    snapshotEvidence.projectLocal?.created?.label !== "native-project-smoke" ||
    !projectLocalSnapshotPath.includes("/.neditor/snapshots/") ||
    !projectLocalSnapshotPath.endsWith(".md") ||
    snapshotEvidence.projectLocal?.restored?.containsMutation !== false ||
    !String(snapshotEvidence.projectLocal?.restored?.restoredText || "").includes("Market Entry Report")
  ) {
    issues.push(`native workflow report did not include project-local snapshot restore evidence: ${JSON.stringify(snapshotEvidence)}`);
  }
  const exportProfileEvidence = payload.exportProfileEvidence || {};
  if (
    exportProfileEvidence.saved?.exportTarget !== "pdf" ||
    exportProfileEvidence.saved?.brandProfileDefaults?.name !== "Native Board" ||
    exportProfileEvidence.applied?.target !== "pdf" ||
    exportProfileEvidence.applied?.layoutPreset !== "compact" ||
    exportProfileEvidence.applied?.includeManifest !== false ||
    exportProfileEvidence.applied?.citationStyle !== "ieee" ||
    exportProfileEvidence.reloaded?.activeExportProfileId !== exportProfileEvidence.saved?.id ||
    exportProfileEvidence.reloaded?.target !== "pdf" ||
    exportProfileEvidence.reloaded?.layoutPreset !== "compact" ||
    exportProfileEvidence.reloaded?.brandName !== "Native Board"
  ) {
    issues.push(`native workflow report did not include export profile persistence evidence: ${JSON.stringify(exportProfileEvidence)}`);
  }
  const nativeMenuCommandEvidence = payload.nativeMenuCommandEvidence || {};
  if (
    nativeMenuCommandEvidence.exportMode?.mode !== "export" ||
    nativeMenuCommandEvidence.exportMode?.sidebar !== "exports" ||
    nativeMenuCommandEvidence.outline?.sidebar !== "outline" ||
    nativeMenuCommandEvidence.exports?.sidebar !== "exports" ||
    nativeMenuCommandEvidence.search?.open !== true ||
    nativeMenuCommandEvidence.toc?.inserted !== true ||
    nativeMenuCommandEvidence.equation?.inserted !== true ||
    nativeMenuCommandEvidence.codeFence?.inserted !== true ||
    nativeMenuCommandEvidence.table?.inserted !== true ||
    nativeMenuCommandEvidence.templates?.sidebar !== "templates" ||
    nativeMenuCommandEvidence.docsLive?.open !== true ||
    nativeMenuCommandEvidence.aiPaste?.open !== true
  ) {
    issues.push(`native workflow report did not include native menu command evidence: ${JSON.stringify(nativeMenuCommandEvidence)}`);
  }
  const workspaceTabEvidence = payload.workspaceTabEvidence || {};
  const workspacePaths = {
    boardOne: String(workspaceTabEvidence.boardOnePath || "").replaceAll("\\", "/"),
    boardTwo: String(workspaceTabEvidence.boardTwoPath || "").replaceAll("\\", "/"),
    loose: String(workspaceTabEvidence.loosePath || "").replaceAll("\\", "/"),
  };
  if (
    workspaceTabEvidence.initialBoardGroup?.key !== "set:Native Board Pack" ||
    workspaceTabEvidence.initialBoardGroup?.count < 2 ||
    !Array.isArray(workspaceTabEvidence.pinnedGroup?.paths) ||
    !workspaceTabEvidence.pinnedGroup.paths.map((path) => String(path).replaceAll("\\", "/")).includes(workspacePaths.boardOne) ||
    workspaceTabEvidence.looseAssigned?.textHasDocumentSet !== true ||
    workspaceTabEvidence.looseAssigned?.saved !== true ||
    !Array.isArray(workspaceTabEvidence.closeGroup?.closedPaths) ||
    workspaceTabEvidence.closeGroup.closedPaths.length < 2 ||
    workspaceTabEvidence.closeGroup.closedPaths.map((path) => String(path).replaceAll("\\", "/")).some((path) => !path.includes("native-workspace-")) ||
    String(workspaceTabEvidence.recentReopen?.activePath || "").replaceAll("\\", "/") !== workspacePaths.boardTwo ||
    String(workspaceTabEvidence.restore?.activePath || "").replaceAll("\\", "/") !== workspacePaths.boardTwo ||
    String(workspaceTabEvidence.restore?.pinnedPath || "").replaceAll("\\", "/") !== workspacePaths.boardOne ||
    workspaceTabEvidence.restore?.pinned !== true ||
    Math.abs(Number(workspaceTabEvidence.restore?.editorScrollRatio || 0) - 0.42) > 0.001 ||
    Math.abs(Number(workspaceTabEvidence.restore?.previewScrollRatio || 0) - 0.58) > 0.001
  ) {
    issues.push(`native workflow report did not include workspace tab evidence: ${JSON.stringify(workspaceTabEvidence)}`);
  }
  if (payload.exportReadiness?.target !== "html" || !Array.isArray(payload.exportReadiness?.progressSteps)) {
    issues.push(`native workflow report did not include HTML export readiness evidence: ${JSON.stringify(payload.exportReadiness)}`);
  }
  const exportResult = payload.exportResult || {};
  const nativeMenuExportResult = payload.nativeMenuExportResult || {};
  const expectedExportPath = nativeWorkflowExportPath.replaceAll("\\", "/");
  if (
    exportResult.target !== "html" ||
    String(exportResult.outputPath || "").replaceAll("\\", "/") !== expectedExportPath ||
    !Array.isArray(exportResult.progressSteps) ||
    !exportResult.progressSteps.includes("render")
  ) {
    issues.push(`native workflow report did not include HTML export write evidence: ${JSON.stringify(exportResult)}`);
  }
  if (
    nativeMenuExportResult.target !== "html" ||
    nativeMenuExportResult.sidebar !== "exports" ||
    String(nativeMenuExportResult.outputPath || "").replaceAll("\\", "/") !== expectedExportPath ||
    !Array.isArray(nativeMenuExportResult.progressSteps) ||
    !nativeMenuExportResult.progressSteps.some((step) => String(step).startsWith("render:complete"))
  ) {
    issues.push(`native workflow report did not include native-menu HTML export evidence: ${JSON.stringify(nativeMenuExportResult)}`);
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
