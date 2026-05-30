import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputDir = join(root, ".tmp", "performance-profile");
const reportPath = join(outputDir, "report.json");
const templatesDir = join(outputDir, "templates");
const defaultEvidenceDir = join(outputDir, "external");
const evidenceDir = resolve(process.env.NEDITOR_PERFORMANCE_PROFILE_EVIDENCE_DIR || defaultEvidenceDir);
const explicitEvidence = process.env.NEDITOR_PERFORMANCE_PROFILE_EVIDENCE
  ? [resolve(process.env.NEDITOR_PERFORMANCE_PROFILE_EVIDENCE)]
  : [];
const currentSourceCommit = gitCommit();
const currentSourceTreeClean = gitTreeClean();
const requiredScenarioIds = [
  "startup-open-document",
  "large-document-edit-preview",
  "export-suite",
  "native-file-watch-conflict",
  "agent-workflow-review",
];
const requiredExportTargets = ["html", "pdf", "docx", "pptx"];

mkdirSync(outputDir, { recursive: true });
mkdirSync(templatesDir, { recursive: true });
writeTemplate();

const evidenceFiles = [...explicitEvidence, ...discoverEvidenceFiles(evidenceDir)];
const evidence = evidenceFiles.map((path) => validateEvidenceFile(path));
const accepted = evidence.filter((item) => item.status === "accepted");
const invalid = evidence.filter((item) => item.status === "invalid");
const stale = evidence.filter((item) => item.status === "stale");
const status = invalid.length > 0 ? "failed" : accepted.length > 0 ? "accepted" : "pending-release-device-profile";

writeReport(status, evidence, invalid);

if (invalid.length > 0) {
  console.error("Performance profile evidence failed validation:");
  for (const item of invalid) {
    console.error(`- ${relative(item.path)}: ${item.issues.join("; ")}`);
  }
  process.exit(1);
}

console.log(`Performance profile evidence is ${status}; wrote ${relative(reportPath)}.`);

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

  requireValue(evidence.schema === "neditor.performance-profile-evidence.v1", "schema must be neditor.performance-profile-evidence.v1", itemIssues);
  requireValue(evidence.appVersion === packageJson.version, `appVersion must match package.json version ${packageJson.version}`, itemIssues);
  if (evidence.sourceCommit && evidence.sourceCommit !== currentSourceCommit) {
    return {
      path,
      status: "stale",
      platform: evidence.platform || null,
      arch: evidence.arch || null,
      generatedAt: evidence.generatedAt || null,
      durationMinutes: Number(evidence.durationMinutes || 0),
      sourceCommit: evidence.sourceCommit,
      issues: [`sourceCommit ${evidence.sourceCommit} does not match current git commit ${currentSourceCommit}`],
    };
  }
  requireValue(evidence.sourceCommit === currentSourceCommit, `sourceCommit must match current git commit ${currentSourceCommit}`, itemIssues);
  requireValue(evidence.sourceTreeClean === true, "sourceTreeClean must be true when evidence is collected", itemIssues);
  requireValue(evidence.status === "passed", "status must be passed", itemIssues);
  requireValue(isIsoDate(evidence.generatedAt), "generatedAt must be an ISO timestamp", itemIssues);
  requireValue(evidence.runtime === "tauri-release", "runtime must be tauri-release", itemIssues);
  requireValue(nonEmpty(evidence.platform), "platform must be supplied", itemIssues);
  requireValue(nonEmpty(evidence.arch), "arch must be supplied", itemIssues);
  requireValue(nonEmpty(evidence.deviceClass), "deviceClass must be supplied", itemIssues);
  requireValue(Number(evidence.durationMinutes || 0) >= 30, "durationMinutes must be at least 30", itemIssues);
  requireValue(nonEmpty(evidence.environment?.osVersion), "environment.osVersion must be supplied", itemIssues);
  requireValue(nonEmpty(evidence.environment?.cpu), "environment.cpu must be supplied", itemIssues);
  requireValue(Number(evidence.environment?.memoryGb || 0) >= 4, "environment.memoryGb must be at least 4", itemIssues);
  requireValue(evidence.binary?.target === "release", "binary.target must be release", itemIssues);
  requireValue(evidence.binary?.appVersion === packageJson.version, `binary.appVersion must match ${packageJson.version}`, itemIssues);
  requireValue(Number(evidence.binary?.bytes || 0) > 1_000_000, "binary.bytes must describe a real release artifact", itemIssues);
  requireValue(isSha256(evidence.binary?.sha256), "binary.sha256 must be a 64-character SHA-256", itemIssues);
  validateScenarios(evidence.scenarios, itemIssues);
  validateSoak(evidence.soak, evidence.durationMinutes, itemIssues);
  validateExports(evidence.exports, itemIssues);
  requireValue(isSha256(evidence.artifacts?.summarySha256), "artifacts.summarySha256 must be a 64-character SHA-256", itemIssues);
  if (evidence.artifacts?.traceSha256 !== undefined) {
    requireValue(isSha256(evidence.artifacts.traceSha256), "artifacts.traceSha256 must be a 64-character SHA-256 when supplied", itemIssues);
  }
  requireValue(nonEmpty(evidence.reviewer?.name), "reviewer.name must be supplied", itemIssues);
  requireValue(nonEmpty(evidence.reviewer?.role), "reviewer.role must be supplied", itemIssues);
  requireValue(isIsoDate(evidence.reviewer?.reviewedAt), "reviewer.reviewedAt must be an ISO timestamp", itemIssues);

  return {
    path,
    status: itemIssues.length === 0 ? "accepted" : "invalid",
    platform: evidence.platform || null,
    arch: evidence.arch || null,
    generatedAt: evidence.generatedAt || null,
    durationMinutes: Number(evidence.durationMinutes || 0),
    issues: itemIssues,
  };
}

