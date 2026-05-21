import { readFileSync } from "node:fs";
import process from "node:process";

const sourcePath = "src/App.vue";
const source = readFileSync(sourcePath, "utf8");
const issues = [];

checkVueSfcBlockOrder();

if (issues.length > 0) {
  console.error("Project structure guard failed:");
  for (const issue of issues) {
    console.error(`- ${issue}`);
  }
  process.exit(1);
}

console.log("Checked project structure guardrails.");

function checkVueSfcBlockOrder() {
  const template = vueBlock("template");
  const script = firstTag("script");
  const style = firstTag("style");
  if (!template) issues.push(`${sourcePath}: missing <template> block`);
  if (!script) issues.push(`${sourcePath}: missing <script setup> block`);
  if (!style) issues.push(`${sourcePath}: missing <style> block`);
  if (!template || !script || !style) return;

  if (!/\bsetup\b/.test(script.openingTag)) {
    issues.push(`${sourcePath}:${lineFor(script.start)} script block must use <script setup>`);
  }
  if (!(template.start < script.start && script.start < style.start)) {
    issues.push(`${sourcePath}: Vue SFC blocks must stay ordered as template, script setup, style`);
  }
  if (template.end > script.start || script.end > style.start) {
    issues.push(`${sourcePath}: Vue SFC blocks must not overlap`);
  }
  if (source.slice(0, template.start).trim()) {
    issues.push(`${sourcePath}: <template> must be the first top-level block`);
  }
  assertSingleTopLevelTag("script");
  assertSingleTopLevelTag("style");
}

function vueBlock(tagName) {
  const openPattern = new RegExp(`<${tagName}\\b[^>]*>`, "gi");
  const closePattern = new RegExp(`</${tagName}>`, "gi");
  const firstOpen = openPattern.exec(source);
  if (!firstOpen) return null;
  let depth = 1;
  let cursor = firstOpen.index + firstOpen[0].length;
  while (depth > 0) {
    openPattern.lastIndex = cursor;
    closePattern.lastIndex = cursor;
    const nextOpen = openPattern.exec(source);
    const nextClose = closePattern.exec(source);
    if (!nextClose) {
      issues.push(`${sourcePath}:${lineFor(firstOpen.index)} missing </${tagName}>`);
      return null;
    }
    if (nextOpen && nextOpen.index < nextClose.index) {
      depth += 1;
      cursor = nextOpen.index + nextOpen[0].length;
    } else {
      depth -= 1;
      cursor = nextClose.index + nextClose[0].length;
    }
  }
  return {
    start: firstOpen.index,
    end: cursor,
    openingTag: firstOpen[0],
  };
}

function firstTag(tagName) {
  const tagPattern = new RegExp(`<${tagName}\\b[^>]*>`, "i");
  const open = tagPattern.exec(source);
  if (!open) return null;
  const closeTag = `</${tagName}>`;
  const closeIndex = source.indexOf(closeTag, open.index + open[0].length);
  if (closeIndex < 0) {
    issues.push(`${sourcePath}:${lineFor(open.index)} missing ${closeTag}`);
    return null;
  }
  return {
    start: open.index,
    end: closeIndex + closeTag.length,
    openingTag: open[0],
  };
}

function assertSingleTopLevelTag(tagName) {
  const matches = source.match(new RegExp(`<${tagName}\\b[^>]*>`, "gi")) || [];
  if (matches.length !== 1) {
    issues.push(`${sourcePath}: expected exactly one <${tagName}> block, found ${matches.length}`);
  }
}

function lineFor(index) {
  return source.slice(0, index).split(/\r?\n/).length;
}
