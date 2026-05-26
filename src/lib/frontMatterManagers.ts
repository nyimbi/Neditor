export type SupportedDataSourceKind = "csv" | "tsv" | "json" | "yaml";
export type FrontMatterDataSourceKind = SupportedDataSourceKind | string;
export type FrontMatterDataSourceStatus = "ready" | "missing-path" | "unsupported-type" | "blocked-path";

export interface FrontMatterDataSourceRow {
  id: string;
  name: string;
  path: string;
  kind: FrontMatterDataSourceKind;
  source: string;
  status: FrontMatterDataSourceStatus;
  detail: string;
  line: number;
}

export interface FrontMatterVariableRow {
  key: string;
  value: string;
  status: "ready" | "empty";
  line: number;
}

export const DATA_SOURCE_TYPE_OPTIONS: SupportedDataSourceKind[] = ["csv", "tsv", "json", "yaml"];

const frontMatterVariableExcludedKeys = new Set([
  "brand",
  "layout",
  "dataSources",
  "csvFiles",
  "tsvFiles",
  "jsonFiles",
  "yamlFiles",
  "ymlFiles",
  "bibliography",
  "indexExclude",
]);

const metadataVariableExcludedKeys = new Set([
  "source_hash",
  "sourceHash",
  "compiled_at",
  "exported_at",
  "output_path",
  "output_hash",
  "app_version",
]);

export function appendFrontMatterDataSource(
  text: string,
  source: { name: string; path: string; kind: SupportedDataSourceKind },
) {
  const entry = [
    `  - name: ${yamlInlineString(source.name || dataSourceNameFromPath(source.path))}`,
    `    path: ${yamlInlineString(source.path)}`,
    `    type: ${source.kind}`,
  ];
  const lines = text.startsWith("---\n") ? text.split("\n") : ["---", "---", "", ...text.split("\n")];
  const endIndex = lines.findIndex((candidate, index) => index > 0 && candidate.trim() === "---");
  if (endIndex <= 0) return `---\ndataSources:\n${entry.join("\n")}\n---\n\n${text}`;
  const startIndex = lines.findIndex((candidate, index) => index > 0 && index < endIndex && candidate.trim() === "dataSources:");
  if (startIndex > 0) {
    let insertIndex = startIndex + 1;
    while (insertIndex < endIndex && (/^\s/.test(lines[insertIndex]) || lines[insertIndex].trim() === "")) insertIndex += 1;
    lines.splice(insertIndex, 0, ...entry);
  } else {
    lines.splice(endIndex, 0, "dataSources:", ...entry);
  }
  return lines.join("\n");
}

export function parseFrontMatterDataSources(text: string): FrontMatterDataSourceRow[] {
  if (!text.startsWith("---\n")) return [];
  const lines = text.split("\n");
  const endIndex = lines.findIndex((candidate, index) => index > 0 && candidate.trim() === "---");
  if (endIndex <= 0) return [];
  const rows: FrontMatterDataSourceRow[] = [];
  let section = "";
  let current: Partial<FrontMatterDataSourceRow> | null = null;
  const flushCurrent = () => {
    if (!current) return;
    rows.push(normalizeFrontMatterDataSource(current, rows.length));
    current = null;
  };
  for (let index = 1; index < endIndex; index += 1) {
    const raw = lines[index];
    const topLevel = raw.match(/^([A-Za-z][\w-]*):\s*(.*)$/);
    if (topLevel) {
      flushCurrent();
      section = topLevel[1];
      const aliasKind = dataSourceAliasKind(section);
      if (aliasKind && topLevel[2].trim().startsWith("[")) {
        for (const item of topLevel[2].trim().replace(/^\[|\]$/g, "").split(",")) {
          const path = cleanYamlScalar(item);
          if (path) {
            rows.push(normalizeFrontMatterDataSource({ path, kind: aliasKind, source: section, line: index + 1 }, rows.length));
          }
        }
      }
      continue;
    }
    if (section === "dataSources") {
      const item = raw.match(/^\s*-\s*(.*)$/);
      if (item) {
        flushCurrent();
        current = { source: section, line: index + 1 };
        applyDataSourcePair(current, item[1]);
        continue;
      }
      const pair = raw.match(/^\s+([\w-]+):\s*(.*)$/);
      if (pair && current) {
        applyDataSourcePair(current, `${pair[1]}: ${pair[2]}`);
      }
      continue;
    }
    const aliasKind = dataSourceAliasKind(section);
    if (aliasKind) {
      const item = raw.match(/^\s*-\s*(.+)$/);
      if (item) {
        rows.push(
          normalizeFrontMatterDataSource(
            {
              path: cleanYamlScalar(item[1]),
              kind: aliasKind,
              source: section,
              line: index + 1,
            },
            rows.length,
          ),
        );
      }
    }
  }
  flushCurrent();
  return rows;
}

