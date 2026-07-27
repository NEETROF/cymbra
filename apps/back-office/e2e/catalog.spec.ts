import { test, expect, seed, sampleHit } from "./fixtures";

// Connect Code.FailedPrecondition = 9.
const FAILED_PRECONDITION = 9;

test.describe("queue", () => {
  test("renders one row per hit and the KPI cards from real counts", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: {
        hits: [sampleHit(), sampleHit({ id: "22222222-2222-2222-2222-222222222222", title: "Gymnopédie" })],
        counts: { pending: 5, accepted: 3, rejected: 1 },
      },
    });
    await page.goto("/queue");

    await expect(page.locator("tbody tr")).toHaveCount(2);
    await expect(page.getByText("Clair de Lune")).toBeVisible();
    await expect(page.getByText("Gymnopédie")).toBeVisible();

    // Total = pending + accepted + rejected = 9.
    await expect(page.getByTestId("stat-total").getByTestId("stat-value")).toHaveText("9");
    await expect(page.getByTestId("stat-approved").getByTestId("stat-value")).toHaveText("3");
    await expect(page.getByTestId("stat-pending").getByTestId("stat-value")).toHaveText("5");
  });

  test("clicking a row opens its detail page (self-sufficient on deep-link)", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { hits: [sampleHit()] } });
    await page.goto("/queue");

    await page.locator("tbody tr").first().click();
    await expect(page).toHaveURL(/\/score\/11111111-1111-1111-1111-111111111111$/);
    await expect(page.getByRole("heading", { name: "Clair de Lune" })).toBeVisible();
    await expect(page.getByText(/Score loaded \(3 bytes\)/)).toBeVisible();
  });
});

test.describe("score detail", () => {
  test("accepting a score records the decision and returns to the queue", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { hit: sampleHit(), hits: [sampleHit()] } });
    await page.goto("/score/11111111-1111-1111-1111-111111111111");

    await expect(page.getByRole("heading", { name: "Clair de Lune" })).toBeVisible();
    await page.getByRole("button", { name: "Accept" }).click();
    await expect(page).toHaveURL(/\/queue$/);
  });

  test("a bytes fetch failure degrades gracefully — metadata still shows", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: {
        hit: sampleHit(),
        fail: { getCatalogScoreBytes: { code: FAILED_PRECONDITION, message: "catalog score bytes not available yet" } },
      },
    });
    await page.goto("/score/11111111-1111-1111-1111-111111111111");

    // Metadata is unaffected; the bytes error is an informational note, not a crash.
    await expect(page.getByRole("heading", { name: "Clair de Lune" })).toBeVisible();
    await expect(page.getByText("Not available yet. Try again later.")).toBeVisible();
    await expect(page.locator("body")).not.toContainText("catalog score bytes not available yet");
  });
});

test.describe("i18n", () => {
  test("the language toggle switches the whole console to French", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { hits: [sampleHit()] } });
    await page.goto("/queue");

    await expect(page.getByRole("heading", { name: "Review queue" })).toBeVisible();
    await page.getByRole("button", { name: "FR", exact: true }).click();
    await expect(page.getByRole("heading", { name: "File de revue" })).toBeVisible();
  });
});
