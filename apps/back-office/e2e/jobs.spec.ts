import { test, expect, seed } from "./fixtures";
import type { E2EAttempt, E2EJob } from "../src/lib/e2e-seam";

// Change: add-admin-jobs-console. Drives the "Jobs" console in a real browser against the
// gated fake seam (no backend): the queue and period cards, the paginated table, a
// cancellation through the confirm dialog, the rows that must offer no Cancel, the state
// filter — plus the scope gate (only a `global` admin reaches it).

const BASE = Date.now() - 60 * 60 * 1000;

/** A queued job, oldest first by `i`. */
function job(i: number, over: Partial<E2EJob> = {}): E2EJob {
  return {
    id: `${String(i).padStart(8, "0")}-aaaa-4aaa-8aaa-aaaaaaaaaaaa`,
    kind: "verification_email",
    channel: "identity.verification_email",
    state: "ready",
    attemptsMade: 0,
    attemptsLeft: 5,
    enqueuedAtMs: BASE + i * 1000,
    nextAttemptAtMs: BASE + i * 1000,
    ...over,
  };
}

const KINDS = [
  { name: "verification_email", channel: "identity.verification_email", cancellable: true },
  { name: "render_preview", channel: "music.render_preview", cancellable: true },
  { name: "purge_user", channel: "identity.purge_user", cancellable: false },
];

const statValue = (page: import("@playwright/test").Page, id: string) =>
  page.getByTestId(`stat-${id}`).getByTestId("stat-value");

