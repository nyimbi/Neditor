export type IncludeDirectiveSyntax = "bang" | "braces" | "comment";

export interface IncludeDirectiveSyntaxOption {
  value: IncludeDirectiveSyntax;
  label: string;
  detail: string;
}

export interface IncludeTargetResolution {
  path: string;
  error: string;
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

export function resolveIncludeTargetPath(parentDocumentPath: string, value: string): IncludeTargetResolution {
  const target = normalizeIncludeTarget(value);
  if (!target) return { path: "", error: "Enter a child document path." };
  if (!parentDocumentPath.trim()) return { path: "", error: "Save the parent document before creating included files." };
  if (isAbsoluteIncludeTarget(target)) return { path: "", error: "Use a relative include path so the child stays with the parent document." };

  const targetParts = target.split("/").filter(Boolean);
  if (targetParts.some((part) => part === "..")) {
    return { path: "", error: "Keep included documents inside the parent document folder." };
  }
  if (targetParts.some((part) => part === ".")) {
    return { path: "", error: "Remove standalone dot path segments from the include path." };
  }
  if (targetParts.some((part) => /[:*?"<>|]/.test(part))) {
    return { path: "", error: "Remove characters that are unsafe in portable file names." };
  }

  const parentFolder = folderFromPath(parentDocumentPath);
  if (!parentFolder) return { path: "", error: "Save the parent document in a folder before creating included files." };
  return { path: `${parentFolder}/${targetParts.join("/")}`, error: "" };
}

export function includeChildDocumentTitle(value: string) {
  const target = normalizeIncludeTarget(value);
  const fileName = target.split("/").filter(Boolean).pop() || "Included Document";
  const stem = fileName.replace(/\.[^.]+$/, "");
  const title = stem
    .replace(/[-_]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  return title ? title.replace(/\b\w/g, (character) => character.toUpperCase()) : "Included Document";
}

export function includeChildDocumentStarterMarkdown(value: string) {
  const title = includeChildDocumentTitle(value);
  const yamlTitle = title.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `---\ntitle: "${yamlTitle}"\nstatus: draft\n---\n\n# ${title}\n\nWrite this section here.\n`;
}

function isAbsoluteIncludeTarget(target: string) {
  return target.startsWith("/") || /^[A-Za-z]:\//.test(target) || target.startsWith("//");
}

function folderFromPath(path: string) {
  const normalized = path.trim().replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  return index >= 0 ? normalized.slice(0, index) || '/' : '';
}
