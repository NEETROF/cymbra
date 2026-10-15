import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { beforeAll, describe, expect, it, vi } from "vitest";
import takenWith from "../fixtures/lingua/taken-with.json";

// The Lingua pages' captures, video and store buttons (change: add-site-lingua-showcase, D9, D10,
// D11), rendered through Astro's Container API on the fixtures' pairs, as
// `test/astro/lingua-page.spec.ts` renders them; and the Music page, whose buttons choose nothing.

const coverage = vi.hoisted(() => ({ tops: [] as number[], glossed: {} as Record<string, number[]> }));
const manifest = vi.hoisted(() => ({ routes: {} as Record<string, string[]> }));
vi.mock("../../src/data/lingua-coverage.json", () => ({ default: coverage }));
vi.mock("../../../lingua-extension/model-manifest.json", () => ({ default: manifest }));
vi.mock("../../src/lib/config", async (importOriginal) => {
  const original = await importOriginal<typeof import("../../src/lib/config")>();
  return { ...original, config: { ...original.config, discordUrl: null } };
});

import FrenchPage from "../../src/pages/lingua.astro";
import EnglishPage from "../../src/pages/en/lingua.astro";
import LocalePage from "../../src/pages/[locale]/lingua.astro";
import MusicPage from "../../src/pages/music.astro";
import Downloads from "../../src/components/Downloads.astro";

let container: AstroContainer;
beforeAll(async () => {
  container = await AstroContainer.create();
  coverage.tops = takenWith.coverage.tops;
  coverage.glossed = takenWith.coverage.glossed;
  manifest.routes = takenWith.routes;
});
type Page = Parameters<AstroContainer["renderToString"]>[0];
const render = (page: Page, params?: Record<string, string>) => container.renderToString(page, { partial: false, params });

const pages = [
  ["fr", () => render(FrenchPage)],
  ["en", () => render(EnglishPage)],
  ["es", () => render(LocalePage as unknown as Page, { locale: "es" })],
] as const;

/** Every opening tag `name` of `html`. */
const tags = (html: string, name: string) => [...html.matchAll(new RegExp(`<${name}\\b[^>]*>`, "g"))].map((m) => m[0]);
/** An attribute's value; "" for a boolean attribute; null when absent. */
function attr(tag: string, name: string): string | null {
  const valued = tag.match(new RegExp(`\\s${name}="([^"]*)"`));
  if (valued) return valued[1];
  return new RegExp(`\\s${name}(?=[\\s>/])`).test(tag) ? "" : null;
}
const mainOf = (html: string) => html.match(/<main>(.*?)<\/main>/s)?.[1] ?? "";

describe("the captures (D10)", () => {
  it.each(pages)("%s: every image has its width, height and alt text; the hero loads first, the others lazily", async (_lang, page) => {
    const images = tags(mainOf(await page()), "img");
    expect(images).toHaveLength(6);
    for (const img of images) {
      expect(Number(attr(img, "width")), img).toBeGreaterThan(0);
      expect(Number(attr(img, "height")), img).toBeGreaterThan(0);
      expect(attr(img, "alt")?.length ?? 0, img).toBeGreaterThan(30);
    }
    const [hero, ...rest] = images;
    expect(attr(hero, "loading")).toBe("eager");
    expect(attr(hero, "fetchpriority")).toBe("high");
    for (const img of rest) {
      expect(attr(img, "loading"), img).toBe("lazy");
      expect(attr(img, "decoding"), img).toBe("async");
    }
  });

  it.each(pages)("%s: the hero serves the card's close-up below 640 px, in its own shape", async (_lang, page) => {
    const hero = (await page()).match(/<figure class="hero-shot shot">(.*?)<\/figure>/s)?.[1] ?? "";
    const sources = tags(hero, "source");
    expect(sources.map((s) => [attr(s, "media"), attr(s, "type")])).toEqual([["(max-width: 639px)", "image/webp"]]);
    const [phone] = sources;
    expect(attr(phone, "srcset")).toContain("card-closeup");
    expect(Number(attr(phone, "height"))).toBeGreaterThan(Number(attr(phone, "width")));
    const [img] = tags(hero, "img");
    expect(attr(img, "srcset")).toMatch(/card\.(webp|jpg)/);
    expect(attr(img, "sizes")).toBeTruthy();
  });

  it.each(pages)("%s: each capture of « En images » links to its larger image; the hero does not", async (_lang, page) => {
    const html = mainOf(await page());
    const showcase = html.match(/<section class="showcase container">(.*?)<\/section>/s)?.[1] ?? "";
    const links = tags(showcase, "a").filter((a) => attr(a, "class") === "shot-zoom");
    expect(links).toHaveLength(5);
    for (const a of links) expect(attr(a, "href")).toMatch(/(second|review|stats|reader|reader-eink)\b/);
    const hero = html.match(/<figure class="hero-shot shot">(.*?)<\/figure>/s)?.[1] ?? "";
    expect(tags(hero, "a")).toEqual([]);
  });

  it.each(pages)("%s: the hero note follows the hero's capture", async (_lang, page) => {
    const html = mainOf(await page());
    const figure = html.indexOf('<figure class="hero-shot shot">');
    const note = html.indexOf("hero-note");
    expect(figure).toBeGreaterThan(0);
    expect(note).toBeGreaterThan(figure);
    expect(html.indexOf("</section>")).toBeLessThan(figure);
  });
});

