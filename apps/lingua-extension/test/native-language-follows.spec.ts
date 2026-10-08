import { describe, expect, it, vi } from "vitest";
import { acceptedLanguages, readerPairs, readingLanguage } from "@/analyzer/pairs.ts";
import type { NativeLanguage, StudiedLanguage } from "@/analyzer/types.ts";
import { mountSettings } from "@/reading/settings-view.ts";
import { studiedForNative } from "@/state/native-language.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { SyncEngine, type SyncClients } from "@/sync/sync.ts";
import { makeFakePort } from "./helpers.ts";

// What follows a change of native language (add-lingua-native-language-choice D5), on the modules
// that measure it, with es-en shipping beside the French-native pairs: a French reader of English and
// Spanish chooses English. Their profile becomes English-native, studying Spanish alone
// (`studiedForNative`); each module reads it from the engine rebuilt for English (D3).

const MIXED = ["en-fr", "es-fr", "es-en"];

function profile(native: NativeLanguage, studied: StudiedLanguage[]) {
  return { nativeLanguage: async () => native, studiedLanguages: async () => studied };
}

const before = profile("fr", ["en", "es"]);
const after = profile("en", studiedForNative(["en", "es"], "en", MIXED) as StudiedLanguage[]);

function fakeArea(seed: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = { ...seed };
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in store).map((k) => [k, store[k]]));
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

const settle = async (): Promise<void> => {
  for (let i = 0; i < 4; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

describe("Changing the native language", () => {
  it("the accepted languages, and the language read in, are Spanish's", async () => {
    expect(await acceptedLanguages(before, MIXED)).toEqual(["en", "es"]);
    expect(await acceptedLanguages(after, MIXED)).toEqual(["es"]);
    expect(await readingLanguage(after, MIXED)).toBe("es");
  });

  it("the translation pairs follow: es-en, never a pair glossed in French", async () => {
    expect(readerPairs(await acceptedLanguages(before, MIXED), "fr", MIXED)).toEqual(["en-fr", "es-fr"]);
    expect(readerPairs(await acceptedLanguages(after, MIXED), "en", MIXED)).toEqual(["es-en"]);
  });

  /** One sync of a device whose last sync accepted `last`, its cursors past what it pulled. */
  async function syncAfter(last: StudiedLanguage[]) {
    const { port } = makeFakePort();
    Object.assign(port, after, { restore: async () => {}, backup: async () => "B" });
    const pullChanges = vi.fn(async () => ({ changes: [], cursor: 7n, declaredLevels: [] }));
    const pullCards = vi.fn(async () => ({ cards: [], cursor: 9n }));
    const clients = {
      knownWords: { pushOps: vi.fn(async () => ({ applied: 0n, cursor: 0n })), pullChanges },
      deck: { pushCards: vi.fn(async () => ({ applied: 0n, cursor: 0n })), pullCards },
      stats: { upsertDailyStats: vi.fn(async () => ({ upserted: 0n })) },
      data: { getDataState: vi.fn(async () => ({ erasedAt: 0n })), eraseMyData: vi.fn() },
    } as unknown as SyncClients;
    const storage = fakeArea({
      lingua: { v: 2, backup: "B" },
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-card-cursor": 7,
      "cymbra-lingua-sync-labels": true,
      "cymbra-lingua-sync-languages": last,
    });
    await new SyncEngine({
      port,
      storage,
      clients: () => clients,
      deviceId: "d",
      acceptedLanguages: () => acceptedLanguages(port, MIXED),
    }).sync();
    return { pullChanges, pullCards, storage };
  }

  it("the sync accepts Spanish alone: narrowed, the cursors kept and English's cards left where they are", async () => {
    const { pullChanges, pullCards, storage } = await syncAfter(["en", "es"]);
    expect(pullChanges).toHaveBeenCalledWith({ cursor: 42n });
    expect(pullCards).toHaveBeenCalledWith({ cursor: 7n, languages: ["es"], anyGlossLanguage: true });
    expect(storage.store["cymbra-lingua-sync-languages"]).toEqual(["es"]);
  });

  it("A native language that was the only studied one: Spanish is new, its cards pulled from the start", async () => {
    const { pullChanges, pullCards } = await syncAfter(["en"]);
    expect(pullChanges).toHaveBeenCalledWith({ cursor: 0n });
    expect(pullCards).toHaveBeenCalledWith({ cursor: 0n, languages: ["es"], anyGlossLanguage: true });
  });

  it("A full reset keeps the native language: Réglages start the English reader's Spanish over", async () => {
    // The engine's reset builds the fresh state on its own native language (crates/lingua-wasm,
    // tests/reprofile.rs): an English engine's reader studies Spanish again, the first pair's.
    vi.stubGlobal("chrome", {
      runtime: { sendMessage: async () => undefined },
      storage: { local: { get: async () => ({}), set: async () => {} }, onChanged: { addListener: () => {} } },
    });
    const fake = makeFakePort();
    Object.assign(fake.port, after);
    fake.port.reset = async () => {};
    fake.port.hasLevels = async () => true;
    const container = document.createElement("div");
    document.body.replaceChildren(container);
    mountSettings(container, fake.port, fakeArea(), {
      persist: async () => {},
      store: fakeArea(),
      sync: {
        available: async () => false,
        syncNow: async () => ({ ok: true }),
        lastSync: async () => null,
        now: () => 0,
        watch: () => {},
      },
      openPage: () => {},
      pairs: MIXED,
      interfaceLanguage: "en",
    });
    await settle();
    const asked = fake.calls.languages.length;
    const click = (label: string) =>
      [...container.querySelectorAll<HTMLButtonElement>("button")].find((b) => b.textContent === label)!.click();
    click("Reset…");
    click("Full — erase everything");
    click("Yes, confirm");
    await settle();
    // Every accepted language starts over: Spanish alone, the English reader's.
    expect(fake.calls.languages.slice(asked)).toContain("es");
    expect(fake.calls.languages.slice(asked)).not.toContain("en");
    expect(await fake.port.nativeLanguage()).toBe("en");
    vi.unstubAllGlobals();
  });
});
