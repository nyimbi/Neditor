import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = join(root, ".tmp", "release-readiness", "report.json");
const failures = [];

const requiredReports = [
  requiredReport("browser-environment", ".tmp/e2e-environment/report.json", ["passed"]),
  requiredReport("browser-workflows", ".tmp/e2e-browser/report.json", [], browserWorkflowAccepted),
  requiredReport("static-accessibility", ".tmp/accessibility/report.json", ["pass", "passed"]),
  requiredReport("runtime-accessibility", ".tmp/accessibility/runtime-report.json", [], runtimeAccessibilityAccepted),
  requiredReport("manual-accessibility-contract", ".tmp/accessibility/manual-review-summary.json", [
    "pending-human-review",
    "human-reviewed",
  ]),
  requiredReport("platform-package-config", ".tmp/desktop-bundle/platform-package-config-report.json", ["passed"]),
  requiredReport("external-platform-evidence", ".tmp/platform-evidence/report.json", [], platformEvidenceAccepted),
  requiredReport("release-signing-evidence", ".tmp/release-signing/report.json", [], releaseSigningAccepted),
  requiredReport("ai-provider-evidence", ".tmp/ai-provider-evidence/report.json", [], aiProviderEvidenceAccepted),
  requiredReport("ai-runtime-evidence", ".tmp/ai-runtime-evidence/report.json", [], aiRuntimeEvidenceAccepted),
  requiredReport("security-review-evidence", ".tmp/security-review/report.json", [], securityReviewEvidenceAccepted),
  requiredReport("release-ci-workflow", ".tmp/release-ci/workflow-report.json", [], releaseCiWorkflowAccepted),
  requiredReport("release-evidence-kit", ".tmp/release-evidence-kit/report.json", [], releaseEvidenceKitAccepted),
  requiredReport("desktop-command-smoke", ".tmp/desktop-smoke/native-command-report.json", [], desktopCommandPassed),
  requiredReport("rendered-export-audit", ".tmp/rendered-export-audit/rendered-export-audit-report.json", [], renderedExportAuditAccepted),
  requiredReport("rendered-export-visual-summary", ".tmp/rendered-export-audit/visual-review-summary.json", [], visualSummaryPassed),
  requiredReport("google-docs-import-evidence", ".tmp/google-docs-import/report.json", [], googleDocsImportAccepted),
  requiredReport("external-engine-probe", ".tmp/external-engines/probe-report.json", [], externalEngineProbePassed),
  requiredReport("performance-audit", ".tmp/performance-audit/report.json", [], performanceAuditAccepted),
  requiredReport("performance-profile-evidence", ".tmp/performance-profile/report.json", [], performanceProfileEvidenceAccepted),
];

if (process.platform === "darwin") {
  requiredReports.push(requiredReport("macos-app-bundle", ".tmp/desktop-bundle/macos-app-report.json", [], macosAppBundleAccepted));
  requiredReports.push(requiredReport("macos-dmg-classification", ".tmp/desktop-bundle/macos-dmg-report.json", [], macosDmgAccepted));
  requiredReports.push(requiredReport("macos-native-launch", ".tmp/desktop-smoke/launch-report.json", [], macosLaunchPassed));
  requiredReports.push(requiredReport("macos-webdriver-fallback", ".tmp/desktop-webdriver/report.json", [], webdriverOrFallbackPassed));
}

const checks = requiredReports.map((report) => evaluateReport(report));
const evidenceGaps = collectEvidenceGaps(checks);
const status = failures.length > 0 ? "failed" : evidenceGaps.length > 0 ? "current-host-ready-with-external-gaps" : "ready";

writeReport({
  generatedAt: new Date().toISOString(),
  platform: process.platform,
  arch: process.arch,
  status,
  summary: {
    requiredChecks: checks.length,
    accepted: checks.filter((check) => check.accepted).length,
    failed: failures.length,
    evidenceGaps: evidenceGaps.length,
  },
  checks,
  evidenceGaps,
  failures,
});

if (failures.length > 0) {
  console.error("Release readiness aggregation failed:");
  for (const failure of failures) console.error(`- ${failure}`);
  console.error(`Wrote ${relative(reportPath)}.`);
  process.exit(1);
}

console.log(`Release readiness is ${status}; wrote ${relative(reportPath)}.`);

