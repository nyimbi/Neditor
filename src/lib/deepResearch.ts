import type { AiProviderProfileId } from "./aiProviderPackages.js";
import { normalizeCitationKey } from "./bibliographyManager.js";

export type DeepResearchSearchProvider = "duckduckgo" | "searxng" | "tavily";

export interface DeepResearchSource {
  title: string;
  url: string;
  snippet: string;
  source: string;
  fitScore?: number;
  fitLabel?: string;
  fitReasons?: string[];
}

export interface DeepResearchSourceFit {
  score: number;
  label: string;
  reasons: string[];
}

export interface DeepResearchSettings {
  topic: string;
  documentType: string;
  audience: string;
  searchProvider: DeepResearchSearchProvider;
  searxngUrl: string;
  tavilyApiKey: string;
  iterations: number;
  resultsPerIteration: number;
  targetPages: number;
  providerProfileId: AiProviderProfileId;
  model: string;
  endpoint: string;
  keyEnv: string;
}

export interface DeepResearchIteration {
  index: number;
  query: string;
  results: DeepResearchSource[];
  summary: string;
  gaps: string[];
}

export interface DeepResearchRun {
  topic: string;
  documentType: string;
  audience: string;
  searchProvider: DeepResearchSearchProvider;
  iterations: DeepResearchIteration[];
  draftMarkdown: string;
}

export interface DeepResearchBibliographySource {
  citation_key?: string;
  title: string;
  url: string;
  snippet?: string;
  source?: string;
  relative_path?: string;
  sha256?: string;
  downloaded_at?: string;
}

export interface DeepResearchDocumentOptions {
  generatedAt?: string;
  savedSourceCount?: number;
  sourceLibraryAuditMarkdown?: string;
  bibliographySources?: DeepResearchBibliographySource[];
}

const WORDS_PER_MARKDOWN_PAGE = 500;
const EXPANSION_PAGES_PER_PASS = 5;
const MAX_EXPANSION_PASSES = 40;

export function normalizeDeepResearchSettings(input: Partial<DeepResearchSettings>): DeepResearchSettings {
  return {
    topic: normalizeText(input.topic, 240),
    documentType: normalizeText(input.documentType, 80) || "research brief",
    audience: normalizeText(input.audience, 120) || "business readers",
    searchProvider: normalizeSearchProvider(input.searchProvider),
    searxngUrl: normalizeText(input.searxngUrl, 300) || "http://127.0.0.1:8080",
    tavilyApiKey: normalizeText(input.tavilyApiKey, 300),
    iterations: clampInteger(input.iterations, 1, 5, 3),
    resultsPerIteration: clampInteger(input.resultsPerIteration, 3, 12, 5),
    targetPages: clampInteger(input.targetPages, 1, 200, 5),
    providerProfileId: input.providerProfileId || "ollama-local",
    model: normalizeText(input.model, 120) || "llama3.1",
    endpoint: normalizeText(input.endpoint, 300) || "http://127.0.0.1:11434/api/chat",
    keyEnv: normalizeText(input.keyEnv, 80) || "NEDITOR_AI_API_KEY",
  };
}

export function deepResearchQueryPrompt(settings: DeepResearchSettings, iterations: DeepResearchIteration[]) {
  const previous = iterations
    .map((iteration) => `Iteration ${iteration.index}: ${iteration.query}\nFindings: ${iteration.summary}\nGaps: ${iteration.gaps.join("; ")}`)
    .join("\n\n");
  return [
    `Research topic: ${settings.topic}`,
    `Document type: ${settings.documentType}`,
    `Audience: ${settings.audience}`,
    `Target length: about ${settings.targetPages} page${settings.targetPages === 1 ? "" : "s"} (${targetWordCount(settings)} words).`,
    "",
    previous ? `Previous research:\n${previous}` : "No prior research has been run yet.",
    "",
    "Return one focused web search query only. It should find concrete, citable sources for the next section of the document.",
  ].join("\n");
}

