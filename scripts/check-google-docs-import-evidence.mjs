import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const auditDir = resolve(process.env.NEDITOR_RENDERED_EXPORT_AUDIT_DIR || join(root, ".tmp", "rendered-export-audit"));
const evidencePath = resolve(
  process.env.NEDITOR_GOOGLE_DOCS_IMPORT_EVIDENCE || join(root, ".tmp", "google-docs-import", "external", "import-evidence.json"),
);
const reportPath = join(root, ".tmp", "google-docs-import", "report.json");
const templatePath = join(root, ".tmp", "google-docs-import", "import-evidence.template.json");
const sourceDocxPath = join(auditDir, "rendered-export-audit.docx");
const googleDocsPackagePath = join(auditDir, "rendered-export-audit.google-docs.zip");
const auditReportPath = join(auditDir, "rendered-export-audit-report.json");
const localIssues = [];
const importIssues = [];

const localArtifacts = validateLocalArtifacts();
writeTemplate(localArtifacts);
const importEvidence = validateImportEvidence(localArtifacts);
const status = localIssues.length > 0 ? "failed" : importIssues.length > 0 ? "failed" : importEvidence.status;

writeReport({
  generatedAt: new Date().toISOString(),
  platform: process.platform,
  arch: process.arch,
  status,
  sourceArtifacts: localArtifacts,
  importEvidence,
  templatePath: relative(templatePath),
  evidencePath: relative(evidencePath),
  issues: [...localIssues, ...importIssues],
});

if (localIssues.length > 0 || importIssues.length > 0) {
  console.error("Google Docs import evidence failed validation:");
  for (const issue of [...localIssues, ...importIssues]) console.error(`- ${issue}`);
  console.error(`Wrote ${relative(reportPath)}.`);
  process.exit(1);
}

console.log(`Google Docs import evidence is ${status}; wrote ${relative(reportPath)}.`);

function validateLocalArtifacts() {
  const docx = fileEvidence(sourceDocxPath, "rendered-export-audit.docx", 1000);
  const googleDocsPackage = fileEvidence(googleDocsPackagePath, "rendered-export-audit.google-docs.zip", 1000);
  let auditTarget = null;
  if (!existsSync(auditReportPath)) {
    localIssues.push(`missing rendered export audit report: ${relative(auditReportPath)}`);
  } else {
    try {
      const auditReport = JSON.parse(readFileSync(auditReportPath, "utf8"));
      auditTarget = (auditReport.targets || []).find((target) => target.target === "google-docs") || null;
      if (!auditTarget) {
        localIssues.push("rendered export audit report is missing google-docs target evidence");
      }
    } catch (error) {
      localIssues.push(`rendered export audit report is not valid JSON: ${error.message}`);
    }
  }
  return {
    status: localIssues.length === 0 ? "accepted" : "failed",
    auditDir: relative(auditDir),
    auditReport: relative(auditReportPath),
    docx,
    googleDocsPackage,
    auditTarget: auditTarget
      ? {
          target: auditTarget.target,
          path: auditTarget.path,
          bytes: auditTarget.bytes,
          sha256: auditTarget.sha256,
        }
      : null,
  };
}

