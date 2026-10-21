# Tasks

## 1. The rule (crates/lingua-core)

- [ ] 1.1 `analysis/pipeline.rs`: `alone_in_capitals` answers for Italian as for French; `analysis/italian.rs`: the acronym step before the cascade, as `french::acronym` (D1); tests: « I paesi della NATO hanno votato », « LA NATO HA VOTATO », « sono nato », « Nato a Roma », « AI MIEI AMICI », « L'ONU », « DELL'ANSA »; `cargo test -p lingua-core` passes.
- [ ] 1.2 `analysis/mod.rs`: `ITALIAN_ANALYZER_VERSION = "1.3.0"`, its doc; `tests/italian_fixtures.rs` gains the rule's cases; `apps/lingua-extension/test/packs.spec.ts` reads `1.3.0`.

## 2. The table and the builder (scripts/lingua-data, crates/lingua-pack)

- [ ] 2.1 `scripts/lingua-data/reduce_italian_acronyms.py`: `ACRONYMS` with its rows and reasons (D2), its test holding each expansion or sense to the entry, the capitals, the lowercase form; `reduce-it-fr.py` writes the rows into `forms.tsv` and `lexical.tsv` and glosses them (D3); `test_reduce_it_fr.py` covers a row, its gloss from the entry, from the expansions, and none; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes.
- [ ] 2.2 `crates/lingua-pack/src/lib.rs`: a form holding a capital accepted in a pack studying French or Italian (D2), the refusal's message naming both; `tests/committed_tables.rs`: Italian's rows present as forms in capitals mapping to themselves and as dictionary words, with no rank and no level; `tables/it/lexical.tsv` held to it-fr's glossed lemmas less the names (D5); `cargo test -p lingua-pack` passes.
- [ ] 2.3 [owner-dispatched] `lingua-pack-update` in mode `reduce` for it-fr: `tables/it/{forms.tsv, lexical.tsv}` and `tables/it-fr/{pin.json, manifest.json, README.md}` move; `it-ud.sh`'s figures recorded; the names count (D5) recorded in the pull request.

## 3. The baseline

- [ ] 3.1 `scripts/lingua-data/testdata/it-fr/`: `manifest.json` at `1.3.0`, `forms.tsv` with `NATO` → `NATO` and `nato` → `nascere`; `tests/support/italian.rs`: a phrase probe « I paesi della NATO » and a grammar probe (`NATO`, `NATO`); re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; the pull request names what moved; verify the test passes without re-blessing afterwards.

## 4. Gates

- [ ] 4.1 The six pairs do not move: the six baselines pass without re-blessing; `git diff --stat` shows no pin of the six touched.
- [ ] 4.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 4.3 `openspec validate keep-lingua-italian-acronyms-and-names --strict` passes; `python3 scripts/openspec_archive_order.py keep-lingua-italian-acronyms-and-names` names its prerequisites (changes 16 and 17 added to `archiveAfter` once proposed); `docs/lingua/italian-programme.md` row 11's status names this change's pull requests.

## 5. Owner

- [ ] 5.1 [manual] The owner reads the table's rows with their measurements (D2) and settles which acronyms are rows (`NATO`, `USA`, `ANSA`, and any the measurement adds), and reads the names count of D5.
