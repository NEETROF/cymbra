import { describe, expect, it } from "vitest";
import {
  afterReport,
  type Consent,
  countingOn,
  due,
  EMPTY_STATE,
  expire,
  forget,
  install,
  LEAVING_PAGES,
  leavingUrl,
  localDay,
  mondayOf,
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
// installation, steps are filed under its week, and each is recorded once.

/** A local date at a given hour (the runner's own time zone, as the device's). */
const at = (y: number, m: number, d: number, h = 12, min = 0): Date => new Date(y, m - 1, d, h, min);
/** Wednesday 21 October 2026, local. */
const WEDNESDAY = localDay(at(2026, 10, 21));
const MONDAY = localDay(at(2026, 10, 19));

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
    s = record(s, "set_up", WEDNESDAY);
    s = recordAnalysis(s, WEDNESDAY);
    expect(s.waiting).toEqual(["installed", "set_up", "first_analysis"]);
    // Each once: again on day 0 adds nothing.
    s = record(recordAnalysis(s, WEDNESDAY), "set_up", WEDNESDAY);
    expect(s.waiting).toEqual(["installed", "set_up", "first_analysis"]);
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
    expect(s.waiting).toEqual(["returned_days_7_13"]);
    expect(s.waiting).not.toContain("returned_day_1");
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
    expect(s.week).toBeNull();
    // Nothing is recorded afterwards either.
    expect(recordAnalysis(s, MONDAY + 201)).toEqual(s);
  });

  it("held steps are forgotten after day 59 (Chrome, never answered)", () => {
    const held = record(install(EMPTY_STATE, MONDAY), "set_up", MONDAY);
    expect(expire(held, MONDAY + 59)).toBe(held);
    const gone = expire(held, MONDAY + 60);
    expect(gone.waiting).toEqual([]);
    expect(gone.day0).toBeNull();
  });

  it("An update, not an installation: with no day 0, nothing is ever recorded", () => {
    let s = EMPTY_STATE;
    s = recordAnalysis(s, WEDNESDAY);
    s = record(s, "set_up", WEDNESDAY + 1);
    s = recordUse(s, WEDNESDAY + 8);
    expect(s).toEqual(EMPTY_STATE);
  });

  it("A clock change around midnight: the local day decides, and a clock set back before day 0 records nothing", () => {
    const day0 = localDay(at(2026, 10, 19, 23, 59));
    expect(localDay(at(2026, 10, 20, 0, 1))).toBe(day0 + 1);
    let s = installedOn(day0);
    // Back on the next local day, a minute after midnight: day 1.
    s = recordUse(s, localDay(at(2026, 10, 20, 0, 1)));
    expect(s.waiting).toEqual(["returned_day_1"]);
    // The clock moved back to the day before installing: nothing.
    const back = recordAnalysis(installedOn(day0), day0 - 1);
    expect(back.waiting).toEqual([]);
  });

  it("A page in no studied language is never a use: only an analysis or a surface records (the host decides)", () => {
    // A use on day 1 records the return; nothing else does.
    expect(recordUse(installedOn(MONDAY), MONDAY + 1).waiting).toEqual(["returned_day_1"]);
  });

  it("a step already reported is never recorded again", () => {
    let s = recordUse(installedOn(MONDAY), MONDAY + 1);
    s = afterReport(s, ["returned_day_1"], "accepted", 1);
    expect(recordUse(s, MONDAY + 1).waiting).toEqual([]);
    expect(s.sent).toEqual(["installed", "returned_day_1"]);
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
    expect(due(s, 5)).toBe(false);
  });

  it("a transport failure or too many requests: kept, and tried again an hour later", () => {
    const s = afterReport(waiting, ["installed", "set_up"], "retry", 1_000);
    expect(s.waiting).toEqual(["installed", "set_up"]);
    expect(s.lastFailure).toBe(1_000);
    expect(due(s, 1_000 + RETRY_AFTER_MS - 1)).toBe(false);
    expect(due(s, 1_000 + RETRY_AFTER_MS)).toBe(true);
  });

  it("nothing waiting is never due", () => {
    expect(due(EMPTY_STATE, 0)).toBe(false);
    expect(due(waiting, 0)).toBe(true);
  });
});

describe("forgetting, and reading what was stored", () => {
  it("forget keeps only whether the sentence was shown", () => {
    const s = { ...record(install(EMPTY_STATE, MONDAY), "set_up", MONDAY), noticeShown: true, lastFailure: 3 };
    expect(forget(s)).toEqual({ ...EMPTY_STATE, noticeShown: true });
  });

  it("a stored state is read defensively, its week recomputed from day 0", () => {
    expect(parseState(undefined)).toEqual(EMPTY_STATE);
    expect(parseState("nonsense")).toEqual(EMPTY_STATE);
    expect(
      parseState({
        day0: WEDNESDAY,
        week: 1,
        sent: ["installed", "bogus", "installed"],
        waiting: ["set_up", 7],
        noticeShown: true,
        lastFailure: 12,
      }),
    ).toEqual({
      day0: WEDNESDAY,
      week: MONDAY,
      sent: ["installed"],
      waiting: ["set_up"],
      noticeShown: true,
      lastFailure: 12,
    });
    expect(parseState({ day0: 1.5, lastFailure: "x" })).toEqual(EMPTY_STATE);
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
