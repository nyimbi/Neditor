export interface CommandPaletteSearchable {
  name: string;
  group: string;
  description?: string;
  keywords?: string[];
}

type CommandMetadataValue = string | number | boolean | null | undefined;

export function compactCommandKeywords(values: CommandMetadataValue[]): string[] {
  return values
    .map((value) => (typeof value === "string" ? value.trim() : value === null || value === undefined || value === false ? "" : String(value)))
    .filter(Boolean);
}

export function joinCommandDescription(values: CommandMetadataValue[]): string {
  return compactCommandKeywords(values).join(" | ");
}

export function commandSearchText(command: CommandPaletteSearchable): string {
  return compactCommandKeywords([command.name, command.group, command.description || "", ...(command.keywords || [])])
    .join(" ")
    .toLowerCase();
}
