# Design — remove-lingua-several-languages-offer

## Context

See proposal.md (Why). What exists on `main`:

| Where | What |
|---|---|
| `apps/lingua-extension/src/i18n/fr/studied-languages.ts` | the box's French module, the source: `studiedNote` and `severalLanguagesOffer` (« Plusieurs langues à la fois : gratuit pour l'instant. »), its doc comment citing `enable-lingua-spanish` D6 |
| `…/src/i18n/en/studied-languages.ts`, `…/src/i18n/es/studied-languages.ts` | the drafts, typed `typeof fr`: "Several languages at once: free for now.", « Varios idiomas a la vez: gratis por ahora. » |
| `…/src/reading/studied-languages-view.ts` | `mountStudiedLanguages`: the boxes (`.set-languages`), then two `.set-note` — `studiedNote`, then `severalLanguagesOffer`; `offerFor(native)` hides the block when the reader's native language has fewer than two studied languages listed |
| `…/src/reading/settings-view.ts` | Réglages' « Langues étudiées » block, mounted by every host (side panel, popup, the page's drawer, the book reader), in the interface language (`settingsCopy`) |
| `…/src/onboarding/onboarding.ts` | onboarding's languages step, the same view, handed the French module until change 17 (#786) hands it the page's language |
| `…/test/studied-languages-view.spec.ts`, `…/test/settings-language.spec.ts` | the extension's two tests that name the line |
| `apps/site/src/lib/lingua-text.ts` | the Lingua page's text tables, fr, en, es; `languagesCard` is the body of the page's last card (« 🧭 Anglais et espagnol → français »), two sentences, the second the line |
| `apps/site/test/lingua-text.spec.ts` | pins the French card's body, line included; the English and Spanish bodies are not pinned |
| `apps/site/test/fixtures/lingua/main.{fr,en}.html` | the `<main>` of `/lingua/` and `/en/lingua/`, byte for byte, read by `test/astro/lingua-page.spec.ts` (`yarn test`) and `test/post-build/lingua.spec.ts` (`yarn check:routes`); refreshed as `apps/site/README.md` says |

The line's origin is `enable-lingua-spanish`'s design D6 and task 1.4 (change 28 of the Spanish
programme), carried to the site's Lingua page. No requirement of `openspec/specs/` names it, in
`lingua-browser-extension` or in `site-lingua-page`.

## Decisions

### D1 — What the box shows after the change

The line is removed, not reworded: no other text takes its place.

| Reader | Today | After |
|---|---|---|
| Offered fewer than two languages (English and Spanish speakers on `main`) | box hidden | box hidden, unchanged |
| Offered two or more, French interface (French speakers: English, Spanish) | boxes, « Chaque page est lue dans celle de tes langues qu'elle contient. La première cochée sert aux réglages et aux statistiques par défaut. », « Plusieurs langues à la fois : gratuit pour l'instant. » | boxes, the first note alone |
| Offered two or more, English interface (from change 52: Spanish, French) | boxes, "Each page is read in whichever of your languages it holds. The first one checked is the default for the settings and the statistics.", "Several languages at once: free for now." | boxes, the first note alone |
| Offered two or more, Spanish interface (from change 52 if fr-es ships: English, French) | boxes, « Cada página se lee en aquel de tus idiomas que contenga. El primero marcado es el predeterminado para los ajustes y las estadísticas. », « Varios idiomas a la vez: gratis por ahora. » | boxes, the first note alone |

The number of languages the reader studies changes nothing. The line showed whether one box or
several were ticked, and the box after the change is the same either way. This holds in Réglages in
every host and at onboarding: both mount `mountStudiedLanguages`, so the one removal covers them.

### D2 — The key leaves the catalogue

`severalLanguagesOffer` and its doc comment are removed from the French module. The key is also
removed from the English and Spanish drafts: they are typed `typeof fr`, so an object literal that
kept the key would fail `yarn typecheck` on an excess property. The key is removed, not left empty:
`test/i18n.spec.ts` refuses an empty entry, and nothing would read it.

`mountStudiedLanguages` creates one `.set-note` and appends `row, note`. The comments follow:
- the view's header says "Its note", not "Its two notes";
- the French module's header says the onboarding page carries this one text;
- the comment citing D6 goes with the element.

`StudiedLanguagesCopy` (`settings-copy.ts`) is `typeof` the French module and needs no edit. The
module list in `src/i18n/README.md` is unchanged: the module keeps its note.

### D3 — The site's Lingua pages

