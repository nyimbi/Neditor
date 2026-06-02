export const GOOGLE_DOCS_OAUTH_SCOPES = [
  "https://www.googleapis.com/auth/drive.file",
  "https://www.googleapis.com/auth/documents",
] as const;

export const GOOGLE_DOCX_MIME_TYPE = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
export const GOOGLE_DOCS_MIME_TYPE = "application/vnd.google-apps.document";
export const GOOGLE_DRIVE_UPLOAD_URL =
  "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&fields=id,name,webViewLink,mimeType";

export interface GoogleIntegrationPreferences {
  clientId: string;
  scopes: string[];
  accountHint: string;
  requestOfflineAccess: boolean;
  lastAuthorizedAt: string;
  tokenExpiresAt: string;
}

export interface GoogleOAuthStartResponse {
  authorization_url: string;
  redirect_uri: string;
  state: string;
  code_verifier: string;
  scopes: string[];
  offline_access: boolean;
  expires_in_seconds: number;
}

export interface GoogleOAuthCallbackResponse {
  state: string;
  code?: string | null;
  error?: string | null;
  received: boolean;
}

export interface GoogleOAuthTokenResponse {
  access_token?: string;
  expires_in?: number;
  scope?: string;
  token_type?: string;
  refresh_token?: string;
  error?: string;
  error_description?: string;
}

export interface GoogleDocsLiveImportPreparation {
  file_name: string;
  document_title: string;
  mime_type: string;
  docx_bytes: number[];
  docx_hash: string;
  diagnostics: Array<{ severity: string; message: string }>;
  progress_steps: Array<{ id: string; label: string; status: string }>;
}

export interface GoogleDriveImportResponse {
  id?: string;
  name?: string;
  webViewLink?: string;
  mimeType?: string;
  error?: { message?: string };
}

function normalizedString(value: unknown, limit: number) {
  return typeof value === "string" ? value.trim().slice(0, limit) : "";
}

export function normalizeGoogleOAuthScopes(value: unknown): string[] {
  const raw = Array.isArray(value) ? value : typeof value === "string" ? value.split(/[\s,]+/) : [];
  const scopes = raw
    .filter((scope): scope is string => typeof scope === "string")
    .map((scope) => scope.trim())
    .filter((scope) => scope.startsWith("https://www.googleapis.com/auth/"))
    .slice(0, 8);
  return Array.from(new Set(scopes.length ? scopes : [...GOOGLE_DOCS_OAUTH_SCOPES]));
}

export function normalizeGoogleIntegrationPreferences(value: unknown): GoogleIntegrationPreferences {
  const record = typeof value === "object" && value !== null && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
  return {
    clientId: normalizedString(record.clientId, 240),
    scopes: normalizeGoogleOAuthScopes(record.scopes),
    accountHint: normalizedString(record.accountHint, 160),
    requestOfflineAccess: record.requestOfflineAccess !== false,
    lastAuthorizedAt: normalizedString(record.lastAuthorizedAt, 40),
    tokenExpiresAt: normalizedString(record.tokenExpiresAt, 40),
  };
}

export function googleOAuthScopesText(scopes: string[]) {
  return normalizeGoogleOAuthScopes(scopes).join("\n");
}

export function googleOAuthTokenRequestBody(session: GoogleOAuthStartResponse, clientId: string, code: string) {
  const body = new URLSearchParams();
  body.set("client_id", clientId.trim());
  body.set("code", code);
  body.set("code_verifier", session.code_verifier);
  body.set("grant_type", "authorization_code");
  body.set("redirect_uri", session.redirect_uri);
  return body;
}

export function googleOAuthRefreshTokenRequestBody(clientId: string, refreshToken: string) {
  const body = new URLSearchParams();
  body.set("client_id", clientId.trim());
  body.set("grant_type", "refresh_token");
  body.set("refresh_token", refreshToken);
  return body;
}

export function googleOAuthTokenNeedsRefresh(expiresAt: string, now = Date.now(), skewMs = 120_000) {
  const expiresMs = Date.parse(expiresAt);
  if (!Number.isFinite(expiresMs)) return true;
  return expiresMs <= now + skewMs;
}

export function googleApiAuthErrorNeedsRefresh(status: number, bodyText = "") {
  if (status === 401) return true;
  if (status !== 403) return false;
  return /\b(?:access token|auth|credential|expired|invalid token|login required|token)\b/i.test(bodyText);
}

export function googleDriveExportTextUrl(fileId: string) {
  return `https://www.googleapis.com/drive/v3/files/${encodeURIComponent(fileId)}/export?mimeType=text/plain`;
}

export function googleDocsImportMetadata(fileName: string) {
  return {
    name: fileName.replace(/\.docx$/i, ""),
    mimeType: GOOGLE_DOCS_MIME_TYPE,
  };
}

export function googleDocsMultipartUploadBody(fileName: string, docxBytes: Uint8Array, boundary: string) {
  const metadata = JSON.stringify(googleDocsImportMetadata(fileName));
  return new Blob(
    [
      `--${boundary}\r\n`,
      "Content-Type: application/json; charset=UTF-8\r\n\r\n",
      metadata,
      "\r\n",
      `--${boundary}\r\n`,
      `Content-Type: ${GOOGLE_DOCX_MIME_TYPE}\r\n\r\n`,
      docxBytes,
      "\r\n",
      `--${boundary}--\r\n`,
    ],
    { type: `multipart/related; boundary=${boundary}` },
  );
}
