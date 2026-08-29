import { test, expect, seed } from "./fixtures";
import type { E2EData } from "../src/lib/e2e-seam";

// Change: add-premium-subscription. Drives the music-admin plan console, the accounts
// directory's plan/beta columns + filters, and the flags drawer's beta selector in a
// real browser against the gated fake seam (no backend).

const ada = { userId: "u-ada", handle: "ada", displayName: "Ada Lovelace", roles: [] as string[] };
const bob = { userId: "u-bob", handle: "bob", displayName: "Bob Ross", roles: [] as string[] };
const cleo = { userId: "u-cleo", handle: "cleo", displayName: "Cleo", roles: [] as string[] };

const campaigns: NonNullable<E2EData["campaigns"]> = [
  { key: "spring-trial", name: "Spring trial", kind: "premium_trial", durationDays: 90 },
  { key: "midi-drums", name: "MIDI drums", kind: "feature" },
  { key: "old-beta", name: "Old beta", kind: "feature", closedAt: "2026-01-01T00:00:00Z", acceptsEnrolment: false },
];

// ada: an active premium-trial row (via code) + a feature-beta membership; cleo: the
// beta only; bob: free, no betas.
const plans: NonNullable<E2EData["plans"]> = {
  "u-ada": {
    rows: [
      {
        id: "e-ada-trial",
        source: "code",
        providerRef: "",
        campaignId: "spring-trial",
        startsAt: "2026-08-01T00:00:00Z",
        endsAt: "2026-10-30T00:00:00Z",
        status: "active",
      },
    ],
    memberships: [
      {
        campaignKey: "midi-drums",
        campaignName: "MIDI drums",
        kind: "feature",
        userId: "u-ada",
        enrolledAt: "2026-08-02T00:00:00Z",
        source: "code",
      },
    ],
  },
  "u-cleo": {
    memberships: [
      {
        campaignKey: "midi-drums",
        campaignName: "MIDI drums",
        kind: "feature",
        userId: "u-cleo",
        enrolledAt: "2026-08-03T00:00:00Z",
        source: "admin",
      },
    ],
  },
};

const data: E2EData = { accounts: [ada, bob, cleo], campaigns, plans };

