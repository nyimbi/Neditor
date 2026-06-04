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

/**
 * Catalog last updated: June 2026.
 * Models marked recommended=true appear in the NEditor model picker UI.
 * Ordered: best-for-documents first.
 */
export const OLLAMA_MODEL_CATALOG: OllamaModelCard[] = [
  // ── Qwen 3.5 family — NEW BEST (256K context + vision + thinking + tools) ──
  {
    id: "qwen3.5:9b",
    label: "Qwen 3.5 9B",
    family: "Qwen",
    params: "9B",
    contextTokens: 262144,
    vramGb: 7.0,
    diskGb: 6.6,
    speed: "medium",
    quality: "excellent",
    tasks: ["business-writing", "long-document", "summarization", "reasoning"],
    recommended: true,
    badge: "Best overall",
    whyRecommended: "The new top pick. 256k context, vision, thinking mode, and tools in a 9B model. Handles entire contract stacks or multi-chapter reports. Thinking mode improves analytical accuracy. Multilingual (201 languages).",
    temperature: 0.2,
    numCtx: 65536,
    repeatPenalty: 1.1,
    tags: ["256k-context", "vision", "thinking", "tools", "multilingual"],
  },
  {
    id: "qwen3.5:4b",
    label: "Qwen 3.5 4B",
    family: "Qwen",
    params: "4B",
    contextTokens: 262144,
    vramGb: 4.0,
    diskGb: 3.4,
    speed: "fast",
    quality: "good",
    tasks: ["fast-edit", "long-document", "summarization"],
    recommended: true,
    badge: "Fast + vision",
    whyRecommended: "256k context and vision in just 4B parameters and 4GB VRAM. Processes entire long documents quickly. Perfect for iterative editing and summarisation when speed matters.",
    temperature: 0.2,
    numCtx: 65536,
    repeatPenalty: 1.1,
    tags: ["256k-context", "vision", "thinking", "fast", "low-vram"],
  },
  {
    id: "qwen3.5:2b",
    label: "Qwen 3.5 2B",
    family: "Qwen",
    params: "2B",
    contextTokens: 262144,
    vramGb: 3.0,
    diskGb: 2.7,
    speed: "fast",
    quality: "good",
    tasks: ["fast-edit", "summarization"],
    recommended: true,
    badge: "Tiny + 256K",
    whyRecommended: "256k context in 2.7GB. The smallest model that can read an entire long document in one shot. Use for quick paragraph rewrites, section summaries, or CPU-only machines with limited RAM.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.1,
    tags: ["256k-context", "vision", "tiny", "cpu-friendly"],
  },
  // ── Qwen 3 family — 256K context + thinking mode ──────────────────────────
  {
    id: "qwen3:4b",
    label: "Qwen 3 4B",
    family: "Qwen",
    params: "4B",
    contextTokens: 262144,
    vramGb: 3.0,
    diskGb: 2.5,
    speed: "fast",
    quality: "excellent",
    tasks: ["business-writing", "long-document", "reasoning", "fast-edit"],
    recommended: true,
    badge: "Rivals 72B",
    whyRecommended: "Exceptional quality for its size — benchmarks rival Qwen2.5-72B. 256k context, togglable thinking mode for harder tasks, and only 2.5GB. The best quality-per-gigabyte model available.",
    temperature: 0.2,
    numCtx: 65536,
    repeatPenalty: 1.05,
    tags: ["256k-context", "thinking", "tools", "exceptional-quality"],
  },
  {
    id: "qwen3:8b",
    label: "Qwen 3 8B",
    family: "Qwen",
    params: "8B",
    contextTokens: 40960,
    vramGb: 6.0,
    diskGb: 5.2,
    speed: "medium",
    quality: "excellent",
    tasks: ["business-writing", "reasoning", "summarization"],
    recommended: true,
    badge: "Best thinking",
    whyRecommended: "Thinking mode at 8B. When you need step-by-step reasoning for complex analytical tasks, citation verification, or structured extraction. Note: only 40k context — use qwen3:4b for very long documents.",
    temperature: 0.15,
    numCtx: 32768,
    repeatPenalty: 1.05,
    tags: ["thinking", "tools", "reasoning", "analytical"],
  },
  // ── Proven workhorses ─────────────────────────────────────────────────────
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
    whyRecommended: "Meta's battle-tested 8B. 128k context, excellent markdown structure, the most widely compatible model. Best fallback when you need guaranteed reliability over cutting-edge quality.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.08,
    tags: ["reliable", "128k-context", "compatible", "proven"],
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
    badge: "Best prose",
    whyRecommended: "Google's highest prose quality at ≤9B. Writing is notably fluent and coherent. Short context (8k) means it works best for individual sections, executive summaries, and targeted rewrites rather than full documents.",
    temperature: 0.15,
    numCtx: 8192,
    repeatPenalty: 1.0,
    tags: ["high-quality", "prose", "executive-writing", "google"],
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
    badge: "Deep reasoning",
    whyRecommended: "Dedicated reasoning model. Best for citation verification, QA, analytical summaries, and complex structured extraction. Emits a thinking trace before answering — NEditor strips it automatically.",
    temperature: 0.1,
    numCtx: 32768,
    repeatPenalty: 1.05,
    tags: ["reasoning", "analytical", "QA", "think-trace"],
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
    whyRecommended: "Code-specialised 7B. Use for Python/R/SQL transform blocks, calculation fences, and structured data generation. Better than general-purpose models for precise technical output.",
    temperature: 0.1,
    numCtx: 32768,
    repeatPenalty: 1.0,
    tags: ["code", "transforms", "sql", "python", "128k-context"],
  },
  // ── Legacy / compatibility ────────────────────────────────────────────────
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
    recommended: false,
    badge: "Previous gen",
    whyRecommended: "Superseded by Qwen 3.5 9B but still excellent. Use if you have it installed and don't want to re-download. 128k context, strong instruction following.",
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
  // ── Legacy — superseded but listed for users who have them installed ────────
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
    recommended: false,
    badge: "Superseded",
    whyRecommended: "Superseded by qwen3.5:4b which has 256K context + vision at similar VRAM. Keep if already installed.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.1,
    tags: ["128k-context", "google"],
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
    tasks: ["fast-edit", "summarization"],
    recommended: false,
    badge: "Superseded",
    whyRecommended: "Superseded by qwen3.5:2b and qwen3.5:4b which have 256K context. Keep if already installed.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.1,
    tags: ["128k-context", "microsoft"],
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
    badge: "Compatibility",
    whyRecommended: "Industry-standard 7B. Best OpenAI API compatibility. Use as fallback when other models are unavailable.",
    temperature: 0.2,
    numCtx: 32768,
    repeatPenalty: 1.1,
    tags: ["compatible", "reliable"],
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
    badge: "Superseded",
    whyRecommended: "Superseded by qwen3.5:2b which has 256K context + vision. Keep if already installed.",
    temperature: 0.2,
    numCtx: 16384,
    repeatPenalty: 1.1,
    tags: ["small", "llama"],
  },
];

/**
 * Best model IDs for each task type, ordered by recommendation.
 * Updated June 2026: Qwen3.5 family takes top slots for context-heavy tasks.
 */
export const RECOMMENDED_TASK_MODELS: Record<OllamaTaskType, string[]> = {
  "business-writing": ["qwen3.5:9b", "qwen3:4b", "llama3.1:8b"],
  "long-document":    ["qwen3.5:9b", "qwen3.5:4b", "qwen3:4b"],
  "summarization":    ["qwen3.5:9b", "gemma2:9b", "qwen3.5:4b"],
  "reasoning":        ["deepseek-r1:8b", "qwen3:8b", "qwen3.5:9b"],
  "code":             ["qwen2.5-coder:7b", "deepseek-r1:8b", "qwen3:8b"],
  "fast-edit":        ["qwen3.5:2b", "qwen3.5:4b", "qwen3:4b"],
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
