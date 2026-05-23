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
const outputPath = resolve(
  args.output || process.env.NEDITOR_GOOGLE_DOCS_IMPORT_EVIDENCE || join(root, ".tmp", "google-docs-import", "external", "import-evidence.json"),
);
const sourceDocxPath = resolve(args["source-docx"] || join(auditDir, "rendered-export-audit.docx"));
const googleDocsPackagePath = resolve(args["google-docs-package"] || join(auditDir, "rendered-export-audit.google-docs.zip"));
const exportedDocxPath = resolve(required("exported-docx", "NEDITOR_GOOGLE_DOCS_EXPORTED_DOCX"));
const importedDocumentId = String(args["document-id"] || process.env.NEDITOR_GOOGLE_DOCS_DOCUMENT_ID || "").trim();
const importedDocumentTitle = String(args["document-title"] || process.env.NEDITOR_GOOGLE_DOCS_DOCUMENT_TITLE || "").trim();
const importedDocumentUrl = String(
  args["document-url"] || process.env.NEDITOR_GOOGLE_DOCS_DOCUMENT_URL || googleDocsUrl(importedDocumentId),
).trim();
const readbackText = readback();
const sourceCommit = String(args["source-commit"] || process.env.NEDITOR_SOURCE_COMMIT || gitCommit()).trim();
const sourceTreeClean = gitTreeClean();
const requiredText = ["Rendered Export Audit", "Control summary", "AI Provenance"];
const issues = [];

if (!sourceCommit) issues.push("Source commit is required. Run from a Git checkout or pass --source-commit / NEDITOR_SOURCE_COMMIT.");
if (!sourceTreeClean) issues.push("Google Docs import evidence must be collected from a clean Git tree.");
if (!importedDocumentId) issues.push("Imported Google document id is required. Pass --document-id or NEDITOR_GOOGLE_DOCS_DOCUMENT_ID.");
if (!importedDocumentTitle) issues.push("Imported Google document title is required. Pass --document-title or NEDITOR_GOOGLE_DOCS_DOCUMENT_TITLE.");
if (!importedDocumentUrl.includes("docs.google.com/document")) issues.push("Imported Google document URL must be a Google Docs URL.");
for (const marker of requiredText) {
  if (!readbackText.includes(marker)) issues.push(`Google Docs readback text is missing required marker: ${marker}`);
}

const sourceDocx = fileEvidence(sourceDocxPath, "rendered export DOCX", 1000);
const googleDocsPackage = fileEvidence(googleDocsPackagePath, "rendered Google Docs package", 1000);
const exportedDocx = fileEvidence(exportedDocxPath, "Google Docs exported DOCX", 1000);

if (issues.length > 0) {
  console.error("Google Docs import evidence collection failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(
  outputPath,
  `${JSON.stringify(
    {
      schema: "neditor.google-docs-import-evidence.v1",
      status: "passed",
      generatedAt: new Date().toISOString(),
      appVersion: packageJson.version,
      sourceCommit,
      sourceTreeClean,
      importMethod: "google-drive-import-document",
      sourceArtifacts: {
        docxPath: relative(sourceDocxPath),
        docxSha256: sourceDocx.sha256,
        googleDocsPackagePath: relative(googleDocsPackagePath),
        googleDocsPackageSha256: googleDocsPackage.sha256,
      },
      importedDocument: {
        id: importedDocumentId,
        title: importedDocumentTitle,
        url: importedDocumentUrl,
      },
      readback: {
        paragraphCount: paragraphCount(readbackText),
        requiredText,
      },
      exportedDocx: {
        path: relative(exportedDocxPath),
        bytes: exportedDocx.bytes,
        sha256: exportedDocx.sha256,
      },
      unresolvedBlockers: [],
      notes:
        "Generated from a native Google Docs import, connector readback text, and a DOCX export from Google Drive.",
    },
    null,
    2,
  )}\n`,
);

console.log(`Collected Google Docs import evidence: ${relative(outputPath)}`);

function required(argName, envName) {
  const value = args[argName] || process.env[envName];
  if (!value) {
    console.error(`Missing required ${argName}. Pass --${argName} or set ${envName}.`);
    process.exit(1);
  }
  return String(value);
}

function readback() {
  const textPath = args["readback-text-file"] || process.env.NEDITOR_GOOGLE_DOCS_READBACK_TEXT_FILE;
  if (textPath) return readFileSync(resolve(String(textPath)), "utf8");
  const inline = args["readback-text"] || process.env.NEDITOR_GOOGLE_DOCS_READBACK_TEXT;
  if (!inline) {
    console.error("Missing Google Docs readback text. Pass --readback-text-file or set NEDITOR_GOOGLE_DOCS_READBACK_TEXT_FILE.");
    process.exit(1);
  }
  return String(inline);
}

function fileEvidence(path, label, minBytes) {
  if (!existsSync(path)) {
    issues.push(`Missing ${label}: ${relative(path)}`);
    return { bytes: 0, sha256: null };
  }
  const bytes = statSync(path).size;
  if (bytes < minBytes) issues.push(`${label} is unexpectedly small: ${bytes} bytes`);
  return {
    bytes,
    sha256: createHash("sha256").update(readFileSync(path)).digest("hex"),
  };
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

function googleDocsUrl(id) {
  return id ? `https://docs.google.com/document/d/${id}/edit` : "";
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
