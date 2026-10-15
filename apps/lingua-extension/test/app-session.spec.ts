import { describe, expect, it, vi } from "vitest";
import type { CardOp } from "@/analyzer/port.ts";
import type { AppMessage } from "@/app/bridge.ts";
import { webkitSend } from "@/app/bridge.ts";
import { APP_PREFS_KEY, openAppSession } from "@/app/session.ts";
import { appCopy, savedAtLine } from "@/app/view.ts";
import { encodeAppOps, encodeSharedState, parseAppOps } from "@/state/app-relay.ts";
import { DAILY_KEY, loadDailyStats, recordReview } from "@/state/dailystats.ts";
import { makeFakePort } from "./helpers.ts";

const card = (client_ts: number, lemma = "dog"): CardOp => ({
  client_id: lemma,
  language: "en",
  lemma,
  surface_form: lemma,
  source_sentence: "",
  source: "",
  gloss: "",
  fsrs_state: "{}",
  deleted: false,
  client_ts,
  device_id: "",
});

/** The App Group as the app's bridge sees it. */
function bridge(state: string | null, pending: string | null = null) {
  const written: string[] = [];
  const send = vi.fn(async (message: AppMessage): Promise<unknown> => {
    switch (message.type) {
      case "review.state":
        return state === null ? {} : { state };
      case "review.pendingOps":
        return pending === null ? {} : { ops: pending, writtenAt: 1 };
      case "review.writeOps":
        written.push(message.ops);
        return {};
      default:
        return {};
    }
  });
  return { send, written };
}

function fakePort() {
  const { port } = makeFakePort();
  const calls = { restored: [] as string[], appliedCards: [] as CardOp[][] };
  port.restore = async (b) => void calls.restored.push(b);
  port.backup = async () => "ENGINE";
  port.applyCardOps = async (ops) => (calls.appliedCards.push(ops), ops.length);
  port.applyStatusChanges = async () => 0;
  port.exportStatusOps = async () => [];
  return { port, calls };
}

const shared = (prefs: Record<string, unknown> = {}) =>
  encodeSharedState({ backup: "SHARED", savedAt: 99, prefs, storeItems: { daily: 1 } });