function validateScenarios(scenarios, issues) {
  if (!Array.isArray(scenarios)) {
    issues.push("scenarios must be an array");
    return;
  }
  const byId = new Map(scenarios.map((scenario) => [scenario?.id, scenario]));
  for (const id of requiredScenarioIds) {
    const scenario = byId.get(id);
    if (!scenario) {
      issues.push(`missing scenario ${id}`);
      continue;
    }
    requireValue(scenario.status === "passed", `${id}.status must be passed`, issues);
    requireValue(Number(scenario.samples || 0) >= 3, `${id}.samples must be at least 3`, issues);
    requireValue(Number(scenario.p95InteractionMs || 0) > 0, `${id}.p95InteractionMs must be positive`, issues);
    requireValue(Number(scenario.p95InteractionMs || 0) <= 500, `${id}.p95InteractionMs must be <= 500`, issues);
    requireValue(Number(scenario.maxInteractionMs || 0) <= 2_000, `${id}.maxInteractionMs must be <= 2000`, issues);
    requireValue(Number(scenario.peakRssMb || 0) > 0, `${id}.peakRssMb must be positive`, issues);
    requireValue(Number(scenario.peakRssMb || 0) <= 4_096, `${id}.peakRssMb must be <= 4096`, issues);
    requireValue(Number(scenario.memoryGrowthMb || 0) <= 300, `${id}.memoryGrowthMb must be <= 300`, issues);
  }
}

function validateSoak(soak, durationMinutes, issues) {
  requireValue(Number(soak?.durationMinutes || 0) >= Number(durationMinutes || 0), "soak.durationMinutes must cover the evidence duration", issues);
  requireValue(Number(soak?.crashes ?? 1) === 0, "soak.crashes must be 0", issues);
  requireValue(Number(soak?.hangs ?? 1) === 0, "soak.hangs must be 0", issues);
  requireValue(Number(soak?.errorDialogs ?? 1) === 0, "soak.errorDialogs must be 0", issues);
  requireValue(Number(soak?.memoryGrowthMb || 0) <= 300, "soak.memoryGrowthMb must be <= 300", issues);
}

