import { activeDocumentState } from "./documentSelectors.js";
import { normalizeBusinessProfile } from "./businessDocuments.js";
import { normalizeGoogleIntegrationPreferences } from "./googleAuth.js";
import { normalizeCustomTransformTemplates } from "./transformTemplates.js";
import { applyPersistedUiPreferences } from "./uiPreferences.js";
import {
  clampScrollRatio,
  normalizeAgentRunHistory,
  normalizeAiCleanupDefaults,
  normalizeAiProviderDefaults,
  normalizeBibliographyDefaults,
  normalizeBrandProfileDefaults,
  normalizeDocsLiveDraftHistory,
  normalizeExportDefaults,
  normalizeExportProfiles,
  normalizeGitIntegrationPreferences,
  normalizePersistedWorkspaceForSave,
  normalizeTtsPreferences,
  type PersistedScrollPosition,
  type PersistedWorkspace,
} from "./workspacePersistence.js";
import type { OpenDocument } from "../types.js";

type RequiredPersistedValue<K extends keyof PersistedWorkspace> = NonNullable<PersistedWorkspace[K]>;

export interface WorkspacePersistenceStateInput {
  documents: OpenDocument[];
  activeId: string;
  theme: RequiredPersistedValue<"theme">;
  previewTheme: RequiredPersistedValue<"previewTheme">;
  toolbarDisplay: RequiredPersistedValue<"toolbarDisplay">;
  toolbarTextSize: number;
  toolbarCollapsedRows: string[];
  editorPaneRatio: number;
  splitSourcePanes: boolean;
  editorKeymapMode: RequiredPersistedValue<"editorKeymapMode">;
  wordWrap: boolean;
  lineNumbers: boolean;
  codeFolding: boolean;
  highContrast: boolean;
  reducedMotion: boolean;
  autosave: boolean;
  autosaveDelayMs: number;
  autoSnapshot: boolean;
  snapshotIntervalMs: number;
  snapshotStorage: RequiredPersistedValue<"snapshotStorage">;
  editorFont: string;
  previewFont: string;
  editorFontSize: number;
  previewFontSize: number;
  editorLineHeight: number;
  previewLineHeight: number;
  exportTarget: RequiredPersistedValue<"exportTarget">;
  exportDefaults: RequiredPersistedValue<"exportDefaults">;
  bibliographyDefaults: RequiredPersistedValue<"bibliographyDefaults">;
  brandProfileDefaults: RequiredPersistedValue<"brandProfileDefaults">;
  businessProfile: RequiredPersistedValue<"businessProfile">;
  aiProviderDefaults: RequiredPersistedValue<"aiProviderDefaults">;
  googleIntegration: RequiredPersistedValue<"googleIntegration">;
  ttsPreferences: RequiredPersistedValue<"ttsPreferences">;
  exportProfiles: RequiredPersistedValue<"exportProfiles">;
  activeExportProfileId: string;
  gitIntegration: RequiredPersistedValue<"gitIntegration">;
  aiCleanupDefaults: RequiredPersistedValue<"aiCleanupDefaults">;
  agentRunHistory: RequiredPersistedValue<"agentRunHistory">;
  docsLiveDraftHistory: RequiredPersistedValue<"docsLiveDraftHistory">;
  guidedDemoCompletedStepIds: string[];
  recentFiles: string[];
  recentFolders: string[];
  recentlyClosed: string[];
  workspaceRoot: string | null;
  mode: RequiredPersistedValue<"mode">;
  sidebar: RequiredPersistedValue<"sidebar">;
  transformEnginePaths: Record<string, string>;
  trustedTransformEngines: Record<string, boolean>;
  disabledTransformEngines: Record<string, boolean>;
  transformInputModes: RequiredPersistedValue<"transformInputModes">;
  transformTimeoutMs: number;
  customTransformTemplates: RequiredPersistedValue<"customTransformTemplates">;
}

export type WorkspacePreferenceStateInput = Omit<WorkspacePersistenceStateInput, "documents" | "activeId">;

export interface WorkspaceRestoreRequest {
  openFiles: string[];
  activePath: string | null;
  pinnedFiles: string[];
  scrollPositions: Record<string, PersistedScrollPosition>;
}

export interface PersistedWorkspacePreferenceResult {
  state: WorkspacePreferenceStateInput;
  restoreRequest: WorkspaceRestoreRequest | null;
}

