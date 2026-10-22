# enable-lingua-italian — Italian as a studied language: it-fr ships for French speakers

## Why

Change 20 of the [Italian programme](../../../docs/lingua/italian-programme.md), the stage-1 release:
French speakers learn Italian through it-fr — today's readers, and no Italian text to write (I5).
Everything Italian needs is built silently before it, each a precondition listed under Impact, not a
fact today: the translation models (2), the split pack format (3, 4), the variant and backup schema 4
(5), the tokenisation, analysis and detection guard (6, 7, 8), the forms, unmarked spellings, acronyms,
expression keys, grammar and levels (9–14), the voice (15), it-fr's glosses and their refinement (16,
17), the card (18), the marks (19), and the site's description of Italian with the listings (21).
Every one of those changes leaves the same edits to this one: `packs.json` listing it-fr, and the
extension's own studied-language type learning `it` — as matrix change 52, `enable-lingua-french`,
did for French.

Measured on a scratch copy of `main` (design, *Measured*): with the type widened, four type errors,
two filled by change 15, one by change 18, one a test of this change's; with it-fr listed, 100 of
4,347 tests fail — 93 throw until change 15's catalogue names Italian, and 8 remain then: one for
change 5's analyser constant, seven that read the list, this change's. Three gaps no earlier change
closes: the word card's guard row for Italian's neighbours, keyed by the studied-language type; the
French description, which the version check requires to name what the listed pairs study and which
no longer fits 112 characters with a third language said the direct way; and online translation,
whose backend refuses any pair with Italian.

## What Changes

- **it-fr ships** (design D1, D2): `packs.json` becomes `["en-fr", "es-fr", "es-en", "en-es",
  "fr-en", "fr-es", "it-fr"]`, `check_variants`'s list the same. Italian comes last, so every native
  language keeps its default pair; French speakers are offered English, Spanish and Italian. it-fr is
  listed only at or above change 16's floor and once change 17 has fixed its known defects (Open
  Question 1).
- **The extension studies Italian** (D3): `StudiedLanguage` gains `it`; `profile.ts` reads a stored
  profile's `Italian`. Réglages and onboarding offer Italian to a French speaker after English and
  Spanish; « Niveau d'italien estimé » (change 14), the card's Italian forms named in French (change
  18), the voice of Italy (change 15). `GUARDED.it` keeps a word card on a page declaring one of the
  neighbours change 8's guard keeps (Catalan, Galician, Venetian, Sicilian, Neapolitan, Sardinian,
  Corsican, Friulian) from being read in Italian.
- **Translation as M15 settled it** (D5): it-fr through it-en then en-fr, the model host serving it-en
  after the owner's redeploy (change 2); marked if change 19 put it on the first tier, else offered
  without a mark unless the owner decides otherwise (Open Question 2).
- **Online translation accepts Italian** (D5): the backend's language list (`LANGUAGES`) gains `it`
  and names it Italian, deployed before the extension's release.
- **The memory sentence** (D6): a French speaker reading English, Spanish and Italian needs three
  models, which the engine holds two at a time: « environ 340 Mo », as for two; the download is the
  catalogue's sum, ≈ 78,2 Mo. The requirement reads « two models or more ».
- **The French description and activation page** (D7): `_locales/fr` names Italian within 112
  characters — recommended « Le web en anglais, espagnol ou italien : mots inconnus surlignés,
  pourcentage honnête. Hors ligne et privé. » (107) —, `check_version`'s `LANGUAGE_NAMES` learns
  Italian; the host app's French `lede` and `step3` name English, Spanish and Italian. The owner reads
  the drafts (M9). `_locales/en` and `es`, the English and Spanish pages, and `CFBundleLocalizations`
  do not move.
- **The published coverage** (D8): `gloss_coverage.py --write` publishes it-fr's figures; the site's
  French Lingua page and home card then name Italian from change 21's description, with no code
  change here.
- **Backup v4 in readers' hands** (D4): a release reading schema 4 — the first cut after change 5's
  implementation — is live on every store before this change's release, so a rollback keeps an Italian
  reader's data.
- **Requirements true of their time** (D13): change 9's « no package SHALL list it-fr », change 4's
  list, change 2's *Every reader today*, the guard requirement's lists of neighbours, and two of
  `enable-lingua-french`'s (*Guided activation*'s French page, the memory sentence) MODIFIED, each
  archived after the change that holds it.
