import {
  clampAutosaveDelay,
  clampFontSize,
  clampLineHeight,
  clampPaneRatio,
  clampSnapshotInterval,
  clampToolbarTextSize,
  type EditorKeymapMode,
  type PersistedWorkspace,
  type PreviewTheme,
  type SidebarPanel,
  type SnapshotStorage,
  type ThemePreference,
  type ToolbarDisplay,
  type WorkbenchMode,
} from "./workspacePersistence.js";

export interface UiPreferencesState {
  theme: ThemePreference;
  previewTheme: PreviewTheme;
  toolbarDisplay: ToolbarDisplay;
  toolbarTextSize: number;
  toolbarCollapsedRows: string[];
  editorPaneRatio: number;
  splitSourcePanes: boolean;
  editorKeymapMode: EditorKeymapMode;
  wordWrap: boolean;
  lineNumbers: boolean;
  codeFolding: boolean;
  highContrast: boolean;
  reducedMotion: boolean;
  autosave: boolean;
  autosaveDelayMs: number;
  autoSnapshot: boolean;
  snapshotIntervalMs: number;
  snapshotStorage: SnapshotStorage;
  editorFont: string;
  previewFont: string;
  editorFontSize: number;
  previewFontSize: number;
  editorLineHeight: number;
  previewLineHeight: number;
  mode: WorkbenchMode;
  sidebar: SidebarPanel;
}

const themePreferences = new Set<ThemePreference>(["system", "light", "dark"]);
const previewThemes = new Set<PreviewTheme>(["match", "light", "dark"]);
const toolbarDisplays = new Set<ToolbarDisplay>(["both", "icons", "text"]);
const editorKeymapModes = new Set<EditorKeymapMode>(["default", "emacs", "vim"]);
const snapshotStorages = new Set<SnapshotStorage>(["app-data", "project-local"]);
const workbenchModes = new Set<WorkbenchMode>(["split", "source", "preview", "focus", "outline", "export", "review", "presentation"]);
const sidebarPanels = new Set<SidebarPanel>([
  "files",
  "outline",
  "diagnostics",
  "tables",
  "templates",
  "layout",
  "references",
  "exports",
  "versioning",
  "review",
  "help",
  "settings",
]);

function validSetValue<T extends string>(values: Set<T>, value: unknown, fallback: T) {
  return typeof value === "string" && values.has(value as T) ? (value as T) : fallback;
}

function persistedBoolean(value: unknown, fallback: boolean) {
  return typeof value === "boolean" ? value : fallback;
}

function persistedString(value: unknown, fallback: string) {
  return typeof value === "string" && value ? value : fallback;
}

export function applyPersistedUiPreferences(current: UiPreferencesState, persisted: PersistedWorkspace): UiPreferencesState {
  return {
    theme: validSetValue(themePreferences, persisted.theme, current.theme),
    previewTheme: validSetValue(previewThemes, persisted.previewTheme, current.previewTheme),
    toolbarDisplay: validSetValue(toolbarDisplays, persisted.toolbarDisplay, current.toolbarDisplay),
    toolbarTextSize: typeof persisted.toolbarTextSize === "number" ? clampToolbarTextSize(persisted.toolbarTextSize) : current.toolbarTextSize,
    toolbarCollapsedRows: Array.isArray(persisted.toolbarCollapsedRows) ? persisted.toolbarCollapsedRows : current.toolbarCollapsedRows,
    editorPaneRatio: typeof persisted.editorPaneRatio === "number" ? clampPaneRatio(persisted.editorPaneRatio) : current.editorPaneRatio,
    splitSourcePanes: persistedBoolean(persisted.splitSourcePanes, current.splitSourcePanes),
    editorKeymapMode: validSetValue(editorKeymapModes, persisted.editorKeymapMode, current.editorKeymapMode),
    wordWrap: persistedBoolean(persisted.wordWrap, current.wordWrap),
    lineNumbers: persistedBoolean(persisted.lineNumbers, current.lineNumbers),
    codeFolding: persistedBoolean(persisted.codeFolding, current.codeFolding),
    highContrast: persistedBoolean(persisted.highContrast, current.highContrast),
    reducedMotion: persistedBoolean(persisted.reducedMotion, current.reducedMotion),
    autosave: persistedBoolean(persisted.autosave, current.autosave),
    autosaveDelayMs: typeof persisted.autosaveDelayMs === "number" ? clampAutosaveDelay(persisted.autosaveDelayMs) : current.autosaveDelayMs,
    autoSnapshot: persistedBoolean(persisted.autoSnapshot, current.autoSnapshot),
    snapshotIntervalMs:
      typeof persisted.snapshotIntervalMs === "number" ? clampSnapshotInterval(persisted.snapshotIntervalMs) : current.snapshotIntervalMs,
    snapshotStorage: validSetValue(snapshotStorages, persisted.snapshotStorage, current.snapshotStorage),
    editorFont: persistedString(persisted.editorFont, current.editorFont),
    previewFont: persistedString(persisted.previewFont, current.previewFont),
    editorFontSize: typeof persisted.editorFontSize === "number" ? clampFontSize(persisted.editorFontSize) : current.editorFontSize,
    previewFontSize: typeof persisted.previewFontSize === "number" ? clampFontSize(persisted.previewFontSize) : current.previewFontSize,
    editorLineHeight: typeof persisted.editorLineHeight === "number" ? clampLineHeight(persisted.editorLineHeight) : current.editorLineHeight,
    previewLineHeight: typeof persisted.previewLineHeight === "number" ? clampLineHeight(persisted.previewLineHeight) : current.previewLineHeight,
    mode: validSetValue(workbenchModes, persisted.mode, current.mode),
    sidebar: validSetValue(sidebarPanels, persisted.sidebar, current.sidebar),
  };
}
