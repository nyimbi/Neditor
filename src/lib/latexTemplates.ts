import { normalizeCustomLatexTemplateProfiles, type CustomLatexTemplateProfile, type LatexTemplatePreset } from "./workspacePersistence.js";

export interface LatexTemplateProfile {
  id: LatexTemplatePreset;
  label: string;
  summary: string;
  bestFor: string[];
  source: "built-in" | "custom";
}

export interface LatexTemplateExportProfile {
  id: string;
  label: string;
  documentClass: string;
  classOptions: string;
  packages: string[];
  geometry: string;
  hypersetup: string;
  header: string;
  chapterStyle: boolean;
}

export const latexTemplateProfiles: LatexTemplateProfile[] = [
  {
    id: "article",
    label: "Article",
    summary: "Clean default for short papers, memos, and general TeX handoff.",
    bestFor: ["Short reports", "Technical notes", "General-purpose export"],
    source: "built-in",
  },
  {
    id: "business-report",
    label: "Business Report",
    summary: "Executive-facing report layout with stronger headings and audit metadata.",
    bestFor: ["Consulting reports", "Board packs", "Management documents"],
    source: "built-in",
  },
  {
    id: "proposal",
    label: "Proposal",
    summary: "Proposal-oriented layout tuned for scope, approach, team, and commercial narrative.",
    bestFor: ["Proposals", "Statements of work", "Business development"],
    source: "built-in",
  },
  {
    id: "rfp-response",
    label: "RFP Response",
    summary: "Compliance-first template with room for requirements, matrices, and attachments.",
    bestFor: ["RFP responses", "Tenders", "RFQs"],
    source: "built-in",
  },
  {
    id: "technical-report",
    label: "Technical Report",
    summary: "Report class with math, figures, long tables, source references, and appendices.",
    bestFor: ["Architecture reports", "Research reports", "Engineering documentation"],
    source: "built-in",
  },
  {
    id: "academic-paper",
    label: "Academic Paper",
    summary: "Article class with bibliography and citation-friendly defaults.",
    bestFor: ["Academic papers", "Research briefs", "Conference drafts"],
    source: "built-in",
  },
  {
    id: "textbook",
    label: "Textbook",
    summary: "Book class structure for chapter-based long-form technical or instructional material.",
    bestFor: ["Textbooks", "Tutorial manuals", "Course material"],
    source: "built-in",
  },
  {
    id: "book",
    label: "Book",
    summary: "Book class structure for long-form manuscripts and multi-chapter documents.",
    bestFor: ["Books", "Novels", "Long-form manuscripts"],
    source: "built-in",
  },
];
const builtInLatexTemplateIds = new Set(latexTemplateProfiles.map((profile) => profile.id));

export function latexTemplateProfileById(id: string) {
  return latexTemplateProfiles.find((profile) => profile.id === id) || latexTemplateProfiles[0];
}

export function lookupLatexTemplateProfile(id: string, profiles: LatexTemplateProfile[]): LatexTemplateProfile {
  return profiles.find((profile) => profile.id === id) || latexTemplateProfiles[0];
}

export function createCustomLatexTemplateId(name: string) {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
  return `custom-latex-${slug || Date.now().toString(36)}`;
}

export function blankCustomLatexTemplateProfile(): CustomLatexTemplateProfile {
  return {
    id: "",
    name: "Company LaTeX template",
    summary: "Reusable organization or publisher LaTeX profile.",
    documentClass: "article",
    classOptions: "11pt",
    packages: [
      "\\usepackage[utf8]{inputenc}",
      "\\usepackage[T1]{fontenc}",
      "\\usepackage{geometry}",
      "\\usepackage{hyperref}",
      "\\usepackage{longtable}",
      "\\usepackage{graphicx}",
    ],
    geometry: "margin=1in",
    hypersetup: "colorlinks=true,linkcolor=blue,urlcolor=blue",
    header: "",
    chapterStyle: false,
    bestFor: ["Company reports", "Client deliverables"],
    sourcePath: "",
  };
}

function normalizedWorkspaceRoot(root: string) {
  return root.trim().replace(/[\\/]+$/g, "");
}

export function workspaceLatexTemplateLibraryPath(root: string) {
  const normalizedRoot = normalizedWorkspaceRoot(root);
  return normalizedRoot ? `${normalizedRoot}/.neditor/latex-templates.json` : ".neditor/latex-templates.json";
}

export function workspaceLatexTemplatesFromJson(text: string): CustomLatexTemplateProfile[] {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return [];
  }
  const templates = Array.isArray(value)
    ? value
    : typeof value === "object" && value !== null && Array.isArray((value as Record<string, unknown>).templates)
      ? ((value as Record<string, unknown>).templates as unknown[])
      : [];
  return normalizeLatexTemplateLibraryItems(templates);
}

