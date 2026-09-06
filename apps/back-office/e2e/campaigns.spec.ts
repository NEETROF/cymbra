import { test, expect, seed } from "./fixtures";
import type { E2EData } from "../src/lib/e2e-seam";

// Changes: add-premium-subscription, restructure-back-office-users-console. Drives the
// music-admin campaigns page — lifecycle, codes, members — and the flags drawer's beta
// selector, in a real browser against the gated fake seam (no backend). One account's
// subscription is NOT here any more: see user-detail.spec.ts.

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

test.describe("campaigns console (music admin only)", () => {
  test("closing a feature campaign marks it closed and lists its members as revoked", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/campaigns");

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
    await page.goto("/campaigns");

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
    await page.goto("/campaigns");
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

  test("the page holds no account lookup — that work lives on the account's own page", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/campaigns");

    await expect(page.getByRole("heading", { name: "Campaigns" })).toBeVisible();
    await expect(page.getByPlaceholder("handle or account id")).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Look up" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Grant premium" })).toHaveCount(0);
  });

  test("the old /plans path still resolves", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/plans");

    await expect(page).toHaveURL(/\/campaigns$/);
    await expect(page.getByRole("heading", { name: "Campaigns" })).toBeVisible();
  });

  test("a member row opens that member's account page", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/campaigns");

    await page
      .getByTestId("campaigns")
      .getByRole("row", { name: /midi-drums/ })
      .getByRole("button", { name: "Members" })
      .click();
    // The directory was never loaded here, so the row shows the id — the link is what
    // answers "who is this?", in one click.
    await page.getByTestId("members").getByRole("link").first().click();

    await expect(page).toHaveURL(/\/users\/u-(ada|cleo)$/);
    await expect(page.getByTestId("entitlements")).toBeVisible();
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
