import { test, expect, seed, sampleHit } from "./fixtures";

// Burn-down review mode: the moderator fixes the metadata of the score in front of
// them through the edit drawer and keeps chaining decisions, without a round-trip to
// the detail page.
test.describe("review mode", () => {
  const second = sampleHit({
    id: "22222222-2222-2222-2222-222222222222",
    title: "Gymnopédie",
    composer: "Erik Satie",
    level: "beginner",
  });

  const start = async (page: import("@playwright/test").Page) => {
    await seed(page, {
      loginAs: "moderator",
      data: { hits: [sampleHit(), second], counts: { pending: 2, accepted: 0, rejected: 0 } },
    });
    await page.goto("/music/review");
  };

  test("edits the current score's metadata through the drawer, then advances", async ({ page }) => {
    await start(page);

    // The page itself only shows the read-only summary — no form until asked.
    await expect(page.locator("dialog.drawer")).toHaveCount(0);
    await expect(page.locator(".preview dl.meta")).toContainText("Clair de Lune");

    await page.getByRole("button", { name: "Edit (E)" }).click();
    const drawer = page.locator("dialog.drawer");
    await expect(drawer).toBeVisible();

    const title = drawer.getByLabel("Title");
    await expect(title).toHaveValue("Clair de Lune");
    await title.fill("Clair de lune (rev.)");
    await drawer.getByRole("button", { name: "Save changes" }).click();

    // A successful save closes the drawer and refreshes the summary; the queue has
    // NOT moved on.
    await expect(drawer).toHaveCount(0);
    await expect(page.getByRole("heading", { name: "Clair de lune (rev.)", level: 2 })).toBeVisible();
    await expect(page.locator(".preview dl.meta")).toContainText("Clair de lune (rev.)");
    await expect(page.getByText("0 reviewed")).toBeVisible();

    // Accepting advances, and the drawer re-seeds from the next score.
    await page.getByRole("button", { name: "Accept (A)" }).click();
    await expect(page.getByText("1 reviewed")).toBeVisible();
    await page.getByRole("button", { name: "Edit (E)" }).click();
    await expect(drawer.getByLabel("Title")).toHaveValue("Gymnopédie");
  });

  test("does not decide the score while the drawer has the keyboard", async ({ page }) => {
    await start(page);
    await page.getByRole("button", { name: "Edit (E)" }).click();
    const drawer = page.locator("dialog.drawer");

    // "a" (accept), "r" (reject), "p" (re-queue) and "s" (skip) are all plain letters
    // a moderator types into the composer field.
    const composer = drawer.getByLabel("Composer");
    await composer.fill("");
    await composer.pressSequentially("Rameau, arr. Sapp");
    await expect(composer).toHaveValue("Rameau, arr. Sapp");
    await expect(page.getByText("0 reviewed")).toBeVisible();

    // The level <select> takes keystrokes too (type-ahead) — still not a decision.
    await drawer.getByLabel("Level").press("a");
    await expect(page.getByText("0 reviewed")).toBeVisible();

    // Escape closes the drawer instead of leaving review mode, and drops the edit.
    await page.keyboard.press("Escape");
    await expect(drawer).toHaveCount(0);
    await expect(page).toHaveURL(/\/music\/review$/);
    await page.getByRole("button", { name: "Edit (E)" }).click();
    await expect(drawer.getByLabel("Composer")).toHaveValue("Claude Debussy");
  });
});
