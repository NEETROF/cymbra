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

    await expect(page).toHaveURL(/\/music\/queue$/);
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

test.describe("memory-only session", () => {
  test("sign-in persists no token in localStorage or sessionStorage", async ({ page }) => {
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
    await expect(page).toHaveURL(/\/music\/queue$/);

    // The access token lives only in memory; only the locale key may be persisted.
    const storage = await page.evaluate(() => ({
      local: JSON.stringify(localStorage),
      session: JSON.stringify(sessionStorage),
    }));
    expect(storage.local).not.toContain("accessToken");
    expect(storage.local).not.toContain(tokenFor("moderator"));
    expect(storage.session).not.toContain("accessToken");
    expect(storage.session).not.toContain(tokenFor("moderator"));
  });

  test("a reload silently re-mints the session from the refresh cookie", async ({ page }) => {
    // `loginAs` seeds the fake HttpOnly cookie (session), not a persisted token.
    await seed(page, {
      loginAs: "moderator",
      data: { hits: [sampleHit()], counts: { pending: 1, accepted: 0, rejected: 0 } },
    });
    await page.goto("/music/queue");
    await expect(page).toHaveURL(/\/music\/queue$/);
    await expect(page.getByRole("heading", { name: "Catalog review" })).toBeVisible();

    // No persisted token, yet a reload re-mints an access token from the cookie.
    await page.reload();
    await expect(page).toHaveURL(/\/music\/queue$/);
    await expect(page.getByRole("heading", { name: "Catalog review" })).toBeVisible();
    const local = await page.evaluate(() => JSON.stringify(localStorage));
    expect(local).not.toContain("accessToken");
  });

  test("a returning user lands authenticated even when the cookie refresh is slow (no boot race)", async ({ page }) => {
    // A non-instant refresh is the case that exposed the router-install race: the
    // guard must not evaluate the (still-empty) session before the boot re-mint lands.
    await seed(page, {
      loginAs: "moderator",
      data: { hits: [sampleHit()], counts: { pending: 1, accepted: 0, rejected: 0 }, refreshDelayMs: 200 },
    });
    await page.goto("/music/queue");
    await expect(page).toHaveURL(/\/music\/queue$/);
    await expect(page.getByRole("heading", { name: "Catalog review" })).toBeVisible();
    // Must NOT have been bounced to sign-in mid-boot.
    await expect(page.getByRole("button", { name: "Sign in" })).toHaveCount(0);
  });

  test("an expired access token refreshes and retries instead of signing out", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: {
        hits: [sampleHit()],
        counts: { pending: 1, accepted: 0, rejected: 0 },
        // The first data-plane call 401s once; the cookie refresh + retry recovers it.
        failOnce: { searchCatalog: { code: UNAUTHENTICATED, message: "[unauthenticated] token expired" } },
      },
    });
    await page.goto("/music/queue");

    // Despite the one-shot 401, we stay on the queue with data (no redirect to sign-in).
    await expect(page).toHaveURL(/\/music\/queue$/);
    await expect(page.getByRole("heading", { name: "Catalog review" })).toBeVisible();
    await expect(page.getByText("Clair de Lune")).toBeVisible();
    await expect(page.locator("body")).not.toContainText("unauthenticated");
  });
});

test.describe("route guards", () => {
  test("an unauthenticated visitor is redirected to sign-in", async ({ page }) => {
    await seed(page);
    await page.goto("/music/queue");
    await expect(page).toHaveURL(/\/signin$/);
    await expect(page.getByRole("button", { name: "Sign in" })).toBeVisible();
  });

  test("an already-signed-in moderator visiting /signin is sent to the queue", async ({ page }) => {
    await seed(page, {
      loginAs: "moderator",
      data: { hits: [sampleHit()], counts: { pending: 1, accepted: 0, rejected: 0 } },
    });
    await page.goto("/signin");
    // The active in-memory session must not leave the user stranded on the login form
    // (previously the app shell rendered around it).
    await expect(page).toHaveURL(/\/music\/queue$/);
    await expect(page.getByRole("heading", { name: "Catalog review" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Sign in" })).toHaveCount(0);
  });

  test("a moderator cannot reach the admin-only users route", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: { hits: [sampleHit()] } });
    await page.goto("/users");
    // The admin guard bounces a non-admin to the catalog.
    await expect(page).toHaveURL(/\/music\/catalog$/);
    await expect(page.getByRole("heading", { name: "Catalog" })).toBeVisible();
  });
});