test.describe("plans console (music admin only)", () => {
  test("a lookup shows the trial row, the beta membership and the effective plan", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/plans");
    await expect(page.getByRole("heading", { name: "Plans" })).toBeVisible();

    await page.getByPlaceholder("handle or account id").fill("ada");
    await page.getByRole("button", { name: "Look up" }).click();

    const summary = page.getByTestId("effective-plan");
    await expect(summary).toContainText("Premium");
    await expect(summary).toContainText("trial");
    await expect(summary).toContainText("spring-trial");
    // The trial row and the beta membership are both listed.
    await expect(page.getByTestId("entitlements")).toContainText("code");
    await expect(page.getByTestId("entitlements")).toContainText("spring-trial");
    await expect(page.getByTestId("memberships")).toContainText("midi-drums");
    // A code row is revocable from the console (store rows would not be).
    await expect(page.getByTestId("entitlements").getByRole("button", { name: "Revoke" })).toBeVisible();
  });

  test("revoking an entitlement asks for an audited reason in-app, then ends the row", async ({ page }) => {
    // This flow used to ask through `window.prompt`, which blocks the renderer:
    // unreachable from Playwright, and it froze browser automation in a
    // production session. It is an in-app dialog now, hence this coverage.
    await seed(page, { loginAs: "admin", data });
    await page.goto("/plans");
    await page.getByPlaceholder("handle or account id").fill("ada");
    await page.getByRole("button", { name: "Look up" }).click();

    await page.getByTestId("entitlements").getByRole("button", { name: "Revoke" }).click();
    const dialog = page.getByRole("dialog").filter({ hasText: "Confirmation" });
    await expect(dialog).toBeVisible();
    // No reason typed ⇒ the console will not send it (the reason is audited).
    await expect(dialog.getByRole("button", { name: "Confirm" })).toBeDisabled();
    await dialog.getByLabel("Reason").fill("granted by mistake");
    await dialog.getByRole("button", { name: "Confirm" }).click();

    await expect(page.getByText("Entitlement revoked.")).toBeVisible();
    await expect(page.getByTestId("entitlements")).toContainText("revoked");
  });

  test("granting premium with an end date and a reason adds an admin row and re-looks up", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/plans");
    await page.getByPlaceholder("handle or account id").fill("bob");
    await page.getByRole("button", { name: "Look up" }).click();
    await expect(page.getByTestId("effective-plan")).toContainText("free");

    await page.getByRole("button", { name: "Grant premium" }).click();
    const dialog = page.getByRole("dialog");
    await expect(dialog.getByRole("heading", { name: "Grant premium to bob" })).toBeVisible();
    // Without a date the open-ended checkbox is required; give a date instead.
    await dialog.getByLabel("End date", { exact: true }).fill("2027-01-31");
    await dialog.getByLabel("Reason").fill("beta thanks");
    await dialog.getByRole("button", { name: "Confirm" }).click();

    await expect(page.getByText("Premium granted.")).toBeVisible();
    await expect(page.getByTestId("effective-plan")).toContainText("Premium");
    await expect(page.getByTestId("entitlements")).toContainText("admin");
  });

  test("an open-ended grant needs the explicit confirmation; refused by the server otherwise", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/plans");
    await page.getByPlaceholder("handle or account id").fill("bob");
    await page.getByRole("button", { name: "Look up" }).click();
    await page.getByRole("button", { name: "Grant premium" }).click();
    const dialog = page.getByRole("dialog");
    await dialog.getByLabel("Reason").fill("forever");
    // No date, no confirmation ⇒ the console won't even send it.
    await expect(dialog.getByRole("button", { name: "Confirm" })).toBeDisabled();
    await dialog.getByLabel("I confirm an open-ended grant (no end date)").check();
    await dialog.getByRole("button", { name: "Confirm" }).click();
    await expect(page.getByText("Premium granted.")).toBeVisible();
    // Flagged as open-ended in the listing.
    await expect(page.getByTestId("entitlements")).toContainText("open-ended");
  });

  test("enrolling a handle in a feature campaign adds a membership", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/plans");
    await page.getByPlaceholder("handle or account id").fill("bob");
    await page.getByRole("button", { name: "Look up" }).click();

    await page.getByRole("button", { name: "Enrol in campaign" }).click();
    const dialog = page.getByRole("dialog");
    await dialog.getByLabel("Campaign").selectOption({ label: "MIDI drums (feature beta)" });
    await dialog.getByLabel("Reason").fill("tester");
    await dialog.getByRole("button", { name: "Confirm" }).click();

    await expect(page.getByText("Enrolled.")).toBeVisible();
    await expect(page.getByTestId("memberships")).toContainText("midi-drums");
    await expect(page.getByTestId("effective-plan")).toContainText("midi-drums");
  });

  test("closing a feature campaign marks it closed and lists its members as revoked", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/plans");

    const row = page.getByTestId("campaigns").getByRole("row", { name: /midi-drums/ });
    await row.getByRole("button", { name: "Members" }).click();
    await expect(page.getByRole("heading", { name: "Members of midi-drums" })).toBeVisible();
    await expect(page.getByTestId("members").getByRole("row")).toHaveCount(3); // header + ada + cleo

    await row.getByRole("button", { name: "Close campaign" }).click();
    // In-app confirmation, so the flow stays drivable from Playwright.
    const confirmClose = page.getByRole("dialog").filter({ hasText: "Confirmation" });
    await expect(confirmClose).toContainText("PAUSE");
    await confirmClose.getByRole("button", { name: "Confirm" }).click();
    await expect(page.getByText("Campaign closed.")).toBeVisible();
    await expect(row.getByRole("button", { name: "Close campaign" })).toHaveCount(0);
    await expect(row.getByRole("button", { name: "Mint codes" })).toHaveCount(0);
    // Members re-listed as ended.
    await expect(page.getByTestId("members")).toContainText("revoked");
  });

  test("minting codes shows the clear text once; dismissed, it is gone", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { ...data, mintedCodes: ["QQQQ-1111", "WWWW-2222", "EEEE-3333"] } });
    await page.goto("/plans");

    await page
      .getByTestId("campaigns")
      .getByRole("row", { name: /spring-trial/ })
      .getByRole("button", { name: "Mint codes" })
      .click();
    const dialog = page.getByRole("dialog");
    await dialog.getByLabel("How many").fill("2");
    await dialog.getByRole("button", { name: "Mint codes" }).click();

    await expect(page.getByRole("heading", { name: "2 codes — shown once" })).toBeVisible();
    const codes = page.getByTestId("minted-codes");
    await expect(codes).toContainText("QQQQ-1111");
    await expect(codes).toContainText("WWWW-2222");
    await expect(codes).not.toContainText("EEEE-3333");
    await expect(page.getByRole("button", { name: "Download .txt" })).toBeVisible();

    await page.getByRole("button", { name: "Done" }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.locator("body")).not.toContainText("QQQQ-1111");
  });

  test("creating a trial campaign lists it with its duration", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/plans");
    await page.getByRole("button", { name: "Create campaign" }).click();
    const dialog = page.getByRole("dialog");
    await dialog.getByLabel("Key").fill("autumn-trial");
    await dialog.getByLabel("Name").fill("Autumn trial");
    await expect(dialog.getByLabel("Duration (days)")).toHaveValue("90");
    await dialog.getByLabel("Duration (days)").fill("30");
    await dialog.getByRole("button", { name: "Create campaign" }).click();
    await expect(page.getByText("Campaign created.")).toBeVisible();
    const row = page.getByTestId("campaigns").getByRole("row", { name: /autumn-trial/ });
    await expect(row).toContainText("30 days");
  });
});

