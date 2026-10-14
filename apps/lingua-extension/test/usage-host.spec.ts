import { describe, expect, it, vi } from "vitest";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";
import type { UsageReport } from "@/usage/client.ts";
import {
  type BrowserFamily,
  type Consent,
  type CountsMode,
  localDay,
  mondayOf,
  type ReportOutcome,
  RETRY_AFTER_MS,
  USAGE_COUNTS_KEY,
  USAGE_COUNTS_OFF_KEY,
  type UsageState,
} from "@/usage/counts.ts";
import { UsageHost } from "@/usage/host.ts";

// The background's side of the counts (add-lingua-usage-counts D1, D5, D6, D7, D12): each trigger
// records a step and sends what waits, as each browser's consent allows — nothing on Chromium
// before "yes", nothing in Safari before the sentence, on Firefox only with its consent.

function fakeArea(store: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in store) out[k] = store[k];
      return out;
    },
    async set(items) {
      Object.assign(store, structuredClone(items));
    },
  };
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
}

function setup(opts: Setup = {}) {
  const area = fakeArea();
  let consent: Consent = opts.consent ?? { kind: "switch", off: undefined };
  let now = START;
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
    setConsent: (c: Consent) => (consent = c),
    setOutcome: (o: ReportOutcome) => (outcome = o),
    advance: (ms: number) => (now = new Date(now.getTime() + ms)),
  };
}

const week = mondayOf(localDay(START));

describe("Chromium asks before anything leaves", () => {
  it("Chrome, agreed on the welcome tab: installed and set up leave in one report after the press, none before", async () => {
    const t = setup();
    await t.host.installed("install");
    await t.host.setUp();
    expect(t.reports).toEqual([]);
    expect(await t.host.status()).toBe("ask");
    await t.host.answer(true);
    expect(t.reports).toEqual([
      { installWeek: week, browser: "chromium", version: "1.10", steps: ["installed", "set_up"] },
    ]);
    expect(await t.host.status()).toBe("none");
    expect(t.state()?.waiting).toEqual([]);
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
    expect(t.state()?.waiting).toEqual(["installed", "first_analysis", "returned_day_1"]);
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
    expect(t.reports).toEqual([]);
    expect(await t.host.switchState()).toEqual({ shown: true, on: false });
    expect(t.urls.at(-1)).toBe("");
  });

  it("ticking the switch is a yes", async () => {
    const t = setup();
    await t.host.installed("install");
    const reply = await t.host.setSwitch(true);
    expect(reply).toEqual({ view: { shown: true, on: true } });
    expect(t.reports).toHaveLength(1);
  });
});

describe("Safari counts once the sentence was shown", () => {
  it("Safari, popup first: nothing before the sentence, the waiting steps right after it", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.analysed();
    expect(t.reports).toEqual([]);
    expect(await t.host.status()).toBe("inform");
    await t.host.noticeShown();
    expect(t.reports.map((r) => r.steps)).toEqual([["installed", "first_analysis"]]);
    expect(t.reports[0].browser).toBe("safari");
    expect(await t.host.status()).toBe("none");
  });

  it("Safari never sets an uninstall address", async () => {
    const t = setup({ browser: "safari" });
    await t.host.registerLeavingPage();
    await t.host.setSwitch(false);
    expect(t.urls).toEqual([]);
  });

  it("Turning counting off with steps waiting: neither is ever sent", async () => {
    const t = setup({ browser: "safari", outcome: "retry" });
    await t.host.installed("install");
    await t.host.noticeShown();
    await t.host.setUp();
    expect(t.state()?.waiting).toEqual(["installed", "set_up"]);
    await t.host.setSwitch(false);
    expect(t.state()?.waiting).toEqual([]);
    t.setOutcome("accepted");
    t.advance(2 * RETRY_AFTER_MS);
    await t.host.wake();
    expect(t.reports).toHaveLength(1); // the one failed attempt, before "off"
  });

  it("Turning it on again: nothing more is reported for this installation", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advance(2 * DAY_MS);
    await t.host.setSwitch(false);
    t.advance(DAY_MS);
    await t.host.setSwitch(true);
    t.advance(5 * DAY_MS);
    await t.host.used();
    expect(t.reports.map((r) => r.steps)).toEqual([["installed"]]);
    expect(await t.host.switchState()).toEqual({ shown: true, on: true });
  });

  it("a full reset forgets day 0 and the unsent steps, and leaves the switch as it was", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.forgetAll();
    expect(t.state()?.day0).toBeNull();
    expect(t.area.store[USAGE_COUNTS_OFF_KEY]).toBeUndefined();
    await t.host.noticeShown();
    expect(t.reports).toEqual([]);
  });
});

