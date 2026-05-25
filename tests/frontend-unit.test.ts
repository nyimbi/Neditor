import { deepEqual, equal, ok } from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  beginLatestDocumentTask,
  cancelLatestDocumentTask,
  isLatestDocumentTaskCurrent,
  type LatestDocumentTaskGate,
} from "../src/lib/asyncGuards.js";
import { inspectAiRuntimeReadiness } from "../src/lib/aiRuntimeReadiness.js";
import {
  aiProviderProfiles,
  buildAiProviderRequestPackage,
  buildAiProviderResponseReviewMarkdown,
  executeAiProviderRequestPackage,
  formatAiProviderSourcePack,
} from "../src/lib/aiProviderPackages.js";
import {
  agenticWorkflowPlaybooks,
  buildAgenticLifecycleTaskBrief,
  buildAgenticReleaseEvidenceAuditPackage,
  buildAgenticSectionWorkBrief,
  buildAgenticSourcePack,
  buildAgenticWorkflowPlan,
  buildAgenticWorkflowRun,
  serializeAgenticSourcePackItem,
} from "../src/lib/agenticWorkflows.js";
import {
  bibliographyEntryStub,
  bibliographyStubsForMissingKeys,
  citationReferenceSnippet,
  normalizeCitationKey,
} from "../src/lib/bibliographyManager.js";
import { buildConflictDiff } from "../src/lib/conflict.js";
import {
  citationTodoAuditMarkdown,
  citationTodoComment,
  deferCitationTodo,
  extractCitationTodoItems,
  resolveCitationTodo,
} from "../src/lib/citationTodoWorkflow.js";
import { createDebouncedTextCommit, PREVIEW_DEBOUNCE_MS } from "../src/lib/debounce.js";
import {
  buildDocsLiveDraft,
  buildDocsLiveQuestionnaire,
  docsLivePlaceholderEntries,
  docsLiveDocumentTypes,
  extractDocsLivePlaceholders,
  normalizeDocsLiveDocumentType,
  removeDocsLivePlaceholder,
  upsertDocsLivePlaceholder,
} from "../src/lib/docsLive.js";
import { outlinePlanFromMarkdown, outlinePlanToMarkdown, parseOutlinePlan } from "../src/lib/documentOutline.js";
import { markdownListContinuation } from "../src/lib/markdownEditing.js";
import { extractMarkdownSection, findMarkdownSectionRange, replaceOrAppendMarkdownSection } from "../src/lib/markdownSectionMerge.js";
import {
  builtinTransformTemplates,
  normalizeCustomTransformTemplates,
  transformTemplateFillFields,
  transformTemplateMarkdown,
} from "../src/lib/transformTemplates.js";
import {
  migratePersistedWorkspace,
  normalizeAgentRunHistory,
  normalizeCitationStyle,
  normalizeDocsLiveDraftHistory,
  WORKSPACE_SCHEMA_VERSION,
} from "../src/lib/workspacePersistence.js";
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

test("citation TODO workflow extracts resolves defers and audits blockers", () => {
  const source = [
    "# Draft",
    "Revenue grew by 18%. citation TODO",
    "<!-- citation-todo: deferred | reason: Waiting on finance; needs citation -->",
    "Margin improved. needs citation",
  ].join("\n");
  const todos = extractCitationTodoItems(source);

  equal(todos.length, 3);
  equal(todos[0].status, "open");
  equal(todos[1].status, "deferred");
  equal(todos[1].note, "Waiting on finance; needs citation");

  const resolved = resolveCitationTodo(source, todos[0], "[@finance2026]", "Audited forecast");
  ok(resolved.includes("Revenue grew by 18%. [@finance2026] <!-- citation-resolved: Audited forecast -->"));
  ok(!extractCitationTodoItems(resolved).some((item) => item.line === 2));

  const deferred = deferCitationTodo(source, todos[2], "Need source owner");
  ok(deferred.includes("<!-- citation-todo: deferred | reason: Need source owner | original: needs citation -->"));
  ok(citationTodoComment("Board pack --> source").includes("Board pack source"));
  ok(citationTodoAuditMarkdown(todos).includes("Line 2 (open): Revenue grew by 18%. [citation TODO]"));
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

test("editable outline planner creates document skeletons before drafting content", () => {
  const plan = "- Executive Summary\n  - Decision Needed\n  - Key Risks\n2. Financial Case\n  - Launch Plan";
  deepEqual(parseOutlinePlan(plan), [
    { level: 1, title: "Executive Summary" },
    { level: 2, title: "Decision Needed" },
    { level: 2, title: "Key Risks" },
    { level: 1, title: "Financial Case" },
    { level: 2, title: "Launch Plan" },
  ]);

  const markdown = outlinePlanToMarkdown(plan, { title: "Board Brief", includeToc: true });
  ok(markdown.includes("title: Board Brief"));
  ok(markdown.includes("toc: true"));
  ok(markdown.includes("# Board Brief"));
  ok(markdown.includes("[TOC]"));
  ok(markdown.includes("## Executive Summary"));
  ok(markdown.includes("### Decision Needed"));
  ok(markdown.includes("### Key Risks"));
  ok(markdown.includes("## Financial Case"));
  ok(markdown.includes("### Launch Plan"));
  ok(markdown.includes("<!-- Draft this section. -->"));

  equal(outlinePlanFromMarkdown(markdown), "- Board Brief\n  - Executive Summary\n    - Decision Needed\n    - Key Risks\n  - Financial Case\n    - Launch Plan");
});

test("Docs Live turns outline, voice context, and placeholders into a reviewable draft", () => {
  deepEqual(extractDocsLivePlaceholders("client: Acme\nAudience is executive team.\ndeadline: June 1"), {
    client: "Acme",
    audience: "executive team",
    deadline: "June 1",
  });
  deepEqual(docsLivePlaceholderEntries("Client Name: Acme\nowner = Finance\nclient_name: duplicate ignored"), [
    { key: "client name", value: "Acme", kind: "text", source: "", reviewStatus: "provided" },
    { key: "owner", value: "Finance", kind: "text", source: "", reviewStatus: "provided" },
  ]);
  deepEqual(docsLivePlaceholderEntries("budget: $250K | type=money | source=Finance workbook | status=verified"), [
    { key: "budget", value: "$250K", kind: "money", source: "Finance workbook", reviewStatus: "verified" },
  ]);
  equal(
    upsertDocsLivePlaceholder("client: Acme", "Budget", "$250K", {
      kind: "money",
      source: "Finance workbook",
      reviewStatus: "needs-review",
    }),
    "client: Acme\nbudget: $250K | type=money | source=Finance workbook | status=needs-review",
  );
  equal(upsertDocsLivePlaceholder("client: Acme", "Deadline", "June 1"), "client: Acme\ndeadline: June 1");
  equal(upsertDocsLivePlaceholder("client: Acme", "client", "Globex"), "client: Globex");
  equal(removeDocsLivePlaceholder("client: Acme\nowner: Finance", "client"), "owner: Finance");

  const questionnaire = buildDocsLiveQuestionnaire("proposal", {
    title: "Acme Renewal Proposal",
    outline: "- Executive Summary\n- Proposed Approach\n- Investment",
    placeholders: "client: Acme\nowner: Commercial team",
  });
  ok(questionnaire.includes("Who is the client or sponsor?"));
  ok(questionnaire.includes('What should "Acme Renewal Proposal" help the reader decide'));
  ok(questionnaire.includes('For "Executive Summary", what facts'));
  ok(questionnaire.includes("Outcome"));
  ok(questionnaire.includes("Distribution Target"));

  const draft = buildDocsLiveDraft({
    documentType: "proposal",
    title: "Acme Renewal Proposal",
    outline: "- Executive Summary\n- Proposed Approach\n- Investment",
    transcript: "Create a client proposal for Acme. The audience is the executive team. Focus on a fast first draft.",
    context: "The goal is to renew the platform contract. Include a clear recommendation and review notes.",
    questionnaireAnswers: "The reader should approve renewal. Keep pricing assumptions marked for human review.",
    placeholders: "client: Acme | type=client | source=CRM | status=verified\nowner: Commercial team\ndeadline: June 1",
    draftingDepth: "detailed",
    generatedAt: "2026-05-23T09:00:00.000Z",
  });

  equal(draft.documentType, "proposal");
  equal(draft.title, "Acme Renewal Proposal");
  equal(draft.sections.length, 3);
  equal(draft.workflow[2].id, "draft");
  equal(draft.workflow[4].id, "humanize");
  equal(draft.workflow[5].id, "review");
  ok(draft.reviewPacket.contextSources.some((source) => source.includes("AI-created questionnaire answers captured")));
  ok(draft.reviewPacket.sectionRunbook[0].includes("draft body, run QA"));
  ok(draft.reviewPacket.qaRegister.some((item) => item.includes("Executive Summary must tie")));
  ok(draft.reviewPacket.humanizationChecklist.some((item) => item.includes("Remove AI cruft")));
  ok(draft.reviewPacket.humanizationChecklist.some((item) => item.includes("prompt-shaped phrasing")));
  ok(draft.reviewPacket.reviewerHandoff.some((item) => item.includes("AI-assisted markers")));
  equal(draft.sections[0].qaChecks.length, 3);
  equal(draft.sections[0].humanizationNotes.length, 3);
  deepEqual(
    draft.sections[0].stagePlan.map((stage) => stage.id),
    ["draft", "qa", "humanize", "review"],
  );
  ok(draft.sections[0].qaSummary.includes("Executive Summary must tie"));
  ok(draft.sections[0].humanizedAngle.includes("responsible subject-matter owner"));
  ok(draft.sections[0].reviewHandoff.includes("Commercial team should verify"));
  equal(draft.placeholders.client, "Acme");
  ok(draft.markdown.includes("| Client | Acme | Client | CRM | verified |"));
  ok(draft.markdown.includes("provider: NEditor Docs Live"));
  ok(draft.markdown.includes("model: local-guided-drafting"));
  ok(draft.markdown.includes("workflow: outline-to-section-draft-qa-humanize-review"));
  ok(draft.markdown.includes("<!-- ai-assisted: status=needs-review"));
  ok(draft.markdown.includes("## Draft Context"));
  ok(draft.markdown.includes("## Drafting Plan"));
  ok(draft.markdown.includes("## Section-by-section Draft Runbook"));
  ok(draft.markdown.includes("### Context Package"));
  ok(draft.markdown.includes("### Section Work Queue"));
  ok(draft.markdown.includes("### Assumption Register"));
  ok(draft.markdown.includes("### Humanization Checklist"));
  ok(draft.markdown.includes("### Review Packet"));
  ok(draft.markdown.includes("### Section QA"));
  ok(draft.markdown.includes("### Review Handoff"));
  ok(draft.markdown.includes("## Review Handoff"));
  ok(draft.markdown.includes("## Review Preparation"));
  ok(draft.markdown.includes("### Quality Assurance"));
  ok(draft.markdown.includes("### Humanization Pass"));
  ok(draft.markdown.includes("responsible subject-matter owner"));
  ok(draft.markdown.includes("Commercial team should verify"));
  ok(draft.markdown.includes("The reader should approve renewal"));
  ok(draft.markdown.includes("Commercial team"));

  const technicalDraft = buildDocsLiveDraft({
    documentType: "technical-architecture",
    title: "Integration Plan",
    outline: "- API Contract",
    context: "audience: engineering review board. evidence: architecture notes. owner: Platform.",
    placeholders: "audience: engineering review board\nowner: Platform\nevidence: architecture notes",
    draftingDepth: "technical",
    generatedAt: "2026-05-23T09:00:00.000Z",
  });
  ok(technicalDraft.markdown.includes("technical depth"));
  ok(technicalDraft.markdown.includes("verify technical terms"));
});

test("Docs Live covers business technical legal marketing and customer document blueprints", () => {
  for (const id of [
    "business-case",
    "operating-procedure",
    "technical-architecture",
    "adr",
    "release-notes",
    "contract-brief",
    "marketing-brief",
    "customer-case-study",
  ]) {
    ok(docsLiveDocumentTypes.some((type) => type.id === id), `missing ${id}`);
  }

  equal(normalizeDocsLiveDocumentType("Draft a standard operating procedure for month end close"), "operating-procedure");
  equal(normalizeDocsLiveDocumentType("Create release notes with known issues and upgrade notes"), "release-notes");
  equal(normalizeDocsLiveDocumentType("Write a customer case study with verified results"), "customer-case-study");

  const draft = buildDocsLiveDraft({
    documentType: "contract brief",
    title: "Vendor Renewal Contract Brief",
    context: "audience is legal reviewers. owner is Procurement. evidence is signed term sheet.",
    placeholders: "approver: General Counsel",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });

  equal(draft.documentType, "contract-brief");
  ok(draft.outlineText.includes("Commercial Terms"));
  ok(draft.questionnaire.includes("commercial, legal, operational, or data terms"));
  ok(draft.markdown.includes("Contract brief"));
  ok(draft.reviewPacket.sectionRunbook.some((item) => item.includes("Commercial Terms")));
});

test("Docs Live section drafts can replace matching Markdown sections", () => {
  const source = [
    "# Capital Allocation Memo",
    "",
    "<!-- ai-assisted: status=needs-review | source=old | promptSummary=old section -->",
    "## Current Ask",
    "",
    "Old draft text.",
    "",
    "## Risks",
    "",
    "Keep this risk section.",
    "",
  ].join("\n");
  const draft = buildDocsLiveDraft({
    documentType: "board-memo",
    title: "Capital Allocation Memo - Current Ask",
    outline: "- Current Ask",
    context: "audience: board. evidence: audited forecast. owner: Finance.",
    placeholders: "audience: board\nowner: Finance\ndeadline: June 1\nevidence: audited forecast",
    draftingDepth: "detailed",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });

  const extracted = extractMarkdownSection(draft.markdown, "Current Ask", 2);
  ok(extracted.includes("source=NEditor Docs Live"));
  ok(extracted.includes("## Current Ask"));
  ok(extracted.includes("Section QA"));
  const existing = findMarkdownSectionRange(source, "Current Ask");
  equal(existing?.level, 2);
  const merged = replaceOrAppendMarkdownSection(source, draft.markdown, "Current Ask", 2);

  ok(!merged.includes("Old draft text."));
  ok(!merged.includes("promptSummary=old section"));
  ok(merged.includes("audited forecast"));
  ok(merged.includes("## Risks\n\nKeep this risk section."));

  const appended = replaceOrAppendMarkdownSection("# Capital Allocation Memo\n\nNo matching section yet.\n", draft.markdown, "Current Ask", 2);
  ok(appended.includes("No matching section yet."));
  ok(appended.includes("## Current Ask"));
});

test("AI runtime readiness reports voice and clipboard capability without storing clipboard content", async () => {
  const report = await inspectAiRuntimeReadiness({
    now: () => "2026-05-25T10:00:00.000Z",
    secureContext: true,
    hasSpeechRecognition: true,
    queryPermission: async (name) => (name === "microphone" ? "granted" : "prompt"),
    readClipboard: async () => ({ kind: "rich", length: 123 }),
    canWriteClipboard: true,
  });

  equal(report.generatedAt, "2026-05-25T10:00:00.000Z");
  equal(report.secureContext, true);
  equal(report.speechRecognition.supported, true);
  equal(report.microphonePermission.state, "granted");
  equal(report.clipboardRead.supported, true);
  equal(report.clipboardRead.state, "granted");
  equal(report.clipboardWrite.supported, true);
  deepEqual(report.issues, []);
  ok(report.markdown.includes("| Speech recognition | yes | available |"));
  ok(report.markdown.includes("Clipboard rich read succeeded (123 characters detected, content not stored)."));
  ok(report.markdown.includes("- No blocking runtime issues detected."));
  ok(!report.markdown.includes("Runtime clipboard proof"));
});

test("AI runtime readiness flags missing secure voice and clipboard capabilities", async () => {
  const report = await inspectAiRuntimeReadiness({
    now: () => "2026-05-25T10:00:00.000Z",
    secureContext: false,
    hasSpeechRecognition: false,
    queryPermission: async (name) => (name === "microphone" ? "denied" : "denied"),
    readClipboard: async () => null,
    canWriteClipboard: false,
  });

  equal(report.secureContext, false);
  equal(report.speechRecognition.supported, false);
  equal(report.microphonePermission.state, "denied");
  equal(report.clipboardRead.supported, false);
  equal(report.clipboardWrite.supported, false);
  ok(report.issues.some((issue) => issue.includes("secure runtime context")));
  ok(report.issues.some((issue) => issue.includes("SpeechRecognition API is unavailable")));
  ok(report.issues.some((issue) => issue.includes("microphone permission is denied")));
  ok(report.issues.some((issue) => issue.includes("clipboard-read permission is denied")));
  ok(report.issues.some((issue) => issue.includes("Clipboard write API is unavailable")));
  ok(report.markdown.includes("| Clipboard read | no | denied |"));
});

test("agentic workflow planner coordinates creation revision review and distribution", () => {
  const plan = buildAgenticWorkflowPlan({
    instruction:
      "Create a board memo for the executive team, revise it for the CFO, check citations and risks, then publish as PDF and Google Docs. audience: executive team owner: Strategy deadline: June 1",
    documentTitle: "Expansion Options",
    documentText: "# Expansion Options\n\n## Current State\n\nDraft notes.",
    selectedText: "This section is too generic.",
  });

  equal(plan.documentType, "board-memo");
  equal(plan.title, "Expansion Options");
  deepEqual(plan.distributionTargets, ["pdf", "google-docs"]);
  ok(plan.lanes.includes("create"));
  ok(plan.lanes.includes("revise"));
  ok(plan.lanes.includes("review"));
  ok(plan.lanes.includes("distribute"));
  ok(plan.context.includes("Agent lanes requested"));
  ok(plan.placeholderText.includes("audience: executive team"));
  ok(plan.placeholderText.includes("distribution: pdf, google-docs"));
  equal(plan.contextCompleteness.status, "usable");
  ok(plan.contextCompleteness.present.includes("audience"));
  ok(plan.contextCompleteness.missing.includes("examples"));
  ok(plan.suggestedOutline.includes("Board memo"));
  ok(plan.revisionInstruction.includes("revise it for the CFO"));
  ok(plan.revisionModes.includes("executive-summary"));
  ok(plan.revisionModes.includes("evidence"));
  ok(plan.revisionModes.includes("legal-caution"));
  ok(plan.qualityGates.some((gate) => gate.label === "Board Decision"));
  ok(plan.qualityGates.some((gate) => gate.label === "Financial Case"));
  ok(plan.qualityGates.some((gate) => gate.label === "Distribution Readiness"));
  ok(plan.missingInputs.includes("evidence"));
  ok(plan.steps.some((step) => step.action === "open-docs-live"));
  ok(plan.steps.some((step) => step.action === "open-ai-paste"));
  ok(plan.steps.some((step) => step.action === "open-review"));
  ok(plan.steps.some((step) => step.action === "prepare-export"));
});

test("agentic workflow planner uses context answers to close missing inputs", () => {
  const plan = buildAgenticWorkflowPlan({
    instruction: "Create a board memo, review risks, and prepare PDF distribution.",
    contextAnswers:
      "audience: board\nowner: Finance\ndeadline: June 1\ntone: direct\nevidence: audited forecast\nreviewer: CFO\nstatus: approved for review",
    documentTitle: "Capital Allocation Memo",
    documentText: "# Capital Allocation Memo\n\n## Current Ask\n\nDraft notes.",
  });

  equal(plan.contextAnswers.includes("audience: board"), true);
  ok(plan.context.includes("Agent context answers:"));
  ok(plan.placeholderText.includes("audience: board"));
  ok(plan.placeholderText.includes("owner: Finance"));
  ok(plan.placeholderText.includes("evidence: audited forecast"));
  equal(plan.contextCompleteness.status, "strong");
  ok(plan.contextCompleteness.score >= 80);
  ok(plan.distributionTargets.includes("pdf"));
  ok(!plan.missingInputs.includes("evidence"));
  ok(!plan.missingInputs.includes("approval status for distribution"));

  const run = buildAgenticWorkflowRun({
    instruction: plan.instruction,
    contextAnswers: plan.contextAnswers,
    documentTitle: "Capital Allocation Memo",
    documentText: "# Capital Allocation Memo\n\n## Current Ask\n\nDraft notes.",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });

  ok(run.markdown.includes("Agent context answers:"));
  ok(run.markdown.includes("### Context Completeness"));
  ok(run.auditTrail.contextFingerprint.length === 16);
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Context completeness" && item.status === "available"));
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Evidence" && item.status === "available"));
});

test("agentic source pack builder structures notes urls files claims and reviewer comments", () => {
  const sourcePack = buildAgenticSourcePack(
    [
      "[claim] ARR forecast: ARR grows 18% in Q2 according to finance workbook",
      "[url] Pricing page: https://example.com/pricing",
      "[file] Finance workbook: /workspace/finance.xlsx",
      "[reviewer-comment] CFO: Confirm renewal risk before board review",
      "Source: Gartner report on market demand",
      serializeAgenticSourcePackItem("note", "Workshop", "Customer success wants a plain-language rollout note."),
    ].join("\n"),
  );

  equal(sourcePack.items.length, 6);
  equal(sourcePack.claims.length, 1);
  equal(sourcePack.urls.length, 1);
  equal(sourcePack.files.length, 1);
  equal(sourcePack.reviewerComments.length, 1);
  equal(sourcePack.references.length, 1);
  ok(sourcePack.markdown.includes("[claim] ARR forecast"));

  const run = buildAgenticWorkflowRun({
    instruction: "Create a board memo, review evidence, and prepare PDF. audience: board owner: CFO deadline: June 1",
    sourcePackText: sourcePack.markdown,
    documentTitle: "Board Memo",
    documentText: "# Board Memo\n\nARR grows by 18%.",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });
  const providerPackage = buildAiProviderRequestPackage(run, { profileId: "manual-review" });

  ok(run.plan.context.includes("User source pack: 6 item"));
  ok(run.plan.sourcePack.claims.some((item) => item.detail.includes("ARR grows 18%")));
  ok(run.markdown.includes("### User Source Pack"));
  ok(run.lifecycleTasks.some((task) => task.id === "task-source-pack-review"));
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "User source pack" && item.status === "available"));
  ok(providerPackage.sourcePack.userSources.some((item) => item.includes("Pricing page")));
  ok(providerPackage.sourcePack.claimReview.some((item) => item.includes("User source claim")));
  ok(formatAiProviderSourcePack(providerPackage.sourcePack).includes("User-managed source pack:"));
  ok(providerPackage.userPrompt.includes("User-managed source pack:"));
});

