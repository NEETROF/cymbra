// Provider (Google / Apple) authorization requests for `identity.launchWebAuthFlow`
// (add-lingua-account-parity, design D3/D5). Pure: URL building, redirect parsing and
// cancel detection are unit-tested; the background only supplies the browser API.

export type Provider = "google" | "apple";

export interface AuthorizeRequest {
  url: string;
  /** The `state` sent, checked against the redirect to reject a forged response. */
  state: string;
}

export interface RequestParams {
  clientId: string;
  redirectUri: string;
  state: string;
  nonce: string;
}

/** Google: OpenID implicit flow, the id_token comes back in the fragment. */
export function googleAuthorizeRequest(p: RequestParams): AuthorizeRequest {
  const url = new URL("https://accounts.google.com/o/oauth2/v2/auth");
  url.searchParams.set("client_id", p.clientId);
  url.searchParams.set("response_type", "id_token");
  url.searchParams.set("redirect_uri", p.redirectUri);
  url.searchParams.set("scope", "openid email");
  // Always show the account chooser: the browser's signed-in Google account is often not
  // the one the reader uses for Cymbra (Music re-prompts the same way).
  url.searchParams.set("prompt", "select_account");
  url.searchParams.set("state", p.state);
  url.searchParams.set("nonce", p.nonce);
  return { url: url.toString(), state: p.state };
}

/**
 * Apple: NO scope, so Apple accepts `response_mode=fragment` (it demands form_post as soon
 * as name/email is requested, and launchWebAuthFlow cannot read a POST body). Cymbra ID
 * resolves the account by (provider, subject) and never reads the email claim. Apple does
 * not accept `id_token` alone, so the (never exchanged) code comes along.
 */
export function appleAuthorizeRequest(p: RequestParams): AuthorizeRequest {
  const url = new URL("https://appleid.apple.com/auth/authorize");
  url.searchParams.set("client_id", p.clientId);
  url.searchParams.set("redirect_uri", p.redirectUri);
  url.searchParams.set("response_type", "code id_token");
  url.searchParams.set("response_mode", "fragment");
  url.searchParams.set("state", p.state);
  url.searchParams.set("nonce", p.nonce);
  return { url: url.toString(), state: p.state };
}

/** Provider error values that mean the reader backed out, not a failure. */
const CANCEL_ERRORS = new Set(["user_cancelled_authorize", "access_denied"]);

/**
 * Read the id_token from the final redirect URL. Returns null when the provider reports a
 * user cancel; throws when the provider reports another error, the `state` does not match,
 * or no id_token is present. The authorization `code` is ignored on purpose.
 */
export function idTokenFromRedirect(redirect: string, expectedState: string): string | null {
  const url = new URL(redirect);
  const params = new URLSearchParams(url.hash.slice(1));
  // Some providers report errors in the query rather than the fragment.
  const error = params.get("error") ?? url.searchParams.get("error");
  if (error) {
    if (CANCEL_ERRORS.has(error)) return null;
    throw new Error(`provider error: ${error}`);
  }
  if (params.get("state") !== expectedState) throw new Error("provider state mismatch");
  const idToken = params.get("id_token");
  if (!idToken) throw new Error("provider returned no id_token");
  return idToken;
}

/** Whether a launchWebAuthFlow rejection is the reader closing the window (Chrome/Firefox). */
export function isUserCancel(e: unknown): boolean {
  const message = e instanceof Error ? e.message : String(e);
  return /did not approve access|user cancel|cancell?ed|denied access/i.test(message);
}

export type LaunchWebAuthFlow = (url: string) => Promise<string | undefined>;

/** Run a request through the browser flow: the id_token, or null when the reader cancelled. */
export async function runAuthFlow(launch: LaunchWebAuthFlow, req: AuthorizeRequest): Promise<string | null> {
  let redirect: string | undefined;
  try {
    redirect = await launch(req.url);
  } catch (e) {
    if (isUserCancel(e)) return null;
    throw e;
  }
  if (!redirect) return null;
  return idTokenFromRedirect(redirect, req.state);
}

export interface Providers {
  google: boolean;
  apple: boolean;
}

/**
 * A provider is offered only when its client id was configured at build time AND the
 * browser has `identity.launchWebAuthFlow` (absent on Firefox for Android and Safari, whose
 * build also drops the `identity` permission).
 * Feature detection, not the build target: Firefox desktop and Android share a build.
 */
export function availableProviders(
  clientIds: { google: string; apple: string },
  identity: { launchWebAuthFlow?: unknown } | undefined,
): Providers {
  const hasFlow = typeof identity?.launchWebAuthFlow === "function";
  return { google: hasFlow && clientIds.google.length > 0, apple: hasFlow && clientIds.apple.length > 0 };
}
