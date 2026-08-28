import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { setWebAuthClientForTest, WebAuthError, type WebAuthClient } from "@/lib/web-auth";
import { setRegeneratePreviewForTest, setUploadForTest } from "@/stores/soundfonts";
import { setRegenerateScorePreviewForTest } from "@/stores/catalog";

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
  /** Declared flag/config keys for the "Notifications" panel (change:
   * add-push-notifications), which is a filtered view over the flag registry.
   * Mutated in place by setFlag/setConfig/clearOverride so the panel's re-read
   * reflects the change. `value` is a bool for a flag, a number for an hour. */
  flags?: E2EFlag[];
  /** Plan console fixtures (change: add-premium-subscription): campaigns (mutated in
   * place by create/close), per-account plan data (mutated by grant/revoke/enrol —
   * keyed by user id; a handle lookup resolves through `accounts`), and the
   * deterministic clear-text codes `mintCodes` hands out. */
  campaigns?: E2ECampaign[];
  plans?: Record<string, E2EAccountPlan>;
  mintedCodes?: string[];
  /** Force a method to reject with a ConnectError, keyed by method name. */
  fail?: Record<string, E2EFailure>;
  /** Force a method to reject with a ConnectError exactly ONCE (then succeed) —
   * used to exercise the silent refresh-and-retry path. Keyed by method name. */
  failOnce?: Record<string, E2EFailure>;
}

/** One declared key as the e2e seam models it (a bool flag or an int config). */
export interface E2EFlag {
  key: string;
  app?: string;
  value: boolean | number;
  hasOverride?: boolean;
  editable?: boolean;
  /** `global` | `staff_only` | `premium_only` | `beta:<key>` (default global). */
  rolloutScope?: string;
}

/** One campaign as the seam models it (a subset of `CampaignMsg`). */
export interface E2ECampaign {
  key: string;
  name: string;
  kind: "premium_trial" | "feature";
  durationDays?: number;
  acceptsEnrolment?: boolean;
  enrollmentClosesAt?: string;
  closedAt?: string;
}

/** One account's plan data: entitlement rows + memberships; the effective plan is
 * derived like the server does (any active row ⇒ premium; a trial-campaign row marks
 * the trial). */
