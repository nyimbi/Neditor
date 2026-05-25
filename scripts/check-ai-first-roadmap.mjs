import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const roadmapPath = join(root, "docs", "ai-first-platform-roadmap.md");
const readmePath = join(root, "README.md");
const matrixPath = join(root, "docs", "spec-completion-matrix.md");
const appPath = join(root, "src", "App.vue");
const unitTestPath = join(root, "tests", "frontend-unit.test.ts");
const e2ePath = join(root, "e2e", "app-workflows.spec.ts");
const outputDir = join(root, ".tmp", "ai-first-roadmap");
const reportPath = join(outputDir, "report.json");

const roadmap = readFileSync(roadmapPath, "utf8");
const readme = readFileSync(readmePath, "utf8");
const matrix = readFileSync(matrixPath, "utf8");
const app = readFileSync(appPath, "utf8");
const unitTest = readFileSync(unitTestPath, "utf8");
const e2e = readFileSync(e2ePath, "utf8");

const issues = [];
const items = parseRoadmapItems(roadmap);
const sections = parseRoadmapSections(roadmap);

if (!/^# AI-First Platform Roadmap/m.test(roadmap)) {
  issues.push("roadmap title is missing");
}
if (!roadmap.includes("Operating Principles")) {
  issues.push("roadmap operating principles are missing");
}
if (!roadmap.includes("Near-Term Implementation Order")) {
  issues.push("roadmap implementation order is missing");
}
if (items.length !== 50) {
  issues.push(`roadmap must contain exactly 50 numbered changes, found ${items.length}`);
}
for (let index = 0; index < items.length; index += 1) {
  const expected = index + 1;
  if (items[index].number !== expected) {
    issues.push(`roadmap item ${index + 1} should be numbered ${expected}, found ${items[index].number}`);
  }
}
for (const section of sections) {
  if (section.items.length !== 5) {
    issues.push(`roadmap section "${section.title}" should contain 5 changes, found ${section.items.length}`);
  }
}

requireText(readme, "docs/ai-first-platform-roadmap.md", "README links the AI-first roadmap");
requireText(readme, "50 concrete", "README describes the roadmap as 50 concrete changes");
requireText(matrix, "docs/ai-first-platform-roadmap.md", "spec matrix cites the AI-first roadmap");
requireText(matrix, "pnpm run check:ai-roadmap", "spec matrix cites the roadmap verifier");

const productSurfaceChecks = [
  ["Docs Live voice drafting", app],
  ["AI Agent Workspace", app],
  ["Document intent sheet", app],
  ["Agent workflow playbooks", app],
  ["AI control center", app],
  ["Agent lifecycle task board", app],
  ["Agent source pack builder", app],
  ["AI-first platform roadmap", app],
  ["Provider profile", app],
  ["Run provider request", app],
  ["AI runtime readiness", app],
  ["NEditor guided demo", app],
  ["buildAgenticWorkflowRun", unitTest],
  ["agentic workflow run", unitTest],
  ["AI provider packages", unitTest],
  ["AI agent workspace", e2e],
  ["Guided demo", e2e],
];

for (const [needle, haystack] of productSurfaceChecks) {
  requireText(haystack, needle, `implemented/tested AI-first surface: ${needle}`);
}

const report = {
  schema: "neditor.ai-first-roadmap-report.v1",
  generatedAt: new Date().toISOString(),
  status: issues.length ? "failed" : "passed",
  roadmapPath: relative(roadmapPath),
  itemCount: items.length,
  sectionCount: sections.length,
  sections: sections.map((section) => ({
    title: section.title,
    itemCount: section.items.length,
    items: section.items.map((item) => item.title),
  })),
  productSurfaceChecks: productSurfaceChecks.map(([needle]) => needle),
  issues,
};

mkdirSync(outputDir, { recursive: true });
writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);

if (issues.length) {
  console.error("AI-first roadmap validation failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  console.error(`Wrote ${relative(reportPath)}.`);
  process.exit(1);
}

console.log(`AI-first roadmap contract passed for ${items.length} changes across ${sections.length} sections; wrote ${relative(reportPath)}.`);

function parseRoadmapSections(text) {
  const parsed = [];
  let current = null;
  for (const line of text.split(/\r?\n/)) {
    const heading = line.match(/^##\s+(\d+\.\s+.+)$/);
    if (heading) {
      current = {
        title: heading[1],
        items: [],
      };
      parsed.push(current);
      continue;
    }
    const item = parseItem(line);
    if (item && current) current.items.push(item);
  }
  return parsed.filter((section) => /^\d+\./.test(section.title));
}

function parseRoadmapItems(text) {
  return text
    .split(/\r?\n/)
    .map(parseItem)
    .filter(Boolean);
}

function parseItem(line) {
  const match = line.match(/^(\d+)\.\s+\*\*(.+?)\*\*:\s+(.+)$/);
  if (!match) return null;
  return {
    number: Number(match[1]),
    title: match[2].trim(),
    detail: match[3].trim(),
  };
}

function requireText(haystack, needle, label) {
  if (!haystack.includes(needle)) {
    issues.push(`${label} is missing ${JSON.stringify(needle)}`);
  }
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
