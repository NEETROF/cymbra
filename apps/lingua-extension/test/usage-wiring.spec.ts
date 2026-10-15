import { describe, expect, it, vi } from "vitest";
import { INTERFACE_LANGUAGE_KEY } from "@/i18n/language.ts";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import type { UsageReport } from "@/usage/client.ts";
import { type Consent, RETRY_AFTER_MS, USAGE_COUNTS_KEY, type UsageState } from "@/usage/counts.ts";
import { UsageHost } from "@/usage/host.ts";
import { attachUsage, isAnalysedPage, type UsageEvents } from "@/usage/wiring.ts";

// The counts' wiring in the background (add-lingua-usage-counts task 4.3, review of #958): which of
// the background's events is a use, which only a wake, and which follow the consent and the
// interface language — driven through fake events onto a real host.

type Listener = (...args: never[]) => unknown;

function event<L extends Listener>() {
  const listeners: L[] = [];
  return {
    listeners,
    addListener: (l: L) => void listeners.push(l),
    fire: (...args: Parameters<L>) => listeners.map((l) => l(...args)),
  };
}

/** Wednesday 21 October 2026, local noon. */
const START = new Date(2026, 9, 21, 12);
const DAY_MS = 86_400_000;
/** The Monday after START's week, 9 a.m. */
const NEXT_MONDAY = new Date(2026, 9, 26, 9);
const TAB = { tab: { id: 7 } };

function wired(opts: { browser?: "chromium" | "firefox" | "safari"; consent?: Consent } = {}) {
  const store: Record<string, unknown> = {};
  const area: AsyncStorageArea = {
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      return Object.fromEntries(list.filter((k) => k in store).map((k) => [k, store[k]]));
    },
    async set(items) {
      Object.assign(store, structuredClone(items));
    },
  };
  let now = START;
  let consent: Consent = opts.consent ?? { kind: "switch", off: false };
  let outcome: "accepted" | "retry" = "accepted";
  const reports: UsageReport[] = [];
  const urls: string[] = [];
  const browser = opts.browser ?? "chromium";
  const host = new UsageHost({
    area,
    mode: browser === "chromium" ? "ask" : "on",
    browser,
    version: "1.10.0",
    consent: async () => consent,
    client: () => ({
      report: async (r) => {
        reports.push(r);
        return outcome;
      },
    }),
    now: () => now,
    language: async () => (store[INTERFACE_LANGUAGE_KEY] as InterfaceLanguage | undefined) ?? "fr",
    setUninstallURL: async (url) => void urls.push(url),
  });
  const onMessage = event<Parameters<UsageEvents["onMessage"]["addListener"]>[0]>();
  const onInstalled = event<(d: { reason: string }) => void>();
  const onStartup = event<() => void>();
  const onStorageChanged = event<(changes: Record<string, unknown>, area: string) => void>();
  const onPermissionsAdded = event<(p: unknown) => void>();
  const onPermissionsRemoved = event<(p: unknown) => void>();
  attachUsage(host, {
    onMessage,
    onInstalled,
    onStartup,
    onStorageChanged,
    ...(browser === "firefox" ? { onPermissionsAdded, onPermissionsRemoved } : {}),
  });
  /** Let the fire-and-forget work the listeners started finish. */
  const settle = async (): Promise<void> => {
    await host.status();
    await host.registerLeavingPage();
    urls.pop();
  };
  return {
    host,
    store,
    reports,
    urls,
    onMessage,
    onInstalled,
    onStartup,
    onStorageChanged,
    onPermissionsAdded,
    onPermissionsRemoved,
    settle,
    held: () => (store[USAGE_COUNTS_KEY] as UsageState | undefined)?.waiting.map((w) => w.step) ?? [],
    advanceTo: (d: Date) => (now = d),
    advance: (ms: number) => (now = new Date(now.getTime() + ms)),
    setConsent: (c: Consent) => (consent = c),
    setOutcome: (o: "accepted" | "retry") => (outcome = o),
  };
}

const send = (t: ReturnType<typeof wired>, message: unknown, sender: object = TAB) =>
  t.onMessage.fire(message, sender as { tab?: { id?: number } }, () => {});

