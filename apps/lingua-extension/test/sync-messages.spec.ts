import { describe, expect, it, vi } from "vitest";
import { isSyncMessage, LAST_SYNC_KEY, loadLastSync, requestSync, syncAvailable, syncNow } from "@/sync/messages.ts";
import { lastSyncLabel, syncErrorCopy } from "@/sync/status.ts";
import { authErrorOf } from "@/state/auth-errors.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";

function fakeArea(): AsyncStorageArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = {};
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in store) out[k] = store[k];
      return out;
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

describe("sync messages", () => {
  it("recognises only its own messages", () => {
    expect(isSyncMessage({ type: "sync:request" })).toBe(true);
    expect(isSyncMessage({ type: "sync:request", force: true })).toBe(true);
    expect(isSyncMessage({ type: "account:state" })).toBe(false);
    expect(isSyncMessage(null)).toBe(false);
  });

  it("asks for a sync without failing the caller when the background is unreachable", async () => {
    const send = vi.fn(async () => {
      throw new Error("Could not establish connection");
    });

    await expect(requestSync("surface", send)).resolves.toBeUndefined();
    expect(send).toHaveBeenCalledWith({ type: "sync:request", reason: "surface" });
  });

  it("forces a sync and carries back its category", async () => {
    expect(await syncNow(async () => ({ ok: true }))).toEqual({ ok: true });
    expect(await syncNow(async () => ({ ok: false, error: "unauthenticated" }))).toEqual({
      ok: false,
      error: "unauthenticated",
    });
    const force = vi.fn(async () => ({ ok: true }));
    await syncNow(force);
    expect(force).toHaveBeenCalledWith({ type: "sync:request", force: true });
  });

  it("treats an unreachable background as unavailable", async () => {
    const send = async () => {
      throw new Error("no receiver");
    };
    expect(await syncNow(send)).toEqual({ ok: false, error: "unavailable" });
    expect(await syncAvailable(send)).toBe(false);
  });

  it("reads whether an account is signed in", async () => {
    expect(await syncAvailable(async () => ({ ok: true, state: { signedIn: true } }))).toBe(true);
    expect(await syncAvailable(async () => ({ ok: true, state: { signedIn: false } }))).toBe(false);
    expect(await syncAvailable(async () => undefined)).toBe(false);
  });

  it("reads the last successful sync, or null", async () => {
    const area = fakeArea();
    expect(await loadLastSync(area)).toBeNull();
    await area.set({ [LAST_SYNC_KEY]: 0 });
    expect(await loadLastSync(area)).toBeNull();
    await area.set({ [LAST_SYNC_KEY]: 1_700_000_000_000 });
    expect(await loadLastSync(area)).toBe(1_700_000_000_000);
  });
});

describe("sync status copy", () => {
  const now = Date.UTC(2026, 8, 17, 12, 0, 0);

  it("says how long ago this device synced", () => {
    expect(lastSyncLabel(null, now)).toBe("Pas encore synchronisé sur cet appareil.");
    expect(lastSyncLabel(now - 5_000, now)).toBe("Synchronisé à l'instant.");
    expect(lastSyncLabel(now + 5_000, now)).toBe("Synchronisé à l'instant."); // a clock nudged back
    expect(lastSyncLabel(now - 3 * 60_000, now)).toBe("Synchronisé il y a 3 min.");
    expect(lastSyncLabel(now - 5 * 3_600_000, now)).toBe("Synchronisé il y a 5 h.");
    expect(lastSyncLabel(now - 3 * 86_400_000, now)).toMatch(/^Dernière synchronisation le /);
  });

  it("names a full storage instead of blaming the network", () => {
    // Safari says it in words and only in words; the reader gets an actionable sentence.
    expect(authErrorOf(new Error("Invalid call to browser.storage.local.set(). Exceeded storage quota."))).toBe(
      "storageFull",
    );
    expect(authErrorOf(new Error("QUOTA_BYTES quota exceeded"))).toBe("storageFull");
    const quota = new Error("too big");
    quota.name = "QuotaExceededError";
    expect(authErrorOf(quota)).toBe("storageFull");
    // An ordinary failed fetch is still an unreachable server.
    expect(authErrorOf(new TypeError("Load failed"))).toBe("unavailable");

    expect(syncErrorCopy("storageFull")).toMatch(/mémoire de l’extension est pleine/);
  });

  it("explains a failure by category, never by message", () => {
    expect(syncErrorCopy("unavailable")).toMatch(/injoignable/);
    expect(syncErrorCopy("unauthenticated")).toMatch(/reconnecte/);
    expect(syncErrorCopy("conflict")).toMatch(/déjà en cours/);
    expect(syncErrorCopy("unknown")).toMatch(/a échoué/);
    expect(syncErrorCopy(undefined)).toMatch(/a échoué/);
  });
});
