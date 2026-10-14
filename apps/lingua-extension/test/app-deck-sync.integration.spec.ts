import { afterEach, describe, expect, it, vi } from "vitest";
import type { AppMessage } from "@/app/bridge.ts";
import { type AppDeckSync, appDeckSync } from "@/app/deck-sync.ts";
import { type AppSession, openAppSession } from "@/app/session.ts";
import { accountDigest, parseAppOps } from "@/state/app-relay.ts";
import { addDailyCounts } from "@/state/dailystats.ts";
import type { NativeMessage } from "@/state/native-signin.ts";
import { type ReviewRelay, reviewRelay } from "@/state/review-relay.ts";
import { saveBackup } from "@/state/storage.ts";
import { SyncEngine } from "@/sync/sync.ts";
import {
  answer,
  capturedCard,
  type FakeSyncServer,
  fakeSyncServer,
  lwwPort,
  memoryArea,
  memoryLocal,
} from "./support/sync-server.ts";

// « Reviewed in the app, seen on another device without opening Safari » (add-lingua-app-deck-sync
// task 5.1, D4): three hosts on one in-memory server applying the server's rules — the iPhone's Safari
// extension (its relay and its own sync), the Cymbra Lingua app on the same iPhone (the relay and its
// sync), and a Mac's extension. The engine is a double holding whole card states under the engine's
// last-write-wins: what travels is a card's state, never a grade, so applying it twice changes nothing.

const USER = "user-1";
const DAY = 20_000;
const LEMMAS = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];

/** The App Group the extension's native handler and the app's bridge share. */
function appGroup() {
  let state: string | null = null;
  let ops: { ops: string; writtenAt: number } | null = null;
  let clock = 0;
  const native = async (message: NativeMessage): Promise<unknown> => {
    switch (message.type) {
      case "review.saveState":
        state = message.state;
        return {};
      case "review.readOps":
        return ops ?? {};
      case "review.clearOps":
        if (ops?.writtenAt === message.writtenAt) ops = null;
        return {};
      default:
        return {};
    }
  };
  const app = async (message: AppMessage): Promise<unknown> => {
    switch (message.type) {
      case "review.state":
        return state === null ? {} : { state };
      case "review.pendingOps":
        return ops ?? {};
      case "review.writeOps":
        ops = { ops: message.ops, writtenAt: ++clock };
        return {};
      default:
        return {};
    }
  };
  return { native, app, pendingOps: () => parseAppOps(ops?.ops) };
}

/** A Lingua extension on a device: its store, its engine, its sync. */
function extension(server: FakeSyncServer, deviceId: string, cards: ReturnType<typeof capturedCard>[] = []) {
  const port = lwwPort();
  const store = memoryArea();
  const engine = new SyncEngine({
    port,
    storage: store,
    clients: () => server.clients,
    deviceId,
    acceptedLanguages: async () => ["en"],
  });
  return {
    port,
    store,
    engine,
    async seed() {
      await port.applyCardOps(cards);
      await saveBackup(store, await port.backup());
    },
    /** The cards as this device's store holds them. */
    async cards() {
      const view = lwwPort();
      const stored = (await store.get("lingua")).lingua as { backup: string } | undefined;
      await view.restore(stored?.backup ?? "{}");
      return view.state().cards;
    },
  };
}

/** Safari's extension on the iPhone, with its relay to the app. */
function safari(server: FakeSyncServer, group: ReturnType<typeof appGroup>, signedIn = true) {
  const host = extension(
    server,
    "safari-iphone",
    LEMMAS.map((l) => capturedCard(l, 1_000)),
  );
  const prefs = memoryArea();
  const relay: ReviewRelay = reviewRelay({
    send: group.native,
    store: host.store,
    prefs,
    port: host.port,
    exclusive: (task) => task(),
    now: () => 2_000,
    account: { signedIn: async () => signedIn, userId: async () => USER },
  });
  return { ...host, relay, prefs };
}