function requiredReport(id, relativePath, acceptedStatuses = [], customAccept = null) {
  return {
    id,
    relativePath,
    acceptedStatuses,
    customAccept,
  };
}

function evaluateReport(spec) {
  const absolutePath = join(root, spec.relativePath);
  if (!existsSync(absolutePath)) {
    const failure = `${spec.id} report is missing at ${spec.relativePath}`;
    failures.push(failure);
    return {
      id: spec.id,
      path: spec.relativePath,
      status: "missing",
      accepted: false,
      generatedAt: null,
      detail: failure,
    };
  }

  let report;
  try {
    report = JSON.parse(readFileSync(absolutePath, "utf8"));
  } catch (error) {
    const failure = `${spec.id} report is not valid JSON: ${error}`;
    failures.push(failure);
    return {
      id: spec.id,
      path: spec.relativePath,
      status: "invalid-json",
      accepted: false,
      generatedAt: null,
      detail: String(error),
    };
  }

  const custom = spec.customAccept?.(report);
  const statusValue = String(report.status ?? custom?.status ?? "present");
  const accepted =
    custom?.accepted ??
    (spec.acceptedStatuses.length === 0 ? true : spec.acceptedStatuses.includes(statusValue));
  if (!accepted) {
    failures.push(`${spec.id} report status ${statusValue} is not accepted`);
  }
  return {
    id: spec.id,
    path: spec.relativePath,
    status: custom?.status ?? statusValue,
    accepted,
    generatedAt: report.generatedAt || null,
    detail: custom?.detail || "",
  };
}

