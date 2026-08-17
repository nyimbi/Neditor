/** Mock for @tauri-apps/plugin-updater */

export interface Update {
  version: string;
  currentVersion: string;
  body?: string | null;
  date?: string | null;
  downloadAndInstall(_onChunk?: (progress: { event: string; data?: unknown }) => void): Promise<void>;
}

export async function check(_options?: unknown): Promise<Update | null> {
  return null;
}
