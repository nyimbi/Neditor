import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = join(root, ".tmp", "desktop-bundle", "macos-dmg-report.json");
const dmgDirectory = join(root, "src-tauri", "target", "release", "bundle", "dmg");
const dmgPath = join(dmgDirectory, "NEditor_0.1.0_aarch64.dmg");
const manualProbePath = join(dmgDirectory, "NEditor_manual_probe.dmg");
const dmgScriptPath = join(dmgDirectory, "bundle_dmg.sh");
const macosBundleDirectory = join(root, "src-tauri", "target", "release", "bundle", "macos");

if (process.platform !== "darwin") {
  console.log("Desktop DMG smoke currently verifies macOS DMG bundling; skipping on this host.");
  process.exit(0);
}

removeIfPresent(dmgPath);
removeIfPresent(manualProbePath);

const tauriResult = run("./node_modules/.bin/tauri", ["build", "--bundles", "dmg"]);
if (tauriResult.status === 0) {
  requireDmgArtifact(dmgPath);
  writeReport({
    status: "passed",
    result: "created-dmg",
    dmg: artifactInfo(dmgPath),
    tauri: commandReport(tauriResult),
  });
  console.log(`Checked NEditor macOS DMG artifact at ${relative(dmgPath)}.`);
  process.exit(0);
}

const probeResult = existsSync(dmgScriptPath) ? runDmgProbe() : null;
if (probeResult?.status === 0) {
  requireDmgArtifact(manualProbePath);
  writeReport({
    status: "passed",
    result: "created-manual-probe-dmg",
    dmg: artifactInfo(manualProbePath),
    tauri: commandReport(tauriResult),
    manualProbe: commandReport(probeResult),
  });
  console.log(`Checked NEditor macOS DMG artifact at ${relative(manualProbePath)}.`);
  process.exit(0);
}

const combinedOutput = [tauriResult.output, probeResult?.output].filter(Boolean).join("\n");
if (isKnownSandboxedHdiutilFailure(combinedOutput)) {
  writeReport({
    status: "classified-host-limitation",
    result: "hdiutil-sandbox-device-not-configured",
    tauri: commandReport(tauriResult),
    manualProbe: probeResult ? commandReport(probeResult) : null,
    classification: {
      host: `${process.platform}-${process.arch}`,
      cause: "hdiutil could not start hdiejectd because the process is sandboxed, then returned Device not configured.",
      appBundleStillBuilt: existsSync(join(macosBundleDirectory, "NEditor.app")),
    },
  });
  console.log("Classified macOS DMG bundling as a host hdiutil sandbox limitation.");
  process.exit(0);
}

writeReport({
  status: "failed",
  result: "unclassified-dmg-failure",
  tauri: commandReport(tauriResult),
  manualProbe: probeResult ? commandReport(probeResult) : null,
});
console.error("Desktop DMG smoke failed with an unclassified bundling error.");
console.error(combinedOutput.trim());
process.exit(1);

function runDmgProbe() {
  return run("bash", [
    dmgScriptPath,
    "--hdiutil-verbose",
    "--skip-jenkins",
    "--volname",
    "NEditor",
    "--volicon",
    join(dmgDirectory, "icon.icns"),
    "--icon",
    "NEditor.app",
    "130",
    "190",
    "--app-drop-link",
    "425",
    "190",
    manualProbePath,
    macosBundleDirectory,
  ]);
}

function run(cmd, args) {
  const result = spawnSync(cmd, args, {
    cwd: root,
    encoding: "utf8",
  });
  return {
    cmd,
    args,
    status: result.status ?? 1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    output: [result.stdout?.trim(), result.stderr?.trim()].filter(Boolean).join("\n"),
  };
}

function isKnownSandboxedHdiutilFailure(output) {
  return (
    output.includes("Cannot start hdiejectd because app is sandboxed") &&
    output.includes("hdiutil: create failed - Device not configured")
  );
}

function requireDmgArtifact(path) {
  if (!existsSync(path) || !statSync(path).isFile()) {
    throw new Error(`Expected DMG artifact is missing: ${relative(path)}`);
  }
  const size = statSync(path).size;
  if (size < 1_000_000) {
    throw new Error(`Expected DMG artifact is unexpectedly small: ${relative(path)} is ${size} bytes`);
  }
}

function artifactInfo(path) {
  const stats = statSync(path);
  return {
    path: relative(path),
    size: stats.size,
  };
}

function commandReport(result) {
  return {
    command: [result.cmd, ...result.args].join(" "),
    status: result.status,
    stdoutTail: tail(result.stdout),
    stderrTail: tail(result.stderr),
  };
}

function tail(value) {
  const lines = value.trim().split(/\r?\n/).filter(Boolean);
  return lines.slice(-12);
}

function writeReport(report) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify({ generatedAt: new Date().toISOString(), ...report }, null, 2)}\n`);
}

function removeIfPresent(path) {
  if (existsSync(path)) {
    unlinkSync(path);
  }
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
