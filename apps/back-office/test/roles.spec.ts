import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { PAGE_SIZE, useRolesStore } from "@/stores/roles";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";

describe("roles store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("lists a page of the account directory", async () => {
    const accounts = [{ userId: "t", handle: "ada", displayName: "Ada", roles: ["moderator"] }];
    const { clients, state } = makeFakeClients({ accounts });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.list("ada", 0);

    expect(state.listAccountsCalls).toEqual([{ query: "ada", limit: PAGE_SIZE, offset: 0 }]);
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
