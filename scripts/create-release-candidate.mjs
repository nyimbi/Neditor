import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, extname, join, relative, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outputDir = resolve(process.env.NEDITOR_RELEASE_CANDIDATE_DIR || join(root, ".tmp", "release-candidate"));
const args = new Set(process.argv.slice(2));
const skipBuild = args.has("--skip-build");
const allowDirty = args.has("--allow-dirty");
const skipEvidence = args.has("--skip-evidence") || allowDirty;
const skipPrerequisiteEvidence = args.has("--skip-prerequisite-evidence") || skipEvidence;
const refreshBrowserEvidence = args.has("--refresh-browser-evidence");
const refreshNativeLaunchEvidence = args.has("--refresh-native-launch-evidence");
const commandResults = [];

const packageJson = readJson("package.json");
const tauriConfig = readJson("src-tauri/tauri.conf.json");
const sourceCommit = git(["rev-parse", "HEAD"]).stdout.trim();
const sourceTreeCleanBefore = gitTreeClean();

if (!sourceTreeCleanBefore && !allowDirty) {
  fail("Release candidates must be created from a clean Git worktree. Commit or stash changes, or pass --allow-dirty for a non-releaseable local dry run.");
}

if (!skipBuild) {
  run("pnpm", ["run", "build"]);
  run("cargo", ["build", "--manifest-path", "src-tauri/Cargo.toml", "--locked", "--release"]);
  run("pnpm", ["run", "prepare:sidecars"]);
}

if (!skipEvidence) {
  if (!skipPrerequisiteEvidence) {
    refreshPrerequisiteEvidence();
    // Tauri prerequisite builds can refresh target/release/ned after beforeBuildCommand prepares sidecars.
    run("pnpm", ["run", "prepare:sidecars"]);
  }
  runReadinessBootstrap();
  run("pnpm", ["run", "collect:evidence-kit"]);
  run("pnpm", ["run", "check:evidence-kit"]);
  run("pnpm", ["run", "check:release-readiness"]);
  run("pnpm", ["run", "collect:evidence-kit"]);
  run("pnpm", ["run", "check:evidence-kit"]);
  run("pnpm", ["run", "check:release-readiness"]);
}

const readiness = readJson(".tmp/release-readiness/report.json");
const evidenceKit = readJson(".tmp/release-evidence-kit/manifest.json");
const evidenceKitReport = readOptionalJson(".tmp/release-evidence-kit/report.json");
const evidenceWorkItemsById = new Map((Array.isArray(evidenceKit.gapWorkItems) ? evidenceKit.gapWorkItems : []).map((item) => [item.id, item]));
const evidenceKitCurrentForSource = evidenceKit.sourceCommit === sourceCommit;
const candidateEvidenceGaps = releaseCandidateGaps(readiness);
const evidenceKitCoversReadiness = candidateEvidenceGaps.every((gap) => gap.readyToSend);
const evidenceKitReportCurrentForReadiness =
  evidenceKitReport?.status === "passed" &&
  evidenceKitReport.sourceCommit === sourceCommit &&
  evidenceKitReport.currentSourceCommit === sourceCommit &&
  evidenceKitReport.sourceTreeClean === true &&
  evidenceKitReport.currentSourceTreeClean === true &&
  evidenceKitReport.currentReadinessStatus === readiness.status &&
  Number(evidenceKitReport.summary?.issues || 0) === 0 &&
  Number(evidenceKitReport.summary?.gaps || -1) === candidateEvidenceGaps.length;
const sourceTreeCleanAfter = gitTreeClean();
const artifacts = collectArtifacts();
const requiredArtifacts = ["frontend:index", "native:app-binary", "native:ned-cli", "native:prepared-ned-sidecar"];
const missingRequired = requiredArtifacts.filter((kind) => !artifacts.some((artifact) => artifact.kind === kind));
const sidecarMismatches = preparedSidecarMismatches(artifacts);

if (missingRequired.length) {
  fail(`Release candidate is missing required artifact(s): ${missingRequired.join(", ")}`);
}
if (sidecarMismatches.length) {
  fail(`Prepared ned sidecar hash does not match the release CLI binary: ${sidecarMismatches.map((artifact) => artifact.path).join(", ")}`);
}

