// The public distribution channels of each Cymbra app. One source: the home hub,
// the product pages and the post-checkout `Downloads` block all read from here.
//
// Cymbra Music is live on both stores — the App Store record 6789557194 covers
// iOS, iPadOS and macOS, so one link serves the three. The desktop builds
// (Windows / Linux) and the Lingua extension are not published yet: they render
// as dimmed buttons, the way the store buttons did before the listings existed.

import type { Lang } from "./i18n";

export interface StoreLink {
  label: string;
  href: string;
  /** A published listing. `false` renders the button disabled (`.btn.disabled`). */
  live: boolean;
}

export const MUSIC_APP_STORE = "https://apps.apple.com/app/id6789557194";
export const MUSIC_GOOGLE_PLAY = "https://play.google.com/store/apps/details?id=com.cymbra.music";

/** Where to get Cymbra Music, most-used platform first. */
export function musicStores(lang: Lang): StoreLink[] {
  return [
    { label: "App Store (iOS, iPadOS, macOS)", href: MUSIC_APP_STORE, live: true },
    { label: "Google Play", href: MUSIC_GOOGLE_PLAY, live: true },
    { label: lang === "fr" ? "Windows / Linux — bientôt" : "Windows / Linux — soon", href: "#", live: false },
  ];
}

/** Where Cymbra Lingua will be installed from. Nothing is published yet. */
export function linguaStores(_lang: Lang): StoreLink[] {
  return [
    { label: "Chrome", href: "#", live: false },
    { label: "Firefox", href: "#", live: false },
    { label: "Safari", href: "#", live: false },
  ];
}
