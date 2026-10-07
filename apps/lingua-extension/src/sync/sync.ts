import type { Client } from "@connectrpc/connect";
import type { CardOp, DeclaredLevelOp, LinguaPort, StatusChangeIn } from "../analyzer/port.ts";
import type { DeckService } from "../gen/deck_pb.ts";
import type { KnownWordsService } from "../gen/known_words_pb.ts";
import type { LinguaDataService } from "../gen/lingua_data_pb.ts";
import type { StatsService } from "../gen/stats_pb.ts";
import { clearDailyStats, loadDailyStats } from "../state/dailystats.ts";
import { type AsyncStorageArea, DEFAULT_CALIBRATION, loadStored, saveBackup } from "../state/storage.ts";
import { acceptedLanguages, readingLanguage } from "../analyzer/pairs.ts";
import type { StudiedLanguage } from "../analyzer/types.ts";

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
//
// Privacy controls (add-lingua-privacy-controls): a card goes up without the page it was
// captured from, and every sync first reads the account's erasure mark — a device whose
// store predates a « Effacer mes données Lingua » empties itself before pushing, so it
// cannot bring the erased data back (design D2/D3).

const DEVICE_KEY = "cymbra-lingua-device";
const STATUS_CURSOR_KEY = "cymbra-lingua-status-cursor";
const CARD_CURSOR_KEY = "cymbra-lingua-card-cursor";
/** The account erasure mark this device last acted on (server epoch millis). */
const ERASED_AT_KEY = "cymbra-lingua-erased-at";
/**
 * The languages of this device's last successful sync (add-lingua-language-sync-client D4). Absent:
 * English alone, what every device accepted before cards carried a language.
 */
const SYNC_LANGUAGES_KEY = "cymbra-lingua-sync-languages";
/** Max ops per request (bounded batches; the server resumes by outbox offset). */
const BATCH = 500;

export interface SyncClients {
  knownWords: Client<typeof KnownWordsService>;
  deck: Client<typeof DeckService>;
  stats: Client<typeof StatsService>;
  data: Client<typeof LinguaDataService>;
}

export interface SyncDeps {
  /** A dedicated engine, hydrated from the local backup each sync. */
  port: LinguaPort;
  /** The reader's durable store (backup + cursors + device id), owned by the background. */
  storage: AsyncStorageArea;
  /** The gRPC-web clients, resolved lazily (only touched while signed in). */
  clients: () => SyncClients;
  /** Stable per-install device id (LWW tie-break). */
  deviceId: string;
  /** The languages this device accepts (design D1); default: the reader's, as the package ships them. */
  acceptedLanguages?: () => Promise<StudiedLanguage[]>;
}

export interface SyncResult {
  pushedStatuses: number;
  pushedCards: number;
  pulled: number;
}

export class SyncEngine {
  /** The account's erasure mark as of the current sync (0 = never erased). */
  private erasedAt = 0;
  /** Whether the server keys cards by language, as of the current sync's data state (design D3). */
  private cardLanguage = false;

  constructor(private readonly deps: SyncDeps) {}

  /**
   * One full exchange: hydrate from the local backup, push the local statuses + deck,
   * pull remote changes and apply them, and — only if a pull changed something — persist
   * the merged backup (which propagates to every context). Returns null when there is no
   * local state yet. The caller guarantees a signed-in session.
   */
  async sync(): Promise<SyncResult | null> {
    await this.checkErasure();
    const stored = await loadStored(this.deps.storage);
    if (stored.kind !== "v2") return null; // nothing local to sync yet
    await this.deps.port.restore(stored.backup);
    // The reader's languages came with the backup: what this device accepts, and whether that grew.
    const languages = await (this.deps.acceptedLanguages?.() ?? acceptedLanguages(this.deps.port));
    await this.widen(languages);

    const pushedStatuses = await this.pushStatuses();
    await this.pushDeclaredLevels();
    const pushedCards = await this.pushCards();
    await this.pushStats();

    // Fetch pulls WITHOUT applying, so apply + persist happen together at the end.
    const statuses = await this.fetchStatuses(languages);
    const cards = await this.fetchCards(languages);

    let pulled = 0;
    if (statuses.changes.length > 0 || statuses.declaredLevels.length > 0 || cards.ops.length > 0) {
      // Re-hydrate from the LATEST backup before applying + persisting, so a local
      // mutation made in another context during the (slow) push/pull round-trip is not
      // clobbered by writing back the stale start-of-sync snapshot. Applies are
      // idempotent LWW, so layering the pulled changes onto the fresh state is safe.
      const latest = await loadStored(this.deps.storage);
      if (latest.kind === "v2") await this.deps.port.restore(latest.backup);
      if (statuses.changes.length > 0) pulled += await this.deps.port.applyStatusChanges(statuses.changes);
      if (statuses.declaredLevels.length > 0)
        pulled += await this.deps.port.applyDeclaredLevelChanges(statuses.declaredLevels);
      if (cards.ops.length > 0) pulled += await this.deps.port.applyCardOps(cards.ops);
      if (pulled > 0) await saveBackup(this.deps.storage, await this.deps.port.backup());
    }

    // Advance the cursors only AFTER the merged state is durably saved: an interruption
    // before saveBackup leaves the cursor unmoved, so the changes are simply re-pulled
    // (and re-applied idempotently) next time — never dropped.
    await this.saveCursor(STATUS_CURSOR_KEY, statuses.cursor);
    await this.saveCursor(CARD_CURSOR_KEY, cards.cursor);
    await this.deps.storage.set({ [SYNC_LANGUAGES_KEY]: languages });
    return { pushedStatuses, pushedCards, pulled };
  }

