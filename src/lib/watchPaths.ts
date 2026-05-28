export type WatchPathRole = "root" | "include";

export interface WatchContextState {
  documentId: string;
  rootPath: string;
  openRootPaths: string[];
  includedPaths: string[];
  signature: string;
}

interface WatchDocumentReference {
  id: string;
  path: string | null;
}

interface WatchPathRoleInput {
  path: string;
  role?: string | null;
}

export function normalizeWatchPath(path?: string | null): string {
  const normalized = (path || "").replace(/\\/g, "/").replace(/\/+$/, "");
  return /^[a-z]:/i.test(normalized) ? normalized.toLowerCase() : normalized;
}

export function sameWatchPath(left?: string | null, right?: string | null): boolean {
  return normalizeWatchPath(left) === normalizeWatchPath(right);
}

export function buildWatchedPathRoles(files: WatchPathRoleInput[]): Record<string, WatchPathRole> {
  return files.reduce<Record<string, WatchPathRole>>((roles, file) => {
    const role = file.role === "root" ? "root" : "include";
    roles[file.path] = role;
    roles[normalizeWatchPath(file.path)] = role;
    return roles;
  }, {});
}

export function resolveWatchReason(
  path: string,
  rootPath: string,
  includedPaths: string[],
  watchedPathRoles: Record<string, WatchPathRole>,
): WatchPathRole | null {
  const normalizedPath = normalizeWatchPath(path);
  return (
    watchedPathRoles[path] ||
    watchedPathRoles[normalizedPath] ||
    (sameWatchPath(path, rootPath)
      ? "root"
      : includedPaths.some((includedPath) => sameWatchPath(path, includedPath))
        ? "include"
        : null)
  );
}

export function isCurrentWatchContext(
  context: WatchContextState,
  currentContext: WatchContextState | null,
  activeDocument: WatchDocumentReference | null | undefined,
): boolean {
  if (!currentContext || currentContext.signature !== context.signature) return false;
  return Boolean(activeDocument?.id === context.documentId && activeDocument.path && sameWatchPath(activeDocument.path, context.rootPath));
}

export function documentForWatchContext<T extends WatchDocumentReference>(
  documents: T[],
  context: WatchContextState,
): T | null {
  const document = documents.find((candidate) => candidate.id === context.documentId);
  if (!document?.path || !sameWatchPath(document.path, context.rootPath)) return null;
  return document;
}

export function documentForWatchedRoot<T extends WatchDocumentReference>(
  documents: T[],
  path: string,
  context: WatchContextState,
): T | null {
  const contextDocument = documentForWatchContext(documents, context);
  if (contextDocument?.path && sameWatchPath(contextDocument.path, path)) return contextDocument;
  return documents.find((document) => Boolean(document.path) && sameWatchPath(document.path, path)) || null;
}

export function watchedPathsForContext(watchedPaths: string[], context: Pick<WatchContextState, "rootPath" | "includedPaths">): string[] {
  return watchedPaths.length ? watchedPaths : [context.rootPath, ...context.includedPaths];
}
