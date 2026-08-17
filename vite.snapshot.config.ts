/**
 * Vite config for the snapshot test bundle.
 *
 * Builds tests/lib/AppTestEntry.ts → .tmp-tests/AppBundle.js (ESM).
 * vue, pinia, and heavy editor deps are externalized so the test runner
 * and the bundle share exactly one Pinia instance (enabling store patching).
 * All @tauri-apps/* imports are aliased to local mock stubs.
 */

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

const ROOT = __dirname;

export default defineConfig({
  plugins: [vue()],
  build: {
    lib: {
      entry: resolve(ROOT, "tests/lib/AppTestEntry.js"),
      name: "AppBundle",
      fileName: "AppBundle",
      formats: ["es"],
    },
    outDir: resolve(ROOT, ".tmp-tests"),
    emptyOutDir: false,
    sourcemap: false,
    minify: false,
    rollupOptions: {
      external: [
        "vue",
        "pinia",
        /^@codemirror\//,
        "katex",
      ],
    },
  },
  resolve: {
    alias: {
      "@tauri-apps/api/core":          resolve(ROOT, "tests/lib/mocks/tauri-api-core.ts"),
      "@tauri-apps/api/event":         resolve(ROOT, "tests/lib/mocks/tauri-api-event.ts"),
      "@tauri-apps/api/path":          resolve(ROOT, "tests/lib/mocks/tauri-api-path.ts"),
      "@tauri-apps/api/window":        resolve(ROOT, "tests/lib/mocks/tauri-api-window.ts"),
      "@tauri-apps/plugin-dialog":     resolve(ROOT, "tests/lib/mocks/tauri-plugin-dialog.ts"),
      "@tauri-apps/plugin-opener":     resolve(ROOT, "tests/lib/mocks/tauri-plugin-opener.ts"),
      "@tauri-apps/plugin-store":      resolve(ROOT, "tests/lib/mocks/tauri-plugin-store.ts"),
      "@tauri-apps/plugin-fs":         resolve(ROOT, "tests/lib/mocks/tauri-plugin-fs.ts"),
      "@tauri-apps/plugin-updater":    resolve(ROOT, "tests/lib/mocks/tauri-plugin-updater.ts"),
      "@tauri-apps/plugin-shell":      resolve(ROOT, "tests/lib/mocks/tauri-plugin-opener.ts"),
      "@tauri-apps/plugin-window-state": resolve(ROOT, "tests/lib/mocks/tauri-plugin-window-state.ts"),
    },
  },
  define: {
    // Silence Tauri environment checks that test for window.__TAURI_INTERNALS__
    "window.__TAURI_INTERNALS__": "undefined",
  },
});
