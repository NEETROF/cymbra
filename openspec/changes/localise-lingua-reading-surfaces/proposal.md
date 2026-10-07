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

Each surface starts reading its copy from `src/i18n/<language>/<surface>.ts`, picked by the
interface language it reads with its other preferences before rendering; the static French of its
HTML page is filled from the catalogue at mount and the page's `lang` follows; each file comes
off the lint's baseline. For a reader of French nothing moves: the catalogue's French is the
literals byte for byte, and the spec files that assert French copy pass unchanged — they are the
check.

## What Changes

- **The popup** (`popup.ts`, `popup.html`): the status, the level call to action, the counts, the
  account line, the buttons; the page keeps its skeleton and ids, its text nodes filled at mount.
- **The HUD and the drawer** (`hud.ts`, `drawer.ts`): the labels, the aria labels, the tab names.
- **The word card** (`wordpopup.ts`, `gloss-pages.ts`): « Mot à mot… », the waiting and no-gloss
  texts, the page labels, « forme vue », the listen labels (« ▶ Mot », « ▶ Sélection »…), the
  actions, « Fermer ». The grammar lines are change 18's renderer.
- **The selection card** (`selection-card.ts`): the kind labels and the rarity bands, their numbers
  through the catalogue's formatting.
- **The side panel's page** (`sidepanel.html`): the tab names.
- **The reader** (`reader/copy.ts`, `app.ts`, `library.ts`, `reader.html`): the library's and the
  reader's copy, already an object, becomes the catalogue's module.
- **`lang`** on each page and injected host, from the interface language.
- **The baseline** loses these files; the formats « fr-FR » in these files go through the catalogue.

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
  `selection-card`, `rarity-text`, `hud`, `drawer`, `reader-app`, `popup`…) pass unchanged; the
  French bundles grow by the English and Spanish copy of these surfaces (measured per entry in the
  pull request).
- **Order.** After change 13; independent of 15–17.
- **Not here.** Réglages and its blocks (15); review and statistics (16); account and onboarding
  (17); the grammar lines (18); the languages' names (19).