function collectEvidenceGaps(checks) {
  const reports = Object.fromEntries(checks.map((check) => [check.id, readOptionalJson(check.path)]));
  const gaps = [];

  const releaseSigning = reports["release-signing-evidence"];
  const missingSigningPlatforms = missingReleaseSigningEvidence(releaseSigning);
  const platformConfig = reports["platform-package-config"];
  const signing = platformConfig?.signing || {};
  if (missingSigningPlatforms.length > 0 || signing.status === "unsigned-local-builds") {
    gaps.push({
      id: "release-signing-and-notarization",
      status: "pending-release-credentials",
      evidence: releaseSigning ? ".tmp/release-signing/report.json" : ".tmp/desktop-bundle/platform-package-config-report.json",
      detail:
        missingSigningPlatforms.length > 0
          ? `Credentialed release signing/notarization evidence is missing for: ${missingSigningPlatforms.join(", ")}.`
          : "Local artifacts are intentionally unsigned; distribution signing, notarization, and installer attestation require release credentials.",
    });
  }

  const platformEvidence = reports["external-platform-evidence"];
  const missingWebdriverPlatforms = missingPlatformEvidence(platformEvidence, "webdriver");
  if (missingWebdriverPlatforms.length > 0) {
    gaps.push({
      id: "windows-linux-tauri-webdriver-execution",
      status: "pending-supported-hosts",
      evidence: platformEvidence ? ".tmp/platform-evidence/report.json" : ".tmp/desktop-webdriver/report.json",
      detail: `Official Tauri WebDriver execution still needs supported-host reports for: ${missingWebdriverPlatforms.join(", ")}.`,
    });
  }

  const missingPackagePlatforms = missingPlatformEvidence(platformEvidence, "packageArtifacts");
  if (process.platform !== "win32" && missingPackagePlatforms.includes("win32")) {
    gaps.push({
      id: "windows-package-artifact-proof",
      status: "pending-windows-host",
      evidence: platformEvidence ? ".tmp/platform-evidence/report.json" : ".tmp/desktop-bundle/platform-package-config-report.json",
      detail: "Configuration proves Windows bundle targets and icons, but installer artifact execution needs a Windows host.",
    });
  }
  if (process.platform !== "linux" && missingPackagePlatforms.includes("linux")) {
    gaps.push({
      id: "linux-package-artifact-proof",
      status: "pending-linux-host",
      evidence: platformEvidence ? ".tmp/platform-evidence/report.json" : ".tmp/desktop-bundle/platform-package-config-report.json",
      detail: "Configuration proves Linux bundle targets, but AppImage/deb/rpm artifact execution needs a Linux host.",
    });
  }

  const renderedSummary = reports["rendered-export-visual-summary"];
  const googleDocsImport = reports["google-docs-import-evidence"];
  if (googleDocsImport?.importEvidence?.status !== "accepted") {
    gaps.push({
      id: "google-docs-live-import-readback",
      status: googleDocsImport?.importEvidence?.status || "pending-google-drive-authorization",
      evidence: ".tmp/google-docs-import/report.json",
      detail: "Local Google Docs package proof is present, but live Google Docs import/readback evidence needs an authorized Drive session.",
    });
  }

  const aiProvider = reports["ai-provider-evidence"];
  if (Number(aiProvider?.summary?.acceptedEvidence || 0) < 1) {
    gaps.push({
      id: "ai-provider-live-endpoint-proof",
      status: aiProvider?.status || "pending-live-provider-evidence",
      evidence: ".tmp/ai-provider-evidence/report.json",
      detail: "Agentic provider execution is implemented, but a live approved-provider endpoint response needs credentialed evidence without stored secrets.",
    });
  }

  const aiRuntime = reports["ai-runtime-evidence"];
  if (Number(aiRuntime?.summary?.acceptedEvidence || 0) < 1) {
    gaps.push({
      id: "ai-runtime-real-device-proof",
      status: aiRuntime?.status || "pending-real-runtime-evidence",
      evidence: ".tmp/ai-runtime-evidence/report.json",
      detail: "Docs Live runtime readiness is implemented, but real microphone permission and clipboard read/write proof needs a real browser or Tauri WebView device session.",
    });
  }

  const performanceProfile = reports["performance-profile-evidence"];
  if (Number(performanceProfile?.summary?.acceptedEvidence || 0) < 1) {
    gaps.push({
      id: "release-device-native-performance-profile",
      status: performanceProfile?.status || "pending-release-device-profile",
      evidence: ".tmp/performance-profile/report.json",
      detail: "Bounded local performance checks pass, but a sustained release-device native profile with profiler artifact hashes is still pending.",
    });
  }

  const securityReview = reports["security-review-evidence"];
  if (Number(securityReview?.summary?.acceptedEvidence || 0) < 1) {
    gaps.push({
      id: "independent-security-review-signoff",
      status: securityReview?.status || "pending-independent-security-review",
      evidence: ".tmp/security-review/report.json",
      detail: "Security controls are implemented locally, but independent security review sign-off with report hashes is still pending.",
    });
  }

  if (renderedSummary?.humanSignoff?.status !== "human-reviewed") {
    gaps.push({
      id: "rendered-export-native-viewer-human-signoff",
      status: renderedSummary?.humanSignoff?.status || "pending-human-review",
      evidence: ".tmp/rendered-export-audit/visual-review-summary.json",
      detail: "Automated visual review is present, but completed native-viewer human sign-off has not been supplied.",
    });
  }

  const accessibility = reports["manual-accessibility-contract"];
  if (accessibility?.humanSignoff?.status !== "human-reviewed") {
    gaps.push({
      id: "accessibility-assistive-technology-human-signoff",
      status: accessibility?.humanSignoff?.status || "pending-human-review",
      evidence: ".tmp/accessibility/manual-review-summary.json",
      detail: "Static and runtime accessibility checks pass, but real screen-reader/native assistive-technology sign-off is pending.",
    });
  }

  const externalEngines = reports["external-engine-probe"];
  const missingEngines = (externalEngines?.engines || [])
    .filter((engine) => engine.status === "missing" && engine.externalEvidence?.status !== "accepted")
    .map((engine) => engine.name);
  if (missingEngines.length > 0) {
    gaps.push({
      id: "optional-external-engines",
      status: "partially-installed",
      evidence: ".tmp/external-engines/probe-report.json",
      detail: `Missing optional engines without accepted external evidence: ${missingEngines.join(", ")}.`,
    });
  }

  return gaps;
}

