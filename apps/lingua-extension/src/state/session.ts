import type { Client } from "@connectrpc/connect";
import type { AuthService } from "@/gen/auth_pb";
import type { AsyncStorageArea } from "./storage.ts";

// The account session (add-lingua-connected-clients §1). Owns the token pair and the
// sign-in / refresh / sign-out flows against the AuthService. Local-first: without a
// session nothing changes, and signing out never touches the reader's own state.
//
// Tokens are split by volatility (design D1): the short-lived ACCESS token in
// chrome.storage.session (in memory, gone when the browser closes, never on disk), the
// rotating server-revocable REFRESH token in chrome.storage.local (so the session
// survives a browser restart). A background service worker is ephemeral, so the access
// token is also mirrored in memory for the transport's synchronous getter and rebuilt
// from the refresh token on wake.

const ACCESS_KEY = "cymbra-lingua-access"; // chrome.storage.session
const REFRESH_KEY = "cymbra-lingua-refresh"; // chrome.storage.local
/**
 * The last sign-in failure, in chrome.storage.session (transient, gone on browser
 * close). Chrome tears the browser-action popup down when the Google auth window takes
 * focus, so the popup's awaited result never renders; persisting the error here lets the
 * popup surface it on its next open. Cleared on a successful sign-in.
 */
export const SIGNIN_ERROR_KEY = "cymbra-lingua-signin-error";
const AUDIENCE = "lingua";

/** Everything the session needs, injected so the flows are testable without Chrome. */
export interface SessionDeps {
  /** The AuthService client (from api() in the background, a fake in tests). */
  auth: () => Client<typeof AuthService>;
  /** chrome.storage.session-backed area (access token). */
  sessionArea: AsyncStorageArea;
  /** chrome.storage.local-backed area (refresh token). */
  localArea: AsyncStorageArea;
  /** Run the provider OAuth flow and return an id_token (chrome.identity in prod). */
  getGoogleIdToken: () => Promise<string>;
}

export interface SessionState {
  signedIn: boolean;
}

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

  /** Email + password (existing SignInLocal), scoped to the `lingua` audience. */
  async signInLocal(email: string, password: string): Promise<void> {
    // No recordError here: the local form stays open on failure, so its error renders
    // inline — persisting it would re-show stale on the next popup open. Only the Google
    // flow (which tears the popup down) needs the persisted channel.
    await this.store(await this.deps.auth().signInLocal({ email, password, audience: AUDIENCE }));
  }

  /** "Continue with Google": run the OAuth flow, exchange the id_token via SignInOidc. */
  async signInWithGoogle(): Promise<void> {
    try {
      const idToken = await this.deps.getGoogleIdToken();
      await this.store(await this.deps.auth().signInOidc({ idToken, audience: AUDIENCE }));
    } catch (e) {
      await this.recordError(e);
      throw e;
    }
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

  /** Persist a sign-in failure so the (possibly torn-down) popup can show it on reopen. */
  private async recordError(e: unknown): Promise<void> {
    await this.deps.sessionArea.set({ [SIGNIN_ERROR_KEY]: e instanceof Error ? e.message : String(e) });
  }

  private async purge(): Promise<void> {
    this.access = null;
    this.refreshTok = null;
    await this.deps.sessionArea.set({ [ACCESS_KEY]: null });
    await this.deps.localArea.set({ [REFRESH_KEY]: null });
  }
}

function strOrNull(v: unknown): string | null {
  return typeof v === "string" && v.length > 0 ? v : null;
}
