# add-site-lingua-showcase — show Lingua at work on the site, and give the stores pictures made from the real extension

## Why

Guillaume, someone who lands on `cymbra.app/lingua/` today reads words only. The page has no
picture of the extension, no video, and three store buttons of equal weight
(`apps/site/src/components/LinguaPage.astro`, `src/components/StoreButtons.astro` with
`primary` unset). A link to the page shared on Discord or in a message shows no preview image:
`src/layouts/Base.astro` writes `og:title`, `og:description` and `og:type`, and no `og:image`,
`og:url` or canonical link. Search engines get no `robots.txt` and no `sitemap.xml` — both answer
404 on the live site (checked on 2026-10-13) — and every `hreflang` points at an address the host
redirects (`https://cymbra.app/lingua` answers 308 to `/lingua/`).

Three things on the site are out of date or wrong:

- The French home page says « Lisez le web en anglais » (`src/pages/index.astro`, a literal),
  while French speakers study English **and** Spanish. The English and Spanish home pages are
  already built from the shipped pairs; the French one is not, and `apps/site/README.md` says it
  needs no builder, which the text contradicts.
- The page and its store button say « Firefox (ordinateur) » (`src/lib/lingua-text.ts`,
  `src/lib/stores.ts`). Firefox for Android works: the manifest declares `gecko_android` since
  #599 (`apps/lingua-extension/tool/manifests.mjs`), and AMO lists 1.9.0 as compatible with
  Android 128 and later.
- Nothing on the page shows that the extension also reads your EPUB books, or what a review card
  looks like.

The store listings have the same problem. `apps/lingua-extension/STORE-LISTING.md` says that
screenshots "cannot be generated from the repository", so every set is a manual chore across three
interface languages; its screenshot plan still shows "the four answers ("Again", "Hard", "Good",
"Easy")" while the review has had two answers since `refine-lingua-review-session` — "Didn't know" /
"Knew it" (`src/i18n/en/review.ts`), « Pas su » / « Su », « No la sabía » / « La sabía ». Its
descriptions say "Firefox for desktop" twice, for the platforms and for extended translation,
which also runs on Firefox for Android (`apps/lingua-extension/TRANSLATION.md`, "What never
happens"). `apps/lingua-apple/STORE-LISTING.md` repeats the four answers in its screenshot plan.

None of this needs a new feature in the extension. It needs pictures of the real one, a page that
shows them, and listing texts that match what a reader sees.

## What Changes

- **A capture command, run on your Mac, not in CI** (`apps/lingua-extension`, `yarn capture`):
  it builds the Chromium package into a folder of its own, with the development switch the build
  already has to run the reader on every page (`LINGUA_ALL_URLS=1`), loads it unpacked in Playwright's
  Chromium (Chrome for Testing — branded Chrome ignores `--load-extension`), with Chrome's field
  trials left on (Playwright's `--disable-field-trial-config` is what hid the blank EPUB reader on
  2026-09-24), opens committed article pages — one per studied language, English, Spanish and
  French, written for the purpose — and a public-domain EPUB, restores a committed reader state
  through the extension's own restore, and captures, in each interface language (fr, en, es):
  the highlighted page with its percentage pill and a word card open, a page in the reader's
  second studied language, a review card in the review panel the extension draws in the page (the
  one Firefox readers use), the statistics ladder and the EPUB reader. It writes
  the site's images, 1280×800 PNGs without alpha for the Chrome Web Store and addons.mozilla.org,
  a share image, and a silent demo video of about 30 seconds with its poster. Safari and App Store
  captures stay manual.
- **The Lingua page shows the product**: a capture in the hero, then the video and four captures
  further down, each sized, lazy-loaded below the first screen, with alt text in the page's
  language; the video has a poster and no sound, and never starts on its own for a visitor who asks
  for reduced motion or saves data.
- **One store button is primary for the visitor's browser** (a few lines of script; without it the
  three buttons stay equal): Safari on iPhone, iPad and Mac, and any browser on iPhone or iPad →
  App Store; Firefox, desktop or Android → Firefox Add-ons (its Android page on Android); Chrome,
  Edge and other Chromium browsers on a computer → Chrome Web Store; another Android browser →
  Firefox, with a line saying Lingua runs in Firefox there. The page names Firefox for Android.
  Nothing is stored or sent: the site keeps no analytics.
- **Every page gets a share preview and a canonical address**: `og:image` (1200×630) with its
  alt text, `twitter:card` `summary_large_image`, `og:url`, `og:site_name`, and a canonical link;
  `hreflang` names the address the host serves. The site publishes `robots.txt` and a
  `sitemap.xml` generated after the build from the pages themselves.
- **The French home page names English and Spanish**, built from the shipped pairs like the
  English and Spanish ones.
- **A weight budget** for the site's images and video, checked by the build test `site-check`
  already runs.
- **The listing texts** (`apps/lingua-extension/STORE-LISTING.md`, `apps/lingua-apple/STORE-LISTING.md`):
  the review's two answers, Firefox for Android, the screenshot plan pointing at the capture
  command. You paste them by hand, as always.

## Capabilities

### New Capabilities

- `site-discoverability`: what search engines and link previews read on every page of the
  site — the canonical address, the share preview, `robots.txt` and `sitemap.xml`. No existing
  capability covers it: `site-locales` (open in `add-site-spanish-locale`) names a page's
  translations, `site-lingua-page` one page's content.

### Modified Capabilities

- `site-lingua-page`: ADDED *The Lingua page shows the extension at work*, *The Lingua page points
  each visitor to the store of their browser*, *The Lingua page's media stay within a weight budget*.
  The requirement held by `add-site-lingua-matrix-pages` is not modified.
- `site-locales`: ADDED *The French home page describes Lingua for the readers it serves*, beside
  the English and Spanish home pages' requirements (`add-lingua-french-listings`,
  `extend-site-spanish-locale`).
- `lingua-browser-extension`: ADDED *Store and site captures come from the real extension, by one
  command* and *The store listings describe the extension as it is*. The requirement *The store
  listings name each studied language*, held by three open changes, is not modified.

## Impact

- **Products.** Lingua: a local tool under `apps/lingua-extension/tool/capture/` and its fixtures,
  a `capture` build output (`dist-capture/`, through a new output-folder option of `build.mjs`)
  that no package, release or store upload reads; the
  shipped packages do not change by a byte. Site: `Base.astro`, `LinguaPage.astro`,
  `StoreButtons.astro`, `lingua-text.ts`, `stores.ts`, the French home page, a sitemap step in
  `astro.config.mjs`, `public/robots.txt`, images under `src/assets/`, videos under
  `public/media/`. Music: consumed only — its committed macOS store capture of each language
  becomes its page's share image. ID and the back office: untouched.
- **Listings.** Text only, pasted by the owner. No `_locales` summary changes.
- **CI.** `site-check`'s filter gains `apps/music/store/macos/desktop_1440x900/**`; its existing
  `check:routes` step runs the new post-build tests. The capture command is never run in CI.
- **Order.** Every delta is ADDED, so the change archives in any order (no `archiveAfter`). The site goes out
  with `site-deploy`, by the owner, under the deploy rule of `update-lingua-privacy-annex-languages`
  (task 7.1).
- **Not here.** Translation shown in captures (it downloads a model); a Chrome Web Store video
  (YouTube only); Safari and App Store captures; site analytics of any kind.
