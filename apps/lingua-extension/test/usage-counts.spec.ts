// A time zone with a daylight-saving change, so the day boundary is tested across one: Paris
// leaves summer time on Sunday 25 October 2026 (a 25-hour day). Set before any date is made (each
// test file runs in its own process); a test checks it took effect.
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
  leavingPageOn,
  leavingUrl,
  localDay,
  noticeKind,
  parseState,
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
// installation, steps are filed under it, each is recorded once, and each may leave at the first
// wake after it happened (owner, 2026-10-16).

/** A local date at a given hour (the device's time zone). */
const at = (y: number, m: number, d: number, h = 12, min = 0): Date => new Date(y, m - 1, d, h, min);
/** Wednesday 21 October 2026, local. */
const WEDNESDAY = localDay(at(2026, 10, 21));
const MONDAY = localDay(at(2026, 10, 19));

/** An installation on `day0`, its installed step already sent. */
function installedOn(day0: number): UsageState {
  return afterReport(install(EMPTY_STATE, day0), ["installed"], "accepted", 0);
}

describe("Six anonymous steps, each reported once per installation", () => {
  it("days are counted on the device's calendar, as day numbers", () => {
    expect(MONDAY).toBe(Date.UTC(2026, 9, 19) / 86_400_000);
    expect(WEDNESDAY).toBe(MONDAY + 2);
    // Morning and night of one local day are one day.
    expect(localDay(at(2026, 10, 21, 0, 0))).toBe(WEDNESDAY);
    expect(localDay(at(2026, 10, 21, 23, 59))).toBe(WEDNESDAY);
  });

  it("A reader who sets up and reads on day 0: installed, set up and first page analysed, under Wednesday 21 October", () => {
    let s = install(EMPTY_STATE, WEDNESDAY);
    expect(s.day0).toBe(WEDNESDAY);
    expect(s.countable).toBe(true);
    s = record(s, "set_up", WEDNESDAY);
    s = recordAnalysis(s, WEDNESDAY);
    expect(s.waiting).toEqual(["installed", "set_up", "first_analysis"]);
    // Ready at once: nothing holds a step back but the consent and the backoff.
    expect(due(s, 0)).toBe(true);
    // Each once: again on day 0 adds nothing.
    s = record(recordAnalysis(s, WEDNESDAY), "set_up", WEDNESDAY);
    expect(s.waiting).toEqual(["installed", "set_up", "first_analysis"]);
  });

  it("an installation on a Sunday is filed under that Sunday, not its week", () => {
    const sunday = localDay(at(2026, 10, 25));
    expect(install(EMPTY_STATE, sunday).day0).toBe(sunday);
    expect(sunday).toBe(MONDAY + 6);
  });

  it("A weekend reader: back in days 7–13 once, never back on day 1", () => {
    let s = installedOn(MONDAY);
    s = recordUse(s, MONDAY + 6);
    expect(s.waiting).toEqual([]);
    s = recordUse(recordUse(s, MONDAY + 8), MONDAY + 9);
    expect(s.waiting).toEqual(["returned_days_7_13"]);
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
    expect(s.waiting).toEqual(["returned_day_1", "returned_days_7_13", "returned_days_30_59"]);
  });

  it("A late return: nothing is reported, and the device holds no day 0 any more", () => {
    const s = recordUse(installedOn(MONDAY), MONDAY + 200);
    expect(s.waiting).toEqual([]);
    expect(s.day0).toBeNull();
    // Nothing is recorded afterwards either.
    expect(recordAnalysis(s, MONDAY + 201)).toEqual(s);
  });

  it("day 59 is the last day a step is recorded; at day 60 everything is forgotten", () => {
    expect(forgetsOn(MONDAY)).toBe(MONDAY + 60);
    let s = recordUse(installedOn(MONDAY), MONDAY + LAST_DAY);
    expect(s.waiting).toEqual(["returned_days_30_59"]);
    expect(expire(s, MONDAY + LAST_DAY)).toBe(s);
    // Day 60: the step not sent by then goes with day 0.
    s = record(s, "set_up", MONDAY + 60);
    expect(s.waiting).toEqual([]);
    expect(s.day0).toBeNull();
    expect(s.countable).toBe(true);
  });

  it("held steps are forgotten at day 60 (Chrome, never answered)", () => {
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

  it("a step already reported is never recorded again", () => {
    let s = recordUse(installedOn(MONDAY), MONDAY + 1);
    s = afterReport(s, ["returned_day_1"], "accepted", 1);
    expect(recordUse(s, MONDAY + 1).waiting).toEqual([]);
    expect(s.sent).toEqual(["installed", "returned_day_1"]);
  });
});

describe("The day boundary: the device's local calendar", () => {
  it("the device's time zone has a daylight-saving change on Sunday 25 October", () => {
    expect(at(2026, 10, 24).getTimezoneOffset()).not.toBe(at(2026, 10, 26).getTimezoneOffset());
  });

  it("midnight: a use at 23:59 on day 0 is not a return; one at 00:00 the next local day is day 1", () => {
    const day0 = localDay(at(2026, 10, 20, 23, 59));
    let s = installedOn(day0);
    s = recordUse(s, localDay(at(2026, 10, 20, 23, 59)));
    expect(s.waiting).toEqual([]);
    const midnight = localDay(at(2026, 10, 21, 0, 0));
    expect(midnight).toBe(day0 + 1);
    s = recordUse(s, midnight);
    expect(s.waiting).toEqual(["returned_day_1"]);
  });

  it("the daylight-saving day is one day: installed on it, back the next day is day 1", () => {
    // Sunday 25 October lasts 25 hours in Paris: its first and last minutes are one day.
    const dst = localDay(at(2026, 10, 25, 0, 30));
    expect(localDay(at(2026, 10, 25, 2, 30))).toBe(dst);
    expect(localDay(at(2026, 10, 25, 23, 59))).toBe(dst);
    const next = localDay(at(2026, 10, 26, 0, 1));
    expect(next).toBe(dst + 1);
    let s = install(EMPTY_STATE, dst);
    expect(s.day0).toBe(dst);
    s = recordUse(s, next);
    expect(s.waiting).toEqual(["installed", "returned_day_1"]);
  });

  it("installed the day before the change, back across it: day 1 and day 7 fall where the calendar says", () => {
    const saturday = localDay(at(2026, 10, 24, 22, 0));
    let s = installedOn(saturday);
    s = recordUse(s, localDay(at(2026, 10, 25, 0, 5)));
    s = recordUse(s, localDay(at(2026, 10, 31, 23, 55)));
    expect(s.waiting).toEqual(["returned_day_1", "returned_days_7_13"]);
  });

  it("a clock set back before day 0 records nothing, and the report keeps day 0 as it was", () => {
    const day0 = WEDNESDAY;
    const back = recordAnalysis(record(installedOn(day0), "set_up", day0 - 1), day0 - 1);
    expect(back.waiting).toEqual([]);
    expect(back.day0).toBe(day0);
    // Back to the right date, the steps are recorded as usual.
    expect(recordAnalysis(back, day0).waiting).toEqual(["first_analysis"]);
  });

  it("a clock set back within the days of the installation: a return follows the day it reads, once", () => {
    // Back on day 8, then the clock is put back to day 1: day 1 is what the device says.
    let s = installedOn(WEDNESDAY);
    s = recordUse(s, WEDNESDAY + 8);
    s = recordUse(s, WEDNESDAY + 1);
    expect(s.waiting).toEqual(["returned_days_7_13", "returned_day_1"]);
    // Each still once, whatever the clock does next.
    expect(recordUse(recordUse(s, WEDNESDAY + 1), WEDNESDAY + 9).waiting).toEqual([
      "returned_days_7_13",
      "returned_day_1",
    ]);
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

  it("only the steps reported are done: one recorded meanwhile still waits", () => {
    const more = record(waiting, "first_analysis", MONDAY);
    const s = afterReport(more, ["installed", "set_up"], "accepted", 5);
    expect(s.waiting).toEqual(["first_analysis"]);
  });

  it("refused as malformed: dropped, so a bug cannot loop", () => {
    const s = afterReport(waiting, ["installed", "set_up"], "invalid", 5);
    expect(s.waiting).toEqual([]);
    expect(due(s, 5)).toBe(false);
  });

  it("a transport failure or too many requests: kept, and tried again an hour later", () => {
    const s = afterReport(waiting, ["installed", "set_up"], "retry", 1_000_000);
    expect(s.waiting).toEqual(["installed", "set_up"]);
    expect(s.lastFailure).toBe(1_000_000);
    expect(due(s, 1_000_000 + RETRY_AFTER_MS - 1)).toBe(false);
    expect(due(s, 1_000_000 + RETRY_AFTER_MS)).toBe(true);
  });

  it("a clock set back after a failure does not hold the steps until it catches up", () => {
    const s = afterReport(waiting, ["installed", "set_up"], "retry", 1_000_000);
    expect(due(s, 1_000_000 - 24 * RETRY_AFTER_MS)).toBe(true);
  });

  it("nothing waiting is never due", () => {
    expect(due(EMPTY_STATE, 0)).toBe(false);
    expect(due(waiting, 0)).toBe(true);
  });
});

describe("forgetting, and reading what was stored", () => {
  it("forget keeps only whether the sentence was shown and whether the installation counts", () => {
    const s = { ...record(install(EMPTY_STATE, MONDAY), "set_up", MONDAY), noticeShown: true, lastFailure: 3 };
    expect(forget(s)).toEqual({ ...EMPTY_STATE, noticeShown: true, countable: true });
  });

  it("a stored state is read defensively", () => {
    expect(parseState(undefined)).toEqual(EMPTY_STATE);
    expect(parseState("nonsense")).toEqual(EMPTY_STATE);
    expect(
      parseState({
        day0: WEDNESDAY,
        sent: ["installed", "bogus", "installed"],
        waiting: ["set_up", "set_up", "bogus", 7, null, { step: "first_analysis" }],
        noticeShown: true,
        lastFailure: 12,
        countable: true,
      }),
    ).toEqual({
      day0: WEDNESDAY,
      sent: ["installed"],
      waiting: ["set_up"],
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
    // A counted installation gets its leaving page while counting is on, and keeps it past day
    // 59 (its leaving answer is counted when it arrives, not under day 0).
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