/** One opening of the app: the session over the App Group and the synchronisation. */
async function openApp(
  server: FakeSyncServer,
  group: ReturnType<typeof appGroup>,
  nowMs: number,
  local = memoryLocal(),
  userId = USER,
): Promise<{
  session: AppSession;
  sync: AppDeckSync;
  port: ReturnType<typeof lwwPort>;
  local: Storage;
  reviewing: { now: boolean };
}> {
  const port = lwwPort();
  const reviewing = { now: false };
  let sync: AppDeckSync | null = null;
  const session = await openAppSession({
    send: group.app,
    port: () => port,
    now: () => nowMs,
    onSaved: () => sync?.saved(),
  });
  if (!session) throw new Error("no shared state");
  sync = appDeckSync({
    port,
    store: session.store,
    pendingOps: session.pendingOps,
    sharedAccount: session.sharedAccount,
    sharedErasedAt: session.sharedErasedAt,
    discardPending: session.discardPending,
    signedIn: () => true,
    userId: async () => userId,
    clients: () => server.clients,
    local,
    reviewing: () => reviewing.now,
    now: () => nowMs,
  });
  await sync.open();
  return { session, sync, port, local, reviewing };
}

/** The app's review of `lemmas`, each answer saved and counted as the review page does. */
async function review(
  app: { session: AppSession; port: ReturnType<typeof lwwPort>; reviewing: { now: boolean } },
  lemmas: string[],
  at: number,
) {
  for (const [i, lemma] of lemmas.entries()) {
    // A session under way until its last answer, saved with no card left.
    app.reviewing.now = i < lemmas.length - 1;
    await answer(app.port, lemma, at + i * 1000);
    await addDailyCounts(app.session.store, DAY, "en", "fr", { reviews: 1 });
    await app.session.store.set({ lingua: { v: 2, backup: await app.port.backup() } });
  }
}

const reps = (card: { fsrs_state: string } | undefined): number =>
  card ? (JSON.parse(card.fsrs_state) as { reps: number }).reps : -1;

afterEach(() => vi.restoreAllMocks());

