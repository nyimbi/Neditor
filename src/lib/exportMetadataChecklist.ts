import type { ExportDefaults, ExportTarget } from "./workspacePersistence.js";
import { frontMatterListValues, frontMatterScalarValue } from "./frontMatter.js";

export type ExportMetadataChecklistStatus = "complete" | "missing" | "invalid" | "optional";

export interface ExportMetadataChecklistItem {
  id: string;
  label: string;
  status: ExportMetadataChecklistStatus;
  detail: string;
  suggestion: string;
}

export interface ExportMetadataChecklistInput {
  target: ExportTarget;
  text?: string;
  metadata?: Record<string, unknown> | null;
  exportDefaults?: Partial<
    Pick<
      ExportDefaults,
      | "htmlDescription"
      | "canonicalUrl"
      | "htmlLanguage"
      | "includeManifest"
      | "includeComments"
      | "includeProvenance"
      | "includeGlossary"
      | "layoutPreset"
    >
  >;
  outlineCount?: number;
}

export interface ExportReadinessSummaryInput {
  ready?: boolean;
  error_count?: number;
  warning_count?: number;
  info_count?: number;
}

export interface ExportStepAssistanceInput extends ExportMetadataChecklistInput {
  checklist?: ExportMetadataChecklistItem[];
  readiness?: ExportReadinessSummaryInput | null;
  notes?: string;
}

export interface ExportStepAssistance {
  stepId: string;
  stepLabel: string;
  suggestedAnswer: string;
  rationale: string;
  contextSignals: string[];
  actionLabel: string;
}

export const PUBLIC_METADATA_TARGETS = new Set<ExportTarget>(["html", "blog", "substack", "epub"]);
export const DISTRIBUTION_APPROVAL_TARGETS = new Set<ExportTarget>(["pptx", "blog", "substack", "google-docs", "epub"]);
export const EXPORT_TARGET_LABELS: Record<string, string> = {
  html: "HTML",
  pdf: "PDF",
  docx: "DOCX",
  pptx: "PPTX",
  "markdown-bundle": "Markdown bundle",
  markdown: "Markdown bundle",
  blog: "blog package",
  substack: "Substack package",
  latex: "LaTeX",
  "google-docs": "Google Docs package",
  epub: "EPUB ebook",
};

export function buildExportMetadataChecklist(input: ExportMetadataChecklistInput): ExportMetadataChecklistItem[] {
  const target = input.target;
  const metadata = input.metadata || {};
  const text = input.text || "";
  const items: ExportMetadataChecklistItem[] = [];

  if (DISTRIBUTION_APPROVAL_TARGETS.has(target)) {
    const status = metadataText(metadata, text, ["status"]) || "draft";
    const approver = metadataText(metadata, text, ["approvedBy", "reviewer"]);
    const approvedAt = metadataText(metadata, text, ["approvedAt"]);
    const owner = metadataText(metadata, text, ["owner"]);
    const releaseTarget = metadataText(metadata, text, ["releaseTarget"]);
    const sourceConfidence = metadataText(metadata, text, ["sourceConfidence", "source_confidence"]);
    const unresolvedComments = unresolvedReviewCommentCount(text);
    const missing = [
      !["approved", "published"].includes(status) ? "approved or published status" : "",
      !approver ? "approvedBy or reviewer" : "",
      !approvedAt ? "approvedAt" : "",
      !owner ? "owner" : "",
      !releaseTarget ? "releaseTarget" : "",
      !isReleaseReadySourceConfidence(sourceConfidence) ? "sourceConfidence" : "",
      unresolvedComments ? "resolved comments" : "",
    ].filter(Boolean);
    items.push({
      id: "release-approval",
      label: "Release approval",
      status: missing.length ? "missing" : "complete",
      detail: missing.length
        ? `Missing ${missing.join(", ")}${unresolvedComments ? ` (${unresolvedComments} unresolved)` : ""}.`
        : `Approved for ${releaseTarget || EXPORT_TARGET_LABELS[target]} with ${sourceConfidence} source confidence.`,
      suggestion: "Use the Review panel or Add suggested metadata before external distribution, then resolve comments and confirm source confidence.",
    });
  }

  if (target === "html" || target === "blog" || target === "substack") {
    const description = exportOptionOrMetadata(input.exportDefaults, "htmlDescription", metadata, text, [
      "description",
      "summary",
      "subtitle",
      "excerpt",
    ]);
    items.push({
      id: "public-description",
      label: target === "html" ? "Search/social description" : "Publishing preview",
      status: description ? "complete" : target === "html" ? "optional" : "missing",
      detail: description ? trimForChecklist(description) : "No description, summary, subtitle, or excerpt is available.",
      suggestion: target === "html" ? "Add a description for richer link previews." : "Add a description or excerpt before publishing.",
    });

    const canonical = exportOptionOrMetadata(input.exportDefaults, "canonicalUrl", metadata, text, ["canonicalUrl", "canonical_url"]);
    items.push({
      id: "canonical-url",
      label: "Canonical URL",
      status: canonical ? (isPublicHttpUrl(canonical) ? "complete" : "invalid") : "optional",
      detail: canonical ? canonical : "No canonical URL set yet.",
      suggestion: "Use the final public http:// or https:// URL when one exists.",
    });
  }

  if (target === "blog" || target === "substack") {
    const tags = metadataList(metadata, text, ["tags", "keywords"]);
    items.push({
      id: "publishing-tags",
      label: "Tags and keywords",
      status: tags.length ? "complete" : "missing",
      detail: tags.length ? tags.join(", ") : "No tags or keywords are present.",
      suggestion: "Add tags or keywords so publishing packages are easier to find and archive.",
    });
  }

  if (PUBLIC_METADATA_TARGETS.has(target)) {
    const language = exportOptionOrMetadata(input.exportDefaults, "htmlLanguage", metadata, text, ["language", "lang", "locale"]);
    items.push({
      id: "language",
      label: target === "epub" ? "Reader language" : "Document language",
      status: language ? (isLanguageTag(language) ? "complete" : "invalid") : "optional",
      detail: language ? language : "Will default to en.",
      suggestion: "Use a language tag such as en, en-US, fr, or pt-BR.",
    });
  }

  if (target === "epub") {
    const creator = metadataText(metadata, text, ["author", "approvedBy", "reviewer"]);
    const outlineCount = input.outlineCount || 0;
    items.push({
      id: "epub-creator",
      label: "Ebook creator",
      status: creator ? "complete" : "missing",
      detail: creator || "No author, approver, or reviewer is present.",
      suggestion: "Add author, approvedBy, or reviewer so reader metadata is clear.",
    });
    items.push({
      id: "epub-outline",
      label: "Reader outline",
      status: outlineCount ? "complete" : "missing",
      detail: outlineCount ? `${outlineCount} heading entries available.` : "No heading outline is available.",
      suggestion: "Add chapter or section headings before exporting an ebook.",
    });
  }

  return items;
}

