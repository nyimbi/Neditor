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
  const lines = startsWithFrontMatter(text) ? text.split(/\r?\n/) : ["---", "---", "", ...text.split(/\r?\n/)];
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
  if (!startsWithFrontMatter(text)) return [];
  const lines = text.split(/\r?\n/);
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
        for (const item of splitInlineYamlList(topLevel[2])) {
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
  if (!startsWithFrontMatter(text)) return [];
  const lines = text.split(/\r?\n/);
  const endIndex = lines.findIndex((candidate, index) => index > 0 && candidate.trim() === "---");
  if (endIndex <= 0) return [];
  const rows: FrontMatterVariableRow[] = [];
  const stack: Array<{ indent: number; path: string; excluded: boolean; anchor: string }> = [];
  const anchors = new Map<string, string>();
  const mapAnchors = new Map<string, Array<{ key: string; value: string; line: number }>>();
  for (let index = 1; index < endIndex; index += 1) {
    const raw = lines[index];
    const indentMatch = raw.match(/^(\s*)/);
    const rawIndent = yamlIndentWidth(indentMatch?.[1] || "");
    while (stack.length && stack[stack.length - 1].indent >= rawIndent) stack.pop();
    const parent = stack[stack.length - 1];
    const mergeMatch = raw.match(/^\s*<<:\s*(.*)$/);
    if (mergeMatch && parent && !parent.excluded) {
      for (const alias of yamlAliasNames(mergeMatch[1])) {
        for (const entry of mapAnchors.get(alias) || []) {
          recordMapAnchorEntry(mapAnchors, parent, entry.key, entry.value, entry.line, true);
          setVariableRow(
            rows,
            {
              key: `${parent.path}.${entry.key}`,
              value: entry.value,
              status: entry.value ? "ready" : "empty",
              line: index + 1,
            },
            true,
          );
        }
      }
      continue;
    }
    const match = raw.match(/^(\s*)([A-Za-z][\w-]*):\s*(.*)$/);
    if (!match) continue;
    const indent = yamlIndentWidth(match[1]);
    const key = match[2];
    const path = parent ? `${parent.path}.${key}` : key;
    const excluded = Boolean(parent?.excluded || (!parent && frontMatterVariableExcludedKeys.has(key)));
    const hasChildren = hasIndentedYamlChildren(lines, endIndex, index, indent);
    const parsed = parseYamlScalar(match[3]);
    if (parsed.anchor && hasChildren && !mapAnchors.has(parsed.anchor)) mapAnchors.set(parsed.anchor, []);
    let value = parsed.alias ? anchors.get(parsed.alias) || parsed.value : parsed.value;
    if (value === "|" || value === ">") {
      value = collectYamlBlockScalar(lines, endIndex, index, indent, value);
      if (parsed.anchor && value) anchors.set(parsed.anchor, value);
      if (!excluded && value) setVariableRow(rows, { key: path, value, status: "ready", line: index + 1 });
      continue;
    }
    if (parsed.anchor && value && !value.startsWith("[") && !value.startsWith("{")) anchors.set(parsed.anchor, value);
    if (hasChildren) stack.push({ indent, path, excluded, anchor: parsed.anchor });
    if (excluded) continue;
    if (!value || value === "[]" || value === "{}") {
      if (!hasChildren) setVariableRow(rows, { key: path, value: "", status: "empty", line: index + 1 });
      continue;
    }
    if (value.startsWith("[") || value.startsWith("{")) continue;
    for (const owner of stack.filter((entry) => entry.anchor)) {
      const relativeKey = path.startsWith(`${owner.path}.`) ? path.slice(owner.path.length + 1) : "";
      if (relativeKey) recordMapAnchorEntry(mapAnchors, owner, relativeKey, value, index + 1);
    }
    setVariableRow(rows, {
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
  if (isBlockedLocalDataSourcePath(path)) return "blocked-path";
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
  const withoutComment = stripYamlComment(value).trim();
  const decorated = stripLeadingYamlDecorators(withoutComment);
  if (decorated.scalar.length >= 2) {
    const quote = decorated.scalar[0];
    if ((quote === "\"" || quote === "'") && decorated.scalar.endsWith(quote)) {
      const body = decorated.scalar.slice(1, -1);
      return quote === "'" ? body.replace(/''/g, "'") : body.replace(/\\"/g, "\"");
    }
  }
  return decorated.scalar;
}

function parseYamlScalar(value: string) {
  const withoutComment = stripYamlComment(value).trim();
  const decorated = stripLeadingYamlDecorators(withoutComment);
  const alias = decorated.scalar.match(/^\*([A-Za-z0-9_-]+)$/)?.[1] || "";
  return {
    anchor: decorated.anchor,
    alias,
    value: cleanYamlScalar(decorated.scalar),
  };
}

function stripLeadingYamlDecorators(value: string) {
  let scalar = value.trim();
  let anchor = "";
  let previous = "";
  while (scalar && scalar !== previous) {
    previous = scalar;
    const anchorMatch = scalar.match(/^&([A-Za-z0-9_-]+)(?:\s+|$)(.*)$/);
    if (anchorMatch) {
      anchor = anchorMatch[1];
      scalar = anchorMatch[2].trim();
      continue;
    }
    const tagMatch = scalar.match(/^(?:!![A-Za-z0-9_.:/-]+|![A-Za-z0-9_.:/-]+|!<[^>]+>)(?:\s+|$)(.*)$/);
    if (tagMatch) {
      scalar = tagMatch[1].trim();
    }
  }
  return { anchor, scalar };
}

function collectYamlBlockScalar(lines: string[], endIndex: number, index: number, indent: number, style: string) {
  const collected: string[] = [];
  for (let nextIndex = index + 1; nextIndex < endIndex; nextIndex += 1) {
    const next = lines[nextIndex];
    if (!next.trim()) {
      collected.push("");
      continue;
    }
    const nextIndent = yamlIndentWidth(next.match(/^\s*/)?.[0] || "");
    if (nextIndent <= indent) break;
    collected.push(next.slice(Math.min(next.length, indent + 2)));
  }
  const text = style === ">" ? collected.map((line) => line.trim()).filter(Boolean).join(" ") : collected.join("\n").trim();
  return text.replace(/\s+/g, " ").trim();
}

function startsWithFrontMatter(text: string) {
  return /^---\r?\n/.test(text);
}

function splitInlineYamlList(value: string) {
  const trimmed = stripYamlComment(value).trim();
  if (!trimmed.startsWith("[") || !trimmed.endsWith("]")) return [];
  const inner = trimmed.slice(1, -1);
  const items: string[] = [];
  let quote = "";
  let escaped = false;
  let current = "";
  for (const char of inner) {
    if (escaped) {
      current += char;
      escaped = false;
      continue;
    }
    if (quote === "\"" && char === "\\") {
      current += char;
      escaped = true;
      continue;
    }
    if ((char === "\"" || char === "'") && !quote) {
      quote = char;
      current += char;
      continue;
    }
    if (char === quote) {
      quote = "";
      current += char;
      continue;
    }
    if (char === "," && !quote) {
      if (current.trim()) items.push(current.trim());
      current = "";
      continue;
    }
    current += char;
  }
  if (current.trim()) items.push(current.trim());
  return items;
}

function yamlAliasNames(value: string) {
  const scalar = stripLeadingYamlDecorators(stripYamlComment(value).trim()).scalar;
  const direct = scalar.match(/^\*([A-Za-z0-9_-]+)$/)?.[1];
  if (direct) return [direct];
  if (!scalar.startsWith("[") || !scalar.endsWith("]")) return [];
  return splitInlineYamlList(scalar)
    .map((item) => cleanYamlScalar(item).match(/^\*([A-Za-z0-9_-]+)$/)?.[1] || "")
    .filter(Boolean);
}

function setVariableRow(rows: FrontMatterVariableRow[], row: FrontMatterVariableRow, keepExisting = false) {
  const index = rows.findIndex((candidate) => candidate.key === row.key);
  if (index >= 0) {
    if (!keepExisting) rows[index] = row;
    return;
  }
  rows.push(row);
}

function recordMapAnchorEntry(
  mapAnchors: Map<string, Array<{ key: string; value: string; line: number }>>,
  owner: { anchor: string },
  key: string,
  value: string,
  line: number,
  keepExisting = false,
) {
  if (!owner.anchor || !key) return;
  const entries = mapAnchors.get(owner.anchor) || [];
  const index = entries.findIndex((entry) => entry.key === key);
  const anchoredEntry = { key, value, line };
  if (index >= 0) {
    if (!keepExisting) entries[index] = anchoredEntry;
  } else {
    entries.push(anchoredEntry);
  }
  mapAnchors.set(owner.anchor, entries);
}

function stripYamlComment(value: string) {
  let quote = "";
  let escaped = false;
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (quote === "\"" && char === "\\") {
      escaped = true;
      continue;
    }
    if ((char === "\"" || char === "'") && !quote) {
      quote = char;
      continue;
    }
    if (char === quote) {
      quote = "";
      continue;
    }
    if (char === "#" && !quote && (index === 0 || /\s/.test(value[index - 1]))) {
      return value.slice(0, index).trimEnd();
    }
  }
  return value;
}

function isBlockedLocalDataSourcePath(path: string) {
  const normalized = path.trim().replace(/\\/g, "/");
  return (
    normalized.startsWith("/") ||
    normalized.startsWith("//") ||
    /^[a-z][a-z0-9+.-]*:/i.test(normalized) ||
    normalized.split("/").some((segment) => segment === "..")
  );
}

function yamlInlineString(value: string) {
  return JSON.stringify(value);
}
