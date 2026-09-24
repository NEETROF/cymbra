## Context

Lingua's reading module (`src/reading/` and the `ReadingSession` in `src/content.ts`) turns a
DOM into analysis blocks, sends them through the `LinguaPort` seam, paints the answer with the
CSS Custom Highlight API and hangs the word popup, the selection card, the HUD and the drawer
off the page. It is one implementation with two hosts already — the injected content script,
and the review views rendered into the side panel and the drawer — and it binds to the global
`document` of whatever page it runs in.

A book is not a page. The web readers that hold books either need a network to show a chapter
(Kavita, calibre's content server) or render each chapter in an iframe (Readium, epub.js,
foliate-js, Bibi), where the top-frame content script sees nothing. The readers asking for
this read DRM-free EPUB — novels and technical books — on a laptop now and on an Android
e-ink device next, and they read where there is no network.

Three platform facts shape the answer (see the `browser-extension-architecture` skill):

- An **extension page** loads from the installed bundle, never from the network; its CSP
  already allows WASM (`manifest.json`, `extension_pages`); on Chromium the engine runs in the
  page, on Firefox and Safari it is reached in the event page over the same port
  (`createLinguaPort`). No content script has to be injected into it: `<all_urls>` does not
  match `moz-extension://` or `chrome-extension://`, so the static Firefox script and the
  activeTab path both leave it alone.
- The **reader's state** (statuses, cards, stats, cursors) lives in one IndexedDB database
  owned by the background; surfaces reach it through `messagedArea`. That rule is about
  state, not about blobs: a book file is not state, and it must not travel through messages.
- **Safari** suspends its background page whenever idle and bounds the extension process's
  memory; **Firefox for Android** shares the Firefox build; **e-ink** pays a visible refresh
  for every paint.

## Goals / Non-Goals

**Goals:**

- Read an imported EPUB, offline, with Lingua's highlighting, popup, selection card, drawer
  and statistics, on every variant the extension ships (Chromium, Firefox desktop and Android,
  Safari macOS and iOS) from one page.
- Reuse the reading module as it is: one implementation, one more host. The only change to it
  is *which document* it works on.
- Paint once per page turn, so an e-ink screen refreshes once.
- Keep every promise as written: no page or book text leaves the device, no new permission, no
  new disclosure, nothing new synchronised.
- Write no rendering code: foliate-js does the book.

**Non-Goals:**

- Synchronising the reading position between devices. It is reading history, it touches the
  backend and the privacy annex, and it deserves its own change with its own disclosure.
- Fetching books from a catalogue (OPDS: Kavita, Standard Ebooks, Gutenberg). A follow-up; the
  library's import seam is written so a fetched file enters the same way as a picked one.
- The whole-book coverage score and the preparation deck. Follow-ups; the library keeps enough
  (the file, its hash) for them to be computed later without re-importing.
- Formats other than EPUB, protected books, a bookshelf shared between readers, a reader on
  the site for readers without the extension.
- Matching a native e-ink reader's page-turn latency. A browser paints; the goal is one paint.

## Decisions

### D1. The reader is a page of the extension, not a site, a PWA or a third-party reader

An extension page is the only host that is offline by construction on every variant, that
hosts the reading module natively, and that needs no manifest change. The alternatives each
fail one of those:

- **A third-party local-first web reader (Readest web)** keeps books in the browser and syncs
  progress, but renders in iframes (`all_frames` for every page of every site on Firefox's
  static injection), its offline behaviour in Firefox for Android is unverified, and it ties
  the reader to a third-party account or WebDAV.
- **A PWA on cymbra.app** would reach readers without the extension, but its offline copy is
  a service worker the browser may evict, injection into a PWA window on Firefox for Android
  is unverified, and it needs the site, Cloudflare Pages and web-auth for a feature whose
  whole value is the extension's engine. If a site reader is ever wanted, the reader module
  can move to a shared package (`packages/`, the `web-auth` pattern) and be rendered there too.
- **A Cymbra reader as its own product** has no advantage over Readest, Moon+ or the Boox's
  native reader without Lingua's layer. The layer is the product; the reader lives with it, as
  a `lingua-*` capability.

