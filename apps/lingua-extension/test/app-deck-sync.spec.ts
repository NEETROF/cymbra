import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  APP_PULL_WAIT_MS,
  APP_PUSH_DEBOUNCE_MS,
  APP_SYNC_KEY,
  APP_SYNC_THROTTLE_MS,
  type AppDeckSyncDeps,
  appDeckSync,
  layeredArea,
} from "@/app/deck-sync.ts";
import { accountDigest } from "@/state/app-relay.ts";
import { saveBackup } from "@/state/storage.ts";
import { answer, capturedCard, fakeSyncServer, lwwPort, memoryArea, memoryLocal } from "./support/sync-server.ts";

// The app's synchronisation (add-lingua-app-deck-sync D1–D10), over the in-memory server and the
// engine double of test/support/sync-server.ts: when it exchanges, what it pushes, where it keeps its
// bookkeeping, and when it refuses.

const USER = "user-1";

async function setup(
  over: Partial<AppDeckSyncDeps> & { serverCards?: string[]; deckCards?: string[]; nowMs?: number } = {},
) {
  const server = fakeSyncServer();
  // What another device already pushed.
  if (over.serverCards?.length) {
    await server.clients.deck.pushCards({
      cards: over.serverCards.map((lemma) => ({
        clientId: lemma,
        language: "en",
        lemma,
        surfaceForm: lemma,
        sourceSentence: "",
        gloss: "",
        glossLanguage: "",
        fsrsState: JSON.stringify({ reps: 3 }),
        deleted: false,
        clientTs: 5_000n,
        deviceId: "mac",
      })),
    });
  }
  server.calls.length = 0;
  const port = lwwPort();
  await port.applyCardOps((over.deckCards ?? ["one", "two", "three"]).map((l) => capturedCard(l, 1_000)));
  const store = memoryArea();
  await saveBackup(store, await port.backup());
  const since = 10_000;
  let reviewing = false;
  const now = { ms: over.nowMs ?? 20_000 };
  const local = memoryLocal();
  const deps: AppDeckSyncDeps = {
    port,
    store,
    pendingOps: async () => {
      const cards = (await port.exportCardOps()).filter((c) => c.client_ts >= since);
      const statuses = (await port.exportStatusOps()).filter((s) => s.updated_at >= since);
      return { cards, statuses };
    },
    sharedAccount: await accountDigest(USER),
    sharedErasedAt: 0,
    discardPending: vi.fn(async () => {}),
    signedIn: () => true,
    userId: vi.fn(async () => USER),
    clients: () => server.clients,
    local,
    reviewing: () => reviewing,
    onApplied: vi.fn(),
    onChange: vi.fn(),
    now: () => now.ms,
    ...over,
  };
  const sync = appDeckSync(deps);
  const save = async (): Promise<void> => {
    await store.set({ lingua: { v: 2, backup: await port.backup() } });
    sync.saved();
  };
  return {
    server,
    port,
    store,
    local,
    deps,
    sync,
    now,
    save,
    setReviewing: (on: boolean) => void (reviewing = on),
  };
}

