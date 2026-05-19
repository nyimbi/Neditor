export interface ConflictDiffRow {
  key: string;
  kind: "equal" | "local" | "external";
  local: string;
  external: string;
  localLine: number | null;
  externalLine: number | null;
}

export function buildConflictDiff(localText: string, externalText: string): ConflictDiffRow[] {
  const localLines = localText.split(/\r?\n/);
  const externalLines = externalText.split(/\r?\n/);
  if (localLines.length * externalLines.length > 250_000) {
    return [
      {
        key: "large-conflict",
        kind: "local",
        local: `${localLines.length} local lines`,
        external: `${externalLines.length} external lines`,
        localLine: null,
        externalLine: null,
      },
    ];
  }

  const scores = Array.from({ length: localLines.length + 1 }, () => Array(externalLines.length + 1).fill(0));
  for (let localIndex = localLines.length - 1; localIndex >= 0; localIndex -= 1) {
    for (let externalIndex = externalLines.length - 1; externalIndex >= 0; externalIndex -= 1) {
      scores[localIndex][externalIndex] =
        localLines[localIndex] === externalLines[externalIndex]
          ? scores[localIndex + 1][externalIndex + 1] + 1
          : Math.max(scores[localIndex + 1][externalIndex], scores[localIndex][externalIndex + 1]);
    }
  }

  const rows: ConflictDiffRow[] = [];
  let localIndex = 0;
  let externalIndex = 0;
  while (localIndex < localLines.length || externalIndex < externalLines.length) {
    const key = `${localIndex}:${externalIndex}:${rows.length}`;
    if (localIndex < localLines.length && externalIndex < externalLines.length && localLines[localIndex] === externalLines[externalIndex]) {
      rows.push({
        key,
        kind: "equal",
        local: localLines[localIndex],
        external: externalLines[externalIndex],
        localLine: localIndex + 1,
        externalLine: externalIndex + 1,
      });
      localIndex += 1;
      externalIndex += 1;
    } else if (localIndex >= localLines.length) {
      rows.push({
        key,
        kind: "external",
        local: "",
        external: externalLines[externalIndex],
        localLine: null,
        externalLine: externalIndex + 1,
      });
      externalIndex += 1;
    } else if (externalIndex >= externalLines.length || scores[localIndex + 1][externalIndex] >= scores[localIndex][externalIndex + 1]) {
      rows.push({
        key,
        kind: "local",
        local: localLines[localIndex],
        external: "",
        localLine: localIndex + 1,
        externalLine: null,
      });
      localIndex += 1;
    } else {
      rows.push({
        key,
        kind: "external",
        local: "",
        external: externalLines[externalIndex],
        localLine: null,
        externalLine: externalIndex + 1,
      });
      externalIndex += 1;
    }
  }
  return rows;
}
