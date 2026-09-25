import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CardOp, DeclaredLevelOp, StatusChangeIn, StatusOp } from "@/analyzer/port.ts";
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
  const { port } = makeFakePort();
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
  return { port, calls };
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
          gloss: "rarement",
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
          lemma: "seldom",
          surfaceForm: "seldom",
          sourceSentence: "s",
          gloss: "rarement",
          fsrsState: "{}",
          deleted: false,
          clientTs: 1800n,
          deviceId: "dev-1",
        },
      ],
    });
    expect(f.pullChanges).toHaveBeenCalledWith({ cursor: 0n });
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

  it("upserts the local daily stats (mapping reviews→reviewsDone + device id)", async () => {
    const { port } = syncPort({});
    const f = fakeClients();
    const storage = fakeArea({
      ...v2("BACKUP"),
      "cymbra-lingua-daily": { 20000: { exposures: 12, wordsLearned: 3, reviews: 5 } },
    });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "dev-2" });
    await engine.sync();
    expect(f.upsertDailyStats).toHaveBeenCalledWith({
      stats: [{ day: 20000, language: "en", deviceId: "dev-2", exposures: 12, wordsLearned: 3, reviewsDone: 5 }],
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
    const { port, calls } = syncPort(localOps);
    const storage = fakeArea({
      ...v2("OLD"),
      "cymbra-lingua-status-cursor": 42,
      "cymbra-lingua-card-cursor": 7,
      "cymbra-lingua-daily": { 20000: { exposures: 1, wordsLearned: 1, reviews: 1 } },
    });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    expect(calls.resets).toBe(1);
    expect(f.pushOps).not.toHaveBeenCalled(); // nothing old goes back up
    expect(f.upsertDailyStats).not.toHaveBeenCalled();
    expect(storage.store["cymbra-lingua-daily"]).toEqual({});
    expect(f.pullChanges).toHaveBeenCalledWith({ cursor: 0n }); // re-pulls from scratch
    expect(storage.store["cymbra-lingua-erased-at"]).toBe(MARK);
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
      "cymbra-lingua-daily": { 20000: { exposures: 1, wordsLearned: 1, reviews: 1 } },
    });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.eraseAll();
    expect(f.eraseMyData).toHaveBeenCalledOnce();
    expect(calls.resets).toBe(1);
    expect(storage.store[ROOT_KEY]).toEqual({ v: 2, backup: "MERGED-BACKUP" }); // the emptied engine, saved
    expect(storage.store["cymbra-lingua-daily"]).toEqual({});
    expect(storage.store["cymbra-lingua-status-cursor"]).toBe(0);
    expect(storage.store["cymbra-lingua-erased-at"]).toBe(MARK);
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
