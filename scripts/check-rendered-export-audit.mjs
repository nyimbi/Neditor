import { rmSync, existsSync, statSync, readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { inflateRawSync } from "node:zlib";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const auditDir = resolve(process.env.NEDITOR_RENDERED_EXPORT_AUDIT_DIR || join(root, ".tmp", "rendered-export-audit"));
const requiredFiles = [
  ["rendered-export-audit.html", 1000],
  ["rendered-export-audit.pdf", 1000],
  ["rendered-export-audit.docx", 1000],
  ["rendered-export-audit.pptx", 1000],
  ["rendered-export-audit.markdown-bundle.zip", 1000],
  ["rendered-export-audit.blog.zip", 1000],
  ["rendered-export-audit.substack.zip", 1000],
  ["rendered-export-audit.tex", 1000],
  ["rendered-export-audit.google-docs.zip", 1000],
  ["rendered-export-audit-report.json", 500],
  ["README.md", 100],
];
const viewerProof = [];

if (auditDir.includes(`${root}/.tmp/`)) {
  rmSync(auditDir, { recursive: true, force: true });
}

const result = spawnSync(
  "cargo",
  [
    "test",
    "--locked",
    "representative_rendered_export_artifacts_are_package_inspectable",
    "--lib",
    "--",
    "--nocapture",
  ],
  {
    cwd: join(root, "src-tauri"),
    env: {
      ...process.env,
      NEDITOR_RENDERED_EXPORT_AUDIT_DIR: auditDir,
    },
    shell: process.platform === "win32",
    stdio: "inherit",
  },
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

const issues = [];
for (const [file, minBytes] of requiredFiles) {
  const path = join(auditDir, file);
  if (!existsSync(path)) {
    issues.push(`missing audit artifact: ${file}`);
    continue;
  }
  const size = statSync(path).size;
  if (size < minBytes) {
    issues.push(`${file} is unexpectedly small: ${size} bytes`);
  }
}

if (issues.length === 0) {
  const report = JSON.parse(readFileSync(join(auditDir, "rendered-export-audit-report.json"), "utf8"));
  const targets = new Set(report.targets?.map((target) => target.target));
  for (const target of ["html", "pdf", "docx", "pptx", "markdown-bundle", "blog", "substack", "latex", "google-docs"]) {
    if (!targets.has(target)) {
      issues.push(`audit report is missing target ${target}`);
    }
  }
  if (!Array.isArray(report.manualChecklist) || report.manualChecklist.length < 5) {
    issues.push("audit report manual checklist is incomplete");
  }
}

if (issues.length === 0) {
  collectViewerProof(issues, viewerProof);
}

if (issues.length === 0) {
  collectMacTextutilProof(issues, viewerProof);
}

if (issues.length === 0) {
  collectMacQuickLookProof(issues, viewerProof);
}

if (issues.length === 0) {
  const pdflatex = spawnSync("pdflatex", ["--version"], { stdio: "ignore" });
  if (pdflatex.status === 0) {
    const latexBuildDir = join(auditDir, "latex-compile");
    rmSync(latexBuildDir, { recursive: true, force: true });
    mkdirSync(latexBuildDir, { recursive: true });
    const latexArgs = [
      "-interaction=nonstopmode",
      "-halt-on-error",
      "-output-directory",
      latexBuildDir,
      join(auditDir, "rendered-export-audit.tex"),
    ];
    const firstCompile = spawnSync("pdflatex", latexArgs, { stdio: "inherit" });
    const secondCompile =
      firstCompile.status === 0
        ? spawnSync("pdflatex", latexArgs, { stdio: "inherit" })
        : firstCompile;
    if (secondCompile.status !== 0) {
      issues.push("pdflatex failed to compile rendered-export-audit.tex");
    } else {
      const compiledPdf = join(latexBuildDir, "rendered-export-audit.pdf");
      if (!existsSync(compiledPdf) || statSync(compiledPdf).size < 1000) {
        issues.push("pdflatex did not produce a meaningful rendered-export-audit.pdf");
      }
    }
  } else {
    console.log("Skipping LaTeX compile proof because pdflatex is not installed.");
  }
}

if (issues.length > 0) {
  console.error("Rendered export audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

writeFileSync(
  join(auditDir, "viewer-proof.json"),
  `${JSON.stringify({ generatedAt: new Date().toISOString(), assertions: viewerProof }, null, 2)}\n`,
);
console.log(`Rendered export audit artifacts verified in ${auditDir}`);

function collectViewerProof(issues, assertions) {
  const html = readTextArtifact("rendered-export-audit.html");
  assertContains(assertions, issues, "html", html, [
    "class=\"cover\"",
    "APPROVED",
    "Control summary",
    "transform-chart",
    "Architecture diagram",
    "export-comments",
    "AI Provenance",
    "Legal Disclaimer",
  ]);

  const pdfText = readFileSync(join(auditDir, "rendered-export-audit.pdf"), "latin1");
  assertContains(assertions, issues, "pdf-text", pdfText, [
    "Cover: Rendered Export Audit",
    "Watermark: APPROVED",
    "Page 1 of 6",
    "Review Comments",
    "AI Provenance",
    "Legal Disclaimer",
  ]);

  const docx = join(auditDir, "rendered-export-audit.docx");
  const docxEntries = listZipEntries(docx);
  assertEntries(assertions, issues, "docx-package", docxEntries, [
    "word/document.xml",
    "word/header1.xml",
    "word/footer1.xml",
    "word/comments.xml",
    "docProps/core.xml",
    "docProps/custom.xml",
  ]);
  assertContains(assertions, issues, "docx-document", readZipEntryText(docx, "word/document.xml"), [
    "Cover: Rendered Export Audit",
    "Table: tbl:controls: Control summary",
    "Architecture diagram",
    "Review Comments",
    "AI Provenance",
    "Legal Disclaimer",
  ]);
  assertContains(assertions, issues, "docx-custom-properties", readZipEntryText(docx, "docProps/custom.xml"), [
    "NEditorApprovedBy",
    "Release QA",
    "NEditorLegalDisclaimer",
    "For rendered export audit only.",
  ]);

  const pptx = join(auditDir, "rendered-export-audit.pptx");
  const pptxEntries = listZipEntries(pptx);
  assertEntries(assertions, issues, "pptx-package", pptxEntries, [
    "ppt/presentation.xml",
    "ppt/slides/slide1.xml",
    "ppt/slides/slide3.xml",
    "docProps/custom.xml",
  ]);
  const pptxSlides = pptxEntries
    .filter((entry) => /^ppt\/slides\/slide\d+\.xml$/.test(entry))
    .map((entry) => readZipEntryText(pptx, entry))
    .join("\n");
  assertContains(assertions, issues, "pptx-slides", pptxSlides, [
    "Rendered Export Audit",
    "Control summary",
    "Export manifest",
    "Review Comments",
    "AI Provenance",
    "Watermark: APPROVED",
  ]);

  const markdownBundle = join(auditDir, "rendered-export-audit.markdown-bundle.zip");
  assertEntries(assertions, issues, "markdown-bundle-package", listZipEntries(markdownBundle), [
    "document.md",
    "manifest.json",
    "document-ast.json",
    "transform-artifacts.json",
    "media/image1.svg",
  ]);
  assertContains(assertions, issues, "markdown-bundle-manifest", readZipEntryText(markdownBundle, "manifest.json"), [
    "chart-7c90163b489345d0419c71dceb12fef4b97a7bc06da83500852fceab86bf2011",
    "Export Confidence",
    "markdown-bundle",
  ]);

  const blog = join(auditDir, "rendered-export-audit.blog.zip");
  assertEntries(assertions, issues, "blog-package", listZipEntries(blog), [
    "post.md",
    "post.html",
    "substack-copy.html",
    "post.txt",
    "metadata.json",
    "rss-item.xml",
    "manifest.json",
  ]);
  assertContains(assertions, issues, "blog-metadata", readZipEntryText(blog, "metadata.json"), [
    "\"exportTarget\": \"blog\"",
    "\"packageType\": \"publishing-handoff\"",
    "\"primaryPublishFile\": \"post.html\"",
    "\"publishingSteps\":",
    "\"ready\": true",
    "\"slug\": \"rendered-export-audit\"",
    "\"status\": \"approved\"",
  ]);
  assertContains(assertions, issues, "blog-post", readZipEntryText(blog, "post.html"), [
    "Rendered Export Audit",
    "Control summary",
    "Architecture diagram",
    "Rendered artifact audit passed",
  ]);
  assertContains(assertions, issues, "blog-text-fallback", readZipEntryText(blog, "post.txt"), [
    "Review Comments",
    "AI Provenance",
    "Legal Disclaimer",
  ]);
  assertContains(assertions, issues, "blog-rss", readZipEntryText(blog, "rss-item.xml"), [
    "<title>Rendered Export Audit</title>",
    "<description>Board package</description>",
  ]);

  const substack = join(auditDir, "rendered-export-audit.substack.zip");
  assertEntries(assertions, issues, "substack-package", listZipEntries(substack), [
    "post.md",
    "post.html",
    "substack-copy.html",
    "post.txt",
    "metadata.json",
    "manifest.json",
  ]);
  assertContains(assertions, issues, "substack-metadata", readZipEntryText(substack, "metadata.json"), [
    "\"exportTarget\": \"substack\"",
    "\"primaryPublishFile\": \"substack-copy.html\"",
    "\"publishingSteps\":",
    "\"ready\": true",
    "\"slug\": \"rendered-export-audit\"",
  ]);
  assertContains(assertions, issues, "substack-copy", readZipEntryText(substack, "substack-copy.html"), [
    "<article>",
    "Rendered Export Audit",
    "Control summary",
    "Architecture diagram",
    "Rendered artifact audit passed",
  ]);
  assertContains(assertions, issues, "substack-text-fallback", readZipEntryText(substack, "post.txt"), [
    "Review Comments",
    "AI Provenance",
    "Legal Disclaimer",
  ]);

  assertContains(assertions, issues, "latex-source", readTextArtifact("rendered-export-audit.tex"), [
    "\\documentclass[11pt]{article}",
    "\\title{Rendered Export Audit}",
    "\\author{Release QA}",
    "\\caption{Control summary}\\label{tbl:controls}",
    "\\label{fig:architecture}",
    "\\subsection*{AI Provenance}",
  ]);

  const googleDocs = join(auditDir, "rendered-export-audit.google-docs.zip");
  const googleDocsEntries = readZipEntries(googleDocs);
  assertEntries(assertions, issues, "google-docs-package", Array.from(googleDocsEntries.keys()), [
    "document.docx",
    "document.html",
    "document.md",
    "document.txt",
    "metadata.json",
    "manifest.json",
    "README.md",
  ]);
  assertContains(assertions, issues, "google-docs-html", readZipEntryText(googleDocs, "document.html"), [
    "Rendered Export Audit",
    "Control summary",
    "AI Provenance",
  ]);
  assertContains(assertions, issues, "google-docs-metadata", readZipEntryText(googleDocs, "metadata.json"), [
    "\"exportTarget\": \"google-docs\"",
    "\"packageType\": \"google-docs-import-handoff\"",
    "\"primaryImportFile\": \"document.docx\"",
    "\"importSteps\":",
    "\"ready\": true",
    "\"importHint\": \"Upload document.docx to Google Docs",
    "\"version\": \"3.1.4\"",
  ]);
  assertContains(assertions, issues, "google-docs-readme", readZipEntryText(googleDocs, "README.md"), [
    "primary file to upload or convert in Google Docs",
    "## Import Workflow",
    "document.docx",
    "manifest.json",
  ]);
  const googleDocsDocx = googleDocsEntries.get("document.docx");
  assertEntries(assertions, issues, "google-docs-docx-package", listZipBufferEntries(googleDocsDocx), [
    "word/document.xml",
    "word/header1.xml",
    "word/footer1.xml",
    "docProps/core.xml",
    "docProps/custom.xml",
  ]);
  assertContains(assertions, issues, "google-docs-docx-document", readZipBufferEntryText(googleDocsDocx, "word/document.xml"), [
    "Rendered Export Audit",
    "Control summary",
    "Architecture diagram",
    "AI Provenance",
  ]);
}

function collectMacQuickLookProof(issues, assertions) {
  if (process.platform !== "darwin") return;
  const qlmanage = spawnSync("qlmanage", ["-h"], { stdio: "ignore" });
  if (qlmanage.status !== 0) {
    assertions.push({ scope: "macos-quicklook-pdf", assertion: "qlmanage available", passed: false });
    issues.push("qlmanage is unavailable for macOS Quick Look PDF proof");
    return;
  }
  const quicklookDir = join(auditDir, "quicklook");
  rmSync(quicklookDir, { recursive: true, force: true });
  mkdirSync(quicklookDir, { recursive: true });
  const result = spawnSync(
    "qlmanage",
    ["-t", "-s", "900", "-o", quicklookDir, join(auditDir, "rendered-export-audit.pdf")],
    {
      encoding: "utf8",
      timeout: 15_000,
    },
  );
  const output = [result.stdout?.trim(), result.stderr?.trim()].filter(Boolean).join("\n");
  const thumbnail = join(quicklookDir, "rendered-export-audit.pdf.png");
  if (output.includes("sandbox initialization failed: Operation not permitted")) {
    assertions.push({
      scope: "macos-quicklook-pdf",
      assertion: "renders PDF thumbnail through Quick Look",
      passed: false,
      skipped: true,
      reason: "qlmanage cannot initialize its sandbox when launched from this Node verifier on the current host",
    });
    return;
  }
  const passed =
    result.status === 0 &&
    existsSync(thumbnail) &&
    statSync(thumbnail).isFile() &&
    statSync(thumbnail).size > 10_000;
  assertions.push({
    scope: "macos-quicklook-pdf",
    assertion: "renders PDF thumbnail through Quick Look",
    passed,
    thumbnail: relativeToAudit(thumbnail),
    bytes: existsSync(thumbnail) ? statSync(thumbnail).size : 0,
  });
  if (!passed) {
    issues.push(`macOS Quick Look did not render a meaningful PDF thumbnail${output ? `:\n${output}` : ""}`);
  }
}

function collectMacTextutilProof(issues, assertions) {
  if (process.platform !== "darwin") return;
  const result = spawnSync(
    "textutil",
    ["-convert", "txt", "-stdout", join(auditDir, "rendered-export-audit.docx")],
    {
      encoding: "utf8",
      timeout: 15_000,
    },
  );
  if (result.status !== 0) {
    assertions.push({
      scope: "macos-textutil-docx",
      assertion: "extracts DOCX text through macOS textutil",
      passed: false,
      stderr: result.stderr?.trim() || "",
    });
    issues.push("macOS textutil failed to extract rendered-export-audit.docx");
    return;
  }
  assertContains(assertions, issues, "macos-textutil-docx", result.stdout ?? "", [
    "Rendered Export Audit",
    "Control summary",
    "Review Comments",
    "AI Provenance",
    "Legal Disclaimer",
  ]);
}

function readTextArtifact(file) {
  return readFileSync(join(auditDir, file), "utf8");
}

function assertContains(assertions, issues, scope, text, needles) {
  for (const needle of needles) {
    const passed = text.includes(needle);
    assertions.push({ scope, assertion: `contains ${needle}`, passed });
    if (!passed) issues.push(`${scope} is missing expected text: ${needle}`);
  }
}

function assertEntries(assertions, issues, scope, entries, expectedEntries) {
  const entrySet = new Set(entries);
  for (const entry of expectedEntries) {
    const passed = entrySet.has(entry);
    assertions.push({ scope, assertion: `contains ${entry}`, passed });
    if (!passed) issues.push(`${scope} is missing expected package entry: ${entry}`);
  }
}

function listZipEntries(path) {
  return Array.from(readZipEntries(path).keys());
}

function listZipBufferEntries(buffer) {
  return Array.from(readZipEntries(buffer).keys());
}

function readZipEntryText(path, entryName) {
  const entries = readZipEntries(path);
  const entry = entries.get(entryName);
  if (!entry) throw new Error(`missing ZIP entry ${entryName} in ${path}`);
  return entry.toString("utf8");
}

function readZipBufferEntryText(buffer, entryName) {
  const entries = readZipEntries(buffer);
  const entry = entries.get(entryName);
  if (!entry) throw new Error(`missing ZIP entry ${entryName} in nested ZIP`);
  return entry.toString("utf8");
}

function readZipEntries(pathOrBuffer) {
  const buffer = Buffer.isBuffer(pathOrBuffer) ? pathOrBuffer : readFileSync(pathOrBuffer);
  const entries = new Map();
  let offset = 0;
  while (offset + 30 <= buffer.length) {
    const signature = buffer.readUInt32LE(offset);
    if (signature !== 0x04034b50) break;
    const method = buffer.readUInt16LE(offset + 8);
    const compressedSize = buffer.readUInt32LE(offset + 18);
    const fileNameLength = buffer.readUInt16LE(offset + 26);
    const extraLength = buffer.readUInt16LE(offset + 28);
    const nameStart = offset + 30;
    const nameEnd = nameStart + fileNameLength;
    const dataStart = nameEnd + extraLength;
    const dataEnd = dataStart + compressedSize;
    const name = buffer.subarray(nameStart, nameEnd).toString("utf8");
    const compressed = buffer.subarray(dataStart, dataEnd);
    if (!name.endsWith("/")) {
      entries.set(name, unzipEntryData(method, compressed, name));
    }
    offset = dataEnd;
  }
  return entries;
}

function relativeToAudit(path) {
  return path.startsWith(auditDir) ? path.slice(auditDir.length + 1) : path;
}

function unzipEntryData(method, data, name) {
  if (method === 0) return data;
  if (method === 8) return inflateRawSync(data);
  throw new Error(`unsupported ZIP compression method ${method} for ${name}`);
}