describe("the app's reviews reach the account without Safari (add-lingua-app-deck-sync 5.1)", () => {
  it("Reviewed in the app, seen on another device without opening Safari — then every host converges", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const server = fakeSyncServer();
    const group = appGroup();
    const iphone = safari(server, group);
    await iphone.seed();
    await iphone.engine.sync(); // Safari's deck is on the account
    await iphone.relay.accountChanged(true); // and named to the app by its digest
    const mac = extension(server, "mac");
    await mac.seed();
    await mac.engine.sync();
    expect(Object.keys(await mac.cards())).toHaveLength(10);

    // The app reviews the ten cards, Safari closed, and goes to the background.
    const app = await openApp(server, group, 10_000);
    expect(app.sync.line()).toBe("synced");
    await review(app, LEMMAS, 10_000);
    await app.sync.background();
    expect(server.calls).not.toContain("upsertDailyStats");

    // The Mac syncs: none of the ten is due as before — each holds the app's answer.
    await mac.engine.sync();
    const onMac = await mac.cards();
    for (const lemma of LEMMAS) expect(reps(onMac[lemma]), lemma).toBe(1);

    // Safari wakes: it takes the relay (the same states) and syncs its whole deck.
    expect(await iphone.relay.collect()).toBe(10);
    await iphone.engine.sync();
    await mac.engine.sync();

    // Every host and the server converge, each card answered once.
    const onServer = server.cards();
    const onSafari = await iphone.cards();
    const onMacAfter = await mac.cards();
    for (const lemma of LEMMAS) {
      expect(reps(onServer.get(lemma)), lemma).toBe(1);
      expect(reps(onSafari[lemma]), lemma).toBe(1);
      expect(reps(onMacAfter[lemma]), lemma).toBe(1);
      expect(onSafari[lemma].client_ts).toBe(onServer.get(lemma)?.client_ts);
    }
    // Ten reviews in the app count ten in the account's statistics, not twenty.
    expect(server.reviews(DAY, "en")).toBe(10);
  });

  it("A change the extension made afterwards still wins", async () => {
    const server = fakeSyncServer();
    const group = appGroup();
    const iphone = safari(server, group);
    await iphone.seed();
    await iphone.engine.sync();
    await iphone.relay.accountChanged(true);
    const app = await openApp(server, group, 10_000);
    await review(app, ["one"], 10_000);
    await app.sync.background();

    // Later, in Safari, before it takes the relay: « one » answered again.
    await iphone.port.restore(((await iphone.store.get("lingua")).lingua as { backup: string }).backup);
    await iphone.port.applyCardOps([{ ...capturedCard("one", 50_000), fsrs_state: JSON.stringify({ reps: 7 }) }]);
    await saveBackup(iphone.store, await iphone.port.backup());
    await iphone.relay.collect();
    await iphone.engine.sync();

    const mac = extension(server, "mac");
    await mac.seed();
    await mac.engine.sync();
    expect(reps((await mac.cards()).one)).toBe(7);
    expect(reps((await iphone.cards()).one)).toBe(7);
  });

  it("Offline on the train: the reviews wait in the App Group and leave at the next opening", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const server = fakeSyncServer();
    const group = appGroup();
    const iphone = safari(server, group);
    await iphone.seed();
    await iphone.engine.sync();
    await iphone.relay.accountChanged(true);
    const local = memoryLocal();

    server.offline = true;
    const offline = await openApp(server, group, 10_000, local);
    await review(offline, ["one", "two", "three", "four", "five"], 10_000);
    await offline.sync.background();
    expect(offline.sync.line()).toBe("failed");
    expect(
      group
        .pendingOps()
        ?.cardOps.map((c) => c.lemma)
        .sort(),
    ).toEqual(["five", "four", "one", "three", "two"]);

    server.offline = false;
    const online = await openApp(server, group, 60_000, local);
    expect(online.sync.line()).toBe("synced");
    for (const lemma of ["one", "two", "three", "four", "five"]) expect(reps(server.cards().get(lemma)), lemma).toBe(1);
    // Still handed to the extension as well.
    expect(group.pendingOps()?.cardOps).toHaveLength(5);
  });

  it("Erased on the Mac, Safari not run since: no card from before, nothing pushed, nothing left to collect", async () => {
    const server = fakeSyncServer();
    const group = appGroup();
    const iphone = safari(server, group);
    await iphone.seed();
    await iphone.engine.sync();
    await iphone.relay.accountChanged(true);
    // An earlier opening reviewed offline: its changes wait in the App Group.
    server.offline = true;
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const earlier = await openApp(server, group, 10_000);
    await review(earlier, ["one", "two"], 10_000);
    server.offline = false;

    // On the Mac: « Effacer mes données Lingua ». Then a card captured there afterwards.
    const mac = extension(server, "mac");
    await mac.seed();
    await mac.engine.sync();
    server.erase(100_000);
    await mac.engine.sync();
    await mac.port.applyCardOps([capturedCard("fresh", 200_000)]);
    await saveBackup(mac.store, await mac.port.backup());
    await mac.engine.sync();
    server.calls.length = 0;

    const app = await openApp(server, group, 300_000);
    expect(Object.keys(app.port.state().cards)).toEqual(["fresh"]);
    expect(server.calls).not.toContain("pushCards");
    expect(server.calls).not.toContain("pushOps");
    // Nothing from before the erasure is left for the extension: no card of the old deck, no count of
    // the earlier opening. (A card pulled since, dated after the opening began, may ride along: the
    // extension applies it as it would a pull, D3.)
    const left = group.pendingOps();
    expect((left?.cardOps ?? []).filter((c) => LEMMAS.includes(c.lemma))).toEqual([]);
    expect(left?.statusOps ?? []).toEqual([]);
    for (const counts of Object.values(left?.daily ?? {})) expect(counts).toEqual({});
    expect(server.cards().has("one")).toBe(false);
    // Safari, waking, erases at its own sync and keeps none of the app's old answers.
    await iphone.relay.collect();
    await iphone.engine.sync();
    expect(Object.keys(await iphone.cards())).toEqual(["fresh"]);
  });

  it("Another account in Safari: the app sends no deck, and its reviews still reach the extension", async () => {
    const server = fakeSyncServer();
    const group = appGroup();
    const iphone = safari(server, group);
    await iphone.seed();
    await iphone.relay.accountChanged(true);
    server.calls.length = 0;
    const app = await openApp(server, group, 10_000, memoryLocal(), "someone-else");
    expect(app.session.sharedAccount).toBe(await accountDigest(USER));
    expect(app.sync.line()).toBe("otherAccount");
    await review(app, ["one"], 10_000);
    await app.sync.background();
    expect(server.calls).toEqual([]);
    expect(await iphone.relay.collect()).toBe(1);
  });
});
