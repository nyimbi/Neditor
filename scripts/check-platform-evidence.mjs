import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const currentSourceCommit = gitCommit();
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
const requiredWebdriverAssertions = [
  "initial native title includes NEditor",
  "desktop shell renders primary commands",
  "native WebDriver switches modes and opens command palette",
  "desktop WebDriver inserts calc and chart templates from packaged templates panel",
  "desktop WebDriver edits document structure in outline mode",
  "desktop WebDriver runs full native workflow evidence bundle",
  "native title exposes dirty document state",
  "desktop WebDriver saves and reopens real Markdown file through dialog-free smoke path",
  "desktop WebDriver renames, duplicates, and exposes reveal affordance for real Markdown files",
  "desktop export readiness returns manifest progress evidence",
  "desktop WebDriver writes HTML export through dialog-free smoke path",
  "desktop preferences apply in packaged WebDriver session",
];

mkdirSync(templateDir, { recursive: true });
writeTemplates();

const platforms = platformSpecs.map((spec) => evaluatePlatform(spec));
const missingItems = platforms.flatMap((platform) => platform.missingEvidence);
const invalidItems = platforms.flatMap((platform) => platform.invalidEvidence);
const staleItems = platforms.flatMap((platform) => platform.staleEvidence);
const status =
  invalidItems.length > 0
    ? "failed"
    : missingItems.length > 0 || staleItems.length > 0
      ? "pending-external-evidence"
      : "complete";

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
    staleEvidence: staleItems.length,
  },
  platforms,
  missingEvidence: missingItems,
  invalidEvidence: invalidItems,
  staleEvidence: staleItems,
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
  const staleEvidence = checks.filter((check) => check.status === "stale");
  const missingEvidence = checks.filter((check) => check.status === "missing");
  const status =
    invalidEvidence.length > 0
      ? "failed"
      : missingEvidence.length > 0 || staleEvidence.length > 0
        ? "pending-external-evidence"
        : "complete";

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
    staleEvidence: staleEvidence.map((check) => ({
      id: check.id,
      platform: spec.platform,
      path: check.path,
      detail: check.detail,
      sourceCommit: check.sourceCommit,
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
  if (report.sourceCommit && report.sourceCommit !== currentSourceCommit) {
    return stale(
      "package-artifacts",
      spec.packagePath,
      `${spec.name} package artifact evidence was collected for ${report.sourceCommit}; current commit is ${currentSourceCommit}.`,
      report.sourceCommit,
    );
  }
  const problems = [];
  requireValue(report.schema === "neditor.platform-package-artifacts.v1", problems, "schema must be neditor.platform-package-artifacts.v1");
  requireValue(report.platform === spec.platform, problems, `platform must be ${spec.platform}`);
  requireValue(report.status === "passed", problems, "status must be passed");
  requireValue(isIsoDate(report.generatedAt), problems, "generatedAt must be an ISO timestamp");
  requireValue(report.appVersion === packageJson.version, problems, `appVersion must match package.json version ${packageJson.version}`);
  requireValue(report.sourceCommit === currentSourceCommit, problems, `sourceCommit must match current git commit ${currentSourceCommit}`);
  requireValue(report.sourceTreeClean === true, problems, "sourceTreeClean must be true");
  requireValue(identifiesTauriPackageBuild(report.command), problems, "command must identify the Tauri package build");
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
    appVersion: report.appVersion,
    sourceCommit: report.sourceCommit,
    sourceTreeClean: report.sourceTreeClean,
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
  if (report.sourceCommit && report.sourceCommit !== currentSourceCommit) {
    return stale(
      "tauri-webdriver",
      spec.webdriverPath,
      `${spec.name} Tauri WebDriver evidence was collected for ${report.sourceCommit}; current commit is ${currentSourceCommit}.`,
      report.sourceCommit,
    );
  }
  if (report.schema !== "neditor.tauri-webdriver-report.v2") {
    return stale(
      "tauri-webdriver",
      spec.webdriverPath,
      `${spec.name} Tauri WebDriver evidence must be regenerated with the v2 native workflow bundle contract.`,
      report.sourceCommit || "",
    );
  }
  const problems = [];
  requireValue(report.platform === spec.platform, problems, `platform must be ${spec.platform}`);
  requireValue(report.status === "passed", problems, "status must be passed");
  requireValue(isIsoDate(report.generatedAt), problems, "generatedAt must be an ISO timestamp");
  requireValue(report.appVersion === packageJson.version, problems, `appVersion must match package.json version ${packageJson.version}`);
  requireValue(report.sourceCommit === currentSourceCommit, problems, `sourceCommit must match current git commit ${currentSourceCommit}`);
  requireValue(report.sourceTreeClean === true, problems, "sourceTreeClean must be true");
  const workflowPlan = Array.isArray(report.workflowPlan) ? report.workflowPlan : [];
  const assertions = Array.isArray(report.assertions) ? report.assertions : [];
  requireValue(assertions.length >= requiredWebdriverAssertions.length, problems, "assertions must include the full desktop workflow proof");
  const assertionByName = new Map(assertions.map((assertion) => [assertion.name, assertion]));
  for (const requiredAssertion of requiredWebdriverAssertions) {
    requireValue(workflowPlan.includes(requiredAssertion), problems, `workflowPlan missing assertion: ${requiredAssertion}`);
    const assertion = assertionByName.get(requiredAssertion);
    requireValue(Boolean(assertion), problems, `missing assertion: ${requiredAssertion}`);
    requireValue(assertion?.status === "passed", problems, `assertion did not pass: ${requiredAssertion}`);
  }
  const outlineEvidence = report.outlineArtifacts?.sourceEvidence || {};
  requireValue(outlineEvidence.executiveSummary === true, problems, "outlineArtifacts.sourceEvidence.executiveSummary must be true");
  requireValue(outlineEvidence.newSubsection === true, problems, "outlineArtifacts.sourceEvidence.newSubsection must be true");
  requireValue(outlineEvidence.dataTablePreserved === true, problems, "outlineArtifacts.sourceEvidence.dataTablePreserved must be true");
  requireValue(outlineEvidence.sourceGovernancePreserved === true, problems, "outlineArtifacts.sourceEvidence.sourceGovernancePreserved must be true");
  requireValue(Number(report.fileArtifacts?.bytes) > 0, problems, "fileArtifacts.bytes must be present");
  requireValue(Number(report.fileArtifacts?.renamedBytes) > 0, problems, "fileArtifacts.renamedBytes must be present");
  requireValue(Number(report.fileArtifacts?.duplicateBytes) > 0, problems, "fileArtifacts.duplicateBytes must be present");
  requireValue(report.fileArtifacts?.revealLabel === "Reveal", problems, "fileArtifacts.revealLabel must prove reveal control is present");
  requireValue(report.fileArtifacts?.revealControlEnabled === true, problems, "fileArtifacts.revealControlEnabled must prove reveal control is enabled");
  requireValue(Number(report.exportArtifacts?.outputBytes) > 1000, problems, "exportArtifacts.outputBytes must be > 1000");
  requireValue(Number(report.exportArtifacts?.manifestBytes) > 100, problems, "exportArtifacts.manifestBytes must be > 100");
  requireValue(report.exportArtifacts?.target === "html", problems, "exportArtifacts.target must be html");
  requireValue(isSha256(report.exportArtifacts?.outputHash), problems, "exportArtifacts.outputHash must be a sha256");
  requireValue(report.transformTemplateArtifacts?.sourceHasCalcTemplate === true, problems, "transformTemplateArtifacts.sourceHasCalcTemplate must be true");
  requireValue(report.transformTemplateArtifacts?.sourceHasChartTemplate === true, problems, "transformTemplateArtifacts.sourceHasChartTemplate must be true");
  requireValue(report.transformTemplateArtifacts?.previewHasDoseResult === true, problems, "transformTemplateArtifacts.previewHasDoseResult must be true");
  requireValue(
    String(report.transformTemplateArtifacts?.doseFillFields || "").includes("weight_kg") &&
      String(report.transformTemplateArtifacts?.doseFillFields || "").includes("tablet_strength_mg"),
    problems,
    "transformTemplateArtifacts.doseFillFields must include dose placeholders",
  );
  requireValue(
    Array.isArray(report.exportArtifacts?.progressEvidence) &&
      report.exportArtifacts.progressEvidence.some((step) => String(step).includes("Render") && String(step).includes("complete")),
    problems,
    "exportArtifacts.progressEvidence must include a completed render step",
  );
  const nativeWorkflow = report.nativeWorkflowArtifacts || {};
  requireValue(nativeWorkflow.status === "passed", problems, "nativeWorkflowArtifacts.status must be passed");
  requireValue(Number(nativeWorkflow.assertionCount) >= 100, problems, "nativeWorkflowArtifacts.assertionCount must prove the full native workflow bundle");
  requireValue(
    Number(nativeWorkflow.passedAssertionCount) === Number(nativeWorkflow.assertionCount),
    problems,
    "nativeWorkflowArtifacts.passedAssertionCount must equal assertionCount",
  );
  const requiredNativeAssertions = Array.isArray(nativeWorkflow.requiredAssertions) ? nativeWorkflow.requiredAssertions : [];
  for (const requiredAssertion of [
    "native workflow restored project-local snapshot",
    "native workflow committed native Git document",
    "native workflow restored native Git revision",
    "native workflow continued markdown list in editor",
    "native workflow inserted paired bracket in editor",
    "native workflow applied Emacs keybinding mode",
    "native workflow edited with Vim operator motions",
    "native workflow loaded source table from native writing tools menu",
    "native workflow jumped preview table artifact to source",
    "native workflow reported missing external engine path diagnostic",
    "native workflow blocked untrusted external transform probe",
    "native workflow wrote html export artifact",
  ]) {
    requireValue(requiredNativeAssertions.includes(requiredAssertion), problems, `nativeWorkflowArtifacts.requiredAssertions missing ${requiredAssertion}`);
  }
  requireValue(nativeWorkflow.hasEditorErgonomicsEvidence === true, problems, "nativeWorkflowArtifacts.hasEditorErgonomicsEvidence must be true");
  requireValue(nativeWorkflow.hasKeybindingEvidence === true, problems, "nativeWorkflowArtifacts.hasKeybindingEvidence must be true");
  requireValue(nativeWorkflow.hasTableEvidence === true, problems, "nativeWorkflowArtifacts.hasTableEvidence must be true");
  requireValue(nativeWorkflow.hasGitVersioningEvidence === true, problems, "nativeWorkflowArtifacts.hasGitVersioningEvidence must be true");
  requireValue(nativeWorkflow.hasTransformSafetyEvidence === true, problems, "nativeWorkflowArtifacts.hasTransformSafetyEvidence must be true");
  requireValue(Number(nativeWorkflow.modeCount) >= 5, problems, "nativeWorkflowArtifacts.modeCount must cover multiple desktop modes");

  if (problems.length > 0) {
    return invalid("tauri-webdriver", spec.webdriverPath, `${spec.name} Tauri WebDriver evidence is invalid: ${problems.join("; ")}`);
  }

  return {
    id: "tauri-webdriver",
    path: spec.webdriverPath,
    status: "accepted",
    detail: `${spec.name} Tauri WebDriver report supplied with ${report.assertions.length} assertions.`,
    generatedAt: report.generatedAt,
    appVersion: report.appVersion,
    sourceCommit: report.sourceCommit,
    sourceTreeClean: report.sourceTreeClean,
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

function identifiesTauriPackageBuild(command) {
  const normalized = String(command || "").replaceAll("\\", "/");
  return normalized.includes("tauri build") || normalized.includes("scripts/run-tauri-build.mjs");
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

function stale(id, path, detail, sourceCommit) {
  return {
    id,
    path,
    status: "stale",
    detail,
    sourceCommit,
  };
}

function requireValue(condition, problems, message) {
  if (!condition) problems.push(message);
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function isSha256(value) {
  return typeof value === "string" && /^(?:sha256:)?[a-f0-9]{64}$/i.test(value);
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
          appVersion: packageJson.version,
          sourceCommit: currentSourceCommit || "replace-with-current-git-commit",
          sourceTreeClean: true,
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
          schema: "neditor.tauri-webdriver-report.v2",
          generatedAt: new Date().toISOString(),
          platform: spec.platform,
          status: "passed",
          appVersion: packageJson.version,
          sourceCommit: currentSourceCommit || "replace-with-current-git-commit",
          sourceTreeClean: true,
          workflowPlan: requiredWebdriverAssertions,
          assertions: requiredWebdriverAssertions.map((name) => ({ name, status: "passed" })),
          outlineArtifacts: {
            sourceEvidence: {
              executiveSummary: true,
              newSubsection: true,
              dataTablePreserved: true,
              sourceGovernancePreserved: true,
            },
          },
          transformTemplateArtifacts: {
            sourceHasCalcTemplate: true,
            sourceHasChartTemplate: true,
            previewHasDoseResult: true,
            doseFillFields: "weight_kg tablet_strength_mg",
          },
          nativeWorkflowArtifacts: {
            status: "passed",
            phase: "final",
            assertionCount: 100,
            passedAssertionCount: 100,
            requiredAssertions: [
              "native workflow restored project-local snapshot",
              "native workflow committed native Git document",
              "native workflow restored native Git revision",
              "native workflow continued markdown list in editor",
              "native workflow inserted paired bracket in editor",
              "native workflow applied Emacs keybinding mode",
              "native workflow edited with Vim operator motions",
              "native workflow loaded source table from native writing tools menu",
              "native workflow jumped preview table artifact to source",
              "native workflow reported missing external engine path diagnostic",
              "native workflow blocked untrusted external transform probe",
              "native workflow wrote html export artifact",
            ],
            modeCount: 5,
            hasEditorErgonomicsEvidence: true,
            hasKeybindingEvidence: true,
            hasTableEvidence: true,
            hasGitVersioningEvidence: true,
            hasTransformSafetyEvidence: true,
          },
          fileArtifacts: {
            bytes: 1234,
            renamedBytes: 1234,
            duplicateBytes: 1234,
            revealLabel: "Reveal",
            revealControlEnabled: true,
          },
          exportArtifacts: {
            outputBytes: 12345,
            manifestBytes: 1234,
            target: "html",
            outputHash: "replace-with-64-character-sha256",
            progressEvidence: ["Render complete"],
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

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0) return "";
  return result.stdout.trim();
}
