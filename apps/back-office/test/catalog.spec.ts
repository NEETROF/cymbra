import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { QUEUE_SORT, setRegenerateScorePreviewForTest, useCatalogStore } from "@/stores/catalog";
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

  it("forwards the all-statuses mode and the source filter to the search RPC", async () => {
    const { clients, state } = makeFakeClients({ hits: [{ id: "a" }], total: 1 });
    setClientsForTest(clients);
    const store = useCatalogStore();

    await store.search({ allStatuses: true, source: "user-proposal" });

    expect(state.searchCalls[0].allStatuses).toBe(true);
    expect(state.searchCalls[0].source).toBe("user-proposal");
    // An empty source is normalised to undefined (no filter).
    await store.search({ allStatuses: true, source: "" });
    expect(state.searchCalls[1].source).toBeUndefined();
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

  it("update forwards only the curatorial fields to the edit RPC", async () => {
    const { clients, state } = makeFakeClients();
    setClientsForTest(clients);
    const store = useCatalogStore();

    await store.updateCatalogScore("score-3", {
      title: "Clair de Lune",
      composer: "Claude Debussy",
      arranger: "",
      level: "advanced",
    });

    expect(state.editCalls).toEqual([
      { scoreId: "score-3", title: "Clair de Lune", composer: "Claude Debussy", arranger: "", level: "advanced" },
    ]);
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

  it("downloadBytes returns the score's bytes and leaves no lingering per-row state", async () => {
    const { clients } = makeFakeClients();
    setClientsForTest(clients);
    const store = useCatalogStore();

    const bytes = await store.downloadBytes("score-1");

    expect(bytes).toBeInstanceOf(Uint8Array);
    expect(bytes).toHaveLength(3);
    // On success the transient row state is dropped — the bytes are handed to the
    // caller (to save to disk), never pinned in the store.
    expect(store.downloads["score-1"]).toBeUndefined();
  });

  it("downloadBytes surfaces a localized per-row error without throwing", async () => {
    const { clients } = makeFakeClients();
    (clients.score as unknown as { getCatalogScoreBytes: () => Promise<never> }).getCatalogScoreBytes = () =>
      Promise.reject(new Error("kaboom"));
    setClientsForTest(clients);
    const store = useCatalogStore();

    const bytes = await store.downloadBytes("score-x");

    expect(bytes).toBeNull();
    const st = store.downloads["score-x"];
    expect(st?.status).toBe("error");
    if (st?.status === "error") {
      // A user-facing message, never the raw technical error.
      expect(st.error).not.toContain("kaboom");
      expect(st.error.length).toBeGreaterThan(0);
    }
  });

  it("tracks each row's download independently — one failure does not affect another", async () => {
    const { clients } = makeFakeClients();
    (
      clients.score as unknown as {
        getCatalogScoreBytes: (r: { catalogId: string }) => Promise<{ data: Uint8Array }>;
      }
    ).getCatalogScoreBytes = ({ catalogId }) =>
      catalogId === "bad" ? Promise.reject(new Error("nope")) : Promise.resolve({ data: new Uint8Array([9]) });
    setClientsForTest(clients);
    const store = useCatalogStore();

    const [ok, bad] = await Promise.all([store.downloadBytes("good"), store.downloadBytes("bad")]);

    expect(ok).toHaveLength(1);
    expect(bad).toBeNull();
    expect(store.downloads["good"]).toBeUndefined(); // cleared on success
    expect(store.downloads["bad"]?.status).toBe("error"); // its own error, retained
  });

  // --- audio teaser (change: add-score-daily-access-rewards) -----------------

  afterEach(() => {
    vi.unstubAllGlobals();
    setRegenerateScorePreviewForTest(async () => {});
  });

  it("forwards the has-preview filter to the search RPC (undefined = any)", async () => {
    const { clients, state } = makeFakeClients({ hits: [{ id: "a" }], total: 1 });
    setClientsForTest(clients);
    const store = useCatalogStore();

    await store.search({ allStatuses: true, hasPreview: false });
    expect(state.searchCalls[0].hasPreview).toBe(false);
    await store.search({ allStatuses: true, hasPreview: true });
    expect(state.searchCalls[1].hasPreview).toBe(true);
    await store.search({ allStatuses: true });
    expect(state.searchCalls[2].hasPreview).toBeUndefined();
  });

  it("regenerateScorePreview flips the loaded row's hasPreview without a re-list", async () => {
    const { clients, state } = makeFakeClients({ hits: [{ id: "a", hasPreview: false }], total: 1 });
    setClientsForTest(clients);
    const called: string[] = [];
    setRegenerateScorePreviewForTest(async (id) => {
      called.push(id);
    });
    const store = useCatalogStore();
    await store.search({ allStatuses: true });

    const outcome = await store.regenerateScorePreview("a");

    expect(outcome.status).toBe("success");
    expect(called).toEqual(["a"]);
    expect(store.preview.status).toBe("success");
    expect(store.previewTarget).toBe("a");
    const row =
      store.result.status === "success" &&
      (store.result.data.hits as { id: string; hasPreview?: boolean }[]).find((h) => h.id === "a");
    expect(row && row.hasPreview).toBe(true);
    expect(state.searchCalls).toHaveLength(1);
  });

  it("captures a preview regeneration failure in the preview state (no throw)", async () => {
    const { clients } = makeFakeClients();
    setClientsForTest(clients);
    setRegenerateScorePreviewForTest(async () => {
      throw new Error("HTTP 412");
    });
    const store = useCatalogStore();

    const outcome = await store.regenerateScorePreview("a");

    expect(outcome.status).toBe("error");
    expect(store.preview.status).toBe("error");
  });

  it("scorePreviewClip fetches the score preview route", async () => {
    const { clients } = makeFakeClients();
    setClientsForTest(clients);
    const fetchMock = vi.fn(() => Promise.resolve(new Response(new Uint8Array([1, 2, 3]), { status: 200 })));
    vi.stubGlobal("fetch", fetchMock);
    const store = useCatalogStore();

    const bytes = await store.scorePreviewClip("a");

    expect(bytes).toEqual(new Uint8Array([1, 2, 3]));
    expect(String((fetchMock.mock.calls[0] as unknown[])[0])).toContain("/scores/a/preview");
  });
});
