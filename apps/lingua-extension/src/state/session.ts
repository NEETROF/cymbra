import type { Client } from "@connectrpc/connect";
import type { AuthService } from "@/gen/auth_pb";
import { AccountError, type AuthErrorKind, authErrorOf } from "./auth-errors.ts";
import type { Provider } from "./oidc.ts";
import type { AsyncStorageArea } from "./storage.ts";

// The account session (add-lingua-connected-clients §1, add-lingua-account-parity). Owns
// the token pair and every AuthService flow: sign-in (email, Google, Apple), sign-up,
// email verification, password reset, refresh and sign-out. Local-first: without a
// session nothing changes, and signing out never touches the reader's own state.
//
// Tokens are split by volatility (design D1): the short-lived ACCESS token in
// chrome.storage.session (in memory, gone when the browser closes, never on disk), the
// rotating server-revocable REFRESH token in chrome.storage.local (so the session
// survives a browser restart). A background service worker is ephemeral, so the access
// token is also mirrored in memory for the transport's synchronous getter and rebuilt
// from the refresh token on wake.
//
// Every failure is rethrown as an AccountError carrying a category (design D7): no
// caller ever sees — or can display — a raw gRPC/Connect message.

const ACCESS_KEY = "cymbra-lingua-access"; // chrome.storage.session
const REFRESH_KEY = "cymbra-lingua-refresh"; // chrome.storage.local
/**
 * The last provider sign-in failure, in chrome.storage.session (transient, gone on browser
 * close). Chrome tears the browser-action popup down when the provider's auth window takes
 * focus, so the popup's awaited result never renders; persisting the failure here lets the
 * popup surface it on its next open. Cleared on a successful sign-in.
 */
export const SIGNIN_ERROR_KEY = "cymbra-lingua-signin-error";
const AUDIENCE = "lingua";

/** What SIGNIN_ERROR_KEY holds: the provider and the category, never a message. */
export interface PersistedSignInError {
  provider: Provider;
  kind: AuthErrorKind;
}

export function isPersistedSignInError(v: unknown): v is PersistedSignInError {
  const e = v as Partial<PersistedSignInError> | null;
  return (e?.provider === "google" || e?.provider === "apple") && typeof e.kind === "string";
}

/** Everything the session needs, injected so the flows are testable without Chrome. */
export interface SessionDeps {
  /** The AuthService client (from api() in the background, a fake in tests). */
  auth: () => Client<typeof AuthService>;
  /** chrome.storage.session-backed area (access token). */
  sessionArea: AsyncStorageArea;
  /** chrome.storage.local-backed area (refresh token). */
  localArea: AsyncStorageArea;
  /** Run Google's flow: the id_token, or null when the reader closed the window. */
  getGoogleIdToken: () => Promise<string | null>;
  /** Run Apple's flow: the id_token, or null when the reader closed the window. */
  getAppleIdToken: () => Promise<string | null>;
}

export interface SessionState {
  signedIn: boolean;
}

export type ProviderOutcome = "signedIn" | "cancelled";

export class Session {
  private access: string | null = null;
  private refreshTok: string | null = null;

  constructor(private readonly deps: SessionDeps) {}

  /** The current access token, for the transport's synchronous auth interceptor. */
  token(): string | null {
    return this.access;
  }

  state(): SessionState {
    return { signedIn: this.refreshTok != null };
  }

  /**
   * Restore a session at startup: reload the refresh token (survives a restart) and the
   * cached access token (survives a service-worker nap). With only the refresh token —
   * the worker restarted and the in-memory access token was lost — mint a fresh one.
   * Returns whether a usable session was restored.
   */
  async resume(): Promise<boolean> {
    this.refreshTok = strOrNull((await this.deps.localArea.get(REFRESH_KEY))[REFRESH_KEY]);
    this.access = strOrNull((await this.deps.sessionArea.get(ACCESS_KEY))[ACCESS_KEY]);
    if (this.access) return true;
    if (this.refreshTok) return this.refresh();
    return false;
  }

