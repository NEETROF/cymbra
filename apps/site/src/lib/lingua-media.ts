// The Lingua page's captures (change: add-site-lingua-showcase, D7, D10), made by `yarn capture` in
// apps/lingua-extension (tool/capture/) and committed: per interface language, the WebP sources
// under `src/assets/lingua/<lang>/` (2560 px wide — the build makes the served sizes), the share
// image, the demo's poster, and the demo itself under `public/media/lingua/<lang>/demo.mp4`, served
// as it is. A page asks for its own language's set; a missing file fails the build, naming it.
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

/** The captures of the page in `lang`. */
export function linguaMedia(lang: Lang): LinguaMedia {
  return {
    card: file(lang, "card.webp"),
    closeUp: file(lang, "card-closeup.webp"),
    shots: Object.fromEntries(SHOT_KEYS.map((key) => [key, file(lang, `${key}.webp`)])) as Record<ShotKey, ImageMetadata>,
    poster: file(lang, "demo-poster.webp"),
    share: file(lang, "share.jpg"),
    video: `/media/lingua/${lang}/demo.mp4`,
  };
}
