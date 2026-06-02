export interface AiRuntimeReadinessReport {
  generatedAt: string;
  secureContext: boolean;
  speechRecognition: AiRuntimeCapability;
  microphonePermission: AiRuntimeCapability;
  clipboardRead: AiRuntimeCapability;
  clipboardWrite: AiRuntimeCapability;
  issues: string[];
  markdown: string;
}

export interface AiRuntimeCapability {
  supported: boolean;
  state: "available" | "granted" | "prompt" | "denied" | "unavailable" | "unknown";
  detail: string;
}

export interface AiRuntimeReadinessEnvironment {
  now?: () => string;
  secureContext?: boolean;
  hasSpeechRecognition?: boolean;
  queryPermission?: (name: "microphone" | "clipboard-read" | "clipboard-write") => Promise<PermissionState | "unknown">;
  readClipboard?: () => Promise<{ kind: "rich" | "plain"; length: number } | null>;
  canWriteClipboard?: boolean;
  clipboardTimeoutMs?: number;
}

export async function inspectAiRuntimeReadiness(environment: AiRuntimeReadinessEnvironment = {}): Promise<AiRuntimeReadinessReport> {
  const generatedAt = environment.now?.() || new Date().toISOString();
  const secureContext = environment.secureContext ?? runtimeSecureContext();
  const hasSpeech = environment.hasSpeechRecognition ?? runtimeHasSpeechRecognition();
  const microphoneState = await queryPermissionState(environment, "microphone");
  const clipboardReadState = await queryPermissionState(environment, "clipboard-read");
  const clipboardWriteState = await queryPermissionState(environment, "clipboard-write");
  const clipboardReadProof = await probeClipboardRead(environment);
  const canWriteClipboard = environment.canWriteClipboard ?? runtimeCanWriteClipboard();

  const speechRecognition = capability(
    hasSpeech && secureContext,
    hasSpeech ? (secureContext ? "available" : "unavailable") : "unavailable",
    hasSpeech ? (secureContext ? "SpeechRecognition API is available." : "SpeechRecognition requires a secure context.") : "SpeechRecognition API is unavailable.",
  );
  const microphonePermission = capability(
    hasSpeech && microphoneState !== "denied",
    microphoneState,
    permissionDetail("microphone", microphoneState),
  );
  const clipboardRead = capability(
    Boolean(clipboardReadProof) || (clipboardReadState !== "denied" && runtimeCanReadClipboard()),
    clipboardReadProof ? "granted" : clipboardReadState,
    clipboardReadProof
      ? `Clipboard ${clipboardReadProof.kind} read succeeded (${clipboardReadProof.length} characters detected, content not stored).`
      : permissionDetail("clipboard-read", clipboardReadState),
  );
  const clipboardWrite = capability(
    canWriteClipboard && clipboardWriteState !== "denied",
    canWriteClipboard ? clipboardWriteState : "unavailable",
    canWriteClipboard ? permissionDetail("clipboard-write", clipboardWriteState) : "Clipboard write API is unavailable.",
  );
  const issues = buildIssues(secureContext, speechRecognition, microphonePermission, clipboardRead, clipboardWrite);
  const report: Omit<AiRuntimeReadinessReport, "markdown"> = {
    generatedAt,
    secureContext,
    speechRecognition,
    microphonePermission,
    clipboardRead,
    clipboardWrite,
    issues,
  };

  return {
    ...report,
    markdown: readinessMarkdown(report),
  };
}

function capability(supported: boolean, state: AiRuntimeCapability["state"] | PermissionState | "unknown", detail: string): AiRuntimeCapability {
  return {
    supported,
    state: normalizeState(state),
    detail,
  };
}

async function queryPermissionState(
  environment: AiRuntimeReadinessEnvironment,
  name: "microphone" | "clipboard-read" | "clipboard-write",
): Promise<PermissionState | "unknown"> {
  try {
    if (environment.queryPermission) return await environment.queryPermission(name);
    const permissions = navigator.permissions;
    if (!permissions?.query) return "unknown";
    const status = await permissions.query({ name: name as PermissionName });
    return status.state;
  } catch {
    return "unknown";
  }
}