Each table's `languagesCard` keeps its first sentence and loses the second. The card's title, key
and place are unchanged:

| Page | Today | After |
|---|---|---|
| `/lingua/` | « Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne. Plusieurs langues à la fois : gratuit pour l'instant. » | « Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne. » |
| `/en/lingua/` | "Choose your languages in Settings: each page is read in its own. Several languages at once: free for now." | "Choose your languages in Settings: each page is read in its own." |
| `/es/lingua/` (built from change 35) | « Elige tus idiomas en los Ajustes: cada página se lee en el suyo. Varios idiomas a la vez: gratis por ahora. » | « Elige tus idiomas en los Ajustes: cada página se lee en el suyo. » |

The site's other pages are not part of this decision, and nothing else on the Lingua pages speaks
of price. Their only near word is the levels' sentence about a CEFR list that "can be shipped
freely", which is about its licence.

**The tests.** `test/lingua-text.spec.ts` pins the card's body in the three languages:
- the French one, already pinned, loses the line;
- the English test (« English: today's sentence… ») and the Spanish one (« Spanish: English for
  Spanish speakers first… ») each gain one assertion of the card's body.

The fixtures are re-recorded with `apps/site/README.md`'s command, after `yarn build` with
`PUBLIC_DISCORD_URL` unset. `taken-with.json` does not move: the pairs, figures and routes are the
same.

### D4 — What moves, measured

Both measurements were made on copies of `main` in a scratch directory:
- **the extension:** on 2026-10-09 at d8cddb31, with the generated stubs and the engine rebuilt
  (`yarn gen:proto`, `yarn gen:wasm`);
- **the site:** on 2026-10-10 at 3043f023, after `yarn install --immutable`.

Nothing under `apps/` moved between the two commits. Since then, `main` has moved by #848, whose only
file under `apps/` is the snapshot `test/baseline/word-card-en-es.txt`; it touches no file of this change.

**The extension.**
- **With the line removed and the tests unchanged**, exactly two assertions fail:
  - `test/settings-language.spec.ts` › the studied languages' block › « An English-native reader:
    its notes are English, and so are the languages' names » (it expects "Several languages at once:
    free for now.");
  - `test/studied-languages-view.spec.ts` › « Langues étudiées » › « says that several languages at
    once are free for now ».

  The other 3,131 tests pass.
- **The tests after the change.**
  - The first test loses that one assertion and keeps the rest (the English note, the languages'
    English names).
  - The second becomes « says nothing of price, in its one note », run for `fr`, `en` and `es`. It
    mounts the box with `settingsCopy(language).studiedLanguages` and two languages offered, and
    expects it shown, with its `.set-note` texts exactly `[studiedNote]`.

  123 files, 3,133 → 3,135 tests (one test becomes three).
- **Gates:** `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test` (line coverage gate at
  80 %), `yarn build` and `yarn check:variants` all pass. `check_variants.mjs` reads nothing of the
  line.
- **Bundles.**
  - On `main`, the line is in `content.js`, `onboarding.js`, `popup.js`, `reader.js` and
    `sidepanel.js` of each target (`dist-chromium`, `dist-firefox`, `dist-safari`).
  - After the change, no bundle of any target holds any of the three texts.

**The site.**
- **With the line removed and the tests unchanged**, three assertions fail:
  - in `yarn test`: the French card of `test/lingua-text.spec.ts`, and `test/astro/lingua-page.spec.ts`'s
    « /lingua/ renders the fixture's <main>, byte for byte » and its `/en/lingua/` twin;
  - in `yarn check:routes`, after the build: `test/post-build/lingua.spec.ts`'s French and English
    « <main> is the fixture's, byte for byte ».
- **After the change:** `yarn check`, `yarn typecheck`, `yarn test` (84 tests, as on `main`),
  `yarn build` and `yarn check:routes` (33 passed, 1 skipped, as on `main`) pass.
- **The built site.**
  - Of 33 HTML pages, `lingua/index.html` and `en/lingua/index.html` differ from `main`'s build.
  - Each differs only by that sentence: `sienne. Plusieurs langues à la fois : gratuit pour
    l'instant.</p>` → `sienne.</p>`, and `own. Several languages at once: free for now.</p>` →
    `own.</p>`.
  - No asset under `_astro/` moves.
  - The fixtures change by the same sentence and nothing else.

**What does not move:**
- the snapshots under `apps/lingua-extension/test/baseline/` and the goldens of `crates/lingua-wasm`,
  since neither renders the box;
