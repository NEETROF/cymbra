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
background-owned state store, with two object stores: `books` (the hash as key; title,
authors, language, cover, size, added-at, the last location and its updated-at) and `files`
(the hash as key; the `Blob`). The reader page opens it directly: a 40 MB file must not cross a
message, and the single-owner rule exists to keep surfaces agreeing on *state* — the library
has one writer, the reader page, and its only concurrent writes are positions, resolved by
their timestamp.

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

- Whether Chromium's default quota for the extension origin holds a realistic library without
  `unlimitedStorage` (spike, D5).
- Whether a section painted whole stays fluid on an iPhone, and what the largest EPUB the
  Safari extension process can open is (spike, D4).
- Whether the reveal cap should be one value or scale with the section's length; the spike's
  numbers on the tablet and the laptop decide.