test.describe("accounts directory: plan badges + filters", () => {
  test("a music admin sees plan/beta badges and filters by trial and by beta", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/roles");
    await expect(page.getByRole("columnheader", { name: "Plan" })).toBeVisible();

    // Badges per row, from the batch call.
    const adaRow = page.getByRole("row", { name: /ada/ });
    await expect(adaRow).toContainText("Premium");
    await expect(adaRow).toContainText("trial");
    await expect(adaRow).toContainText("midi-drums");
    await expect(page.getByRole("row", { name: /bob/ })).toContainText("free");

    // Filter by premium trial ⇒ only ada.
    await page.getByRole("combobox", { name: "Plan" }).selectOption("trial");
    await expect(page.getByText("ada", { exact: true })).toBeVisible();
    await expect(page.getByText("bob", { exact: true })).toHaveCount(0);
    await expect(page.getByText("cleo", { exact: true })).toHaveCount(0);

    // Filter by beta ⇒ ada + cleo, whatever their plan.
    await page.getByRole("combobox", { name: "Plan" }).selectOption("any");
    await page.getByRole("combobox", { name: "Beta" }).selectOption("midi-drums");
    await expect(page.getByText("ada", { exact: true })).toBeVisible();
    await expect(page.getByText("cleo", { exact: true })).toBeVisible();
    await expect(page.getByText("bob", { exact: true })).toHaveCount(0);

    // Both criteria: trial ∩ beta ⇒ ada only.
    await page.getByRole("combobox", { name: "Plan" }).selectOption("trial");
    await expect(page.getByText("ada", { exact: true })).toBeVisible();
    await expect(page.getByText("cleo", { exact: true })).toHaveCount(0);
  });

  test("the beta filter lists only the open campaigns", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/roles");
    const options = page.getByRole("combobox", { name: "Beta" }).locator("option");
    await expect(options.filter({ hasText: "MIDI drums" })).toHaveCount(1);
    await expect(options.filter({ hasText: "Spring trial" })).toHaveCount(1);
    await expect(options.filter({ hasText: "Old beta" })).toHaveCount(0);
  });
});

test.describe("flags console: plan / beta rollout", () => {
  test("the rollout selector offers premium_only and beta:<key> for open campaigns only", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: { ...data, flags: [{ key: "rating.enabled", app: "music", value: false }] },
    });
    await page.goto("/flags");
    await page.getByRole("button", { name: "Edit" }).first().click();

    const select = page.getByRole("combobox", { name: /rollout scope/ });
    await expect(select).toBeVisible();
    const values = await select.locator("option").evaluateAll((els) => els.map((e) => (e as HTMLOptionElement).value));
    expect(values).toContain("premium_only");
    expect(values).toContain("beta:midi-drums");
    expect(values).toContain("beta:spring-trial");
    expect(values).not.toContain("beta:old-beta");
    // Labels come from the campaign names, never free text.
    await expect(select.locator("option[value='beta:midi-drums']")).toHaveText("beta: MIDI drums");

    // Save with a beta scope: the row is marked as beta-scoped.
    await select.selectOption("beta:midi-drums");
    await page.getByRole("dialog").getByRole("button", { name: "Save" }).click();
    await expect(page.getByRole("row", { name: /rating\.enabled/ })).toContainText("beta");
  });
});