- packs, tables, pins, analyser versions, and every crate (no Rust);
- the stored profile and backup;
- `apps/lingua-extension/STORE-LISTING.md` and `apps/lingua-apple/STORE-LISTING.md` (change 53's,
  see *Settled questions*);
- the site's other 31 pages, its routes, `taken-with.json` and `src/data/lingua-coverage.json`;
- every other text of the extension's catalogue and of the site's tables.

### D5 — The spec deltas, and the order with the other changes

**`lingua-browser-extension`: MODIFIED** *The reader chooses the languages they study*, the
requirement that specifies the box.
- **It can be modified.** No open change holds it: checked on `main` and on the spec deltas of every
  open pull request. Change 52 (#847) only ADDS *French is studied by readers of English and
  Spanish*.
- **One sentence is added**: the choice says nothing of price, in the settings or at onboarding, in
  any interface language.
- **Its text and its three scenarios are kept**: *Adding Spanish*, *The last language*, *Every
  reader today*, word for word, so the archive keeps every scenario.
- **Two scenarios are added**: the French interface, and the English and Spanish ones.
- **Nothing is REMOVED.** The line was only ever a design decision (D6 of `enable-lingua-spanish`),
  never a requirement.

**`site-lingua-page`: ADDED** *The Lingua page says nothing of price*, with two scenarios: today's
French and English pages, and the Spanish page once it is built.
- The capability's one requirement, *The Lingua page names its languages and publishes their
  coverage*, does not name the line, and the open change `add-site-lingua-matrix-pages` (change 30)
  holds it.
- Modifying it would mean an `archiveAfter` on change 30, whose last task is the owner's deploy.
  The requirement would also gain a sentence about something other than coverage.
- An ADDED requirement waits for nothing. Its name is not one of the two change 53 ADDS to the same
  capability (*The Lingua page says what each studied language's card and levels are*, *The
  coverage table reads at a phone's width*).

**The order:**
- **Change 52 (`enable-lingua-french`)** lists a second pair for English and Spanish speakers, so
  their box shows. Recommended: this change is released, and the site deployed, no later than change
  52, so English and Spanish speakers never see the line.
  - In code the two are independent: change 52 does not touch the line.
  - Its tests on `studied-languages-view.spec.ts` are about which languages are offered.
- **Change 53 (`add-lingua-french-listings`)** records the decision in its D2.
  - Its implementation carries the French store description's removal (*Settled questions*, 1) and
    rewrites the listing files' notes on « Several languages at once ». This change edits neither
    listing file.
  - Its task 3.1 renames `spanishCard`/`spanishLevels` in `apps/site/src/lib/lingua-text.ts`. They
    are the lines just above `languagesCard` in the French table, so whichever merges second
    rebases.
- **Change 17's implementation (#786)** edits the header comment of `studied-languages-view.ts`, a
  few lines above the comment this change edits. Whichever merges second rebases trivially.
- **`enable-lingua-spanish`** introduced the line. Its design D6 and task 1.4 are left as they are,
  the record of that change; this change supersedes D6.

None of these needs `archiveAfter`: no other open change modifies or adds a requirement this change
modifies, and its ADDED requirement is new.
`python3 scripts/openspec_archive_order.py remove-lingua-several-languages-offer` exits 0.

## Risks / Trade-offs

- **The French interface moves**, against the programme's rule « …nor the French interface ».
  Accepted by the owner's decision: one line, named here, and nothing else of the French catalogue.
- **The French store description keeps the line until change 53's release** → change 53 pastes the
  listings with change 52's release. The site and the extension may drop the line earlier.

## Settled questions

1. **The French store description** — settled by the owner on 2026-10-10 (in session): **the line
   is removed there too, by change 53's implementation.**
   - The description in question is that of the Chrome Web Store and addons.mozilla.org
     (`apps/lingua-extension/STORE-LISTING.md`, *Description*, FR). It reads « Choisissez dans les
     Réglages les langues que vous apprenez : chaque page est lue dans la sienne. Plusieurs langues
     à la fois : gratuit pour l'instant. ».
   - Change 53 already rewrites that file and pastes it with change 52's release.
   - This change does not edit the file. The App Store descriptions, and the English and Spanish
     texts of the other two stores, never carried the line.
2. **The site's Lingua pages** — settled by the owner on 2026-10-10 (in session): **the line is
   removed there too, in this change** (D3), with the site deployed in the owner's release task.