export function parseFrontMatterVariables(text: string): FrontMatterVariableRow[] {
  if (!text.startsWith("---\n")) return [];
  const lines = text.split("\n");
  const endIndex = lines.findIndex((candidate, index) => index > 0 && candidate.trim() === "---");
  if (endIndex <= 0) return [];
  const rows: FrontMatterVariableRow[] = [];
  const stack: Array<{ indent: number; path: string; excluded: boolean }> = [];
  for (let index = 1; index < endIndex; index += 1) {
    const raw = lines[index];
    const match = raw.match(/^(\s*)([A-Za-z][\w-]*):\s*(.*)$/);
    if (!match) continue;
    const indent = yamlIndentWidth(match[1]);
    while (stack.length && stack[stack.length - 1].indent >= indent) stack.pop();
    const parent = stack[stack.length - 1];
    const key = match[2];
    const path = parent ? `${parent.path}.${key}` : key;
    const excluded = Boolean(parent?.excluded || (!parent && frontMatterVariableExcludedKeys.has(key)));
    const hasChildren = hasIndentedYamlChildren(lines, endIndex, index, indent);
    if (hasChildren) stack.push({ indent, path, excluded });
    if (excluded) continue;
    const value = cleanYamlScalar(match[3]);
    if (!value || value === "[]" || value === "{}") {
      if (!hasChildren) rows.push({ key: path, value: "", status: "empty", line: index + 1 });
      continue;
    }
    if (/^[>|]$/.test(value) || value.startsWith("[") || value.startsWith("{")) continue;
    rows.push({
      key: path,
      value,
      status: "ready",
      line: index + 1,
    });
  }
  return rows.sort((left, right) => left.key.localeCompare(right.key));
}

export function parseMergedMetadataVariables(metadata: Record<string, unknown>, frontMatterRows: FrontMatterVariableRow[]): FrontMatterVariableRow[] {
  const frontMatterKeys = new Set(frontMatterRows.map((row) => row.key));
  const rows: FrontMatterVariableRow[] = [];
  collectMergedMetadataVariables(metadata, "", frontMatterKeys, rows);
  return rows.sort((left, right) => left.key.localeCompare(right.key));
}