function macosAppBundleAccepted(report) {
  const issues = [];
  const executablePath = report.executable?.path;
  const iconPath = report.icon?.path;
  const plist = report.plist || {};

  if (!validIsoDate(report.generatedAt)) issues.push("missing-generatedAt");
  if (report.appBundle !== "src-tauri/target/release/bundle/macos/NEditor.app") issues.push("unexpected-app-bundle");
  if (!artifactMatchesReport(executablePath, report.executable?.size, true)) issues.push("invalid-executable-artifact");
  if (!artifactMatchesReport(iconPath, report.icon?.size, false)) issues.push("invalid-icon-artifact");
  if (plist.CFBundleDisplayName !== "NEditor") issues.push("invalid-display-name");
  if (plist.CFBundleExecutable !== "neditor") issues.push("invalid-executable-name");
  if (plist.CFBundleIdentifier !== "com.neditor.desktop") issues.push("invalid-bundle-identifier");
  if (plist.CFBundleShortVersionString !== plist.CFBundleVersion) issues.push("version-mismatch");
  if (plist.CFBundlePackageType !== "APPL") issues.push("invalid-package-type");
  if (plist.CFBundleIconFile !== "icon.icns") issues.push("invalid-icon-reference");
  if (plist.NSHighResolutionCapable !== true) issues.push("missing-high-resolution-flag");

  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? "passed" : "incomplete",
    detail:
      issues.length === 0
        ? `app=${report.appBundle} executableBytes=${report.executable?.size} iconBytes=${report.icon?.size}`
        : issues.join(","),
  };
}

function macosDmgAccepted(report) {
  const issues = [];
  const appReport = readOptionalJson(".tmp/desktop-bundle/macos-app-report.json");
  const acceptedStatuses = new Set(["passed", "created", "created-manual-probe-dmg", "classified-host-limitation"]);

  if (!validIsoDate(report.generatedAt)) issues.push("missing-generatedAt");
  if (!acceptedStatuses.has(report.status) && !acceptedStatuses.has(report.result)) {
    issues.push(`status=${report.status || report.result || "missing"}`);
  }
  if (appReport?.generatedAt && Date.parse(report.generatedAt || "") < Date.parse(appReport.generatedAt)) {
    issues.push("older-than-app-bundle-report");
  }

  if (report.status === "passed") {
    if (!artifactMatchesReport(report.dmg?.path, report.dmg?.size, false)) issues.push("invalid-dmg-artifact");
  } else if (report.status === "classified-host-limitation") {
    if (report.result !== "hdiutil-sandbox-device-not-configured") issues.push("unexpected-host-limitation-result");
    if (report.classification?.appBundleStillBuilt !== true) issues.push("missing-app-bundle-fallback-proof");
    if (!String(report.classification?.cause || "").includes("hdiutil")) issues.push("missing-hdiutil-cause");
    const stderr = Array.isArray(report.manualProbe?.stderrTail) ? report.manualProbe.stderrTail.join("\n") : "";
    if (!stderr.includes("Device not configured")) issues.push("missing-manual-probe-classification");
  }

  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? report.status || report.result : "incomplete",
    detail:
      issues.length === 0
        ? report.status === "classified-host-limitation"
          ? "hdiutil sandbox limitation classified with app bundle fallback proof"
          : `dmg=${report.dmg?.path || report.result}`
        : issues.join(","),
  };
}

function browserWorkflowAccepted(report) {
  const issues = [];
  const command = Array.isArray(report.command) ? report.command.map(String) : [];
  const summary = report.summary || {};
  const workflowEvidence = report.workflowEvidence || {};

  if (report.schema !== "neditor.e2e-browser-workflow.v1") issues.push("missing-schema");
  if (report.scope !== "full-suite") issues.push(`scope=${report.scope || "missing"}`);
  if (report.status !== "passed") issues.push(`status=${report.status || "missing"}`);
  if (!validIsoDate(report.generatedAt)) issues.push("missing-generatedAt");
  if (!command.some((part) => part.includes("playwright")) || !command.includes("test")) issues.push("missing-playwright-command");
  if (Number(summary.tests || 0) < 1) issues.push("missing-test-count");
  if (Number(summary.passed || 0) < Number(summary.tests || 0)) issues.push("incomplete-pass-count");
  if (Number(summary.failed || 0) > 0 || Number(summary.timedOut || 0) > 0) issues.push("failed-or-timed-out-tests");
  if (workflowEvidence.docsLiveDraft !== true) issues.push("missing-docs-live-workflow-proof");
  if (!freshForSources(report.generatedAt, ["scripts/run-e2e.mjs", "e2e/app-workflows.spec.ts", "playwright.config.ts"])) {
    issues.push("stale-for-browser-workflow-sources");
  }

  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? "passed" : "incomplete",
    detail:
      issues.length === 0
        ? `tests=${summary.tests} passed=${summary.passed} docsLiveDraft=${workflowEvidence.docsLiveDraft}`
        : issues.join(","),
  };
}

