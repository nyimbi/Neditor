import { deepEqual, equal, ok } from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { buildConflictDiff } from "../src/lib/conflict.js";
import { appendConflictMergeLine, applyAiPasteInsertion, quoteMarkdown } from "../src/lib/workflows.js";
import {
  formatTableTotal,
  parseTableCellSpan,
  parseMarkdownTables,
  parseTablePaste,
  serializeMarkdownTable,
  setTableCellSpan,
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

test("AI paste insertion modes preserve workflow-specific output", () => {
  equal(quoteMarkdown("alpha\n\nbeta"), "> alpha\n>\n> beta");
  equal(applyAiPasteInsertion("# Report", "Cleaned", "insert"), "# Report\n\nCleaned\n");
  equal(applyAiPasteInsertion("# Report", "Cleaned", "replace"), "Cleaned");
  equal(applyAiPasteInsertion("# Report", "Cleaned", "quote"), "# Report\n\n> Cleaned\n");
  equal(applyAiPasteInsertion("# Report", "Cleaned", "appendix"), "# Report\n\n## AI Draft Appendix\n\nCleaned\n");
});

test("local verification scripts expose frontend and browser checks", () => {
  const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
    scripts: Record<string, string>;
  };
  const { scripts } = packageJson;

  equal(scripts.check, "vue-tsc --noEmit");
  equal(scripts.build, "vue-tsc --noEmit && vite build");
  equal(scripts["test:unit"], "tsc -p tsconfig.test.json && node --test .tmp-tests/tests/frontend-unit.test.js");
  equal(scripts["test:e2e"], "playwright test");
});