const manifest = {
  schema: "neditor.local-release-candidate.v1",
  generatedAt: new Date().toISOString(),
  releaseable:
    sourceTreeCleanBefore &&
    sourceTreeCleanAfter &&
    evidenceKitCurrentForSource &&
    evidenceKitCoversReadiness &&
    evidenceKitReportCurrentForReadiness &&
    readiness.status === "current-host-ready-with-external-gaps",
  product: {
    name: tauriConfig.productName || packageJson.name,
    packageName: packageJson.name,
    version: packageJson.version,
    tauriVersion: tauriConfig.version,
    identifier: tauriConfig.identifier,
    license: packageJson.license,
  },
  source: {
    commit: sourceCommit,
    treeCleanBefore: sourceTreeCleanBefore,
    treeCleanAfter: sourceTreeCleanAfter,
  },
  host: {
    platform: process.platform,
    arch: process.arch,
    node: process.version,
  },
  readiness: {
    status: readiness.status,
    summary: readiness.summary || null,
    evidenceGapCount: candidateEvidenceGaps.length,
    evidenceGaps: candidateEvidenceGaps,
  },
  evidenceKit: {
    path: ".tmp/release-evidence-kit/manifest.json",
    reportPath: ".tmp/release-evidence-kit/report.json",
    schema: evidenceKit.schema,
    sourceCommit: evidenceKit.sourceCommit,
    currentForSource: evidenceKitCurrentForSource,
    coversReadiness: evidenceKitCoversReadiness,
    reportStatus: evidenceKitReport?.status || "missing",
    reportCurrentForReadiness: evidenceKitReportCurrentForReadiness,
    gapCount: Array.isArray(evidenceKit.gaps) ? evidenceKit.gaps.length : 0,
    workItemCount: Array.isArray(evidenceKit.gapWorkItems) ? evidenceKit.gapWorkItems.length : 0,
    specWorkOrders: evidenceKit.specCompletionWorkOrders || null,
  },
  artifacts,
  commands: commandResults,
  nextSteps: nextSteps(readiness),
};

mkdirSync(outputDir, { recursive: true });
writeFileSync(join(outputDir, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
writeFileSync(join(outputDir, "SHA256SUMS"), renderSha256Sums(artifacts));
writeFileSync(join(outputDir, "README.md"), renderReadme(manifest));

console.log(`Release candidate manifest written to ${relativePath(join(outputDir, "manifest.json"))}.`);
console.log(`Release candidate SHA256SUMS written to ${relativePath(join(outputDir, "SHA256SUMS"))}.`);
if (!manifest.releaseable) {
  console.log("Release candidate is not final-releaseable; inspect README.md for remaining gates.");
}

function collectArtifacts() {
  const entries = [
    artifact("frontend:index", "dist/index.html"),
    artifact("native:app-binary", "src-tauri/target/release/neditor"),
    artifact("native:ned-cli", "src-tauri/target/release/ned"),
    ...collectPreparedSidecarArtifacts(),
    ...collectFrontendAssets(),
    ...collectBundleArtifacts(),
  ].filter(Boolean);
  return entries.sort((a, b) => a.path.localeCompare(b.path));
}

function collectPreparedSidecarArtifacts() {
  const binariesDir = join(root, "src-tauri", "binaries");
  if (!existsSync(binariesDir)) return [];
  return readdirSync(binariesDir)
    .filter((name) => /^ned-[^/\\]+(?:\.exe)?$/.test(name))
    .filter((name) => statSync(join(binariesDir, name)).isFile())
    .map((name) => artifact("native:prepared-ned-sidecar", join("src-tauri", "binaries", name)));
}

function preparedSidecarMismatches(artifacts) {
  const cli = artifacts.find((artifact) => artifact.kind === "native:ned-cli");
  if (!cli) return [];
  return artifacts.filter((artifact) => artifact.kind === "native:prepared-ned-sidecar" && artifact.sha256 !== cli.sha256);
}

function refreshPrerequisiteEvidence() {
  const commands = [
    ["pnpm", ["run", "check:release-ci"]],
    ["pnpm", ["run", "check:homebrew"]],
    ["pnpm", ["run", "check:platform-packaging"]],
    ["pnpm", ["run", "check:spec-completion"]],
    ["pnpm", ["run", "check:a11y"]],
    ["pnpm", ["run", "check:platform-evidence"]],
    ["pnpm", ["run", "check:release-signing"]],
    ["pnpm", ["run", "check:ai-provider"]],
    ["pnpm", ["run", "check:ai-runtime"]],
    ["pnpm", ["run", "check:security-review"]],
    ["pnpm", ["run", "check:performance-profile"]],
    ["pnpm", ["run", "check:google-docs-import"]],
    ["pnpm", ["run", "test:rendered-exports"]],
    ["pnpm", ["run", "check:tables:manual"]],
    ["pnpm", ["run", "check:a11y:manual"]],
    ["pnpm", ["run", "test:desktop-bundle"]],
    ["pnpm", ["run", "test:desktop-dmg"]],
    ["pnpm", ["run", "test:desktop-smoke"]],
  ];
  if (refreshBrowserEvidence) {
    commands.push(["pnpm", ["run", "check:a11y:runtime"]], ["pnpm", ["run", "test:performance-audit"]]);
  }
  if (refreshNativeLaunchEvidence) {
    commands.push(["pnpm", ["run", "test:desktop-smoke"], { NEDITOR_DESKTOP_SMOKE_LAUNCH: "1" }], ["pnpm", ["run", "test:tauri-webdriver"]]);
  }
  for (const [command, commandArgs, env] of commands) run(command, commandArgs, env || {});
}

function collectFrontendAssets() {
  const assetsDir = join(root, "dist", "assets");
  if (!existsSync(assetsDir)) return [];
  return readdirSync(assetsDir)
    .filter((name) => statSync(join(assetsDir, name)).isFile())
    .map((name) => artifact(`frontend:asset:${extname(name).slice(1) || "file"}`, join("dist", "assets", name)));
}

function collectBundleArtifacts() {
  const bundleDir = join(root, "src-tauri", "target", "release", "bundle");
  if (!existsSync(bundleDir)) return [];
  const output = [];
  for (const path of walk(bundleDir)) {
    const rel = relativePath(path);
    if (isReleaseBundleFile(rel)) output.push(artifact(`bundle:${bundleKind(rel)}`, rel));
  }
  return output;
}

function artifact(kind, relPath) {
  const path = resolve(root, relPath);
  if (!existsSync(path) || !statSync(path).isFile()) return null;
  const stats = statSync(path);
  return {
    kind,
    path: relativePath(path),
    size: stats.size,
    sha256: sha256(path),
  };
}

function isReleaseBundleFile(path) {
  if (path.endsWith("/NEditor.app/Contents/Info.plist")) return true;
  if (path.endsWith("/NEditor.app/Contents/MacOS/neditor")) return true;
  if (path.endsWith("/NEditor.app/Contents/MacOS/ned")) return true;
  return /\.(dmg|msi|exe|deb|rpm|AppImage|appimage|zip)$/i.test(path);
}

function bundleKind(path) {
  if (path.includes(".app/Contents/")) return "macos-app";
  const extension = extname(path).replace(".", "").toLowerCase();
  return extension || "file";
}

function walk(directory) {
  const output = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) output.push(...walk(path));
    if (entry.isFile()) output.push(path);
  }
  return output;
}

