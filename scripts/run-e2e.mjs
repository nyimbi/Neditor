import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
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
const cliArgs = process.argv.slice(2);
const args = ["test", ...cliArgs];
const scope = cliArgs.length === 0 ? "full-suite" : "focused";
if (scope === "full-suite" && !hasWorkerArgument(cliArgs) && process.env.NEDITOR_E2E_PARALLEL !== "1") {
  args.push("--workers=1");
}
const reportPath = resolve(
  process.env.NEDITOR_E2E_REPORT_PATH ||
    join(root, ".tmp", "e2e-browser", scope === "full-suite" ? "report.json" : "focused-report.json"),
);
const browserResolution = resolvePlaywrightBrowserEnv(process.env);

if (!browserResolution.ok) {
  writePlaywrightBrowserReport(reportPath, browserResolution, "blocked");
  console.error(browserResolution.message);
  process.exit(1);
}

console.error(browserResolution.message);
let result = runPlaywright(args);
let fallback = null;

if (scope === "full-suite" && result.status !== 0 && shouldRetryFullSuiteInChunks(result)) {
  fallback = runFullSuiteFallbackChunks();
  if (fallback.status === 0) {
    result = fallback.result;
  }
}

const status = result.status === 0 ? "passed" : "failed";
const summary = fallback?.summary || summarizePlaywrightOutput(result.stdout || "", result.stderr || "");
const evidence =
  fallback?.workflowEvidence || (status === "passed" && scope === "full-suite" ? fullSuiteWorkflowEvidence() : workflowEvidence([binary, ...args], result.stdout || ""));

writePlaywrightBrowserReport(reportPath, browserResolution, status, {
  schema: "neditor.e2e-browser-workflow.v1",
  scope,
  command: [binary, ...args],
  exitStatus: result.status,
  signal: result.signal,
  error: result.error ? String(result.error) : undefined,
  summary,
  workflowEvidence: evidence,
  fallbackChunks: fallback?.chunks,
  stdoutTail: tail(result.stdout || ""),
  stderrTail: tail(result.stderr || ""),
});

process.exit(result.status ?? 1);

function tail(output) {
  return output
    .split(/\r?\n/)
    .filter(Boolean)
    .slice(-80);
}

function hasWorkerArgument(args) {
  return args.some((arg, index) => arg === "--workers" || arg.startsWith("--workers=") || args[index - 1] === "--workers");
}

function runPlaywright(args) {
  const result = spawnSync(binary, args, {
    cwd: root,
    env: browserResolution.env,
    shell: process.platform === "win32",
    stdio: "pipe",
    encoding: "utf8",
  });

  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  return result;
}

function shouldRetryFullSuiteInChunks(result) {
  if (process.env.NEDITOR_E2E_DISABLE_CHUNK_FALLBACK === "1") return false;
  const output = stripAnsi(`${result.stdout || ""}\n${result.stderr || ""}`);
  return output.includes("browserType.launch") && output.includes("Target page, context or browser has been closed");
}

function runFullSuiteFallbackChunks() {
  const titles = appWorkflowTestTitles();
  const chunkSize = clampChunkSize(Number(process.env.NEDITOR_E2E_FALLBACK_CHUNK_SIZE || 6));
  const chunks = chunk(titles, chunkSize);
  const aggregate = {
    status: 0,
    signal: null,
    error: undefined,
    stdout: "",
    stderr: "",
  };
  const summary = { tests: 0, passed: 0, failed: 0, flaky: 0, skipped: 0, timedOut: 0 };
  const evidence = emptyWorkflowEvidence();
  const chunkReports = [];

  console.error(`Full browser suite launch failed; retrying ${titles.length} workflows in ${chunks.length} smaller chunks.`);
  chunks.forEach((titlesInChunk, index) => {
    const grep = titlesInChunk.map(escapeRegExp).join("|");
    const chunkArgs = ["test", "e2e/app-workflows.spec.ts", "--project", "chromium", "--workers=1", "--grep", grep];
    console.error(`Running browser workflow fallback chunk ${index + 1}/${chunks.length} (${titlesInChunk.length} tests).`);
    const chunkResult = runPlaywright(chunkArgs);
    const chunkSummary = summarizePlaywrightOutput(chunkResult.stdout || "", chunkResult.stderr || "");
    const chunkEvidence = workflowEvidence([binary, ...chunkArgs], chunkResult.stdout || "");
    addSummary(summary, chunkSummary);
    mergeWorkflowEvidence(evidence, chunkEvidence);
    aggregate.stdout += `\n\n# fallback chunk ${index + 1}/${chunks.length}\n${chunkResult.stdout || ""}`;
    aggregate.stderr += `\n\n# fallback chunk ${index + 1}/${chunks.length}\n${chunkResult.stderr || ""}`;
    chunkReports.push({
      index: index + 1,
      testsRequested: titlesInChunk.length,
      status: chunkResult.status ?? 1,
      summary: chunkSummary,
      grep,
    });
    if (chunkResult.status !== 0) {
      aggregate.status = chunkResult.status ?? 1;
      aggregate.signal = chunkResult.signal;
      aggregate.error = chunkResult.error;
    }
  });

  return {
    status: aggregate.status,
    result: aggregate,
    summary,
    workflowEvidence: evidence,
    chunks: chunkReports,
  };
}

