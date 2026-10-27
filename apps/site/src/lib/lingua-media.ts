// The Lingua page's captures (change: add-site-lingua-showcase, D7, D10), made by `yarn capture` in
// apps/lingua-extension (tool/capture/) and committed: per interface language, the WebP sources
// under `src/assets/lingua/<lang>/` (2560 px wide — the build makes the served sizes), the share
// image, the demo's poster, and the demo itself under `public/media/lingua/<lang>/demo.mp4`, served
// as it is. A page asks for its own language's set; a missing file fails the build, naming it.
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import type { ImageMetadata } from "astro";
import type { Lang } from "./i18n";
import { SHOT_KEYS, type ShotKey } from "./lingua-text";

const FILES = import.meta.glob<ImageMetadata>("../assets/lingua/*/*.{webp,jpg}", { eager: true, import: "default" });

export interface LinguaMedia {
  /** The hero: the first studied language's page, its pill and a word card open. */
  card: ImageMetadata;
  /** The hero on a phone: the card and the words around it. */
  closeUp: ImageMetadata;
  shots: Record<ShotKey, ImageMetadata>;
  poster: ImageMetadata;
  /** 1200×630, the `og:image` of the page. */
  share: ImageMetadata;
  /** The demo, from the site's root: `public/` serves it as it is. */
  video: string;
}

function file(lang: Lang, name: string): ImageMetadata {
  const image = FILES[`../assets/lingua/${lang}/${name}`];
  if (!image) {
    throw new Error(`lingua-media.ts: src/assets/lingua/${lang}/${name} is missing — run \`yarn capture\` in apps/lingua-extension`);
  }
  return image;
}

/**
 * The demo's address, once its file is known to exist under `public/`: it is only a path, so a page
 * built without its video would otherwise ship a dead `<source>` (change: add-site-italian-locale,
 * D6 — `/it/lingua/` is built as soon as en-it ships, and must not be built without its media).
 */
export function demoVideo(lang: Lang, root: string = process.cwd()): string {
  const path = `/media/lingua/${lang}/demo.mp4`;
  // From the site's root, where `astro build` and the tests run: the bundled module's own URL
  // would not lead back to `public/`.
  if (!existsSync(resolve(root, `public${path}`))) {
    throw new Error(`lingua-media.ts: public${path} is missing — run \`yarn capture\` in apps/lingua-extension`);
  }
  return path;
}

/** The captures of the page in `lang`. */
export function linguaMedia(lang: Lang): LinguaMedia {
  return {
    card: file(lang, "card.webp"),
    closeUp: file(lang, "card-closeup.webp"),
    shots: Object.fromEntries(SHOT_KEYS.map((key) => [key, file(lang, `${key}.webp`)])) as Record<ShotKey, ImageMetadata>,
    poster: file(lang, "demo-poster.webp"),
    share: file(lang, "share.jpg"),
    video: demoVideo(lang),
  };
}
