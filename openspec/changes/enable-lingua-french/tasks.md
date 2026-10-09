# Tasks

Implemented once the prerequisites of proposal.md (Impact) are on `main` — changes 34 and 35, every
French change, `refine-lingua-fr-en-glosses` — and change 49's first committed measurement is known
(design D2). Where a task says « with fr-es », it is done only when fr-es's tables are committed.

## 1. The list (apps/lingua-extension, scripts/lingua-data, apps/site)

- [ ] 1.1 `packs.json` `["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"]` — `[…, "fr-en"]` when change 49 measured fr-es under its floor — and `check_variants`'s `SHIPPED_PAIRS` the same, its comment naming this change (D1, D2).
- [ ] 1.2 With fr-es: `scripts/lingua-data/testdata/fr-es/` (`forms.tsv`, `freq.tsv`, `gloss.tsv`, `mwe.tsv`, `level.tsv`, `manifest.json`, `NOTICE`), hand-written in `testdata/fr-en/`'s layout, glossed in Spanish, at French's current analyser version (D10).
- [ ] 1.3 `python3 scripts/lingua-data/gloss_coverage.py --write`, committing `apps/site/src/data/lingua-coverage.json` with fr-en's figures (and fr-es's); `test_gloss_coverage.py`: fr-en's and fr-es's committed figures « published once listed » (changes 48's and 49's tests), and a test that every pair of `packs.json` with a `FLOORS` entry measures at or above it on the committed tables (D2, D8).
- [ ] 1.4 The tests that read the default list or held a French pair inert, each keeping the earlier lists it named as explicit arguments (design *Measured*, D1, D5): `manifests.spec.ts`, `packs.spec.ts`, `pairs.spec.ts` (French's two pairs, English's and Spanish's two each — one for Spanish under the floor), `model-manifest.spec.ts` (change 50's French routes needed now; en-es needed by a Spanish-native reader of English or of French), `model-controller.spec.ts` (change 50's « nothing of fr-en is downloaded »), `translate-relay.spec.ts` where it names change 52; `crates/lingua-pack/tests/committed_tables.rs`: change 43's *The pack builds where the others' do* asserts `packs.json` lists fr-en; with fr-es, change 49's « listed nowhere » assertion goes, its scenario holding only while the list does not name fr-es (D11, D14).

## 2. The extension's French (apps/lingua-extension)

- [ ] 2.1 `src/analyzer/types.ts`: `StudiedLanguage = "en" | "es" | "fr"`; `src/state/profile.ts`: `NAMES` gains `French: "fr"`, its comment no longer saying French is dropped; `yarn typecheck` clean (change 51 on `main`) (D3).
- [ ] 2.2 Tests (D3): `profile.spec.ts` — a stored profile studying Spanish then French, English native, read as `["es", "fr"]`; `studied-languages-view.spec.ts` — offered Spanish and French to an English speaker, English and French to a Spanish speaker (English alone, hidden, without fr-es), English and Spanish to a French speaker, never French; `language-labels.spec.ts`, `settings-language.spec.ts`, `onboarding-level-row.spec.ts`, `popup.spec.ts`, `stats-view.spec.ts` — French studied in the English and Spanish interfaces, the labels of design *Measured* (estimated level titles, the note, the ladder's borrowed sizes, the prompt, « no text »); `wordpopup.spec.ts` — a French card's studied words carry `lang="fr"`. The French interface's existing assertions pass unchanged.

## 3. The memory sentence (apps/lingua-extension)

- [ ] 3.1 `src/translate/host/model-controller.ts`: the cost's flag `twoModels`, set when the models the reader's pairs need are two (`needs.needed.length > 1`), replacing `pivot`; `src/translate/model-messages.ts` and `src/reading/translation-setting.ts` read it; the copy's keys and text unchanged (D6).
- [ ] 3.2 Tests: `model-controller.spec.ts` and `translation-setting.spec.ts` — an English-native reader of Spanish and French: 52,475,767 B and « about 340 MB »; of French alone: 26,234,715 B and « about 200 MB »; a Spanish-native reader of French: 51,608,069 B and « unos 340 MB »; every row of design D6's table for the four earlier pairs as before; `model-messages.spec.ts` for the renamed flag.

## 4. The manifest, the comments and the documents

- [ ] 4.1 `_locales/en/messages.json` `extensionDescription`: Spanish and French (change 53's draft, design D7, or the owner's words); with fr-es, `_locales/es`: English and French; each translator's `description` naming the pairs it follows; `_locales/fr` untouched; `yarn check:version` (112 characters, and change 53's rule that each description names what its readers can study, once 53 is on `main`) (D7).
- [ ] 4.2 The comments that name change 52 as the future — `src/translate/markup.ts`, `src/analyzer/language-labels.ts`, `tool/soak_engine.mjs`, `scripts/lingua-data/SOURCES.md`, the French tables' `README.md`, and whatever the French changes add before this one (`git grep -n "change 52\|enable-lingua-french"`) —; `README.md` and `REVIEWERS.md` (what ships, French for English speakers, and for Spanish speakers with fr-es), `TRANSLATION.md` (the routes table), `tool/marks/README.md` (« marked »), as changes 34 and 35 did for es-en and en-es (D11).

## 5. Checks

- [ ] 5.1 In `apps/lingua-extension`: `yarn gen:pack` and `yarn gen:pack:real` build every listed pack; `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; the built manifests carry `_locales/fr`, `en` and `es` and no other, `default_locale` English; the bundles' growth against `main`, every target, and each zipped package's size in the pull request, against design *Measured* (≈ 12.1 MB with six pairs, ≈ 10.8 MB with fr-en alone).
- [ ] 5.2 Nothing else moves (design *What moves*): `cargo test -p lingua-wasm` — the English, Spanish, es-en, en-es and French baselines, `cross_native.rs` and change 51's `fr_es_baseline` — without re-blessing; `cargo test -p lingua-pack`; `git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables` empty; `wasm-pack test --node crates/lingua-wasm` (nothing of it moves); the extension's snapshots pass as committed; `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 5.3 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `python3 scripts/lingua-data/gloss_coverage.py --check`. In `apps/site`: `yarn test`, `yarn build`, `yarn check:routes` — the three Lingua pages name the French pairs. The `lingua-apple-build` workflow green, `CFBundleLocalizations` `[fr, en, es]` (D7).
- [ ] 5.4 `openspec validate enable-lingua-french --strict` passes, and `python3 scripts/openspec_archive_order.py enable-lingua-french` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` still open (0 once they are archived). The MODIFIED blocks of changes 43's, 48's and (with fr-es) 49's requirements re-copied from those changes as they then stand, only D14's clauses differing (the diff in the pull request), and the archive simulated again on a scratch copy of `openspec/`; under the floor, 49's block and `add-lingua-pack-fr-es` in `archiveAfter` removed (D14).

## 6. Dogfood (owner, with Claude where a session can drive the browser)

- [ ] 6.1 [manual] Chrome (macOS), Firefox (macOS), Firefox for Android, Safari (macOS), Safari (iOS): the pass of design D12, on test accounts, with the English interface and (with fr-es) the Spanish one; it carries change 47's task 6.2 and change 51's task 6.2, whose findings go to this pull request; the owner ticks those two tasks once their findings are in.

## 7. Release (owner)

- [ ] 7.1 [manual] The prerequisites of proposal.md (Impact) met — among them a release that reads backup schema 3 live on every store, at the latest `fix-lingua-lemma-lookup`'s (D4) — the English (and Spanish) descriptions read (M9, Open Question 1), Open Questions 2–5 answered, and the owner's go-ahead before merge.
- [ ] 7.2 [manual] TestFlight first; the first store submission carrying French carries change 53's listings and the descriptions of task 4.1; the site deployed after the merge, with fr-en's (and fr-es's) figures, before the listings are pasted; each store submission is the owner's (D13, M18).
- [ ] 7.3 Change 52 is marked done in `docs/lingua/language-matrix-programme.md`, saying whether fr-es shipped.