function desktopCommandPassed(report) {
  const issues = [];
  if (!artifactMatchesReport(report.binary, report.binarySize, true)) issues.push("invalid-desktop-binary-artifact");
  if (!reportFileFreshForArtifact(".tmp/desktop-smoke/native-command-report.json", report.binary)) {
    issues.push("native-command-report-stale-for-binary");
  }
  if (report.nativeCommandWorkflow?.status !== 0) issues.push("native-command-workflow-failed");
  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? "passed" : "failed",
    detail:
      issues.length === 0
        ? `durationMs=${report.nativeCommandWorkflow.durationMs} binaryBytes=${report.binarySize}`
        : issues.join(","),
  };
}

function runtimeAccessibilityAccepted(report) {
  const issues = [];
  const expectedCount = Array.isArray(report.expectedWorkflows) ? report.expectedWorkflows.length : 0;
  const linkedReport = readOptionalJson(report.e2eReport || "");

  if (report.status !== "passed") issues.push(`status=${report.status || "missing"}`);
  if (expectedCount < 6) issues.push("missing-expected-workflows");
  if (Array.isArray(report.issues) && report.issues.length > 0) issues.push(`issues=${report.issues.length}`);
  if (!focusedPlaywrightReportAccepted(linkedReport, expectedCount)) issues.push("invalid-focused-e2e-report");

  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? "passed" : "incomplete",
    detail: issues.length === 0 ? `workflows=${expectedCount} focusedReport=${report.e2eReport}` : issues.join(","),
  };
}

function performanceAuditAccepted(report) {
  const issues = [];
  const resultById = new Map((Array.isArray(report.results) ? report.results : []).map((result) => [result?.id, result]));
  const rust = resultById.get("rust-performance-suite");
  const browser = resultById.get("browser-large-document-workflow");
  const linkedReport = readOptionalJson(browser?.evidenceReport || "");

  if (report.status !== "pass" && report.status !== "passed") issues.push(`status=${report.status || "missing"}`);
  if (Number(report.summary?.checks || 0) < 2) issues.push("missing-check-count");
  if (Number(report.summary?.failed || 0) > 0) issues.push("failed-checks");
  if (rust?.status !== "pass") issues.push("missing-rust-performance-suite");
  if (browser?.status !== "pass") issues.push("missing-browser-large-document-workflow");
  if (!focusedPlaywrightReportAccepted(linkedReport, 1)) issues.push("invalid-large-document-e2e-report");

  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? "passed" : "incomplete",
    detail:
      issues.length === 0
        ? `checks=${report.summary?.checks} largeDocumentReport=${browser?.evidenceReport}`
        : issues.join(","),
  };
}

function performanceProfileEvidenceAccepted(report) {
  const invalid = Number(report.summary?.invalidEvidence || 0);
  return {
    accepted: invalid === 0,
    status: report.status || "unknown",
    detail:
      invalid === 0
        ? `performanceProfile=${report.status || "unknown"} accepted=${Number(report.summary?.acceptedEvidence || 0)}`
        : `invalid performance profile evidence count=${invalid}`,
  };
}

function focusedPlaywrightReportAccepted(report, minimumTests) {
  return (
    report?.schema === "neditor.e2e-browser-workflow.v1" &&
    report.scope === "focused" &&
    report.status === "passed" &&
    Number(report.summary?.tests || 0) >= minimumTests &&
    Number(report.summary?.passed || 0) >= Number(report.summary?.tests || 0) &&
    Number(report.summary?.failed || 0) === 0 &&
    Number(report.summary?.timedOut || 0) === 0
  );
}

function artifactMatchesReport(relativePath, expectedSize, requireExecutable) {
  if (typeof relativePath !== "string" || !relativePath) return false;
  const absolutePath = join(root, relativePath);
  const stat = statFile(absolutePath);
  if (!stat) return false;
  if (!stat.isFile()) return false;
  if (Number(expectedSize || 0) < 1 || stat.size !== Number(expectedSize)) return false;
  if (requireExecutable && (stat.mode & 0o111) === 0) return false;
  return true;
}

function artifactExists(relativePath, requireExecutable) {
  if (typeof relativePath !== "string" || !relativePath) return false;
  const stat = statFile(join(root, relativePath));
  if (!stat?.isFile()) return false;
  return !requireExecutable || (stat.mode & 0o111) !== 0;
}

