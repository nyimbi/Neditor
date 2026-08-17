/** Mock for @tauri-apps/api/path */

export async function homeDir(): Promise<string> { return "/home/test"; }
export async function appDataDir(): Promise<string> { return "/tmp/test-app-data"; }
export async function appConfigDir(): Promise<string> { return "/tmp/test-app-config"; }
export async function documentDir(): Promise<string> { return "/home/test/Documents"; }
export async function desktopDir(): Promise<string> { return "/home/test/Desktop"; }
export async function downloadDir(): Promise<string> { return "/home/test/Downloads"; }
export async function join(...paths: string[]): Promise<string> { return paths.join("/"); }
export async function resolve(...paths: string[]): Promise<string> { return paths.join("/"); }
export async function basename(path: string, _ext?: string): Promise<string> {
  return path.split("/").pop() ?? path;
}
export async function dirname(path: string): Promise<string> {
  return path.split("/").slice(0, -1).join("/");
}
export async function extname(path: string): Promise<string> {
  const parts = path.split(".");
  return parts.length > 1 ? `.${parts.pop()}` : "";
}
export async function isAbsolute(path: string): Promise<boolean> {
  return path.startsWith("/");
}