async function probeClipboardRead(environment: AiRuntimeReadinessEnvironment) {
  try {
    if (environment.readClipboard) return await environment.readClipboard();
    const clipboard = navigator.clipboard as unknown as
      | {
          read?: () => Promise<Array<{ types: readonly string[]; getType(type: string): Promise<Blob> }>>;
          readText?: () => Promise<string>;
        }
      | undefined;
    if (!clipboard) return null;
    const timeoutMs = environment.clipboardTimeoutMs ?? 400;
    if (clipboard.read) {
      const items = await bounded(clipboard.read(), timeoutMs);
      for (const item of items || []) {
        const type = ["text/html", "text/plain"].find((candidate) => item.types.includes(candidate));
        if (!type) continue;
        const text = await (await item.getType(type)).text();
        return { kind: type === "text/html" ? "rich" : "plain", length: text.length } as const;
      }
    }
    if (clipboard.readText) {
      const text = await bounded(clipboard.readText(), timeoutMs);
      return text ? { kind: "plain" as const, length: text.length } : null;
    }
  } catch {
    return null;
  }
  return null;
}

function bounded<T>(task: Promise<T>, timeoutMs = 400) {
  return Promise.race<T | null>([task, new Promise((resolve) => globalThis.setTimeout(() => resolve(null), timeoutMs))]);
}

function runtimeSecureContext() {
  return Boolean(globalThis.isSecureContext);
}

function runtimeHasSpeechRecognition() {
  const speechWindow = window as Window & {
    SpeechRecognition?: unknown;
    webkitSpeechRecognition?: unknown;
  };
  return Boolean(speechWindow.SpeechRecognition || speechWindow.webkitSpeechRecognition);
}

function runtimeCanReadClipboard() {
  return Boolean(navigator.clipboard?.read || navigator.clipboard?.readText);
}

function runtimeCanWriteClipboard() {
  return Boolean(navigator.clipboard?.writeText);
}

function normalizeState(state: PermissionState | "available" | "unavailable" | "unknown"): AiRuntimeCapability["state"] {
  if (state === "granted" || state === "prompt" || state === "denied" || state === "available" || state === "unavailable") return state;
  return "unknown";
}

function permissionDetail(name: string, state: PermissionState | "available" | "unavailable" | "unknown") {
  if (state === "granted") return `${name} permission is granted.`;
  if (state === "prompt") return `${name} permission will prompt when used.`;
  if (state === "denied") return `${name} permission is denied.`;
  if (state === "unavailable") return `${name} capability is unavailable.`;
  return `${name} permission state is unknown in this runtime.`;
}

function buildIssues(
  secureContext: boolean,
  speechRecognition: AiRuntimeCapability,
  microphonePermission: AiRuntimeCapability,
  clipboardRead: AiRuntimeCapability,
  clipboardWrite: AiRuntimeCapability,
) {
  return [
    secureContext ? "" : "AI voice and clipboard APIs require a secure runtime context.",
    speechRecognition.supported ? "" : speechRecognition.detail,
    microphonePermission.state === "denied" ? microphonePermission.detail : "",
    clipboardRead.supported ? "" : clipboardRead.detail,
    clipboardWrite.supported ? "" : clipboardWrite.detail,
  ].filter(Boolean);
}

function readinessMarkdown(report: Omit<AiRuntimeReadinessReport, "markdown">) {
  return [
    "# AI Runtime Readiness",
    "",
    `Generated: ${report.generatedAt}`,
    "",
    `Secure context: ${report.secureContext ? "yes" : "no"}`,
    "",
    "| Capability | Supported | State | Detail |",
    "| --- | --- | --- | --- |",
    readinessRow("Speech recognition", report.speechRecognition),
    readinessRow("Microphone permission", report.microphonePermission),
    readinessRow("Clipboard read", report.clipboardRead),
    readinessRow("Clipboard write", report.clipboardWrite),
    "",
    "## Issues",
    "",
    ...(report.issues.length ? report.issues.map((issue) => `- ${issue}`) : ["- No blocking runtime issues detected."]),
    "",
  ].join("\n");
}

function readinessRow(label: string, capability: AiRuntimeCapability) {
  return `| ${label} | ${capability.supported ? "yes" : "no"} | ${capability.state} | ${capability.detail.replace(/\|/g, "/")} |`;
}
