import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = parseArgs(process.argv.slice(2));
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const auditDir = resolve(args["audit-dir"] || process.env.NEDITOR_RENDERED_EXPORT_AUDIT_DIR || join(root, ".tmp", "rendered-export-audit"));
const sourceDocxPath = resolve(args["source-docx"] || join(auditDir, "rendered-export-audit.docx"));
const googleDocsPackagePath = resolve(args["google-docs-package"] || join(auditDir, "rendered-export-audit.google-docs.zip"));
const outputPath = resolve(
  args.output || process.env.NEDITOR_GOOGLE_DOCS_IMPORT_EVIDENCE || join(root, ".tmp", "google-docs-import", "external", "import-evidence.json"),
);
const exportedDocxPath = resolve(
  args["exported-docx-output"] || process.env.NEDITOR_GOOGLE_DOCS_EXPORTED_DOCX || join(root, ".tmp", "google-docs-import", "external", "exported-google-docs.docx"),
);
const readbackTextPath = resolve(
  args["readback-text-output"] || process.env.NEDITOR_GOOGLE_DOCS_READBACK_TEXT_FILE || join(root, ".tmp", "google-docs-import", "external", "readback.txt"),
);
const reportPath = resolve(args.report || join(root, ".tmp", "google-docs-import", "live-import-report.json"));
const title = String(args["document-title"] || process.env.NEDITOR_GOOGLE_DOCS_DOCUMENT_TITLE || "NEditor Rendered Export Audit Import Proof").trim();
const keepDocument = args["keep-document"] !== false && args["delete-after"] !== true;
const dryRun = args["dry-run"] === true || args.help === true;
const requiredText = ["Rendered Export Audit", "Control summary", "AI Provenance"];

if (args.help) {
  console.log(helpText());
  process.exit(0);
}

const localIssues = validateLocalInputs();
if (dryRun) {
  const report = {
    schema: "neditor.google-docs-live-import-plan.v1",
    status: localIssues.length ? "not-ready" : "ready",
    generatedAt: new Date().toISOString(),
    sourceDocx: fileEvidence(sourceDocxPath, 0),
    googleDocsPackage: fileEvidence(googleDocsPackagePath, 0),
    outputPath: relative(outputPath),
    exportedDocxPath: relative(exportedDocxPath),
    readbackTextPath: relative(readbackTextPath),
    requiresEnvironment: ["NEDITOR_GOOGLE_ACCESS_TOKEN or GOOGLE_ACCESS_TOKEN"],
    optionalEnvironment: ["NEDITOR_GOOGLE_REFRESH_TOKEN", "NEDITOR_GOOGLE_CLIENT_ID"],
    nextCommand:
      "NEDITOR_GOOGLE_ACCESS_TOKEN=<token> pnpm run collect:google-docs-live -- --keep-document",
    issues: localIssues,
  };
  writeJson(reportPath, report);
  console.log(JSON.stringify(report, null, 2));
  process.exit(localIssues.length ? 1 : 0);
}

const run = await runLiveImport();
writeJson(reportPath, run.report);
writeJson(outputPath, run.evidence);
mkdirSync(dirname(readbackTextPath), { recursive: true });
writeFileSync(readbackTextPath, run.readbackText);
mkdirSync(dirname(exportedDocxPath), { recursive: true });
writeFileSync(exportedDocxPath, run.exportedDocx);
console.log(`Collected live Google Docs import evidence: ${relative(outputPath)}`);
console.log(`Validate it with: pnpm run check:google-docs-import`);

