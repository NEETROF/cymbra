import { test, expect, seed } from "./fixtures";
import type { E2EJob } from "../src/lib/e2e-seam";

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

    await expect(page).toHaveURL(/\/jobs$/);
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
    await page.goto("/jobs");
    await expect(page.getByTestId("job-row")).toHaveCount(3);
    await expect(statValue(page, "period-cancelled")).toHaveText("2");

    const target = page.getByTestId("job-row").filter({ hasText: "render_preview" });
    await target.getByTestId("job-cancel").click();
    const dialog = page.getByRole("dialog");
    await expect(dialog).toContainText("render_preview");
    await expect(dialog).toContainText("00000002");
    await dialog.getByRole("button", { name: "Confirm" }).click();

    await expect(page.getByText("Job cancelled.")).toBeVisible();
    await expect(dialog).toHaveCount(0);
    await expect(page.getByTestId("job-row")).toHaveCount(2);
    await expect(page.getByTestId("job-row").filter({ hasText: "render_preview" })).toHaveCount(0);
    await expect(statValue(page, "period-cancelled")).toHaveText("3");
    await expect(statValue(page, "queue-total")).toHaveText("2");
  });

  test("dismissing the confirmation cancels nothing", async ({ page }) => {
    await seed(page, { loginAs: "global-admin", data: { jobs: [job(1)], jobKinds: KINDS } });
    await page.goto("/jobs");

    await page.getByTestId("job-cancel").click();
    await page.getByRole("dialog").getByRole("button", { name: "Cancel" }).click();

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
    await page.goto("/jobs");

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
    await page.goto("/jobs");
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
    await page.goto("/jobs");

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
    await page.goto("/jobs");
    await expect(page).not.toHaveURL(/\/jobs$/);
  });
});
