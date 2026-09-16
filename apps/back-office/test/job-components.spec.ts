import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import { i18n } from "@/i18n";
import { AttemptOutcome } from "@/gen/jobs_admin_pb";
import CadenceBadge from "@/components/CadenceBadge.vue";
import JobHistoryTable from "@/components/JobHistoryTable.vue";
import JobKindBreakdown from "@/components/JobKindBreakdown.vue";
import type { FinishedAttemptRow, JobKindInfo, JobStats } from "@/stores/jobs";

// Change: add-jobs-console-history. The presentational pieces of the Jobs screen: props
// in, events out, no store and no API.

const global = { plugins: [i18n] };

const kinds: JobKindInfo[] = [
  {
    name: "plans_reconcile",
    channel: "plans.reconcile",
    cancellable: true,
    schedules: [{ name: "plans_reconcile_daily", cron: "10 4 * * *", timezone: "UTC", enabled: true }],
  },
  {
    name: "streak_reminder",
    channel: "notifications.streak",
    cancellable: true,
    schedules: [{ name: "streak_reminder_hourly", cron: "5 * * * *", timezone: "UTC", enabled: true }],
  },
  { name: "verification_email", channel: "auth.email", cancellable: true, schedules: [] },
];

const zero = { completed: 0, failedAttempts: 0, deadLettered: 0, cancelled: 0, avgRunMs: null };

function stats(over: Partial<JobStats> = {}): JobStats {
  return {
    queue: { total: 0, running: 0, ready: 0, scheduled: 0, retryWait: 0, blocked: 0, exhausted: 0 },
    period: { ...zero, completed: 30 },
    historySince: null,
    byKind: [
      {
        kind: "verification_email",
        period: { ...zero, completed: 30, avgRunMs: 850 },
        lastFinishedAt: Date.now() - 60_000,
      },
    ],
    ...over,
  };
}

describe("CadenceBadge", () => {
  it("claims nothing while the kinds are unknown", () => {
    const w = mount(CadenceBadge, { props: { schedules: null }, global });
    expect(w.find('[data-testid="cadence"]').exists()).toBe(false);
  });

  it.each([
    [[], "onDemand", "On demand", "neutral"],
    [kinds[1].schedules, "scheduled", "Scheduled · hourly at :05", "accent"],
    [[{ ...kinds[0].schedules[0], enabled: false }], "paused", "Schedule paused", "pending"],
  ] as const)("renders %j as %s", (schedules, type, text, variant) => {
    const w = mount(CadenceBadge, { props: { schedules }, global });
    const tag = w.get('[data-testid="cadence"]');
    expect(tag.attributes("data-cadence")).toBe(type);
    expect(tag.text()).toBe(text);
    expect(tag.classes()).toContain(variant);
  });
});

describe("JobKindBreakdown", () => {
  it("lists every registered kind, with zeros and its cadence", () => {
    const w = mount(JobKindBreakdown, { props: { stats: stats(), kinds, kindFilter: "" }, global });
    const rows = w.findAll('[data-testid="breakdown-row"]');
    expect(rows.map((r) => r.attributes("data-kind"))).toEqual([
      "plans_reconcile",
      "streak_reminder",
      "verification_email",
    ]);

    const plans = rows[0];
    expect(plans.get('[data-testid="breakdown-completed"]').text()).toBe("0");
    expect(plans.get('[data-testid="breakdown-last"]').text()).toBe("—");
    expect(plans.get('[data-testid="cadence"]').text()).toBe("Scheduled · daily at 04:10 (UTC)");
    expect(plans.classes()).toContain("idle");

    const email = rows[2];
    expect(email.get('[data-testid="breakdown-completed"]').text()).toBe("30");
    expect(email.text()).toContain("850 ms");
    expect(email.get('[data-testid="breakdown-last"]').text()).toBe("1 minute ago");
    expect(email.get('[data-testid="breakdown-last"]').attributes("title")).toBeTruthy();
    expect(email.get('[data-testid="cadence"]').text()).toBe("On demand");
  });

  it("narrows to the filtered kind", () => {
    const w = mount(JobKindBreakdown, {
      props: { stats: stats(), kinds, kindFilter: "streak_reminder" },
      global,
    });
    expect(w.findAll('[data-testid="breakdown-row"]').map((r) => r.attributes("data-kind"))).toEqual([
      "streak_reminder",
    ]);
  });

  it("asks for a kind's history when it is selected", async () => {
    const w = mount(JobKindBreakdown, { props: { stats: stats(), kinds, kindFilter: "" }, global });
    await w.findAll('[data-testid="breakdown-open"]')[1].trigger("click");
    expect(w.emitted("select")).toEqual([["streak_reminder"]]);
  });

  it("says the breakdown is unavailable instead of showing zeros", () => {
    const w = mount(JobKindBreakdown, { props: { stats: stats({ byKind: [] }), kinds, kindFilter: "" }, global });
    expect(w.get('[data-testid="breakdown-unavailable"]').text()).toContain("isn't available");
    expect(w.findAll('[data-testid="breakdown-row"]')).toHaveLength(0);
  });

  it("renders nothing without statistics, and no cadence without kinds", () => {
    expect(mount(JobKindBreakdown, { props: { stats: null, kinds, kindFilter: "" }, global }).html()).not.toContain(
      "breakdown",
    );
    const w = mount(JobKindBreakdown, { props: { stats: stats(), kinds: null, kindFilter: "" }, global });
    expect(w.findAll('[data-testid="breakdown-row"]')).toHaveLength(1);
    expect(w.find('[data-testid="cadence"]').exists()).toBe(false);
  });
});

