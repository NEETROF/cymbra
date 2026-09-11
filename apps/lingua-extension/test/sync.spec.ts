import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CardOp, StatusChangeIn, StatusOp } from "@/analyzer/port.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { getOrCreateDeviceId, SyncEngine, type SyncClients } from "@/sync/sync.ts";
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
  onApplyStatuses?: (c: StatusChangeIn[]) => number;
  onApplyCards?: (c: CardOp[]) => number;
}) {
  const { port } = makeFakePort();
  const calls = { restored: [] as string[], appliedStatuses: [] as StatusChangeIn[][], appliedCards: [] as CardOp[][] };
  port.restore = async (j) => void calls.restored.push(j);
  port.backup = async () => "MERGED-BACKUP";
  port.exportStatusOps = async () => over.statusOps ?? [];
  port.exportCardOps = async () => over.cardOps ?? [];
  port.applyStatusChanges = async (c) => {
    calls.appliedStatuses.push(c);
    return over.onApplyStatuses?.(c) ?? 0;
  };
  port.applyCardOps = async (c) => {
    calls.appliedCards.push(c);
    return over.onApplyCards?.(c) ?? 0;
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

function fakeClients() {
  const pushOps = vi.fn(async () => ({ applied: 0n, cursor: 0n }));
  const pushCards = vi.fn(async () => ({ applied: 0n, cursor: 0n }));
  const pullChanges = vi.fn(async () => ({ changes: [] as WireChange[], cursor: 7n }));
  const pullCards = vi.fn(async () => ({ cards: [] as WireCard[], cursor: 9n }));
  const clients = { knownWords: { pushOps, pullChanges }, deck: { pushCards, pullCards } } as unknown as SyncClients;
  return { clients, pushOps, pushCards, pullChanges, pullCards };
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
          source: "https://x",
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

  it("applies pulled changes onto the LATEST backup, not the start-of-sync snapshot (no clobber)", async () => {
    const storage = fakeArea(v2("B0"));
    const f = fakeClients();
    // A concurrent local mutation lands (another context persists) during the pull.
    f.pullChanges.mockImplementationOnce(async () => {
      await storage.set(v2("B1-concurrent"));
      return { changes: [{ language: "en", lemma: "city", status: "known", updatedAt: 5n, sequence: 1n }], cursor: 3n };
    });
    const { port, calls } = syncPort({ onApplyStatuses: () => 1 });
    const engine = new SyncEngine({ port, storage, clients: () => f.clients, deviceId: "d" });
    await engine.sync();
    // The pulled change is applied on top of the re-loaded latest backup, not B0.
    expect(calls.restored).toEqual(["B0", "B1-concurrent"]);
  });
});
