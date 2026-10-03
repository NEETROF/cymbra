import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mountSettings } from "@/reading/settings-view.ts";
import { ReadingSession } from "@/reading/session.ts";
import { mountReview } from "@/review/review-page.ts";
import { type AsyncStorageArea, ROOT_KEY, STORAGE_VERSION } from "@/state/storage.ts";
import { STORE_CHANGED_KEY } from "@/state/store.ts";
import { mountStats } from "@/stats/view.ts";
import { SyncEngine, type SyncClients } from "@/sync/sync.ts";
import { makeFakePort } from "./helpers.ts";

// Each surface reads in the reader's first studied language that the package ships, or the
// default pair's (add-lingua-studied-language-profile). The bundle's pairs are a build-time
// constant, so the pairs the rule consults are set per test here.
const packs = vi.hoisted(() => ({ shipped: ["en-fr"] }));
vi.mock("@/analyzer/pairs.ts", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/analyzer/pairs.ts")>();
  return {
    ...actual,
    readingLanguage: (port: Parameters<typeof actual.readingLanguage>[0]) =>
      actual.readingLanguage(port, packs.shipped),
  };
});

type ChangeListener = (changes: Record<string, { newValue?: unknown }>, areaName: string) => void;
let listeners: ChangeListener[];
/** What the background's store answers a surface with. */
let stored: Record<string, unknown>;

function fakeArea(seed: Record<string, unknown> = {}): AsyncStorageArea {
  const raw: Record<string, unknown> = { ...seed };
  return {
    async get(keys) {
      const list = keys == null ? Object.keys(raw) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in raw).map((k) => [k, raw[k]]));
    },
    async set(items) {
      Object.assign(raw, items);
    },
  };
}

/** A backup another context wrote, announced the way the store announces it. */
function storeElsewhere(backup: string): void {
  stored = { [ROOT_KEY]: { v: STORAGE_VERSION, backup } };
  for (const listener of listeners)
    listener({ [STORE_CHANGED_KEY]: { newValue: { rev: 1, keys: [ROOT_KEY] } } }, "local");
}

const settle = async (): Promise<void> => {
  for (let i = 0; i < 3; i++) await new Promise((resolve) => setTimeout(resolve, 0));
};

/** A fake port whose reader studies Spanish, then English. */
async function spanishThenEnglish() {
  const fake = makeFakePort();
  await fake.port.setStudiedLanguages(["es", "en"]);
  return fake;
}

beforeEach(() => {
  listeners = [];
  stored = {};
  vi.stubGlobal("chrome", {
    runtime: {
      sendMessage: vi.fn(async (msg: { type?: string }) => (msg?.type === "store:get" ? { items: stored } : undefined)),
      onMessage: { addListener: () => {} },
    },
    storage: {
      local: { get: async () => ({}), set: async () => {} },
      onChanged: { addListener: (fn: ChangeListener) => void listeners.push(fn) },
    },
  });
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.replaceChildren();
  document.documentElement.querySelectorAll("[data-cymbra-lingua-skip]").forEach((el) => el.remove());
});

describe.each([
  { shipped: ["en-fr"], reads: "en" },
  { shipped: ["en-fr", "es-fr"], reads: "es" },
])("a reader of Spanish then English, with $shipped shipped", ({ shipped, reads }) => {
  beforeEach(() => {
    packs.shipped = shipped;
  });

  it("reads the page in the first shipped language", async () => {
    const { port, calls } = await spanishThenEnglish();
    await new ReadingSession(port, { css: { tokens: "", popup: "", drawer: "", hud: "" }, surface: "book" }).start(
      null,
    );
    expect(calls.languages.length).toBeGreaterThan(0);
    expect(new Set(calls.languages)).toEqual(new Set([reads]));
  });

  it("sets Réglages in it", async () => {
    const { port, calls } = await spanishThenEnglish();
    const container = document.createElement("div");
    document.body.append(container);
    mountSettings(container, port, fakeArea(), {
      persist: async () => {},
      store: fakeArea(),
      sync: {
        available: async () => false,
        syncNow: async () => ({ ok: true }),
        lastSync: async () => null,
        now: () => 0,
        watch: () => {},
      },
    });
    await vi.waitFor(() => expect(calls.languages.length).toBeGreaterThan(0));
    expect(new Set(calls.languages)).toEqual(new Set([reads]));
  });

  it("counts its statistics", async () => {
    const { port, calls } = await spanishThenEnglish();
    await mountStats(document.createElement("div"), port, fakeArea());
    expect(calls.languages.length).toBeGreaterThan(0);
    expect(new Set(calls.languages)).toEqual(new Set([reads]));
  });

  it("credits its pack in Révision", async () => {
    const { port, calls } = await spanishThenEnglish();
    const container = document.createElement("div");
    document.body.append(container);
    mountReview(container, port, fakeArea(), { now: () => 0 });
    await settle();
    expect(new Set(calls.languages)).toEqual(new Set([reads]));
  });

  it("leaves the reader's studied languages as they were", async () => {
    const { port } = await spanishThenEnglish();
    await mountStats(document.createElement("div"), port, fakeArea());
    expect(await port.studiedLanguages()).toEqual(["es", "en"]);
  });
});

describe("with en-fr and es-fr shipped", () => {
  beforeEach(() => {
    packs.shipped = ["en-fr", "es-fr"];
  });

  it("an open page follows studied languages changed in another context", async () => {
    const { port, calls } = makeFakePort();
    // The backup another context stores puts Spanish first.
    port.restore = async (backup) => {
      if (backup === "SPANISH FIRST") await port.setStudiedLanguages(["es", "en"]);
    };
    await new ReadingSession(port, { css: { tokens: "", popup: "", drawer: "", hud: "" }, surface: "book" }).start(
      null,
    );
    expect(new Set(calls.languages)).toEqual(new Set(["en"]));

    calls.languages.length = 0;
    storeElsewhere("SPANISH FIRST");
    await vi.waitFor(() => expect(calls.languages.length).toBeGreaterThan(0));
    expect(new Set(calls.languages)).toEqual(new Set(["es"]));
  });

  it("an erasure recalibrates the default language, the reset profile's", async () => {
    const { port, calls } = await spanishThenEnglish();
    const pulled = { changes: [], declaredLevels: [], cursor: 0n };
    const clients = {
      knownWords: { pushOps: vi.fn(async () => ({})), pullChanges: vi.fn(async () => pulled) },
      deck: { pushCards: vi.fn(async () => ({})), pullCards: vi.fn(async () => ({ cards: [], cursor: 0n })) },
      stats: { upsertDailyStats: vi.fn(async () => ({})) },
      data: { getDataState: vi.fn(async () => ({ erasedAt: 5n })), eraseMyData: vi.fn() },
    } as unknown as SyncClients;
    // A device that synced before the account was erased elsewhere.
    const storage = fakeArea({ "cymbra-lingua-status-cursor": 3 });

    await new SyncEngine({ port, storage, clients: () => clients, deviceId: "d" }).sync();

    expect(await port.studiedLanguages()).toEqual(["en"]);
    expect(new Set(calls.languages)).toEqual(new Set(["en"]));
  });
});
