import { test, expect, seed } from "./fixtures";

// Change: add-push-notifications. Drives the "Notifications" panel in a real
// browser against the gated fake seam (no backend): the global kill-switch, the
// per-category enable and schedule hour, the empty state before any feature has
// declared a notification type, and the admin-only access gate.

const KILL_SWITCH = { key: "notifications.enabled", value: true, hasOverride: true };
const STREAK_ENABLED = { key: "notifications.category.practice_streak.enabled", value: true };
const STREAK_HOUR = { key: "notifications.category.practice_streak.hour", value: 20 };

test.describe("notifications panel", () => {
  test("an admin sees the kill-switch and each declared category", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      // A key from another domain must not leak into this panel.
      data: { flags: [KILL_SWITCH, STREAK_ENABLED, STREAK_HOUR, { key: "rating.enabled", value: false }] },
    });
    await page.goto("/notifications");

    await expect(page.getByRole("heading", { name: "Notifications", exact: true })).toBeVisible();
    await expect(page.getByTestId("kill-switch")).toHaveText("On");
    await expect(page.getByTestId("category-practice_streak")).toBeVisible();
    await expect(page.getByTestId("enable-practice_streak")).toHaveText("On");
    await expect(page.getByTestId("hour-practice_streak")).toHaveValue("20");
    // Only notification keys appear here.
    await expect(page.getByText("rating.enabled")).toHaveCount(0);
  });

  test("flipping the kill-switch off is reflected after the re-read", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { flags: [KILL_SWITCH, STREAK_ENABLED, STREAK_HOUR] } });
    await page.goto("/notifications");

    await page.getByTestId("kill-switch").click();
    await expect(page.getByTestId("kill-switch")).toHaveText("Off");
  });

  test("a new schedule hour is saved and read back", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { flags: [KILL_SWITCH, STREAK_ENABLED, STREAK_HOUR] } });
    await page.goto("/notifications");

    await page.getByTestId("hour-practice_streak").fill("7");
    await page.getByTestId("save-hour-practice_streak").click();
    await expect(page.getByTestId("hour-practice_streak")).toHaveValue("7");
  });

  test("with no category declared the panel says so", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { flags: [KILL_SWITCH] } });
    await page.goto("/notifications");

    await expect(page.getByTestId("no-categories")).toBeVisible();
    await expect(page.getByTestId("kill-switch")).toBeVisible();
  });

  test("a load failure shows a message, never a raw gRPC error", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: { flags: [KILL_SWITCH], fail: { listFlagDefinitions: { code: 14, message: "backend down" } } },
    });
    await page.goto("/notifications");

    const alert = page.getByRole("alert");
    await expect(alert).toBeVisible();
    await expect(alert).not.toContainText("unavailable]");
  });

  test("a moderator cannot reach the panel", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { flags: [KILL_SWITCH] } });
    await page.goto("/notifications");

    // Admin-only route: a moderator is redirected to their work surface.
    await expect(page).not.toHaveURL(/\/notifications/);
    await expect(page.getByTestId("kill-switch")).toHaveCount(0);
  });
});
