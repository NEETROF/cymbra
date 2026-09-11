import { test, expect, seed } from "./fixtures";

// Change: add-lingua-back-office. Drives the "Lingua" ops console in a real browser
// against the gated fake seam (no backend): the aggregate tiles, the per-day series,
// the per-language breakdown and the read-only pack registry — plus the scope gate
// (only a `lingua`-scope admin reaches it; a moderator and a music-only admin do not).

test.describe("lingua ops console", () => {
  test("a lingua admin sees the tiles, series and pack registry", async ({ page }) => {
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
        linguaPacks: [
          {
            studied: "en",
            native: "fr",
            packVersion: "0.0.0-testdata",
            analyzerVersion: "1.0.0",
            builtAt: "2026-09-11",
            sizeBytes: 747,
            notice: "AGID; wordfreq; kaikki.",
          },
        ],
      },
    });
    await page.goto("/lingua");

    await expect(page.getByRole("heading", { name: "Lingua" })).toBeVisible();
    // The aggregate tiles.
    await expect(page.getByTestId("active-accounts")).toContainText("42");
    await expect(page.getByTestId("words-learned")).toContainText("120");
    await expect(page.getByTestId("reviews")).toContainText("300");
    // The per-day series render on canvas.
    await expect(page.locator("canvas").first()).toBeVisible();
    // The per-language breakdown and the read-only pack registry.
    await expect(page.getByTestId("language-row")).toContainText("en");
    await expect(page.getByTestId("pack-row")).toContainText("0.0.0-testdata");
    // The "synced accounts" bias is stated on screen.
    await expect(page.getByText(/only accounts that sync/i)).toBeVisible();
  });

  test("a non-admin moderator has neither the link nor access", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: {} });
    await page.goto("/music/queue");
    await expect(page.getByRole("link", { name: "Lingua" })).toHaveCount(0);
    await page.goto("/lingua");
    await expect(page).not.toHaveURL(/\/lingua$/);
  });

  test("a music-only admin is redirected (wrong scope)", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: {} });
    await page.goto("/music/queue");
    // The nav link is scope-gated: a music admin never sees it.
    await expect(page.getByRole("link", { name: "Lingua" })).toHaveCount(0);
    await page.goto("/lingua");
    await expect(page).not.toHaveURL(/\/lingua$/);
  });
});