export interface E2EAccountPlan {
  rows?: Record<string, unknown>[];
  memberships?: Record<string, unknown>[];
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

/** Manual-QA fallback: when Playwright hasn't injected `window.__CYMBRA_E2E__`
 * (e.g. a human driving the VITE_E2E dev server), the seed may be placed in
 * `sessionStorage["cymbra.e2e-seed"]` (JSON) and survives the reload that boots
 * the app. Still e2e-only code — never bundled without VITE_E2E. */
function readSessionSeed(): E2EData | undefined {
  try {
    const raw = sessionStorage.getItem("cymbra.e2e-seed");
    return raw ? (JSON.parse(raw) as E2EData) : undefined;
  } catch {
    return undefined;
  }
}

export function installE2EClients(): void {
  const data: E2EData = window.__CYMBRA_E2E__ ?? readSessionSeed() ?? {};
  const hits = data.hits ?? [];
  const counts = { pending: 0, accepted: 0, rejected: 0, ...(data.counts ?? {}) };
  const tokens = data.tokens ?? { accessToken: "", refreshToken: "r" };
  // Mutable copy so add/edit/delete change the catalog the next list reflects.
  const soundfonts: Record<string, unknown>[] = (data.soundfonts ?? []).map((f) => ({ ...f }));
  // Mutable copy so a flag/config write changes what the next list returns.
  const flags: E2EFlag[] = (data.flags ?? []).map((f) => ({ ...f }));
  const findFlag = (key: string) => flags.find((f) => f.key === key);
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

  // Plan console state (change: add-premium-subscription). Mutable copies so grant /
  // revoke / enrol / create / close change what the next lookup or list returns.
  const campaigns: E2ECampaign[] = (data.campaigns ?? []).map((c) => ({ acceptsEnrolment: true, ...c }));
  const plans: Record<string, { rows: Record<string, unknown>[]; memberships: Record<string, unknown>[] }> = {};
  for (const [uid, p] of Object.entries(data.plans ?? {})) {
    plans[uid] = {
      rows: (p.rows ?? []).map((r) => ({ ...r })),
      memberships: (p.memberships ?? []).map((m) => ({ ...m })),
    };
  }
  const planOf = (uid: string) => (plans[uid] ??= { rows: [], memberships: [] });
  /** Resolve `{userId, handle}` to a directory user id (handle → id via `accounts`). */
  const resolveUser = (req: { userId?: string; handle?: string }): string => {
    if (req.userId) return req.userId;
    const acc = byScope.find((a) => a.handle === req.handle);
    if (!acc) throw new ConnectError("account not found", Code.NotFound);
    return acc.userId;
  };
  const activeRows = (uid: string) =>
    planOf(uid).rows.filter((r) => (r.status ?? "active") === "active" && !r.revokedAt);
  const campaignByKey = (key: string) => campaigns.find((c) => c.key === key);
  const campaignById = (id: string | undefined) => campaigns.find((c) => c.key === id);
  const trialRow = (uid: string) =>
    activeRows(uid).find((r) => campaignById(r.campaignId as string | undefined)?.kind === "premium_trial");
  /** The effective plan snapshot the server would compute for an account. */
  const snapshotOf = (uid: string) => {
    const rows = activeRows(uid);
    const later = rows.reduce<Record<string, unknown> | null>((best, r) => {
      if (!best) return r;
      if (!r.endsAt) return r; // open-ended wins
      if (!best.endsAt) return best;
      return (r.endsAt as string) > (best.endsAt as string) ? r : best;
    }, null);
    const trial = trialRow(uid);
    const memberships = planOf(uid).memberships.filter((m) => !m.revokedAt);
    return {
      plan: later ? "premium" : "free",
      source: later?.source,
      endsAt: later?.endsAt,
      endsWithoutRenewal: !!later && later.source !== "apple" && later.source !== "google",
      trialCampaignKey: trial?.campaignId,
      trialCampaignName: trial ? campaignById(trial.campaignId as string)?.name : undefined,
      trialEndsAt: trial?.endsAt,
      betas: memberships.map((m) => ({
        campaignKey: m.campaignKey,
        campaignName: m.campaignName ?? campaignByKey(m.campaignKey as string)?.name ?? "",
        kind: m.kind,
        joinedAt: m.enrolledAt,
        endsAt: m.endsAt,
      })),
      managedOn: 0,
      canPurchaseHere: false,
      purchaseChannel: 0,
      products: [],
      unlocks: [],
    };
  };
  const nowIso = () => new Date().toISOString();

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
        // A copy, like a real RPC deserializing a fresh message: callers rely on the
        // new identity to know the row changed (the seeded object is also the one the
        // list handed out, so returning it as-is would look like "nothing moved").
        return { ...(data.hit ?? hits.find((h) => h.id === req.catalogId) ?? hits[0] ?? {}) };
      },
      // Curatorial metadata edit (change: add-catalog-metadata-editing), reachable
      // from the detail page AND from review mode. Applied in place so a re-read
      // reflects it, like the server recomputing the row.
      updateCatalogScore: async (req: { scoreId: string } & Record<string, unknown>) => {
        failIfSet("updateCatalogScore");
        const { scoreId, ...edit } = req;
        const row = hits.find((h) => h.id === scoreId);
        if (row) Object.assign(row, edit);
        if (data.hit && data.hit.id === scoreId) Object.assign(data.hit, edit);
        return {};
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
      // Reward pricing (change: add-soundfont-reward-pricing). Admin-only server-side;
      // kept on the row so a test can assert the price that was sent.
      setSoundFontPricing: async (req: { id: string; pointCost: number; redeemable: boolean }) => {
        failIfSet("setSoundFontPricing");
        const f = soundfonts.find((s) => s.id === req.id);
        if (f) {
          f.pointCost = req.pointCost;
          f.redeemable = req.redeemable;
        }
        return {};
      },
      // Moderation decision (change: add-soundfont-moderation). A rejection may
      // carry the moderator's reason (change: add-soundfont-uploader-attribution),
      // kept on the row so a test can assert it was sent.
      setSoundFontModerationStatus: async (req: { id: string; status: string; reason?: string }) => {
        failIfSet("setSoundFontModerationStatus");
        const f = soundfonts.find((s) => s.id === req.id);
        if (f) {
          f.moderationStatus = req.status;
          f.reviewReason = req.status === "rejected" ? req.reason : undefined;
        }
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
      listAccounts: async (req: { query: string; limit: number; offset: number; ids?: string[] }) => {
        failIfSet("listAccounts");
        const q = (req.query ?? "").toLowerCase();
        // `ids` (pre-resolved by the plan service) narrows the directory like the server.
        const scoped = req.ids && req.ids.length > 0 ? byScope.filter((a) => req.ids!.includes(a.userId)) : byScope;
        const filtered = q
          ? scoped.filter(
              (a) => (a.handle ?? "").toLowerCase().includes(q) || (a.displayName ?? "").toLowerCase().includes(q),
            )
          : scoped;
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
    flags: {
      listFlagDefinitions: async () => {
        failIfSet("listFlagDefinitions");
        return {
          definitions: flags.map((f) => {
            const isBool = typeof f.value === "boolean";
            const wire = isBool
              ? { kind: { case: "boolValue", value: f.value as boolean } }
              : { kind: { case: "intValue", value: BigInt(f.value as number) } };
            return {
              key: f.key,
              app: f.app ?? "all",
              valueType: isBool ? "bool" : "int",
              defaultValue: wire,
              effectiveValue: wire,
              hasOverride: f.hasOverride ?? false,
              rolloutScope: f.rolloutScope ?? "global",
              sensitive: false,
              doc: "",
              editable: f.editable ?? true,
              updatedBy: "",
              updatedAt: "",
            };
          }),
        };
      },
      listFlagChanges: async () => ({ changes: [] }),
      setFlag: async (req: { key: string; enabled: boolean; rolloutScope?: string }) => {
        failIfSet("setFlag");
        const f = findFlag(req.key);
        if (f) {
          f.value = req.enabled;
          f.hasOverride = true;
          if (req.rolloutScope) f.rolloutScope = req.rolloutScope;
        }
        return {};
      },
      setConfig: async (req: {
        key: string;
        value: { kind?: { case: string; value: unknown } };
        rolloutScope?: string;
      }) => {
        failIfSet("setConfig");
        const f = findFlag(req.key);
        if (f && req.value.kind?.case === "intValue") {
          f.value = Number(req.value.kind.value as bigint);
          f.hasOverride = true;
          if (req.rolloutScope) f.rolloutScope = req.rolloutScope;
        }
        return {};
      },
      clearOverride: async (req: { key: string }) => {
        failIfSet("clearOverride");
        const f = findFlag(req.key);
        if (f) f.hasOverride = false;
        return {};
      },
    },
    // Plan console (change: add-premium-subscription). Music-admin only server-side;
    // here every mutation is applied to the seeded state so the re-lookup / re-list
    // the store performs reflects it.
    plans: {
      lookupAccountPlan: async (req: { userId: string; handle: string }) => {
        failIfSet("lookupAccountPlan");
        const uid = resolveUser(req);
        const p = planOf(uid);
        return {
          userId: uid,
          snapshot: snapshotOf(uid),
          rows: p.rows.map((r) => ({ ...r })),
          memberships: [...p.memberships],
          // The server builds this deep link from its own project id (D5); the
          // fake mirrors a configured aggregator so the link renders in e2e.
          aggregatorCustomerUrl: `https://app.revenuecat.com/customers/proj-e2e/${uid}`,
        };
      },
      getPlansForAccounts: async (req: { userIds: string[] }) => {
        failIfSet("getPlansForAccounts");
        return {
          badges: req.userIds.map((uid) => {
            const snap = snapshotOf(uid);
            return {
              userId: uid,
              plan: snap.plan,
              trial: !!snap.trialCampaignKey,
              endsAt: snap.endsAt,
              betaKeys: snap.betas.map((b) => b.campaignKey),
            };
          }),
        };
      },
      listAccountIdsByPlan: async (req: { plan: string; betaCampaignKey: string }) => {
        failIfSet("listAccountIdsByPlan");
        const ids = byScope
          .map((a) => a.userId)
          .filter((uid) => {
            const snap = snapshotOf(uid);
            const planOk =
              req.plan === "premium" ? snap.plan === "premium" : req.plan === "trial" ? !!snap.trialCampaignKey : true;
            const betaOk = req.betaCampaignKey ? snap.betas.some((b) => b.campaignKey === req.betaCampaignKey) : true;
            return planOk && betaOk;
          });
        return { userIds: ids };
      },
      grantPremium: async (req: { userId: string; handle: string; endsAt?: string; confirmOpenEnded: boolean }) => {
        failIfSet("grantPremium");
        if (!req.endsAt && !req.confirmOpenEnded) {
          throw new ConnectError("open-ended grant requires confirmation", Code.FailedPrecondition);
        }
        const uid = resolveUser(req);
        const row = {
          id: `e-${Date.now()}`,
          source: "admin",
          providerRef: "",
          startsAt: nowIso(),
          endsAt: req.endsAt,
          status: "active",
        };
        planOf(uid).rows.push(row);
        return { row };
      },
      revokeEntitlement: async (req: { entitlementId: string }) => {
        failIfSet("revokeEntitlement");
        for (const p of Object.values(plans)) {
          const r = p.rows.find((x) => x.id === req.entitlementId);
          if (r) {
            r.status = "revoked";
            r.revokedAt = nowIso();
          }
        }
        return {};
      },
      enrolHandle: async (req: { userId: string; handle: string; campaignKey: string }) => {
        failIfSet("enrolHandle");
        const uid = resolveUser(req);
        const c = campaignByKey(req.campaignKey);
        if (!c || c.closedAt) throw new ConnectError("campaign not open", Code.FailedPrecondition);
        const endsAt =
          c.kind === "premium_trial" && c.durationDays
            ? new Date(Date.now() + c.durationDays * 86_400_000).toISOString()
            : undefined;
        const membership = {
          campaignKey: c.key,
          campaignName: c.name,
          kind: c.kind,
          userId: uid,
          enrolledAt: nowIso(),
          endsAt,
          source: "admin",
        };
        planOf(uid).memberships.push(membership);
        if (c.kind === "premium_trial") {
          planOf(uid).rows.push({
            id: `e-${Date.now()}`,
            source: "admin",
            providerRef: "",
            campaignId: c.key,
            startsAt: nowIso(),
            endsAt,
            status: "active",
          });
        }
        return { membership };
      },
      revokeMembership: async (req: { userId: string; handle: string; campaignKey: string }) => {
        failIfSet("revokeMembership");
        const uid = resolveUser(req);
        const m = planOf(uid).memberships.find((x) => x.campaignKey === req.campaignKey && !x.revokedAt);
        if (m) m.revokedAt = nowIso();
        return {};
      },
      createCampaign: async (req: { key: string; name: string; kind: string; durationDays?: number }) => {
        failIfSet("createCampaign");
        if (campaignByKey(req.key)) throw new ConnectError("key taken", Code.AlreadyExists);
        const c: E2ECampaign = {
          key: req.key,
          name: req.name,
          kind: req.kind as E2ECampaign["kind"],
          durationDays: req.durationDays,
          acceptsEnrolment: true,
        };
        campaigns.push(c);
        return { campaign: { ...c, id: c.key, createdBy: "u1", createdAt: nowIso() } };
      },
      listCampaigns: async (req: { includeClosed: boolean }) => {
        failIfSet("listCampaigns");
        return {
          campaigns: campaigns
            .filter((c) => req.includeClosed || !c.closedAt)
            .map((c) => ({ ...c, id: c.key, createdBy: "u1", createdAt: "2026-01-01T00:00:00Z" })),
        };
      },
      closeEnrollment: async (req: { campaignKey: string }) => {
        failIfSet("closeEnrollment");
        const c = campaignByKey(req.campaignKey);
        if (c) {
          c.acceptsEnrolment = false;
          c.enrollmentClosesAt = nowIso();
        }
        return {};
      },
      closeCampaign: async (req: { campaignKey: string }) => {
        failIfSet("closeCampaign");
        const c = campaignByKey(req.campaignKey);
        if (c) {
          c.acceptsEnrolment = false;
          c.closedAt = nowIso();
          for (const p of Object.values(plans)) {
            for (const m of p.memberships) if (m.campaignKey === c.key && !m.revokedAt) m.revokedAt = c.closedAt;
          }
        }
        return {};
      },
      mintCodes: async (req: { campaignKey: string; count: number }) => {
        failIfSet("mintCodes");
        const pool = data.mintedCodes ?? ["E2E-CODE-1", "E2E-CODE-2", "E2E-CODE-3"];
        return { codes: pool.slice(0, req.count) };
      },
      revokeCodes: async () => {
        failIfSet("revokeCodes");
        return { revoked: 0 };
      },
      listMembers: async (req: { campaignKey: string }) => {
        failIfSet("listMembers");
        const members: Record<string, unknown>[] = [];
        for (const [uid, p] of Object.entries(plans)) {
          for (const m of p.memberships) if (m.campaignKey === req.campaignKey) members.push({ ...m, userId: uid });
        }
        return { members };
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
  // add-soundfont-entitlement-previews). A successful regeneration flips the font's
  // `has_preview` so the next admin-list turns its "Generate sample" slot into a play
  // button (mirroring the server storing the preview object).
  setRegeneratePreviewForTest(async (id) => {
    if (data.fail?.regeneratePreview) {
      const f = data.fail.regeneratePreview;
      throw new ConnectError(f.message, f.code as Code);
    }
    const f = soundfonts.find((s) => s.id === id);
    if (f) f.hasPreview = true;
  });

  // The score audio teaser's "Generate sample" is an HTTP route too (change:
  // add-score-daily-access-rewards). A successful regeneration flips the piece's
  // `hasPreview` (mirroring the server stamping the row's marker) so the detail view's
  // slot becomes a play button.
  setRegenerateScorePreviewForTest(async (id) => {
    if (data.fail?.regenerateScorePreview) {
      const f = data.fail.regenerateScorePreview;
      throw new ConnectError(f.message, f.code as Code);
    }
    const h = hits.find((x) => x.id === id);
    if (h) h.hasPreview = true;
    if (data.hit && data.hit.id === id) data.hit.hasPreview = true;
  });

  setClientsForTest(clients);
  setWebAuthClientForTest(webAuthClient);
}
