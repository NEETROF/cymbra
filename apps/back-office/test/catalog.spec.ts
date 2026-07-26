import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { QUEUE_SORT, useCatalogStore } from "@/stores/catalog";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";

describe("catalog store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("sends the moderation-status filter and sort to the search RPC", async () => {
    const { clients, state } = makeFakeClients({ hits: [{ id: "a" }], total: 1 });
    setClientsForTest(clients);
    const store = useCatalogStore();

    await store.search({
      moderationStatus: "pending",
      sort: [{ field: "status_rank", descending: true }],
    });

    expect(state.searchCalls).toHaveLength(1);
    expect(state.searchCalls[0].moderationStatus).toBe("pending");
    expect(state.searchCalls[0].sort).toEqual([{ field: "status_rank", descending: true }]);
    expect(store.total).toBe(1);
    expect(store.hits).toHaveLength(1);
  });

  it("queue ordering sends the default review-priority sort list", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    const store = useCatalogStore();

    await store.search({ moderationStatus: "pending", sort: [...QUEUE_SORT] });

    expect(state.searchCalls[0].sort.map((k) => k.field)).toEqual([
      "needs_review",
      "status_rank",
      "measure_count",
      "staff_count",
    ]);
  });

  it("accept/reject evaluates then re-queries so the row reflects the change", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    const store = useCatalogStore();
    await store.search({ moderationStatus: "pending" }); // 1 search

    await store.setModerationStatus("score-7", "accepted");

    expect(state.evaluateCalls).toEqual([{ scoreId: "score-7", status: "accepted" }]);
    // A follow-up search ran (re-fetch after the write).
    expect(state.searchCalls).toHaveLength(2);
  });

  it("surfaces an error without throwing", async () => {
    const { clients } = makeFakeClients();
    // Make searchCatalog reject.
    (clients.score as unknown as { searchCatalog: () => Promise<never> }).searchCatalog = () =>
      Promise.reject(new Error("boom"));
    setClientsForTest(clients);
    const store = useCatalogStore();
    await store.search({ moderationStatus: "pending" });
    expect(store.error).toBe("boom");
    expect(store.hits).toEqual([]);
  });
});
