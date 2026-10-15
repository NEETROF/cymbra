import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { describe, expect, it } from "vitest";

// Run after `yarn build` (see `vitest.build.config.ts`). The weight budget of the Lingua page's
// media and of the site as a whole (change: add-site-lingua-showcase, D14). The numbers live in
// this one table; `apps/site/README.md` points here. A failure names each file and its size. The
// captures and videos come from `yarn capture` in apps/lingua-extension: one heavier than its budget
// is re-encoded there, not let through here.
const BUDGET = {
  /** Each image variant the Lingua pages serve (`<img>`, `srcset`). */
  imageVariant: 250 * 1024,
  /** The largest variant of the Lingua pages' hero capture. */
  heroVariant: 200 * 1024,
  /** Each page's share image (`og:image`). */
  shareImage: 300 * 1024,
  /** Each demo video's poster. */
  poster: 150 * 1024,
  /** Each demo video. */
  video: 4 * 1024 * 1024,
  /** Every image, poster and video of the site together. */
  allMedia: 30 * 1024 * 1024,
  /** Cloudflare Pages: no file of 25 MiB or more, fewer than 20,000 files. */
  file: 25 * 1024 * 1024,
  files: 20_000,
};

const dist = resolve(__dirname, "../../dist");
const LINGUA_PAGES = ["lingua/index.html", "en/lingua/index.html", "es/lingua/index.html"];
const MEDIA = /\.(avif|webp|jpe?g|png|gif|svg|ico|mp4|webm)$/i;

function filesUnder(dir = dist): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    return entry.isDirectory() ? filesUnder(path) : [relative(dist, path)];
  });
}
const sizeOf = (file: string) => statSync(resolve(dist, file)).size;
const kb = (bytes: number) => `${(bytes / 1024).toFixed(1)} KB`;
/** Each of `files` heavier than `limit`, as « file: size ». */
const over = (files: string[], limit: number) =>
  [...new Set(files)].filter((f) => sizeOf(f) > limit).map((f) => `${f}: ${kb(sizeOf(f))} (budget ${kb(limit)})`);
/** A served address as a file of `dist/`. */
const local = (url: string) => decodeURIComponent(new URL(url, "https://cymbra.app").pathname).replace(/^\//, "");

const read = (page: string) => readFileSync(resolve(dist, page), "utf8");
const srcsetFiles = (value: string) => value.split(",").map((part) => local(part.trim().split(/\s+/)[0]));

describe("the Lingua page's media stay within their budget", () => {
  const html = LINGUA_PAGES.map(read);
  const images = html.flatMap((page) => [
    ...[...page.matchAll(/<img [^>]*src="([^"]+)"/g)].map((m) => local(m[1])),
    ...[...page.matchAll(/ srcset="([^"]+)"/g)].flatMap((m) => srcsetFiles(m[1])),
  ]);

  it("has captures to check", () => {
    expect(images.length).toBeGreaterThan(30);
  });

  it(`each image variant ≤ ${kb(BUDGET.imageVariant)}`, () => {
    expect(over(images, BUDGET.imageVariant)).toEqual([]);
  });

  it(`the hero's largest variant ≤ ${kb(BUDGET.heroVariant)}`, () => {
    for (const page of html) {
      const hero = page.match(/<figure class="hero-shot shot">(.*?)<\/figure>/s)?.[1] ?? "";
      // The wide image's variants: the `<img>`'s srcset (the close-up's `<source media>` is a phone's).
      const sources = [...hero.matchAll(/<img [^>]*srcset="([^"]+)"/g)].flatMap((m) => srcsetFiles(m[1]));
      expect(sources.length).toBeGreaterThan(0);
      const largest = sources.reduce((a, b) => (sizeOf(a) >= sizeOf(b) ? a : b));
      expect(over([largest], BUDGET.heroVariant)).toEqual([]);
    }
  });

  it(`each poster ≤ ${kb(BUDGET.poster)} and each video ≤ ${kb(BUDGET.video)}`, () => {
    const posters = html.flatMap((page) => [...page.matchAll(/<video [^>]*poster="([^"]+)"/g)].map((m) => local(m[1])));
    const videos = html.flatMap((page) => [...page.matchAll(/<source src="([^"]+\.mp4)"/g)].map((m) => local(m[1])));
    expect(posters).toHaveLength(LINGUA_PAGES.length);
    expect(videos).toHaveLength(LINGUA_PAGES.length);
    expect(over(posters, BUDGET.poster)).toEqual([]);
    expect(over(videos, BUDGET.video)).toEqual([]);
  });
});

describe("the site's media and files stay within the host's limits", () => {
  const files = filesUnder();

  it(`each share image ≤ ${kb(BUDGET.shareImage)}`, () => {
    const shares = files
      .filter((f) => f.endsWith(".html"))
      .flatMap((page) => [...read(page).matchAll(/<meta property="og:image" content="([^"]+)">/g)].map((m) => local(m[1])));
    expect(shares.length).toBeGreaterThan(0);
    expect(over(shares, BUDGET.shareImage)).toEqual([]);
  });

  it(`every image, poster and video together ≤ ${kb(BUDGET.allMedia)}`, () => {
    const total = files.filter((f) => MEDIA.test(f)).reduce((sum, f) => sum + sizeOf(f), 0);
    expect(total, `the site's media weigh ${kb(total)}`).toBeLessThanOrEqual(BUDGET.allMedia);
  });

  it(`no file of ${kb(BUDGET.file)} or more, fewer than ${BUDGET.files} files`, () => {
    expect(files.filter((f) => sizeOf(f) >= BUDGET.file).map((f) => `${f}: ${kb(sizeOf(f))}`)).toEqual([]);
    expect(files.length).toBeLessThan(BUDGET.files);
  });
});
