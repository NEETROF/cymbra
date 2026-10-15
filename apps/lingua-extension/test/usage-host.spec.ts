// Paris leaves summer time on Sunday 25 October 2026, in the first week of START's installation:
// the midnight and daylight-saving tests below run across it. Set before any date is made.
process.env.TZ = "Europe/Paris";

import { describe, expect, it, vi } from "vitest";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import type { UsageReport } from "@/usage/client.ts";
import {
  type BrowserFamily,
  type Consent,
  type CountsMode,
  forgetsOn,
  localDay,
  type ReportOutcome,
  RETRY_AFTER_MS,
  USAGE_COUNTS_KEY,
  USAGE_COUNTS_OFF_KEY,
  type UsageState,
} from "@/usage/counts.ts";
import { UsageHost } from "@/usage/host.ts";

// The background's side of the counts (add-lingua-usage-counts D1, D5, D6, D7, D12): each trigger
// records a step and, at that same wake, sends what waits, filed under day 0 — as each browser's
// consent allows: nothing on Chromium before "yes", nothing in Safari before the sentence, on
// Firefox only with its consent, and never for an installation that predates the counts.

interface FakeArea extends AsyncStorageArea {
  store: Record<string, unknown>;
  /** The next write this matches throws (the background stopped before it was saved). */
  failSetIf: ((items: Record<string, unknown>) => boolean) | null;
}

function fakeArea(store: Record<string, unknown> = {}): FakeArea {
  const area: FakeArea = {
    store,
    failSetIf: null,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in store) out[k] = store[k];
      return out;
    },
    async set(items) {
      if (area.failSetIf?.(items)) {
        area.failSetIf = null;
        throw new Error("the service worker was stopped");
      }
      Object.assign(store, structuredClone(items));
    },
  };
  return area;
}

/** Wednesday 21 October 2026, local noon. */
const START = new Date(2026, 9, 21, 12);
const DAY_MS = 86_400_000;

interface Setup {
  mode?: CountsMode;
  browser?: BrowserFamily;
  consent?: Consent;
  outcome?: ReportOutcome;
  language?: InterfaceLanguage;
  firefox?: { request: boolean };
  area?: FakeArea;
  now?: Date;
}

function setup(opts: Setup = {}) {
  const area = opts.area ?? fakeArea();
  let consent: Consent = opts.consent ?? { kind: "switch", off: undefined };
  let now = opts.now ?? START;
  const reports: UsageReport[] = [];
  let outcome: ReportOutcome = opts.outcome ?? "accepted";
  const report = vi.fn(async (r: UsageReport) => {
    reports.push(r);
    return outcome;
  });
  const urls: string[] = [];
  const browser = opts.browser ?? "chromium";
  const dataCollection = opts.firefox
    ? {
        request: vi.fn(async () => {
          if (!opts.firefox!.request)
            throw new Error("permissions.request may only be called from a user input handler");
          consent = { kind: "grant", supported: true, granted: true };
          return true;
        }),
        remove: vi.fn(async () => {
          consent = { kind: "grant", supported: true, granted: false };
          return true;
        }),
      }
    : undefined;
  const host = new UsageHost({
    area,
    mode: opts.mode ?? (browser === "chromium" ? "ask" : "on"),
    browser,
    version: "1.10.3",
    consent: async () =>
      consent.kind === "switch"
        ? {
            kind: "switch",
            off:
              typeof area.store[USAGE_COUNTS_OFF_KEY] === "boolean"
                ? (area.store[USAGE_COUNTS_OFF_KEY] as boolean)
                : consent.off,
          }
        : consent,
    client: () => ({ report }),
    now: () => now,
    language: async () => opts.language ?? "fr",
    setUninstallURL:
      browser === "safari"
        ? null
        : async (url) => {
            urls.push(url);
          },
    dataCollection,
  });
  return {
    host,
    area,
    reports,
    report,
    urls,
    dataCollection,
    state: () => area.store[USAGE_COUNTS_KEY] as UsageState | undefined,
    held: () => (area.store[USAGE_COUNTS_KEY] as UsageState | undefined)?.waiting,
    steps: () => reports.map((r) => r.steps),
    setConsent: (c: Consent) => (consent = c),
    setOutcome: (o: ReportOutcome) => (outcome = o),
    advance: (ms: number) => (now = new Date(now.getTime() + ms)),
    advanceTo: (date: Date) => (now = date),
  };
}

