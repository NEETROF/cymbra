import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { effectScope } from "vue";
import { flushPromises } from "@vue/test-utils";
import { setClientsForTest } from "@/lib/api";
import { makeFakeClients } from "./fakes";
import { useReviewSession } from "@/composables/useReviewSession";

const hit = (id: string) => ({ id, title: id, source: "pdmx", moderationStatus: "pending" });

describe("useReviewSession (burn-down)", () => {
  beforeEach(() => setActivePinia(createPinia()));
  afterEach(() => setClientsForTest(null as never));

  it("walks the queue, evaluating each score and advancing", async () => {
    // Page 0 has two scores, then (after they're decided) the queue is empty.
    const { clients, state } = makeFakeClients();
    let call = 0;
    (clients.score as unknown as { searchCatalog: () => Promise<unknown> }).searchCatalog = () => {
      call += 1;
      return Promise.resolve(
        call === 1 ? { hits: [hit("a"), hit("b")], total: 2, nextOffset: 2 } : { hits: [], total: 0, nextOffset: 0 },
      );
    };
    setClientsForTest(clients);

    const scope = effectScope();
    const s = scope.run(() => useReviewSession())!;
    await s.start();
    await flushPromises();

    expect(s.current.value?.id).toBe("a");
    expect(s.done.value).toBe(false);

    await s.decide("accepted");
    await flushPromises();
    expect(s.current.value?.id).toBe("b"); // auto-advanced
    expect(state.evaluateCalls).toEqual([{ scoreId: "a", status: "accepted" }]);

    await s.decide("rejected");
    await flushPromises();
    // Deck exhausted → re-fetch (call 2) returns empty → done, no infinite loop.
    expect(s.current.value).toBeNull();
    expect(s.done.value).toBe(true);
    expect(s.reviewedCount.value).toBe(2);
    expect(state.evaluateCalls).toHaveLength(2);
    scope.stop();
  });

  it("skips without evaluating, and the skipped score does not reappear", async () => {
    const { clients, state } = makeFakeClients();
    // Page 0 always returns the same score; only the handled-filter stops the loop.
    (clients.score as unknown as { searchCatalog: () => Promise<unknown> }).searchCatalog = () =>
      Promise.resolve({ hits: [hit("x")], total: 1, nextOffset: 1 });
    setClientsForTest(clients);

    const scope = effectScope();
    const s = scope.run(() => useReviewSession())!;
    await s.start();
    await flushPromises();
    expect(s.current.value?.id).toBe("x");

    await s.skip();
    await flushPromises();
    // Re-fetch returns "x" again, but it's filtered out (handled) → done.
    expect(s.done.value).toBe(true);
    expect(state.evaluateCalls).toHaveLength(0); // skip never evaluates
    scope.stop();
  });

  it("passes the rejection reason through to the evaluate call", async () => {
    const { clients, state } = makeFakeClients();
    (clients.score as unknown as { searchCatalog: () => Promise<unknown> }).searchCatalog = () =>
      Promise.resolve({ hits: [hit("a")], total: 1, nextOffset: 1 });
    setClientsForTest(clients);

    const scope = effectScope();
    const s = scope.run(() => useReviewSession())!;
    await s.start();
    await flushPromises();

    await s.decide("rejected", "blurry scan");
    await flushPromises();
    expect(state.evaluateCalls).toEqual([{ scoreId: "a", status: "rejected", reason: "blurry scan" }]);
    scope.stop();
  });
});
