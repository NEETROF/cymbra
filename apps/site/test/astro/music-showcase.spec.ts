import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { beforeAll, describe, expect, it } from "vitest";
import { LANGS } from "../../src/lib/i18n";
import { GALLERY, VIDEO_COPY } from "../../src/lib/music-media";
import MusicShowcase from "../../src/components/MusicShowcase.astro";

// The Music page's video and gallery, and the layout's share preview, as the build renders
// them (change: add-site-music-showcase, spec site-music-page).

/** The few entities Astro writes in text and attributes. */
const decode = (s: string) =>
  s.replace(/&#39;|&#x27;/g, "'").replace(/&quot;/g, '"').replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&");

let container: AstroContainer;
beforeAll(async () => {
  container = await AstroContainer.create();
});

describe("MusicShowcase", () => {
  for (const lang of LANGS) {
    it(`${lang}: the video starts silent, off the first render, with a sound button in ${lang}`, async () => {
      const html = await container.renderToString(MusicShowcase, { props: { lang } });
      const video = html.match(/<video[^>]*>/)?.[0] ?? "";
      for (const attr of ["muted", "loop", "playsinline", 'preload="none"', "poster="]) {
        expect(video, attr).toContain(attr);
      }
      expect(video).not.toContain("autoplay");
      expect(html).toMatch(new RegExp(`<source src="[^"]*showcase[^"]*\\.mp4" type="video/mp4"`));
      expect(html).toContain(`data-listen="${VIDEO_COPY[lang].listen}"`);
      expect(html).toContain(`data-mute="${VIDEO_COPY[lang].mute}"`);
    });

    it(`${lang}: the gallery shows its captures in ${lang}, described, lazy after the first`, async () => {
      const html = await container.renderToString(MusicShowcase, { props: { lang } });
      const imgs = [...html.matchAll(/<img [^>]*>/g)].map((m) => m[0]);
      expect(imgs).toHaveLength(GALLERY.length);
      GALLERY.forEach((shot, i) => {
        expect(decode(imgs[i].match(/alt="([^"]*)"/)?.[1] ?? "")).toBe(shot.alt[lang]);
        expect(imgs[i]).toContain(i === 0 ? 'loading="eager"' : 'loading="lazy"');
        expect(decode(html)).toContain(shot.caption[lang]);
      });
      // Only this language's files.
      for (const other of LANGS.filter((l) => l !== lang)) {
        expect(html).not.toContain(`/music/${other}/`);
      }
      expect(html).toContain("image/avif");
    });
  }
});
