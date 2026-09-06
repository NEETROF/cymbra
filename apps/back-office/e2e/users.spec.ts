import { test, expect, seed } from "./fixtures";
import type { E2EData } from "../src/lib/e2e-seam";

// Change: restructure-back-office-users-console. The users directory is a surface for
// FINDING an account — search, scope, plan and beta filters, pagination — and for opening
// it. Every per-account gesture moved to `/users/{user_id}` (see user-detail.spec.ts).

const ada = { userId: "u-ada", handle: "ada", displayName: "Ada Lovelace", roles: [] as string[] };
const bob = { userId: "u-bob", handle: "bob", displayName: "Bob Ross", roles: ["moderator"] };
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

test.describe("users directory (admin only)", () => {
  test("an admin browses the directory and opens an account", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/users");

    await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
    // Read-only: the row carries no action buttons any more — they live on the account.
    await expect(page.getByRole("button", { name: "Grant moderator" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Revoke sessions" })).toHaveCount(0);

    // The handle is a real link: keyboard-reachable, openable in a new tab.
    await page.getByRole("link", { name: "ada" }).click();

    await expect(page).toHaveURL(/\/users\/u-ada$/);
    await expect(page.getByRole("heading", { name: "ada" })).toBeVisible();
  });

  test("clicking anywhere on the row opens that account too", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada, bob] } });
    await page.goto("/users");

    await page.getByRole("row", { name: /Bob Ross/ }).click();

    await expect(page).toHaveURL(/\/users\/u-bob$/);
  });

  test("the old /roles path still resolves", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/roles");

    await expect(page).toHaveURL(/\/users$/);
    await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
  });

  test("a music-only admin sees no scope selector (single scope)", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/users");

    await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
    // Only `music` is authorized → the scope picker is hidden.
    await expect(page.getByRole("combobox", { name: "Scope" })).toHaveCount(0);
  });

  test("a global admin switches scope and the roles column follows", async ({ page }) => {
    const tara = {
      userId: "u-tara",
      handle: "tara",
      displayName: "Tara",
      rolesByScope: { global: [] as string[], music: ["moderator"], live: [] },
    };
    await seed(page, { loginAs: "global-admin", data: { accounts: [tara] } });
    await page.goto("/users");

    const scope = page.getByRole("combobox", { name: "Scope" });
    await expect(scope).toBeVisible();
    await scope.selectOption("music");
    await expect(page.getByRole("row", { name: /tara/ })).toContainText("moderator");

    await scope.selectOption("live");
    await expect(page.getByRole("row", { name: /tara/ })).not.toContainText("moderator");
  });

  test("filtering by handle narrows the directory", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada, bob] } });
    await page.goto("/users");

    await expect(page.getByRole("link", { name: "ada" })).toBeVisible();
    await expect(page.getByRole("link", { name: "bob" })).toBeVisible();

    await page.getByPlaceholder("filter by handle or email").fill("ada");
    await page.getByRole("button", { name: "Search" }).click();

    await expect(page.getByRole("link", { name: "ada" })).toBeVisible();
    await expect(page.getByRole("link", { name: "bob" })).toHaveCount(0);
  });

  test("an empty result shows a friendly message, not a raw code", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/users");

    await page.getByPlaceholder("filter by handle or email").fill("zzz-nobody");
    await page.getByRole("button", { name: "Search" }).click();

    await expect(page.getByText("No accounts.")).toBeVisible();
    await expect(page.locator("body")).not.toContainText("unavailable");
    await expect(page.locator("body")).not.toContainText("[");
  });
});

test.describe("users directory: plan badges + filters", () => {
  test("a music admin sees plan/beta badges and filters by trial and by beta", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users");
    await expect(page.getByRole("columnheader", { name: "Plan" })).toBeVisible();

    // Badges per row, from the batch call.
    const adaRow = page.getByRole("row", { name: /ada/ });
    await expect(adaRow).toContainText("Premium");
    await expect(adaRow).toContainText("trial");
    await expect(adaRow).toContainText("midi-drums");
    await expect(page.getByRole("row", { name: /bob/ })).toContainText("free");

    // Filter by premium trial ⇒ only ada.
    await page.getByRole("combobox", { name: "Plan" }).selectOption("trial");
    await expect(page.getByRole("link", { name: "ada" })).toBeVisible();
    await expect(page.getByRole("link", { name: "bob" })).toHaveCount(0);
    await expect(page.getByRole("link", { name: "cleo" })).toHaveCount(0);

    // Filter by beta ⇒ ada + cleo, whatever their plan.
    await page.getByRole("combobox", { name: "Plan" }).selectOption("any");
    await page.getByRole("combobox", { name: "Beta" }).selectOption("midi-drums");
    await expect(page.getByRole("link", { name: "ada" })).toBeVisible();
    await expect(page.getByRole("link", { name: "cleo" })).toBeVisible();
    await expect(page.getByRole("link", { name: "bob" })).toHaveCount(0);

    // Both criteria: trial ∩ beta ⇒ ada only.
    await page.getByRole("combobox", { name: "Plan" }).selectOption("trial");
    await expect(page.getByRole("link", { name: "ada" })).toBeVisible();
    await expect(page.getByRole("link", { name: "cleo" })).toHaveCount(0);
  });

  test("from a filtered list straight to one account's rows", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users");
    await page.getByRole("combobox", { name: "Beta" }).selectOption("midi-drums");

    await page.getByRole("link", { name: "ada" }).click();

    // The account's own entitlements and memberships are already listed — the admin
    // never re-identified them between the two screens.
    await expect(page.getByTestId("entitlements")).toContainText("spring-trial");
    await expect(page.getByTestId("memberships")).toContainText("midi-drums");
  });

  test("the beta filter lists only the open campaigns", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users");
    const options = page.getByRole("combobox", { name: "Beta" }).locator("option");
    await expect(options.filter({ hasText: "MIDI drums" })).toHaveCount(1);
    await expect(options.filter({ hasText: "Spring trial" })).toHaveCount(1);
    await expect(options.filter({ hasText: "Old beta" })).toHaveCount(0);
  });
});
