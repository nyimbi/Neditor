/**
 * NEditor Ollama model catalog, streaming execution, and context management.
 * Curated ≤9B model recommendations for local AI business document generation.
 */

export type OllamaTaskType =
  | "business-writing"
  | "long-document"
  | "summarization"
  | "reasoning"
  | "code"
  | "fast-edit";

export interface OllamaModelCard {
  id: string;
  label: string;
  family: string;
  params: string;
  contextTokens: number;
  vramGb: number;
  diskGb: number;
  speed: "fast" | "medium" | "slow";
  quality: "excellent" | "good" | "adequate";
  tasks: OllamaTaskType[];
  recommended: boolean;
  badge: string;
  whyRecommended: string;
  temperature: number;
  numCtx: number;
  repeatPenalty: number;
  tags: string[];
}

export const OLLAMA_MODEL_CATALOG: OllamaModelCard[] = [
  {
    id: "qwen2.5:7b",
    label: "Qwen 2.5 7B",
    family: "Qwen",
    params: "7B",
    contextTokens: 131072,
    vramGb: 5.5,
    diskGb: 4.7,
    speed: "medium",
    quality: "excellent",
    tasks: ["business-writing", "long-document", "summarization"],
    recommended: true,
    badge: "Best overall",
    whyRecommended: "Top pick for business documents. 128k context handles book-length drafts. Superior instruction following and formatting discipline. Multilingual.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.1,
    tags: ["multilingual", "long-context", "instruction-following"],
  },
  {
    id: "llama3.1:8b",
    label: "Llama 3.1 8B",
    family: "Llama",
    params: "8B",
    contextTokens: 131072,
    vramGb: 6.5,
    diskGb: 4.9,
    speed: "medium",
    quality: "excellent",
    tasks: ["business-writing", "long-document", "summarization"],
    recommended: true,
    badge: "Most reliable",
    whyRecommended: "Meta's flagship 8B. Battle-tested, reliable, excellent markdown structure. 128k context. Safe default for any new setup.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.08,
    tags: ["reliable", "long-context", "well-tested"],
  },
  {
    id: "gemma2:9b",
    label: "Gemma 2 9B",
    family: "Gemma",
    params: "9B",
    contextTokens: 8192,
    vramGb: 7.0,
    diskGb: 5.5,
    speed: "medium",
    quality: "excellent",
    tasks: ["business-writing", "summarization"],
    recommended: true,
    badge: "Best quality",
    whyRecommended: "Google's highest quality at ≤9B. Superior writing fluency and coherence. Use for executive summaries and high-stakes prose where quality beats length.",
    temperature: 0.15,
    numCtx: 8192,
    repeatPenalty: 1.0,
    tags: ["high-quality", "google", "executive-prose"],
  },
  {
    id: "deepseek-r1:8b",
    label: "DeepSeek R1 8B",
    family: "DeepSeek",
    params: "8B",
    contextTokens: 131072,
    vramGb: 6.5,
    diskGb: 4.9,
    speed: "slow",
    quality: "excellent",
    tasks: ["reasoning", "summarization", "long-document"],
    recommended: true,
    badge: "Best reasoning",
    whyRecommended: "Best reasoning model at ≤9B. Use for citation verification, quality assurance, analytical summaries, and complex structured extraction.",
    temperature: 0.1,
    numCtx: 32768,
    repeatPenalty: 1.05,
    tags: ["reasoning", "analytical", "QA", "thinking"],
  },
  {
    id: "phi4-mini:3.8b",
    label: "Phi-4 Mini 3.8B",
    family: "Phi",
    params: "3.8B",
    contextTokens: 131072,
    vramGb: 3.0,
    diskGb: 2.5,
    speed: "fast",
    quality: "good",
    tasks: ["fast-edit", "summarization", "long-document"],
    recommended: true,
    badge: "CPU friendly",
    whyRecommended: "Microsoft compact model with 128k context. Best VRAM efficiency. Ideal for CPU-only machines or 4 GB VRAM systems.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.1,
    tags: ["low-vram", "cpu-friendly", "long-context"],
  },
  {
    id: "gemma3:4b",
    label: "Gemma 3 4B",
    family: "Gemma",
    params: "4B",
    contextTokens: 131072,
    vramGb: 3.5,
    diskGb: 3.3,
    speed: "fast",
    quality: "good",
    tasks: ["fast-edit", "summarization", "long-document"],
    recommended: true,
    badge: "Best small",
    whyRecommended: "Newest Gemma with 128k context. Best quality-to-speed ratio for fast iteration. Excellent at following structured templates.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.1,
    tags: ["fast", "long-context", "gemma"],
  },
  {
    id: "qwen2.5:3b",
    label: "Qwen 2.5 3B",
    family: "Qwen",
    params: "3B",
    contextTokens: 131072,
    vramGb: 2.5,
    diskGb: 1.9,
    speed: "fast",
    quality: "good",
    tasks: ["fast-edit", "summarization"],
    recommended: true,
    badge: "Fastest Qwen",
    whyRecommended: "Qwen quality in a tiny package. 128k context, 2.5 GB VRAM. Ideal for quick section rewrites and short summaries.",
    temperature: 0.2,
    numCtx: 16384,
    repeatPenalty: 1.1,
    tags: ["fast", "small", "qwen"],
  },
  {
    id: "qwen2.5-coder:7b",
    label: "Qwen 2.5 Coder 7B",
    family: "Qwen",
    params: "7B",
    contextTokens: 131072,
    vramGb: 5.5,
    diskGb: 4.7,
    speed: "medium",
    quality: "excellent",
    tasks: ["code", "reasoning"],
    recommended: true,
    badge: "Best for transforms",
    whyRecommended: "Best code-specialised model at 7B. Use for Python/R/SQL transform blocks, calculations, and structured data generation.",
    temperature: 0.1,
    numCtx: 32768,
    repeatPenalty: 1.0,
    tags: ["code", "transforms", "sql", "python"],
  },
  {
    id: "mistral:7b",
    label: "Mistral 7B",
    family: "Mistral",
    params: "7B",
    contextTokens: 32768,
    vramGb: 5.5,
    diskGb: 4.1,
    speed: "medium",
    quality: "good",
    tasks: ["business-writing", "summarization"],
    recommended: false,
    badge: "Industry standard",
    whyRecommended: "Industry-standard 7B. Reliable, widely tested, excellent OpenAI API compatibility.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.1,
    tags: ["compatible", "reliable", "mistral"],
  },
  {
    id: "llama3.2:3b",
    label: "Llama 3.2 3B",
    family: "Llama",
    params: "3B",
    contextTokens: 131072,
    vramGb: 2.5,
    diskGb: 2.0,
    speed: "fast",
    quality: "good",
    tasks: ["fast-edit", "summarization"],
    recommended: false,
    badge: "Compact Llama",
    whyRecommended: "Meta's compact 3B. Fast for quick paragraph edits and short summaries.",
    temperature: 0.2,
    numCtx: 16384,
    repeatPenalty: 1.1,
    tags: ["fast", "small", "llama"],
  },
];

