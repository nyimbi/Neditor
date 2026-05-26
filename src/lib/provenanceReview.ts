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
