import { readFileSync } from "node:fs";
import process from "node:process";

const sourcePath = "src/App.vue";
const source = readFileSync(sourcePath, "utf8");
const templateStart = source.indexOf("<template>");
const scriptStart = source.indexOf("<script");
const templateEnd = source.lastIndexOf("</template>", scriptStart);
const template =
  templateStart >= 0 && templateEnd > templateStart
    ? source.slice(templateStart + "<template>".length, templateEnd)
    : "";
const issues = [];

checkButtons();
checkFormControls();
checkDialogs();
checkSkipLinks();

if (issues.length > 0) {
  console.error("Accessibility guard failed:");
  for (const issue of issues) {
    console.error(`- ${issue}`);
  }
  process.exit(1);
}

console.log("Checked App.vue template accessibility guardrails.");

function checkButtons() {
  const buttonPattern = /<button\b([^>]*)>([\s\S]*?)<\/button>/g;
  for (const match of template.matchAll(buttonPattern)) {
    const [, attrs, body] = match;
    if (hasAccessibleName(attrs)) {
      continue;
    }
    const text = visibleText(body);
    if (!text || isGenericButtonText(text)) {
      issues.push(
        `${sourcePath}:${lineFor(match.index)} button needs descriptive text or aria-label`,
      );
    }
  }
}

function checkFormControls() {
  const controlPattern = /<(input|select|textarea)\b([^>]*)>/g;
  for (const match of template.matchAll(controlPattern)) {
    const [, tag, attrs] = match;
    if (/\btype\s*=\s*["']hidden["']/.test(attrs)) {
      continue;
    }
    if (hasAccessibleName(attrs) || isInsideLabel(match.index)) {
      continue;
    }
    issues.push(`${sourcePath}:${lineFor(match.index)} ${tag} needs a label or aria-label`);
  }
}

function checkDialogs() {
  const dialogPattern = /<[^>]+\brole\s*=\s*["']dialog["'][^>]*>/g;
  for (const match of template.matchAll(dialogPattern)) {
    const attrs = match[0];
    if (!/\baria-modal\s*=\s*["']true["']/.test(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} dialog must set aria-modal="true"`);
    }
    if (!hasAccessibleName(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} dialog needs aria-label or aria-labelledby`);
    }
    if (!/\btabindex\s*=\s*["']-1["']/.test(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} dialog must be programmatically focusable`);
    }
    if (!/@keydown\s*=/.test(attrs)) {
      issues.push(`${sourcePath}:${lineFor(match.index)} dialog must handle keyboard focus trapping and Escape`);
    }
  }
}

function checkSkipLinks() {
  const skipLinks = template.match(/<nav\b[^>]*class\s*=\s*["'][^"']*\bskip-links\b[^"']*["'][\s\S]*?<\/nav>/);
  if (!skipLinks) {
    issues.push(`${sourcePath}:1 skip-links navigation is missing`);
    return;
  }
  const hrefPattern = /href\s*=\s*["']#([^"']+)["']/g;
  const targets = [...skipLinks[0].matchAll(hrefPattern)].map((match) => match[1]);
  const requiredTargets = [
    "main-commands",
    "document-workspace",
    "document-sidebar",
    "markdown-source",
    "live-preview",
    "document-status",
  ];
  for (const target of requiredTargets) {
    if (!targets.includes(target)) {
      issues.push(`${sourcePath}:1 skip-links navigation is missing #${target}`);
      continue;
    }
    const targetPattern = new RegExp(`\\bid\\s*=\\s*["']${target}["'][^>]*\\btabindex\\s*=\\s*["']-1["']|\\btabindex\\s*=\\s*["']-1["'][^>]*\\bid\\s*=\\s*["']${target}["']`);
    if (!targetPattern.test(template)) {
      issues.push(`${sourcePath}:1 skip target #${target} must exist and be programmatically focusable`);
    }
  }
}

function hasAccessibleName(attrs) {
  return /\b(:?aria-label|aria-labelledby|title)\s*=/.test(attrs);
}

function isInsideLabel(index) {
  const before = template.slice(0, index);
  return before.lastIndexOf("<label") > before.lastIndexOf("</label>");
}

function visibleText(body) {
  return body
    .replace(/<[^>]*>/g, " ")
    .replace(/\{\{[^}]+\}\}/g, " value ")
    .replace(/\s+/g, " ")
    .trim();
}

function isGenericButtonText(text) {
  return ["x", "×", "use"].includes(text.toLowerCase());
}

function lineFor(index) {
  return template.slice(0, index).split("\n").length + 1;
}
