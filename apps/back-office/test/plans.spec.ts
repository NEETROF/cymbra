import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import { accountRef, isRevocableSource, membersCsv, revenueCatCustomerUrl, usePlansStore } from "@/stores/plans";
import { makeFakeClients } from "./fakes";

const UUID = "019f60be-6cd9-74e2-a600-9893bd2aaa3a";

const trialCampaign = {
  id: "c1",
  key: "spring-trial",
  name: "Spring trial",
  kind: "premium_trial",
  durationDays: 90,
  acceptsEnrolment: true,
  closedAt: undefined,
};
const featureCampaign = { id: "c2", key: "midi-drums", name: "MIDI drums", kind: "feature", acceptsEnrolment: true };
const closedCampaign = {
  id: "c3",
  key: "old-beta",
  name: "Old beta",
  kind: "feature",
  closedAt: "2026-01-01T00:00:00Z",
};

const lookup = {
  userId: UUID,
  snapshot: {
    plan: "premium",
    source: "code",
    endsAt: "2026-11-01T00:00:00Z",
    trialCampaignKey: "spring-trial",
    betas: [],
  },
  rows: [{ id: "e1", source: "code", providerRef: "", startsAt: "2026-08-01T00:00:00Z", status: "active" }],
  memberships: [],
};

describe("plans helpers", () => {
  it("splits a lookup into a user id (UUID) or a handle (anything else, @ tolerated)", () => {
    expect(accountRef(UUID)).toEqual({ userId: UUID });
    expect(accountRef(" @ada ")).toEqual({ handle: "ada" });
    expect(accountRef("ada")).toEqual({ handle: "ada" });
  });

  it("only code/admin rows are revocable from the console (store rows are read-only)", () => {
    expect(isRevocableSource("code")).toBe(true);
    expect(isRevocableSource("admin")).toBe(true);
    for (const s of ["apple", "google", "web"]) expect(isRevocableSource(s)).toBe(false);
  });

  it("links an account to its aggregator customer page only when the project id is configured", () => {
    expect(revenueCatCustomerUrl(undefined, "u1")).toBeUndefined();
    expect(revenueCatCustomerUrl("proj1", undefined)).toBeUndefined();
    expect(revenueCatCustomerUrl("proj1", "018f0000-0000-7000-8000-000000000001")).toBe(
      "https://app.revenuecat.com/customers/proj1/018f0000-0000-7000-8000-000000000001",
    );
    // ids are path segments: escaped, never trusted
    expect(revenueCatCustomerUrl("p/q", "a b")).toBe("https://app.revenuecat.com/customers/p%2Fq/a%20b");
  });

  it("exports members as CSV with the handle when known, else the user id", () => {
    const csv = membersCsv(
      [
        {
          userId: "u1",
          campaignKey: "k",
          kind: "feature",
          source: "code",
          enrolledAt: "2026-08-01",
          endsAt: undefined,
        },
        {
          userId: "u2",
          campaignKey: "k",
          kind: "feature",
          source: "admin",
          enrolledAt: "2026-08-02",
          endsAt: "2026-09-01",
        },
      ] as never,
      (id) => (id === "u1" ? 'ada "the" first' : undefined),
    );
    const lines = csv.trim().split("\r\n");
    expect(lines[0]).toBe("account,user_id,campaign,kind,source,enrolled_at,ends_at,revoked_at");
    expect(lines[1]).toBe('"ada ""the"" first","u1","k","feature","code","2026-08-01","",""');
    expect(lines[2]).toBe('"u2","u2","k","feature","admin","2026-08-02","2026-09-01",""');
  });
});

