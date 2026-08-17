/** Mock for @tauri-apps/plugin-fs */

export type UnwatchFn = () => void;
export interface WatchEvent {
  type: unknown;
  paths: string[];
  attrs: unknown;
}

export async function watch(
  _paths: string | string[],
  _callback: (event: WatchEvent) => void,
  _options?: unknown,
): Promise<UnwatchFn> {
  return () => {};
}

export async function unwatch(_rid: unknown): Promise<void> {}

export async function readTextFile(_path: string, _options?: unknown): Promise<string> {
  return "";
}

export async function writeTextFile(
  _path: string,
  _contents: string,
  _options?: unknown,
): Promise<void> {}

export async function exists(_path: string, _options?: unknown): Promise<boolean> {
  return false;
}

export async function mkdir(_path: string, _options?: unknown): Promise<void> {}

export async function readDir(_path: string, _options?: unknown): Promise<unknown[]> {
  return [];
}

export async function remove(_path: string, _options?: unknown): Promise<void> {}

export async function copyFile(
  _fromPath: string,
  _toPath: string,
  _options?: unknown,
): Promise<void> {}

export async function rename(
  _fromPath: string,
  _toPath: string,
  _options?: unknown,
): Promise<void> {}

export async function stat(_path: string, _options?: unknown): Promise<unknown> {
  return { isFile: true, isDirectory: false, isSymlink: false, size: 0, mtime: null, atime: null, ctime: null };
}
