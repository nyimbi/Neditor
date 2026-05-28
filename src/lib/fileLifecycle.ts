import type { OpenDocument } from "../types.js";

export interface FileContentResponse {
  path: string;
  text: string;
  hash: string;
  modified?: string | null;
}

export interface FileRenameMetadata {
  path: string;
  hash?: string | null;
  modified?: string | null;
}

export function titleFromPath(path: string | null) {
  if (!path) return "Untitled";
  return path.split(/[\\/]/).pop() || path;
}

export function folderFromPath(path: string | null) {
  if (!path) return null;
  const separator = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return separator > 0 ? path.slice(0, separator) : null;
}

export function createUntitledDocumentState(text: string, savedHash: string, createId: () => string, title = "Untitled"): OpenDocument {
  return {
    id: createId(),
    path: null,
    title: title.trim() || "Untitled",
    text,
    savedHash,
    savedText: text,
    dirty: true,
  };
}

export function applySavedDocumentState(document: OpenDocument, response: FileContentResponse) {
  const nextDocument: OpenDocument = {
    ...document,
    path: response.path,
    title: titleFromPath(response.path),
    text: response.text,
    savedHash: response.hash,
    savedText: response.text,
    modified: response.modified,
    dirty: false,
  };
  return {
    document: nextDocument,
    statusMessage: `Saved ${nextDocument.title}`,
  };
}

export function applyRevertedDocumentState(document: OpenDocument, response: FileContentResponse) {
  const nextDocument: OpenDocument = {
    ...document,
    text: response.text,
    savedHash: response.hash,
    savedText: response.text,
    modified: response.modified,
    dirty: false,
  };
  return {
    document: nextDocument,
    statusMessage: `Reverted ${nextDocument.title} to saved content`,
  };
}

export function applyUntitledRevertState(document: OpenDocument, text: string, savedHash: string) {
  const nextDocument: OpenDocument = {
    ...document,
    text,
    savedHash,
    savedText: text,
    dirty: true,
  };
  return {
    document: nextDocument,
    statusMessage: "Reverted untitled document to starter content",
  };
}

export function applyUpdatedDocumentTextState(document: OpenDocument, text: string, hashText: (value: string) => string) {
  const dirty = typeof document.savedText === "string" ? text !== document.savedText : hashText(text) !== document.savedHash;
  return {
    document: {
      ...document,
      text,
      dirty,
    },
  };
}

export function applyRenamedDocumentState(document: OpenDocument, metadata: FileRenameMetadata) {
  const nextDocument: OpenDocument = {
    ...document,
    path: metadata.path,
    title: titleFromPath(metadata.path),
    savedHash: metadata.hash || document.savedHash,
    modified: metadata.modified,
  };
  return {
    document: nextDocument,
    statusMessage: `Renamed ${nextDocument.title}`,
  };
}

export function createDuplicateDocumentState(response: FileContentResponse, createId: () => string): OpenDocument {
  return {
    id: createId(),
    path: response.path,
    title: titleFromPath(response.path),
    text: response.text,
    savedHash: response.hash,
    savedText: response.text,
    dirty: false,
    modified: response.modified,
  };
}

export function createOpenedDocumentState(response: FileContentResponse, createId: () => string): OpenDocument {
  return createDuplicateDocumentState(response, createId);
}
