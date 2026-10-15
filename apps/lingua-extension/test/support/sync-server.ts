import type { CardOp, StatusChangeIn, StatusOp } from "@/analyzer/port.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import type { SyncClients } from "@/sync/sync.ts";
import { type FakePort, makeFakePort } from "../helpers.ts";

// Test doubles for the synchronisation (add-lingua-app-deck-sync): an in-memory Lingua server applying
// the server's rules — last-write-wins on a newer `updated_at`, or an equal one and a greater device id;
// ops dated before the account's erasure mark refused; daily statistics replaced by (day, language,
// device) — and an engine double that holds cards and statuses under the engine's own rule (an
// incoming op wins when it is not older). Hand-written: the generated Connect clients and the WASM
// engine have no generated mocks, and these carry behaviour across hosts, not expectations.

interface CardRow {
  card: CardOp;
  seq: number;
}
interface StatusRow {
  status: StatusOp;
  device: string;
  seq: number;
}
export interface StatRow {
  day: number;
  language: string;
  deviceId: string;
  reviewsDone: number;
  wordsLearned: number;
}

const wins = (rowTs: number, rowDevice: string, ts: number, device: string): boolean =>
  ts > rowTs || (ts === rowTs && device > rowDevice);

export interface FakeSyncServer {
  clients: SyncClients;
  /** Every RPC by name, in order. */
  calls: string[];
  /** The cards as the server holds them. */
  cards(): Map<string, CardOp & { deviceId: string }>;
  statuses(): Map<string, StatusOp>;
  /** The statistics summed across devices for a day and language, as `GetStats` reads them. */
  reviews(day: number, language: string): number;
  stats: StatRow[];
  /** « Effacer mes données Lingua » from another device: rows gone, the mark set. */
  erase(at: number): void;
  /** Every RPC fails, as offline. */
  offline: boolean;
  /** Hold the pulls until released (a slow network). */
  holdPulls(): () => void;
}

