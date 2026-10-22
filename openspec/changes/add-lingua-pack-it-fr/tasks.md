# Tasks

## 0. Before the dispatch

- [ ] 0.1 [owner] The prototype re-run on change 9's committed ranks with the two fallbacks (D2, D7): the floor set to those figures less two points if higher than `(85.7, 79.8, 69.9)`, never lower; the owner settles it.

## 1. The sources (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py`: `EDITIONS["fr"]` gains `kaikki-fr-Italien.jsonl` and `kaikki-fr-traductions-it.jsonl`; `EDITIONS["it"]` (the Italian Wiktionary) with `kaikki-it-traductions-fr.jsonl`; `DUMPS["it-fr"]` names the three beside change 9's file; `FLOORS["it-fr"]` (D3, D7); `test_pack_sources.py` covers them; `SOURCES.md` names them with their licences.

## 2. The reducer

- [ ] 2.1 `reduce-it-fr.py` writes `tables/it-fr/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}` through `reduce_common.native_tables` with `reduce_edition_fr.FR`: definitions, translations read backwards, the Italian Wiktionary's translations (D1, D2); D4's rule; `lexical.tsv` as the dictionary words plus the acronym rows, the rows glossed by `acronym_gloss`, the levels narrowed by `words` (D5); the expressions with and without a space (D6); the report's coverage, sources and sample (D8).
- [ ] 2.2 `test_reduce_it_fr.py`: each source's order and the three-word cap; `del` « Du » read as a gloss and « Pluriel de giorno » kept a form; `roma` no dictionary word, `marco` one, `NATO`/`USA` rows; every levelled lemma a dictionary word; `d'accordo` an expression; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes.

## 3. The tables

- [ ] 3.1 [owner-dispatched] `lingua-pack-update` in mode `update` for it-fr, with the owner's go-ahead for the sources release: the workflow's pull request commits `tables/it-fr/` and `tables/it/{lexical.tsv, level.tsv}`, every other studied table byte for byte; the coverage at or above the floor; `tables/it-fr/README.md` and the programme's *Measured figures* record it (D8); `bash scripts/lingua-data/build.sh it-fr /tmp/it-fr.lingua` passes against the pin.

## 4. The baseline and the checks

- [ ] 4.1 `crates/lingua-wasm/tests/italian_baseline.rs` and `support/italian.rs`: `PackSource::Tables` for it-fr; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; the pull request lists what moved (D9); verify the test passes without re-blessing afterwards.
- [ ] 4.2 `crates/lingua-pack/tests/committed_tables.rs`: it-fr built to its pin, whole and in parts; `lexical.tsv` and the levels held as D10 says; the six other packs to their pins.

## 5. Gates

- [ ] 5.1 The six pairs do not move: their baselines pass without re-blessing; `git diff --stat` shows no pin of the six touched.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants` (no package change).
- [ ] 5.3 `openspec validate add-lingua-pack-it-fr --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-pack-it-fr` names its prerequisites, and changes 11 and 12 name it among theirs; `docs/lingua/italian-programme.md` row 16's status names this change's pull requests.

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage, the sample by source and the golden's diff, and approves the reduction's pull request.
