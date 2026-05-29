import type { LatexTemplatePreset } from "./workspacePersistence.js";

export interface LatexTemplateProfile {
  id: LatexTemplatePreset;
  label: string;
  summary: string;
  bestFor: string[];
}

export const latexTemplateProfiles: LatexTemplateProfile[] = [
  {
    id: "article",
    label: "Article",
    summary: "Clean default for short papers, memos, and general TeX handoff.",
    bestFor: ["Short reports", "Technical notes", "General-purpose export"],
  },
  {
    id: "business-report",
    label: "Business Report",
    summary: "Executive-facing report layout with stronger headings and audit metadata.",
    bestFor: ["Consulting reports", "Board packs", "Management documents"],
  },
  {
    id: "proposal",
    label: "Proposal",
    summary: "Proposal-oriented layout tuned for scope, approach, team, and commercial narrative.",
    bestFor: ["Proposals", "Statements of work", "Business development"],
  },
  {
    id: "rfp-response",
    label: "RFP Response",
    summary: "Compliance-first template with room for requirements, matrices, and attachments.",
    bestFor: ["RFP responses", "Tenders", "RFQs"],
  },
  {
    id: "technical-report",
    label: "Technical Report",
    summary: "Report class with math, figures, long tables, source references, and appendices.",
    bestFor: ["Architecture reports", "Research reports", "Engineering documentation"],
  },
  {
    id: "academic-paper",
    label: "Academic Paper",
    summary: "Article class with bibliography and citation-friendly defaults.",
    bestFor: ["Academic papers", "Research briefs", "Conference drafts"],
  },
  {
    id: "textbook",
    label: "Textbook",
    summary: "Book class structure for chapter-based long-form technical or instructional material.",
    bestFor: ["Textbooks", "Tutorial manuals", "Course material"],
  },
  {
    id: "book",
    label: "Book",
    summary: "Book class structure for long-form manuscripts and multi-chapter documents.",
    bestFor: ["Books", "Novels", "Long-form manuscripts"],
  },
];

export function latexTemplateProfileById(id: LatexTemplatePreset) {
  return latexTemplateProfiles.find((profile) => profile.id === id) || latexTemplateProfiles[0];
}
