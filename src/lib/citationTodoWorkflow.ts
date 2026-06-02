import { citationTodoMarkerRegex } from "./citationTodoPatterns.js";

export type CitationTodoStatus = "open" | "deferred";

export interface CitationTodoItem {
  id: string;
  line: number;
  column: number;
  marker: string;
  status: CitationTodoStatus;
  excerpt: string;
  note?: string;
}

const TEXT_TODO_SOURCE = citationTodoMarkerRegex().source;
const COMMENT_TODO_SOURCE = /<!--\s*citation-todo:\s*(open|deferred)\b([^>]*)-->/.source;

function textTodoPat() { return new RegExp(TEXT_TODO_SOURCE, "gi"); }
function commentTodoPat() { return new RegExp(COMMENT_TODO_SOURCE, "gi"); }

export function extractCitationTodoItems(markdown: string): CitationTodoItem[] {
  const items: CitationTodoItem[] = [];
  const lines = markdown.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    collectCommentTodos(items, line, index + 1);
    collectTextTodos(items, line, index + 1);
  }
  return items.slice(0, 100);
}

export function resolveCitationTodo(markdown: string, item: CitationTodoItem, citationReference: string, note = "") {
  const replacement = `${citationReference}${note.trim() ? ` <!-- citation-resolved: ${sanitizeCommentValue(note)} -->` : ""}`;
  return replaceCitationTodoMarker(markdown, item, replacement);
}

export function deferCitationTodo(markdown: string, item: CitationTodoItem, reason = "") {
  const normalizedReason = sanitizeCommentValue(reason) || "Source review deferred";
  const original = sanitizeCommentValue(item.marker);
  return replaceCitationTodoMarker(markdown, item, `<!-- citation-todo: deferred | reason: ${normalizedReason} | original: ${original} -->`);
}

export function citationTodoComment(note = "") {
  const normalizedNote = sanitizeCommentValue(note) || "Add source before release";
  return `<!-- citation-todo: open | note: ${normalizedNote} -->`;
}

export function citationTodoAuditMarkdown(items: CitationTodoItem[]) {
  if (!items.length) {
    return "## Citation TODO Audit\n\nNo citation TODOs are currently detected.\n";
  }
  return [
    "## Citation TODO Audit",
    "",
    ...items.map((item) => `- [ ] Line ${item.line} (${item.status}): ${item.excerpt}${item.note ? ` | ${item.note}` : ""}`),
    "",
  ].join("\n");
}

function collectCommentTodos(items: CitationTodoItem[], line: string, lineNumber: number) {
  for (const match of line.matchAll(commentTodoPat())) {
    const status = match[1]?.toLowerCase() === "deferred" ? "deferred" : "open";
    const marker = match[0];
    const column = (match.index || 0) + 1;
    items.push({
      id: citationTodoId(lineNumber, column, marker),
      line: lineNumber,
      column,
      marker,
      status,
      excerpt: lineExcerpt(line, marker),
      note: parseCommentNote(match[2] || ""),
    });
  }
}

function collectTextTodos(items: CitationTodoItem[], line: string, lineNumber: number) {
  const searchableLine = line.replace(commentTodoPat(), (match) => " ".repeat(match.length));
  for (const match of searchableLine.matchAll(textTodoPat())) {
    const marker = match[0];
    const column = (match.index || 0) + 1;
    items.push({
      id: citationTodoId(lineNumber, column, marker),
      line: lineNumber,
      column,
      marker,
      status: "open",
      excerpt: lineExcerpt(line, marker),
    });
  }
}

function replaceCitationTodoMarker(markdown: string, item: CitationTodoItem, replacement: string) {
  const newline = markdown.includes("\r\n") ? "\r\n" : "\n";
  const lines = markdown.split(/\r?\n/);
  const index = item.line - 1;
  if (index < 0 || index >= lines.length) return markdown;
  const line = lines[index];
  const preferredIndex = Math.max(item.column - 1, 0);
  const markerIndex = line.indexOf(item.marker, preferredIndex);
  const start = markerIndex >= 0 ? markerIndex : -1;
  if (start < 0) return markdown;
  lines[index] = `${line.slice(0, start)}${replacement}${line.slice(start + item.marker.length)}`;
  return lines.join(newline);
}

function parseCommentNote(attributes: string) {
  const match = attributes.match(/\|\s*(?:note|reason):\s*([^|]+)/i);
  return match ? match[1].trim() : undefined;
}

function lineExcerpt(line: string, marker: string) {
  return line.replace(marker, `[${marker}]`).replace(/\s+/g, " ").trim().slice(0, 220);
}

function citationTodoId(line: number, column: number, marker: string) {
  return `${line}:${column}:${marker.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")}`;
}

function sanitizeCommentValue(value: string) {
  return value.replace(/-->/g, "").replace(/\s+/g, " ").trim().slice(0, 240);
}
