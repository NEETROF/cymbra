import { test, expect, seed } from "./fixtures";
import type { E2EData } from "../src/lib/e2e-seam";

// Change: add-lingua-usage-counts (task 3.2). Drives the "Installations" section of the Lingua
// console against the gated fake seam (no backend): the funnel by installation week with each step
// as a count and a share, the "not yet" cells, the capped mark, the browser filter reaching both the
// funnel and the leaving reasons, the note, the chart, and a failed read shown as a localised error.

const DAY = 24 * 60 * 60 * 1000;
/** The Monday (UTC) of the week `daysAgo` days before today — weeks relative to the run's date. */
function mondayDaysAgo(daysAgo: number): string {
  const d = new Date(Date.now() - daysAgo * DAY);
  d.setUTCDate(d.getUTCDate() - ((d.getUTCDay() + 6) % 7));
  return d.toISOString().slice(0, 10);
}

const OLD = mondayDaysAgo(120); // every return window closed long ago
const CURRENT = mondayDaysAgo(0); // this week: no return window has closed yet

const data: E2EData = {
  linguaFunnel: [
    // The spec's week: 40 installations, 25 first pages analysed, 8 returns in days 7–13.
    {
      week: OLD,
      browser: "firefox",
      installed: 30,
      setUp: 24,
      firstAnalysis: 20,
      returnedDay1: 10,
      returnedDays7To13: 6,
      returnedDays30To59: 3,
    },
    {
      week: OLD,
      browser: "chromium",
      installed: 10,
      setUp: 6,
      firstAnalysis: 5,
      returnedDay1: 2,
      returnedDays7To13: 2,
      returnedDays30To59: 1,
    },
    { week: CURRENT, browser: "safari", installed: 12, setUp: 9, firstAnalysis: 6, returnedDay1: 1, capped: true },
  ],
  linguaLeaveReasons: [
    { reason: "slow", browser: "firefox", count: 3 },
    { reason: "not_using", browser: "chromium", count: 5 },
    { reason: "unclear", browser: "firefox", count: 1 },
  ],
};

test.describe("lingua installations", () => {
  test("a lingua admin reads the funnel by installation week", async ({ page }) => {
    await seed(page, { loginAs: "lingua-admin", data });
    await page.goto("/lingua/overview");

    const section = page.getByTestId("installations");
    await expect(section.getByRole("heading", { name: "Installations", exact: true })).toBeVisible();

    const old = section.locator(`[data-testid="funnel-row"][data-week="${OLD}"]`);
    await expect(old.getByTestId("installed")).toHaveText("40");
    await expect(old.getByTestId("step-firstAnalysis")).toHaveText(/^25 \(63\s?%\)$/);
    await expect(old.getByTestId("step-returnedDays7To13")).toHaveText(/^8 \(20\s?%\)$/);
    await expect(old.getByTestId("step-returnedDays30To59")).toHaveText(/^4 \(10\s?%\)$/);
    await expect(old.getByTestId("capped")).toHaveCount(0);

    // This week: the steps without a window show as they stand; every return reads "not yet".
    const current = section.locator(`[data-testid="funnel-row"][data-week="${CURRENT}"]`);
    await expect(current.getByTestId("installed")).toHaveText("12");
    await expect(current.getByTestId("step-setUp")).toHaveText(/^9 \(75\s?%\)$/);
    await expect(current.getByTestId("step-returnedDay1")).toHaveText("not yet");
    await expect(current.getByTestId("step-returnedDays7To13")).toHaveText("not yet");
    await expect(current.getByTestId("step-returnedDays30To59")).toHaveText("not yet");
    await expect(current.getByTestId("capped")).toHaveText("capped");
    await expect(section.getByTestId("capped-note")).toBeVisible();

    // One series of installations per week, through the console's chart.
    await expect(section.locator("canvas")).toBeVisible();
    // The note on what is counted.
    await expect(section.getByTestId("installations-note")).toContainText(
      "on Chromium, only those whose reader agreed",
    );
    await expect(section.getByTestId("installations-note")).toContainText("each device each count as one installation");

    // The leaving reasons: all six, in the page's order, with their labels.
    const reasons = section.getByTestId("reason-row");
    await expect(reasons).toHaveCount(6);
    await expect(reasons.locator("td:first-child")).toHaveText([
      "I wasn't using it",
      "Too many words highlighted, or too few",
      "My language isn't there",
      "It slowed my browser down",
      "I didn't understand how to use it",
      "Another reason",
    ]);
    await expect(reasons.locator("td:last-child")).toHaveText(["5", "0", "0", "3", "1", "0"]);
  });

  test("the browser filter applies to the funnel and the leaving reasons alike", async ({ page }) => {
    await seed(page, { loginAs: "lingua-admin", data });
    await page.goto("/lingua/overview");
    const section = page.getByTestId("installations");
    await expect(section.getByTestId("funnel-row")).toHaveCount(2);

    await section.getByTestId("browser").selectOption("firefox");

    const old = section.locator(`[data-testid="funnel-row"][data-week="${OLD}"]`);
    await expect(old.getByTestId("installed")).toHaveText("30");
    await expect(old.getByTestId("step-firstAnalysis")).toHaveText(/^20 \(67\s?%\)$/);
    // The Safari-only week is gone.
    await expect(section.getByTestId("funnel-row")).toHaveCount(1);
    await expect(section.getByTestId("reason-row").locator("td:last-child")).toHaveText(["0", "0", "0", "3", "1", "0"]);

    await section.getByTestId("browser").selectOption("");
    await expect(section.getByTestId("funnel-row")).toHaveCount(2);
  });

  test("a failed funnel read is a localised error; the rest stands", async ({ page }) => {
    await seed(page, {
      loginAs: "lingua-admin",
      data: {
        ...data,
        linguaUsage: { activeAccounts: 42 },
        fail: { adminGetUsageFunnel: { code: 14, message: "[unavailable] lingua database down" } },
      },
    });
    await page.goto("/lingua/overview");

    const section = page.getByTestId("installations");
    await expect(section.getByTestId("funnel-error")).toBeVisible();
    await expect(page.locator("body")).not.toContainText("database down");
    await expect(page.locator("body")).not.toContainText("unavailable]");
    // The leaving reasons and the rest of the console still read.
    await expect(section.getByTestId("reason-row")).toHaveCount(6);
    await expect(page.getByTestId("active-accounts")).toContainText("42");
  });

  test("the section reads in French", async ({ page }) => {
    await seed(page, { loginAs: "lingua-admin", data });
    await page.addInitScript(() => localStorage.setItem("cymbra.bo.locale", "fr"));
    await page.goto("/lingua/overview");

    const section = page.getByTestId("installations");
    const current = section.locator(`[data-testid="funnel-row"][data-week="${CURRENT}"]`);
    await expect(current.getByTestId("step-returnedDay1")).toHaveText("pas encore");
    await expect(current.getByTestId("capped")).toHaveText("plafonné");
    await expect(section.getByTestId("reason-row").first()).toContainText("Je ne m’en servais pas");
  });
});
