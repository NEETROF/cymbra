import { test, expect, seed } from "./fixtures";

// Change: add-lingua-back-office. Drives the "Lingua" ops console in a real browser
// against the gated fake seam (no backend): the aggregate tiles, the per-day series,
// the per-language breakdown and the studied-language filter it feeds — plus the scope gate
// (only a `lingua`-scope admin reaches it; a moderator and a music-only admin do not).

test.describe("lingua ops console", () => {
  test("a lingua admin sees the tiles, series and language breakdown", async ({ page }) => {
    await seed(page, {
      loginAs: "lingua-admin",
      data: {
        linguaUsage: {
          activeAccounts: 42,
          wordsLearned: 120,
          reviews: 300,
          exposures: 900,
          byLanguage: [{ language: "en", activeAccounts: 42, wordsLearned: 120, reviews: 300 }],
        },
      },
    });
    await page.goto("/lingua/overview");

    await expect(page.getByRole("heading", { name: "Lingua" })).toBeVisible();
    // The aggregate tiles.
    await expect(page.getByTestId("active-accounts")).toContainText("42");
    await expect(page.getByTestId("words-learned")).toContainText("120");
    await expect(page.getByTestId("reviews")).toContainText("300");
    // The per-day series render on canvas.
    await expect(page.locator("canvas").first()).toBeVisible();
    // The per-language breakdown, and the studied-language filter drawn from it (change:
    // remove-lingua-pack-registry — the filter used to list the registered packs).
    await expect(page.getByTestId("language-row")).toContainText("en");
    await expect(page.getByTestId("language").locator("option")).toHaveText([/./, "en"]);
    // The pack registry is gone from the screen.
    await expect(page.getByTestId("pack-row")).toHaveCount(0);
    // The "synced accounts" bias is stated on screen.
    await expect(page.getByText(/only accounts that sync/i)).toBeVisible();
  });

  test("a non-admin moderator has neither the link nor access", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: {} });
    await page.goto("/music/queue");
    await expect(page.getByTestId("nav-section-lingua")).toHaveCount(0);
    await expect(page.getByRole("link", { name: "Overview" })).toHaveCount(0);
    await page.goto("/lingua/overview");
    await expect(page).not.toHaveURL(/\/lingua\/overview$/);
  });

  test("a music-only admin is redirected (wrong scope)", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: {} });
    await page.goto("/music/queue");
    // The Lingua section is scope-gated: a music admin never sees it.
    await expect(page.getByTestId("nav-section-lingua")).toHaveCount(0);
    await expect(page.getByRole("link", { name: "Overview" })).toHaveCount(0);
    await page.goto("/lingua/overview");
    await expect(page).not.toHaveURL(/\/lingua\/overview$/);
  });
});