function reportFileFreshForArtifact(reportRelativePath, artifactRelativePath) {
  if (typeof reportRelativePath !== "string" || typeof artifactRelativePath !== "string") return false;
  const reportStat = statFile(join(root, reportRelativePath));
  const artifactStat = statFile(join(root, artifactRelativePath));
  if (!reportStat?.isFile() || !artifactStat?.isFile()) return false;
  return reportStat.mtimeMs + 1000 >= artifactStat.mtimeMs;
}

function statFile(path) {
  if (!existsSync(path)) return null;
  try {
    return statSync(path);
  } catch {
    return null;
  }
}

function visualSummaryPassed(report) {
  const automated = report.automatedVisualReview?.status === "automated-reviewed";
  return {
    accepted: automated,
    status: automated ? "automated-reviewed" : report.automatedVisualReview?.status || "missing-automated-review",
    detail: report.humanSignoff?.status ? `humanSignoff=${report.humanSignoff.status}` : "",
  };
}

function externalEngineProbePassed(report) {
  const incompatible = (report.engines || []).filter((engine) => engine.status === "incompatible");
  const invalidEvidence = Number(report.summary?.invalidExternalEvidence || 0);
  return {
    accepted: incompatible.length === 0 && invalidEvidence === 0,
    status: incompatible.length === 0 && invalidEvidence === 0 ? "passed" : "failed",
    detail:
      incompatible.length === 0 && invalidEvidence === 0
        ? "installed engines smoke-compatible"
        : `incompatible=${incompatible.map((engine) => engine.name).join(", ")} invalidExternalEvidence=${invalidEvidence}`,
  };
}

function platformEvidenceAccepted(report) {
  const invalid = Number(report.summary?.invalidEvidence || 0);
  return {
    accepted: invalid === 0,
    status: report.status || "unknown",
    detail:
      invalid === 0
        ? `externalPlatformEvidence=${report.status || "unknown"}`
        : `invalid external platform evidence count=${invalid}`,
  };
}

function releaseSigningAccepted(report) {
  const invalid = Number(report.summary?.invalidEvidence || 0);
  return {
    accepted: invalid === 0,
    status: report.status || "unknown",
    detail:
      invalid === 0
        ? `releaseSigningEvidence=${report.status || "unknown"}`
        : `invalid release signing evidence count=${invalid}`,
  };
}

function aiProviderEvidenceAccepted(report) {
  const invalid = Number(report.summary?.invalidEvidence || 0);
  return {
    accepted: invalid === 0,
    status: report.status || "unknown",
    detail:
      invalid === 0
        ? `aiProviderEvidence=${report.status || "unknown"} accepted=${Number(report.summary?.acceptedEvidence || 0)}`
        : `invalid AI provider evidence count=${invalid}`,
  };
}

function aiRuntimeEvidenceAccepted(report) {
  const invalid = Number(report.summary?.invalidEvidence || 0);
  return {
    accepted: invalid === 0,
    status: report.status || "unknown",
    detail:
      invalid === 0
        ? `aiRuntimeEvidence=${report.status || "unknown"} accepted=${Number(report.summary?.acceptedEvidence || 0)}`
        : `invalid AI runtime evidence count=${invalid}`,
  };
}

function securityReviewEvidenceAccepted(report) {
  const invalid = Number(report.summary?.invalidEvidence || 0);
  return {
    accepted: invalid === 0,
    status: report.status || "unknown",
    detail:
      invalid === 0
        ? `securityReview=${report.status || "unknown"} accepted=${Number(report.summary?.acceptedEvidence || 0)}`
        : `invalid security review evidence count=${invalid}`,
  };
}

function releaseCiWorkflowAccepted(report) {
  const issues = [];
  if (report.schema !== "neditor.release-ci-workflow-report.v1") issues.push("missing-schema");
  if (report.status !== "passed") issues.push(`status=${report.status || "missing"}`);
  if (!validIsoDate(report.generatedAt)) issues.push("missing-generatedAt");
  if (report.workflowPath !== ".github/workflows/neditor-release-evidence.yml") issues.push("unexpected-workflow-path");
  if (report.packageScript !== "node scripts/check-release-ci-workflow.mjs") issues.push("unexpected-package-script");
  if (Array.isArray(report.issues) && report.issues.length > 0) issues.push(`issues=${report.issues.length}`);
  if (!freshForSources(report.generatedAt, [".github/workflows/neditor-release-evidence.yml", "scripts/check-release-ci-workflow.mjs", "package.json"])) {
    issues.push("stale-for-release-ci-sources");
  }

  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? "passed" : "incomplete",
    detail: issues.length === 0 ? `workflow=${report.workflowPath}` : issues.join(","),
  };
}

