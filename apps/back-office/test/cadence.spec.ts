import { describe, expect, it } from "vitest";
import { i18n } from "@/i18n";
import { cadenceText, kindCadence, kindCadenceLabel, parseCron, type ScheduleInfo } from "@/lib/cadence";
import { formatAbsolute, formatCount, formatDuration, formatRelative } from "@/lib/jobFormat";

// Change: add-jobs-console-history. The cadence a kind carries in the Jobs console, read
// from the raw cron the server returns, and the formatting helpers of that screen.

const t = i18n.global.t as unknown as (key: string, values?: Record<string, unknown>) => string;

const schedule = (cron: string, over: Partial<ScheduleInfo> = {}): ScheduleInfo => ({
  name: `s-${cron}`,
  cron,
  timezone: "UTC",
  enabled: true,
  ...over,
});

describe("parseCron", () => {
  // The ten schedules the migrations seed (backend/jobs/migrations 0007–0017).
  it.each([
    ["0 * * * *", "Scheduled · hourly at :00"], // orphan_reap, session_reap
    ["5 * * * *", "Scheduled · hourly at :05"], // streak_reminder
    ["15 * * * *", "Scheduled · hourly at :15"], // consensus_honesty_settlement
    ["20 0 * * *", "Scheduled · daily at 00:20 (UTC)"], // global_season_snapshot
    ["10 3 * * *", "Scheduled · daily at 03:10 (UTC)"], // usage_rollup
    ["30 3 * * *", "Scheduled · daily at 03:30 (UTC)"], // play_detail_prune
    ["40 3 * * *", "Scheduled · daily at 03:40 (UTC)"], // usage_purge
    ["10 4 * * *", "Scheduled · daily at 04:10 (UTC)"], // plans_reconcile
    ["40 4 * * *", "Scheduled · daily at 04:40 (UTC)"], // plans_withdraw
  ])("labels the seeded cron %s", (cron, label) => {
    expect(kindCadenceLabel(kindCadence([schedule(cron)]), t)).toBe(label);
  });

  it("keeps a daily schedule's own timezone", () => {
    expect(parseCron("0 7 * * *", "Europe/Paris")).toEqual({
      type: "daily",
      hour: 7,
      minute: 0,
      timezone: "Europe/Paris",
    });
    expect(cadenceText(parseCron("0 7 * * *", "Europe/Paris"), t)).toBe("daily at 07:00 (Europe/Paris)");
  });

  it("drops a zero seconds field, as the scheduler accepts it", () => {
    expect(parseCron("0 5 * * * *", "UTC")).toEqual({ type: "hourly", minute: 5 });
  });

  it.each(["0 */6 * * 1-5", "0 3 1 * *", "61 * * * *", "5 24 * * *", "*/5 * * * *", "0 3 * *", "7 0 3 * * *"])(
    "shows %s as written",
    (cron) => {
      expect(parseCron(cron, "Europe/Paris")).toEqual({ type: "custom", cron, timezone: "Europe/Paris" });
    },
  );

  it("renders an unusual cron verbatim with its timezone", () => {
    expect(kindCadenceLabel(kindCadence([schedule("0 */6 * * 1-5", { timezone: "Europe/Paris" })]), t)).toBe(
      "Scheduled · 0 */6 * * 1-5 (Europe/Paris)",
    );
  });
});

describe("kindCadence", () => {
  it("a kind without schedule runs on demand", () => {
    expect(kindCadence([])).toEqual({ type: "onDemand" });
    expect(kindCadenceLabel({ type: "onDemand" }, t)).toBe("On demand");
  });

  it("a kind whose schedules are all disabled is paused", () => {
    const cadence = kindCadence([schedule("40 3 * * *", { enabled: false })]);
    expect(cadence).toEqual({ type: "paused" });
    expect(kindCadenceLabel(cadence, t)).toBe("Schedule paused");
  });

  it("lists only the enabled schedules of a kind", () => {
    const cadence = kindCadence([
      schedule("5 * * * *"),
      schedule("0 9 * * *", { enabled: false }),
      schedule("30 18 * * *"),
    ]);
    expect(kindCadenceLabel(cadence, t)).toBe("Scheduled · hourly at :05, daily at 18:30 (UTC)");
  });
});

describe("job formatting", () => {
  it("formats counts, durations and times", () => {
    expect(formatCount(undefined, "en")).toBe("—");
    expect(formatCount(1234, "en")).toBe("1,234");
    expect(formatDuration(null, t, "en")).toBe("—");
    expect(formatDuration(undefined, t, "en")).toBe("—");
    expect(formatDuration(420, t, "en")).toBe("420 ms");
    expect(formatDuration(1500, t, "en")).toBe("1.5 s");
    expect(formatDuration(90_000, t, "en")).toBe("1.5 min");
    const now = new Date("2026-09-15T12:00:00Z").getTime();
    expect(formatRelative(null, now, "en")).toBe("—");
    expect(formatRelative(now - 30_000, now, "en")).toBe("30 seconds ago");
    expect(formatRelative(now + 5 * 60_000, now, "en")).toBe("in 5 minutes");
    expect(formatRelative(now - 3 * 3_600_000, now, "en")).toBe("3 hours ago");
    expect(formatRelative(now - 2 * 86_400_000, now, "en")).toBe("2 days ago");
    expect(formatAbsolute(now, "en")).toBe(new Date(now).toLocaleString("en"));
  });
});