const attempt = (over: Partial<FinishedAttemptRow> = {}): FinishedAttemptRow => ({
  jobId: "7a1b2c3d-1111-4222-8333-444455556666",
  kind: "streak_reminder",
  channel: "notifications.streak",
  outcome: AttemptOutcome.SUCCEEDED,
  attempt: 1,
  startedAt: Date.now() - 62_000,
  finishedAt: Date.now() - 60_000,
  durationMs: 2_000,
  ...over,
});

function history(props: Partial<InstanceType<typeof JobHistoryTable>["$props"]> = {}) {
  return mount(JobHistoryTable, {
    props: {
      attempts: [
        attempt(),
        attempt({ kind: "verification_email", outcome: AttemptOutcome.FAILED, attempt: 2 }),
        attempt({ outcome: AttemptOutcome.ABANDONED, durationMs: null, jobId: "0000aaaa-1111" }),
        attempt({ outcome: AttemptOutcome.UNSPECIFIED, jobId: "ffff0000-1111" }),
      ],
      total: 60,
      loading: false,
      error: null,
      offset: 0,
      limit: 25,
      outcome: AttemptOutcome.UNSPECIFIED,
      kinds,
      ...props,
    },
    global,
  });
}

describe("JobHistoryTable", () => {
  it("shows each attempt's kind, cadence, outcome, attempt and run time", () => {
    const w = history();
    const rows = w.findAll('[data-testid="history-row"]');
    expect(rows).toHaveLength(4);
    expect(rows[0].get('[data-testid="history-outcome-tag"]').text()).toBe("Succeeded");
    expect(rows[0].get('[data-testid="history-outcome-tag"]').classes()).toContain("accepted");
    expect(rows[0].get('[data-testid="cadence"]').text()).toBe("Scheduled · hourly at :05");
    expect(rows[0].get('[data-testid="history-duration"]').text()).toBe("2 s");
    expect(rows[0].text()).toContain("1 minute ago");
    expect(rows[0].get("code").attributes("title")).toBe("7a1b2c3d-1111-4222-8333-444455556666");

    expect(rows[1].get('[data-testid="history-outcome-tag"]').text()).toBe("Failed");
    expect(rows[1].get('[data-testid="cadence"]').text()).toBe("On demand");
    expect(rows[1].text()).toContain("2");

    expect(rows[2].get('[data-testid="history-outcome-tag"]').text()).toBe("Abandoned");
    expect(rows[2].get('[data-testid="history-duration"]').text()).toBe("—");
    // An outcome this build does not know reads as a dash, not a raw number.
    expect(rows[3].get('[data-testid="history-outcome-tag"]').text()).toBe("—");

    expect(w.get('[data-testid="history-total"]').text()).toBe("Finished attempts: 60");
  });

  it("relays the outcome filter and the page", async () => {
    const w = history({ outcome: AttemptOutcome.FAILED });
    const select = w.get('[data-testid="history-outcome"]');
    expect((select.element as HTMLSelectElement).value).toBe(String(AttemptOutcome.FAILED));

    await select.setValue(String(AttemptOutcome.ABANDONED));
    expect(w.emitted("outcome")).toEqual([[AttemptOutcome.ABANDONED]]);

    await w.get(".pager button:last-child").trigger("click");
    expect(w.emitted("page")).toEqual([[25]]);

    await w.setProps({ outcome: AttemptOutcome.SUCCEEDED });
    expect((select.element as HTMLSelectElement).value).toBe(String(AttemptOutcome.SUCCEEDED));
  });

  it("shows the loading, empty and error states", () => {
    expect(history({ attempts: [], loading: true }).find('[data-testid="history-loading"]').exists()).toBe(true);
    const empty = history({ attempts: [], total: 0 });
    expect(empty.get('[data-testid="history-empty"]').text()).toBe("No attempt finished in this period.");
    const failed = history({ attempts: [], total: 0, error: "Service unavailable. Try again." });
    expect(failed.get('[data-testid="history-error"]').text()).toBe("Service unavailable. Try again.");
    expect(failed.find('[data-testid="history-empty"]').exists()).toBe(false);
    expect(failed.find('[data-testid="history-total"]').exists()).toBe(false);
  });

  it("claims no cadence while the kinds are unknown", () => {
    expect(history({ kinds: null }).find('[data-testid="cadence"]').exists()).toBe(false);
  });
});
