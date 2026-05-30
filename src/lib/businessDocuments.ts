export interface BusinessProfile {
  fullName: string;
  email: string;
  phone: string;
  roleTitle: string;
  companyName: string;
  companyAddress: string;
  website: string;
  industry: string;
  defaultClientName: string;
  brandVoice: string;
}

export interface BusinessProfileField {
  key: keyof BusinessProfile;
  label: string;
  placeholder: string;
}

export type BusinessDocumentKind =
  | "blank"
  | "tutorial"
  | "lesson-plan"
  | "lesson-content"
  | "textbook"
  | "technical-textbook"
  | "novel"
  | "podcast-script"
  | "movie-script"
  | "proposal"
  | "rfp"
  | "rfp-response"
  | "rfq"
  | "tender"
  | "sow"
  | "capability-statement"
  | "case-study"
  | "report"
  | "business-case"
  | "executive-brief"
  | "board-decision-memo"
  | "policy-brief"
  | "research-report"
  | "grant-application"
  | "standard-operating-procedure"
  | "product-requirements-document"
  | "project-charter"
  | "quarterly-business-review"
  | "due-diligence-memo"
  | "contract-review-brief"
  | "implementation-playbook"
  | "incident-postmortem"
  | "meeting-decision-pack"
  | "market-research-report";

export interface BusinessDocumentTemplate {
  id: BusinessDocumentKind;
  label: string;
  summary: string;
  docsLiveType: string;
  bestFor: string[];
  aiPrompt: string;
  outline: string[];
}

export type DocumentOutlineTemplateSource = "builtin" | "custom";

export interface DocumentOutlineTemplate {
  id: string;
  source: DocumentOutlineTemplateSource;
  name: string;
  category: string;
  summary: string;
  docsLiveType?: string;
  outline: string[];
  tags: string[];
  bestFor: string[];
}

export type CustomDocumentOutlineTemplate = Omit<DocumentOutlineTemplate, "source">;

export type BusinessSnippetKind =
  | "identity"
  | "proposal"
  | "procurement"
  | "delivery"
  | "governance"
  | "review";

export interface BusinessDocumentSnippet {
  id: string;
  label: string;
  kind: BusinessSnippetKind;
  summary: string;
  body: string;
}

export interface VersionedBusinessClause {
  id: string;
  label: string;
  kind: BusinessSnippetKind;
  currentVersion: string;
  summary: string;
  body: string;
  staleMarkers: string[];
}

export type CustomVersionedBusinessClause = VersionedBusinessClause;

export interface VersionedClauseAuditItem {
  id: string;
  label: string;
  currentVersion: string;
  status: "current" | "stale" | "missing";
  line: number;
  detail: string;
}

export interface AiDocumentWizardStep {
  id: string;
  label: string;
  prompt: string;
}

export interface AiDocumentWizardStepAssistance {
  stepId: string;
  stepLabel: string;
  suggestedAnswer: string;
  rationale: string;
  contextSignals: string[];
}

export interface RfpWizardStepAssistance extends AiDocumentWizardStepAssistance {
  actionLabel: string;
}

export interface AgenticCliIntegration {
  id: "claude-code" | "codex" | "opencode" | "google-antigravity";
  label: string;
  command: string;
  summary: string;
  handoff: string;
}

export type RfpSourceKind = "markdown" | "pdf" | "docx" | "url";

export interface RfpSourceInput {
  kind: RfpSourceKind;
  title?: string;
  url?: string;
  text: string;
}

export type RfpRequirementType = "MANDATORY" | "SCORED" | "DEADLINE" | "FORMAT" | "IMPLIED" | "REQUIREMENT";

export interface RfpRequirement {
  id: string;
  requirementType: RfpRequirementType;
  category: string;
  text: string;
  sourceLine: number;
  sourceExcerpt: string;
  disqualificationRisk: boolean;
  confidence: "high" | "medium" | "low";
  responseStrategy: string;
  evidenceNeeded: string;
  owner: string;
}

export interface RfpComplianceRow extends RfpRequirement {
  complianceStatus: "Responsive draft prepared" | "Needs evidence review";
  responseSection: string;
  suggestedResponse: string;
  verification: string;
  verificationChecklist: string[];
}

export interface RfpScoringWeight {
  criterion: string;
  weight: number;
  unit: "%" | "points";
  sourceLine: number;
}

export interface RfpAnnexReference {
  annex: string;
  label: string;
  sourceLine: number;
  requirement: string;
}

export type RfpComplianceChecklistRisk = "critical" | "high" | "standard";

export interface RfpComplianceChecklistItem {
  id: string;
  section: string;
  requirement: string;
  verification: string;
  reference: string;
  risk: RfpComplianceChecklistRisk;
  owner: string;
  sourceLine: number;
}

export interface RfpProposalMetadata {
  submissionDeadline: string;
  pageLimit: number;
  pageLimitSource: string;
  currency: string;
  evaluationModel: string;
  passFailCriteria: string[];
}

export interface RfpProposalActivity {
  label: string;
  sourceLine: number;
  placeholder: string;
}

export interface RfpProposalTeamRequirement {
  role: string;
  minimumExperience: string;
  sourceLine: number;
}

export interface RfpProposalOutline {
  metadata: RfpProposalMetadata;
  scoringScheme: RfpScoringWeight[];
  activities: RfpProposalActivity[];
  deliverables: RfpProposalActivity[];
  timelineMilestones: RfpProposalActivity[];
  approvalPeriods: RfpProposalActivity[];
  annexes: RfpAnnexReference[];
  teamRequirements: RfpProposalTeamRequirement[];
  technicalMandates: string[];
  sustainabilityRequirements: string[];
  riskQaKpiRequirements: string[];
  pageAllocations: Array<{ section: string; pages: string; basis: string }>;
}

export interface RfpVerificationSummary {
  totalRequirements: number;
  complianceRows: number;
  rowsNeedingEvidence: number;
  allRequirementsMapped: boolean;
  checklist: string[];
}

export interface RfpAnalysis {
  source: {
    kind: RfpSourceKind;
    title: string;
    url: string;
    lineCount: number;
    wordCount: number;
  };
  requirements: RfpRequirement[];
  complianceRows: RfpComplianceRow[];
  complianceChecklist: RfpComplianceChecklistItem[];
  verificationSummary: RfpVerificationSummary;
  capabilities: string[];
  statedIntent: string[];
  impliedIntent: string[];
  timelines: string[];
  budgetHints: string[];
  evaluationCriteria: string[];
  mandatoryAttachments: string[];
  criticalDisqualifiers: string[];
  scoringWeights: RfpScoringWeight[];
  annexReferences: RfpAnnexReference[];
  bilingualRequirements: string[];
  placeholderRisks: string[];
  proposalOutline: RfpProposalOutline;
  risks: string[];
  questions: string[];
  warnings: string[];
  completenessScore: number;
}

export interface RfpWizardStepAssistanceInput {
  sourceKind: RfpSourceKind;
  sourceTitle?: string;
  sourceUrl?: string;
  sourceText?: string;
  responseNotes?: string;
  analysis?: RfpAnalysis | null;
  profile?: Partial<BusinessProfile>;
}

export const businessProfileFields: BusinessProfileField[] = [
  { key: "fullName", label: "Your name", placeholder: "Jane Doe" },
  { key: "email", label: "Email address", placeholder: "jane@example.com" },
  { key: "phone", label: "Phone", placeholder: "+1 555 0100" },
  { key: "roleTitle", label: "Role or title", placeholder: "Managing Partner" },
  { key: "companyName", label: "Company name", placeholder: "Acme Advisory LLC" },
  { key: "companyAddress", label: "Company address", placeholder: "123 Market Street, Suite 400, San Francisco, CA" },
  { key: "website", label: "Website", placeholder: "https://example.com" },
  { key: "industry", label: "Industry", placeholder: "Management consulting" },
  { key: "defaultClientName", label: "Default client", placeholder: "Client organization" },
  { key: "brandVoice", label: "Brand voice", placeholder: "Clear, commercially practical, evidence-led" },
];

export const defaultBusinessProfile: BusinessProfile = {
  fullName: "",
  email: "",
  phone: "",
  roleTitle: "",
  companyName: "",
  companyAddress: "",
  website: "",
  industry: "",
  defaultClientName: "",
  brandVoice: "",
};

