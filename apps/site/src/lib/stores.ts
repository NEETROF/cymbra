// The public distribution channels of each Cymbra app. One source: the home hub,
// the product pages and the post-checkout `Downloads` block all read from here.
//
// Cymbra Music is live on both stores — the App Store record 6789557194 covers
// iOS, iPadOS and macOS, so one link serves the three. Cymbra Lingua is live on the
// Chrome Web Store, on Firefox Add-ons (desktop only) and on the App Store, whose record
// 6813053825 (its Safari app) covers iPhone, iPad and Mac. The desktop builds of Music
// (Windows / Linux) are not published yet: they render as dimmed buttons, the way the
// store buttons did before the listings existed.

import type { Lang } from "./i18n";

export interface StoreLink {
  label: string;
  href: string;
  /** A published listing. `false` renders the button disabled (`.btn.disabled`). */
  live: boolean;
}

export const MUSIC_APP_STORE = "https://apps.apple.com/app/id6789557194";
export const MUSIC_GOOGLE_PLAY = "https://play.google.com/store/apps/details?id=com.cymbra.music";
export const LINGUA_CHROME_WEB_STORE =
  "https://chromewebstore.google.com/detail/cymbra-lingua/lodgdmkjlbpieomelpdkfaifdbipfncd";
export const LINGUA_FIREFOX_ADDONS = "https://addons.mozilla.org/firefox/addon/cymbra-lingua/";
export const LINGUA_APP_STORE = "https://apps.apple.com/app/id6813053825";

// The labels that differ by language; the store names do not.
const DESKTOP_SOON: Record<Lang, string> = {
  fr: "Windows / Linux — bientôt",
  en: "Windows / Linux — soon",
  es: "Windows / Linux — próximamente",
};
const FIREFOX_DESKTOP: Record<Lang, string> = {
  fr: "Firefox (ordinateur)",
  en: "Firefox (desktop)",
  es: "Firefox (ordenador)",
};

/** Where to get Cymbra Music, most-used platform first. */
export function musicStores(lang: Lang): StoreLink[] {
  return [
    { label: "App Store (iOS, iPadOS, macOS)", href: MUSIC_APP_STORE, live: true },
    { label: "Google Play", href: MUSIC_GOOGLE_PLAY, live: true },
    { label: DESKTOP_SOON[lang], href: "#", live: false },
  ];
}

/** Where to get Cymbra Lingua: its Safari app's one App Store record serves iPhone, iPad and Mac. */
export function linguaStores(lang: Lang): StoreLink[] {
  return [
    { label: "Chrome", href: LINGUA_CHROME_WEB_STORE, live: true },
    { label: FIREFOX_DESKTOP[lang], href: LINGUA_FIREFOX_ADDONS, live: true },
    { label: "Safari (iPhone, iPad, Mac)", href: LINGUA_APP_STORE, live: true },
  ];
}
