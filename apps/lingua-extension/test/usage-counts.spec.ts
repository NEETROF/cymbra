// A time zone with a daylight-saving change, so the week boundary is tested across one: Paris
// leaves summer time on Sunday 25 October 2026. Set before any date is made (each test file runs
// in its own process); the first test checks it took effect.
process.env.TZ = "Europe/Paris";

import { describe, expect, it } from "vitest";
import {
  afterReport,
  type Consent,
  countingOn,
  due,
  EMPTY_STATE,
  expire,
  forget,
  forgetsOn,
  install,
  LAST_DAY,
  LEAVING_PAGES,
  leavesFrom,
  leavingPageOn,
  leavingUrl,
  localDay,
  mondayOf,
  noticeKind,
  parseState,
  ready,
  record,
  recordAnalysis,
  recording,
  recordUse,
  returnOn,
  RETRY_AFTER_MS,
  sending,
  shortVersion,
  switchView,
  type UsageState,
} from "@/usage/counts.ts";

// The anonymous usage counts' state (add-lingua-usage-counts, spec `lingua-usage-counts`, "Six
// anonymous steps, each reported once per installation"): day 0 is the local day of the
// installation, steps are filed under its week, each is recorded once, and each leaves the device
// only from the Monday after the week it happened in.

/** A local date at a given hour (the device's time zone). */
const at = (y: number, m: number, d: number, h = 12, min = 0): Date => new Date(y, m - 1, d, h, min);
/** Wednesday 21 October 2026, local. */
const WEDNESDAY = localDay(at(2026, 10, 21));
const MONDAY = localDay(at(2026, 10, 19));
const NEXT_MONDAY = MONDAY + 7;

const steps = (s: UsageState): string[] => s.waiting.map((w) => w.step);

/** An installation on `day0`, its installed step already sent. */
function installedOn(day0: number): UsageState {
  return afterReport(install(EMPTY_STATE, day0), ["installed"], "accepted", 0);
}