describe("which message is a use", () => {
  it("a stats message is an analysed page only with a percentage, from a tab, reader on", () => {
    expect(isAnalysedPage({ type: "stats", pct: 40, language: "es" }, TAB)).toBe(true);
    expect(isAnalysedPage({ type: "stats", pct: 0 }, TAB)).toBe(true);
    expect(isAnalysedPage({ type: "stats", pct: null, language: "es" }, TAB)).toBe(false);
    expect(isAnalysedPage({ type: "stats", pct: 40, disabled: true }, TAB)).toBe(false);
    expect(isAnalysedPage({ type: "stats", pct: 40 }, {})).toBe(false);
    expect(isAnalysedPage({ type: "openPanel" }, TAB)).toBe(false);
    expect(isAnalysedPage(null, TAB)).toBe(false);
  });

  it("A page in no studied language: on day 1, back on day 1 is not recorded", async () => {
    const t = wired();
    t.onInstalled.fire({ reason: "install" });
    await t.settle();
    t.advance(DAY_MS);
    send(t, { type: "stats", pct: null, language: "es" });
    send(t, { type: "sync:request", reason: "page" });
    await t.settle();
    expect(t.held()).toEqual(["installed"]);
    // A page analysed in a studied language is the first analysis and a use.
    send(t, { type: "stats", pct: 35, language: "es" });
    await t.settle();
    expect(t.held()).toEqual(["installed", "first_analysis", "returned_day_1"]);
  });

  it("a surface the reader opened is a use; a page load is only a wake; a forced sync is neither", async () => {
    const t = wired();
    t.onInstalled.fire({ reason: "install" });
    await t.settle();
    t.advance(DAY_MS);
    send(t, { type: "sync:request", force: true });
    send(t, { type: "sync:request", reason: "page" });
    send(t, { type: "sync:request" });
    await t.settle();
    expect(t.held()).toEqual(["installed"]);
    send(t, { type: "sync:request", reason: "surface" });
    await t.settle();
    expect(t.held()).toEqual(["installed", "returned_day_1"]);
  });

  it("the welcome tab opening is not a use: its messages record nothing", async () => {
    const t = wired({ browser: "safari", consent: { kind: "switch", off: undefined } });
    t.onInstalled.fire({ reason: "install" });
    await t.settle();
    t.advance(DAY_MS);
    // What the welcome tab sends: the sentence's status, then that it was shown.
    const replies: unknown[] = [];
    const kept = t.onMessage.fire({ type: "usage:status" }, TAB, (r) => void replies.push(r));
    expect(kept).toContain(true); // answered asynchronously
    send(t, { type: "usage:notice-shown" });
    await t.settle();
    expect(replies).toEqual([{ notice: "inform" }]);
    expect(t.held()).toEqual(["installed"]);
  });
});

describe("the wakes that send what may leave", () => {
  it("an update starts nothing; an installation is day 0", async () => {
    const t = wired();
    t.onInstalled.fire({ reason: "update" });
    await t.settle();
    expect(t.held()).toEqual([]);
    t.onInstalled.fire({ reason: "install" });
    await t.settle();
    expect(t.held()).toEqual(["installed"]);
  });

  it("a browser start retries what a failure kept, an hour later", async () => {
    const t = wired();
    t.setOutcome("retry");
    t.onInstalled.fire({ reason: "install" });
    await t.settle();
    t.advanceTo(NEXT_MONDAY);
    t.onStartup.fire();
    await t.settle();
    expect(t.reports).toHaveLength(1);
    t.setOutcome("accepted");
    t.advance(RETRY_AFTER_MS);
    t.onStartup.fire();
    await t.settle();
    expect(t.reports.map((r) => r.steps)).toEqual([["installed"], ["installed"]]);
    expect(t.held()).toEqual([]);
  });
});

describe("what the leaving page follows", () => {
  it("is set from the start, then follows the interface language", async () => {
    const t = wired({ browser: "firefox", consent: { kind: "grant", supported: true, granted: true } });
    // From the start, before any installation is recorded: nothing to leave yet.
    await t.host.status();
    expect(t.urls).toEqual([""]);
    t.onInstalled.fire({ reason: "install" });
    await t.settle();
    expect(t.urls.at(-1)).toBe("https://cymbra.app/lingua/au-revoir/?b=firefox&v=1.10");
    t.store[INTERFACE_LANGUAGE_KEY] = "es";
    t.onStorageChanged.fire({ [INTERFACE_LANGUAGE_KEY]: { newValue: "es" } }, "local");
    await t.settle();
    expect(t.urls.at(-1)).toBe("https://cymbra.app/es/lingua/adios/?b=firefox&v=1.10");
    const count = t.urls.length;
    // Another key, or another area, changes nothing.
    t.onStorageChanged.fire({ "cymbra-lingua-enabled": { newValue: false } }, "local");
    t.onStorageChanged.fire({ [INTERFACE_LANGUAGE_KEY]: { newValue: "en" } }, "sync");
    await t.settle();
    expect(t.urls).toHaveLength(count);
  });

  it("Firefox: its consent given or withdrawn in the add-on manager is followed", async () => {
    const t = wired({ browser: "firefox", consent: { kind: "grant", supported: true, granted: true } });
    t.onInstalled.fire({ reason: "install" });
    await t.settle();
    t.setConsent({ kind: "grant", supported: true, granted: false });
    // A permission that is not the data-collection consent is not followed.
    t.onPermissionsRemoved.fire({ origins: ["<all_urls>"] });
    await t.settle();
    expect(t.held()).toEqual(["installed"]);
    t.onPermissionsRemoved.fire({ data_collection: ["technicalAndInteraction"] });
    await t.settle();
    expect(t.held()).toEqual([]);
    expect(t.urls.at(-1)).toBe("");
    t.setConsent({ kind: "grant", supported: true, granted: true });
    t.onPermissionsAdded.fire({ data_collection: ["technicalAndInteraction"] });
    await t.settle();
    expect(t.urls.at(-1)).toBe("https://cymbra.app/lingua/au-revoir/?b=firefox&v=1.10");
  });

  it("other browsers do not listen to permission changes", () => {
    const t = wired({ browser: "chromium" });
    expect(t.onPermissionsAdded.listeners).toHaveLength(0);
    expect(t.onPermissionsRemoved.listeners).toHaveLength(0);
  });

  it("a usage message that fails is answered with nothing", async () => {
    const t = wired();
    const failing = vi.spyOn(t.host, "handle").mockRejectedValueOnce(new Error("boom"));
    const replies: unknown[] = [];
    t.onMessage.fire({ type: "usage:status" }, TAB, (r) => void replies.push(r));
    await t.host.status();
    expect(failing).toHaveBeenCalled();
    expect(replies).toEqual([undefined]);
  });
});