describe("the app opens on the extension's state (add-lingua-apple-review)", () => {
  it("is nothing before the extension has shared its state", async () => {
    const { port } = fakePort();
    expect(await openAppSession({ send: bridge(null).send, port: () => port, now: () => 0 })).toBeNull();
  });

  it("restores the shared backup, in an engine built for it", async () => {
    const { port, calls } = fakePort();
    const built: string[] = [];
    const session = await openAppSession({
      send: bridge(shared()).send,
      port: (b) => (built.push(b), port),
      now: () => 5_500,
    });
    expect(built).toEqual(["SHARED"]);
    expect(calls.restored).toEqual(["SHARED"]);
    expect(session?.savedAt).toBe(99);
    expect(await session?.store.get(["lingua", "daily", "absent"])).toEqual({
      lingua: { v: 2, backup: "ENGINE" },
      daily: 1,
    });
    expect(await session?.store.get(null)).toHaveProperty("daily");
    expect(await session?.store.get("daily")).toEqual({ daily: 1 });
  });

  it("writes the review's changes since it opened after every save of the backup", async () => {
    const { port } = fakePort();
    port.exportCardOps = async () => [card(1000, "before"), card(6000, "graded")];
    const b = bridge(shared());
    const session = await openAppSession({ send: b.send, port: () => port, now: () => 5_500 });
    await session?.store.set({ "cymbra-lingua-device": "id" });
    expect(b.written).toEqual([]);
    await session?.store.set({ lingua: { v: 2, backup: "X" } });
    const ops = parseAppOps(b.written[0]);
    expect(ops?.since).toBe(5000);
    expect(ops?.cardOps.map((c) => c.lemma)).toEqual(["graded"]);
  });

  it("lays the changes the extension has not taken back on the state, and keeps writing them", async () => {
    const { port, calls } = fakePort();
    port.exportCardOps = async () => [card(2000, "earlier"), card(9000, "now")];
    const pending = encodeAppOps({ since: 2000, cardOps: [card(2000, "earlier")], statusOps: [] });
    const b = bridge(shared(), pending);
    const session = await openAppSession({ send: b.send, port: () => port, now: () => 9_000 });
    expect(calls.appliedCards.map((ops) => ops.map((c) => c.lemma))).toEqual([["earlier"]]);
    await session?.store.set({ lingua: { v: 2, backup: "X" } });
    expect(parseAppOps(b.written[0])?.cardOps.map((c) => c.lemma)).toEqual(["earlier", "now"]);
  });

  it("counts the day's reviews of this opening, beside the earlier openings' not yet taken", async () => {
    const { port } = fakePort();
    port.exportCardOps = async () => [];
    const day = 20000;
    const earlier = { first: { [day]: { en: { reviews: 2, wordsLearned: 0, native: "fr" as const } } } };
    const sharedState = encodeSharedState({
      backup: "SHARED",
      savedAt: 1,
      prefs: {},
      storeItems: {
        [DAILY_KEY]: { [day]: { en: { exposures: 5, unknownSeen: 0, wordsLearned: 0, reviews: 1, native: "fr" } } },
      },
    });
    const b = bridge(sharedState, encodeAppOps({ since: 0, cardOps: [], statusOps: [], daily: earlier }));
    const session = await openAppSession({ send: b.send, port: () => port, now: () => 0, openingId: "second" });
    // The statistics show the shared counts with the earlier opening's.
    expect((await loadDailyStats(session!.store))[day].en.reviews).toBe(3);
    // A grade: its backup first, then its count.
    await session!.store.set({ lingua: { v: 2, backup: "X" } });
    expect(parseAppOps(b.written[0])?.daily).toEqual({ ...earlier, second: {} });
    await recordReview(session!.store, day, "en", "fr");
    expect(parseAppOps(b.written[1])?.daily).toEqual({
      ...earlier,
      second: { [day]: { en: { reviews: 1, wordsLearned: 0, native: "fr" } } },
    });
  });

  it("says the state and its language before the engine starts, so the loading copy follows them", async () => {
    const order: string[] = [];
    const { port } = fakePort();
    port.restore = async () => void order.push("engine");
    await openAppSession({
      send: bridge(
        shared({ "cymbra-lingua-interface-language": "es", "cymbra-lingua-colours": { preset: "eink-mono" } }),
      ).send,
      port: () => port,
      now: () => 0,
      fallbackLanguage: "en",
      onState: (state, language) =>
        order.push(`state:${language}:${JSON.stringify(state.prefs["cymbra-lingua-colours"])}`),
    });
    expect(order).toEqual(['state:es:{"preset":"eink-mono"}', "engine"]);
  });

  it("names each opening on its own unless told", async () => {
    const { port } = fakePort();
    port.exportCardOps = async () => [];
    const b = bridge(shared());
    const session = await openAppSession({ send: b.send, port: () => port, now: () => 0 });
    await session!.store.set({ lingua: { v: 2, backup: "X" } });
    expect(Object.keys(parseAppOps(b.written[0])?.daily ?? {})).toHaveLength(1);
  });

  it("speaks the extension's interface language, else the app's, else French", async () => {
    const open = (prefs: Record<string, unknown>, fallbackLanguage?: string) =>
      openAppSession({ send: bridge(shared(prefs)).send, port: () => fakePort().port, now: () => 0, fallbackLanguage });
    expect((await open({ "cymbra-lingua-interface-language": "es" }, "en"))?.interfaceLanguage).toBe("es");
    expect((await open({}, "en"))?.interfaceLanguage).toBe("en");
    expect((await open({}, "xx"))?.interfaceLanguage).toBe("fr");
    // The language the extension names, apart (add-lingua-app-email-signin D3): the email requests' locale.
    expect((await open({ "cymbra-lingua-interface-language": "es" }, "en"))?.sharedLanguage).toBe("es");
    expect((await open({ "cymbra-lingua-interface-language": "xx" }, "en"))?.sharedLanguage).toBeNull();
    expect((await open({}, "en"))?.sharedLanguage).toBeNull();
    const session = await open({}, "en");
    expect(await session?.prefs.get("cymbra-lingua-interface-language")).toEqual({
      "cymbra-lingua-interface-language": "en",
    });
  });

  it("reads the extension's preferences under what the app chose itself, kept in its own storage", async () => {
    const local = new Map<string, string>();
    const storage = {
      getItem: (k: string) => local.get(k) ?? null,
      setItem: (k: string, v: string) => void local.set(k, v),
    };
    const open = () =>
      openAppSession({
        send: bridge(shared({ "cymbra-lingua-review-language": "en", "cymbra-lingua-new-words-per-day": 5 })).send,
        port: () => fakePort().port,
        now: () => 0,
        local: storage,
      });
    const first = await open();
    await first?.prefs.set({ "cymbra-lingua-review-language": "es" });
    expect(JSON.parse(local.get(APP_PREFS_KEY) ?? "{}")).toEqual({ "cymbra-lingua-review-language": "es" });
    const again = await open();
    expect(await again?.prefs.get(["cymbra-lingua-review-language", "cymbra-lingua-new-words-per-day"])).toEqual({
      "cymbra-lingua-review-language": "es",
      "cymbra-lingua-new-words-per-day": 5,
    });
  });

  it("keeps a choice for the launch when its storage refuses, and reads garbage as nothing chosen", async () => {
    const storage = {
      getItem: () => "{not json",
      setItem: () => {
        throw new Error("quota");
      },
    };
    const session = await openAppSession({
      send: bridge(shared()).send,
      port: () => fakePort().port,
      now: () => 0,
      local: storage,
    });
    await session?.prefs.set({ "cymbra-lingua-review-language": "es" });
    expect(await session?.prefs.get("cymbra-lingua-review-language")).toEqual({
      "cymbra-lingua-review-language": "es",
    });
  });
});

