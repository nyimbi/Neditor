import type { DocumentDiagnostic, ExportProgressStep, ExportReadinessReport } from "../types.js";

export interface ExportDocumentResponse {
  output_path: string;
  manifest_path?: string | null;
  diagnostics?: DocumentDiagnostic[];
  progress_steps?: ExportProgressStep[];
}

export function beginExportWorkflowState() {
  return {
    exportBusy: true,
    lastExportOutputPath: "",
    lastExportManifestPath: "",
    lastExportDiagnostics: [] as DocumentDiagnostic[],
    lastExportProgressSteps: [] as ExportProgressStep[],
    lastError: "",
  };
}

export function finishExportWorkflowState() {
  return {
    exportBusy: false,
    exportProgress: "",
    lastExportProgressSteps: [] as ExportProgressStep[],
  };
}

export function exportProgressState(exportProgress: string) {
  return { exportProgress };
}

export function applyExportSuccessState(response: ExportDocumentResponse) {
  return {
    lastExportOutputPath: response.output_path,
    lastExportManifestPath: response.manifest_path || "",
    lastExportDiagnostics: response.diagnostics || [],
    lastExportProgressSteps: response.progress_steps || [],
    lastError: "",
    statusMessage: `Exported ${response.output_path}${response.manifest_path ? ` with manifest ${response.manifest_path}` : ""}`,
  };
}

export function applyExportFailureState(error: unknown, sourceFile: string | null, exportTarget: string) {
  const message = error instanceof Error ? error.message : String(error);
  return {
    lastError: message,
    lastExportDiagnostics: [
      {
        severity: "error",
        message,
        source_file: sourceFile,
        line: null,
        column: null,
        end_line: null,
        end_column: null,
        suggestion: "Review export readiness diagnostics and target settings before retrying.",
        related: [exportTarget],
      },
    ] satisfies DocumentDiagnostic[],
    statusMessage: `Export failed: ${message}`,
  };
}

export function beginExportReadinessState() {
  return {
    exportBusy: true,
    lastError: "",
    exportProgress: "Checking export readiness",
  };
}

export function applyExportReadinessState(exportReadiness: ExportReadinessReport) {
  return {
    exportReadiness,
    statusMessage: exportReadiness.ready
      ? "Document is ready for export"
      : `${exportReadiness.error_count} errors, ${exportReadiness.warning_count} warnings before export`,
  };
}
