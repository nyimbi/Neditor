/** Mock for @tauri-apps/plugin-window-state */

export function restoreStateCurrent(_flags?: unknown): Promise<void> {
  return Promise.resolve();
}

export function saveWindowState(_flags?: unknown): Promise<void> {
  return Promise.resolve();
}

export function StateFlags(): number { return 0; }
