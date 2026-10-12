import { describe, expect, it, vi } from "vitest";
import type { CardOp } from "@/analyzer/port.ts";
import { DAILY_KEY } from "@/state/dailystats.ts";
import { encodeAppOps, parseSharedState } from "@/state/app-relay.ts";
import type { NativeMessage } from "@/state/native-signin.ts";
import { reviewRelay, SHARE_DEBOUNCE_MS } from "@/state/review-relay.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { makeFakePort } from "./helpers.ts";

function area(seed: Record<string, unknown> = {}): AsyncStorageArea & { items: Record<string, unknown> } {
  const items: Record<string, unknown> = { ...seed };
  return {
    items,
    async get(keys) {
      const list = keys == null ? Object.keys(items) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in items).map((k) => [k, items[k]]));
    },
    async set(next) {
      Object.assign(items, next);
    },
  };
}

const v2 = (backup: string) => ({ lingua: { v: 2, backup } });
const card: CardOp = {
  client_id: "c",
  language: "en",
  lemma: "dog",
  surface_form: "dog",
  source_sentence: "",
  source: "",
  gloss: "",
  fsrs_state: "{}",
  deleted: false,
  client_ts: 5000,
  device_id: "",
};

/** A fake native port: the App Group as the handler keeps it. */
function native(waiting: { ops: string; writtenAt: number } | null = null) {
  const sent: NativeMessage[] = [];
  let pending = waiting;
  const send = vi.fn(async (message: NativeMessage): Promise<unknown> => {
    sent.push(message);
    switch (message.type) {
      case "review.readOps":
        return pending ?? {};
      case "review.clearOps":
        if (pending?.writtenAt === message.writtenAt) pending = null;
        return {};
      default:
        return {};
    }
  });
  return { send, sent, pending: () => pending };
}

function relay(
  over: { store?: ReturnType<typeof area>; send?: ReturnType<typeof native>["send"]; applied?: number } = {},
) {
  const store = over.store ?? area(v2("LOCAL"));
  const prefs = area({ "cymbra-lingua-interface-language": "en", "cymbra-lingua-session": "secret" });
  const { port } = makeFakePort();
  const restored: string[] = [];
  port.restore = async (b) => void restored.push(b);
  port.backup = async () => "MERGED";
  port.applyCardOps = async (ops) => over.applied ?? ops.length;
  port.applyStatusChanges = async () => 0;
  const exclusive = vi.fn((task: () => Promise<number>) => task());
  const timers: (() => void)[] = [];
  const r = reviewRelay({
    send: over.send ?? native().send,
    store,
    prefs,
    port,
    exclusive: exclusive as unknown as <T>(task: () => Promise<T>) => Promise<T>,
    now: () => 7777,
    setTimer: (run) => timers.push(run),
    clearTimer: () => void timers.pop(),
  });
  return { r, store, restored, exclusive, timers };
}

describe("the extension hands its state to the app (add-lingua-apple-review D3)", () => {
  it("sends the backup, the review's preferences and the daily counts, never anything else", async () => {
    const n = native();
    const store = area({ ...v2("LOCAL"), [DAILY_KEY]: { 1: {} }, "cymbra-lingua-device": "id" });
    const { r } = relay({ send: n.send, store });
    await r.share();
    const message = n.sent[0];
    expect(message.type).toBe("review.saveState");
    const state = parseSharedState(message.type === "review.saveState" ? message.state : null);
    expect(state).toMatchObject({ backup: "LOCAL", savedAt: 7777 });
    expect(state?.prefs).toEqual({ "cymbra-lingua-interface-language": "en" });
    expect(state?.storeItems).toEqual({ [DAILY_KEY]: { 1: {} } });
  });

  it("sends nothing before a backup exists, and survives a missing handler", async () => {
    const n = native();
    await relay({ send: n.send, store: area() }).r.share();
    expect(n.sent).toEqual([]);
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    await relay({ send: vi.fn(async () => Promise.reject(new Error("no handler"))) }).r.share();
    expect(warn).toHaveBeenCalled();
    warn.mockRestore();
  });

  it("hands a burst of saves over once", async () => {
    const n = native();
    const { r, timers } = relay({ send: n.send });
    r.stateChanged();
    r.stateChanged();
    expect(timers).toHaveLength(1);
    timers[0]();
    await vi.waitFor(() => expect(n.sent.map((m) => m.type)).toEqual(["review.saveState"]));
    expect(SHARE_DEBOUNCE_MS).toBeGreaterThan(0);
  });
});

