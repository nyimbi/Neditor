import type { ConflictDiffRow } from "./conflict.js";

export type AiPasteInsertMode = "insert" | "quote" | "replace" | "appendix";
export type ConflictMergeSource = "local" | "external";

export interface ConflictMergePart {
  id: string;
  source: ConflictMergeSource;
  line: number;
  text: string;
}

export function quoteMarkdown(text: string) {
  return text
    .split(/\r?\n/)
    .map((line) => (line ? `> ${line}` : ">"))
    .join("\n");
}

export function applyAiPasteInsertion(currentText: string, cleanedMarkdown: string, mode: AiPasteInsertMode) {
  if (mode === "replace") {
    return cleanedMarkdown;
  }
  if (mode === "quote") {
    return `${currentText}\n\n${quoteMarkdown(cleanedMarkdown)}\n`;
  }
  if (mode === "appendix") {
    return `${currentText}\n\n## AI Draft Appendix\n\n${cleanedMarkdown}\n`;
  }
  return `${currentText}\n\n${cleanedMarkdown}\n`;
}

export function conflictMergeLine(row: ConflictDiffRow, source: ConflictMergeSource) {
  const hasLine = source === "local" ? row.localLine !== null : row.externalLine !== null;
  if (!hasLine) return "";
  const value = source === "local" ? row.local : row.external;
  return value || "";
}

export function appendConflictMergeLine(currentText: string, row: ConflictDiffRow, source: ConflictMergeSource) {
  const value = conflictMergeLine(row, source);
  if (!value) return currentText;
  return currentText ? `${currentText}\n${value}` : value;
}

export function conflictMergePart(row: ConflictDiffRow, source: ConflictMergeSource): ConflictMergePart | null {
  const line = source === "local" ? row.localLine : row.externalLine;
  if (line === null) return null;
  return {
    id: `${source}:${line}:${row.key}`,
    source,
    line,
    text: source === "local" ? row.local : row.external,
  };
}

export function appendConflictMergePart(parts: ConflictMergePart[], row: ConflictDiffRow, source: ConflictMergeSource) {
  const part = conflictMergePart(row, source);
  if (!part || parts.some((existing) => existing.id === part.id)) return parts;
  return [...parts, part];
}

export function removeConflictMergePart(parts: ConflictMergePart[], id: string) {
  return parts.filter((part) => part.id !== id);
}

export function moveConflictMergePart(parts: ConflictMergePart[], id: string, direction: -1 | 1) {
  const index = parts.findIndex((part) => part.id === id);
  const target = index + direction;
  if (index < 0 || target < 0 || target >= parts.length) return parts;
  const next = [...parts];
  const [part] = next.splice(index, 1);
  next.splice(target, 0, part);
  return next;
}

export function renderConflictMergeParts(parts: ConflictMergePart[]) {
  return parts.map((part) => part.text).join("\n");
}