export const RECOMMENDED_TASK_MODELS: Record<OllamaTaskType, string[]> = {
  "business-writing": ["qwen2.5:7b", "llama3.1:8b", "gemma2:9b"],
  "long-document":    ["qwen2.5:7b", "llama3.1:8b", "phi4-mini:3.8b"],
  "summarization":    ["qwen2.5:7b", "gemma2:9b", "gemma3:4b"],
  "reasoning":        ["deepseek-r1:8b", "qwen2.5:7b", "llama3.1:8b"],
  "code":             ["qwen2.5-coder:7b", "deepseek-r1:8b", "qwen2.5:7b"],
  "fast-edit":        ["phi4-mini:3.8b", "gemma3:4b", "qwen2.5:3b"],
};

export function modelCardById(id: string): OllamaModelCard | undefined {
  return OLLAMA_MODEL_CATALOG.find(m => m.id === id);
}

export function recommendedModelsForTask(task: OllamaTaskType): OllamaModelCard[] {
  const ids = RECOMMENDED_TASK_MODELS[task] ?? [];
  return ids.map(id => modelCardById(id)).filter(Boolean) as OllamaModelCard[];
}

export function bestAvailableModel(
  installedModelNames: string[],
  task: OllamaTaskType,
): OllamaModelCard | undefined {
  const recommended = recommendedModelsForTask(task);
  const installed = new Set(installedModelNames.map(n => {
    const parts = n.split(":");
    return parts[0] + ":" + (parts.length > 1 ? parts.slice(1).join(":") : "latest");
  }));
  for (const model of recommended) {
    if (installed.has(model.id)) return model;
  }
  for (const model of recommended) {
    const family = model.id.split(":")[0];
    if ([...installed].some(n => n.startsWith(family))) return model;
  }
  return undefined;
}

