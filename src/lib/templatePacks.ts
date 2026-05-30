import type {
  BusinessDocumentSnippet,
  BusinessDocumentTemplate,
  CustomDocumentOutlineTemplate,
  CustomVersionedBusinessClause,
  DocumentOutlineTemplate,
} from "./businessDocuments.js";
import { normalizeCustomDocumentOutlineTemplates, normalizeCustomVersionedClauses } from "./businessDocuments.js";
import type { CustomTransformTemplate, TransformTemplate } from "./transformTemplates.js";
import { normalizeCustomTransformTemplates } from "./transformTemplates.js";
import type { CustomLatexTemplateProfile } from "./workspacePersistence.js";

export const templatePackSchema = "neditor.template-pack.v1";

export interface TemplatePackMetadata {
  id: string;
  name: string;
  publisher: string;
  version: string;
  license: string;
  summary: string;
  homepage: string;
  tags: string[];
  usageGuidance: string[];
  outlineRules: string[];
  placeholders: string[];
  examples: string[];
}

export interface NeditorTemplatePack {
  schema: typeof templatePackSchema;
  metadata: TemplatePackMetadata;
  businessTemplates: BusinessDocumentTemplate[];
  snippets: BusinessDocumentSnippet[];
  outlines: CustomDocumentOutlineTemplate[];
  transforms: CustomTransformTemplate[];
  latexTemplates: CustomLatexTemplateProfile[];
  clauses: CustomVersionedBusinessClause[];
}

export interface BuildTemplatePackInput {
  name: string;
  publisher?: string;
  version?: string;
  license?: string;
  summary?: string;
  homepage?: string;
  tags?: string[];
  usageGuidance?: string[];
  outlineRules?: string[];
  examples?: string[];
  businessTemplates?: BusinessDocumentTemplate[];
  snippets?: BusinessDocumentSnippet[];
  outlines?: DocumentOutlineTemplate[];
  transforms?: TransformTemplate[];
  latexTemplates?: CustomLatexTemplateProfile[];
  clauses?: CustomVersionedBusinessClause[];
}

export interface TemplatePackInstallInput {
  existingOutlines: CustomDocumentOutlineTemplate[];
  existingTransforms: CustomTransformTemplate[];
  existingLatexTemplates: CustomLatexTemplateProfile[];
  existingClauses: CustomVersionedBusinessClause[];
  pack: NeditorTemplatePack;
}

export interface TemplatePackInstallResult {
  outlines: CustomDocumentOutlineTemplate[];
  transforms: CustomTransformTemplate[];
  latexTemplates: CustomLatexTemplateProfile[];
  clauses: CustomVersionedBusinessClause[];
  added: {
    outlines: number;
    transforms: number;
    latexTemplates: number;
    clauses: number;
  };
}

export interface TemplatePackSummaryRow {
  label: string;
  value: string;
}

export function buildTemplatePack(input: BuildTemplatePackInput): NeditorTemplatePack {
  const businessTemplates = dedupeById(input.businessTemplates || []).slice(0, 30);
  const snippets = dedupeById(input.snippets || []).slice(0, 60);
  const outlines = normalizeCustomDocumentOutlineTemplates(
    (input.outlines || []).map((template) => ({
      ...template,
      id: marketplaceId(template.id),
      source: undefined,
    })),
  ).slice(0, 40);
  const transforms = normalizeCustomTransformTemplates(
    (input.transforms || []).map((template) => ({
      ...template,
      id: marketplaceId(template.id),
      source: undefined,
    })),
  ).slice(0, 80);
  const latexTemplates = dedupeById(input.latexTemplates || []).slice(0, 20);
  const clauses = normalizeCustomVersionedClauses(input.clauses || []).slice(0, 40);
  const metadata: TemplatePackMetadata = {
    id: marketplaceId(input.name || "template-pack"),
    name: input.name.trim() || "NEditor template pack",
    publisher: input.publisher?.trim() || "Local workspace",
    version: input.version?.trim() || "1.0.0",
    license: input.license?.trim() || "Workspace use",
    summary: input.summary?.trim() || defaultPackSummary({ businessTemplates, snippets, outlines, transforms, latexTemplates, clauses }),
    homepage: input.homepage?.trim() || "",
    tags: cleanList(input.tags).slice(0, 20),
    usageGuidance: cleanList(input.usageGuidance).slice(0, 12),
    outlineRules: cleanList(input.outlineRules).slice(0, 12),
    examples: cleanList(input.examples).slice(0, 12),
    placeholders: packPlaceholders({ businessTemplates, snippets, outlines, transforms, latexTemplates, clauses }).slice(0, 80),
  };
  return {
    schema: templatePackSchema,
    metadata,
    businessTemplates,
    snippets,
    outlines,
    transforms,
    latexTemplates,
    clauses,
  };
}

export function templatePackJson(pack: NeditorTemplatePack) {
  return `${JSON.stringify(pack, null, 2)}\n`;
}

