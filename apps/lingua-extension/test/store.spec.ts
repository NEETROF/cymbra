import { IDBFactory } from "fake-indexeddb";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { INTERFACE_LANGUAGE_KEY, interfaceLanguage } from "@/i18n/language.ts";
import {
  idbArea,
  isStoreMessage,
  messagedArea,
  MIGRATED_KEY,
  migrateStore,
  openStore,
  ownerArea,
  dropRetiredKeys,
  rememberInterfaceLanguage,
  STORE_KEYS,
  type StoreReply,
} from "@/state/store.ts";
import { type AsyncStorageArea, nativeLanguageOfStored, ROOT_KEY, saveBackup } from "@/state/storage.ts";

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
    expect(await area.get("cymbra-lingua-daily-v2")).toEqual({});

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

describe("the owner's handle", () => {
  it("says which keys a write changed", async () => {
    // Everything that must follow a mutation hangs off this: telling the surfaces, and
    // scheduling the sync. The sync trigger used to watch a storage key the data has since
    // left, and silently stopped firing (dogfooding: a level chosen on the phone never left it).
    const announced: string[][] = [];
    const backing = await freshStore();
    const owner = ownerArea(backing, (keys) => announced.push(keys));

    await owner.set({ [ROOT_KEY]: { v: 2, backup: "B" }, "cymbra-lingua-daily-v2": {} });

    expect(announced).toEqual([[ROOT_KEY, "cymbra-lingua-daily-v2"]]);
    expect(await backing.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: { v: 2, backup: "B" } });
  });

  it("announces nothing when the write fails", async () => {
    const announced: string[][] = [];
    const owner = ownerArea(
      {
        get: async () => ({}),
        set: async () => {
          throw new Error("refused");
        },
      },
      (keys) => announced.push(keys),
    );

    await expect(owner.set({ [ROOT_KEY]: "x" })).rejects.toThrow("refused");
    expect(announced).toEqual([]);
  });

  it("reads straight through", async () => {
    const backing = await freshStore();
    await backing.set({ [ROOT_KEY]: "READ" });

    expect(await ownerArea(backing, () => {}).get(ROOT_KEY)).toEqual({ [ROOT_KEY]: "READ" });
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
      "cymbra-lingua-daily-v2": { 20000: { exposures: 3 } },
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-enabled": true, // a preference: it stays where it is
      "cymbra-lingua-refresh": "TOKEN", // as does the session
    });
  });

  it("copies the reader's data, then lets the previous copy go", async () => {
    const store = await freshStore();

    const moved = await migrateStore(previous, store);

    expect(moved.sort()).toEqual([ROOT_KEY, "cymbra-lingua-daily-v2", "cymbra-lingua-status-cursor"].sort());
    expect(await store.get(ROOT_KEY)).toEqual({ [ROOT_KEY]: { v: 2, backup: "DECK" } });
    expect(await store.get("cymbra-lingua-status-cursor")).toEqual({ "cymbra-lingua-status-cursor": 42 });
    // The copy held space in the very area whose fullness the move escapes.
    expect(previous.store[ROOT_KEY]).toBeNull();
    // Preferences and tokens are not ours to touch.
    expect(previous.store["cymbra-lingua-enabled"]).toBe(true);
    expect(previous.store["cymbra-lingua-refresh"]).toBe("TOKEN");
    expect(await store.get("cymbra-lingua-enabled")).toEqual({});
  });

  it("releases a copy an earlier build left behind", async () => {
    // Devices that migrated while the copy was kept as a rollback path: the release it
    // guarded is verified, so the next start hands the space back.
    const store = await freshStore();
    await store.set({ [ROOT_KEY]: { v: 2, backup: "DECK" } });
    await previous.set({ [MIGRATED_KEY]: true });

    expect(await migrateStore(previous, store)).toEqual([]);

    expect(previous.store[ROOT_KEY]).toBeNull();
    // What the store does not hold is left alone rather than lost.
    expect(previous.store["cymbra-lingua-daily-v2"]).toEqual({ 20000: { exposures: 3 } });
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
    expect(STORE_KEYS).toContain("cymbra-lingua-daily-v4");
    expect(STORE_KEYS).toContain("cymbra-lingua-daily-v3"); // read as French-native until the first v4 write, kept for a downgrade
    expect(STORE_KEYS).toContain("cymbra-lingua-daily-v2"); // read as English until the first v3 write
    expect(STORE_KEYS).toContain("cymbra-lingua-card-cursor");
  });
});

