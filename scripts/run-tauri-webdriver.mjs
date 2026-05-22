import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const serverUrl = process.env.NEDITOR_TAURI_WEBDRIVER_URL || "http://127.0.0.1:4444";
const required = process.argv.includes("--strict") || process.env.NEDITOR_TAURI_WEBDRIVER_REQUIRED === "1";
const timeoutMs = Number(process.env.NEDITOR_TAURI_WEBDRIVER_TIMEOUT_MS || 30_000);
const application = desktopBinaryPath();
const reportPath = join(root, ".tmp", "desktop-webdriver", "report.json");
const workflowReportPath = join(root, ".tmp", "desktop-webdriver", "native-workflow-report.json");
const workflowExportPath = join(root, ".tmp", "desktop-webdriver", "native-workflow-export.html");
const workflowExportManifestPath = `${workflowExportPath}.manifest.json`;
const macosUnsupportedMessage =
  "Official Tauri WebDriver currently supports desktop automation on Windows and Linux only; macOS has no WKWebView driver in that stack.";
const webdriverWorkflowPlan = [
  "initial native title includes NEditor",
  "desktop shell renders primary commands",
  "native WebDriver switches modes and opens command palette",
  "native title exposes dirty document state",
  "desktop template insertion reaches editor and preview",
  "desktop export readiness returns manifest progress evidence",
  "desktop WebDriver writes HTML export through dialog-free smoke path",
  "desktop preferences persist across WebDriver restart",
];
const report = {
  generatedAt: new Date().toISOString(),
  platform: process.platform,
  arch: process.arch,
  application: relative(application),
  serverUrl,
  timeoutMs,
  required,
  status: "pending",
  supportedDesktopPlatforms: ["linux", "win32"],
  workflowPlan: webdriverWorkflowPlan,
  workflowSmokeReport: relative(workflowReportPath),
  dependencies: [],
  assertions: [],
  exportArtifacts: null,
  skippedReason: null,
  fallback: null,
};
let tauriDriver = null;

const controlByLabelScript = `function controlByLabel(labelText, selector) {
  const normalized = (value) => value.replace(/\\s+/g, ' ').trim();
  const label = [...document.querySelectorAll('label')].find((item) => normalized(item.textContent || '').includes(labelText));
  const control = label?.querySelector(selector);
  if (!control) throw new Error('Missing control for ' + labelText);
  return control;
}`;

const readPreferenceScript = `function readPreferences() {
  const controlByLabel = ${controlByLabelScript};
  return {
    theme: controlByLabel('Theme', 'select').value,
    previewTheme: controlByLabel('Preview theme', 'select').value,
    wordWrap: controlByLabel('Word wrap', 'input').checked,
    lineNumbers: controlByLabel('Line numbers', 'input').checked,
    highContrast: controlByLabel('High contrast', 'input').checked,
    reducedMotion: controlByLabel('Reduced motion', 'input').checked,
  };
}`;

if (!existsSync(application) || !statSync(application).isFile()) {
  fail(`desktop binary is missing: ${relative(application)}. Run ./node_modules/.bin/tauri build --no-bundle first.`);
}

if (process.platform === "darwin") {
  const message = `${macosUnsupportedMessage} Use NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke for the bounded macOS GUI launch smoke.`;
  report.status = required ? "failed" : "skipped";
  report.skippedReason = macosUnsupportedMessage;
  report.fallback = "NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke";
  writeReport();
  if (required) fail(message);
  console.log(`Skipped Tauri WebDriver smoke on macOS. ${message}`);
  process.exit(0);
}

if (!["linux", "win32"].includes(process.platform)) {
  const message = `Tauri WebDriver smoke is not configured for ${process.platform}.`;
  report.status = required ? "failed" : "skipped";
  report.skippedReason = message;
  writeReport();
  if (required) fail(message);
  console.log(`Skipped Tauri WebDriver smoke. ${message}`);
  process.exit(0);
}

