// A focused, burn-down review session: it walks the review queue (pending +
// community-flagged) one score at a time and auto-advances after each accept/reject/
// re-queue, until the queue is empty. NOT an infinite loop — a decided score leaves the
// pending set server-side, so the deck naturally shrinks to nothing. Skipped scores are
// remembered so they don't reappear this session. Bytes for the next score are
// prefetched so advancing is instant. All API access goes through the catalog store.

import { computed, ref } from "vue";
import { type Async, failure, idle, loading, success } from "@/lib/async";
import { useCatalogStore, type ModerationStatus } from "@/stores/catalog";
import type { CatalogHit } from "@/gen/score_pb";

export function useReviewSession() {
  const store = useCatalogStore();
  const deck = ref<CatalogHit[]>([]); // current batch, minus already-handled ids
  const index = ref(0);
  const loadState = ref<Async<void>>(idle); // fetching a batch
  const deciding = ref<Async<void>>(idle); // an accept/reject in flight
  const reviewedCount = ref(0);
  const handled = new Set<string>(); // decided or skipped this session
  let exhausted = false;

  const current = computed<CatalogHit | null>(() => deck.value[index.value] ?? null);
  const done = computed(() => current.value == null && exhausted);
  const remaining = computed(() => Math.max(0, deck.value.length - index.value));

  // Prefetch cache (id -> bytes promise) so advancing to the next score is instant.
  const bytesCache = new Map<string, Promise<Uint8Array>>();
  function bytesFor(id: string): Promise<Uint8Array> {
    let p = bytesCache.get(id);
    if (!p) {
      p = store.fetchBytes(id);
      bytesCache.set(id, p);
    }
    return p;
  }
  function prefetchNext(): void {
    const next = deck.value[index.value + 1];
    if (next) void bytesFor(next.id).catch(() => {});
  }

  // Always fetch from offset 0: decided scores drop out of the queue server-side, so the
  // next page-0 is the still-pending remainder; handled ids (incl. skips) are filtered.
  async function loadBatch(): Promise<void> {
    const page = await store.fetchReviewPage(0);
    const fresh = (page.hits as CatalogHit[]).filter((h) => !handled.has(h.id));
    deck.value = fresh;
    index.value = 0;
    exhausted = fresh.length === 0;
    prefetchNext();
  }

  async function runLoad(): Promise<void> {
    loadState.value = loading;
    try {
      await loadBatch();
      loadState.value = success(undefined);
    } catch {
      loadState.value = failure("review_load_failed");
    }
  }

  async function start(): Promise<void> {
    handled.clear();
    bytesCache.clear();
    reviewedCount.value = 0;
    exhausted = false;
    deck.value = [];
    index.value = 0;
    await runLoad();
  }

  async function advance(): Promise<void> {
    index.value += 1;
    prefetchNext();
    if (index.value >= deck.value.length && !exhausted) await runLoad();
  }

  async function decide(status: ModerationStatus): Promise<void> {
    const cur = current.value;
    if (!cur || deciding.value.status === "loading") return;
    handled.add(cur.id);
    deciding.value = loading;
    try {
      await store.evaluate(cur.id, status);
      reviewedCount.value += 1;
      deciding.value = success(undefined);
    } catch {
      handled.delete(cur.id); // allow a retry
      deciding.value = failure("review_decide_failed");
      return; // don't advance on a failed decision
    }
    await advance();
  }

  async function skip(): Promise<void> {
    const cur = current.value;
    if (cur) handled.add(cur.id); // won't reappear this session
    await advance();
  }

  return {
    current,
    done,
    remaining,
    reviewedCount,
    loadState,
    deciding,
    start,
    decide,
    skip,
    bytesFor,
  };
}
