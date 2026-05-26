import { EditorSelection } from "@codemirror/state";
import { cursorGroupLeft, cursorGroupRight } from "@codemirror/commands";
import type { EditorView, KeyBinding } from "@codemirror/view";

export interface EmacsKillRing {
  text: string;
}

export type EmacsWordDirection = "forward" | "backward";

export function emacsSupplementalKeymap(killRing: EmacsKillRing): KeyBinding[] {
  return [
    { key: "Ctrl-k", run: (view) => emacsKillLine(view, killRing) },
    { key: "Ctrl-y", run: (view) => emacsYank(view, killRing) },
    { key: "Ctrl-w", run: (view) => emacsKillSelection(view, killRing) },
    { key: "Alt-d", run: (view) => emacsKillWord(view, killRing, "forward") },
    { key: "Alt-Backspace", run: (view) => emacsKillWord(view, killRing, "backward") },
    { key: "Alt-f", run: cursorGroupRight },
    { key: "Alt-b", run: cursorGroupLeft },
  ];
}

export function emacsKillLineRange(text: string, cursor: number) {
  const safeCursor = Math.max(0, Math.min(text.length, cursor));
  const lineEnd = text.indexOf("\n", safeCursor);
  const to = lineEnd === -1 ? text.length : lineEnd === safeCursor ? lineEnd + 1 : lineEnd;
  return { from: safeCursor, to, text: text.slice(safeCursor, to) };
}

export function emacsWordRange(text: string, cursor: number, direction: EmacsWordDirection) {
  const safeCursor = Math.max(0, Math.min(text.length, cursor));
  if (direction === "backward") {
    let position = safeCursor;
    while (position > 0 && isEmacsWhitespace(text[position - 1])) position -= 1;
    while (position > 0 && !isEmacsWhitespace(text[position - 1])) position -= 1;
    return { from: position, to: safeCursor, text: text.slice(position, safeCursor) };
  }
  let position = safeCursor;
  while (position < text.length && isEmacsWhitespace(text[position])) position += 1;
  while (position < text.length && !isEmacsWhitespace(text[position])) position += 1;
  return { from: safeCursor, to: position, text: text.slice(safeCursor, position) };
}

function emacsKillLine(view: EditorView, killRing: EmacsKillRing) {
  const selection = view.state.selection.main;
  if (!selection.empty) return emacsKillSelection(view, killRing);
  const range = emacsKillLineRange(view.state.doc.toString(), selection.head);
  return emacsKillRange(view, killRing, range.from, range.to);
}

function emacsKillWord(view: EditorView, killRing: EmacsKillRing, direction: EmacsWordDirection) {
  const selection = view.state.selection.main;
  if (!selection.empty) return emacsKillSelection(view, killRing);
  const range = emacsWordRange(view.state.doc.toString(), selection.head, direction);
  return emacsKillRange(view, killRing, range.from, range.to);
}

function emacsKillSelection(view: EditorView, killRing: EmacsKillRing) {
  const selection = view.state.selection.main;
  if (selection.empty) return false;
  return emacsKillRange(view, killRing, selection.from, selection.to);
}

function emacsKillRange(view: EditorView, killRing: EmacsKillRing, from: number, to: number) {
  const start = Math.max(0, Math.min(view.state.doc.length, from));
  const end = Math.max(0, Math.min(view.state.doc.length, to));
  if (start === end) return true;
  killRing.text = view.state.doc.sliceString(start, end);
  view.dispatch({
    changes: { from: start, to: end, insert: "" },
    selection: EditorSelection.cursor(start),
    scrollIntoView: true,
  });
  view.focus();
  return true;
}

function emacsYank(view: EditorView, killRing: EmacsKillRing) {
  if (!killRing.text) return false;
  const selection = view.state.selection.main;
  view.dispatch({
    changes: { from: selection.from, to: selection.to, insert: killRing.text },
    selection: EditorSelection.cursor(selection.from + killRing.text.length),
    scrollIntoView: true,
  });
  view.focus();
  return true;
}

function isEmacsWhitespace(char: string | undefined) {
  return !char || /\s/.test(char);
}
