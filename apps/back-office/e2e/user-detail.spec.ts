import { test, expect, seed } from "./fixtures";
import type { E2EData } from "../src/lib/e2e-seam";

// Change: restructure-back-office-users-console. One account, one address: everything the
// console knows and can do about a person, on `/users/{user_id}` — subscription, roles in
// every scope the caller administers, audit history, reliability, sessions.

const ada = { userId: "u-ada", handle: "ada", displayName: "Ada Lovelace", roles: [] as string[] };
const bob = { userId: "u-bob", handle: "bob", displayName: "Bob Ross", roles: [] as string[] };

const campaigns: NonNullable<E2EData["campaigns"]> = [
  { key: "spring-trial", name: "Spring trial", kind: "premium_trial", durationDays: 90 },
  { key: "midi-drums", name: "MIDI drums", kind: "feature" },
];

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
};

const data: E2EData = { accounts: [ada, bob], campaigns, plans };

test.describe("account detail: subscription", () => {
  test("a deep link shows the trial row, the beta membership and the effective plan", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    // Straight to the URL: no directory page visited first, nothing typed.
    await page.goto("/users/u-ada");

    await expect(page.getByRole("heading", { name: "ada" })).toBeVisible();
    const summary = page.getByTestId("effective-plan");
    await expect(summary).toContainText("Premium");
    await expect(summary).toContainText("trial");
    await expect(summary).toContainText("spring-trial");
    await expect(page.getByTestId("entitlements")).toContainText("code");
    await expect(page.getByTestId("entitlements")).toContainText("spring-trial");
    await expect(page.getByTestId("memberships")).toContainText("midi-drums");
    // A code row is revocable from the console (store rows would not be).
    await expect(page.getByTestId("entitlements").getByRole("button", { name: "Revoke" })).toBeVisible();
    // And no lookup field anywhere: the page already knows whose account it is.
    await expect(page.getByPlaceholder("handle or account id")).toHaveCount(0);
  });

  test("an unknown id shows a localized not-found state", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users/u-nobody");

    await expect(page.getByRole("heading", { name: "Account not found" })).toBeVisible();
    await expect(page.locator("body")).not.toContainText("[");
    // A way back, so the page is not a dead end.
    await expect(page.getByRole("link", { name: "All users" })).toBeVisible();
  });

  test("revoking an entitlement asks for an audited reason in-app, then ends the row", async ({ page }) => {
    // This flow used to ask through `window.prompt`, which blocks the renderer:
    // unreachable from Playwright, and it froze browser automation in a
    // production session. It is an in-app dialog now, hence this coverage.
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users/u-ada");

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

  // The dialog opens with focus inside it — otherwise the Escape handler on the
  // dialog element never receives the keydown and the key does nothing at all.
  test("escape backs out of the revoke dialog without revoking", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users/u-ada");

    await page.getByTestId("entitlements").getByRole("button", { name: "Revoke" }).click();
    const dialog = page.getByRole("dialog").filter({ hasText: "Confirmation" });
    await expect(dialog).toBeVisible();
    await page.keyboard.press("Escape");

    await expect(dialog).toBeHidden();
    await expect(page.getByTestId("entitlements")).not.toContainText("revoked");
  });

  test("granting premium with an end date and a reason adds an admin row and re-reads", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users/u-bob");
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

  test("an open-ended grant needs the explicit confirmation", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users/u-bob");

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

  test("enrolling the account in a feature campaign adds a membership", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users/u-bob");

    await page.getByRole("button", { name: "Enrol in campaign" }).click();
    const dialog = page.getByRole("dialog");
    await dialog.getByLabel("Campaign").selectOption({ label: "MIDI drums (feature beta)" });
    await dialog.getByLabel("Reason").fill("tester");
    await dialog.getByRole("button", { name: "Confirm" }).click();

    await expect(page.getByText("Enrolled.")).toBeVisible();
    await expect(page.getByTestId("memberships")).toContainText("midi-drums");
    await expect(page.getByTestId("effective-plan")).toContainText("midi-drums");
  });

  test("the reason typed on an action comes back on the page", async ({ page }) => {
    // The console demands a free-text justification on every plan mutation. It used to be
    // written to an audit trail no surface could show — the operator typed it for nothing.
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users/u-bob");
    await expect(page.getByTestId("plan-audit")).toContainText("No plan change recorded");

    await page.getByRole("button", { name: "Enrol in campaign" }).click();
    const dialog = page.getByRole("dialog");
    await dialog.getByLabel("Campaign").selectOption({ label: "MIDI drums (feature beta)" });
    await dialog.getByLabel("Reason").fill("early access for the drums beta");
    await dialog.getByRole("button", { name: "Confirm" }).click();
    await expect(page.getByText("Enrolled.")).toBeVisible();

    // Straight away, on the same page — no refresh, no other screen.
    const audit = page.getByTestId("plan-audit");
    await expect(audit).toContainText("early access for the drums beta");
    await expect(audit).toContainText("enrolled");
    await expect(audit).toContainText("midi-drums");
    await expect(audit).toContainText("e2e-admin");

    // And the next act appends to it, with its own reason.
    await page.getByTestId("memberships").getByRole("button", { name: "Revoke" }).click();
    const reasonDialog = page.getByRole("dialog").filter({ hasText: "Confirmation" });
    await reasonDialog.getByLabel("Reason").fill("wrong person");
    await reasonDialog.getByRole("button", { name: "Confirm" }).click();

    await expect(audit).toContainText("wrong person");
    await expect(audit).toContainText("membership revoked");
    await expect(audit.getByRole("row")).toHaveCount(3); // header + the two acts
  });

  test("an admin outside the music scope gets no subscription block at all", async ({ page }) => {
    await seed(page, { loginAs: "live-admin", data });
    await page.goto("/users/u-ada");

    // Absent, not empty: an empty "Subscription" section would read as "no plan".
    await expect(page.getByRole("heading", { name: "ada" })).toBeVisible();
    await expect(page.getByTestId("effective-plan")).toHaveCount(0);
    await expect(page.getByTestId("entitlements")).toHaveCount(0);
    await expect(page.locator("body")).not.toContainText("Subscription");
    await expect(page.locator("body")).not.toContainText("spring-trial");
    // The rest of the page is still theirs to work with.
    await expect(page.getByRole("heading", { name: "Role history" })).toBeVisible();
  });

  test("switching accounts never shows the previous account's rights", async ({ page }) => {
    await seed(page, { loginAs: "admin", data });
    await page.goto("/users/u-ada");
    await expect(page.getByTestId("effective-plan")).toContainText("Premium");

    await page.getByRole("link", { name: "All users" }).click();
    await page.getByRole("link", { name: "bob" }).click();

    await expect(page.getByRole("heading", { name: "bob" })).toBeVisible();
    await expect(page.getByTestId("effective-plan")).toContainText("free");
    await expect(page.getByTestId("entitlements")).not.toContainText("spring-trial");
  });
});

