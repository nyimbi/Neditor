import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const evidenceDir = resolve(process.env.NEDITOR_RELEASE_SIGNING_DIR || join(root, ".tmp", "release-signing", "external"));
const reportPath = join(root, ".tmp", "release-signing", "report.json");
const templateDir = join(root, ".tmp", "release-signing", "templates");

const signingSpecs = [
  {
    platform: "darwin",
    name: "macOS",
    evidencePath: "darwin/signing-evidence.json",
    requiredProof: ["codesign", "notarization", "spctl"],
    artifactKinds: ["app", "dmg"],
  },
  {
    platform: "win32",
    name: "Windows",
    evidencePath: "win32/signing-evidence.json",
    requiredProof: ["authenticode", "timestamp"],
    artifactKinds: ["msi", "nsis", "exe"],
  },
  {
    platform: "linux",
    name: "Linux",
    evidencePath: "linux/signing-evidence.json",
    requiredProof: ["package-signature", "checksum"],
    artifactKinds: ["appimage", "deb", "rpm"],
  },
];

mkdirSync(templateDir, { recursive: true });
writeTemplates();

const platforms = signingSpecs.map((spec) => evaluateSigningEvidence(spec));
const missingItems = platforms.filter((platform) => platform.status === "missing");
const invalidItems = platforms.filter((platform) => platform.status === "invalid");
const status = invalidItems.length > 0 ? "failed" : missingItems.length > 0 ? "pending-release-credentials" : "complete";

writeReport({
  generatedAt: new Date().toISOString(),
  platform: process.platform,
  arch: process.arch,
  status,
  evidenceDir: relative(evidenceDir),
  templateDir: relative(templateDir),
  summary: {
    requiredPlatforms: signingSpecs.length,
    completePlatforms: platforms.filter((platform) => platform.status === "accepted").length,
    missingEvidence: missingItems.length,
    invalidEvidence: invalidItems.length,
  },
  platforms,
  missingEvidence: missingItems.map((item) => ({
    platform: item.platform,
    path: item.path,
    detail: item.detail,
  })),
  invalidEvidence: invalidItems.map((item) => ({
    platform: item.platform,
    path: item.path,
    detail: item.detail,
  })),
});

if (invalidItems.length > 0) {
  console.error("Release signing evidence failed validation:");
  for (const issue of invalidItems) console.error(`- ${issue.detail}`);
  console.error(`Wrote ${relative(reportPath)}.`);
  process.exit(1);
}

console.log(`Release signing evidence is ${status}; wrote ${relative(reportPath)}.`);

