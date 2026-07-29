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
    await page.goto("/music/queue");

    await expect(page.locator("tbody tr")).toHaveCount(2);
    await expect(page.getByText("Clair de Lune")).toBeVisible();
    await expect(page.getByText("Gymnopédie")).toBeVisible();

    // Total = pending + accepted + rejected = 9.
    await expect(page.getByTestId("stat-total").getByTestId("stat-value")).toHaveText("9");
    await expect(page.getByTestId("stat-approved").getByTestId("stat-value")).toHaveText("3");
    await expect(page.getByTestId("stat-pending").getByTestId("stat-value")).toHaveText("5");
  });

  test("paginates when the results exceed one page", async ({ page }) => {
    const many = Array.from({ length: 60 }, (_, i) =>
      sampleHit({ id: `${String(i).padStart(8, "0")}-0000-0000-0000-000000000000`, title: `Score ${i + 1}` }),
    );
    await seed(page, { loginAs: "moderator", data: { hits: many, counts: { pending: 60, accepted: 0, rejected: 0 } } });
    await page.goto("/music/catalog");

    // Page 1: the first window of 50, Previous disabled, Next available.
    await expect(page.locator("tbody tr")).toHaveCount(50);
    await expect(page.getByText("1–50 of 60")).toBeVisible();
    await expect(page.getByRole("button", { name: /Previous/ })).toBeDisabled();

    await page.getByRole("button", { name: /Next/ }).click();

    // Page 2: the remaining 10 rows, Next now disabled.
    await expect(page.getByText("51–60 of 60")).toBeVisible();
    await expect(page.locator("tbody tr")).toHaveCount(10);
    await expect(page.getByText("Score 51")).toBeVisible();
    await expect(page.getByRole("button", { name: /Next/ })).toBeDisabled();

    await page.getByRole("button", { name: /Previous/ }).click();
    await expect(page.getByText("1–50 of 60")).toBeVisible();
    await expect(page.locator("tbody tr")).toHaveCount(50);
  });

  test("clicking a row opens its detail page (self-sufficient on deep-link)", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { hits: [sampleHit()] } });
    await page.goto("/music/queue");

    await page.locator("tbody tr").first().click();
    await expect(page).toHaveURL(/\/music\/score\/11111111-1111-1111-1111-111111111111$/);
    await expect(page.getByRole("heading", { name: "Clair de Lune" })).toBeVisible();
    // The preview fetched the bytes and ran the wasm renderer on them; the fake seam's
    // 3 placeholder bytes aren't valid MusicXML, so it degrades to the render-failed
    // note (proving the bytes path is self-sufficient on a deep link).
    await expect(page.getByText("Notation could not be rendered.")).toBeVisible();
  });
});

test.describe("score detail", () => {
  test("accepting a score records the decision and returns to the queue", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { hit: sampleHit(), hits: [sampleHit()] } });
    await page.goto("/music/score/11111111-1111-1111-1111-111111111111");

    await expect(page.getByRole("heading", { name: "Clair de Lune" })).toBeVisible();
    await page.getByRole("button", { name: "Accept" }).click();
    await expect(page).toHaveURL(/\/music\/queue$/);
  });

  test("a bytes fetch failure degrades gracefully — metadata still shows", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: {
        hit: sampleHit(),
        fail: { getCatalogScoreBytes: { code: FAILED_PRECONDITION, message: "catalog score bytes not available yet" } },
      },
    });
    await page.goto("/music/score/11111111-1111-1111-1111-111111111111");

    // Metadata is unaffected; the bytes error is an informational note, not a crash.
    await expect(page.getByRole("heading", { name: "Clair de Lune" })).toBeVisible();
    await expect(page.getByText("Not available yet. Try again later.")).toBeVisible();
    await expect(page.locator("body")).not.toContainText("catalog score bytes not available yet");
  });
});

test.describe("i18n", () => {
  test("the language toggle switches the whole console to French", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { hits: [sampleHit()] } });
    await page.goto("/music/queue");

    await expect(page.getByRole("heading", { name: "Catalog review" })).toBeVisible();
    await page.getByRole("button", { name: "FR", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Revue du catalogue" })).toBeVisible();
  });
});
