import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { useLinguaStore } from "@/stores/lingua";
import {
  LEAVE_REASONS,
  type FunnelCounts,
  finalOn,
  funnelWeek,
  installationsSeries,
  mondayOf,
  orderReasons,
  shareOf,
} from "@/lib/linguaFunnel";
import en from "@/i18n/locales/en.json";
import fr from "@/i18n/locales/fr.json";

// The Installations section of the Lingua console (change: add-lingua-usage-counts, tasks 3.1):
// the anonymous funnel by installation week and the leaving reasons, each its own Async union in
// the store, with the shares and the "not yet" cells computed there.

interface Win {
  fromDay: string;
  toDay: string;
}
interface FilterReq {
  window: Win;
  browser: string;
}

/** The week of 2026-09-28 (a Monday): its last day is Sunday 2026-10-04. */
const WEEK = "2026-09-28";

function counts(over: Partial<FunnelCounts> = {}): FunnelCounts {
  return {
    week: WEEK,
    installed: 40,
    setUp: 30,
    firstAnalysis: 25,
    returnedDay1: 12,
    returnedDays7To13: 8,
    returnedDays30To59: 4,
    capped: false,
    ...over,
  };
}

function fakeClients(opts: { failFunnel?: boolean; failReasons?: boolean } = {}) {
  const funnelCalls: FilterReq[] = [];
  const reasonCalls: FilterReq[] = [];
  const clients = {
    lingua: {
      adminGetLinguaUsage: async () => ({
        activeAccounts: 0n,
        wordsLearned: 0n,
        reviews: 0n,
        wordsRead: 0n,
        newWordsSeen: 0n,
        byLanguage: [],
      }),
      adminGetLinguaUsageSeries: async () => ({ points: [] }),
      adminGetOnlineTranslationUsage: async () => ({
        rows: [],
        answered: 0n,
        failed: 0n,
        limited: 0n,
        costMicroUsd: 0n,
      }),
      adminGetUsageFunnel: async (req: FilterReq) => {
        funnelCalls.push(req);
        if (opts.failFunnel) throw new ConnectError("[permission_denied] lingua admin only", Code.PermissionDenied);
        return {
          weeks: [
            {
              week: "2026-07-06",
              installed: 40n,
              setUp: 30n,
              firstAnalysis: 25n,
              returnedDay1: 12n,
              returnedDays7To13: 8n,
              returnedDays30To59: 4n,
              capped: false,
            },
            {
              week: WEEK,
              installed: 5000n,
              setUp: 10n,
              firstAnalysis: 7n,
              returnedDay1: 3n,
              returnedDays7To13: 2n,
              returnedDays30To59: 0n,
              capped: true,
            },
          ],
        };
      },
      adminGetLeaveReasons: async (req: FilterReq) => {
        reasonCalls.push(req);
        if (opts.failReasons) throw new ConnectError("[unavailable] database down", Code.Unavailable);
        return {
          reasons: [
            { reason: "slow", count: 3n },
            { reason: "not_using", count: 5n },
          ],
          capped: true,
        };
      },
    },
  } as unknown as Clients;
  return { clients, funnelCalls, reasonCalls };
}

describe("shareOf", () => {
  it("is a rounded whole percent of the week's installations", () => {
    expect(shareOf(25, 40)).toBe(63); // 62.5 rounds up
    expect(shareOf(8, 40)).toBe(20);
    expect(shareOf(40, 40)).toBe(100);
    expect(shareOf(0, 40)).toBe(0);
    expect(shareOf(1, 3)).toBe(33);
  });

  it("is absent when the week has no installation", () => {
    expect(shareOf(0, 0)).toBeNull();
    expect(shareOf(3, 0)).toBeNull();
  });
});

