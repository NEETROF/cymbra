import { afterEach, describe, expect, it, vi } from "vitest";
import { reviewOutlookJson } from "@/analyzer/engine.ts";
import { BADGE_COUNT_COLOUR, BADGE_NEUTRAL_COLOUR, BADGE_PERCENT_COLOUR } from "@/state/badge.ts";
import type { CardOp } from "@/analyzer/port.ts";
import { encodeAppOps } from "@/state/app-relay.ts";
import {
  BADGE_APPLIED_KEY,
  type BadgeAction,
  type BadgeHost,
  badgeHost,
  type BadgeHostDeps,
} from "@/state/badge-host.ts";
import type { NativeMessage } from "@/state/native-signin.ts";
import { reviewRelay } from "@/state/review-relay.ts";
import { ownerArea } from "@/state/store.ts";
import {
  OUTLOOK_KEY,
  OUTLOOK_VERSION,
  outlookFingerprint,
  type ReviewOutlook,
  UPCOMING_MAX,
} from "@/state/review-outlook.ts";
import { localDayStart } from "@/review/session.ts";
import { BADGE_COUNT_KEY, COLOURS_KEY, NEW_WORDS_PER_DAY_KEY, ROOT_KEY } from "@/state/storage.ts";
import { makeFakePort, memoryArea } from "./helpers.ts";
import { realGlue, realPort } from "./real-wasm.ts";

// The background's badge wiring (add-lingua-comeback-hints D2–D4), on fakes: the action API is a
// mock, the stores are in memory, the WASM function a mock returning an outlook. What is pinned:
// when the outlook is worked out (after a change of the data, coalesced; when missing, of another
// version or used up; never while the count is off), and that an event touches the action API only
// when what it shows changes.

const DAY_START = localDayStart(1_790_000_000);
const MORNING = DAY_START + 8 * 3_600;

function outlook(over: Partial<ReviewOutlook["languages"][number]> = {}, at = MORNING): ReviewOutlook {
  return {
    version: OUTLOOK_VERSION,
    computedAt: at,
    dayStart: localDayStart(at),
    languages: [{ language: "en", cards: 20, due: 12, upcoming: [MORNING + 3_600], fresh: 0, introduced: 0, ...over }],
  };
}

function setup(over: Partial<BadgeHostDeps> = {}, prefs: Record<string, unknown> = {}) {
  const action = {
    setBadgeText: vi.fn<(details: { text: string | null; tabId?: number }) => Promise<void>>(async () => {}),
    setBadgeBackgroundColor: vi.fn(async () => {}),
    setBadgeTextColor: vi.fn(async () => {}),
    setTitle: vi.fn(async () => {}),
  } satisfies BadgeAction;
  let now = MORNING;
  const deps: BadgeHostDeps = {
    action,
    prefs: memoryArea(prefs),
    applied: memoryArea(),
    store: memoryArea({ [ROOT_KEY]: { v: 2, backup: '{"backup":true}' } }),
    computeOutlook: vi.fn(async (_backup: string, at: number) => JSON.stringify(outlook({}, at))),
    now: () => now,
    supported: true,
    interfaceLanguage: async () => "fr",
    ...over,
  };
  return {
    host: badgeHost(deps),
    deps,
    action,
    prefs: deps.prefs as ReturnType<typeof memoryArea>,
    compute: deps.computeOutlook as ReturnType<typeof vi.fn>,
    at: (t: number) => {
      now = t;
    },
    /** The reader's data written anew (a backup the engine wrote). */
    write: (backup: string) => {
      (deps.store as ReturnType<typeof memoryArea>).raw[ROOT_KEY] = { v: 2, backup };
    },
    /** Every action call since the last `clear`. */
    calls: () => [
      ...action.setBadgeText.mock.calls,
      ...action.setBadgeBackgroundColor.mock.calls,
      ...action.setTitle.mock.calls,
    ],
    clear: () => {
      for (const fn of Object.values(action)) fn.mockClear();
    },
  };
}

