import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run, success } from "@/lib/async";
import type { AccountPlanBadge, CampaignMsg, LookupAccountPlanResponse, MembershipMsg } from "@/gen/plans_pb";

/** The directory's plan criterion, as the server names it (`ListAccountIdsByPlan.plan`). */
export type PlanFilter = "any" | "premium" | "trial";
export type CampaignKind = "premium_trial" | "feature";

/** A target account: by id when known (directory rows), else by handle (console lookup). */
export type AccountRef = { userId: string; handle?: undefined } | { handle: string; userId?: undefined };

/** Split a free-text lookup into the RPC's `userId`/`handle` pair — a UUID is an id,
 *  anything else is a handle (a leading `@` is tolerated). */
export function accountRef(handleOrId: string): AccountRef {
  const s = handleOrId.trim().replace(/^@/, "");
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(s) ? { userId: s } : { handle: s };
}
/** The wire pair for a target (the RPCs take both fields; exactly one is set). */
const wireRef = (r: AccountRef) => ({ userId: r.userId ?? "", handle: r.handle ?? "" });

/** The store/web sources end on the provider's side — only `code`/`admin` rows are
 *  revocable from the console. */
export const REVOCABLE_SOURCES = ["code", "admin"] as const;
export const isRevocableSource = (source: string): boolean => (REVOCABLE_SOURCES as readonly string[]).includes(source);

/** The store aggregator's customer page for an account (change:
 *  swap-store-billing-to-revenuecat, D5) — where transactions, renewals, refunds
 *  and revenue live. `undefined` when the project id is not configured or there
 *  is no account. */
export function revenueCatCustomerUrl(project: string | undefined, userId: string | undefined): string | undefined {
  if (!project || !userId) return undefined;
  return `https://app.revenuecat.com/customers/${encodeURIComponent(project)}/${encodeURIComponent(userId)}`;
}

/** CSV of a campaign's members (handle when the row carries one, else user id) —
 *  the export the console offers per campaign. Fields are quoted; RFC 4180. */
export function membersCsv(members: MembershipMsg[], handleFor: (userId: string) => string | undefined): string {
  const q = (v: string | undefined) => `"${(v ?? "").replaceAll('"', '""')}"`;
  const head = ["account", "user_id", "campaign", "kind", "source", "enrolled_at", "ends_at", "revoked_at"];
  const rows = members.map((m) =>
    [handleFor(m.userId) ?? m.userId, m.userId, m.campaignKey, m.kind, m.source, m.enrolledAt, m.endsAt, m.revokedAt]
      .map(q)
      .join(","),
  );
  return [head.join(","), ...rows].join("\r\n") + "\r\n";
}

