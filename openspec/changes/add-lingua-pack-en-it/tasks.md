# Tasks

## 0. Before the dispatch

- [ ] 0.1 [owner] The measurement re-run on the committed English ranks with D2's sources and change 35's rules (D6): `FLOORS["en-it"]` fixed at those figures less two points (about 89.4 / 80.7 / 68.3), the owner settling it before the first committed measurement; the figures written into the requirement *en-it is committed beside English's studied tables…* and a test holding `FLOORS` to them.

## 1. The sources

- [ ] 1.1 `pack_sources.py` (D7): `EDITIONS["en"]` gains `kaikki-en-traductions-it.jsonl`; `EDITIONS["it"]` gains `kaikki-it-Inglese.jsonl`, and `kaikki-it-traductions-en.jsonl` if change 22 has not registered it; `DUMPS["en-it"]`; `test_pack_sources.py`'s catalogue tests; `SOURCES.md`; every committed pair's rule files, digests and `pack_version` unchanged.

## 2. The reducer

- [ ] 2.1 `reduce-en-it.py` (D1–D5): `tables/en/` as committed; the sources in D2's order with the Italian edition's rules; en-es's table readers copied (D3); the expressions; writes `tables/en-it/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}`, the table share measured, both sides credited; its rule digest `reduce-en-it.py`, `reduce_common.py`, `reduce_edition_it.py`.
- [ ] 2.2 `test_reduce_en_it.py`: each source's place in the order; a word listed once; letters out; the translators' typography; a definition first; the expressions' keys; the Python tests pass.

## 3. The pipeline and the tables

- [ ] 3.1 `gloss_coverage.py` `FLOORS["en-it"]` and `test_gloss_coverage.py`; `lingua-extension-check.yml`'s floor and baselines steps; `lingua-pack-update.yml`'s pair option, `keep` text and bless steps; `test_reduce_loops.py` (D8).
- [ ] 3.2 [owner-dispatched] `lingua-pack-update` in mode `update` for en-it, with the owner's go-ahead for its sources release: the pull request commits `tables/en-it/` and `tables/en/` byte for byte; the coverage at or above the floor; `tables/en-it/README.md` (D9) and the programme's *Measured figures*; the pack built against its pin.

## 4. The baseline and the checks

- [ ] 4.1 `crates/lingua-wasm/tests/en_it_baseline.rs` and `baseline/en-it.golden` (en-es's 40 extra lemmas; the studied side en-fr's), blessed in task 3.2's pull request; `cross_native.rs` with en-it; both workflows run it (D8).
- [ ] 4.2 `crates/lingua-pack/tests/committed_tables.rs` and `testdata/en-it/`: en-it built to its pin, its credits, listed in no package while the list does not name it.
- [ ] 4.3 `apps/lingua-extension/test/row-gloss-tables.spec.ts` with en-it; `test/word-card-en-it.spec.ts` and `test/baseline/word-card-en-it.txt`, the card in Italian over en-it's golden, re-blessed with it (D8).

## 5. Gates

- [ ] 5.1 Nothing else moves: every other pair's tables, pins, packs and goldens unchanged.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; the Python tests; `gloss_coverage.py --check`; in `apps/lingua-extension`: `yarn gen:pack`, `yarn test`.
- [ ] 5.3 `openspec validate add-lingua-pack-en-it --strict`; the archive order; `docs/lingua/italian-programme.md` row 36's status.

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage by source, the sample and the golden, and approves the reduction's pull request.
