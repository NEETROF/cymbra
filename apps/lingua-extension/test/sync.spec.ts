import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CardOp, DeclaredLevelOp, StatusChangeIn, StatusOp } from "@/analyzer/port.ts";
import type { StudiedLanguage } from "@/analyzer/types.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { clearSyncCursors, getOrCreateDeviceId, SyncEngine, type SyncClients } from "@/sync/sync.ts";
import { makeFakePort } from "./helpers.ts";

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

const ROOT_KEY = "lingua";
const v2 = (backup: string) => ({ [ROOT_KEY]: { v: 2, backup } });

/** A fake port exposing controllable sync exports + recording applies. */
function syncPort(over: {
  statusOps?: StatusOp[];
  cardOps?: CardOp[];
  levelOps?: DeclaredLevelOp[];
  onApplyStatuses?: (c: StatusChangeIn[]) => number;
  onApplyCards?: (c: CardOp[]) => number;
  onApplyLevels?: (c: DeclaredLevelOp[]) => number;
}) {
  const { port, calls: fake } = makeFakePort();
  let wiped = false;
  const calls = {
    resets: 0,
    restored: [] as string[],
    appliedStatuses: [] as StatusChangeIn[][],
    appliedCards: [] as CardOp[][],
    appliedLevels: [] as DeclaredLevelOp[][],
  };
  port.restore = async (j) => void calls.restored.push(j);
  port.backup = async () => "MERGED-BACKUP";
  // A reset empties what the engine would export afterwards, like the real one.
  port.reset = async () => {
    wiped = true;
    calls.resets += 1;
  };
  port.exportStatusOps = async () => (wiped ? [] : (over.statusOps ?? []));
  port.exportCardOps = async () => (wiped ? [] : (over.cardOps ?? []));
  port.exportDeclaredLevels = async () => (wiped ? [] : (over.levelOps ?? []));
  port.applyStatusChanges = async (c) => {
    calls.appliedStatuses.push(c);
    return over.onApplyStatuses?.(c) ?? 0;
  };
  port.applyCardOps = async (c) => {
    calls.appliedCards.push(c);
    return over.onApplyCards?.(c) ?? 0;
  };
  port.applyDeclaredLevelChanges = async (c) => {
    calls.appliedLevels.push(c);
    return over.onApplyLevels?.(c) ?? 0;
  };
  return { port, calls, asked: fake.languages };
}

interface WireChange {
  language: string;
  lemma: string;
  status: string;
  updatedAt: bigint;
  sequence: bigint;
}
interface WireCard {
  clientId: string;
  language?: string;
  glossLanguage?: string;
  lemma: string;
  surfaceForm: string;
  sourceSentence: string;
  source: string;
  gloss: string;
  fsrsState: string;
  deleted: boolean;
  clientTs: bigint;
  deviceId: string;
}
interface WireLevel {
  language: string;
  level: string;
  updatedAt: bigint;
  sequence: bigint;
}

function fakeClients() {
  const pushOps = vi.fn(async () => ({ applied: 0n, cursor: 0n }));
  const pushCards = vi.fn(async () => ({ applied: 0n, cursor: 0n }));
  const pullChanges = vi.fn(async () => ({
    changes: [] as WireChange[],
    cursor: 7n,
    declaredLevels: [] as WireLevel[],
  }));
  const pullCards = vi.fn(async () => ({ cards: [] as WireCard[], cursor: 9n }));
  const upsertDailyStats = vi.fn(async () => ({ upserted: 0n }));
  const getDataState = vi.fn(async () => ({ erasedAt: 0n }));
  const eraseMyData = vi.fn(async () => ({ erasedAt: 0n }));
  const clients = {
    knownWords: { pushOps, pullChanges },
    deck: { pushCards, pullCards },
    stats: { upsertDailyStats },
    data: { getDataState, eraseMyData },
  } as unknown as SyncClients;
  return { clients, pushOps, pushCards, pullChanges, pullCards, upsertDailyStats, getDataState, eraseMyData };
}

beforeEach(() => vi.clearAllMocks());

describe("getOrCreateDeviceId", () => {
  it("generates once and reuses the stored id", async () => {
    const area = fakeArea();
    const id1 = await getOrCreateDeviceId(area);
    const id2 = await getOrCreateDeviceId(area);
    expect(id1).toMatch(/[0-9a-f-]{36}/);
    expect(id2).toBe(id1);
  });
});

