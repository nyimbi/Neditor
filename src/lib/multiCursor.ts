export interface MultiCursorRange {
  from: number;
  to: number;
}

export interface MultiCursorOccurrenceResult {
  term: string;
  ranges: MultiCursorRange[];
}

export function occurrenceRangesForSelection(text: string, from: number, to: number, limit = 500): MultiCursorOccurrenceResult {
  const source = String(text ?? "");
  const selection = normalizeRange(from, to, source.length);
  const selected = source.slice(selection.from, selection.to);
  const term = selected || wordAtPosition(source, selection.from);
  if (!term || !term.trim()) return { term: "", ranges: [] };
  return {
    term,
    ranges: findExactOccurrences(source, term, limit),
  };
}

export function splitSelectionIntoLineRanges(text: string, from: number, to: number): MultiCursorRange[] {
  const source = String(text ?? "");
  const selection = normalizeRange(from, to, source.length);
  if (selection.from === selection.to) return [selection];

  const ranges: MultiCursorRange[] = [];
  for (const line of lineRanges(source)) {
    if (line.to < selection.from) continue;
    if (line.from > selection.to) break;
    if (line.from === selection.to && line.from === source.length && source[selection.to - 1] === "\n") break;
    const fromInLine = Math.max(line.from, selection.from);
    const toInLine = Math.min(line.to, selection.to);
    if (fromInLine <= toInLine) ranges.push({ from: fromInLine, to: Math.max(fromInLine, toInLine) });
  }
  return ranges.length ? ranges : [selection];
}

function findExactOccurrences(text: string, term: string, limit: number): MultiCursorRange[] {
  const ranges: MultiCursorRange[] = [];
  let index = text.indexOf(term);
  while (index >= 0 && ranges.length < limit) {
    ranges.push({ from: index, to: index + term.length });
    index = text.indexOf(term, index + Math.max(1, term.length));
  }
  return ranges;
}

function wordAtPosition(text: string, position: number) {
  const cursor = Math.max(0, Math.min(text.length, position));
  let from = cursor;
  let to = cursor;
  while (from > 0 && isWordChar(text[from - 1])) from -= 1;
  while (to < text.length && isWordChar(text[to])) to += 1;
  return text.slice(from, to);
}

function lineRanges(text: string): MultiCursorRange[] {
  const ranges: MultiCursorRange[] = [];
  let from = 0;
  for (let index = 0; index <= text.length; index += 1) {
    if (index === text.length || text[index] === "\n") {
      ranges.push({ from, to: index });
      from = index + 1;
    }
  }
  return ranges;
}

function normalizeRange(from: number, to: number, max: number): MultiCursorRange {
  const start = clamp(from, 0, max);
  const end = clamp(to, 0, max);
  return start <= end ? { from: start, to: end } : { from: end, to: start };
}

function clamp(value: number, min: number, max: number) {
  if (!Number.isFinite(value)) return min;
  return Math.max(min, Math.min(max, Math.trunc(value)));
}

function isWordChar(char: string | undefined) {
  return Boolean(char && /[A-Za-z0-9_-]/.test(char));
}
