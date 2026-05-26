import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = join(root, ".tmp", "homebrew", "homebrew-packaging-report.json");
const packageJson = readJson("package.json");
const tauriConfig = readJson("src-tauri/tauri.conf.json");
const templatePath = "packaging/homebrew/Casks/neditor.rb.template";
const docsPath = "docs/homebrew-distribution.md";
const template = readText(templatePath);
const docs = readText(docsPath);
const README = readText("README.md");
const issues = [];
const blockers = [];

const suppliedCaskPath = process.env.NEDITOR_HOMEBREW_CASK
  ? resolve(process.env.NEDITOR_HOMEBREW_CASK)
  : defaultHomebrewCaskPath();
const suppliedArtifactPath = process.env.NEDITOR_HOMEBREW_ARTIFACT
  ? resolve(process.env.NEDITOR_HOMEBREW_ARTIFACT)
  : defaultHomebrewArtifactPath();
const releaseSigningReport = readOptionalJson(".tmp/release-signing/report.json");
const releaseReadinessReport = readOptionalJson(".tmp/release-readiness/report.json");

requireEqual(packageJson.scripts?.["check:homebrew"], "node scripts/check-homebrew-packaging.mjs", "package.json must expose check:homebrew");
requireIncludes(README, "pnpm run check:homebrew", "README must document the Homebrew packaging gate");
requireIncludes(docs, "packaging/homebrew/Casks/neditor.rb.template", "Homebrew docs must point at the cask template");
requireIncludes(docs, "signed and notarized", "Homebrew docs must require signed and notarized macOS artifacts");
requireIncludes(docs, "brew audit --cask --new", "Homebrew docs must include the final cask audit command");
requireIncludes(docs, "`ned`", "Homebrew docs must describe the ned CLI helper");

validateCaskTemplate(template, { placeholdersRequired: true, label: templatePath });
requireEqual(tauriConfig.productName, "NEditor", "Tauri productName must match the Homebrew app stanza");
requireEqual(tauriConfig.identifier, "com.neditor.desktop", "Tauri identifier must match cask zap paths");
if (tauriConfig.bundle?.active !== true) issues.push("Tauri bundling must remain active for Homebrew distribution");
if (!bundleTargetsMacArtifact(tauriConfig.bundle?.targets)) {
  issues.push(`Tauri bundle targets must include macOS app/dmg output; found ${JSON.stringify(tauriConfig.bundle?.targets)}`);
}

const caskEvidence = suppliedCaskPath ? validateSuppliedCask(suppliedCaskPath) : null;
const artifactEvidence = suppliedArtifactPath ? validateSuppliedArtifact(suppliedArtifactPath, caskEvidence) : null;

if (!suppliedCaskPath) {
  blockers.push({
    id: "homebrew-final-cask",
    status: "pending-release-cask",
    detail: "Set NEDITOR_HOMEBREW_CASK to a release cask with concrete version and sha256 before publishing a tap.",
  });
}
if (!suppliedArtifactPath) {
  blockers.push({
    id: "homebrew-release-artifact",
    status: "pending-macos-artifact",
    detail: "Set NEDITOR_HOMEBREW_ARTIFACT to the signed/notarized macOS zip or DMG to verify the cask checksum.",
  });
}
if (!macosSigningAccepted(releaseSigningReport)) {
  blockers.push({
    id: "homebrew-macos-signing",
    status: "pending-signing-notarization",
    detail: "Homebrew distribution requires accepted macOS signing/notarization evidence from pnpm run check:release-signing.",
  });
}
if (releaseReadinessReport && releaseReadinessReport.status === "failed") {
  blockers.push({
    id: "homebrew-release-readiness",
    status: "release-readiness-failed",
    detail: "pnpm run check:release-readiness currently reports failures; do not publish the cask.",
  });
}

writeReport({
  generatedAt: new Date().toISOString(),
  status: issues.length > 0 ? "failed" : blockers.length > 0 ? "passed-with-release-blockers" : "ready",
  appVersion: packageJson.version,
  cask: {
    template: templatePath,
    suppliedPath: suppliedCaskPath,
    expectedToken: "neditor",
    artifactName: `NEditor-${packageJson.version}-macos.zip`,
    supplied: caskEvidence,
  },
  artifact: artifactEvidence,
  qualityGates: {
    caskTemplate: issues.length === 0,
    macosSigningAccepted: macosSigningAccepted(releaseSigningReport),
    releaseReadinessStatus: releaseReadinessReport?.status || "missing",
  },
  blockers,
  issues,
});