describe("Six anonymous steps, each reported once per installation", () => {
  it("days are counted on the device's calendar, and filed under the week's Monday", () => {
    // 2026-10-19 is a Monday.
    expect(new Date(Date.UTC(2026, 9, 19)).getUTCDay()).toBe(1);
    expect(MONDAY).toBe(Date.UTC(2026, 9, 19) / 86_400_000);
    for (let d = 0; d < 7; d++) expect(mondayOf(MONDAY + d)).toBe(MONDAY);
    expect(mondayOf(MONDAY - 1)).toBe(MONDAY - 7);
    // The epoch was a Thursday; its Monday is before it.
    expect(mondayOf(0)).toBe(-3);
    expect(mondayOf(-3)).toBe(-3);
  });

  it("A reader who sets up and reads on day 0: installed, set up and first page analysed, under Monday 19 October", () => {
    let s = install(EMPTY_STATE, WEDNESDAY);
    expect(s.week).toBe(MONDAY);
    expect(s.countable).toBe(true);
    s = record(s, "set_up", WEDNESDAY);
    s = recordAnalysis(s, WEDNESDAY);
    expect(steps(s)).toEqual(["installed", "set_up", "first_analysis"]);
    // Each leaves from the Monday after the week it happened in.
    expect(s.waiting.map((w) => w.from)).toEqual([NEXT_MONDAY, NEXT_MONDAY, NEXT_MONDAY]);
    // Each once: again on day 0 adds nothing.
    s = record(recordAnalysis(s, WEDNESDAY), "set_up", WEDNESDAY);
    expect(steps(s)).toEqual(["installed", "set_up", "first_analysis"]);
  });

  it("An installation on a Sunday is filed under the Monday before", () => {
    const sunday = localDay(at(2026, 10, 25));
    expect(install(EMPTY_STATE, sunday).week).toBe(MONDAY);
  });

  it("A weekend reader: back in days 7–13 once, never back on day 1", () => {
    let s = installedOn(MONDAY);
    s = recordUse(s, MONDAY + 6);
    expect(s.waiting).toEqual([]);
    s = recordUse(recordUse(s, MONDAY + 8), MONDAY + 9);
    expect(steps(s)).toEqual(["returned_days_7_13"]);
  });

  it("each window: day 1, days 7 to 13, days 30 to 59, and nothing else", () => {
    expect([0, 1, 2, 6, 7, 13, 14, 29, 30, 59, 60].map(returnOn)).toEqual([
      null,
      "returned_day_1",
      null,
      null,
      "returned_days_7_13",
      "returned_days_7_13",
      null,
      null,
      "returned_days_30_59",
      "returned_days_30_59",
      null,
    ]);
    let s = installedOn(MONDAY);
    for (const d of [1, 7, 30]) s = recordUse(s, MONDAY + d);
    expect(steps(s)).toEqual(["returned_day_1", "returned_days_7_13", "returned_days_30_59"]);
  });

  it("A late return: nothing is reported, and the device holds no day 0 any more", () => {
    const s = recordUse(installedOn(MONDAY), MONDAY + 200);
    expect(s.waiting).toEqual([]);
    expect(s.day0).toBeNull();
    expect(s.week).toBeNull();
    // Nothing is recorded afterwards either.
    expect(recordAnalysis(s, MONDAY + 201)).toEqual(s);
  });

  it("nothing is recorded after day 59; the steps held then leave the week after, and then everything goes", () => {
    let s = recordUse(installedOn(MONDAY), MONDAY + LAST_DAY);
    expect(steps(s)).toEqual(["returned_days_30_59"]);
    // Day 60: no new step, but the held one stays until it may leave.
    s = record(s, "set_up", MONDAY + 60);
    expect(steps(s)).toEqual(["returned_days_30_59"]);
    const from = leavesFrom(MONDAY + LAST_DAY);
    expect(ready(s, from - 1)).toEqual([]);
    expect(ready(expire(s, from), from)).toEqual(["returned_days_30_59"]);
    // A whole week to leave, then the device forgets day 0 and the step.
    expect(forgetsOn(MONDAY)).toBe(from + 7);
    expect(expire(s, from + 6)).toBe(s);
    const gone = expire(s, from + 7);
    expect(gone.waiting).toEqual([]);
    expect(gone.day0).toBeNull();
    expect(gone.countable).toBe(true);
  });

  it("whatever the weekday of day 0, every step gets a week to leave, and nothing outlives day 72", () => {
    for (let d = 0; d < 7; d++) {
      const day0 = MONDAY + d;
      const lastLeaves = leavesFrom(day0 + LAST_DAY);
      expect(forgetsOn(day0)).toBe(lastLeaves + 7);
      expect(forgetsOn(day0) - day0).toBeGreaterThanOrEqual(67);
      expect(forgetsOn(day0) - day0).toBeLessThanOrEqual(73);
    }
  });

  it("held steps are forgotten once the last week is over (Chrome, never answered)", () => {
    const held = record(install(EMPTY_STATE, MONDAY), "set_up", MONDAY);
    expect(expire(held, forgetsOn(MONDAY) - 1)).toBe(held);
    const gone = expire(held, forgetsOn(MONDAY));
    expect(gone.waiting).toEqual([]);
    expect(gone.day0).toBeNull();
  });

  it("An update, not an installation: with no day 0, nothing is ever recorded", () => {
    let s = EMPTY_STATE;
    s = recordAnalysis(s, WEDNESDAY);
    s = record(s, "set_up", WEDNESDAY + 1);
    s = recordUse(s, WEDNESDAY + 8);
    expect(s).toEqual(EMPTY_STATE);
    expect(s.countable).toBe(false);
  });

  it("A clock change around midnight: the local day decides, and a clock set back before day 0 records nothing", () => {
    const day0 = localDay(at(2026, 10, 19, 23, 59));
    expect(localDay(at(2026, 10, 20, 0, 1))).toBe(day0 + 1);
    let s = installedOn(day0);
    // Back on the next local day, a minute after midnight: day 1.
    s = recordUse(s, localDay(at(2026, 10, 20, 0, 1)));
    expect(steps(s)).toEqual(["returned_day_1"]);
    // The clock moved back to the day before installing: nothing.
    const back = recordAnalysis(installedOn(day0), day0 - 1);
    expect(back.waiting).toEqual([]);
  });

  it("A page in no studied language is never a use: only an analysis or a surface records (the host decides)", () => {
    // A use on day 1 records the return; nothing else does.
    expect(steps(recordUse(installedOn(MONDAY), MONDAY + 1))).toEqual(["returned_day_1"]);
  });

  it("a step already reported is never recorded again", () => {
    let s = recordUse(installedOn(MONDAY), MONDAY + 1);
    s = afterReport(s, ["returned_day_1"], "accepted", 1);
    expect(recordUse(s, MONDAY + 1).waiting).toEqual([]);
    expect(s.sent).toEqual(["installed", "returned_day_1"]);
  });
});

