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
  const exportDone = Boolean(input.exportIncludeManifest && input.exportLayoutPreset && input.citationStyle);
  const googleDone = Boolean(input.googleClientId && input.googleScopeCount && input.googleAuthorized);
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
    { id: "release", label: "Release gates", done: false, detail: "external evidence required" },
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
        ? `Google authorization is ready for this session with ${input.googleScopeCount} scope(s). Keep the access token session-only, use the configured desktop OAuth client, and reauthorize before Google Docs import/export evidence if the token expires at ${input.googleTokenExpiresAt || "session end"}.`
        : `Add a desktop OAuth client ID, keep the Google Docs and Drive scopes least-privilege, then sign in with Google before exporting or verifying Google Docs collaboration packages.`;
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
      suggestedAnswer = "Treat release readiness as incomplete until external evidence is supplied for signing/notarization, Windows and Linux package proof, Google Docs live import/readback, live provider evidence, real-device AI runtime evidence, rendered export sign-off, accessibility sign-off, and sustained performance profiling.";
      rationale = "Some production gates cannot be proven on the current host; they must remain visible release blockers rather than being hidden behind local green checks.";
      contextSignals.push("External evidence required");
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
  ] as const;
}
