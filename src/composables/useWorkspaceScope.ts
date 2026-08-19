import { computed } from "vue";
import { useDocumentsStore } from "../stores/documents";

/**
 * Tracks workspace roots, recent folders, and pinned tabs.
 *
 * The store owns one workspace root at a time (`store.workspaceRoot`).
 * `roots` adapts it to a readonly array for the public API.
 * `pinned` is derived from `store.documents[].pinned`.
 */
export function useWorkspaceScope() {
  const store = useDocumentsStore();

  /** Current workspace roots as a readonly list (0 or 1 items). */
  const roots = computed<readonly string[]>(() =>
    store.workspaceRoot ? [store.workspaceRoot] : [],
  );

  /** Open a folder as the active workspace root. */
  async function addRoot(path: string): Promise<void> {
    await store.openFolder(path);
  }

  /**
   * Remove a workspace root.  Clears the current root if it matches `path`
   * and removes it from recents; no-op if the root does not match.
   */
  function removeRoot(path: string): void {
    if (store.workspaceRoot === path) {
      store.$patch({
        workspaceRoot: null,
        recentFolders: store.recentFolders.filter((f) => f !== path),
      });
    }
  }

  /** Recent workspace folder paths (most-recent first). */
  const recents = computed<readonly string[]>(() => store.recentFolders);

  /** IDs of currently pinned document tabs. */
  const pinned = computed<readonly string[]>(() =>
    store.documents.filter((d) => d.pinned).map((d) => d.id),
  );

  /** Pin a document tab by id. */
  function pin(tabId: string): void {
    store.setPinned(tabId, true);
  }

  /** Unpin a document tab by id. */
  function unpin(tabId: string): void {
    store.setPinned(tabId, false);
  }

  return {
    roots,
    addRoot,
    removeRoot,
    recents,
    pinned,
    pin,
    unpin,
  };
}