test.describe("jobs console", () => {
  test("a global admin sees the queue cards, the rows and pagination", async ({ page }) => {
    const jobs = Array.from({ length: 30 }, (_, i) => job(i + 1));
    await seed(page, {
      loginAs: "global-admin",
      data: { jobs, jobKinds: KINDS, jobPeriod: { completed: 40, failedAttempts: 2, deadLettered: 1, cancelled: 0 } },
    });
    await page.goto("/music/queue");
    await page.getByRole("link", { name: "Jobs" }).click();

    await expect(page).toHaveURL(/\/admin\/jobs$/);
    await expect(page.getByRole("heading", { name: "Jobs" })).toBeVisible();
    await expect(statValue(page, "queue-total")).toHaveText("30");
    await expect(statValue(page, "queue-ready")).toHaveText("30");
    await expect(statValue(page, "period-completed")).toHaveText("40");
    await expect(statValue(page, "period-avg")).toHaveText("—");

    await expect(page.getByTestId("job-row")).toHaveCount(25);
    await expect(page.getByTestId("job-row").first()).toContainText("00000001");
    await expect(page.getByText("1–25 of 30")).toBeVisible();

    await page.getByRole("button", { name: "Next →" }).click();
    await expect(page.getByTestId("job-row")).toHaveCount(5);
    await expect(page.getByTestId("job-row").first()).toContainText("00000026");
    await expect(page.getByText("26–30 of 30")).toBeVisible();
  });

  test("cancelling a ready job removes the row, toasts and bumps the cancelled figure", async ({ page }) => {
    await seed(page, {
      loginAs: "global-admin",
      data: {
        jobs: [job(1), job(2, { kind: "render_preview", channel: "music.render_preview" }), job(3)],
        jobKinds: KINDS,
        jobPeriod: { completed: 0, failedAttempts: 0, deadLettered: 0, cancelled: 2 },
      },
    });
    await page.goto("/admin/jobs");
    await expect(page.getByTestId("job-row")).toHaveCount(3);
    await expect(statValue(page, "period-cancelled")).toHaveText("2");

    const target = page.getByTestId("job-row").filter({ hasText: "render_preview" });
    await target.getByTestId("job-cancel").click();
    const dialog = page.getByRole("dialog");
    await expect(dialog).toContainText("render_preview");
    await expect(dialog).toContainText("00000002");
    // The dialog names both choices: "Cancel" alone would read as the action itself.
    await expect(dialog.getByRole("button", { name: "Keep job" })).toBeVisible();
    await dialog.getByRole("button", { name: "Cancel job" }).click();

    await expect(page.getByText("Job cancelled.")).toBeVisible();
    await expect(dialog).toHaveCount(0);
    await expect(page.getByTestId("job-row")).toHaveCount(2);
    await expect(page.getByTestId("job-row").filter({ hasText: "render_preview" })).toHaveCount(0);
    await expect(statValue(page, "period-cancelled")).toHaveText("3");
    await expect(statValue(page, "queue-total")).toHaveText("2");
  });

  test("dismissing the confirmation cancels nothing", async ({ page }) => {
    await seed(page, { loginAs: "global-admin", data: { jobs: [job(1)], jobKinds: KINDS } });
    await page.goto("/admin/jobs");

    await page.getByTestId("job-cancel").click();
    await page.getByRole("dialog").getByRole("button", { name: "Keep job" }).click();

    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.getByTestId("job-row")).toHaveCount(1);
    await expect(statValue(page, "period-cancelled")).toHaveText("0");
  });

  test("running and protected rows offer no Cancel", async ({ page }) => {
    await seed(page, {
      loginAs: "global-admin",
      data: {
        jobs: [
          job(1, { state: "running", attemptsMade: 2, attemptsLeft: 3, startedAtMs: Date.now() - 5000 }),
          job(2, { kind: "purge_user", channel: "identity.purge_user" }),
          job(3),
        ],
        jobKinds: KINDS,
      },
    });
    await page.goto("/admin/jobs");

    const rows = page.getByTestId("job-row");
    await expect(rows).toHaveCount(3);
    await expect(rows.nth(0).getByTestId("job-state")).toHaveText("Running");
    await expect(rows.nth(0).getByTestId("job-cancel")).toHaveCount(0);
    await expect(rows.nth(1)).toContainText("purge_user");
    await expect(rows.nth(1).getByTestId("job-cancel")).toHaveCount(0);
    await expect(rows.nth(2).getByTestId("job-cancel")).toHaveCount(1);
    await expect(statValue(page, "queue-running")).toHaveText("1");
  });

  test("filtering by state narrows the rows and the total", async ({ page }) => {
    const jobs = Array.from({ length: 30 }, (_, i) =>
      job(i + 1, i % 7 === 0 ? { state: "retry_wait", attemptsMade: 1, attemptsLeft: 4 } : {}),
    );
    await seed(page, { loginAs: "global-admin", data: { jobs, jobKinds: KINDS } });
    await page.goto("/admin/jobs");
    await expect(page.getByTestId("jobs-total")).toContainText("30");
    await expect(statValue(page, "queue-retry")).toHaveText("5");

    await page.getByTestId("filter-state").selectOption({ label: "Waiting for retry" });

    await expect(page.getByTestId("job-row")).toHaveCount(5);
    await expect(page.getByTestId("jobs-total")).toContainText("5");
    for (const state of await page.getByTestId("job-state").allTextContents()) {
      expect(state.trim()).toBe("Waiting for retry");
    }
    // No pager once everything fits on one page.
    await expect(page.getByRole("navigation", { name: "pagination" })).toHaveCount(0);
  });

  test("an invalid custom period is refused on screen", async ({ page }) => {
    await seed(page, { loginAs: "global-admin", data: { jobs: [job(1)], jobKinds: KINDS } });
    await page.goto("/admin/jobs");

    await page.getByTestId("period-custom").click();
    await expect(page.getByTestId("period-custom")).toHaveAttribute("aria-pressed", "true");
    await page.getByTestId("period-from").fill("2026-01-02T10:00");
    await page.getByTestId("period-to").fill("2026-01-01T10:00");
    await page.getByTestId("period-apply").click();

    await expect(page.getByTestId("stats-error")).toHaveText("The start must be before the end.");
  });

  test("a music admin has neither the link nor access", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: {} });
    await page.goto("/music/queue");
    await expect(page.getByRole("link", { name: "Jobs" })).toHaveCount(0);
    await page.goto("/admin/jobs");
    await expect(page).not.toHaveURL(/\/admin\/jobs$/);
  });
});

