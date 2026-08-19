import { open, save } from "@tauri-apps/plugin-dialog";
import { useDocumentsStore } from "../stores/documents";
import { useToasts } from "../lib/toasts";

/**
 * New / open / save / save-as / rename / duplicate / revert document flows.
 *
 * @param flush   - must be `flushEditorTextToStore` from App.vue; syncs the
 *                  CodeMirror buffer to the store before any save IPC call.
 * @param smokeMd - `desktopWorkflowSmokeMarkdownPath` from App.vue; injected
 *                  so the canonical function definition stays in App.vue where
 *                  desktop-smoke integration tests scan for it.
 * @param smokeMdNamed - `desktopWorkflowSmokeNamedMarkdownPath` from App.vue;
 *                  same rationale as smokeMd.
 */
export function useFileOps(
  flush: () => void,
  smokeMd: () => Promise<string | null>,
  smokeMdNamed: (fileStem: string) => Promise<string | null>,
) {
  const store = useDocumentsStore();
  const toasts = useToasts();

  async function openDocument(): Promise<void> {
    const smokePath = await smokeMd();
    if (smokePath) {
      await store.openPath(smokePath);
      return;
    }
    const selected = await open({
      multiple: false,
      filters: [{ name: "Markdown", extensions: ["md", "markdown", "mdown", "txt"] }],
    });
    if (typeof selected === "string") await store.openPath(selected);
  }

  async function openFolder(): Promise<void> {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (typeof selected === "string") await store.openFolder(selected);
  }

  async function saveDocument(): Promise<void> {
    const active = store.activeDocument;
    if (!active.path) {
      await saveAs();
      return;
    }
    flush();
    await store.saveActive();
    if (active.path) {
      toasts.push({ kind: "success", title: "Saved", body: active.path });
    }
  }

  async function saveAs(): Promise<void> {
    const active = store.activeDocument;
    const path =
      (await smokeMd()) ||
      (await save({
        filters: [{ name: "Markdown", extensions: ["md"] }],
        defaultPath: active.title.endsWith(".md") ? active.title : `${active.title}.md`,
      }));
    if (path) {
      flush();
      await store.saveActive(path);
      toasts.push({ kind: "success", title: "Saved", body: path });
    }
  }

  async function rename(newName?: string): Promise<void> {
    const active = store.activeDocument;
    const path =
      (await smokeMdNamed("native-workflow-renamed")) ||
      (await save({
        filters: [{ name: "Markdown", extensions: ["md"] }],
        defaultPath: newName ?? (active.title.endsWith(".md") ? active.title : `${active.title}.md`),
      }));
    if (path) await store.renameActive(path);
  }

  async function duplicate(): Promise<void> {
    const active = store.activeDocument;
    const path =
      (await smokeMdNamed("native-workflow-duplicate")) ||
      (await save({
        filters: [{ name: "Markdown", extensions: ["md"] }],
        defaultPath: `${active.title.replace(/\.[^.]+$/, "")} copy.md`,
      }));
    if (path) await store.duplicateActive(path);
  }

  async function revert(): Promise<void> {
    const active = store.activeDocument;
    if (!active.path) return;
    await store.openPath(active.path);
  }

  return {
    openDocument,
    openFolder,
    saveDocument,
    saveAs,
    rename,
    duplicate,
    revert,
  };
}
