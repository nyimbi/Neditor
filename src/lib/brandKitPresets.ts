import {
  normalizeBrandProfileDefaults,
  normalizeExportDefaults,
  type BrandProfileDefaults,
  type ExportDefaults,
} from "./workspacePersistence.js";

export interface BrandKitPreset {
  id: string;
  label: string;
  summary: string;
  bestFor: string[];
  designNotes: string[];
  brand: Partial<BrandProfileDefaults>;
  exportDefaults: Partial<ExportDefaults>;
}

export interface AppliedBrandKitPresetState {
  brandProfileDefaults: BrandProfileDefaults;
  exportDefaults: ExportDefaults;
}

export interface BrandKitPreviewRow {
  label: string;
  value: string;
}

export const brandKitPresets: BrandKitPreset[] = [
  {
    id: "board-memo",
    label: "Board Memo",
    summary: "Executive, restrained, and confidential for board packs and decision memos.",
    bestFor: ["Board packs", "Decision memos", "Executive approvals"],
    designNotes: ["Compact page flow", "Confidential watermark", "Page numbers on"],
    brand: {
      name: "Board Office",
      color: "#174A7C",
      font: "Aptos, Arial, sans-serif",
      header: "{{title}}",
      footer: "Board pack | {{status}} | v{{version}}",
      watermark: "Confidential",
      legalDisclaimer: "Prepared for board review. Do not distribute without authorization.",
    },
    exportDefaults: {
      coverPage: true,
      pageNumbers: true,
      layoutPreset: "compact",
      latexTemplate: "business-report",
      includeComments: true,
      includeProvenance: true,
      includeGlossary: false,
      includeAgenda: true,
    },
  },
  {
    id: "consulting-report",
    label: "Consulting Report",
    summary: "Polished advisory report defaults for client-facing findings and recommendations.",
    bestFor: ["Client reports", "Strategy decks", "Operating model reviews"],
    designNotes: ["Business layout", "Cover page on", "Comments hidden from delivery"],
    brand: {
      name: "Advisory Practice",
      color: "#006B5F",
      font: "Aptos, Arial, sans-serif",
      header: "{{client}} | {{title}}",
      footer: "{{company}} advisory report | {{date}}",
      watermark: "",
      legalDisclaimer: "This document is provided for client use under the applicable engagement terms.",
    },
    exportDefaults: {
      coverPage: true,
      pageNumbers: true,
      layoutPreset: "business",
      latexTemplate: "technical-report",
      includeComments: false,
      includeProvenance: true,
      includeGlossary: true,
      includeAgenda: true,
    },
  },
  {
    id: "proposal-response",
    label: "Proposal Response",
    summary: "Procurement-safe proposal styling with compliance, provenance, and appendix support enabled.",
    bestFor: ["RFP responses", "Tenders", "RFQs", "Grant proposals"],
    designNotes: ["Compliance-ready", "Glossary on", "Comments retained for internal QA"],
    brand: {
      name: "Bid Team",
      color: "#6F4E00",
      font: "Aptos, Arial, sans-serif",
      header: "{{opportunity}} | Technical proposal",
      footer: "{{company}} proposal | {{status}}",
      watermark: "Draft",
      legalDisclaimer: "Commercially confidential proposal material. Use only for the named procurement.",
    },
    exportDefaults: {
      coverPage: true,
      pageNumbers: true,
      layoutPreset: "business",
      latexTemplate: "rfp-response",
      includeComments: true,
      includeProvenance: true,
      includeGlossary: true,
      includeAgenda: false,
    },
  },
  {
    id: "policy-brief",
    label: "Policy Brief",
    summary: "Clear, compact, evidence-forward defaults for recommendations and public-interest analysis.",
    bestFor: ["Policy memos", "Briefing notes", "Public sector papers"],
    designNotes: ["Compact layout", "Source provenance on", "No watermark by default"],
    brand: {
      name: "Policy Unit",
      color: "#3F4B5F",
      font: "Aptos, Arial, sans-serif",
      header: "{{title}}",
      footer: "Policy brief | {{date}}",
      watermark: "",
      legalDisclaimer: "Recommendations are based on the sources and assumptions available at preparation time.",
    },
    exportDefaults: {
      coverPage: false,
      pageNumbers: true,
      layoutPreset: "compact",
      latexTemplate: "article",
      includeComments: false,
      includeProvenance: true,
      includeGlossary: true,
      includeAgenda: false,
    },
  },
  {
    id: "academic-article",
    label: "Academic Article",
    summary: "Citation-heavy article settings for research notes, papers, and methodological appendices.",
    bestFor: ["Research papers", "Methods notes", "Academic articles"],
    designNotes: ["Article template", "Glossary on", "Delivery comments off"],
    brand: {
      name: "Research Group",
      color: "#8A1538",
      font: "Georgia, 'Times New Roman', serif",
      header: "{{title}}",
      footer: "Research draft | {{date}}",
      watermark: "",
      legalDisclaimer: "Draft research material. Verify citations, data, and permissions before publication.",
    },
    exportDefaults: {
      coverPage: false,
      pageNumbers: true,
      layoutPreset: "business",
      latexTemplate: "academic-paper",
      includeComments: false,
      includeProvenance: true,
      includeGlossary: true,
      includeAgenda: false,
    },
  },
  {
    id: "newsletter",
    label: "Newsletter",
    summary: "Readable publishing defaults for updates, community posts, and recurring communications.",
    bestFor: ["Newsletters", "Internal updates", "Substack drafts", "Blog posts"],
    designNotes: ["Presentation layout", "Cover off", "Publishing metadata ready"],
    brand: {
      name: "Editorial Desk",
      color: "#A51C30",
      font: "Inter, Aptos, Arial, sans-serif",
      header: "{{title}}",
      footer: "Published by {{company}} | {{website}}",
      watermark: "",
      legalDisclaimer: "Prepared for publication. Confirm rights for images, quotes, and external links.",
    },
    exportDefaults: {
      coverPage: false,
      pageNumbers: false,
      layoutPreset: "presentation",
      latexTemplate: "article",
      includeComments: false,
      includeProvenance: false,
      includeGlossary: false,
      includeAgenda: false,
    },
  },
];

