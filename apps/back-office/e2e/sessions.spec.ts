import { test, expect, seed } from "./fixtures";

test.describe("active sessions", () => {
  test("lists sessions, flags this device, and revokes another one", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: {
        sessions: [
          { id: "sess-current", audience: "music" },
          { id: "sess-other", audience: "live" },
        ],
      },
    });
    await page.goto("/sessions");

    await expect(page.getByRole("heading", { name: "Active sessions" })).toBeVisible();
    // The token's `sid` matches "sess-current" → flagged as this device (no revoke button).
    await expect(page.getByText("This device")).toBeVisible();

    // Exactly one other session is revocable.
    const revoke = page.getByRole("button", { name: "Revoke", exact: true });
    await expect(revoke).toHaveCount(1);
    await revoke.click();

    // After revoking the other, only the current session remains — nothing left to revoke.
    await expect(page.getByRole("button", { name: "Revoke", exact: true })).toHaveCount(0);
    await expect(page.getByText("This device")).toBeVisible();
  });

  test("sign out everywhere ends the session and returns to sign-in", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: { sessions: [{ id: "sess-current", audience: "music" }] },
    });
    await page.goto("/sessions");

    await page.getByRole("button", { name: "Sign out everywhere" }).click();
    await expect(page).toHaveURL(/\/signin$/);
  });
});
