import type { ExportDefaults, ExportTarget } from "./workspacePersistence.js";

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
  exportDefaults?: Partial<Pick<ExportDefaults, "htmlDescription" | "canonicalUrl" | "htmlLanguage">>;
  outlineCount?: number;
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
    const missing = [
      !["approved", "published"].includes(status) ? "approved or published status" : "",
      !approver ? "approvedBy or reviewer" : "",
      !approvedAt ? "approvedAt" : "",
      !owner ? "owner" : "",
      !releaseTarget ? "releaseTarget" : "",
    ].filter(Boolean);
    items.push({
      id: "release-approval",
      label: "Release approval",
      status: missing.length ? "missing" : "complete",
      detail: missing.length ? `Missing ${missing.join(", ")}.` : `Approved for ${releaseTarget || EXPORT_TARGET_LABELS[target]}.`,
      suggestion: "Use the Review panel or Add suggested metadata before external distribution.",
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

function frontMatterScalarValue(text: string, key: string) {
  const lines = frontMatterLines(text);
  if (!lines.length) return "";
  const line = lines.find((candidate) => candidate.trimStart().startsWith(`${key}:`));
  if (!line) return "";
  return line.split(":").slice(1).join(":").trim().replace(/^["']|["']$/g, "");
}

function frontMatterListValues(text: string, key: string) {
  const lines = frontMatterLines(text);
  if (!lines.length) return [];
  const startIndex = lines.findIndex((candidate) => candidate.trimStart().startsWith(`${key}:`));
  if (startIndex < 0) return [];
  const inlineValue = lines[startIndex].split(":").slice(1).join(":").trim();
  if (inlineValue.startsWith("[") && inlineValue.endsWith("]")) {
    return inlineValue
      .slice(1, -1)
      .split(",")
      .map((value) => value.trim().replace(/^["']|["']$/g, ""))
      .filter(Boolean);
  }
  const values: string[] = [];
  for (let index = startIndex + 1; index < lines.length; index += 1) {
    const match = lines[index].match(/^\s+-\s+(.+?)\s*$/);
    if (!match) break;
    values.push(match[1].trim().replace(/^["']|["']$/g, ""));
  }
  return Array.from(new Set(values));
}

function frontMatterLines(text: string) {
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  return match ? match[1].split(/\r?\n/) : [];
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

function trimForChecklist(value: string) {
  const trimmed = value.trim();
  return trimmed.length > 96 ? `${trimmed.slice(0, 93)}...` : trimmed;
}
