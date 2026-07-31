import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { setWebAuthClientForTest, WebAuthError, type WebAuthClient } from "@/lib/web-auth";

// E2E test seam (loaded ONLY when VITE_E2E=1 — see main.ts). Playwright seeds
// `window.__CYMBRA_E2E__` with canned data via addInitScript before the app boots;
// this turns that data into fake Connect clients so every flow (routing, guards,
// stores, i18n, error mapping) runs in a real browser with no backend. The fakes
// only implement the methods the console calls, cast to the generated client types.

export interface E2EFailure {
  code: number;
  message: string;
}

export interface E2EData {
  /** What `signInLocal`/`signInOidc`/`refresh` return. */
  tokens?: { accessToken: string; refreshToken: string };
  /** Simulates the HttpOnly refresh cookie existing at boot: when true, the fake
   * web-auth `refresh` re-mints `tokens` (so a reload stays signed in); when false
   * it 401s (no session). `loginAs` in the e2e fixtures sets this. */
  session?: boolean;
  /** Artificial latency (ms) on the fake web-auth `refresh`, to mimic a real network
   * round-trip. Used to catch the boot-order race where the router's initial guard
   * could run before the cookie re-mint completed. */
  refreshDelayMs?: number;
  /** Rows returned by the list search (limit > 1). */
  hits?: Record<string, unknown>[];
  /** Per-status totals returned by the count-only stat queries (limit 1). */
  counts?: { pending?: number; accepted?: number; rejected?: number };
  /** Metadata for `getCatalogScore`; falls back to a matching/first hit. */
  hit?: Record<string, unknown> | null;
  /** Bytes for `getCatalogScoreBytes`. */
  bytes?: number[] | null;
  /** Audit rows for `listRoleGrants`. */
  grants?: Record<string, unknown>[];
  /** The account's stored language returned by `getAccount` (change: sync-account-
   * language-preference); `setLocale` writes it here so a re-read reflects it. */
  accountLocale?: string;
  /** Accounts for the admin directory (`listAccounts`); roles are mutated in place
   * by grant/revoke so the UI reflects the change on re-list. */
  accounts?: DirectoryAccount[];
  /** Force a method to reject with a ConnectError, keyed by method name. */
  fail?: Record<string, E2EFailure>;
  /** Force a method to reject with a ConnectError exactly ONCE (then succeed) —
   * used to exercise the silent refresh-and-retry path. Keyed by method name. */
  failOnce?: Record<string, E2EFailure>;
}

interface DirectoryAccount {
  userId: string;
  handle?: string;
  displayName?: string;
  roles: string[];
}

declare global {
  interface Window {
    __CYMBRA_E2E__?: E2EData;
  }
}

