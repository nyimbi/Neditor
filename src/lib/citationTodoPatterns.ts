export const CITATION_TODO_MARKER_PATTERN = String.raw`citation\s*TODOs?|TODO:?\s*(?:add\s+)?citation(?:\s+needed)?|source\s+needed|needs\s+citation|citation\s+needed|cite\s+needed`;

export function citationTodoMarkerRegex() {
  return new RegExp(CITATION_TODO_MARKER_PATTERN, "gi");
}

export function countCitationTodoMarkers(text: string) {
  return (text.match(citationTodoMarkerRegex()) || []).length;
}