describe("A step leaves the week after the one it happened in", () => {
  it("the device's time zone has a daylight-saving change in this week", () => {
    // Sunday 25 October 2026 is the day Paris leaves summer time: the hours around it differ.
    expect(at(2026, 10, 24).getTimezoneOffset()).not.toBe(at(2026, 10, 26).getTimezoneOffset());
  });

  it("Sunday 23:59 is still the step's week; Monday 00:00 is the first moment it may leave — across the DST change", () => {
    const sundayNight = localDay(at(2026, 10, 25, 23, 59));
    const mondayMidnight = localDay(at(2026, 10, 26, 0, 0));
    expect(mondayMidnight).toBe(sundayNight + 1);
    expect(mondayMidnight).toBe(NEXT_MONDAY);
    // Installed on the Wednesday of the DST week, set up on its Sunday night.
    let s = install(EMPTY_STATE, WEDNESDAY);
    s = record(s, "set_up", sundayNight);
    expect(ready(s, sundayNight)).toEqual([]);
    expect(due(s, sundayNight, 0)).toBe(false);
    expect(ready(s, mondayMidnight)).toEqual(["installed", "set_up"]);
    expect(due(s, mondayMidnight, 0)).toBe(true);
  });

  it("a step on a Monday waits for the next one, never leaving in its own week", () => {
    let s = installedOn(MONDAY);
    // Day 1 is a Tuesday: it leaves from the Monday after, not before.
    s = recordUse(s, MONDAY + 1);
    for (let d = 1; d < 7; d++) expect(ready(s, MONDAY + d)).toEqual([]);
    expect(ready(s, NEXT_MONDAY)).toEqual(["returned_day_1"]);
    // A return on day 8 (a Tuesday) waits for the end of its own week, day 13.
    s = recordUse(afterReport(s, ["returned_day_1"], "accepted", 0), MONDAY + 8);
    expect(ready(s, MONDAY + 13)).toEqual([]);
    expect(ready(s, MONDAY + 14)).toEqual(["returned_days_7_13"]);
  });

  it("each step leaves on its own Monday: those ready go, the others wait", () => {
    let s = install(EMPTY_STATE, MONDAY);
    s = recordUse(s, MONDAY + 1);
    s = recordUse(s, MONDAY + 7);
    expect(ready(s, NEXT_MONDAY)).toEqual(["installed", "returned_day_1"]);
    s = afterReport(s, ready(s, NEXT_MONDAY), "accepted", 0);
    expect(steps(s)).toEqual(["returned_days_7_13"]);
    expect(ready(s, NEXT_MONDAY + 6)).toEqual([]);
    expect(ready(s, NEXT_MONDAY + 7)).toEqual(["returned_days_7_13"]);
  });
});

