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
    // The audit history was reloaded and surfaced.
    expect(store.grants).toHaveLength(1);
    expect((store.grants[0] as { action: string }).action).toBe("grant");
  });

  it("revokes a role", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    const store = useRolesStore();
    await store.revoke("t", "moderator");
    expect(state.revokeCalls).toEqual([{ userId: "t", scope: "music", role: "moderator" }]);
  });

  it("captures an error from a denied grant", async () => {
    const { clients } = makeFakeClients();
    (clients.user as unknown as { grantRole: () => Promise<never> }).grantRole = () =>
      Promise.reject(new Error("permission denied"));
    setClientsForTest(clients);
    const store = useRolesStore();
    await expect(store.grant("t", "admin")).rejects.toThrow();
    expect(store.error).toBe("permission denied");
  });
});