test.describe("account detail: roles, history, reliability, sessions", () => {
  test("a role is granted in a named scope and the page reflects it", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/users/u-ada");

    await page.getByRole("button", { name: "Grant moderator in music" }).click();

    await expect(page.getByRole("button", { name: "Revoke moderator in music" })).toBeVisible();
  });

  test("the role history follows the grant, with no page refresh", async ({ page }) => {
    // The history sits on the same screen as the toggle that writes to it. It used to
    // stay one refresh behind: the roles updated, the audit table did not.
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/users/u-ada");
    await expect(page.getByTestId("role-history")).toContainText("No role changes yet.");

    await page.getByRole("button", { name: "Grant moderator in music" }).click();

    const history = page.getByTestId("role-history");
    await expect(history).toContainText("grant");
    await expect(history).toContainText("moderator");
    await expect(history).toContainText("e2e-admin");

    // And the revocation lands on top of it, still without a refresh.
    await page.getByRole("button", { name: "Revoke moderator in music" }).click();
    await expect(history.getByRole("row")).toHaveCount(3); // header + revoke + grant
    await expect(history).toContainText("revoke");
  });

  test("granting a role keeps the operator where they were on the page", async ({ page }) => {
    // The refresh after the action used to fold the account back through `loading`: the
    // page unmounted, the document collapsed to a couple of lines, the browser clamped
    // the scroll to the top — and the subscription panel remounted and re-fetched.
    await seed(page, { loginAs: "global-admin", data: { ...data, accounts: [ada] } });
    await page.setViewportSize({ width: 1280, height: 420 });
    await page.goto("/users/u-ada");
    await expect(page.getByTestId("role-history")).toBeVisible();

    await page.getByRole("button", { name: "Grant moderator in music" }).scrollIntoViewIfNeeded();
    const before = await page.evaluate(() => window.scrollY);
    expect(before).toBeGreaterThan(0);

    await page.getByRole("button", { name: "Grant moderator in music" }).click();
    await expect(page.getByRole("button", { name: "Revoke moderator in music" })).toBeVisible();

    // Same place, and the page never blanked out under them.
    expect(await page.evaluate(() => window.scrollY)).toBe(before);
    await expect(page.getByTestId("effective-plan")).toBeVisible();
  });

  test("a global admin manages every scope on the one page, no selector", async ({ page }) => {
    const tara = {
      userId: "u-tara",
      handle: "tara",
      displayName: "Tara",
      rolesByScope: { global: [] as string[], music: [], live: [] },
    };
    await seed(page, { loginAs: "global-admin", data: { accounts: [tara] } });
    await page.goto("/users/u-tara");

    await expect(page.getByRole("button", { name: "Grant moderator in global" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Grant moderator in live" })).toBeVisible();

    await page.getByRole("button", { name: "Grant admin in live" }).click();
    await expect(page.getByRole("button", { name: "Revoke admin in live" })).toBeVisible();
    // The grant landed in `live` only.
    await expect(page.getByRole("button", { name: "Grant admin in global" })).toBeVisible();
  });

  test("the audit history shows the acting admin's handle, not the raw id", async ({ page }) => {
    const grants = [
      {
        targetUserId: "u-ada",
        scope: "music",
        role: "moderator",
        action: "grant",
        actingAdmin: "019f60be-6cd9-74e2-a600-9893bd2aaa3a",
        actingAdminHandle: "bossadmin",
        at: 1_700_000_000,
      },
    ];
    await seed(page, { loginAs: "admin", data: { accounts: [ada], grants } });
    await page.goto("/users/u-ada");

    // No button to press: the history is part of the account's page.
    await expect(page.getByTestId("role-history")).toContainText("bossadmin");
    await expect(page.locator("body")).not.toContainText("019f60be-6cd9");
  });

  test("the reliability panel renders read-only curator metrics", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: {
        accounts: [ada],
        reliability: {
          totalRatings: 57,
          coverageContribution: 42,
          alignmentRate: 0.82,
          settledCount: 40,
          alignedCount: 33,
        },
      },
    });
    await page.goto("/users/u-ada");

    await page.getByRole("button", { name: "Reliability" }).click();

    await expect(page.getByRole("heading", { name: "Curator reliability" })).toBeVisible();
    await expect(page.getByText("57", { exact: true })).toBeVisible();
    await expect(page.getByText("42", { exact: true })).toBeVisible();
    await expect(page.getByText("82%", { exact: true })).toBeVisible();
    await expect(page.getByText("33 of 40 settled ratings aligned")).toBeVisible();
    // Read-only: the panel never offers a role change of its own.
  });

  test("a failed reliability read surfaces a humanized error, not a raw code", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: {
        accounts: [ada],
        // Connect UNAVAILABLE = 14.
        fail: { getCuratorReliability: { code: 14, message: "[unavailable] curator lookup down" } },
      },
    });
    await page.goto("/users/u-ada");

    await page.getByRole("button", { name: "Reliability" }).click();

    await expect(page.getByRole("alert")).toHaveText("Service unavailable. Try again.");
    await expect(page.locator("body")).not.toContainText("curator lookup down");
    await expect(page.locator("body")).not.toContainText("[unavailable]");
  });

  test("a failed session revoke surfaces an error instead of failing silently", async ({ page }) => {
    await seed(page, {
      loginAs: "admin",
      data: {
        accounts: [ada],
        // Connect UNAVAILABLE = 14.
        fail: { revokeAccountSessions: { code: 14, message: "[unavailable] backend down" } },
      },
    });
    await page.goto("/users/u-ada");

    await page.getByRole("button", { name: "Revoke sessions" }).click();
    // In-app confirmation (never window.confirm — a native dialog blocks the renderer).
    await page.getByRole("dialog").getByRole("button", { name: "Confirm" }).click();

    await expect(page.getByRole("alert")).toHaveText("Service unavailable. Try again.");
    await expect(page.locator("body")).not.toContainText("backend down");
  });
});