export function parseTemplatePackJson(text: string): NeditorTemplatePack | null {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  if (!value || typeof value !== "object") return null;
  const record = value as Partial<NeditorTemplatePack>;
  if (record.schema !== templatePackSchema) return null;
  return buildTemplatePack({
    name: record.metadata?.name || "Imported template pack",
    publisher: record.metadata?.publisher,
    version: record.metadata?.version,
    license: record.metadata?.license,
    summary: record.metadata?.summary,
    homepage: record.metadata?.homepage,
    tags: record.metadata?.tags || [],
    usageGuidance: record.metadata?.usageGuidance || [],
    outlineRules: record.metadata?.outlineRules || [],
    examples: record.metadata?.examples || [],
    businessTemplates: Array.isArray(record.businessTemplates) ? record.businessTemplates : [],
    snippets: Array.isArray(record.snippets) ? record.snippets : [],
    outlines: Array.isArray(record.outlines) ? record.outlines.map((template) => ({ ...template, source: "custom" })) : [],
    transforms: Array.isArray(record.transforms) ? record.transforms.map((template) => ({ ...template, source: "custom" })) : [],
    latexTemplates: Array.isArray(record.latexTemplates) ? record.latexTemplates : [],
    clauses: Array.isArray(record.clauses) ? record.clauses : [],
  });
}

export function installTemplatePackState(input: TemplatePackInstallInput): TemplatePackInstallResult {
  const outlineIds = new Set(input.existingOutlines.map((template) => template.id));
  const transformIds = new Set(input.existingTransforms.map((template) => template.id));
  const latexIds = new Set(input.existingLatexTemplates.map((template) => template.id));
  const clauseIds = new Set(input.existingClauses.map((clause) => clause.id));
  const newOutlines = input.pack.outlines.filter((template) => !outlineIds.has(template.id));
  const newTransforms = input.pack.transforms.filter((template) => !transformIds.has(template.id));
  const newLatexTemplates = input.pack.latexTemplates.filter((template) => !latexIds.has(template.id));
  const newClauses = input.pack.clauses.filter((clause) => !clauseIds.has(clause.id));
  return {
    outlines: normalizeCustomDocumentOutlineTemplates([...input.existingOutlines, ...newOutlines]),
    transforms: normalizeCustomTransformTemplates([...input.existingTransforms, ...newTransforms]),
    latexTemplates: [...input.existingLatexTemplates, ...newLatexTemplates].slice(0, 40),
    clauses: normalizeCustomVersionedClauses([...input.existingClauses, ...newClauses]),
    added: {
      outlines: newOutlines.length,
      transforms: newTransforms.length,
      latexTemplates: newLatexTemplates.length,
      clauses: newClauses.length,
    },
  };
}

export function templatePackSummaryRows(pack: NeditorTemplatePack): TemplatePackSummaryRow[] {
  return [
    { label: "Business templates", value: String(pack.businessTemplates.length) },
    { label: "Snippets", value: String(pack.snippets.length) },
    { label: "Outlines", value: String(pack.outlines.length) },
    { label: "Transform templates", value: String(pack.transforms.length) },
    { label: "LaTeX templates", value: String(pack.latexTemplates.length) },
    { label: "Versioned clauses", value: String(pack.clauses.length) },
    { label: "Placeholders", value: String(pack.metadata.placeholders.length) },
  ];
}

function defaultPackSummary(input: Pick<NeditorTemplatePack, "businessTemplates" | "snippets" | "outlines" | "transforms" | "latexTemplates" | "clauses">) {
  const total =
    input.businessTemplates.length +
    input.snippets.length +
    input.outlines.length +
    input.transforms.length +
    input.latexTemplates.length +
    input.clauses.length;
  return `Portable NEditor pack with ${total} reusable template item${total === 1 ? "" : "s"}.`;
}

function packPlaceholders(input: Pick<NeditorTemplatePack, "businessTemplates" | "snippets" | "outlines" | "transforms" | "latexTemplates" | "clauses">) {
  const text = [
    ...input.businessTemplates.flatMap((item) => [item.aiPrompt, item.outline.join("\n")]),
    ...input.snippets.map((item) => item.body),
    ...input.outlines.flatMap((item) => item.outline),
    ...input.transforms.map((item) => item.body),
    ...input.latexTemplates.flatMap((item) => [item.header, item.geometry, item.hypersetup]),
    ...input.clauses.map((item) => item.body),
  ].join("\n");
  return [...new Set([...text.matchAll(/\{\{([a-zA-Z0-9_. -]+)\}\}/g)].map((match) => match[1].trim()).filter(Boolean))].sort();
}

function cleanList(value: unknown) {
  if (Array.isArray(value)) return value.map((item) => String(item).trim()).filter(Boolean);
  if (typeof value === "string") return value.split(/\r?\n|,/).map((item) => item.trim()).filter(Boolean);
  return [];
}

function dedupeById<T extends { id: string }>(items: T[]) {
  const seen = new Set<string>();
  const result: T[] = [];
  for (const item of items) {
    if (!item?.id || seen.has(item.id)) continue;
    seen.add(item.id);
    result.push(item);
  }
  return result;
}

function marketplaceId(value: string) {
  const id = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return id || "template-pack";
}
