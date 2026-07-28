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
    expect(store.result.status).toBe("success");
    if (store.result.status === "success") {
      expect(store.result.data.total).toBe(1);
      expect(store.result.data.hits).toHaveLength(1);
    }
  });

  it("forwards review-queue mode to the search RPC (pending + flagged accepted)", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    const store = useCatalogStore();

    await store.search({ reviewQueue: true, sort: [...QUEUE_SORT] });

    expect(state.searchCalls[0].reviewQueue).toBe(true);
    // The review-priority sort still leads with the re-review flag.
    expect(state.searchCalls[0].sort[0].field).toBe("needs_review");
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

  it("loads per-status counts for the header stat cards", async () => {
    const { clients, state } = makeFakeClients({ total: 4 });
    setClientsForTest(clients);
    const store = useCatalogStore();

    await store.loadStats();

    // Three count-only queries (limit 1), one per status.
    expect(state.searchCalls).toHaveLength(3);
    expect(state.searchCalls.every((c) => c.limit === 1)).toBe(true);
    expect(store.stats.status).toBe("success");
    if (store.stats.status === "success") {
      expect(store.stats.data.total).toBe(
        store.stats.data.pending + store.stats.data.accepted + store.stats.data.rejected,
      );
    }
  });

  it("fetches a score's bytes and metadata by id (detail deep-link)", async () => {
    const { clients } = makeFakeClients({ hits: [{ id: "score-9", title: "Nocturne" }] });
    setClientsForTest(clients);
    const store = useCatalogStore();

    const bytes = await store.fetchBytes("score-9");
    const hit = await store.fetchHit("score-9");

    expect(bytes).toBeInstanceOf(Uint8Array);
    expect(bytes).toHaveLength(3);
    expect((hit as { id: string }).id).toBe("score-9");
  });

  it("surfaces an error without throwing", async () => {
    const { clients } = makeFakeClients();
    // Make searchCatalog reject.
    (clients.score as unknown as { searchCatalog: () => Promise<never> }).searchCatalog = () =>
      Promise.reject(new Error("boom"));
    setClientsForTest(clients);
    const store = useCatalogStore();
    await store.search({ moderationStatus: "pending" });
    expect(store.result.status).toBe("error");
    if (store.result.status === "error") {
      // The union holds a user-facing message, never the raw technical error.
      expect(store.result.error).not.toContain("boom");
      expect(store.result.error.length).toBeGreaterThan(0);
    }
  });
});
