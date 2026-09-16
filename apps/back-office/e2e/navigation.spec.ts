import { test, expect, seed } from "./fixtures";
import type { E2EData } from "../src/lib/e2e-seam";

// Change: restructure-back-office-navigation. The sidebar is grouped by product, shows an
// entry exactly when its page opens, every page lives under its section's prefix, and the
// paths the pages had before keep working.

const ada = { userId: "u-ada", handle: "ada", displayName: "Ada Lovelace", roles: [] as string[] };

const userScores: NonNullable<E2EData["userScores"]> = [
  {
    id: "s1",
    ownerId: "u-ada",
    title: "Reported Piece",
    composer: "Anon",
    sizeBytes: "1024",
    createdAt: "1760000000",
    rightsBasis: "private_use",
  },
  {
    id: "s2",
    ownerId: "u-bob",
    title: "Another Score",
    composer: "Someone",
    sizeBytes: "2048",
    createdAt: "1760000000",
    rightsBasis: "private_use",
  },
];

const sidebar = (page: import("@playwright/test").Page) => page.getByRole("navigation", { name: "Menu" });

test.describe("grouped sidebar", () => {
  test("a global admin sees Music, Lingua and Administration, each a named group", async ({ page }) => {
    await seed(page, { loginAs: "global-admin", data: {} });
    await page.goto("/music/queue");

    const groups = sidebar(page).getByRole("group");
    await expect(groups).toHaveCount(3);
    await expect(groups.nth(0)).toHaveAccessibleName("Music");
    await expect(groups.nth(1)).toHaveAccessibleName("Lingua");
    await expect(groups.nth(2)).toHaveAccessibleName("Administration");

    const music = sidebar(page).getByRole("group", { name: "Music" }).getByRole("link");
    await expect(music).toHaveText([
      "Catalog review",
      "Catalog",
      "Private scores",
      "Sound fonts",
      "Campaigns",
      "Usage",
    ]);
    await expect(sidebar(page).getByRole("group", { name: "Lingua" }).getByRole("link")).toHaveText(["Overview"]);
    await expect(sidebar(page).getByRole("group", { name: "Administration" }).getByRole("link")).toHaveText([
      "Users",
      "Feature flags",
      "Notifications",
      "Jobs",
    ]);
  });

  test("a lingua-only admin lands on Lingua and is offered no Music page", async ({ page }) => {
    await seed(page, { loginAs: "lingua-admin", data: {} });
    await page.goto("/");

    await expect(page).toHaveURL(/\/lingua\/overview$/);
    await expect(sidebar(page).getByRole("group", { name: "Music" })).toHaveCount(0);
    for (const name of ["Sound fonts", "Campaigns", "Usage"]) {
      await expect(page.getByRole("link", { name })).toHaveCount(0);
    }
    // And the pages themselves are closed to them, not only unlisted.
    await page.goto("/music/campaigns");
    await expect(page).toHaveURL(/\/lingua\/overview$/);
  });

  test("a music moderator sees only what they moderate", async ({ page }) => {
    await seed(page, { loginAs: "moderator", data: {} });
    await page.goto("/");

    await expect(page).toHaveURL(/\/music\/queue$/);
    await expect(sidebar(page).getByRole("group")).toHaveCount(1);
    await expect(sidebar(page).getByRole("link")).toHaveText(["Catalog review", "Catalog"]);
  });

  test("a sidebar entry opens its page under the section prefix", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { userScores } });
    await page.goto("/music/queue");

    await sidebar(page).getByRole("link", { name: "Private scores" }).click();
    await expect(page).toHaveURL(/\/music\/private-scores$/);
    await expect(page.getByRole("heading", { name: "Private scores — takedown on notice" })).toBeVisible();
  });
});

test.describe("former paths", () => {
  test("a former takedown link keeps its query", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { userScores } });
    await page.goto("/takedowns?owner=u-ada");

    await expect(page).toHaveURL(/\/music\/private-scores\?owner=u-ada$/);
    await expect(page.getByText("Reported Piece")).toBeVisible();
    await expect(page.getByText("Another Score")).toHaveCount(0);
  });

  test("a former account link keeps its id and section", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada] } });
    await page.goto("/users/u-ada?tab=roles");

    await expect(page).toHaveURL(/\/admin\/users\/u-ada\?tab=roles$/);
    await expect(page.getByRole("heading", { name: "ada" })).toBeVisible();
  });

  test("a former path does not open a page the operator may not open", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: {} });
    await page.goto("/jobs");

    await expect(page).toHaveURL(/\/music\/queue$/);
  });
});

test.describe("account and private scores", () => {
  test("from an account to its private scores, and back to the account", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { accounts: [ada], userScores } });
    await page.goto("/admin/users/u-ada");

    await page.getByTestId("account-private-scores").click();
    await expect(page).toHaveURL(/\/music\/private-scores\?owner=u-ada$/);
    await expect(page.getByLabel("Owner id")).toHaveValue("u-ada");
    await expect(page.getByText("Reported Piece")).toBeVisible();
    await expect(page.getByText("Another Score")).toHaveCount(0);

    await page.getByRole("link", { name: "u-ada" }).click();
    await expect(page).toHaveURL(/\/admin\/users\/u-ada$/);
    await expect(page.getByRole("heading", { name: "ada" })).toBeVisible();
  });

  test("a lookup survives a reload", async ({ page }) => {
    await seed(page, { loginAs: "admin", data: { userScores } });
    await page.goto("/music/private-scores");

    await page.getByLabel("Title contains").fill("another");
    await page.getByRole("button", { name: "Search" }).click();
    await expect(page).toHaveURL(/\/music\/private-scores\?title=another$/);
    await expect(page.getByText("Another Score")).toBeVisible();

    await page.reload();
    await expect(page.getByLabel("Title contains")).toHaveValue("another");
    await expect(page.getByText("Another Score")).toBeVisible();
    await expect(page.getByText("Reported Piece")).toHaveCount(0);
  });

  test("an admin outside the music scope gets no private-scores link", async ({ page }) => {
    await seed(page, { loginAs: "live-admin", data: { accounts: [ada] } });
    await page.goto("/admin/users/u-ada");

    await expect(page.getByRole("heading", { name: "ada" })).toBeVisible();
    await expect(page.getByTestId("account-private-scores")).toHaveCount(0);
  });
});