describe("funnelWeek: the 'not yet' cells", () => {
  // A step leaves the device the Monday after its week: a Sunday (10-04) installation's day 1 (10-05)
  // arrives from 10-12, its day 13 (10-17) from 10-19, its day 59 (12-02) from 12-07.
  it("dates each return's closing from the week's Monday: + 14, + 21, + 70", () => {
    expect(finalOn(WEEK, "returnedDay1")).toBe("2026-10-12");
    expect(finalOn(WEEK, "returnedDays7To13")).toBe("2026-10-19");
    expect(finalOn(WEEK, "returnedDays30To59")).toBe("2026-12-07");
    expect(finalOn(WEEK, "setUp")).toBeNull();
    expect(finalOn(WEEK, "firstAnalysis")).toBeNull();
  });

  const kinds = (today: string) => {
    const row = funnelWeek(counts(), today);
    return [row.cells.returnedDay1.kind, row.cells.returnedDays7To13.kind, row.cells.returnedDays30To59.kind];
  };

  it("reads 'not yet' for every return during the week itself", () => {
    expect(kinds("2026-10-01")).toEqual(["notYet", "notYet", "notYet"]);
  });

  it("shows day 1 on the day it closes, not the day before", () => {
    expect(kinds("2026-10-11")).toEqual(["notYet", "notYet", "notYet"]);
    expect(kinds("2026-10-12")).toEqual(["count", "notYet", "notYet"]);
  });

  it("shows days 7–13 on the day they close, not the day before", () => {
    expect(kinds("2026-10-18")).toEqual(["count", "notYet", "notYet"]);
    expect(kinds("2026-10-19")).toEqual(["count", "count", "notYet"]);
  });

  it("shows days 30–59 on the day they close, not the day before", () => {
    expect(kinds("2026-12-06")).toEqual(["count", "count", "notYet"]);
    expect(kinds("2026-12-07")).toEqual(["count", "count", "count"]);
  });

  it("never holds back the steps without a window, even in the current week", () => {
    const row = funnelWeek(counts(), "2026-09-29");
    expect(row.cells.setUp).toEqual({ kind: "count", count: 30, share: 75 });
    expect(row.cells.firstAnalysis).toEqual({ kind: "count", count: 25, share: 63 });
  });

  it("gives each closed step its count and its share of installed", () => {
    const row = funnelWeek(counts(), "2027-01-01");
    expect(row.installed).toBe(40);
    expect(row.cells.returnedDay1).toEqual({ kind: "count", count: 12, share: 30 });
    expect(row.cells.returnedDays7To13).toEqual({ kind: "count", count: 8, share: 20 });
    expect(row.cells.returnedDays30To59).toEqual({ kind: "count", count: 4, share: 10 });
  });

  it("keeps the counts but no share for a week with no installation", () => {
    const row = funnelWeek(counts({ installed: 0, setUp: 0, firstAnalysis: 1 }), "2027-01-01");
    expect(row.cells.firstAnalysis).toEqual({ kind: "count", count: 1, share: null });
  });

  it("carries the capped mark", () => {
    expect(funnelWeek(counts({ capped: true }), "2027-01-01").capped).toBe(true);
    expect(funnelWeek(counts(), "2027-01-01").capped).toBe(false);
  });
});

describe("installationsSeries", () => {
  it("lists every Monday the window touches, a quiet week at 0", () => {
    // 2026-09-30 is a Wednesday (week of 09-28); 2026-10-14 a Wednesday (week of 10-12).
    expect(installationsSeries([{ week: "2026-10-05", installed: 7 }], "2026-09-30", "2026-10-14")).toEqual({
      weeks: ["2026-09-28", "2026-10-05", "2026-10-12"],
      installed: [0, 7, 0],
    });
  });

  it("keeps a week the server returned outside the range, and survives a cleared date", () => {
    expect(installationsSeries([{ week: "2026-09-21", installed: 2 }], "", "")).toEqual({
      weeks: ["2026-09-21"],
      installed: [2],
    });
  });

  it("finds the Monday of any day, Sunday included", () => {
    expect(mondayOf("2026-10-04")).toBe("2026-09-28"); // a Sunday
    expect(mondayOf("2026-09-28")).toBe("2026-09-28");
    expect(mondayOf("not a day")).toBeNull();
  });
});

describe("orderReasons", () => {
  it("lists the six reasons in the page's order, 0 for one nobody gave", () => {
    expect(orderReasons([{ reason: "slow", count: 3 }])).toEqual(
      LEAVE_REASONS.map((reason) => ({ reason, count: reason === "slow" ? 3 : 0 })),
    );
  });

  it("keeps a reason it does not know after the six, and sums repeats", () => {
    const out = orderReasons([
      { reason: "price", count: 1 },
      { reason: "other", count: 1 },
      { reason: "other", count: 2 },
    ]);
    expect(out.map((r) => r.reason)).toEqual([...LEAVE_REASONS, "price"]);
    expect(out.find((r) => r.reason === "other")?.count).toBe(3);
  });

  it("labels the six reasons in both locales", () => {
    for (const json of [en, fr]) expect(Object.keys(json.lingua.installations.reasons)).toEqual([...LEAVE_REASONS]);
  });
});

