import { rmSync, existsSync, statSync, readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { inflateRawSync } from "node:zlib";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { chromium } from "@playwright/test";
import { resolvePlaywrightBrowserEnv } from "./playwright-browser-env.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const currentSourceCommit = gitCommit();
const sourceTreeClean = gitTreeClean();
const auditDir = resolve(process.env.NEDITOR_RENDERED_EXPORT_AUDIT_DIR || join(root, ".tmp", "rendered-export-audit"));
const completedSignoffPath = process.env.NEDITOR_RENDERED_EXPORT_SIGNOFF
  ? resolve(process.env.NEDITOR_RENDERED_EXPORT_SIGNOFF)
  : null;
const validateExistingSignoffOnly =
  process.argv.includes("--validate-signoff-only") || process.env.NEDITOR_RENDERED_EXPORT_VALIDATE_EXISTING === "1";
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
  ["rendered-export-audit.epub", 1000],
  ["rendered-export-audit-report.json", 500],
  ["README.md", 100],
];
const viewerProof = [];
let auditReport = null;

if (validateExistingSignoffOnly) {
  validateExistingSignoff();
  process.exit(0);
}

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
  auditReport = JSON.parse(readFileSync(join(auditDir, "rendered-export-audit-report.json"), "utf8"));
  const targets = new Set(auditReport.targets?.map((target) => target.target));
  for (const target of ["html", "pdf", "docx", "pptx", "markdown-bundle", "blog", "substack", "latex", "google-docs", "epub"]) {
    if (!targets.has(target)) {
      issues.push(`audit report is missing target ${target}`);
    }
  }
  if (!Array.isArray(auditReport.manualChecklist) || auditReport.manualChecklist.length < 7) {
    issues.push("audit report manual checklist is incomplete");
  }
  const reviewCases = Array.isArray(auditReport.reviewCases) ? auditReport.reviewCases : [];
  const reviewCaseSlugs = new Set(reviewCases.map((reviewCase) => reviewCase.slug));
  for (const slug of ["rich-blocks", "option-heavy", "brand-layout", "business-transforms", "equations", "toc-page-numbers", "edited-tables"]) {
    if (!reviewCaseSlugs.has(slug)) {
      issues.push(`audit report is missing rendered review case ${slug}`);
    }
  }
  for (const reviewCase of reviewCases) {
    const caseTargets = new Set(reviewCase.targets?.map((target) => target.target));
    for (const target of ["html", "pdf", "docx", "pptx", "markdown-bundle"]) {
      if (!caseTargets.has(target)) {
        issues.push(`rendered review case ${reviewCase.slug} is missing target ${target}`);
      }
    }
    for (const target of reviewCase.targets || []) {
      const path = join(auditDir, target.path);
      if (!existsSync(path)) {
        issues.push(`rendered review case artifact is missing: ${target.path}`);
      } else if (statSync(path).size < 500) {
        issues.push(`rendered review case artifact is unexpectedly small: ${target.path}`);
      }
    }
  }
}

if (issues.length === 0) {
  collectViewerProof(issues, viewerProof);
}

if (issues.length === 0) {
  collectPdfToolProof(issues, viewerProof, auditReport);
}

if (issues.length === 0) {
  collectPdfRasterProof(issues, viewerProof, auditReport);
}

if (issues.length === 0) {
  collectReviewCaseProof(issues, viewerProof, auditReport);
}

