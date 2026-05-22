import { deepEqual, equal, ok } from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  beginLatestDocumentTask,
  cancelLatestDocumentTask,
  isLatestDocumentTaskCurrent,
  type LatestDocumentTaskGate,
} from "../src/lib/asyncGuards.js";
import {
  bibliographyEntryStub,
  bibliographyStubsForMissingKeys,
  citationReferenceSnippet,
  normalizeCitationKey,
} from "../src/lib/bibliographyManager.js";
import { buildConflictDiff } from "../src/lib/conflict.js";
import { createDebouncedTextCommit, PREVIEW_DEBOUNCE_MS } from "../src/lib/debounce.js";
import { markdownListContinuation } from "../src/lib/markdownEditing.js";
import {
  builtinTransformTemplates,
  normalizeCustomTransformTemplates,
  transformTemplateFillFields,
  transformTemplateMarkdown,
} from "../src/lib/transformTemplates.js";
import { migratePersistedWorkspace, normalizeCitationStyle, WORKSPACE_SCHEMA_VERSION } from "../src/lib/workspacePersistence.js";
import {
  appendConflictMergeLine,
  appendConflictMergePart,
  applyAiPasteInsertion,
  moveConflictMergePart,
  quoteMarkdown,
  removeConflictMergePart,
  renderConflictMergeParts,
  type ConflictMergePart,
} from "../src/lib/workflows.js";
import {
  formatTableTotal,
  parseTableCellSpan,
  parseMarkdownTables,
  parseTablePaste,
  serializeMarkdownTable,
  setTableCellSpan,
  sortTableDraftRows,
  tableColumnRange,
  validateTableDraft,
  type TableDraft,
} from "../src/lib/tables.js";

test("table parsing preserves captions, alignment, and escaped pipes", () => {
  const [table] = parseMarkdownTables(
    'Table: Regional revenue {#tbl:revenue}\n| Region | Revenue | Note |\n| :--- | ---: | --- |\n| East | $1,200 | margin\\|stable |\n',
  );

  equal(table.id, "tbl:revenue");
  equal(table.caption, "Regional revenue");
  deepEqual(table.alignments, ["left", "right", "left"]);
  deepEqual(table.rows[0], ["East", "$1,200", "margin|stable"]);
});

test("table paste handles quoted CSV and markdown table captions", () => {
  deepEqual(parseTablePaste('"Region, code",Revenue\n"East, KE",1200').rows, [
    ["Region, code", "Revenue"],
    ["East, KE", "1200"],
  ]);

  const markdown = parseTablePaste("Table: Sales {#tbl:sales}\n| Region | Revenue |\n| --- | ---: |\n| West | 900 |");
  equal(markdown.id, "tbl:sales");
  equal(markdown.caption, "Sales");
  deepEqual(markdown.alignments, ["left", "right"]);
});

test("table validation and formatting cover editor formulas and totals", () => {
  const draft: TableDraft = {
    id: "tbl revenue",
    caption: "Revenue",
    headers: ["Region", "Revenue"],
    alignments: ["left", "right"],
    formats: ["text", "currency"],
    rows: [
      ["East", "$1,200"],
      ["West", "$800"],
      ["Total", "=SUM(B1:B3)"],
    ],
  };

  const issues = validateTableDraft(draft);
  ok(issues.some((issue) => issue.message.includes("Table id must start")));
  ok(issues.some((issue) => issue.message.includes("outside the editable data range")));
  equal(formatTableTotal(draft, 1), "$2000");
  equal(tableColumnRange(27, 12), "AB1:AB12");
  deepEqual(serializeMarkdownTable({ ...draft, id: "tbl:revenue" }).slice(0, 3), [
    "Table: Revenue {#tbl:revenue}",
    "| Region | Revenue |",
    "| --- | ---: |",
  ]);
});

