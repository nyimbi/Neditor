export type DocumentLayoutPresetId =
  | "two-column-section"
  | "three-column-brief"
  | "wide-landscape-section"
  | "single-column-reset";

export interface DocumentLayoutPreset {
  id: DocumentLayoutPresetId;
  label: string;
  shortLabel: string;
  summary: string;
  commandName: string;
  keywords: string[];
  snippet: string;
}

export const documentLayoutPresets: DocumentLayoutPreset[] = [
  {
    id: "two-column-section",
    label: "Two-column section",
    shortLabel: "2 Cols",
    summary: "Start a polished two-column business section with an explicit gutter for briefs, proposals, and market analysis.",
    commandName: "Insert two-column section",
    keywords: ["layout", "columns", "two column", "business brief", "proposal", "market analysis"],
    snippet: [
      "```layout",
      "columns: 2",
      "columnGap: 18pt",
      "section: business-analysis",
      "```",
      "",
      "## Business Analysis",
      "",
      "*Draft concise, column-friendly analysis here.*",
    ].join("\n"),
  },
  {
    id: "three-column-brief",
    label: "Three-column brief",
    shortLabel: "3 Cols",
    summary: "Create a dense three-column brief section for highlights, options, risks, or short executive talking points.",
    commandName: "Insert three-column brief",
    keywords: ["layout", "columns", "three column", "brief", "highlights", "options", "risk"],
    snippet: [
      "{{section-break columns=3 columnGap=14pt margins=narrow}}",
      "",
      "## Executive Highlights",
      "",
      "### Priority",
      "",
      "*Summarize the first priority.*",
      "",
      "### Evidence",
      "",
      "*Summarize the strongest evidence.*",
      "",
      "### Decision",
      "",
      "*State the decision or next action.*",
    ].join("\n"),
  },
  {
    id: "wide-landscape-section",
    label: "Wide landscape section",
    shortLabel: "Wide",
    summary: "Switch to single-column landscape layout for wide tables, timelines, compliance matrices, and appendix evidence.",
    commandName: "Insert wide landscape section",
    keywords: ["layout", "landscape", "wide table", "timeline", "compliance matrix", "appendix"],
    snippet: [
      "{{section-break columns=1 pageSize=letter orientation=landscape margins=narrow}}",
      "",
      "## Wide Evidence Section",
      "",
      "| Item | Owner | Date | Status | Evidence |",
      "| --- | --- | --- | --- | --- |",
      "| Requirement | Owner | YYYY-MM-DD | Open | Source or attachment |",
    ].join("\n"),
  },
  {
    id: "single-column-reset",
    label: "Return to single column",
    shortLabel: "1 Col",
    summary: "Reset the document back to normal portrait single-column flow after a columned or landscape section.",
    commandName: "Return to single-column layout",
    keywords: ["layout", "single column", "reset", "portrait", "normal margins"],
    snippet: [
      "{{section-break columns=1 margins=normal orientation=portrait}}",
      "",
      "## Continued Narrative",
      "",
      "*Continue the main document flow here.*",
    ].join("\n"),
  },
];

export function documentLayoutPresetById(id: DocumentLayoutPresetId) {
  return documentLayoutPresets.find((preset) => preset.id === id) || documentLayoutPresets[0];
}
