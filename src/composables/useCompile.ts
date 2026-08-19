import { computed, watch } from "vue";
import { useDocumentsStore } from "../stores/documents";
import { useToasts } from "../lib/toasts";

/**
 * Wraps compile-failure state: debounced error toasting and the
 * "Copy error for support" helper.  The compile pipeline itself lives in
 * the store; this composable reacts to store.consecutiveCompileFailures and
 * surfaces user-facing feedback.
 */
export function useCompile() {
  const store = useDocumentsStore();
  const toasts = useToasts();

  /** Readable summary of the current compile state from the store. */
  const compileState = computed(() => ({
    consecutiveFailures: store.consecutiveCompileFailures,
    errorKind: store.lastCompileErrorKind,
    errorMessage: store.lastCompileErrorMessage,
  }));

  function copyPreviewErrorForSupport(): void {
    const info = JSON.stringify({
      errorKind: store.lastCompileErrorKind,
      errorMessage: store.lastCompileErrorMessage,
      documentTitle: store.activeDocument?.title,
      consecutiveFailures: store.consecutiveCompileFailures,
    }, null, 2);
    navigator.clipboard.writeText(info).catch(() => null);
  }

  // Item C: compile failure toast (warn after first failure, "Report this" hint after 3)
  watch(() => store.consecutiveCompileFailures, (count, prev) => {
    if (count === 0 || count <= (prev ?? 0)) return;
    const kindLabels: Record<string, string> = {
      'compile-failed': 'Compile failed',
      'backend-unavailable': 'Backend not responding',
      'transform-error': 'Transform engine error',
    };
    const kindLabel = kindLabels[store.lastCompileErrorKind] ?? 'Compile failed';
    if (count === 1) {
      toasts.push({ kind: "warning", title: "Preview unavailable", body: kindLabel });
    } else if (count === 3) {
      toasts.push({
        kind: "warning",
        title: "Preview unavailable (3rd failure)",
        body: `${kindLabel} -- repeated failures detected`,
        actionLabel: "Copy error for support",
        onAction: () => copyPreviewErrorForSupport(),
      });
    }
  });

  return {
    compileState,
    copyPreviewErrorForSupport,
  };
}
