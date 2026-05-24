import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputDir = resolve(process.env.NEDITOR_RELEASE_EVIDENCE_KIT_DIR || join(root, ".tmp", "release-evidence-kit"));
const sourceCommit = gitCommit();
const sourceTreeClean = gitTreeClean();
const readiness = readJson(join(root, ".tmp", "release-readiness", "report.json"));
const gaps = Array.isArray(readiness?.evidenceGaps) ? readiness.evidenceGaps : Array.isArray(readiness?.gaps) ? readiness.gaps : [];

const templateCopies = [
  [".tmp/platform-evidence/templates/win32-package-artifacts.template.json", "templates/platform/win32-package-artifacts.template.json"],
  [".tmp/platform-evidence/templates/win32-tauri-webdriver-report.template.json", "templates/platform/win32-tauri-webdriver-report.template.json"],
  [".tmp/platform-evidence/templates/linux-package-artifacts.template.json", "templates/platform/linux-package-artifacts.template.json"],
  [".tmp/platform-evidence/templates/linux-tauri-webdriver-report.template.json", "templates/platform/linux-tauri-webdriver-report.template.json"],
  [".tmp/release-signing/templates/darwin-signing-evidence.template.json", "templates/signing/darwin-signing-evidence.template.json"],
  [".tmp/release-signing/templates/win32-signing-evidence.template.json", "templates/signing/win32-signing-evidence.template.json"],
  [".tmp/release-signing/templates/linux-signing-evidence.template.json", "templates/signing/linux-signing-evidence.template.json"],
  [".tmp/ai-provider-evidence/templates/provider-evidence.template.json", "templates/ai-provider/provider-evidence.template.json"],
  [".tmp/google-docs-import/import-evidence.template.json", "templates/google-docs/import-evidence.template.json"],
  [".tmp/rendered-export-audit/visual-review-signoff.template.json", "templates/rendered-export/visual-review-signoff.template.json"],
  [".tmp/accessibility/manual-review-template.json", "templates/accessibility/manual-review-template.json"],
];

