import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { PAGE_SIZE, useRolesStore } from "@/stores/roles";
import { useAuthStore } from "@/stores/auth";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients, makeJwt } from "./fakes";

describe("roles store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("lists a page of the account directory", async () => {
    const accounts = [{ userId: "t", handle: "ada", displayName: "Ada", roles: ["moderator"] }];
    const { clients, state } = makeFakeClients({ accounts });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.list("ada", 0);

    expect(state.listAccountsCalls).toEqual([{ query: "ada", limit: PAGE_SIZE, offset: 0, ids: [] }]);
    expect(store.directory.status).toBe("success");
    if (store.directory.status === "success") {
      expect(store.directory.data.total).toBe(1);
      expect(store.directory.data.accounts).toHaveLength(1);
    }
  });

  it("grants a role in the given scope then re-lists the current page", async () => {
    const accounts = [{ userId: "t", handle: "ada", rolesByScope: [{ scope: "music", roles: [] }] }];
    const { clients, state } = makeFakeClients({ accounts });
    setClientsForTest(clients);
    const store = useRolesStore();
    await store.list("", 0); // sets the current page

    await store.grant("t", "moderator", "music");

    expect(state.grantCalls).toEqual([{ userId: "t", scope: "music", role: "moderator" }]);
    expect(store.op.status).toBe("success");
    // The directory was reloaded so the row reflects the change (2 list calls total).
    expect(state.listAccountsCalls).toHaveLength(2);
    expect(store.directory.status).toBe("success");
  });

  it("revokes a role", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    const store = useRolesStore();
    await store.revoke("t", "moderator", "music");
    expect(state.revokeCalls).toEqual([{ userId: "t", scope: "music", role: "moderator" }]);
  });

  it("loads a per-account audit history", async () => {
    const grants = [
      { targetUserId: "t", scope: "music", role: "moderator", action: "grant", actingAdmin: "a", at: 1n },
    ];
    const { clients } = makeFakeClients({ grants });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.listGrants("t");

    expect(store.grants.status).toBe("success");
    if (store.grants.status === "success") {
      expect(store.grants.data).toHaveLength(1);
      expect((store.grants.data[0] as { action: string }).action).toBe("grant");
    }
  });

  it("loads a user's read-only curator reliability", async () => {
    const reliability = {
      totalRatings: 57n,
      coverageContribution: 42n,
      alignmentRate: 0.82,
      settledCount: 40n,
      alignedCount: 33n,
    };
    const { clients, state } = makeFakeClients({ reliability });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.loadReliability("t");

    expect(state.reliabilityCalls).toEqual(["t"]);
    expect(store.reliability.status).toBe("success");
    if (store.reliability.status === "success") {
      expect(store.reliability.data.totalRatings).toBe(57n);
      expect(store.reliability.data.alignmentRate).toBeCloseTo(0.82);
    }
  });

  it("captures a denied reliability read in the union instead of throwing", async () => {
    const { clients } = makeFakeClients();
    (clients.score as unknown as { getCuratorReliability: () => Promise<never> }).getCuratorReliability = () =>
      Promise.reject(new Error("permission denied"));
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.loadReliability("t");

    expect(store.reliability.status).toBe("error");
    if (store.reliability.status === "error") {
      expect(store.reliability.error).not.toContain("permission denied");
      expect(store.reliability.error.length).toBeGreaterThan(0);
    }
  });

  // --- plan / beta criteria (change: add-premium-subscription) ---

  it("a plan filter is pre-resolved into ids by the plan service and passed to listAccounts", async () => {
    const accounts = [{ userId: "u1", handle: "ada", rolesByScope: [] }];
    const { clients, state } = makeFakeClients({ accounts, idsByPlan: ["u1", "u9"] });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.list("", 0, "trial", "");

    expect(state.idsByPlanCalls).toEqual([{ plan: "trial", betaCampaignKey: "" }]);
    expect(state.listAccountsCalls).toEqual([{ query: "", limit: PAGE_SIZE, offset: 0, ids: ["u1", "u9"] }]);
    expect(store.params).toMatchObject({ plan: "trial", beta: "" });
  });

  it("a beta filter with an empty resolved set is an empty page without calling listAccounts", async () => {
    const { clients, state } = makeFakeClients({ idsByPlan: [] });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.list("", 0, "any", "midi-drums");

    expect(state.idsByPlanCalls).toEqual([{ plan: "any", betaCampaignKey: "midi-drums" }]);
    expect(state.listAccountsCalls).toEqual([]);
    expect(store.directory).toEqual({ status: "success", data: { accounts: [], total: 0 } });
  });

  it("no filter ⇒ the plan service is not consulted; the page's badges are batched for a music admin only", async () => {
    const accounts = [{ userId: "u1", handle: "ada", rolesByScope: [] }];
    const { clients, state } = makeFakeClients({ accounts });
    setClientsForTest(clients);
    const store = useRolesStore();

    // Signed in as a non-music admin (roles are `global: user`) ⇒ no badge batch.
    await store.list("", 0);
    expect(state.idsByPlanCalls).toEqual([]);
    expect(state.plansForAccountsCalls).toEqual([]);

    // A music-scope admin gets the batch, one call for the page.
    useAuthStore().setToken(
      makeJwt({ sub: "a", roles: ["admin"], roles_by_scope: { music: ["admin"] }, exp: 4102444800 }),
    );
    await store.list("", 0);
    expect(state.plansForAccountsCalls).toEqual([["u1"]]);
  });

  // --- one account, for the detail page (change: restructure-back-office-users-console) ---

  it("loads a single account by id, so the detail page needs no directory page", async () => {
    const accounts = [
      { userId: "u1", handle: "ada", rolesByScope: [] },
      { userId: "u2", handle: "bob", rolesByScope: [] },
    ];
    const { clients, state } = makeFakeClients({ accounts });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.loadAccount("u2");

    // The directory's `ids` filter carries it — no new RPC, and no page of 25 to sift.
    expect(state.listAccountsCalls).toEqual([{ query: "", limit: 1, offset: 0, ids: ["u2"] }]);
    expect(store.account.status).toBe("success");
    if (store.account.status === "success") expect(store.account.data?.handle).toBe("bob");
  });

  it("an id that matches nothing is a renderable `null`, not an error", async () => {
    const { clients } = makeFakeClients({ accounts: [{ userId: "u1", handle: "ada", rolesByScope: [] }] });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.loadAccount("nobody");

    expect(store.account).toEqual({ status: "success", data: null });
  });

  it("switching accounts drops every per-account slot before the next load", async () => {
    const accounts = [
      { userId: "u1", handle: "ada", rolesByScope: [] },
      { userId: "u2", handle: "bob", rolesByScope: [] },
    ];
    const grants = [
      { targetUserId: "u1", scope: "music", role: "moderator", action: "grant", actingAdmin: "a", at: 1n },
    ];
    const { clients } = makeFakeClients({ accounts, grants, reliability: { totalRatings: 3n } });
    setClientsForTest(clients);
    const store = useRolesStore();
    await store.loadAccount("u1");
    await store.listGrants("u1");
    await store.loadReliability("u1");

    store.resetAccount();

    // Nothing of account u1 survives: a frame of one account's rights under another
    // account's name is unacceptable on a page where rights are revoked.
    expect(store.account.status).toBe("idle");
    expect(store.grants.status).toBe("idle");
    expect(store.reliability.status).toBe("idle");
  });

  it("a role change made on a detail page re-reads that account, not a directory page", async () => {
    const accounts = [{ userId: "u1", handle: "ada", rolesByScope: [{ scope: "music", roles: [] }] }];
    const { clients, state } = makeFakeClients({ accounts });
    setClientsForTest(clients);
    const store = useRolesStore();
    await store.loadAccount("u1");

    await store.grant("u1", "moderator", "music", "account");

    expect(state.grantCalls).toEqual([{ userId: "u1", scope: "music", role: "moderator" }]);
    // Two single-account reads (the initial load + the refresh), no directory page.
    expect(state.listAccountsCalls).toEqual([
      { query: "", limit: 1, offset: 0, ids: ["u1"] },
      { query: "", limit: 1, offset: 0, ids: ["u1"] },
    ]);
  });

  it("a role change on a detail page re-reads the audit history too, not only the roles", async () => {
    // The detail page shows the roles AND the history side by side, and a grant writes
    // to both. Re-reading only the roles left the history a page-refresh behind the
    // action the operator had just taken, on the same screen.
    const accounts = [{ userId: "u1", handle: "ada", rolesByScope: [{ scope: "music", roles: [] }] }];
    const { clients, state } = makeFakeClients({ accounts });
    setClientsForTest(clients);
    const store = useRolesStore();
    await store.loadAccount("u1");
    await store.listGrants("u1");
    expect(state.listRoleGrantsCalls).toEqual(["u1"]);

    await store.grant("u1", "moderator", "music", "account");

    expect(state.listRoleGrantsCalls).toEqual(["u1", "u1"]);

    await store.revoke("u1", "moderator", "music", "account");

    expect(state.listRoleGrantsCalls).toEqual(["u1", "u1", "u1"]);
  });

  it("a directory-page role change does NOT read a per-account history", async () => {
    const { clients, state } = makeFakeClients({ accounts: [{ userId: "u1", rolesByScope: [] }] });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.grant("u1", "moderator", "music");

    expect(state.listRoleGrantsCalls).toEqual([]);
  });

  it("captures a denied grant in the op state instead of throwing", async () => {
    const { clients } = makeFakeClients();
    (clients.user as unknown as { grantRole: () => Promise<never> }).grantRole = () =>
      Promise.reject(new Error("permission denied"));
    setClientsForTest(clients);
    const store = useRolesStore();
    // The union captures the failure — the action does not throw.
    const outcome = await store.grant("t", "admin", "music");
    expect(outcome.status).toBe("error");
    expect(store.op.status).toBe("error");
    // A user-facing message, not the raw error text.
    if (store.op.status === "error") {
      expect(store.op.error).not.toContain("permission denied");
      expect(store.op.error.length).toBeGreaterThan(0);
    }
  });
});
