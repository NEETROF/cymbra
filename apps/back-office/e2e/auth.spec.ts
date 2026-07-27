import { test, expect, seed, tokenFor, sampleHit } from "./fixtures";

// Connect Code.Unauthenticated = 16 (see @connectrpc/connect).
const UNAUTHENTICATED = 16;

test.describe("sign-in", () => {
  test("a moderator signs in and lands on the review queue", async ({ page }) => {
    await seed(page, {
      data: {
        tokens: { accessToken: tokenFor("moderator"), refreshToken: "r" },
        hits: [sampleHit()],
        counts: { pending: 1, accepted: 0, rejected: 0 },
      },
    });
    await page.goto("/signin");

    await page.getByPlaceholder("email").fill("mod@cymbra.app");
    await page.getByPlaceholder("password").fill("secret");
    await page.getByRole("button", { name: "Sign in" }).click();

    await expect(page).toHaveURL(/\/queue$/);
    await expect(page.getByRole("heading", { name: "Catalog review" })).toBeVisible();
    await expect(page.getByText("Clair de Lune")).toBeVisible();
  });

  test("bad credentials show a localized message, never a raw gRPC code", async ({ page }) => {
    await seed(page, {
      data: { fail: { signInLocal: { code: UNAUTHENTICATED, message: "[unauthenticated] invalid credentials" } } },
    });
    await page.goto("/signin");

    await page.getByPlaceholder("email").fill("mod@cymbra.app");
    await page.getByPlaceholder("password").fill("wrong");
    await page.getByRole("button", { name: "Sign in" }).click();

    const alert = page.getByRole("alert");
    await expect(alert).toHaveText("Invalid credentials or expired session.");
    // The raw cause must never leak into the DOM.
    await expect(page.locator("body")).not.toContainText("unauthenticated");
    await expect(page.locator("body")).not.toContainText("invalid credentials");
    // Bad-credentials sign-in must NOT redirect (stays on /signin).
    await expect(page).toHaveURL(/\/signin$/);
  });

  test("a signed-in non-moderator hits the access-denied screen", async ({ page }) => {
    await seed(page, { data: { tokens: { accessToken: tokenFor("none"), refreshToken: "r" } } });
    await page.goto("/signin");

    await page.getByPlaceholder("email").fill("user@cymbra.app");
    await page.getByPlaceholder("password").fill("secret");
    await page.getByRole("button", { name: "Sign in" }).click();

    await expect(page).toHaveURL(/\/denied$/);
    await expect(page.getByRole("heading", { name: "Access denied" })).toBeVisible();
  });
});

test.describe("route guards", () => {
  test("an unauthenticated visitor is redirected to sign-in", async ({ page }) => {
    await seed(page);
    await page.goto("/queue");
    await expect(page).toHaveURL(/\/signin$/);
    await expect(page.getByRole("button", { name: "Sign in" })).toBeVisible();
  });

  test("a moderator cannot reach the admin-only roles route", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { hits: [sampleHit()] } });
    await page.goto("/roles");
    // The admin guard bounces a non-admin to the catalog.
    await expect(page).toHaveURL(/\/$/);
    await expect(page.getByRole("heading", { name: "Catalog" })).toBeVisible();
  });
});
