import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";

// Boot timing — developer tool, zero production overhead.
// Read via `window.__neditor_boot` in DevTools console.
// Also emitted as a structured object for the Tauri dev panel.
declare global {
  interface Window {
    __neditor_boot: {
      bootStart: number;
      mountStart?: number;
      editorReady?: number;
      previewReady?: number;
      pluginsReady?: number;
    };
  }
}
window.__neditor_boot = { bootStart: performance.now() };

createApp(App).use(createPinia()).mount("#app");
