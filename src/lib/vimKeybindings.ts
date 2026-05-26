import { EditorSelection } from "@codemirror/state";
import {
  cursorCharLeft,
  cursorCharRight,
  cursorDocEnd,
  cursorDocStart,
  cursorLineDown,
  cursorLineEnd,
  cursorLineStart,
  cursorLineUp,
  deleteCharForward,
  deleteLine,
  deleteToLineEnd,
  insertNewlineAndIndent,
  redo,
  undo,
} from "@codemirror/commands";
import type { EditorView } from "@codemirror/view";

export type VimInputMode = "insert" | "normal";
export type VimPendingOperator = "" | "d" | "c";
export type VimMotionKey = "w" | "e" | "b";

export interface VimKeybindingController {
  pendingOperator(): VimPendingOperator;
  setInputMode(mode: VimInputMode): void;
  setPendingOperator(operator: VimPendingOperator): void;
}

export function resetVimPendingOperator(controller: VimKeybindingController) {
  controller.setPendingOperator("");
}

export function handleVimNormalKey(event: KeyboardEvent, view: EditorView, controller: VimKeybindingController) {
  if (event.metaKey || event.altKey) return false;
  if (event.ctrlKey) {
    if (event.key.toLowerCase() === "r") {
      event.preventDefault();
      return redo(view);
    }
    return false;
  }
  const pendingOperator = controller.pendingOperator();
  if (pendingOperator) {
    if (event.key === pendingOperator) {
      event.preventDefault();
      return pendingOperator === "d" ? vimDeleteCurrentLine(view, controller) : vimChangeCurrentLine(view, controller);
    }
    if (isVimOperatorMotionKey(event.key)) {
      event.preventDefault();
      return vimApplyOperatorMotion(view, controller, pendingOperator, event.key);
    }
    if (event.key === "0" || event.key === "$") {
      event.preventDefault();
      return vimApplyLineOperatorMotion(view, controller, pendingOperator, event.key);
    }
    controller.setPendingOperator("");
  }
  const run = {
    h: () => cursorCharLeft(view),
    j: () => cursorLineDown(view),
    k: () => cursorLineUp(view),
    l: () => cursorCharRight(view),
    w: () => vimMoveWordForward(view),
    e: () => vimMoveWordEnd(view),
    b: () => vimMoveWordBackward(view),
    "0": () => cursorLineStart(view),
    "^": () => vimMoveFirstNonWhitespace(view),
    $: () => cursorLineEnd(view),
    x: () => deleteCharForward(view),
    D: () => deleteToLineEnd(view),
    C: () => vimChangeToLineEnd(view, controller),
    I: () => vimInsertAtLineStart(view, controller),
    A: () => vimAppendAtLineEnd(view, controller),
    i: () => vimEnterInsertMode(view, controller),
    a: () => vimInsertAfterCursor(view, controller),
    o: () => vimOpenLineBelow(view, controller),
    O: () => vimOpenLineAbove(view, controller),
    u: () => undo(view),
    G: () => cursorDocEnd(view),
    g: () => cursorDocStart(view),
    J: () => vimJoinLineWithNext(view),
  }[event.key];
  if (run) {
    event.preventDefault();
    return run();
  }
  if (event.key === "d" || event.key === "c") {
    event.preventDefault();
    controller.setPendingOperator(event.key);
    return true;
  }
  if (event.key.length === 1) {
    event.preventDefault();
    return true;
  }
  return false;
}

export function nextVimWordStart(text: string, cursor: number) {
  let position = Math.max(0, Math.min(text.length, cursor));
  if (position >= text.length) return position;
  if (isVimWhitespace(text[position])) {
    while (position < text.length && isVimWhitespace(text[position])) position += 1;
  } else {
    while (position < text.length && !isVimWhitespace(text[position])) position += 1;
    while (position < text.length && isVimWhitespace(text[position])) position += 1;
  }
  return position;
}

export function vimWordEnd(text: string, cursor: number) {
  if (!text.length) return 0;
  let position = Math.max(0, Math.min(text.length - 1, cursor + 1));
  while (position < text.length && isVimWhitespace(text[position])) position += 1;
  while (position < text.length - 1 && !isVimWhitespace(text[position + 1])) position += 1;
  return position;
}

export function previousVimWordStart(text: string, cursor: number) {
  let position = Math.max(0, Math.min(text.length, cursor) - 1);
  while (position > 0 && isVimWhitespace(text[position])) position -= 1;
  while (position > 0 && !isVimWhitespace(text[position - 1])) position -= 1;
  return position;
}

export function vimMotionRange(text: string, cursor: number, motion: VimMotionKey, operator: "d" | "c" = "d") {
  const start = Math.max(0, Math.min(text.length, cursor));
  if (motion === "b") {
    return normalizedRange(previousVimWordStart(text, start), start);
  }
  if (motion === "e" || (motion === "w" && operator === "c" && !isVimWhitespace(text[start]))) {
    return normalizedRange(start, Math.min(text.length, vimWordEnd(text, start) + 1));
  }
  return normalizedRange(start, nextVimWordStart(text, start));
}