describe("the badge's outlook", () => {
  it("A review finished in the browser: the data changed, the outlook is worked out and the count shown", async () => {
    const t = setup();
    await t.host.dataChanged();
    expect(t.compute).toHaveBeenCalledWith('{"backup":true}', MORNING, DAY_START);
    // Kept with the digest of what it read in the backup.
    expect(t.prefs.raw[OUTLOOK_KEY]).toEqual({ ...outlook(), inputs: await outlookFingerprint('{"backup":true}') });
    expect(t.action.setBadgeText).toHaveBeenCalledWith({ text: "12" });
    expect(t.action.setBadgeBackgroundColor).toHaveBeenCalledWith({ color: BADGE_COUNT_COLOUR });
    expect(t.action.setTitle).toHaveBeenCalledWith({ title: "Cymbra Lingua — 12 carte(s) à revoir" });

    // The last due card answered: the next outlook says nothing waits, and the badge empties.
    t.compute.mockImplementation(async () => JSON.stringify(outlook({ due: 0, upcoming: [] })));
    t.write('{"backup":"answered"}');
    await t.host.dataChanged();
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "" });
    expect(t.action.setTitle).toHaveBeenLastCalledWith({ title: "Cymbra Lingua" });
  });

  it("works one run at a time: changes during a run make one more, not one each", async () => {
    const t = setup();
    let release: () => void = () => {};
    t.compute.mockImplementationOnce(
      (_b: string, at: number) =>
        new Promise<string>((resolve) => (release = () => resolve(JSON.stringify(outlook({}, at))))),
    );
    const first = t.host.dataChanged();
    await vi.waitFor(() => expect(t.compute).toHaveBeenCalledTimes(1));
    t.write('{"backup":"second"}');
    const second = t.host.dataChanged();
    t.write('{"backup":"third"}');
    const third = t.host.dataChanged();
    release();
    await Promise.all([first, second, third]);
    expect(t.compute).toHaveBeenCalledTimes(2);
    expect(t.compute).toHaveBeenLastCalledWith('{"backup":"third"}', MORNING, DAY_START);
  });

  it("is worked out at an event when missing, of another version, or used up — and only then", async () => {
    const t = setup();
    await t.host.refresh();
    expect(t.compute).toHaveBeenCalledTimes(1); // missing

    await t.host.refresh();
    expect(t.compute).toHaveBeenCalledTimes(1); // there: read only

    t.prefs.raw[OUTLOOK_KEY] = { ...outlook(), version: OUTLOOK_VERSION + 1 };
    await t.host.refresh();
    expect(t.compute).toHaveBeenCalledTimes(2); // another version

    const times = Array.from({ length: UPCOMING_MAX }, (_, i) => MORNING + (i + 1) * 60);
    t.prefs.raw[OUTLOOK_KEY] = outlook({ upcoming: times });
    t.compute.mockImplementation(async () => JSON.stringify(outlook({ upcoming: times })));
    t.at(MORNING + 50 * 60);
    await t.host.refresh();
    expect(t.compute).toHaveBeenCalledTimes(2); // not used up yet
    t.at(MORNING + UPCOMING_MAX * 60);
    await t.host.refresh();
    expect(t.compute).toHaveBeenCalledTimes(3); // the 101st card may be due
  });

  it("A card comes due while the reader is away: nothing runs, the next event counts it", async () => {
    const t = setup();
    await t.host.dataChanged();
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "12" });
    t.at(MORNING + 2 * 3_600); // past the upcoming due time
    await t.host.refresh();
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "13" });
    expect(t.compute).toHaveBeenCalledTimes(1);
  });

  it("New words beyond the day's allowance wait, and a new day lets the allowance in", async () => {
    const t = setup({}, { [NEW_WORDS_PER_DAY_KEY]: 10 });
    t.compute.mockImplementation(async () =>
      JSON.stringify(outlook({ due: 3, upcoming: [], fresh: 40, introduced: 10 })),
    );
    await t.host.dataChanged();
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "3" });
    t.at(DAY_START + 86_400 + 9 * 3_600);
    await t.host.refresh();
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "13" });
  });

  it("an event changes nothing on the icon when the count is the same", async () => {
    const t = setup();
    await t.host.refresh();
    t.clear();
    await t.host.refresh();
    await t.host.refresh();
    expect(t.calls()).toEqual([]);
  });

  it("a backup the function cannot read keeps the outlook there was", async () => {
    const t = setup({}, { [OUTLOOK_KEY]: outlook() });
    t.compute.mockRejectedValue(new Error("malformed backup"));
    vi.spyOn(console, "warn").mockImplementation(() => {});
    await t.host.dataChanged();
    expect(t.prefs.raw[OUTLOOK_KEY]).toEqual(outlook());
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "12" });
  });

  it("no backup yet: nothing waits, without asking the function", async () => {
    const t = setup({ store: memoryArea() });
    await t.host.refresh();
    expect(t.compute).not.toHaveBeenCalled();
    expect(t.prefs.raw[OUTLOOK_KEY]).toMatchObject({ languages: [] });
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "" });
  });
});

