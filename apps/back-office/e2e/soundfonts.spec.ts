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

    // Remove it, answering the in-app confirmation.
    await page
      .getByRole("row", { name: /YDP Grand \(edited\)/ })
      .getByRole("button", { name: "Remove" })
      .click();
    await page.getByRole("dialog").getByRole("button", { name: "Confirm" }).click();
    await expect(page.getByText("YDP Grand (edited)")).toHaveCount(0);
    await expect(page.getByText("Upright Piano KW")).toBeVisible();
  });

  // Change: add-soundfont-uploader-attribution — the review queue names the
  // contributor, surfaces a reopened font's justification, and rejecting captures
  // a reason (sent with the evaluate call; the seam stores it on the row).
  test("uploader pseudo + resubmission note render, and reject captures a reason", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: {
        soundfonts: [
          {
            id: "community-grand",
            label: "Community Grand",
            instrument: "piano",
            license: "CC-BY 4.0",
            attribution: "Sample Author",
            moderationStatus: "pending",
            uploaderDisplayName: "alice",
            resubmissionNote: "now relicensed under CC-BY",
            hasPreview: true,
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

    // The user-contributed row names its uploader and shows the justification…
    const community = page.getByRole("row", { name: /Community Grand/ });
    await expect(community.getByText("Proposed by alice")).toBeVisible();
    await expect(community.getByText("Re-submission: now relicensed under CC-BY")).toBeVisible();
    // …while the seeded font shows neither.
    const seeded = page.getByRole("row", { name: /Upright Piano KW/ });
    await expect(seeded.getByText(/Proposed by/)).toHaveCount(0);
    await expect(seeded.getByText(/Re-submission/)).toHaveCount(0);

    // Reject opens the inline reason input; confirming sends the reason and the
    // row flips to Rejected.
    await community.getByRole("button", { name: "Reject" }).click();
    await community.getByPlaceholder("Rejection reason (shown to the uploader)").fill("blurry samples");
    await community.getByRole("button", { name: "Reject" }).click();
    await expect(page.getByRole("row", { name: /Community Grand/ }).getByText("Rejected")).toBeVisible();
  });

  test("a non-admin moderator cannot reach the screen", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { counts: { pending: 0, accepted: 0, rejected: 0 } } });
    await page.goto("/music/queue");
    await expect(page.getByRole("link", { name: "Sound fonts" })).toHaveCount(0);
    await page.goto("/soundfonts");
    await expect(page).not.toHaveURL(/\/soundfonts$/);
  });
});
