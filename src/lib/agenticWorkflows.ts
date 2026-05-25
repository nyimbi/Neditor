import { buildDocsLiveDraft, docsLiveDocumentTypes, normalizeDocsLiveDocumentType, type DocsLiveDocumentType } from "./docsLive.js";
import { outlinePlanFromMarkdown } from "./documentOutline.js";
import type { ExportTarget } from "./workspacePersistence.js";

export type AgenticWorkflowLane = "create" | "compose" | "edit" | "revise" | "review" | "distribute";
export type AgenticWorkflowAction =
  | "open-docs-live"
  | "generate-docs-live-draft"
  | "open-outline"
  | "open-ai-paste"
  | "open-review"
  | "prepare-export"
  | "open-exports";

export interface AgenticWorkflowRequest {
  instruction: string;
  documentTitle?: string;
  documentText?: string;
  selectedText?: string;
}

export interface AgenticWorkflowStep {
  id: string;
  lane: AgenticWorkflowLane;
  title: string;
  detail: string;
  action: AgenticWorkflowAction;
  status: "ready" | "needs-input";
}

export interface AgenticWorkflowPlan {
  instruction: string;
  title: string;
  documentType: DocsLiveDocumentType;
  primaryLane: AgenticWorkflowLane;
  lanes: AgenticWorkflowLane[];
  suggestedOutline: string;
  context: string;
  placeholderText: string;
  revisionInstruction: string;
  distributionTargets: ExportTarget[];
  missingInputs: string[];
  steps: AgenticWorkflowStep[];
}

export interface AgenticWorkflowRunRequest extends AgenticWorkflowRequest {
  generatedAt?: string;
}

export interface AgenticWorkflowRevision {
  originalText: string;
  proposedText: string;
  changeSummary: string[];
}

export interface AgenticWorkflowRun {
  plan: AgenticWorkflowPlan;
  summary: string;
  markdown: string;
  applicationMode: "replace-document" | "replace-selection" | "append-packet";
  revision: AgenticWorkflowRevision | null;
  controlCenter: AgenticControlCenter;
  reviewChecklist: string[];
  distributionChecklist: string[];
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  blockers: string[];
}

export interface AgenticDistributionTargetPlan {
  target: ExportTarget;
  label: string;
  purpose: string;
  preflightChecks: string[];
  handoffSteps: string[];
  evidenceRequired: string[];
}

export type AgenticControlStatus = "ready" | "needs-input" | "blocked";
export type AgenticEvidenceStatus = "available" | "missing" | "needs-review";

export interface AgenticControlItem {
  label: string;
  detail: string;
  status: AgenticEvidenceStatus;
}

export interface AgenticNextAction {
  label: string;
  detail: string;
  lane: AgenticWorkflowLane;
  action: AgenticWorkflowAction;
  status: AgenticControlStatus;
}

export interface AgenticControlCenter {
  status: AgenticControlStatus;
  readinessScore: number;
  summary: string;
  nextActions: AgenticNextAction[];
  sourceGrounding: AgenticControlItem[];
  governance: AgenticControlItem[];
  distribution: AgenticControlItem[];
}

const exportSignals: Array<[ExportTarget, RegExp]> = [
  ["html", /\bhtml|website|web page|landing page\b/i],
  ["pdf", /\bpdf|print|signed|board pack\b/i],
  ["docx", /\bdocx|word\b/i],
  ["pptx", /\bpptx|powerpoint|slides?|deck\b/i],
  ["markdown-bundle", /\bmarkdown bundle|source package|bundle\b/i],
  ["blog", /\bblog|cms|wordpress\b/i],
  ["substack", /\bsubstack|newsletter\b/i],
  ["latex", /\blatex|academic|paper\b/i],
  ["google-docs", /\bgoogle docs?|gdoc|collaborative review\b/i],
];

const laneSignals: Array<[AgenticWorkflowLane, RegExp]> = [
  ["create", /\b(create|draft|write|new|start|generate|compose)\b/i],
  ["compose", /\b(outline|section by section|flesh out|compose|expand)\b/i],
  ["edit", /\b(edit|change|insert|delete|replace|add|remove)\b/i],
  ["revise", /\b(revise|rewrite|shorten|expand|simplify|humanize|tone|polish|make it)\b/i],
  ["review", /\b(review|qa|quality|proof|fact.?check|citations?|approval|risk|governance)\b/i],
  ["distribute", /\b(export|publish|send|distribute|deliver|package|substack|blog|google docs?|pdf|docx|pptx|latex|html)\b/i],
];

const defaultOutlineByLane: Record<AgenticWorkflowLane, string[]> = {
  create: ["Executive Summary", "Context", "Recommendation", "Risks", "Next Steps"],
  compose: ["Executive Summary", "Section Work Queue", "Quality Checks", "Review Handoff"],
  edit: ["Change Intent", "Affected Sections", "Proposed Edits", "Review Notes"],
  revise: ["Revision Goals", "Tone and Audience", "Proposed Rewrite", "Human Review"],
  review: ["Review Summary", "Open Questions", "Risks", "Required Fixes", "Approval Checklist"],
  distribute: ["Distribution Summary", "Target Channels", "Preflight Checks", "Publishing Handoff"],
};

