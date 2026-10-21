# Tasks

## 1. The apostrophe rule (crates/lingua-core)

- [ ] 1.1 `analysis/tokenize.rs`: Italian's words seam tries, for a word ending in a vowel and an apostrophe with no letter after it and no opening quote before it, the word's final vowel accented against the lexicon (D1), after change 6's listed words; the token is the accented form spanning the apostrophe, the capital kept; tests over a stand-in lexicon: `perche'`, `piu'`, `citta'`, `e'`, `E'`, `puo'`, `cosi'`, `caffe'`; `po'`, `da'` unchanged; `ciao'` → `ciao`; « ‘perche’ » → `perche`; `cargo test -p lingua-core` passes.
- [ ] 1.2 `analysis/mod.rs`: `ITALIAN_ANALYZER_VERSION = "1.2.0"`, its doc; `tests/italian_fixtures.rs` gains the rule's cases; `apps/lingua-extension/test/packs.spec.ts` reads `1.2.0`.

## 2. The tables (scripts/lingua-data)

- [ ] 2.1 `reduce-it-fr.py`: the dictionary's spellings (D2) read as 43d reads French's, with the frequency condition; `test_reduce_it_fr.py` covers a marked spelling, one with a sense of its own, one whose word is rarer.
- [ ] 2.2 The reviewed list (D3): a script lists every candidate with its frequencies and the condition that keeps or drops it; the kept rows join `reduce-it-fr.py` as one literal with their reasons, the dropped ones recorded by group in the design; tests cover a kept row, a word of its own, a cross-language exclusion.
- [ ] 2.3 [owner-dispatched] `lingua-pack-update` in mode `reduce` for it-fr from its pinned sources: `tables/it/forms.tsv` gains the rows, `tables/it-fr/{pin.json, manifest.json, README.md}` move; `it-ud.sh`'s figures recorded; `bash scripts/lingua-data/build.sh it-fr /tmp/it-fr.lingua` passes against the new pin.

## 3. The baseline

- [ ] 3.1 `crates/lingua-wasm/tests/italian_baseline.rs`: the `informale` page's words assert their accented readings; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; the pull request shows the diff is the `informale` page's and the version line's; verify the test passes without re-blessing afterwards.

## 4. Owner

- [ ] 4.1 [manual] The owner reads D3's list with its frequencies and its groups left out, and may move a row before the tables are reduced.

## 5. Gates

- [ ] 5.1 The six pairs do not move: the six baselines pass without re-blessing; `git diff --stat` shows no pin of the six touched.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 5.3 `openspec validate read-lingua-italian-unmarked-spellings --strict` passes; `python3 scripts/openspec_archive_order.py read-lingua-italian-unmarked-spellings` names its two prerequisites; `docs/lingua/italian-programme.md` row 10's status names this change's pull requests.