function evaluateSigningEvidence(spec) {
  const path = join(evidenceDir, spec.evidencePath);
  if (!existsSync(path)) {
    return {
      platform: spec.platform,
      name: spec.name,
      path: spec.evidencePath,
      status: "missing",
      detail: `${spec.name} release signing/notarization evidence has not been supplied.`,
    };
  }

  let evidence;
  try {
    evidence = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    return invalid(spec, `${spec.evidencePath} is not valid JSON: ${error.message}`);
  }

  const problems = [];
  requireValue(evidence.schema === "neditor.release-signing-evidence.v1", problems, "schema must be neditor.release-signing-evidence.v1");
  requireValue(evidence.platform === spec.platform, problems, `platform must be ${spec.platform}`);
  requireValue(evidence.status === "passed", problems, "status must be passed");
  requireValue(isIsoDate(evidence.generatedAt), problems, "generatedAt must be an ISO timestamp");
  requireValue(Boolean(String(evidence.releaseVersion || "").trim()), problems, "releaseVersion is required");
  const artifacts = Array.isArray(evidence.artifacts) ? evidence.artifacts : [];
  requireValue(artifacts.length > 0, problems, "artifacts must include at least one signed release artifact");
  for (const artifact of artifacts) {
    requireValue(spec.artifactKinds.includes(String(artifact.kind || "").toLowerCase()), problems, `artifact kind must be one of ${spec.artifactKinds.join(", ")}`);
    requireValue(Boolean(String(artifact.path || "").trim()), problems, "artifact path is required");
    requireValue(Number(artifact.bytes) > 1000, problems, `artifact ${artifact.path || "(unknown)"} must record bytes > 1000`);
    requireValue(isSha256(artifact.sha256), problems, `artifact ${artifact.path || "(unknown)"} must record a sha256`);
  }
  const proof = Array.isArray(evidence.proof) ? evidence.proof : [];
  const proofKinds = new Set(proof.map((item) => item.kind));
  for (const kind of spec.requiredProof) {
    requireValue(proofKinds.has(kind), problems, `proof must include ${kind}`);
  }
  for (const item of proof) {
    requireValue(item.status === "passed", problems, `proof ${item.kind || "(unknown)"} must have status passed`);
    requireValue(Boolean(String(item.command || "").trim()), problems, `proof ${item.kind || "(unknown)"} command is required`);
    requireValue(Boolean(String(item.summary || "").trim()), problems, `proof ${item.kind || "(unknown)"} summary is required`);
  }

  if (problems.length > 0) {
    return invalid(spec, `${spec.name} release signing evidence is invalid: ${problems.join("; ")}`);
  }

  return {
    platform: spec.platform,
    name: spec.name,
    path: spec.evidencePath,
    status: "accepted",
    detail: `${spec.name} release signing evidence supplied with ${proof.length} proof checks.`,
    generatedAt: evidence.generatedAt,
    releaseVersion: evidence.releaseVersion,
    artifacts: artifacts.map((artifact) => ({
      kind: artifact.kind,
      path: artifact.path,
      bytes: artifact.bytes,
      sha256: artifact.sha256,
    })),
    proof: proof.map((item) => ({
      kind: item.kind,
      command: item.command,
      status: item.status,
      summary: item.summary,
    })),
  };
}

function invalid(spec, detail) {
  return {
    platform: spec.platform,
    name: spec.name,
    path: spec.evidencePath,
    status: "invalid",
    detail,
  };
}

function requireValue(condition, problems, message) {
  if (!condition) problems.push(message);
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function isSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/i.test(value);
}

function writeTemplates() {
  for (const spec of signingSpecs) {
    const templatePath = join(templateDir, `${spec.platform}-signing-evidence.template.json`);
    writeFileSync(
      templatePath,
      `${JSON.stringify(
        {
          schema: "neditor.release-signing-evidence.v1",
          platform: spec.platform,
          status: "passed",
          generatedAt: new Date().toISOString(),
          releaseVersion: "0.1.0",
          artifacts: [
            {
              kind: spec.artifactKinds[0],
              path: `src-tauri/target/release/bundle/${spec.name.toLowerCase()}/NEditor-placeholder`,
              bytes: 123456,
              sha256: "replace-with-64-character-sha256",
            },
          ],
          proof: spec.requiredProof.map((kind) => ({
            kind,
            status: "passed",
            command: commandHint(spec.platform, kind),
            summary: "Replace this with the verifier output summary from the release host.",
          })),
          notes: "Fill this from the credentialed release host after signing and verifying the distribution artifacts.",
        },
        null,
        2,
      )}\n`,
    );
  }
}

function commandHint(platform, kind) {
  const hints = {
    "darwin:codesign": "codesign --verify --deep --strict --verbose=2 NEditor.app",
    "darwin:notarization": "xcrun notarytool history --team-id TEAMID",
    "darwin:spctl": "spctl --assess --type execute --verbose NEditor.app",
    "win32:authenticode": "powershell Get-AuthenticodeSignature NEditor.exe",
    "win32:timestamp": "signtool verify /pa /tw NEditor.exe",
    "linux:package-signature": "gpg --verify NEditor.deb.sig NEditor.deb",
    "linux:checksum": "sha256sum --check NEditor.sha256",
  };
  return hints[`${platform}:${kind}`] || "replace-with-verification-command";
}

function writeReport(report) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
