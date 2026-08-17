/**
 * Central provider fetch helper for all AI HTTP calls.
 *
 * Design guarantees:
 * - Never throws; all outcomes are discriminated via ProviderFetchResult.
 * - Applies a configurable timeout (default 60 s); the timer is ALWAYS cleared
 *   in the finally block so no timer ever leaks after normal completion.
 * - Supports caller-supplied AbortSignal; both abort sources are combined so
 *   either one firing is sufficient to cancel the request.
 * - `data` on success is the raw response body text; callers parse JSON as needed.
 */

export type ProviderFetchOpts = {
  /** Wall-clock timeout in milliseconds before the request is aborted. Default: 60 000. */
  timeoutMs?: number;
  /** Optional caller-managed signal (e.g., from a store AbortController). */
  signal?: AbortSignal;
  /** Phase notification callback for progress UI. */
  onProgress?: (phase: "connect" | "stream" | "done") => void;
  /** Human-readable label used in error messages and progress UI. */
  label: string;
};

export type ProviderError = {
  kind: "timeout" | "aborted" | "network" | "http" | "parse";
  message: string;
  /** HTTP status code, present for kind="http". */
  status?: number;
  retriable: boolean;
  hint?: string;
};

export type ProviderFetchResult<T = string> =
  | { ok: true; data: T; ms: number }
  | { ok: false; error: ProviderError };

const DEFAULT_TIMEOUT_MS = 60_000;

/** Combine N AbortSignals: the returned signal fires when any source fires. */
function combineSignals(signals: Array<AbortSignal | undefined>): {
  signal: AbortSignal;
  cleanup: () => void;
} {
  const controller = new AbortController();
  const cleanups: Array<() => void> = [];

  for (const sig of signals) {
    if (!sig) continue;
    if (sig.aborted) {
      controller.abort(sig.reason);
      return { signal: controller.signal, cleanup: () => {} };
    }
    const handler = () => {
      if (!controller.signal.aborted) controller.abort(sig.reason);
    };
    sig.addEventListener("abort", handler, { once: true });
    cleanups.push(() => sig.removeEventListener("abort", handler));
  }

  return {
    signal: controller.signal,
    cleanup: () => cleanups.forEach((f) => f()),
  };
}

/**
 * Perform a single AI provider HTTP request with timeout and abort support.
 *
 * @param url   Full endpoint URL (already model-substituted by caller).
 * @param init  Fetch init without `signal`; signal is managed here.
 * @param opts  Timeout, progress, label, and optional caller signal.
 * @returns     A discriminated union — check `.ok` before using `.data`.
 */
export async function providerFetch<T = string>(
  url: string,
  init: RequestInit,
  opts: ProviderFetchOpts,
): Promise<ProviderFetchResult<T>> {
  const timeoutMs = opts.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  const timeoutController = new AbortController();

  // Timer fires → abort. Cleared in finally so it never leaks.
  let timer: ReturnType<typeof setTimeout> | undefined = setTimeout(
    () => timeoutController.abort("timeout"),
    timeoutMs,
  );

  const { signal: combined, cleanup } = combineSignals([
    opts.signal,
    timeoutController.signal,
  ]);

  const t0 = Date.now();
  try {
    opts.onProgress?.("connect");

    let response: Response;
    try {
      response = await fetch(url, { ...init, signal: combined });
    } catch (err) {
      if (err instanceof Error && err.name === "AbortError") {
        if (timeoutController.signal.aborted) {
          return {
            ok: false,
            error: {
              kind: "timeout",
              message: `"${opts.label}" timed out after ${timeoutMs / 1000}s`,
              retriable: true,
              hint: `Provider took longer than ${timeoutMs / 1000}s to respond. Check network or increase the AI timeout in Settings.`,
            },
          };
        }
        return {
          ok: false,
          error: {
            kind: "aborted",
            message: `"${opts.label}" was cancelled`,
            retriable: false,
            hint: "The request was cancelled.",
          },
        };
      }
      return {
        ok: false,
        error: {
          kind: "network",
          message: err instanceof Error ? err.message : String(err),
          retriable: true,
          hint: "Check the provider endpoint URL in Settings and verify the provider is reachable.",
        },
      };
    }

    opts.onProgress?.("stream");

    let rawText: string;
    try {
      rawText = await response.text();
    } catch (err) {
      return {
        ok: false,
        error: {
          kind: "network",
          message: err instanceof Error ? err.message : String(err),
          retriable: true,
          hint: "Provider connection dropped while reading the response.",
        },
      };
    }

    if (!response.ok) {
      const retriable = response.status >= 500 || response.status === 429;
      return {
        ok: false,
        error: {
          kind: "http",
          message: `${response.status} ${response.statusText}${rawText ? ` — ${rawText.slice(0, 200)}` : ""}`,
          status: response.status,
          retriable,
          hint: httpErrorHint(response.status),
        },
      };
    }

    opts.onProgress?.("done");
    // data is the raw response text; callers parse JSON as needed.
    return { ok: true, data: rawText as unknown as T, ms: Date.now() - t0 };
  } finally {
    clearTimeout(timer);
    timer = undefined;
    cleanup();
  }
}

function httpErrorHint(status: number): string {
  if (status === 401 || status === 403) return "Provider rejected the API key. Check the key in Settings.";
  if (status === 429) return "Provider rate limit reached. Wait a moment and retry.";
  if (status === 404) return "Provider endpoint not found. Check the endpoint URL in Settings.";
  if (status >= 500) return "Provider returned a server error. Check the provider status page or retry later.";
  return "Check the provider endpoint URL and API key in Settings.";
}

/**
 * Thrown by executeAiProvider* functions when providerFetch returns ok:false.
 * Callers can instanceof-check to extract the structured ProviderError.
 */
export class ProviderFetchError extends Error {
  readonly providerError: ProviderError;
  constructor(providerError: ProviderError) {
    super(providerError.message);
    this.name = "ProviderFetchError";
    this.providerError = providerError;
  }
}
