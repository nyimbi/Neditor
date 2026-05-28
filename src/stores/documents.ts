import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { watch as watchFs, type UnwatchFn, type WatchEvent } from "@tauri-apps/plugin-fs";
import { Store } from "@tauri-apps/plugin-store";
import { beginLatestDocumentTask, cancelLatestDocumentTask, isLatestDocumentTaskCurrent } from "../lib/asyncGuards";
import { normalizeBusinessProfile, type BusinessProfile } from "../lib/businessDocuments";
import { saveAiProviderDefaultsState, saveBusinessProfileState, saveTtsPreferencesState } from "../lib/configurationProfiles";
import {
  acceptExternalRootConflictState,
  applyExternalRootReloadState,
  applyRootConflictMergeState,
  createExternalConflictState,
  keepLocalRootConflictState,
  type ExternalConflictState,
} from "../lib/conflict";
import {
  closeDocumentTabState,
  moveDocumentTabState,
  setPinnedDocumentState,
} from "../lib/documentTabs";
import { buildDocumentCompileOptions, buildDocumentExportOptions } from "../lib/documentExportOptions";
import { activeDocumentState, externalTransformEnginesState, windowTitleState } from "../lib/documentSelectors";
import { applyExportProfileState, deleteExportProfileState, saveExportProfileState } from "../lib/exportProfiles";
import {
  applyExportFailureState,
  applyExportReadinessState,
  applyExportSuccessState,
  beginExportReadinessState,
  beginExportWorkflowState,
  exportProgressState,
  finishExportWorkflowState,
} from "../lib/exportWorkflowState";
import {
  applyRevertedDocumentState,
  applyUntitledRevertState,
  applyUpdatedDocumentTextState,
  createDuplicateDocumentState,
  createOpenedDocumentState,
  createUntitledDocumentState,
  folderFromPath,
  titleFromPath,
} from "../lib/fileLifecycle";
import { isAiSourceFenceOpener, rewriteAiAssistedMarker, rewriteAiSourceReviewBlock } from "../lib/provenanceReview";
import { appendChangeNoteMarker, appendReviewCommentMarker, resolveReviewCommentAtLine } from "../lib/reviewMarkers";
import {
  applyTransformProbeFailureState,
  applyTransformProbeSuccessState,
  clampTransformTimeout,
  setTransformBooleanFlag,
  setTransformInputModeState,
  updateTransformEnginePathState,
  type TransformProbeResult,
} from "../lib/transformSettings";
import {
  applyRestoredGitRevisionState,
  applyRestoredSnapshotState,
  clearGitVersioningState,
  gitStatusDetailsRequired,
  snapshotTextForDocument,
} from "../lib/versioningState";
import {
  deleteCustomTransformTemplateState,
  saveCustomTransformTemplateState,
  type CustomTransformTemplate,
} from "../lib/transformTemplates";
import {
  buildWatchedPathRoles,
  documentForWatchedRoot as documentForWatchedRootState,
  documentForWatchContext as documentForWatchContextState,
  isCurrentWatchContext,
  resolveWatchReason,
  sameWatchPath,
  watchedPathsForContext,
  type WatchContextState,
} from "../lib/watchPaths";
import { applyAiPasteInsertion, type AiPasteInsertMode } from "../lib/workflows";
import {
  clearAgentRunHistoryState,
  clearDocsLiveDraftHistoryState,
  recordAgentRunHistoryState,
  recordDocsLiveDraftHistoryState,
  recordGuidedDemoStepState,
  removeAgentRunHistoryState,
  removeDocsLiveDraftHistoryState,
  resetGuidedDemoProgressState,
} from "../lib/workflowHistory";
import {
  applyActiveWorkspaceDocumentState,
  applyDuplicatedWorkspaceDocumentState,
  applyNewWorkspaceDocumentState,
  applyOpenedWorkspaceDocumentState,
  applyOpenRecentWorkspaceFileFailureState,
  applyOpenRecentWorkspaceFolderFailureState,
  applyOpenWorkspaceFolderFailureState,
  applyOpenWorkspaceFolderSuccessState,
  applyWorkspaceRefreshFailureState,
  applyWorkspaceRefreshSuccessState,
  applyWorkspaceRestoreState,
  applyRenamedWorkspaceDocumentState,
  applySavedWorkspaceDocumentState,
  clearWorkspaceRefreshState,
  createRestoredWorkspaceDocumentState,
  setDocumentScrollState,
} from "../lib/workspaceNavigation";
import { applyPersistedWorkspacePreferenceState, buildPersistedWorkspaceState } from "../lib/workspacePersistenceState";
import {
  clampPaneRatio,
  migratePersistedWorkspace,
  normalizeAiCleanupDefaults,
  normalizeAiProviderDefaults,
  normalizeBibliographyDefaults,
  normalizeBrandProfileDefaults,
  normalizeGitIntegrationPreferences,
  normalizeTtsPreferences,
  type DocsLiveDraftHistoryItem,
  type EditorKeymapMode,
  type ExportDefaults,
  type ExportProfile,
  type ExportTarget,
  type AgentRunHistoryItem,
  type AiProviderDefaults,
  type PersistedScrollPosition,
  type PreviewTheme,
  type SnapshotStorage,
  type ToolbarDisplay,
  type TtsPreferences,
} from "../lib/workspacePersistence";
import type {
  AiCleanupResponse,
  AiCleanupOptions,
  CompileResponse,
  DocumentDiagnostic,
  ExportProgressStep,
  ExportReadinessReport,
  GitHistoryEntry,
  GitStatus,
  OpenDocument,
  SnapshotListItem,
  TransformEngineMetadata,
  WatchFileResponse,
  WorkspaceFileEntry,
} from "../types";

let preferencesStore: Store | null = null;
let unwatchFileChanges: UnlistenFn | UnwatchFn | null = null;
let unwatchFileErrors: UnlistenFn | null = null;

interface FileMetadataResponse {
  path?: string;
  exists: boolean;
  hash?: string | null;
  modified?: string | null;
  role?: "root" | "include" | string;
}

interface DocumentWatchEvent {
  path: string;
  reason: "root" | "include";
  kind: string;
  hash?: string | null;
  modified?: string | null;
}

type WatchContext = WatchContextState;

const staleSaveConflictMessage = "File changed on disk since it was opened; resolve the external conflict before saving.";

interface BackendWatchEvent {
  paths: string[];
  kind: string;
}

const starterDocument = `---
title: Market Entry Report
subtitle: FY27 Expansion Strategy
author: Strategy Team
version: 1.0.0
status: draft
classification: confidential
toc: true
client: Example Corp
date: 2026-05-18
brand:
  name: Example Corp
  color: "#275DA8"
layout:
  pageSize: A4
  header: "{{title}}"
  footer: "{{classification}} | Page {{page}} of {{pages}}"
---

# Market Entry Report

[TOC]

## Executive Summary

Prepared for {{client}} on {{date}}.

\`\`\`calc
revenue = 125000
cost = 74000
profit = revenue - cost
margin = profit / revenue
\`\`\`

Expected margin: {{=margin | percent}}

## Source Governance

\`\`\`ai-source
provider: OpenAI
model: ChatGPT
date: 2026-05-18
reviewedBy: Strategy Team
status: human-reviewed
\`\`\`

## Data Table

\`\`\`csv
Region,Revenue
East,120
West,98
North,132
\`\`\`

## Terms

\`\`\`glossary
ARR: Annual recurring revenue.
CAC: Customer acquisition cost.
NDR: Net dollar retention.
\`\`\`

[INDEX]
`;

