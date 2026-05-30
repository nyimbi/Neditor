export type DatabaseProfileDriver = "sqlite" | "duckdb" | "postgres" | "mysql" | "generic";
export type DatabaseProfileConnectionMode = "file" | "environment" | "manual";

export interface DatabaseProfile {
  id: string;
  name: string;
  driver: DatabaseProfileDriver;
  connectionMode: DatabaseProfileConnectionMode;
  databasePath: string;
  dsnEnv: string;
  host: string;
  port: string;
  databaseName: string;
  username: string;
  secretEnv: string;
  readonly: boolean;
  tags: string[];
  notes: string;
}

export interface DatabaseProfileStateResult {
  profiles: DatabaseProfile[];
  activeId: string;
  changed: boolean;
  profile?: DatabaseProfile;
}

export const databaseProfileDrivers: Array<{ value: DatabaseProfileDriver; label: string }> = [
  { value: "sqlite", label: "SQLite" },
  { value: "duckdb", label: "DuckDB" },
  { value: "postgres", label: "PostgreSQL" },
  { value: "mysql", label: "MySQL" },
  { value: "generic", label: "Generic SQL" },
];

export const databaseProfileConnectionModes: Array<{ value: DatabaseProfileConnectionMode; label: string }> = [
  { value: "file", label: "Local file" },
  { value: "environment", label: "Environment variable" },
  { value: "manual", label: "Manual/session only" },
];

export function blankDatabaseProfile(): DatabaseProfile {
  return {
    id: "",
    name: "Local analytics database",
    driver: "sqlite",
    connectionMode: "file",
    databasePath: "data/example.sqlite",
    dsnEnv: "NEDITOR_DATABASE_URL",
    host: "",
    port: "",
    databaseName: "",
    username: "",
    secretEnv: "",
    readonly: true,
    tags: ["reporting", "readonly"],
    notes: "Use read-only queries and keep credentials in environment variables, not documents.",
  };
}

export function normalizeDatabaseProfiles(value: unknown): DatabaseProfile[] {
  if (!Array.isArray(value)) return [];
  return dedupeById(value.map((item, index) => normalizeDatabaseProfile(item, index)).filter(Boolean) as DatabaseProfile[]).slice(0, 40);
}

export function normalizeDatabaseProfile(value: unknown, index = 0): DatabaseProfile | null {
  if (!value || typeof value !== "object") return null;
  const record = value as Partial<DatabaseProfile>;
  const name = stringValue(record.name) || "Database profile";
  const driver = databaseProfileDriverValue(record.driver);
  const connectionMode = databaseProfileConnectionModeValue(record.connectionMode);
  const databasePath = stripUrlCredentials(stringValue(record.databasePath));
  const dsnEnv = envNameValue(record.dsnEnv) || "NEDITOR_DATABASE_URL";
  const secretEnv = envNameValue(record.secretEnv);
  const host = stripUrlCredentials(stringValue(record.host));
  return {
    id: slug(stringValue(record.id) || name || `database-${index + 1}`),
    name,
    driver,
    connectionMode,
    databasePath,
    dsnEnv,
    host,
    port: stringValue(record.port).replace(/[^\d]/g, "").slice(0, 5),
    databaseName: stringValue(record.databaseName),
    username: stringValue(record.username),
    secretEnv,
    readonly: record.readonly !== false,
    tags: cleanList(record.tags).slice(0, 12),
    notes: stringValue(record.notes) || "Keep credentials outside documents.",
  };
}

export function saveDatabaseProfileState(
  profiles: DatabaseProfile[],
  activeId: string,
  profile: Partial<DatabaseProfile>,
): DatabaseProfileStateResult {
  const normalized = normalizeDatabaseProfile(profile);
  if (!normalized) return { profiles, activeId, changed: false };
  const next = [...profiles.filter((item) => item.id !== normalized.id), normalized].sort((left, right) => left.name.localeCompare(right.name));
  return {
    profiles: next,
    activeId: normalized.id,
    changed: JSON.stringify(next) !== JSON.stringify(profiles) || activeId !== normalized.id,
    profile: normalized,
  };
}

export function deleteDatabaseProfileState(profiles: DatabaseProfile[], activeId: string, id: string): DatabaseProfileStateResult {
  const next = profiles.filter((profile) => profile.id !== id);
  const nextActiveId = activeId === id ? next[0]?.id || "" : activeId;
  return { profiles: next, activeId: nextActiveId, changed: next.length !== profiles.length };
}

export function databaseProfileSummary(profile: DatabaseProfile) {
  const connection =
    profile.connectionMode === "file"
      ? profile.databasePath || "missing file path"
      : profile.connectionMode === "environment"
        ? profile.dsnEnv || "missing DSN env var"
        : "manual/session connection";
  const auth = profile.secretEnv ? `secret env ${profile.secretEnv}` : "no secret stored";
  return `${profile.driver.toUpperCase()} | ${connection} | ${profile.readonly ? "read-only" : "write-capable"} | ${auth}`;
}

