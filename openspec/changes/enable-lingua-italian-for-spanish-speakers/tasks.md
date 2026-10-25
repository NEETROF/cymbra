# Tasks

Implemented once the prerequisites of proposal.md (Impact) are on `main`: changes 25–29 and 31, and change 20 released.

## 1. The list

- [x] 1.1 `packs.json` gains `"it-es"` at the end of the list, after `"it-en"` or after any Italian-native pair listed first (D1, D13). `check_variants`'s `SHIPPED_PAIRS` is the same, and its comment names this change.
  - `…, "it-fr", "it-en", "it-es"`: no Italian-native pair is listed (OQ4, D13's second branch).
- [x] 1.2 `scripts/lingua-data/testdata/it-es/`: the it-fr fixture glossed in Spanish, its Italian files byte for byte `testdata/it-fr/`'s.
  - `forms.tsv`, `freq.tsv`, `level.tsv` and `tags.tsv` byte for byte it-fr's (it-fr's fixture carries no `grammar.tsv` or `lexical.tsv`); the manifest is it-en's with `native` `es` (analyser `1.4.0`, `pack_version` `0.0.1-fixture`); its 132 glosses and 16 expressions in Spanish. `yarn gen:pack` builds it-es's layer on it-fr's base, byte for byte.
- [x] 1.3 `crates/lingua-pack/tests/pipeline_testdata.rs`: `the_it_es_fixture_is_the_italian_fixture_glossed_in_spanish` and it-es's row in the fixtures' pins.
  - Its whole pack pinned at 7,112 B (`d1bf4b65…`), a new row; no existing pin moved.
- [x] 1.4 `gloss_coverage.py --write` with it-es's figures; `test_gloss_coverage.py`: it-es « published once listed », and `ListedFloorsTest` naming it (D2, D7).
  - it-es's figures, published: 85.2 / 76.1 / 65.1 % at the 5,000, 10,000 and 20,000 commonest lemmas, against the floor 83.2 / 74.0 / 63.1; no other pair's figure moved. `test_it_en_is_listed_and_published_once_listed` reads it-en right after it-fr, no longer last; `test_spec_scenario_it_es_s_floor_holds` checks the published figures once listed.
