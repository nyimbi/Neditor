import { activeDocumentState } from "./documentSelectors.js";
import {
  clampScrollRatio,
  normalizePersistedWorkspaceForSave,
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