- **A dogfood pass on five targets** (D11) in the French interface, carrying change 15's voice
  checklist; TestFlight first; the owner's go-ahead before the merge and before each submission (I12).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *The shipped pairs are one list* (held by change 4,
  `ship-lingua-pack-bases-once`): the list ends with it-fr; *No French for French speakers* and
  *French for English speakers alone* name it-fr; *French speakers learning Italian* and *No Italian
  for English and Spanish speakers* added.
  MODIFIED *Italian's forms and frequencies* (change 9): a package carries it-fr once the list names
  it, in the requirement and in *The reference and the pipeline*; every other word verbatim.
- `lingua-browser-extension`: ADDED *Italian is studied by readers of French*. MODIFIED *No word is
  read in a language whose guard keeps the document's language out* (held by
  `read-lingua-card-in-another-studied-language`): Italian's neighbours added, *A Sicilian page*
  added.
- `lingua-translation`: MODIFIED *The setting's memory follows the models the reader's pairs need*
  (held by `enable-lingua-french`): « two models or more », two scenarios added. MODIFIED
  *Translation models are listed in one catalogue, with a route per pair* (held by change 2): *Every
  reader today* says a French-native reader also downloads it-en once they accept Italian.
- `lingua-apple-app`: MODIFIED *Guided activation* (held by `enable-lingua-french`): *The French page*
  names Italian once it-fr is listed; *Every language a French speaker can study* added.

Change 2's *A route of an Italian pair*, change 16's « no package SHALL list it before Italian is
enabled » and change 19's scenarios stay true as written. The online-translation requirement already
asks the service to accept any pair of Lingua's languages: the backend follows it, the requirement
does not move.

## Impact

- **Products.** Cymbra Lingua only:
  - `apps/lingua-extension` — *changed*: `packs.json`, `tool/check_variants.mjs`,
    `src/analyzer/types.ts`, `src/state/profile.ts`, `src/analyzer/language-labels.ts`,
    `src/reading/session.ts` (`GUARDED`, `THREE_LETTER`'s `srd` and `cos`), `tool/check_version.mjs`
    (`LANGUAGE_NAMES`), `_locales/fr/messages.json` and `manifest.json`'s French literal, the
    comments that name `enable-lingua-italian` as the future, `README.md`, `REVIEWERS.md`,
    `TRANSLATION.md`, `tool/marks/README.md`; the tests of design D10.
  - `scripts/lingua-data/` — *changed*: `test_gloss_coverage.py`; *consumed*: `gloss_coverage.py`,
    every table, unchanged.
  - `crates/lingua-pack/tests/committed_tables.rs` — change 9's scenario asserts the list names it-fr.
  - `apps/site` — `src/data/lingua-coverage.json`, written by the script, and the fixtures
    refreshed from the built pages (`test/fixtures/lingua/`, `test/astro/lingua-page.spec.ts`'s row
    count); no page or text changes.
  - `backend/lingua` — `src/online_translation_core.rs`: `LANGUAGES` gains `it`, `language_name`
    names it Italian, with a test; deployed before the extension's release.
  - `apps/lingua-apple` — `Shared (App)/Resources/copy.js` (`fr` `lede`, `step3`) and
    `Base.lproj/Main.html` (its two French nodes).

  ID, Music, Live and the back office (its name for `it` is change 21's) are untouched; of the
  backend, only online translation's list moves: the server stores any studied language a client
  sends, so Italian cards, statuses and days sync with no other change. lingua-core, lingua-wasm, the packs, every table and every golden: untouched.
- **Prerequisites, all the owner's** (I12, M18):
  - Implementations merged: changes 2–19 of the programme, change 16's floor fixed by the owner and
    its tables committed at or above it, change 17 (Open Question 1), and change 21's description of
    Italian on the site (D8).
  - The model host redeployed with it-en (change 2's task 4.1), and the backend deployed with
    online translation's Italian (D5).
  - A release that reads backup schema 4 live on the Chrome Web Store, addons.mozilla.org and the App
    Store (D4).
  - The owner's readings: the description and the activation page's drafts (M9), change 19's
    figures (Open Question 2), change 16's sample.
  - Test accounts for the dogfood.
  - Change 21's listings in the same submission (D12).
- **Release.** The first packages in which Italian is studied, released on their own (Open Question
  4), TestFlight first; the site deployed after the merge, with it-fr's figures, before the listings
  are pasted. Nothing is submitted by this pull request.
- **Order.** After every stage-1 change and change 21's implementation; before stage 2 (it-en, it-es),
  which widens nothing this change has not. Archived after the changes of `.openspec.yaml`.
- **Effort, against 4–7.5 ideal days**: 3.85–7.75 (design, *Effort*), the dogfood the uncertain part.
