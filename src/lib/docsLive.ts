import { parseOutlinePlan, type OutlinePlanItem } from "./documentOutline.js";

export const docsLiveDocumentTypes = [
  { id: "business-brief", label: "Business brief" },
  { id: "board-memo", label: "Board memo" },
  { id: "proposal", label: "Proposal" },
  { id: "strategy-plan", label: "Strategy plan" },
  { id: "project-plan", label: "Project plan" },
  { id: "research-brief", label: "Research brief" },
  { id: "policy", label: "Policy" },
  { id: "meeting-brief", label: "Meeting brief" },
] as const;

export type DocsLiveDocumentType = (typeof docsLiveDocumentTypes)[number]["id"];

export interface DocsLiveDraftRequest {
  documentType?: string;
  title?: string;
  outline?: string;
  context?: string;
  transcript?: string;
  placeholders?: string;
  generatedAt?: string;
}

export interface DocsLiveDraft {
  title: string;
  documentType: DocsLiveDocumentType;
  outlineText: string;
  questionnaire: string;
  markdown: string;
  placeholders: Record<string, string>;
  sections: Array<{ title: string; level: number; qaFocus: string }>;
  issues: string[];
}

interface DocsLiveBlueprint {
  label: string;
  defaultOutline: string[];
  questions: string[];
  sectionFocus: string[];
}

const blueprints: Record<DocsLiveDocumentType, DocsLiveBlueprint> = {
  "business-brief": {
    label: "Business brief",
    defaultOutline: ["Executive Summary", "Context", "Options", "Recommendation", "Next Steps"],
    questions: [
      "Who is the audience?",
      "What decision or action should this document support?",
      "What constraints, risks, or dates must be reflected?",
      "Which facts, metrics, or sources need verification?",
    ],
    sectionFocus: ["decision", "context", "tradeoffs", "recommendation", "owners"],
  },
  "board-memo": {
    label: "Board memo",
    defaultOutline: ["Executive Summary", "Decision Needed", "Strategic Context", "Financial Case", "Risks", "Requested Approval"],
    questions: [
      "What board decision is required?",
      "What are the strategic alternatives?",
      "What financial or operational evidence should anchor the memo?",
      "Which risks should directors challenge?",
    ],
    sectionFocus: ["decision", "governance", "evidence", "risk", "approval"],
  },
  proposal: {
    label: "Proposal",
    defaultOutline: ["Executive Summary", "Client Need", "Proposed Approach", "Scope", "Timeline", "Investment", "Acceptance"],
    questions: [
      "Who is the client or sponsor?",
      "What problem does the proposal solve?",
      "What is in scope and explicitly out of scope?",
      "What budget, timeline, and acceptance criteria matter?",
    ],
    sectionFocus: ["client", "problem", "approach", "scope", "timeline", "commercials"],
  },
  "strategy-plan": {
    label: "Strategy plan",
    defaultOutline: ["Strategic Intent", "Current State", "Market Forces", "Choices", "Roadmap", "Measures"],
    questions: [
      "What ambition or strategic bet should lead?",
      "What evidence describes the current state?",
      "What choices should be made or rejected?",
      "How will progress be measured?",
    ],
    sectionFocus: ["intent", "baseline", "market", "choices", "execution", "metrics"],
  },
  "project-plan": {
    label: "Project plan",
    defaultOutline: ["Objective", "Scope", "Workstreams", "Milestones", "Risks", "Governance"],
    questions: [
      "What outcome defines success?",
      "What workstreams and owners are known?",
      "What milestones or dependencies should be scheduled?",
      "What risks and governance routines are required?",
    ],
    sectionFocus: ["outcome", "scope", "owners", "milestones", "risks", "cadence"],
  },
  "research-brief": {
    label: "Research brief",
    defaultOutline: ["Research Question", "Method", "Findings", "Implications", "Evidence Gaps", "References"],
    questions: [
      "What research question should the brief answer?",
      "What sources or methods are acceptable?",
      "What findings are already known?",
      "Where does evidence remain weak?",
    ],
    sectionFocus: ["question", "method", "findings", "implications", "gaps", "sources"],
  },
  policy: {
    label: "Policy",
    defaultOutline: ["Purpose", "Scope", "Policy Requirements", "Roles", "Exceptions", "Review Cycle"],
    questions: [
      "Who must follow the policy?",
      "What behavior is required, allowed, or prohibited?",
      "Who approves exceptions?",
      "How often should the policy be reviewed?",
    ],
    sectionFocus: ["purpose", "scope", "requirements", "roles", "exceptions", "review"],
  },
  "meeting-brief": {
    label: "Meeting brief",
    defaultOutline: ["Purpose", "Attendees", "Agenda", "Pre-reads", "Decisions", "Follow-ups"],
    questions: [
      "What should the meeting accomplish?",
      "Who is attending and what do they need?",
      "Which materials should be read in advance?",
      "What decisions or follow-ups must be captured?",
    ],
    sectionFocus: ["purpose", "people", "agenda", "materials", "decisions", "actions"],
  },
};