const pushedCards = (server: ReturnType<typeof fakeSyncServer>) => [...server.cards().keys()].sort();

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
  vi.spyOn(console, "warn").mockImplementation(() => {});
});
afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("the app is a device of its own (D1, D10)", () => {
  it("generates its device id once, apart from any the extension shared, and keeps it per account", async () => {
    const { sync, store, local, server, port, save } = await setup();
    await store.set({ "cymbra-lingua-device": "extension-id" });
    await sync.open();
    const book = JSON.parse(local.getItem(APP_SYNC_KEY) ?? "{}") as { account: string; items: Record<string, unknown> };
    const id = book.items["cymbra-lingua-device"];
    expect(typeof id).toBe("string");
    expect(id).not.toBe("extension-id");
    expect(book.account).toBe(await accountDigest(USER));
    expect(store.items["cymbra-lingua-device"]).toBe("extension-id"); // untouched
    // Its pushes carry it.
    await answer(port, "one", 30_000);
    await save();
    await vi.advanceTimersByTimeAsync(0);
    expect(server.cards().get("one")?.deviceId).toBe(id);
  });

  it("drops another account's bookkeeping, and pulls from the start when it is lost", async () => {
    const { sync, local, server } = await setup({ serverCards: ["far"] });
    local.setItem(
      APP_SYNC_KEY,
      JSON.stringify({ account: "another", items: { "cymbra-lingua-card-cursor": 99, "cymbra-lingua-device": "old" } }),
    );
    await sync.open();
    const book = JSON.parse(local.getItem(APP_SYNC_KEY) ?? "{}") as { items: Record<string, unknown> };
    expect(book.items["cymbra-lingua-device"]).not.toBe("old");
    expect(server.calls).toContain("pullCards");
    expect(book.items["cymbra-lingua-card-cursor"]).toBeGreaterThan(0);
  });

  it("routes the bookkeeping to its own area and the reader's data to the store", async () => {
    const store = memoryArea({ lingua: 1 });
    const book = { account: "a", items: {} as Record<string, unknown> };
    const save = vi.fn();
    const area = layeredArea(store, book, save);
    await area.set({ "cymbra-lingua-status-cursor": 4, lingua: 2 });
    expect(book.items).toEqual({ "cymbra-lingua-status-cursor": 4 });
    expect(store.items.lingua).toBe(2);
    expect(save).toHaveBeenCalledOnce();
    expect(await area.get(null)).toEqual({ lingua: 2, "cymbra-lingua-status-cursor": 4 });
    expect(await area.get(["cymbra-lingua-status-cursor", "absent"])).toEqual({ "cymbra-lingua-status-cursor": 4 });
    await area.set({ lingua: 3 });
    expect(save).toHaveBeenCalledOnce();
  });
});

describe("the opening's exchange (D2)", () => {
  it("a quick pull is applied before the review mounts", async () => {
    const { sync, port, deps } = await setup({ serverCards: ["far"] });
    await sync.open();
    expect(port.state().cards.far).toBeDefined();
    expect(deps.onApplied).toHaveBeenCalledOnce();
    expect(sync.line()).toBe("synced");
  });

  it("a slow one opens the review on the snapshot, and is applied once it lands", async () => {
    const { sync, port, server, deps } = await setup({ serverCards: ["far"] });
    const release = server.holdPulls();
    const opening = sync.open();
    await vi.advanceTimersByTimeAsync(APP_PULL_WAIT_MS);
    await opening;
    expect(port.state().cards.far).toBeUndefined();
    release();
    await vi.advanceTimersByTimeAsync(0);
    await vi.waitFor(() => expect(port.state().cards.far).toBeDefined());
    expect(deps.onApplied).toHaveBeenCalledOnce();
  });

  it("one arriving during a session waits for its end", async () => {
    const { sync, port, server, setReviewing } = await setup({ serverCards: ["far"] });
    const release = server.holdPulls();
    const opening = sync.open();
    await vi.advanceTimersByTimeAsync(APP_PULL_WAIT_MS);
    await opening;
    setReviewing(true);
    release();
    await vi.advanceTimersByTimeAsync(5_000);
    expect(port.state().cards.far).toBeUndefined();
    setReviewing(false);
    await vi.advanceTimersByTimeAsync(1_000);
    await vi.waitFor(() => expect(port.state().cards.far).toBeDefined());
  });

  it("a failure opens on the snapshot and says so in one line", async () => {
    const { sync, port, server, deps } = await setup();
    server.offline = true;
    await sync.open();
    expect(Object.keys(port.state().cards)).toEqual(["one", "two", "three"]);
    expect(sync.line()).toBe("failed");
    expect(deps.onChange).toHaveBeenCalled();
  });

  it("an account it cannot read is a failure, and holds nothing back", async () => {
    const { sync, server } = await setup({ userId: async () => Promise.reject(new Error("offline")) });
    await sync.open();
    expect(sync.line()).toBe("failed");
    expect(server.calls).toEqual([]);
  });

  it("throttles openings to one exchange every few seconds, counted from the last success", async () => {
    const first = await setup();
    await first.sync.open();
    const calls = first.server.calls.length;
    const again = appDeckSync({ ...first.deps });
    first.now.ms += APP_SYNC_THROTTLE_MS - 1;
    await again.open();
    expect(first.server.calls.length).toBe(calls);
    first.now.ms += 1;
    await appDeckSync({ ...first.deps }).open();
    expect(first.server.calls.length).toBeGreaterThan(calls);
  });
});