describe("after a report", () => {
  const waiting = record(install(EMPTY_STATE, MONDAY), "set_up", MONDAY);

  it("accepted: the steps are done", () => {
    const s = afterReport(waiting, ["installed", "set_up"], "accepted", 5);
    expect(s.waiting).toEqual([]);
    expect(s.sent).toEqual(["installed", "set_up"]);
    expect(s.lastFailure).toBeNull();
  });

  it("refused as malformed: dropped, so a bug cannot loop", () => {
    const s = afterReport(waiting, ["installed", "set_up"], "invalid", 5);
    expect(s.waiting).toEqual([]);
    expect(due(s, NEXT_MONDAY, 5)).toBe(false);
  });

  it("a transport failure or too many requests: kept, and tried again an hour later", () => {
    const s = afterReport(waiting, ["installed", "set_up"], "retry", 1_000);
    expect(steps(s)).toEqual(["installed", "set_up"]);
    expect(s.lastFailure).toBe(1_000);
    expect(due(s, NEXT_MONDAY, 1_000 + RETRY_AFTER_MS - 1)).toBe(false);
    expect(due(s, NEXT_MONDAY, 1_000 + RETRY_AFTER_MS)).toBe(true);
  });

  it("nothing ready is never due", () => {
    expect(due(EMPTY_STATE, NEXT_MONDAY, 0)).toBe(false);
    expect(due(waiting, MONDAY, 0)).toBe(false);
    expect(due(waiting, NEXT_MONDAY, 0)).toBe(true);
  });
});

describe("forgetting, and reading what was stored", () => {
  it("forget keeps only whether the sentence was shown and whether the installation counts", () => {
    const s = { ...record(install(EMPTY_STATE, MONDAY), "set_up", MONDAY), noticeShown: true, lastFailure: 3 };
    expect(forget(s)).toEqual({ ...EMPTY_STATE, noticeShown: true, countable: true });
  });

  it("a stored state is read defensively, its week recomputed from day 0", () => {
    expect(parseState(undefined)).toEqual(EMPTY_STATE);
    expect(parseState("nonsense")).toEqual(EMPTY_STATE);
    expect(
      parseState({
        day0: WEDNESDAY,
        week: 1,
        sent: ["installed", "bogus", "installed"],
        waiting: [
          { step: "set_up", from: NEXT_MONDAY },
          { step: "set_up", from: NEXT_MONDAY + 7 },
          { step: "bogus", from: NEXT_MONDAY },
          { step: "first_analysis", from: "soon" },
          "set_up",
          7,
          null,
        ],
        noticeShown: true,
        lastFailure: 12,
        countable: true,
      }),
    ).toEqual({
      day0: WEDNESDAY,
      week: MONDAY,
      sent: ["installed"],
      waiting: [{ step: "set_up", from: NEXT_MONDAY }],
      noticeShown: true,
      lastFailure: 12,
      countable: true,
    });
    expect(parseState({ day0: 1.5, lastFailure: "x", countable: "yes" })).toEqual(EMPTY_STATE);
  });
});

