import { parseOutlinePlan, type OutlinePlanItem } from "./documentOutline.js";

export const docsLiveDocumentTypes = [
  { id: "business-brief", label: "Business brief" },
  { id: "board-memo", label: "Board memo" },
  { id: "proposal", label: "Proposal" },
  { id: "rfp-response", label: "RFP response" },
  { id: "rfq-response", label: "RFQ response" },
  { id: "tender-response", label: "Tender response" },
  { id: "tutorial", label: "Tutorial" },
  { id: "lesson-plan", label: "Lesson plan" },
  { id: "lesson-content", label: "Lesson content" },
  { id: "technical-textbook", label: "Technical textbook" },
  { id: "novel", label: "Novel" },
  { id: "podcast-script", label: "Podcast script" },
  { id: "movie-script", label: "Movie script" },
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

export interface DocsLiveOutlineTypeInput {
  docsLiveType?: string;
  id?: string;
  name?: string;
  category?: string;
  tags?: string[];
  bestFor?: string[];
}

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

export type DocsLiveDraftDepth = "summary" | "standard" | "detailed" | "technical" | "legal" | "executive";
export type DocsLivePlaceholderKind =
  | "text"
  | "person"
  | "date"
  | "money"
  | "number"
  | "source"
  | "reviewer"
  | "client"
  | "decision"
  | "channel";
export type DocsLivePlaceholderReviewStatus = "provided" | "needs-review" | "verified";

export interface DocsLivePlaceholderEntry {
  key: string;
  value: string;
  kind: DocsLivePlaceholderKind;
  source: string;
  reviewStatus: DocsLivePlaceholderReviewStatus;
}

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

export interface DocsLiveSuggestedAnswer {
  id: string;
  stepLabel: string;
  question: string;
  answer: string;
  source: string;
  rationale: string;
  contextSignals: string[];
}

export interface DocsLiveDecisionPrompt {
  key: string;
  label: string;
  placeholder: string;
  help: string;
  kind: DocsLivePlaceholderKind;
}

export interface DocsLiveWizardProfile {
  documentType: DocsLiveDocumentType;
  label: string;
  planningLabel: string;
  sequencingLabel: string;
  qualityLabel: string;
  unitLabel: string;
  defaultOutlineMarkdown: string;
  decisionPrompts: DocsLiveDecisionPrompt[];
  planningArtifacts: string[];
  qualityChecks: string[];
}

export interface DocsLiveWorkflowStep {
  id: string;
  label: string;
  status: "ready" | "needs-input" | "complete";
  detail: string;
  assistance: string;
  contextSignals: string[];
}

export interface DocsLiveSectionDraft {
  title: string;
  level: number;
  planningOnly: boolean;
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
  status: "complete" | "needs-review" | "needs-input";
  detail: string;
}

export interface DocsLiveReviewPacket {
  contextSources: string[];
  sectionRunbook: string[];
  qaRegister: string[];
  humanizationChecklist: string[];
  reviewerHandoff: string[];
}

export interface DocsLiveReviewPacketMarkdownOptions {
  generatedAt: string;
  source?: string;
}

interface DocsLiveBlueprint {
  label: string;
  defaultOutline: string[];
  questions: string[];
  sectionFocus: string[];
  workflow?: DocsLiveWorkflowProfile;
}

interface DocsLiveWorkflowProfile {
  planningLabel: string;
  planningInstruction: string;
  planningArtifacts: string[];
  sequencingLabel: string;
  sequencingInstruction: string;
  sequenceAcceptance: string[];
  qualityLabel: string;
  qualityInstruction: string;
  qualityChecks: string[];
  unitLabel: string;
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
  "rfp-response": {
    label: "RFP response",
    defaultOutline: ["Executive Response", "Compliance Matrix", "Understanding of Requirements", "Proposed Solution", "Implementation Plan", "Team and Experience", "Pricing Response", "Risk and Assumptions", "Appendices"],
    questions: [
      "What RFP requirements, evaluation criteria, and mandatory attachments must be mirrored?",
      "Where are responses compliant, partially compliant, or exceptions?",
      "Which evidence, prior experience, or proof points should be cited?",
      "What submission deadline, format, and approvals are required?",
    ],
    sectionFocus: ["buyer", "compliance", "requirements", "solution", "delivery", "team", "pricing", "risk", "submission"],
  },
  "rfq-response": {
    label: "RFQ response",
    defaultOutline: ["Quotation Summary", "Buyer Requirements", "Quoted Items", "Pricing Table", "Inclusions", "Exclusions", "Delivery Schedule", "Commercial Terms", "Validity and Acceptance"],
    questions: [
      "What products, services, quantities, and units are being quoted?",
      "Which pricing basis, inclusions, exclusions, taxes, or shipping terms matter?",
      "What delivery schedule and validity period should be stated?",
      "Who can accept the quotation and by what date?",
    ],
    sectionFocus: ["buyer", "requirements", "items", "pricing", "inclusions", "exclusions", "delivery", "terms", "acceptance"],
  },
  "tender-response": {
    label: "Tender response",
    defaultOutline: ["Bid Summary", "Mandatory Submission Checklist", "Compliance Statement", "Technical Methodology", "Work Plan", "Key Personnel", "Quality and Risk Management", "Commercial Offer", "Required Attachments"],
    questions: [
      "What tender instructions, mandatory documents, and submission rules must be followed?",
      "Which technical method, staffing plan, quality controls, and risk mitigations are required?",
      "Which eligibility, legal, tax, or registration evidence must be attached?",
      "What approval path is needed before submission?",
    ],
    sectionFocus: ["bid", "checklist", "compliance", "method", "plan", "team", "quality", "commercial", "attachments"],
  },
  tutorial: {
    label: "Tutorial",
    defaultOutline: ["Learning Goals", "Audience and Prerequisites", "Before You Begin", "Step-by-Step Walkthrough", "Practice Exercise", "Troubleshooting", "Next Steps"],
    questions: [
      "Who is learning this and what should they be able to do afterward?",
      "What prerequisites, permissions, tools, or examples are needed?",
      "Which steps require screenshots, checks, or warnings?",
      "What troubleshooting guidance and next steps should be included?",
    ],
    sectionFocus: ["learning goals", "audience", "prerequisites", "steps", "practice", "troubleshooting", "next steps"],
  },
  "lesson-plan": {
    label: "Lesson plan",
    defaultOutline: ["Learning Objectives", "Standards and Prerequisites", "Materials", "Lesson Flow", "Guided Practice", "Assessment", "Differentiation", "Homework or Extension"],
    questions: [
      "Who are the learners and what should they know or do by the end?",
      "Which standards, prior knowledge, materials, or accessibility needs matter?",
      "How should the lesson move through warm-up, instruction, practice, assessment, and closure?",
      "What evidence will show that learners understood the lesson?",
    ],
    sectionFocus: ["objectives", "standards", "materials", "sequence", "practice", "assessment", "differentiation", "extension"],
  },
  "lesson-content": {
    label: "Lesson content",
    defaultOutline: ["Opening Hook", "Core Explanation", "Worked Example", "Practice Activity", "Knowledge Check", "Discussion Prompts", "Teacher Notes", "Learner Handout"],
    questions: [
      "What concept, skill, or process should the lesson content teach?",
      "Which examples, analogies, visuals, equations, or source materials should be included?",
      "What learner misconceptions or difficult steps need extra support?",
      "What checks, prompts, and handout material should be generated?",
    ],
    sectionFocus: ["hook", "explanation", "example", "practice", "check", "discussion", "teacher notes", "handout"],
  },
  "technical-textbook": {
    label: "Technical textbook",
    defaultOutline: [
      "Textbook Architecture",
      "Chapter Outline",
      "Reader Prerequisites",
      "Chapter 1 - Conceptual Foundation",
      "Chapter 2 - Technical Model",
      "Chapter 3 - Worked Examples",
      "Chapter 4 - Practice Exercises",
      "Chapter 5 - Pitfalls and Review",
      "Instructional Quality Review",
    ],
    questions: [
      "What subject, reader level, and prerequisite knowledge should the chapter assume?",
      "What textbook outline, chapter order, learning outcomes, and prerequisite sequence should be locked before prose is drafted?",
      "Which definitions, equations, code, diagrams, or technical standards must be covered?",
      "Which worked examples, exercises, glossary terms, citations, and review questions should be assigned to each chapter?",
      "What instructional quality criteria should be checked after the sequential chapter draft is complete?",
    ],
    sectionFocus: ["textbook architecture", "chapter outline", "prerequisites", "conceptual foundation", "technical model", "worked examples", "practice exercises", "pitfalls and review", "instructional QA"],
    workflow: {
      planningLabel: "Textbook architecture",
      planningInstruction:
        "Lock the textbook outline before prose is drafted: chapter order, learning outcomes, prerequisites, examples, exercises, and assessment logic.",
      planningArtifacts: [
        "Subject scope and reader level",
        "Prerequisite map and chapter order",
        "Learning outcomes for every chapter",
        "Notation, glossary terms, equations, code, or standards that must stay consistent",
        "Worked examples, exercises, assessment path, and citation expectations",
      ],
      sequencingLabel: "Sequential chapter drafting",
      sequencingInstruction:
        "Draft chapters in order, carrying definitions, notation, examples, exercises, and learner scaffolding forward only after the previous chapter contract is reviewed.",
      sequenceAcceptance: [
        "Chapter purpose, prerequisite dependency, and learner outcome are explicit before drafting.",
        "New definitions, notation, equations, code, and examples are introduced once and reused consistently.",
        "Exercises and checks reinforce the current chapter before relying on later material.",
        "The chapter handoff names what the next chapter may assume.",
      ],
      qualityLabel: "Instructional quality review",
      qualityInstruction:
        "Review the completed chapter sequence for technical accuracy, learning progression, equation/code integrity, exercise coverage, glossary consistency, and citation readiness.",
      qualityChecks: [
        "Technical claims, equations, code, diagrams, and standards are accurate and source-ready.",
        "The learning progression has no skipped prerequisite, unexplained term, or silent notation change.",
        "Examples, exercises, checks, and assessments match the stated outcomes.",
        "Glossary, citations, review questions, and unresolved assumptions are ready for human review.",
      ],
      unitLabel: "chapter",
    },
  },
  novel: {
    label: "Novel",
    defaultOutline: [
      "Story Premise",
      "Character Arcs",
      "World and Continuity Rules",
      "Plot Outline",
      "Chapter 1 - Opening Image",
      "Chapter 2 - Inciting Incident",
      "Chapter 3 - Rising Complications",
      "Chapter 4 - Midpoint Reversal",
      "Chapter 5 - Crisis and Climax",
      "Chapter 6 - Resolution",
      "Narrative Quality Review",
    ],
    questions: [
      "What genre, premise, point of view, tense, and target reader should guide the draft?",
      "What plot outline, character arcs, world rules, and chapter sequence should be locked before prose is drafted?",
      "Who are the central characters and what do they want, fear, hide, or change?",
      "What conflict, stakes, setting rules, and thematic questions should shape the story?",
      "What narrative quality criteria should be checked after chapters are drafted sequentially?",
    ],
    sectionFocus: ["premise", "character arcs", "continuity rules", "plot outline", "opening image", "inciting incident", "rising complications", "midpoint reversal", "crisis and climax", "resolution", "narrative QA"],
    workflow: {
      planningLabel: "Plot architecture",
      planningInstruction:
        "Lock the plot before prose is drafted: premise, character arcs, point of view, world rules, act turns, chapter order, and continuity promises.",
      planningArtifacts: [
        "Genre, premise, point of view, tense, and target reader promise",
        "Protagonist goal, fear, flaw, pressure, and change arc",
        "Central conflict, stakes, antagonistic force, and thematic question",
        "World rules, continuity promises, act turns, and chapter beat outline",
        "Voice, pacing, scene style, and revision constraints",
      ],
      sequencingLabel: "Sequential chapter drafting",
      sequencingInstruction:
        "Draft chapters in order so causality, character motivation, tension, revelations, scene goals, and continuity evolve deliberately from one chapter to the next.",
      sequenceAcceptance: [
        "Chapter goal, conflict, turn, emotional consequence, and open question are clear before prose expansion.",
        "Character motivation follows from the previous chapter rather than resetting.",
        "Revelations, stakes, setting details, and continuity promises do not contradict earlier chapters.",
        "The chapter ending gives the next chapter a specific causal handoff.",
      ],
      qualityLabel: "Narrative quality review",
      qualityInstruction:
        "Review the completed chapter sequence for story logic, emotional causality, character arc movement, voice, pacing, scene necessity, continuity, and AI-sounding prose.",
      qualityChecks: [
        "Story logic, emotional causality, stakes escalation, and chapter-to-chapter continuity hold together.",
        "Character choices reveal motivation, pressure, and change rather than summarizing intent.",
        "Scenes earn their place through goal, conflict, turn, revelation, or consequence.",
        "Voice, pacing, sensory specificity, dialogue, and prose texture do not read like generic AI output.",
      ],
      unitLabel: "chapter",
    },
  },
  "podcast-script": {
    label: "Podcast script",
    defaultOutline: [
      "Episode Architecture",
      "Segment Rundown",
      "Cold Open",
      "Intro",
      "Segment 1",
      "Segment 2",
      "Guest Questions",
      "Sponsor or Promo Read",
      "Outro",
      "Production Notes",
      "Audio Production Review",
    ],
    questions: [
      "What show format, audience, tone, and episode objective should the script follow?",
      "What segment rundown, timing, host promise, guest arc, and listener takeaway should be locked before script copy is drafted?",
      "Who are the hosts or guests and what should each segment accomplish?",
      "What stories, facts, sound cues, sponsor copy, or calls to action are required?",
      "What audio production criteria should be checked after the segment sequence is drafted?",
    ],
    sectionFocus: ["episode architecture", "rundown", "hook", "intro", "segment argument", "segment story", "guest arc", "sponsor read", "outro", "production", "audio QA"],
    workflow: {
      planningLabel: "Episode architecture",
      planningInstruction:
        "Lock the episode architecture before script copy is drafted: audience promise, segment order, timing, host and guest roles, sound cues, sponsor obligations, facts, claims, and listener takeaway.",
      planningArtifacts: [
        "Audience, show format, tone, and episode promise",
        "Segment rundown with timing, purpose, transitions, and host ownership",
        "Guest arc, required questions, story beats, and follow-up prompts",
        "Sponsor or promo obligations, calls to action, legal constraints, and production notes",
        "Fact/source list, sound cues, transcript needs, and publishing checklist",
      ],
      sequencingLabel: "Sequential segment drafting",
      sequencingInstruction:
        "Draft segments in order so the cold open, intro, interview or narrative blocks, sponsor reads, and outro build listener attention without repeating context or dropping promised takeaways.",
      sequenceAcceptance: [
        "The segment has a clear listener promise, purpose, time budget, and transition into the next segment.",
        "Host copy, guest prompts, claims, sound cues, and calls to action are separated for production.",
        "Facts, names, links, sponsor terms, and sensitive claims are marked for verification.",
        "The segment ending gives the next segment a specific audio handoff rather than a generic transition.",
      ],
      qualityLabel: "Audio production quality review",
      qualityInstruction:
        "Review the completed episode sequence for listener flow, timing, host voice, interview logic, sponsor compliance, fact/source readiness, transcript readiness, and production handoff clarity.",
      qualityChecks: [
        "The episode promise, segment order, timing, and listener takeaway remain coherent from cold open to outro.",
        "Host voice, guest questions, transitions, and sound cues are specific enough for recording.",
        "Sponsor reads, legal notes, claims, links, credits, and calls to action are marked for approval.",
        "Transcript, show notes, publishing checklist, and unresolved production assumptions are ready for review.",
      ],
      unitLabel: "segment",
    },
  },
  "movie-script": {
    label: "Movie script",
    defaultOutline: [
      "Screen Story Architecture",
      "Logline",
      "Characters",
      "World and Tone",
      "Beat Sheet",
      "Act I",
      "Act II",
      "Act III",
      "Key Scenes",
      "Dialogue Notes",
      "Production Constraints",
      "Screenplay Quality Review",
    ],
    questions: [
      "What genre, logline, audience, tone, and format constraints should guide the screenplay?",
      "What story architecture, act turns, scene order, character arcs, and continuity promises should be locked before screenplay pages are drafted?",
      "Who are the primary characters and what are their arcs?",
      "What major turning points, set pieces, dialogue style, and visual motifs matter?",
      "What screenplay quality criteria should be checked after beats and scenes are drafted sequentially?",
    ],
    sectionFocus: ["story architecture", "logline", "characters", "world", "beat sheet", "setup", "confrontation", "resolution", "scenes", "dialogue", "production", "screenplay QA"],
    workflow: {
      planningLabel: "Screen story architecture",
      planningInstruction:
        "Lock the screen story architecture before screenplay pages are drafted: logline, protagonist want and need, central conflict, act turns, scene order, visual rules, dialogue promises, and production constraints.",
      planningArtifacts: [
        "Logline, genre, audience, tone, format, and rating constraints",
        "Character arcs, wants, needs, conflicts, relationships, and reversals",
        "World rules, visual motifs, locations, production limits, and continuity promises",
        "Beat sheet with act turns, midpoint, low point, climax, and final image",
        "Scene order, set pieces, dialogue approach, and review constraints",
      ],
      sequencingLabel: "Sequential beat drafting",
      sequencingInstruction:
        "Draft beats in order so action, dialogue, character choice, visual motif, revelation, and production reality progress deliberately from one beat to the next.",
      sequenceAcceptance: [
        "The beat has a visual objective, conflict, turn, consequence, and handoff into the next beat.",
        "Character motivation follows from the previous beat rather than resetting.",
        "Dialogue, action, location, and production constraints are playable on screen.",
        "Continuity, tone, rating constraints, visual motifs, and unresolved questions are marked before moving on.",
      ],
      qualityLabel: "Screenplay quality review",
      qualityInstruction:
        "Review the completed beat sequence for screen story logic, visual playability, character motivation, act-turn causality, dialogue texture, pacing, continuity, tone, and production feasibility.",
      qualityChecks: [
        "The logline, act turns, stakes, reversals, climax, and final image form a coherent screen story.",
        "Every scene or beat has visible action, conflict, turn, consequence, and production-aware staging.",
        "Character choices, dialogue, tone, continuity, and visual motifs remain consistent.",
        "Locations, rating issues, effects, legal risks, budget assumptions, and unresolved notes are ready for review.",
      ],
      unitLabel: "beat",
    },
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
  ["rfp-response", /\b(rfp|request for proposal|proposal response)\b/i],
  ["rfq-response", /\b(rfq|request for quote|request for quotation|quotation response)\b/i],
  ["tender-response", /\b(tender|bid response|submission checklist)\b/i],
  ["tutorial", /\b(tutorial|training guide|walkthrough|how-to|how to)\b/i],
  ["lesson-plan", /\b(lesson plan|curriculum plan|teaching plan|class plan)\b/i],
  ["lesson-content", /\b(lesson content|learning content|student handout|learner handout|teacher notes)\b/i],
  ["technical-textbook", /\b(technical textbook|textbook chapter|technical chapter|course textbook)\b/i],
  ["novel", /\b(novel|fiction|chapter beats|story arc|characters?)\b/i],
  ["podcast-script", /\b(podcast|episode script|show notes|host script)\b/i],
  ["movie-script", /\b(movie script|screenplay|film script|logline|act i|act ii|act iii)\b/i],
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
  "outcome",
  "distribution target",
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

export function docsLiveDocumentTypeForOutlineSignal(outline: DocsLiveOutlineTypeInput): DocsLiveDocumentType {
  const explicit = outline.docsLiveType?.trim().toLowerCase();
  if (docsLiveDocumentTypes.some((type) => type.id === explicit)) return explicit as DocsLiveDocumentType;
  const signals = [outline.id, outline.name, outline.category, ...(outline.tags || []), ...(outline.bestFor || [])].filter(Boolean).join(" ");
  return normalizeDocsLiveDocumentType(signals);
}

export function buildDocsLiveQuestionnaire(documentType: string, request: DocsLiveQuestionnaireRequest = {}) {
  return docsLiveQuestionnaireQuestions(documentType, request).map((question, index) => `${index + 1}. ${question}`).join("\n");
}

export function docsLiveDefaultOutlineMarkdown(documentType: string) {
  const type = normalizeDocsLiveDocumentType(documentType);
  return blueprints[type].defaultOutline.map((section) => `- ${section}`).join("\n");
}

export function docsLiveWizardProfile(documentType: string): DocsLiveWizardProfile {
  const type = normalizeDocsLiveDocumentType(documentType);
  const blueprint = blueprints[type];
  const workflow = workflowProfileFor(blueprint);
  return {
    documentType: type,
    label: blueprint.label,
    planningLabel: workflow.planningLabel,
    sequencingLabel: workflow.sequencingLabel,
    qualityLabel: workflow.qualityLabel,
    unitLabel: workflow.unitLabel,
    defaultOutlineMarkdown: docsLiveDefaultOutlineMarkdown(type),
    decisionPrompts: docsLiveDecisionPrompts(type, blueprint),
    planningArtifacts: workflow.planningArtifacts,
    qualityChecks: workflow.qualityChecks,
  };
}

export function buildDocsLiveSuggestedAnswers(documentType: string, request: DocsLiveQuestionnaireRequest = {}): DocsLiveSuggestedAnswer[] {
  const type = normalizeDocsLiveDocumentType(documentType);
  const blueprint = blueprints[type];
  const outlineItems = parseOutlinePlan(request.outline || "");
  const contextInput = [request.context, request.transcript].filter(Boolean).join("\n");
  const placeholders = extractDocsLivePlaceholders([request.placeholders, contextInput].filter(Boolean).join("\n"));
  const contextSentences = extractContextSentences(contextInput);
  const questions = docsLiveQuestionnaireQuestions(type, request);
  return questions.map((question, index) => {
    const stepLabel = suggestedAnswerStepLabel(index, question, blueprint);
    const contextSignals = suggestedAnswerContextSignals(stepLabel, blueprint, outlineItems, placeholders, contextSentences, request);
    const answer = suggestedQuestionnaireAnswer(question, index, blueprint, outlineItems, placeholders, contextSentences, request);
    return {
      id: `${type}-${index + 1}`,
      stepLabel,
      question,
      answer,
      source: suggestedAnswerSource(placeholders, contextSentences, outlineItems),
      rationale: suggestedAnswerRationale(stepLabel, question, contextSignals, blueprint),
      contextSignals,
    };
  });
}

function docsLiveQuestionnaireQuestions(documentType: string, request: DocsLiveQuestionnaireRequest = {}) {
  const type = normalizeDocsLiveDocumentType(documentType);
  const outlineItems = parseOutlinePlan(request.outline || "");
  const placeholders = extractDocsLivePlaceholders([request.placeholders, request.context, request.transcript].filter(Boolean).join("\n"));
  const questions = [...blueprints[type].questions];
  const title = (request.title || "").trim();
  if (title) questions.unshift(`What should "${title}" help the reader decide, approve, or do?`);
  for (const section of outlineItems.slice(0, 8)) {
    questions.push(`For "${section.title}", what facts, examples, calculations, decisions, or caveats must be included?`);
  }
  const missing = ["audience", "outcome", "owner", "deadline", "distribution target", "evidence", "tone", "reviewer"].filter((key) => !placeholders[key]);
  if (missing.length) {
    questions.push(`Which ${missing.map(titleCase).join(", ")} values should Docs Live use as placeholders or review prompts?`);
  }
  questions.push("What must remain visibly marked for human review before export or publication?");
  return Array.from(new Set(questions));
}

function docsLiveDecisionPrompts(type: DocsLiveDocumentType, blueprint: DocsLiveBlueprint): DocsLiveDecisionPrompt[] {
  const common: DocsLiveDecisionPrompt[] = [
    {
      key: "audience",
      label: audienceDecisionLabel(type),
      placeholder: audienceDecisionPlaceholder(type),
      help: audienceDecisionHelp(type),
      kind: "client",
    },
    {
      key: "outcome",
      label: outcomeDecisionLabel(type),
      placeholder: outcomeDecisionPlaceholder(type),
      help: outcomeDecisionHelp(type),
      kind: "decision",
    },
    {
      key: "owner",
      label: ownerDecisionLabel(type),
      placeholder: ownerDecisionPlaceholder(type),
      help: "The accountable human reviewer or decision owner for this draft.",
      kind: "person",
    },
    {
      key: "deadline",
      label: deadlineDecisionLabel(type),
      placeholder: deadlineDecisionPlaceholder(type),
      help: "The date or milestone that should shape scope, length, and review urgency.",
      kind: "date",
    },
    {
      key: "distribution target",
      label: distributionDecisionLabel(type),
      placeholder: distributionDecisionPlaceholder(type),
      help: "Where the document needs to land so Docs Live can shape format, tone, and readiness checks.",
      kind: "channel",
    },
  ];
  const extras = extraDecisionPrompts(type);
  return [...common, ...extras].slice(0, blueprint.workflow ? 8 : 7);
}

function extraDecisionPrompts(type: DocsLiveDocumentType): DocsLiveDecisionPrompt[] {
  const byType: Partial<Record<DocsLiveDocumentType, DocsLiveDecisionPrompt[]>> = {
    novel: [
      {
        key: "genre promise",
        label: "Genre and shelf",
        placeholder: "adult speculative mystery with literary pacing",
        help: "The commercial and creative shelf promise: what kind of reader should pick this up and why.",
        kind: "decision",
      },
      {
        key: "point of view",
        label: "Point of view and tense",
        placeholder: "close third, past tense, rotating two protagonists",
        help: "A structural choice that controls voice, scene access, and continuity before prose is drafted.",
        kind: "text",
      },
      {
        key: "protagonist arc",
        label: "Protagonist arc",
        placeholder: "from risk-averse analyst to public whistleblower",
        help: "The emotional business decision of the story: what change must the reader believe.",
        kind: "decision",
      },
    ],
    "technical-textbook": [
      {
        key: "reader level",
        label: "Reader level",
        placeholder: "senior undergraduate, professional engineer, executive primer",
        help: "The learning level determines assumptions, examples, equations, and pace.",
        kind: "decision",
      },
      {
        key: "assessment model",
        label: "Assessment model",
        placeholder: "worked examples plus chapter exercises and capstone",
        help: "The instructional decision that shapes chapter sequencing and quality review.",
        kind: "decision",
      },
    ],
    "podcast-script": [
      {
        key: "listener promise",
        label: "Listener promise",
        placeholder: "one practical takeaway for operations leaders in 20 minutes",
        help: "The audio product decision that controls pacing, segment order, and handoff.",
        kind: "decision",
      },
      {
        key: "show format",
        label: "Show format",
        placeholder: "host monologue, interview, narrative documentary",
        help: "The production choice that controls voice, roles, cues, and timing.",
        kind: "channel",
      },
    ],
    "movie-script": [
      {
        key: "logline",
        label: "Logline",
        placeholder: "A climate analyst must expose a rigged forecast before the auction closes.",
        help: "The screen story decision that governs every beat, reversal, and scene test.",
        kind: "decision",
      },
      {
        key: "production constraint",
        label: "Production constraint",
        placeholder: "limited locations, no VFX, ensemble cast of five",
        help: "The business constraint that keeps the screenplay draft producible.",
        kind: "text",
      },
    ],
    "rfp-response": [
      {
        key: "win theme",
        label: "Win theme",
        placeholder: "lowest-risk compliant delivery with proven team and references",
        help: "The strategic bid decision that every outline section should reinforce.",
        kind: "decision",
      },
      {
        key: "mandatory gates",
        label: "Mandatory gates",
        placeholder: "signed annexes, page limit, roles, deadline, format",
        help: "Pass/fail submission constraints that must be surfaced before drafting.",
        kind: "source",
      },
    ],
    proposal: [
      {
        key: "commercial model",
        label: "Commercial model",
        placeholder: "fixed fee, milestone billing, retainer, pilot-to-scale",
        help: "The business decision that shapes scope, pricing, risk, and acceptance.",
        kind: "money",
      },
    ],
  };
  return byType[type] || [];
}

function audienceDecisionLabel(type: DocsLiveDocumentType) {
  return type === "novel" ? "Target reader" : type === "podcast-script" ? "Target listener" : type === "movie-script" ? "Target viewer" : "Audience";
}

function audienceDecisionPlaceholder(type: DocsLiveDocumentType) {
  if (type === "novel") return "adult speculative fiction readers, YA fantasy, cozy mystery fans";
  if (type === "podcast-script") return "CFOs, founders, field teams, teachers";
  if (type === "movie-script") return "festival audience, streaming thriller viewers, family audience";
  return "executive team, board, customers, learners";
}

function audienceDecisionHelp(type: DocsLiveDocumentType) {
  if (type === "novel") return "The reader promise and market shelf, not a corporate recipient.";
  if (type === "podcast-script") return "The listener profile determines pacing, vocabulary, and audio handoffs.";
  if (type === "movie-script") return "The intended viewer controls tone, rating, scene density, and payoff.";
  return "The decision-maker or reader whose needs the document must satisfy.";
}

function outcomeDecisionLabel(type: DocsLiveDocumentType) {
  if (type === "novel") return "Creative promise";
  if (type === "movie-script") return "Story payoff";
  if (type === "podcast-script") return "Listener takeaway";
  if (type === "technical-textbook" || type === "lesson-plan" || type === "lesson-content") return "Learning outcome";
  return "Business decision";
}

function outcomeDecisionPlaceholder(type: DocsLiveDocumentType) {
  if (type === "novel") return "reader feels the cost of truth and wants book two";
  if (type === "movie-script") return "audience understands the protagonist's irreversible choice";
  if (type === "podcast-script") return "listener can make one better operational decision";
  if (type === "technical-textbook") return "reader can implement and evaluate the method independently";
  return "approve renewal, choose vendor, fund pilot, align launch plan";
}

function outcomeDecisionHelp(type: DocsLiveDocumentType) {
  if (type === "novel") return "The creative and business promise the story must satisfy for its target reader.";
  if (type === "movie-script") return "The final audience effect the beat sheet and scenes must earn.";
  if (type === "podcast-script") return "The concrete listener value the episode must deliver.";
  if (type === "technical-textbook" || type === "lesson-plan" || type === "lesson-content") return "The measurable learning result the content must support.";
  return "The business choice, approval, or action the document should make easier.";
}

function ownerDecisionLabel(type: DocsLiveDocumentType) {
  return type === "novel" || type === "movie-script" ? "Creative owner" : "Owner";
}

function ownerDecisionPlaceholder(type: DocsLiveDocumentType) {
  return type === "novel" || type === "movie-script" ? "lead author, editor, showrunner" : "Finance, Product, Legal, proposal lead";
}

function deadlineDecisionLabel(type: DocsLiveDocumentType) {
  return type === "novel" || type === "movie-script" ? "Milestone" : "Deadline";
}

function deadlineDecisionPlaceholder(type: DocsLiveDocumentType) {
  return type === "novel" ? "pitch package by June 1, first chapter review Friday" : "June 1, end of Q2, submission date";
}

function distributionDecisionLabel(type: DocsLiveDocumentType) {
  return type === "novel" ? "Publishing path" : type === "movie-script" ? "Submission path" : "Distribution target";
}

function distributionDecisionPlaceholder(type: DocsLiveDocumentType) {
  if (type === "novel") return "query package, serialized web novel, editor review, self-publish";
  if (type === "movie-script") return "pitch deck, screenplay PDF, producers, table read";
  if (type === "podcast-script") return "recording script, show notes, transcript, Substack";
  return "PDF, Google Docs, Substack, board pack, tender portal";
}

export function buildDocsLiveDraft(request: DocsLiveDraftRequest): DocsLiveDraft {
  const documentType = inferDocumentType(request);
  const blueprint = blueprints[documentType];
  const contextInput = [request.transcript, request.context, request.questionnaireAnswers].filter(Boolean).join("\n");
  const placeholderInput = [request.placeholders, request.context, request.questionnaireAnswers, request.transcript].filter(Boolean).join("\n");
  const placeholders = extractDocsLivePlaceholders(placeholderInput);
  const placeholderEntries = hydrateDocsLivePlaceholderEntries(placeholders, docsLivePlaceholderEntries(placeholderInput));
  const title = resolveTitle(request, blueprint, placeholders);
  const requestedOutlineText = (request.outline || "").trim();
  const requestedOutlineItems = parseOutlinePlan(requestedOutlineText);
  const outlineWasProvided = requestedOutlineItems.length > 0;
  const outlineText = outlineWasProvided ? requestedOutlineText : resolveOutlineText(request, blueprint);
  const outlineItems = outlineWasProvided ? requestedOutlineItems : parseOutlinePlan(outlineText);
  const sections = outlineItems.length ? outlineItems : blueprint.defaultOutline.map((section) => ({ level: 1, title: section }));
  const generatedAt = request.generatedAt || new Date().toISOString();
  const contextSentences = extractContextSentences(contextInput);
  const issues = buildDraftIssues(request, placeholders, outlineWasProvided);
  const draftingDepth = normalizeDraftingDepth(request.draftingDepth);
  const planningOnly = Boolean(blueprint.workflow && !outlineWasProvided);
  const sectionDrafts = sections.map((section, index) => buildSectionDraft(section, index, blueprint, placeholders, contextSentences, planningOnly));
  const workflow = buildDocsLiveWorkflow(sectionDrafts, placeholders, contextSentences, issues, blueprint, outlineWasProvided);
  const reviewPacket = buildDocsLiveReviewPacket(request, sectionDrafts, placeholders, contextSentences, issues, blueprint, outlineWasProvided);
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
      placeholdersTable(placeholderEntries),
      "",
      docsLiveContextSummary(contextSentences),
      "",
      docsLiveReviewMarker("Docs Live systematic outline-to-draft workflow"),
      "",
      longFormPlanningGateMarkdown(blueprint, sectionDrafts),
      "",
      sequentialDraftQueueMarkdown(blueprint, sectionDrafts),
      "",
      draftingPlanTable(workflow, sectionDrafts, draftingDepth, blueprint),
      "",
      reviewPacketMarkdown(reviewPacket, blueprint),
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
      ...qualityChecklist(sections, placeholders, blueprint, outlineWasProvided).map((item) => `- [ ] ${item}`),
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
  const placeholders = Object.fromEntries(docsLivePlaceholderEntries(input).map((entry) => [entry.key, entry.value]));
  for (const key of placeholderSignals) {
    const signal = new RegExp(`\\b${key}\\s+(?:is|are|=|:)\\s+([^.;\\n]+)`, "i");
    const match = input.match(signal);
    if (match && !placeholders[normalizePlaceholderKey(key)]) {
      placeholders[normalizePlaceholderKey(key)] = match[1].trim();
    }
  }
  return placeholders;
}

export function docsLivePlaceholderEntries(input: string): DocsLivePlaceholderEntry[] {
  const entries: DocsLivePlaceholderEntry[] = [];
  const seen = new Set<string>();
  for (const line of input.split(/\r?\n/)) {
    const pair = line.match(/^\s*([A-Za-z][A-Za-z0-9 _-]{1,36})\s*[:=]\s*(.+?)\s*$/);
    if (!pair) continue;
    const key = normalizePlaceholderKey(pair[1]);
    if (!key || seen.has(key)) continue;
    seen.add(key);
    const { value, kind, source, reviewStatus } = parsePlaceholderValueMetadata(pair[2]);
    if (!value) continue;
    entries.push({ key, value, kind, source, reviewStatus });
  }
  return entries;
}

export function serializeDocsLivePlaceholders(entries: DocsLivePlaceholderEntry[]) {
  return entries
    .map((entry) => ({
      key: normalizePlaceholderKey(entry.key),
      value: entry.value.trim(),
      kind: normalizePlaceholderKind(entry.kind),
      source: entry.source.trim(),
      reviewStatus: normalizePlaceholderReviewStatus(entry.reviewStatus),
    }))
    .filter((entry) => entry.key && entry.value)
    .map((entry) => {
      const metadata = [
        entry.kind !== "text" ? `type=${entry.kind}` : "",
        entry.source ? `source=${entry.source}` : "",
        entry.reviewStatus !== "provided" ? `status=${entry.reviewStatus}` : "",
      ].filter(Boolean);
      return `${entry.key}: ${entry.value}${metadata.length ? ` | ${metadata.join(" | ")}` : ""}`;
    })
    .join("\n");
}

export function upsertDocsLivePlaceholder(
  input: string,
  key: string,
  value: string,
  metadata: Partial<Pick<DocsLivePlaceholderEntry, "kind" | "source" | "reviewStatus">> = {},
) {
  const normalizedKey = normalizePlaceholderKey(key);
  if (!normalizedKey || !value.trim()) return input.trim();
  const entries = docsLivePlaceholderEntries(input).filter((entry) => entry.key !== normalizedKey);
  entries.push({
    key: normalizedKey,
    value: value.trim(),
    kind: normalizePlaceholderKind(metadata.kind),
    source: (metadata.source || "").trim(),
    reviewStatus: normalizePlaceholderReviewStatus(metadata.reviewStatus),
  });
  return serializeDocsLivePlaceholders(entries);
}

export function removeDocsLivePlaceholder(input: string, key: string) {
  const normalizedKey = normalizePlaceholderKey(key);
  return serializeDocsLivePlaceholders(docsLivePlaceholderEntries(input).filter((entry) => entry.key !== normalizedKey));
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

function buildDraftIssues(request: DocsLiveDraftRequest, placeholders: Record<string, string>, outlineWasProvided: boolean) {
  const issues: string[] = [];
  if (!Object.keys(placeholders).length) issues.push("No placeholder values were detected; draft includes review prompts for missing specifics.");
  if (!outlineWasProvided) issues.push("No usable outline was supplied; Docs Live generated a document-type outline that must be reviewed before drafting is accepted.");
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
  blueprint: DocsLiveBlueprint,
  outlineWasProvided: boolean,
): DocsLiveWorkflowStep[] {
  const workflow = workflowProfileFor(blueprint);
  const planningOnly = Boolean(blueprint.workflow && !outlineWasProvided);
  const outlineStatus = sections.length ? (outlineWasProvided ? "complete" : "ready") : "needs-input";
  const outlineDetail = outlineWasProvided
    ? blueprint.workflow
      ? `${workflow.planningInstruction} ${sections.length} planned ${workflow.unitLabel}${sections.length === 1 ? "" : "s"} ready for systematic drafting.`
      : `${sections.length} planned section${sections.length === 1 ? "" : "s"} ready for systematic drafting.`
    : blueprint.workflow
      ? `Docs Live generated ${sections.length} suggested ${workflow.unitLabel}${sections.length === 1 ? "" : "s"} from the document type. Review and approve the ${workflow.planningLabel.toLowerCase()} before prose is accepted.`
      : `Docs Live generated ${sections.length} suggested outline section${sections.length === 1 ? "" : "s"} from the document type. Review and approve the outline before accepting the draft.`;
  const steps: Array<Omit<DocsLiveWorkflowStep, "assistance" | "contextSignals">> = [
    {
      id: "outline",
      label: outlineWasProvided ? (blueprint.workflow ? `${workflow.planningLabel} locked` : "Outline locked") : "Suggested outline ready",
      status: outlineStatus,
      detail: outlineDetail,
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
      label: workflow.sequencingLabel,
      status: planningOnly ? "needs-input" : blueprint.workflow || !outlineWasProvided ? "ready" : "complete",
      detail: planningOnly
        ? `${workflow.planningLabel} is not approved yet; Docs Live is holding prose expansion until the outline or plot is locked.`
        : !outlineWasProvided
        ? "Draft text is a scaffold until the suggested outline is reviewed and accepted."
        : blueprint.workflow
        ? `${workflow.sequencingInstruction} Start only after the ${workflow.planningLabel.toLowerCase()} approval gate is checked.`
        : "Each outline item receives a body draft, local evidence prompts, and a review handoff.",
    },
    {
      id: "qa",
      label: workflow.qualityLabel,
      status: issues.length ? "needs-input" : "complete",
      detail: issues.length
        ? `${issues.length} item${issues.length === 1 ? "" : "s"} need attention before review. ${workflow.qualityInstruction}`
        : workflow.qualityInstruction,
    },
    {
      id: "humanize",
      label: "Humanization pass",
      status: planningOnly ? "needs-input" : "complete",
      detail: planningOnly
        ? "Humanization waits until chapter prose exists; the current artifact is an outline or plot plan."
        : "Draft text is stripped of common AI phrasing and marked for human review.",
    },
    {
      id: "review",
      label: "Review handoff",
      status: planningOnly ? "needs-input" : "complete",
      detail: planningOnly
        ? "Review should approve the architecture before accepting any generated prose."
        : "Each section carries reviewer questions, unresolved assumptions, and sign-off prompts.",
    },
  ];
  return steps.map((step) => ({
    ...step,
    assistance: docsLiveWorkflowStepAssistance(step.id, {
      sections,
      placeholders,
      contextSentences,
      issues,
      blueprint,
      outlineWasProvided,
      planningOnly,
    }),
    contextSignals: docsLiveWorkflowStepSignals(step.id, {
      sections,
      placeholders,
      contextSentences,
      issues,
      blueprint,
      outlineWasProvided,
      planningOnly,
    }),
  }));
}

function docsLiveWorkflowStepAssistance(
  stepId: DocsLiveWorkflowStep["id"],
  context: {
    sections: DocsLiveSectionDraft[];
    placeholders: Record<string, string>;
    contextSentences: string[];
    issues: string[];
    blueprint: DocsLiveBlueprint;
    outlineWasProvided: boolean;
    planningOnly: boolean;
  },
) {
  const workflow = workflowProfileFor(context.blueprint);
  const placeholderKeys = Object.keys(context.placeholders);
  const firstUnit = firstDraftableWorkflowUnit(workflow, context.sections);
  switch (stepId) {
    case "outline":
      return context.outlineWasProvided
        ? `Treat the current outline as the approved work queue; verify ${firstUnit?.title || `the first ${workflow.unitLabel}`} has a clear purpose, owner, evidence need, and acceptance check before drafting.`
        : `Review the AI-generated ${workflow.planningLabel.toLowerCase()}, rename weak sections, add missing decision/evidence steps, and explicitly approve it before prose generation.`;
    case "context":
      return placeholderKeys.length
        ? `Promote the strongest context into reusable placeholders for ${placeholderKeys.slice(0, 4).join(", ")} and add any missing audience, outcome, owner, deadline, evidence, and reviewer values.`
        : "Start with audience, outcome, owner, deadline, tone, evidence, and reviewer; keep unknown facts bracketed so the draft does not invent them.";
    case "draft":
      return context.planningOnly
        ? `Do not flesh out prose yet; approve the ${workflow.planningLabel.toLowerCase()}, choose the first ${workflow.unitLabel}, and write section-specific completion criteria first.`
        : `Draft ${firstUnit?.title || `the next ${workflow.unitLabel}`} first, carry forward only verified context, and leave unresolved facts as visible review prompts.`;
    case "qa":
      return context.issues.length
        ? `Resolve the ${context.issues.length} open QA blocker${context.issues.length === 1 ? "" : "s"} before export, then run ${workflow.qualityLabel.toLowerCase()} against claims, calculations, citations, and assumptions.`
        : `Run ${workflow.qualityLabel.toLowerCase()} now: check claims, calculations, citations, continuity, section purpose, and reviewer acceptance criteria.`;
    case "humanize":
      return context.planningOnly
        ? "Wait for drafted prose, then replace generic AI phrasing with concrete owner language, examples, dates, source references, and natural transitions."
        : "Read each section as the intended owner, cut generic setup phrases, add concrete examples, and make unresolved assumptions sound like review notes rather than filler.";
    case "review":
      return "Assign an accountable reviewer, keep AI-assisted provenance visible, collect approvals and unresolved evidence, then package the document for the selected export or publishing path.";
    default:
      return "Use the current document context to choose the next smallest reviewable action before generating or applying more text.";
  }
}

function docsLiveWorkflowStepSignals(
  stepId: DocsLiveWorkflowStep["id"],
  context: {
    sections: DocsLiveSectionDraft[];
    placeholders: Record<string, string>;
    contextSentences: string[];
    issues: string[];
    blueprint: DocsLiveBlueprint;
    outlineWasProvided: boolean;
    planningOnly: boolean;
  },
) {
  const workflow = workflowProfileFor(context.blueprint);
  const signals = [
    context.outlineWasProvided
      ? `${context.sections.length} supplied outline section${context.sections.length === 1 ? "" : "s"}`
      : `${context.sections.length} suggested ${workflow.unitLabel}${context.sections.length === 1 ? "" : "s"}`,
    `${Object.keys(context.placeholders).length} placeholder value${Object.keys(context.placeholders).length === 1 ? "" : "s"}`,
    `${context.contextSentences.length} extracted context point${context.contextSentences.length === 1 ? "" : "s"}`,
    `${context.issues.length} QA issue${context.issues.length === 1 ? "" : "s"}`,
  ];
  if (context.planningOnly) signals.push(`${workflow.planningLabel} approval pending`);
  signals.push(`Current stage: ${stepId}`);
  return signals;
}

function buildDocsLiveReviewPacket(
  request: DocsLiveDraftRequest,
  sections: DocsLiveSectionDraft[],
  placeholders: Record<string, string>,
  contextSentences: string[],
  issues: string[],
  blueprint: DocsLiveBlueprint,
  outlineWasProvided: boolean,
): DocsLiveReviewPacket {
  const placeholderCount = Object.keys(placeholders).length;
  const workflow = workflowProfileFor(blueprint);
  const outlineContextSource = !outlineWasProvided
    ? blueprint.workflow
      ? `Docs Live generated a suggested ${workflow.planningLabel.toLowerCase()} from the document type; approve it before accepting prose.`
      : "Docs Live generated a suggested document-type outline; approve it before accepting prose."
    : blueprint.workflow
      ? `${workflow.planningInstruction} ${sections.length} planned ${workflow.unitLabel}${sections.length === 1 ? "" : "s"} locked before drafting.`
      : `${sections.length} outline section${sections.length === 1 ? "" : "s"} locked before drafting.`;
  const contextSources = [
    outlineContextSource,
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
      blueprint.workflow
        ? section.planningOnly
          ? `${index + 1}. ${section.title}: hold prose drafting; refine the ${workflow.planningLabel.toLowerCase()}, acceptance criteria, continuity handoff, and review evidence first.`
          : `${index + 1}. ${section.title}: draft this ${workflow.unitLabel} in sequence only after the ${workflow.planningLabel.toLowerCase()} is locked, run ${workflow.qualityLabel.toLowerCase()} against ${section.qaFocus}, humanize the prose, then hand to reviewer.`
        : `${index + 1}. ${section.title}: draft body, run QA against ${section.qaFocus}, humanize the prose, then hand to reviewer.`,
  );
  const qaRegister = [
    ...issues,
    ...(blueprint.workflow ? [workflow.qualityInstruction] : []),
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

function draftingPlanTable(
  workflow: DocsLiveWorkflowStep[],
  sections: DocsLiveSectionDraft[],
  draftingDepth: DocsLiveDraftDepth,
  blueprint: DocsLiveBlueprint,
) {
  const profile = workflowProfileFor(blueprint);
  return [
    "## Drafting Plan",
    "",
    blueprint.workflow
      ? `Docs Live first locks the ${profile.planningLabel.toLowerCase()} before prose is drafted, drafts ${profile.unitLabel}s in order at ${draftingDepth} depth, then runs ${profile.qualityLabel.toLowerCase()} before review handoff.`
      : `Docs Live will work through the outline section by section at ${draftingDepth} depth, then attach QA, humanization, and review handoff notes.`,
    "",
    "| Stage | Status | Detail | AI assistance |",
    "| --- | --- | --- | --- |",
    ...workflow.map((step) => `| ${escapeTableCell(step.label)} | ${escapeTableCell(step.status)} | ${escapeTableCell(step.detail)} | ${escapeTableCell(step.assistance)} |`),
    "",
    "| Section | Drafting brief | QA focus |",
    "| --- | --- | --- |",
    ...sections.map((section) => `| ${escapeTableCell(section.title)} | ${escapeTableCell(section.draftingBrief)} | ${escapeTableCell(section.qaFocus)} |`),
  ].join("\n");
}

function longFormPlanningGateMarkdown(blueprint: DocsLiveBlueprint, sections: DocsLiveSectionDraft[]) {
  if (!blueprint.workflow) return "";
  const profile = workflowProfileFor(blueprint);
  const firstDraftUnit = firstDraftableWorkflowUnit(profile, sections);
  return [
    `## ${titleCase(profile.planningLabel)} Approval Gate`,
    "",
    `${profile.planningInstruction} Treat this as the first work product: the wizard should refine this plan before it fleshes out ${profile.unitLabel}s.`,
    "",
    "| Required planning artifact | Status | Review question |",
    "| --- | --- | --- |",
    ...profile.planningArtifacts.map((artifact) =>
      `| ${escapeTableCell(artifact)} | needs-review | Has this been specific enough to guide every ${profile.unitLabel}? |`,
    ),
    "",
    `- [ ] Approve ${profile.planningLabel.toLowerCase()} before drafting ${firstDraftUnit ? firstDraftUnit.title : `the first ${profile.unitLabel}`}.`,
    `- [ ] Freeze the ${profile.unitLabel} order or record why it changed before prose is expanded.`,
    "- [ ] Capture open questions as review comments instead of hiding them in polished prose.",
  ].join("\n");
}

function sequentialDraftQueueMarkdown(blueprint: DocsLiveBlueprint, sections: DocsLiveSectionDraft[]) {
  if (!blueprint.workflow) return "";
  const profile = workflowProfileFor(blueprint);
  const draftableSections = draftableWorkflowSections(profile, sections);
  return [
    `## Sequential ${titleCase(profile.unitLabel)} Draft Queue`,
    "",
    `${profile.sequencingInstruction} Each ${profile.unitLabel} is accepted before the next one is fleshed out.`,
    "",
    "| Order | Unit | Expansion rule | Acceptance criteria |",
    "| --- | --- | --- | --- |",
    ...draftableSections.map((section, index) =>
      `| ${index + 1} | ${escapeTableCell(section.title)} | ${escapeTableCell(section.draftingBrief)} | ${escapeTableCell(profile.sequenceAcceptance[index % profile.sequenceAcceptance.length])} |`,
    ),
    "",
    `## Final ${titleCase(profile.qualityLabel)}`,
    "",
    `${profile.qualityInstruction} Run this after the last ${profile.unitLabel} is drafted, not as a substitute for per-${profile.unitLabel} checks.`,
    "",
    ...profile.qualityChecks.map((check) => `- [ ] ${check}`),
  ].join("\n");
}

function firstDraftableWorkflowUnit(profile: DocsLiveWorkflowProfile, sections: DocsLiveSectionDraft[]) {
  const draftable = draftableWorkflowSections(profile, sections);
  return draftable.find((section) => /^chapter\s+\d+/i.test(section.title))
    || draftable.find((section) => /\b(cold open|intro|segment|act|scene|beat)\b/i.test(section.title))
    || draftable[0]
    || sections[0];
}

function draftableWorkflowSections(profile: DocsLiveWorkflowProfile, sections: DocsLiveSectionDraft[]) {
  const planningLabel = profile.planningLabel.toLowerCase();
  const qualityLabel = profile.qualityLabel.toLowerCase();
  return sections.filter((section) => {
    const title = section.title.toLowerCase();
    return title !== planningLabel
      && title !== qualityLabel
      && title !== "segment rundown"
      && title !== "beat sheet"
      && !title.endsWith("quality review")
      && !title.endsWith("production review");
  });
}

function reviewPacketMarkdown(packet: DocsLiveReviewPacket, blueprint: DocsLiveBlueprint) {
  const profile = workflowProfileFor(blueprint);
  return [
    "## Section-by-section Draft Runbook",
    "",
    blueprint.workflow
      ? `Docs Live uses the ${profile.planningLabel.toLowerCase()} as the work queue. The outline or plot is settled first; each ${profile.unitLabel} is then drafted sequentially, checked, humanized, and packaged for review before ${profile.qualityLabel.toLowerCase()}.`
      : "Docs Live uses the outline as the work queue. Each section is drafted, checked, humanized, and packaged for review before the next approval step.",
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

export function buildDocsLiveReviewPacketMarkdown(
  draft: DocsLiveDraft,
  options: DocsLiveReviewPacketMarkdownOptions,
) {
  const packet = draft.reviewPacket;
  const source = options.source || "NEditor Docs Live";
  const lines = [
    "## Docs Live Review Packet",
    "",
    "```ai-audit",
    "type: docs-live-review-packet",
    `generatedAt: ${options.generatedAt}`,
    `title: ${docsLiveAuditInline(draft.title)}`,
    `documentType: ${docsLiveAuditInline(draft.documentType)}`,
    `sections: ${draft.sections.length}`,
    `source: ${docsLiveAuditInline(source)}`,
    "```",
    "",
    "### Context Package",
    "",
    ...packet.contextSources.map((source) => `- ${docsLiveAuditInline(source)}`),
    "",
    "### Section Work Queue",
    "",
    ...packet.sectionRunbook.map((item) => `- ${docsLiveAuditInline(item)}`),
    "",
    "### Assumption Register",
    "",
    ...packet.qaRegister.map((item) => `- [ ] ${docsLiveAuditInline(item)}`),
    "",
    "### Humanization Checklist",
    "",
    ...packet.humanizationChecklist.map((item) => `- [ ] ${docsLiveAuditInline(item)}`),
    "",
    "### Reviewer Handoff",
    "",
    ...packet.reviewerHandoff.map((item) => `- [ ] ${docsLiveAuditInline(item)}`),
  ];
  return lines.join("\n");
}

export function docsLiveAuditInline(value: string) {
  return (value || "").replace(/\r?\n/g, " ").trim();
}

function suggestedAnswerStepLabel(index: number, question: string, blueprint: DocsLiveBlueprint) {
  const lower = question.toLowerCase();
  const workflow = workflowProfileFor(blueprint);
  if (lower.includes("placeholder")) return "Placeholder values";
  if (lower.includes("human review") || lower.includes("export") || lower.includes("publication")) return "Review and distribution";
  if (lower.includes("outline") || lower.includes("architecture") || lower.includes("plot") || lower.includes("rundown") || lower.includes("beat sheet")) {
    return workflow.planningLabel;
  }
  if (lower.startsWith("for \"") || lower.includes("section") || lower.includes("chapter") || lower.includes("segment") || lower.includes("beat")) return workflow.sequencingLabel;
  if (lower.includes("quality") || lower.includes("checked")) return workflow.qualityLabel;
  return index === 0 ? "Intent and outcome" : "Context capture";
}

function suggestedAnswerSource(placeholders: Record<string, string>, contextSentences: string[], outlineItems: OutlinePlanItem[]) {
  const sources = [];
  if (Object.keys(placeholders).length) sources.push(`${Object.keys(placeholders).length} placeholder value${Object.keys(placeholders).length === 1 ? "" : "s"}`);
  if (contextSentences.length) sources.push(`${contextSentences.length} context note${contextSentences.length === 1 ? "" : "s"}`);
  if (outlineItems.length) sources.push(`${outlineItems.length} outline item${outlineItems.length === 1 ? "" : "s"}`);
  return sources.length ? `Suggested from ${sources.join(", ")}.` : "Suggested from the selected document type; replace bracketed values before drafting.";
}

function suggestedAnswerContextSignals(
  stepLabel: string,
  blueprint: DocsLiveBlueprint,
  outlineItems: OutlinePlanItem[],
  placeholders: Record<string, string>,
  contextSentences: string[],
  request: DocsLiveQuestionnaireRequest,
) {
  const signals = [`document type: ${blueprint.label}`, `step: ${stepLabel}`];
  if ((request.title || "").trim()) signals.push(`title: ${(request.title || "").trim()}`);
  if (outlineItems.length) signals.push(`outline items: ${outlineItems.length}`);
  if (Object.keys(placeholders).length) signals.push(`placeholder values: ${Object.keys(placeholders).length}`);
  if (contextSentences.length) signals.push(`context notes: ${contextSentences.length}`);
  if ((request.transcript || "").trim()) signals.push("voice/freeform transcript present");
  const namedSignals = ["client", "audience", "outcome", "owner", "deadline", "distribution target", "evidence", "tone", "reviewer"].filter((key) => placeholders[key]);
  if (namedSignals.length) signals.push(`known fields: ${namedSignals.join(", ")}`);
  return Array.from(new Set(signals));
}

function suggestedAnswerRationale(stepLabel: string, question: string, contextSignals: string[], blueprint: DocsLiveBlueprint) {
  const lower = question.toLowerCase();
  if (lower.includes("placeholder")) return "Fills repeated document facts first so later drafting steps can reuse names, dates, owners, evidence, and distribution targets consistently.";
  if (lower.includes("human review") || lower.includes("export") || lower.includes("publication")) return "Keeps generated material gated before export by naming what needs evidence, review, approval, or publication metadata.";
  if (lower.includes("outline") || lower.includes("architecture") || lower.includes("plot") || lower.includes("rundown") || lower.includes("beat sheet")) {
    return `Locks the ${blueprint.label.toLowerCase()} structure before prose so each later section can be drafted and reviewed sequentially.`;
  }
  if (lower.startsWith("for \"") || lower.includes("facts") || lower.includes("examples") || lower.includes("calculations") || lower.includes("caveats")) {
    return "Turns the selected outline item into a scoped drafting brief with facts, caveats, evidence, and reviewer ownership visible.";
  }
  if (lower.includes("quality") || lower.includes("checked")) return "Converts quality expectations into explicit review gates instead of hiding weak assumptions inside polished prose.";
  if (contextSignals.some((signal) => signal.startsWith("placeholder values:")) || contextSignals.some((signal) => signal.startsWith("context notes:"))) {
    return "Uses supplied context and placeholders to create a reviewable starting answer while leaving uncertain facts editable.";
  }
  return `Uses the ${blueprint.label.toLowerCase()} blueprint as a conservative starting point for the ${stepLabel.toLowerCase()} step and marks replaceable assumptions for review.`;
}

function suggestedQuestionnaireAnswer(
  question: string,
  index: number,
  blueprint: DocsLiveBlueprint,
  outlineItems: OutlinePlanItem[],
  placeholders: Record<string, string>,
  contextSentences: string[],
  request: DocsLiveQuestionnaireRequest,
) {
  const lower = question.toLowerCase();
  const workflow = workflowProfileFor(blueprint);
  const title = (request.title || placeholders.title || blueprint.label).trim();
  const audience = placeholders.audience || placeholders.reader || placeholders.customer || placeholders.client || `[primary ${blueprint.label.toLowerCase()} audience]`;
  const outcome = placeholders.outcome || placeholders.goal || placeholders.decision || suggestedOutcomeFor(blueprint);
  const owner = placeholders.owner || placeholders.reviewer || placeholders.author || "[named owner]";
  const deadline = placeholders.deadline || placeholders.date || "[review date]";
  const distribution = placeholders["distribution target"] || placeholders.channel || placeholders.target || "[delivery channel]";
  const evidence = placeholders.evidence || placeholders.source || contextSentences[0] || "[source evidence]";
  const tone = placeholders.tone || placeholders.voice || "[desired tone]";
  const outlineSummary = outlineItems.length ? outlineItems.slice(0, 6).map((item) => item.title).join(" -> ") : blueprint.defaultOutline.slice(0, 5).join(" -> ");
  const context = contextSentences[index % Math.max(1, contextSentences.length)] || `${blueprint.label} should be specific, reviewable, and ready for human approval.`;

  if (lower.includes("what should") && lower.includes("help the reader")) {
    return `${title} should help ${audience} ${outcome}. Success means the reader can see the recommendation, the evidence, the owner, the deadline, and the open review items without guessing.`;
  }
  if (lower.includes("placeholder")) {
    return [
      `audience: ${audience}`,
      `outcome: ${outcome}`,
      `owner: ${owner}`,
      `deadline: ${deadline}`,
      `distribution target: ${distribution}`,
      `evidence: ${evidence}`,
      `tone: ${tone}`,
      `reviewer: ${placeholders.reviewer || owner}`,
    ].join("\n");
  }
  if (lower.includes("human review") || lower.includes("export") || lower.includes("publication")) {
    return `Keep these visibly marked until review: unverified facts and claims, missing citations or source files, unresolved placeholders, legal/commercial approvals, export metadata for ${distribution}, and any section where ${owner} has not signed off.`;
  }
  if (lower.includes("outline") || lower.includes("architecture") || lower.includes("plot") || lower.includes("rundown") || lower.includes("beat sheet")) {
    return `${workflow.planningLabel} should lock: ${workflow.planningArtifacts.join("; ")}. Proposed order: ${outlineSummary}. Do not draft prose until ${owner} approves this structure.`;
  }
  if (lower.includes("quality") || lower.includes("checked")) {
    return `${workflow.qualityLabel} should check ${workflow.qualityChecks.join("; ")}. Any failure stays as a review task instead of being hidden in polished prose.`;
  }
  if (lower.includes("facts") || lower.includes("examples") || lower.includes("calculations") || lower.includes("caveats")) {
    const section = question.match(/For "(.+?)"/)?.[1] || outlineItems[index % Math.max(1, outlineItems.length)]?.title || "this section";
    return `${section} should use ${evidence}, name ${owner} as the accountable reviewer, preserve caveats explicitly, and connect to the next ${workflow.unitLabel}. Context to reflect: ${context}`;
  }
  if (lower.includes("client") || lower.includes("sponsor")) {
    return `Client or sponsor: ${placeholders.client || placeholders.sponsor || audience}. Their primary need is ${outcome}, and the draft should make ownership, timing, and evidence easy to verify.`;
  }
  if (lower.includes("audience") || lower.includes("reader") || lower.includes("learner") || lower.includes("listener")) {
    return `Audience: ${audience}. Assume they need ${outcome}, prefer ${tone}, and will trust the draft only when claims are tied to ${evidence}.`;
  }
  if (lower.includes("owner") || lower.includes("responsible")) {
    return `Owner: ${owner}. They should approve the ${workflow.planningLabel.toLowerCase()}, review every ${workflow.unitLabel} before the next one is drafted, and sign off before ${distribution}.`;
  }
  return `${blueprint.label} context: ${context} Use ${workflow.planningLabel.toLowerCase()} first, draft each ${workflow.unitLabel} in sequence, run ${workflow.qualityLabel.toLowerCase()}, humanize the result, and leave unresolved items visible for ${owner}.`;
}

function suggestedOutcomeFor(blueprint: DocsLiveBlueprint) {
  const label = blueprint.label.toLowerCase();
  if (label.includes("proposal")) return "approve the recommended approach";
  if (label.includes("rfp") || label.includes("tender") || label.includes("rfq")) return "evaluate a compliant response";
  if (label.includes("textbook") || label.includes("lesson") || label.includes("tutorial")) return "learn the material and know what to do next";
  if (label.includes("novel")) return "understand the story promise, plot, characters, and next drafting step";
  if (label.includes("podcast")) return "record a coherent episode with clear listener value";
  if (label.includes("movie")) return "evaluate and draft a playable screen story";
  return "make the intended decision or take the next action";
}

function parsePlaceholderValueMetadata(raw: string): Pick<DocsLivePlaceholderEntry, "value" | "kind" | "source" | "reviewStatus"> {
  const [valuePart, ...metadataParts] = raw.split("|").map((part) => part.trim());
  let kind: DocsLivePlaceholderKind = "text";
  let source = "";
  let reviewStatus: DocsLivePlaceholderReviewStatus = "provided";
  for (const part of metadataParts) {
    const pair = part.match(/^([A-Za-z][A-Za-z0-9 _-]{1,24})\s*=\s*(.+)$/);
    if (!pair) continue;
    const key = normalizePlaceholderKey(pair[1]);
    const value = pair[2].trim();
    if (key === "type" || key === "kind" || key === "category") kind = normalizePlaceholderKind(value);
    if (key === "source" || key === "evidence") source = value;
    if (key === "status" || key === "review" || key === "review status") reviewStatus = normalizePlaceholderReviewStatus(value);
  }
  return { value: valuePart.trim(), kind, source, reviewStatus };
}

function normalizePlaceholderKind(value: unknown): DocsLivePlaceholderKind {
  const normalized = typeof value === "string" ? value.trim().toLowerCase().replace(/[_ ]+/g, "-") : "";
  if (
    normalized === "person" ||
    normalized === "date" ||
    normalized === "money" ||
    normalized === "number" ||
    normalized === "source" ||
    normalized === "reviewer" ||
    normalized === "client" ||
    normalized === "decision" ||
    normalized === "channel"
  ) {
    return normalized;
  }
  return "text";
}

function normalizePlaceholderReviewStatus(value: unknown): DocsLivePlaceholderReviewStatus {
  const normalized = typeof value === "string" ? value.trim().toLowerCase().replace(/[_ ]+/g, "-") : "";
  if (normalized === "needs-review" || normalized === "verified") return normalized;
  return "provided";
}

function hydrateDocsLivePlaceholderEntries(placeholders: Record<string, string>, entries: DocsLivePlaceholderEntry[]) {
  const byKey = new Map(entries.map((entry) => [entry.key, entry]));
  return Object.entries(placeholders).map(([key, value]) => byKey.get(key) || {
    key,
    value,
    kind: "text" as const,
    source: "",
    reviewStatus: "provided" as const,
  });
}

function placeholdersTable(entries: DocsLivePlaceholderEntry[]) {
  if (!entries.length) return "<!-- Docs Live placeholders: add key facts before review. -->";
  return [
    "## Draft Inputs",
    "",
    "| Placeholder | Value | Type | Source | Review status |",
    "| --- | --- | --- | --- | --- |",
    ...entries.map((entry) =>
      `| ${titleCase(entry.key)} | ${escapeTableCell(entry.value)} | ${titleCase(entry.kind)} | ${escapeTableCell(entry.source || "not supplied")} | ${entry.reviewStatus} |`,
    ),
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
  planningOnly: boolean,
): DocsLiveSectionDraft {
  const focus = blueprint.sectionFocus[index % blueprint.sectionFocus.length];
  const workflow = workflowProfileFor(blueprint);
  const owner = placeholders.owner || placeholders.reviewer || "the named owner";
  const evidence = placeholders.evidence || placeholders.source || "the strongest available evidence";
  const draftingBrief = blueprint.workflow
    ? `Draft this ${workflow.unitLabel} only after the ${workflow.planningLabel.toLowerCase()} is locked; frame the ${focus} for ${placeholders.audience || "the intended reader"} and connect it to the next ${workflow.unitLabel}.`
    : `Frame the ${focus} for ${placeholders.audience || "the intended reader"} and connect it to the next decision.`;
  const contextBridge = contextSentences[index % Math.max(1, contextSentences.length)] || "Use the outline intent and keep unresolved facts visibly marked.";
  const qaSummary = blueprint.workflow
    ? `${section.title} must advance the locked ${workflow.planningLabel.toLowerCase()}, tie ${focus} claims to ${evidence}, and satisfy ${workflow.qualityLabel.toLowerCase()}.`
    : `${section.title} must tie ${focus} claims to ${evidence}, name ownership, and avoid unsupported certainty.`;
  const humanizedAngle = blueprint.workflow
    ? `Make ${section.title} read like a deliberate ${workflow.unitLabel} written by a human author: concrete moments, specific terms, varied cadence, and no generic AI filler.`
    : `Make ${section.title} sound like a responsible subject-matter owner wrote it: specific nouns, concrete verbs, and no generic AI filler.`;
  const reviewHandoff = blueprint.workflow
    ? `${owner} should verify the ${focus}, confirm this ${workflow.unitLabel} follows the planned sequence, and decide whether it can move into ${workflow.qualityLabel.toLowerCase()}.`
    : `${owner} should verify the ${focus}, fill missing facts, and decide whether this section can be marked human-reviewed.`;
  return {
    title: section.title,
    level: section.level,
    planningOnly,
    qaFocus: focus,
    draftingBrief,
    contextBridge,
    stagePlan: [
      {
        id: "draft",
        label: planningOnly ? "Plan before drafting" : "Draft body",
        status: planningOnly ? "needs-input" : "complete",
        detail: planningOnly
          ? `Prose is intentionally blocked until the ${workflow.planningLabel.toLowerCase()} is approved. Context available: ${contextBridge}`
          : `${draftingBrief} Context used: ${contextBridge}`,
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
      ...(blueprint.workflow ? [`The ${workflow.planningLabel.toLowerCase()} remains stable before drafting continues to the next ${workflow.unitLabel}.`] : []),
      `Claims are tied to ${evidence}, a named owner, a date, or a citation.`,
      `The section explains what ${owner} should do next.`,
    ],
    qaSummary,
    humanizationNotes: [
      "Replace generic claims with named facts, numbers, teams, customers, dates, or examples.",
      ...(blueprint.workflow ? [`Preserve chapter-to-chapter continuity so the next ${workflow.unitLabel} can build on this one without re-planning.`] : []),
      "Cut filler phrases, repeated framing, and any sentence that sounds like a prompt response.",
      "Keep the cadence natural: short setup, specific evidence, then a concrete implication.",
    ],
    humanizedAngle,
    reviewQuestions: [
      `Does ${section.title} answer the reader's likely first question?`,
      ...(blueprint.workflow ? [`Does this ${workflow.unitLabel} follow the locked sequence and prepare the next ${workflow.unitLabel}?`] : []),
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
  const promptSummary = sanitizeMarkerValue(
    section.planningOnly ? `Planned ${section.title} before prose drafting ${index + 1} of ${total}` : `Drafted ${section.title} section ${index + 1} of ${total}`,
  );
  const body = sectionBodyParagraphs(section, index, subject, audience, context, placeholders, contextSentences, draftingDepth);
  const bodyParagraphs = section.planningOnly
    ? [
        `Prose intentionally blocked until the ${workflowProfileFor(blueprint).planningLabel.toLowerCase()} is approved. Use this ${section.title.toLowerCase()} entry to refine goals, dependencies, acceptance criteria, continuity, evidence, and reviewer questions before drafting.`,
        `Context available for planning: ${context}`,
      ]
    : body;
  return [
    docsLiveReviewMarker(promptSummary),
    `${"#".repeat(level)} ${section.title}`,
    "",
    `**Drafting brief.** ${section.draftingBrief}`,
    "",
    ...bodyParagraphs.flatMap((paragraph) => [paragraph, ""]),
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
  const fourth = depthReviewSentence(draftingDepth);
  if (draftingDepth === "summary" || draftingDepth === "executive") return [first, second];
  if (draftingDepth === "detailed" || draftingDepth === "technical" || draftingDepth === "legal") return [first, second, third, fourth];
  return [first, second, fourth];
}

function depthReviewSentence(draftingDepth: DocsLiveDraftDepth) {
  if (draftingDepth === "technical") {
    return "Before this section is approved, verify technical terms, assumptions, dependencies, examples, interfaces, and source references.";
  }
  if (draftingDepth === "legal") {
    return "Before this section is approved, verify obligations, caveats, defined parties, authority, dates, and reviewer assumptions.";
  }
  if (draftingDepth === "executive") {
    return "Before this section is approved, verify the decision headline, tradeoff, owner, and timing are explicit enough for executive review.";
  }
  return "Before this section is approved, remove unsupported certainty, add citations or calculations for factual claims, and keep only language a responsible human reviewer would stand behind.";
}

function factSentence(placeholders: Record<string, string>) {
  const entries = Object.entries(placeholders).filter(([key]) => !["tone", "reviewer", "approver"].includes(key));
  if (!entries.length) return "";
  return entries
    .slice(0, 6)
    .map(([key, value]) => `${titleCase(key)}: ${value}`)
    .join("; ");
}

function qualityChecklist(sections: OutlinePlanItem[], placeholders: Record<string, string>, blueprint: DocsLiveBlueprint, outlineWasProvided: boolean) {
  const workflow = workflowProfileFor(blueprint);
  return [
    `Every planned section has a drafted body (${sections.length} section${sections.length === 1 ? "" : "s"}).`,
    ...(blueprint.workflow
      ? [
          outlineWasProvided
            ? `${workflow.planningLabel} was locked before prose drafting began.`
            : `${workflow.planningLabel} was generated from the document type and must be approved before prose is accepted.`,
          `${workflow.sequencingLabel} was followed without skipping ahead or changing continuity silently.`,
          workflow.qualityInstruction,
        ]
      : [outlineWasProvided ? "Outline was supplied before prose drafting began." : "Suggested outline was generated from the document type and must be approved before prose is accepted."]),
    "Each recommendation, risk, date, and amount is backed by source material.",
    "The opening section states the audience, decision, and desired next action.",
    Object.keys(placeholders).length ? "Placeholder values were inserted and checked for accuracy." : "Missing placeholder values were filled or explicitly marked.",
    "AI provenance and review markers remain until a human reviewer signs off.",
  ];
}

function workflowProfileFor(blueprint: DocsLiveBlueprint): DocsLiveWorkflowProfile {
  return blueprint.workflow || {
    planningLabel: "Outline",
    planningInstruction: "Lock the outline before prose is drafted.",
    planningArtifacts: ["Audience", "purpose", "outline order", "source needs", "review owner"],
    sequencingLabel: "Section-by-section draft",
    sequencingInstruction: "Draft each section in outline order with local evidence prompts and a review handoff.",
    sequenceAcceptance: ["Section purpose, evidence needs, unresolved assumptions, and review owner are clear before moving on."],
    qualityLabel: "Quality assurance",
    qualityInstruction: "Check every section for factual support, reader fit, unresolved assumptions, and review readiness.",
    qualityChecks: ["Reader purpose is clear.", "Evidence and assumptions are visible.", "Review handoff is complete."],
    unitLabel: "section",
  };
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
  if (value === "concise") return "summary";
  if (value === "summary" || value === "standard" || value === "detailed" || value === "technical" || value === "legal" || value === "executive") return value;
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
