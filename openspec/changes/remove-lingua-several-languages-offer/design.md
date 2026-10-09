# Design — remove-lingua-several-languages-offer

## Context

See proposal.md (Why). What exists on `main`:

| Where | What |
|---|---|
| `src/i18n/fr/studied-languages.ts` | the box's French module, the source: `studiedNote` and `severalLanguagesOffer` (« Plusieurs langues à la fois : gratuit pour l'instant. »), its doc comment citing `enable-lingua-spanish` D6 |
| `src/i18n/en/studied-languages.ts`, `src/i18n/es/studied-languages.ts` | the drafts, typed `typeof fr`: "Several languages at once: free for now.", « Varios idiomas a la vez: gratis por ahora. » |
| `src/reading/studied-languages-view.ts` | `mountStudiedLanguages`: the boxes (`.set-languages`), then two `.set-note` — `studiedNote`, then `severalLanguagesOffer`; `offerFor(native)` hides the block when the reader's native language has fewer than two studied languages listed |
| `src/reading/settings-view.ts` | Réglages' « Langues étudiées » block, mounted by every host (side panel, popup, the page's drawer, the book reader), in the interface language (`settingsCopy`) |
| `src/onboarding/onboarding.ts` | onboarding's languages step, the same view, handed the French module until change 17 (#786) hands it the page's language |
| `test/studied-languages-view.spec.ts`, `test/settings-language.spec.ts` | the two tests that name the line |

The line's origin is `enable-lingua-spanish`'s design D6 and task 1.4 (change 28 of the Spanish
programme). No requirement of `openspec/specs/` names it.

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

### D3 — What moves, measured

Measured on 2026-10-09 in a scratch copy of `apps/lingua-extension` at `main` (d8cddb31; `main` has
since moved by docs only, nothing under `apps/` or `crates/`). Generated stubs and the engine were
rebuilt from the copy (`yarn gen:proto`, `yarn gen:wasm`).

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
- **Gates in the copy:** `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test` (line
  coverage gate at 80 %), `yarn build` and `yarn check:variants` all pass. `check_variants.mjs`
  reads nothing of the line.
- **Bundles.**
  - On `main`, the line is in `content.js`, `onboarding.js`, `popup.js`, `reader.js` and
    `sidepanel.js` of each target (`dist-chromium`, `dist-firefox`, `dist-safari`).
  - After the change, no bundle of any target holds any of the three texts.

**What does not move:**
- the snapshots under `test/baseline/` and the goldens of `crates/lingua-wasm`, since the box is
  rendered by neither;
- packs, tables, pins, analyser versions, and every crate (no Rust);
- the stored profile and backup;
- `apps/lingua-extension/STORE-LISTING.md`, `apps/lingua-apple/STORE-LISTING.md`, `apps/site`;
- every other text of the catalogue.

### D4 — The spec delta: MODIFIED, the scenarios kept

The behaviour belongs to *The reader chooses the languages they study* (`lingua-browser-extension`),
the requirement that specifies the box.
- **It can be modified.** No open change holds it: checked on `main` and on the spec deltas of every
  open pull request. Change 52 (#847) only ADDS *French is studied by readers of English and
  Spanish*.
- **One sentence is added**: the choice says nothing of price, in the settings or at onboarding, in
  any interface language.
- **Its text and its three scenarios are kept**: *Adding Spanish*, *The last language*, *Every
  reader today*, word for word, so the archive keeps every scenario.
- **Two scenarios are added**: the French interface, and the English and Spanish ones, each quoting
  the line that is no longer shown.
- **Nothing is REMOVED.** The line was only ever a design decision (D6 of `enable-lingua-spanish`),
  never a requirement.

### D5 — Order with the other changes

- **Change 52 (`enable-lingua-french`)** lists a second pair for English and Spanish speakers, so
  their box shows. Recommended: this change is released no later than change 52, so English and
  Spanish speakers never see the line. In code the two are independent: change 52 does not touch the
  line, and its tests on `studied-languages-view.spec.ts` are about which languages are offered.
- **Change 53 (`add-lingua-french-listings`)** records this decision in its D2. Its tasks 1.1 and 1.3
  rewrite the listing files' notes on « Several languages at once ». This change does not edit
  either listing file.
- **Change 17's implementation (#786)** edits the header comment of `studied-languages-view.ts`, a
  few lines above the comment this change edits. Whichever merges second rebases trivially.
- **`enable-lingua-spanish`** introduced the line. Its design D6 and task 1.4 are left as they are,
  the record of that change; this change supersedes D6.

None of these needs `archiveAfter`: no other open change modifies or adds the requirement this change
modifies. `python3 scripts/openspec_archive_order.py remove-lingua-several-languages-offer` exits 0.

## Risks / Trade-offs

- **The French interface moves**, against the programme's rule « …nor the French interface ».
  Accepted by the owner's decision: one line, named here, and nothing else of the French catalogue.
- **The same line elsewhere** (open questions 1 and 2) → until the owner answers, the line can still
  be read in the French store description and on the site's Lingua pages. This change does not
  decide for them.

## Open Questions

1. **The French store description.** The French description of the Chrome Web Store and
   addons.mozilla.org (`apps/lingua-extension/STORE-LISTING.md`, *Description*, FR) still reads
   « Choisissez dans les Réglages les langues que vous apprenez : chaque page est lue dans la sienne.
   Plusieurs langues à la fois : gratuit pour l'instant. ». The App Store descriptions, and the
   English and Spanish texts of the other two stores, do not carry the line. Does the French
   description keep it?
   - If not, its natural home is change 53's implementation: change 53 already rewrites this file
     (its task 1.1 keeps the FR description unchanged) and pastes the texts with change 52's
     release.
   - Removing the line here instead would mean pasting the French description into both dashboards
     on its own.
2. **The site's Lingua pages.** cymbra.app's Lingua page shows the line in its languages card, in
   French, English and Spanish (`languagesCard`, `apps/site/src/lib/lingua-text.ts`, and its test
   `test/lingua-text.spec.ts`, and the recorded pages under `test/fixtures/lingua/`). Does it keep
   it? If not, it can go in either of two places:
   - change 53's task 3.1, which already edits that file;
   - a site change of its own, deployed on its own.
