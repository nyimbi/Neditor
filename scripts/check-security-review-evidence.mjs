import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputDir = join(root, ".tmp", "security-review");
const reportPath = join(outputDir, "report.json");
const templatesDir = join(outputDir, "templates");
const defaultEvidenceDir = join(outputDir, "external");
const evidenceDir = resolve(process.env.NEDITOR_SECURITY_REVIEW_EVIDENCE_DIR || defaultEvidenceDir);
const explicitEvidence = process.env.NEDITOR_SECURITY_REVIEW_EVIDENCE
  ? [resolve(process.env.NEDITOR_SECURITY_REVIEW_EVIDENCE)]
  : [];
const currentSourceCommit = gitCommit();
const currentSourceTreeClean = gitTreeClean();
const requiredBoundaryIds = [
  "tauri-command-boundary",
  "filesystem-boundary",
  "snapshot-boundary",
  "include-boundary",
  "export-boundary",
  "git-boundary",
  "external-transform-boundary",
  "ai-provider-boundary",
  "persistence-boundary",
];
const requiredArtifactIds = [
  "security-threat-model",
  "tauri-config",
  "rust-command-surface",
  "external-transform-runner",
  "git-restore-and-tag",
  "snapshot-restore",
  "ai-provider-packages",
  "workspace-persistence",
  "release-evidence-contracts",
];

mkdirSync(outputDir, { recursive: true });
mkdirSync(templatesDir, { recursive: true });
writeTemplate();

const evidenceFiles = [...explicitEvidence, ...discoverEvidenceFiles(evidenceDir)];
const evidence = evidenceFiles.map((path) => validateEvidenceFile(path));
const accepted = evidence.filter((item) => item.status === "accepted");
const invalid = evidence.filter((item) => item.status === "invalid");
const status = invalid.length > 0 ? "failed" : accepted.length > 0 ? "accepted" : "pending-independent-security-review";

writeReport(status, evidence, invalid);

if (invalid.length > 0) {
  console.error("Security review evidence failed validation:");
  for (const item of invalid) {
    console.error(`- ${relative(item.path)}: ${item.issues.join("; ")}`);
  }
  process.exit(1);
}

console.log(`Security review evidence is ${status}; wrote ${relative(reportPath)}.`);

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

  requireValue(evidence.schema === "neditor.security-review-evidence.v1", "schema must be neditor.security-review-evidence.v1", itemIssues);
  requireValue(evidence.appVersion === packageJson.version, `appVersion must match package.json version ${packageJson.version}`, itemIssues);
  requireValue(evidence.sourceCommit === currentSourceCommit, `sourceCommit must match current git commit ${currentSourceCommit}`, itemIssues);
  requireValue(evidence.sourceTreeClean === true, "sourceTreeClean must be true when evidence is collected", itemIssues);
  requireValue(evidence.status === "passed", "status must be passed", itemIssues);
  requireValue(isIsoDate(evidence.generatedAt), "generatedAt must be an ISO timestamp", itemIssues);
  requireValue(evidence.reviewType === "independent-security-review", "reviewType must be independent-security-review", itemIssues);
  requireValue(evidence.independentReviewer === true, "independentReviewer must be true", itemIssues);
  requireValue(nonEmpty(evidence.reviewer?.name), "reviewer.name must be supplied", itemIssues);
  requireValue(nonEmpty(evidence.reviewer?.role), "reviewer.role must be supplied", itemIssues);
  requireValue(nonEmpty(evidence.reviewer?.organization), "reviewer.organization must be supplied", itemIssues);
  requireValue(isIsoDate(evidence.reviewer?.reviewedAt), "reviewer.reviewedAt must be an ISO timestamp", itemIssues);
  validateScope(evidence.scope, itemIssues);
  validateFindings(evidence.findings, itemIssues);
  validateArtifacts(evidence.artifacts, itemIssues);
  validateTools(evidence.tools, itemIssues);
  validateSignoff(evidence.signoff, itemIssues);

  return {
    path,
    status: itemIssues.length === 0 ? "accepted" : "invalid",
    reviewer: evidence.reviewer?.name || null,
    organization: evidence.reviewer?.organization || null,
    generatedAt: evidence.generatedAt || null,
    issues: itemIssues,
  };
}