requireCommand("tauri-driver", "Install with: cargo install tauri-driver --locked");
if (process.platform === "linux") {
  requireCommand("WebKitWebDriver", "Install the WebKitGTK WebDriver package, for example webkit2gtk-driver on Debian/Ubuntu.");
}
if (process.platform === "win32") {
  requireCommand("msedgedriver", "Install Microsoft Edge WebDriver and ensure msedgedriver.exe is on PATH.");
}

try {
  await runWebDriverSmoke();
  report.status = "passed";
  writeReport();
  console.log("Tauri WebDriver smoke passed against the built NEditor desktop binary.");
} finally {
  tauriDriver?.kill();
}

async function runWebDriverSmoke() {
  rmSync(workflowExportPath, { force: true });
  rmSync(workflowExportManifestPath, { force: true });
  rmSync(workflowReportPath, { force: true });

  tauriDriver = spawn("tauri-driver", [], {
    cwd: root,
    env: {
      ...process.env,
      NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT: workflowReportPath,
    },
    stdio: ["ignore", "pipe", "pipe"],
  });

  let driverOutput = "";
  tauriDriver.stdout.on("data", (chunk) => {
    driverOutput += chunk.toString();
  });
  tauriDriver.stderr.on("data", (chunk) => {
    driverOutput += chunk.toString();
  });
  tauriDriver.on("exit", (code, signal) => {
    if (code !== null && code !== 0) {
      driverOutput += `\ntauri-driver exited with code ${code}`;
    } else if (signal) {
      driverOutput += `\ntauri-driver exited with signal ${signal}`;
    }
  });

  await waitForDriver();
  let session = await createSession();
  let originalPreferences = null;
  try {
    await assertInitialShell(session);
    await assertModeSwitchAndCommandPalette(session);
    await assertDirtyTitleWorkflow(session);
    await assertTransformTemplateWorkflow(session);
    await assertExportReadinessWorkflow(session);
    await assertHtmlExportWriteWorkflow(session);
    originalPreferences = await readDesktopPreferences(session);
    session = await assertPreferenceRestartWorkflow(session, originalPreferences);
  } finally {
    if (session && originalPreferences) {
      await restoreDesktopPreferences(session, originalPreferences).catch(() => undefined);
    }
    if (session) {
      await webdriver("DELETE", `/session/${session}`).catch(() => undefined);
    }
  }

  function driverFailureDetail() {
    return driverOutput.trim() ? `\n${driverOutput.trim()}` : "";
  }

  async function waitForDriver() {
    const started = Date.now();
    while (Date.now() - started < timeoutMs) {
      if (tauriDriver.exitCode !== null) {
        throw new Error(`tauri-driver exited before accepting sessions.${driverFailureDetail()}`);
      }
      try {
        await webdriver("GET", "/status", undefined, 1000);
        return;
      } catch {
        await delay(250);
      }
    }
    throw new Error(`timed out waiting for tauri-driver at ${serverUrl}.${driverFailureDetail()}`);
  }
}

async function assertInitialShell(session) {
  const title = await webdriver("GET", `/session/${session}/title`);
  if (!String(title.value || "").includes("NEditor")) {
    throw new Error(`expected native window title to include NEditor, found ${JSON.stringify(title.value)}`);
  }
  recordAssertion("initial native title includes NEditor");

  const shell = await findElement(session, ".app-shell");
  const shellText = await elementText(session, shell);
  for (const expected of ["New", "Open", "Save", "Commands"]) {
    if (!shellText.includes(expected)) {
      throw new Error(`desktop shell did not render expected command ${expected}`);
    }
  }
  recordAssertion("desktop shell renders primary commands");
}