describe("the video (D9)", () => {
  it.each(pages)("%s: muted, inline, with controls and a poster, nothing downloaded first, never autoplaying", async (lang, page) => {
    const html = await page();
    const videos = tags(html, "video");
    expect(videos).toHaveLength(1);
    const [video] = videos;
    for (const name of ["muted", "playsinline", "controls"]) expect(attr(video, name), name).toBe("");
    expect(attr(video, "preload")).toBe("none");
    expect(attr(video, "poster")).toBeTruthy();
    expect(attr(video, "autoplay")).toBeNull();
    expect(html).toContain(`<source src="/media/lingua/${lang}/demo.mp4" type="video/mp4">`);
  });
});

describe("the store buttons (D11)", () => {
  it.each(pages)("%s: each button says its store, alike until the script runs; the hints are hidden", async (_lang, page) => {
    const html = await page();
    const row = html.match(/<div class="cta-row" data-store-choice>(.*?)<\/div>/s)?.[1] ?? "";
    expect(tags(row, "a").map((a) => [attr(a, "data-store"), attr(a, "class")])).toEqual([
      ["chrome", "btn btn-ghost"],
      ["firefox", "btn btn-ghost"],
      ["safari", "btn btn-ghost"],
    ]);
    expect(row).toContain('data-android-href="https://addons.mozilla.org/android/addon/cymbra-lingua/"');
    const hints = tags(html, "p").filter((p) => attr(p, "data-store-hint") !== null);
    expect(hints.map((p) => [attr(p, "data-store-hint"), attr(p, "hidden")])).toEqual([
      ["ios", ""],
      ["android", ""],
    ]);
  });

  it("the Music page's buttons choose nothing and carry no script", async () => {
    const html = await render(MusicPage);
    expect(html).not.toContain("data-store");
    expect(html).not.toContain("<script");
    expect(html).toContain('<a class="btn btn-primary" href="https://apps.apple.com/app/id6789557194" rel="noopener">');
  });

  it("the post-checkout downloads render as before", async () => {
    const html = await container.renderToString(Downloads, { props: { lang: "en" } });
    expect(html).toBe(
      '<section class="downloads"><h3>Get Cymbra Music</h3><div class="cta-row left"><a class="btn btn-ghost" href="https://apps.apple.com/app/id6789557194" rel="noopener">App Store (iOS, iPadOS, macOS)</a><a class="btn btn-ghost" href="https://play.google.com/store/apps/details?id=com.cymbra.music" rel="noopener">Google Play</a><a class="btn btn-ghost disabled" href="#" aria-disabled="true">Windows / Linux — soon</a></div></section>',
    );
  });
});
