import { forgetRecentItem } from "./recentItems.js";
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
