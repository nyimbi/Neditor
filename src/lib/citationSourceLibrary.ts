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

export interface CitationEvidenceClaim {
  kind: string;
  sourceLine: number;
  text: string;
  reason: string;
}

export interface ClaimSourceEvidenceMatch {
  claim: CitationEvidenceClaim;
  source: CitationSourceAuditItem;
  score: number;
  reasons: string[];
  recommendation: string;
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

export function matchClaimsToCitationSources(
  claims: CitationEvidenceClaim[],
  sources: CitationSourceAuditItem[],
  options: { perClaim?: number } = {},
): ClaimSourceEvidenceMatch[] {
  const perClaim = Math.max(1, Math.min(options.perClaim || 3, 5));
  const matches: ClaimSourceEvidenceMatch[] = [];
  for (const claim of claims.slice(0, 40)) {
    const terms = evidenceTerms(claim.text);
    const ranked = sources
      .map((source) => scoreClaimSourceMatch(claim, terms, source))
      .filter((match): match is ClaimSourceEvidenceMatch => Boolean(match))
      .sort((left, right) => right.score - left.score || left.source.title.localeCompare(right.source.title))
      .slice(0, perClaim);
    matches.push(...ranked);
  }
  return matches;
}

export function claimEvidenceMatrixMarkdown(
  claims: CitationEvidenceClaim[],
  sources: CitationSourceAuditItem[],
) {
  const matches = matchClaimsToCitationSources(claims, sources);
  if (!claims.length) return "## Claim Evidence Matrix\n\nNo current-document claims were detected for source review.\n";
  if (!sources.length) {
    return [
      "## Claim Evidence Matrix",
      "",
      "No saved source library is associated with this document yet. Download sources before completing claim evidence mapping.",
      "",
      "| Line | Claim | Needed evidence |",
      "| ---: | --- | --- |",
      ...claims.slice(0, 40).map((claim) => `| ${claim.sourceLine} | ${escapeTableCell(claim.text)} | ${escapeTableCell(claim.reason)} |`),
      "",
    ].join("\n");
  }
  const rows = matches.length
    ? matches.map((match) => {
        const citation = match.source.citation_key ? `[@${match.source.citation_key}]` : "-";
        const localPath = match.source.relative_path || match.source.path || "-";
        return `| ${match.claim.sourceLine} | ${escapeTableCell(match.claim.kind)} | ${escapeTableCell(match.claim.text)} | ${escapeTableCell(citation)} | ${match.score} | ${escapeTableCell(match.reasons.join("; "))} | ${escapeTableCell(localPath)} | ${escapeTableCell(match.recommendation)} |`;
      })
    : claims.slice(0, 40).map((claim) => `| ${claim.sourceLine} | ${escapeTableCell(claim.kind)} | ${escapeTableCell(claim.text)} | - | 0 | No saved source metadata appears to match. | - | Search or download evidence before review. |`);
  return [
    "## Claim Evidence Matrix",
    "",
    `Claims reviewed: ${claims.length}`,
    `Saved sources available: ${sources.length}`,
    `Suggested source matches: ${matches.length}`,
    "",
    "| Line | Claim type | Claim | Suggested citation | Match score | Match reasons | Local source | Review action |",
    "| ---: | --- | --- | --- | ---: | --- | --- | --- |",
    ...rows,
    "",
  ].join("\n");
}

function scoreClaimSourceMatch(
  claim: CitationEvidenceClaim,
  claimTerms: string[],
  source: CitationSourceAuditItem,
): ClaimSourceEvidenceMatch | null {
  if (!claimTerms.length) return null;
  const title = normalizeEvidenceText(source.title);
  const snippet = normalizeEvidenceText(source.snippet || "");
  const metadata = normalizeEvidenceText([
    source.url,
    source.relative_path,
    source.source,
    source.fit_label,
    ...(source.fit_reasons || []),
  ].filter(Boolean).join(" "));
  let score = 0;
  const reasons: string[] = [];
  const titleMatches = claimTerms.filter((term) => title.includes(term));
  const snippetMatches = claimTerms.filter((term) => snippet.includes(term));
  const metadataMatches = claimTerms.filter((term) => metadata.includes(term));
  if (titleMatches.length) {
    score += titleMatches.length * 10;
    reasons.push(`title: ${titleMatches.slice(0, 4).join(", ")}`);
  }
  if (snippetMatches.length) {
    score += snippetMatches.length * 7;
    reasons.push(`snippet: ${snippetMatches.slice(0, 4).join(", ")}`);
  }
  if (metadataMatches.length) {
    score += metadataMatches.length * 4;
    reasons.push(`metadata: ${metadataMatches.slice(0, 4).join(", ")}`);
  }
  if (source.fit_score !== undefined) score += Math.round(source.fit_score / 20);
  if (source.file_exists === false) {
    score = Math.max(0, score - 8);
    reasons.push("local file missing");
  } else if (source.hash_matches === false) {
    score = Math.max(0, score - 5);
    reasons.push("local file modified");
  }
  if (score < 8) return null;
  return {
    claim,
    source,
    score,
    reasons,
    recommendation: source.file_exists === false
      ? "Re-download or replace the missing source before citing this claim."
      : source.hash_matches === false
        ? "Verify the modified local file before accepting this citation."
        : `Review the source and add [@${source.citation_key}] if it supports the claim.`,
  };
}

function evidenceTerms(value: string) {
  const stopWords = new Set([
    "about",
    "after",
    "also",
    "before",
    "because",
    "could",
    "from",
    "have",
    "into",
    "more",
    "should",
    "that",
    "their",
    "there",
    "these",
    "this",
    "through",
    "will",
    "with",
    "would",
  ]);
  const terms = normalizeEvidenceText(value)
    .split(/\s+/)
    .filter((term) => term.length > 2 && !stopWords.has(term) && !/^\d+$/.test(term));
  return Array.from(new Set(terms)).slice(0, 12);
}

function normalizeEvidenceText(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, " ").replace(/\s+/g, " ").trim();
}

function escapeTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}
