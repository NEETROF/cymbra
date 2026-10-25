# Tasks

## 1. The rule (crates/lingua-core)

- [x] 1.1 `analysis/pipeline.rs`: the gates of `resolve_lemmas` and `alone_in_capitals` open to Italian, `french::acronym` serving as is (D1); tests: « I paesi della NATO hanno votato », « LA NATO HA VOTATO », « sono nato », « Nato a Roma », « negli USA », « NEGLI USA », « degli USA », « DEGLI USA », « AI MIEI AMICI »; `cargo test -p lingua-core` passes.
- [x] 1.2 `analysis/mod.rs`: `ITALIAN_ANALYZER_VERSION = "1.3.0"`, its doc; `tests/italian_fixtures.rs` gains the rule's cases; `apps/lingua-extension/test/packs.spec.ts` matches Italian's version by its pattern (it already did: unchanged). Italian's `1.3.0` is English's and French's string too: `language.rs`'s and `pack.rs`'s version tests hold each language to its own constant (design, *Departures*).

## 2. The table, the builder and the checks (scripts/lingua-data, crates/lingua-pack)

- [x] 2.1 `scripts/lingua-data/reduce_italian_acronyms.py`: `ACRONYMS` with `NATO` and `USA` and their reasons (D2); `check_rows` accepting a row naming no expansion and no synonym when its entry exists, refusing a row with no entry; its test holds each expansion or synonym to the entry, the capitals, the lowercase a form of another word; `reduce-it-fr.py` writes the rows into `forms.tsv`; `test_reduce_it_fr.py` covers a row; the gloss rule (D3) written as a function `acronym_gloss(row, glosses)` tested on `USA` (synonyms), on an expansion-bearing row, and on `NATO` (none), for change 16 to call; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes.
- [x] 2.2 `crates/lingua-pack/src/lib.rs`: a form or a lemma holding a capital accepted in a pack studying French or Italian (D2), the refusal's message naming both; `src/tables.rs` `check_studied`: the reference's acronym rows exempted — `glossed ∪ rows` and `(glossed − names) ∪ rows` accepted (D5); `tests/committed_tables.rs`: Italian's rows present as forms in capitals mapping to themselves, with no rank and no level, and the ranked checks exempting them; `cargo test -p lingua-pack` passes.
- [x] 2.3 [owner-dispatched] `lingua-pack-update` in mode `reduce` for it-fr: `tables/it/forms.tsv`, `tables/it/lexical.tsv` (the rows alone, D6) and `tables/it-fr/{pin.json, manifest.json, README.md}` move; the glosses untouched (change 16's). Done: `lingua-pack-update` run 40013770231 on `641a25f12`, its commit `f18aa3230` fast-forwarded onto the branch (the pin byte for byte the local reduction's, the pack's sha256 `846ac003…`); `measure/it-ud.sh` on these tables: UD Italian-PUD 99.26 / 91.31 / 99.90, MarkIT's test section 99.43 / 94.65 / 98.87.
- [ ] 2.4 Handed to change 16 (D6): `lexical.tsv` rewritten as `dictionary_words ∪ ACRONYMS` and the rows glossed by `acronym_gloss`; the names count of D5 recorded in its pull request; this change archives before 16 and 17, not after them: the `archiveAfter` entry naming 16 was dropped on 2026-10-25, as it closed a cycle with 16 and 14 (design, *The archive order*).

## 3. The baseline

- [x] 3.1 `scripts/lingua-data/testdata/it-fr/`: `manifest.json` at `1.3.0`, `forms.tsv` with `NATO` → `NATO` and `nato` → `nascere`; `tests/support/italian.rs`: a phrase probe « I paesi della NATO » and a grammar probe (`NATO`, `NATO`); re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; the pull request names what moved (the version line and the two probes, D7); verify the test passes without re-blessing afterwards.

## 4. Gates

- [x] 4.1 The six pairs do not move: the six baselines pass without re-blessing; `git diff --stat` shows no pin of the six touched.
- [x] 4.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants`.
- [x] 4.3 `openspec validate keep-lingua-italian-acronyms-and-names --strict` passes; `python3 scripts/openspec_archive_order.py keep-lingua-italian-acronyms-and-names` names its prerequisites (changes 8 and 10 bind once on `main`, 16 and 17 added to `archiveAfter` once proposed); `docs/lingua/italian-programme.md` row 11's status names this change's pull requests. Validation passes; the archive order exits 10, naming the open prerequisites; row 11 names #1045, analyser 1.3.0 and the reduction run 40013770231.

## 5. Owner

- [x] 5.1 [manual] The owner reads the table's rows with their measurements (D2) and settles which acronyms are rows — `NATO`, `USA`, and `TARI`, `AGI`, `FAI` or any the measurement adds — and that the two-letter codes stay out. Settled by the owner (Guillaume) on 2026-10-23: the rows are `NATO` and `USA` only; `FAI` and `AGI` are too rare and stay out; `TARI` is already read as the tax.
