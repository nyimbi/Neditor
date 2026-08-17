/**
 * ship-release.mjs — single-command release automation for NEditor.
 *
 * Usage:
 *   node scripts/ship-release.mjs [--dry-run] [--sign] [--homebrew] [--skip-preflight]
 *
 *   --dry-run        Run every step except tag push, release create, and
 *                    homebrew push.  Compile/verify still execute.
 *   --sign           Hard-require Apple signing; fail if APPLE_ID is unset.
 *   --homebrew       Enable Homebrew tap update (default: off).
 *   --skip-preflight Skip step 1 version/branch/readiness checks.
 */

import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import process from "node:process";
import { fileURLToPath } from "node:url";

// ─── Bootstrap ──────────────────────────────────────────────────────────────

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const args = parseArgs(process.argv.slice(2));
const DRY_RUN = args["dry-run"] === true;
const HARD_SIGN = args["sign"] === true;
const HOMEBREW = args["homebrew"] === true;
const SKIP_PREFLIGHT = args["skip-preflight"] === true;

const PLATFORM = process.platform; // "darwin" | "win32" | "linux"

// Apple signing (macOS)
const APPLE_ID = process.env.APPLE_ID ?? "";
const APPLE_PASSWORD = process.env.APPLE_PASSWORD ?? "";
const APPLE_TEAM_ID = process.env.APPLE_TEAM_ID ?? "";
const APPLE_SIGNING_IDENTITY = process.env.APPLE_SIGNING_IDENTITY ?? "";

const releaseArtifacts = []; // absolute paths collected during compile/verify
const stepTimings = [];

log("=".repeat(72));
log(`NEditor ship-release  platform=${PLATFORM}  dry-run=${DRY_RUN}  sign=${HARD_SIGN}  homebrew=${HOMEBREW}`);
log("=".repeat(72));

if (DRY_RUN) log("[dry-run] All shell commands will be printed but not executed; tag/release/homebrew pushes will be skipped.\n");

// ─── Step 1: Preflight ───────────────────────────────────────────────────────

if (SKIP_PREFLIGHT) {
  log("[skip-preflight] Skipping step 1.\n");
} else {
  step("preflight", () => {
    // 1a. Tree clean
    const dirty = git(["status", "--porcelain"]).trim();
    if (dirty) fail("Working tree is not clean. Commit or stash changes before releasing.");

    // 1b. Branch is main
    const branch = git(["rev-parse", "--abbrev-ref", "HEAD"]).trim();
    if (branch !== "main") fail(`Must release from 'main'; current branch is '${branch}'.`);

    // 1c. Version consistency
    const pkgVersion = readJson("package.json").version;
    const tauriVersion = readJson("src-tauri/tauri.conf.json").version;
    const cargoVersion = cargoTomlVersion();

    log(`  package.json        ${pkgVersion}`);
    log(`  tauri.conf.json     ${tauriVersion}`);
    log(`  Cargo.toml          ${cargoVersion}`);

    if (pkgVersion !== tauriVersion || pkgVersion !== cargoVersion) {
      fail(
        `Version mismatch — package.json=${pkgVersion} tauri.conf.json=${tauriVersion} Cargo.toml=${cargoVersion}. ` +
          "Align all three before releasing.",
      );
    }

    // 1d. Release readiness gate
    run("pnpm", ["run", "check:release-readiness"]);
  });
}

// ─── Resolve version ─────────────────────────────────────────────────────────

const VERSION = readJson("package.json").version;
const TAG = `v${VERSION}`;
log(`\nReleasing ${TAG}\n`);

// ─── Step 2+3: Compile (+ sign via Tauri env) ────────────────────────────────

step("compile", () => {
  if (PLATFORM === "darwin") {
    compileMacOS();
  } else if (PLATFORM === "win32") {
    log("  [compile] Windows target");
    compileWindows();
  } else if (PLATFORM === "linux") {
    log("  [compile] Linux target");
    compileLinux();
  } else {
    fail(`Unsupported host platform: ${PLATFORM}`);
  }
});