const explicitTypeSignals: Array<[DocsLiveDocumentType, RegExp]> = [
  ["board-memo", /\b(board|directors?|approval)\b/i],
  ["proposal", /\b(proposal|scope of work|client|pricing|investment)\b/i],
  ["strategy-plan", /\b(strategy|market|roadmap|strategic)\b/i],
  ["project-plan", /\b(project plan|workstream|milestone|dependency)\b/i],
  ["research-brief", /\b(research|findings|method|sources?|evidence)\b/i],
  ["policy", /\b(policy|requirements?|exceptions?|compliance)\b/i],
  ["meeting-brief", /\b(meeting|agenda|attendees?|pre-reads?)\b/i],
];

const placeholderSignals = [
  "client",
  "company",
  "customer",
  "audience",
  "decision",
  "goal",
  "deadline",
  "date",
  "budget",
  "owner",
  "region",
  "product",
  "tone",
  "reviewer",
  "approver",
  "industry",
];

export function normalizeDocsLiveDocumentType(input = ""): DocsLiveDocumentType {
  const normalized = input.toLowerCase().trim();
  if (docsLiveDocumentTypes.some((type) => type.id === normalized)) return normalized as DocsLiveDocumentType;
  const direct = docsLiveDocumentTypes.find((type) => type.label.toLowerCase() === normalized);
  if (direct) return direct.id;
  const signaled = explicitTypeSignals.find(([, signal]) => signal.test(input));
  return signaled?.[0] || "business-brief";
}

export function buildDocsLiveQuestionnaire(documentType: string) {
  const type = normalizeDocsLiveDocumentType(documentType);
  return blueprints[type].questions.map((question, index) => `${index + 1}. ${question}`).join("\n");
}

export function buildDocsLiveDraft(request: DocsLiveDraftRequest): DocsLiveDraft {
  const documentType = inferDocumentType(request);
  const blueprint = blueprints[documentType];
  const placeholders = extractDocsLivePlaceholders([request.placeholders, request.context, request.transcript].filter(Boolean).join("\n"));
  const title = resolveTitle(request, blueprint, placeholders);
  const outlineText = resolveOutlineText(request, blueprint);
  const outlineItems = parseOutlinePlan(outlineText);
  const sections = outlineItems.length ? outlineItems : blueprint.defaultOutline.map((section) => ({ level: 1, title: section }));
  const generatedAt = request.generatedAt || new Date().toISOString();
  const contextSentences = extractContextSentences([request.transcript, request.context].filter(Boolean).join("\n"));
  const issues = buildDraftIssues(request, placeholders, sections);
  const markdown = humanizeDraftText(
    [
      "---",
      `title: ${yamlScalar(title)}`,
      "status: draft",
      `documentType: ${yamlScalar(blueprint.label)}`,
      "toc: true",
      "---",
      "",
      `# ${title}`,
      "",
      "[TOC]",
      "",
      docsLiveSourceBlock(generatedAt, documentType, contextSentences),
      "",
      placeholdersTable(placeholders),
      "",
      ...sections.flatMap((section, index) => draftSection(section, index, sections.length, blueprint, placeholders, contextSentences)),
      "## Review Preparation",
      "",
      "### Quality Assurance",
      "",
      ...qualityChecklist(sections, placeholders).map((item) => `- [ ] ${item}`),
      "",
      "### Humanization Pass",
      "",
      "- [ ] Replace generic claims with named facts, numbers, owners, dates, or citations.",
      "- [ ] Remove filler phrases and keep one clear idea per paragraph.",
      "- [ ] Read each section aloud and shorten any sentence that sounds scripted.",
      "- [ ] Mark AI-assisted sections as human-reviewed only after a person confirms the content.",
      "",
      "### Reviewer Notes",
      "",
      "<!-- comment: unresolved | author: Docs Live | at: " + generatedAt + " | Confirm facts, figures, citations, and final tone before export. -->",
      "",
    ].join("\n"),
  );

  return {
    title,
    documentType,
    outlineText,
    questionnaire: buildDocsLiveQuestionnaire(documentType),
    markdown,
    placeholders,
    sections: sections.map((section, index) => ({
      title: section.title,
      level: section.level,
      qaFocus: blueprint.sectionFocus[index % blueprint.sectionFocus.length],
    })),
    issues,
  };
}

