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
export type VimPendingOperator = "" | "d" | "c" | "y";
export type VimMotionKey = "w" | "e" | "b";

export interface VimRegister {
  text: string;
  linewise: boolean;
}

export interface VimKeybindingController {
  pendingOperator(): VimPendingOperator;
  yankRegister(): VimRegister;
  setInputMode(mode: VimInputMode): void;
  setPendingOperator(operator: VimPendingOperator): void;
  setYankRegister(register: VimRegister): void;
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
      if (pendingOperator === "d") return vimDeleteCurrentLine(view, controller);
      if (pendingOperator === "c") return vimChangeCurrentLine(view, controller);
      return vimYankCurrentLine(view, controller);
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
    P: () => vimPasteBeforeCursor(view, controller),
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
    p: () => vimPasteAfterCursor(view, controller),
  }[event.key];
  if (run) {
    event.preventDefault();
    return run();
  }
  if (event.key === "d" || event.key === "c" || event.key === "y") {
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
    const startKind = vimCharacterKind(text[position]);
    while (position < text.length && vimCharacterKind(text[position]) === startKind) position += 1;
    while (position < text.length && isVimWhitespace(text[position])) position += 1;
  }
  return position;
}

export function vimWordEnd(text: string, cursor: number) {
  if (!text.length) return 0;
  let position = Math.max(0, Math.min(text.length - 1, cursor + 1));
  while (position < text.length && isVimWhitespace(text[position])) position += 1;
  const startKind = vimCharacterKind(text[position]);
  while (position < text.length - 1 && vimCharacterKind(text[position + 1]) === startKind) position += 1;
  return position;
}

export function previousVimWordStart(text: string, cursor: number) {
  let position = Math.max(0, Math.min(text.length, cursor) - 1);
  while (position > 0 && isVimWhitespace(text[position])) position -= 1;
  const startKind = vimCharacterKind(text[position]);
  while (position > 0 && vimCharacterKind(text[position - 1]) === startKind) position -= 1;
  return position;
}

export function vimMotionRange(text: string, cursor: number, motion: VimMotionKey, operator: "d" | "c" | "y" = "d") {
  const start = Math.max(0, Math.min(text.length, cursor));
  if (motion === "b") {
    return normalizedRange(previousVimWordStart(text, start), start);
  }
  if (motion === "e" || (motion === "w" && operator === "c" && !isVimWhitespace(text[start]))) {
    return normalizedRange(start, Math.min(text.length, vimWordEnd(text, start) + 1));
  }
  return normalizedRange(start, nextVimWordStart(text, start));
}

export function vimLineTextRange(text: string, cursor: number) {
  const safeCursor = Math.max(0, Math.min(text.length, cursor));
  let lineStart = text.lastIndexOf("\n", Math.max(0, safeCursor - 1)) + 1;
  let nextLineStart = text.indexOf("\n", safeCursor);
  if (nextLineStart === -1) nextLineStart = text.length;
  else nextLineStart += 1;
  if (lineStart > text.length) lineStart = text.length;
  return {
    from: lineStart,
    to: Math.max(lineStart, nextLineStart),
    text: ensureLinewiseRegisterText(text.slice(lineStart, Math.max(lineStart, nextLineStart))),
    linewise: true,
  };
}

export function vimPastePosition(text: string, cursor: number, register: VimRegister, placement: "after" | "before") {
  return vimPasteEdit(text, cursor, register, placement).position;
}

export function vimPasteEdit(text: string, cursor: number, register: VimRegister, placement: "after" | "before") {
  const safeCursor = Math.max(0, Math.min(text.length, cursor));
  if (!register.linewise) {
    return {
      position: placement === "after" ? Math.min(text.length, safeCursor + 1) : safeCursor,
      text: register.text,
    };
  }
  const registerText = ensureLinewiseRegisterText(register.text);
  if (placement === "before") {
    return {
      position: text.lastIndexOf("\n", Math.max(0, safeCursor - 1)) + 1,
      text: registerText,
    };
  }
  const lineEnd = text.indexOf("\n", safeCursor);
  if (lineEnd === -1) {
    return {
      position: text.length,
      text: text.length && !text.endsWith("\n") ? `\n${registerText}` : registerText,
    };
  }
  return {
    position: lineEnd + 1,
    text: registerText,
  };
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
  const range = vimLineTextRange(view.state.doc.toString(), view.state.selection.main.head);
  controller.setYankRegister({ text: range.text, linewise: true });
  return deleteLine(view);
}

