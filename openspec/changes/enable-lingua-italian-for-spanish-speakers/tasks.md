# Tasks

Implemented once the prerequisites of proposal.md (Impact) are on `main`: changes 25–29 and 31, and change 20 released.

## 1. The list

- [ ] 1.1 `packs.json` gains `"it-es"` at the end of the list, after `"it-en"` or after any Italian-native pair listed first (D1, D13). `check_variants`'s `SHIPPED_PAIRS` is the same, and its comment names this change.
- [ ] 1.2 `scripts/lingua-data/testdata/it-es/`: the it-fr fixture glossed in Spanish, its Italian files byte for byte `testdata/it-fr/`'s.
- [ ] 1.3 `crates/lingua-pack/tests/pipeline_testdata.rs`: `the_it_es_fixture_is_the_italian_fixture_glossed_in_spanish` and it-es's row in the fixtures' pins.
- [ ] 1.4 `gloss_coverage.py --write` with it-es's figures; `test_gloss_coverage.py`: it-es « published once listed », and `ListedFloorsTest` naming it (D2, D7).
- [ ] 1.5 The tests that read the list (D9), each keeping the earlier lists as explicit arguments; `engine.spec.ts`; `committed_tables.rs` asserts that `packs.json` lists it-es.

## 2. The extension

- [ ] 2.1 The offer's tests (D3), in the Spanish interface:
  - `studied-languages-view.spec.ts`: inglés, francés and italiano offered to a Spanish speaker;
  - `profile.spec.ts`: English then Italian, Spanish native;
  - `language-labels.spec.ts`, `settings-language.spec.ts`, `onboarding-level-row.spec.ts`, `popup.spec.ts`, `stats-view.spec.ts`: « Nivel de italiano estimado »;
  - `wordpopup.spec.ts`: an Italian card in Spanish, « pretérito perfecto simple de indicativo ».
- [ ] 2.2 Translation's tests:
  - `translate-relay.spec.ts`: it-es marked if change 29 put it on the tier;
  - `model-controller.spec.ts` and `translation-setting.spec.ts`: the two Spanish cases of D5;
  - `model-residency.spec.ts` and `native-language.spec.ts` (D8, D9).
- [ ] 2.3 If the owner withholds it-es's translation (D4):
  - `routes["it-es"]` dropped from `model-manifest.json`, and its tests follow;
  - the three texts of D4 MODIFIED here, each copied from its latest holder.

## 3. The description, the activation page and the documents

- [ ] 3.1 `_locales/es/messages.json` `extensionDescription`: D6's draft or the owner's words. Its translator's `description` names Italian (it-es) beside English (en-es) and French (fr-es). Then `yarn check:version`.
- [ ] 3.2 `copy.js`: the `es` `lede` and `step3` (D6), its header comment and the `es` table's comment. `apple-activation-page.spec.ts`: the Spanish page names inglés, francés and italiano, in that order, and its comment follows.
- [ ] 3.3 The comments and documents of D9 (`git grep -n "enable-lingua-italian-for-spanish-speakers\|change 30"`), keeping the Italian programme's hits.

## 4. Checks

- [ ] 4.1 In `apps/lingua-extension`: `yarn gen:pack`, `yarn gen:pack:real`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`. Each zipped package's size goes in the pull request.
- [ ] 4.2 Nothing else moves:
  - `cargo test -p lingua-pack` and `cargo test -p lingua-wasm`, every baseline without re-blessing;
  - `git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables ':!*README.md'` is empty;
  - `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 4.3 The Python tests and `gloss_coverage.py --check`. In `apps/site`: `node test/support/refresh-lingua-fixtures.mjs`, `test/astro/lingua-page.spec.ts` counting a row per listed pair (nine to eleven), `yarn test`, `yarn build`, `yarn check:routes`. The `lingua-apple-build` workflow green, `CFBundleLocalizations` unchanged by this change.
- [ ] 4.4 OpenSpec:
  - `openspec validate enable-lingua-italian-for-spanish-speakers --strict`;
  - the archive order, `refine-lingua-it-es-glosses` named;
  - the MODIFIED blocks re-copied as D13 says: from change 50 if implemented, else from change 45 if implemented, else from change 25 with `enable-lingua-italian-speakers` dropped from `archiveAfter` (or from `openspec/specs/` if archived), only D12's clauses differing, in the branch's wording of the list (D13);
  - the archive simulated on a scratch copy of `openspec/`.

## 5. Dogfood and release (owner)

- [ ] 5.1 [manual] The pass of design D10 on the five targets, in the Spanish interface, with the sample of pivot glosses.
- [ ] 5.2 [manual] The owner reads the drafts (D6, M9) and answers the Open Questions, Open Question 2 at change 29's figures.
- [ ] 5.3 [manual] The owner's go-ahead before merge. Released on its own, TestFlight first, with change 31's listings, the site deployed first; each submission the owner's (D11).
- [ ] 5.4 Change 30 marked done in `docs/lingua/italian-programme.md`.
