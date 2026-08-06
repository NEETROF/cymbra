import { test, expect, seed } from "./fixtures";

// Change: add-soundfont-entitlement-previews. The per-row control is merged: a font
// with NO preview shows "Generate sample"; once a preview exists the same slot becomes
// a play button (which auditions the server-rendered clip). Drives it in a real
// browser against the fake seam (no backend).

test("the play control is merged with Generate sample by preview availability", async ({ page }) => {
  await seed(page, {
    loginAs: "admin",
    data: {
      soundfonts: [
        {
          id: "ydp-grand",
          label: "YDP Grand Piano",
          instrument: "piano",
          license: "CC-BY 3.0",
          attribution: "Roberto / Zenph Studios",
          moderationStatus: "accepted",
          // No hasPreview → the slot starts as "Generate sample".
        },
      ],
    },
  });
  await page.goto("/soundfonts");
  await expect(page.getByRole("heading", { name: "Sound fonts" })).toBeVisible();
  await expect(page.getByText("YDP Grand Piano")).toBeVisible();

  // Hide Vite's dev error overlay — a fresh worktree hasn't run `yarn gen:wasm`, so a
  // lazily-imported worker (unrelated to this screen) triggers the overlay. The
  // SoundFonts screen itself renders fine underneath.
  await page.addStyleTag({ content: "vite-error-overlay{display:none !important}" });

  // No preview yet → the slot is "Generate sample", and there is no play button.
  const generate = page.getByRole("button", { name: "Generate sample" });
  await expect(generate).toBeVisible();
  await expect(page.getByRole("button", { name: "Play" })).toHaveCount(0);

  // Generate the preview → success, and the slot becomes a play button.
  await generate.click();
  await expect(page.getByText("Sample generated.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Play" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Generate sample" })).toHaveCount(0);
});
