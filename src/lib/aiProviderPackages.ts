import type { AgenticWorkflowRun } from "./agenticWorkflows.js";

export type AiProviderProfileId = "manual-review" | "openai-compatible" | "anthropic-compatible" | "gemini-compatible" | "local-http";
export type AiProviderBodyStyle = "messages" | "system-and-messages" | "contents" | "prompt";

export interface AiProviderProfile {
  id: AiProviderProfileId;
  label: string;
  endpoint: string;
  model: string;
  bodyStyle: AiProviderBodyStyle;
  authHeader: string;
  summary: string;
}

export interface AiProviderRequestPackageOptions {
  profileId?: AiProviderProfileId;
  endpoint?: string;
  model?: string;
  keyEnv?: string;
}

export interface AiProviderRequestPackage {
  profile: AiProviderProfile;
  systemPrompt: string;
  userPrompt: string;
  requestBody: Record<string, unknown>;
  redactedHeaders: Record<string, string>;
  curl: string;
  checklist: string[];
  markdown: string;
}

export interface AiProviderExecutionResult {
  ok: boolean;
  status: number;
  statusText: string;
  markdown: string;
  rawText: string;
}

export type AiProviderFetch = (input: string, init: { method: string; headers: Record<string, string>; body: string }) => Promise<{
  ok: boolean;
  status: number;
  statusText: string;
  text(): Promise<string>;
}>;

export const aiProviderProfiles: AiProviderProfile[] = [
  {
    id: "manual-review",
    label: "Manual provider handoff",
    endpoint: "",
    model: "human-approved-provider",
    bodyStyle: "prompt",
    authHeader: "",
    summary: "Creates a provider-neutral prompt package for secure copy/paste into an approved AI tool.",
  },
  {
    id: "openai-compatible",
    label: "OpenAI-compatible JSON",
    endpoint: "https://api.openai.com/v1/chat/completions",
    model: "gpt-4.1",
    bodyStyle: "messages",
    authHeader: "Authorization",
    summary: "Creates a redacted HTTP JSON starter for OpenAI-compatible chat endpoints.",
  },
  {
    id: "anthropic-compatible",
    label: "Anthropic-compatible JSON",
    endpoint: "https://api.anthropic.com/v1/messages",
    model: "claude-sonnet",
    bodyStyle: "system-and-messages",
    authHeader: "x-api-key",
    summary: "Creates a redacted HTTP JSON starter for Claude-style message endpoints.",
  },
  {
    id: "gemini-compatible",
    label: "Gemini-compatible JSON",
    endpoint: "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent",
    model: "gemini-pro",
    bodyStyle: "contents",
    authHeader: "x-goog-api-key",
    summary: "Creates a redacted HTTP JSON starter for Gemini-style content endpoints.",
  },
  {
    id: "local-http",
    label: "Local HTTP model",
    endpoint: "http://127.0.0.1:11434/api/generate",
    model: "local-document-model",
    bodyStyle: "prompt",
    authHeader: "",
    summary: "Creates a local HTTP prompt package for private model gateways.",
  },
];

export function providerProfileById(id: string | undefined): AiProviderProfile {
  return aiProviderProfiles.find((profile) => profile.id === id) || aiProviderProfiles[0];
}

export function buildAiProviderRequestPackage(
  run: AgenticWorkflowRun,
  options: AiProviderRequestPackageOptions = {},
): AiProviderRequestPackage {
  const baseProfile = providerProfileById(options.profileId);
  const profile: AiProviderProfile = {
    ...baseProfile,
    endpoint: normalizeEndpoint(options.endpoint) || baseProfile.endpoint,
    model: normalizeField(options.model, 120) || baseProfile.model,
  };
  const keyEnv = normalizeEnvName(options.keyEnv) || "NEDITOR_AI_API_KEY";
  const systemPrompt = buildSystemPrompt(run);
  const userPrompt = buildUserPrompt(run);
  const requestBody = buildRequestBody(profile, systemPrompt, userPrompt);
  const redactedHeaders = buildHeaders(profile, keyEnv);
  const curl = buildCurl(profile, redactedHeaders, requestBody);
  const checklist = buildChecklist(profile, keyEnv);
  const markdown = buildMarkdown(profile, systemPrompt, userPrompt, requestBody, redactedHeaders, curl, checklist);

  return {
    profile,
    systemPrompt,
    userPrompt,
    requestBody,
    redactedHeaders,
    curl,
    checklist,
    markdown,
  };
}