describe("plans store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("looks up an account by handle and holds the response in the union", async () => {
    const { clients, state } = makeFakeClients({ lookup });
    setClientsForTest(clients);
    const store = usePlansStore();

    await store.lookup("ada");

    expect(state.lookupCalls).toEqual([{ userId: "", handle: "ada" }]);
    expect(store.lookupResult.status).toBe("success");
    if (store.lookupResult.status === "success") {
      expect(store.lookupResult.data.snapshot?.plan).toBe("premium");
      expect(store.lookupResult.data.rows).toHaveLength(1);
    }
  });

  it("looks up by id when given a UUID", async () => {
    const { clients, state } = makeFakeClients({ lookup });
    setClientsForTest(clients);
    await usePlansStore().lookup(UUID);
    expect(state.lookupCalls).toEqual([{ userId: UUID, handle: "" }]);
  });

  it("captures a denied lookup in the union with a humanized message", async () => {
    const { clients } = makeFakeClients();
    (clients.plans as unknown as { lookupAccountPlan: () => Promise<never> }).lookupAccountPlan = () =>
      Promise.reject(new ConnectError("music admin only", Code.PermissionDenied));
    setClientsForTest(clients);
    const store = usePlansStore();

    await store.lookup("ada");

    expect(store.lookupResult.status).toBe("error");
    if (store.lookupResult.status === "error") {
      expect(store.lookupResult.error).toBe("Access denied.");
      expect(store.lookupResult.error).not.toContain("music admin only");
    }
  });

  it("grants premium (bounded) then re-runs the last lookup", async () => {
    const { clients, state } = makeFakeClients({ lookup });
    setClientsForTest(clients);
    const store = usePlansStore();
    await store.lookup("ada");

    const outcome = await store.grantPremium({
      target: { userId: UUID },
      endsAt: "2026-12-31T23:59:59.000Z",
      confirmOpenEnded: false,
      reason: "thanks",
    });

    expect(outcome.status).toBe("success");
    expect(state.grantPremiumCalls).toEqual([
      { userId: UUID, handle: "", endsAt: "2026-12-31T23:59:59.000Z", confirmOpenEnded: false, reason: "thanks" },
    ]);
    // The lookup was re-run so the console shows the recomputed plan (2 calls).
    expect(state.lookupCalls).toHaveLength(2);
  });

  it("an open-ended grant refused by the server lands in `op` as an error, never a throw", async () => {
    const { clients, state } = makeFakeClients({ lookup });
    (clients.plans as unknown as { grantPremium: () => Promise<never> }).grantPremium = () =>
      Promise.reject(new ConnectError("open-ended grant needs confirm", Code.FailedPrecondition));
    setClientsForTest(clients);
    const store = usePlansStore();
    await store.lookup("ada");

    const outcome = await store.grantPremium({ target: { handle: "ada" }, confirmOpenEnded: false, reason: "r" });

    expect(outcome.status).toBe("error");
    expect(store.op.status).toBe("error");
    if (store.op.status === "error") {
      expect(store.op.error).toBe("Not available yet. Try again later.");
      expect(store.op.error).not.toContain("open-ended");
    }
    // No re-lookup after a failed mutation.
    expect(state.lookupCalls).toHaveLength(1);
  });

  it("revokes an entitlement / enrols / revokes a membership with a reason and re-looks up", async () => {
    const { clients, state } = makeFakeClients({ lookup });
    setClientsForTest(clients);
    const store = usePlansStore();
    await store.lookup("ada");

    await store.revokeEntitlement("e1", "abuse");
    await store.enrolHandle({ target: { handle: "ada" }, campaignKey: "midi-drums", reason: "tester" });
    await store.revokeMembership({ target: { userId: UUID }, campaignKey: "midi-drums", reason: "done" });

    expect(state.revokeEntitlementCalls).toEqual([{ entitlementId: "e1", reason: "abuse" }]);
    expect(state.enrolCalls).toEqual([{ userId: "", handle: "ada", campaignKey: "midi-drums", reason: "tester" }]);
    expect(state.revokeMembershipCalls).toEqual([
      { userId: UUID, handle: "", campaignKey: "midi-drums", reason: "done" },
    ]);
    expect(state.lookupCalls).toHaveLength(4);
  });

  it("lists campaigns and derives the open ones (closed campaigns excluded)", async () => {
    const { clients } = makeFakeClients({ campaigns: [trialCampaign, featureCampaign, closedCampaign] });
    setClientsForTest(clients);
    const store = usePlansStore();

    await store.loadCampaigns();

    expect(store.campaigns.status).toBe("success");
    expect(store.openCampaigns.map((c) => c.key)).toEqual(["spring-trial", "midi-drums"]);
  });

  it("creates a campaign (duration only for trials), closes enrolment / a campaign, then re-lists", async () => {
    const { clients, state } = makeFakeClients({ campaigns: [featureCampaign] });
    setClientsForTest(clients);
    const store = usePlansStore();

    await store.createCampaign({ key: "t", name: "T", kind: "premium_trial", durationDays: 30 });
    await store.createCampaign({ key: "f", name: "F", kind: "feature", durationDays: 30 });
    await store.closeEnrollment("midi-drums");
    await store.closeCampaign("midi-drums");

    expect(state.createCampaignCalls).toEqual([
      { key: "t", name: "T", kind: "premium_trial", durationDays: 30 },
      { key: "f", name: "F", kind: "feature", durationDays: undefined },
    ]);
    expect(state.closeEnrollmentCalls).toEqual(["midi-drums"]);
    expect(state.closeCampaignCalls).toEqual(["midi-drums"]);
    expect(store.campaigns.status).toBe("success");
  });

  it("reopens a closed campaign and its enrolment as two separate acts", async () => {
    // Closing is a pause — it touches no membership — so reopening restores
    // people rather than re-enrolling them, and the count comes from the
    // server: the "active membership" rule must not be copied into the console.
    const { clients, state } = makeFakeClients({ campaigns: [featureCampaign], reactivatable: 12 });
    setClientsForTest(clients);
    const store = usePlansStore();

    expect(await store.reactivatableMembers("midi-drums")).toBe(12);
    await store.reopenCampaign("midi-drums");
    expect(state.reopenCampaignCalls).toEqual(["midi-drums"]);
    // Reopening the campaign leaves enrolment alone: closing a campaign closed
    // it as a side effect, and reopening it is its own decision.
    expect(state.reopenEnrollmentCalls).toEqual([]);

    await store.reopenEnrollment("midi-drums");
    expect(state.reopenEnrollmentCalls).toEqual(["midi-drums"]);
  });

  it("mints codes: the clear text lands in `minted` once and can be cleared", async () => {
    const { clients, state } = makeFakeClients({ mintedCodes: ["AAAA-1111", "BBBB-2222", "CCCC-3333"] });
    setClientsForTest(clients);
    const store = usePlansStore();

    const outcome = await store.mintCodes("midi-drums", 2, "press");

    expect(state.mintCalls).toEqual([{ campaignKey: "midi-drums", count: 2, issuedToHint: "press" }]);
    expect(outcome.status).toBe("success");
    expect(store.minted).toEqual({ status: "success", data: ["AAAA-1111", "BBBB-2222"] });
    store.clearMinted();
    expect(store.minted.status).toBe("idle");
  });

  it("revokes codes per campaign or by id", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    const store = usePlansStore();
    await store.revokeCodes({ campaignKey: "midi-drums" });
    await store.revokeCodes({ codeIds: ["c1", "c2"] });
    expect(state.revokeCodesCalls).toEqual([
      { campaignKey: "midi-drums", codeIds: [] },
      { campaignKey: "", codeIds: ["c1", "c2"] },
    ]);
  });

  it("lists a campaign's members", async () => {
    const members = [{ userId: "u1", campaignKey: "midi-drums", kind: "feature", enrolledAt: "x", source: "code" }];
    const { clients, state } = makeFakeClients({ members });
    setClientsForTest(clients);
    const store = usePlansStore();
    await store.loadMembers("midi-drums");
    expect(state.listMembersCalls).toEqual(["midi-drums"]);
    expect(store.membersKey).toBe("midi-drums");
    expect(store.members.status).toBe("success");
  });

  it("passes the plan × beta criterion through and returns the resolved ids", async () => {
    const { clients, state } = makeFakeClients({ idsByPlan: ["u1", "u3"] });
    setClientsForTest(clients);
    const store = usePlansStore();
    const ids = await store.accountIdsByPlan("trial", "midi-drums");
    expect(state.idsByPlanCalls).toEqual([{ plan: "trial", betaCampaignKey: "midi-drums" }]);
    expect(ids).toEqual(["u1", "u3"]);
  });

  it("batches the page's badges keyed by user id, and skips the call for an empty page", async () => {
    const badges = [
      { userId: "u1", plan: "premium", trial: true, betaKeys: ["midi-drums"] },
      { userId: "u2", plan: "free", trial: false, betaKeys: [] },
    ];
    const { clients, state } = makeFakeClients({ badges });
    setClientsForTest(clients);
    const store = usePlansStore();

    await store.plansForAccounts([]);
    expect(state.plansForAccountsCalls).toEqual([]);
    expect(store.badges).toEqual({ status: "success", data: {} });

    await store.plansForAccounts(["u1", "u2"]);
    expect(state.plansForAccountsCalls).toEqual([["u1", "u2"]]);
    if (store.badges.status === "success") {
      expect(store.badges.data.u1).toMatchObject({ plan: "premium", trial: true });
      expect(store.badges.data.u2).toMatchObject({ plan: "free" });
    } else throw new Error("expected success");
  });
});
