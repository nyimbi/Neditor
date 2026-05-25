import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sourcePath = "src/App.vue";
const reportPath = resolve(
  process.env.NEDITOR_ACCESSIBILITY_REPORT || join(root, ".tmp", "accessibility", "report.json"),
);
const source = readFileSync(join(root, sourcePath), "utf8");
const templateStart = source.indexOf("<template>");
const scriptStart = source.indexOf("<script");
const templateEnd = source.lastIndexOf("</template>", scriptStart);
const template =
  templateStart >= 0 && templateEnd > templateStart
    ? source.slice(templateStart + "<template>".length, templateEnd)
    : "";
const issues = [];
const checks = [
  {
    id: "button-names",
    description: "Buttons expose descriptive text, aria labels, labelled-by references, or titles.",
    run: checkButtons,
  },
  {
    id: "button-hover-help",
    description: "Buttons receive delegated hover and focus help from labels, titles, or data-help text.",
    run: checkButtonHoverHelp,
  },
  {
    id: "form-control-labels",
    description: "Inputs, selects, and textareas are labelled directly or by wrapping labels.",
    run: checkFormControls,
  },
  {
    id: "dialog-contracts",
    description: "Dialogs are modal, named, focusable, and keyboard handled.",
    run: checkDialogs,
  },
  {
    id: "skip-links",
    description: "Primary workbench regions have visible-on-focus skip links and focusable targets.",
    run: checkSkipLinks,
  },
  {
    id: "status-announcements",
    description: "Status, watch, compile, export, and error messages are exposed as live regions.",
    run: checkStatusAnnouncements,
  },
  {
    id: "diagnostic-labels",
    description: "Diagnostic collections and items are exposed as named lists and list items.",
    run: checkDiagnosticLabels,
  },
  {
    id: "conflict-diff-labels",
    description: "Conflict diff cells are named groups with source-side context.",
    run: checkConflictDiffLabels,
  },
  {
    id: "table-editor-labels",
    description: "Table editor grid, cells, totals, and controls are labelled.",
    run: checkTableEditorLabels,
  },
  {
    id: "contrast-motion-css",
    description: "High-contrast and reduced-motion CSS contracts are enforced.",
    run: checkContrastMotionCss,
  },
  {
    id: "editor-preview-surfaces",
    description: "Editor and preview surfaces expose document-oriented screen-reader semantics.",
    run: checkEditorPreviewSurfaceLabels,
  },
];
const checkResults = [];

for (const check of checks) {
  const issueCountBefore = issues.length;
  check.run();
  checkResults.push({
    id: check.id,
    description: check.description,
    status: issues.length === issueCountBefore ? "pass" : "fail",
  });
}

if (issues.length > 0) {
  writeReport("fail", checkResults, issues);
  console.error("Accessibility guard failed:");
  for (const issue of issues) {
    console.error(`- ${issue}`);
  }
  process.exit(1);
}

writeReport("pass", checkResults, issues);
console.log(`Checked App.vue template accessibility guardrails; wrote ${reportPath}.`);

function checkButtons() {
  const buttonPattern = /<button\b([^>]*)>([\s\S]*?)<\/button>/g;
  for (const match of template.matchAll(buttonPattern)) {
    const [, attrs, body] = match;
    if (hasAccessibleName(attrs)) {
      continue;
    }
    const text = visibleText(body);
    if (!text || isGenericButtonText(text)) {
      issues.push(
        `${sourcePath}:${lineFor(match.index)} button needs descriptive text or aria-label`,
      );
    }
  }
}

