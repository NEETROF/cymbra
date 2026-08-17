import { computed, ref, type ComputedRef, type Ref } from "vue";
import { type Async, failure, idle, loading, messageOf, success } from "./async";
import type { WebAuthClient } from "./client";

// A minimal browser session over the web-auth client, for front-ends without a
// store of their own (the public site's islands). The access token lives in memory
// only; persistence across pages is the HttpOnly refresh cookie, re-minted by
// `boot()` on mount (spec `site-web-signin`).

/** `signedOut` = no session (show the sign-in form); `signedIn` = a bearer is held. */
export type WebSessionState =
  { readonly kind: "signedOut" } | { readonly kind: "signedIn"; readonly accessToken: string };

export interface WebSession {
  /** Current session; starts `signedOut` until `boot()` resolves. */
  readonly state: Ref<WebSessionState>;
  /** The in-memory bearer, `null` when signed out. */
  readonly accessToken: ComputedRef<string | null>;
  /** The last sign-in / boot attempt (for the form's busy + error display). */
  readonly attempt: Ref<Async<void>>;
  /** Re-mint from the refresh cookie (silent: a missing cookie is not an error). */
  boot(): Promise<void>;
  signInLocal(email: string, password: string): Promise<Async<void>>;
  signInOidc(idToken: string): Promise<Async<void>>;
  /** Refresh the bearer (e.g. after a 401); signs out when the cookie is gone. */
  refresh(): Promise<string | null>;
  signOut(): Promise<void>;
}

/**
 * Build a session for `audience` over `client`. `mapError` localizes failures for
 * `attempt` (default: the raw message).
 */
export function createWebSession(
  client: WebAuthClient,
  audience: string,
  mapError: (e: unknown) => string = messageOf,
): WebSession {
  const state = ref<WebSessionState>({ kind: "signedOut" });
  const attempt = ref<Async<void>>(idle);
  const accessToken = computed(() => (state.value.kind === "signedIn" ? state.value.accessToken : null));

  async function attemptWith(fn: () => Promise<{ accessToken: string }>): Promise<Async<void>> {
    attempt.value = loading;
    try {
      const { accessToken } = await fn();
      state.value = { kind: "signedIn", accessToken };
      attempt.value = success(undefined);
    } catch (e) {
      attempt.value = failure(mapError(e));
    }
    return attempt.value;
  }

  async function refresh(): Promise<string | null> {
    try {
      const { accessToken } = await client.refresh();
      state.value = { kind: "signedIn", accessToken };
      return accessToken;
    } catch {
      state.value = { kind: "signedOut" };
      return null;
    }
  }

  return {
    state,
    accessToken,
    attempt,
    boot: async () => {
      await refresh();
    },
    signInLocal: (email, password) => attemptWith(() => client.signInLocal(email, password, audience)),
    signInOidc: (idToken) => attemptWith(() => client.signInOidc(idToken, audience)),
    refresh,
    signOut: async () => {
      await client.logout();
      state.value = { kind: "signedOut" };
      attempt.value = idle;
    },
  };
}
