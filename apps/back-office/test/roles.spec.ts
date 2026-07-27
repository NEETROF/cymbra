import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useRolesStore } from "@/stores/roles";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";

describe("roles store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("grants a role (music scope by default) then refreshes the audit", async () => {
    const grants = [
      { targetUserId: "t", scope: "music", role: "moderator", action: "grant", actingAdmin: "a", at: 1n },
    ];
    const { clients, state } = makeFakeClients({ grants });
    setClientsForTest(clients);
    const store = useRolesStore();

    await store.grant("t", "moderator");

    expect(state.grantCalls).toEqual([{ userId: "t", scope: "music", role: "moderator" }]);
    expect(store.op.status).toBe("success");
    // The audit history was reloaded and surfaced.
    expect(store.grants.status).toBe("success");
    if (store.grants.status === "success") {
      expect(store.grants.data).toHaveLength(1);
      expect((store.grants.data[0] as { action: string }).action).toBe("grant");
    }
  });

  it("revokes a role", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    const store = useRolesStore();
    await store.revoke("t", "moderator");
    expect(state.revokeCalls).toEqual([{ userId: "t", scope: "music", role: "moderator" }]);
  });

  it("captures a denied grant in the op state instead of throwing", async () => {
    const { clients } = makeFakeClients();
    (clients.user as unknown as { grantRole: () => Promise<never> }).grantRole = () =>
      Promise.reject(new Error("permission denied"));
    setClientsForTest(clients);
    const store = useRolesStore();
    // The union captures the failure — the action does not throw.
    const outcome = await store.grant("t", "admin");
    expect(outcome.status).toBe("error");
    expect(store.op.status).toBe("error");
    // A user-facing message, not the raw error text.
    if (store.op.status === "error") {
      expect(store.op.error).not.toContain("permission denied");
      expect(store.op.error.length).toBeGreaterThan(0);
    }
  });
});