const placeholderNames = ["audience", "owner", "deadline", "tone", "evidence", "reviewer"];

export function buildAgenticWorkflowPlan(request: AgenticWorkflowRequest): AgenticWorkflowPlan {
  const instruction = request.instruction.trim();
  const corpus = [instruction, request.documentTitle, request.documentText, request.selectedText].filter(Boolean).join("\n");
  const lanes = detectLanes(corpus);
  const primaryLane = lanes[0] || "create";
  const documentType = normalizeDocsLiveDocumentType(corpus);
  const title = inferTitle(request, documentType, primaryLane);
  const distributionTargets = detectDistributionTargets(corpus);
  const context = buildContext(request, lanes, distributionTargets);
  const placeholderText = buildPlaceholderText(corpus, title, lanes, distributionTargets);
  const suggestedOutline = buildSuggestedOutline(request, primaryLane, documentType);
  const missingInputs = buildMissingInputs(corpus, lanes, distributionTargets);
  const revisionInstruction = buildRevisionInstruction(instruction, lanes, request.selectedText);
  const steps = buildPlanSteps(lanes, missingInputs, distributionTargets, Boolean(request.documentText?.trim()), Boolean(request.selectedText?.trim()));

  return {
    instruction,
    title,
    documentType,
    primaryLane,
    lanes,
    suggestedOutline,
    context,
    placeholderText,
    revisionInstruction,
    distributionTargets,
    missingInputs,
    steps,
  };
}

export function buildAgenticWorkflowRun(request: AgenticWorkflowRunRequest): AgenticWorkflowRun {
  const plan = buildAgenticWorkflowPlan(request);
  const hasSelection = Boolean(request.selectedText?.trim());
  const hasDocument = Boolean(request.documentText?.trim());
  const revision = plan.lanes.some((lane) => lane === "edit" || lane === "revise") ? buildRevision(request, plan) : null;
  const draft = plan.lanes.some((lane) => lane === "create" || lane === "compose")
    ? buildDocsLiveDraft({
        documentType: plan.documentType,
        title: plan.title,
        outline: plan.suggestedOutline,
        context: plan.context,
        questionnaireAnswers: plan.missingInputs.length
          ? `Unresolved agent inputs:\n${plan.missingInputs.map((input) => `- ${input}`).join("\n")}`
          : "",
        placeholders: plan.placeholderText,
        draftingDepth: plan.primaryLane === "compose" ? "detailed" : "standard",
        generatedAt: request.generatedAt,
      })
    : null;
  const reviewChecklist = buildReviewChecklist(plan, revision);
  const distributionTargetPlans = buildDistributionTargetPlans(plan);
  const distributionChecklist = buildDistributionChecklist(plan, distributionTargetPlans);
  const blockers = buildRunBlockers(plan, hasDocument, hasSelection);
  const applicationMode = inferApplicationMode(plan, hasDocument, hasSelection);
  const controlCenter = buildControlCenter({ plan, blockers, hasDocument, hasSelection, revision, distributionTargetPlans });
  const markdown = buildRunMarkdown({
    plan,
    draftMarkdown: draft?.markdown || "",
    revision,
    controlCenter,
    reviewChecklist,
    distributionChecklist,
    distributionTargetPlans,
    blockers,
    generatedAt: request.generatedAt || new Date().toISOString(),
  });

  return {
    plan,
    summary: `Agent run prepared ${plan.lanes.map(titleCase).join(", ")} for ${plan.title}.`,
    markdown,
    applicationMode,
    revision,
    controlCenter,
    reviewChecklist,
    distributionChecklist,
    distributionTargetPlans,
    blockers,
  };
}

function detectLanes(corpus: string): AgenticWorkflowLane[] {
  const detected = laneSignals.flatMap(([lane, signal]) => (signal.test(corpus) ? [lane] : []));
  if (!detected.length) return ["create", "review"];
  const ordered: AgenticWorkflowLane[] = [];
  for (const lane of ["create", "compose", "edit", "revise", "review", "distribute"] as const) {
    if (detected.includes(lane)) ordered.push(lane);
  }
  if ((ordered.includes("create") || ordered.includes("compose") || ordered.includes("revise")) && !ordered.includes("review")) ordered.push("review");
  if (ordered.includes("distribute") && !ordered.includes("review")) ordered.splice(Math.max(0, ordered.indexOf("distribute")), 0, "review");
  return ordered;
}

function detectDistributionTargets(corpus: string): ExportTarget[] {
  const targets = exportSignals.flatMap(([target, signal]) => (signal.test(corpus) ? [target] : []));
  return Array.from(new Set(targets));
}

