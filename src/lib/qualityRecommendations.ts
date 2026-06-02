import type { DocumentDiagnostic, SemanticDocument } from "../types.js";
import { countCitationTodoMarkers } from "./citationTodoPatterns.js";
import { frontMatterScalarValue } from "./frontMatter.js";
import { markdownFenceOpener, stripMarkdownFencedBlocks } from "./provenanceReview.js";

export type QualityRecommendationSeverity = "pass" | "improve" | "risk" | "blocker";

export interface QualityRecommendation {
  id: string;
  label: string;
  severity: QualityRecommendationSeverity;
  recommendation: string;
  action: string;
}

export interface QualityRecommendationInput {
  text: string;
  semantic?: Pick<SemanticDocument, "title" | "comments" | "ai_sources" | "ai_assisted_sections"> | null;
  diagnostics?: DocumentDiagnostic[] | null;
}

export interface QualityStepAssistance {
  id: string;
  label: string;
  suggestedAnswer: string;
  rationale: string;
  contextSignals: string[];
  actionLabel: string;
}

export interface QualityStepAssistanceInput {
  recommendations: QualityRecommendation[];
  documentTitle?: string | null;
  documentText?: string | null;
  exportTarget?: string | null;
  reviewNotes?: string | null;
}

// NOTE: These module-level singletons use the /g flag. They are safe with String.prototype.match()
// (which resets lastIndex before each call) but are NOT safe with .exec()-in-a-loop patterns,
// which rely on lastIndex state. Do not use these regexes with .exec() across iterations.
const PLACEHOLDER_RE = /\{\{[^}]+\}\}|\b(?:TODO|TBD|FIXME)\b/gi;
const CITATION_RE = /\[@[A-Za-z0-9_:.#$%&+?~/-]+\]/g;
const BIBLIOGRAPHY_LANGUAGES = new Set(["bibtex", "hayagriva", "bibliography"]);
const RAW_BIBLIOGRAPHY_ENTRY_RE = /^@(?:article|book|misc|techreport|inproceedings)\s*[{(]/i;
const HEADING_RE = /^#{1,4}\s+\S.+$/gm;
const GENERIC_AI_PHRASE_RE = /\b(?:leverage|robust|seamless|cutting-edge|world-class|game-changing|holistic|synergy)\b/gi;
const DEEP_RESEARCH_MARKER_RE = /provider:\s*NEditor Deep Research|^##\s+(?:Deep Research Evidence Log|Source Citation Index)\b/im;

export function buildQualityRecommendations(input: QualityRecommendationInput): QualityRecommendation[] {
  const text = input.text || "";
  const analysisText = stripMarkdownFencedBlocks(text);
  const semantic = input.semantic;
  const diagnostics = input.diagnostics || [];
  const recommendations: QualityRecommendation[] = [];
  const unresolved = (semantic?.comments || []).filter((comment) => comment.state !== "resolved").length;
  const aiPending = [...(semantic?.ai_sources || []), ...(semantic?.ai_assisted_sections || [])].filter((item) => item.status !== "human-reviewed").length;
  const placeholderCount = placeholderMarkerCount(analysisText);
  const citationCount = (analysisText.match(CITATION_RE) || []).length;
  const bibliographyPresent = hasBibliographyEvidence(text);
  const deepResearchDocument = DEEP_RESEARCH_MARKER_RE.test(text);
  const inlineCitationReviewText = deepResearchDocument
    ? documentBodyForInlineCitationReview(analysisText)
    : analysisText;
  const deepResearchBodyCitationCount = (inlineCitationReviewText.match(CITATION_RE) || []).length;
  const citationTodoCount = countCitationTodoMarkers(inlineCitationReviewText);
  const headings = (analysisText.match(HEADING_RE) || []).length;
  const longParagraphs = longParagraphCount(analysisText);
  const genericPhrases = analysisText.match(GENERIC_AI_PHRASE_RE) || [];
  const diagnosticErrors = diagnostics.filter((diagnostic) => diagnostic.severity === "error").length;
  const diagnosticWarnings = diagnostics.filter((diagnostic) => diagnostic.severity === "warning").length;
  const hasDocumentTitle = Boolean((semantic?.title || frontMatterScalarValue(text, "title") || firstHeading(analysisText)).trim());
  const layoutSignals = documentLayoutSignals(text);

  if (diagnosticErrors || diagnosticWarnings) {
    recommendations.push({
      id: "compiler-diagnostics",
      label: "Compiler diagnostics",
      severity: diagnosticErrors ? "blocker" : "risk",
      recommendation: `${diagnosticErrors} errors and ${diagnosticWarnings} warnings need review before export.`,
      action: "Open Diagnostics, fix blocking issues, then re-run QA and export readiness.",
    });
  }
  if (placeholderCount) {
    recommendations.push({
      id: "placeholders",
      label: "Unresolved placeholders",
      severity: "risk",
      recommendation: `${placeholderCount} placeholders or TODO markers remain in the document.`,
      action: "Resolve values, mark unknowns as explicit assumptions, or keep them in a reviewer handoff section.",
    });
  }
  if (citationCount && !bibliographyPresent) {
    recommendations.push({
      id: "citation-evidence",
      label: "Citation evidence",
      severity: "risk",
      recommendation: `${citationCount} citation marker(s) are present but no bibliography block is visible.`,
      action: "Insert bibliography entries or add citation TODOs for every unsupported source.",
    });
  }
  if (deepResearchDocument && bibliographyPresent && !deepResearchBodyCitationCount && !citationTodoCount) {
    recommendations.push({
      id: "deep-research-citation-grounding",
      label: "Deep Research citation grounding",
      severity: "risk",
      recommendation: "This Deep Research document has bibliography/index evidence, but no inline body citation markers or citation TODOs outside generated handoff sections.",
      action: "Add the provided `[@key]` markers next to supported claims, or mark unsupported claims with Citation TODO before review or export.",
    });
  }
  if (unresolved) {
    recommendations.push({
      id: "review-comments",
      label: "Review comments",
      severity: "risk",
      recommendation: `${unresolved} unresolved review comment(s) still need a decision.`,
      action: "Resolve comments, defer them with owner/date, or document why they are acceptable for this release.",
    });
  }
  if (aiPending) {
    recommendations.push({
      id: "ai-provenance",
      label: "AI provenance",
      severity: "risk",
      recommendation: `${aiPending} AI-assisted source or section marker(s) are not human reviewed.`,
      action: "Inspect the generated material, remove AI cruft, verify claims, and mark reviewed only after human sign-off.",
    });
  }
  if (!hasDocumentTitle) {
    recommendations.push({
      id: "document-identity",
      label: "Document identity",
      severity: "improve",
      recommendation: "No front matter title or first heading is visible, which weakens review, export naming, and handoff context.",
      action: "Add a precise title in front matter or as the first H1 before sending the document for review.",
    });
  }
  if (headings < 2) {
    recommendations.push({
      id: "structure",
      label: "Document structure",
      severity: "improve",
      recommendation: "The document has fewer than two headings, making navigation, outline review, and export structure weaker.",
      action: "Use Outline mode or Docs Live to add chapters, sections, subsections, and review checkpoints.",
    });
  }
  if (longParagraphs) {
    recommendations.push({
      id: "readability",
      label: "Readability",
      severity: "improve",
      recommendation: `${longParagraphs} long paragraph(s) may be hard for business reviewers to scan.`,
      action: "Split long paragraphs into shorter points, lists, examples, or decision tables.",
    });
  }
  if (genericPhrases.length) {
    recommendations.push({
      id: "humanization",
      label: "Humanization",
      severity: "improve",
      recommendation: `${uniqueLowercase(genericPhrases).slice(0, 5).join(", ")} may read as generic AI wording.`,
      action: "Replace broad adjectives with named facts, proof points, constraints, and concrete reader outcomes.",
    });
  }
  if (layoutSignals.wideTables && !layoutSignals.hasLandscapeSection) {
    recommendations.push({
      id: "layout-wide-tables",
      label: "Wide table layout",
      severity: "risk",
      recommendation: `${layoutSignals.wideTables} wide table(s) may overflow or become hard to read in PDF/DOCX exports.`,
      action: "Move wide tables, timelines, and compliance matrices into a wide landscape section before export.",
    });
  }
  if (layoutSignals.hasMultiColumnLayout && !layoutSignals.hasExplicitColumnGap) {
    recommendations.push({
      id: "layout-column-gutter",
      label: "Column gutter",
      severity: "improve",
      recommendation: "A multi-column layout is present without an explicit column gap, which can produce inconsistent visual density across outputs.",
      action: "Set `columnGap` or use the two-column/three-column layout presets so preview, PDF, and DOCX spacing stays intentional.",
    });
  }
  if (layoutSignals.hasDenseSectionBreak && !layoutSignals.hasSingleColumnResetAfterDenseSection) {
    recommendations.push({
      id: "layout-section-reset",
      label: "Section layout reset",
      severity: "improve",
      recommendation: "A dense columned or landscape section changes the flow but no later single-column portrait reset is visible.",
      action: "Insert a return-to-single-column layout section before normal narrative, appendices, or reviewer handoff content continues.",
    });
  }
  if (!recommendations.length) {
    recommendations.push({
      id: "qa-ready",
      label: "QA baseline",
      severity: "pass",
      recommendation: "No obvious deterministic QA blockers were found in structure, citations, placeholders, comments, or AI review state.",
      action: "Run export readiness and a human review pass before external distribution.",
    });
  }
  return recommendations;
}

export function buildQualityStepAssistance(input: QualityStepAssistanceInput): QualityStepAssistance[] {
  const recommendations = input.recommendations.length
    ? input.recommendations
    : [{ id: "qa-ready", label: "QA baseline", severity: "pass", recommendation: "No obvious deterministic QA blockers were found.", action: "Run export readiness and human review." } satisfies QualityRecommendation];
  const counts = qualityRecommendationCounts(recommendations);
  const activeFindings = recommendations.filter((item) => item.severity !== "pass");
  const title = (input.documentTitle || "").trim() || "current document";
  const exportTarget = (input.exportTarget || "").trim() || "review package";
  const wordTotal = wordCount(input.documentText || "");
  const reviewNoteWords = wordCount(input.reviewNotes || "");
  const topFindings = activeFindings.slice(0, 3).map((item) => item.label).join(", ") || "QA baseline";
  const hasFinding = (id: string) => recommendations.some((item) => item.id === id);
  const riskSignal = `${counts.blocker} blockers, ${counts.risk} risks, ${counts.improve} improvements`;
  const documentSignal = `${wordTotal} document words`;
  const notesSignal = reviewNoteWords ? `${reviewNoteWords} review-note words captured` : "No quality review notes captured yet";

  return [
    {
      id: "qa-triage",
      label: "Triage the review",
      suggestedAnswer: counts.blocker
        ? `Treat "${title}" as blocked until the compiler or structural blockers are resolved, then re-run QA before preparing the ${exportTarget}.`
        : counts.risk
          ? `Prioritize the ${counts.risk} risk item(s) in "${title}", document every decision, and keep the ${exportTarget} in review until evidence and owner handoffs are complete.`
          : `Use the QA baseline for "${title}" as a readiness signal, then run export readiness and a human review before sending the ${exportTarget}.`,
      rationale: counts.blocker
        ? "Blockers can invalidate exported artifacts, so the first AI-guided answer should stop distribution and focus the reviewer on repairs."
        : "Triage turns a flat QA list into a reviewer-safe sequence: risks first, improvements second, export last.",
      contextSignals: [riskSignal, `Top findings: ${topFindings}`, documentSignal],
      actionLabel: "Add triage guidance",
    },
    {
      id: "evidence-review",
      label: "Verify evidence",
      suggestedAnswer: hasFinding("citation-evidence") || hasFinding("placeholders") || hasFinding("review-comments")
        ? "Resolve placeholders, citation gaps, and open review comments by assigning an owner, evidence source, and accept/defer decision for each item before sign-off."
        : "Confirm that claims, citations, assumptions, and reviewer comments are still aligned with the latest document context before approving the draft.",
      rationale: "Business review depends on traceable evidence and explicit decisions, especially when a document is moving toward a public or client-facing export.",
      contextSignals: [
        hasFinding("citation-evidence") ? "Citation evidence gap detected" : "No citation-evidence gap detected",
        hasFinding("placeholders") ? "Placeholder cleanup required" : "No unresolved placeholder finding",
        hasFinding("review-comments") ? "Open review comments detected" : "No open review-comment finding",
      ],
      actionLabel: "Add evidence guidance",
    },
    {
      id: "humanization",
      label: "Improve voice and clarity",
      suggestedAnswer: hasFinding("humanization") || hasFinding("readability") || hasFinding("ai-provenance")
        ? "Replace generic AI wording with named facts, split dense passages for scanning, and require human sign-off for every AI-assisted section before reviewer handoff."
        : "Do a final voice pass for specificity, reader relevance, and plain-language clarity even when deterministic AI-wording checks are clean.",
      rationale: "The QA pass should not only catch defects; it should make the document sound accountable, specific, and review-ready.",
      contextSignals: [
        hasFinding("humanization") ? "Generic phrasing detected" : "No generic phrasing finding",
        hasFinding("readability") ? "Long-paragraph finding detected" : "No long-paragraph finding",
        hasFinding("ai-provenance") ? "AI provenance needs review" : "AI provenance appears reviewed or absent",
      ],
      actionLabel: "Add humanization guidance",
    },
    {
      id: "review-handoff",
      label: "Prepare reviewer handoff",
      suggestedAnswer: activeFindings.length
        ? `Create a reviewer handoff for "${title}" that lists unresolved QA decisions, owners, due dates, and the exact ${exportTarget} readiness gate that must be passed next.`
        : `Package "${title}" for review with the QA baseline, export-readiness evidence, and the final ${exportTarget} checklist. Add any reviewer-specific questions before distribution.`,
      rationale: "A strong handoff preserves accountability across QA, approval, export, and distribution instead of leaving reviewers to infer next steps.",
      contextSignals: [riskSignal, notesSignal, `Target: ${exportTarget}`],
      actionLabel: "Add handoff guidance",
    },
  ];
}

export function formatQualityRecommendationSummary(recommendations: QualityRecommendation[]) {
  const counts = qualityRecommendationCounts(recommendations);
  return `${counts.blocker} blockers, ${counts.risk} risks, ${counts.improve} improvements`;
}

export function qualityRecommendationMarkdown(recommendations: QualityRecommendation[], generatedAt = new Date().toISOString()) {
  const rows = recommendations
    .map((item) => `| ${markdownTableCell(item.label)} | ${item.severity} | ${markdownTableCell(item.recommendation)} | ${markdownTableCell(item.action)} |`)
    .join("\n");
  return [
    "## Quality Assurance and Improvement Report",
    "",
    `Generated: ${generatedAt}`,
    `Summary: ${formatQualityRecommendationSummary(recommendations)}`,
    "",
    "| Area | Severity | Recommendation | Action |",
    "| --- | --- | --- | --- |",
    rows,
    "",
  ].join("\n");
}

function qualityRecommendationCounts(recommendations: QualityRecommendation[]) {
  return recommendations.reduce<Record<QualityRecommendationSeverity, number>>(
    (acc, item) => {
      acc[item.severity] += 1;
      return acc;
    },
    { pass: 0, improve: 0, risk: 0, blocker: 0 },
  );
}

function longParagraphCount(text: string) {
  return text
    .split(/\n{2,}/)
    .filter((paragraph) => !paragraph.trim().startsWith("#") && paragraph.trim().split(/\s+/).length > 95).length;
}

function placeholderMarkerCount(text: string) {
  return (text.match(PLACEHOLDER_RE) || []).filter((marker) => !isDocumentControlDirective(marker)).length;
}

function isDocumentControlDirective(marker: string) {
  return /^\{\{\s*(?:section-break|page-break|include|slide)\b/i.test(marker);
}

function documentLayoutSignals(text: string) {
  const layoutText = stripFencedBlocksExceptLayout(text);
  const lines = layoutText.split(/\r?\n/);
  const sectionBreaks = lines
    .map((line, index) => ({ line, index }))
    .filter(({ line }) => /\{\{\s*section-break\b/i.test(line));
  const denseSectionBreakIndexes = sectionBreaks
    .filter(({ line }) => {
      const columns = Number(line.match(/\bcolumns\s*=\s*(\d+)/i)?.[1] || "0");
      return columns > 1 || /\borientation\s*=\s*landscape\b/i.test(line);
    })
    .map(({ index }) => index);
  const singleColumnResetIndexes = sectionBreaks
    .filter(({ line }) => {
      const columns = Number(line.match(/\bcolumns\s*=\s*(\d+)/i)?.[1] || "0");
      return columns === 1 && (!/\borientation\s*=/i.test(line) || /\borientation\s*=\s*portrait\b/i.test(line));
    })
    .map(({ index }) => index);

  return {
    hasMultiColumnLayout: /\bcolumns\s*[:=]\s*[2-9]\b/i.test(layoutText),
    hasExplicitColumnGap: /\b(?:columnGap|column-gap|column_gap|gutter|columnGutter|column_gutter)\s*[:=]\s*["']?[^\s}"']+/i.test(layoutText),
    hasLandscapeSection: /\{\{\s*section-break\b[^}]*\borientation\s*=\s*landscape\b/i.test(layoutText),
    hasDenseSectionBreak: denseSectionBreakIndexes.length > 0,
    hasSingleColumnResetAfterDenseSection: denseSectionBreakIndexes.some((index) => singleColumnResetIndexes.some((resetIndex) => resetIndex > index)),
    wideTables: wideMarkdownTableCount(layoutText),
  };
}

function stripFencedBlocksExceptLayout(text: string) {
  const kept: string[] = [];
  let fenceMarker = "";
  let keepingFence = false;
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trimStart();
    if (fenceMarker) {
      if (keepingFence) kept.push(line);
      if (trimmed.startsWith(fenceMarker)) {
        fenceMarker = "";
        keepingFence = false;
      }
      continue;
    }
    const opener = markdownFenceOpener(line);
    if (opener) {
      fenceMarker = opener.marker;
      keepingFence = opener.language === "layout";
      if (keepingFence) kept.push(line);
      continue;
    }
    kept.push(line);
  }
  return kept.join("\n");
}

/** Counts wide markdown tables. Accepts pre-stripped layout text (fenced blocks already removed). */
function wideMarkdownTableCount(layoutText: string) {
  const lines = layoutText.split(/\r?\n/);
  let count = 0;
  for (let index = 0; index < lines.length - 1; index += 1) {
    const headerCells = markdownTableCellCount(lines[index]);
    if (headerCells < 5) continue;
    const separatorCells = markdownTableSeparatorCellCount(lines[index + 1]);
    if (separatorCells >= headerCells) count += 1;
  }
  return count;
}

function markdownTableCellCount(line: string) {
  const trimmed = line.trim();
  if (!trimmed.includes("|")) return 0;
  const withoutOuterPipes = trimmed.replace(/^\|/, "").replace(/\|$/, "");
  return splitMarkdownTableCells(withoutOuterPipes).filter((cell) => cell.trim()).length;
}

function markdownTableSeparatorCellCount(line: string) {
  const trimmed = line.trim();
  if (!trimmed.includes("|")) return 0;
  const withoutOuterPipes = trimmed.replace(/^\|/, "").replace(/\|$/, "");
  const cells = splitMarkdownTableCells(withoutOuterPipes).map((cell) => cell.trim());
  return cells.every((cell) => /^:?-{3,}:?$/.test(cell)) ? cells.length : 0;
}

function splitMarkdownTableCells(row: string) {
  const cells: string[] = [];
  let current = "";
  let escaped = false;
  for (const char of row) {
    if (char === "|" && !escaped) {
      cells.push(current);
      current = "";
      continue;
    }
    current += char;
    escaped = char === "\\" && !escaped;
    if (char !== "\\") escaped = false;
  }
  cells.push(current);
  return cells;
}

function firstHeading(text: string) {
  return text.match(/^#\s+(.+)$/m)?.[1] || "";
}

function documentBodyForInlineCitationReview(text: string) {
  return removeMarkdownSections(text, [
    "Source Citation Index",
    "Source Citation Index Addendum",
    "Bibliography",
    "Deep Research Evidence Log",
    "Deep Research Evidence Log Addendum",
    "Source Library Audit",
    "Source Library Audit Addendum",
    "Quality Assurance & Review Handoff",
    "Quality Assurance and Improvement Report",
  ]);
}

function removeMarkdownSections(text: string, headings: string[]) {
  const targets = new Set(headings.map(normalizeHeadingText));
  const kept: string[] = [];
  let skippedLevel = 0;
  for (const line of text.split(/\r?\n/)) {
    const heading = line.match(/^(#{1,6})\s+(.+?)\s*#*\s*$/);
    if (heading) {
      const level = heading[1].length;
      const title = normalizeHeadingText(heading[2]);
      if (skippedLevel && level <= skippedLevel) skippedLevel = 0;
      if (targets.has(title) || isGeneratedHandoffHeading(title)) {
        skippedLevel = level;
        continue;
      }
    }
    if (!skippedLevel) kept.push(line);
  }
  return kept.join("\n");
}

function normalizeHeadingText(value: string) {
  return value.trim().toLowerCase();
}

function isGeneratedHandoffHeading(value: string) {
  return /^iteration\s+\d+\s*:/.test(value);
}

function wordCount(value: string) {
  return value.trim() ? value.trim().split(/\s+/).length : 0;
}

function hasBibliographyEvidence(text: string) {
  let fenceMarker = "";
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trimStart();
    if (fenceMarker) {
      if (trimmed.startsWith(fenceMarker)) fenceMarker = "";
      continue;
    }
    const opener = markdownFenceOpener(line);
    if (opener) {
      if (BIBLIOGRAPHY_LANGUAGES.has(opener.language)) return true;
      fenceMarker = opener.marker;
      continue;
    }
    if (line.includes("[BIBLIOGRAPHY]") || RAW_BIBLIOGRAPHY_ENTRY_RE.test(trimmed)) return true;
  }
  return false;
}

function uniqueLowercase(values: string[]) {
  return Array.from(new Set(values.map((value) => value.toLowerCase())));
}

function markdownTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}