async function runLiveImport() {
  if (localIssues.length) fail(localIssues);
  const sourceCommit = gitCommit();
  const sourceTreeClean = gitTreeClean();
  const issues = [];
  if (!sourceCommit) issues.push("Source commit is required. Run from a Git checkout.");
  if (!sourceTreeClean) issues.push("Google Docs import evidence must be collected from a clean Git tree.");
  if (!title) issues.push("Document title is required.");
  if (issues.length) fail(issues);

  const token = await resolveAccessToken();
  const sourceBytes = readFileSync(sourceDocxPath);
  const boundary = `neditor-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
  const uploadResponse = await googleFetch(
    "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&fields=id,name,webViewLink,mimeType",
    {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": `multipart/related; boundary=${boundary}`,
      },
      body: multipartUploadBody(boundary, title, sourceBytes),
    },
  );
  if (!uploadResponse.id) fail(["Google Drive upload did not return an imported document id."]);

  const documentId = uploadResponse.id;
  const readbackText = await googleText(
    `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(documentId)}/export?mimeType=text/plain`,
    token,
  );
  const exportedDocx = await googleBytes(
    `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(documentId)}/export?mimeType=${encodeURIComponent(
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    )}`,
    token,
  );

  if (!keepDocument) {
    await googleFetch(`https://www.googleapis.com/drive/v3/files/${encodeURIComponent(documentId)}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${token}` },
      allowEmpty: true,
    });
  }

  const missingMarkers = requiredText.filter((marker) => !readbackText.includes(marker));
  if (missingMarkers.length) fail(missingMarkers.map((marker) => `Google Docs readback text is missing required marker: ${marker}`));
  if (exportedDocx.length < 1000) fail([`Google Docs exported DOCX is unexpectedly small: ${exportedDocx.length} bytes`]);

  const evidence = {
    schema: "neditor.google-docs-import-evidence.v1",
    status: "passed",
    generatedAt: new Date().toISOString(),
    appVersion: packageJson.version,
    sourceCommit,
    sourceTreeClean,
    importMethod: "google-drive-import-document",
    sourceArtifacts: {
      docxPath: relative(sourceDocxPath),
      docxSha256: sha256(sourceBytes),
      googleDocsPackagePath: relative(googleDocsPackagePath),
      googleDocsPackageSha256: sha256File(googleDocsPackagePath),
    },
    importedDocument: {
      id: documentId,
      title: uploadResponse.name || title,
      url: uploadResponse.webViewLink || `https://docs.google.com/document/d/${documentId}/edit`,
    },
    readback: {
      paragraphCount: paragraphCount(readbackText),
      requiredText,
    },
    exportedDocx: {
      path: relative(exportedDocxPath),
      bytes: exportedDocx.length,
      sha256: sha256(exportedDocx),
    },
    unresolvedBlockers: [],
    notes:
      "Generated by scripts/collect-google-docs-live-import.mjs from a live Google Drive import, text readback, and DOCX export. No token is stored in this evidence file.",
  };
  return {
    evidence,
    readbackText,
    exportedDocx,
    report: {
      schema: "neditor.google-docs-live-import-report.v1",
      status: "passed",
      generatedAt: evidence.generatedAt,
      appVersion: packageJson.version,
      sourceCommit,
      sourceTreeClean,
      keptImportedDocument: keepDocument,
      importedDocument: evidence.importedDocument,
      outputs: {
        evidence: relative(outputPath),
        exportedDocx: relative(exportedDocxPath),
        readbackText: relative(readbackTextPath),
      },
      requiredText,
      unresolvedBlockers: [],
    },
  };
}

async function resolveAccessToken() {
  const direct = String(args["access-token"] || process.env.NEDITOR_GOOGLE_ACCESS_TOKEN || process.env.GOOGLE_ACCESS_TOKEN || "").trim();
  if (direct) return direct;
  const refreshToken = String(args["refresh-token"] || process.env.NEDITOR_GOOGLE_REFRESH_TOKEN || process.env.GOOGLE_REFRESH_TOKEN || "").trim();
  const clientId = String(args["client-id"] || process.env.NEDITOR_GOOGLE_CLIENT_ID || process.env.GOOGLE_CLIENT_ID || "").trim();
  if (!refreshToken || !clientId) {
    fail([
      "Missing Google access token. Set NEDITOR_GOOGLE_ACCESS_TOKEN or GOOGLE_ACCESS_TOKEN.",
      "Alternatively set NEDITOR_GOOGLE_REFRESH_TOKEN and NEDITOR_GOOGLE_CLIENT_ID to refresh a desktop OAuth session token.",
    ]);
  }
  const body = new URLSearchParams();
  body.set("client_id", clientId);
  body.set("grant_type", "refresh_token");
  body.set("refresh_token", refreshToken);
  const response = await fetch("https://oauth2.googleapis.com/token", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body,
  });
  const token = await response.json().catch(() => ({}));
  if (!response.ok || !token.access_token) {
    fail([`Google token refresh failed with HTTP ${response.status}: ${token.error_description || token.error || "unknown error"}`]);
  }
  return token.access_token;
}

async function googleFetch(url, options = {}) {
  const response = await fetch(url, options);
  if (options.allowEmpty && response.ok) return {};
  const text = await response.text();
  if (!response.ok) {
    fail([`Google API request failed with HTTP ${response.status}: ${redactToken(text)}`]);
  }
  return text ? JSON.parse(text) : {};
}

