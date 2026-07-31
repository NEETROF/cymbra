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

  test("a music-only admin sees no scope selector (single scope)", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/roles");

    await expect(page.getByRole("heading", { name: "Roles" })).toBeVisible();
    // Only `music` is authorized → the scope picker is hidden.
    await expect(page.getByRole("combobox", { name: "Scope" })).toHaveCount(0);
  });

  test("a global admin switches scope and grants a role in the selected scope", async ({ page }) => {
    const tara = {
      userId: "u-tara",
      handle: "tara",
      displayName: "Tara",
      rolesByScope: { global: [] as string[], music: [], live: [] },
    };
    await seed(page, { loginAs: "global-admin", data: { accounts: [tara] } });
    await page.goto("/roles");

    // A global admin gets the scope selector (global/music/live).
    const scope = page.getByRole("combobox", { name: "Scope" });
    await expect(scope).toBeVisible();
    await scope.selectOption("live");

    // Granting now targets the `live` scope; the row reflects it after the re-list.
    await expect(page.getByRole("button", { name: "Grant moderator" })).toBeVisible();
    await page.getByRole("button", { name: "Grant moderator" }).click();
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

  test("the audit history shows the acting admin's handle, not the raw id", async ({ page }) => {
    const grants = [
      {
        targetUserId: "u-ada",
        scope: "music",
        role: "moderator",
        action: "grant",
        actingAdmin: "019f60be-6cd9-74e2-a600-9893bd2aaa3a",
        actingAdminHandle: "bossadmin",
        at: 1_700_000_000,
      },
    ];
    await seed(page, { loginAs: "admin", data: { accounts: [ada], grants } });
    await page.goto("/roles");

    await page.getByRole("button", { name: "History" }).click();

    await expect(page.getByText("bossadmin")).toBeVisible();
    await expect(page.locator("body")).not.toContainText("019f60be-6cd9");
  });

  test("a failed admin session-revoke surfaces an error instead of failing silently", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: {
        accounts: [ada],
        // Connect UNAVAILABLE = 14.
        fail: { revokeAccountSessions: { code: 14, message: "[unavailable] backend down" } },
      },
    });
    page.on("dialog", (d) => d.accept()); // confirm the destructive action
    await page.goto("/roles");

    await page.getByRole("button", { name: "Revoke sessions" }).click();

    // The failure is shown (not swallowed into the sessions store), and no raw code leaks.
    await expect(page.getByRole("alert")).toHaveText("Service unavailable. Try again.");
    await expect(page.locator("body")).not.toContainText("backend down");
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