describe("the retired keys", () => {
  it("drops the whole-document daily counts everywhere, and keeps the rest of the reader's data", async () => {
    const store = await freshStore();
    await store.set({
      "cymbra-lingua-daily": { 20000: { exposures: 90_000 } },
      "cymbra-lingua-daily-v2": { 20001: { exposures: 12, unknownSeen: 2 } },
      [ROOT_KEY]: { v: 2, backup: "B" },
      "cymbra-lingua-status-cursor": 7,
    });
    const settings = fakeArea({ "cymbra-lingua-daily": { 19999: { exposures: 5 } }, other: 1 });

    await dropRetiredKeys(store, settings);

    expect(await store.get("cymbra-lingua-daily")).toEqual({});
    expect(await store.get(["cymbra-lingua-daily-v2", ROOT_KEY, "cymbra-lingua-status-cursor"])).toEqual({
      "cymbra-lingua-daily-v2": { 20001: { exposures: 12, unknownSeen: 2 } },
      [ROOT_KEY]: { v: 2, backup: "B" },
      "cymbra-lingua-status-cursor": 7,
    });
    expect(settings.store["cymbra-lingua-daily"]).toBeNull();
    expect(settings.store.other).toBe(1);
  });

  it("has nothing to do once they are gone, so today's counts survive a restart", async () => {
    const store = await freshStore();
    await store.set({ "cymbra-lingua-daily-v2": { 20001: { exposures: 12 } } });
    const writes = vi.spyOn(store, "set");
    await dropRetiredKeys(store);
    await dropRetiredKeys(store);
    expect(writes).not.toHaveBeenCalled();
    expect(await store.get("cymbra-lingua-daily-v2")).toEqual({
      "cymbra-lingua-daily-v2": { 20001: { exposures: 12 } },
    });
  });

  it("is never a key the store claims to own", () => {
    expect(STORE_KEYS).not.toContain("cymbra-lingua-daily");
  });
});