  /**
   * A device that accepts a language it did not pulls everything again (design D4): the server
   * kept no memory of the cards it withheld, and this device skipped that language's statuses.
   */
  private async widen(languages: StudiedLanguage[]): Promise<void> {
    const stored = (await this.deps.storage.get(SYNC_LANGUAGES_KEY))[SYNC_LANGUAGES_KEY];
    const before = Array.isArray(stored) ? stored.filter((l): l is string => typeof l === "string") : ["en"];
    if (languages.some((language) => !before.includes(language))) await clearSyncCursors(this.deps.storage);
  }

  /**
   * « Effacer mes données Lingua »: the server first, then this device, then the mark is
   * remembered so the next sync does not wipe again. A failed call throws before anything
   * local is touched. The caller serializes it with `sync()`.
   */
  async eraseAll(): Promise<void> {
    const res = await this.deps.clients().data.eraseMyData({});
    await this.wipeLocal();
    await this.deps.storage.set({ [ERASED_AT_KEY]: Number(res.erasedAt) });
    this.erasedAt = Number(res.erasedAt);
  }

  /**
   * Read the account's erasure mark before anything is pushed. A mark newer than the one
   * this device acted on means its store predates the erasure: empty it. A device with no
   * recorded mark that never synced adopts the current one (its store never held the
   * erased data); one that did sync before marks existed counts as never having seen any.
   */
  private async checkErasure(): Promise<void> {
    const res = await this.deps.clients().data.getDataState({});
    this.cardLanguage = res.cardLanguage === true;
    const mark = Number(res.erasedAt);
    const recorded = (await this.deps.storage.get(ERASED_AT_KEY))[ERASED_AT_KEY];
    const known = typeof recorded === "number" ? recorded : (await this.hasSynced()) ? 0 : mark;
    if (mark > known) await this.wipeLocal();
    if (recorded !== mark) await this.deps.storage.set({ [ERASED_AT_KEY]: mark });
    this.erasedAt = mark;
  }

  private async hasSynced(): Promise<boolean> {
    return (await this.loadCursor(STATUS_CURSOR_KEY)) > 0 || (await this.loadCursor(CARD_CURSOR_KEY)) > 0;
  }

  /**
   * Empty this device's Lingua store — statuses, deck, level, calibration, exposure
   * counters, local daily stats — and the pull cursors. The other contexts follow the
   * saved backup through storage.onChanged.
   */
  private async wipeLocal(): Promise<void> {
    const { port, storage } = this.deps;
    await port.reset();
    // The reset returned the profile to the engine's native language, studying its first pack's
    // language alone — the default profile, for a reader of French (generalise-lingua-native-language
    // D5): calibrate the language it reads in.
    const lang = port.for(await readingLanguage(port));
    await lang.setCalibration((await lang.hasLevels()) ? 0 : DEFAULT_CALIBRATION);
    await saveBackup(storage, await port.backup());
    await clearDailyStats(storage);
    await clearSyncCursors(storage);
  }