export async function executeAiProviderRequestPackage(
  requestPackage: AiProviderRequestPackage,
  apiKey = "",
  fetcher: AiProviderFetch = globalThis.fetch.bind(globalThis) as AiProviderFetch,
): Promise<AiProviderExecutionResult> {
  if (!requestPackage.profile.endpoint) {
    throw new Error("This provider profile is a manual handoff and does not define an endpoint.");
  }
  const headers = concreteHeaders(requestPackage.redactedHeaders, apiKey);
  const endpoint = requestPackage.profile.endpoint.replace("{model}", encodeURIComponent(requestPackage.profile.model));
  const response = await fetcher(endpoint, {
    method: "POST",
    headers,
    body: JSON.stringify(requestPackage.requestBody),
  });
  const rawText = await response.text();
  const markdown = extractProviderMarkdown(rawText, requestPackage.profile.bodyStyle);
  if (!response.ok) {
    throw new Error(`Provider request failed: ${response.status} ${response.statusText}${rawText ? ` - ${rawText.slice(0, 240)}` : ""}`);
  }
  if (!markdown.trim()) {
    throw new Error("Provider response did not contain usable Markdown text.");
  }
  return {
    ok: response.ok,
    status: response.status,
    statusText: response.statusText,
    markdown,
    rawText,
  };
}

function buildSystemPrompt(run: AgenticWorkflowRun) {
  return [
    "You are an expert document co-writer inside NEditor.",
    "Respect local-first document ownership and preserve Markdown structure.",
    "Return only Markdown that a human reviewer can inspect.",
    "Keep AI provenance, unresolved assumptions, QA gates, and reviewer handoff notes visible.",
    `Workflow lanes: ${run.plan.lanes.join(", ")}.`,
    `Document type: ${run.plan.documentType}.`,
    `Application mode: ${run.applicationMode}.`,
  ].join("\n");
}

function buildUserPrompt(run: AgenticWorkflowRun) {
  return [
    `Instruction:\n${run.plan.instruction || "Improve this document."}`,
    "",
    `Context:\n${run.plan.context}`,
    "",
    `Placeholders:\n${run.plan.placeholderText}`,
    "",
    `Suggested outline:\n${run.plan.suggestedOutline}`,
    "",
    `Reviewer agents:\n${run.reviewerAgents.map((agent) => `- ${agent.label} [${agent.status}]: ${agent.mandate}`).join("\n")}`,
    "",
    `Section work queue:\n${run.sectionWorkQueue
      .map((section) => `- ${section.order}. ${section.heading} (${section.lane}; reviewers: ${section.reviewerAgentIds.join(", ")}): ${section.draftingInstruction}`)
      .join("\n")}`,
    "",
    run.revision
      ? `Revision proposal to improve:\n${run.revision.proposedText}`
      : `Agent draft packet to improve:\n${run.markdown}`,
    "",
    "Required response:",
    "- Return Markdown only.",
    "- Keep unresolved facts visibly marked.",
    "- Preserve or add ai-source and ai-assisted review metadata.",
    "- Include a final QA checklist and review handoff.",
  ].join("\n");
}

function concreteHeaders(headers: Record<string, string>, apiKey: string) {
  const concrete: Record<string, string> = {};
  for (const [key, value] of Object.entries(headers)) {
    if (value.includes("${")) {
      if (!apiKey.trim()) throw new Error(`Provider request needs a session API key for ${key}.`);
      concrete[key] = value.toLowerCase().startsWith("bearer ") ? `Bearer ${apiKey.trim()}` : apiKey.trim();
    } else {
      concrete[key] = value;
    }
  }
  return concrete;
}

function extractProviderMarkdown(rawText: string, bodyStyle: AiProviderBodyStyle) {
  const parsed = parseJson(rawText);
  if (!parsed) return rawText.trim();
  if (bodyStyle === "messages") {
    const choices = arrayValue(parsed.choices);
    const first = recordValue(choices[0]);
    const message = recordValue(first?.message);
    return stringValue(message?.content) || stringValue(first?.text) || rawText.trim();
  }
  if (bodyStyle === "system-and-messages") {
    const content = arrayValue(parsed.content);
    return content.map((item) => stringValue(recordValue(item)?.text)).filter(Boolean).join("\n\n") || stringValue(parsed.completion) || rawText.trim();
  }
  if (bodyStyle === "contents") {
    const candidates = arrayValue(parsed.candidates);
    const candidate = recordValue(candidates[0]);
    const content = recordValue(candidate?.content);
    const parts = arrayValue(content?.parts);
    return parts.map((item) => stringValue(recordValue(item)?.text)).filter(Boolean).join("\n\n") || rawText.trim();
  }
  return stringValue(parsed.response) || stringValue(parsed.output) || stringValue(parsed.text) || rawText.trim();
}

