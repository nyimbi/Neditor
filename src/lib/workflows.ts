import type { ConflictDiffRow } from "./conflict.js";

export type AiPasteInsertMode = "insert" | "quote" | "replace" | "appendix";

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

export function conflictMergeLine(row: ConflictDiffRow, source: "local" | "external") {
  const hasLine = source === "local" ? row.localLine !== null : row.externalLine !== null;
  if (!hasLine) return "";
  const value = source === "local" ? row.local : row.external;
  return value || "";
}

export function appendConflictMergeLine(currentText: string, row: ConflictDiffRow, source: "local" | "external") {
  const value = conflictMergeLine(row, source);
  if (!value) return currentText;
  return currentText ? `${currentText}\n${value}` : value;
}
