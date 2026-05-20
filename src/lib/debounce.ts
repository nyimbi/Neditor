export const PREVIEW_DEBOUNCE_MS = 80;

export interface DebouncedTextCommitScheduler {
  setTimeout(callback: () => void, delayMs: number): number;
  clearTimeout(handle: number): void;
}

export interface DebouncedTextCommit {
  schedule(text: string): void;
  flush(text: string): void;
  cancel(): void;
}

export function createDebouncedTextCommit(
  commit: (text: string) => void,
  scheduler: DebouncedTextCommitScheduler,
  delayMs = PREVIEW_DEBOUNCE_MS,
): DebouncedTextCommit {
  let currentHandle: number | null = null;
  let pendingText: string | null = null;

  function clearPending() {
    if (currentHandle === null) return;
    scheduler.clearTimeout(currentHandle);
    currentHandle = null;
  }

  return {
    schedule(text) {
      clearPending();
      pendingText = text;
      const handle = scheduler.setTimeout(() => {
        if (currentHandle !== handle) return;
        const textToCommit = pendingText;
        pendingText = null;
        clearPending();
        if (textToCommit !== null) commit(textToCommit);
      }, delayMs);
      currentHandle = handle;
    },

    flush(text) {
      clearPending();
      pendingText = null;
      commit(text);
    },

    cancel() {
      clearPending();
      pendingText = null;
    },
  };
}