export function fakeSyncServer(): FakeSyncServer {
  const cards = new Map<string, CardRow>();
  const statuses = new Map<string, StatusRow>();
  let stats: StatRow[] = [];
  let seq = 0;
  let erasedAt = 0;
  const calls: string[] = [];
  let held: Promise<void> | null = null;
  const server: FakeSyncServer = {
    clients: null as unknown as SyncClients,
    calls,
    offline: false,
    stats,
    cards: () => new Map([...cards].map(([k, r]) => [k, { ...r.card, deviceId: r.card.device_id }])),
    statuses: () => new Map([...statuses].map(([k, r]) => [k, r.status])),
    reviews: (day, language) =>
      stats.filter((s) => s.day === day && s.language === language).reduce((n, s) => n + s.reviewsDone, 0),
    erase(at) {
      erasedAt = at;
      cards.clear();
      statuses.clear();
      stats = [];
      server.stats = stats;
    },
    holdPulls() {
      let release: () => void = () => {};
      held = new Promise((resolve) => (release = resolve));
      return () => {
        held = null;
        release();
      };
    },
  };
  const rpc =
    <A, R>(name: string, fn: (a: A) => R | Promise<R>) =>
    async (a: A): Promise<R> => {
      calls.push(name);
      if (server.offline) throw Object.assign(new Error("unavailable"), { code: 14 });
      return fn(a);
    };
  interface WireStatus {
    language: string;
    lemma: string;
    status: string;
    provenance: string;
    clientTs: bigint;
    deviceId: string;
  }
  interface WireCard {
    clientId: string;
    language: string;
    lemma: string;
    surfaceForm: string;
    sourceSentence: string;
    gloss: string;
    glossLanguage: string;
    fsrsState: string;
    deleted: boolean;
    clientTs: bigint;
    deviceId: string;
  }
  server.clients = {
    knownWords: {
      pushOps: rpc("pushOps", ({ ops }: { ops: WireStatus[] }) => {
        for (const o of ops) {
          const ts = Number(o.clientTs);
          if (ts < erasedAt) continue;
          const key = `${o.language}:${o.lemma}`;
          const row = statuses.get(key);
          if (row && !wins(row.status.updated_at, row.device, ts, o.deviceId)) continue;
          statuses.set(key, {
            status: {
              language: o.language,
              lemma: o.lemma,
              status: o.status,
              provenance: o.provenance,
              updated_at: ts,
            },
            device: o.deviceId,
            seq: ++seq,
          });
        }
        return { applied: 0n, cursor: BigInt(seq) };
      }),
      pullChanges: rpc("pullChanges", async ({ cursor }: { cursor: bigint }) => {
        await held;
        const changes = [...statuses.values()]
          .filter((r) => r.seq > Number(cursor))
          .sort((a, b) => a.seq - b.seq)
          .map((r) => ({
            language: r.status.language,
            lemma: r.status.lemma,
            status: r.status.status,
            provenance: r.status.provenance,
            updatedAt: BigInt(r.status.updated_at),
            sequence: BigInt(r.seq),
          }));
        return { changes, cursor: BigInt(seq), declaredLevels: [] };
      }),
    },
    deck: {
      pushCards: rpc("pushCards", ({ cards: pushed }: { cards: WireCard[] }) => {
        for (const c of pushed) {
          const ts = Number(c.clientTs);
          if (ts < erasedAt) continue;
          const row = cards.get(c.clientId);
          if (row && !wins(row.card.client_ts, row.card.device_id, ts, c.deviceId)) continue;
          cards.set(c.clientId, {
            card: {
              client_id: c.clientId,
              language: c.language,
              lemma: c.lemma,
              surface_form: c.surfaceForm,
              source_sentence: c.sourceSentence,
              source: "",
              gloss: c.gloss,
              fsrs_state: c.fsrsState,
              deleted: c.deleted,
              client_ts: ts,
              device_id: c.deviceId,
            },
            seq: ++seq,
          });
        }
        return { applied: 0n, cursor: BigInt(seq) };
      }),
      pullCards: rpc("pullCards", async ({ cursor }: { cursor: bigint }) => {
        await held;
        const out = [...cards.values()]
          .filter((r) => r.seq > Number(cursor))
          .sort((a, b) => a.seq - b.seq)
          .map(({ card: c }) => ({
            clientId: c.client_id,
            language: c.language,
            lemma: c.lemma,
            surfaceForm: c.surface_form,
            sourceSentence: c.source_sentence,
            source: "",
            gloss: c.gloss,
            glossLanguage: "",
            fsrsState: c.fsrs_state,
            deleted: c.deleted,
            clientTs: BigInt(c.client_ts),
            deviceId: c.device_id,
          }));
        return { cards: out, cursor: BigInt(seq) };
      }),
    },
    stats: {
      upsertDailyStats: rpc(
        "upsertDailyStats",
        ({
          stats: rows,
        }: {
          stats: { day: number; language: string; deviceId: string; reviewsDone: number; wordsLearned: number }[];
        }) => {
          for (const s of rows) {
            const at = stats.findIndex(
              (r) => r.day === s.day && r.language === s.language && r.deviceId === s.deviceId,
            );
            const row = {
              day: s.day,
              language: s.language,
              deviceId: s.deviceId,
              reviewsDone: s.reviewsDone,
              wordsLearned: s.wordsLearned,
            };
            if (at >= 0) stats[at] = row;
            else stats.push(row);
          }
          return { upserted: 0n };
        },
      ),
    },
    data: {
      getDataState: rpc("getDataState", () => ({
        erasedAt: BigInt(erasedAt),
        cardLanguage: true,
        languageLabels: true,
      })),
      eraseMyData: rpc("eraseMyData", () => {
        server.erase(Date.now());
        return { erasedAt: BigInt(erasedAt) };
      }),
    },
  } as unknown as SyncClients;
  return server;
}

interface EngineState {
  cards: Record<string, CardOp>;
  statuses: Record<string, StatusOp>;
}

