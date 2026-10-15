# Design — add-site-lingua-showcase

## Context

See proposal.md (Why). What exists, read on `main` at e07d23bd3 and re-read on review at fb9c607c2
(nothing under `apps/`, `.github/` or `scripts/` moved between the two):

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
| `apps/lingua-extension/build.mjs`, `tool/manifests.mjs` | Chromium: `<all_urls>` is an **optional** host permission; `syncReaderRegistration` registers the reader on every page once it is granted (`src/background.ts`), on install too; `LINGUA_ALL_URLS=1` (`buildManifest`'s `allUrls`) adds the static content script on every page to a Chromium build — a development switch, documented in `README.md` ("Chromium-only dev escape hatch") and held by `test/manifests.spec.ts` ("forces the static reader on Chromium only when asked"); the build writes `dist-<target>/` only, no output-folder option; the dev key pins the id `figfjglfdiffocldficbimecjnhnkhkh`; Firefox: `gecko_android` declared (#599), and `side_panel` deleted with no `sidebar_action` declared |
| `apps/lingua-extension/src/reading/drawer.ts`, `src/reading/session.ts` | the in-page panel (Révision / Statistiques / Réglages), a closed-shadow overlay mounting the same views as Chrome's side panel; the only panel on Firefox and Safari, the Alt+Shift+D review on Chrome; a reading session opens it on `{ type: "openDrawer", view }`, the message the Firefox background sends |
| `apps/lingua-extension/src/content.ts` | guards against running twice (`__cymbraLinguaReading`): a registered script and a static one never start two sessions |
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
- Playwright's default headless binary (`chrome-headless-shell`) does not load extensions; the
  full Chromium build does, headless too, when launched with `channel: 'chromium'` (Playwright's
  Chrome-extensions guide).
- Chrome's side panel is browser UI, not a page: a page screenshot does not include it. Its
  document is a DevTools target of its own (reached that way on 2026-10-06 to debug #717). The
  in-page panel (`drawer.ts`) is part of the page, and a page screenshot includes it.
- The main spec's *Firefox variant* requirement still says the panel is a `sidebar_action`; the
  code removed it (`firefoxManifest`'s comment: "there is no side_panel AND no sidebar_action"). This change does
  not touch that requirement; D6 follows the code.
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

### D1 — The capture build: the existing "reader on every page" switch, in a folder of its own

On Chromium the reader starts on a page after the toolbar button (`activeTab`) or once the reader
has granted `<all_urls>` (« Toujours surligner »), and both pass through browser UI a test driver
cannot reach. The build already has a way round it: `LINGUA_ALL_URLS=1` makes `buildManifest`
(`tool/manifests.mjs`, `allUrls`) add the static content script on every page that the Firefox and
Safari variants always carry. It is documented in `README.md` as a Chromium-only development
switch, `test/manifests.spec.ts` holds it, and `build.mjs` hands it to `buildManifest` alone, so
the bundles do not move. If the background also registers its dynamic copy, `content.ts`'s guard
keeps it to one reading session. The reader the captures show is the same `content.js`, at
`document_idle`, that a Chrome reader who said yes to « Toujours surligner » gets; only the way it
is attached differs.

`yarn capture` builds the Chromium variant with that switch into `dist-capture/`. `build.mjs` gains
one thing, an output-folder option (`LINGUA_DIST_DIR`, `dist-<target>` by default), so that a
capture never rewrites the `dist-chromium/` a dogfood browser has loaded — often a build made with
the production settings. Before loading it, the tool builds a plain Chromium variant with the same
settings into a temporary folder and refuses to go on unless every file but `manifest.json` is
identical and the manifests differ by that content script alone; then it deletes the temporary
folder. `dist-chromium/`, `dist-firefox/`, `dist-safari/`, the release workflow and
`check:variants` never read `dist-capture/`.

Rejected: a new manifest shape that makes `<all_urls>` a required host permission (this proposal's
first draft: a second capture-only variant of the manifest to write and test, when the repository
already has and tests one); clicking « Analyser cette page » (the popup is browser UI; opened as a
tab, `popup.html` is its own active tab); accepting the permission prompt (browser UI); the
extension's keyboard commands (keys sent through DevTools reach the page, not Chrome's shortcut
handler).

### D2 — Playwright's Chromium, with Chrome's field trials on

`apps/lingua-extension` gains `playwright-core` as a dev dependency, pinned to the back office's
Playwright version so both use one cached browser. The tool calls
`chromium.launchPersistentContext` (extensions need one) with `--disable-extensions-except` and
`--load-extension` on `dist-capture/`, and `ignoreDefaultArgs` removing
`--disable-field-trial-config` (and `--disable-extensions` if the pinned version passes it). It
never asks for `channel: 'chrome'` (branded Chrome). Each run starts on a fresh temporary profile,
deleted afterwards, so no run inherits another's state; the onboarding tab the install opens is
closed. Headless is the default once task 1.1 shows the captures match a headed run — launched
with `channel: 'chromium'`, since the default headless binary loads no extension — and `--headed`
opens the window to watch it. Nothing the shots need is browser UI (D6), so headless is expected
to work. The tool prints `chrome://version`'s command line in its log, so the owner can see which
flags the captures ran with.

Driving the product: the word card (`wordpopup.ts`), the pill (`hud.ts`) and the in-page panel
(`drawer.ts`) each live in a **closed** shadow root, which Playwright's locators do not enter, and
the highlights are painted with `CSS.highlights`, so a highlighted word is not an element. The
tool therefore finds a word from the page's own text (a `Range` and its `getClientRects()`), finds
a control of those surfaces through the DevTools protocol (`DOM.getDocument` with `pierce: true`,
which reaches closed roots as DevTools' own Elements panel does, then `DOM.getBoxModel`), and
clicks at that point with `page.mouse`, as a reader's pointer would; once a surface has the
focus, its own keys work too (the review reveals on Space and grades on ← and →,
`src/review/view.ts`). Nothing is clicked by a fixed coordinate, so a layout change fails loudly
on the missing control rather than clicking the wrong one. Task 1.1 checks it.

Rejected: Puppeteer (a second browser toolchain in the repo); Firefox through WebDriver BiDi for
the AMO set (the page, the word card and the in-page review panel are the same code on both, D6;
only the browser's own frame differs, and no shot shows it).

### D3 — Fixtures: our own articles and public-domain books, served locally

`tool/capture/fixtures/pages/{en,es,fr}.html`: one article per studied language, about 450 words,
written for the purpose on one topic (the same subject in the three languages, so the captures
read as one story), at a B1–B2 vocabulary so a B1 reader sees a sensible share highlighted. Plain
neutral typography, no publication name, logo or brand of anyone else. Licensed as the
repository (Apache-2.0), which `tool/capture/fixtures/README.md` states.

`tool/capture/fixtures/books/`: one chapter per studied language from a public-domain work —
English: Jane Austen, *Pride and Prejudice* (1813), chapter 1; Spanish: Benito Pérez Galdós,
*Marianela* (1878), chapter 1; French: Jules Verne, *Le Tour du monde en quatre-vingts jours*
(1872), chapter 1. Each is the original-language text, not a translation (a translation has its
translator's own copyright); the authors died in 1817, 1920 and 1905, so the works are in the
public domain in France, Spain, the European Union and the United States. The text is taken from a
transcription of a nineteenth-century printing — Project Gutenberg's, for instance, whose licence
lets its header, licence and name be removed — never from a modern annotated or critical edition,
whose apparatus, notes or modernised text may carry rights of their own. It is committed without
any distributor's header or licence, the source printing named in the README. The tool assembles each into an EPUB 3
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
| `review` | The article with the in-page review panel open, a card revealed with its sentence and the two answers (D6) | English | Spanish | English |
| `stats` | The statistics page, the first studied language's ladder and the fortnight | English | Spanish | English |
| `reader` | The EPUB reader on the first studied language's book, highlighted | Austen | Galdós | Austen |
| `reader-eink` | The same page in the « E-ink contrasté » preset (open question 3) | idem | idem | idem |

The Chrome Web Store and AMO set is `card`, `second`, `review`, `stats`, `reader`, in that order —
five, the Chrome Web Store's cap. The reader's two languages are shown by `card` and `second`, so
no slot is spent on "the French capture" alone, which `STORE-LISTING.md` planned as the fifth
before the reader existed (open question 4).

### D6 — The review shot: the in-page review panel, opened by the product's own message

The review is shown in the panel every variant draws inside the page: the drawer
(`src/reading/drawer.ts`), a closed-shadow overlay that mounts the same review, statistics and
settings views as Chrome's side panel. On Firefox and Safari it is the only panel —
`firefoxManifest` deletes `side_panel` and declares no `sidebar_action` — so it is what an
addons.mozilla.org visitor will get; on Chrome it is the review that Alt+Shift+D opens. The tool
opens it the way the Firefox background does: `chrome.tabs.sendMessage(tabId, { type:
"openDrawer", view: "review" })` from an extension page, which the page's reading session answers
(`src/reading/session.ts`). The page and the panel are then one page screenshot at 1280×800: no
DevTools target, no composition, nothing drawn that is not the product's.

Rejected: Chrome's side panel, opened with a user gesture through DevTools, captured through its
own DevTools target and composed beside the page (this proposal's first draft). It assumed that
Firefox shows the same page in a `sidebar_action`, which the code removed, so the AMO set would
have shown a panel no Firefox reader sees; and the gesture, the second target's screenshot and the
composition were the riskiest part of the spike. A Chrome-only review shot in the side panel can
follow later if the owner wants Chrome's set to show it.

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
`og:image` (a 1200×630 JPEG made by `getImage`, named by its absolute address — `https://cymbra.app`
+ the hashed path `getImage` returns — since several link previewers ignore a relative one),
`og:image:width`, `og:image:height`, `og:image:alt`, `og:site_name` « Cymbra », `twitter:card`
`summary_large_image`; and for every page
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
slots, macOS's 1280×800 included, are not filled from the Chromium captures — they are Chromium's
rendering of the extension, not the Safari app's, and App Review asks for the app in use
(Guideline 2.3.3).

No `_locales` summary changes, so the extension's version check is untouched.

## Risks / Trade-offs

- **Repository weight.** Three videos (≈ 12 MB) and ≈ 25 site sources (≈ 6 MB) per refresh, kept
  in history. Mitigated by refreshing only when the interface visibly changes and by D14's caps;
  store PNGs are not committed (open question 5).
- **The review panel shown.** Chrome's set shows the in-page panel (D6), which Chrome readers
  reach with Alt+Shift+D while the page's own buttons open the side panel there; both hold the same review view,
  the same card and the same two answers. Accepted for this change; a side-panel shot for Chrome
  alone can follow.
- **Headless.** If task 1.1 finds a headless run differs from a headed one, the tool stays headed
  (a window open on the Mac for a few minutes); nothing else changes.
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

1. Task 1.1's spike confirms D1, D2 and D6 on the pinned Playwright, decides headed or headless,
   and D8's recording path.
2. The tool, fixtures and seed land; the implementation pull request carries the first generated
   set (site images, share images, videos) so the site builds.
3. The site work lands in the same pull request; `site-check` runs the budget.
4. The owner reviews the copy, then deploys (task 7.1) and uploads the store captures by hand.

Rollback: revert the pull request; the shipped packages are untouched, and the site redeploys from
the previous ref.

## A leaner first version (an option for the owner)

The full scope above is 11.75–18.25 ideal days (tasks.md). Most of it is the capture tool, the
fixtures, five shots in three interface languages and the site's head — the core of the request,
which no cut can remove without losing it. What can wait without losing anything you asked for
except the video:

| Left for later | Saves | Later, as its own small change |
|---|---|---|
| The demo video (D8, D9, its markup, script, poster and budget lines) | 1.5–2.5 d | 1.5–2.5 d, reusing the tool, the fixtures and the seed |
| `--record-seed`: the three seed backups are made once by hand in the capture browser and saved through Réglages › Données › Sauvegarder (still the product's own format, D4) | 0.5–0.75 d | only if re-recording by hand becomes a chore |
| The e-ink figure (open question 3) | 0.25 d | 0.25 d |
| The Music pages' own share image: they take the brand image, so no crop of Music's capture and no new `site-check` filter | 0.25 d | 0.25 d |

The leaner first version is **9.25–14.5 ideal days** (1: 4–6.5, 2: 1.5–2.25, 3: 3–4.5, 4: 0.5–1,
5: 0.25): about a fifth less, **not half**. It is therefore not the recommendation (open question
6); it is the right choice only if having the pictures and the store screenshots live a few days
sooner matters more to you than launching the video with them. If chosen, the video sentences of
`site-lingua-page`'s first requirement and of the capture requirement move to the follow-up change
before implementation starts, and tasks 1.6 and the video parts of 3.1 and 3.4 go with them.

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
6. **Everything at once, or the leaner first version without the video?** Recommendation: everything
   at once (11.75–18.25 days). The leaner version (9.25–14.5 days, above) saves about a fifth,
   not half, and drops the one thing that shows Lingua moving, which you asked for. Choose the
   leaner one only if the pictures must be online sooner and the video can follow a few weeks
   later.

### The owner's answers (2026-10-14)

Given before implementation started (task 6.1):

1. **Hero**: `card` — the highlighted page with a word card open, and the close-up crop of the
   card on phones.
2. **Video**: on the site, self-hosted, one per interface language, below the hero (in « En
   images »), never starting on its own beyond D9's rule (muted, in view, motion welcome, data not
   saved).
3. **E-ink**: one capture, `reader-eink` — the EPUB reader in « E-ink contrasté » — on the site
   only.
4. **The fifth store slot**: the EPUB reader (`reader`).
5. **Store PNGs**: not committed — generated locally under `store-captures/` (git-ignored) and
   uploaded by the owner.
6. **Scope**: everything at once, the video included — not the leaner first version.

## Deviations in the implementation

What task 1.1's spike found, and where the implementation departs from D1–D16:

- **Spike (task 1.1).** Every part of D1, D2 and D6 held on Playwright 1.62.0 (Chrome for Testing
  151): the reader starts on a served page from the static content script; `openDrawer` sent from an
  extension page opens the in-page panel (the tab is found with `chrome.tabs.query({ active: true })`
  once the article tab is brought to the front: without the `tabs` permission an extension page
  sees no tab's address); a word found from the page's text and a control found through
  `DOM.getDocument` with `pierce: true` are clicked with the mouse. Headless (`channel: 'chromium'`)
  and headed runs show the same pages, pixel for pixel but for the window's scrollbar, which a headed
  window draws (the article shifts ~8 px) and headless hides: headless stays the default. Two
  headless runs on one day write byte-identical site images. `recordVideo`, re-encoded by ffmpeg
  (H.264, CRF 23), reads well at 1280×800: no screencast fallback.
- **A host name for the fixtures (D3).** The pages are served from `127.0.0.1`, under the reserved
  name `reading.example` (`--host-resolver-rules`): a card keeps the host it was met on as its source,
  which the review capture shows — « reading.example » rather than « 127.0.0.1 ».
- **The seed (D4).** `--record-seed` chooses the second studied language and level B1 on the install's
  welcome page (`onboarding.html`), then marks known and adds to the deck the words `plan.mjs` lists,
  as many as the page highlights at B1 (about 30 known, 12 cards in all), the deck's one second
  apart (a card's capture time is kept to the second, and the review takes new cards in that order);
  it answers the deck's first four in a short review, and marks « Ignorer » the book's names the EPUB
  reader highlights (Bennet, Bingley, Morris, « Oh »), as a reader does. A word goes to the deck only
  when its card has a translation. The tool pretty-prints the backup the product saves; Prettier
  ignores those files. Before restoring a seed, the tool moves its dates — levels declared, words
  read, cards met, answered and due — by the whole days between its recording and the run
  (`tool/capture/seed.mjs`), so that every run, whatever its day, finds the same cards due; the tool
  and the browser read their clock in Europe/Paris.
- **The statistics (D4).** Thirty days ending on the run's, not a fortnight: the statistics page opens
  on its 30-day range, which a fortnight left half empty (`tool/capture/days.mjs`; the spec's scenario
  says thirty).
- **Captures that would mislead are refused (D5).** The tool fails, naming the shot and the word, on
  a hero or second card without a translation, a second card or a revealed review card whose gloss
  reads as a dictionary's list (`glossProblems`: over 60 characters, a parenthesis, a first name's
  sense — « Aube, aurore, … (sunrise); … Prénom féminin » was the French set's review card), a book
  name still highlighted, a statistics page without its language picker, a pill that does not rise
  in the video, or a video whose last frame shows the note on agent sessions. The words were chosen so
  that each set passes: the review reveals « Émission » (fr), "airline" (en), « Emisión » (es); the
  Spanish set's hero is « vanished » (« abandoned » opened on a noun, « Despreocupación »), its second
  card « empreinte », the English set's « couchette » ("sleeping berth": « empreinte »'s English
  gloss is a parenthesis).
- **The English set's reader at C1 (D5).** Galdós's first chapter is 82 % known at B1 — a page of
  highlights. The tool declares C1 in Réglages before the reader's two captures (the last ones):
  92 %, as the other sets' Austen at B1. The other English-set captures stay at B1.
- **The close-up and the crops (D7, D10).** The captures are taken at 3× so that the phone's close-up
  holds 1260 px for a 420-px phone at 3×; the whole-window sources are written at 2× (2560 px). The
  hero is cut to 16:9 from the window's foot (the card and the pill kept, the article's top lines
  gone), and the share image starts at the window's top when the page's title sits just above the
  card. The site's `second` and `review` are portrait crops (600 and 640 CSS px wide) — the card with the start
  of its lines, the review panel with the page beside it — readable in a 430-px column, and each
  figure links to its 1280-px image; `stats` is cut below its ladder, before « Renforcer un niveau »
  (and the line it repeats, a product matter left to its own change). The store PNGs stay whole
  windows.
- **The video (D8).** « + Deck » is pressed on the verb whose card opened (4–15 s), then « Je connais »
  on words of the first screen until the pill's share rises (one word in each set: 93 → 94 %, 92 →
  93 %); the statistics then scroll from the ladder to the thirty days' charts and stop above the
  page's closing note on agent sessions. The videos last about 28 s and weigh about 1.2 MB each.
- **The site's variants (D7).** WebP alone, at 640, 960 and 1280 px for the captures shown in a column
  or centred, 1920 too for the hero, shown wide, and up to 1260 px for the phone's close-up: every
  variant stays within D14's budget. AVIF was dropped: built from these captures (text on a flat
  page) at the same quality, it was heavier than WebP for most variants (5.0 MB against 4.8 MB over
  the three pages, 58 variants heavier), so it cost bytes without paying. `Shot.astro` reads no
  property of an imported image (Astro would then ship the source beside its variants).
- **The layout (D10).** The hero's note follows its capture, and the hero has less air above and below,
  so that a 1280×800 window shows most of the capture (it starts at 408–471 px, 55–66 % of it above
  the fold, where it started at 748 px). After the video, five captures in two columns:
  `second` and `review`, then `stats` alone on its row, centred, then `reader` with `reader-eink` beside
  it (open question 3).
- **The brand image (D12).** Astro rasterises an SVG only with `image.dangerouslyProcessSVG`, set in
  `astro.config.mjs` for that one file of ours.
- **The byte pin (task 3.5).** The Lingua page's fixtures leave aside its image addresses and script
  bodies (`test/support/media.mjs`): the build hashes them and the Container API names them by their
  path on the machine. `test/support/refresh-lingua-fixtures.mjs` replaces the README's one-liner;
  `taken-with.json` takes the figures `lingua-pack-update` refreshed since (fr-en, fr-es).
- **CI (task 2.2).** `site-check`'s filter also gains `apps/lingua-extension/tool/capture/plan.mjs`
  and `apps/lingua-extension/src/i18n/**`: the site's tests read the capture plan and the labels the
  alt texts quote.
- **Two pull requests.** release-please assigns a squash commit to components by path, so a
  `feat(site)` squash touching `apps/lingua-extension` and `apps/lingua-apple` would release both for
  byte-identical packages. The extension side (the capture tool, its fixtures and seeds, `build.mjs`'s
  output folder, `playwright-core`, the READMEs and the listings) lands first as a `chore(lingua)`
  pull request; the site's side, the committed captures and this change's tasks follow in a
  `feat(site)` one.
- **The listings' counts (D16).** Re-measured as pasted (paragraphs a blank line apart, lines joined,
  bold dropped); the previous counts predated the « Traduction en ligne » paragraph.