describe("the app's bridge", () => {
  it("is the web view's handler, or nothing outside the app", async () => {
    expect(webkitSend()).toBeNull();
    const postMessage = vi.fn(async () => ({ ok: true }));
    (globalThis as { webkit?: unknown }).webkit = { messageHandlers: { lingua: { postMessage } } };
    try {
      expect(await webkitSend()?.({ type: "review.state" })).toEqual({ ok: true });
      expect(postMessage).toHaveBeenCalledWith({ type: "review.state" });
    } finally {
      delete (globalThis as { webkit?: unknown }).webkit;
    }
  });
});

describe("the app's home", () => {
  it("says when the extension saved the deck, or nothing when it did not say", () => {
    expect(savedAtLine("fr", 0)).toBeNull();
    expect(savedAtLine("en", Date.UTC(2026, 9, 12, 9, 41))).toMatch(/^Cards from Safari, /);
  });

  it("speaks each interface language", () => {
    expect(appCopy("es").review).toBe("Repaso");
  });
});

describe("the opening serves the app's synchronisation (add-lingua-app-deck-sync)", () => {
  const status = (updated_at: number, lemma = "dog") => ({
    language: "en",
    lemma,
    status: "known",
    provenance: "srs",
    updated_at,
  });

  it("names the extension's account and erasure mark as the state carries them", async () => {
    const { port } = fakePort();
    const withAccount = encodeSharedState({
      backup: "SHARED",
      savedAt: 1,
      prefs: {},
      storeItems: {},
      account: "digest",
      erasedAt: 42,
    });
    const session = await openAppSession({ send: bridge(withAccount).send, port: () => port, now: () => 0 });
    expect(session?.sharedAccount).toBe("digest");
    expect(session?.sharedErasedAt).toBe(42);
    const older = await openAppSession({ send: bridge(shared()).send, port: () => fakePort().port, now: () => 0 });
    expect(older?.sharedAccount).toBeUndefined();
    expect(older?.sharedErasedAt).toBe(0);
  });

  it("pushes what it hands over: the changes since the opening began, and says each save", async () => {
    const { port } = fakePort();
    port.exportCardOps = async () => [card(1000, "before"), card(6000, "graded")];
    port.exportStatusOps = async () => [status(1000, "before"), status(7000, "known")];
    const onSaved = vi.fn();
    const b = bridge(shared());
    const session = await openAppSession({ send: b.send, port: () => port, now: () => 5_500, onSaved });
    expect(session?.since).toBe(5000);
    const ops = await session!.pendingOps();
    expect(ops.cards.map((c) => c.lemma)).toEqual(["graded"]);
    expect(ops.statuses.map((s) => s.lemma)).toEqual(["known"]);
    await session!.store.set({ "cymbra-lingua-status-cursor": 1 });
    expect(onSaved).not.toHaveBeenCalled();
    await session!.store.set({ lingua: { v: 2, backup: "X" } });
    expect(onSaved).toHaveBeenCalledOnce();
    // The daily payload as before: the app's counts still travel to the extension.
    expect(parseAppOps(b.written[0])).toHaveProperty("daily");
  });

  it("after an erasure, keeps only the changes made after its mark, and the earlier openings' counts are gone", async () => {
    const day = 20000;
    const earlier = { first: { [day]: { en: { reviews: 2, wordsLearned: 0, native: "fr" as const } } } };
    const pending = encodeAppOps({
      since: 1000,
      cardOps: [card(1000, "before"), card(9000, "after")],
      statusOps: [status(1000, "old")],
      daily: earlier,
    });
    const { port, calls } = fakePort();
    let exported: CardOp[] = [];
    port.applyCardOps = async (ops) => {
      calls.appliedCards.push(ops);
      exported = ops;
      return ops.length;
    };
    port.exportCardOps = async () => exported;
    const b = bridge(shared(), pending);
    const session = await openAppSession({ send: b.send, port: () => port, now: () => 10_000, openingId: "now" });
    calls.appliedCards.length = 0;
    exported = [];
    await session!.discardPending(5000);
    expect(calls.appliedCards.map((ops) => ops.map((c) => c.lemma))).toEqual([["after"]]);
    const written = parseAppOps(b.written.at(-1));
    expect(written?.cardOps.map((c) => c.lemma)).toEqual(["after"]);
    expect(written?.statusOps).toEqual([]);
    expect(written?.daily).toEqual({ now: {} });
  });

  it("after an erasure with nothing newer, writes an empty payload", async () => {
    const { port } = fakePort();
    port.exportCardOps = async () => [];
    const b = bridge(shared(), encodeAppOps({ since: 1000, cardOps: [card(1000, "before")], statusOps: [] }));
    const session = await openAppSession({ send: b.send, port: () => port, now: () => 10_000 });
    await session!.discardPending(5000);
    const written = parseAppOps(b.written.at(-1));
    expect(written?.cardOps).toEqual([]);
    expect(written?.statusOps).toEqual([]);
  });
});