// Music-admin plan console (change: add-premium-subscription). Every RPC is behind the
// injectable client seam and every resource is ONE `Async` union — a refused grant
// (e.g. open-ended without confirmation) lands in `op` as `{ status: "error" }`, never a
// throw. After a mutation that changes an account, the last lookup is re-run so the
// console shows the server's recomputed effective plan.
export const usePlansStore = defineStore("plans", () => {
  const lookupResult = ref<Async<LookupAccountPlanResponse>>(idle);
  const campaigns = ref<Async<CampaignMsg[]>>(idle);
  const members = ref<Async<MembershipMsg[]>>(idle);
  /** Clear-text codes of the LAST mint — shown once; the view clears it when dismissed. */
  const minted = ref<Async<string[]>>(idle);
  /** Batch plan/beta badges for a directory page, keyed by user id. */
  const badges = ref<Async<Record<string, AccountPlanBadge>>>(idle);
  const op = ref<Async<void>>(idle);
  // The last successful lookup query, re-run after an account-changing mutation.
  const lastLookup = ref<string | null>(null);
  const membersKey = ref<string | null>(null);

  /** Campaigns still accepting members or at least not closed (the flags console's
   *  beta selector and the directory's beta filter list exactly these). */
  const openCampaigns = computed<CampaignMsg[]>(() =>
    campaigns.value.status === "success" ? campaigns.value.data.filter((c) => !c.closedAt) : [],
  );

  async function lookup(handleOrId: string) {
    lastLookup.value = handleOrId;
    await run(lookupResult, () => api().plans.lookupAccountPlan(wireRef(accountRef(handleOrId))));
  }

  function clearLookup() {
    lastLookup.value = null;
    lookupResult.value = idle;
  }

  async function loadCampaigns(includeClosed = true) {
    await run(campaigns, async () => (await api().plans.listCampaigns({ includeClosed })).campaigns);
  }

  async function loadMembers(campaignKey: string) {
    membersKey.value = campaignKey;
    await run(members, async () => (await api().plans.listMembers({ campaignKey })).members);
  }

  /** Fold a mutation into `op`; on success run `after` (re-list what it changed). */
  async function mutate(fn: () => Promise<unknown>, after?: () => Promise<void>) {
    const outcome = await run(op, async () => {
      await fn();
    });
    if (outcome.status === "success" && after) await after();
    return outcome;
  }
  const relookup = async () => {
    if (lastLookup.value) await lookup(lastLookup.value);
  };
  const reloadCampaigns = () => loadCampaigns();
  const reloadMembers = async () => {
    if (membersKey.value) await loadMembers(membersKey.value);
  };

  function grantPremium(p: { target: AccountRef; endsAt?: string; confirmOpenEnded: boolean; reason: string }) {
    return mutate(
      () =>
        api().plans.grantPremium({
          ...wireRef(p.target),
          endsAt: p.endsAt,
          confirmOpenEnded: p.confirmOpenEnded,
          reason: p.reason,
        }),
      relookup,
    );
  }

  function revokeEntitlement(entitlementId: string, reason: string) {
    return mutate(() => api().plans.revokeEntitlement({ entitlementId, reason }), relookup);
  }

  function enrolHandle(p: { target: AccountRef; campaignKey: string; reason: string }) {
    return mutate(
      () => api().plans.enrolHandle({ ...wireRef(p.target), campaignKey: p.campaignKey, reason: p.reason }),
      relookup,
    );
  }

  function revokeMembership(p: { target: AccountRef; campaignKey: string; reason: string }) {
    return mutate(
      () => api().plans.revokeMembership({ ...wireRef(p.target), campaignKey: p.campaignKey, reason: p.reason }),
      async () => {
        await relookup();
        await reloadMembers();
      },
    );
  }

  function createCampaign(p: { key: string; name: string; kind: CampaignKind; durationDays?: number }) {
    return mutate(
      () =>
        api().plans.createCampaign({
          key: p.key,
          name: p.name,
          kind: p.kind,
          durationDays: p.kind === "premium_trial" ? p.durationDays : undefined,
        }),
      reloadCampaigns,
    );
  }

  function closeEnrollment(campaignKey: string) {
    return mutate(() => api().plans.closeEnrollment({ campaignKey }), reloadCampaigns);
  }

  function closeCampaign(campaignKey: string) {
    return mutate(
      () => api().plans.closeCampaign({ campaignKey }),
      async () => {
        await reloadCampaigns();
        await reloadMembers();
      },
    );
  }

  /** Reopen a closed campaign. Closing is a PAUSE — it touches no membership —
   *  so this brings back every member that was not individually revoked; the
   *  RPC answers how many, and the caller states it. Enrolment stays closed:
   *  that is [reopenEnrollment]'s job. */
  function reopenCampaign(campaignKey: string) {
    return mutate(
      () => api().plans.reopenCampaign({ campaignKey }),
      async () => {
        await reloadCampaigns();
        await reloadMembers();
      },
    );
  }

  /** Reopen a campaign's enrolment, so its codes are redeemable again. */
  function reopenEnrollment(campaignKey: string) {
    return mutate(() => api().plans.reopenEnrollment({ campaignKey }), reloadCampaigns);
  }

  /** How many memberships a reopening would restore — read BEFORE reopening, so
   *  the confirmation says what the action will do rather than what it did. */
  async function reactivatableMembers(campaignKey: string): Promise<number> {
    return (await api().plans.previewReopenCampaign({ campaignKey })).reactivated;
  }

  /** Mint N codes; the clear text lands in `minted` ONCE (never re-fetchable). */
  async function mintCodes(campaignKey: string, count: number, issuedToHint = "") {
    return run(minted, async () => (await api().plans.mintCodes({ campaignKey, count, issuedToHint })).codes);
  }
  function clearMinted() {
    minted.value = idle;
  }

  function revokeCodes(p: { campaignKey: string } | { codeIds: string[] }) {
    return mutate(() =>
      api().plans.revokeCodes({
        campaignKey: "campaignKey" in p ? p.campaignKey : "",
        codeIds: "codeIds" in p ? p.codeIds : [],
      }),
    );
  }

  /** Resolve a plan × beta criterion into account ids (the directory then lists them).
   *  Throws on failure so the caller's `run(...)` folds it into ITS union. */
  async function accountIdsByPlan(plan: PlanFilter, betaCampaignKey = ""): Promise<string[]> {
    return (await api().plans.listAccountIdsByPlan({ plan, betaCampaignKey })).userIds;
  }

  /** Batch badges for the displayed directory page (one call per page). */
  async function plansForAccounts(userIds: string[]) {
    if (userIds.length === 0) {
      badges.value = success({});
      return;
    }
    await run(badges, async () => {
      const resp = await api().plans.getPlansForAccounts({ userIds });
      return Object.fromEntries(resp.badges.map((b) => [b.userId, b]));
    });
  }

  return {
    lookupResult,
    lastLookup,
    campaigns,
    openCampaigns,
    members,
    membersKey,
    minted,
    badges,
    op,
    lookup,
    clearLookup,
    loadCampaigns,
    loadMembers,
    grantPremium,
    revokeEntitlement,
    enrolHandle,
    revokeMembership,
    createCampaign,
    closeEnrollment,
    closeCampaign,
    reopenCampaign,
    reopenEnrollment,
    reactivatableMembers,
    mintCodes,
    clearMinted,
    revokeCodes,
    accountIdsByPlan,
    plansForAccounts,
  };
});