test("table span helpers preserve merged-cell attributes through serialization", () => {
  const draft: TableDraft = {
    id: "tbl:merged",
    caption: "Merged plan",
    headers: ["Phase", "Scope", "Owner"],
    alignments: ["left", "left", "left"],
    formats: ["text", "text", "text"],
    rows: [
      [setTableCellSpan("Discovery", 2, 1), "", "PM"],
      [setTableCellSpan("Delivery", 1, 2), "Build", "Lead"],
      ["", "Launch", "Ops"],
    ],
  };

  deepEqual(parseTableCellSpan(draft.rows[0][0]), {
    text: "Discovery",
    colspan: 2,
    rowspan: 1,
  });
  equal(setTableCellSpan(draft.rows[0][0], 1, 1), "Discovery");
  ok(!validateTableDraft(draft).some((issue) => issue.severity === "error"));
  ok(serializeMarkdownTable(draft).join("\n").includes("Discovery {colspan=2}"));
  ok(serializeMarkdownTable(draft).join("\n").includes("Delivery {rowspan=2}"));
});

test("table draft sorting preserves summary rows and typed ordering", () => {
  const draft: TableDraft = {
    id: "tbl:sort",
    caption: "Sort behavior",
    headers: ["Region", "Revenue", "Due"],
    alignments: ["left", "right", "left"],
    formats: ["text", "currency", "date"],
    rows: [
      ["West", "$800", "2026-05-03"],
      ["East", "$1,200", "2026-05-01"],
      ["North", "$950", "2026-05-02"],
      ["Total", "=SUM(B1:B3)", ""],
    ],
  };

  deepEqual(sortTableDraftRows(draft, 1, "desc").rows.map((row) => row[0]), ["East", "North", "West", "Total"]);
  deepEqual(sortTableDraftRows(draft, 2, "asc").rows.map((row) => row[0]), ["East", "North", "West", "Total"]);
  deepEqual(sortTableDraftRows(draft, 0, "asc").rows.map((row) => row[0]), ["East", "North", "West", "Total"]);
});

test("bibliography manager helpers generate repairable citation snippets", () => {
  equal(normalizeCitationKey(" @Risk 2026! "), "Risk-2026");
  equal(citationReferenceSnippet("@porter1985", "p. 42"), "[@porter1985, p. 42]");
  equal(citationReferenceSnippet(""), "");
  equal(
    bibliographyEntryStub({
      key: "porter1985",
      title: "Competitive {Advantage}",
      author: "Porter, Michael E.",
      issued: "1985-01-01",
    }),
    "@misc{porter1985,\n  title = {Competitive Advantage},\n  author = {Porter, Michael E.},\n  year = {1985}\n}",
  );
  equal(
    bibliographyStubsForMissingKeys(["@missing2026", "missing2026", "other key"]),
    "```bibtex\n@misc{missing2026,\n  title = {TODO: Add title},\n  author = {TODO},\n  year = {TODO}\n}\n\n@misc{other-key,\n  title = {TODO: Add title},\n  author = {TODO},\n  year = {TODO}\n}\n```\n",
  );
});

test("conflict diff keeps local and external edits aligned for merge UI", () => {
  const rows = buildConflictDiff("alpha\nlocal\nomega", "alpha\nexternal\nomega");

  deepEqual(
    rows.map((row) => row.kind),
    ["equal", "local", "external", "equal"],
  );
  equal(rows[1].local, "local");
  equal(rows[2].external, "external");
  equal(rows[3].localLine, 3);
  equal(rows[3].externalLine, 3);
});

test("conflict merge helpers compose selected local and external lines", () => {
  const rows = buildConflictDiff("alpha\nlocal\nomega", "alpha\nexternal\nomega");
  let merged = "";

  merged = appendConflictMergeLine(merged, rows[0], "local");
  merged = appendConflictMergeLine(merged, rows[1], "local");
  merged = appendConflictMergeLine(merged, rows[2], "external");
  merged = appendConflictMergeLine(merged, rows[3], "external");

  equal(merged, "alpha\nlocal\nexternal\nomega");
  equal(appendConflictMergeLine(merged, rows[1], "external"), merged);
});