// ─── Token estimation ──────────────────────────────────────────────────────────

/** Rough token estimate: ~3.5 chars/token for English business text. */
export function estimateTokenCount(text: string): number {
  return Math.ceil(text.length / 3.5);
}

export interface ContextBudget {
  systemTokens: number;
  userTokens: number;
  totalUsed: number;
  windowTokens: number;
  reservedForOutput: number;
  available: number;
  utilizationPct: number;
  overBudget: boolean;
}

export function computeContextBudget(
  systemPrompt: string,
  userPrompt: string,
  contextWindow: number,
  reserveOutput = 4096,
): ContextBudget {
  const systemTokens = estimateTokenCount(systemPrompt);
  const userTokens = estimateTokenCount(userPrompt);
  const totalUsed = systemTokens + userTokens;
  const budget = contextWindow - reserveOutput;
  return {
    systemTokens,
    userTokens,
    totalUsed,
    windowTokens: contextWindow,
    reservedForOutput: reserveOutput,
    available: Math.max(0, budget - totalUsed),
    utilizationPct: Math.min(100, Math.round((totalUsed / budget) * 100)),
    overBudget: totalUsed > budget,
  };
}

export function trimToContextBudget(
  systemPrompt: string,
  userPrompt: string,
  contextWindow: number,
  reserveOutput = 4096,
): { systemPrompt: string; userPrompt: string; truncated: boolean } {
  const budget = contextWindow - reserveOutput;
  const totalChars = budget * 3.5;
  if (systemPrompt.length + userPrompt.length <= totalChars) {
    return { systemPrompt, userPrompt, truncated: false };
  }
  const systemAlloc = Math.min(systemPrompt.length, Math.floor(totalChars * 0.25));
  const userBudget = totalChars - systemAlloc;
  const trimmedSystem = systemPrompt.slice(0, systemAlloc);
  const half = Math.floor(userBudget / 2);
  const trimmedUser = userPrompt.length > userBudget
    ? userPrompt.slice(0, half) + "\n\n[… document context trimmed to fit context window …]\n\n" + userPrompt.slice(-half)
    : userPrompt;
  return { systemPrompt: trimmedSystem, userPrompt: trimmedUser, truncated: true };
}

// ─── Ollama health check ───────────────────────────────────────────────────────

export interface OllamaHealthResult {
  running: boolean;
  endpoint: string;
  modelCount: number;
  version: string;
  error: string;
}

function ollamaBaseEndpoint(endpoint: string): string {
  return endpoint
    .replace(/\/api\/(chat|generate|tags|pull|delete|show)[^]*$/, "")
    .replace(/\/$/, "");
}

export async function checkOllamaHealth(endpoint: string): Promise<OllamaHealthResult> {
  const base = ollamaBaseEndpoint(endpoint);
  const tagsUrl = `${base}/api/tags`;
  try {
    const response = await globalThis.fetch(tagsUrl, {
      method: "GET",
      signal: AbortSignal.timeout(3000),
    });
    if (!response.ok) {
      return { running: false, endpoint: base, modelCount: 0, version: "", error: `HTTP ${response.status}` };
    }
    const data = await response.json() as { models?: unknown[]; version?: string };
    let version = typeof data.version === "string" ? data.version : "";
    if (!version) {
      try {
        const vr = await globalThis.fetch(`${base}/api/version`, { signal: AbortSignal.timeout(2000) });
        if (vr.ok) { const vd = await vr.json() as { version?: string }; version = vd.version ?? ""; }
      } catch { /* ignore */ }
    }
    return {
      running: true,
      endpoint: base,
      modelCount: Array.isArray(data.models) ? data.models.length : 0,
      version,
      error: "",
    };
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    return { running: false, endpoint: base, modelCount: 0, version: "", error: msg.includes("abort") ? "timeout — is Ollama running?" : msg };
  }
}

