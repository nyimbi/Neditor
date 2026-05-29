import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const candidateDir = resolveCandidateDir();
const issues = [];
const warnings = [];
const requiredKinds = ["frontend:index", "native:app-binary", "native:ned-cli", "native:prepared-ned-sidecar"];

const manifestPath = join(candidateDir, "manifest.json");
const sumsPath = join(candidateDir, "SHA256SUMS");
const readmePath = join(candidateDir, "README.md");
const reportPath = join(candidateDir, "check-report.json");

const manifest = readCandidateJson(manifestPath);
const sumsText = readCandidateText(sumsPath);
const readmeText = readCandidateText(readmePath);

if (manifest) validateManifest(manifest);
if (manifest && sumsText !== null) validateSha256Sums(manifest, sumsText);
if (manifest && readmeText !== null) validateReadme(readmeText);
if (manifest) validatePreparedSidecar(manifest);

const report = {
  schema: "neditor.local-release-candidate-check.v1",
  generatedAt: new Date().toISOString(),
  status: issues.length ? "failed" : "passed",
  candidateDir: relativePath(candidateDir),
  manifestPath: relativePath(manifestPath),
  summary: {
    issues: issues.length,
    warnings: warnings.length,
    artifacts: Array.isArray(manifest?.artifacts) ? manifest.artifacts.length : 0,
    requiredKinds,
  },
  issues,
  warnings,
};

mkdirSync(candidateDir, { recursive: true });
writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);

if (issues.length) {
  console.error(`Release candidate check failed with ${issues.length} issue(s). Report: ${relativePath(reportPath)}`);
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

console.log(`Release candidate check passed. Report: ${relativePath(reportPath)}`);

function resolveCandidateDir() {
  const args = process.argv.slice(2);
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--dir" || arg === "--candidate-dir") {
      const value = args[index + 1];
      if (!value) {
        console.error(`${arg} requires a directory path`);
        process.exit(1);
      }
      return resolve(root, value);
    }
    if (arg.startsWith("--dir=")) return resolve(root, arg.slice("--dir=".length));
    if (arg.startsWith("--candidate-dir=")) return resolve(root, arg.slice("--candidate-dir=".length));
  }
  return resolve(process.env.NEDITOR_RELEASE_CANDIDATE_DIR || join(root, ".tmp", "release-candidate"));
}

function readCandidateJson(path) {
  const text = readCandidateText(path);
  if (text === null) return null;
  try {
    return JSON.parse(text);
  } catch (error) {
    issues.push(`${relativePath(path)} is not valid JSON: ${error instanceof Error ? error.message : String(error)}`);
    return null;
  }
}

function readCandidateText(path) {
  if (!existsSync(path)) {
    issues.push(`${relativePath(path)} is missing`);
    return null;
  }
  if (!statSync(path).isFile()) {
    issues.push(`${relativePath(path)} is not a file`);
    return null;
  }
  return readFileSync(path, "utf8");
}

function validateManifest(candidate) {
  if (candidate.schema !== "neditor.local-release-candidate.v1") {
    issues.push(`manifest schema must be neditor.local-release-candidate.v1, got ${JSON.stringify(candidate.schema)}`);
  }
  if (!candidate.product?.version) issues.push("manifest product.version is missing");
  if (!candidate.source?.commit) issues.push("manifest source.commit is missing");
  if (typeof candidate.releaseable !== "boolean") issues.push("manifest releaseable must be boolean");
  if (!Array.isArray(candidate.artifacts) || !candidate.artifacts.length) {
    issues.push("manifest artifacts must be a non-empty array");
    return;
  }

  const artifactPaths = new Set();
  const artifactKinds = new Set();
  for (const [index, artifact] of candidate.artifacts.entries()) {
    const label = artifact?.path || `artifact[${index}]`;
    if (!artifact || typeof artifact !== "object") {
      issues.push(`artifact[${index}] must be an object`);
      continue;
    }
    if (!artifact.kind) issues.push(`${label} is missing kind`);
    if (artifact.kind) artifactKinds.add(artifact.kind);
    if (!validRelativeArtifactPath(artifact.path)) {
      issues.push(`${label} must use a safe repository-relative path`);
      continue;
    }
    if (artifactPaths.has(artifact.path)) issues.push(`${artifact.path} is listed more than once`);
    artifactPaths.add(artifact.path);
    const absolute = resolve(root, artifact.path);
    if (!existsSync(absolute)) {
      issues.push(`${artifact.path} does not exist`);
      continue;
    }
    if (!statSync(absolute).isFile()) {
      issues.push(`${artifact.path} is not a file`);
      continue;
    }
    const size = statSync(absolute).size;
    if (artifact.size !== size) issues.push(`${artifact.path} size mismatch: manifest ${artifact.size}, actual ${size}`);
    const digest = sha256(absolute);
    if (artifact.sha256 !== digest) issues.push(`${artifact.path} SHA-256 mismatch: manifest ${artifact.sha256}, actual ${digest}`);
  }

  for (const kind of requiredKinds) {
    if (!artifactKinds.has(kind)) issues.push(`manifest is missing required artifact kind ${kind}`);
  }
}

function validateSha256Sums(candidate, text) {
  const expected = new Set((candidate.artifacts || []).map((artifact) => `${artifact.sha256}  ${artifact.path}`));
  const actual = new Set(
    text
      .split(/\r?\n/)
      .map((line) => line.trimEnd())
      .filter(Boolean),
  );
  for (const line of expected) {
    if (!actual.has(line)) issues.push(`SHA256SUMS is missing ${line}`);
  }
  for (const line of actual) {
    if (!expected.has(line)) issues.push(`SHA256SUMS has unexpected line ${line}`);
  }
}

function validateReadme(text) {
  if (!text.includes("Releaseable on this host:")) issues.push("README.md must state whether the candidate is releaseable on this host");
  if (!text.includes("SHA256SUMS")) issues.push("README.md must point reviewers to SHA256SUMS");
  if (!text.includes("## Artifacts")) issues.push("README.md must include an Artifacts section");
}

function validatePreparedSidecar(candidate) {
  const cli = candidate.artifacts?.find((artifact) => artifact.kind === "native:ned-cli");
  const sidecars = candidate.artifacts?.filter((artifact) => artifact.kind === "native:prepared-ned-sidecar") || [];
  if (!cli || !sidecars.length) return;
  for (const sidecar of sidecars) {
    if (sidecar.sha256 !== cli.sha256) issues.push(`Prepared ned sidecar hash must match native:ned-cli: ${sidecar.path}`);
  }
}

function validRelativeArtifactPath(path) {
  if (typeof path !== "string" || !path.trim()) return false;
  if (isAbsolute(path)) return false;
  const normalized = path.replace(/\\/g, "/");
  if (normalized.includes("\0")) return false;
  if (normalized.split("/").includes("..")) return false;
  return resolve(root, normalized).startsWith(`${root}/`);
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function relativePath(path) {
  const rendered = relative(root, path).replace(/\\/g, "/");
  return rendered || ".";
}
