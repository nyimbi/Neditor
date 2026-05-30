export type AccessibilityQaStatus = "ready" | "needs-review" | "blocked";

export interface AccessibilityQaInput {
  highContrast: boolean;
  reducedMotion: boolean;
  toolbarDisplay: string;
  commandCount: number;
  menuCount: number;
  helpTopicCount: number;
  currentMode: string;
  currentSidebar: string;
  hasSkipLinks?: boolean;
  hasHoverHelp?: boolean;
  hasStatusRegion?: boolean;
  manualAssistiveTechSignoff?: boolean;
}

export interface AccessibilityQaItem {
  id: string;
  label: string;
  status: AccessibilityQaStatus;
  detail: string;
  action: string;
}

export interface AccessibilityQaReport {
  status: AccessibilityQaStatus;
  summary: string;
  items: AccessibilityQaItem[];
  counts: Record<AccessibilityQaStatus, number>;
}

export function buildAccessibilityQaReport(input: AccessibilityQaInput): AccessibilityQaReport {
  const items: AccessibilityQaItem[] = [
    {
      id: "keyboard-command-access",
      label: "Keyboard command access",
      status: input.commandCount >= 50 && input.menuCount >= 5 ? "ready" : "needs-review",
      detail: `${input.commandCount} command palette actions and ${input.menuCount} menus are available from the keyboard.`,
      action: input.commandCount >= 50 ? "Keep command names plain and searchable as new workflows are added." : "Expose missing workflows through the command palette and menus.",
    },
    {
      id: "skip-links-landmarks",
      label: "Skip links and workbench landmarks",
      status: input.hasSkipLinks === false ? "blocked" : "ready",
      detail: input.hasSkipLinks === false ? "Skip-link evidence is missing from the current shell." : "Skip links and labeled workbench regions are present.",
      action: input.hasSkipLinks === false ? "Restore skip links before release." : "Verify skip links with keyboard and screen-reader smoke passes.",
    },
    {
      id: "hover-help-fallback",
      label: "Button help fallback",
      status: input.hasHoverHelp === false ? "needs-review" : "ready",
      detail: input.hasHoverHelp === false ? "Hover/focus button help was not detected." : "Button help is available through hover and focus affordances.",
      action: input.hasHoverHelp === false ? "Add title, aria-label, or data-help text to new controls." : "Keep every new button supplied with hover/focus help.",
    },
    {
      id: "motion-contrast",
      label: "Motion and contrast preferences",
      status: input.highContrast || input.reducedMotion ? "ready" : "needs-review",
      detail: `High contrast is ${input.highContrast ? "on" : "off"}; reduced motion is ${input.reducedMotion ? "on" : "off"}; toolbar buttons show ${input.toolbarDisplay}.`,
      action: input.highContrast || input.reducedMotion ? "Review the current preferences with the user profile before delivery." : "Offer high contrast and reduced motion during onboarding or accessibility review.",
    },
    {
      id: "status-region",
      label: "Status and progress feedback",
      status: input.hasStatusRegion === false ? "needs-review" : "ready",
      detail: input.hasStatusRegion === false ? "No status region evidence was supplied." : `Status feedback is visible while working in ${input.currentMode}/${input.currentSidebar}.`,
      action: input.hasStatusRegion === false ? "Restore status feedback before long-running tasks." : "Keep AI, export, save, and setup progress reflected in status messages.",
    },
    {
      id: "plain-language-help",
      label: "Plain-language help coverage",
      status: input.helpTopicCount >= 10 ? "ready" : "needs-review",
      detail: `${input.helpTopicCount} help topics are available for guided assistance.`,
      action: input.helpTopicCount >= 10 ? "Keep help topics linked to the relevant menus and commands." : "Add help topics for the highest-risk workflows before release.",
    },
    {
      id: "manual-assistive-tech-signoff",
      label: "Manual assistive-technology sign-off",
      status: input.manualAssistiveTechSignoff ? "ready" : "needs-review",
      detail: input.manualAssistiveTechSignoff ? "Manual assistive-technology sign-off is recorded." : "Manual screen-reader sign-off is still required for release evidence.",
      action: input.manualAssistiveTechSignoff ? "Archive sign-off with the release evidence bundle." : "Run the manual accessibility sign-off checklist with VoiceOver or another screen reader.",
    },
  ];
  const counts = {
    ready: items.filter((item) => item.status === "ready").length,
    "needs-review": items.filter((item) => item.status === "needs-review").length,
    blocked: items.filter((item) => item.status === "blocked").length,
  };
  const status: AccessibilityQaStatus = counts.blocked ? "blocked" : counts["needs-review"] ? "needs-review" : "ready";
  return {
    status,
    counts,
    summary: `${counts.ready} ready | ${counts["needs-review"]} need review | ${counts.blocked} blocked`,
    items,
  };
}

export function accessibilityQaMarkdown(report: AccessibilityQaReport, generatedAt = new Date().toISOString()) {
  return [
    "## Accessibility QA Report",
    "",
    `Status: ${report.status}`,
    `Generated: ${generatedAt}`,
    `Summary: ${report.summary}`,
    "",
    "| Area | Status | Detail | Action |",
    "| --- | --- | --- | --- |",
    ...report.items.map((item) => `| ${escapeTableCell(item.label)} | ${item.status} | ${escapeTableCell(item.detail)} | ${escapeTableCell(item.action)} |`),
    "",
  ].join("\n");
}

function escapeTableCell(value: string) {
  return value.replace(/\|/g, "\\|").replace(/\r?\n/g, " ").trim();
}
