import type { DocumentDiagnostic, SemanticDocument } from "../types.js";
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

const PLACEHOLDER_RE = /\{\{[^}]+\}\}|\b(?:TODO|TBD|FIXME)\b/gi;
const CITATION_RE = /\[@[A-Za-z0-9_:.#$%&+?~/-]+\]/g;
const BIBLIOGRAPHY_LANGUAGES = new Set(["bibtex", "hayagriva", "bibliography"]);
const RAW_BIBLIOGRAPHY_ENTRY_RE = /^@(?:article|book|misc|techreport|inproceedings)\s*[{(]/i;
const HEADING_RE = /^#{1,4}\s+\S.+$/gm;
const GENERIC_AI_PHRASE_RE = /\b(?:leverage|robust|seamless|cutting-edge|world-class|game-changing|holistic|synergy)\b/gi;

export function buildQualityRecommendations(input: QualityRecommendationInput): QualityRecommendation[] {
  const text = input.text || "";
  const analysisText = stripMarkdownFencedBlocks(text);
  const semantic = input.semantic;
  const diagnostics = input.diagnostics || [];
  const recommendations: QualityRecommendation[] = [];
  const unresolved = (semantic?.comments || []).filter((comment) => comment.state !== "resolved").length;
  const aiPending = [...(semantic?.ai_sources || []), ...(semantic?.ai_assisted_sections || [])].filter((item) => item.status !== "human-reviewed").length;
  const placeholderCount = (analysisText.match(PLACEHOLDER_RE) || []).length;
  const citationCount = (analysisText.match(CITATION_RE) || []).length;
  const bibliographyPresent = hasBibliographyEvidence(text);
  const headings = (analysisText.match(HEADING_RE) || []).length;
  const longParagraphs = longParagraphCount(analysisText);
  const genericPhrases = analysisText.match(GENERIC_AI_PHRASE_RE) || [];
  const diagnosticErrors = diagnostics.filter((diagnostic) => diagnostic.severity === "error").length;
  const diagnosticWarnings = diagnostics.filter((diagnostic) => diagnostic.severity === "warning").length;
  const hasDocumentTitle = Boolean((semantic?.title || frontMatterScalarValue(text, "title") || firstHeading(analysisText)).trim());

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

function firstHeading(text: string) {
  return text.match(/^#\s+(.+)$/m)?.[1] || "";
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
