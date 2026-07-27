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
  sort?: SortKeyInit[];
  limit?: number;
  offset?: number;
}

export interface CatalogResult {
  hits: CatalogHit[];
  total: number;
  nextOffset: number;
}

// The queue's default "review priority" ordering (design D5): flagged re-reviews
// first (inert until #2), then pending, then the most substantial scores.
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
  const lastParams = reactive<SearchParams>({ limit: 50, offset: 0 });
  // Header stat cards. Kept in its own Async so a stats failure never blocks the
  // list — the cards just fall back to "—".
  const stats = ref<Async<CatalogStats>>(idle);

  async function search(params: SearchParams) {
    Object.assign(lastParams, { limit: 50, offset: 0 }, params);
    await run(result, async () => {
      const resp = await api().score.searchCatalog({
        query: params.query ?? "",
        author: params.author,
        level: params.level,
        isPiano: params.isPiano,
        moderationStatus: params.moderationStatus,
        sort: params.sort ?? [],
        limit: params.limit ?? 50,
        offset: params.offset ?? 0,
      });
      return { hits: resp.hits, total: resp.total, nextOffset: resp.nextOffset };
    });
  }

  /** Evaluate a score; on success re-run the last query so the row reflects it. */
  async function setModerationStatus(scoreId: string, status: ModerationStatus) {
    await api().score.setModerationStatus({ scoreId, status });
    await search(lastParams);
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

  return { result, lastParams, stats, search, loadStats, setModerationStatus, fetchBytes, fetchHit };
});
