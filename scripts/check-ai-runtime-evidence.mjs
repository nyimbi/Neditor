import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const outputDir = join(root, ".tmp", "ai-runtime-evidence");
const reportPath = join(outputDir, "report.json");
const templatesDir = join(outputDir, "templates");
const defaultEvidenceDir = join(outputDir, "external");
const evidenceDir = resolve(process.env.NEDITOR_AI_RUNTIME_EVIDENCE_DIR || defaultEvidenceDir);
const explicitEvidence = process.env.NEDITOR_AI_RUNTIME_EVIDENCE ? [resolve(process.env.NEDITOR_AI_RUNTIME_EVIDENCE)] : [];
const currentSourceCommit = gitCommit();
const currentSourceTreeClean = gitTreeClean();
const forbiddenEvidenceKeys = new Set([
  "audioSample",
  "audioSamples",
  "recordedAudio",
  "audioBlob",
  "audioTranscript",
  "clipboardText",
  "clipboardHtml",
  "clipboardPlainText",
  "rawClipboard",
]);
const forbiddenEvidenceValuePatterns = [
  /Runtime clipboard proof/i,
  /BEGIN\s+(AUDIO|CLIPBOARD)/i,
  /data:audio\//i,
];

mkdirSync(outputDir, { recursive: true });
mkdirSync(templatesDir, { recursive: true });
writeTemplate();

const evidenceFiles = [...explicitEvidence, ...discoverEvidenceFiles(evidenceDir)];
const evidence = evidenceFiles.map((path) => validateEvidenceFile(path));
const accepted = evidence.filter((item) => item.status === "accepted");
const invalid = evidence.filter((item) => item.status === "invalid");
const stale = evidence.filter((item) => item.status === "stale");
const status = invalid.length > 0 ? "failed" : accepted.length > 0 ? "accepted" : "pending-real-runtime-evidence";

writeReport(status, evidence, invalid);

if (invalid.length > 0) {
  console.error("AI runtime evidence failed validation:");
  for (const item of invalid) {
    console.error(`- ${relative(item.path)}: ${item.issues.join("; ")}`);
  }
  process.exit(1);
}

console.log(`AI runtime evidence is ${status}; wrote ${relative(reportPath)}.`);

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

  requireValue(evidence.schema === "neditor.ai-runtime-evidence.v1", "schema must be neditor.ai-runtime-evidence.v1", itemIssues);
  requireValue(evidence.appVersion === packageJson.version, `appVersion must match package.json version ${packageJson.version}`, itemIssues);
  if (evidence.sourceCommit && evidence.sourceCommit !== currentSourceCommit) {
    return {
      path,
      status: "stale",
      runtime: evidence.runtime || null,
      platform: evidence.platform || null,
      generatedAt: evidence.generatedAt || null,
      sourceCommit: evidence.sourceCommit,
      issues: [`sourceCommit ${evidence.sourceCommit} does not match current git commit ${currentSourceCommit}`],
    };
  }
  requireValue(evidence.sourceCommit === currentSourceCommit, `sourceCommit must match current git commit ${currentSourceCommit}`, itemIssues);
  requireValue(evidence.sourceTreeClean === true, "sourceTreeClean must be true when evidence is collected", itemIssues);
  requireValue(evidence.status === "passed", "status must be passed", itemIssues);
  requireValue(isIsoDate(evidence.generatedAt), "generatedAt must be an ISO timestamp", itemIssues);
  requireValue(["tauri-webview", "browser"].includes(evidence.runtime), "runtime must be tauri-webview or browser", itemIssues);
  requireValue(Boolean(evidence.secureContext), "secureContext must be true", itemIssues);
  requireValue(evidence.speechRecognition?.supported === true, "speechRecognition.supported must be true", itemIssues);
  requireValue(evidence.speechRecognition?.state === "available", "speechRecognition.state must be available", itemIssues);
  requireValue(evidence.microphonePermission?.state === "granted", "microphonePermission.state must be granted", itemIssues);
  requireValue(evidence.microphoneProbe?.attempted === true, "microphoneProbe.attempted must be true", itemIssues);
  requireValue(["permission-granted", "stream-opened"].includes(evidence.microphoneProbe?.result), "microphoneProbe.result must prove permission-granted or stream-opened", itemIssues);
  requireValue(evidence.microphoneProbe?.audioStored === false, "microphoneProbe.audioStored must be false", itemIssues);
  requireValue(evidence.clipboardRead?.supported === true, "clipboardRead.supported must be true", itemIssues);
  requireValue(evidence.clipboardRead?.state === "granted", "clipboardRead.state must be granted", itemIssues);
  requireValue(["plain", "rich"].includes(evidence.clipboardRead?.kind), "clipboardRead.kind must be plain or rich", itemIssues);
  requireValue(Number(evidence.clipboardRead?.charactersDetected || 0) > 0, "clipboardRead.charactersDetected must be greater than zero", itemIssues);
  requireValue(evidence.clipboardRead?.contentStored === false, "clipboardRead.contentStored must be false", itemIssues);
  requireValue(evidence.clipboardWrite?.supported === true, "clipboardWrite.supported must be true", itemIssues);
  requireValue(evidence.clipboardWrite?.writeSucceeded === true, "clipboardWrite.writeSucceeded must be true", itemIssues);
  requireValue(evidence.clipboardWrite?.contentStored === false, "clipboardWrite.contentStored must be false", itemIssues);
  for (const finding of forbiddenEvidenceFindings(evidence)) {
    itemIssues.push(finding);
  }

  return {
    path,
    status: itemIssues.length === 0 ? "accepted" : "invalid",
    runtime: evidence.runtime || null,
    platform: evidence.platform || null,
    generatedAt: evidence.generatedAt || null,
    issues: itemIssues,
  };
}

