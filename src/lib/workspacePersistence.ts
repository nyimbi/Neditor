import type { AiCleanupOptions } from "../types.js";

export const WORKSPACE_SCHEMA_VERSION = 2;

export const SUPPORTED_CITATION_STYLES = [
  "title",
  "neditor-title",
  "author-year",
  "author_year",
  "apa",
  "american-psychological-association",
  "chicago-author-date",
  "chicago",
  "harvard",
  "council-of-science-editors-author-date",
  "key",
  "citation-key",
  "citation_key",
  "numeric",
  "ieee",
  "vancouver",
  "nature",
  "american-medical-association",
  "ama",
  "elsevier-vancouver",
] as const;

export type CitationStyle = (typeof SUPPORTED_CITATION_STYLES)[number];
export type LayoutPreset = "business" | "compact" | "presentation";
export type PreviewTheme = "match" | "light" | "dark";
export type SnapshotStorage = "app-data" | "project-local";
export type ExportTarget = "html" | "pdf" | "docx" | "pptx" | "markdown-bundle";
export type WorkbenchMode = "split" | "source" | "preview" | "focus" | "export" | "review" | "presentation";
export type SidebarPanel = "files" | "outline" | "diagnostics" | "tables" | "references" | "exports" | "versioning" | "review" | "settings";
export type ThemePreference = "system" | "light" | "dark";
export type TransformInputMode = "stdin" | "file";

export interface ExportDefaults {
  includeManifest: boolean;
  includeStyles: boolean;
  includeSyntaxHighlighting: boolean;
  coverPage: boolean;
  pageNumbers: boolean;
  layoutPreset: LayoutPreset;
  includeComments: boolean;
  includeProvenance: boolean;
  includeGlossary: boolean;
  includeAgenda: boolean;
}

export interface BibliographyDefaults {
  citationStyle: CitationStyle;
}

export interface BrandProfileDefaults {
  name: string;
  color: string;
  logo: string;
  font: string;
  header: string;
  footer: string;
  watermark: string;
  legalDisclaimer: string;
}

export interface GitIntegrationPreferences {
  enabled: boolean;
  warnOnDirtyExport: boolean;
}

export interface PersistedScrollPosition {
  editor?: number;
  preview?: number;
}

