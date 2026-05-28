export interface DocumentSelectionItem {
  id: string;
  title: string;
  dirty: boolean;
}

export interface TransformEngineSelectionItem {
  requiresExecution?: boolean;
}

export function activeDocumentState<T extends DocumentSelectionItem>(documents: T[], activeId: string): T | null {
  return documents.find((document) => document.id === activeId) || documents[0] || null;
}

export function windowTitleState(document: Pick<DocumentSelectionItem, "title" | "dirty"> | null, appName = "NEditor"): string {
  const title = document?.title || "Untitled";
  return `${document?.dirty ? "* " : ""}${title} - ${appName}`;
}

export function externalTransformEnginesState<T extends TransformEngineSelectionItem>(engines: T[]): T[] {
  return engines.filter((engine) => engine.requiresExecution);
}
