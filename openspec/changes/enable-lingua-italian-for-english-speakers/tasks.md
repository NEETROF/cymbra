# Tasks

Implemented once the prerequisites of proposal.md (Impact) are on `main` — changes 22, 23, 24 and 26, and change 20 released.

## 1. The list

- [x] 1.1 `packs.json` ends with `"it-fr", "it-en"`; `check_variants`'s `SHIPPED_PAIRS` the same, its comment naming this change; `scripts/lingua-data/testdata/it-en/`, the it-fr fixture glossed in English — its Italian files byte for byte `testdata/it-fr/`'s; `crates/lingua-pack/tests/pipeline_testdata.rs`: `the_it_en_fixture_is_the_italian_fixture_glossed_in_english` and it-en's row in the fixtures' pins (D1).
  - `testdata/it-en/`: `forms.tsv`, `freq.tsv`, `level.tsv` and `tags.tsv` byte for byte it-fr's (it-fr's fixture carries no `grammar.tsv` or `lexical.tsv`), the manifest's analyser version and `levels_estimated` the same, `pack_version` `0.0.1-fixture`; its 132 glosses and 16 expressions in English. `yarn gen:pack` builds it-en's layer on it-fr's base, byte for byte. Its whole pack pinned at 7,221 B (`f5bfe833…`).
