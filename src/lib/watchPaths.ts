export type WatchPathRole = "root" | "include";

interface WatchPathRoleInput {
  path: string;
  role?: string | null;
}

export function normalizeWatchPath(path?: string | null): string {
  const normalized = (path || "").replace(/\\/g, "/").replace(/\/+$/, "");
  return /^[a-z]:/i.test(normalized) ? normalized.toLowerCase() : normalized;
}

export function sameWatchPath(left?: string | null, right?: string | null): boolean {
  return normalizeWatchPath(left) === normalizeWatchPath(right);
}

export function buildWatchedPathRoles(files: WatchPathRoleInput[]): Record<string, WatchPathRole> {
  return files.reduce<Record<string, WatchPathRole>>((roles, file) => {
    const role = file.role === "root" ? "root" : "include";
    roles[file.path] = role;
    roles[normalizeWatchPath(file.path)] = role;
    return roles;
  }, {});
}
