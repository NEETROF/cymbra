import { defineStore } from "pinia";
import { webAuth } from "@/lib/web-auth";
import { adminScopes, decodeClaims, isAdmin, isModerator, type Scope, type TokenClaims } from "@/lib/jwt";
import { useLocaleStore } from "@/stores/locale";

// The back office targets the dedicated `back-office` audience: its token carries
// the admin's real roles across global/music/live, so a single session can
// administer every scope they are entitled to (change: scope-aware-role-admin).
const AUDIENCE = "back-office";

interface AuthState {
  // Memory-only: the access token is NEVER persisted (localStorage/sessionStorage/
  // non-HttpOnly cookie). The refresh token lives only in the HttpOnly cookie the
  // server manages — unreadable by this JS. A reload silently re-mints via `bootstrap`.
  accessToken: string | null;
  claims: TokenClaims;
  // Whether the initial cookie-refresh attempt has run, so the router can wait for a
  // decided auth state (signed-in vs no-session) instead of flashing sign-in.
  bootstrapped: boolean;
}

export const useAuthStore = defineStore("auth", {
  state: (): AuthState => ({
    accessToken: null,
    claims: { roles: [], rolesByScope: {} },
    bootstrapped: false,
  }),
  getters: {
    isAuthenticated: (s): boolean => !!s.accessToken,
    roles: (s): string[] => s.claims.roles,
    isModerator: (s): boolean => isModerator(s.claims.roles),
    isAdmin: (s): boolean => isAdmin(s.claims.roles),
    userId: (s): string | undefined => s.claims.sub,
    /** The scopes the signed-in admin may administer (empty for a non-admin). The
     * Roles page and its scope selector are driven by this — a `music`-only admin
     * never sees `live` or `global` (change: scope-aware-role-admin). */
    adminScopes: (s): Scope[] => adminScopes(s.claims.rolesByScope),
  },
  actions: {
    setToken(accessToken: string) {
      this.accessToken = accessToken;
      this.claims = decodeClaims(accessToken);
    },
    async signInLocal(email: string, password: string) {
      const { accessToken } = await webAuth().signInLocal(email, password, AUDIENCE);
      this.setToken(accessToken);
      // Reconcile the account language into the UI (change: sync-account-language-
      // preference). Fire-and-forget so sign-in never blocks on it.
      void useLocaleStore().reconcile();
    },
    async signInOidc(idToken: string) {
      const { accessToken } = await webAuth().signInOidc(idToken, AUDIENCE);
      this.setToken(accessToken);
      void useLocaleStore().reconcile();
    },
    /** Mint a fresh access token from the refresh cookie. Throws if there is no
     * valid session (caller decides: silent on boot, sign-out on a live 401). */
    async refresh() {
      const { accessToken } = await webAuth().refresh();
      this.setToken(accessToken);
    },
    /** On app load, try to re-mint an access token from the refresh cookie. A missing
     * or invalid cookie just means "no session" — swallow it and land on sign-in. */
    async bootstrap() {
      try {
        await this.refresh();
        // Session re-minted from the cookie: reconcile the account language into the
        // UI (change: sync-account-language-preference). Fire-and-forget so boot is
        // never blocked on it.
        void useLocaleStore().reconcile();
      } catch {
        // No session; stay signed out (no console noise).
      } finally {
        this.bootstrapped = true;
      }
    },
    /** Revoke the server session (clears the cookie) and drop the in-memory token. */
    async signOut() {
      await webAuth().logout();
      this.accessToken = null;
      this.claims = { roles: [], rolesByScope: {} };
    },
  },
});