function compileMacOS() {
  log("  [macOS] Building aarch64-apple-darwin ned binary …");
  run("cargo", ["build", "--manifest-path", "src-tauri/Cargo.toml", "--locked", "--release", "--bin", "ned", "--target", "aarch64-apple-darwin"]);

  log("  [macOS] Building x86_64-apple-darwin ned binary …");
  run("cargo", ["build", "--manifest-path", "src-tauri/Cargo.toml", "--locked", "--release", "--bin", "ned", "--target", "x86_64-apple-darwin"]);

  // Universal binary destinations
  const arm = join(root, "src-tauri/target/aarch64-apple-darwin/release/ned");
  const intel = join(root, "src-tauri/target/x86_64-apple-darwin/release/ned");
  const binariesDir = join(root, "src-tauri/binaries");
  const universalBin = join(binariesDir, "ned-universal-apple-darwin");
  const universalRelease = join(root, "src-tauri/target/universal-apple-darwin/release/ned");

  if (!DRY_RUN) {
    mkdirSync(binariesDir, { recursive: true });
    mkdirSync(dirname(universalRelease), { recursive: true });
  }

  log(`  [macOS] lipo → ${relative(universalBin)}`);
  run("lipo", ["-create", "-output", universalBin, arm, intel]);

  log(`  [macOS] lipo → ${relative(universalRelease)}`);
  run("lipo", ["-create", "-output", universalRelease, arm, intel]);

  // Build tauri DMG (Tauri reads Apple signing env vars automatically)
  const signingEnv = buildAppleSigningEnv();
  log("  [macOS] pnpm tauri build --bundles dmg --target universal-apple-darwin …");
  run("pnpm", ["tauri", "build", "--bundles", "dmg", "--target", "universal-apple-darwin"], signingEnv);

  // Collect DMGs (or list expected paths in dry-run)
  const dmgDir = join(root, "src-tauri/target/universal-apple-darwin/release/bundle/dmg");
  if (DRY_RUN) {
    const expectedDmg = join(dmgDir, `NEditor_${VERSION}_universal.dmg`);
    log(`  [dry-run] Expected artifact: ${relative(expectedDmg)}`);
    releaseArtifacts.push(expectedDmg);
  } else {
    collectBundleArtifacts(dmgDir, [".dmg"]);
  }
}

function compileWindows() {
  run("pnpm", ["tauri", "build", "--bundles", "msi,nsis"]);
  const bundleDir = join(root, "src-tauri/target/release/bundle");
  if (DRY_RUN) {
    for (const ext of ["msi", "exe"]) {
      const p = join(bundleDir, `NEditor_${VERSION}_x64_en-US.${ext}`);
      log(`  [dry-run] Expected artifact: ${relative(p)}`);
      releaseArtifacts.push(p);
    }
  } else {
    collectBundleArtifacts(bundleDir, [".msi", ".exe"]);
  }
}

function compileLinux() {
  run("pnpm", ["tauri", "build", "--bundles", "deb,appimage"]);
  const bundleDir = join(root, "src-tauri/target/release/bundle");
  if (DRY_RUN) {
    for (const [sub, ext] of [["deb", "deb"], ["appimage", "AppImage"]]) {
      const p = join(bundleDir, sub, `neditor_${VERSION}_amd64.${ext}`);
      log(`  [dry-run] Expected artifact: ${relative(p)}`);
      releaseArtifacts.push(p);
    }
  } else {
    collectBundleArtifacts(bundleDir, [".deb", ".AppImage", ".appimage"]);
  }
}

function buildAppleSigningEnv() {
  if (!APPLE_ID) {
    if (HARD_SIGN) fail("--sign requires APPLE_ID env var; set Apple credentials or remove --sign.");
    log("  [macOS] APPLE_ID unset — unsigned build (warning: not suitable for distribution).");
    return {};
  }
  if (!APPLE_PASSWORD || !APPLE_TEAM_ID) {
    fail("APPLE_ID is set but APPLE_PASSWORD or APPLE_TEAM_ID is missing.");
  }
  log("  [macOS] Apple credentials present — Tauri will sign and notarize.");
  return {
    APPLE_ID,
    APPLE_PASSWORD,
    APPLE_TEAM_ID,
    ...(APPLE_SIGNING_IDENTITY ? { APPLE_SIGNING_IDENTITY } : {}),
  };
}

