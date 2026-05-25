import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const matrixPath = join(root, "docs", "spec-completion-matrix.md");
const outputDir = join(root, ".tmp", "spec-completion");
const reportPath = join(outputDir, "report.json");
const allowedStatuses = new Set(["Complete", "Partial", "Unverified", "Missing", "Deferred"]);

const markdown = readFileSync(matrixPath, "utf8");
const rows = parseMatrixRows(markdown);
const issues = validateMatrix(markdown, rows);
const summary = summarizeRows(rows);
const status = issues.length ? "failed" : summary.openRows > 0 ? "partial-with-release-risks" : "complete";

mkdirSync(outputDir, { recursive: true });
writeFileSync(
  reportPath,
  `${JSON.stringify(
    {
      schema: "neditor.spec-completion-report.v1",
      generatedAt: new Date().toISOString(),
      status,
      matrixPath: relative(matrixPath),
      summary,
      issues,
      openRows: rows
        .filter((row) => ["Partial", "Unverified", "Missing"].includes(row.status))
        .map((row) => ({
          specSection: row.specSection,
          requirementArea: row.requirementArea,
          status: row.status,
          remainingGap: row.remainingGap,
        })),
    },
    null,
    2,
  )}\n`,
);

if (issues.length) {
  console.error("Spec completion matrix validation failed:");
  for (const issue of issues) console.error(`- ${issue}`);
  console.error(`Wrote ${relative(reportPath)}.`);
  process.exit(1);
}

console.log(`Spec completion matrix is ${status}; wrote ${relative(reportPath)}.`);

function parseMatrixRows(text) {
  const parsed = [];
  let section = "";
  const lines = text.split(/\r?\n/);
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const heading = line.match(/^##\s+(.+)$/);
    if (heading) section = heading[1].trim();
    if (!line.startsWith("|")) continue;
    if (/^\|\s*-+/.test(line)) continue;
    if (line.includes("Requirement area") && line.includes("Current status")) continue;

    const cells = splitTableRow(line);
    if (cells.length !== 5) continue;
    parsed.push({
      section,
      line: index + 1,
      specSection: cleanCell(cells[0]),
      requirementArea: cleanCell(cells[1]),
      status: cleanCell(cells[2]),
      evidence: cleanCell(cells[3]),
      remainingGap: cleanCell(cells[4]),
    });
  }
  return parsed;
}

function validateMatrix(text, rows) {
  const foundIssues = [];
  if (!/^# NEditor Specification Completion Matrix/m.test(text)) {
    foundIssues.push("matrix title is missing");
  }
  if (!/^Updated:\s+\d{4}-\d{2}-\d{2}$/m.test(text)) {
    foundIssues.push("Updated date must use YYYY-MM-DD");
  }
  if (!text.includes("Status vocabulary:")) {
    foundIssues.push("status vocabulary section is missing");
  }
  if (!text.includes("Current major verification gaps:")) {
    foundIssues.push("Current major verification gaps section is missing");
  }
  if (!text.includes("## Next Matrix Work")) {
    foundIssues.push("Next Matrix Work section is missing");
  }
  if (rows.length < 45) {
    foundIssues.push(`matrix has too few requirement rows: ${rows.length}`);
  }

  const duplicateKeys = new Set();
  const seenKeys = new Set();
  for (const row of rows) {
    const key = `${row.specSection}::${row.requirementArea}`;
    if (seenKeys.has(key)) duplicateKeys.add(key);
    seenKeys.add(key);
    if (!row.specSection) foundIssues.push(`line ${row.line}: spec section is empty`);
    if (!row.requirementArea) foundIssues.push(`line ${row.line}: requirement area is empty`);
    if (!allowedStatuses.has(row.status)) foundIssues.push(`line ${row.line}: invalid status ${JSON.stringify(row.status)}`);
    if (!row.evidence || isPlaceholder(row.evidence)) foundIssues.push(`line ${row.line}: evidence is empty or placeholder`);
    if (!row.remainingGap || isPlaceholder(row.remainingGap)) foundIssues.push(`line ${row.line}: remaining gap is empty or placeholder`);
    if (["Partial", "Unverified", "Missing"].includes(row.status) && !hasSubstantiveGap(row.remainingGap)) {
      foundIssues.push(`line ${row.line}: ${row.status} row must name a substantive remaining gap`);
    }
    if (row.status === "Complete" && /\b(missing|unverified|not implemented|todo|tbd)\b/i.test(row.evidence)) {
      foundIssues.push(`line ${row.line}: Complete row evidence contains unresolved language`);
    }
  }

  for (const duplicate of duplicateKeys) {
    foundIssues.push(`duplicate requirement row: ${duplicate}`);
  }

  return foundIssues;
}

function summarizeRows(rows) {
  const byStatus = Object.fromEntries([...allowedStatuses].map((status) => [status, 0]));
  for (const row of rows) byStatus[row.status] = (byStatus[row.status] || 0) + 1;
  const openRows = Number(byStatus.Partial || 0) + Number(byStatus.Unverified || 0) + Number(byStatus.Missing || 0);
  return {
    totalRows: rows.length,
    completeRows: Number(byStatus.Complete || 0),
    partialRows: Number(byStatus.Partial || 0),
    unverifiedRows: Number(byStatus.Unverified || 0),
    missingRows: Number(byStatus.Missing || 0),
    deferredRows: Number(byStatus.Deferred || 0),
    openRows,
    sections: Array.from(new Set(rows.map((row) => row.section))).filter(Boolean),
  };
}

function splitTableRow(line) {
  const cells = [];
  let cell = "";
  let escaped = false;
  const trimmed = line.trim().replace(/^\|/, "").replace(/\|$/, "");
  for (const char of trimmed) {
    if (char === "|" && !escaped) {
      cells.push(cell);
      cell = "";
    } else {
      cell += char;
    }
    escaped = char === "\\" && !escaped;
    if (char !== "\\") escaped = false;
  }
  cells.push(cell);
  return cells;
}

function cleanCell(value) {
  return value.replace(/\\\|/g, "|").replace(/<br\s*\/?>/gi, " ").replace(/\s+/g, " ").trim();
}

function isPlaceholder(value) {
  return /^(?:n\/a|none|tbd|todo|-|\.)$/i.test(value.trim());
}

function hasSubstantiveGap(value) {
  return value.trim().length >= 12 && !/^(?:keep current|revisit only if|no gap)$/i.test(value.trim());
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
