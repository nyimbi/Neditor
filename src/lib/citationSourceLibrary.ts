export interface CitationSourceAuditItem {
  citation_key: string;
  title: string;
  url: string;
  snippet?: string;
  source?: string;
  path?: string;
  relative_path: string;
  sha256: string;
  bytes: number;
  downloaded_at?: string;
  media_type?: string;
  fit_score?: number;
  fit_label?: string;
  fit_reasons?: string[];
  file_exists?: boolean;
  hash_matches?: boolean;
  current_sha256?: string;
  current_bytes?: number;
}

export function citationSourceLibraryAuditMarkdown(sources: CitationSourceAuditItem[]) {
  if (!sources.length) return "## Source Library Audit\n\nNo saved citation sources are currently associated with this document.\n";
  const rows = sources.map((source) => {
    const fit = source.fit_score === undefined ? "not scored" : `${source.fit_score}/100 ${source.fit_label || ""}`.trim();
    const localPath = source.relative_path || source.path || "";
    const localStatus = source.file_exists === false ? `missing: ${localPath}` : source.hash_matches === false ? `modified: ${localPath}` : localPath;
    const hash = source.sha256 ? source.sha256.slice(0, 16) : "";
    const currentHash = source.hash_matches === false && source.current_sha256 ? `current sha256: ${source.current_sha256.slice(0, 16)}` : "";
    const currentBytes = source.hash_matches === false && source.current_bytes !== undefined ? `current bytes: ${source.current_bytes}` : "";
    const reviewNotes = [
      source.source ? `provider: ${source.source}` : "",
      source.media_type ? `type: ${source.media_type}` : "",
      source.file_exists === false ? "local file missing" : "",
      source.hash_matches === false ? "local file modified after download" : "",
      currentHash,
      currentBytes,
      source.fit_reasons?.length ? source.fit_reasons.join("; ") : "",
      source.downloaded_at ? `downloaded: ${source.downloaded_at}` : "",
    ].filter(Boolean).join("; ");
    return `| @${escapeTableCell(source.citation_key)} | ${escapeTableCell(source.title)} | ${escapeTableCell(fit)} | ${escapeTableCell(localStatus)} | ${escapeTableCell(hash)} | ${escapeTableCell(reviewNotes)} | ${escapeTableCell(source.url)} |`;
  });
  return [
    "## Source Library Audit",
    "",
    `Saved sources: ${sources.length}`,
    "",
    "| Citation key | Title | Fit | Local file | SHA-256 prefix | Review notes | URL |",
    "| --- | --- | --- | --- | --- | --- | --- |",
    ...rows,
    "",
  ].join("\n");
}

function escapeTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}