// ─── Step 4: Verify ──────────────────────────────────────────────────────────

step("verify", () => {
  if (DRY_RUN) {
    log("  [dry-run] Skipping DMG mount / codesign / lipo-info verify (no built artifacts).");
    log(`  [dry-run] Would write SHA256SUMS covering ${releaseArtifacts.length} artifact(s).`);
    return;
  }

  if (PLATFORM === "darwin") {
    verifyMacOS();
  } else if (PLATFORM === "win32") {
    verifyWindows();
  } else {
    verifyLinux();
  }

  // Write SHA256SUMS alongside artifacts
  const sumLines = releaseArtifacts.map((a) => `${sha256file(a)}  ${relative(a)}`).join("\n");
  const sumsPath = join(root, "SHA256SUMS");
  writeFileSync(sumsPath, `${sumLines}\n`);
  log(`  SHA256SUMS → ${relative(sumsPath)}`);
  releaseArtifacts.push(sumsPath);
});

function verifyMacOS() {
  const dmgs = releaseArtifacts.filter((a) => a.toLowerCase().endsWith(".dmg"));
  if (!dmgs.length) { log("  [verify] No DMG artifacts found; skipping macOS verification."); return; }

  for (const dmg of dmgs) {
    log(`  [verify] Mounting ${relative(dmg)} …`);
    const mountPoint = mkdtempSync(join(tmpdir(), "neditor-dmg-"));
    const attach = spawnSync("hdiutil", ["attach", "-nobrowse", "-mountpoint", mountPoint, dmg], { cwd: root, encoding: "utf8", stdio: "pipe" });

    if (attach.status !== 0) {
      log(`  [verify] WARNING: hdiutil attach failed: ${attach.stderr?.trim()}`);
      continue;
    }

    try {
      const apps = findDotAppDirs(mountPoint);
      if (!apps.length) { log("  [verify] WARNING: no .app found inside DMG."); continue; }
      const appPath = apps[0];
      log(`  [verify] .app: ${appPath}`);

      // lipo -info on each binary
      for (const bin of ["neditor", "ned"]) {
        const binPath = join(appPath, "Contents/MacOS", bin);
        if (existsSync(binPath)) {
          const r = spawnSync("lipo", ["-info", binPath], { encoding: "utf8", stdio: "pipe" });
          log(`  [verify] lipo -info ${bin}: ${r.stdout?.trim() || r.stderr?.trim()}`);
        }
      }

      // PlistBuddy version + copyright
      const plist = join(appPath, "Contents/Info.plist");
      if (existsSync(plist)) {
        const pb = (cmd) => spawnSync("/usr/libexec/PlistBuddy", ["-c", cmd, plist], { encoding: "utf8", stdio: "pipe" }).stdout?.trim();
        const bundleVersion = pb("Print CFBundleShortVersionString");
        const copyright = pb("Print NSHumanReadableCopyright");
        log(`  [verify] CFBundleShortVersionString: ${bundleVersion}`);
        log(`  [verify] NSHumanReadableCopyright: ${copyright}`);
        if (bundleVersion && bundleVersion !== VERSION) {
          fail(`DMG version mismatch: plist reports ${bundleVersion}, expected ${VERSION}.`);
        }
      }

      // Codesign (informational)
      const cs = spawnSync("codesign", ["--verify", "--deep", "--strict", appPath], { encoding: "utf8", stdio: "pipe" });
      log(`  [verify] codesign: ${cs.status === 0 ? "valid signature" : `unsigned/invalid — ${cs.stderr?.trim()}`}`);
    } finally {
      spawnSync("hdiutil", ["detach", mountPoint, "-quiet"], { encoding: "utf8", stdio: "pipe" });
    }

    log(`  [verify] SHA256 ${relative(dmg)}: ${sha256file(dmg)}`);
  }
}