describe("the extension takes back the app's review", () => {
  it("applies the changes on the latest backup, saves, then clears them", async () => {
    const n = native({ ops: encodeAppOps({ since: 0, cardOps: [card], statusOps: [] }), writtenAt: 11 });
    const { r, store, restored, exclusive } = relay({ send: n.send });
    expect(await r.collect()).toBe(1);
    expect(exclusive).toHaveBeenCalledOnce();
    expect(restored).toEqual(["LOCAL"]);
    expect(store.items.lingua).toEqual({ v: 2, backup: "MERGED" });
    expect(n.pending()).toBeNull();
  });

  it("adds the day's reviews counted in the app, once", async () => {
    const daily = { opening: { "20000": { en: { reviews: 2, wordsLearned: 1, native: "fr" as const } } } };
    const payload = encodeAppOps({ since: 0, cardOps: [card], statusOps: [], daily });
    const store = area(v2("LOCAL"));
    await relay({ send: native({ ops: payload, writtenAt: 1 }).send, store }).r.collect();
    await relay({ send: native({ ops: payload, writtenAt: 2 }).send, store }).r.collect();
    const day = (
      store.items[DAILY_KEY] as Record<string, Record<string, { reviews: number; wordsLearned: number }>>
    )[20000];
    expect(day.en).toMatchObject({ reviews: 2, wordsLearned: 1 });
  });

  it("saves nothing when nothing changed, and still clears", async () => {
    const n = native({ ops: encodeAppOps({ since: 0, cardOps: [card], statusOps: [] }), writtenAt: 11 });
    const { r, store } = relay({ send: n.send, applied: 0 });
    expect(await r.collect()).toBe(0);
    expect(store.items.lingua).toEqual({ v: 2, backup: "LOCAL" });
    expect(n.pending()).toBeNull();
  });

  it("does nothing when nothing waits or the handler is missing", async () => {
    const n = native();
    expect(await relay({ send: n.send }).r.collect()).toBe(0);
    expect(n.sent.map((m) => m.type)).toEqual(["review.readOps"]);
    expect(await relay({ send: vi.fn(async () => Promise.reject(new Error("x"))) }).r.collect()).toBe(0);
  });

  it("drops unreadable changes without applying them", async () => {
    const n = native({ ops: "garbage", writtenAt: 3 });
    const { r, exclusive } = relay({ send: n.send });
    expect(await r.collect()).toBe(0);
    expect(exclusive).not.toHaveBeenCalled();
    expect(n.pending()).toBeNull();
  });

  it("keeps the changes for the next time when they cannot be saved", async () => {
    const n = native({ ops: encodeAppOps({ since: 0, cardOps: [card], statusOps: [] }), writtenAt: 11 });
    const store = area(v2("LOCAL"));
    store.set = async () => Promise.reject(new Error("disk"));
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    expect(await relay({ send: n.send, store }).r.collect()).toBe(0);
    expect(n.pending()).not.toBeNull();
    warn.mockRestore();
  });

  it("applies nothing without a local backup", async () => {
    const n = native({ ops: encodeAppOps({ since: 0, cardOps: [card], statusOps: [] }), writtenAt: 11 });
    const { r, restored } = relay({ send: n.send, store: area() });
    expect(await r.collect()).toBe(0);
    expect(restored).toEqual([]);
  });

  it("runs one collection at a time", async () => {
    const n = native({ ops: encodeAppOps({ since: 0, cardOps: [card], statusOps: [] }), writtenAt: 11 });
    const { r } = relay({ send: n.send });
    const [a, b] = [r.collect(), r.collect()];
    expect(a).toBe(b);
    await a;
  });
});
