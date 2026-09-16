import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { meLabel, useMeStore } from "@/stores/me";

// The signed-in account shown in the sidebar. Driven through the injectable client seam
// — no network, no component.

function wire(account: unknown | (() => never)): void {
  setClientsForTest({
    user: {
      getAccount: async () => {
        if (typeof account === "function") (account as () => never)();
        return account;
      },
    },
  } as unknown as Clients);
}

describe("meLabel", () => {
  it("prefers the handle, then the display name, then a short id", () => {
    const base = { userId: "0f9c2a4e-1111-4222-8333-444455556666", handle: null, displayName: null };
    expect(meLabel({ ...base, handle: "Cymbra", displayName: "Guillaume" })).toBe("Cymbra");
    expect(meLabel({ ...base, displayName: "Guillaume" })).toBe("Guillaume");
    expect(meLabel(base)).toBe("0f9c2a4e");
  });

  it("falls back to the token's user id while the profile is still loading", () => {
    expect(meLabel(null, "abcdef1234")).toBe("abcdef12");
    expect(meLabel(null)).toBe("");
  });
});

describe("me store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("loads the caller's own account", async () => {
    wire({ userId: "u1", handle: "Cymbra", displayName: "Guillaume" });
    const store = useMeStore();
    await store.load();
    expect(store.profile.status).toBe("success");
    expect(store.me).toEqual({ userId: "u1", handle: "Cymbra", displayName: "Guillaume" });
  });

  it("maps an account without a handle to nulls, not undefined", async () => {
    wire({ userId: "u1" });
    const store = useMeStore();
    await store.load();
    expect(store.me).toEqual({ userId: "u1", handle: null, displayName: null });
  });

  it("keeps a failure in the union and reports no account", async () => {
    wire(() => {
      throw new ConnectError("nope", Code.Unavailable);
    });
    const store = useMeStore();
    await store.load();
    expect(store.profile.status).toBe("error");
    expect(store.me).toBeNull();
  });

  it("forgets the account on sign-out", async () => {
    wire({ userId: "u1", handle: "Cymbra" });
    const store = useMeStore();
    await store.load();
    store.clear();
    expect(store.profile.status).toBe("idle");
    expect(store.me).toBeNull();
  });
});
