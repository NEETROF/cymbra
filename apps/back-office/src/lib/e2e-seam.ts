import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { setWebAuthClientForTest, WebAuthError, type WebAuthClient } from "@/lib/web-auth";
import { setRegeneratePreviewForTest, setUploadForTest } from "@/stores/soundfonts";

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
  /** Curator reliability returned by `getCuratorReliability` (change: add-curation-
   * rewards). Fields mirror the proto (totalRatings, coverageContribution,
   * alignmentRate 0–1, settledCount, alignedCount). */
  reliability?: Record<string, unknown>;
  /** The account's stored language returned by `getAccount` (change: sync-account-
   * language-preference); `setLocale` writes it here so a re-read reflects it. */
  accountLocale?: string;
  /** Accounts for the admin directory (`listAccounts`); roles are mutated in place
   * by grant/revoke so the UI reflects the change on re-list. */
  accounts?: DirectoryAccount[];
  /** SoundFont catalog rows for the admin management screen (change:
   * add-soundfont-back-office-management). Mutated in place by add/edit/delete. */
  soundfonts?: Record<string, unknown>[];
  /** Usage-analytics fixtures for the "Usage" console (change: add-feature-usage-
   * analytics): the distinct-users summary, the data-driven action list, and the
   * action breakdown. */
  usageSummary?: {
    totalUsers?: number;
    byPlatform?: { platform: string; users: number }[];
    byDeviceClass?: { deviceClass: string; users: number }[];
  };
  usageActions?: string[];
  usageBreakdown?: { action: string; variant?: string; events: number }[];
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
  /** Roles in the `music` scope (shorthand). Seeds may use this OR `rolesByScope`. */
  roles?: string[];
  /** Roles grouped by scope (`global`/`music`/`live`); scope-aware role admin. */
  rolesByScope?: Record<string, string[]>;
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
  // Mutable copy so add/edit/delete change the catalog the next list reflects.
  const soundfonts: Record<string, unknown>[] = (data.soundfonts ?? []).map((f) => ({ ...f }));
  // Mutable per-scope copy so grant/revoke change roles in the right scope and the
  // next listAccounts reflects it. A seed's flat `roles` is treated as `music`.
  const byScope: { userId: string; handle?: string; displayName?: string; roles: Record<string, string[]> }[] = (
    data.accounts ?? []
  ).map((a) => {
    const roles: Record<string, string[]> = {};
    const src = a.rolesByScope ?? { music: a.roles ?? [] };
    for (const [scope, rs] of Object.entries(src)) roles[scope] = [...rs];
    return { userId: a.userId, handle: a.handle, displayName: a.displayName, roles };
  });

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
      // Read-only curator reliability (change: add-curation-rewards); moderator/admin
      // gated server-side. `failOnce`/`fail` exercise the retry and error-mapping paths.
      getCuratorReliability: async () => {
        failOnceIfSet("getCuratorReliability");
        failIfSet("getCuratorReliability");
        return (
          data.reliability ?? {
            totalRatings: 0,
            coverageContribution: 0,
            alignmentRate: 0,
            settledCount: 0,
            alignedCount: 0,
          }
        );
      },
      // Public catalog listing (preview font picker on review/detail).
      listSoundFonts: async () => ({ soundfonts: soundfonts.map(({ id, label }) => ({ id, label })) }),
      // SoundFont catalog admin (change: add-soundfont-back-office-management).
      adminListSoundFonts: async () => {
        failIfSet("adminListSoundFonts");
        return { soundfonts: soundfonts.map((f) => ({ hasObject: true, ...f })) };
      },
      updateSoundFont: async (req: { id: string } & Record<string, unknown>) => {
        failIfSet("updateSoundFont");
        const f = soundfonts.find((s) => s.id === req.id);
        if (f) Object.assign(f, req);
        return {};
      },
      deleteSoundFont: async (req: { id: string }) => {
        failIfSet("deleteSoundFont");
        const i = soundfonts.findIndex((s) => s.id === req.id);
        if (i >= 0) soundfonts.splice(i, 1);
        return {};
      },
      // Moderation decision (change: add-soundfont-moderation).
      setSoundFontModerationStatus: async (req: { id: string; status: string }) => {
        failIfSet("setSoundFontModerationStatus");
        const f = soundfonts.find((s) => s.id === req.id);
        if (f) f.moderationStatus = req.status;
        return {};
      },
    },
    user: {
      grantRole: async (req: { userId: string; scope: string; role: string }) => {
        failIfSet("grantRole");
        const acc = byScope.find((a) => a.userId === req.userId);
        if (acc) {
          const rs = (acc.roles[req.scope] ??= []);
          if (!rs.includes(req.role)) rs.push(req.role);
        }
        return {};
      },
      revokeRole: async (req: { userId: string; scope: string; role: string }) => {
        failIfSet("revokeRole");
        const acc = byScope.find((a) => a.userId === req.userId);
        if (acc && acc.roles[req.scope]) acc.roles[req.scope] = acc.roles[req.scope].filter((r) => r !== req.role);
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
          ? byScope.filter(
              (a) => (a.handle ?? "").toLowerCase().includes(q) || (a.displayName ?? "").toLowerCase().includes(q),
            )
          : byScope;
        const page = filtered.slice(req.offset ?? 0, (req.offset ?? 0) + (req.limit ?? 25)).map((a) => ({
          userId: a.userId,
          handle: a.handle,
          displayName: a.displayName,
          rolesByScope: Object.entries(a.roles).map(([scope, roles]) => ({ scope, roles })),
        }));
        return { accounts: page, total: filtered.length };
      },
    },
    usage: {
      getUsersSummary: async () => {
        failIfSet("getUsersSummary");
        const s = data.usageSummary ?? {};
        return {
          totalUsers: BigInt(s.totalUsers ?? 0),
          byPlatform: (s.byPlatform ?? []).map((p) => ({ platform: p.platform, users: BigInt(p.users) })),
          byDeviceClass: (s.byDeviceClass ?? []).map((d) => ({ deviceClass: d.deviceClass, users: BigInt(d.users) })),
        };
      },
      getActionBreakdown: async () => {
        failIfSet("getActionBreakdown");
        return {
          rows: (data.usageBreakdown ?? []).map((r) => ({
            action: r.action,
            variant: r.variant ?? "",
            events: BigInt(r.events),
          })),
        };
      },
      listActions: async () => ({ actions: data.usageActions ?? [] }),
      getUsageSeries: async (req: { dimension: number }) => {
        failIfSet("getUsageSeries");
        const today = new Date().toISOString().slice(0, 10);
        const s = data.usageSummary ?? {};
        // Synthesise a single-day point per series so the line charts render.
        if (req.dimension === 0) {
          return {
            points: (s.byPlatform ?? []).map((p) => ({ day: today, series: p.platform, value: BigInt(p.users) })),
          };
        }
        if (req.dimension === 1) {
          return {
            points: (s.byDeviceClass ?? []).map((d) => ({ day: today, series: d.deviceClass, value: BigInt(d.users) })),
          };
        }
        return {
          points: (data.usageBreakdown ?? []).map((r) => ({ day: today, series: r.action, value: BigInt(r.events) })),
        };
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

  // The `.sf2` upload is an HTTP route (not a gRPC client), so route it through the
  // same seed: a successful "upload" appends the font, which the next admin-list shows.
  setUploadForTest(async (font) => {
    if (data.fail?.upload) {
      const f = data.fail.upload;
      throw new ConnectError(f.message, f.code as Code);
    }
    soundfonts.push({
      id: font.id,
      label: font.label,
      objectKey: `${font.id}.sf2`,
      instrument: font.instrument,
      license: font.license,
      attribution: font.attribution,
      hasObject: true,
    });
  });

  // "Generate sample" is likewise an HTTP route (change:
  // add-soundfont-entitlement-previews) — a successful regeneration is a no-op here
  // (the preview object is server-side; the console only reflects success/failure).
  setRegeneratePreviewForTest(async () => {
    if (data.fail?.regeneratePreview) {
      const f = data.fail.regeneratePreview;
      throw new ConnectError(f.message, f.code as Code);
    }
  });

  setClientsForTest(clients);
  setWebAuthClientForTest(webAuthClient);
}