test("conflict merge composition helpers preserve blank lines and ordering", () => {
  const rows = buildConflictDiff("alpha\n\nlocal\nomega", "alpha\nexternal\n\nomega");
  let parts: ConflictMergePart[] = [];

  parts = appendConflictMergePart(parts, rows[0], "local");
  parts = appendConflictMergePart(parts, rows[1], "external");
  parts = appendConflictMergePart(parts, rows[2], "local");
  parts = appendConflictMergePart(parts, rows[3], "local");
  parts = appendConflictMergePart(parts, rows[4], "external");
  parts = appendConflictMergePart(parts, rows[4], "external");

  equal(parts.length, 5);
  equal(renderConflictMergeParts(parts), "alpha\nexternal\n\nlocal\nomega");

  parts = moveConflictMergePart(parts, parts[3].id, -1);
  equal(renderConflictMergeParts(parts), "alpha\nexternal\nlocal\n\nomega");

  parts = removeConflictMergePart(parts, parts[1].id);
  equal(renderConflictMergeParts(parts), "alpha\nlocal\n\nomega");
});

test("AI paste insertion modes preserve workflow-specific output", () => {
  equal(quoteMarkdown("alpha\n\nbeta"), "> alpha\n>\n> beta");
  equal(applyAiPasteInsertion("# Report", "Cleaned", "insert"), "# Report\n\nCleaned\n");
  equal(applyAiPasteInsertion("# Report", "Cleaned", "replace"), "Cleaned");
  equal(applyAiPasteInsertion("# Report", "Cleaned", "quote"), "# Report\n\n> Cleaned\n");
  equal(applyAiPasteInsertion("# Report", "Cleaned", "appendix"), "# Report\n\n## AI Draft Appendix\n\nCleaned\n");
});

test("latest document task guard rejects stale and cancelled compile results", () => {
  const gate: LatestDocumentTaskGate = { sequence: 0 };
  const firstDocument = { id: "doc-1", text: "first draft" };
  const first = beginLatestDocumentTask(gate, firstDocument);

  ok(isLatestDocumentTaskCurrent(gate, first, firstDocument));
  ok(!isLatestDocumentTaskCurrent(gate, first, { id: "doc-1", text: "second draft" }));
  ok(!isLatestDocumentTaskCurrent(gate, first, { id: "doc-2", text: "first draft" }));

  const second = beginLatestDocumentTask(gate, { id: "doc-1", text: "second draft" });
  ok(!isLatestDocumentTaskCurrent(gate, first, { id: "doc-1", text: "first draft" }));
  ok(isLatestDocumentTaskCurrent(gate, second, { id: "doc-1", text: "second draft" }));

  cancelLatestDocumentTask(gate);
  ok(!isLatestDocumentTaskCurrent(gate, second, { id: "doc-1", text: "second draft" }));
});

test("markdown list continuation handles tasks numbers and blockquotes", () => {
  deepEqual(markdownListContinuation("- First item"), { kind: "continue", insert: "\n- " });
  deepEqual(markdownListContinuation("  3) Third item"), { kind: "continue", insert: "\n  4) " });
  deepEqual(markdownListContinuation("- [x] Completed task"), { kind: "continue", insert: "\n- [ ] " });
  deepEqual(markdownListContinuation("> - Quoted item"), { kind: "continue", insert: "\n> - " });
  deepEqual(markdownListContinuation("> 2. Quoted numbered item"), { kind: "continue", insert: "\n> 3. " });
  deepEqual(markdownListContinuation("  - "), { kind: "exit", fromColumn: 0, replacement: "  " });
  deepEqual(markdownListContinuation("> - [ ] "), { kind: "exit", fromColumn: 2, replacement: "" });
  equal(markdownListContinuation("plain paragraph"), null);
});

test("preview debounce coalesces edits inside the spec timing budget", () => {
  ok(PREVIEW_DEBOUNCE_MS <= 100);
  const commits: string[] = [];
  let nextHandle = 1;
  const scheduled = new Map<number, { callback: () => void; delayMs: number }>();
  const debounce = createDebouncedTextCommit((text) => commits.push(text), {
    setTimeout(callback, delayMs) {
      const handle = nextHandle;
      nextHandle += 1;
      scheduled.set(handle, { callback, delayMs });
      return handle;
    },
    clearTimeout(handle) {
      scheduled.delete(handle);
    },
  });

  debounce.schedule("first");
  debounce.schedule("second");
  equal(commits.length, 0);
  equal(scheduled.size, 1);
  const [job] = [...scheduled.values()];
  equal(job.delayMs, PREVIEW_DEBOUNCE_MS);
  job.callback();
  deepEqual(commits, ["second"]);
  equal(scheduled.size, 0);

  debounce.schedule("third");
  debounce.flush("forced");
  deepEqual(commits, ["second", "forced"]);
  equal(scheduled.size, 0);

  debounce.schedule("cancelled");
  debounce.cancel();
  equal(scheduled.size, 0);
  deepEqual(commits, ["second", "forced"]);
});

