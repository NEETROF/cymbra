import { test, expect, seed, sampleHit } from "./fixtures";

// Change: add-score-daily-access-rewards. The score detail's teaser control is merged:
// a piece with NO audio sample shows "Generate sample"; once one exists the same slot
// becomes a play button (which auditions the server-rendered clip the app plays on a
// locked piece). The catalog filter bar gets a "No sample" view for the backfill.

test("the detail's Generate sample becomes a play control once a sample exists", async ({ page }) => {
  await seed(page, {
    loginAs: "moderator",
    data: { hit: sampleHit(), hits: [sampleHit()] },
  });
  await page.goto("/music/score/11111111-1111-1111-1111-111111111111");
  await expect(page.getByRole("heading", { name: "Clair de Lune" })).toBeVisible();
  await page.addStyleTag({ content: "vite-error-overlay{display:none !important}" });

  const generate = page.getByTestId("generate-sample");
  await expect(generate).toBeVisible();
  await expect(page.getByTestId("play-sample")).toHaveCount(0);

  await generate.click();
  await expect(page.getByText("Sample generated.")).toBeVisible();
  await expect(page.getByTestId("play-sample")).toBeVisible();
  await expect(page.getByTestId("generate-sample")).toHaveCount(0);
});

test("a regeneration failure surfaces as a toast, the slot stays Generate sample", async ({ page }) => {
  await seed(page, {
    loginAs: "moderator",
    data: {
      hit: sampleHit(),
      hits: [sampleHit()],
      fail: { regenerateScorePreview: { code: 9, message: "no preview font configured" } },
    },
  });
  await page.goto("/music/score/11111111-1111-1111-1111-111111111111");
  await expect(page.getByRole("heading", { name: "Clair de Lune" })).toBeVisible();
  await page.addStyleTag({ content: "vite-error-overlay{display:none !important}" });

  await page.getByTestId("generate-sample").click();
  await expect(page.getByRole("alert").first()).toBeVisible();
  await expect(page.getByTestId("generate-sample")).toBeVisible();
});

test("the catalog filter bar offers the no-sample view", async ({ page }) => {
  await seed(page, { loginAs: "moderator", data: { hits: [sampleHit()] } });
  await page.goto("/music/catalog");
  await expect(page.getByRole("heading", { name: "Catalog" })).toBeVisible();
  const select = page.getByLabel("audio sample");
  await expect(select).toBeVisible();
  await select.selectOption("no");
  await expect(select).toHaveValue("no");
});