export function extractDocsLivePlaceholders(input: string): Record<string, string> {
  const placeholders: Record<string, string> = {};
  for (const line of input.split(/\r?\n/)) {
    const pair = line.match(/^\s*([A-Za-z][A-Za-z0-9 _-]{1,36})\s*[:=]\s*(.+?)\s*$/);
    if (pair) placeholders[normalizePlaceholderKey(pair[1])] = pair[2].trim();
  }
  for (const key of placeholderSignals) {
    const signal = new RegExp(`\\b${key}\\s+(?:is|are|=|:)\\s+([^.;\\n]+)`, "i");
    const match = input.match(signal);
    if (match && !placeholders[normalizePlaceholderKey(key)]) {
      placeholders[normalizePlaceholderKey(key)] = match[1].trim();
    }
  }
  return placeholders;
}

function inferDocumentType(request: DocsLiveDraftRequest) {
  const explicit = normalizeDocsLiveDocumentType(request.documentType || "");
  if ((request.documentType || "").trim()) return explicit;
  return normalizeDocsLiveDocumentType([request.transcript, request.context, request.title].filter(Boolean).join("\n"));
}

function resolveOutlineText(request: DocsLiveDraftRequest, blueprint: DocsLiveBlueprint) {
  const provided = (request.outline || "").trim();
  if (parseOutlinePlan(provided).length) return provided;
  return blueprint.defaultOutline.map((section) => `- ${section}`).join("\n");
}

function resolveTitle(request: DocsLiveDraftRequest, blueprint: DocsLiveBlueprint, placeholders: Record<string, string>) {
  const explicit = (request.title || placeholders.title || "").trim();
  if (explicit) return explicit;
  const subject = placeholders.client || placeholders.company || placeholders.customer || placeholders.product || placeholders.goal;
  return subject ? `${subject} ${blueprint.label}` : blueprint.label;
}

function buildDraftIssues(request: DocsLiveDraftRequest, placeholders: Record<string, string>, sections: OutlinePlanItem[]) {
  const issues: string[] = [];
  if (!Object.keys(placeholders).length) issues.push("No placeholder values were detected; draft includes review prompts for missing specifics.");
  if (!sections.length) issues.push("No usable outline was supplied; Docs Live used a document-type outline.");
  if (![request.context, request.transcript].some((value) => value?.trim())) issues.push("No document context was supplied; section drafts are scaffolded for human completion.");
  return issues;
}

function docsLiveSourceBlock(generatedAt: string, documentType: DocsLiveDocumentType, contextSentences: string[]) {
  const promptSummary = sanitizeMarkerValue(
    contextSentences[0] || `Voice-guided ${blueprints[documentType].label.toLowerCase()} draft from outline and placeholders`,
  );
  return [
    "```ai-source",
    "provider: NEditor Docs Live",
    "model: local-guided-drafting",
    `date: ${generatedAt}`,
    `promptSummary: ${promptSummary}`,
    "reviewedBy: ",
    "reviewedAt: ",
    "status: needs-review",
    "```",
  ].join("\n");
}

