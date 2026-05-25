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
  { id: "business-case", label: "Business case" },
  { id: "operating-procedure", label: "Operating procedure" },
  { id: "technical-architecture", label: "Technical architecture" },
  { id: "adr", label: "Architecture decision record" },
  { id: "release-notes", label: "Release notes" },
  { id: "contract-brief", label: "Contract brief" },
  { id: "marketing-brief", label: "Marketing brief" },
  { id: "customer-case-study", label: "Customer case study" },
] as const;

export type DocsLiveDocumentType = (typeof docsLiveDocumentTypes)[number]["id"];

export interface DocsLiveDraftRequest {
  documentType?: string;
  title?: string;
  outline?: string;
  context?: string;
  questionnaireAnswers?: string;
  transcript?: string;
  placeholders?: string;
  draftingDepth?: DocsLiveDraftDepth;
  generatedAt?: string;
}

export interface DocsLiveQuestionnaireRequest {
  title?: string;
  outline?: string;
  context?: string;
  transcript?: string;
  placeholders?: string;
}

export type DocsLiveDraftDepth = "concise" | "standard" | "detailed";

export interface DocsLiveDraft {
  title: string;
  documentType: DocsLiveDocumentType;
  outlineText: string;
  questionnaire: string;
  markdown: string;
  placeholders: Record<string, string>;
  workflow: DocsLiveWorkflowStep[];
  reviewPacket: DocsLiveReviewPacket;
  sections: DocsLiveSectionDraft[];
  issues: string[];
}

export interface DocsLiveWorkflowStep {
  id: string;
  label: string;
  status: "ready" | "needs-input" | "complete";
  detail: string;
}

export interface DocsLiveSectionDraft {
  title: string;
  level: number;
  qaFocus: string;
  draftingBrief: string;
  stagePlan: DocsLiveSectionStage[];
  contextBridge: string;
  qaChecks: string[];
  qaSummary: string;
  humanizationNotes: string[];
  humanizedAngle: string;
  reviewQuestions: string[];
  reviewHandoff: string;
}

export interface DocsLiveSectionStage {
  id: "draft" | "qa" | "humanize" | "review";
  label: string;
  status: "complete" | "needs-review";
  detail: string;
}