function fallbackHash(text: string) {
  let hash = 0;
  for (let index = 0; index < text.length; index += 1) {
    hash = (hash << 5) - hash + text.charCodeAt(index);
    hash |= 0;
  }
  return String(hash);
}

function errorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function isStaleSaveConflict(error: unknown) {
  return errorText(error).includes(staleSaveConflictMessage);
}

function isMissingTauriBackendError(error: unknown) {
  const message = errorText(error);
  return message.includes("reading 'invoke'") || message.includes("__TAURI_INTERNALS__");
}

function equivalentSha256Hash(left?: string | null, right?: string | null) {
  const normalize = (value?: string | null) => (value || "").replace(/^sha256:/, "");
  return Boolean(left && right && normalize(left) === normalize(right));
}

function watchEventIsAccessOnly(event: WatchEvent) {
  return typeof event.type === "object" && "access" in event.type;
}

function stringifyWatchEventKind(kind: WatchEvent["type"]) {
  if (typeof kind === "string") return kind;
  return Object.keys(kind)[0] || "other";
}

function externalTransformProbeBody(name: string) {
  switch (name) {
    case "dot":
    case "graphviz":
    case "circo":
    case "neato":
    case "fdp":
    case "osage":
    case "twopi":
      return "digraph G { a -> b }\n";
    case "plantuml":
      return "@startuml\nAlice -> Bob: probe\n@enduml\n";
    case "d2":
      return "a -> b\n";
    case "pikchr":
      return 'box "NEditor"\n';
    default:
      return "neditor transform probe\n";
  }
}