describe("lingua store: installations", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    // Only the clock: the store decides "not yet" against today's UTC day.
    vi.useFakeTimers({ toFake: ["Date"] });
    vi.setSystemTime(new Date("2026-10-05T23:30:00Z")); // while WEEK's day-1 return is not final
  });
  afterEach(() => vi.useRealTimers());

  it("loads the funnel with shares, 'not yet' cells and the capped mark", async () => {
    const { clients } = fakeClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load({ fromDay: "2026-07-01", toDay: "2026-10-05" });

    expect(store.funnel.status).toBe("success");
    if (store.funnel.status !== "success") return;
    const [old, young] = store.funnel.data.weeks;
    // An old week: every figure final, as a count and a share of "installed".
    expect(old.week).toBe("2026-07-06");
    expect(old.cells.firstAnalysis).toEqual({ kind: "count", count: 25, share: 63 });
    expect(old.cells.returnedDays7To13).toEqual({ kind: "count", count: 8, share: 20 });
    expect(old.capped).toBe(false);
    // A young, capped week: its returns are not final yet.
    expect(young.capped).toBe(true);
    expect(young.cells.returnedDay1).toEqual({ kind: "notYet" });
    expect(young.cells.returnedDays7To13).toEqual({ kind: "notYet" });
    expect(young.cells.setUp).toEqual({ kind: "count", count: 10, share: 0 });
    // The chart's series covers every week of the window.
    expect(store.funnel.data.series.weeks[0]).toBe("2026-06-29");
    expect(store.funnel.data.series.weeks.at(-1)).toBe("2026-10-05");
    expect(store.funnel.data.series.installed[store.funnel.data.series.weeks.indexOf(WEEK)]).toBe(5000);
  });

  it("shows a day-1 return once today reaches the day it closes", async () => {
    vi.setSystemTime(new Date("2026-10-12T00:00:01Z"));
    const { clients } = fakeClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load();

    if (store.funnel.status !== "success") throw new Error(store.funnel.status);
    const young = store.funnel.data.weeks.find((w) => w.week === WEEK);
    expect(young?.cells.returnedDay1).toEqual({ kind: "count", count: 3, share: 0 });
    expect(young?.cells.returnedDays7To13).toEqual({ kind: "notYet" });
  });

  it("loads the six leaving reasons in order, with the capped mark", async () => {
    const { clients } = fakeClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load();

    expect(store.reasons.status).toBe("success");
    if (store.reasons.status !== "success") return;
    expect(store.reasons.data.reasons.map((r) => [r.reason, r.count])).toEqual([
      ["not_using", 5],
      ["highlighting", 0],
      ["missing_language", 0],
      ["slow", 3],
      ["unclear", 0],
      ["other", 0],
    ]);
    expect(store.reasons.data.capped).toBe(true);
  });

  it("sends the console's window and the browser filter to both RPCs", async () => {
    const { clients, funnelCalls, reasonCalls } = fakeClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load({ fromDay: "2026-09-01", toDay: "2026-09-30" });
    expect(funnelCalls.at(-1)).toEqual({ window: { fromDay: "2026-09-01", toDay: "2026-09-30" }, browser: "" });
    expect(reasonCalls.at(-1)).toEqual({ window: { fromDay: "2026-09-01", toDay: "2026-09-30" }, browser: "" });

    await store.loadInstallations({ browser: "firefox" });
    expect(store.filters.browser).toBe("firefox");
    expect(funnelCalls.at(-1)).toEqual({ window: { fromDay: "2026-09-01", toDay: "2026-09-30" }, browser: "firefox" });
    expect(reasonCalls.at(-1)).toEqual({ window: { fromDay: "2026-09-01", toDay: "2026-09-30" }, browser: "firefox" });

    // A later full reload keeps the filter.
    await store.load();
    expect(funnelCalls.at(-1)?.browser).toBe("firefox");
    expect(reasonCalls.at(-1)?.browser).toBe("firefox");
  });

  it("a failed funnel read is a localised error, and leaves the reasons standing", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    const { clients } = fakeClients({ failFunnel: true });
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load();

    expect(store.funnel.status).toBe("error");
    if (store.funnel.status === "error") {
      expect(store.funnel.error).toBe(en.errors.permissionDenied);
      expect(store.funnel.error).not.toContain("permission_denied");
    }
    expect(store.reasons.status).toBe("success");
    expect(store.report.status).toBe("success");
  });

  it("a failed reasons read is a localised error, and leaves the funnel standing", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    const { clients } = fakeClients({ failReasons: true });
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.loadInstallations();

    expect(store.reasons.status).toBe("error");
    if (store.reasons.status === "error") {
      expect(store.reasons.error).toBe(en.errors.unavailable);
      expect(store.reasons.error).not.toContain("database down");
    }
    expect(store.funnel.status).toBe("success");
  });
});