function run(command, args, env = {}) {
  const startedAt = new Date().toISOString();
  const result = spawnSync(command, args, { cwd: root, encoding: "utf8", env: { ...process.env, ...env }, stdio: "pipe" });
  const report = {
    command: [command, ...args].join(" "),
    env: Object.keys(env).sort(),
    startedAt,
    status: result.status ?? 1,
    stdoutTail: tail(result.stdout || ""),
    stderrTail: tail(result.stderr || ""),
  };
  commandResults.push(report);
  if (report.status !== 0 && !allowBootstrapReadinessFailure(report)) {
    writeCommandFailure(report);
    fail(`${report.command} failed with exit code ${report.status}`);
  }
  return result;
}

function runReadinessBootstrap() {
  run("pnpm", ["run", "check:release-readiness"], { NEDITOR_RELEASE_CANDIDATE_BOOTSTRAP: "1" });
}

function allowBootstrapReadinessFailure(report) {
  if (!report.env.includes("NEDITOR_RELEASE_CANDIDATE_BOOTSTRAP")) return false;
  const readinessReport = readOptionalJson(".tmp/release-readiness/report.json");
  const failures = Array.isArray(readinessReport?.failures) ? readinessReport.failures : [];
  const staleEvidenceKitOnly = failures.length > 0 && failures.every((failure) => String(failure).startsWith("release-evidence-kit "));
  if (staleEvidenceKitOnly) {
    report.allowedFailure = true;
    report.allowedFailureReason = "bootstrap-readiness-report-for-stale-evidence-kit";
  }
  return staleEvidenceKitOnly;
}

function writeCommandFailure(report) {
  mkdirSync(outputDir, { recursive: true });
  writeFileSync(
    join(outputDir, "failed-command.json"),
    `${JSON.stringify(
      {
        schema: "neditor.local-release-candidate-command-failure.v1",
        generatedAt: new Date().toISOString(),
        sourceCommit,
        report,
        commands: commandResults,
      },
      null,
      2,
    )}\n`,
  );
}

function renderSha256Sums(artifacts) {
  return `${artifacts.map((artifact) => `${artifact.sha256}  ${artifact.path}`).join("\n")}\n`;
}

