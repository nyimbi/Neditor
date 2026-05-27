import type { TtsEngineId, TtsPreferences } from "./workspacePersistence.js";

export interface TtsEngineOption {
  id: TtsEngineId;
  label: string;
}

export interface TtsEngineStatus {
  id: string;
  label: string;
  available: boolean;
  detail: string;
}

export interface TtsInspectionReport {
  engines: TtsEngineStatus[];
}

export interface TtsModelDownloadPlan {
  engine: "supertonic-cli";
  model: string;
  approximateSize: string;
  storagePath: string;
  source: string;
  command: string;
  acknowledged: boolean;
}

export const ttsEngineOptions = [
  { id: "browser-speech", label: "Browser or system speech" },
  { id: "macos-say", label: "macOS Say" },
  { id: "supertonic-cli", label: "Supertonic CLI" },
] as const satisfies readonly TtsEngineOption[];

export const supertonicModelName = "supertonic-3";
export const supertonicModelApproximateSize = "~305 MB";
export const supertonicModelSource = "Hugging Face model download managed by the Supertonic CLI";

export function selectedTtsEngineLabel(engine: TtsEngineId) {
  return ttsEngineOptions.find((option) => option.id === engine)?.label || "Browser or system speech";
}

export function formatTtsSetupSummary(preferences: TtsPreferences) {
  return `${selectedTtsEngineLabel(preferences.engine)} | ${preferences.language} | ${preferences.rate.toFixed(1)}x`;
}

export function selectedTtsEngineStatus(preferences: TtsPreferences, report: TtsInspectionReport | null | undefined) {
  return report?.engines.find((engine) => engine.id === preferences.engine) || null;
}

export function formatTtsRuntimeSummary(preferences: TtsPreferences, report: TtsInspectionReport | null | undefined) {
  if (!report) return "TTS runtime has not been checked.";
  if (preferences.engine === "browser-speech") return "Browser speech will be checked by the web runtime before playback.";
  return selectedTtsEngineStatus(preferences, report)?.detail || "Selected native TTS engine has no runtime status.";
}

export function buildTtsModelDownloadPlan(
  preferences: TtsPreferences,
  defaultStoragePath: string,
): TtsModelDownloadPlan | null {
  if (preferences.engine !== "supertonic-cli") return null;
  const storagePath = preferences.supertonicModelStoragePath.trim() || defaultStoragePath;
  const command = `${preferences.supertonicCommand.trim() || "supertonic"} download`;
  return {
    engine: "supertonic-cli",
    model: supertonicModelName,
    approximateSize: supertonicModelApproximateSize,
    storagePath,
    source: supertonicModelSource,
    command,
    acknowledged: preferences.supertonicModelDownloadAcknowledged,
  };
}

export function ttsReadIsDisabled(isBusy: boolean, plan: TtsModelDownloadPlan | null | undefined) {
  return isBusy || Boolean(plan && !plan.acknowledged);
}

export function ttsModelDownloadClipboardText(plan: TtsModelDownloadPlan) {
  return [
    `Model: ${plan.model}`,
    `Approximate size: ${plan.approximateSize}`,
    `Storage location: ${plan.storagePath}`,
    `Download command: ${plan.command}`,
  ].join("\n");
}

export function ttsModelAcknowledgementMessage(plan: TtsModelDownloadPlan) {
  return `Review and acknowledge the ${plan.model} download (${plan.approximateSize}) to ${plan.storagePath} before using Supertonic.`;
}

export function nativeTtsVoiceForPreferences(preferences: TtsPreferences) {
  if (preferences.engine === "supertonic-cli") return preferences.supertonicVoice || preferences.voice;
  return preferences.voice;
}

export function nativeTtsLanguageForPreferences(preferences: TtsPreferences) {
  if (preferences.engine === "supertonic-cli") return preferences.supertonicLanguage || preferences.language;
  return preferences.language;
}

export function nativeTtsRateForPreferences(preferences: TtsPreferences) {
  return Math.round(preferences.rate * 175);
}