// ─── Streaming execution ───────────────────────────────────────────────────────

export interface OllamaStreamOptions {
  temperature?: number;
  numCtx?: number;
  repeatPenalty?: number;
  topP?: number;
  stopTokens?: string[];
}

export interface OllamaStreamResult {
  totalText: string;
  promptTokens: number;
  outputTokens: number;
}

/**
 * Stream an Ollama chat completion token-by-token.
 * onToken receives each fragment + the running total.
 * Supports AbortSignal for cancellation.
 * Automatically trims context and strips DeepSeek <think> blocks.
 */
export async function executeStreamingOllamaPrompt(
  endpoint: string,
  model: string,
  systemPrompt: string,
  userPrompt: string,
  onToken: (text: string, totalSoFar: string) => void,
  signal?: AbortSignal,
  opts: OllamaStreamOptions = {},
): Promise<OllamaStreamResult> {
  const base = ollamaBaseEndpoint(endpoint);
  const chatEndpoint = `${base}/api/chat`;

  const card = OLLAMA_MODEL_CATALOG.find(m => m.id === model);
  const numCtx = opts.numCtx ?? card?.numCtx ?? 32768;
  const contextWindow = card?.contextTokens ?? numCtx;
  const { systemPrompt: sys, userPrompt: usr } = trimToContextBudget(
    systemPrompt, userPrompt, Math.min(contextWindow, numCtx)
  );

  const response = await globalThis.fetch(chatEndpoint, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      model,
      messages: [
        { role: "system", content: sys },
        { role: "user", content: usr },
      ],
      stream: true,
      options: {
        temperature: opts.temperature ?? card?.temperature ?? 0.2,
        num_ctx: numCtx,
        repeat_penalty: opts.repeatPenalty ?? card?.repeatPenalty ?? 1.1,
        top_p: opts.topP ?? 0.9,
        ...(opts.stopTokens?.length ? { stop: opts.stopTokens } : {}),
      },
    }),
    signal,
  });

  if (!response.ok) {
    const errorText = await response.text().catch(() => "");
    throw new Error(`Ollama request failed: ${response.status} ${response.statusText}${errorText ? ` — ${errorText.slice(0, 240)}` : ""}`);
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error("Streaming response body is not readable.");

  const decoder = new TextDecoder();
  let totalText = "";
  let streamDone = false;
  let promptTokens = 0;
  let outputTokens = 0;
  let buffer = "";
  // Track whether we're inside a <think> block for DeepSeek R1
  let inThinkBlock = false;

  while (!streamDone) {
    const { value, done } = await reader.read();
    if (done) break;
    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() ?? "";
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      try {
        const parsed = JSON.parse(trimmed) as {
          message?: { content?: string };
          response?: string;
          done?: boolean;
          prompt_eval_count?: number;
          eval_count?: number;
        };
        const token = (parsed?.message?.content ?? parsed?.response ?? "") as string;
        if (token) {
          // Filter DeepSeek R1 thinking tokens
          const hasOpen = token.includes("<think>");
          const hasClose = token.includes("</think>");
          if (hasOpen) inThinkBlock = true;
          if (!inThinkBlock) {
            totalText += token;
            onToken(token, totalText);
          } else if (hasClose) {
            inThinkBlock = false;
            // Emit any text after the closing tag
            const afterThink = token.split("</think>").slice(1).join("</think>");
            if (afterThink) {
              totalText += afterThink;
              onToken(afterThink, totalText);
            }
          }
        }
        if (parsed?.done) {
          streamDone = true;
          promptTokens = parsed.prompt_eval_count ?? 0;
          outputTokens = parsed.eval_count ?? 0;
        }
      } catch { /* skip malformed NDJSON lines */ }
    }
  }

  return { totalText, promptTokens, outputTokens };
}