function validateExports(exports, issues) {
  requireValue(Number(exports?.completed || 0) >= requiredExportTargets.length, "exports.completed must cover required targets", issues);
  requireValue(Number(exports?.failed ?? 1) === 0, "exports.failed must be 0", issues);
  const targets = new Set(Array.isArray(exports?.targets) ? exports.targets : []);
  for (const target of requiredExportTargets) {
    requireValue(targets.has(target), `exports.targets must include ${target}`, issues);
  }
  requireValue(Number(exports?.p95Ms || 0) > 0, "exports.p95Ms must be positive", issues);
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
  const templatePath = join(templatesDir, "native-profile.template.json");
  const metricsTemplatePath = join(templatesDir, "native-profile-metrics.template.json");
  writeFileSync(
    templatePath,
    `${JSON.stringify(
      {
        schema: "neditor.performance-profile-evidence.v1",
        generatedAt: new Date().toISOString(),
        status: "passed",
        appVersion: packageJson.version,
        sourceCommit: currentSourceCommit,
        sourceTreeClean: currentSourceTreeClean,
        platform: process.platform,
        arch: process.arch,
        runtime: "tauri-release",
        deviceClass: "release-laptop-or-workstation",
        durationMinutes: 45,
        environment: {
          osVersion: "replace-with-os-version",
          cpu: "replace-with-cpu-model",
          memoryGb: 16,
          powerMode: "plugged-in",
        },
        binary: {
          target: "release",
          appVersion: packageJson.version,
          path: "replace-with-profiled-release-binary-path",
          bytes: 1234567,
          sha256: "replace-with-64-character-sha256",
        },
        scenarios: requiredScenarioIds.map((id) => ({
          id,
          status: "passed",
          samples: 5,
          p95InteractionMs: 250,
          maxInteractionMs: 750,
          peakRssMb: 900,
          memoryGrowthMb: 80,
          notes: "Replace with profiler notes and measurement method.",
        })),
        soak: {
          durationMinutes: 45,
          crashes: 0,
          hangs: 0,
          errorDialogs: 0,
          memoryGrowthMb: 80,
        },
        exports: {
          completed: 5,
          failed: 0,
          targets: ["html", "pdf", "docx", "pptx", "markdown-bundle"],
          p95Ms: 3_000,
        },
        artifacts: {
          summarySha256: "replace-with-64-character-sha256",
          traceSha256: "replace-with-64-character-sha256",
          notes: "Return hashes, not raw profiler traces.",
        },
        reviewer: {
          name: "replace-with-reviewer-name",
          role: "release-performance-reviewer",
          reviewedAt: new Date().toISOString(),
        },
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(
    metricsTemplatePath,
    `${JSON.stringify(
      {
        schema: "neditor.performance-profile-metrics.v1",
        durationMinutes: 45,
        platform: process.platform,
        arch: process.arch,
        deviceClass: "release-laptop-or-workstation",
        environment: {
          osVersion: "replace-with-profiled-device-os-version",
          cpu: "replace-with-profiled-device-cpu-model",
          memoryGb: 16,
          powerMode: "plugged-in",
        },
        scenarios: requiredScenarioIds.map((id) => ({
          id,
          status: "passed",
          samples: 5,
          p95InteractionMs: 250,
          maxInteractionMs: 750,
          peakRssMb: 900,
          memoryGrowthMb: 80,
          notes: "Replace with measurement method, profiler name, and observed behavior.",
        })),
        soak: {
          durationMinutes: 45,
          crashes: 0,
          hangs: 0,
          errorDialogs: 0,
          memoryGrowthMb: 80,
        },
        exports: {
          completed: 5,
          failed: 0,
          targets: ["html", "pdf", "docx", "pptx", "markdown-bundle"],
          p95Ms: 3_000,
        },
        artifacts: {
          notes: "Summarize profiler artifact paths and measurement context. The collector records hashes from --summary-artifact and optional --trace-artifact.",
        },
      },
      null,
      2,
    )}\n`,
  );
}

function writeReport(status, evidence, invalid) {
  const templatePath = join(templatesDir, "native-profile.template.json");
  const metricsTemplatePath = join(templatesDir, "native-profile-metrics.template.json");
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        schema: "neditor.performance-profile-report.v1",
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
        metricsTemplate: {
          path: relative(metricsTemplatePath),
          bytes: statSync(metricsTemplatePath).size,
        },
        summary: {
          acceptedEvidence: evidence.filter((item) => item.status === "accepted").length,
          invalidEvidence: invalid.length,
          staleEvidence: stale.length,
          discoveredEvidence: evidence.length,
        },
        acceptedProfiles: evidence.filter((item) => item.status === "accepted").map((item) => ({
          platform: item.platform,
          arch: item.arch,
          generatedAt: item.generatedAt,
          durationMinutes: item.durationMinutes,
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
