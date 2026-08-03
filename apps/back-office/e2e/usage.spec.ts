import { test, expect, seed } from "./fixtures";

// Change: add-feature-usage-analytics. Drives the "Usage" analytics console in a
// real browser against the gated fake seam (no backend): the distinct-users
// summary, the action breakdown, and the data-driven action filter — plus the
// admin-only access gate.

test.describe("usage analytics console", () => {
  test("an admin sees the distinct-users summary and action breakdown", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: {
        usageSummary: {
          totalUsers: 42,
          byPlatform: [
            { platform: "ios", users: 25 },
            { platform: "android", users: 17 },
          ],
          byDeviceClass: [{ deviceClass: "phone", users: 42 }],
        },
        usageActions: ["auth_sign_in", "play_start"],
        usageBreakdown: [{ action: "play_start", variant: "", events: 300 }],
      },
    });
    await page.goto("/usage");

    await expect(page.getByRole("heading", { name: "Feature usage" })).toBeVisible();
    // Exact distinct-user total for the period.
    await expect(page.getByTestId("total-users")).toContainText("42");
    // Split by platform.
    await expect(page.getByTestId("platform-row")).toHaveCount(2);
    await expect(page.getByTestId("platform-row").first()).toContainText("ios");
    // Action breakdown.
    await expect(page.getByTestId("action-row")).toContainText("play_start");

    // The action filter is data-driven from the aggregates.
    await expect(page.getByTestId("action").locator("option")).toContainText([
      "auth_sign_in",
      "play_start",
    ]);
  });

  test("a non-admin moderator cannot reach the screen", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: {} });
    await page.goto("/music/queue");
    await expect(page.getByRole("link", { name: "Usage" })).toHaveCount(0);
    await page.goto("/usage");
    await expect(page).not.toHaveURL(/\/usage$/);
  });
});