const runbooks = [
  {
    file: "runbooks/windows-platform.md",
    title: "Windows Package And WebDriver Proof",
    gaps: ["windows-linux-tauri-webdriver-execution", "windows-package-artifact-proof"],
    commands: [
      "Optional CI path: gh workflow run neditor-release-evidence.yml",
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run build",
      "./node_modules/.bin/tauri build --bundles all",
      "pnpm run test:tauri-webdriver -- --strict",
      "NEDITOR_PLATFORM_EVIDENCE_PLATFORM=win32 pnpm run collect:platform-evidence",
      "pnpm run check:platform-evidence",
    ],
    returns: [".tmp/platform-evidence/external/win32/package-artifacts.json", ".tmp/platform-evidence/external/win32/tauri-webdriver-report.json"],
  },
  {
    file: "runbooks/linux-platform.md",
    title: "Linux Package And WebDriver Proof",
    gaps: ["windows-linux-tauri-webdriver-execution", "linux-package-artifact-proof"],
    commands: [
      "Optional CI path: gh workflow run neditor-release-evidence.yml",
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run build",
      "./node_modules/.bin/tauri build --bundles all",
      "pnpm run test:tauri-webdriver -- --strict",
      "NEDITOR_PLATFORM_EVIDENCE_PLATFORM=linux pnpm run collect:platform-evidence",
      "pnpm run check:platform-evidence",
    ],
    returns: [".tmp/platform-evidence/external/linux/package-artifacts.json", ".tmp/platform-evidence/external/linux/tauri-webdriver-report.json"],
  },
  {
    file: "runbooks/release-signing.md",
    title: "Credentialed Release Signing Proof",
    gaps: ["release-signing-and-notarization"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run build",
      "./node_modules/.bin/tauri build --bundles all",
      "Run scripts/collect-release-signing-evidence.mjs with platform-specific --artifact and --proof arguments.",
      "pnpm run check:release-signing",
    ],
    returns: [
      ".tmp/release-signing/external/darwin/signing-evidence.json",
      ".tmp/release-signing/external/win32/signing-evidence.json",
      ".tmp/release-signing/external/linux/signing-evidence.json",
    ],
  },
  {
    file: "runbooks/google-docs-import.md",
    title: "Google Docs Live Import And Readback Proof",
    gaps: ["google-docs-live-import-readback"],
    commands: [
      "pnpm run test:rendered-exports",
      "Import .tmp/rendered-export-audit/rendered-export-audit.docx into native Google Docs with an authorized Drive session.",
      "Read back required document text markers and export DOCX from Google Docs.",
      "Run pnpm run collect:google-docs-import with --document-id, --document-title, --readback-text-file, and --exported-docx.",
      "pnpm run check:google-docs-import",
    ],
    returns: [".tmp/google-docs-import/external/import-evidence.json"],
  },
  {
    file: "runbooks/ai-provider-endpoint.md",
    title: "Approved AI Provider Live Endpoint Proof",
    gaps: ["ai-provider-live-endpoint-proof"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "Set NEDITOR_AI_PROVIDER_PROFILE, NEDITOR_AI_PROVIDER_ENDPOINT, NEDITOR_AI_PROVIDER_MODEL, and NEDITOR_AI_PROVIDER_API_KEY_ENV.",
      "Set the API key in the environment variable named by NEDITOR_AI_PROVIDER_API_KEY_ENV.",
      "pnpm run collect:ai-provider",
      "pnpm run check:ai-provider",
    ],
    returns: [".tmp/ai-provider-evidence/external/provider-evidence.json"],
  },
  {
    file: "runbooks/rendered-export-human-review.md",
    title: "Rendered Export Native-Viewer Human Signoff",
    gaps: ["rendered-export-native-viewer-human-signoff"],
    commands: [
      "pnpm run test:rendered-exports",
      "Review every primary and review-case artifact in native/browser viewers.",
      "Fill templates/rendered-export/visual-review-signoff.template.json.",
      "NEDITOR_RENDERED_EXPORT_SIGNOFF=/path/to/completed-signoff.json pnpm run test:rendered-exports -- --validate-signoff-only",
    ],
    returns: ["completed visual-review-signoff JSON"],
  },
  {
    file: "runbooks/accessibility-human-review.md",
    title: "Assistive-Technology Human Signoff",
    gaps: ["accessibility-assistive-technology-human-signoff"],
    commands: [
      "pnpm run check:a11y",
      "pnpm run check:a11y:runtime",
      "pnpm run check:a11y:manual",
      "Perform screen-reader, keyboard-only, native-shell, and export-artifact sessions.",
      "Fill templates/accessibility/manual-review-template.json.",
      "NEDITOR_ACCESSIBILITY_SIGNOFF=/path/to/completed-signoff.json pnpm run check:a11y:manual",
    ],
    returns: ["completed accessibility manual-review signoff JSON"],
  },
];