test("workspace persistence migration versions and normalizes saved settings", () => {
  const migrated = migratePersistedWorkspace({
    schemaVersion: 1,
    theme: "solarized",
    previewTheme: "dark",
    toolbarDisplay: "icons",
    toolbarTextSize: 20,
    codeFolding: false,
    editorPaneRatio: 0.95,
    editorFontSize: 99,
    previewLineHeight: 0.2,
    autosaveDelayMs: 10,
    snapshotIntervalMs: 9_999_999,
    exportTarget: "google-docs",
    exportDefaults: {
      includeManifest: false,
      includeCoverPage: false,
      includePageNumbers: false,
      layoutPreset: "compact",
    },
    bibliographyDefaults: { citationStyle: "APA" },
    brandProfileDefaults: { color: "  #123456  ", watermark: "Draft" },
    gitIntegration: { enabled: false },
    aiCleanupDefaults: { preserveHeadings: true, convertTables: false },
    recentFiles: ["/a.md", 42, "/a.md", "/b.md"],
    recentFolders: ["/workspace", ""],
    workspacePath: "/legacy/workspace",
    openFiles: ["/a.md", "/b.md"],
    activeFile: "/b.md",
    scrollPositions: {
      "/a.md": { editor: 2, preview: -1 },
      "/ignored.md": "not-a-position",
    },
    mode: "presentation",
    sidebar: "settings",
    transformEnginePaths: { dot: "/usr/bin/dot", bad: 10 },
    trustedTransformEngines: { dot: true, bad: "yes" },
    disabledTransformEngines: { d2: true, dot: false, bad: "no" },
    transformInputModes: { dot: "stdin", bad: "pipe" },
    transformTimeoutMs: 99_999,
    customTransformTemplates: [
      {
        id: "custom-margin",
        name: "Custom margin",
        category: "Finance",
        transform: "calc",
        summary: "Reusable margin block.",
        body: "```calc\nrevenue = 1\ncost = 1\n```\n",
        tags: ["margin", 42],
      },
      { id: "missing-body", name: "Ignored" },
    ],
  });

  equal(migrated.schemaVersion, WORKSPACE_SCHEMA_VERSION);
  equal(migrated.theme, undefined);
  equal(migrated.previewTheme, "dark");
  equal(migrated.toolbarDisplay, "icons");
  equal(migrated.toolbarTextSize, 15);
  equal(migrated.codeFolding, false);
  equal(migrated.editorPaneRatio, 0.75);
  equal(migrated.editorFontSize, 22);
  equal(migrated.previewLineHeight, 1);
  equal(migrated.autosaveDelayMs, 500);
  equal(migrated.snapshotIntervalMs, 3_600_000);
  equal(migrated.exportTarget, "google-docs");
  deepEqual(migrated.exportDefaults, {
    includeManifest: false,
    includeStyles: true,
    includeSyntaxHighlighting: true,
    coverPage: false,
    pageNumbers: false,
    layoutPreset: "compact",
    includeComments: true,
    includeProvenance: true,
    includeGlossary: true,
    includeAgenda: true,
  });
  deepEqual(migrated.bibliographyDefaults, { citationStyle: "apa" });
  equal(normalizeCitationStyle("numeric"), "numeric");
  equal(normalizeCitationStyle("ieee"), "ieee");
  equal(normalizeCitationStyle("unknown-style"), "title");
  equal(migrated.brandProfileDefaults?.color, "#123456");
  equal(migrated.brandProfileDefaults?.watermark, "Draft");
  deepEqual(migrated.gitIntegration, { enabled: false, warnOnDirtyExport: true });
  deepEqual(migrated.aiCleanupDefaults, {
    addProvenance: true,
    markAsDraft: true,
    insertCitationTodos: true,
    preserveHeadings: true,
    convertNumberedLists: true,
    convertTables: false,
  });
  deepEqual(migrated.recentFiles, ["/a.md", "/b.md"]);
  deepEqual(migrated.recentFolders, ["/workspace"]);
  equal(migrated.workspaceRoot, "/legacy/workspace");
  equal(migrated.activePath, "/b.md");
  deepEqual(migrated.scrollPositions, { "/a.md": { editor: 1, preview: 0 } });
  equal(migrated.mode, "presentation");
  equal(migrated.sidebar, "settings");
  deepEqual(migrated.transformEnginePaths, { dot: "/usr/bin/dot" });
  deepEqual(migrated.trustedTransformEngines, { dot: true });
  deepEqual(migrated.disabledTransformEngines, { d2: true, dot: false });
  deepEqual(migrated.transformInputModes, { dot: "stdin" });
  equal(migrated.transformTimeoutMs, 30_000);
  deepEqual(migrated.customTransformTemplates, [
    {
      id: "custom-margin",
      name: "Custom margin",
      category: "Finance",
      transform: "calc",
      summary: "Reusable margin block.",
      body: "```calc\nrevenue = 1\ncost = 1\n```",
      tags: ["margin"],
    },
  ]);
});

