export interface LatestDocumentTaskGate {
  sequence: number;
}

export interface LatestDocumentTaskSnapshot {
  token: number;
  documentId: string;
  text: string;
}

export interface DocumentTaskSubject {
  id: string;
  text: string;
}

export function beginLatestDocumentTask(gate: LatestDocumentTaskGate, document: DocumentTaskSubject): LatestDocumentTaskSnapshot {
  gate.sequence += 1;
  return {
    token: gate.sequence,
    documentId: document.id,
    text: document.text,
  };
}

export function cancelLatestDocumentTask(gate: LatestDocumentTaskGate): number {
  gate.sequence += 1;
  return gate.sequence;
}

export function isLatestDocumentTaskCurrent(
  gate: LatestDocumentTaskGate,
  snapshot: LatestDocumentTaskSnapshot,
  document: DocumentTaskSubject | null | undefined,
): boolean {
  return Boolean(
    document &&
      gate.sequence === snapshot.token &&
      document.id === snapshot.documentId &&
      document.text === snapshot.text,
  );
}
