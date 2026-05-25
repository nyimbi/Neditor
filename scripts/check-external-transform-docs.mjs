import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const docPath = join(root, "docs", "external-transforms.md");
const outputPath = join(root, ".tmp", "external-transform-docs", "report.json");
const markdown = readFileSync(docPath, "utf8");
const windowsSection = section(markdown, "Windows");

const requirements = [
  ["Windows section exists", () => windowsSection.length > 0],
  ["Graphviz winget command", () => windowsSection.includes("winget install Graphviz.Graphviz")],
  ["D2 winget command", () => windowsSection.includes("winget install Terrastruct.D2")],
  ["Java runtime winget command", () => windowsSection.includes("winget install EclipseAdoptium.Temurin.21.JRE")],
  ["PlantUML winget command", () => windowsSection.includes("winget install PlantUML.PlantUML")],
  ["Rustup command for Pikchr CLI", () => windowsSection.includes("winget install Rustlang.Rustup")],
  ["Pikchr CLI install command", () => windowsSection.includes("cargo install pikchr-cli --locked")],
  ["Graphviz DOT path", () => windowsSection.includes("C:\\Program Files\\Graphviz\\bin\\dot.exe")],
  ["Graphviz layout paths", () => ["circo", "neato", "fdp", "osage", "twopi"].every((name) => windowsSection.includes(`C:\\Program Files\\Graphviz\\bin\\${name}.exe`))],
  ["D2 executable path", () => windowsSection.includes("d2.exe")],
  ["PlantUML executable path", () => windowsSection.includes("plantuml.exe")],
  ["Pikchr CLI executable path", () => windowsSection.includes("C:\\Users\\<you>\\.cargo\\bin\\pikchr-cli.exe")],
  ["PowerShell Pikchr probe", () => windowsSection.includes("$env:NEDITOR_TEST_PIKCHR") && windowsSection.includes("pnpm run check:engines")],
  ["Shim guidance", () => /package manager shim/i.test(windowsSection) && /normal terminal/i.test(windowsSection)],
  ["Explicit path preference", () => /Prefer explicit executable paths/i.test(windowsSection)],
];

const checks = requirements.map(([name, predicate]) => {
  let passed = false;
  try {
    passed = Boolean(predicate());
  } catch {
    passed = false;
  }
  return { name, status: passed ? "passed" : "failed" };
});
const issues = checks.filter((check) => check.status !== "passed");

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(
  outputPath,
  `${JSON.stringify(
    {
      schema: "neditor.external-transform-docs.v1",
      generatedAt: new Date().toISOString(),
      status: issues.length > 0 ? "failed" : "passed",
      docPath: "docs/external-transforms.md",
      checks,
      issues,
    },
    null,
    2,
  )}\n`,
);

if (issues.length > 0) {
  console.error("External transform documentation check failed:");
  for (const issue of issues) console.error(`- ${issue.name}`);
  console.error(`Wrote ${relative(outputPath)}.`);
  process.exit(1);
}

console.log(`External transform documentation check passed; wrote ${relative(outputPath)}.`);

function section(text, heading) {
  const lines = text.split(/\r?\n/);
  const start = lines.findIndex((line) => line.trim() === `## ${heading}`);
  if (start < 0) return "";
  const end = lines.findIndex((line, index) => index > start && /^##\s+/.test(line));
  return lines.slice(start + 1, end < 0 ? undefined : end).join("\n");
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