function validateImportEvidence(localArtifacts) {
  if (!existsSync(evidencePath)) {
    return {
      status: "pending-google-drive-authorization",
      detail:
        "Live Google Docs import/readback evidence has not been supplied. Use Google Drive import_document plus get_document_text/export readback with a refreshed connector token.",
    };
  }

  let evidence;
  try {
    evidence = JSON.parse(readFileSync(evidencePath, "utf8"));
  } catch (error) {
    importIssues.push(`Google Docs import evidence is not valid JSON: ${error.message}`);
    return {
      status: "invalid",
      detail: "Evidence JSON could not be parsed.",
    };
  }

  requireValue(evidence.schema === "neditor.google-docs-import-evidence.v1", "schema must be neditor.google-docs-import-evidence.v1");
  requireValue(evidence.status === "passed", "status must be passed");
  requireValue(isIsoDate(evidence.generatedAt), "generatedAt must be an ISO timestamp");
  requireValue(evidence.importMethod === "google-drive-import-document", "importMethod must be google-drive-import-document");
  requireValue(Boolean(String(evidence.importedDocument?.id || "").trim()), "importedDocument.id is required");
  requireValue(Boolean(String(evidence.importedDocument?.title || "").trim()), "importedDocument.title is required");
  requireValue(Boolean(String(evidence.importedDocument?.url || "").includes("docs.google.com/document")), "importedDocument.url must be a Google Docs URL");
  requireValue(evidence.sourceArtifacts?.docxSha256 === localArtifacts.docx.sha256, "sourceArtifacts.docxSha256 must match the current rendered DOCX");
  requireValue(
    evidence.sourceArtifacts?.googleDocsPackageSha256 === localArtifacts.googleDocsPackage.sha256,
    "sourceArtifacts.googleDocsPackageSha256 must match the current Google Docs package",
  );

  const textMarkers = Array.isArray(evidence.readback?.requiredText) ? evidence.readback.requiredText : [];
  for (const marker of ["Rendered Export Audit", "Control summary", "AI Provenance"]) {
    requireValue(textMarkers.includes(marker), `readback.requiredText must include ${marker}`);
  }
  requireValue(Number(evidence.readback?.paragraphCount) > 0, "readback.paragraphCount must be positive");
  const exported = evidence.exportedDocx || {};
  requireValue(Number(exported.bytes) > 1000, "exportedDocx.bytes must be > 1000");
  requireValue(isSha256(exported.sha256), "exportedDocx.sha256 must be a sha256");
  requireValue(Array.isArray(evidence.unresolvedBlockers) && evidence.unresolvedBlockers.length === 0, "unresolvedBlockers must be an empty array");

  if (importIssues.length > 0) {
    return {
      status: "invalid",
      detail: importIssues.join("; "),
    };
  }

  return {
    status: "accepted",
    generatedAt: evidence.generatedAt,
    importedDocument: evidence.importedDocument,
    readback: {
      paragraphCount: evidence.readback.paragraphCount,
      requiredText: textMarkers,
    },
    exportedDocx: {
      bytes: exported.bytes,
      sha256: exported.sha256,
    },
  };
}

function fileEvidence(path, label, minBytes) {
  if (!existsSync(path)) {
    localIssues.push(`missing ${label}: ${relative(path)}`);
    return {
      path: relative(path),
      exists: false,
      bytes: 0,
      sha256: null,
    };
  }
  const bytes = statSync(path).size;
  if (bytes < minBytes) {
    localIssues.push(`${label} is unexpectedly small: ${bytes} bytes`);
  }
  return {
    path: relative(path),
    exists: true,
    bytes,
    sha256: sha256File(path),
  };
}

function writeTemplate(localArtifacts) {
  mkdirSync(dirname(templatePath), { recursive: true });
  writeFileSync(
    templatePath,
    `${JSON.stringify(
      {
        schema: "neditor.google-docs-import-evidence.v1",
        status: "passed",
        generatedAt: new Date().toISOString(),
        importMethod: "google-drive-import-document",
        sourceArtifacts: {
          docxPath: localArtifacts.docx.path,
          docxSha256: localArtifacts.docx.sha256 || "run-pnpm-run-test-rendered-exports-first",
          googleDocsPackagePath: localArtifacts.googleDocsPackage.path,
          googleDocsPackageSha256:
            localArtifacts.googleDocsPackage.sha256 || "run-pnpm-run-test-rendered-exports-first",
        },
        importedDocument: {
          id: "google-doc-id",
          title: "NEditor Rendered Export Audit Import Proof",
          url: "https://docs.google.com/document/d/google-doc-id/edit",
        },
        readback: {
          paragraphCount: 1,
          requiredText: ["Rendered Export Audit", "Control summary", "AI Provenance"],
        },
        exportedDocx: {
          bytes: 12345,
          sha256: "replace-with-64-character-sha256",
        },
        unresolvedBlockers: [],
        notes:
          "Fill this after importing rendered-export-audit.docx into native Google Docs, reading document text, and exporting DOCX back from Drive.",
      },
      null,
      2,
    )}\n`,
  );
}

function requireValue(condition, message) {
  if (!condition) importIssues.push(message);
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function isSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/i.test(value);
}

function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function writeReport(report) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
