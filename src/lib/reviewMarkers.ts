const reviewCommentFallback = "Review comment";
const changeNoteFallback = "Change note";

function sanitizeMarkerText(text: string, fallback: string) {
  return (text.trim() || fallback).replace(/-->/g, "->");
}

export function reviewCommentMarker(text: string, createdAt: string) {
  return `<!-- comment: unresolved | author: local | at: ${createdAt} | ${sanitizeMarkerText(text, reviewCommentFallback)} -->`;
}

export function changeNoteMarker(text: string, createdAt: string) {
  return `<!-- change: author: local | at: ${createdAt} | ${sanitizeMarkerText(text, changeNoteFallback)} -->`;
}

export function appendReviewCommentMarker(documentText: string, text: string, createdAt: string) {
  return `${documentText}\n\n${reviewCommentMarker(text, createdAt)}\n`;
}

export function appendChangeNoteMarker(documentText: string, text: string, createdAt: string) {
  return `${documentText}\n\n${changeNoteMarker(text, createdAt)}\n`;
}

export function resolveReviewCommentAtLine(documentText: string, line: number) {
  const lines = documentText.split("\n");
  if (line < 1 || line > lines.length) return null;
  const index = line - 1;
  const currentLine = lines[index] || "";
  const resolvedLine = currentLine.replace(/<!--\s*comment:\s*unresolved\b/, "<!-- comment: resolved");
  if (resolvedLine === currentLine) return null;
  lines[index] = resolvedLine;
  return lines.join("\n");
}