describe("the count's switch (D7)", () => {
  it("Turned off: no outlook worked out, no count, the title the extension's name", async () => {
    const t = setup({}, { [BADGE_COUNT_KEY]: false });
    await t.host.dataChanged();
    expect(t.compute).not.toHaveBeenCalled();
    await t.host.refresh();
    expect(t.compute).not.toHaveBeenCalled();
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "" });
    expect(t.action.setTitle).toHaveBeenLastCalledWith({ title: "Cymbra Lingua" });
  });

  it("turned off drops the outlook; turned on again works it out at once", async () => {
    const t = setup();
    await t.host.dataChanged();
    t.prefs.raw[BADGE_COUNT_KEY] = false;
    await t.host.settingChanged();
    expect(t.prefs.raw[OUTLOOK_KEY]).toBeNull();
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "" });
    t.prefs.raw[BADGE_COUNT_KEY] = true;
    await t.host.settingChanged();
    expect(t.compute).toHaveBeenCalledTimes(2);
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "12" });
  });
});

describe("a tab's report", () => {
  it("an analysed page shows its percentage; nothing to count shows the global count", async () => {
    const t = setup();
    await t.host.tabReport(7, { pct: 94 });
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ tabId: 7, text: "94%" });
    expect(t.action.setBadgeBackgroundColor).toHaveBeenLastCalledWith({ tabId: 7, color: BADGE_PERCENT_COLOUR });
    await t.host.tabReport(7, { pct: null });
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ tabId: 7, text: null });
    expect(t.action.setBadgeBackgroundColor).toHaveBeenLastCalledWith({ tabId: 7, color: BADGE_COUNT_COLOUR });
    await t.host.tabReport(7, { pct: null, disabled: true });
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ tabId: 7, text: null });
  });

  it("E-ink colours: the percentage black", async () => {
    const t = setup({}, { [COLOURS_KEY]: { preset: "eink-mono" } });
    await t.host.tabReport(7, { pct: 94 });
    expect(t.action.setBadgeBackgroundColor).toHaveBeenLastCalledWith({ tabId: 7, color: "#000000" });
  });
});

describe("Safari 17 (D4)", () => {
  it("shows no count and keeps the badge as before", async () => {
    const t = setup({ supported: false });
    await t.host.dataChanged();
    await t.host.refresh();
    expect(t.compute).not.toHaveBeenCalled();
    expect(t.calls()).toEqual([]);
    await t.host.tabReport(3, { pct: null });
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ tabId: 3, text: "—" });
    expect(t.action.setBadgeBackgroundColor).toHaveBeenLastCalledWith({ tabId: 3, color: BADGE_NEUTRAL_COLOUR });
  });
});

describe("what was last applied", () => {
  it("is kept beside the worker, so a woken worker does not repaint the same count", async () => {
    const t = setup();
    await t.host.refresh();
    expect(await t.deps.applied!.get(BADGE_APPLIED_KEY)).toEqual({
      [BADGE_APPLIED_KEY]: { text: "12", colour: BADGE_COUNT_COLOUR, title: "Cymbra Lingua — 12 carte(s) à revoir" },
    });
    const woken = badgeHost(t.deps);
    t.clear();
    await woken.refresh();
    expect(t.calls()).toEqual([]);
  });

  it("without a session area, every event applies", async () => {
    const t = setup({ applied: null });
    await t.host.refresh();
    t.clear();
    await t.host.refresh();
    expect(t.action.setBadgeText).toHaveBeenCalledWith({ text: "12" });
  });
});