describe("the interface language follows the stored profile", () => {
  // A package glossed for readers of Spanish and of English, so that a profile's choice can be
  // read (M22: a language no listed pair is glossed in reads French).
  const PAIRS = ["en-fr", "es-fr", "en-es", "es-en"];
  const spanish = JSON.stringify({ profile: { native_language: "Spanish", studied_languages: ["English"] } });
  const english = JSON.stringify({ profile: { native_language: "English", studied_languages: ["Spanish"] } });
  const noNative = JSON.stringify({ profile: { studied_languages: ["English"] } });

  /** What the owner says when it cannot keep the key: a test listens, the extension logs. */
  function quietWarnings() {
    return vi.spyOn(console, "warn").mockImplementation(() => {});
  }

  it("The key follows the profile: a backup the owner writes mirrors its native language", async () => {
    const preferences = fakeArea();
    const owner = ownerArea(fakeArea(), () => {}, preferences, PAIRS);
    await owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } });
    await owner.mirrored();
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("es");
    // A surface opened afterwards reads it from its preferences, with no engine.
    expect(await interfaceLanguage(preferences)).toBe("es");
  });

  it("A restore from a file: the backup saved through the owner writes the key", async () => {
    const preferences = fakeArea({ [INTERFACE_LANGUAGE_KEY]: "fr" });
    const owner = ownerArea(fakeArea(), () => {}, preferences, PAIRS);
    await saveBackup(owner, spanish);
    await owner.mirrored();
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("es");
  });

  it("Every installed reader: a profile naming no native language reads French", async () => {
    const preferences = fakeArea();
    const owner = ownerArea(fakeArea(), () => {}, preferences, PAIRS);
    await owner.set({ [ROOT_KEY]: { v: 2, backup: noNative } });
    await owner.mirrored();
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("fr");
    expect(nativeLanguageOfStored(undefined)).toBe("fr");
    expect(nativeLanguageOfStored({ statuses: {} })).toBe("fr"); // a v1 store predates the profile
  });

  it("a language no shipped pair is glossed in reads French, as the native language does", () => {
    expect(nativeLanguageOfStored({ v: 2, backup: spanish }, ["en-fr", "es-fr"])).toBe("fr");
    expect(nativeLanguageOfStored({ v: 2, backup: spanish }, PAIRS)).toBe("es");
  });

  it("the owner's forget of the backup mirrors French, like a device with none", async () => {
    const preferences = fakeArea({ [INTERFACE_LANGUAGE_KEY]: "es" });
    const owner = ownerArea(fakeArea(), () => {}, preferences, PAIRS);
    await owner.set({ [ROOT_KEY]: null });
    await owner.mirrored();
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("fr");
  });

  it("a backup that does not parse mirrors French, and says so", async () => {
    const warn = quietWarnings();
    const preferences = fakeArea({ [INTERFACE_LANGUAGE_KEY]: "es" });
    const owner = ownerArea(fakeArea(), () => {}, preferences, PAIRS);
    await owner.set({ [ROOT_KEY]: { v: 2, backup: "{not json" } });
    await owner.mirrored();
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("fr");
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(SyntaxError));
    warn.mockRestore();
  });

  it("never delays a write's announcement by the mirror", async () => {
    // The backup is written at every status change: the surfaces hear of it as soon as it
    // has landed, whatever the mirror then does.
    let release!: () => void;
    const preferences: AsyncStorageArea = {
      get: async () => ({}),
      set: () => new Promise<void>((resolve) => (release = resolve)),
    };
    const announced: string[][] = [];
    const owner = ownerArea(fakeArea(), (keys) => announced.push(keys), preferences, PAIRS);
    await owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } });
    expect(announced).toEqual([[ROOT_KEY]]);
    await new Promise((resolve) => setTimeout(resolve, 0)); // the mirror is now waiting on the write
    release();
    await owner.mirrored();
  });

  it("writes the key once for a burst of backups naming the same language", async () => {
    const preferences = fakeArea();
    const writes = vi.spyOn(preferences, "set");
    const owner = ownerArea(fakeArea(), () => {}, preferences, PAIRS);
    // Three status changes in a row, then a restart of the chain with a profile that did move.
    await Promise.all([
      owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } }),
      owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } }),
      owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } }),
    ]);
    await owner.mirrored();
    await owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } });
    await owner.mirrored();
    expect(writes).toHaveBeenCalledTimes(1);
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("es");
    await owner.set({ [ROOT_KEY]: { v: 2, backup: english } });
    await owner.mirrored();
    expect(writes).toHaveBeenCalledTimes(2);
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("en");
  });

  it("mirrors the last of a burst, whichever language the earlier ones named", async () => {
    const preferences = fakeArea();
    const owner = ownerArea(fakeArea(), () => {}, preferences, PAIRS);
    await Promise.all([
      owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } }),
      owner.set({ [ROOT_KEY]: { v: 2, backup: english } }),
    ]);
    await owner.mirrored();
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("en");
  });

  it("the first start keeps a key the device already holds, without reading the backup", async () => {
    const preferences = fakeArea({ [INTERFACE_LANGUAGE_KEY]: "es" });
    const store = fakeArea({ [ROOT_KEY]: { v: 2, backup: noNative } });
    const reads = vi.spyOn(store, "get");
    await rememberInterfaceLanguage(store, preferences, PAIRS);
    expect(reads).not.toHaveBeenCalled();
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("es");
  });

  it("the first start computes an absent or unknown key from the stored profile", async () => {
    // A device updated with a profile already stored: the one case the migration needs.
    const preferences = fakeArea();
    await rememberInterfaceLanguage(fakeArea({ [ROOT_KEY]: { v: 2, backup: spanish } }), preferences, PAIRS);
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("es");
    const unknown = fakeArea({ [INTERFACE_LANGUAGE_KEY]: "de" });
    await rememberInterfaceLanguage(fakeArea(), unknown, PAIRS);
    expect(unknown.store[INTERFACE_LANGUAGE_KEY]).toBe("fr");
  });

  it("writes nothing for a key that is not the backup, and announces as before", async () => {
    const preferences = fakeArea();
    const announced: string[][] = [];
    const owner = ownerArea(fakeArea(), (keys) => announced.push(keys), preferences, PAIRS);
    await owner.set({ "cymbra-lingua-device": "dev-1" });
    await owner.mirrored();
    expect(preferences.store).toEqual({});
    expect(announced).toEqual([["cymbra-lingua-device"]]);
  });

  it("a mirror that cannot be written costs the backup nothing, is said, and is tried again", async () => {
    const warn = quietWarnings();
    const store = fakeArea();
    let full = true;
    const preferences = fakeArea();
    const underlying = preferences.set.bind(preferences);
    preferences.set = async (items) => {
      if (full) throw new Error("QUOTA_BYTES exceeded");
      await underlying(items);
    };
    const announced: string[][] = [];
    const owner = ownerArea(store, (keys) => announced.push(keys), preferences, PAIRS);
    await expect(owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } })).resolves.toBeUndefined();
    expect(store.store[ROOT_KEY]).toEqual({ v: 2, backup: spanish });
    expect(announced).toEqual([[ROOT_KEY]]);
    await owner.mirrored();
    expect(preferences.store).toEqual({});
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
    // The area has room again: the next write of the same language still finds the key missing.
    full = false;
    await owner.set({ [ROOT_KEY]: { v: 2, backup: spanish } });
    await owner.mirrored();
    expect(preferences.store[INTERFACE_LANGUAGE_KEY]).toBe("es");
    warn.mockRestore();
  });
});