if (issues.length > 0) {
  console.error("Homebrew packaging audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

console.log(`Checked Homebrew cask packaging contract; wrote ${relative(reportPath)}.`);

function validateCaskTemplate(source, { placeholdersRequired, label }) {
  requireIncludes(source, 'cask "neditor" do', `${label} must define the neditor cask token`);
  requireIncludes(source, 'version "__VERSION__"', `${label} must carry a replaceable version placeholder`);
  requireIncludes(source, 'sha256 "__SHA256__"', `${label} must carry a replaceable sha256 placeholder`);
  requireIncludes(source, "github.com/nyimbi/Neditor/releases/download/v#{version}", `${label} must use versioned GitHub release artifacts`);
  requireIncludes(source, 'verified: "github.com/nyimbi/Neditor/"', `${label} must constrain the verified download host`);
  requireIncludes(source, 'name "NEditor"', `${label} must name the application`);
  requireIncludes(source, 'desc "Local-first AI-assisted Markdown workbench for business documents"', `${label} must describe the product`);
  requireIncludes(source, 'homepage "https://github.com/nyimbi/Neditor"', `${label} must declare the project homepage`);
  requireIncludes(source, 'app "NEditor.app"', `${label} must install the app bundle`);
  requireIncludes(source, 'binary "#{appdir}/NEditor.app/Contents/MacOS/ned", target: "ned"', `${label} must expose the ned command-line helper from inside the app bundle`);
  requireIncludes(source, "~/Library/Application Support/com.neditor.desktop", `${label} must include app-data zap cleanup`);
  if (!placeholdersRequired) {
    if (source.includes("__VERSION__") || source.includes("__SHA256__")) {
      issues.push(`${label} must not contain template placeholders`);
    }
  }
}

function validateSuppliedCask(caskPath) {
  if (!existsSync(caskPath)) {
    issues.push(`supplied Homebrew cask is missing: ${caskPath}`);
    return { status: "missing", path: caskPath };
  }
  const source = readFileSync(caskPath, "utf8");
  validateCaskTemplate(source, { placeholdersRequired: false, label: caskPath });
  const version = source.match(/^\s*version\s+"([^"]+)"/m)?.[1] || "";
  const sha256 = source.match(/^\s*sha256\s+"([^"]+)"/m)?.[1] || "";
  if (version !== packageJson.version) {
    issues.push(`supplied Homebrew cask version ${version || "(missing)"} must match package.json ${packageJson.version}`);
  }
  if (!/^[a-f0-9]{64}$/i.test(sha256)) {
    issues.push("supplied Homebrew cask sha256 must be a 64-character hexadecimal digest");
  }
  return {
    status: "checked",
    path: caskPath,
    version,
    sha256,
    hasPlaceholders: source.includes("__VERSION__") || source.includes("__SHA256__"),
  };
}

function validateSuppliedArtifact(artifactPath, caskEvidence) {
  if (!existsSync(artifactPath)) {
    issues.push(`supplied Homebrew artifact is missing: ${artifactPath}`);
    return { status: "missing", path: artifactPath };
  }
  const stat = statSync(artifactPath);
  if (!stat.isFile()) {
    issues.push(`supplied Homebrew artifact must be a file: ${artifactPath}`);
    return { status: "not-file", path: artifactPath };
  }
  const data = readFileSync(artifactPath);
  const sha256 = createHash("sha256").update(data).digest("hex");
  if (caskEvidence?.sha256 && /^[a-f0-9]{64}$/i.test(caskEvidence.sha256) && caskEvidence.sha256.toLowerCase() !== sha256) {
    issues.push("supplied Homebrew artifact sha256 does not match the supplied cask");
  }
  return {
    status: "checked",
    path: artifactPath,
    bytes: stat.size,
    sha256,
  };
}

function macosSigningAccepted(report) {
  const darwin = report?.platforms?.find((platform) => platform?.platform === "darwin");
  return darwin?.status === "accepted";
}

function defaultHomebrewCaskPath() {
  const path = join(root, ".tmp", "homebrew", "external", "neditor.rb");
  return existsSync(path) ? path : null;
}

function defaultHomebrewArtifactPath() {
  const candidates = [
    join(root, ".tmp", "homebrew", "external", "neditor-release-artifact"),
    join(root, ".tmp", "homebrew", "external", `NEditor-${packageJson.version}-macos.zip`),
    join(root, ".tmp", "homebrew", "external", `NEditor-${packageJson.version}-macos.dmg`),
    join(root, ".tmp", "homebrew", "external", "NEditor-macos.zip"),
    join(root, ".tmp", "homebrew", "external", "NEditor-macos.dmg"),
  ];
  return candidates.find((path) => existsSync(path)) || null;
}

function bundleTargetsMacArtifact(targets) {
  if (targets === "all") return true;
  const values = Array.isArray(targets) ? targets : [targets].filter(Boolean);
  return values.some((target) => ["all", "app", "dmg"].includes(target));
}

function requireIncludes(text, needle, message) {
  if (!text.includes(needle)) issues.push(message);
}

function requireEqual(actual, expected, message) {
  if (actual !== expected) issues.push(`${message}: expected ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
}

function readJson(relativePath) {
  return JSON.parse(readText(relativePath));
}

function readText(relativePath) {
  return readFileSync(join(root, relativePath), "utf8");
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
