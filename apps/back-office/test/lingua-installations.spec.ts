import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { Code, ConnectError } from "@connectrpc/connect";
import { setClientsForTest } from "@/lib/api";
import type { Clients } from "@/lib/transport";
import { useLinguaStore } from "@/stores/lingua";
import {
  LEAVE_REASONS,
  type FunnelCounts,
  type WeekCounts,
  dailyInstallations,
  finalFrom,
  funnelDay,
  funnelWeek,
  installationFunnel,
  mondayOf,
  orderReasons,
  shareOf,
  sumByWeek,
  weeklyInstallations,
} from "@/lib/linguaFunnel";
import en from "@/i18n/locales/en.json";
import fr from "@/i18n/locales/fr.json";

// The Installations section of the Lingua console (change: add-lingua-usage-counts, tasks 3.1):
// the anonymous funnel by installation day — or summed by week, without another read — and the
// leaving reasons, each its own Async union in the store, with the shares and the "not yet" cells
// computed there. Devices send each step as soon as it happens (owner decision 2026-10-16), so a
// return is final the day after its window closed.

interface Win {
  fromDay: string;
  toDay: string;
}
interface FilterReq {
  window: Win;
  browser: string;
}

/** A Thursday: its day-1 window ends 10-02, days 7–13 end 10-14, days 30–59 end 11-29. */
const DAY = "2026-10-01";
/** The week of 2026-09-28 (a Monday): its last day is Sunday 2026-10-04. */
const WEEK = "2026-09-28";