export function deepResearchReflectionPrompt(settings: DeepResearchSettings, query: string, results: DeepResearchSource[], iterations: DeepResearchIteration[]) {
  return [
    `Topic: ${settings.topic}`,
    `Document type: ${settings.documentType}`,
    `Audience: ${settings.audience}`,
    `Search query: ${query}`,
    "",
    "Search results:",
    ...results.map((result, index) => {
      const fit = result.fitScore === undefined
        ? ""
        : `\nFit: ${result.fitScore}/100 ${result.fitLabel || ""}${result.fitReasons?.length ? ` - ${result.fitReasons.join("; ")}` : ""}`;
      return `${index + 1}. ${result.title}\nURL: ${result.url}\nSource: ${result.source}${fit}\nSnippet: ${result.snippet || "(no snippet)"}`;
    }),
    "",
    iterations.length ? `Prior iteration summaries:\n${iterations.map((iteration) => `- ${iteration.summary}`).join("\n")}` : "",
    "",
    "Return concise Markdown with exactly these headings:",
    "### Summary",
    "### Knowledge Gaps",
    "Use bullets under Knowledge Gaps. Do not invent source facts beyond the snippets and titles.",
  ].join("\n");
}

export function deepResearchDraftPrompt(settings: DeepResearchSettings, iterations: DeepResearchIteration[]) {
  return [
    `Create a ${settings.documentType} for ${settings.audience}.`,
    `Topic: ${settings.topic}`,
    `Target length: about ${settings.targetPages} page${settings.targetPages === 1 ? "" : "s"} (${targetWordCount(settings)} words).`,
    "",
    "Use this research log only as grounding. Keep claims cautious when snippets are limited.",
    "",
    formatDeepResearchLog(iterations),
    "",
    "Return a polished Markdown document with:",
    "- A clear title.",
    "- An executive summary.",
    "- A structured body with headings.",
    "- A findings or recommendations section when useful.",
    "- A limitations and open questions section.",
    "- A Sources section with Markdown links for every source used.",
    "- Citation TODOs for claims that need source-document verification.",
    "- Enough section depth, examples, implications, and caveats to approach the target length without padding.",
  ].join("\n");
}

export function deepResearchExpansionPrompt(
  settings: DeepResearchSettings,
  draftMarkdown: string,
  iterations: DeepResearchIteration[],
  currentPages = estimateMarkdownPages(draftMarkdown),
  pass = 1,
  maxPasses = expansionPassBudget(settings),
) {
  const remainingPages = pageShortfall(settings, draftMarkdown);
  const passTargetPages = Math.min(remainingPages, EXPANSION_PAGES_PER_PASS);
  return [
    `Expand this ${settings.documentType} toward ${settings.targetPages} pages (${targetWordCount(settings)} words) for ${settings.audience}.`,
    `Topic: ${settings.topic}`,
    `Current estimate: ${currentPages} page${currentPages === 1 ? "" : "s"}.`,
    `Expansion pass: ${pass}/${maxPasses}.`,
    `This pass should add about ${passTargetPages} page${passTargetPages === 1 ? "" : "s"} of new, substantive material unless the target is already met.`,
    "",
    "Use the research log to add substantive sections, examples, implications, constraints, and review TODOs. Do not pad with repetition.",
    "Prefer new useful sections, tables, assumptions, decision implications, risks, implementation detail, and clearly marked source-verification TODOs over repeated prose.",
    "",
    "Research log:",
    formatDeepResearchLog(iterations),
    "",
    "Current draft:",
    draftMarkdown,
    "",
    "Return the full expanded Markdown document, including Sources and Review TODOs.",
  ].join("\n");
}

