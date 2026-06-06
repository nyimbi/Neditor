import { activeDocumentState } from "./documentSelectors.js";
import {
  normalizeBusinessProfile,
  normalizeCustomBusinessSnippets,
  normalizeCustomDocumentOutlineTemplates,
  normalizeCustomVersionedClauses,
} from "./businessDocuments.js";
import { normalizeDatabaseProfiles, type DatabaseProfile } from "./databaseProfiles.js";
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
  normalizeCustomLatexTemplateProfiles,
  normalizeDocsLiveDraftHistory,
  normalizeExportDefaults,
  normalizeExportProfiles,
  normalizeGitIntegrationPreferences,
  normalizePersistedWorkspaceForSave,
  normalizePublishingDestinationProfiles,
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
  publishingDestinationProfiles: RequiredPersistedValue<"publishingDestinationProfiles">;
  activePublishingDestinationId: string;
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
  databaseProfiles: DatabaseProfile[];
  activeDatabaseProfileId: string;
  customLatexTemplates: RequiredPersistedValue<"customLatexTemplates">;
  customBusinessSnippets: RequiredPersistedValue<"customBusinessSnippets">;
  customDocumentOutlineTemplates: RequiredPersistedValue<"customDocumentOutlineTemplates">;
  customVersionedClauses: RequiredPersistedValue<"customVersionedClauses">;
  documentMemoryText: string;
  uiMode?: 'writer' | 'pilot';
  pilotActivityPanel?: string;
  presentationTheme?: string;
  presentationTransition?: string;
  webhookConfigs: Array<{ id: string; name: string; url: string; events: string[]; enabled: boolean }>;
  auditEnabled: boolean;
  auditAuthor: string;
  auditMaxBytes: number;
  searchMaxResults: number;
  searchDefaultCaseSensitive: boolean;
  humanizerDefaultMode: "light" | "standard" | "heavy";
  compareMaxLines: number;
  compareIgnoreWhitespace: boolean;
  pandocBinaryPath: string;
  curlBinaryPath: string;
  restFetchAllowedHosts: string[];
  restFetchTimeoutMs: number;
  mailMergeRequireWorkspaceRoot: boolean;
  mailMergeMaxRecords: number;
  mailMergeDefaultDelimiter: "," | "\t";
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
  const publishingDestinationProfiles = normalizePublishingDestinationProfiles(persisted.publishingDestinationProfiles);
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
    publishingDestinationProfiles,
    activePublishingDestinationId:
      persisted.activePublishingDestinationId &&
      publishingDestinationProfiles.some((profile) => profile.id === persisted.activePublishingDestinationId)
        ? persisted.activePublishingDestinationId
        : "",
    gitIntegration: persisted.gitIntegration ? normalizeGitIntegrationPreferences(persisted.gitIntegration) : current.gitIntegration,
    aiCleanupDefaults: persisted.aiCleanupDefaults ? normalizeAiCleanupDefaults(persisted.aiCleanupDefaults) : current.aiCleanupDefaults,
    aiProviderDefaults: normalizeAiProviderDefaults(persisted.aiProviderDefaults),
    googleIntegration: normalizeGoogleIntegrationPreferences(persisted.googleIntegration),
    ttsPreferences: normalizeTtsPreferences(persisted.ttsPreferences),
    agentRunHistory: normalizeAgentRunHistory(persisted.agentRunHistory),
    docsLiveDraftHistory: normalizeDocsLiveDraftHistory(persisted.docsLiveDraftHistory),
    guidedDemoCompletedStepIds: persisted.guidedDemoCompletedStepIds || [],
    uiMode: (persisted.uiMode === 'writer' || persisted.uiMode === 'pilot') ? persisted.uiMode : undefined,
    pilotActivityPanel: typeof persisted.pilotActivityPanel === 'string' ? persisted.pilotActivityPanel : undefined,
    presentationTheme: typeof persisted.presentationTheme === "string" ? persisted.presentationTheme : current.presentationTheme,
    presentationTransition: typeof persisted.presentationTransition === "string" ? persisted.presentationTransition : current.presentationTransition,
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
    databaseProfiles: normalizeDatabaseProfiles(persisted.databaseProfiles),
    activeDatabaseProfileId: persisted.activeDatabaseProfileId || "",
    customLatexTemplates: normalizeCustomLatexTemplateProfiles(persisted.customLatexTemplates),
    customBusinessSnippets: normalizeCustomBusinessSnippets(persisted.customBusinessSnippets),
    customDocumentOutlineTemplates: normalizeCustomDocumentOutlineTemplates(persisted.customDocumentOutlineTemplates),
    customVersionedClauses: normalizeCustomVersionedClauses(persisted.customVersionedClauses),
    documentMemoryText: persisted.documentMemoryText || "",
    webhookConfigs: Array.isArray(persisted.webhookConfigs) ? persisted.webhookConfigs : current.webhookConfigs,
    auditEnabled: typeof persisted.auditEnabled === "boolean" ? persisted.auditEnabled : current.auditEnabled,
    auditAuthor: typeof persisted.auditAuthor === "string" ? persisted.auditAuthor : current.auditAuthor,
    auditMaxBytes: typeof persisted.auditMaxBytes === "number" ? persisted.auditMaxBytes : current.auditMaxBytes,
    searchMaxResults: typeof persisted.searchMaxResults === "number" ? persisted.searchMaxResults : current.searchMaxResults,
    searchDefaultCaseSensitive: typeof persisted.searchDefaultCaseSensitive === "boolean" ? persisted.searchDefaultCaseSensitive : current.searchDefaultCaseSensitive,
    humanizerDefaultMode: (persisted.humanizerDefaultMode === "light" || persisted.humanizerDefaultMode === "standard" || persisted.humanizerDefaultMode === "heavy") ? persisted.humanizerDefaultMode : current.humanizerDefaultMode,
    compareMaxLines: typeof persisted.compareMaxLines === "number" ? persisted.compareMaxLines : current.compareMaxLines,
    compareIgnoreWhitespace: typeof persisted.compareIgnoreWhitespace === "boolean" ? persisted.compareIgnoreWhitespace : current.compareIgnoreWhitespace,
    pandocBinaryPath: typeof persisted.pandocBinaryPath === "string" ? persisted.pandocBinaryPath : current.pandocBinaryPath,
    curlBinaryPath: typeof persisted.curlBinaryPath === "string" ? persisted.curlBinaryPath : current.curlBinaryPath,
    restFetchAllowedHosts: Array.isArray(persisted.restFetchAllowedHosts) ? persisted.restFetchAllowedHosts : current.restFetchAllowedHosts,
    restFetchTimeoutMs: typeof persisted.restFetchTimeoutMs === "number" ? persisted.restFetchTimeoutMs : current.restFetchTimeoutMs,
    mailMergeRequireWorkspaceRoot: typeof persisted.mailMergeRequireWorkspaceRoot === "boolean" ? persisted.mailMergeRequireWorkspaceRoot : current.mailMergeRequireWorkspaceRoot,
    mailMergeMaxRecords: typeof persisted.mailMergeMaxRecords === "number" ? persisted.mailMergeMaxRecords : current.mailMergeMaxRecords,
    mailMergeDefaultDelimiter: (persisted.mailMergeDefaultDelimiter === "," || persisted.mailMergeDefaultDelimiter === "\t") ? persisted.mailMergeDefaultDelimiter : current.mailMergeDefaultDelimiter,
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
    publishingDestinationProfiles: state.publishingDestinationProfiles,
    activePublishingDestinationId: state.activePublishingDestinationId,
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
    uiMode: state.uiMode,
    pilotActivityPanel: state.pilotActivityPanel,
    presentationTheme: state.presentationTheme,
    presentationTransition: state.presentationTransition,
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
    databaseProfiles: state.databaseProfiles,
    activeDatabaseProfileId: state.activeDatabaseProfileId,
    customLatexTemplates: state.customLatexTemplates,
    customBusinessSnippets: state.customBusinessSnippets,
    customDocumentOutlineTemplates: state.customDocumentOutlineTemplates,
    customVersionedClauses: state.customVersionedClauses,
    documentMemoryText: state.documentMemoryText,
    webhookConfigs: state.webhookConfigs,
    auditEnabled: state.auditEnabled,
    auditAuthor: state.auditAuthor,
    auditMaxBytes: state.auditMaxBytes,
    searchMaxResults: state.searchMaxResults,
    searchDefaultCaseSensitive: state.searchDefaultCaseSensitive,
    humanizerDefaultMode: state.humanizerDefaultMode,
    compareMaxLines: state.compareMaxLines,
    compareIgnoreWhitespace: state.compareIgnoreWhitespace,
    pandocBinaryPath: state.pandocBinaryPath,
    curlBinaryPath: state.curlBinaryPath,
    restFetchAllowedHosts: state.restFetchAllowedHosts,
    restFetchTimeoutMs: state.restFetchTimeoutMs,
    mailMergeRequireWorkspaceRoot: state.mailMergeRequireWorkspaceRoot,
    mailMergeMaxRecords: state.mailMergeMaxRecords,
    mailMergeDefaultDelimiter: state.mailMergeDefaultDelimiter,
  });
}
