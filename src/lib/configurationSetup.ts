export const configurationSetupSteps = [
  {
    id: "identity",
    title: "Business identity",
    summary: "Reusable name, company, address, website, tone, and client defaults for templates and generated drafts.",
    actionLabel: "Set up identity",
  },
  {
    id: "llm-access",
    title: "LLM access",
    summary: "Choose the approved provider profile, model, endpoint, and environment variable used for API requests.",
    actionLabel: "Save LLM defaults",
  },
  {
    id: "local-agents",
    title: "Local agent tools",
    summary: "Prepare governed handoffs for Claude Code, Codex, OpenCode, and Google Antigravity without storing secrets.",
    actionLabel: "Open provider handoff",
  },
  {
    id: "voice-runtime",
    title: "Docs Live voice",
    summary: "Check microphone, speech recognition, and clipboard readiness before voice-driven document creation.",
    actionLabel: "Check runtime",
  },
  {
    id: "tts",
    title: "Read aloud",
    summary: "Configure browser speech, macOS Say, or Supertonic for selected text and full-document reading.",
    actionLabel: "Check TTS",
  },
  {
    id: "exports",
    title: "Export defaults",
    summary: "Set brand, bibliography, HTML, PDF, Office, publishing, Google Docs, LaTeX, EPUB, and evidence defaults.",
    actionLabel: "Review exports",
  },
  {
    id: "google-auth",
    title: "Google Docs authorization",
    summary: "Configure Google sign-in for Docs and Drive export workflows with session-only tokens.",
    actionLabel: "Connect Google",
  },
  {
    id: "transforms",
    title: "Transforms and templates",
    summary: "Configure external engines, trusted paths, timeout, input modes, and reusable calculation templates.",
    actionLabel: "Review engines",
  },
  {
    id: "release",
    title: "Distribution readiness",
    summary: "Track Homebrew, platform packaging, signing, accessibility, performance, security, and release evidence gates.",
    actionLabel: "Open release checks",
  },
  {
    id: "imports",
    title: "Imports and data sources",
    summary: "Verify pandoc and curl are installed for document import, DOI lookup, and live REST data sources.",
    actionLabel: "Check import tools",
  },
  {
    id: "automation",
    title: "Automation and webhooks",
    summary: "Configure webhook endpoints that fire on document approval, export, and status change. Set mail merge defaults.",
    actionLabel: "Configure automation",
  },
  {
    id: "audit",
    title: "Audit and compliance",
    summary: "Enable the append-only document audit log, set authorship identity, and configure log retention for this workspace.",
    actionLabel: "Configure audit",
  },
  {
    id: "support",
    title: "Support bundle",
    summary: "Create redaction-safe setup, release, evidence, engine, and spec-completion diagnostics for help desks and internal IT.",
    actionLabel: "Preview support bundle",
  },
] as const;

export type ConfigurationSetupStepId = (typeof configurationSetupSteps)[number]["id"];

export interface ConfigurationSetupStatusItem {
  id: ConfigurationSetupStepId;
  label: string;
  done: boolean;
  detail: string;
}

export interface ConfigurationSetupStatus {
  items: ConfigurationSetupStatusItem[];
  complete: number;
  total: number;
}

export interface ConfigurationSetupStepAssistance {
  stepId: ConfigurationSetupStepId;
  stepLabel: string;
  suggestedAnswer: string;
  rationale: string;
  contextSignals: string[];
  actionLabel: string;
}

export interface ConfigurationSetupStatusInput {
  businessDone: number;
  businessTotal: number;
  businessCompletion: string;
  aiProviderProfileId: string;
  aiProviderModel: string;
  aiProviderKeyEnv: string;
  localAgentProfileCount: number;
  docsLiveRuntimeIssueCount: number | null;
  ttsReady: boolean;
  ttsRuntimeSummary: string;
  exportTarget: string;
  exportIncludeManifest: boolean;
  exportLayoutPreset: string;
  citationStyle: string;
  googleClientId: string;
  googleScopeCount: number;
  googleAuthorized: boolean;
  googleTokenExpiresAt: string;
  externalEngineCount: number;
  transformReadyOrDisabled: boolean;
  releaseEvidenceStatus: string;
  releaseEvidenceSummary: string;
  releaseEvidenceBlockedCount: number;
  releaseEvidenceManualCount: number;
  releaseEvidenceCredentialedCount: number;
  releaseEvidenceCrossPlatformCount: number;
  releaseEvidenceStaleCount: number;
  releaseEvidenceReadyToSendCount: number;
  supportBundleReady: boolean;
  supportBundleStatus: string;
  supportBundleRecommendationCount: number;
  supportBundleEvidenceAttentionCount: number;
  pandocAvailable: boolean;
  curlAvailable: boolean;
  webhookCount: number;
  auditEnabled: boolean;
}

