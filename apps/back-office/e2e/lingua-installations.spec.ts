import { test, expect, seed } from "./fixtures";
import type { E2EData } from "../src/lib/e2e-seam";

// Change: add-lingua-usage-counts (task 3.2). Drives the "Installations" section of the Lingua
// console against the gated fake seam (no backend): the funnel by installation day, latest first,
// with each step as a count and a share, the "not yet" cells, the capped mark, the Day / Week toggle
// summing the same days by week, the browser filter reaching both the funnel and the leaving
// reasons, the notes, the chart, and a failed read shown as a localised error.

const DAY = 24 * 60 * 60 * 1000;
/** The UTC day `daysAgo` days before today — days relative to the run's date. */
function dayAgo(daysAgo: number): string {
  return new Date(Date.now() - daysAgo * DAY).toISOString().slice(0, 10);
}
/** The Monday (UTC) of the week holding the day `daysAgo` days before today. */
function mondayDaysAgo(daysAgo: number): string {
  const d = new Date(Date.now() - daysAgo * DAY);
  d.setUTCDate(d.getUTCDate() - ((d.getUTCDay() + 6) % 7));
  return d.toISOString().slice(0, 10);
}
/** `day` moved by `n` days (UTC). */
function plusDays(day: string, n: number): string {
  return new Date(new Date(`${day}T00:00:00Z`).getTime() + n * DAY).toISOString().slice(0, 10);
}

const OLD_MONDAY = mondayDaysAgo(120); // every return window closed long ago
const OLD_THURSDAY = plusDays(OLD_MONDAY, 3); // the same week
const TWO_DAYS_AGO = dayAgo(2); // day 1 has closed; days 7–13 have not
const YESTERDAY = dayAgo(1); // day 1 is still open
const TODAY = dayAgo(0); // no return window has closed yet

const data: E2EData = {
  linguaFunnel: [
    // Summed by week, the spec's figures: 40 installations, 25 first pages analysed, 8 returns in days 7–13.
    {
      day: OLD_MONDAY,
      browser: "firefox",
      installed: 30,
      setUp: 24,
      firstAnalysis: 20,
      returnedDay1: 10,
      returnedDays7To13: 6,
      returnedDays30To59: 3,
    },
    {
      day: OLD_THURSDAY,
      browser: "chromium",
      installed: 10,
      setUp: 6,
      firstAnalysis: 5,
      returnedDay1: 2,
      returnedDays7To13: 2,
      returnedDays30To59: 1,
    },
    { day: TWO_DAYS_AGO, browser: "firefox", installed: 8, setUp: 6, firstAnalysis: 4, returnedDay1: 2 },
    { day: YESTERDAY, browser: "chromium", installed: 5, setUp: 3, firstAnalysis: 2, returnedDay1: 1 },
    { day: TODAY, browser: "safari", installed: 12, setUp: 9, firstAnalysis: 6, capped: true },
  ],
  linguaLeaveReasons: [
    { reason: "slow", browser: "firefox", count: 3 },
    { reason: "not_using", browser: "chromium", count: 5 },
    { reason: "unclear", browser: "firefox", count: 1 },
  ],
};