- [x] 1.2 `gloss_coverage.py --write` with it-en's figures; `test_gloss_coverage.py`: it-en « published once listed », `ListedFloorsTest` naming it (D2, D7).
  - it-en's figures, published: 91.3 / 84.2 / 75.8 % at the 5,000, 10,000 and 20,000 commonest lemmas. `--write` also moves it-fr's to 89.4 / 82.0 / 73.4 (from #1058's 89.2 / 81.8 / 73.3): change 17 refined it-fr's tables on `main` after #1058 measured them, so #1058's file is stale on the stacked base, and `--check` passes only with the refresh. `test_it_fr_is_listed_and_published_once_listed` reads it-fr after the French pairs, no longer last.
- [x] 1.3 The tests that read the list (D9), each keeping the earlier lists as explicit arguments; `engine.spec.ts`; `committed_tables.rs` asserts `packs.json` lists it-en.
  - `packs.spec.ts`, `pairs.spec.ts`, `manifests.spec.ts` (it-en adds a layer and no base), `model-manifest.spec.ts`, `studied-languages-view.spec.ts`, `app-translation.spec.ts`, `check-version.spec.ts` keep change 20's list as an explicit argument; `engine.spec.ts`: an English reader fetches the Italian base then it-en's layer, a Spanish reader is refused before any fetch; `committed_tables.rs`: `spec_scenario_it_en_is_shipped_once_italian_is_enabled_for_english_speakers`.

## 2. The extension

- [x] 2.1 Tests (D3): `studied-languages-view.spec.ts` — Spanish, French and Italian offered to an English speaker, Italian to no Spanish speaker; `profile.spec.ts` — Spanish then Italian, English native; `language-labels.spec.ts`, `settings-language.spec.ts`, `onboarding-level-row.spec.ts`, `popup.spec.ts`, `stats-view.spec.ts` — Italian in the English interface ("Estimated Italian level"); `wordpopup.spec.ts` — an Italian card in English, "past historic (passato remoto)".
  - `language-labels.spec.ts`: every message of `every("en", "it")`; `settings-language.spec.ts`: Spanish, French then Italian offered on the eight pairs, "Estimated Italian level" and its note, never Italian for a Spanish speaker; `onboarding-level-row.spec.ts`, `popup.spec.ts`, `stats-view.spec.ts` in English; `wordpopup.spec.ts`: « parlò », "third-person singular past historic (passato remoto) of parlare", every Italian word `lang="it"`. Nothing in `src/` needed a change (D3).
- [x] 2.2 `translate-relay.spec.ts`: it-en marked if change 24 listed it; `model-controller.spec.ts` and `translation-setting.spec.ts`: the two English cases of D5; `model-residency.spec.ts` and `native-language.spec.ts` (D8, D9).
  - it-en is in `MARKED_PAIRS` (change 24, OQ2): marked; it-es is now the one marked pair not shipped. Measured from the catalogue: it-en alone 26,242,301 B ("26.2 MB", "about 200 MB"); Spanish, French and Italian 78,718,068 B ("78.7 MB", "about 340 MB"); the two scenarios carry the figures. `model-residency.spec.ts`: es-en, fr-en, it-en, es-en — the least recently used one-model route gives way; `native-language.spec.ts`: a French-native reader of Italian switching to English keeps Italian on the eight pairs, dropped on the seven.

## 3. The description, the activation page and the documents

- [x] 3.1 `_locales/en/messages.json` `extensionDescription` (D6's draft or the owner's words), its translator's `description` naming Italian (it-en) beside Spanish (es-en) and French (fr-en); `yarn check:version`.
  - The owner's version A (OQ1, 2026-10-25), 108 characters: "The web in Spanish, French or Italian: unknown words highlighted, an honest percentage. Offline and private." `check-version.spec.ts`: it-en listed with change 52's English fails for English, the new description before it-en fails too, both pass together.
- [x] 3.2 `copy.js` `en` `lede` and `step3` (D6), its header comment and the `en` table's comment; `apple-activation-page.spec.ts`: the English page names Spanish, French and Italian, in that order, and its comment follows.
  - D6's drafts (OQ1). The French and Spanish tables untouched; `apple-activation-page.spec.ts` gains *Italian for an English speaker*, and the Spanish page still names no Italian.
- [x] 3.3 The comments and documents of D9 (`git grep -n "enable-lingua-italian-for-english-speakers\|change 25"`, keeping the Italian programme's hits).
  - `README.md` (eight layers and four bases, 13.0 MB), `REVIEWERS.md` (twelve files, what an English-speaking reader downloads), `TRANSLATION.md` (it-en's route shipped, marked), `tool/marks/README.md`, `src/translate/markup.ts`, `src/app/translation.ts`, `src/i18n/README.md`, `scripts/lingua-data/SOURCES.md`, `tables/it-en/README.md`, `tool/check_variants.mjs`. « change 25 » also names the matrix programme's change 25 in `TRANSLATION.md` and `tool/marks/README.md`: left.

## 4. Checks

- [x] 4.1 In `apps/lingua-extension`: `yarn gen:pack`, `yarn gen:pack:real`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; each zipped package's size in the pull request.
  - All green on the stacked base (`yarn fetch:engine` first). 168 files, 4,538 tests, lines 98.4 %; the variant check passes on the fixture and the real parts; `check:version` passes for fr, en and es. Built manifests: `_locales` `en`, `es`, `fr`, `default_locale` `en`. Zipped as the release zips them: Chromium 13,541,150 B, Firefox 13,524,030 B, Safari 13,526,607 B (it-en's layer 910,661 B unzipped). Parts: eight layers and four bases, 13,043,624 B.
- [x] 4.2 Nothing else moves: `cargo test -p lingua-pack`, `cargo test -p lingua-wasm` — every baseline without re-blessing; `git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables ':!*README.md'` empty; `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`.
  - `cargo test -p lingua-pack -p lingua-wasm`: 393 passed, no baseline re-blessed; the tables diff is empty; fmt and clippy clean.
- [ ] 4.3 The Python tests and `gloss_coverage.py --check`; in `apps/site`: `node test/support/refresh-lingua-fixtures.mjs`, `test/astro/lingua-page.spec.ts` counting eight rows, `yarn test`, `yarn build`, `yarn check:routes`; the `lingua-apple-build` workflow green, `CFBundleLocalizations` still `[fr, en, es]`.
  - Python 1,178 tests OK; `--check` passes. The site needs change 21's names of Italian (#1057, not on this base): its build fails on the stacked base, as #1058's does. Run on a scratch merge with #1057: fixtures refreshed (the English page names Italian for English speakers, an « Italian → English » row), `lingua-page.spec.ts` counting eight rows, `yarn test` (243), `yarn build`, `yarn check:routes` (61) green; the files copied here. `lingua-apple-build` runs in CI only: waits for the pull request.
- [ ] 4.4 `openspec validate enable-lingua-italian-for-english-speakers --strict`; the archive order, `refine-lingua-it-en-glosses` added to `archiveAfter` once proposed; the MODIFIED blocks re-copied from change 20 as it then stands, only D12's clauses differing; the archive simulated on a scratch copy of `openspec/`.
  - Valid; `openspec_archive_order.py` exits 10 naming the `archiveAfter` changes and their own predecessors (`add-lingua-apple`, `enable-lingua-french`) still open. The MODIFIED blocks compared with change 20's as they stand on #1058: only D12's clauses differ, but for the translation requirement's two French scenarios, re-copied (52,0 Mo, 78,2 Mo). The simulated archive stops upstream, at `add-lingua-italian-tokenisation` (its MODIFIED block renames two scenarios main's spec holds), before reaching this change: to run again once the earlier Italian changes archive.

## 5. Dogfood and release (owner)

- [ ] 5.1 [manual] The pass of design D10 on the five targets, in the English interface.
- [ ] 5.2 [manual] The owner reads the drafts (D6, M9) and answers the Open Questions.
- [ ] 5.3 [manual] The owner's go-ahead before merge; released on its own, TestFlight first, with change 26's listings, the site deployed first; each submission the owner's (D11).
- [ ] 5.4 Change 25 marked done in `docs/lingua/italian-programme.md`.
