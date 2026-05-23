import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = resolve(
  process.env.NEDITOR_PERFORMANCE_AUDIT_REPORT ||
    join(root, ".tmp", "performance-audit", "report.json"),
);
const largeDocumentE2eReportPath = join(root, ".tmp", "performance-audit", "e2e-large-document-report.json");

const commands = [
  {
    id: "rust-performance-suite",
    description: "Compiler/export/cache/memory stress tests",
    cwd: join(root, "src-tauri"),
    command: "cargo",
    args: ["test", "--locked", "performance_tests", "--lib"],
  },
  {
    id: "browser-large-document-workflow",
    description: "Large document edit, preview update, and scroll-sync workflow",
    cwd: root,
    command: process.execPath,
    displayCommand:
      'node scripts/run-e2e.mjs e2e/app-workflows.spec.ts --grep "large document" --project chromium',
    args: [
      join(root, "scripts", "run-e2e.mjs"),
      "e2e/app-workflows.spec.ts",
      "--grep",
      "large document",
      "--project",
      "chromium",
    ],
    env: {
      NEDITOR_E2E_REPORT_PATH: largeDocumentE2eReportPath,
    },
    evidenceReport: ".tmp/performance-audit/e2e-large-document-report.json",
  },
];

const results = [];
let failed = false;

for (const spec of commands) {
  const startedAt = Date.now();
  const result = spawnSync(spec.command, spec.args, {
    cwd: spec.cwd,
    encoding: "utf8",
    shell: process.platform === "win32",
    env: {
      ...process.env,
      ...(spec.env || {}),
    },
  });
  const elapsedMs = Date.now() - startedAt;
  const entry = {
    id: spec.id,
    description: spec.description,
    command: spec.displayCommand ?? [spec.command, ...spec.args].join(" "),
    cwd: relativeCwd(spec.cwd),
    evidenceReport: spec.evidenceReport || null,
    status: result.status === 0 ? "pass" : "fail",
    exitCode: result.status,
    elapsedMs,
    stdoutTail: tail(result.stdout),
    stderrTail: tail(result.stderr),
  };
  results.push(entry);
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  if (result.status !== 0) {
    failed = true;
    break;
  }
}

const report = {
  generatedAt: new Date().toISOString(),
  status: failed ? "fail" : "pass",
  summary: {
    checks: results.length,
    passed: results.filter((result) => result.status === "pass").length,
    failed: results.filter((result) => result.status === "fail").length,
  },
  evidence: [
    "Rust performance_tests cover large compile, repeated export stability, bounded compile/export memory growth, and external transform cache reuse.",
    "Browser workflow covers large document editing, preview update latency budget, and editor-to-preview scroll synchronization.",
  ],
  results,
};

mkdirSync(dirname(reportPath), { recursive: true });
writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);

if (failed) {
  console.error(`Performance audit failed; wrote ${reportPath}.`);
  process.exit(1);
}

console.log(`Performance audit passed; wrote ${reportPath}.`);

function tail(value) {
  const text = value ?? "";
  const lines = text.trim().split(/\r?\n/).filter(Boolean);
  return lines.slice(-30);
}

function relativeCwd(cwd) {
  return cwd === root ? "." : cwd.replace(`${root}/`, "");
}
