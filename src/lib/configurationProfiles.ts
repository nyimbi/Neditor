import { normalizeBusinessProfile, type BusinessProfile } from "./businessDocuments.js";
import {
  normalizeAiProviderDefaults,
  normalizeTtsPreferences,
  type AiProviderDefaults,
  type TtsPreferences,
} from "./workspacePersistence.js";

export interface ConfigurationProfileStateResult<T> {
  value: T;
  changed: boolean;
}

function changedByJson<T>(current: T, next: T) {
  return JSON.stringify(current) !== JSON.stringify(next);
}

export function saveBusinessProfileState(
  current: BusinessProfile,
  profile: Partial<BusinessProfile>,
): ConfigurationProfileStateResult<BusinessProfile> {
  const value = normalizeBusinessProfile(profile);
  return { value, changed: changedByJson(current, value) };
}

export function saveAiProviderDefaultsState(
  current: AiProviderDefaults,
  defaults: Partial<AiProviderDefaults>,
): ConfigurationProfileStateResult<AiProviderDefaults> {
  const value = normalizeAiProviderDefaults(defaults);
  return { value, changed: changedByJson(current, value) };
}

export function saveTtsPreferencesState(
  current: TtsPreferences,
  defaults: Partial<TtsPreferences>,
): ConfigurationProfileStateResult<TtsPreferences> {
  const value = normalizeTtsPreferences(defaults);
  return { value, changed: changedByJson(current, value) };
}
