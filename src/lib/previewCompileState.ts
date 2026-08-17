export interface PreviewCompileSuccessInput {
  startedAtMs: number;
  finishedAtMs: number;
  textLength: number;
  diagnosticCount: number;
  compiledAt?: string;
}

export type CompileErrorKind = "compile-failed" | "backend-unavailable" | "transform-error" | "";

export function beginPreviewCompileState() {
  return {
    compileBusy: true,
    compileProgress: "Compiling preview",
  };
}

export function finishPreviewCompileState() {
  return {
    compileBusy: false,
    compileProgress: "",
  };
}

export function cancelPreviewCompileState() {
  return {
    compileBusy: false,
    compileProgress: "",
    statusMessage: "Cancelled preview compile",
  };
}

export function applyPreviewCompileSuccessState(input: PreviewCompileSuccessInput) {
  return {
    lastPreviewCompileDurationMs: Math.max(0, Math.round(input.finishedAtMs - input.startedAtMs)),
    lastPreviewCompiledCharacters: Math.max(0, Math.trunc(input.textLength)),
    lastPreviewCompiledAt: input.compiledAt || new Date().toISOString(),
    statusMessage: `${Math.max(0, Math.trunc(input.diagnosticCount))} diagnostics`,
    lastError: "",
    previewFailed: false,
    consecutiveCompileFailures: 0,
    lastCompileErrorKind: "" as CompileErrorKind,
    lastCompileErrorMessage: "",
  };
}

export function applyPreviewCompileFailureState(error: unknown, backendUnavailable = false) {
  if (backendUnavailable) {
    return {
      lastError: "",
      statusMessage: "Editing locally; preview backend unavailable in browser",
      previewFailed: true,
      lastCompileErrorKind: "backend-unavailable" as CompileErrorKind,
      lastCompileErrorMessage: "Preview backend not available in this environment",
    };
  }
  const message = previewCompileErrorText(error);
  const kind: CompileErrorKind = detectCompileErrorKind(error);
  return {
    lastError: message,
    previewFailed: true,
    lastCompileErrorKind: kind,
    lastCompileErrorMessage: message,
  };
}

function detectCompileErrorKind(error: unknown): CompileErrorKind {
  const msg = error instanceof Error ? error.message : String(error);
  if (msg.includes("transform") || msg.includes("engine")) return "transform-error";
  return "compile-failed";
}

function previewCompileErrorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
