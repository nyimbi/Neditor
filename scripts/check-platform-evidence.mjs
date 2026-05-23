import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const evidenceDir = resolve(process.env.NEDITOR_PLATFORM_EVIDENCE_DIR || join(root, ".tmp", "platform-evidence", "external"));
const reportPath = join(root, ".tmp", "platform-evidence", "report.json");
const templateDir = join(root, ".tmp", "platform-evidence", "templates");
const platformSpecs = [
  {
    platform: "win32",
    name: "Windows",
    packagePath: "win32/package-artifacts.json",
    webdriverPath: "win32/tauri-webdriver-report.json",
    artifactKinds: ["msi", "nsis", "exe"],
  },
  {
    platform: "linux",
    name: "Linux",
    packagePath: "linux/package-artifacts.json",
    webdriverPath: "linux/tauri-webdriver-report.json",
    artifactKinds: ["appimage", "deb", "rpm"],
  },
];

mkdirSync(templateDir, { recursive: true });
writeTemplates();

const platforms = platformSpecs.map((spec) => evaluatePlatform(spec));
const missingItems = platforms.flatMap((platform) => platform.missingEvidence);
const invalidItems = platforms.flatMap((platform) => platform.invalidEvidence);
const status = invalidItems.length > 0 ? "failed" : missingItems.length > 0 ? "pending-external-evidence" : "complete";

writeReport({
  generatedAt: new Date().toISOString(),
  platform: process.platform,
  arch: process.arch,
  status,
  evidenceDir: relative(evidenceDir),
  templateDir: relative(templateDir),
  summary: {
    requiredPlatforms: platformSpecs.length,
    completePlatforms: platforms.filter((platform) => platform.status === "complete").length,
    missingEvidence: missingItems.length,
    invalidEvidence: invalidItems.length,
  },
  platforms,
  missingEvidence: missingItems,
  invalidEvidence: invalidItems,
});

if (invalidItems.length > 0) {
  console.error("External platform evidence failed validation:");
  for (const issue of invalidItems) console.error(`- ${issue.detail}`);
  console.error(`Wrote ${relative(reportPath)}.`);
  process.exit(1);
}

console.log(`External platform evidence is ${status}; wrote ${relative(reportPath)}.`);

function evaluatePlatform(spec) {
  const packageEvidence = evaluatePackageArtifacts(spec);
  const webdriverEvidence = evaluateWebdriverReport(spec);
  const checks = [packageEvidence, webdriverEvidence];
  const invalidEvidence = checks.filter((check) => check.status === "invalid");
  const missingEvidence = checks.filter((check) => check.status === "missing");
  const status =
    invalidEvidence.length > 0 ? "failed" : missingEvidence.length > 0 ? "pending-external-evidence" : "complete";

  return {
    platform: spec.platform,
    name: spec.name,
    status,
    packageArtifacts: packageEvidence,
    webdriver: webdriverEvidence,
    missingEvidence: missingEvidence.map((check) => ({
      id: check.id,
      platform: spec.platform,
      path: check.path,
      detail: check.detail,
    })),
    invalidEvidence: invalidEvidence.map((check) => ({
      id: check.id,
      platform: spec.platform,
      path: check.path,
      detail: check.detail,
    })),
  };
}

function evaluatePackageArtifacts(spec) {
  const path = join(evidenceDir, spec.packagePath);
  if (!existsSync(path)) {
    return missingEvidence("package-artifacts", spec.packagePath, `${spec.name} package artifact evidence has not been supplied.`);
  }

  const evidence = readJsonEvidence(path, "package-artifacts", spec.packagePath);
  if (evidence.status === "invalid") return evidence;

  const report = evidence.report;
  const problems = [];
  requireValue(report.schema === "neditor.platform-package-artifacts.v1", problems, "schema must be neditor.platform-package-artifacts.v1");
  requireValue(report.platform === spec.platform, problems, `platform must be ${spec.platform}`);
  requireValue(report.status === "passed", problems, "status must be passed");
  requireValue(isIsoDate(report.generatedAt), problems, "generatedAt must be an ISO timestamp");
  requireValue(String(report.command || "").includes("tauri build"), problems, "command must identify the Tauri package build");
  const artifacts = Array.isArray(report.artifacts) ? report.artifacts : [];
  requireValue(artifacts.length > 0, problems, "artifacts must include at least one package artifact");
  for (const artifact of artifacts) {
    requireValue(spec.artifactKinds.includes(String(artifact.kind || "").toLowerCase()), problems, `artifact kind must be one of ${spec.artifactKinds.join(", ")}`);
    requireValue(Boolean(String(artifact.path || "").trim()), problems, "artifact path is required");
    requireValue(Number(artifact.bytes) > 1000, problems, `artifact ${artifact.path || "(unknown)"} must record bytes > 1000`);
    requireValue(isSha256(artifact.sha256), problems, `artifact ${artifact.path || "(unknown)"} must record a sha256`);
  }

  if (problems.length > 0) {
    return invalid("package-artifacts", spec.packagePath, `${spec.name} package artifact evidence is invalid: ${problems.join("; ")}`);
  }

  return {
    id: "package-artifacts",
    path: spec.packagePath,
    status: "accepted",
    detail: `${spec.name} package artifact evidence supplied for ${artifacts.map((artifact) => artifact.kind).join(", ")}.`,
    generatedAt: report.generatedAt,
    command: report.command,
    artifacts: artifacts.map((artifact) => ({
      kind: artifact.kind,
      path: artifact.path,
      bytes: artifact.bytes,
      sha256: artifact.sha256,
    })),
  };
}