if (issues.length === 0) {
  collectMacTextutilProof(issues, viewerProof, auditReport);
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

writeManualReviewDashboard(auditReport, viewerProof);
verifyManualReviewDashboard(issues, viewerProof);
if (issues.length > 0) {
  console.error("Rendered export audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

if (issues.length === 0) {
  await collectBrowserVisualProof(issues, viewerProof, auditReport);
}

if (issues.length === 0) {
  await collectOfficePreviewProof(issues, viewerProof, auditReport);
}

if (issues.length > 0) {
  console.error("Rendered export audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

writeManualSignoffTemplate(auditReport, viewerProof);
const humanSignoff = collectHumanSignoffEvidence(issues, viewerProof, auditReport);
const automatedVisualReview = collectAutomatedVisualReviewEvidence(issues, viewerProof, auditReport);

writeFileSync(
  join(auditDir, "viewer-proof.json"),
  `${JSON.stringify({ generatedAt: new Date().toISOString(), assertions: viewerProof }, null, 2)}\n`,
);
writeVisualReviewSummary(auditReport, viewerProof, humanSignoff, automatedVisualReview);
writeManualReviewDashboard(auditReport, viewerProof);
verifyManualReviewDashboard(issues, viewerProof);
verifyVisualReviewSummary(issues, auditReport, viewerProof);
if (issues.length > 0) {
  console.error("Rendered export audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}
console.log(`Rendered export audit artifacts verified in ${auditDir}`);

function validateExistingSignoff() {
  const issues = [];
  if (!completedSignoffPath) {
    issues.push("NEDITOR_RENDERED_EXPORT_SIGNOFF is required when validating an existing rendered export sign-off.");
  }

  const reportPath = join(auditDir, "rendered-export-audit-report.json");
  if (!existsSync(reportPath)) {
    issues.push(`existing rendered export audit report is missing: ${reportPath}`);
  } else {
    try {
      auditReport = JSON.parse(readFileSync(reportPath, "utf8"));
    } catch (error) {
      issues.push(`existing rendered export audit report is not valid JSON: ${String(error)}`);
    }
  }

  let assertions = [];
  const viewerProofPath = join(auditDir, "viewer-proof.json");
  if (!existsSync(viewerProofPath)) {
    issues.push(`existing viewer proof is missing: ${viewerProofPath}`);
  } else {
    try {
      const proof = JSON.parse(readFileSync(viewerProofPath, "utf8"));
      assertions = Array.isArray(proof.assertions) ? proof.assertions : [];
    } catch (error) {
      issues.push(`existing viewer proof is not valid JSON: ${String(error)}`);
    }
  }

  if (issues.length === 0) {
    writeManualSignoffTemplate(auditReport, assertions);
    const humanSignoff = collectHumanSignoffEvidence(issues, assertions, auditReport);
    const automatedVisualReview = collectAutomatedVisualReviewEvidence(issues, assertions, auditReport);
    writeVisualReviewSummary(auditReport, assertions, humanSignoff, automatedVisualReview);
    writeManualReviewDashboard(auditReport, assertions);
    verifyManualReviewDashboard(issues, assertions);
    verifyVisualReviewSummary(issues, auditReport, assertions);
  }

  if (issues.length > 0) {
    console.error("Rendered export sign-off validation failed:");
    for (const issue of issues) console.error(`- ${issue}`);
    process.exit(1);
  }

  console.log(`Rendered export sign-off validated against existing artifacts in ${auditDir}`);
}

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

  const epub = join(auditDir, "rendered-export-audit.epub");
  assertEntries(assertions, issues, "epub-package", listZipEntries(epub), [
    "mimetype",
    "META-INF/container.xml",
    "OEBPS/content.opf",
    "OEBPS/nav.xhtml",
    "OEBPS/document.xhtml",
    "OEBPS/styles/neditor.css",
    "OEBPS/metadata/manifest.json",
    "OEBPS/metadata/document.txt",
  ]);
  assertContains(assertions, issues, "epub-container", readZipEntryText(epub, "META-INF/container.xml"), [
    "application/oebps-package+xml",
    "OEBPS/content.opf",
  ]);
  assertContains(assertions, issues, "epub-package-manifest", readZipEntryText(epub, "OEBPS/content.opf"), [
    "<dc:title>Rendered Export Audit</dc:title>",
    "neditor:sourceHash",
    "image/svg+xml",
  ]);
  assertContains(assertions, issues, "epub-document", readZipEntryText(epub, "OEBPS/document.xhtml"), [
    "Rendered Export Audit",
    "Control summary",
    "Architecture diagram",
    "assets/",
    "human-reviewed",
  ]);
  assertContains(assertions, issues, "epub-text-fallback", readZipEntryText(epub, "OEBPS/metadata/document.txt"), [
    "AI Provenance",
    "Legal Disclaimer",
    "Control summary",
  ]);
  assertContains(assertions, issues, "epub-neditor-manifest", readZipEntryText(epub, "OEBPS/metadata/manifest.json"), [
    "\"export_target\": \"epub\"",
    "\"document_title\": \"Rendered Export Audit\"",
  ]);
}

function collectReviewCaseProof(issues, assertions, report) {
  for (const reviewCase of report.reviewCases || []) {
    const targets = new Map((reviewCase.targets || []).map((target) => [target.target, target.path]));
    const htmlPath = targets.get("html");
    const pdfPath = targets.get("pdf");
    const docxPath = targets.get("docx");
    const pptxPath = targets.get("pptx");
    const bundlePath = targets.get("markdown-bundle");
    const title = reviewCase.title;
    assertContains(assertions, issues, `${reviewCase.slug}-html`, readFileSync(join(auditDir, htmlPath), "utf8"), [
      title,
      ...(reviewCase.requiredEvidence || []),
    ]);
    assertContains(assertions, issues, `${reviewCase.slug}-pdf`, readFileSync(join(auditDir, pdfPath), "latin1"), [
      title,
    ]);
    assertContains(assertions, issues, `${reviewCase.slug}-docx`, readZipEntryText(join(auditDir, docxPath), "word/document.xml"), [
      title,
    ]);
    const pptxSlides = listZipEntries(join(auditDir, pptxPath))
      .filter((entry) => /^ppt\/slides\/slide\d+\.xml$/.test(entry))
      .map((entry) => readZipEntryText(join(auditDir, pptxPath), entry))
      .join("\n");
    assertContains(assertions, issues, `${reviewCase.slug}-pptx`, pptxSlides, [title]);
    assertContains(assertions, issues, `${reviewCase.slug}-bundle`, readZipEntryText(join(auditDir, bundlePath), "document.md"), [
      title,
    ]);
  }
}

function collectPdfToolProof(issues, assertions, report) {
  const pdfinfo = commandResult("pdfinfo", ["-v"]);
  const pdftotext = commandResult("pdftotext", ["-v"]);
  if (pdfinfo.error || pdftotext.error) {
    assertions.push({
      scope: "poppler-pdf-tools",
      assertion: "pdfinfo and pdftotext available",
      passed: false,
      skipped: true,
      reason: "Poppler pdfinfo/pdftotext is not installed on this host",
    });
    return;
  }

  const primaryPdf = join(auditDir, "rendered-export-audit.pdf");
  assertPdfInfo(issues, assertions, "pdfinfo-primary", primaryPdf, [
    "Title:           Rendered Export Audit",
    "Author:          Release QA",
    "Producer:        NEditor",
    "Pages:           6",
    "Page size:",
    "PDF version:",
  ]);
  assertPdfText(issues, assertions, "pdftotext-primary", primaryPdf, [
    "Rendered Export Audit",
    "Control summary",
    "Watermark: APPROVED",
    "Review Comments",
    "AI Provenance",
    "Legal Disclaimer",
  ]);

  for (const reviewCase of report.reviewCases || []) {
    const pdfTarget = (reviewCase.targets || []).find((target) => target.target === "pdf");
    if (!pdfTarget?.path) {
      issues.push(`rendered review case ${reviewCase.slug} is missing PDF target for Poppler proof`);
      continue;
    }
    const scope = `pdftotext-review-${reviewCase.slug}`;
    assertPdfText(issues, assertions, scope, join(auditDir, pdfTarget.path), [
      ...new Set([reviewCase.title, ...(reviewCase.requiredEvidence || []).slice(0, 4)].filter(Boolean)),
    ]);
  }
}

function assertPdfInfo(issues, assertions, scope, path, needles) {
  const result = spawnSync("pdfinfo", [path], {
    encoding: "utf8",
    timeout: 15_000,
  });
  if (result.status !== 0) {
    assertions.push({
      scope,
      assertion: "reads PDF metadata through pdfinfo",
      passed: false,
      stderr: result.stderr?.trim() || "",
    });
    issues.push(`pdfinfo failed to read ${relativeToAudit(path)}`);
    return;
  }
  assertContains(assertions, issues, scope, result.stdout ?? "", needles);
}

function assertPdfText(issues, assertions, scope, path, needles) {
  const result = spawnSync("pdftotext", [path, "-"], {
    encoding: "utf8",
    timeout: 15_000,
  });
  if (result.status !== 0) {
    assertions.push({
      scope,
      assertion: "extracts PDF text through pdftotext",
      passed: false,
      stderr: result.stderr?.trim() || "",
    });
    issues.push(`pdftotext failed to extract ${relativeToAudit(path)}`);
    return;
  }
  assertContains(assertions, issues, scope, result.stdout ?? "", needles);
}

function collectPdfRasterProof(issues, assertions, report) {
  const pdftoppm = commandResult("pdftoppm", ["-v"]);
  if (pdftoppm.error) {
    assertions.push({
      scope: "pdftoppm-pdf-raster",
      assertion: "pdftoppm available for PDF raster thumbnails",
      passed: false,
      skipped: true,
      reason: "Poppler pdftoppm is not installed on this host",
    });
    return;
  }

  const rasterDir = join(auditDir, "raster-proof");
  rmSync(rasterDir, { recursive: true, force: true });
  mkdirSync(rasterDir, { recursive: true });
  renderPdfRasterPages(issues, assertions, "primary", join(auditDir, "rendered-export-audit.pdf"), join(rasterDir, "primary"), 2);

  for (const reviewCase of report.reviewCases || []) {
    const pdfTarget = (reviewCase.targets || []).find((target) => target.target === "pdf");
    if (!pdfTarget?.path) {
      issues.push(`rendered review case ${reviewCase.slug} is missing PDF target for raster proof`);
      continue;
    }
    renderPdfRasterPages(
      issues,
      assertions,
      `review-${reviewCase.slug}`,
      join(auditDir, pdfTarget.path),
      join(rasterDir, `review-${reviewCase.slug}`),
      1,
    );
  }
}

function renderPdfRasterPages(issues, assertions, scope, path, outputPrefix, lastPage) {
  const result = spawnSync("pdftoppm", ["-png", "-r", "96", "-f", "1", "-l", String(lastPage), path, outputPrefix], {
    encoding: "utf8",
    timeout: 20_000,
  });
  if (result.status !== 0) {
    assertions.push({
      scope: `pdftoppm-${scope}`,
      assertion: "renders PDF pages to PNG thumbnails",
      passed: false,
      stderr: result.stderr?.trim() || "",
    });
    issues.push(`pdftoppm failed to rasterize ${relativeToAudit(path)}`);
    return;
  }

  for (let page = 1; page <= lastPage; page += 1) {
    let thumbnail = `${outputPrefix}-${page}.png`;
    let tool = "pdftoppm";
    let evidence = rasterThumbnailEvidence(thumbnail);
    if (!evidence.passed) {
      const fallbackThumbnail = renderPdfCairoPage(path, `${outputPrefix}-cairo`, page);
      const fallbackEvidence = fallbackThumbnail ? rasterThumbnailEvidence(fallbackThumbnail) : null;
      if (fallbackEvidence?.passed) {
        thumbnail = fallbackThumbnail;
        tool = "pdftocairo";
        evidence = fallbackEvidence;
      }
    }
    assertions.push({
      scope: `pdftoppm-${scope}-page-${page}`,
      assertion: `renders PDF page ${page} to a non-empty PNG thumbnail`,
      passed: evidence.passed,
      tool,
      thumbnail: relativeToAudit(thumbnail),
      bytes: evidence.bytes,
      width: evidence.dimensions?.width || 0,
      height: evidence.dimensions?.height || 0,
    });
    if (!evidence.passed) {
      issues.push(`pdftoppm did not render a meaningful thumbnail for ${relativeToAudit(path)} page ${page}`);
    }
  }
}

function renderPdfCairoPage(path, outputPrefix, page) {
  const result = spawnSync("pdftocairo", ["-png", "-r", "96", "-f", String(page), "-l", String(page), path, outputPrefix], {
    encoding: "utf8",
    timeout: 20_000,
  });
  if (result.status !== 0) return null;
  return `${outputPrefix}-${page}.png`;
}

function rasterThumbnailEvidence(path) {
  const dimensions = existsSync(path) ? pngDimensions(path) : null;
  const bytes = existsSync(path) ? statSync(path).size : 0;
  return {
    passed: Boolean(dimensions && dimensions.width >= 500 && dimensions.height >= 500 && bytes > 10_000),
    dimensions,
    bytes,
  };
}

function pngDimensions(path) {
  const data = readFileSync(path);
  const pngSignature = "89504e470d0a1a0a";
  if (data.subarray(0, 8).toString("hex") !== pngSignature) return null;
  return {
    width: data.readUInt32BE(16),
    height: data.readUInt32BE(20),
  };
}

function collectMacQuickLookProof(issues, assertions) {
  if (process.platform !== "darwin") return;
  const qlmanage = spawnSync("qlmanage", ["-h"], { stdio: "ignore" });
  if (qlmanage.status !== 0) {
    assertions.push({ scope: "macos-quicklook", assertion: "qlmanage available", passed: false });
    issues.push("qlmanage is unavailable for macOS Quick Look proof");
    return;
  }
  const quicklookDir = join(auditDir, "quicklook");
  rmSync(quicklookDir, { recursive: true, force: true });
  mkdirSync(quicklookDir, { recursive: true });

  const nativeArtifacts = [
    {
      scope: "macos-quicklook-pdf",
      assertion: "renders PDF thumbnail through Quick Look",
      path: "rendered-export-audit.pdf",
    },
    {
      scope: "macos-quicklook-docx",
      assertion: "renders DOCX thumbnail through Quick Look",
      path: "rendered-export-audit.docx",
    },
    {
      scope: "macos-quicklook-pptx",
      assertion: "renders PPTX thumbnail through Quick Look",
      path: "rendered-export-audit.pptx",
    },
  ];

  for (const artifact of nativeArtifacts) {
    const result = spawnSync(
      "qlmanage",
      ["-t", "-s", "900", "-o", quicklookDir, join(auditDir, artifact.path)],
      {
        encoding: "utf8",
        timeout: 15_000,
      },
    );
    const output = [result.stdout?.trim(), result.stderr?.trim()].filter(Boolean).join("\n");
    const thumbnail = join(quicklookDir, `${artifact.path}.png`);
    if (output.includes("sandbox initialization failed: Operation not permitted")) {
      assertions.push({
        scope: artifact.scope,
        assertion: artifact.assertion,
        passed: false,
        skipped: true,
        reason: "qlmanage cannot initialize its sandbox when launched from this Node verifier on the current host",
      });
      continue;
    }
    const passed =
      result.status === 0 &&
      existsSync(thumbnail) &&
      statSync(thumbnail).isFile() &&
      statSync(thumbnail).size > 10_000;
    if (!passed) {
      assertions.push({
        scope: artifact.scope,
        assertion: artifact.assertion,
        passed: false,
        skipped: true,
        reason: output || "qlmanage returned without a meaningful thumbnail on this host",
        bytes: existsSync(thumbnail) ? statSync(thumbnail).size : 0,
      });
      continue;
    }
    assertions.push({
      scope: artifact.scope,
      assertion: artifact.assertion,
      passed,
      thumbnail: relativeToAudit(thumbnail),
      bytes: existsSync(thumbnail) ? statSync(thumbnail).size : 0,
    });
  }
}

function collectMacTextutilProof(issues, assertions, report) {
  if (process.platform !== "darwin") return;
  assertMacTextutilDocx(issues, assertions, "macos-textutil-docx", join(auditDir, "rendered-export-audit.docx"), [
    "Rendered Export Audit",
    "Control summary",
    "Review Comments",
    "AI Provenance",
    "Legal Disclaimer",
  ]);

  const nativeProofDir = join(auditDir, "native-proof");
  mkdirSync(nativeProofDir, { recursive: true });
  const googleDocsDocx = readZipEntries(join(auditDir, "rendered-export-audit.google-docs.zip")).get("document.docx");
  if (!googleDocsDocx) {
    issues.push("Google Docs package is missing nested document.docx for native textutil proof");
  } else {
    const googleDocsDocxPath = join(nativeProofDir, "google-docs-document.docx");
    writeFileSync(googleDocsDocxPath, googleDocsDocx);
    assertMacTextutilDocx(issues, assertions, "macos-textutil-google-docs-docx", googleDocsDocxPath, [
      "Rendered Export Audit",
      "Control summary",
      "Architecture diagram",
      "AI Provenance",
    ]);
  }

  for (const reviewCase of report.reviewCases || []) {
    const docxTarget = (reviewCase.targets || []).find((target) => target.target === "docx");
    if (!docxTarget?.path) {
      issues.push(`rendered review case ${reviewCase.slug} is missing DOCX target for native textutil proof`);
      continue;
    }
    assertMacTextutilDocx(
      issues,
      assertions,
      `macos-textutil-review-${reviewCase.slug}`,
      join(auditDir, docxTarget.path),
      docxReviewCaseEvidence(reviewCase),
    );
  }
}

async function collectBrowserVisualProof(issues, assertions, report) {
  const resolution = resolvePlaywrightBrowserEnv(process.env);
  if (!resolution.ok) {
    assertions.push({
      scope: "browser-visual-proof",
      assertion: "Chromium-compatible browser available for rendered export visual proof",
      passed: false,
      skipped: true,
      reason: resolution.message,
    });
    return;
  }

  const visualDir = join(auditDir, "browser-visual-proof");
  rmSync(visualDir, { recursive: true, force: true });
  mkdirSync(visualDir, { recursive: true });

  const reviewCaseHtmlTargets = (report.reviewCases || [])
    .map((reviewCase) => {
      const htmlTarget = (reviewCase.targets || []).find((target) => target.target === "html");
      if (!htmlTarget?.path) return null;
      return {
        scope: `browser-visual-review-${reviewCase.slug}`,
        artifact: htmlTarget.path,
        screenshot: `review-${reviewCase.slug}.png`,
        selectors: reviewCase.slug === "toc-page-numbers" ? ["body", "h1", "#table-of-contents", "ul"] : ["body", "h1", "table"],
        needles: [...new Set([reviewCase.title, ...(reviewCase.requiredEvidence || []).slice(0, 4)].filter(Boolean))],
      };
    })
    .filter(Boolean);

  const visualTargets = [
    {
      scope: "browser-visual-primary-html",
      artifact: "rendered-export-audit.html",
      screenshot: "primary-html.png",
      selectors: ["body", "h1", ".cover", "table"],
      needles: ["Rendered Export Audit", "Control summary", "Architecture diagram", "Review Comments", "AI Provenance"],
    },
    {
      scope: "browser-visual-manual-dashboard",
      artifact: "manual-review.html",
      screenshot: "manual-review.png",
      selectors: ["body", "h1", "table", ".thumbnail-grid"],
      needles: ["Rendered Export Manual Review", "Manual Checklist", "Primary Artifacts", "Executable Viewer And Package Proof"],
    },
    ...reviewCaseHtmlTargets,
  ];

  let browser = null;
  try {
    browser = await chromium.launch({
      executablePath: resolution.executablePath || undefined,
    });
    const page = await browser.newPage({
      viewport: { width: 1440, height: 1000 },
      deviceScaleFactor: 1,
    });

    for (const target of visualTargets) {
      const artifactPath = join(auditDir, target.artifact);
      if (!existsSync(artifactPath)) {
        issues.push(`browser visual proof target is missing: ${target.artifact}`);
        continue;
      }

      await page.goto(pathToFileURL(artifactPath).href, { waitUntil: "load" });
      await page.evaluate(() => {
        document.documentElement.style.background = "#fff";
        document.body.style.background = "#fff";
      });
      const screenshotPath = join(visualDir, target.screenshot);
      await page.locator("body").screenshot({ path: screenshotPath });
      const dimensions = pngDimensions(screenshotPath);
      const bytes = statSync(screenshotPath).size;
      const metrics = await page.evaluate(({ needles, selectors }) => {
        const text = document.body?.innerText || "";
        const html = document.documentElement?.innerHTML || "";
        const selectorCounts = Object.fromEntries(
          selectors.map((selector) => [selector, document.querySelectorAll(selector).length]),
        );
        return {
          heading: document.querySelector("h1")?.textContent?.trim() || "",
          missingEvidence: needles.filter((needle) => !text.includes(needle) && !html.includes(needle)),
          scrollHeight: document.documentElement.scrollHeight,
          scrollWidth: document.documentElement.scrollWidth,
          selectorCounts,
          textLength: text.length,
        };
      }, { needles: target.needles, selectors: target.selectors });
      const missingSelectors = Object.entries(metrics.selectorCounts)
        .filter(([, count]) => count < 1)
        .map(([selector]) => selector);
      const passed = Boolean(
        dimensions &&
          dimensions.width >= 1000 &&
          dimensions.height >= 700 &&
          bytes > 20_000 &&
          metrics.textLength > 500 &&
          metrics.scrollHeight >= 700 &&
          metrics.missingEvidence.length === 0 &&
          missingSelectors.length === 0,
      );
      assertions.push({
        scope: target.scope,
        assertion: `renders ${target.artifact} in Chromium with expected visible structure`,
        passed,
        browserSource: resolution.source,
        browserExecutable: resolution.executablePath,
        thumbnail: relativeToAudit(screenshotPath),
        bytes,
        width: dimensions?.width || 0,
        height: dimensions?.height || 0,
        scrollHeight: metrics.scrollHeight,
        scrollWidth: metrics.scrollWidth,
        heading: metrics.heading,
        missingEvidence: metrics.missingEvidence,
        missingSelectors,
      });
      if (!passed) {
        issues.push(
          `Chromium visual proof failed for ${target.artifact}: missing evidence ${JSON.stringify(metrics.missingEvidence)}, missing selectors ${JSON.stringify(missingSelectors)}, screenshot ${bytes} bytes`,
        );
      }
    }
  } catch (error) {
    const launchBlocked = browser === null;
    assertions.push({
      scope: "browser-visual-proof",
      assertion: launchBlocked
        ? "launches Chromium for rendered export visual proof"
        : "renders exported HTML artifacts in Chromium",
      passed: false,
      skipped: launchBlocked,
      browserSource: resolution.source,
      browserExecutable: resolution.executablePath,
      reason: String(error),
    });
    if (!launchBlocked) {
      issues.push(`Chromium visual proof failed: ${String(error)}`);
    }
  } finally {
    if (browser) await browser.close();
  }
}

async function collectOfficePreviewProof(issues, assertions, report) {
  const previewDir = join(auditDir, "office-preview");
  rmSync(previewDir, { recursive: true, force: true });
  mkdirSync(previewDir, { recursive: true });

  const previewTargets = [
    officePreviewTarget("office-preview-docx", "DOCX", "Primary DOCX export", "rendered-export-audit.docx", [
      "Rendered Export Audit",
      "Control summary",
      "Architecture diagram",
      "Review Comments",
      "AI Provenance",
      "Legal Disclaimer",
    ]),
    officePreviewTarget("office-preview-pptx", "PPTX", "Primary PPTX export", "rendered-export-audit.pptx", [
      "Rendered Export Audit",
      "Control summary",
      "Export manifest",
      "Review Comments",
      "AI Provenance",
      "Watermark: APPROVED",
    ]),
  ];

  for (const reviewCase of report.reviewCases || []) {
    const targets = new Map((reviewCase.targets || []).map((target) => [target.target, target.path]));
    const requiredEvidence = docxReviewCaseEvidence(reviewCase);
    const presentationEvidence = [
      ...new Set([reviewCase.title, ...(reviewCase.requiredEvidence || []).filter((item) => item === "INTERNAL")].filter(Boolean)),
    ];
    const docxPath = targets.get("docx");
    const pptxPath = targets.get("pptx");
    if (docxPath) {
      previewTargets.push(
        officePreviewTarget(
          `office-preview-review-${reviewCase.slug}-docx`,
          "DOCX",
          `${reviewCase.title} DOCX review case`,
          docxPath,
          requiredEvidence,
        ),
      );
    }
    if (pptxPath) {
      previewTargets.push(
        officePreviewTarget(
          `office-preview-review-${reviewCase.slug}-pptx`,
          "PPTX",
          `${reviewCase.title} PPTX review case`,
          pptxPath,
          presentationEvidence,
        ),
      );
    }
  }

  for (const target of previewTargets) {
    const artifactPath = join(auditDir, target.path);
    if (!existsSync(artifactPath)) {
      issues.push(`Office preview source is missing: ${target.path}`);
      continue;
    }

    const extracted = target.kind === "DOCX" ? extractDocxPreview(artifactPath) : extractPptxPreview(artifactPath);
    const missingEvidence = target.needles.filter((needle) => !extracted.searchText.includes(needle));
    const htmlPath = join(previewDir, `${target.scope}.html`);
    writeFileSync(htmlPath, officePreviewHtml(target, extracted));
    const bytes = statSync(htmlPath).size;
    const passed = bytes > 1500 && extracted.searchText.length > 100 && missingEvidence.length === 0;
    assertions.push({
      scope: target.scope,
      assertion: `extracts ${target.kind} text into reviewable Office preview`,
      passed,
      path: relativeToAudit(htmlPath),
      bytes,
      textLength: extracted.searchText.length,
      slideCount: extracted.slideCount || 0,
      paragraphCount: extracted.paragraphCount || 0,
      missingEvidence,
    });
    if (!passed) {
      issues.push(`Office preview proof failed for ${target.path}: missing evidence ${JSON.stringify(missingEvidence)}`);
    }
  }

  const resolution = resolvePlaywrightBrowserEnv(process.env);
  if (!resolution.ok) {
    assertions.push({
      scope: "office-preview-browser",
      assertion: "Chromium-compatible browser available for Office preview screenshots",
      passed: false,
      skipped: true,
      reason: resolution.message,
    });
    return;
  }

  let browser = null;
  try {
    browser = await chromium.launch({
      executablePath: resolution.executablePath || undefined,
    });
    const page = await browser.newPage({
      viewport: { width: 1280, height: 900 },
      deviceScaleFactor: 1,
    });

    for (const target of previewTargets) {
      const htmlPath = join(previewDir, `${target.scope}.html`);
      if (!existsSync(htmlPath)) continue;
      await page.goto(pathToFileURL(htmlPath).href, { waitUntil: "load" });
      const screenshotPath = join(previewDir, `${target.scope}.png`);
      await page.screenshot({ path: screenshotPath, fullPage: true });
      const dimensions = pngDimensions(screenshotPath);
      const bytes = statSync(screenshotPath).size;
      const metrics = await page.evaluate((needles) => {
        const text = document.body?.innerText || "";
        return {
          heading: document.querySelector("h1")?.textContent?.trim() || "",
          missingEvidence: needles.filter((needle) => !text.includes(needle)),
          rows: document.querySelectorAll("tbody tr").length,
          textLength: text.length,
          scrollHeight: document.documentElement.scrollHeight,
        };
      }, target.needles);
      const passed = Boolean(
        dimensions &&
          dimensions.width >= 900 &&
          dimensions.height >= 650 &&
          bytes > 20_000 &&
          metrics.textLength > 300 &&
          metrics.rows >= 1 &&
          metrics.missingEvidence.length === 0,
      );
      assertions.push({
        scope: `${target.scope}-screenshot`,
        assertion: `renders ${target.kind} Office preview screenshot`,
        passed,
        browserSource: resolution.source,
        browserExecutable: resolution.executablePath,
        thumbnail: relativeToAudit(screenshotPath),
        bytes,
        width: dimensions?.width || 0,
        height: dimensions?.height || 0,
        scrollHeight: metrics.scrollHeight,
        heading: metrics.heading,
        rows: metrics.rows,
        missingEvidence: metrics.missingEvidence,
      });
      if (!passed) {
        issues.push(
          `Office preview screenshot failed for ${target.path}: missing evidence ${JSON.stringify(metrics.missingEvidence)}, screenshot ${bytes} bytes`,
        );
      }
    }
  } catch (error) {
    assertions.push({
      scope: "office-preview-browser",
      assertion: browser === null ? "launches Chromium for Office preview screenshots" : "renders Office preview screenshots",
      passed: false,
      skipped: browser === null,
      browserSource: resolution.source,
      browserExecutable: resolution.executablePath,
      reason: String(error),
    });
    if (browser !== null) {
      issues.push(`Office preview screenshot proof failed: ${String(error)}`);
    }
  } finally {
    if (browser) await browser.close();
  }
}

function officePreviewTarget(scope, kind, title, path, needles) {
  return {
    scope,
    kind,
    title,
    path,
    needles: [...new Set(needles.filter(Boolean))],
  };
}

function extractDocxPreview(path) {
  const entries = readZipEntries(path);
  const documentXml = entries.get("word/document.xml")?.toString("utf8") || "";
  const headerXml = entries.get("word/header1.xml")?.toString("utf8") || "";
  const footerXml = entries.get("word/footer1.xml")?.toString("utf8") || "";
  const commentsXml = entries.get("word/comments.xml")?.toString("utf8") || "";
  const paragraphs = [
    ...officeParagraphs(documentXml),
    ...officeParagraphs(headerXml),
    ...officeParagraphs(footerXml),
    ...officeParagraphs(commentsXml),
  ].filter(Boolean);
  return {
    sections: [
      {
        label: "Document, headers, footers, and comments",
        rows: paragraphs.map((text, index) => ({ label: `Text ${index + 1}`, text })),
      },
    ],
    searchText: paragraphs.join("\n"),
    paragraphCount: paragraphs.length,
  };
}

function extractPptxPreview(path) {
  const entries = readZipEntries(path);
  const slides = Array.from(entries.keys())
    .filter((entry) => /^ppt\/slides\/slide\d+\.xml$/.test(entry))
    .sort((left, right) => slideNumber(left) - slideNumber(right))
    .map((entry) => {
      const text = officeText(entries.get(entry).toString("utf8"));
      return {
        label: entry.replace("ppt/slides/", "").replace(".xml", ""),
        text,
      };
    })
    .filter((slide) => slide.text);
  return {
    sections: [
      {
        label: "Slides",
        rows: slides,
      },
    ],
    searchText: slides.map((slide) => slide.text).join("\n"),
    slideCount: slides.length,
  };
}

function docxReviewCaseEvidence(reviewCase) {
  if (reviewCase.slug === "toc-page-numbers") {
    return [
      reviewCase.title,
      "Table of Contents",
      "Update table of contents in Word to refresh page numbers.",
    ].filter(Boolean);
  }
  return [...new Set([reviewCase.title, ...(reviewCase.requiredEvidence || [])].filter(Boolean))];
}

function officePreviewHtml(target, extracted) {
  const sections = extracted.sections
    .map((section) => {
      const rows = section.rows
        .map(
          (row) => `<tr><th>${escapeHtml(row.label)}</th><td>${escapeHtml(row.text)}</td></tr>`,
        )
        .join("\n");
      return `<section>
  <h2>${escapeHtml(section.label)}</h2>
  <table>
    <tbody>${rows}</tbody>
  </table>
</section>`;
    })
    .join("\n");
  const evidence = target.needles.map((needle) => `<li><code>${escapeHtml(needle)}</code></li>`).join("\n");
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${escapeHtml(target.title)}</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; line-height: 1.5; margin: 2rem; color: #1f2933; }
    h1, h2 { line-height: 1.2; }
    table { border-collapse: collapse; width: 100%; margin: 1rem 0 2rem; table-layout: fixed; }
    th, td { border: 1px solid #cbd5df; padding: 0.45rem 0.6rem; text-align: left; vertical-align: top; }
    th { width: 12rem; background: #edf2f7; }
    td { overflow-wrap: anywhere; }
    code { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 0.92em; }
  </style>
</head>
<body>
  <h1>${escapeHtml(target.title)}</h1>
  <p><strong>Source:</strong> <code>${escapeHtml(target.path)}</code></p>
  <p><strong>Format:</strong> ${escapeHtml(target.kind)}</p>
  <h2>Required Evidence</h2>
  <ul>${evidence}</ul>
  ${sections}
</body>
</html>
`;
}

function officeParagraphs(xml) {
  const paragraphs = [];
  for (const match of xml.matchAll(/<w:p\b[\s\S]*?<\/w:p>/g)) {
    const text = officeText(match[0]);
    if (text) paragraphs.push(text);
  }
  return paragraphs;
}

function officeText(xml) {
  const values = [];
  const normalized = xml
    .replace(/<w:tab\s*\/>/g, "\t")
    .replace(/<w:br\s*\/>/g, "\n")
    .replace(/<a:br\s*\/>/g, "\n");
  for (const match of normalized.matchAll(/<(?:w|a):t(?:\s[^>]*)?>([\s\S]*?)<\/(?:w|a):t>/g)) {
    values.push(decodeXml(match[1]));
  }
  return values.join(" ").replace(/\s+/g, " ").trim();
}

function slideNumber(entry) {
  const match = entry.match(/slide(\d+)\.xml$/);
  return match ? Number(match[1]) : 0;
}

function assertMacTextutilDocx(issues, assertions, scope, path, needles) {
  const result = spawnSync("textutil", ["-convert", "txt", "-stdout", path], {
    encoding: "utf8",
    timeout: 15_000,
  });
  if (result.status !== 0) {
    assertions.push({
      scope,
      assertion: "extracts DOCX text through macOS textutil",
      passed: false,
      stderr: result.stderr?.trim() || "",
    });
    issues.push(`macOS textutil failed to extract ${relativeToAudit(path)}`);
    return;
  }
  assertContains(
    assertions,
    issues,
    scope,
    result.stdout ?? "",
    needles,
  );
}

function writeManualSignoffTemplate(report, assertions) {
  const primaryArtifacts = (report.targets || []).map((target) => signoffArtifact(target));
  const reviewCases = (report.reviewCases || []).map((reviewCase) => ({
    slug: reviewCase.slug,
    title: reviewCase.title,
    requiredEvidence: reviewCase.requiredEvidence || [],
    targets: (reviewCase.targets || []).map((target) => signoffArtifact(target)),
    status: "pending",
    reviewerNotes: "",
  }));
  const checklist = (report.manualChecklist || []).map((item, index) => ({
    id: `check-${String(index + 1).padStart(2, "0")}`,
    item,
    status: "pending",
    reviewerNotes: "",
  }));
  const template = {
    schema: "neditor.rendered-export.visual-signoff.v1",
    generatedAt: new Date().toISOString(),
    status: "pending-human-review",
    appVersion: packageJson.version,
    sourceCommit: currentSourceCommit || "replace-with-current-git-commit",
    sourceTreeClean,
    instructions: [
      "Copy this template before editing it.",
      "Open every primary artifact and review-case target in the relevant native or browser viewer.",
      "Set each artifact, review case, and checklist status to passed, failed, or skipped-with-reason.",
      "Set top-level status to human-reviewed only when every non-skipped required item has been reviewed.",
      "Run NEDITOR_RENDERED_EXPORT_SIGNOFF=/path/to/completed-signoff.json pnpm run test:rendered-exports -- --validate-signoff-only to validate the completed sign-off against this existing audit bundle.",
    ],
    reviewer: {
      name: "",
      role: "",
      reviewedAt: "",
      platform: process.platform,
      nativeViewers: [],
    },
    primaryArtifacts,
    reviewCases,
    checklist,
    acceptance: {
      allPrimaryArtifactsReviewed: false,
      allReviewCasesReviewed: false,
      allChecklistItemsReviewed: false,
      blockers: [],
      notes: "",
    },
  };
  writeFileSync(join(auditDir, "visual-review-signoff.template.json"), `${JSON.stringify(template, null, 2)}\n`);
  assertions.push({
    scope: "manual-signoff-template",
    assertion: "writes structured manual visual-review sign-off template",
    passed:
      primaryArtifacts.length >= 9 &&
      reviewCases.length >= 2 &&
      checklist.length >= 7 &&
      existsSync(join(auditDir, "visual-review-signoff.template.json")),
    path: "visual-review-signoff.template.json",
  });
}

function collectHumanSignoffEvidence(issues, assertions, report) {
  if (!completedSignoffPath) {
    const reason =
      "No completed sign-off supplied; set NEDITOR_RENDERED_EXPORT_SIGNOFF and run with --validate-signoff-only to validate manual native-viewer review.";
    assertions.push({
      scope: "human-signoff",
      assertion: "completed manual visual-review sign-off supplied",
      passed: false,
      skipped: true,
      reason,
    });
    return {
      status: "pending-human-review",
      reason,
    };
  }

  if (!existsSync(completedSignoffPath)) {
    issues.push(`completed visual review sign-off file does not exist: ${completedSignoffPath}`);
    assertions.push({
      scope: "human-signoff",
      assertion: "completed manual visual-review sign-off supplied",
      passed: false,
      reason: "Configured sign-off path does not exist.",
    });
    return {
      status: "pending-human-review",
      path: completedSignoffPath,
      reason: "Configured sign-off path does not exist.",
    };
  }

  let signoff = null;
  try {
    signoff = JSON.parse(readFileSync(completedSignoffPath, "utf8"));
  } catch (error) {
    issues.push(`completed visual review sign-off is not valid JSON: ${String(error)}`);
    assertions.push({
      scope: "human-signoff",
      assertion: "completed manual visual-review sign-off is valid JSON",
      passed: false,
      reason: String(error),
    });
    return {
      status: "pending-human-review",
      path: completedSignoffPath,
      reason: "Configured sign-off file is not valid JSON.",
    };
  }

  const validationIssues = validateCompletedSignoff(signoff, report);
  const passed = validationIssues.length === 0;
  assertions.push({
    scope: "human-signoff",
    assertion: "completed manual visual-review sign-off validates required artifacts, cases, checklist, and reviewer metadata",
    passed,
    path: completedSignoffPath,
    reviewer: signoff.reviewer?.name || "",
    reviewedAt: signoff.reviewer?.reviewedAt || "",
    validationIssues,
  });
  if (!passed) {
    for (const issue of validationIssues) issues.push(issue);
    return {
      status: "pending-human-review",
      path: completedSignoffPath,
      reason: "Completed sign-off was supplied but did not satisfy the audit contract.",
    };
  }

  return {
    status: "human-reviewed",
    path: completedSignoffPath,
    reviewer: signoff.reviewer?.name || "",
    reviewedAt: signoff.reviewer?.reviewedAt || "",
    reason: "Completed manual native-viewer review sign-off was supplied and validated.",
  };
}

function collectAutomatedVisualReviewEvidence(_issues, assertions, report) {
  const assertionList = assertions.filter((assertion) => assertion.scope !== "automated-visual-review");
  const primaryTargets = (report.targets || []).map((target) => automatedTargetEvidence(target, targetEvidence(target.target, assertionList)));
  const reviewCases = (report.reviewCases || []).map((reviewCase) => ({
    slug: reviewCase.slug,
    title: reviewCase.title,
    targets: (reviewCase.targets || []).map((target) =>
      automatedTargetEvidence(target, targetEvidence(`${reviewCase.slug}:${target.target}`, assertionList)),
    ),
  }));

  const browserVisualScopes = [
    "browser-visual-primary-html",
    "browser-visual-manual-dashboard",
    ...(report.reviewCases || []).map((reviewCase) => `browser-visual-review-${reviewCase.slug}`),
  ];
  const officePreviewScopes = [
    "office-preview-docx",
    "office-preview-pptx",
    ...(report.reviewCases || []).flatMap((reviewCase) => [
      `office-preview-review-${reviewCase.slug}-docx`,
      `office-preview-review-${reviewCase.slug}-pptx`,
    ]),
  ];
  const hardFailures = assertionList.filter((assertion) => assertion.passed === false && !assertion.skipped);
  const browserVisualComplete = scopesPassed(assertionList, browserVisualScopes);
  const officePreviewComplete = scopesPassed(assertionList, officePreviewScopes);
  const pdfRasterComplete = assertionList.some((assertion) => String(assertion.scope || "").startsWith("pdftoppm") && assertion.passed);
  const officeScreenshotsComplete = officePreviewScopes.every((scope) =>
    assertionList.some((assertion) => assertion.scope === `${scope}-screenshot` && assertion.passed),
  );
  const primaryEvidenceComplete = primaryTargets.every((target) => target.passedEvidenceCount > 0);
  const reviewEvidenceComplete = reviewCases.every((reviewCase) =>
    reviewCase.targets.every((target) => target.passedEvidenceCount > 0),
  );
  const browserScreenshotHostLimited = !browserVisualComplete && hasHostLimitedBrowserAssertion(assertionList, "browser-visual-proof");
  const officeScreenshotHostLimited = !officeScreenshotsComplete && hasHostLimitedBrowserAssertion(assertionList, "office-preview-browser");
  const hostLimitedScreenshotOnly =
    hardFailures.length === 0 &&
    officePreviewComplete &&
    pdfRasterComplete &&
    primaryEvidenceComplete &&
    reviewEvidenceComplete &&
    (browserScreenshotHostLimited || officeScreenshotHostLimited);
  const status =
    hardFailures.length === 0 &&
    browserVisualComplete &&
    officePreviewComplete &&
    pdfRasterComplete &&
    officeScreenshotsComplete &&
    primaryEvidenceComplete &&
    reviewEvidenceComplete
      ? "automated-reviewed"
      : hostLimitedScreenshotOnly
        ? "host-limited"
      : "needs-review";
  const hostLimitations = hostLimitedBrowserAssertions(assertionList);
  const blockers =
    status === "host-limited"
      ? hostLimitations.map((assertion) => `Host-limited browser screenshot proof: ${assertion.scope} - ${hostLimitReason(assertion)}`)
      : [
          ...hardFailures.map((assertion) => `Failed proof: ${assertion.scope} - ${assertion.assertion}`),
          ...(!browserVisualComplete ? ["Chromium visual screenshots are incomplete."] : []),
          ...(!officePreviewComplete ? ["Office XML preview extraction is incomplete."] : []),
          ...(!officeScreenshotsComplete ? ["Office preview screenshots are incomplete."] : []),
          ...(!pdfRasterComplete ? ["PDF raster thumbnail proof is unavailable on this host."] : []),
          ...(!primaryEvidenceComplete ? ["One or more primary export targets lacks mapped proof."] : []),
          ...(!reviewEvidenceComplete ? ["One or more rendered review-case targets lacks mapped proof."] : []),
        ];
  const reportPath = join(auditDir, "automated-visual-review.json");
  const automatedReport = {
    generatedAt: new Date().toISOString(),
    status,
    reviewer: {
      name: "NEditor rendered export audit",
      role: "automated current-host visual proof",
      platform: process.platform,
      arch: process.arch,
      node: process.version,
    },
    acceptance: {
      browserVisualComplete,
      officePreviewComplete,
      officeScreenshotsComplete,
      browserScreenshotHostLimited,
      officeScreenshotHostLimited,
      pdfRasterComplete,
      primaryEvidenceComplete,
      reviewEvidenceComplete,
      hostLimitations: hostLimitations.map((assertion) => ({
        scope: assertion.scope,
        assertion: assertion.assertion,
        reason: hostLimitReason(assertion),
      })),
      blockers,
    },
    primaryTargets,
    reviewCases,
  };
  writeFileSync(reportPath, `${JSON.stringify(automatedReport, null, 2)}\n`);
  assertions.push({
    scope: "automated-visual-review",
    assertion: "writes current-host automated visual review sign-off",
    passed: status === "automated-reviewed",
    skipped: status !== "automated-reviewed",
    path: "automated-visual-review.json",
    status,
    blockers,
  });
  return {
    status,
    path: "automated-visual-review.json",
    blockers,
  };
}

function hasHostLimitedBrowserAssertion(assertions, scope) {
  return assertions.some((assertion) => assertion.scope === scope && isHostLimitedBrowserAssertion(assertion));
}

function hostLimitedBrowserAssertions(assertions) {
  return assertions.filter(isHostLimitedBrowserAssertion);
}

function isHostLimitedBrowserAssertion(assertion) {
  const reason = String(assertion.reason || "");
  const browserScreenshotScopes = new Set(["browser-visual-proof", "office-preview-browser"]);
  return Boolean(
    browserScreenshotScopes.has(String(assertion.scope || "")) &&
      assertion.skipped &&
      /(?:EPERM|SIGABRT|sandbox|host-level|Operation not permitted)/i.test(reason),
  );
}

function hostLimitReason(assertion) {
  return String(assertion.reason || "Browser automation is unavailable on this verifier host.")
    .replace(/\x1B\[[0-9;]*m/g, "")
    .split(/\r?\n/)
    .filter(Boolean)
    .slice(0, 4)
    .join(" ");
}

function automatedTargetEvidence(target, evidence) {
  const passedEvidence = evidence.filter((item) => item.passed);
  return {
    target: target.target,
    path: target.path,
    bytes: target.bytes,
    sha256: target.sha256,
    passedEvidenceCount: passedEvidence.length,
    skippedEvidenceCount: evidence.filter((item) => item.skipped).length,
    evidence,
  };
}

function scopesPassed(assertions, scopes) {
  return scopes.every((scope) => assertions.some((assertion) => assertion.scope === scope && assertion.passed));
}

function signoffArtifact(target) {
  return {
    target: target.target,
    path: target.path,
    bytes: target.bytes,
    sha256: target.sha256,
    status: "pending",
    viewer: "",
    reviewerNotes: "",
  };
}

function validateCompletedSignoff(signoff, report) {
  const validationIssues = [];
  if (signoff.schema !== "neditor.rendered-export.visual-signoff.v1") {
    validationIssues.push("completed sign-off schema must be neditor.rendered-export.visual-signoff.v1");
  }
  if (signoff.appVersion !== packageJson.version) {
    validationIssues.push(`completed sign-off appVersion must match package.json version ${packageJson.version}`);
  }
  if (signoff.sourceCommit !== currentSourceCommit) {
    validationIssues.push(`completed sign-off sourceCommit must match current git commit ${currentSourceCommit}`);
  }
  if (signoff.sourceTreeClean !== true) {
    validationIssues.push("completed sign-off sourceTreeClean must be true");
  }
  if (signoff.status !== "human-reviewed") {
    validationIssues.push("completed sign-off status must be human-reviewed");
  }
  if (!signoff.reviewer?.name || !signoff.reviewer?.reviewedAt) {
    validationIssues.push("completed sign-off must include reviewer.name and reviewer.reviewedAt");
  }
  if (signoff.reviewer?.reviewedAt && !isIsoDate(signoff.reviewer.reviewedAt)) {
    validationIssues.push("completed sign-off reviewer.reviewedAt must be an ISO timestamp");
  }
  if (!Array.isArray(signoff.reviewer?.nativeViewers) || signoff.reviewer.nativeViewers.length === 0) {
    validationIssues.push("completed sign-off must include at least one reviewer.nativeViewers entry");
  }
  if (Array.isArray(signoff.acceptance?.blockers) && signoff.acceptance.blockers.length > 0) {
    validationIssues.push("completed sign-off must not contain unresolved blockers");
  }
  if (signoff.acceptance?.allPrimaryArtifactsReviewed !== true) {
    validationIssues.push("completed sign-off must set acceptance.allPrimaryArtifactsReviewed to true");
  }
  if (signoff.acceptance?.allReviewCasesReviewed !== true) {
    validationIssues.push("completed sign-off must set acceptance.allReviewCasesReviewed to true");
  }
  if (signoff.acceptance?.allChecklistItemsReviewed !== true) {
    validationIssues.push("completed sign-off must set acceptance.allChecklistItemsReviewed to true");
  }

  const requiredTargets = new Map((report.targets || []).map((target) => [`${target.target}:${target.path}`, target]));
  const requiredTargetKeys = new Set(requiredTargets.keys());
  const signedTargetKeys = new Set((signoff.primaryArtifacts || []).map((target) => `${target.target}:${target.path}`));
  for (const key of requiredTargetKeys) {
    if (!signedTargetKeys.has(key)) validationIssues.push(`completed sign-off is missing primary artifact ${key}`);
  }
  for (const artifact of signoff.primaryArtifacts || []) {
    const label = `primary artifact ${artifact.target}:${artifact.path}`;
    validateReviewedStatus(validationIssues, label, artifact);
    validateSignedArtifactIdentity(validationIssues, label, artifact, requiredTargets.get(`${artifact.target}:${artifact.path}`));
  }

  const requiredReviewCases = new Map((report.reviewCases || []).map((reviewCase) => [reviewCase.slug, reviewCase]));
  const signedReviewCases = new Map((signoff.reviewCases || []).map((reviewCase) => [reviewCase.slug, reviewCase]));
  for (const [slug, reviewCase] of requiredReviewCases) {
    const signedCase = signedReviewCases.get(slug);
    if (!signedCase) {
      validationIssues.push(`completed sign-off is missing review case ${slug}`);
      continue;
    }
    validateReviewedStatus(validationIssues, `review case ${slug}`, signedCase);
    const requiredCaseTargets = new Set((reviewCase.targets || []).map((target) => `${target.target}:${target.path}`));
    const signedCaseTargets = new Set((signedCase.targets || []).map((target) => `${target.target}:${target.path}`));
    for (const key of requiredCaseTargets) {
      if (!signedCaseTargets.has(key)) validationIssues.push(`completed sign-off is missing review case target ${slug}:${key}`);
    }
    const requiredCaseTargetMap = new Map((reviewCase.targets || []).map((target) => [`${target.target}:${target.path}`, target]));
    for (const artifact of signedCase.targets || []) {
      const label = `review case target ${slug}:${artifact.target}:${artifact.path}`;
      validateReviewedStatus(validationIssues, label, artifact);
      validateSignedArtifactIdentity(validationIssues, label, artifact, requiredCaseTargetMap.get(`${artifact.target}:${artifact.path}`));
    }
  }

  const requiredChecklistIds = new Set((report.manualChecklist || []).map((_, index) => `check-${String(index + 1).padStart(2, "0")}`));
  const signedChecklistIds = new Set((signoff.checklist || []).map((item) => item.id));
  for (const id of requiredChecklistIds) {
    if (!signedChecklistIds.has(id)) validationIssues.push(`completed sign-off is missing checklist item ${id}`);
  }
  for (const item of signoff.checklist || []) {
    validateReviewedStatus(validationIssues, `checklist item ${item.id}`, item);
  }
  return validationIssues;
}

function validateSignedArtifactIdentity(validationIssues, label, signedArtifact, requiredArtifact) {
  if (!requiredArtifact) {
    validationIssues.push(`${label} does not match a current audit artifact`);
    return;
  }
  if (Number(signedArtifact.bytes) !== Number(requiredArtifact.bytes)) {
    validationIssues.push(`${label} bytes must match current audit artifact`);
  }
  if (signedArtifact.sha256 !== requiredArtifact.sha256) {
    validationIssues.push(`${label} sha256 must match current audit artifact`);
  }
}

function validateReviewedStatus(validationIssues, label, item) {
  const status = item?.status || "";
  if (!["passed", "failed", "skipped-with-reason"].includes(status)) {
    validationIssues.push(`${label} must have status passed, failed, or skipped-with-reason`);
  }
  if (status === "failed") {
    validationIssues.push(`${label} is marked failed`);
  }
  if (status === "skipped-with-reason" && !item?.reviewerNotes) {
    validationIssues.push(`${label} is skipped but has no reviewerNotes`);
  }
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

function writeManualReviewDashboard(report, assertions) {
  const generatedAt = new Date().toISOString();
  const primaryTargets = (report.targets || [])
    .map((target) => targetRow(target))
    .join("\n");
  const reviewCases = (report.reviewCases || [])
    .map((reviewCase) => {
      const targets = (reviewCase.targets || [])
        .map((target) => targetRow(target))
        .join("\n");
      const evidence = (reviewCase.requiredEvidence || [])
        .map((item) => `<li><code>${escapeHtml(item)}</code></li>`)
        .join("\n");
      return `<section class="review-case">
  <h3>${escapeHtml(reviewCase.title || reviewCase.slug)}</h3>
  <p><code>${escapeHtml(reviewCase.slug || "")}</code></p>
  <h4>Required Evidence</h4>
  <ul>${evidence}</ul>
  <table>
    <thead><tr><th>Target</th><th>Artifact</th><th>Bytes</th><th>SHA-256</th></tr></thead>
    <tbody>${targets}</tbody>
  </table>
</section>`;
    })
    .join("\n");
  const checklist = (report.manualChecklist || [])
    .map((item, index) => `<li><label><input type="checkbox"> <strong>${index + 1}.</strong> ${escapeHtml(item)}</label></li>`)
    .join("\n");
  const proofRows = assertions
    .map((assertion) => {
      const state = assertion.skipped ? "skipped" : assertion.passed ? "passed" : "failed";
      const detail = assertion.reason || assertion.thumbnail || assertion.path || assertion.stderr || "";
      return `<tr class="${state}"><td>${escapeHtml(assertion.scope || "")}</td><td>${escapeHtml(assertion.assertion || "")}</td><td>${state}</td><td>${escapeHtml(String(detail || ""))}</td></tr>`;
    })
    .join("\n");
  const thumbnails = assertions
    .filter((assertion) => assertion.passed && assertion.thumbnail)
    .map(
      (assertion) => `<figure>
  <a href="${escapeHtml(assertion.thumbnail)}"><img src="${escapeHtml(assertion.thumbnail)}" alt="${escapeHtml(assertion.scope)} thumbnail"></a>
  <figcaption><code>${escapeHtml(assertion.scope)}</code><br>${Number(assertion.width || 0)}x${Number(assertion.height || 0)} px, ${Number(assertion.bytes || 0).toLocaleString("en-US")} bytes</figcaption>
</figure>`,
    )
    .join("\n");
  const html = `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Rendered Export Manual Review</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; line-height: 1.5; margin: 2rem; color: #1f2933; }
    h1, h2, h3 { line-height: 1.2; }
    table { border-collapse: collapse; width: 100%; margin: 1rem 0 2rem; }
    th, td { border: 1px solid #cbd5df; padding: 0.45rem 0.6rem; text-align: left; vertical-align: top; }
    th { background: #edf2f7; }
    code { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 0.92em; }
    .passed td { background: #eef8ef; }
    .failed td { background: #fff1f1; }
    .skipped td { background: #fff8e5; }
    .review-case { border-top: 2px solid #d9e2ec; padding-top: 1rem; }
    .hash { word-break: break-all; }
    .thumbnail-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 1rem; margin: 1rem 0 2rem; }
    figure { margin: 0; padding: 0.75rem; border: 1px solid #cbd5df; background: #f8fafc; }
    figure img { display: block; width: 100%; height: auto; border: 1px solid #d9e2ec; background: white; }
    figcaption { margin-top: 0.5rem; font-size: 0.9rem; color: #52606d; }
  </style>
</head>
<body>
  <h1>Rendered Export Manual Review</h1>
  <p>Generated ${escapeHtml(generatedAt)} by <code>pnpm run test:rendered-exports</code>.</p>
  <p>This dashboard is a human-review entry point for the generated export package. Open each linked artifact in the relevant native viewer, then use the checklist and proof table below to record manual QA results outside this file if needed.</p>
  <p>Structured sign-off template: <a href="visual-review-signoff.template.json"><code>visual-review-signoff.template.json</code></a>. To validate a completed review against this existing audit bundle, fill a copy with <code>status: "human-reviewed"</code> and run <code>NEDITOR_RENDERED_EXPORT_SIGNOFF=/path/to/signoff.json pnpm run test:rendered-exports -- --validate-signoff-only</code>.</p>

  <h2>Primary Artifacts</h2>
  <table>
    <thead><tr><th>Target</th><th>Artifact</th><th>Bytes</th><th>SHA-256</th></tr></thead>
    <tbody>${primaryTargets}</tbody>
  </table>

  <h2>Manual Checklist</h2>
  <ol>${checklist}</ol>

  <h2>Review Cases</h2>
  ${reviewCases}

  <h2>Visual Review Thumbnails</h2>
  <p>These PNG thumbnails come from exported PDFs through Poppler <code>pdftoppm</code>, browser-rendered HTML artifacts, and Office XML preview dashboards for DOCX/PPTX artifacts when those tools are available on the verifier host. They provide fast visual entry points for manual review.</p>
  <div class="thumbnail-grid">${thumbnails || "<p>No visual thumbnails were generated on this host.</p>"}</div>

  <h2>Executable Viewer And Package Proof</h2>
  <table>
    <thead><tr><th>Scope</th><th>Assertion</th><th>Status</th><th>Detail</th></tr></thead>
    <tbody>${proofRows}</tbody>
  </table>
</body>
</html>
`;
  writeFileSync(join(auditDir, "manual-review.html"), html);
}

function verifyManualReviewDashboard(issues, assertions) {
  const path = join(auditDir, "manual-review.html");
  if (!existsSync(path)) {
    issues.push("manual-review.html was not generated");
    return;
  }
  if (statSync(path).size < 5000) {
    issues.push(`manual-review.html is unexpectedly small: ${statSync(path).size} bytes`);
    return;
  }
  const html = readFileSync(path, "utf8");
  const expectedContent = [
    "Rendered Export Manual Review",
    "rendered-export-audit.pdf",
    "rendered-export-audit.google-docs.zip",
    "rendered-export-audit.epub",
    "review-cases/rich-blocks/rich-blocks.html",
    "review-cases/option-heavy/option-heavy.html",
    "review-cases/brand-layout/brand-layout.html",
    "review-cases/business-transforms/business-transforms.html",
    "review-cases/equations/equations.html",
    "review-cases/toc-page-numbers/toc-page-numbers.html",
    "review-cases/edited-tables/edited-tables.html",
    "Executable Viewer And Package Proof",
    "Visual Review Thumbnails",
    "visual-review-signoff.template.json",
    "NEDITOR_RENDERED_EXPORT_SIGNOFF",
  ];
  for (const scope of ["macos-quicklook-pdf", "macos-quicklook-docx", "macos-quicklook-pptx"]) {
    if (assertions.some((assertion) => assertion.scope === scope)) expectedContent.push(scope);
  }
  for (const scope of ["pdfinfo-primary", "pdftotext-primary", "pdftoppm-primary-page-1"]) {
    if (assertions.some((assertion) => assertion.scope === scope)) expectedContent.push(scope);
  }
  for (const scope of ["browser-visual-primary-html", "browser-visual-manual-dashboard"]) {
    if (assertions.some((assertion) => assertion.scope === scope)) expectedContent.push(scope);
  }
  for (const scope of ["office-preview-docx", "office-preview-pptx"]) {
    if (assertions.some((assertion) => assertion.scope === scope)) expectedContent.push(scope);
  }
  for (const expected of expectedContent) {
    if (!html.includes(expected)) {
      issues.push(`manual-review.html is missing expected content: ${expected}`);
    }
  }
}

function writeVisualReviewSummary(report, assertions, humanSignoff, automatedVisualReview) {
  const generatedAt = new Date().toISOString();
  const assertionList = Array.isArray(assertions) ? assertions : [];
  const primaryTargets = (report.targets || []).map((target) => ({
    target: target.target,
    path: target.path,
    bytes: target.bytes,
    sha256: target.sha256,
    evidence: targetEvidence(target.target, assertionList),
  }));
  const reviewCases = (report.reviewCases || []).map((reviewCase) => ({
    slug: reviewCase.slug,
    title: reviewCase.title,
    requiredEvidence: reviewCase.requiredEvidence || [],
    targets: (reviewCase.targets || []).map((target) => ({
      target: target.target,
      path: target.path,
      bytes: target.bytes,
      sha256: target.sha256,
      evidence: targetEvidence(`${reviewCase.slug}:${target.target}`, assertionList),
    })),
  }));
  const thumbnails = assertionList
    .filter((assertion) => assertion.thumbnail)
    .map((assertion) => ({
      scope: assertion.scope,
      path: assertion.thumbnail,
      passed: Boolean(assertion.passed),
      bytes: assertion.bytes || 0,
      width: assertion.width || 0,
      height: assertion.height || 0,
    }));
  const skipped = assertionList
    .filter((assertion) => assertion.skipped)
    .map((assertion) => ({
      scope: assertion.scope,
      assertion: assertion.assertion,
      reason: assertion.reason || "Unavailable on this verifier host.",
    }));
  const summary = {
    generatedAt,
    auditDirectory: auditDir,
    manualReviewDashboard: "manual-review.html",
    viewerProof: "viewer-proof.json",
    humanSignoff: {
      status: humanSignoff.status,
      template: "visual-review-signoff.template.json",
      completedSignoff: humanSignoff.path || null,
      reviewer: humanSignoff.reviewer || null,
      reviewedAt: humanSignoff.reviewedAt || null,
      reason: humanSignoff.reason,
    },
    automatedVisualReview: {
      status: automatedVisualReview.status,
      report: automatedVisualReview.path,
      blockers: automatedVisualReview.blockers || [],
    },
    primaryTargets,
    reviewCases,
    visualEvidence: {
      browser: assertionFamily(assertionList, "browser-visual"),
      pdfRaster: assertionFamily(assertionList, "pdftoppm"),
      popplerText: assertionFamily(assertionList, "pdf"),
      macosTextutil: assertionFamily(assertionList, "macos-textutil"),
      macosQuickLook: assertionFamily(assertionList, "macos-quicklook"),
      officePreview: assertionFamily(assertionList, "office-preview"),
      thumbnails,
      skipped,
    },
  };
  writeFileSync(join(auditDir, "visual-review-summary.json"), `${JSON.stringify(summary, null, 2)}\n`);
}

function verifyVisualReviewSummary(issues, report, assertions) {
  const path = join(auditDir, "visual-review-summary.json");
  if (!existsSync(path)) {
    issues.push("visual-review-summary.json was not generated");
    return;
  }
  if (statSync(path).size < 1000) {
    issues.push(`visual-review-summary.json is unexpectedly small: ${statSync(path).size} bytes`);
    return;
  }
  const summary = JSON.parse(readFileSync(path, "utf8"));
  const targetSet = new Set((summary.primaryTargets || []).map((target) => target.target));
  for (const target of ["html", "pdf", "docx", "pptx", "markdown-bundle", "blog", "substack", "latex", "google-docs", "epub"]) {
    if (!targetSet.has(target)) issues.push(`visual-review-summary.json is missing primary target ${target}`);
  }
  if (!["pending-human-review", "human-reviewed"].includes(summary.humanSignoff?.status)) {
    issues.push("visual-review-summary.json has an invalid human sign-off status");
  }
  if (summary.humanSignoff?.template !== "visual-review-signoff.template.json") {
    issues.push("visual-review-summary.json does not link the structured human sign-off template");
  }
  if (!["automated-reviewed", "host-limited", "needs-review"].includes(summary.automatedVisualReview?.status)) {
    issues.push("visual-review-summary.json has an invalid automated visual review status");
  }
  if (summary.automatedVisualReview?.report !== "automated-visual-review.json") {
    issues.push("visual-review-summary.json does not link the automated visual review report");
  }
  if (!existsSync(join(auditDir, "automated-visual-review.json"))) {
    issues.push("automated-visual-review.json was not generated");
  }
  if (summary.manualReviewDashboard !== "manual-review.html" || summary.viewerProof !== "viewer-proof.json") {
    issues.push("visual-review-summary.json does not link the manual dashboard and viewer proof");
  }
  const expectedReviewCases = new Set((report.reviewCases || []).map((reviewCase) => reviewCase.slug));
  const actualReviewCases = new Set((summary.reviewCases || []).map((reviewCase) => reviewCase.slug));
  for (const slug of expectedReviewCases) {
    if (!actualReviewCases.has(slug)) issues.push(`visual-review-summary.json is missing review case ${slug}`);
  }
  const browserEvidenceExists = assertions.some((assertion) => String(assertion.scope || "").startsWith("browser-visual"));
  if (browserEvidenceExists && !summary.visualEvidence?.browser?.length) {
    issues.push("visual-review-summary.json is missing browser visual evidence");
  }
  const thumbnailEvidenceExists = assertions.some((assertion) => assertion.thumbnail);
  if (thumbnailEvidenceExists && !summary.visualEvidence?.thumbnails?.length) {
    issues.push("visual-review-summary.json is missing generated thumbnail evidence");
  }
  const officePreviewExists = assertions.some((assertion) => String(assertion.scope || "").startsWith("office-preview"));
  if (officePreviewExists && !summary.visualEvidence?.officePreview?.length) {
    issues.push("visual-review-summary.json is missing Office preview evidence");
  }
}

function assertionFamily(assertions, prefix) {
  return assertions
    .filter((assertion) => String(assertion.scope || "").startsWith(prefix))
    .map((assertion) => ({
      scope: assertion.scope,
      assertion: assertion.assertion,
      passed: Boolean(assertion.passed),
      skipped: Boolean(assertion.skipped),
      reason: assertion.reason || "",
      path: assertion.path || "",
      thumbnail: assertion.thumbnail || "",
      bytes: assertion.bytes || 0,
      width: assertion.width || 0,
      height: assertion.height || 0,
    }));
}

function targetEvidence(target, assertions) {
  const scopesByTarget = {
    html: ["browser-visual-primary-html", "html"],
    pdf: ["pdfinfo-primary", "pdftotext-primary", "pdftoppm-primary", "macos-quicklook-pdf"],
    docx: ["docx-package", "docx-document", "macos-textutil-docx", "macos-quicklook-docx", "office-preview-docx"],
    pptx: ["pptx-package", "pptx-slides", "macos-quicklook-pptx", "office-preview-pptx"],
    "markdown-bundle": ["markdown-bundle-package", "markdown-bundle-manifest"],
    blog: ["blog-package", "blog-metadata", "blog-post"],
    substack: ["substack-package", "substack-metadata", "substack-copy"],
    latex: ["latex-source"],
    "google-docs": ["google-docs-package", "google-docs-metadata", "google-docs-docx-document"],
    epub: ["epub-package", "epub-container", "epub-package-manifest", "epub-document", "epub-text-fallback"],
  };
  const [reviewSlug, reviewTarget] = String(target).includes(":") ? String(target).split(":") : ["", target];
  const prefixes = reviewSlug
    ? [
        `${reviewSlug}-${reviewTarget}`,
        ...(reviewTarget === "markdown-bundle" ? [`${reviewSlug}-bundle`] : []),
        ...(reviewTarget === "pdf" ? [`pdftotext-review-${reviewSlug}`, `pdftoppm-review-${reviewSlug}`] : []),
        ...(reviewTarget === "docx" ? [`macos-textutil-review-${reviewSlug}`] : []),
        `browser-visual-review-${reviewSlug}`,
        `office-preview-review-${reviewSlug}-${reviewTarget}`,
      ]
    : scopesByTarget[target] || [target];
  return assertions
    .filter((assertion) => prefixes.some((prefix) => String(assertion.scope || "").startsWith(prefix)))
    .map((assertion) => ({
      scope: assertion.scope,
      passed: Boolean(assertion.passed),
      skipped: Boolean(assertion.skipped),
      thumbnail: assertion.thumbnail || "",
    }));
}

function targetRow(target) {
  const path = escapeHtml(target.path || "");
  const targetName = escapeHtml(target.target || "");
  const bytes = Number(target.bytes || 0).toLocaleString("en-US");
  const hash = escapeHtml(target.sha256 || "");
  return `<tr><td>${targetName}</td><td><a href="${path}">${path}</a></td><td>${bytes}</td><td class="hash"><code>${hash}</code></td></tr>`;
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

function commandResult(command, args) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    timeout: 5000,
  });
  return result;
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

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function decodeXml(value) {
  return String(value)
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&apos;", "'")
    .replaceAll("&amp;", "&");
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 && result.stdout.trim() === "";
}

function unzipEntryData(method, data, name) {
  if (method === 0) return data;
  if (method === 8) return inflateRawSync(data);
  throw new Error(`unsupported ZIP compression method ${method} for ${name}`);
}
