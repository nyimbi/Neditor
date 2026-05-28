export type IncludeDirectiveSyntax = "bang" | "braces" | "comment";

export interface IncludeDirectiveSyntaxOption {
  value: IncludeDirectiveSyntax;
  label: string;
  detail: string;
}

export const includeDirectiveSyntaxOptions: IncludeDirectiveSyntaxOption[] = [
  {
    value: "bang",
    label: "!include",
    detail: "Best default for readable master documents.",
  },
  {
    value: "braces",
    label: "{{include}}",
    detail: "Useful when you prefer template-style directives.",
  },
  {
    value: "comment",
    label: "HTML comment",
    detail: "Keeps the include instruction hidden in many Markdown viewers.",
  },
];

export function normalizeIncludeTarget(value: string) {
  let target = value.trim().replace(/[\r\n]+/g, " ").replace(/\\/g, "/");
  target = target.replace(/^["'`]|["'`]$/g, "").trim();
  return target.startsWith("./") ? target.slice(2) : target;
}

export function formatIncludeDirective(value: string, syntax: IncludeDirectiveSyntax = "bang") {
  const target = normalizeIncludeTarget(value);
  if (!target) return "";
  if (syntax === "braces") return `{{include ${target}}}`;
  if (syntax === "comment") return `<!-- include: ${target} -->`;
  return `!include ${target}`;
}

export function includeDirectiveHelpText(syntax: IncludeDirectiveSyntax) {
  return includeDirectiveSyntaxOptions.find((option) => option.value === syntax)?.detail || includeDirectiveSyntaxOptions[0].detail;
}
