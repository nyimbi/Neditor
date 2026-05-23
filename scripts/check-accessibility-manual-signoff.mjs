import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const currentSourceCommit = gitCommit();
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
const requiredReviewSessions = [
  {
    id: "screen-reader-navigation",
    label: "Screen-reader navigation through the main workbench and document regions.",
  },
  {
    id: "keyboard-only-navigation",
    label: "Keyboard-only operation for commands, panels, dialogs, search, outline, and export readiness.",
  },
  {
    id: "native-desktop-shell",
    label: "Native desktop shell, menus, toolbar rows, file workflows, and outline mode with platform assistive technology.",
  },
  {
    id: "export-artifact-review",
    label: "Accessible review of exported HTML/PDF/DOCX/PPTX/publishing/handoff artifacts.",
  },
];

mkdirSync(outputDir, { recursive: true });

const issues = [];
const prerequisiteReports = {
  static: readReport(staticReportPath, ["pass", "passed"]),
  runtime: readReport(runtimeReportPath, "passed"),
};
writeFileSync(templatePath, `${JSON.stringify(createTemplate(prerequisiteReports), null, 2)}\n`);
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

function createTemplate(prerequisiteReports) {
  return {
    schema: "neditor.accessibility.manual-signoff.v1",
    appVersion: packageJson.version,
    sourceCommit: currentSourceCommit || "replace-with-current-git-commit",
    sourceTreeClean: true,
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
    prerequisiteReports,
    reviewSessions: requiredReviewSessions.map((session) => ({
      id: session.id,
      label: session.label,
      status: "pending",
      assistiveTechnology: {
        name: "",
        version: "",
        settings: "",
      },
      platform: {
        os: process.platform,
        version: "",
        device: "",
      },
      browserOrWebview: {
        name: "",
        version: "",
      },
      durationMinutes: 0,
      evidenceReference: "",
      notes: "",
      blockers: [],
    })),
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
      sha256: null,
    };
  }
  try {
    const raw = readFileSync(path, "utf8");
    const report = JSON.parse(raw);
    const expected = Array.isArray(expectedStatus) ? expectedStatus : [expectedStatus];
    return {
      path: relativePath(path),
      status: expected.includes(report.status) ? "passed" : report.status || "unknown",
      generatedAt: report.generatedAt || null,
      sha256: sha256Text(raw),
    };
  } catch (error) {
    return {
      path: relativePath(path),
      status: "invalid-json",
      sha256: null,
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
  if (signoff.appVersion !== packageJson.version) {
    issues.push(`completed sign-off appVersion must match package.json version ${packageJson.version}`);
  }
  if (signoff.sourceCommit !== currentSourceCommit) {
    issues.push(`completed sign-off sourceCommit must match current git commit ${currentSourceCommit}`);
  }
  if (signoff.sourceTreeClean !== true) {
    issues.push("completed sign-off sourceTreeClean must be true");
  }
  if (!signoff.reviewer?.name?.trim()) {
    issues.push("completed sign-off must include reviewer.name");
  }
  if (!signoff.reviewer?.role?.trim()) {
    issues.push("completed sign-off must include reviewer.role");
  }
  if (!isIsoDate(signoff.reviewedAt)) {
    issues.push("completed sign-off must include reviewedAt as an ISO date string");
  }
  if (!signoff.platform?.os?.trim()) {
    issues.push("completed sign-off must include platform.os");
  }
  if (!signoff.platform?.version?.trim()) {
    issues.push("completed sign-off must include platform.version");
  }
  if (!signoff.platform?.device?.trim()) {
    issues.push("completed sign-off must include platform.device");
  }
  if (!signoff.assistiveTechnology?.name?.trim()) {
    issues.push("completed sign-off must include assistiveTechnology.name");
  }
  if (!signoff.assistiveTechnology?.version?.trim()) {
    issues.push("completed sign-off must include assistiveTechnology.version");
  }
  if (!signoff.browserOrWebview?.name?.trim()) {
    issues.push("completed sign-off must include browserOrWebview.name");
  }
  if (!signoff.browserOrWebview?.version?.trim()) {
    issues.push("completed sign-off must include browserOrWebview.version");
  }
  validatePrerequisiteIdentity(signoff.prerequisiteReports, issues);
  validateReviewSessions(signoff.reviewSessions, issues);

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
    if (!substantiveText(item.notes)) {
      issues.push(`checklist item ${expected.id} must include substantive reviewer notes`);
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
    reviewSessions: Array.isArray(signoff.reviewSessions) ? signoff.reviewSessions.map((session) => session.id) : [],
  };
}

function validateReviewSessions(sessions, issues) {
  if (!Array.isArray(sessions)) {
    issues.push("completed sign-off must include reviewSessions");
    return;
  }
  const submittedSessions = new Map(sessions.map((session) => [session.id, session]));
  for (const required of requiredReviewSessions) {
    const session = submittedSessions.get(required.id);
    if (!session) {
      issues.push(`completed sign-off is missing review session ${required.id}`);
      continue;
    }
    if (session.status !== "pass") {
      issues.push(`review session ${required.id} must be pass before sign-off can be accepted`);
    }
    if (!session.assistiveTechnology?.name?.trim()) {
      issues.push(`review session ${required.id} must include assistiveTechnology.name`);
    }
    if (!session.assistiveTechnology?.version?.trim()) {
      issues.push(`review session ${required.id} must include assistiveTechnology.version`);
    }
    if (!session.platform?.os?.trim() || !session.platform?.version?.trim() || !session.platform?.device?.trim()) {
      issues.push(`review session ${required.id} must include platform os, version, and device`);
    }
    if (!session.browserOrWebview?.name?.trim() || !session.browserOrWebview?.version?.trim()) {
      issues.push(`review session ${required.id} must include browserOrWebview name and version`);
    }
    if (Number(session.durationMinutes || 0) <= 0) {
      issues.push(`review session ${required.id} must include a positive durationMinutes value`);
    }
    if (!substantiveText(session.evidenceReference)) {
      issues.push(`review session ${required.id} must include an evidenceReference`);
    }
    if (!substantiveText(session.notes)) {
      issues.push(`review session ${required.id} must include substantive notes`);
    }
    const blockers = Array.isArray(session.blockers) ? session.blockers : [];
    if (blockers.length > 0) {
      issues.push(`review session ${required.id} must not include unresolved blockers`);
    }
  }
}

function validatePrerequisiteIdentity(submittedReports, issues) {
  if (!submittedReports || typeof submittedReports !== "object") {
    issues.push("completed sign-off must include prerequisiteReports from the generated template");
    return;
  }
  for (const key of ["static", "runtime"]) {
    const current = prerequisiteReports[key];
    const submitted = submittedReports[key];
    if (!submitted) {
      issues.push(`completed sign-off is missing prerequisiteReports.${key}`);
      continue;
    }
    if (submitted.path !== current.path) {
      issues.push(`completed sign-off prerequisiteReports.${key}.path must match the current report`);
    }
    if (submitted.status !== current.status) {
      issues.push(`completed sign-off prerequisiteReports.${key}.status must match the current report`);
    }
    if (submitted.generatedAt !== current.generatedAt) {
      issues.push(`completed sign-off prerequisiteReports.${key}.generatedAt must match the current report`);
    }
    if (submitted.sha256 !== current.sha256) {
      issues.push(`completed sign-off prerequisiteReports.${key}.sha256 must match the current report`);
    }
  }
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function substantiveText(value) {
  return typeof value === "string" && value.trim().length >= 12;
}

function relativePath(path) {
  return path.startsWith(root) ? path.replace(`${root}/`, "") : path;
}

function sha256Text(text) {
  return createHash("sha256").update(text).digest("hex");
}

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0) return "";
  return result.stdout.trim();
}
