// The browser web-auth surface (change: add-web-auth-cookies) — the implementation
// lives in the shared `@cymbra/web-auth` package (change: add-site-account-pages),
// used by this console and the public site alike. This module is the back office's
// injectable seam over it (mirrors lib/api.ts): stores depend on `webAuth()`, tests
// swap a fake via `setWebAuthClientForTest`.

import { createWebAuthClient as createClient, type WebAuthClient } from "@cymbra/web-auth";

export { WebAuthError, type WebAuthClient, type WebAuthTokens } from "@cymbra/web-auth";

// Default to the backend's local HTTP addr so `yarn dev` works without a `.env`. The
// web-auth surface lives on the HTTP server (JWKS/health), not the gRPC-web port.
const DEFAULT_WEB_AUTH_URL = "http://localhost:8081";

export function webAuthBaseUrl(): string {
  const url = import.meta.env.VITE_WEB_AUTH_URL;
  if (!url) {
    console.warn(
      `VITE_WEB_AUTH_URL is not set — defaulting to ${DEFAULT_WEB_AUTH_URL}. ` +
        "Set it in apps/back-office/.env (see .env.example) for other environments.",
    );
    return DEFAULT_WEB_AUTH_URL;
  }
  return url;
}

/** The real client against this deployment's web-auth base URL. */
export function createWebAuthClient(): WebAuthClient {
  return createClient(webAuthBaseUrl());
}

// Lazily-initialised singleton (mirrors lib/api.ts): `initWebAuth` wires the real
// client at startup; tests/e2e inject a fake via `setWebAuthClientForTest`.
let client: WebAuthClient | null = null;

export function initWebAuth(): void {
  client = createWebAuthClient();
}

export function setWebAuthClientForTest(fake: WebAuthClient): void {
  client = fake;
}

export function webAuth(): WebAuthClient {
  if (!client) throw new Error("webAuth() used before initWebAuth()/setWebAuthClientForTest()");
  return client;
}