function vimEnterInsertMode(view: EditorView, controller: VimKeybindingController) {
  controller.setPendingOperator("");
  controller.setInputMode("insert");
  view.focus();
  return true;
}

function vimInsertAtLineStart(view: EditorView, controller: VimKeybindingController) {
  cursorLineStart(view);
  return vimEnterInsertMode(view, controller);
}

function vimAppendAtLineEnd(view: EditorView, controller: VimKeybindingController) {
  cursorLineEnd(view);
  return vimEnterInsertMode(view, controller);
}

function vimInsertAfterCursor(view: EditorView, controller: VimKeybindingController) {
  cursorCharRight(view);
  return vimEnterInsertMode(view, controller);
}

function vimOpenLineBelow(view: EditorView, controller: VimKeybindingController) {
  cursorLineEnd(view);
  insertNewlineAndIndent(view);
  return vimEnterInsertMode(view, controller);
}

function vimOpenLineAbove(view: EditorView, controller: VimKeybindingController) {
  cursorLineStart(view);
  insertNewlineAndIndent(view);
  cursorLineUp(view);
  return vimEnterInsertMode(view, controller);
}

function vimMoveWordForward(view: EditorView) {
  return vimMoveCursor(view, nextVimWordStart(view.state.doc.toString(), view.state.selection.main.head));
}

function vimMoveWordEnd(view: EditorView) {
  return vimMoveCursor(view, vimWordEnd(view.state.doc.toString(), view.state.selection.main.head));
}

function vimMoveWordBackward(view: EditorView) {
  return vimMoveCursor(view, previousVimWordStart(view.state.doc.toString(), view.state.selection.main.head));
}

function vimMoveFirstNonWhitespace(view: EditorView) {
  const line = view.state.doc.lineAt(view.state.selection.main.head);
  const firstNonWhitespace = line.text.search(/\S/);
  return vimMoveCursor(view, line.from + Math.max(0, firstNonWhitespace));
}

function vimMoveCursor(view: EditorView, position: number) {
  const docLength = view.state.doc.length;
  view.dispatch({
    selection: EditorSelection.cursor(Math.max(0, Math.min(docLength, position))),
    scrollIntoView: true,
  });
  view.focus();
  return true;
}

function vimDeleteCurrentLine(view: EditorView, controller: VimKeybindingController) {
  controller.setPendingOperator("");
  return deleteLine(view);
}

function vimChangeCurrentLine(view: EditorView, controller: VimKeybindingController) {
  const line = view.state.doc.lineAt(view.state.selection.main.head);
  view.dispatch({
    changes: { from: line.from, to: line.to, insert: "" },
    selection: EditorSelection.cursor(line.from),
    scrollIntoView: true,
  });
  return vimEnterInsertMode(view, controller);
}

function vimChangeToLineEnd(view: EditorView, controller: VimKeybindingController) {
  deleteToLineEnd(view);
  return vimEnterInsertMode(view, controller);
}

function vimApplyOperatorMotion(view: EditorView, controller: VimKeybindingController, operator: "d" | "c", motion: VimMotionKey) {
  const range = vimMotionRange(view.state.doc.toString(), view.state.selection.main.head, motion, operator);
  return vimApplyOperatorRange(view, controller, operator, range.from, range.to);
}

function vimApplyLineOperatorMotion(view: EditorView, controller: VimKeybindingController, operator: "d" | "c", motion: "0" | "$") {
  const cursor = view.state.selection.main.head;
  const line = view.state.doc.lineAt(cursor);
  const range = motion === "0" ? normalizedRange(line.from, cursor) : normalizedRange(cursor, line.to);
  return vimApplyOperatorRange(view, controller, operator, range.from, range.to);
}

function vimApplyOperatorRange(view: EditorView, controller: VimKeybindingController, operator: "d" | "c", from: number, to: number) {
  controller.setPendingOperator("");
  const docLength = view.state.doc.length;
  const start = Math.max(0, Math.min(docLength, from));
  const end = Math.max(0, Math.min(docLength, to));
  if (start !== end) {
    view.dispatch({
      changes: { from: start, to: end, insert: "" },
      selection: EditorSelection.cursor(start),
      scrollIntoView: true,
    });
  }
  if (operator === "c") return vimEnterInsertMode(view, controller);
  view.focus();
  return true;
}

function vimJoinLineWithNext(view: EditorView) {
  const line = view.state.doc.lineAt(view.state.selection.main.head);
  if (line.number >= view.state.doc.lines) return true;
  const nextLine = view.state.doc.line(line.number + 1);
  view.dispatch({
    changes: {
      from: line.to,
      to: nextLine.to,
      insert: ` ${nextLine.text.trimStart()}`,
    },
    selection: EditorSelection.cursor(line.to + 1),
    scrollIntoView: true,
  });
  view.focus();
  return true;
}

function isVimOperatorMotionKey(key: string): key is VimMotionKey {
  return key === "w" || key === "e" || key === "b";
}

function normalizedRange(from: number, to: number) {
  return from <= to ? { from, to } : { from: to, to: from };
}

function isVimWhitespace(char: string | undefined) {
  return !char || /\s/.test(char);
}