  /** Email + password (SignInLocal), scoped to the `lingua` audience. */
  async signInLocal(email: string, password: string): Promise<void> {
    // No recordError here: the local form stays open on failure, so its error renders
    // inline — persisting it would re-show stale on the next popup open. Only the provider
    // flows (which tear the popup down) need the persisted channel.
    await categorized(async () =>
      this.store(await this.deps.auth().signInLocal({ email, password, audience: AUDIENCE })),
    );
  }

  /**
   * "Continue with Google/Apple": run the provider flow, exchange the id_token via
   * SignInOidc. A closed window is a cancel — no RPC, nothing persisted.
   */
  async signInWithProvider(provider: Provider): Promise<ProviderOutcome> {
    try {
      const getIdToken = provider === "apple" ? this.deps.getAppleIdToken : this.deps.getGoogleIdToken;
      const idToken = await getIdToken();
      if (idToken == null) return "cancelled";
      await this.store(await this.deps.auth().signInOidc({ idToken, audience: AUDIENCE }));
      return "signedIn";
    } catch (e) {
      const kind = authErrorOf(e);
      await this.recordError({ provider, kind });
      throw new AccountError(kind);
    }
  }

  /** Create a local account; the backend emails a verification code (in `locale`). */
  async signUp(email: string, password: string, locale: string): Promise<void> {
    await categorized(() => this.deps.auth().signUpLocal({ email, password, locale }));
  }

  /** Confirm an email with the emailed code. */
  async verifyEmail(code: string): Promise<void> {
    await categorized(() => this.deps.auth().verifyEmail({ token: code }));
  }

  async resendVerification(email: string, locale: string): Promise<void> {
    await categorized(() => this.deps.auth().resendVerification({ email, locale }));
  }

  /** Same answer whether or not the account exists (the backend never enumerates). */
  async requestPasswordReset(email: string, locale: string): Promise<void> {
    await categorized(() => this.deps.auth().requestPasswordReset({ email, locale }));
  }

  async resetPassword(code: string, newPassword: string): Promise<void> {
    await categorized(() => this.deps.auth().resetPassword({ token: code, newPassword }));
  }

  /**
   * Refresh the access token from the refresh token (the transport's single retry path).
   * A failed refresh means the session is truly gone, so it purges. Returns success.
   */
  async refresh(): Promise<boolean> {
    if (!this.refreshTok) return false;
    try {
      await this.store(await this.deps.auth().refresh({ refreshToken: this.refreshTok }));
      return true;
    } catch {
      await this.purge();
      return false;
    }
  }

  /** Revoke the refresh token server-side (best effort) and purge locally. */
  async signOut(): Promise<void> {
    const tok = this.refreshTok;
    if (tok) {
      try {
        await this.deps.auth().logout({ refreshToken: tok });
      } catch {
        // Best effort: purge locally regardless of the server's answer.
      }
    }
    await this.purge();
  }

  private async store(pair: { accessToken: string; refreshToken: string }): Promise<void> {
    this.access = pair.accessToken;
    this.refreshTok = pair.refreshToken;
    // Clear any stale sign-in error alongside the new access token (one success wipes it).
    await this.deps.sessionArea.set({ [ACCESS_KEY]: pair.accessToken, [SIGNIN_ERROR_KEY]: null });
    await this.deps.localArea.set({ [REFRESH_KEY]: pair.refreshToken });
  }

  /** Persist a provider failure so the (possibly torn-down) popup can show it on reopen. */
  private async recordError(error: PersistedSignInError): Promise<void> {
    try {
      await this.deps.sessionArea.set({ [SIGNIN_ERROR_KEY]: error });
    } catch {
      // storage.session unavailable: the live reply still carries the category.
    }
  }

  private async purge(): Promise<void> {
    this.access = null;
    this.refreshTok = null;
    await this.deps.sessionArea.set({ [ACCESS_KEY]: null });
    await this.deps.localArea.set({ [REFRESH_KEY]: null });
  }
}

/** Run a call, rethrowing any failure as a categorized AccountError. */
async function categorized<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    throw new AccountError(authErrorOf(e));
  }
}

function strOrNull(v: unknown): string | null {
  return typeof v === "string" && v.length > 0 ? v : null;
}
