export type MarkdownListContinuation =
  | { kind: "continue"; insert: string }
  | { kind: "exit"; fromColumn: number; replacement: string };

export function markdownListContinuation(beforeCursor: string): MarkdownListContinuation | null {
  const quotePrefix = beforeCursor.match(/^((?:\s*>[ \t]?)+)/)?.[1] || "";
  const remainder = beforeCursor.slice(quotePrefix.length);
  const bullet = remainder.match(/^(\s*)([-+*])\s+(?:\[([ xX])\]\s*)?(.*)$/);
  const numbered = remainder.match(/^(\s*)(\d+)([.)])\s+(.*)$/);
  if (!bullet && !numbered) return null;

  if (bullet) {
    const [, indent, marker, taskState, rawContent] = bullet;
    const taskMarker = taskState === undefined ? "" : "[ ] ";
    const content = rawContent.trim();
    const markerText = `${indent}${marker} ${taskMarker}`;
    if (!content) {
      return { kind: "exit", fromColumn: quotePrefix.length, replacement: indent };
    }
    return { kind: "continue", insert: `\n${quotePrefix}${markerText}` };
  }

  const [, indent, number, suffix, rawContent] = numbered || [];
  const content = (rawContent || "").trim();
  const markerText = `${indent}${Number(number || "0") + 1}${suffix || "."} `;
  if (!content) {
    return { kind: "exit", fromColumn: quotePrefix.length, replacement: indent };
  }
  return { kind: "continue", insert: `\n${quotePrefix}${markerText}` };
}
