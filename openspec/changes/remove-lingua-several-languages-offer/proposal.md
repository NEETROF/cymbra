# remove-lingua-several-languages-offer — the language choice and the Lingua page say nothing of price

## Why

Under its boxes, the extension's « Langues étudiées » carries the line « Plusieurs langues à la
fois : gratuit pour l'instant. », and its drafts "Several languages at once: free for now." and
« Varios idiomas a la vez: gratis por ahora. » (`severalLanguagesOffer`,
`src/i18n/{fr,en,es}/studied-languages.ts`). The site's Lingua pages end their languages card with
the same line, in French, English and Spanish (`languagesCard`, `apps/site/src/lib/lingua-text.ts`).
The line came with `enable-lingua-spanish` (its design D6) as a design decision: no requirement asks
for it.

The owner decided that the interface, the site's Lingua pages and the store listings carry no line
about price:
- **2026-10-09:** for the interface;
- **2026-10-10:** for the site's Lingua pages and the French store description.

The line leaves the interface and the site's Lingua pages, in French, English and Spanish.

The store listings already carry no such line, with one exception: the French description on the
Chrome Web Store and addons.mozilla.org. Change 53's implementation removes it from there
(`add-lingua-french-listings`, which already rewrites that file and pastes it with change 52's
release). This change does not edit the listing files.

Who sees the line today:
- **In the extension**, the box shows only when the reader's native language has two studied
  languages listed. French speakers have English and Spanish, so they see the line in Réglages and at
  onboarding. English and Spanish speakers would first see it with change 52
  (`enable-lingua-french`), which lists French for them: beside Spanish for English speakers, and
  beside English for Spanish speakers if fr-es ships.
- **On the site**, `/lingua/` and `/en/lingua/` show it. `/es/lingua/` is built once a pair glossed
  in Spanish ships (change 35), and would show it from then on.

This is row 53b of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
outside the 57. Change 53 (`add-lingua-french-listings`, design D2) records the decision and points
here.

## What Changes

- **The line leaves « Langues étudiées »** (design D1).
  - **Where:** Réglages in every host (the side panel, the popup, the page's drawer, the book reader)
    and onboarding, in every interface language, whether the reader studies one language or several.
  - **What stays:** the boxes and their one note, « Chaque page est lue dans celle de tes langues
    qu'elle contient. La première cochée sert aux réglages et aux statistiques par défaut. » (in
    English and Spanish, its drafts). No line replaces the one removed.
  - **Unchanged:** ticking, the last box that cannot be unticked, and the box hidden below two
    languages.
- **The key leaves the catalogue** (D2). `severalLanguagesOffer` is removed from the French source
  module and from its English and Spanish drafts. The drafts are typed after the French module, so a
  key left in either fails `yarn typecheck`. `mountStudiedLanguages` no longer creates its second
  note.
- **The line leaves the site's Lingua pages** (D3). Each language's `languagesCard` keeps its first
  sentence only:
  - « Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne. »;
  - "Choose your languages in Settings: each page is read in its own.";
  - « Elige tus idiomas en los Ajustes: cada página se lee en el suyo. ».
- **What moves, measured in copies of `main`** (D4).
  - **Extension: two tests.**
    - `test/studied-languages-view.spec.ts`'s « says that several languages at once are free for
      now » becomes a test that the box's notes are exactly its one note, in French, English and
      Spanish.
    - `test/settings-language.spec.ts` loses one assertion.
    - 3,133 tests become 3,135.
  - **Site: two of the 33 built pages**, `/lingua/` and `/en/lingua/`, each by that sentence and
    nothing else.
    - Their fixtures, `test/fixtures/lingua/main.{fr,en}.html`, are re-recorded.
    - `test/lingua-text.spec.ts` pins the card's text in the three languages.
    - 84 tests stay 84.
- **The French interface moves by one line, by the owner's decision.** The programme's rule « en-fr
  and es-fr output does not move, nor the French interface » is departed from for this line alone.
  No pack output moves.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED — *The reader chooses the languages they study*, which no
  open change holds (design D5).
  - It gains one sentence: the choice says nothing of price, in the settings or at onboarding, in any
    interface language.
  - Its text and its three scenarios are kept as they are, and two scenarios are added.
  - Nothing is REMOVED: no requirement ever named the line.
- `site-lingua-page`: ADDED — *The Lingua page says nothing of price*, with two scenarios.
  - **Why ADDED:** the page's one requirement, *The Lingua page names its languages and publishes
    their coverage*, does not name the line. It is also held by the open change
    `add-site-lingua-matrix-pages` (change 30), so this change does not modify it.
  - **No `archiveAfter`:** an ADDED requirement waits for nothing (design D5).

## Impact

- **Products.** Cymbra Lingua, and the site's Lingua pages:
  - `apps/lingua-extension`, *changed*:
    - `src/i18n/fr/studied-languages.ts`, `src/i18n/en/studied-languages.ts` and
      `src/i18n/es/studied-languages.ts` (one key each);
    - `src/reading/studied-languages-view.ts` (the second note and its comments);
    - tests: `test/studied-languages-view.spec.ts`, `test/settings-language.spec.ts`.

    *Consumed, unchanged*: `mountSettings` and the onboarding page, which mount the view.
  - `apps/site` (site), *changed*:
    - `src/lib/lingua-text.ts` (`languagesCard`, fr, en, es);
    - `test/lingua-text.spec.ts`;
    - `test/fixtures/lingua/main.fr.html` and `main.en.html`, re-recorded.

    There is no Spanish fixture: no page `/es/lingua/` is built on `main`.

  ID, Music, Live, the back office, the backend, `crates/`, the tables, packs, pins and goldens, the
  snapshots under `test/baseline/`, the store listing files, the Apple host app's own code, the
  agent plugin and every other page of the site are untouched.
- **Release.**
  - **The extension's next release** (Chrome Web Store, addons.mozilla.org, the Safari host app). On
    `main`, five bundles of each target carry the line: `content.js`, `onboarding.js`, `popup.js`,
    `reader.js` and `sidepanel.js` of `dist-chromium`, `dist-firefox` and `dist-safari`. After this
    change, none does.
  - **A site deploy** for `/lingua/` and `/en/lingua/`.

  Recommended: both no later than change 52's release, so English and Spanish speakers never see the
  line.
- **Compatibility.** No stored format, message, wire field, setting or route moves.
- **Order.** This change is independent of every open change and needs no `archiveAfter` (design D5):
  - change 52 does not touch the line;
  - change 53 removes it from the French store description and rewrites the listing files' notes
    about it. Its task 3.1 edits the lines of `lingua-text.ts` next to `languagesCard`, so whichever
    merges second rebases;
  - change 17's implementation (#786) edits the view's header comment, next to this change's lines
    but not on them.
- **Effort**: 0.5–1 ideal day.
