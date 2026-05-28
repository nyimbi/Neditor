import type { OpenDocument } from "../types.js";
import { titleFromPath } from "./fileLifecycle.js";
import { forgetRecentItem, rememberRecentItem } from "./recentItems.js";
import { clampScrollRatio } from "./workspacePersistence.js";

export interface DocumentScrollState {
  id: string;
  editorScrollRatio?: number;
  previewScrollRatio?: number;
}

export interface SetDocumentScrollResult<T extends DocumentScrollState> {
  documents: T[];
  changed: boolean;
}

export interface ForgetWorkspaceFolderResult<T> {
  recentFolders: string[];
  workspaceRoot: string | null;
  workspaceFiles: T[];
  changed: boolean;
}

export interface WorkspaceRestoreFileResponse {
  path: string;
  text: string;
  hash: string;
  modified?: string | null;
}

export interface WorkspaceRestoreStateResult {
  documents: OpenDocument[];
  activeId: string;
  recentFiles: string[];
  recentlyClosed: string[];
  missingWorkspaceFiles: string[];
  statusMessage?: string;
  persistRequired: boolean;
}

export interface OpenedWorkspaceDocumentStateResult {
  documents: OpenDocument[];
  activeId: string;
  recentFiles: string[];
  recentlyClosed: string[];
  missingWorkspaceFiles: string[];
  statusMessage: string;
}

export interface OpenWorkspaceFolderSuccessStateResult {
  recentFolders: string[];
  sidebar: "files";
  statusMessage: string;
}

export interface OpenWorkspaceFolderFailureStateResult<T> {
  workspaceRoot: string | null;
  workspaceFiles: T[];
  statusMessage: string;
}

export interface OpenRecentWorkspaceFolderFailureStateResult<T> extends ForgetWorkspaceFolderResult<T> {
  statusMessage: string;
}

export function setDocumentScrollState<T extends DocumentScrollState>(
  documents: T[],
  id: string,
  scroll: { editor?: number; preview?: number },
): SetDocumentScrollResult<T> {
  let changed = false;
  const nextDocuments = documents.map((document) => {
    if (document.id !== id) return document;
    const nextDocument = { ...document };
    if (typeof scroll.editor === "number") {
      const nextEditorRatio = clampScrollRatio(scroll.editor);
      changed = changed || nextDocument.editorScrollRatio !== nextEditorRatio;
      nextDocument.editorScrollRatio = nextEditorRatio;
    }
    if (typeof scroll.preview === "number") {
      const nextPreviewRatio = clampScrollRatio(scroll.preview);
      changed = changed || nextDocument.previewScrollRatio !== nextPreviewRatio;
      nextDocument.previewScrollRatio = nextPreviewRatio;
    }
    return nextDocument;
  });
  return { documents: changed ? nextDocuments : documents, changed };
}

export function forgetWorkspaceFolderState<T>(
  recentFolders: string[],
  workspaceRoot: string | null,
  workspaceFiles: T[],
  path: string | null,
): ForgetWorkspaceFolderResult<T> {
  if (!path) {
    return { recentFolders, workspaceRoot, workspaceFiles, changed: false };
  }
  const nextRecentFolders = forgetRecentItem(recentFolders, path);
  const rootMatched = workspaceRoot === path;
  return {
    recentFolders: nextRecentFolders,
    workspaceRoot: rootMatched ? null : workspaceRoot,
    workspaceFiles: rootMatched ? [] : workspaceFiles,
    changed: rootMatched || nextRecentFolders.length !== recentFolders.length,
  };
}

export function applyOpenWorkspaceFolderSuccessState(recentFolders: string[], path: string): OpenWorkspaceFolderSuccessStateResult {
  return {
    recentFolders: rememberRecentItem(recentFolders, path, 12),
    sidebar: "files",
    statusMessage: `Opened workspace ${titleFromPath(path)}`,
  };
}

export function applyOpenWorkspaceFolderFailureState<T>(
  previousRoot: string | null,
  previousFiles: T[],
  path: string,
): OpenWorkspaceFolderFailureStateResult<T> {
  return {
    workspaceRoot: previousRoot,
    workspaceFiles: previousFiles,
    statusMessage: `Could not open workspace ${titleFromPath(path)}`,
  };
}

export function applyOpenRecentWorkspaceFolderFailureState<T>(
  recentFolders: string[],
  workspaceRoot: string | null,
  workspaceFiles: T[],
  path: string,
): OpenRecentWorkspaceFolderFailureStateResult<T> {
  return {
    ...forgetWorkspaceFolderState(recentFolders, workspaceRoot, workspaceFiles, path),
    statusMessage: `Removed missing recent folder ${titleFromPath(path)}`,
  };
}

export function createRestoredWorkspaceDocumentState(
  response: WorkspaceRestoreFileResponse,
  requestedPath: string,
  pinnedFiles: string[],
  scrollPositions: Record<string, { editor?: number; preview?: number }>,
  createId: () => string,
): OpenDocument {
  const scrollPosition = scrollPositions[response.path] || scrollPositions[requestedPath] || {};
  return {
    id: createId(),
    path: response.path,
    title: titleFromPath(response.path),
    text: response.text,
    savedHash: response.hash,
    savedText: response.text,
    dirty: false,
    pinned: pinnedFiles.includes(response.path),
    modified: response.modified,
    editorScrollRatio: clampScrollRatio(scrollPosition.editor),
    previewScrollRatio: clampScrollRatio(scrollPosition.preview),
  };
}

export function applyWorkspaceRestoreState(
  currentDocuments: OpenDocument[],
  currentActiveId: string,
  recentFiles: string[],
  recentlyClosed: string[],
  restored: OpenDocument[],
  missing: string[],
  activePath: string | null,
): WorkspaceRestoreStateResult {
  const nextRecentFiles = missing.reduce((items, path) => forgetRecentItem(items, path), recentFiles);
  const nextRecentlyClosed = missing.reduce((items, path) => forgetRecentItem(items, path), recentlyClosed);
  const missingStatus = missing.length
    ? `${missing.length} restored ${missing.length === 1 ? "document was" : "documents were"} missing`
    : undefined;
  if (!restored.length) {
    return {
      documents: currentDocuments,
      activeId: currentActiveId,
      recentFiles: nextRecentFiles,
      recentlyClosed: nextRecentlyClosed,
      missingWorkspaceFiles: missing,
      statusMessage: missingStatus,
      persistRequired: Boolean(missing.length),
    };
  }
  return {
    documents: restored,
    activeId: restored.find((document) => document.path === activePath)?.id || restored[0].id,
    recentFiles: nextRecentFiles,
    recentlyClosed: nextRecentlyClosed,
    missingWorkspaceFiles: missing,
    statusMessage: missingStatus,
    persistRequired: Boolean(missing.length),
  };
}

export function applyOpenedWorkspaceDocumentState(
  documents: OpenDocument[],
  recentFiles: string[],
  recentlyClosed: string[],
  missingWorkspaceFiles: string[],
  document: OpenDocument,
): OpenedWorkspaceDocumentStateResult {
  return {
    documents: [...documents, document],
    activeId: document.id,
    recentFiles: rememberRecentItem(recentFiles, document.path, 20),
    recentlyClosed: forgetRecentItem(recentlyClosed, document.path),
    missingWorkspaceFiles: missingWorkspaceFiles.filter((missing) => missing !== document.path),
    statusMessage: `Opened ${document.title}`,
  };
}