test("transform template library covers reusable calculations and custom template normalization", () => {
  const calcTemplates = builtinTransformTemplates.filter((template) => template.transform === "calc");
  const businessTemplates = calcTemplates.filter((template) => template.category === "Business");
  const scienceTemplates = calcTemplates.filter((template) => template.category === "Science");
  const mathTemplates = calcTemplates.filter((template) => template.category === "Mathematics");

  ok(calcTemplates.length >= 30);
  ok(businessTemplates.length >= 12);
  ok(scienceTemplates.length >= 8);
  ok(mathTemplates.length >= 8);
  for (const transform of ["chart", "vega-lite", "timeline", "roadmap", "adr", "mermaid", "pikchr", "dot", "plantuml", "csv", "json-schema", "openapi", "qr"]) {
    ok(builtinTransformTemplates.some((template) => template.transform === transform), `missing ${transform} template`);
  }
  const doseTemplate = builtinTransformTemplates.find((template) => template.id === "calc-science-dose");
  if (!doseTemplate) throw new Error("missing dose template");
  deepEqual(
    transformTemplateFillFields(doseTemplate).map((field) => field.name),
    ["weight_kg", "dose_mg_per_kg", "tablet_strength_mg"],
  );
  const chartTemplate = builtinTransformTemplates.find((template) => template.transform === "chart");
  if (!chartTemplate) throw new Error("missing chart template");
  ok(transformTemplateFillFields(chartTemplate).some((field) => field.name === "title"));
  ok(transformTemplateMarkdown(builtinTransformTemplates[0]).endsWith("\n"));
  deepEqual(normalizeCustomTransformTemplates([{ id: "x", name: "X", transform: "calc", body: "```calc\nx = 1\n```\n" }]), [
    {
      id: "x",
      name: "X",
      category: "Custom",
      transform: "calc",
      summary: "Reusable transform template.",
      body: "```calc\nx = 1\n```",
      tags: [],
    },
  ]);
});