function placeholdersTable(placeholders: Record<string, string>) {
  const entries = Object.entries(placeholders);
  if (!entries.length) return "<!-- Docs Live placeholders: add key facts before review. -->";
  return [
    "## Draft Inputs",
    "",
    "| Placeholder | Value |",
    "| --- | --- |",
    ...entries.map(([key, value]) => `| ${titleCase(key)} | ${escapeTableCell(value)} |`),
  ].join("\n");
}

function draftSection(
  section: OutlinePlanItem,
  index: number,
  total: number,
  blueprint: DocsLiveBlueprint,
  placeholders: Record<string, string>,
  contextSentences: string[],
) {
  const level = Math.min(6, Math.max(2, section.level + 1));
  const focus = blueprint.sectionFocus[index % blueprint.sectionFocus.length];
  const audience = placeholders.audience || "the intended reader";
  const owner = placeholders.owner || placeholders.reviewer || "[owner]";
  const deadline = placeholders.deadline || placeholders.date || "[date]";
  const context = contextSentences[index % Math.max(1, contextSentences.length)] || "Use the provided outline and replace placeholders with verified facts.";
  const promptSummary = sanitizeMarkerValue(`Drafted ${section.title} section ${index + 1} of ${total}`);
  return [
    `<!-- ai-assisted: status=needs-review | reviewedBy= | reviewedAt= | source=NEditor Docs Live | promptSummary=${promptSummary} -->`,
    `${"#".repeat(level)} ${section.title}`,
    "",
    `**Purpose.** This section should help ${audience} understand the ${focus} behind ${section.title.toLowerCase()}.`,
    "",
    `**Draft.** ${context} Frame the point in plain language, name the tradeoff, and connect it to the action the reader should take.`,
    "",
    "- Key point: [replace with the most important fact or recommendation].",
    `- Owner: ${owner}.`,
    `- Timing: ${deadline}.`,
    "- Evidence: [add source, citation, table, or calculation].",
    "",
    "**Review focus.** Confirm the claim, remove unsupported qualifiers, and make the paragraph sound like a knowledgeable person wrote it.",
    "",
  ];
}

function qualityChecklist(sections: OutlinePlanItem[], placeholders: Record<string, string>) {
  return [
    `Every planned section has a drafted body (${sections.length} section${sections.length === 1 ? "" : "s"}).`,
    "Each recommendation, risk, date, and amount is backed by source material.",
    "The opening section states the audience, decision, and desired next action.",
    Object.keys(placeholders).length ? "Placeholder values were inserted and checked for accuracy." : "Missing placeholder values were filled or explicitly marked.",
    "AI provenance and review markers remain until a human reviewer signs off.",
  ];
}

function extractContextSentences(input: string) {
  return input
    .replace(/\s+/g, " ")
    .split(/(?<=[.!?])\s+/)
    .map((sentence) => sentence.trim())
    .filter((sentence) => sentence.length > 16)
    .slice(0, 12);
}

function humanizeDraftText(markdown: string) {
  return markdown
    .replace(/\butilize\b/gi, "use")
    .replace(/\bleverage\b/gi, "use")
    .replace(/\brobust\b/gi, "clear")
    .replace(/\bseamless\b/gi, "smooth")
    .replace(/\n{3,}/g, "\n\n")
    .trimEnd()
    .concat("\n");
}

function normalizePlaceholderKey(key: string) {
  return key
    .trim()
    .replace(/[_-]+/g, " ")
    .replace(/\s+/g, " ")
    .toLowerCase();
}

function titleCase(value: string) {
  return value.replace(/\b\w/g, (match) => match.toUpperCase());
}

function escapeTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}

function sanitizeMarkerValue(value: string) {
  return value.replace(/[|\n\r]/g, " ").replace(/-->/g, "->").replace(/\s+/g, " ").trim().slice(0, 180);
}

function yamlScalar(value: string) {
  if (/^[A-Za-z0-9 _.,:/()-]+$/.test(value)) return value;
  return JSON.stringify(value);
}