/** Day 0 of an installation made at START: what every report of it is filed under. */
const DAY0 = localDay(START);
const granted: Consent = { kind: "grant", supported: true, granted: true };

describe("Chromium asks before anything leaves", () => {
  it("Chrome, agreed on the welcome tab: installed and set up leave in one report at the press, none before it", async () => {
    const t = setup();
    await t.host.installed("install");
    await t.host.setUp();
    t.advance(DAY_MS); // the reader answers the next day, after a few wakes
    await t.host.wake();
    expect(t.reports).toEqual([]);
    expect(await t.host.status()).toBe("ask");
    await t.host.answer(true);
    expect(t.reports).toEqual([
      { installDay: DAY0, browser: "chromium", version: "1.10", steps: ["installed", "set_up"] },
    ]);
    expect(t.state()?.waiting).toEqual([]);
    expect(await t.host.status()).toBe("none");
    // The leaving page is registered once the reader agreed.
    expect(t.urls.at(-1)).toBe("https://cymbra.app/lingua/au-revoir/?b=chromium&v=1.10");
  });

  it("Chrome, never answered: no step is ever reported", async () => {
    const t = setup();
    await t.host.installed("install");
    await t.host.analysed();
    t.advance(DAY_MS);
    await t.host.used();
    await t.host.wake();
    expect(t.reports).toEqual([]);
    expect(t.held()).toEqual(["installed", "first_analysis", "returned_day_1"]);
    await t.host.registerLeavingPage();
    expect(t.urls.at(-1)).toBe("");
  });

  it("Chrome, declined: the held steps and day 0 are deleted, nothing is reported, the switch shows off", async () => {
    const t = setup();
    await t.host.installed("install");
    await t.host.answer(false);
    expect(t.state()?.day0).toBeNull();
    expect(t.state()?.waiting).toEqual([]);
    expect(t.area.store[USAGE_COUNTS_OFF_KEY]).toBe(true);
    await t.host.analysed();
    t.advance(DAY_MS);
    await t.host.wake();
    expect(t.reports).toEqual([]);
    expect(await t.host.switchState()).toEqual({ shown: true, on: false });
    expect(t.urls.at(-1)).toBe("");
  });

  it("ticking the switch is a yes: what waited leaves at once", async () => {
    const t = setup();
    await t.host.installed("install");
    const reply = await t.host.setSwitch(true);
    expect(reply).toEqual({ view: { shown: true, on: true } });
    expect(t.steps()).toEqual([["installed"]]);
  });
});

describe("Safari counts once the sentence was shown", () => {
  it("Safari, popup first: nothing before the sentence, the waiting steps as soon as it is shown", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.analysed();
    expect(await t.host.status()).toBe("inform");
    await t.host.wake();
    expect(t.reports).toEqual([]);
    await t.host.noticeShown();
    expect(t.steps()).toEqual([["installed", "first_analysis"]]);
    expect(t.reports[0].browser).toBe("safari");
    expect(await t.host.status()).toBe("none");
  });

  it("Safari never sets an uninstall address", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.registerLeavingPage();
    await t.host.setSwitch(false);
    expect(t.urls).toEqual([]);
  });

  it("Turning counting off with steps waiting: neither is ever sent", async () => {
    const t = setup({ browser: "safari", outcome: "retry" });
    await t.host.installed("install");
    await t.host.noticeShown(); // offline: the attempt fails
    await t.host.setUp();
    expect(t.held()).toEqual(["installed", "set_up"]);
    await t.host.setSwitch(false);
    expect(t.state()?.waiting).toEqual([]);
    t.setOutcome("accepted");
    t.advance(2 * RETRY_AFTER_MS);
    await t.host.wake();
    expect(t.steps()).toEqual([["installed"]]); // the one failed attempt, before "off"
  });

  it("Turning it on again: nothing more is reported for this installation", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.noticeShown();
    expect(t.steps()).toEqual([["installed"]]);
    t.advance(2 * DAY_MS);
    await t.host.setSwitch(false);
    t.advance(DAY_MS);
    await t.host.setSwitch(true);
    t.advance(5 * DAY_MS);
    await t.host.used();
    await t.host.wake();
    expect(t.steps()).toEqual([["installed"]]);
    expect(await t.host.switchState()).toEqual({ shown: true, on: true });
  });

  it("a full reset forgets day 0 and the unsent steps, and leaves the switch as it was", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.forgetAll();
    expect(t.state()?.day0).toBeNull();
    expect(t.area.store[USAGE_COUNTS_OFF_KEY]).toBeUndefined();
    await t.host.noticeShown();
    await t.host.wake();
    expect(t.reports).toEqual([]);
  });
});

