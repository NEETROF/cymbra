import type { Client } from "@connectrpc/connect";
import type { AuthService } from "@/gen/auth_pb";
import type { FlagService } from "@/gen/flags_pb";
import type { OnlineTranslationService } from "@/gen/online_translation_pb";
import { flagOn, toValues } from "../flags/store.ts";
import type { ReviewOnlineSource } from "../review/review-page.ts";
import type { Provider } from "../state/oidc.ts";
import { REFRESH_KEY, Session } from "../state/session.ts";
import type { AsyncStorageArea } from "../state/storage.ts";
import {
  answerOnlineTranslation,
  ONLINE_TRANSLATION_FLAG,
  ONLINE_UNAVAILABLE,
  onlineMessage,
  type OnlineTranslationResult,
} from "../translate/online.ts";
import type { AppSend } from "./bridge.ts";

// The Cymbra Lingua app's own session, for « Traduction en ligne » in its review and nothing else
// (add-lingua-review-translation-fallback D8). The extension's session is never relayed: its access
// token lives 15 minutes and its background does not run while the app is in front, and sharing its
// refresh token would make the first rotation a replay that revokes the family. So the app signs in
// on its own, with the Apple or Google sheet it already runs, on the first press of the button: the
// bridge's `auth.signIn` answers the provider's id_token, exchanged here through `SignInOidc` for the
// `lingua` audience by the extension's own Session (single-flight refresh, categorised errors). The
// access token stays in this page's memory; the refresh token is kept in the app's Keychain through
// the bridge (`auth.readRefresh`, `auth.writeRefresh`, `auth.clear`) — never in web storage, whose
// durability under a custom scheme is not guaranteed.

/** What the account needs of the transport: the clients, bound to this session's bearer. */
export interface AppAccountClients {
  auth: Client<typeof AuthService>;
  flags: Pick<Client<typeof FlagService>, "getEffectiveFlags">;
  onlineTranslation: Pick<Client<typeof OnlineTranslationService>, "translate">;
}

export interface AppAccountDeps {
  send: AppSend;
  /** The clients, once the transport can read `token` and refresh through `refresh`. */
  clients: () => AppAccountClients;
  /** The reader's native language: an online translation's target. */
  native: () => Promise<string>;
  /** « Traduction en ligne » is ticked in the extension of this device (the relayed setting). */
  chosen: boolean;
  /** The extension's last snapshot of the account's access, read before the app has a session. */
  relayedAccess: boolean;
  /** The page's own storage: where a sign-out from the app is remembered. */
  local?: Pick<Storage, "getItem" | "setItem" | "removeItem"> | null;
  /** The account's state changed (signed in, out, its access read): the page repaints. */
  onChange?: () => void;
}

/**
 * The reader ended the app's session from the app: the review offers no online translation until they
 * sign in again from the app's line (the spec's « Signing out of the app »).
 */
export const APP_SIGNED_OUT_KEY = "cymbra-lingua-app-signed-out";

export interface AppAccount {
  /** The bearer the transport attaches, from memory. */
  token(): string | null;
  /** The transport's single-flight refresh. */
  refresh(): Promise<boolean>;
  /** Reopen the session the Keychain holds, if any; whether one is usable. */
  resume(): Promise<boolean>;
  signedIn(): boolean;
  /** The reader signed out from the app, and has not signed in since. */
  signedOut(): boolean;
  /** Open the app's sheet and sign in; whether a session was opened. */
  signIn(): Promise<boolean>;
  /** End the app's session (revoked server-side, best effort) — the extension's is untouched. */
  signOut(): Promise<void>;
  /** « Traduction en ligne » in the review, under the selection card's rule. */
  online: ReviewOnlineSource;
}

/**
 * The Session's storage, in the app: the refresh token in the Keychain through the bridge, everything
 * else — the access token among it — in memory only, gone with the page.
 */