- [x] 1.5 The tests that read the list (D9), each keeping the earlier lists as explicit arguments; `engine.spec.ts`; `committed_tables.rs` asserts that `packs.json` lists it-es.
  - `packs.spec.ts`, `pairs.spec.ts`, `manifests.spec.ts` (it-es adds a layer and no base), `model-manifest.spec.ts`, `studied-languages-view.spec.ts` (change 25's « Italian to no Spanish speaker » on its eight pairs), `app-translation.spec.ts`, `check-version.spec.ts`, `model-controller.spec.ts` keep change 25's list as an explicit argument; `engine.spec.ts`: a Spanish reader fetches the English base and en-es's layer, then the Italian base and it-es's layer, change 25's refusal kept on its eight pairs; `committed_tables.rs`: `spec_scenario_it_es_is_shipped_once_italian_is_enabled_for_spanish_speakers`, and it-en's twin reads it-en after it-fr.

## 2. The extension

- [x] 2.1 The offer's tests (D3), in the Spanish interface:
  - `studied-languages-view.spec.ts`: inglés, francés and italiano offered to a Spanish speaker;
  - `profile.spec.ts`: English then Italian, Spanish native;
  - `language-labels.spec.ts`, `settings-language.spec.ts`, `onboarding-level-row.spec.ts`, `popup.spec.ts`, `stats-view.spec.ts`: « Nivel de italiano estimado »;
  - `wordpopup.spec.ts`: an Italian card in Spanish, « pretérito perfecto simple de indicativo ».
  - Done as listed; `studied-languages-view.spec.ts` also offers English then Italian when fr-es is under its floor. `wordpopup.spec.ts`: « parlò », « tercera persona del singular del pretérito perfecto simple de indicativo de parlare », every Italian word `lang="it"`. Nothing in `src/` needed a change (D3).
- [x] 2.2 Translation's tests:
  - `translate-relay.spec.ts`: it-es marked if change 29 put it on the tier;
  - `model-controller.spec.ts` and `translation-setting.spec.ts`: the two Spanish cases of D5;
  - `model-residency.spec.ts` and `native-language.spec.ts` (D8, D9).
  - it-es is in `MARKED_PAIRS` (change 29, OQ2): marked; no marked pair is left unshipped. Measured from the catalogue: Italian alone 51,615,655 B (« 51,6 MB », « unos 340 MB »); English, French and Italian 77,850,370 B (« 77,9 MB », « unos 340 MB »); the two scenarios carry the figures. `model-residency.spec.ts`: en-es, fr-es, it-es, fr-es — en-es never deleted, fr-en then it-en give way; `native-language.spec.ts`: a reader of Italian switching to Spanish keeps it on the nine pairs, drops it on the eight.
- [x] 2.3 If the owner withholds it-es's translation (D4):
  - `routes["it-es"]` dropped from `model-manifest.json`, and its tests follow;
  - the three texts of D4 MODIFIED here, each copied from its latest holder.
  - Not taken: it-es is on the first tier (change 29, OQ2), its route kept and marked.

## 3. The description, the activation page and the documents

- [x] 3.1 `_locales/es/messages.json` `extensionDescription`: D6's draft or the owner's words. Its translator's `description` names Italian (it-es) beside English (en-es) and French (fr-es). Then `yarn check:version`.
  - The owner's words (OQ1, 2026-10-25), 105 characters: « La web en inglés, francés o italiano: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión. » `check-version.spec.ts`: it-es listed with change 52's Spanish fails for Spanish, the new description before it-es fails too, both pass together.
- [x] 3.2 `copy.js`: the `es` `lede` and `step3` (D6), its header comment and the `es` table's comment. `apple-activation-page.spec.ts`: the Spanish page names inglés, francés and italiano, in that order, and its comment follows.
  - D6's drafts (OQ1). The French and English tables untouched; `apple-activation-page.spec.ts` gains *Italian for a Spanish speaker*, and change 25's « the Spanish page names no Italian » is dropped.
- [x] 3.3 The comments and documents of D9 (`git grep -n "enable-lingua-italian-for-spanish-speakers\|change 30"`), keeping the Italian programme's hits.
  - `README.md` (nine layers and four bases, 13.4 MB), `REVIEWERS.md` (thirteen files, what a Spanish-speaking reader downloads), `TRANSLATION.md` (it-es's route shipped, marked; the three-model table's Spanish row holds), `tool/marks/README.md`, `src/translate/markup.ts`, `src/app/translation.ts`, `src/i18n/README.md`, `scripts/lingua-data/SOURCES.md`, `tables/it-es/README.md`, `tool/check_variants.mjs`, `test/word-card-it-es.spec.ts`, `test/translate-marks.spec.ts`. « change 30 » also names other programmes' change 30 (`apps/site/test/support/lingua.ts`, the store listings, older proposals): left. The site's `glossesOrigin` sentences are change 31's (its D6), not this change's: untouched.

## 4. Checks

- [x] 4.1 In `apps/lingua-extension`: `yarn gen:pack`, `yarn gen:pack:real`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`. Each zipped package's size goes in the pull request.
  - All green on the stacked base (`yarn fetch:engine` first). 168 files, 4,567 tests, lines 98.4 %; the variant check passes on the fixture and the real parts; `check:version` passes for fr, en and es. Zipped as the release zips them: Chromium 13,878,533 B, Firefox 13,861,409 B, Safari 13,863,990 B (it-es's layer 345,542 B unzipped). Parts: nine layers and four bases, 13,389,166 B.
- [x] 4.2 Nothing else moves:
  - `cargo test -p lingua-pack` and `cargo test -p lingua-wasm`, every baseline without re-blessing;
  - `git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables ':!*README.md'` is empty;
  - `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
  - `cargo test -p lingua-pack -p lingua-wasm`: 394 passed, no baseline re-blessed (it-es's fixture pin is a new row); the tables diff against `origin/main` is empty; fmt and clippy clean.
- [ ] 4.3 The Python tests and `gloss_coverage.py --check`. In `apps/site`: `node test/support/refresh-lingua-fixtures.mjs`, `test/astro/lingua-page.spec.ts` counting a row per listed pair (nine to eleven), `yarn test`, `yarn build`, `yarn check:routes`. The `lingua-apple-build` workflow green, `CFBundleLocalizations` unchanged by this change.
  - Python 1,179 tests OK; `--check` passes. The site needs change 21's names of Italian (#1057) and changes 26's and 31's site patches, none on this base. Run on a scratch merge with #1057 and both patches: fixtures refreshed (the French and English pages gain an « Italien → espagnol » / « Italian → Spanish » row; `taken-with.json` carries it-es), `lingua-page.spec.ts` counting nine rows, `yarn test` (254), `yarn build`, `yarn check:routes` (61) green; the Spanish home card reads « Lee la web en inglés, en francés o en italiano… »; the four files copied here. `lingua-apple-build` runs in CI only: waits for the pull request; no Apple project file is touched.
- [ ] 4.4 OpenSpec:
  - `openspec validate enable-lingua-italian-for-spanish-speakers --strict`;
  - the archive order, `refine-lingua-it-es-glosses` named;
  - the MODIFIED blocks re-copied as D13 says: from change 50 if implemented, else from change 45 if implemented, else from change 25 with `enable-lingua-italian-speakers` dropped from `archiveAfter` (or from `openspec/specs/` if archived), only D12's clauses differing, in the branch's wording of the list (D13);
  - the archive simulated on a scratch copy of `openspec/`.
  - Valid; `openspec_archive_order.py` exits 10, naming the `archiveAfter` changes and their predecessors (`add-lingua-apple`, `enable-lingua-french`) still open, `refine-lingua-it-es-glosses` among them. D13's second branch: the list's and memory blocks re-copied from change 25 as it stands on its branch, only D12's clauses differing (its measured figures kept, the Spanish scenarios carrying 51,6 MB and 77,9 MB), `enable-lingua-italian-speakers` dropped from `archiveAfter`; *Guided activation* is change 25's block plus D12's two clauses. The simulated archive stops upstream, at `add-lingua-italian-tokenisation` (its block renames two scenarios main's spec holds), as change 25's did: to run again once the earlier Italian changes archive.

## 5. Dogfood and release (owner)

- [ ] 5.1 [manual] The pass of design D10 on the five targets, in the Spanish interface, with the sample of pivot glosses.
- [ ] 5.2 [manual] The owner reads the drafts (D6, M9) and answers the Open Questions, Open Question 2 at change 29's figures.
- [ ] 5.3 [manual] The owner's go-ahead before merge. Released on its own, TestFlight first, with change 31's listings, the site deployed first; each submission the owner's (D11).
- [ ] 5.4 Change 30 marked done in `docs/lingua/italian-programme.md`.
