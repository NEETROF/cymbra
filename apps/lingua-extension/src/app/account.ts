import type { Client } from "@connectrpc/connect";
import type { AuthService } from "@/gen/auth_pb";
import type { FlagService } from "@/gen/flags_pb";
import type { OnlineTranslationService } from "@/gen/online_translation_pb";
import type { UserService } from "@/gen/user_pb";
import { type AccountHostDeps, handleAccountMessage } from "../account/host.ts";
import type { AccountMessage, AccountReply } from "../account/messages.ts";
import type { AccountPort } from "../account/profile.ts";
import { flagOn, toValues } from "../flags/store.ts";
import type { InterfaceLanguage } from "../i18n/index.ts";
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
//
// An email account (add-lingua-app-email-signin): the sheet's third entry answers `{method: "email"}`,
// and the page shows the extension's own email sign-in (`AccountFlow` in its app scope, email-panel.ts)
// whose messages are answered here by the background's own dispatch, `handleAccountMessage`, over this
// Session — `SignInLocal` for the `lingua` audience. The password stays in the page: it goes to Cymbra
// ID and nowhere else, never to the bridge nor to any storage.

/** The account messages the page sends while signing in with an email, answered over the app's Session. */
export type AppAccountSend = (message: AccountMessage) => Promise<AccountReply | null>;

/**
 * Show the email sign-in over `send`; resolves true once signed in, false when the reader closed it.
 * The page's panel (email-panel.ts); a seam so the session is tested without the DOM.
 */
export type EmailSignIn = (send: AppAccountSend) => Promise<boolean>;

/** What the sheet answered for an email account: the page's own form takes over. */
function choseEmail(reply: unknown): boolean {
  return (reply as { method?: unknown } | null | undefined)?.method === "email";
}

/** The panel never reaches the account's profile (no handle step in the app): a call is a bug, made loud. */
const NO_PROFILE: AccountPort = {
  profile: () => Promise.reject(new Error("the app reads no profile")),
  checkHandle: () => Promise.reject(new Error("the app sets no handle")),
  setHandle: () => Promise.reject(new Error("the app sets no handle")),
  deleteAccount: () => Promise.reject(new Error("the app deletes no account")),
  identities: () => Promise.reject(new Error("the app lists no identities")),
};

/**
 * The locale the email sign-in's requests carry (resending the code, requesting a reset —
 * `AccountLanguage.locale`, add-lingua-app-email-signin D3): the interface language the reader chose in
 * the extension when the shared state names it, else the device's whole language tag, else French.
 * The account's stored language wins server-side anyway.
 */
export function appAccountLocale(shared: InterfaceLanguage | null, deviceTag: string): string {
  return shared ?? (deviceTag || "fr");
}

/** What the account needs of the transport: the clients, bound to this session's bearer. */
export interface AppAccountClients {
  auth: Client<typeof AuthService>;
  flags: Pick<Client<typeof FlagService>, "getEffectiveFlags">;
  onlineTranslation: Pick<Client<typeof OnlineTranslationService>, "translate">;
  /** The account's handle and its email-and-password address, for the signed-in line. */
  user: Pick<Client<typeof UserService>, "getAccount" | "listIdentities">;
}

/**
 * Which account the app is signed in to, as its line names it (add-lingua-app-email-signin D9): the
 * handle, else the address of its email and password, else neither. In memory only, never stored.
 */
