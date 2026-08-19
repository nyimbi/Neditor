import { computed, ref, watch } from "vue";
import { useDocumentsStore } from "../stores/documents";
import type { ProviderError } from "../lib/providerRuntime";

/**
 * Wraps AI run lifecycle state (progress pill, retry, error display).
 * The actual AI functions (runDeepResearch, runAgentProviderRequest) live in
 * App.vue and call store.startAiRun/finishAiRun directly; this composable
 * owns the supplementary state and helpers that surround those calls.
 */
export function useAi() {
  const store = useDocumentsStore();

  // ── Elapsed seconds counter for the active AI run status pill ─────────────
  const aiElapsedSeconds = ref(0);
  let aiElapsedTimer: ReturnType<typeof setInterval> | undefined;
  watch(() => store.aiRun, (run) => {
    clearInterval(aiElapsedTimer);
    aiElapsedTimer = undefined;
    if (run) {
      aiElapsedSeconds.value = 0;
      aiElapsedTimer = setInterval(() => {
        aiElapsedSeconds.value = Math.floor((Date.now() - run.startedAt) / 1000);
      }, 1000);
    }
  });

  /** Last AI function invoked; used by the Retry button. */
  const lastAiRunFn = ref<(() => void) | null>(null);

  // ── Public readonly reactive ───────────────────────────────────────────────
  const hasActiveAi = computed(() => !!store.aiRun);
  const lastError = computed(() => store.aiLastError);

  // ── Helpers ───────────────────────────────────────────────────────────────
  function aiErrorKindLabel(kind: ProviderError["kind"]): string {
    return (
      { timeout: "Timeout", aborted: "Cancelled", network: "Network", http: "Provider", parse: "Parse" }[kind] ?? kind
    );
  }

  async function copyAiErrorDetails() {
    if (!store.aiLastError) return;
    const text = [
      `Kind: ${store.aiLastError.kind}`,
      `Message: ${store.aiLastError.message}`,
      store.aiLastError.hint ? `Hint: ${store.aiLastError.hint}` : "",
      store.aiLastError.status ? `Status: ${store.aiLastError.status}` : "",
    ].filter(Boolean).join("\n");
    try { await navigator.clipboard?.writeText(text); } catch { /* clipboard unavailable */ }
  }

  function retryLastAiRun() {
    store.aiLastError = null;
    lastAiRunFn.value?.();
  }

  function cancelActiveAi() {
    store.cancelAiRun();
  }

  /**
   * Convenience wrapper: sets lastAiRunFn for Retry support, starts an AI run,
   * awaits the caller's async fn (receiving the AbortController), then always
   * calls finishAiRun.  The caller is responsible for catching errors and
   * setting store.aiLastError / store.lastError as needed.
   */
  async function runAi(label: string, fn: (controller: AbortController) => Promise<void>): Promise<void> {
    lastAiRunFn.value = () => void runAi(label, fn);
    const controller = store.startAiRun(label);
    try {
      await fn(controller);
    } finally {
      store.finishAiRun();
    }
  }

  return {
    aiElapsedSeconds,
    lastAiRunFn,
    hasActiveAi,
    lastError,
    aiErrorKindLabel,
    copyAiErrorDetails,
    retryLastAiRun,
    cancelActiveAi,
    runAi,
  };
}
