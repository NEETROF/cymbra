## Why

Lingua reads the page a reader is already on, and a reader who wants to read a **book** with it
has nowhere to put one. The web readers that hold books are either online-only (Kavita, calibre's
content server) or render each chapter in iframes the content script does not see (Readium,
epub.js, foliate-js, Bibi) — and none of them works in a train. The readers who asked are reading
novels and technical books as DRM-free EPUB, on a laptop today and on an Android e-ink device
next, mostly away from a network.

The cheapest place for a book to be readable offline, with Lingua's highlighting, on every
browser Lingua ships on, is **a page of the extension itself**: an extension page loads from the
installed bundle and never from the network, the analysis engine and the pack are already
there, and the reading module that highlights, glosses and captures is already written — it only
needs a document to work on. The book rendering is not written either: foliate-js, the MIT
engine behind Readest, already does pagination, fonts, images, tables of contents and positions.

## What Changes

- **A reader page inside the extension** (`reader.html`, every variant): a library of the
  reader's own EPUB files, imported through the browser's file picker, kept in an
  extension-owned IndexedDB database, opened offline. Rendering by a vendored, commit-pinned
  copy of foliate-js.
- **The reading module gains a document parameter.** Today it binds to the global `document`
  of the page it is injected into; the reader mounts it on the document of each book section
  foliate-js renders. Same blocks, same engine port, same highlights, same word popup, same
  selection card, same drawer, same statistics — nothing duplicated. The content script keeps
  passing the global document and behaves exactly as before.
- **Reading that paints once.** A section is analysed and highlighted before it is revealed, and
  a section is painted whole rather than by viewport window, so a page turn on an e-ink screen
  costs one refresh, not two. Paginated flow by default, tap zones to turn, no animation.
- **Highlights told apart without colour.** The two statuses currently differ by tint only; on a
  monochrome screen they read the same. The two underline styles become distinct.
- **Cards from a book carry the book as their source** — title and chapter — in the same
  local-only field that holds a page address today. It is never sent, as the address is not.
- **A protected book is refused with a plain sentence**, never opened half-broken.

**Deliberately out of scope**, each a follow-up change: synchronising the reading position
between devices (touches the backend and the privacy annex), fetching books from an OPDS
catalogue (Kavita, Standard Ebooks, Gutenberg), the whole-book coverage score, the preparation
deck, formats other than EPUB.

## Capabilities

### New Capabilities

- `lingua-reader`: the extension's own book reader — the library, import and deletion, offline
  opening, paginated rendering suited to e-ink, the reading module mounted on the book, the
  local reading position, the refusal of protected books, where the reader opens from.

### Modified Capabilities

- `lingua-browser-extension`: **Cymbra visual identity** — highlight tints must stay
  distinguishable without colour (two underline styles), not only by tint.
- `lingua-browser-extension`: **The reader never fights the platform's text selection** — on
  Safari, a finished phrase selection is removed once the finger lifts, so the platform's
  callout stops covering the expression card (found reading a book on an iPad).
- `lingua-decks-review`: **Card schema with provenance** — the source of an encounter may be a
  book (title and chapter), next to a page address or an agent session.
- `lingua-privacy`: **A card's page address stays on the device** — the same guarantee is
  stated for a book source: pushed empty, kept locally.

## Impact

**Products.**

- **Cymbra Lingua, the extension (`apps/lingua-extension`)** — everything new lives here: the
  page, the library store, the vendored renderer, the document parameter of the reading
  module, the popup and settings entry points, the store-listing sentence that describes the
  reader. No new permission: the file picker and IndexedDB need none, and an extension page is
  not injected into.
- **Cymbra Lingua, the Apple host app (`apps/lingua-apple`)** — consumes the rebuilt
  `dist-safari` bundle; nothing of its own changes. As for every extension release, the
  Apple release must be dispatched with `deliver` afterwards or Safari keeps the old bundle.
- **Backend, site, Music, Live, back office, Cymbra ID** — untouched. The reader syncs
  nothing new: word statuses and cards flow as they do today, the book file and the reading
  position stay on the device.

**Promises.** Page text still never leaves the device, and the book's does not either. The
disclosures — the privacy annex, the three store listings, the App Store answers — need no
new line; the Chrome Web Store single-purpose statement gains a clause saying the reader reads
books the reader imports, on the same purpose.

**Dependencies.** foliate-js (MIT), vendored at a pinned commit because it has no release and
declares its API unstable; ignored by lint, format and coverage; carried in the AMO source
archive like every other source.

**Cost.** Book files occupy the browser's storage for the extension: tens of megabytes per
book, on the reader's device only. Nothing runs on Cymbra's hardware.