export function workspaceLatexTemplateLibraryJson(templates: CustomLatexTemplateProfile[]) {
  const normalized = normalizeLatexTemplateLibraryItems(templates).map((template) => ({
    id: template.id,
    name: template.name,
    summary: template.summary,
    documentClass: template.documentClass,
    classOptions: template.classOptions,
    packages: template.packages,
    geometry: template.geometry,
    hypersetup: template.hypersetup,
    header: template.header,
    chapterStyle: template.chapterStyle,
    bestFor: template.bestFor,
    sourcePath: template.sourcePath,
  }));
  return `${JSON.stringify({ schema: "neditor.workspace-latex-templates.v1", templates: normalized }, null, 2)}\n`;
}

function normalizeLatexTemplateLibraryItems(value: unknown): CustomLatexTemplateProfile[] {
  if (!Array.isArray(value)) return [];
  const mapped = value.map((item) => {
    if (!item || typeof item !== "object") return item;
    const record = item as Record<string, unknown>;
    return {
      id: record.id,
      name: record.name ?? record.label,
      summary: record.summary,
      documentClass: record.documentClass ?? record.document_class,
      classOptions: record.classOptions ?? record.class_options,
      packages: record.packages,
      geometry: record.geometry,
      hypersetup: record.hypersetup,
      header: record.header,
      chapterStyle: record.chapterStyle ?? record.chapter_style,
      bestFor: record.bestFor ?? record.best_for,
      sourcePath: record.sourcePath ?? record.source_path,
    };
  });
  return normalizeCustomLatexTemplateProfiles(mapped);
}

export function latexTemplateProfilesForPicker(customTemplates: CustomLatexTemplateProfile[]): LatexTemplateProfile[] {
  return [
    ...latexTemplateProfiles,
    ...customTemplates.map((template) => ({
      id: template.id,
      label: template.name,
      summary: template.summary || `Custom ${template.documentClass} LaTeX profile.`,
      bestFor: template.bestFor.length ? template.bestFor : ["Custom LaTeX handoff"],
      source: "custom" as const,
    })),
  ];
}

export function latexTemplateExportProfileForSelection(
  selectedId: string,
  customTemplates: CustomLatexTemplateProfile[],
): LatexTemplateExportProfile | null {
  if (builtInLatexTemplateIds.has(selectedId)) return null;
  const template = customTemplates.find((item) => item.id === selectedId);
  if (!template) return null;
  return {
    id: template.id,
    label: template.name,
    documentClass: template.documentClass,
    classOptions: template.classOptions || "11pt",
    packages: template.packages,
    geometry: template.geometry || "margin=1in",
    hypersetup: template.hypersetup || "colorlinks=true,linkcolor=blue,urlcolor=blue",
    header: template.header,
    chapterStyle: template.chapterStyle,
  };
}

export function saveCustomLatexTemplateProfileState(
  templates: CustomLatexTemplateProfile[],
  draft: CustomLatexTemplateProfile,
  createId: (name: string) => string = createCustomLatexTemplateId,
) {
  const draftId = draft.id.trim();
  const id = !draftId || builtInLatexTemplateIds.has(draftId) ? createId(draft.name) : draftId;
  const profile: CustomLatexTemplateProfile = {
    ...blankCustomLatexTemplateProfile(),
    ...draft,
    id,
    name: draft.name.trim() || "Company LaTeX template",
    packages: draft.packages.map((item) => item.trim()).filter(Boolean).slice(0, 24),
    bestFor: draft.bestFor.map((item) => item.trim()).filter(Boolean).slice(0, 8),
  };
  const existing = templates.some((template) => template.id === id);
  const nextTemplates = existing ? templates.map((template) => (template.id === id ? profile : template)) : [...templates, profile].slice(0, 40);
  return {
    profile,
    templates: nextTemplates,
    changed: true,
    statusMessage: `Saved LaTeX template "${profile.name}"`,
  };
}

export function deleteCustomLatexTemplateProfileState(
  templates: CustomLatexTemplateProfile[],
  activeTemplateId: string,
  id: string,
) {
  const profile = templates.find((template) => template.id === id);
  return {
    templates: templates.filter((template) => template.id !== id),
    activeTemplateId: activeTemplateId === id ? "article" : activeTemplateId,
    changed: templates.some((template) => template.id === id),
    statusMessage: profile ? `Deleted LaTeX template "${profile.name}"` : "",
  };
}
