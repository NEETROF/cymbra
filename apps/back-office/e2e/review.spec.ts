import { test, expect, seed, sampleHit } from "./fixtures";

// Burn-down review mode: the moderator fixes the metadata of the score in front of
// them and keeps chaining decisions, without a round-trip to the detail page.
test.describe("review mode", () => {
  const second = sampleHit({
    id: "22222222-2222-2222-2222-222222222222",
    title: "Gymnopédie",
    composer: "Erik Satie",
    level: "beginner",
  });

  test("edits the current score's metadata in place, then advances", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: { hits: [sampleHit(), second], counts: { pending: 2, accepted: 0, rejected: 0 } },
    });
    await page.goto("/music/review");

    const form = page.locator("form.edit");
    const title = form.getByLabel("Title");
    await expect(title).toHaveValue("Clair de Lune");

    await title.fill("Clair de lune (rev.)");
    await form.getByRole("button", { name: "Save changes" }).click();

    // The header re-reads the row from the backend; the queue has NOT moved on.
    await expect(page.getByRole("heading", { name: "Clair de lune (rev.)", level: 2 })).toBeVisible();
    await expect(page.getByText("0 reviewed")).toBeVisible();

    // Accepting advances, and the form re-seeds from the next score.
    await page.getByRole("button", { name: "Accept (A)" }).click();
    await expect(title).toHaveValue("Gymnopédie");
    await expect(page.getByText("1 reviewed")).toBeVisible();
  });

  test("does not decide the score when a shortcut key is typed into the form", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: { hits: [sampleHit(), second], counts: { pending: 2, accepted: 0, rejected: 0 } },
    });
    await page.goto("/music/review");

    const form = page.locator("form.edit");
    // "a" (accept), "r" (reject), "p" (re-queue) and "s" (skip) are all plain letters
    // a moderator types into the composer field.
    await form.getByLabel("Composer").fill("");
    await form.getByLabel("Composer").pressSequentially("Rameau, arr. Sapp");
    await expect(form.getByLabel("Composer")).toHaveValue("Rameau, arr. Sapp");
    await expect(page.getByText("0 reviewed")).toBeVisible();

    // The level <select> takes keystrokes too (type-ahead) — still not a decision.
    await form.getByLabel("Level").press("a");
    await expect(page.getByText("0 reviewed")).toBeVisible();
  });
});