async function googleText(url, token) {
  const response = await fetch(url, { headers: { Authorization: `Bearer ${token}` } });
  const text = await response.text();
  if (!response.ok) fail([`Google Docs text export failed with HTTP ${response.status}: ${redactToken(text)}`]);
  return text;
}

async function googleBytes(url, token) {
  const response = await fetch(url, { headers: { Authorization: `Bearer ${token}` } });
  const bytes = Buffer.from(await response.arrayBuffer());
  if (!response.ok) fail([`Google Docs DOCX export failed with HTTP ${response.status}: ${redactToken(bytes.toString("utf8", 0, Math.min(bytes.length, 2000)))}`]);
  return bytes;
}

function multipartUploadBody(boundary, name, docxBytes) {
  return Buffer.concat([
    Buffer.from(`--${boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n`),
    Buffer.from(
      JSON.stringify({
        name,
        mimeType: "application/vnd.google-apps.document",
      }),
    ),
    Buffer.from(`\r\n--${boundary}\r\nContent-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document\r\n\r\n`),
    docxBytes,
    Buffer.from(`\r\n--${boundary}--\r\n`),
  ]);
}

function validateLocalInputs() {
  const issues = [];
  for (const [path, label] of [
    [sourceDocxPath, "rendered export DOCX"],
    [googleDocsPackagePath, "Google Docs package"],
  ]) {
    if (!existsSync(path)) {
      issues.push(`Missing ${label}: ${relative(path)}`);
      continue;
    }
    const bytes = statSync(path).size;
    if (bytes < 1000) issues.push(`${label} is unexpectedly small: ${bytes} bytes`);
  }
  return issues;
}

function fileEvidence(path, minBytes) {
  if (!existsSync(path)) return { path: relative(path), exists: false, bytes: 0, sha256: null };
  const bytes = statSync(path).size;
  return {
    path: relative(path),
    exists: true,
    bytes,
    sha256: bytes >= minBytes ? sha256File(path) : null,
  };
}

function writeJson(path, value) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function paragraphCount(text) {
  return Math.max(
    1,
    text
      .split(/\n{1,}/)
      .map((line) => line.trim())
      .filter(Boolean).length,
  );
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function sha256File(path) {
  return sha256(readFileSync(path));
}

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  return result.status === 0 ? result.stdout.trim() : "";
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain"], { cwd: root, encoding: "utf8" });
  return result.status === 0 && result.stdout.trim() === "";
}

function fail(issues) {
  const report = {
    schema: "neditor.google-docs-live-import-report.v1",
    status: "failed",
    generatedAt: new Date().toISOString(),
    issues,
  };
  writeJson(reportPath, report);
  console.error("Live Google Docs import evidence collection failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

function redactToken(text) {
  return String(text).replace(/ya29\.[A-Za-z0-9._-]+/g, "ya29.<redacted>").slice(0, 2000);
}

function parseArgs(values) {
  const parsed = {};
  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (!value.startsWith("--")) continue;
    const key = value.slice(2);
    const next = values[index + 1];
    if (!next || next.startsWith("--")) {
      parsed[key] = true;
    } else {
      parsed[key] = next;
      index += 1;
    }
  }
  return parsed;
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}

function helpText() {
  return `Usage:
  pnpm run collect:google-docs-live -- --keep-document
  NEDITOR_GOOGLE_ACCESS_TOKEN=<token> pnpm run collect:google-docs-live
  NEDITOR_GOOGLE_REFRESH_TOKEN=<refresh> NEDITOR_GOOGLE_CLIENT_ID=<desktop-client-id> pnpm run collect:google-docs-live

Options:
  --access-token <token>          Session-only Google OAuth access token. Prefer env vars to avoid shell history.
  --refresh-token <token>         Optional refresh token for desktop OAuth clients.
  --client-id <id>                Desktop OAuth client id used with --refresh-token.
  --source-docx <path>            DOCX to import. Defaults to rendered-export-audit.docx.
  --google-docs-package <path>    Google Docs handoff ZIP whose hash must match release evidence.
  --document-title <title>        Imported Google Doc title.
  --output <path>                 Evidence JSON output.
  --exported-docx-output <path>   Returned Google Docs DOCX export path.
  --readback-text-output <path>   Returned Google Docs text readback path.
  --keep-document                 Leave imported Google Doc in Drive. Default.
  --delete-after                  Delete imported Google Doc after evidence collection.
  --dry-run                       Validate local artifacts and print the credentialed command plan.

The script uploads the rendered DOCX as a native Google Doc, reads back text,
exports DOCX from Drive, and writes neditor.google-docs-import-evidence.v1
without storing OAuth tokens.`;
}
