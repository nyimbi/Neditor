/**
 * Vite bundle entry — exports App component + stores so snapshot tests
 * can interact with Pinia state from outside the bundle while keeping
 * the Tauri aliases applied during compilation.
 */
export { default } from "../../src/App.vue";
export { useDocumentsStore } from "../../src/stores/documents";
export { useToasts } from "../../src/lib/toasts";