describe("when the app pushes (D3, D5)", () => {
  it("three answers in two seconds give one push, carrying exactly the relayed ops", async () => {
    const { sync, server, port, save, setReviewing } = await setup();
    await sync.open();
    server.calls.length = 0;
    setReviewing(true);
    for (const [i, lemma] of ["one", "two", "three"].entries()) {
      await answer(port, lemma, 30_000 + i * 700);
      await save();
      await vi.advanceTimersByTimeAsync(700);
    }
    expect(server.calls).toEqual([]);
    await vi.advanceTimersByTimeAsync(APP_PUSH_DEBOUNCE_MS);
    expect(server.calls.filter((c) => c === "pushCards")).toHaveLength(1);
    expect(pushedCards(server)).toEqual(["one", "three", "two"]);
    // No full exchange while the session runs.
    expect(server.calls).not.toContain("pullCards");
    expect(server.calls).not.toContain("upsertDailyStats");
  });

  it("pushes nothing of Safari's snapshot", async () => {
    const { sync, server, port, save } = await setup();
    await sync.open();
    await answer(port, "two", 30_000);
    await save();
    await vi.advanceTimersByTimeAsync(0);
    expect(pushedCards(server)).toEqual(["two"]);
  });

  it("at a session's end: a push, then a full exchange", async () => {
    const { sync, server, port, save } = await setup();
    await sync.open();
    server.calls.length = 0;
    await answer(port, "one", 30_000);
    await save(); // the last answer: no card left
    await vi.waitFor(() => expect(server.calls).toContain("pullCards"));
    expect(server.calls.indexOf("pushCards")).toBeLessThan(server.calls.indexOf("pullCards"));
  });

  it("going to the background pushes at once", async () => {
    const { sync, server, port, save, setReviewing } = await setup();
    await sync.open();
    server.calls.length = 0;
    setReviewing(true);
    await answer(port, "one", 30_000);
    await save();
    await sync.background();
    expect(server.calls).toContain("pushCards");
    expect(server.calls).not.toContain("pullCards");
    // The debounced push does not run again.
    server.calls.length = 0;
    await vi.advanceTimersByTimeAsync(APP_PUSH_DEBOUNCE_MS);
    expect(server.calls).toEqual([]);
  });

  it("a pull written back is not taken for an answer", async () => {
    const { sync, server } = await setup({ serverCards: ["far"] });
    await sync.open();
    server.calls.length = 0;
    await vi.advanceTimersByTimeAsync(APP_PUSH_DEBOUNCE_MS);
    expect(server.calls).toEqual([]);
  });
});

