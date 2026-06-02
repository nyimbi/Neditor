import {
  normalizeBibliographyDefaults,
  normalizeBrandProfileDefaults,
  normalizeExportDefaults,
  normalizeExportProfiles,
  type BibliographyDefaults,
  type BrandProfileDefaults,
  type ExportDefaults,
  type ExportProfile,
  type ExportTarget,
} from "./workspacePersistence.js";

export interface ExportProfileSnapshot {
  exportTarget: ExportTarget;
  exportDefaults: ExportDefaults;
  bibliographyDefaults: BibliographyDefaults;
  brandProfileDefaults: BrandProfileDefaults;
}

export interface SaveExportProfileResult {
  profile: ExportProfile;
  profiles: ExportProfile[];
  activeExportProfileId: string;
  statusMessage: string;
}

export interface ApplyExportProfileResult extends ExportProfileSnapshot {
  activeExportProfileId: string;
  statusMessage: string;
}

export interface DeleteExportProfileResult {
  profiles: ExportProfile[];
  activeExportProfileId: string;
  statusMessage: string;
}

export function createExportProfileId() {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `export-profile-${Date.now().toString(36)}`;
}

export function saveExportProfileState(
  profiles: ExportProfile[],
  activeExportProfileId: string | null,
  name: string,
  snapshot: ExportProfileSnapshot,
  createId: () => string = createExportProfileId,
): SaveExportProfileResult {
  const profileName = name.trim() || "Export profile";
  const normalizedProfiles = normalizeExportProfiles(profiles);
  const existing = activeExportProfileId
    ? normalizedProfiles.find((profile) => profile.id === activeExportProfileId)
    : null;
  // Detect stale ID: non-empty but not found in current profiles
  const staleId = activeExportProfileId && !existing ? activeExportProfileId : null;
  const profile: ExportProfile = {
    id: existing?.id || createId(),
    name: profileName,
    exportTarget: snapshot.exportTarget,
    exportDefaults: normalizeExportDefaults(snapshot.exportDefaults),
    bibliographyDefaults: normalizeBibliographyDefaults(snapshot.bibliographyDefaults),
    brandProfileDefaults: normalizeBrandProfileDefaults(snapshot.brandProfileDefaults),
  };
  const nextProfiles = existing
    ? normalizedProfiles.map((item) => (item.id === existing.id ? profile : item))
    : normalizeExportProfiles([...normalizedProfiles, profile]);
  const statusMessage = staleId
    ? `Saved export profile "${profile.name}" (previous profile ID "${staleId}" not found; created new profile)`
    : `Saved export profile "${profile.name}"`;
  return {
    profile,
    profiles: nextProfiles,
    activeExportProfileId: profile.id,
    statusMessage,
  };
}

export function applyExportProfileState(profiles: ExportProfile[], id: string): ApplyExportProfileResult | null {
  const profile = normalizeExportProfiles(profiles).find((item) => item.id === id);
  if (!profile) return null;
  return {
    exportTarget: profile.exportTarget,
    exportDefaults: normalizeExportDefaults(profile.exportDefaults),
    bibliographyDefaults: normalizeBibliographyDefaults(profile.bibliographyDefaults),
    brandProfileDefaults: normalizeBrandProfileDefaults(profile.brandProfileDefaults),
    activeExportProfileId: profile.id,
    statusMessage: `Applied export profile "${profile.name}"`,
  };
}

export function deleteExportProfileState(
  profiles: ExportProfile[],
  activeExportProfileId: string,
  id: string,
): DeleteExportProfileResult {
  const normalizedProfiles = normalizeExportProfiles(profiles);
  const profile = normalizedProfiles.find((item) => item.id === id);
  return {
    profiles: normalizedProfiles.filter((item) => item.id !== id),
    activeExportProfileId: activeExportProfileId === id ? "" : activeExportProfileId,
    statusMessage: profile ? `Deleted export profile "${profile.name}"` : "",
  };
}
