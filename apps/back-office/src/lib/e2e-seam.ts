import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";

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
  /** Accounts for the admin directory (`listAccounts`); roles are mutated in place
   * by grant/revoke so the UI reflects the change on re-list. */
  accounts?: DirectoryAccount[];
  /** Force a method to reject with a ConnectError, keyed by method name. */
  fail?: Record<string, E2EFailure>;
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
    },
    score: {
      searchCatalog: async (req: { moderationStatus?: string; limit?: number }) => {
        failIfSet("searchCatalog");
        // The header stat cards issue count-only queries (limit 1); return the
        // per-status total for those, and the row list for the real query.
        if ((req.limit ?? 50) <= 1) {
          const total = counts[(req.moderationStatus ?? "pending") as keyof typeof counts] ?? 0;
          return { hits: [], total, nextOffset: 0 };
        }
        return { hits, total: hits.length, nextOffset: hits.length };
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

  setClientsForTest(clients);
}
