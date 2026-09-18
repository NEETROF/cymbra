import { IDBFactory } from "fake-indexeddb";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  idbArea,
  isStoreMessage,
  messagedArea,
  MIGRATED_KEY,
  migrateStore,
  openStore,
  STORE_KEYS,
  type StoreReply,
} from "@/state/store.ts";
import { type AsyncStorageArea, ROOT_KEY } from "@/state/storage.ts";

function fakeArea(seed: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = { ...seed };
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

/** A fresh database per test: fake-indexeddb keeps one per factory. */
async function freshStore(): Promise<AsyncStorageArea> {
  return idbArea(await openStore(new IDBFactory()));
}

describe("the durable store", () => {
  it("round-trips values, reads several keys, and forgets on null", async () => {
    const area = await freshStore();

    await area.set({ [ROOT_KEY]: { v: 2, backup: "BACKUP" }, "cymbra-lingua-device": "dev-1" });

    expect(await area.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: { v: 2, backup: "BACKUP" } });
    expect(await area.get([ROOT_KEY, "cymbra-lingua-device"])).toEqual({
      [ROOT_KEY]: { v: 2, backup: "BACKUP" },
      "cymbra-lingua-device": "dev-1",
    });
    // A key that was never written simply is not there.
    expect(await area.get("cymbra-lingua-daily")).toEqual({});

    await area.set({ "cymbra-lingua-device": null });
    expect(await area.get("cymbra-lingua-device")).toEqual({});
  });

  it("holds far more than the settings area would accept", async () => {
    // The incident this change answers: a store that grew past a few megabytes failed
    // every write. Ten megabytes is unremarkable here.
    const area = await freshStore();
    const big = "x".repeat(10 * 1024 * 1024);

    await area.set({ [ROOT_KEY]: { v: 2, backup: big } });

    const read = (await area.get(ROOT_KEY))[ROOT_KEY] as { backup: string };
    expect(read.backup).toHaveLength(big.length);
  });

  it("survives being reopened", async () => {
    const factory = new IDBFactory();
    await idbArea(await openStore(factory)).set({ [ROOT_KEY]: { v: 2, backup: "KEEP" } });

    const reopened = idbArea(await openStore(factory));

    expect(await reopened.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: { v: 2, backup: "KEEP" } });
  });
});

describe("the messaged area", () => {
  it("asks the owner and returns what it answers", async () => {
    const send = vi.fn(async () => ({ ok: true, items: { [ROOT_KEY]: "FROM-OWNER" } }) satisfies StoreReply);
    const area = messagedArea(send);

    expect(await area.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: "FROM-OWNER" });
    expect(send).toHaveBeenCalledWith({ type: "store:get", keys: ROOT_KEY });

    await area.set({ [ROOT_KEY]: "TO-OWNER" });
    expect(send).toHaveBeenLastCalledWith({ type: "store:set", items: { [ROOT_KEY]: "TO-OWNER" } });
  });

  it("reads nothing rather than throwing when the owner says nothing", async () => {
    const area = messagedArea(async () => undefined);
    expect(await area.get(ROOT_KEY)).toEqual({});
  });

  it("recognises only its own messages", () => {
    expect(isStoreMessage({ type: "store:get", keys: null })).toBe(true);
    expect(isStoreMessage({ type: "store:set", items: {} })).toBe(true);
    expect(isStoreMessage({ type: "sync:request" })).toBe(false);
    expect(isStoreMessage(null)).toBe(false);
  });
});