export interface DocsLiveReviewPacket {
  contextSources: string[];
  sectionRunbook: string[];
  qaRegister: string[];
  humanizationChecklist: string[];
  reviewerHandoff: string[];
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
  "business-case": {
    label: "Business case",
    defaultOutline: ["Executive Summary", "Problem", "Options", "Financial Case", "Risks", "Recommendation", "Implementation Plan"],
    questions: [
      "What decision should the business case enable?",
      "Which options should be compared?",
      "What investment, benefit, risk, or ROI assumptions must be included?",
      "What implementation constraints or dependencies matter?",
    ],
    sectionFocus: ["decision", "problem", "options", "financials", "risk", "recommendation", "execution"],
  },
  "operating-procedure": {
    label: "Operating procedure",
    defaultOutline: ["Purpose", "Scope", "Prerequisites", "Procedure", "Controls", "Exceptions", "Revision History"],
    questions: [
      "Who performs the procedure and when?",
      "What systems, permissions, materials, or inputs are required?",
      "What controls, checks, or approvals prevent mistakes?",
      "What exceptions or escalation paths should be documented?",
    ],
    sectionFocus: ["purpose", "scope", "inputs", "steps", "controls", "exceptions", "change control"],
  },
  "technical-architecture": {
    label: "Technical architecture",
    defaultOutline: ["Overview", "Goals", "Context", "Components", "Data Flow", "Security", "Operations", "Open Decisions"],
    questions: [
      "What system, product, or platform is being described?",
      "What architectural goals and non-goals should constrain the design?",
      "Which components, integrations, data flows, and trust boundaries matter?",
      "What operational, security, or migration risks require review?",
    ],
    sectionFocus: ["overview", "goals", "context", "components", "data", "security", "operations", "decisions"],
  },
  adr: {
    label: "Architecture decision record",
    defaultOutline: ["Status", "Context", "Decision", "Consequences", "Alternatives", "Follow-ups"],
    questions: [
      "What decision needs to be recorded?",
      "What context, constraints, or forces shaped the decision?",
      "Which alternatives were considered and rejected?",
      "What consequences or follow-up actions should future readers know?",
    ],
    sectionFocus: ["status", "context", "decision", "tradeoffs", "alternatives", "follow-ups"],
  },
  "release-notes": {
    label: "Release notes",
    defaultOutline: ["Release Summary", "Highlights", "New Features", "Fixes", "Known Issues", "Upgrade Notes", "Support"],
    questions: [
      "Who is the release audience?",
      "What changed and why does it matter?",
      "Which upgrade steps, compatibility notes, or known issues are required?",
      "What support or rollback information should be included?",
    ],
    sectionFocus: ["audience", "value", "features", "fixes", "known issues", "upgrade", "support"],
  },
  "contract-brief": {
    label: "Contract brief",
    defaultOutline: ["Purpose", "Parties", "Commercial Terms", "Obligations", "Risks", "Approvals", "Open Questions"],
    questions: [
      "Which agreement or transaction is being summarized?",
      "Who are the parties and accountable reviewers?",
      "Which commercial, legal, operational, or data terms need attention?",
      "What risks, approvals, or open questions must remain visible?",
    ],
    sectionFocus: ["purpose", "parties", "commercials", "obligations", "risk", "approvals", "questions"],
  },
  "marketing-brief": {
    label: "Marketing brief",
    defaultOutline: ["Objective", "Audience", "Positioning", "Message", "Channels", "Assets", "Success Measures"],
    questions: [
      "What campaign, launch, or message is being briefed?",
      "Who is the audience and what should they believe or do?",
      "What positioning, proof points, and claims are approved?",
      "Which channels, assets, dates, and measures define success?",
    ],
    sectionFocus: ["objective", "audience", "positioning", "message", "channels", "assets", "measures"],
  },
  "customer-case-study": {
    label: "Customer case study",
    defaultOutline: ["Customer Snapshot", "Challenge", "Solution", "Implementation", "Results", "Quote Prompts", "Review Approvals"],
    questions: [
      "Which customer, industry, and use case should the story feature?",
      "What problem did the customer need to solve?",
      "What solution, implementation path, and measurable results can be verified?",
      "Which quotes, approvals, and confidentiality constraints apply?",
    ],
    sectionFocus: ["customer", "challenge", "solution", "implementation", "results", "quotes", "approvals"],
  },
};

