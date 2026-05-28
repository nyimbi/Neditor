import {
  normalizeAgentRunHistory,
  normalizeDocsLiveDraftHistory,
  type AgentRunHistoryItem,
  type DocsLiveDraftHistoryItem,
} from "./workspacePersistence.js";

export function recordAgentRunHistoryState(
  history: AgentRunHistoryItem[],
  item: AgentRunHistoryItem,
): AgentRunHistoryItem[] {
  return normalizeAgentRunHistory([item, ...history.filter((entry) => entry.runId !== item.runId)]);
}

export function removeAgentRunHistoryState(history: AgentRunHistoryItem[], runId: string): AgentRunHistoryItem[] {
  return history.filter((entry) => entry.runId !== runId);
}

export function recordDocsLiveDraftHistoryState(
  history: DocsLiveDraftHistoryItem[],
  item: DocsLiveDraftHistoryItem,
): DocsLiveDraftHistoryItem[] {
  return normalizeDocsLiveDraftHistory([item, ...history.filter((entry) => entry.draftId !== item.draftId)]);
}

export function removeDocsLiveDraftHistoryState(
  history: DocsLiveDraftHistoryItem[],
  draftId: string,
): DocsLiveDraftHistoryItem[] {
  return history.filter((entry) => entry.draftId !== draftId);
}

export function recordGuidedDemoStepState(stepIds: string[], stepId: string): string[] {
  const normalizedStepId = stepId.trim();
  if (!normalizedStepId || stepIds.includes(normalizedStepId)) return stepIds;
  return [...stepIds, normalizedStepId].slice(0, 40);
}