export function deepResearchQualityPrompt(
  settings: DeepResearchSettings,
  draftMarkdown: string,
  iterations: DeepResearchIteration[],
  currentPages = estimateMarkdownPages(draftMarkdown),
) {
  return [
    `Quality-assure and humanize this ${settings.documentType} for ${settings.audience}.`,
    `Topic: ${settings.topic}`,
    `Target length: ${settings.targetPages} page${settings.targetPages === 1 ? "" : "s"}; current estimate: ${currentPages} page${currentPages === 1 ? "" : "s"}.`,
    "",
    "Mandatory review actions:",
    "- Preserve the full Markdown document, not just review notes.",
    "- Keep or improve the current length unless removing duplicated padding.",
    "- Remove generic AI phrasing, overclaiming, repetition, and filler.",
    "- Add source-verification TODOs for claims not directly grounded in the research log.",
    "- Make executive summary, section headings, limitations, and recommendations concrete.",
    "- Add a final 'Quality Assurance & Review Handoff' section with evidence checks, open gaps, human-review tasks, and distribution cautions.",
    "",
    "Research log:",
    formatDeepResearchLog(iterations),
    "",
    "Draft to improve:",
    draftMarkdown,
    "",
    "Return the complete review-ready Markdown document only.",
  ].join("\n");
}