function validateScope(scope, issues) {
  const boundaryIds = new Set(Array.isArray(scope?.trustBoundaries) ? scope.trustBoundaries : []);
  for (const id of requiredBoundaryIds) {
    requireValue(boundaryIds.has(id), `scope.trustBoundaries must include ${id}`, issues);
  }
  const artifactIds = new Set(Array.isArray(scope?.reviewedArtifacts) ? scope.reviewedArtifacts : []);
  for (const id of requiredArtifactIds) {
    requireValue(artifactIds.has(id), `scope.reviewedArtifacts must include ${id}`, issues);
  }
}

function validateFindings(findings, issues) {
  requireValue(Number(findings?.critical || 0) === 0, "findings.critical must be 0", issues);
  requireValue(Number(findings?.high || 0) === 0, "findings.high must be 0", issues);
  requireValue(Number(findings?.unresolved || 0) === 0, "findings.unresolved must be 0", issues);
  requireValue(Number(findings?.medium || 0) <= 3, "findings.medium must be <= 3", issues);
  requireValue(Array.isArray(findings?.acceptedRisks), "findings.acceptedRisks must be an array", issues);
}

function validateArtifacts(artifacts, issues) {
  requireValue(isSha256(artifacts?.reportSha256), "artifacts.reportSha256 must be a 64-character SHA-256", issues);
  requireValue(nonEmpty(artifacts?.reportReference), "artifacts.reportReference must be supplied", issues);
  if (artifacts?.toolOutputSha256 !== undefined) {
    requireValue(isSha256(artifacts.toolOutputSha256), "artifacts.toolOutputSha256 must be a 64-character SHA-256 when supplied", issues);
  }
}

function validateTools(tools, issues) {
  requireValue(Array.isArray(tools) && tools.length > 0, "tools must include at least one review method or scanner", issues);
  for (const tool of Array.isArray(tools) ? tools : []) {
    requireValue(nonEmpty(tool.name), "tool.name must be supplied", issues);
    requireValue(nonEmpty(tool.result), "tool.result must be supplied", issues);
  }
}

function validateSignoff(signoff, issues) {
  requireValue(signoff?.approvedForRelease === true, "signoff.approvedForRelease must be true", issues);
  requireValue(signoff?.secretsStored === false, "signoff.secretsStored must be false", issues);
  requireValue(signoff?.networkTelemetryAdded === false, "signoff.networkTelemetryAdded must be false", issues);
  requireValue(signoff?.externalExecutionReviewed === true, "signoff.externalExecutionReviewed must be true", issues);
  requireValue(signoff?.providerBoundaryReviewed === true, "signoff.providerBoundaryReviewed must be true", issues);
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
  const templatePath = join(templatesDir, "security-review.template.json");
  writeFileSync(
    templatePath,
    `${JSON.stringify(
      {
        schema: "neditor.security-review-evidence.v1",
        generatedAt: new Date().toISOString(),
        status: "passed",
        appVersion: packageJson.version,
        sourceCommit: currentSourceCommit,
        sourceTreeClean: currentSourceTreeClean,
        reviewType: "independent-security-review",
        independentReviewer: true,
        reviewer: {
          name: "replace-with-reviewer-name",
          role: "security-reviewer",
          organization: "replace-with-independent-team-or-company",
          reviewedAt: new Date().toISOString(),
        },
        scope: {
          trustBoundaries: requiredBoundaryIds,
          reviewedArtifacts: requiredArtifactIds,
        },
        tools: [
          {
            name: "manual-threat-model-review",
            result: "passed",
          },
        ],
        findings: {
          critical: 0,
          high: 0,
          medium: 0,
          low: 0,
          unresolved: 0,
          acceptedRisks: [],
        },
        artifacts: {
          reportReference: "replace-with-review-report-location-or-ticket",
          reportSha256: "replace-with-64-character-sha256",
          toolOutputSha256: "replace-with-64-character-sha256",
        },
        signoff: {
          approvedForRelease: true,
          secretsStored: false,
          networkTelemetryAdded: false,
          externalExecutionReviewed: true,
          providerBoundaryReviewed: true,
        },
      },
      null,
      2,
    )}\n`,
  );
}

function writeReport(status, evidence, invalid) {
  const templatePath = join(templatesDir, "security-review.template.json");
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        schema: "neditor.security-review-report.v1",
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
        acceptedReviews: evidence.filter((item) => item.status === "accepted").map((item) => ({
          reviewer: item.reviewer,
          organization: item.organization,
          generatedAt: item.generatedAt,
          path: relative(item.path),
        })),
        evidence: evidence.map((item) => ({
          ...item,
          path: relative(item.path),
        })),
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

function nonEmpty(value) {
  return typeof value === "string" && value.trim().length > 0;
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
