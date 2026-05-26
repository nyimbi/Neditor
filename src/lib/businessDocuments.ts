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
  | "tutorial"
  | "lesson-plan"
  | "lesson-content"
  | "technical-textbook"
  | "novel"
  | "podcast-script"
  | "movie-script"
  | "proposal"
  | "rfp"
  | "rfq"
  | "tender"
  | "sow"
  | "capability-statement"
  | "case-study"
  | "business-case"
  | "executive-brief";

export interface BusinessDocumentTemplate {
  id: BusinessDocumentKind;
  label: string;
  summary: string;
  docsLiveType: string;
  bestFor: string[];
  aiPrompt: string;
  outline: string[];
}

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

export interface AiDocumentWizardStep {
  id: string;
  label: string;
  prompt: string;
}

export interface AgenticCliIntegration {
  id: "claude-code" | "codex" | "opencode";
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

export interface RfpRequirement {
  id: string;
  category: string;
  text: string;
  sourceLine: number;
  responseStrategy: string;
  evidenceNeeded: string;
  owner: string;
}

export interface RfpComplianceRow extends RfpRequirement {
  complianceStatus: "Responsive draft prepared" | "Needs evidence review";
  responseSection: string;
  verification: string;
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
  capabilities: string[];
  statedIntent: string[];
  impliedIntent: string[];
  timelines: string[];
  budgetHints: string[];
  evaluationCriteria: string[];
  mandatoryAttachments: string[];
  risks: string[];
  questions: string[];
  warnings: string[];
  completenessScore: number;
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
    summary: "Creates episode scripts with cold open, host segments, guest questions, sponsor copy, and production notes.",
    docsLiveType: "podcast-script",
    bestFor: ["Podcast episodes", "Interview shows", "Narrative audio"],
    aiPrompt: "Create a podcast episode script with show brief, cold open, intro, segmented host copy, guest questions, sponsor or promo reads, outro, timing, transitions, and production notes.",
    outline: ["Show Brief", "Cold Open", "Intro", "Segment 1", "Segment 2", "Guest Questions", "Sponsor or Promo Read", "Outro", "Production Notes"],
  },
  {
    id: "movie-script",
    label: "Movie script",
    summary: "Develops screenplay structure with logline, characters, acts, key scenes, dialogue notes, and production constraints.",
    docsLiveType: "movie-script",
    bestFor: ["Screenplays", "Film treatments", "Scene planning"],
    aiPrompt: "Create a movie script planning packet with logline, character arcs, world and tone, three-act structure, key scenes, dialogue notes, visual motifs, and production constraints.",
    outline: ["Logline", "Characters", "World and Tone", "Act I", "Act II", "Act III", "Key Scenes", "Dialogue Notes", "Production Constraints"],
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
  return Object.fromEntries(
    Object.entries(normalized).map(([key, value]) => [key, value || `{{${key}}}`]),
  ) as Record<keyof BusinessProfile, string>;
}

export function businessProfilePlaceholderText(profile: Partial<BusinessProfile> = {}) {
  const placeholders = businessProfilePlaceholderMap(profile);
  return Object.entries(placeholders)
    .map(([key, value]) => `${key}: ${value}`)
    .join("\n");
}

export function fillBusinessTemplate(markdown: string, profile: Partial<BusinessProfile> = {}, extra: Record<string, string> = {}) {
  const placeholders = { ...businessProfilePlaceholderMap(profile), ...extra };
  return markdown.replace(/\{\{([a-zA-Z0-9_ -]+)\}\}/g, (match, key: string) => {
    const normalizedKey = key.trim().replace(/\s+/g, "_");
    return placeholders[normalizedKey as keyof BusinessProfile] || extra[normalizedKey] || match;
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
  return "";
}

export function businessSnippetMarkdown(snippet: BusinessDocumentSnippet, profile: Partial<BusinessProfile> = {}) {
  return `${fillBusinessTemplate(snippet.body, profile).trimEnd()}\n`;
}

export function businessWizardContext(template: BusinessDocumentTemplate, profile: Partial<BusinessProfile> = {}) {
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
    "Agent handoff options:",
    ...agenticCliIntegrations.map((integration) => `- ${integration.label} (${integration.command}): ${integration.handoff}`),
  ].join("\n");
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
    significantLines
      .filter((item) => isRequirementLine(item.line))
      .map((item, index) => buildRfpRequirement(item.line, item.index, index + 1, normalizedProfile)),
  );
  if (!requirements.length && normalizedText.trim()) {
    requirements.push(...fallbackRequirements(significantLines, normalizedProfile));
  }

  const timelines = extractMatchingLines(significantLines, /\b(deadline|due|schedule|timeline|milestone|weeks?|months?|days?|implementation|start date|go-live|submission)\b/i, 8);
  const budgetHints = extractMatchingLines(significantLines, /\b(budget|price|pricing|cost|fee|fees|commercial|payment|invoice|rate|rates|discount|value for money|\$|usd|eur|gbp|kes)\b/i, 8);
  const evaluationCriteria = extractMatchingLines(significantLines, /\b(evaluation|scor|weight|criteria|points|award|selection|rated|technical merit|best value)\b/i, 8);
  const mandatoryAttachments = extractMatchingLines(significantLines, /\b(attachment|appendix|form|certificate|insurance|tax|license|licence|registration|declaration|signature|signed|mandatory document)\b/i, 10);
  const capabilities = inferRfpCapabilities(requirements, normalizedText, normalizedProfile);
  const statedIntent = inferStatedRfpIntent(significantLines, requirements);
  const impliedIntent = inferImpliedRfpIntent(requirements, timelines, budgetHints, evaluationCriteria, mandatoryAttachments, normalizedProfile);
  const complianceRows = requirements.map((requirement) => ({
    ...requirement,
    complianceStatus: requirement.evidenceNeeded.includes("placeholder") ? "Needs evidence review" as const : "Responsive draft prepared" as const,
    responseSection: responseSectionForCategory(requirement.category),
    verification: `Mapped to ${responseSectionForCategory(requirement.category)} and Compliance Matrix; reviewer must confirm evidence before submission.`,
  }));
  const risks = inferRfpRisks(requirements, timelines, budgetHints, mandatoryAttachments);
  const questions = inferRfpQuestions(requirements, timelines, budgetHints, evaluationCriteria, mandatoryAttachments, normalizedProfile);
  const warnings = inferRfpWarnings(input, normalizedText, requirements);
  const completenessScore = Math.max(0, Math.min(100, Math.round(
    20 +
      Math.min(requirements.length, 12) * 4 +
      Math.min(capabilities.length, 6) * 3 +
      Math.min(timelines.length, 4) * 3 +
      Math.min(budgetHints.length, 4) * 3 +
      Math.min(evaluationCriteria.length, 4) * 3 +
      Math.min(mandatoryAttachments.length, 5) * 2 -
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
    capabilities,
    statedIntent,
    impliedIntent,
    timelines,
    budgetHints,
    evaluationCriteria,
    mandatoryAttachments,
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
    "| ID | Requirement | Category | Compliance status | Response section | Evidence / proof | Verification |",
    "| --- | --- | --- | --- | --- | --- | --- |",
    ...rows.map((row) => `| ${escapeMarkdownTableCell(row.id)} | ${escapeMarkdownTableCell(row.text)} | ${escapeMarkdownTableCell(row.category)} | ${escapeMarkdownTableCell(row.complianceStatus)} | ${escapeMarkdownTableCell(row.responseSection)} | ${escapeMarkdownTableCell(row.evidenceNeeded)} | ${escapeMarkdownTableCell(row.verification)} |`),
    rows.length ? "" : "| RFP-REQ-001 | Paste or import the RFP text to populate requirements. | Intake | Needs evidence review | Requirements Analysis | Source RFP text | Not verified. |",
  ].join("\n");
}

export function rfpResponseMarkdown(analysis: RfpAnalysis, profile: Partial<BusinessProfile> = {}) {
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
      "[TOC]",
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
      "## Capability Match",
      "",
      capabilityBullets,
      "",
      "## Proposed Solution",
      "",
      "Our response is organized around the buyer's stated outcomes, mandatory requirements, evaluation criteria, and delivery constraints. Each requirement has a drafted response path, an evidence placeholder, and a reviewer verification note.",
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

function isRequirementLine(line: string) {
  if (line.length < 18) return false;
  if (/^(table of contents|contents|page \d+|copyright)\b/i.test(line)) return false;
  return /(\b(shall|must|required|mandatory|provide|submit|include|describe|demonstrate|support|deliver|implement|maintain|comply|bidder|proposer|vendor|contractor|respondent)\b|^\s*(\d+(\.\d+){0,4}|[A-Z]\.|\([a-z0-9]+\)|[-*])\s+)/i.test(line);
}

function buildRfpRequirement(line: string, sourceLine: number, index: number, profile: Record<keyof BusinessProfile, string>): RfpRequirement {
  const category = categorizeRequirement(line);
  return {
    id: `RFP-REQ-${String(index).padStart(3, "0")}`,
    category,
    text: stripRequirementPrefix(line),
    sourceLine,
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
  return normalizeWhitespace(line.replace(/^(\s*(\d+(\.\d+){0,4}|[A-Z]\.|\([a-z0-9]+\)|[-*])\s*)/i, ""));
}

function categorizeRequirement(line: string) {
  const value = line.toLowerCase();
  if (/\b(price|pricing|cost|fee|commercial|budget|payment|invoice|rate)\b/.test(value)) return "Pricing";
  if (/\b(deadline|timeline|schedule|milestone|implementation|delivery|go-live|days|weeks|months)\b/.test(value)) return "Timeline";
  if (/\b(security|privacy|data|confidential|encryption|accessibility|compliance|regulatory|audit)\b/.test(value)) return "Compliance";
  if (/\b(team|personnel|staff|experience|reference|case stud|qualification|certification)\b/.test(value)) return "Team and Experience";
  if (/\b(attach|form|signed|signature|certificate|insurance|tax|registration|declaration)\b/.test(value)) return "Mandatory Attachment";
  if (/\b(report|status|governance|meeting|communication|quality|risk|sla|support)\b/.test(value)) return "Delivery Governance";
  if (/\b(solution|technical|system|platform|integration|api|architecture|training|documentation)\b/.test(value)) return "Technical Solution";
  return "Requirement";
}

function responseSectionForCategory(category: string) {
  const map: Record<string, string> = {
    Pricing: "Pricing and Budget Response",
    Timeline: "Implementation Plan and Timeline",
    Compliance: "Compliance Matrix",
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
    "Team and Experience": "Attach named roles, relevant experience, certifications, references, and case evidence.",
    "Mandatory Attachment": "Add the required attachment to the submission checklist and assign an owner.",
    "Delivery Governance": "Describe governance cadence, quality controls, risk management, reporting, and escalation.",
    "Technical Solution": "Explain the proposed technical approach, integrations, controls, training, and support model.",
    Requirement: "Answer directly, cite supporting evidence, and keep unresolved assumptions visible.",
  };
  return map[category] || map.Requirement;
}

function evidenceForCategory(category: string) {
  const map: Record<string, string> = {
    Pricing: "Pricing schedule, assumptions, approvals, and commercial terms.",
    Timeline: "Delivery plan, milestone schedule, dependency log, and named delivery owner.",
    Compliance: "Policy, certificate, audit result, control description, or signed declaration.",
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

function inferRfpRisks(requirements: RfpRequirement[], timelines: string[], budgetHints: string[], attachments: string[]) {
  const risks = new Set<string>();
  if (requirements.length > 30) risks.add("Large requirement set; assign matrix owners and verify every row before submission.");
  if (!timelines.length) risks.add("No explicit timeline detected; confirm submission deadline and delivery milestones.");
  if (!budgetHints.length) risks.add("No budget or pricing hint detected; confirm pricing format, assumptions, taxes, and validity period.");
  if (attachments.length) risks.add("Mandatory attachments detected; missing forms or signatures can make the bid nonresponsive.");
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

function escapeMarkdownTableCell(value: string) {
  return normalizeWhitespace(value).replace(/\|/g, "\\|");
}
