/**
 * Presentation editor: slide parsing (mirrors Rust build_pptx_slides),
 * theme definitions, and speaker notes management.
 */

export type SlideLayout = "title" | "section" | "content" | "two-column";
export type PresentationTheme = "corporate" | "minimal" | "dark" | "nature" | "warm";
export type PresentationTransition = "none" | "fade" | "push" | "wipe" | "zoom";

export interface SlideData {
  index: number;
  title: string;
  lines: string[];
  layout: SlideLayout;
  notes: string;
  sourceLine: number;
}

export interface PresentationThemeInfo {
  id: PresentationTheme;
  label: string;
  bg: string;
  text: string;
  accent: string;
  previewStart: string;
  previewEnd: string;
}

export const PRESENTATION_THEMES: PresentationThemeInfo[] = [
  { id: "corporate", label: "Corporate Blue", bg: "#1f3a5f", text: "#ffffff", accent: "#4b9cd3", previewStart: "#1f3a5f", previewEnd: "#2d5f8a" },
  { id: "minimal",   label: "Minimal",        bg: "#ffffff", text: "#1e293b", accent: "#275DA8", previewStart: "#ffffff", previewEnd: "#e2e8f0" },
  { id: "dark",      label: "Dark Pro",        bg: "#0f172a", text: "#f1f5f9", accent: "#0f766e", previewStart: "#0f172a", previewEnd: "#1e293b" },
  { id: "nature",    label: "Nature",          bg: "#1a3326", text: "#f0fdf4", accent: "#4ade80", previewStart: "#1a3326", previewEnd: "#14532d" },
  { id: "warm",      label: "Warm",            bg: "#2d1b0e", text: "#fefce8", accent: "#f59e0b", previewStart: "#2d1b0e", previewEnd: "#451a03" },
];

export const PRESENTATION_TRANSITIONS: Array<{ id: PresentationTransition; label: string }> = [
  { id: "none",  label: "None (instant)" },
  { id: "fade",  label: "Fade" },
  { id: "push",  label: "Push" },
  { id: "wipe",  label: "Wipe" },
  { id: "zoom",  label: "Zoom" },
];

export function themeById(id: string): PresentationThemeInfo {
  return PRESENTATION_THEMES.find(t => t.id === id) ?? PRESENTATION_THEMES[0];
}

export function parseSlidesFromBlocks(
  blocks: Array<{ kind: string; level?: number; text?: string; items?: string[]; headers?: string[]; rows?: string[][]; caption?: string; directive?: string; settings?: { title?: string; layout?: string; notes?: string }; line: number }>,
  documentTitle: string,
  metadata: Record<string, unknown>,
): SlideData[] {
  const slides: SlideData[] = [];

  // Title slide
  slides.push({
    index: 0, title: documentTitle,
    lines: [metadata.author, metadata.date, metadata.subtitle].filter(Boolean).map(String),
    layout: "title", notes: "", sourceLine: 0,
  });

  let title = "";
  let lines: string[] = [];
  let layout: SlideLayout = "content";
  let notes = "";
  let sourceLine = 0;

  const flush = () => {
    if (title || lines.length > 0) {
      slides.push({ index: slides.length, title, lines: [...lines], layout, notes, sourceLine });
    }
    title = ""; lines = []; layout = "content"; notes = "";
  };

  for (const block of blocks) {
    if (block.kind === "heading" && (block.level ?? 1) <= 2) {
      flush(); title = block.text ?? ""; sourceLine = block.line;
    } else if (block.kind === "heading") {
      lines.push(block.text ?? "");
    } else if (block.kind === "paragraph") {
      lines.push(block.text ?? "");
    } else if (block.kind === "list") {
      lines.push(...(block.items ?? []));
    } else if (block.kind === "table") {
      lines.push(`[Table: ${block.caption ?? (block.headers ?? []).join(", ")}]`);
    } else if (block.kind === "layout") {
      const d = block.directive ?? "";
      if (d === "section-break" || d === "slide") {
        flush(); title = block.settings?.title ?? ""; sourceLine = block.line;
        const lv = block.settings?.layout ?? "";
        layout = (d === "section-break" || lv === "section") ? "section" : (lv === "two-column" || lv === "columns") ? "two-column" : "content";
        notes = block.settings?.notes ?? "";
      } else if (d === "page-break") {
        flush(); sourceLine = block.line;
      }
    }
  }
  flush();
  return slides;
}

export function upsertSlideNotes(source: string, slideTitle: string, newNotes: string): string {
  if (!slideTitle.trim()) return source;
  // Preserve literal content; normalise internal newlines only
  const safe = newNotes.replace(/\n/g, " ").replace(/"/g, "'").trim();
  const escaped = slideTitle.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

  // 1. Update existing {{ slide / section-break }} directive.
  //    Use (?:[^}]|\}(?!\}))* so that a lone } inside a directive value
  //    (e.g. footer="{budget}") does not prematurely terminate the match.
  const di = "(?:[^}]|\\}(?!\\}))*?";
  const directivePattern = new RegExp(
    `(\\{\\{\\s*(?:slide|section-break)\\s${di}title="${escaped}"${di})(?:\\s+notes="[^"]*")?(${di}\\}\\})`,
    "g",
  );
  let replaced = false;
  let out = source.replace(directivePattern, (_m, before, after) => {
    replaced = true;
    return safe ? `${before} notes="${safe}"${after}` : `${before}${after}`;
  });
  if (replaced) return out;

  // 2. Update an existing neditor-notes comment (from a previous call).
  const ce = slideTitle.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const commentPattern = new RegExp(`<!--\\s*neditor-slide-notes:\\s*"${ce}"[^>]*-->`, "g");
  out = source.replace(commentPattern, () => {
    replaced = true;
    return safe ? `<!-- neditor-slide-notes: "${slideTitle}" notes="${safe}" -->` : "";
  });
  if (replaced) return out;

  // 3. No directive found — insert a comment before the matching heading.
  //    This handles the common case of heading-based slides (## Title).
  if (safe) {
    const headingPattern = new RegExp(`^(#{1,2}[ \\t]+${escaped}[ \\t]*)$`, "m");
    const inserted = source.replace(headingPattern, (match) =>
      `<!-- neditor-slide-notes: "${slideTitle}" notes="${safe}" -->\n${match}`,
    );
    if (inserted !== source) return inserted;
  }

  return source;
}
