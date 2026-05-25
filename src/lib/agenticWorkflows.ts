import { buildDocsLiveDraft, docsLiveDocumentTypes, normalizeDocsLiveDocumentType, type DocsLiveDocumentType, type DocsLiveDraftDepth } from "./docsLive.js";
import { extractCitationTodoItems } from "./citationTodoWorkflow.js";
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
  contextAnswers?: string;
  sourcePackText?: string;
  memoryText?: string;
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

export interface AgenticLifecycleTask {
  id: string;
  lane: AgenticWorkflowLane;
  title: string;
  owner: string;
  status: AgenticControlStatus;
  action: AgenticWorkflowAction;
  sectionId?: string;
  target?: ExportTarget;
  evidence: string[];
  nextStep: string;
}

export interface AgenticWorkflowPlan {
  instruction: string;
  title: string;
  documentType: DocsLiveDocumentType;
  primaryLane: AgenticWorkflowLane;
  lanes: AgenticWorkflowLane[];
  documentIntent: AgenticDocumentIntentSheet;
  contextAnswers: string;
  sourcePackText: string;
  sourcePack: AgenticSourcePack;
  memoryText: string;
  documentMemory: AgenticDocumentMemory;
  suggestedOutline: string;
  outlineVariants: AgenticOutlineVariant[];
  context: string;
  placeholderText: string;
  contextCompleteness: AgenticContextCompleteness;
  revisionInstruction: string;
  revisionModes: AgenticRevisionMode[];
  qualityGates: AgenticQualityGate[];
  distributionTargets: ExportTarget[];
  missingInputs: string[];
  steps: AgenticWorkflowStep[];
}

export type AgenticDocumentIntentFieldStatus = "provided" | "needs-review" | "missing";

export interface AgenticDocumentIntentField {
  key: string;
  label: string;
  value: string;
  status: AgenticDocumentIntentFieldStatus;
  source: "instruction" | "context" | "source-pack" | "current-document" | "derived" | "missing";
  guidance: string;
}

export interface AgenticDocumentIntentSheet {
  summary: string;
  completenessScore: number;
  status: "ready" | "needs-input";
  fields: AgenticDocumentIntentField[];
  missingFields: string[];
  reviewPrompts: string[];
  markdown: string;
}

export interface AgenticContextCompleteness {
  score: number;
  status: "strong" | "usable" | "thin";
  present: string[];
  missing: string[];
  recommendations: string[];
}

export type AgenticSourcePackItemKind = "note" | "url" | "file" | "reference" | "reviewer-comment" | "claim";

export interface AgenticSourcePackItem {
  id: string;
  kind: AgenticSourcePackItemKind;
  label: string;
  detail: string;
}

export interface AgenticSourcePack {
  items: AgenticSourcePackItem[];
  urls: AgenticSourcePackItem[];
  files: AgenticSourcePackItem[];
  references: AgenticSourcePackItem[];
  reviewerComments: AgenticSourcePackItem[];
  claims: AgenticSourcePackItem[];
  notes: AgenticSourcePackItem[];
  markdown: string;
}

export type AgenticDocumentMemoryKind =
  | "terminology"
  | "style"
  | "accepted-decision"
  | "rejected-direction"
  | "review-preference"
  | "distribution-preference";

export interface AgenticDocumentMemoryEntry {
  id: string;
  kind: AgenticDocumentMemoryKind;
  label: string;
  detail: string;
  source: "user-memory" | "context" | "current-document";
}

export interface AgenticDocumentMemory {
  entries: AgenticDocumentMemoryEntry[];
  terminology: AgenticDocumentMemoryEntry[];
  style: AgenticDocumentMemoryEntry[];
  acceptedDecisions: AgenticDocumentMemoryEntry[];
  rejectedDirections: AgenticDocumentMemoryEntry[];
  reviewPreferences: AgenticDocumentMemoryEntry[];
  distributionPreferences: AgenticDocumentMemoryEntry[];
  markdown: string;
  summary: string;
}

export interface AgenticQualityGate {
  id: string;
  label: string;
  appliesTo: DocsLiveDocumentType | "blog" | "newsletter" | "distribution";
  detail: string;
  evidenceRequired: string[];
}

export interface AgenticWorkflowPlaybook {
  id: string;
  label: string;
  summary: string;
  instruction: string;
  bestFor: string[];
  expectedOutputs: string[];
}

export interface AgenticWorkflowRunRequest extends AgenticWorkflowRequest {
  generatedAt?: string;
}

export interface AgenticWorkflowRevision {
  originalText: string;
  proposedText: string;
  changeSummary: string[];
  revisionPasses: AgenticRevisionPass[];
  meaningDriftFindings: AgenticMeaningDriftFinding[];
}

export interface AgenticEditAcceptanceItem {
  id: string;
  scope: "selection" | "section" | "document";
  heading: string;
  originalText: string;
  proposedText: string;
  changeSummary: string[];
  riskNotes: string[];
  recommendation: string;
}

export type AgenticRevisionMode =
  | "clarity"
  | "brevity"
  | "tone"
  | "evidence"
  | "legal-caution"
  | "executive-summary"
  | "accessibility"
  | "humanization";

export interface AgenticRevisionPass {
  mode: AgenticRevisionMode;
  label: string;
  rationale: string;
  checklist: string[];
}

export interface AgenticMeaningDriftFinding {
  kind: "number" | "date" | "commitment" | "caveat";
  severity: "warning" | "blocker";
  original: string;
  proposed: string;
  detail: string;
  recommendation: string;
}

export interface AgenticDocumentEvidence {
  unresolvedPlaceholders: string[];
  citationTodos: string[];
  claimInventory: AgenticDocumentClaim[];
  humanizationFindings: AgenticHumanizationFinding[];
  reviewCommentResolutions: AgenticReviewCommentResolution[];
  unreviewedAiMarkers: number;
  unresolvedComments: number;
  approvalMetadataMissing: string[];
  brokenLinkHints: string[];
  referenceHints: string[];
}

export interface AgenticDocumentClaim {
  kind: "number" | "date" | "commitment" | "quote" | "claim";
  sourceLine: number;
  text: string;
  reason: string;
}

export interface AgenticHumanizationFinding {
  kind: "generic-phrase" | "overconfident-claim" | "repetition" | "vague-transition";
  sourceLine: number;
  text: string;
  recommendation: string;
}

export interface AgenticReviewCommentResolution {
  id: string;
  line: number;
  author: string;
  createdAt: string;
  excerpt: string;
  requiredAction: string;
  resolutionOptions: string[];
  blocker: boolean;
}

export type AgenticApprovalGateStatus = "ready" | "needs-review" | "blocked";
export type AgenticApprovalGateFieldKey =
  | "status"
  | "reviewer"
  | "approvedAt"
  | "owner"
  | "releaseTarget"
  | "sourceConfidence";

export interface AgenticApprovalGateField {
  key: AgenticApprovalGateFieldKey;
  label: string;
  value: string;
  status: "present" | "missing" | "needs-review";
  guidance: string;
}

export interface AgenticApprovalGate {
  status: AgenticApprovalGateStatus;
  summary: string;
  requiredBeforeDistribution: boolean;
  fields: AgenticApprovalGateField[];
  blockers: string[];
  metadataScaffold: string;
}

export interface AgenticWorkflowRun {
  plan: AgenticWorkflowPlan;
  summary: string;
  markdown: string;
  applicationMode: "replace-document" | "replace-selection" | "append-packet";
  revision: AgenticWorkflowRevision | null;
  editAcceptanceQueue: AgenticEditAcceptanceItem[];
  controlCenter: AgenticControlCenter;
  auditTrail: AgenticAuditTrail;
  documentEvidence: AgenticDocumentEvidence;
  lifecycleTasks: AgenticLifecycleTask[];
  reviewerAgents: AgenticReviewerAgent[];
  sectionWorkQueue: AgenticSectionWorkItem[];
  sectionDraftHistory: AgenticSectionDraftHistoryItem[];
  transformRecommendations: AgenticTransformRecommendation[];
  dataNarrativeLinks: AgenticDataNarrativeLink[];
  approvalGate: AgenticApprovalGate;
  automationQueue: AgenticAutomationTask[];
  outlineCritique: AgenticOutlineCritiqueItem[];
  preReviewRehearsal: AgenticPreReviewRehearsalItem[];
  releaseEvidenceBundle: AgenticReleaseEvidenceBundle;
  reviewChecklist: string[];
  distributionChecklist: string[];
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  blockers: string[];
}

export interface AgenticReleaseEvidenceBundle {
  id: string;
  summary: string;
  items: AgenticReleaseEvidenceItem[];
  blockers: string[];
}

export interface AgenticReleaseEvidenceItem {
  label: string;
  owner: string;
  status: AgenticEvidenceStatus;
  detail: string;
  requiredBeforeRelease: boolean;
}

export interface AgenticOutlineCritiqueItem {
  severity: "info" | "warning" | "blocker";
  area: "coverage" | "sequence" | "duplication" | "depth" | "specificity";
  heading: string;
  detail: string;
  recommendation: string;
}

export interface AgenticOutlineVariant {
  id: string;
  label: string;
  strategy: "executive-first" | "problem-solution" | "evidence-led" | "risk-first" | "technical-deep" | "publishing-narrative";
  summary: string;
  outline: string;
  bestFor: string[];
  tradeoffs: string[];
  risks: string[];
}

export type AgenticPreReviewRehearsalKind = "question" | "objection" | "redline" | "missing-evidence";

export interface AgenticPreReviewRehearsalItem {
  id: string;
  kind: AgenticPreReviewRehearsalKind;
  reviewer: AgenticReviewerAgentId;
  prompt: string;
  whyItMatters: string;
  suggestedResponse: string;
  relatedSectionId?: string;
  releaseBlocker: boolean;
}

export interface AgenticDistributionTargetPlan {
  target: ExportTarget;
  label: string;
  purpose: string;
  preflightChecks: string[];
  handoffSteps: string[];
  evidenceRequired: string[];
}

export type AgenticTransformRecommendationKind =
  | "calc"
  | "chart"
  | "table"
  | "diagram"
  | "timeline"
  | "roadmap"
  | "schema"
  | "equation"
  | "publishing";

export interface AgenticTransformRecommendation {
  id: string;
  kind: AgenticTransformRecommendationKind;
  label: string;
  purpose: string;
  insertionTarget: string;
  sectionId?: string;
  templateId?: string;
  sourceSignal: string;
  narrativeReviewTrigger: string;
  evidenceRequired: string[];
  riskLevel: "low" | "medium" | "high";
  suggestedMarkdown: string;
  owner: string;
}

export interface AgenticDataNarrativeLink {
  id: string;
  sourceKind: AgenticTransformRecommendationKind | AgenticDocumentClaim["kind"] | "source-pack" | "metadata";
  sourceLabel: string;
  affectedSection: string;
  sectionId?: string;
  changeSignal: string;
  narrativeRisk: string;
  reviewAction: string;
  evidenceRequired: string[];
  status: "watch" | "needs-review" | "blocked";
  owner: string;
}

export type AgenticAutomationTaskKind =
  | "evidence-scan"
  | "outline-critique"
  | "transform-validation"
  | "export-preflight"
  | "accessibility-check"
  | "readiness-refresh";

export interface AgenticAutomationTask {
  id: string;
  kind: AgenticAutomationTaskKind;
  label: string;
  owner: string;
  status: AgenticControlStatus;
  safeToAutoRun: boolean;
  trigger: string;
  action: AgenticWorkflowAction;
  evidence: string[];
  nextStep: string;
  manualOnlyReason?: string;
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

export interface AgenticAuditTrail {
  runId: string;
  generatedAt: string;
  plannerVersion: string;
  instructionFingerprint: string;
  contextFingerprint: string;
  sourceFingerprint: string;
  outputFingerprint: string;
  applicationMode: AgenticWorkflowRun["applicationMode"];
  rollbackPlan: string[];
  reviewEvents: string[];
}

export type AgenticReviewerAgentId = "editor" | "evidence" | "risk" | "citation" | "governance" | "export";
export type AgenticReviewerAgentStatus = "ready" | "needs-review" | "blocked";

export interface AgenticReviewerAgent {
  id: AgenticReviewerAgentId;
  label: string;
  mandate: string;
  status: AgenticReviewerAgentStatus;
  findings: string[];
  requiredActions: string[];
}

export interface AgenticSectionWorkItem {
  id: string;
  order: number;
  heading: string;
  level: number;
  lane: AgenticWorkflowLane;
  draftingDepth: DocsLiveDraftDepth;
  contract: AgenticSectionContract;
  draftingInstruction: string;
  completionCriteria: string[];
  reviewerAgentIds: AgenticReviewerAgentId[];
}

export type AgenticSectionDraftAcceptanceStatus = "drafted" | "needs-review" | "accepted";

export interface AgenticSectionDraftHistoryItem {
  id: string;
  sectionId: string;
  sectionHeading: string;
  generatedAt: string;
  versionLabel: string;
  promptSummary: string;
  rationale: string;
  reviewerNotes: string[];
  sectionFingerprint: string;
  sourceFingerprint: string;
  restorePointMarkdown: string;
  acceptanceStatus: AgenticSectionDraftAcceptanceStatus;
}

export interface AgenticSectionContract {
  purpose: string;
  targetReader: string;
  desiredDecision: string;
  evidenceExpectations: string[];
  owner: string;
  riskLevel: "low" | "medium" | "high";
  doneCriteria: string[];
}

const agentPlannerVersion = "agentic-workflow-v3-control-audit";

export const agenticWorkflowPlaybooks: AgenticWorkflowPlaybook[] = [
  {
    id: "board-memo-to-approval",
    label: "Board Memo To Approval",
    summary: "Create a decision-ready board memo, identify missing evidence, review risks, and prepare controlled PDF plus Google Docs handoff.",
    instruction:
      "Create a board memo for senior decision makers, compose it section by section, check evidence gaps, review risks and approvals, humanize the tone, then prepare PDF and Google Docs distribution.",
    bestFor: ["Board papers", "executive decisions", "investment approvals"],
    expectedOutputs: ["Board memo draft", "risk and evidence review", "PDF handoff", "Google Docs collaboration package"],
  },
  {
    id: "proposal-to-client-package",
    label: "Client Proposal Package",
    summary: "Turn rough notes into a client-facing proposal with placeholders, proof points, review checks, and DOCX/PDF export readiness.",
    instruction:
      "Create a client proposal from the current notes, structure the offer, clarify audience value, add placeholders for client, owner, evidence, pricing, and deadline, review claims and tone, then prepare DOCX and PDF distribution.",
    bestFor: ["Consulting proposals", "sales documents", "statements of work"],
    expectedOutputs: ["Proposal draft", "placeholder checklist", "claim review", "DOCX/PDF package"],
  },
  {
    id: "sop-from-outline",
    label: "SOP From Outline",
    summary: "Use an outline or rough process notes to build an operating procedure with responsibilities, checks, exceptions, and review gates.",
    instruction:
      "Use the current outline to create a standard operating procedure, flesh out each section systematically, add responsibilities, inputs, outputs, exceptions, controls, and review gates, then prepare HTML and Google Docs distribution.",
    bestFor: ["Operating procedures", "training documents", "policy rollouts"],
    expectedOutputs: ["Procedure draft", "section work queue", "control checklist", "HTML/Google Docs handoff"],
  },
  {
    id: "technical-paper-with-latex",
    label: "Technical Paper With LaTeX",
    summary: "Build a technical or research document with citation discipline, equations, evidence review, and LaTeX export checks.",
    instruction:
      "Create a technical paper from the current outline or notes, compose each section, check citations, equations, tables, references, assumptions, and evidence, then prepare LaTeX, PDF, and Google Docs distribution.",
    bestFor: ["Research notes", "technical architecture", "academic drafts"],
    expectedOutputs: ["Technical draft", "citation review", "LaTeX export checklist", "PDF/Google Docs package"],
  },
  {
    id: "publish-to-blog-and-substack",
    label: "Publish To Blog And Substack",
    summary: "Transform a draft into web and newsletter copy with editorial cleanup, metadata, links, excerpts, and publishing evidence.",
    instruction:
      "Revise the current document for web readers, humanize the voice, tighten headings, verify claims and links, create an excerpt, tags, subject line, preview text, and call to action, then prepare blog, Substack, and HTML distribution.",
    bestFor: ["Thought leadership", "newsletters", "public announcements"],
    expectedOutputs: ["Web-ready revision", "publishing metadata", "link/citation review", "blog/Substack/HTML packages"],
  },
  {
    id: "executive-revision-pass",
    label: "Executive Revision Pass",
    summary: "Rewrite selected text or a whole document for executives, preserving facts while tightening decisions, risks, and next actions.",
    instruction:
      "Revise the selected text or current document for an executive audience, make it concise and decision-oriented, preserve verified facts, surface risks and assumptions, add reviewer handoff notes, and prepare export readiness for PDF and DOCX.",
    bestFor: ["CFO review", "CEO updates", "leadership briefings"],
    expectedOutputs: ["Selection-aware revision", "risk review", "humanization pass", "PDF/DOCX readiness"],
  },
  {
    id: "strategy-memo-from-research",
    label: "Strategy Memo From Research",
    summary: "Turn research notes into a strategy memo with options, tradeoffs, risks, decisions, and executive-ready recommendations.",
    instruction:
      "Create a strategy memo from the current research notes, organize the situation, options, tradeoffs, recommendation, risks, owner, and decision timeline, check evidence and assumptions, then prepare PDF and Google Docs distribution.",
    bestFor: ["Strategy memos", "market-entry analysis", "portfolio decisions"],
    expectedOutputs: ["Strategy memo draft", "options and tradeoffs", "evidence review", "PDF/Google Docs package"],
  },
  {
    id: "policy-to-approval",
    label: "Policy To Approval",
    summary: "Draft a policy with scope, obligations, exceptions, controls, reviewer sign-off, and approval metadata.",
    instruction:
      "Create a policy document from the current context, define scope, roles, requirements, exceptions, controls, enforcement, review cadence, and approval metadata, then prepare DOCX, PDF, and Google Docs distribution.",
    bestFor: ["Internal policies", "compliance rollouts", "governance updates"],
    expectedOutputs: ["Policy draft", "control checklist", "approval metadata tasks", "DOCX/PDF/Google Docs package"],
  },
  {
    id: "release-notes-to-publish",
    label: "Release Notes To Publish",
    summary: "Convert shipped changes into release notes with audience framing, known issues, upgrade notes, and publishing packages.",
    instruction:
      "Create release notes from the current change list, group features, fixes, known issues, upgrade notes, owners, and customer impact, humanize the voice, verify links, then prepare blog, Substack, HTML, and Google Docs distribution.",
    bestFor: ["Product releases", "customer announcements", "upgrade notes"],
    expectedOutputs: ["Release notes draft", "known-issues review", "link and claim check", "blog/Substack/HTML package"],
  },
  {
    id: "grant-application-review",
    label: "Grant Application Review",
    summary: "Build a grant application narrative with eligibility, budget, outcomes, attachments, evidence, and reviewer gates.",
    instruction:
      "Create a grant application from the current notes, structure eligibility, need, outcomes, budget, timeline, attachments, risks, reviewer questions, and evidence gaps, then prepare DOCX, PDF, and Google Docs distribution.",
    bestFor: ["Grant applications", "funding proposals", "nonprofit reports"],
    expectedOutputs: ["Grant narrative draft", "budget and attachment checklist", "evidence gaps", "DOCX/PDF/Google Docs package"],
  },
];

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

const revisionModeSignals: Array<[AgenticRevisionMode, RegExp]> = [
  ["clarity", /\b(clear|clarity|clarify|simplify|plain|make sense|readable)\b/i],
  ["brevity", /\b(shorten|concise|brief|crisp|tighten|summari[sz]e|less wordy)\b/i],
  ["tone", /\b(tone|voice|style|formal|friendly|professional|humanize|reader)\b/i],
  ["evidence", /\b(evidence|source|citation|fact.?check|verify|claim|data|reference)\b/i],
  ["legal-caution", /\b(legal|risks?|compliance|policy|obligations?|liability|approve|approval|must|shall)\b/i],
  ["executive-summary", /\b(executive|board|ceo|cfo|leadership|director|decision|recommendation)\b/i],
  ["accessibility", /\b(accessib|screen reader|plain language|non-technical|layperson|jargon)\b/i],
  ["humanization", /\b(humanize|less ai|natural|generic|robotic|bland|stale)\b/i],
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
  const contextAnswers = request.contextAnswers?.trim() || "";
  const sourcePackText = request.sourcePackText?.trim() || "";
  const sourcePack = buildAgenticSourcePack(sourcePackText);
  const memoryText = request.memoryText?.trim() || "";
  const documentMemory = buildAgenticDocumentMemory({
    memoryText,
    contextAnswers,
    documentText: request.documentText || "",
  });
  const corpus = [instruction, contextAnswers, sourcePackText, memoryText, request.documentTitle, request.documentText, request.selectedText].filter(Boolean).join("\n");
  const lanes = detectLanes(corpus);
  const primaryLane = lanes[0] || "create";
  const documentType = normalizeDocsLiveDocumentType(corpus);
  const title = inferTitle(request, documentType, primaryLane);
  const distributionTargets = detectDistributionTargets(corpus);
  const context = buildContext(request, lanes, distributionTargets);
  const placeholderText = buildPlaceholderText(corpus, title, lanes, distributionTargets);
  const contextCompleteness = scoreContextCompleteness(corpus, context, placeholderText, distributionTargets);
  const documentIntent = buildDocumentIntentSheet({
    request,
    corpus,
    title,
    documentType,
    lanes,
    distributionTargets,
    contextCompleteness,
  });
  const suggestedOutline = buildSuggestedOutline(request, primaryLane, documentType);
  const outlineVariants = buildOutlineVariants({
    suggestedOutline,
    documentType,
    distributionTargets,
    contextCompleteness,
    documentIntent,
    sourcePack,
    documentMemory,
  });
  const missingInputs = buildMissingInputs(corpus, lanes, distributionTargets);
  const revisionInstruction = buildRevisionInstruction(instruction, lanes, request.selectedText);
  const revisionModes = detectRevisionModes(corpus, lanes, documentType);
  const qualityGates = buildQualityGates(documentType, distributionTargets);
  const steps = buildPlanSteps(lanes, missingInputs, distributionTargets, Boolean(request.documentText?.trim()), Boolean(request.selectedText?.trim()));

  return {
    instruction,
    title,
    documentType,
    primaryLane,
    lanes,
    documentIntent,
    contextAnswers,
    sourcePackText,
    sourcePack,
    memoryText,
    documentMemory,
    suggestedOutline,
    outlineVariants,
    context,
    placeholderText,
    contextCompleteness,
    revisionInstruction,
    revisionModes,
    qualityGates,
    distributionTargets,
    missingInputs,
    steps,
  };
}

export function buildAgenticWorkflowRun(request: AgenticWorkflowRunRequest): AgenticWorkflowRun {
  const generatedAt = request.generatedAt || new Date().toISOString();
  const plan = buildAgenticWorkflowPlan(request);
  const hasSelection = Boolean(request.selectedText?.trim());
  const hasDocument = Boolean(request.documentText?.trim());
  const revision = plan.lanes.some((lane) => lane === "edit" || lane === "revise") ? buildRevision(request, plan) : null;
  const editAcceptanceQueue = buildEditAcceptanceQueue(request, plan, revision);
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
        generatedAt,
      })
    : null;
  const distributionTargetPlans = buildDistributionTargetPlans(plan);
  const distributionChecklist = buildDistributionChecklist(plan, distributionTargetPlans);
  const blockers = buildRunBlockers(plan, hasDocument, hasSelection);
  const applicationMode = inferApplicationMode(plan, hasDocument, hasSelection);
  const documentEvidence = analyzeAgenticDocumentEvidence(request.documentText || "", plan);
  const reviewChecklist = buildReviewChecklist(plan, revision, editAcceptanceQueue, documentEvidence);
  const outlineCritique = buildOutlineCritique(plan);
  const approvalGate = buildApprovalGate(plan, documentEvidence, request.documentText || "");
  const controlCenter = buildControlCenter({
    plan,
    blockers,
    hasDocument,
    hasSelection,
    revision,
    distributionTargetPlans,
    documentEvidence,
    outlineCritique,
    approvalGate,
  });
  const reviewerAgents = buildReviewerAgents({
    plan,
    draftMarkdown: draft?.markdown || "",
    revision,
    editAcceptanceQueue,
    controlCenter,
    distributionTargetPlans,
    blockers,
    documentEvidence,
    outlineCritique,
  });
  const sectionWorkQueue = buildSectionWorkQueue(plan, reviewerAgents);
  const sectionDraftHistory = buildSectionDraftHistory({
    plan,
    sectionWorkQueue,
    draftSections: draft?.sections || [],
    draftMarkdown: draft?.markdown || "",
    generatedAt,
  });
  const transformRecommendations = buildTransformRecommendations({
    plan,
    sectionWorkQueue,
    documentEvidence,
    distributionTargetPlans,
  });
  const dataNarrativeLinks = buildDataNarrativeLinks({
    plan,
    sectionWorkQueue,
    documentEvidence,
    transformRecommendations,
  });
  const preReviewRehearsal = buildPreReviewRehearsal({
    plan,
    reviewerAgents,
    sectionWorkQueue,
    documentEvidence,
    revision,
    distributionTargetPlans,
    blockers,
  });
  const automationQueue = buildAutomationQueue({
    plan,
    documentEvidence,
    outlineCritique,
    distributionTargetPlans,
    controlCenter,
    blockers,
    transformRecommendations,
    approvalGate,
  });
  const lifecycleTasks = buildLifecycleTasks({
    plan,
    revision,
    editAcceptanceQueue,
    reviewerAgents,
    sectionWorkQueue,
    sectionDraftHistory,
    transformRecommendations,
    dataNarrativeLinks,
    approvalGate,
    automationQueue,
    preReviewRehearsal,
    distributionTargetPlans,
    blockers,
    documentEvidence,
    outlineCritique,
  });
  const auditTrail = buildAuditTrail({
    plan,
    request,
    revision,
    editAcceptanceQueue,
    draftMarkdown: draft?.markdown || "",
    lifecycleTasks,
    reviewerAgents,
    sectionWorkQueue,
    sectionDraftHistory,
    transformRecommendations,
    dataNarrativeLinks,
    approvalGate,
    automationQueue,
    preReviewRehearsal,
    documentEvidence,
    outlineCritique,
    reviewChecklist,
    distributionChecklist,
    distributionTargetPlans,
    blockers,
    applicationMode,
    generatedAt,
  });
  const releaseEvidenceBundle = buildReleaseEvidenceBundle({
    plan,
    auditTrail,
    controlCenter,
    lifecycleTasks,
    reviewerAgents,
    sectionWorkQueue,
    sectionDraftHistory,
    transformRecommendations,
    dataNarrativeLinks,
    approvalGate,
    automationQueue,
    preReviewRehearsal,
    distributionTargetPlans,
    documentEvidence,
    blockers,
  });
  const markdown = buildRunMarkdown({
    plan,
    draftMarkdown: draft?.markdown || "",
    revision,
    editAcceptanceQueue,
    controlCenter,
    documentEvidence,
    auditTrail,
    releaseEvidenceBundle,
    lifecycleTasks,
    reviewerAgents,
    sectionWorkQueue,
    sectionDraftHistory,
    transformRecommendations,
    dataNarrativeLinks,
    approvalGate,
    automationQueue,
    outlineCritique,
    preReviewRehearsal,
    reviewChecklist,
    distributionChecklist,
    distributionTargetPlans,
    blockers,
    generatedAt,
  });

  return {
    plan,
    summary: `Agent run prepared ${plan.lanes.map(titleCase).join(", ")} for ${plan.title}.`,
    markdown,
    applicationMode,
    revision,
    editAcceptanceQueue,
    controlCenter,
    auditTrail,
    documentEvidence,
    releaseEvidenceBundle,
    lifecycleTasks,
    reviewerAgents,
    sectionWorkQueue,
    sectionDraftHistory,
    transformRecommendations,
    dataNarrativeLinks,
    approvalGate,
    automationQueue,
    outlineCritique,
    preReviewRehearsal,
    reviewChecklist,
    distributionChecklist,
    distributionTargetPlans,
    blockers,
  };
}

export function buildAgenticSectionWorkBrief(section: AgenticSectionWorkItem, reviewerAgents: AgenticReviewerAgent[]): string {
  const reviewerLabels = section.reviewerAgentIds.map((id) => reviewerAgents.find((agent) => agent.id === id)?.label || titleCase(id));
  return [
    `## ${section.heading} Work Brief`,
    "",
    "```ai-section-task",
    `id: ${section.id}`,
    `order: ${section.order}`,
    `level: ${section.level}`,
    `lane: ${section.lane}`,
    `draftingDepth: ${section.draftingDepth}`,
    `riskLevel: ${section.contract.riskLevel}`,
    `reviewers: ${section.reviewerAgentIds.join(", ")}`,
    "status: needs-draft",
    "```",
    "",
    "### Section Contract",
    "",
    `Purpose: ${section.contract.purpose}`,
    "",
    `Target reader: ${section.contract.targetReader}`,
    "",
    `Desired decision or outcome: ${section.contract.desiredDecision}`,
    "",
    `Accountable owner: ${section.contract.owner}`,
    "",
    "Evidence expectations:",
    ...section.contract.evidenceExpectations.map((item) => `- [ ] ${item}`),
    "",
    "Done criteria:",
    ...section.contract.doneCriteria.map((item) => `- [ ] ${item}`),
    "",
    "### Drafting Instruction",
    "",
    section.draftingInstruction,
    "",
    "### Completion Criteria",
    "",
    ...section.completionCriteria.map((item) => `- [ ] ${item}`),
    "",
    "### Assigned Reviewers",
    "",
    ...reviewerLabels.map((label) => `- [ ] ${label}`),
    "",
  ].join("\n");
}

export function buildAgenticLifecycleTaskBrief(task: AgenticLifecycleTask): string {
  const metadata = [
    `id: ${task.id}`,
    `lane: ${task.lane}`,
    `owner: ${task.owner}`,
    `status: ${task.status}`,
    `action: ${task.action}`,
    task.sectionId ? `sectionId: ${task.sectionId}` : "",
    task.target ? `target: ${task.target}` : "",
  ].filter(Boolean);
  return [
    `## ${task.title} Task Brief`,
    "",
    "```ai-lifecycle-task",
    ...metadata,
    "```",
    "",
    "### Next Step",
    "",
    task.nextStep,
    "",
    "### Evidence Checklist",
    "",
    ...task.evidence.map((item) => `- [ ] ${item}`),
    "",
    "### Handoff Notes",
    "",
    "- [ ] Record who completed the task and when.",
    "- [ ] Keep unresolved assumptions visible in review notes.",
    "- [ ] Preserve AI provenance and run identifiers until human approval is complete.",
    "",
  ].join("\n");
}

export function buildAgenticTransformRecommendationMarkdown(item: AgenticTransformRecommendation): string {
  const metadata = [
    `id: ${item.id}`,
    `kind: ${item.kind}`,
    `owner: ${item.owner}`,
    `riskLevel: ${item.riskLevel}`,
    item.templateId ? `templateId: ${item.templateId}` : "",
    item.sectionId ? `sectionId: ${item.sectionId}` : "",
    `insertionTarget: ${sanitizeMarkerValue(item.insertionTarget)}`,
    "status: needs-review",
  ].filter(Boolean);
  return [
    `## ${item.label}`,
    "",
    "<!-- ai-assisted: needs-review | source: NEditor Agent Workspace | workflow: agent-selected-transform -->",
    "",
    "```ai-transform-recommendation",
    ...metadata,
    "```",
    "",
    item.purpose,
    "",
    `Source signal: ${item.sourceSignal}`,
    "",
    `Narrative review trigger: ${item.narrativeReviewTrigger}`,
    "",
    "Evidence required:",
    ...item.evidenceRequired.map((evidence) => `- [ ] ${evidence}`),
    "",
    item.suggestedMarkdown.trim(),
    "",
  ].join("\n");
}

