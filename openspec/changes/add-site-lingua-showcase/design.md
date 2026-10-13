# Design — add-site-lingua-showcase

## Context

See proposal.md (Why). What exists, read on `main` at e07d23bd3:

| Where | What |
|---|---|
| `apps/site/src/layouts/Base.astro` | `<title>`, `description`, `og:title`, `og:description`, `og:type`; one `hreflang` per `alternates` entry, written as declared (`/lingua`, no trailing slash); `noindex` for personal pages; "the layout ships no JS" |
| `apps/site/src/components/LinguaPage.astro` | hero (kicker, headline, tagline, `StoreButtons`, hero note), feature cards, coverage table, `Community`, closing; every text slot through `set:html` from `src/lib/lingua-text.ts` (one table per site language) |
| `apps/site/src/components/StoreButtons.astro` | a row of links; `primary` marks the first live link, the Lingua page leaves it unset |
| `apps/site/src/lib/stores.ts` | `linguaStores(lang)`: Chrome, « Firefox (ordinateur) » / "Firefox (desktop)" / « Firefox (escritorio) », Safari; a comment says AMO is "desktop only" |
| `apps/site/src/pages/index.astro` | the French home's Lingua card, literal: « Lisez le web en anglais… » |
| `apps/site/src/pages/{en,es}/index.astro` | Lingua cards built from the shipped pairs (`englishHomeLinguaCard`, `spanishHomeLinguaCard`) |
| `apps/site/astro.config.mjs` | `site: 'https://cymbra.app'`, a `localised404` build hook; no sitemap |
| `apps/site/public/` | icons and the Cymbra ID brand mark; no `robots.txt` |
| `apps/site/yarn.lock` | `sharp@0.35.4` (Astro's image service) already resolved |
| `.github/workflows/site-check.yml` | `check`, `typecheck`, `test`, `build`, then `check:routes` (the post-build tests in `test/post-build/`) |
| `apps/lingua-extension/build.mjs`, `tool/manifests.mjs` | Chromium: `<all_urls>` is an **optional** host permission; `syncReaderRegistration` registers the reader on every page once it is granted (`src/background.ts`), on install too; the dev key pins the id `figfjglfdiffocldficbimecjnhnkhkh`; Firefox: `gecko_android` declared (#599) |
| `apps/lingua-extension/src/reading/settings-view.ts` | Réglages › Données › Restaurer: a file input, `port.restore(text)`, then `saveRestoredBackup` |
| `apps/lingua-extension/src/state/dailystats.ts` | daily counts under `cymbra-lingua-daily-v4`, not part of a backup |
| `apps/lingua-extension/STORE-LISTING.md` | "Screenshots cannot be generated from the repository"; plan of five shots per listing language; "the four answers"; "Firefox for desktop" |
| `apps/music/tool/capture_store_screenshots.sh`, spec `music-store-listing-capture` | Music's precedent: one command per platform and locale, seeded state, PNGs committed under `apps/music/store/` |
| `apps/back-office/package.json` | `@playwright/test ^1.62.0`; no Playwright in `apps/lingua-extension` |

Facts the decisions lean on, checked rather than remembered:

- Branded Google Chrome (137 and later) ignores `--load-extension`; Playwright's Chromium (a Chrome
  for Testing build) honours it, with a persistent context.
- Playwright starts Chromium with `--disable-field-trial-config`: on 2026-09-24 the EPUB reader
  was blank for the owner and green under Playwright, because a Chrome field trial isolated its
  `blob:` sections. The reader now serves sections from the service worker, but a capture that
  turns trials off would still not show Chrome as readers run it.
- Chrome's side panel is browser UI, not a page: a page screenshot does not include it. Its
  document is a DevTools target of its own (reached that way on 2026-10-06 to debug #717).
- Cloudflare Pages: 25 MiB per file, 20,000 files per site
  (developers.cloudflare.com/pages/platform/limits).
- Chrome Web Store screenshots: 1280×800 or 640×400, no alpha, up to five. addons.mozilla.org
  recommends 1280×800 (1.6:1). Both take the same set.
- Live site, 2026-10-13: `/robots.txt` and `/sitemap.xml` answer 404; `/lingua` answers 308 to
  `/lingua/`; the Firefox button reads « Firefox (ordinateur) ».

## Goals / Non-Goals

**Goals:**
- A visitor sees the extension at work, in their language, before reading a paragraph.
- One command makes every capture the site and two of the three stores need, from the shipped
  code, in fr, en and es.
- Shared links and search results show the right page, with an image.
- The listings say what the product does today.

**Non-Goals:**
- Any change to the shipped packages, their permissions or their code.
- Safari, iPhone, iPad and Mac App Store captures (simulators and Safari stay manual).
- Extended translation in captures (it downloads a model from `models.cymbra.app`; the capture
  stays offline).
- Site analytics, A/B tests, cookies, or any third-party request from the site.
- A Chrome Web Store promo video (that field takes a YouTube link only).
- Captions or marketing text drawn inside the images.

## Decisions

### D1 — A capture build that has the global grant, nothing else different

`node build.mjs chromium --capture` (behind `yarn capture`) writes `dist-capture/`: the Chromium
variant's bundles, byte for byte, and a manifest whose `<all_urls>` is a host permission instead
of an optional one. Chrome grants a required host permission at load, `onInstalled` runs
`syncReaderRegistration`, and the reader is registered on every page — the path of a reader who
said yes to « Toujours surligner ». `dist-chromium/`, `dist-firefox/`, `dist-safari/`, the release
workflow and `check:variants` never read `dist-capture/`; a unit test holds that every file but
`manifest.json` is identical to `dist-chromium/`'s, and that the two manifests differ by that
permission alone. It is a folder of its own so that a capture never rebuilds the `dist-chromium/`
a dogfood browser has loaded.

Rejected: clicking « Analyser cette page » (the popup is browser UI; opened as a tab, `popup.html`
is its own active tab); accepting the permission prompt (browser UI); the extension's keyboard
commands (keys sent through DevTools reach the page, not Chrome's shortcut handler).

### D2 — Playwright's Chromium, with Chrome's field trials on

`apps/lingua-extension` gains `playwright-core` as a dev dependency, pinned to the back office's
Playwright version so both use one cached browser. The tool calls
`chromium.launchPersistentContext` (extensions need one) with `--disable-extensions-except` and
`--load-extension` on `dist-capture/`, and `ignoreDefaultArgs` removing
`--disable-field-trial-config` (and `--disable-extensions` if the pinned version passes it). It
never asks for `channel: 'chrome'`. It runs headed by default — a window opens on the Mac for a
few minutes; `--headless` exists once task 1.1 shows the side panel renders in Chromium's headless
mode. The tool prints `chrome://version`'s command line in its log, so the owner can see which
flags the captures ran with.

Rejected: Puppeteer (a second browser toolchain in the repo); Firefox through WebDriver BiDi for
the AMO set (the panel's page is the same document on both, D6, and the in-page parts are the
same code).

### D3 — Fixtures: our own articles and public-domain books, served locally

`tool/capture/fixtures/pages/{en,es,fr}.html`: one article per studied language, about 450 words,
written for the purpose on one topic (the same subject in the three languages, so the captures
read as one story), at a B1–B2 vocabulary so a B1 reader sees a sensible share highlighted. Plain
neutral typography, no publication name, logo or brand of anyone else. Licensed as the
repository (Apache-2.0), which `tool/capture/fixtures/README.md` states.

`tool/capture/fixtures/books/`: one chapter per studied language from a public-domain work —
English: Jane Austen, *Pride and Prejudice* (1813), chapter 1; Spanish: Benito Pérez Galdós,
*Marianela* (1878), chapter 1; French: Jules Verne, *Le Tour du monde en quatre-vingts jours*
(1872), chapter 1. Authors dead more than 100 years; the text committed without any distributor's
header or licence, the source edition named in the README. The tool assembles each into an EPUB 3
with `@zip.js/zip.js` (already a dependency), as `test/epub-fixtures.ts` builds its test books.

A tiny Node server on `127.0.0.1` (ephemeral port) serves the pages: content scripts do not run on
`file://` without a setting. For the video only, each page draws a visible pointer that follows
the mouse, because a recorded Playwright video shows none.

Rejected: Wikipedia articles (CC BY-SA: attribution and share-alike on every store image, and the
text moves under us); live news sites (other people's branding in our store images, and nothing
reproducible).

### D4 — The reader's state: restored from a committed backup, statistics written for the run's dates

Per interface language, `tool/capture/fixtures/state/{fr,en,es}.json` is a backup file of the
extension's own format: the native language (which sets the interface language), two studied
languages (fr: English then Spanish; en: Spanish then French; es: English then French), a level
for each (B1), about twelve cards with the sentences they were captured in, a few dozen words
marked known. The tool restores it the way a reader does: the panel's page opened in a tab,
Réglages › Données › Restaurer, `setInputFiles` on its input — `port.restore` and
`saveRestoredBackup`, nothing written behind the extension's back.

The files are recorded by the product itself: `yarn capture --record-seed` drives the real UI on
the fixture pages (choose the level, mark words known, add cards, answer a short review) and saves
through Réglages › Données › Sauvegarder. They are re-recorded only when the fixtures change; an
older backup is migrated forward on restore, as a reader's is.

Daily statistics are not in a backup. The tool writes the 14 days before the run under
`cymbra-lingua-daily-v4` through the store's own message (`{ type: "store:set", items }`, sent
from an extension page, as `src/state/store.ts`'s messaged area does), so the statistics page
shows a believable fortnight whatever day the owner runs it. The days come from a pure function
whose output a unit test reads back with `dailystats.ts`'s own reader.

Rejected: a hand-written backup (the engine's lossless format is written by Rust and would drift);
clicking the whole state into existence at every run (slow, and words read are counted after a
dwell, so the figures would move from run to run).

### D5 — The shots

Viewport 1280×800, device scale factor 2. Per interface language:

| Key | What | fr | en | es |
|---|---|---|---|---|
| `card` | The first studied language's article, highlighted, the pill showing the share known, the word card open on a verb form whose card names its tense | English page | Spanish page | English page |
| `second` | The second studied language's article, highlighted, the card open (the card names tense and gender) | Spanish | French | French |
| `review` | The article beside the review panel, a card revealed with its sentence and the two answers (D6) | English | Spanish | English |
| `stats` | The statistics page, the first studied language's ladder and the fortnight | English | Spanish | English |
| `reader` | The EPUB reader on the first studied language's book, highlighted | Austen | Galdós | Austen |
| `reader-eink` | The same page in the « E-ink contrasté » preset (open question 3) | idem | idem | idem |

The Chrome Web Store and AMO set is `card`, `second`, `review`, `stats`, `reader`, in that order —
five, the Chrome Web Store's cap. The reader's two languages are shown by `card` and `second`, so
no slot is spent on "the French capture" alone, which `STORE-LISTING.md` planned as the fifth
before the reader existed (open question 4).

### D6 — The review panel, captured where Chrome draws it

The side panel is opened with a user gesture (`Runtime.evaluate` with `userGesture: true` on an
extension page: `chrome.sidePanel.open`), then captured through its own DevTools target
(`Page.captureScreenshot` on the `sidepanel.html` target). The tool composes it beside the page,
with no browser frame drawn: the page 920 px wide, the panel 360 px, 1280×800 in all. The panel's
document is the page Firefox shows in its sidebar (`sidebar_action`, same page — spec *Firefox
variant*), so the composite is true for both stores.

If task 1.1 finds the target cannot be captured, the fallback is the panel's page opened in a
second page 360 px wide with the review language set (`cymbra-lingua-review-language`), composed
the same way. Rejected: drawing a panel that is not the product's.

### D7 — Sizes, formats and where each output goes

| Output | Format | Where | Committed |
|---|---|---|---|
| Site images | WebP, quality 90, at 2× (2560 px wide, crops for the close-up) | `apps/site/src/assets/lingua/{fr,en,es}/<key>.webp` | yes: the site's build needs them |
| Share image | JPEG, 1200×630, cropped from `card` | `apps/site/src/assets/lingua/{fr,en,es}/share.jpg` | yes |
| Demo video and poster | MP4 (D8), WebP poster | `apps/site/public/media/lingua/{fr,en,es}/demo.mp4`, poster beside the images | yes |
| Store captures | PNG 1280×800, RGB, no alpha | `apps/lingua-extension/store-captures/{fr,en,es}/1-card.png` … `5-reader.png` | no, git-ignored (open question 5) |
| Small promo tile | PNG 440×280, no alpha, from `card` | same folder | no |

The store PNGs are scaled down from the 2× capture (sharper than a 1× capture) and the tool fails
if a PNG's header says it has an alpha channel (colour type 2 required), naming the file. Encoding
is ffmpeg's — present on the owner's Mac; the tool checks for it first and prints
`brew install ffmpeg` otherwise — so the extension gains no native npm dependency.

The site serves the images through `astro:assets` `<Picture>`: AVIF and WebP at 640, 960, 1280 and
1920 px, with `width`/`height` so nothing shifts while they load.

### D8 — The demo video

About 30 seconds, per interface language, on the first studied language's article:

| Seconds | What |
|---|---|
| 0–4 | The page loads; the unknown words light up; the pill shows the share known |
| 4–11 | The pointer goes to a highlighted verb; the card opens, with its tense and its gloss |
| 11–15 | « + Deck » (or "+ Deck", « + Mazo »): the word joins the deck |
| 15–21 | « Je connais » on another word: its highlight goes, the pill's share rises |
| 21–30 | The statistics page: the ladder |

Recorded with Playwright's `recordVideo` at 1280×800, then encoded by ffmpeg to H.264 MP4 —
`-an` (no audio track at all), `+faststart`, `yuv420p` — under 4 MB. The poster is the frame with
the card open. If `recordVideo`'s own encoding is too soft once re-encoded (task 1.1 looks), the
fallback is DevTools' screencast frames assembled by ffmpeg.

MP4 only: every browser the extension runs in plays H.264; a Chromium built without proprietary
codecs shows the poster and the controls. Rejected: a WebM beside it (doubles the weight for few
visitors); YouTube or Vimeo (third-party requests and cookies, against the home page's « aucun
pisteur publicitaire »); a GIF (several times heavier for worse pictures).

### D9 — The video starts only when it should

`<video muted playsinline controls preload="none" poster>` — no `autoplay` attribute, so without
script it is a poster with a play button and downloads nothing until pressed. A few lines of
script play it, muted, once at least half of it is on screen, and pause it when it leaves —
only when `prefers-reduced-motion` is `no-preference` and `navigator.connection.saveData` is not
true. It plays once; the controls replay it. Rejected: the `autoplay` attribute (it ignores
reduced motion and costs a mobile visitor 4 MB they did not ask for).

### D10 — The page's layout

- **Hero**: `card` (open question 1) beside or under the store buttons, `loading="eager"`,
  `fetchpriority="high"`; under 640 px wide a close-up of the card and the words around it
  (`<picture>` with a media source), so a phone shows a card it can read.
- **A new section after the hero** — « En images » / "See it at work" / « Míralo en acción »: the
  video first, then `second`, `review`, `stats` and `reader`, each a `<figure>` with a one-line
  caption; `loading="lazy"`, `decoding="async"`.
- Then the feature cards, the coverage table, the community band and the closing, unchanged.

Every alt text and caption is a slot of `lingua-text.ts`'s three tables, named by shot key, and
`test/lingua-text.spec.ts` holds each table to every key. Alt texts describe what is shown in the
page's language and quote no figure the seed could change.

### D11 — The store button for the visitor's browser

`src/lib/store-choice.ts` exports one pure function, `pickStore({ userAgent, brands, platform,
maxTouchPoints })` → `"safari" | "firefox" | "firefox-android" | "chrome" | null`, and whether a
hint is due:

| Visitor | Primary | Hint |
|---|---|---|
| Safari on a Mac | App Store | — |
| iPhone or iPad, any browser (iPadOS reports a Mac with touch points) | App Store | in a browser other than Safari: « Sur iPhone et iPad, Lingua s'utilise dans Safari. » |
| Firefox on a computer | Firefox Add-ons | — |
| Firefox on Android | Firefox Add-ons, its Android page | — |
| Another Android browser | Firefox Add-ons, its Android page | « Sur Android, Lingua s'utilise dans Firefox. » |
| Chrome, Edge, Brave, Opera, Vivaldi on a computer (client hints, else the user agent) | Chrome Web Store | — |
| Anything else | none | — |

`StoreButtons` gains a `choose` prop, used by the Lingua page only: each button carries
`data-store`; a bundled module script marks the chosen one `btn-primary`, points the Firefox link
at `https://addons.mozilla.org/android/addon/cymbra-lingua/` on Android, and reveals the hint. The
buttons keep their order (no jump), and nothing is stored, sent or logged. Without script, the
three are as today. The Firefox label becomes « Firefox (ordinateur et Android) » / "Firefox
(desktop and Android)" / « Firefox (escritorio y Android) », and the hero note's platform sentence
says the same in each table. The Music page's and the post-checkout `Downloads`' buttons do not
change.

Rejected: choosing on the server (the site is static; a Pages Function would add a runtime for one
class name); reordering the buttons (the row jumps after load); hiding the other stores (a visitor
on a borrowed computer still needs them).

### D12 — One address form in the head, and a share image on every page

`Base.astro` gains `image?: { src: ImageMetadata; alt: string }` and writes, for every page:
`og:image` (a 1200×630 JPEG made by `getImage`), `og:image:width`, `og:image:height`,
`og:image:alt`, `og:site_name` « Cymbra », `twitter:card` `summary_large_image`; and for every page
without `noindex`: `<link rel="canonical">` and `og:url`. One function, `servedUrl(path)`, turns
a declared path into the address the host serves — `https://cymbra.app` + the path with a trailing
slash (`/` stays `/`) — and every `hreflang`, the canonical and `og:url` go through it. The language
switch links keep their declared paths.

Which image: the Lingua pages their `share.jpg` of their language; the Music pages Music's own
committed capture of their language (`apps/music/store/macos/desktop_1440x900/{fr,en,es}/01_library.png`,
cropped); every other page a language-neutral site image — the name « Cymbra » on the brand
gradient, drawn as paths in an SVG under `src/assets/` so that no font is needed to rasterise it.

Rejected: `trailingSlash: 'always'` (it moves every internal link, and pinned consumers request
`/account` without the slash); `build.format: 'file'` (it moves the served paths of pinned routes);
WebP share images (some link previewers still read JPEG or PNG only).

### D13 — `sitemap.xml` from the built pages, `robots.txt` by hand

An `astro:build:done` step beside `localised404` reads every built HTML page, keeps those without
`<meta name="robots" content="noindex">`, and writes `dist/sitemap.xml`: one `<url>` per page, its
`<loc>` the page's canonical, one `<xhtml:link rel="alternate" hreflang>` per `hreflang` the page
declares. The build fails if an indexable page has no canonical. No `lastmod`: no page has a date
we could trust. A new page enters the sitemap by existing; a personal page stays out by being
`noindex`, as it already is.

`public/robots.txt`: every agent allowed, and `Sitemap: https://cymbra.app/sitemap.xml`. The
personal pages are not disallowed: a crawler must be able to read their `noindex`.

Rejected: `@astrojs/sitemap` (a new dependency, and it derives alternates from paths, while this
site's twins have their own slugs: `/confidentialite/` ↔ `/en/privacy/` ↔ `/es/privacidad/`); a
hand-kept list (it drifts).

### D14 — The weight budget, in the post-build tests

`test/post-build/media-budget.spec.ts`, run by `yarn check:routes` (already a `site-check` step,
so no new CI step): each served image variant ≤ 250 KB, the hero's largest variant ≤ 200 KB, each
share image ≤ 300 KB, each poster ≤ 150 KB, each video ≤ 4 MB, all images, posters and videos
together ≤ 30 MB, no file in `dist/` of 25 MiB or more, fewer than 20,000 files. A failure names the
file and its size. The numbers live in one table at the top of the file, and `apps/site/README.md`
points at it.

Rejected: Lighthouse in CI (a browser and the network in a gate, for numbers a file size answers).

### D15 — The French home's Lingua card from the shipped pairs

`frenchHomeLinguaCard(pairs)` in `lingua-text.ts`, as its English and Spanish siblings: the studied
languages of the pairs glossed in French, in `packs.json`'s order — « Lisez le web en anglais ou en
espagnol, les mots que vous ne connaissez pas encore sont surlignés sur place… » today — the rest
of the paragraph unchanged, the button to `/lingua`. `apps/site/README.md`'s sentence that the
French card "is literal" goes.

### D16 — The listings and their screenshot plans

`apps/lingua-extension/STORE-LISTING.md`: the Graphics section says screenshots come from
`yarn capture` (prerequisites: `yarn gen:wasm`, `yarn gen:pack:real`, `yarn fetch:engine`), lists
D5's five shots per listing language with the review's two answers as each interface names them,
and says where the files land; « Firefox pour ordinateur » / "Firefox for desktop" / « Firefox para
escritorio » become desktop and Android in the platform line and the extended-translation line of
the FR, EN and both ES descriptions, every count re-measured; the Extra fields' note that the
homepage is "only once the site is deployed" goes, the page being live. The claims table (« Each
claim with the file that makes it true ») gains the Android line's file (`tool/manifests.mjs`,
`gecko_android`).

`apps/lingua-apple/STORE-LISTING.md`: the screenshot plan's two answers; a line that the App Store
slots, macOS's 1280×800 included, are not filled from the Chromium captures — they show Chrome's
panel, and App Review asks for the app in use (Guideline 2.3.3).

No `_locales` summary changes, so the extension's version check is untouched.

## Risks / Trade-offs

- **Repository weight.** Three videos (≈ 12 MB) and ≈ 25 site sources (≈ 6 MB) per refresh, kept
  in history. Mitigated by refreshing only when the interface visibly changes and by D14's caps;
  store PNGs are not committed (open question 5).
- **Panel capture.** If neither D6 path works, the review shot is the in-page parts only and the
  set has four shots until a follow-up; task 1.1 settles it before anything else is built.
- **The fixtures are ours.** A capture of our own article is honest about the product but not about
  "any website"; the listings and page say "the page you read", never name a site.
- **Seeded numbers.** The pill, the ladder and the fortnight show a plausible reader's figures, not
  a claim; alt texts and captions quote none.
- **Browser drift.** A new Playwright or Chrome changes pixels; the captures are regenerated, not
  diffed. Nothing in CI depends on them but their size.
- **Copy.** The English and Spanish alt texts, captions and hints are drafts (M9); the owner reads
  them.
- **Privacy.** The site gains a script that reads the user agent in the visitor's browser and
  sends nothing; no request leaves the page that did not before, except the self-hosted video when
  it plays.

## Migration Plan

1. Task 1.1's spike decides D6's path and headed or headless.
2. The tool, fixtures and seed land; the implementation pull request carries the first generated
   set (site images, share images, videos) so the site builds.
3. The site work lands in the same pull request; `site-check` runs the budget.
4. The owner reviews the copy, then deploys (task 7.1) and uploads the store captures by hand.

Rollback: revert the pull request; the shipped packages are untouched, and the site redeploys from
the previous ref.

## Open Questions

1. **Which capture is the hero?** Recommendation: `card` — the reader's first studied language,
   highlighted, the pill and an open word card in one picture, which is the product in one look;
   with the close-up crop on phones. Alternative: the bare highlighted page (calmer, says less).
2. **Does the video live on the site?** Recommendation: yes, self-hosted, one per interface
   language, under 4 MB each, in the « En images » section rather than the hero, muted and started
   only when in view and when motion is welcome (D9). Not on YouTube, because the site promises no
   trackers; the Chrome Web Store's video field stays empty for now.
3. **E-ink look captures?** Recommendation: one — `reader-eink`, the EPUB reader in « E-ink
   contrasté » — on the site only, beside the reader figure; e-ink readers are an audience the
   listing already names (« une tablette à encre électronique comprise ») and few competitors show.
   Not in the store sets, which are full.
4. **The fifth store slot: the EPUB reader, or the second studied language?** Recommendation: the
   reader (D5); the second language already has `second`. This replaces `STORE-LISTING.md`'s plan
   of a fifth "French" capture.
5. **Commit the store PNGs, as Music does?** Recommendation: no — they are made from the committed
   site sources' run in a minute and uploaded once; committing ≈ 7 MB per refresh buys little.
   Committed, they would sit under `apps/lingua-extension/store/` like Music's.