export const businessDocumentTemplates: BusinessDocumentTemplate[] = [
  {
    id: "blank",
    label: "Blank document",
    summary: "Starts a clean document with only title, purpose, draft space, and review notes.",
    docsLiveType: "business-brief",
    bestFor: ["Freeform drafting", "Custom structures", "One-off documents"],
    aiPrompt: "Start a clean document, ask for purpose, audience, source material, and review owner, then keep structure flexible until the user defines headings.",
    outline: ["Purpose", "Audience", "Draft Section", "Open Questions", "Review Notes"],
  },
  {
    id: "tutorial",
    label: "Tutorial or training guide",
    summary: "Turns a process, product, or workflow into a guided learning document.",
    docsLiveType: "tutorial",
    bestFor: ["Customer enablement", "Internal training", "Step-by-step adoption"],
    aiPrompt: "Build a practical tutorial with prerequisites, learning goals, walkthrough steps, checks, and troubleshooting.",
    outline: ["Learning Goals", "Audience and Prerequisites", "Before You Begin", "Step-by-Step Walkthrough", "Practice Exercise", "Troubleshooting", "Next Steps"],
  },
  {
    id: "lesson-plan",
    label: "Lesson plan",
    summary: "Creates a classroom-ready teaching plan with objectives, flow, assessment, and differentiation.",
    docsLiveType: "lesson-plan",
    bestFor: ["Teachers", "Corporate training", "Workshop facilitators"],
    aiPrompt: "Create a practical lesson plan with objectives, standards or prerequisites, materials, timed activities, checks for understanding, assessment, differentiation, and extension work.",
    outline: ["Learning Objectives", "Standards and Prerequisites", "Materials", "Lesson Flow", "Guided Practice", "Assessment", "Differentiation", "Homework or Extension"],
  },
  {
    id: "lesson-content",
    label: "Lesson content",
    summary: "Builds the teachable content, examples, prompts, checks, and handouts for a lesson.",
    docsLiveType: "lesson-content",
    bestFor: ["Course authors", "Instructional designers", "Enablement teams"],
    aiPrompt: "Create lesson content with a hook, concise explanations, worked examples, practice activities, knowledge checks, teacher notes, and learner-facing handouts.",
    outline: ["Opening Hook", "Core Explanation", "Worked Example", "Practice Activity", "Knowledge Check", "Discussion Prompts", "Teacher Notes", "Learner Handout"],
  },
  {
    id: "textbook",
    label: "Textbook",
    summary: "Creates a non-fiction textbook outline first, then drafts chapters sequentially with learning checks.",
    docsLiveType: "technical-textbook",
    bestFor: ["Course books", "Training manuals", "Long-form instructional content"],
    aiPrompt:
      "First create and lock the textbook plan: subject scope, audience level, prerequisites, chapter sequence, learning outcomes, examples, exercises, glossary, and assessment approach. After approval, draft chapters sequentially and finish with instructional quality review.",
    outline: [
      "Textbook Plan",
      "Audience and Learning Outcomes",
      "Chapter Outline",
      "Chapter 1 - Foundations",
      "Chapter 2 - Core Concepts",
      "Chapter 3 - Worked Examples",
      "Chapter 4 - Practice and Assessment",
      "Glossary",
      "Instructional Quality Review",
    ],
  },
  {
    id: "technical-textbook",
    label: "Technical textbook",
    summary: "Creates a textbook outline first, then drafts chapters sequentially with instructional quality review.",
    docsLiveType: "technical-textbook",
    bestFor: ["Technical education", "Certification courses", "Engineering documentation"],
    aiPrompt:
      "First create and lock the textbook architecture: subject scope, reader level, prerequisite sequence, chapter outline, learning outcomes, notation, examples, exercises, and assessment logic. After the outline is approved, draft chapters sequentially and finish with instructional quality review for technical accuracy, progression, examples, exercises, glossary consistency, and citation readiness.",
    outline: [
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
  },
  {
    id: "novel",
    label: "Novel",
    summary: "Builds the plot first, then drafts chapters in order with narrative quality review.",
    docsLiveType: "novel",
    bestFor: ["Fiction drafting", "Story bibles", "Developmental editing"],
    aiPrompt:
      "First create and lock the story architecture: genre, premise, point of view, character arcs, world rules, plot outline, act turns, chapter order, continuity promises, and thematic questions. After the plot is approved, draft chapters sequentially and finish with narrative quality review for causality, character movement, voice, pacing, scene necessity, continuity, and AI-sounding prose.",
    outline: [
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
  },
  {
    id: "podcast-script",
    label: "Podcast script",
    summary: "Locks episode architecture first, then drafts segments sequentially with audio production review.",
    docsLiveType: "podcast-script",
    bestFor: ["Podcast episodes", "Interview shows", "Narrative audio"],
    aiPrompt:
      "First create and lock the episode architecture: audience promise, segment order, timing, host and guest roles, sound cues, sponsor obligations, facts, claims, transcript needs, and listener takeaway. After the rundown is approved, draft segments sequentially and finish with audio production quality review for listener flow, timing, host voice, interview logic, sponsor compliance, fact/source readiness, transcript readiness, and production handoff clarity.",
    outline: ["Episode Architecture", "Segment Rundown", "Cold Open", "Intro", "Segment 1", "Segment 2", "Guest Questions", "Sponsor or Promo Read", "Outro", "Production Notes", "Audio Production Review"],
  },
  {
    id: "movie-script",
    label: "Movie script",
    summary: "Locks screen story architecture first, then drafts beats sequentially with screenplay quality review.",
    docsLiveType: "movie-script",
    bestFor: ["Screenplays", "Film treatments", "Scene planning"],
    aiPrompt:
      "First create and lock the screen story architecture: logline, protagonist want and need, central conflict, act turns, beat sheet, scene order, visual rules, dialogue promises, tone, rating considerations, and production constraints. After the beat sheet is approved, draft beats sequentially and finish with screenplay quality review for screen story logic, visual playability, character motivation, act-turn causality, dialogue texture, pacing, continuity, tone, and production feasibility.",
    outline: ["Screen Story Architecture", "Logline", "Characters", "World and Tone", "Beat Sheet", "Act I", "Act II", "Act III", "Key Scenes", "Dialogue Notes", "Production Constraints", "Screenplay Quality Review"],
  },
  {
    id: "proposal",
    label: "Client proposal",
    summary: "Frames a client problem, solution, scope, pricing assumptions, and acceptance path.",
    docsLiveType: "proposal",
    bestFor: ["Consulting offers", "Implementation projects", "Commercial services"],
    aiPrompt: "Create a proposal that is client-centered, specific about scope, transparent about assumptions, and ready for review.",
    outline: ["Executive Summary", "Client Situation", "Recommended Approach", "Scope of Work", "Deliverables", "Timeline", "Investment", "Assumptions", "Next Steps"],
  },
  {
    id: "rfp",
    label: "Request for proposal",
    summary: "Creates a buyer-side RFP package with scope, vendor instructions, evaluation criteria, and response matrix.",
    docsLiveType: "project-plan",
    bestFor: ["Procurement packages", "Vendor selection", "Formal buyer requirements"],
    aiPrompt: "Create a buyer-side request for proposal that clearly defines the opportunity, scope, vendor instructions, evaluation method, and required response matrix.",
    outline: ["Opportunity Summary", "Scope of Work", "Vendor Instructions", "Evaluation Criteria", "Required Response Matrix", "Submission Checklist", "Clarification Process"],
  },
  {
    id: "rfp-response",
    label: "RFP response",
    summary: "Structures a response to a request for proposal with compliance, solution, team, and evaluation evidence.",
    docsLiveType: "rfp-response",
    bestFor: ["Public procurement", "Enterprise vendor selection", "Competitive bids"],
    aiPrompt: "Draft an RFP response that mirrors buyer requirements, tracks compliance, and makes differentiators easy to evaluate.",
    outline: ["Executive Response", "Compliance Matrix", "Understanding of Requirements", "Proposed Solution", "Implementation Plan", "Team and Experience", "Pricing Response", "Risk and Assumptions", "Appendices"],
  },
  {
    id: "rfq",
    label: "RFQ response",
    summary: "Prepares a concise quotation response with pricing basis, inclusions, exclusions, and validity.",
    docsLiveType: "rfq-response",
    bestFor: ["Price quotations", "Supplier comparisons", "Standardized services"],
    aiPrompt: "Create an RFQ response focused on clear price structure, included services, exceptions, and commercial terms.",
    outline: ["Quotation Summary", "Buyer Requirements", "Quoted Items", "Pricing Table", "Inclusions", "Exclusions", "Delivery Schedule", "Commercial Terms", "Validity and Acceptance"],
  },
  {
    id: "tender",
    label: "Tender response",
    summary: "Builds a submission package with bid strategy, compliance, method statement, risks, and attachments.",
    docsLiveType: "tender-response",
    bestFor: ["Government tenders", "Formal bids", "Regulated procurement"],
    aiPrompt: "Build a tender response with strict compliance tracking, method statement detail, governance, and submission readiness.",
    outline: ["Bid Summary", "Mandatory Submission Checklist", "Compliance Statement", "Technical Methodology", "Work Plan", "Key Personnel", "Quality and Risk Management", "Commercial Offer", "Required Attachments"],
  },
  {
    id: "sow",
    label: "Statement of work",
    summary: "Defines project scope, deliverables, responsibilities, acceptance criteria, and change control.",
    docsLiveType: "project-plan",
    bestFor: ["Delivery kickoff", "Contract attachments", "Services governance"],
    aiPrompt: "Create a statement of work that reduces ambiguity in scope, responsibilities, acceptance, and change control.",
    outline: ["Purpose", "Scope", "Deliverables", "Out of Scope", "Project Plan", "Roles and Responsibilities", "Acceptance Criteria", "Change Control", "Commercial Terms"],
  },
  {
    id: "capability-statement",
    label: "Capability statement",
    summary: "Presents company qualifications, differentiators, proof points, and relevant experience.",
    docsLiveType: "marketing-brief",
    bestFor: ["Business development", "Vendor registration", "Introductory submissions"],
    aiPrompt: "Create a concise capability statement with proof points, services, differentiators, and relevant experience.",
    outline: ["Company Overview", "Core Capabilities", "Differentiators", "Relevant Experience", "Certifications and Compliance", "Representative Clients", "Contact"],
  },
  {
    id: "case-study",
    label: "Case study",
    summary: "Captures a customer challenge, solution, implementation path, outcomes, and review approvals.",
    docsLiveType: "customer-case-study",
    bestFor: ["Sales proof", "Marketing collateral", "Client success stories"],
    aiPrompt: "Draft a case study that separates verified facts from placeholders and leaves quotes/results ready for approval.",
    outline: ["Customer Snapshot", "Challenge", "Solution", "Implementation", "Results", "Quote Prompts", "Review Approvals"],
  },
  {
    id: "report",
    label: "Business report",
    summary: "Creates a decision report with executive summary, evidence, analysis, recommendations, risks, and next steps.",
    docsLiveType: "business-brief",
    bestFor: ["Management reports", "Analysis memos", "Decision support"],
    aiPrompt: "Create a business report that turns evidence and analysis into clear findings, recommendations, risks, and accountable next steps.",
    outline: ["Executive Summary", "Situation", "Evidence Base", "Analysis", "Recommendations", "Risks and Next Steps", "Review Handoff"],
  },
  {
    id: "business-case",
    label: "Business case",
    summary: "Compares options, investment, benefits, risks, and implementation logic for a decision.",
    docsLiveType: "business-case",
    bestFor: ["Investment approval", "Operating changes", "Portfolio decisions"],
    aiPrompt: "Create a decision-ready business case with assumptions, options, financial logic, risks, and a clear recommendation.",
    outline: ["Executive Summary", "Decision Needed", "Problem", "Options", "Financial Case", "Risks", "Recommendation", "Implementation Plan"],
  },
  {
    id: "executive-brief",
    label: "Executive brief",
    summary: "Condenses context, decision, options, risks, and recommendation for a senior audience.",
    docsLiveType: "business-brief",
    bestFor: ["Leadership updates", "Decision meetings", "Board pre-reads"],
    aiPrompt: "Create a crisp executive brief that surfaces the decision, facts, tradeoffs, and requested action.",
    outline: ["Executive Summary", "Decision Needed", "Context", "Options", "Risks", "Recommendation", "Next Steps"],
  },
  {
    id: "board-decision-memo",
    label: "Board decision memo",
    summary: "Creates a board-ready decision paper with options, financial case, risks, recommendation, and approvals.",
    docsLiveType: "board-memo",
    bestFor: ["Board packs", "Investment approvals", "Executive decisions"],
    aiPrompt: "Create a board decision memo that separates the decision ask, evidence, tradeoffs, risks, financial implications, and approval path.",
    outline: ["Decision Requested", "Executive Summary", "Strategic Context", "Options Considered", "Financial Case", "Risk Assessment", "Implementation Plan", "Recommendation", "Appendices"],
  },
  {
    id: "policy-brief",
    label: "Policy brief",
    summary: "Builds an evidence-led policy brief with problem framing, options, impacts, tradeoffs, and recommendation.",
    docsLiveType: "policy",
    bestFor: ["Public policy", "Research translation", "Advisory briefs"],
    aiPrompt: "Create a policy brief that translates evidence into practical options, impacts, risks, and an actionable recommendation.",
    outline: ["Executive Summary", "Problem Definition", "Policy Context", "Evidence Base", "Options", "Impact Assessment", "Risks and Tradeoffs", "Recommendation", "Implementation Considerations"],
  },
  {
    id: "research-report",
    label: "Research report",
    summary: "Creates a structured research report with questions, method, findings, limitations, recommendations, and bibliography.",
    docsLiveType: "research-brief",
    bestFor: ["Deep research", "Evidence reports", "Analyst deliverables"],
    aiPrompt: "Create a research report that keeps research questions, method, evidence, findings, limitations, recommendations, and citation readiness traceable.",
    outline: ["Abstract", "Introduction", "Research Questions", "Methodology", "Literature and Source Review", "Findings", "Analysis", "Limitations", "Recommendations", "Bibliography"],
  },
  {
    id: "grant-application",
    label: "Grant application",
    summary: "Structures a funder-aligned application with need, beneficiaries, theory of change, work plan, budget, and measurement.",
    docsLiveType: "proposal",
    bestFor: ["Grant proposals", "Foundation applications", "Nonprofit funding"],
    aiPrompt: "Create a grant application that maps the funder's priorities to the applicant profile, need, program design, measurement, budget, sustainability, and required attachments.",
    outline: ["Cover Summary", "Applicant Profile", "Problem and Need", "Target Beneficiaries", "Theory of Change", "Program Design", "Work Plan", "Monitoring and Evaluation", "Budget Narrative", "Sustainability", "Attachments"],
  },
  {
    id: "standard-operating-procedure",
    label: "Standard operating procedure",
    summary: "Creates a repeatable SOP with ownership, procedure steps, controls, exceptions, evidence, and revision history.",
    docsLiveType: "operating-procedure",
    bestFor: ["SOPs", "Work instructions", "Operational controls"],
    aiPrompt: "Create an SOP that is easy to execute, audit, train, and update, with clear ownership, controls, records, exceptions, and approvals.",
    outline: ["Purpose", "Scope", "Owner and Approvers", "Definitions", "Inputs and Systems", "Procedure", "Controls and Checks", "Exceptions", "Records and Evidence", "Revision History"],
  },
  {
    id: "product-requirements-document",
    label: "Product requirements document",
    summary: "Creates a PRD with goals, users, requirements, acceptance criteria, dependencies, metrics, and release risks.",
    docsLiveType: "project-plan",
    bestFor: ["Product teams", "Feature planning", "Release scoping"],
    aiPrompt: "Create a product requirements document that keeps user value, requirements, UX notes, acceptance criteria, dependencies, metrics, and release risk aligned.",
    outline: ["Problem Statement", "Goals and Non-Goals", "Personas and Use Cases", "Requirements", "User Experience Notes", "Technical Dependencies", "Acceptance Criteria", "Metrics", "Risks and Open Questions", "Release Plan"],
  },
  {
    id: "project-charter",
    label: "Project charter",
    summary: "Defines project objectives, scope, stakeholders, governance, milestones, budget, risks, and approvals.",
    docsLiveType: "project-plan",
    bestFor: ["Project kickoff", "Delivery governance", "Executive approvals"],
    aiPrompt: "Create a project charter that gives sponsors and delivery teams a shared scope, governance model, milestone plan, risk posture, and success criteria.",
    outline: ["Executive Summary", "Objectives", "Scope", "Out of Scope", "Stakeholders", "Governance", "Milestones", "Budget and Resources", "Risks and Assumptions", "Success Criteria", "Approvals"],
  },
  {
    id: "quarterly-business-review",
    label: "Quarterly business review",
    summary: "Creates a client or leadership review with scorecards, delivered value, risks, opportunities, and next-quarter actions.",
    docsLiveType: "meeting-brief",
    bestFor: ["Client reviews", "Account management", "Leadership updates"],
    aiPrompt: "Create a quarterly business review that links outcomes, scorecards, delivered value, risks, opportunities, decisions, and next-quarter ownership.",
    outline: ["Executive Summary", "Period Highlights", "Goals and Scorecard", "Usage or Financial Trends", "Delivered Value", "Risks and Blockers", "Opportunities", "Next Quarter Plan", "Decisions and Actions"],
  },
  {
    id: "due-diligence-memo",
    label: "Due diligence memo",
    summary: "Creates an investment or vendor diligence memo with thesis, evidence, risks, financials, operations, and recommendation.",
    docsLiveType: "business-case",
    bestFor: ["Investment diligence", "Vendor assessment", "M&A review"],
    aiPrompt: "Create a diligence memo that separates the decision thesis, market evidence, product or service assessment, financial review, operational review, legal/compliance risks, and recommendation.",
    outline: ["Executive Summary", "Transaction or Decision Context", "Investment Thesis", "Market and Customer Evidence", "Product or Service Assessment", "Financial Review", "Operational Review", "Legal and Compliance Review", "Risk Register", "Recommendation", "Appendices"],
  },
  {
    id: "contract-review-brief",
    label: "Contract review brief",
    summary: "Creates a commercial contract review with key terms, obligations, service levels, data/IP terms, risks, and approval checklist.",
    docsLiveType: "contract-brief",
    bestFor: ["Contract review", "Legal handoff", "Commercial negotiation"],
    aiPrompt: "Create a contract review brief that helps business, legal, and delivery reviewers see obligations, risks, negotiation positions, and approvals quickly.",
    outline: ["Contract Summary", "Parties and Scope", "Commercial Terms", "Delivery Obligations", "Service Levels", "Data and IP Terms", "Liability and Indemnities", "Termination and Renewal", "Negotiation Positions", "Approval Checklist"],
  },
  {
    id: "implementation-playbook",
    label: "Implementation playbook",
    summary: "Creates an operational playbook for implementing a project, process, tool, or platform.",
    docsLiveType: "project-plan",
    bestFor: ["Delivery teams", "Rollouts", "Internal operating guides"],
    aiPrompt: "Create an implementation playbook that gives teams a practical operating model, phased rollout, roles, training, controls, success metrics, and runbook.",
    outline: ["Purpose", "Operating Model", "Scope", "Roles and Responsibilities", "Implementation Phases", "Change Management", "Training Plan", "Risks and Controls", "Success Metrics", "Runbook"],
  },
  {
    id: "incident-postmortem",
    label: "Incident postmortem",
    summary: "Creates a blameless incident review with impact, timeline, response, root cause, corrective actions, and prevention.",
    docsLiveType: "operating-procedure",
    bestFor: ["Service incidents", "Quality failures", "Operational reviews"],
    aiPrompt: "Create a blameless postmortem that preserves facts, customer impact, timeline, detection and response, causes, corrective actions, prevention, and owner sign-off.",
    outline: ["Summary", "Customer or Business Impact", "Timeline", "Detection and Response", "Root Cause Analysis", "What Went Well", "What Did Not Go Well", "Corrective Actions", "Preventive Controls", "Owner Sign-off"],
  },
  {
    id: "meeting-decision-pack",
    label: "Meeting decision pack",
    summary: "Creates a meeting-ready packet with purpose, agenda, pre-read, options, evidence, risks, decisions, and actions.",
    docsLiveType: "meeting-brief",
    bestFor: ["Steering committees", "Executive meetings", "Decision forums"],
    aiPrompt: "Create a decision-focused meeting pack that makes the required decisions, options, evidence, risks, decision log, and action register easy to review.",
    outline: ["Meeting Purpose", "Required Decisions", "Agenda", "Pre-Read Summary", "Options and Tradeoffs", "Evidence Pack", "Risks", "Decision Log", "Action Register", "Follow-up Communications"],
  },
  {
    id: "market-research-report",
    label: "Market research report",
    summary: "Creates a market analysis report with objectives, method, segments, competitors, trends, findings, and recommendations.",
    docsLiveType: "research-brief",
    bestFor: ["Market sizing", "Competitive analysis", "Strategy research"],
    aiPrompt: "Create a market research report that keeps research objectives, methodology, market definition, customer segments, competitors, trends, findings, implications, and recommendations connected.",
    outline: ["Executive Summary", "Research Objectives", "Methodology", "Market Definition", "Customer Segments", "Competitor Landscape", "Trends and Drivers", "Findings", "Implications", "Recommendations", "Bibliography"],
  },
];

const specialistDocumentOutlineTemplates: DocumentOutlineTemplate[] = [
  {
    id: "outline-rfp-technical-proposal",
    source: "builtin",
    name: "RFP technical proposal",
    category: "Procurement",
    summary: "Compliance-first technical proposal outline for evaluated RFP responses.",
    docsLiveType: "rfp-response",
    outline: [
      "Cover",
      "Compliance Checklist",
      "Table of Contents",
      "Executive Summary",
      "Assignment Understanding",
      "Proposed Methodology",
      "Work Plan and Timeline",
      "Team Organization",
      "Past Performance",
      "Risk and Quality Management",
      "Sustainability and Transition",
      "Required Annexes",
    ],
    tags: ["rfp", "proposal", "compliance", "technical"],
    bestFor: ["Competitive RFPs", "Technical proposals", "Compliance-heavy bids"],
  },
  {
    id: "outline-rfp-compliance-review",
    source: "builtin",
    name: "RFP compliance review pack",
    category: "Procurement",
    summary: "Reviewer-focused outline for compliance checks, attachments, disqualifiers, and owner sign-off.",
    docsLiveType: "rfp-response",
    outline: [
      "Source Intake Summary",
      "Critical Disqualifiers",
      "Mandatory Submission Checklist",
      "Compliance Matrix",
      "Attachment Register",
      "Evidence Owner Map",
      "Open Clarifications",
      "Submission QA Sign-off",
    ],
    tags: ["rfp", "compliance", "qa", "attachments"],
    bestFor: ["Bid QA", "Procurement review", "Submission readiness"],
  },
  {
    id: "outline-board-decision-memo",
    source: "builtin",
    name: "Board decision memo",
    category: "Executive",
    summary: "Decision-oriented outline for board or executive approval papers.",
    docsLiveType: "board-memo",
    outline: ["Decision Requested", "Executive Summary", "Strategic Context", "Options Considered", "Financial Case", "Risk Assessment", "Implementation Plan", "Recommendation", "Appendices"],
    tags: ["board", "decision", "executive", "memo"],
    bestFor: ["Board packs", "Investment approvals", "Executive decisions"],
  },
  {
    id: "outline-policy-brief",
    source: "builtin",
    name: "Policy brief",
    category: "Policy",
    summary: "Evidence-led policy outline with options, impacts, risks, and recommendation.",
    docsLiveType: "policy",
    outline: ["Executive Summary", "Problem Definition", "Policy Context", "Evidence Base", "Options", "Impact Assessment", "Risks and Tradeoffs", "Recommendation", "Implementation Considerations"],
    tags: ["policy", "brief", "evidence", "recommendation"],
    bestFor: ["Public policy", "Research translation", "Advisory briefs"],
  },
  {
    id: "outline-research-report",
    source: "builtin",
    name: "Research report",
    category: "Research",
    summary: "Structured research report outline with methodology, findings, citations, and recommendations.",
    docsLiveType: "research-brief",
    outline: ["Abstract", "Introduction", "Research Questions", "Methodology", "Literature and Source Review", "Findings", "Analysis", "Limitations", "Recommendations", "Bibliography"],
    tags: ["research", "report", "citations", "analysis"],
    bestFor: ["Deep research", "Evidence reports", "Analyst deliverables"],
  },
  {
    id: "outline-implementation-playbook",
    source: "builtin",
    name: "Implementation playbook",
    category: "Delivery",
    summary: "Operational outline for implementing a project, tool, process, or platform.",
    docsLiveType: "project-plan",
    outline: ["Purpose", "Operating Model", "Scope", "Roles and Responsibilities", "Implementation Phases", "Change Management", "Training Plan", "Risks and Controls", "Success Metrics", "Runbook"],
    tags: ["implementation", "delivery", "playbook", "operations"],
    bestFor: ["Delivery teams", "Rollouts", "Internal operating guides"],
  },
  {
    id: "outline-grant-application",
    source: "builtin",
    name: "Grant application",
    category: "Funding",
    summary: "Funder-aligned application outline with need, theory of change, work plan, budget, and measurement.",
    docsLiveType: "proposal",
    outline: ["Cover Summary", "Applicant Profile", "Problem and Need", "Target Beneficiaries", "Theory of Change", "Program Design", "Work Plan", "Monitoring and Evaluation", "Budget Narrative", "Sustainability", "Attachments"],
    tags: ["grant", "funding", "proposal", "impact"],
    bestFor: ["Grant proposals", "Foundation applications", "Nonprofit funding"],
  },
  {
    id: "outline-standard-operating-procedure",
    source: "builtin",
    name: "Standard operating procedure",
    category: "Operations",
    summary: "Repeatable SOP outline with ownership, steps, controls, records, exceptions, and revision history.",
    docsLiveType: "operating-procedure",
    outline: ["Purpose", "Scope", "Owner and Approvers", "Definitions", "Inputs and Systems", "Procedure", "Controls and Checks", "Exceptions", "Records and Evidence", "Revision History"],
    tags: ["sop", "procedure", "operations", "controls"],
    bestFor: ["SOPs", "Work instructions", "Operational controls"],
  },
  {
    id: "outline-product-requirements-document",
    source: "builtin",
    name: "Product requirements document",
    category: "Product",
    summary: "Product planning outline for goals, personas, requirements, acceptance criteria, release risks, and metrics.",
    docsLiveType: "project-plan",
    outline: ["Problem Statement", "Goals and Non-Goals", "Personas and Use Cases", "Requirements", "User Experience Notes", "Technical Dependencies", "Acceptance Criteria", "Metrics", "Risks and Open Questions", "Release Plan"],
    tags: ["product", "prd", "requirements", "release"],
    bestFor: ["Product teams", "Feature planning", "Release scoping"],
  },
  {
    id: "outline-project-charter",
    source: "builtin",
    name: "Project charter",
    category: "Delivery",
    summary: "Project start outline for objectives, scope, governance, milestones, assumptions, and success criteria.",
    docsLiveType: "project-plan",
    outline: ["Executive Summary", "Objectives", "Scope", "Out of Scope", "Stakeholders", "Governance", "Milestones", "Budget and Resources", "Risks and Assumptions", "Success Criteria", "Approvals"],
    tags: ["project", "charter", "governance", "delivery"],
    bestFor: ["Project kickoff", "Delivery governance", "Executive approvals"],
  },
  {
    id: "outline-quarterly-business-review",
    source: "builtin",
    name: "Quarterly business review",
    category: "Executive",
    summary: "Client or leadership review outline with outcomes, scorecards, risks, opportunities, and next-quarter plan.",
    docsLiveType: "meeting-brief",
    outline: ["Executive Summary", "Period Highlights", "Goals and Scorecard", "Usage or Financial Trends", "Delivered Value", "Risks and Blockers", "Opportunities", "Next Quarter Plan", "Decisions and Actions"],
    tags: ["qbr", "review", "scorecard", "customer"],
    bestFor: ["Client reviews", "Account management", "Leadership updates"],
  },
  {
    id: "outline-due-diligence-memo",
    source: "builtin",
    name: "Due diligence memo",
    category: "Strategy",
    summary: "Investment or vendor diligence outline for thesis, evidence, risks, financials, operations, and recommendation.",
    docsLiveType: "business-case",
    outline: ["Executive Summary", "Transaction or Decision Context", "Investment Thesis", "Market and Customer Evidence", "Product or Service Assessment", "Financial Review", "Operational Review", "Legal and Compliance Review", "Risk Register", "Recommendation", "Appendices"],
    tags: ["diligence", "investment", "vendor", "risk"],
    bestFor: ["Investment diligence", "Vendor assessment", "M&A review"],
  },
  {
    id: "outline-incident-postmortem",
    source: "builtin",
    name: "Incident postmortem",
    category: "Operations",
    summary: "Blameless incident review outline with timeline, impact, root causes, corrective actions, and prevention.",
    docsLiveType: "operating-procedure",
    outline: ["Summary", "Customer or Business Impact", "Timeline", "Detection and Response", "Root Cause Analysis", "What Went Well", "What Did Not Go Well", "Corrective Actions", "Preventive Controls", "Owner Sign-off"],
    tags: ["incident", "postmortem", "operations", "quality"],
    bestFor: ["Service incidents", "Quality failures", "Operational reviews"],
  },
  {
    id: "outline-meeting-decision-pack",
    source: "builtin",
    name: "Meeting decision pack",
    category: "Executive",
    summary: "Meeting-ready outline that combines agenda, pre-read, decisions, evidence, risks, and action register.",
    docsLiveType: "meeting-brief",
    outline: ["Meeting Purpose", "Required Decisions", "Agenda", "Pre-Read Summary", "Options and Tradeoffs", "Evidence Pack", "Risks", "Decision Log", "Action Register", "Follow-up Communications"],
    tags: ["meeting", "agenda", "decision", "actions"],
    bestFor: ["Steering committees", "Executive meetings", "Decision forums"],
  },
  {
    id: "outline-market-research-report",
    source: "builtin",
    name: "Market research report",
    category: "Research",
    summary: "Market analysis outline with questions, method, segmentation, competitors, findings, and recommendations.",
    docsLiveType: "research-brief",
    outline: ["Executive Summary", "Research Objectives", "Methodology", "Market Definition", "Customer Segments", "Competitor Landscape", "Trends and Drivers", "Findings", "Implications", "Recommendations", "Bibliography"],
    tags: ["market", "research", "competitors", "strategy"],
    bestFor: ["Market sizing", "Competitive analysis", "Strategy research"],
  },
  {
    id: "outline-contract-review-brief",
    source: "builtin",
    name: "Contract review brief",
    category: "Legal",
    summary: "Commercial contract review outline for key terms, obligations, risks, negotiation positions, and approvals.",
    docsLiveType: "contract-brief",
    outline: ["Contract Summary", "Parties and Scope", "Commercial Terms", "Delivery Obligations", "Service Levels", "Data and IP Terms", "Liability and Indemnities", "Termination and Renewal", "Negotiation Positions", "Approval Checklist"],
    tags: ["contract", "legal", "commercial", "risk"],
    bestFor: ["Contract review", "Legal handoff", "Commercial negotiation"],
  },
];

export const builtInDocumentOutlineTemplates: DocumentOutlineTemplate[] = [
  ...businessDocumentTemplates.map((template) => ({
    id: `business-${template.id}`,
    source: "builtin" as const,
    name: template.label,
    category: documentOutlineCategoryForTemplate(template),
    summary: template.summary,
    docsLiveType: template.docsLiveType,
    outline: template.outline,
    tags: [template.id, template.docsLiveType, ...template.bestFor].map((item) => item.toLowerCase().replace(/\s+/g, "-")),
    bestFor: template.bestFor,
  })),
  ...specialistDocumentOutlineTemplates,
];

export const businessDocumentSnippets: BusinessDocumentSnippet[] = [
  {
    id: "company-contact-block",
    label: "Company contact block",
    kind: "identity",
    summary: "Reusable sender and organization block for cover pages, letters, and submissions.",
    body: [
      "**Prepared by:** {{fullName}}, {{roleTitle}}",
      "",
      "**Company:** {{companyName}}",
      "",
      "**Address:** {{companyAddress}}",
      "",
      "**Email:** {{email}}  ",
      "**Phone:** {{phone}}  ",
      "**Website:** {{website}}",
    ].join("\n"),
  },
  {
    id: "company-overview",
    label: "Company overview",
    kind: "identity",
    summary: "Short boilerplate overview for proposals, tenders, and capability statements.",
    body: "{{companyName}} is a {{industry}} organization. We help {{defaultClientName}} make practical decisions with clear evidence, disciplined delivery, and {{brandVoice}} communication.",
  },
  {
    id: "executive-summary",
    label: "Executive summary starter",
    kind: "proposal",
    summary: "A compact executive summary scaffold with reader outcome and recommendation placeholders.",
    body: [
      "## Executive Summary",
      "",
      "{{defaultClientName}} needs {{outcome}}. {{companyName}} recommends {{recommendation}} because {{evidence}}.",
      "",
      "The proposed approach focuses on {{scope}}, with delivery led by {{fullName}} and reviewed against {{success_criteria}}.",
    ].join("\n"),
  },
  {
    id: "scope-of-work",
    label: "Scope of work",
    kind: "delivery",
    summary: "Reusable scope, deliverables, out-of-scope, and acceptance block.",
    body: [
      "## Scope of Work",
      "",
      "### In Scope",
      "",
      "- {{scope_item_1}}",
      "- {{scope_item_2}}",
      "- {{scope_item_3}}",
      "",
      "### Deliverables",
      "",
      "| Deliverable | Acceptance criteria | Owner |",
      "| --- | --- | --- |",
      "| {{deliverable}} | {{acceptance_criteria}} | {{owner}} |",
      "",
      "### Out of Scope",
      "",
      "- {{out_of_scope_item}}",
    ].join("\n"),
  },
  {
    id: "pricing-assumptions",
    label: "Pricing assumptions",
    kind: "proposal",
    summary: "Commercial assumptions that make quotes and proposals easier to review.",
    body: [
      "## Pricing Assumptions",
      "",
      "- Pricing is based on {{pricing_basis}}.",
      "- Fees exclude {{exclusions}} unless stated otherwise.",
      "- The estimate assumes timely access to {{client_inputs}}.",
      "- Pricing remains valid until {{valid_until}}.",
    ].join("\n"),
  },
  {
    id: "rfp-compliance-matrix",
    label: "RFP compliance matrix",
    kind: "procurement",
    summary: "Response matrix for buyer requirements, compliance status, and evidence references.",
    body: [
      "## Compliance Matrix",
      "",
      "| Requirement | Response | Evidence | Owner |",
      "| --- | --- | --- | --- |",
      "| {{requirement_id}} - {{requirement_text}} | {{compliant_partial_or_exception}} | {{evidence_reference}} | {{owner}} |",
    ].join("\n"),
  },
  {
    id: "tender-submission-checklist",
    label: "Tender submission checklist",
    kind: "procurement",
    summary: "Checklist for mandatory tender attachments, sign-offs, and submission readiness.",
    body: [
      "## Mandatory Submission Checklist",
      "",
      "- [ ] Signed submission form",
      "- [ ] Pricing schedule",
      "- [ ] Technical response",
      "- [ ] Compliance declarations",
      "- [ ] Insurance, tax, or registration evidence",
      "- [ ] Authorized sign-off by {{approver}}",
    ].join("\n"),
  },
  {
    id: "tutorial-step",
    label: "Tutorial step",
    kind: "delivery",
    summary: "Repeatable instruction block for tutorials and training guides.",
    body: [
      "### Step {{step_number}}: {{step_title}}",
      "",
      "**Goal:** {{step_goal}}",
      "",
      "1. {{instruction_1}}",
      "2. {{instruction_2}}",
      "3. {{instruction_3}}",
      "",
      "**Check:** {{completion_check}}",
      "",
      "**If this fails:** {{troubleshooting_tip}}",
    ].join("\n"),
  },
  {
    id: "risk-register",
    label: "Risk register",
    kind: "governance",
    summary: "Standard business risk table for proposals, RFPs, tenders, and plans.",
    body: [
      "## Risk Register",
      "",
      "| Risk | Impact | Likelihood | Mitigation | Owner |",
      "| --- | --- | --- | --- | --- |",
      "| {{risk}} | {{impact}} | {{likelihood}} | {{mitigation}} | {{owner}} |",
    ].join("\n"),
  },
  {
    id: "review-handoff",
    label: "Review handoff",
    kind: "review",
    summary: "Review instructions that keep unresolved assumptions visible before export.",
    body: [
      "## Review Handoff",
      "",
      "- Confirm all client names, figures, dates, and claims.",
      "- Resolve placeholders before sending: {{open_placeholders}}.",
      "- Confirm legal, finance, and delivery owner approvals where required.",
      "- Final reviewer: {{reviewer}}.",
    ].join("\n"),
  },
];

export const versionedBusinessClauses: VersionedBusinessClause[] = [
  {
    id: "standard-confidentiality",
    label: "Standard confidentiality",
    kind: "governance",
    currentVersion: "2026.05",
    summary: "Proposal and delivery confidentiality language for client-facing drafts.",
    staleMarkers: ["clause:standard-confidentiality version=2025", "legacy confidentiality clause"],
    body: [
      "<!-- clause:standard-confidentiality version=2026.05 status=current -->",
      "## Confidentiality",
      "",
      "This document is confidential and intended only for {{defaultClientName}} and authorized reviewers. It may not be copied, forwarded, published, or relied on outside the agreed review process without written approval from {{companyName}}.",
    ].join("\n"),
  },
  {
    id: "proposal-validity",
    label: "Proposal validity",
    kind: "proposal",
    currentVersion: "2026.05",
    summary: "Commercial validity and assumption language for proposals and tenders.",
    staleMarkers: ["clause:proposal-validity version=2025", "pricing valid for 90 days unless withdrawn"],
    body: [
      "<!-- clause:proposal-validity version=2026.05 status=current -->",
      "## Proposal Validity",
      "",
      "This proposal remains valid until {{valid_until}} and is subject to the assumptions, exclusions, approval gates, and evidence dependencies stated in this document.",
    ].join("\n"),
  },
  {
    id: "ai-review-disclosure",
    label: "AI review disclosure",
    kind: "review",
    currentVersion: "2026.05",
    summary: "Human-review language for AI-assisted business documents.",
    staleMarkers: ["clause:ai-review-disclosure version=2025", "AI generated and may contain errors"],
    body: [
      "<!-- clause:ai-review-disclosure version=2026.05 status=current -->",
      "## AI-Assisted Drafting Disclosure",
      "",
      "Portions of this document may have been drafted or revised with AI assistance. {{companyName}} remains responsible for human review, source verification, unresolved placeholders, and final approval before distribution.",
    ].join("\n"),
  },
];

export function versionedClauseMarkdown(clause: VersionedBusinessClause, profile: Partial<BusinessProfile> = {}) {
  const body = fillBusinessTemplate(clause.body, profile).trimEnd();
  const currentPattern = new RegExp(`clause:${escapeRegExp(clause.id)}\\s+version=${escapeRegExp(clause.currentVersion)}\\b`, "i");
  if (currentPattern.test(body)) return `${body}\n`;
  return `<!-- clause:${clause.id} version=${clause.currentVersion} status=current -->\n${body}\n`;
}

export function auditVersionedClauses(markdown: string, clauses: VersionedBusinessClause[] = versionedBusinessClauses): VersionedClauseAuditItem[] {
  const lines = markdown.split(/\r?\n/);
  return clauses.map((clause) => {
    const currentPattern = new RegExp(`clause:${escapeRegExp(clause.id)}\\s+version=${escapeRegExp(clause.currentVersion)}\\b`, "i");
    const currentIndex = lines.findIndex((line) => currentPattern.test(line));
    if (currentIndex >= 0) {
      return {
        id: clause.id,
        label: clause.label,
        currentVersion: clause.currentVersion,
        status: "current",
        line: currentIndex + 1,
        detail: `Current ${clause.currentVersion} clause is present.`,
      };
    }
    for (const marker of clause.staleMarkers) {
      const staleIndex = lines.findIndex((line) => line.toLowerCase().includes(marker.toLowerCase()));
      if (staleIndex >= 0) {
        return {
          id: clause.id,
          label: clause.label,
          currentVersion: clause.currentVersion,
          status: "stale",
          line: staleIndex + 1,
          detail: `Stale marker found: ${marker}`,
        };
      }
    }
    return {
      id: clause.id,
      label: clause.label,
      currentVersion: clause.currentVersion,
      status: "missing",
      line: 0,
      detail: "Current approved clause not found in document.",
    };
  });
}

export function createCustomVersionedClauseId() {
  return `custom-clause-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export function blankCustomVersionedClause(): CustomVersionedBusinessClause {
  return {
    id: createCustomVersionedClauseId(),
    label: "Custom approved clause",
    kind: "governance",
    currentVersion: "2026.05",
    summary: "Reusable approved language for client-facing documents.",
    staleMarkers: [],
    body: [
      "## Approved Clause",
      "",
      "Add approved language here. Use {{companyName}}, {{defaultClientName}}, and other saved profile fields where repeated identity should be filled automatically.",
    ].join("\n"),
  };
}

export function normalizeCustomVersionedClauses(value: unknown): CustomVersionedBusinessClause[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const clauses: CustomVersionedBusinessClause[] = [];
  for (const item of value) {
    if (!item || typeof item !== "object") continue;
    const record = item as Record<string, unknown>;
    const id = slugifyClauseId(outlineStringValue(record.id) || outlineStringValue(record.label) || createCustomVersionedClauseId());
    if (!id || seen.has(id)) continue;
    const label = outlineStringValue(record.label) || "Custom approved clause";
    const body = outlineStringValue(record.body);
    if (!body) continue;
    seen.add(id);
    clauses.push({
      id,
      label,
      kind: normalizeBusinessSnippetKind(record.kind),
      currentVersion: outlineStringValue(record.currentVersion ?? record.current_version ?? record.version) || "2026.05",
      summary: outlineStringValue(record.summary) || "Reusable approved language for client-facing documents.",
      staleMarkers: outlineStringArray(record.staleMarkers ?? record.stale_markers, 20),
      body,
    });
  }
  return clauses.slice(0, 80);
}

export interface SaveCustomVersionedClauseStateResult {
  clauses: CustomVersionedBusinessClause[];
  clause: CustomVersionedBusinessClause | null;
  changed: boolean;
}

export interface DeleteCustomVersionedClauseStateResult {
  clauses: CustomVersionedBusinessClause[];
  changed: boolean;
}

export function saveCustomVersionedClauseState(
  clauses: CustomVersionedBusinessClause[],
  clause: CustomVersionedBusinessClause,
): SaveCustomVersionedClauseStateResult {
  const normalizedClauses = normalizeCustomVersionedClauses(clauses);
  const [normalized] = normalizeCustomVersionedClauses([clause]);
  if (!normalized) return { clauses: normalizedClauses, clause: null, changed: false };
  const existingIndex = normalizedClauses.findIndex((candidate) => candidate.id === normalized.id);
  if (existingIndex >= 0) {
    return {
      clauses: normalizedClauses.map((candidate, index) => (index === existingIndex ? normalized : candidate)),
      clause: normalized,
      changed: true,
    };
  }
  return { clauses: [...normalizedClauses, normalized], clause: normalized, changed: true };
}

export function deleteCustomVersionedClauseState(
  clauses: CustomVersionedBusinessClause[],
  id: string,
): DeleteCustomVersionedClauseStateResult {
  const normalizedClauses = normalizeCustomVersionedClauses(clauses);
  const nextClauses = normalizedClauses.filter((clause) => clause.id !== id);
  return { clauses: nextClauses, changed: nextClauses.length !== normalizedClauses.length };
}

function slugifyClauseId(value: string) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80);
}

function normalizeBusinessSnippetKind(value: unknown): BusinessSnippetKind {
  const candidate = typeof value === "string" ? value.trim().toLowerCase() : "";
  if (["identity", "proposal", "procurement", "delivery", "governance", "review"].includes(candidate)) return candidate as BusinessSnippetKind;
  return "governance";
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export const aiDocumentWizardSteps: AiDocumentWizardStep[] = [
  {
    id: "identity",
    label: "Use business identity",
    prompt: "Apply saved sender, company, website, address, role, and contact values wherever the document needs repeatable identity.",
  },
  {
    id: "intent",
    label: "Clarify the job",
    prompt: "Identify audience, reader decision, deadline, distribution target, tone, and must-use evidence before drafting.",
  },
  {
    id: "outline",
    label: "Create outline or plot",
    prompt: "Create or refine the chapter, section, subsection, subsubsection, textbook architecture, or novel plot before writing prose.",
  },
  {
    id: "draft",
    label: "Draft sequentially",
    prompt: "Generate each section or chapter in order only after the outline or plot is approved, preserving placeholders for facts, figures, approvals, and sources.",
  },
  {
    id: "qa",
    label: "Quality assurance review",
    prompt: "Run checks for missing evidence, unsupported claims, stale assumptions, compliance gaps, instructional quality, narrative quality, and export readiness.",
  },
  {
    id: "humanize",
    label: "Humanize for review",
    prompt: "Rewrite generic AI wording into concrete business prose while preserving review comments and unresolved assumptions.",
  },
];

export const agenticCliIntegrations: AgenticCliIntegration[] = [
  {
    id: "claude-code",
    label: "Claude Code",
    command: "claude",
    summary: "Handoff package for teams that use Claude Code for local agentic editing and document co-writing.",
    handoff: "Paste the provider package into Claude Code with the open Markdown file as context, then review and import the edited Markdown.",
  },
  {
    id: "codex",
    label: "Codex",
    command: "codex",
    summary: "Handoff package for Codex CLI workflows that revise documents, create outlines, or run review passes locally.",
    handoff: "Open Codex in the document workspace, paste the package, and ask it to return Markdown plus review notes.",
  },
  {
    id: "opencode",
    label: "OpenCode",
    command: "opencode",
    summary: "Handoff package for OpenCode agent workflows where teams want local or approved-provider document agents.",
    handoff: "Start OpenCode in the project folder, paste the package, and keep generated changes under human review.",
  },
  {
    id: "google-antigravity",
    label: "Google Antigravity",
    command: "antigravity",
    summary: "Handoff package for teams that use Google Antigravity for local or governed document-agent workflows.",
    handoff: "Start Google Antigravity from the document folder with the prepared package, then import only reviewed Markdown changes.",
  },
];

export function normalizeBusinessProfile(value: unknown): BusinessProfile {
  const record = typeof value === "object" && value !== null && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
  const normalized = { ...defaultBusinessProfile };
  for (const field of businessProfileFields) {
    const raw = record[field.key];
    normalized[field.key] = typeof raw === "string" ? raw.trim().slice(0, 500) : "";
  }
  return normalized;
}

export function businessProfilePlaceholderMap(profile: Partial<BusinessProfile> = {}) {
  const normalized = normalizeBusinessProfile(profile);
  const canonical = Object.fromEntries(
    Object.entries(normalized).map(([key, value]) => [key, value || `{{${key}}}`]),
  ) as Record<keyof BusinessProfile, string>;
  const placeholders: Record<string, string> = {
    ...canonical,
    owner: canonical.fullName,
    reviewer: canonical.fullName,
    approver: canonical.fullName,
    preparedBy: canonical.fullName,
    "profile.owner": canonical.fullName,
    "profile.fullName": canonical.fullName,
    "profile.email": canonical.email,
    "profile.roleTitle": canonical.roleTitle,
    "profile.phone": canonical.phone,
    "company.name": canonical.companyName,
    "company.address": canonical.companyAddress,
    "company.website": canonical.website,
    "company.industry": canonical.industry,
    "client.name": canonical.defaultClientName,
  };
  return placeholders;
}

export function businessProfilePlaceholderText(profile: Partial<BusinessProfile> = {}) {
  const placeholders = businessProfilePlaceholderMap(profile);
  return businessProfileFields
    .map((field) => `${field.key}: ${placeholders[field.key]}`)
    .join("\n");
}

export function documentOutlineTemplateToPlannerText(template: Pick<DocumentOutlineTemplate, "outline">) {
  return template.outline
    .map((heading) => heading.replace(/\t/g, "  ").replace(/\s+$/g, ""))
    .filter(Boolean)
    .map((heading) => {
      const indent = heading.match(/^\s*/)?.[0] || "";
      const title = heading.trim();
      return `${indent}- ${title}`;
    })
    .join("\n");
}

export function createCustomDocumentOutlineTemplateId() {
  return `custom-outline-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export function blankCustomDocumentOutlineTemplate(): CustomDocumentOutlineTemplate {
  return {
    id: createCustomDocumentOutlineTemplateId(),
    name: "Custom outline",
    category: "Custom",
    summary: "Reusable document outline.",
    docsLiveType: "business-brief",
    outline: ["Executive Summary", "Main Section", "Review Notes"],
    tags: ["custom"],
    bestFor: ["Reusable planning"],
  };
}

export function normalizeCustomDocumentOutlineTemplates(value: unknown): CustomDocumentOutlineTemplate[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const templates: CustomDocumentOutlineTemplate[] = [];
  for (const item of value) {
    if (!item || typeof item !== "object") continue;
    const record = item as Record<string, unknown>;
    const id = outlineStringValue(record.id) || createCustomDocumentOutlineTemplateId();
    const outline = outlineStringArray(record.outline, 80, true);
    if (!outline.length || seen.has(id)) continue;
    seen.add(id);
    const docsLiveType = outlineStringValue(record.docsLiveType ?? record.docs_live_type);
    templates.push({
      id,
      name: outlineStringValue(record.name) || "Custom outline",
      category: outlineStringValue(record.category) || "Custom",
      summary: outlineStringValue(record.summary) || "Reusable document outline.",
      ...(docsLiveType ? { docsLiveType } : {}),
      outline,
      tags: outlineStringArray(record.tags, 16),
      bestFor: outlineStringArray(record.bestFor, 12),
    });
  }
  return templates.slice(0, 100);
}

export function workspaceOutlineLibraryPath(root: string) {
  const normalizedRoot = root.trim().replace(/[\\/]+$/g, "");
  return normalizedRoot ? `${normalizedRoot}/.neditor/outlines.json` : ".neditor/outlines.json";
}

export function workspaceDocumentOutlineTemplatesFromJson(text: string): CustomDocumentOutlineTemplate[] {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return [];
  }
  let outlines: unknown[] = [];
  if (Array.isArray(value)) {
    outlines = value;
  } else if (typeof value === "object" && value !== null) {
    const record = value as Record<string, unknown>;
    outlines = Array.isArray(record.outlines) ? record.outlines : [];
  }
  return normalizeCustomDocumentOutlineTemplates(
    outlines.map((item) => {
      if (!item || typeof item !== "object") return item;
      const record = item as Record<string, unknown>;
      return {
        id: record.id,
        name: record.name ?? record.label,
        category: record.category,
        summary: record.summary,
        docsLiveType: record.docsLiveType ?? record.docs_live_type,
        outline: record.outline,
        tags: record.tags,
        bestFor: record.bestFor,
      };
    }),
  );
}

export function workspaceDocumentOutlineLibraryJson(templates: CustomDocumentOutlineTemplate[]) {
  const outlines = normalizeCustomDocumentOutlineTemplates(templates).map((template) => ({
    id: template.id,
    label: template.name,
    category: template.category,
    summary: template.summary,
    ...(template.docsLiveType ? { docsLiveType: template.docsLiveType } : {}),
    bestFor: template.bestFor,
    outline: template.outline,
    tags: template.tags,
  }));
  return `${JSON.stringify({ schema: "neditor.workspace-outlines.v1", outlines }, null, 2)}\n`;
}

export interface SaveCustomDocumentOutlineTemplateStateResult {
  templates: CustomDocumentOutlineTemplate[];
  template: CustomDocumentOutlineTemplate | null;
  changed: boolean;
}

export interface DeleteCustomDocumentOutlineTemplateStateResult {
  templates: CustomDocumentOutlineTemplate[];
  changed: boolean;
}

export function saveCustomDocumentOutlineTemplateState(
  templates: CustomDocumentOutlineTemplate[],
  template: CustomDocumentOutlineTemplate,
): SaveCustomDocumentOutlineTemplateStateResult {
  const normalizedTemplates = normalizeCustomDocumentOutlineTemplates(templates);
  const [normalized] = normalizeCustomDocumentOutlineTemplates([template]);
  if (!normalized) return { templates: normalizedTemplates, template: null, changed: false };
  const existingIndex = normalizedTemplates.findIndex((candidate) => candidate.id === normalized.id);
  if (existingIndex >= 0) {
    return {
      templates: normalizedTemplates.map((candidate, index) => (index === existingIndex ? normalized : candidate)),
      template: normalized,
      changed: true,
    };
  }
  return { templates: [...normalizedTemplates, normalized], template: normalized, changed: true };
}

export function deleteCustomDocumentOutlineTemplateState(
  templates: CustomDocumentOutlineTemplate[],
  id: string,
): DeleteCustomDocumentOutlineTemplateStateResult {
  const normalizedTemplates = normalizeCustomDocumentOutlineTemplates(templates);
  const nextTemplates = normalizedTemplates.filter((template) => template.id !== id);
  return { templates: nextTemplates, changed: nextTemplates.length !== normalizedTemplates.length };
}

export function fillBusinessTemplate(markdown: string, profile: Partial<BusinessProfile> = {}, extra: Record<string, string> = {}) {
  const placeholders = { ...businessProfilePlaceholderMap(profile), ...extra };
  return markdown.replace(/\{\{([a-zA-Z0-9_. -]+)\}\}/g, (match, key: string) => {
    const trimmedKey = key.trim();
    const normalizedKey = trimmedKey.replace(/\s+/g, "_");
    return placeholders[trimmedKey] || placeholders[normalizedKey] || extra[trimmedKey] || extra[normalizedKey] || match;
  });
}

export function businessTemplateMarkdown(template: BusinessDocumentTemplate, profile: Partial<BusinessProfile> = {}) {
  const placeholders = businessProfilePlaceholderMap(profile);
  return fillBusinessTemplate(
    [
      "---",
      `title: ${yamlScalar(`${template.label} for ${placeholders.defaultClientName}`)}`,
      "status: draft",
      `documentType: ${yamlScalar(template.label)}`,
      `company: ${yamlScalar(placeholders.companyName)}`,
      `preparedBy: ${yamlScalar(placeholders.fullName)}`,
      "toc: true",
      "---",
      "",
      `# ${template.label}`,
      "",
      businessDocumentSnippets[0].body,
      "",
      "[TOC]",
      "",
      longFormTemplateGate(template),
      "",
      ...template.outline.flatMap((heading) => [`## ${heading}`, "", sectionPromptForHeading(heading)]),
      "## AI Drafting Brief",
      "",
      template.aiPrompt,
      "",
      "## Review Handoff",
      "",
      "- Confirm all placeholders, facts, figures, citations, pricing, and approvals.",
      "- Prepare export or provider handoff only after review readiness passes.",
    ].join("\n"),
    profile,
  );
}

function longFormTemplateGate(template: BusinessDocumentTemplate) {
  if (template.id === "technical-textbook") {
    return [
      "## Textbook Architecture Approval Gate",
      "",
      "- [ ] Define the reader level, prerequisites, chapter order, outcomes, notation, examples, exercises, and assessment path before drafting prose.",
      "- [ ] Approve the outline before Chapter 1 is fleshed out.",
      "- [ ] Draft chapters sequentially and run instructional quality review after the chapter sequence is complete.",
    ].join("\n");
  }
  if (template.id === "novel") {
    return [
      "## Plot Architecture Approval Gate",
      "",
      "- [ ] Define the genre, premise, point of view, protagonist goal, conflict, stakes, world rules, act turns, chapter order, and continuity promises before drafting prose.",
      "- [ ] Approve the plot outline before Chapter 1 is fleshed out.",
      "- [ ] Draft chapters sequentially and run narrative quality review after the chapter sequence is complete.",
    ].join("\n");
  }
  if (template.id === "podcast-script") {
    return [
      "## Episode Architecture Approval Gate",
      "",
      "- [ ] Define the audience promise, segment order, timing, host and guest roles, sound cues, sponsor obligations, claims, and listener takeaway before drafting script copy.",
      "- [ ] Approve the segment rundown before the cold open or Segment 1 is fleshed out.",
      "- [ ] Draft segments sequentially and run audio production quality review after the episode sequence is complete.",
    ].join("\n");
  }
  if (template.id === "movie-script") {
    return [
      "## Screen Story Architecture Approval Gate",
      "",
      "- [ ] Define the logline, protagonist want and need, central conflict, act turns, scene order, visual rules, dialogue promises, and production constraints before drafting screenplay pages.",
      "- [ ] Approve the beat sheet before Act I or the first key scene is fleshed out.",
      "- [ ] Draft beats sequentially and run screenplay quality review after the beat sequence is complete.",
    ].join("\n");
  }
  return "";
}

export function businessSnippetMarkdown(snippet: BusinessDocumentSnippet, profile: Partial<BusinessProfile> = {}) {
  return `${fillBusinessTemplate(snippet.body, profile).trimEnd()}\n`;
}

export function businessWizardContext(template: BusinessDocumentTemplate, profile: Partial<BusinessProfile> = {}) {
  const assistance = buildBusinessWizardStepAssistance(template, profile);
  return [
    `Document builder: ${template.label}`,
    `Builder goal: ${template.aiPrompt}`,
    "",
    "Saved business identity:",
    businessProfilePlaceholderText(profile),
    "",
    "Wizard workflow:",
    ...aiDocumentWizardSteps.map((step, index) => `${index + 1}. ${step.label}: ${step.prompt}`),
    "",
    "AI suggested optimal answers:",
    ...assistance.flatMap((item, index) => [
      `${index + 1}. ${item.stepLabel}`,
      `Suggested answer: ${item.suggestedAnswer}`,
      `Rationale: ${item.rationale}`,
      `Context signals: ${item.contextSignals.join("; ")}`,
    ]),
    "",
    "Agent handoff options:",
    ...agenticCliIntegrations.map((integration) => `- ${integration.label} (${integration.command}): ${integration.handoff}`),
  ].join("\n");
}

export function buildBusinessWizardStepAssistance(
  template: BusinessDocumentTemplate,
  profile: Partial<BusinessProfile> = {},
): AiDocumentWizardStepAssistance[] {
  const normalized = normalizeBusinessProfile(profile);
  const placeholders = businessProfilePlaceholderMap(normalized);
  const completion = businessProfileCompletionSummary(normalized);
  const longForm = isLongFormTemplate(template);
  const procurement = isProcurementTemplate(template);
  const primaryUse = template.bestFor[0] || "document creation";
  const outlineSummary = template.outline.slice(0, 6).join(" -> ");
  const client = placeholders.defaultClientName;
  const company = placeholders.companyName;
  const owner = placeholders.fullName;
  const voice = placeholders.brandVoice;
  const identitySignal = completion.missing.length
    ? `Missing identity fields: ${completion.missing.join(", ")}`
    : "Business identity is complete";
  const baseSignals = [
    `Document type: ${template.label}`,
    `Best for: ${template.bestFor.join(", ") || primaryUse}`,
    `Profile completeness: ${completion.completed}/${completion.total}`,
    identitySignal,
  ];

  return aiDocumentWizardSteps.map((step): AiDocumentWizardStepAssistance => {
    let suggestedAnswer = "";
    let rationale = "";
    const contextSignals = [...baseSignals];
    switch (step.id) {
      case "identity":
        suggestedAnswer = [
          `Use ${owner} as accountable author for ${company}.`,
          `Prepare the document for ${client}.`,
          `Apply the saved website, address, email, phone, industry, and brand voice wherever repeated identity appears.`,
          completion.missing.length ? `Leave unresolved identity values as visible placeholders: ${completion.missing.join(", ")}.` : "No identity placeholders need to remain open.",
        ].join(" ");
        rationale = "Business documents repeat sender, company, client, and voice details; using saved identity prevents inconsistent handoff metadata.";
        break;
      case "intent":
        suggestedAnswer = [
          `${template.label} goal: ${template.aiPrompt}`,
          `Primary reader/use: ${primaryUse}.`,
          procurement ? "Mirror buyer requirements, compliance evidence, deadlines, attachments, pricing assumptions, and evaluator scoring." : "State audience, desired reader action, deadline, success criteria, evidence, and review owner before drafting.",
          `Tone should be ${voice}.`,
        ].join(" ");
        rationale = "The best answer ties the business outcome to the template's specific document job instead of asking the user to start from a blank prompt.";
        break;
      case "outline":
        suggestedAnswer = [
          longForm ? "Approve the structure before prose is generated." : "Use the template outline as the first planning draft.",
          `Starting sequence: ${outlineSummary}.`,
          procurement ? "Keep compliance, mandatory attachments, buyer intent, pricing, and risks visible as first-class sections." : "Let the user edit headings before section drafting begins.",
        ].join(" ");
        rationale = longForm
          ? "Long-form and creative documents need architecture or plot approval before section-by-section drafting starts."
          : "Business users need a concrete outline they can approve or edit before NEditor fills in prose.";
        contextSignals.push(`Outline sections: ${template.outline.length}`);
        break;
      case "draft":
        suggestedAnswer = [
          "Draft sequentially from the approved outline.",
          `Ground every section in ${company}, ${client}, and the available source material.`,
          "Keep unverified facts, figures, quotes, pricing, and approvals as explicit placeholders or review questions.",
          procurement ? "For every requirement, include response, evidence owner, compliance status, and verification note." : "Do not collapse review notes into polished claims until the user confirms them.",
        ].join(" ");
        rationale = "Sequential drafting keeps continuity and makes it easier for non-technical users to review one section at a time.";
        break;
      case "qa":
        suggestedAnswer = [
          "Run QA before export or provider handoff.",
          procurement
            ? "Check requirement coverage, compliance matrix completeness, attachment list, scoring alignment, pricing assumptions, and exception handling."
            : "Check placeholders, unsupported claims, stale assumptions, citations, reviewer comments, structure, and export readiness.",
          longForm ? "Add instructional, narrative, audio, or screenplay quality checks appropriate to the document type." : "Create visible review tasks for every blocker.",
        ].join(" ");
        rationale = "A responsible AI-first workflow treats QA as a required stage, not an optional polish pass.";
        contextSignals.push(procurement ? "Procurement/compliance workflow" : "General business QA workflow");
        break;
      case "humanize":
        suggestedAnswer = [
          `Humanize into ${voice} prose.`,
          "Replace generic AI phrasing with concrete nouns, named owners, exact reviewer questions, and source-grounded caveats.",
          "Preserve AI provenance and needs-review markers until a human approves the final draft.",
        ].join(" ");
        rationale = "Humanization should improve specificity and readability without hiding uncertainty, provenance, or review obligations.";
        break;
      default:
        suggestedAnswer = `${step.label}: answer using ${template.label}, ${company}, ${client}, and the saved business profile as context.`;
        rationale = "Fallback suggestion keeps the wizard step tied to the current business context.";
    }
    return {
      stepId: step.id,
      stepLabel: step.label,
      suggestedAnswer,
      rationale,
      contextSignals: Array.from(new Set(contextSignals.filter(Boolean))),
    };
  });
}

function businessProfileCompletionSummary(profile: BusinessProfile) {
  const missing = businessProfileFields
    .filter((field) => !profile[field.key].trim())
    .map((field) => field.label);
  return {
    total: businessProfileFields.length,
    completed: businessProfileFields.length - missing.length,
    missing,
  };
}

function isLongFormTemplate(template: BusinessDocumentTemplate) {
  return template.id === "technical-textbook" || template.id === "novel" || template.id === "podcast-script" || template.id === "movie-script";
}

function isProcurementTemplate(template: BusinessDocumentTemplate) {
  return template.id === "rfp" || template.id === "rfq" || template.id === "tender";
}

export function buildRfpWizardStepAssistance(input: RfpWizardStepAssistanceInput): RfpWizardStepAssistance[] {
  const profile = normalizeBusinessProfile(input.profile || {});
  const company = profile.companyName || "the responding organization";
  const client = profile.defaultClientName || input.analysis?.source.title || input.sourceTitle || "the buyer";
  const owner = profile.fullName || profile.roleTitle || "the accountable response owner";
  const sourceTitle = normalizeWhitespace(input.sourceTitle || input.analysis?.source.title || "RFP source");
  const sourceUrl = normalizeWhitespace(input.sourceUrl || input.analysis?.source.url || "");
  const sourceText = input.sourceText || "";
  const sourceWords = input.analysis?.source.wordCount || sourceText.split(/\s+/).filter(Boolean).length;
  const notesWords = (input.responseNotes || "").split(/\s+/).filter(Boolean).length;
  const analysis = input.analysis || null;
  const baseSignals = [
    `Source type: ${input.sourceKind.toUpperCase()}`,
    `Source title: ${sourceTitle}`,
    sourceUrl ? "URL supplied" : "No URL supplied",
    sourceWords ? `Source words: ${sourceWords}` : "No source text captured",
    notesWords ? `Response notes words: ${notesWords}` : "No response-context notes yet",
    `Responder: ${company}`,
    `Buyer/client: ${client}`,
    `Owner: ${owner}`,
  ];
  const analyzedSignals = analysis
    ? [
        `Requirements: ${analysis.requirements.length}`,
        `Compliance rows: ${analysis.complianceRows.length}`,
        `Evidence checks: ${analysis.verificationSummary.rowsNeedingEvidence}`,
        `Stated intent: ${analysis.statedIntent.length}`,
        `Implied intent: ${analysis.impliedIntent.length}`,
        `Readiness score: ${analysis.completenessScore}/100`,
      ]
    : ["RFP not analyzed yet"];
  const rowPreview = analysis?.complianceRows.slice(0, 3).map((row) => `${row.id} -> ${row.responseSection}`).join("; ") || "No requirement rows yet";
  const requirementSummary = analysis
    ? `${analysis.requirements.length} extracted requirement(s), ${analysis.capabilities.length} capability hint(s), ${analysis.timelines.length} timeline hint(s), ${analysis.budgetHints.length} budget hint(s), and ${analysis.mandatoryAttachments.length} mandatory attachment hint(s).`
    : "Analyze the full source RFP before drafting so requirements, attachments, timelines, and budget hints are mapped.";

  const steps: Array<Omit<RfpWizardStepAssistance, "contextSignals"> & { extraSignals?: string[] }> = [
    {
      stepId: "source-intake",
      stepLabel: "Source intake",
      actionLabel: "Use intake guidance",
      suggestedAnswer: [
        `Use ${sourceTitle} as the authoritative buyer source for ${client}.`,
        input.sourceKind === "url" ? "Fetch the URL and confirm the fetched text includes all addenda, tables, and attachments before analysis." : "Confirm the imported or pasted text includes all addenda, exhibits, pricing forms, and submission instructions.",
        sourceWords < 400 ? "The source looks short; treat the analysis as incomplete until the full RFP package is captured." : "The source length is sufficient for a first requirement scan, but still verify appendices and attachments.",
      ].join(" "),
      rationale: "RFP response quality depends on complete source capture before any response prose is generated.",
      extraSignals: [sourceWords < 400 ? "Short-source warning" : "Source ready for analysis"],
    },
    {
      stepId: "requirement-analysis",
      stepLabel: "Requirement analysis",
      actionLabel: "Use analysis guidance",
      suggestedAnswer: [
        requirementSummary,
        "Every mandatory, scored, contractual, delivery, attachment, and format requirement should become a compliance row with owner, evidence needed, verification, and response section.",
      ].join(" "),
      rationale: "A responsive bid needs a traceable requirement inventory before the team writes polished narrative.",
      extraSignals: analyzedSignals,
    },
    {
      stepId: "buyer-intent",
      stepLabel: "Buyer intent",
      actionLabel: "Use intent guidance",
      suggestedAnswer: analysis
        ? [
            `Stated intent: ${analysis.statedIntent.slice(0, 3).join("; ") || "none detected"}.`,
            `Implied intent: ${analysis.impliedIntent.slice(0, 3).join("; ") || "none detected"}.`,
            "Use both stated and implied intent to shape the executive response, differentiators, risk posture, and compliance emphasis.",
          ].join(" ")
        : "After analysis, summarize both stated buyer goals and implied evaluator concerns before drafting the executive response.",
      rationale: "RFP evaluators score more than literal compliance; intent guidance keeps the response aligned to the buyer's outcome and constraints.",
      extraSignals: analysis ? analyzedSignals : [],
    },
    {
      stepId: "response-drafting",
      stepLabel: "Response drafting",
      actionLabel: "Use drafting guidance",
      suggestedAnswer: [
        `Draft the response section-by-section for ${company}, starting from the compliance matrix rather than generic proposal copy.`,
        `Use these first response-section routes: ${rowPreview}.`,
        "Keep suggested answers evidence-gated until the named owner attaches proof.",
      ].join(" "),
      rationale: "Drafting from mapped requirements prevents a persuasive but non-responsive proposal.",
      extraSignals: analysis ? [`Response sections: ${new Set(analysis.complianceRows.map((row) => row.responseSection)).size}`] : [],
    },
    {
      stepId: "evidence-qa",
      stepLabel: "Evidence QA",
      actionLabel: "Use QA guidance",
      suggestedAnswer: analysis
        ? [
            `${analysis.verificationSummary.rowsNeedingEvidence} row(s) still need evidence review.`,
            "Verify attachments, certifications, pricing assumptions, timeline commitments, risks, exceptions, and every suggested answer before submission.",
            analysis.questions.length ? `Open reviewer questions: ${analysis.questions.slice(0, 4).join("; ")}.` : "No open reviewer questions were generated, but require human sign-off before export.",
          ].join(" ")
        : "Run analysis first, then review evidence gaps, attachment obligations, exceptions, pricing assumptions, and reviewer questions before response generation.",
      rationale: "Evidence QA keeps the AI-generated response from overstating unsupported capabilities or missing mandatory instructions.",
      extraSignals: analysis ? analyzedSignals : [],
    },
    {
      stepId: "handoff-distribution",
      stepLabel: "Handoff and distribution",
      actionLabel: "Use handoff guidance",
      suggestedAnswer: [
        `Prepare a governed handoff for ${owner}: full source summary, compliance matrix, Requirement Response Drafts, evidence gaps, owner assignments, and submission checklist.`,
        "Use Docs Live for section-by-section drafting or Agent handoff for a local Claude Code, Codex, OpenCode, or Google Antigravity response pass; keep review blockers visible before export.",
      ].join(" "),
      rationale: "The final handoff must be actionable for business reviewers and local agents without losing compliance traceability.",
      extraSignals: analysis ? analyzedSignals : [],
    },
  ];

  return steps.map(({ extraSignals, ...step }) => ({
    ...step,
    contextSignals: Array.from(new Set([...baseSignals, ...(extraSignals || [])].filter(Boolean))),
  }));
}

export function analyzeRfpSource(input: RfpSourceInput, profile: Partial<BusinessProfile> = {}): RfpAnalysis {
  const normalizedProfile = businessProfilePlaceholderMap(profile);
  const title = normalizeWhitespace(input.title || input.url || "Imported RFP");
  const url = normalizeWhitespace(input.url || "");
  const normalizedText = normalizeRfpText(input.text);
  const lines = normalizedText.split(/\r?\n/);
  const significantLines = lines
    .map((line, index) => ({ line: normalizeWhitespace(line), index: index + 1 }))
    .filter((item) => item.line.length > 0);
  const requirements = dedupeRequirements(
    extractRfpRequirementCandidates(significantLines)
      .map((item, index) => buildRfpRequirement(item.line, item.index, index + 1, normalizedProfile)),
  );
  if (!requirements.length && normalizedText.trim()) {
    requirements.push(...fallbackRequirements(significantLines, normalizedProfile));
  }

  const timelines = extractMatchingLines(significantLines, /\b(deadline|due|schedule|timeline|milestone|weeks?|months?|days?|implementation|start date|go-live|submission)\b/i, 8);
  const budgetHints = extractMatchingLines(significantLines, /\b(budget|price|pricing|cost|fee|fees|commercial|payment|invoice|rate|rates|discount|value for money|\$|usd|eur|gbp|kes)\b/i, 8);
  const evaluationCriteria = extractMatchingLines(significantLines, /\b(evaluation|scor|weight|criteria|points|award|selection|rated|technical merit|best value)\b/i, 8);
  const scoringWeights = extractRfpScoringWeights(significantLines);
  const annexReferences = extractRfpAnnexReferences(significantLines);
  const bilingualRequirements = extractMatchingLines(significantLines, /\b(bilingual|english\s*\/\s*french|en\s*\/\s*fr|french|français|francais|translation|translated)\b/i, 10);
  const placeholderRisks = extractMatchingLines(significantLines, /\b(tbd|to be confirmed|to be assigned|placeholder|insert|fill in|not yet available|pending)\b/i, 10);
  const mandatoryAttachments = dedupeStrings([
    ...extractMatchingLines(significantLines, /\b(attachment|appendix|annex|annexure|form|certificate|insurance|tax|license|licence|registration|declaration|signature|signed|mandatory document|audited accounts|bank statements?)\b/i, 20),
    ...annexReferences.map((item) => item.requirement),
  ]).slice(0, 20);
  const capabilities = inferRfpCapabilities(requirements, normalizedText, normalizedProfile);
  const statedIntent = inferStatedRfpIntent(significantLines, requirements);
  const impliedIntent = inferImpliedRfpIntent(requirements, timelines, budgetHints, evaluationCriteria, mandatoryAttachments, normalizedProfile);
  const complianceRows = requirements.map(buildRfpComplianceRow);
  const complianceChecklist = extractRfpComplianceChecklist({
    complianceRows,
    mandatoryAttachments,
    scoringWeights,
    annexReferences,
    bilingualRequirements,
    placeholderRisks,
  });
  const criticalDisqualifiers = complianceChecklist
    .filter((item) => item.risk === "critical")
    .map((item) => `${item.id}: ${item.requirement}`);
  const verificationSummary = buildRfpVerificationSummary(requirements, complianceRows, mandatoryAttachments, evaluationCriteria, scoringWeights, annexReferences, bilingualRequirements, placeholderRisks);
  const proposalOutline = buildRfpProposalOutline({
    lines: significantLines,
    requirements,
    complianceRows,
    scoringWeights,
    annexReferences,
    bilingualRequirements,
    criticalDisqualifiers,
  });
  const risks = inferRfpRisks(requirements, timelines, budgetHints, mandatoryAttachments, criticalDisqualifiers, placeholderRisks);
  const questions = inferRfpQuestions(requirements, timelines, budgetHints, evaluationCriteria, mandatoryAttachments, normalizedProfile);
  const warnings = inferRfpWarnings(input, normalizedText, requirements);
  const completenessScore = Math.max(0, Math.min(100, Math.round(
    20 +
      Math.min(requirements.length, 12) * 4 +
      Math.min(capabilities.length, 6) * 3 +
      Math.min(timelines.length, 4) * 3 +
      Math.min(budgetHints.length, 4) * 3 +
      Math.min(evaluationCriteria.length, 4) * 3 +
      Math.min(scoringWeights.length, 5) * 2 +
      Math.min(mandatoryAttachments.length, 5) * 2 +
      Math.min(annexReferences.length, 5) * 2 -
      warnings.length * 5,
  )));

  return {
    source: {
      kind: input.kind,
      title,
      url,
      lineCount: significantLines.length,
      wordCount: normalizedText.split(/\s+/).filter(Boolean).length,
    },
    requirements,
    complianceRows,
    complianceChecklist,
    verificationSummary,
    capabilities,
    statedIntent,
    impliedIntent,
    timelines,
    budgetHints,
    evaluationCriteria,
    mandatoryAttachments,
    criticalDisqualifiers,
    scoringWeights,
    annexReferences,
    bilingualRequirements,
    placeholderRisks,
    proposalOutline,
    risks,
    questions,
    warnings,
    completenessScore,
  };
}

export function rfpComplianceMatrixMarkdown(analysis: RfpAnalysis) {
  const rows = analysis.complianceRows.length ? analysis.complianceRows : [];
  return [
    "## Compliance Matrix",
    "",
    "| ID | Type | Risk | Requirement | Category | Compliance status | Response section | Suggested response | Evidence / proof | Verification |",
    "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ...rows.map((row) => `| ${escapeMarkdownTableCell(row.id)} | ${escapeMarkdownTableCell(row.requirementType)} | ${row.disqualificationRisk ? "Disqualification risk" : "Standard review"} | ${escapeMarkdownTableCell(row.text)} | ${escapeMarkdownTableCell(row.category)} | ${escapeMarkdownTableCell(row.complianceStatus)} | ${escapeMarkdownTableCell(row.responseSection)} | ${escapeMarkdownTableCell(row.suggestedResponse)} | ${escapeMarkdownTableCell(row.evidenceNeeded)} | ${escapeMarkdownTableCell(row.verification)} |`),
    rows.length ? "" : "| RFP-REQ-001 | REQUIREMENT | Standard review | Paste or import the RFP text to populate requirements. | Intake | Needs evidence review | Requirements Analysis | Analyze the full source RFP before drafting a response. | Source RFP text | Not verified. |",
  ].join("\n");
}

export function rfpComplianceChecklistMarkdown(analysis: RfpAnalysis) {
  const rows = analysis.complianceChecklist.length ? analysis.complianceChecklist : [
    {
      id: "RFP-CHECK-001",
      section: "Intake",
      requirement: "Paste or import the full RFP source to populate the compliance checklist.",
      verification: "Re-run RFP analysis after source import and confirm all sections, annexes, and tables were captured.",
      reference: "RFP source",
      risk: "high" as RfpComplianceChecklistRisk,
      owner: "Bid Owner",
      sourceLine: 0,
    },
  ];
  return [
    "## Compliance Checklist",
    "",
    "### Critical Disqualification Traps",
    "",
    ...criticalChecklistRows(rows).map((row) => `- [ ] **${row.id}:** ${row.requirement} (${row.reference}) - ${row.verification}`),
    criticalChecklistRows(rows).length ? "" : "- [ ] No explicit automatic-exclusion wording detected; reviewer must still inspect the source RFP.",
    "",
    "### Full Checklist",
    "",
    "| ID | Section | Risk | Requirement | Verification method | Owner | Reference |",
    "| --- | --- | --- | --- | --- | --- | --- |",
    ...rows.map((row) => `| ${escapeMarkdownTableCell(row.id)} | ${escapeMarkdownTableCell(row.section)} | ${escapeMarkdownTableCell(row.risk)} | ${escapeMarkdownTableCell(row.requirement)} | ${escapeMarkdownTableCell(row.verification)} | ${escapeMarkdownTableCell(row.owner)} | ${escapeMarkdownTableCell(row.reference)} |`),
  ].join("\n");
}

export function rfpProposalOutlineMarkdown(analysis: RfpAnalysis, profile: Partial<BusinessProfile> = {}, responseNotes = "") {
  const placeholders = businessProfilePlaceholderMap(profile);
  const outline = analysis.proposalOutline;
  const notes = responseNotes.trim();
  const scoringRows = outline.scoringScheme.length
    ? outline.scoringScheme.map((item) => `| ${escapeMarkdownTableCell(item.criterion)} | ${item.weight}${item.unit} | ${subCriterionForScoringItem(item.criterion)} | ${subWeightForScoringItem(item)} |`)
    : ["| No explicit weights detected | Assumed equal | Major proposal sections | Equal split |"];
  const pageRows = outline.pageAllocations.map((item) => `| ${escapeMarkdownTableCell(item.section)} | ${escapeMarkdownTableCell(item.pages)} | ${escapeMarkdownTableCell(item.basis)} |`);
  const teamRows = outline.teamRequirements.length
    ? outline.teamRequirements.map((item) => `| ${escapeMarkdownTableCell(item.role)} | ${escapeMarkdownTableCell(item.minimumExperience)} | *Name or TBC* | Source line ${item.sourceLine} |`)
    : ["| Team Lead | *Confirm minimum requirements* | *Name or TBC* | RFP team scan |", "| Core Team | *Map roles to ToR activities* | *Names or TBC* | RFP team scan |"];
  const annexRows = outline.annexes.length
    ? outline.annexes.map((item) => `| ${escapeMarkdownTableCell(item.annex)} | ${escapeMarkdownTableCell(item.label || item.requirement)} | *Confirm page-limit treatment* | Source line ${item.sourceLine} |`)
    : ["| Annexes | *List all required forms and schedules* | *Confirm page-limit treatment* | Manual review |"];
  const passFailRows = outline.metadata.passFailCriteria.length
    ? outline.metadata.passFailCriteria.map((item, index) => `| PF-${String(index + 1).padStart(2, "0")} | ${escapeMarkdownTableCell(item)} | *Map to proposal section or signed annex* |`)
    : ["| PF-01 | No explicit automatic-exclusion wording detected | *Reviewer must inspect full RFP and add gates* |"];
  const methodologyPages = pageAllocationFor(outline, "Proposed Methodology & Technical Approach", "4-5 pages");
  const teamPages = pageAllocationFor(outline, "Team Organization & Key Personnel", "2-3 pages");
  const orgPages = pageAllocationFor(outline, "Organizational Capacity & Past Performance", "2-3 pages");
  const activities = outline.activities.length ? outline.activities : [{ label: "Activity 1 - RFP ToR activity", sourceLine: 0, placeholder: "Describe approach, tools, outputs" }];
  const deliverables = outline.deliverables.length ? outline.deliverables : [{ label: "Deliverable register", sourceLine: 0, placeholder: "List reports, prototypes, handover assets" }];
  const milestones = outline.timelineMilestones.length ? outline.timelineMilestones : [{ label: "Submission and delivery schedule", sourceLine: 0, placeholder: "Add deadline, milestones, approvals" }];
  const technicalMandates = outline.technicalMandates.length ? outline.technicalMandates : ["[CLARIFICATION NEEDED: confirm open-source, interoperability, data-format, hosting, and API mandates]"];
  const sustainability = outline.sustainabilityRequirements.length ? outline.sustainabilityRequirements : ["[CLARIFICATION NEEDED: confirm maintenance, handover, and post-project support obligations]"];
  const riskQaKpis = outline.riskQaKpiRequirements.length ? outline.riskQaKpiRequirements : ["[CLARIFICATION NEEDED: confirm QA, testing, validation, risk, and KPI obligations]"];
  return fillBusinessTemplate(
    [
      "---",
      `title: ${yamlScalar(`Technical Proposal Outline for ${placeholders.defaultClientName}`)}`,
      "status: outline",
      "documentType: RFP technical proposal outline",
      `company: ${yamlScalar(placeholders.companyName)}`,
      `preparedBy: ${yamlScalar(placeholders.fullName)}`,
      `rfpSource: ${yamlScalar(analysis.source.title)}`,
      analysis.source.url ? `rfpUrl: ${yamlScalar(analysis.source.url)}` : "",
      "toc: true",
      "---",
      "",
      `# Technical Proposal Outline for ${placeholders.defaultClientName}`,
      "",
      businessDocumentSnippets[0].body,
      "",
      rfpComplianceChecklistMarkdown(analysis),
      "",
      "[TOC]",
      "",
      "## 1. RFP Metadata",
      "",
      `- Submission deadline: ${outline.metadata.submissionDeadline}`,
      `- Page limit: ${outline.metadata.pageLimitSource}`,
      `- Currency: ${outline.metadata.currency}`,
      `- Evaluation model: ${outline.metadata.evaluationModel}`,
      `- Pass/fail criteria: ${outline.metadata.passFailCriteria.length || "None explicitly detected; manual confirmation required"}`,
      notes ? `- Bid context notes: ${notes}` : "",
      "",
      rfpProposalPlanningPromptMarkdown(analysis, responseNotes),
      "",
      "## 2. Scoring Scheme and Page Allocation",
      "",
      "| Criterion | Weight | Sub-criterion | Sub-weight |",
      "| --- | --- | --- | --- |",
      ...scoringRows,
      "",
      "| Proposal section | Suggested pages | Basis |",
      "| --- | --- | --- |",
      ...pageRows,
      "",
      "## 3. Mandatory Pass/Fail Gates",
      "",
      "| Gate | Requirement | Where addressed |",
      "| --- | --- | --- |",
      ...passFailRows,
      "",
      "## 4. Terms of Reference Map",
      "",
      "### 4.1 Activities",
      "",
      ...activities.map((item, index) => `- ${index + 1}. ${item.label} - *${item.placeholder}*`),
      "",
      "### 4.2 Deliverables",
      "",
      ...deliverables.map((item) => `- ${item.label} - *${item.placeholder}*`),
      "",
      "### 4.3 Timeline Milestones and Approval Periods",
      "",
      ...milestones.map((item) => `- ${item.label} - *${item.placeholder}*`),
      ...outline.approvalPeriods.map((item) => `- ${item.label} - *${item.placeholder}*`),
      "",
      "### 4.4 Required Annexes",
      "",
      "| Annex | Purpose | Page-limit treatment | Reference |",
      "| --- | --- | --- | --- |",
      ...annexRows,
      "",
      "## 5. Team Composition and Experience Requirements",
      "",
      "| Role | Minimum requirement | Proposed name | Reference |",
      "| --- | --- | --- | --- |",
      ...teamRows,
      "",
      "## 6. Technical Mandates",
      "",
      ...technicalMandates.map((item) => `- ${item}`),
      "",
      "## 7. Sustainability, Risk, QA, and KPI Signals",
      "",
      "### 7.1 Sustainability and Transition",
      "",
      ...sustainability.map((item) => `- ${item}`),
      "",
      "### 7.2 Risk, QA, Validation, and KPIs",
      "",
      ...riskQaKpis.map((item) => `- ${item}`),
      "",
      "## Technical Proposal Outline",
      "",
      `### 1. Executive Summary (${pageAllocationFor(outline, "Executive Summary", "0.5 page")})`,
      "- *High-level value proposition*",
      "- *Mandatory-gate coverage statement*",
      "- *Approach and differentiators*",
      "",
      `### 2. Assignment Understanding & Delivery Approach (${pageAllocationFor(outline, "Assignment Understanding & Delivery Approach", "1 page")})`,
      "- *Context and buyer challenges*",
      "- *Delivery principles and win themes*",
      "- *Stated and implied intent response*",
      "",
      `### 3. Proposed Methodology & Technical Approach (${methodologyPages})`,
      ...activities.map((item, index) => `#### 3.${index + 1} ${item.label}\n- *${item.placeholder}*`),
      `#### 3.${activities.length + 1} Technology & Standards Summary`,
      "- *Open-source, interoperability, APIs, data formats, hosting constraints*",
      "",
      `### 4. Work Plan & Timeline (${pageAllocationFor(outline, "Work Plan & Timeline", "1 page")})`,
      "- *Milestone table or Gantt summary*",
      "- *Approval periods and dependencies*",
      "- *Annex D reference*",
      "",
      `### 5. Team Organization & Key Personnel (${teamPages})`,
      "- *Team Lead mandatory evidence*",
      "- *Responsibility matrix: Role | Name/TBC | ToR activity | Key qualifications*",
      outline.teamRequirements.some((item) => /consortium|subcontract|partner/i.test(item.role)) ? "- *Consortium Management: roles, interfaces, accountability*" : "- *Specialist support and partner roles if applicable*",
      "",
      `### 6. Organizational Capacity & Past Performance (${orgPages})`,
      "- *Firm profile*",
      "- *Relevant project summaries*",
      "- *Financial capacity and references*",
      "",
      `### 7. Open Data, Licensing & Governance Statement (${pageAllocationFor(outline, "Open Data, Licensing & Governance Statement", "0.5-1 page")})`,
      "- *License proposals for data, code, documentation*",
      "- *Governance model and contributor workflow*",
      "",
      `### 8. Risk Management & Mitigation (${pageAllocationFor(outline, "Risk Management & Mitigation", "0.5 page")})`,
      "- *Top risks and mitigations table*",
      "",
      `### 9. Quality Assurance & Monitoring (${pageAllocationFor(outline, "Quality Assurance & Monitoring", "0.5 page")})`,
      "- *KPIs and acceptance checks*",
      "- *Testing, validation, feedback loops*",
      "",
      `### 10. Sustainability & Transition Plan (${pageAllocationFor(outline, "Sustainability & Transition Plan", "0.5-1 page")})`,
      "- *Hosting, maintenance, post-project support*",
      "- *Handover deliverables and runbooks*",
      "",
      `### 11. Compliance Summary Table (${pageAllocationFor(outline, "Compliance Summary Table", "1 page")})`,
      "- *Map each mandatory requirement to section or annex*",
      "",
      "### 12. Required Annexes (not counted unless RFP says otherwise)",
      ...outline.annexes.map((item) => `- ${item.annex} - *${item.label || "Confirm title and format"}*`),
      outline.annexes.length ? "" : "- *Annex B/C/D/E/F/H and any buyer forms after source confirmation*",
      "",
      "## Critical Disqualifiers Checklist",
      "",
      ...criticalChecklistRows(analysis.complianceChecklist).map((item) => `- [ ] ${item.id}: ${item.requirement} - *${item.verification}*`),
      criticalChecklistRows(analysis.complianceChecklist).length ? "" : "- [ ] No explicit automatic-exclusion wording detected - *manual source review required*",
      "",
      "<!-- ai-assisted: status=outline-needs-approval | source=NEditor RFP Proposal Outline Wizard | promptSummary=Extract metadata, scoring, pass/fail gates, ToR, team, technical mandates, sustainability, risk, QA, and page-aware proposal outline before section drafting -->",
    ].filter(Boolean).join("\n"),
    profile,
  );
}

export function rfpProposalOutlineBullets(analysis: RfpAnalysis) {
  const outline = analysis.proposalOutline;
  const activities = outline.activities.length ? outline.activities : [{ label: "Activity 1 - RFP ToR activity", sourceLine: 0, placeholder: "Describe approach, tools, outputs" }];
  return [
    "- Executive Summary",
    "- Assignment Understanding & Delivery Approach",
    "- Proposed Methodology & Technical Approach",
    ...activities.map((item) => `  - ${item.label}`),
    "  - Technology & Standards Summary",
    "- Work Plan & Timeline",
    "- Team Organization & Key Personnel",
    "  - Team Lead",
    "  - Core Team",
    "  - Specialist Support / Consortium Partners",
    "- Organizational Capacity & Past Performance",
    "- Open Data, Licensing & Governance Statement",
    "- Risk Management & Mitigation",
    "- Quality Assurance & Monitoring",
    "- Sustainability & Transition Plan",
    "- Compliance Summary Table",
    "- Required Annexes",
    "- Critical Disqualifiers Checklist",
  ].join("\n");
}

function rfpProposalPlanningPromptMarkdown(analysis: RfpAnalysis, responseNotes = "") {
  const outline = analysis.proposalOutline;
  const scoring = outline.scoringScheme.length
    ? outline.scoringScheme.map((item) => `${item.criterion} (${item.weight}${item.unit})`).join("; ")
    : "No explicit scoring weights detected; infer equal review emphasis until the RFP is confirmed.";
  const passFail = outline.metadata.passFailCriteria.length
    ? outline.metadata.passFailCriteria.slice(0, 6).join("; ")
    : "No explicit pass/fail gates detected; reviewer must inspect mandatory language, submission rules, and annexes.";
  const torSignals = [
    outline.activities.length ? `${outline.activities.length} ToR activity hint(s)` : "no explicit ToR activities detected",
    outline.deliverables.length ? `${outline.deliverables.length} deliverable hint(s)` : "no explicit deliverables detected",
    outline.timelineMilestones.length ? `${outline.timelineMilestones.length} timeline hint(s)` : "no explicit timeline milestones detected",
  ].join("; ");
  const notes = responseNotes.trim();
  return [
    "## Proposal Planning Prompt",
    "",
    "Use this evaluator-driven planning prompt before drafting response prose:",
    "",
    `- Extract the evaluator model, scoring weights, sub-criteria, and likely reviewer evidence checks; mirror these signals in headings, proof points, and the compliance summary: ${scoring}`,
    `- Treat pass/fail gates as hard blockers: ${passFail}`,
    `- Build the Terms of Reference map from activities, deliverables, milestones, approval periods, and annexes: ${torSignals}.`,
    "- Convert team and experience requirements into a role matrix with proposed names, minimum credentials, CV/portfolio evidence, language coverage, and ToR responsibility.",
    `- Turn technical mandates into section requirements: ${markdownInlineList(outline.technicalMandates, "technical standards, integrations, hosting, APIs, data formats, licensing, and interoperability")}.`,
    `- Turn sustainability, transition, maintenance, handover, and support requirements into an operating model: ${markdownInlineList(outline.sustainabilityRequirements, "sustainability, transition, maintenance, handover, and support")}.`,
    `- Turn risk, QA, validation, monitoring, acceptance, and KPI language into controls and reviewer checks: ${markdownInlineList(outline.riskQaKpiRequirements, "risk, QA, validation, monitoring, acceptance, and KPI controls")}.`,
    "- Draft sections sequentially only after the checklist and outline are reviewed; leave evidence gaps as visible placeholders instead of unsupported claims.",
    notes ? `- Apply bid-team context notes while preserving RFP traceability: ${notes}` : "",
  ].filter(Boolean).join("\n");
}

function rfpEvaluatorAlignedSectionDraftsMarkdown(analysis: RfpAnalysis, profile: Partial<BusinessProfile> = {}, responseNotes = "") {
  const placeholders = businessProfilePlaceholderMap(profile);
  const outline = analysis.proposalOutline;
  const activities = outline.activities.length ? outline.activities : [{ label: "Primary ToR activity", sourceLine: 0, placeholder: "Describe the approach, outputs, acceptance checks, and evidence." }];
  const teamRows = outline.teamRequirements.length
    ? outline.teamRequirements.map((item) => `| ${escapeMarkdownTableCell(item.role)} | ${escapeMarkdownTableCell(item.minimumExperience)} | *Name/TBC* | Source line ${item.sourceLine || "manual review"} |`)
    : ["| Team Lead | *Confirm minimum requirements from RFP* | *Name/TBC* | Manual review |", "| Core delivery team | *Map skills to ToR activities* | *Names/TBC* | Manual review |"];
  const technicalMandates = outline.technicalMandates.length ? outline.technicalMandates : ["Confirm technical standards, data formats, integrations, hosting, API, licensing, and interoperability requirements."];
  const sustainability = outline.sustainabilityRequirements.length ? outline.sustainabilityRequirements : ["Confirm maintenance, transition, handover, training, support, and post-project operating obligations."];
  const riskQaKpis = outline.riskQaKpiRequirements.length ? outline.riskQaKpiRequirements : ["Confirm risk controls, QA plan, validation method, monitoring cadence, KPIs, and acceptance checks."];
  const scoringBullets = outline.scoringScheme.length
    ? outline.scoringScheme.map((item) => `- ${item.criterion}: address ${item.weight}${item.unit} through named evidence, section labels, and reviewer checks.`)
    : ["- No explicit scoring weights were detected; use equal emphasis across compliance, technical approach, team, delivery, risk, sustainability, and commercial clarity."];
  const criticalRows = criticalChecklistRows(analysis.complianceChecklist);
  const notes = responseNotes.trim();
  return fillBusinessTemplate(
    [
      "## Evaluator-Aligned Section Drafts",
      "",
      "These draft sections are generated from the compliance checklist, scoring signals, pass/fail gates, Terms of Reference map, team requirements, technical mandates, sustainability obligations, and risk/QA/KPI signals. They remain evidence-gated until owners attach proof.",
      "",
      "### Executive Summary Draft",
      "",
      `${placeholders.companyName} will respond to ${placeholders.defaultClientName} with a compliance-first, evaluator-readable proposal. The response will show how each mandatory requirement is met, how scored criteria are answered with evidence, and where reviewer sign-off is still required before submission.`,
      "",
      "Evaluator priorities to mirror:",
      ...scoringBullets,
      criticalRows.length ? `- Pass/fail gates to clear first: ${criticalRows.map((item) => item.id).join(", ")}.` : "- No explicit automatic-exclusion gate was detected; bid reviewers must confirm mandatory submission language manually.",
      notes ? `- Bid-team emphasis: ${notes}` : "",
      "",
      "### Assignment Understanding and ToR Response Draft",
      "",
      "The response should frame the buyer's stated outcomes and implied evaluator concerns, then map each ToR activity to a method, output, owner, acceptance check, and evidence reference.",
      "",
      ...activities.flatMap((item, index) => [
        `#### ToR Activity ${index + 1}: ${item.label}`,
        "",
        `Draft response: ${placeholders.companyName} will address this activity through a structured work package covering ${item.placeholder.toLowerCase()}. Evidence and assumptions should cite source line ${item.sourceLine || "manual review"} and remain open until the delivery owner confirms feasibility.`,
        "",
      ]),
      "### Technical Methodology Draft",
      "",
      "Technical requirements to cover explicitly:",
      ...technicalMandates.map((item) => `- ${item}`),
      "",
      "Draft response: The methodology should convert each technical mandate into an implementation choice, integration pattern, data or documentation standard, test evidence, and acceptance criterion. Any mandate that cannot be confirmed from the source should remain a clarification item.",
      "",
      "### Team and Experience Draft",
      "",
      "| Role | Minimum requirement | Proposed name | Evidence reference |",
      "| --- | --- | --- | --- |",
      ...teamRows,
      "",
      "Draft response: The team section should link each named role to ToR activities, relevant credentials, comparable work, language or local-knowledge requirements, and CV/reference evidence.",
      "",
      "### Sustainability and Transition Draft",
      "",
      ...sustainability.map((item) => `- ${item}`),
      "",
      "Draft response: The sustainability plan should explain maintenance, handover, knowledge transfer, operational ownership, support model, and post-project continuity without promising unsupported capacity.",
      "",
      "### Risk, QA, Validation, and KPI Draft",
      "",
      ...riskQaKpis.map((item) => `- ${item}`),
      "",
      "Draft response: The risk and QA section should list top delivery, compliance, schedule, commercial, and technical risks; assign mitigations and owners; and show how KPIs, acceptance checks, and validation evidence will be reviewed.",
      "",
      "### Compliance Summary Draft",
      "",
      "The compliance summary should point reviewers back to the front-of-document checklist and compliance matrix. It must show requirement ID, response section, evidence owner, verification method, and unresolved proof gaps for every extracted requirement.",
    ].filter(Boolean).join("\n"),
    profile,
  );
}

export function rfpResponseMarkdown(analysis: RfpAnalysis, profile: Partial<BusinessProfile> = {}, responseNotes = "") {
  const placeholders = businessProfilePlaceholderMap(profile);
  const title = `RFP response for ${placeholders.defaultClientName}`;
  const requirementBullets = analysis.requirements.map((requirement) => `- **${requirement.id}:** ${requirement.text}`).join("\n") || "- No requirements were extracted. Import or paste the RFP text and re-run analysis.";
  const statedIntentBullets = markdownBullets(analysis.statedIntent, "Add stated buyer intent from the RFP overview, purpose, objectives, scope, and award language.");
  const impliedIntentBullets = markdownBullets(analysis.impliedIntent, "Infer unstated priorities from criteria, mandatory evidence, timeline pressure, budget language, and risk signals.");
  const capabilityBullets = markdownBullets(analysis.capabilities, "Add capability evidence.");
  const timelineBullets = markdownBullets(analysis.timelines, "Add the buyer deadline, milestones, and submission time zone.");
  const budgetBullets = markdownBullets(analysis.budgetHints, "Add the pricing basis, budget ceiling, required forms, and assumptions.");
  const criteriaBullets = markdownBullets(analysis.evaluationCriteria, "Add the evaluation criteria and scoring weights.");
  const attachmentBullets = markdownBullets(analysis.mandatoryAttachments, "Add mandatory forms, certificates, declarations, and signatures.");
  const riskBullets = markdownBullets(analysis.risks, "Review source RFP for risks, exceptions, and buyer constraints.");
  const questionBullets = markdownBullets(analysis.questions, "No open questions detected.");
  const verificationBullets = analysis.verificationSummary.checklist.map((item) => `- [ ] ${item}`).join("\n");
  const responseDrafts = rfpRequirementResponseDraftsMarkdown(analysis.complianceRows);
  const scoringBullets = analysis.scoringWeights.length
    ? analysis.scoringWeights.map((item) => `- ${item.criterion}: ${item.weight}${item.unit} (source line ${item.sourceLine})`).join("\n")
    : "- Add scoring weights after reviewing the evaluation section.";
  const notes = responseNotes.trim();
  return fillBusinessTemplate(
    [
      "---",
      `title: ${yamlScalar(title)}`,
      "status: draft",
      "documentType: RFP response",
      `company: ${yamlScalar(placeholders.companyName)}`,
      `preparedBy: ${yamlScalar(placeholders.fullName)}`,
      `rfpSource: ${yamlScalar(analysis.source.title)}`,
      analysis.source.url ? `rfpUrl: ${yamlScalar(analysis.source.url)}` : "",
      "toc: true",
      "---",
      "",
      `# ${title}`,
      "",
      businessDocumentSnippets[0].body,
      "",
      rfpComplianceChecklistMarkdown(analysis),
      "",
      "[TOC]",
      "",
      rfpProposalPlanningPromptMarkdown(analysis, responseNotes),
      "",
      "## Proposal Outline",
      "",
      rfpProposalOutlineBullets(analysis),
      "",
      rfpEvaluatorAlignedSectionDraftsMarkdown(analysis, profile, responseNotes),
      "",
      "## Executive Response",
      "",
      `${placeholders.companyName} has prepared a fully responsive draft for ${placeholders.defaultClientName}. This response mirrors the RFP requirements, maps each requirement into a compliance matrix, and flags the evidence that must be confirmed before submission.`,
      "",
      "## RFP Intake Summary",
      "",
      `- Source type: ${analysis.source.kind.toUpperCase()}`,
      `- Source title: ${analysis.source.title}`,
      analysis.source.url ? `- Source URL: ${analysis.source.url}` : "",
      `- Extracted requirements: ${analysis.requirements.length}`,
      `- Completeness score: ${analysis.completenessScore}/100`,
      `- Source size: ${analysis.source.wordCount} words across ${analysis.source.lineCount} non-empty lines`,
      "",
      notes ? "### Response Context and Decision Notes" : "",
      notes,
      notes ? "" : "",
      "## Requirements Analysis",
      "",
      requirementBullets,
      "",
      "## Buyer Intent Analysis",
      "",
      "### Stated Intent",
      "",
      statedIntentBullets,
      "",
      "### Implied Intent",
      "",
      impliedIntentBullets,
      "",
      rfpComplianceMatrixMarkdown(analysis),
      "",
      responseDrafts,
      "",
      "## Requirement Verification",
      "",
      verificationBullets || "- [ ] Re-run RFP analysis after importing the full source.",
      "",
      "### Requirement-Level Checks",
      "",
      ...analysis.complianceRows.flatMap((row) => [
        `#### ${row.id}: ${row.category}`,
        "",
        `Suggested response: ${row.suggestedResponse}`,
        "",
        ...row.verificationChecklist.map((item) => `- [ ] ${item}`),
        "",
      ]),
      "## Capability Match",
      "",
      capabilityBullets,
      "",
      "## Proposed Solution",
      "",
      "Our response is organized around the buyer's stated outcomes, mandatory requirements, evaluation criteria, and delivery constraints. Each requirement has a drafted response path in the Requirement Response Drafts section, an evidence placeholder, and a reviewer verification note.",
      "",
      "## Implementation Plan and Timeline",
      "",
      timelineBullets,
      "",
      "## Pricing and Budget Response",
      "",
      budgetBullets,
      "",
      "## Evaluation Criteria Response",
      "",
      criteriaBullets,
      "",
      "### Scoring Weights",
      "",
      scoringBullets,
      "",
      "## Mandatory Attachments",
      "",
      attachmentBullets,
      "",
      "## Risk and Assumptions",
      "",
      riskBullets,
      "",
      "## Open Questions for Buyer or Bid Team",
      "",
      questionBullets,
      "",
      "## Submission QA Checklist",
      "",
      "- [ ] Every RFP requirement appears in the compliance matrix.",
      "- [ ] Every matrix row has a response section and evidence owner.",
      "- [ ] Mandatory forms, certificates, declarations, and signatures are attached.",
      "- [ ] Pricing matches the required format and stated assumptions.",
      "- [ ] Timeline, delivery milestones, and submission deadline are confirmed.",
      "- [ ] Legal, finance, and delivery reviewers have approved the final response.",
      "",
      "<!-- ai-assisted: status=needs-review | source=NEditor RFP Response Wizard | promptSummary=Analyze RFP, build compliance matrix, draft responsive response -->",
    ].filter(Boolean).join("\n"),
    profile,
  );
}

function rfpRequirementResponseDraftsMarkdown(rows: RfpComplianceRow[]) {
  if (!rows.length) {
    return [
      "## Requirement Response Drafts",
      "",
      "Import or paste the RFP source to generate requirement-by-requirement response drafts.",
    ].join("\n");
  }
  const grouped = new Map<string, RfpComplianceRow[]>();
  for (const row of rows) {
    const bucket = grouped.get(row.responseSection) || [];
    bucket.push(row);
    grouped.set(row.responseSection, bucket);
  }
  return [
    "## Requirement Response Drafts",
    "",
    "These draft answers are generated from the compliance matrix and must remain evidence-gated until the named owner attaches proof and a reviewer signs off.",
    "",
    ...Array.from(grouped.entries()).flatMap(([section, sectionRows]) => [
      `### ${section}`,
      "",
      ...sectionRows.flatMap((row) => [
        `#### ${row.id}: ${row.category}`,
        "",
        row.suggestedResponse,
        "",
        `- Requirement: ${row.text}`,
        `- Evidence owner: ${row.owner}`,
        `- Evidence needed: ${row.evidenceNeeded}`,
        `- Verification: ${row.verification}`,
        "",
      ]),
    ]),
  ].join("\n");
}

function criticalChecklistRows(rows: RfpComplianceChecklistItem[]) {
  return rows.filter((row) => row.risk === "critical");
}

function extractRfpComplianceChecklist(input: {
  complianceRows: RfpComplianceRow[];
  mandatoryAttachments: string[];
  scoringWeights: RfpScoringWeight[];
  annexReferences: RfpAnnexReference[];
  bilingualRequirements: string[];
  placeholderRisks: string[];
}): RfpComplianceChecklistItem[] {
  const rows: RfpComplianceChecklistItem[] = [];
  const seen = new Set<string>();
  const push = (item: Omit<RfpComplianceChecklistItem, "id">) => {
    const key = `${item.section}|${item.requirement}|${item.reference}`.toLowerCase().replace(/[^a-z0-9|]+/g, " ").trim();
    if (!key || seen.has(key)) return;
    seen.add(key);
    rows.push({ ...item, id: `RFP-CHECK-${String(rows.length + 1).padStart(3, "0")}` });
  };

  for (const row of input.complianceRows) {
    push({
      section: row.disqualificationRisk ? "Critical disqualification traps" : row.responseSection,
      requirement: row.text,
      verification: row.verification,
      reference: `Source line ${row.sourceLine}`,
      risk: row.disqualificationRisk ? "critical" : row.complianceStatus === "Needs evidence review" || row.requirementType !== "REQUIREMENT" ? "high" : "standard",
      owner: row.owner,
      sourceLine: row.sourceLine,
    });
  }

  for (const attachment of input.mandatoryAttachments) {
    push({
      section: "Document checklist - attachments required",
      requirement: `Include and verify mandatory attachment: ${attachment}`,
      verification: "Confirm the named attachment is complete, signed where required, current, and included in the final submission package.",
      reference: "Attachment / annex scan",
      risk: /will be rejected|automatic|disqualif|failure to|must|shall|required|mandatory/i.test(attachment) ? "critical" : "high",
      owner: "Bid Coordinator",
      sourceLine: 0,
    });
  }

  for (const annex of input.annexReferences) {
    push({
      section: "Annex references",
      requirement: annex.requirement,
      verification: `Locate ${annex.annex} in the source package, confirm required format/signature, and attach it to the response bundle.`,
      reference: `Source line ${annex.sourceLine}`,
      risk: "high",
      owner: "Bid Coordinator",
      sourceLine: annex.sourceLine,
    });
  }

  for (const item of input.scoringWeights) {
    push({
      section: "Scored criteria and win themes",
      requirement: `Address scored criterion "${item.criterion}" worth ${item.weight}${item.unit}.`,
      verification: "Mirror this criterion in the executive response, section heading, proof point, and reviewer scoring check.",
      reference: `Source line ${item.sourceLine}`,
      risk: "high",
      owner: "Bid Manager",
      sourceLine: item.sourceLine,
    });
  }

  for (const requirement of input.bilingualRequirements) {
    push({
      section: "Bilingual / language obligations",
      requirement,
      verification: "Confirm French, English/French, translation, workshop, or training-material capability is explicitly covered and owned.",
      reference: "Language requirement scan",
      risk: "high",
      owner: "Delivery Lead",
      sourceLine: 0,
    });
  }

  for (const placeholder of input.placeholderRisks) {
    push({
      section: "Placeholder and response-readiness traps",
      requirement: `Resolve non-final placeholder language before submission: ${placeholder}`,
      verification: "Search the response for TBD, placeholder, to be assigned, to be confirmed, and similar markers; replace or explicitly escalate before final packaging.",
      reference: "Placeholder scan",
      risk: "high",
      owner: "Bid Owner",
      sourceLine: 0,
    });
  }

  return rows;
}

function buildRfpProposalOutline(input: {
  lines: Array<{ line: string; index: number }>;
  requirements: RfpRequirement[];
  complianceRows: RfpComplianceRow[];
  scoringWeights: RfpScoringWeight[];
  annexReferences: RfpAnnexReference[];
  bilingualRequirements: string[];
  criticalDisqualifiers: string[];
}): RfpProposalOutline {
  const pageLimit = extractRfpPageLimit(input.lines);
  const metadata: RfpProposalMetadata = {
    submissionDeadline: extractRfpSubmissionDeadline(input.lines),
    pageLimit: pageLimit.pages,
    pageLimitSource: pageLimit.source,
    currency: extractRfpCurrency(input.lines),
    evaluationModel: inferRfpEvaluationModel(input.lines, input.scoringWeights),
    passFailCriteria: input.criticalDisqualifiers.length
      ? input.criticalDisqualifiers
      : input.complianceRows.filter((row) => row.disqualificationRisk || row.requirementType === "MANDATORY").slice(0, 10).map((row) => `${row.id}: ${row.text}`),
  };
  const activities = extractRfpOutlineItems(
    input.lines,
    /\b(activity|task|work package|workstream|phase)\s*(\d+|[ivx]+|[a-z])?\b/i,
    "Describe approach, tools, outputs",
    10,
  );
  const deliverables = extractRfpOutlineItems(
    input.lines,
    /\b(deliverable|output|report|prototype|platform|manual|training material|workshop|source code|handover)\b/i,
    "Define output, acceptance criteria, owner, and evidence",
    12,
  );
  const timelineMilestones = extractRfpOutlineItems(
    input.lines,
    /\b(month\s*\d+|week\s*\d+|\d+\s*(?:days?|weeks?|months?)|milestone|deadline|submission|inception|prototype|final)\b/i,
    "Map milestone, dependency, approval date, and responsible role",
    12,
  );
  const approvalPeriods = extractRfpOutlineItems(
    input.lines,
    /\b(approval|review period|acceptance|client review|steering committee|sign[- ]off)\b/i,
    "Show review window, decision owner, dependency, and fallback",
    8,
  );
  const teamRequirements = extractRfpTeamRequirements(input.lines, input.bilingualRequirements);
  const technicalMandates = extractMatchingLines(
    input.lines,
    /\b(open[- ]source|license|licence|interoperab|standard|api|ogc|wms|wfs|openapi|netcdf|geojson|postgis|postgres|python|react|cloud|on[- ]prem|hosting|integration|data format|source code|repository)\b/i,
    14,
  );
  const sustainabilityRequirements = extractMatchingLines(
    input.lines,
    /\b(sustainab|maintenance|maintain|roadmap|handover|transition|post[- ]project|support|operations|runbook|knowledge transfer|capacity building)\b/i,
    10,
  );
  const riskQaKpiRequirements = extractMatchingLines(
    input.lines,
    /\b(risk|mitigation|quality assurance|qa\b|testing|validation|pilot|kpi|key performance|monitoring|completion rate|trained|users|acceptance criteria)\b/i,
    12,
  );
  return {
    metadata,
    scoringScheme: input.scoringWeights,
    activities,
    deliverables,
    timelineMilestones,
    approvalPeriods,
    annexes: input.annexReferences,
    teamRequirements,
    technicalMandates,
    sustainabilityRequirements,
    riskQaKpiRequirements,
    pageAllocations: buildRfpPageAllocations(pageLimit.pages, input.scoringWeights),
  };
}

function extractRfpSubmissionDeadline(lines: Array<{ line: string; index: number }>) {
  const deadlineLine = lines.find((item) => /\b(submission deadline|deadline for submission|due no later than|closing date|close date|proposals? due|submit.*by)\b/i.test(item.line));
  if (!deadlineLine) return "Not specified - confirm deadline, time, and time zone";
  const dateMatch = deadlineLine.line.match(/\b(\d{1,2}(?:st|nd|rd|th)?\s+[A-Za-z]+\s+\d{4}|[A-Za-z]+\s+\d{1,2},?\s+\d{4}|\d{4}[-/]\d{1,2}[-/]\d{1,2}|\d{1,2}[-/]\d{1,2}[-/]\d{2,4})(?:[,\s]+(?:at\s*)?(\d{1,2}:\d{2}\s*(?:am|pm|AM|PM)?|\d{1,2}\s*(?:am|pm|AM|PM)))?/);
  return dateMatch ? `${dateMatch[0]} (source line ${deadlineLine.index})` : `${stripRequirementPrefix(deadlineLine.line)} (source line ${deadlineLine.index})`;
}

function extractRfpPageLimit(lines: Array<{ line: string; index: number }>) {
  for (const item of lines) {
    const match = item.line.match(/\b(?:maximum|limit(?:ed)? to|not exceed|no more than)\s+(\d{1,3})\s+pages?\b/i) || item.line.match(/\b(\d{1,3})\s+page\s+(?:limit|maximum)\b/i);
    if (match) {
      const pages = Number(match[1]);
      if (Number.isFinite(pages) && pages > 0) return { pages, source: `${pages} pages (source line ${item.index})` };
    }
  }
  return { pages: 15, source: "Not specified - assume 15 pages" };
}

function extractRfpCurrency(lines: Array<{ line: string; index: number }>) {
  const currencyLine = lines.find((item) => /\b(currency|financial proposal|pricing|price|cost|budget|usd|eur|gbp|kes|cad|aud|\$|€|£)\b/i.test(item.line));
  if (!currencyLine) return "Not specified";
  const symbol = currencyLine.line.match(/\b(USD|EUR|GBP|KES|CAD|AUD|ZAR|CHF|JPY)\b|[$€£]/i)?.[0];
  if (!symbol) return `Not explicit; pricing line ${currencyLine.index} needs review`;
  const map: Record<string, string> = { "$": "USD or local dollar - confirm", "€": "EUR", "£": "GBP" };
  return map[symbol] || symbol.toUpperCase();
}

function inferRfpEvaluationModel(lines: Array<{ line: string; index: number }>, scoringWeights: RfpScoringWeight[]) {
  const explicit = lines.find((item) => /\b(QCBS|quality and cost|least cost|LCS|fixed budget|FBS|best value|technical and financial|quality[- ]based)\b/i.test(item.line));
  if (explicit) return `${stripRequirementPrefix(explicit.line)} (source line ${explicit.index})`;
  const technical = scoringWeights.filter((item) => /technical|methodology|team|experience|approach|quality/i.test(item.criterion)).reduce((sum, item) => sum + item.weight, 0);
  const cost = scoringWeights.filter((item) => /price|cost|financial|commercial/i.test(item.criterion)).reduce((sum, item) => sum + item.weight, 0);
  if (technical && cost) return `Inferred QCBS / technical-financial scoring (${technical}:${cost} by detected weights)`;
  if (technical) return "Inferred quality-based evaluation from technical scoring weights";
  return "Not explicit - infer after confirming evaluation weights";
}

function extractRfpOutlineItems(lines: Array<{ line: string; index: number }>, pattern: RegExp, placeholder: string, limit: number): RfpProposalActivity[] {
  const seen = new Set<string>();
  const items: RfpProposalActivity[] = [];
  for (const item of lines) {
    if (!pattern.test(item.line)) continue;
    const label = stripRequirementPrefix(item.line).replace(/\s+/g, " ").trim();
    const key = label.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().slice(0, 120);
    if (!key || seen.has(key)) continue;
    seen.add(key);
    items.push({ label, sourceLine: item.index, placeholder });
    if (items.length >= limit) break;
  }
  return items;
}

function extractRfpTeamRequirements(lines: Array<{ line: string; index: number }>, bilingualRequirements: string[]) {
  const teamLines = extractRfpOutlineItems(
    lines,
    /\b(team lead|project manager|architect|expert|specialist|analyst|engineer|consultant|personnel|staff|role|cv|curriculum|experience|years?|degree|certification|multidisciplinary|consortium|subcontract)\b/i,
    "Map role to ToR activity and attach CV evidence",
    14,
  );
  const output: RfpProposalTeamRequirement[] = teamLines.map((item) => {
    const role = normalizeWhitespace(item.label.match(/\b([A-Z][A-Za-z /&-]*(?:Lead|Manager|Architect|Expert|Specialist|Analyst|Engineer|Consultant|Team|Consortium|Subcontractor))\b/)?.[1] || item.label.split(/[;:|.]/)[0] || "Required role");
    const minimumExperience = item.label.match(/\b(\d+\+?\s+years?[^.;|]*)/i)?.[1] || (/\bdegree|certif|minimum|required|must|shall/i.test(item.label) ? item.label : "Confirm minimum experience and credentials");
    return { role, minimumExperience: normalizeWhitespace(minimumExperience), sourceLine: item.sourceLine };
  });
  if (bilingualRequirements.length && !output.some((item) => /language|bilingual|french/i.test(item.role))) {
    output.push({ role: "Bilingual EN/FR delivery capability", minimumExperience: "Confirm French/English workshop, training-material, and review coverage", sourceLine: 0 });
  }
  return output.slice(0, 14);
}

function buildRfpPageAllocations(totalPages: number, weights: RfpScoringWeight[]) {
  const base = [
    { section: "Executive Summary", pages: "0.5", basis: "Front matter and win theme" },
    { section: "Assignment Understanding & Delivery Approach", pages: "1", basis: "Buyer context and stated/implied intent" },
    { section: "Proposed Methodology & Technical Approach", pages: "4-5", basis: "Default largest technical scoring section" },
    { section: "Work Plan & Timeline", pages: "1", basis: "Milestones, approvals, and Gantt reference" },
    { section: "Team Organization & Key Personnel", pages: "2-3", basis: "Default team and CV scoring section" },
    { section: "Organizational Capacity & Past Performance", pages: "2-3", basis: "Default experience and firm capability section" },
    { section: "Open Data, Licensing & Governance Statement", pages: "0.5-1", basis: "Technical mandate coverage" },
    { section: "Risk Management & Mitigation", pages: "0.5", basis: "Risk controls" },
    { section: "Quality Assurance & Monitoring", pages: "0.5", basis: "QA and KPI coverage" },
    { section: "Sustainability & Transition Plan", pages: "0.5-1", basis: "Handover and operating model" },
    { section: "Compliance Summary Table", pages: "1", basis: "Mandatory requirement traceability" },
  ];
  if (!weights.length) return base.map((item) => ({ ...item, basis: `${item.basis}; no explicit weights, ${totalPages}-page assumed cap` }));
  return base.map((item) => {
    const matchingWeight = weights.find((weight) => sectionMatchesWeight(item.section, weight.criterion));
    if (!matchingWeight) return item;
    const pages = Math.max(0.5, Math.round(totalPages * (matchingWeight.weight / 100) * 2) / 2);
    return { ...item, pages: `${pages}`, basis: `${matchingWeight.weight}${matchingWeight.unit} detected for ${matchingWeight.criterion}` };
  });
}

function sectionMatchesWeight(section: string, criterion: string) {
  const haystack = `${section} ${criterion}`.toLowerCase();
  if (/method|approach|technical/.test(haystack) && /method|approach|technical|solution/.test(criterion.toLowerCase())) return true;
  if (/team|personnel|key/.test(haystack) && /team|personnel|key|staff|experience/.test(criterion.toLowerCase())) return true;
  if (/capacity|past performance|experience/.test(haystack) && /experience|past|capacity|reference/.test(criterion.toLowerCase())) return true;
  if (/work plan|timeline/.test(haystack) && /work plan|timeline|schedule/.test(criterion.toLowerCase())) return true;
  return false;
}

function pageAllocationFor(outline: RfpProposalOutline, section: string, fallback: string) {
  return outline.pageAllocations.find((item) => item.section === section)?.pages || fallback;
}

function subCriterionForScoringItem(criterion: string) {
  const clean = normalizeWhitespace(criterion);
  if (/\bmethod|approach|technical\b/i.test(clean)) return "*Understanding, methodology, work plan*";
  if (/\bteam|personnel|lead\b/i.test(clean)) return "*Team Lead, role fit, CV evidence*";
  if (/\bexperience|past|capacity\b/i.test(clean)) return "*Relevant projects, references, firm capacity*";
  if (/\bprice|cost|financial\b/i.test(clean)) return "*Financial proposal; separate response if required*";
  return "*Confirm sub-criteria in RFP*";
}

function subWeightForScoringItem(item: RfpScoringWeight) {
  if (item.weight >= 15) return "*Split across detected sub-criteria*";
  return `${item.weight}${item.unit}`;
}

function sectionPromptForHeading(heading: string) {
  return `<!-- Draft ${heading.toLowerCase()} with concrete evidence, unresolved placeholders, and review notes. -->`;
}

function yamlScalar(value: string) {
  const clean = value.replace(/\s+/g, " ").trim() || "TBD";
  return JSON.stringify(clean);
}

function normalizeRfpText(text: string) {
  return text
    .replace(/\r/g, "\n")
    .replace(/<script[\s\S]*?<\/script>/gi, " ")
    .replace(/<style[\s\S]*?<\/style>/gi, " ")
    .replace(/<[^>]+>/g, " ")
    .replace(/\u00a0/g, " ")
    .replace(/[ \t]+/g, " ")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function normalizeWhitespace(value: string) {
  return value.replace(/\s+/g, " ").trim();
}

function outlineStringValue(value: unknown) {
  return typeof value === "string" ? value.trim().slice(0, 500) : "";
}

function outlineStringArray(value: unknown, limit: number, preserveLeadingWhitespace = false) {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const output: string[] = [];
  for (const item of value) {
    const normalized = preserveLeadingWhitespace && typeof item === "string" ? item.replace(/\t/g, "  ").replace(/\s+$/g, "").slice(0, 500) : outlineStringValue(item);
    const key = normalized.toLowerCase();
    if (!normalized || seen.has(key)) continue;
    seen.add(key);
    output.push(normalized);
    if (output.length >= limit) break;
  }
  return output;
}

function documentOutlineCategoryForTemplate(template: BusinessDocumentTemplate) {
  if (["rfp", "rfq", "tender"].includes(template.id)) return "Procurement";
  if (["tutorial", "lesson-plan", "lesson-content", "technical-textbook"].includes(template.id)) return "Learning";
  if (["novel", "podcast-script", "movie-script"].includes(template.id)) return "Creative";
  if (["proposal", "sow", "capability-statement", "case-study"].includes(template.id)) return "Business Development";
  if (["business-case", "executive-brief"].includes(template.id)) return "Executive";
  return "General";
}

function isRequirementLine(line: string) {
  if (line.length < 18) return false;
  if (/^(table of contents|contents|page \d+|copyright)\b/i.test(line)) return false;
  return /(\b(shall|must|required|mandatory|provide|submit|include|describe|demonstrate|support|deliver|implement|maintain|comply|bidder|proposer|vendor|contractor|respondent|evaluation|scoring|criteria|deadline|due date|page limit|font size|certificate|declaration)\b|^\s*(\d+(\.\d+){0,4}|[A-Z]\.|\([a-z0-9]+\)|[-*])\s+)/i.test(line);
}

function extractRfpRequirementCandidates(lines: Array<{ line: string; index: number }>) {
  const candidates: Array<{ line: string; index: number }> = [];
  let tableHeaders: string[] = [];
  let tableContextActive = false;
  for (const item of lines) {
    const normalized = normalizeRfpTableLikeLine(item.line);
    if (isRfpTableSeparatorLine(normalized)) continue;
    const cells = rfpTableCells(normalized);
    if (cells.length >= 2) {
      if (isRfpTableHeaderLine(normalized) || isRfpRequirementTableHeaderCells(cells)) {
        tableHeaders = cells;
        tableContextActive = isRfpRequirementTableHeaderCells(cells);
        continue;
      }
      if (tableContextActive) {
        const tableRequirement = rfpRequirementCandidateFromTableRow(tableHeaders, cells);
        if (tableRequirement) {
          candidates.push({ line: tableRequirement, index: item.index });
          continue;
        }
      }
    } else {
      tableHeaders = [];
      tableContextActive = false;
    }
    if (isRequirementLine(normalized) || isRfpTableRequirementLine(normalized) || isExplicitRfpConstraintLine(normalized)) {
      candidates.push({ line: normalized, index: item.index });
    }
  }
  return candidates;
}

function normalizeRfpTableLikeLine(line: string) {
  const cells = rfpTableCells(line);
  if (cells.length < 2) return line;
  return cells.join(" | ");
}

function rfpTableCells(line: string) {
  return line
    .split(/\s*(?:\||\t)\s*/g)
    .map((cell) => normalizeWhitespace(cell))
    .filter(Boolean);
}

function isRfpTableSeparatorLine(line: string) {
  const cells = rfpTableCells(line);
  return cells.length > 1 && cells.every((cell) => /^:?-{3,}:?$/.test(cell));
}

function isRfpRequirementTableHeaderCells(cells: string[]) {
  const header = cells.join(" ").toLowerCase();
  if (!/\b(requirement|minimum|mandatory|required|criteria|criterion|points?|score|weight|role|position|personnel|expert|specialist|qualification|experience|evidence|attachment|deliverable|annex|response|proof)\b/.test(header)) {
    return false;
  }
  return cells.filter(isRfpRequirementTableHeaderCell).length >= Math.max(2, Math.ceil(cells.length / 2));
}

function isRfpRequirementTableHeaderCell(cell: string) {
  const clean = normalizeWhitespace(cell);
  if (!clean || /\d/.test(clean) || clean.length > 48) return false;
  return /\b(requirement|minimum|required|mandatory|criteria|criterion|role|position|personnel|qualification|experience|evidence|attachment|deliverable|annex|response|proof|points?|score|weight|status|owner|section)\b/i.test(clean);
}

function rfpRequirementCandidateFromTableRow(headers: string[], cells: string[]) {
  if (cells.length < 2 || cells.every((cell) => /^yes|no|n\/a$/i.test(cell))) return "";
  const headerText = headers.join(" ").toLowerCase();
  const rowText = cells.join(" ").toLowerCase();
  const tableHasRequirementContext = /\b(requirement|minimum|mandatory|required|criteria|criterion|role|position|personnel|expert|specialist|qualification|experience|evidence|attachment|deliverable|annex|points?|score|weight)\b/.test(headerText);
  const rowHasRequirementSignal = /\b(\d+\+?\s+years?|degree|certif|must|shall|required|mandatory|yes|pass\/fail|points?|pts?|%|annex|form|certificate|signed|submit|deliver|provide|expert|specialist|architect|manager|lead|analyst|engineer)\b/.test(rowText);
  if (!tableHasRequirementContext || !rowHasRequirementSignal) return "";
  const pairs = cells.map((cell, index) => {
    const header = normalizeRfpTableHeader(headers[index] || `Column ${index + 1}`);
    return `${header}: ${cell}`;
  });
  return pairs.join(" | ");
}

function normalizeRfpTableHeader(header: string) {
  const clean = normalizeWhitespace(header.replace(/[*_`]/g, ""));
  if (!clean) return "Table field";
  return clean.charAt(0).toUpperCase() + clean.slice(1);
}

function isRfpTableRequirementLine(line: string) {
  if (!line.includes("|")) return false;
  if (isRfpTableHeaderLine(line)) return false;
  const lower = line.toLowerCase();
  const hasComplianceSignal = /\b(mandatory|required|yes|pass\/fail|compliant|non-compliant|disqualif|reject|scored|points?|weight|deadline|attachment|certificate|form|submission)\b/.test(lower);
  const hasRequirementSubject = /\b(requirement|description|response|vendor|bidder|proposer|contractor|document|evidence|deliverable|criteria|proof|section)\b/.test(lower);
  return hasComplianceSignal && hasRequirementSubject;
}

function isRfpTableHeaderLine(line: string) {
  return /^\s*(requirement|description|item|criterion|criteria)\s*\|/i.test(line) && /\|\s*(mandatory|required|evidence|response|owner|status|weight|points?)\b/i.test(line);
}

function isExplicitRfpConstraintLine(line: string) {
  return /\b(submission deadline|deadline for submission|due no later than|will be rejected|automatic exclusion|non-responsive|maximum \d+ pages?|limit of \d+ pages?|font size|times new roman|arial|calibri|scoring|weighted|points?)\b/i.test(line);
}

function buildRfpRequirement(line: string, sourceLine: number, index: number, profile: Record<keyof BusinessProfile, string>): RfpRequirement {
  const category = categorizeRequirement(line);
  const requirementType = classifyRfpRequirementType(line, category);
  const disqualificationRisk = hasRfpDisqualificationRisk(line);
  const confidence = requirementConfidence(line, requirementType, disqualificationRisk);
  return {
    id: `RFP-REQ-${String(index).padStart(3, "0")}`,
    requirementType,
    category,
    text: stripRequirementPrefix(line),
    sourceLine,
    sourceExcerpt: line,
    disqualificationRisk,
    confidence,
    responseStrategy: responseStrategyForCategory(category, profile),
    evidenceNeeded: evidenceForCategory(category),
    owner: ownerForCategory(category),
  };
}

function dedupeRequirements(requirements: RfpRequirement[]) {
  const seen = new Set<string>();
  const deduped: RfpRequirement[] = [];
  for (const requirement of requirements) {
    const key = requirement.text.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().slice(0, 140);
    if (!key || seen.has(key)) continue;
    seen.add(key);
    deduped.push({ ...requirement, id: `RFP-REQ-${String(deduped.length + 1).padStart(3, "0")}` });
  }
  return deduped.slice(0, 80);
}

function fallbackRequirements(lines: Array<{ line: string; index: number }>, profile: Record<keyof BusinessProfile, string>) {
  return lines
    .filter((item) => item.line.length > 40)
    .slice(0, 8)
    .map((item, index) => buildRfpRequirement(item.line, item.index, index + 1, profile));
}

function stripRequirementPrefix(line: string) {
  return normalizeWhitespace(line.replace(/^(\s*(\d+(?:\.\d+){0,4}\.?|[A-Z]\.|\([a-z0-9]+\)|[-*])\s*)/i, ""));
}

function categorizeRequirement(line: string) {
  const value = line.toLowerCase();
  if (/\b(maximum \d+ pages?|limit of \d+ pages?|font size|times new roman|arial|calibri|margin|page limit|format|file type|pdf|docx)\b/.test(value)) return "Format";
  if (/\b(price|pricing|cost|fee|commercial|budget|payment|invoice|rate)\b/.test(value)) return "Pricing";
  if (/\b(deadline|timeline|schedule|milestone|implementation|delivery|go-live|days|weeks|months)\b/.test(value)) return "Timeline";
  if (/\b(security|privacy|data|confidential|encryption|accessibility|compliance|regulatory|audit)\b/.test(value)) return "Compliance";
  if (/\b(team|personnel|staff|experience|reference|case stud|qualification|certification)\b/.test(value)) return "Team and Experience";
  if (/\b(attach|form|signed|signature|certificate|insurance|tax|registration|declaration)\b/.test(value)) return "Mandatory Attachment";
  if (/\b(report|status|governance|meeting|communication|quality|risk|sla|support)\b/.test(value)) return "Delivery Governance";
  if (/\b(solution|technical|system|platform|integration|api|architecture|training|documentation)\b/.test(value)) return "Technical Solution";
  return "Requirement";
}

function classifyRfpRequirementType(line: string, category: string): RfpRequirementType {
  const value = line.toLowerCase();
  if (/\b(evaluation|scor|weight|criteria|points?|award|selection|rated|technical merit|best value)\b/.test(value)) return "SCORED";
  if (/\b(deadline|due no later than|submission date|submission time|closing date|close date)\b/.test(value)) return "DEADLINE";
  if (/\b(maximum \d+ pages?|limit of \d+ pages?|font size|times new roman|arial|calibri|margin|page limit|format|file type|pdf|docx)\b/.test(value)) return "FORMAT";
  if (/\b(shall|must|required|mandatory|submit|required form|certificate|declaration|signed|signature|minimum)\b/.test(value) || category === "Mandatory Attachment") return "MANDATORY";
  if (/\b(expect|prefer|should|may score|value for money|intent|objective|outcome)\b/.test(value)) return "IMPLIED";
  return "REQUIREMENT";
}

function hasRfpDisqualificationRisk(line: string) {
  return /\b(will be rejected|automatic(?:ally)? excluded|automatic exclusion|disqualif(?:y|ication|ied)|non-responsive|shall be excluded|failure to (?:submit|provide|include|meet)|must be submitted|late submissions? (?:will|shall) not be accepted)\b/i.test(line);
}

function requirementConfidence(line: string, type: RfpRequirementType, disqualificationRisk: boolean): "high" | "medium" | "low" {
  if (disqualificationRisk) return "high";
  if (type === "MANDATORY" || type === "DEADLINE" || type === "FORMAT") return "high";
  if (type === "SCORED" || /\b(shall|must|required|submit|include|provide|demonstrate)\b/i.test(line)) return "medium";
  return "low";
}

function responseSectionForCategory(category: string) {
  const map: Record<string, string> = {
    Pricing: "Pricing and Budget Response",
    Timeline: "Implementation Plan and Timeline",
    Compliance: "Compliance Matrix",
    Format: "Submission Format",
    "Team and Experience": "Capability Match",
    "Mandatory Attachment": "Mandatory Attachments",
    "Delivery Governance": "Risk and Assumptions",
    "Technical Solution": "Proposed Solution",
    Requirement: "Requirements Analysis",
  };
  return map[category] || "Requirements Analysis";
}

function responseStrategyForCategory(category: string, profile: Record<keyof BusinessProfile, string>) {
  const company = profile.companyName;
  const map: Record<string, string> = {
    Pricing: `State ${company}'s pricing basis, assumptions, exclusions, validity, and required commercial forms.`,
    Timeline: "Mirror every buyer deadline, milestone, dependency, and approval date in the implementation plan.",
    Compliance: "Map the requirement to a compliance response with proof, exception handling, and reviewer sign-off.",
    Format: "Mirror the buyer's page limit, font, file type, naming, and packaging rules before final export.",
    "Team and Experience": "Attach named roles, relevant experience, certifications, references, and case evidence.",
    "Mandatory Attachment": "Add the required attachment to the submission checklist and assign an owner.",
    "Delivery Governance": "Describe governance cadence, quality controls, risk management, reporting, and escalation.",
    "Technical Solution": "Explain the proposed technical approach, integrations, controls, training, and support model.",
    Requirement: "Answer directly, cite supporting evidence, and keep unresolved assumptions visible.",
  };
  return map[category] || map.Requirement;
}

function buildRfpComplianceRow(requirement: RfpRequirement): RfpComplianceRow {
  const responseSection = responseSectionForCategory(requirement.category);
  const needsEvidenceReview = requirementNeedsEvidenceReview(requirement);
  const suggestedResponse = suggestedRfpRequirementResponse(requirement, responseSection, needsEvidenceReview);
  const verificationChecklist = [
    `${requirement.id} maps source line ${requirement.sourceLine} to ${responseSection}.`,
    `Requirement type: ${requirement.requirementType}; confidence: ${requirement.confidence}.`,
    requirement.disqualificationRisk
      ? "Disqualification risk flagged: missing or late response may make the bid non-responsive."
      : "No automatic disqualification wording detected; reviewer should still confirm the RFP source.",
    `Suggested answer reviewed: ${suggestedResponse}`,
    `Evidence required: ${requirement.evidenceNeeded}`,
    `Owner assigned: ${requirement.owner}.`,
    needsEvidenceReview
      ? "Not fully verified until the owner attaches proof and a reviewer signs off."
      : "Responsive draft prepared; reviewer should confirm before submission.",
  ];
  return {
    ...requirement,
    complianceStatus: needsEvidenceReview ? "Needs evidence review" : "Responsive draft prepared",
    responseSection,
    suggestedResponse,
    verification: `${requirement.id} mapped from source line ${requirement.sourceLine} to ${responseSection} and Compliance Matrix; type ${requirement.requirementType}; ${requirement.disqualificationRisk ? "disqualification risk flagged; " : ""}evidence owner ${requirement.owner} must confirm proof before submission.`,
    verificationChecklist,
  };
}

function suggestedRfpRequirementResponse(requirement: RfpRequirement, responseSection: string, needsEvidenceReview: boolean) {
  const cleanRequirement = requirement.text.replace(/\s+/g, " ").trim().replace(/\.$/, "");
  const caveat = needsEvidenceReview
    ? ` This answer is ready for ${requirement.owner} evidence attachment and reviewer sign-off before submission.`
    : " This answer is draft-ready; final reviewer should confirm no exception is needed.";
  const categoryLead: Record<string, string> = {
    Pricing: "We will provide a transparent commercial response with pricing basis, assumptions, exclusions, validity, and required commercial forms aligned to the buyer's format.",
    Timeline: "We will meet the required schedule through a phased implementation plan with named milestones, dependencies, approval points, and delivery ownership.",
    Compliance: "We will comply with the requirement by mapping controls, exceptions, proof artifacts, and reviewer sign-off in the compliance response.",
    Format: "We will package the response in the buyer's required format with page, font, file type, naming, and submission checks completed before upload.",
    "Team and Experience": "We will demonstrate delivery capacity through named roles, relevant experience, certifications, references, and comparable project proof.",
    "Mandatory Attachment": "We will include the required attachment in the submission checklist and assign ownership for completion before final packaging.",
    "Delivery Governance": "We will manage delivery through governance cadence, quality controls, risk management, reporting, escalation, and service-level commitments.",
    "Technical Solution": "We will address the requirement through a practical technical approach covering architecture, integrations, controls, training, documentation, and support.",
    Requirement: "We will answer the requirement directly, cite supporting evidence, and keep unresolved assumptions visible for review.",
  };
  return [
    categoryLead[requirement.category] || categoryLead.Requirement,
    `Specific requirement: ${cleanRequirement}.`,
    `Response section: ${responseSection}.`,
    `Evidence to attach: ${requirement.evidenceNeeded}`,
    caveat,
  ].join(" ");
}

function requirementNeedsEvidenceReview(requirement: RfpRequirement) {
  if (!requirement.evidenceNeeded.trim()) return true;
  return !/\b(no additional evidence|not applicable|n\/a)\b/i.test(requirement.evidenceNeeded);
}

function buildRfpVerificationSummary(
  requirements: RfpRequirement[],
  complianceRows: RfpComplianceRow[],
  attachments: string[],
  criteria: string[],
  scoringWeights: RfpScoringWeight[],
  annexReferences: RfpAnnexReference[],
  bilingualRequirements: string[],
  placeholderRisks: string[],
): RfpVerificationSummary {
  const rowIds = new Set(complianceRows.map((row) => row.id));
  const allRequirementsMapped = requirements.length > 0 && requirements.every((requirement) => rowIds.has(requirement.id));
  const rowsNeedingEvidence = complianceRows.filter((row) => row.complianceStatus === "Needs evidence review").length;
  const disqualificationRisks = complianceRows.filter((row) => row.disqualificationRisk).length;
  const scoredRows = Math.max(complianceRows.filter((row) => row.requirementType === "SCORED").length, scoringWeights.length);
  const checklist = [
    `${requirements.length} extracted requirement(s) mapped to ${complianceRows.length} compliance row(s).`,
    allRequirementsMapped
      ? "Every extracted requirement has a compliance matrix row."
      : "Coverage gap: one or more extracted requirements are missing compliance rows.",
    rowsNeedingEvidence
      ? `${rowsNeedingEvidence} row(s) still need attached evidence or reviewer sign-off before submission.`
      : "No evidence-review blockers were detected in the extracted requirements.",
    disqualificationRisks
      ? `${disqualificationRisks} row(s) contain explicit rejection, exclusion, or non-responsive bid risk language.`
      : "No explicit rejection or automatic exclusion language was detected in extracted rows.",
    scoredRows
      ? `${scoredRows} scored or weighted row(s) should be mirrored in win themes and evaluator-facing headings.`
      : "No scored rows were detected; confirm evaluation weighting manually.",
    attachments.length
      ? `${attachments.length} mandatory attachment hint(s) need checklist confirmation.`
      : "No mandatory attachment hints were detected; confirm appendices manually.",
    annexReferences.length
      ? `${annexReferences.length} annex reference(s) were extracted and must be matched to signed or completed response documents.`
      : "No annex references were detected; confirm annex schedules manually.",
    bilingualRequirements.length
      ? `${bilingualRequirements.length} bilingual or French-language obligation(s) need delivery and training-material coverage.`
      : "No bilingual language requirement was detected; confirm language obligations manually.",
    placeholderRisks.length
      ? `${placeholderRisks.length} placeholder or unfinished-response trap(s) were detected and must be resolved before submission.`
      : "No placeholder trap wording was detected in the source scan.",
    criteria.length
      ? `${criteria.length} evaluation criteria hint(s) should be mirrored in the executive response and section scoring.`
      : "No evaluation criteria hints were detected; confirm scoring weights manually.",
  ];
  return {
    totalRequirements: requirements.length,
    complianceRows: complianceRows.length,
    rowsNeedingEvidence,
    allRequirementsMapped,
    checklist,
  };
}

function evidenceForCategory(category: string) {
  const map: Record<string, string> = {
    Pricing: "Pricing schedule, assumptions, approvals, and commercial terms.",
    Timeline: "Delivery plan, milestone schedule, dependency log, and named delivery owner.",
    Compliance: "Policy, certificate, audit result, control description, or signed declaration.",
    Format: "Formatted proposal export, page-count proof, font/style settings, file naming proof, and final packaging checklist.",
    "Team and Experience": "CV, biography, certification, reference, case study, or project proof.",
    "Mandatory Attachment": "Required form, certificate, signed statement, registration, or insurance proof.",
    "Delivery Governance": "Governance plan, quality plan, risk register, reporting sample, or SLA.",
    "Technical Solution": "Architecture note, method statement, integration plan, training plan, or support model.",
    Requirement: "Source evidence placeholder, owner confirmation, and reviewer approval.",
  };
  return map[category] || map.Requirement;
}

function ownerForCategory(category: string) {
  const map: Record<string, string> = {
    Pricing: "Finance / Commercial",
    Timeline: "Delivery Lead",
    Compliance: "Legal / Compliance",
    Format: "Bid Coordinator",
    "Team and Experience": "Bid Manager",
    "Mandatory Attachment": "Bid Coordinator",
    "Delivery Governance": "Delivery Lead",
    "Technical Solution": "Solution Lead",
    Requirement: "Bid Owner",
  };
  return map[category] || "Bid Owner";
}

function extractMatchingLines(lines: Array<{ line: string; index: number }>, pattern: RegExp, limit: number) {
  return lines
    .filter((item) => pattern.test(item.line))
    .map((item) => stripRequirementPrefix(item.line))
    .filter((line, index, array) => array.findIndex((candidate) => candidate.toLowerCase() === line.toLowerCase()) === index)
    .slice(0, limit);
}

function dedupeStrings(values: string[]) {
  const seen = new Set<string>();
  const deduped: string[] = [];
  for (const value of values.map(normalizeWhitespace).filter(Boolean)) {
    const key = value.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim();
    if (!key || seen.has(key)) continue;
    seen.add(key);
    deduped.push(value);
  }
  return deduped;
}

function extractRfpScoringWeights(lines: Array<{ line: string; index: number }>): RfpScoringWeight[] {
  const output: RfpScoringWeight[] = [];
  const seen = new Set<string>();
  const pattern = /([A-Za-z][A-Za-z0-9 /&()'.,:-]{2,90}?)\s+(\d{1,3})\s*(%|percent|points?|pts?)\b/gi;
  for (const item of lines) {
    for (const match of item.line.matchAll(pattern)) {
      const rawCriterion = normalizeWhitespace(match[1] || "")
        .replace(/^(evaluation criteria|criteria|scoring|maximum points?|points?|weight(?:ed)?|award)\s*[:;-]?\s*/i, "")
        .replace(/[,;:|-]+$/g, "")
        .trim();
      const criterion = rawCriterion || "Scored criterion";
      const weight = Number(match[2]);
      const unit = /^%|percent$/i.test(match[3]) ? "%" : "points";
      const key = `${criterion.toLowerCase()}|${weight}|${unit}`;
      if (!criterion || !Number.isFinite(weight) || weight <= 0 || seen.has(key)) continue;
      seen.add(key);
      output.push({ criterion, weight, unit, sourceLine: item.index });
    }
  }
  return output.slice(0, 30);
}

function extractRfpAnnexReferences(lines: Array<{ line: string; index: number }>): RfpAnnexReference[] {
  const output: RfpAnnexReference[] = [];
  const seen = new Set<string>();
  const pattern = /\b(Annex(?:ure)?\s+([A-Z0-9]+))(?:\s*[-:]\s*([^.;\n]+))?/gi;
  for (const item of lines) {
    for (const match of item.line.matchAll(pattern)) {
      const annex = normalizeWhitespace(match[1] || "");
      const label = normalizeWhitespace(match[3] || "");
      const requirement = label ? `${annex}: ${label}` : `${annex} must be reviewed, completed, and included if required.`;
      const key = `${annex}|${label}`.toLowerCase();
      if (!annex || seen.has(key)) continue;
      seen.add(key);
      output.push({ annex, label, sourceLine: item.index, requirement });
    }
  }
  return output.slice(0, 30);
}

function inferRfpCapabilities(requirements: RfpRequirement[], text: string, profile: Record<keyof BusinessProfile, string>) {
  const capabilities = new Set<string>();
  if (profile.companyName && !profile.companyName.startsWith("{{")) capabilities.add(`${profile.companyName} company profile and delivery credentials`);
  if (profile.industry && !profile.industry.startsWith("{{")) capabilities.add(`${profile.industry} domain expertise`);
  const lower = text.toLowerCase();
  const probes: Array<[RegExp, string]> = [
    [/\b(training|enablement|workshop|knowledge transfer)\b/, "Training, enablement, and knowledge-transfer plan"],
    [/\b(integration|api|system|platform|technical)\b/, "Technical implementation and integration capability"],
    [/\b(security|privacy|data protection|audit)\b/, "Security, privacy, and compliance controls"],
    [/\b(reporting|governance|status|sla|support)\b/, "Governance, reporting, service management, and support model"],
    [/\b(case stud|reference|experience|qualification)\b/, "Relevant experience, references, and proof points"],
    [/\b(pricing|cost|commercial|budget)\b/, "Commercial model, pricing assumptions, and value narrative"],
  ];
  for (const [pattern, label] of probes) {
    if (pattern.test(lower) || requirements.some((requirement) => pattern.test(requirement.text))) capabilities.add(label);
  }
  if (!capabilities.size) capabilities.add("Bid-specific capability narrative to be completed from the RFP source and business profile");
  return [...capabilities].slice(0, 10);
}

function inferStatedRfpIntent(lines: Array<{ line: string; index: number }>, requirements: RfpRequirement[]) {
  const explicitIntent = extractMatchingLines(
    lines,
    /\b(purpose|objective|goal|intent|seeking|scope of work|background|overview|the successful|the selected|vendor will|contractor will|proposer shall)\b/i,
    8,
  );
  if (explicitIntent.length) return explicitIntent;
  return requirements
    .slice(0, 5)
    .map((requirement) => `The buyer states a need for ${requirement.text.replace(/\.$/, "")}.`);
}

function inferImpliedRfpIntent(
  requirements: RfpRequirement[],
  timelines: string[],
  budgetHints: string[],
  criteria: string[],
  attachments: string[],
  profile: Record<keyof BusinessProfile, string>,
) {
  const intent = new Set<string>();
  if (criteria.length) intent.add("The buyer likely wants an easily scored response; mirror evaluation criteria and make proof points visible.");
  if (requirements.some((requirement) => requirement.category === "Compliance") || attachments.length) intent.add("The buyer is managing procurement risk; include evidence, declarations, and reviewer sign-off instead of broad claims.");
  if (timelines.length) intent.add("The buyer is time-sensitive; show a credible mobilization plan, milestones, dependencies, and named delivery ownership.");
  if (budgetHints.length) intent.add("The buyer is commercially constrained; make price basis, assumptions, exclusions, and value-for-money explicit.");
  if (requirements.some((requirement) => requirement.category === "Team and Experience")) intent.add("The buyer needs confidence in delivery capacity; foreground relevant team credentials, references, and comparable work.");
  if (requirements.some((requirement) => requirement.category === "Technical Solution")) intent.add("The buyer wants a practical implementation answer; connect the solution design to requirements, integrations, training, and support.");
  if (profile.industry && !profile.industry.startsWith("{{")) intent.add(`The response should translate ${profile.industry} expertise into buyer-specific outcomes, not generic capability language.`);
  if (!intent.size) intent.add("The buyer likely wants a low-risk, complete, easy-to-evaluate response; keep every requirement mapped and every assumption visible.");
  return [...intent];
}

function inferRfpRisks(
  requirements: RfpRequirement[],
  timelines: string[],
  budgetHints: string[],
  attachments: string[],
  criticalDisqualifiers: string[] = [],
  placeholderRisks: string[] = [],
) {
  const risks = new Set<string>();
  if (requirements.length > 30) risks.add("Large requirement set; assign matrix owners and verify every row before submission.");
  if (!timelines.length) risks.add("No explicit timeline detected; confirm submission deadline and delivery milestones.");
  if (!budgetHints.length) risks.add("No budget or pricing hint detected; confirm pricing format, assumptions, taxes, and validity period.");
  if (attachments.length) risks.add("Mandatory attachments detected; missing forms or signatures can make the bid nonresponsive.");
  if (criticalDisqualifiers.length) risks.add("Critical disqualification traps detected; verify each pass/fail item before any response drafting is treated as complete.");
  if (placeholderRisks.length) risks.add("Placeholder or unfinished-response wording detected; unresolved placeholders can make a submission non-compliant.");
  if (requirements.some((requirement) => requirement.category === "Compliance")) risks.add("Compliance requirements need legal or control-owner sign-off before final submission.");
  if (!risks.size) risks.add("Keep all generated responses under human review until source evidence and approvals are verified.");
  return [...risks];
}

function inferRfpQuestions(
  requirements: RfpRequirement[],
  timelines: string[],
  budgetHints: string[],
  criteria: string[],
  attachments: string[],
  profile: Record<keyof BusinessProfile, string>,
) {
  const questions = new Set<string>();
  if (!profile.defaultClientName || profile.defaultClientName.startsWith("{{")) questions.add("Who is the buying organization or issuing authority?");
  if (!timelines.length) questions.add("What is the submission deadline, time zone, validity period, and delivery schedule?");
  if (!budgetHints.length) questions.add("What pricing format, budget ceiling, taxes, and commercial assumptions are required?");
  if (!criteria.length) questions.add("What evaluation criteria and scoring weights should the response optimize for?");
  if (!attachments.length) questions.add("Which mandatory forms, certificates, declarations, and signatures are required?");
  if (!requirements.length) questions.add("Can the full RFP text be imported or pasted so every requirement is mapped?");
  questions.add("Which claims require citations, customer references, or reviewer approval before submission?");
  return [...questions];
}

function inferRfpWarnings(input: RfpSourceInput, text: string, requirements: RfpRequirement[]) {
  const warnings: string[] = [];
  if (!text.trim()) warnings.push("No RFP text was supplied; paste extracted PDF/DOCX text, Markdown, or fetched URL content before relying on the matrix.");
  if ((input.kind === "pdf" || input.kind === "docx") && text.length < 400) warnings.push("PDF/DOCX input looks short; confirm text extraction captured the full RFP, including appendices.");
  if (input.kind === "url" && !input.url) warnings.push("URL source type selected without a URL.");
  if (requirements.length > 0 && requirements.length < 3) warnings.push("Only a few requirements were detected; verify headings, tables, and attachments manually.");
  return warnings;
}

function markdownBullets(items: string[], fallback: string) {
  return (items.length ? items : [fallback]).map((item) => `- ${item}`).join("\n");
}

function markdownInlineList(items: string[], fallback: string) {
  const values = items.map(normalizeWhitespace).filter(Boolean);
  return values.length ? values.slice(0, 6).join("; ") : fallback;
}

function escapeMarkdownTableCell(value: string) {
  return normalizeWhitespace(value).replace(/\|/g, "\\|");
}