export function databaseProfileWarnings(profile: DatabaseProfile): string[] {
  const warnings: string[] = [];
  if (profile.connectionMode === "file" && !profile.databasePath.trim()) warnings.push("Add a local database file path.");
  if (profile.connectionMode === "environment" && !profile.dsnEnv.trim()) warnings.push("Set a DSN environment variable name.");
  if (profile.driver !== "sqlite" && profile.connectionMode === "file") warnings.push("File mode is only directly executable by the current SQLite SQL transform.");
  if (!profile.readonly) warnings.push("Write-capable profiles require extra review before inserting queries into client-facing documents.");
  if (looksLikeSecret(profile.databasePath) || looksLikeSecret(profile.host) || looksLikeSecret(profile.notes)) {
    warnings.push("A field looks like it may contain a secret; move credentials to an environment variable.");
  }
  return warnings;
}

export function databaseProfileSqlTransform(profile: DatabaseProfile, query = defaultDatabaseProfileQuery(profile)) {
  const safeProfile = normalizeDatabaseProfile(profile) || blankDatabaseProfile();
  const options = [`profile="${safeProfile.id}"`, `driver="${safeProfile.driver}"`];
  if (safeProfile.connectionMode === "file" && safeProfile.driver === "sqlite") options.push(`database="${escapeFenceOption(safeProfile.databasePath)}"`);
  if (safeProfile.connectionMode === "environment") options.push(`databaseEnv="${escapeFenceOption(safeProfile.dsnEnv)}"`);
  if (safeProfile.readonly) options.push('readonly="true"');
  const comments = [
    `-- Profile: ${safeProfile.name}`,
    `-- ${databaseProfileSummary(safeProfile)}`,
    "-- Keep credentials in environment variables or session configuration, never in the document.",
  ];
  return [`\`\`\`sql ${options.join(" ")}`, ...comments, query.trim(), "```", ""].join("\n");
}

export function databaseProfilePreviewRows(profile: DatabaseProfile): Array<{ label: string; value: string }> {
  return [
    { label: "Driver", value: profile.driver.toUpperCase() },
    { label: "Connection", value: profile.connectionMode },
    { label: "Database", value: profile.connectionMode === "file" ? profile.databasePath || "not set" : profile.dsnEnv || "not set" },
    { label: "Auth", value: profile.secretEnv ? `Environment: ${profile.secretEnv}` : "No stored secret" },
    { label: "Mode", value: profile.readonly ? "Read-only" : "Write-capable" },
  ];
}

function defaultDatabaseProfileQuery(profile: DatabaseProfile) {
  if (profile.driver === "sqlite" || profile.driver === "duckdb") {
    return "SELECT\n  name,\n  amount\nFROM results\nORDER BY amount DESC\nLIMIT 25;";
  }
  return "SELECT\n  name,\n  amount\nFROM reporting.results\nORDER BY amount DESC\nLIMIT 25;";
}

function databaseProfileDriverValue(value: unknown): DatabaseProfileDriver {
  const normalized = stringValue(value).toLowerCase();
  return ["sqlite", "duckdb", "postgres", "mysql", "generic"].includes(normalized) ? (normalized as DatabaseProfileDriver) : "sqlite";
}

function databaseProfileConnectionModeValue(value: unknown): DatabaseProfileConnectionMode {
  const normalized = stringValue(value).toLowerCase();
  return ["file", "environment", "manual"].includes(normalized) ? (normalized as DatabaseProfileConnectionMode) : "file";
}

function stringValue(value: unknown) {
  return typeof value === "string" ? value.trim() : "";
}

function cleanList(value: unknown) {
  if (Array.isArray(value)) return value.map((item) => String(item).trim()).filter(Boolean);
  if (typeof value === "string") return value.split(/\r?\n|,/).map((item) => item.trim()).filter(Boolean);
  return [];
}

function dedupeById(items: DatabaseProfile[]) {
  const seen = new Set<string>();
  const result: DatabaseProfile[] = [];
  for (const item of items) {
    if (!item.id || seen.has(item.id)) continue;
    seen.add(item.id);
    result.push(item);
  }
  return result;
}

function slug(value: string) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "") || "database-profile";
}

function envNameValue(value: unknown) {
  const normalized = stringValue(value).replace(/[^A-Za-z0-9_]/g, "_").replace(/^(\d)/, "_$1");
  return normalized.toUpperCase();
}

function stripUrlCredentials(value: string) {
  return value.replace(/([a-z][a-z0-9+.-]*:\/\/)([^/@\s:]+):([^/@\s]+)@/gi, "$1");
}

function looksLikeSecret(value: string) {
  return /\b(password|passwd|secret|token|apikey|api_key)\s*[:=]/i.test(value) || /:\/\/[^/@\s:]+:[^/@\s]+@/.test(value);
}

function escapeFenceOption(value: string) {
  return value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}
