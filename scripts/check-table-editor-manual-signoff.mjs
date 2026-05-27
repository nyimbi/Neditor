import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const currentSourceCommit = gitCommit();
const sourceTreeClean = gitTreeClean();
const outputDir = join(root, ".tmp", "table-editor");
const templatePath = join(outputDir, "manual-review-template.json");
const summaryPath = join(outputDir, "manual-review-summary.json");
const completedSignoffPath = process.env.NEDITOR_TABLE_EDITOR_SIGNOFF
  ? resolve(process.env.NEDITOR_TABLE_EDITOR_SIGNOFF)
  : null;

const prerequisiteReports = {
  specCompletion: readReport(join(root, ".tmp", "spec-completion", "report.json"), ["partial-with-release-risks", "complete"]),
  browserWorkflows: readReport(join(root, ".tmp", "e2e-browser", "report.json"), ["passed"]),
  renderedExports: readReport(join(root, ".tmp", "rendered-export-audit", "rendered-export-audit-report.json"), [
    "passed",
    "complete",
  ]),
  desktopSmoke: readReport(join(root, ".tmp", "desktop-smoke", "native-workflow-report.json"), ["passed"]),
};

const reviewSessions = [
  {
    id: "source-to-grid-round-trip",
    label:
      "Edit an existing Markdown pipe table in source, verify the visual grid reloads the same table, then apply grid edits back to the original source range.",
  },
  {
    id: "grid-to-source-round-trip",
    label:
      "Create a visual table, add/remove/reorder rows and columns, edit alignment/format/formula cells, then verify readable Markdown source is written and reparsed.",
  },
  {
    id: "concurrent-source-protection",
    label:
      "While a visual draft is dirty, change the source table directly and verify NEditor blocks silent overwrite until the reviewer reloads or explicitly applies over current source.",
  },
  {
    id: "spreadsheet-and-data-exchange",
    label:
      "Import CSV/XLSX table data, edit source and grid values, export CSV/XLSX, and verify edited source values rather than stale visual data are exported.",
  },
  {
    id: "rendered-export-review",
    label:
      "Open the edited-table rendered audit artifacts in native/browser viewers and verify values, formulas, escaped pipes, alignment, captions, and manifests survive.",
  },
  {
    id: "supported-host-review",
    label:
      "Repeat the table workflows in each supported host available for release validation and record platform-specific viewer or WebDriver evidence.",
  },
];

const checklist = [
  {
    id: "two-way-source-grid",
    label: "Markdown source edits and visual grid edits round-trip without changing the wrong table.",
  },
  {
    id: "dirty-edit-guards",
    label: "Dirty source/grid states, invalid table source, and concurrent edits cannot be lost by switching tables or creating a new table.",
  },
  {
    id: "formula-and-summary-rows",
    label: "Supported formula and summary rows calculate/export correctly, and unsupported formulas produce understandable guidance.",
  },
  {
    id: "alignment-escaping-readable-markdown",
    label: "Alignment markers, escaped pipes, captions, labels, and readable Markdown table formatting are preserved.",
  },
  {
    id: "spreadsheet-exchange",
    label: "CSV/XLSX import and export use the current edited table content, including dirty but valid source text.",
  },
  {
    id: "export-artifact-fidelity",
    label: "HTML/PDF/DOCX/PPTX/Markdown-bundle outputs preserve edited tables with inspectable manifest evidence.",
  },
  {
    id: "accessibility-keyboard",
    label: "Keyboard-only and assistive-technology traversal covers table actions, grid cells, source editing, and status messages.",
  },
  {
    id: "supported-hosts",
    label: "Supported OS/webview hosts either pass table workflows or have documented, release-reviewed host limitations.",
  },
];

mkdirSync(outputDir, { recursive: true });

const issues = [];
writeFileSync(templatePath, `${JSON.stringify(createTemplate(), null, 2)}\n`);
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
      issues.push(`cannot accept completed sign-off because prerequisite report ${name} is ${report.status}`);
    }
  }
}

const summary = {
  schema: "neditor.table-editor.manual-review-summary.v1",
  generatedAt: new Date().toISOString(),
  status: issues.length > 0 ? "failed" : humanSignoff.status,
  appVersion: packageJson.version,
  sourceCommit: currentSourceCommit,
  sourceTreeClean,
  template: relativePath(templatePath),
  completedSignoffPath,
  prerequisiteReports,
  reviewSessions: reviewSessions.map((session) => session.id),
  checklist: checklist.map((item) => item.id),
  humanSignoff,
  issues,
};

writeFileSync(summaryPath, `${JSON.stringify(summary, null, 2)}\n`);

