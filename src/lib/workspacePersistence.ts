import type { AiCleanupOptions } from "../types.js";
import type { AiProviderProfileId } from "./aiProviderPackages.js";
import { normalizeBusinessProfile, type BusinessProfile } from "./businessDocuments.js";
import { normalizeGoogleIntegrationPreferences, type GoogleIntegrationPreferences } from "./googleAuth.js";
import type { PublishingContentFormat, PublishingDestinationProfile, PublishingTargetKind } from "./publishingWorkflow.js";
import { normalizeCustomTransformTemplates, type CustomTransformTemplate } from "./transformTemplates.js";

export const WORKSPACE_SCHEMA_VERSION = 2;
const TOOLBAR_COLLAPSE_ROW_IDS = ["file", "writing", "review-navigation", "view"];

export const SUPPORTED_CITATION_STYLES = [
  "title",
  "neditor-title",
  "author-year",
  "author_year",
  "apa",
  "american-psychological-association",
  "chicago-author-date",
  "chicago",
  "mla",
  "modern-language-association",
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
export type ExportTarget =
  | "html"
  | "pdf"
  | "docx"
  | "pptx"
  | "markdown-bundle"
  | "blog"
  | "substack"
  | "latex"
  | "google-docs"
  | "epub";
export type WorkbenchMode = "split" | "source" | "preview" | "focus" | "outline" | "export" | "review" | "presentation";
export type SidebarPanel =
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
  | "settings";
export type ThemePreference = "system" | "light" | "dark";
export type ToolbarDisplay = "both" | "icons" | "text";
export type EditorKeymapMode = "default" | "emacs" | "vim";
export type TransformInputMode = "stdin" | "file";

export interface ExportDefaults {
  includeManifest: boolean;
  includeStyles: boolean;
  includeSyntaxHighlighting: boolean;
  htmlLanguage: string;
  htmlDescription: string;
  canonicalUrl: string;
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

export interface ExportProfile {
  id: string;
  name: string;
  exportTarget: ExportTarget;
  exportDefaults: ExportDefaults;
  bibliographyDefaults: BibliographyDefaults;
  brandProfileDefaults: BrandProfileDefaults;
}

export interface GitIntegrationPreferences {
  enabled: boolean;
  warnOnDirtyExport: boolean;
}

export interface AiProviderDefaults {
  profileId: AiProviderProfileId;
  endpoint: string;
  model: string;
  keyEnv: string;
}

export type { GoogleIntegrationPreferences };

export type TtsEngineId = "browser-speech" | "macos-say" | "supertonic-cli";

export interface TtsPreferences {
  engine: TtsEngineId;
  voice: string;
  language: string;
  rate: number;
  supertonicCommand: string;
  supertonicVoice: string;
  supertonicLanguage: string;
  supertonicSpeed: number;
  supertonicModelDownloadAcknowledged: boolean;
  supertonicModelStoragePath: string;
}

const AI_PROVIDER_PROFILE_IDS = [
  "manual-review",
  "openai-compatible",
  "anthropic-compatible",
  "gemini-compatible",
  "local-http",
  "local-openai",
  "private-openai",
  "claude-code-cli",
  "codex-cli",
  "opencode-cli",
  "google-antigravity-cli",
] as const satisfies readonly AiProviderProfileId[];

const TTS_ENGINE_IDS = ["browser-speech", "macos-say", "supertonic-cli"] as const;

export type AgentLifecycleExecutionStatus = "queued" | "in-progress" | "needs-review" | "complete" | "blocked";

export interface AgentLifecycleTaskState {
  taskId: string;
  title: string;
  lane: string;
  status: AgentLifecycleExecutionStatus;
  note?: string;
  updatedAt: string;
  completedAt?: string;
}

export type AgentEditAcceptanceStatus = "queued" | "accepted" | "rejected" | "needs-revision";

export interface AgentEditAcceptanceState {
  itemId: string;
  heading: string;
  scope: "selection" | "section" | "document";
  status: AgentEditAcceptanceStatus;
  note?: string;
  updatedAt: string;
  appliedAt?: string;
}

export type AgentAutomationExecutionStatus = "queued" | "running" | "complete" | "blocked";

export interface AgentAutomationTaskState {
  taskId: string;
  label: string;
  status: AgentAutomationExecutionStatus;
  result?: string;
  updatedAt: string;
  completedAt?: string;
}

export type AgentRunHistoryControlStatus = "ready" | "needs-input" | "blocked";
export type AgentRunHistoryEvidenceStatus = "available" | "missing" | "needs-review";

export interface AgentRunHistoryControlItem {
  label: string;
  detail: string;
  status: AgentRunHistoryEvidenceStatus;
}

export interface AgentRunHistoryNextAction {
  label: string;
  detail: string;
  lane: string;
  action: string;
  status: AgentRunHistoryControlStatus;
}

export interface AgentRunHistoryControlCenter {
  status: AgentRunHistoryControlStatus;
  readinessScore: number;
  summary: string;
  nextActions: AgentRunHistoryNextAction[];
  sourceGrounding: AgentRunHistoryControlItem[];
  governance: AgentRunHistoryControlItem[];
  distribution: AgentRunHistoryControlItem[];
}

export interface AgentRunHistoryDocumentClaim {
  kind: "number" | "date" | "commitment" | "quote" | "claim";
  sourceLine: number;
  text: string;
  reason: string;
}

export interface AgentRunHistoryHumanizationFinding {
  kind: "generic-phrase" | "overconfident-claim" | "repetition" | "vague-transition";
  sourceLine: number;
  text: string;
  recommendation: string;
}

export interface AgentRunHistoryReviewCommentResolution {
  id: string;
  line: number;
  author: string;
  createdAt: string;
  excerpt: string;
  requiredAction: string;
  resolutionOptions: string[];
  blocker: boolean;
}

export interface AgentRunHistoryDocumentEvidence {
  unresolvedPlaceholders: string[];
  citationTodos: string[];
  claimInventory: AgentRunHistoryDocumentClaim[];
  humanizationFindings: AgentRunHistoryHumanizationFinding[];
  reviewCommentResolutions: AgentRunHistoryReviewCommentResolution[];
  unreviewedAiMarkers: number;
  unresolvedComments: number;
  approvalMetadataMissing: string[];
  brokenLinkHints: string[];
  referenceHints: string[];
}

export interface AgentRunHistoryOutlineCritiqueItem {
  severity: "info" | "warning" | "blocker";
  area: "coverage" | "sequence" | "duplication" | "depth" | "specificity";
  heading: string;
  detail: string;
  recommendation: string;
}

export interface AgentRunHistorySourcePack {
  contextSources: string[];
  userSources: string[];
  claimReview: string[];
  cleanupBlockers: string[];
  governanceBlockers: string[];
  distributionBlockers: string[];
  releaseEvidence: string[];
}

export interface AgentRunHistorySectionDraftItem {
  id: string;
  sectionId: string;
  sectionHeading: string;
  generatedAt: string;
  versionLabel: string;
  promptSummary: string;
  rationale: string;
  reviewerNotes: string[];
  sectionFingerprint: string;
  sourceFingerprint: string;
  restorePointMarkdown: string;
  acceptanceStatus: "drafted" | "needs-review" | "accepted";
}

export type AgentRunHistoryDocumentIntentStatus = "provided" | "needs-review" | "missing";

export interface AgentRunHistoryDocumentIntentField {
  key: string;
  label: string;
  value: string;
  status: AgentRunHistoryDocumentIntentStatus;
  source: string;
  guidance: string;
}

export interface AgentRunHistoryDocumentIntent {
  summary: string;
  completenessScore: number;
  status: "ready" | "needs-input";
  fields: AgentRunHistoryDocumentIntentField[];
  missingFields: string[];
  reviewPrompts: string[];
}

export interface AgentRunHistoryItem {
  runId: string;
  title: string;
  generatedAt: string;
  updatedAt: string;
  instruction: string;
  contextAnswers?: string;
  sourcePackText?: string;
  memoryText?: string;
  documentType: string;
  lanes: string[];
  distributionTargets: ExportTarget[];
  status: "generated" | "applied" | "provider-applied";
  applicationMode: "replace-document" | "replace-selection" | "append-packet";
  readinessScore: number;
  outputFingerprint: string;
  sourceFingerprint: string;
  contextFingerprint: string;
  instructionFingerprint: string;
  packetMarkdown?: string;
  packetPreview?: string;
  sectionCount?: number;
  sectionDraftVersionCount?: number;
  sectionDraftHistory?: AgentRunHistorySectionDraftItem[];
  transformRecommendationCount?: number;
  dataNarrativeLinkCount?: number;
  approvalGateStatus?: "ready" | "needs-review" | "blocked";
  automationTaskCount?: number;
  automationTaskStates?: AgentAutomationTaskState[];
  reviewerCount?: number;
  preReviewPromptCount?: number;
  taskCount?: number;
  lifecycleTaskStates?: AgentLifecycleTaskState[];
  editAcceptanceStates?: AgentEditAcceptanceState[];
  controlCenter?: AgentRunHistoryControlCenter;
  documentEvidence?: AgentRunHistoryDocumentEvidence;
  outlineCritique?: AgentRunHistoryOutlineCritiqueItem[];
  sourcePack?: AgentRunHistorySourcePack;
  documentIntent?: AgentRunHistoryDocumentIntent;
  appliedAt?: string;
  providerProfile?: string;
}

export interface DocsLiveDraftHistoryItem {
  draftId: string;
  title: string;
  generatedAt: string;
  updatedAt: string;
  documentType: string;
  sectionCount: number;
  issueCount: number;
  outlineText?: string;
  instruction?: string;
  markdown: string;
  markdownPreview?: string;
  reviewPacketMarkdown?: string;
  reviewPacketPreview?: string;
  outputFingerprint: string;
}

export interface PersistedScrollPosition {
  editor?: number;
  preview?: number;
}

export interface PersistedWorkspace {
  schemaVersion?: number;
  theme?: ThemePreference;
  previewTheme?: PreviewTheme;
  toolbarDisplay?: ToolbarDisplay;
  toolbarTextSize?: number;
  toolbarCollapsedRows?: string[];
  editorPaneRatio?: number;
  splitSourcePanes?: boolean;
  editorKeymapMode?: EditorKeymapMode;
  wordWrap?: boolean;
  lineNumbers?: boolean;
  codeFolding?: boolean;
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
  businessProfile?: Partial<BusinessProfile>;
  aiProviderDefaults?: Partial<AiProviderDefaults>;
  googleIntegration?: Partial<GoogleIntegrationPreferences>;
  ttsPreferences?: Partial<TtsPreferences>;
  exportProfiles?: Partial<ExportProfile>[];
  activeExportProfileId?: string;
  publishingDestinationProfiles?: Partial<PublishingDestinationProfile>[];
  activePublishingDestinationId?: string;
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
  customTransformTemplates?: CustomTransformTemplate[];
  aiCleanupDefaults?: Partial<AiCleanupOptions>;
  agentRunHistory?: Partial<AgentRunHistoryItem>[];
  docsLiveDraftHistory?: Partial<DocsLiveDraftHistoryItem>[];
  guidedDemoCompletedStepIds?: string[];
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

function clampNumber(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
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

function normalizedString(value: unknown, limit: number) {
  return typeof value === "string" ? value.trim().slice(0, limit) : "";
}

export function clampLineHeight(value: number) {
  return Math.min(Math.max(Number(value) || 1.55, 1), 2.4);
}

export function clampFontSize(value: number) {
  return Math.min(Math.max(Number(value) || 14, 12), 22);
}

export function clampToolbarTextSize(value: number) {
  return Math.min(Math.max(Number(value) || 10, 9), 15);
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
    htmlLanguage: normalizedString(defaults.htmlLanguage, 35),
    htmlDescription: normalizedString(defaults.htmlDescription, 280),
    canonicalUrl: normalizedString(defaults.canonicalUrl, 2048),
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

function normalizeExportProfile(profile: unknown, index: number): ExportProfile | null {
  if (!isRecord(profile)) return null;
  const id = stringValue(profile.id)?.trim() || `export-profile-${index + 1}`;
  const name = stringValue(profile.name)?.trim() || `Export profile ${index + 1}`;
  const exportTarget =
    enumValue(profile.exportTarget, ["html", "pdf", "docx", "pptx", "markdown-bundle", "blog", "substack", "latex", "google-docs", "epub"] as const) ||
    "html";
  return {
    id,
    name,
    exportTarget,
    exportDefaults: isRecord(profile.exportDefaults) ? normalizeExportDefaults(profile.exportDefaults) : normalizeExportDefaults({}),
    bibliographyDefaults: isRecord(profile.bibliographyDefaults)
      ? normalizeBibliographyDefaults(profile.bibliographyDefaults)
      : normalizeBibliographyDefaults({}),
    brandProfileDefaults: isRecord(profile.brandProfileDefaults)
      ? normalizeBrandProfileDefaults(profile.brandProfileDefaults)
      : normalizeBrandProfileDefaults({}),
  };
}

export function normalizeExportProfiles(value: unknown): ExportProfile[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const profiles: ExportProfile[] = [];
  for (const item of value) {
    const profile = normalizeExportProfile(item, profiles.length);
    if (!profile || seen.has(profile.id)) continue;
    seen.add(profile.id);
    profiles.push(profile);
    if (profiles.length >= 20) break;
  }
  return profiles;
}

function normalizePublishingDestinationProfile(profile: unknown, index: number): PublishingDestinationProfile | null {
  if (!isRecord(profile)) return null;
  const id = stringValue(profile.id)?.trim() || `publishing-destination-${index + 1}`;
  const name = stringValue(profile.name)?.trim() || `Publishing destination ${index + 1}`;
  const targetKind =
    enumValue(profile.targetKind, ["generic-webhook", "wordpress-rest", "ghost-admin", "substack-manual"] as const) ||
    ("generic-webhook" satisfies PublishingTargetKind);
  const contentFormat = enumValue(profile.contentFormat, ["html", "markdown", "text"] as const) || ("html" satisfies PublishingContentFormat);
  return {
    id,
    name,
    targetKind,
    endpointUrl: stringValue(profile.endpointUrl)?.trim() || "",
    contentFormat,
    authHeaderName: stringValue(profile.authHeaderName)?.trim() || "Authorization",
  };
}

export function normalizePublishingDestinationProfiles(value: unknown): PublishingDestinationProfile[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const profiles: PublishingDestinationProfile[] = [];
  for (const item of value) {
    const profile = normalizePublishingDestinationProfile(item, profiles.length);
    if (!profile || seen.has(profile.id)) continue;
    seen.add(profile.id);
    profiles.push(profile);
    if (profiles.length >= 20) break;
  }
  return profiles;
}

export function normalizeGitIntegrationPreferences(defaults: Partial<GitIntegrationPreferences>): GitIntegrationPreferences {
  return {
    enabled: typeof defaults.enabled === "boolean" ? defaults.enabled : true,
    warnOnDirtyExport: typeof defaults.warnOnDirtyExport === "boolean" ? defaults.warnOnDirtyExport : true,
  };
}

export function normalizeAiProviderDefaults(defaults: unknown): AiProviderDefaults {
  const record = isRecord(defaults) ? defaults : {};
  const profileId = enumValue(record.profileId, AI_PROVIDER_PROFILE_IDS) || "manual-review";
  return {
    profileId,
    endpoint: normalizedString(record.endpoint, 300),
    model: normalizedString(record.model, 160) || "human-approved-provider",
    keyEnv: normalizeEnvName(record.keyEnv) || "NEDITOR_AI_API_KEY",
  };
}

export function normalizeTtsPreferences(defaults: unknown): TtsPreferences {
  const record = isRecord(defaults) ? defaults : {};
  const engine = enumValue(record.engine, TTS_ENGINE_IDS) || "browser-speech";
  const rate = numberValue(record.rate);
  const supertonicSpeed = numberValue(record.supertonicSpeed);
  return {
    engine,
    voice: normalizedString(record.voice, 120),
    language: normalizedString(record.language, 24) || "en-US",
    rate: rate === undefined ? 1 : clampNumber(rate, 0.5, 2),
    supertonicCommand: normalizedString(record.supertonicCommand, 300) || "supertonic",
    supertonicVoice: normalizedString(record.supertonicVoice, 80) || "M1",
    supertonicLanguage: normalizedString(record.supertonicLanguage, 24) || "en",
    supertonicSpeed: supertonicSpeed === undefined ? 1 : clampNumber(supertonicSpeed, 0.7, 2),
    supertonicModelDownloadAcknowledged: record.supertonicModelDownloadAcknowledged === true,
    supertonicModelStoragePath: normalizedString(record.supertonicModelStoragePath, 400),
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

function normalizeEnvName(value: unknown) {
  const name = normalizedString(value, 80).replace(/[^A-Z0-9_]/gi, "_").toUpperCase();
  return /^[A-Z_][A-Z0-9_]*$/.test(name) ? name : "";
}

function normalizeAgentLifecycleTaskStates(value: unknown): AgentLifecycleTaskState[] {
  if (!Array.isArray(value)) return [];
  const states: AgentLifecycleTaskState[] = [];
  const seen = new Set<string>();
  for (const item of value) {
    if (!isRecord(item)) continue;
    const taskId = normalizedString(item.taskId, 120);
    if (!taskId || seen.has(taskId)) continue;
    const status = enumValue(item.status, ["queued", "in-progress", "needs-review", "complete", "blocked"] as const) || "queued";
    seen.add(taskId);
    states.push({
      taskId,
      title: normalizedString(item.title, 180) || "Agent lifecycle task",
      lane: normalizedString(item.lane, 40) || "review",
      status,
      note: normalizedString(item.note, 1_200) || undefined,
      updatedAt: normalizedString(item.updatedAt, 40) || new Date(0).toISOString(),
      completedAt: normalizedString(item.completedAt, 40) || undefined,
    });
    if (states.length >= 60) break;
  }
  return states;
}

function normalizeAgentEditAcceptanceStates(value: unknown): AgentEditAcceptanceState[] {
  if (!Array.isArray(value)) return [];
  const states: AgentEditAcceptanceState[] = [];
  const seen = new Set<string>();
  for (const item of value) {
    if (!isRecord(item)) continue;
    const itemId = normalizedString(item.itemId, 140);
    if (!itemId || seen.has(itemId)) continue;
    const scope = enumValue(item.scope, ["selection", "section", "document"] as const) || "section";
    const status = enumValue(item.status, ["queued", "accepted", "rejected", "needs-revision"] as const) || "queued";
    seen.add(itemId);
    states.push({
      itemId,
      heading: normalizedString(item.heading, 180) || "Agent edit",
      scope,
      status,
      note: normalizedString(item.note, 1_200) || undefined,
      updatedAt: normalizedString(item.updatedAt, 40) || new Date(0).toISOString(),
      appliedAt: normalizedString(item.appliedAt, 40) || undefined,
    });
    if (states.length >= 60) break;
  }
  return states;
}

function normalizeAgentAutomationTaskStates(value: unknown): AgentAutomationTaskState[] {
  if (!Array.isArray(value)) return [];
  const states: AgentAutomationTaskState[] = [];
  const seen = new Set<string>();
  for (const item of value) {
    if (!isRecord(item)) continue;
    const taskId = normalizedString(item.taskId, 140);
    if (!taskId || seen.has(taskId)) continue;
    seen.add(taskId);
    const status = enumValue(item.status, ["queued", "running", "complete", "blocked"] as const) || "queued";
    const result = normalizedString(item.result, 1_500);
    const completedAt = normalizedString(item.completedAt, 40);
    states.push({
      taskId,
      label: normalizedString(item.label, 180) || "Agent automation check",
      status,
      updatedAt: normalizedString(item.updatedAt, 40) || new Date(0).toISOString(),
      ...(result ? { result } : {}),
      ...(completedAt ? { completedAt } : {}),
    });
    if (states.length >= 24) break;
  }
  return states;
}

function normalizeAgentRunHistoryControlItems(value: unknown): AgentRunHistoryControlItem[] {
  if (!Array.isArray(value)) return [];
  const items: AgentRunHistoryControlItem[] = [];
  for (const item of value) {
    if (!isRecord(item)) continue;
    const label = normalizedString(item.label, 80);
    const detail = normalizedString(item.detail, 500);
    if (!label || !detail) continue;
    items.push({
      label,
      detail,
      status: enumValue(item.status, ["available", "missing", "needs-review"] as const) || "needs-review",
    });
    if (items.length >= 20) break;
  }
  return items;
}

function normalizeAgentRunHistoryNextActions(value: unknown): AgentRunHistoryNextAction[] {
  if (!Array.isArray(value)) return [];
  const actions: AgentRunHistoryNextAction[] = [];
  for (const item of value) {
    if (!isRecord(item)) continue;
    const label = normalizedString(item.label, 100);
    const detail = normalizedString(item.detail, 500);
    if (!label || !detail) continue;
    actions.push({
      label,
      detail,
      lane:
        enumValue(item.lane, ["create", "compose", "edit", "revise", "review", "distribute"] as const) ||
        normalizedString(item.lane, 40) ||
        "review",
      action:
        enumValue(
          item.action,
          ["open-docs-live", "generate-docs-live-draft", "open-outline", "open-ai-paste", "open-review", "prepare-export", "open-exports"] as const,
        ) ||
        normalizedString(item.action, 80) ||
        "open-review",
      status: enumValue(item.status, ["ready", "needs-input", "blocked"] as const) || "needs-input",
    });
    if (actions.length >= 20) break;
  }
  return actions;
}

function normalizeAgentRunHistoryControlCenter(value: unknown): AgentRunHistoryControlCenter | undefined {
  if (!isRecord(value)) return undefined;
  return {
    status: enumValue(value.status, ["ready", "needs-input", "blocked"] as const) || "needs-input",
    readinessScore: Math.min(Math.max(numberValue(value.readinessScore) ?? 0, 0), 100),
    summary: normalizedString(value.summary, 500) || "AI control center snapshot",
    nextActions: normalizeAgentRunHistoryNextActions(value.nextActions),
    sourceGrounding: normalizeAgentRunHistoryControlItems(value.sourceGrounding),
    governance: normalizeAgentRunHistoryControlItems(value.governance),
    distribution: normalizeAgentRunHistoryControlItems(value.distribution),
  };
}

function normalizeAgentRunHistoryDocumentClaims(value: unknown): AgentRunHistoryDocumentClaim[] {
  if (!Array.isArray(value)) return [];
  const claims: AgentRunHistoryDocumentClaim[] = [];
  for (const item of value) {
    if (!isRecord(item)) continue;
    const text = normalizedString(item.text, 500);
    if (!text) continue;
    claims.push({
      kind: enumValue(item.kind, ["number", "date", "commitment", "quote", "claim"] as const) || "claim",
      sourceLine: Math.max(Math.floor(numberValue(item.sourceLine) ?? 0), 0),
      text,
      reason: normalizedString(item.reason, 240) || "Needs source review",
    });
    if (claims.length >= 80) break;
  }
  return claims;
}

function normalizeAgentRunHistoryHumanizationFindings(value: unknown): AgentRunHistoryHumanizationFinding[] {
  if (!Array.isArray(value)) return [];
  const findings: AgentRunHistoryHumanizationFinding[] = [];
  for (const item of value) {
    if (!isRecord(item)) continue;
    const text = normalizedString(item.text, 500);
    if (!text) continue;
    findings.push({
      kind: enumValue(item.kind, ["generic-phrase", "overconfident-claim", "repetition", "vague-transition"] as const) || "generic-phrase",
      sourceLine: Math.max(Math.floor(numberValue(item.sourceLine) ?? 0), 0),
      text,
      recommendation: normalizedString(item.recommendation, 320) || "Rewrite in concrete, reader-centered language.",
    });
    if (findings.length >= 80) break;
  }
  return findings;
}

function normalizeAgentRunHistoryReviewCommentResolutions(value: unknown): AgentRunHistoryReviewCommentResolution[] {
  if (!Array.isArray(value)) return [];
  const comments: AgentRunHistoryReviewCommentResolution[] = [];
  for (const item of value) {
    if (!isRecord(item)) continue;
    const excerpt = normalizedString(item.excerpt, 500);
    if (!excerpt) continue;
    const line = Math.max(Math.floor(numberValue(item.line) ?? 0), 0);
    comments.push({
      id: normalizedString(item.id, 120) || `review-comment-${line}-${comments.length + 1}`,
      line,
      author: normalizedString(item.author, 120) || "Reviewer",
      createdAt: normalizedString(item.createdAt, 120) || "",
      excerpt,
      requiredAction: normalizedString(item.requiredAction, 500) || "Answer, resolve, or carry forward this review comment.",
      resolutionOptions: normalizedStringArray(item.resolutionOptions, 10, 300),
      blocker: booleanValue(item.blocker) || false,
    });
    if (comments.length >= 80) break;
  }
  return comments;
}

function normalizedStringArray(value: unknown, limit: number, itemLimit: number) {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const result: string[] = [];
  for (const item of value) {
    const normalized = normalizedString(item, itemLimit);
    if (!normalized || seen.has(normalized)) continue;
    seen.add(normalized);
    result.push(normalized);
    if (result.length >= limit) break;
  }
  return result;
}

function normalizeAgentRunHistoryDocumentEvidence(value: unknown): AgentRunHistoryDocumentEvidence | undefined {
  if (!isRecord(value)) return undefined;
  const reviewCommentResolutions = normalizeAgentRunHistoryReviewCommentResolutions(value.reviewCommentResolutions);
  return {
    unresolvedPlaceholders: stringArray(value.unresolvedPlaceholders, 80) || [],
    citationTodos: stringArray(value.citationTodos, 80) || [],
    claimInventory: normalizeAgentRunHistoryDocumentClaims(value.claimInventory),
    humanizationFindings: normalizeAgentRunHistoryHumanizationFindings(value.humanizationFindings),
    reviewCommentResolutions,
    unreviewedAiMarkers: Math.max(Math.floor(numberValue(value.unreviewedAiMarkers) ?? 0), 0),
    unresolvedComments: Math.max(Math.floor(numberValue(value.unresolvedComments) ?? reviewCommentResolutions.length), 0),
    approvalMetadataMissing: stringArray(value.approvalMetadataMissing, 20) || [],
    brokenLinkHints: stringArray(value.brokenLinkHints, 80) || [],
    referenceHints: stringArray(value.referenceHints, 80) || [],
  };
}

function normalizeAgentRunHistoryOutlineCritique(value: unknown): AgentRunHistoryOutlineCritiqueItem[] {
  if (!Array.isArray(value)) return [];
  const items: AgentRunHistoryOutlineCritiqueItem[] = [];
  for (const item of value) {
    if (!isRecord(item)) continue;
    const detail = normalizedString(item.detail, 500);
    if (!detail) continue;
    items.push({
      severity: enumValue(item.severity, ["info", "warning", "blocker"] as const) || "warning",
      area: enumValue(item.area, ["coverage", "sequence", "duplication", "depth", "specificity"] as const) || "coverage",
      heading: normalizedString(item.heading, 140) || "Document outline",
      detail,
      recommendation: normalizedString(item.recommendation, 500) || "Review this outline issue before drafting.",
    });
    if (items.length >= 60) break;
  }
  return items;
}

function normalizeAgentRunHistorySourcePack(value: unknown): AgentRunHistorySourcePack | undefined {
  if (!isRecord(value)) return undefined;
  return {
    contextSources: stringArray(value.contextSources, 80) || [],
    userSources: stringArray(value.userSources, 120) || [],
    claimReview: stringArray(value.claimReview, 120) || [],
    cleanupBlockers: stringArray(value.cleanupBlockers, 120) || [],
    governanceBlockers: stringArray(value.governanceBlockers, 80) || [],
    distributionBlockers: stringArray(value.distributionBlockers, 80) || [],
    releaseEvidence: stringArray(value.releaseEvidence, 80) || [],
  };
}

function normalizeAgentRunHistorySectionDrafts(value: unknown): AgentRunHistorySectionDraftItem[] {
  if (!Array.isArray(value)) return [];
  const items: AgentRunHistorySectionDraftItem[] = [];
  const seen = new Set<string>();
  for (const item of value) {
    if (!isRecord(item)) continue;
    const id = normalizedString(item.id, 140);
    const sectionId = normalizedString(item.sectionId, 140);
    const restorePointMarkdown = normalizedString(item.restorePointMarkdown, 8_000);
    if (!id || !sectionId || !restorePointMarkdown || seen.has(id)) continue;
    seen.add(id);
    items.push({
      id,
      sectionId,
      sectionHeading: normalizedString(item.sectionHeading, 180) || "Section",
      generatedAt: normalizedString(item.generatedAt, 40) || new Date(0).toISOString(),
      versionLabel: normalizedString(item.versionLabel, 80) || "section draft",
      promptSummary: normalizedString(item.promptSummary, 1_000) || "Draft this section from the saved agent context.",
      rationale: normalizedString(item.rationale, 1_000) || "Review this saved draft before reusing it.",
      reviewerNotes: normalizedStringArray(item.reviewerNotes, 12, 600),
      sectionFingerprint: normalizedString(item.sectionFingerprint, 32) || id.slice(0, 32),
      sourceFingerprint: normalizedString(item.sourceFingerprint, 32) || "",
      restorePointMarkdown,
      acceptanceStatus: enumValue(item.acceptanceStatus, ["drafted", "needs-review", "accepted"] as const) || "needs-review",
    });
    if (items.length >= 60) break;
  }
  return items;
}

function normalizeAgentRunHistoryDocumentIntent(value: unknown): AgentRunHistoryDocumentIntent | undefined {
  if (!isRecord(value)) return undefined;
  const fields: AgentRunHistoryDocumentIntentField[] = [];
  if (Array.isArray(value.fields)) {
    for (const item of value.fields) {
      if (!isRecord(item)) continue;
      const key = normalizedString(item.key, 80);
      const label = normalizedString(item.label, 120);
      if (!key || !label) continue;
      fields.push({
        key,
        label,
        value: normalizedString(item.value, 300) || "TBD",
        status: enumValue(item.status, ["provided", "needs-review", "missing"] as const) || "needs-review",
        source: normalizedString(item.source, 80) || "unknown",
        guidance: normalizedString(item.guidance, 500) || "Review this intent field before relying on generated content.",
      });
      if (fields.length >= 40) break;
    }
  }
  const summary = normalizedString(value.summary, 500);
  if (!summary && !fields.length) return undefined;
  return {
    summary: summary || "Document intent sheet snapshot",
    completenessScore: Math.min(Math.max(numberValue(value.completenessScore) ?? 0, 0), 100),
    status: enumValue(value.status, ["ready", "needs-input"] as const) || "needs-input",
    fields,
    missingFields: normalizedStringArray(value.missingFields, 30, 160),
    reviewPrompts: normalizedStringArray(value.reviewPrompts, 40, 500),
  };
}

function normalizeAgentRunHistoryItem(value: unknown): AgentRunHistoryItem | null {
  if (!isRecord(value)) return null;
  const runId = normalizedString(value.runId, 80);
  if (!runId) return null;
  const status = enumValue(value.status, ["generated", "applied", "provider-applied"] as const) || "generated";
  const applicationMode =
    enumValue(value.applicationMode, ["replace-document", "replace-selection", "append-packet"] as const) || "append-packet";
  const distributionTargets = Array.isArray(value.distributionTargets)
    ? value.distributionTargets
        .filter((target): target is ExportTarget =>
          ["html", "pdf", "docx", "pptx", "markdown-bundle", "blog", "substack", "latex", "google-docs", "epub"].includes(String(target)),
        )
        .slice(0, 12)
    : [];
  const lifecycleTaskStates = normalizeAgentLifecycleTaskStates(value.lifecycleTaskStates);
  const editAcceptanceStates = normalizeAgentEditAcceptanceStates(value.editAcceptanceStates);
  const automationTaskStates = normalizeAgentAutomationTaskStates(value.automationTaskStates);
  const controlCenter = normalizeAgentRunHistoryControlCenter(value.controlCenter);
  const documentEvidence = normalizeAgentRunHistoryDocumentEvidence(value.documentEvidence);
  const outlineCritique = normalizeAgentRunHistoryOutlineCritique(value.outlineCritique);
  const sourcePack = normalizeAgentRunHistorySourcePack(value.sourcePack);
  const sectionDraftHistory = normalizeAgentRunHistorySectionDrafts(value.sectionDraftHistory);
  const documentIntent = normalizeAgentRunHistoryDocumentIntent(value.documentIntent);
  return {
    runId,
    title: normalizedString(value.title, 120) || "Agent run",
    generatedAt: normalizedString(value.generatedAt, 40),
    updatedAt: normalizedString(value.updatedAt, 40),
    instruction: normalizedString(value.instruction, 500),
    contextAnswers: normalizedString(value.contextAnswers, 4_000) || undefined,
    sourcePackText: normalizedString(value.sourcePackText, 8_000) || undefined,
    memoryText: normalizedString(value.memoryText, 8_000) || undefined,
    documentType: normalizedString(value.documentType, 80),
    lanes: stringArray(value.lanes, 12) || [],
    distributionTargets,
    status,
    applicationMode,
    readinessScore: Math.min(Math.max(numberValue(value.readinessScore) ?? 0, 0), 100),
    outputFingerprint: normalizedString(value.outputFingerprint, 32),
    sourceFingerprint: normalizedString(value.sourceFingerprint, 32),
    contextFingerprint: normalizedString(value.contextFingerprint, 32),
    instructionFingerprint: normalizedString(value.instructionFingerprint, 32),
    packetMarkdown: normalizedString(value.packetMarkdown, 24_000) || undefined,
    packetPreview: normalizedString(value.packetPreview, 1_200) || undefined,
    sectionCount: Math.max(numberValue(value.sectionCount) ?? 0, 0),
    sectionDraftVersionCount: Math.max(numberValue(value.sectionDraftVersionCount) ?? sectionDraftHistory.length, 0),
    ...(sectionDraftHistory.length ? { sectionDraftHistory } : {}),
    transformRecommendationCount: Math.max(numberValue(value.transformRecommendationCount) ?? 0, 0),
    dataNarrativeLinkCount: Math.max(numberValue(value.dataNarrativeLinkCount) ?? 0, 0),
    approvalGateStatus: enumValue(value.approvalGateStatus, ["ready", "needs-review", "blocked"] as const) || undefined,
    automationTaskCount: Math.max(numberValue(value.automationTaskCount) ?? automationTaskStates.length, 0),
    ...(automationTaskStates.length ? { automationTaskStates } : {}),
    reviewerCount: Math.max(numberValue(value.reviewerCount) ?? 0, 0),
    preReviewPromptCount: Math.max(numberValue(value.preReviewPromptCount) ?? 0, 0),
    taskCount: Math.max(numberValue(value.taskCount) ?? 0, 0),
    ...(lifecycleTaskStates.length ? { lifecycleTaskStates } : {}),
    ...(editAcceptanceStates.length ? { editAcceptanceStates } : {}),
    ...(controlCenter ? { controlCenter } : {}),
    ...(documentEvidence ? { documentEvidence } : {}),
    ...(outlineCritique.length ? { outlineCritique } : {}),
    ...(sourcePack ? { sourcePack } : {}),
    ...(documentIntent ? { documentIntent } : {}),
    appliedAt: normalizedString(value.appliedAt, 40) || undefined,
    providerProfile: normalizedString(value.providerProfile, 120) || undefined,
  };
}

export function normalizeAgentRunHistory(value: unknown): AgentRunHistoryItem[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const history: AgentRunHistoryItem[] = [];
  for (const entry of value) {
    const item = normalizeAgentRunHistoryItem(entry);
    if (!item || seen.has(item.runId)) continue;
    seen.add(item.runId);
    history.push(item);
    if (history.length >= 50) break;
  }
  return history;
}

function normalizeDocsLiveDraftHistoryItem(value: unknown): DocsLiveDraftHistoryItem | null {
  if (!isRecord(value)) return null;
  const draftId = normalizedString(value.draftId, 100);
  const markdown = normalizedString(value.markdown, 48_000);
  if (!draftId || !markdown) return null;
  return {
    draftId,
    title: normalizedString(value.title, 160) || "Docs Live draft",
    generatedAt: normalizedString(value.generatedAt, 40) || new Date(0).toISOString(),
    updatedAt: normalizedString(value.updatedAt, 40) || normalizedString(value.generatedAt, 40) || new Date(0).toISOString(),
    documentType: normalizedString(value.documentType, 80) || "business-brief",
    sectionCount: Math.max(Math.floor(numberValue(value.sectionCount) ?? 0), 0),
    issueCount: Math.max(Math.floor(numberValue(value.issueCount) ?? 0), 0),
    outlineText: normalizedString(value.outlineText, 8_000) || undefined,
    instruction: normalizedString(value.instruction, 4_000) || undefined,
    markdown,
    markdownPreview: normalizedString(value.markdownPreview, 1_200) || undefined,
    reviewPacketMarkdown: normalizedString(value.reviewPacketMarkdown, 18_000) || undefined,
    reviewPacketPreview: normalizedString(value.reviewPacketPreview, 1_200) || undefined,
    outputFingerprint: normalizedString(value.outputFingerprint, 32) || draftId,
  };
}

export function normalizeDocsLiveDraftHistory(value: unknown): DocsLiveDraftHistoryItem[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const history: DocsLiveDraftHistoryItem[] = [];
  for (const entry of value) {
    const item = normalizeDocsLiveDraftHistoryItem(entry);
    if (!item || seen.has(item.draftId)) continue;
    seen.add(item.draftId);
    history.push(item);
    if (history.length >= 30) break;
  }
  return history;
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
  const toolbarDisplay = enumValue(raw.toolbarDisplay, ["both", "icons", "text"] as const);
  if (toolbarDisplay) migrated.toolbarDisplay = toolbarDisplay;
  const toolbarTextSize = numberValue(raw.toolbarTextSize);
  if (toolbarTextSize !== undefined) migrated.toolbarTextSize = clampToolbarTextSize(toolbarTextSize);
  if (Array.isArray(raw.toolbarCollapsedRows)) {
    migrated.toolbarCollapsedRows = Array.from(
      new Set(raw.toolbarCollapsedRows.filter((item): item is string => typeof item === "string" && TOOLBAR_COLLAPSE_ROW_IDS.includes(item))),
    );
  }
  const editorPaneRatio = numberValue(raw.editorPaneRatio);
  if (editorPaneRatio !== undefined) migrated.editorPaneRatio = clampPaneRatio(editorPaneRatio);
  const splitSourcePanes = booleanValue(raw.splitSourcePanes);
  if (splitSourcePanes !== undefined) migrated.splitSourcePanes = splitSourcePanes;
  const editorKeymapMode = enumValue(raw.editorKeymapMode, ["default", "emacs", "vim"] as const);
  if (editorKeymapMode) migrated.editorKeymapMode = editorKeymapMode;
  for (const key of ["wordWrap", "lineNumbers", "codeFolding", "highContrast", "reducedMotion", "autosave", "autoSnapshot"] as const) {
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
  const exportTarget = enumValue(raw.exportTarget, [
    "html",
    "pdf",
    "docx",
    "pptx",
    "markdown-bundle",
    "blog",
    "substack",
    "latex",
    "google-docs",
    "epub",
  ] as const);
  if (exportTarget) migrated.exportTarget = exportTarget;
  if (isRecord(raw.exportDefaults)) migrated.exportDefaults = normalizeExportDefaults(raw.exportDefaults);
  if (isRecord(raw.bibliographyDefaults)) migrated.bibliographyDefaults = normalizeBibliographyDefaults(raw.bibliographyDefaults);
  if (isRecord(raw.brandProfileDefaults)) migrated.brandProfileDefaults = normalizeBrandProfileDefaults(raw.brandProfileDefaults);
  migrated.businessProfile = normalizeBusinessProfile(raw.businessProfile);
  migrated.aiProviderDefaults = normalizeAiProviderDefaults(raw.aiProviderDefaults);
  migrated.googleIntegration = normalizeGoogleIntegrationPreferences(raw.googleIntegration);
  migrated.ttsPreferences = normalizeTtsPreferences(raw.ttsPreferences);
  const exportProfiles = normalizeExportProfiles(raw.exportProfiles);
  if (exportProfiles.length) {
    migrated.exportProfiles = exportProfiles;
    const activeExportProfileId = stringValue(raw.activeExportProfileId);
    if (activeExportProfileId && exportProfiles.some((profile) => profile.id === activeExportProfileId)) {
      migrated.activeExportProfileId = activeExportProfileId;
    }
  }
  const publishingDestinationProfiles = normalizePublishingDestinationProfiles(raw.publishingDestinationProfiles);
  if (publishingDestinationProfiles.length) {
    migrated.publishingDestinationProfiles = publishingDestinationProfiles;
    const activePublishingDestinationId = stringValue(raw.activePublishingDestinationId);
    if (activePublishingDestinationId && publishingDestinationProfiles.some((profile) => profile.id === activePublishingDestinationId)) {
      migrated.activePublishingDestinationId = activePublishingDestinationId;
    }
  }
  if (isRecord(raw.gitIntegration)) migrated.gitIntegration = normalizeGitIntegrationPreferences(raw.gitIntegration);
  if (isRecord(raw.aiCleanupDefaults)) migrated.aiCleanupDefaults = normalizeAiCleanupDefaults(raw.aiCleanupDefaults);
  const agentRunHistory = normalizeAgentRunHistory(raw.agentRunHistory);
  if (agentRunHistory.length) migrated.agentRunHistory = agentRunHistory;
  const docsLiveDraftHistory = normalizeDocsLiveDraftHistory(raw.docsLiveDraftHistory);
  if (docsLiveDraftHistory.length) migrated.docsLiveDraftHistory = docsLiveDraftHistory;
  migrated.guidedDemoCompletedStepIds = stringArray(raw.guidedDemoCompletedStepIds, 40);
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
  const mode = enumValue(raw.mode, ["split", "source", "preview", "focus", "outline", "export", "review", "presentation"] as const);
  if (mode) migrated.mode = mode;
  const sidebar = enumValue(
    raw.sidebar,
    ["files", "outline", "diagnostics", "tables", "templates", "references", "exports", "versioning", "review", "help", "settings"] as const,
  );
  if (sidebar) migrated.sidebar = sidebar;
  migrated.transformEnginePaths = stringRecord(raw.transformEnginePaths);
  migrated.trustedTransformEngines = booleanRecord(raw.trustedTransformEngines);
  migrated.disabledTransformEngines = booleanRecord(raw.disabledTransformEngines);
  migrated.transformInputModes = inputModeRecord(raw.transformInputModes);
  const transformTimeoutMs = numberValue(raw.transformTimeoutMs);
  if (transformTimeoutMs !== undefined) migrated.transformTimeoutMs = Math.min(Math.max(transformTimeoutMs, 1), 30000);
  migrated.customTransformTemplates = normalizeCustomTransformTemplates(raw.customTransformTemplates);
  return migrated;
}

export function migratePersistedWorkspace(value: unknown): PersistedWorkspace {
  return isRecord(value) ? normalizeWorkspaceRecord(value) : { schemaVersion: WORKSPACE_SCHEMA_VERSION };
}

export function normalizePersistedWorkspaceForSave(workspace: PersistedWorkspace): PersistedWorkspace {
  return migratePersistedWorkspace(workspace);
}