function verifyWindows() {
  for (const installer of releaseArtifacts.filter((a) => a.endsWith(".msi") || a.endsWith(".exe"))) {
    log(`  [verify] SHA256 ${relative(installer)}: ${sha256file(installer)}`);
    const r = spawnSync("signtool", ["verify", "/pa", installer], { encoding: "utf8", stdio: "pipe" });
    log(`  [verify] signtool: ${r.status === 0 ? "valid Authenticode signature" : "not signed or verification failed"}`);
  }
}

function verifyLinux() {
  for (const pkg of releaseArtifacts.filter((a) => a.endsWith(".deb") || /\.AppImage$/i.test(a))) {
    log(`  [verify] SHA256 ${relative(pkg)}: ${sha256file(pkg)}`);
  }
}

// ─── Step 5: Tag + push ───────────────────────────────────────────────────────

step("tag", () => {
  // Local tag
  const localExists = spawnSync("git", ["tag", "-l", TAG], { cwd: root, encoding: "utf8", stdio: "pipe" }).stdout.trim() === TAG;

  if (localExists) {
    log(`  Tag ${TAG} already exists locally — skipping creation.`);
  } else if (DRY_RUN) {
    log(`  [dry-run] Would run: git tag ${TAG}`);
  } else {
    run("git", ["tag", TAG]);
    log(`  Created tag ${TAG}.`);
  }

  // Remote tag — check for divergence
  const remoteTagLine = spawnSync("git", ["ls-remote", "--tags", "origin", TAG], { cwd: root, encoding: "utf8", stdio: "pipe" }).stdout.trim();
  if (remoteTagLine) {
    const remoteCommit = remoteTagLine.split(/\s+/)[0];
    const localCommit = git(["rev-parse", TAG]).trim();
    if (remoteCommit !== localCommit) {
      fail(`Tag ${TAG} on remote points at ${remoteCommit} but local is ${localCommit}. Resolve before releasing.`);
    }
    log(`  Tag ${TAG} already on remote at correct commit (${remoteCommit.slice(0, 8)}).`);
  } else if (DRY_RUN) {
    log(`  [dry-run] Would run: git push origin ${TAG}`);
  } else {
    run("git", ["push", "origin", TAG]);
    log(`  Pushed ${TAG} to origin.`);
  }
});

// ─── Step 6: GitHub Release ───────────────────────────────────────────────────

let releaseUrl = "";

step("release", () => {
  const notesFile = join(root, `docs/release-notes/${TAG}.md`);

  if (!existsSync(notesFile)) {
    const prevTag = latestPreviousTag(TAG);
    const logRange = prevTag ? `${prevTag}..HEAD` : "HEAD";
    const logLines = spawnSync("git", ["log", logRange, "--oneline"], { cwd: root, encoding: "utf8", stdio: "pipe" }).stdout.trim();
    const placeholder = [
      `# NEditor ${VERSION}`,
      "",
      "## Changes",
      "",
      ...logLines.split("\n").filter(Boolean).map((l) => `- ${l}`),
      "",
      "## Known Issues",
      "",
      "<!-- fill in before publishing -->",
      "",
    ].join("\n");
    mkdirSync(dirname(notesFile), { recursive: true });
    writeFileSync(notesFile, placeholder);
    fail(
      `Release notes missing: ${relative(notesFile)}\n` +
        "A placeholder was generated from the commit log. Edit it, then re-run ship-release.",
    );
  }

  if (DRY_RUN) {
    log(`  [dry-run] Would run: gh release create ${TAG} --title "NEditor ${VERSION}" --notes-file ${relative(notesFile)}`);
    log(`  [dry-run] Artifacts (${releaseArtifacts.length}):`);
    for (const a of releaseArtifacts) log(`    ${relative(a)}`);
    releaseUrl = `https://github.com/(dry-run)/releases/tag/${TAG}`;
    return;
  }

  const ghArgs = ["release", "create", TAG, "--title", `NEditor ${VERSION}`, "--notes-file", notesFile, ...releaseArtifacts];
  log(`  gh ${ghArgs.join(" ")}`);
  const result = spawnSync("gh", ghArgs, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] });
  if (result.status !== 0) fail(`gh release create exited ${result.status}`);
  releaseUrl = result.stdout.trim();
  log(`  Release URL: ${releaseUrl}`);
});

