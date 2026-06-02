import type { OpenDocument } from "../types.js";
import { titleFromPath } from "./fileLifecycle.js";

export interface ConflictDiffRow {
  key: string;
  kind: "equal" | "local" | "external";
  local: string;
  external: string;
  localLine: number | null;
  externalLine: number | null;
}

export type ExternalConflictReason = "root" | "include";

export interface ExternalConflictState {
  documentId: string;
  path: string;
  reason: ExternalConflictReason;
  message: string;
  externalHash: string;
  externalText: string;
}

export interface ExternalConflictFileResponse {
  path?: string;
  text: string;
  hash: string;
  modified?: string | null;
}

export function createExternalConflictState(
  document: Pick<OpenDocument, "id">,
  path: string,
  reason: ExternalConflictReason,
  message: string,
  externalHash: string,
  externalText: string,
): ExternalConflictState {
  return {
    documentId: document.id,
    path,
    reason,
    message,
    externalHash,
    externalText,
  };
}

export function acceptExternalRootConflictState<T extends OpenDocument>(
  document: T,
  response: ExternalConflictFileResponse,
) {
  return {
    document: {
      ...document,
      text: response.text,
      savedHash: response.hash,
      savedText: response.text,
      modified: response.modified,
      dirty: false,
    },
    externalConflict: null,
    statusMessage: "Accepted external file changes",
  };
}

export function applyExternalRootReloadState<T extends OpenDocument>(
  document: T,
  response: ExternalConflictFileResponse,
  activeDocumentId: string,
) {
  const nextPath = response.path || document.path;
  return {
    document: {
      ...document,
      path: nextPath,
      title: titleFromPath(nextPath),
      text: response.text,
      savedHash: response.hash,
      savedText: response.text,
      modified: response.modified,
      dirty: false,
    },
    externalConflict: null,
    statusMessage:
      document.id === activeDocumentId
        ? "Reloaded external changes"
        : `Reloaded external changes for ${titleFromPath(nextPath)}`,
  };
}

export function keepLocalRootConflictState<T extends OpenDocument>(document: T, conflict: ExternalConflictState) {
  return {
    document: {
      ...document,
      savedHash: conflict.externalHash,
      savedText: conflict.externalText || document.savedText,
    },
    externalHash: conflict.externalHash,
    externalConflict: null,
    statusMessage: "Keeping local edits",
  };
}

export function applyRootConflictMergeState<T extends OpenDocument>(
  document: T,
  conflict: ExternalConflictState,
  text: string,
) {
  const externalText = conflict.externalText;
  return {
    document: {
      ...document,
      text,
      savedHash: conflict.externalHash,
      savedText: externalText,
      dirty: text !== externalText,
    },
    externalHash: conflict.externalHash,
    externalConflict: null,
    statusMessage: "Merged external changes into the working document",
  };
}

export function buildConflictDiff(localText: string, externalText: string): ConflictDiffRow[] {
  const localLines = localText.split(/\r?\n/);
  const externalLines = externalText.split(/\r?\n/);
  if (localLines.length * externalLines.length > 250_000) {
    return [
      {
        key: "large-conflict",
        kind: "local",
        local: `${localLines.length} local lines`,
        external: `${externalLines.length} external lines`,
        localLine: null,
        externalLine: null,
      },
    ];
  }

  const scores = Array.from({ length: localLines.length + 1 }, () => Array(externalLines.length + 1).fill(0));
  for (let localIndex = localLines.length - 1; localIndex >= 0; localIndex -= 1) {
    for (let externalIndex = externalLines.length - 1; externalIndex >= 0; externalIndex -= 1) {
      scores[localIndex][externalIndex] =
        localLines[localIndex] === externalLines[externalIndex]
          ? scores[localIndex + 1][externalIndex + 1] + 1
          : Math.max(scores[localIndex + 1][externalIndex], scores[localIndex][externalIndex + 1]);
    }
  }

  const rows: ConflictDiffRow[] = [];
  let localIndex = 0;
  let externalIndex = 0;
  while (localIndex < localLines.length || externalIndex < externalLines.length) {
    const key = `${localIndex}:${externalIndex}:${rows.length}`;
    if (localIndex < localLines.length && externalIndex < externalLines.length && localLines[localIndex] === externalLines[externalIndex]) {
      rows.push({
        key,
        kind: "equal",
        local: localLines[localIndex],
        external: externalLines[externalIndex],
        localLine: localIndex + 1,
        externalLine: externalIndex + 1,
      });
      localIndex += 1;
      externalIndex += 1;
    } else if (localIndex >= localLines.length) {
      rows.push({
        key,
        kind: "external",
        local: "",
        external: externalLines[externalIndex],
        localLine: null,
        externalLine: externalIndex + 1,
      });
      externalIndex += 1;
    } else if (externalIndex >= externalLines.length || scores[localIndex + 1][externalIndex] >= scores[localIndex][externalIndex + 1]) {
      rows.push({
        key,
        kind: "local",
        local: localLines[localIndex],
        external: "",
        localLine: localIndex + 1,
        externalLine: null,
      });
      localIndex += 1;
    } else {
      rows.push({
        key,
        kind: "external",
        local: "",
        external: externalLines[externalIndex],
        localLine: null,
        externalLine: externalIndex + 1,
      });
      externalIndex += 1;
    }
  }
  return rows;
}