async function assertModeSwitchAndCommandPalette(session) {
  const mode = await execute(session, `
    const select = document.querySelector('[aria-label="View mode"]');
    select.value = 'preview';
    select.dispatchEvent(new Event('change', { bubbles: true }));
    return document.querySelector('.workspace')?.className || '';
  `);
  if (!String(mode.value || "").includes("mode-preview")) {
    throw new Error(`expected WebDriver mode switch to reach preview mode, found ${JSON.stringify(mode.value)}`);
  }

  const palette = await execute(session, `
    const button = [...document.querySelectorAll('button')].find((item) => item.textContent.trim() === 'Commands');
    button.click();
    return document.querySelector('[role="dialog"][aria-label="Command palette"] input')?.getAttribute('aria-label') || '';
  `);
  if (!String(palette.value || "").includes("Search commands")) {
    throw new Error("command palette did not open through the native WebDriver session");
  }
  recordAssertion("native WebDriver switches modes and opens command palette");
}

async function assertDirtyTitleWorkflow(session) {
  const dirtyTitle = await waitForValue(
    session,
    `
      const newButton = [...document.querySelectorAll('button')].find((item) => item.textContent.trim() === 'New');
      newButton.click();
      return {
        title: document.title,
        tab: document.querySelector('.document-tabs .tab.active')?.textContent || '',
      };
    `,
    (value) => String(value?.title || "").startsWith("* ") && String(value?.tab || "").includes("*Untitled"),
    "dirty document title marker",
  );
  if (!String(dirtyTitle.title || "").includes("NEditor")) {
    throw new Error(`dirty title did not include the application name: ${JSON.stringify(dirtyTitle)}`);
  }

  const nativeTitle = await webdriver("GET", `/session/${session}/title`);
  if (!String(nativeTitle.value || "").startsWith("* ")) {
    throw new Error(`native title did not expose dirty state: ${JSON.stringify(nativeTitle.value)}`);
  }
  recordAssertion("native title exposes dirty document state");
}

async function assertTransformTemplateWorkflow(session) {
  await showSidebar(session, "templates", "Custom template");
  await execute(session, `
    const normalized = (value) => value.replace(/\\s+/g, ' ').trim();
    const controlByLabel = ${controlByLabelScript};
    const category = controlByLabel('Category', 'select');
    category.value = 'Science';
    category.dispatchEvent(new Event('change', { bubbles: true }));
    const transform = controlByLabel('Transform', 'select');
    transform.value = 'calc';
    transform.dispatchEvent(new Event('change', { bubbles: true }));
    const search = controlByLabel('Search', 'input');
    search.value = 'dose';
    search.dispatchEvent(new Event('input', { bubbles: true }));
    const template = [...document.querySelectorAll('.template-card')].find((item) => normalized(item.textContent || '').includes('Dose by weight'));
    if (!template) throw new Error('Dose by weight template was not visible in the desktop template panel');
    const preview = template.querySelector('details');
    if (preview && !preview.open) preview.querySelector('summary')?.click();
    const insert = [...template.querySelectorAll('button')].find((item) => normalized(item.textContent || '') === 'Insert');
    if (!insert) throw new Error('Dose by weight template did not expose an Insert button');
    insert.click();
    return true;
  `);
  const inserted = await waitForValue(
    session,
    `
      return {
        editor: document.querySelector('.cm-content')?.textContent || '',
        preview: document.querySelector('.preview-document')?.textContent || '',
        status: document.querySelector('.status-bar')?.textContent || '',
      };
    `,
    (value) =>
      String(value?.editor || "").includes("weight_kg = 72") &&
      String(value?.editor || "").includes("total_dose_mg") &&
      String(value?.preview || "").includes("Total dose") &&
      String(value?.status || "").includes("Inserted Dose by weight template"),
    "template insertion in editor and preview",
  );
  if (!String(inserted.preview || "").includes("mg")) {
    throw new Error(`desktop preview did not render the inserted calculation output: ${JSON.stringify(inserted)}`);
  }
  recordAssertion("desktop template insertion reaches editor and preview");
}

