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
export type VimPendingOperator = "" | "d";

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
  if (controller.pendingOperator()) {
    if (controller.pendingOperator() === "d" && event.key === "d") {
      event.preventDefault();
      return vimDeleteCurrentLine(view, controller);
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
    $: () => cursorLineEnd(view),
    x: () => deleteCharForward(view),
    D: () => deleteToLineEnd(view),
    I: () => vimInsertAtLineStart(view, controller),
    A: () => vimAppendAtLineEnd(view, controller),
    i: () => vimEnterInsertMode(view, controller),
    a: () => vimInsertAfterCursor(view, controller),
    o: () => vimOpenLineBelow(view, controller),
    O: () => vimOpenLineAbove(view, controller),
    u: () => undo(view),
    G: () => cursorDocEnd(view),
    g: () => cursorDocStart(view),
  }[event.key];
  if (run) {
    event.preventDefault();
    return run();
  }
  if (event.key === "d") {
    event.preventDefault();
    controller.setPendingOperator("d");
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

function isVimWhitespace(char: string | undefined) {
  return !char || /\s/.test(char);
}
