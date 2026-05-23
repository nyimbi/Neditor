import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = join(root, ".tmp", "desktop-bundle", "platform-package-config-report.json");
const packageJson = readJson("package.json");
const tauriConfig = readJson("src-tauri/tauri.conf.json");
const cargoToml = readText("src-tauri/Cargo.toml");
const licenseText = readText("LICENSE");
const issues = [];

const cargoPackage = {
  name: tomlScalar(cargoToml, "package", "name"),
  version: tomlScalar(cargoToml, "package", "version"),
  license: tomlScalar(cargoToml, "package", "license"),
  description: tomlScalar(cargoToml, "package", "description"),
};
const bundle = tauriConfig.bundle || {};
const windowConfig = tauriConfig.app?.windows?.[0] || {};
const iconEvidence = collectIconEvidence(bundle.icon || []);
const targetEvidence = classifyBundleTargets(bundle.targets);
const signing = signingEvidence();

requireEqual(packageJson.name, cargoPackage.name, "npm and Cargo package names must match");
requireEqual(packageJson.version, cargoPackage.version, "npm and Cargo package versions must match");
requireEqual(packageJson.license, "MIT", "npm package license must remain MIT");
requireEqual(cargoPackage.license, "MIT", "Cargo package license must remain MIT");
requireEqual(tauriConfig.productName, "NEditor", "Tauri product name must remain NEditor");
requireEqual(tauriConfig.version, packageJson.version, "Tauri version must match package.json");
requireEqual(tauriConfig.identifier, "com.neditor.desktop", "Tauri bundle identifier must remain stable");
requireEqual(bundle.active, true, "Tauri bundling must remain active");
if (!targetEvidence.allTargets) {
  issues.push(`Tauri bundle targets must remain all-platform; found ${JSON.stringify(bundle.targets)}`);
}
requireEqual(bundle.license, "MIT", "Tauri bundle license must remain MIT");
requireEqual(bundle.licenseFile, "../LICENSE", "Tauri bundle licenseFile must point to root LICENSE");
if (!licenseText.includes("MIT License") || !licenseText.includes("Permission is hereby granted")) {
  issues.push("root LICENSE does not contain the canonical MIT license text");
}
if (!cargoPackage.description || !cargoPackage.description.includes("local-first")) {
  issues.push("Cargo package description must communicate local-first positioning");
}
if (!String(bundle.copyright || "").includes("2026")) {
  issues.push("Tauri bundle copyright must include the release year");
}
for (const requiredKind of ["png", "icns", "ico", "windowsTilePng"]) {
  if (!iconEvidence.kinds.includes(requiredKind)) {
    issues.push(`Tauri icon set is missing required ${requiredKind} evidence`);
  }
}
if (windowConfig.width < 1200 || windowConfig.height < 800 || windowConfig.minWidth < 900 || windowConfig.minHeight < 600) {
  issues.push("Tauri main window dimensions must preserve a production desktop workbench footprint");
}
const csp = String(tauriConfig.app?.security?.csp || "");
for (const token of ["default-src 'self'", "object-src 'none'", "frame-ancestors 'none'", "connect-src 'self' ipc:"]) {
  if (!csp.includes(token)) issues.push(`Tauri CSP is missing ${token}`);
}

writeReport();