describe("SyncEngine", () => {
  it("does nothing when there is no local state yet", async () => {
    const { port } = syncPort({});
    const f = fakeClients();
    const engine = new SyncEngine({ port, storage: fakeArea(), clients: () => f.clients, deviceId: "dev" });
    expect(await engine.sync()).toBeNull();
    expect(f.pushOps).not.toHaveBeenCalled();
  });

  it("pushes local statuses/cards (mapping updated_at→clientTs + device id) and advances cursors", async () => {
    const { port } = syncPort({
      statusOps: [{ language: "en", lemma: "run", status: "known", provenance: "manual", updated_at: 1700 }],
      cardOps: [
        {
          client_id: "seldom",
          language: "en",
          lemma: "seldom",
          surface_form: "seldom",
          source_sentence: "s",
          source: "https://x",
          gloss: "rarement", // a French gloss: the engine leaves its label out (add-lingua-card-gloss-language)
          fsrs_state: "{}",
          deleted: false,
          client_ts: 1800,
          device_id: "",
        },
      ],
    });
    const f = fakeClients();
    const storage = fakeArea(v2("BACKUP"));
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "dev-1" });

    const res = await engine.sync();
    expect(res).toEqual({ pushedStatuses: 1, pushedCards: 1, pulled: 0 });
    expect(f.pushOps).toHaveBeenCalledWith({
      ops: [
        { language: "en", lemma: "run", status: "known", provenance: "manual", clientTs: 1700n, deviceId: "dev-1" },
      ],
    });
    expect(f.pushCards).toHaveBeenCalledWith({
      cards: [
        {
          clientId: "seldom",
          language: "en",
          lemma: "seldom",
          surfaceForm: "seldom",
          sourceSentence: "s",
          gloss: "rarement",
          glossLanguage: "", // an absent label goes as empty, which the server reads as French
          fsrsState: "{}",
          deleted: false,
          clientTs: 1800n,
          deviceId: "dev-1",
        },
      ],
    });
    expect(f.pullChanges).toHaveBeenCalledWith({ cursor: 0n });
    expect(f.pullCards).toHaveBeenCalledWith({ cursor: 0n, languages: ["en"], anyGlossLanguage: true });
    expect(storage.store["cymbra-lingua-status-cursor"]).toBe(7);
    expect(storage.store["cymbra-lingua-card-cursor"]).toBe(9);
  });

  it("applies pulled changes (mapping bigint→number) and persists the merged backup only when something changed", async () => {
    const f = fakeClients();
    f.pullChanges.mockResolvedValueOnce({
      changes: [{ language: "en", lemma: "city", status: "known", updatedAt: 42n, sequence: 3n }],
      cursor: 12n,
      declaredLevels: [],
    });
    const { port, calls } = syncPort({ onApplyStatuses: () => 1 });
    const storage = fakeArea(v2("BACKUP"));
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });

    const res = await engine.sync();
    expect(res?.pulled).toBe(1);
    expect(calls.appliedStatuses[0]).toEqual([{ language: "en", lemma: "city", status: "known", updated_at: 42 }]);
    expect(storage.store[ROOT_KEY]).toEqual({ v: 2, backup: "MERGED-BACKUP" });
  });

  it("does not persist when a pull changed nothing", async () => {
    const f = fakeClients(); // pulls return empty
    const { port } = syncPort({});
    const storage = fakeArea(v2("ORIGINAL"));
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    expect(storage.store[ROOT_KEY]).toEqual({ v: 2, backup: "ORIGINAL" }); // untouched
  });

  it("sends one statistic per day and studied language, each with its language", async () => {
    const { port } = syncPort({});
    const f = fakeClients();
    const storage = fakeArea({
      ...v2("BACKUP"),
      "cymbra-lingua-daily-v3": {
        20000: {
          en: { exposures: 12, unknownSeen: 4, wordsLearned: 3, reviews: 5 },
          es: { exposures: 30, unknownSeen: 9, wordsLearned: 1, reviews: 0 },
        },
      },
    });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "dev-2" });
    await engine.sync();
    expect(f.upsertDailyStats).toHaveBeenCalledWith({
      stats: [
        {
          day: 20000,
          language: "en",
          deviceId: "dev-2",
          exposures: 12,
          unknownSeen: 4,
          wordsLearned: 3,
          reviewsDone: 5,
          nativeLanguage: "fr",
        },
        {
          day: 20000,
          language: "es",
          deviceId: "dev-2",
          exposures: 30,
          unknownSeen: 9,
          wordsLearned: 1,
          reviewsDone: 0,
          nativeLanguage: "fr",
        },
      ],
    });
  });

  it("upserts the local daily stats (mapping reviews→reviewsDone + device id)", async () => {
    const { port } = syncPort({});
    const f = fakeClients();
    const storage = fakeArea({
      ...v2("BACKUP"),
      "cymbra-lingua-daily-v2": {
        20000: { exposures: 12, unknownSeen: 4, wordsLearned: 3, reviews: 5 },
        20001: { exposures: 0, wordsLearned: 1, reviews: 0 }, // written before any reading: 0 new words
      },
    });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "dev-2" });
    await engine.sync();
    // unknownSeen is always set — its presence is what the server stores a stat on.
    expect(f.upsertDailyStats).toHaveBeenCalledWith({
      stats: [
        {
          day: 20000,
          language: "en",
          deviceId: "dev-2",
          exposures: 12,
          unknownSeen: 4,
          wordsLearned: 3,
          reviewsDone: 5,
          nativeLanguage: "fr",
        },
        {
          day: 20001,
          language: "en",
          deviceId: "dev-2",
          exposures: 0,
          unknownSeen: 0,
          wordsLearned: 1,
          reviewsDone: 0,
          nativeLanguage: "fr",
        },
      ],
    });
  });

  it("pushes the declared level (mapping updated_at→clientTs + device id) alongside the statuses", async () => {
    const { port } = syncPort({ levelOps: [{ language: "en", level: "B2", updated_at: 1700 }] });
    const f = fakeClients();
    const engine = new SyncEngine({
      port,
      storage: fakeArea(v2("BACKUP")),
      clients: () => f.clients,
      deviceId: "dev-1",
    });
    await engine.sync();
    // The level rides a pushOps call (empty ops, one declared level).
    expect(f.pushOps).toHaveBeenCalledWith({
      ops: [],
      declaredLevels: [{ language: "en", level: "B2", clientTs: 1700n, deviceId: "dev-1" }],
    });
  });

  it("applies a pulled declared level (mapping bigint→number) and persists the merge", async () => {
    const f = fakeClients();
    f.pullChanges.mockResolvedValueOnce({
      changes: [],
      cursor: 8n,
      declaredLevels: [{ language: "en", level: "C1", updatedAt: 99n, sequence: 4n }],
    });
    const { port, calls } = syncPort({ onApplyLevels: () => 1 });
    const storage = fakeArea(v2("BACKUP"));
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });

    const res = await engine.sync();
    expect(res?.pulled).toBe(1);
    expect(calls.appliedLevels[0]).toEqual([{ language: "en", level: "C1", updated_at: 99 }]);
    expect(storage.store[ROOT_KEY]).toEqual({ v: 2, backup: "MERGED-BACKUP" });
  });

  it("applies pulled changes onto the LATEST backup, not the start-of-sync snapshot (no clobber)", async () => {
    const storage = fakeArea(v2("B0"));
    const f = fakeClients();
    // A concurrent local mutation lands (another context persists) during the pull.
    f.pullChanges.mockImplementationOnce(async () => {
      await storage.set(v2("B1-concurrent"));
      return {
        changes: [{ language: "en", lemma: "city", status: "known", updatedAt: 5n, sequence: 1n }],
        cursor: 3n,
        declaredLevels: [],
      };
    });
    const { port, calls } = syncPort({ onApplyStatuses: () => 1 });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    // The pulled change is applied on top of the re-loaded latest backup, not B0.
    expect(calls.restored).toEqual(["B0", "B1-concurrent"]);
  });
});

