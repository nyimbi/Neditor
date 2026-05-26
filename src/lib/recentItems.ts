export function rememberRecentItem(items: string[], item: string | null | undefined, limit: number): string[] {
  const normalized = (item || "").trim();
  if (!normalized) return items;
  return [normalized, ...items.filter((candidate) => candidate !== normalized)].slice(0, limit);
}

export function forgetRecentItem(items: string[], item: string | null | undefined): string[] {
  const normalized = (item || "").trim();
  if (!normalized) return items;
  return items.filter((candidate) => candidate !== normalized);
}