function renderReadme(candidate) {
  const gapLines = candidate.readiness.evidenceGaps.length
    ? candidate.readiness.evidenceGaps.map((gap) => releaseGateLine(gap)).join("\n")
    : "- None.";
  const artifactLines = candidate.artifacts.map((artifact) => `- \`${artifact.path}\` (${artifact.kind}, ${artifact.size} bytes)`).join("\n");
  const commandLines = candidate.commands.length
    ? candidate.commands.map((command) => `- \`${command.command}\` -> ${command.status}${command.allowedFailure ? ` (${command.allowedFailureReason})` : ""}`).join("\n")
    : "- No commands were run; this was created with skip flags.";
  const nextStepLines = candidate.nextSteps.map((step) => `- ${step}`).join("\n");
  return `${[
    "# NEditor Local Release Candidate",
    "",
    `Generated: ${candidate.generatedAt}`,
    `Version: ${candidate.product.version}`,
    `Commit: ${candidate.source.commit}`,
    `Releaseable on this host: ${candidate.releaseable ? "yes" : "no"}`,
    `Readiness status: ${candidate.readiness.status}`,
    "",
    "## Artifacts",
    "",
    artifactLines,
    "",
    "## SHA-256 Checksums",
    "",
    "See `SHA256SUMS` for hashes of every compiled frontend, native binary, and discovered bundle artifact.",
    "",
    "## Commands",
    "",
    commandLines,
    "",
    "## Remaining Release Gates",
    "",
    gapLines,
    "",
    "## Next Steps",
    "",
    nextStepLines,
    "",
  ].join("\n")}\n`;
}

function releaseCandidateGaps(readiness) {
  return evidenceGaps(readiness).map((gap) => {
    const workItem = evidenceWorkItemsById.get(gap.id) || {};
    return {
      id: gap.id,
      status: gap.status,
      detail: gap.detail || workItem.detail || "",
      evidence: gap.evidence || workItem.evidence || null,
      runbooks: workItem.runbooks || [],
      returnedEvidencePaths: workItem.returns || gap.returnedEvidencePaths || gap.returnPaths || [],
      validatorCommands: workItem.validatorCommands || gap.validatorCommands || [],
      ingestCommand: workItem.ingestCommand || null,
      finalReadinessCommand: workItem.finalReadinessCommand || "pnpm run check:release-readiness",
      readyToSend: workItem.readyToSend === true,
    };
  });
}

function releaseGateLine(gap) {
  const runbooks = gap.runbooks.length ? gap.runbooks.map((runbook) => runbook.path).join(", ") : "no runbook";
  const returns = gap.returnedEvidencePaths.length ? gap.returnedEvidencePaths.join(", ") : "no return paths";
  const validators = gap.validatorCommands.length ? gap.validatorCommands.join(" | ") : "no validator commands";
  return `- ${gap.id}: ${gap.status}; runbooks: ${runbooks}; returns: ${returns}; validators: ${validators}`;
}

function nextSteps(readiness) {
  const gaps = evidenceGaps(readiness);
  if (!gaps.length) return ["Tag the release, publish signed artifacts, and archive this release-candidate directory."];
  return [
    "Send `.tmp/release-evidence-kit` to supported-host owners and human reviewers.",
    "Ingest returned evidence with `pnpm run ingest:evidence -- --source /path/to/unpacked-artifacts`.",
    "Rerun `pnpm run collect:evidence-kit`, `pnpm run check:evidence-kit`, and `pnpm run check:release-readiness`.",
    "Regenerate this release candidate after every source, artifact, signing, or evidence change.",
  ];
}

function evidenceGaps(readiness) {
  return Array.isArray(readiness?.evidenceGaps) ? readiness.evidenceGaps : Array.isArray(readiness?.gaps) ? readiness.gaps : [];
}

function git(args) {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
  if (result.status !== 0) fail(`git ${args.join(" ")} failed: ${result.stderr || result.stdout}`);
  return result;
}

function gitTreeClean() {
  return git(["status", "--porcelain"]).stdout.trim() === "";
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function readJson(relativePath) {
  return JSON.parse(readFileSync(join(root, relativePath), "utf8"));
}

function readOptionalJson(relativePath) {
  const path = join(root, relativePath);
  if (!existsSync(path)) return null;
  return JSON.parse(readFileSync(path, "utf8"));
}

function tail(value) {
  return value.trim().split(/\r?\n/).filter(Boolean).slice(-10);
}

function relativePath(path) {
  return relative(root, path).replace(/\\/g, "/");
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
