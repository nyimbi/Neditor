import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const issues = [];

if (process.platform !== "darwin") {
  console.log("Desktop bundle smoke currently verifies macOS .app bundles; skipping on this host.");
  process.exit(0);
}

const tauriConfig = readJson("src-tauri/tauri.conf.json");
const appRoot = join(root, "src-tauri", "target", "release", "bundle", "macos", "NEditor.app");
const plistPath = join(appRoot, "Contents", "Info.plist");
const executablePath = join(appRoot, "Contents", "MacOS", "neditor");
const nedPath = firstExistingPath([
  join(appRoot, "Contents", "MacOS", "ned"),
  join(appRoot, "Contents", "Resources", "ned"),
  join(appRoot, "Contents", "Resources", "binaries", "ned"),
]);
const iconPath = join(appRoot, "Contents", "Resources", "icon.icns");
const showcasePath = join(appRoot, "Contents", "Resources", "examples", "showcase", "neditor-capability-showcase.md");

requireDirectory(appRoot, "macOS app bundle is missing; run ./node_modules/.bin/tauri build --bundles app first");
requireFile(plistPath, "macOS app bundle Info.plist is missing");
requireExecutable(executablePath, "macOS app bundle executable is missing or not executable");
requireExecutable(nedPath, "macOS app bundle ned CLI sidecar is missing or not executable");
requireFile(iconPath, "macOS app bundle icon is missing");
requireFile(showcasePath, "macOS app bundle capability showcase example is missing");
requireMinimumSize(executablePath, 1_000_000, "macOS app bundle executable is unexpectedly small");
requireMinimumSize(nedPath, 100_000, "macOS app bundle ned CLI sidecar is unexpectedly small");
requireMinimumSize(iconPath, 1_000, "macOS app bundle icon is unexpectedly small");
requireMinimumSize(showcasePath, 10_000, "macOS app bundle capability showcase example is unexpectedly small");

let plist = {};
if (issues.length === 0) {
  plist = readPlist(plistPath);
  requireEqual(plist.CFBundleDisplayName, tauriConfig.productName, "CFBundleDisplayName must match Tauri productName");
  requireEqual(plist.CFBundleName, tauriConfig.productName, "CFBundleName must match Tauri productName");
  requireEqual(plist.CFBundleExecutable, "neditor", "CFBundleExecutable must point at the bundled binary");
  requireEqual(plist.CFBundleIdentifier, tauriConfig.identifier, "CFBundleIdentifier must match Tauri identifier");
  requireEqual(plist.CFBundleShortVersionString, tauriConfig.version, "CFBundleShortVersionString must match Tauri version");
  requireEqual(plist.CFBundleVersion, tauriConfig.version, "CFBundleVersion must match Tauri version");
  requireEqual(plist.CFBundlePackageType, "APPL", "CFBundlePackageType must be APPL");
  requireEqual(plist.CFBundleIconFile, "icon.icns", "CFBundleIconFile must reference the packaged icon");
  requireEqual(
    plist.NSHumanReadableCopyright,
    tauriConfig.bundle?.copyright,
    "NSHumanReadableCopyright must match Tauri bundle copyright",
  );
  requireEqual(plist.NSHighResolutionCapable, true, "macOS bundle must declare high-resolution support");
}

if (issues.length > 0) {
  console.error("Desktop bundle smoke failed:");
  for (const issue of issues) {
    console.error(`- ${issue}`);
  }
  process.exit(1);
}

writeBundleReport(plist);
console.log("Checked NEditor macOS .app bundle metadata, executable, and icon.");

function readJson(relativePath) {
  return JSON.parse(readFileSync(join(root, relativePath), "utf8"));
}

function readPlist(path) {
  const result = spawnSync("plutil", ["-convert", "json", "-o", "-", path], {
    encoding: "utf8",
  });
  if (result.status !== 0) {
    const detail = [result.stdout?.trim(), result.stderr?.trim()].filter(Boolean).join("\n");
    issues.push(`failed to parse Info.plist with plutil${detail ? `:\n${detail}` : ""}`);
    return {};
  }
  return JSON.parse(result.stdout);
}

function requireDirectory(path, message) {
  if (!existsSync(path) || !statSync(path).isDirectory()) {
    issues.push(message);
  }
}

function requireFile(path, message) {
  if (!existsSync(path) || !statSync(path).isFile()) {
    issues.push(message);
  }
}

function requireExecutable(path, message) {
  if (!existsSync(path) || !statSync(path).isFile() || (statSync(path).mode & 0o111) === 0) {
    issues.push(message);
  }
}

function requireMinimumSize(path, minimumBytes, message) {
  if (existsSync(path) && statSync(path).size < minimumBytes) {
    issues.push(`${message}: ${statSync(path).size} bytes`);
  }
}

function firstExistingPath(paths) {
  return paths.find((path) => existsSync(path)) || paths[0];
}

function requireEqual(actual, expected, message) {
  if (actual !== expected) {
    issues.push(`${message}: expected ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function writeBundleReport(plist) {
  const directory = join(root, ".tmp", "desktop-bundle");
  mkdirSync(directory, { recursive: true });
  writeFileSync(
    join(directory, "macos-app-report.json"),
    `${JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        appBundle: relative(appRoot),
        executable: {
          path: relative(executablePath),
          size: statSync(executablePath).size,
          mode: `0${(statSync(executablePath).mode & 0o777).toString(8)}`,
        },
        cli: {
          path: relative(nedPath),
          size: statSync(nedPath).size,
          mode: `0${(statSync(nedPath).mode & 0o777).toString(8)}`,
        },
        icon: {
          path: relative(iconPath),
          size: statSync(iconPath).size,
        },
        showcase: {
          path: relative(showcasePath),
          size: statSync(showcasePath).size,
        },
        plist: {
          CFBundleDisplayName: plist.CFBundleDisplayName,
          CFBundleExecutable: plist.CFBundleExecutable,
          CFBundleIdentifier: plist.CFBundleIdentifier,
          CFBundleShortVersionString: plist.CFBundleShortVersionString,
          CFBundleVersion: plist.CFBundleVersion,
          CFBundlePackageType: plist.CFBundlePackageType,
          CFBundleIconFile: plist.CFBundleIconFile,
          NSHumanReadableCopyright: plist.NSHumanReadableCopyright,
          NSHighResolutionCapable: plist.NSHighResolutionCapable,
        },
      },
      null,
      2,
    )}\n`,
  );
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
