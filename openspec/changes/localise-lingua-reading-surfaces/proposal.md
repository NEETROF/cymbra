# localise-lingua-reading-surfaces — the reading surfaces read their copy from the catalogue

## Why

Change 14 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 1, a silent release, and the first of the four that move the interface's copy into the
catalogue change 13 built. The reading surfaces are what a reader meets on a page and in a book:
the popup and its page, the HUD, the injected drawer, the word card, the selection card, the side
panel's page, and the EPUB reader with its library. Their copy — ≈ 150 unique texts in
`popup/popup.ts` and `popup.html`, `reading/{hud,drawer,wordpopup,selection-card}.ts`,
`sidepanel.html`, `reader/{app,copy,library}.ts` and `reader.html` — is still literals beside the
catalogue's entries for them.

Each surface starts reading its copy from change 13's `src/i18n/<language>/<surface>.ts`, picked
by the interface language read before the surface is built and handed to it with the copy; the static French
of its HTML page is filled from the catalogue before the page shows and the page's `lang`
follows; each file comes off the lint's baseline. For a reader of French nothing moves: the
catalogue's French is the literals byte for byte, and a test asserts it for every surface — the
spec files that exist, and new ones for the popup, the drawer and the side panel, which have
none today.

## What Changes

- **The popup** (`popup.ts`, `popup.html`): the status, the level call to action, the counts, the
  account line, the buttons; the page keeps its skeleton and ids, its text nodes filled at mount.
- **The HUD and the drawer** (`hud.ts`, `drawer.ts`): the labels, the aria labels, the tab names.
- **The word card** (`wordpopup.ts`): « Mot à mot… », the waiting and no-gloss texts, the page
  labels, « forme vue », the listen labels (« ▶ Mot », « ▶ Sélection »…), the actions, « Fermer ».
  The grammar lines are change 18's renderer.
- **The selection card** (`selection-card.ts`): the kind labels and the rarity bands, their numbers
  through the catalogue's formatting.
- **The side panel's page** (`sidepanel.html`): the tab names.
- **The reader** (`reader/copy.ts`, `app.ts`, `library.ts`, `reader.html`): `copy.ts` keeps
  exporting `COPY` in today's shape (`importFailed.{protected, storage, …}` included), built from
  the catalogue's `reader` module for the interface language, so `reader-app.spec.ts` passes
  unchanged; `library.ts` and `reader.html` read the module; `reader/reader.ts` builds the book's
  `ReadingSession` as `content.ts` does, and hands it the language and the copy the same way.
- **`lang`** on each page and injected host, from the interface language (the studied words
  inside a grammar line are change 18's).
- **The baseline** loses these files; the numbers and percentages in these files go through the
  catalogue's `formatNumber` and `formatPercent`, the French forms unchanged.
- **The content script reads the key before it builds the session**, which hands each surface
  its copy and the interface language at construction; what an open surface does when the key
  changes is change 20's (it rebuilds the session and reloads the pages).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: ADDED *The reading surfaces speak the interface language* — the
  surfaces named, their pages' `lang`, their copy from the catalogue, the French unchanged.

No requirement is modified; every requirement that quotes these surfaces' French copy is read
under the umbrella rule of change 13.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (the files above, `src/i18n/{fr,en,es}/
  {popup,hud,drawer,card,selection,sidepanel,reader}.ts`, `test/lint-copy.spec.ts`'s baseline, the
  tests that render these surfaces). ID, Music, Live, the back office and the site are untouched.
- **No byte moves.** The spec files asserting these surfaces' French copy (`wordpopup`,
  `selection-card`, `rarity-text`, `hud`, `reader-app`) pass unchanged, and new specs for the
  popup, the drawer and the side panel assert the text their pages and modules held; the bundles
  grow by the English and Spanish copy of these surfaces (measured per entry in the pull request).
- **Order.** After change 13 (archived after it); independent of 15–17; before 18, 19 and 20,
  which build on the hand-over.
- **Not here.** Réglages and its blocks (15); review and statistics (16); account and onboarding
  (17); the grammar lines (18); the languages' names (19).
