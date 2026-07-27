import { test, expect, seed, sampleHit } from "./fixtures";

const grantRow = {
  at: 1_700_000_000, // seconds
  action: "grant",
  scope: "music",
  role: "moderator",
  actingAdmin: "admin-1",
};

test.describe("roles (admin only)", () => {
  test("an admin sees the Roles nav and loads an account's audit history", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { hits: [sampleHit()], grants: [grantRow] } });
    await page.goto("/roles");

    await expect(page.getByRole("heading", { name: "Roles" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Roles" })).toBeVisible();

    await page.getByPlaceholder("target user id (UUID)").fill("00000000-0000-0000-0000-000000000000");
    await page.getByRole("button", { name: "Load history" }).click();

    const row = page.locator("tbody tr").first();
    await expect(row).toContainText("grant");
    await expect(row).toContainText("music");
    await expect(row).toContainText("moderator");
    await expect(row).toContainText("admin-1");
  });

  test("granting a role succeeds without surfacing an error", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { grants: [] } });
    await page.goto("/roles");

    await page.getByPlaceholder("target user id (UUID)").fill("00000000-0000-0000-0000-000000000000");
    await page.getByRole("button", { name: "Grant", exact: true }).click();

    await expect(page.getByRole("alert")).toHaveCount(0);
  });
});