describe("only what the outlook reads works it out again (review finding: every exposure save recomputed it)", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  /** A deck of `cards` English words on the real engine, a few of them read on the page. */
  async function reader(cards: number) {
    const port = realPort();
    const english = port.for("en");
    for (let i = 0; i < cards; i++) {
      await english.addCard({
        lemma: `word${i}`,
        surface: `word${i}`,
        sentence: "",
        url: "",
        gloss: null,
        capturedAt: DAY_START - 86_400 + i,
      });
    }
    return { port, english };
  }

  /**
   * Sixty seconds of reading as the page does it: an exposure flush every two seconds
   * (`EXPOSURE_FLUSH_MS`), each saving the whole backup — then one word captured. Returns how many
   * times the outlook was worked out, `fingerprint` given or not.
   */
  async function sixtySecondsOfReading(fingerprint?: BadgeHostDeps["fingerprint"]) {
    const { port, english } = await reader(12);
    const compute = vi.fn((backup: string, now: number, dayStart: number) =>
      reviewOutlookJson(backup, now, dayStart, async () => realGlue()),
    );
    const t = setup({ computeOutlook: compute, ...(fingerprint ? { fingerprint } : {}) });
    t.write(await port.backup());
    await t.host.dataChanged();
    for (let second = 2; second <= 60; second += 2) {
      t.at(MORNING + second);
      await english.recordExposures(
        [`word${second % 12}`, `seen${second}`],
        "reading:example.com",
        (MORNING + second) * 1000,
      );
      await english.promoteByExposure(4, (MORNING + second) * 1000);
      t.write(await port.backup());
      await t.host.dataChanged();
    }
    const whileReading = compute.mock.calls.length;
    await english.addCard({
      lemma: "captured",
      surface: "captured",
      sentence: "",
      url: "",
      gloss: null,
      capturedAt: MORNING + 61,
    });
    t.write(await port.backup());
    await t.host.dataChanged();
    return { whileReading, afterCapture: compute.mock.calls.length, t };
  }

  it("Reading for a minute: thirty exposure saves work nothing out again; a capture does", async () => {
    // Before: every save of the backup worked the outlook out — 1 at the start, 30 while reading.
    const before = await sixtySecondsOfReading(async () => null);
    expect(before.whileReading).toBe(31);
    // After: the deck and the studied languages did not move, so nothing is worked out.
    const after = await sixtySecondsOfReading();
    expect(after.whileReading).toBe(1);
    expect(after.afterCapture).toBe(2);
    // 12 new words, ten let in today; the captured one is the thirteenth: the count is still 10.
    expect(after.t.prefs.raw[OUTLOOK_KEY]).toMatchObject({ languages: [{ language: "en", cards: 13, fresh: 13 }] });
    expect(after.t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "10" });
  });

  it("A change of the studied languages works it out again", async () => {
    const { port } = await reader(3);
    const t = setup({ computeOutlook: (b, n, d) => reviewOutlookJson(b, n, d, async () => realGlue()) });
    t.write(await port.backup());
    await t.host.dataChanged();
    const compute = vi.spyOn(t.deps, "computeOutlook");
    const rebuilt = badgeHost(t.deps);
    await port.setStudiedLanguages(["en", "es"]);
    t.write(await port.backup());
    await rebuilt.dataChanged();
    expect(compute).toHaveBeenCalledTimes(1);
    expect(t.prefs.raw[OUTLOOK_KEY]).toMatchObject({ languages: [{ language: "en" }, { language: "es" }] });
  });

  it("the same deck, its outlook stale or used up, is still worked out again", async () => {
    const t = setup();
    const inputs = await outlookFingerprint('{"backup":true}');
    // Worked out later than now: the clock went back.
    t.prefs.raw[OUTLOOK_KEY] = { ...outlook({ due: 30 }, MORNING + 5 * 3_600), inputs };
    await t.host.dataChanged();
    expect(t.compute).toHaveBeenCalledTimes(1);
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "12" });
  });
});

describe("the clock going back (review finding: the count went too high)", () => {
  it("an event finding an outlook worked out later than now works a new one out", async () => {
    const t = setup({}, { [OUTLOOK_KEY]: outlook({ due: 30 }, MORNING + 5 * 3_600) });
    await t.host.refresh();
    expect(t.compute).toHaveBeenCalledWith('{"backup":true}', MORNING, DAY_START);
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "12" });
  });

  it("an outlook of a later day is stale too; a backup that cannot be read then shows nothing, not too much", async () => {
    const tomorrow = outlook({ due: 30 }, MORNING + 86_400);
    const t = setup({}, { [OUTLOOK_KEY]: tomorrow });
    t.compute.mockRejectedValue(new Error("malformed backup"));
    vi.spyOn(console, "warn").mockImplementation(() => {});
    await t.host.refresh();
    expect(t.compute).toHaveBeenCalledTimes(1);
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "" });
  });
});

