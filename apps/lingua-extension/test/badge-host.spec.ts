import { describe, expect, it, vi } from "vitest";
import { BADGE_COUNT_COLOUR, BADGE_NEUTRAL_COLOUR, BADGE_PERCENT_COLOUR } from "@/state/badge.ts";
import { BADGE_APPLIED_KEY, type BadgeAction, badgeHost, type BadgeHostDeps } from "@/state/badge-host.ts";
import { OUTLOOK_KEY, OUTLOOK_VERSION, type ReviewOutlook, UPCOMING_MAX } from "@/state/review-outlook.ts";
import { localDayStart } from "@/review/session.ts";
import { BADGE_COUNT_KEY, COLOURS_KEY, NEW_WORDS_PER_DAY_KEY, ROOT_KEY } from "@/state/storage.ts";
import { memoryArea } from "./helpers.ts";

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
    setBadgeText: vi.fn(async () => {}),
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
    expect(t.prefs.raw[OUTLOOK_KEY]).toEqual(outlook());
    expect(t.action.setBadgeText).toHaveBeenCalledWith({ text: "12" });
    expect(t.action.setBadgeBackgroundColor).toHaveBeenCalledWith({ color: BADGE_COUNT_COLOUR });
    expect(t.action.setTitle).toHaveBeenCalledWith({ title: "Cymbra Lingua — 12 carte(s) à revoir" });

    // The last due card answered: the next outlook says nothing waits, and the badge empties.
    t.compute.mockImplementation(async () => JSON.stringify(outlook({ due: 0, upcoming: [] })));
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
    const second = t.host.dataChanged();
    const third = t.host.dataChanged();
    release();
    await Promise.all([first, second, third]);
    expect(t.compute).toHaveBeenCalledTimes(2);
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
