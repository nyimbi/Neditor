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
  const nextHistory = history.filter((entry) => entry.runId !== runId);
  return nextHistory.length === history.length ? history : nextHistory;
}

export function clearAgentRunHistoryState(history: AgentRunHistoryItem[]) {
  return history.length ? [] : history;
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
  const nextHistory = history.filter((entry) => entry.draftId !== draftId);
  return nextHistory.length === history.length ? history : nextHistory;
}

export function clearDocsLiveDraftHistoryState(history: DocsLiveDraftHistoryItem[]) {
  return history.length ? [] : history;
}

export function recordGuidedDemoStepState(stepIds: string[], stepId: string): string[] {
  const normalizedStepId = stepId.trim();
  if (!normalizedStepId || stepIds.includes(normalizedStepId)) return stepIds;
  return [...stepIds, normalizedStepId].slice(0, 40);
}

export function resetGuidedDemoProgressState(stepIds: string[]) {
  return stepIds.length ? [] : stepIds;
}
