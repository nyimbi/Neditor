import { rmSync, existsSync, statSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const auditDir = resolve(process.env.NEDITOR_RENDERED_EXPORT_AUDIT_DIR || join(root, ".tmp", "rendered-export-audit"));
const requiredFiles = [
  ["rendered-export-audit.html", 1000],
  ["rendered-export-audit.pdf", 1000],
  ["rendered-export-audit.docx", 1000],
  ["rendered-export-audit.pptx", 1000],
  ["rendered-export-audit.markdown-bundle.zip", 1000],
  ["rendered-export-audit.blog.zip", 1000],
  ["rendered-export-audit.substack.zip", 1000],
  ["rendered-export-audit.tex", 1000],
  ["rendered-export-audit.google-docs.zip", 1000],
  ["rendered-export-audit-report.json", 500],
  ["README.md", 100],
];

if (auditDir.includes(`${root}/.tmp/`)) {
  rmSync(auditDir, { recursive: true, force: true });
}

const result = spawnSync(
  "cargo",
  [
    "test",
    "--locked",
    "representative_rendered_export_artifacts_are_package_inspectable",
    "--lib",
    "--",
    "--nocapture",
  ],
  {
    cwd: join(root, "src-tauri"),
    env: {
      ...process.env,
      NEDITOR_RENDERED_EXPORT_AUDIT_DIR: auditDir,
    },
    shell: process.platform === "win32",
    stdio: "inherit",
  },
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

const issues = [];
for (const [file, minBytes] of requiredFiles) {
  const path = join(auditDir, file);
  if (!existsSync(path)) {
    issues.push(`missing audit artifact: ${file}`);
    continue;
  }
  const size = statSync(path).size;
  if (size < minBytes) {
    issues.push(`${file} is unexpectedly small: ${size} bytes`);
  }
}

if (issues.length === 0) {
  const report = JSON.parse(readFileSync(join(auditDir, "rendered-export-audit-report.json"), "utf8"));
  const targets = new Set(report.targets?.map((target) => target.target));
  for (const target of ["html", "pdf", "docx", "pptx", "markdown-bundle", "blog", "substack", "latex", "google-docs"]) {
    if (!targets.has(target)) {
      issues.push(`audit report is missing target ${target}`);
    }
  }
  if (!Array.isArray(report.manualChecklist) || report.manualChecklist.length < 5) {
    issues.push("audit report manual checklist is incomplete");
  }
}

if (issues.length > 0) {
  console.error("Rendered export audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

console.log(`Rendered export audit artifacts verified in ${auditDir}`);