// Change: add-jobs-console-history — the finished attempts, the per-kind breakdown and the
// cadence of the kinds that run on their own.
const SCHEDULED_KINDS = [
  {
    name: "session_reap",
    channel: "auth.session_reap",
    cancellable: true,
    schedules: [{ name: "session_reap_hourly", cron: "0 * * * *", timezone: "UTC", enabled: true }],
  },
  {
    name: "plans_reconcile",
    channel: "plans.reconcile",
    cancellable: true,
    schedules: [{ name: "plans_reconcile_daily", cron: "10 4 * * *", timezone: "UTC", enabled: true }],
  },
  {
    name: "usage_purge",
    channel: "analytics.usage_purge",
    cancellable: true,
    schedules: [{ name: "usage_purge_daily", cron: "40 3 * * *", timezone: "UTC", enabled: false }],
  },
  { name: "verification_email", channel: "identity.verification_email", cancellable: true },
];

const NOW = Date.now();
const MIN = 60 * 1000;

/** 30 hourly reaps, one abandoned, and a verification email that failed then succeeded. */
const ATTEMPTS: E2EAttempt[] = [
  ...Array.from({ length: 30 }, (_, i): E2EAttempt => ({
    jobId: `${String(i + 1).padStart(8, "0")}-bbbb-4bbb-8bbb-bbbbbbbbbbbb`,
    kind: "session_reap",
    outcome: "succeeded",
    finishedAtMs: NOW - (i + 1) * 30 * MIN,
    durationMs: 40,
  })),
  {
    jobId: "abandon0-cccc-4ccc-8ccc-cccccccccccc",
    kind: "session_reap",
    outcome: "abandoned",
    finishedAtMs: NOW - 5 * MIN,
  },
  {
    jobId: "email000-dddd-4ddd-8ddd-dddddddddddd",
    kind: "verification_email",
    outcome: "failed",
    attempt: 1,
    finishedAtMs: NOW - 3 * MIN,
    durationMs: 1200,
  },
  {
    jobId: "email000-dddd-4ddd-8ddd-dddddddddddd",
    kind: "verification_email",
    outcome: "succeeded",
    attempt: 2,
    finishedAtMs: NOW - 2 * MIN,
    durationMs: 800,
  },
];

