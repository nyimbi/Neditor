import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outputDir = join(root, ".tmp", "accessibility");
const staticReportPath = join(outputDir, "report.json");
const runtimeReportPath = join(outputDir, "runtime-report.json");
const templatePath = join(outputDir, "manual-review-template.json");
const summaryPath = join(outputDir, "manual-review-summary.json");
const completedSignoffPath = process.env.NEDITOR_ACCESSIBILITY_SIGNOFF
  ? resolve(process.env.NEDITOR_ACCESSIBILITY_SIGNOFF)
  : null;

const checklist = [
  {
    id: "screen-reader-workbench-regions",
    label: "Screen reader can traverse commands, workspace tabs, sidebar, source, preview, and status regions in order.",
  },
  {
    id: "keyboard-only-core-workflows",
    label: "Keyboard-only operation covers file commands, command palette, search, toolbar groups, outline mode, and export readiness.",
  },
  {
    id: "modal-focus-and-escape",
    label: "AI Paste, command palette, conflict, template, and export dialogs keep focus contained, close on Escape, and restore focus.",
  },
  {
    id: "editor-preview-reading",
    label: "Source editor, diagnostics, rendered preview, headings, generated sections, and transform artifacts have understandable names.",
  },
  {
    id: "status-live-regions",
    label: "Compile, watch, save, export, readiness, and error messages are announced without stealing focus.",
  },
  {
    id: "contrast-motion-typography",
    label: "High contrast, reduced motion, toolbar display, text-size controls, and typography preferences are usable.",
  },
  {
    id: "native-desktop-traversal",
    label: "Native desktop window, menus, toolbar rows, file workflows, and outline mode are traversable with platform assistive technology.",
  },
  {
    id: "export-review-artifacts",
    label: "HTML, PDF, DOCX, PPTX, blog, Substack, LaTeX, and Google Docs handoff surfaces expose reviewable accessible output.",
  },
];

mkdirSync(outputDir, { recursive: true });
writeFileSync(templatePath, `${JSON.stringify(createTemplate(), null, 2)}\n`);

const issues = [];
const prerequisiteReports = {
  static: readReport(staticReportPath, ["pass", "passed"]),
  runtime: readReport(runtimeReportPath, "passed"),
};
let humanSignoff = {
  status: "pending-human-review",
  path: completedSignoffPath,
  reviewer: null,
  reviewedAt: null,
};

if (completedSignoffPath) {
  humanSignoff = validateCompletedSignoff(completedSignoffPath, issues);
  for (const [name, report] of Object.entries(prerequisiteReports)) {
    if (report.status !== "passed") {
      issues.push(`cannot accept completed sign-off because ${name} accessibility report is ${report.status}`);
    }
  }
}

const summary = {
  generatedAt: new Date().toISOString(),
  status: issues.length > 0 ? "failed" : humanSignoff.status,
  template: relativePath(templatePath),
  completedSignoffPath,
  prerequisiteReports,
  checklist: checklist.map((item) => item.id),
  humanSignoff,
  issues,
};

writeFileSync(summaryPath, `${JSON.stringify(summary, null, 2)}\n`);

if (issues.length > 0) {
  console.error("Accessibility manual sign-off validation failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  console.error(`Wrote ${relativePath(summaryPath)}.`);
  process.exit(1);
}

if (completedSignoffPath) {
  console.log(`Accessibility manual sign-off accepted; wrote ${relativePath(summaryPath)}.`);
} else {
  console.log(`Accessibility manual review template written to ${relativePath(templatePath)}.`);
  console.log(`No NEDITOR_ACCESSIBILITY_SIGNOFF supplied; summary remains pending at ${relativePath(summaryPath)}.`);
}

function createTemplate() {
  return {
    schema: "neditor.accessibility.manual-signoff.v1",
    reviewer: {
      name: "",
      role: "",
    },
    reviewedAt: new Date().toISOString(),
    platform: {
      os: process.platform,
      version: "",
      device: "",
    },
    assistiveTechnology: {
      name: "",
      version: "",
      settings: "",
    },
    browserOrWebview: {
      name: "",
      version: "",
    },
    checklist: checklist.map((item) => ({
      id: item.id,
      label: item.label,
      status: "pending",
      notes: "",
    })),
    unresolvedBlockers: [],
  };
}

function readReport(path, expectedStatus) {
  if (!existsSync(path)) {
    return {
      path: relativePath(path),
      status: "missing",
    };
  }
  try {
    const report = JSON.parse(readFileSync(path, "utf8"));
    const expected = Array.isArray(expectedStatus) ? expectedStatus : [expectedStatus];
    return {
      path: relativePath(path),
      status: expected.includes(report.status) ? "passed" : report.status || "unknown",
      generatedAt: report.generatedAt || null,
    };
  } catch (error) {
    return {
      path: relativePath(path),
      status: "invalid-json",
      error: String(error),
    };
  }
}

function validateCompletedSignoff(path, issues) {
  let signoff;
  try {
    signoff = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    issues.push(`could not read completed sign-off: ${error}`);
    return {
      status: "failed",
      path,
      reviewer: null,
      reviewedAt: null,
    };
  }

  if (signoff.schema !== "neditor.accessibility.manual-signoff.v1") {
    issues.push("completed sign-off schema must be neditor.accessibility.manual-signoff.v1");
  }
  if (!signoff.reviewer?.name?.trim()) {
    issues.push("completed sign-off must include reviewer.name");
  }
  if (!isIsoDate(signoff.reviewedAt)) {
    issues.push("completed sign-off must include reviewedAt as an ISO date string");
  }
  if (!signoff.platform?.os?.trim()) {
    issues.push("completed sign-off must include platform.os");
  }
  if (!signoff.assistiveTechnology?.name?.trim()) {
    issues.push("completed sign-off must include assistiveTechnology.name");
  }

  const submittedItems = new Map((signoff.checklist || []).map((item) => [item.id, item]));
  for (const expected of checklist) {
    const item = submittedItems.get(expected.id);
    if (!item) {
      issues.push(`completed sign-off is missing checklist item ${expected.id}`);
      continue;
    }
    if (item.status !== "pass") {
      issues.push(`checklist item ${expected.id} must be pass before sign-off can be accepted`);
    }
    if (!String(item.notes || "").trim()) {
      issues.push(`checklist item ${expected.id} must include reviewer notes`);
    }
  }

  const blockers = Array.isArray(signoff.unresolvedBlockers) ? signoff.unresolvedBlockers : [];
  if (blockers.length > 0) {
    issues.push("completed sign-off must not include unresolvedBlockers");
  }

  return {
    status: issues.length > 0 ? "failed" : "human-reviewed",
    path,
    reviewer: signoff.reviewer?.name || null,
    reviewedAt: signoff.reviewedAt || null,
    platform: signoff.platform || null,
    assistiveTechnology: signoff.assistiveTechnology || null,
  };
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function relativePath(path) {
  return path.startsWith(root) ? path.replace(`${root}/`, "") : path;
}
