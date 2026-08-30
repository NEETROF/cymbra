import { test, expect, seed } from "./fixtures";
import type { E2EData } from "../src/lib/e2e-seam";

// Private-score takedown (change: add-private-score-catalog): drives the
// music-admin screen in a real browser against the gated fake seam (no backend).
// Covers the three things the requirement is about — the lookup, the refusal
// without a reason, and a successful, confirmed removal.

const userScores: NonNullable<E2EData["userScores"]> = [
  {
    id: "s1",
    ownerId: "u-ada",
    title: "Reported Piece",
    composer: "Anon",
    sizeBytes: "1024",
    createdAt: "1760000000",
    rightsBasis: "own_work",
  },
  {
    id: "s2",
    ownerId: "u-bob",
    title: "Another Score",
    composer: "Someone",
    sizeBytes: "2048",
    createdAt: "1760000000",
    rightsBasis: "private_use",
  },
];

test.describe("private-score takedown", () => {
  test("a music admin looks a reported score up by title", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { userScores } });
    await page.goto("/takedowns");

    // The search is refused client-side until a criterion is given.
    await expect(page.getByRole("button", { name: "Search" })).toBeDisabled();

    await page.getByLabel("Title contains").fill("reported");
    await page.getByRole("button", { name: "Search" }).click();

    await expect(page.getByText("Reported Piece")).toBeVisible();
    await expect(page.getByText("Another Score")).toHaveCount(0);
  });

  test("removal needs an explicit confirmation and a reason", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { userScores } });
    await page.goto("/takedowns");
    await page.getByLabel("Owner id").fill("u-ada");
    await page.getByRole("button", { name: "Search" }).click();
    await expect(page.getByText("Reported Piece")).toBeVisible();

    await page.getByRole("button", { name: "Remove", exact: true }).click();
    // The dialog states the consequence, and the confirm stays disabled while the
    // reason is empty — the audit trail must never record a blank motive.
    await expect(page.getByText(/permanently deletes the score/i)).toBeVisible();
    await expect(page.getByRole("button", { name: "Remove permanently" })).toBeDisabled();

    await page.getByLabel(/Reason/).fill("DMCA #42");
    await expect(page.getByRole("button", { name: "Remove permanently" })).toBeEnabled();
    await page.getByRole("button", { name: "Remove permanently" }).click();

    // Removed: the confirmation shows and the row is gone from the re-run search.
    await expect(page.getByText("Score removed.")).toBeVisible();
    await expect(page.getByText("Reported Piece")).toHaveCount(0);
  });

  test("the dialog can be dismissed without removing anything", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { userScores } });
    await page.goto("/takedowns");
    await page.getByLabel("Owner id").fill("u-ada");
    await page.getByRole("button", { name: "Search" }).click();
    await expect(page.getByText("Reported Piece")).toBeVisible();

    const dialog = page.getByRole("heading", { name: "Remove this score?" });

    // Escape backs out — the score is untouched.
    await page.getByRole("button", { name: "Remove", exact: true }).click();
    await expect(dialog).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();

    // Clicking the backdrop backs out too.
    await page.getByRole("button", { name: "Remove", exact: true }).click();
    await expect(dialog).toBeVisible();
    await page.locator(".overlay").click({ position: { x: 5, y: 5 } });
    await expect(dialog).toBeHidden();

    // Neither dismissal removed the row.
    await expect(page.getByText("Reported Piece")).toBeVisible();
  });

  test("a moderator never reaches the screen", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { userScores } });
    await page.goto("/takedowns");
    // The route guard sends a non-admin away; the takedown form is not rendered.
    await expect(page.getByRole("button", { name: "Search" })).toHaveCount(0);
  });
});
