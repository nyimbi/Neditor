export type PrintPreviewPageSize = "A4" | "Letter";
export type PrintPreviewOrientation = "portrait" | "landscape";
export type PrintPreviewMargins = "normal" | "narrow" | "wide";

export interface PrintPreviewOptions {
  layoutPreset?: string;
  coverPage?: boolean;
  pageNumbers?: boolean;
}

export interface PrintPreviewSection {
  line: number;
  columns: number;
  pageSize: PrintPreviewPageSize;
  orientation: PrintPreviewOrientation;
  margins: PrintPreviewMargins;
  label: string;
}

export interface PrintPreviewReport {
  pageSize: PrintPreviewPageSize;
  orientation: PrintPreviewOrientation;
  margins: PrintPreviewMargins;
  columns: number;
  estimatedPages: number;
  wordCount: number;
  pageBreaks: number;
  sectionBreaks: PrintPreviewSection[];
  summary: string;
  warnings: string[];
}

const defaultWordsPerPageByPreset: Record<string, number> = {
  business: 520,
  compact: 700,
  presentation: 260,
};

export function buildPrintPreviewReport(markdown: string, options: PrintPreviewOptions = {}): PrintPreviewReport {
  const layoutBlock = frontMatterLayoutBlock(markdown);
  const pageSize = normalizePageSize(valueFromLayout(layoutBlock, "pageSize") || "A4");
  const orientation = normalizeOrientation(valueFromLayout(layoutBlock, "orientation") || "portrait");
  const margins = normalizeMargins(valueFromLayout(layoutBlock, "margins") || (options.layoutPreset === "compact" ? "narrow" : "normal"));
  const columns = normalizeColumns(valueFromLayout(layoutBlock, "columns") || "1");
  const wordCount = countPreviewWords(markdown);
  const pageBreaks = (markdown.match(/\{\{\s*page-break\s*\}\}/gi) || []).length;
  const sectionBreaks = extractSectionBreaks(markdown, { pageSize, orientation, margins, columns });
  const estimatedWordsPerPage = estimateWordsPerPage(options.layoutPreset || "business", margins, columns, orientation);
  const estimatedPages = Math.max(1, Math.ceil(wordCount / estimatedWordsPerPage) + pageBreaks + (options.coverPage ? 1 : 0));
  const warnings = printPreviewWarnings(markdown, { wordCount, estimatedPages, pageBreaks, sectionBreaks, columns, orientation, pageSize });
  return {
    pageSize,
    orientation,
    margins,
    columns,
    estimatedPages,
    wordCount,
    pageBreaks,
    sectionBreaks,
    warnings,
    summary: `${estimatedPages} estimated page${estimatedPages === 1 ? "" : "s"} | ${pageSize} ${orientation} | ${margins} margins | ${columns} column${columns === 1 ? "" : "s"}${options.pageNumbers ? " | page numbers" : ""}`,
  };
}

function countPreviewWords(markdown: string) {
  return markdown
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/~~~[\s\S]*?~~~/g, " ")
    .replace(/^---\n[\s\S]*?\n---(?:\n|$)/, " ")
    .replace(/\{\{[^}]+\}\}/g, " ")
    .split(/\s+/)
    .filter((word) => /[A-Za-z0-9]/.test(word)).length;
}

function estimateWordsPerPage(layoutPreset: string, margins: PrintPreviewMargins, columns: number, orientation: PrintPreviewOrientation) {
  const base = defaultWordsPerPageByPreset[layoutPreset] || defaultWordsPerPageByPreset.business;
  const marginFactor = margins === "narrow" ? 1.16 : margins === "wide" ? 0.82 : 1;
  const columnFactor = columns > 1 ? 0.9 : 1;
  const orientationFactor = orientation === "landscape" ? 1.12 : 1;
  return Math.max(180, Math.round(base * marginFactor * columnFactor * orientationFactor));
}

function extractSectionBreaks(markdown: string, defaults: Omit<PrintPreviewSection, "line" | "label">) {
  const sections: PrintPreviewSection[] = [];
  const lines = markdown.split(/\r?\n/);
  lines.forEach((line, index) => {
    if (!/\{\{\s*section-break\b/i.test(line)) return;
    sections.push({
      line: index + 1,
      columns: normalizeColumns(directiveValue(line, "columns") || String(defaults.columns)),
      pageSize: normalizePageSize(directiveValue(line, "pageSize") || defaults.pageSize),
      orientation: normalizeOrientation(directiveValue(line, "orientation") || defaults.orientation),
      margins: normalizeMargins(directiveValue(line, "margins") || defaults.margins),
      label: directiveValue(line, "section") || `Section ${sections.length + 1}`,
    });
  });
  return sections;
}

function printPreviewWarnings(markdown: string, report: Pick<PrintPreviewReport, "wordCount" | "estimatedPages" | "pageBreaks" | "sectionBreaks" | "columns" | "orientation" | "pageSize">) {
  const warnings: string[] = [];
  const wideTableLines = markdown.split(/\r?\n/).filter((line) => /^\|.*\|$/.test(line) && line.length > 110).length;
  if (wideTableLines && report.orientation === "portrait") warnings.push(`${wideTableLines} wide table line${wideTableLines === 1 ? "" : "s"} may need landscape or a wide section.`);
  if (report.columns > 1 && report.wordCount > 900) warnings.push("Multi-column flow is approximate; verify final PDF or DOCX pagination before sending.");
  if (report.sectionBreaks.length > 6) warnings.push("Many section breaks can create unexpected page flow; review each transition before export.");
  if (!report.pageBreaks && report.estimatedPages > 8) warnings.push("Long document has no explicit page breaks; add breaks before appendices or major parts if needed.");
  if (report.pageSize === "Letter" && /\bA4\b/i.test(markdown)) warnings.push("Document text mentions A4 while preview uses Letter.");
  return warnings;
}

function directiveValue(text: string, key: string) {
  const match = text.match(new RegExp(`\\b${key}\\s*=\\s*("[^"]+"|'[^']+'|[^\\s}]+)`, "i"));
  return match ? match[1].replace(/^["']|["']$/g, "") : "";
}

function frontMatterLayoutBlock(markdown: string) {
  const match = markdown.match(/^---\s*\n([\s\S]*?)\n---/);
  const body = match?.[1] || "";
  const layout = body.match(/^layout:\s*\n((?:\s{2,}.+\n?)+)/m);
  return layout?.[1] || "";
}

function valueFromLayout(layoutBlock: string, key: string) {
  const match = layoutBlock.match(new RegExp(`^\\s+${key}:\\s*(.+)$`, "im"));
  return match ? match[1].trim().replace(/^["']|["']$/g, "") : "";
}

function normalizePageSize(value: string): PrintPreviewPageSize {
  return /^letter$/i.test(value.trim()) ? "Letter" : "A4";
}

function normalizeOrientation(value: string): PrintPreviewOrientation {
  return /^landscape$/i.test(value.trim()) ? "landscape" : "portrait";
}

function normalizeMargins(value: string): PrintPreviewMargins {
  if (/^narrow$/i.test(value.trim())) return "narrow";
  if (/^wide$/i.test(value.trim())) return "wide";
  return "normal";
}

function normalizeColumns(value: string) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) return 1;
  return Math.max(1, Math.min(4, parsed));
}
