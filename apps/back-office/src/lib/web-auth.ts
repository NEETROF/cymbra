// The browser web-auth surface (change: add-web-auth-cookies). Unlike the gRPC-web
// data plane, sign-in/refresh/logout go over plain HTTP so the backend can set,
// rotate, and clear the refresh token in an `HttpOnly` cookie the page can't read.
// The access token comes back in the JSON body for in-memory use only.
//
// This is the injectable seam (mirrors lib/api.ts): stores depend on `webAuth()`,
// tests swap a fake via `setWebAuthClientForTest`.

export interface WebAuthTokens {
  /** Short-lived access token for the gRPC `Authorization: Bearer` header. */
  accessToken: string;
}

export interface WebAuthClient {
  signInLocal(email: string, password: string, audience: string): Promise<WebAuthTokens>;
  signInOidc(idToken: string, audience: string): Promise<WebAuthTokens>;
  /** Mint a new access token from the refresh cookie (credentialed; reads the cookie). */
  refresh(): Promise<WebAuthTokens>;
  /** Revoke the session and clear the cookie (best-effort). */
  logout(): Promise<void>;
}

/** A failed web-auth call, carrying the HTTP status so `humanError` can localize it. */
export class WebAuthError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "WebAuthError";
  }
}

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

// Credentialed POST with the JSON content-type + custom header that force a CORS
// preflight (CSRF defence: a cross-site `<form>` can satisfy neither).
async function post(path: string, body?: unknown): Promise<Response> {
  return fetch(`${webAuthBaseUrl()}${path}`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json", "X-Cymbra-Web": "1" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
}

async function toError(resp: Response): Promise<WebAuthError> {
  let message = resp.statusText;
  try {
    const data: unknown = await resp.json();
    if (data && typeof (data as { error?: unknown }).error === "string") {
      message = (data as { error: string }).error;
    }
  } catch {
    // Non-JSON error body — keep the status text.
  }
  return new WebAuthError(resp.status, message);
}

async function tokensOrThrow(resp: Response): Promise<WebAuthTokens> {
  if (!resp.ok) throw await toError(resp);
  const data = (await resp.json()) as { accessToken?: string };
  return { accessToken: data.accessToken ?? "" };
}

export function createWebAuthClient(): WebAuthClient {
  // Single-flight the refresh: concurrent callers (the boot re-mint and a 401-triggered
  // retry, several 401s at once, or a double-mounted boot) MUST share ONE request.
  // Two refreshes racing on the same cookie each rotate the refresh token, and the
  // second one replays an already-rotated token — which the server's reuse detection
  // treats as theft and revokes the whole session, bouncing the user to sign-in.
  let inflightRefresh: Promise<WebAuthTokens> | null = null;
  return {
    signInLocal: (email, password, audience) =>
      post("/web/auth/signin", { kind: "local", email, password, audience }).then(tokensOrThrow),
    signInOidc: (idToken, audience) =>
      post("/web/auth/signin", { kind: "oidc", idToken, audience }).then(tokensOrThrow),
    refresh: () => {
      inflightRefresh ??= post("/web/auth/refresh")
        .then(tokensOrThrow)
        .finally(() => {
          inflightRefresh = null;
        });
      return inflightRefresh;
    },
    logout: async () => {
      // Ignore the outcome: the server clears the cookie and the store clears memory
      // regardless, so a network blip on logout still ends the local session.
      await post("/web/auth/logout").catch(() => undefined);
    },
  };
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
