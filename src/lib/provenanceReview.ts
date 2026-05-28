const AI_SOURCE_FENCE_LANGUAGES = new Set([
  "ai-source",
  "ai_source",
  "ai-provenance",
  "ai_provenance",
  "llm-source",
  "llm_source",
  "llm-provenance",
  "llm_provenance",
]);

export function markdownFenceOpener(line: string) {
  const trimmed = line.trimStart();
  const marker = trimmed.startsWith("```") ? "```" : trimmed.startsWith("~~~") ? "~~~" : "";
  if (!marker) return null;
  const info = trimmed.slice(marker.length).trim();
  const language = (info.split(/\s+/).find(Boolean) || "").toLowerCase();
  return { marker, info, language };
}

export function isAiSourceFenceOpener(line: string) {
  const opener = markdownFenceOpener(line);
  return Boolean(opener && AI_SOURCE_FENCE_LANGUAGES.has(opener.language));
}

export function stripMarkdownFencedBlocks(value: string) {
  let fenceMarker = "";
  return value
    .split(/\r?\n/)
    .map((line) => {
      const trimmed = line.trimStart();
      if (fenceMarker) {
        if (trimmed.startsWith(fenceMarker)) fenceMarker = "";
        return "";
      }
      const opener = markdownFenceOpener(line);
      if (opener) {
        fenceMarker = opener.marker;
        return "";
      }
      return line;
    })
    .join("\n");
}

export function parseAiAssistedMarker(line: string) {
  const content = line.match(/<!--\s*ai-assisted:(.*?)-->/)?.[1] || "";
  const fields = new Map<string, string>();
  for (const part of content
    .split("|")
    .map((entry) => entry.trim())
    .filter(Boolean)) {
    const pair = part.match(/^([^:=]+)\s*[:=]\s*(.*)$/);
    if (pair) {
      fields.set(pair[1].trim(), pair[2].trim());
    } else if (["human-reviewed", "needs-review", "unreviewed"].includes(part)) {
      fields.set("status", part);
    }
  }
  return fields;
}

export function serializeAiAssistedMarker(fields: Map<string, string>) {
  const orderedKeys = ["status", "reviewedBy", "reviewedAt", "source", "promptSummary"];
  const parts = orderedKeys
    .filter((key) => fields.has(key))
    .map((key) => `${key}=${fields.get(key) || ""}`);
  for (const [key, value] of fields) {
    if (!orderedKeys.includes(key)) {
      parts.push(`${key}=${value}`);
    }
  }
  return `<!-- ai-assisted: ${parts.join(" | ")} -->`;
}

export function rewriteAiAssistedMarker(line: string, reviewed: boolean, reviewedAt = new Date().toISOString()) {
  const fields = line.includes("<!-- ai-assisted:")
    ? parseAiAssistedMarker(line)
    : new Map<string, string>([
        ["source", "AI paste cleanup"],
        ["promptSummary", "AI paste cleanup review required"],
      ]);
  fields.set("status", reviewed ? "human-reviewed" : "needs-review");
  fields.set("reviewedBy", reviewed ? "local" : "");
  fields.set("reviewedAt", reviewed ? reviewedAt : "");
  return serializeAiAssistedMarker(fields);
}

function rewriteYamlLikeField(lines: string[], key: string, value: string) {
  const index = lines.findIndex((line) => line.trimStart().startsWith(`${key}:`));
  const replacement = `${key}: ${value}`;
  if (index >= 0) {
    lines[index] = replacement;
  } else {
    lines.push(replacement);
  }
}

export function rewriteAiSourceReviewBlock(
  lines: string[],
  startIndex: number,
  reviewed: boolean,
  reviewedAt = new Date().toISOString(),
) {
  const opener = markdownFenceOpener(lines[startIndex] || "");
  if (!opener || !AI_SOURCE_FENCE_LANGUAGES.has(opener.language)) return false;
  const endIndex = lines.findIndex(
    (line, index) => index > startIndex && line.trimStart().startsWith(opener.marker),
  );
  if (endIndex < 0) return false;
  const body = lines.slice(startIndex + 1, endIndex);
  rewriteYamlLikeField(body, "status", reviewed ? "human-reviewed" : "needs-review");
  rewriteYamlLikeField(body, "reviewedBy", reviewed ? "local" : "");
  rewriteYamlLikeField(body, "reviewedAt", reviewed ? reviewedAt : "");
  lines.splice(startIndex + 1, endIndex - startIndex - 1, ...body);
  return true;
}
