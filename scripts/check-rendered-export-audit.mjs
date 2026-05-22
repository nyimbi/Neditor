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
let auditReport = null;

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
  for (const target of ["html", "pdf", "docx", "pptx", "markdown-bundle", "blog", "substack", "latex", "google-docs"]) {
    if (!targets.has(target)) {
      issues.push(`audit report is missing target ${target}`);
    }
  }
  if (!Array.isArray(auditReport.manualChecklist) || auditReport.manualChecklist.length < 7) {
    issues.push("audit report manual checklist is incomplete");
  }
  const reviewCases = Array.isArray(auditReport.reviewCases) ? auditReport.reviewCases : [];
  const reviewCaseSlugs = new Set(reviewCases.map((reviewCase) => reviewCase.slug));
  for (const slug of ["rich-blocks", "option-heavy"]) {
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

if (issues.length > 0) {
  console.error("Rendered export audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}

writeFileSync(
  join(auditDir, "viewer-proof.json"),
  `${JSON.stringify({ generatedAt: new Date().toISOString(), assertions: viewerProof }, null, 2)}\n`,
);
writeManualReviewDashboard(auditReport, viewerProof);
verifyManualReviewDashboard(issues, viewerProof);
if (issues.length > 0) {
  console.error("Rendered export audit failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  process.exit(1);
}
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
    const thumbnail = `${outputPrefix}-${page}.png`;
    const dimensions = existsSync(thumbnail) ? pngDimensions(thumbnail) : null;
    const bytes = existsSync(thumbnail) ? statSync(thumbnail).size : 0;
    const passed = Boolean(dimensions && dimensions.width >= 500 && dimensions.height >= 500 && bytes > 10_000);
    assertions.push({
      scope: `pdftoppm-${scope}-page-${page}`,
      assertion: `renders PDF page ${page} to a non-empty PNG thumbnail`,
      passed,
      thumbnail: relativeToAudit(thumbnail),
      bytes,
      width: dimensions?.width || 0,
      height: dimensions?.height || 0,
    });
    if (!passed) {
      issues.push(`pdftoppm did not render a meaningful thumbnail for ${relativeToAudit(path)} page ${page}`);
    }
  }
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
    assertions.push({
      scope: artifact.scope,
      assertion: artifact.assertion,
      passed,
      thumbnail: relativeToAudit(thumbnail),
      bytes: existsSync(thumbnail) ? statSync(thumbnail).size : 0,
    });
    if (!passed) {
      issues.push(`macOS Quick Look did not render a meaningful thumbnail for ${artifact.path}${output ? `:\n${output}` : ""}`);
    }
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
      [...new Set([reviewCase.title, ...(reviewCase.requiredEvidence || [])].filter(Boolean))],
    );
  }
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
      const detail = assertion.reason || assertion.thumbnail || assertion.stderr || "";
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

  <h2>Primary Artifacts</h2>
  <table>
    <thead><tr><th>Target</th><th>Artifact</th><th>Bytes</th><th>SHA-256</th></tr></thead>
    <tbody>${primaryTargets}</tbody>
  </table>

  <h2>Manual Checklist</h2>
  <ol>${checklist}</ol>

  <h2>Review Cases</h2>
  ${reviewCases}

  <h2>PDF Raster Review Thumbnails</h2>
  <p>These PNG thumbnails are generated from exported PDFs with Poppler <code>pdftoppm</code> when it is available on the verifier host. They provide fast visual entry points for manual PDF review.</p>
  <div class="thumbnail-grid">${thumbnails || "<p>No PDF raster thumbnails were generated on this host.</p>"}</div>

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
    "review-cases/rich-blocks/rich-blocks.html",
    "review-cases/option-heavy/option-heavy.html",
    "Executable Viewer And Package Proof",
    "PDF Raster Review Thumbnails",
    "macos-quicklook-pdf",
  ];
  for (const scope of ["pdfinfo-primary", "pdftotext-primary", "pdftoppm-primary-page-1"]) {
    if (assertions.some((assertion) => assertion.scope === scope)) expectedContent.push(scope);
  }
  for (const expected of expectedContent) {
    if (!html.includes(expected)) {
      issues.push(`manual-review.html is missing expected content: ${expected}`);
    }
  }
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

function unzipEntryData(method, data, name) {
  if (method === 0) return data;
  if (method === 8) return inflateRawSync(data);
  throw new Error(`unsupported ZIP compression method ${method} for ${name}`);
}