async function assertExportReadinessWorkflow(session) {
  await showSidebar(session, "exports", "Target");
  await execute(session, `
    const controlByLabel = ${controlByLabelScript};
    const target = controlByLabel('Target', 'select');
    target.value = 'html';
    target.dispatchEvent(new Event('change', { bubbles: true }));
    const prepare = [...document.querySelectorAll('button')].find((item) => item.textContent.trim() === 'Prepare for export');
    prepare.click();
    return true;
  `);
  const readiness = await waitForValue(
    session,
    `
      return {
        status: document.querySelector('article.readiness strong')?.textContent || '',
        manifest: document.querySelector('.sidebar pre')?.textContent || '',
        progress: [...document.querySelectorAll('[aria-label="Export readiness progress"] li')].map((item) => item.textContent.trim()),
      };
    `,
    (value) =>
      ["Ready", "Needs attention"].includes(String(value?.status || "")) &&
      String(value?.manifest || "").includes('"export_target": "html"') &&
      Array.isArray(value?.progress) &&
      value.progress.length > 0,
    "export readiness result",
  );
  if (!String(readiness.manifest || "").includes('"progress_steps"')) {
    throw new Error(`export readiness manifest did not include progress evidence: ${JSON.stringify(readiness)}`);
  }
  recordAssertion("desktop export readiness returns manifest progress evidence");
}

async function assertHtmlExportWriteWorkflow(session) {
  await showSidebar(session, "exports", "Target");
  await execute(session, `
    const controlByLabel = ${controlByLabelScript};
    const target = controlByLabel('Target', 'select');
    target.value = 'html';
    target.dispatchEvent(new Event('change', { bubbles: true }));
    const normalized = (value) => value.replace(/\\s+/g, ' ').trim();
    const exportButton = [...document.querySelectorAll('button')].find((item) => normalized(item.textContent || '') === 'Export HTML');
    if (!exportButton) throw new Error('Export HTML button was not visible in the desktop export panel');
    exportButton.click();
    return true;
  `);
  const expectedOutput = workflowExportPath.replaceAll("\\", "/");
  const expectedManifest = workflowExportManifestPath.replaceAll("\\", "/");
  const result = await waitForValue(
    session,
    `
      return {
        output: document.querySelector('.export-result')?.textContent || '',
        status: document.querySelector('.status-bar')?.textContent || '',
        progress: [...document.querySelectorAll('[aria-label="Last export progress"] li')].map((item) => item.textContent.trim()),
        diagnostics: [...document.querySelectorAll('.export-result .diagnostic')].map((item) => item.textContent.trim()),
      };
    `,
    (value) =>
      String(value?.output || "").replaceAll("\\", "/").includes(expectedOutput) &&
      String(value?.output || "").replaceAll("\\", "/").includes(expectedManifest) &&
      Array.isArray(value?.progress) &&
      value.progress.some((step) => step.includes("Render") && step.includes("complete")) &&
      !String(value?.output || "").includes("error"),
    "written HTML export result",
  );

  if (!existsSync(workflowExportPath)) {
    throw new Error(`desktop WebDriver HTML export artifact was not written: ${relative(workflowExportPath)}`);
  }
  if (!existsSync(workflowExportManifestPath)) {
    throw new Error(`desktop WebDriver HTML export manifest was not written: ${relative(workflowExportManifestPath)}`);
  }
  const html = readFileSync(workflowExportPath, "utf8");
  if (!html.includes("Market Entry Report") || !html.includes("Total dose")) {
    throw new Error(`desktop WebDriver HTML export did not include expected document/template content: ${relative(workflowExportPath)}`);
  }
  const manifest = JSON.parse(readFileSync(workflowExportManifestPath, "utf8"));
  if (manifest.export_target !== "html" || !manifest.output_hash) {
    throw new Error(`desktop WebDriver HTML export manifest did not include target/hash evidence: ${JSON.stringify(manifest)}`);
  }
  report.exportArtifacts = {
    outputPath: relative(workflowExportPath),
    manifestPath: relative(workflowExportManifestPath),
    outputBytes: statSync(workflowExportPath).size,
    manifestBytes: statSync(workflowExportManifestPath).size,
    target: manifest.export_target,
    outputHash: manifest.output_hash,
    progressEvidence: result.progress,
  };
  recordAssertion("desktop WebDriver writes HTML export through dialog-free smoke path");
}

