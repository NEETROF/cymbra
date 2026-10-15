// Which store serves the visitor's browser (change: add-site-lingua-showcase, D11). Read in the
// visitor's browser by the Lingua page's store buttons (`StoreChoice.astro`), from what the browser
// says of itself; nothing is stored, sent or logged. Pure, so that a table of user agents tests it.

/** The store a browser installs Cymbra Lingua from. */
export type Store = "chrome" | "firefox" | "firefox-android" | "safari";

/** A line the page adds when the visitor's browser cannot run Lingua where they are. */
export type StoreHint = "ios" | "android";

/** What the browser says of itself: `navigator.userAgent`, `userAgentData.brands`, `platform`, `maxTouchPoints`. */
export interface BrowserFacts {
  userAgent: string;
  /** The client hints' brands (Chromium only), when the browser gives them. */
  brands?: readonly { brand: string }[];
  platform?: string;
  maxTouchPoints?: number;
}

export interface StoreChoice {
  store: Store | null;
  hint: StoreHint | null;
}

/** Browsers on iOS and iPadOS other than Safari name themselves in the user agent. */
const IOS_OTHER_BROWSER = /\b(CriOS|FxiOS|EdgiOS|OPiOS|OPT|GSA|YaBrowser|DuckDuckGo|Brave)\//;
const CHROMIUM_BRANDS = /^(Chromium|Google Chrome|Microsoft Edge|Brave|Opera|Vivaldi)$/;
const CHROMIUM_UA = /\b(Chrome|Chromium|Edg|OPR|Vivaldi)\//;
const DESKTOP_UA = /\b(Windows NT|Macintosh|X11|Linux x86_64|CrOS)\b/;

/**
 * The store of the visitor's browser, and whether a hint is due:
 * - an iPhone or an iPad (iPadOS says it is a Mac with a touch screen): the App Store, with a hint in
 *   a browser other than Safari — Lingua runs in Safari there;
 * - Android: Firefox Add-ons' Android listing, with a hint in a browser other than Firefox;
 * - Firefox on a computer: Firefox Add-ons;
 * - Safari on a Mac: the App Store;
 * - Chrome, Edge, Brave, Opera, Vivaldi on a computer (client hints, else the user agent): the Chrome
 *   Web Store;
 * - anything else: none.
 */
export function pickStore({ userAgent: ua, brands, platform = "", maxTouchPoints = 0 }: BrowserFacts): StoreChoice {
  const ipadAsMac = (/\bMacintosh\b/.test(ua) || platform === "MacIntel") && maxTouchPoints > 1;
  if (/\b(iPhone|iPad|iPod)\b/.test(ua) || ipadAsMac) {
    return { store: "safari", hint: IOS_OTHER_BROWSER.test(ua) ? "ios" : null };
  }
  if (/\bAndroid\b/.test(ua)) {
    return { store: "firefox-android", hint: /\bFirefox\//.test(ua) ? null : "android" };
  }
  if (/\bFirefox\//.test(ua) && DESKTOP_UA.test(ua)) return { store: "firefox", hint: null };
  const chromium = brands?.some(({ brand }) => CHROMIUM_BRANDS.test(brand)) ?? CHROMIUM_UA.test(ua);
  if (chromium && DESKTOP_UA.test(ua)) return { store: "chrome", hint: null };
  if (/\bMacintosh\b/.test(ua) && /\bVersion\/[\d.]+.*\bSafari\//.test(ua)) return { store: "safari", hint: null };
  return { store: null, hint: null };
}
