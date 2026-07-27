import { defineStore } from "pinia";
import { webAuth } from "@/lib/web-auth";
import { decodeClaims, isAdmin, isModerator, type TokenClaims } from "@/lib/jwt";

// The back office targets the `music` audience so `music`-scoped roles
// (moderator/admin) flow into the token (design D2).
const AUDIENCE = "music";

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
    claims: { roles: [] },
    bootstrapped: false,
  }),
  getters: {
    isAuthenticated: (s): boolean => !!s.accessToken,
    roles: (s): string[] => s.claims.roles,
    isModerator: (s): boolean => isModerator(s.claims.roles),
    isAdmin: (s): boolean => isAdmin(s.claims.roles),
    userId: (s): string | undefined => s.claims.sub,
  },
  actions: {
    setToken(accessToken: string) {
      this.accessToken = accessToken;
      this.claims = decodeClaims(accessToken);
    },
    async signInLocal(email: string, password: string) {
      const { accessToken } = await webAuth().signInLocal(email, password, AUDIENCE);
      this.setToken(accessToken);
    },
    async signInOidc(idToken: string) {
      const { accessToken } = await webAuth().signInOidc(idToken, AUDIENCE);
      this.setToken(accessToken);
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
      this.claims = { roles: [] };
    },
  },
});
