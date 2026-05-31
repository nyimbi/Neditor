import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const workflowPath = join(root, ".github", "workflows", "neditor-release-evidence.yml");
const reportPath = join(root, ".tmp", "release-ci", "workflow-report.json");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const issues = [];

if (!existsSync(workflowPath)) {
  issues.push("missing .github/workflows/neditor-release-evidence.yml");
} else {
  const workflow = readFileSync(workflowPath, "utf8");
  validateWorkflow(workflow);
}

if (packageJson.scripts?.["check:release-ci"] !== "node scripts/check-release-ci-workflow.mjs") {
  issues.push("package.json must expose check:release-ci");
}

writeReport();

if (issues.length > 0) {
  console.error("Release CI workflow validation failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

console.log(`Release CI workflow is wired; wrote ${relative(reportPath)}.`);

function validateWorkflow(workflow) {
  requireIncludes(workflow, "name: NEditor Release Evidence", "workflow must be named NEditor Release Evidence");
  requireIncludes(workflow, "workflow_dispatch:", "workflow must be manually dispatchable");
  requireIncludes(workflow, "permissions:", "workflow must declare permissions");
  requireIncludes(workflow, "contents: read", "workflow must use read-only repository contents permission");
  requireIncludes(workflow, "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24", "workflow must opt JavaScript actions into Node 24");
  requireIncludes(workflow, "NEDITOR_TAURI_WEBDRIVER_TIMEOUT_MS", "workflow must allow enough time for hosted Tauri WebDriver startup");
  requireIncludes(workflow, "browser-workflows:", "workflow must include browser workflow proof job");
  requireIncludes(workflow, "platform-proof:", "workflow must include platform proof job");
  requireIncludes(workflow, "optional-engine-proof:", "workflow must include optional engine proof job");
  requireIncludes(workflow, "rendered-export-review:", "workflow must include rendered export review job");
  requireIncludes(workflow, "accessibility-review:", "workflow must include accessibility review job");
  requireIncludes(workflow, "ubuntu-latest", "workflow must run Linux proof on ubuntu-latest");
  requireIncludes(workflow, "windows-latest", "workflow must run Windows proof on windows-latest");
  requireIncludes(workflow, "platform: win32", "workflow matrix must include win32 platform");
  requireIncludes(workflow, "platform: linux", "workflow matrix must include linux platform");
  requireIncludes(workflow, "bundles: nsis", "Windows proof must build the hosted-runner-stable NSIS bundle target");
  requireIncludes(workflow, "bundles: deb rpm appimage", "Linux proof must build supported Linux bundle targets");
  requireIncludes(workflow, "pnpm exec playwright install --with-deps chromium", "browser job must install Chromium with host dependencies");
  requireIncludes(workflow, "pnpm run check:e2e-env", "browser job must run browser environment preflight");
  requireIncludes(workflow, "pnpm run test:e2e", "browser job must run full browser workflow suite");
  requireIncludes(workflow, "cargo install tauri-driver --locked", "platform job must install tauri-driver");
  requireIncludes(workflow, "webkit2gtk-driver", "Linux job must install WebKitGTK WebDriver");
  requireIncludes(workflow, "xvfb-run -a pnpm run test:tauri-webdriver -- --strict", "Linux job must run strict WebDriver workflow under xvfb");
  requireIncludes(workflow, "choco install selenium-chromium-edge-driver", "Windows job must install Edge WebDriver");
  requireIncludes(workflow, "MSEDGEDRIVER_TELEMETRY_OPTOUT", "Windows WebDriver job must opt out of EdgeDriver telemetry");
  requireIncludes(workflow, "pnpm run test:tauri-webdriver -- --strict", "Windows job must run strict WebDriver workflow");
  requireIncludes(workflow, "pnpm tauri build --bundles ${{ matrix.bundles }}", "platform job must build matrix-supported Tauri package targets");
  requireIncludes(workflow, "NEDITOR_PLATFORM_EVIDENCE_PLATFORM: ${{ matrix.platform }}", "platform job must set evidence platform");
  requireIncludes(workflow, "pnpm run collect:platform-evidence", "platform job must collect validator-ready evidence");
  requireIncludes(workflow, "pnpm run check:platform-evidence", "platform job must validate collected evidence");
  requireIncludes(workflow, "actions/upload-artifact@v7", "workflow must upload evidence artifacts with the Node 24 action runtime");
  requireIncludes(workflow, ".tmp/e2e-browser/report.json", "browser report must be uploaded");
  requireIncludes(workflow, ".tmp/platform-evidence/external/${{ matrix.platform }}/package-artifacts.json", "package evidence must be uploaded");
  requireIncludes(workflow, ".tmp/platform-evidence/external/${{ matrix.platform }}/tauri-webdriver-report.json", "WebDriver evidence must be uploaded");
  requireIncludes(workflow, "${{ matrix.artifact }}-json", "platform JSON evidence must be uploaded separately for lightweight ingest");
  requireIncludes(workflow, "src-tauri/target/release/bundle/**", "built package artifacts must be uploaded for inspection");
  requireIncludes(workflow, "pnpm run collect:engine-evidence -- --require-installed", "optional engine job must collect required installed-engine evidence");
  requireIncludes(workflow, "graphviz d2 plantuml sqlite", "Windows optional engine job must install Graphviz, D2, PlantUML, and SQLite");
  requireIncludes(workflow, "cargo install pikchr-cli --locked", "optional engine job must install Pikchr CLI");
  requireIncludes(workflow, "d2-v0.7.1-windows-amd64.tar.gz", "Windows optional engine job must fall back to the official D2 Windows archive");
  requireIncludes(workflow, "NEDITOR_TEST_D2", "Windows optional engine job must pass discovered D2 executable path to the probe");
  requireIncludes(workflow, ".tmp/external-engines/external/${{ matrix.platform }}/**", "optional engine job must upload platform-qualified matrix engine evidence");
  requireIncludes(workflow, "neditor-optional-engine-evidence-win32", "optional engine job must upload Windows engine evidence artifact");
  requireIncludes(workflow, "pnpm run test:rendered-exports", "rendered export job must run the rendered export audit");
  requireIncludes(workflow, "poppler-utils", "rendered export job must install Poppler proof tools");
  requireIncludes(workflow, "libwebkit2gtk-4.1-dev", "rendered export job must install Tauri Linux build libraries");
  requireIncludes(workflow, ".tmp/rendered-export-audit/**", "rendered export review package must be uploaded");
  requireIncludes(workflow, "pnpm run check:a11y", "accessibility job must run static accessibility checks");
  requireIncludes(workflow, "pnpm run check:a11y:runtime", "accessibility job must run runtime accessibility checks");
  requireIncludes(workflow, "pnpm run check:a11y:manual", "accessibility job must generate the manual accessibility contract");
  requireIncludes(workflow, ".tmp/accessibility/**", "accessibility review package must be uploaded");
  if (/\t/.test(workflow)) issues.push("workflow must not contain tab indentation");
}

function requireIncludes(source, expected, message) {
  if (!source.includes(expected)) issues.push(message);
}

function writeReport() {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        schema: "neditor.release-ci-workflow-report.v1",
        generatedAt: new Date().toISOString(),
        status: issues.length === 0 ? "passed" : "failed",
        workflowPath: relative(workflowPath),
        packageScript: packageJson.scripts?.["check:release-ci"] || null,
        issues,
      },
      null,
      2,
    )}\n`,
  );
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
