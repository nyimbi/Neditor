import type { SemanticDocument } from "../types.js";

export type ReleaseChecklistStatus = "complete" | "missing" | "needs-review";

export interface ReleaseChecklistItem {
  id: string;
  label: string;
  status: ReleaseChecklistStatus;
  detail: string;
  action: string;
}

export interface ReleaseReadinessInput {
  text: string;
  semantic?: Pick<SemanticDocument, "status" | "comments" | "change_notes" | "ai_sources" | "ai_assisted_sections"> | null;
}

export function buildReleaseReadinessChecklist(input: ReleaseReadinessInput): ReleaseChecklistItem[] {
  const text = input.text || "";
  const semantic = input.semantic;
  const status = frontMatterAnyScalar(text, ["status"]) || semantic?.status || "draft";
  const version = frontMatterAnyScalar(text, ["version"]);
  const owner = frontMatterAnyScalar(text, ["owner"]);
  const releaseTarget = frontMatterAnyScalar(text, ["releaseTarget"]);
  const approvedBy = frontMatterAnyScalar(text, ["approvedBy", "reviewer"]);
  const approvedAt = frontMatterAnyScalar(text, ["approvedAt"]);
  const comments = semantic?.comments || [];
  const unresolved = comments.filter((comment) => comment.state !== "resolved").length;
  const changeNotes = semantic?.change_notes?.length || 0;
  const aiItems = [...(semantic?.ai_sources || []), ...(semantic?.ai_assisted_sections || [])];
  const aiPending = aiItems.filter((item) => item.status !== "human-reviewed").length;
  const approved = status === "approved" || status === "published";

  return [
    {
      id: "release-state",
      label: "Release state",
      status: approved ? "complete" : "needs-review",
      detail: approved ? `${status} release state is set.` : `Current status is ${status}.`,
      action: "Set status to approved or published when the document is final.",
    },
    {
      id: "release-metadata",
      label: "Ownership metadata",
      status: version && owner && releaseTarget ? "complete" : "missing",
      detail: [
        version ? `version ${version}` : "missing version",
        owner ? `owner ${owner}` : "missing owner",
        releaseTarget ? `target ${releaseTarget}` : "missing releaseTarget",
      ].join(" | "),
      action: "Capture version, owner, and release target before export or tagging.",
    },
    {
      id: "approval-audit",
      label: "Approval audit",
      status: approvedBy && approvedAt ? "complete" : "missing",
      detail: approvedBy && approvedAt ? `${approvedBy} at ${approvedAt}` : "approvedBy/reviewer or approvedAt is missing.",
      action: "Record approver and approval timestamp for traceability.",
    },
    {
      id: "review-comments",
      label: "Review comments",
      status: unresolved ? "needs-review" : "complete",
      detail: unresolved ? `${unresolved} unresolved review comment(s).` : "No unresolved review comments.",
      action: "Resolve comments or record why they are accepted for release.",
    },
    {
      id: "change-notes",
      label: "Change notes",
      status: changeNotes ? "complete" : "needs-review",
      detail: changeNotes ? `${changeNotes} change note(s) recorded.` : "No change notes recorded.",
      action: "Add a change note for the release delta or review decision.",
    },
    {
      id: "ai-review",
      label: "AI review",
      status: aiPending ? "needs-review" : "complete",
      detail: aiPending ? `${aiPending} AI-assisted item(s) still need human review.` : "AI provenance is reviewed or not present.",
      action: "Mark AI-assisted sources and sections as human reviewed before final release.",
    },
  ];
}

export function formatReleaseChecklistSummary(items: ReleaseChecklistItem[]) {
  const counts = releaseChecklistCounts(items);
  return `${counts.complete} complete, ${counts.missing} missing, ${counts["needs-review"]} need review`;
}

export function releaseChecklistHelp(items: ReleaseChecklistItem[]) {
  return items.every((item) => item.status === "complete")
    ? "The document has the local release metadata and review audit expected before tagging or external distribution."
    : "Resolve release metadata, approvals, comments, change notes, and AI review state before final export or tagging.";
}

export function releaseReadinessAuditMarkdown(items: ReleaseChecklistItem[]) {
  const rows = items
    .map((item) => `| ${markdownTableCell(item.label)} | ${item.status} | ${markdownTableCell(item.detail)} | ${markdownTableCell(item.action)} |`)
    .join("\n");
  return `## Release Readiness Audit\n\n| Area | Status | Detail | Action |\n| --- | --- | --- | --- |\n${rows}\n`;
}

function releaseChecklistCounts(items: ReleaseChecklistItem[]) {
  return items.reduce<Record<ReleaseChecklistStatus, number>>(
    (acc, item) => {
      acc[item.status] += 1;
      return acc;
    },
    { complete: 0, missing: 0, "needs-review": 0 },
  );
}

function frontMatterAnyScalar(text: string, keys: string[]) {
  for (const key of keys) {
    const value = frontMatterScalar(text, key);
    if (value) return value;
  }
  return "";
}

function frontMatterScalar(text: string, key: string) {
  const match = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) return "";
  const keyRe = new RegExp(`^${escapeRegExp(key)}\\s*:\\s*(.+)$`, "im");
  const value = match[1].match(keyRe)?.[1] || "";
  return value.replace(/^['"]|['"]$/g, "").trim();
}

function markdownTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
