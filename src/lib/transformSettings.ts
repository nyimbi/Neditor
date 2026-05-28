export type TransformInputModePreference = "stdin" | "file";

export interface TransformProbeResult {
  ok: boolean;
  message: string;
  diagnostics: string[];
  cacheKey?: string;
}

export interface TransformEnginePathState {
  transformEnginePaths: Record<string, string>;
  trustedTransformEngines: Record<string, boolean>;
  transformProbeResults: Record<string, TransformProbeResult>;
}

export function updateTransformEnginePathState(
  state: TransformEnginePathState,
  name: string,
  path: string,
): TransformEnginePathState {
  const previousPath = state.transformEnginePaths[name] || "";
  const trustedAfterPathChange = previousPath === path ? Boolean(state.trustedTransformEngines[name]) : false;
  const trustRequiresReview = Boolean(path) && !trustedAfterPathChange;
  return {
    transformEnginePaths: { ...state.transformEnginePaths, [name]: path },
    trustedTransformEngines: { ...state.trustedTransformEngines, [name]: trustedAfterPathChange },
    transformProbeResults: {
      ...state.transformProbeResults,
      [name]: {
        ok: false,
        message: "Probe required after engine path change.",
        diagnostics: [
          ...(trustRequiresReview ? ["Trust was cleared because the executable path changed."] : []),
          "Run a probe to verify the configured engine path.",
        ],
      },
    },
  };
}

export function setTransformBooleanFlag(
  flags: Record<string, boolean>,
  name: string,
  value: boolean,
): Record<string, boolean> {
  return { ...flags, [name]: value };
}

export function setTransformInputModeState(
  modes: Record<string, TransformInputModePreference>,
  name: string,
  mode: TransformInputModePreference,
): Record<string, TransformInputModePreference> {
  return { ...modes, [name]: mode };
}

export function clampTransformTimeout(timeoutMs: number): number {
  return Math.min(Math.max(Number(timeoutMs) || 1, 1), 30000);
}
