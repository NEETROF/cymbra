# Tasks

## 1. The reducer (scripts/lingua-data)

- [ ] 1.1 `reduce-it-fr.py`: `ENGLISH_BANDS` equal to fr-en's, `no_level` with D2's rules (unknown or name-only, a single character that is no word or only a spelling, only a spelling by French's prefixes, elsewhere as a guard; `words` read when given), `estimated_levels` (a left-out lemma giving its slot), the report line; `tables/it/level.tsv` written; `levels_estimated: true` in it-fr's manifest and the NOTICE line (D1–D3).
- [ ] 1.2 `test_reduce_it_fr.py`: each rule on fixture entries (`km`, `milano`, `k`, `r`, `e`, `the`, `perchè`, `del` levelled, a lemma whose form reads as another left out); the bands' sizes; a left-out lemma giving its slot; `words` narrowing the levels when given; the constant equal to `reduce-fr-en.py`'s; `python3 -m unittest discover -s scripts/lingua-data -p "test_reduce_it_fr.py"` passes.

## 2. The tables

- [ ] 2.1 [owner-dispatched] `lingua-pack-update` in mode `reduce` for it-fr from its pinned sources: `tables/it/level.tsv` committed; `forms.tsv`, `freq.tsv`, `grammar.tsv`, `lexical.tsv`, `tags.tsv` and `studied.json` byte-identical; `tables/it-fr/{manifest.json, NOTICE, pin.json (base and layer), README.md}` recorded again; the base's growth recorded; the report line and the measurement of the proposal re-run on change 9's ranks (against the English level of each translation; on UD Italian-PUD, the share of words each declared level presumes known) recorded in the pull request and the README.
- [ ] 2.2 `scripts/lingua-data/SOURCES.md`: Italian's levels are estimated, no list read.

## 3. Checks

- [ ] 3.1 `crates/lingua-pack/tests/committed_tables.rs`: Italian's 8,302 levels, English's sizes, each a ranked lemma of `tables/it/freq.tsv`, none on an acronym or spelling row; the Italian folder test widened by `level.tsv`; it-fr's pack gives each its level and says its levels are estimated; the six other packs build to their pins.
- [ ] 3.2 `crates/lingua-wasm/tests/languages.rs`: every Italian pack says its levels are estimated — `levelsEstimated` for `it`, and `levelLadder` rows carrying English's typical vocabularies, `typicalFrom` English and the six sizes —, as French's scenario test does.

## 4. Gates

- [ ] 4.1 Nothing else moves (D7): no table, pin or golden of the six pairs touched; the six baselines pass without re-blessing; the Italian golden does not move.
- [ ] 4.2 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`.
- [ ] 4.3 `openspec validate add-lingua-italian-levels --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-levels` names its prerequisites; `docs/lingua/italian-programme.md` row 14's status names this change's pull requests.

## 5. Owner

- [ ] 5.1 [manual] The owner reads the levels' figures on change 9's ranks beside French's and Spanish's, and approves the reduction's pull request; a licence request for an Italian CEFR list stays the owner's, outside this change.
