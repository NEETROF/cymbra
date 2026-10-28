// The public distribution channels of each Cymbra app. One source: the home hub,
// the product pages and the post-checkout `Downloads` block all read from here.
//
// Cymbra Music is live on both stores — the App Store record 6789557194 covers
// iOS, iPadOS and macOS, so one link serves the three. Cymbra Lingua is live on the
// Chrome Web Store, on Firefox Add-ons — desktop and Android, one listing, the manifest declaring
// `gecko_android` (apps/lingua-extension/tool/manifests.mjs) — and on the App Store, whose record
// 6813053825 (its Safari app) covers iPhone, iPad and Mac. The desktop builds of Music
// (Windows through the Microsoft Store, the Linux AppImage) live in the Music page's
// download section, which this button points at (change: add-music-download-page).

import type { Lang } from "./i18n";

export interface StoreLink {
  label: string;
  href: string;
  /** A published listing. `false` renders the button disabled (`.btn.disabled`). */
  live: boolean;
  /**
   * The store, for the Lingua page's choice of a primary button (change: add-site-lingua-showcase,
   * D11): `StoreButtons` writes it as `data-store` only when it is asked to choose.
   */
  store?: "chrome" | "firefox" | "safari";
  /** Another address for Android, which the choice points the button at there (Firefox Add-ons). */
  androidHref?: string;
}

export const MUSIC_APP_STORE = "https://apps.apple.com/app/id6789557194";
export const MUSIC_GOOGLE_PLAY = "https://play.google.com/store/apps/details?id=com.cymbra.music";
export const LINGUA_CHROME_WEB_STORE =
  "https://chromewebstore.google.com/detail/cymbra-lingua/lodgdmkjlbpieomelpdkfaifdbipfncd";
export const LINGUA_FIREFOX_ADDONS = "https://addons.mozilla.org/firefox/addon/cymbra-lingua/";
/** The same add-on's page in Firefox Add-ons for Android, where a phone installs it. */
export const LINGUA_FIREFOX_ANDROID = "https://addons.mozilla.org/android/addon/cymbra-lingua/";
export const LINGUA_APP_STORE = "https://apps.apple.com/app/id6813053825";

// The labels that differ by language; the store names do not.
const DESKTOP_LABEL = "Windows / Linux";
/** The Music page's download section, per language (`MusicDownloadsIsland`). */
export const MUSIC_DESKTOP_DOWNLOADS: Record<Lang, string> = {
  fr: "/music#telecharger",
  en: "/en/music#download",
  es: "/es/music#descargar",
};
const FIREFOX: Record<Lang, string> = {
  fr: "Firefox (ordinateur et Android)",
  en: "Firefox (desktop and Android)",
  es: "Firefox (escritorio y Android)",
};

/** Cymbra Music's store listings alone — where the desktop downloads are shown beside them. */
export function musicAppStores(): StoreLink[] {
  return [
    { label: "App Store (iOS, iPadOS, macOS)", href: MUSIC_APP_STORE, live: true },
    { label: "Google Play", href: MUSIC_GOOGLE_PLAY, live: true },
  ];
}

/** Where to get Cymbra Music, most-used platform first; the desktop button leads to the Music page's downloads. */
export function musicStores(lang: Lang): StoreLink[] {
  return [...musicAppStores(), { label: DESKTOP_LABEL, href: MUSIC_DESKTOP_DOWNLOADS[lang], live: true }];
}

/** Where to get Cymbra Lingua: its Safari app's one App Store record serves iPhone, iPad and Mac. */
export function linguaStores(lang: Lang): StoreLink[] {
  return [
    { label: "Chrome", href: LINGUA_CHROME_WEB_STORE, live: true, store: "chrome" },
    {
      label: FIREFOX[lang],
      href: LINGUA_FIREFOX_ADDONS,
      live: true,
      store: "firefox",
      androidHref: LINGUA_FIREFOX_ANDROID,
    },
    { label: "Safari (iPhone, iPad, Mac)", href: LINGUA_APP_STORE, live: true, store: "safari" },
  ];
}