test("workbench command bar exposes icon display controls and workflow groups", () => {
  const app = readFileSync("src/App.vue", "utf8");
  const store = readFileSync("src/stores/documents.ts", "utf8");
  const types = readFileSync("src/types.ts", "utf8");
  const tauriLib = readFileSync("src-tauri/src/lib.rs", "utf8");

  ok(app.includes(':data-toolbar-display="store.toolbarDisplay"'));
  ok(app.includes(':style="appShellStyle"'));
  ok(app.includes('aria-label="Toolbar button display"'));
  ok(app.includes('aria-label="Toolbar text size"'));
  ok(app.includes("Export HTML"));
  ok(app.includes("exportDocumentAs(\"html\")"));
  ok(app.includes('listen<string>("neditor-menu-command"'));
  ok(app.includes('"neditor-export-html": "html"'));
  ok(tauriLib.includes('SubmenuBuilder::new(app, "Export")'));
  ok(tauriLib.includes('SubmenuBuilder::new(app, "Edit")'));
  ok(tauriLib.includes('SubmenuBuilder::new(app, "View")'));
  ok(tauriLib.includes('"neditor-export-html", "HTML Export"'));
  ok(app.includes('case "neditor-mode-export"'));
  ok(tauriLib.includes('app.emit("neditor-menu-command", id)'));
  ok(app.includes("commandToolbarRows"));
  ok(app.includes("command-toolbar-row"));
  ok(app.includes('foldGutter()'));
  ok(app.includes('codeFolding({ placeholderText: " folded " })'));
  ok(app.includes('class="icon-command"'));
  for (const label of ["Document", "Manage", "Write", "Navigate", "Insert", "Review"]) {
    ok(app.includes(`label: "${label}"`), `missing ${label} command group`);
  }
  for (const icon of ["saveAs", "snapshot", "templates", "equation", "outline", "fold", "unfold", "comment", "html"]) {
    ok(app.includes(`${icon}: [`), `missing ${icon} icon path`);
  }
  ok(app.includes('store.sidebar === \'templates\''));
  ok(store.includes("toolbarTextSize: 10"));
  ok(types.includes("savedText?: string"));
  ok(store.includes('doc.dirty = typeof doc.savedText === "string" ? text !== doc.savedText : fallbackHash(text) !== doc.savedHash'));
  ok(store.includes("doc.savedText = response.text"));
});

test("local verification scripts expose local baseline checks", () => {
  const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
    scripts: Record<string, string>;
  };
  const { scripts } = packageJson;

  equal(scripts.check, "vue-tsc --noEmit");
  equal(scripts["check:a11y"], "node scripts/check-accessibility.mjs");
  equal(scripts["check:deps"], "node scripts/check-dependency-admission.mjs");
  equal(scripts["check:docs"], "node scripts/check-markdown-links.mjs");
  equal(scripts["check:engines"], "node scripts/check-external-engines.mjs");
  equal(scripts["check:e2e-env"], "node scripts/check-e2e-environment.mjs");
  equal(scripts["check:structure"], "node scripts/check-project-structure.mjs");
  equal(scripts["verify:local"], "node scripts/run-local-verification.mjs");
  equal(scripts["verify:local:full"], "node scripts/run-local-verification.mjs --full");
  equal(scripts.build, "vue-tsc --noEmit && vite build");
  equal(scripts["test:desktop-bundle"], "node scripts/check-desktop-bundle.mjs");
  equal(scripts["test:desktop-dmg"], "node scripts/check-desktop-dmg.mjs");
  equal(scripts["test:desktop-smoke"], "node scripts/check-desktop-smoke.mjs");
  equal(scripts["test:tauri-webdriver"], "node scripts/run-tauri-webdriver.mjs");
  equal(scripts["test:rendered-exports"], "node scripts/check-rendered-export-audit.mjs");
  equal(scripts["test:unit"], "tsc -p tsconfig.test.json && node --test .tmp-tests/tests/frontend-unit.test.js");
  equal(scripts["test:e2e"], "node scripts/run-e2e.mjs");
});

test("desktop WebDriver harness covers native restart and export workflows", () => {
  const script = readFileSync("scripts/run-tauri-webdriver.mjs", "utf8");

  ok(script.includes("assertDirtyTitleWorkflow(session)"));
  ok(script.includes("assertTransformTemplateWorkflow(session)"));
  ok(script.includes("assertExportReadinessWorkflow(session)"));
  ok(script.includes("assertHtmlExportWriteWorkflow(session)"));
  ok(script.includes("assertPreferenceRestartWorkflow(session, originalPreferences)"));
  ok(script.includes("NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT"));
  ok(script.includes("native-workflow-export.html"));
  ok(script.includes("desktop WebDriver writes HTML export through dialog-free smoke path"));
  ok(script.includes("desktop template insertion reaches editor and preview"));
  ok(script.includes("Dose by weight"));
  ok(script.includes(".preview-document"));
  ok(script.includes("document.querySelector('.sidebar pre')"));
  ok(script.includes('"export_target": "html"'));
  ok(script.includes("manifest.output_hash"));
  ok(script.includes("workflowPlan: webdriverWorkflowPlan"));
  ok(script.includes("persisted desktop preferences after restart"));
  ok(script.includes("Official Tauri WebDriver currently supports desktop automation on Windows and Linux only"));
});

