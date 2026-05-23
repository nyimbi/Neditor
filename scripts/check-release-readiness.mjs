import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = join(root, ".tmp", "release-readiness", "report.json");
const failures = [];

const requiredReports = [
  requiredReport("browser-environment", ".tmp/e2e-environment/report.json", ["passed"]),
  requiredReport("browser-workflows", ".tmp/e2e-browser/report.json", ["passed"]),
  requiredReport("static-accessibility", ".tmp/accessibility/report.json", ["pass", "passed"]),
  requiredReport("runtime-accessibility", ".tmp/accessibility/runtime-report.json", ["passed"]),
  requiredReport("manual-accessibility-contract", ".tmp/accessibility/manual-review-summary.json", [
    "pending-human-review",
    "human-reviewed",
  ]),
  requiredReport("platform-package-config", ".tmp/desktop-bundle/platform-package-config-report.json", ["passed"]),
  requiredReport("external-platform-evidence", ".tmp/platform-evidence/report.json", [], platformEvidenceAccepted),
  requiredReport("release-signing-evidence", ".tmp/release-signing/report.json", [], releaseSigningAccepted),
  requiredReport("desktop-command-smoke", ".tmp/desktop-smoke/native-command-report.json", [], desktopCommandPassed),
  requiredReport("rendered-export-audit", ".tmp/rendered-export-audit/rendered-export-audit-report.json", [], reportExists),
  requiredReport("rendered-export-visual-summary", ".tmp/rendered-export-audit/visual-review-summary.json", [], visualSummaryPassed),
  requiredReport("external-engine-probe", ".tmp/external-engines/probe-report.json", [], externalEngineProbePassed),
  requiredReport("performance-audit", ".tmp/performance-audit/report.json", ["pass", "passed"]),
];

if (process.platform === "darwin") {
  requiredReports.push(requiredReport("macos-app-bundle", ".tmp/desktop-bundle/macos-app-report.json", [], reportExists));
  requiredReports.push(requiredReport("macos-dmg-classification", ".tmp/desktop-bundle/macos-dmg-report.json", [
    "created",
    "created-manual-probe-dmg",
    "classified-host-limitation",
  ]));
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
  const webdriver = reports["macos-webdriver-fallback"];
  if (missingWebdriverPlatforms.length > 0 || (process.platform === "darwin" && webdriver?.status === "skipped")) {
    gaps.push({
      id: "windows-linux-tauri-webdriver-execution",
      status: "pending-supported-hosts",
      evidence: platformEvidence ? ".tmp/platform-evidence/report.json" : ".tmp/desktop-webdriver/report.json",
      detail:
        missingWebdriverPlatforms.length > 0
          ? `Official Tauri WebDriver execution still needs supported-host reports for: ${missingWebdriverPlatforms.join(", ")}.`
          : "macOS records fresh native fallback proof; official Tauri WebDriver execution still needs Windows/Linux hosts.",
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
    .filter((engine) => engine.status === "missing")
    .map((engine) => engine.name);
  if (missingEngines.length > 0) {
    gaps.push({
      id: "optional-external-engines",
      status: "partially-installed",
      evidence: ".tmp/external-engines/probe-report.json",
      detail: `Missing optional engines on this host: ${missingEngines.join(", ")}.`,
    });
  }

  return gaps;
}

function reportExists() {
  return {
    accepted: true,
    status: "present",
  };
}

function desktopCommandPassed(report) {
  const passed = report.nativeCommandWorkflow?.status === 0;
  return {
    accepted: passed,
    status: passed ? "passed" : "failed",
    detail: passed ? `durationMs=${report.nativeCommandWorkflow.durationMs}` : "native command workflow did not pass",
  };
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
  return {
    accepted: incompatible.length === 0,
    status: incompatible.length === 0 ? "passed" : "failed",
    detail: incompatible.length === 0 ? "installed engines smoke-compatible" : `incompatible=${incompatible.map((engine) => engine.name).join(", ")}`,
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
  const passed = report.launch?.status === "survived-until-timeout" || report.status === "survived-until-timeout";
  return {
    accepted: passed,
    status: passed ? "passed" : report.launch?.status || report.status || "unknown",
    detail: passed ? "bounded GUI launch survived until timeout" : "",
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
  const fallbackPassed = report.status === "skipped" && report.fallbackProof?.status === "passed" && report.fallbackProof?.freshForBinary;
  return {
    accepted: fallbackPassed,
    status: report.status || "unknown",
    detail: fallbackPassed ? "macOS unsupported WebDriver skip has fresh native fallback proof" : "WebDriver did not pass and no accepted fallback proof was found",
  };
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