export function keychainArea(send: AppSend): AsyncStorageArea {
  const memory: Record<string, unknown> = {};
  return {
    async get(keys) {
      const list = keys == null ? Object.keys(memory) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const key of list) {
        if (key === REFRESH_KEY) {
          const reply = (await send({ type: "auth.readRefresh" })) as { refresh?: unknown } | null;
          if (typeof reply?.refresh === "string" && reply.refresh) out[key] = reply.refresh;
        } else if (key in memory) {
          out[key] = memory[key];
        }
      }
      return out;
    },
    async set(items) {
      for (const [key, value] of Object.entries(items)) {
        if (key !== REFRESH_KEY) {
          memory[key] = value;
          continue;
        }
        if (typeof value === "string" && value) await send({ type: "auth.writeRefresh", refresh: value });
        else await send({ type: "auth.clear" });
      }
    },
  };
}

function memoryArea(): AsyncStorageArea {
  const items: Record<string, unknown> = {};
  return {
    async get(keys) {
      const list = keys == null ? Object.keys(items) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in items).map((k) => [k, items[k]]));
    },
    async set(next) {
      Object.assign(items, next);
    },
  };
}

/** What the sheet answered: a provider's id_token, or nothing (closed, failed). */
function handed(reply: unknown): { provider: Provider; idToken: string } | null {
  const r = reply as { provider?: unknown; idToken?: unknown } | null | undefined;
  if ((r?.provider !== "apple" && r?.provider !== "google") || typeof r.idToken !== "string" || !r.idToken) return null;
  return { provider: r.provider, idToken: r.idToken };
}

export function appAccount(deps: AppAccountDeps): AppAccount {
  const session = new Session({
    auth: () => deps.clients().auth,
    sessionArea: memoryArea(),
    localArea: keychainArea(deps.send),
    // The provider flows of a browser do not exist here: the app's sheet runs them (signIn).
    getGoogleIdToken: async () => null,
    getAppleIdToken: async () => null,
  });
  /** The account's own access, read once signed in; until then the extension's snapshot decides. */
  let ownAccess: boolean | null = null;
  const signedIn = (): boolean => session.state().signedIn;
  const signedOut = (): boolean => {
    try {
      return deps.local?.getItem(APP_SIGNED_OUT_KEY) === "1";
    } catch {
      return false;
    }
  };
  const markSignedOut = (out: boolean): void => {
    try {
      if (out) deps.local?.setItem(APP_SIGNED_OUT_KEY, "1");
      else deps.local?.removeItem(APP_SIGNED_OUT_KEY);
    } catch {
      /* this launch only */
    }
  };

  async function readAccess(): Promise<void> {
    try {
      const wire = await deps.clients().flags.getEffectiveFlags({ knownVersion: "", app: "lingua" });
      ownAccess = flagOn(toValues(wire.flags), ONLINE_TRANSLATION_FLAG);
    } catch (e: unknown) {
      // Unreachable: keep what was known. The server checks the flag before any provider anyway.
      console.warn(
        "[Cymbra Lingua] the app's flags could not be read:",
        (e as { code?: unknown } | null)?.code ?? "error",
      );
    }
    deps.onChange?.();
  }

  async function signIn(): Promise<boolean> {
    let token: ReturnType<typeof handed>;
    try {
      token = handed(await deps.send({ type: "auth.signIn" }));
    } catch {
      token = null;
    }
    if (!token) return false;
    try {
      await session.signInWithIdToken(token.provider, token.idToken);
    } catch {
      return false;
    }
    markSignedOut(false);
    await readAccess();
    return true;
  }

  const online: ReviewOnlineSource = {
    offered: () => {
      if (!deps.chosen) return false;
      if (signedIn()) return ownAccess ?? deps.relayedAccess;
      return deps.relayedAccess && !signedOut();
    },
    async translate(request): Promise<OnlineTranslationResult> {
      const message = onlineMessage(request);
      if (!message) return ONLINE_UNAVAILABLE;
      // The first press without a session opens the sheet; a reader who closes it gets no translation.
      if (!signedIn() && !(await signIn())) return ONLINE_UNAVAILABLE;
      return answerOnlineTranslation(
        { client: () => deps.clients().onlineTranslation, native: deps.native, signedIn },
        message,
      );
    },
  };

  return {
    token: () => session.token(),
    refresh: () => session.refresh(),
    async resume() {
      const usable = await session.resume().catch(() => false);
      if (usable) await readAccess();
      return usable;
    },
    signedIn,
    signedOut,
    signIn,
    async signOut() {
      await session.signOut();
      ownAccess = null;
      markSignedOut(true);
      deps.onChange?.();
    },
    online,
  };
}
