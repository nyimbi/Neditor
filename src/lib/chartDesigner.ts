export type ChartDesignerKind = "bar" | "line" | "horizontal-bar" | "scorecard";

export interface ChartDesignerKindOption {
  id: ChartDesignerKind;
  label: string;
  summary: string;
  bestFor: string[];
}

export interface ChartDesignerDraft {
  kind: ChartDesignerKind;
  title: string;
  subtitle: string;
  source: string;
  xField: string;
  yField: string;
  valueSuffix: string;
  target: string;
  targetLabel: string;
  showValues: boolean;
  paletteText: string;
  dataText: string;
}

interface ParsedChartData {
  headers: string[];
  rows: string[][];
}

export const chartDesignerKindOptions: ChartDesignerKindOption[] = [
  {
    id: "bar",
    label: "Vertical bar",
    summary: "Best for comparing a few business categories, KPIs, or plan-versus-actual values.",
    bestFor: ["KPIs", "scorecards", "category comparisons"],
  },
  {
    id: "line",
    label: "Trend line",
    summary: "Best for month-by-month, quarter-by-quarter, or milestone trends.",
    bestFor: ["revenue trends", "usage trends", "delivery progress"],
  },
  {
    id: "horizontal-bar",
    label: "Horizontal bar",
    summary: "Best for ranked lists, long account names, risks, and requirement comparisons.",
    bestFor: ["rankings", "risk lists", "long labels"],
  },
  {
    id: "scorecard",
    label: "Executive scorecard",
    summary: "Best for board-ready charts with subtitle, source note, target line, and value labels.",
    bestFor: ["board packs", "proposals", "executive reports"],
  },
];

const sampleDataByKind: Record<ChartDesignerKind, string> = {
  bar: ["Metric, Value", "Revenue, 125", "Margin, 61", "NPS, 48"].join("\n"),
  line: ["Month, Revenue", "Jan, 420", "Feb, 460", "Mar, 515", "Apr, 552"].join("\n"),
  "horizontal-bar": ["Account, Risk", "Very Long Enterprise Account, 72", "Growth Segment, 34", "Expansion Candidate, 18"].join("\n"),
  scorecard: ["Segment, Coverage", "Enterprise, 112", "Mid-market, 78", "SMB, 44"].join("\n"),
};

export function chartDesignerDefaultDraft(kind: ChartDesignerKind = "bar"): ChartDesignerDraft {
  const isScorecard = kind === "scorecard";
  return {
    kind,
    title: isScorecard ? "Executive pipeline coverage" : kind === "line" ? "Monthly revenue" : kind === "horizontal-bar" ? "Renewal risk by account" : "Quarterly KPI plan",
    subtitle: isScorecard ? "Weighted qualified pipeline by segment" : "",
    source: isScorecard ? "CRM export, May 2026" : "",
    xField: kind === "line" ? "Month" : kind === "horizontal-bar" ? "Account" : isScorecard ? "Segment" : "Metric",
    yField: kind === "line" ? "Revenue" : kind === "horizontal-bar" ? "Risk" : isScorecard ? "Coverage" : "Value",
    valueSuffix: kind === "horizontal-bar" || isScorecard ? "%" : "",
    target: kind === "horizontal-bar" ? "40" : isScorecard ? "85" : "",
    targetLabel: kind === "horizontal-bar" ? "Escalation" : isScorecard ? "Board plan" : "",
    showValues: true,
    paletteText: ["#2563eb", "#16a34a", "#f59e0b"].join("\n"),
    dataText: sampleDataByKind[kind],
  };
}

export function chartDesignerDraftFromMarkdownTable(markdown: string, base: ChartDesignerDraft = chartDesignerDefaultDraft()): ChartDesignerDraft | null {
  const table = parseMarkdownTable(markdown);
  if (!table || table.headers.length < 2 || !table.rows.length) return null;
  const [xField, yField] = table.headers;
  return {
    ...base,
    title: base.title || `${yField} by ${xField}`,
    xField,
    yField,
    dataText: [table.headers.join(", "), ...table.rows.map((row) => row.slice(0, table.headers.length).join(", "))].join("\n"),
  };
}