rmSync(outputDir, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
mkdirSync(outputDir, { recursive: true });

const copiedTemplates = copyTemplates();
const staleTemplates = copiedTemplates.filter((template) => template.copied && template.freshness.status === "stale");
const manifest = {
  schema: "neditor.release-evidence-kit.v1",
  generatedAt: new Date().toISOString(),
  appVersion: packageJson.version,
  sourceCommit,
  sourceTreeClean,
  readinessStatus: readiness?.status || "unknown",
  releaseReadinessReport: relative(join(root, ".tmp", "release-readiness", "report.json")),
  gaps: gaps.map((gap) => ({
    id: gap.id || gap.check || gap.name,
    detail: gap.detail || gap.reason || gap.message || "",
    evidence: gap.evidence || null,
  })),
  copiedTemplates,
  missingTemplates: copiedTemplates.filter((template) => !template.copied),
  staleTemplates,
  runbooks: runbooks.map((runbook) => ({
    title: runbook.title,
    path: runbook.file,
    gaps: runbook.gaps,
    returns: runbook.returns,
  })),
};

writeRunbooks();
writeFileSync(join(outputDir, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
writeFileSync(join(outputDir, "README.md"), readme(manifest));

console.log(`Release evidence kit written to ${relative(outputDir)}.`);
if (!sourceTreeClean) {
  console.log("Source tree is currently dirty; regenerate this kit from a clean checkout before sending it to external reviewers.");
}
if (manifest.missingTemplates.length > 0) {
  console.log(`Missing ${manifest.missingTemplates.length} template(s); run the listed prerequisite checks and regenerate the kit.`);
}
if (manifest.staleTemplates.length > 0) {
  console.log(`Stale ${manifest.staleTemplates.length} template(s); rerun prerequisite checks from the current clean source and regenerate the kit.`);
}

function copyTemplates() {
  return templateCopies.map(([from, to]) => {
    const source = join(root, from);
    const destination = join(outputDir, to);
    const copied = existsSync(source);
    const freshness = copied ? inspectTemplateFreshness(source) : { status: "missing", issues: ["template file is missing"] };
    if (copied) {
      mkdirSync(dirname(destination), { recursive: true });
      cpSync(source, destination);
    }
    return {
      source: from,
      path: to,
      copied,
      freshness,
    };
  });
}

function inspectTemplateFreshness(path) {
  let template;
  try {
    template = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    return {
      status: "stale",
      issues: [`template is not valid JSON: ${String(error)}`],
    };
  }

  const issues = [];
  if (template.appVersion && template.appVersion !== packageJson.version) {
    issues.push(`appVersion ${template.appVersion} does not match ${packageJson.version}`);
  }
  if (template.releaseVersion && template.releaseVersion !== packageJson.version) {
    issues.push(`releaseVersion ${template.releaseVersion} does not match ${packageJson.version}`);
  }
  if (template.sourceCommit && template.sourceCommit !== sourceCommit) {
    issues.push(`sourceCommit ${template.sourceCommit} does not match ${sourceCommit || "<unknown>"}`);
  }
  if (template.sourceTreeClean !== undefined && template.sourceTreeClean !== true) {
    issues.push("sourceTreeClean is not true");
  }

  return {
    status: issues.length > 0 ? "stale" : "current",
    schema: template.schema || null,
    generatedAt: template.generatedAt || null,
    appVersion: template.appVersion || template.releaseVersion || null,
    sourceCommit: template.sourceCommit || null,
    sourceTreeClean: template.sourceTreeClean ?? null,
    issues,
  };
}

function writeRunbooks() {
  for (const runbook of runbooks) {
    const body = [
      `# ${runbook.title}`,
      "",
      `Source commit: \`${sourceCommit || "<source-commit>"}\``,
      `App version: \`${packageJson.version}\``,
      "",
      "The source tree must be clean when evidence is collected. `git status --porcelain` must print nothing.",
      "",
      "## Commands",
      "",
      ...runbook.commands.map((command) => `- \`${command}\``),
      "",
      "## Return Evidence",
      "",
      ...runbook.returns.map((item) => `- \`${item}\``),
      "",
    ].join("\n");
    const destination = join(outputDir, runbook.file);
    mkdirSync(dirname(destination), { recursive: true });
    writeFileSync(destination, body);
  }
}

function readme(manifest) {
  const gapLines = manifest.gaps.length
    ? manifest.gaps.map((gap) => `- \`${gap.id}\`: ${gap.detail}`).join("\n")
    : "- No external gaps were present in the current release readiness report.";
  const runbookLines = manifest.runbooks.map((runbook) => `- [${runbook.title}](${runbook.path})`).join("\n");
  const missingLines = manifest.missingTemplates.length
    ? manifest.missingTemplates.map((template) => `- \`${template.source}\``).join("\n")
    : "- None.";
  const staleLines = manifest.staleTemplates.length
    ? manifest.staleTemplates.map((template) => `- \`${template.source}\`: ${template.freshness.issues.join("; ")}`).join("\n")
    : "- None.";
  return `${[
    "# NEditor Release Evidence Kit",
    "",
    `Generated: ${manifest.generatedAt}`,
    `App version: ${manifest.appVersion}`,
    `Source commit: ${manifest.sourceCommit || "<unknown>"}`,
    `Source tree clean at generation: ${manifest.sourceTreeClean ? "yes" : "no"}`,
    `Release readiness status: ${manifest.readinessStatus}`,
    "",
    "## Remaining Evidence Gaps",
    "",
    gapLines,
    "",
    "## Runbooks",
    "",
    runbookLines,
    "",
    "## Missing Templates",
    "",
    missingLines,
    "",
    "## Stale Templates",
    "",
    staleLines,
    "",
    "## Ingest Returned Evidence",
    "",
    "Place returned proof files under a return directory using any path listed in the runbooks, then run `pnpm run ingest:evidence -- --source /path/to/return-dir`.",
    "Use `pnpm run ingest:evidence -- --list` to print every recognized return path.",
    "",
    "Completed evidence must match the current app version, source commit, and clean source-tree requirements enforced by the validators.",
    "",
  ].join("\n")}\n`;
}

function readJson(path) {
  if (!existsSync(path)) return null;
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch {
    return null;
  }
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
  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 && result.stdout.trim() === "";
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