export interface AppAccountName {
  handle: string | null;
  email: string | null;
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
  /** The page's email sign-in, shown when the sheet answers `{method: "email"}`; absent = none. */
  emailSignIn?: EmailSignIn;
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
  /** The account signed in to, once read (D9); null signed out, or before and when it could not be read. */
  name(): AppAccountName | null;
  /** The reader signed out from the app, and has not signed in since. */
  signedOut(): boolean;
  /** Open the app's sheet and sign in; whether a session was opened. */
  signIn(): Promise<boolean>;
  /** End the app's session (revoked server-side, best effort) — the extension's is untouched. */
  signOut(): Promise<void>;
  /**
   * The account's id (add-lingua-app-deck-sync D7): the one `GetAccount` the line's name already read
   * this session, else read now. Never stored.
   */
  userId(): Promise<string>;
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
  /** The account's name for the signed-in line (D9): read once per session, in memory only. */
  let accountName: AppAccountName | null = null;
  /** The address an email sign-in used: the email fallback without another request. */
  let signedInEmail: string | null = null;
  /** The account's id as the name's `GetAccount` read it, this session (add-lingua-app-deck-sync D7). */
  let idRead: Promise<string> | null = null;
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

  /**
   * Which account the line names (D9): its handle (`GetAccount`, as the extension's account page reads
   * it), else the address of an email sign-in, else its email-and-password identity (`ListIdentities`).
   * A read that fails leaves the plain line; nothing is stored.
   */
  async function readName(): Promise<void> {
    try {
      const user = deps.clients().user;
      const reading = user.getAccount({});
      idRead = reading.then((a) => a.userId);
      idRead.catch(() => undefined);
      const handle = (await reading).handle || null;
      let email = signedInEmail;
      if (!handle && !email) {
        const local = (await user.listIdentities({})).identities.find((i) => i.provider === "local");
        email = local?.subject || null;
      }
      accountName = handle || email ? { handle, email } : null;
    } catch (e: unknown) {
      accountName = signedInEmail ? { handle: null, email: signedInEmail } : null;
      console.warn(
        "[Cymbra Lingua] the app's account could not be read:",
        (e as { code?: unknown } | null)?.code ?? "error",
      );
    }
    deps.onChange?.();
  }

  /** The account's access and its name, read together once a session is open. */
  async function readAccount(): Promise<void> {
    await Promise.all([readAccess(), readName()]);
  }

  /** The background's dispatch, in the page, over the app's Session: the same categories, the same copy. */
  const hostDeps: AccountHostDeps = {
    session,
    account: NO_PROFILE,
    // The sheet ran Apple and Google; the panel offers none, and hands nothing off.
    providers: () => ({ google: false, apple: false }),
    eraseLinguaData: () => Promise.reject(new Error("the app erases nothing")),
    onSignedIn: () => {},
  };
  const accountSend: AppAccountSend = async (message) => {
    const reply = await handleAccountMessage(message, hostDeps);
    // The address that signed in names the account when it has no handle (D9) — never the password.
    if (message.type === "account:signInLocal" && reply.ok) signedInEmail = message.email;
    return reply;
  };

  /** Signed in: the account's own access and name are read, the page repaints. */
  async function opened(): Promise<boolean> {
    markSignedOut(false);
    await readAccount();
    return true;
  }

  async function signIn(): Promise<boolean> {
    let reply: unknown;
    try {
      reply = await deps.send({ type: "auth.signIn" });
    } catch {
      reply = null;
    }
    signedInEmail = null;
    if (choseEmail(reply)) {
      if (!deps.emailSignIn) return false;
      const done = await deps.emailSignIn(accountSend).catch(() => false);
      return done && signedIn() ? opened() : false;
    }
    const token = handed(reply);
    if (!token) return false;
    try {
      await session.signInWithIdToken(token.provider, token.idToken);
    } catch {
      return false;
    }
    return opened();
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
      if (usable) await readAccount();
      return usable;
    },
    signedIn,
    name: () => (signedIn() && accountName ? { ...accountName } : null),
    async userId() {
      const read = await idRead?.catch(() => null);
      if (read) return read;
      return (await deps.clients().user.getAccount({})).userId;
    },
    signedOut,
    signIn,
    async signOut() {
      await session.signOut();
      ownAccess = null;
      accountName = null;
      signedInEmail = null;
      idRead = null;
      markSignedOut(true);
      deps.onChange?.();
    },
    online,
  };
}
