export interface BibliographyStubInput {
  key: string;
  title?: string | null;
  author?: string | null;
  issued?: string | null;
}

export function normalizeCitationKey(key: string): string {
  return key
    .trim()
    .replace(/^@+/, "")
    .replace(/\s+/g, "-")
    .replace(/[^A-Za-z0-9_:/.-]/g, "")
    .replace(/^-+|-+$/g, "");
}

export function citationReferenceSnippet(key: string, locator = ""): string {
  const normalized = normalizeCitationKey(key);
  if (!normalized) return "";
  const trimmedLocator = locator.trim().replace(/^\s*[,;]\s*/, "");
  return trimmedLocator ? `[@${normalized}, ${trimmedLocator}]` : `[@${normalized}]`;
}

export function bibliographyEntryStub(input: BibliographyStubInput): string {
  const key = normalizeCitationKey(input.key);
  if (!key) return "";
  const title = bibtexValue(input.title || "TODO: Add title");
  const author = bibtexValue(input.author || "TODO");
  const year = bibtexYear(input.issued);
  return `@misc{${key},\n  title = {${title}},\n  author = {${author}},\n  year = {${year}}\n}`;
}

export function bibliographyStubsForMissingKeys(keys: string[]): string {
  const stubs = uniqueNormalizedKeys(keys)
    .map((key) => bibliographyEntryStub({ key }))
    .filter(Boolean);
  if (!stubs.length) return "";
  return `\`\`\`bibtex\n${stubs.join("\n\n")}\n\`\`\`\n`;
}

function uniqueNormalizedKeys(keys: string[]): string[] {
  return Array.from(new Set(keys.map(normalizeCitationKey).filter(Boolean))).sort((left, right) => left.localeCompare(right));
}

function bibtexValue(value: string): string {
  return value
    .trim()
    .replace(/[{}]/g, "")
    .replace(/\s+/g, " ");
}

function bibtexYear(value?: string | null): string {
  const year = String(value || "").match(/\d{4}/)?.[0];
  return year || "TODO";
}