describe("moving the reader's data", () => {
  let previous: ReturnType<typeof fakeArea>;

  beforeEach(() => {
    previous = fakeArea({
      [ROOT_KEY]: { v: 2, backup: "DECK" },
      "cymbra-lingua-daily": { 20000: { exposures: 3 } },
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-enabled": true, // a preference: it stays where it is
      "cymbra-lingua-refresh": "TOKEN", // as does the session
    });
  });

  it("copies the reader's data and leaves the previous copy in place", async () => {
    const store = await freshStore();

    const moved = await migrateStore(previous, store);

    expect(moved.sort()).toEqual([ROOT_KEY, "cymbra-lingua-daily", "cymbra-lingua-status-cursor"].sort());
    expect(await store.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: { v: 2, backup: "DECK" } });
    expect(await store.get("cymbra-lingua-status-cursor")).toEqual({ "cymbra-lingua-status-cursor": 42 });
    // A build that rolls back must still find a deck.
    expect(previous.store[ROOT_KEY]).toEqual({ v: 2, backup: "DECK" });
    // Preferences and tokens were not touched.
    expect(await store.get("cymbra-lingua-enabled")).toEqual({});
    expect(await store.get("cymbra-lingua-refresh")).toEqual({});
  });

  it("does nothing on a second start", async () => {
    const store = await freshStore();
    await migrateStore(previous, store);
    await store.set({ [ROOT_KEY]: { v: 2, backup: "NEWER" } }); // the reader kept going

    expect(await migrateStore(previous, store)).toEqual([]);

    expect(await store.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: { v: 2, backup: "NEWER" } });
  });

  it("redoes a run that was cut short, because the mark is written last", async () => {
    const store = await freshStore();
    const failing: AsyncStorageArea = {
      get: (keys) => store.get(keys),
      set: async () => {
        throw new Error("killed mid-write");
      },
    };

    await expect(migrateStore(previous, failing)).rejects.toThrow();
    expect(previous.store[MIGRATED_KEY]).toBeUndefined();

    expect((await migrateStore(previous, store)).length).toBeGreaterThan(0);
    expect(previous.store[MIGRATED_KEY]).toBe(true);
  });

  it("drops the previous copy rather than staying stuck when the area is full", async () => {
    // What a reader's phone actually hit: the copies are what fill the area, and the mark
    // cannot be written into a full one — so the move never finished and the extension fell
    // back onto the very area that was full.
    const store = await freshStore();
    let isFull = true; // until the copies it holds are dropped
    const full: AsyncStorageArea & { store: Record<string, unknown> } = {
      store: previous.store,
      get: (keys) => previous.get(keys),
      set: async (items) => {
        const freeing = Object.values(items).every((value) => value === null);
        if (isFull && !freeing) {
          throw new Error("Invalid call to browser.storage.local.set(). Exceeded storage quota.");
        }
        if (freeing) isFull = false; // the space the copies took is back
        await previous.set(items);
      },
    };

    const moved = await migrateStore(full, store);

    expect(moved).toContain(ROOT_KEY);
    expect(await store.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: { v: 2, backup: "DECK" } });
    expect(full.store[ROOT_KEY]).toBeNull(); // the space is back
    expect(full.store[MIGRATED_KEY]).toBe(true); // and the move is done, not retried for ever
    expect(full.store["cymbra-lingua-refresh"]).toBe("TOKEN"); // the session was not touched
  });

  it("never puts a stale copy over what the store already holds", async () => {
    // A first run copied and was cut short before its mark; the reader kept working.
    const store = await freshStore();
    await store.set({ [ROOT_KEY]: { v: 2, backup: "NEWER" } });

    await migrateStore(previous, store);

    expect(await store.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: { v: 2, backup: "NEWER" } });
    // What the store did not have is still copied.
    expect(await store.get("cymbra-lingua-status-cursor")).toEqual({ "cymbra-lingua-status-cursor": 42 });
  });

  it("has nothing to move on a fresh install", async () => {
    const store = await freshStore();

    expect(await migrateStore(fakeArea(), store)).toEqual([]);
  });

  it("moves every key it claims to own", async () => {
    // The list is the contract between the owner and the surfaces: keep it honest.
    expect(STORE_KEYS).toContain(ROOT_KEY);
    expect(STORE_KEYS).toContain("cymbra-lingua-daily");
    expect(STORE_KEYS).toContain("cymbra-lingua-card-cursor");
  });
});