export function applyPersistedWorkspacePreferenceState(
  current: WorkspacePreferenceStateInput,
  persisted: PersistedWorkspace,
): PersistedWorkspacePreferenceResult {
  const exportProfiles = normalizeExportProfiles(persisted.exportProfiles);
  const state: WorkspacePreferenceStateInput = {
    ...current,
    ...applyPersistedUiPreferences(current, persisted),
    exportTarget: persisted.exportTarget || current.exportTarget,
    exportDefaults: persisted.exportDefaults ? normalizeExportDefaults(persisted.exportDefaults) : current.exportDefaults,
    bibliographyDefaults: persisted.bibliographyDefaults
      ? normalizeBibliographyDefaults(persisted.bibliographyDefaults)
      : current.bibliographyDefaults,
    brandProfileDefaults: persisted.brandProfileDefaults ? normalizeBrandProfileDefaults(persisted.brandProfileDefaults) : current.brandProfileDefaults,
    businessProfile: normalizeBusinessProfile(persisted.businessProfile),
    exportProfiles,
    activeExportProfileId:
      persisted.activeExportProfileId && exportProfiles.some((profile) => profile.id === persisted.activeExportProfileId)
        ? persisted.activeExportProfileId
        : "",
    gitIntegration: persisted.gitIntegration ? normalizeGitIntegrationPreferences(persisted.gitIntegration) : current.gitIntegration,
    aiCleanupDefaults: persisted.aiCleanupDefaults ? normalizeAiCleanupDefaults(persisted.aiCleanupDefaults) : current.aiCleanupDefaults,
    aiProviderDefaults: normalizeAiProviderDefaults(persisted.aiProviderDefaults),
    googleIntegration: normalizeGoogleIntegrationPreferences(persisted.googleIntegration),
    ttsPreferences: normalizeTtsPreferences(persisted.ttsPreferences),
    agentRunHistory: normalizeAgentRunHistory(persisted.agentRunHistory),
    docsLiveDraftHistory: normalizeDocsLiveDraftHistory(persisted.docsLiveDraftHistory),
    guidedDemoCompletedStepIds: persisted.guidedDemoCompletedStepIds || [],
    recentFiles: persisted.recentFiles || [],
    recentFolders: persisted.recentFolders || [],
    recentlyClosed: persisted.recentlyClosed || [],
    workspaceRoot: persisted.workspaceRoot || null,
    transformEnginePaths: persisted.transformEnginePaths || {},
    trustedTransformEngines: persisted.trustedTransformEngines || {},
    disabledTransformEngines: persisted.disabledTransformEngines || {},
    transformInputModes: persisted.transformInputModes || {},
    transformTimeoutMs:
      typeof persisted.transformTimeoutMs === "number" ? Math.min(Math.max(persisted.transformTimeoutMs, 1), 30000) : current.transformTimeoutMs,
    customTransformTemplates: normalizeCustomTransformTemplates(persisted.customTransformTemplates),
  };
  const restoreRequest = persisted.openFiles?.length
    ? {
        openFiles: persisted.openFiles,
        activePath: persisted.activePath || null,
        pinnedFiles: persisted.pinnedFiles || [],
        scrollPositions: persisted.scrollPositions || {},
      }
    : null;
  return { state, restoreRequest };
}

export function buildPersistedWorkspaceState(state: WorkspacePersistenceStateInput): PersistedWorkspace {
  const activeDocument = activeDocumentState(state.documents, state.activeId) || state.documents[0] || null;
  return normalizePersistedWorkspaceForSave({
    theme: state.theme,
    previewTheme: state.previewTheme,
    toolbarDisplay: state.toolbarDisplay,
    toolbarTextSize: state.toolbarTextSize,
    toolbarCollapsedRows: state.toolbarCollapsedRows,
    editorPaneRatio: state.editorPaneRatio,
    splitSourcePanes: state.splitSourcePanes,
    editorKeymapMode: state.editorKeymapMode,
    wordWrap: state.wordWrap,
    lineNumbers: state.lineNumbers,
    codeFolding: state.codeFolding,
    highContrast: state.highContrast,
    reducedMotion: state.reducedMotion,
    autosave: state.autosave,
    autosaveDelayMs: state.autosaveDelayMs,
    autoSnapshot: state.autoSnapshot,
    snapshotIntervalMs: state.snapshotIntervalMs,
    snapshotStorage: state.snapshotStorage,
    editorFont: state.editorFont,
    previewFont: state.previewFont,
    editorFontSize: state.editorFontSize,
    previewFontSize: state.previewFontSize,
    editorLineHeight: state.editorLineHeight,
    previewLineHeight: state.previewLineHeight,
    exportTarget: state.exportTarget,
    exportDefaults: state.exportDefaults,
    bibliographyDefaults: state.bibliographyDefaults,
    brandProfileDefaults: state.brandProfileDefaults,
    businessProfile: state.businessProfile,
    aiProviderDefaults: state.aiProviderDefaults,
    googleIntegration: state.googleIntegration,
    ttsPreferences: state.ttsPreferences,
    exportProfiles: state.exportProfiles,
    activeExportProfileId: state.activeExportProfileId,
    gitIntegration: state.gitIntegration,
    aiCleanupDefaults: state.aiCleanupDefaults,
    agentRunHistory: state.agentRunHistory,
    docsLiveDraftHistory: state.docsLiveDraftHistory,
    guidedDemoCompletedStepIds: state.guidedDemoCompletedStepIds,
    recentFiles: state.recentFiles.slice(0, 20),
    recentFolders: state.recentFolders.slice(0, 12),
    recentlyClosed: state.recentlyClosed.slice(0, 20),
    workspaceRoot: state.workspaceRoot,
    mode: state.mode,
    sidebar: state.sidebar,
    openFiles: state.documents.map((document) => document.path).filter((path): path is string => Boolean(path)),
    scrollPositions: Object.fromEntries(
      state.documents
        .filter((document) => document.path)
        .map((document) => [
          document.path as string,
          {
            editor: clampScrollRatio(document.editorScrollRatio),
            preview: clampScrollRatio(document.previewScrollRatio),
          },
        ]),
    ),
    pinnedFiles: state.documents.filter((document) => document.pinned && document.path).map((document) => document.path as string),
    activePath: activeDocument?.path || null,
    transformEnginePaths: state.transformEnginePaths,
    trustedTransformEngines: state.trustedTransformEngines,
    disabledTransformEngines: state.disabledTransformEngines,
    transformInputModes: state.transformInputModes,
    transformTimeoutMs: state.transformTimeoutMs,
    customTransformTemplates: state.customTransformTemplates,
  });
}