export function deepResearchQualityAuditMarkdown(
  settings: DeepResearchSettings,
  draftMarkdown: string,
  iterations: DeepResearchIteration[],
) {
  const currentPages = estimateMarkdownPages(draftMarkdown);
  const sources = uniqueSources(iterations);
  const openGaps = iterations.flatMap((iteration) => iteration.gaps).filter(Boolean);
  const citationTodos = (draftMarkdown.match(/citation TODO|\[@TODO|TODO citation/gi) || []).length;
  const targetStatus = currentPages >= settings.targetPages
    ? `Reached target: about ${currentPages}/${settings.targetPages} pages.`
    : `Below target: about ${currentPages}/${settings.targetPages} pages; provider expansion should be reviewed before release.`;
  return [
    "## Quality Assurance & Review Handoff",
    "",
    `- Length status: ${targetStatus}`,
    `- Source inventory: ${sources.length} unique source candidate${sources.length === 1 ? "" : "s"} across ${iterations.length} research iteration${iterations.length === 1 ? "" : "s"}.`,
    `- Citation TODO count: ${citationTodos}.`,
    `- Knowledge gaps carried forward: ${openGaps.length}.`,
    "",
    "### Evidence Checks",
    "",
    "- [ ] Open every saved source document and verify the claims that depend on it.",
    "- [ ] Resolve citation TODOs before external distribution.",
    "- [ ] Confirm dates, figures, named organizations, legal/regulatory claims, and recommendations against source documents.",
    "- [ ] Preserve source-library audit evidence with the final review packet.",
    "",
    "### Humanization Checks",
    "",
    "- [ ] Remove generic AI phrasing, duplicated transitions, and unsupported certainty.",
    "- [ ] Confirm the voice fits the stated audience and document type.",
    "- [ ] Tighten headings so each section makes a distinct contribution.",
    "",
    "### Open Knowledge Gaps",
    "",
    ...(openGaps.length ? openGaps.slice(0, 12).map((gap) => `- ${gap}`) : ["- No explicit knowledge gaps were recorded by the research loop."]),
    "",
  ].join("\n");
}

export function deepResearchDocumentMarkdown(
  settings: DeepResearchSettings,
  draftMarkdown: string,
  iterations: DeepResearchIteration[],
  options: DeepResearchDocumentOptions = {},
) {
  const generatedAt = options.generatedAt || new Date().toISOString();
  const sources = uniqueSources(iterations);
  const body = draftMarkdown.trim();
  const withFrontMatter = hasFrontMatter(body) ? body : [
    "---",
    `title: ${yamlString(settings.topic || "Deep Research Draft")}`,
    `documentType: ${yamlString(settings.documentType)}`,
    `audience: ${yamlString(settings.audience)}`,
    "status: draft",
    "owner: TODO owner",
    "releaseTarget: review package",
    `deepResearchTopic: ${yamlString(settings.topic)}`,
    `deepResearchTargetPages: ${settings.targetPages}`,
    `deepResearchIterations: ${iterations.length}`,
    `deepResearchSourceCandidates: ${sources.length}`,
    `deepResearchSavedSources: ${Math.max(0, options.savedSourceCount || 0)}`,
    `aiProviderProfile: ${yamlString(settings.providerProfileId)}`,
    `aiProviderModel: ${yamlString(settings.model)}`,
    `generatedAt: ${yamlString(generatedAt)}`,
    "---",
    "",
    body,
  ].join("\n");
  const withProvenance = hasDeepResearchProvenance(withFrontMatter)
    ? withFrontMatter
    : insertAfterFrontMatter(withFrontMatter, deepResearchProvenanceBlock(settings, generatedAt));
  const bibliography = deepResearchBibliographyMarkdown(iterations, options.bibliographySources);
  const withBibliographyEntries = !bibliography || hasBibliographyEntries(withProvenance)
    ? withProvenance
    : `${withProvenance.trim()}\n\n${bibliography}`;
  const withBibliographyMarker = hasBibliographyMarker(withBibliographyEntries)
    ? withBibliographyEntries
    : `${withBibliographyEntries.trim()}\n\n## Bibliography\n\n[BIBLIOGRAPHY]\n`;
  const withEvidenceLog = hasDeepResearchEvidenceLog(withBibliographyMarker)
    ? withBibliographyMarker
    : `${withBibliographyMarker.trim()}\n\n## Deep Research Evidence Log\n\n${formatDeepResearchLog(iterations)}\n`;
  const sourceLibraryAudit = normalizeSourceLibraryAudit(options.sourceLibraryAuditMarkdown);
  return !sourceLibraryAudit || hasSourceLibraryAudit(withEvidenceLog)
    ? withEvidenceLog
    : `${withEvidenceLog.trim()}\n\n${sourceLibraryAudit}\n`;
}

export function deepResearchBibliographyMarkdown(
  iterations: DeepResearchIteration[],
  savedSources: DeepResearchBibliographySource[] = [],
) {
  const bibliographySources = bibliographySourcesForDocument(iterations, savedSources);
  if (!bibliographySources.length) return "";
  const usedKeys = new Set<string>();
  const entries = bibliographySources.map((source, index) => {
    const id = uniqueBibliographyKey(source, index, usedKeys);
    const entry: Record<string, unknown> = {
      id,
      type: "webpage",
      title: source.title || id,
      URL: source.url,
    };
    const accessed = cslDateFromIso(source.downloaded_at);
    if (accessed) entry.accessed = accessed;
    const note = deepResearchBibliographyNote(source);
    if (note) entry.note = note;
    return entry;
  });
  return ["```bibliography", JSON.stringify(entries, null, 2), "```", ""].join("\n");
}

export function fallbackDeepResearchQuery(settings: DeepResearchSettings, iterations: DeepResearchIteration[]) {
  const suffixes = ["overview evidence", "recent data sources", "risks limitations", "case studies", "implementation guidance"];
  const suffix = suffixes[iterations.length % suffixes.length];
  return `${settings.topic} ${settings.documentType} ${suffix}`;
}

export function rankDeepResearchSources(sources: DeepResearchSource[], query: string) {
  return sources
    .map((source, index) => {
      const fit = assessDeepResearchSource(source, query);
      return {
        ...source,
        fitScore: fit.score,
        fitLabel: fit.label,
        fitReasons: fit.reasons,
        __index: index,
      };
    })
    .sort((left, right) => (right.fitScore || 0) - (left.fitScore || 0) || left.__index - right.__index)
    .map(({ __index: _index, ...source }) => source);
}

export function assessDeepResearchSource(source: DeepResearchSource, query: string): DeepResearchSourceFit {
  const queryTokens = significantWords(query);
  const titleTokens = new Set(significantWords(source.title));
  const snippetTokens = new Set(significantWords(source.snippet));
  let score = 20;
  const reasons: string[] = [];

  const titleMatches = queryTokens.filter((token) => titleTokens.has(token)).length;
  const snippetMatches = queryTokens.filter((token) => snippetTokens.has(token)).length;
  if (titleMatches) {
    score += Math.min(24, titleMatches * 8);
    reasons.push(`${titleMatches} query term${titleMatches === 1 ? "" : "s"} in title`);
  }
  if (snippetMatches) {
    score += Math.min(18, snippetMatches * 4);
    reasons.push(`${snippetMatches} query term${snippetMatches === 1 ? "" : "s"} in snippet`);
  }

  const lowerText = `${source.title} ${source.snippet}`.toLowerCase();
  const normalizedQuery = query.trim().toLowerCase();
  if (normalizedQuery.length >= 12 && lowerText.includes(normalizedQuery)) {
    score += 10;
    reasons.push("exact query phrase match");
  }

  const urlAssessment = assessSourceUrl(source.url);
  score += urlAssessment.scoreDelta;
  reasons.push(...urlAssessment.reasons);

  if (!source.snippet.trim()) {
    score -= 5;
    reasons.push("no snippet for quick evidence review");
  }
  if (/\b(?:blog|forum|reddit|x\.com|twitter|facebook|linkedin|medium\.com|substack\.com)\b/i.test(source.url)) {
    score -= 10;
    reasons.push("publication context may need extra verification");
  }
  if (/\b(?:search|login|signup|account)\b/i.test(source.url)) {
    score -= 8;
    reasons.push("URL may not point directly to source evidence");
  }

  const bounded = Math.max(0, Math.min(100, Math.round(score)));
  return {
    score: bounded,
    label: sourceFitLabel(bounded),
    reasons: reasons.length ? reasons.slice(0, 4) : ["general web source; inspect before citing"],
  };
}

export function parseReflection(markdown: string) {
  const summary = sectionText(markdown, "Summary") || markdown.trim();
  const gapsText = sectionText(markdown, "Knowledge Gaps");
  const gaps = gapsText
    .split("\n")
    .map((line) => line.replace(/^[-*]\s*/, "").trim())
    .filter(Boolean)
    .slice(0, 8);
  return { summary, gaps };
}

export function fallbackResearchDraft(settings: DeepResearchSettings, iterations: DeepResearchIteration[]) {
  const sourceLines = uniqueSources(iterations).map((source, index) => `${index + 1}. [${source.title}](${source.url}) - ${source.snippet || source.source}`);
  return [
    `# ${settings.topic}`,
    "",
    "## Executive Summary",
    "",
    `This ${settings.documentType} summarizes initial web research for ${settings.audience}.`,
    `Target length: ${settings.targetPages} page${settings.targetPages === 1 ? "" : "s"}; expand with verified source notes before final approval.`,
    "",
    "## Research Findings",
    "",
    ...iterations.map((iteration) => `### ${iteration.query}\n\n${iteration.summary}\n`),
    "## Open Questions",
    "",
    ...iterations.flatMap((iteration) => iteration.gaps.map((gap) => `- ${gap}`)),
    "",
    "## Sources",
    "",
    ...(sourceLines.length ? sourceLines : ["- No sources were returned by the selected search provider."]),
    "",
    "## Review TODOs",
    "",
    "- [ ] Download and inspect source documents before approving factual claims.",
    "- [ ] Replace broad statements with verified citations.",
  ].join("\n");
}

export function estimateMarkdownPages(markdown: string) {
  const words = markdown
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/[#>*_[\]()`|:-]/g, " ")
    .split(/\s+/)
    .filter(Boolean).length;
  return Math.max(1, Math.ceil(words / WORDS_PER_MARKDOWN_PAGE));
}

export function targetWordCount(settings: DeepResearchSettings) {
  return settings.targetPages * WORDS_PER_MARKDOWN_PAGE;
}

export function expansionPassBudget(settings: DeepResearchSettings) {
  return Math.min(MAX_EXPANSION_PASSES, Math.max(1, Math.ceil(settings.targetPages / EXPANSION_PAGES_PER_PASS)));
}

export function pageShortfall(settings: DeepResearchSettings, markdown: string) {
  return Math.max(0, settings.targetPages - estimateMarkdownPages(markdown));
}

export function formatDeepResearchLog(iterations: DeepResearchIteration[]) {
  if (!iterations.length) return "No research iterations yet.";
  return iterations
    .map((iteration) =>
      [
        `## Iteration ${iteration.index}: ${iteration.query}`,
        "",
        iteration.summary,
        "",
        "Knowledge gaps:",
        ...(iteration.gaps.length ? iteration.gaps.map((gap) => `- ${gap}`) : ["- No gaps recorded."]),
        "",
        "Sources:",
        ...iteration.results.map((result) => {
          const fit = result.fitScore === undefined ? "" : ` (${result.fitScore}/100 ${result.fitLabel || "fit"})`;
          return `- [${result.title}](${result.url})${fit} - ${result.snippet || result.source}`;
        }),
      ].join("\n"),
    )
    .join("\n\n");
}

function assessSourceUrl(url: string) {
  const reasons: string[] = [];
  let scoreDelta = 0;
  try {
    const parsed = new URL(url);
    const host = parsed.hostname.toLowerCase();
    const path = parsed.pathname.toLowerCase();
    if (host.endsWith(".gov") || host.includes(".gov.")) {
      scoreDelta += 15;
      reasons.push("government source domain");
    } else if (host.endsWith(".edu") || host.includes(".edu.")) {
      scoreDelta += 12;
      reasons.push("academic source domain");
    } else if (host.endsWith(".org") || host.includes(".org.")) {
      scoreDelta += 6;
      reasons.push("organization source domain");
    }
    if (/\.(pdf)(?:$|[?#])/.test(path)) {
      scoreDelta += 12;
      reasons.push("downloadable PDF source");
    } else if (/\.(docx?|rtf)(?:$|[?#])/.test(path)) {
      scoreDelta += 8;
      reasons.push("downloadable document source");
    } else if (/\.(csv|xlsx?|json)(?:$|[?#])/.test(path)) {
      scoreDelta += 8;
      reasons.push("data file source");
    } else if (/\.(html?|md|txt)(?:$|[?#])/.test(path)) {
      scoreDelta += 3;
      reasons.push("reviewable text source");
    }
  } catch {
    scoreDelta -= 10;
    reasons.push("invalid URL");
  }
  return { scoreDelta, reasons };
}

function sourceFitLabel(score: number) {
  if (score >= 75) return "strong";
  if (score >= 55) return "good";
  if (score >= 35) return "review";
  return "weak";
}

function significantWords(value: string) {
  const stopWords = new Set([
    "about",
    "after",
    "also",
    "and",
    "from",
    "into",
    "with",
    "that",
    "this",
    "the",
    "for",
    "are",
    "was",
    "were",
    "will",
    "report",
    "brief",
    "study",
    "source",
  ]);
  return value
    .toLowerCase()
    .split(/[^a-z0-9]+/)
    .filter((word) => word.length >= 4 && !stopWords.has(word));
}

function uniqueSources(iterations: DeepResearchIteration[]) {
  const byUrl = new Map<string, DeepResearchSource>();
  for (const iteration of iterations) {
    for (const result of iteration.results) {
      if (!byUrl.has(result.url)) byUrl.set(result.url, result);
    }
  }
  return Array.from(byUrl.values());
}

function bibliographySourcesForDocument(
  iterations: DeepResearchIteration[],
  savedSources: DeepResearchBibliographySource[],
) {
  const byUrl = new Map<string, DeepResearchBibliographySource>();
  for (const source of savedSources) {
    const url = source.url.trim();
    if (url) byUrl.set(url, source);
  }
  for (const source of uniqueSources(iterations)) {
    const url = source.url.trim();
    if (!url || byUrl.has(url)) continue;
    byUrl.set(url, {
      title: source.title,
      url,
      snippet: source.snippet,
      source: source.source,
    });
  }
  return Array.from(byUrl.values());
}

function uniqueBibliographyKey(
  source: DeepResearchBibliographySource,
  index: number,
  usedKeys: Set<string>,
) {
  const preferred = normalizeCitationKey(source.citation_key || bibliographyKeyBase(source) || `source-${index + 1}`);
  const base = preferred || `source-${index + 1}`;
  let candidate = base;
  let suffix = 2;
  while (usedKeys.has(candidate)) {
    candidate = `${base}-${suffix}`;
    suffix += 1;
  }
  usedKeys.add(candidate);
  return candidate;
}

function bibliographyKeyBase(source: DeepResearchBibliographySource) {
  const fromTitle = source.title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
  if (fromTitle) return fromTitle;
  try {
    const parsed = new URL(source.url);
    return parsed.hostname.replace(/^www\./, "").replace(/[^a-z0-9]+/gi, "-").toLowerCase();
  } catch {
    return "";
  }
}

function cslDateFromIso(value: string | undefined) {
  if (!value) return undefined;
  const date = new Date(value);
  if (!Number.isFinite(date.getTime())) return undefined;
  return { "date-parts": [[date.getUTCFullYear(), date.getUTCMonth() + 1, date.getUTCDate()]] };
}

function deepResearchBibliographyNote(source: DeepResearchBibliographySource) {
  const parts = [
    "Deep Research source",
    source.source ? `provider: ${source.source}` : "",
    source.relative_path ? `Downloaded source: ${source.relative_path}` : "",
    source.sha256 ? `sha256 ${source.sha256}` : "",
    source.snippet ? `Snippet: ${source.snippet}` : "",
  ].filter(Boolean);
  return parts.join(" | ");
}

function sectionText(markdown: string, heading: string) {
  const pattern = new RegExp(`^#{2,3}\\s+${heading}\\s*$`, "im");
  const match = markdown.match(pattern);
  if (!match || match.index === undefined) return "";
  const start = match.index + match[0].length;
  const rest = markdown.slice(start);
  const next = rest.search(/^#{2,3}\s+/m);
  return (next >= 0 ? rest.slice(0, next) : rest).trim();
}

function normalizeSearchProvider(value: unknown): DeepResearchSearchProvider {
  if (value === "searxng" || value === "tavily") return value;
  return "duckduckgo";
}

function normalizeText(value: unknown, maxLength: number) {
  return typeof value === "string" ? value.trim().slice(0, maxLength) : "";
}

function hasFrontMatter(markdown: string) {
  return /^---\r?\n[\s\S]*?\r?\n---(?:\r?\n|$)/.test(markdown);
}

function hasDeepResearchProvenance(markdown: string) {
  return /```ai-source[\s\S]*provider:\s*NEditor Deep Research/i.test(markdown);
}

function hasDeepResearchEvidenceLog(markdown: string) {
  return /^##\s+Deep Research Evidence Log\b/im.test(markdown);
}

function hasBibliographyEntries(markdown: string) {
  return /^(```|~~~)(?:bibliography|bibtex|hayagriva)\b/im.test(markdown);
}

function hasBibliographyMarker(markdown: string) {
  return /\[BIBLIOGRAPHY\]/i.test(markdown);
}

function hasSourceLibraryAudit(markdown: string) {
  return /^##\s+Source Library Audit\b/im.test(markdown);
}

function normalizeSourceLibraryAudit(markdown: string | undefined) {
  const trimmed = (markdown || "").trim();
  if (!trimmed || /No saved citation sources/i.test(trimmed)) return "";
  return /^##\s+Source Library Audit\b/im.test(trimmed)
    ? trimmed
    : `## Source Library Audit\n\n${trimmed}`;
}

function insertAfterFrontMatter(markdown: string, block: string) {
  const match = markdown.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/);
  if (!match) return `${block}\n\n${markdown}`;
  const start = markdown.slice(0, match[0].length).trimEnd();
  const rest = markdown.slice(match[0].length).trimStart();
  return `${start}\n\n${block}\n\n${rest}`;
}

function deepResearchProvenanceBlock(settings: DeepResearchSettings, generatedAt: string) {
  return [
    "```ai-source",
    "provider: NEditor Deep Research",
    `model: ${markerValue(settings.model || settings.providerProfileId)}`,
    `date: ${markerValue(generatedAt)}`,
    `promptSummary: ${markerValue(`Generated ${settings.documentType} for ${settings.audience} on ${settings.topic || "research topic"}`)}`,
    "reviewedBy: ",
    "reviewedAt: ",
    "status: needs-review",
    "```",
  ].join("\n");
}

function yamlString(value: string) {
  return JSON.stringify(value || "");
}

function markerValue(value: string) {
  return value.replace(/\r?\n/g, " ").replace(/\|/g, "/").trim();
}

function clampInteger(value: unknown, min: number, max: number, fallback: number) {
  const number = typeof value === "number" && Number.isFinite(value) ? Math.round(value) : fallback;
  return Math.min(max, Math.max(min, number));
}
