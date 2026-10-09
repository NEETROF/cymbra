# remove-lingua-several-languages-offer — the language choice says nothing of price

## Why

Under its boxes, the extension's « Langues étudiées » carries the line « Plusieurs langues à la
fois : gratuit pour l'instant. », and its drafts "Several languages at once: free for now." and
« Varios idiomas a la vez: gratis por ahora. » (`severalLanguagesOffer`,
`src/i18n/{fr,en,es}/studied-languages.ts`). It came with `enable-lingua-spanish` (its design D6),
as a design decision: no requirement asks for it.

The owner decided on 2026-10-09 that the interface carries no line about price, as the store
listings already don't: the App Store's, in every language, and the English and Spanish texts of
the Chrome Web Store and addons.mozilla.org. The line leaves the interface, in French, English and
Spanish.

Two texts outside the interface still carry the line:
- the French description on the Chrome Web Store and addons.mozilla.org;
- the site's Lingua pages, in French, English and Spanish.

Both are outside this change and left to the owner (design, open questions 1 and 2).

Who sees it today: the box shows only when the reader's native language has two studied languages
listed. French speakers have English and Spanish, so they see the line in Réglages and at
onboarding. English and Spanish speakers would first see it with change 52 (`enable-lingua-french`),
which lists French for them: beside Spanish for English speakers, and beside English for Spanish
speakers if fr-es ships.

This is row 53b of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside the 57. Change 53 (`add-lingua-french-listings`, design D2) records the decision and points
here.

## What Changes

- **The line leaves « Langues étudiées »** (design D1). This applies in Réglages in every host (the
  side panel, the popup, the page's drawer, the book reader) and at onboarding, in every interface
  language, whether the reader studies one language or several. The box keeps its boxes and its one
  note: « Chaque page est lue dans celle de tes langues qu'elle contient. La première cochée sert aux
  réglages et aux statistiques par défaut. » (in English and Spanish, its drafts). No line replaces
  it. Ticking, the last box that cannot be unticked, and the box hidden below two languages are
  unchanged.
- **The key leaves the catalogue** (D2): `severalLanguagesOffer` is removed from the French source
  module and from its English and Spanish drafts, which are typed after it, so a key left in either
  fails `yarn typecheck`. `mountStudiedLanguages` no longer creates its second note.
- **Two tests move** (D3), measured in a copy of `main` with the line removed. Only two assertions
  fail. `test/studied-languages-view.spec.ts`'s « says that several languages at once are free for
  now » becomes a test that the box's notes are exactly its one note, in French, English and
  Spanish. `test/settings-language.spec.ts` loses one assertion. 3,133 tests become 3,135.
- **The French interface moves by one line, by the owner's decision.** The programme's rule « en-fr
  and es-fr output does not move, nor the French interface » is departed from for this line alone.
  No pack output moves.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED — *The reader chooses the languages they study*, which no
  open change holds. It gains one sentence: the choice says nothing of price, in the settings or at
  onboarding, in any interface language. Its text and its three scenarios are kept as they are, and
  two scenarios are added (design D4). Nothing is REMOVED: no requirement ever named the line.

## Impact

- **Products.** Cymbra Lingua only:
  - `apps/lingua-extension`, *changed*:
    - `src/i18n/fr/studied-languages.ts`, `src/i18n/en/studied-languages.ts` and
      `src/i18n/es/studied-languages.ts` (one key each);
    - `src/reading/studied-languages-view.ts` (the second note and its comments);
    - tests: `test/studied-languages-view.spec.ts`, `test/settings-language.spec.ts`.

    *Consumed, unchanged*: `mountSettings` and the onboarding page, which mount the view.

  ID, Music, Live, the back office, the backend, the site, `crates/`, the tables, packs, pins and
  goldens, the snapshots under `test/baseline/`, the store listing files, the Apple host app's own
  code and the agent plugin are untouched.
- **Release.** The extension's next release (Chrome Web Store, addons.mozilla.org, the Safari host
  app). On `main`, five bundles of each target carry the line: `content.js`, `onboarding.js`,
  `popup.js`, `reader.js` and `sidepanel.js` of `dist-chromium`, `dist-firefox` and `dist-safari`.
  After this change, none does. Recommended: released no later than change 52, so English and
  Spanish speakers never see the line.
- **Compatibility.** No stored format, message, wire field or setting moves.
- **Order.** This change is independent of every open change and needs no `archiveAfter` (design D5):
  - change 52 does not touch the line;
  - change 53 rewrites only the listing files' notes about it;
  - change 17's implementation (#786) edits the view's header comment, next to this change's lines
    but not on them.
- **Effort**: 0.25–0.5 ideal day.