test.describe("jobs history", () => {
  test("the history lists finished attempts newest first, pages and filters by outcome", async ({ page }) => {
    await seed(page, { loginAs: "global-admin", data: { jobKinds: SCHEDULED_KINDS, jobAttempts: ATTEMPTS } });
    await page.goto("/admin/jobs");
    await page.getByTestId("tab-history").click();

    const rows = page.getByTestId("history-row");
    await expect(rows).toHaveCount(25);
    await expect(page.getByTestId("history-total")).toHaveText("Finished attempts: 33");
    await expect(rows.first()).toContainText("verification_email");
    await expect(rows.first()).toContainText("Succeeded");
    await expect(rows.first()).toContainText("800 ms");
    await expect(rows.nth(1)).toContainText("Failed");
    await expect(rows.nth(2)).toContainText("Abandoned");
    await expect(rows.nth(2).getByTestId("history-duration")).toHaveText("—");

    await page.getByRole("button", { name: "Next →" }).click();
    await expect(rows).toHaveCount(8);
    await expect(page.getByText("26–33 of 33")).toBeVisible();

    await page.getByTestId("history-outcome").selectOption({ label: "Failed" });
    await expect(rows).toHaveCount(1);
    await expect(page.getByTestId("history-total")).toHaveText("Finished attempts: 1");
    await expect(rows.first()).toContainText("verification_email");
  });

  test("kinds carry their cadence in the filter, the breakdown and the rows", async ({ page }) => {
    await seed(page, {
      loginAs: "global-admin",
      data: {
        jobKinds: SCHEDULED_KINDS,
        jobAttempts: ATTEMPTS,
        jobs: [job(1, { kind: "session_reap", channel: "auth.session_reap" })],
      },
    });
    await page.goto("/admin/jobs");

    await expect(page.getByTestId("filter-kind").locator("option")).toHaveText([
      "Any",
      "session_reap — Scheduled · hourly at :00",
      "plans_reconcile — Scheduled · daily at 04:10 (UTC)",
      "usage_purge — Schedule paused",
      "verification_email — On demand",
    ]);
    await expect(page.getByTestId("job-row").first().getByTestId("cadence")).toHaveText("Scheduled · hourly at :00");

    const breakdown = page.getByTestId("breakdown-row");
    const row = (kind: string) => page.locator(`[data-testid="breakdown-row"][data-kind="${kind}"]`);
    await expect(breakdown).toHaveCount(4);
    await expect(row("session_reap").getByTestId("breakdown-completed")).toHaveText("30");
    await expect(row("verification_email").getByTestId("breakdown-completed")).toHaveText("1");
    // Scheduled, but nothing ran over the period: a zero row next to its cadence.
    await expect(row("plans_reconcile").getByTestId("breakdown-completed")).toHaveText("0");
    await expect(row("plans_reconcile").getByTestId("breakdown-last")).toHaveText("—");
    await expect(row("plans_reconcile").getByTestId("cadence")).toHaveText("Scheduled · daily at 04:10 (UTC)");
    await expect(row("usage_purge").getByTestId("cadence")).toHaveText("Schedule paused");
    // The breakdown adds up to the card.
    await expect(page.getByTestId("stat-period-completed").getByTestId("stat-value")).toHaveText("31");

    await page.getByTestId("tab-history").click();
    await expect(page.getByTestId("history-row").first().getByTestId("cadence")).toHaveText("On demand");
  });

  test("selecting a kind in the breakdown opens its history", async ({ page }) => {
    await seed(page, { loginAs: "global-admin", data: { jobKinds: SCHEDULED_KINDS, jobAttempts: ATTEMPTS } });
    await page.goto("/admin/jobs");

    await page
      .locator('[data-testid="breakdown-row"][data-kind="verification_email"]')
      .getByTestId("breakdown-open")
      .click();

    await expect(page.getByTestId("tab-history")).toHaveAttribute("aria-selected", "true");
    await expect(page.getByTestId("filter-kind")).toHaveValue("verification_email");
    await expect(page.getByTestId("history-row")).toHaveCount(2);
    await expect(page.getByTestId("breakdown-row")).toHaveCount(1);
    await expect(page.getByTestId("stat-period-completed").getByTestId("stat-value")).toHaveText("1");

    // Back on the queue, the kind filter still applies.
    await page.getByTestId("tab-queue").click();
    await expect(page.getByTestId("jobs-empty")).toBeVisible();
  });

  test("a server without the breakdown says so instead of showing zeros", async ({ page }) => {
    await seed(page, {
      loginAs: "global-admin",
      data: { jobKinds: SCHEDULED_KINDS, jobAttempts: ATTEMPTS, jobByKindMissing: true },
    });
    await page.goto("/admin/jobs");

    await expect(page.getByTestId("breakdown-unavailable")).toBeVisible();
    await expect(page.getByTestId("breakdown-row")).toHaveCount(0);
  });

  test("a failed history read shows a localised error, never a raw code", async ({ page }) => {
    await seed(page, {
      loginAs: "global-admin",
      data: {
        jobKinds: SCHEDULED_KINDS,
        jobAttempts: ATTEMPTS,
        fail: { adminListJobHistory: { code: 12, message: "unimplemented" } },
      },
    });
    await page.goto("/admin/jobs");
    await page.getByTestId("tab-history").click();

    await expect(page.getByTestId("history-error")).toBeVisible();
    await expect(page.locator("body")).not.toContainText("unimplemented");
  });

  test("a module admin still has neither the entry nor the history", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { jobKinds: SCHEDULED_KINDS, jobAttempts: ATTEMPTS } });
    await page.goto("/admin/jobs");
    await expect(page).not.toHaveURL(/\/admin\/jobs$/);
    await expect(page.getByTestId("tab-history")).toHaveCount(0);
  });
});