export interface ConfigurationSetupAssistanceInput {
  step: (typeof configurationSetupSteps)[number];
  status: ConfigurationSetupStatus;
  setupSummary: string;
  setupNotesWordCount: number;
  businessDone: number;
  businessTotal: number;
  missingBusinessLabels: string[];
  agentProviderId: string;
  agentProviderModel: string;
  agentProviderEndpoint: string;
  agentProviderKeyEnv: string;
  localAgentProfileCount: number;
  docsLiveRuntimeIssueCount: number | null;
  ttsEngine: string;
  ttsRuntimeSummary: string;
  exportTarget: string;
  exportLayoutPreset: string;
  citationStyle: string;
  googleClientId: string;
  googleScopeCount: number;
  googleAuthorized: boolean;
  googleTokenExpiresAt: string;
  readyEngineCount: number;
  disabledEngineCount: number;
  externalEngineCount: number;
  releaseEvidenceStatus: string;
  releaseEvidenceSummary: string;
  releaseEvidenceBlockedCount: number;
  releaseEvidenceManualCount: number;
  releaseEvidenceCredentialedCount: number;
  releaseEvidenceCrossPlatformCount: number;
  releaseEvidenceStaleCount: number;
  releaseEvidenceReadyToSendCount: number;
  supportBundleReady: boolean;
  supportBundleStatus: string;
  supportBundleRecommendationCount: number;
  supportBundleEvidenceAttentionCount: number;
  pandocAvailable: boolean;
  curlAvailable: boolean;
  webhookCount: number;
  auditEnabled: boolean;
}

export interface ConfigurationCenterSectionInput {
  setupSummary: string;
  toolbarDisplay: string;
  editorKeymapMode: string;
  autosave: boolean;
  snapshotStorage: string;
  exportTarget: string;
  citationStyle: string;
  googleReady: boolean;
  aiProviderProfileId: string;
  ttsEngine: string;
  externalEngineCount: number;
  installerPlanCount: number;
  releaseEvidenceStatus: string;
  releaseEvidenceSummary: string;
  supportBundleStatus: string;
  supportBundleRecommendationCount: number;
  pandocAvailable: boolean;
  curlAvailable: boolean;
  webhookCount: number;
  auditEnabled: boolean;
}

export function configurationSetupStepById(stepId: string): (typeof configurationSetupSteps)[number] {
  return configurationSetupSteps.find((step) => step.id === stepId) || configurationSetupSteps[0];
}

export function isConfigurationSetupStepId(stepId: string): stepId is ConfigurationSetupStepId {
  return configurationSetupSteps.some((step) => step.id === stepId);
}

