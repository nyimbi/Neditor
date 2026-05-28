export type PublishingTargetKind = "generic-webhook" | "wordpress-rest" | "ghost-admin" | "substack-manual";
export type PublishingContentFormat = "html" | "markdown" | "text";

export interface PublishingHandoffInput {
  title: string;
  compiledMarkdown: string;
  html: string;
  text?: string;
  metadata?: Record<string, unknown>;
  exportTarget?: string;
  sourceHash?: string;
  readiness?: { ready: boolean; error_count: number; warning_count: number; info_count: number } | null;
  appVersion?: string | null;
}

export interface PublishingEndpointInput {
  targetKind: PublishingTargetKind;
  endpointUrl: string;
  contentFormat: PublishingContentFormat;
  authHeaderName?: string;
  authToken?: string;
}

export interface PublishingHandoff {
  title: string;
  slug: string;
  status: string;
  description: string;
  tags: string[];
  canonicalUrl: string;
  language: string;
  exportTarget: string;
  readinessLabel: string;
  sourceHash: string;
  appVersion: string;
  markdown: string;
  html: string;
  text: string;
  checklist: Array<{ id: string; label: string; status: "ready" | "needs-review"; detail: string }>;
}

export interface PublishingRequestPreview {
  canSend: boolean;
  method: "POST";
  url: string;
  headers: Record<string, string>;
  body: Record<string, unknown>;
  bodyText: string;
  warnings: string[];
}

export const publishingTargetLabels: Record<PublishingTargetKind, string> = {
  "generic-webhook": "Generic webhook",
  "wordpress-rest": "WordPress REST draft",
  "ghost-admin": "Ghost Admin draft",
  "substack-manual": "Substack manual handoff",
};

export function buildPublishingHandoff(input: PublishingHandoffInput): PublishingHandoff {
  const metadata = input.metadata || {};
  const title = stringValue(metadata.title) || input.title.trim() || "Untitled document";
  const description =
    firstString(metadata, ["description", "summary", "subtitle", "excerpt"]) || firstParagraph(input.compiledMarkdown);
  const tags = stringList(metadata.tags).length ? stringList(metadata.tags) : stringList(metadata.keywords);
  const status = stringValue(metadata.status) || "draft";
  const readiness = input.readiness;
  const checklist: PublishingHandoff["checklist"] = [
    {
      id: "readiness",
      label: "Export readiness",
      status: readiness?.ready ? "ready" : "needs-review",
      detail: readiness
        ? `${readiness.error_count} errors, ${readiness.warning_count} warnings, ${readiness.info_count} info`
        : "Run Prepare for export before external publishing.",
    },
    {
      id: "description",
      label: "Public summary",
      status: description ? "ready" : "needs-review",
      detail: description ? description : "Add a description, summary, subtitle, or excerpt.",
    },
    {
      id: "tags",
      label: "Discovery tags",
      status: tags.length ? "ready" : "needs-review",
      detail: tags.length ? tags.join(", ") : "Add tags or keywords for publishing archives.",
    },
    {
      id: "approval",
      label: "Approval status",
      status: ["approved", "published"].includes(status.toLowerCase()) ? "ready" : "needs-review",
      detail:
        ["approved", "published"].includes(status.toLowerCase())
          ? `Status is ${status}.`
          : "Keep as draft or in-review until a reviewer approves the post.",
    },
  ];

  return {
    title,
    slug: slugValue(metadata.slug) || slugValue(title),
    status,
    description,
    tags,
    canonicalUrl: firstString(metadata, ["canonicalUrl", "canonical_url", "url"]),
    language: firstString(metadata, ["language", "lang", "locale"]) || "en",
    exportTarget: input.exportTarget || "blog",
    readinessLabel: readiness
      ? readiness.ready
        ? "ready"
        : `${readiness.error_count} errors, ${readiness.warning_count} warnings`
      : "readiness not run",
    sourceHash: input.sourceHash || "",
    appVersion: input.appVersion || "",
    markdown: input.compiledMarkdown,
    html: input.html,
    text: input.text || markdownToPlainText(input.compiledMarkdown),
    checklist,
  };
}

