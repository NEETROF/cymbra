import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { SHARE_HEIGHT, SHARE_WIDTH } from "../../src/lib/music-media";
import { outputFileFor } from "../../src/lib/pinned-routes";
import { pngSize } from "../support/media";

// Run after `yarn build`: every Music page declares a share preview whose image is in the
// build, at an absolute https://cymbra.app/ URL, at 1200×630 (change: add-site-music-showcase,
// spec site-music-page) — so a link pasted in a chat shows a picture.

const dist = resolve(__dirname, "../../dist");
const MUSIC_PAGES = ["/music", "/en/music", "/es/music"];

describe("Music pages: share preview and video behaviour", () => {
  for (const page of MUSIC_PAGES) {
    it(`${page}`, () => {
      const html = readFileSync(resolve(dist, outputFileFor(page)), "utf8");
      const url = html.match(/<meta property="og:image" content="([^"]+)"/)?.[1];
      expect(url, `${page} has no og:image`).toBeDefined();
      expect(url!.startsWith("https://cymbra.app/"), url).toBe(true);
      const file = resolve(dist, new URL(url!).pathname.slice(1));
      expect(existsSync(file), `${file} is not in the build`).toBe(true);
      expect(pngSize(new Uint8Array(readFileSync(file)))).toEqual({ width: SHARE_WIDTH, height: SHARE_HEIGHT });
      expect(html).toContain('<meta name="twitter:card" content="summary_large_image">');
    });

    it(`${page}: the video's script holds back on reduced motion and only plays in view`, () => {
      const html = readFileSync(resolve(dist, outputFileFor(page)), "utf8");
      expect(html).toContain("prefers-reduced-motion");
      expect(html).toContain("IntersectionObserver");
      expect(html).not.toMatch(/<video[^>]*autoplay/);
    });
  }
});
