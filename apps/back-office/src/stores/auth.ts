import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { decodeClaims, isAdmin, isModerator, type TokenClaims } from "@/lib/jwt";

// The back office targets the `music` audience so `music`-scoped roles
// (moderator/admin) flow into the token (design D2).
const AUDIENCE = "music";
const STORAGE_KEY = "cymbra.bo.tokens";

interface StoredTokens {
  accessToken: string;
  refreshToken: string;
}

function load(): StoredTokens | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as StoredTokens) : null;
  } catch {
    return null;
  }
}

interface AuthState {
  accessToken: string | null;
  refreshToken: string | null;
  claims: TokenClaims;
  error: string | null;
}

export const useAuthStore = defineStore("auth", {
  state: (): AuthState => {
    const stored = load();
    return {
      accessToken: stored?.accessToken ?? null,
      refreshToken: stored?.refreshToken ?? null,
      claims: stored ? decodeClaims(stored.accessToken) : { roles: [] },
      error: null,
    };
  },
  getters: {
    isAuthenticated: (s): boolean => !!s.accessToken,
    roles: (s): string[] => s.claims.roles,
    isModerator: (s): boolean => isModerator(s.claims.roles),
    isAdmin: (s): boolean => isAdmin(s.claims.roles),
    userId: (s): string | undefined => s.claims.sub,
  },
  actions: {
    setTokens(accessToken: string, refreshToken: string) {
      this.accessToken = accessToken;
      this.refreshToken = refreshToken;
      this.claims = decodeClaims(accessToken);
      this.error = null;
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ accessToken, refreshToken }));
    },
    async signInLocal(email: string, password: string) {
      const pair = await api().auth.signInLocal({ email, password, audience: AUDIENCE });
      this.setTokens(pair.accessToken, pair.refreshToken);
    },
    async signInOidc(idToken: string) {
      const pair = await api().auth.signInOidc({ idToken, audience: AUDIENCE });
      this.setTokens(pair.accessToken, pair.refreshToken);
    },
    async refresh() {
      if (!this.refreshToken) return;
      const pair = await api().auth.refresh({ refreshToken: this.refreshToken });
      this.setTokens(pair.accessToken, pair.refreshToken);
    },
    signOut() {
      this.accessToken = null;
      this.refreshToken = null;
      this.claims = { roles: [] };
      localStorage.removeItem(STORAGE_KEY);
    },
  },
});