### D2. foliate-js, vendored at a pinned commit, behind a renderer seam

foliate-js (MIT) renders EPUB 2 and 3, paginates, handles fonts, images, the table of contents
and positions, and is used in production by Readest. It has no release and says its API may
change at any time, so it is **vendored** (`apps/lingua-extension/vendor/foliate-js/`, the
commit recorded next to it, refreshed by a script) rather than taken as a git submodule (CI
checkouts would need `submodules: true` in every workflow) or a git dependency (an unbuilt
package Yarn would fetch on every install). The vendored tree is excluded from lint, format
and coverage, and carried in the AMO source archive as source.

The reader talks to it through a small `BookRenderer` seam — open a file, go to a location,
next and previous, the current location, an event when a section document is ready, an event
when the location changes — so tests use a fake renderer under jsdom (which cannot host
foliate-js's iframes) and a later renderer change stays local.

Alternatives: epub.js (older, larger, iframe-based too, less maintained), Bibi (dormant since
2021, no CFI positions), a custom inline renderer (writing a reader; no).

### D3. The reading module takes a document; the content script passes the global one

foliate-js renders each section into an iframe from a `blob:` URL of the extension's origin,
so the reader page can reach each section's document. The reading module is mounted **on that
document**: `collectBlocks`, `scan`, `rangeForToken`, the highlight registry
(`CSS.highlights` is per document — the section window's registry, not the page's), the
observers, the exposure tracker. The word popup, the selection card, the HUD and the drawer
stay in the reader page's own document, as they are the page's chrome, and receive the
section's events. The content script passes `document` and `window` and changes nothing else.

This is the one real refactor of the change, and it is bounded: every `document.` and
`window.` in `src/reading/` and `ReadingSession` becomes a parameter or a field. It also
unblocks, later and cheaply, highlighting third-party readers that render in iframes
(`all_frames` would mount the same session on each frame's document). It is the first thing
the spike proves.

Alternative: render sections inline into the page's own DOM instead of iframes. It would spare
the parameter, but foliate-js does not offer it and writing that renderer is the thing this
change refuses to write.

**Amended during implementation — Chromium serves sections from its service worker.** A
`blob:` document is *not* always reachable from the extension page that made it. Chrome's
migration to "block the V8 optimizer on unfamiliar sites"
(`MigrateToBlockV8OptimizerOnUnfamiliarSites`, a field trial) places such a document in a
process of its own: its origin is still the extension's, yet the page gets a `SecurityError`
and foliate-js stops before painting (`contentDocument` is null) — a blank book. It is on in
Chrome for Testing's default configuration, and can reach any Chrome in the trial's arm.
Playwright launches Chrome with `--disable-field-trial-config`, which is why the automated
passes never saw it. The Chromium variant therefore routes every section through the
background service worker (`src/reader/section-server.ts`): what foliate-js produced for the
section, resources already rewritten to `blob:` URLs, is put in Cache Storage and the frame is
pointed at `chrome-extension://<id>/reader-section/…`, which the worker answers — a plain
extension URL, kept in the page's process. The section is served with `script-src 'none'`.
foliate-js is not modified: the adapter wraps each section's `load`/`unload`. Firefox keeps
the `blob:` path (its event page is no service worker, and the reader works there); Safari's
event page is none either — if Safari splits `blob:` documents too, it needs another answer.

### D4. One paint per page turn

Two things paint a highlighted page twice today, and an e-ink screen shows both:

- the section appears, then the highlights arrive a few hundred milliseconds later;
- within a section, highlights are painted per viewport window (the Safari fix of
  `add-lingua-apple`: ±1 viewport, `IntersectionObserver`), so a page turn inside a section
  paints the next page's highlights after the turn.

The reader page therefore **reveals a section only once it is analysed and painted**, bounded
by a cap after which it is revealed anyway so a slow engine never blanks the book, and **paints
a whole section at once**, without the viewport window. A section is a chapter — thousands of
words — not a fifteen-thousand-word encyclopedia page; painting it whole is within what Safari
handled in the spike, and it is measured on an iPhone as part of this change. Page turns
inside a section then paint nothing new: the ranges are already there. Between sections, the
next one is not pre-rendered (foliate-js loads on demand); the reveal rule covers it.

The flow is paginated by default with tap zones to turn and no transition; scrolled flow stays
available as a setting for the laptop.

### D5. The library is its own database, opened by the reader page

Book files go in a second IndexedDB database (`cymbra-lingua-library`), distinct from the
background-owned state store, with three object stores, each keyed by the hash: `books` (title,
authors, language, cover, size, added-at), `files` (the `Blob`) and `positions` (the last
location and its updated-at). The reader page opens it directly: a 40 MB file must not cross a
message, and the single-owner rule exists to keep surfaces agreeing on *state* — the library
has one writer, the reader page, and its only concurrent writes are positions, resolved by
their timestamp.

A book's record is written once, at import; a position, at every page turn. They were one
record at first, and WebKit loses a `Blob` read back from IndexedDB when its record is written
again with it: on iOS and iPadOS the cover of the book just read came back unreadable
(`NotFoundError`). The position has its own store since (database version 2, whose upgrade
copies the positions the first dogfood builds kept in `books`).

The key is the SHA-256 of the file: importing the same file twice yields one book, and the hash
is what a later change would synchronise a position against. The OPF's `dc:identifier` is
kept as metadata, not as the key: publishers reuse it across editions.

The browser may evict site data under storage pressure; a library that vanishes on a train is
the failure this change exists to prevent. The reader page requests persistent storage
(`navigator.storage.persist()`) where the browser offers it, and tells the reader when it was
refused. Whether Chromium needs the `unlimitedStorage` permission for a library of several
books is measured in the spike, and asked for only if it does — it would be a new permission
to justify.

### D6. What a card from a book remembers

A card captured in the reader carries, in the same local-only `source` field a page address
uses, the book's title and the section's title (chapter), so that review can show where the
word was met. The field is pushed empty and stays on the device, exactly as the address does
(`lingua-privacy`); nothing new reaches the server. Deleting the book leaves the cards and
their source text intact: the source is a label, not a reference.

### D7. Protected books are refused before anything is rendered

An EPUB with `META-INF/encryption.xml` naming a DRM scheme (Adobe ADEPT, Readium LCP, Apple
FairPlay) is refused at import with one sentence in the reader's language — the extension
never shows a raw error — and nothing is stored. Font obfuscation (IDPF or Adobe algorithms,
also declared in that file) is not DRM and is left to foliate-js, which handles it.

### D8. Where the reader opens from, and how it fits the popup

The library opens from the popup ("Bibliothèque") and from the settings view rendered in the
drawer and the side panel — one action, one destination. The background opens or **focuses**
the existing reader tab rather than opening a second one, through the same `openPage`
message the drawer already uses. The reader page pushes the section's percentage to the badge
with the `stats` message the content script uses; the popup recognises a reader tab by its
URL and shows the section's figures instead of "Analyser cette page", which cannot inject into
an extension page anyway.

### D9. Two underline styles

Both statuses paint `underline dotted` today, told apart by amber versus coral. On a
monochrome screen they are the same grey dots. "Learning" keeps the dotted underline;
"unknown" takes a solid one. The tints stay, so a colour screen loses nothing, and every page
the content script highlights benefits — the change is in the token sheet, not in the reader.

## Measurements (spike, §1)

Taken on the implementation itself rather than a throwaway branch — the refactor of D3 was
bounded as hoped — with the real en-fr pack (40 704 lemmas), no level declared (every word
unknown: the worst case for painting), in Playwright's Chromium on a MacBook (Apple silicon),
extension loaded unpacked. The e-ink tablet, Firefox and the iPhone are still to measure.

**1.2 — the document parameter.** The sites touched stayed inside `src/reading/` and the
session: `blocks.ts` (derives its document from the root), `highlight.ts` (one painter per
document, on that window's `CSS.highlights`), `observer.ts` and `exposure-tracker.ts` (the
root's window's observers), `selection.ts` (the window's selection; `instanceof Element`
replaced by a node-type test — a section's nodes are of another realm), and `caretAt`, now in
`session.ts`. One more cross-realm detail surfaced in the tests: a listener's `AbortSignal`
must come from the section's own window. Highlights, the word popup (anchored through the
frame's offset) and the selection card work inside a section.

**1.3 — paint times (laptop, Chromium, engine in the page).**

| Section | Words | Ranges | Hidden (load → painted) |
|---|---|---|---|
| Chapter (Hound of the Baskervilles, SE) | ~3–4 000 | 2 208–4 028 | 26–34 ms |
| Front matter (Pro Git) | a few hundred | 35–597 | 1–12 ms |
| Whole novel in one section (Pride and Prejudice, Gutenberg) | 20 484 | 20 149 | 185 ms |

A page turn inside a section re-registers nothing (the highlight objects are the same before
and after) and never hides the book; it reaches the second frame in 6–35 ms. The cap is set
at **1 500 ms** (`REVEAL_CAP_MS`) until the tablet's numbers: the laptop is an order of
magnitude under it even for the largest section, and Firefox adds a message round trip to the
event page.

**Finding: a section is not always a chapter.** Gutenberg's edition of *Pride and Prejudice*
holds the whole novel in one XHTML file: 20 149 ranges painted at once, more than the ~15 000
that stalled Safari in the Apple spike. Chromium takes it in stride; it is what 1.5 must
measure on the iPhone. If Safari stalls, the viewport window comes back for the reader on
Safari only (`paintWhole` is a per-host flag), at the cost of a second paint there.

**1.4 — storage (Chromium).** Three books (0.5 MB novel, 13.3 MB Pro Git, 23.7 MB illustrated
novel) occupy 41.2 MB of a 10.8 GB quota: `unlimitedStorage` is not needed for room.
`navigator.storage.persist()` is **refused** for the extension's origin, so the library shows
its notice on Chromium. Asking for `unlimitedStorage` would exempt the library from eviction,
at the price of a new permission to justify — left as a decision (see Open Questions);
Firefox is still to measure.

**Offline.** With every request sent to a dead proxy, opening and reading the 23.7 MB book made
52 requests, all `chrome-extension:` or `blob:` — none left the extension; its 36 illustrations
loaded.

**What the automated passes could not see.** They ran Playwright's Chromium, which disables
Chrome's field trials. With them on — Chrome for Testing's defaults, and the first manual pass
on the laptop — a Packt EPUB opened blank; the cause and the service-worker route are in D3.
Re-run with the field trials on, the same four books open and paint (a chapter of 512 to 2 208
ranges, the illustrated Gutenberg edition with its images).

**Entry points.** `tabs.sendMessage` from the popup reaches the reader page in its tab on
Chromium (MDN documents the same on Firefox), so the popup's `getStats` works unchanged and the
page answers `surface: "book"`. A second "Bibliothèque" focuses the open reader tab.

**Book scripts.** Pro Git's chapters carry inline scripts; the sections inherit the extension
pages' CSP and every one of them is refused. That is the intended outcome — a book's script
would otherwise run with the extension's privileges — and `test/reader-csp.spec.ts` keeps the
policy from being loosened.

**What review shows.** Review showed no source at all before this change, page or book. The
engine's review card now carries the card's local source, and review shows it once the answer
is revealed — a page by its site, a book by its title and chapter — so a card from a deleted
book still says where it came from.

**Safari on iOS and iPadOS (simulator, iPad).** The first passes there opened some sections
blank — the page's paper, the chapter's title in the bar, no text — and half-shifted pages
after a few turns. The section was loaded, laid out and painted by the session (its frame in
place, its text dark on transparent, 203 ranges registered, nothing over it): WebKit simply
never drew the frame. The same foliate-js, bundled alone and served to the simulator's
Safari, drew it, with or without the hiding, the highlights and a dark embedding page. Giving
the renderer's element a compositing layer of its own (`transform: translateZ(0)`) makes the
frame draw at once, at every turn and every section change; it costs nothing on Chromium.
Two cosmetic defects came out of the same passes: the reader page is dark and the book light,
so each section's frame got an opaque white canvas over the paper (the book area now declares
itself light), and hiding the whole book area while a section paints showed the dark page for
an instant at every chapter (only the renderer is hidden now; the paper stays).

**Selecting on Safari (simulator and iPad).** The platform's callout covered the expression
card. It cannot be hidden while a selection exists, and removing the selection is the only way
to dismiss it: measured on the Galaxy Tab, Firefox for Android sends the page no touch event at
all during a handle drag (so the end of one is unknowable), while Safari sends `touchend` after
it — on the simulator and on an iPad. So on Safari only, and only while the reader is switched on and
has analysed the page (where its card takes the callout's place), a phrase's selection is
removed once the finger lifts from it, and a single word's is kept so its handles still extend it (the
`lingua-browser-extension` requirement is amended accordingly). In the reader two foliate-js
behaviours stood in the way, both on Safari: its finger pan follows every `touchmove` in a
section and cancels it, so a handle drag slid the page instead of growing the selection
(`touch-guard.ts` keeps the moves of a touch working the selection away from it); and it settles
the page after every lift, reporting a `relocate` even when nothing moved, which closed the card
that lift had just opened (the reader now dismisses only on a real move).

**Platform gaps closed along the way.** foliate-js uses `Object.groupBy`/`Map.groupBy`, missing
from Chrome 116 and from Safari before 17.4 (the Apple app targets iOS 17.2): the reader page
installs both where absent. Its zip reader is the npm package it builds from, at the version it
locks (2.8.22), taken at `lib/zip-core.js` — the package's `exports` would otherwise hand
esbuild its WebAssembly variant.

## Risks / Trade-offs

- **foliate-js changes its API** → pinned commit, a seam, and the vendored tree is refreshed
  deliberately, with the spike's measurements re-run.
- **The document parameter reaches further than `src/reading/`** (the selection module,
  `caretAt`, the exposure tracker's intersection observer) → the spike does the refactor first
  and stops the change if it is not bounded; the content script's behaviour is pinned by the
  existing tests, which keep passing unchanged.
- **Painting a whole section stalls Safari** (the 15 000-range stall of the Apple spike) → a
  section is far smaller than that page; measured on an iPhone before the change ships, and
  the window can be re-enabled for the reader alone if a section proves too large.
- **The extension process on iOS runs out of memory with a large, illustrated EPUB** →
  foliate-js loads one section at a time and reads the file lazily; measured on a device
  with the largest book at hand; if it fails there, the reader is kept out of the Safari
  variant by a build define (`check:variants` verifies the page is absent), not shipped broken.
- **Storage eviction takes the library** → persistent storage requested and its refusal
  shown; the reader can always re-import.
- **The popup's "Analyser cette page" on a reader tab** → recognised by URL; verified on all
  three variants in the device passes.
- **Two reader tabs edit the same position** → last write by timestamp wins; no lock, no
  corruption: a position is one small record.
- **Firefox's static content script is not injected into the page, but Chromium's activeTab
  could be asked to** → `scripting.executeScript` refuses extension pages; nothing to guard,
  verified in the passes.
- **An `.epub` picked on Android arrives with no extension or a generic MIME type** → the
  import sniffs the zip's `mimetype` entry, not the name.

## Migration Plan

Nothing migrates: the state store is untouched, the library is new and empty at first open, no
schema or protocol changes. Shipping is an ordinary extension release, followed by the Apple
release dispatch (`deliver`) so Safari receives the bundle. Rollback is the previous version:
a library left behind by a reverted build is harmless and reclaimed by a full reset.

## Open Questions

- ~~Whether Chromium's default quota holds a realistic library without `unlimitedStorage`~~ —
  it does (10.8 GB). Still open: whether to ask for `unlimitedStorage` anyway, because
  Chromium refuses `persist()` to the extension and the library is therefore evictable there
  (a new permission and a store-listing justification, against a notice every Chrome reader
  sees).
- Whether a section painted whole stays fluid on an iPhone — now with a known worst case, a
  20 000-word section — and what the largest EPUB the Safari extension process can open is
  (1.5).
- Whether the reveal cap should be one value or scale with the section's length: the laptop
  scales roughly linearly (30 ms per chapter, 185 ms for 20 000 words); the tablet decides.