export const useDocumentsStore = defineStore("documents", {
  state: () => ({
    documents: [
      {
        id: crypto.randomUUID(),
        path: null,
        title: "Untitled",
        text: starterDocument,
        savedHash: fallbackHash(starterDocument),
        savedText: starterDocument,
        dirty: true,
      },
    ] as OpenDocument[],
    activeId: "",
    mode: "split" as "split" | "source" | "preview" | "focus" | "outline" | "export" | "review" | "presentation",
    sidebar: "outline" as
      | "files"
      | "outline"
      | "diagnostics"
      | "tables"
      | "templates"
      | "references"
      | "exports"
      | "versioning"
      | "review"
      | "help"
      | "settings",
    theme: "system" as "system" | "light" | "dark",
    previewTheme: "match" as PreviewTheme,
    toolbarDisplay: "both" as ToolbarDisplay,
    toolbarTextSize: 10,
    toolbarCollapsedRows: [] as string[],
    editorPaneRatio: 0.5,
    splitSourcePanes: false,
    editorKeymapMode: "default" as EditorKeymapMode,
    wordWrap: true,
    lineNumbers: true,
    codeFolding: true,
    highContrast: false,
    reducedMotion: false,
    autosave: false,
    autosaveDelayMs: 1500,
    autoSnapshot: false,
    snapshotIntervalMs: 300000,
    snapshotStorage: "app-data" as SnapshotStorage,
    editorFont: "Menlo, Consolas, monospace",
    previewFont: "Inter, Arial, sans-serif",
    editorFontSize: 14,
    previewFontSize: 14,
    editorLineHeight: 1.55,
    previewLineHeight: 1.65,
    exportTarget: "html" as ExportTarget,
    exportDefaults: {
      includeManifest: true,
      includeStyles: true,
      includeSyntaxHighlighting: true,
      htmlLanguage: "",
      htmlDescription: "",
      canonicalUrl: "",
      coverPage: true,
      pageNumbers: true,
      layoutPreset: "business",
      includeComments: true,
      includeProvenance: true,
      includeGlossary: true,
      includeAgenda: true,
    } as ExportDefaults,
    bibliographyDefaults: normalizeBibliographyDefaults({}),
    brandProfileDefaults: normalizeBrandProfileDefaults({}),
    businessProfile: normalizeBusinessProfile({}) as BusinessProfile,
    aiProviderDefaults: normalizeAiProviderDefaults({}) as AiProviderDefaults,
    ttsPreferences: normalizeTtsPreferences({}) as TtsPreferences,
    exportProfiles: [] as ExportProfile[],
    activeExportProfileId: "",
    gitIntegration: normalizeGitIntegrationPreferences({}),
    aiCleanupDefaults: normalizeAiCleanupDefaults({}),
    agentRunHistory: [] as AgentRunHistoryItem[],
    docsLiveDraftHistory: [] as DocsLiveDraftHistoryItem[],
    guidedDemoCompletedStepIds: [] as string[],
    gitStatus: null as GitStatus | null,
    statusMessage: "Ready",
    lastError: "",
    externalHash: "",
    externalConflict: null as ExternalConflictState | null,
    ignoredConflictHashes: {} as Record<string, string>,
    watchSignature: "",
    watchDriver: "off" as "off" | "native" | "plugin",
    watchContext: null as WatchContext | null,
    watchedPaths: [] as string[],
    watchedPathRoles: {} as Record<string, "root" | "include">,
    transformEngines: [] as TransformEngineMetadata[],
    transformEnginePaths: {} as Record<string, string>,
    trustedTransformEngines: {} as Record<string, boolean>,
    disabledTransformEngines: {} as Record<string, boolean>,
    transformInputModes: {} as Record<string, "stdin" | "file">,
    transformTimeoutMs: 5000,
    customTransformTemplates: [] as CustomTransformTemplate[],
    transformProbeResults: {} as Record<string, TransformProbeResult>,
    snapshots: [] as SnapshotListItem[],
    exportReadiness: null as ExportReadinessReport | null,
    compileTaskGate: { sequence: 0 },
    compileBusy: false,
    compileProgress: "",
    lastPreviewCompileDurationMs: null as number | null,
    lastPreviewCompiledCharacters: 0,
    lastPreviewCompiledAt: "",
    exportBusy: false,
    exportProgress: "",
    lastExportOutputPath: "",
    lastExportManifestPath: "",
    lastExportDiagnostics: [] as DocumentDiagnostic[],
    lastExportProgressSteps: [] as ExportProgressStep[],
    aiCleanupIssues: [] as string[],
    aiCleanupPreview: null as AiCleanupResponse | null,
    recentFiles: [] as string[],
    recentFolders: [] as string[],
    recentlyClosed: [] as string[],
    workspaceRoot: null as string | null,
    workspaceFiles: [] as WorkspaceFileEntry[],
    missingWorkspaceFiles: [] as string[],
    gitHistory: [] as GitHistoryEntry[],
    gitDiffText: "",
    releaseTag: "",
    commitMessage: "",
  }),
  getters: {
    activeDocument(state): OpenDocument {
      return activeDocumentState(state.documents, state.activeId) || state.documents[0];
    },
    windowTitle(): string {
      return windowTitleState(this.activeDocument);
    },
    externalTransformEngines(state): TransformEngineMetadata[] {
      return externalTransformEnginesState(state.transformEngines);
    },
  },
  actions: {
    async boot() {
      if (!this.activeId) this.activeId = this.documents[0].id;
      await this.loadPreferences();
      await this.compileActive();
      await this.refreshWorkspace();
      await this.refreshGitStatus();
      await this.listSnapshots();
      try {
        this.transformEngines = await invoke<TransformEngineMetadata[]>("list_transform_engines");
      } catch {
        this.transformEngines = [];
      }
    },
    setActiveDocument(id: string) {
      const active = applyActiveWorkspaceDocumentState(this.documents, this.activeId, id);
      this.activeId = active.activeId;
    },
    async activateDocument(id: string) {
      const active = applyActiveWorkspaceDocumentState(this.documents, this.activeId, id);
      if (!active.changed) return;
      this.activeId = active.activeId;
      await this.compileActive();
      await this.refreshGitStatus();
      await this.persistWorkspace();
    },
    async loadPreferences() {
      try {
        preferencesStore = await Store.load("settings.json");
        const persisted = migratePersistedWorkspace(await preferencesStore.get<unknown>("workspace"));
        const loaded = applyPersistedWorkspacePreferenceState(this, persisted);
        Object.assign(this, loaded.state);
        if (loaded.restoreRequest) {
          await this.restoreWorkspace(
            loaded.restoreRequest.openFiles,
            loaded.restoreRequest.activePath,
            loaded.restoreRequest.pinnedFiles,
            loaded.restoreRequest.scrollPositions,
          );
        }
      } catch (error) {
        this.lastError = isMissingTauriBackendError(error) ? "" : errorText(error);
      }
    },
    async persistWorkspace() {
      if (!preferencesStore) return;
      const workspace = buildPersistedWorkspaceState(this);
      await preferencesStore.set("workspace", workspace);
      await preferencesStore.save();
    },
    recordAgentRunHistory(item: AgentRunHistoryItem) {
      this.agentRunHistory = recordAgentRunHistoryState(this.agentRunHistory, item);
      void this.persistWorkspace();
    },
    removeAgentRunHistory(runId: string) {
      this.agentRunHistory = removeAgentRunHistoryState(this.agentRunHistory, runId);
      void this.persistWorkspace();
    },
    clearAgentRunHistory() {
      const next = clearAgentRunHistoryState(this.agentRunHistory);
      if (next === this.agentRunHistory) return;
      this.agentRunHistory = next;
      void this.persistWorkspace();
    },
    recordDocsLiveDraftHistory(item: DocsLiveDraftHistoryItem) {
      this.docsLiveDraftHistory = recordDocsLiveDraftHistoryState(this.docsLiveDraftHistory, item);
      void this.persistWorkspace();
    },
    removeDocsLiveDraftHistory(draftId: string) {
      this.docsLiveDraftHistory = removeDocsLiveDraftHistoryState(this.docsLiveDraftHistory, draftId);
      void this.persistWorkspace();
    },
    clearDocsLiveDraftHistory() {
      const next = clearDocsLiveDraftHistoryState(this.docsLiveDraftHistory);
      if (next === this.docsLiveDraftHistory) return;
      this.docsLiveDraftHistory = next;
      void this.persistWorkspace();
    },
    saveBusinessProfile(profile: Partial<BusinessProfile>) {
      const next = saveBusinessProfileState(this.businessProfile, profile);
      if (!next.changed) return;
      this.businessProfile = next.value;
      void this.persistWorkspace();
    },
    saveAiProviderDefaults(defaults: Partial<AiProviderDefaults>) {
      const next = saveAiProviderDefaultsState(this.aiProviderDefaults, defaults);
      if (!next.changed) return;
      this.aiProviderDefaults = next.value;
      void this.persistWorkspace();
    },
    saveTtsPreferences(defaults: Partial<TtsPreferences>) {
      const next = saveTtsPreferencesState(this.ttsPreferences, defaults);
      if (!next.changed) return;
      this.ttsPreferences = next.value;
      void this.persistWorkspace();
    },
    recordGuidedDemoStepComplete(stepId: string) {
      const nextStepIds = recordGuidedDemoStepState(this.guidedDemoCompletedStepIds, stepId);
      if (nextStepIds === this.guidedDemoCompletedStepIds) return;
      this.guidedDemoCompletedStepIds = nextStepIds;
      void this.persistWorkspace();
    },
    resetGuidedDemoProgress() {
      const next = resetGuidedDemoProgressState(this.guidedDemoCompletedStepIds);
      if (next === this.guidedDemoCompletedStepIds) return;
      this.guidedDemoCompletedStepIds = next;
      void this.persistWorkspace();
    },
    async restoreWorkspace(
      paths: string[],
      activePath: string | null,
      pinnedFiles: string[] = [],
      scrollPositions: Record<string, PersistedScrollPosition> = {},
    ) {
      const restored: OpenDocument[] = [];
      const missing: string[] = [];
      const seen = new Set<string>();
      for (const path of paths) {
        if (seen.has(path)) continue;
        seen.add(path);
        try {
          const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", { path });
          restored.push(createRestoredWorkspaceDocumentState(response, path, pinnedFiles, scrollPositions, () => crypto.randomUUID()));
        } catch {
          missing.push(path);
        }
      }
      const result = applyWorkspaceRestoreState(
        this.documents,
        this.activeId,
        this.recentFiles,
        this.recentlyClosed,
        restored,
        missing,
        activePath,
      );
      this.recentFiles = result.recentFiles;
      this.recentlyClosed = result.recentlyClosed;
      this.missingWorkspaceFiles = result.missingWorkspaceFiles;
      if (result.statusMessage) this.statusMessage = result.statusMessage;
      if (!restored.length) {
        if (result.persistRequired) await this.persistWorkspace();
        return;
      }
      this.documents = result.documents;
      this.activeId = result.activeId;
      if (result.persistRequired) await this.persistWorkspace();
    },
    newDocument() {
      const document = createUntitledDocumentState(starterDocument, fallbackHash(starterDocument), () => crypto.randomUUID());
      const created = applyNewWorkspaceDocumentState(this.documents, document);
      this.documents = created.documents;
      this.activeId = created.activeId;
      void this.compileActive();
    },
    async openPath(path: string) {
      const existing = this.documents.find((document) => document.path === path);
      if (existing) {
        this.activeId = existing.id;
        await this.compileActive();
        await this.refreshGitStatus();
        return;
      }
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", { path });
      const document = createOpenedDocumentState(response, () => crypto.randomUUID());
      const opened = applyOpenedWorkspaceDocumentState(
        this.documents,
        this.recentFiles,
        this.recentlyClosed,
        this.missingWorkspaceFiles,
        document,
      );
      this.documents = opened.documents;
      this.activeId = opened.activeId;
      this.recentFiles = opened.recentFiles;
      this.recentlyClosed = opened.recentlyClosed;
      this.missingWorkspaceFiles = opened.missingWorkspaceFiles;
      this.statusMessage = opened.statusMessage;
      if (!this.workspaceRoot) {
        const folder = folderFromPath(document.path);
        if (folder) await this.openFolder(folder);
      }
      await this.compileActive();
      await this.refreshGitStatus();
      await this.persistWorkspace();
    },
    async openRecentPath(path: string) {
      try {
        await this.openPath(path);
        return true;
      } catch {
        const failed = applyOpenRecentWorkspaceFileFailureState(
          this.recentFiles,
          this.recentlyClosed,
          this.missingWorkspaceFiles,
          path,
        );
        this.recentFiles = failed.recentFiles;
        this.recentlyClosed = failed.recentlyClosed;
        this.missingWorkspaceFiles = failed.missingWorkspaceFiles;
        this.statusMessage = failed.statusMessage;
        await this.persistWorkspace();
        return false;
      }
    },
    setDocumentScroll(id: string, scroll: { editor?: number; preview?: number }, persist = false) {
      const result = setDocumentScrollState(this.documents, id, scroll);
      if (!result.changed) return;
      this.documents = result.documents;
      if (persist) void this.persistWorkspace();
    },
    async openFolder(path: string) {
      const previousRoot = this.workspaceRoot;
      const previousFiles = this.workspaceFiles;
      this.workspaceRoot = path;
      const opened = await this.refreshWorkspace();
      if (!opened) {
        const failed = applyOpenWorkspaceFolderFailureState(previousRoot, previousFiles, path);
        this.workspaceRoot = failed.workspaceRoot;
        this.workspaceFiles = failed.workspaceFiles;
        this.statusMessage = failed.statusMessage;
        await this.persistWorkspace();
        return false;
      }
      const succeeded = applyOpenWorkspaceFolderSuccessState(this.recentFolders, path);
      this.recentFolders = succeeded.recentFolders;
      this.sidebar = succeeded.sidebar;
      this.statusMessage = succeeded.statusMessage;
      await this.persistWorkspace();
      return true;
    },
    async openRecentFolder(path: string) {
      const opened = await this.openFolder(path);
      if (opened) return true;
      const failed = applyOpenRecentWorkspaceFolderFailureState(this.recentFolders, this.workspaceRoot, this.workspaceFiles, path);
      this.recentFolders = failed.recentFolders;
      this.workspaceRoot = failed.workspaceRoot;
      this.workspaceFiles = failed.workspaceFiles;
      this.statusMessage = failed.statusMessage;
      await this.persistWorkspace();
      return false;
    },
    async refreshWorkspace() {
      if (!this.workspaceRoot) {
        Object.assign(this, clearWorkspaceRefreshState<WorkspaceFileEntry>());
        return true;
      }
      try {
        const workspaceFiles = await invoke<WorkspaceFileEntry[]>("list_workspace_files", {
          request: { root: this.workspaceRoot },
        });
        Object.assign(this, applyWorkspaceRefreshSuccessState(workspaceFiles));
        return true;
      } catch (error) {
        Object.assign(this, applyWorkspaceRefreshFailureState<WorkspaceFileEntry>(error));
        return false;
      }
    },
    async saveActive(path?: string) {
      const doc = this.activeDocument;
      const target = path || doc.path;
      if (!target) throw new Error("Choose a save path before saving this document.");
      const isExistingDocumentSave = Boolean(doc.path && target === doc.path);
      const pathChanged = !doc.path || target !== doc.path;
      if (isExistingDocumentSave) {
        const metadata = await invoke<FileMetadataResponse>("file_metadata", { path: target });
        if (metadata.exists && metadata.hash && metadata.hash !== doc.savedHash) {
          await this.openExternalConflict(doc, target, "root", "The root file changed outside NEditor before save.", metadata.hash);
          this.statusMessage = "Save blocked; resolve external changes first";
          return;
        }
      }
      let response: { path: string; text: string; hash: string; modified?: string };
      try {
        response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("save_file", {
          request: { path: target, text: doc.text, expected_hash: isExistingDocumentSave ? doc.savedHash : null },
        });
      } catch (error) {
        if (isExistingDocumentSave && isStaleSaveConflict(error)) {
          const metadata = await invoke<FileMetadataResponse>("file_metadata", { path: target });
          await this.openExternalConflict(
            doc,
            target,
            "root",
            "The root file changed outside NEditor during save.",
            metadata.hash || "external-change",
          );
          this.statusMessage = "Save blocked; resolve external changes first";
          return;
        }
        throw error;
      }
      const saved = applySavedWorkspaceDocumentState(doc, response, this.recentFiles);
      Object.assign(doc, saved.document);
      this.ignoredConflictHashes = saved.ignoredConflictHashes;
      this.recentFiles = saved.recentFiles;
      this.statusMessage = saved.statusMessage;
      if (this.workspaceRoot) await this.refreshWorkspace();
      await this.refreshGitStatus();
      await this.persistWorkspace();
      if (pathChanged) {
        const savedStatus = this.statusMessage;
        await this.compileActive();
        this.statusMessage = savedStatus;
      }
    },
    async revertActive() {
      const doc = this.activeDocument;
      await this.snapshotBeforeDestructiveAction("pre-revert");
      if (!doc.path) {
        const reverted = applyUntitledRevertState(doc, starterDocument, fallbackHash(starterDocument));
        Object.assign(doc, reverted.document);
        await this.compileActive();
        this.statusMessage = reverted.statusMessage;
        return;
      }
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", {
        path: doc.path,
      });
      const reverted = applyRevertedDocumentState(doc, response);
      Object.assign(doc, reverted.document);
      this.statusMessage = reverted.statusMessage;
      await this.compileActive();
      await this.refreshGitStatus();
    },
    async renameActive(path: string) {
      const doc = this.activeDocument;
      if (!doc.path) throw new Error("Save the document before renaming it.");
      const oldPath = doc.path;
      const metadata = await invoke<{ path: string; exists: boolean; hash?: string; modified?: string }>("rename_file", {
        request: { from: doc.path, to: path },
      });
      const renamed = applyRenamedWorkspaceDocumentState(
        doc,
        metadata,
        oldPath,
        this.recentFiles,
        this.recentlyClosed,
        this.missingWorkspaceFiles,
      );
      Object.assign(doc, renamed.document);
      this.recentFiles = renamed.recentFiles;
      this.recentlyClosed = renamed.recentlyClosed;
      this.missingWorkspaceFiles = renamed.missingWorkspaceFiles;
      this.statusMessage = renamed.statusMessage;
      if (this.workspaceRoot) await this.refreshWorkspace();
      await this.refreshGitStatus();
      await this.persistWorkspace();
      const renameStatus = this.statusMessage;
      await this.compileActive();
      this.statusMessage = renameStatus;
    },
    async duplicateActive(path: string) {
      const doc = this.activeDocument;
      if (!doc.path) await this.saveActive(path);
      const source = this.activeDocument.path;
      if (!source) throw new Error("Save the document before duplicating it.");
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("duplicate_file", {
        request: { from: source, to: path },
      });
      const duplicate = createDuplicateDocumentState(response, () => crypto.randomUUID());
      const duplicated = applyDuplicatedWorkspaceDocumentState(this.documents, this.recentFiles, duplicate);
      this.documents = duplicated.documents;
      this.activeId = duplicated.activeId;
      this.recentFiles = duplicated.recentFiles;
      this.statusMessage = duplicated.statusMessage;
      if (this.workspaceRoot) await this.refreshWorkspace();
      await this.compileActive();
      await this.persistWorkspace();
    },
    async revealActive() {
      const doc = this.activeDocument;
      if (!doc.path) throw new Error("Save the document before revealing it.");
      await invoke("reveal_path", { path: doc.path });
      this.statusMessage = `Revealed ${doc.title} in file manager`;
    },
    updateText(text: string) {
      const doc = this.activeDocument;
      const updated = applyUpdatedDocumentTextState(doc, text, fallbackHash);
      Object.assign(doc, updated.document);
      void this.compileActive();
    },
    async compileActive() {
      const doc = this.activeDocument;
      if (!doc) return;
      const snapshot = beginLatestDocumentTask(this.compileTaskGate, doc);
      const startedAt = performance.now();
      this.compileBusy = true;
      this.compileProgress = "Compiling preview";
      try {
        const compile = await invoke<CompileResponse>("compile_document_with_options", {
          request: { text: snapshot.text, file_path: doc.path, options: this.compileOptionsForActive() },
        });
        if (!isLatestDocumentTaskCurrent(this.compileTaskGate, snapshot, this.activeDocument)) {
          return;
        }
        doc.compile = compile;
        doc.title = String(doc.compile.semantic.title || titleFromPath(doc.path));
        this.lastPreviewCompileDurationMs = Math.max(0, Math.round(performance.now() - startedAt));
        this.lastPreviewCompiledCharacters = snapshot.text.length;
        this.lastPreviewCompiledAt = new Date().toISOString();
        this.statusMessage = `${doc.compile.diagnostics.length} diagnostics`;
        this.lastError = "";
        await this.syncFileWatcher();
      } catch (error) {
        if (isLatestDocumentTaskCurrent(this.compileTaskGate, snapshot, this.activeDocument)) {
          if (isMissingTauriBackendError(error)) {
            this.lastError = "";
            this.statusMessage = "Editing locally; preview backend unavailable in browser";
          } else {
            this.lastError = errorText(error);
          }
        }
      } finally {
        if (isLatestDocumentTaskCurrent(this.compileTaskGate, snapshot, this.activeDocument)) {
          this.compileBusy = false;
          this.compileProgress = "";
        }
      }
    },
    cancelActiveCompile() {
      if (!this.compileBusy) return;
      cancelLatestDocumentTask(this.compileTaskGate);
      this.compileBusy = false;
      this.compileProgress = "";
      this.statusMessage = "Cancelled preview compile";
    },
    async syncFileWatcher() {
      const doc = this.activeDocument;
      if (!doc?.path) {
        if (this.watchSignature) {
          await this.stopFileWatcher();
        }
        this.watchSignature = "";
        this.watchDriver = "off";
        this.watchContext = null;
        this.watchedPaths = [];
        this.watchedPathRoles = {};
        return;
      }
      const openRootPaths = this.documents
        .map((document) => document.path)
        .filter((path): path is string => Boolean(path))
        .filter((path) => !sameWatchPath(path, doc.path));
      const includedPaths = (doc.compile?.export_manifest.included_files || []).map((file) => file.path);
      const watchSnapshot = await invoke<WatchFileResponse>("start_file_watcher", {
        request: { root: doc.path, open_roots: openRootPaths, included: includedPaths },
      });
      const watchedFiles = watchSnapshot.paths.filter((file) => file.exists);
      const watchPaths = watchedFiles.map((file) => file.path);
      const pathRoles = buildWatchedPathRoles(watchedFiles);
      const driver = watchSnapshot.native_watcher ? "native" : "plugin";
      const signature = `${doc.id}\n${driver}\n${watchedFiles.map((file) => `${file.role || "include"}:${file.path}`).join("\n")}`;
      if (signature === this.watchSignature) return;
      const context: WatchContext = {
        documentId: doc.id,
        rootPath: doc.path,
        openRootPaths,
        includedPaths,
        signature,
      };
      this.detachFileWatchListeners();
      if (!watchPaths.length) {
        this.watchSignature = "";
        this.watchDriver = "off";
        this.watchContext = null;
        this.watchedPaths = [];
        this.watchedPathRoles = {};
        return;
      }
      this.watchContext = context;
      if (watchSnapshot.native_watcher) {
        await this.attachBackendFileWatchListeners(context);
      } else {
        unwatchFileChanges = await watchFs(
          watchPaths,
          (event) => {
            void this.handleFsWatchEvent(event, context);
          },
          { delayMs: 250 },
        );
      }
      if (watchSnapshot.watcher_error) {
        this.statusMessage = "Native file watcher unavailable; using plugin watch";
      }
      this.watchSignature = signature;
      this.watchDriver = driver;
      this.watchedPaths = watchPaths;
      this.watchedPathRoles = pathRoles;
      await this.refreshExternalState(doc, undefined, context);
    },
    detachFileWatchListeners() {
      unwatchFileChanges?.();
      unwatchFileChanges = null;
      unwatchFileErrors?.();
      unwatchFileErrors = null;
    },
    async stopFileWatcher() {
      this.detachFileWatchListeners();
      await invoke("stop_file_watcher").catch(() => undefined);
      this.watchSignature = "";
      this.watchDriver = "off";
      this.watchContext = null;
      this.watchedPaths = [];
      this.watchedPathRoles = {};
    },
    async attachBackendFileWatchListeners(context: WatchContext) {
      unwatchFileChanges = await listen<BackendWatchEvent>("neditor-file-watch-event", (event) => {
        void this.handleBackendWatchEvent(event.payload, context);
      });
      unwatchFileErrors = await listen<string>("neditor-file-watch-error", (event) => {
        this.lastError = event.payload;
        this.statusMessage = "File watcher failed";
      });
    },
    async handleFsWatchEvent(event: WatchEvent, context: WatchContext) {
      if (!this.watchContextIsCurrent(context)) return;
      const { rootPath, includedPaths } = context;
      const paths = event.paths.length ? event.paths : [rootPath];
      for (const path of paths) {
        const reason = this.watchReasonForPath(path, rootPath, includedPaths);
        if (!reason || watchEventIsAccessOnly(event)) continue;
        const metadata = await invoke<FileMetadataResponse>("file_metadata", { path });
        await this.handleWatchedFileChange({
          path,
          reason,
          kind: stringifyWatchEventKind(event.type),
          hash: metadata.hash,
          modified: metadata.modified,
        }, context);
      }
    },
    async handleBackendWatchEvent(event: BackendWatchEvent, context: WatchContext) {
      if (!this.watchContextIsCurrent(context)) return;
      const { rootPath, includedPaths } = context;
      const paths = event.paths.length ? event.paths : [rootPath];
      for (const path of paths) {
        const reason = this.watchReasonForPath(path, rootPath, includedPaths);
        if (!reason) continue;
        const metadata = await invoke<FileMetadataResponse>("file_metadata", { path });
        await this.handleWatchedFileChange({
          path,
          reason,
          kind: event.kind,
          hash: metadata.hash,
          modified: metadata.modified,
        }, context);
      }
    },
    watchReasonForPath(path: string, rootPath: string, includedPaths: string[]) {
      return resolveWatchReason(path, rootPath, includedPaths, this.watchedPathRoles);
    },
    watchContextIsCurrent(context: WatchContext) {
      return isCurrentWatchContext(context, this.watchContext, this.activeDocument);
    },
    documentForWatchContext(context: WatchContext) {
      return documentForWatchContextState(this.documents, context);
    },
    documentForWatchedRoot(path: string, context: WatchContext) {
      return documentForWatchedRootState(this.documents, path, context);
    },
    async handleWatchedFileChange(event: DocumentWatchEvent, context: WatchContext) {
      if (!this.watchContextIsCurrent(context)) return;
      const doc = event.reason === "root" ? this.documentForWatchedRoot(event.path, context) : this.documentForWatchContext(context);
      if (!doc) return;
      const watched = watchedPathsForContext(this.watchedPaths, context);
      if (!watched.some((path) => sameWatchPath(path, event.path))) return;
      await this.refreshExternalState(doc, event, context);
    },
    async refreshExternalState(doc?: OpenDocument, event?: DocumentWatchEvent, context?: WatchContext) {
      if (context && !this.watchContextIsCurrent(context)) return;
      const targetDoc = doc || this.activeDocument;
      if (!targetDoc.path) return;
      const metadata = await invoke<FileMetadataResponse>("file_metadata", { path: targetDoc.path });
      this.externalHash = metadata.hash || "";
      const rootEventIsRealChange = event?.reason === "root" && (!event.hash || event.hash !== targetDoc.savedHash);
      const mainChanged =
        rootEventIsRealChange || Boolean(metadata.exists && metadata.hash && metadata.hash !== targetDoc.savedHash);
      const changedInclude =
        event?.reason === "include"
          ? { path: event.path, hash: event.hash || "include-change" }
          : await this.changedIncludedFile(targetDoc);
      const includeChanged = Boolean(changedInclude);
      if (targetDoc.dirty) {
        const ignoredRoot = Boolean(
          mainChanged && metadata.hash && this.ignoredConflictHashes[targetDoc.path] === metadata.hash,
        );
        const ignoredInclude = Boolean(
          changedInclude?.path &&
            changedInclude.hash &&
            this.ignoredConflictHashes[changedInclude.path] === changedInclude.hash,
        );
        if (ignoredRoot || ignoredInclude) return;
      }
      if ((mainChanged || includeChanged) && targetDoc.dirty) {
        await this.openExternalConflict(
          targetDoc,
          mainChanged ? targetDoc.path : changedInclude?.path || event?.path || targetDoc.path,
          mainChanged ? "root" : "include",
          mainChanged
            ? "The root file changed outside NEditor while local edits are unsaved."
            : "An included file changed while local edits are unsaved.",
          (mainChanged ? metadata.hash : changedInclude?.hash) || "include-change",
        );
        this.statusMessage = mainChanged
          ? "External changes detected; compare before overwriting"
          : "Included file changes detected; save or compare before recompiling";
        return;
      }
      if (mainChanged && metadata.hash) {
        const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", { path: targetDoc.path });
        const reloaded = applyExternalRootReloadState(targetDoc, response, this.activeDocument.id);
        Object.assign(targetDoc, reloaded.document);
        this.externalConflict = reloaded.externalConflict;
        if (targetDoc.id === this.activeDocument.id) {
          await this.compileActive();
        }
        this.statusMessage = reloaded.statusMessage;
      } else if (includeChanged) {
        this.externalConflict = null;
        await this.compileActive();
        this.statusMessage = event?.path
          ? `Recompiled after included file changed: ${titleFromPath(event.path)}`
          : "Recompiled after included file changes";
      }
    },
    async acceptExternalChanges() {
      const conflict = this.externalConflict;
      if (!conflict) return;
      const doc = this.documents.find((document) => document.id === conflict.documentId) || this.activeDocument;
      this.setActiveDocument(doc.id);
      if (conflict.reason === "root") {
        await this.snapshotBeforeDestructiveAction("pre-accept-external");
        const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", {
          path: conflict.path,
        });
        const accepted = acceptExternalRootConflictState(doc, response);
        Object.assign(doc, accepted.document);
        this.externalConflict = accepted.externalConflict;
        this.statusMessage = accepted.statusMessage;
      } else {
        await this.compileActive();
        this.statusMessage = "Accepted included file changes";
        this.externalConflict = null;
      }
      this.clearIgnoredConflicts();
      await this.compileActive();
      await this.refreshGitStatus();
    },
    keepLocalChanges() {
      const conflict = this.externalConflict;
      if (conflict?.externalHash) {
        this.rememberIgnoredConflict(conflict.path, conflict.externalHash);
      }
      if (conflict?.reason === "root") {
        const doc = this.documents.find((document) => document.id === conflict.documentId) || this.activeDocument;
        this.setActiveDocument(doc.id);
        const kept = keepLocalRootConflictState(doc, conflict);
        Object.assign(doc, kept.document);
        this.externalHash = kept.externalHash;
        this.externalConflict = kept.externalConflict;
        this.statusMessage = kept.statusMessage;
        return;
      }
      this.externalConflict = null;
      this.statusMessage = "Keeping local edits";
    },
    async saveLocalConflictCopy(path: string) {
      const conflict = this.externalConflict;
      if (conflict?.documentId && conflict.documentId !== this.activeId) {
        this.setActiveDocument(conflict.documentId);
      }
      await this.saveActive(path);
      this.externalConflict = null;
      this.clearIgnoredConflicts();
      this.statusMessage = "Saved local edits as a copy";
    },
    async applyConflictMerge(text: string) {
      const conflict = this.externalConflict;
      if (!conflict || conflict.reason !== "root") return;
      const doc = this.documents.find((document) => document.id === conflict.documentId) || this.activeDocument;
      this.setActiveDocument(doc.id);
      await this.snapshotBeforeDestructiveAction("pre-conflict-merge");
      const merged = applyRootConflictMergeState(doc, conflict, text);
      Object.assign(doc, merged.document);
      this.externalHash = merged.externalHash;
      this.externalConflict = merged.externalConflict;
      this.clearIgnoredConflicts();
      this.statusMessage = merged.statusMessage;
      await this.compileActive();
      await this.refreshGitStatus();
    },
    rememberIgnoredConflict(path: string, hash: string) {
      this.ignoredConflictHashes = { ...this.ignoredConflictHashes, [path]: hash };
    },
    clearIgnoredConflicts() {
      this.ignoredConflictHashes = {};
    },
    async changedIncludedFile(doc: OpenDocument) {
      const includedFiles = doc.compile?.export_manifest.included_files || [];
      if (!includedFiles.length) return null;
      for (const included of includedFiles) {
        try {
          const metadata = await invoke<FileMetadataResponse>("file_metadata", { path: included.path });
          if (!metadata.exists || !equivalentSha256Hash(metadata.hash, included.hash)) {
            return { path: included.path, hash: metadata.hash || "include-change" };
          }
        } catch {
          return { path: included.path, hash: "include-change" };
        }
      }
      return null;
    },
    async openExternalConflict(doc: OpenDocument, path: string, reason: "root" | "include", message: string, externalHash: string) {
      let externalText = "";
      try {
        externalText = (await invoke<{ text: string }>("read_file", { path })).text;
      } catch {
        externalText = reason === "include" ? "The changed included file could not be read. It may have been deleted or moved." : "";
      }
      this.externalConflict = createExternalConflictState(doc, path, reason, message, externalHash, externalText);
    },
    async exportActive(path: string) {
      if (this.exportBusy) return;
      const doc = this.activeDocument;
      Object.assign(this, beginExportWorkflowState());
      try {
        Object.assign(this, exportProgressState("Creating pre-export snapshot"));
        await this.createSnapshot("pre-export");
        Object.assign(this, exportProgressState(`Writing ${this.exportTarget.toUpperCase()} export`));
        const response = await invoke<{
          output_path: string;
          manifest_path?: string | null;
          diagnostics?: DocumentDiagnostic[];
          progress_steps?: ExportProgressStep[];
        }>("export_document", {
          request: {
            text: doc.text,
            file_path: doc.path,
            target: this.exportTarget,
            output_path: path,
            options: this.exportOptionsForActive(),
          },
        });
        Object.assign(this, applyExportSuccessState(response));
        Object.assign(this, exportProgressState("Refreshing export snapshots"));
        await this.listSnapshots();
      } catch (error) {
        Object.assign(this, applyExportFailureState(error, doc.path, this.exportTarget));
      } finally {
        Object.assign(this, finishExportWorkflowState());
      }
    },
    saveCurrentExportProfile(name: string) {
      const result = saveExportProfileState(this.exportProfiles, this.activeExportProfileId, name, {
        exportTarget: this.exportTarget,
        exportDefaults: this.exportDefaults,
        bibliographyDefaults: this.bibliographyDefaults,
        brandProfileDefaults: this.brandProfileDefaults,
      });
      this.exportProfiles = result.profiles;
      this.activeExportProfileId = result.activeExportProfileId;
      this.statusMessage = result.statusMessage;
      void this.persistWorkspace();
      return result.profile;
    },
    async applyExportProfile(id: string) {
      const result = applyExportProfileState(this.exportProfiles, id);
      if (!result) return;
      this.exportTarget = result.exportTarget;
      this.exportDefaults = result.exportDefaults;
      this.bibliographyDefaults = result.bibliographyDefaults;
      this.brandProfileDefaults = result.brandProfileDefaults;
      this.activeExportProfileId = result.activeExportProfileId;
      this.exportReadiness = null;
      this.statusMessage = result.statusMessage;
      await this.compileActive();
      await this.persistWorkspace();
    },
    deleteExportProfile(id: string) {
      const result = deleteExportProfileState(this.exportProfiles, this.activeExportProfileId, id);
      this.exportProfiles = result.profiles;
      this.activeExportProfileId = result.activeExportProfileId;
      if (result.statusMessage) this.statusMessage = result.statusMessage;
      void this.persistWorkspace();
    },
    exportOptionsForActive() {
      return buildDocumentExportOptions({
        exportDefaults: this.exportDefaults,
        bibliographyDefaults: this.bibliographyDefaults,
        brandProfileDefaults: this.brandProfileDefaults,
        gitIntegration: this.gitIntegration,
        transformEnginePaths: this.transformEnginePaths,
        trustedTransformEngines: this.trustedTransformEngines,
        disabledTransformEngines: this.disabledTransformEngines,
        transformInputModes: this.transformInputModes,
        transformTimeoutMs: this.transformTimeoutMs,
        semanticStatus: this.activeDocument.compile?.semantic.status,
      });
    },
    compileOptionsForActive() {
      return buildDocumentCompileOptions({
        bibliographyDefaults: this.bibliographyDefaults,
        brandProfileDefaults: this.brandProfileDefaults,
        transformEnginePaths: this.transformEnginePaths,
        trustedTransformEngines: this.trustedTransformEngines,
        disabledTransformEngines: this.disabledTransformEngines,
        transformInputModes: this.transformInputModes,
        transformTimeoutMs: this.transformTimeoutMs,
      });
    },
    async createSnapshot(label = "manual") {
      const doc = this.activeDocument;
      const snapshotText = snapshotTextForDocument(doc);
      return invoke<{ snapshot_path: string }>("create_snapshot", {
        request: { text: snapshotText, file_path: doc.path, label, storage: this.snapshotStorage },
      });
    },
    async snapshotActive(label = "manual") {
      const response = await this.createSnapshot(label);
      this.statusMessage = `Snapshot saved to ${response.snapshot_path}`;
      await this.listSnapshots();
    },
    async snapshotBeforeDestructiveAction(label: string) {
      await this.createSnapshot(label);
      await this.listSnapshots();
    },
    async listSnapshots() {
      try {
        this.snapshots = await invoke<SnapshotListItem[]>("list_snapshots", {
          request: { file_path: this.activeDocument?.path, storage: this.snapshotStorage },
        });
      } catch {
        this.snapshots = [];
      }
    },
    async restoreSnapshot(snapshotPath: string) {
      await this.snapshotBeforeDestructiveAction("pre-snapshot-restore");
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("restore_snapshot", {
        request: { snapshot_path: snapshotPath, file_path: this.activeDocument?.path, storage: this.snapshotStorage },
      });
      const doc = this.activeDocument;
      const restored = applyRestoredSnapshotState(doc, response, snapshotPath);
      Object.assign(doc, restored.document);
      this.statusMessage = restored.statusMessage;
      await this.compileActive();
      await this.listSnapshots();
    },
    async prepareForExport() {
      if (this.exportBusy) return;
      const doc = this.activeDocument;
      Object.assign(this, beginExportReadinessState());
      try {
        const exportReadiness = await invoke<ExportReadinessReport>("prepare_for_export", {
          request: {
            text: doc.text,
            file_path: doc.path,
            target: this.exportTarget,
            options: this.exportOptionsForActive(),
          },
        });
        Object.assign(this, applyExportReadinessState(exportReadiness));
      } finally {
        Object.assign(this, finishExportWorkflowState());
      }
    },
    async setTransformEnginePath(name: string, path: string) {
      const next = updateTransformEnginePathState(this, name, path);
      this.transformEnginePaths = next.transformEnginePaths;
      this.trustedTransformEngines = next.trustedTransformEngines;
      this.transformProbeResults = next.transformProbeResults;
      await this.persistWorkspace();
    },
    async setTransformTrust(name: string, trusted: boolean) {
      this.trustedTransformEngines = setTransformBooleanFlag(this.trustedTransformEngines, name, trusted);
      await this.persistWorkspace();
    },
    async setTransformDisabled(name: string, disabled: boolean) {
      this.disabledTransformEngines = setTransformBooleanFlag(this.disabledTransformEngines, name, disabled);
      await this.persistWorkspace();
    },
    async setTransformInputMode(name: string, mode: "stdin" | "file") {
      this.transformInputModes = setTransformInputModeState(this.transformInputModes, name, mode);
      await this.persistWorkspace();
    },
    async setTransformTimeout(timeoutMs: number) {
      this.transformTimeoutMs = clampTransformTimeout(timeoutMs);
      await this.persistWorkspace();
    },
    async saveCustomTransformTemplate(template: CustomTransformTemplate) {
      const next = saveCustomTransformTemplateState(this.customTransformTemplates, template);
      if (!next.changed) return;
      this.customTransformTemplates = next.templates;
      await this.persistWorkspace();
    },
    async deleteCustomTransformTemplate(id: string) {
      const next = deleteCustomTransformTemplateState(this.customTransformTemplates, id);
      if (!next.changed) return;
      this.customTransformTemplates = next.templates;
      await this.persistWorkspace();
    },
    setEditorPaneRatio(value: number, persist = true) {
      this.editorPaneRatio = clampPaneRatio(value);
      if (persist) void this.persistWorkspace();
    },
    async testExternalTransform(name: string) {
      const engine = this.transformEngines.find((candidate) => candidate.name === name);
      try {
        const response = await invoke<{ diagnostics: Array<{ message: string }>; cache_key: string }>("run_external_transform", {
          request: {
            name,
            body: externalTransformProbeBody(name),
            engine_path: this.transformEnginePaths[name] || "",
            trusted: Boolean(this.trustedTransformEngines[name]),
            input_mode: this.transformInputModes[name] || "stdin",
            timeout_ms: this.transformTimeoutMs,
            max_input_bytes: engine?.limits.maxInputBytes,
            max_output_bytes: engine?.limits.maxOutputBytes,
          },
        });
        Object.assign(this, applyTransformProbeSuccessState(this.transformProbeResults, name, response));
      } catch (error) {
        Object.assign(this, applyTransformProbeFailureState(this.transformProbeResults, name, error));
      }
    },
    async previewAiPaste(text: string, options: AiCleanupOptions) {
      const response = await invoke<AiCleanupResponse>("cleanup_ai_paste", {
        request: {
          text,
          add_provenance: options.addProvenance,
          mark_as_draft: options.markAsDraft,
          insert_citation_todos: options.insertCitationTodos,
          preserve_headings: options.preserveHeadings,
          convert_numbered_lists: options.convertNumberedLists,
          convert_tables: options.convertTables,
        },
      });
      this.aiCleanupPreview = response;
      this.aiCleanupIssues = response.issues;
      return response;
    },
    insertAiPaste(response: AiCleanupResponse, mode: AiPasteInsertMode) {
      this.updateText(applyAiPasteInsertion(this.activeDocument.text, response.cleaned_markdown, mode));
      this.statusMessage = `Cleaned AI paste with ${response.issues.length} issue notes`;
    },
    async cleanAiPaste(text: string, mode: AiPasteInsertMode, options: AiCleanupOptions) {
      const response = await this.previewAiPaste(text, options);
      this.insertAiPaste(response, mode);
    },
    insertReviewComment(text: string) {
      const createdAt = new Date().toISOString();
      this.updateText(appendReviewCommentMarker(this.activeDocument.text, text, createdAt));
      this.statusMessage = "Inserted review comment";
    },
    insertChangeNote(text: string) {
      const createdAt = new Date().toISOString();
      this.updateText(appendChangeNoteMarker(this.activeDocument.text, text, createdAt));
      this.statusMessage = "Inserted change note";
    },
    resolveReviewComment(line: number) {
      const resolved = resolveReviewCommentAtLine(this.activeDocument.text, line);
      if (!resolved) return;
      this.updateText(resolved);
      this.statusMessage = "Resolved review comment";
    },
    setAiAssistedSectionReviewed(line: number, reviewed: boolean) {
      const lines = this.activeDocument.text.split("\n");
      const index = Math.max(0, line - 1);
      const marker = lines[index] || "";
      if (!marker.includes("<!-- ai-assisted:") && !marker.includes("<!-- draft: AI paste cleanup review required -->")) return;
      lines[index] = rewriteAiAssistedMarker(marker, reviewed);
      this.updateText(lines.join("\n"));
      this.statusMessage = reviewed ? "Marked AI-assisted section as human-reviewed" : "Marked AI-assisted section as needing review";
    },
    setAiSourceReviewed(line: number, reviewed: boolean) {
      const lines = this.activeDocument.text.split("\n");
      const index = Math.max(0, line - 1);
      if (!isAiSourceFenceOpener(lines[index] || "")) return;
      if (!rewriteAiSourceReviewBlock(lines, index, reviewed)) return;
      this.updateText(lines.join("\n"));
      this.statusMessage = reviewed ? "Marked AI source as human-reviewed" : "Marked AI source as needing review";
    },
    async refreshGitStatus() {
      if (!this.gitIntegration.enabled) {
        Object.assign(this, clearGitVersioningState());
        return;
      }
      try {
        const status = await invoke<GitStatus>("get_git_status", { path: this.activeDocument?.path });
        this.gitStatus = status;
        if (gitStatusDetailsRequired(status, this.activeDocument?.path)) {
          await this.refreshGitHistory();
          await this.refreshGitDiff();
        } else {
          this.gitHistory = [];
          this.gitDiffText = "";
        }
      } catch {
        Object.assign(this, clearGitVersioningState());
      }
    },
    async refreshGitHistory() {
      const path = this.activeDocument?.path;
      if (!path) {
        this.gitHistory = [];
        return;
      }
      this.gitHistory = await invoke<GitHistoryEntry[]>("git_history", { request: { path } });
    },
    async refreshGitDiff() {
      const path = this.activeDocument?.path;
      if (!path) {
        this.gitDiffText = "";
        return;
      }
      this.gitDiffText = await invoke<string>("git_diff", { request: { path } });
    },
    async commitActive(message?: string) {
      const path = this.activeDocument?.path;
      const commitMessage = (message || this.commitMessage || `Update ${this.activeDocument.title}`).trim();
      if (!path) throw new Error("Save the document before committing it.");
      await invoke("commit_document_changes", { request: { path, message: commitMessage } });
      this.commitMessage = "";
      this.statusMessage = "Committed document changes";
      await this.refreshGitStatus();
    },
    async tagActiveRelease(tag?: string) {
      const path = this.activeDocument?.path;
      const releaseTag = (tag || this.releaseTag).trim();
      if (!path) throw new Error("Save the document before tagging it.");
      if (!releaseTag) throw new Error("Enter a release tag.");
      await invoke("tag_release", {
        request: { path, tag: releaseTag, message: `Release ${this.activeDocument.title} ${releaseTag}` },
      });
      this.releaseTag = "";
      this.statusMessage = `Tagged release ${releaseTag}`;
      await this.refreshGitStatus();
    },
    async restoreGitRevision(revision: string) {
      const path = this.activeDocument?.path;
      if (!path) throw new Error("Save the document before restoring a revision.");
      await this.snapshotBeforeDestructiveAction("pre-git-restore");
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("restore_git_revision", {
        request: { path, revision },
      });
      const doc = this.activeDocument;
      const restored = applyRestoredGitRevisionState(doc, response, revision);
      Object.assign(doc, restored.document);
      this.statusMessage = restored.statusMessage;
      await this.compileActive();
      await this.refreshGitStatus();
    },
    closeDocument(id: string) {
      const result = closeDocumentTabState(this.documents, this.activeId, this.recentlyClosed, id);
      if (!result) return;
      this.documents = result.documents;
      this.activeId = result.activeId;
      this.recentlyClosed = result.recentlyClosed;
      if (result.closedActiveDocument) {
        void this.compileActive();
      }
      void this.persistWorkspace();
    },
    togglePin(id: string) {
      const document = this.documents.find((item) => item.id === id);
      if (!document) return;
      this.setPinned(id, !document.pinned);
    },
    setPinned(id: string, pinned: boolean) {
      const result = setPinnedDocumentState(this.documents, id, pinned);
      if (!result) return;
      this.documents = result.documents;
      this.statusMessage = result.statusMessage;
      void this.persistWorkspace();
    },
    moveDocument(id: string, targetId: string, placement: "before" | "after") {
      const result = moveDocumentTabState(this.documents, id, targetId, placement);
      if (!result) return;
      this.documents = result.documents;
      this.statusMessage = result.statusMessage;
      void this.persistWorkspace();
    },
  },
});