function evaluateWebdriverReport(spec) {
  const path = join(evidenceDir, spec.webdriverPath);
  if (!existsSync(path)) {
    return missingEvidence("tauri-webdriver", spec.webdriverPath, `${spec.name} Tauri WebDriver report has not been supplied.`);
  }

  const evidence = readJsonEvidence(path, "tauri-webdriver", spec.webdriverPath);
  if (evidence.status === "invalid") return evidence;

  const report = evidence.report;
  const problems = [];
  requireValue(report.platform === spec.platform, problems, `platform must be ${spec.platform}`);
  requireValue(report.status === "passed", problems, "status must be passed");
  requireValue(isIsoDate(report.generatedAt), problems, "generatedAt must be an ISO timestamp");
  requireValue(Array.isArray(report.assertions) && report.assertions.length >= 8, problems, "assertions must include the desktop workflow proof");
  const assertionNames = new Set((report.assertions || []).map((assertion) => assertion.name));
  for (const requiredAssertion of [
    "initial native title includes NEditor",
    "desktop shell renders primary commands",
    "desktop WebDriver saves and reopens real Markdown file through dialog-free smoke path",
    "desktop WebDriver writes HTML export through dialog-free smoke path",
    "desktop preferences persist across WebDriver restart",
  ]) {
    requireValue(assertionNames.has(requiredAssertion), problems, `missing assertion: ${requiredAssertion}`);
  }
  requireValue(Number(report.fileArtifacts?.savedBytes) > 0, problems, "fileArtifacts.savedBytes must be present");
  requireValue(Number(report.exportArtifacts?.outputBytes) > 1000, problems, "exportArtifacts.outputBytes must be > 1000");
  requireValue(isSha256(report.exportArtifacts?.outputHash), problems, "exportArtifacts.outputHash must be a sha256");

  if (problems.length > 0) {
    return invalid("tauri-webdriver", spec.webdriverPath, `${spec.name} Tauri WebDriver evidence is invalid: ${problems.join("; ")}`);
  }

  return {
    id: "tauri-webdriver",
    path: spec.webdriverPath,
    status: "accepted",
    detail: `${spec.name} Tauri WebDriver report supplied with ${report.assertions.length} assertions.`,
    generatedAt: report.generatedAt,
    assertions: report.assertions.length,
    outputHash: report.exportArtifacts.outputHash,
  };
}

function readJsonEvidence(path, id, relativePath) {
  try {
    const report = JSON.parse(readFileSync(path, "utf8"));
    return { id, path: relativePath, status: "present", report };
  } catch (error) {
    return invalid(id, relativePath, `${relativePath} is not valid JSON: ${error.message}`);
  }
}

function missingEvidence(id, path, detail) {
  return {
    id,
    path,
    status: "missing",
    detail,
  };
}

function invalid(id, path, detail) {
  return {
    id,
    path,
    status: "invalid",
    detail,
  };
}

function requireValue(condition, problems, message) {
  if (!condition) problems.push(message);
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function isSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/i.test(value);
}

function writeTemplates() {
  for (const spec of platformSpecs) {
    const packageTemplatePath = join(templateDir, `${spec.platform}-package-artifacts.template.json`);
    const webdriverTemplatePath = join(templateDir, `${spec.platform}-tauri-webdriver-report.template.json`);
    writeFileSync(
      packageTemplatePath,
      `${JSON.stringify(
        {
          schema: "neditor.platform-package-artifacts.v1",
          platform: spec.platform,
          status: "passed",
          generatedAt: new Date().toISOString(),
          command: "pnpm run build && ./node_modules/.bin/tauri build --bundles all",
          artifacts: [
            {
              kind: spec.artifactKinds[0],
              path: `src-tauri/target/release/bundle/${spec.name.toLowerCase()}/NEditor-placeholder`,
              bytes: 123456,
              sha256: "replace-with-64-character-sha256",
            },
          ],
          notes: "Fill this from the supported platform host after inspecting the generated installer/package files.",
        },
        null,
        2,
      )}\n`,
    );
    writeFileSync(
      webdriverTemplatePath,
      `${JSON.stringify(
        {
          generatedAt: new Date().toISOString(),
          platform: spec.platform,
          status: "passed",
          assertions: [
            { name: "initial native title includes NEditor", status: "passed" },
            { name: "desktop shell renders primary commands", status: "passed" },
            { name: "desktop WebDriver saves and reopens real Markdown file through dialog-free smoke path", status: "passed" },
            { name: "desktop WebDriver writes HTML export through dialog-free smoke path", status: "passed" },
            { name: "desktop preferences persist across WebDriver restart", status: "passed" },
          ],
          fileArtifacts: { savedBytes: 1234 },
          exportArtifacts: {
            outputBytes: 12345,
            outputHash: "replace-with-64-character-sha256",
          },
          notes: "Use the real .tmp/desktop-webdriver/report.json from the supported platform host instead of this template.",
        },
        null,
        2,
      )}\n`,
    );
  }
}

function writeReport(report) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
