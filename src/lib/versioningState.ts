import type { GitHistoryEntry, GitStatus, OpenDocument } from "../types.js";

export interface VersioningFileResponse {
  text: string;
  hash: string;
  modified?: string | null;
}

export function snapshotTextForDocument(document: Pick<OpenDocument, "dirty" | "text" | "savedText">): string {
  return document.dirty ? document.text : document.savedText || document.text;
}

export function applyRestoredSnapshotState(document: OpenDocument, response: VersioningFileResponse, snapshotPath: string) {
  return {
    document: {
      ...document,
      text: response.text,
      dirty: true,
    } satisfies OpenDocument,
    statusMessage: `Restored snapshot ${snapshotPath}`,
  };
}

export function applyRestoredGitRevisionState(document: OpenDocument, response: VersioningFileResponse, revision: string) {
  return {
    document: {
      ...document,
      text: response.text,
      savedHash: response.hash,
      savedText: undefined,
      dirty: true,
    } satisfies OpenDocument,
    statusMessage: `Restored revision ${revision.slice(0, 12)}`,
  };
}

export function clearGitVersioningState() {
  return {
    gitStatus: null as GitStatus | null,
    gitHistory: [] as GitHistoryEntry[],
    gitDiffText: "",
  };
}

export function gitStatusDetailsRequired(status: GitStatus, activePath?: string | null): boolean {
  return Boolean(status.inside_repo && activePath);
}