export interface PersistedWorkspace {
  schemaVersion?: number;
  theme?: ThemePreference;
  previewTheme?: PreviewTheme;
  editorPaneRatio?: number;
  wordWrap?: boolean;
  lineNumbers?: boolean;
  highContrast?: boolean;
  reducedMotion?: boolean;
  autosave?: boolean;
  autosaveDelayMs?: number;
  autoSnapshot?: boolean;
  snapshotIntervalMs?: number;
  snapshotStorage?: SnapshotStorage;
  editorFont?: string;
  previewFont?: string;
  editorFontSize?: number;
  previewFontSize?: number;
  editorLineHeight?: number;
  previewLineHeight?: number;
  exportTarget?: ExportTarget;
  exportDefaults?: Partial<ExportDefaults> & {
    includeCoverPage?: boolean;
    includePageNumbers?: boolean;
  };
  bibliographyDefaults?: Partial<BibliographyDefaults>;
  brandProfileDefaults?: Partial<BrandProfileDefaults>;
  gitIntegration?: Partial<GitIntegrationPreferences>;
  recentFiles?: string[];
  recentFolders?: string[];
  recentlyClosed?: string[];
  pinnedFiles?: string[];
  workspaceRoot?: string | null;
  openFiles?: string[];
  activePath?: string | null;
  scrollPositions?: Record<string, PersistedScrollPosition>;
  mode?: WorkbenchMode;
  sidebar?: SidebarPanel;
  transformEnginePaths?: Record<string, string>;
  trustedTransformEngines?: Record<string, boolean>;
  disabledTransformEngines?: Record<string, boolean>;
  transformInputModes?: Record<string, TransformInputMode>;
  transformTimeoutMs?: number;
  aiCleanupDefaults?: Partial<AiCleanupOptions>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function stringValue(value: unknown) {
  return typeof value === "string" ? value : undefined;
}

function booleanValue(value: unknown) {
  return typeof value === "boolean" ? value : undefined;
}

function numberValue(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function stringArray(value: unknown, limit: number) {
  if (!Array.isArray(value)) return undefined;
  const seen = new Set<string>();
  const result: string[] = [];
  for (const item of value) {
    if (typeof item !== "string" || !item.trim() || seen.has(item)) continue;
    seen.add(item);
    result.push(item);
    if (result.length >= limit) break;
  }
  return result;
}

function stringRecord(value: unknown) {
  if (!isRecord(value)) return undefined;
  return Object.fromEntries(
    Object.entries(value).filter((entry): entry is [string, string] => typeof entry[1] === "string"),
  );
}

function booleanRecord(value: unknown) {
  if (!isRecord(value)) return undefined;
  return Object.fromEntries(
    Object.entries(value).filter((entry): entry is [string, boolean] => typeof entry[1] === "boolean"),
  );
}

function inputModeRecord(value: unknown) {
  if (!isRecord(value)) return undefined;
  return Object.fromEntries(
    Object.entries(value).filter((entry): entry is [string, TransformInputMode] => entry[1] === "stdin" || entry[1] === "file"),
  );
}

function enumValue<T extends string>(value: unknown, allowed: readonly T[]) {
  return typeof value === "string" && allowed.includes(value as T) ? (value as T) : undefined;
}

export function clampLineHeight(value: number) {
  return Math.min(Math.max(Number(value) || 1.55, 1), 2.4);
}

export function clampFontSize(value: number) {
  return Math.min(Math.max(Number(value) || 14, 12), 22);
}

export function clampAutosaveDelay(value: number) {
  return Math.min(Math.max(Number(value) || 1500, 500), 30000);
}

export function clampSnapshotInterval(value: number) {
  return Math.min(Math.max(Number(value) || 300000, 30000), 3600000);
}

export function clampPaneRatio(value: number) {
  return Math.min(Math.max(Number(value) || 0.5, 0.25), 0.75);
}

export function clampScrollRatio(value: number | undefined) {
  if (typeof value !== "number" || !Number.isFinite(value)) return 0;
  return Math.min(Math.max(value, 0), 1);
}

export function normalizeCitationStyle(value: unknown): CitationStyle {
  if (typeof value !== "string") return "title";
  const normalized = value.trim().toLowerCase();
  return (SUPPORTED_CITATION_STYLES as readonly string[]).includes(normalized) ? (normalized as CitationStyle) : "title";
}

export function normalizeLayoutPreset(value: unknown): LayoutPreset {
  return value === "compact" || value === "presentation" || value === "business" ? value : "business";
}

export function normalizeExportDefaults(
  defaults: Partial<ExportDefaults> & {
    includeCoverPage?: boolean;
    includePageNumbers?: boolean;
  },
): ExportDefaults {
  return {
    includeManifest: typeof defaults.includeManifest === "boolean" ? defaults.includeManifest : true,
    includeStyles: typeof defaults.includeStyles === "boolean" ? defaults.includeStyles : true,
    includeSyntaxHighlighting:
      typeof defaults.includeSyntaxHighlighting === "boolean" ? defaults.includeSyntaxHighlighting : true,
    coverPage:
      typeof defaults.coverPage === "boolean"
        ? defaults.coverPage
        : typeof defaults.includeCoverPage === "boolean"
          ? defaults.includeCoverPage
          : true,
    pageNumbers:
      typeof defaults.pageNumbers === "boolean"
        ? defaults.pageNumbers
        : typeof defaults.includePageNumbers === "boolean"
          ? defaults.includePageNumbers
          : true,
    layoutPreset: normalizeLayoutPreset(defaults.layoutPreset),
    includeComments: typeof defaults.includeComments === "boolean" ? defaults.includeComments : true,
    includeProvenance: typeof defaults.includeProvenance === "boolean" ? defaults.includeProvenance : true,
    includeGlossary: typeof defaults.includeGlossary === "boolean" ? defaults.includeGlossary : true,
    includeAgenda: typeof defaults.includeAgenda === "boolean" ? defaults.includeAgenda : true,
  };
}

export function normalizeBibliographyDefaults(defaults: Partial<BibliographyDefaults>): BibliographyDefaults {
  return {
    citationStyle: normalizeCitationStyle(defaults.citationStyle),
  };
}

export function normalizeBrandProfileDefaults(defaults: Partial<BrandProfileDefaults>): BrandProfileDefaults {
  return {
    name: typeof defaults.name === "string" ? defaults.name : "",
    color: typeof defaults.color === "string" && defaults.color.trim() ? defaults.color.trim() : "#275DA8",
    logo: typeof defaults.logo === "string" ? defaults.logo : "",
    font: typeof defaults.font === "string" ? defaults.font : "",
    header: typeof defaults.header === "string" ? defaults.header : "",
    footer: typeof defaults.footer === "string" ? defaults.footer : "",
    watermark: typeof defaults.watermark === "string" ? defaults.watermark : "",
    legalDisclaimer: typeof defaults.legalDisclaimer === "string" ? defaults.legalDisclaimer : "",
  };
}

export function normalizeGitIntegrationPreferences(defaults: Partial<GitIntegrationPreferences>): GitIntegrationPreferences {
  return {
    enabled: typeof defaults.enabled === "boolean" ? defaults.enabled : true,
    warnOnDirtyExport: typeof defaults.warnOnDirtyExport === "boolean" ? defaults.warnOnDirtyExport : true,
  };
}

export function normalizeAiCleanupDefaults(defaults: Partial<AiCleanupOptions>): AiCleanupOptions {
  return {
    addProvenance: typeof defaults.addProvenance === "boolean" ? defaults.addProvenance : true,
    markAsDraft: typeof defaults.markAsDraft === "boolean" ? defaults.markAsDraft : true,
    insertCitationTodos: typeof defaults.insertCitationTodos === "boolean" ? defaults.insertCitationTodos : true,
    preserveHeadings: typeof defaults.preserveHeadings === "boolean" ? defaults.preserveHeadings : false,
    convertNumberedLists: typeof defaults.convertNumberedLists === "boolean" ? defaults.convertNumberedLists : true,
    convertTables: typeof defaults.convertTables === "boolean" ? defaults.convertTables : true,
  };
}

function normalizeScrollPositions(value: unknown) {
  if (!isRecord(value)) return undefined;
  const positions: Record<string, PersistedScrollPosition> = {};
  for (const [path, position] of Object.entries(value)) {
    if (!isRecord(position)) continue;
    positions[path] = {
      editor: clampScrollRatio(numberValue(position.editor)),
      preview: clampScrollRatio(numberValue(position.preview)),
    };
  }
  return positions;
}

function normalizeWorkspaceRecord(raw: Record<string, unknown>): PersistedWorkspace {
  const migrated: PersistedWorkspace = {
    schemaVersion: WORKSPACE_SCHEMA_VERSION,
  };
  const theme = enumValue(raw.theme, ["system", "light", "dark"] as const);
  if (theme) migrated.theme = theme;
  const previewTheme = enumValue(raw.previewTheme, ["match", "light", "dark"] as const);
  if (previewTheme) migrated.previewTheme = previewTheme;
  const editorPaneRatio = numberValue(raw.editorPaneRatio);
  if (editorPaneRatio !== undefined) migrated.editorPaneRatio = clampPaneRatio(editorPaneRatio);
  for (const key of ["wordWrap", "lineNumbers", "highContrast", "reducedMotion", "autosave", "autoSnapshot"] as const) {
    const value = booleanValue(raw[key]);
    if (value !== undefined) migrated[key] = value;
  }
  const autosaveDelayMs = numberValue(raw.autosaveDelayMs);
  if (autosaveDelayMs !== undefined) migrated.autosaveDelayMs = clampAutosaveDelay(autosaveDelayMs);
  const snapshotIntervalMs = numberValue(raw.snapshotIntervalMs);
  if (snapshotIntervalMs !== undefined) migrated.snapshotIntervalMs = clampSnapshotInterval(snapshotIntervalMs);
  const snapshotStorage = enumValue(raw.snapshotStorage, ["app-data", "project-local"] as const);
  if (snapshotStorage) migrated.snapshotStorage = snapshotStorage;
  for (const key of ["editorFont", "previewFont"] as const) {
    const value = stringValue(raw[key]);
    if (value !== undefined) migrated[key] = value;
  }
  const editorFontSize = numberValue(raw.editorFontSize);
  if (editorFontSize !== undefined) migrated.editorFontSize = clampFontSize(editorFontSize);
  const previewFontSize = numberValue(raw.previewFontSize);
  if (previewFontSize !== undefined) migrated.previewFontSize = clampFontSize(previewFontSize);
  const editorLineHeight = numberValue(raw.editorLineHeight);
  if (editorLineHeight !== undefined) migrated.editorLineHeight = clampLineHeight(editorLineHeight);
  const previewLineHeight = numberValue(raw.previewLineHeight);
  if (previewLineHeight !== undefined) migrated.previewLineHeight = clampLineHeight(previewLineHeight);
  const exportTarget = enumValue(raw.exportTarget, ["html", "pdf", "docx", "pptx", "markdown-bundle"] as const);
  if (exportTarget) migrated.exportTarget = exportTarget;
  if (isRecord(raw.exportDefaults)) migrated.exportDefaults = normalizeExportDefaults(raw.exportDefaults);
  if (isRecord(raw.bibliographyDefaults)) migrated.bibliographyDefaults = normalizeBibliographyDefaults(raw.bibliographyDefaults);
  if (isRecord(raw.brandProfileDefaults)) migrated.brandProfileDefaults = normalizeBrandProfileDefaults(raw.brandProfileDefaults);
  if (isRecord(raw.gitIntegration)) migrated.gitIntegration = normalizeGitIntegrationPreferences(raw.gitIntegration);
  if (isRecord(raw.aiCleanupDefaults)) migrated.aiCleanupDefaults = normalizeAiCleanupDefaults(raw.aiCleanupDefaults);
  migrated.recentFiles = stringArray(raw.recentFiles, 20);
  migrated.recentFolders = stringArray(raw.recentFolders, 12);
  migrated.recentlyClosed = stringArray(raw.recentlyClosed, 20);
  migrated.pinnedFiles = stringArray(raw.pinnedFiles, 50);
  migrated.openFiles = stringArray(raw.openFiles, 50);
  const workspaceRoot = stringValue(raw.workspaceRoot) ?? stringValue(raw.workspacePath);
  if (workspaceRoot !== undefined) migrated.workspaceRoot = workspaceRoot || null;
  const activePath = stringValue(raw.activePath) ?? stringValue(raw.activeFile);
  if (activePath !== undefined) migrated.activePath = activePath || null;
  const scrollPositions = normalizeScrollPositions(raw.scrollPositions);
  if (scrollPositions) migrated.scrollPositions = scrollPositions;
  const mode = enumValue(raw.mode, ["split", "source", "preview", "focus", "export", "review", "presentation"] as const);
  if (mode) migrated.mode = mode;
  const sidebar = enumValue(
    raw.sidebar,
    ["files", "outline", "diagnostics", "tables", "references", "exports", "versioning", "review", "settings"] as const,
  );
  if (sidebar) migrated.sidebar = sidebar;
  migrated.transformEnginePaths = stringRecord(raw.transformEnginePaths);
  migrated.trustedTransformEngines = booleanRecord(raw.trustedTransformEngines);
  migrated.disabledTransformEngines = booleanRecord(raw.disabledTransformEngines);
  migrated.transformInputModes = inputModeRecord(raw.transformInputModes);
  const transformTimeoutMs = numberValue(raw.transformTimeoutMs);
  if (transformTimeoutMs !== undefined) migrated.transformTimeoutMs = Math.min(Math.max(transformTimeoutMs, 1), 30000);
  return migrated;
}

export function migratePersistedWorkspace(value: unknown): PersistedWorkspace {
  return isRecord(value) ? normalizeWorkspaceRecord(value) : { schemaVersion: WORKSPACE_SCHEMA_VERSION };
}

export function normalizePersistedWorkspaceForSave(workspace: PersistedWorkspace): PersistedWorkspace {
  return migratePersistedWorkspace(workspace);
}