function checkButtonHoverHelp() {
  const requirements = [
    ["tooltip role", /class\s*=\s*["']button-help-tooltip["'][\s\S]*?role\s*=\s*["']tooltip["']/],
    ["mouseover listener", /window\.addEventListener\(["']mouseover["'],\s*handleButtonHelpEnter\)/],
    ["focusin listener", /window\.addEventListener\(["']focusin["'],\s*handleButtonHelpEnter\)/],
    ["mouseout listener", /window\.addEventListener\(["']mouseout["'],\s*handleButtonHelpLeave\)/],
    ["focusout listener", /window\.addEventListener\(["']focusout["'],\s*handleButtonHelpLeave\)/],
    ["listener cleanup", /window\.removeEventListener\(["']mouseover["'],\s*handleButtonHelpEnter\)[\s\S]*window\.removeEventListener\(["']focusout["'],\s*handleButtonHelpLeave\)/],
    ["button event delegation", /target\?\.closest\(["']button["']\)/],
    ["data-help fallback", /button\.getAttribute\(["']data-help["']\)/],
    ["title fallback", /button\.getAttribute\(["']title["']\)/],
    ["aria-label fallback", /button\.getAttribute\(["']aria-label["']\)/],
    ["visible text fallback", /button\.innerText\.replace/],
    ["disabled help fallback", /data-disabled-help/],
  ];
  for (const [label, pattern] of requirements) {
    if (!pattern.test(source)) {
      issues.push(`${sourcePath}:1 button hover help must include ${label}`);
    }
  }
}

function checkFormControls() {
  const controlPattern = /<(input|select|textarea)\b([^>]*)>/g;
  for (const match of template.matchAll(controlPattern)) {
    const [, tag, attrs] = match;
    if (/\btype\s*=\s*["']hidden["']/.test(attrs)) {
      continue;
    }
    if (hasAccessibleName(attrs) || isInsideLabel(match.index)) {
      continue;
    }
    issues.push(`${sourcePath}:${lineFor(match.index)} ${tag} needs a label or aria-label`);
  }
}

function checkDialogs() {
  const dialogPattern = /<[^>]+\brole\s*=\s*["']dialog["'][^>]*>/g;
  for (const match of template.matchAll(dialogPattern)) {
    const attrs = match[0];
    if (!/\baria-modal\s*=\s*["']true["']/.test(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} dialog must set aria-modal="true"`);
    }
    if (!hasAccessibleName(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} dialog needs aria-label or aria-labelledby`);
    }
    if (!/\btabindex\s*=\s*["']-1["']/.test(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} dialog must be programmatically focusable`);
    }
    if (!/@keydown\s*=/.test(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} dialog must handle keyboard focus trapping and Escape`);
    }
  }
}

function checkSkipLinks() {
  const skipLinks = template.match(/<nav\b[^>]*class\s*=\s*["'][^"']*\bskip-links\b[^"']*["'][\s\S]*?<\/nav>/);
  if (!skipLinks) {
    issues.push(`${sourcePath}:1 skip-links navigation is missing`);
    return;
  }
  const hrefPattern = /href\s*=\s*["']#([^"']+)["']/g;
  const targets = [...skipLinks[0].matchAll(hrefPattern)].map((match) => match[1]);
  const requiredTargets = [
    "main-commands",
    "document-workspace",
    "document-sidebar",
    "markdown-source",
    "live-preview",
    "document-status",
  ];
  for (const target of requiredTargets) {
    if (!targets.includes(target)) {
      issues.push(`${sourcePath}:1 skip-links navigation is missing #${target}`);
      continue;
    }
    const targetPattern = new RegExp(`\\bid\\s*=\\s*["']${target}["'][^>]*\\btabindex\\s*=\\s*["']-1["']|\\btabindex\\s*=\\s*["']-1["'][^>]*\\bid\\s*=\\s*["']${target}["']`);
    if (!targetPattern.test(template)) {
      issues.push(`${sourcePath}:1 skip target #${target} must exist and be programmatically focusable`);
    }
  }
}

function checkStatusAnnouncements() {
  const statusBar = template.match(/<footer\b[^>]*id\s*=\s*["']document-status["'][\s\S]*?<\/footer>/);
  if (!statusBar) {
    issues.push(`${sourcePath}:1 document status footer is missing`);
    return;
  }
  const markup = statusBar[0];
  if (!/\baria-label\s*=\s*["']Document status and progress["']/.test(markup)) {
    issues.push(`${sourcePath}:1 document status footer needs an accessible label`);
  }
  for (const className of ["status-message", "watch-status", "compile-actions", "export-progress"]) {
    const pattern = new RegExp(`class\\s*=\\s*["'][^"']*\\b${className}\\b[^"']*["'][\\s\\S]*?role\\s*=\\s*["']status["'][\\s\\S]*?aria-live\\s*=\\s*["']polite["'][\\s\\S]*?aria-atomic\\s*=\\s*["']true["']`);
    if (!pattern.test(markup)) {
      issues.push(`${sourcePath}:1 ${className} must be a polite atomic status live region`);
    }
  }
  const errorPattern = /class\s*=\s*["'][^"']*\berror\b[^"']*["'][\s\S]*?role\s*=\s*["']alert["'][\s\S]*?aria-live\s*=\s*["']assertive["'][\s\S]*?aria-atomic\s*=\s*["']true["']/;
  if (!errorPattern.test(markup)) {
    issues.push(`${sourcePath}:1 error status must be an assertive atomic alert live region`);
  }
}

function checkDiagnosticLabels() {
  const diagnosticArticles = [...template.matchAll(/<article\b[^>]*class\s*=\s*["']diagnostic["'][^>]*>/g)];
  if (diagnosticArticles.length === 0) {
    issues.push(`${sourcePath}:1 diagnostic articles are missing`);
    return;
  }
  for (const match of diagnosticArticles) {
    const attrs = match[0];
    if (!/\brole\s*=\s*["']listitem["']/.test(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} diagnostic article must be a listitem`);
    }
    if (!/:aria-label\s*=\s*["']diagnosticAnnouncementLabel\(diagnostic\)["']/.test(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} diagnostic article needs diagnosticAnnouncementLabel aria-label`);
    }
  }
  for (const label of ["Compiler diagnostics", "Export readiness diagnostics", "Last export diagnostics"]) {
    const pattern = new RegExp(`\\brole\\s*=\\s*["']list["'][^>]*\\baria-label\\s*=\\s*["']${escapeRegExp(label)}["']|\\baria-label\\s*=\\s*["']${escapeRegExp(label)}["'][^>]*\\brole\\s*=\\s*["']list["']`);
    if (!pattern.test(template)) {
      issues.push(`${sourcePath}:1 ${label} must be exposed as a named diagnostic list`);
    }
  }
}

function checkConflictDiffLabels() {
  const conflictDiff = template.match(/<section\b[^>]*class\s*=\s*["']conflict-diff["'][\s\S]*?<\/section>/);
  if (!conflictDiff) {
    issues.push(`${sourcePath}:1 conflict diff section is missing`);
    return;
  }
  const markup = conflictDiff[0];
  for (const source of ["local", "external"]) {
    const pattern = new RegExp(`role\\s*=\\s*["']group["'][\\s\\S]*?:aria-label\\s*=\\s*["']conflictDiffCellLabel\\(row, '${source}'\\)["']|:aria-label\\s*=\\s*["']conflictDiffCellLabel\\(row, '${source}'\\)["'][\\s\\S]*?role\\s*=\\s*["']group["']`);
    if (!pattern.test(markup)) {
      issues.push(`${sourcePath}:1 conflict diff ${source} cell must be a named group`);
    }
  }
}

function checkTableEditorLabels() {
  const grid = template.match(/<div\b[^>]*class\s*=\s*["']table-editor-grid["'][\s\S]*?<\/div>/);
  if (!grid) {
    issues.push(`${sourcePath}:1 table editor grid is missing`);
    return;
  }
  const markup = grid[0];
  if (!/\brole\s*=\s*["']group["']/.test(markup) || !/\baria-label\s*=\s*["']Table editor grid["']/.test(markup)) {
    issues.push(`${sourcePath}:1 table editor grid must be a named group`);
  }
  for (const helper of ["tableHeaderLabel(columnIndex)", "tableCellLabel(rowIndex, columnIndex)", "tableTotalLabel(columnIndex)"]) {
    if (!markup.includes(`:aria-label="${helper}"`)) {
      issues.push(`${sourcePath}:1 table editor grid must use ${helper}`);
    }
  }
  for (const label of ["Row ${rowIndex + 1} controls", "Sort controls for column", "Move controls for column"]) {
    if (!markup.includes(label)) {
      issues.push(`${sourcePath}:1 table editor grid is missing ${label} group labels`);
    }
  }
}

function checkContrastMotionCss() {
  if (!/:data-high-contrast\s*=\s*["']store\.highContrast \? 'true' : 'false'["']/.test(template)) {
    issues.push(`${sourcePath}:1 app shell must expose high-contrast state`);
  }
  if (!/:data-reduced-motion\s*=\s*["']store\.reducedMotion \? 'true' : 'false'["']/.test(template)) {
    issues.push(`${sourcePath}:1 app shell must expose reduced-motion state`);
  }
  const highContrastBlock = cssBlock(".app-shell[data-high-contrast=\"true\"]");
  if (!highContrastBlock.includes("color: #000000") || !highContrastBlock.includes("background: #ffffff")) {
    issues.push(`${sourcePath}:1 high contrast shell must force black-on-white colors`);
  }
  const highContrastControls = cssBlock(".app-shell[data-high-contrast=\"true\"] .titlebar");
  for (const declaration of ["border-color: #000000", "color: #000000", "background: #ffffff"]) {
    if (!highContrastControls.includes(declaration)) {
      issues.push(`${sourcePath}:1 high contrast controls must include ${declaration}`);
    }
  }
  const highContrastFocus = cssBlock(".app-shell[data-high-contrast=\"true\"] :focus-visible");
  if (!highContrastFocus.includes("outline: 3px solid #000000")) {
    issues.push(`${sourcePath}:1 high contrast focus state must use a black outline`);
  }
  const reducedMotionBlock = cssBlock(".app-shell[data-reduced-motion=\"true\"] *");
  for (const declaration of ["scroll-behavior: auto", "transition-duration: 0s", "animation-duration: 0s", "animation-iteration-count: 1"]) {
    if (!reducedMotionBlock.includes(declaration)) {
      issues.push(`${sourcePath}:1 reduced-motion mode must include ${declaration}`);
    }
  }
  const mediaReducedMotionBlock = cssBlock("@media (prefers-reduced-motion: reduce)");
  if (!mediaReducedMotionBlock.includes(".app-shell *") || !mediaReducedMotionBlock.includes("transition-duration: 0s")) {
    issues.push(`${sourcePath}:1 prefers-reduced-motion media query must disable app shell transitions`);
  }
}

function checkEditorPreviewSurfaceLabels() {
  const markdownSource = template.match(/<section\b[^>]*id\s*=\s*["']markdown-source["'][^>]*>/);
  if (!markdownSource || !/\baria-label\s*=\s*["']Markdown source["']/.test(markdownSource[0])) {
    issues.push(`${sourcePath}:1 Markdown source region must be labeled`);
  }
  const livePreview = template.match(/<section\b[^>]*id\s*=\s*["']live-preview["'][^>]*>/);
  if (!livePreview || !/\baria-label\s*=\s*["']Live preview["']/.test(livePreview[0])) {
    issues.push(`${sourcePath}:1 Live preview region must be labeled`);
  }
  const previewDocument = template.match(/<article\b[^>]*class\s*=\s*["']preview-document["'][^>]*>/);
  if (!previewDocument) {
    issues.push(`${sourcePath}:1 preview document article is missing`);
  } else {
    const attrs = previewDocument[0];
    if (!/\brole\s*=\s*["']document["']/.test(attrs)) {
      issues.push(`${sourcePath}:1 preview document must use role=document`);
    }
    if (!/:aria-label\s*=\s*["']previewDocumentLabel["']/.test(attrs)) {
      issues.push(`${sourcePath}:1 preview document must use previewDocumentLabel`);
    }
    if (!/\btabindex\s*=\s*["']0["']/.test(attrs)) {
      issues.push(`${sourcePath}:1 preview document must be keyboard focusable`);
    }
  }
  const contentAttributes = source.match(/EditorView\.contentAttributes\.of\(\{[\s\S]*?\}\)/);
  if (!contentAttributes) {
    issues.push(`${sourcePath}:1 CodeMirror content attributes are missing`);
    return;
  }
  const attrs = contentAttributes[0];
  for (const required of ['role: "textbox"', '"aria-label": "Markdown editor"', '"aria-multiline": "true"', 'spellcheck: "true"', 'autocapitalize: "sentences"']) {
    if (!attrs.includes(required)) {
      issues.push(`${sourcePath}:1 CodeMirror content attributes must include ${required}`);
    }
  }
}

function hasAccessibleName(attrs) {
  return /\b(:?aria-label|aria-labelledby|title)\s*=/.test(attrs);
}

function isInsideLabel(index) {
  const before = template.slice(0, index);
  return before.lastIndexOf("<label") > before.lastIndexOf("</label>");
}

function visibleText(body) {
  return body
    .replace(/<[^>]*>/g, " ")
    .replace(/\{\{[^}]+\}\}/g, " value ")
    .replace(/\s+/g, " ")
    .trim();
}

function isGenericButtonText(text) {
  return ["x", "×", "use"].includes(text.toLowerCase());
}

function lineFor(index) {
  return template.slice(0, index).split("\n").length + 1;
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function cssBlock(selector) {
  const start = source.indexOf(selector);
  if (start < 0) return "";
  const open = source.indexOf("{", start);
  if (open < 0) return "";
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") depth += 1;
    if (char === "}") {
      depth -= 1;
      if (depth === 0) {
        return source.slice(open + 1, index);
      }
    }
  }
  return "";
}

function writeReport(status, checks, issues) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        source: sourcePath,
        status,
        summary: {
          checks: checks.length,
          passed: checks.filter((check) => check.status === "pass").length,
          failed: checks.filter((check) => check.status === "fail").length,
          issues: issues.length,
        },
        checks,
        issues,
      },
      null,
      2,
    )}\n`,
  );
}