const explicitTypeSignals: Array<[DocsLiveDocumentType, RegExp]> = [
  ["business-case", /\b(business case|roi|return on investment|cost benefit|financial case)\b/i],
  ["operating-procedure", /\b(sop|standard operating procedure|operating procedure|runbook|work instruction)\b/i],
  ["technical-architecture", /\b(technical architecture|architecture document|system design|data flow|trust boundaries?)\b/i],
  ["adr", /\b(adr|architecture decision record|decision record)\b/i],
  ["release-notes", /\b(release notes?|changelog|upgrade notes?|known issues?)\b/i],
  ["contract-brief", /\b(contract brief|agreement summary|legal brief|commercial terms|obligations)\b/i],
  ["marketing-brief", /\b(marketing brief|campaign brief|positioning|messaging|channels?)\b/i],
  ["customer-case-study", /\b(case study|customer story|success story|customer proof)\b/i],
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

export function buildDocsLiveQuestionnaire(documentType: string, request: DocsLiveQuestionnaireRequest = {}) {
  const type = normalizeDocsLiveDocumentType(documentType);
  const outlineItems = parseOutlinePlan(request.outline || "");
  const placeholders = extractDocsLivePlaceholders([request.placeholders, request.context, request.transcript].filter(Boolean).join("\n"));
  const questions = [...blueprints[type].questions];
  const title = (request.title || "").trim();
  if (title) questions.unshift(`What should "${title}" help the reader decide, approve, or do?`);
  for (const section of outlineItems.slice(0, 8)) {
    questions.push(`For "${section.title}", what facts, examples, calculations, decisions, or caveats must be included?`);
  }
  const missing = ["audience", "owner", "deadline", "evidence", "tone", "reviewer"].filter((key) => !placeholders[key]);
  if (missing.length) {
    questions.push(`Which ${missing.map(titleCase).join(", ")} values should Docs Live use as placeholders or review prompts?`);
  }
  questions.push("What must remain visibly marked for human review before export or publication?");
  return Array.from(new Set(questions)).map((question, index) => `${index + 1}. ${question}`).join("\n");
}

export function buildDocsLiveDraft(request: DocsLiveDraftRequest): DocsLiveDraft {
  const documentType = inferDocumentType(request);
  const blueprint = blueprints[documentType];
  const contextInput = [request.transcript, request.context, request.questionnaireAnswers].filter(Boolean).join("\n");
  const placeholders = extractDocsLivePlaceholders([request.placeholders, request.context, request.questionnaireAnswers, request.transcript].filter(Boolean).join("\n"));
  const title = resolveTitle(request, blueprint, placeholders);
  const outlineText = resolveOutlineText(request, blueprint);
  const outlineItems = parseOutlinePlan(outlineText);
  const sections = outlineItems.length ? outlineItems : blueprint.defaultOutline.map((section) => ({ level: 1, title: section }));
  const generatedAt = request.generatedAt || new Date().toISOString();
  const contextSentences = extractContextSentences(contextInput);
  const issues = buildDraftIssues(request, placeholders, sections);
  const draftingDepth = normalizeDraftingDepth(request.draftingDepth);
  const sectionDrafts = sections.map((section, index) => buildSectionDraft(section, index, blueprint, placeholders, contextSentences));
  const workflow = buildDocsLiveWorkflow(sectionDrafts, placeholders, contextSentences, issues);
  const reviewPacket = buildDocsLiveReviewPacket(request, sectionDrafts, placeholders, contextSentences, issues);
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
      docsLiveContextSummary(contextSentences),
      "",
      docsLiveReviewMarker("Docs Live systematic outline-to-draft workflow"),
      "",
      draftingPlanTable(workflow, sectionDrafts, draftingDepth),
      "",
      reviewPacketMarkdown(reviewPacket),
      "",
      ...sectionDrafts.flatMap((section, index) =>
        draftSection(section, index, sectionDrafts.length, blueprint, placeholders, contextSentences, draftingDepth),
      ),
      "## Review Handoff",
      "",
      "This draft is ready for human review once each section owner checks facts, numbers, citations, tone, and unresolved assumptions.",
      "",
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
    questionnaire: buildDocsLiveQuestionnaire(documentType, {
      title,
      outline: outlineText,
      context: request.context,
      transcript: request.transcript,
      placeholders: request.placeholders,
    }),
    markdown,
    placeholders,
    workflow,
    reviewPacket,
    sections: sectionDrafts,
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
  if (![request.context, request.transcript, request.questionnaireAnswers].some((value) => value?.trim())) {
    issues.push("No document context was supplied; section drafts are scaffolded for human completion.");
  }
  return issues;
}

function buildDocsLiveWorkflow(
  sections: DocsLiveSectionDraft[],
  placeholders: Record<string, string>,
  contextSentences: string[],
  issues: string[],
): DocsLiveWorkflowStep[] {
  return [
    {
      id: "outline",
      label: "Outline locked",
      status: sections.length ? "complete" : "needs-input",
      detail: `${sections.length} planned section${sections.length === 1 ? "" : "s"} ready for systematic drafting.`,
    },
    {
      id: "context",
      label: "Context captured",
      status: contextSentences.length || Object.keys(placeholders).length ? "complete" : "needs-input",
      detail: contextSentences.length
        ? `${Math.min(contextSentences.length, 12)} context point${contextSentences.length === 1 ? "" : "s"} available.`
        : "Add freeform context, questionnaire answers, or placeholder values before final review.",
    },
    {
      id: "draft",
      label: "Section-by-section draft",
      status: "complete",
      detail: "Each outline item receives a body draft, local evidence prompts, and a review handoff.",
    },
    {
      id: "qa",
      label: "Quality assurance",
      status: issues.length ? "needs-input" : "complete",
      detail: issues.length ? `${issues.length} item${issues.length === 1 ? "" : "s"} need attention before review.` : "Generated QA gates are ready.",
    },
    {
      id: "humanize",
      label: "Humanization pass",
      status: "complete",
      detail: "Draft text is stripped of common AI phrasing and marked for human review.",
    },
    {
      id: "review",
      label: "Review handoff",
      status: "complete",
      detail: "Each section carries reviewer questions, unresolved assumptions, and sign-off prompts.",
    },
  ];
}

function buildDocsLiveReviewPacket(
  request: DocsLiveDraftRequest,
  sections: DocsLiveSectionDraft[],
  placeholders: Record<string, string>,
  contextSentences: string[],
  issues: string[],
): DocsLiveReviewPacket {
  const placeholderCount = Object.keys(placeholders).length;
  const contextSources = [
    `${sections.length} outline section${sections.length === 1 ? "" : "s"} locked before drafting.`,
    request.transcript?.trim() ? "Voice or dictated direction captured as drafting intent." : "Voice direction not supplied; use written context during review.",
    request.context?.trim() ? "Freeform document context captured." : "Freeform document context missing or minimal.",
    request.questionnaireAnswers?.trim()
      ? "AI-created questionnaire answers captured as structured constraints."
      : "Questionnaire answers not supplied; generated questions remain available for review.",
    placeholderCount
      ? `${placeholderCount} placeholder value${placeholderCount === 1 ? "" : "s"} available for names, dates, owners, amounts, or audience.`
      : "No placeholder values detected; bracketed review prompts remain in the draft.",
    contextSentences.length
      ? `${Math.min(contextSentences.length, 12)} context point${contextSentences.length === 1 ? "" : "s"} available for section drafting.`
      : "No context points extracted; each section includes visible review prompts.",
  ];
  const sectionRunbook = sections.map(
    (section, index) =>
      `${index + 1}. ${section.title}: draft body, run QA against ${section.qaFocus}, humanize the prose, then hand to reviewer.`,
  );
  const qaRegister = [
    ...issues,
    ...sections.map((section) => section.qaSummary),
    "Final export should remain blocked until unresolved facts, figures, citations, and assumptions are checked.",
  ];
  const humanizationChecklist = [
    "Remove AI cruft: prompt echoes, generic setup phrases, unsupported confidence, and padded transitions.",
    "Replace prompt-shaped phrasing with natural subject-matter-owner language.",
    "Cut repeated framing, unsupported certainty, filler adjectives, and generic transition sentences.",
    "Add named people, teams, sources, dates, calculations, or examples wherever the draft sounds abstract.",
    "Read the final draft aloud and shorten any sentence that a reviewer would not naturally say.",
  ];
  const reviewerHandoff = [
    "Assign each section to an owner before approval.",
    "Keep AI-assisted markers until the responsible reviewer marks the section human-reviewed.",
    "Confirm document type, audience, decision, tone, and placeholders before export.",
    "Collect missing evidence, citations, calculations, and approvals in the Review panel.",
  ];
  return {
    contextSources,
    sectionRunbook,
    qaRegister,
    humanizationChecklist,
    reviewerHandoff,
  };
}

function docsLiveSourceBlock(generatedAt: string, documentType: DocsLiveDocumentType, contextSentences: string[]) {
  const promptSummary = sanitizeMarkerValue(
    contextSentences[0] || `Voice-guided ${blueprints[documentType].label.toLowerCase()} draft from outline and placeholders`,
  );
  return [
    "```ai-source",
    "provider: NEditor Docs Live",
    "model: local-guided-drafting",
    "workflow: outline-to-section-draft-qa-humanize-review",
    `date: ${generatedAt}`,
    `promptSummary: ${promptSummary}`,
    "reviewedBy: ",
    "reviewedAt: ",
    "status: needs-review",
    "```",
  ].join("\n");
}

function draftingPlanTable(workflow: DocsLiveWorkflowStep[], sections: DocsLiveSectionDraft[], draftingDepth: DocsLiveDraftDepth) {
  return [
    "## Drafting Plan",
    "",
    `Docs Live will work through the outline section by section at ${draftingDepth} depth, then attach QA, humanization, and review handoff notes.`,
    "",
    "| Stage | Status | Detail |",
    "| --- | --- | --- |",
    ...workflow.map((step) => `| ${escapeTableCell(step.label)} | ${escapeTableCell(step.status)} | ${escapeTableCell(step.detail)} |`),
    "",
    "| Section | Drafting brief | QA focus |",
    "| --- | --- | --- |",
    ...sections.map((section) => `| ${escapeTableCell(section.title)} | ${escapeTableCell(section.draftingBrief)} | ${escapeTableCell(section.qaFocus)} |`),
  ].join("\n");
}

function reviewPacketMarkdown(packet: DocsLiveReviewPacket) {
  return [
    "## Section-by-section Draft Runbook",
    "",
    "Docs Live uses the outline as the work queue. Each section is drafted, checked, humanized, and packaged for review before the next approval step.",
    "",
    "### Context Package",
    "",
    ...packet.contextSources.map((source) => `- ${source}`),
    "",
    "### Section Work Queue",
    "",
    ...packet.sectionRunbook.map((item) => `- ${item}`),
    "",
    "### Assumption Register",
    "",
    ...packet.qaRegister.map((item) => `- [ ] ${item}`),
    "",
    "### Humanization Checklist",
    "",
    ...packet.humanizationChecklist.map((item) => `- [ ] ${item}`),
    "",
    "### Review Packet",
    "",
    ...packet.reviewerHandoff.map((item) => `- [ ] ${item}`),
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

function docsLiveContextSummary(contextSentences: string[]) {
  if (!contextSentences.length) return "<!-- Docs Live context: add freeform notes, questionnaire answers, or dictated direction before review. -->";
  return [
    "## Draft Context",
    "",
    ...contextSentences.slice(0, 8).map((sentence) => `- ${sentence}`),
  ].join("\n");
}

function buildSectionDraft(
  section: OutlinePlanItem,
  index: number,
  blueprint: DocsLiveBlueprint,
  placeholders: Record<string, string>,
  contextSentences: string[],
): DocsLiveSectionDraft {
  const focus = blueprint.sectionFocus[index % blueprint.sectionFocus.length];
  const owner = placeholders.owner || placeholders.reviewer || "the named owner";
  const evidence = placeholders.evidence || placeholders.source || "the strongest available evidence";
  const draftingBrief = `Frame the ${focus} for ${placeholders.audience || "the intended reader"} and connect it to the next decision.`;
  const contextBridge = contextSentences[index % Math.max(1, contextSentences.length)] || "Use the outline intent and keep unresolved facts visibly marked.";
  const qaSummary = `${section.title} must tie ${focus} claims to ${evidence}, name ownership, and avoid unsupported certainty.`;
  const humanizedAngle = `Make ${section.title} sound like a responsible subject-matter owner wrote it: specific nouns, concrete verbs, and no generic AI filler.`;
  const reviewHandoff = `${owner} should verify the ${focus}, fill missing facts, and decide whether this section can be marked human-reviewed.`;
  return {
    title: section.title,
    level: section.level,
    qaFocus: focus,
    draftingBrief,
    contextBridge,
    stagePlan: [
      {
        id: "draft",
        label: "Draft body",
        status: "complete",
        detail: `${draftingBrief} Context used: ${contextBridge}`,
      },
      {
        id: "qa",
        label: "QA pass",
        status: "needs-review",
        detail: qaSummary,
      },
      {
        id: "humanize",
        label: "Humanize prose",
        status: "needs-review",
        detail: humanizedAngle,
      },
      {
        id: "review",
        label: "Prepare review",
        status: "needs-review",
        detail: reviewHandoff,
      },
    ],
    qaChecks: [
      `${section.title} makes one clear point before adding detail.`,
      `Claims are tied to ${evidence}, a named owner, a date, or a citation.`,
      `The section explains what ${owner} should do next.`,
    ],
    qaSummary,
    humanizationNotes: [
      "Replace generic claims with named facts, numbers, teams, customers, dates, or examples.",
      "Cut filler phrases, repeated framing, and any sentence that sounds like a prompt response.",
      "Keep the cadence natural: short setup, specific evidence, then a concrete implication.",
    ],
    humanizedAngle,
    reviewQuestions: [
      `Does ${section.title} answer the reader's likely first question?`,
      "What is still unverified and should remain marked before approval?",
    ],
    reviewHandoff,
  };
}

function draftSection(
  section: DocsLiveSectionDraft,
  index: number,
  total: number,
  blueprint: DocsLiveBlueprint,
  placeholders: Record<string, string>,
  contextSentences: string[],
  draftingDepth: DocsLiveDraftDepth,
) {
  const level = Math.min(6, Math.max(2, section.level + 1));
  const childLevel = Math.min(6, level + 1);
  const audience = placeholders.audience || "the intended reader";
  const owner = placeholders.owner || placeholders.reviewer || "[owner]";
  const deadline = placeholders.deadline || placeholders.date || "[date]";
  const subject = placeholders.client || placeholders.company || placeholders.customer || placeholders.product || placeholders.goal || blueprint.label.toLowerCase();
  const context = contextSentences[index % Math.max(1, contextSentences.length)] || "Use the provided outline and replace placeholders with verified facts.";
  const promptSummary = sanitizeMarkerValue(`Drafted ${section.title} section ${index + 1} of ${total}`);
  const body = sectionBodyParagraphs(section, index, subject, audience, context, placeholders, contextSentences, draftingDepth);
  return [
    docsLiveReviewMarker(promptSummary),
    `${"#".repeat(level)} ${section.title}`,
    "",
    `**Drafting brief.** ${section.draftingBrief}`,
    "",
    ...body.flatMap((paragraph) => [paragraph, ""]),
    `${"#".repeat(childLevel)} Section QA`,
    "",
    `${section.qaSummary}`,
    "",
    ...section.qaChecks.map((check) => `- [ ] ${check}`),
    `- [ ] Owner and timing are explicit: ${owner}; ${deadline}.`,
    "",
    `${"#".repeat(childLevel)} Humanization Pass`,
    "",
    `${section.humanizedAngle}`,
    "",
    ...section.humanizationNotes.map((note) => `- [ ] ${note}`),
    "",
    `${"#".repeat(childLevel)} Review Handoff`,
    "",
    `${section.reviewHandoff}`,
    "",
    ...section.reviewQuestions.map((question) => `- ${question}`),
    "",
  ];
}

function docsLiveReviewMarker(promptSummary: string) {
  return `<!-- ai-assisted: status=needs-review | reviewedBy= | reviewedAt= | source=NEditor Docs Live | promptSummary=${sanitizeMarkerValue(promptSummary)} -->`;
}

function sectionBodyParagraphs(
  section: DocsLiveSectionDraft,
  index: number,
  subject: string,
  audience: string,
  context: string,
  placeholders: Record<string, string>,
  contextSentences: string[],
  draftingDepth: DocsLiveDraftDepth,
) {
  const facts = factSentence(placeholders);
  const first = `For ${subject}, ${section.title.toLowerCase()} should give ${audience} a direct read on ${section.qaFocus}. ${context}`;
  const second = facts
    ? `The current working facts are ${facts}. Use them to separate confirmed information from assumptions, then name the decision, risk, or action that follows.`
    : "Replace the bracketed facts with confirmed details, then name the decision, risk, or action that follows.";
  const third =
    contextSentences[(index + 1) % Math.max(1, contextSentences.length)] ||
    "Keep the prose specific enough for review while leaving unresolved claims visibly marked.";
  const fourth = `Before this section is approved, remove unsupported certainty, add citations or calculations for factual claims, and keep only language a responsible human reviewer would stand behind.`;
  if (draftingDepth === "concise") return [first, second];
  if (draftingDepth === "detailed") return [first, second, third, fourth];
  return [first, second, fourth];
}

function factSentence(placeholders: Record<string, string>) {
  const entries = Object.entries(placeholders).filter(([key]) => !["tone", "reviewer", "approver"].includes(key));
  if (!entries.length) return "";
  return entries
    .slice(0, 6)
    .map(([key, value]) => `${titleCase(key)}: ${value}`)
    .join("; ");
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
  const normalizedLines = input
    .split(/\r?\n/)
    .flatMap((line) => line.split(/(?<=[.!?])\s+/))
    .map((sentence) =>
      sentence
        .replace(/^\s*(?:\d+[.)]\s*)?(?:q\d*[:.)]\s*)?/i, "")
        .replace(/\s+/g, " ")
        .trim(),
    )
    .filter((sentence) => sentence.length > 16)
    .slice(0, 12);
  return normalizedLines;
}

function humanizeDraftText(markdown: string) {
  return markdown
    .replace(/\bas an ai(?: language model)?[:,]?\s*/gi, "")
    .replace(/\bit is important to note that\s*/gi, "")
    .replace(/\bin today'?s fast[- ]paced (?:world|environment),?\s*/gi, "")
    .replace(/\bdelve into\b/gi, "examine")
    .replace(/\bnavigate the complexities of\b/gi, "work through")
    .replace(/\bcomprehensive\b/gi, "complete")
    .replace(/\butilize\b/gi, "use")
    .replace(/\bleverage\b/gi, "use")
    .replace(/\brobust\b/gi, "clear")
    .replace(/\bseamless\b/gi, "smooth")
    .replace(/[ \t]{2,}/g, " ")
    .replace(/\n{3,}/g, "\n\n")
    .trimEnd()
    .concat("\n");
}

function normalizeDraftingDepth(value?: string): DocsLiveDraftDepth {
  if (value === "concise" || value === "standard" || value === "detailed") return value;
  return "standard";
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