describe("the switch and a run under way (review finding: a stale count after turning it back on)", () => {
  it("a run that finishes after the switch went off writes nothing; turned on, the count is worked out anew", async () => {
    const t = setup();
    t.compute.mockImplementation(async (backup: string, at: number) =>
      JSON.stringify(outlook({ due: backup.includes("later") ? 4 : 12 }, at)),
    );
    let release: () => void = () => {};
    t.compute.mockImplementationOnce(
      (_b: string, at: number) =>
        new Promise<string>((resolve) => (release = () => resolve(JSON.stringify(outlook({}, at))))),
    );
    const running = t.host.dataChanged();
    await vi.waitFor(() => expect(t.compute).toHaveBeenCalledTimes(1));
    t.prefs.raw[BADGE_COUNT_KEY] = false;
    await t.host.settingChanged();
    expect(t.prefs.raw[OUTLOOK_KEY]).toBeNull();
    release();
    await running;
    expect(t.prefs.raw[OUTLOOK_KEY]).toBeNull();

    // Off, the data moves on unseen; on again, the count is today's, not the run's.
    t.write('{"backup":"later"}');
    await t.host.dataChanged();
    t.prefs.raw[BADGE_COUNT_KEY] = true;
    await t.host.settingChanged();
    expect(t.compute).toHaveBeenLastCalledWith('{"backup":"later"}', MORNING, DAY_START);
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "4" });
  });

  it("off and on again while a run is under way: the run's outlook is dropped, a new one kept", async () => {
    const t = setup();
    let release: () => void = () => {};
    t.compute.mockImplementationOnce(
      (_b: string, at: number) =>
        new Promise<string>((resolve) => (release = () => resolve(JSON.stringify(outlook({ due: 99 }, at))))),
    );
    const running = t.host.dataChanged();
    await vi.waitFor(() => expect(t.compute).toHaveBeenCalledTimes(1));
    t.prefs.raw[BADGE_COUNT_KEY] = false;
    await t.host.settingChanged();
    t.prefs.raw[BADGE_COUNT_KEY] = true;
    const on = t.host.settingChanged();
    release();
    await Promise.all([running, on]);
    expect(t.compute).toHaveBeenCalledTimes(2);
    expect(t.prefs.raw[OUTLOOK_KEY]).toMatchObject({ languages: [{ due: 12 }] });
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "12" });
  });
});

describe("one touch of the icon at a time (review finding: the record and the icon could part)", () => {
  /** setBadgeText held on its first call, until `release`. */
  function holdFirst(t: ReturnType<typeof setup>) {
    let release: () => void = () => {};
    t.action.setBadgeText.mockImplementationOnce(() => new Promise<void>((resolve) => (release = resolve)));
    return () => release();
  }

  it("two events, the first slow: the second's figure is shown last, and is what is remembered", async () => {
    const t = setup({}, { [OUTLOOK_KEY]: outlook() });
    const release = holdFirst(t);
    const first = t.host.refresh();
    await vi.waitFor(() => expect(t.action.setBadgeText).toHaveBeenCalledTimes(1));
    t.prefs.raw[OUTLOOK_KEY] = outlook({ due: 3 });
    const second = t.host.refresh();
    // Everything the second event can do before the first's touch ends.
    await new Promise((resolve) => setTimeout(resolve, 0));
    release();
    await Promise.all([first, second]);
    expect(t.action.setBadgeText.mock.calls.map(([d]) => d.text)).toEqual(["12", "3"]);
    expect(t.action.setTitle).toHaveBeenLastCalledWith({ title: "Cymbra Lingua — 3 carte(s) à revoir" });
    const applied = (await t.deps.applied!.get(BADGE_APPLIED_KEY))[BADGE_APPLIED_KEY];
    expect(applied).toMatchObject({ text: "3" });
  });

  it("a tab's navigation start then its page's report: the percentage, in its colour, wins", async () => {
    const t = setup();
    const release = holdFirst(t);
    const cleared = t.host.tabUpdated(7, "loading");
    await vi.waitFor(() => expect(t.action.setBadgeText).toHaveBeenCalledTimes(1));
    const reported = t.host.tabReport(7, { pct: 94 });
    await new Promise((resolve) => setTimeout(resolve, 0));
    release();
    await Promise.all([cleared, reported]);
    expect(t.action.setBadgeText.mock.calls.map(([d]) => d.text)).toEqual([null, "94%"]);
    expect(t.action.setBadgeBackgroundColor).toHaveBeenLastCalledWith({ tabId: 7, color: BADGE_PERCENT_COLOUR });
  });

  it("a failed touch does not stop the ones after it", async () => {
    const t = setup();
    t.action.setBadgeText.mockRejectedValueOnce(new Error("No tab with id: 7"));
    await expect(t.host.tabReport(7, { pct: 94 })).rejects.toThrow("No tab");
    await t.host.tabReport(8, { pct: 50 });
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ tabId: 8, text: "50%" });
  });
});