test("agentic workflow playbooks cover common business and publishing starts", () => {
  ok(agenticWorkflowPlaybooks.length >= 10);
  ok(agenticWorkflowPlaybooks.some((playbook) => playbook.id === "board-memo-to-approval"));
  ok(agenticWorkflowPlaybooks.some((playbook) => playbook.id === "strategy-memo-from-research"));
  ok(agenticWorkflowPlaybooks.some((playbook) => playbook.id === "policy-to-approval"));
  ok(agenticWorkflowPlaybooks.some((playbook) => playbook.id === "release-notes-to-publish"));
  ok(agenticWorkflowPlaybooks.some((playbook) => playbook.id === "grant-application-review"));
  ok(agenticWorkflowPlaybooks.some((playbook) => playbook.instruction.includes("Substack")));
  ok(agenticWorkflowPlaybooks.every((playbook) => playbook.bestFor.length >= 3));
  ok(agenticWorkflowPlaybooks.every((playbook) => playbook.expectedOutputs.length >= 4));

  const publishingPlan = buildAgenticWorkflowPlan({
    instruction: agenticWorkflowPlaybooks.find((playbook) => playbook.id === "publish-to-blog-and-substack")?.instruction || "",
    documentTitle: "Market Note",
    documentText: "# Market Note\n\nDraft.",
  });

  ok(publishingPlan.lanes.includes("revise"));
  ok(publishingPlan.lanes.includes("review"));
  ok(publishingPlan.lanes.includes("distribute"));
  ok(publishingPlan.distributionTargets.includes("blog"));
  ok(publishingPlan.distributionTargets.includes("substack"));
  ok(publishingPlan.distributionTargets.includes("html"));
});

