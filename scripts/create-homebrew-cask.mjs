import { createHash } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, extname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = readJson("package.json");
const defaultTemplate = join(root, "packaging", "homebrew", "Casks", "neditor.rb.template");
const defaultEvidenceDir = join(root, ".tmp", "homebrew", "external");
const defaultOutput = join(defaultEvidenceDir, "neditor.rb");
const defaultReportPath = join(root, ".tmp", "homebrew", "materialize-cask-report.json");

const args = parseArgs(process.argv.slice(2));
const artifactPath = resolveRequiredPath(
  args.artifact || process.env.NEDITOR_HOMEBREW_ARTIFACT,
  "Usage: pnpm run release:homebrew -- --artifact /path/to/NEditor-<version>-macos.zip [--output /path/to/neditor.rb]",
);
const templatePath = resolve(args.template || process.env.NEDITOR_HOMEBREW_TEMPLATE || defaultTemplate);
const outputPath = resolve(args.output || process.env.NEDITOR_HOMEBREW_CASK || defaultOutput);
const evidenceDir = resolve(args.evidenceDir || process.env.NEDITOR_HOMEBREW_EVIDENCE_DIR || defaultEvidenceDir);
const version = String(args.version || process.env.NEDITOR_RELEASE_VERSION || packageJson.version || "").trim();
const copyArtifact = !args.noCopyArtifact;
const jsonOutput = Boolean(args.json);
const printOnly = Boolean(args.print);

if (printOnly && jsonOutput) fail("Use either --print or --json, not both.");
if (!version) fail("Could not determine release version from --version, NEDITOR_RELEASE_VERSION, or package.json.");
if (version !== packageJson.version) {
  fail(`Release version must match package.json version ${packageJson.version}. Found ${version}.`);
}
if (!existsSync(templatePath)) fail(`Homebrew cask template is missing: ${templatePath}`);
if (!existsSync(artifactPath)) fail(`Homebrew release artifact is missing: ${artifactPath}`);
const artifactStat = statSync(artifactPath);
if (!artifactStat.isFile()) fail(`Homebrew release artifact must be a file: ${artifactPath}`);

const artifactExtension = extname(artifactPath).toLowerCase();
if (![".zip", ".dmg"].includes(artifactExtension)) {
  fail(`Homebrew release artifact must be a .zip or .dmg file. Found ${artifactExtension || "(none)"}.`);
}

const artifactName = `NEditor-${version}-macos${artifactExtension}`;
const artifactSha256 = sha256File(artifactPath);
let caskSource = readFileSync(templatePath, "utf8")
  .replaceAll("__VERSION__", version)
  .replaceAll("__SHA256__", artifactSha256)
  .replaceAll(`NEditor-#{version}-macos.zip`, artifactName.replace(version, "#{version}"));

if (caskSource.includes("__VERSION__") || caskSource.includes("__SHA256__")) {
  fail(`Homebrew cask template still contains unreplaced placeholders: ${templatePath}`);
}
if (!caskSource.includes(`version "${version}"`)) {
  fail("Materialized Homebrew cask does not contain the expected version stanza.");
}
if (!caskSource.includes(`sha256 "${artifactSha256}"`)) {
  fail("Materialized Homebrew cask does not contain the computed artifact SHA-256.");
}

const copiedArtifactPath = copyArtifact ? join(evidenceDir, artifactName) : artifactPath;
const report = {
  schema: "neditor.homebrew-cask-materialization.v1",
  generatedAt: new Date().toISOString(),
  appVersion: packageJson.version,
  releaseVersion: version,
  template: relative(templatePath),
  cask: {
    outputPath: relative(outputPath),
    urlArtifactName: artifactName,
    token: "neditor",
  },
  artifact: {
    sourcePath: relative(artifactPath),
    evidencePath: relative(copiedArtifactPath),
    bytes: artifactStat.size,
    sha256: artifactSha256,
    copied: copyArtifact,
  },
  nextCommands: [
    `NEDITOR_HOMEBREW_CASK=${shellPath(outputPath)} NEDITOR_HOMEBREW_ARTIFACT=${shellPath(copiedArtifactPath)} pnpm run check:homebrew`,
    `brew audit --cask --new ${shellPath(outputPath)}`,
  ],
  releaseWarning:
    "This utility computes cask metadata only. Do not publish until macOS signing/notarization and release-readiness evidence are accepted.",
};

if (printOnly) {
  process.stdout.write(caskSource);
} else {
  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(outputPath, caskSource);
}

if (!printOnly && copyArtifact) {
  mkdirSync(evidenceDir, { recursive: true });
  if (resolve(artifactPath) !== resolve(copiedArtifactPath)) copyFileSync(artifactPath, copiedArtifactPath);
}

if (!printOnly) {
  mkdirSync(dirname(defaultReportPath), { recursive: true });
  writeFileSync(defaultReportPath, `${JSON.stringify(report, null, 2)}\n`);
}

if (jsonOutput) {
  console.log(JSON.stringify(report, null, 2));
} else if (!printOnly) {
  console.log(`Materialized Homebrew cask at ${relative(outputPath)} for ${artifactName}.`);
  console.log(`Computed SHA-256: ${artifactSha256}`);
  console.log(`Wrote ${relative(defaultReportPath)}.`);
}

function parseArgs(values) {
  const parsed = {};
  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (value === "--") continue;
    else if (value === "--artifact") parsed.artifact = requireNext(values, ++index, value);
    else if (value === "--output") parsed.output = requireNext(values, ++index, value);
    else if (value === "--template") parsed.template = requireNext(values, ++index, value);
    else if (value === "--version") parsed.version = requireNext(values, ++index, value);
    else if (value === "--evidence-dir") parsed.evidenceDir = requireNext(values, ++index, value);
    else if (value === "--no-copy-artifact") parsed.noCopyArtifact = true;
    else if (value === "--json") parsed.json = true;
    else if (value === "--print") parsed.print = true;
    else if (value === "--help" || value === "-h") {
      console.log(helpText());
      process.exit(0);
    } else {
      fail(`Unsupported release:homebrew option '${value}'.\n\n${helpText()}`);
    }
  }
  return parsed;
}

function requireNext(values, index, flag) {
  const next = values[index];
  if (!next || next.startsWith("--")) fail(`${flag} requires a value.`);
  return next;
}

function resolveRequiredPath(value, message) {
  if (!value || !String(value).trim()) fail(message);
  return resolve(String(value));
}

function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function readJson(relativePath) {
  return JSON.parse(readFileSync(join(root, relativePath), "utf8"));
}

function shellPath(path) {
  return `'${path.replaceAll("'", "'\\''")}'`;
}

function relative(path) {
  const resolved = resolve(path);
  return resolved.startsWith(root) ? resolved.slice(root.length + 1) : resolved;
}

function helpText() {
  return [
    "Usage: pnpm run release:homebrew -- --artifact /path/to/NEditor-<version>-macos.zip [options]",
    "",
    "Options:",
    "  --artifact <path>       Signed/notarized macOS zip or dmg used by the cask.",
    "  --output <path>         Destination cask path. Defaults to .tmp/homebrew/external/neditor.rb.",
    "  --template <path>       Cask template path. Defaults to packaging/homebrew/Casks/neditor.rb.template.",
    "  --version <version>     Release version. Must match package.json.",
    "  --evidence-dir <path>   Directory that receives the artifact copy.",
    "  --no-copy-artifact      Leave the artifact in place and only write the cask.",
    "  --print                 Print the materialized cask to stdout.",
    "  --json                  Print the materialization report as JSON.",
  ].join("\n");
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