export function chartDesignerMarkdown(draft: ChartDesignerDraft): string {
  const parsed = parseDelimitedChartData(draft.dataText);
  const headers = parsed.headers.length >= 2 ? parsed.headers : ["Label", "Value"];
  const rows = parsed.rows.length ? parsed.rows : [["Example", "100"]];
  const xLabel = draft.xField.trim() || headers[0] || "Label";
  const yLabel = draft.yField.trim() || headers[1] || "Value";
  const xKey = chartFieldKey(xLabel);
  const yKey = chartFieldKey(yLabel);
  const dataKeys = headers.map((header, index) => (index === 0 ? xKey : index === 1 ? yKey : chartFieldKey(header)));
  const palette = draft.paletteText
    .split(/\r?\n|,/)
    .map((color) => color.trim())
    .filter(Boolean);
  const chartType = draft.kind === "scorecard" ? "bar" : draft.kind;
  const lines = [
    "```chart",
    `type: ${chartType}`,
    `title: ${yamlScalar(draft.title.trim() || "Untitled chart")}`,
  ];
  if (draft.subtitle.trim()) lines.push(`subtitle: ${yamlScalar(draft.subtitle.trim())}`);
  if (draft.source.trim()) lines.push(`source: ${yamlScalar(draft.source.trim())}`);
  if (numericLike(draft.target)) lines.push(`target: ${draft.target.trim()}`);
  if (draft.targetLabel.trim()) lines.push(`targetLabel: ${yamlScalar(draft.targetLabel.trim())}`);
  if (draft.valueSuffix.trim()) lines.push(`valueSuffix: ${yamlScalar(draft.valueSuffix.trim())}`);
  if (draft.kind === "scorecard") {
    lines.push('targetColor: "#334155"', 'negativeColor: "#be123c"', 'titleColor: "#111827"', 'textColor: "#0f172a"', 'mutedColor: "#64748b"', 'axisColor: "#cbd5e1"', 'background: "#ffffff"');
  }
  lines.push(`showValues: ${draft.showValues ? "true" : "false"}`);
  if (palette.length) {
    lines.push("palette:");
    for (const color of palette) lines.push(`  - ${yamlScalar(color)}`);
  }
  lines.push("data:");
  for (const row of rows) {
    lines.push(`  - ${dataKeys[0] || xKey}: ${yamlScalar(row[0] ?? "")}`);
    headers.slice(1).forEach((header, index) => {
      const value = row[index + 1] ?? "";
      lines.push(`    ${dataKeys[index + 1] || chartFieldKey(header)}: ${yamlValue(value)}`);
    });
  }
  lines.push(`x: ${xKey}`, `y: ${yKey}`, "```");
  return lines.join("\n");
}

function parseDelimitedChartData(text: string): ParsedChartData {
  const lines = text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  if (!lines.length) return { headers: [], rows: [] };
  const delimiter = lines[0].includes("\t") ? "\t" : ",";
  const rows = lines.map((line) => line.split(delimiter).map((cell) => cell.trim()));
  const headers = rows.shift() || [];
  return {
    headers: headers.map((header, index) => header || `Field ${index + 1}`),
    rows: rows.filter((row) => row.some(Boolean)),
  };
}

function parseMarkdownTable(markdown: string): ParsedChartData | null {
  const lines = markdown
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.startsWith("|") && line.endsWith("|"));
  const separatorIndex = lines.findIndex((line) => /^\|\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|$/.test(line));
  if (separatorIndex <= 0) return null;
  const headers = splitMarkdownTableRow(lines[separatorIndex - 1]);
  const rows = lines.slice(separatorIndex + 1).map(splitMarkdownTableRow).filter((row) => row.some(Boolean));
  return { headers, rows };
}

function splitMarkdownTableRow(line: string) {
  return line
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((cell) => cell.trim());
}

function chartFieldKey(label: string) {
  const key = label
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
  if (!key) return "value";
  return /^\d/.test(key) ? `field_${key}` : key;
}

function yamlValue(value: string) {
  const trimmed = value.trim();
  return numericLike(trimmed) ? trimmed : yamlScalar(trimmed);
}

function yamlScalar(value: string) {
  return JSON.stringify(value);
}

function numericLike(value: string) {
  return /^-?\d+(?:\.\d+)?$/.test(value.trim());
}
