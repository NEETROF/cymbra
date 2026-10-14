# add-lingua-comeback-hints — the toolbar icon says what is waiting, and the end of a review asks, rarely, for a rating

## Why

Guillaume, today nothing in Cymbra Lingua brings a reader back, and nothing asks the readers who
like it to say so where other people choose extensions.

- **The icon only speaks on an analysed page.** The badge shows the page's known percentage, a
  dash « — » when the page has nothing to count, and nothing everywhere else
  (`apps/lingua-extension/src/background.ts`, the `stats` listener). A reader with twelve cards
  due who opens a new tab, a mail or a page in their own language sees no sign of them.
- **The popup hides the deck until the page is analysed.** « Deck (en cours) » and « Réviser (N) »
  sit inside `#controls`, which `renderStats` hides when the tab has no reading session
  (`src/popup/render.ts`, `$("controls").hidden = !present`). On Chromium without the « always
  highlight » grant, that is most tabs. Yet `lingua-decks-review` already says the due count
  SHALL be visible in the icon popup.
- **The due count overstates the work.** `dueCount` counts every never-reviewed card as due
  (`crates/lingua-core/src/decks/fsrs.rs`, `is_due` is true when `due` is `None`), while a session
  lets in only the day's allowance of new words (10 by default). A reader who captured 60 words
  sees « Réviser (60) » and gets 10. The review itself already refuses to trust that figure:
  `ReviewController.next()` measures « Encore 10 » by starting a session instead
  (`src/review/session.ts`).
- **Nobody has rated the extension.** The Firefox Add-ons listing shows version 1.9.0 with no
  rating and no review (checked on 2026-10-13); the Chrome Web Store and the App Store have none
  either. Ratings are what a stranger reads first on a store page, and the readers most likely
  to leave one are those who just finished a review session — exactly the moment the review end
  (« Séance terminée ») already marks.

Both hints stay inside your earlier decisions: no notification, no timer or alarm, nothing on the
page while reading, nothing about what is read leaving the device.

## What Changes