describe("Firefox follows its own data-collection consent", () => {
  it("with the consent, the leaving page is set and installed leaves at installation; the sentence only informs", async () => {
    const t = setup({ browser: "firefox", consent: granted, firefox: { request: true } });
    await t.host.installed("install");
    expect(t.urls.at(-1)).toBe("https://cymbra.app/lingua/au-revoir/?b=firefox&v=1.10");
    expect(t.reports.map((r) => [r.browser, r.installDay, r.steps])).toEqual([["firefox", DAY0, ["installed"]]]);
    expect(await t.host.status()).toBe("inform");
    await t.host.analysed();
    expect(t.steps()).toEqual([["installed"], ["first_analysis"]]);
  });

  it("Declined in Firefox's install prompt: no step is ever reported, and the switch shows off", async () => {
    const t = setup({ browser: "firefox", consent: { kind: "grant", supported: true, granted: false } });
    await t.host.installed("install");
    await t.host.analysed();
    t.advance(DAY_MS);
    await t.host.wake();
    expect(t.reports).toEqual([]);
    expect(t.state()?.day0 ?? null).toBeNull();
    expect(await t.host.switchState()).toEqual({ shown: true, on: false });
    expect(await t.host.status()).toBe("none");
  });

  it("Changed in the add-on manager: the device forgets, and the leaving page goes", async () => {
    const t = setup({ browser: "firefox", consent: granted, outcome: "retry" });
    await t.host.installed("install");
    expect(t.held()).toEqual(["installed"]);
    t.setConsent({ kind: "grant", supported: true, granted: false });
    await t.host.consentChanged();
    expect(t.state()?.waiting).toEqual([]);
    expect(t.urls.at(-1)).toBe("");
  });

  it("An older Firefox: nothing is reported and the switch is not shown", async () => {
    const t = setup({ browser: "firefox", consent: { kind: "grant", supported: false, granted: false } });
    await t.host.installed("install");
    await t.host.wake();
    expect(t.reports).toEqual([]);
    expect(await t.host.switchState()).toEqual({ shown: false, on: false });
  });

  it("the switch withdraws the consent, and asks it again", async () => {
    const t = setup({ browser: "firefox", consent: granted, firefox: { request: true } });
    expect(await t.host.setSwitch(false)).toEqual({ view: { shown: true, on: false } });
    expect(t.dataCollection!.remove).toHaveBeenCalledTimes(1);
    expect(await t.host.setSwitch(true)).toEqual({ view: { shown: true, on: true } });
    expect(t.dataCollection!.request).toHaveBeenCalledTimes(1);
  });

  it("from the in-page drawer, asking needs the toolbar: the reader is told so", async () => {
    const t = setup({
      browser: "firefox",
      consent: { kind: "grant", supported: true, granted: false },
      firefox: { request: false },
    });
    expect(await t.host.setSwitch(true)).toEqual({ view: { shown: true, on: false }, fromToolbar: true });
  });
});