describe("SyncEngine privacy controls (add-lingua-privacy-controls)", () => {
  const MARK = 5_000;
  const localOps = {
    statusOps: [{ language: "en", lemma: "run", status: "known", provenance: "manual", updated_at: 1_000 }],
  };

  it("never sends a card's page address and maps a pulled card to an empty source", async () => {
    const f = fakeClients();
    f.pullCards.mockResolvedValueOnce({
      cards: [
        {
          clientId: "seldom",
          lemma: "seldom",
          surfaceForm: "seldom",
          sourceSentence: "s",
          source: "https://legacy.example",
          gloss: "",
          fsrsState: "{}",
          deleted: false,
          clientTs: 10n,
          deviceId: "mac",
        },
      ],
      cursor: 2n,
    });
    const { port, calls } = syncPort({
      cardOps: [
        {
          client_id: "seldom",
          language: "en",
          lemma: "seldom",
          surface_form: "seldom",
          source_sentence: "s",
          source: "https://page.example",
          gloss: "",
          fsrs_state: "{}",
          deleted: false,
          client_ts: 5,
          device_id: "",
        },
      ],
      onApplyCards: () => 1,
    });
    const engine = new SyncEngine({ port, storage: fakeArea(v2("B")), clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    expect(f.pushCards.mock.calls[0]).not.toHaveProperty("0.cards.0.source");
    expect(calls.appliedCards[0][0].source).toBe("");
  });

  it("never sends the book a card was captured from either (add-lingua-reader)", async () => {
    const f = fakeClients();
    const { port } = syncPort({
      cardOps: [
        {
          client_id: "seldom",
          language: "en",
          lemma: "seldom",
          surface_form: "seldom",
          source_sentence: "They seldom spoke of it.",
          source: "The Hound of the Baskervilles · I: Mr. Sherlock Holmes",
          gloss: "",
          fsrs_state: "{}",
          deleted: false,
          client_ts: 5,
          device_id: "",
        },
      ],
    });
    const engine = new SyncEngine({ port, storage: fakeArea(v2("B")), clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    const [request] = f.pushCards.mock.calls[0] as unknown as [{ cards: Record<string, unknown>[] }];
    const pushed = request.cards[0];
    expect(pushed).not.toHaveProperty("source");
    expect(JSON.stringify(pushed, (_, v) => (typeof v === "bigint" ? String(v) : v))).not.toContain("Baskervilles");
    expect(pushed.sourceSentence).toBe("They seldom spoke of it.");
  });

  it("empties a device whose store predates the erasure before pushing anything", async () => {
    const f = fakeClients();
    f.getDataState.mockResolvedValue({ erasedAt: BigInt(MARK) });
    const { port, calls, asked } = syncPort(localOps);
    const storage = fakeArea({
      ...v2("OLD"),
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-card-cursor": 7,
      "cymbra-lingua-daily-v2": { 20000: { exposures: 1, wordsLearned: 1, reviews: 1 } },
    });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    expect(calls.resets).toBe(1);
    expect(new Set(asked)).toEqual(new Set(["en"])); // the wipe recalibrates English
    expect(f.pushOps).not.toHaveBeenCalled(); // nothing old goes back up
    expect(f.upsertDailyStats).not.toHaveBeenCalled();
    expect(storage.store["cymbra-lingua-daily-v2"]).toEqual({});
    expect(f.pullChanges).toHaveBeenCalledWith({ cursor: 0n }); // re-pulls from scratch
    expect(storage.store["cymbra-lingua-erased-at"]).toBe(MARK);
  });

  it("keeps calibrating English after an erasure for a reader of French who studied Spanish first", async () => {
    // The engine's full reset returns its reader to its native language, studying its first pack's
    // language alone: for an engine started on en-fr, English (generalise-lingua-native-language D5).
    const f = fakeClients();
    f.getDataState.mockResolvedValue({ erasedAt: BigInt(MARK) });
    const { port, calls, asked } = syncPort(localOps);
    await port.setStudiedLanguages(["es", "en"]);
    const reset = port.reset;
    port.reset = async () => {
      await reset();
      await port.setStudiedLanguages(["en"]);
    };
    const storage = fakeArea({ ...v2("OLD"), "cymbra-lingua-status-cursor": 42 });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    expect(calls.resets).toBe(1);
    expect(await port.nativeLanguage()).toBe("fr");
    expect(asked[0]).toBe("en"); // the wipe recalibrates English, not the Spanish studied before
  });

  it("adopts the mark without wiping on a device that never synced", async () => {
    const f = fakeClients();
    f.getDataState.mockResolvedValue({ erasedAt: BigInt(MARK) });
    const { port, calls } = syncPort({});
    const storage = fakeArea(v2("FRESH"));
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    expect(calls.resets).toBe(0);
    expect(storage.store["cymbra-lingua-erased-at"]).toBe(MARK);
  });

  it("does not wipe again once the mark is known, and dates new decisions after it", async () => {
    const f = fakeClients();
    f.getDataState.mockResolvedValue({ erasedAt: BigInt(MARK) });
    const { port, calls } = syncPort({
      ...localOps,
      levelOps: [{ language: "en", level: "B1", updated_at: 2_000 }],
    });
    const storage = fakeArea({ ...v2("NEW"), "cymbra-lingua-erased-at": MARK, "cymbra-lingua-status-cursor": 3 });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    expect(calls.resets).toBe(0);
    expect(f.pushOps).toHaveBeenCalledWith({
      ops: [expect.objectContaining({ lemma: "run", clientTs: BigInt(MARK + 1) })],
    });
    expect(f.pushOps).toHaveBeenCalledWith({
      ops: [],
      declaredLevels: [expect.objectContaining({ level: "B1", clientTs: BigInt(MARK + 1) })],
    });
  });

  it("keeps timestamps untouched when the account was never erased", async () => {
    const f = fakeClients();
    const { port } = syncPort(localOps);
    const engine = new SyncEngine({ port, storage: fakeArea(v2("B")), clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    expect(f.pushOps).toHaveBeenCalledWith({ ops: [expect.objectContaining({ clientTs: 1_000n })] });
  });

  it("erases on the server, then this device, and remembers the mark", async () => {
    const f = fakeClients();
    f.eraseMyData.mockResolvedValue({ erasedAt: BigInt(MARK) });
    const { port, calls } = syncPort(localOps);
    const storage = fakeArea({
      ...v2("OLD"),
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-daily-v2": { 20000: { exposures: 1, wordsLearned: 1, reviews: 1 } },
    });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.eraseAll();
    expect(f.eraseMyData).toHaveBeenCalledOnce();
    expect(calls.resets).toBe(1);
    expect(storage.store[ROOT_KEY]).toEqual({ v: 2, backup: "MERGED-BACKUP" }); // the emptied engine, saved
    expect(storage.store["cymbra-lingua-daily-v2"]).toEqual({});
    expect(storage.store["cymbra-lingua-daily-v3"]).toEqual({});
    expect(storage.store["cymbra-lingua-daily-v4"]).toEqual({});
    expect(storage.store["cymbra-lingua-status-cursor"]).toBe(0);
    expect(storage.store["cymbra-lingua-erased-at"]).toBe(MARK);
  });

  it("restores the stored backup before it resets: the engine resets to the reader's native language (task 4.5)", async () => {
    // The engine's reset keeps its own native language, the one of the backup it last restored: an
    // erasure queued behind a change of native language must not reset to the one the reader left.
    const f = fakeClients();
    f.eraseMyData.mockResolvedValue({ erasedAt: BigInt(MARK) });
    const { port } = syncPort(localOps);
    const order: string[] = [];
    const [restore, reset] = [port.restore, port.reset];
    port.restore = async (json) => {
      order.push(`restore ${json}`);
      await restore(json);
    };
    port.reset = async () => {
      order.push("reset");
      await reset();
    };
    await new SyncEngine({
      port,
      storage: fakeArea(v2("CURRENT")),
      clients: () => f.clients,
      deviceId: "d",
    }).eraseAll();
    expect(order).toEqual(["restore CURRENT", "reset"]);

    // A store with no backup yet has nothing to restore: the engine is reset as it is.
    order.length = 0;
    await new SyncEngine({ port, storage: fakeArea(), clients: () => f.clients, deviceId: "d" }).eraseAll();
    expect(order).toEqual(["reset"]);
  });

  it("erases all the same a backup that will not restore, and says so", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const f = fakeClients();
    f.eraseMyData.mockResolvedValue({ erasedAt: BigInt(MARK) });
    const { port, calls } = syncPort(localOps);
    port.restore = async () => {
      throw new Error("not a backup");
    };
    const storage = fakeArea(v2("{broken"));
    await new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" }).eraseAll();
    expect(calls.resets).toBe(1);
    expect(storage.store[ROOT_KEY]).toEqual({ v: 2, backup: "MERGED-BACKUP" });
    expect(storage.store["cymbra-lingua-erased-at"]).toBe(MARK);
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
    warn.mockRestore();
  });

  it("touches nothing locally when the server erasure fails", async () => {
    const f = fakeClients();
    f.eraseMyData.mockRejectedValue(new Error("unavailable"));
    const { port, calls } = syncPort(localOps);
    const storage = fakeArea({ ...v2("KEEP"), "cymbra-lingua-status-cursor": 42 });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await expect(engine.eraseAll()).rejects.toThrow();
    expect(calls.resets).toBe(0);
    expect(storage.store[ROOT_KEY]).toEqual({ v: 2, backup: "KEEP" });
    expect(storage.store["cymbra-lingua-status-cursor"]).toBe(42);
    expect(storage.store["cymbra-lingua-erased-at"]).toBeUndefined();
  });
});

describe("clearSyncCursors", () => {
  it("resets both pull cursors to 0 so the next sync re-pulls the full server state", async () => {
    const area = fakeArea({
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-card-cursor": 99,
    });
    await clearSyncCursors(area);
    expect(area.store["cymbra-lingua-status-cursor"]).toBe(0);
    expect(area.store["cymbra-lingua-card-cursor"]).toBe(0);
  });
});

describe("SyncEngine and the reader's languages (add-lingua-language-sync-client)", () => {
  const card = (language: string, lemma: string): CardOp => ({
    client_id: lemma,
    language,
    lemma,
    surface_form: lemma,
    source_sentence: "s",
    source: "",
    gloss: "",
    fsrs_state: "{}",
    deleted: false,
    client_ts: 1,
    device_id: "",
  });
  const wireCard = (language: string, clientId: string) => ({
    clientId,
    language,
    lemma: clientId,
    surfaceForm: clientId,
    sourceSentence: "s",
    source: "",
    gloss: "",
    fsrsState: "{}",
    deleted: false,
    clientTs: 5n,
    deviceId: "other",
  });
  const englishAndSpanish = async (): Promise<StudiedLanguage[]> => ["es", "en"];

  it("names the accepted languages, and files a pulled card under its own language", async () => {
    const f = fakeClients();
    f.pullCards.mockResolvedValueOnce({ cards: [wireCard("es", "son"), wireCard("", "son")], cursor: 9n });
    const { port, calls } = syncPort({ onApplyCards: (ops) => ops.length });
    const engine = new SyncEngine({
      port,
      storage: fakeArea(v2("B")),
      clients: () => f.clients,
      deviceId: "d",
      acceptedLanguages: englishAndSpanish,
    });

    await engine.sync();

    expect(f.pullCards).toHaveBeenCalledWith({ cursor: 0n, languages: ["es", "en"], anyGlossLanguage: true });
    expect(calls.appliedCards[0].map((op) => [op.language, op.client_id])).toEqual([
      ["es", "son"],
      ["en", "son"],
    ]);
  });

  it("does not apply a status or a level in a language the device does not accept", async () => {
    const f = fakeClients();
    f.pullChanges.mockResolvedValueOnce({
      changes: [
        { language: "es", lemma: "haber", status: "known", updatedAt: 4n, sequence: 1n },
        { language: "en", lemma: "city", status: "known", updatedAt: 4n, sequence: 2n },
        { language: "", lemma: "town", status: "learning", updatedAt: 4n, sequence: 3n },
      ],
      declaredLevels: [
        { language: "es", level: "A2", updatedAt: 4n, sequence: 4n },
        { language: "en", level: "B1", updatedAt: 4n, sequence: 5n },
      ],
      cursor: 5n,
    });
    const { port, calls } = syncPort({ onApplyStatuses: (c) => c.length, onApplyLevels: (c) => c.length });
    const storage = fakeArea(v2("B"));
    await new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" }).sync();

    expect(calls.appliedStatuses[0].map((c) => c.lemma)).toEqual(["city", "town"]);
    expect(calls.appliedLevels[0].map((l) => l.language)).toEqual(["en"]);
    // The cursor still moves: the Spanish records come back when Spanish is accepted.
    expect(storage.store["cymbra-lingua-status-cursor"]).toBe(5);
  });

  it("pushes each card with its language, a non-English one only to a server that keys cards by language", async () => {
    const withLanguages = fakeClients();
    withLanguages.getDataState.mockResolvedValue({ erasedAt: 0n, cardLanguage: true } as never);
    const { port } = syncPort({ cardOps: [card("en", "son"), card("es", "son")] });
    await new SyncEngine({
      port,
      storage: fakeArea(v2("B")),
      clients: () => withLanguages.clients,
      deviceId: "d",
    }).sync();
    const pushed = (withLanguages.pushCards.mock.calls[0] as unknown as [{ cards: { language: string }[] }])[0].cards;
    expect(pushed.map((c) => c.language)).toEqual(["en", "es"]);

    const older = fakeClients(); // a server that predates the field answers no card_language
    const { port: port2 } = syncPort({ cardOps: [card("en", "son"), card("es", "son")] });
    const res = await new SyncEngine({
      port: port2,
      storage: fakeArea(v2("B")),
      clients: () => older.clients,
      deviceId: "d",
    }).sync();
    const kept = (older.pushCards.mock.calls[0] as unknown as [{ cards: { language: string }[] }])[0].cards;
    expect(kept.map((c) => c.language)).toEqual(["en"]);
    expect(res?.pushedCards).toBe(1);
  });

  it("pulls again from the start when a language is added, and only then", async () => {
    // A device that has already pulled as a client that reads gloss languages: only the languages
    // decide here (the first pull of this build is its own scenario below).
    const cursors = {
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-card-cursor": 7,
      "cymbra-lingua-sync-labels": true,
    };

    // Updating the extension: no stored languages, English alone — nothing is pulled again.
    const updating = fakeClients();
    const keep = fakeArea({ ...v2("B"), ...cursors });
    await new SyncEngine({
      port: syncPort({}).port,
      storage: keep,
      clients: () => updating.clients,
      deviceId: "d",
    }).sync();
    expect(updating.pullChanges).toHaveBeenCalledWith({ cursor: 42n });
    expect(updating.pullCards).toHaveBeenCalledWith({ cursor: 7n, languages: ["en"], anyGlossLanguage: true });
    expect(keep.store["cymbra-lingua-sync-languages"]).toEqual(["en"]);

    // Adding Spanish: both cursors back to 0.
    const widening = fakeClients();
    const widen = fakeArea({ ...v2("B"), ...cursors, "cymbra-lingua-sync-languages": ["en"] });
    await new SyncEngine({
      port: syncPort({}).port,
      storage: widen,
      clients: () => widening.clients,
      deviceId: "d",
      acceptedLanguages: englishAndSpanish,
    }).sync();
    expect(widening.pullChanges).toHaveBeenCalledWith({ cursor: 0n });
    expect(widening.pullCards).toHaveBeenCalledWith({ cursor: 0n, languages: ["es", "en"], anyGlossLanguage: true });
    expect(widen.store["cymbra-lingua-sync-languages"]).toEqual(["es", "en"]);

    // Narrowing back to English: the cursors are kept.
    const narrowing = fakeClients();
    const narrow = fakeArea({ ...v2("B"), ...cursors, "cymbra-lingua-sync-languages": ["es", "en"] });
    await new SyncEngine({
      port: syncPort({}).port,
      storage: narrow,
      clients: () => narrowing.clients,
      deviceId: "d",
    }).sync();
    expect(narrowing.pullChanges).toHaveBeenCalledWith({ cursor: 42n });
  });

  it("saves the languages only once the sync has succeeded", async () => {
    const f = fakeClients();
    f.pullCards.mockRejectedValueOnce(new Error("offline"));
    const storage = fakeArea({ ...v2("B"), "cymbra-lingua-sync-languages": ["en"] });
    await expect(
      new SyncEngine({
        port: syncPort({}).port,
        storage,
        clients: () => f.clients,
        deviceId: "d",
        acceptedLanguages: englishAndSpanish,
      }).sync(),
    ).rejects.toThrow("offline");
    expect(storage.store["cymbra-lingua-sync-languages"]).toEqual(["en"]);
  });
});

describe("SyncEngine and the language of a gloss (add-lingua-native-language-sync-client)", () => {
  /** An English card; the engine labels its gloss only when it is not French. */
  const card = (lemma: string, gloss: string, glossLanguage?: string): CardOp => ({
    client_id: lemma,
    language: "en",
    lemma,
    surface_form: lemma,
    source_sentence: "s",
    source: "",
    gloss,
    ...(glossLanguage ? { gloss_language: glossLanguage } : {}),
    fsrs_state: "{}",
    deleted: false,
    client_ts: 1,
    device_id: "",
  });
  /** A server that stores the language of a gloss and of a day (add-lingua-native-language-server). */
  const labelling = () => {
    const f = fakeClients();
    f.getDataState.mockResolvedValue({ erasedAt: 0n, cardLanguage: true, languageLabels: true } as never);
    return f;
  };
  const pushed = (f: ReturnType<typeof fakeClients>, call = 0) =>
    (
      f.pushCards.mock.calls[call] as unknown as [{ cards: { clientId: string; glossLanguage: string }[] }]
    )[0].cards.map((c) => [c.clientId, c.glossLanguage]);
  /** A card glossed in French and one glossed in English. */
  const deck = [card("seldom", "rarement"), card("harbour", "a port", "en")];

  it("a server that stores the labels: both cards are pushed, each with its gloss language", async () => {
    const f = labelling();
    const { port } = syncPort({ cardOps: deck });
    const res = await new SyncEngine({
      port,
      storage: fakeArea(v2("B")),
      clients: () => f.clients,
      deviceId: "d",
    }).sync();

    expect(pushed(f)).toEqual([
      ["seldom", ""],
      ["harbour", "en"],
    ]);
    expect(res?.pushedCards).toBe(2);
  });

  it("a server that predates the labels: the French-glossed card alone is pushed, the English one at a later sync against a server that stores the labels", async () => {
    const older = fakeClients(); // its data state does not say it stores the labels
    const { port } = syncPort({ cardOps: deck });
    const storage = fakeArea(v2("B"));
    const held = await new SyncEngine({ port, storage, clients: () => older.clients, deviceId: "d" }).sync();
    expect(pushed(older)).toEqual([["seldom", ""]]);
    expect(held?.pushedCards).toBe(1);

    // The push is the whole deck each time: the card held stays local and goes with the next one.
    const newer = labelling();
    const later = await new SyncEngine({ port, storage, clients: () => newer.clients, deviceId: "d" }).sync();
    expect(pushed(newer)).toEqual([
      ["seldom", ""],
      ["harbour", "en"],
    ]);
    expect(later?.pushedCards).toBe(2);
  });

  it("a card pulled with its label is applied with it, and the pull says this build reads every label", async () => {
    const f = labelling();
    f.pullCards.mockResolvedValueOnce({
      cards: [
        { ...wire("harbour"), gloss: "a port", glossLanguage: "en" },
        { ...wire("seldom"), gloss: "rarement", glossLanguage: "" }, // as stored: the engine reads empty as French
      ],
      cursor: 2n,
    });
    const { port, calls } = syncPort({ onApplyCards: (ops) => ops.length });
    await new SyncEngine({ port, storage: fakeArea(v2("B")), clients: () => f.clients, deviceId: "d" }).sync();

    expect(f.pullCards).toHaveBeenCalledWith({ cursor: 0n, languages: ["en"], anyGlossLanguage: true });
    expect(calls.appliedCards[0].map((op) => [op.client_id, op.gloss, op.gloss_language])).toEqual([
      ["harbour", "a port", "en"],
      ["seldom", "rarement", ""],
    ]);
  });

  it("the first pull of this build pulls the cards from the start once, then from its cursor", async () => {
    // A device updated to this build: cursors past the cards the server withheld from it, no marker yet.
    const storage = fakeArea({
      ...v2("B"),
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-card-cursor": 7,
      "cymbra-lingua-sync-languages": ["en"],
    });
    const first = labelling();
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => first.clients, deviceId: "d" }).sync();
    expect(first.pullChanges).toHaveBeenCalledWith({ cursor: 42n }); // the statuses were never withheld
    expect(first.pullCards).toHaveBeenCalledWith({ cursor: 0n, languages: ["en"], anyGlossLanguage: true });
    expect(storage.store["cymbra-lingua-sync-labels"]).toBe(true);

    const second = labelling();
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => second.clients, deviceId: "d" }).sync();
    expect(second.pullCards).toHaveBeenCalledWith({ cursor: 9n, languages: ["en"], anyGlossLanguage: true });
  });

  it("marks the first pull only once the sync has succeeded", async () => {
    const f = labelling();
    f.pullCards.mockRejectedValueOnce(new Error("offline"));
    const storage = fakeArea({ ...v2("B"), "cymbra-lingua-card-cursor": 7 });
    await expect(
      new SyncEngine({ port: syncPort({}).port, storage, clients: () => f.clients, deviceId: "d" }).sync(),
    ).rejects.toThrow("offline");
    expect(storage.store["cymbra-lingua-sync-labels"]).toBeUndefined();
    expect(storage.store["cymbra-lingua-card-cursor"]).toBe(0); // the next sync pulls from the start again
  });

  it("the first pull of this build pulls the cards from the start against an older server too, and marks it", async () => {
    // The marker says this BUILD has pulled as a client that reads labels, whatever the server stores:
    // the card cursor is reset once, not once per server met.
    const storage = fakeArea({
      ...v2("B"),
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-card-cursor": 7,
      "cymbra-lingua-sync-languages": ["en"],
    });
    const older = fakeClients(); // its data state does not say it stores the labels
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => older.clients, deviceId: "d" }).sync();
    expect(older.pullChanges).toHaveBeenCalledWith({ cursor: 42n }); // the statuses were never withheld
    expect(older.pullCards).toHaveBeenCalledWith({ cursor: 0n, languages: ["en"], anyGlossLanguage: true });
    expect(storage.store["cymbra-lingua-sync-labels"]).toBe(true);
  });

  it("a server rolled back to one that stores no label echoes a held card without one: the echo is not applied, a French card's is", async () => {
    const { port, calls } = syncPort({ cardOps: deck, onApplyCards: (ops) => ops.length });
    const storage = fakeArea(v2("B"));
    const newer = labelling();
    await new SyncEngine({ port, storage, clients: () => newer.clients, deviceId: "d" }).sync();
    expect(pushed(newer)).toEqual([
      ["seldom", ""],
      ["harbour", "en"],
    ]);

    // Rolled back, the server keeps the English-glossed card with no label and echoes it as such — French
    // to the engine — dated no earlier than the device's copy, which last-writer-wins would apply.
    const older = fakeClients();
    older.pullCards.mockResolvedValueOnce({
      cards: [
        { ...wire("harbour"), gloss: "a port", glossLanguage: "", clientTs: 1n, deviceId: "d" },
        { ...wire("seldom"), gloss: "rarement", glossLanguage: "", clientTs: 1n, deviceId: "d" },
      ],
      cursor: 2n,
    });
    await new SyncEngine({ port, storage, clients: () => older.clients, deviceId: "d" }).sync();

    expect(pushed(older)).toEqual([["seldom", ""]]); // the English-glossed card is held, as before
    // Its echo is dropped — a label from a server that stores none is not information — and the French
    // card's is applied; the cursor passes both, nothing is re-pulled.
    expect(calls.appliedCards.at(-1)?.map((op) => [op.client_id, op.gloss_language])).toEqual([["seldom", ""]]);
    expect(storage.store["cymbra-lingua-card-cursor"]).toBe(2);
  });

  it("a French card is pushed to a server that predates the labels, as before", async () => {
    const older = fakeClients();
    const { port } = syncPort({ cardOps: [card("seldom", "rarement"), card("dwell", "demeurer", "fr")] });
    const res = await new SyncEngine({
      port,
      storage: fakeArea(v2("B")),
      clients: () => older.clients,
      deviceId: "d",
    }).sync();

    expect(pushed(older)).toEqual([
      ["seldom", ""],
      ["dwell", "fr"],
    ]);
    expect(res?.pushedCards).toBe(2);
  });

  function wire(clientId: string) {
    return {
      clientId,
      language: "en",
      lemma: clientId,
      surfaceForm: clientId,
      sourceSentence: "s",
      source: "",
      gloss: "",
      glossLanguage: "",
      fsrsState: "{}",
      deleted: false,
      clientTs: 5n,
      deviceId: "other",
    };
  }
});

