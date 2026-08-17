/**
 * Type stubs for the Vite-built App snapshot bundle.
 * Generated at build time by vite build --config vite.snapshot.config.ts.
 */
import type { DefineComponent } from "vue";

declare const App: DefineComponent<Record<string, never>, Record<string, never>, unknown>;
export default App;

export declare function useDocumentsStore(): {
  $patch(state: Record<string, unknown>): void;
  bootCritical: () => Promise<void>;
  bootBackground: () => Promise<void>;
  [key: string]: unknown;
};

export declare function useToasts(): {
  push(toast: { kind: string; title: string; body?: string }): string;
  dismiss(id: string): void;
  visible: Array<{ id: string; kind: string; title: string; body?: string }>;
  [key: string]: unknown;
};