describe("one account (D7)", () => {
  async function refused(sharedAccount: string | null | undefined, userId = USER) {
    const t = await setup({ sharedAccount, userId: async () => userId });
    await t.sync.open();
    await answer(t.port, "one", 30_000);
    await t.save();
    await t.sync.background();
    await vi.advanceTimersByTimeAsync(APP_PUSH_DEBOUNCE_MS);
    expect(t.server.calls.filter((c) => c !== "getAccount")).toEqual([]);
    expect(t.sync.active()).toBe(false);
    return t.sync.line();
  }

  it("Another account in Safari: no deck sent, the line says why", async () => {
    expect(await refused(await accountDigest("account-a"), "account-b")).toBe("otherAccount");
  });

  it("Safari signed out", async () => {
    expect(await refused(null)).toBe("safariSignedOut");
  });

  it("An extension that predates the account digest", async () => {
    expect(await refused(undefined)).toBe("olderExtension");
  });

  it("not signed in inside the app: nothing at all, and the line is off", async () => {
    const t = await setup({ signedIn: () => false });
    await t.sync.open();
    await t.sync.background();
    expect(t.server.calls).toEqual([]);
    expect(t.sync.line()).toBe("off");
    expect(t.deps.userId).not.toHaveBeenCalled();
  });

  it("signing in synchronises at once; signing out drops the account's bookkeeping", async () => {
    let signedIn = false;
    const t = await setup({ signedIn: () => signedIn });
    await t.sync.open();
    expect(t.server.calls).toEqual([]);
    signedIn = true;
    await t.sync.accountChanged();
    expect(t.server.calls).toContain("pullCards");
    expect(t.local.getItem(APP_SYNC_KEY)).not.toBeNull();
    signedIn = false;
    await t.sync.accountChanged();
    expect(t.local.getItem(APP_SYNC_KEY)).toBeNull();
    expect(t.sync.line()).toBe("off");
  });
});

describe("an erasure made elsewhere (D8)", () => {
  it("resets, hands the host the mark, and reviews what the server holds", async () => {
    const t = await setup({ serverCards: [] });
    await t.server.clients.data.eraseMyData({});
    await t.server.clients.deck.pushCards({
      cards: [
        {
          clientId: "after",
          language: "en",
          lemma: "after",
          surfaceForm: "after",
          sourceSentence: "",
          gloss: "",
          glossLanguage: "",
          fsrsState: "{}",
          deleted: false,
          clientTs: BigInt(Date.now() + 1000),
          deviceId: "mac",
        },
      ],
    });
    t.server.calls.length = 0;
    await t.sync.open();
    expect(t.deps.discardPending).toHaveBeenCalledOnce();
    expect(Object.keys(t.port.state().cards)).toEqual(["after"]);
    expect(t.server.calls).not.toContain("pushCards");
    expect(t.deps.onApplied).toHaveBeenCalled();
    // The same opening's next exchange does not erase again.
    await t.sync.accountChanged();
    expect(t.deps.discardPending).toHaveBeenCalledOnce();
  });
});

describe("offline and statistics (D6, D9)", () => {
  it("a failed push leaves the changes to a later opening, which pushes them", async () => {
    const t = await setup();
    await t.sync.open();
    t.server.offline = true;
    await answer(t.port, "one", 30_000);
    await t.save();
    await t.sync.background();
    expect(t.sync.line()).toBe("failed");
    t.server.offline = false;
    t.now.ms += APP_SYNC_THROTTLE_MS;
    // The next opening: the same pending changes, laid back on the snapshot.
    await appDeckSync({ ...t.deps }).open();
    expect(pushedCards(t.server)).toEqual(["one"]);
  });

  it("never sends daily statistics", async () => {
    const t = await setup();
    await t.store.set({
      "cymbra-lingua-daily-v4": { 20000: { en: { exposures: 0, wordsLearned: 0, reviews: 3, native: "fr" } } },
    });
    await t.sync.open();
    await answer(t.port, "one", 30_000);
    await t.save();
    await t.sync.background();
    expect(t.server.calls).not.toContain("upsertDailyStats");
  });
});

describe("a session's end saves twice (D5)", () => {
  it("its backup and its count give one push and one full exchange", async () => {
    const { sync, server, port, save } = await setup();
    await sync.open();
    server.calls.length = 0;
    await answer(port, "one", 30_000);
    await save();
    await save();
    await vi.waitFor(() => expect(server.calls).toContain("pullCards"));
    await vi.advanceTimersByTimeAsync(0);
    expect(server.calls.filter((c) => c === "pullCards")).toHaveLength(1);
  });
});
