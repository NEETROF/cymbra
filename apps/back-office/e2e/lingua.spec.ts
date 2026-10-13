import { test, expect, seed } from "./fixtures";

// Change: add-lingua-back-office. Drives the "Lingua" ops console in a real browser
// against the gated fake seam (no backend): the aggregate tiles, the per-day series,
// the per-language breakdown and the studied-language filter it feeds — plus the scope gate
// (only a `lingua`-scope admin reaches it; a moderator and a music-only admin do not).

test.describe("lingua ops console", () => {
  test("a lingua admin sees online translation usage and its estimated cost", async ({ page }) => {
    // change: add-lingua-online-translation — daily totals, no account, no text.
    await seed(page, {
      loginAs: "lingua-admin",
      data: {
        onlineTranslationUsage: [
          { day: "2026-10-10", provider: "gemini", pair: "es-fr", answered: 120, failed: 2, costMicroUsd: 16_800 },
          { day: "2026-10-11", provider: "anthropic", pair: "en-fr", answered: 3, costMicroUsd: 300 },
          { day: "2026-10-11", provider: "none", pair: "en-fr", answered: 0, limited: 4, costMicroUsd: 0 },
        ],
      },
    });
    await page.goto("/lingua/overview");
    const panel = page.getByTestId("online-translation");
    await expect(panel.getByTestId("online-answered")).toContainText("123");
    await expect(panel.getByTestId("online-limited")).toContainText("4");
    await expect(panel.getByTestId("online-cost")).toContainText("$0.0171");
    await expect(panel.getByTestId("online-row").first()).toContainText("gemini");
    await expect(panel.getByTestId("online-row").first()).toContainText("es-fr");
    await expect(panel).toContainText("refused before any provider");
  });

  test("a lingua admin sees the tiles, series and language breakdown", async ({ page }) => {
    await seed(page, {
      loginAs: "lingua-admin",
      data: {
        linguaUsage: {
          activeAccounts: 42,
          wordsLearned: 120,
          reviews: 300,
          wordsRead: 1000,
          newWordsSeen: 50,
          byLanguage: [
            { language: "en", activeAccounts: 42, wordsLearned: 120, reviews: 300, wordsRead: 1000, newWordsSeen: 50 },
          ],
        },
      },
    });
    await page.goto("/lingua/overview");

    await expect(page.getByRole("heading", { name: "Lingua" })).toBeVisible();
    // The aggregate tiles.
    await expect(page.getByTestId("active-accounts")).toContainText("42");
    await expect(page.getByTestId("words-learned")).toContainText("120");
    await expect(page.getByTestId("reviews")).toContainText("300");
    // The reading figures: words read, new words among them, and the comprehension they give.
    await expect(page.getByTestId("words-read")).toContainText(/1[,\s\u202f]?000/);
    await expect(page.getByTestId("new-words-seen")).toContainText("50");
    await expect(page.getByTestId("comprehension")).toContainText("95");
    await expect(page.getByTestId("language-row")).toContainText("95");
    // The per-day series render on canvas.
    await expect(page.locator("canvas").first()).toBeVisible();
    // The per-language breakdown, and the studied-language filter drawn from it (change:
    // remove-lingua-pack-registry — the filter used to list the registered packs).
    // Named in the console's language, not as a code (change: add-admin-lingua-language-labels).
    await expect(page.getByTestId("language-row")).toContainText("English");
    await expect(page.getByTestId("language").locator("option")).toHaveText([/./, "English"]);
    // The pack registry is gone from the screen.
    await expect(page.getByTestId("pack-row")).toHaveCount(0);
    // The "synced accounts" bias is stated on screen.
    await expect(page.getByText(/only accounts that sync/i)).toBeVisible();
  });

  test("comprehension is unavailable while nothing was read", async ({ page }) => {
    await seed(page, {
      loginAs: "lingua-admin",
      data: { linguaUsage: { activeAccounts: 1, wordsLearned: 2, reviews: 3 } },
    });
    await page.goto("/lingua/overview");
    await expect(page.getByTestId("comprehension")).toContainText("—");
    await expect(page.getByTestId("comprehension")).not.toContainText("%");
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