if (issues.length > 0) {
  console.error("Table editor manual sign-off validation failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  console.error(`Wrote ${relativePath(summaryPath)}.`);
  process.exit(1);
}

if (completedSignoffPath) {
  console.log(`Table editor manual sign-off accepted; wrote ${relativePath(summaryPath)}.`);
} else {
  console.log(`Table editor manual review template written to ${relativePath(templatePath)}.`);
  console.log(`No NEDITOR_TABLE_EDITOR_SIGNOFF supplied; summary remains pending at ${relativePath(summaryPath)}.`);
}

function createTemplate() {
  return {
    schema: "neditor.table-editor.manual-signoff.v1",
    appVersion: packageJson.version,
    sourceCommit: currentSourceCommit || "replace-with-current-git-commit",
    sourceTreeClean,
    reviewer: {
      name: "",
      role: "",
      organization: "",
    },
    reviewedAt: new Date().toISOString(),
    platform: {
      os: process.platform,
      version: "",
      device: "",
      webviewOrBrowser: "",
    },
    prerequisiteReports,
    reviewSessions: reviewSessions.map((session) => ({
      id: session.id,
      label: session.label,
      status: "pending",
      platform: {
        os: process.platform,
        version: "",
        device: "",
        webviewOrBrowser: "",
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
    supportedHostResults: [
      {
        platform: process.platform,
        status: "pending",
        evidenceReference: "",
        limitations: [],
        notes: "",
      },
    ],
    unresolvedBlockers: [],
  };
}

function validateCompletedSignoff(path, issues) {
  let signoff;
  try {
    signoff = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    issues.push(`could not read completed sign-off: ${error}`);
    return failedHumanSignoff(path);
  }

  if (signoff.schema !== "neditor.table-editor.manual-signoff.v1") {
    issues.push("completed sign-off schema must be neditor.table-editor.manual-signoff.v1");
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
  if (sourceTreeClean !== true) {
    issues.push("current source tree must be clean before accepting completed table editor sign-off");
  }
  if (!substantiveText(signoff.reviewer?.name)) issues.push("completed sign-off must include reviewer.name");
  if (!substantiveText(signoff.reviewer?.role)) issues.push("completed sign-off must include reviewer.role");
  if (!isIsoDate(signoff.reviewedAt)) issues.push("completed sign-off must include reviewedAt as an ISO date string");
  if (!substantiveText(signoff.platform?.os)) issues.push("completed sign-off must include platform.os");
  if (!substantiveText(signoff.platform?.version)) issues.push("completed sign-off must include platform.version");
  if (!substantiveText(signoff.platform?.device)) issues.push("completed sign-off must include platform.device");
  if (!substantiveText(signoff.platform?.webviewOrBrowser)) {
    issues.push("completed sign-off must include platform.webviewOrBrowser");
  }

  validatePrerequisiteIdentity(signoff.prerequisiteReports, issues);
  validateReviewSessions(signoff.reviewSessions, issues);
  validateChecklist(signoff.checklist, issues);
  validateSupportedHosts(signoff.supportedHostResults, issues);

  const blockers = Array.isArray(signoff.unresolvedBlockers) ? signoff.unresolvedBlockers : [];
  if (blockers.length > 0) issues.push("completed sign-off must not include unresolvedBlockers");

  return {
    status: issues.length > 0 ? "failed" : "human-reviewed",
    path,
    reviewer: signoff.reviewer?.name || null,
    reviewedAt: signoff.reviewedAt || null,
    platform: signoff.platform || null,
    reviewSessions: Array.isArray(signoff.reviewSessions) ? signoff.reviewSessions.map((session) => session.id) : [],
    supportedHostResults: Array.isArray(signoff.supportedHostResults)
      ? signoff.supportedHostResults.map((host) => ({ platform: host.platform, status: host.status }))
      : [],
  };
}

function failedHumanSignoff(path) {
  return {
    status: "failed",
    path,
    reviewer: null,
    reviewedAt: null,
  };
}

function validatePrerequisiteIdentity(submittedReports, issues) {
  if (!submittedReports || typeof submittedReports !== "object") {
    issues.push("completed sign-off must include prerequisiteReports from the generated template");
    return;
  }
  for (const [key, current] of Object.entries(prerequisiteReports)) {
    const submitted = submittedReports[key];
    if (!submitted) {
      issues.push(`completed sign-off is missing prerequisiteReports.${key}`);
      continue;
    }
    if (submitted.path !== current.path) issues.push(`prerequisiteReports.${key}.path must match the current report`);
    if (submitted.status !== current.status) issues.push(`prerequisiteReports.${key}.status must match the current report`);
    if (submitted.generatedAt !== current.generatedAt) {
      issues.push(`prerequisiteReports.${key}.generatedAt must match the current report`);
    }
    if (submitted.sha256 !== current.sha256) issues.push(`prerequisiteReports.${key}.sha256 must match the current report`);
  }
}

function validateReviewSessions(sessions, issues) {
  if (!Array.isArray(sessions)) {
    issues.push("completed sign-off must include reviewSessions");
    return;
  }
  const submitted = new Map(sessions.map((session) => [session.id, session]));
  for (const required of reviewSessions) {
    const session = submitted.get(required.id);
    if (!session) {
      issues.push(`completed sign-off is missing review session ${required.id}`);
      continue;
    }
    if (session.status !== "pass") issues.push(`review session ${required.id} must be pass`);
    if (!substantiveText(session.platform?.os)) issues.push(`review session ${required.id} must include platform.os`);
    if (!substantiveText(session.platform?.version)) issues.push(`review session ${required.id} must include platform.version`);
    if (!substantiveText(session.platform?.device)) issues.push(`review session ${required.id} must include platform.device`);
    if (!substantiveText(session.platform?.webviewOrBrowser)) {
      issues.push(`review session ${required.id} must include platform.webviewOrBrowser`);
    }
    if (Number(session.durationMinutes || 0) <= 0) {
      issues.push(`review session ${required.id} must include a positive durationMinutes value`);
    }
    if (!substantiveText(session.evidenceReference)) {
      issues.push(`review session ${required.id} must include an evidenceReference`);
    }
    if (!substantiveText(session.notes)) issues.push(`review session ${required.id} must include substantive notes`);
    const blockers = Array.isArray(session.blockers) ? session.blockers : [];
    if (blockers.length > 0) issues.push(`review session ${required.id} must not include unresolved blockers`);
  }
}

function validateChecklist(items, issues) {
  if (!Array.isArray(items)) {
    issues.push("completed sign-off must include checklist");
    return;
  }
  const submitted = new Map(items.map((item) => [item.id, item]));
  for (const expected of checklist) {
    const item = submitted.get(expected.id);
    if (!item) {
      issues.push(`completed sign-off is missing checklist item ${expected.id}`);
      continue;
    }
    if (item.status !== "pass") issues.push(`checklist item ${expected.id} must be pass`);
    if (!substantiveText(item.notes)) issues.push(`checklist item ${expected.id} must include substantive reviewer notes`);
  }
}

function validateSupportedHosts(hosts, issues) {
  if (!Array.isArray(hosts) || hosts.length === 0) {
    issues.push("completed sign-off must include supportedHostResults");
    return;
  }
  const passingHosts = hosts.filter((host) => host.status === "pass");
  if (passingHosts.length === 0) issues.push("supportedHostResults must include at least one passing host");
  for (const host of hosts) {
    if (!substantiveText(host.platform)) issues.push("each supportedHostResults item must include platform");
    if (!["pass", "documented-limitation"].includes(host.status)) {
      issues.push(`supported host ${host.platform || "unknown"} status must be pass or documented-limitation`);
    }
    if (!substantiveText(host.evidenceReference)) {
      issues.push(`supported host ${host.platform || "unknown"} must include evidenceReference`);
    }
    if (host.status === "documented-limitation") {
      const limitations = Array.isArray(host.limitations) ? host.limitations : [];
      if (limitations.length === 0) {
        issues.push(`supported host ${host.platform || "unknown"} limitation must include limitations`);
      }
    }
    if (!substantiveText(host.notes)) issues.push(`supported host ${host.platform || "unknown"} must include notes`);
  }
}

function readReport(path, expectedStatus) {
  if (!existsSync(path)) {
    return {
      path: relativePath(path),
      status: "missing",
      generatedAt: null,
      sha256: null,
    };
  }
  try {
    const raw = readFileSync(path, "utf8");
    const report = JSON.parse(raw);
    const expected = Array.isArray(expectedStatus) ? expectedStatus : [expectedStatus];
    const actualStatus = report.status || report.payload?.status || inferReportStatus(report);
    return {
      path: relativePath(path),
      status: expected.includes(actualStatus) ? "passed" : actualStatus || "unknown",
      generatedAt: report.generatedAt || report.payload?.generatedAt || null,
      sha256: sha256Text(raw),
    };
  } catch (error) {
    return {
      path: relativePath(path),
      status: "invalid-json",
      generatedAt: null,
      sha256: null,
      error: String(error),
    };
  }
}

function inferReportStatus(report) {
  const targetCount = Array.isArray(report.targets) ? report.targets.length : 0;
  const reviewCases = Array.isArray(report.reviewCases) ? report.reviewCases : [];
  if (targetCount >= 10 && reviewCases.some((reviewCase) => reviewCase?.slug === "edited-tables")) {
    return "passed";
  }
  return null;
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
  return result.status === 0 ? result.stdout.trim() : "";
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: root,
    encoding: "utf8",
  });
  return result.status === 0 && result.stdout.trim() === "";
}