describe("Each browser's consent rule decides when counting starts", () => {
  const unanswered: Consent = { kind: "switch", off: undefined };
  const yes: Consent = { kind: "switch", off: false };
  const no: Consent = { kind: "switch", off: true };
  const granted: Consent = { kind: "grant", supported: true, granted: true };
  const declined: Consent = { kind: "grant", supported: true, granted: false };
  const oldFirefox: Consent = { kind: "grant", supported: false, granted: false };
  const fresh = install(EMPTY_STATE, MONDAY);
  const shown = { ...fresh, noticeShown: true };

  it("Chromium asks: steps are held until yes, sent only after it; no is the switch off", () => {
    expect(recording(unanswered)).toBe(true);
    expect(sending("ask", unanswered, shown)).toBe(false);
    expect(sending("ask", yes, fresh)).toBe(true);
    expect(recording(no)).toBe(false);
    expect(sending("ask", no, shown)).toBe(false);
    expect(noticeKind("ask", unanswered, shown)).toBe("ask");
    expect(noticeKind("ask", yes, fresh)).toBe("none");
    expect(noticeKind("ask", no, fresh)).toBe("none");
    expect(switchView("ask", unanswered)).toEqual({ shown: true, on: false });
    expect(switchView("ask", yes)).toEqual({ shown: true, on: true });
  });

  it("Safari counts by default, once the sentence was shown", () => {
    expect(recording(unanswered)).toBe(true);
    expect(sending("on", unanswered, fresh)).toBe(false);
    expect(sending("on", unanswered, shown)).toBe(true);
    expect(noticeKind("on", unanswered, fresh)).toBe("inform");
    expect(noticeKind("on", unanswered, shown)).toBe("none");
    expect(switchView("on", unanswered)).toEqual({ shown: true, on: true });
    expect(sending("on", no, shown)).toBe(false);
    expect(switchView("on", no)).toEqual({ shown: true, on: false });
    expect(noticeKind("on", no, fresh)).toBe("none");
  });

  it("Firefox follows its own consent, informed once; an older Firefox counts nothing and shows no switch", () => {
    expect(recording(granted)).toBe(true);
    expect(sending("on", granted, fresh)).toBe(true);
    expect(noticeKind("on", granted, fresh)).toBe("inform");
    expect(noticeKind("on", granted, shown)).toBe("none");
    expect(recording(declined)).toBe(false);
    expect(sending("on", declined, shown)).toBe(false);
    expect(noticeKind("on", declined, fresh)).toBe("none");
    expect(switchView("on", declined)).toEqual({ shown: true, on: false });
    expect(recording(oldFirefox)).toBe(false);
    expect(switchView("on", oldFirefox)).toEqual({ shown: false, on: false });
    expect(countingOn("on", oldFirefox)).toBe(false);
  });

  it("An installation from before the counts is never asked, never informed and gets no leaving page, on any browser", () => {
    const updated = EMPTY_STATE; // an update: no install, no day 0
    for (const [mode, consent] of [
      ["ask", unanswered],
      ["ask", yes],
      ["on", unanswered],
      ["on", granted],
    ] as const) {
      expect(noticeKind(mode, consent, updated)).toBe("none");
      expect(leavingPageOn(mode, consent, updated)).toBe(false);
    }
    // A counted installation gets its leaving page while counting is on, and keeps it past its
    // last week (its leaving answer is counted when it arrives, not under day 0).
    expect(leavingPageOn("on", granted, fresh)).toBe(true);
    expect(leavingPageOn("ask", yes, forget(fresh))).toBe(true);
    expect(leavingPageOn("ask", unanswered, fresh)).toBe(false);
    // Nothing left to count: no sentence either.
    expect(noticeKind("ask", unanswered, forget(fresh))).toBe("none");
  });
});

describe("Leaving asks one question: the uninstall address", () => {
  it("the leaving page in the interface language, with the browser and version only", () => {
    expect(leavingUrl("fr", "chromium", "1.10.2")).toBe("https://cymbra.app/lingua/au-revoir/?b=chromium&v=1.10");
    expect(leavingUrl("en", "firefox", "1.9.0")).toBe("https://cymbra.app/en/lingua/goodbye/?b=firefox&v=1.9");
    expect(leavingUrl("es", "firefox", "2.0.0")).toBe("https://cymbra.app/es/lingua/adios/?b=firefox&v=2.0");
    for (const url of Object.values(LEAVING_PAGES)) expect(url.length).toBeLessThan(255);
  });

  it("the version as major.minor", () => {
    expect(shortVersion("1.9.0")).toBe("1.9");
    expect(shortVersion("1.10")).toBe("1.10");
    expect(shortVersion("3")).toBe("3.0");
  });
});