function buildRevision(request: AgenticWorkflowRunRequest, plan: AgenticWorkflowPlan): AgenticWorkflowRevision {
  const originalText = (request.selectedText || request.documentText || "").trim();
  const proposedText = reviseText(originalText, plan.revisionInstruction, plan.placeholderText);
  const changeSummary = [
    "Preserved the user's intent while making the requested revision explicit.",
    "Added AI-assisted review metadata so the change remains governable before export.",
    plan.lanes.includes("review") ? "Prepared QA prompts for evidence, tone, and reviewer sign-off." : "",
  ].filter(Boolean);

  return {
    originalText,
    proposedText,
    changeSummary,
  };
}

function reviseText(text: string, instruction: string, placeholders: string) {
  const cleaned = humanizeText(text || "Draft the requested change here with verified facts and named owners.");
  const lowerInstruction = instruction.toLowerCase();
  let proposed = cleaned;
  if (/\b(shorten|concise|brief|crisp|executive)\b/.test(lowerInstruction)) {
    proposed = shortenText(proposed);
  }
  if (/\b(expand|detail|flesh out|elaborate)\b/.test(lowerInstruction)) {
    proposed = expandText(proposed, placeholders);
  }
  if (/\b(cfo|finance|financial|budget|investment|roi)\b/.test(lowerInstruction)) {
    proposed = addFinanceFrame(proposed);
  } else if (/\b(board|executive|ceo|leadership)\b/.test(lowerInstruction)) {
    proposed = addExecutiveFrame(proposed);
  }
  if (/\b(humanize|natural|less ai|plain language|non-technical)\b/.test(lowerInstruction)) {
    proposed = humanizeText(proposed);
  }
  return [
    "<!-- ai-assisted: status=needs-review | reviewedBy= | reviewedAt= | source=NEditor Agent Workspace | promptSummary=Agentic revision proposal -->",
    "",
    proposed.trim(),
  ].join("\n");
}

