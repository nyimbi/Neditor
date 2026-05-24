import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const serverUrl = process.env.NEDITOR_TAURI_WEBDRIVER_URL || "http://127.0.0.1:4444";
const required = process.argv.includes("--strict") || process.env.NEDITOR_TAURI_WEBDRIVER_REQUIRED === "1";
const timeoutMs = Number(process.env.NEDITOR_TAURI_WEBDRIVER_TIMEOUT_MS || 30_000);
const application = desktopBinaryPath();
const reportPath = join(root, ".tmp", "desktop-webdriver", "report.json");
const workflowReportPath = join(root, ".tmp", "desktop-webdriver", "native-workflow-report.json");
const workflowFilePath = join(root, ".tmp", "desktop-webdriver", "native-workflow-file.md");
const workflowRenamedPath = join(root, ".tmp", "desktop-webdriver", "native-workflow-renamed.md");
const workflowDuplicatePath = join(root, ".tmp", "desktop-webdriver", "native-workflow-duplicate.md");
const workflowExportPath = join(root, ".tmp", "desktop-webdriver", "native-workflow-export.html");
const workflowExportManifestPath = `${workflowExportPath}.manifest.json`;
const macosFallbackSmokeReportPath = join(root, ".tmp", "desktop-smoke", "native-command-report.json");
const macosFallbackLaunchReportPath = join(root, ".tmp", "desktop-smoke", "launch-report.json");
const macosUnsupportedMessage =
  "Official Tauri WebDriver currently supports desktop automation on Windows and Linux only; macOS has no WKWebView driver in that stack.";