describe("Firefox follows its own data-collection consent", () => {
  const granted: Consent = { kind: "grant", supported: true, granted: true };

  it("with the consent, steps leave at once; the sentence only informs", async () => {
    const t = setup({ browser: "firefox", consent: granted, firefox: { request: true } });
    await t.host.installed("install");
    expect(t.reports.map((r) => [r.browser, r.steps])).toEqual([["firefox", ["installed"]]]);
    expect(await t.host.status()).toBe("inform");
    await t.host.registerLeavingPage();
    expect(t.urls.at(-1)).toBe("https://cymbra.app/lingua/au-revoir/?b=firefox&v=1.10");
  });

  it("Declined in Firefox's install prompt: no step is ever reported, and the switch shows off", async () => {
    const t = setup({ browser: "firefox", consent: { kind: "grant", supported: true, granted: false } });
    await t.host.installed("install");
    await t.host.analysed();
    expect(t.reports).toEqual([]);
    expect(t.state()?.day0 ?? null).toBeNull();
    expect(await t.host.switchState()).toEqual({ shown: true, on: false });
    expect(await t.host.status()).toBe("none");
  });

  it("Changed in the add-on manager: the device forgets, and the leaving page goes", async () => {
    const t = setup({ browser: "firefox", consent: granted, outcome: "retry" });
    await t.host.installed("install");
    t.setConsent({ kind: "grant", supported: true, granted: false });
    await t.host.consentChanged();
    expect(t.state()?.waiting).toEqual([]);
    expect(t.urls.at(-1)).toBe("");
  });

  it("An older Firefox: nothing is reported and the switch is not shown", async () => {
    const t = setup({ browser: "firefox", consent: { kind: "grant", supported: false, granted: false } });
    await t.host.installed("install");
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

describe("Reports ride the extension's own wake-ups", () => {
  it("An update, not an installation: it reports no step, ever", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("update");
    await t.host.noticeShown();
    await t.host.analysed();
    await t.host.setUp();
    t.advance(DAY_MS);
    await t.host.used();
    expect(t.reports).toEqual([]);
  });

  it("Several steps at once: they leave in one report", async () => {
    const t = setup();
    await t.host.installed("install");
    await t.host.setUp();
    await t.host.analysed();
    await t.host.setSwitch(true);
    expect(t.reports.map((r) => r.steps)).toEqual([["installed", "set_up", "first_analysis"]]);
  });

  it("Offline for three days: the waiting steps leave on day 3, filed under the week of day 0", async () => {
    const t = setup({ browser: "safari", outcome: "retry" });
    await t.host.installed("install");
    await t.host.noticeShown();
    await t.host.setUp();
    t.setOutcome("accepted");
    t.advance(3 * DAY_MS);
    await t.host.analysed();
    const last = t.reports.at(-1)!;
    expect(last.installWeek).toBe(week);
    expect(last.steps).toEqual(["installed", "set_up", "first_analysis"]);
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
    await Promise.all([t.host.analysed(), t.host.analysed(), t.host.used()]);
    expect(t.reports.map((r) => r.steps)).toEqual([["installed"], ["first_analysis", "returned_day_1"]]);
  });

  it("a client that throws counts as a failure to retry", async () => {
    const t = setup({ browser: "safari" });
    t.report.mockRejectedValueOnce(new Error("boom"));
    await t.host.installed("install");
    await t.host.noticeShown();
    expect(t.state()?.lastFailure).toBe(START.getTime());
  });

  it("an installation past day 59 stops reading anything: it can never count again", async () => {
    const t = setup({ browser: "safari" });
    await t.host.installed("install");
    await t.host.noticeShown();
    t.advance(60 * DAY_MS);
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
      const t = setup({ browser: "firefox", consent: { kind: "grant", supported: true, granted: true }, language });
      await t.host.registerLeavingPage();
      expect(t.urls).toEqual([url]);
    }
  });

  it("a refused uninstall address is no failure", async () => {
    const t = setup({ browser: "firefox", consent: { kind: "grant", supported: true, granted: true } });
    const host = new UsageHost({
      area: t.area,
      mode: "on",
      browser: "firefox",
      version: "1.10.0",
      consent: async () => ({ kind: "grant", supported: true, granted: true }),
      client: () => ({ report: async () => "accepted" }),
      now: () => START,
      language: async () => "fr",
      setUninstallURL: async () => {
        throw new Error("unsupported");
      },
    });
    await expect(host.registerLeavingPage()).resolves.toBeUndefined();
  });
});
