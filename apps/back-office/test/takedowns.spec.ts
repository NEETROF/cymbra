import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import { useTakedownsStore } from "@/stores/takedowns";
import { makeFakeClients } from "./fakes";

// Private-score takedown store (change: add-private-score-catalog). Driven
// entirely through the injectable client seam — no network, no component.

const score = {
  id: "s1",
  ownerId: "u1",
  title: "Reported Piece",
  composer: "Anon",
  sizeBytes: 1024n,
  createdAt: 1_760_000_000n,
  rightsBasis: "own_work",
};

/** Fake clients whose takedown RPCs are recorded (and optionally fail). */
function wire(opts: { scores?: unknown[]; searchError?: unknown; removeError?: unknown } = {}) {
  const { clients } = makeFakeClients();
  const calls = { search: [] as unknown[], remove: [] as unknown[] };
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (clients.score as any).adminSearchUserScores = vi.fn(async (req: unknown) => {
    calls.search.push(req);
    if (opts.searchError) throw opts.searchError;
    return { scores: opts.scores ?? [score] };
  });
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (clients.score as any).adminRemoveUserScore = vi.fn(async (req: unknown) => {
    calls.remove.push(req);
    if (opts.removeError) throw opts.removeError;
    return {};
  });
  setClientsForTest(clients);
  return calls;
}

describe("takedowns store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("refuses a criterion-less lookup without calling the server", async () => {
    const calls = wire();
    const store = useTakedownsStore();

    expect(store.canSearch()).toBe(false);
    await store.search({ ownerId: "  ", title: "" });

    expect(store.results.status).toBe("error");
    expect(calls.search).toHaveLength(0);
  });

  it("searches by owner or title and exposes the hits", async () => {
    const calls = wire();
    const store = useTakedownsStore();

    await store.search({ ownerId: "u1" });
    expect(store.results.status).toBe("success");
    if (store.results.status === "success") {
      expect(store.results.data.map((s) => s.id)).toEqual(["s1"]);
    }
    expect(calls.search).toHaveLength(1);

    await store.search({ ownerId: "", title: "reported" });
    expect(calls.search).toHaveLength(2);
    expect(calls.search[1]).toMatchObject({ title: "reported" });
  });

  it("puts a denied lookup in the union rather than throwing", async () => {
    wire({ searchError: new ConnectError("nope", Code.PermissionDenied) });
    const store = useTakedownsStore();

    await store.search({ ownerId: "u1" });

    expect(store.results.status).toBe("error");
  });

  it("refuses a removal without a reason and never calls the server", async () => {
    const calls = wire();
    const store = useTakedownsStore();

    await store.remove("s1", "   ");

    expect(store.op.status).toBe("error");
    expect(calls.remove).toHaveLength(0);
  });

  it("removes with a trimmed reason and re-runs the search", async () => {
    const calls = wire();
    const store = useTakedownsStore();
    await store.search({ ownerId: "u1" });

    await store.remove("s1", "  DMCA #42  ");

    expect(store.op.status).toBe("success");
    expect(calls.remove).toEqual([{ id: "s1", reason: "DMCA #42" }]);
    // The table is refreshed so it reflects what is left.
    expect(calls.search).toHaveLength(2);
  });

  it("surfaces a failed removal as an error in the union", async () => {
    wire({ removeError: new ConnectError("boom", Code.Internal) });
    const store = useTakedownsStore();

    await store.remove("s1", "notice");

    expect(store.op.status).toBe("error");
  });
});
