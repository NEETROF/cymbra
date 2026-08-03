import { test, expect, seed } from "./fixtures";

// Change: add-soundfont-back-office-management. Drives the SoundFont admin screen in a
// real browser against the gated fake seam (no backend). Create/edit happen in a
// right-to-left drawer; the upload route is faked through the seed (see e2e-seam), so
// add → appears → edit → remove all run. (Audio preview is not exercised here — the
// wasm synth is covered by the app's own tests.)

test.describe("sound fonts admin", () => {
  test("an admin adds, edits, and removes a font via the drawer", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: {
        soundfonts: [
          {
            id: "upright-piano-kw",
            label: "Upright Piano KW",
            instrument: "piano",
            license: "CC0-1.0",
            attribution: "",
          },
        ],
      },
    });
    await page.goto("/soundfonts");
    await expect(page.getByRole("heading", { name: "Sound fonts" })).toBeVisible();
    await expect(page.getByText("Upright Piano KW")).toBeVisible();

    // Open the create drawer.
    await page.getByRole("button", { name: "Add SoundFont" }).click();
    const drawer = page.getByRole("dialog");
    await expect(drawer.getByRole("heading", { name: "Add a SoundFont" })).toBeVisible();

    // Fill the form + pick a file, then save (scoped to the drawer). The id is
    // auto-minted (uuidv7) on create — there is no id field to fill.
    await drawer.getByLabel("label").fill("YDP Grand Piano");
    await drawer.getByLabel("license").selectOption("CC-BY 3.0");
    await drawer.getByLabel("attribution").fill("Roberto / Zenph Studios");
    await drawer.getByLabel("SoundFont file (.sf2)").setInputFiles({
      name: "ydp.sf2",
      mimeType: "application/octet-stream",
      buffer: Buffer.from([0x52, 0x49, 0x46, 0x46, 0, 0, 0, 0, 0x73, 0x66, 0x62, 0x6b]),
    });
    await drawer.getByRole("button", { name: "Add SoundFont" }).click();
    await expect(page.getByText("YDP Grand Piano")).toBeVisible();

    // Edit it via the drawer.
    await page
      .getByRole("row", { name: /YDP Grand Piano/ })
      .getByRole("button", { name: "Edit" })
      .click();
    const editDrawer = page.getByRole("dialog");
    await expect(editDrawer.getByRole("heading", { name: "Edit SoundFont" })).toBeVisible();
    await editDrawer.getByLabel("label").fill("YDP Grand (edited)");
    await editDrawer.getByRole("button", { name: "Save" }).click();
    await expect(page.getByText("YDP Grand (edited)")).toBeVisible();

    // Remove it (confirm dialog auto-accepted).
    page.on("dialog", (d) => d.accept());
    await page
      .getByRole("row", { name: /YDP Grand \(edited\)/ })
      .getByRole("button", { name: "Remove" })
      .click();
    await expect(page.getByText("YDP Grand (edited)")).toHaveCount(0);
    await expect(page.getByText("Upright Piano KW")).toBeVisible();
  });

  test("a non-admin moderator cannot reach the screen", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { counts: { pending: 0, accepted: 0, rejected: 0 } } });
    await page.goto("/music/queue");
    await expect(page.getByRole("link", { name: "Sound fonts" })).toHaveCount(0);
    await page.goto("/soundfonts");
    await expect(page).not.toHaveURL(/\/soundfonts$/);
  });
});