export function buildPublishingRequestPreview(
  handoff: PublishingHandoff,
  input: PublishingEndpointInput,
): PublishingRequestPreview {
  const warnings: string[] = [];
  const url = input.endpointUrl.trim();
  const canUseUrl = isAllowedPublishingUrl(url);
  if (!url) warnings.push("Add an endpoint URL before sending.");
  if (url && !canUseUrl) warnings.push("Use HTTPS, or HTTP only for localhost/private development endpoints.");
  if (input.targetKind === "substack-manual") warnings.push("Substack is configured as a copy/paste handoff.");

  const body = publishingBodyForTarget(handoff, input);
  const headers: Record<string, string> = { "Content-Type": "application/json" };
  const authHeaderName = (input.authHeaderName || "").trim();
  const authToken = (input.authToken || "").trim();
  if (authHeaderName && authToken) headers[authHeaderName] = authToken;

  return {
    canSend: Boolean(url && canUseUrl && input.targetKind !== "substack-manual"),
    method: "POST",
    url,
    headers,
    body,
    bodyText: JSON.stringify(body, null, 2),
    warnings,
  };
}

export function publishingPrimaryContent(handoff: PublishingHandoff, format: PublishingContentFormat) {
  if (format === "markdown") return handoff.markdown;
  if (format === "text") return handoff.text;
  return handoff.html;
}

export function publishingTargetHelp(targetKind: PublishingTargetKind) {
  if (targetKind === "wordpress-rest") return "Posts a draft-shaped JSON body to a WordPress posts endpoint.";
  if (targetKind === "ghost-admin") return "Builds a Ghost Admin draft payload; configure your endpoint/auth proxy before sending.";
  if (targetKind === "substack-manual") return "Creates copy-ready HTML and metadata because Substack publishing normally happens in its editor.";
  return "Posts a portable NEditor publishing packet to a webhook, automation, or CMS bridge.";
}

function publishingBodyForTarget(handoff: PublishingHandoff, input: PublishingEndpointInput): Record<string, unknown> {
  const content = publishingPrimaryContent(handoff, input.contentFormat);
  if (input.targetKind === "wordpress-rest") {
    return {
      title: handoff.title,
      slug: handoff.slug,
      status: "draft",
      content,
      excerpt: handoff.description,
      meta: publishingAuditMetadata(handoff),
    };
  }
  if (input.targetKind === "ghost-admin") {
    return {
      posts: [
        {
          title: handoff.title,
          slug: handoff.slug,
          status: "draft",
          html: input.contentFormat === "html" ? handoff.html : content,
          custom_excerpt: handoff.description,
          tags: handoff.tags.map((name) => ({ name })),
          metadata: publishingAuditMetadata(handoff),
        },
      ],
    };
  }
  return {
    packageType: "neditor-publishing-handoff",
    target: handoff.exportTarget,
    title: handoff.title,
    slug: handoff.slug,
    status: handoff.status,
    description: handoff.description,
    canonicalUrl: handoff.canonicalUrl,
    language: handoff.language,
    tags: handoff.tags,
    contentFormat: input.contentFormat,
    content,
    markdown: handoff.markdown,
    html: handoff.html,
    text: handoff.text,
    audit: publishingAuditMetadata(handoff),
  };
}

function publishingAuditMetadata(handoff: PublishingHandoff) {
  return {
    sourceHash: handoff.sourceHash,
    appVersion: handoff.appVersion,
    readiness: handoff.readinessLabel,
    checklist: handoff.checklist,
  };
}

function isAllowedPublishingUrl(value: string) {
  try {
    const url = new URL(value);
    if (url.protocol === "https:") return true;
    if (url.protocol !== "http:") return false;
    const host = url.hostname.toLowerCase();
    return host === "localhost" || host === "127.0.0.1" || host === "::1" || host.endsWith(".local");
  } catch {
    return false;
  }
}

function firstString(metadata: Record<string, unknown>, keys: string[]) {
  for (const key of keys) {
    const value = stringValue(metadata[key]);
    if (value) return value;
  }
  return "";
}

function stringValue(value: unknown) {
  return typeof value === "string" ? value.trim() : "";
}

function stringList(value: unknown): string[] {
  if (Array.isArray(value)) return value.map(stringValue).filter(Boolean);
  if (typeof value === "string") return value.split(",").map((item) => item.trim()).filter(Boolean);
  return [];
}

function slugValue(value: unknown) {
  return stringValue(value)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function firstParagraph(markdown: string) {
  return markdown
    .split(/\n{2,}/)
    .map((block) => block.trim())
    .find((block) => block && !block.startsWith("#") && !block.startsWith("---"))
    ?.replace(/\s+/g, " ")
    .slice(0, 280) || "";
}

function markdownToPlainText(markdown: string) {
  return markdown
    .replace(/```[\s\S]*?```/g, "")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/!\[[^\]]*]\([^)]*\)/g, "")
    .replace(/\[([^\]]+)]\([^)]*\)/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/[*_~>#-]/g, "")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}