export function installE2EClients(): void {
  const data: E2EData = window.__CYMBRA_E2E__ ?? {};
  const hits = data.hits ?? [];
  const counts = { pending: 0, accepted: 0, rejected: 0, ...(data.counts ?? {}) };
  const tokens = data.tokens ?? { accessToken: "", refreshToken: "r" };
  // Mutable copy so grant/revoke change roles and the next listAccounts reflects it.
  const accounts: DirectoryAccount[] = (data.accounts ?? []).map((a) => ({ ...a, roles: [...(a.roles ?? [])] }));

  function failIfSet(method: string): void {
    const f = data.fail?.[method];
    if (f) throw new ConnectError(f.message, f.code as Code);
  }

  // One-shot failures (consumed on first use) exercise the refresh-and-retry path:
  // the first call 401s, the interceptor refreshes via the cookie, then retries here
  // and succeeds.
  const failOnce = { ...(data.failOnce ?? {}) };
  function failOnceIfSet(method: string): void {
    const f = failOnce[method];
    if (f) {
      delete failOnce[method];
      throw new ConnectError(f.message, f.code as Code);
    }
  }

  const clients = {
    auth: {
      signInLocal: async () => {
        failIfSet("signInLocal");
        return tokens;
      },
      signInOidc: async () => {
        failIfSet("signInOidc");
        return tokens;
      },
      refresh: async () => tokens,
      revokeAccountSessions: async () => {
        failIfSet("revokeAccountSessions");
        return {};
      },
    },
    score: {
      searchCatalog: async (req: { moderationStatus?: string; limit?: number; offset?: number }) => {
        failOnceIfSet("searchCatalog");
        failIfSet("searchCatalog");
        // The header stat cards issue count-only queries (limit 1); return the
        // per-status total for those, and the row list for the real query.
        if ((req.limit ?? 50) <= 1) {
          const total = counts[(req.moderationStatus ?? "pending") as keyof typeof counts] ?? 0;
          return { hits: [], total, nextOffset: 0 };
        }
        // Real list query: paginate over the seeded rows exactly like the server, so
        // the offset/limit window (and the pager driven by `total`) can be exercised.
        const offset = req.offset ?? 0;
        const page = hits.slice(offset, offset + (req.limit ?? 50));
        return { hits: page, total: hits.length, nextOffset: offset + page.length };
      },
      setModerationStatus: async () => {
        failIfSet("setModerationStatus");
        return {};
      },
      getCatalogScore: async (req: { catalogId: string }) => {
        failIfSet("getCatalogScore");
        return data.hit ?? hits.find((h) => h.id === req.catalogId) ?? hits[0] ?? {};
      },
      getCatalogScoreBytes: async () => {
        failIfSet("getCatalogScoreBytes");
        return { data: Uint8Array.from(data.bytes ?? [1, 2, 3]) };
      },
    },
    user: {
      grantRole: async (req: { userId: string; role: string }) => {
        failIfSet("grantRole");
        const acc = accounts.find((a) => a.userId === req.userId);
        if (acc && !acc.roles.includes(req.role)) acc.roles.push(req.role);
        return {};
      },
      revokeRole: async (req: { userId: string; role: string }) => {
        failIfSet("revokeRole");
        const acc = accounts.find((a) => a.userId === req.userId);
        if (acc) acc.roles = acc.roles.filter((r) => r !== req.role);
        return {};
      },
      listRoleGrants: async () => ({ grants: data.grants ?? [] }),
      getAccount: async () => {
        failIfSet("getAccount");
        return { userId: "u1", locale: data.accountLocale };
      },
      setLocale: async (req: { locale: string }) => {
        failIfSet("setLocale");
        if (req.locale) data.accountLocale = req.locale; // reflect the write
        return { userId: "u1", locale: data.accountLocale };
      },
      listAccounts: async (req: { query: string; limit: number; offset: number }) => {
        failIfSet("listAccounts");
        const q = (req.query ?? "").toLowerCase();
        const filtered = q
          ? accounts.filter(
              (a) => (a.handle ?? "").toLowerCase().includes(q) || (a.displayName ?? "").toLowerCase().includes(q),
            )
          : accounts;
        const page = filtered.slice(req.offset ?? 0, (req.offset ?? 0) + (req.limit ?? 25));
        return { accounts: page, total: filtered.length };
      },
    },
  } as unknown as Clients;

  // Web-auth (cookie) fake: sign-in mints a token and "sets the cookie" (session on);
  // refresh re-mints only while a session exists; logout ends it. `data.session` seeds
  // the boot state (an existing HttpOnly cookie) so a reload silently re-mints.
  let session = data.session ?? false;
  // Reuse the same `fail` map as the gRPC fakes, translating the Connect code to the
  // HTTP status the real web-auth surface would return.
  const codeToHttp: Record<number, number> = { 16: 401, 7: 403, 5: 404, 9: 412, 3: 400, 8: 429, 14: 503 };
  function webAuthFail(method: string): void {
    const f = data.fail?.[method];
    if (f) throw new WebAuthError(codeToHttp[f.code] ?? 500, f.message);
  }
  const webAuthClient: WebAuthClient = {
    signInLocal: async () => {
      webAuthFail("signInLocal");
      session = true;
      return { accessToken: tokens.accessToken };
    },
    signInOidc: async () => {
      webAuthFail("signInOidc");
      session = true;
      return { accessToken: tokens.accessToken };
    },
    refresh: async () => {
      // Optional latency so the boot re-mint isn't instantaneous — exercises the
      // router-install boot order (the guard must wait for the session to resolve).
      if (data.refreshDelayMs) await new Promise((r) => setTimeout(r, data.refreshDelayMs));
      if (!session) throw new WebAuthError(401, "no session");
      return { accessToken: tokens.accessToken };
    },
    logout: async () => {
      session = false;
    },
  };

  setClientsForTest(clients);
  setWebAuthClientForTest(webAuthClient);
}