/** An engine double holding cards and statuses under the engine's rule, its backup their JSON. */
export function lwwPort(): FakePort & { state: () => EngineState } {
  const { port } = makeFakePort();
  let state: EngineState = { cards: {}, statuses: {} };
  const read = (backup: string): EngineState => {
    try {
      const parsed = JSON.parse(backup) as Partial<EngineState>;
      return { cards: parsed.cards ?? {}, statuses: parsed.statuses ?? {} };
    } catch {
      return { cards: {}, statuses: {} };
    }
  };
  port.backup = async () => JSON.stringify(state);
  port.restore = async (backup) => {
    state = read(backup);
  };
  port.reset = async () => {
    state = { cards: {}, statuses: {} };
  };
  port.exportCardOps = async () => Object.values(state.cards).map((c) => ({ ...c }));
  port.exportStatusOps = async () => Object.values(state.statuses).map((s) => ({ ...s }));
  port.exportDeclaredLevels = async () => [];
  port.applyDeclaredLevelChanges = async () => 0;
  port.applyCardOps = async (ops) => {
    let changed = 0;
    for (const op of ops) {
      const existing = state.cards[op.client_id];
      if (existing && op.client_ts < existing.client_ts) continue;
      const next: CardOp = { ...op, device_id: "", source: existing?.source || op.source };
      if (JSON.stringify(existing) !== JSON.stringify(next)) changed += 1;
      state.cards[op.client_id] = next;
    }
    return changed;
  };
  port.applyStatusChanges = async (changes: StatusChangeIn[]) => {
    let changed = 0;
    for (const c of changes) {
      const key = `${c.language}:${c.lemma}`;
      const existing = state.statuses[key];
      if (existing && c.updated_at < existing.updated_at) continue;
      const next: StatusOp = {
        language: c.language,
        lemma: c.lemma,
        status: c.status,
        provenance: c.provenance ?? "manual",
        updated_at: c.updated_at,
      };
      if (JSON.stringify(existing) !== JSON.stringify(next)) changed += 1;
      state.statuses[key] = next;
    }
    return changed;
  };
  return Object.assign(port, { state: () => state });
}

/** A card a reader captured. */
export function capturedCard(lemma: string, ts: number): CardOp {
  return {
    client_id: lemma,
    language: "en",
    lemma,
    surface_form: lemma,
    source_sentence: `A ${lemma}.`,
    source: "https://example.com",
    gloss: "",
    fsrs_state: JSON.stringify({ reps: 0 }),
    deleted: false,
    client_ts: ts,
    device_id: "",
  };
}

/** The review's answer to a card: its whole state moved on, dated `ts`, then saved as the page does. */
export async function answer(port: FakePort & { state: () => EngineState }, lemma: string, ts: number): Promise<void> {
  const card = port.state().cards[lemma];
  if (!card) throw new Error(`no card ${lemma}`);
  const reps = (JSON.parse(card.fsrs_state) as { reps: number }).reps + 1;
  port.state().cards[lemma] = { ...card, fsrs_state: JSON.stringify({ reps }), client_ts: ts };
}

/** A plain storage area. */
export function memoryArea(seed: Record<string, unknown> = {}): AsyncStorageArea & { items: Record<string, unknown> } {
  const items: Record<string, unknown> = { ...seed };
  return {
    items,
    async get(keys) {
      const list = keys == null ? Object.keys(items) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in items).map((k) => [k, items[k]]));
    },
    async set(next) {
      Object.assign(items, next);
    },
  };
}

/** The page's localStorage, in memory. */
export function memoryLocal(): Storage & { items: Map<string, string> } {
  const items = new Map<string, string>();
  return {
    items,
    get length() {
      return items.size;
    },
    clear: () => items.clear(),
    getItem: (k) => items.get(k) ?? null,
    key: (i) => [...items.keys()][i] ?? null,
    removeItem: (k) => void items.delete(k),
    setItem: (k, v) => void items.set(k, v),
  };
}