describe("a tab's navigation (review finding: Safari 18 may keep a percentage past its page)", () => {
  it("at its start, the tab's own percentage goes, so the tab shows the count", async () => {
    const t = setup();
    await t.host.tabReport(7, { pct: 94 });
    await t.host.tabUpdated(7, "loading");
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ tabId: 7, text: null });
    expect(t.action.setBadgeBackgroundColor).toHaveBeenLastCalledWith({ tabId: 7, color: BADGE_COUNT_COLOUR });
  });

  it("at its end, the page is asked to say its figure again — a page that stayed puts its percentage back", async () => {
    const askReport = vi.fn(async () => {});
    const t = setup({ askReport });
    await t.host.tabUpdated(7, "complete");
    expect(askReport).toHaveBeenCalledWith(7);
    expect(t.action.setBadgeText).not.toHaveBeenCalled();
  });

  it("a page without the reader does not answer: nothing breaks", async () => {
    const t = setup({ askReport: async () => Promise.reject(new Error("Receiving end does not exist")) });
    await expect(t.host.tabUpdated(7, "complete")).resolves.toBeUndefined();
    await t.host.tabUpdated(7, undefined);
    expect(t.calls()).toEqual([]);
  });

  it("Safari 17 keeps one badge for every tab: a load clears nothing there", async () => {
    const askReport = vi.fn(async () => {});
    const t = setup({ supported: false, askReport });
    await t.host.tabUpdated(7, "loading");
    await t.host.tabUpdated(7, "complete");
    expect(t.calls()).toEqual([]);
    expect(askReport).not.toHaveBeenCalled();
  });
});

describe("a review made in the Cymbra Lingua app (add-lingua-app-deck-sync)", () => {
  const CARD: CardOp = {
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

  it("handed back through the App Group, it is written through the owner's area, and the count follows", async () => {
    const raw = memoryArea({ [ROOT_KEY]: { v: 2, backup: '{"backup":true}' } });
    const runs: Promise<void>[] = [];
    let host: BadgeHost | null = null;
    // As background.ts wires it: every write of the backup by its owner tells the badge.
    const store = ownerArea(raw, (keys) => {
      if (keys.includes(ROOT_KEY) && host) runs.push(host.dataChanged());
    });
    const t = setup({ store });
    host = t.host;
    await t.host.dataChanged();
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "12" });

    // The app answered the cards that were due; the relay lays its changes on the sync's engine and saves.
    const { port } = makeFakePort();
    port.restore = async () => {};
    port.backup = async () => '{"backup":"from the app"}';
    port.applyCardOps = async (ops) => ops.length;
    port.applyStatusChanges = async () => 0;
    let waiting: { ops: string; writtenAt: number } | null = {
      ops: encodeAppOps({ since: 0, cardOps: [CARD], statusOps: [] }),
      writtenAt: 1,
    };
    const send = async (message: NativeMessage): Promise<unknown> => {
      if (message.type === "review.readOps") return waiting ?? {};
      if (message.type === "review.clearOps") waiting = null;
      return {};
    };
    const relay = reviewRelay({ send, store, prefs: memoryArea(), port, exclusive: (task) => task(), now: () => 0 });
    t.compute.mockImplementation(async () => JSON.stringify(outlook({ due: 0, upcoming: [] })));

    expect(await relay.collect()).toBe(1);
    await Promise.all(runs);
    expect(t.compute).toHaveBeenLastCalledWith('{"backup":"from the app"}', MORNING, DAY_START);
    expect(t.action.setBadgeText).toHaveBeenLastCalledWith({ text: "" });
  });
});