export function buildAgenticDataNarrativeAuditMarkdown(items: AgenticDataNarrativeLink[]): string {
  return [
    "## Data-to-Narrative Review Queue",
    "",
    "```ai-data-narrative-bridge",
    `links: ${items.length}`,
    "status: needs-review",
    "```",
    "",
    ...(items.length
      ? [
          "| Source | Affected section | Status | Review action |",
          "| --- | --- | --- | --- |",
          ...items.map(
            (item) =>
              `| ${escapeTableCell(`${item.sourceKind}: ${item.sourceLabel}`)} | ${escapeTableCell(item.affectedSection)} | ${item.status} | ${escapeTableCell(item.reviewAction)} |`,
          ),
          "",
          ...items.flatMap((item) => [
            `### ${item.sourceLabel}`,
            "",
            `Change signal: ${item.changeSignal}`,
            "",
            `Narrative risk: ${item.narrativeRisk}`,
            "",
            `Owner: ${item.owner}`,
            "",
            "Evidence required:",
            ...item.evidenceRequired.map((evidence) => `- [ ] ${evidence}`),
            "",
          ]),
        ]
      : ["No data-to-narrative links were prepared."]),
  ].join("\n");
}

export function buildAgenticApprovalGateMarkdown(gate: AgenticApprovalGate): string {
  return [
    "## Approval Metadata Gate",
    "",
    "```approval-gate",
    `status: ${gate.status}`,
    `requiredBeforeDistribution: ${gate.requiredBeforeDistribution ? "true" : "false"}`,
    `blockers: ${gate.blockers.length}`,
    "```",
    "",
    gate.summary,
    "",
    "| Field | Value | Status | Guidance |",
    "| --- | --- | --- | --- |",
    ...gate.fields.map(
      (field) =>
        `| ${escapeTableCell(field.label)} | ${escapeTableCell(field.value || "missing")} | ${field.status} | ${escapeTableCell(field.guidance)} |`,
    ),
    "",
    "### Blockers",
    "",
    ...(gate.blockers.length ? gate.blockers.map((blocker) => `- [ ] ${blocker}`) : ["No approval metadata blockers are active."]),
    "",
    "### Metadata Scaffold",
    "",
    fencedBlock("yaml", gate.metadataScaffold),
    "",
  ].join("\n");
}

export function buildAgenticReleaseEvidenceAuditPackage(run: AgenticWorkflowRun): string {
  return [
    "## NEditor Release Evidence Audit Package",
    "",
    `Run ID: ${run.auditTrail.runId}`,
    "",
    `Document: ${run.plan.title}`,
    "",
    `Generated: ${run.auditTrail.generatedAt}`,
    "",
    `Application mode: ${run.applicationMode}`,
    "",
    ...releaseEvidenceBundleMarkdown(run.releaseEvidenceBundle),
    ...auditTrailMarkdown(run.auditTrail),
    ...controlCenterMarkdown(run.controlCenter),
    ...reviewerAgentsMarkdown(run.reviewerAgents),
    ...sectionDraftHistoryMarkdown(run.sectionDraftHistory),
    ...transformRecommendationsMarkdown(run.transformRecommendations),
    ...dataNarrativeLinksMarkdown(run.dataNarrativeLinks),
    ...buildAgenticApprovalGateMarkdown(run.approvalGate).split("\n"),
    ...automationQueueMarkdown(run.automationQueue),
    ...lifecycleTasksMarkdown(run.lifecycleTasks),
    ...(run.distributionTargetPlans.length ? distributionTargetRunbookMarkdown(run.distributionTargetPlans) : ["## Distribution Runbooks", "", "No distribution target runbooks were staged for this run.", ""]),
  ].join("\n");
}

export function buildAgenticSourcePack(sourcePackText: string): AgenticSourcePack {
  const items = parseAgenticSourcePackItems(sourcePackText).slice(0, 80);
  const byKind = (kind: AgenticSourcePackItemKind) => items.filter((item) => item.kind === kind);
  return {
    items,
    urls: byKind("url"),
    files: byKind("file"),
    references: byKind("reference"),
    reviewerComments: byKind("reviewer-comment"),
    claims: byKind("claim"),
    notes: byKind("note"),
    markdown: formatAgenticSourcePack(items),
  };
}

export function buildAgenticDocumentMemory(input: {
  memoryText?: string;
  contextAnswers?: string;
  documentText?: string;
}): AgenticDocumentMemory {
  const entries = [
    ...parseAgenticMemoryText(input.memoryText || "", "user-memory"),
    ...deriveAgenticMemoryEntries(input.contextAnswers || "", "context"),
    ...deriveAgenticMemoryEntries(input.documentText || "", "current-document"),
  ].slice(0, 60);
  const deduped: AgenticDocumentMemoryEntry[] = [];
  const seen = new Set<string>();
  for (const entry of entries) {
    const key = `${entry.kind}:${entry.label.toLowerCase()}:${entry.detail.toLowerCase()}`;
    if (seen.has(key)) continue;
    seen.add(key);
    deduped.push(entry);
    if (deduped.length >= 40) break;
  }
  const byKind = (kind: AgenticDocumentMemoryKind) => deduped.filter((item) => item.kind === kind);
  const summaryParts = [
    byKind("terminology").length ? `${byKind("terminology").length} terminology` : "",
    byKind("style").length ? `${byKind("style").length} style` : "",
    byKind("accepted-decision").length ? `${byKind("accepted-decision").length} accepted decisions` : "",
    byKind("rejected-direction").length ? `${byKind("rejected-direction").length} rejected directions` : "",
    byKind("review-preference").length ? `${byKind("review-preference").length} review preferences` : "",
    byKind("distribution-preference").length ? `${byKind("distribution-preference").length} distribution preferences` : "",
  ].filter(Boolean);
  return {
    entries: deduped,
    terminology: byKind("terminology"),
    style: byKind("style"),
    acceptedDecisions: byKind("accepted-decision"),
    rejectedDirections: byKind("rejected-direction"),
    reviewPreferences: byKind("review-preference"),
    distributionPreferences: byKind("distribution-preference"),
    markdown: formatAgenticDocumentMemory(deduped),
    summary: summaryParts.length ? `Document memory carries ${summaryParts.join(", ")}.` : "No reusable document memory has been captured yet.",
  };
}

export function serializeAgenticSourcePackItem(kind: AgenticSourcePackItemKind, label: string, detail: string) {
  const cleanKind = kind || "note";
  const cleanLabel = label.trim() || titleCase(cleanKind);
  const cleanDetail = detail.trim();
  return `[${cleanKind}] ${cleanLabel}${cleanDetail ? `: ${cleanDetail}` : ""}`;
}

function parseAgenticSourcePackItems(sourcePackText: string): AgenticSourcePackItem[] {
  const items: AgenticSourcePackItem[] = [];
  const seen = new Set<string>();
  for (const rawLine of sourcePackText.split(/\r?\n/)) {
    const line = rawLine.trim().replace(/^[-*]\s+/, "");
    if (!line) continue;
    const item = parseAgenticSourcePackLine(line);
    const key = `${item.kind}:${item.label.toLowerCase()}:${item.detail.toLowerCase()}`;
    if (seen.has(key)) continue;
    seen.add(key);
    items.push(item);
  }
  return items;
}

function parseAgenticSourcePackLine(line: string): AgenticSourcePackItem {
  const tagged = line.match(/^\[(note|url|file|reference|reviewer-comment|claim)\]\s*([^:]+)?(?::\s*([\s\S]+))?$/i);
  if (tagged) {
    const kind = tagged[1].toLowerCase() as AgenticSourcePackItemKind;
    const label = (tagged[2] || titleCase(kind)).trim();
    const detail = (tagged[3] || "").trim();
    return sourcePackItem(kind, label, detail || label);
  }
  const url = line.match(/\bhttps?:\/\/\S+/i)?.[0];
  if (url) return sourcePackItem("url", url.replace(/[),.;]+$/, ""), line);
  if (/\b(?:file|path|attachment)\s*[:=]/i.test(line) || /(?:^|\/)[\w .-]+\.(?:md|pdf|docx|xlsx|csv|txt|png|jpe?g)$/i.test(line)) {
    return sourcePackItem("file", line.replace(/\b(?:file|path|attachment)\s*[:=]\s*/i, "").slice(0, 120), line);
  }
  if (/\b(?:reviewer|comment|feedback|note from)\b/i.test(line)) return sourcePackItem("reviewer-comment", line.slice(0, 120), line);
  if (/\b(?:source|reference|citation|doi|isbn|author|paper|report)\b/i.test(line)) return sourcePackItem("reference", line.slice(0, 120), line);
  if (classifyClaimSignal(line)) return sourcePackItem("claim", line.slice(0, 120), line);
  return sourcePackItem("note", line.slice(0, 120), line);
}

function sourcePackItem(kind: AgenticSourcePackItemKind, label: string, detail: string): AgenticSourcePackItem {
  const safeLabel = label.trim() || titleCase(kind);
  const safeDetail = detail.trim() || safeLabel;
  return {
    id: `source-${kind}-${stableFingerprint(`${kind}\n${safeLabel}\n${safeDetail}`).slice(0, 12)}`,
    kind,
    label: safeLabel.slice(0, 180),
    detail: safeDetail.slice(0, 1_200),
  };
}

function parseAgenticMemoryText(text: string, source: AgenticDocumentMemoryEntry["source"]) {
  const entries: AgenticDocumentMemoryEntry[] = [];
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim().replace(/^[-*]\s+/, "");
    if (!line) continue;
    const tagged = line.match(/^\[(terminology|term|style|accepted-decision|accepted|decision|rejected-direction|rejected|avoid|review-preference|review|distribution-preference|distribution)\]\s*([^:]+)?(?::\s*([\s\S]+))?$/i);
    if (tagged) {
      const kind = normalizeMemoryKind(tagged[1]);
      const label = (tagged[2] || titleCase(kind)).trim();
      const detail = (tagged[3] || label).trim();
      entries.push(memoryEntry(kind, label, detail, source));
      continue;
    }
    const derived = memoryEntryFromSignal(line, source);
    if (derived) entries.push(derived);
    if (entries.length >= 60) break;
  }
  return entries;
}