async function assertPreferenceRestartWorkflow(session, originalPreferences) {
  const targetPreferences = {
    theme: originalPreferences.theme === "dark" ? "light" : "dark",
    previewTheme: "dark",
    wordWrap: !originalPreferences.wordWrap,
    lineNumbers: !originalPreferences.lineNumbers,
    highContrast: !originalPreferences.highContrast,
    reducedMotion: !originalPreferences.reducedMotion,
  };
  await applyDesktopPreferences(session, targetPreferences);
  await saveWorkspace(session);
  await delay(500);
  await webdriver("DELETE", `/session/${session}`).catch(() => undefined);

  const restartedSession = await createSession();
  try {
    await showSidebar(restartedSession, "settings", "Word wrap");
    await waitForValue(
      restartedSession,
      `
        const read = ${readPreferenceScript};
        return read();
      `,
      (value) => preferencesMatch(value, targetPreferences),
      "persisted desktop preferences after restart",
    );
    recordAssertion("desktop preferences persist across WebDriver restart");
    return restartedSession;
  } catch (error) {
    await webdriver("DELETE", `/session/${restartedSession}`).catch(() => undefined);
    throw error;
  }
}

async function readDesktopPreferences(session) {
  await showSidebar(session, "settings", "Word wrap");
  const response = await execute(session, `
    const read = ${readPreferenceScript};
    return read();
  `);
  return response.value;
}

async function restoreDesktopPreferences(session, preferences) {
  await applyDesktopPreferences(session, preferences);
  await saveWorkspace(session);
  await delay(250);
}

async function applyDesktopPreferences(session, preferences) {
  await showSidebar(session, "settings", "Word wrap");
  await execute(session, `
    const prefs = ${JSON.stringify(preferences)};
    const controlByLabel = ${controlByLabelScript};
    const setSelect = (labelText, value) => {
      const select = controlByLabel(labelText, 'select');
      select.value = value;
      select.dispatchEvent(new Event('change', { bubbles: true }));
    };
    const setCheckbox = (labelText, checked) => {
      const input = controlByLabel(labelText, 'input');
      if (input.checked !== checked) {
        input.checked = checked;
        input.dispatchEvent(new Event('input', { bubbles: true }));
        input.dispatchEvent(new Event('change', { bubbles: true }));
      }
    };
    setSelect('Theme', prefs.theme);
    setSelect('Preview theme', prefs.previewTheme);
    setCheckbox('Word wrap', prefs.wordWrap);
    setCheckbox('Line numbers', prefs.lineNumbers);
    setCheckbox('High contrast', prefs.highContrast);
    setCheckbox('Reduced motion', prefs.reducedMotion);
    return true;
  `);
}

async function showSidebar(session, value, labelText) {
  await execute(session, `
    const sidebar = document.querySelector('[aria-label="Sidebar panel"]');
    sidebar.value = ${JSON.stringify(value)};
    sidebar.dispatchEvent(new Event('change', { bubbles: true }));
    return true;
  `);
  await waitForValue(
    session,
    `
      const normalized = (value) => value.replace(/\\s+/g, ' ').trim();
      return [...document.querySelectorAll('label')].some((label) => normalized(label.textContent || '').includes(${JSON.stringify(labelText)}));
    `,
    Boolean,
    `${value} sidebar controls`,
  );
}

async function saveWorkspace(session) {
  await execute(session, `
    const button = [...document.querySelectorAll('button')].find((item) => item.textContent.trim() === 'Save Workspace');
    button.click();
    return true;
  `);
}