describe("An installation from before the first counting release (an update from 1.9.0)", () => {
  /** Updated, then everything a reader does for two weeks with counting on. */
  async function updateAndUse(t: ReturnType<typeof setup>): Promise<void> {
    await t.host.installed("update");
    await t.host.registerLeavingPage();
    await t.host.noticeShown();
    await t.host.analysed();
    await t.host.setUp();
    t.advance(DAY_MS);
    await t.host.used();
    t.advanceTo(new Date(2026, 10, 2, 9));
    await t.host.wake();
  }

  it("Chrome: never asked, never counted, no leaving page — even after a yes from Réglages", async () => {
    const t = setup();
    await updateAndUse(t);
    expect(await t.host.status()).toBe("none");
    await t.host.setSwitch(true);
    await t.host.wake();
    expect(t.reports).toEqual([]);
    expect(t.urls.length).toBeGreaterThan(0);
    expect(t.urls.every((u) => u === "")).toBe(true);
  });

  it("Firefox with its consent granted: nothing counted, and no leaving page", async () => {
    const t = setup({ browser: "firefox", consent: granted });
    await updateAndUse(t);
    expect(await t.host.status()).toBe("none");
    expect(t.reports).toEqual([]);
    expect(t.urls.every((u) => u === "")).toBe(true);
  });

  it("Safari: no sentence, nothing counted", async () => {
    const t = setup({ browser: "safari" });
    await updateAndUse(t);
    expect(await t.host.status()).toBe("none");
    expect(t.reports).toEqual([]);
  });
});

describe("A step leaves at the first wake after it happens (owner, 2026-10-16)", () => {
  it("each step leaves at the wake that records it, every report filed under day 0", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.noticeShown();
    await t.host.setUp();
    t.advance(DAY_MS);
    await t.host.used();
    t.advance(7 * DAY_MS); // day 8
    await t.host.analysed();
    expect(t.steps()).toEqual([
      ["installed"],
      ["set_up"],
      ["returned_day_1"],
      ["first_analysis", "returned_days_7_13"],
    ]);
    expect(t.reports.every((r) => r.installDay === DAY0)).toBe(true);
    expect(t.held()).toEqual([]);
  });

  it("the midnight boundary, on the day Paris leaves summer time: 23:59 is still day 0, 00:00 is day 1", async () => {
    const sunday = new Date(2026, 9, 25, 0, 30); // a 25-hour day
    const t = setup({ browser: "safari", now: sunday });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advanceTo(new Date(2026, 9, 25, 23, 59));
    await t.host.used();
    expect(t.steps()).toEqual([["installed"]]);
    t.advanceTo(new Date(2026, 9, 26, 0, 0));
    await t.host.used();
    expect(t.steps()).toEqual([["installed"], ["returned_day_1"]]);
    const day0 = localDay(sunday);
    expect(t.reports.map((r) => r.installDay)).toEqual([day0, day0]);
    expect(day0).toBe(DAY0 + 4);
  });

  it("an installation the evening before the change: back the next morning is day 1", async () => {
    const t = setup({ browser: "safari", now: new Date(2026, 9, 24, 22, 0) });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advanceTo(new Date(2026, 9, 25, 8, 0));
    await t.host.analysed();
    expect(t.steps()).toEqual([["installed"], ["first_analysis", "returned_day_1"]]);
    expect(t.reports[1].installDay).toBe(localDay(new Date(2026, 9, 24, 22, 0)));
  });

  it("a clock set back before day 0 records nothing; the reports keep day 0 as it was", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advance(-3 * DAY_MS);
    await t.host.analysed();
    await t.host.setUp();
    expect(t.steps()).toEqual([["installed"]]);
    t.advanceTo(START);
    await t.host.analysed();
    expect(t.steps()).toEqual([["installed"], ["first_analysis"]]);
    expect(t.reports.every((r) => r.installDay === DAY0)).toBe(true);
  });

  it("a clock set back after a failure retries without waiting for it to catch up", async () => {
    const t = setup({ browser: "safari", outcome: "retry" });
    await t.host.installed("install");
    await t.host.noticeShown();
    expect(t.report).toHaveBeenCalledTimes(1);
    t.setOutcome("accepted");
    t.advance(-2 * 60 * 60 * 1000);
    await t.host.wake();
    expect(t.report).toHaveBeenCalledTimes(2);
    expect(t.held()).toEqual([]);
  });
});

