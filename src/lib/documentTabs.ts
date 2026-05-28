import { forgetRecentItem, rememberRecentItem } from "./recentItems.js";
import type { OpenDocument } from "../types.js";

export type DocumentTab = Pick<OpenDocument, "id" | "path" | "title" | "pinned">;
export type TabPlacement = "before" | "after";

export interface CloseDocumentTabResult<T extends DocumentTab> {
  documents: T[];
  activeId: string;
  recentlyClosed: string[];
  closedActiveDocument: boolean;
}

export interface SetPinnedDocumentResult<T extends DocumentTab> {
  documents: T[];
  statusMessage: string;
}

export interface MoveDocumentTabResult<T extends DocumentTab> {
  documents: T[];
  statusMessage: string;
}

export function closeDocumentTabState<T extends DocumentTab>(
  documents: T[],
  activeId: string,
  recentlyClosed: string[],
  id: string,
): CloseDocumentTabResult<T> | null {
  if (documents.length === 1) return null;
  const index = documents.findIndex((document) => document.id === id);
  if (index < 0) return null;
  const closed = documents[index];
  const nextDocuments = documents.filter((document) => document.id !== id);
  const closedActiveDocument = closed.id === activeId;
  return {
    documents: nextDocuments,
    activeId: closedActiveDocument ? nextDocuments[Math.max(0, index - 1)].id : activeId,
    recentlyClosed: closed.path ? rememberRecentItem(recentlyClosed, closed.path, 20) : recentlyClosed,
    closedActiveDocument,
  };
}

export function setPinnedDocumentState<T extends DocumentTab>(
  documents: T[],
  id: string,
  pinned: boolean,
): SetPinnedDocumentResult<T> | null {
  const document = documents.find((item) => item.id === id);
  if (!document) return null;
  const nextDocuments = documents
    .map((item) => (item.id === id ? { ...item, pinned } : item))
    .sort((left, right) => Number(Boolean(right.pinned)) - Number(Boolean(left.pinned)));
  return {
    documents: nextDocuments,
    statusMessage: pinned ? `Pinned ${document.title}` : `Unpinned ${document.title}`,
  };
}

export function moveDocumentTabState<T extends DocumentTab>(
  documents: T[],
  id: string,
  targetId: string,
  placement: TabPlacement,
): MoveDocumentTabResult<T> | null {
  if (id === targetId) return null;
  const fromIndex = documents.findIndex((document) => document.id === id);
  const moving = documents[fromIndex];
  if (fromIndex < 0 || !moving) return null;
  const withoutMoving = documents.filter((document) => document.id !== id);
  const targetIndex = withoutMoving.findIndex((document) => document.id === targetId);
  if (targetIndex < 0) return null;
  const insertionIndex = placement === "before" ? targetIndex : targetIndex + 1;
  const nextDocuments = withoutMoving.slice();
  nextDocuments.splice(insertionIndex, 0, moving);
  return {
    documents: nextDocuments,
    statusMessage: `Moved ${moving.title} tab ${placement} target`,
  };
}

export function forgetDocumentPathState(
  recentFiles: string[],
  recentlyClosed: string[],
  missingWorkspaceFiles: string[],
  path: string | null,
) {
  if (!path) return { recentFiles, recentlyClosed, missingWorkspaceFiles };
  return {
    recentFiles: forgetRecentItem(recentFiles, path),
    recentlyClosed: forgetRecentItem(recentlyClosed, path),
    missingWorkspaceFiles: forgetRecentItem(missingWorkspaceFiles, path),
  };
}
