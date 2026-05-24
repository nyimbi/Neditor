import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputDir = join(root, ".tmp", "ai-provider-evidence");
const reportPath = join(outputDir, "report.json");
const templatesDir = join(outputDir, "templates");
const defaultEvidenceDir = join(outputDir, "external");
const evidenceDir = resolve(process.env.NEDITOR_AI_PROVIDER_EVIDENCE_DIR || defaultEvidenceDir);
const explicitEvidence = process.env.NEDITOR_AI_PROVIDER_EVIDENCE ? [resolve(process.env.NEDITOR_AI_PROVIDER_EVIDENCE)] : [];
const currentSourceCommit = gitCommit();
const currentSourceTreeClean = gitTreeClean();
const requiredMarker = "NEDITOR_PROVIDER_EVIDENCE_OK";
const supportedProfiles = new Set(["openai-compatible", "anthropic-compatible", "gemini-compatible", "local-http"]);
const issues = [];

mkdirSync(outputDir, { recursive: true });
mkdirSync(templatesDir, { recursive: true });
writeTemplate();

const evidenceFiles = [...explicitEvidence, ...discoverEvidenceFiles(evidenceDir)];
const evidence = evidenceFiles.map((path) => validateEvidenceFile(path));
const accepted = evidence.filter((item) => item.status === "accepted");
const invalid = evidence.filter((item) => item.status === "invalid");
const status = invalid.length > 0 ? "failed" : accepted.length > 0 ? "accepted" : "pending-live-provider-evidence";

writeReport(status, evidence, invalid);

if (invalid.length > 0) {
  console.error("AI provider evidence failed validation:");
  for (const item of invalid) {
    console.error(`- ${relative(item.path)}: ${item.issues.join("; ")}`);
  }
  process.exit(1);
}

console.log(`AI provider evidence is ${status}; wrote ${relative(reportPath)}.`);

function validateEvidenceFile(path) {
  const itemIssues = [];
  if (!existsSync(path)) {
    return {
      path,
      status: "invalid",
      issues: ["evidence file is missing"],
    };
  }

  let evidence;
  try {
    evidence = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    return {
      path,
      status: "invalid",
      issues: [`evidence is not valid JSON: ${String(error)}`],
    };
  }

  requireValue(evidence.schema === "neditor.ai-provider-evidence.v1", "schema must be neditor.ai-provider-evidence.v1", itemIssues);
  requireValue(evidence.appVersion === packageJson.version, `appVersion must match package.json version ${packageJson.version}`, itemIssues);
  requireValue(evidence.sourceCommit === currentSourceCommit, `sourceCommit must match current git commit ${currentSourceCommit}`, itemIssues);
  requireValue(evidence.sourceTreeClean === true, "sourceTreeClean must be true when evidence is collected", itemIssues);
  requireValue(evidence.status === "passed", "status must be passed", itemIssues);
  requireValue(isIsoDate(evidence.generatedAt), "generatedAt must be an ISO timestamp", itemIssues);
  requireValue(supportedProfiles.has(evidence.providerProfile), `providerProfile must be one of ${[...supportedProfiles].join(", ")}`, itemIssues);
  requireValue(typeof evidence.endpointHost === "string" && evidence.endpointHost.length > 0, "endpointHost is required", itemIssues);
  requireValue(!String(evidence.endpointHost).includes("@"), "endpointHost must not include credentials", itemIssues);
  requireValue(typeof evidence.model === "string" && evidence.model.length > 0, "model is required", itemIssues);
  requireValue(evidence.secretMaterialStored === false, "secretMaterialStored must be false", itemIssues);
  if (evidence.providerProfile === "local-http") {
    requireValue(evidence.request?.apiKeyEnv === null, "local-http evidence must not record an API key environment variable", itemIssues);
  } else {
    requireValue(evidence.request?.apiKeyEnv && !String(evidence.request.apiKeyEnv).includes("sk-"), "request.apiKeyEnv must name an environment variable, not a secret", itemIssues);
  }
  requireValue(isIsoDate(evidence.request?.startedAt), "request.startedAt must be an ISO timestamp", itemIssues);
  requireValue(isSha256(evidence.request?.promptSha256), "request.promptSha256 must be a SHA-256 hex digest", itemIssues);
  requireValue(Number(evidence.response?.httpStatus || 0) >= 200 && Number(evidence.response?.httpStatus || 0) < 300, "response.httpStatus must be 2xx", itemIssues);
  requireValue(isIsoDate(evidence.response?.finishedAt), "response.finishedAt must be an ISO timestamp", itemIssues);
  requireValue(isSha256(evidence.response?.rawSha256), "response.rawSha256 must be a SHA-256 hex digest", itemIssues);
  requireValue(isSha256(evidence.response?.extractedTextSha256), "response.extractedTextSha256 must be a SHA-256 hex digest", itemIssues);
  const markers = Array.isArray(evidence.response?.markers) ? evidence.response.markers : [];
  requireValue(markers.includes(requiredMarker), `response.markers must include ${requiredMarker}`, itemIssues);
  const preview = String(evidence.response?.preview || "");
  requireValue(preview.length <= 800, "response.preview must be bounded to 800 characters", itemIssues);
  requireValue(!/sk-[A-Za-z0-9_-]{12,}/.test(JSON.stringify(evidence)), "evidence must not contain API key-looking secrets", itemIssues);

  return {
    path,
    status: itemIssues.length === 0 ? "accepted" : "invalid",
    providerProfile: evidence.providerProfile || null,
    endpointHost: evidence.endpointHost || null,
    model: evidence.model || null,
    generatedAt: evidence.generatedAt || null,
    issues: itemIssues,
  };
}

