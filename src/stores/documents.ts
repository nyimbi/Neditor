import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { watch as watchFs, type UnwatchFn, type WatchEvent } from "@tauri-apps/plugin-fs";
import { Store } from "@tauri-apps/plugin-store";
import type {
  AiCleanupResponse,
  AiCleanupOptions,
  CompileResponse,
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

type AiPasteInsertMode = "insert" | "quote" | "replace" | "appendix";

interface ExportDefaults {
  includeManifest: boolean;
  includeComments: boolean;
  includeProvenance: boolean;
  includeGlossary: boolean;
}

interface PersistedWorkspace {
  theme?: "system" | "light" | "dark";
  wordWrap?: boolean;
  lineNumbers?: boolean;
  highContrast?: boolean;
  reducedMotion?: boolean;
  autosave?: boolean;
  autosaveDelayMs?: number;
  autoSnapshot?: boolean;
  snapshotIntervalMs?: number;
  editorFont?: string;
  previewFont?: string;
  editorLineHeight?: number;
  previewLineHeight?: number;
  exportTarget?: "html" | "pdf" | "docx" | "pptx" | "markdown-bundle";
  exportDefaults?: Partial<ExportDefaults>;
  recentFiles?: string[];
  recentFolders?: string[];
  recentlyClosed?: string[];
  pinnedFiles?: string[];
  workspaceRoot?: string | null;
  openFiles?: string[];
  activePath?: string | null;
  transformEnginePaths?: Record<string, string>;
  trustedTransformEngines?: Record<string, boolean>;
  transformInputModes?: Record<string, "stdin" | "file">;
  transformTimeoutMs?: number;
}

interface FileMetadataResponse {
  path?: string;
  exists: boolean;
  hash?: string | null;
  modified?: string | null;
}

interface ExternalConflict {
  path: string;
  reason: "root" | "include";
  message: string;
  externalHash: string;
  externalText?: string;
}

interface DocumentWatchEvent {
  path: string;
  reason: "root" | "include";
  kind: string;
  hash?: string | null;
  modified?: string | null;
}

function quoteMarkdown(text: string) {
  return text
    .split(/\r?\n/)
    .map((line) => (line ? `> ${line}` : ">"))
    .join("\n");
}

function parseAiAssistedMarker(line: string) {
  const content = line.match(/<!--\s*ai-assisted:(.*?)-->/)?.[1] || "";
  const fields = new Map<string, string>();
  for (const part of content
    .split("|")
    .map((entry) => entry.trim())
    .filter(Boolean)) {
    const pair = part.match(/^([^:=]+)\s*[:=]\s*(.*)$/);
    if (pair) {
      fields.set(pair[1].trim(), pair[2].trim());
    } else if (["human-reviewed", "needs-review", "unreviewed"].includes(part)) {
      fields.set("status", part);
    }
  }
  return fields;
}

function serializeAiAssistedMarker(fields: Map<string, string>) {
  const orderedKeys = ["status", "reviewedBy", "reviewedAt", "source", "promptSummary"];
  const parts = orderedKeys
    .filter((key) => fields.has(key))
    .map((key) => `${key}=${fields.get(key) || ""}`);
  for (const [key, value] of fields) {
    if (!orderedKeys.includes(key)) {
      parts.push(`${key}=${value}`);
    }
  }
  return `<!-- ai-assisted: ${parts.join(" | ")} -->`;
}

function rewriteAiAssistedMarker(line: string, reviewed: boolean) {
  const fields = line.includes("<!-- ai-assisted:")
    ? parseAiAssistedMarker(line)
    : new Map<string, string>([
        ["source", "AI paste cleanup"],
        ["promptSummary", "AI paste cleanup review required"],
      ]);
  fields.set("status", reviewed ? "human-reviewed" : "needs-review");
  fields.set("reviewedBy", reviewed ? "local" : "");
  fields.set("reviewedAt", reviewed ? new Date().toISOString() : "");
  return serializeAiAssistedMarker(fields);
}

function rewriteYamlLikeField(lines: string[], key: string, value: string) {
  const index = lines.findIndex((line) => line.trimStart().startsWith(`${key}:`));
  const replacement = `${key}: ${value}`;
  if (index >= 0) {
    lines[index] = replacement;
  } else {
    lines.push(replacement);
  }
}

function rewriteAiSourceBlock(lines: string[], startIndex: number, reviewed: boolean) {
  const endIndex = lines.findIndex((line, index) => index > startIndex && line.trim() === "```");
  if (endIndex < 0) return false;
  const body = lines.slice(startIndex + 1, endIndex);
  rewriteYamlLikeField(body, "status", reviewed ? "human-reviewed" : "needs-review");
  rewriteYamlLikeField(body, "reviewedBy", reviewed ? "local" : "");
  rewriteYamlLikeField(body, "reviewedAt", reviewed ? new Date().toISOString() : "");
  lines.splice(startIndex + 1, endIndex - startIndex - 1, ...body);
  return true;
}

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

function titleFromPath(path: string | null) {
  if (!path) return "Untitled";
  return path.split(/[\\/]/).pop() || path;
}

function folderFromPath(path: string | null) {
  if (!path) return null;
  const separator = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return separator > 0 ? path.slice(0, separator) : null;
}

function watchEventIsAccessOnly(event: WatchEvent) {
  return typeof event.type === "object" && "access" in event.type;
}

function stringifyWatchEventKind(kind: WatchEvent["type"]) {
  if (typeof kind === "string") return kind;
  return Object.keys(kind)[0] || "other";
}

function clampLineHeight(value: number) {
  return Math.min(Math.max(Number(value) || 1.55, 1), 2.4);
}

function clampAutosaveDelay(value: number) {
  return Math.min(Math.max(Number(value) || 1500, 500), 30000);
}

function clampSnapshotInterval(value: number) {
  return Math.min(Math.max(Number(value) || 300000, 30000), 3600000);
}

function normalizeExportDefaults(defaults: Partial<ExportDefaults>): ExportDefaults {
  return {
    includeManifest: typeof defaults.includeManifest === "boolean" ? defaults.includeManifest : true,
    includeComments: typeof defaults.includeComments === "boolean" ? defaults.includeComments : true,
    includeProvenance: typeof defaults.includeProvenance === "boolean" ? defaults.includeProvenance : true,
    includeGlossary: typeof defaults.includeGlossary === "boolean" ? defaults.includeGlossary : true,
  };
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
        dirty: true,
      },
    ] as OpenDocument[],
    activeId: "",
    mode: "split" as "split" | "source" | "preview" | "focus" | "export" | "review" | "presentation",
    sidebar: "outline" as
      | "files"
      | "outline"
      | "diagnostics"
      | "tables"
      | "references"
      | "exports"
      | "versioning"
      | "review"
      | "settings",
    theme: "system" as "system" | "light" | "dark",
    wordWrap: true,
    lineNumbers: true,
    highContrast: false,
    reducedMotion: false,
    autosave: false,
    autosaveDelayMs: 1500,
    autoSnapshot: false,
    snapshotIntervalMs: 300000,
    editorFont: "Menlo, Consolas, monospace",
    previewFont: "Inter, Arial, sans-serif",
    editorLineHeight: 1.55,
    previewLineHeight: 1.65,
    exportTarget: "html" as "html" | "pdf" | "docx" | "pptx" | "markdown-bundle",
    exportDefaults: {
      includeManifest: true,
      includeComments: true,
      includeProvenance: true,
      includeGlossary: true,
    } as ExportDefaults,
    gitStatus: null as GitStatus | null,
    statusMessage: "Ready",
    lastError: "",
    externalHash: "",
    externalConflict: null as ExternalConflict | null,
    ignoredConflictHash: "",
    watchSignature: "",
    watchedPaths: [] as string[],
    transformEngines: [] as TransformEngineMetadata[],
    transformEnginePaths: {} as Record<string, string>,
    trustedTransformEngines: {} as Record<string, boolean>,
    transformInputModes: {} as Record<string, "stdin" | "file">,
    transformTimeoutMs: 5000,
    snapshots: [] as SnapshotListItem[],
    exportReadiness: null as ExportReadinessReport | null,
    aiCleanupIssues: [] as string[],
    aiCleanupPreview: null as AiCleanupResponse | null,
    recentFiles: [] as string[],
    recentFolders: [] as string[],
    recentlyClosed: [] as string[],
    workspaceRoot: null as string | null,
    workspaceFiles: [] as WorkspaceFileEntry[],
    gitHistory: [] as GitHistoryEntry[],
    gitDiffText: "",
    releaseTag: "",
    commitMessage: "",
  }),
  getters: {
    activeDocument(state): OpenDocument {
      return state.documents.find((document) => document.id === state.activeId) || state.documents[0];
    },
    windowTitle(): string {
      const doc = this.activeDocument;
      return `${doc.dirty ? "* " : ""}${doc.title} - NEditor`;
    },
    externalTransformEngines(state): TransformEngineMetadata[] {
      return state.transformEngines.filter((engine) => engine.requiresExecution);
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
    async loadPreferences() {
      try {
        preferencesStore = await Store.load("settings.json");
        const persisted = (await preferencesStore.get<PersistedWorkspace>("workspace")) || {};
        if (persisted.theme) this.theme = persisted.theme;
        if (typeof persisted.wordWrap === "boolean") this.wordWrap = persisted.wordWrap;
        if (typeof persisted.lineNumbers === "boolean") this.lineNumbers = persisted.lineNumbers;
        if (typeof persisted.highContrast === "boolean") this.highContrast = persisted.highContrast;
        if (typeof persisted.reducedMotion === "boolean") this.reducedMotion = persisted.reducedMotion;
        if (typeof persisted.autosave === "boolean") this.autosave = persisted.autosave;
        if (typeof persisted.autosaveDelayMs === "number") this.autosaveDelayMs = clampAutosaveDelay(persisted.autosaveDelayMs);
        if (typeof persisted.autoSnapshot === "boolean") this.autoSnapshot = persisted.autoSnapshot;
        if (typeof persisted.snapshotIntervalMs === "number") this.snapshotIntervalMs = clampSnapshotInterval(persisted.snapshotIntervalMs);
        if (persisted.editorFont) this.editorFont = persisted.editorFont;
        if (persisted.previewFont) this.previewFont = persisted.previewFont;
        if (typeof persisted.editorLineHeight === "number") this.editorLineHeight = clampLineHeight(persisted.editorLineHeight);
        if (typeof persisted.previewLineHeight === "number") this.previewLineHeight = clampLineHeight(persisted.previewLineHeight);
        if (persisted.exportTarget) this.exportTarget = persisted.exportTarget;
        if (persisted.exportDefaults) this.exportDefaults = normalizeExportDefaults(persisted.exportDefaults);
        this.recentFiles = persisted.recentFiles || [];
        this.recentFolders = persisted.recentFolders || [];
        this.recentlyClosed = persisted.recentlyClosed || [];
        this.workspaceRoot = persisted.workspaceRoot || null;
        this.transformEnginePaths = persisted.transformEnginePaths || {};
        this.trustedTransformEngines = persisted.trustedTransformEngines || {};
        this.transformInputModes = persisted.transformInputModes || {};
        if (typeof persisted.transformTimeoutMs === "number") {
          this.transformTimeoutMs = Math.min(Math.max(persisted.transformTimeoutMs, 1), 30000);
        }
        if (persisted.openFiles?.length) {
          await this.restoreWorkspace(persisted.openFiles, persisted.activePath || null, persisted.pinnedFiles || []);
        }
      } catch (error) {
        this.lastError = error instanceof Error ? error.message : String(error);
      }
    },
    async persistWorkspace() {
      if (!preferencesStore) return;
      const workspace: PersistedWorkspace = {
        theme: this.theme,
        wordWrap: this.wordWrap,
        lineNumbers: this.lineNumbers,
        highContrast: this.highContrast,
        reducedMotion: this.reducedMotion,
        autosave: this.autosave,
        autosaveDelayMs: this.autosaveDelayMs,
        autoSnapshot: this.autoSnapshot,
        snapshotIntervalMs: this.snapshotIntervalMs,
        editorFont: this.editorFont,
        previewFont: this.previewFont,
        editorLineHeight: this.editorLineHeight,
        previewLineHeight: this.previewLineHeight,
        exportTarget: this.exportTarget,
        exportDefaults: this.exportDefaults,
        recentFiles: this.recentFiles.slice(0, 20),
        recentFolders: this.recentFolders.slice(0, 12),
        recentlyClosed: this.recentlyClosed.slice(0, 20),
        workspaceRoot: this.workspaceRoot,
        openFiles: this.documents.map((document) => document.path).filter((path): path is string => Boolean(path)),
        pinnedFiles: this.documents
          .filter((document) => document.pinned && document.path)
          .map((document) => document.path as string),
        activePath: this.activeDocument?.path || null,
        transformEnginePaths: this.transformEnginePaths,
        trustedTransformEngines: this.trustedTransformEngines,
        transformInputModes: this.transformInputModes,
        transformTimeoutMs: this.transformTimeoutMs,
      };
      await preferencesStore.set("workspace", workspace);
      await preferencesStore.save();
    },
    async restoreWorkspace(paths: string[], activePath: string | null, pinnedFiles: string[] = []) {
      const restored: OpenDocument[] = [];
      for (const path of paths) {
        try {
          const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", { path });
          restored.push({
            id: crypto.randomUUID(),
            path: response.path,
            title: titleFromPath(response.path),
            text: response.text,
            savedHash: response.hash,
            dirty: false,
            pinned: pinnedFiles.includes(response.path),
            modified: response.modified,
          });
        } catch {
          this.recentFiles = this.recentFiles.filter((recent) => recent !== path);
        }
      }
      if (!restored.length) return;
      this.documents = restored;
      this.activeId = restored.find((document) => document.path === activePath)?.id || restored[0].id;
    },
    newDocument() {
      const document: OpenDocument = {
        id: crypto.randomUUID(),
        path: null,
        title: "Untitled",
        text: starterDocument,
        savedHash: fallbackHash(starterDocument),
        dirty: true,
      };
      this.documents.push(document);
      this.activeId = document.id;
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
      const document: OpenDocument = {
        id: crypto.randomUUID(),
        path: response.path,
        title: titleFromPath(response.path),
        text: response.text,
        savedHash: response.hash,
        dirty: false,
        modified: response.modified,
      };
      this.documents.push(document);
      this.activeId = document.id;
      this.statusMessage = `Opened ${document.title}`;
      this.rememberFile(document.path);
      this.recentlyClosed = this.recentlyClosed.filter((recent) => recent !== document.path);
      if (!this.workspaceRoot) {
        const folder = folderFromPath(document.path);
        if (folder) await this.openFolder(folder);
      }
      await this.compileActive();
      await this.refreshGitStatus();
      await this.persistWorkspace();
    },
    async openFolder(path: string) {
      this.workspaceRoot = path;
      this.rememberFolder(path);
      await this.refreshWorkspace();
      this.sidebar = "files";
      this.statusMessage = `Opened workspace ${titleFromPath(path)}`;
      await this.persistWorkspace();
    },
    async refreshWorkspace() {
      if (!this.workspaceRoot) {
        this.workspaceFiles = [];
        return;
      }
      try {
        this.workspaceFiles = await invoke<WorkspaceFileEntry[]>("list_workspace_files", {
          request: { root: this.workspaceRoot },
        });
        this.lastError = "";
      } catch (error) {
        this.workspaceFiles = [];
        this.lastError = error instanceof Error ? error.message : String(error);
      }
    },
    async saveActive(path?: string) {
      const doc = this.activeDocument;
      const target = path || doc.path;
      if (!target) throw new Error("Choose a save path before saving this document.");
      const isExistingDocumentSave = Boolean(doc.path && target === doc.path);
      if (isExistingDocumentSave) {
        const metadata = await invoke<FileMetadataResponse>("file_metadata", { path: target });
        if (metadata.exists && metadata.hash && metadata.hash !== doc.savedHash) {
          await this.openExternalConflict(target, "root", "The root file changed outside NEditor before save.", metadata.hash);
          this.statusMessage = "Save blocked; resolve external changes first";
          return;
        }
      }
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("save_file", {
        request: { path: target, text: doc.text, expected_hash: isExistingDocumentSave ? doc.savedHash : null },
      });
      doc.path = response.path;
      doc.title = titleFromPath(response.path);
      doc.savedHash = response.hash;
      doc.modified = response.modified;
      doc.dirty = false;
      this.statusMessage = `Saved ${doc.title}`;
      this.rememberFile(doc.path);
      if (this.workspaceRoot) await this.refreshWorkspace();
      await this.refreshGitStatus();
      await this.persistWorkspace();
    },
    async revertActive() {
      const doc = this.activeDocument;
      await this.snapshotBeforeDestructiveAction("pre-revert");
      if (!doc.path) {
        doc.text = starterDocument;
        doc.savedHash = fallbackHash(starterDocument);
        doc.dirty = true;
        await this.compileActive();
        this.statusMessage = "Reverted untitled document to starter content";
        return;
      }
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", {
        path: doc.path,
      });
      doc.text = response.text;
      doc.savedHash = response.hash;
      doc.modified = response.modified;
      doc.dirty = false;
      this.statusMessage = `Reverted ${doc.title} to saved content`;
      await this.compileActive();
      await this.refreshGitStatus();
    },
    async renameActive(path: string) {
      const doc = this.activeDocument;
      if (!doc.path) throw new Error("Save the document before renaming it.");
      const metadata = await invoke<{ path: string; exists: boolean; hash?: string; modified?: string }>("rename_file", {
        request: { from: doc.path, to: path },
      });
      doc.path = metadata.path;
      doc.title = titleFromPath(metadata.path);
      doc.savedHash = metadata.hash || doc.savedHash;
      doc.modified = metadata.modified;
      this.statusMessage = `Renamed ${doc.title}`;
      this.rememberFile(doc.path);
      if (this.workspaceRoot) await this.refreshWorkspace();
      await this.refreshGitStatus();
      await this.persistWorkspace();
    },
    async duplicateActive(path: string) {
      const doc = this.activeDocument;
      if (!doc.path) await this.saveActive(path);
      const source = this.activeDocument.path;
      if (!source) throw new Error("Save the document before duplicating it.");
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("duplicate_file", {
        request: { from: source, to: path },
      });
      const duplicate: OpenDocument = {
        id: crypto.randomUUID(),
        path: response.path,
        title: titleFromPath(response.path),
        text: response.text,
        savedHash: response.hash,
        dirty: false,
        modified: response.modified,
      };
      this.documents.push(duplicate);
      this.activeId = duplicate.id;
      this.statusMessage = `Duplicated ${duplicate.title}`;
      this.rememberFile(duplicate.path);
      if (this.workspaceRoot) await this.refreshWorkspace();
      await this.compileActive();
      await this.persistWorkspace();
    },
    async revealActive() {
      const doc = this.activeDocument;
      if (!doc.path) throw new Error("Save the document before revealing it.");
      await invoke("reveal_path", { path: doc.path });
    },
    updateText(text: string) {
      const doc = this.activeDocument;
      doc.text = text;
      doc.dirty = fallbackHash(text) !== doc.savedHash;
      void this.compileActive();
    },
    async compileActive() {
      const doc = this.activeDocument;
      if (!doc) return;
      try {
        doc.compile = await invoke<CompileResponse>("compile_document", {
          request: { text: doc.text, file_path: doc.path },
        });
        doc.title = String(doc.compile.semantic.title || titleFromPath(doc.path));
        this.statusMessage = `${doc.compile.diagnostics.length} diagnostics`;
        this.lastError = "";
        await this.syncFileWatcher();
      } catch (error) {
        this.lastError = error instanceof Error ? error.message : String(error);
      }
    },
    async syncFileWatcher() {
      const doc = this.activeDocument;
      if (!doc?.path) {
        if (this.watchSignature) {
          await this.stopFileWatcher();
        }
        this.watchSignature = "";
        this.watchedPaths = [];
        return;
      }
      const includedPaths = (doc.compile?.export_manifest.included_files || []).map((file) => file.path);
      const watchSnapshot = await invoke<WatchFileResponse>("start_file_watcher", {
        request: { root: doc.path, included: includedPaths },
      });
      const watchPaths = watchSnapshot.paths.filter((file) => file.exists).map((file) => file.path);
      const driver = watchSnapshot.native_watcher ? "native" : "plugin";
      const signature = `${driver}\n${watchPaths.join("\n")}`;
      if (signature === this.watchSignature) return;
      this.detachFileWatchListeners();
      if (!watchPaths.length) {
        this.watchSignature = "";
        this.watchedPaths = [];
        return;
      }
      if (watchSnapshot.native_watcher) {
        await this.attachBackendFileWatchListeners(doc.path, includedPaths);
      } else {
        unwatchFileChanges = await watchFs(
          watchPaths,
          (event) => {
            void this.handleFsWatchEvent(event, doc.path as string, includedPaths);
          },
          { delayMs: 250 },
        );
      }
      this.watchSignature = signature;
      this.watchedPaths = watchPaths;
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
    },
    async attachBackendFileWatchListeners(rootPath: string, includedPaths: string[]) {
      unwatchFileChanges = await listen<BackendWatchEvent>("neditor-file-watch-event", (event) => {
        void this.handleBackendWatchEvent(event.payload, rootPath, includedPaths);
      });
      unwatchFileErrors = await listen<string>("neditor-file-watch-error", (event) => {
        this.lastError = event.payload;
        this.statusMessage = "File watcher failed";
      });
    },
    async handleFsWatchEvent(event: WatchEvent, rootPath: string, includedPaths: string[]) {
      const paths = event.paths.length ? event.paths : [rootPath];
      for (const path of paths) {
        const reason = path === rootPath ? "root" : includedPaths.includes(path) ? "include" : null;
        if (!reason || watchEventIsAccessOnly(event)) continue;
        const metadata = await invoke<FileMetadataResponse>("file_metadata", { path });
        await this.handleWatchedFileChange({
          path,
          reason,
          kind: stringifyWatchEventKind(event.type),
          hash: metadata.hash,
          modified: metadata.modified,
        });
      }
    },
    async handleBackendWatchEvent(event: BackendWatchEvent, rootPath: string, includedPaths: string[]) {
      const paths = event.paths.length ? event.paths : [rootPath];
      for (const path of paths) {
        const reason = path === rootPath ? "root" : includedPaths.includes(path) ? "include" : null;
        if (!reason) continue;
        const metadata = await invoke<FileMetadataResponse>("file_metadata", { path });
        await this.handleWatchedFileChange({
          path,
          reason,
          kind: event.kind,
          hash: metadata.hash,
          modified: metadata.modified,
        });
      }
    },
    async handleWatchedFileChange(event: DocumentWatchEvent) {
      const doc = this.activeDocument;
      if (!doc?.path) return;
      const watched = this.watchedPaths.length
        ? this.watchedPaths
        : [doc.path, ...(doc.compile?.export_manifest.included_files || []).map((file) => file.path)];
      if (!watched.includes(event.path)) return;
      await this.refreshExternalState(event);
    },
    async refreshExternalState(event?: DocumentWatchEvent) {
      const doc = this.activeDocument;
      if (!doc.path) return;
      const metadata = await invoke<FileMetadataResponse>("file_metadata", { path: doc.path });
      this.externalHash = metadata.hash || "";
      const rootEventIsRealChange = event?.reason === "root" && (!event.hash || event.hash !== doc.savedHash);
      const mainChanged =
        rootEventIsRealChange || Boolean(metadata.exists && metadata.hash && metadata.hash !== doc.savedHash);
      const includeChanged = event?.reason === "include" || (await this.hasChangedIncludedFiles(doc));
      if (metadata.hash && this.ignoredConflictHash === metadata.hash && doc.dirty) return;
      if ((mainChanged || includeChanged) && doc.dirty) {
        await this.openExternalConflict(
          event?.path || doc.path,
          mainChanged ? "root" : "include",
          mainChanged
            ? "The root file changed outside NEditor while local edits are unsaved."
            : "An included file changed while local edits are unsaved.",
          (mainChanged ? metadata.hash : event?.hash) || "include-change",
        );
        this.statusMessage = mainChanged
          ? "External changes detected; compare before overwriting"
          : "Included file changes detected; save or compare before recompiling";
        return;
      }
      if (mainChanged && metadata.hash) {
        const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", { path: doc.path });
        doc.text = response.text;
        doc.savedHash = response.hash;
        doc.modified = response.modified;
        doc.dirty = false;
        this.externalConflict = null;
        this.statusMessage = "Reloaded external changes";
        await this.compileActive();
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
      if (conflict.reason === "root") {
        await this.snapshotBeforeDestructiveAction("pre-accept-external");
        const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("read_file", {
          path: conflict.path,
        });
        const doc = this.activeDocument;
        doc.text = response.text;
        doc.savedHash = response.hash;
        doc.modified = response.modified;
        doc.dirty = false;
        this.statusMessage = "Accepted external file changes";
      } else {
        await this.compileActive();
        this.statusMessage = "Accepted included file changes";
      }
      this.externalConflict = null;
      this.ignoredConflictHash = "";
      await this.refreshGitStatus();
    },
    keepLocalChanges() {
      if (this.externalConflict?.externalHash) {
        this.ignoredConflictHash = this.externalConflict.externalHash;
      }
      this.externalConflict = null;
      this.statusMessage = "Keeping local edits";
    },
    async saveLocalConflictCopy(path: string) {
      await this.saveActive(path);
      this.externalConflict = null;
      this.ignoredConflictHash = "";
      this.statusMessage = "Saved local edits as a copy";
    },
    async applyConflictMerge(text: string) {
      const conflict = this.externalConflict;
      if (!conflict || conflict.reason !== "root") return;
      await this.snapshotBeforeDestructiveAction("pre-conflict-merge");
      const doc = this.activeDocument;
      doc.text = text;
      doc.savedHash = conflict.externalHash;
      doc.dirty = text !== (conflict.externalText || "");
      this.externalHash = conflict.externalHash;
      this.externalConflict = null;
      this.ignoredConflictHash = "";
      this.statusMessage = "Merged external changes into the working document";
      await this.compileActive();
      await this.refreshGitStatus();
    },
    async hasChangedIncludedFiles(doc: OpenDocument) {
      const includedFiles = doc.compile?.export_manifest.included_files || [];
      if (!includedFiles.length) return false;
      for (const included of includedFiles) {
        try {
          const metadata = await invoke<FileMetadataResponse>("file_metadata", { path: included.path });
          if (!metadata.exists || metadata.hash !== included.hash) return true;
        } catch {
          return true;
        }
      }
      return false;
    },
    async openExternalConflict(path: string, reason: "root" | "include", message: string, externalHash: string) {
      let externalText = "";
      try {
        externalText = (await invoke<{ text: string }>("read_file", { path })).text;
      } catch {
        externalText = reason === "include" ? "The changed included file could not be read. It may have been deleted or moved." : "";
      }
      this.externalConflict = {
        path,
        reason,
        message,
        externalHash,
        externalText,
      };
    },
    async exportActive(path: string) {
      const doc = this.activeDocument;
      await this.createSnapshot("pre-export");
      const response = await invoke<{ output_path: string; manifest_path?: string }>("export_document", {
        request: {
          text: doc.text,
          file_path: doc.path,
          target: this.exportTarget,
          output_path: path,
          options: this.exportOptionsForActive(),
        },
      });
      this.statusMessage = `Exported ${response.output_path}${response.manifest_path ? " with manifest" : ""}`;
      await this.listSnapshots();
    },
    exportOptionsForActive() {
      const defaults = normalizeExportDefaults(this.exportDefaults);
      return {
        includeManifest: defaults.includeManifest,
        includeComments: defaults.includeComments,
        includeProvenance: defaults.includeProvenance,
        includeGlossary: defaults.includeGlossary,
        watermark: this.activeDocument.compile?.semantic.status === "draft" ? "DRAFT" : "",
      };
    },
    async createSnapshot(label = "manual") {
      const doc = this.activeDocument;
      return invoke<{ snapshot_path: string }>("create_snapshot", {
        request: { text: doc.text, file_path: doc.path, label },
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
      this.snapshots = await invoke<SnapshotListItem[]>("list_snapshots", { filePath: this.activeDocument?.path });
    },
    async restoreSnapshot(snapshotPath: string) {
      await this.snapshotBeforeDestructiveAction("pre-snapshot-restore");
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("restore_snapshot", {
        snapshotPath,
      });
      const doc = this.activeDocument;
      doc.text = response.text;
      doc.dirty = true;
      this.statusMessage = `Restored snapshot ${response.path}`;
      await this.compileActive();
    },
    async prepareForExport() {
      const doc = this.activeDocument;
      this.exportReadiness = await invoke<ExportReadinessReport>("prepare_for_export", {
        request: {
          text: doc.text,
          file_path: doc.path,
          target: this.exportTarget,
          options: this.exportOptionsForActive(),
        },
      });
      this.statusMessage = this.exportReadiness.ready
        ? "Document is ready for export"
        : `${this.exportReadiness.error_count} errors, ${this.exportReadiness.warning_count} warnings before export`;
    },
    async setTransformEnginePath(name: string, path: string) {
      this.transformEnginePaths = { ...this.transformEnginePaths, [name]: path };
      await this.persistWorkspace();
    },
    async setTransformTrust(name: string, trusted: boolean) {
      this.trustedTransformEngines = { ...this.trustedTransformEngines, [name]: trusted };
      await this.persistWorkspace();
    },
    async setTransformInputMode(name: string, mode: "stdin" | "file") {
      this.transformInputModes = { ...this.transformInputModes, [name]: mode };
      await this.persistWorkspace();
    },
    async setTransformTimeout(timeoutMs: number) {
      this.transformTimeoutMs = Math.min(Math.max(Number(timeoutMs) || 1, 1), 30000);
      await this.persistWorkspace();
    },
    async testExternalTransform(name: string) {
      const engine = this.transformEngines.find((candidate) => candidate.name === name);
      try {
        const response = await invoke<{ diagnostics: Array<{ message: string }>; cache_key: string }>("run_external_transform", {
          request: {
            name,
            body: "neditor transform probe\n",
            engine_path: this.transformEnginePaths[name] || "",
            trusted: Boolean(this.trustedTransformEngines[name]),
            input_mode: this.transformInputModes[name] || "stdin",
            timeout_ms: this.transformTimeoutMs,
            max_input_bytes: engine?.limits.maxInputBytes,
            max_output_bytes: engine?.limits.maxOutputBytes,
          },
        });
        const detail = response.diagnostics[0]?.message || response.cache_key;
        this.statusMessage = `${name} transform probe succeeded: ${detail}`;
      } catch (error) {
        this.lastError = error instanceof Error ? error.message : String(error);
        this.statusMessage = `${name} transform probe failed`;
      }
    },
    async previewAiPaste(text: string, options: AiCleanupOptions) {
      const response = await invoke<AiCleanupResponse>("cleanup_ai_paste", {
        request: {
          text,
          add_provenance: options.addProvenance,
          mark_as_draft: options.markAsDraft,
          insert_citation_todos: options.insertCitationTodos,
        },
      });
      this.aiCleanupPreview = response;
      this.aiCleanupIssues = response.issues;
      return response;
    },
    insertAiPaste(response: AiCleanupResponse, mode: AiPasteInsertMode) {
      if (mode === "replace") {
        this.updateText(response.cleaned_markdown);
      } else if (mode === "quote") {
        this.updateText(`${this.activeDocument.text}\n\n${quoteMarkdown(response.cleaned_markdown)}\n`);
      } else if (mode === "appendix") {
        this.updateText(`${this.activeDocument.text}\n\n## AI Draft Appendix\n\n${response.cleaned_markdown}\n`);
      } else {
        this.updateText(`${this.activeDocument.text}\n\n${response.cleaned_markdown}\n`);
      }
      this.statusMessage = `Cleaned AI paste with ${response.issues.length} issue notes`;
    },
    async cleanAiPaste(text: string, mode: AiPasteInsertMode, options: AiCleanupOptions) {
      const response = await this.previewAiPaste(text, options);
      this.insertAiPaste(response, mode);
    },
    insertReviewComment(text: string) {
      const comment = (text.trim() || "Review comment").replace(/-->/g, "->");
      const createdAt = new Date().toISOString();
      this.updateText(`${this.activeDocument.text}\n\n<!-- comment: unresolved | author: local | at: ${createdAt} | ${comment} -->\n`);
      this.statusMessage = "Inserted review comment";
    },
    resolveReviewComment(line: number) {
      const lines = this.activeDocument.text.split("\n");
      const index = Math.max(0, line - 1);
      if (!lines[index]?.includes("<!-- comment:")) return;
      lines[index] = lines[index].replace("unresolved", "resolved");
      this.updateText(lines.join("\n"));
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
      if (!lines[index]?.trimStart().startsWith("```ai-source")) return;
      if (!rewriteAiSourceBlock(lines, index, reviewed)) return;
      this.updateText(lines.join("\n"));
      this.statusMessage = reviewed ? "Marked AI source as human-reviewed" : "Marked AI source as needing review";
    },
    async refreshGitStatus() {
      try {
        this.gitStatus = await invoke<GitStatus>("get_git_status", { path: this.activeDocument?.path });
        if (this.activeDocument?.path) {
          await this.refreshGitHistory();
          await this.refreshGitDiff();
        }
      } catch {
        this.gitStatus = null;
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
    },
    async restoreGitRevision(revision: string) {
      const path = this.activeDocument?.path;
      if (!path) throw new Error("Save the document before restoring a revision.");
      await this.snapshotBeforeDestructiveAction("pre-git-restore");
      const response = await invoke<{ path: string; text: string; hash: string; modified?: string }>("restore_git_revision", {
        request: { path, revision },
      });
      const doc = this.activeDocument;
      doc.text = response.text;
      doc.savedHash = response.hash;
      doc.dirty = true;
      this.statusMessage = `Restored revision ${revision.slice(0, 12)}`;
      await this.compileActive();
      await this.refreshGitStatus();
    },
    closeDocument(id: string) {
      if (this.documents.length === 1) return;
      const index = this.documents.findIndex((document) => document.id === id);
      if (index >= 0) {
        const [closed] = this.documents.slice(index, index + 1);
        if (closed?.path) {
          this.recentlyClosed = [closed.path, ...this.recentlyClosed.filter((recent) => recent !== closed.path)].slice(0, 20);
        }
        this.documents.splice(index, 1);
        this.activeId = this.documents[Math.max(0, index - 1)].id;
        void this.persistWorkspace();
      }
    },
    togglePin(id: string) {
      const document = this.documents.find((item) => item.id === id);
      if (!document) return;
      document.pinned = !document.pinned;
      this.documents.sort((left, right) => Number(Boolean(right.pinned)) - Number(Boolean(left.pinned)));
      void this.persistWorkspace();
    },
    rememberFile(path: string | null) {
      if (!path) return;
      this.recentFiles = [path, ...this.recentFiles.filter((recent) => recent !== path)].slice(0, 20);
    },
    rememberFolder(path: string | null) {
      if (!path) return;
      this.recentFolders = [path, ...this.recentFolders.filter((recent) => recent !== path)].slice(0, 12);
    },
  },
});
