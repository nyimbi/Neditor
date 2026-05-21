import { spawn, spawnSync } from "node:child_process";
import { existsSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const serverUrl = process.env.NEDITOR_TAURI_WEBDRIVER_URL || "http://127.0.0.1:4444";
const required = process.argv.includes("--strict") || process.env.NEDITOR_TAURI_WEBDRIVER_REQUIRED === "1";
const timeoutMs = Number(process.env.NEDITOR_TAURI_WEBDRIVER_TIMEOUT_MS || 30_000);
const application = desktopBinaryPath();
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
  const message =
    "Official Tauri WebDriver currently supports desktop automation on Windows and Linux only; macOS has no WKWebView driver in that stack. Use NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke for the bounded macOS GUI launch smoke.";
  if (required) fail(message);
  console.log(`Skipped Tauri WebDriver smoke on macOS. ${message}`);
  process.exit(0);
}

if (!["linux", "win32"].includes(process.platform)) {
  const message = `Tauri WebDriver smoke is not configured for ${process.platform}.`;
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
  console.log("Tauri WebDriver smoke passed against the built NEditor desktop binary.");
} finally {
  tauriDriver?.kill();
}

async function runWebDriverSmoke() {
  tauriDriver = spawn("tauri-driver", [], {
    cwd: root,
    env: process.env,
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
    await assertExportReadinessWorkflow(session);
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

  const shell = await findElement(session, ".app-shell");
  const shellText = await elementText(session, shell);
  for (const expected of ["New", "Open", "Save", "Commands"]) {
    if (!shellText.includes(expected)) {
      throw new Error(`desktop shell did not render expected command ${expected}`);
    }
  }
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
    fail(`${cmd} is required for Tauri WebDriver smoke on ${process.platform}. ${installHint}`);
  }
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
  console.error(message);
  process.exit(1);
}
