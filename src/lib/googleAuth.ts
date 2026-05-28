export const GOOGLE_DOCS_OAUTH_SCOPES = [
  "https://www.googleapis.com/auth/drive.file",
  "https://www.googleapis.com/auth/documents",
] as const;

export interface GoogleIntegrationPreferences {
  clientId: string;
  scopes: string[];
  accountHint: string;
  lastAuthorizedAt: string;
  tokenExpiresAt: string;
}

export interface GoogleOAuthStartResponse {
  authorization_url: string;
  redirect_uri: string;
  state: string;
  code_verifier: string;
  scopes: string[];
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
  error?: string;
  error_description?: string;
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
