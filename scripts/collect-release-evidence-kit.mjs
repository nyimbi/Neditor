import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputDir = resolve(process.env.NEDITOR_RELEASE_EVIDENCE_KIT_DIR || join(root, ".tmp", "release-evidence-kit"));
const sourceCommit = gitCommit();
const sourceTreeClean = gitTreeClean();
const readiness = readJson(join(root, ".tmp", "release-readiness", "report.json"));
const specWorkOrderSchema = "neditor.spec-completion-work-orders.v1";
const specWorkOrdersPath = join(root, ".tmp", "spec-completion", "work-orders.json");
const specWorkOrdersMarkdownPath = join(root, ".tmp", "spec-completion", "work-orders.md");
const specWorkOrders = readJson(specWorkOrdersPath);
const readinessStatus = effectiveReadinessStatus(readiness);
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
  [".tmp/ai-runtime-evidence/templates/runtime-evidence.template.json", "templates/ai-runtime/runtime-evidence.template.json"],
  [".tmp/ai-runtime-evidence/templates/runtime-readiness.template.json", "templates/ai-runtime/runtime-readiness.template.json"],
  [".tmp/security-review/templates/security-review.template.json", "templates/security/security-review.template.json"],
  [".tmp/performance-profile/templates/native-profile.template.json", "templates/performance/native-profile.template.json"],
  [".tmp/performance-profile/templates/native-profile-metrics.template.json", "templates/performance/native-profile-metrics.template.json"],
  [".tmp/google-docs-import/import-evidence.template.json", "templates/google-docs/import-evidence.template.json"],
  [".tmp/rendered-export-audit/visual-review-signoff.template.json", "templates/rendered-export/visual-review-signoff.template.json"],
  [".tmp/table-editor/manual-review-template.json", "templates/table-editor/manual-review-template.json"],
  [".tmp/accessibility/manual-review-template.json", "templates/accessibility/manual-review-template.json"],
  [".tmp/external-engines/templates/pikchr.template.json", "templates/external-engines/pikchr.template.json"],
  [".tmp/external-engines/templates/sqlite.template.json", "templates/external-engines/sqlite.template.json"],
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
    gaps: ["release-signing-and-notarization", "homebrew-macos-signing"],
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
    file: "runbooks/homebrew-release.md",
    title: "Homebrew Cask Release Proof",
    gaps: ["homebrew-final-cask", "homebrew-release-artifact"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run prepare:sidecars",
      "pnpm run build",
      "./node_modules/.bin/tauri build --bundles app,dmg",
      "Produce a signed and notarized macOS zip or DMG containing NEditor.app and the ned CLI helper.",
      `pnpm run release:homebrew -- --artifact /path/to/NEditor-${packageJson.version}-macos.zip --output Casks/neditor.rb`,
      "Confirm .tmp/homebrew/materialize-cask-report.json names the final artifact and SHA-256.",
      `NEDITOR_HOMEBREW_CASK=Casks/neditor.rb NEDITOR_HOMEBREW_ARTIFACT=/path/to/NEditor-${packageJson.version}-macos.zip pnpm run check:homebrew`,
      "brew audit --cask --new Casks/neditor.rb",
      "brew install --cask Casks/neditor.rb",
      "brew uninstall --cask neditor",
      "Copy the completed Casks/neditor.rb, release artifact, and .tmp/homebrew/materialize-cask-report.json into the return bundle.",
      "pnpm run check:homebrew",
    ],
    returns: [
      "homebrew/neditor.rb",
      `homebrew/NEditor-${packageJson.version}-macos.zip or homebrew/NEditor-${packageJson.version}-macos.dmg`,
      "homebrew/materialize-cask-report.json",
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
    file: "runbooks/ai-runtime-device.md",
    title: "Docs Live Runtime Device Proof",
    gaps: ["ai-runtime-real-device-proof"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run build",
      "Open Docs Live in the packaged Tauri WebView or target browser.",
      "Use Check AI runtime with microphone permission granted and rich/plain clipboard text available.",
      "Save the JSON-shaped runtime readiness result using templates/ai-runtime/runtime-readiness.template.json as the expected shape; do not store audio or clipboard text.",
      "Run pnpm run collect:ai-runtime -- --readiness-json /path/to/runtime-readiness.json --microphone-result stream-opened --clipboard-write-succeeded true.",
      "pnpm run check:ai-runtime",
    ],
    returns: [".tmp/ai-runtime-evidence/external/runtime-evidence.json"],
  },
  {
    file: "runbooks/independent-security-review.md",
    title: "Independent Security Review Signoff",
    gaps: ["independent-security-review-signoff"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run verify:local -- --list",
      "Review docs/security-threat-model.md plus the Tauri command surface, file/Git/snapshot/transform/provider boundaries, persistence migration, and release evidence contracts.",
      "Run pnpm run collect:security-review -- --report-file /path/to/security-report.md --tool-output-file /path/to/scanner-output.txt --reviewer-name \"Reviewer Name\" --reviewer-organization \"Independent Org\".",
      "pnpm run check:security-review",
    ],
    returns: [".tmp/security-review/external/security-review.json"],
  },
  {
    file: "runbooks/rendered-export-human-review.md",
    title: "Rendered Export Native-Viewer Human Signoff",
    gaps: ["rendered-export-native-viewer-human-signoff", "rendered-export-automated-visual-proof"],
    commands: [
      "pnpm run test:rendered-exports",
      "Review every primary and review-case artifact in native/browser viewers.",
      "Fill templates/rendered-export/visual-review-signoff.template.json.",
      "NEDITOR_RENDERED_EXPORT_SIGNOFF=/path/to/completed-signoff.json pnpm run test:rendered-exports -- --validate-signoff-only",
    ],
    returns: ["completed visual-review-signoff JSON", ".tmp/rendered-export-audit/visual-review-summary.json"],
  },
  {
    file: "runbooks/macos-native-launch.md",
    title: "macOS Native Launch And WebDriver Proof",
    gaps: ["macos-native-launch-current-binary-proof", "macos-native-window-visibility-proof", "macos-webdriver-current-binary-proof"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run build",
      "./node_modules/.bin/tauri build --no-bundle",
      "pnpm run test:desktop-smoke",
      "NEDITOR_DESKTOP_SMOKE_LAUNCH=1 pnpm run test:desktop-smoke",
      "pnpm run test:tauri-webdriver",
      "pnpm run check:release-readiness",
    ],
    returns: [
      ".tmp/desktop-smoke/launch-report.json",
      ".tmp/desktop-smoke/native-window-report.json",
      ".tmp/desktop-smoke/native-ui-report.json",
      ".tmp/desktop-smoke/native-workflow-report.json",
      ".tmp/desktop-webdriver/report.json",
    ],
  },
  {
    file: "runbooks/release-device-performance-profile.md",
    title: "Release Device Native Performance Profile",
    gaps: ["release-device-native-performance-profile"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run build",
      "./node_modules/.bin/tauri build --no-bundle",
      "Run a 30+ minute native release-device profile covering startup/open, large document edit/preview, export suite, file-watch conflict, and Agent Workspace review workflows.",
      "Write profiler metrics using templates/performance/native-profile-metrics.template.json: durationMinutes, scenarios, soak, exports, environment, and artifact notes.",
      "Run pnpm run collect:performance-profile -- --metrics /path/to/metrics.json --summary-artifact /path/to/summary.txt --trace-artifact /path/to/trace.json --reviewer-name \"Reviewer Name\".",
      "pnpm run check:performance-profile",
    ],
    returns: [".tmp/performance-profile/external/native-profile.json"],
  },
  {
    file: "runbooks/table-editor-human-review.md",
    title: "Table Editor Two-Way Human Signoff",
    gaps: ["table-editor-manual-supported-host-signoff"],
    commands: [
      "pnpm run test:unit",
      "pnpm run test:e2e",
      "pnpm run test:rendered-exports",
      "pnpm run test:desktop-smoke",
      "pnpm run check:tables:manual",
      "Perform source-to-grid, grid-to-source, concurrent-source protection, spreadsheet exchange, rendered export, keyboard/accessibility, and supported-host table sessions.",
      "Run pnpm run collect:tables:manual -- --template templates/table-editor/manual-review-template.json --reviewer-name \"Reviewer Name\" --platform-version \"OS Version\" --platform-device \"Device\" --webview-or-browser \"Tauri WebView\" --evidence-reference /path/to/table-review-artifacts --notes \"Reviewed source/grid/export workflows with no blockers.\".",
      "NEDITOR_TABLE_EDITOR_SIGNOFF=/path/to/completed-signoff.json pnpm run check:tables:manual",
    ],
    returns: ["completed table editor manual-review signoff JSON"],
  },
  {
    file: "runbooks/accessibility-human-review.md",
    title: "Assistive-Technology Human Signoff",
    gaps: ["accessibility-assistive-technology-human-signoff", "runtime-accessibility-browser-proof"],
    commands: [
      "pnpm run check:a11y",
      "pnpm run check:a11y:runtime",
      "pnpm run check:a11y:manual",
      "Perform screen-reader, keyboard-only, native-shell, and export-artifact sessions.",
      "Fill templates/accessibility/manual-review-template.json.",
      "NEDITOR_ACCESSIBILITY_SIGNOFF=/path/to/completed-signoff.json pnpm run check:a11y:manual",
    ],
    returns: ["completed accessibility manual-review signoff JSON", ".tmp/accessibility/runtime-report.json"],
  },
  {
    file: "runbooks/manual-review.md",
    title: "Spec Manual Review Work-Order Signoffs",
    gaps: ["spec-manual-review-work-order-signoffs"],
    commands: [
      "pnpm run check:spec-completion",
      "pnpm run check:manual-review",
      "Open manual-review/dashboard.html or manual-review/dashboard.md in this evidence kit and locate the assigned manual-review work order.",
      "Fill templates/spec-manual-review/<work-order-id>.template.json with named reviewer, current app version, current source commit, clean-source provenance, artifact paths, checklist outcomes, and zero unresolved blockers.",
      "Put screenshots or native-viewer/export evidence under .tmp/manual-review/<work-order-id>/artifacts/.",
      "pnpm run ingest:evidence -- --source .tmp/manual-review/<work-order-id>",
      "pnpm run check:manual-review",
      "pnpm run check:release-readiness",
    ],
    returns: [".tmp/manual-review/<work-order-id>/signoff.json", ".tmp/manual-review/<work-order-id>/artifacts/"],
  },
  {
    file: "runbooks/optional-external-engines.md",
    title: "Optional External Engine Proof",
    gaps: ["optional-external-engines"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "Install or build the missing optional engine, such as Pikchr or sqlite3, on the verifier host.",
      "Run pnpm run collect:engine-evidence with the engine path environment variable when needed, for example NEDITOR_TEST_PIKCHR=/absolute/path/to/pikchr NEDITOR_TEST_SQLITE3=/absolute/path/to/sqlite3 pnpm run collect:engine-evidence.",
      "Copy the generated proof from .tmp/external-engines/external/, for example pikchr.json or sqlite.json.",
      "NEDITOR_EXTERNAL_ENGINE_EVIDENCE_DIR=.tmp/external-engines/external pnpm run check:engines",
    ],
    returns: [".tmp/external-engines/external/pikchr.json", ".tmp/external-engines/external/sqlite.json"],
  },
  {
    file: "runbooks/spec-completion-closure.md",
    title: "Spec Completion Matrix Closure",
    gaps: ["spec-completion-open-items", "homebrew-release-readiness"],
    commands: [
      "git fetch --all --tags",
      `git checkout ${sourceCommit || "<source-commit>"}`,
      "git status --porcelain",
      "pnpm install --frozen-lockfile",
      "pnpm run check:spec-completion",
      "Open .tmp/spec-completion/report.json and work each openRows item until direct current evidence proves it.",
      "Update docs/spec-completion-matrix.md only after the implementation, tests, artifacts, or platform proof exist.",
      "pnpm run verify:local -- --list",
      "pnpm run check:spec-completion",
      "pnpm run check:release-readiness",
    ],
    returns: ["updated docs/spec-completion-matrix.md", ".tmp/spec-completion/report.json"],
  },
];