if (issues.length > 0) {
  console.error("Platform package configuration audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

console.log(`Checked cross-platform package configuration and signing stance in ${relative(reportPath)}.`);

function collectIconEvidence(iconEntries) {
  const entries = iconEntries.map((entry) => {
    const path = join(root, "src-tauri", entry);
    const exists = existsSync(path);
    return {
      path: `src-tauri/${entry}`,
      exists,
      size: exists ? statSync(path).size : 0,
      kind: iconKind(entry),
    };
  });
  const windowsTileEntries = [
    "Square30x30Logo.png",
    "Square44x44Logo.png",
    "Square71x71Logo.png",
    "Square89x89Logo.png",
    "Square107x107Logo.png",
    "Square142x142Logo.png",
    "Square150x150Logo.png",
    "Square284x284Logo.png",
    "Square310x310Logo.png",
    "StoreLogo.png",
  ].map((entry) => {
    const path = join(root, "src-tauri", "icons", entry);
    const exists = existsSync(path);
    return {
      path: `src-tauri/icons/${entry}`,
      exists,
      size: exists ? statSync(path).size : 0,
      kind: "windowsTilePng",
    };
  });
  for (const entry of [...entries, ...windowsTileEntries]) {
    if (!entry.exists) issues.push(`packaging icon is missing: ${entry.path}`);
    if (entry.exists && entry.size < 500) issues.push(`packaging icon is unexpectedly small: ${entry.path}`);
  }
  const allEntries = [...entries, ...windowsTileEntries];
  return {
    entries: allEntries,
    kinds: [...new Set(allEntries.filter((entry) => entry.exists).map((entry) => entry.kind))].sort(),
  };
}

function classifyBundleTargets(targets) {
  const explicit = Array.isArray(targets) ? targets : [targets].filter(Boolean);
  return {
    configured: targets,
    allTargets: targets === "all" || explicit.includes("all"),
    impliedPlatforms: ["macos", "windows", "linux"],
    expectedHostArtifacts: {
      macos: ["app", "dmg"],
      windows: ["msi", "nsis", "exe"],
      linux: ["appimage", "deb", "rpm"],
    },
  };
}

function signingEvidence() {
  return {
    status: "unsigned-local-builds",
    reason:
      "Local verification proves package configuration and unsigned artifacts. Distribution signing, notarization, and installer attestation require release credentials outside this repository.",
    macos: {
      signingIdentityConfigured: Boolean(process.env.APPLE_SIGNING_IDENTITY || process.env.APPLE_CERTIFICATE),
      notarizationConfigured: Boolean(process.env.APPLE_API_KEY || process.env.APPLE_ID),
    },
    windows: {
      certificateConfigured: Boolean(process.env.WINDOWS_CERTIFICATE || process.env.WINDOWS_CODESIGN_CERTIFICATE),
      timestampServerConfigured: Boolean(process.env.WINDOWS_TIMESTAMP_URL),
    },
    linux: {
      packageSigningConfigured: Boolean(process.env.LINUX_PACKAGE_SIGNING_KEY || process.env.GPG_SIGNING_KEY),
    },
  };
}

function iconKind(entry) {
  if (entry.endsWith(".icns")) return "icns";
  if (entry.endsWith(".ico")) return "ico";
  if (entry.endsWith(".png")) return "png";
  return "unknown";
}

function readJson(relativePath) {
  return JSON.parse(readText(relativePath));
}

function readText(relativePath) {
  return readFileSync(join(root, relativePath), "utf8");
}

function tomlScalar(toml, section, key) {
  const sectionMatch = toml.match(new RegExp(`\\[${escapeRegExp(section)}\\]([\\s\\S]*?)(?:\\n\\[|$)`));
  const body = sectionMatch?.[1] || "";
  const scalarMatch = body.match(new RegExp(`^${escapeRegExp(key)}\\s*=\\s*"([^"]*)"`, "m"));
  return scalarMatch?.[1] || "";
}

function requireEqual(actual, expected, message) {
  if (actual !== expected) {
    issues.push(`${message}: expected ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function writeReport() {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        platform: process.platform,
        arch: process.arch,
        status: issues.length === 0 ? "passed" : "failed",
        product: {
          npmName: packageJson.name,
          cargoName: cargoPackage.name,
          tauriProductName: tauriConfig.productName,
          version: packageJson.version,
          identifier: tauriConfig.identifier,
          license: packageJson.license,
          cargoLicense: cargoPackage.license,
          bundleLicense: bundle.license,
          licenseFile: bundle.licenseFile,
        },
        bundle: {
          active: bundle.active,
          targets: targetEvidence,
          copyright: bundle.copyright,
        },
        mainWindow: {
          title: windowConfig.title,
          width: windowConfig.width,
          height: windowConfig.height,
          minWidth: windowConfig.minWidth,
          minHeight: windowConfig.minHeight,
        },
        security: {
          cspTokens: {
            defaultSelf: cspIncludes("default-src 'self'"),
            objectNone: cspIncludes("object-src 'none'"),
            frameAncestorsNone: cspIncludes("frame-ancestors 'none'"),
            ipcConnect: cspIncludes("connect-src 'self' ipc:"),
          },
        },
        icons: iconEvidence,
        signing,
        issues,
      },
      null,
      2,
    )}\n`,
  );
}

function cspIncludes(token) {
  return String(tauriConfig.app?.security?.csp || "").includes(token);
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