// ─── Step 7: Homebrew (opt-in) ────────────────────────────────────────────────

let homebrewSha = "";

if (HOMEBREW) {
  step("homebrew", () => {
    const tapRepo = "nyimbi/homebrew-neditor";
    const tapToken = process.env.HOMEBREW_TAP_TOKEN ?? "";
    const ghEnv = tapToken ? { GH_TOKEN: tapToken } : {};

    // Create tap repo if it does not exist
    const repoCheck = spawnSync("gh", ["repo", "view", tapRepo, "--json", "name"], { encoding: "utf8", stdio: "pipe" });
    if (repoCheck.status !== 0) {
      if (DRY_RUN) {
        log(`  [dry-run] Would create: gh repo create ${tapRepo} --public`);
      } else {
        run("gh", ["repo", "create", tapRepo, "--public", "--description", "Homebrew tap for NEditor"], ghEnv);
      }
    }

    const tapDir = mkdtempSync(join(tmpdir(), "neditor-tap-"));

    if (DRY_RUN) {
      log(`  [dry-run] Would clone ${tapRepo} into temp dir`);
    } else {
      run("gh", ["repo", "clone", tapRepo, tapDir], ghEnv);
    }

    // Generate cask
    run("pnpm", ["run", "release:homebrew"]);

    const generatedCask = join(root, ".tmp/homebrew/external/neditor.rb");
    if (!DRY_RUN && !existsSync(generatedCask)) {
      fail(`Expected generated cask at ${relative(generatedCask)} but file is missing.`);
    }

    if (DRY_RUN) {
      log(`  [dry-run] Would copy cask to ${tapRepo}/Casks/neditor.rb and push.`);
      homebrewSha = "(dry-run)";
      return;
    }

    const caskDest = join(tapDir, "Casks/neditor.rb");
    mkdirSync(dirname(caskDest), { recursive: true });
    writeFileSync(caskDest, readFileSync(generatedCask, "utf8"));
    homebrewSha = sha256file(generatedCask);

    spawnSync("git", ["add", "Casks/neditor.rb"], { cwd: tapDir, stdio: "inherit" });
    const commit = spawnSync("git", ["commit", "-m", `chore: update neditor cask to ${TAG}`], { cwd: tapDir, stdio: "inherit" });
    if (commit.status !== 0) fail("git commit in homebrew tap failed.");
    const push = spawnSync("git", ["push"], { cwd: tapDir, stdio: "inherit" });
    if (push.status !== 0) fail("git push in homebrew tap failed.");

    log(`  Homebrew tap updated. Cask SHA256: ${homebrewSha}`);
  });
} else {
  log("[homebrew] Skipped (pass --homebrew to enable).\n");
}

// ─── Summary ─────────────────────────────────────────────────────────────────

log("\n" + "=".repeat(72));
log("RELEASE SUMMARY");
log("=".repeat(72));
log(`Version:     ${VERSION}`);
log(`Tag:         ${TAG}`);
log(`Release URL: ${releaseUrl || "(skipped — dry-run)"}`);
log(`Artifacts published: ${releaseArtifacts.length}`);
for (const a of releaseArtifacts) log(`  ${relative(a)}`);
if (homebrewSha) log(`Homebrew tap SHA256: ${homebrewSha}`);
log("\nStep timings:");
for (const { name, ms } of stepTimings) log(`  ${name.padEnd(14)} ${ms}ms`);
if (DRY_RUN) log("\n⚠  DRY-RUN: no tags, GitHub releases, or homebrew pushes were created.");
log("=".repeat(72));