function counts(over: Partial<FunnelCounts> = {}): FunnelCounts {
  return {
    day: DAY,
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

function weekCounts(over: Partial<WeekCounts> = {}): WeekCounts {
  return { ...counts(), week: WEEK, ...over };
}

function fakeClients(opts: { failFunnel?: boolean; failReasons?: boolean } = {}) {
  const funnelCalls: FilterReq[] = [];
  const reasonCalls: FilterReq[] = [];
  const day = (d: string, n: [bigint, bigint, bigint, bigint, bigint, bigint], capped = false) => ({
    day: d,
    installed: n[0],
    setUp: n[1],
    firstAnalysis: n[2],
    returnedDay1: n[3],
    returnedDays7To13: n[4],
    returnedDays30To59: n[5],
    capped,
  });
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
        // Ascending by day, like the service. 07-06 (a Monday) and 07-09 share a week.
        return {
          days: [
            day("2026-07-06", [40n, 30n, 25n, 12n, 8n, 4n]),
            day("2026-07-09", [10n, 5n, 5n, 3n, 2n, 1n]),
            day("2026-10-04", [5000n, 10n, 7n, 3n, 2n, 0n], true),
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
  it("is a rounded whole percent of the row's installations", () => {
    expect(shareOf(25, 40)).toBe(63); // 62.5 rounds up
    expect(shareOf(8, 40)).toBe(20);
    expect(shareOf(40, 40)).toBe(100);
    expect(shareOf(0, 40)).toBe(0);
    expect(shareOf(1, 3)).toBe(33);
  });

  it("is absent when the row has no installation", () => {
    expect(shareOf(0, 0)).toBeNull();
    expect(shareOf(3, 0)).toBeNull();
  });
});

describe("funnelDay: the 'not yet' cells of an installation day", () => {
  it("makes each return final the day after its window: day + 2, + 14, + 60", () => {
    expect(finalFrom(DAY, "returnedDay1")).toBe("2026-10-03");
    expect(finalFrom(DAY, "returnedDays7To13")).toBe("2026-10-15");
    expect(finalFrom(DAY, "returnedDays30To59")).toBe("2026-11-30");
    expect(finalFrom(DAY, "setUp")).toBeNull();
    expect(finalFrom(DAY, "firstAnalysis")).toBeNull();
  });

  const kinds = (today: string) => {
    const row = funnelDay(counts(), today);
    return [row.cells.returnedDay1.kind, row.cells.returnedDays7To13.kind, row.cells.returnedDays30To59.kind];
  };

  it("reads 'not yet' for every return on the installation day itself", () => {
    expect(kinds(DAY)).toEqual(["notYet", "notYet", "notYet"]);
  });

  it("holds day 1 through day + 1 and shows it from day + 2", () => {
    expect(kinds("2026-10-02")).toEqual(["notYet", "notYet", "notYet"]);
    expect(kinds("2026-10-03")).toEqual(["count", "notYet", "notYet"]);
  });

  it("holds days 7–13 through day + 13 and shows them from day + 14", () => {
    expect(kinds("2026-10-14")).toEqual(["count", "notYet", "notYet"]);
    expect(kinds("2026-10-15")).toEqual(["count", "count", "notYet"]);
  });

  it("holds days 30–59 through day + 59 and shows them from day + 60", () => {
    expect(kinds("2026-11-29")).toEqual(["count", "count", "notYet"]);
    expect(kinds("2026-11-30")).toEqual(["count", "count", "count"]);
  });

  it("never holds back the steps without a window, even on the installation day", () => {
    const row = funnelDay(counts(), DAY);
    expect(row.cells.setUp).toEqual({ kind: "count", count: 30, share: 75 });
    expect(row.cells.firstAnalysis).toEqual({ kind: "count", count: 25, share: 63 });
  });

  it("gives each closed step its count and its share of installed", () => {
    const row = funnelDay(counts(), "2027-01-01");
    expect(row.period).toBe(DAY);
    expect(row.installed).toBe(40);
    expect(row.cells.returnedDay1).toEqual({ kind: "count", count: 12, share: 30 });
    expect(row.cells.returnedDays7To13).toEqual({ kind: "count", count: 8, share: 20 });
    expect(row.cells.returnedDays30To59).toEqual({ kind: "count", count: 4, share: 10 });
  });

  it("keeps the counts but no share for a day with no installation", () => {
    const row = funnelDay(counts({ installed: 0, setUp: 0, firstAnalysis: 1 }), "2027-01-01");
    expect(row.cells.firstAnalysis).toEqual({ kind: "count", count: 1, share: null });
  });

  it("carries the capped mark", () => {
    expect(funnelDay(counts({ capped: true }), "2027-01-01").capped).toBe(true);
    expect(funnelDay(counts(), "2027-01-01").capped).toBe(false);
  });
});

describe("funnelWeek: the 'not yet' cells of a week", () => {
  // The window has to close for the week's last day, Sunday 10-04: day 1 on 10-05, days 7–13 on
  // 10-17, days 30–59 on 12-02.
  const kinds = (today: string) => {
    const row = funnelWeek(weekCounts(), today);
    return [row.cells.returnedDay1.kind, row.cells.returnedDays7To13.kind, row.cells.returnedDays30To59.kind];
  };

  it("holds day 1 through the Monday after the week and shows it from the Tuesday", () => {
    expect(kinds("2026-10-05")).toEqual(["notYet", "notYet", "notYet"]);
    expect(kinds("2026-10-06")).toEqual(["count", "notYet", "notYet"]);
  });

  it("holds days 7–13 through Sunday + 13 and shows them from Sunday + 14", () => {
    expect(kinds("2026-10-17")).toEqual(["count", "notYet", "notYet"]);
    expect(kinds("2026-10-18")).toEqual(["count", "count", "notYet"]);
  });

  it("holds days 30–59 through Sunday + 59 and shows them from Sunday + 60", () => {
    expect(kinds("2026-12-02")).toEqual(["count", "count", "notYet"]);
    expect(kinds("2026-12-03")).toEqual(["count", "count", "count"]);
  });

  it("is headed by the week's Monday, with its shares and capped mark", () => {
    const row = funnelWeek(weekCounts({ capped: true }), "2027-01-01");
    expect(row.period).toBe(WEEK);
    expect(row.cells.returnedDays7To13).toEqual({ kind: "count", count: 8, share: 20 });
    expect(row.capped).toBe(true);
  });
});

describe("sumByWeek", () => {
  it("sums the days by their Monday, Sunday included, capped when any day is", () => {
    const weeks = sumByWeek([
      counts({ day: "2026-09-28", installed: 10, setUp: 8, returnedDay1: 2 }),
      counts({ day: "2026-10-04", installed: 30, setUp: 20, returnedDay1: 5, capped: true }),
      counts({ day: "2026-10-05", installed: 7 }),
    ]);
    expect(weeks.map((w) => [w.week, w.installed, w.setUp, w.returnedDay1, w.capped])).toEqual([
      ["2026-09-28", 40, 28, 7, true],
      ["2026-10-05", 7, 30, 12, false],
    ]);
    expect(weeks[0].firstAnalysis).toBe(50);
    expect(weeks[0].returnedDays30To59).toBe(8);
  });

  it("lists the weeks ascending, whatever order the days came in", () => {
    const weeks = sumByWeek([counts({ day: "2026-10-12" }), counts({ day: "2026-09-29" })]);
    expect(weeks.map((w) => w.week)).toEqual(["2026-09-28", "2026-10-12"]);
  });

  it("keeps a malformed day in a row of its own", () => {
    expect(sumByWeek([counts({ day: "not a day" })]).map((w) => w.week)).toEqual(["not a day"]);
  });
});

describe("the chart's series", () => {
  it("by day: every day of the window, a quiet day at 0", () => {
    expect(dailyInstallations([{ day: "2026-10-01", installed: 7 }], "2026-09-29", "2026-10-03")).toEqual({
      labels: ["2026-09-29", "2026-09-30", "2026-10-01", "2026-10-02", "2026-10-03"],
      installed: [0, 0, 7, 0, 0],
    });
  });

  it("by day: stops at 400 days, keeps a day returned outside the window, survives a cleared date", () => {
    expect(dailyInstallations([], "2025-01-01", "2026-12-31").labels).toHaveLength(400);
    expect(dailyInstallations([{ day: "2026-09-21", installed: 2 }], "2026-09-23", "2026-09-23")).toEqual({
      labels: ["2026-09-21", "2026-09-23"],
      installed: [2, 0],
    });
    expect(dailyInstallations([{ day: "2026-09-21", installed: 2 }], "", "")).toEqual({
      labels: ["2026-09-21"],
      installed: [2],
    });
  });

  it("by week: every Monday the window touches, a quiet week at 0", () => {
    // 2026-09-30 is a Wednesday (week of 09-28); 2026-10-14 a Wednesday (week of 10-12).
    expect(weeklyInstallations([{ week: "2026-10-05", installed: 7 }], "2026-09-30", "2026-10-14")).toEqual({
      labels: ["2026-09-28", "2026-10-05", "2026-10-12"],
      installed: [0, 7, 0],
    });
  });

  it("by week: keeps a week the server returned outside the range, and survives a cleared date", () => {
    expect(weeklyInstallations([{ week: "2026-09-21", installed: 2 }], "", "")).toEqual({
      labels: ["2026-09-21"],
      installed: [2],
    });
  });

  it("finds the Monday of any day, Sunday included", () => {
    expect(mondayOf("2026-10-04")).toBe("2026-09-28"); // a Sunday
    expect(mondayOf("2026-09-28")).toBe("2026-09-28");
    expect(mondayOf("not a day")).toBeNull();
  });
});

describe("installationFunnel", () => {
  it("lists the rows latest first in both views, and the series ascending", () => {
    const funnel = installationFunnel(
      [counts({ day: "2026-09-29" }), counts({ day: "2026-10-06" }), counts({ day: "2026-10-01" })],
      { fromDay: "2026-09-29", toDay: "2026-10-06" },
      "2027-01-01",
    );
    expect(funnel.day.rows.map((r) => r.period)).toEqual(["2026-10-06", "2026-10-01", "2026-09-29"]);
    expect(funnel.week.rows.map((r) => [r.period, r.installed])).toEqual([
      ["2026-10-05", 40],
      ["2026-09-28", 80],
    ]);
    expect(funnel.day.series.labels).toHaveLength(8);
    expect(funnel.day.series.labels[0]).toBe("2026-09-29");
    expect(funnel.week.series).toEqual({ labels: ["2026-09-28", "2026-10-05"], installed: [80, 40] });
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
    vi.setSystemTime(new Date("2026-10-05T23:30:00Z")); // the day after 10-04: its day-1 return is not final
  });
  afterEach(() => vi.useRealTimers());

  it("loads the funnel by day, latest first, with shares, 'not yet' cells and the capped mark", async () => {
    const { clients } = fakeClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load({ fromDay: "2026-07-01", toDay: "2026-10-05" });

    expect(store.funnelGrain).toBe("day");
    expect(store.funnel.status).toBe("success");
    if (store.funnel.status !== "success") return;
    const { rows, series } = store.funnel.data.day;
    expect(rows.map((r) => r.period)).toEqual(["2026-10-04", "2026-07-09", "2026-07-06"]);
    const [young, , old] = rows;
    // An old day: every figure final, as a count and a share of "installed".
    expect(old.cells.firstAnalysis).toEqual({ kind: "count", count: 25, share: 63 });
    expect(old.cells.returnedDays7To13).toEqual({ kind: "count", count: 8, share: 20 });
    expect(old.capped).toBe(false);
    // Yesterday, capped: its returns are not final yet; the steps without a window are.
    expect(young.capped).toBe(true);
    expect(young.cells.returnedDay1).toEqual({ kind: "notYet" });
    expect(young.cells.returnedDays7To13).toEqual({ kind: "notYet" });
    expect(young.cells.setUp).toEqual({ kind: "count", count: 10, share: 0 });
    // The chart's series: every day of the window, a quiet day at 0.
    expect(series.labels[0]).toBe("2026-07-01");
    expect(series.labels.at(-1)).toBe("2026-10-05");
    expect(series.labels).toHaveLength(97);
    expect(series.installed[series.labels.indexOf("2026-10-04")]).toBe(5000);
    expect(series.installed[series.labels.indexOf("2026-10-05")]).toBe(0);
  });

  it("sums the same days by week, latest first, without another read", async () => {
    const { clients, funnelCalls } = fakeClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load({ fromDay: "2026-07-01", toDay: "2026-10-05" });
    const reads = funnelCalls.length;
    store.funnelGrain = "week";

    expect(funnelCalls).toHaveLength(reads);
    if (store.funnel.status !== "success") throw new Error(store.funnel.status);
    const { rows, series } = store.funnel.data.week;
    expect(rows.map((r) => r.period)).toEqual(["2026-09-28", "2026-07-06"]);
    const [young, old] = rows;
    // 07-06 and 07-09 summed: 50 installed, 30 first pages, 15 back on day 1.
    expect(old.installed).toBe(50);
    expect(old.cells.firstAnalysis).toEqual({ kind: "count", count: 30, share: 60 });
    expect(old.cells.returnedDay1).toEqual({ kind: "count", count: 15, share: 30 });
    expect(old.capped).toBe(false);
    // The week of 10-04 holds a capped day; its day-1 return is final from tomorrow, for its Sunday.
    expect(young.capped).toBe(true);
    expect(young.cells.returnedDay1).toEqual({ kind: "notYet" });
    expect(series.labels[0]).toBe("2026-06-29");
    expect(series.labels.at(-1)).toBe("2026-10-05");
    expect(series.installed[series.labels.indexOf("2026-07-06")]).toBe(50);
  });

  it("shows a day-1 return from the second day after the installation", async () => {
    vi.setSystemTime(new Date("2026-10-06T00:00:01Z"));
    const { clients } = fakeClients();
    setClientsForTest(clients);
    const store = useLinguaStore();

    await store.load();

    if (store.funnel.status !== "success") throw new Error(store.funnel.status);
    const young = store.funnel.data.day.rows.find((r) => r.period === "2026-10-04");
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

    // A later full reload keeps the filter, and the grain the admin picked.
    store.funnelGrain = "week";
    await store.load();
    expect(funnelCalls.at(-1)?.browser).toBe("firefox");
    expect(reasonCalls.at(-1)?.browser).toBe("firefox");
    expect(store.funnelGrain).toBe("week");
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