export function formatExportMetadataChecklistSummary(items: ExportMetadataChecklistItem[]) {
  const counts = items.reduce<Record<ExportMetadataChecklistStatus, number>>(
    (acc, item) => {
      acc[item.status] += 1;
      return acc;
    },
    { complete: 0, missing: 0, invalid: 0, optional: 0 },
  );
  return `${counts.complete} complete, ${counts.missing} missing, ${counts.invalid} invalid, ${counts.optional} optional`;
}

export function exportMetadataChecklistHelp(target: ExportTarget) {
  const label = EXPORT_TARGET_LABELS[target] || target;
  return `Preflight metadata for ${label} before writing files. Prepare for export still runs the authoritative backend validation.`;
}

export function buildExportStepAssistance(input: ExportStepAssistanceInput): ExportStepAssistance[] {
  const targetLabel = EXPORT_TARGET_LABELS[input.target] || input.target;
  const checklist = input.checklist || buildExportMetadataChecklist(input);
  const missing = checklist.filter((item) => item.status === "missing" || item.status === "invalid");
  const optional = checklist.filter((item) => item.status === "optional");
  const readiness = input.readiness || null;
  const notesWords = (input.notes || "").split(/\s+/).filter(Boolean).length;
  const options = input.exportDefaults || {};
  const baseSignals = [
    `Target: ${targetLabel}`,
    `Metadata checklist: ${formatExportMetadataChecklistSummary(checklist)}`,
    readiness ? `Backend readiness: ${readiness.ready ? "ready" : "needs attention"}` : "Backend readiness not run",
    readiness ? `Diagnostics: ${readiness.error_count || 0} errors, ${readiness.warning_count || 0} warnings, ${readiness.info_count || 0} info` : "Diagnostics not available yet",
    `Export notes words: ${notesWords}`,
    `Outline entries: ${input.outlineCount || 0}`,
  ];
  const missingLabels = missing.map((item) => item.label).join(", ") || "none";
  const auditOptions = [
    options.includeManifest ? "manifest" : "",
    options.includeComments ? "comments" : "",
    options.includeProvenance ? "AI provenance" : "",
    options.includeGlossary ? "glossary" : "",
  ].filter(Boolean);
  const metadataGuidance = missing.length
    ? `Fix the blocking ${targetLabel} metadata first: ${missingLabels}. Use Add suggested metadata where appropriate, then replace TODO values before delivery.`
    : `The ${targetLabel} metadata checklist has no blocking missing or invalid items. Review optional items (${optional.map((item) => item.label).join(", ") || "none"}) before final delivery.`;
  const readinessGuidance = readiness
    ? readiness.ready
      ? `Backend export readiness is currently ready for ${targetLabel}. Keep the manifest and readiness diagnostics with the review record before writing the final artifact.`
      : `Backend export readiness needs attention: resolve ${readiness.error_count || 0} error(s), ${readiness.warning_count || 0} warning(s), and review ${readiness.info_count || 0} info item(s), then rerun Prepare for export.`
    : `Run Prepare for export for ${targetLabel} after metadata review so backend diagnostics, manifests, references, transforms, and layout evidence are checked before files are written.`;

  return [
    {
      stepId: "target-metadata",
      stepLabel: "Target metadata",
      actionLabel: "Use metadata guidance",
      suggestedAnswer: metadataGuidance,
      rationale: "Target-specific metadata controls whether the package is acceptable for publishing, client delivery, Google Docs handoff, EPUB readers, or review archives.",
      contextSignals: [...baseSignals, `Missing/invalid metadata: ${missingLabels}`],
    },
    {
      stepId: "readiness-diagnostics",
      stepLabel: "Readiness diagnostics",
      actionLabel: "Use readiness guidance",
      suggestedAnswer: readinessGuidance,
      rationale: "The frontend checklist is helpful, but backend export readiness is the authoritative gate before artifacts are written.",
      contextSignals: baseSignals,
    },
    {
      stepId: "artifact-evidence",
      stepLabel: "Artifact evidence",
      actionLabel: "Use evidence guidance",
      suggestedAnswer: [
        `Package ${targetLabel} with ${auditOptions.length ? auditOptions.join(", ") : "the minimal selected options"}.`,
        options.layoutPreset ? `Use the ${options.layoutPreset} layout preset.` : "Choose a layout preset before final export.",
        "Keep output path, manifest path, source hash, readiness diagnostics, and any manual review notes with the delivery record.",
      ].join(" "),
      rationale: "Production exports need artifact-level evidence, not only a successful button click.",
      contextSignals: [...baseSignals, `Audit options: ${auditOptions.join(", ") || "none"}`],
    },
  ];
}