export function brandKitPresetById(id: string): BrandKitPreset | null {
  return brandKitPresets.find((preset) => preset.id === id) || null;
}

export function applyBrandKitPresetState(
  currentBrandProfileDefaults: Partial<BrandProfileDefaults>,
  currentExportDefaults: Partial<ExportDefaults>,
  preset: BrandKitPreset,
): AppliedBrandKitPresetState {
  return {
    brandProfileDefaults: normalizeBrandProfileDefaults({
      ...currentBrandProfileDefaults,
      ...preset.brand,
    }),
    exportDefaults: normalizeExportDefaults({
      ...currentExportDefaults,
      ...preset.exportDefaults,
    }),
  };
}

export function buildBrandKitPreviewRows(
  brandProfileDefaults: Partial<BrandProfileDefaults>,
  exportDefaults: Partial<ExportDefaults>,
): BrandKitPreviewRow[] {
  const brand = normalizeBrandProfileDefaults(brandProfileDefaults);
  const exports = normalizeExportDefaults(exportDefaults);
  return [
    { label: "Brand", value: brand.name || "Unnamed brand" },
    { label: "Color", value: brand.color },
    { label: "Typeface", value: brand.font || "Default export font" },
    { label: "Layout", value: exports.layoutPreset },
    { label: "Cover", value: exports.coverPage ? "enabled" : "off" },
    { label: "Page numbers", value: exports.pageNumbers ? "enabled" : "off" },
    { label: "Watermark", value: brand.watermark || "none" },
    { label: "LaTeX profile", value: String(exports.latexTemplate || "article") },
  ];
}