- **The toolbar icon counts the cards waiting** where it shows no percentage: on tabs not
  analysed, on pages with nothing to count (the dash goes), and where highlighting is switched
  off. It shows the number of cards a review would hold **today** across the studied languages —
  the reviewed cards due plus the new words within each language's daily allowance — nothing at
  0, « 99+ » above 99. On an analysed page the percentage stays. The count has no « % » and a
  colour of its own (teal against the percentage's violet, black on the e-ink colour presets);
  the icon's title says it in words, language by language.
- **Computed from events only.** The background works the count out from the reader's own deck
  when the data changes (an answer, a capture, a sync, a restore, a review made in the Cymbra
  Lingua app) and refreshes it when the browser starts, a tab is activated or a window focused.
  No alarm, no timer, no new permission, and working out the count sends nothing. **So the count can
  be behind:** a card that comes due at 14:00 while you are away from the browser shows at your next
  tab switch, not at 14:00; cards reviewed on your phone still count until this browser syncs.
- **The synchronisation is not changed.** On Chrome the icon now wakes the extension at the first
  tab switch after half a minute of quiet, and the extension's start already syncs a signed-in
  reader: more exchanges with the server than today, counted in the dogfood. Changing when the
  start syncs is left to a separate change if that count is too high (design, *Follow-up*).
- **A switch in Réglages** (French UI copy: « Afficher les cartes à réviser sur l'icône »), in
  « Rythme de révision », on by default, kept on the device.
- **One figure everywhere — a visible change for today's readers.** The popup's « Réviser (N) »,
  the review page's « N à revoir » and the badge all count what a review would hold today, from one
  core function. A reader who captured 60 words and reviewed none sees « Réviser (10) » where they
  see « Réviser (60) » today; without this, the icon would say 10 and the popup 60.
- **The popup offers the review on every tab**, analysed or not; where the in-page drawer cannot
  open (a Firefox or Safari tab without the reader), the review page opens in a tab.
- **A rating request at the end of a review session, at most twice**, in the side panel and the
  in-page drawer on Chrome and Firefox: after the fifth finished session, on the third day of
  review or later, a short line and three buttons (French UI copy: « Noter Cymbra Lingua »,
  « Plus tard », « Ne plus demander »). « Plus tard » (or closing without answering) asks once more
  after ten more sessions; the request is shown at most twice, ever — you asked for a one-time
  request, see the design's open question 6. It offers nothing in exchange and does not depend on
  how the session went. The store page opens only when the reader chooses to rate, through the
  background, so the page being read is never sent as a referrer. What the request remembers stays
  on the device.
- **Safari and the Cymbra Lingua app follow Apple's rule** (App Review Guideline 5.6.1: use the
  provided API, custom review prompts are disallowed): the Safari extension shows no request; the
  app asks through StoreKit's own prompt at the end of a session, at most once per version; a
  plain « Noter Cymbra Lingua » link in Réglages › Données, in every browser, opens the store page
  (the App Store's write-review page in Safari).
- **Safari before version 18 shows no count**: there one badge serves every tab, and a count would
  overwrite the page percentages. Its setting is not offered there.
- Copy in French, English and Spanish (`src/i18n/{fr,en,es}/`), one sentence in the privacy
  annex in the site's three languages.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-browser-extension`: modified — *Page percentage always visible* (the badge away from an
  analysed page); added — the toolbar icon counts the cards waiting for review, and Réglages carry a
  link to rate the extension.
- `lingua-decks-review`: modified — *Review present right next to the reading* (the due count is
  what a review would hold today, in the popup on every tab); added — the end of a session asks, at
  most twice, for a rating of the extension.
- `lingua-apple-app`: added — the app asks for an App Store rating only through the system's prompt.

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.**
  - `crates/lingua-core` (`decks/review.rs`): the count of cards a review would hold today and a
    small outlook of when more come due, from the deck alone.
  - `crates/lingua-wasm`: the engine's count, and a standalone `reviewOutlook` over a backup that
    needs no pack, as `reprofileBackup` already is.
  - `apps/lingua-extension`: the background's badge (`background.ts` and a new pure module), the
    popup (`src/popup/`), the review page and its end (`src/review/`), Réglages
    (`src/reading/settings-view.ts`), a store-links constant, the catalogue's `popup`, `review`
    and `settings` modules and a new `badge` module in fr/en/es.
  - `apps/lingua-apple`: the bridge's `review.sessionFinished`, a review-request policy in
    `LinguaSignIn`, StoreKit's request in `ViewController.swift`.
  - `apps/site`: one sentence in Annex B of the privacy policy (fr, en, es).
- Cymbra ID, Music, Live and the back office are untouched. No backend, no `.proto`, no flag, no
  new browser permission (so no new consent prompt on update). The backup format and the sync's
  code are unchanged; two new device-only preferences and one derived key live in
  `chrome.storage.local`. On Chromium the background starts more often (tab switches, window
  focus), and its existing start-time sync with it.

**Order.** No open change modifies the two requirements this change modifies (*Page percentage
always visible*, *Review present right next to the reading*), checked against `main` on 2026-10-14:
`add-lingua-card-sentence-translation`, `add-lingua-card-edit` and
`add-lingua-review-translation-fallback` add or modify other `lingua-decks-review` requirements
(the card, its gloss, its edit — not the end of a session nor the popup), and the last two modify
other `lingua-apple-app` requirements, where this change only adds one. No `archiveAfter` is
needed. Two overlaps in code, not in specs: the app's page–native bridge (`src/app/bridge.ts`,
`AppBridgeMessage`), which `add-lingua-review-translation-fallback` also extends with `auth.*`
messages — whichever lands second rebases its message cases; and Annex B of the privacy policy, which
`update-lingua-privacy-annex-languages` and `add-lingua-review-translation-fallback` also edit —
this change's sentence reaches the live site only after that change's deploy gate (its task 3.2).