function forbiddenEvidenceFindings(value, path = "$") {
  if (Array.isArray(value)) {
    return value.flatMap((item, index) => forbiddenEvidenceFindings(item, `${path}[${index}]`));
  }
  if (!value || typeof value !== "object") {
    if (typeof value === "string" && forbiddenEvidenceValuePatterns.some((pattern) => pattern.test(value))) {
      return [`${path} must not contain audio or clipboard sample content`];
    }
    return [];
  }

  const findings = [];
  for (const [key, entry] of Object.entries(value)) {
    const entryPath = `${path}.${key}`;
    if (forbiddenEvidenceKeys.has(key)) {
      findings.push(`${entryPath} must not store audio or clipboard content`);
    }
    findings.push(...forbiddenEvidenceFindings(entry, entryPath));
  }
  return findings;
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
  const templatePath = join(templatesDir, "runtime-evidence.template.json");
  const readinessTemplatePath = join(templatesDir, "runtime-readiness.template.json");
  writeFileSync(
    templatePath,
    `${JSON.stringify(
      {
        schema: "neditor.ai-runtime-evidence.v1",
        generatedAt: new Date().toISOString(),
        status: "passed",
        appVersion: packageJson.version,
        sourceCommit: currentSourceCommit,
        sourceTreeClean: currentSourceTreeClean,
        platform: process.platform,
        arch: process.arch,
        runtime: "tauri-webview",
        secureContext: true,
        speechRecognition: {
          supported: true,
          state: "available",
        },
        microphonePermission: {
          state: "granted",
        },
        microphoneProbe: {
          attempted: true,
          result: "permission-granted",
          audioStored: false,
          notes: "Describe the host, OS permission prompt, and device class without recording audio.",
        },
        clipboardRead: {
          supported: true,
          state: "granted",
          kind: "rich",
          charactersDetected: 30,
          contentStored: false,
        },
        clipboardWrite: {
          supported: true,
          writeSucceeded: true,
          contentStored: false,
        },
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(
    readinessTemplatePath,
    `${JSON.stringify(
      {
        schema: "neditor.ai-runtime-readiness.v1",
        generatedAt: new Date().toISOString(),
        platform: process.platform,
        arch: process.arch,
        runtime: "tauri-webview",
        secureContext: true,
        speechRecognition: {
          supported: true,
          state: "available",
          detail: "SpeechRecognition API is available.",
        },
        microphonePermission: {
          supported: true,
          state: "granted",
          detail: "microphone permission is granted.",
        },
        clipboardRead: {
          supported: true,
          state: "granted",
          detail: "Clipboard rich read succeeded (30 characters detected, content not stored).",
        },
        clipboardWrite: {
          supported: true,
          state: "granted",
          detail: "clipboard-write permission is granted.",
        },
        issues: [],
      },
      null,
      2,
    )}\n`,
  );
}

function writeReport(status, evidence, invalid) {
  const templatePath = join(templatesDir, "runtime-evidence.template.json");
  const readinessTemplatePath = join(templatesDir, "runtime-readiness.template.json");
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        schema: "neditor.ai-runtime-evidence-report.v1",
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
        readinessTemplate: {
          path: relative(readinessTemplatePath),
          bytes: statSync(readinessTemplatePath).size,
        },
        summary: {
          acceptedEvidence: evidence.filter((item) => item.status === "accepted").length,
          invalidEvidence: invalid.length,
          staleEvidence: stale.length,
          discoveredEvidence: evidence.length,
        },
        acceptedRuntimes: evidence.filter((item) => item.status === "accepted").map((item) => ({
          runtime: item.runtime,
          platform: item.platform,
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