function releaseEvidenceKitAccepted(report) {
  const issues = [];
  if (report.schema !== "neditor.release-evidence-kit-report.v1") issues.push("missing-schema");
  if (report.status !== "passed") issues.push(`status=${report.status || "missing"}`);
  if (!validIsoDate(report.generatedAt)) issues.push("missing-generatedAt");
  if (report.sourceCommit !== report.currentSourceCommit) issues.push("source-commit-mismatch");
  if (report.sourceTreeClean !== true) issues.push("source-tree-not-clean");
  if (report.currentSourceTreeClean !== true) issues.push("current-source-tree-not-clean");
  if (report.appVersion !== report.currentAppVersion) issues.push("app-version-mismatch");
  if (report.readinessStatus !== report.currentReadinessStatus) issues.push("readiness-status-mismatch");
  if (Number(report.summary?.missingTemplates || 0) !== 0) issues.push("missing-templates");
  if (Number(report.summary?.staleTemplates || 0) !== 0) issues.push("stale-templates");
  if (Number(report.summary?.copiedTemplates || 0) < 15) issues.push("incomplete-template-set");
  if (Number(report.summary?.runbooks || 0) < 11) issues.push("incomplete-runbook-set");
  if (Number(report.summary?.issues || 0) !== 0) issues.push("reported-issues");

  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? "passed" : "incomplete",
    detail:
      issues.length === 0
        ? `gaps=${report.summary?.gaps} templates=${report.summary?.copiedTemplates} runbooks=${report.summary?.runbooks}`
        : issues.join(","),
  };
}

function renderedExportAuditAccepted(report) {
  const targetNames = new Set((Array.isArray(report.targets) ? report.targets : []).map((target) => target?.target));
  const requiredTargets = ["html", "pdf", "docx", "pptx", "markdown-bundle", "blog", "substack", "latex", "google-docs"];
  const missingTargets = requiredTargets.filter((target) => !targetNames.has(target));
  const reviewCases = Array.isArray(report.reviewCases) ? report.reviewCases : [];
  const reviewCaseBySlug = new Map(reviewCases.map((reviewCase) => [reviewCase?.slug, reviewCase]));
  const missingReviewCases = ["rich-blocks", "option-heavy"].filter((slug) => !reviewCaseBySlug.has(slug));
  const incompleteReviewCases = [];
  for (const slug of ["rich-blocks", "option-heavy"]) {
    const reviewCase = reviewCaseBySlug.get(slug);
    if (!reviewCase) continue;
    const caseTargets = new Set((Array.isArray(reviewCase.targets) ? reviewCase.targets : []).map((target) => target?.target));
    const missingCaseTargets = ["html", "pdf", "docx", "pptx", "markdown-bundle"].filter((target) => !caseTargets.has(target));
    if (missingCaseTargets.length > 0) incompleteReviewCases.push(`${slug}:${missingCaseTargets.join(",")}`);
  }
  const checklistCount = Array.isArray(report.manualChecklist) ? report.manualChecklist.length : 0;
  const accepted = missingTargets.length === 0 && missingReviewCases.length === 0 && incompleteReviewCases.length === 0 && checklistCount >= 7;
  return {
    accepted,
    status: accepted ? "accepted" : "incomplete",
    detail: accepted
      ? `renderedTargets=${requiredTargets.length} reviewCases=${reviewCases.length} checklist=${checklistCount}`
      : `missingTargets=${missingTargets.join(",") || "none"} missingReviewCases=${missingReviewCases.join(",") || "none"} incompleteReviewCases=${incompleteReviewCases.join(";") || "none"} checklist=${checklistCount}`,
  };
}

function googleDocsImportAccepted(report) {
  const invalid = Array.isArray(report.issues) && report.issues.length > 0;
  return {
    accepted: !invalid && report.sourceArtifacts?.status === "accepted",
    status: report.status || "unknown",
    detail: invalid
      ? `googleDocsImportIssues=${report.issues.length}`
      : `googleDocsImport=${report.importEvidence?.status || report.status || "unknown"}`,
  };
}