test("agentic workflow run generates auditable creation and distribution packets", () => {
  const run = buildAgenticWorkflowRun({
    instruction:
      "Create a proposal for Acme, compose it section by section, review evidence, and publish to Substack plus Google Docs. audience: executive committee owner: Growth deadline: June 1 evidence: CRM forecast",
    documentTitle: "Acme Growth Proposal",
    documentText: "",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });

  equal(run.applicationMode, "replace-document");
  equal(run.revision, null);
  ok(run.summary.includes("Create"));
  ok(run.plan.distributionTargets.includes("substack"));
  ok(run.plan.distributionTargets.includes("google-docs"));
  equal(run.distributionTargetPlans.length, 2);
  ok(run.distributionTargetPlans.some((plan) => plan.label === "Substack newsletter package"));
  ok(run.distributionTargetPlans.some((plan) => plan.evidenceRequired.some((item) => item.includes("Google Drive import/readback"))));
  ok(run.markdown.includes("provider: NEditor Agent Workspace"));
  ok(run.markdown.includes("model: local-agentic-workflow"));
  ok(run.markdown.includes("## Generated Draft"));
  ok(run.markdown.includes("provider: NEditor Docs Live"));
  ok(run.markdown.includes("## Quality Assurance"));
  ok(run.markdown.includes("### Document-Type Quality Gates"));
  ok(run.markdown.includes("Client Need"));
  ok(run.markdown.includes("## Review Comment Resolution Queue"));
  ok(run.lifecycleTasks.some((task) => task.id === "task-quality-gates"));
  ok(run.markdown.includes("## Distribution"));
  ok(run.markdown.includes("### Target Runbooks"));
  ok(run.markdown.includes("## AI Control Center"));
  ok(run.markdown.includes("### Source Grounding"));
  ok(run.markdown.includes("## Outline Critique"));
  ok(run.markdown.includes("## Claim Inventory"));
  ok(run.markdown.includes("## Humanization Findings"));
  ok(run.markdown.includes("## Review Agents"));
  ok(run.markdown.includes("### Editorial Reviewer"));
  ok(run.markdown.includes("### Export Reviewer"));
  ok(run.markdown.includes("## Agent Lifecycle Task Board"));
  ok(run.markdown.includes("Final human approval and release readiness"));
  ok(run.markdown.includes("## Section Work Queue"));
  ok(run.markdown.includes("Completion criteria:"));
  ok(run.markdown.includes("## Agent Audit Trail"));
  ok(run.markdown.includes("### Rollback Plan"));
  ok(run.markdown.includes("## Release Evidence Bundle"));
  ok(run.releaseEvidenceBundle.items.some((item) => item.label === "Agent audit trail" && item.status === "available"));
  ok(run.releaseEvidenceBundle.items.some((item) => item.label === "Distribution artifacts" && item.status === "needs-review"));
  ok(run.releaseEvidenceBundle.items.some((item) => item.label === "Substack newsletter package evidence" && item.requiredBeforeRelease));
  ok(
    run.releaseEvidenceBundle.items.some(
      (item) =>
        item.label === "Google Docs collaboration package evidence" &&
        item.detail.includes("Google Drive import/readback evidence"),
    ),
  );
  const releaseAuditPackage = buildAgenticReleaseEvidenceAuditPackage(run);
  ok(releaseAuditPackage.includes("## NEditor Release Evidence Audit Package"));
  ok(releaseAuditPackage.includes("## Release Evidence Bundle"));
  ok(releaseAuditPackage.includes("## Agent Audit Trail"));
  ok(releaseAuditPackage.includes("## Agent Lifecycle Task Board"));
  ok(releaseAuditPackage.includes("### Target Runbooks"));
  ok(run.markdown.includes("Substack newsletter package"));
  ok(run.markdown.includes("Google Docs collaboration package"));
  ok(run.markdown.includes("Substack newsletter package evidence"));
  ok(run.markdown.includes("Google Docs collaboration package evidence"));
  ok(run.auditTrail.runId.startsWith("agent-20260524T10000"));
  equal(run.auditTrail.applicationMode, "replace-document");
  equal(run.auditTrail.instructionFingerprint.length, 16);
  equal(run.auditTrail.outputFingerprint.length, 16);
  ok(run.auditTrail.rollbackPlan.some((item) => item.includes("snapshot")));
  ok(run.auditTrail.reviewEvents.some((item) => item.includes("Distribution evidence requirements")));
  ok(run.controlCenter.readinessScore > 0);
  ok(run.controlCenter.status === "needs-input" || run.controlCenter.status === "ready");
  ok(run.controlCenter.nextActions.some((action) => action.label === "Verify target artifacts"));
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Evidence" && item.status === "available"));
  ok(run.controlCenter.governance.some((item) => item.label === "AI provenance" && item.status === "available"));
  ok(run.controlCenter.distribution.some((item) => item.label === "Substack newsletter package"));
  ok(run.lifecycleTasks.length >= run.sectionWorkQueue.length + run.reviewerAgents.length);
  ok(run.lifecycleTasks.some((task) => task.title.includes("Resolve intent") && task.owner === "Planner Agent"));
  ok(run.lifecycleTasks.some((task) => task.owner === "Docs Live Section Agent" && task.action === "generate-docs-live-draft" && task.sectionId));
  ok(run.lifecycleTasks.some((task) => task.owner === "Distribution Agent" && task.lane === "distribute" && task.target === "substack"));
  equal(run.reviewerAgents.length, 6);
  ok(run.reviewerAgents.some((agent) => agent.id === "editor" && agent.findings.some((item) => item.includes("Outline"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "evidence" && agent.requiredActions.some((item) => item.includes("Verify every material claim"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "export" && agent.requiredActions.some((item) => item.includes("Google Docs collaboration package"))));
  ok(run.auditTrail.reviewEvents.some((item) => item.includes("Reviewer agents prepared")));
  ok(run.auditTrail.reviewEvents.some((item) => item.includes("Lifecycle task board prepared")));
  ok(run.auditTrail.reviewEvents.some((item) => item.includes("Outline critique prepared")));
  ok(run.auditTrail.reviewEvents.some((item) => item.includes("Humanization scan")));
  ok(run.sectionWorkQueue.length >= 5);
  ok(run.sectionWorkQueue.every((section) => section.completionCriteria.length >= 4));
  ok(run.sectionWorkQueue.some((section) => section.reviewerAgentIds.includes("export")));
  ok(run.auditTrail.reviewEvents.some((item) => item.includes("Section work queue prepared")));
  ok(run.sectionWorkQueue.every((section) => section.draftingDepth));
  ok(run.sectionWorkQueue.some((section) => section.draftingDepth === "detailed"));
  ok(run.markdown.includes("Drafting depth:"));
  const sectionBrief = buildAgenticSectionWorkBrief(run.sectionWorkQueue[0], run.reviewerAgents);
  ok(sectionBrief.includes("```ai-section-task"));
  ok(sectionBrief.includes("draftingDepth:"));
  ok(sectionBrief.includes("### Drafting Instruction"));
  ok(sectionBrief.includes("### Completion Criteria"));
  ok(sectionBrief.includes("### Assigned Reviewers"));
  const taskBrief = buildAgenticLifecycleTaskBrief(run.lifecycleTasks.find((task) => task.sectionId) || run.lifecycleTasks[0]);
  ok(taskBrief.includes("```ai-lifecycle-task"));
  ok(taskBrief.includes("### Evidence Checklist"));
  ok(taskBrief.includes("### Handoff Notes"));
  ok(run.distributionChecklist.some((item) => item.startsWith("Substack newsletter package:")));
  ok(run.reviewChecklist.some((item) => item.includes("human-reviewed")));
});

test("agentic workflow run proposes selection-aware revisions with review metadata", () => {
  const run = buildAgenticWorkflowRun({
    instruction:
      "Revise this for the CFO, make it concise, humanize the tone, then check risks. audience: finance committee owner: Strategy deadline: June 1 tone: concise evidence: signed forecast reviewer: CFO",
    documentTitle: "Expansion Options",
    documentText: "# Expansion Options\n\nDraft body.",
    selectedText:
      "It is important to note that leveraging various growth opportunities can be robust. The plan increases ARR by 24%. The CFO must approve by June 1.",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });

  equal(run.applicationMode, "replace-selection");
  ok(run.revision);
  ok(run.revision?.proposedText.includes("source=NEditor Agent Workspace"));
  ok(run.revision?.proposedText.includes("Finance review focus"));
  ok(run.revision?.revisionPasses.some((pass) => pass.mode === "brevity"));
  ok(run.revision?.revisionPasses.some((pass) => pass.mode === "humanization"));
  ok(run.revision?.revisionPasses.some((pass) => pass.mode === "legal-caution"));
  equal(run.editAcceptanceQueue.length, 1);
  equal(run.editAcceptanceQueue[0].scope, "selection");
  ok(run.editAcceptanceQueue[0].riskNotes.some((note) => note.includes("June 1")));
  ok(!run.revision?.proposedText.includes("It is important to note"));
  ok(!run.revision?.proposedText.includes("leveraging"));
  ok(run.revision?.meaningDriftFindings.some((finding) => finding.kind === "date" && finding.original.includes("June 1")));
  ok(run.revision?.meaningDriftFindings.some((finding) => finding.kind === "commitment" && finding.original.includes("must approve")));
  ok(run.reviewChecklist.some((item) => item.includes("Compare the revision proposal")));
  ok(run.reviewChecklist.some((item) => item.includes("Complete revision passes")));
  ok(run.reviewChecklist.some((item) => item.includes("edit acceptance queue")));
  ok(run.reviewChecklist.some((item) => item.includes("Resolve all meaning-drift findings")));
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Selected text" && item.status === "available"));
  ok(run.controlCenter.governance.some((item) => item.label === "Revision audit" && item.status === "needs-review"));
  ok(run.controlCenter.nextActions.some((action) => action.lane === "revise"));
  ok(run.lifecycleTasks.some((task) => task.id === "task-revision-proposal" && task.status === "blocked"));
  ok(run.lifecycleTasks.some((task) => task.id === "task-revision-proposal" && task.evidence.some((item) => item.includes("Brevity pass"))));
  ok(run.lifecycleTasks.some((task) => task.id === "task-edit-acceptance-queue"));
  ok(run.reviewerAgents.some((agent) => agent.id === "editor" && agent.requiredActions.some((item) => item.includes("Compare the proposed revision"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "editor" && agent.requiredActions.some((item) => item.includes("edit acceptance queue"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "editor" && agent.requiredActions.some((item) => item.includes("revision pass checklist"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "risk" && agent.requiredActions.some((item) => item.includes("meaning-drift"))));
  equal(run.auditTrail.applicationMode, "replace-selection");
  ok(run.auditTrail.rollbackPlan.some((item) => item.includes("editor undo")));
  ok(run.markdown.includes("Apply mode: replace-selection"));
  ok(run.markdown.includes("## Revision Proposal"));
  ok(run.markdown.includes("## Edit Acceptance Queue"));
  ok(run.markdown.includes("### Planned Revision Modes"));
  ok(run.markdown.includes("### Revision Passes"));
  ok(run.markdown.includes("### Meaning Drift"));
  ok(run.markdown.includes("### Original Text"));
  ok(run.markdown.includes("### Proposed Text"));
});

test("agentic workflow creates section-level edit acceptance queues", () => {
  const run = buildAgenticWorkflowRun({
    instruction:
      "Revise this report section by section, make it concise, check risks, and preserve evidence. audience: leadership owner: Strategy deadline: June 1 evidence: forecast",
    documentTitle: "Operating Review",
    documentText: [
      "# Operating Review",
      "",
      "## Summary",
      "",
      "It is important to note that the plan increases ARR by 18%.",
      "",
      "## Risks",
      "",
      "The team must approve the migration by June 1 unless the vendor misses security review.",
    ].join("\n"),
    generatedAt: "2026-05-24T10:00:00.000Z",
  });

  ok(run.editAcceptanceQueue.length >= 2);
  ok(run.editAcceptanceQueue.some((item) => item.scope === "section" && item.heading === "Summary"));
  ok(run.editAcceptanceQueue.some((item) => item.scope === "section" && item.heading === "Risks"));
  ok(run.editAcceptanceQueue.every((item) => item.proposedText.includes("source=NEditor Agent Workspace")));
  ok(run.markdown.includes("## Edit Acceptance Queue"));
  ok(run.auditTrail.reviewEvents.some((event) => event.includes("Edit acceptance queue prepared")));
});

test("agentic workflow reviewers inspect current document evidence", () => {
  const run = buildAgenticWorkflowRun({
    instruction:
      "Review this board memo for evidence, governance, links, and PDF distribution. audience: board owner: Finance deadline: June 1 evidence: audited forecast status: ready",
    documentTitle: "Board Memo",
    documentText: [
      "---",
      "title: Board Memo",
      "status: draft",
      "---",
      "",
      "# Board Memo",
      "",
      "ARR grows by 18%. citation TODO",
      "Furthermore, this comprehensive analysis clearly unlocks the potential for growth.",
      "",
      "{{client_name}} must approve [OWNER].",
      "",
      "[placeholder link](https://example.com/review)",
      "See {@missing-ref} before export.",
      "",
      "```ai-source",
      "provider: OpenAI",
      "model: ChatGPT",
      "status: needs-review",
      "```",
      "",
      "<!-- ai-assisted: status=needs-review | source=NEditor Docs Live | promptSummary=Draft -->",
      "<!-- comment: unresolved | author: Reviewer | at: 2026-05-24 | Confirm finance source. -->",
    ].join("\n"),
    generatedAt: "2026-05-24T10:00:00.000Z",
  });

  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Document placeholders" && item.status === "needs-review"));
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Outline" && item.detail.includes("critique")));
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Evidence" && item.detail.includes("citation TODO")));
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Claim inventory" && item.status === "needs-review"));
  ok(run.controlCenter.sourceGrounding.some((item) => item.label === "Reference integrity" && item.status === "needs-review"));
  ok(run.controlCenter.nextActions.some((action) => action.label === "Resolve document placeholders" && action.action === "open-ai-paste"));
  ok(run.controlCenter.nextActions.some((action) => action.label === "Review evidence and governance blockers" && action.detail.includes("citation TODO")));
  ok(run.controlCenter.nextActions.some((action) => action.label === "Resolve review comments" && action.status === "blocked"));
  ok(run.controlCenter.nextActions.some((action) => action.label === "Verify claim inventory" && action.detail.includes("candidate claim")));
  ok(run.controlCenter.nextActions.some((action) => action.label === "Humanize current document" && action.action === "open-ai-paste"));
  ok(run.controlCenter.nextActions.some((action) => action.label === "Repair reference integrity" && action.action === "open-review"));
  ok(run.controlCenter.nextActions.some((action) => action.label === "Repair distribution blockers" && action.action === "prepare-export"));
  ok(run.controlCenter.governance.some((item) => item.label === "AI provenance" && item.status === "needs-review"));
  ok(run.controlCenter.governance.some((item) => item.label === "Humanization" && item.status === "needs-review"));
  ok(run.controlCenter.governance.some((item) => item.label === "Human review" && item.detail.includes("unresolved current-document review comment")));
  ok(run.controlCenter.governance.some((item) => item.label === "Approval metadata" && item.detail.includes("approvedAt")));
  ok(run.controlCenter.governance.some((item) => item.label === "Approval metadata" && item.detail.includes("owner")));
  ok(run.controlCenter.governance.some((item) => item.label === "Approval metadata" && item.detail.includes("releaseTarget")));
  ok(run.controlCenter.distribution.some((item) => item.detail.includes("placeholder or suspicious link")));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-placeholders" && task.evidence.some((item) => item.includes("{{client_name}}"))));
  ok(run.lifecycleTasks.some((task) => task.id === "task-outline-critique" && task.action === "open-outline"));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-citations" && task.owner === "Evidence Agent"));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-claim-inventory" && task.evidence.some((item) => item.includes("ARR grows by 18%"))));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-humanization" && task.evidence.some((item) => item.includes("comprehensive analysis"))));
  ok(run.documentEvidence.reviewCommentResolutions.some((comment) => comment.excerpt.includes("Confirm finance source") && comment.requiredAction.includes("source evidence")));
  ok(run.lifecycleTasks.some((task) => task.id.startsWith("task-review-comment-") && task.nextStep.includes("source evidence")));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-ai-review" && task.owner === "Governance Agent"));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-links" && task.action === "prepare-export"));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-references" && task.evidence.some((item) => item.includes("missing-ref"))));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-approval-metadata" && task.evidence.some((item) => item.includes("approvedAt"))));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-approval-metadata" && task.nextStep.includes("owner")));
  ok(run.lifecycleTasks.some((task) => task.id === "task-evidence-approval-metadata" && task.evidence.some((item) => item.includes("releaseTarget"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "editor" && agent.findings.some((item) => item.includes("{{client_name}}"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "editor" && agent.requiredActions.some((item) => item.includes("outline critique"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "editor" && agent.requiredActions.some((item) => item.includes("humanization findings"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "evidence" && agent.findings.some((item) => item.includes("citation TODO"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "evidence" && agent.requiredActions.some((item) => item.includes("claim inventory"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "citation" && agent.findings.some((item) => item.includes("reference integrity"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "risk" && agent.requiredActions.some((item) => item.includes("review comment resolution queue"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "governance" && agent.requiredActions.some((item) => item.includes("human-reviewed"))));
  ok(run.reviewerAgents.some((agent) => agent.id === "export" && agent.requiredActions.some((item) => item.includes("approvedAt"))));
});

test("AI provider packages redact secrets and preserve agent governance context", () => {
  const run = buildAgenticWorkflowRun({
    instruction:
      "Create a board memo, revise for the CFO, review evidence, and prepare PDF. audience: board owner: Finance deadline: June 1 evidence: audited forecast",
    documentTitle: "Capital Allocation Memo",
    documentText: [
      "# Capital Allocation Memo",
      "",
      "Revenue grows by 18%. citation TODO",
      "Furthermore, this comprehensive analysis clearly unlocks the potential for growth.",
      "{{client_name}} must approve the investment.",
      "<!-- comment: unresolved | author: CFO | at: 2026-05-24 | Confirm revenue basis. -->",
    ].join("\n"),
    selectedText: "The investment grows revenue by 18%.",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });
  const providerPackage = buildAiProviderRequestPackage(run, {
    profileId: "openai-compatible",
    model: "approved-doc-model",
    keyEnv: "client_ai_key",
  });

  equal(providerPackage.profile.model, "approved-doc-model");
  equal(providerPackage.redactedHeaders.Authorization, "Bearer ${CLIENT_AI_KEY}");
  ok(providerPackage.systemPrompt.includes("preserve Markdown structure"));
  ok(providerPackage.userPrompt.includes("Capital Allocation Memo"));
  ok(providerPackage.userPrompt.includes("Source evidence pack:"));
  ok(providerPackage.userPrompt.includes("Line 3 [number]: Revenue grows by 18%"));
  ok(providerPackage.sourcePack.claimReview.some((item) => item.includes("Revenue grows by 18%")));
  ok(providerPackage.sourcePack.cleanupBlockers.some((item) => item.includes("comprehensive analysis")));
  ok(providerPackage.sourcePack.governanceBlockers.some((item) => item.includes("unresolved review comment")));
  ok(providerPackage.sourcePack.releaseEvidence.some((item) => item.includes("Release blocker")));
  ok(formatAiProviderSourcePack(providerPackage.sourcePack).includes("Claims and citation review:"));
  ok(formatAiProviderSourcePack(providerPackage.sourcePack).includes("Distribution blockers:"));
  ok(formatAiProviderSourcePack(providerPackage.sourcePack).includes("Release evidence bundle:"));
  ok(providerPackage.userPrompt.includes("Reviewer agents:"));
  ok(providerPackage.userPrompt.includes("Lifecycle task board:"));
  ok(providerPackage.userPrompt.includes("Release evidence bundle:"));
  ok(providerPackage.userPrompt.includes("depth; reviewers:"));
  ok(providerPackage.userPrompt.includes("Final human approval and release readiness"));
  ok(providerPackage.userPrompt.includes("Section work queue:"));
  ok(providerPackage.userPrompt.includes("Required response"));
  ok(JSON.stringify(providerPackage.requestBody).includes("approved-doc-model"));
  ok(providerPackage.curl.includes("${CLIENT_AI_KEY}"));
  ok(!providerPackage.curl.includes("client_ai_key"));
  ok(providerPackage.markdown.includes("OpenAI-compatible JSON Request Package"));
  ok(providerPackage.markdown.includes("## Source Evidence Pack"));
  ok(providerPackage.markdown.includes("Safety Checklist"));
  ok(providerPackage.checklist.some((item) => item.includes("source-pack review item")));
  ok(providerPackage.checklist.some((item) => item.includes("approves this provider")));

  const localProfiles = aiProviderProfiles.filter((profile) => profile.id === "local-openai" || profile.id === "private-openai");
  equal(localProfiles.length, 2);
  for (const profile of localProfiles) {
    const localPackage = buildAiProviderRequestPackage(run, { profileId: profile.id });
    equal(Object.keys(localPackage.redactedHeaders).includes("Authorization"), false);
    equal(localPackage.profile.authHeader, "");
    ok(localPackage.profile.summary.includes("gateway"));
    ok(JSON.stringify(localPackage.requestBody).includes(localPackage.profile.model));
  }
});

test("AI provider execution extracts Markdown without persisting secrets", async () => {
  const run = buildAgenticWorkflowRun({
    instruction: "Revise the summary for the board. audience: board owner: Strategy deadline: June 1 evidence: board pack",
    documentTitle: "Board Summary",
    documentText: "# Board Summary\n\nDraft.",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });
  const providerPackage = buildAiProviderRequestPackage(run, {
    profileId: "openai-compatible",
    model: "approved-doc-model",
    keyEnv: "NEDITOR_SECRET",
  });
  const calls: Array<{ input: string; init: { headers: Record<string, string>; body: string } }> = [];
  const result = await executeAiProviderRequestPackage(providerPackage, "session-secret", async (input, init) => {
    calls.push({ input, init });
    return {
      ok: true,
      status: 200,
      statusText: "OK",
      async text() {
        return JSON.stringify({ choices: [{ message: { content: "# Provider Draft\n\nReview-ready content." } }] });
      },
    };
  });

  equal(result.markdown, "# Provider Draft\n\nReview-ready content.");
  equal(calls[0].init.headers.Authorization, "Bearer session-secret");
  ok(!providerPackage.markdown.includes("session-secret"));
  ok(calls[0].init.body.includes("approved-doc-model"));
});

test("AI provider responses are wrapped as governed review drafts", () => {
  const reviewDraft = buildAiProviderResponseReviewMarkdown("# Provider Draft\n\nARR grows by 18%.", {
    profileLabel: "OpenAI-compatible JSON",
    model: "approved-doc-model",
    runId: "agent-20260524T100000-demo",
    generatedAt: "2026-05-24T10:00:00.000Z",
  });

  ok(reviewDraft.includes("## AI Provider Response Review Draft"));
  ok(reviewDraft.includes("```ai-source"));
  ok(reviewDraft.includes("provider: OpenAI-compatible JSON"));
  ok(reviewDraft.includes("model: approved-doc-model"));
  ok(reviewDraft.includes("status: needs-review"));
  ok(reviewDraft.includes("source=NEditor Provider Handoff"));
  ok(reviewDraft.includes("### Provider Output"));
  ok(reviewDraft.includes("# Provider Draft"));
  ok(reviewDraft.includes("### Review Before Use"));
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
    toolbarCollapsedRows: ["file", "view", "file", "", 42],
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
      htmlLanguage: " en-US ",
      htmlDescription: " Board-ready HTML summary ",
      canonicalUrl: " https://example.com/board-ready ",
      layoutPreset: "compact",
    },
    bibliographyDefaults: { citationStyle: "APA" },
    brandProfileDefaults: { color: "  #123456  ", watermark: "Draft" },
    activeExportProfileId: "client-pdf",
    exportProfiles: [
      {
        id: "client-pdf",
        name: " Client PDF ",
        exportTarget: "pdf",
        exportDefaults: { includeManifest: false, layoutPreset: "compact", pageNumbers: false },
        bibliographyDefaults: { citationStyle: "ieee" },
        brandProfileDefaults: { name: "Acme", color: " #006699 ", footer: "Confidential" },
      },
      {
        id: "client-pdf",
        name: "Duplicate ignored",
      },
      {
        id: "client-html",
        name: "",
        exportTarget: "html",
      },
    ],
    gitIntegration: { enabled: false },
    aiCleanupDefaults: { preserveHeadings: true, convertTables: false },
    agentRunHistory: [
      {
        runId: "agent-1",
        title: " Board Memo ",
        generatedAt: "2026-05-25T10:00:00.000Z",
        updatedAt: "2026-05-25T10:05:00.000Z",
        instruction: "Create a board memo",
        contextAnswers: "audience: board\nowner: CFO\nevidence: audited forecast",
        sourcePackText: "[claim] Forecast: Revenue grew 18%.",
        documentType: "board-memo",
        lanes: ["create", "review", "create"],
        distributionTargets: ["pdf", "bad-target"],
        status: "applied",
        applicationMode: "replace-document",
        readinessScore: 200,
        outputFingerprint: "abcdef0123456789",
        sourceFingerprint: "1111111111111111",
        contextFingerprint: "2222222222222222",
        instructionFingerprint: "3333333333333333",
        packetMarkdown: "# Agent Packet\n\nGenerated body",
        packetPreview: "Generated body preview",
        sectionCount: 8,
        reviewerCount: 6,
        taskCount: 14,
        lifecycleTaskStates: [
          {
            taskId: "task-intake-context",
            title: " Resolve context ",
            lane: "create",
            status: "complete",
            note: " Context approved ",
            updatedAt: "2026-05-25T10:04:00.000Z",
            completedAt: "2026-05-25T10:04:30.000Z",
          },
          {
            taskId: "task-intake-context",
            title: "Duplicate ignored",
          },
        ],
        editAcceptanceStates: [
          {
            itemId: "accept-section-summary",
            heading: " Summary ",
            scope: "section",
            status: "accepted",
            note: " CFO accepted. ",
            updatedAt: "2026-05-25T10:04:40.000Z",
            appliedAt: "2026-05-25T10:04:50.000Z",
          },
        ],
        controlCenter: {
          status: "blocked",
          readinessScore: -20,
          summary: " Resolve source gaps ",
          nextActions: [
            {
              label: " Attach evidence ",
              detail: " Add audited source links before provider handoff. ",
              lane: "review",
              action: "open-review",
              status: "needs-input",
            },
          ],
          sourceGrounding: [{ label: " Current document ", detail: " Claims found. ", status: "needs-review" }],
          governance: [{ label: " Approval ", detail: " Missing approval metadata. ", status: "missing" }],
          distribution: [{ label: " Substack ", detail: " Preflight pending. ", status: "needs-review" }],
        },
        documentEvidence: {
          unresolvedPlaceholders: ["[client]", "[client]"],
          citationTodos: ["TODO citation"],
          claimInventory: [{ kind: "number", sourceLine: 4.8, text: " Revenue grew 18%. ", reason: " metric " }],
          humanizationFindings: [
            { kind: "generic-phrase", sourceLine: 7, text: " It is important to note. ", recommendation: " Use specific owner language. " },
          ],
          reviewCommentResolutions: [
            {
              id: " review-comment-9-abc ",
              line: 9.7,
              author: " CFO ",
              createdAt: " 2026-05-25 ",
              excerpt: " Confirm forecast basis. ",
              requiredAction: " Attach source evidence. ",
              resolutionOptions: [" Resolve with source ", " Resolve with source "],
              blocker: true,
            },
          ],
          unreviewedAiMarkers: 2,
          unresolvedComments: 1,
          approvalMetadataMissing: ["approvedBy"],
          brokenLinkHints: ["https://example.com/tbd"],
          referenceHints: ["Cross reference {@missing-ref} does not match a heading slug or {#missing-ref} label in the current source."],
        },
        outlineCritique: [
          {
            severity: "warning",
            area: "coverage",
            heading: " Findings ",
            detail: " Add decision section. ",
            recommendation: " Include requested decision before appendices. ",
          },
        ],
        sourcePack: {
          contextSources: ["Current document"],
          userSources: ["[claim] Forecast: Revenue grew 18%."],
          claimReview: ["Line 4: Revenue grew 18%."],
          cleanupBlockers: ["Placeholder [client]"],
          governanceBlockers: ["Missing approvedBy"],
          distributionBlockers: ["Substack preflight pending"],
          releaseEvidence: ["Release evidence blocker"],
        },
        appliedAt: "2026-05-25T10:06:00.000Z",
        providerProfile: " local ",
      },
      {
        runId: "agent-1",
        title: "Duplicate ignored",
      },
    ],
    docsLiveDraftHistory: [
      {
        draftId: " docs-live-1 ",
        title: " Market Plan ",
        generatedAt: "2026-05-25T11:00:00.000Z",
        updatedAt: "2026-05-25T11:01:00.000Z",
        documentType: "marketing-brief",
        sectionCount: 3.8,
        issueCount: 2.2,
        outlineText: "# Outline\n\n## Launch",
        instruction: " Build a first draft ",
        markdown: "# Market Plan\n\nDraft body",
        markdownPreview: " Draft body preview ",
        reviewPacketMarkdown: "## Docs Live Review Packet\n\n- Check sources",
        reviewPacketPreview: " Check sources ",
        outputFingerprint: "4444444444444444",
      },
      {
        draftId: "docs-live-1",
        markdown: "# Duplicate ignored",
      },
      {
        draftId: "missing-markdown",
      },
    ],
    guidedDemoCompletedStepIds: ["ai-create", "", "export", "ai-create", 42],
    recentFiles: ["/a.md", 42, "/a.md", "/b.md"],
    recentFolders: ["/workspace", ""],
    workspacePath: "/legacy/workspace",
    openFiles: ["/a.md", "/b.md"],
    activeFile: "/b.md",
    scrollPositions: {
      "/a.md": { editor: 2, preview: -1 },
      "/ignored.md": "not-a-position",
    },
    mode: "outline",
    sidebar: "help",
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
  deepEqual(migrated.toolbarCollapsedRows, ["file", "view"]);
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
    htmlLanguage: "en-US",
    htmlDescription: "Board-ready HTML summary",
    canonicalUrl: "https://example.com/board-ready",
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
  equal(migrated.activeExportProfileId, "client-pdf");
  deepEqual(migrated.exportProfiles?.map((profile) => profile.id), ["client-pdf", "client-html"]);
  deepEqual(migrated.exportProfiles?.[0], {
    id: "client-pdf",
    name: "Client PDF",
    exportTarget: "pdf",
    exportDefaults: {
      includeManifest: false,
      includeStyles: true,
      includeSyntaxHighlighting: true,
      htmlLanguage: "",
      htmlDescription: "",
      canonicalUrl: "",
      coverPage: true,
      pageNumbers: false,
      layoutPreset: "compact",
      includeComments: true,
      includeProvenance: true,
      includeGlossary: true,
      includeAgenda: true,
    },
    bibliographyDefaults: { citationStyle: "ieee" },
    brandProfileDefaults: {
      name: "Acme",
      color: "#006699",
      logo: "",
      font: "",
      header: "",
      footer: "Confidential",
      watermark: "",
      legalDisclaimer: "",
    },
  });
  equal(migrated.exportProfiles?.[1]?.name, "Export profile 2");
  deepEqual(migrated.gitIntegration, { enabled: false, warnOnDirtyExport: true });
  deepEqual(migrated.aiCleanupDefaults, {
    addProvenance: true,
    markAsDraft: true,
    insertCitationTodos: true,
    preserveHeadings: true,
    convertNumberedLists: true,
    convertTables: false,
  });
  deepEqual(migrated.agentRunHistory?.[0], {
    runId: "agent-1",
    title: "Board Memo",
    generatedAt: "2026-05-25T10:00:00.000Z",
    updatedAt: "2026-05-25T10:05:00.000Z",
    instruction: "Create a board memo",
    contextAnswers: "audience: board\nowner: CFO\nevidence: audited forecast",
    sourcePackText: "[claim] Forecast: Revenue grew 18%.",
    documentType: "board-memo",
    lanes: ["create", "review"],
    distributionTargets: ["pdf"],
    status: "applied",
    applicationMode: "replace-document",
    readinessScore: 100,
    outputFingerprint: "abcdef0123456789",
    sourceFingerprint: "1111111111111111",
    contextFingerprint: "2222222222222222",
    instructionFingerprint: "3333333333333333",
    packetMarkdown: "# Agent Packet\n\nGenerated body",
    packetPreview: "Generated body preview",
    sectionCount: 8,
    reviewerCount: 6,
    taskCount: 14,
    lifecycleTaskStates: [
      {
        taskId: "task-intake-context",
        title: "Resolve context",
        lane: "create",
        status: "complete",
        note: "Context approved",
        updatedAt: "2026-05-25T10:04:00.000Z",
        completedAt: "2026-05-25T10:04:30.000Z",
      },
    ],
    editAcceptanceStates: [
      {
        itemId: "accept-section-summary",
        heading: "Summary",
        scope: "section",
        status: "accepted",
        note: "CFO accepted.",
        updatedAt: "2026-05-25T10:04:40.000Z",
        appliedAt: "2026-05-25T10:04:50.000Z",
      },
    ],
    controlCenter: {
      status: "blocked",
      readinessScore: 0,
      summary: "Resolve source gaps",
      nextActions: [
        {
          label: "Attach evidence",
          detail: "Add audited source links before provider handoff.",
          lane: "review",
          action: "open-review",
          status: "needs-input",
        },
      ],
      sourceGrounding: [{ label: "Current document", detail: "Claims found.", status: "needs-review" }],
      governance: [{ label: "Approval", detail: "Missing approval metadata.", status: "missing" }],
      distribution: [{ label: "Substack", detail: "Preflight pending.", status: "needs-review" }],
    },
    documentEvidence: {
      unresolvedPlaceholders: ["[client]"],
      citationTodos: ["TODO citation"],
      claimInventory: [{ kind: "number", sourceLine: 4, text: "Revenue grew 18%.", reason: "metric" }],
      humanizationFindings: [
        { kind: "generic-phrase", sourceLine: 7, text: "It is important to note.", recommendation: "Use specific owner language." },
      ],
      reviewCommentResolutions: [
        {
          id: "review-comment-9-abc",
          line: 9,
          author: "CFO",
          createdAt: "2026-05-25",
          excerpt: "Confirm forecast basis.",
          requiredAction: "Attach source evidence.",
          resolutionOptions: ["Resolve with source"],
          blocker: true,
        },
      ],
      unreviewedAiMarkers: 2,
      unresolvedComments: 1,
      approvalMetadataMissing: ["approvedBy"],
      brokenLinkHints: ["https://example.com/tbd"],
      referenceHints: ["Cross reference {@missing-ref} does not match a heading slug or {#missing-ref} label in the current source."],
    },
    outlineCritique: [
      {
        severity: "warning",
        area: "coverage",
        heading: "Findings",
        detail: "Add decision section.",
        recommendation: "Include requested decision before appendices.",
      },
    ],
    sourcePack: {
      contextSources: ["Current document"],
      userSources: ["[claim] Forecast: Revenue grew 18%."],
      claimReview: ["Line 4: Revenue grew 18%."],
      cleanupBlockers: ["Placeholder [client]"],
      governanceBlockers: ["Missing approvedBy"],
      distributionBlockers: ["Substack preflight pending"],
      releaseEvidence: ["Release evidence blocker"],
    },
    appliedAt: "2026-05-25T10:06:00.000Z",
    providerProfile: "local",
  });
  equal(migrated.agentRunHistory?.length, 1);
  equal(normalizeAgentRunHistory([{ runId: "" }]).length, 0);
  deepEqual(migrated.docsLiveDraftHistory?.[0], {
    draftId: "docs-live-1",
    title: "Market Plan",
    generatedAt: "2026-05-25T11:00:00.000Z",
    updatedAt: "2026-05-25T11:01:00.000Z",
    documentType: "marketing-brief",
    sectionCount: 3,
    issueCount: 2,
    outlineText: "# Outline\n\n## Launch",
    instruction: "Build a first draft",
    markdown: "# Market Plan\n\nDraft body",
    markdownPreview: "Draft body preview",
    reviewPacketMarkdown: "## Docs Live Review Packet\n\n- Check sources",
    reviewPacketPreview: "Check sources",
    outputFingerprint: "4444444444444444",
  });
  equal(migrated.docsLiveDraftHistory?.length, 1);
  equal(normalizeDocsLiveDraftHistory([{ draftId: "missing-markdown" }]).length, 0);
  deepEqual(migrated.guidedDemoCompletedStepIds, ["ai-create", "export"]);
  deepEqual(migrated.recentFiles, ["/a.md", "/b.md"]);
  deepEqual(migrated.recentFolders, ["/workspace"]);
  equal(migrated.workspaceRoot, "/legacy/workspace");
  equal(migrated.activePath, "/b.md");
  deepEqual(migrated.scrollPositions, { "/a.md": { editor: 1, preview: 0 } });
  equal(migrated.mode, "outline");
  equal(migrated.sidebar, "help");
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
  const tauriConf = readFileSync("src-tauri/tauri.conf.json", "utf8");

  ok(app.includes(':data-toolbar-display="store.toolbarDisplay"'));
  ok(app.includes(':style="appShellStyle"'));
  ok(app.includes('aria-label="Toolbar button display"'));
  ok(app.includes('aria-label="Toolbar text size"'));
  ok(app.includes("toolbarCollapsedRows"));
  ok(app.includes("command-toolbar-heading"));
  ok(app.includes("Collapse all toolbars"));
  ok(app.includes("Expand all toolbars"));
  ok(app.includes("toggleToolbarRow"));
  ok(app.includes("Help Center"));
  ok(app.includes('aria-label="Help center"'));
  ok(app.includes("filteredHelpTopics"));
  ok(app.includes("button-help-tooltip"));
  ok(app.includes('role="tooltip"'));
  ok(app.includes("handleButtonHelpEnter"));
  ok(app.includes('window.addEventListener("mouseover", handleButtonHelpEnter)'));
  ok(app.includes('window.addEventListener("focusin", handleButtonHelpEnter)'));
  ok(app.includes('button.getAttribute("data-help")'));
  ok(app.includes('button.getAttribute("aria-label")'));
  ok(app.includes('button.innerText.replace'));
  ok(app.includes("NEditor Guided Demo"));
  ok(app.includes("guidedDemoSteps"));
  ok(app.includes("guidedDemoCompletionSummary"));
  ok(app.includes("guidedDemoCompletedCount"));
  ok(app.includes("guidedDemoCompletedStepIds"));
  ok(app.includes("recordGuidedDemoStepComplete"));
  ok(app.includes("resetGuidedDemoProgress"));
  ok(app.includes("Mark done"));
  ok(app.includes("Insert checklist"));
  ok(app.includes("Copy checklist"));
  ok(app.includes("guidedDemoChecklistMarkdown"));
  ok(app.includes("## NEditor Guided Demo Checklist"));
  ok(app.includes("AI Agent Workspace"));
  ok(app.includes('aria-label="Docs Live placeholder manager"'));
  ok(app.includes('aria-label="AI Create intent brief"'));
  ok(app.includes("docsLiveIntentFields"));
  ok(app.includes("docsLiveIntentCompletion"));
  ok(app.includes("updateDocsLiveIntentField"));
  ok(app.includes("docsLivePlaceholderValue"));
  ok(app.includes("distribution target"));
  ok(app.includes("docsLivePlaceholderRows"));
  ok(app.includes("docsLiveMissingPlaceholderKeys"));
  ok(app.includes("addDocsLivePlaceholder"));
  ok(app.includes("updateDocsLivePlaceholder"));
  ok(app.includes("docsLivePlaceholderKindOptions"));
  ok(app.includes("docsLivePlaceholderReviewStatusOptions"));
  ok(app.includes("docsLivePlaceholderDraftSource"));
  ok(app.includes("updateDocsLivePlaceholderMetadata"));
  ok(app.includes("Review status for"));
  ok(app.includes("removeDocsLivePlaceholderValue"));
  ok(app.includes('aria-label="Agent workflow playbooks"'));
  ok(app.includes('aria-label="Filter agent workflow playbooks"'));
  ok(app.includes("agenticWorkflowPlaybooks"));
  ok(app.includes("filteredAgenticWorkflowPlaybooks"));
  ok(app.includes("agentPlaybookQuery"));
  ok(app.includes("agentPlaybookFocusFilter"));
  ok(app.includes("agentPlaybookTargetFilter"));
  ok(app.includes("agentPlaybookTargets"));
  ok(app.includes("agentPlaybookFocusLabel"));
  ok(app.includes("applyAgentWorkflowPlaybook"));
  ok(app.includes('aria-label="Filter agent run history"'));
  ok(app.includes("filteredAgentRunHistory"));
  ok(app.includes("agentHistoryQuery"));
  ok(app.includes("agentHistoryStatusFilter"));
  ok(app.includes("agentHistoryLaneFilter"));
  ok(app.includes("agentHistoryTargetFilter"));
  ok(app.includes("agentHistoryAuditMarkdown"));
  ok(app.includes("insertAgentHistoryAudit"));
  ok(app.includes("copyAgentHistoryAudit"));
  ok(app.includes("removeAgentHistoryRun"));
  ok(app.includes("clearAgentHistory"));
  ok(app.includes("## Agent Run History Audit"));
  ok(app.includes("```ai-audit"));
  ok(app.includes("Inserted agent history audit"));
  ok(app.includes("Copied agent history audit"));
  ok(app.includes("Removed saved agent run"));
  ok(app.includes("Cleared saved agent run history"));
  ok(app.includes("No agent runs match the current history filters."));
  ok(app.includes("Workflow Playbooks"));
  ok(app.includes("Agent Workspace playbooks"));
  ok(app.includes("AI-first platform roadmap"));
  ok(app.includes("Understand the 50 product changes"));
  ok(app.includes("release evidence bundles"));
  ok(app.includes("agent-playbooks"));
  ok(app.includes("Run a workflow playbook"));
  ok(app.includes("strategy memos, policies, release notes, grant applications"));
  ok(app.includes("Policy playbook"));
  ok(app.includes("Grant playbook"));
  ok(app.includes("agent-lifecycle-governance"));
  ok(app.includes("Agent lifecycle governance"));
  ok(app.includes("Open lifecycle board"));
  ok(app.includes("Turn AI plans into owned tasks"));
  ok(app.includes("Provider responses are applied as needs-review material"));
  ok(app.includes("lifecycle-tasks"));
  ok(app.includes("Turn plans into owned tasks"));
  ok(app.includes("provider-governance"));
  ok(app.includes("Govern provider handoffs"));
  ok(app.includes("Apply response wraps returned Markdown in AI provenance"));
  ok(app.includes("buildAgenticWorkflowPlan"));
  ok(app.includes("buildAgenticWorkflowRun"));
  ok(app.includes("agentPlan"));
  ok(app.includes("agentPlan.contextCompleteness"));
  ok(app.includes("agentPlan.revisionModes"));
  ok(app.includes("Revision passes"));
  ok(app.includes("agent-revision-modes"));
  ok(app.includes("agentPlan.qualityGates"));
  ok(app.includes("Quality gates"));
  ok(app.includes("agent-quality-gates"));
  ok(app.includes("Agent edit acceptance queue"));
  ok(app.includes("agentEditAcceptanceRows"));
  ok(app.includes("acceptedAgentEditCount"));
  ok(app.includes("setAgentEditAcceptanceStatus"));
  ok(app.includes("applyAcceptedAgentEdits"));
  ok(app.includes("Review Comment Resolution Queue"));
  ok(app.includes("agentRun.documentEvidence.reviewCommentResolutions"));
  ok(app.includes("setAgentReviewCommentStatus"));
  ok(app.includes("Release Evidence Bundle"));
  ok(app.includes("agentRun.releaseEvidenceBundle"));
  ok(app.includes("agent-release-evidence"));
  ok(app.includes("insertAgentReleaseEvidenceAuditPackage"));
  ok(app.includes("copyAgentReleaseEvidenceAuditPackage"));
  ok(app.includes("buildAgenticReleaseEvidenceAuditPackage"));
  ok(app.includes("Insert audit package"));
  ok(app.includes("Copy audit package"));
  ok(app.includes("Revise"));
  ok(app.includes("agent-context-score"));
  ok(app.includes("agentRun"));
  ok(app.includes("agentContextAnswers"));
  ok(app.includes("agentSourcePackText"));
  ok(app.includes("Source Pack Builder"));
  ok(app.includes("agentSourcePackPreview"));
  ok(app.includes("addAgentSourcePackItem"));
  ok(app.includes("removeAgentSourcePackItem"));
  ok(app.includes("commandAgentInstructionAvailable"));
  ok(app.includes("runCommandPaletteAgentInstruction"));
  ok(app.includes('aria-label="AI command route"'));
  ok(app.includes('aria-label="AI command plan preview"'));
  ok(app.includes("commandAgentPlanPreview"));
  ok(app.includes("openCommandPaletteAgentPlan"));
  ok(app.includes("Planned agent workflow from command palette instruction"));
  ok(app.includes("Generate with AI agent"));
  ok(app.includes("Plan first"));
  ok(app.includes("Generate Packet"));
  ok(app.includes("Generated agent packet from command palette instruction"));
  ok(app.includes("Context answers and constraints"));
  ok(app.includes("Replan with answers"));
  ok(app.includes("These answers feed the next plan, packet, Docs Live handoff, and provider request."));
  ok(app.includes('aria-label="Citation TODO workflow"'));
  ok(app.includes("citationTodoItems"));
  ok(app.includes("openCitationTodoCount"));
  ok(app.includes("insertCitationTodoAudit"));
  ok(app.includes("copyCitationTodoAudit"));
  ok(app.includes("resolveCitationTodoItem"));
  ok(app.includes("deferCitationTodoItem"));
  ok(app.includes("Generate agent packet"));
  ok(app.includes("Apply agent output"));
  ok(app.includes("appendAgentWorkspacePacket"));
  ok(app.includes("copyAgentWorkspacePacket"));
  ok(app.includes("Appended agent packet for review"));
  ok(app.includes("Copied current agent packet"));
  ok(app.includes("agent-run-packet-actions"));
  ok(app.includes('aria-label="Agent generated output"'));
  ok(app.includes('aria-label="AI control center"'));
  ok(app.includes('aria-label="Persistent AI control center"'));
  ok(app.includes("persistent-agent-control"));
  ok(app.includes("activeAgentControlCenter"));
  ok(app.includes("agentRun.controlCenter"));
  ok(app.includes('aria-label="Agent lifecycle task board"'));
  ok(app.includes('aria-label="Filter agent lifecycle tasks"'));
  ok(app.includes("agentLifecycleTaskRows"));
  ok(app.includes("agentLifecycleTaskTotal"));
  ok(app.includes("agentTaskLaneFilter"));
  ok(app.includes("agentTaskStatusFilter"));
  ok(app.includes("agentTaskOwnerFilter"));
  ok(app.includes("agentTaskSectionFilter"));
  ok(app.includes("agentTaskTargetFilter"));
  ok(app.includes("agentTaskEvidenceFilter"));
  ok(app.includes("agentTaskOwnerOptions"));
  ok(app.includes("agentTaskSectionOptions"));
  ok(app.includes("agentTaskTargetOptions"));
  ok(app.includes("release-blocker"));
  ok(app.includes("agentTaskQuery"));
  ok(app.includes("agentLifecycleTaskStates"));
  ok(app.includes("agentRunHistoryTaskStateSummary"));
  ok(app.includes("agentRunHistoryEvidenceSummary"));
  ok(app.includes("agentRunHistoryOutlineSummary"));
  ok(app.includes("agentRunHistorySourcePackSummary"));
  ok(app.includes("item.controlCenter"));
  ok(app.includes("item.documentEvidence"));
  ok(app.includes("item.sourcePack"));
  ok(app.includes("Task states:"));
  ok(app.includes("setAgentLifecycleTaskStatus"));
  ok(app.includes("setAgentLifecycleTaskNote"));
  ok(app.includes("Task note"));
  ok(app.includes("Needs review"));
  ok(app.includes("Execution note"));
  ok(app.includes("active.compile?.metadata.owner"));
  ok(app.includes("active.compile?.metadata.releaseTarget"));
  ok(app.includes("Release target"));
  ok(app.includes("releaseTarget"));
  ok(app.includes("runAgentLifecycleTask"));
  ok(app.includes("insertAgentLifecycleTaskBrief"));
  ok(app.includes("copyAgentLifecycleTaskBrief"));
  ok(app.includes("buildAgenticLifecycleTaskBrief"));
  ok(app.includes("Insert brief"));
  ok(app.includes("Copy brief"));
  ok(app.includes("task.sectionId"));
  ok(app.includes("task.target"));
  ok(app.includes("Lifecycle Task Board"));
  ok(app.includes('aria-label="Agent reviewer agents"'));
  ok(app.includes("agentRun.reviewerAgents"));
  ok(app.includes("reviewer.requiredActions"));
  ok(app.includes('aria-label="Agent section work queue"'));
  ok(app.includes("agentRun.sectionWorkQueue"));
  ok(app.includes("agentSectionDraftingDepthOptions"));
  ok(app.includes("section.draftingDepth"));
  ok(app.includes("section.completionCriteria"));
  ok(app.includes("insertAgentSectionBrief"));
  ok(app.includes("draftAgentSectionWithDocsLive"));
  ok(app.includes("docsLiveTargetSection"));
  ok(app.includes("Replace matching section"));
  ok(app.includes("replaceOrAppendMarkdownSection"));
  ok(app.includes("Applied Docs Live draft to"));
  ok(app.includes("appendDocsLiveDraftForReview"));
  ok(app.includes("copyDocsLiveDraft"));
  ok(app.includes("docsLiveReviewPacketMarkdown"));
  ok(app.includes("insertDocsLiveReviewPacket"));
  ok(app.includes("copyDocsLiveReviewPacket"));
  ok(app.includes("## Docs Live Review Packet"));
  ok(app.includes("type: docs-live-review-packet"));
  ok(app.includes("store.recordDocsLiveDraftHistory"));
  ok(app.includes("docsLiveDraftHistoryItem"));
  ok(app.includes("docsLiveHistoryPreview"));
  ok(app.includes("appendDocsLiveHistoryDraft"));
  ok(app.includes("copyDocsLiveHistoryDraft"));
  ok(app.includes("insertDocsLiveHistoryReviewPacket"));
  ok(app.includes("copyDocsLiveHistoryReviewPacket"));
  ok(app.includes("removeDocsLiveHistoryDraft"));
  ok(app.includes("clearDocsLiveDraftHistory"));
  ok(app.includes("latestDocsLiveDraftHistory"));
  ok(app.includes("openDocsLiveHistory"));
  ok(app.includes("appendLatestDocsLiveDraft"));
  ok(app.includes("copyLatestDocsLiveDraft"));
  ok(app.includes("insertLatestDocsLiveReviewPacket"));
  ok(app.includes("copyLatestDocsLiveReviewPacket"));
  ok(app.includes("Open Docs Live draft history"));
  ok(app.includes("Append latest Docs Live draft"));
  ok(app.includes("Copy latest Docs Live review packet"));
  ok(app.includes("Appended Docs Live draft for review"));
  ok(app.includes("Copied Docs Live draft"));
  ok(app.includes("Inserted Docs Live review packet"));
  ok(app.includes("Copied Docs Live review packet"));
  ok(app.includes("Appended saved Docs Live draft"));
  ok(app.includes("Copied saved Docs Live review packet"));
  ok(app.includes("Removed saved Docs Live draft"));
  ok(app.includes("Cleared saved Docs Live draft history"));
  ok(app.includes("docs-live-draft-actions"));
  ok(app.includes("docs-live-history"));
  ok(app.includes("Draft in Docs Live"));
  ok(app.includes('aria-label="Agent audit trail"'));
  ok(app.includes("agentRun.auditTrail"));
  ok(app.includes("Rollback plan"));
  ok(app.includes('aria-label="Agent run history"'));
  ok(app.includes("store.agentRunHistory"));
  ok(app.includes("recordAgentRunHistory"));
  ok(app.includes("replanAgentHistoryRun"));
  ok(app.includes("Replan"));
  ok(app.includes("appendAgentHistoryPacket"));
  ok(app.includes("copyAgentHistoryPacket"));
  ok(app.includes("item.packetPreview"));
  ok(app.includes("item.sectionCount"));
  ok(app.includes("item.taskCount"));
  ok(app.includes("Source grounding"));
  ok(app.includes("Distribution state"));
  ok(app.includes('aria-label="Agent distribution target runbooks"'));
  ok(app.includes("distributionTargetPlans"));
  ok(app.includes("Build provider request"));
  ok(app.includes("Copy provider package"));
  ok(app.includes("Copy source pack"));
  ok(app.includes("copyAgentProviderSourcePack"));
  ok(app.includes("agentProviderSourcePackMarkdown"));
  ok(app.includes('aria-label="AI provider source evidence pack"'));
  ok(app.includes("Run provider request"));
  ok(app.includes("buildAiProviderResponseReviewMarkdown"));
  ok(app.includes("Apply wraps this output in needs-review provenance"));
  ok(app.includes("packetMarkdownOverride"));
  ok(app.includes("stableFingerprint(packetMarkdownOverride)"));
  ok(app.includes("agentPacketPreview"));
  ok(app.includes('recordAgentRunHistory(agentRun.value, "provider-applied"'));
  ok(app.includes("Session API key"));
  ok(app.includes("executeAiProviderRequestPackage"));
  ok(app.includes('aria-label="AI provider response"'));
  ok(app.includes('aria-label="AI provider handoff"'));
  ok(app.includes("buildAiProviderRequestPackage"));
  ok(app.includes("inspectAiRuntimeReadiness"));
  ok(app.includes("Check AI runtime"));
  ok(app.includes('aria-label="AI runtime readiness"'));
  ok(app.includes('aria-label="AI runtime readiness report"'));
  ok(app.includes("AI-first document creation"));
  ok(app.includes("startAiDocumentCreation"));
  ok(app.includes('id: "ai-create", label: "AI Create"'));
  ok(app.includes('id: "agent", label: "Agent"'));
  ok(app.includes("runAgentControlAction"));
  ok(app.includes("ensureAgentPlanForControlAction"));
  ok(app.includes("normalizeAgentControlLane"));
  ok(app.includes("normalizeAgentControlWorkflowAction"));
  ok(app.includes("AgenticNextAction"));
  ok(app.includes("AgentRunHistoryNextAction"));
  ok(app.includes("Run action"));
  ok(app.includes("Help: Docs Live"));
  ok(app.includes("Help: Export and publishing"));
  ok(app.includes("Help: AI-first composition"));
  ok(app.includes('aria-label="Table of contents manager"'));
  ok(app.includes("tocDepthOptions"));
  ok(app.includes("tocDepthDraft"));
  ok(app.includes("tocNumberedDraft"));
  ok(app.includes("tocManagerSummary"));
  ok(app.includes("enableFrontMatterToc"));
  ok(app.includes("applyTocSettings"));
  ok(app.includes("frontMatterScalarValue"));
  ok(app.includes("Applied TOC settings"));
  ok(app.includes('aria-label="Captions and generated lists manager"'));
  ok(app.includes("captionedReferenceItems"));
  ok(app.includes("captionManagerSummary"));
  ok(app.includes("CaptionedReferenceItem"));
  ok(app.includes("captionKindLabel"));
  ok(app.includes("Insert list of figures"));
  ok(app.includes("Insert list of tables"));
  ok(app.includes("Insert reference"));
  ok(app.includes('aria-label="Cross reference manager"'));
  ok(app.includes('aria-label="Reference label inventory"'));
  ok(app.includes("CrossReferenceRow"));
  ok(app.includes("ReferenceLabelRow"));
  ok(app.includes("crossReferenceRows"));
  ok(app.includes("referenceLabelRows"));
  ok(app.includes("crossReferenceManagerSummary"));
  ok(app.includes("referenceLabelManagerSummary"));
  ok(app.includes("insertCrossReferenceForLabel"));
  ok(app.includes("goToReferenceLabel"));
  ok(app.includes("Insert another"));
  ok(app.includes("Go to label"));
  ok(app.includes('aria-label="Index manager"'));
  ok(app.includes("indexTermDraft"));
  ok(app.includes("indexExcludeDraft"));
  ok(app.includes("indexExclusionTerms"));
  ok(app.includes("insertIndexMarkerFromDraft"));
  ok(app.includes("addIndexExclusion"));
  ok(app.includes("removeIndexExclusion"));
  ok(app.includes("upsertFrontMatterListField"));
  ok(app.includes("frontMatterListValues"));
  ok(app.includes("Index exclusions"));
  ok(app.includes("openHelp(\"keyboard-shortcuts\")"));
  ok(app.includes("Docs Live"));
  ok(app.includes("openDocsLiveFromOutline"));
  ok(app.includes("openDocsLiveFromDocumentOutline"));
  ok(app.includes("docs-live-section-stage-list"));
  ok(app.includes("docs-live-review-packet"));
  ok(app.includes("docs-live-review-actions"));
  ok(app.includes("Docs Live review preparation packet"));
  ok(app.includes("Review preparation packet"));
  ok(app.includes("Recent Docs Live drafts"));
  ok(app.includes("Section runbook"));
  ok(app.includes("QA register"));
  ok(app.includes("Humanization checklist"));
  ok(app.includes("Review packet"));
  ok(app.includes("SpeechRecognition"));
  ok(app.includes("buildDocsLiveDraft"));
  ok(app.includes("docsLiveQuestionnaireAnswerText"));
  ok(app.includes("AI-created questionnaire"));
  ok(app.includes("Questionnaire answers"));
  ok(app.includes("Generate draft"));
  ok(app.includes("native workflow opened Docs Live from native writing tools menu"));
  ok(app.includes("native workflow generated Docs Live section draft from native writing tools menu"));
  ok(app.includes("native workflow applied Docs Live section draft for review"));
  ok(app.includes("Export HTML"));
  ok(app.includes('id: "export-html", label: "HTML Export", title: "Export standalone HTML"'));
  ok(app.includes('aria-label="HTML export options"'));
  ok(app.includes("HTML delivery"));
  ok(app.includes("store.exportDefaults.htmlLanguage"));
  ok(app.includes("store.exportDefaults.canonicalUrl"));
  ok(app.includes("exportDocumentAs(\"html\")"));
  ok(app.includes('aria-label="Export profiles"'));
  ok(app.includes("saveCurrentExportProfile"));
  ok(store.includes("saveCurrentExportProfile"));
  ok(store.includes("applyExportProfile"));
  ok(store.includes("deleteExportProfile"));
  ok(store.includes("docsLiveDraftHistory"));
  ok(store.includes("recordDocsLiveDraftHistory"));
  ok(store.includes("removeDocsLiveDraftHistory"));
  ok(store.includes("clearDocsLiveDraftHistory"));
  ok(store.includes("removeAgentRunHistory"));
  ok(store.includes("clearAgentRunHistory"));
  ok(store.includes("normalizeDocsLiveDraftHistory"));
  ok(app.includes('listen<string>("neditor-menu-command"'));
  ok(app.includes('"neditor-export-html": "html"'));
  ok(app.includes("collectNativeMenuCommandEvidence"));
  ok(app.includes("native workflow inserted table from native writing tools menu"));
  ok(app.includes("collectNativeWorkspaceTabEvidence"));
  ok(app.includes("native workflow restored workspace tabs with active pinned and scroll state"));
  ok(app.includes("flushEditorTextToStore();"));
  ok(tauriLib.includes('SubmenuBuilder::new(app, "Export")'));
  ok(tauriLib.includes('SubmenuBuilder::new(app, "Edit")'));
  ok(tauriLib.includes('SubmenuBuilder::new(app, "View")'));
  ok(tauriLib.includes('SubmenuBuilder::new(app, "Help")'));
  ok(tauriLib.includes('"neditor-export-html", "HTML Export"'));
  ok(tauriLib.includes('"neditor-open-docs-live", "Docs Live"'));
  ok(tauriLib.includes('"neditor-open-help", "NEditor Help Center"'));
  ok(tauriLib.includes('"neditor-open-agent-workspace",'));
  ok(tauriLib.includes('"neditor-ai-create-document",'));
  ok(tauriLib.includes('"neditor-guided-demo", "Guided Demo"'));
  ok(tauriLib.includes('"neditor-help-exports",'));
  ok(tauriConf.includes("connect-src 'self' ipc: https:"));
  ok(tauriLib.includes('"neditor-mode-outline", "Outline Mode"'));
  ok(app.includes('case "neditor-mode-export"'));
  ok(app.includes('case "neditor-mode-outline"'));
  ok(app.includes('case "neditor-open-help"'));
  ok(app.includes('case "neditor-open-agent-workspace"'));
  ok(app.includes('case "neditor-ai-create-document"'));
  ok(app.includes('case "neditor-guided-demo"'));
  ok(tauriLib.includes('app.emit("neditor-menu-command", id)'));
  ok(app.includes("commandToolbarRows"));
  ok(app.includes("command-toolbar-row"));
  ok(app.includes('value="outline"'));
  ok(app.includes('id="outline-mode"'));
  ok(app.includes("createOutlineHeading"));
  ok(app.includes("renameOutlineHeading"));
  ok(app.includes("deleteOutlineHeading"));
  ok(app.includes('foldGutter()'));
  ok(app.includes('codeFolding({ placeholderText: " folded " })'));
  ok(app.includes('class="icon-command"'));
  for (const label of ["Document", "Manage", "Write", "Navigate", "Insert", "Review"]) {
    ok(app.includes(`label: "${label}"`), `missing ${label} command group`);
  }
  for (const icon of ["saveAs", "snapshot", "templates", "equation", "outline", "fold", "unfold", "comment", "html", "mic", "help", "collapse", "expand"]) {
    ok(app.includes(`${icon}: [`), `missing ${icon} icon path`);
  }
  ok(app.includes('value="help"'));
  ok(store.includes('| "help"'));
  ok(app.includes('store.sidebar === \'templates\''));
  ok(store.includes("toolbarTextSize: 10"));
  ok(store.includes("toolbarCollapsedRows: []"));
  ok(types.includes("savedText?: string"));
  ok(store.includes('doc.dirty = typeof doc.savedText === "string" ? text !== doc.savedText : fallbackHash(text) !== doc.savedHash'));
  ok(store.includes("doc.savedText = response.text"));
});

test("local verification scripts expose local baseline checks", () => {
  const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
    scripts: Record<string, string>;
  };
  const { scripts } = packageJson;
  const verification = readFileSync("scripts/run-local-verification.mjs", "utf8");
  const e2eEnvironment = readFileSync("scripts/check-e2e-environment.mjs", "utf8");
  const browserEnv = readFileSync("scripts/playwright-browser-env.mjs", "utf8");
  const accessibilityGuard = readFileSync("scripts/check-accessibility.mjs", "utf8");
  const aiRoadmap = readFileSync("scripts/check-ai-first-roadmap.mjs", "utf8");
  const aiProviderEvidence = readFileSync("scripts/check-ai-provider-evidence.mjs", "utf8");
  const aiProviderCollector = readFileSync("scripts/collect-ai-provider-evidence.mjs", "utf8");
  const aiRuntimeEvidence = readFileSync("scripts/check-ai-runtime-evidence.mjs", "utf8");
  const securityReview = readFileSync("scripts/check-security-review-evidence.mjs", "utf8");
  const specCompletion = readFileSync("scripts/check-spec-completion-matrix.mjs", "utf8");
  const googleDocsImport = readFileSync("scripts/check-google-docs-import-evidence.mjs", "utf8");
  const googleDocsCollector = readFileSync("scripts/collect-google-docs-import-evidence.mjs", "utf8");
  const platformCollector = readFileSync("scripts/collect-platform-evidence.mjs", "utf8");
  const evidenceKitCollector = readFileSync("scripts/collect-release-evidence-kit.mjs", "utf8");
  const evidenceKitChecker = readFileSync("scripts/check-release-evidence-kit.mjs", "utf8");
  const evidenceIngest = readFileSync("scripts/ingest-release-evidence.mjs", "utf8");
  const releaseCi = readFileSync("scripts/check-release-ci-workflow.mjs", "utf8");
  const releaseWorkflow = readFileSync(".github/workflows/neditor-release-evidence.yml", "utf8");
  const platformPackaging = readFileSync("scripts/check-platform-packaging.mjs", "utf8");
  const platformEvidence = readFileSync("scripts/check-platform-evidence.mjs", "utf8");
  const performanceProfile = readFileSync("scripts/check-performance-profile-evidence.mjs", "utf8");
  const signingCollector = readFileSync("scripts/collect-release-signing-evidence.mjs", "utf8");
  const releaseSigning = readFileSync("scripts/check-release-signing.mjs", "utf8");

  equal(scripts.check, "vue-tsc --noEmit");
  equal(scripts["check:ai-roadmap"], "node scripts/check-ai-first-roadmap.mjs");
  equal(scripts["check:ai-provider"], "node scripts/check-ai-provider-evidence.mjs");
  equal(scripts["check:ai-runtime"], "node scripts/check-ai-runtime-evidence.mjs");
  equal(scripts["check:a11y"], "node scripts/check-accessibility.mjs");
  equal(scripts["check:a11y:manual"], "node scripts/check-accessibility-manual-signoff.mjs");
  equal(scripts["check:a11y:runtime"], "node scripts/check-accessibility-runtime.mjs");
  equal(scripts["check:deps"], "node scripts/check-dependency-admission.mjs");
  equal(scripts["check:docs"], "node scripts/check-markdown-links.mjs");
  equal(scripts["check:engines"], "node scripts/check-external-engines.mjs");
  equal(scripts["check:e2e-env"], "node scripts/check-e2e-environment.mjs");
  equal(scripts["check:google-docs-import"], "node scripts/check-google-docs-import-evidence.mjs");
  equal(scripts["check:platform-evidence"], "node scripts/check-platform-evidence.mjs");
  equal(scripts["check:platform-packaging"], "node scripts/check-platform-packaging.mjs");
  equal(scripts["check:performance-profile"], "node scripts/check-performance-profile-evidence.mjs");
  equal(scripts["check:release-ci"], "node scripts/check-release-ci-workflow.mjs");
  equal(scripts["check:evidence-kit"], "node scripts/check-release-evidence-kit.mjs");
  equal(scripts["check:release-signing"], "node scripts/check-release-signing.mjs");
  equal(scripts["check:release-readiness"], "node scripts/check-release-readiness.mjs");
  equal(scripts["check:security-review"], "node scripts/check-security-review-evidence.mjs");
  equal(scripts["check:spec-completion"], "node scripts/check-spec-completion-matrix.mjs");
  equal(scripts["check:structure"], "node scripts/check-project-structure.mjs");
  equal(scripts["collect:ai-provider"], "node scripts/collect-ai-provider-evidence.mjs");
  equal(scripts["collect:google-docs-import"], "node scripts/collect-google-docs-import-evidence.mjs");
  equal(scripts["collect:platform-evidence"], "node scripts/collect-platform-evidence.mjs");
  equal(scripts["collect:evidence-kit"], "node scripts/collect-release-evidence-kit.mjs");
  equal(scripts["collect:release-signing"], "node scripts/collect-release-signing-evidence.mjs");
  equal(scripts["ingest:evidence"], "node scripts/ingest-release-evidence.mjs");
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
  ok(verification.includes('command("Browser workflow environment", "node", ["scripts/check-e2e-environment.mjs"])'));
  ok(verification.includes('command("Browser workflow suite", "node", ["scripts/run-e2e.mjs"])'));
  ok(verification.includes('command("Accessibility runtime audit", "pnpm", ["run", "check:a11y:runtime"])'));
  ok(verification.includes('command("Accessibility manual review contract", "pnpm", ["run", "check:a11y:manual"])'));
  ok(verification.includes('command("Google Docs import evidence contract", "pnpm", ["run", "check:google-docs-import"])'));
  ok(verification.includes('command("AI-first roadmap contract", "pnpm", ["run", "check:ai-roadmap"])'));
  ok(verification.includes('command("AI provider evidence contract", "pnpm", ["run", "check:ai-provider"])'));
  ok(verification.includes('command("AI runtime evidence contract", "pnpm", ["run", "check:ai-runtime"])'));
  ok(verification.includes('command("Security review evidence contract", "pnpm", ["run", "check:security-review"])'));
  ok(verification.includes('command("AI provider live endpoint evidence contract", "pnpm", ["run", "check:ai-provider"])'));
  ok(verification.includes('command("Release device performance profile contract", "pnpm", ["run", "check:performance-profile"])'));
  ok(verification.includes('command("Platform package configuration", "pnpm", ["run", "check:platform-packaging"])'));
  ok(verification.includes('command("Release evidence workflow guard", "pnpm", ["run", "check:release-ci"])'));
  ok(verification.includes('command("External platform evidence contract", "pnpm", ["run", "check:platform-evidence"])'));
  ok(verification.includes('command("Release signing evidence contract", "pnpm", ["run", "check:release-signing"])'));
  ok(verification.includes('command("Spec completion matrix contract", "pnpm", ["run", "check:spec-completion"])'));
  ok(verification.includes('command("Release evidence kit generation", "pnpm", ["run", "collect:evidence-kit"])'));
  ok(verification.includes('command("Release evidence kit contract", "pnpm", ["run", "check:evidence-kit"])'));
  ok(verification.includes('command("Release readiness aggregation", "pnpm", ["run", "check:release-readiness"])'));
  ok(verification.includes("Desktop macOS GUI launch smoke"));
  ok(verification.includes('NEDITOR_DESKTOP_SMOKE_LAUNCH: "1"'));
  ok(verification.includes("env: { ...process.env, ...item.env }"));
  ok(e2eEnvironment.includes("NEDITOR_E2E_ENV_ATTEMPTS"));
  ok(e2eEnvironment.includes("NEDITOR_E2E_ENV_RETRY_BACKOFF_MS"));
  ok(e2eEnvironment.includes("isTransientBrowserLaunchFailure"));
  ok(browserEnv.includes('join(root, ".tmp", "ms-playwright")'));
  ok(browserEnv.includes("PLAYWRIGHT_BROWSERS_PATH: baseEnv.PLAYWRIGHT_BROWSERS_PATH ?? projectBrowserCache"));
  ok(accessibilityGuard.includes("button-hover-help"));
  ok(accessibilityGuard.includes("handleButtonHelpEnter"));
  ok(accessibilityGuard.includes("data-help fallback"));
  ok(accessibilityGuard.includes("disabled help fallback"));
  ok(accessibilityGuard.includes("role\\s*=\\s*[\"']tooltip"));
  ok(aiRoadmap.includes("neditor.ai-first-roadmap-report.v1"));
  ok(aiRoadmap.includes("roadmap must contain exactly 50 numbered changes"));
  ok(aiRoadmap.includes("docs/ai-first-platform-roadmap.md"));
  ok(aiRoadmap.includes("AI Agent Workspace"));
  ok(aiRoadmap.includes("NEditor guided demo"));
  ok(aiProviderEvidence.includes("neditor.ai-provider-evidence.v1"));
  ok(aiProviderEvidence.includes("NEDITOR_AI_PROVIDER_EVIDENCE_DIR"));
  ok(aiProviderEvidence.includes("pending-live-provider-evidence"));
  ok(aiProviderEvidence.includes("NEDITOR_PROVIDER_EVIDENCE_OK"));
  ok(aiProviderEvidence.includes("appVersion must match package.json version"));
  ok(aiProviderEvidence.includes("sourceCommit must match current git commit"));
  ok(aiProviderEvidence.includes("sourceTreeClean must be true"));
  ok(aiProviderEvidence.includes("evidence must not contain API key-looking secrets"));
  ok(aiProviderEvidence.includes("local-openai"));
  ok(aiProviderEvidence.includes("private-openai"));
  ok(aiProviderCollector.includes("NEDITOR_AI_PROVIDER_PROFILE"));
  ok(aiProviderCollector.includes("NEDITOR_AI_PROVIDER_ENDPOINT"));
  ok(aiProviderCollector.includes("NEDITOR_AI_PROVIDER_MODEL"));
  ok(aiProviderCollector.includes("NEDITOR_AI_PROVIDER_API_KEY_ENV"));
  ok(aiProviderCollector.includes("AI provider evidence must be collected from a clean Git tree"));
  ok(aiProviderCollector.includes("secretMaterialStored: false"));
  ok(aiProviderCollector.includes("local-openai"));
  ok(aiProviderCollector.includes("private-openai"));
  ok(aiProviderCollector.includes("anthropic-version"));
  ok(aiProviderCollector.includes("gemini-generate-content"));
  ok(aiRuntimeEvidence.includes("neditor.ai-runtime-evidence.v1"));
  ok(aiRuntimeEvidence.includes("NEDITOR_AI_RUNTIME_EVIDENCE_DIR"));
  ok(aiRuntimeEvidence.includes("NEDITOR_AI_RUNTIME_EVIDENCE"));
  ok(aiRuntimeEvidence.includes("pending-real-runtime-evidence"));
  ok(aiRuntimeEvidence.includes("appVersion must match package.json version"));
  ok(aiRuntimeEvidence.includes("sourceCommit must match current git commit"));
  ok(aiRuntimeEvidence.includes("sourceTreeClean must be true"));
  ok(aiRuntimeEvidence.includes("speechRecognition.state must be available"));
  ok(aiRuntimeEvidence.includes("microphonePermission.state must be granted"));
  ok(aiRuntimeEvidence.includes("microphoneProbe.audioStored must be false"));
  ok(aiRuntimeEvidence.includes("clipboardRead.contentStored must be false"));
  ok(aiRuntimeEvidence.includes("clipboardWrite.writeSucceeded must be true"));
  ok(aiRuntimeEvidence.includes("forbiddenEvidenceKeys"));
  ok(aiRuntimeEvidence.includes("clipboardText"));
  ok(aiRuntimeEvidence.includes("audioSample"));
  ok(securityReview.includes("neditor.security-review-evidence.v1"));
  ok(securityReview.includes("NEDITOR_SECURITY_REVIEW_EVIDENCE_DIR"));
  ok(securityReview.includes("NEDITOR_SECURITY_REVIEW_EVIDENCE"));
  ok(securityReview.includes("pending-independent-security-review"));
  ok(securityReview.includes("sourceCommit must match current git commit"));
  ok(securityReview.includes("sourceTreeClean must be true"));
  ok(securityReview.includes("tauri-command-boundary"));
  ok(securityReview.includes("external-transform-boundary"));
  ok(securityReview.includes("ai-provider-boundary"));
  ok(securityReview.includes("findings.critical must be 0"));
  ok(securityReview.includes("signoff.approvedForRelease must be true"));
  ok(securityReview.includes("signoff.networkTelemetryAdded must be false"));
  ok(specCompletion.includes("neditor.spec-completion-report.v1"));
  ok(specCompletion.includes("partial-with-release-risks"));
  ok(specCompletion.includes("Current major verification gaps"));
  ok(specCompletion.includes("Next Matrix Work"));
  ok(specCompletion.includes("openRows"));
  ok(specCompletion.includes("Partial"));
  ok(specCompletion.includes("Unverified"));
  ok(specCompletion.includes("Missing"));
  ok(specCompletion.includes("remaining gap is empty or placeholder"));
  ok(performanceProfile.includes("neditor.performance-profile-evidence.v1"));
  ok(performanceProfile.includes("NEDITOR_PERFORMANCE_PROFILE_EVIDENCE_DIR"));
  ok(performanceProfile.includes("NEDITOR_PERFORMANCE_PROFILE_EVIDENCE"));
  ok(performanceProfile.includes("pending-release-device-profile"));
  ok(performanceProfile.includes("sourceCommit must match current git commit"));
  ok(performanceProfile.includes("sourceTreeClean must be true"));
  ok(performanceProfile.includes("durationMinutes must be at least 30"));
  ok(performanceProfile.includes("startup-open-document"));
  ok(performanceProfile.includes("large-document-edit-preview"));
  ok(performanceProfile.includes("agent-workflow-review"));
  ok(performanceProfile.includes("binary.sha256 must be a 64-character SHA-256"));
  ok(googleDocsImport.includes("neditor.google-docs-import-evidence.v1"));
  ok(googleDocsImport.includes("appVersion must match package.json version"));
  ok(googleDocsImport.includes("sourceCommit must match current git commit"));
  ok(googleDocsImport.includes("sourceTreeClean must be true"));
  ok(googleDocsImport.includes("NEDITOR_GOOGLE_DOCS_IMPORT_EVIDENCE"));
  ok(googleDocsImport.includes("pending-google-drive-authorization"));
  ok(googleDocsImport.includes("rendered-export-audit.google-docs.zip"));
  ok(googleDocsImport.includes("Rendered Export Audit"));
  ok(googleDocsCollector.includes("neditor.google-docs-import-evidence.v1"));
  ok(googleDocsCollector.includes("NEDITOR_GOOGLE_DOCS_EXPORTED_DOCX"));
  ok(googleDocsCollector.includes("NEDITOR_GOOGLE_DOCS_READBACK_TEXT_FILE"));
  ok(googleDocsCollector.includes("Google Docs import evidence must be collected from a clean Git tree"));
  ok(googleDocsCollector.includes("importMethod: \"google-drive-import-document\""));
  ok(googleDocsCollector.includes("Control summary"));
  ok(googleDocsCollector.includes("AI Provenance"));
  ok(platformPackaging.includes("platform-package-config-report.json"));
  ok(platformPackaging.includes("unsigned-local-builds"));
  ok(platformPackaging.includes("windowsTilePng"));
  ok(platformPackaging.includes("Tauri bundle targets must remain all-platform"));
  ok(platformEvidence.includes("neditor.platform-package-artifacts.v1"));
  ok(platformEvidence.includes("appVersion must match package.json version"));
  ok(platformEvidence.includes("sourceCommit must match current git commit"));
  ok(platformEvidence.includes("sourceTreeClean must be true"));
  ok(platformEvidence.includes("win32/tauri-webdriver-report.json"));
  ok(platformEvidence.includes("linux/package-artifacts.json"));
  ok(platformEvidence.includes("pending-external-evidence"));
  ok(platformEvidence.includes("replace-with-64-character-sha256"));
  ok(platformEvidence.includes("requiredWebdriverAssertions"));
  ok(platformEvidence.includes("desktop WebDriver edits document structure in outline mode"));
  ok(platformEvidence.includes("desktop WebDriver renames, duplicates, and exposes reveal affordance for real Markdown files"));
  ok(platformEvidence.includes("outlineArtifacts.sourceEvidence.newSubsection must be true"));
  ok(platformEvidence.includes("outlineArtifacts.sourceEvidence.sourceGovernancePreserved must be true"));
  ok(platformCollector.includes("/^[ MADRCU?!]{1,2}\\s+/"));
  ok(platformCollector.includes('["src-tauri/Cargo.lock", "src-tauri/Cargo.toml"].includes(path)'));
  ok(platformEvidence.includes("exportArtifacts.progressEvidence must include a completed render step"));
  ok(platformCollector.includes("NEDITOR_PLATFORM_EVIDENCE_PLATFORM"));
  ok(platformCollector.includes("NEDITOR_PLATFORM_BUILD_COMMAND"));
  ok(platformCollector.includes("NEDITOR_SOURCE_COMMIT"));
  ok(platformCollector.includes("neditor.platform-package-artifacts.v1"));
  ok(platformCollector.includes("Desktop WebDriver report sourceCommit"));
  ok(platformCollector.includes("Desktop WebDriver report sourceTreeClean must be true"));
  ok(platformCollector.includes("Platform evidence must be collected from a clean Git tree"));
  ok(platformCollector.includes("Run pnpm run test:tauri-webdriver first"));
  ok(platformCollector.includes("Desktop WebDriver report status must be passed"));
  ok(releaseSigning.includes("neditor.release-signing-evidence.v1"));
  ok(releaseSigning.includes("releaseVersion must match package.json version"));
  ok(releaseSigning.includes("sourceCommit must match current git commit"));
  ok(releaseSigning.includes("sourceTreeClean must be true"));
  ok(releaseSigning.includes("darwin/signing-evidence.json"));
  ok(releaseSigning.includes("win32/signing-evidence.json"));
  ok(releaseSigning.includes("linux/signing-evidence.json"));
  ok(releaseSigning.includes("pending-release-credentials"));
  ok(releaseSigning.includes("codesign --verify"));
  ok(signingCollector.includes("NEDITOR_RELEASE_SIGNING_PLATFORM"));
  ok(signingCollector.includes("NEDITOR_RELEASE_VERSION"));
  ok(signingCollector.includes("NEDITOR_SOURCE_COMMIT"));
  ok(signingCollector.includes("neditor.release-signing-evidence.v1"));
  ok(signingCollector.includes("Release version must match package.json version"));
  ok(signingCollector.includes("Release signing evidence must be collected from a clean Git tree"));
  ok(signingCollector.includes("Missing required"));
  ok(signingCollector.includes("Release signing proof command failed"));
  ok(evidenceKitCollector.includes("neditor.release-evidence-kit.v1"));
  ok(evidenceKitCollector.includes("windows-package-artifact-proof"));
  ok(evidenceKitCollector.includes("linux-package-artifact-proof"));
  ok(evidenceKitCollector.includes("ai-provider-live-endpoint-proof"));
  ok(evidenceKitCollector.includes("ai-runtime-real-device-proof"));
  ok(evidenceKitCollector.includes("independent-security-review-signoff"));
  ok(evidenceKitCollector.includes("release-device-native-performance-profile"));
  ok(evidenceKitCollector.includes("google-docs-live-import-readback"));
  ok(evidenceKitCollector.includes("rendered-export-native-viewer-human-signoff"));
  ok(evidenceKitCollector.includes("accessibility-assistive-technology-human-signoff"));
  ok(evidenceKitCollector.includes("optional-external-engines"));
  ok(evidenceKitCollector.includes("sourceTreeClean"));
  ok(evidenceKitCollector.includes("staleTemplates"));
  ok(evidenceKitCollector.includes("inspectTemplateFreshness"));
  ok(evidenceKitCollector.includes("sourceCommit"));
  ok(evidenceKitCollector.includes("Optional CI path: gh workflow run neditor-release-evidence.yml"));
  ok(evidenceKitCollector.includes("provider-evidence.template.json"));
  ok(evidenceKitCollector.includes("runtime-evidence.template.json"));
  ok(evidenceKitCollector.includes("security-review.template.json"));
  ok(evidenceKitCollector.includes("native-profile.template.json"));
  ok(evidenceKitCollector.includes("visual-review-signoff.template.json"));
  ok(evidenceKitCollector.includes("manual-review-template.json"));
  ok(evidenceKitCollector.includes("pikchr.template.json"));
  ok(evidenceKitCollector.includes("spec-completion-open-items"));
  ok(evidenceKitCollector.includes("runbooks/spec-completion-closure.md"));
  ok(evidenceKitCollector.includes("gapWorkItems"));
  ok(evidenceKitCollector.includes("pnpm run ingest:evidence"));
  ok(evidenceKitChecker.includes("neditor.release-evidence-kit.v1"));
  ok(evidenceKitChecker.includes("neditor.release-evidence-kit-report.v1"));
  ok(evidenceKitChecker.includes("runbooks/ai-provider-endpoint.md"));
  ok(evidenceKitChecker.includes("runbooks/ai-runtime-device.md"));
  ok(evidenceKitChecker.includes("runbooks/independent-security-review.md"));
  ok(evidenceKitChecker.includes("runbooks/release-device-performance-profile.md"));
  ok(evidenceKitChecker.includes("runbooks/optional-external-engines.md"));
  ok(evidenceKitChecker.includes("runbooks/spec-completion-closure.md"));
  ok(evidenceKitChecker.includes("expectedTemplateCount = 15"));
  ok(evidenceKitChecker.includes("report.json"));
  ok(evidenceKitChecker.includes("sourceTreeClean must be true"));
  ok(evidenceKitChecker.includes("current source tree must be clean"));
  ok(evidenceKitChecker.includes("currentSourceTreeClean"));
  ok(evidenceKitChecker.includes("staleTemplates must be empty"));
  ok(evidenceKitChecker.includes("missingTemplates must be empty"));
  ok(evidenceKitChecker.includes("manifest gaps must mirror the release readiness report"));
  ok(evidenceIngest.includes("neditor.release-evidence-ingest.v1"));
  ok(evidenceIngest.includes("NEDITOR_RELEASE_EVIDENCE_RETURN_DIR"));
  ok(evidenceIngest.includes("platform/win32-package-artifacts.json"));
  ok(evidenceIngest.includes("ai-provider/provider-evidence.json"));
  ok(evidenceIngest.includes("ai-runtime/runtime-evidence.json"));
  ok(evidenceIngest.includes("security-review-signoff"));
  ok(evidenceIngest.includes("security/security-review.json"));
  ok(evidenceIngest.includes("check:security-review"));
  ok(evidenceIngest.includes("performance-native-profile"));
  ok(evidenceIngest.includes("performance/native-profile.json"));
  ok(evidenceIngest.includes("check:performance-profile"));
  ok(evidenceIngest.includes("NEDITOR_RENDERED_EXPORT_SIGNOFF"));
  ok(evidenceIngest.includes("NEDITOR_ACCESSIBILITY_SIGNOFF"));
  ok(evidenceIngest.includes("external-engine-pikchr"));
  ok(evidenceIngest.includes("external-engines/external/pikchr.json"));
  ok(evidenceIngest.includes("check:engines"));
  ok(evidenceIngest.includes("pnpm"));
  ok(evidenceIngest.includes("check:release-signing"));
  ok(evidenceIngest.includes("check:ai-provider"));
  ok(evidenceIngest.includes("check:ai-runtime"));
  ok(evidenceIngest.includes("check:google-docs-import"));
  ok(releaseCi.includes("neditor.release-ci-workflow-report.v1"));
  ok(releaseCi.includes("browser-workflows:"));
  ok(releaseCi.includes("platform-proof:"));
  ok(releaseCi.includes("rendered-export-review:"));
  ok(releaseCi.includes("accessibility-review:"));
  ok(releaseCi.includes("FORCE_JAVASCRIPT_ACTIONS_TO_NODE24"));
  ok(releaseCi.includes("NEDITOR_TAURI_WEBDRIVER_TIMEOUT_MS"));
  ok(releaseCi.includes("xvfb-run -a pnpm run test:tauri-webdriver -- --strict"));
  ok(releaseCi.includes("pnpm tauri build --bundles ${{ matrix.bundles }}"));
  ok(releaseCi.includes(".tmp/platform-evidence/external/${{ matrix.platform }}/tauri-webdriver-report.json"));
  ok(releaseCi.includes(".tmp/rendered-export-audit/**"));
  ok(releaseCi.includes(".tmp/accessibility/**"));
  ok(releaseWorkflow.includes("name: NEditor Release Evidence"));
  ok(releaseWorkflow.includes("workflow_dispatch:"));
  ok(releaseWorkflow.includes('FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: "true"'));
  ok(releaseWorkflow.includes('NEDITOR_TAURI_WEBDRIVER_TIMEOUT_MS: "90000"'));
  ok(releaseWorkflow.includes("Browser workflow proof"));
  ok(releaseWorkflow.includes("PLAYWRIGHT_BROWSERS_PATH=.tmp/ms-playwright pnpm exec playwright install --with-deps chromium"));
  ok(releaseWorkflow.includes("pnpm run check:e2e-env"));
  ok(releaseWorkflow.includes("pnpm run test:e2e"));
  ok(releaseWorkflow.includes("platform: win32"));
  ok(releaseWorkflow.includes("platform: linux"));
  ok(releaseWorkflow.includes("bundles: msi nsis"));
  ok(releaseWorkflow.includes("bundles: deb rpm appimage"));
  ok(releaseWorkflow.includes("cargo install tauri-driver --locked"));
  ok(releaseWorkflow.includes("webkit2gtk-driver"));
  ok(releaseWorkflow.includes("choco install selenium-chromium-edge-driver"));
  ok(releaseWorkflow.includes("MSEDGEDRIVER_TELEMETRY_OPTOUT"));
  ok(releaseWorkflow.includes("pnpm run collect:platform-evidence"));
  ok(releaseWorkflow.includes("pnpm run check:platform-evidence"));
  ok(releaseWorkflow.includes("Rendered export review package"));
  ok(releaseWorkflow.includes("poppler-utils"));
  ok(releaseWorkflow.includes("libwebkit2gtk-4.1-dev"));
  ok(releaseWorkflow.includes("pnpm run test:rendered-exports"));
  ok(releaseWorkflow.includes("neditor-rendered-export-review-package"));
  ok(releaseWorkflow.includes("Accessibility review package"));
  ok(releaseWorkflow.includes("pnpm run check:a11y:runtime"));
  ok(releaseWorkflow.includes("pnpm run check:a11y:manual"));
  ok(releaseWorkflow.includes("neditor-accessibility-review-package"));
});

test("runtime accessibility audit executes focused browser workflows", () => {
  const script = readFileSync("scripts/check-accessibility-runtime.mjs", "utf8");
  ok(script.includes("exposes keyboard skip links to primary workbench regions"));
  ok(script.includes("keeps primary workbench regions accessible across desktop and narrow viewports"));
  ok(script.includes("manages modal focus and Escape return paths"));
  ok(script.includes("supports keyboard-only operation for deep workbench controls"));
  ok(script.includes("exposes status and progress messages as live regions"));
  ok(script.includes("persists editor settings and runs search plus heading commands"));
  ok(script.includes(".tmp"));
  ok(script.includes("accessibility"));
  ok(script.includes("runtime-report.json"));
  ok(script.includes("scripts/run-e2e.mjs"));
  ok(script.includes("--grep"));
  ok(script.includes("findSystemChromium"));
  ok(script.includes("system-chromium-fallback"));
});

test("manual accessibility signoff validates screen-reader review evidence", () => {
  const script = readFileSync("scripts/check-accessibility-manual-signoff.mjs", "utf8");
  ok(script.includes("neditor.accessibility.manual-signoff.v1"));
  ok(script.includes("NEDITOR_ACCESSIBILITY_SIGNOFF"));
  ok(script.includes("manual-review-template.json"));
  ok(script.includes("manual-review-summary.json"));
  ok(script.includes("screen-reader-workbench-regions"));
  ok(script.includes("keyboard-only-core-workflows"));
  ok(script.includes("native-desktop-traversal"));
  ok(script.includes("assistiveTechnology"));
  ok(script.includes("appVersion"));
  ok(script.includes("sourceCommit"));
  ok(script.includes("sourceTreeClean"));
  ok(script.includes("sourceTreeClean = gitTreeClean()"));
  ok(script.includes("completed sign-off appVersion must match package.json version"));
  ok(script.includes("completed sign-off sourceCommit must match current git commit"));
  ok(script.includes("completed sign-off sourceTreeClean must be true"));
  ok(script.includes("requiredReviewSessions"));
  ok(script.includes("screen-reader-navigation"));
  ok(script.includes("native-desktop-shell"));
  ok(script.includes("validateReviewSessions"));
  ok(script.includes("durationMinutes"));
  ok(script.includes("evidenceReference"));
  ok(script.includes("substantive reviewer notes"));
  ok(script.includes("prerequisiteReports"));
  ok(script.includes("validatePrerequisiteIdentity"));
  ok(script.includes("sha256 must match the current report"));
  ok(script.includes("sha256Text"));
  ok(script.includes("unresolvedBlockers"));
  ok(script.includes("pending-human-review"));
  ok(script.includes("human-reviewed"));
});

test("release readiness aggregation records external evidence gaps", () => {
  const script = readFileSync("scripts/check-release-readiness.mjs", "utf8");
  ok(script.includes("current-host-ready-with-external-gaps"));
  ok(script.includes("browserWorkflowAccepted"));
  ok(script.includes("runtimeAccessibilityAccepted"));
  ok(script.includes("performanceAuditAccepted"));
  ok(script.includes("focusedPlaywrightReportAccepted"));
  ok(script.includes("macosAppBundleAccepted"));
  ok(script.includes("macosDmgAccepted"));
  ok(script.includes("artifactMatchesReport"));
  ok(script.includes("reportFileFreshForArtifact"));
  ok(script.includes("native-command-report-stale-for-binary"));
  ok(script.includes("launch-report-stale-for-binary"));
  ok(script.includes("webdriver-report-stale-for-binary"));
  ok(script.includes("fallback-smoke-report-stale-for-binary"));
  ok(script.includes("neditor.e2e-browser-workflow.v1"));
  ok(script.includes("scope !== \"full-suite\""));
  ok(script.includes("missing-docs-live-workflow-proof"));
  ok(script.includes("invalid-focused-e2e-report"));
  ok(script.includes("invalid-large-document-e2e-report"));
  ok(script.includes("older-than-app-bundle-report"));
  ok(script.includes("hdiutil sandbox limitation classified with app bundle fallback proof"));
  ok(script.includes("freshForSources"));
  ok(script.includes("windows-linux-tauri-webdriver-execution"));
  ok(script.includes("external-platform-evidence"));
  ok(script.includes("missingPlatformEvidence"));
  ok(script.includes("if (missingWebdriverPlatforms.length > 0)"));
  ok(!script.includes("missingWebdriverPlatforms.length > 0 ||"));
  ok(script.includes("release-signing-evidence"));
  ok(script.includes("release-ci-workflow"));
  ok(script.includes("releaseCiWorkflowAccepted"));
  ok(script.includes("neditor.release-ci-workflow-report.v1"));
  ok(script.includes("stale-for-release-ci-sources"));
  ok(script.includes("release-evidence-kit"));
  ok(script.includes("releaseEvidenceKitAccepted"));
  ok(script.includes("neditor.release-evidence-kit-report.v1"));
  ok(script.includes("current-source-tree-not-clean"));
  ok(script.includes("summary?.copiedTemplates || 0) < 15"));
  ok(script.includes("summary?.runbooks || 0) < 12"));
  ok(script.includes("security-review-evidence"));
  ok(script.includes("independent-security-review-signoff"));
  ok(script.includes("securityReviewEvidenceAccepted"));
  ok(script.includes("spec-completion-matrix"));
  ok(script.includes("spec-completion-open-items"));
  ok(script.includes("specCompletionAccepted"));
  ok(script.includes("neditor.spec-completion-report.v1"));
  ok(script.includes("performance-profile-evidence"));
  ok(script.includes("release-device-native-performance-profile"));
  ok(script.includes("performanceProfileEvidenceAccepted"));
  ok(script.includes("missingReleaseSigningEvidence"));
  ok(script.includes("ai-provider-evidence"));
  ok(script.includes("ai-provider-live-endpoint-proof"));
  ok(script.includes("aiProviderEvidenceAccepted"));
  ok(script.includes("ai-runtime-evidence"));
  ok(script.includes("ai-runtime-real-device-proof"));
  ok(script.includes("aiRuntimeEvidenceAccepted"));
  ok(script.includes("google-docs-import-evidence"));
  ok(script.includes("google-docs-live-import-readback"));
  ok(script.includes("release-signing-and-notarization"));
  ok(script.includes("accessibility-assistive-technology-human-signoff"));
  ok(script.includes("rendered-export-native-viewer-human-signoff"));
  ok(script.includes("renderedExportAuditAccepted"));
  ok(script.includes('"markdown-bundle", "blog", "substack", "latex", "google-docs"'));
  ok(script.includes('"rich-blocks", "option-heavy"'));
  ok(script.includes("invalidExternalEvidence"));
  ok(script.includes("engine.externalEvidence?.status !== \"accepted\""));
  ok(script.includes('"release-readiness"'));
  ok(script.includes("runtime-report.json"));
  ok(script.includes("platform-package-config-report.json"));
  ok(script.includes("fresh native fallback proof"));
});

test("browser e2e runner emits structured workflow evidence for release readiness", () => {
  const script = readFileSync("scripts/run-e2e.mjs", "utf8");

  ok(script.includes("neditor.e2e-browser-workflow.v1"));
  ok(script.includes("NEDITOR_E2E_REPORT_PATH"));
  ok(script.includes("full-suite"));
  ok(script.includes("focused-report.json"));
  ok(script.includes("summarizePlaywrightOutput"));
  ok(script.includes("workflowEvidence"));
  ok(script.includes("docsLiveDraft"));
  ok(script.includes("generates a Docs Live draft from outline, context, and placeholders"));
  ok(script.includes("stdoutTail"));
  ok(script.includes("stderrTail"));
});

test("focused browser audits write dedicated reports without replacing the full-suite proof", () => {
  const accessibility = readFileSync("scripts/check-accessibility-runtime.mjs", "utf8");
  const performance = readFileSync("scripts/check-performance-audit.mjs", "utf8");

  ok(accessibility.includes("NEDITOR_E2E_REPORT_PATH"));
  ok(accessibility.includes("e2e-runtime-report.json"));
  ok(performance.includes("NEDITOR_E2E_REPORT_PATH"));
  ok(performance.includes("e2e-large-document-report.json"));
  ok(performance.includes("evidenceReport"));
});

test("external engine probe records render smoke artifacts", () => {
  const script = readFileSync("scripts/check-external-engines.mjs", "utf8");

  ok(script.includes("artifactDir"));
  ok(script.includes("runSmoke"));
  ok(script.includes("Installed engines with failed smoke proof"));
  ok(script.includes("plantuml-file"));
  ok(script.includes("pikchr-cli"));
  ok(script.includes("missingNeedles"));
  ok(script.includes("neditor.external-engine-evidence.v1"));
  ok(script.includes("NEDITOR_EXTERNAL_ENGINE_EVIDENCE_DIR"));
  ok(script.includes("externalEvidence"));
  ok(script.includes("invalidExternalEvidence"));
  ok(script.includes("missingEvidence"));
  ok(script.includes("writeEvidenceTemplates"));
  ok(script.includes("replace-with-64-character-sha256"));
});

test("rendered export audit exposes structured manual sign-off workflow", () => {
  const script = readFileSync("scripts/check-rendered-export-audit.mjs", "utf8");

  ok(script.includes("visual-review-signoff.template.json"));
  ok(script.includes("NEDITOR_RENDERED_EXPORT_SIGNOFF"));
  ok(script.includes("--validate-signoff-only"));
  ok(script.includes("NEDITOR_RENDERED_EXPORT_VALIDATE_EXISTING"));
  ok(script.includes("validateExistingSignoff"));
  ok(script.includes("Rendered export sign-off validated against existing artifacts"));
  ok(script.includes("neditor.rendered-export.visual-signoff.v1"));
  ok(script.includes("appVersion"));
  ok(script.includes("sourceCommit"));
  ok(script.includes("sourceTreeClean"));
  ok(script.includes("sourceTreeClean = gitTreeClean()"));
  ok(script.includes("completed sign-off appVersion must match package.json version"));
  ok(script.includes("completed sign-off sourceCommit must match current git commit"));
  ok(script.includes("completed sign-off sourceTreeClean must be true"));
  ok(script.includes("collectHumanSignoffEvidence"));
  ok(script.includes("collectAutomatedVisualReviewEvidence"));
  ok(script.includes("validateCompletedSignoff"));
  ok(script.includes("validateSignedArtifactIdentity"));
  ok(script.includes("reviewer.reviewedAt must be an ISO timestamp"));
  ok(script.includes("reviewer.nativeViewers"));
  ok(script.includes("sha256 must match current audit artifact"));
  ok(script.includes("bytes must match current audit artifact"));
  ok(script.includes("collectOfficePreviewProof"));
  ok(script.includes('page.locator("body").screenshot'));
  ok(script.includes("pdftocairo"));
  ok(script.includes("renderPdfCairoPage"));
  ok(script.includes("automated-visual-review.json"));
  ok(script.includes('"automated-reviewed"'));
  ok(script.includes("office-preview-docx"));
  ok(script.includes("office-preview-pptx"));
  ok(script.includes("Office preview screenshots"));
  ok(script.includes("visualEvidence?.officePreview"));
  ok(script.includes("automatedVisualReview"));
  ok(script.includes('"pending-human-review"'));
  ok(script.includes('"human-reviewed"'));
  ok(script.includes("allPrimaryArtifactsReviewed"));
  ok(script.includes("allReviewCasesReviewed"));
  ok(script.includes("allChecklistItemsReviewed"));
});

test("desktop WebDriver harness covers native settings and export workflows", () => {
  const script = readFileSync("scripts/run-tauri-webdriver.mjs", "utf8");

  ok(script.includes("assertDirtyTitleWorkflow(session)"));
  ok(script.includes("assertOutlineModeWorkflow(session)"));
  ok(script.includes("assertFileSaveOpenWorkflow(session)"));
  ok(script.includes("assertRenameDuplicateRevealWorkflow(session)"));
  ok(script.includes("assertExportReadinessWorkflow(session)"));
  ok(script.includes("assertHtmlExportWriteWorkflow(session)"));
  ok(script.includes("assertPreferenceWorkflow(session, originalPreferences)"));
  ok(script.includes("NEDITOR_DESKTOP_WORKFLOW_SMOKE_REPORT"));
  ok(script.includes("native-workflow-file.md"));
  ok(script.includes("native-workflow-renamed.md"));
  ok(script.includes("native-workflow-duplicate.md"));
  ok(script.includes("activePath || \"\").replace(/\\\\/g, \"/\").includes(\"native-workflow-file.md\")"));
  ok(script.includes("activePath || \"\").replace(/\\\\/g, \"/\").includes(\"native-workflow-renamed.md\")"));
  ok(script.includes("activateDocumentTabByPath(session, \"native-workflow-duplicate.md\")"));
  ok(script.includes("data-document-path"));
  ok(script.includes("native-workflow-export.html"));
  ok(script.includes("desktop WebDriver edits document structure in outline mode"));
  ok(script.includes("outlineArtifacts"));
  ok(script.includes("outlineModeEvidenceScript"));
  ok(script.includes("editorDocumentTextFunction"));
  ok(script.includes("window.__NEDITOR_DESKTOP_WORKFLOW__?.activeDocumentText?.()"));
  ok(script.includes("new dirty document before reopening saved file"));
  ok(script.includes("String(value?.tab || \"\").includes(\"Market Entry Report\")"));
  ok(script.includes("dataTablePreserved"));
  ok(script.includes("sourceGovernancePreserved"));
  ok(script.includes("Source Governance"));
  ok(script.includes("New subsection"));
  ok(script.includes("desktop WebDriver saves and reopens real Markdown file through dialog-free smoke path"));
  ok(script.includes("desktop WebDriver renames, duplicates, and exposes reveal affordance for real Markdown files"));
  ok(script.includes("desktop WebDriver writes HTML export through dialog-free smoke path"));
  ok(script.includes("desktop WebDriver Markdown file did not preserve document content"));
  ok(script.includes("document.querySelector('.sidebar pre')"));
  ok(script.includes('"export_target": "html"'));
  ok(script.includes("manifest.output_hash"));
  ok(script.includes("workflowPlan: webdriverWorkflowPlan"));
  ok(script.includes("appVersion: packageJson.version"));
  ok(script.includes("sourceCommit: gitCommit()"));
  ok(script.includes("sourceTreeClean: gitTreeClean()"));
  ok(script.includes("allowedDesktopWorkflowDirtyEntry"));
  ok(script.includes('["src-tauri/Cargo.lock", "src-tauri/Cargo.toml"].includes(path)'));
  ok(script.includes("desktop preferences apply in packaged WebDriver session"));
  ok(script.includes("Official Tauri WebDriver currently supports desktop automation on Windows and Linux only"));
  ok(script.includes("collectMacosNativeProof"));
  ok(script.includes("fallbackProof"));
  ok(script.includes("native-command-report.json"));
  ok(script.includes("native smoke report is older than the desktop binary"));
  ok(script.includes("freshForBinary"));
  ok(script.includes("native launch did not survive the bounded smoke window"));
  ok(script.includes("native workflow rendered outline mode structure only"));
  ok(script.includes("native workflow navigated outline heading to source"));
  ok(script.includes("outlineModeTitles"));
  ok(script.includes("native workflow exported html from native menu command"));
  ok(script.includes("native workflow restored workspace tabs with active pinned and scroll state"));
  const app = readFileSync("src/App.vue", "utf8");
  ok(app.includes("__NEDITOR_DESKTOP_WORKFLOW__"));
  ok(app.includes("desktop_workflow_smoke_enabled"));
  ok(app.includes(":data-document-path=\"document.path || ''\""));
  ok(app.includes("syncingEditorFromStore"));
  ok(app.includes("syncEditorViewFromActiveDocument"));
  ok(app.includes("previewTextCommit.cancel();\n  store.updateText(lines.join"));
  ok(app.includes("const text = editorView?.state.doc.toString() ?? active.value.text"));
  ok(app.includes("previewTextCommit.cancel();\n  store.updateText(`${before}${prefix}${block}${suffix}${after}`);"));
  ok(app.includes("void nextTick(() => syncEditorViewFromActiveDocument())"));
  ok(app.includes("[\"split\", \"source\", \"focus\"].includes(mode)"));
});

test("desktop launch smoke records native UI workbench surfaces", () => {
  const app = readFileSync("src/App.vue", "utf8");
  const rust = readFileSync("src-tauri/src/lib.rs", "utf8");
  const smoke = readFileSync("scripts/check-desktop-smoke.mjs", "utf8");

  ok(app.includes("write_desktop_ui_smoke_report"));
  ok(app.includes("desktop_workflow_smoke_autorun_enabled"));
  ok(app.includes("write_desktop_workflow_smoke_report"));
  ok(app.includes("desktop_workflow_smoke_file_path"));
  ok(app.includes("desktop_workflow_smoke_named_path"));
  ok(app.includes("desktopWorkflowSmokeMarkdownPath"));
  ok(app.includes("desktopWorkflowSmokeNamedMarkdownPath"));
  ok(app.includes("desktop_workflow_smoke_export_path"));
  ok(app.includes("emit_desktop_workflow_smoke_menu_command"));
  ok(app.includes("native workflow saved document to real file"));
  ok(app.includes("native workflow save cleared native title"));
  ok(app.includes("native workflow opened saved real file"));
  ok(app.includes("native workflow dirtied native title for opened real file"));
  ok(app.includes("native workflow reverted saved real file"));
  ok(app.includes("native workflow revert cleared native title"));
  ok(app.includes("collectNativeSnapshotEvidence"));
  ok(app.includes("native workflow created and listed app-data snapshot"));
  ok(app.includes("native workflow restored app-data snapshot"));
  ok(app.includes("native workflow created and listed project-local snapshot"));
  ok(app.includes("native workflow restored project-local snapshot"));
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
  ok(app.includes("collectNativeExportProfileEvidence"));
  ok(app.includes("native workflow saved export profile"));
  ok(app.includes("native workflow applied export profile"));
  ok(app.includes("native workflow reloaded export profile from settings store"));
  ok(app.includes("collectNativeModeEvidence"));
  ok(app.includes("[\"split\", \"source\", \"preview\", \"focus\", \"outline\", \"export\", \"review\", \"presentation\"]"));
  ok(app.includes("native workflow rendered outline mode structure only"));
  ok(app.includes("outlineTitles"));
  ok(app.includes("native workflow rendered export mode preview content"));
  ok(app.includes("native workflow rendered review mode governance content"));
  ok(app.includes("native workflow rendered presentation outline content"));
  ok(app.includes("collectNativeEditorErgonomicsEvidence"));
  ok(app.includes("native workflow reported editor word statistics"));
  ok(app.includes("native workflow exposed spellcheck editor attributes"));
  ok(app.includes("native workflow rendered line numbers word wrap and folding gutter"));
  ok(app.includes("native workflow opened editor search panel"));
  ok(app.includes("native workflow replaced editor search target"));
  ok(app.includes("native workflow opened Docs Live from native writing tools menu"));
  ok(app.includes("native workflow generated Docs Live section draft from native writing tools menu"));
  ok(app.includes("native workflow applied Docs Live section draft for review"));
  ok(app.includes("native workflow continued markdown list in editor"));
  ok(app.includes("native workflow inserted paired bracket in editor"));
  ok(app.includes("native workflow edited multiple cursors in editor"));
  ok(app.includes("collectNativeOutlineNavigationEvidence"));
  ok(app.includes("native workflow navigated outline heading to source"));
  ok(smoke.includes("native workflow report did not include editor ergonomics evidence"));
  ok(smoke.includes("native workflow report did not include outline navigation evidence"));
  ok(smoke.includes("native workflow report did not include rendered outline-mode structure"));
  ok(smoke.includes("native workflow report did not include rendered export-mode content"));
  ok(smoke.includes("native workflow report did not include rendered review-mode governance content"));
  ok(smoke.includes("native workflow report did not include rendered presentation outline content"));
  ok(smoke.includes("nativeMenuCommandEvidence.docsLive?.open !== true"));
  ok(smoke.includes("nativeMenuCommandEvidence.docsLive?.generated?.workflow !== true"));
  ok(smoke.includes("nativeMenuCommandEvidence.docsLive?.applied?.hasDraftingPlan !== true"));
  ok(app.includes("collectNativeThemeAccessibilityEvidence"));
  ok(app.includes("native workflow applied high contrast attributes and colors"));
  ok(app.includes("native workflow applied preview theme and typography"));
  ok(app.includes("commandLabels"));
  ok(app.includes("#document-workspace"));
  ok(app.includes("#live-preview"));
  ok(app.includes("previewLabel"));
  ok(rust.includes("fn write_desktop_ui_smoke_report"));
  ok(rust.includes("fn desktop_workflow_smoke_enabled"));
  ok(rust.includes("fn desktop_workflow_smoke_autorun_enabled"));
  ok(rust.includes("fn desktop_workflow_smoke_named_path"));
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
  ok(smoke.includes("native workflow report did not include app-data snapshot restore evidence"));
  ok(smoke.includes("native workflow report did not include project-local snapshot restore evidence"));
  ok(smoke.includes("native workflow report did not include export profile persistence evidence"));
  ok(smoke.includes("native workflow saved Markdown file was not written"));
  ok(smoke.includes("native workflow included watcher file was not written"));
  ok(smoke.includes("native workflow local conflict copy was not written"));
  ok(smoke.includes("native workflow HTML export artifact was not written"));
  ok(smoke.includes("native-menu HTML export evidence"));
  ok(smoke.includes("native workflow report did not include theme/accessibility evidence"));
  ok(smoke.includes("native UI report did not include rendered preview identity or content"));
  ok(smoke.includes("status = \"limited\""));
});
