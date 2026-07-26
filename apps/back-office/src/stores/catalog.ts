import { reactive, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
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

// The queue's default "review priority" ordering (design D5): flagged re-reviews
// first (inert until #2), then pending, then the most substantial scores.
export const QUEUE_SORT: SortKeyInit[] = [
  { field: "needs_review", descending: true },
  { field: "status_rank", descending: true },
  { field: "measure_count", descending: true },
  { field: "staff_count", descending: true },
];

export const useCatalogStore = defineStore("catalog", () => {
  const hits = ref<CatalogHit[]>([]);
  const total = ref(0);
  const nextOffset = ref(0);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const lastParams = reactive<SearchParams>({ limit: 50, offset: 0 });

  async function search(params: SearchParams) {
    loading.value = true;
    error.value = null;
    Object.assign(lastParams, { limit: 50, offset: 0 }, params);
    try {
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
      hits.value = resp.hits;
      total.value = resp.total;
      nextOffset.value = resp.nextOffset;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      hits.value = [];
      total.value = 0;
    } finally {
      loading.value = false;
    }
  }

  /** Evaluate a score; on success re-run the last query so the row reflects it. */
  async function setModerationStatus(scoreId: string, status: ModerationStatus) {
    await api().score.setModerationStatus({ scoreId, status });
    await search(lastParams);
  }

  async function fetchBytes(catalogId: string): Promise<Uint8Array> {
    const resp = await api().score.getCatalogScoreBytes({ catalogId });
    return resp.data;
  }

  return { hits, total, nextOffset, loading, error, lastParams, search, setModerationStatus, fetchBytes };
});
