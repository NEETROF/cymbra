# Tasks

## 0. Before the dispatch

- [ ] 0.1 [owner] The measurement re-run on the committed French ranks with D2's sources and the real `reduce_edition_it.py` (D6). `FLOORS["fr-it"]` is fixed at those figures less two points (about 79.3 / 68.1 / 55.8), the owner settling it before the first committed measurement. The figures are written into the requirement *fr-it is committed beside French's studied tables…*, and a test holds `FLOORS` to them.

## 1. The sources

- [ ] 1.1 `pack_sources.py` (D7): `EDITIONS["it"]` gains `kaikki-it-Francese.jsonl`; `DUMPS["fr-it"]`. Then `test_pack_sources.py` and `SOURCES.md`. Every committed pair's rule files, digests and `pack_version` stay unchanged.

## 2. The reducer

- [ ] 2.1 `reduce-fr-it.py` (D1–D5):
  - reads `tables/fr/` as committed;
  - reads the sources in D2's order with the Italian edition's rules;
  - copies fr-es's add-pack readers and its determiner rule (D3), and adds the borrowed-gloss rule (D4);
  - reduces the expressions;
  - writes `tables/fr-it/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}`, with the table share measured and both sides credited;
  - its rule digest is `reduce-fr-it.py`, `reduce_common.py`, `reduce_edition_it.py`, `reduce_french_acronyms.py`.
- [ ] 2.2 `test_reduce_fr_it.py`: each source's place in the order; a definition first (`maison`); a word only a table glosses; a borrowed gloss giving way (`fond`), the direct table first, and kept when no table glosses; the apostrophe (`chef-d'œuvre` « capolavoro »); letters out (`h`, `x`); a word listed once (`mon`); the studied word as no definition (`igloo`) and a cognate kept; `ce` read as a determiner; the typography; the expressions' keys.
- [ ] 2.3 `test_reduce_editions.py`: the Italian edition's digest test names en-it and fr-it, and no other committed pair. The Python tests pass.

## 3. The pipeline and the tables

- [ ] 3.1 `gloss_coverage.py` `FLOORS["fr-it"]` and `test_gloss_coverage.py`; `lingua-extension-check.yml`'s floor and baselines steps; `lingua-pack-update.yml`'s pair option and bless steps; `test_reduce_loops.py` (D8).
- [ ] 3.2 [owner-dispatched] `lingua-pack-update` in mode `update` for fr-it, with the owner's go-ahead for its sources release. The resulting pull request:
  - commits `tables/fr-it/`, with `tables/fr/` unchanged byte for byte;
  - holds the coverage at or above the floor;
  - writes `tables/fr-it/README.md` (D9) with its sample of 100, and the programme's *Measured figures*;
  - builds the pack against its pin.

## 4. The baseline and the checks

- [ ] 4.1 `crates/lingua-wasm/tests/fr_it_baseline.rs` and `baseline/fr-it.golden`, with the studied side fr-en's, blessed in task 3.2's pull request; `cross_native.rs` with fr-it; both workflows run it (D8).
- [ ] 4.2 `crates/lingua-pack/tests/committed_tables.rs`: fr-it in every hard-coded list of committed pairs and French packs, among them `spec_scenario_the_committed_pairs` and `spec_scenario_the_acronym_in_both_french_packs` (`CA` glossed from fr-it's own expansions); fr-it built to its pin, its credits, its sources releases, listed in no package while the list does not name it. Also `scripts/lingua-data/testdata/fr-it/` (`pipeline_testdata.rs`), and `tables/fr-en/README.md` naming fr-it among the pairs reading `../fr/`.
- [ ] 4.3 `apps/lingua-extension/test/row-gloss-tables.spec.ts` with fr-it; `test/word-card-fr-it.spec.ts` and `test/baseline/word-card-fr-it.txt`, the card in Italian over fr-it's golden, re-blessed with it (D8).
- [x] 4.4 `docs/lingua/italian-programme.md`, in this proposal's pull request: the baselines line, fr-it's *Measured figures* row, rows 37 and 51 (D8, D10).

## 5. Gates

- [ ] 5.1 Nothing else moves: every other pair's tables, pins, packs and goldens are unchanged.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; the Python tests; `gloss_coverage.py --check`; in `apps/lingua-extension`: `yarn gen:pack`, `yarn test`.
- [ ] 5.3 `openspec validate add-lingua-pack-fr-it --strict`; the archive order; change 36's requirement and the main spec's form-of requirement re-copied as they then stand, only D4's and D10's clauses differing; `docs/lingua/italian-programme.md` row 47's status.

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage by source, the sample and the golden, answers the design's Open Questions, and approves the reduction's pull request.
