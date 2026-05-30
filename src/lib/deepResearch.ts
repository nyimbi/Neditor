import type { AiProviderProfileId } from "./aiProviderPackages.js";
import { normalizeCitationKey } from "./bibliographyManager.js";
import { countCitationTodoMarkers } from "./citationTodoPatterns.js";

export type DeepResearchSearchProvider = "duckduckgo" | "searxng" | "tavily" | "local-library";

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

export interface DeepResearchSourceQualityReviewItem {
  iteration: number;
  query: string;
  title: string;
  url: string;
  source: string;
  fitScore: number;
  fitLabel: string;
  fitReasons: string[];
  reviewAction: string;
}

export interface DeepResearchEvidenceConflict {
  id: string;
  topic: string;
  family: string;
  severity: "review" | "risk";
  signals: string[];
  sources: Array<{
    title: string;
    url: string;
    source: string;
    stance: string;
    evidence: string;
  }>;
  recommendation: string;
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
  owner?: string;
  preparedBy?: string;
  organization?: string;
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

export function deepResearchDraftPrompt(
  settings: DeepResearchSettings,
  iterations: DeepResearchIteration[],
  savedSources: DeepResearchBibliographySource[] = [],
) {
  const citationGuidance = deepResearchCitationGuidanceMarkdown(iterations, savedSources);
  return [
    `Create a ${settings.documentType} for ${settings.audience}.`,
    `Topic: ${settings.topic}`,
    `Target length: about ${settings.targetPages} page${settings.targetPages === 1 ? "" : "s"} (${targetWordCount(settings)} words).`,
    "",
    "Use this research log only as grounding. Keep claims cautious when snippets are limited.",
    "Use the citation keys below for source-grounded claims, for example `[@source-key]`. Do not invent citation keys.",
    "Mark unsupported or source-ambiguous claims as `Citation TODO:` instead of pretending they are verified.",
    "",
    citationGuidance,
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
    "- Inline citation markers using the provided `[@key]` values where source support is clear.",
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
  savedSources: DeepResearchBibliographySource[] = [],
) {
  const remainingPages = pageShortfall(settings, draftMarkdown);
  const passTargetPages = Math.min(remainingPages, EXPANSION_PAGES_PER_PASS);
  const citationGuidance = deepResearchCitationGuidanceMarkdown(iterations, savedSources);
  return [
    `Expand this ${settings.documentType} toward ${settings.targetPages} pages (${targetWordCount(settings)} words) for ${settings.audience}.`,
    `Topic: ${settings.topic}`,
    `Current estimate: ${currentPages} page${currentPages === 1 ? "" : "s"}.`,
    `Expansion pass: ${pass}/${maxPasses}.`,
    `This pass should add about ${passTargetPages} page${passTargetPages === 1 ? "" : "s"} of new, substantive material unless the target is already met.`,
    "",
    "Use the research log to add substantive sections, examples, implications, constraints, and review TODOs. Do not pad with repetition.",
    "Prefer new useful sections, tables, assumptions, decision implications, risks, implementation detail, and clearly marked source-verification TODOs over repeated prose.",
    "Use only the listed citation keys for source-grounded claims; mark unsupported claims as `Citation TODO:`.",
    "",
    citationGuidance,
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
  savedSources: DeepResearchBibliographySource[] = [],
) {
  const citationGuidance = deepResearchCitationGuidanceMarkdown(iterations, savedSources);
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
    "- Preserve valid `[@key]` citation markers and add listed keys where the research log directly supports a claim.",
    "- Do not invent citation keys; use `Citation TODO:` for claims that need source-document verification.",
    "- Make executive summary, section headings, limitations, and recommendations concrete.",
    "- Add a final 'Quality Assurance & Review Handoff' section with evidence checks, open gaps, human-review tasks, and distribution cautions.",
    "",
    citationGuidance,
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
  const citationTodos = countCitationTodoMarkers(draftMarkdown);
  const conflicts = detectDeepResearchEvidenceConflicts(iterations);
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
    `- Evidence conflict review: ${conflicts.length ? `${conflicts.length} possible conflict${conflicts.length === 1 ? "" : "s"} require human review.` : "No obvious opposing source signals detected in snippets."}`,
    "",
    "### Evidence Checks",
    "",
    "- [ ] Open every saved source document and verify the claims that depend on it.",
    "- [ ] Resolve citation TODOs before external distribution.",
    "- [ ] Confirm dates, figures, named organizations, legal/regulatory claims, and recommendations against source documents.",
    "- [ ] Resolve possible source conflicts before presenting contested findings as settled.",
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
    ...(conflicts.length ? ["### Evidence Conflicts", "", ...conflicts.map((conflict) => `- ${conflict.id}: ${conflict.topic} - ${conflict.signals.join(" versus ")}. ${conflict.recommendation}`), ""] : []),
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
  const owner = normalizeText(options.owner, 160) || "review owner";
  const preparedBy = normalizeText(options.preparedBy, 160);
  const organization = normalizeText(options.organization, 160);
  const withFrontMatter = hasFrontMatter(body) ? body : [
    "---",
    `title: ${yamlString(settings.topic || "Deep Research Draft")}`,
    `documentType: ${yamlString(settings.documentType)}`,
    `audience: ${yamlString(settings.audience)}`,
    "status: draft",
    `owner: ${yamlString(owner)}`,
    ...(preparedBy ? [`preparedBy: ${yamlString(preparedBy)}`] : []),
    ...(organization ? [`organization: ${yamlString(organization)}`] : []),
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
  return appendDeepResearchEvidenceSections(withProvenance, iterations, options);
}

export function deepResearchReviewPackageMarkdown(
  settings: DeepResearchSettings,
  draftMarkdown: string,
  iterations: DeepResearchIteration[],
  options: DeepResearchDocumentOptions = {},
) {
  const generatedAt = options.generatedAt || new Date().toISOString();
  const body = draftMarkdown.trim();
  const withProvenance = hasDeepResearchProvenance(body)
    ? body
    : `${deepResearchProvenanceBlock(settings, generatedAt)}\n\n${body}`;
  return appendDeepResearchEvidenceSections(withProvenance, iterations, options);
}

export function deepResearchAuditPacketMarkdown(
  settings: DeepResearchSettings,
  draftMarkdown: string,
  iterations: DeepResearchIteration[],
  options: DeepResearchDocumentOptions = {},
) {
  const generatedAt = options.generatedAt || new Date().toISOString();
  const pages = estimateMarkdownPages(draftMarkdown || "");
  const citationTodos = countCitationTodoMarkers(draftMarkdown || "");
  const uniqueSourceCount = uniqueSources(iterations).length;
  const sourceQualityRows = deepResearchSourceQualityRows(iterations);
  const searchRows = iterations.length
    ? iterations.map((iteration) => [
        String(iteration.index),
        iteration.query,
        `${iteration.results.length} result${iteration.results.length === 1 ? "" : "s"}`,
        iteration.gaps.join("; ") || "No explicit gap recorded",
      ])
    : [["-", "No search iterations recorded", "0 results", "Run Deep Research before finalizing the audit packet"]];
  const conflictReview = deepResearchEvidenceConflictMarkdown(iterations).trim();
  const citationIndex = deepResearchCitationIndexMarkdown(iterations, options.bibliographySources).trim();
  const bibliography = deepResearchBibliographyMarkdown(iterations, options.bibliographySources).trim();
  const sourceLibraryAudit = normalizeSourceLibraryAudit(options.sourceLibraryAuditMarkdown)
    || "## Source Library Audit\n\nNo saved citation source audit is available for this packet.\n";
  return [
    "# Deep Research Audit Packet",
    "",
    `Generated: ${generatedAt}`,
    "",
    "## Run Configuration",
    "",
    "| Field | Value |",
    "| --- | --- |",
    `| Topic | ${tableCell(settings.topic || "Untitled research")} |`,
    `| Document type | ${tableCell(settings.documentType)} |`,
    `| Audience | ${tableCell(settings.audience)} |`,
    `| Search provider | ${tableCell(settings.searchProvider)} |`,
    `| AI provider profile | ${tableCell(settings.providerProfileId)} |`,
    `| Model | ${tableCell(settings.model)} |`,
    `| Target pages | ${settings.targetPages} |`,
    `| Draft pages estimated | ${pages} |`,
    `| Target words | ${targetWordCount(settings)} |`,
    `| Search iterations | ${iterations.length} |`,
    `| Unique source candidates | ${uniqueSourceCount} |`,
    `| Saved source documents | ${Math.max(0, options.savedSourceCount || 0)} |`,
    `| Citation TODOs in draft | ${citationTodos} |`,
    "",
    "## Search Query Log",
    "",
    "| Iteration | Query | Results | Gaps carried forward |",
    "| --- | --- | --- | --- |",
    ...searchRows.map((row) => `| ${row.map(tableCell).join(" | ")} |`),
    "",
    "## Source Quality Summary",
    "",
    "| Fit band | Sources | Review meaning |",
    "| --- | ---: | --- |",
    ...sourceQualityRows.map((row) => `| ${row.label} | ${row.count} | ${tableCell(row.guidance)} |`),
    "",
    conflictReview,
    "",
    "## Bibliography State",
    "",
    bibliography || "No bibliography records could be generated from the current research iterations.",
    "",
    "## Source Citation Index State",
    "",
    citationIndex || "No source citation index records are available yet.",
    "",
    sourceLibraryAudit.trim(),
    "",
    "## Reviewer Sign-Off Checklist",
    "",
    "- [ ] Every material draft claim has an inline citation or citation TODO.",
    "- [ ] Every cited source appears in the bibliography and source citation index.",
    "- [ ] Every downloaded source file exists and matches its recorded hash.",
    "- [ ] Every evidence conflict is resolved, caveated, or assigned to a reviewer.",
    "- [ ] Search query coverage is sufficient for the requested document scope and page count.",
    "- [ ] Final export includes the bibliography, source audit, and review handoff.",
    "",
  ].join("\n");
}

function appendDeepResearchEvidenceSections(
  markdown: string,
  iterations: DeepResearchIteration[],
  options: DeepResearchDocumentOptions,
) {
  const bibliographyRecords = deepResearchBibliographyRecords(iterations, options.bibliographySources);
  const missingBibliography = deepResearchBibliographyMarkdownFromRecords(
    missingDeepResearchBibliographyRecords(markdown, bibliographyRecords),
  );
  const withBibliographyEntries = !missingBibliography
    ? markdown
    : `${markdown.trim()}\n\n${missingBibliography}`;
  const missingCitationIndex = deepResearchCitationIndexMarkdownFromRecords(
    missingDeepResearchCitationIndexRecords(withBibliographyEntries, bibliographyRecords),
  );
  const citationIndexHeading = hasDeepResearchCitationIndex(withBibliographyEntries)
    ? "## Source Citation Index Addendum"
    : "## Source Citation Index";
  const withCitationIndex = !missingCitationIndex
    ? withBibliographyEntries
    : `${withBibliographyEntries.trim()}\n\n${citationIndexHeading}\n\n${missingCitationIndex}`;
  const withBibliographyMarker = hasBibliographyMarker(withCitationIndex)
    ? withCitationIndex
    : `${withCitationIndex.trim()}\n\n## Bibliography\n\n[BIBLIOGRAPHY]\n`;
  const missingEvidenceIterations = missingDeepResearchEvidenceIterations(withBibliographyMarker, iterations);
  const missingEvidenceLog = missingEvidenceIterations.length
    ? formatDeepResearchLog(missingEvidenceIterations)
    : "";
  const evidenceLogHeading = hasDeepResearchEvidenceLog(withBibliographyMarker)
    ? "## Deep Research Evidence Log Addendum"
    : "## Deep Research Evidence Log";
  const withEvidenceLog = !missingEvidenceLog
    ? withBibliographyMarker
    : `${withBibliographyMarker.trim()}\n\n${evidenceLogHeading}\n\n${missingEvidenceLog}\n`;
  const conflictReview = deepResearchConflictReviewCompletionMarkdown(withEvidenceLog, iterations);
  const withConflictReview = conflictReview
    ? `${withEvidenceLog.trim()}\n\n${conflictReview}\n`
    : withEvidenceLog;
  const sourceLibraryAudit = normalizeSourceLibraryAudit(options.sourceLibraryAuditMarkdown);
  const sourceLibraryAuditCompletion = sourceLibraryAuditCompletionMarkdown(withConflictReview, sourceLibraryAudit);
  return !sourceLibraryAuditCompletion
    ? withConflictReview
    : `${withConflictReview.trim()}\n\n${sourceLibraryAuditCompletion}\n`;
}

export function deepResearchBibliographyMarkdown(
  iterations: DeepResearchIteration[],
  savedSources: DeepResearchBibliographySource[] = [],
) {
  return deepResearchBibliographyMarkdownFromRecords(deepResearchBibliographyRecords(iterations, savedSources));
}

export function deepResearchCitationIndexMarkdown(
  iterations: DeepResearchIteration[],
  savedSources: DeepResearchBibliographySource[] = [],
) {
  return deepResearchCitationIndexMarkdownFromRecords(deepResearchBibliographyRecords(iterations, savedSources));
}

function deepResearchBibliographyMarkdownFromRecords(records: DeepResearchBibliographyRecord[]) {
  const entries = records.map((record) => record.entry);
  if (!entries.length) return "";
  return ["```bibliography", JSON.stringify(entries, null, 2), "```", ""].join("\n");
}

function deepResearchCitationIndexMarkdownFromRecords(records: DeepResearchBibliographyRecord[]) {
  if (!records.length) return "";
  return [
    "| Citation | Source | Evidence | Local copy |",
    "| --- | --- | --- | --- |",
    ...records.map(({ citationKey, source }) => [
      `[@${citationKey}]`,
      markdownLink(source.title || citationKey, source.url),
      tableCell(source.snippet || source.source || "Review source before release."),
      tableCell(source.relative_path || ""),
    ].join(" | ")).map((row) => `| ${row} |`),
    "",
  ].join("\n");
}

export function deepResearchCitationGuidanceMarkdown(
  iterations: DeepResearchIteration[],
  savedSources: DeepResearchBibliographySource[] = [],
) {
  const records = deepResearchBibliographyRecords(iterations, savedSources);
  if (!records.length) return "No citation keys are available yet. Use Citation TODO markers for claims that need verification.";
  return [
    "Available source citation keys:",
    "",
    "| Citation | Source | Evidence cue |",
    "| --- | --- | --- |",
    ...records.map(({ citationKey, source }) => [
      `[@${citationKey}]`,
      markdownLink(source.title || citationKey, source.url),
      tableCell(source.snippet || source.source || "Review source before release."),
    ].join(" | ")).map((row) => `| ${row} |`),
  ].join("\n");
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

export function detectDeepResearchEvidenceConflicts(iterations: DeepResearchIteration[]) {
  const conflicts: DeepResearchEvidenceConflict[] = [];
  for (const iteration of iterations) {
    const stancedSources = iteration.results
      .map((source) => ({ source, stances: conflictStancesForSource(source) }))
      .filter((item) => item.stances.length);
    for (const family of CONFLICT_STANCE_FAMILIES) {
      const positive = stancedSources
        .filter((item) => item.stances.some((stance) => stance.family === family.id && stance.polarity === "positive"))
        .map((item) => item.source);
      const negative = stancedSources
        .filter((item) => item.stances.some((stance) => stance.family === family.id && stance.polarity === "negative"))
        .map((item) => item.source);
      if (!positive.length || !negative.length || !hasDistinctSourceUrls(positive, negative)) continue;
      const sources = [
        ...positive.slice(0, 3).map((source) => conflictSourceSummary(source, family.positiveLabel)),
        ...negative.slice(0, 3).map((source) => conflictSourceSummary(source, family.negativeLabel)),
      ];
      conflicts.push({
        id: `DR-CONFLICT-${String(iteration.index).padStart(2, "0")}-${String(conflicts.length + 1).padStart(2, "0")}`,
        topic: iteration.query,
        family: family.label,
        severity: family.risk ? "risk" : "review",
        signals: [family.positiveLabel, family.negativeLabel],
        sources,
        recommendation: `Open the cited sources for ${iteration.query}, decide which claim is best supported, and mark unresolved disagreement as a limitation or citation TODO.`,
      });
    }
  }
  return conflicts;
}

export function deepResearchEvidenceConflictMarkdown(iterations: DeepResearchIteration[]) {
  return deepResearchEvidenceConflictMarkdownFromConflicts(detectDeepResearchEvidenceConflicts(iterations));
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

export function deepResearchSourceQualityReviewItems(iterations: DeepResearchIteration[]): DeepResearchSourceQualityReviewItem[] {
  return iterations.flatMap((iteration) =>
    iteration.results.map((source) => {
      const fit = source.fitScore === undefined || !source.fitLabel
        ? assessDeepResearchSource(source, iteration.query)
        : {
            score: source.fitScore,
            label: source.fitLabel,
            reasons: source.fitReasons?.length ? source.fitReasons : ["review source before citing"],
          };
      return {
        iteration: iteration.index,
        query: iteration.query,
        title: source.title,
        url: source.url,
        source: source.source,
        fitScore: fit.score,
        fitLabel: fit.label,
        fitReasons: fit.reasons,
        reviewAction: sourceQualityReviewAction(fit.label),
      };
    }),
  ).sort((left, right) => left.fitScore - right.fitScore || left.iteration - right.iteration || left.title.localeCompare(right.title));
}

export function deepResearchSourceQualityMarkdown(iterations: DeepResearchIteration[]) {
  const items = deepResearchSourceQualityReviewItems(iterations);
  const summaryRows = deepResearchSourceQualityRows(iterations);
  const weakOrReview = items.filter((item) => item.fitLabel === "weak" || item.fitLabel === "review").length;
  return [
    "## Deep Research Source Quality Review",
    "",
    items.length
      ? `${items.length} source candidate(s) reviewed; ${weakOrReview} require extra caution before citation.`
      : "No source candidates are available yet. Run Deep Research before inserting the source quality review.",
    "",
    "### Fit Bands",
    "",
    "| Fit band | Sources | Review meaning |",
    "| --- | ---: | --- |",
    ...summaryRows.map((row) => `| ${row.label} | ${row.count} | ${tableCell(row.guidance)} |`),
    "",
    "### Source Review Queue",
    "",
    "| Iteration | Fit | Score | Source | Why it was scored this way | Review action |",
    "| ---: | --- | ---: | --- | --- | --- |",
    ...(items.length
      ? items.map((item) => `| ${item.iteration} | ${tableCell(item.fitLabel)} | ${item.fitScore} | ${markdownLink(tableCell(item.title), item.url)} (${tableCell(item.source)}) | ${tableCell(item.fitReasons.join("; "))} | ${tableCell(item.reviewAction)} |`)
      : ["| - | none | 0 | No source candidates | Run Deep Research | Re-run source search, then review every candidate before citing. |"]),
    "",
  ].join("\n");
}

function deepResearchEvidenceConflictMarkdownFromConflicts(conflicts: DeepResearchEvidenceConflict[]) {
  if (!conflicts.length) {
    return [
      "## Deep Research Evidence Conflict Review",
      "",
      "No immediate opposing source signals were detected in search-result snippets. Review source documents before treating this as final proof of consensus.",
      "",
    ].join("\n");
  }
  return [
    "## Deep Research Evidence Conflict Review",
    "",
    `Possible conflicts detected: ${conflicts.length}`,
    "",
    "| Conflict | Severity | Research area | Opposing signals | Source evidence | Review action |",
    "| --- | --- | --- | --- | --- | --- |",
    ...conflicts.map((conflict) => `| ${tableCell(conflict.id)} | ${conflict.severity} | ${tableCell(conflict.topic)} | ${tableCell(conflict.signals.join(" versus "))} | ${tableCell(conflict.sources.map((source) => `${source.stance}: ${source.title}`).join("; "))} | ${tableCell(conflict.recommendation)} |`),
    "",
  ].join("\n");
}

function deepResearchSourceQualityRows(iterations: DeepResearchIteration[]) {
  const counts = new Map<string, number>([
    ["strong", 0],
    ["good", 0],
    ["review", 0],
    ["weak", 0],
  ]);
  for (const iteration of iterations) {
    for (const source of iteration.results) {
      const label = source.fitLabel || assessDeepResearchSource(source, iteration.query).label;
      counts.set(label, (counts.get(label) || 0) + 1);
    }
  }
  return [
    { label: "strong", count: counts.get("strong") || 0, guidance: "Likely useful source candidates, still verify against the downloaded document." },
    { label: "good", count: counts.get("good") || 0, guidance: "Usable source candidates that need normal evidence review." },
    { label: "review", count: counts.get("review") || 0, guidance: "Weakly matched or context-limited sources; use cautiously." },
    { label: "weak", count: counts.get("weak") || 0, guidance: "Low-confidence sources; avoid relying on them without corroboration." },
  ];
}

function sourceQualityReviewAction(label: string) {
  if (label === "strong") return "Open the saved source, verify the cited claim, then prefer it for high-value evidence.";
  if (label === "good") return "Use after normal source review and match the claim to the exact passage.";
  if (label === "review") return "Corroborate with a stronger independent source before relying on this claim.";
  return "Avoid citing unless a reviewer confirms the source is authoritative and directly relevant.";
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

export function fallbackResearchDraft(
  settings: DeepResearchSettings,
  iterations: DeepResearchIteration[],
  savedSources: DeepResearchBibliographySource[] = [],
) {
  const sourceLines = deepResearchBibliographyRecords(iterations, savedSources)
    .map(({ citationKey, source }, index) => `${index + 1}. [@${citationKey}] ${markdownLink(source.title || citationKey, source.url)} - ${source.snippet || source.source || "Review source before release."}`);
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
    "- [ ] Confirm every inline citation key resolves to the bibliography.",
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

interface ConflictStanceFamily {
  id: string;
  label: string;
  positiveLabel: string;
  negativeLabel: string;
  positive: RegExp;
  negative: RegExp;
  risk?: boolean;
}

const CONFLICT_STANCE_FAMILIES: ConflictStanceFamily[] = [
  {
    id: "direction",
    label: "Directional finding",
    positiveLabel: "increase or growth signal",
    negativeLabel: "decrease or contraction signal",
    positive: /\b(?:increase[ds]?|increasing|growth|grew|grow(?:s|ing)?|rising|rose|expanded?|expansion|higher|accelerat(?:e|es|ed|ing))\b/i,
    negative: /\b(?:decrease[ds]?|decreasing|decline[ds]?|declining|fell|falling|fall|reduc(?:e|es|ed|ing|tion)|lower|contract(?:s|ed|ion|ing))\b/i,
  },
  {
    id: "recommendation",
    label: "Recommendation or effectiveness",
    positiveLabel: "supportive or effective signal",
    negativeLabel: "opposing or ineffective signal",
    positive: /\b(?:support(?:s|ed|ing)?|recommend(?:s|ed|ing)?|effective|benefit(?:s|ed|ing)?|success(?:ful)?|improve(?:s|d|ment|ing)|validated|promising)\b/i,
    negative: /\b(?:oppose(?:s|d|ing)?|critic(?:ize|ized|ism|al)|ineffective|fail(?:s|ed|ure|ing)?|harm(?:s|ed|ful)?|concern(?:s)?|warning|warn(?:s|ed|ing))\b/i,
    risk: true,
  },
  {
    id: "obligation",
    label: "Requirement or obligation",
    positiveLabel: "mandatory or required signal",
    negativeLabel: "optional or not-required signal",
    positive: /\b(?:must|shall|required|mandatory|obligation|compulsory|requirement)\b/i,
    negative: /\b(?:optional|voluntary|not required|not mandatory|may choose|at discretion|no requirement)\b/i,
    risk: true,
  },
  {
    id: "certainty",
    label: "Evidence certainty",
    positiveLabel: "confirmed or proven signal",
    negativeLabel: "uncertain or disputed signal",
    positive: /\b(?:confirmed|proven|demonstrated|evidence shows|validated|verified|conclusive)\b/i,
    negative: /\b(?:inconclusive|insufficient evidence|mixed evidence|disputed|uncertain|not enough evidence|contested)\b/i,
    risk: true,
  },
];

function conflictStancesForSource(source: DeepResearchSource) {
  const text = `${source.title} ${source.snippet}`.replace(/\s+/g, " ");
  return CONFLICT_STANCE_FAMILIES.flatMap((family) => {
    const stances: Array<{ family: string; polarity: "positive" | "negative" }> = [];
    if (family.positive.test(text)) stances.push({ family: family.id, polarity: "positive" });
    if (family.negative.test(text)) stances.push({ family: family.id, polarity: "negative" });
    return stances;
  });
}

function hasDistinctSourceUrls(left: DeepResearchSource[], right: DeepResearchSource[]) {
  const leftUrls = new Set(left.map((source) => source.url.trim()).filter(Boolean));
  return right.some((source) => source.url.trim() && !leftUrls.has(source.url.trim()));
}

function conflictSourceSummary(source: DeepResearchSource, stance: string) {
  return {
    title: source.title,
    url: source.url,
    source: source.source,
    stance,
    evidence: source.snippet || source.url,
  };
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
  const iterationSources = uniqueSources(iterations);
  const iterationUrls = new Set(iterationSources.map((source) => source.url.trim()).filter(Boolean));
  const byUrl = new Map<string, DeepResearchBibliographySource>();
  for (const source of savedSources) {
    const url = source.url.trim();
    if (url && (!iterationUrls.size || iterationUrls.has(url))) byUrl.set(url, source);
  }
  for (const source of iterationSources) {
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

interface DeepResearchBibliographyRecord {
  citationKey: string;
  source: DeepResearchBibliographySource;
  entry: Record<string, unknown>;
}

function deepResearchBibliographyRecords(
  iterations: DeepResearchIteration[],
  savedSources: DeepResearchBibliographySource[] = [],
): DeepResearchBibliographyRecord[] {
  const bibliographySources = bibliographySourcesForDocument(iterations, savedSources);
  const usedKeys = new Set<string>();
  return bibliographySources.map((source, index) => {
    const citationKey = uniqueBibliographyKey(source, index, usedKeys);
    const entry: Record<string, unknown> = {
      id: citationKey,
      type: "webpage",
      title: source.title || citationKey,
      URL: source.url,
    };
    const accessed = cslDateFromIso(source.downloaded_at);
    if (accessed) entry.accessed = accessed;
    const note = deepResearchBibliographyNote(source);
    if (note) entry.note = note;
    return { citationKey, source, entry };
  });
}

function missingDeepResearchBibliographyRecords(
  markdown: string,
  records: DeepResearchBibliographyRecord[],
) {
  if (!records.length) return [];
  const blocks = bibliographyFenceBlocks(markdown);
  if (!blocks.length) return records;
  return records.filter(
    (record) => !blocks.some((block) => bibliographyBlockContainsKey(block, record.citationKey)),
  );
}

function missingDeepResearchCitationIndexRecords(
  markdown: string,
  records: DeepResearchBibliographyRecord[],
) {
  if (!records.length) return [];
  const section = sectionText(markdown, "Source Citation Index");
  if (!section) return records;
  return records.filter((record) => !citationIndexSectionContainsKey(section, record.citationKey));
}

function missingDeepResearchEvidenceIterations(
  markdown: string,
  iterations: DeepResearchIteration[],
) {
  if (!iterations.length) return [];
  if (!hasDeepResearchEvidenceLog(markdown)) return iterations;
  return iterations.filter((iteration) => !deepResearchEvidenceLogContainsIteration(markdown, iteration));
}

function deepResearchConflictReviewCompletionMarkdown(markdown: string, iterations: DeepResearchIteration[]) {
  const conflicts = detectDeepResearchEvidenceConflicts(iterations);
  if (!conflicts.length) return "";
  if (!hasDeepResearchConflictReview(markdown)) {
    return deepResearchEvidenceConflictMarkdownFromConflicts(conflicts);
  }
  const existingSection = sectionText(markdown, "Deep Research Evidence Conflict Review");
  const missingConflicts = conflicts.filter((conflict) => !existingConflictReviewContainsId(existingSection, conflict.id));
  if (!missingConflicts.length) return "";
  return [
    "## Deep Research Evidence Conflict Review Addendum",
    "",
    deepResearchEvidenceConflictMarkdownFromConflicts(missingConflicts)
      .replace(/^##\s+Deep Research Evidence Conflict Review\s*\n+/i, "")
      .trim(),
    "",
  ].join("\n");
}

function bibliographyFenceBlocks(markdown: string) {
  const blocks: string[] = [];
  const fencePattern = /^(```|~~~)(?:bibliography|bibtex|hayagriva)\b[^\n]*\r?\n([\s\S]*?)^\1\s*$/gim;
  for (const match of markdown.matchAll(fencePattern)) {
    blocks.push(match[2] || "");
  }
  return blocks;
}

function bibliographyBlockContainsKey(block: string, key: string) {
  const escaped = escapeRegExp(key);
  return [
    new RegExp(`"id"\\s*:\\s*"${escaped}"`),
    new RegExp(`@\\w+\\s*[({]\\s*${escaped}\\s*,`, "i"),
    new RegExp(`^\\s*${escaped}\\s*:`, "m"),
  ].some((pattern) => pattern.test(block));
}

function citationIndexSectionContainsKey(section: string, key: string) {
  return new RegExp(`\\[@${escapeRegExp(key)}(?:\\]|[\\s,;:])`).test(section);
}

function deepResearchEvidenceLogContainsIteration(markdown: string, iteration: DeepResearchIteration) {
  const escapedQuery = escapeRegExp(iteration.query.trim());
  return new RegExp(`^##\\s+Iteration\\s+${iteration.index}:\\s+${escapedQuery}\\s*$`, "im").test(markdown);
}

function sourceLibraryAuditCompletionMarkdown(markdown: string, audit: string) {
  if (!audit) return "";
  if (!hasSourceLibraryAudit(markdown)) return audit;
  const existingSection = sectionText(markdown, "Source Library Audit");
  const missingKeys = sourceLibraryAuditKeys(audit).filter(
    (key) => !sourceLibraryAuditSectionContainsKey(existingSection, key),
  );
  if (!missingKeys.length) return "";
  const addendum = sourceLibraryAuditRowsForKeys(audit, missingKeys);
  if (!addendum) return "";
  return `## Source Library Audit Addendum\n\n${addendum}`;
}

function sourceLibraryAuditKeys(markdown: string) {
  const keys = new Set<string>();
  for (const match of markdown.matchAll(/(?:^|\|)\s*@([A-Za-z0-9_.:-]+)\s*(?=\|)/gm)) {
    keys.add(match[1]);
  }
  return Array.from(keys);
}

function sourceLibraryAuditRowsForKeys(markdown: string, keys: string[]) {
  const keySet = new Set(keys);
  const lines = markdown.split(/\r?\n/);
  const headerIndex = lines.findIndex((line) => /^\|\s*Citation key\s*\|/i.test(line));
  const dividerIndex = headerIndex >= 0 ? headerIndex + 1 : -1;
  const matchingRows = lines.filter((line) => {
    const match = line.match(/^\|\s*@([A-Za-z0-9_.:-]+)\s*\|/);
    return match ? keySet.has(match[1]) : false;
  });
  if (!matchingRows.length) return "";
  const header = headerIndex >= 0 && dividerIndex < lines.length
    ? [lines[headerIndex], lines[dividerIndex]]
    : ["| Citation key | Audit row |", "| --- | --- |"];
  return [
    `Saved sources needing audit addendum: ${matchingRows.length}`,
    "",
    ...header,
    ...matchingRows,
    "",
  ].join("\n");
}

function sourceLibraryAuditSectionContainsKey(section: string, key: string) {
  return new RegExp(`(?:^|\\|)\\s*@${escapeRegExp(key)}\\s*(?:\\||$)`, "m").test(section);
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
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

function markdownLink(label: string, url: string) {
  const safeLabel = label.replace(/[[\]|]/g, "").replace(/\s+/g, " ").trim() || url;
  const safeUrl = url.replace(/[()\s]/g, encodeURIComponent);
  return `[${safeLabel}](${safeUrl})`;
}

function tableCell(value: string) {
  return (value || "-")
    .replace(/\r?\n/g, " ")
    .replace(/\|/g, "\\|")
    .trim() || "-";
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
  if (value === "searxng" || value === "tavily" || value === "local-library") return value;
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
  return /^##\s+Deep Research Evidence Log\s*$/im.test(markdown);
}

function hasDeepResearchCitationIndex(markdown: string) {
  return /^##\s+Source Citation Index\s*$/im.test(markdown);
}

function hasBibliographyMarker(markdown: string) {
  return /\[BIBLIOGRAPHY\]/i.test(markdown);
}

function hasSourceLibraryAudit(markdown: string) {
  return /^##\s+Source Library Audit\s*$/im.test(markdown);
}

function hasDeepResearchConflictReview(markdown: string) {
  return /^##\s+Deep Research Evidence Conflict Review\s*$/im.test(markdown);
}

function existingConflictReviewContainsId(section: string, id: string) {
  return new RegExp(`(?:^|\\|)\\s*${escapeRegExp(id)}\\s*(?:\\||:|$)`, "m").test(section);
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
