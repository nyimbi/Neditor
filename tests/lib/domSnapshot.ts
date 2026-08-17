/**
 * DOM snapshot helper for node --test.
 *
 * Usage:
 *   await domSnapshot("workbench-default", wrapper.element.outerHTML);
 *
 * On first run (or UPDATE_SNAPSHOTS=1) writes the snapshot file.
 * On subsequent runs compares and throws with a diff on mismatch.
 *
 * Dynamic-strip rules (applied before write/compare):
 *   - id="uid-\d+" / aria-labelledby="uid-\d+" → id="uid-STABLE"
 *   - data-uid-* attributes → data-uid-STABLE
 *   - "\d+s elapsed" / "\d+:\d+" time patterns → "Ns elapsed"
 *   - ISO timestamps (YYYY-MM-DDTHH:MM:SS…) → "TIMESTAMP"
 *   - Inline style properties that contain px measurements from layout → stripped
 *   - Whitespace between tags normalised; whitespace inside text nodes preserved.
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

// Resolve snapshot directory relative to THIS source file's location.
// At runtime this file is at .tmp-tests/tests/lib/domSnapshot.js, so we
// need two directories up from the source tree, then into tests/snapshots.
// We use import.meta.url to be runtime-location independent.
const HERE = dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = join(HERE, "..", "..", "..", "tests", "snapshots");

export function stripDynamic(html: string): string {
  return html
    // Stable UID attributes
    .replace(/\bid="uid-\d+"/g, 'id="uid-STABLE"')
    .replace(/\baria-labelledby="uid-\d+"/g, 'aria-labelledby="uid-STABLE"')
    .replace(/\baria-controls="uid-\d+"/g, 'aria-controls="uid-STABLE"')
    .replace(/\bdata-uid-[a-z0-9-]+="\d+"/g, 'data-uid-STABLE="STABLE"')
    // Elapsed-time text
    .replace(/\b\d+s elapsed\b/g, "Ns elapsed")
    .replace(/\b\d{1,2}:\d{2}\b/g, "N:NN")
    // ISO 8601 timestamps
    .replace(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?/g, "TIMESTAMP")
    // CSS transition / animation timings that vary
    .replace(/\btransition-duration:\s*[\d.]+ms\b/g, "transition-duration:Nms")
    // Inline style heights/widths set by JS layout (CodeMirror, resize handles)
    .replace(/\bstyle="(?:[^"]*\b(?:height|width|top|left|transform):[^;"]*;?\s*)+"/g, 'style="LAYOUT"')
    // Normalize whitespace between tags, preserve within text nodes
    .replace(/>\s+</g, ">\n<")
    .trim();
}

export async function domSnapshot(name: string, html: string): Promise<void> {
  mkdirSync(SNAPSHOT_DIR, { recursive: true });

  const stable = stripDynamic(html);
  const snapshotPath = join(SNAPSHOT_DIR, `${name}.html`);
  const update = process.env["UPDATE_SNAPSHOTS"] === "1";

  if (!existsSync(snapshotPath) || update) {
    writeFileSync(snapshotPath, stable, "utf8");
    if (update) {
      console.log(`[snapshot] updated: ${name}.html`);
    } else {
      console.log(`[snapshot] created: ${name}.html`);
    }
    return;
  }

  const stored = readFileSync(snapshotPath, "utf8");
  if (stored === stable) return;

  // Produce a line-level diff for the failure message.
  const storedLines = stored.split("\n");
  const stableLines = stable.split("\n");
  const maxLines = Math.max(storedLines.length, stableLines.length);
  const diffLines: string[] = [];
  let diffCount = 0;

  for (let i = 0; i < maxLines; i++) {
    const a = storedLines[i] ?? "<missing>";
    const b = stableLines[i] ?? "<missing>";
    if (a !== b) {
      diffCount++;
      if (diffLines.length < 40) {
        diffLines.push(`line ${i + 1}:`);
        diffLines.push(`  - ${a}`);
        diffLines.push(`  + ${b}`);
      }
    }
  }

  const truncated = diffCount > 40 ? `\n  … (${diffCount - 40} more diff lines)` : "";
  throw new Error(
    `Snapshot mismatch for "${name}" (${diffCount} line${diffCount === 1 ? "" : "s"} differ):\n` +
      diffLines.join("\n") +
      truncated +
      `\n\nRun UPDATE_SNAPSHOTS=1 pnpm run test:unit to refresh.`,
  );
}