function vimChangeCurrentLine(view: EditorView, controller: VimKeybindingController) {
  const line = view.state.doc.lineAt(view.state.selection.main.head);
  controller.setYankRegister({ text: line.text, linewise: true });
  view.dispatch({
    changes: { from: line.from, to: line.to, insert: "" },
    selection: EditorSelection.cursor(line.from),
    scrollIntoView: true,
  });
  return vimEnterInsertMode(view, controller);
}

function vimChangeToLineEnd(view: EditorView, controller: VimKeybindingController) {
  const cursor = view.state.selection.main.head;
  const line = view.state.doc.lineAt(cursor);
  controller.setYankRegister({ text: view.state.doc.sliceString(cursor, line.to), linewise: false });
  deleteToLineEnd(view);
  return vimEnterInsertMode(view, controller);
}

function vimApplyOperatorMotion(view: EditorView, controller: VimKeybindingController, operator: VimPendingOperator, motion: VimMotionKey) {
  if (operator !== "d" && operator !== "c" && operator !== "y") return false;
  const range = vimMotionRange(view.state.doc.toString(), view.state.selection.main.head, motion, operator);
  return vimApplyOperatorRange(view, controller, operator, range.from, range.to);
}

function vimApplyLineOperatorMotion(view: EditorView, controller: VimKeybindingController, operator: VimPendingOperator, motion: "0" | "$") {
  if (operator !== "d" && operator !== "c" && operator !== "y") return false;
  const cursor = view.state.selection.main.head;
  const line = view.state.doc.lineAt(cursor);
  const range = motion === "0" ? normalizedRange(line.from, cursor) : normalizedRange(cursor, line.to);
  return vimApplyOperatorRange(view, controller, operator, range.from, range.to);
}

function vimApplyOperatorRange(view: EditorView, controller: VimKeybindingController, operator: "d" | "c" | "y", from: number, to: number) {
  controller.setPendingOperator("");
  const docLength = view.state.doc.length;
  const start = Math.max(0, Math.min(docLength, from));
  const end = Math.max(0, Math.min(docLength, to));
  const yankedText = view.state.doc.sliceString(start, end);
  if (yankedText) controller.setYankRegister({ text: yankedText, linewise: false });
  if (operator === "y") {
    view.focus();
    return true;
  }
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

function vimYankCurrentLine(view: EditorView, controller: VimKeybindingController) {
  controller.setPendingOperator("");
  const range = vimLineTextRange(view.state.doc.toString(), view.state.selection.main.head);
  controller.setYankRegister({ text: range.text, linewise: true });
  view.focus();
  return true;
}

function vimPasteAfterCursor(view: EditorView, controller: VimKeybindingController) {
  return vimPasteRegister(view, controller, "after");
}

function vimPasteBeforeCursor(view: EditorView, controller: VimKeybindingController) {
  return vimPasteRegister(view, controller, "before");
}

function vimPasteRegister(view: EditorView, controller: VimKeybindingController, placement: "after" | "before") {
  const register = controller.yankRegister();
  if (!register.text) return true;
  const paste = vimPasteEdit(view.state.doc.toString(), view.state.selection.main.head, register, placement);
  view.dispatch({
    changes: { from: paste.position, to: paste.position, insert: paste.text },
    selection: EditorSelection.cursor(paste.position + paste.text.length),
    scrollIntoView: true,
  });
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

function ensureLinewiseRegisterText(text: string) {
  return text.endsWith("\n") ? text : `${text}\n`;
}

function vimCharacterKind(char: string | undefined) {
  if (isVimWhitespace(char)) return "space";
  return /[A-Za-z0-9_]/.test(char || "") ? "word" : "punctuation";
}

function isVimWhitespace(char: string | undefined) {
  return !char || /\s/.test(char);
}
