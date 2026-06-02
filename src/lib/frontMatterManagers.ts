export type SupportedDataSourceKind = "csv" | "tsv" | "json" | "yaml" | "xlsx";
export type FrontMatterDataSourceKind = SupportedDataSourceKind | string;
export type FrontMatterDataSourceStatus = "ready" | "missing-path" | "unsupported-type" | "blocked-path";

export interface FrontMatterDataSourceRow {
  id: string;
  name: string;
  path: string;
  kind: FrontMatterDataSourceKind;
  sheetName?: string;
  sheetIndex?: number;
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

interface InlineYamlMapEntry {
  key: string;
  value: string;
  line: number;
  keepExisting: boolean;
}

export const DATA_SOURCE_TYPE_OPTIONS: SupportedDataSourceKind[] = ["csv", "tsv", "json", "yaml", "xlsx"];

const frontMatterVariableExcludedKeys = new Set([
  "brand",
  "layout",
  "dataSources",
  "csvFiles",
  "tsvFiles",
  "jsonFiles",
  "yamlFiles",
  "ymlFiles",
  "xlsxFiles",
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

const yamlAnchorNamePattern = "[A-Za-z0-9_.:/@-]+";
const yamlKeyNamePattern = "[A-Za-z][A-Za-z0-9_-]*(?:\\.[A-Za-z][A-Za-z0-9_-]*)*";
const yamlTagNamePattern = "[A-Za-z0-9_.:/@-]+";
const yamlAliasScalarRegex = new RegExp(`^\\*(${yamlAnchorNamePattern})$`);
const yamlAnchorPrefixRegex = new RegExp(`^&(${yamlAnchorNamePattern})(?:\\s+|$)(.*)$`);
const yamlTagPrefixRegex = new RegExp(
  `^(!<[^>]+>|!!${yamlTagNamePattern}|!${yamlTagNamePattern}!${yamlTagNamePattern}|!${yamlTagNamePattern}|!)(?:\\s+|$)(.*)$`,
);
const yamlKeyScalarRegex = new RegExp(`^${yamlKeyNamePattern}$`);
const yamlKeyValueRegex = new RegExp(`^(${yamlKeyNamePattern}):\\s*(.*)$`);
const yamlMaybeIndentedKeyValueRegex = new RegExp(`^(\\s*)(${yamlKeyNamePattern}):\\s*(.*)$`);
const yamlTopLevelKeyValueRegex = new RegExp(`^(${yamlKeyNamePattern}):\\s*(.*)$`);
const yamlIndentedKeyValueRegex = new RegExp(`^\\s+(${yamlKeyNamePattern}):\\s*(.*)$`);

export function appendFrontMatterDataSource(
  text: string,
  source: { name: string; path: string; kind: SupportedDataSourceKind; sheetName?: string; sheetIndex?: number },
) {
  const entry = [
    `  - name: ${yamlInlineString(source.name || dataSourceNameFromPath(source.path))}`,
    `    path: ${yamlInlineString(source.path)}`,
    `    type: ${source.kind}`,
  ];
  if (source.kind === "xlsx") {
    if (source.sheetName?.trim()) entry.push(`    sheet: ${yamlInlineString(source.sheetName.trim())}`);
    if (Number.isFinite(source.sheetIndex) && Number(source.sheetIndex) > 0) {
      entry.push(`    sheetIndex: ${Math.trunc(Number(source.sheetIndex))}`);
    }
  }
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
  const anchors = new Map<string, string>();
  const mapAnchors = new Map<string, Array<{ key: string; value: string; line: number }>>();
  const sequenceAnchors = new Map<string, Array<Partial<FrontMatterDataSourceRow>>>();
  let section = "";
  let current: Partial<FrontMatterDataSourceRow> | null = null;
  let currentMapAnchor: { anchor: string; indent: number } | null = null;
  let currentSequenceAnchor: {
    anchor: string;
    indent: number;
    rows: Array<Partial<FrontMatterDataSourceRow>>;
    current: Partial<FrontMatterDataSourceRow> | null;
  } | null = null;
  const flushCurrent = () => {
    if (!current) return;
    rows.push(normalizeFrontMatterDataSource(current, rows.length));
    current = null;
  };
  const flushSequenceAnchorItem = () => {
    if (!currentSequenceAnchor?.current) return;
    currentSequenceAnchor.rows.push(currentSequenceAnchor.current);
    currentSequenceAnchor.current = null;
  };
  const stopSequenceAnchor = () => {
    if (!currentSequenceAnchor) return;
    flushSequenceAnchorItem();
    if (currentSequenceAnchor.rows.length) sequenceAnchors.set(currentSequenceAnchor.anchor, currentSequenceAnchor.rows);
    if (currentSequenceAnchor.rows.length) mapAnchors.delete(currentSequenceAnchor.anchor);
    currentSequenceAnchor = null;
  };
  for (let index = 1; index < endIndex; index += 1) {
    const raw = lines[index];
    const rawIndent = yamlIndentWidth(raw.match(/^\s*/)?.[0] || "");
    const topLevel = raw.match(yamlTopLevelKeyValueRegex);
    if (topLevel) {
      flushCurrent();
      stopSequenceAnchor();
      currentMapAnchor = null;
      section = topLevel[1];
      const aliasKind = dataSourceAliasKind(section);
      const parsedTopLevel = parseYamlScalar(topLevel[2]);
      if (parsedTopLevel.anchor && hasIndentedYamlChildren(lines, endIndex, index, 0)) {
        if (!mapAnchors.has(parsedTopLevel.anchor)) mapAnchors.set(parsedTopLevel.anchor, []);
        currentMapAnchor = { anchor: parsedTopLevel.anchor, indent: 0 };
        if (firstIndentedYamlChildIsSequence(lines, endIndex, index, 0)) {
          currentSequenceAnchor = { anchor: parsedTopLevel.anchor, indent: 0, rows: [], current: null };
        }
      }
      if (parsedTopLevel.anchor && parsedTopLevel.value && !parsedTopLevel.value.startsWith("[") && !parsedTopLevel.value.startsWith("{")) {
        anchors.set(parsedTopLevel.anchor, parsedTopLevel.value);
      }
      if (parsedTopLevel.anchor && parsedTopLevel.value.startsWith("[")) {
        anchors.set(parsedTopLevel.anchor, parsedTopLevel.value);
      }
      if (parsedTopLevel.anchor && parsedTopLevel.value.startsWith("{")) {
        for (const entry of parseInlineYamlMap(parsedTopLevel.value, anchors, mapAnchors, index + 1)) {
          recordMapAnchorEntry(mapAnchors, { anchor: parsedTopLevel.anchor }, entry.key, entry.value, entry.line, entry.keepExisting);
        }
      }
      if (section === "dataSources" && parsedTopLevel.value.startsWith("{")) {
        const inlineRow: Partial<FrontMatterDataSourceRow> = { source: section, line: index + 1 };
        if (applyInlineDataSourceObject(inlineRow, parsedTopLevel.value, anchors, mapAnchors)) {
          rows.push(normalizeFrontMatterDataSource(inlineRow, rows.length));
        }
      }
      if (section === "dataSources" && parsedTopLevel.alias) {
        const aliasedRow: Partial<FrontMatterDataSourceRow> = { source: section, line: index + 1 };
        if (sequenceAnchors.has(parsedTopLevel.alias)) {
          for (const itemRow of sequenceAnchors.get(parsedTopLevel.alias) || []) {
            rows.push(normalizeFrontMatterDataSource({ ...itemRow, source: section }, rows.length));
          }
        } else if (applyDataSourceMerge(aliasedRow, parsedTopLevel.value, mapAnchors)) {
          rows.push(normalizeFrontMatterDataSource(aliasedRow, rows.length));
        } else {
          const aliasedValue = anchors.get(parsedTopLevel.alias) || "";
          if (aliasedValue.startsWith("[")) {
            for (const item of splitInlineYamlList(aliasedValue)) {
              const itemRow = parseDataSourceListItem(item, section, index + 1, anchors, mapAnchors);
              if (itemRow) rows.push(normalizeFrontMatterDataSource(itemRow, rows.length));
            }
          } else if (aliasedValue) {
            rows.push(normalizeFrontMatterDataSource({ ...aliasedRow, path: aliasedValue }, rows.length));
          }
        }
      }
      if (section === "dataSources" && parsedTopLevel.value.startsWith("[")) {
        for (const item of splitInlineYamlList(parsedTopLevel.value)) {
          const inlineRow = parseDataSourceListItem(item, section, index + 1, anchors, mapAnchors);
          if (inlineRow) rows.push(normalizeFrontMatterDataSource(inlineRow, rows.length));
        }
      }
      if (
        section === "dataSources" &&
        !parsedTopLevel.alias &&
        parsedTopLevel.value &&
        !parsedTopLevel.value.startsWith("{") &&
        !parsedTopLevel.value.startsWith("[") &&
        parsedTopLevel.value !== "|" &&
        parsedTopLevel.value !== ">"
      ) {
        rows.push(normalizeFrontMatterDataSource({ source: section, line: index + 1, path: parsedTopLevel.value }, rows.length));
      }
      if (aliasKind && parsedTopLevel.value.startsWith("[")) {
        for (const item of splitInlineYamlList(parsedTopLevel.value)) {
          const path = resolveDataSourceScalar(item, anchors);
          if (path) {
            rows.push(normalizeFrontMatterDataSource({ path, kind: aliasKind, source: section, line: index + 1 }, rows.length));
          }
        }
      }
      if (aliasKind && parsedTopLevel.alias) {
        const aliasedValue = anchors.get(parsedTopLevel.alias) || "";
        if (sequenceAnchors.has(parsedTopLevel.alias)) {
          for (const itemRow of sequenceAnchors.get(parsedTopLevel.alias) || []) {
            rows.push(normalizeFrontMatterDataSource({ ...itemRow, kind: aliasKind, source: section }, rows.length));
          }
        } else if (aliasedValue.startsWith("[")) {
          for (const item of splitInlineYamlList(aliasedValue)) {
            const path = resolveDataSourceScalar(item, anchors);
            if (path) rows.push(normalizeFrontMatterDataSource({ path, kind: aliasKind, source: section, line: index + 1 }, rows.length));
          }
        } else if (aliasedValue) {
          rows.push(normalizeFrontMatterDataSource({ path: aliasedValue, kind: aliasKind, source: section, line: index + 1 }, rows.length));
        }
      }
      if (
        aliasKind &&
        !parsedTopLevel.alias &&
        parsedTopLevel.value &&
        !parsedTopLevel.value.startsWith("{") &&
        !parsedTopLevel.value.startsWith("[") &&
        parsedTopLevel.value !== "|" &&
        parsedTopLevel.value !== ">"
      ) {
        rows.push(
          normalizeFrontMatterDataSource(
            { path: parsedTopLevel.value, kind: aliasKind, source: section, line: index + 1 },
            rows.length,
          ),
        );
      }
      continue;
    }
    if (currentMapAnchor && rawIndent > currentMapAnchor.indent) {
      const anchorPair = raw.match(yamlIndentedKeyValueRegex);
      if (anchorPair) {
        const parsed = parseYamlScalar(anchorPair[2]);
        const value = parsed.alias ? anchors.get(parsed.alias) || parsed.value : parsed.value;
        if (parsed.anchor && value && !value.startsWith("[") && !value.startsWith("{")) anchors.set(parsed.anchor, value);
        if (value && !value.startsWith("[") && !value.startsWith("{") && value !== "|" && value !== ">") {
          recordMapAnchorEntry(mapAnchors, currentMapAnchor, anchorPair[1], value, index + 1);
        }
      }
    } else {
      currentMapAnchor = null;
    }
    if (currentSequenceAnchor && rawIndent > currentSequenceAnchor.indent) {
      const sequenceItem = raw.match(/^\s*-\s*(.*)$/);
      if (sequenceItem) {
        flushSequenceAnchorItem();
        currentSequenceAnchor.current = { line: index + 1 };
        if (
          !applyDataSourceMerge(currentSequenceAnchor.current, sequenceItem[1], mapAnchors) &&
          !applyInlineDataSourceObject(currentSequenceAnchor.current, sequenceItem[1], anchors, mapAnchors)
        ) {
          applyDataSourcePair(currentSequenceAnchor.current, sequenceItem[1], anchors);
        }
      } else if (currentSequenceAnchor.current) {
        const pair = raw.match(/^\s+([\w-]+):\s*(.*)$/);
        if (pair) applyDataSourcePair(currentSequenceAnchor.current, `${pair[1]}: ${pair[2]}`, anchors);
      }
    } else {
      stopSequenceAnchor();
    }
    if (section === "dataSources") {
      const item = raw.match(/^\s*-\s*(.*)$/);
      if (item) {
        flushCurrent();
        current = { source: section, line: index + 1 };
        if (!applyDataSourceMerge(current, item[1], mapAnchors) && !applyInlineDataSourceObject(current, item[1], anchors, mapAnchors)) {
          applyDataSourcePair(current, item[1], anchors);
        }
        continue;
      }
      const mergePair = raw.match(/^\s+<<:\s*(.*)$/);
      if (mergePair && current) {
        applyDataSourceMerge(current, mergePair[1], mapAnchors);
        continue;
      }
      const pair = raw.match(/^\s+([\w-]+):\s*(.*)$/);
      if (pair && current) {
        applyDataSourcePair(current, `${pair[1]}: ${pair[2]}`, anchors);
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
              path: resolveDataSourceScalar(item[1], anchors),
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
  stopSequenceAnchor();
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
  const sequenceIndexes = new Map<string, number>();
  const recordScalarForPath = (path: string, excluded: boolean, value: string, line: number, keepExisting = false) => {
    for (const owner of stack.filter((entry) => entry.anchor)) {
      const relativeKey = path.startsWith(`${owner.path}.`) ? path.slice(owner.path.length + 1) : "";
      if (relativeKey) recordMapAnchorEntry(mapAnchors, owner, relativeKey, value, line, keepExisting);
    }
    if (!excluded) {
      setVariableRow(
        rows,
        {
          key: path,
          value,
          status: value ? "ready" : "empty",
          line,
        },
        keepExisting,
      );
    }
  };
  const recordEntriesForPath = (path: string, excluded: boolean, entries: InlineYamlMapEntry[]) => {
    for (const entry of entries) {
      recordScalarForPath(`${path}.${entry.key}`, excluded, entry.value, entry.line, entry.keepExisting);
    }
  };
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
    const itemMatch = raw.match(/^(\s*)-\s*(.*)$/);
    if (itemMatch && parent) {
      const itemIndent = yamlIndentWidth(itemMatch[1]);
      const itemIndexKey = `${parent.path}:${itemIndent}`;
      const itemIndex = sequenceIndexes.get(itemIndexKey) || 0;
      sequenceIndexes.set(itemIndexKey, itemIndex + 1);
      const itemPath = `${parent.path}.${itemIndex}`;
      const itemExcluded = parent.excluded;
      const itemHasChildren = hasIndentedYamlChildren(lines, endIndex, index, itemIndent);
      const itemDecorated = stripLeadingYamlDecorators(stripYamlComment(itemMatch[2]).trim());
      const itemAnchor = itemDecorated.anchor;
      if (itemAnchor && !mapAnchors.has(itemAnchor)) mapAnchors.set(itemAnchor, []);
      if (itemHasChildren || itemAnchor) stack.push({ indent: itemIndent, path: itemPath, excluded: itemExcluded, anchor: itemAnchor });
      const itemMerge = itemDecorated.scalar.match(/^<<:\s*(.*)$/);
      const itemPair = itemDecorated.scalar.match(yamlKeyValueRegex);
      if (itemMerge) {
        recordEntriesForPath(
          itemPath,
          itemExcluded,
          yamlAliasNames(itemMerge[1]).flatMap((alias) =>
            (mapAnchors.get(alias) || []).map((entry) => ({ ...entry, keepExisting: true })),
          ),
        );
      } else if (itemPair) {
        const itemKey = itemPair[1];
        const itemValue = parseYamlScalar(itemPair[2]);
        let value = itemValue.alias ? anchors.get(itemValue.alias) || itemValue.value : itemValue.value;
        const fieldPath = `${itemPath}.${itemKey}`;
        if (itemValue.alias && mapAnchors.has(itemValue.alias)) {
          recordEntriesForPath(
            fieldPath,
            itemExcluded,
            (mapAnchors.get(itemValue.alias) || []).map((entry) => ({ ...entry, keepExisting: true })),
          );
        } else if (value.startsWith("{")) {
          const inlineEntries = parseInlineYamlMap(value, anchors, mapAnchors, index + 1);
          if (itemValue.anchor) {
            if (!mapAnchors.has(itemValue.anchor)) mapAnchors.set(itemValue.anchor, []);
            for (const entry of inlineEntries) {
              recordMapAnchorEntry(mapAnchors, { anchor: itemValue.anchor }, entry.key, entry.value, entry.line, entry.keepExisting);
            }
          }
          recordEntriesForPath(fieldPath, itemExcluded, inlineEntries);
        } else if (value.startsWith("[")) {
          const inlineEntries = parseInlineYamlSequence(value, anchors, mapAnchors, index + 1);
          if (itemValue.anchor) {
            if (!mapAnchors.has(itemValue.anchor)) mapAnchors.set(itemValue.anchor, []);
            for (const entry of inlineEntries) {
              recordMapAnchorEntry(mapAnchors, { anchor: itemValue.anchor }, entry.key, entry.value, entry.line, entry.keepExisting);
            }
          }
          recordEntriesForPath(fieldPath, itemExcluded, inlineEntries);
        } else if (value || !itemHasChildren) {
          if (value === "[]" || value === "{}") value = "";
          if (itemValue.anchor && value && !value.startsWith("[") && !value.startsWith("{")) anchors.set(itemValue.anchor, value);
          if (value !== "|" && value !== ">" && !value.startsWith("[") && !value.startsWith("{")) {
            recordScalarForPath(fieldPath, itemExcluded, value, index + 1);
          }
        }
      } else if (itemDecorated.scalar) {
        const itemValue = parseYamlScalar(itemMatch[2]);
        let value = itemValue.alias ? anchors.get(itemValue.alias) || itemValue.value : itemValue.value;
        if (itemValue.alias && mapAnchors.has(itemValue.alias)) {
          recordEntriesForPath(
            itemPath,
            itemExcluded,
            (mapAnchors.get(itemValue.alias) || []).map((entry) => ({ ...entry, keepExisting: true })),
          );
        } else if (value.startsWith("{")) {
          const inlineEntries = parseInlineYamlMap(value, anchors, mapAnchors, index + 1);
          recordMapAnchorEntries(mapAnchors, itemValue.anchor, inlineEntries);
          recordEntriesForPath(itemPath, itemExcluded, inlineEntries);
        } else if (value.startsWith("[")) {
          const inlineEntries = parseInlineYamlSequence(value, anchors, mapAnchors, index + 1);
          recordMapAnchorEntries(mapAnchors, itemValue.anchor, inlineEntries);
          recordEntriesForPath(itemPath, itemExcluded, inlineEntries);
        } else {
          if (value === "[]" || value === "{}") value = "";
          if (itemValue.anchor && value && !value.startsWith("[") && !value.startsWith("{")) anchors.set(itemValue.anchor, value);
          if (value !== "|" && value !== ">" && !value.startsWith("[") && !value.startsWith("{")) {
            recordScalarForPath(itemPath, itemExcluded, value, index + 1);
          }
        }
      }
      continue;
    }
    const match = raw.match(yamlMaybeIndentedKeyValueRegex);
    if (!match) continue;
    const indent = yamlIndentWidth(match[1]);
    const key = match[2];
    const path = parent ? `${parent.path}.${key}` : key;
    const excluded = Boolean(parent?.excluded || (!parent && frontMatterVariableExcludedKeys.has(rootYamlKey(key))));
    const hasChildren = hasIndentedYamlChildren(lines, endIndex, index, indent);
    const parsed = parseYamlScalar(match[3]);
    if (parsed.anchor && hasChildren && !mapAnchors.has(parsed.anchor)) mapAnchors.set(parsed.anchor, []);
    let value = parsed.alias ? anchors.get(parsed.alias) || parsed.value : parsed.value;
    if (parsed.alias && mapAnchors.has(parsed.alias)) {
      const aliasedEntries = mapAnchors.get(parsed.alias) || [];
      if (parsed.anchor) {
        if (!mapAnchors.has(parsed.anchor)) mapAnchors.set(parsed.anchor, []);
        for (const entry of aliasedEntries) {
          recordMapAnchorEntry(mapAnchors, { anchor: parsed.anchor }, entry.key, entry.value, entry.line, true);
        }
      }
      for (const owner of stack.filter((entry) => entry.anchor)) {
        const relativeKey = path.startsWith(`${owner.path}.`) ? path.slice(owner.path.length + 1) : "";
        if (!relativeKey) continue;
        for (const entry of aliasedEntries) {
          recordMapAnchorEntry(mapAnchors, owner, `${relativeKey}.${entry.key}`, entry.value, entry.line, true);
        }
      }
      if (!excluded) {
        for (const entry of aliasedEntries) {
          setVariableRow(
            rows,
            {
              key: `${path}.${entry.key}`,
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
    if (value === "|" || value === ">") {
      value = collectYamlBlockScalar(lines, endIndex, index, indent, value);
      if (parsed.anchor && value) anchors.set(parsed.anchor, value);
      if (!excluded && value) setVariableRow(rows, { key: path, value, status: "ready", line: index + 1 });
      continue;
    }
    if (value.startsWith("{")) {
      const inlineEntries = parseInlineYamlMap(value, anchors, mapAnchors, index + 1);
      if (parsed.anchor) {
        if (!mapAnchors.has(parsed.anchor)) mapAnchors.set(parsed.anchor, []);
        for (const entry of inlineEntries) {
          recordMapAnchorEntry(mapAnchors, { anchor: parsed.anchor }, entry.key, entry.value, entry.line, entry.keepExisting);
        }
      }
      for (const owner of stack.filter((entry) => entry.anchor)) {
        const relativeKey = path.startsWith(`${owner.path}.`) ? path.slice(owner.path.length + 1) : "";
        if (!relativeKey) continue;
        for (const entry of inlineEntries) {
          recordMapAnchorEntry(
            mapAnchors,
            owner,
            `${relativeKey}.${entry.key}`,
            entry.value,
            entry.line,
            entry.keepExisting,
          );
        }
      }
      if (!excluded) {
        for (const entry of inlineEntries) {
          setVariableRow(
            rows,
            {
              key: `${path}.${entry.key}`,
              value: entry.value,
              status: entry.value ? "ready" : "empty",
              line: entry.line,
            },
            entry.keepExisting,
          );
        }
      }
      continue;
    }
    if (value.startsWith("[")) {
      const inlineEntries = parseInlineYamlSequence(value, anchors, mapAnchors, index + 1);
      if (parsed.anchor) {
        if (!mapAnchors.has(parsed.anchor)) mapAnchors.set(parsed.anchor, []);
        for (const entry of inlineEntries) {
          recordMapAnchorEntry(mapAnchors, { anchor: parsed.anchor }, entry.key, entry.value, entry.line, entry.keepExisting);
        }
      }
      for (const owner of stack.filter((entry) => entry.anchor)) {
        const relativeKey = path.startsWith(`${owner.path}.`) ? path.slice(owner.path.length + 1) : "";
        if (!relativeKey) continue;
        for (const entry of inlineEntries) {
          recordMapAnchorEntry(
            mapAnchors,
            owner,
            `${relativeKey}.${entry.key}`,
            entry.value,
            entry.line,
            entry.keepExisting,
          );
        }
      }
      if (!excluded) {
        for (const entry of inlineEntries) {
          setVariableRow(
            rows,
            {
              key: `${path}.${entry.key}`,
              value: entry.value,
              status: entry.value ? "ready" : "empty",
              line: entry.line,
            },
            entry.keepExisting,
          );
        }
      }
      continue;
    }
    if (parsed.anchor && value && !value.startsWith("[") && !value.startsWith("{")) anchors.set(parsed.anchor, value);
    if (hasChildren) stack.push({ indent, path, excluded, anchor: parsed.anchor });
    if (excluded) continue;
    if (!value || value === "[]" || value === "{}") {
      if (!hasChildren) recordScalarForPath(path, excluded, "", index + 1);
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

function applyDataSourcePair(row: Partial<FrontMatterDataSourceRow>, pairText: string, anchors?: Map<string, string>) {
  const pair = pairText.match(/^([\w-]+):\s*(.*)$/);
  if (!pair) {
    const value = resolveDataSourceScalar(pairText, anchors);
    if (value) row.path = value;
    return;
  }
  const key = pair[1];
  const value = resolveDataSourceScalar(pair[2], anchors);
  if (key === "name" || key === "title") row.name = value;
  if (key === "path" || key === "file") row.path = value;
  if (key === "type" || key === "kind") row.kind = normalizeDataSourceKind(value);
  if (["sheet", "sheetName", "sheet_name", "worksheet", "worksheetName", "worksheet_name"].includes(key)) row.sheetName = value;
  if (["sheetIndex", "sheet_index", "worksheetIndex", "worksheet_index"].includes(key)) row.sheetIndex = dataSourceSheetIndexFromText(value);
}

function resolveDataSourceScalar(value: string, anchors?: Map<string, string>) {
  const parsed = parseYamlScalar(value);
  const resolved = parsed.alias && anchors ? anchors.get(parsed.alias) || parsed.value : parsed.value;
  if (parsed.anchor && anchors && resolved && !resolved.startsWith("[") && !resolved.startsWith("{")) {
    anchors.set(parsed.anchor, resolved);
  }
  return resolved;
}

function applyDataSourceMerge(
  row: Partial<FrontMatterDataSourceRow>,
  value: string,
  mapAnchors: Map<string, Array<{ key: string; value: string; line: number }>>,
) {
  const aliases = yamlAliasNames(value);
  if (!aliases.length) return false;
  let applied = false;
  for (const alias of aliases) {
    for (const entry of mapAnchors.get(alias) || []) {
      applyDataSourcePair(row, `${entry.key}: ${entry.value}`);
      applied = true;
    }
  }
  return applied;
}

function applyInlineDataSourceObject(
  row: Partial<FrontMatterDataSourceRow>,
  value: string,
  anchors: Map<string, string>,
  mapAnchors: Map<string, Array<{ key: string; value: string; line: number }>>,
) {
  const decorated = stripLeadingYamlDecorators(stripYamlComment(value).trim());
  const trimmed = decorated.scalar;
  if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) return false;
  const entries = parseInlineYamlMap(trimmed, anchors, mapAnchors, row.line || 0);
  if (!entries.length) return false;
  if (decorated.anchor) {
    if (!mapAnchors.has(decorated.anchor)) mapAnchors.set(decorated.anchor, []);
    for (const entry of entries) {
      recordMapAnchorEntry(mapAnchors, { anchor: decorated.anchor }, entry.key, entry.value, entry.line, entry.keepExisting);
    }
  }
  for (const entry of entries) {
    applyDataSourcePair(row, `${entry.key}: ${entry.value}`);
  }
  return true;
}

function parseDataSourceListItem(
  item: string,
  source: string,
  line: number,
  anchors: Map<string, string>,
  mapAnchors: Map<string, Array<{ key: string; value: string; line: number }>>,
) {
  const row: Partial<FrontMatterDataSourceRow> = { source, line };
  if (!applyInlineDataSourceObject(row, item, anchors, mapAnchors)) {
    applyDataSourcePair(row, item, anchors);
  }
  return row.path || row.name || row.kind ? row : null;
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
    ...(row.sheetName ? { sheetName: row.sheetName } : {}),
    ...(row.sheetIndex ? { sheetIndex: row.sheetIndex } : {}),
    source: row.source || "dataSources",
    status,
    detail: dataSourceStatusDetail(status, path, kind, row.sheetName, row.sheetIndex),
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
    xlsxFiles: "xlsx",
  };
  return aliases[section] || null;
}

function normalizeDataSourceKind(value: string): FrontMatterDataSourceKind {
  const normalized = value.trim().toLowerCase();
  if (normalized === "yml") return "yaml";
  if (DATA_SOURCE_TYPE_OPTIONS.includes(normalized as SupportedDataSourceKind)) return normalized;
  if (!normalized) return "csv";
  return normalized;
}

function dataSourceStatus(path: string, kind: FrontMatterDataSourceKind): FrontMatterDataSourceStatus {
  if (!path) return "missing-path";
  if (!DATA_SOURCE_TYPE_OPTIONS.includes(kind as SupportedDataSourceKind)) return "unsupported-type";
  if (isBlockedLocalDataSourcePath(path)) return "blocked-path";
  return "ready";
}

function dataSourceStatusDetail(
  status: FrontMatterDataSourceStatus,
  path: string,
  kind: FrontMatterDataSourceKind,
  sheetName?: string,
  sheetIndex?: number,
) {
  if (status === "missing-path") return "Add a local file path inside the document folder.";
  if (status === "unsupported-type") return `Use CSV, TSV, JSON, YAML, or XLSX instead of ${kind}.`;
  if (status === "blocked-path") return `${path} is outside the document folder; keep data sources local to the project.`;
  if (kind === "xlsx" && sheetName) return `Ready to import worksheet "${sheetName}".`;
  if (kind === "xlsx" && sheetIndex) return `Ready to import worksheet ${sheetIndex}.`;
  return "Ready for compiler import and export manifest evidence.";
}

function dataSourceSheetIndexFromText(value: string): number | undefined {
  const parsed = Number.parseInt(value.trim(), 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
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

function rootYamlKey(key: string) {
  return key.split(".")[0] || key;
}

function hasIndentedYamlChildren(lines: string[], endIndex: number, index: number, indent: number) {
  for (let nextIndex = index + 1; nextIndex < endIndex; nextIndex += 1) {
    const next = lines[nextIndex];
    if (!next.trim() || next.trimStart().startsWith("#")) continue;
    return yamlIndentWidth(next.match(/^\s*/)?.[0] || "") > indent;
  }
  return false;
}

function firstIndentedYamlChildIsSequence(lines: string[], endIndex: number, index: number, indent: number) {
  for (let nextIndex = index + 1; nextIndex < endIndex; nextIndex += 1) {
    const next = lines[nextIndex];
    if (!next.trim() || next.trimStart().startsWith("#")) continue;
    const nextIndent = yamlIndentWidth(next.match(/^\s*/)?.[0] || "");
    return nextIndent > indent && /^\s*-\s*/.test(next);
  }
  return false;
}

function cleanYamlScalar(value: string) {
  const withoutComment = stripYamlComment(value).trim();
  const decorated = stripLeadingYamlDecorators(withoutComment);
  return cleanDecoratedYamlScalar(decorated.scalar, decorated.tags);
}

function cleanDecoratedYamlScalar(value: string, tags: string[]) {
  let scalar = value;
  if (scalar.length >= 2) {
    const quote = scalar[0];
    if ((quote === "\"" || quote === "'") && scalar.endsWith(quote)) {
      const body = scalar.slice(1, -1);
      scalar = quote === "'" ? body.replace(/''/g, "'") : body.replace(/\\"/g, "\"");
    }
  }
  return normalizeTaggedYamlScalar(scalar, tags);
}

function parseYamlScalar(value: string) {
  const withoutComment = stripYamlComment(value).trim();
  const decorated = stripLeadingYamlDecorators(withoutComment);
  const alias = decorated.scalar.match(yamlAliasScalarRegex)?.[1] || "";
  return {
    anchor: decorated.anchor,
    alias,
    value: cleanDecoratedYamlScalar(decorated.scalar, decorated.tags),
  };
}

function stripLeadingYamlDecorators(value: string) {
  let scalar = value.trim();
  let anchor = "";
  const tags: string[] = [];
  let previous = "";
  while (scalar && scalar !== previous) {
    previous = scalar;
    const anchorMatch = scalar.match(yamlAnchorPrefixRegex);
    if (anchorMatch) {
      anchor = anchorMatch[1];
      scalar = anchorMatch[2].trim();
      continue;
    }
    const tagMatch = scalar.match(yamlTagPrefixRegex);
    if (tagMatch) {
      tags.push(tagMatch[1]);
      scalar = tagMatch[2].trim();
    }
  }
  return { anchor, scalar, tags };
}

function normalizeTaggedYamlScalar(value: string, tags: string[]) {
  if (tags.some(isYamlNullTag) && isYamlNullToken(value)) return "";
  if (tags.some(isYamlBoolTag)) {
    if (/^(?:true|yes|y|on)$/i.test(value)) return "true";
    if (/^(?:false|no|n|off)$/i.test(value)) return "false";
  }
  return value;
}

function isYamlNullTag(tag: string) {
  const normalized = tag.toLowerCase();
  return normalized === "!!null" || normalized === "!<tag:yaml.org,2002:null>";
}

function isYamlBoolTag(tag: string) {
  const normalized = tag.toLowerCase();
  return normalized === "!!bool" || normalized === "!<tag:yaml.org,2002:bool>";
}

function isYamlNullToken(value: string) {
  return value === "" || value === "~" || /^null$/i.test(value);
}

function collectYamlBlockScalar(lines: string[], endIndex: number, index: number, indent: number, style: string) {
  const collected: string[] = [];
  let contentIndentChars: number | null = null;
  for (let nextIndex = index + 1; nextIndex < endIndex; nextIndex += 1) {
    const next = lines[nextIndex];
    if (!next.trim()) {
      collected.push("");
      continue;
    }
    const nextIndent = yamlIndentWidth(next.match(/^\s*/)?.[0] || "");
    if (nextIndent <= indent) break;
    if (contentIndentChars === null) {
      contentIndentChars = next.match(/^\s*/)?.[0].length ?? 0;
    }
    collected.push(next.slice(contentIndentChars));
  }
  if (style === ">") {
    const text = collected.map((line) => line.trim()).filter(Boolean).join(" ");
    return text.replace(/\s+/g, " ").trim();
  }
  return collected.join("\n").trim();
}

function startsWithFrontMatter(text: string) {
  return /^---\r?\n/.test(text);
}

function splitInlineYamlList(value: string) {
  const trimmed = stripYamlComment(value).trim();
  if (!trimmed.startsWith("[") || !trimmed.endsWith("]")) return [];
  const inner = trimmed.slice(1, -1);
  return splitInlineYamlCollection(inner);
}

function parseInlineYamlMap(
  value: string,
  anchors: Map<string, string>,
  mapAnchors: Map<string, Array<{ key: string; value: string; line: number }>>,
  line: number,
): InlineYamlMapEntry[] {
  const trimmed = stripYamlComment(value).trim();
  if (!trimmed.startsWith("{") || !trimmed.endsWith("}")) return [];
  const entries: InlineYamlMapEntry[] = [];
  for (const item of splitInlineYamlCollection(trimmed.slice(1, -1))) {
    const separator = findInlineYamlKeySeparator(item);
    if (separator < 0) continue;
    const key = cleanYamlScalar(item.slice(0, separator)).trim();
    const rawValue = item.slice(separator + 1).trim();
    if (key === "<<") {
      for (const alias of yamlAliasNames(rawValue)) {
        for (const entry of mapAnchors.get(alias) || []) {
          entries.push({ ...entry, keepExisting: true });
        }
      }
      continue;
    }
    if (!yamlKeyScalarRegex.test(key)) continue;
    const parsed = parseYamlScalar(rawValue);
    let entryValue = parsed.alias ? anchors.get(parsed.alias) || parsed.value : parsed.value;
    if (parsed.anchor && entryValue && !entryValue.startsWith("[") && !entryValue.startsWith("{")) anchors.set(parsed.anchor, entryValue);
    if (entryValue === "[]" || entryValue === "{}") entryValue = "";
    if (entryValue.startsWith("{")) {
      const nestedEntries = parseInlineYamlMap(entryValue, anchors, mapAnchors, line);
      if (parsed.anchor) {
        if (!mapAnchors.has(parsed.anchor)) mapAnchors.set(parsed.anchor, []);
        for (const nested of nestedEntries) {
          recordMapAnchorEntry(mapAnchors, { anchor: parsed.anchor }, nested.key, nested.value, nested.line, nested.keepExisting);
        }
      }
      for (const nested of nestedEntries) {
        entries.push({ ...nested, key: `${key}.${nested.key}` });
      }
      continue;
    }
    if (entryValue.startsWith("[")) {
      const nestedEntries = parseInlineYamlSequence(entryValue, anchors, mapAnchors, line);
      recordMapAnchorEntries(mapAnchors, parsed.anchor, nestedEntries);
      for (const nested of nestedEntries) {
        entries.push({ ...nested, key: `${key}.${nested.key}` });
      }
      continue;
    }
    if (entryValue === "|" || entryValue === ">" || entryValue.startsWith("[") || entryValue.startsWith("{")) continue;
    entries.push({ key, value: entryValue, line, keepExisting: false });
  }
  return entries;
}

function parseInlineYamlSequence(
  value: string,
  anchors: Map<string, string>,
  mapAnchors: Map<string, Array<{ key: string; value: string; line: number }>>,
  line: number,
): InlineYamlMapEntry[] {
  const trimmed = stripYamlComment(value).trim();
  if (!trimmed.startsWith("[") || !trimmed.endsWith("]")) return [];
  const entries: InlineYamlMapEntry[] = [];
  splitInlineYamlCollection(trimmed.slice(1, -1)).forEach((item, itemIndex) => {
    const indexKey = String(itemIndex);
    const parsed = parseYamlScalar(item);
    let entryValue = parsed.alias ? anchors.get(parsed.alias) || parsed.value : parsed.value;
    if (parsed.alias && mapAnchors.has(parsed.alias)) {
      for (const entry of mapAnchors.get(parsed.alias) || []) {
        entries.push({ ...entry, key: `${indexKey}.${entry.key}`, keepExisting: true });
      }
      return;
    }
    if (entryValue === "[]" || entryValue === "{}") entryValue = "";
    if (entryValue.startsWith("{")) {
      const nestedEntries = parseInlineYamlMap(entryValue, anchors, mapAnchors, line);
      recordMapAnchorEntries(mapAnchors, parsed.anchor, nestedEntries);
      for (const entry of nestedEntries) {
        entries.push({ ...entry, key: `${indexKey}.${entry.key}` });
      }
      return;
    }
    if (entryValue.startsWith("[")) {
      const nestedEntries = parseInlineYamlSequence(entryValue, anchors, mapAnchors, line);
      recordMapAnchorEntries(mapAnchors, parsed.anchor, nestedEntries);
      for (const entry of nestedEntries) {
        entries.push({ ...entry, key: `${indexKey}.${entry.key}` });
      }
      return;
    }
    if (entryValue === "|" || entryValue === ">" || entryValue.startsWith("[") || entryValue.startsWith("{")) return;
    if (parsed.anchor && entryValue) anchors.set(parsed.anchor, entryValue);
    entries.push({ key: indexKey, value: entryValue, line, keepExisting: false });
  });
  return entries;
}

function splitInlineYamlCollection(inner: string) {
  const items: string[] = [];
  let quote = "";
  let escaped = false;
  let current = "";
  let depth = 0;
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
    if (!quote && (char === "[" || char === "{")) {
      depth += 1;
      current += char;
      continue;
    }
    if (!quote && (char === "]" || char === "}")) {
      depth = Math.max(0, depth - 1);
      current += char;
      continue;
    }
    if (char === "," && !quote && depth === 0) {
      if (current.trim()) items.push(current.trim());
      current = "";
      continue;
    }
    current += char;
  }
  if (current.trim()) items.push(current.trim());
  return items;
}

function findInlineYamlKeySeparator(value: string) {
  let quote = "";
  let escaped = false;
  let depth = 0;
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
    if (quote) continue;
    if (char === "[" || char === "{") {
      depth += 1;
      continue;
    }
    if (char === "]" || char === "}") {
      depth = Math.max(0, depth - 1);
      continue;
    }
    if (char === ":" && depth === 0) return index;
  }
  return -1;
}

function yamlAliasNames(value: string) {
  const scalar = stripLeadingYamlDecorators(stripYamlComment(value).trim()).scalar.replace(/^<<:\s*/, "");
  const direct = scalar.match(yamlAliasScalarRegex)?.[1];
  if (direct) return [direct];
  if (!scalar.startsWith("[") || !scalar.endsWith("]")) return [];
  return splitInlineYamlList(scalar)
    .map((item) => cleanYamlScalar(item).match(yamlAliasScalarRegex)?.[1] || "")
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

function recordMapAnchorEntries(
  mapAnchors: Map<string, Array<{ key: string; value: string; line: number }>>,
  anchor: string,
  entries: InlineYamlMapEntry[],
) {
  if (!anchor) return;
  if (!mapAnchors.has(anchor)) mapAnchors.set(anchor, []);
  for (const entry of entries) {
    recordMapAnchorEntry(mapAnchors, { anchor }, entry.key, entry.value, entry.line, entry.keepExisting);
  }
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