rmSync(outputDir, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
mkdirSync(outputDir, { recursive: true });

const copiedTemplates = copyTemplates();
const copiedSpecWorkOrders = copySpecWorkOrders();
const copiedSpecWorkOrderRunbooks = copySpecWorkOrderRunbooks();
const copiedManualReviewAssets = copyManualReviewAssets();
const staleTemplates = copiedTemplates.filter((template) => template.copied && template.freshness.status === "stale");
const manifest = {
  schema: "neditor.release-evidence-kit.v1",
  generatedAt: new Date().toISOString(),
  appVersion: packageJson.version,
  sourceCommit,
  sourceTreeClean,
  readinessStatus,
  releaseReadinessReport: relative(join(root, ".tmp", "release-readiness", "report.json")),
  gaps: gaps.map((gap) => ({
    id: gap.id || gap.check || gap.name,
    detail: gap.detail || gap.reason || gap.message || "",
    evidence: gap.evidence || null,
  })),
  copiedTemplates,
  manualReviewAssets: copiedManualReviewAssets,
  specCompletionWorkOrders: copiedSpecWorkOrders,
  specCompletionRunbooks: copiedSpecWorkOrderRunbooks,
  missingTemplates: copiedTemplates.filter((template) => !template.copied),
  missingSpecCompletionRunbooks: copiedSpecWorkOrderRunbooks.filter((runbook) => !runbook.available),
  staleTemplates,
  runbooks: runbooks.map((runbook) => ({
    title: runbook.title,
    path: runbook.file,
    gaps: runbook.gaps,
    returns: runbook.returns,
    validatorCommands: validatorCommandsForRunbook(runbook),
    ingestCommand: ingestCommand(),
    finalReadinessCommand: finalReadinessCommand(),
  })),
  gapWorkItems: gapWorkItems(),
};

writeRunbooks();
writeFileSync(join(outputDir, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
writeFileSync(join(outputDir, "README.md"), readme(manifest));
writeEvidenceKitReport(manifest);

console.log(`Release evidence kit written to ${relative(outputDir)}.`);
if (!sourceTreeClean) {
  console.log("Source tree is currently dirty; regenerate this kit from a clean checkout before sending it to external reviewers.");
}
if (manifest.missingTemplates.length > 0) {
  console.log(`Missing ${manifest.missingTemplates.length} template(s); run the listed prerequisite checks and regenerate the kit.`);
}
if (manifest.missingSpecCompletionRunbooks.length > 0) {
  console.log(`Missing ${manifest.missingSpecCompletionRunbooks.length} spec-completion runbook(s); add the missing runbooks and regenerate the kit.`);
}
if (manifest.staleTemplates.length > 0) {
  console.log(`Stale ${manifest.staleTemplates.length} template(s); rerun prerequisite checks from the current clean source and regenerate the kit.`);
}

function writeEvidenceKitReport(manifest) {
  const reportIssues = [];
  if (!sourceTreeClean) reportIssues.push("source-tree-not-clean");
  if (manifest.missingTemplates.length > 0) reportIssues.push(`missing-templates=${manifest.missingTemplates.length}`);
  if (manifest.missingSpecCompletionRunbooks.length > 0) {
    reportIssues.push(`missing-spec-completion-runbooks=${manifest.missingSpecCompletionRunbooks.length}`);
  }
  if (manifest.staleTemplates.length > 0) reportIssues.push(`stale-templates=${manifest.staleTemplates.length}`);
  if (copiedSpecWorkOrders.total !== copiedSpecWorkOrders.readyToSend) {
    reportIssues.push(`spec-work-orders-not-ready=${copiedSpecWorkOrders.total - copiedSpecWorkOrders.readyToSend}`);
  }
  if (
    copiedManualReviewAssets.expectedManualWorkOrders > 0 &&
    copiedManualReviewAssets.templates.total < copiedManualReviewAssets.expectedManualWorkOrders
  ) {
    reportIssues.push(
      `manual-review-templates-missing=${copiedManualReviewAssets.expectedManualWorkOrders - copiedManualReviewAssets.templates.total}`,
    );
  }
  if (
    copiedManualReviewAssets.expectedManualWorkOrders > 0 &&
    (!copiedManualReviewAssets.dashboardMarkdown.copied || !copiedManualReviewAssets.assignmentsCsv.copied)
  ) {
    reportIssues.push("manual-review-dashboard-missing");
  }
  writeFileSync(
    join(outputDir, "report.json"),
    `${JSON.stringify(
      {
        schema: "neditor.release-evidence-kit-report.v1",
        generatedAt: new Date().toISOString(),
        status: reportIssues.length === 0 ? "passed" : "failed",
        manifestPath: relative(join(outputDir, "manifest.json")),
        releaseReadinessReport: relative(join(root, ".tmp", "release-readiness", "report.json")),
        sourceCommit,
        currentSourceCommit: sourceCommit,
        sourceTreeClean,
        currentSourceTreeClean: sourceTreeClean,
        appVersion: packageJson.version,
        currentAppVersion: packageJson.version,
        readinessStatus,
        currentReadinessStatus: readinessStatus,
        summary: {
          gaps: manifest.gaps.length,
          copiedTemplates: manifest.copiedTemplates.length,
          missingTemplates: manifest.missingTemplates.length,
          missingSpecCompletionRunbooks: manifest.missingSpecCompletionRunbooks.length,
          staleTemplates: manifest.staleTemplates.length,
          runbooks: manifest.runbooks.length,
          specCompletionRunbooks: manifest.specCompletionRunbooks.length,
          manualReviewTemplates: copiedManualReviewAssets.templates.total,
          manualReviewDashboardCopied: copiedManualReviewAssets.dashboardMarkdown.copied || copiedManualReviewAssets.dashboardHtml.copied,
          specWorkOrders: copiedSpecWorkOrders.total,
          specWorkOrdersReady: copiedSpecWorkOrders.readyToSend,
          issues: reportIssues.length,
        },
        gapIds: manifest.gaps.map((gap) => gap.id),
        issues: reportIssues,
      },
      null,
      2,
    )}\n`,
  );
}

function copyManualReviewAssets() {
  const sourceTemplateDir = join(root, ".tmp", "manual-review", "templates");
  const destinationTemplateDir = join(outputDir, "templates", "spec-manual-review");
  const dashboardMarkdownSource = join(root, ".tmp", "manual-review", "dashboard.md");
  const dashboardHtmlSource = join(root, ".tmp", "manual-review", "dashboard.html");
  const assignmentsCsvSource = join(root, ".tmp", "manual-review", "assignments.csv");
  const copiedTemplateFiles = [];
  const expectedManualWorkOrders = Array.isArray(specWorkOrders?.workOrders)
    ? specWorkOrders.workOrders.filter((order) => order.classification === "manual-review").length
    : 0;

  if (existsSync(sourceTemplateDir)) {
    mkdirSync(destinationTemplateDir, { recursive: true });
    for (const file of readdirSync(sourceTemplateDir).filter((name) => name.endsWith(".template.json")).sort()) {
      const source = join(sourceTemplateDir, file);
      const destination = join(destinationTemplateDir, file);
      if (!statSync(source).isFile()) continue;
      cpSync(source, destination);
      copiedTemplateFiles.push(`templates/spec-manual-review/${file}`);
    }
  }

  return {
    expectedManualWorkOrders,
    templates: {
      sourceDir: ".tmp/manual-review/templates",
      path: "templates/spec-manual-review",
      copied: copiedTemplateFiles.length > 0,
      total: copiedTemplateFiles.length,
      files: copiedTemplateFiles,
    },
    dashboardMarkdown: copyOptionalManualAsset(dashboardMarkdownSource, "manual-review/dashboard.md"),
    dashboardHtml: copyOptionalManualAsset(dashboardHtmlSource, "manual-review/dashboard.html"),
    assignmentsCsv: copyOptionalManualAsset(assignmentsCsvSource, "manual-review/assignments.csv"),
  };
}

function copyOptionalManualAsset(source, destinationPath) {
  const copied = existsSync(source) && statSync(source).isFile();
  if (copied) {
    const destination = join(outputDir, destinationPath);
    mkdirSync(dirname(destination), { recursive: true });
    cpSync(source, destination);
  }
  return {
    source: relative(source),
    path: destinationPath,
    copied,
  };
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

function copySpecWorkOrders() {
  const jsonDestination = join(outputDir, "spec-completion", "work-orders.json");
  const markdownDestination = join(outputDir, "spec-completion", "work-orders.md");
  const jsonCopied = existsSync(specWorkOrdersPath);
  const markdownCopied = existsSync(specWorkOrdersMarkdownPath);
  if (jsonCopied) {
    mkdirSync(dirname(jsonDestination), { recursive: true });
    cpSync(specWorkOrdersPath, jsonDestination);
  }
  if (markdownCopied) {
    mkdirSync(dirname(markdownDestination), { recursive: true });
    cpSync(specWorkOrdersMarkdownPath, markdownDestination);
  }
  return {
    jsonPath: "spec-completion/work-orders.json",
    markdownPath: "spec-completion/work-orders.md",
    jsonCopied,
    markdownCopied,
    expectedSchema: specWorkOrderSchema,
    schema: specWorkOrders?.schema || null,
    total: Number(specWorkOrders?.summary?.total || specWorkOrders?.workOrders?.length || 0),
    readyToSend: Number(specWorkOrders?.summary?.readyToSend || 0),
    generatedAt: specWorkOrders?.generatedAt || null,
  };
}

function copySpecWorkOrderRunbooks() {
  const generatedRunbookPaths = new Set(runbooks.map((runbook) => runbook.file));
  const referencedRunbookPaths = new Set(
    (Array.isArray(specWorkOrders?.workOrders) ? specWorkOrders.workOrders : [])
      .flatMap((order) => (Array.isArray(order.runbooks) ? order.runbooks : []))
      .filter(Boolean),
  );
  return [...referencedRunbookPaths].sort().map((runbookPath) => {
    const generatedByKit = generatedRunbookPaths.has(runbookPath);
    if (generatedByKit) {
      return {
        path: runbookPath,
        source: "generated-release-kit-runbook",
        available: true,
        copied: false,
        generatedByKit: true,
      };
    }
    const source = join(root, runbookPath);
    const destination = join(outputDir, runbookPath);
    const available = existsSync(source);
    if (available) {
      mkdirSync(dirname(destination), { recursive: true });
      cpSync(source, destination);
    }
    return {
      path: runbookPath,
      source: runbookPath,
      available,
      copied: available,
      generatedByKit: false,
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
    const validatorCommands = validatorCommandsForRunbook(runbook);
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
      "## Validate Before Returning",
      "",
      ...validatorCommands.map((command) => `- \`${command}\``),
      "",
      "## Ingest On The Release Host",
      "",
      `- \`${ingestCommand()}\``,
      `- \`${finalReadinessCommand()}\``,
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
  const workItemLines = manifest.gapWorkItems.length
    ? manifest.gapWorkItems
        .map((item) => [
          `- [ ] \`${item.id}\`: ${item.detail || "See release readiness report."}`,
          ...item.runbooks.map((runbook) => `  - Runbook: [${runbook.title}](${runbook.path})`),
          ...item.returns.map((returned) => `  - Return: \`${returned}\``),
          ...item.validatorCommands.map((command) => `  - Validate: \`${command}\``),
          `  - Ingest: \`${item.ingestCommand}\``,
          `  - Final readiness: \`${item.finalReadinessCommand}\``,
        ].join("\n"))
        .join("\n")
    : "- No external work items are required by the current release readiness report.";
  const missingLines = manifest.missingTemplates.length
    ? manifest.missingTemplates.map((template) => `- \`${template.source}\``).join("\n")
    : "- None.";
  const staleLines = manifest.staleTemplates.length
    ? manifest.staleTemplates.map((template) => `- \`${template.source}\`: ${template.freshness.issues.join("; ")}`).join("\n")
    : "- None.";
  const workOrders = manifest.specCompletionWorkOrders || {};
  const manualReviewAssets = manifest.manualReviewAssets || {};
  const specRunbookLines = manifest.specCompletionRunbooks.length
    ? manifest.specCompletionRunbooks
        .map((runbook) => `- [${runbook.path}](${runbook.path})${runbook.generatedByKit ? " (generated for this kit)" : ""}`)
        .join("\n")
    : "- None.";
  const workOrderLines = workOrders.jsonCopied
    ? [
        `- JSON: [${workOrders.jsonPath}](${workOrders.jsonPath})`,
        `- Markdown: ${workOrders.markdownCopied ? `[${workOrders.markdownPath}](${workOrders.markdownPath})` : "missing"}`,
        `- Ready work orders: ${workOrders.readyToSend}/${workOrders.total}`,
      ].join("\n")
    : "- Spec-completion work orders are missing. Run `pnpm run check:spec-completion`, then regenerate this kit.";
  const manualTemplateLines = manualReviewAssets.templates?.copied
    ? [
        `- Templates: [${manualReviewAssets.templates.path}](${manualReviewAssets.templates.path}) (${manualReviewAssets.templates.total} files)`,
        `- Dashboard: ${manualReviewAssets.dashboardHtml?.copied ? `[${manualReviewAssets.dashboardHtml.path}](${manualReviewAssets.dashboardHtml.path})` : "missing"}`,
        `- Markdown dashboard: ${manualReviewAssets.dashboardMarkdown?.copied ? `[${manualReviewAssets.dashboardMarkdown.path}](${manualReviewAssets.dashboardMarkdown.path})` : "missing"}`,
        `- Assignment CSV: ${manualReviewAssets.assignmentsCsv?.copied ? `[${manualReviewAssets.assignmentsCsv.path}](${manualReviewAssets.assignmentsCsv.path})` : "missing"}`,
      ].join("\n")
    : "- Spec manual-review templates are missing. Run `pnpm run check:manual-review`, then regenerate this kit.";
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
    "## Gap Work Items",
    "",
    workItemLines,
    "",
    "## Spec Completion Work Orders",
    "",
    workOrderLines,
    "",
    "## Spec Manual Review Templates",
    "",
    manualTemplateLines,
    "",
    "## Spec Completion Runbooks",
    "",
    specRunbookLines,
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

function gapWorkItems() {
  return gaps.map((gap) => {
    const id = gap.id || gap.check || gap.name || "unknown-gap";
    const matchingRunbooks = runbooks.filter((runbook) => runbook.gaps.includes(id));
    const returns = Array.from(new Set(matchingRunbooks.flatMap((runbook) => runbook.returns)));
    const validatorCommands = Array.from(new Set(matchingRunbooks.flatMap((runbook) => validatorCommandsForRunbook(runbook))));
    return {
      id,
      status: gap.status || "pending",
      detail: gap.detail || gap.reason || gap.message || "",
      evidence: gap.evidence || null,
      runbooks: matchingRunbooks.map((runbook) => ({
        title: runbook.title,
        path: runbook.file,
      })),
      returns,
      validatorCommands,
      ingestCommand: ingestCommand(),
      finalReadinessCommand: finalReadinessCommand(),
      readyToSend: matchingRunbooks.length > 0 && returns.length > 0 && validatorCommands.length > 0,
    };
  });
}

function validatorCommandsForRunbook(runbook) {
  const commands = runbook.commands.filter(
    (command) => /\bpnpm run (check:[a-z0-9:-]+|test:[a-z0-9:-]+)\b/i.test(command) || /^NEDITOR_[A-Z0-9_]+=/.test(command),
  );
  return Array.from(new Set(commands));
}

function ingestCommand() {
  return "pnpm run ingest:evidence -- --source /path/to/return-dir";
}

function finalReadinessCommand() {
  return "pnpm run check:release-readiness";
}

function effectiveReadinessStatus(readinessReport) {
  if (!readinessReport) return "unknown";
  const status = readinessReport.status || "unknown";
  const failures = Array.isArray(readinessReport.failures) ? readinessReport.failures : [];
  const onlyEvidenceKitBootstrapFailure =
    status === "failed" && failures.length > 0 && failures.every((failure) => String(failure).startsWith("release-evidence-kit "));
  if (!onlyEvidenceKitBootstrapFailure) return status;
  return Number(readinessReport.summary?.evidenceGaps || 0) > 0 ? "current-host-ready-with-external-gaps" : "ready";
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
