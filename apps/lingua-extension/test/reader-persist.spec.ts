import { describe, expect, it, vi } from "vitest";
import { type AskedOnce, requestPersistence, type StorageManagerLike } from "@/reader/persist.ts";

// The library asks the browser to keep its storage — once — and says when it was refused
// (add-lingua-reader 3.4).

function memo(asked = false): AskedOnce & { marked: boolean } {
  const m = {
    marked: false,
    asked: async () => asked || m.marked,
    markAsked: async () => {
      m.marked = true;
    },
  };
  return m;
}

function storage(persisted: boolean, grants: boolean): StorageManagerLike & { persist: ReturnType<typeof vi.fn> } {
  return { persisted: vi.fn(async () => persisted), persist: vi.fn(async () => grants) };
}

describe("requestPersistence", () => {
  it("is granted without asking when the storage is already kept", async () => {
    const s = storage(true, false);
    expect(await requestPersistence(memo(), s)).toBe("granted");
    expect(s.persist).not.toHaveBeenCalled();
  });

  it("asks the first time and remembers it asked", async () => {
    const m = memo();
    const s = storage(false, true);
    expect(await requestPersistence(m, s)).toBe("granted");
    expect(s.persist).toHaveBeenCalledOnce();
    expect(m.marked).toBe(true);
  });

  it("reports a refusal, and does not ask again", async () => {
    const m = memo();
    const s = storage(false, false);
    expect(await requestPersistence(m, s)).toBe("refused");
    expect(await requestPersistence(m, s)).toBe("refused");
    expect(s.persist).toHaveBeenCalledOnce();
  });

  it("says the question does not exist where the browser has no storage manager", async () => {
    expect(await requestPersistence(memo(), undefined)).toBe("unavailable");
    expect(await requestPersistence(memo(), {} as StorageManagerLike)).toBe("unavailable");
  });

  it("never throws", async () => {
    const s = { persisted: vi.fn(async () => Promise.reject(new Error("no"))), persist: vi.fn() };
    expect(await requestPersistence(memo(), s)).toBe("unavailable");
  });
});