function discoverEvidenceFiles(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true })
    .flatMap((entry) => {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) return discoverEvidenceFiles(path);
      return entry.isFile() && entry.name.endsWith(".json") ? [path] : [];
    })
    .sort();
}

function writeTemplate() {
  const templatePath = join(templatesDir, "provider-evidence.template.json");
  writeFileSync(
    templatePath,
    `${JSON.stringify(
      {
        schema: "neditor.ai-provider-evidence.v1",
        generatedAt: new Date().toISOString(),
        status: "passed",
        appVersion: packageJson.version,
        sourceCommit: currentSourceCommit,
        sourceTreeClean: currentSourceTreeClean,
        providerProfile: "openai-compatible",
        endpointHost: "api.provider.example",
        model: "approved-model-name",
        secretMaterialStored: false,
        request: {
          startedAt: new Date().toISOString(),
          apiKeyEnv: "PROVIDER_API_KEY",
          promptSha256: sha256("Replace with the prompt sent to the provider."),
          bodyShape: "chat-completions-compatible",
        },
        response: {
          finishedAt: new Date().toISOString(),
          httpStatus: 200,
          rawSha256: "replace-with-64-character-sha256",
          extractedTextSha256: "replace-with-64-character-sha256",
          markers: [requiredMarker],
          preview: "Bounded provider response preview without secrets.",
        },
      },
      null,
      2,
    )}\n`,
  );
}

function writeReport(status, evidence, invalid) {
  const templatePath = join(templatesDir, "provider-evidence.template.json");
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        schema: "neditor.ai-provider-evidence-report.v1",
        generatedAt: new Date().toISOString(),
        status,
        appVersion: packageJson.version,
        sourceCommit: currentSourceCommit,
        sourceTreeClean: currentSourceTreeClean,
        evidenceDir: relative(evidenceDir),
        template: {
          path: relative(templatePath),
          bytes: statSync(templatePath).size,
        },
        summary: {
          acceptedEvidence: evidence.filter((item) => item.status === "accepted").length,
          invalidEvidence: invalid.length,
          discoveredEvidence: evidence.length,
        },
        requiredMarker,
        acceptedProfiles: evidence.filter((item) => item.status === "accepted").map((item) => ({
          providerProfile: item.providerProfile,
          endpointHost: item.endpointHost,
          model: item.model,
          generatedAt: item.generatedAt,
          path: relative(item.path),
        })),
        evidence: evidence.map((item) => ({
          ...item,
          path: relative(item.path),
        })),
        issues,
      },
      null,
      2,
    )}\n`,
  );
}

function requireValue(condition, issue, target) {
  if (!condition) target.push(issue);
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function isSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/i.test(value);
}

function sha256(value) {
  return createHash("sha256").update(String(value)).digest("hex");
}

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0) return "";
  return result.stdout.trim();
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 && result.stdout.trim() === "";
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
