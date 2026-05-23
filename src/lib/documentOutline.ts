export interface OutlinePlanItem {
  level: number;
  title: string;
}

export interface OutlineDocumentOptions {
  title?: string;
  includeToc?: boolean;
  placeholder?: string;
}

const defaultPlaceholder = "<!-- Draft this section. -->";

export function parseOutlinePlan(input: string): OutlinePlanItem[] {
  return input
    .split(/\r?\n/)
    .map((line) => parseOutlineLine(line))
    .filter((item): item is OutlinePlanItem => Boolean(item));
}

export function outlinePlanFromMarkdown(markdown: string): string {
  return markdown
    .split(/\r?\n/)
    .flatMap((line) => {
      const match = line.match(/^(#{1,6})\s+(.+?)\s*#*\s*$/);
      if (!match) return [];
      return [`${"  ".repeat(Math.max(0, match[1].length - 1))}- ${match[2].trim()}`];
    })
    .join("\n");
}

export function outlinePlanToMarkdown(input: string, options: OutlineDocumentOptions = {}) {
  const items = parseOutlinePlan(input);
  if (!items.length) return "";
  const explicitTitle = (options.title || "").trim();
  const first = items[0];
  const title = explicitTitle || first.title;
  const includeToc = options.includeToc !== false;
  const placeholder = options.placeholder ?? defaultPlaceholder;
  const lines = [
    "---",
    `title: ${yamlScalar(title)}`,
    "status: draft",
    includeToc ? "toc: true" : "",
    "---",
    "",
  ].filter((line) => line !== "");

  const contentItems = explicitTitle ? items : items.slice(1);
  lines.push(`# ${title}`, "");
  if (includeToc) lines.push("[TOC]", "");
  for (const item of contentItems) {
    const level = Math.min(6, Math.max(2, item.level + 1));
    lines.push(`${"#".repeat(level)} ${item.title}`, "", placeholder, "");
  }
  return `${lines.join("\n").replace(/\n{3,}/g, "\n\n").trimEnd()}\n`;
}

function parseOutlineLine(line: string): OutlinePlanItem | null {
  const trimmedRight = line.trimEnd();
  if (!trimmedRight.trim()) return null;
  const heading = trimmedRight.match(/^\s*(#{1,6})\s+(.+?)\s*#*\s*$/);
  if (heading) {
    return normalizeItem(heading[1].length, heading[2]);
  }
  const indent = trimmedRight.match(/^\s*/)?.[0] || "";
  const level = Math.min(6, Math.floor(indent.replace(/\t/g, "  ").length / 2) + 1);
  const title = trimmedRight
    .trim()
    .replace(/^[-*+]\s+/, "")
    .replace(/^\d+[.)]\s+/, "")
    .trim();
  return normalizeItem(level, title);
}

function normalizeItem(level: number, title: string): OutlinePlanItem | null {
  const cleanTitle = title.replace(/\s+/g, " ").trim();
  if (!cleanTitle) return null;
  return { level: Math.min(6, Math.max(1, level)), title: cleanTitle };
}

function yamlScalar(value: string) {
  if (/^[A-Za-z0-9 _.,:/-]+$/.test(value)) return value;
  return JSON.stringify(value);
}
