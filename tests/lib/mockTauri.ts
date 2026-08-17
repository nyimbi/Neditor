/**
 * Tauri IPC mock surface — spec-required entry point.
 *
 * Covers the invoke commands that fire during App.vue's initial paint:
 *   read_workspace_settings, list_preview_themes, run_external_transform,
 *   compile_document, and the full boot sequence commands.
 *
 * Individual package re-exports let tests import this file for ad-hoc
 * programmatic stubs during snapshot creation.
 */

export { invoke, convertFileSrc } from "./mocks/tauri-api-core.js";
export { listen, once, emit } from "./mocks/tauri-api-event.js";
export { homeDir, appDataDir, join } from "./mocks/tauri-api-path.js";
export { getCurrentWindow } from "./mocks/tauri-api-window.js";
export { confirm, open, save } from "./mocks/tauri-plugin-dialog.js";
export { openUrl } from "./mocks/tauri-plugin-opener.js";
export { Store } from "./mocks/tauri-plugin-store.js";
export { watch as watchFs } from "./mocks/tauri-plugin-fs.js";
export { check as checkUpdate } from "./mocks/tauri-plugin-updater.js";
