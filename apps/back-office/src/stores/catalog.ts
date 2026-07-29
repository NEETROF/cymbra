import { reactive, ref } from "vue";
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

export const useCatalogStore = defineStore("catalog", () => {
  // One value for the whole search lifecycle — views match on it exhaustively.
  const result = ref<Async<CatalogResult>>(idle);
  const lastParams = reactive<SearchParams>({ limit: PAGE_SIZE, offset: 0 });
  // Header stat cards. Kept in its own Async so a stats failure never blocks the
  // list — the cards just fall back to "—".
  const stats = ref<Async<CatalogStats>>(idle);

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

  /** Evaluate a score (raw RPC, no list refresh) — used by the review session, which
   * advances the deck itself instead of re-querying. */
  async function evaluate(scoreId: string, status: ModerationStatus) {
    await api().score.setModerationStatus({ scoreId, status });
  }

  /** Evaluate a score; on success re-run the last query so the row reflects it. */
  async function setModerationStatus(scoreId: string, status: ModerationStatus) {
    await evaluate(scoreId, status);
    await search(lastParams);
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
    search,
    loadStats,
    evaluate,
    setModerationStatus,
    fetchReviewPage,
    fetchBytes,
    fetchHit,
  };
});