// ─── Utilities ───────────────────────────────────────────────────────────────

function step(name, fn) {
  log(`\n── Step: ${name} ──`);
  const t0 = Date.now();
  fn();
  const ms = Date.now() - t0;
  stepTimings.push({ name, ms });
  log(`   done (${ms}ms)`);
}

/**
 * Spawn a command with inherited stdio, failing on non-zero exit.
 * In --dry-run mode the command is logged but not executed.
 * @param {string} command
 * @param {string[]} cmdArgs
 * @param {Record<string, string>} extraEnv
 */
function run(command, cmdArgs, extraEnv = {}) {
  const envPrefix = Object.keys(extraEnv).length ? `${Object.keys(extraEnv).join("=")}=… ` : "";
  log(`  + ${envPrefix}${[command, ...cmdArgs].join(" ")}`);
  if (DRY_RUN) { log("    (dry-run: skipped)"); return; }
  const result = spawnSync(command, cmdArgs, {
    cwd: root,
    env: { ...process.env, ...extraEnv },
    stdio: "inherit",
  });
  if (result.status !== 0) {
    fail(`Command failed (exit ${result.status}): ${command} ${cmdArgs.join(" ")}`);
  }
}

function git(gitArgs) {
  const r = spawnSync("git", gitArgs, { cwd: root, encoding: "utf8", stdio: "pipe" });
  if (r.status !== 0) fail(`git ${gitArgs.join(" ")} failed: ${r.stderr?.trim() || r.stdout?.trim()}`);
  return r.stdout;
}

function collectBundleArtifacts(dir, extensions) {
  if (!existsSync(dir)) { log(`  [collect] Bundle dir not found: ${relative(dir)}`); return; }
  for (const file of walkDir(dir)) {
    if (extensions.some((ext) => file.toLowerCase().endsWith(ext.toLowerCase()))) {
      releaseArtifacts.push(file);
      log(`  [collect] ${relative(file)}  (${Math.round(statSync(file).size / 1024)} KB)`);
    }
  }
}

function walkDir(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...walkDir(p));
    else out.push(p);
  }
  return out;
}

/** Find all paths that are directories ending in `ext` (like ".app"). */
function findDotAppDirs(dir) {
  const results = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) {
      if (name.endsWith(".app")) results.push(p);
      else results.push(...findDotAppDirs(p));
    }
  }
  return results;
}

function sha256file(filePath) {
  return createHash("sha256").update(readFileSync(filePath)).digest("hex");
}

function readJson(relPath) {
  return JSON.parse(readFileSync(join(root, relPath), "utf8"));
}

function cargoTomlVersion() {
  const content = readFileSync(join(root, "src-tauri/Cargo.toml"), "utf8");
  const m = content.match(/^version\s*=\s*"([^"]+)"/m);
  if (!m) fail("Could not parse version from src-tauri/Cargo.toml.");
  return m[1];
}

function latestPreviousTag(currentTag) {
  const r = spawnSync("git", ["tag", "--sort=-version:refname"], { cwd: root, encoding: "utf8", stdio: "pipe" });
  const tags = r.stdout.trim().split("\n").filter(Boolean);
  const idx = tags.indexOf(currentTag);
  if (idx === -1) return tags[0] ?? null;
  return tags[idx + 1] ?? null;
}

function relative(absPath) {
  return absPath.startsWith(root + "/") ? absPath.slice(root.length + 1) : absPath;
}

function log(msg) {
  process.stdout.write(`${msg}\n`);
}

function fail(msg) {
  process.stderr.write(`\nERROR: ${msg}\n`);
  process.exit(1);
}

function parseArgs(argv) {
  const parsed = {};
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (!arg.startsWith("--")) continue;
    const key = arg.slice(2);
    const next = argv[i + 1];
    if (!next || next.startsWith("--")) {
      parsed[key] = true;
    } else {
      parsed[key] = next;
      i++;
    }
  }
  return parsed;
}
