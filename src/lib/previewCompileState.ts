export interface PreviewCompileSuccessInput {
  startedAtMs: number;
  finishedAtMs: number;
  textLength: number;
  diagnosticCount: number;
  compiledAt?: string;
}

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
  };
}

export function applyPreviewCompileFailureState(error: unknown, backendUnavailable = false) {
  if (backendUnavailable) {
    return {
      lastError: "",
      statusMessage: "Editing locally; preview backend unavailable in browser",
    };
  }
  return {
    lastError: previewCompileErrorText(error),
  };
}

function previewCompileErrorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