test("desktop launch smoke records native UI workbench surfaces", () => {
  const app = readFileSync("src/App.vue", "utf8");
  const rust = readFileSync("src-tauri/src/lib.rs", "utf8");
  const smoke = readFileSync("scripts/check-desktop-smoke.mjs", "utf8");

  ok(app.includes("write_desktop_ui_smoke_report"));
  ok(app.includes("desktop_workflow_smoke_enabled"));
  ok(app.includes("write_desktop_workflow_smoke_report"));
  ok(app.includes("desktop_workflow_smoke_file_path"));
  ok(app.includes("desktop_workflow_smoke_export_path"));
  ok(app.includes("emit_desktop_workflow_smoke_menu_command"));
  ok(app.includes("native workflow saved document to real file"));
  ok(app.includes("native workflow opened saved real file"));
  ok(app.includes("native workflow reverted saved real file"));
  ok(app.includes("native workflow reloaded clean external watcher change"));
  ok(app.includes("native workflow restored clean watcher reload"));
  ok(app.includes("native workflow watched included file with native driver"));
  ok(app.includes("native workflow recompiled clean included watcher change"));
  ok(app.includes("native workflow restored included watcher root"));
  ok(app.includes("native workflow blocked stale save with external conflict"));
  ok(app.includes("native workflow rendered conflict modal controls"));
  ok(app.includes("native workflow conflict modal seeded local merge base"));
  ok(app.includes("native workflow conflict modal seeded external merge base"));
  ok(app.includes("native workflow kept local conflict changes"));
  ok(app.includes("native workflow saved kept-local conflict changes"));
  ok(app.includes("native workflow saved local conflict copy"));
  ok(app.includes("native workflow merged external conflict changes"));
  ok(app.includes("native workflow accepted external conflict changes"));
  ok(app.includes("native workflow inserted calc template into source"));
  ok(app.includes("native workflow prepared html export readiness"));
  ok(app.includes("native workflow wrote html export artifact"));
  ok(app.includes("native workflow exported html from native menu command"));
  ok(app.includes("collectNativeModeEvidence"));
  ok(app.includes("[\"split\", \"source\", \"preview\", \"focus\", \"export\", \"review\", \"presentation\"]"));
  ok(app.includes("collectNativeThemeAccessibilityEvidence"));
  ok(app.includes("native workflow applied high contrast attributes and colors"));
  ok(app.includes("native workflow applied preview theme and typography"));
  ok(app.includes("commandLabels"));
  ok(app.includes("#document-workspace"));
  ok(app.includes("#live-preview"));
  ok(app.includes("previewLabel"));
  ok(rust.includes("fn write_desktop_ui_smoke_report"));
  ok(rust.includes("fn desktop_workflow_smoke_enabled"));
  ok(rust.includes("fn write_desktop_workflow_smoke_report"));
  ok(rust.includes("fn emit_desktop_workflow_smoke_menu_command"));
  ok(rust.includes("NEDITOR_DESKTOP_UI_SMOKE_REPORT"));
  ok(rust.includes("NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT"));
  ok(smoke.includes("native-ui-report.json"));
  ok(smoke.includes("native-workflow-report.json"));
  ok(smoke.includes("validateNativeUiReport"));
  ok(smoke.includes("validateNativeWorkflowReport"));
  ok(smoke.includes("native UI report did not include command button"));
  ok(smoke.includes("native workflow report did not include passing assertion"));
  ok(smoke.includes("native workflow report did not include mode evidence"));
  ok(smoke.includes("native workflow saved Markdown file was not written"));
  ok(smoke.includes("native workflow included watcher file was not written"));
  ok(smoke.includes("native workflow local conflict copy was not written"));
  ok(smoke.includes("native workflow HTML export artifact was not written"));
  ok(smoke.includes("native-menu HTML export evidence"));
  ok(smoke.includes("native workflow report did not include theme/accessibility evidence"));
  ok(smoke.includes("native UI report did not include rendered preview identity or content"));
  ok(smoke.includes("status = \"limited\""));
});
