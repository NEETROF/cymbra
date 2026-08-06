import { test, expect, seed } from "./fixtures";

// Change: add-soundfont-entitlement-previews. The "Generate sample" per-row action
// (re)renders a font's server preview clip through the injectable HTTP seam. Drives
// it in a real browser against the fake seam (no backend) and captures screenshots.

test("Generate sample action renders and reports success", async ({ page }) => {
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
        },
        {
          id: "upright-piano-kw",
          label: "Upright Piano KW",
          instrument: "piano",
          license: "CC0-1.0",
          attribution: "",
          moderationStatus: "accepted",
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

  // The new per-row action is present on every row.
  const generate = page.getByRole("button", { name: "Generate sample" });
  await expect(generate.first()).toBeVisible();

  // Clicking it (re)renders the preview and reports success.
  await generate.first().click();
  await expect(page.getByText("Sample generated.")).toBeVisible();
});
