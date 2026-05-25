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
    label: "Create outline",
    prompt: "Create or refine the chapter, section, subsection, and subsubsection structure before writing prose.",
  },
  {
    id: "draft",
    label: "Draft section by section",
    prompt: "Generate each section systematically with placeholders preserved for facts, figures, approvals, and sources.",
  },
  {
    id: "qa",
    label: "Quality assurance",
    prompt: "Run checks for missing evidence, unsupported claims, stale assumptions, compliance gaps, and export readiness.",
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

function sectionPromptForHeading(heading: string) {
  return `<!-- Draft ${heading.toLowerCase()} with concrete evidence, unresolved placeholders, and review notes. -->`;
}

function yamlScalar(value: string) {
  const clean = value.replace(/\s+/g, " ").trim() || "TBD";
  return JSON.stringify(clean);
}
