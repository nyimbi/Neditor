export type CalloutPresetTone = "neutral" | "positive" | "caution" | "critical" | "evidence" | "action";

export interface CalloutPreset {
  id: string;
  label: string;
  calloutType: string;
  tone: CalloutPresetTone;
  summary: string;
  title: string;
  body: string[];
  bestFor: string[];
}

export const calloutPresets: CalloutPreset[] = [
  {
    id: "decision",
    label: "Decision",
    calloutType: "decision",
    tone: "action",
    summary: "Show the exact choice, owner, and decision date.",
    title: "Decision needed",
    body: ["Decision: {{decision}}", "Owner: {{owner}}", "Needed by: {{date}}"],
    bestFor: ["Board papers", "Executive memos", "Approval requests"],
  },
  {
    id: "recommendation",
    label: "Recommendation",
    calloutType: "recommendation",
    tone: "positive",
    summary: "Make the preferred option visible before detailed evidence.",
    title: "Recommendation",
    body: ["Recommended option: {{option}}", "Why now: {{reason}}", "Evidence: {{evidence}}"],
    bestFor: ["Consulting reports", "Business cases", "Policy briefs"],
  },
  {
    id: "risk",
    label: "Risk",
    calloutType: "risk",
    tone: "critical",
    summary: "Flag a material risk with mitigation and accountable owner.",
    title: "Material risk",
    body: ["Risk: {{risk}}", "Impact: {{impact}}", "Mitigation: {{mitigation}}", "Owner: {{owner}}"],
    bestFor: ["Proposals", "Delivery plans", "Diligence memos"],
  },
  {
    id: "warning",
    label: "Warning",
    calloutType: "warning",
    tone: "caution",
    summary: "Warn reviewers about a constraint, dependency, or submission trap.",
    title: "Watch point",
    body: ["Constraint: {{constraint}}", "Consequence: {{consequence}}", "Action: {{action}}"],
    bestFor: ["RFP responses", "Compliance reviews", "Release handoffs"],
  },
  {
    id: "assumption",
    label: "Assumption",
    calloutType: "assumption",
    tone: "neutral",
    summary: "Keep uncertain inputs visible so reviewers can validate or reject them.",
    title: "Assumption",
    body: ["Assumption: {{assumption}}", "Validation needed: {{validation}}", "Fallback: {{fallback}}"],
    bestFor: ["Forecasts", "Pricing", "Research reports"],
  },
  {
    id: "evidence",
    label: "Evidence",
    calloutType: "evidence",
    tone: "evidence",
    summary: "Highlight source-backed proof, data, or reviewer evidence.",
    title: "Evidence",
    body: ["Source: {{source}}", "Finding: {{finding}}", "Confidence: {{confidence}}"],
    bestFor: ["Research reports", "Deep research", "Technical papers"],
  },
  {
    id: "action",
    label: "Action",
    calloutType: "action",
    tone: "action",
    summary: "Turn a section into a visible next-step commitment.",
    title: "Action required",
    body: ["Action: {{action}}", "Owner: {{owner}}", "Due: {{date}}", "Evidence of completion: {{evidence}}"],
    bestFor: ["Meeting notes", "Project plans", "Review closeout"],
  },
  {
    id: "note",
    label: "Note",
    calloutType: "note",
    tone: "neutral",
    summary: "Add a low-friction note without changing document flow.",
    title: "Note",
    body: ["{{note}}"],
    bestFor: ["Explanatory notes", "Reader guidance", "Draft context"],
  },
];

export function calloutPresetById(id: string): CalloutPreset | null {
  return calloutPresets.find((preset) => preset.id === id) || null;
}

export function calloutPresetMarkdown(preset: CalloutPreset): string {
  const lines = [`> [!${preset.calloutType}] ${preset.title}`, ...preset.body.map((line) => `> ${line}`)];
  return `${lines.join("\n")}\n`;
}
