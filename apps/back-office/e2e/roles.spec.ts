import { test, expect, seed } from "./fixtures";

const ada = { userId: "u-ada", handle: "ada", displayName: "Ada Lovelace", roles: [] as string[] };
const bob = { userId: "u-bob", handle: "bob", displayName: "Bob Ross", roles: ["moderator"] };

test.describe("roles directory (admin only)", () => {
  test("an admin sees the account directory and grants a role on a row", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/roles");

    await expect(page.getByRole("heading", { name: "Roles" })).toBeVisible();
    await expect(page.getByText("ada", { exact: true })).toBeVisible();
    // Before: grantable, not yet held.
    await expect(page.getByRole("button", { name: "Grant moderator" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Revoke moderator" })).toHaveCount(0);

    await page.getByRole("button", { name: "Grant moderator" }).click();

    // After the grant + re-list, the row reflects the role (toggle flips to revoke).
    await expect(page.getByRole("button", { name: "Revoke moderator" })).toBeVisible();
  });

  test("filtering by handle narrows the directory", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada, bob] } });
    await page.goto("/roles");

    await expect(page.getByText("ada", { exact: true })).toBeVisible();
    await expect(page.getByText("bob", { exact: true })).toBeVisible();

    await page.getByPlaceholder("filter by handle or email").fill("ada");
    await page.getByRole("button", { name: "Search" }).click();

    await expect(page.getByText("ada", { exact: true })).toBeVisible();
    await expect(page.getByText("bob", { exact: true })).toHaveCount(0);
  });

  test("an empty result shows a friendly message, not a raw code", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/roles");

    await page.getByPlaceholder("filter by handle or email").fill("zzz-nobody");
    await page.getByRole("button", { name: "Search" }).click();

    await expect(page.getByText("No accounts.")).toBeVisible();
    await expect(page.locator("body")).not.toContainText("unavailable");
    await expect(page.locator("body")).not.toContainText("[");
  });
});