  /** An op made after an erasure is dated after its mark, whatever this device's clock says. */
  private afterMark(ts: number): number {
    return this.erasedAt > 0 ? Math.max(ts, this.erasedAt + 1) : ts;
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
          clientTs: BigInt(this.afterMark(o.updated_at)),
          deviceId: this.deps.deviceId,
        })),
      });
    }
    return ops.length;
  }

  /**
   * Push the declared-level decisions (add-lingua-cefr-levels). They ride
   * KnownWordsService alongside the statuses and share the same cursor, so no
   * separate cursor is tracked. Sent in one request (there is one per language);
   * skipped when there is nothing to push.
   */
  private async pushDeclaredLevels(): Promise<void> {
    const levels = await this.deps.port.exportDeclaredLevels();
    if (levels.length === 0) return;
    await this.deps.clients().knownWords.pushOps({
      ops: [],
      declaredLevels: levels.map((l) => ({
        language: l.language,
        level: l.level,
        clientTs: BigInt(this.afterMark(l.updated_at)),
        deviceId: this.deps.deviceId,
      })),
    });
  }

  private async pushCards(): Promise<number> {
    // A card in another language waits for a server that keys cards by language (design D3):
    // an older one would store it as English, under the key of the English card.
    const ops = (await this.deps.port.exportCardOps()).filter(
      (c) => this.cardLanguage || (c.language || "en") === "en",
    );
    for (const batch of chunk(ops, BATCH)) {
      await this.deps.clients().deck.pushCards({
        cards: batch.map((c) => ({
          clientId: c.client_id,
          language: c.language,
          lemma: c.lemma,
          surfaceForm: c.surface_form,
          sourceSentence: c.source_sentence,
          // No `source`: the page a card was captured from stays on the device.
          gloss: c.gloss,
          fsrsState: c.fsrs_state,
          deleted: c.deleted,
          clientTs: BigInt(this.afterMark(c.client_ts)),
          deviceId: this.deps.deviceId,
        })),
      });
    }
    return ops.length;
  }

  /** Idempotent upsert of the local daily aggregates (replace-by-key server-side), one per day and language. */
  private async pushStats(): Promise<void> {
    const daily = await loadDailyStats(this.deps.storage);
    const stats = Object.entries(daily).flatMap(([day, byLanguage]) =>
      Object.entries(byLanguage).map(([language, s]) => ({
        day: Number(day),
        language,
        deviceId: this.deps.deviceId,
        exposures: s.exposures,
        // Always set (0 included): its presence is what the server stores a stat on.
        unknownSeen: s.unknownSeen ?? 0,
        wordsLearned: s.wordsLearned,
        reviewsDone: s.reviews,
      })),
    );
    if (stats.length > 0) await this.deps.clients().stats.upsertDailyStats({ stats });
  }

  /**
   * Fetch status + declared-level changes after the stored cursor (no apply, no
   * cursor write). Levels share the status cursor server-side, so one pull returns
   * both and one cursor covers them.
   */
  private async fetchStatuses(languages: StudiedLanguage[]): Promise<{
    changes: StatusChangeIn[];
    declaredLevels: DeclaredLevelOp[];
    cursor: number;
  }> {
    const cursor = await this.loadCursor(STATUS_CURSOR_KEY);
    const res = await this.deps.clients().knownWords.pullChanges({ cursor: BigInt(cursor) });
    // The server returns every language's; this device applies the ones it accepts (design D2).
    // A dropped record comes back with a full re-pull when its language is accepted (D4).
    const accepted = (language: string): boolean => languages.includes((language || "en") as StudiedLanguage);
    return {
      changes: res.changes
        .filter((c) => accepted(c.language))
        .map((c) => ({
          language: c.language,
          lemma: c.lemma,
          status: c.status,
          provenance: c.provenance,
          updated_at: Number(c.updatedAt),
        })),
      declaredLevels: res.declaredLevels
        .filter((l) => accepted(l.language))
        .map((l) => ({
          language: l.language,
          level: l.level,
          updated_at: Number(l.updatedAt),
        })),
      cursor: Number(res.cursor),
    };
  }

  /** Fetch card ops after the stored cursor (no apply, no cursor write). */
  private async fetchCards(languages: StudiedLanguage[]): Promise<{ ops: CardOp[]; cursor: number }> {
    const cursor = await this.loadCursor(CARD_CURSOR_KEY);
    // The server returns the cards of these languages only (add-lingua-card-language).
    const res = await this.deps.clients().deck.pullCards({ cursor: BigInt(cursor), languages });
    return {
      ops: res.cards.map((c) => ({
        client_id: c.clientId,
        language: c.language || "en",
        lemma: c.lemma,
        surface_form: c.surfaceForm,
        source_sentence: c.sourceSentence,
        source: "", // never on the wire; the local card keeps its own
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

/**
 * Reset the pull cursors to 0 so the NEXT sync re-pulls the full server state
 * (backend returns everything with `sequence > 0`). Called after a full local
 * wipe so a signed-in user's statuses + deck re-download from the server instead
 * of staying gone — the reset becomes a repair, not a loss. Exposure counters are
 * device-local and do not come back.
 */
export async function clearSyncCursors(storage: AsyncStorageArea): Promise<void> {
  await storage.set({ [STATUS_CURSOR_KEY]: 0, [CARD_CURSOR_KEY]: 0 });
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
