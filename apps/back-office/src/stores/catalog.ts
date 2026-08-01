import { reactive, ref, toRef } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import type { CatalogHit } from "@/gen/score_pb";

export type ModerationStatus = "pending" | "accepted" | "rejected";
export interface SortKeyInit {
  field: string;
  descending: boolean;
}

/** The filter bar's current selection (kept here so both the bar and views share it). */
export interface Filters {
  query: string;
  author: string;
  level: string;
  isPiano: boolean | undefined;
  moderationStatus: ModerationStatus;
}

// The hub filters (text/author/level/facets) plus the BO-only status filter and
// the structured sort. Only fields the console actually surfaces are modelled.
export interface SearchParams {
  query?: string;
  author?: string;
  level?: string;
  isPiano?: boolean;
  moderationStatus?: ModerationStatus;
  // Review-queue mode: the moderation work list = pending scores PLUS accepted
  // scores flagged for re-review by community ratings. Overrides moderationStatus.
  reviewQueue?: boolean;
  sort?: SortKeyInit[];
  limit?: number;
  offset?: number;
}

export interface CatalogResult {
  hits: CatalogHit[];
  total: number;
  nextOffset: number;
}

/** Server page size — one screen of rows per SearchCatalog request. */
export const PAGE_SIZE = 50;

// The queue's default "review priority" ordering (design D5): flagged re-reviews
// first (now wired to #2's community ratings), then pending, then the most
// substantial scores. Used with reviewQueue mode so flagged accepted scores rank
// above the pending backlog.
export const QUEUE_SORT: SortKeyInit[] = [
  { field: "needs_review", descending: true },
  { field: "status_rank", descending: true },
  { field: "measure_count", descending: true },
  { field: "staff_count", descending: true },
];

export interface CatalogStats {
  total: number;
  accepted: number;
  pending: number;
  rejected: number;
}

// Module-scope data helpers (no store state — kept out of the setup, and flatter,
// per Sonar S7721/S2004). They call the API seam like everything else.
async function countByStatus(moderationStatus: ModerationStatus): Promise<number> {
  const resp = await api().score.searchCatalog({ query: "", moderationStatus, sort: [], limit: 1, offset: 0 });
  return resp.total;
}

async function fetchBytes(catalogId: string): Promise<Uint8Array> {
  const resp = await api().score.getCatalogScoreBytes({ catalogId });
  return resp.data;
}

/** One score's metadata by id — so the detail view is self-sufficient (works on
 * refresh / deep-link, not dependent on the last search's list). */
async function fetchHit(catalogId: string): Promise<CatalogHit> {
  return api().score.getCatalogScore({ catalogId });
}

/** Evaluate a score (raw RPC, no list refresh) — used by the review session, which
 * advances the deck itself instead of re-querying. */
async function evaluate(scoreId: string, status: ModerationStatus) {
  await api().score.setModerationStatus({ scoreId, status });
}

/** A moderator's curatorial edit — only descriptive/attribution fields (never the
 * MusicXML-derived facets). Each field present is applied (an empty string clears
 * composer/arranger/level; an empty title is rejected server-side). */
export interface MetadataEdit {
  title?: string;
  composer?: string;
  arranger?: string;
  level?: string;
}

/** Persist a curatorial metadata edit (raw RPC behind the api() seam). The backend
 * recomputes the derived search keys and audits the change; the caller refreshes the
 * hit on success. Guarded server-side by `require_moderator_or_admin`. */
async function updateCatalogScore(scoreId: string, edit: MetadataEdit) {
  await api().score.updateCatalogScore({ scoreId, ...edit });
}

/** Fetch one page of the review queue (pending + community-flagged, priority-sorted)
 * WITHOUT touching `result` — the review session owns its own deck state. */
async function fetchReviewPage(offset: number): Promise<CatalogResult> {
  const resp = await api().score.searchCatalog({
    query: "",
    reviewQueue: true,
    sort: QUEUE_SORT,
    limit: PAGE_SIZE,
    offset,
  });
  return { hits: resp.hits, total: resp.total, nextOffset: resp.nextOffset };
}

export const useCatalogStore = defineStore("catalog", () => {
  // One value for the whole search lifecycle — views match on it exhaustively.
  const result = ref<Async<CatalogResult>>(idle);
  const lastParams = reactive<SearchParams>({ limit: PAGE_SIZE, offset: 0 });
  // Header stat cards. Kept in its own Async so a stats failure never blocks the
  // list — the cards just fall back to "—".
  const stats = ref<Async<CatalogStats>>(idle);
  // Per-row download state, keyed by catalog id, so each row shows its OWN
  // loading/error and one slow/failed download never blocks the rest of the table.
  // Only the loading/error signal lives here — the bytes are handed to the caller and
  // dropped on success (see `downloadBytes`), never pinned in the store.
  const downloads = reactive<Record<string, Async<Uint8Array>>>({});

  async function search(params: SearchParams) {
    Object.assign(lastParams, { limit: PAGE_SIZE, offset: 0 }, params);
    await run(result, async () => {
      const resp = await api().score.searchCatalog({
        query: params.query ?? "",
        author: params.author,
        level: params.level,
        isPiano: params.isPiano,
        moderationStatus: params.moderationStatus,
        reviewQueue: params.reviewQueue,
        sort: params.sort ?? [],
        limit: params.limit ?? PAGE_SIZE,
        offset: params.offset ?? 0,
      });
      return { hits: resp.hits, total: resp.total, nextOffset: resp.nextOffset };
    });
  }

  /** Evaluate a score; on success re-run the last query so the row reflects it. */
  async function setModerationStatus(scoreId: string, status: ModerationStatus) {
    await evaluate(scoreId, status);
    await search(lastParams);
  }

  /** Fetch one score's decoded MusicXML bytes for a local download, tracking the
   * per-row `Async` state in `downloads`. Reuses `GetCatalogScoreBytes` (already gated
   * to moderator/admin server-side). Returns the bytes on success (which the caller
   * saves to disk) and `null` on failure — the row's error state carries a localized
   * message; a raw gRPC/exception string is never surfaced. Never throws. */
  async function downloadBytes(catalogId: string): Promise<Uint8Array | null> {
    const outcome = await run(toRef(downloads, catalogId), () => fetchBytes(catalogId));
    if (outcome.status === "success") {
      // The row only needed the loading/error signal; drop the bytes so we don't pin
      // every downloaded score in memory.
      delete downloads[catalogId];
      return outcome.data;
    }
    return null;
  }

  /** Per-status counts for the header cards — three cheap count-only queries
   * (limit 1, we only read `total`) run in parallel. */
  async function loadStats() {
    await run(stats, async () => {
      const [pending, accepted, rejected] = await Promise.all([
        countByStatus("pending"),
        countByStatus("accepted"),
        countByStatus("rejected"),
      ]);
      return { pending, accepted, rejected, total: pending + accepted + rejected };
    });
  }

  return {
    result,
    lastParams,
    stats,
    downloads,
    search,
    loadStats,
    evaluate,
    setModerationStatus,
    fetchReviewPage,
    fetchBytes,
    downloadBytes,
    fetchHit,
    updateCatalogScore,
  };
});