export function buildConfigurationSetupStatus(input: ConfigurationSetupStatusInput): ConfigurationSetupStatus {
  const llmDone = Boolean(input.aiProviderProfileId && input.aiProviderModel && input.aiProviderKeyEnv);
  const runtimeDone = input.docsLiveRuntimeIssueCount !== null;
  const exportDone = Boolean(input.exportTarget && input.exportIncludeManifest && input.exportLayoutPreset && input.citationStyle);
  const googleDone = Boolean(input.googleClientId && input.googleScopeCount && input.googleAuthorized);
  const releaseDone = input.releaseEvidenceReadyToSendCount > 0;
  const releaseNeedsAction =
    input.releaseEvidenceBlockedCount ||
    input.releaseEvidenceManualCount ||
    input.releaseEvidenceCredentialedCount ||
    input.releaseEvidenceCrossPlatformCount ||
    input.releaseEvidenceStaleCount;
  const items: ConfigurationSetupStatusItem[] = [
    {
      id: "identity",
      label: "Identity",
      done: input.businessDone >= Math.min(6, input.businessTotal),
      detail: input.businessCompletion,
    },
    { id: "llm-access", label: "LLM defaults", done: llmDone, detail: input.aiProviderProfileId },
    {
      id: "local-agents",
      label: "Local agents",
      done: input.localAgentProfileCount >= 4,
      detail: `${input.localAgentProfileCount} agent handoffs`,
    },
    {
      id: "voice-runtime",
      label: "Voice runtime",
      done: runtimeDone,
      detail: runtimeDone ? `${input.docsLiveRuntimeIssueCount || 0} issues` : "not checked",
    },
    {
      id: "tts",
      label: "Read aloud",
      done: input.ttsReady,
      detail: input.ttsRuntimeSummary,
    },
    { id: "exports", label: "Exports", done: exportDone, detail: input.exportTarget.toUpperCase() },
    {
      id: "google-auth",
      label: "Google Docs",
      done: googleDone,
      detail: googleDone ? `authorized until ${input.googleTokenExpiresAt || "session end"}` : "sign-in required",
    },
    {
      id: "transforms",
      label: "Transforms",
      done: input.transformReadyOrDisabled,
      detail: `${input.externalEngineCount} external engines`,
    },
    {
      id: "release",
      label: "Release gates",
      done: releaseDone,
      detail: releaseDone
        ? "ready to send"
        : releaseNeedsAction
          ? `${input.releaseEvidenceStatus}: ${input.releaseEvidenceSummary}`
          : "release evidence not generated",
    },
    {
      id: "imports",
      label: "Imports and data sources",
      done: input.pandocAvailable || input.curlAvailable,
      detail: [input.pandocAvailable ? "pandoc ready" : "pandoc missing", input.curlAvailable ? "curl ready" : "curl missing"].join("; "),
    },
    {
      id: "automation",
      label: "Automation",
      done: input.webhookCount > 0,
      detail: input.webhookCount > 0 ? `${input.webhookCount} webhook(s) configured` : "no webhooks configured",
    },
    {
      id: "audit",
      label: "Audit and compliance",
      done: input.auditEnabled,
      detail: input.auditEnabled ? "audit log enabled" : "audit log disabled",
    },
    {
      id: "support",
      label: "Support bundle",
      done: input.supportBundleReady,
      detail: input.supportBundleReady
        ? `${input.supportBundleStatus || "preview ready"}; ${input.supportBundleRecommendationCount} recommendation(s); ${input.supportBundleEvidenceAttentionCount} evidence report(s) need attention`
        : input.supportBundleStatus || "preview required",
    },
  ];
  return {
    items,
    complete: items.filter((item) => item.done).length,
    total: items.length,
  };
}

export function formatConfigurationSetupSummary(status: ConfigurationSetupStatus) {
  return `${status.complete}/${status.total} setup areas ready`;
}