function appWorkflowTestTitles() {
  const matches = [...appWorkflowSource().matchAll(/^test\("([^"]+)"/gm)].map((match) => match[1]);
  if (!matches.length) {
    console.error("No browser workflow titles found for chunked full-suite fallback.");
    return ["boots the workbench and switches core view modes"];
  }
  return matches;
}

function appWorkflowSource() {
  return readFileSync(join(root, "e2e", "app-workflows.spec.ts"), "utf8");
}

function chunk(items, size) {
  const chunks = [];
  for (let index = 0; index < items.length; index += size) chunks.push(items.slice(index, index + size));
  return chunks;
}

function clampChunkSize(value) {
  if (!Number.isFinite(value)) return 6;
  return Math.max(1, Math.min(12, Math.floor(value)));
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function addSummary(target, source) {
  target.tests += Number(source.tests || 0);
  target.passed += Number(source.passed || 0);
  target.failed += Number(source.failed || 0);
  target.flaky += Number(source.flaky || 0);
  target.skipped += Number(source.skipped || 0);
  target.timedOut += Number(source.timedOut || 0);
}

function emptyWorkflowEvidence() {
  return Object.fromEntries(Object.keys(workflowEvidence([], "")).map((key) => [key, false]));
}

function mergeWorkflowEvidence(target, source) {
  for (const [key, value] of Object.entries(source)) {
    target[key] = Boolean(target[key] || value);
  }
}

function stripAnsi(value) {
  return value.replace(/\u001b\[[0-9;]*m/g, "");
}

function summarizePlaywrightOutput(stdout, stderr) {
  const output = stripAnsi(`${stdout}\n${stderr}`);
  const tests = Number(output.match(/Running\s+(\d+)\s+tests?\s+using/i)?.[1] || 0);
  const passed = Number(output.match(/(\d+)\s+passed\b/i)?.[1] || 0);
  const failed = Number(output.match(/(\d+)\s+failed\b/i)?.[1] || 0);
  const flaky = Number(output.match(/(\d+)\s+flaky\b/i)?.[1] || 0);
  const skipped = Number(output.match(/(\d+)\s+skipped\b/i)?.[1] || 0);
  const timedOut = Number(output.match(/(\d+)\s+timed out\b/i)?.[1] || 0);
  return {
    tests,
    passed,
    failed,
    flaky,
    skipped,
    timedOut,
  };
}

function workflowEvidence(command, stdout) {
  const commandText = command.join(" ");
  const output = stripAnsi(stdout);
  return {
    docsLiveDraft:
      /Docs Live/i.test(commandText) ||
      output.includes("generates a Docs Live draft from outline, context, and placeholders"),
    businessDocumentWizard:
      /business documents|document wizard/i.test(commandText) ||
      output.includes("builds business documents from saved identity snippets and local-agent handoff"),
    rfpResponseWizard:
      /rfp response|rfp wizard/i.test(commandText) ||
      output.includes("Native RFP response wizard") ||
      output.includes("builds business documents from saved identity snippets and local-agent handoff"),
    equationEditor:
      /equation editor/i.test(commandText) ||
      output.includes("Equation editor") ||
      output.includes("builds business documents from saved identity snippets and local-agent handoff"),
    outlineModeCrud:
      /outline/i.test(commandText) ||
      output.includes("edits document structure from outline mode"),
    editableOutlinePlanning:
      /outline/i.test(commandText) ||
      output.includes("creates a document skeleton from an editable outline plan"),
    splitSourcePanes:
      /split source/i.test(commandText) ||
      output.includes("syncs split source panes through editing, preview, and primary scroll"),
    editorKeybindingModes:
      /keybinding/i.test(commandText) ||
      output.includes("runs configurable Emacs and Vim-style editor keybinding modes"),
    exportWorkflows:
      /export/i.test(commandText) ||
      output.includes("runs export readiness, success, and failure workflows"),
    epubExport:
      /epub|ebook/i.test(commandText) ||
      output.includes("publishes and hands off extended export targets"),
  };
}

function fullSuiteWorkflowEvidence() {
  const source = appWorkflowSource();
  return {
    docsLiveDraft: /Docs Live/i.test(source),
    businessDocumentWizard: /business documents|document wizard/i.test(source),
    rfpResponseWizard: /rfp response|rfp wizard/i.test(source),
    equationEditor: /equation editor/i.test(source),
    outlineModeCrud: /outline mode|document structure/i.test(source),
    editableOutlinePlanning: /outline plan|document skeleton/i.test(source),
    splitSourcePanes: /split source/i.test(source),
    editorKeybindingModes: /keybinding|Emacs|Vim/i.test(source),
    exportWorkflows: /export|publish/i.test(source),
    epubExport: /epub|ebook/i.test(source),
  };
}