export function dataSourceNameFromPath(path: string) {
  const file = path.split(/[\\/]/).pop() || "Data source";
  return file.replace(/\.[^.]+$/, "").replace(/[-_]+/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function applyDataSourcePair(row: Partial<FrontMatterDataSourceRow>, pairText: string) {
  const pair = pairText.match(/^([\w-]+):\s*(.*)$/);
  if (!pair) {
    if (pairText.trim()) row.path = cleanYamlScalar(pairText);
    return;
  }
  const key = pair[1];
  const value = cleanYamlScalar(pair[2]);
  if (key === "name" || key === "title") row.name = value;
  if (key === "path" || key === "file") row.path = value;
  if (key === "type" || key === "kind") row.kind = normalizeDataSourceKind(value);
}

function normalizeFrontMatterDataSource(row: Partial<FrontMatterDataSourceRow>, index: number): FrontMatterDataSourceRow {
  const path = row.path || "";
  const kind = row.kind || normalizeDataSourceKind(path.split(".").pop() || "");
  const status = dataSourceStatus(path, kind);
  return {
    id: `${row.source || "dataSources"}-${row.line || 0}-${path || index}`,
    name: row.name || dataSourceNameFromPath(path),
    path,
    kind,
    source: row.source || "dataSources",
    status,
    detail: dataSourceStatusDetail(status, path, kind),
    line: row.line || 0,
  };
}

function dataSourceAliasKind(section: string): FrontMatterDataSourceKind | null {
  const aliases: Record<string, FrontMatterDataSourceKind> = {
    csvFiles: "csv",
    tsvFiles: "tsv",
    jsonFiles: "json",
    yamlFiles: "yaml",
    ymlFiles: "yaml",
  };
  return aliases[section] || null;
}

function normalizeDataSourceKind(value: string): FrontMatterDataSourceKind {
  const normalized = value.trim().toLowerCase();
  if (normalized === "yml") return "yaml";
  if (DATA_SOURCE_TYPE_OPTIONS.includes(normalized as SupportedDataSourceKind)) return normalized;
  return normalized || "csv";
}

function dataSourceStatus(path: string, kind: FrontMatterDataSourceKind): FrontMatterDataSourceStatus {
  if (!path) return "missing-path";
  if (!DATA_SOURCE_TYPE_OPTIONS.includes(kind as SupportedDataSourceKind)) return "unsupported-type";
  if (path.startsWith("/") || path.includes("..")) return "blocked-path";
  return "ready";
}

function dataSourceStatusDetail(status: FrontMatterDataSourceStatus, path: string, kind: FrontMatterDataSourceKind) {
  if (status === "missing-path") return "Add a local file path inside the document folder.";
  if (status === "unsupported-type") return `Use CSV, TSV, JSON, or YAML instead of ${kind}.`;
  if (status === "blocked-path") return `${path} is outside the document folder; keep data sources local to the project.`;
  return "Ready for compiler import and export manifest evidence.";
}

function collectMergedMetadataVariables(
  value: unknown,
  path: string,
  frontMatterKeys: Set<string>,
  rows: FrontMatterVariableRow[],
) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return;
  for (const [key, child] of Object.entries(value as Record<string, unknown>)) {
    const childPath = path ? `${path}.${key}` : key;
    if (frontMatterKeys.has(childPath) || metadataVariableExcludedKeys.has(childPath) || metadataVariableExcludedKeys.has(key)) continue;
    if (isScalarMetadataValue(child)) {
      const rendered = cleanMetadataVariableValue(child);
      rows.push({ key: childPath, value: rendered, status: rendered ? "ready" : "empty", line: 0 });
      continue;
    }
    collectMergedMetadataVariables(child, childPath, frontMatterKeys, rows);
  }
}

function isScalarMetadataValue(value: unknown) {
  return typeof value === "string" || typeof value === "number" || typeof value === "boolean" || value === null;
}

function cleanMetadataVariableValue(value: unknown) {
  if (value === null || value === undefined) return "";
  return String(value).trim();
}

function yamlIndentWidth(indent: string) {
  return indent.replace(/\t/g, "  ").length;
}

function hasIndentedYamlChildren(lines: string[], endIndex: number, index: number, indent: number) {
  for (let nextIndex = index + 1; nextIndex < endIndex; nextIndex += 1) {
    const next = lines[nextIndex];
    if (!next.trim() || next.trimStart().startsWith("#")) continue;
    return yamlIndentWidth(next.match(/^\s*/)?.[0] || "") > indent;
  }
  return false;
}

function cleanYamlScalar(value: string) {
  return value.trim().replace(/\s+#.*$/, "").replace(/^["']|["']$/g, "");
}

function yamlInlineString(value: string) {
  return JSON.stringify(value);
}
