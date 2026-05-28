import {
  normalizeBrandProfileDefaults,
  normalizeCitationStyle,
  normalizeExportDefaults,
  type BibliographyDefaults,
  type BrandProfileDefaults,
  type ExportDefaults,
  type GitIntegrationPreferences,
  type TransformInputMode,
} from "./workspacePersistence.js";

export interface DocumentTransformOptionState {
  transformEnginePaths: Record<string, string>;
  trustedTransformEngines: Record<string, boolean>;
  disabledTransformEngines: Record<string, boolean>;
  transformInputModes: Record<string, TransformInputMode>;
  transformTimeoutMs: number;
}

export interface DocumentCompileOptionState extends DocumentTransformOptionState {
  bibliographyDefaults: BibliographyDefaults;
  brandProfileDefaults: BrandProfileDefaults;
}

export interface DocumentExportOptionState extends DocumentCompileOptionState {
  exportDefaults: ExportDefaults;
  gitIntegration: GitIntegrationPreferences;
  semanticStatus?: string | null;
}

export function buildDocumentCompileOptions(state: DocumentCompileOptionState) {
  return {
    defaultCitationStyle: normalizeCitationStyle(state.bibliographyDefaults.citationStyle),
    defaultBrandProfile: normalizeBrandProfileDefaults(state.brandProfileDefaults),
    transformEnginePaths: state.transformEnginePaths,
    trustedTransformEngines: state.trustedTransformEngines,
    disabledTransformEngines: state.disabledTransformEngines,
    transformInputModes: state.transformInputModes,
    transformTimeoutMs: state.transformTimeoutMs,
  };
}

export function buildDocumentExportOptions(state: DocumentExportOptionState) {
  const defaults = normalizeExportDefaults(state.exportDefaults);
  const compileOptions = buildDocumentCompileOptions(state);
  const brandProfile = compileOptions.defaultBrandProfile;
  return {
    includeManifest: defaults.includeManifest,
    includeStyles: defaults.includeStyles,
    includeSyntaxHighlighting: defaults.includeSyntaxHighlighting,
    htmlLanguage: defaults.htmlLanguage,
    htmlDescription: defaults.htmlDescription,
    canonicalUrl: defaults.canonicalUrl,
    coverPage: defaults.coverPage,
    pageNumbers: defaults.pageNumbers,
    layoutPreset: defaults.layoutPreset,
    includeComments: defaults.includeComments,
    includeProvenance: defaults.includeProvenance,
    includeGlossary: defaults.includeGlossary,
    includeAgenda: defaults.includeAgenda,
    ...compileOptions,
    warnOnDirtyGit: state.gitIntegration.enabled && state.gitIntegration.warnOnDirtyExport,
    watermark: state.semanticStatus === "draft" ? "DRAFT" : brandProfile.watermark,
  };
}
