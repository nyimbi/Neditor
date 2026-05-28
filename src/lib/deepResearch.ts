import type { AiProviderProfileId } from "./aiProviderPackages.js";

export type DeepResearchSearchProvider = "duckduckgo" | "searxng" | "tavily";

export interface DeepResearchSource {
  title: string;
  url: string;
  snippet: string;
  source: string;
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
    ...results.map((result, index) => `${index + 1}. ${result.title}\nURL: ${result.url}\nSource: ${result.source}\nSnippet: ${result.snippet || "(no snippet)"}`),
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

export function fallbackDeepResearchQuery(settings: DeepResearchSettings, iterations: DeepResearchIteration[]) {
  const suffixes = ["overview evidence", "recent data sources", "risks limitations", "case studies", "implementation guidance"];
  const suffix = suffixes[iterations.length % suffixes.length];
  return `${settings.topic} ${settings.documentType} ${suffix}`;
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
        ...iteration.results.map((result) => `- [${result.title}](${result.url}) - ${result.snippet || result.source}`),
      ].join("\n"),
    )
    .join("\n\n");
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

function clampInteger(value: unknown, min: number, max: number, fallback: number) {
  const number = typeof value === "number" && Number.isFinite(value) ? Math.round(value) : fallback;
  return Math.min(max, Math.max(min, number));
}