export function buildConfigurationSetupStepAssistance(input: ConfigurationSetupAssistanceInput): ConfigurationSetupStepAssistance {
  const status = input.status.items.find((item) => item.id === input.step.id);
  const contextSignals = [
    `Setup area: ${input.step.title}`,
    `Status: ${status?.done ? "ready" : "needs work"}`,
    `Status detail: ${status?.detail || input.step.summary}`,
    `Overall readiness: ${input.setupSummary}`,
    `Setup notes words: ${input.setupNotesWordCount}`,
  ];
  let suggestedAnswer = "";
  let rationale = "";
  switch (input.step.id) {
    case "identity":
      suggestedAnswer = input.missingBusinessLabels.length
        ? `Complete the reusable business profile before creating production documents. Add ${input.missingBusinessLabels.slice(0, 5).join(", ")} first, then verify sender, company, default client, website, address, and brand voice.`
        : "Business identity is ready. Use the saved profile as the default sender, company, client, and brand voice for templates, snippets, Docs Live, and local-agent handoffs.";
      rationale = "Identity values appear repeatedly in proposals, RFPs, exports, snippets, and AI prompts; setting them once prevents inconsistent document metadata.";
      contextSignals.push(`Business profile fields: ${input.businessDone}/${input.businessTotal}`);
      break;
    case "llm-access":
      suggestedAnswer = `Use ${input.agentProviderId} with model ${input.agentProviderModel || "[model required]"} and keep the API key in ${input.agentProviderKeyEnv || "[environment variable required]"}. Save only non-secret defaults; enter session keys only when running a provider request.`;
      rationale = "LLM access should be easy for business users while preserving local-first security and avoiding stored secrets.";
      contextSignals.push(`Provider endpoint: ${input.agentProviderEndpoint || "not set"}`, `Key env: ${input.agentProviderKeyEnv || "not set"}`);
      break;
    case "local-agents":
      suggestedAnswer = "Keep Claude Code, Codex, OpenCode, and Google Antigravity as governed handoff targets. Verify each CLI on PATH before relying on it, and include document context, evidence gates, and rollback instructions in every handoff.";
      rationale = "Local-agent tools are powerful only when they receive bounded, auditable work packages instead of loose prompts.";
      contextSignals.push(`Agent profiles: ${input.localAgentProfileCount}`);
      break;
    case "voice-runtime":
      suggestedAnswer =
        input.docsLiveRuntimeIssueCount !== null
          ? `Voice setup has ${input.docsLiveRuntimeIssueCount} issue(s). Resolve microphone, SpeechRecognition, and clipboard blockers before promising voice-first drafting to users.`
          : "Run the Docs Live runtime check on the target device, then record whether speech recognition, microphone permission, and clipboard read/write are available without storing audio or clipboard content.";
      rationale = "Voice drafting depends on real browser/runtime permissions, so setup guidance must be based on current-device evidence.";
      contextSignals.push(input.docsLiveRuntimeIssueCount !== null ? "Runtime report present" : "Runtime report missing");
      break;
    case "tts":
      suggestedAnswer = `Use ${input.ttsEngine} for read-aloud by default. If Supertonic is selected, show the model name, size, storage location, source, and command, and require explicit acknowledgement before any model download.`;
      rationale = "Read-aloud setup must be transparent about local engines and model downloads so users control storage, bandwidth, and privacy.";
      contextSignals.push(`TTS engine: ${input.ttsEngine}`, `TTS status: ${input.ttsRuntimeSummary}`);
      break;
    case "exports":
      suggestedAnswer = `Review export defaults for ${input.exportTarget.toUpperCase()}, layout preset ${input.exportLayoutPreset}, citation style ${input.citationStyle}, manifests, approval metadata, brand settings, and target-specific publishing requirements before client delivery.`;
      rationale = "Production export setup needs consistent metadata and evidence packages across PDF, DOCX, HTML, Google Docs, LaTeX, EPUB, blog, and Substack targets.";
      contextSignals.push(`Export target: ${input.exportTarget}`, `Layout preset: ${input.exportLayoutPreset}`);
      break;
    case "google-auth":
      suggestedAnswer = input.googleAuthorized
        ? `Google authorization is ready for this session with ${input.googleScopeCount} scope(s). Keep Google tokens session-only, use the configured desktop OAuth client, and request in-memory session refresh when long Google Docs import/readback evidence may outlive the access token expiry at ${input.googleTokenExpiresAt || "session end"}.`
        : `Add a desktop OAuth client ID, keep the Google Docs and Drive scopes least-privilege, request session refresh for longer import/readback workflows, then sign in with Google before exporting or verifying Google Docs collaboration packages.`;
      rationale = "Google Docs distribution needs an explicit user grant; NEditor should never store Google access tokens in workspace preferences.";
      contextSignals.push(
        `Google client ID: ${input.googleClientId ? "configured" : "missing"}`,
        `Google scope count: ${input.googleScopeCount}`,
        `Google authorized: ${input.googleAuthorized ? "yes" : "no"}`,
      );
      break;
    case "transforms":
      suggestedAnswer = `Configure only the transform engines users actually need. Ready engines: ${input.readyEngineCount}; disabled engines: ${input.disabledEngineCount}; total known engines: ${input.externalEngineCount}. Keep untrusted handlers disabled until paths, permissions, timeouts, and input modes are verified.`;
      rationale = "Transform setup can execute external tools, so handler readiness must be explicit, trust-gated, and easy to audit.";
      contextSignals.push(`Ready engines: ${input.readyEngineCount}`, `Disabled engines: ${input.disabledEngineCount}`);
      break;
    case "release":
      suggestedAnswer =
        input.releaseEvidenceReadyToSendCount > 0
          ? `Release evidence is ready to send. Archive the evidence dashboard, export visual QA, accessibility QA, signed/notarized artifacts, Homebrew audit, and platform package proof with the release packet.`
          : `Keep release setup open until the evidence dashboard has no blocked or stale lanes. Current release evidence is ${input.releaseEvidenceStatus}: ${input.releaseEvidenceSummary}. Resolve blocked (${input.releaseEvidenceBlockedCount}), stale (${input.releaseEvidenceStaleCount}), manual (${input.releaseEvidenceManualCount}), credentialed (${input.releaseEvidenceCredentialedCount}), and cross-platform (${input.releaseEvidenceCrossPlatformCount}) lanes before distribution.`;
      rationale = "Release readiness should be based on the same evidence lanes release managers inspect before publishing, not on a static setup placeholder.";
      contextSignals.push(
        `Release status: ${input.releaseEvidenceStatus}`,
        `Release summary: ${input.releaseEvidenceSummary}`,
        `Ready-to-send lanes: ${input.releaseEvidenceReadyToSendCount}`,
      );
      break;
    case "imports":
      suggestedAnswer = input.pandocAvailable && input.curlAvailable
        ? "Both pandoc and curl are available. Document import, DOI lookup, and live REST data source population are ready to use."
        : `Install missing tools before relying on import or data-source features: ${[!input.pandocAvailable ? "pandoc" : null, !input.curlAvailable ? "curl" : null].filter(Boolean).join(", ")}. pandoc handles document conversion; curl is required for DOI resolution and REST data endpoints.`;
      rationale = "Import and data-source workflows fail silently when binary dependencies are absent; checking them at setup time surfaces the gap before users hit it mid-document.";
      contextSignals.push(
        `pandoc available: ${input.pandocAvailable ? "yes" : "no"}`,
        `curl available: ${input.curlAvailable ? "yes" : "no"}`,
      );
      break;
    case "automation":
      suggestedAnswer = input.webhookCount > 0
        ? `${input.webhookCount} webhook endpoint(s) are configured. Review event bindings for document approval, export completion, and status change, and confirm mail merge defaults are set for batch dispatch.`
        : "No webhook endpoints are configured. Add at least one endpoint for document approval or export events to enable automated downstream integrations. Set mail merge defaults before first batch run.";
      rationale = "Webhook and mail merge automation reduces manual handoff steps; configuring endpoints at setup time prevents silent drops when approval or export events fire.";
      contextSignals.push(`Webhook count: ${input.webhookCount}`);
      break;
    case "audit":
      suggestedAnswer = input.auditEnabled
        ? "The append-only audit log is enabled. Confirm authorship identity is set, log retention matches your compliance policy, and document compare history and humanizer audit records are included in the log scope."
        : "Enable the append-only document audit log before handling production documents. Set authorship identity and log retention period to satisfy compliance requirements. Audit records for compare history and humanizer rewrites should be included from the start.";
      rationale = "Audit trails must be established before documents are created; retroactive logging cannot capture authorship or change provenance for records already written.";
      contextSignals.push(`Audit log enabled: ${input.auditEnabled ? "yes" : "no"}`);
      break;
    case "support":
      suggestedAnswer = input.supportBundleReady
        ? `Support bundle preview is ready. Review ${input.supportBundleRecommendationCount} recommendation(s) and ${input.supportBundleEvidenceAttentionCount} evidence report(s) needing attention, then save the JSON before handing the installation or release case to help desk, internal IT, or release management.`
        : "Preview the support bundle before support handoff. The bundle should summarize setup diagnostics, release readiness, evidence reports, transform engines, specification work orders, and release-candidate state without including document content or secrets.";
      rationale = "Non-technical support teams need one redaction-safe artifact that explains setup state, release gaps, and evidence work without requiring them to inspect developer tools.";
      contextSignals.push(
        `Support bundle status: ${input.supportBundleStatus || "not previewed"}`,
        `Support recommendations: ${input.supportBundleRecommendationCount}`,
        `Evidence reports needing attention: ${input.supportBundleEvidenceAttentionCount}`,
      );
      break;
  }
  return {
    stepId: input.step.id,
    stepLabel: input.step.title,
    suggestedAnswer,
    rationale,
    contextSignals: Array.from(new Set(contextSignals)),
    actionLabel: "Add to setup notes",
  };
}