function humanizeText(text: string) {
  return text
    .replace(/\b(it is important to note that|in today's fast-paced world|leveraging|robust|seamlessly|delve into)\b/gi, "")
    .replace(/\butilize\b/gi, "use")
    .replace(/\bvarious\b/gi, "specific")
    .replace(/[ \t]{2,}/g, " ")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function shortenText(text: string) {
  const sentences = text.split(/(?<=[.!?])\s+/).filter(Boolean);
  if (sentences.length <= 2) return text;
  return sentences.slice(0, 2).join(" ");
}

function expandText(text: string, placeholders: string) {
  const evidence = extractKeyValue(placeholders, "evidence") || "the strongest verified evidence";
  const owner = extractKeyValue(placeholders, "owner") || "the accountable owner";
  return [
    text,
    "",
    `This section should be completed with ${evidence}, a named implication for the reader, and a clear owner: ${owner}.`,
  ].join("\n");
}

function addFinanceFrame(text: string) {
  return `Finance review focus: state the investment, cost, risk, and measurable return before the recommendation.\n\n${text}`;
}

function addExecutiveFrame(text: string) {
  return `Decision focus: make the recommendation, the reason to act now, and the consequence of waiting clear in the first paragraph.\n\n${text}`;
}

function inferTitle(request: AgenticWorkflowRequest, documentType: DocsLiveDocumentType, lane: AgenticWorkflowLane) {
  const explicit = request.instruction.match(/\b(?:called|titled|title)\s+["']?([^"'\n.]{4,80})["']?/i)?.[1]?.trim();
  if (explicit) return explicit;
  if (request.documentTitle?.trim()) return request.documentTitle.replace(/\.[^.]+$/, "");
  const typeLabel = docsLiveDocumentTypes.find((type) => type.id === documentType)?.label || "Document";
  return `${titleCase(lane)} ${typeLabel}`;
}

function buildContext(request: AgenticWorkflowRequest, lanes: AgenticWorkflowLane[], targets: ExportTarget[]) {
  return [
    `User intent: ${request.instruction.trim() || "Create and improve the current document."}`,
    request.documentTitle ? `Current document: ${request.documentTitle}` : "",
    request.selectedText?.trim() ? `Selected text to act on: ${request.selectedText.trim().slice(0, 1200)}` : "",
    request.documentText?.trim() ? `Current document context available: ${Math.min(request.documentText.trim().length, 2000)} characters.` : "No current document body supplied.",
    lanes.length ? `Agent lanes requested: ${lanes.map(titleCase).join(", ")}.` : "",
    targets.length ? `Distribution targets requested: ${targets.join(", ")}.` : "",
  ]
    .filter(Boolean)
    .join("\n");
}

function buildPlaceholderText(corpus: string, title: string, lanes: AgenticWorkflowLane[], targets: ExportTarget[]) {
  const values: Record<string, string> = {
    title,
    audience: extractKeyValue(corpus, "audience") || "TBD audience",
    owner: extractKeyValue(corpus, "owner") || "TBD owner",
    deadline: extractKeyValue(corpus, "deadline") || "TBD deadline",
    tone: extractKeyValue(corpus, "tone") || inferTone(corpus),
    evidence: extractKeyValue(corpus, "evidence") || "TBD evidence",
    workflow: lanes.join(", "),
  };
  if (targets.length) values.distribution = targets.join(", ");
  return Object.entries(values)
    .map(([key, value]) => `${key}: ${value}`)
    .join("\n");
}

function buildSuggestedOutline(request: AgenticWorkflowRequest, primaryLane: AgenticWorkflowLane, documentType: DocsLiveDocumentType) {
  const existingOutline = request.documentText ? outlinePlanFromMarkdown(request.documentText) : "";
  if (existingOutline && primaryLane !== "create") return existingOutline;
  const typeLabel = docsLiveDocumentTypes.find((type) => type.id === documentType)?.label || titleCase(primaryLane);
  const sections = defaultOutlineByLane[primaryLane] || defaultOutlineByLane.create;
  return [`- ${typeLabel}`, ...sections.map((section) => `  - ${section}`)].join("\n");
}

function buildMissingInputs(corpus: string, lanes: AgenticWorkflowLane[], targets: ExportTarget[]) {
  const missing = placeholderNames.filter((name) => !new RegExp(`\\b${name}\\b\\s*(?:is|=|:)`, "i").test(corpus));
  if (lanes.includes("review") && !/\b(source|evidence|citation|reference|data)\b/i.test(corpus)) missing.push("source evidence or citation expectations");
  if (targets.length && !/\b(status|approval|approved|reviewer)\b/i.test(corpus)) missing.push("approval status for distribution");
  return Array.from(new Set(missing));
}

function buildRevisionInstruction(instruction: string, lanes: AgenticWorkflowLane[], selectedText?: string) {
  if (lanes.includes("revise")) return instruction;
  if (lanes.includes("edit")) return `Apply these edit instructions carefully: ${instruction}`;
  if (selectedText?.trim()) return `Improve the selected text while preserving intent: ${instruction}`;
  return `Prepare the document for human review, then identify the highest-value revision pass: ${instruction}`;
}

function buildPlanSteps(
  lanes: AgenticWorkflowLane[],
  missingInputs: string[],
  targets: ExportTarget[],
  hasDocument: boolean,
  hasSelection: boolean,
): AgenticWorkflowStep[] {
  const steps: AgenticWorkflowStep[] = [];
  if (lanes.includes("create")) {
    steps.push({
      id: "intent",
      lane: "create",
      title: "Capture intent and missing context",
      detail: missingInputs.length ? `Ask for or mark missing inputs: ${missingInputs.join(", ")}.` : "Intent has enough context for a first draft.",
      action: "open-docs-live",
      status: missingInputs.length ? "needs-input" : "ready",
    });
  }
  if (lanes.includes("compose")) {
    steps.push({
      id: "compose-outline",
      lane: "compose",
      title: "Compose from outline",
      detail: hasDocument ? "Use the current outline as the section work queue." : "Create a document outline before drafting body text.",
      action: hasDocument ? "generate-docs-live-draft" : "open-outline",
      status: "ready",
    });
  }
  if (lanes.includes("edit") || lanes.includes("revise")) {
    steps.push({
      id: "revise",
      lane: lanes.includes("revise") ? "revise" : "edit",
      title: hasSelection ? "Revise selected text" : "Plan revision pass",
      detail: hasSelection ? "Apply the requested change to the selected text and preview the result." : "Use AI Paste cleanup or Docs Live to propose a tracked rewrite.",
      action: "open-ai-paste",
      status: "ready",
    });
  }
  if (lanes.includes("review")) {
    steps.push({
      id: "review",
      lane: "review",
      title: "Run review readiness",
      detail: "Check comments, AI provenance, evidence gaps, QA notes, and human-review status.",
      action: "open-review",
      status: "ready",
    });
  }
  if (lanes.includes("distribute")) {
    steps.push({
      id: "distribution",
      lane: "distribute",
      title: "Prepare distribution",
      detail: targets.length ? `Prepare export readiness for ${targets.join(", ")}.` : "Choose target channels, then run export readiness.",
      action: targets.length ? "prepare-export" : "open-exports",
      status: "ready",
    });
  }
  return steps.length ? steps : buildPlanSteps(["create", "review"], missingInputs, targets, hasDocument, hasSelection);
}

function buildReviewChecklist(plan: AgenticWorkflowPlan, revision: AgenticWorkflowRevision | null) {
  return [
    "Confirm the document has a clear audience, decision, owner, deadline, and review status.",
    "Replace unresolved placeholders with verified names, dates, numbers, and source references.",
    "Check each factual claim against the evidence or add a citation TODO before export.",
    "Mark every AI source block and AI-assisted section human-reviewed only after a person verifies it.",
    revision ? "Compare the revision proposal against the original text before applying final edits." : "",
    plan.distributionTargets.length ? "Run export readiness for every requested distribution target." : "",
  ].filter(Boolean);
}

function buildDistributionTargetPlans(plan: AgenticWorkflowPlan): AgenticDistributionTargetPlan[] {
  return plan.distributionTargets.map((target) => {
    const profile = distributionProfile(target);
    return {
      target,
      label: profile.label,
      purpose: profile.purpose,
      preflightChecks: [
        "Confirm approval status, reviewer, approvedAt, and export metadata are present.",
        "Run export readiness and resolve unresolved comments, AI review warnings, broken links, and missing assets.",
        ...profile.preflightChecks,
      ],
      handoffSteps: profile.handoffSteps,
      evidenceRequired: [
        "Export manifest with target, version, source path, generatedAt, byte size, and SHA-256 where available.",
        "Human reviewer note or checklist entry confirming the delivered artifact was inspected.",
        ...profile.evidenceRequired,
      ],
    };
  });
}

function buildDistributionChecklist(plan: AgenticWorkflowPlan, targetPlans: AgenticDistributionTargetPlan[]) {
  if (!plan.distributionTargets.length) {
    return ["Select distribution targets, then run export readiness before release."];
  }
  return targetPlans.flatMap((targetPlan) => [
    `${targetPlan.label}: ${targetPlan.preflightChecks[0]}`,
    `${targetPlan.label}: ${targetPlan.handoffSteps[0]}`,
    `${targetPlan.label}: retain ${targetPlan.evidenceRequired[0].toLowerCase()}`,
  ]);
}

function buildControlCenter(input: {
  plan: AgenticWorkflowPlan;
  blockers: string[];
  hasDocument: boolean;
  hasSelection: boolean;
  revision: AgenticWorkflowRevision | null;
  distributionTargetPlans: AgenticDistributionTargetPlan[];
}): AgenticControlCenter {
  const { plan, blockers, hasDocument, hasSelection, revision, distributionTargetPlans } = input;
  const hardBlockers = blockers.filter((blocker) => !blocker.startsWith("Missing input:"));
  const status: AgenticControlStatus = hardBlockers.length ? "blocked" : blockers.length ? "needs-input" : "ready";
  const sourceGrounding = buildSourceGrounding(plan, hasDocument, hasSelection);
  const governance = buildGovernanceItems(plan, revision, blockers);
  const distribution = buildDistributionItems(plan, distributionTargetPlans);
  const readinessScore = scoreControlCenter(status, sourceGrounding, governance, distribution, blockers);
  const nextActions = buildNextActions(plan, status, blockers, distributionTargetPlans);
  const summary =
    status === "blocked"
      ? "Agent run is blocked until source context or target instructions are supplied."
      : status === "needs-input"
        ? "Agent run is usable as a draft packet, but missing inputs must be resolved before approval or distribution."
        : "Agent run is ready for governed drafting, review, and target-specific distribution prep.";

  return {
    status,
    readinessScore,
    summary,
    nextActions,
    sourceGrounding,
    governance,
    distribution,
  };
}

function buildSourceGrounding(plan: AgenticWorkflowPlan, hasDocument: boolean, hasSelection: boolean): AgenticControlItem[] {
  const evidenceValue = extractKeyValue(plan.placeholderText, "evidence");
  return [
    {
      label: "User instruction",
      detail: plan.instruction ? "Plain-language intent captured as the agent objective." : "No explicit instruction was supplied.",
      status: plan.instruction ? "available" : "missing",
    },
    {
      label: "Current document",
      detail: hasDocument ? "Current Markdown body is included in the context pack." : "No current document body is available to ground the run.",
      status: hasDocument ? "available" : "needs-review",
    },
    {
      label: "Selected text",
      detail: hasSelection ? "Selection is available for precise edit/revision work." : "No selected text was supplied for selection-aware edits.",
      status: hasSelection ? "available" : plan.lanes.some((lane) => lane === "edit" || lane === "revise") ? "missing" : "needs-review",
    },
    {
      label: "Outline",
      detail: plan.suggestedOutline.trim() ? "Outline is available as the composition work queue." : "No outline is available for section-by-section drafting.",
      status: plan.suggestedOutline.trim() ? "available" : "missing",
    },
    {
      label: "Evidence",
      detail:
        evidenceValue && !/^TBD\b/i.test(evidenceValue)
          ? `Evidence expectation captured: ${evidenceValue}.`
          : "Evidence is not yet specific enough for final claims.",
      status: evidenceValue && !/^TBD\b/i.test(evidenceValue) ? "available" : "needs-review",
    },
  ];
}

function buildGovernanceItems(plan: AgenticWorkflowPlan, revision: AgenticWorkflowRevision | null, blockers: string[]): AgenticControlItem[] {
  return [
    {
      label: "AI provenance",
      detail: "Agent output includes ai-source and ai-assisted review metadata.",
      status: "available",
    },
    {
      label: "Human review",
      detail: blockers.length ? "Human review remains blocked by missing inputs or workflow constraints." : "Reviewer can inspect QA gates before marking sections human-reviewed.",
      status: blockers.length ? "needs-review" : "available",
    },
    {
      label: "Revision audit",
      detail: revision ? "Original text, proposed text, and change summary are captured for comparison." : "No selection-aware revision is part of this run.",
      status: revision ? "available" : "needs-review",
    },
    {
      label: "Approval metadata",
      detail: plan.distributionTargets.length ? "Approval status, reviewer, and approvedAt must be confirmed before distribution." : "Distribution approval metadata is not required until a target is selected.",
      status: plan.distributionTargets.length ? "needs-review" : "available",
    },
  ];
}

function buildDistributionItems(plan: AgenticWorkflowPlan, targetPlans: AgenticDistributionTargetPlan[]): AgenticControlItem[] {
  if (!plan.distributionTargets.length) {
    return [
      {
        label: "Distribution targets",
        detail: "No export or publishing target is selected yet.",
        status: "needs-review",
      },
    ];
  }
  return targetPlans.map((targetPlan) => ({
    label: targetPlan.label,
    detail: `${targetPlan.preflightChecks.length} preflight checks, ${targetPlan.handoffSteps.length} handoff step, and ${targetPlan.evidenceRequired.length} evidence requirements are staged.`,
    status: "needs-review" as const,
  }));
}

function buildNextActions(
  plan: AgenticWorkflowPlan,
  status: AgenticControlStatus,
  blockers: string[],
  targetPlans: AgenticDistributionTargetPlan[],
): AgenticNextAction[] {
  const actions: AgenticNextAction[] = [];
  if (blockers.length) {
    actions.push({
      label: "Resolve missing inputs",
      detail: blockers.slice(0, 4).join("; "),
      lane: "create",
      action: "open-docs-live",
      status: "needs-input",
    });
  }
  for (const step of plan.steps) {
    actions.push({
      label: step.title,
      detail: step.detail,
      lane: step.lane,
      action: step.action,
      status: step.status === "needs-input" ? "needs-input" : status === "blocked" ? "blocked" : "ready",
    });
  }
  if (targetPlans.length) {
    actions.push({
      label: "Verify target artifacts",
      detail: `Retain evidence for ${targetPlans.map((target) => target.label).join(", ")} before publication.`,
      lane: "distribute",
      action: "prepare-export",
      status: "needs-input",
    });
  }
  return actions.slice(0, 8);
}

function scoreControlCenter(
  status: AgenticControlStatus,
  sourceGrounding: AgenticControlItem[],
  governance: AgenticControlItem[],
  distribution: AgenticControlItem[],
  blockers: string[],
) {
  const allItems = [...sourceGrounding, ...governance, ...distribution];
  const missingPenalty = allItems.filter((item) => item.status === "missing").length * 16;
  const reviewPenalty = allItems.filter((item) => item.status === "needs-review").length * 7;
  const blockerPenalty = blockers.length * 8;
  const statusPenalty = status === "blocked" ? 20 : status === "needs-input" ? 8 : 0;
  return Math.max(0, Math.min(100, 100 - missingPenalty - reviewPenalty - blockerPenalty - statusPenalty));
}

function distributionProfile(target: ExportTarget) {
  const profiles: Record<
    ExportTarget,
    {
      label: string;
      purpose: string;
      preflightChecks: string[];
      handoffSteps: string[];
      evidenceRequired: string[];
    }
  > = {
    html: {
      label: "HTML review copy",
      purpose: "Browser-readable review, static publishing, and lightweight stakeholder circulation.",
      preflightChecks: ["Set language, title, canonical URL if publishing, and visible export stylesheet."],
      handoffSteps: ["Export standalone HTML, open it in a browser, and confirm headings, tables, links, and AI provenance render correctly."],
      evidenceRequired: ["Browser screenshot or reviewer note for the generated HTML file."],
    },
    pdf: {
      label: "PDF controlled copy",
      purpose: "Board packs, signed approvals, print review, and fixed-layout circulation.",
      preflightChecks: ["Confirm page size, cover, headers, footers, page numbers, watermark, and approval metadata."],
      handoffSteps: ["Export PDF, inspect page count and text extraction, then send only after status is approved or published."],
      evidenceRequired: ["PDF metadata/text proof or native-viewer sign-off for the final PDF."],
    },
    docx: {
      label: "DOCX editable review",
      purpose: "Word-based redlines, legal review, and stakeholder edits outside NEditor.",
      preflightChecks: ["Confirm comments, change notes, AI provenance appendix, bibliography, and table formatting are review-ready."],
      handoffSteps: ["Export DOCX, open in Word or an approved viewer, and ask reviewers to preserve tracked-review context."],
      evidenceRequired: ["DOCX viewer note or extracted text proof for key sections and appendices."],
    },
    pptx: {
      label: "PPTX executive handoff",
      purpose: "Slide-based executive review and presentation-outline handoff.",
      preflightChecks: ["Confirm agenda flow, section titles, speaker notes, figures, and decision slides."],
      handoffSteps: ["Export PPTX, open the deck, and confirm each generated slide matches the intended narrative."],
      evidenceRequired: ["PPTX viewer note or Office preview proof for the generated deck."],
    },
    "markdown-bundle": {
      label: "Markdown source bundle",
      purpose: "Auditable source package for teams that review or archive Markdown and assets.",
      preflightChecks: ["Confirm included files, assets, transform artifacts, and manifest entries are complete."],
      handoffSteps: ["Export the bundle, inspect the manifest, and archive it with the review record."],
      evidenceRequired: ["Bundle manifest proof listing source files, assets, and transform artifacts."],
    },
    blog: {
      label: "Blog publishing package",
      purpose: "CMS or blog handoff with Markdown, HTML, text, assets, RSS seed, and publish metadata.",
      preflightChecks: ["Confirm slug, excerpt, tags, canonical URL, images, alt text, and publish workflow metadata."],
      handoffSteps: ["Export the blog package, copy the prepared content into the CMS, and keep the package manifest with the approval record."],
      evidenceRequired: ["CMS preview note or package manifest with post.md, post.html, post.txt, and rss-item.xml."],
    },
    substack: {
      label: "Substack newsletter package",
      purpose: "Newsletter handoff with Substack-safe HTML, Markdown, text, assets, and publish metadata.",
      preflightChecks: ["Confirm subject line, preview text, subscriber context, links, images, and call to action."],
      handoffSteps: ["Export the Substack package, paste substack-copy.html or Markdown into Substack, and send a test preview before scheduling."],
      evidenceRequired: ["Substack preview note or package manifest with substack-copy.html and publish metadata."],
    },
    latex: {
      label: "LaTeX source export",
      purpose: "Academic, technical, or formal typesetting handoff with inspectable TeX source.",
      preflightChecks: ["Confirm equations, cross references, bibliography, labels, and document metadata compile cleanly."],
      handoffSteps: ["Export LaTeX, compile with the approved TeX toolchain, and inspect warnings before sharing the PDF."],
      evidenceRequired: ["TeX compile log summary and generated PDF hash when available."],
    },
    "google-docs": {
      label: "Google Docs collaboration package",
      purpose: "Google Docs import handoff for collaborative review while preserving a local source of truth.",
      preflightChecks: ["Confirm DOCX, HTML, Markdown, text, assets, import metadata, and unresolved blockers are ready."],
      handoffSteps: ["Export the Google Docs package, import document.docx into Google Docs, read back required text markers, and keep the Drive URL in the review record."],
      evidenceRequired: ["Google Drive import/readback evidence with imported document URL and exported DOCX hash."],
    },
  };
  return profiles[target];
}

function buildRunBlockers(plan: AgenticWorkflowPlan, hasDocument: boolean, hasSelection: boolean) {
  const blockers = [...plan.missingInputs.map((input) => `Missing input: ${input}`)];
  if ((plan.lanes.includes("edit") || plan.lanes.includes("revise")) && !hasDocument && !hasSelection) {
    blockers.push("Revision requested but no document or selection was supplied.");
  }
  if (plan.lanes.includes("distribute") && !plan.distributionTargets.length) {
    blockers.push("Distribution requested but no export or publishing target was identified.");
  }
  return Array.from(new Set(blockers));
}

function inferApplicationMode(plan: AgenticWorkflowPlan, hasDocument: boolean, hasSelection: boolean): AgenticWorkflowRun["applicationMode"] {
  if (hasSelection && (plan.lanes.includes("edit") || plan.lanes.includes("revise"))) return "replace-selection";
  if ((plan.lanes.includes("create") || plan.lanes.includes("compose")) && !hasDocument) return "replace-document";
  if (plan.lanes.includes("create") && !plan.lanes.includes("edit") && !plan.lanes.includes("revise")) return "replace-document";
  return "append-packet";
}

function buildRunMarkdown(input: {
  plan: AgenticWorkflowPlan;
  draftMarkdown: string;
  revision: AgenticWorkflowRevision | null;
  controlCenter: AgenticControlCenter;
  reviewChecklist: string[];
  distributionChecklist: string[];
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  blockers: string[];
  generatedAt: string;
}) {
  const { plan, draftMarkdown, revision, controlCenter, reviewChecklist, distributionChecklist, distributionTargetPlans, blockers, generatedAt } = input;
  const lines = [
    "---",
    `title: ${yamlScalar(`${plan.title} Agent Run`)}`,
    "status: draft",
    "toc: true",
    "---",
    "",
    `# ${plan.title} Agent Run`,
    "",
    "```ai-source",
    "provider: NEditor Agent Workspace",
    "model: local-agentic-workflow",
    `date: ${generatedAt}`,
    `promptSummary: ${sanitizeMarkerValue(plan.instruction || "Agentic document workflow")}`,
    "reviewedBy: ",
    "reviewedAt: ",
    "status: needs-review",
    "```",
    "",
    "## Agent Plan",
    "",
    `Primary lane: ${titleCase(plan.primaryLane)}`,
    "",
    `Workflow lanes: ${plan.lanes.map(titleCase).join(" -> ")}`,
    "",
    "### Context Pack",
    "",
    fencedBlock("text", plan.context),
    "",
    "### Placeholders",
    "",
    fencedBlock("yaml", plan.placeholderText),
    "",
    "### Suggested Outline",
    "",
    fencedBlock("text", plan.suggestedOutline),
    "",
  ];

  if (blockers.length) {
    lines.push("### Blockers", "", ...blockers.map((blocker) => `- [ ] ${blocker}`), "");
  }
  lines.push(...controlCenterMarkdown(controlCenter));
  if (draftMarkdown.trim()) {
    lines.push("## Generated Draft", "", draftMarkdown.trim(), "");
  }
  if (revision) {
    lines.push(
      "## Revision Proposal",
      "",
      "### Change Summary",
      "",
      ...revision.changeSummary.map((item) => `- ${item}`),
      "",
      "### Original Text",
      "",
      fencedBlock("markdown", revision.originalText || "(No source text supplied.)"),
      "",
      "### Proposed Text",
      "",
      revision.proposedText,
      "",
    );
  }
  lines.push(
    "## Quality Assurance",
    "",
    ...reviewChecklist.map((item) => `- [ ] ${item}`),
    "",
    "## Distribution",
    "",
    ...distributionChecklist.map((item) => `- [ ] ${item}`),
    "",
    ...(distributionTargetPlans.length ? distributionTargetRunbookMarkdown(distributionTargetPlans) : []),
    "## Human Review Handoff",
    "",
    "A person should verify sources, numbers, tone, reviewer metadata, and export readiness before this agent run is accepted.",
    "",
  );
  return lines.join("\n").replace(/\n{3,}/g, "\n\n").trimEnd() + "\n";
}

function controlCenterMarkdown(controlCenter: AgenticControlCenter) {
  return [
    "## AI Control Center",
    "",
    `Status: ${controlCenter.status}`,
    "",
    `Readiness score: ${controlCenter.readinessScore}/100`,
    "",
    controlCenter.summary,
    "",
    "### Next Actions",
    "",
    ...controlCenter.nextActions.map((action) => `- [ ] ${action.label} (${action.lane}, ${action.status}): ${action.detail}`),
    "",
    "### Source Grounding",
    "",
    ...controlCenter.sourceGrounding.map((item) => `- ${item.label} [${item.status}]: ${item.detail}`),
    "",
    "### Governance",
    "",
    ...controlCenter.governance.map((item) => `- ${item.label} [${item.status}]: ${item.detail}`),
    "",
    "### Distribution State",
    "",
    ...controlCenter.distribution.map((item) => `- ${item.label} [${item.status}]: ${item.detail}`),
    "",
  ];
}

function fencedBlock(language: string, value: string) {
  return ["```" + language, value.trim() || "(empty)", "```"].join("\n");
}

function distributionTargetRunbookMarkdown(targetPlans: AgenticDistributionTargetPlan[]) {
  const lines = ["### Target Runbooks", ""];
  for (const targetPlan of targetPlans) {
    lines.push(
      `#### ${targetPlan.label}`,
      "",
      `Purpose: ${targetPlan.purpose}`,
      "",
      "Preflight:",
      ...targetPlan.preflightChecks.map((item) => `- [ ] ${item}`),
      "",
      "Handoff:",
      ...targetPlan.handoffSteps.map((item) => `- [ ] ${item}`),
      "",
      "Evidence:",
      ...targetPlan.evidenceRequired.map((item) => `- [ ] ${item}`),
      "",
    );
  }
  return lines;
}

function extractKeyValue(corpus: string, key: string) {
  const keys = ["audience", "owner", "deadline", "tone", "evidence", "reviewer", "client", "company", "distribution"];
  const nextKey = keys.filter((item) => item !== key).join("|");
  return corpus.match(new RegExp(`\\b${key}\\s*(?:is|=|:)\\s*([^\\n.]+?)(?=\\s+(?:${nextKey})\\s*(?:is|=|:)|[.\\n]|$)`, "i"))?.[1]?.trim();
}

function sanitizeMarkerValue(value: string) {
  return value.replace(/[\r\n|]+/g, " ").replace(/\s+/g, " ").trim().slice(0, 160);
}

function yamlScalar(value: string) {
  if (/^[A-Za-z0-9 _.,:/-]+$/.test(value)) return value;
  return JSON.stringify(value);
}

function inferTone(corpus: string) {
  if (/\b(board|executive|cfo|ceo)\b/i.test(corpus)) return "executive and decision-oriented";
  if (/\bplain|simple|non-technical\b/i.test(corpus)) return "plain-language";
  if (/\blegal|compliance|risk\b/i.test(corpus)) return "careful and evidence-led";
  return "professional and direct";
}

function titleCase(value: string) {
  return value
    .replace(/[-_]+/g, " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase())
    .trim();
}
