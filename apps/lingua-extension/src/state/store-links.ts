// Where Cymbra Lingua is listed, and where each build's readers rate it (add-lingua-comeback-hints
// D12). The site holds the same listings (apps/site/src/lib/stores.ts), which the extension's build
// does not reach: test/store-links.spec.ts reads that file and checks the ids are the same, so a
// listing moved on the site fails here. Opened only when the reader asks, by the background, so no
// address of a page being read goes with it.

/** The Chrome Web Store item id. */
export const CHROME_WEB_STORE_ID = "lodgdmkjlbpieomelpdkfaifdbipfncd";
/** The Firefox Add-ons slug. */
export const FIREFOX_ADDONS_SLUG = "cymbra-lingua";
/** The App Store record of the Safari app (iPhone, iPad and Mac). */
export const APP_STORE_ID = "6813053825";

export const CHROME_WEB_STORE_LISTING = `https://chromewebstore.google.com/detail/cymbra-lingua/${CHROME_WEB_STORE_ID}`;
export const FIREFOX_ADDONS_LISTING = `https://addons.mozilla.org/firefox/addon/${FIREFOX_ADDONS_SLUG}/`;
export const APP_STORE_LISTING = `https://apps.apple.com/app/id${APP_STORE_ID}`;

/** The build targets, as `__TARGET__` names them. */
export type BuildTarget = typeof __TARGET__;

/**
 * Where a build's reader rates it: the Chrome Web Store listing's reviews; the Firefox Add-ons
 * listing, whose « Rate your experience » takes the rating (AMO redirects to the reader's locale);
 * the App Store's write-review page, which Apple's own sample opens from an interface action.
 */
export function reviewPageFor(target: BuildTarget): string {
  switch (target) {
    case "firefox":
      return FIREFOX_ADDONS_LISTING;
    case "safari":
      return `${APP_STORE_LISTING}?action=write-review`;
    default:
      return `${CHROME_WEB_STORE_LISTING}/reviews`;
  }
}

/** This build's review page. */
export const STORE_REVIEW_PAGE = reviewPageFor(__TARGET__);