describe("Reports ride the extension's own wake-ups", () => {
  it("Several steps at once: they leave in one report", async () => {
    const t = setup();
    await t.host.installed("install");
    await t.host.setUp();
    await t.host.analysed();
    await t.host.setSwitch(true);
    expect(t.steps()).toEqual([["installed", "set_up", "first_analysis"]]);
  });

  it("Offline for days: the waiting steps leave when the network is back, filed under day 0", async () => {
    const t = setup({ browser: "safari", outcome: "retry" });
    await t.host.installed("install");
    await t.host.noticeShown(); // offline: fails
    await t.host.setUp(); // within the hour: no attempt
    expect(t.report).toHaveBeenCalledTimes(1);
    t.setOutcome("accepted");
    t.advanceTo(new Date(2026, 9, 29, 10)); // the Thursday of the following week
    await t.host.wake();
    const last = t.reports.at(-1)!;
    expect(last.installDay).toBe(DAY0);
    expect(last.steps).toEqual(["installed", "set_up"]);
    expect(t.held()).toEqual([]);
  });

  it("The server is down: no further attempt before an hour has passed", async () => {
    const t = setup({ browser: "safari", outcome: "retry" });
    await t.host.installed("install");
    await t.host.noticeShown();
    expect(t.report).toHaveBeenCalledTimes(1);
    for (let i = 0; i < 5; i++) {
      t.advance(2 * 60 * 1000);
      await t.host.analysed();
    }
    expect(t.report).toHaveBeenCalledTimes(1);
    t.advance(RETRY_AFTER_MS);
    await t.host.wake();
    expect(t.report).toHaveBeenCalledTimes(2);
  });

  it("A report refused as malformed is never sent again", async () => {
    const t = setup({ browser: "safari", outcome: "invalid" });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advance(2 * RETRY_AFTER_MS);
    await t.host.wake();
    expect(t.report).toHaveBeenCalledTimes(1);
    expect(t.state()?.sent).toEqual(["installed"]);
  });

  it("two pages' messages at once never send the same steps twice", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advance(DAY_MS);
    await Promise.all([t.host.analysed(), t.host.analysed(), t.host.used(), t.host.wake()]);
    expect(t.steps()).toEqual([["installed"], ["first_analysis", "returned_day_1"]]);
  });

  it("a client that throws counts as a failure to retry", async () => {
    const t = setup({ browser: "safari" });
    t.report.mockRejectedValueOnce(new Error("boom"));
    await t.host.installed("install");
    await t.host.noticeShown();
    expect(t.state()?.lastFailure).toBe(START.getTime());
    expect(t.held()).toEqual(["installed"]);
  });

  /**
   * Delivery is at least once (design D5): the server counted the report, but the background
   * was stopped before the outcome was saved — the steps are still waiting, and go again.
   */
  it("a report accepted but not saved as sent is sent again after a restart", async () => {
    const area = fakeArea();
    const first = setup({ browser: "safari", area });
    await first.host.installed("install");
    area.failSetIf = (items) => ((items[USAGE_COUNTS_KEY] as UsageState | undefined)?.sent.length ?? 0) > 0;
    await expect(first.host.noticeShown()).rejects.toThrow("stopped");
    expect(first.steps()).toEqual([["installed"]]);
    // The background starts again on the same storage.
    const second = setup({ browser: "safari", area, now: START });
    await second.host.wake();
    expect(second.steps()).toEqual([["installed"]]);
    expect(second.state()?.sent).toEqual(["installed"]);
  });
});

