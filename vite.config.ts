import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes("node_modules")) return undefined;
          // CodeMirror — large but only needed once editor mounts.
          if (id.includes("/@codemirror/") || id.includes("/lezer/") || id.includes("/@lezer/")) return "vendor-cm";
          // KaTeX — math rendering, deferred after editor.
          if (id.includes("/katex/")) return "vendor-katex";
          // Tauri plugin bindings — small but isolated.
          if (id.includes("/@tauri-apps/")) return "tauri";
          // Vue + Pinia — tiny, needed immediately for app shell.
          if (id.includes("/vue/") || id.includes("/pinia/")) return "vue-vendor";
          // Everything else (markdown-it, yaml, etc.).
          return "vendor";
        },
      },
    },
  },
}));