function exportOptionOrMetadata(
  exportDefaults: ExportMetadataChecklistInput["exportDefaults"],
  optionKey: keyof NonNullable<ExportMetadataChecklistInput["exportDefaults"]>,
  metadata: Record<string, unknown>,
  text: string,
  metadataKeys: string[],
) {
  const option = exportDefaults?.[optionKey];
  if (typeof option === "string" && option.trim()) return option.trim();
  return metadataText(metadata, text, metadataKeys);
}

function metadataText(metadata: Record<string, unknown>, text: string, keys: string[]) {
  for (const key of keys) {
    const value = metadataValueAtPath(metadata, key);
    if (typeof value === "string" && value.trim()) return value.trim();
    if (typeof value === "number" || typeof value === "boolean") return String(value);
    const frontMatterValue = frontMatterScalarValue(text, key);
    if (frontMatterValue.trim()) return frontMatterValue.trim();
  }
  return "";
}

function metadataList(metadata: Record<string, unknown>, text: string, keys: string[]) {
  for (const key of keys) {
    const value = metadataValueAtPath(metadata, key);
    const values = Array.isArray(value)
      ? value.map((item) => String(item).trim()).filter(Boolean)
      : typeof value === "string"
        ? value.split(",").map((item) => item.trim()).filter(Boolean)
        : [];
    if (values.length) return Array.from(new Set(values));
    const frontMatterValues = frontMatterListValues(text, key);
    if (frontMatterValues.length) return frontMatterValues;
  }
  return [];
}

function metadataValueAtPath(metadata: Record<string, unknown>, key: string): unknown {
  if (Object.prototype.hasOwnProperty.call(metadata, key)) return metadata[key];
  return key.split(".").reduce<unknown>((current, part) => {
    if (!current || typeof current !== "object") return undefined;
    return (current as Record<string, unknown>)[part];
  }, metadata);
}

function isPublicHttpUrl(value: string) {
  try {
    const url = new URL(value.trim());
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

function isLanguageTag(value: string) {
  const trimmed = value.trim();
  return Boolean(trimmed) && trimmed.length <= 35 && trimmed.split("-").every((part) => Boolean(part) && part.length <= 8 && /^[a-z0-9]+$/i.test(part));
}

function isReleaseReadySourceConfidence(value: string) {
  const normalized = value.trim().toLowerCase();
  return Boolean(normalized) && !["todo", "tbd", "unknown", "unverified", "needs-review", "needs review", "draft"].includes(normalized);
}

function unresolvedReviewCommentCount(text: string) {
  const matches = text.match(/<!--\s*comment:[\s\S]*?-->/gi) || [];
  return matches.filter((comment) => !/\bresolved\b/i.test(comment)).length;
}

function trimForChecklist(value: string) {
  const trimmed = value.trim();
  return trimmed.length > 96 ? `${trimmed.slice(0, 93)}...` : trimmed;
}