test.describe("lingua installations", () => {
  test("a lingua admin reads the funnel by installation day, latest first", async ({ page }) => {
    await seed(page, { loginAs: "lingua-admin", data });
    await page.goto("/lingua/overview");

    const section = page.getByTestId("installations");
    await expect(section.getByRole("heading", { name: "Installations", exact: true })).toBeVisible();
    await expect(section.getByTestId("grain-day")).toHaveAttribute("aria-pressed", "true");
    await expect(section.getByRole("columnheader", { name: "Installation day" })).toBeVisible();
    await expect(section.getByRole("heading", { name: "Installations per day" })).toBeVisible();

    const rows = section.getByTestId("funnel-row");
    await expect(rows).toHaveCount(5);
    expect(await rows.evaluateAll((trs) => trs.map((tr) => tr.getAttribute("data-period")))).toEqual([
      TODAY,
      YESTERDAY,
      TWO_DAYS_AGO,
      OLD_THURSDAY,
      OLD_MONDAY,
    ]);

    const row = (day: string) => section.locator(`[data-testid="funnel-row"][data-period="${day}"]`);
    const old = row(OLD_MONDAY);
    await expect(old.getByTestId("installed")).toHaveText("30");
    await expect(old.getByTestId("step-firstAnalysis")).toHaveText(/^20 \(67\s?%\)$/);
    await expect(old.getByTestId("step-returnedDays7To13")).toHaveText(/^6 \(20\s?%\)$/);
    await expect(old.getByTestId("step-returnedDays30To59")).toHaveText(/^3 \(10\s?%\)$/);
    await expect(old.getByTestId("capped")).toHaveCount(0);

    // Two days ago: day 1 has closed, days 7–13 have not.
    await expect(row(TWO_DAYS_AGO).getByTestId("step-returnedDay1")).toHaveText(/^2 \(25\s?%\)$/);
    await expect(row(TWO_DAYS_AGO).getByTestId("step-returnedDays7To13")).toHaveText("not yet");
    // Yesterday: day 1 is still open.
    await expect(row(YESTERDAY).getByTestId("step-returnedDay1")).toHaveText("not yet");

    // Today: the steps without a window show as they stand; every return reads "not yet".
    const today = row(TODAY);
    await expect(today.getByTestId("installed")).toHaveText("12");
    await expect(today.getByTestId("step-setUp")).toHaveText(/^9 \(75\s?%\)$/);
    await expect(today.getByTestId("step-returnedDay1")).toHaveText("not yet");
    await expect(today.getByTestId("step-returnedDays7To13")).toHaveText("not yet");
    await expect(today.getByTestId("step-returnedDays30To59")).toHaveText("not yet");
    await expect(today.getByTestId("capped")).toHaveText("capped");
    await expect(section.getByTestId("capped-note")).toBeVisible();

    // One series of installations per day, through the console's chart.
    await expect(section.locator("canvas")).toBeVisible();
    // The note on what is counted: steps are sent as they happen, filed by installation day
    // (owner, 2026-10-16) — nothing about a following week any more.
    const note = section.getByTestId("installations-note");
    await expect(note).toContainText("on Chromium, only those whose reader agreed");
    await expect(note).toContainText("each device each count as one installation");
    await expect(note).toContainText("A device sends each step as soon as it happens");
    await expect(note).toContainText("filed by installation day");
    await expect(note).toContainText("No figure is finer than a day and a browser family");
    await expect(note).not.toContainText("following week");
    await expect(note).not.toContainText("the week after");
    await expect(section.getByTestId("not-yet-note")).toContainText("day 1, 13 or 59 after the installation day");

    // The leaving reasons: all six, in the page's order, with their labels.
    const reasons = section.getByTestId("reason-row");
    await expect(reasons).toHaveCount(6);
    await expect(reasons.locator("th")).toHaveText([
      "I wasn't using it",
      "Too many words highlighted, or too few",
      "My language isn't there",
      "It slowed my browser down",
      "I didn't understand how to use it",
      "Another reason",
    ]);
    await expect(reasons.locator("td:last-child")).toHaveText(["5", "0", "0", "3", "1", "0"]);
  });

  test("the week view sums the same days by week, latest first", async ({ page }) => {
    await seed(page, { loginAs: "lingua-admin", data });
    await page.goto("/lingua/overview");
    const section = page.getByTestId("installations");
    await expect(section.getByTestId("funnel-row")).toHaveCount(5);

    await section.getByTestId("grain-week").click();

    await expect(section.getByTestId("grain-week")).toHaveAttribute("aria-pressed", "true");
    await expect(section.getByTestId("grain-day")).toHaveAttribute("aria-pressed", "false");
    await expect(section.getByRole("columnheader", { name: "Installation week" })).toBeVisible();
    await expect(section.getByRole("heading", { name: "Installations per week" })).toBeVisible();

    // The old week: its Monday and Thursday summed — the spec's 40, 25 (63 %) and 8 (20 %).
    const old = section.locator(`[data-testid="funnel-row"][data-period="${OLD_MONDAY}"]`);
    await expect(old.getByTestId("installed")).toHaveText("40");
    await expect(old.getByTestId("step-firstAnalysis")).toHaveText(/^25 \(63\s?%\)$/);
    await expect(old.getByTestId("step-returnedDays7To13")).toHaveText(/^8 \(20\s?%\)$/);
    await expect(old.getByTestId("step-returnedDays30To59")).toHaveText(/^4 \(10\s?%\)$/);

    // Rows are Mondays, latest first: this week's on top, its returns "not yet" until its last
    // day's windows close, and capped since today's day is.
    const periods = await section
      .getByTestId("funnel-row")
      .evaluateAll((trs) => trs.map((tr) => tr.getAttribute("data-period") ?? ""));
    expect(periods).toEqual([...periods].sort().reverse());
    expect(periods[0]).toBe(mondayDaysAgo(0));
    expect(periods.at(-1)).toBe(OLD_MONDAY);
    const current = section.locator(`[data-testid="funnel-row"][data-period="${mondayDaysAgo(0)}"]`);
    await expect(current.getByTestId("step-returnedDay1")).toHaveText("not yet");
    await expect(current.getByTestId("capped")).toHaveText("capped");

    // Back to days: the five rows again.
    await section.getByTestId("grain-day").click();
    await expect(section.getByTestId("funnel-row")).toHaveCount(5);
  });

  test("the browser filter applies to the funnel and the leaving reasons alike", async ({ page }) => {
    await seed(page, { loginAs: "lingua-admin", data });
    await page.goto("/lingua/overview");
    const section = page.getByTestId("installations");
    await expect(section.getByTestId("funnel-row")).toHaveCount(5);

    await section.getByTestId("browser").selectOption("firefox");

    // Only the Firefox days remain.
    const rows = section.getByTestId("funnel-row");
    await expect(rows).toHaveCount(2);
    await expect(rows.first()).toHaveAttribute("data-period", TWO_DAYS_AGO);
    const old = section.locator(`[data-testid="funnel-row"][data-period="${OLD_MONDAY}"]`);
    await expect(old.getByTestId("installed")).toHaveText("30");
    await expect(old.getByTestId("step-firstAnalysis")).toHaveText(/^20 \(67\s?%\)$/);
    await expect(section.getByTestId("reason-row").locator("td:last-child")).toHaveText(["0", "0", "0", "3", "1", "0"]);

    // The week view sums the filtered days only: the old week is Firefox's Monday alone.
    await section.getByTestId("grain-week").click();
    await expect(old.getByTestId("installed")).toHaveText("30");

    await section.getByTestId("browser").selectOption("");
    await expect(old.getByTestId("installed")).toHaveText("40");
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
    const today = section.locator(`[data-testid="funnel-row"][data-period="${TODAY}"]`);
    await expect(today.getByTestId("step-returnedDay1")).toHaveText("pas encore");
    await expect(today.getByTestId("capped")).toHaveText("plafonné");
    await expect(section.getByRole("columnheader", { name: "Jour d’installation" })).toBeVisible();
    await expect(section.getByTestId("grain-day")).toHaveText("Par jour");
    await expect(section.getByTestId("installations-note")).toContainText("dès qu’elle a lieu");
    await expect(section.getByTestId("reason-row").first()).toContainText("Je ne m’en servais pas");

    await section.getByTestId("grain-week").click();
    await expect(section.getByRole("columnheader", { name: "Semaine d’installation" })).toBeVisible();
  });
});
