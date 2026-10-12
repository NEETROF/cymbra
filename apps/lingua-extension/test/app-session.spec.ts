import { describe, expect, it, vi } from "vitest";
import type { CardOp } from "@/analyzer/port.ts";
import type { AppMessage } from "@/app/bridge.ts";
import { webkitSend } from "@/app/bridge.ts";
import { APP_PREFS_KEY, openAppSession } from "@/app/session.ts";
import { appCopy, renderLanguageDue, savedAtLine } from "@/app/view.ts";
import { encodeAppOps, encodeSharedState, parseAppOps } from "@/state/app-relay.ts";
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
    const saved = vi.fn();
    session?.onSaved(saved);
    await session?.store.set({ "cymbra-lingua-daily-v4": {} });
    expect(b.written).toEqual([]);
    await session?.store.set({ lingua: { v: 2, backup: "X" } });
    const ops = parseAppOps(b.written[0]);
    expect(ops?.since).toBe(5000);
    expect(ops?.cardOps.map((c) => c.lemma)).toEqual(["graded"]);
    expect(saved).toHaveBeenCalledOnce();
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

  it("speaks the extension's interface language, else the app's, else French", async () => {
    const open = (prefs: Record<string, unknown>, fallbackLanguage?: string) =>
      openAppSession({ send: bridge(shared(prefs)).send, port: () => fakePort().port, now: () => 0, fallbackLanguage });
    expect((await open({ "cymbra-lingua-interface-language": "es" }, "en"))?.interfaceLanguage).toBe("es");
    expect((await open({}, "en"))?.interfaceLanguage).toBe("en");
    expect((await open({}, "xx"))?.interfaceLanguage).toBe("fr");
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

  it("gives each studied language its due count", async () => {
    const { port } = makeFakePort();
    port.studiedLanguages = async () => ["en", "es"];
    port.dueCount = async (_now, languages) => (languages?.[0] === "en" ? 3 : 1);
    const list = document.createElement("ul");
    await renderLanguageDue(list, port, 0, "fr");
    expect([...list.children].map((li) => li.textContent)).toEqual(["Anglais : 3 à revoir", "Espagnol : 1 à revoir"]);
    expect(appCopy("es").review).toBe("Repaso");
  });
});
