import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  APP_STORE_ID,
  APP_STORE_LISTING,
  CHROME_WEB_STORE_ID,
  CHROME_WEB_STORE_LISTING,
  FIREFOX_ADDONS_LISTING,
  FIREFOX_ADDONS_SLUG,
  reviewPageFor,
  STORE_REVIEW_PAGE,
} from "@/state/store-links.ts";

// The extension's store addresses (add-lingua-comeback-hints D12) are the site's: the site's
// module is read as text — the extension's build does not reach apps/site — and its Chrome Web
// Store id, Firefox Add-ons slug and App Store id must be the ones here.

const SITE_STORES = join(dirname(fileURLToPath(import.meta.url)), "../../site/src/lib/stores.ts");

/** The string constant `name` of the site's module. */
function siteConstant(name: string): string {
  const source = readFileSync(SITE_STORES, "utf8");
  const found = new RegExp(`export const ${name}\\s*=\\s*"([^"]+)"`, "s").exec(source);
  if (!found) throw new Error(`no ${name} in apps/site/src/lib/stores.ts`);
  return found[1];
}

describe("the store addresses", () => {
  it("are the site's: the same Chrome Web Store id, Firefox Add-ons slug and App Store id", () => {
    expect(siteConstant("LINGUA_CHROME_WEB_STORE")).toContain(`/${CHROME_WEB_STORE_ID}`);
    expect(siteConstant("LINGUA_FIREFOX_ADDONS")).toContain(`/addon/${FIREFOX_ADDONS_SLUG}/`);
    expect(siteConstant("LINGUA_APP_STORE")).toContain(`/id${APP_STORE_ID}`);
    expect(siteConstant("LINGUA_CHROME_WEB_STORE")).toBe(CHROME_WEB_STORE_LISTING);
    expect(siteConstant("LINGUA_FIREFOX_ADDONS")).toBe(FIREFOX_ADDONS_LISTING);
    expect(siteConstant("LINGUA_APP_STORE")).toBe(APP_STORE_LISTING);
  });

  it("send each build's reader to its own store's review page", () => {
    expect(reviewPageFor("chromium")).toBe(`${CHROME_WEB_STORE_LISTING}/reviews`);
    expect(reviewPageFor("firefox")).toBe(FIREFOX_ADDONS_LISTING);
    expect(reviewPageFor("safari")).toBe(`https://apps.apple.com/app/id${APP_STORE_ID}?action=write-review`);
    expect(STORE_REVIEW_PAGE).toBe(reviewPageFor("chromium")); // the build under test
  });
});