const webdriverWorkflowPlan = [
  "initial native title includes NEditor",
  "desktop shell renders primary commands",
  "native WebDriver switches modes and opens command palette",
  "desktop WebDriver edits document structure in outline mode",
  "native title exposes dirty document state",
  "desktop WebDriver saves and reopens real Markdown file through dialog-free smoke path",
  "desktop WebDriver renames, duplicates, and exposes reveal affordance for real Markdown files",
  "desktop export readiness returns manifest progress evidence",
  "desktop WebDriver writes HTML export through dialog-free smoke path",
  "desktop preferences apply in packaged WebDriver session",
];
const report = {
  generatedAt: new Date().toISOString(),
  platform: process.platform,
  arch: process.arch,
  appVersion: packageJson.version,
  sourceCommit: gitCommit(),
  sourceTreeClean: gitTreeClean(),
  application: relative(application),
  serverUrl,
  timeoutMs,
  required,
  status: "pending",
  supportedDesktopPlatforms: ["linux", "win32"],
  workflowPlan: webdriverWorkflowPlan,
  workflowSmokeReport: relative(workflowReportPath),
  workflowFilePath: relative(workflowFilePath),
  dependencies: [],
  assertions: [],
  outlineArtifacts: null,
  fileArtifacts: null,
  exportArtifacts: null,
  preferenceArtifacts: null,
  skippedReason: null,
  fallback: null,
  fallbackProof: null,
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

const outlineModeEvidenceScript = `
  const visible = (element) => Boolean(element && element.offsetParent !== null);
  const workspace = document.querySelector('.workspace');
  const outline = document.querySelector('#outline-mode');
  const rows = [...document.querySelectorAll('#outline-mode .outline-mode-row')];
  const titles = rows.map((row) => row.querySelector('input')?.value || '').filter(Boolean);
  const levels = Object.fromEntries(rows.map((row) => [row.querySelector('input')?.value || '', row.querySelector('select')?.value || '']).filter(([title]) => title));
  return {
    mode: [...(workspace?.classList || [])].find((name) => name.startsWith('mode-')) || '',
    outlineVisible: visible(outline),
    sourceVisible: visible(document.querySelector('#markdown-source')),
    previewVisible: visible(document.querySelector('#live-preview')),
    outlineText: outline?.textContent?.replace(/\\s+/g, ' ').trim() || '',
    titles,
    levels,
  };
`;

const editorDocumentTextFunction = `
  function editorDocumentText() {
    const candidates = [...document.querySelectorAll('.cm-editor, .cm-content')];
    for (const node of candidates) {
      let cmView = node.cmView;
      const seen = new Set();
      while (cmView && !seen.has(cmView)) {
        seen.add(cmView);
        if (cmView.view?.state?.doc) return cmView.view.state.doc.toString();
        cmView = cmView.parent;
      }
    }
    return document.querySelector('.cm-content')?.textContent || '';
  }
`;

if (!existsSync(application) || !statSync(application).isFile()) {
  fail(`desktop binary is missing: ${relative(application)}. Run ./node_modules/.bin/tauri build --no-bundle first.`);
}

if (process.platform === "darwin") {
  const message = `${macosUnsupportedMessage} Use NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke for the bounded macOS GUI launch smoke.`;
  const fallbackProof = collectMacosNativeProof();
  report.status = required ? "failed" : "skipped";
  report.skippedReason = macosUnsupportedMessage;
  report.fallback = "NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke";
  report.fallbackProof = fallbackProof;
  writeReport();
  if (required) fail(message);
  console.log(
    fallbackProof.status === "passed"
      ? `Skipped Tauri WebDriver smoke on macOS. Native launch fallback proof is current in ${relative(macosFallbackSmokeReportPath)}.`
      : `Skipped Tauri WebDriver smoke on macOS. ${message}`,
  );
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
  rmSync(workflowFilePath, { force: true });
  rmSync(workflowRenamedPath, { force: true });
  rmSync(workflowDuplicatePath, { force: true });
  rmSync(workflowExportPath, { force: true });
  rmSync(workflowExportManifestPath, { force: true });
  rmSync(workflowReportPath, { force: true });

  tauriDriver = spawn("tauri-driver", [], {
    cwd: root,
    env: {
      ...process.env,
      NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT: workflowReportPath,
      NEDITOR_DESKTOP_WORKFLOW_DISABLE_AUTORUN: "1",
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
    await assertOutlineModeWorkflow(session);
    await assertDirtyTitleWorkflow(session);
    await assertFileSaveOpenWorkflow(session);
    await assertRenameDuplicateRevealWorkflow(session);
    await assertExportReadinessWorkflow(session);
    await assertHtmlExportWriteWorkflow(session);
    originalPreferences = await readDesktopPreferences(session);
    await assertPreferenceWorkflow(session, originalPreferences);
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

  await waitForValue(
    session,
    "return document.querySelector('.app-shell')?.textContent || '';",
    (value) => ["New", "Open", "Save", "Commands"].every((expected) => String(value || "").includes(expected)),
    "desktop shell primary commands",
  );
  recordAssertion("desktop shell renders primary commands");
}

async function assertModeSwitchAndCommandPalette(session) {
  await setViewMode(session, "preview", "WebDriver mode switch to preview mode");

  await execute(session, `
    const normalized = (value) => String(value || '').replace(/\\s+/g, ' ').trim();
    const button = [...document.querySelectorAll('button')].find((item) =>
      normalized(item.textContent).includes('Commands') ||
      normalized(item.getAttribute('aria-label')).includes('Commands') ||
      normalized(item.getAttribute('title')).includes('Commands')
    );
    if (!button) throw new Error('Commands button was not visible');
    button.click();
    return true;
  `);
  await waitForValue(
    session,
    `
      return document.querySelector('[role="dialog"][aria-label="Command palette"] input')?.getAttribute('aria-label') || '';
    `,
    (value) => String(value || "").includes("Search commands"),
    "native command palette input",
  );
  await closeCommandPalette(session);
  recordAssertion("native WebDriver switches modes and opens command palette");
}

async function assertOutlineModeWorkflow(session) {
  await setViewMode(session, "outline", "desktop outline mode selection");
  const initial = await waitForValue(
    session,
    outlineModeEvidenceScript,
    (value) =>
      value?.mode === "mode-outline" &&
      value?.outlineVisible === true &&
      value?.sourceVisible === false &&
      value?.previewVisible === false &&
      Array.isArray(value?.titles) &&
      value.titles.includes("Market Entry Report") &&
      value.titles.includes("Executive Summary") &&
      !String(value?.outlineText || "").includes("Prepared for"),
    "desktop outline mode shell",
  );

  await clickOutlineAction(session, "Executive Summary", "Add child");
  await waitForOutlineTitle(session, "New subsection");

  const finalOutline = await waitForValue(
    session,
    outlineModeEvidenceScript,
    (value) =>
      Array.isArray(value?.titles) &&
      value.titles.includes("Executive Summary") &&
      value.titles.includes("New subsection") &&
      value.titles.includes("Source Governance") &&
      value.levels?.["Data Table"] === "2" &&
      !String(value?.outlineText || "").includes("Prepared for"),
    "desktop outline mode structure evidence",
  );
  await setViewMode(session, "source", "desktop source mode selection after outline edits");
  const source = await waitForValue(
    session,
    `
      ${editorDocumentTextFunction}
      return {
        mode: document.querySelector('.workspace')?.className || '',
        editor: editorDocumentText(),
        renderedEditor: document.querySelector('.cm-content')?.textContent || '',
      };
    `,
    (value) =>
      String(value?.mode || "").includes("mode-source") &&
      String(value?.editor || "").includes("## Executive Summary") &&
      String(value?.editor || "").includes("### New subsection") &&
      String(value?.editor || "").includes("## Data Table") &&
      String(value?.editor || "").includes("## Source Governance"),
    "desktop outline edits reflected in source",
  );
  report.outlineArtifacts = {
    initialTitles: initial.titles,
    finalTitles: finalOutline.titles,
    finalLevels: finalOutline.levels,
    sourceEvidence: {
      executiveSummary: String(source.editor || "").includes("## Executive Summary"),
      newSubsection: String(source.editor || "").includes("### New subsection"),
      dataTablePreserved: String(source.editor || "").includes("## Data Table"),
      sourceGovernancePreserved: String(source.editor || "").includes("## Source Governance"),
    },
  };
  recordAssertion("desktop WebDriver edits document structure in outline mode");
}

async function assertDirtyTitleWorkflow(session) {
  await execute(session, `
    const newButton = [...document.querySelectorAll('button')].find((item) => item.textContent.trim() === 'New');
    if (!newButton) throw new Error('New document button was not visible');
    newButton.click();
    return true;
  `);
  const dirtyTitle = await waitForValue(
    session,
    `
      return {
        title: document.title,
        tab: document.querySelector('.document-tabs .tab.active')?.textContent || '',
      };
    `,
    (value) => String(value?.title || "").startsWith("* ") && Boolean(String(value?.tab || "").trim()),
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

async function clickOutlineAction(session, title, action) {
  await waitForValue(
    session,
    `
    const title = ${JSON.stringify(title)};
    const action = ${JSON.stringify(action)};
    const rows = [...document.querySelectorAll('#outline-mode .outline-mode-row')];
    const row = rows.find((item) => item.querySelector('input')?.value === title);
    if (!row) {
      return {
        clicked: false,
        reason: 'missing-row',
        titles: rows.map((item) => item.querySelector('input')?.value || ''),
      };
    }
    const button = [...row.querySelectorAll('button')].find((item) => item.textContent.trim() === action);
    if (!button) {
      return {
        clicked: false,
        reason: 'missing-action',
        actions: [...row.querySelectorAll('button')].map((item) => item.textContent.trim()),
      };
    }
    button.click();
    return { clicked: true };
  `,
    (value) => value?.clicked === true,
    `outline action ${action} for ${title}`,
  );
}

async function waitForOutlineTitle(session, title) {
  return waitForValue(
    session,
    outlineModeEvidenceScript,
    (value) => Array.isArray(value?.titles) && value.titles.includes(title),
    `outline title ${title}`,
  );
}

async function setViewMode(session, mode, description) {
  await waitForValue(
    session,
    `
      const mode = ${JSON.stringify(mode)};
      const select = document.querySelector('[aria-label="View mode"]');
      if (!select) {
        return { changed: false, reason: 'missing-select' };
      }
      select.value = mode;
      select.dispatchEvent(new Event('input', { bubbles: true }));
      select.dispatchEvent(new Event('change', { bubbles: true }));
      return {
        changed: true,
        selected: select.value,
        workspace: document.querySelector('.workspace')?.className || '',
      };
    `,
    (value) => value?.changed === true && value?.selected === mode,
    `${description} control update`,
  );
  await waitForValue(
    session,
    "return document.querySelector('.workspace')?.className || '';",
    (value) => String(value || "").includes(`mode-${mode}`),
    description,
  );
}

async function closeCommandPalette(session) {
  await waitForValue(
    session,
    `
      const dialog = document.querySelector('[role="dialog"][aria-label="Command palette"]');
      if (!dialog) return { open: false };
      const close = dialog.querySelector('[aria-label="Close command palette"]');
      close?.click();
      dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
      return { open: Boolean(document.querySelector('[role="dialog"][aria-label="Command palette"]')) };
    `,
    (value) => value?.open === false,
    "native command palette closed",
  );
}

async function assertFileSaveOpenWorkflow(session) {
  const expectedPath = workflowFilePath.replaceAll("\\", "/");
  await execute(session, `
    const saveButton = document.querySelector('#main-commands button[aria-label="Save"]');
    if (!saveButton) throw new Error('Save button was not visible in the desktop command bar');
    saveButton.click();
    return true;
  `);
  await waitForValue(
    session,
    `
      return {
        title: document.title,
        tab: document.querySelector('.document-tabs .tab.active')?.textContent || '',
        status: document.querySelector('.status-bar')?.textContent || '',
        editor: document.querySelector('.cm-content')?.textContent || '',
      };
    `,
    (value) =>
      !String(value?.title || "").startsWith("* ") &&
      String(value?.tab || "").includes("native-workflow-file") &&
      String(value?.editor || "").trim().length > 20,
    "saved real Markdown file",
  );
  if (!existsSync(workflowFilePath)) {
    throw new Error(`desktop WebDriver Markdown file was not written: ${relative(workflowFilePath)}`);
  }
  const savedText = readFileSync(workflowFilePath, "utf8");
  if (savedText.trim().length <= 20) {
    throw new Error(`desktop WebDriver Markdown file did not preserve document content: ${relative(workflowFilePath)}`);
  }

  await execute(session, `
    const newButton = document.querySelector('#main-commands button[aria-label="New"]');
    if (!newButton) throw new Error('New button was not visible in the desktop command bar');
    newButton.click();
    return true;
  `);
  await waitForValue(
    session,
    `
      ${editorDocumentTextFunction}
      return {
        title: document.title,
        tab: document.querySelector('.document-tabs .tab.active')?.textContent || '',
        editor: editorDocumentText(),
      };
    `,
    (value) =>
      String(value?.title || "").startsWith("* ") &&
      String(value?.tab || "").includes("Untitled") &&
      String(value?.editor || "").includes("Market Entry Report"),
    "new dirty document before reopening saved file",
  );
  await execute(session, `
    const openButton = document.querySelector('#main-commands button[aria-label="Open"]');
    if (!openButton) throw new Error('Open button was not visible in the desktop command bar');
    openButton.click();
    return true;
  `);
  const reopened = await waitForValue(
    session,
    `
      ${editorDocumentTextFunction}
      return {
        title: document.title,
        tab: document.querySelector('.document-tabs .tab.active')?.textContent || '',
        status: document.querySelector('.status-bar')?.textContent || '',
        editor: editorDocumentText(),
      };
    `,
    (value) =>
      !String(value?.title || "").startsWith("* ") &&
      String(value?.tab || "").includes("native-workflow-file") &&
      String(value?.editor || "").trim().length > 20,
    "reopened real Markdown file",
  );
  report.fileArtifacts = {
    path: relative(workflowFilePath),
    bytes: statSync(workflowFilePath).size,
    title: reopened.title,
    expectedPath,
  };
  recordAssertion("desktop WebDriver saves and reopens real Markdown file through dialog-free smoke path");
}

async function assertRenameDuplicateRevealWorkflow(session) {
  await execute(session, `
    const renameButton = document.querySelector('#main-commands button[aria-label="Rename"]');
    if (!renameButton) throw new Error('Rename button was not visible in the desktop command bar');
    renameButton.click();
    return true;
  `);
  const renamed = await waitForValue(
    session,
    `
      return {
        title: document.title,
        tab: document.querySelector('.document-tabs .tab.active')?.textContent || '',
        status: document.querySelector('.status-bar')?.textContent || '',
        editor: document.querySelector('.cm-content')?.textContent || '',
      };
    `,
    (value) =>
      !String(value?.title || "").startsWith("* ") &&
      String(value?.tab || "").includes("native-workflow-renamed") &&
      String(value?.status || "").includes("Renamed") &&
      String(value?.editor || "").trim().length > 20,
    "renamed real Markdown file",
  );
  if (!existsSync(workflowRenamedPath)) {
    throw new Error(`desktop WebDriver renamed Markdown file was not written: ${relative(workflowRenamedPath)}`);
  }
  if (existsSync(workflowFilePath)) {
    throw new Error(`desktop WebDriver rename left the old Markdown path behind: ${relative(workflowFilePath)}`);
  }

  await activateDocumentTab(session, "native-workflow-renamed");
  await execute(session, `
    const normalized = (value) => value.replace(/\\s+/g, ' ').trim();
    const activeTab = normalized(document.querySelector('.document-tabs .tab.active')?.textContent || '');
    if (!activeTab.includes('native-workflow-renamed')) throw new Error('Expected renamed workflow file to be active before duplicate, found ' + activeTab);
    const duplicateButton = document.querySelector('#main-commands button[aria-label="Duplicate"]');
    if (!duplicateButton) throw new Error('Duplicate button was not visible in the desktop command bar');
    duplicateButton.click();
    return true;
  `);
  await waitForPathExists(workflowDuplicatePath, "duplicated real Markdown file artifact");
  await activateDocumentTab(session, "native-workflow-duplicate");
  const duplicated = await waitForValue(
    session,
    `
      return {
        title: document.title,
        tab: document.querySelector('.document-tabs .tab.active')?.textContent || '',
        status: document.querySelector('.status-bar')?.textContent || '',
        editor: document.querySelector('.cm-content')?.textContent || '',
      };
    `,
    (value) =>
      !String(value?.title || "").startsWith("* ") &&
      String(value?.tab || "").includes("native-workflow-duplicate") &&
      String(value?.editor || "").trim().length > 20,
    "duplicated real Markdown file",
  );
  if (!existsSync(workflowDuplicatePath)) {
    throw new Error(`desktop WebDriver duplicate Markdown file was not written: ${relative(workflowDuplicatePath)}`);
  }
  const duplicateText = readFileSync(workflowDuplicatePath, "utf8");
  if (duplicateText.trim().length <= 20) {
    throw new Error(`desktop WebDriver duplicate Markdown file did not preserve document content: ${relative(workflowDuplicatePath)}`);
  }

  const revealControl = await execute(session, `
    const normalized = (value) => value.replace(/\\s+/g, ' ').trim();
    const revealButton = document.querySelector('#main-commands button[aria-label="Reveal"]');
    if (!revealButton) throw new Error('Reveal button was not visible in the desktop command bar');
    return {
      label: normalized(revealButton.textContent || ''),
      enabled: !revealButton.disabled,
      status: document.querySelector('.status-bar')?.textContent || '',
    };
  `);
  if (revealControl.value?.label !== "Reveal" || revealControl.value?.enabled !== true) {
    throw new Error(`desktop WebDriver reveal affordance was not enabled for duplicated file: ${JSON.stringify(revealControl.value)}`);
  }
  report.fileArtifacts = {
    ...report.fileArtifacts,
    renamedPath: relative(workflowRenamedPath),
    renamedBytes: statSync(workflowRenamedPath).size,
    duplicatePath: relative(workflowDuplicatePath),
    duplicateBytes: statSync(workflowDuplicatePath).size,
    renameTitle: renamed.title,
    duplicateTitle: duplicated.title,
    revealLabel: revealControl.value.label,
    revealControlEnabled: revealControl.value.enabled,
    revealStatusBeforeLaunch: revealControl.value.status,
  };
  recordAssertion("desktop WebDriver renames, duplicates, and exposes reveal affordance for real Markdown files");
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
  if (!html.includes("<html") || !html.includes("<body")) {
    throw new Error(`desktop WebDriver HTML export did not include expected HTML document structure: ${relative(workflowExportPath)}`);
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

async function assertPreferenceWorkflow(session, originalPreferences) {
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
  const appliedPreferences = await waitForValue(
    session,
    `
      const read = ${readPreferenceScript};
      return read();
    `,
    (value) => preferencesMatch(value, targetPreferences),
    "applied desktop preferences in packaged session",
  );
  report.preferenceArtifacts = {
    originalPreferences,
    targetPreferences,
    appliedPreferences,
  };
  recordAssertion("desktop preferences apply in packaged WebDriver session");
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
  const expectedSidebar = value;
  const expectedLabels = Array.isArray(labelText) ? labelText : [labelText];
  await waitForValue(
    session,
    `
      const normalized = (value) => value.replace(/\\s+/g, ' ').trim();
      const expectedLabels = ${JSON.stringify(expectedLabels)};
      const sidebar = document.querySelector('[aria-label="Sidebar panel"]');
      if (!sidebar) {
        return {
          changed: false,
          reason: 'missing-sidebar-select',
          sidebarValue: '',
          hasExpectedLabel: false,
          labels: [],
        };
      }
      sidebar.value = ${JSON.stringify(value)};
      sidebar.dispatchEvent(new Event('input', { bubbles: true }));
      sidebar.dispatchEvent(new Event('change', { bubbles: true }));
      const labels = [...document.querySelectorAll('label')].map((label) => normalized(label.textContent || ''));
      return {
        changed: true,
        sidebarValue: sidebar?.value || '',
        hasExpectedLabel: expectedLabels.every((expected) => labels.some((label) => label.includes(expected))),
        labels: labels.slice(0, 20),
      };
    `,
    (result) => result?.changed === true && result?.sidebarValue === expectedSidebar && result?.hasExpectedLabel === true,
    `${value} sidebar controls`,
  );
}

async function saveWorkspace(session) {
  await execute(session, `
    const button = document.querySelector('#main-commands button[aria-label="Save Workspace"]');
    button.click();
    return true;
  `);
}

async function activateDocumentTab(session, labelFragment) {
  await waitForValue(
    session,
    `
      const normalized = (value) => String(value || '').replace(/\\s+/g, ' ').trim();
      const target = [...document.querySelectorAll('.document-tabs .tab')].find((item) => normalized(item.textContent).includes(${JSON.stringify(labelFragment)}));
      if (!target) {
        return {
          found: false,
          active: normalized(document.querySelector('.document-tabs .tab.active')?.textContent || ''),
          tabs: [...document.querySelectorAll('.document-tabs .tab')].map((item) => normalized(item.textContent)).slice(0, 10),
        };
      }
      target.querySelector('.tab-main')?.click();
      return {
        found: true,
        active: normalized(document.querySelector('.document-tabs .tab.active')?.textContent || ''),
      };
    `,
    (value) => value?.found === true && String(value?.active || "").includes(labelFragment),
    `${labelFragment} document tab activation`,
  );
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

async function waitForPathExists(path, description) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (existsSync(path)) return;
    await delay(250);
  }
  throw new Error(`timed out waiting for ${description}: ${relative(path)}`);
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

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0) return "";
  return result.stdout.trim();
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain", "--untracked-files=no"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 && result.stdout.trim() === "";
}

function collectMacosNativeProof() {
  if (!existsSync(macosFallbackSmokeReportPath)) {
    return {
      status: "missing",
      reportPath: relative(macosFallbackSmokeReportPath),
      launchReportPath: relative(macosFallbackLaunchReportPath),
      reason: "Run NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke before collecting macOS fallback proof.",
    };
  }
  let smoke = null;
  try {
    smoke = JSON.parse(readFileSync(macosFallbackSmokeReportPath, "utf8"));
  } catch (error) {
    return {
      status: "invalid",
      reportPath: relative(macosFallbackSmokeReportPath),
      launchReportPath: relative(macosFallbackLaunchReportPath),
      reason: `native smoke report is not valid JSON: ${error.message}`,
    };
  }
  const workflow = smoke.nativeWorkflow?.payload || smoke.nativeWorkflow || {};
  const launch = readJsonIfPresent(macosFallbackLaunchReportPath);
  const smokeStats = statSync(macosFallbackSmokeReportPath);
  const applicationStats = statSync(application);
  const launchStats = existsSync(macosFallbackLaunchReportPath) ? statSync(macosFallbackLaunchReportPath) : null;
  const assertions = Array.isArray(workflow.assertions) ? workflow.assertions : [];
  const passedAssertions = assertions.filter((assertion) => assertion?.passed === true);
  const issues = [];
  if (smokeStats.mtimeMs + 1000 < applicationStats.mtimeMs) {
    issues.push("native smoke report is older than the desktop binary");
  }
  if (!launch) {
    issues.push("native launch report is missing or invalid");
  } else if (launch.platform !== "darwin") {
    issues.push(`native launch report platform is ${JSON.stringify(launch.platform)}`);
  } else if (launch.processAlive !== true || launch.status !== "survived-until-timeout") {
    issues.push(`native launch did not survive the bounded smoke window: ${JSON.stringify(launch.status)}`);
  }
  if (launchStats && launchStats.mtimeMs + 1000 < applicationStats.mtimeMs) {
    issues.push("native launch report is older than the desktop binary");
  }
  if (smoke.platform !== "darwin") issues.push(`native smoke report platform is ${JSON.stringify(smoke.platform)}`);
  if (smoke.nativeWindow?.window?.visible !== true) issues.push("native smoke did not record a visible window");
  if (smoke.nativeUi?.payload?.surfaces?.source !== true || smoke.nativeUi?.payload?.surfaces?.preview !== true) {
    issues.push("native smoke did not record source and preview surfaces");
  }
  if (workflow.status !== "passed") issues.push(`native workflow status is ${JSON.stringify(workflow.status)}`);
  for (const requiredAssertion of [
    "native workflow saved document to real file",
    "native workflow opened saved real file",
    "native workflow wrote html export artifact",
    "native workflow exported html from native menu command",
    "native workflow rendered outline mode structure only",
    "native workflow navigated outline heading to source",
    "native workflow restored workspace tabs with active pinned and scroll state",
    "native workflow restored project-local snapshot",
  ]) {
    if (!passedAssertions.some((assertion) => assertion.name === requiredAssertion)) {
      issues.push(`native workflow is missing assertion: ${requiredAssertion}`);
    }
  }
  const exportManifest = readJsonIfPresent(workflow.exportResult?.manifestPath);
  if (workflow.exportResult?.target !== "html" || exportManifest?.export_target !== "html" || !exportManifest?.output_hash) {
    issues.push("native workflow did not record HTML export target/hash evidence");
  }
  const outlineMode = Array.isArray(workflow.modeEvidence) ? workflow.modeEvidence.find((entry) => entry?.mode === "outline") : null;
  const outlineTitles = Array.isArray(outlineMode?.outlineTitles) ? outlineMode.outlineTitles : [];
  if (outlineMode?.outlineVisible !== true || outlineMode?.sourceVisible !== false || outlineMode?.previewVisible !== false) {
    issues.push("native workflow did not record outline mode with source and preview hidden");
  }
  if (!outlineTitles.includes("Market Entry Report") || !outlineTitles.includes("Executive Summary")) {
    issues.push("native workflow did not record expected outline mode title values");
  }
  const outlineNavigation = workflow.outlineNavigationEvidence?.outline || {};
  if (
    outlineNavigation.sidebar !== "outline" ||
    outlineNavigation.mode !== "split" ||
    outlineNavigation.buttonFound !== true ||
    outlineNavigation.selectedLine !== outlineNavigation.targetLine ||
    !String(outlineNavigation.selectedText || "").includes("## Native Outline Target")
  ) {
    issues.push("native workflow did not record outline sidebar navigation to CodeMirror source");
  }
  return {
    status: issues.length === 0 ? "passed" : "incomplete",
    reportPath: relative(macosFallbackSmokeReportPath),
    launchReportPath: relative(macosFallbackLaunchReportPath),
    binaryMtime: applicationStats.mtime.toISOString(),
    reportMtime: smokeStats.mtime.toISOString(),
    launchReportMtime: launchStats?.mtime.toISOString() || "",
    launchStatus: launch?.status || "",
    processAlive: launch?.processAlive === true,
    freshForBinary:
      smokeStats.mtimeMs + 1000 >= applicationStats.mtimeMs &&
      Boolean(launchStats && launchStats.mtimeMs + 1000 >= applicationStats.mtimeMs),
    assertionCount: assertions.length,
    passedAssertionCount: passedAssertions.length,
    windowTitle: smoke.nativeWindow?.window?.title || "",
    exportTarget: workflow.exportResult?.target || "",
    exportPath: workflow.exportResult?.outputPath || "",
    exportManifestPath: workflow.exportResult?.manifestPath || "",
    outputHash: exportManifest?.output_hash || "",
    filePath: workflow.fileWorkflow?.filePath || "",
    outlineModeTitles: outlineTitles,
    outlineNavigationSelectedLine: outlineNavigation.selectedLine || 0,
    outlineNavigationTargetLine: outlineNavigation.targetLine || 0,
    issues,
  };
}

function readJsonIfPresent(path) {
  if (!path || !existsSync(path)) return null;
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch {
    return null;
  }
}

function firstLine(text) {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find(Boolean);
}