function deriveAgenticMemoryEntries(text: string, source: AgenticDocumentMemoryEntry["source"]) {
  return parseAgenticMemoryText(
    text
      .split(/\r?\n/)
      .filter((line) => /\b(prefer|avoid|do not|don't|use term|call it|accepted|approved|rejected|reviewer prefers|publish|distribution)\b/i.test(line))
      .join("\n"),
    source,
  );
}

function memoryEntryFromSignal(line: string, source: AgenticDocumentMemoryEntry["source"]): AgenticDocumentMemoryEntry | null {
  if (/\b(?:use term|call it|terminology|glossary)\b/i.test(line)) return memoryEntry("terminology", line.slice(0, 90), line, source);
  if (/\b(?:tone|style|voice|prefer|plain language|executive|technical)\b/i.test(line)) return memoryEntry("style", line.slice(0, 90), line, source);
  if (/\b(?:accepted|approved|decision|decided|chosen)\b/i.test(line)) return memoryEntry("accepted-decision", line.slice(0, 90), line, source);
  if (/\b(?:rejected|avoid|do not|don't|never use)\b/i.test(line)) return memoryEntry("rejected-direction", line.slice(0, 90), line, source);
  if (/\b(?:reviewer prefers|review preference|approver wants|legal wants|cfo wants)\b/i.test(line)) return memoryEntry("review-preference", line.slice(0, 90), line, source);
  if (/\b(?:distribution|publish|export|substack|blog|google docs|pdf|docx)\b/i.test(line)) {
    return memoryEntry("distribution-preference", line.slice(0, 90), line, source);
  }
  return null;
}

function normalizeMemoryKind(value: string): AgenticDocumentMemoryKind {
  const normalized = value.toLowerCase();
  if (normalized === "term") return "terminology";
  if (normalized === "accepted" || normalized === "decision") return "accepted-decision";
  if (normalized === "rejected" || normalized === "avoid") return "rejected-direction";
  if (normalized === "review") return "review-preference";
  if (normalized === "distribution") return "distribution-preference";
  return normalized as AgenticDocumentMemoryKind;
}

function memoryEntry(
  kind: AgenticDocumentMemoryKind,
  label: string,
  detail: string,
  source: AgenticDocumentMemoryEntry["source"],
): AgenticDocumentMemoryEntry {
  const safeLabel = label.trim() || titleCase(kind);
  const safeDetail = detail.trim() || safeLabel;
  return {
    id: `memory-${kind}-${stableFingerprint(`${kind}\n${safeLabel}\n${safeDetail}\n${source}`).slice(0, 12)}`,
    kind,
    label: safeLabel.slice(0, 160),
    detail: safeDetail.slice(0, 1_200),
    source,
  };
}

function formatAgenticDocumentMemory(entries: AgenticDocumentMemoryEntry[]) {
  if (!entries.length) return "";
  return entries.map((entry) => `- [${entry.kind}] ${entry.label}: ${entry.detail} (${entry.source})`).join("\n");
}

function formatAgenticSourcePack(items: AgenticSourcePackItem[]) {
  if (!items.length) return "";
  return items.map((item) => `- [${item.kind}] ${item.label}: ${item.detail}`).join("\n");
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
  const revisionPasses = buildRevisionPasses(plan.revisionModes);
  const proposedText = reviseText(originalText, plan.revisionInstruction, plan.placeholderText, revisionPasses);
  const meaningDriftFindings = findMeaningDrift(originalText, proposedText);
  const changeSummary = [
    "Preserved the user's intent while making the requested revision explicit.",
    revisionPasses.length ? `Applied revision passes: ${revisionPasses.map((pass) => pass.label).join(", ")}.` : "",
    "Added AI-assisted review metadata so the change remains governable before export.",
    meaningDriftFindings.length
      ? `Meaning-drift scan found ${meaningDriftFindings.length} number, date, commitment, or caveat item(s) that need reviewer confirmation.`
      : "Meaning-drift scan did not find changed numbers, dates, commitments, or caveats.",
    plan.lanes.includes("review") ? "Prepared QA prompts for evidence, tone, and reviewer sign-off." : "",
  ].filter(Boolean);

  return {
    originalText,
    proposedText,
    changeSummary,
    revisionPasses,
    meaningDriftFindings,
  };
}

function buildEditAcceptanceQueue(
  request: AgenticWorkflowRunRequest,
  plan: AgenticWorkflowPlan,
  revision: AgenticWorkflowRevision | null,
): AgenticEditAcceptanceItem[] {
  if (!revision) return [];
  if (request.selectedText?.trim()) {
    return [
      editAcceptanceItem({
        scope: "selection",
        heading: "Selected text revision",
        originalText: revision.originalText,
        proposedText: revision.proposedText,
        changeSummary: revision.changeSummary,
        driftFindings: revision.meaningDriftFindings,
      }),
    ];
  }

  const sections = markdownRevisionSections(revision.originalText);
  if (sections.length <= 1) {
    return [
      editAcceptanceItem({
        scope: "document",
        heading: plan.title,
        originalText: revision.originalText,
        proposedText: revision.proposedText,
        changeSummary: revision.changeSummary,
        driftFindings: revision.meaningDriftFindings,
      }),
    ];
  }

  return sections.slice(0, 12).map((section) => {
    const proposedText = reviseText(section.markdown, plan.revisionInstruction, plan.placeholderText, revision.revisionPasses);
    return editAcceptanceItem({
      scope: "section",
      heading: section.heading,
      originalText: section.markdown,
      proposedText,
      changeSummary: [
        `Prepared a scoped revision for "${section.heading}".`,
        `Applied revision passes: ${revision.revisionPasses.map((pass) => pass.label).join(", ") || "none"}.`,
      ],
      driftFindings: findMeaningDrift(section.markdown, proposedText),
    });
  });
}

function editAcceptanceItem(input: {
  scope: AgenticEditAcceptanceItem["scope"];
  heading: string;
  originalText: string;
  proposedText: string;
  changeSummary: string[];
  driftFindings: AgenticMeaningDriftFinding[];
}): AgenticEditAcceptanceItem {
  const riskNotes = input.driftFindings.length
    ? input.driftFindings.map((finding) => `${titleCase(finding.kind)} ${finding.severity}: ${finding.detail}`)
    : ["No changed or missing numbers, dates, commitments, or caveats were detected for this item."];
  return {
    id: `accept-${input.scope}-${stableFingerprint([input.heading, input.originalText, input.proposedText].join("\n")).slice(0, 12)}`,
    scope: input.scope,
    heading: input.heading,
    originalText: input.originalText,
    proposedText: input.proposedText,
    changeSummary: input.changeSummary,
    riskNotes,
    recommendation: input.driftFindings.some((finding) => finding.severity === "blocker")
      ? "Resolve risk notes before accepting this edit."
      : "Accept, reject, or request another revision before applying this edit.",
  };
}

function markdownRevisionSections(markdown: string) {
  const lines = markdown.split(/\r?\n/);
  const sections: Array<{ heading: string; markdown: string }> = [];
  const starts: Array<{ index: number; level: number; heading: string }> = [];
  for (let index = 0; index < lines.length; index += 1) {
    const match = lines[index].match(/^(#{1,4})\s+(.+?)\s*$/);
    if (!match) continue;
    starts.push({ index, level: match[1].length, heading: match[2].replace(/\s+#+\s*$/, "").trim() });
  }
  for (let index = 0; index < starts.length; index += 1) {
    const start = starts[index];
    let end = lines.length;
    for (let scan = index + 1; scan < starts.length; scan += 1) {
      if (starts[scan].level <= start.level) {
        end = starts[scan].index;
        break;
      }
    }
    const sectionMarkdown = lines.slice(start.index, end).join("\n").trim();
    if (sectionMarkdown) sections.push({ heading: start.heading, markdown: sectionMarkdown });
  }
  return sections;
}

function detectRevisionModes(corpus: string, lanes: AgenticWorkflowLane[], documentType: DocsLiveDocumentType): AgenticRevisionMode[] {
  if (!lanes.some((lane) => lane === "edit" || lane === "revise")) return [];
  const detected = revisionModeSignals.flatMap(([mode, signal]) => (signal.test(corpus) ? [mode] : []));
  if (lanes.includes("review") && !detected.includes("evidence")) detected.push("evidence");
  if (documentType === "board-memo" && !detected.includes("executive-summary")) detected.push("executive-summary");
  if (!detected.includes("clarity")) detected.unshift("clarity");
  return Array.from(new Set(detected)).slice(0, 8);
}

function buildRevisionPasses(modes: AgenticRevisionMode[]): AgenticRevisionPass[] {
  return modes.map((mode) => revisionPassProfile(mode));
}

function revisionPassProfile(mode: AgenticRevisionMode): AgenticRevisionPass {
  const profiles: Record<AgenticRevisionMode, AgenticRevisionPass> = {
    clarity: {
      mode,
      label: "Clarity",
      rationale: "Make the reader purpose, decision, and sentence flow easier to understand.",
      checklist: ["Remove ambiguous phrasing.", "Make the reader outcome explicit.", "Keep terminology consistent."],
    },
    brevity: {
      mode,
      label: "Brevity",
      rationale: "Compress the text without silently dropping facts, caveats, owners, or obligations.",
      checklist: ["Remove filler and repeated setup.", "Preserve material numbers, dates, owners, and commitments.", "Flag any compression risk."],
    },
    tone: {
      mode,
      label: "Tone",
      rationale: "Match voice, formality, and level of detail to the named audience.",
      checklist: ["Adapt sentence style to the audience.", "Avoid overstatement.", "Keep the document voice consistent."],
    },
    evidence: {
      mode,
      label: "Evidence",
      rationale: "Keep factual claims source-aware and leave unresolved claims visible.",
      checklist: ["Preserve source-sensitive claims.", "Add or retain citation TODOs for unsupported facts.", "Avoid inventing evidence."],
    },
    "legal-caution": {
      mode,
      label: "Legal Caution",
      rationale: "Surface obligations, approvals, risks, and policy-sensitive wording before acceptance.",
      checklist: ["Preserve commitments and caveats.", "Flag approval or compliance language.", "Avoid creating new obligations."],
    },
    "executive-summary": {
      mode,
      label: "Executive Summary",
      rationale: "Lead with the decision, financial/risk implication, and next action.",
      checklist: ["Put the recommendation early.", "Name the risk or tradeoff.", "State the next action or owner."],
    },
    accessibility: {
      mode,
      label: "Accessibility",
      rationale: "Make the revision usable for non-technical readers and assistive workflows.",
      checklist: ["Prefer plain language.", "Avoid unexplained jargon.", "Keep structure scannable."],
    },
    humanization: {
      mode,
      label: "Humanization",
      rationale: "Remove generic AI phrasing and make the draft sound like accountable human prose.",
      checklist: ["Remove stale AI phrases.", "Use concrete nouns and verbs.", "Keep confidence proportional to evidence."],
    },
  };
  return profiles[mode];
}

function findMeaningDrift(originalText: string, proposedText: string): AgenticMeaningDriftFinding[] {
  const proposedBody = stripAiReviewMetadata(proposedText);
  return [
    ...missingMeaningTokens("number", originalText, proposedBody, /\b\d+(?:[.,]\d+)*(?:\s?%|\s?[A-Za-z]{1,4})?\b/g),
    ...missingMeaningTokens(
      "date",
      originalText,
      proposedBody,
      /\b(?:20\d{2}|19\d{2}|Q[1-4]\s+20\d{2}|(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Sept|Oct|Nov|Dec)[a-z]*\.?\s+\d{1,2}(?:,\s*\d{4})?|\d{4}-\d{2}-\d{2})\b/gi,
    ),
    ...missingMeaningSentences(
      "commitment",
      originalText,
      proposedBody,
      /\b(?:must|shall|required|commits?|guarantees?|will|deadline|approve|approval|due)\b/i,
    ),
    ...missingMeaningSentences("caveat", originalText, proposedBody, /\b(?:unless|except|subject to|may|might|could|risk|assumption|caveat)\b/i),
  ].slice(0, 12);
}

function missingMeaningTokens(
  kind: AgenticMeaningDriftFinding["kind"],
  originalText: string,
  proposedText: string,
  pattern: RegExp,
): AgenticMeaningDriftFinding[] {
  const originalTokens = uniqueMatches(originalText, pattern);
  const proposedTokens = new Set(uniqueMatches(proposedText, pattern).map((item) => item.toLowerCase()));
  return originalTokens
    .filter((token) => !proposedTokens.has(token.toLowerCase()))
    .map((token) => ({
      kind,
      severity: "blocker" as const,
      original: token,
      proposed: "Missing from proposed revision",
      detail: `The original ${kind} "${token}" is not present in the proposed revision.`,
      recommendation: `Confirm whether "${token}" should be preserved, intentionally removed, or replaced with sourced reviewer approval.`,
    }));
}

function missingMeaningSentences(
  kind: "commitment" | "caveat",
  originalText: string,
  proposedText: string,
  signal: RegExp,
): AgenticMeaningDriftFinding[] {
  const originalSentences = splitSentences(originalText).filter((sentence) => signal.test(sentence));
  const proposedLower = proposedText.toLowerCase();
  return originalSentences
    .filter((sentence) => !proposedLower.includes(sentence.toLowerCase()))
    .map((sentence) => ({
      kind,
      severity: kind === "commitment" ? ("blocker" as const) : ("warning" as const),
      original: sentence,
      proposed: "Missing or materially rewritten in proposed revision",
      detail: `The original ${kind} statement may have been removed or compressed.`,
      recommendation: `Review this ${kind} statement before accepting the revision: ${sentence}`,
    }));
}

function splitSentences(text: string) {
  return text
    .split(/(?<=[.!?])\s+/)
    .map((sentence) => sentence.trim())
    .filter(Boolean);
}

function uniqueMatches(text: string, pattern: RegExp) {
  return Array.from(new Set([...text.matchAll(pattern)].map((match) => match[0].replace(/\s+/g, " ").trim()).filter(Boolean)));
}

function stripAiReviewMetadata(text: string) {
  return text.replace(/<!--\s*ai-assisted:[\s\S]*?-->/g, "").trim();
}

function reviseText(text: string, instruction: string, placeholders: string, revisionPasses: AgenticRevisionPass[]) {
  const cleaned = humanizeText(text || "Draft the requested change here with verified facts and named owners.");
  const lowerInstruction = instruction.toLowerCase();
  const modes = new Set(revisionPasses.map((pass) => pass.mode));
  let proposed = cleaned;
  if (modes.has("clarity")) {
    proposed = clarifyText(proposed);
  }
  if (modes.has("brevity") || /\b(shorten|concise|brief|crisp|executive)\b/.test(lowerInstruction)) {
    proposed = shortenText(proposed);
  }
  if (/\b(expand|detail|flesh out|elaborate)\b/.test(lowerInstruction)) {
    proposed = expandText(proposed, placeholders);
  }
  if (modes.has("executive-summary") && /\b(cfo|finance|financial|budget|investment|roi)\b/.test(lowerInstruction)) {
    proposed = addFinanceFrame(proposed);
  } else if (modes.has("executive-summary") || /\b(board|executive|ceo|leadership)\b/.test(lowerInstruction)) {
    proposed = addExecutiveFrame(proposed);
  }
  if (modes.has("accessibility")) {
    proposed = accessibilityText(proposed);
  }
  if (modes.has("humanization") || /\b(humanize|natural|less ai|plain language|non-technical)\b/.test(lowerInstruction)) {
    proposed = humanizeText(proposed);
  }
  if (modes.has("legal-caution") || modes.has("evidence")) {
    proposed = addRevisionReviewNote(proposed, modes);
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

function clarifyText(text: string) {
  return text
    .replace(/\bcan be\b/gi, "is")
    .replace(/\bthis section\b/gi, "this section")
    .replace(/\bopportunities\b/gi, "options")
    .replace(/[ \t]{2,}/g, " ")
    .trim();
}

function accessibilityText(text: string) {
  return text
    .replace(/\bARR\b/g, "ARR")
    .replace(/\bROI\b/g, "ROI")
    .replace(/\butilisation\b/gi, "use")
    .replace(/[ \t]{2,}/g, " ")
    .trim();
}

function addRevisionReviewNote(text: string, modes: Set<AgenticRevisionMode>) {
  const notes = [
    modes.has("evidence") ? "Evidence pass: verify every factual claim or keep a citation TODO before acceptance." : "",
    modes.has("legal-caution") ? "Legal caution pass: confirm obligations, approvals, risks, and caveats with the responsible reviewer." : "",
  ].filter(Boolean);
  return notes.length ? `${text}\n\n_Review note: ${notes.join(" ")}_` : text;
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
  const sourcePack = buildAgenticSourcePack(request.sourcePackText || "");
  const documentMemory = buildAgenticDocumentMemory({
    memoryText: request.memoryText || "",
    contextAnswers: request.contextAnswers || "",
    documentText: request.documentText || "",
  });
  return [
    `User intent: ${request.instruction.trim() || "Create and improve the current document."}`,
    request.contextAnswers?.trim() ? `Agent context answers: ${request.contextAnswers.trim().slice(0, 1600)}` : "",
    documentMemory.entries.length ? `Document memory: ${documentMemory.summary}` : "",
    documentMemory.markdown ? `Reusable document memory:\n${documentMemory.markdown.slice(0, 2200)}` : "",
    sourcePack.items.length
      ? `User source pack: ${sourcePack.items.length} item(s), including ${sourcePack.claims.length} claim(s), ${sourcePack.urls.length} URL(s), ${sourcePack.files.length} file(s), ${sourcePack.references.length} reference(s), and ${sourcePack.reviewerComments.length} reviewer comment(s).`
      : "",
    sourcePack.markdown ? `Structured source pack:\n${sourcePack.markdown.slice(0, 2200)}` : "",
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

function buildDocumentIntentSheet(input: {
  request: AgenticWorkflowRequest;
  corpus: string;
  title: string;
  documentType: DocsLiveDocumentType;
  lanes: AgenticWorkflowLane[];
  distributionTargets: ExportTarget[];
  contextCompleteness: AgenticContextCompleteness;
}): AgenticDocumentIntentSheet {
  const { request, corpus, title, documentType, lanes, distributionTargets, contextCompleteness } = input;
  const corpusParts = [
    { source: "context" as const, text: request.contextAnswers || "" },
    { source: "instruction" as const, text: request.instruction || "" },
    { source: "source-pack" as const, text: request.sourcePackText || "" },
    { source: "current-document" as const, text: [request.documentTitle || "", request.documentText || "", request.selectedText || ""].join("\n") },
  ];
  const fieldProfiles: Array<{
    key: string;
    label: string;
    aliases: string[];
    fallback?: string;
    source?: AgenticDocumentIntentField["source"];
    required: boolean;
    guidance: string;
  }> = [
    {
      key: "document-type",
      label: "Document type",
      aliases: ["document type", "type"],
      fallback: docsLiveDocumentTypes.find((type) => type.id === documentType)?.label || titleCase(documentType),
      source: "derived",
      required: true,
      guidance: "Confirm the document type because it controls outline defaults, quality gates, and review expectations.",
    },
    {
      key: "title",
      label: "Working title",
      aliases: ["title", "document title", "called", "titled"],
      fallback: title,
      source: request.documentTitle ? "current-document" : "derived",
      required: true,
      guidance: "Use a specific working title so drafts, exports, and review packets are traceable.",
    },
    {
      key: "audience",
      label: "Audience",
      aliases: ["audience", "reader", "recipient", "stakeholder"],
      required: true,
      guidance: "Name the reader or decision-maker before drafting tone, detail, and calls to action.",
    },
    {
      key: "outcome",
      label: "Outcome",
      aliases: ["outcome", "decision", "goal", "purpose", "reader should", "objective"],
      required: true,
      guidance: "State the decision, approval, understanding, or action the document should produce.",
    },
    {
      key: "owner",
      label: "Owner",
      aliases: ["owner", "accountable owner", "author"],
      required: true,
      guidance: "Assign an accountable owner for follow-up, review, and export handoff.",
    },
    {
      key: "deadline",
      label: "Deadline",
      aliases: ["deadline", "due", "due date", "needed by"],
      required: true,
      guidance: "Capture timing so the agent can prioritize completeness, review, and distribution readiness.",
    },
    {
      key: "tone",
      label: "Tone",
      aliases: ["tone", "voice", "style"],
      fallback: inferTone(corpus),
      source: "derived",
      required: false,
      guidance: "Confirm tone and detail level before humanization or executive review.",
    },
    {
      key: "evidence",
      label: "Evidence",
      aliases: ["evidence", "source", "sources", "data", "proof", "reference"],
      required: true,
      guidance: "Name the source material that will support material claims, numbers, dates, and risks.",
    },
    {
      key: "reviewer",
      label: "Reviewer",
      aliases: ["reviewer", "approver", "reviewed by", "approved by"],
      required: Boolean(distributionTargets.length || lanes.includes("review")),
      guidance: "Name who must inspect the draft before it becomes approved or publishable.",
    },
    {
      key: "approval-status",
      label: "Approval status",
      aliases: ["status", "approval status", "release status"],
      required: Boolean(distributionTargets.length),
      guidance: "Distribution needs explicit draft, in-review, approved, published, or archived status.",
    },
    {
      key: "distribution",
      label: "Distribution targets",
      aliases: ["distribution", "export", "publish", "release target"],
      fallback: distributionTargets.length ? distributionTargets.join(", ") : "",
      source: distributionTargets.length ? "derived" : "missing",
      required: lanes.includes("distribute"),
      guidance: "Select target outputs before export readiness, manifests, and publishing handoffs.",
    },
    {
      key: "constraints",
      label: "Constraints",
      aliases: ["constraints", "requirements", "must", "must not", "risk", "risks", "legal", "budget"],
      required: false,
      guidance: "Capture policy, legal, budget, risk, confidentiality, or formatting limits before provider handoff.",
    },
  ];

  const fields = fieldProfiles.map((profile) => {
    const found = findIntentValue(corpusParts, profile.aliases);
    const value = found?.value || profile.fallback || "";
    const status: AgenticDocumentIntentFieldStatus = value
      ? found || profile.source !== "missing"
        ? profile.required && !found && profile.source === "derived"
          ? "needs-review"
          : "provided"
        : "missing"
      : profile.required
        ? "missing"
        : "needs-review";
    return {
      key: profile.key,
      label: profile.label,
      value: value || "TBD",
      status,
      source: found?.source || profile.source || (value ? "derived" : "missing"),
      guidance: profile.guidance,
    };
  });
  const requiredFields = fields.filter((field) => field.key !== "constraints" && field.key !== "tone");
  const missingFields = fields.filter((field) => field.status === "missing").map((field) => field.label);
  const providedWeight = requiredFields.filter((field) => field.status === "provided").length;
  const reviewWeight = requiredFields.filter((field) => field.status === "needs-review").length * 0.5;
  const completenessScore = Math.round(((providedWeight + reviewWeight) / Math.max(1, requiredFields.length)) * 100);
  const reviewPrompts = [
    ...fields
      .filter((field) => field.status !== "provided")
      .map((field) => `${field.label}: ${field.guidance}`),
    contextCompleteness.status === "thin" ? "Context completeness is thin; ask the user for the missing intent fields before drafting final copy." : "",
  ].filter(Boolean);
  const status: AgenticDocumentIntentSheet["status"] = missingFields.some((label) => ["Audience", "Outcome", "Owner", "Evidence"].includes(label))
    ? "needs-input"
    : "ready";
  const summary =
    status === "ready"
      ? `Document intent is ready for ${titleCase(lanes[0] || "create")} with ${completenessScore}/100 completeness.`
      : `Document intent needs ${missingFields.join(", ") || "review"} before responsible drafting or distribution.`;
  const markdown = [
    "| Field | Value | Status | Source | Guidance |",
    "| --- | --- | --- | --- | --- |",
    ...fields.map(
      (field) =>
        `| ${escapeTableCell(field.label)} | ${escapeTableCell(field.value)} | ${field.status} | ${field.source} | ${escapeTableCell(field.guidance)} |`,
    ),
  ].join("\n");
  return {
    summary,
    completenessScore,
    status,
    fields,
    missingFields,
    reviewPrompts,
    markdown,
  };
}

function findIntentValue(
  corpusParts: Array<{ source: AgenticDocumentIntentField["source"]; text: string }>,
  aliases: string[],
): { value: string; source: AgenticDocumentIntentField["source"] } | null {
  for (const part of corpusParts) {
    if (!part.text.trim()) continue;
    for (const alias of aliases) {
      const value = extractIntentKeyValue(part.text, alias);
      if (value) return { value, source: part.source };
    }
  }
  return null;
}

function extractIntentKeyValue(text: string, alias: string) {
  const escaped = alias.replace(/[.*+?^${}()|[\]\\]/g, "\\$&").replace(/\s+/g, "\\s+");
  const keyValue = text.match(new RegExp(`\\b${escaped}\\b\\s*(?:is|=|:|should be|should)\\s*([^\\n.]+)`, "i"))?.[1]?.trim();
  if (keyValue) return cleanIntentValue(keyValue);
  if (alias === "outcome" || alias === "decision" || alias === "purpose" || alias === "goal") {
    const purpose = text.match(/\b(?:to|so that|in order to)\s+([^.\n]{8,180})/i)?.[1]?.trim();
    if (purpose) return cleanIntentValue(purpose);
  }
  if (alias === "audience" || alias === "reader" || alias === "recipient") {
    const audience = text.match(/\bfor\s+(?:the\s+)?([^.\n]{4,100}?)(?=\s+(?:owner|deadline|tone|evidence|reviewer)\s*(?:is|=|:)|[.\n]|$)/i)?.[1]?.trim();
    if (audience) return cleanIntentValue(audience);
  }
  return "";
}

function cleanIntentValue(value: string) {
  return value
    .replace(/\s+(?:audience|owner|deadline|tone|evidence|reviewer|status|distribution|release target)\s*(?:is|=|:).+$/i, "")
    .replace(/\s+/g, " ")
    .trim()
    .slice(0, 180);
}

function scoreContextCompleteness(corpus: string, context: string, placeholderText: string, targets: ExportTarget[]): AgenticContextCompleteness {
  const combined = [corpus, context, placeholderText].join("\n");
  const checks = [
    contextCompletenessCheck("audience", combined, /\baudience\s*(?:is|=|:)|\bfor\s+(?:the\s+)?(?:board|executive|customer|client|reader|team|committee)\b/i),
    contextCompletenessCheck("evidence", combined, /\bevidence\s*(?:is|=|:)|\b(source|data|metric|forecast|research|citation|reference)\b/i),
    contextCompletenessCheck("constraints", combined, /\b(constraint|risk|must|must not|requirement|policy|legal|budget|deadline)\b/i),
    contextCompletenessCheck("examples", combined, /\b(example|sample|reference doc|prior version|style guide|template)\b/i),
    contextCompletenessCheck("tone", combined, /\btone\s*(?:is|=|:)|\b(concise|formal|plain language|executive|technical|legal|friendly)\b/i),
    contextCompletenessCheck("approval", combined, /\b(approval|approved|reviewer|approver|sign[- ]?off|release status)\b/i),
  ];
  const present = checks.filter((check) => check.present).map((check) => check.label);
  const missing = checks.filter((check) => !check.present).map((check) => check.label);
  const score = Math.round((present.length / checks.length) * 100);
  const status: AgenticContextCompleteness["status"] = score >= 80 ? "strong" : score >= 50 ? "usable" : "thin";
  const recommendations = [
    missing.includes("audience") ? "Name the audience or decision-maker before drafting." : "",
    missing.includes("evidence") ? "Add source facts, data, citations, or evidence expectations." : "",
    missing.includes("constraints") ? "Capture risks, constraints, deadlines, budget, legal, or policy limits." : "",
    missing.includes("examples") ? "Attach an example, style reference, prior version, or template when tone/shape matters." : "",
    missing.includes("tone") ? "State the tone and detail level expected by the reader." : "",
    missing.includes("approval") || targets.length ? "Identify reviewer, approver, release status, or sign-off expectations." : "",
  ].filter(Boolean);
  return { score, status, present, missing, recommendations };
}

function contextCompletenessCheck(label: string, value: string, pattern: RegExp) {
  return { label, present: pattern.test(value) && !new RegExp(`\\bTBD\\s+${label}\\b`, "i").test(value) };
}

function buildSuggestedOutline(request: AgenticWorkflowRequest, primaryLane: AgenticWorkflowLane, documentType: DocsLiveDocumentType) {
  const existingOutline = request.documentText ? outlinePlanFromMarkdown(request.documentText) : "";
  if (existingOutline && primaryLane !== "create") return existingOutline;
  const typeLabel = docsLiveDocumentTypes.find((type) => type.id === documentType)?.label || titleCase(primaryLane);
  const sections = defaultOutlineByLane[primaryLane] || defaultOutlineByLane.create;
  return [`- ${typeLabel}`, ...sections.map((section) => `  - ${section}`)].join("\n");
}

function buildOutlineVariants(input: {
  suggestedOutline: string;
  documentType: DocsLiveDocumentType;
  distributionTargets: ExportTarget[];
  contextCompleteness: AgenticContextCompleteness;
  documentIntent: AgenticDocumentIntentSheet;
  sourcePack: AgenticSourcePack;
  documentMemory: AgenticDocumentMemory;
}): AgenticOutlineVariant[] {
  const baseSections = parseOutlineSections(input.suggestedOutline);
  const root = baseSections[0]?.heading || docsLiveDocumentTypes.find((type) => type.id === input.documentType)?.label || titleCase(input.documentType);
  const content = baseSections.length > 1 ? baseSections.slice(1).map((section) => section.heading) : baseSections.map((section) => section.heading);
  const evidenceAnchor = content.find((heading) => /\b(evidence|findings|analysis|financial|data|metrics?|source|proof)\b/i.test(heading)) || "Evidence And Findings";
  const riskAnchor = content.find((heading) => /\b(risks?|assumptions?|constraints?|mitigation|legal|compliance)\b/i.test(heading)) || "Risks And Assumptions";
  const actionAnchor = content.find((heading) => /\b(recommendation|decision|approval|ask|next steps?|handoff|distribution)\b/i.test(heading)) || "Recommendation And Next Steps";
  const contextAnchor = content.find((heading) => /\b(context|background|current state|problem|need)\b/i.test(heading)) || "Context And Reader Need";
  const hasPublishingTarget = input.distributionTargets.some((target) => target === "blog" || target === "substack" || target === "html");
  const hasTechnicalTarget = input.distributionTargets.some((target) => target === "latex" || target === "google-docs") || /\b(technical|architecture|research|paper|spec)\b/i.test(input.documentType);
  const sourceDetail = input.sourcePack.claims.length ? `${input.sourcePack.claims.length} managed claim(s)` : input.contextCompleteness.present.includes("evidence") ? "provided evidence" : "evidence still to confirm";
  const memoryDetail = input.documentMemory.entries.length ? "document memory" : "current context";

  const variants: Array<Omit<AgenticOutlineVariant, "id">> = [
    {
      label: "Executive-first",
      strategy: "executive-first",
      summary: "Lead with the decision, then give context, evidence, risk, and handoff details.",
      outline: outlineVariantText(root, ["Decision Snapshot", actionAnchor, contextAnchor, evidenceAnchor, riskAnchor, "Approval And Handoff"]),
      bestFor: ["board review", "executive committee", "time-constrained approvers"],
      tradeoffs: ["Fastest route to the ask.", "May feel abrupt if the audience needs more context first."],
      risks: ["Weak if decision language is not backed by source evidence.", `Requires ${sourceDetail} to be checked before release.`],
    },
    {
      label: "Problem-solution",
      strategy: "problem-solution",
      summary: "Build agreement around the problem before introducing the solution and next actions.",
      outline: outlineVariantText(root, [contextAnchor, "Impact And Stakes", "Options Considered", actionAnchor, riskAnchor, "Implementation Plan"]),
      bestFor: ["client proposals", "strategy memos", "change management"],
      tradeoffs: ["Creates narrative buy-in.", "Takes longer to reach the recommendation."],
      risks: ["Can bury the ask if the recommendation section is too late.", "Needs explicit owner and deadline before approval."],
    },
    {
      label: "Evidence-led",
      strategy: "evidence-led",
      summary: "Put source-backed findings before interpretation, recommendation, and distribution.",
      outline: outlineVariantText(root, ["Source Basis", evidenceAnchor, "Interpretation", actionAnchor, riskAnchor, "Review Questions"]),
      bestFor: ["research briefs", "financial cases", "claims-heavy documents"],
      tradeoffs: ["Makes audit and citation review easier.", "Can read dry without a strong executive framing section."],
      risks: [`Depends on ${sourceDetail}.`, "Unsupported claims should remain citation TODOs until verified."],
    },
    {
      label: "Risk-first",
      strategy: "risk-first",
      summary: "Surface constraints, approvals, and sensitive assumptions before asking for action.",
      outline: outlineVariantText(root, ["Approval Context", riskAnchor, contextAnchor, evidenceAnchor, actionAnchor, "Controls And Follow-up"]),
      bestFor: ["policies", "regulated material", "high-stakes approvals"],
      tradeoffs: ["Reduces review surprise.", "Can feel defensive if risks are not tied to decisions."],
      risks: ["Needs governance review before publishing.", "May slow drafting until owner and reviewer are named."],
    },
    {
      label: hasTechnicalTarget ? "Technical-deep" : "Publishing narrative",
      strategy: hasTechnicalTarget ? "technical-deep" : "publishing-narrative",
      summary: hasTechnicalTarget
        ? "Separate method, architecture, evidence, and submission details for technical review."
        : "Use a reader-friendly narrative arc for blog, newsletter, or web publishing.",
      outline: hasTechnicalTarget
        ? outlineVariantText(root, ["Abstract", "Method Or Architecture", evidenceAnchor, "Implementation Details", riskAnchor, "References And Submission Notes"])
        : outlineVariantText(root, ["Hook", contextAnchor, evidenceAnchor, "What This Means", actionAnchor, "Publishing Metadata"]),
      bestFor: hasTechnicalTarget ? ["technical papers", "architecture docs", "LaTeX exports"] : ["blog posts", "Substack", "public web pages"],
      tradeoffs: hasTechnicalTarget
        ? ["Improves technical review and export handoff.", "Requires more source discipline before drafting."]
        : ["Improves reader momentum.", "Needs careful humanization to avoid generic thought-leadership tone."],
      risks: hasPublishingTarget
        ? [`Must apply ${memoryDetail} to preserve voice and rejected directions.`, "Publishing metadata and link checks remain required."]
        : [`Must apply ${memoryDetail} to terminology and reviewer preferences.`, "References, labels, and equations need export checks when present."],
    },
  ];

  return variants.map((variant) => ({
    ...variant,
    id: `outline-${variant.strategy}-${stableFingerprint([variant.label, variant.outline, input.documentType].join("\n")).slice(0, 10)}`,
  }));
}

function outlineVariantText(root: string, headings: string[]) {
  const uniqueHeadings = Array.from(new Set(headings.map((heading) => heading.trim()).filter(Boolean)));
  return [`- ${root}`, ...uniqueHeadings.map((heading) => `  - ${heading}`)].join("\n");
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

function buildReviewChecklist(
  plan: AgenticWorkflowPlan,
  revision: AgenticWorkflowRevision | null,
  editAcceptanceQueue: AgenticEditAcceptanceItem[],
  documentEvidence: AgenticDocumentEvidence,
) {
  return [
    "Confirm the document has a clear audience, decision, owner, deadline, and review status.",
    "Replace unresolved placeholders with verified names, dates, numbers, and source references.",
    "Check each factual claim against the evidence or add a citation TODO before export.",
    "Run the pre-review rehearsal and answer likely reviewer questions, objections, redlines, and missing-evidence requests.",
    "Mark every AI source block and AI-assisted section human-reviewed only after a person verifies it.",
    revision ? "Compare the revision proposal against the original text before applying final edits." : "",
    revision?.revisionPasses.length ? `Complete revision passes before acceptance: ${revision.revisionPasses.map((pass) => pass.label).join(", ")}.` : "",
    editAcceptanceQueue.length ? "Accept, reject, or request revision for each edit acceptance queue item before applying changes." : "",
    ...plan.qualityGates.map((gate) => `${gate.label}: ${gate.detail}`),
    documentEvidence.reviewCommentResolutions.length ? "Resolve, answer, or deliberately carry forward every review comment resolution queue item before release." : "",
    revision?.meaningDriftFindings.length ? "Resolve all meaning-drift findings before accepting the revision." : "",
    plan.distributionTargets.length ? "Run export readiness for every requested distribution target." : "",
  ].filter(Boolean);
}

function buildQualityGates(documentType: DocsLiveDocumentType, distributionTargets: ExportTarget[]): AgenticQualityGate[] {
  const profile = qualityGateProfiles()[documentType] || qualityGateProfiles()["business-brief"];
  const gates: AgenticQualityGate[] = profile.map((gate, index) => ({
    ...gate,
    id: `quality-${documentType}-${index + 1}-${stableFingerprint(`${gate.label}\n${gate.detail}`).slice(0, 8)}`,
    appliesTo: documentType,
  }));
  if (distributionTargets.includes("blog")) gates.push(publishingQualityGate("blog"));
  if (distributionTargets.includes("substack")) gates.push(publishingQualityGate("newsletter"));
  if (distributionTargets.length) gates.push(distributionQualityGate(distributionTargets));
  return gates.slice(0, 10);
}

function qualityGateProfiles(): Record<DocsLiveDocumentType, Array<Omit<AgenticQualityGate, "id" | "appliesTo">>> {
  return {
    "business-brief": [
      qualityGate("Reader Decision", "The brief states the requested decision, recommendation, owner, and deadline in the first screen.", [
        "Decision or action named",
        "Owner and due date present",
      ]),
      qualityGate("Evidence Spine", "Every material claim, metric, or risk has a source note or citation TODO.", ["Claims checked", "Evidence or TODO present"]),
      qualityGate("Executive Scan", "Headings, bullets, and summary language support a five-minute executive read.", ["Scannable headings", "No generic filler"]),
    ],
    "board-memo": [
      qualityGate("Board Decision", "The memo identifies the board decision, requested approval, and consequence of delay.", [
        "Decision requested",
        "Approval language reviewed",
      ]),
      qualityGate("Financial Case", "Investment, cost, risk, return, and assumptions are visible and source-backed.", ["Financial evidence attached", "Assumptions named"]),
      qualityGate("Director Challenge", "Strategic alternatives, dissent, and key risks are explicit enough for director scrutiny.", ["Alternatives considered", "Risks and mitigations reviewed"]),
    ],
    proposal: [
      qualityGate("Client Need", "The proposal ties the offer to the client's stated problem, value, and success criteria.", ["Client problem named", "Success criteria listed"]),
      qualityGate("Scope Discipline", "In-scope, out-of-scope, timeline, dependencies, and acceptance criteria are explicit.", ["Scope boundaries present", "Acceptance criteria present"]),
      qualityGate("Commercial Review", "Pricing, assumptions, obligations, and approval terms are checked before client handoff.", ["Investment reviewed", "Terms and assumptions reviewed"]),
    ],
    "strategy-plan": [
      qualityGate("Strategic Choice", "The plan names the strategic bet, alternatives rejected, and tradeoffs.", ["Choice stated", "Rejected alternatives captured"]),
      qualityGate("Execution Measures", "Roadmap, owners, milestones, and measurable outcomes are tied to evidence.", ["Owners and milestones present", "Metrics source-backed"]),
      qualityGate("Market Reality", "Market forces, constraints, risks, and assumptions are visible.", ["Market evidence present", "Assumptions reviewed"]),
    ],
    "project-plan": [
      qualityGate("Delivery Ownership", "Outcomes, workstreams, owners, dependencies, and milestones are assigned.", ["Owners named", "Milestones listed"]),
      qualityGate("Risk Controls", "Risks, mitigations, escalation path, and governance cadence are explicit.", ["Risks reviewed", "Escalation path present"]),
      qualityGate("Scope Integrity", "Scope, exclusions, and acceptance criteria are clear enough for delivery tracking.", ["Scope boundaries present", "Acceptance criteria present"]),
    ],
    "research-brief": [
      qualityGate("Research Question", "The brief states the question, method, source boundaries, and confidence level.", ["Question stated", "Method and source boundaries present"]),
      qualityGate("Finding Traceability", "Findings distinguish evidence, interpretation, gaps, and open questions.", ["Findings source-backed", "Evidence gaps listed"]),
      qualityGate("Reference Readiness", "Citations, bibliography expectations, and unsupported claims are marked before handoff.", ["References present", "Citation TODOs resolved or explicit"]),
    ],
    policy: [
      qualityGate("Policy Authority", "Purpose, scope, required behavior, approver, and exception owner are explicit.", ["Scope and requirements present", "Approver named"]),
      qualityGate("Compliance Safety", "Must/shall language, obligations, exceptions, and review cadence are checked for unintended commitments.", [
        "Obligations reviewed",
        "Exceptions path present",
      ]),
      qualityGate("Operational Fit", "Roles, responsibilities, controls, and rollout implications are clear to non-technical users.", ["Roles named", "Controls listed"]),
    ],
    "meeting-brief": [
      qualityGate("Meeting Outcome", "Purpose, decisions needed, attendees, and follow-ups are visible.", ["Outcome stated", "Decisions listed"]),
      qualityGate("Preparation Readiness", "Pre-reads, context, agenda, and open questions are ready before the meeting.", ["Pre-reads listed", "Open questions present"]),
      qualityGate("Action Capture", "Owners, deadlines, and follow-up expectations are explicit.", ["Owners named", "Dates present"]),
    ],
    "business-case": [
      qualityGate("Investment Decision", "Problem, options, recommendation, ROI, and implementation path are decision-ready.", ["Options compared", "ROI assumptions present"]),
      qualityGate("Assumption Integrity", "Financial and operational assumptions are sourced and caveated.", ["Assumptions source-backed", "Caveats present"]),
      qualityGate("Implementation Risk", "Dependencies, risks, owners, and next steps are explicit.", ["Dependencies listed", "Owners named"]),
    ],
    "operating-procedure": [
      qualityGate("Procedure Completeness", "Prerequisites, inputs, ordered steps, outputs, controls, and exceptions are complete.", ["Prerequisites present", "Controls and exceptions listed"]),
      qualityGate("Operator Usability", "Steps are unambiguous, testable, and written for the role performing the work.", ["Role named", "Steps are action-oriented"]),
      qualityGate("Change Control", "Revision history, owner, review cycle, and escalation path are visible.", ["Owner present", "Review cadence present"]),
    ],
    "technical-architecture": [
      qualityGate("Architecture Traceability", "Context, constraints, decisions, alternatives, and tradeoffs are explicit.", ["Constraints listed", "Alternatives reviewed"]),
      qualityGate("Trust Boundaries", "Data flows, security boundaries, dependencies, and failure modes are reviewed.", ["Data flow present", "Security review notes present"]),
      qualityGate("Implementation Handoff", "Risks, open questions, operational concerns, and review owners are clear.", ["Open questions listed", "Owners named"]),
    ],
    adr: [
      qualityGate("Decision Record", "Context, decision, options considered, consequences, and status are complete.", ["Decision stated", "Consequences listed"]),
      qualityGate("Reviewability", "Assumptions, constraints, and rejected alternatives are easy to audit later.", ["Rejected options present", "Constraints listed"]),
      qualityGate("Implementation Link", "Follow-up tasks, owners, and related architecture references are captured.", ["Owners named", "Related references present"]),
    ],
    "release-notes": [
      qualityGate("Audience Segmentation", "User-facing changes, technical changes, fixes, and breaking changes are separated.", ["Change groups present", "Audience clear"]),
      qualityGate("Upgrade Safety", "Migration steps, compatibility notes, risks, and rollback guidance are visible.", ["Migration notes present", "Rollback guidance present"]),
      qualityGate("Evidence and Links", "Issue links, version, date, and verification notes are attached.", ["Version and date present", "Verification notes present"]),
    ],
    "contract-brief": [
      qualityGate("Commercial Terms", "Parties, scope, obligations, dates, payment terms, and approvals are visible.", ["Parties named", "Obligations reviewed"]),
      qualityGate("Risk and Exceptions", "Liability, renewal, termination, exceptions, and open legal questions are flagged.", ["Legal questions listed", "Risk terms reviewed"]),
      qualityGate("Approval Path", "Reviewer, approver, status, and handoff deadline are explicit.", ["Reviewer named", "Approval status present"]),
    ],
    "marketing-brief": [
      qualityGate("Audience Offer Fit", "Audience, offer, positioning, proof points, and call to action are aligned.", ["Audience named", "CTA present"]),
      qualityGate("Claim Safety", "Customer claims, statistics, testimonials, and competitive statements are sourced.", ["Proof points sourced", "Unsupported claims marked"]),
      qualityGate("Channel Readiness", "Tone, assets, metadata, and distribution channel requirements are listed.", ["Assets listed", "Channel requirements present"]),
    ],
    "customer-case-study": [
      qualityGate("Customer Consent", "Customer name, permissions, quote approvals, and anonymization needs are explicit.", ["Consent status present", "Quotes approved or marked"]),
      qualityGate("Outcome Evidence", "Problem, solution, metrics, and customer outcomes are traceable to sources.", ["Metrics sourced", "Customer proof present"]),
      qualityGate("Story Integrity", "Narrative avoids exaggeration and separates customer quote from company interpretation.", ["Quotes separated", "Claims reviewed"]),
    ],
  };
}

function qualityGate(label: string, detail: string, evidenceRequired: string[]): Omit<AgenticQualityGate, "id" | "appliesTo"> {
  return { label, detail, evidenceRequired };
}

function publishingQualityGate(appliesTo: "blog" | "newsletter"): AgenticQualityGate {
  const isNewsletter = appliesTo === "newsletter";
  return {
    id: `quality-${appliesTo}`,
    appliesTo,
    label: isNewsletter ? "Newsletter Handoff" : "Blog Handoff",
    detail: isNewsletter
      ? "Subject line, preview text, excerpt, CTA, links, and Substack-safe copy are ready for publishing."
      : "Slug, excerpt, tags, canonical URL, CTA, links, and CMS-safe copy are ready for publishing.",
    evidenceRequired: isNewsletter
      ? ["Subject line and preview text present", "Links and CTA checked"]
      : ["Slug/excerpt/tags present", "Canonical URL and links checked"],
  };
}

function distributionQualityGate(targets: ExportTarget[]): AgenticQualityGate {
  return {
    id: "quality-distribution-readiness",
    appliesTo: "distribution",
    label: "Distribution Readiness",
    detail: `Requested targets (${targets.join(", ")}) have approval metadata, export readiness, and artifact evidence requirements staged.`,
    evidenceRequired: ["Status, approver or reviewer, approvedAt, owner, and releaseTarget metadata present", "Target artifacts or runbooks verified"],
  };
}

function buildDistributionTargetPlans(plan: AgenticWorkflowPlan): AgenticDistributionTargetPlan[] {
  return plan.distributionTargets.map((target) => {
    const profile = distributionProfile(target);
    return {
      target,
      label: profile.label,
      purpose: profile.purpose,
      preflightChecks: [
        "Confirm approved or published status, approver or reviewer, approvedAt, owner, releaseTarget, and export metadata are present.",
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

function buildApprovalGate(plan: AgenticWorkflowPlan, documentEvidence: AgenticDocumentEvidence, documentText: string): AgenticApprovalGate {
  const requiredBeforeDistribution = Boolean(plan.distributionTargets.length || plan.lanes.includes("distribute"));
  const targets = plan.distributionTargets.join(", ");
  const owner =
    approvalMetadataValue(documentText, ["owner"]) ||
    extractKeyValue(plan.placeholderText, "owner") ||
    extractIntentField(plan.documentIntent, "owner");
  const reviewer =
    approvalMetadataValue(documentText, ["approvedBy", "reviewer"]) ||
    extractKeyValue(plan.placeholderText, "reviewer") ||
    extractIntentField(plan.documentIntent, "reviewer");
  const releaseTarget =
    approvalMetadataValue(documentText, ["releaseTarget"]) ||
    extractKeyValue(plan.placeholderText, "releaseTarget") ||
    extractIntentField(plan.documentIntent, "distribution") ||
    targets;
  const sourceConfidence =
    approvalMetadataValue(documentText, ["sourceConfidence"]) ||
    extractKeyValue(plan.placeholderText, "sourceConfidence") ||
    extractKeyValue(plan.context, "sourceConfidence");
  const fields: AgenticApprovalGateField[] = [
    approvalStatusField(approvalMetadataValue(documentText, ["status"]) || extractKeyValue(plan.placeholderText, "status") || ""),
    approvalRequiredField(
      "reviewer",
      "Reviewer or approver",
      reviewer,
      "Name the human reviewer or approver accountable for the final distribution decision.",
    ),
    approvalRequiredField("approvedAt", "Approved at", approvalMetadataValue(documentText, ["approvedAt"]), "Record the approval timestamp or date."),
    approvalRequiredField("owner", "Owner", owner, "Identify the accountable business owner for the document."),
    approvalRequiredField("releaseTarget", "Release target", releaseTarget, "Match the approval to the target export or publishing channel."),
    approvalSourceConfidenceField(sourceConfidence || ""),
  ];
  const blockers = requiredBeforeDistribution
    ? [
        ...fields
          .filter((field) => field.status !== "present")
          .map((field) => `${field.label}: ${field.guidance}`),
        ...(documentEvidence.unresolvedComments
          ? [`Unresolved comments: close ${documentEvidence.unresolvedComments} review comment(s) before release approval.`]
          : []),
        ...(documentEvidence.unreviewedAiMarkers
          ? [`AI provenance review: mark ${documentEvidence.unreviewedAiMarkers} AI-assisted marker(s) human-reviewed before release.`]
          : []),
        ...(documentEvidence.citationTodos.length
          ? [`Source confidence: resolve ${documentEvidence.citationTodos.length} citation TODO marker(s) before declaring source confidence.`]
          : []),
      ]
    : [];
  const status: AgenticApprovalGateStatus = blockers.length ? "blocked" : fields.some((field) => field.status !== "present") ? "needs-review" : "ready";
  return {
    status,
    requiredBeforeDistribution,
    fields,
    blockers,
    summary: requiredBeforeDistribution
      ? blockers.length
        ? `${blockers.length} approval gate blocker(s) must be cleared before distribution to ${targets || "the selected target"}.`
        : `Approval metadata is complete for distribution to ${targets || "the selected target"}.`
      : status === "ready"
        ? "Approval metadata is complete, though no distribution target is active."
        : "Approval metadata is staged for review; it becomes required when a distribution target is selected.",
    metadataScaffold: approvalGateMetadataScaffold({
      status: approvalMetadataValue(documentText, ["status"]) || "needs-review",
      reviewer,
      approvedAt: approvalMetadataValue(documentText, ["approvedAt"]),
      owner,
      releaseTarget,
      sourceConfidence: sourceConfidence || "needs-review",
    }),
  };
}

function approvalStatusField(value: string): AgenticApprovalGateField {
  const normalized = value.trim();
  const approved = /^(approved|published|ready|reviewed)$/i.test(normalized);
  return {
    key: "status",
    label: "Status",
    value: normalized,
    status: approved ? "present" : normalized ? "needs-review" : "missing",
    guidance: "Set status to approved, published, ready, or reviewed before distribution.",
  };
}

function approvalRequiredField(
  key: Exclude<AgenticApprovalGateFieldKey, "status" | "sourceConfidence">,
  label: string,
  value: string,
  guidance: string,
): AgenticApprovalGateField {
  const normalized = value.trim();
  return {
    key,
    label,
    value: normalized,
    status: normalized && !/^TBD\b|needs-review|unknown$/i.test(normalized) ? "present" : normalized ? "needs-review" : "missing",
    guidance,
  };
}

function approvalSourceConfidenceField(value: string): AgenticApprovalGateField {
  const normalized = value.trim();
  return {
    key: "sourceConfidence",
    label: "Source confidence",
    value: normalized,
    status: /^(verified|high|confirmed|source-checked|audited)$/i.test(normalized)
      ? "present"
      : normalized
        ? "needs-review"
        : "missing",
    guidance: "State source confidence as verified, high, confirmed, source-checked, or audited.",
  };
}

function approvalMetadataValue(documentText: string, keys: string[]) {
  for (const key of keys) {
    const match = documentText.match(new RegExp(`^${key}:\\s*(.+)$`, "im"));
    const value = match?.[1]?.trim().replace(/^["']|["']$/g, "");
    if (value) return value;
  }
  return "";
}

function approvalGateMetadataScaffold(values: {
  status: string;
  reviewer: string;
  approvedAt: string;
  owner: string;
  releaseTarget: string;
  sourceConfidence: string;
}) {
  return [
    "---",
    "# approval-gate: complete before export, publishing, or external handoff",
    `status: ${values.status || "needs-review"}`,
    `reviewer: ${values.reviewer || ""}`,
    `approvedAt: ${values.approvedAt || ""}`,
    `owner: ${values.owner || ""}`,
    `releaseTarget: ${values.releaseTarget || ""}`,
    `sourceConfidence: ${values.sourceConfidence || "needs-review"}`,
    "---",
  ].join("\n");
}

function buildLifecycleTasks(input: {
  plan: AgenticWorkflowPlan;
  revision: AgenticWorkflowRevision | null;
  editAcceptanceQueue: AgenticEditAcceptanceItem[];
  reviewerAgents: AgenticReviewerAgent[];
  sectionWorkQueue: AgenticSectionWorkItem[];
  sectionDraftHistory: AgenticSectionDraftHistoryItem[];
  transformRecommendations: AgenticTransformRecommendation[];
  dataNarrativeLinks: AgenticDataNarrativeLink[];
  approvalGate: AgenticApprovalGate;
  automationQueue: AgenticAutomationTask[];
  preReviewRehearsal: AgenticPreReviewRehearsalItem[];
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  blockers: string[];
  documentEvidence: AgenticDocumentEvidence;
  outlineCritique: AgenticOutlineCritiqueItem[];
}): AgenticLifecycleTask[] {
  const { plan, revision, editAcceptanceQueue, reviewerAgents, sectionWorkQueue, sectionDraftHistory, transformRecommendations, dataNarrativeLinks, approvalGate, automationQueue, preReviewRehearsal, distributionTargetPlans, blockers, documentEvidence, outlineCritique } = input;
  const tasks: AgenticLifecycleTask[] = [];
  const hasBlockers = blockers.length > 0;
  const baseStatus: AgenticControlStatus = hasBlockers ? "needs-input" : "ready";
  const evidenceTasks = buildDocumentEvidenceLifecycleTasks(documentEvidence, plan);

  tasks.push({
    id: "task-intake-context",
    lane: "create",
    title: "Resolve intent, context, and placeholder inputs",
    owner: "Planner Agent",
    status: hasBlockers || plan.contextCompleteness.status === "thin" || plan.documentIntent.status === "needs-input" ? "needs-input" : "ready",
    action: "open-docs-live",
    evidence: [
      `Document intent: ${plan.documentIntent.completenessScore}/100 (${plan.documentIntent.status})`,
      `Context completeness: ${plan.contextCompleteness.score}/100 (${plan.contextCompleteness.status})`,
      ...(blockers.length ? blockers.slice(0, 6) : ["Instruction, context pack, placeholders, and outline are available."]),
      ...plan.documentIntent.reviewPrompts.slice(0, 4),
      ...plan.contextCompleteness.recommendations.slice(0, 4),
    ],
    nextStep:
      hasBlockers || plan.contextCompleteness.status === "thin" || plan.documentIntent.status === "needs-input"
        ? "Capture missing intent fields and improve audience, outcome, owner, evidence, constraints, examples, tone, or approval context."
        : "Proceed to outline or section drafting.",
  });

  tasks.push({
    id: "task-outline-structure",
    lane: "compose",
    title: "Lock the working outline and section order",
    owner: "Composition Agent",
    status: plan.suggestedOutline.trim() ? "ready" : "blocked",
    action: "open-outline",
    evidence: plan.suggestedOutline.trim() ? parseOutlineSections(plan.suggestedOutline).slice(0, 6).map((section) => section.heading) : ["No outline supplied."],
    nextStep: "Review the outline in outline mode before drafting body text.",
  });

  if (plan.outlineVariants.length) {
    tasks.push({
      id: "task-outline-variants",
      lane: "compose",
      title: "Compare outline variants before drafting",
      owner: "Composition Agent",
      status: "needs-input",
      action: "open-outline",
      evidence: plan.outlineVariants.map((variant) => `${variant.label}: ${variant.summary}`),
      nextStep: "Choose the outline structure that best matches the audience, evidence, risk, and distribution channel before generating section drafts.",
    });
  }

  if (outlineCritique.length) {
    tasks.push({
      id: "task-outline-critique",
      lane: "compose",
      title: "Resolve outline critique before drafting",
      owner: "Composition Agent",
      status: outlineCritique.some((item) => item.severity === "blocker") ? "blocked" : "needs-input",
      action: "open-outline",
      evidence: outlineCritique.slice(0, 8).map((item) => `${titleCase(item.area)}: ${item.detail}`),
      nextStep: "Address missing, duplicate, weak, or poorly sequenced outline items before section drafting starts.",
    });
  }

  if (preReviewRehearsal.length) {
    tasks.push({
      id: "task-pre-review-rehearsal",
      lane: "review",
      title: "Rehearse likely reviewer objections",
      owner: "Review Lead",
      status: preReviewRehearsal.some((item) => item.releaseBlocker) ? "needs-input" : "ready",
      action: "open-review",
      evidence: preReviewRehearsal.slice(0, 10).map((item) => `${titleCase(item.kind)} from ${titleCase(item.reviewer)} reviewer: ${item.prompt}`),
      nextStep: "Answer likely reviewer questions, objections, redlines, and missing-evidence requests before formal review.",
    });
  }

  if (revision) {
    tasks.push({
      id: "task-revision-proposal",
      lane: plan.lanes.includes("revise") ? "revise" : "edit",
      title: "Compare and apply the revision proposal",
      owner: "Revision Agent",
      status: revision.meaningDriftFindings.length ? "blocked" : baseStatus,
      action: "open-ai-paste",
      evidence: [
        ...revision.changeSummary,
        ...revision.revisionPasses.map((pass) => `${pass.label} pass: ${pass.rationale}`),
        ...revision.meaningDriftFindings.map((finding) => `${titleCase(finding.kind)} ${finding.severity}: ${finding.detail}`),
      ],
      nextStep: revision.meaningDriftFindings.length
        ? "Resolve meaning-drift findings for changed numbers, dates, commitments, or caveats before accepting the replacement."
        : "Compare original and proposed text before accepting the replacement.",
    });
  }

  if (editAcceptanceQueue.length) {
    tasks.push({
      id: "task-edit-acceptance-queue",
      lane: plan.lanes.includes("revise") ? "revise" : "edit",
      title: "Resolve edit acceptance queue",
      owner: "Revision Agent",
      status: editAcceptanceQueue.some((item) => item.riskNotes.some((note) => /\bblocker\b/i.test(note))) ? "needs-input" : baseStatus,
      action: "open-ai-paste",
      evidence: editAcceptanceQueue
        .slice(0, 8)
        .map((item) => `${titleCase(item.scope)} ${item.heading}: ${item.recommendation}`),
      nextStep: "Accept, reject, or request another revision for each queued edit before applying accepted changes.",
    });
  }

  if (plan.sourcePack.items.length) {
    tasks.push({
      id: "task-source-pack-review",
      lane: "review",
      title: "Validate user source pack",
      owner: "Evidence Agent",
      status: "needs-input",
      action: "open-review",
      evidence: plan.sourcePack.items.slice(0, 8).map((item) => `${titleCase(item.kind)} ${item.label}: ${item.detail}`),
      nextStep: "Confirm each source-pack note, URL, file, reference, reviewer comment, and claim is safe and relevant before provider handoff.",
    });
  }

  if (plan.documentMemory.entries.length) {
    tasks.push({
      id: "task-document-memory-review",
      lane: "review",
      title: "Apply reusable document memory",
      owner: "Planner Agent",
      status: "needs-input",
      action: "open-review",
      evidence: plan.documentMemory.entries.slice(0, 8).map((entry) => `${titleCase(entry.kind)} ${entry.label}: ${entry.detail}`),
      nextStep: "Confirm terminology, style decisions, accepted choices, rejected directions, and review preferences before generating or accepting new content.",
    });
  }

  if (plan.qualityGates.length) {
    tasks.push({
      id: "task-quality-gates",
      lane: "review",
      title: "Complete document-type quality gates",
      owner: "Quality Agent",
      status: "needs-input",
      action: "open-review",
      evidence: plan.qualityGates.slice(0, 10).map((gate) => `${gate.label}: ${gate.evidenceRequired.join("; ")}`),
      nextStep: "Work through every document-type quality gate before review sign-off or distribution.",
    });
  }

  tasks.push(...evidenceTasks);

  const sectionTaskLimit = Math.max(6, 14 - evidenceTasks.length);
  for (const section of sectionWorkQueue.slice(0, sectionTaskLimit)) {
    tasks.push({
      id: `task-${section.id}`,
      lane: section.lane,
      title: `Draft section ${section.order}: ${section.heading}`,
      owner: "Docs Live Section Agent",
      status: baseStatus,
      action: "generate-docs-live-draft",
      sectionId: section.id,
      evidence: [
        `Purpose: ${section.contract.purpose}`,
        `Target reader: ${section.contract.targetReader}`,
        `Desired outcome: ${section.contract.desiredDecision}`,
        `Risk level: ${section.contract.riskLevel}`,
        ...section.completionCriteria,
      ],
      nextStep: "Draft the section against its contract, then route it through assigned reviewer agents.",
    });
  }

  if (sectionDraftHistory.length) {
    tasks.push({
      id: "task-section-draft-history",
      lane: "compose",
      title: "Preserve composable section draft versions",
      owner: "Docs Live Section Agent",
      status: sectionDraftHistory.some((item) => item.acceptanceStatus !== "accepted") ? "needs-input" : "ready",
      action: "generate-docs-live-draft",
      evidence: sectionDraftHistory.slice(0, 10).map((item) => `${item.versionLabel}: ${item.sectionHeading} (${item.sectionFingerprint})`),
      nextStep: "Review saved section versions, accept or revise useful drafts, and keep restore points for rollback before applying final prose.",
    });
  }

  if (transformRecommendations.length) {
    tasks.push({
      id: "task-agent-transform-recommendations",
      lane: "compose",
      title: "Review agent-selected transform recommendations",
      owner: "Transform Agent",
      status: transformRecommendations.some((item) => item.riskLevel === "high") ? "needs-input" : "ready",
      action: "open-review",
      evidence: transformRecommendations.slice(0, 10).map((item) => `${titleCase(item.kind)} ${item.label}: ${item.sourceSignal}`),
      nextStep: "Choose, insert, validate, or reject recommended calc, chart, table, diagram, timeline, schema, equation, or publishing blocks before narrative depends on them.",
    });
  }

  if (dataNarrativeLinks.length) {
    tasks.push({
      id: "task-data-narrative-bridge",
      lane: "review",
      title: "Review data-to-narrative dependencies",
      owner: "Data Narrative Agent",
      status: dataNarrativeLinks.some((item) => item.status === "blocked" || item.status === "needs-review") ? "needs-input" : "ready",
      action: "open-review",
      evidence: dataNarrativeLinks.slice(0, 10).map((item) => `${titleCase(item.sourceKind)} ${item.sourceLabel}: ${item.reviewAction}`),
      nextStep: "Confirm changed claims, calculations, charts, tables, timelines, schemas, or publishing metadata trigger review of every dependent narrative section before export.",
    });
  }

  if (approvalGate.requiredBeforeDistribution || approvalGate.blockers.length) {
    tasks.push({
      id: "task-approval-gate",
      lane: approvalGate.requiredBeforeDistribution ? "distribute" : "review",
      title: "Clear approval metadata gate",
      owner: "Governance Agent",
      status: approvalGate.status === "ready" ? "ready" : approvalGate.status === "blocked" ? "blocked" : "needs-input",
      action: approvalGate.requiredBeforeDistribution ? "prepare-export" : "open-review",
      evidence: [
        approvalGate.summary,
        ...approvalGate.fields.map((field) => `${field.label}: ${field.value || "missing"} (${field.status})`),
        ...approvalGate.blockers.slice(0, 8),
      ],
      nextStep: approvalGate.blockers.length
        ? "Complete the approval metadata scaffold, close unresolved comments, confirm source confidence, and regenerate export readiness before distribution."
        : "Attach the approval gate evidence to the release record before final handoff.",
    });
  }

  if (automationQueue.length) {
    tasks.push({
      id: "task-agent-automation-scheduler",
      lane: "review",
      title: "Run safe agent automation queue",
      owner: "Automation Scheduler",
      status: automationQueue.some((item) => item.status === "blocked") ? "blocked" : "needs-input",
      action: "open-review",
      evidence: automationQueue.map((item) => `${item.label}: ${item.safeToAutoRun ? "safe" : "manual"}; ${item.trigger}`),
      nextStep: "Run safe queued checks, attach evidence, and keep destructive or external actions manual.",
    });
  }

  for (const reviewer of reviewerAgents) {
    tasks.push({
      id: `task-reviewer-${reviewer.id}`,
      lane: "review",
      title: `${reviewer.label} sign-off`,
      owner: reviewer.label,
      status: reviewer.status === "blocked" ? "blocked" : reviewer.status === "needs-review" ? "needs-input" : "ready",
      action: "open-review",
      evidence: [...reviewer.findings.slice(0, 3), ...reviewer.requiredActions.slice(0, 3)],
      nextStep: reviewer.requiredActions[0] || "Record reviewer approval and move to distribution readiness.",
    });
  }

  for (const targetPlan of distributionTargetPlans) {
    tasks.push({
      id: `task-distribution-${targetPlan.target}`,
      lane: "distribute",
      title: `Prepare ${targetPlan.label}`,
      owner: "Distribution Agent",
      status: "needs-input",
      action: "prepare-export",
      target: targetPlan.target,
      evidence: [targetPlan.preflightChecks[0], targetPlan.handoffSteps[0], targetPlan.evidenceRequired[0]],
      nextStep: "Run export readiness, generate the target artifact, and attach evidence to the review record.",
    });
  }

  tasks.push({
    id: "task-final-release-readiness",
    lane: "review",
    title: "Final human approval and release readiness",
    owner: "Governance Agent",
    status: hasBlockers || distributionTargetPlans.length ? "needs-input" : "ready",
    action: plan.distributionTargets.length ? "prepare-export" : "open-review",
    evidence: [
      "All AI-assisted sections must be source-checked and marked human-reviewed.",
      "All requested distribution artifacts must retain export manifests or equivalent proof.",
      "Release status, approver or reviewer, approvedAt, owner, and releaseTarget metadata must match the intended handoff.",
    ],
    nextStep: "Resolve outstanding review and distribution evidence before publishing or archiving.",
  });

  return limitLifecycleTasks(tasks, [
    "task-agent-automation-scheduler",
    "task-agent-transform-recommendations",
    "task-data-narrative-bridge",
    "task-approval-gate",
    "task-section-draft-history",
    "task-pre-review-rehearsal",
    "task-final-release-readiness",
    ...distributionTargetPlans.map((targetPlan) => `task-distribution-${targetPlan.target}`),
  ]);
}

const maxLifecycleTaskCount = 36;

function limitLifecycleTasks(tasks: AgenticLifecycleTask[], protectedTaskIds: string[]) {
  if (tasks.length <= maxLifecycleTaskCount) return tasks;
  const protectedIds = new Set(protectedTaskIds);
  const protectedCount = tasks.filter((task) => protectedIds.has(task.id)).length;
  const optionalSlots = Math.max(0, maxLifecycleTaskCount - protectedCount);
  const selectedOptionalIds = new Set(tasks.filter((task) => !protectedIds.has(task.id)).slice(0, optionalSlots).map((task) => task.id));
  return tasks.filter((task) => protectedIds.has(task.id) || selectedOptionalIds.has(task.id));
}

function buildDocumentEvidenceLifecycleTasks(documentEvidence: AgenticDocumentEvidence, plan: AgenticWorkflowPlan): AgenticLifecycleTask[] {
  const tasks: AgenticLifecycleTask[] = [];
  if (documentEvidence.unresolvedPlaceholders.length) {
    tasks.push({
      id: "task-evidence-placeholders",
      lane: "edit",
      title: "Resolve current-document placeholders",
      owner: "Editorial Agent",
      status: "needs-input",
      action: "open-ai-paste",
      evidence: documentEvidence.unresolvedPlaceholders.slice(0, 8),
      nextStep: "Replace placeholders with verified values or mark intentionally deferred placeholders in the review notes.",
    });
  }
  if (documentEvidence.citationTodos.length) {
    tasks.push({
      id: "task-evidence-citations",
      lane: "review",
      title: "Resolve citation TODOs",
      owner: "Evidence Agent",
      status: "needs-input",
      action: "open-review",
      evidence: documentEvidence.citationTodos.slice(0, 8),
      nextStep: "Attach source references to each citation TODO or keep the unresolved citations as explicit release blockers.",
    });
  }
  if (documentEvidence.claimInventory.length) {
    tasks.push({
      id: "task-evidence-claim-inventory",
      lane: "review",
      title: "Verify claim inventory",
      owner: "Evidence Agent",
      status: "needs-input",
      action: "open-review",
      evidence: documentEvidence.claimInventory.slice(0, 8).map((claim) => `Line ${claim.sourceLine}: ${claim.text}`),
      nextStep: "Match each extracted claim to a source, citation, reviewer note, or explicit deferral before final approval.",
    });
  }
  if (documentEvidence.humanizationFindings.length) {
    tasks.push({
      id: "task-evidence-humanization",
      lane: "revise",
      title: "Humanize current-document language",
      owner: "Editorial Agent",
      status: "needs-input",
      action: "open-ai-paste",
      evidence: documentEvidence.humanizationFindings.slice(0, 8).map((finding) => `Line ${finding.sourceLine}: ${finding.text}`),
      nextStep: "Rewrite generic, repetitive, vague, or overconfident language into concrete owner-written prose before approval.",
    });
  }
  if (documentEvidence.unresolvedComments) {
    const comments = documentEvidence.reviewCommentResolutions.slice(0, 8);
    for (const comment of comments) {
      tasks.push({
        id: `task-${comment.id}`,
        lane: "review",
        title: `Resolve review comment on line ${comment.line}`,
        owner: "Risk Reviewer",
        status: comment.blocker ? "blocked" : "needs-input",
        action: "open-review",
        evidence: [
          `Line ${comment.line}${comment.author ? ` by ${comment.author}` : ""}: ${comment.excerpt}`,
          `Required action: ${comment.requiredAction}`,
          ...comment.resolutionOptions.slice(0, 3).map((option) => `Option: ${option}`),
        ],
        nextStep: comment.requiredAction,
      });
    }
    if (documentEvidence.unresolvedComments > comments.length) {
      tasks.push({
        id: "task-evidence-comments-overflow",
        lane: "review",
        title: "Close remaining unresolved review comments",
        owner: "Risk Reviewer",
        status: "needs-input",
        action: "open-review",
        evidence: [`${documentEvidence.unresolvedComments - comments.length} additional unresolved review comment(s) need triage.`],
        nextStep: "Resolve, answer, or deliberately carry forward every remaining review comment before release handoff.",
      });
    }
  }
  if (documentEvidence.unreviewedAiMarkers) {
    tasks.push({
      id: "task-evidence-ai-review",
      lane: "review",
      title: "Mark AI-assisted material human-reviewed",
      owner: "Governance Agent",
      status: "needs-input",
      action: "open-review",
      evidence: [`${documentEvidence.unreviewedAiMarkers} AI provenance marker(s) still need human review.`],
      nextStep: "Inspect the AI-assisted material, preserve provenance, and mark only accepted blocks human-reviewed.",
    });
  }
  if (documentEvidence.brokenLinkHints.length) {
    tasks.push({
      id: "task-evidence-links",
      lane: plan.distributionTargets.length ? "distribute" : "review",
      title: "Repair placeholder or suspicious links",
      owner: "Citation Reviewer",
      status: "needs-input",
      action: plan.distributionTargets.length ? "prepare-export" : "open-review",
      evidence: documentEvidence.brokenLinkHints.slice(0, 8),
      nextStep: "Replace placeholder links with final URLs, anchors, or remove them before publishing.",
    });
  }
  if (documentEvidence.referenceHints.length) {
    tasks.push({
      id: "task-evidence-references",
      lane: "review",
      title: "Repair cross-reference integrity",
      owner: "Citation Reviewer",
      status: "needs-input",
      action: "open-review",
      evidence: documentEvidence.referenceHints.slice(0, 8),
      nextStep: "Fix missing, malformed, or ambiguous labels and cross references before publishing or external review.",
    });
  }
  if (documentEvidence.approvalMetadataMissing.length) {
    tasks.push({
      id: "task-evidence-approval-metadata",
      lane: "distribute",
      title: "Complete approval metadata",
      owner: "Distribution Agent",
      status: "needs-input",
      action: "prepare-export",
      evidence: documentEvidence.approvalMetadataMissing.map((item) => `Missing ${item}`),
      nextStep: "Add approved or published status, approver or reviewer, approvedAt, owner, and releaseTarget metadata before export or publishing handoff.",
    });
  }
  return tasks;
}

function buildSectionWorkQueue(plan: AgenticWorkflowPlan, reviewerAgents: AgenticReviewerAgent[]): AgenticSectionWorkItem[] {
  const sections = parseOutlineSections(plan.suggestedOutline);
  const evidence = extractKeyValue(plan.placeholderText, "evidence") || "verified source material";
  const activeReviewerIds = new Set(reviewerAgents.map((agent) => agent.id));
  return sections.slice(0, 18).map((section, index) => {
    const reviewerAgentIds = sectionReviewerIds(section.heading, plan).filter((id) => activeReviewerIds.has(id));
    const draftingDepth = sectionDraftingDepth(section.heading, section.level, plan);
    const contract = buildSectionContract(section.heading, section.level, draftingDepth, plan, reviewerAgentIds);
    return {
      id: `section-${String(index + 1).padStart(2, "0")}-${stableFingerprint(section.heading).slice(0, 8)}`,
      order: index + 1,
      heading: section.heading,
      level: section.level,
      lane: plan.lanes.includes("compose") || plan.lanes.includes("create") ? "compose" : plan.primaryLane,
      draftingDepth,
      contract,
      draftingInstruction: [
        `Draft or revise "${section.heading}" for ${contract.targetReader} at ${draftingDepth} depth.`,
        contract.purpose,
        `Use ${evidence} for material claims and name ${contract.owner} where accountability or follow-through is required.`,
        `Drive toward this outcome: ${contract.desiredDecision}`,
        plan.distributionTargets.length ? `Preserve structure and metadata needed for ${plan.distributionTargets.join(", ")} distribution.` : "",
      ]
        .filter(Boolean)
        .join(" "),
      completionCriteria: [
        ...contract.doneCriteria,
        ...contract.evidenceExpectations.map((item) => `Evidence: ${item}`),
        "Reviewer notes identify any remaining decision, source, or approval dependency.",
      ],
      reviewerAgentIds,
    };
  });
}

type AgenticDocsLiveSectionSnapshot = {
  title: string;
  qaFocus: string;
  draftingBrief: string;
  qaSummary: string;
  humanizationNotes: string[];
  reviewQuestions: string[];
  reviewHandoff: string;
};

function buildSectionDraftHistory(input: {
  plan: AgenticWorkflowPlan;
  sectionWorkQueue: AgenticSectionWorkItem[];
  draftSections: AgenticDocsLiveSectionSnapshot[];
  draftMarkdown: string;
  generatedAt: string;
}): AgenticSectionDraftHistoryItem[] {
  const { plan, sectionWorkQueue, draftSections, draftMarkdown, generatedAt } = input;
  if (!sectionWorkQueue.length || !(plan.lanes.includes("create") || plan.lanes.includes("compose"))) return [];
  const draftByTitle = new Map(draftSections.map((section) => [normalizeSectionDraftTitle(section.title), section]));
  return sectionWorkQueue.slice(0, 18).map((section, index) => {
    const matchedDraft = draftByTitle.get(normalizeSectionDraftTitle(section.heading)) || draftSections[index];
    const promptSummary = [
      `Draft "${section.heading}" at ${section.draftingDepth} depth for ${section.contract.targetReader}.`,
      section.contract.desiredDecision,
      section.contract.evidenceExpectations[0] || "",
    ]
      .filter(Boolean)
      .join(" ");
    const rationale = matchedDraft?.draftingBrief || section.contract.purpose;
    const reviewerNotes = Array.from(
      new Set([
        ...(matchedDraft?.reviewQuestions || []),
        matchedDraft?.qaSummary || "",
        matchedDraft?.reviewHandoff || "",
        ...section.reviewerAgentIds.map((id) => `${titleCase(id)} reviewer must confirm this section before final release.`),
      ].filter(Boolean)),
    ).slice(0, 6);
    const restorePointMarkdown = buildSectionDraftRestorePoint(section, matchedDraft, plan, generatedAt);
    const sectionFingerprint = stableFingerprint([section.id, section.heading, promptSummary, restorePointMarkdown].join("\n"));
    return {
      id: `section-draft-${section.id}-${sectionFingerprint.slice(0, 10)}`,
      sectionId: section.id,
      sectionHeading: section.heading,
      generatedAt,
      versionLabel: `v${String(index + 1).padStart(2, "0")} ${section.draftingDepth}`,
      promptSummary,
      rationale,
      reviewerNotes,
      sectionFingerprint,
      sourceFingerprint: stableFingerprint([plan.instruction, plan.context, plan.placeholderText, draftMarkdown].join("\n")),
      restorePointMarkdown,
      acceptanceStatus: "needs-review",
    };
  });
}

function normalizeSectionDraftTitle(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
}

function buildSectionDraftRestorePoint(
  section: AgenticSectionWorkItem,
  matchedDraft: AgenticDocsLiveSectionSnapshot | undefined,
  plan: AgenticWorkflowPlan,
  generatedAt: string,
) {
  const lines = [
    `## ${section.heading}`,
    "",
    "<!-- ai-assisted: needs-review | source: NEditor Agent Workspace | workflow: composable-section-draft-history -->",
    "",
    "```ai-section-draft",
    `sectionId: ${section.id}`,
    `generatedAt: ${generatedAt}`,
    `documentType: ${plan.documentType}`,
    `draftingDepth: ${section.draftingDepth}`,
    `reviewers: ${section.reviewerAgentIds.join(", ") || "editor"}`,
    "status: needs-review",
    "```",
    "",
    matchedDraft?.draftingBrief || section.draftingInstruction,
    "",
    "### Evidence And Review Notes",
    "",
    ...section.contract.evidenceExpectations.map((item) => `- [ ] ${item}`),
    matchedDraft?.qaSummary ? `- [ ] ${matchedDraft.qaSummary}` : "",
    ...(matchedDraft?.humanizationNotes || []).map((item) => `- [ ] ${item}`),
    ...(matchedDraft?.reviewQuestions || []).map((item) => `- [ ] ${item}`),
    "",
    "### Acceptance Criteria",
    "",
    ...section.completionCriteria.map((item) => `- [ ] ${item}`),
    matchedDraft?.reviewHandoff ? `- [ ] ${matchedDraft.reviewHandoff}` : "",
    "",
  ];
  return lines.filter((line) => line !== undefined).join("\n").replace(/\n{3,}/g, "\n\n");
}

function buildTransformRecommendations(input: {
  plan: AgenticWorkflowPlan;
  sectionWorkQueue: AgenticSectionWorkItem[];
  documentEvidence: AgenticDocumentEvidence;
  distributionTargetPlans: AgenticDistributionTargetPlan[];
}): AgenticTransformRecommendation[] {
  const { plan, sectionWorkQueue, documentEvidence, distributionTargetPlans } = input;
  const recommendations: AgenticTransformRecommendation[] = [];
  const corpus = [
    plan.instruction,
    plan.context,
    plan.placeholderText,
    plan.sourcePack.markdown,
    ...plan.sourcePack.items.flatMap((item) => [item.label, item.detail]),
    ...sectionWorkQueue.flatMap((section) => [section.heading, section.contract.purpose, section.contract.desiredDecision]),
    ...documentEvidence.claimInventory.map((claim) => claim.text),
    ...distributionTargetPlans.flatMap((target) => [target.label, ...target.preflightChecks, ...target.evidenceRequired]),
  ]
    .join("\n")
    .toLowerCase();
  const target = (signal: RegExp, fallback = "Current cursor") => {
    const section = sectionWorkQueue.find((item) => signal.test(item.heading) || signal.test(item.contract.purpose));
    return {
      sectionId: section?.id,
      insertionTarget: section ? `Section ${section.order}: ${section.heading}` : fallback,
    };
  };
  const add = (item: Omit<AgenticTransformRecommendation, "id">) => {
    const id = `transform-${item.kind}-${stableFingerprint([item.kind, item.label, item.insertionTarget, item.sourceSignal].join("\n")).slice(0, 10)}`;
    if (recommendations.some((candidate) => candidate.id === id || candidate.label === item.label)) return;
    recommendations.push({ id, ...item });
  };

  if (/\b(roi|payback|investment|budget|financial|finance|forecast|revenue|arr|margin|cost|pricing|runway|cash|npv|irr)\b/.test(corpus)) {
    const location = target(/\b(financial|finance|investment|business case|pricing|budget|cost|forecast|evidence|analysis|recommendation)\b/i);
    add({
      kind: "calc",
      label: "Financial model calculation",
      purpose: "Ground the financial argument in a reusable calculation before prose depends on ROI, margin, forecast, or budget claims.",
      ...location,
      templateId: /\b(runway|cash|burn)\b/.test(corpus) ? "calc-business-runway" : /\b(pricing|price|margin)\b/.test(corpus) ? "calc-business-pricing-sensitivity" : "calc-business-roi",
      sourceSignal: "Financial, ROI, pricing, revenue, ARR, budget, forecast, or runway language detected in the instruction, source pack, or outline.",
      narrativeReviewTrigger: "Any changed financial input must create a review task for the recommendation, risk, and executive summary narrative.",
      evidenceRequired: ["Named source for each input value", "Reviewer confirmation of assumptions", "Export preview with rendered calculation output"],
      riskLevel: "high",
      suggestedMarkdown: [
        "```calc",
        "investment = 125000",
        "annual_benefit = 52000",
        "annual_cost = 9000",
        "net_annual_benefit = annual_benefit - annual_cost",
        "roi = net_annual_benefit / investment",
        "payback_years = investment / net_annual_benefit",
        "```",
        "ROI: {{=roi | percent}}",
        "Payback: {{=payback_years}} years",
      ].join("\n"),
      owner: "Transform Agent",
    });
  }

  if (/\b(kpi|metrics?|forecast|growth|retention|churn|pipeline|conversion|cohort|revenue|arr|trend|series|chart)\b/.test(corpus)) {
    const location = target(/\b(metrics?|kpi|forecast|evidence|findings|analysis|results|growth|retention)\b/i);
    add({
      kind: "chart",
      label: "Metric trend chart",
      purpose: "Turn recurring metrics into a visual checkpoint so the narrative can be reviewed against the plotted values.",
      ...location,
      templateId: /\b(kpi|metrics?)\b/.test(corpus) ? "chart-business-kpi" : "chart-finance-line",
      sourceSignal: "Metric, KPI, forecast, growth, retention, cohort, or trend language detected.",
      narrativeReviewTrigger: "When a chart value changes, review every sentence that interprets trend, causality, risk, or recommendation strength.",
      evidenceRequired: ["Metric definitions", "Source period for every value", "Rendered chart preview before export"],
      riskLevel: "medium",
      suggestedMarkdown: [
        "```chart",
        "type: line",
        "title: Monthly metric trend",
        "x: Month",
        "y: Value",
        "data:",
        "  - [Jan, 120]",
        "  - [Feb, 135]",
        "  - [Mar, 148]",
        "```",
      ].join("\n"),
      owner: "Data Narrative Agent",
    });
  }

  if (/\b(option|tradeoff|comparison|scenario|sensitivity|risk|assumption|vendor|proposal|recommendation|decision)\b/.test(corpus)) {
    const location = target(/\b(option|tradeoff|risk|recommendation|decision|analysis|scenario|assumption)\b/i);
    add({
      kind: "table",
      label: "Decision comparison table",
      purpose: "Make options, assumptions, evidence, and recommendation logic scannable before a reviewer approves the narrative.",
      ...location,
      sourceSignal: "Decision, option, scenario, sensitivity, risk, proposal, or recommendation language detected.",
      narrativeReviewTrigger: "Changing an option score or assumption should reopen the recommendation paragraph and risk register.",
      evidenceRequired: ["Source or owner for each option", "Decision criteria", "Reviewer note for the chosen recommendation"],
      riskLevel: "high",
      suggestedMarkdown: [
        "| Option | Evidence | Upside | Risk | Recommendation |",
        "| --- | --- | --- | --- | --- |",
        "| Option A | Source or owner | Expected benefit | Key risk | Accept / revise / reject |",
        "| Option B | Source or owner | Expected benefit | Key risk | Accept / revise / reject |",
      ].join("\n"),
      owner: "Strategy Agent",
    });
  }

  if (/\b(deadline|timeline|roadmap|implementation|rollout|milestone|phase|schedule|launch|q[1-4]|due)\b/.test(corpus)) {
    const location = target(/\b(timeline|roadmap|implementation|rollout|next steps?|milestone|schedule|launch)\b/i);
    add({
      kind: /\b(roadmap|phase|rollout)\b/.test(corpus) ? "roadmap" : "timeline",
      label: "Milestone timeline",
      purpose: "Convert dates, owners, and dependencies into a structured timeline that the plan narrative can reference safely.",
      ...location,
      templateId: /\b(roadmap|phase|rollout)\b/.test(corpus) ? "roadmap-product-release" : "timeline-project-plan",
      sourceSignal: "Timeline, deadline, implementation, rollout, milestone, schedule, quarter, or launch language detected.",
      narrativeReviewTrigger: "Any moved milestone must reopen owner commitments, deadline claims, and distribution preflight notes.",
      evidenceRequired: ["Owner for every milestone", "Date or date range source", "Dependency or blocker for each critical path item"],
      riskLevel: "medium",
      suggestedMarkdown: [
        "```timeline",
        "title: Delivery timeline",
        "2026-06-01: Confirm scope and source owners",
        "2026-06-15: Complete review-ready draft",
        "2026-06-30: Export and distribution sign-off",
        "```",
      ].join("\n"),
      owner: "Delivery Agent",
    });
  }

  if (/\b(api|openapi|schema|json|data model|architecture|system|integration|workflow|process|diagram|mermaid)\b/.test(corpus)) {
    const location = target(/\b(architecture|api|schema|data model|integration|workflow|process|technical|method)\b/i);
    add({
      kind: /\b(schema|openapi|json)\b/.test(corpus) ? "schema" : "diagram",
      label: "Architecture or schema block",
      purpose: "Represent systems, APIs, data shape, or workflows as structured output before technical prose is finalized.",
      ...location,
      templateId: /\b(openapi|api)\b/.test(corpus) ? "openapi-starter" : /\b(schema|json)\b/.test(corpus) ? "json-schema-object" : "mermaid-process-flow",
      sourceSignal: "API, schema, JSON, architecture, integration, process, workflow, diagram, or Mermaid language detected.",
      narrativeReviewTrigger: "Changing a node, field, endpoint, or edge should trigger review of implementation, dependency, and risk sections.",
      evidenceRequired: ["System or API owner", "Version or interface source", "Rendered diagram/schema validation before export"],
      riskLevel: "high",
      suggestedMarkdown: [
        "```mermaid",
        "flowchart LR",
        "  Source[Source data] --> Agent[Document agent]",
        "  Agent --> Review[Human review]",
        "  Review --> Export[Distribution package]",
        "```",
      ].join("\n"),
      owner: "Technical Agent",
    });
  }

  if (documentEvidence.claimInventory.length || plan.sourcePack.items.length || /\b(citation|source|reference|evidence|claim|bibliography|quote)\b/.test(corpus)) {
    const location = target(/\b(evidence|source|reference|claim|findings|analysis|appendix|citation)\b/i, "Review or evidence section");
    add({
      kind: "table",
      label: "Source-to-claim ledger",
      purpose: "Bind claims to sources so an agent can detect stale narrative when source data or citations change.",
      ...location,
      sourceSignal: "Claims, source-pack items, citation TODOs, references, or evidence language detected.",
      narrativeReviewTrigger: "Any source status change must reopen claims, executive summary, and export readiness.",
      evidenceRequired: ["Claim text", "Source owner or reference", "Verification status", "Reviewer decision"],
      riskLevel: documentEvidence.claimInventory.length ? "high" : "medium",
      suggestedMarkdown: [
        "| Claim | Source | Status | Narrative section to review |",
        "| --- | --- | --- | --- |",
        "| Material claim or number | Source title, file, URL, or owner | pending / verified / rejected | Section heading |",
      ].join("\n"),
      owner: "Evidence Agent",
    });
  }

  if (/\b(latex|equation|formula|scientific|research|paper|academic|method|molarity|dose|hypothesis)\b/.test(corpus)) {
    const location = target(/\b(method|equation|formula|analysis|research|technical|appendix)\b/i);
    add({
      kind: "equation",
      label: "Equation review block",
      purpose: "Stage equations and scientific assumptions where LaTeX export, citations, and calculation checks can be reviewed together.",
      ...location,
      templateId: /\b(dose|weight)\b/.test(corpus) ? "calc-science-dose" : "calc-science-molarity",
      sourceSignal: "LaTeX, equation, formula, scientific, research, method, dose, or hypothesis language detected.",
      narrativeReviewTrigger: "When a formula, unit, or parameter changes, review the method, findings, and limitations prose.",
      evidenceRequired: ["Units and parameter definitions", "Source for formula or method", "LaTeX/PDF export proof"],
      riskLevel: "high",
      suggestedMarkdown: [
        "```calc",
        "mass_g = 5.84",
        "moles = 0.1",
        "molar_mass = mass_g / moles",
        "```",
        "Equation check: {{=molar_mass}} g/mol",
      ].join("\n"),
      owner: "Scientific Review Agent",
    });
  }

  if (distributionTargetPlans.some((targetPlan) => targetPlan.target === "blog" || targetPlan.target === "substack" || targetPlan.target === "html")) {
    add({
      kind: "publishing",
      label: "Publishing metadata table",
      purpose: "Prepare channel-specific metadata and reuse it across blog, Substack, and HTML distribution checks.",
      insertionTarget: "Distribution or publishing handoff",
      sourceSignal: "Blog, Substack, newsletter, or HTML distribution target detected.",
      narrativeReviewTrigger: "Changing title, excerpt, subject line, CTA, or tags should reopen the first paragraph and distribution preview.",
      evidenceRequired: ["Channel title", "Excerpt or preview text", "CTA", "Link and metadata preview"],
      riskLevel: "medium",
      suggestedMarkdown: [
        "| Channel | Title | Excerpt / preview text | Tags | CTA |",
        "| --- | --- | --- | --- | --- |",
        "| Blog | Working title | Search/social excerpt | tag-1, tag-2 | Primary action |",
        "| Substack | Subject line | Inbox preview | tag-1, tag-2 | Reader action |",
      ].join("\n"),
      owner: "Distribution Agent",
    });
  }

  if (!recommendations.length) {
    add({
      kind: "table",
      label: "Document evidence table",
      purpose: "Create a lightweight source and decision ledger before the agent drafts or revises unsupported prose.",
      insertionTarget: "Review or evidence section",
      sourceSignal: "No specific transform signal was detected, so the agent recommends a general evidence ledger.",
      narrativeReviewTrigger: "Any evidence status change should reopen related claims and reviewer handoff notes.",
      evidenceRequired: ["Claim or decision", "Source or owner", "Status", "Reviewer note"],
      riskLevel: "low",
      suggestedMarkdown: [
        "| Item | Source or owner | Status | Review note |",
        "| --- | --- | --- | --- |",
        "| Claim, decision, or assumption | Source title, file, URL, or owner | pending / verified | Human review note |",
      ].join("\n"),
      owner: "Evidence Agent",
    });
  }

  return recommendations.slice(0, 10);
}

function buildDataNarrativeLinks(input: {
  plan: AgenticWorkflowPlan;
  sectionWorkQueue: AgenticSectionWorkItem[];
  documentEvidence: AgenticDocumentEvidence;
  transformRecommendations: AgenticTransformRecommendation[];
}): AgenticDataNarrativeLink[] {
  const { plan, sectionWorkQueue, documentEvidence, transformRecommendations } = input;
  const links: AgenticDataNarrativeLink[] = [];
  const seen = new Set<string>();
  const add = (item: Omit<AgenticDataNarrativeLink, "id">) => {
    const id = `data-narrative-${stableFingerprint([item.sourceKind, item.sourceLabel, item.affectedSection, item.changeSignal].join("\n")).slice(0, 12)}`;
    if (seen.has(id)) return;
    seen.add(id);
    links.push({ id, ...item });
  };
  const targetFor = (candidate: { sectionId?: string; insertionTarget?: string; sourceLabel?: string; sourceText?: string }) => {
    const explicit = candidate.sectionId ? sectionWorkQueue.find((section) => section.id === candidate.sectionId) : null;
    if (explicit) return explicit;
    const targetText = [candidate.insertionTarget, candidate.sourceLabel, candidate.sourceText].filter(Boolean).join(" ");
    return (
      sectionWorkQueue.find((section) => {
        const sectionText = `${section.heading} ${section.contract.purpose} ${section.contract.desiredDecision}`;
        return sharesNarrativeSignal(targetText, sectionText);
      }) ||
      sectionWorkQueue.find((section) => /\b(summary|recommendation|decision|evidence|analysis|findings|risk|next steps?)\b/i.test(section.heading)) ||
      sectionWorkQueue[0]
    );
  };

  for (const recommendation of transformRecommendations) {
    const section = targetFor({
      sectionId: recommendation.sectionId,
      insertionTarget: recommendation.insertionTarget,
      sourceLabel: recommendation.label,
      sourceText: recommendation.purpose,
    });
    add({
      sourceKind: recommendation.kind,
      sourceLabel: recommendation.label,
      affectedSection: section ? `Section ${section.order}: ${section.heading}` : recommendation.insertionTarget,
      sectionId: section?.id,
      changeSignal: recommendation.sourceSignal,
      narrativeRisk: recommendation.narrativeReviewTrigger,
      reviewAction: `After inserting or changing this ${recommendation.kind} block, review ${section?.heading || recommendation.insertionTarget} and any executive summary or recommendation language that depends on it.`,
      evidenceRequired: [
        ...recommendation.evidenceRequired.slice(0, 3),
        "Narrative review note confirms dependent prose still matches the structured block.",
      ],
      status: recommendation.riskLevel === "high" ? "needs-review" : "watch",
      owner: recommendation.owner,
    });
  }

  for (const claim of documentEvidence.claimInventory.slice(0, 10)) {
    const section = targetFor({ sourceLabel: titleCase(claim.kind), sourceText: claim.text });
    add({
      sourceKind: claim.kind,
      sourceLabel: claim.text,
      affectedSection: section ? `Section ${section.order}: ${section.heading}` : "Current document",
      sectionId: section?.id,
      changeSignal: `${claim.reason} Line ${claim.sourceLine}.`,
      narrativeRisk: "If the claim source, value, date, owner, or confidence changes, the related summary, recommendation, risk, and export-readiness prose may become stale.",
      reviewAction: `Verify the claim source and reread ${section?.heading || "the affected narrative"} before accepting or distributing the document.`,
      evidenceRequired: ["Source or owner for the claim", "Verification status", "Narrative note confirming dependent prose was reviewed"],
      status: claim.kind === "commitment" || claim.kind === "number" || claim.kind === "date" ? "needs-review" : "watch",
      owner: "Evidence Agent",
    });
  }

  if (plan.sourcePack.items.length && links.length < 12) {
    const sourceItem = plan.sourcePack.items[0];
    const section = targetFor({ sourceLabel: sourceItem.label, sourceText: sourceItem.detail });
    add({
      sourceKind: "source-pack",
      sourceLabel: sourceItem.label,
      affectedSection: section ? `Section ${section.order}: ${section.heading}` : "Current document",
      sectionId: section?.id,
      changeSignal: `Source pack item ${sourceItem.kind}: ${sourceItem.detail}`,
      narrativeRisk: "Changing or rejecting a source-pack item can invalidate claims, examples, reviewer responses, or distribution metadata.",
      reviewAction: "Refresh the source pack and review dependent claims before provider handoff or export.",
      evidenceRequired: ["Updated source-pack item", "Reviewer confirmation", "Dependent narrative review note"],
      status: "watch",
      owner: "Evidence Agent",
    });
  }

  if (!links.length) {
    const section = sectionWorkQueue[0];
    add({
      sourceKind: "metadata",
      sourceLabel: "Document evidence state",
      affectedSection: section ? `Section ${section.order}: ${section.heading}` : plan.title,
      sectionId: section?.id,
      changeSignal: "No structured data, claim, or source signal was detected yet.",
      narrativeRisk: "If evidence is added later, dependent prose must be reviewed before export.",
      reviewAction: "Create at least one source, claim, table, calculation, or transform link before release if the document makes factual claims.",
      evidenceRequired: ["Source or claim inventory", "Narrative dependency note", "Final reviewer confirmation"],
      status: "watch",
      owner: "Governance Agent",
    });
  }

  return links.slice(0, 18);
}

function sharesNarrativeSignal(left: string, right: string) {
  const stopwords = new Set(["the", "and", "for", "with", "from", "this", "that", "section", "document", "agent"]);
  const words = (value: string) =>
    new Set(
      value
        .toLowerCase()
        .match(/\b[a-z0-9]{4,}\b/g)
        ?.filter((word) => !stopwords.has(word)) || [],
    );
  const leftWords = words(left);
  if (!leftWords.size) return false;
  for (const word of words(right)) {
    if (leftWords.has(word)) return true;
  }
  return false;
}

function buildSectionContract(
  heading: string,
  level: number,
  draftingDepth: DocsLiveDraftDepth,
  plan: AgenticWorkflowPlan,
  reviewerAgentIds: AgenticReviewerAgentId[],
): AgenticSectionContract {
  const headingText = heading.toLowerCase();
  const audience = extractKeyValue(plan.placeholderText, "audience") || extractIntentField(plan.documentIntent, "audience") || "the intended reader";
  const owner = extractKeyValue(plan.placeholderText, "owner") || extractIntentField(plan.documentIntent, "owner") || "the accountable owner";
  const outcome = extractKeyValue(plan.placeholderText, "outcome") || extractIntentField(plan.documentIntent, "outcome") || "a clear next decision";
  const evidence = extractKeyValue(plan.placeholderText, "evidence") || extractIntentField(plan.documentIntent, "evidence") || "verified source material";
  const purpose = sectionPurpose(heading, plan.documentType, outcome);
  const desiredDecision = sectionDesiredDecision(heading, plan.documentType, outcome);
  const evidenceExpectations = sectionEvidenceExpectations(heading, evidence, plan, reviewerAgentIds);
  const riskLevel = sectionRiskLevel(headingText, level, draftingDepth, plan, reviewerAgentIds);
  const doneCriteria = [
    "Reader purpose is explicit in the opening sentence.",
    "No unresolved placeholders remain unless they are intentionally marked for review.",
    "Tone matches the audience and avoids generic AI phrasing.",
    riskLevel === "high" ? "Risk, approval, or legal-sensitive language is reviewed before release." : "Reviewer handoff notes state any remaining dependency.",
  ];

  return {
    purpose,
    targetReader: audience,
    desiredDecision,
    evidenceExpectations,
    owner,
    riskLevel,
    doneCriteria,
  };
}

function extractIntentField(intent: AgenticDocumentIntentSheet, key: string) {
  const field = intent.fields.find((item) => item.key === key);
  return field && field.status !== "missing" ? field.value : "";
}

function sectionPurpose(heading: string, documentType: DocsLiveDocumentType, outcome: string) {
  if (/\b(summary|overview|abstract|executive)\b/i.test(heading)) {
    return `Orient the reader quickly and state what the ${titleCase(documentType)} is asking them to understand or approve.`;
  }
  if (/\b(context|background|current state|problem|need)\b/i.test(heading)) {
    return "Establish the situation, constraints, and why the document matters now.";
  }
  if (/\b(evidence|findings|analysis|financial|data|metrics?|forecast|research)\b/i.test(heading)) {
    return "Present the factual basis and make source-backed implications easy to inspect.";
  }
  if (/\b(recommendation|decision|approval|ask|next steps?|handoff)\b/i.test(heading)) {
    return `Convert the preceding evidence into ${outcome}.`;
  }
  if (/\b(risks?|assumptions?|constraints?|mitigation|legal|compliance)\b/i.test(heading)) {
    return "Make uncertainty, obligations, and mitigations visible before the document is approved or distributed.";
  }
  if (/\b(scope|deliverables?|timeline|implementation|steps?|procedure|workflow)\b/i.test(heading)) {
    return "Define what will happen, who is responsible, and how execution boundaries are controlled.";
  }
  return `Advance the ${titleCase(documentType)} toward ${outcome}.`;
}

function sectionDesiredDecision(heading: string, documentType: DocsLiveDocumentType, outcome: string) {
  if (/\b(summary|overview|abstract|executive)\b/i.test(heading)) return `Reader can identify the document's purpose and the requested outcome: ${outcome}.`;
  if (/\b(context|background|current state|problem|need)\b/i.test(heading)) return "Reader agrees the problem framing and constraints are accurate.";
  if (/\b(evidence|findings|analysis|financial|data|metrics?|forecast|research)\b/i.test(heading)) return "Reader trusts the source-backed evidence enough to continue to recommendations.";
  if (/\b(recommendation|decision|approval|ask)\b/i.test(heading)) return `Reader can accept, reject, or revise the requested decision: ${outcome}.`;
  if (/\b(risks?|assumptions?|constraints?|mitigation|legal|compliance)\b/i.test(heading)) return "Reviewer can confirm whether risks and obligations are acceptable.";
  if (/\b(next steps?|handoff|publish|distribution|export)\b/i.test(heading)) return "Owner knows the exact follow-up actions, channel, and approval dependency.";
  return `Reader has enough context to move the ${titleCase(documentType)} toward ${outcome}.`;
}

function sectionEvidenceExpectations(
  heading: string,
  evidence: string,
  plan: AgenticWorkflowPlan,
  reviewerAgentIds: AgenticReviewerAgentId[],
) {
  const expectations = [
    `Use ${evidence} for material claims or mark missing support with citation TODOs.`,
    plan.sourcePack.claims.length ? `Check against ${plan.sourcePack.claims.length} structured source-pack claim(s).` : "",
    plan.documentMemory.entries.length ? "Apply relevant document memory for terminology, accepted decisions, and rejected directions." : "",
    reviewerAgentIds.includes("citation") ? "Confirm citation, reference, table, equation, or source labels before release." : "",
    /\b(financial|forecast|budget|pricing|investment|roi|metric|data|evidence|findings|analysis)\b/i.test(heading)
      ? "State the source, date, and confidence for numbers, forecasts, metrics, or quoted facts."
      : "",
  ].filter(Boolean);
  return Array.from(new Set(expectations)).slice(0, 5);
}

function sectionRiskLevel(
  headingText: string,
  level: number,
  draftingDepth: DocsLiveDraftDepth,
  plan: AgenticWorkflowPlan,
  reviewerAgentIds: AgenticReviewerAgentId[],
): AgenticSectionContract["riskLevel"] {
  if (
    reviewerAgentIds.includes("risk") ||
    draftingDepth === "legal" ||
    /\b(approval|approved|legal|contract|compliance|privacy|security|obligations?|risks?|liability|warranty|financial|forecast|investment|budget)\b/.test(headingText)
  ) {
    return "high";
  }
  if (plan.distributionTargets.length || level >= 3 || reviewerAgentIds.includes("citation")) return "medium";
  return "low";
}

function sectionDraftingDepth(heading: string, level: number, plan: AgenticWorkflowPlan): DocsLiveDraftDepth {
  const headingText = heading.toLowerCase();
  const documentType = plan.documentType.toLowerCase();
  if (/\b(executive summary|summary|overview|abstract|decision needed|recommendation)\b/.test(headingText)) return "executive";
  if (/\b(legal|contract|terms|compliance|policy|obligation|risk|privacy|security)\b/.test(headingText) || /\b(policy|contract)\b/.test(documentType)) return "legal";
  if (/\b(technical|architecture|implementation|api|schema|equation|method|methodology|integration|data model)\b/.test(headingText) || /\b(technical|architecture|adr|research)\b/.test(documentType)) return "technical";
  if (/\b(context|background|current state|financial|investment|pricing|model|analysis|evidence|metrics|forecast|business case|roi)\b/.test(headingText)) return "detailed";
  if (level >= 3) return "detailed";
  return "standard";
}

function parseOutlineSections(outline: string) {
  const parsed = outline
    .split(/\r?\n/)
    .map((line) => {
      const heading = line
        .trim()
        .replace(/^#{1,6}\s+/, "")
        .replace(/^[-*+]\s+/, "")
        .replace(/^\d+[.)]\s+/, "")
        .trim();
      if (!heading) return null;
      const markdownLevel = line.trim().match(/^(#{1,6})\s+/)?.[1]?.length;
      const indentLevel = Math.floor((line.match(/^\s*/)?.[0].length || 0) / 2) + 1;
      return {
        heading,
        level: markdownLevel || indentLevel,
      };
    })
    .filter((section): section is { heading: string; level: number } => Boolean(section));
  return parsed.length ? parsed : [{ heading: "Document", level: 1 }];
}

function buildOutlineCritique(plan: AgenticWorkflowPlan): AgenticOutlineCritiqueItem[] {
  if (!plan.suggestedOutline.trim()) {
    return [
      {
        severity: "blocker",
        area: "coverage",
        heading: "(missing outline)",
        detail: "No outline is available for section-by-section drafting.",
        recommendation: "Create at least three outline sections before generating body copy.",
      },
    ];
  }
  const sections = parseOutlineSections(plan.suggestedOutline);
  const critique: AgenticOutlineCritiqueItem[] = [];
  const bodySections = sections.length > 1 ? sections.slice(1) : sections;
  if (bodySections.length < 3) {
    critique.push({
      severity: "blocker",
      area: "coverage",
      heading: sections[0]?.heading || "(outline)",
      detail: `Only ${bodySections.length} draftable outline section(s) were found.`,
      recommendation: "Add enough sections to cover context, evidence, recommendation, risk, and handoff before drafting.",
    });
  }

  const counts = new Map<string, number>();
  for (const section of sections) {
    const key = normalizeOutlineHeading(section.heading);
    counts.set(key, (counts.get(key) || 0) + 1);
  }
  for (const [heading, count] of counts) {
    if (count > 1) {
      critique.push({
        severity: "warning",
        area: "duplication",
        heading,
        detail: `${count} outline entries normalize to the same heading.`,
        recommendation: "Rename, merge, or remove duplicated outline headings before assigning section work.",
      });
    }
  }

  for (const section of bodySections) {
    if (section.level > 4) {
      critique.push({
        severity: "warning",
        area: "depth",
        heading: section.heading,
        detail: `Heading is nested at level ${section.level}, deeper than the first-class outline mode supports.`,
        recommendation: "Promote deeply nested material or convert it into bullets inside a parent section.",
      });
    }
    if (/^(introduction|overview|background|miscellaneous|other|notes|conclusion)$/i.test(section.heading.trim())) {
      critique.push({
        severity: "info",
        area: "specificity",
        heading: section.heading,
        detail: "Heading is generic and may not tell the drafting agent what decision, evidence, or action belongs there.",
        recommendation: "Rename the section around the reader outcome, evidence type, or decision it supports.",
      });
    }
  }

  for (const expected of expectedOutlineSignals(plan)) {
    if (!sections.some((section) => expected.pattern.test(section.heading))) {
      critique.push({
        severity: expected.required ? "warning" : "info",
        area: "coverage",
        heading: expected.label,
        detail: `The ${expected.label} section expected for ${titleCase(plan.documentType)} is not present.`,
        recommendation: expected.recommendation,
      });
    }
  }

  const firstActionIndex = sections.findIndex((section) => /\b(next steps?|requested approval|approval|handoff|publish|distribution)\b/i.test(section.heading));
  const firstContextIndex = sections.findIndex((section) => /\b(context|background|current state|problem|need|evidence|findings)\b/i.test(section.heading));
  if (firstActionIndex >= 0 && (firstContextIndex < 0 || firstActionIndex < firstContextIndex)) {
    critique.push({
      severity: "warning",
      area: "sequence",
      heading: sections[firstActionIndex]?.heading || "Action section",
      detail: "Action or approval section appears before context or evidence.",
      recommendation: "Move context, evidence, or findings before approval and next-step sections.",
    });
  }

  return critique.slice(0, 12);
}

function normalizeOutlineHeading(value: string) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
}

function expectedOutlineSignals(plan: AgenticWorkflowPlan) {
  const common = [
    {
      label: "Evidence or findings",
      pattern: /\b(evidence|findings|financial|data|metrics?|source|proof)\b/i,
      required: plan.lanes.includes("review"),
      recommendation: "Add an evidence, findings, data, or financial case section so claims have a source-review home.",
    },
    {
      label: "Risk or assumptions",
      pattern: /\b(risks?|assumptions?|constraints?|mitigation)\b/i,
      required: plan.lanes.includes("review") || plan.distributionTargets.length > 0,
      recommendation: "Add a risk, assumption, or constraint section before review or distribution.",
    },
  ];
  const byType: Partial<Record<DocsLiveDocumentType, typeof common>> = {
    "board-memo": [
      {
        label: "Decision needed",
        pattern: /\b(decision|ask|approval requested|requested approval)\b/i,
        required: true,
        recommendation: "Add a decision or requested approval section for directors.",
      },
      {
        label: "Financial case",
        pattern: /\b(financial|investment|budget|forecast|case)\b/i,
        required: true,
        recommendation: "Add a financial case section with the evidence directors need.",
      },
      {
        label: "Risks",
        pattern: /\b(risks?|mitigation|challenge)\b/i,
        required: true,
        recommendation: "Add a risk section before the approval request.",
      },
    ],
    proposal: [
      {
        label: "Client need",
        pattern: /\b(client need|need|problem|challenge)\b/i,
        required: true,
        recommendation: "Add a client need or problem section before the proposed approach.",
      },
      {
        label: "Scope",
        pattern: /\b(scope|deliverables?|out of scope)\b/i,
        required: true,
        recommendation: "Add a scope section so acceptance and delivery boundaries are explicit.",
      },
      {
        label: "Investment",
        pattern: /\b(investment|pricing|fees?|commercial|budget)\b/i,
        required: true,
        recommendation: "Add an investment or commercial section before acceptance.",
      },
    ],
    "research-brief": [
      {
        label: "Method",
        pattern: /\b(method|methodology|sources?|approach)\b/i,
        required: true,
        recommendation: "Add a method or source approach section before findings.",
      },
      {
        label: "References",
        pattern: /\b(references?|bibliography|citations?|sources?)\b/i,
        required: true,
        recommendation: "Add a references or source section for research handoff.",
      },
    ],
    "operating-procedure": [
      {
        label: "Steps",
        pattern: /\b(steps?|procedure|workflow|process)\b/i,
        required: true,
        recommendation: "Add a procedure or steps section that operators can follow.",
      },
      {
        label: "Controls",
        pattern: /\b(controls?|checks?|exceptions?|escalation)\b/i,
        required: true,
        recommendation: "Add controls, checks, or exceptions before release.",
      },
    ],
  };
  return [...(byType[plan.documentType] || []), ...common];
}

function sectionReviewerIds(heading: string, plan: AgenticWorkflowPlan): AgenticReviewerAgentId[] {
  const ids: AgenticReviewerAgentId[] = ["editor", "evidence", "governance"];
  if (/\b(risks?|assumptions?|constraints?|approval|decision|legal|compliance)\b/i.test(heading)) ids.push("risk");
  if (/\b(source|citation|reference|bibliography|evidence|data|metric|equation)\b/i.test(heading) || plan.distributionTargets.includes("latex")) {
    ids.push("citation");
  }
  if (plan.distributionTargets.length || /\b(distribution|publish|handoff|export|next steps)\b/i.test(heading)) ids.push("export");
  return Array.from(new Set(ids));
}

function buildReviewerAgents(input: {
  plan: AgenticWorkflowPlan;
  draftMarkdown: string;
  revision: AgenticWorkflowRevision | null;
  editAcceptanceQueue: AgenticEditAcceptanceItem[];
  controlCenter: AgenticControlCenter;
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  blockers: string[];
  documentEvidence: AgenticDocumentEvidence;
  outlineCritique: AgenticOutlineCritiqueItem[];
}): AgenticReviewerAgent[] {
  const { plan, draftMarkdown, revision, editAcceptanceQueue, controlCenter, distributionTargetPlans, blockers, documentEvidence, outlineCritique } = input;
  const evidenceValue = extractKeyValue(plan.placeholderText, "evidence");
  const hasSpecificEvidence = Boolean(evidenceValue && !/^TBD\b/i.test(evidenceValue));
  const hasDraft = Boolean(draftMarkdown.trim());
  const missingInputs = plan.missingInputs.join(", ");
  const hardBlockers = blockers.filter((blocker) => !blocker.startsWith("Missing input:"));
  const distributionLabels = distributionTargetPlans.map((target) => target.label).join(", ");

  return [
    reviewerAgent({
      id: "editor",
      label: "Editorial Reviewer",
      mandate: "Improve clarity, audience fit, structure, tone, and human readability before anyone treats the packet as final copy.",
      findings: [
        plan.suggestedOutline.trim() ? "Outline is available as an editorial structure for section-by-section review." : "No outline was available to validate narrative structure.",
        hasDraft ? "Generated draft exists and needs audience-fit and stale phrase cleanup." : revision ? "Revision proposal exists and needs before/after editorial comparison." : "No generated body copy is present yet.",
        documentEvidence.unresolvedPlaceholders.length
          ? `Current document has unresolved placeholders: ${documentEvidence.unresolvedPlaceholders.slice(0, 4).join(", ")}.`
          : "No obvious placeholder tokens were found in the current document.",
        outlineCritique.length
          ? `Outline critique found ${outlineCritique.length} structure, coverage, sequencing, duplication, or specificity item(s).`
          : "No outline critique items were detected.",
        revision?.meaningDriftFindings.length
          ? `Meaning-drift scan found ${revision.meaningDriftFindings.length} changed or missing number, date, commitment, or caveat item(s).`
          : revision
            ? "Meaning-drift scan did not flag changed numbers, dates, commitments, or caveats."
            : "",
        editAcceptanceQueue.length
          ? `Edit acceptance queue prepared ${editAcceptanceQueue.length} item(s) for item-by-item approval.`
          : "",
        `Document-type quality gates: ${plan.qualityGates.map((gate) => gate.label).join(", ")}.`,
        documentEvidence.humanizationFindings.length
          ? `Humanization scan found ${documentEvidence.humanizationFindings.length} generic, repetitive, vague, or overconfident phrasing item(s).`
          : "No obvious generic AI phrasing patterns were detected in the current document.",
        `Tone target: ${extractKeyValue(plan.placeholderText, "tone") || "professional and direct"}.`,
      ],
      requiredActions: [
        missingInputs.includes("audience") ? "Confirm the intended audience before approving voice, detail level, and calls to action." : "",
        outlineCritique.length ? "Resolve outline critique items before treating section drafting as locked." : "",
        revision ? "Compare the proposed revision to the source text for meaning drift and over-compression." : "",
        revision?.revisionPasses.length ? `Complete the ${revision.revisionPasses.map((pass) => pass.label).join(", ")} revision pass checklist before sign-off.` : "",
        editAcceptanceQueue.length ? "Resolve every edit acceptance queue item as accepted, rejected, or needing another revision before applying changes." : "",
        "Complete the document-type quality gates before marking editorial review finished.",
        revision?.meaningDriftFindings.length ? "Resolve meaning-drift findings before accepting the proposed revision." : "",
        documentEvidence.humanizationFindings.length ? "Rewrite current-document humanization findings before final reader review." : "",
        hasDraft ? "Run a humanization pass for generic AI phrasing, repetition, and claims that sound confident without support." : "",
        documentEvidence.unresolvedPlaceholders.length ? "Resolve or intentionally preserve current-document placeholders before final approval." : "",
      ],
    }),
    reviewerAgent({
      id: "evidence",
      label: "Evidence Reviewer",
      mandate: "Trace claims, numbers, dates, and decisions back to named source material before approval.",
      findings: [
        hasSpecificEvidence ? `Evidence expectation captured: ${evidenceValue}.` : "Evidence is still generic or missing.",
        plan.sourcePack.items.length
          ? `User source pack contributes ${plan.sourcePack.items.length} structured item(s): ${plan.sourcePack.items.slice(0, 4).map((item) => `${item.kind} ${item.label}`).join(", ")}.`
          : "No user-managed source pack items were supplied.",
        controlCenter.sourceGrounding.some((item) => item.label === "Current document" && item.status === "available")
          ? "Current document context is available for source comparison."
          : "Current document source text is not available for full grounding.",
        documentEvidence.citationTodos.length
          ? `Current document has citation TODOs: ${documentEvidence.citationTodos.slice(0, 4).join(", ")}.`
          : "No citation TODO markers were found in the current document.",
        documentEvidence.claimInventory.length
          ? `Claim inventory captured ${documentEvidence.claimInventory.length} number, date, quote, commitment, or factual claim candidate(s).`
          : "No obvious claim inventory candidates were detected in the current document.",
      ],
      requiredActions: [
        hasSpecificEvidence ? "Verify every material claim against the supplied evidence before final export." : "Supply source evidence, data references, or citation expectations before final approval.",
        plan.sourcePack.items.length ? "Validate source-pack items and preserve relevant source labels in provider handoff or review notes." : "",
        "Mark unsupported claims with citation TODOs instead of letting them ship as confident assertions.",
        documentEvidence.claimInventory.length ? "Review the current-document claim inventory and attach sources or deferrals to each material item." : "",
        documentEvidence.citationTodos.length ? "Resolve current-document citation TODOs or keep them as explicit blockers in the review record." : "",
      ],
    }),
    reviewerAgent({
      id: "risk",
      label: "Risk Reviewer",
      mandate: "Surface decision risk, operational assumptions, missing approvals, and blocker severity.",
      findings: [
        blockers.length ? `${blockers.length} blocker or missing-input item(s) remain.` : "No blocker items were generated for this packet.",
        documentEvidence.unresolvedComments
          ? `${documentEvidence.unresolvedComments} unresolved review comment(s) remain in the current document: ${documentEvidence.reviewCommentResolutions
              .slice(0, 3)
              .map((comment) => `line ${comment.line} ${comment.excerpt}`)
              .join("; ")}.`
          : "No unresolved review comments were detected in the current document.",
        revision?.meaningDriftFindings.length
          ? `${revision.meaningDriftFindings.filter((item) => item.severity === "blocker").length} blocker meaning-drift item(s) require approval.`
          : "No blocker meaning-drift items were detected in the revision proposal.",
        plan.lanes.includes("distribute") ? "Distribution is in scope, so approval and release risk must be checked explicitly." : "Distribution is not currently in scope.",
      ],
      requiredActions: [
        ...hardBlockers,
        blockers.length && !hardBlockers.length ? "Resolve all missing-input blockers before marking the packet approved." : "",
        revision?.meaningDriftFindings.length ? "Approve, restore, or explicitly document every meaning-drift finding before release handoff." : "",
        documentEvidence.reviewCommentResolutions.length ? "Work through the review comment resolution queue with resolution notes before release handoff." : "",
        plan.lanes.includes("distribute") ? "Confirm approved or published status, approver or reviewer, approvedAt, owner, and releaseTarget before any external handoff." : "",
      ],
    }),
    reviewerAgent({
      id: "citation",
      label: "Citation Reviewer",
      mandate: "Ensure citation expectations, bibliography notes, links, and source markers survive drafting and export.",
      findings: [
        plan.instruction.match(/\bcitations?|references?|sources?\b/i) ? "The instruction explicitly asks for source or citation review." : "Citation handling is inferred from evidence and review requirements.",
        documentEvidence.brokenLinkHints.length
          ? `Current document has link placeholders or suspicious links: ${documentEvidence.brokenLinkHints.slice(0, 4).join(", ")}.`
          : "No obvious placeholder links were found in the current document.",
        documentEvidence.referenceHints.length
          ? `Current document has reference integrity findings: ${documentEvidence.referenceHints.slice(0, 4).join("; ")}.`
          : "No obvious malformed or unmatched cross references were found in the current document.",
        plan.distributionTargets.includes("latex") ? "LaTeX export requires bibliography, labels, equations, and cross-reference checks." : "No LaTeX-specific citation target is active.",
      ],
      requiredActions: [
        "Add citation TODOs beside factual claims that do not have a named source.",
        documentEvidence.brokenLinkHints.length ? "Repair placeholder or suspicious links before publishing." : "",
        documentEvidence.referenceHints.length ? "Repair missing labels, malformed label syntax, or unmatched cross references before export handoff." : "",
        plan.distributionTargets.includes("html") || plan.distributionTargets.includes("blog") || plan.distributionTargets.includes("substack")
          ? "Check external links, canonical URL expectations, and visible source notes for web publishing."
          : "",
        plan.distributionTargets.includes("latex") ? "Confirm bibliography entries and citation keys compile in the exported TeX source." : "",
      ],
    }),
    reviewerAgent({
      id: "governance",
      label: "Governance Reviewer",
      mandate: "Keep AI provenance, audit fingerprints, rollback instructions, and human review status visible.",
      findings: [
        "AI source metadata, control-center status, and audit fingerprints are included in the run packet.",
        documentEvidence.unreviewedAiMarkers
          ? `${documentEvidence.unreviewedAiMarkers} current-document AI provenance marker(s) still need human review.`
          : "No unreviewed current-document AI provenance markers were detected.",
        `Apply mode is ${inferApplicationMode(plan, Boolean(controlCenter.sourceGrounding.find((item) => item.label === "Current document" && item.status === "available")), Boolean(controlCenter.sourceGrounding.find((item) => item.label === "Selected text" && item.status === "available")))}.`,
      ],
      requiredActions: [
        "Do not remove AI provenance until a human reviewer has accepted the generated section or packet.",
        documentEvidence.unreviewedAiMarkers ? "Mark current-document AI source and AI-assisted markers human-reviewed only after inspection." : "",
        "Retain run ID, source fingerprint, and output fingerprint in review notes when applying agent output.",
      ],
    }),
    reviewerAgent({
      id: "export",
      label: "Export Reviewer",
      mandate: "Validate channel-specific packaging, manifests, previews, and distribution evidence before release.",
      findings: [
        distributionTargetPlans.length ? `Target runbooks staged for ${distributionLabels}.` : "No export or publishing target is selected yet.",
        documentEvidence.approvalMetadataMissing.length
          ? `Current document is missing distribution approval metadata: ${documentEvidence.approvalMetadataMissing.join(", ")}.`
          : "Required approval metadata is present or no distribution target is active.",
        distributionTargetPlans.length
          ? "Each target has preflight checks, handoff steps, and evidence requirements."
          : "Export reviewer cannot complete target checks until a target is chosen.",
      ],
      requiredActions: [
        distributionTargetPlans.length ? "Run export readiness for every target and keep manifest evidence with the review record." : "Choose at least one target before claiming distribution readiness.",
        documentEvidence.approvalMetadataMissing.length
          ? `Add missing approval metadata before distribution: ${documentEvidence.approvalMetadataMissing.join(", ")}.`
          : "",
        ...distributionTargetPlans.map((target) => `${target.label}: ${target.evidenceRequired[0]}`),
      ],
    }),
  ];
}

function reviewerAgent(input: Omit<AgenticReviewerAgent, "status">): AgenticReviewerAgent {
  const requiredActions = input.requiredActions.filter(Boolean);
  const findings = input.findings.filter(Boolean);
  const status: AgenticReviewerAgentStatus =
    requiredActions.some((item) => /\b(blocked|no document|no export|supply|resolve all|choose at least one)\b/i.test(item)) ||
    findings.some((item) => /\bnot available|missing\b/i.test(item))
      ? "blocked"
      : requiredActions.length
        ? "needs-review"
        : "ready";
  return {
    ...input,
    status,
    findings,
    requiredActions,
  };
}

function buildPreReviewRehearsal(input: {
  plan: AgenticWorkflowPlan;
  reviewerAgents: AgenticReviewerAgent[];
  sectionWorkQueue: AgenticSectionWorkItem[];
  documentEvidence: AgenticDocumentEvidence;
  revision: AgenticWorkflowRevision | null;
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  blockers: string[];
}): AgenticPreReviewRehearsalItem[] {
  const { plan, reviewerAgents, sectionWorkQueue, documentEvidence, revision, distributionTargetPlans, blockers } = input;
  const items: AgenticPreReviewRehearsalItem[] = [];
  const reviewerActive = (id: AgenticReviewerAgentId) => reviewerAgents.some((agent) => agent.id === id);
  const add = (
    kind: AgenticPreReviewRehearsalKind,
    reviewer: AgenticReviewerAgentId,
    prompt: string,
    whyItMatters: string,
    suggestedResponse: string,
    releaseBlocker: boolean,
    relatedSectionId?: string,
  ) => {
    if (!reviewerActive(reviewer)) return;
    items.push({
      id: `pre-review-${kind}-${reviewer}-${stableFingerprint([kind, reviewer, prompt, relatedSectionId || ""].join("\n")).slice(0, 10)}`,
      kind,
      reviewer,
      prompt,
      whyItMatters,
      suggestedResponse,
      relatedSectionId,
      releaseBlocker,
    });
  };

  const decisionSection = sectionWorkQueue.find((section) => /\b(decision|approval|ask|recommendation|next steps?)\b/i.test(section.heading));
  const evidenceSection = sectionWorkQueue.find((section) => /\b(evidence|findings|analysis|financial|data|metrics?|source|proof)\b/i.test(section.heading));
  const riskSection = sectionWorkQueue.find((section) => /\b(risks?|assumptions?|constraints?|mitigation|legal|compliance)\b/i.test(section.heading));
  const audience = extractKeyValue(plan.placeholderText, "audience") || extractIntentField(plan.documentIntent, "audience") || "the intended reader";
  const evidence = extractKeyValue(plan.placeholderText, "evidence") || extractIntentField(plan.documentIntent, "evidence") || "named source evidence";

  add(
    "question",
    "editor",
    `Will ${audience} understand the requested decision after the first page or opening section?`,
    "Reviewers often reject otherwise useful drafts when the ask is buried behind process or background.",
    "Move the ask, owner, and decision language into the opening structure or add an executive summary bullet.",
    false,
    decisionSection?.id,
  );
  add(
    "objection",
    "risk",
    "What assumption would make this recommendation unsafe, late, or too expensive?",
    "Risk reviewers need to see the failure mode before approving action or distribution.",
    riskSection ? "Strengthen the risk section with mitigation, owner, and decision impact." : "Add a risk or assumptions section before formal review.",
    true,
    riskSection?.id,
  );
  add(
    "missing-evidence",
    "evidence",
    `Which material claim still depends on ${evidence} and lacks a named source, date, or confidence level?`,
    "Claims without source labels become release blockers in client, board, research, and publishing workflows.",
    evidenceSection ? "Attach source labels in the evidence section and add citation TODOs beside unsupported claims." : "Create an evidence section or source ledger before drafting final claims.",
    !plan.sourcePack.claims.length && documentEvidence.claimInventory.length > 0,
    evidenceSection?.id,
  );
  add(
    "redline",
    "governance",
    "Which sentence would legal, compliance, or an approver redline because it sounds like an unsupported promise?",
    "Overconfident or obligation-shaped language should be caught before external handoff.",
    "Rewrite commitments with caveats, conditions, owner, and approval status, then keep the redline note in the review record.",
    documentEvidence.claimInventory.some((claim) => claim.kind === "commitment") || blockers.length > 0,
    riskSection?.id,
  );

  if (revision) {
    add(
      "question",
      "editor",
      "What did the revision compress or remove that a reviewer may consider material?",
      "Revision passes can accidentally remove caveats, obligations, dates, numbers, or context.",
      "Compare original and proposed text, then record accepted meaning changes or restore the caveat.",
      Boolean(revision.meaningDriftFindings.length),
    );
  }

  if (documentEvidence.reviewCommentResolutions.length) {
    add(
      "objection",
      "risk",
      `Which unresolved comment is most likely to block approval: ${documentEvidence.reviewCommentResolutions[0]?.excerpt || "unresolved review comment"}?`,
      "Open comments are reviewer objections already present in the document.",
      "Resolve the comment, record a resolution note, or explicitly carry it forward as a release blocker.",
      true,
    );
  }

  if (documentEvidence.humanizationFindings.length) {
    add(
      "redline",
      "editor",
      `Which phrase should be rewritten because it sounds generic or AI-authored: "${documentEvidence.humanizationFindings[0]?.text || "generic phrasing"}"?`,
      "Human reviewers often flag generic AI phrasing even when facts are correct.",
      "Rewrite the phrase in the user's voice, tie it to a concrete source or implication, and rerun review.",
      false,
    );
  }

  if (distributionTargetPlans.length) {
    add(
      "missing-evidence",
      "export",
      `What proof will show the ${distributionTargetPlans.map((target) => target.label).join(", ")} artifact was actually inspected?`,
      "Distribution is not complete until target-specific preview, import, or package evidence exists.",
      `Attach manifest, preview, readback, or reviewer evidence for ${distributionTargetPlans.map((target) => target.label).join(", ")} before release.`,
      true,
    );
  }

  add(
    "question",
    "citation",
    "Which citation, reference, link, table, equation, or label could fail when exported?",
    "Cross-target exports can expose weak source labels that are invisible in the drafting pane.",
    "Run citation/link/reference checks, repair labels, and keep export readiness evidence with the run.",
    Boolean(documentEvidence.citationTodos.length || documentEvidence.referenceHints.length || documentEvidence.brokenLinkHints.length),
  );

  return items.slice(0, 12);
}

function buildAutomationQueue(input: {
  plan: AgenticWorkflowPlan;
  documentEvidence: AgenticDocumentEvidence;
  outlineCritique: AgenticOutlineCritiqueItem[];
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  controlCenter: AgenticControlCenter;
  blockers: string[];
  transformRecommendations: AgenticTransformRecommendation[];
  approvalGate: AgenticApprovalGate;
}): AgenticAutomationTask[] {
  const { plan, documentEvidence, outlineCritique, distributionTargetPlans, controlCenter, blockers, transformRecommendations, approvalGate } = input;
  const evidenceBlockerCount =
    documentEvidence.unresolvedPlaceholders.length +
    documentEvidence.citationTodos.length +
    documentEvidence.claimInventory.length +
    documentEvidence.humanizationFindings.length +
    documentEvidence.reviewCommentResolutions.length +
    documentEvidence.brokenLinkHints.length +
    documentEvidence.referenceHints.length +
    documentEvidence.approvalMetadataMissing.length +
    approvalGate.blockers.length;
  const transformHints = [
    plan.sourcePack.items.some((item) => /\b(table|chart|calc|diagram|timeline|roadmap|schema|openapi|json|csv|qr)\b/i.test(`${item.label} ${item.detail}`))
      ? "Source pack contains structured-data or transform cues."
      : "",
    /\b(calc|chart|diagram|timeline|roadmap|table|schema|openapi|latex|equation)\b/i.test(plan.instruction)
      ? "Instruction mentions transform, table, diagram, schema, equation, or data work."
      : "",
    transformRecommendations.length
      ? `Agent selected ${transformRecommendations.length} transform recommendation(s): ${transformRecommendations.map((item) => item.label).join("; ")}.`
      : "",
  ].filter(Boolean);
  const tasks: AgenticAutomationTask[] = [
    automationTask({
      kind: "evidence-scan",
      label: "Refresh current-document evidence scan",
      owner: "Evidence Agent",
      status: evidenceBlockerCount ? "needs-input" : "ready",
      trigger: "Run after each draft, provider response, paste cleanup, or accepted revision.",
      action: "open-review",
      evidence: [
        `${documentEvidence.unresolvedPlaceholders.length} placeholders`,
        `${documentEvidence.citationTodos.length} citation TODOs`,
        `${documentEvidence.claimInventory.length} claims`,
        `${documentEvidence.reviewCommentResolutions.length} review comments`,
      ],
      nextStep: "Open Review, refresh evidence findings, and keep unresolved source gaps as release blockers.",
    }),
    automationTask({
      kind: "outline-critique",
      label: "Refresh outline critique",
      owner: "Composition Agent",
      status: outlineCritique.some((item) => item.severity === "blocker") ? "blocked" : outlineCritique.length ? "needs-input" : "ready",
      trigger: "Run whenever the outline, section order, document type, or audience changes.",
      action: "open-outline",
      evidence: outlineCritique.length ? outlineCritique.slice(0, 6).map((item) => `${titleCase(item.area)} ${item.severity}: ${item.detail}`) : ["No outline critique blockers detected."],
      nextStep: "Open Outline mode, resolve structure issues, then regenerate section contracts and drafting tasks.",
    }),
    automationTask({
      kind: "transform-validation",
      label: "Validate recommended transforms and templates",
      owner: "Transform Agent",
      status: transformRecommendations.some((item) => item.riskLevel === "high") || transformHints.length ? "needs-input" : "ready",
      trigger: "Run before inserting calc, table, chart, diagram, timeline, roadmap, schema, QR, or equation blocks.",
      action: "open-review",
      evidence: transformHints.length ? transformHints : ["No transform-specific source cues detected in the current run."],
      nextStep: "Validate selected templates, source data, and transform engine readiness before generated narrative depends on structured outputs.",
    }),
    automationTask({
      kind: "export-preflight",
      label: "Run target-aware export preflight",
      owner: "Distribution Agent",
      status: approvalGate.status === "blocked" ? "blocked" : distributionTargetPlans.length ? "needs-input" : "ready",
      trigger: "Run when distribution targets, release metadata, assets, links, or export options change.",
      action: "prepare-export",
      evidence: distributionTargetPlans.length
        ? [
            `Approval gate: ${approvalGate.status}`,
            ...approvalGate.blockers.slice(0, 5),
            ...distributionTargetPlans.map((target) => `${target.label}: ${target.preflightChecks[0]}`),
          ]
        : ["No export or publishing target requested yet."],
      nextStep:
        approvalGate.status === "blocked"
          ? "Clear the approval metadata gate before generating export or publishing evidence."
          : "Open export readiness, confirm metadata and target-specific blockers, then attach manifest or package evidence.",
    }),
    automationTask({
      kind: "accessibility-check",
      label: "Queue accessibility and readability check",
      owner: "Accessibility Agent",
      status: plan.distributionTargets.length || plan.lanes.includes("distribute") ? "needs-input" : "ready",
      trigger: "Run before external review, publishing, PDF/DOCX/PPTX export, or executive handoff.",
      action: "open-review",
      evidence: [
        "Check headings, link text, table readability, alt text, color-independent meaning, and review notes before publishing.",
        plan.distributionTargets.length ? `Targets: ${plan.distributionTargets.join(", ")}` : "No external target requested yet.",
      ],
      nextStep: "Review accessibility risks and keep manual assistive-technology sign-off as release evidence when required.",
    }),
    automationTask({
      kind: "readiness-refresh",
      label: "Refresh AI control-center readiness",
      owner: "Governance Agent",
      status: controlCenter.status === "blocked" || blockers.length ? "blocked" : controlCenter.status === "needs-input" ? "needs-input" : "ready",
      trigger: "Run after any task status, evidence, approval metadata, provider output, or export state changes.",
      action: "open-review",
      evidence: [
        `Readiness score ${controlCenter.readinessScore}/100`,
        `Control status ${controlCenter.status}`,
        `Approval gate ${approvalGate.status}`,
        blockers.length ? `${blockers.length} blocker(s): ${blockers.slice(0, 3).join("; ")}` : "No run-level blockers detected.",
      ],
      nextStep: "Refresh readiness and confirm whether the next safe action is drafting, review, provider handoff, export preflight, or human approval.",
    }),
  ];
  return tasks;
}

function automationTask(input: Omit<AgenticAutomationTask, "id" | "safeToAutoRun" | "manualOnlyReason"> & { safeToAutoRun?: boolean; manualOnlyReason?: string }): AgenticAutomationTask {
  const safeToAutoRun = input.safeToAutoRun ?? true;
  return {
    id: `automation-${input.kind}-${stableFingerprint([input.kind, input.label, input.trigger].join("\n")).slice(0, 10)}`,
    safeToAutoRun,
    manualOnlyReason: safeToAutoRun ? undefined : input.manualOnlyReason || "This task may change files, publish content, or call an external service.",
    ...input,
  };
}

function buildControlCenter(input: {
  plan: AgenticWorkflowPlan;
  blockers: string[];
  hasDocument: boolean;
  hasSelection: boolean;
  revision: AgenticWorkflowRevision | null;
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  documentEvidence: AgenticDocumentEvidence;
  outlineCritique: AgenticOutlineCritiqueItem[];
  approvalGate: AgenticApprovalGate;
}): AgenticControlCenter {
  const { plan, blockers, hasDocument, hasSelection, revision, distributionTargetPlans, documentEvidence, outlineCritique, approvalGate } = input;
  const hardBlockers = blockers.filter((blocker) => !blocker.startsWith("Missing input:"));
  const status: AgenticControlStatus =
    hardBlockers.length || approvalGate.status === "blocked" ? "blocked" : blockers.length || approvalGate.status === "needs-review" ? "needs-input" : "ready";
  const sourceGrounding = buildSourceGrounding(plan, hasDocument, hasSelection, documentEvidence, outlineCritique);
  const governance = buildGovernanceItems(plan, revision, blockers, documentEvidence, approvalGate);
  const distribution = buildDistributionItems(plan, distributionTargetPlans, documentEvidence, approvalGate);
  const readinessScore = scoreControlCenter(status, sourceGrounding, governance, distribution, blockers);
  const nextActions = buildNextActions(plan, status, blockers, distributionTargetPlans, documentEvidence, approvalGate);
  const summary =
    status === "blocked"
      ? approvalGate.status === "blocked"
        ? "Agent run is blocked until the approval metadata gate is cleared for distribution."
        : "Agent run is blocked until source context or target instructions are supplied."
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

function buildAuditTrail(input: {
  plan: AgenticWorkflowPlan;
  request: AgenticWorkflowRunRequest;
  revision: AgenticWorkflowRevision | null;
  editAcceptanceQueue: AgenticEditAcceptanceItem[];
  draftMarkdown: string;
  lifecycleTasks: AgenticLifecycleTask[];
  reviewerAgents: AgenticReviewerAgent[];
  sectionWorkQueue: AgenticSectionWorkItem[];
  sectionDraftHistory: AgenticSectionDraftHistoryItem[];
  transformRecommendations: AgenticTransformRecommendation[];
  dataNarrativeLinks: AgenticDataNarrativeLink[];
  approvalGate: AgenticApprovalGate;
  automationQueue: AgenticAutomationTask[];
  preReviewRehearsal: AgenticPreReviewRehearsalItem[];
  documentEvidence: AgenticDocumentEvidence;
  outlineCritique: AgenticOutlineCritiqueItem[];
  reviewChecklist: string[];
  distributionChecklist: string[];
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  blockers: string[];
  applicationMode: AgenticWorkflowRun["applicationMode"];
  generatedAt: string;
}): AgenticAuditTrail {
  const {
    plan,
    request,
    revision,
    editAcceptanceQueue,
    draftMarkdown,
    lifecycleTasks,
    reviewerAgents,
    sectionWorkQueue,
    sectionDraftHistory,
    transformRecommendations,
    dataNarrativeLinks,
    approvalGate,
    automationQueue,
    preReviewRehearsal,
    documentEvidence,
    outlineCritique,
    reviewChecklist,
    distributionChecklist,
    distributionTargetPlans,
    blockers,
    applicationMode,
    generatedAt,
  } = input;
  const contextPayload = [
    plan.documentIntent.summary,
    plan.documentIntent.markdown,
    plan.documentMemory.summary,
    plan.documentMemory.markdown,
    plan.context,
    plan.sourcePackText,
    plan.memoryText,
    plan.sourcePack.markdown,
    plan.placeholderText,
    plan.suggestedOutline,
    ...plan.outlineVariants.flatMap((variant) => [variant.label, variant.strategy, variant.summary, variant.outline, ...variant.tradeoffs, ...variant.risks]),
    plan.revisionInstruction,
    plan.revisionModes.join(", "),
  ].join("\n---\n");
  const sourcePayload = [request.documentTitle || "", request.documentText || "", request.selectedText || ""].join("\n---\n");
  const outputPayload = [
    draftMarkdown,
    revision?.proposedText || "",
    ...lifecycleTasks.flatMap((task) => [
      task.id,
      task.lane,
      task.title,
      task.owner,
      task.status,
      task.sectionId || "",
      task.target || "",
      task.nextStep,
      ...task.evidence,
    ]),
    ...reviewerAgents.flatMap((agent) => [agent.id, agent.label, agent.mandate, agent.status, ...agent.findings, ...agent.requiredActions]),
    ...sectionDraftHistory.flatMap((item) => [
      item.id,
      item.sectionId,
      item.sectionHeading,
      item.versionLabel,
      item.promptSummary,
      item.rationale,
      item.sectionFingerprint,
      item.sourceFingerprint,
      item.acceptanceStatus,
      ...item.reviewerNotes,
    ]),
    ...transformRecommendations.flatMap((item) => [
      item.id,
      item.kind,
      item.label,
      item.purpose,
      item.insertionTarget,
      item.sectionId || "",
      item.templateId || "",
      item.sourceSignal,
      item.narrativeReviewTrigger,
      item.riskLevel,
      item.owner,
      item.suggestedMarkdown,
      ...item.evidenceRequired,
    ]),
    ...dataNarrativeLinks.flatMap((item) => [
      item.id,
      item.sourceKind,
      item.sourceLabel,
      item.affectedSection,
      item.sectionId || "",
      item.changeSignal,
      item.narrativeRisk,
      item.reviewAction,
      item.status,
      item.owner,
      ...item.evidenceRequired,
    ]),
    approvalGate.status,
    approvalGate.summary,
    String(approvalGate.requiredBeforeDistribution),
    ...approvalGate.fields.flatMap((field) => [field.key, field.label, field.value, field.status, field.guidance]),
    ...approvalGate.blockers,
    approvalGate.metadataScaffold,
    ...automationQueue.flatMap((item) => [
      item.id,
      item.kind,
      item.label,
      item.owner,
      item.status,
      String(item.safeToAutoRun),
      item.trigger,
      item.nextStep,
      item.manualOnlyReason || "",
      ...item.evidence,
    ]),
    ...preReviewRehearsal.flatMap((item) => [
      item.id,
      item.kind,
      item.reviewer,
      item.prompt,
      item.whyItMatters,
      item.suggestedResponse,
      item.relatedSectionId || "",
      String(item.releaseBlocker),
    ]),
    ...(revision?.revisionPasses.flatMap((pass) => [pass.mode, pass.label, pass.rationale, ...pass.checklist]) || []),
    ...editAcceptanceQueue.flatMap((item) => [
      item.id,
      item.scope,
      item.heading,
      ...item.changeSummary,
      ...item.riskNotes,
      item.recommendation,
    ]),
    plan.documentIntent.summary,
    ...plan.documentIntent.fields.flatMap((field) => [field.key, field.value, field.status, field.source, field.guidance]),
    ...plan.documentIntent.reviewPrompts,
    plan.documentMemory.summary,
    ...plan.documentMemory.entries.flatMap((entry) => [entry.kind, entry.label, entry.detail, entry.source]),
    ...plan.outlineVariants.flatMap((variant) => [
      variant.id,
      variant.label,
      variant.strategy,
      variant.summary,
      variant.outline,
      ...variant.bestFor,
      ...variant.tradeoffs,
      ...variant.risks,
    ]),
    ...plan.sourcePack.items.flatMap((item) => [item.kind, item.label, item.detail]),
    ...sectionWorkQueue.flatMap((section) => [
      section.id,
      section.heading,
      section.contract.purpose,
      section.contract.targetReader,
      section.contract.desiredDecision,
      section.contract.owner,
      section.contract.riskLevel,
      ...section.contract.evidenceExpectations,
      ...section.contract.doneCriteria,
      section.draftingInstruction,
      ...section.completionCriteria,
      ...section.reviewerAgentIds,
    ]),
    ...outlineCritique.flatMap((item) => [item.severity, item.area, item.heading, item.detail, item.recommendation]),
    ...documentEvidence.humanizationFindings.flatMap((item) => [item.kind, String(item.sourceLine), item.text, item.recommendation]),
    ...documentEvidence.reviewCommentResolutions.flatMap((item) => [
      item.id,
      String(item.line),
      item.author,
      item.createdAt,
      item.excerpt,
      item.requiredAction,
      String(item.blocker),
      ...item.resolutionOptions,
    ]),
    ...reviewChecklist,
    ...distributionChecklist,
    ...distributionTargetPlans.flatMap((target) => [target.target, target.label, ...target.preflightChecks, ...target.handoffSteps, ...target.evidenceRequired]),
    ...blockers,
  ].join("\n");
  return {
    runId: `agent-${compactTimestamp(generatedAt)}-${stableFingerprint([plan.title, plan.instruction, contextPayload].join("\n")).slice(0, 10)}`,
    generatedAt,
    plannerVersion: agentPlannerVersion,
    instructionFingerprint: stableFingerprint(plan.instruction || "(empty instruction)"),
    contextFingerprint: stableFingerprint(contextPayload),
    sourceFingerprint: stableFingerprint(sourcePayload),
    outputFingerprint: stableFingerprint(outputPayload),
    applicationMode,
    rollbackPlan: rollbackPlan(applicationMode),
    reviewEvents: [
      "Agent plan generated from current instruction, document context, and selection state.",
      `Document intent sheet prepared at ${plan.documentIntent.completenessScore}/100 with ${plan.documentIntent.missingFields.length} missing field(s).`,
      "AI provenance metadata attached to generated packet.",
      blockers.length ? `Human review required before release because ${blockers.length} blocker item(s) remain.` : "No blocker items detected at packet generation time.",
      `Reviewer agents prepared for ${reviewerAgents.map((agent) => agent.label).join(", ")}.`,
      `Lifecycle task board prepared for ${lifecycleTasks.length} task(s) across ${Array.from(new Set(lifecycleTasks.map((task) => task.lane))).map(titleCase).join(", ")}.`,
      `Section draft history preserved ${sectionDraftHistory.length} composable draft restore point(s).`,
      `Transform recommendations prepared ${transformRecommendations.length} agent-selected structured block(s) for calc, chart, table, diagram, timeline, schema, equation, or publishing work.`,
      `Data-to-narrative bridge prepared ${dataNarrativeLinks.length} dependency link(s) between claims, structured blocks, source signals, and affected narrative sections.`,
      `Approval gate prepared with ${approvalGate.status} status and ${approvalGate.blockers.length} blocker(s) before distribution.`,
      `Automation scheduler queued ${automationQueue.length} safe local check(s) with destructive actions kept manual.`,
      `Pre-review rehearsal prepared ${preReviewRehearsal.length} likely reviewer question, objection, redline, or missing-evidence prompt(s).`,
      `Outline variant comparison prepared ${plan.outlineVariants.length} alternative structure(s) for user selection before drafting.`,
      `Section work queue prepared for ${sectionWorkQueue.length} outline item(s).`,
      `Section contract cards prepared for ${sectionWorkQueue.length} outline item(s) with purpose, reader, evidence, owner, risk, and done criteria.`,
      outlineCritique.length
        ? `Outline critique prepared ${outlineCritique.length} structure and coverage item(s).`
        : "Outline critique found no structure or coverage issues.",
      documentEvidence.humanizationFindings.length
        ? `Humanization scan prepared ${documentEvidence.humanizationFindings.length} phrasing item(s) for editorial cleanup.`
        : "Humanization scan found no generic phrasing items.",
      revision?.revisionPasses.length
        ? `Revision pass plan prepared for ${revision.revisionPasses.map((pass) => pass.label).join(", ")}.`
        : "No explicit revision pass plan was needed for this run.",
      editAcceptanceQueue.length
        ? `Edit acceptance queue prepared ${editAcceptanceQueue.length} item(s) for item-by-item approval.`
        : "No edit acceptance queue was needed for this run.",
      plan.sourcePack.items.length
        ? `User source pack included ${plan.sourcePack.items.length} structured item(s) for provider and reviewer grounding.`
        : "No user-managed source pack items were supplied.",
      plan.documentMemory.entries.length
        ? `Reusable document memory included ${plan.documentMemory.entries.length} terminology, style, decision, review, or distribution item(s).`
        : "No reusable document memory items were supplied.",
      `Document-type quality gates staged: ${plan.qualityGates.map((gate) => gate.label).join(", ")}.`,
      documentEvidence.reviewCommentResolutions.length
        ? `Review comment resolution queue staged for ${documentEvidence.reviewCommentResolutions.length} unresolved comment(s).`
        : "No review comment resolution queue was needed.",
      distributionTargetPlans.length
        ? `Distribution evidence requirements staged for ${distributionTargetPlans.map((target) => target.label).join(", ")}.`
        : "No distribution target selected at packet generation time.",
    ],
  };
}

function buildReleaseEvidenceBundle(input: {
  plan: AgenticWorkflowPlan;
  auditTrail: AgenticAuditTrail;
  controlCenter: AgenticControlCenter;
  lifecycleTasks: AgenticLifecycleTask[];
  reviewerAgents: AgenticReviewerAgent[];
  sectionWorkQueue: AgenticSectionWorkItem[];
  sectionDraftHistory: AgenticSectionDraftHistoryItem[];
  transformRecommendations: AgenticTransformRecommendation[];
  dataNarrativeLinks: AgenticDataNarrativeLink[];
  approvalGate: AgenticApprovalGate;
  automationQueue: AgenticAutomationTask[];
  preReviewRehearsal: AgenticPreReviewRehearsalItem[];
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  documentEvidence: AgenticDocumentEvidence;
  blockers: string[];
}): AgenticReleaseEvidenceBundle {
  const { plan, auditTrail, controlCenter, lifecycleTasks, reviewerAgents, sectionWorkQueue, sectionDraftHistory, transformRecommendations, dataNarrativeLinks, approvalGate, automationQueue, preReviewRehearsal, distributionTargetPlans, documentEvidence, blockers } = input;
  const taskBlockers = lifecycleTasks.filter((task) => task.status === "blocked" || task.status === "needs-input");
  const reviewerBlockers = reviewerAgents.filter((agent) => agent.status !== "ready");
  const items: AgenticReleaseEvidenceItem[] = [
    releaseEvidenceItem(
      "Agent audit trail",
      "Governance Agent",
      "available",
      `Run ${auditTrail.runId} includes instruction, context, source, and output fingerprints.`,
      true,
    ),
    releaseEvidenceItem(
      "Source grounding",
      "Evidence Agent",
      controlCenter.sourceGrounding.some((item) => item.status === "missing")
        ? "missing"
        : controlCenter.sourceGrounding.some((item) => item.status === "needs-review")
          ? "needs-review"
          : "available",
      "Context completeness, current document evidence, selected text, source pack, citation TODOs, and claim inventory are captured in the control center.",
      true,
    ),
    releaseEvidenceItem(
      "Document intent sheet",
      "Planner Agent",
      plan.documentIntent.status === "ready" ? "available" : "needs-review",
      `${plan.documentIntent.completenessScore}/100 intent completeness; missing ${plan.documentIntent.missingFields.join(", ") || "none"}.`,
      true,
    ),
    releaseEvidenceItem(
      "Document memory pack",
      "Planner Agent",
      plan.documentMemory.entries.length ? "available" : "needs-review",
      plan.documentMemory.entries.length
        ? `${plan.documentMemory.entries.length} reusable terminology, style, decision, review, or distribution memory item(s) are attached.`
        : "No reusable document memory has been captured for future agent runs.",
      false,
    ),
    releaseEvidenceItem(
      "Outline variant comparison",
      "Composition Agent",
      plan.outlineVariants.length ? "needs-review" : "missing",
      plan.outlineVariants.length
        ? `${plan.outlineVariants.length} alternative outline structure(s) are available for executive-first, problem-solution, evidence-led, risk-first, and channel-specific comparison.`
        : "No outline variants were generated for comparison.",
      true,
    ),
    releaseEvidenceItem(
      "Section contract cards",
      "Planner Agent",
      sectionWorkQueue.length ? "needs-review" : "missing",
      sectionWorkQueue.length
        ? `${sectionWorkQueue.length} section contract card(s) define purpose, reader, desired outcome, evidence expectations, owner, risk, and done criteria.`
        : "No section contract cards were generated because no outline is available.",
      true,
    ),
    releaseEvidenceItem(
      "Composable section draft history",
      "Docs Live Section Agent",
      sectionDraftHistory.length ? "needs-review" : "missing",
      sectionDraftHistory.length
        ? `${sectionDraftHistory.length} section draft restore point(s) preserve prompt summaries, rationale, reviewer notes, fingerprints, and reusable Markdown.`
        : "No section draft restore points were generated.",
      true,
    ),
    releaseEvidenceItem(
      "Agent-selected transforms",
      "Transform Agent",
      transformRecommendations.length ? "needs-review" : "missing",
      transformRecommendations.length
        ? `${transformRecommendations.length} recommended calc, chart, table, diagram, timeline, schema, equation, or publishing block(s) are linked to source signals and narrative review triggers.`
        : "No agent-selected structured transform recommendations were prepared.",
      true,
    ),
    releaseEvidenceItem(
      "Data-to-narrative bridge",
      "Data Narrative Agent",
      dataNarrativeLinks.some((item) => item.status === "blocked" || item.status === "needs-review") ? "needs-review" : "available",
      dataNarrativeLinks.length
        ? `${dataNarrativeLinks.length} structured dependency link(s) connect source changes to affected narrative sections and review actions.`
        : "No data-to-narrative dependency links were prepared.",
      true,
    ),
    releaseEvidenceItem(
      "Approval metadata gate",
      "Governance Agent",
      approvalGate.status === "ready" ? "available" : approvalGate.status === "blocked" ? "missing" : "needs-review",
      approvalGate.blockers.length
        ? `${approvalGate.summary} Blockers: ${approvalGate.blockers.slice(0, 4).join("; ")}.`
        : approvalGate.summary,
      approvalGate.requiredBeforeDistribution,
    ),
    releaseEvidenceItem(
      "Agent automation scheduler",
      "Automation Scheduler",
      automationQueue.some((item) => item.status === "blocked") ? "needs-review" : "available",
      `${automationQueue.length} safe local automation check(s) are queued for evidence scan, outline critique, transform validation, export preflight, accessibility, and readiness refresh.`,
      true,
    ),
    releaseEvidenceItem(
      "Reference integrity",
      "Citation Reviewer",
      documentEvidence.referenceHints.length ? "needs-review" : "available",
      documentEvidence.referenceHints.length
        ? `${documentEvidence.referenceHints.length} label or cross-reference issue(s) require repair before handoff.`
        : "No obvious malformed or unmatched cross references were detected.",
      Boolean(distributionTargetPlans.length),
    ),
    releaseEvidenceItem(
      "Human review closure",
      "Governance Agent",
      documentEvidence.unreviewedAiMarkers || documentEvidence.unresolvedComments || taskBlockers.length ? "needs-review" : "available",
      `${documentEvidence.unreviewedAiMarkers} AI marker(s), ${documentEvidence.unresolvedComments} review comment(s), and ${taskBlockers.length} unresolved lifecycle task(s) require closure evidence.`,
      true,
    ),
    releaseEvidenceItem(
      "Reviewer sign-off",
      "Review Lead",
      reviewerBlockers.length ? "needs-review" : "available",
      reviewerBlockers.length
        ? `${reviewerBlockers.length} reviewer agent(s) still require action: ${reviewerBlockers.map((agent) => agent.label).join(", ")}.`
        : "All reviewer agents are ready.",
      true,
    ),
    releaseEvidenceItem(
      "Pre-review rehearsal",
      "Review Lead",
      preReviewRehearsal.some((item) => item.releaseBlocker) ? "needs-review" : "available",
      `${preReviewRehearsal.length} likely reviewer prompt(s) staged; ${preReviewRehearsal.filter((item) => item.releaseBlocker).length} should be resolved before formal review or release.`,
      true,
    ),
    releaseEvidenceItem(
      "Document-type quality gates",
      "Quality Agent",
      plan.qualityGates.length ? "needs-review" : "available",
      plan.qualityGates.length
        ? `${plan.qualityGates.length} quality gate(s) are staged for ${plan.documentType}; completion evidence must be attached before release.`
        : "No document-type quality gates were required.",
      true,
    ),
    releaseEvidenceItem(
      "Distribution artifacts",
      "Distribution Agent",
      distributionTargetPlans.length ? "needs-review" : "available",
      distributionTargetPlans.length
        ? `${distributionTargetPlans.map((target) => target.label).join(", ")} need generated artifacts, manifests, and handoff evidence.`
        : "No distribution target is active.",
      Boolean(distributionTargetPlans.length),
    ),
    ...distributionTargetPlans.map((targetPlan) =>
      releaseEvidenceItem(
        `${targetPlan.label} evidence`,
        "Distribution Agent",
        "needs-review",
        `${targetPlan.target} handoff requires ${targetPlan.preflightChecks.length} preflight check(s), ${targetPlan.handoffSteps.length} handoff step(s), and evidence: ${targetPlan.evidenceRequired.join("; ")}.`,
        true,
      ),
    ),
    releaseEvidenceItem(
      "Approval metadata",
      "Governance Agent",
      documentEvidence.approvalMetadataMissing.length ? "missing" : "available",
      documentEvidence.approvalMetadataMissing.length
        ? `Missing approval metadata: ${documentEvidence.approvalMetadataMissing.join(", ")}.`
        : "Required approval metadata is present or not required for this run.",
      Boolean(distributionTargetPlans.length),
    ),
    releaseEvidenceItem(
      "Provider handoff proof",
      "Provider Operator",
      plan.sourcePack.items.length || plan.lanes.some((lane) => lane === "create" || lane === "revise") ? "needs-review" : "available",
      "If an external provider is used, retain the redacted request package, response hash, imported response wrapper, and source-pack evidence.",
      false,
    ),
  ];
  const bundleBlockers = [
    ...blockers,
    ...items.filter((item) => item.requiredBeforeRelease && item.status !== "available").map((item) => `${item.label}: ${item.detail}`),
  ];
  return {
    id: `release-evidence-${auditTrail.runId}`,
    summary: bundleBlockers.length
      ? `${bundleBlockers.length} release evidence blocker(s) remain before distribution or archival.`
      : "Release evidence bundle is ready for human approval and archival.",
    items,
    blockers: bundleBlockers.slice(0, 16),
  };
}

function releaseEvidenceItem(
  label: string,
  owner: string,
  status: AgenticEvidenceStatus,
  detail: string,
  requiredBeforeRelease: boolean,
): AgenticReleaseEvidenceItem {
  return { label, owner, status, detail, requiredBeforeRelease };
}

function rollbackPlan(applicationMode: AgenticWorkflowRun["applicationMode"]) {
  if (applicationMode === "replace-selection") {
    return [
      "Review the selected range before applying the agent output.",
      "Use editor undo immediately after apply if the replacement is not acceptable.",
      "Keep the generated agent packet as review material until the reviewer accepts the change.",
    ];
  }
  if (applicationMode === "replace-document") {
    return [
      "Create or keep a snapshot before replacing the current document.",
      "Use local snapshot or Git history to restore the prior document if the draft is rejected.",
      "Keep AI provenance and review metadata visible until human review is complete.",
    ];
  }
  return [
    "Append the packet instead of overwriting existing source.",
    "Remove the appended agent packet if review rejects the proposal.",
    "Retain the run ID and fingerprints in review notes when accepting any generated section.",
  ];
}

function buildSourceGrounding(
  plan: AgenticWorkflowPlan,
  hasDocument: boolean,
  hasSelection: boolean,
  documentEvidence: AgenticDocumentEvidence,
  outlineCritique: AgenticOutlineCritiqueItem[],
): AgenticControlItem[] {
  const evidenceValue = extractKeyValue(plan.placeholderText, "evidence");
  return [
    {
      label: "User instruction",
      detail: plan.instruction ? "Plain-language intent captured as the agent objective." : "No explicit instruction was supplied.",
      status: plan.instruction ? "available" : "missing",
    },
    {
      label: "Document intent sheet",
      detail: `${plan.documentIntent.summary} Missing fields: ${plan.documentIntent.missingFields.join(", ") || "none"}.`,
      status: plan.documentIntent.status === "ready" ? "available" : "needs-review",
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
      detail: outlineCritique.length
        ? `Outline is available, with ${outlineCritique.length} critique item(s) to resolve before drafting.`
        : plan.suggestedOutline.trim()
          ? "Outline is available as the composition work queue."
          : "No outline is available for section-by-section drafting.",
      status: !plan.suggestedOutline.trim() ? "missing" : outlineCritique.length ? "needs-review" : "available",
    },
    {
      label: "Outline variants",
      detail: plan.outlineVariants.length
        ? `${plan.outlineVariants.length} alternative outline structure(s) are ready for comparison before section drafting.`
        : "No outline variants are available for comparison.",
      status: plan.outlineVariants.length ? "available" : "missing",
    },
    {
      label: "Context completeness",
      detail: `${plan.contextCompleteness.score}/100 (${plan.contextCompleteness.status}); missing ${plan.contextCompleteness.missing.join(", ") || "none"}.`,
      status: plan.contextCompleteness.status === "thin" ? "needs-review" : "available",
    },
    {
      label: "User source pack",
      detail: plan.sourcePack.items.length
        ? `${plan.sourcePack.items.length} user-managed source item(s): ${plan.sourcePack.claims.length} claim(s), ${plan.sourcePack.urls.length} URL(s), ${plan.sourcePack.files.length} file(s), ${plan.sourcePack.references.length} reference(s), ${plan.sourcePack.reviewerComments.length} reviewer comment(s).`
        : "No user-managed source pack items were supplied.",
      status: plan.sourcePack.items.length ? "available" : "needs-review",
    },
    {
      label: "Document memory",
      detail: plan.documentMemory.entries.length ? plan.documentMemory.summary : "No reusable terminology, style, decision, or review memory is available.",
      status: "available",
    },
    {
      label: "Evidence",
      detail:
        documentEvidence.citationTodos.length
          ? `${documentEvidence.citationTodos.length} citation TODO marker(s) remain in the current document.`
          : evidenceValue && !/^TBD\b/i.test(evidenceValue)
          ? `Evidence expectation captured: ${evidenceValue}.`
          : "Evidence is not yet specific enough for final claims.",
      status: documentEvidence.citationTodos.length ? "needs-review" : evidenceValue && !/^TBD\b/i.test(evidenceValue) ? "available" : "needs-review",
    },
    {
      label: "Claim inventory",
      detail: documentEvidence.claimInventory.length
        ? `${documentEvidence.claimInventory.length} candidate claim(s) extracted for source review.`
        : "No current-document claims were detected for source review.",
      status: documentEvidence.claimInventory.length ? "needs-review" : "available",
    },
    {
      label: "Reference integrity",
      detail: documentEvidence.referenceHints.length
        ? `${documentEvidence.referenceHints.length} label or cross-reference issue(s) need review.`
        : "No obvious malformed or unmatched cross references were detected.",
      status: documentEvidence.referenceHints.length ? "needs-review" : "available",
    },
    {
      label: "Document placeholders",
      detail: documentEvidence.unresolvedPlaceholders.length
        ? `Unresolved placeholders detected: ${documentEvidence.unresolvedPlaceholders.slice(0, 5).join(", ")}.`
        : "No obvious current-document placeholders were detected.",
      status: documentEvidence.unresolvedPlaceholders.length ? "needs-review" : "available",
    },
  ];
}

function buildGovernanceItems(
  plan: AgenticWorkflowPlan,
  revision: AgenticWorkflowRevision | null,
  blockers: string[],
  documentEvidence: AgenticDocumentEvidence,
  approvalGate: AgenticApprovalGate,
): AgenticControlItem[] {
  return [
    {
      label: "AI provenance",
      detail: documentEvidence.unreviewedAiMarkers
        ? `${documentEvidence.unreviewedAiMarkers} current-document AI marker(s) still need human review; generated packet also includes provenance metadata.`
        : "Agent output includes ai-source and ai-assisted review metadata.",
      status: documentEvidence.unreviewedAiMarkers ? "needs-review" : "available",
    },
    {
      label: "Human review",
      detail: documentEvidence.unresolvedComments
        ? `${documentEvidence.unresolvedComments} unresolved current-document review comment(s) remain.`
        : blockers.length
          ? "Human review remains blocked by missing inputs or workflow constraints."
          : "Reviewer can inspect QA gates before marking sections human-reviewed.",
      status: blockers.length || documentEvidence.unresolvedComments ? "needs-review" : "available",
    },
    {
      label: "Humanization",
      detail: documentEvidence.humanizationFindings.length
        ? `${documentEvidence.humanizationFindings.length} current-document phrasing item(s) need humanization before reader review.`
        : "No obvious generic AI phrasing patterns were detected.",
      status: documentEvidence.humanizationFindings.length ? "needs-review" : "available",
    },
    {
      label: "Revision audit",
      detail: revision
        ? revision.meaningDriftFindings.length
          ? `Original and proposed text are captured, with ${revision.revisionPasses.length} revision pass(es) and ${revision.meaningDriftFindings.length} meaning-drift item(s) requiring review.`
          : `Original text, proposed text, ${revision.revisionPasses.length} revision pass(es), change summary, and meaning-drift scan are captured for comparison.`
        : "No selection-aware revision is part of this run.",
      status: revision ? (revision.meaningDriftFindings.length ? "needs-review" : "available") : "needs-review",
    },
    {
      label: "Approval metadata",
      detail: approvalGate.blockers.length
        ? `${approvalGate.summary} Missing or unresolved gate evidence: ${approvalGate.blockers.slice(0, 4).join("; ")}.`
        : plan.distributionTargets.length
          ? approvalGate.summary
          : "Distribution approval metadata is not required until a target is selected.",
      status: approvalGate.status === "blocked" ? "missing" : approvalGate.status === "needs-review" ? "needs-review" : "available",
    },
    {
      label: "Approval metadata gate",
      detail: `${approvalGate.summary} Required before distribution: ${approvalGate.requiredBeforeDistribution ? "yes" : "no"}.`,
      status: approvalGate.status === "blocked" ? "missing" : approvalGate.status === "needs-review" ? "needs-review" : "available",
    },
  ];
}

function buildDistributionItems(
  plan: AgenticWorkflowPlan,
  targetPlans: AgenticDistributionTargetPlan[],
  documentEvidence: AgenticDocumentEvidence,
  approvalGate: AgenticApprovalGate,
): AgenticControlItem[] {
  if (!plan.distributionTargets.length) {
    return [
      {
        label: "Distribution targets",
        detail: "No export or publishing target is selected yet.",
        status: "needs-review",
      },
    ];
  }
  return targetPlans.map((targetPlan) => {
    const blockers = [
      approvalGate.status === "blocked" ? `${approvalGate.blockers.length} approval gate blocker(s)` : "",
      documentEvidence.brokenLinkHints.length ? `${documentEvidence.brokenLinkHints.length} placeholder or suspicious link(s)` : "",
      documentEvidence.referenceHints.length ? `${documentEvidence.referenceHints.length} reference issue(s)` : "",
    ].filter(Boolean);
    return {
      label: targetPlan.label,
      detail: blockers.length
        ? `${targetPlan.preflightChecks.length} preflight checks staged; repair ${blockers.join(" and ")} before handoff.`
        : `${targetPlan.preflightChecks.length} preflight checks, ${targetPlan.handoffSteps.length} handoff step, and ${targetPlan.evidenceRequired.length} evidence requirements are staged.`,
      status: approvalGate.status === "blocked" ? "missing" as const : "needs-review" as const,
    };
  });
}

function buildNextActions(
  plan: AgenticWorkflowPlan,
  status: AgenticControlStatus,
  blockers: string[],
  targetPlans: AgenticDistributionTargetPlan[],
  documentEvidence: AgenticDocumentEvidence,
  approvalGate: AgenticApprovalGate,
): AgenticNextAction[] {
  const actions: AgenticNextAction[] = [];
  if (approvalGate.requiredBeforeDistribution && approvalGate.status !== "ready") {
    actions.push({
      label: "Clear approval metadata gate",
      detail: approvalGate.blockers.length
        ? approvalGate.blockers.slice(0, 4).join("; ")
        : "Review approval metadata before distribution.",
      lane: "distribute",
      action: "prepare-export",
      status: approvalGate.status === "blocked" ? "blocked" : "needs-input",
    });
  }
  if (blockers.length) {
    actions.push({
      label: "Resolve missing inputs",
      detail: blockers.slice(0, 4).join("; "),
      lane: "create",
      action: "open-docs-live",
      status: "needs-input",
    });
  }
  if (documentEvidence.unresolvedPlaceholders.length) {
    actions.push({
      label: "Resolve document placeholders",
      detail: `${documentEvidence.unresolvedPlaceholders.length} placeholder marker(s) need verified values or explicit deferral.`,
      lane: "edit",
      action: "open-ai-paste",
      status: "needs-input",
    });
  }
  if (documentEvidence.citationTodos.length || documentEvidence.unresolvedComments || documentEvidence.unreviewedAiMarkers) {
    actions.push({
      label: "Review evidence and governance blockers",
      detail: [
        documentEvidence.citationTodos.length ? `${documentEvidence.citationTodos.length} citation TODO(s)` : "",
        documentEvidence.unresolvedComments ? `${documentEvidence.unresolvedComments} unresolved comment(s)` : "",
        documentEvidence.unreviewedAiMarkers ? `${documentEvidence.unreviewedAiMarkers} AI review marker(s)` : "",
      ]
        .filter(Boolean)
        .join(", "),
      lane: "review",
      action: "open-review",
      status: "needs-input",
    });
  }
  if (documentEvidence.reviewCommentResolutions.length) {
    const blockerCount = documentEvidence.reviewCommentResolutions.filter((comment) => comment.blocker).length;
    actions.push({
      label: "Resolve review comments",
      detail: `${documentEvidence.reviewCommentResolutions.length} comment(s) need resolution notes${blockerCount ? `, including ${blockerCount} blocker-risk comment(s)` : ""}.`,
      lane: "review",
      action: "open-review",
      status: blockerCount ? "blocked" : "needs-input",
    });
  }
  if (documentEvidence.claimInventory.length) {
    actions.push({
      label: "Verify claim inventory",
      detail: `${documentEvidence.claimInventory.length} candidate claim(s) need source, citation, reviewer note, or deferral.`,
      lane: "review",
      action: "open-review",
      status: "needs-input",
    });
  }
  if (documentEvidence.humanizationFindings.length) {
    actions.push({
      label: "Humanize current document",
      detail: `${documentEvidence.humanizationFindings.length} phrasing item(s) need concrete, owner-written language.`,
      lane: "revise",
      action: "open-ai-paste",
      status: "needs-input",
    });
  }
  if (documentEvidence.referenceHints.length) {
    actions.push({
      label: "Repair reference integrity",
      detail: `${documentEvidence.referenceHints.length} malformed, missing, or unmatched label/reference issue(s) need review.`,
      lane: "review",
      action: "open-review",
      status: "needs-input",
    });
  }
  if (documentEvidence.approvalMetadataMissing.length || documentEvidence.brokenLinkHints.length) {
    actions.push({
      label: "Repair distribution blockers",
      detail: [
        documentEvidence.approvalMetadataMissing.length ? `missing ${documentEvidence.approvalMetadataMissing.join(", ")}` : "",
        documentEvidence.brokenLinkHints.length ? `${documentEvidence.brokenLinkHints.length} placeholder or suspicious link(s)` : "",
      ]
        .filter(Boolean)
        .join("; "),
      lane: plan.distributionTargets.length ? "distribute" : "review",
      action: plan.distributionTargets.length ? "prepare-export" : "open-review",
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

function analyzeAgenticDocumentEvidence(documentText: string, plan: AgenticWorkflowPlan): AgenticDocumentEvidence {
  const unresolvedPlaceholders = Array.from(
    new Set(
      [
        ...documentText.matchAll(/\b(?:TBD|TODO|FIXME|PLACEHOLDER|INSERT\s+[A-Z ]+|\[[A-Z][A-Z0-9 _-]{2,}\])\b/g),
        ...documentText.matchAll(/\{\{\s*[^}]+\s*\}\}/g),
      ]
        .map((match) => match[0].replace(/\s+/g, " ").trim())
        .filter(Boolean),
    ),
  ).slice(0, 12);
  const citationTodos = Array.from(new Set(extractCitationTodoItems(documentText).map((item) => item.excerpt))).slice(0, 12);
  const unreviewedAiSources = [...documentText.matchAll(/```ai-source[\s\S]*?```/g)].filter((block) => !/\bstatus:\s*human-reviewed\b/i.test(block[0])).length;
  const unreviewedAiAssisted = [...documentText.matchAll(/<!--\s*ai-assisted:[\s\S]*?-->/g)].filter((marker) => !/\bstatus\s*=\s*human-reviewed\b/i.test(marker[0])).length;
  const reviewCommentResolutions = extractReviewCommentResolutions(documentText);
  const unresolvedComments = reviewCommentResolutions.length;
  const approvalMetadataMissing = plan.distributionTargets.length ? releaseMetadataMissing(documentText) : [];
  const brokenLinkHints = Array.from(
    new Set([...documentText.matchAll(/\]\((?:TODO|TBD|#|https?:\/\/example\.com)[^)]*\)/gi)].map((match) => match[0])),
  ).slice(0, 12);
  const referenceHints = analyzeReferenceIntegrity(documentText);
  const claimInventory = extractDocumentClaimInventory(documentText);
  const humanizationFindings = extractHumanizationFindings(documentText);

  return {
    unresolvedPlaceholders,
    citationTodos,
    claimInventory,
    humanizationFindings,
    reviewCommentResolutions,
    unreviewedAiMarkers: unreviewedAiSources + unreviewedAiAssisted,
    unresolvedComments,
    approvalMetadataMissing,
    brokenLinkHints,
    referenceHints,
  };
}

function analyzeReferenceIntegrity(documentText: string) {
  const labels = new Set<string>();
  const hints = new Set<string>();
  for (const heading of documentText.matchAll(/^(#{1,6})\s+(.+)$/gm)) {
    const text = heading[2].replace(/\{#[^}]+\}/g, "").trim();
    const slug = slugifyReferenceLabel(text);
    if (slug) labels.add(slug);
  }
  for (const label of documentText.matchAll(/\{#([^}]+)\}/g)) {
    const key = label[1].trim();
    if (!key || key.toLowerCase().startsWith("index:")) continue;
    if (!isReferenceKeyValid(key)) {
      hints.add(`Label {#${key}} has invalid characters; use letters, numbers, colon, dash, underscore, or period.`);
      continue;
    }
    labels.add(key);
  }
  for (const reference of documentText.matchAll(/\{@([^}]+)\}/g)) {
    const key = reference[1].trim();
    if (!key || !isReferenceKeyValid(key)) {
      hints.add(`Cross reference {@${key || "?"}} has invalid or empty syntax.`);
      continue;
    }
    if (!labels.has(key)) hints.add(`Cross reference {@${key}} does not match a heading slug or {#${key}} label in the current source.`);
  }
  for (const unclosed of documentText.matchAll(/\{[#@][^\n}]*$/gm)) {
    hints.add(`Unclosed reference marker: ${unclosed[0].slice(0, 80)}`);
  }
  return Array.from(hints).slice(0, 12);
}

function isReferenceKeyValid(key: string) {
  return /^[A-Za-z0-9_.:-]+$/.test(key);
}

function slugifyReferenceLabel(text: string) {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");
}

function releaseMetadataMissing(documentText: string) {
  const hasValue = (key: string) => new RegExp(`^${key}:\\s*\\S`, "im").test(documentText);
  const statusApproved = /^status:\s*(approved|published|ready|reviewed)\b/im.test(documentText);
  return [
    statusApproved ? "" : "status: approved, published, ready, or reviewed",
    hasValue("approvedBy") || hasValue("reviewer") ? "" : "approvedBy or reviewer",
    hasValue("approvedAt") ? "" : "approvedAt",
    hasValue("owner") ? "" : "owner",
    hasValue("releaseTarget") ? "" : "releaseTarget",
    hasValue("sourceConfidence") ? "" : "sourceConfidence",
  ].filter(Boolean);
}

function extractReviewCommentResolutions(documentText: string): AgenticReviewCommentResolution[] {
  const comments: AgenticReviewCommentResolution[] = [];
  const seen = new Set<string>();
  const pattern = /<!--\s*comment:\s*unresolved\b([\s\S]*?)-->/gi;
  for (const match of documentText.matchAll(pattern)) {
    const raw = (match[1] || "").replace(/\s+/g, " ").trim();
    const parsed = parseReviewCommentBody(raw);
    const line = sourceLineForIndex(documentText, match.index || 0);
    const excerpt = parsed.excerpt || "Unresolved review comment";
    const key = `${line}:${excerpt.toLowerCase()}`;
    if (seen.has(key)) continue;
    seen.add(key);
    comments.push({
      id: `review-comment-${line}-${stableFingerprint(`${line}\n${parsed.author}\n${excerpt}`).slice(0, 8)}`,
      line,
      author: parsed.author,
      createdAt: parsed.createdAt,
      excerpt,
      requiredAction: reviewCommentRequiredAction(excerpt),
      resolutionOptions: reviewCommentResolutionOptions(excerpt),
      blocker: reviewCommentIsBlocker(excerpt),
    });
    if (comments.length >= 24) break;
  }
  return comments;
}

function parseReviewCommentBody(raw: string) {
  const segments = raw
    .replace(/^\|/, "")
    .split("|")
    .map((segment) => segment.trim())
    .filter(Boolean);
  let author = "Reviewer";
  let createdAt = "";
  const notes: string[] = [];
  for (const segment of segments) {
    const keyValue = segment.match(/^([a-z][a-z0-9_-]*)\s*:\s*(.+)$/i);
    if (keyValue) {
      const key = keyValue[1].toLowerCase();
      const value = keyValue[2].trim();
      if (key === "author" || key === "reviewer" || key === "by") {
        author = value;
      } else if (key === "at" || key === "date" || key === "created") {
        createdAt = value;
      } else if (key === "note" || key === "comment" || key === "text") {
        notes.push(value);
      }
      continue;
    }
    notes.push(segment);
  }
  return {
    author: author.slice(0, 80),
    createdAt: createdAt.slice(0, 80),
    excerpt: (notes.join(" | ") || raw || "Unresolved review comment").slice(0, 240),
  };
}

function reviewCommentRequiredAction(excerpt: string) {
  if (/\b(source|citation|evidence|proof|data|metric|finance|forecast|basis)\b/i.test(excerpt)) {
    return "Attach source evidence, add a citation TODO, or document why the claim is being carried forward.";
  }
  if (/\b(approval|approved|legal|contract|compliance|policy|risk)\b/i.test(excerpt)) {
    return "Get the named approval or record a risk owner and explicit carry-forward decision.";
  }
  if (/\b(tone|clarity|rewrite|wording|audience|concise)\b/i.test(excerpt)) {
    return "Revise the wording, then record whether the reviewer accepted the change.";
  }
  return "Answer the reviewer, apply the requested change, or carry the comment forward with owner and date.";
}

function reviewCommentResolutionOptions(excerpt: string) {
  const options = [
    "Resolve after applying the requested document change.",
    "Answer with a reviewer-visible resolution note.",
    "Carry forward intentionally with owner, date, and release impact.",
  ];
  if (/\b(source|citation|evidence|proof|data|metric|finance|forecast|basis)\b/i.test(excerpt)) {
    options.unshift("Attach source evidence or a citation TODO next to the affected claim.");
  }
  if (/\b(approval|approved|legal|contract|compliance|policy|risk)\b/i.test(excerpt)) {
    options.unshift("Route to the approving reviewer before distribution.");
  }
  return Array.from(new Set(options)).slice(0, 5);
}

function reviewCommentIsBlocker(excerpt: string) {
  return /\b(block(?:er|ing)?|must|required|approval|approved|legal|contract|compliance|policy|risk|source|citation|evidence|finance|forecast|basis)\b/i.test(
    excerpt,
  );
}

function sourceLineForIndex(text: string, index: number) {
  return text.slice(0, index).split(/\r?\n/).length;
}

function extractDocumentClaimInventory(documentText: string): AgenticDocumentClaim[] {
  const claims: AgenticDocumentClaim[] = [];
  const seen = new Set<string>();
  let inFence = false;
  const lines = documentText.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    const rawLine = lines[index];
    const trimmed = rawLine.trim();
    if (/^```/.test(trimmed)) {
      inFence = !inFence;
      continue;
    }
    if (
      inFence ||
      !trimmed ||
      /^---$/.test(trimmed) ||
      /^#{1,6}\s/.test(trimmed) ||
      /^<!--/.test(trimmed) ||
      /^\|?[\s:|-]+\|/.test(trimmed)
    ) {
      continue;
    }
    const text = trimmed
      .replace(/^[-*+]\s+/, "")
      .replace(/^\d+[.)]\s+/, "")
      .replace(/\s+/g, " ")
      .slice(0, 220);
    const signal = classifyClaimSignal(text);
    if (!signal) continue;
    const key = text.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    claims.push({
      kind: signal.kind,
      sourceLine: index + 1,
      text,
      reason: signal.reason,
    });
    if (claims.length >= 18) break;
  }
  return claims;
}

function classifyClaimSignal(text: string): Pick<AgenticDocumentClaim, "kind" | "reason"> | null {
  if (/"[^"]{8,}"/.test(text) || /'[^']{8,}'/.test(text)) return { kind: "quote", reason: "Quoted material should be traceable to a source." };
  if (/\b(?:will|must|shall|commit(?:s|ted)?|guarantee(?:s|d)?|approve(?:s|d)?|deliver(?:s|ed)?|launch(?:es|ed)?)\b/i.test(text)) {
    return { kind: "commitment", reason: "Commitment or obligation needs owner and approval review." };
  }
  if (/(?:[$€£]\s?\d|\b\d+(?:\.\d+)?\s?%|\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b|\b\d+(?:\.\d+)?x\b)/i.test(text)) {
    return { kind: "number", reason: "Number, currency, percentage, or multiplier needs source verification." };
  }
  if (/\b(?:Q[1-4]\s+FY\d{2,4}|FY\d{2,4}|20\d{2}|Jan(?:uary)?|Feb(?:ruary)?|Mar(?:ch)?|Apr(?:il)?|May|Jun(?:e)?|Jul(?:y)?|Aug(?:ust)?|Sep(?:t(?:ember)?)?|Oct(?:ober)?|Nov(?:ember)?|Dec(?:ember)?)\b/i.test(text)) {
    return { kind: "date", reason: "Date or time-bound statement needs source and freshness review." };
  }
  if (/\b(?:increase(?:s|d)?|decrease(?:s|d)?|reduce(?:s|d)?|grow(?:s|th)?|improve(?:s|d)?|outperform(?:s|ed)?|risk(?:s)?|because|therefore|shows?|proves?)\b/i.test(text)) {
    return { kind: "claim", reason: "Causal, performance, or risk claim needs evidence review." };
  }
  return null;
}

function extractHumanizationFindings(documentText: string): AgenticHumanizationFinding[] {
  const findings: AgenticHumanizationFinding[] = [];
  const seen = new Set<string>();
  const sentenceStarts = new Map<string, { count: number; line: number; text: string }>();
  let inFence = false;
  const lines = documentText.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    const trimmed = lines[index].trim();
    if (/^```/.test(trimmed)) {
      inFence = !inFence;
      continue;
    }
    if (inFence || !trimmed || /^#{1,6}\s/.test(trimmed) || /^---$/.test(trimmed) || /^<!--/.test(trimmed)) continue;
    const text = trimmed.replace(/^[-*+]\s+/, "").replace(/^\d+[.)]\s+/, "").replace(/\s+/g, " ").slice(0, 240);
    const start = text.match(/^([A-Z][A-Za-z'-]{2,})(?:\s+[A-Za-z'-]{2,}){0,2}/)?.[0].toLowerCase();
    if (start) {
      const current = sentenceStarts.get(start) || { count: 0, line: index + 1, text };
      sentenceStarts.set(start, { ...current, count: current.count + 1 });
    }
    for (const finding of classifyHumanizationLine(text, index + 1)) {
      const key = `${finding.kind}:${finding.text.toLowerCase()}`;
      if (seen.has(key)) continue;
      seen.add(key);
      findings.push(finding);
      if (findings.length >= 18) return findings;
    }
  }
  for (const [start, value] of sentenceStarts) {
    if (value.count >= 3 && !seen.has(`repetition:${start}`)) {
      findings.push({
        kind: "repetition",
        sourceLine: value.line,
        text: value.text,
        recommendation: `Vary repeated sentence openings such as "${start}" so the section sounds intentionally written rather than templated.`,
      });
      if (findings.length >= 18) break;
    }
  }
  return findings;
}

function classifyHumanizationLine(text: string, sourceLine: number): AgenticHumanizationFinding[] {
  const findings: AgenticHumanizationFinding[] = [];
  const genericPhrase = text.match(
    /\b(?:in today's (?:fast-paced|ever-changing|dynamic) (?:world|landscape|environment)|it is important to note|it should be noted|this comprehensive (?:guide|overview|analysis)|delve into|unlock(?:ing)? the potential|seamless(?:ly)?|robust solution|cutting-edge|game[- ]changer|leverage synergies|at the end of the day)\b/i,
  )?.[0];
  if (genericPhrase) {
    findings.push({
      kind: "generic-phrase",
      sourceLine,
      text,
      recommendation: `Replace "${genericPhrase}" with specific context, actor, action, or evidence from this document.`,
    });
  }
  if (/\b(?:clearly|obviously|undoubtedly|certainly|always|never|guaranteed|proves?|will definitely)\b/i.test(text) && !/\b(source|evidence|according to|because|citation|data)\b/i.test(text)) {
    findings.push({
      kind: "overconfident-claim",
      sourceLine,
      text,
      recommendation: "Qualify confident language or attach evidence so the claim does not sound unsupported.",
    });
  }
  const vagueTransition = text.match(/^(?:Furthermore|Moreover|Additionally|In conclusion|Overall|Ultimately|It is worth noting)\b/i)?.[0];
  if (vagueTransition) {
    findings.push({
      kind: "vague-transition",
      sourceLine,
      text,
      recommendation: `Replace "${vagueTransition}" with a transition that names the business reason, contrast, or next decision.`,
    });
  }
  return findings;
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
  editAcceptanceQueue: AgenticEditAcceptanceItem[];
  controlCenter: AgenticControlCenter;
  documentEvidence: AgenticDocumentEvidence;
  auditTrail: AgenticAuditTrail;
  releaseEvidenceBundle: AgenticReleaseEvidenceBundle;
  lifecycleTasks: AgenticLifecycleTask[];
  reviewerAgents: AgenticReviewerAgent[];
  sectionWorkQueue: AgenticSectionWorkItem[];
  sectionDraftHistory: AgenticSectionDraftHistoryItem[];
  transformRecommendations: AgenticTransformRecommendation[];
  dataNarrativeLinks: AgenticDataNarrativeLink[];
  approvalGate: AgenticApprovalGate;
  automationQueue: AgenticAutomationTask[];
  outlineCritique: AgenticOutlineCritiqueItem[];
  preReviewRehearsal: AgenticPreReviewRehearsalItem[];
  reviewChecklist: string[];
  distributionChecklist: string[];
  distributionTargetPlans: AgenticDistributionTargetPlan[];
  blockers: string[];
  generatedAt: string;
}) {
  const {
    plan,
    draftMarkdown,
    revision,
    editAcceptanceQueue,
    controlCenter,
    documentEvidence,
    auditTrail,
    releaseEvidenceBundle,
    lifecycleTasks,
    reviewerAgents,
    sectionWorkQueue,
    sectionDraftHistory,
    transformRecommendations,
    dataNarrativeLinks,
    approvalGate,
    automationQueue,
    outlineCritique,
    preReviewRehearsal,
    reviewChecklist,
    distributionChecklist,
    distributionTargetPlans,
    blockers,
    generatedAt,
  } = input;
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
    `Context completeness: ${plan.contextCompleteness.score}/100 (${plan.contextCompleteness.status})`,
    "",
    "### Document Intent Sheet",
    "",
    plan.documentIntent.summary,
    "",
    plan.documentIntent.markdown,
    "",
    ...(plan.documentIntent.reviewPrompts.length
      ? ["Intent review prompts:", ...plan.documentIntent.reviewPrompts.map((prompt) => `- [ ] ${prompt}`), ""]
      : []),
    "### Context Pack",
    "",
    fencedBlock("text", plan.context),
    "",
    "### Context Completeness",
    "",
    `Present: ${plan.contextCompleteness.present.join(", ") || "none"}`,
    "",
    `Missing: ${plan.contextCompleteness.missing.join(", ") || "none"}`,
    "",
    ...plan.contextCompleteness.recommendations.map((item) => `- [ ] ${item}`),
    "",
    "### Document Memory",
    "",
    plan.documentMemory.summary,
    "",
    ...(plan.documentMemory.entries.length
      ? [
          "| Kind | Label | Source | Detail |",
          "| --- | --- | --- | --- |",
          ...plan.documentMemory.entries.map(
            (entry) =>
              `| ${entry.kind} | ${escapeTableCell(entry.label)} | ${entry.source} | ${escapeTableCell(entry.detail)} |`,
          ),
        ]
      : ["No reusable document memory was supplied."]),
    "",
    "### User Source Pack",
    "",
    ...(plan.sourcePack.items.length
      ? [
          `Items: ${plan.sourcePack.items.length}; claims: ${plan.sourcePack.claims.length}; URLs: ${plan.sourcePack.urls.length}; files: ${plan.sourcePack.files.length}; references: ${plan.sourcePack.references.length}; reviewer comments: ${plan.sourcePack.reviewerComments.length}.`,
          "",
          ...plan.sourcePack.items.map((item) => `- [${item.kind}] ${item.label}: ${item.detail}`),
        ]
      : ["No user-managed source pack items were supplied."]),
    "",
    "### Document-Type Quality Gates",
    "",
    ...qualityGatesMarkdown(plan.qualityGates),
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

  lines.push(...outlineVariantsMarkdown(plan.outlineVariants));

  if (plan.revisionModes.length) {
    lines.push("### Planned Revision Modes", "", ...plan.revisionModes.map((mode) => `- ${revisionPassProfile(mode).label}`), "");
  }
  if (blockers.length) {
    lines.push("### Blockers", "", ...blockers.map((blocker) => `- [ ] ${blocker}`), "");
  }
  lines.push(...controlCenterMarkdown(controlCenter));
  lines.push(...outlineCritiqueMarkdown(outlineCritique));
  lines.push(...claimInventoryMarkdown(documentEvidence.claimInventory));
  lines.push(...humanizationFindingsMarkdown(documentEvidence.humanizationFindings));
  lines.push(...reviewCommentResolutionsMarkdown(documentEvidence.reviewCommentResolutions));
  lines.push(...editAcceptanceQueueMarkdown(editAcceptanceQueue));
  lines.push(...lifecycleTasksMarkdown(lifecycleTasks));
  lines.push(...reviewerAgentsMarkdown(reviewerAgents));
  lines.push(...preReviewRehearsalMarkdown(preReviewRehearsal));
  lines.push(...sectionWorkQueueMarkdown(sectionWorkQueue));
  lines.push(...sectionDraftHistoryMarkdown(sectionDraftHistory));
  lines.push(...transformRecommendationsMarkdown(transformRecommendations));
  lines.push(...dataNarrativeLinksMarkdown(dataNarrativeLinks));
  lines.push(...buildAgenticApprovalGateMarkdown(approvalGate).split("\n"));
  lines.push(...automationQueueMarkdown(automationQueue));
  lines.push(...auditTrailMarkdown(auditTrail));
  lines.push(...releaseEvidenceBundleMarkdown(releaseEvidenceBundle));
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
      "### Revision Passes",
      "",
      ...revisionPassesMarkdown(revision.revisionPasses),
      "",
      "### Meaning Drift",
      "",
      ...meaningDriftMarkdown(revision.meaningDriftFindings),
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

function editAcceptanceQueueMarkdown(editAcceptanceQueue: AgenticEditAcceptanceItem[]) {
  if (!editAcceptanceQueue.length) return [];
  const lines = ["## Edit Acceptance Queue", ""];
  for (const item of editAcceptanceQueue) {
    lines.push(
      `### ${item.heading}`,
      "",
      `Scope: ${item.scope}`,
      "",
      `Recommendation: ${item.recommendation}`,
      "",
      "Change summary:",
      ...item.changeSummary.map((summary) => `- ${summary}`),
      "",
      "Risk notes:",
      ...item.riskNotes.map((note) => `- [ ] ${note}`),
      "",
      "Original:",
      "",
      fencedBlock("markdown", item.originalText || "(No source text supplied.)"),
      "",
      "Proposed:",
      "",
      item.proposedText,
      "",
    );
  }
  return lines;
}

function qualityGatesMarkdown(qualityGates: AgenticQualityGate[]) {
  if (!qualityGates.length) return ["- No document-type quality gates were selected."];
  return qualityGates.flatMap((gate) => [
    `- [ ] ${gate.label} (${gate.appliesTo}): ${gate.detail}`,
    ...gate.evidenceRequired.map((item) => `  - Evidence: ${item}`),
  ]);
}

function revisionPassesMarkdown(revisionPasses: AgenticRevisionPass[]) {
  if (!revisionPasses.length) return ["- No explicit revision passes were detected for this run."];
  return revisionPasses.flatMap((pass) => [
    `- [ ] ${pass.label}: ${pass.rationale}`,
    ...pass.checklist.map((item) => `  - ${item}`),
  ]);
}

function meaningDriftMarkdown(findings: AgenticMeaningDriftFinding[]) {
  if (!findings.length) return ["- No changed or missing numbers, dates, commitments, or caveats were detected."];
  return findings.map(
    (finding) =>
      `- [ ] ${titleCase(finding.kind)} ${finding.severity}: ${finding.detail} Recommendation: ${finding.recommendation}`,
  );
}

function lifecycleTasksMarkdown(lifecycleTasks: AgenticLifecycleTask[]) {
  const lines = ["## Agent Lifecycle Task Board", ""];
  for (const task of lifecycleTasks) {
    lines.push(
      `### ${task.title}`,
      "",
      `Lane: ${task.lane}`,
      "",
      `Owner: ${task.owner}`,
      "",
      `Status: ${task.status}`,
      "",
      task.sectionId ? `Section ID: ${task.sectionId}` : "",
      task.target ? `Target: ${task.target}` : "",
      task.sectionId || task.target ? "" : "",
      `Next step: ${task.nextStep}`,
      "",
      "Evidence:",
      ...task.evidence.map((item) => `- ${item}`),
      "",
    );
  }
  return lines;
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

function outlineVariantsMarkdown(variants: AgenticOutlineVariant[]) {
  const lines = ["### Outline Variants", ""];
  if (!variants.length) {
    return [...lines, "No outline variants were generated.", ""];
  }
  for (const variant of variants) {
    lines.push(
      `#### ${variant.label}`,
      "",
      `Strategy: ${variant.strategy}`,
      "",
      variant.summary,
      "",
      "Best for:",
      ...variant.bestFor.map((item) => `- ${item}`),
      "",
      "Tradeoffs:",
      ...variant.tradeoffs.map((item) => `- ${item}`),
      "",
      "Risks:",
      ...variant.risks.map((item) => `- ${item}`),
      "",
      fencedBlock("text", variant.outline),
      "",
    );
  }
  return lines;
}

function outlineCritiqueMarkdown(outlineCritique: AgenticOutlineCritiqueItem[]) {
  if (!outlineCritique.length) {
    return ["## Outline Critique", "", "No outline critique items were detected.", ""];
  }
  return [
    "## Outline Critique",
    "",
    "| Severity | Area | Heading | Finding | Recommendation |",
    "| --- | --- | --- | --- | --- |",
    ...outlineCritique.map(
      (item) =>
        `| ${item.severity} | ${item.area} | ${escapeTableCell(item.heading)} | ${escapeTableCell(item.detail)} | ${escapeTableCell(item.recommendation)} |`,
    ),
    "",
  ];
}

function claimInventoryMarkdown(claimInventory: AgenticDocumentClaim[]) {
  if (!claimInventory.length) {
    return ["## Claim Inventory", "", "No current-document claims were detected for source review.", ""];
  }
  return [
    "## Claim Inventory",
    "",
    "| Line | Type | Review trigger | Claim text |",
    "| ---: | --- | --- | --- |",
    ...claimInventory.map(
      (claim) =>
        `| ${claim.sourceLine} | ${claim.kind} | ${escapeTableCell(claim.reason)} | ${escapeTableCell(claim.text)} |`,
    ),
    "",
  ];
}

function humanizationFindingsMarkdown(findings: AgenticHumanizationFinding[]) {
  if (!findings.length) {
    return ["## Humanization Findings", "", "No current-document humanization findings were detected.", ""];
  }
  return [
    "## Humanization Findings",
    "",
    "| Line | Pattern | Text | Recommendation |",
    "| ---: | --- | --- | --- |",
    ...findings.map(
      (finding) =>
        `| ${finding.sourceLine} | ${finding.kind} | ${escapeTableCell(finding.text)} | ${escapeTableCell(finding.recommendation)} |`,
    ),
    "",
  ];
}

function reviewCommentResolutionsMarkdown(comments: AgenticReviewCommentResolution[]) {
  if (!comments.length) {
    return ["## Review Comment Resolution Queue", "", "No unresolved current-document review comments were detected.", ""];
  }
  return [
    "## Review Comment Resolution Queue",
    "",
    "| Line | Author | Blocker | Comment | Required action | Resolution options |",
    "| ---: | --- | --- | --- | --- | --- |",
    ...comments.map(
      (comment) =>
        `| ${comment.line} | ${escapeTableCell(comment.author)} | ${comment.blocker ? "yes" : "no"} | ${escapeTableCell(comment.excerpt)} | ${escapeTableCell(comment.requiredAction)} | ${escapeTableCell(comment.resolutionOptions.join("; "))} |`,
    ),
    "",
  ];
}

function reviewerAgentsMarkdown(reviewerAgents: AgenticReviewerAgent[]) {
  const lines = ["## Review Agents", ""];
  for (const agent of reviewerAgents) {
    lines.push(
      `### ${agent.label}`,
      "",
      `Status: ${agent.status}`,
      "",
      `Mandate: ${agent.mandate}`,
      "",
      "Findings:",
      ...agent.findings.map((item) => `- ${item}`),
      "",
      "Required actions:",
      ...agent.requiredActions.map((item) => `- [ ] ${item}`),
      "",
    );
  }
  return lines;
}

function preReviewRehearsalMarkdown(items: AgenticPreReviewRehearsalItem[]) {
  if (!items.length) {
    return ["## Pre-Review Rehearsal", "", "No pre-review rehearsal prompts were generated.", ""];
  }
  return [
    "## Pre-Review Rehearsal",
    "",
    "| Type | Reviewer | Blocker | Prompt | Why it matters | Suggested response |",
    "| --- | --- | --- | --- | --- | --- |",
    ...items.map(
      (item) =>
        `| ${item.kind} | ${item.reviewer} | ${item.releaseBlocker ? "yes" : "no"} | ${escapeTableCell(item.prompt)} | ${escapeTableCell(item.whyItMatters)} | ${escapeTableCell(item.suggestedResponse)} |`,
    ),
    "",
  ];
}

function sectionWorkQueueMarkdown(sectionWorkQueue: AgenticSectionWorkItem[]) {
  const lines = ["## Section Work Queue", ""];
  for (const section of sectionWorkQueue) {
    lines.push(
      `### ${section.order}. ${section.heading}`,
      "",
      `Level: ${section.level}`,
      "",
      `Lane: ${section.lane}`,
      "",
      `Drafting depth: ${section.draftingDepth}`,
      "",
      `Contract risk: ${section.contract.riskLevel}`,
      "",
      "Section contract:",
      `- Purpose: ${section.contract.purpose}`,
      `- Target reader: ${section.contract.targetReader}`,
      `- Desired outcome: ${section.contract.desiredDecision}`,
      `- Owner: ${section.contract.owner}`,
      "",
      "Evidence expectations:",
      ...section.contract.evidenceExpectations.map((item) => `- [ ] ${item}`),
      "",
      "Contract done criteria:",
      ...section.contract.doneCriteria.map((item) => `- [ ] ${item}`),
      "",
      `Reviewers: ${section.reviewerAgentIds.join(", ")}`,
      "",
      section.draftingInstruction,
      "",
      "Completion criteria:",
      ...section.completionCriteria.map((item) => `- [ ] ${item}`),
      "",
    );
  }
  return lines;
}

function sectionDraftHistoryMarkdown(items: AgenticSectionDraftHistoryItem[]) {
  if (!items.length) {
    return ["## Section Draft History", "", "No composable section draft restore points were generated.", ""];
  }
  const lines = [
    "## Section Draft History",
    "",
    "| Version | Section | Status | Section fingerprint | Prompt summary |",
    "| --- | --- | --- | --- | --- |",
    ...items.map(
      (item) =>
        `| ${escapeTableCell(item.versionLabel)} | ${escapeTableCell(item.sectionHeading)} | ${item.acceptanceStatus} | ${item.sectionFingerprint} | ${escapeTableCell(item.promptSummary)} |`,
    ),
    "",
  ];
  for (const item of items) {
    lines.push(
      `### ${item.versionLabel}: ${item.sectionHeading}`,
      "",
      `Source fingerprint: ${item.sourceFingerprint}`,
      "",
      `Rationale: ${item.rationale}`,
      "",
      "Reviewer notes:",
      ...item.reviewerNotes.map((note) => `- [ ] ${note}`),
      "",
      "Restore point:",
      "",
      fencedBlock("markdown", item.restorePointMarkdown),
      "",
    );
  }
  return lines;
}

function transformRecommendationsMarkdown(items: AgenticTransformRecommendation[]) {
  if (!items.length) {
    return ["## Agent-Selected Transforms", "", "No transform recommendations were prepared for this run.", ""];
  }
  const lines = [
    "## Agent-Selected Transforms",
    "",
    "| Kind | Recommendation | Target | Risk | Trigger |",
    "| --- | --- | --- | --- | --- |",
    ...items.map(
      (item) =>
        `| ${item.kind} | ${escapeTableCell(item.label)} | ${escapeTableCell(item.insertionTarget)} | ${item.riskLevel} | ${escapeTableCell(item.narrativeReviewTrigger)} |`,
    ),
    "",
  ];
  for (const item of items) {
    lines.push(
      `### ${item.label}`,
      "",
      `Owner: ${item.owner}`,
      "",
      `Kind: ${item.kind}`,
      "",
      ...(item.templateId ? [`Template: ${item.templateId}`, ""] : []),
      `Insertion target: ${item.insertionTarget}`,
      "",
      `Source signal: ${item.sourceSignal}`,
      "",
      `Narrative review trigger: ${item.narrativeReviewTrigger}`,
      "",
      "Evidence required:",
      ...item.evidenceRequired.map((evidence) => `- [ ] ${evidence}`),
      "",
      "Suggested block:",
      "",
      fencedBlock("markdown", item.suggestedMarkdown.trim()),
      "",
    );
  }
  return lines;
}

function dataNarrativeLinksMarkdown(items: AgenticDataNarrativeLink[]) {
  if (!items.length) {
    return ["## Data-to-Narrative Bridge", "", "No data-to-narrative dependency links were prepared.", ""];
  }
  return [
    "## Data-to-Narrative Bridge",
    "",
    "| Source | Affected section | Status | Review action |",
    "| --- | --- | --- | --- |",
    ...items.map(
      (item) =>
        `| ${escapeTableCell(`${item.sourceKind}: ${item.sourceLabel}`)} | ${escapeTableCell(item.affectedSection)} | ${item.status} | ${escapeTableCell(item.reviewAction)} |`,
    ),
    "",
    "### Dependency Details",
    "",
    ...items.flatMap((item) => [
      `#### ${item.sourceLabel}`,
      "",
      `Change signal: ${item.changeSignal}`,
      "",
      `Narrative risk: ${item.narrativeRisk}`,
      "",
      `Owner: ${item.owner}`,
      "",
      "Evidence required:",
      ...item.evidenceRequired.map((evidence) => `- [ ] ${evidence}`),
      "",
    ]),
  ];
}

function automationQueueMarkdown(items: AgenticAutomationTask[]) {
  if (!items.length) {
    return ["## Agent Automation Scheduler", "", "No safe automation tasks were queued for this run.", ""];
  }
  return [
    "## Agent Automation Scheduler",
    "",
    "| Check | Owner | Status | Safe | Trigger | Next step |",
    "| --- | --- | --- | --- | --- | --- |",
    ...items.map(
      (item) =>
        `| ${escapeTableCell(item.label)} | ${escapeTableCell(item.owner)} | ${item.status} | ${item.safeToAutoRun ? "yes" : "manual"} | ${escapeTableCell(item.trigger)} | ${escapeTableCell(item.nextStep)} |`,
    ),
    "",
    "### Evidence Inputs",
    "",
    ...items.flatMap((item) => [
      `#### ${item.label}`,
      "",
      ...item.evidence.map((evidence) => `- [ ] ${evidence}`),
      item.manualOnlyReason ? `- Manual-only: ${item.manualOnlyReason}` : "",
      "",
    ].filter(Boolean)),
  ];
}

function auditTrailMarkdown(auditTrail: AgenticAuditTrail) {
  return [
    "## Agent Audit Trail",
    "",
    `Run ID: ${auditTrail.runId}`,
    "",
    `Generated: ${auditTrail.generatedAt}`,
    "",
    `Planner: ${auditTrail.plannerVersion}`,
    "",
    `Apply mode: ${auditTrail.applicationMode}`,
    "",
    "| Fingerprint | Value |",
    "| --- | --- |",
    `| Instruction | ${auditTrail.instructionFingerprint} |`,
    `| Context | ${auditTrail.contextFingerprint} |`,
    `| Source | ${auditTrail.sourceFingerprint} |`,
    `| Output payload | ${auditTrail.outputFingerprint} |`,
    "",
    "### Rollback Plan",
    "",
    ...auditTrail.rollbackPlan.map((item) => `- ${item}`),
    "",
    "### Review Events",
    "",
    ...auditTrail.reviewEvents.map((item) => `- ${item}`),
    "",
  ];
}

function releaseEvidenceBundleMarkdown(bundle: AgenticReleaseEvidenceBundle) {
  return [
    "## Release Evidence Bundle",
    "",
    `Bundle ID: ${bundle.id}`,
    "",
    bundle.summary,
    "",
    "| Evidence | Owner | Status | Required | Detail |",
    "| --- | --- | --- | --- | --- |",
    ...bundle.items.map(
      (item) =>
        `| ${escapeTableCell(item.label)} | ${escapeTableCell(item.owner)} | ${item.status} | ${item.requiredBeforeRelease ? "yes" : "no"} | ${escapeTableCell(item.detail)} |`,
    ),
    "",
    ...(bundle.blockers.length ? ["### Release Evidence Blockers", "", ...bundle.blockers.map((blocker) => `- [ ] ${blocker}`), ""] : []),
  ];
}

function fencedBlock(language: string, value: string) {
  return ["```" + language, value.trim() || "(empty)", "```"].join("\n");
}

function escapeTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
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
  const keys = [
    "audience",
    "owner",
    "deadline",
    "tone",
    "evidence",
    "reviewer",
    "approvedBy",
    "approvedAt",
    "status",
    "releaseTarget",
    "sourceConfidence",
    "client",
    "company",
    "distribution",
  ];
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

function compactTimestamp(value: string) {
  return value.replace(/[^0-9A-Za-z]+/g, "").slice(0, 15) || "undated";
}

export function stableFingerprint(value: string) {
  let first = 0x811c9dc5;
  let second = 0x9e3779b9;
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    first ^= code;
    first = Math.imul(first, 0x01000193) >>> 0;
    second ^= code + index;
    second = Math.imul(second, 0x85ebca6b) >>> 0;
  }
  return `${first.toString(16).padStart(8, "0")}${second.toString(16).padStart(8, "0")}`;
}

function titleCase(value: string) {
  return value
    .replace(/[-_]+/g, " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase())
    .trim();
}