export function configurationSetupAssistanceBlock(assistance: ConfigurationSetupStepAssistance) {
  return [
    `${assistance.stepLabel}: ${assistance.suggestedAnswer}`,
    `Rationale: ${assistance.rationale}`,
    `Context signals: ${assistance.contextSignals.join("; ")}`,
  ].join("\n");
}

export function buildConfigurationCenterSections(input: ConfigurationCenterSectionInput) {
  return [
    {
      id: "overview",
      label: "Overview",
      summary: input.setupSummary,
      detail: "Start here for setup readiness and guided configuration.",
    },
    {
      id: "appearance",
      label: "Appearance and editor",
      summary: `${input.toolbarDisplay}; ${input.editorKeymapMode}`,
      detail: "Theme, toolbar density, editor ergonomics, typography, and accessibility.",
    },
    {
      id: "files",
      label: "Files and history",
      summary: `${input.autosave ? "autosave on" : "autosave off"}; ${input.snapshotStorage}`,
      detail: "Autosave, snapshots, Git behavior, recents, and workspace recovery.",
    },
    {
      id: "exports",
      label: "Exports and brand",
      summary: `${input.exportTarget.toUpperCase()}; ${input.citationStyle}`,
      detail: "Export defaults, publishing metadata, bibliography style, layout, and brand package.",
    },
    {
      id: "google-auth",
      label: "Google Docs",
      summary: input.googleReady ? "authorized" : "needs sign-in",
      detail: "Google account authorization, Docs scopes, Drive file access, and session token state.",
    },
    {
      id: "ai",
      label: "AI, agents, and voice",
      summary: `${input.aiProviderProfileId}; ${input.ttsEngine}`,
      detail: "LLM access, local agents, AI cleanup, Docs Live runtime, and read-aloud setup.",
    },
    {
      id: "transforms",
      label: "Transforms",
      summary: `${input.externalEngineCount} external engines; ${input.installerPlanCount} installer plan`,
      detail: "Download handlers, set executable paths, trust engines, probe setup, timeout, and execution modes.",
    },
    {
      id: "imports",
      label: "Imports and data",
      summary: input.pandocAvailable && input.curlAvailable
        ? "pandoc and curl ready"
        : input.pandocAvailable
          ? "pandoc ready; curl missing"
          : input.curlAvailable
            ? "curl ready; pandoc missing"
            : "pandoc and curl not found",
      detail: "Document import via pandoc, DOI lookup, live REST data sources, and mail merge field population.",
    },
    {
      id: "automation",
      label: "Automation",
      summary: input.webhookCount > 0 ? `${input.webhookCount} webhook(s) configured` : "no webhooks configured",
      detail: "Webhook endpoints for document approval, export, and status change events; mail merge defaults and batch dispatch settings.",
    },
    {
      id: "audit",
      label: "Audit and compliance",
      summary: input.auditEnabled ? "audit log enabled" : "audit log disabled",
      detail: "Append-only document audit trail, authorship identity, log retention policy, document compare history, and humanizer audit records.",
    },
    {
      id: "release",
      label: "Release evidence",
      summary: `${input.releaseEvidenceStatus}; ${input.releaseEvidenceSummary}`,
      detail: "Release gates, evidence freshness, credentialed proof, platform packaging, signing, Homebrew, and ready-to-send state.",
    },
    {
      id: "support",
      label: "Support and diagnostics",
      summary: `${input.supportBundleStatus || "preview required"}; ${input.supportBundleRecommendationCount} recommendation(s)`,
      detail: "Redaction-safe support bundle preview, setup diagnostics, release evidence summaries, spec work orders, transform health, and handoff JSON.",
    },
  ] as const;
}