describe("From day 60, nothing changes the installation any more (review of #958)", () => {
  const DAY_59 = new Date(START.getTime() + 59 * DAY_MS);
  const DAY_60 = new Date(START.getTime() + 60 * DAY_MS);

  it("day 60 is when the device forgets the installation, across the daylight-saving change", () => {
    expect(forgetsOn(DAY0)).toBe(DAY0 + 60);
    expect(localDay(DAY_60)).toBe(DAY0 + 60);
  });

  it("Chrome: a yes on day 59 still sends what was held", async () => {
    const t = setup();
    await t.host.installed("install");
    await t.host.setUp();
    t.advanceTo(DAY_59);
    await t.host.answer(true);
    expect(t.steps()).toEqual([["installed", "set_up"]]);
  });

  it("Chrome: a yes on day 60 sends nothing", async () => {
    const t = setup();
    await t.host.installed("install");
    await t.host.setUp();
    t.advanceTo(DAY_60);
    await t.host.answer(true);
    expect(t.reports).toEqual([]);
    expect(t.state()?.day0).toBeNull();
    expect(t.state()?.waiting).toEqual([]);
  });

  it("Safari: the sentence shown on day 60 sends nothing", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.analysed();
    t.advanceTo(DAY_60);
    expect(await t.host.status()).toBe("none");
    await t.host.noticeShown();
    expect(t.reports).toEqual([]);
  });

  it("a step that failed to leave by day 59 is forgotten at day 60, not sent", async () => {
    const t = setup({ browser: "safari", outcome: "retry" });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advanceTo(DAY_59);
    await t.host.used(); // back in days 30–59, still offline
    expect(t.held()).toEqual(["installed", "returned_days_30_59"]);
    t.setOutcome("accepted");
    t.advanceTo(DAY_60);
    await t.host.wake();
    expect(t.reports.filter((r) => r.steps.includes("returned_days_30_59"))).toHaveLength(1); // the failed try
    expect(t.state()?.day0).toBeNull();
    expect(t.state()?.waiting).toEqual([]);
    // The installation still counts as one made by a counting release.
    expect(t.state()?.countable).toBe(true);
  });

  it("the switch moved on day 60 sends nothing", async () => {
    const t = setup();
    await t.host.installed("install");
    t.advanceTo(DAY_60);
    await t.host.setSwitch(true);
    expect(t.reports).toEqual([]);
  });

  it("an installation past day 59 stops reading anything", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advanceTo(DAY_60);
    await t.host.used();
    expect(t.state()?.day0).toBeNull();
    const reads = vi.spyOn(t.area, "get");
    await t.host.analysed();
    await t.host.used();
    expect(reads).not.toHaveBeenCalled();
  });
});

describe("the surfaces' messages", () => {
  it("each message is answered", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    expect(await t.host.handle({ type: "usage:status" })).toEqual({ notice: "inform" });
    expect(await t.host.handle({ type: "usage:switch-state" })).toEqual({ shown: true, on: true });
    expect(await t.host.handle({ type: "usage:step", step: "set_up" })).toEqual({ ok: true });
    expect(await t.host.handle({ type: "usage:notice-shown" })).toEqual({ ok: true });
    expect(await t.host.handle({ type: "usage:consent", yes: true })).toEqual({ ok: true });
    expect(await t.host.handle({ type: "usage:switch", on: false })).toEqual({ view: { shown: true, on: false } });
    expect(await t.host.handle({ type: "usage:forget" })).toEqual({ ok: true });
  });

  it("the leaving page follows the interface language", async () => {
    for (const [language, url] of [
      ["fr", "https://cymbra.app/lingua/au-revoir/?b=firefox&v=1.10"],
      ["en", "https://cymbra.app/en/lingua/goodbye/?b=firefox&v=1.10"],
      ["es", "https://cymbra.app/es/lingua/adios/?b=firefox&v=1.10"],
    ] as const) {
      const t = setup({ browser: "firefox", consent: granted, language });
      await t.host.installed("install");
      await t.host.registerLeavingPage();
      expect(t.urls).toEqual([url, url]);
    }
  });

  it("a refused uninstall address is no failure", async () => {
    const t = setup({ browser: "firefox", consent: granted });
    const host = new UsageHost({
      area: t.area,
      mode: "on",
      browser: "firefox",
      version: "1.10.0",
      consent: async () => granted,
      client: () => ({ report: async () => "accepted" }),
      now: () => START,
      language: async () => "fr",
      setUninstallURL: async () => {
        throw new Error("unsupported");
      },
    });
    await host.installed("install");
    await expect(host.registerLeavingPage()).resolves.toBeUndefined();
  });
});