describe("SyncEngine and the native language of a day (add-lingua-native-language-sync-client)", () => {
  const TODAY = 20_650;
  const COUNTS = { exposures: 12, unknownSeen: 4, wordsLearned: 3, reviews: 5 };
  /** Today's English statistic, counted under `native`, in the shape this build writes. */
  const today = (native: string) => ({ "cymbra-lingua-daily-v4": { [TODAY]: { en: { ...COUNTS, native } } } });
  const labelling = () => {
    const f = fakeClients();
    f.getDataState.mockResolvedValue({ erasedAt: 0n, cardLanguage: true, languageLabels: true } as never);
    return f;
  };
  const sent = (f: ReturnType<typeof fakeClients>, call = 0) =>
    (f.upsertDailyStats.mock.calls[call] as unknown as [{ stats: Record<string, unknown>[] }])[0].stats;
  const row = (nativeLanguage: string, day = TODAY, language = "en") => ({
    day,
    language,
    deviceId: "d",
    exposures: COUNTS.exposures,
    unknownSeen: COUNTS.unknownSeen,
    wordsLearned: COUNTS.wordsLearned,
    reviewsDone: COUNTS.reviews,
    nativeLanguage,
  });

  it("a French-native device: today's English statistic carries fr, and the row is as before", async () => {
    const older = fakeClients(); // whatever the server stores, a French day goes
    const storage = fakeArea({ ...v2("B"), ...today("fr") });
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => older.clients, deviceId: "d" }).sync();
    expect(sent(older)).toEqual([row("fr")]);
  });

  it("a Spanish-native device: today's English statistic carries es, to a server that stores the labels", async () => {
    const f = labelling();
    const storage = fakeArea({ ...v2("B"), ...today("es") });
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => f.clients, deviceId: "d" }).sync();
    expect(sent(f)).toEqual([row("es")]);
  });

  it("a server that predates the labels: a Spanish-native day is held, and sent at a later sync against a server that stores them", async () => {
    const storage = fakeArea({
      ...v2("B"),
      "cymbra-lingua-daily-v4": {
        [TODAY - 1]: { en: { ...COUNTS, native: "fr" } }, // counted before the native language changed
        [TODAY]: { en: { ...COUNTS, native: "es" } },
      },
    });
    const older = fakeClients();
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => older.clients, deviceId: "d" }).sync();
    expect(sent(older)).toEqual([row("fr", TODAY - 1)]);

    const newer = labelling();
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => newer.clients, deviceId: "d" }).sync();
    expect(sent(newer)).toEqual([row("fr", TODAY - 1), row("es")]);
  });

  it("holds nothing but the day: a Spanish-native device with no French day sends no statistic to an older server", async () => {
    const older = fakeClients();
    const storage = fakeArea({ ...v2("B"), ...today("es") });
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => older.clients, deviceId: "d" }).sync();
    expect(older.upsertDailyStats).not.toHaveBeenCalled();
  });

  it("statistics written before the label, per studied language (v3), are sent once as French, their counts unchanged", async () => {
    const older = fakeClients();
    const storage = fakeArea({ ...v2("B"), "cymbra-lingua-daily-v3": { [TODAY]: { en: COUNTS, es: COUNTS } } });
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => older.clients, deviceId: "d" }).sync();
    expect(sent(older)).toEqual([row("fr"), row("fr", TODAY, "es")]);
  });

  it("statistics written before the label, per day only (v2 alone), are sent as English and French", async () => {
    const older = fakeClients();
    const storage = fakeArea({ ...v2("B"), "cymbra-lingua-daily-v2": { [TODAY]: COUNTS } });
    await new SyncEngine({ port: syncPort({}).port, storage, clients: () => older.clients, deviceId: "d" }).sync();
    expect(sent(older)).toEqual([row("fr")]);
  });
});
