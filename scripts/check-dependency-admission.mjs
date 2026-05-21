import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const issues = [];
const allowedLicenseTokens = [
  "MIT",
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Zlib",
  "0BSD",
  "CC0-1.0",
];
const forbiddenLicenseTokens = [
  "GPL",
  "LGPL",
  "AGPL",
  "SSPL",
  "BUSL",
  "Commons Clause",
  "Proprietary source-available",
];

const packageJson = JSON.parse(readText("package.json"));
const cargoToml = readText("src-tauri/Cargo.toml");
const licenseText = readText("LICENSE");
const admission = readText("docs/dependency-admission.md");
const admittedRows = parseAdmissionRows(admission);

requireEqual(packageJson.license, "MIT", "package.json must keep NEditor under MIT");
requireIncludes(cargoToml, 'license = "MIT"', "src-tauri/Cargo.toml must keep NEditor under MIT");
requireIncludes(licenseText, "MIT License", "LICENSE must contain the MIT license text");

const manifestDependencies = [
  ...javascriptManifestDependencies(packageJson),
  ...rustManifestDependencies(cargoToml),
];

for (const dependency of manifestDependencies) {
  const row = admittedRows.get(dependency.name);
  if (!row) {
    issues.push(
      `${dependency.source} dependency ${dependency.name} is missing from docs/dependency-admission.md`,
    );
    continue;
  }
  if (!licenseExpectationIsPermissive(row.licenseExpectation)) {
    issues.push(
      `${dependency.name} admission row has unsupported license expectation: ${row.licenseExpectation}`,
    );
  }
}

for (const [name, row] of admittedRows) {
  if (!licenseExpectationIsPermissive(row.licenseExpectation)) {
    issues.push(`${name} admission row has unsupported license expectation: ${row.licenseExpectation}`);
  }
}

if (issues.length > 0) {
  console.error("Dependency admission check failed:");
  for (const issue of issues) {
    console.error(`- ${issue}`);
  }
  process.exit(1);
}

console.log(
  `Checked dependency admission records for ${manifestDependencies.length} manifest dependencies across package.json and src-tauri/Cargo.toml.`,
);

function readText(relativePath) {
  return readFileSync(join(root, relativePath), "utf8");
}

function requireEqual(actual, expected, message) {
  if (actual !== expected) {
    issues.push(`${message}: expected ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function requireIncludes(text, needle, message) {
  if (!text.includes(needle)) {
    issues.push(message);
  }
}

function javascriptManifestDependencies(manifest) {
  return [
    ...dependencyEntries(manifest.dependencies, "package.json dependencies"),
    ...dependencyEntries(manifest.devDependencies, "package.json devDependencies"),
    ...dependencyEntries(manifest.optionalDependencies, "package.json optionalDependencies"),
    ...dependencyEntries(manifest.peerDependencies, "package.json peerDependencies"),
  ];
}

function dependencyEntries(record, source) {
  return Object.keys(record || {})
    .sort()
    .map((name) => ({ name, source }));
}

function rustManifestDependencies(text) {
  const dependencies = [];
  let section = "";
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.replace(/#.*/, "").trim();
    if (!line) continue;
    const sectionMatch = line.match(/^\[([^\]]+)\]$/);
    if (sectionMatch) {
      section = sectionMatch[1];
      continue;
    }
    if (!["dependencies", "build-dependencies", "dev-dependencies"].includes(section)) {
      continue;
    }
    const dependencyMatch = line.match(/^([A-Za-z0-9_.-]+)\s*=/);
    if (!dependencyMatch) continue;
    dependencies.push({
      name: dependencyMatch[1],
      source: `src-tauri/Cargo.toml ${section}`,
    });
  }
  return dependencies.sort((left, right) => left.name.localeCompare(right.name));
}

function parseAdmissionRows(text) {
  const rows = new Map();
  for (const line of text.split(/\r?\n/)) {
    if (!line.startsWith("| `")) continue;
    const cells = line
      .split("|")
      .slice(1, -1)
      .map((cell) => cell.trim());
    if (cells.length < 3) continue;
    const name = cells[0].match(/^`([^`]+)`$/)?.[1];
    if (!name) continue;
    rows.set(name, {
      licenseExpectation: cells[2],
      line,
    });
  }
  return rows;
}

function licenseExpectationIsPermissive(text) {
  if (forbiddenLicenseTokens.some((token) => text.includes(token))) {
    return false;
  }
  return allowedLicenseTokens.some((token) => text.includes(token));
}
