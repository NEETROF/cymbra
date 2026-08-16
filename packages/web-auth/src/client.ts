// The browser web-auth surface (change: add-web-auth-cookies). Unlike the gRPC-web
// data plane, sign-in/refresh/logout go over plain HTTP so the backend can set,
// rotate, and clear the refresh token in an `HttpOnly` cookie the page can't read.
// The access token comes back in the JSON body for **in-memory use only** — this
// package never writes a token to `localStorage` / `sessionStorage`.

export interface WebAuthTokens {
  /** Short-lived access token for the `Authorization: Bearer` header. */
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

/** A failed web call, carrying the HTTP status so the caller can localize it. */
export class WebAuthError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "WebAuthError";
  }
}

/** The custom header that forces a CORS preflight (CSRF defence, see the backend). */
export const CSRF_HEADER = "X-Cymbra-Web";

// Credentialed POST with the JSON content-type + custom header that force a CORS
// preflight (CSRF defence: a cross-site `<form>` can satisfy neither).
async function post(baseUrl: string, path: string, body?: unknown): Promise<Response> {
  return fetch(`${baseUrl}${path}`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json", [CSRF_HEADER]: "1" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
}

/** Turn a non-OK response into a `WebAuthError` (message from `{ error }` when JSON). */
export async function toError(resp: Response): Promise<WebAuthError> {
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

/**
 * The real client against `baseUrl` (the backend HTTP origin, e.g.
 * `https://api.cymbra.app`).
 */
export function createWebAuthClient(baseUrl: string): WebAuthClient {
  // Single-flight the refresh: concurrent callers (the boot re-mint and a 401-triggered
  // retry, several 401s at once, or a double-mounted boot) MUST share ONE request.
  // Two refreshes racing on the same cookie each rotate the refresh token, and the
  // second one replays an already-rotated token — which the server's reuse detection
  // treats as theft and revokes the whole session, bouncing the user to sign-in.
  let inflightRefresh: Promise<WebAuthTokens> | null = null;
  return {
    signInLocal: (email, password, audience) =>
      post(baseUrl, "/web/auth/signin", { kind: "local", email, password, audience }).then(tokensOrThrow),
    signInOidc: (idToken, audience) =>
      post(baseUrl, "/web/auth/signin", { kind: "oidc", idToken, audience }).then(tokensOrThrow),
    refresh: () => {
      inflightRefresh ??= post(baseUrl, "/web/auth/refresh")
        .then(tokensOrThrow)
        .finally(() => {
          inflightRefresh = null;
        });
      return inflightRefresh;
    },
    logout: async () => {
      // Ignore the outcome: the server clears the cookie and the caller clears memory
      // regardless, so a network blip on logout still ends the local session.
      await post(baseUrl, "/web/auth/logout").catch(() => undefined);
    },
  };
}

export interface FetchJsonInit {
  method?: "GET" | "POST";
  /** Bearer access token (in memory), sent as `Authorization`. */
  accessToken?: string;
  body?: unknown;
}

/**
 * Bearer JSON call to a backend route (e.g. `/web/plans/me`): `Authorization: Bearer`
 * from the in-memory token, JSON body, and a `WebAuthError` (status + the route's
 * `{ error }` message) on a non-OK answer. No cookies are sent — the bearer is the
 * credential.
 */
export async function fetchJson<T>(url: string, init: FetchJsonInit = {}): Promise<T> {
  const headers: Record<string, string> = {};
  if (init.accessToken) headers.Authorization = `Bearer ${init.accessToken}`;
  if (init.body !== undefined) headers["Content-Type"] = "application/json";
  const resp = await fetch(url, {
    method: init.method ?? (init.body === undefined ? "GET" : "POST"),
    headers,
    body: init.body === undefined ? undefined : JSON.stringify(init.body),
  });
  if (!resp.ok) throw await toError(resp);
  return (await resp.json()) as T;
}
