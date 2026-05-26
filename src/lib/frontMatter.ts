export function frontMatterScalarValue(text: string, key: string) {
  const lines = frontMatterLines(text);
  if (!lines.length) return "";
  const line = lines.find((candidate) => frontMatterKeyLine(candidate, key));
  if (!line) return "";
  return line.split(":").slice(1).join(":").trim().replace(/^["']|["']$/g, "");
}

export function frontMatterAnyScalar(text: string, keys: string[]) {
  for (const key of keys) {
    const value = frontMatterScalarValue(text, key);
    if (value.trim()) return value.trim();
  }
  return "";
}

export function frontMatterListValues(text: string, key: string) {
  const lines = frontMatterLines(text);
  if (!lines.length) return [];
  const startIndex = lines.findIndex((candidate) => frontMatterKeyLine(candidate, key));
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

export function frontMatterAnyList(text: string, keys: string[]) {
  for (const key of keys) {
    const values = frontMatterListValues(text, key);
    if (values.length) return values;
  }
  return [];
}

export function upsertFrontMatterField(text: string, key: string, value: string) {
  const line = `${key}: ${value}`;
  const parsed = parseFrontMatter(text);
  if (!parsed) return `---\n${line}\n---\n\n${text}`;
  const lines = text.split(/\r?\n/);
  const existingIndex = lines.findIndex((candidate, index) => index > 0 && index < parsed.endIndex && frontMatterKeyLine(candidate, key));
  if (existingIndex > 0) {
    lines[existingIndex] = line;
  } else {
    lines.splice(parsed.endIndex, 0, line);
  }
  return lines.join("\n");
}

export function removeFrontMatterField(text: string, key: string) {
  const parsed = parseFrontMatter(text);
  if (!parsed) return text;
  const lines = text.split(/\r?\n/);
  const existingIndex = lines.findIndex((candidate, index) => index > 0 && index < parsed.endIndex && frontMatterKeyLine(candidate, key));
  if (existingIndex > 0) lines.splice(existingIndex, 1);
  return lines.join("\n");
}

export function upsertFrontMatterListField(text: string, key: string, values: string[]) {
  const uniqueValues = Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)));
  const block = uniqueValues.length
    ? [`${key}:`, ...uniqueValues.map((value) => `  - ${yamlInlineString(value)}`)]
    : [`${key}: []`];
  const parsed = parseFrontMatter(text);
  if (!parsed) return `---\n${block.join("\n")}\n---\n\n${text}`;
  const lines = text.split(/\r?\n/);
  const startIndex = lines.findIndex((candidate, index) => index > 0 && index < parsed.endIndex && frontMatterKeyLine(candidate, key));
  if (startIndex > 0) {
    let deleteCount = 1;
    while (startIndex + deleteCount < parsed.endIndex && /^\s+-\s+/.test(lines[startIndex + deleteCount])) deleteCount += 1;
    lines.splice(startIndex, deleteCount, ...block);
  } else {
    lines.splice(parsed.endIndex, 0, ...block);
  }
  return lines.join("\n");
}

function frontMatterLines(text: string) {
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  return match ? match[1].split(/\r?\n/) : [];
}

function parseFrontMatter(text: string) {
  if (!/^---\r?\n/.test(text)) return null;
  const lines = text.split(/\r?\n/);
  const endIndex = lines.findIndex((candidate, index) => index > 0 && candidate.trim() === "---");
  return endIndex > 0 ? { endIndex } : null;
}

function frontMatterKeyLine(line: string, key: string) {
  return new RegExp(`^\\s*${escapeRegExp(key)}\\s*:`).test(line);
}

function yamlInlineString(value: string) {
  return JSON.stringify(value);
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
