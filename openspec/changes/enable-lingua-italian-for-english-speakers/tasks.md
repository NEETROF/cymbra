# Tasks

Implemented once the prerequisites of proposal.md (Impact) are on `main` — changes 22, 23, 24 and 26, and change 20 released.

## 1. The list

- [ ] 1.1 `packs.json` ends with `"it-fr", "it-en"`; `check_variants`'s `SHIPPED_PAIRS` the same, its comment naming this change; `scripts/lingua-data/testdata/it-en/`, the it-fr fixture glossed in English — its Italian files byte for byte `testdata/it-fr/`'s; `crates/lingua-pack/tests/pipeline_testdata.rs`: `the_it_en_fixture_is_the_italian_fixture_glossed_in_english` and it-en's row in the fixtures' pins (D1).
- [ ] 1.2 `gloss_coverage.py --write` with it-en's figures; `test_gloss_coverage.py`: it-en « published once listed », `ListedFloorsTest` naming it (D2, D7).
- [ ] 1.3 The tests that read the list (D9), each keeping the earlier lists as explicit arguments; `engine.spec.ts`; `committed_tables.rs` asserts `packs.json` lists it-en.

## 2. The extension

- [ ] 2.1 Tests (D3): `studied-languages-view.spec.ts` — Spanish, French and Italian offered to an English speaker, Italian to no Spanish speaker; `profile.spec.ts` — Spanish then Italian, English native; `language-labels.spec.ts`, `settings-language.spec.ts`, `onboarding-level-row.spec.ts`, `popup.spec.ts`, `stats-view.spec.ts` — Italian in the English interface ("Estimated Italian level"); `wordpopup.spec.ts` — an Italian card in English, "past historic (passato remoto)".
- [ ] 2.2 `translate-relay.spec.ts`: it-en marked if change 24 listed it; `model-controller.spec.ts` and `translation-setting.spec.ts`: the two English cases of D5; `model-residency.spec.ts` and `native-language.spec.ts` (D8, D9).

## 3. The description, the activation page and the documents

- [ ] 3.1 `_locales/en/messages.json` `extensionDescription` (D6's draft or the owner's words), its translator's `description` naming Italian (it-en) beside Spanish (es-en) and French (fr-en); `yarn check:version`.
- [ ] 3.2 `copy.js` `en` `lede` and `step3` (D6), its header comment and the `en` table's comment; `apple-activation-page.spec.ts`: the English page names Spanish, French and Italian, in that order, and its comment follows.
- [ ] 3.3 The comments and documents of D9 (`git grep -n "enable-lingua-italian-for-english-speakers\|change 25"`, keeping the Italian programme's hits).

## 4. Checks

- [ ] 4.1 In `apps/lingua-extension`: `yarn gen:pack`, `yarn gen:pack:real`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; each zipped package's size in the pull request.
- [ ] 4.2 Nothing else moves: `cargo test -p lingua-pack`, `cargo test -p lingua-wasm` — every baseline without re-blessing; `git diff --stat origin/main -- crates/lingua-core crates/lingua-wasm scripts/lingua-data/tables ':!*README.md'` empty; `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] 4.3 The Python tests and `gloss_coverage.py --check`; in `apps/site`: `node test/support/refresh-lingua-fixtures.mjs`, `test/astro/lingua-page.spec.ts` counting eight rows, `yarn test`, `yarn build`, `yarn check:routes`; the `lingua-apple-build` workflow green, `CFBundleLocalizations` still `[fr, en, es]`.
- [ ] 4.4 `openspec validate enable-lingua-italian-for-english-speakers --strict`; the archive order, `refine-lingua-it-en-glosses` added to `archiveAfter` once proposed; the MODIFIED blocks re-copied from change 20 as it then stands, only D12's clauses differing; the archive simulated on a scratch copy of `openspec/`.

## 5. Dogfood and release (owner)

- [ ] 5.1 [manual] The pass of design D10 on the five targets, in the English interface.
- [ ] 5.2 [manual] The owner reads the drafts (D6, M9) and answers the Open Questions.
- [ ] 5.3 [manual] The owner's go-ahead before merge; released on its own, TestFlight first, with change 26's listings, the site deployed first; each submission the owner's (D11).
- [ ] 5.4 Change 25 marked done in `docs/lingua/italian-programme.md`.
