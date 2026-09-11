import type { Client } from "@connectrpc/connect";
import type { CardOp, LinguaPort, StatusChangeIn } from "../analyzer/port.ts";
import type { DeckService } from "../gen/deck_pb.ts";
import type { KnownWordsService } from "../gen/known_words_pb.ts";
import { type AsyncStorageArea, loadStored, saveBackup } from "../state/storage.ts";

// The extension sync engine (add-lingua-connected-clients §2). When signed in it pushes
// the local word-statuses and deck to the cymbra.lingua.v1 services and pulls the merged
// changes back, keeping the local store the display authority (design D4). It owns a
// DEDICATED lingua engine (not the reading one) that it hydrates from the persisted
// backup before each run and writes back after a pull — so the merge is a plain
// restore → apply → backup, and other contexts pick it up via storage.onChanged.
//
// Wire timestamps are epoch millis (int64 → bigint in the generated messages). Cursors
// are the server's monotonic sequence, stored as plain numbers (JSON-safe) and widened
// to bigint per call. The push is a full idempotent upload (LWW server-side; replaying a
// batch is a no-op), which makes first sign-in just a large push — not a special case.

const DEVICE_KEY = "cymbra-lingua-device";
const STATUS_CURSOR_KEY = "cymbra-lingua-status-cursor";
const CARD_CURSOR_KEY = "cymbra-lingua-card-cursor";
/** Max ops per request (bounded batches; the server resumes by outbox offset). */
const BATCH = 500;

export interface SyncClients {
  knownWords: Client<typeof KnownWordsService>;
  deck: Client<typeof DeckService>;
}

export interface SyncDeps {
  /** A dedicated engine, hydrated from the local backup each sync. */
  port: LinguaPort;
  /** chrome.storage.local-backed area (backup + cursors + device id). */
  storage: AsyncStorageArea;
  /** The gRPC-web clients, resolved lazily (only touched while signed in). */
  clients: () => SyncClients;
  /** Stable per-install device id (LWW tie-break). */
  deviceId: string;
}

export interface SyncResult {
  pushedStatuses: number;
  pushedCards: number;
  pulled: number;
}

export class SyncEngine {
  constructor(private readonly deps: SyncDeps) {}

  /**
   * One full exchange: hydrate from the local backup, push the local statuses + deck,
   * pull remote changes and apply them, and — only if a pull changed something — persist
   * the merged backup (which propagates to every context). Returns null when there is no
   * local state yet. The caller guarantees a signed-in session.
   */
  async sync(): Promise<SyncResult | null> {
    const stored = await loadStored(this.deps.storage);
    if (stored.kind !== "v2") return null; // nothing local to sync yet
    await this.deps.port.restore(stored.backup);

    const pushedStatuses = await this.pushStatuses();
    const pushedCards = await this.pushCards();

    // Fetch pulls WITHOUT applying, so apply + persist happen together at the end.
    const statuses = await this.fetchStatuses();
    const cards = await this.fetchCards();

    let pulled = 0;
    if (statuses.changes.length > 0 || cards.ops.length > 0) {
      // Re-hydrate from the LATEST backup before applying + persisting, so a local
      // mutation made in another context during the (slow) push/pull round-trip is not
      // clobbered by writing back the stale start-of-sync snapshot. Applies are
      // idempotent LWW, so layering the pulled changes onto the fresh state is safe.
      const latest = await loadStored(this.deps.storage);
      if (latest.kind === "v2") await this.deps.port.restore(latest.backup);
      if (statuses.changes.length > 0) pulled += await this.deps.port.applyStatusChanges(statuses.changes);
      if (cards.ops.length > 0) pulled += await this.deps.port.applyCardOps(cards.ops);
      if (pulled > 0) await saveBackup(this.deps.storage, await this.deps.port.backup());
    }

    // Advance the cursors only AFTER the merged state is durably saved: an interruption
    // before saveBackup leaves the cursor unmoved, so the changes are simply re-pulled
    // (and re-applied idempotently) next time — never dropped.
    await this.saveCursor(STATUS_CURSOR_KEY, statuses.cursor);
    await this.saveCursor(CARD_CURSOR_KEY, cards.cursor);
    return { pushedStatuses, pushedCards, pulled };
  }

  private async pushStatuses(): Promise<number> {
    const ops = await this.deps.port.exportStatusOps();
    for (const batch of chunk(ops, BATCH)) {
      await this.deps.clients().knownWords.pushOps({
        ops: batch.map((o) => ({
          language: o.language,
          lemma: o.lemma,
          status: o.status,
          provenance: o.provenance,
          clientTs: BigInt(o.updated_at),
          deviceId: this.deps.deviceId,
        })),
      });
    }
    return ops.length;
  }

  private async pushCards(): Promise<number> {
    const ops = await this.deps.port.exportCardOps();
    for (const batch of chunk(ops, BATCH)) {
      await this.deps.clients().deck.pushCards({
        cards: batch.map((c) => ({
          clientId: c.client_id,
          lemma: c.lemma,
          surfaceForm: c.surface_form,
          sourceSentence: c.source_sentence,
          source: c.source,
          gloss: c.gloss,
          fsrsState: c.fsrs_state,
          deleted: c.deleted,
          clientTs: BigInt(c.client_ts),
          deviceId: this.deps.deviceId,
        })),
      });
    }
    return ops.length;
  }

  /** Fetch status changes after the stored cursor (no apply, no cursor write). */
  private async fetchStatuses(): Promise<{ changes: StatusChangeIn[]; cursor: number }> {
    const cursor = await this.loadCursor(STATUS_CURSOR_KEY);
    const res = await this.deps.clients().knownWords.pullChanges({ cursor: BigInt(cursor) });
    return {
      changes: res.changes.map((c) => ({
        language: c.language,
        lemma: c.lemma,
        status: c.status,
        updated_at: Number(c.updatedAt),
      })),
      cursor: Number(res.cursor),
    };
  }

  /** Fetch card ops after the stored cursor (no apply, no cursor write). */
  private async fetchCards(): Promise<{ ops: CardOp[]; cursor: number }> {
    const cursor = await this.loadCursor(CARD_CURSOR_KEY);
    const res = await this.deps.clients().deck.pullCards({ cursor: BigInt(cursor) });
    return {
      ops: res.cards.map((c) => ({
        client_id: c.clientId,
        language: "en",
        lemma: c.lemma,
        surface_form: c.surfaceForm,
        source_sentence: c.sourceSentence,
        source: c.source,
        gloss: c.gloss,
        fsrs_state: c.fsrsState,
        deleted: c.deleted,
        client_ts: Number(c.clientTs),
        device_id: c.deviceId,
      })),
      cursor: Number(res.cursor),
    };
  }

  private async loadCursor(key: string): Promise<number> {
    const got = await this.deps.storage.get(key);
    return typeof got[key] === "number" ? (got[key] as number) : 0;
  }

  private async saveCursor(key: string, cursor: number): Promise<void> {
    await this.deps.storage.set({ [key]: cursor });
  }
}

function chunk<T>(arr: T[], size: number): T[][] {
  const out: T[][] = [];
  for (let i = 0; i < arr.length; i += size) out.push(arr.slice(i, i + size));
  return out;
}

/** The stable per-install device id (LWW tie-break), generated once and stored. */
export async function getOrCreateDeviceId(storage: AsyncStorageArea): Promise<string> {
  const got = await storage.get(DEVICE_KEY);
  const existing = got[DEVICE_KEY];
  if (typeof existing === "string" && existing.length > 0) return existing;
  const id = crypto.randomUUID();
  await storage.set({ [DEVICE_KEY]: id });
  return id;
}
