## 1. Spike — prove the two things the design rests on (one day, throwaway branch)

- [x] 1.1 Vendor foliate-js at a pinned commit under `apps/lingua-extension/vendor/foliate-js/` and open a real EPUB in a bare `reader.html` extension page on Chrome desktop, then on the Galaxy Tab S6 Lite under Firefox for Android (`yarn dogfood:firefox-android`, real pack via `gen:pack:real`, `--user 0` to see Firefox): the file picker returns the file, the book paginates, the tap zones turn pages
  - Chrome desktop: done on the implementation branch (Playwright's Chromium, unpacked build) — picker, pagination, tap zones. Pending: the Galaxy Tab S6 Lite under Firefox for Android.
  - Galaxy Tab S6 Lite, Firefox for Android (release, temporary add-on via web-ext, real pack, 2026-09-24): EPUBs picked from the Android picker, opened and read. Files pushed with `adb` show in the picker only once MediaStore has scanned them (`content call --method scan_volume --uri content://media --arg external_primary`). The tap zones turn pages on the tablet (2026-09-25).
- [x] 1.2 Do the document-parameter refactor of D3 on the spike branch, mount `ReadingSession` on the section document delivered by foliate-js's `load` event, and confirm highlights, the word popup and the selection card work inside the section; list every `document.`/`window.` site touched — if the list reaches beyond `src/reading/`, `content.ts` and the selection helpers, stop and revise the design
- [ ] 1.3 Measure on the tablet and the laptop: analysis time of a chapter-sized section, time from `load` to painted highlights, and whether a section painted whole (no viewport window) turns pages with one paint; record the numbers in design.md and pick the reveal cap from them
  - Laptop (Chromium): recorded in design.md, « Measurements » — cap set at 1 500 ms. Pending: the tablet, and Firefox (engine in the event page).
- [ ] 1.4 Measure storage: import three books of realistic size on Chrome and Firefox, check the quota reported by `navigator.storage.estimate()`, whether `navigator.storage.persist()` is granted, and whether Chromium needs `unlimitedStorage` — record the answer in design.md D5
  - Chrome: recorded in design.md — 3 books = 41 MB of a 10.8 GB quota, `persist()` refused, `unlimitedStorage` not needed for room (open question: for eviction). Pending: Firefox.
- [ ] 1.5 Build the safari variant of the spike, open the largest EPUB at hand in Safari on an iPhone, and note memory (`phys_footprint`) and whether a whole-section paint stays fluid; record it in design.md and decide whether the Safari variant ships the reader
  - iOS simulator and an iPad (Safari, 2026-09-24): the reader opens, reads and turns pages once two WebKit defects were fixed — a section frame never drawn without a compositing layer, and a cover lost when its record was written back (design.md, « Measurements » and D5). Pending: the iPhone, its memory on the largest book.

## 2. The reading module takes a document

- [x] 2.1 Give `collectBlocks`, `scan`, `rangeForToken`, the observers, `caretAt`, the selection module and the exposure tracker a `doc`/`win` parameter (or a `ReadingHost` holding both), defaulting to the globals so every existing call site compiles unchanged
- [x] 2.2 Make `highlight.ts` register and clear highlights on the *section window's* `CSS.highlights` and inject the token sheet into the section document; keep the viewport window as an option (`window: true`), on for the content script, off for the reader
- [x] 2.3 Let `ReadingSession` be constructed for a given document and a given surface host (where the popup, card, HUD and drawer mount), with the content script passing `document`/`window` for both; the existing content-script tests pass without modification
- [x] 2.4 Extend `test/lint-page-context.spec.ts` or add a spec so that no module under `src/reading/` reads the global `document` except through the parameter — the family of bugs where code runs in the wrong context must not regain a member here

## 3. The library

- [x] 3.1 `src/reader/library.ts`: the `cymbra-lingua-library` IndexedDB database with `books` and `files` object stores, keyed by the SHA-256 of the file; `importFile(file)`, `list()`, `get(hash)`, `remove(hash)`, `savePosition(hash, location, at)` (last write by `at` wins); unit tests under `fake-indexeddb`
- [x] 3.2 `src/reader/epub-meta.ts`: read the container and OPF for title, authors, language, cover image and `dc:identifier` (kept as metadata, never as key); sniff the zip's `mimetype` entry so a file with a generic type or no suffix still imports
- [x] 3.3 `src/reader/drm.ts`: parse `META-INF/encryption.xml`; refuse Adobe ADEPT, Readium LCP and Apple FairPlay declarations with the localised sentence; let IDPF and Adobe font obfuscation through; tests with fixture XML for each case
- [x] 3.4 Request persistent storage at first library open and surface a refusal in the library view, in the reader's language

## 4. The reader page

- [x] 4.1 `src/reader/reader.html`, `reader.ts`, `reader.css`: the library view (import button wired to `<input type="file" accept=".epub,application/epub+zip">`, book cards with cover, title, authors, delete) and the reading view (foliate-js host, tap zones, previous/next, table of contents, a toolbar carrying the HUD's percentage); add them to `build.mjs`'s entry points and copied pages for every variant
- [x] 4.2 `src/reader/renderer.ts`: the `BookRenderer` seam over foliate-js (open, goTo, next, prev, current location, `onSectionReady(doc, index)`, `onRelocate(location)`) and a fake implementation for tests; exclude `vendor/foliate-js/` and the thin adapter from lint, format and coverage, with the reason recorded in `vitest.config.ts`
- [x] 4.3 Mount `ReadingSession` on each section document from `onSectionReady`, with the port from `createLinguaPort()`; unmount on section unload; honour the global "Surlignage activé" toggle as the content script does
- [x] 4.4 The reveal rule of D4: hide the section until its first paint resolves or the cap elapses, then reveal once; paginated flow by default, scrolled flow as a setting rendered by `mountSettings` (one impl, every host)
- [x] 4.5 Save the position on every relocate through `savePosition`; reopen a book at its saved location; last write wins between two tabs
- [x] 4.6 Cards captured in the reader carry `"<book title> · <section title>"` in the local `source` field; the review surfaces show it where they show a page address; the sync path still pushes `source` empty — assert it in the sync tests
- [x] 4.7 Push the section's percentage to the badge with the `stats` message; make the popup recognise a reader tab by URL and show the section's figures without the "Analyser cette page" button
- [x] 4.8 Entry points: a "Bibliothèque" button in the popup and a row in the settings view (drawer and side panel), both sending `openPage("reader.html")`; the background focuses an existing reader tab instead of opening another
- [x] 4.9 Every string in French with the existing wording conventions (never "lemme"; a plain sentence for the protected-book refusal, the persistence refusal and an unreadable file), and no colour literal outside `tokens.css` (lint)
- [x] 4.10 On Safari, drop a phrase's selection when the finger lifts from it (`SelectionWatcher`, `dropPhraseOnLift` on the Safari build only, and only with the reader switched on and the page analysed), keep a single word's; measured first on the platforms: Safari (simulator and iPad) reports the lift after a handle drag, Firefox for Android does not. In the reader, foliate-js's finger pan kept from a touch working the selection (`touch-guard.ts`, Safari), and the card dismissed only on a real move, not when foliate settles the page in place after a lift. Verified on an iPad (2026-09-25): a word keeps its handles and its card, a phrase dragged out loses the callout and keeps its card, the callout stays with the reader switched off

## 5. Two underline styles

- [x] 5.1 In `tokens.css`, keep `underline dotted` for "learning" and switch "unknown" to `underline solid`; both tints unchanged
- [ ] 5.2 Check the two styles on a light page, a dark page and a greyscale rendering (a screenshot desaturated is enough), and on the Boox or the tablet in greyscale
  - Light page, dark page and greyscale renderings checked on the desk build (dotted vs solid stay apart). Pending: the Boox or the tablet in greyscale.

## 6. Prove it

- [x] 6.1 `yarn typecheck && yarn lint && yarn test && yarn format:check`, coverage still ≥ 80% with the vendored tree and the thin adapter excluded, and `yarn build` for all three variants; `yarn check:variants` passes (and, if 1.5 kept the reader out of Safari, verifies the page is absent from `dist-safari`)
- [ ] 6.2 Chrome desktop: import a novel and a technical book, read offline (network disabled in DevTools), highlights, popup, selection card, drawer, deck count; the badge and the popup on the reader tab; a second "Bibliothèque" click focuses the tab
  - Headless Chromium: both books imported, read with every request held to the extension (dead proxy: 0 left it), highlights, word popup, selection card, « + Deck » with the book as source, badge, `getStats` on the reader tab, second « Bibliothèque » focusing the tab, reopening at the saved position. Pending: a headed pass for the Side Panel opened from the reader's toolbar and the toolbar popup as it renders on a reader tab.
- [ ] 6.3 Firefox desktop and the Galaxy Tab S6 Lite: the same pass, plus airplane mode on the tablet; the picker on Android; one paint per page turn observed
  - Firefox 156 desktop (macOS), unpacked build via web-ext: a book imported, read, and translated in the reader (2026-09-23). Galaxy Tab S6 Lite (2026-09-25): books picked from the Android picker, read, the tap zones turn pages, and reading works in airplane mode, also after Firefox restarts. Pending: the rest of the pass on Firefox desktop, and one paint per page turn observed.
- [ ] 6.4 Safari macOS and an iPhone, if 1.5 kept the reader in the variant: the same pass; the file picker from the Files app; memory on the largest book
- [x] 6.5 Protected book refused (an ADEPT sample), an obfuscated-font book accepted (a Standard Ebooks title), a file renamed without `.epub` accepted
  - Samples built from a real book: an ADEPT declaration + `rights.xml` (refused with the sentence), an IDPF font declaration (imported, rendered), the file with no extension (imported).
- [x] 6.6 Delete a book with cards captured from it and confirm the cards keep their sentence and source text in review

## 7. Ship

- [x] 7.1 Update `STORE-LISTING.md`: the single-purpose statement gains the reader (books the reader imports, same purpose), the feature list names it, and the permission justifications state that no permission was added; refresh the Chrome, AMO and App Store descriptions accordingly
  - Chrome and AMO copy updated; the App Store paragraph is staged in apps/lingua-apple/STORE-LISTING.md, to paste once 1.5 keeps the reader in Safari.
- [x] 7.2 Add `apps/lingua-extension/vendor/VENDOR.md` (foliate-js commit, licence, refresh script) and `tool/vendor_foliate.sh`; confirm the AMO source archive carries the vendored tree
- [x] 7.3 Update `README.md` (reader page, library database, the document parameter, the reveal rule) and the `browser-extension-architecture` skill's context table with the reader page as an extension page that owns the library database
- [ ] 7.4 Release the extension, then dispatch `lingua-apple-release` with `deliver` so Safari receives the bundle; verify on the Boox when it arrives, with the same pass as 6.3
