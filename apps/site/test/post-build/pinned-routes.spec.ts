import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { PINNED_ROUTES, outputFileFor } from "../../src/lib/pinned-routes";

// Run after `yarn build` (see `vitest.build.config.ts`): a route a shipped client or
// a store listing points at must have survived the build. Deleting one of those pages
// fails the pull request here instead of being discovered in production — where the
// host answers `200` with the fallback page and nothing looks wrong.

const dist = resolve(__dirname, "../../dist");

describe("pinned routes survive the build", () => {
  it("has a build to check", () => {
    expect(
      existsSync(dist),
      "dist/ is missing — run `yarn build` before `yarn check:routes`",
    ).toBe(true);
  });

  it("produces a page for every pinned route", () => {
    // Report every missing route at once: fixing them one CI run at a time is how a
    // gate becomes something people route around.
    const missing = PINNED_ROUTES.filter(
      (r) => !existsSync(resolve(dist, outputFileFor(r.path))),
    ).map((r) => `${r.path} → dist/${outputFileFor(r.path)} (pinned by ${r.pinnedBy})`);

    expect(missing, `pinned routes missing from the build:\n  ${missing.join("\n  ")}`).toEqual([]);
  });

  it("ships a not-found page in each locale", () => {
    // Cloudflare Pages serves the nearest `404.html` up the requested path. Without
    // the English one, an unmatched `/en/...` answers in French; without either, the
    // host falls back to the home page with a 200 and the breakage is invisible.
    for (const file of ["404.html", "en/404.html"]) {
      expect(existsSync(resolve(dist, file)), `dist/${file} is missing`).toBe(true);
    }
  });

  it("maps a route to the file Astro writes for it", () => {
    expect(outputFileFor("/")).toBe("index.html");
    expect(outputFileFor("/en/")).toBe("en/index.html");
    expect(outputFileFor("/account")).toBe("account/index.html");
    expect(outputFileFor("/checkout/done")).toBe("checkout/done/index.html");
  });
});