function parseJson(value: string): Record<string, unknown> | null {
  try {
    const parsed = JSON.parse(value);
    return typeof parsed === "object" && parsed !== null && !Array.isArray(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function recordValue(value: unknown): Record<string, unknown> | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value) ? (value as Record<string, unknown>) : undefined;
}

function arrayValue(value: unknown): unknown[] {
  return Array.isArray(value) ? value : [];
}

function stringValue(value: unknown) {
  return typeof value === "string" ? value.trim() : "";
}

function buildRequestBody(profile: AiProviderProfile, systemPrompt: string, userPrompt: string): Record<string, unknown> {
  if (profile.bodyStyle === "system-and-messages") {
    return {
      model: profile.model,
      system: systemPrompt,
      messages: [{ role: "user", content: userPrompt }],
      temperature: 0.2,
    };
  }
  if (profile.bodyStyle === "contents") {
    return {
      model: profile.model,
      contents: [{ role: "user", parts: [{ text: `${systemPrompt}\n\n${userPrompt}` }] }],
      generationConfig: { temperature: 0.2 },
    };
  }
  if (profile.bodyStyle === "prompt") {
    return {
      model: profile.model,
      prompt: `${systemPrompt}\n\n${userPrompt}`,
      stream: false,
      temperature: 0.2,
    };
  }
  return {
    model: profile.model,
    messages: [
      { role: "system", content: systemPrompt },
      { role: "user", content: userPrompt },
    ],
    temperature: 0.2,
  };
}

function buildHeaders(profile: AiProviderProfile, keyEnv: string) {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (profile.authHeader) {
    headers[profile.authHeader] = profile.authHeader.toLowerCase() === "authorization" ? `Bearer \${${keyEnv}}` : `\${${keyEnv}}`;
  }
  return headers;
}

function buildCurl(profile: AiProviderProfile, headers: Record<string, string>, body: Record<string, unknown>) {
  if (!profile.endpoint) return "";
  const headerArgs = Object.entries(headers)
    .map(([key, value]) => `  -H '${shellEscape(`${key}: ${value}`)}'`)
    .join(" \\\n");
  return [
    `curl -sS '${shellEscape(profile.endpoint.replace("{model}", encodeURIComponent(profile.model)))}' \\`,
    "  -X POST \\",
    headerArgs ? `${headerArgs} \\` : "",
    `  --data '${shellEscape(JSON.stringify(body, null, 2))}'`,
  ]
    .filter(Boolean)
    .join("\n");
}

function buildChecklist(profile: AiProviderProfile, keyEnv: string) {
  return [
    "Confirm your organization approves this provider and model for the document classification.",
    profile.endpoint ? "Review the endpoint before sending any content." : "Paste the prompt only into an approved provider workspace.",
    profile.authHeader ? `Store the API key outside NEditor source as ${keyEnv}; never paste secrets into Markdown.` : "",
    "Remove sensitive client data unless the provider contract allows it.",
    "Save the provider response as a review draft, not as approved final content.",
    "Run NEditor review readiness and export readiness after importing the response.",
  ].filter(Boolean);
}

function buildMarkdown(
  profile: AiProviderProfile,
  systemPrompt: string,
  userPrompt: string,
  requestBody: Record<string, unknown>,
  headers: Record<string, string>,
  curl: string,
  checklist: string[],
) {
  return [
    `# ${profile.label} Request Package`,
    "",
    profile.summary,
    "",
    "## Safety Checklist",
    "",
    ...checklist.map((item) => `- [ ] ${item}`),
    "",
    "## System Prompt",
    "",
    fencedBlock("text", systemPrompt),
    "",
    "## User Prompt",
    "",
    fencedBlock("text", userPrompt),
    "",
    "## Redacted Headers",
    "",
    fencedBlock("json", JSON.stringify(headers, null, 2)),
    "",
    "## Request Body",
    "",
    fencedBlock("json", JSON.stringify(requestBody, null, 2)),
    "",
    curl ? "## cURL Starter" : "",
    curl ? "" : "",
    curl ? fencedBlock("bash", curl) : "",
    "",
  ]
    .filter((line, index, lines) => line || lines[index - 1] !== "")
    .join("\n")
    .trimEnd() + "\n";
}

function fencedBlock(language: string, value: string) {
  return ["```" + language, value.trim(), "```"].join("\n");
}

function normalizeEndpoint(value: string | undefined) {
  const trimmed = (value || "").trim();
  if (!trimmed) return "";
  if (!/^https?:\/\//i.test(trimmed)) return "";
  return trimmed.slice(0, 2048);
}

function normalizeField(value: string | undefined, limit: number) {
  return (value || "").trim().slice(0, limit);
}

function normalizeEnvName(value: string | undefined) {
  const normalized = (value || "").trim().replace(/[^A-Z0-9_]/gi, "_").toUpperCase();
  return /^[A-Z][A-Z0-9_]{2,80}$/.test(normalized) ? normalized : "";
}

function shellEscape(value: string) {
  return value.replace(/'/g, "'\\''");
}
