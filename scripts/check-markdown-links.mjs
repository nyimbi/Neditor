import { existsSync, readFileSync } from "node:fs";
import { extname, relative, resolve } from "node:path";
import process from "node:process";

const root = process.cwd();
const markdownFiles = [
  "README.md",
  "docs/dependency-admission.md",
  "docs/external-transforms.md",
  "docs/ipc-command-coverage.md",
  "docs/progress.md",
  "docs/security-threat-model.md",
  "docs/spec-completion-matrix.md",
  "docs/specification.md",
  "docs/storage-model.md",
  "docs/todo.md",
];
const linkPattern = /!?\[[^\]]*]\(([^)]+)\)/g;
const absoluteUrlPattern = /^[a-zA-Z][a-zA-Z0-9+.-]*:/;
const allowedExtensions = new Set([
  "",
  ".bib",
  ".csv",
  ".json",
  ".md",
  ".svg",
  ".tsv",
  ".yaml",
  ".yml",
]);

const missing = [];

for (const file of markdownFiles) {
  const absoluteFile = resolve(root, file);
  const text = readFileSync(absoluteFile, "utf8");
  for (const match of text.matchAll(linkPattern)) {
    const rawTarget = match[1].trim();
    const target = normalizeTarget(rawTarget);
    if (!target) {
      continue;
    }
    if (!allowedExtensions.has(extname(target))) {
      continue;
    }
    const resolved = resolve(absoluteFile, "..", target);
    if (relative(root, resolved).startsWith("..")) {
      continue;
    }
    if (!existsSync(resolved)) {
      const line = text.slice(0, match.index).split("\n").length;
      missing.push(`${file}:${line}: ${rawTarget}`);
    }
  }
}

if (missing.length > 0) {
  console.error("Missing local markdown links:");
  for (const entry of missing) {
    console.error(`- ${entry}`);
  }
  process.exit(1);
}

console.log(`Checked ${markdownFiles.length} markdown files; local links resolve.`);

function normalizeTarget(rawTarget) {
  if (!rawTarget || rawTarget.startsWith("#") || absoluteUrlPattern.test(rawTarget)) {
    return null;
  }
  const target = rawTarget.split(/\s+/)[0].replace(/^<|>$/g, "");
  if (!target || target.startsWith("#")) {
    return null;
  }
  return decodeURIComponent(target.split("#", 1)[0]);
}