function missingPlatformEvidence(report, key) {
  if (!report || !Array.isArray(report.platforms)) {
    return ["win32", "linux"];
  }
  return report.platforms
    .filter((platform) => platform?.[key]?.status !== "accepted")
    .map((platform) => platform.platform)
    .filter((platform) => platform === "win32" || platform === "linux");
}

function missingReleaseSigningEvidence(report) {
  if (!report || !Array.isArray(report.platforms)) {
    return ["darwin", "win32", "linux"];
  }
  return report.platforms
    .filter((platform) => platform?.status !== "accepted")
    .map((platform) => platform.platform)
    .filter((platform) => ["darwin", "win32", "linux"].includes(platform));
}

function macosLaunchPassed(report) {
  const issues = [];
  const status = report.launch?.status || report.status;
  if (status !== "survived-until-timeout") issues.push(`status=${status || "missing"}`);
  if (!artifactExists(report.binary, true)) issues.push("missing-launch-binary");
  if (!reportFileFreshForArtifact(".tmp/desktop-smoke/launch-report.json", report.binary)) {
    issues.push("launch-report-stale-for-binary");
  }
  if (report.processAlive !== true && report.launch?.processAlive !== true) issues.push("process-not-alive");
  if (report.nativeWindow?.window?.visible !== true && report.launch?.nativeWindow?.window?.visible !== true) {
    issues.push("missing-native-window-proof");
  }
  return {
    accepted: issues.length === 0,
    status: issues.length === 0 ? "passed" : status || "unknown",
    detail: issues.length === 0 ? "bounded GUI launch survived until timeout with current binary proof" : issues.join(","),
  };
}

function webdriverOrFallbackPassed(report) {
  if (report.status === "passed") {
    return {
      accepted: true,
      status: "passed",
      detail: "WebDriver workflow passed",
    };
  }
  const fallbackIssues = [];
  const proof = report.fallbackProof || {};
  if (report.status !== "skipped") fallbackIssues.push(`status=${report.status || "missing"}`);
  if (proof.status !== "passed") fallbackIssues.push(`fallbackStatus=${proof.status || "missing"}`);
  if (!artifactExists(report.application, true)) fallbackIssues.push("missing-application-binary");
  if (!reportFileFreshForArtifact(".tmp/desktop-webdriver/report.json", report.application)) {
    fallbackIssues.push("webdriver-report-stale-for-binary");
  }
  if (!reportFileFreshForArtifact(proof.reportPath, report.application)) {
    fallbackIssues.push("fallback-smoke-report-stale-for-binary");
  }
  if (!reportFileFreshForArtifact(proof.launchReportPath, report.application)) {
    fallbackIssues.push("fallback-launch-report-stale-for-binary");
  }
  if (proof.freshForBinary !== true) fallbackIssues.push("fallback-did-not-self-report-freshness");
  if (proof.launchStatus !== "survived-until-timeout") fallbackIssues.push("fallback-launch-not-bounded-survival");
  if (proof.processAlive !== true) fallbackIssues.push("fallback-process-not-alive");
  if (Number(proof.passedAssertionCount || 0) < 1 || proof.passedAssertionCount !== proof.assertionCount) {
    fallbackIssues.push("fallback-assertions-not-all-passing");
  }
  const fallbackPassed = fallbackIssues.length === 0;
  return {
    accepted: fallbackPassed,
    status: report.status || "unknown",
    detail: fallbackPassed
      ? "macOS unsupported WebDriver skip has fresh native fallback proof for current binary"
      : fallbackIssues.join(","),
  };
}

function validIsoDate(value) {
  return typeof value === "string" && Number.isFinite(Date.parse(value));
}

function freshForSources(generatedAt, relativePaths) {
  const generatedMs = Date.parse(generatedAt || "");
  if (!Number.isFinite(generatedMs)) return false;
  return relativePaths.every((relativePath) => {
    try {
      return generatedMs >= statSync(join(root, relativePath)).mtimeMs;
    } catch {
      return false;
    }
  });
}

function readOptionalJson(relativePath) {
  const absolutePath = join(root, relativePath);
  if (!existsSync(absolutePath)) return null;
  try {
    return JSON.parse(readFileSync(absolutePath, "utf8"));
  } catch {
    return null;
  }
}

function writeReport(report) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
