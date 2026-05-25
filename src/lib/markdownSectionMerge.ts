export interface MarkdownSectionRange {
  startLine: number;
  endLine: number;
  startOffset: number;
  endOffset: number;
  level: number;
  heading: string;
}

function normalizeHeading(text: string) {
  return text
    .replace(/\s+#+\s*$/, "")
    .replace(/\s+/g, " ")
    .trim()
    .toLowerCase();
}

function lineOffsets(lines: string[]) {
  const offsets: number[] = [];
  let offset = 0;
  for (const line of lines) {
    offsets.push(offset);
    offset += line.length + 1;
  }
  return offsets;
}

function headingFromLine(line: string) {
  const match = line.match(/^(#{1,6})\s+(.+?)\s*$/);
  if (!match) return null;
  return {
    level: match[1].length,
    heading: match[2].replace(/\s+#+\s*$/, "").trim(),
  };
}

export function findMarkdownSectionRange(markdown: string, heading: string, preferredLevel?: number): MarkdownSectionRange | null {
  const lines = markdown.split(/\r?\n/);
  const offsets = lineOffsets(lines);
  const target = normalizeHeading(heading);
  const matches: MarkdownSectionRange[] = [];

  for (let index = 0; index < lines.length; index += 1) {
    const candidate = headingFromLine(lines[index]);
    if (!candidate || normalizeHeading(candidate.heading) !== target) continue;
    let endLine = lines.length;
    for (let scan = index + 1; scan < lines.length; scan += 1) {
      const next = headingFromLine(lines[scan]);
      if (next && next.level <= candidate.level) {
        endLine = scan;
        break;
      }
    }
    matches.push({
      startLine: index,
      endLine,
      startOffset: offsets[index] ?? 0,
      endOffset: endLine < lines.length ? (offsets[endLine] ?? markdown.length) : markdown.length,
      level: candidate.level,
      heading: candidate.heading,
    });
  }

  return matches.find((match) => match.level === preferredLevel) || matches[0] || null;
}

export function extractMarkdownSection(markdown: string, heading: string, preferredLevel?: number) {
  const lines = markdown.split(/\r?\n/);
  const offsets = lineOffsets(lines);
  const range = findMarkdownSectionRange(markdown, heading, preferredLevel);
  if (!range) return "";
  const precedingLine = range.startLine > 0 ? lines[range.startLine - 1] : "";
  const startOffset = precedingLine?.trim().startsWith("<!-- ai-assisted:")
    ? (offsets[range.startLine - 1] ?? range.startOffset)
    : range.startOffset;
  return markdown.slice(startOffset, range.endOffset).trim();
}

export function replaceOrAppendMarkdownSection(documentText: string, draftMarkdown: string, heading: string, preferredLevel?: number) {
  const replacement = extractMarkdownSection(draftMarkdown, heading, preferredLevel);
  if (!replacement) return documentText;
  const existing = findMarkdownSectionRange(documentText, heading, preferredLevel);
  if (!existing) return `${documentText.trimEnd()}\n\n${replacement}\n`;
  const lines = documentText.split(/\r?\n/);
  const offsets = lineOffsets(lines);
  const precedingLine = existing.startLine > 0 ? lines[existing.startLine - 1] : "";
  const startOffset = precedingLine?.trim().startsWith("<!-- ai-assisted:")
    ? (offsets[existing.startLine - 1] ?? existing.startOffset)
    : existing.startOffset;
  const before = documentText.slice(0, startOffset).trimEnd();
  const after = documentText.slice(existing.endOffset).trimStart();
  return [before, replacement, after].filter(Boolean).join("\n\n") + "\n";
}