async function waitForValue(session, script, predicate, description) {
  const started = Date.now();
  let lastValue = null;
  while (Date.now() - started < timeoutMs) {
    const response = await execute(session, script);
    lastValue = response.value;
    if (predicate(lastValue)) return lastValue;
    await delay(250);
  }
  throw new Error(`timed out waiting for ${description}; last value: ${JSON.stringify(lastValue)}`);
}

function preferencesMatch(actual, expected) {
  return (
    actual?.theme === expected.theme &&
    actual?.previewTheme === expected.previewTheme &&
    actual?.wordWrap === expected.wordWrap &&
    actual?.lineNumbers === expected.lineNumbers &&
    actual?.highContrast === expected.highContrast &&
    actual?.reducedMotion === expected.reducedMotion
  );
}

async function createSession() {
  const response = await webdriver("POST", "/session", {
    capabilities: {
      alwaysMatch: {
        browserName: "wry",
        "tauri:options": {
          application,
        },
      },
    },
  });
  const session = response.value?.sessionId || response.sessionId;
  if (!session) {
    throw new Error(`tauri-driver did not return a session id: ${JSON.stringify(response)}`);
  }
  return session;
}

async function findElement(session, selector) {
  const response = await webdriver("POST", `/session/${session}/element`, {
    using: "css selector",
    value: selector,
  });
  const id = elementId(response.value);
  if (!id) throw new Error(`could not find element ${selector}`);
  return id;
}

async function elementText(session, id) {
  const response = await webdriver("GET", `/session/${session}/element/${id}/text`);
  return String(response.value || "");
}

async function execute(session, script) {
  return webdriver("POST", `/session/${session}/execute/sync`, {
    script,
    args: [],
  });
}

async function webdriver(method, path, body, requestTimeoutMs = timeoutMs) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), requestTimeoutMs);
  try {
    const response = await fetch(`${serverUrl}${path}`, {
      method,
      body: body === undefined ? undefined : JSON.stringify(body),
      headers: body === undefined ? undefined : { "content-type": "application/json" },
      signal: controller.signal,
    });
    const text = await response.text();
    const payload = text ? JSON.parse(text) : {};
    if (!response.ok) {
      throw new Error(`${method} ${path} failed with ${response.status}: ${text}`);
    }
    return payload;
  } finally {
    clearTimeout(timeout);
  }
}

function elementId(value) {
  return value?.["element-6066-11e4-a52e-4f735466cecf"] || value?.ELEMENT;
}

function requireCommand(cmd, installHint) {
  const lookup =
    process.platform === "win32"
      ? spawnSync("where", [cmd], { encoding: "utf8", shell: true })
      : spawnSync("sh", ["-c", `command -v ${shellQuote(cmd)}`], { encoding: "utf8" });
  if (lookup.status !== 0) {
    report.dependencies.push({ command: cmd, status: "missing", installHint });
    report.status = "failed";
    writeReport();
    fail(`${cmd} is required for Tauri WebDriver smoke on ${process.platform}. ${installHint}`);
  }
  report.dependencies.push({ command: cmd, status: "found", path: firstLine(`${lookup.stdout}${lookup.stderr}`) || cmd });
}

function shellQuote(value) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function desktopBinaryPath() {
  const name = process.platform === "win32" ? "neditor.exe" : "neditor";
  return join(root, "src-tauri", "target", "release", name);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}

function delay(ms) {
  return new Promise((resolveDelay) => setTimeout(resolveDelay, ms));
}

function fail(message) {
  if (report.status === "pending") {
    report.status = "failed";
    report.error = message;
    writeReport();
  }
  console.error(message);
  process.exit(1);
}

function recordAssertion(name) {
  report.assertions.push({
    name,
    status: "passed",
    elapsedMs: Date.now() - Date.parse(report.generatedAt),
  });
}

function writeReport() {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

function firstLine(text) {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find(Boolean);
}
