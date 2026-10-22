# Tasks

## 0. Before the dispatch

- [ ] 0.1 [owner] The measurement re-run on the committed Spanish ranks with D2–D4's sources and rules and the real `reduce_edition_it.py` (D6). `FLOORS["es-it"]` is fixed at those figures less two points (about 83.7 / 73.6 / 61.9), the owner settling it before the first committed measurement. The figures are written into the requirement *es-it is committed beside Spanish's studied tables…*, and a test holds `FLOORS` to them.
- [ ] 0.2 [owner] The reviewed function-word list (D4), `scripts/lingua-data/es_it_function_words.tsv`, a rule file of es-it's reducer: each entry a Spanish lemma, Italian words one of es-it's sources pairs with it, and that source; proposed by the implementing agent or change 33's Italian reviewer agent, approved entry by entry by the owner.

## 1. The sources

- [ ] 1.1 `pack_sources.py` (D7): `DUMPS["es-it"]`, change 27a's files named under their editions; `test_pack_sources.py`; `SOURCES.md`. Every committed pair's rule files, digests and `pack_version` unchanged.

## 2. The reducer

- [ ] 2.1 `reduce-es-it.py` (D1–D5):
  - reads `tables/es/` as committed, with the Italian edition's rules and never the Spanish edition's;
  - reads the sources in D2's order, with D3's pivot filters and D4's rules;
  - reduces the expressions;
  - writes `tables/es-it/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}`, the source of every row kept for the README;
  - its rule digest is `reduce-es-it.py`, `reduce_common.py`, `reduce_edition_it.py`.
- [ ] 2.2 `test_reduce_es_it.py`: each source's place in the order; a pivot ordered by support; a name kept off; a letter, `o` kept and `y` « e, ed »; Wikidata's letters left out; the reviewed list read first (`durante`) and failing on an entry no source supports; a heraldic translation left out (`curva`); no key from a pointer wording; a cognate keeping its definition (`alcaloide`); a pivot gloss for `comprador`; the expressions' keys, with the count of pointer wordings left out.
- [ ] 2.3 `test_reduce_editions.py`: the Italian edition's digest test names en-it, fr-it and es-it, and no other committed pair. The Python tests pass.

## 3. The pipeline and the tables

- [ ] 3.1 `gloss_coverage.py` `FLOORS["es-it"]` and `test_gloss_coverage.py`; `lingua-extension-check.yml`'s floor and baselines steps; `lingua-pack-update.yml`'s pair option and bless steps; `test_reduce_loops.py` (D8).
- [ ] 3.2 [owner-dispatched] `lingua-pack-update` in mode `update` for es-it, with the owner's go-ahead for its sources release. The resulting pull request:
  - commits `tables/es-it/`, with `tables/es/` unchanged byte for byte;
  - holds the coverage at or above the floor;
  - writes `tables/es-it/README.md` (D9) with its sample of 100 marked by source, and the programme's *Measured figures*;
  - builds the pack against its pin.

## 4. The baseline and the checks

- [ ] 4.1 `crates/lingua-wasm/tests/es_it_baseline.rs` and `baseline/es-it.golden`, es-it alone, the studied side es-fr's, blessed in task 3.2's pull request; `cross_native.rs` with es-it; both workflows run it (D8).
- [ ] 4.2 `crates/lingua-pack/tests/committed_tables.rs`: es-it in every hard-coded list of committed and Spanish pairs; built to its pin, its credits, its sources releases, listed in no package while the list does not name it. Also `scripts/lingua-data/testdata/es-it/` (`pipeline_testdata.rs`), and `tables/es-fr/README.md` naming es-it among the pairs reading `../es/`.
- [ ] 4.3 `apps/lingua-extension/test/row-gloss-tables.spec.ts` with es-it; `test/word-card-es-it.spec.ts` and `test/baseline/word-card-es-it.txt`, the card in Italian over es-it's golden, re-blessed with it (D8).
- [x] 4.4 `docs/lingua/italian-programme.md`, in this proposal's pull request: I4 and I11 for es-it, the *Measured figures*, row 51 and the size.

## 5. Gates

- [ ] 5.1 Nothing else moves: every other pair's tables, pins, packs and goldens unchanged.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; the Python tests; `gloss_coverage.py --check`; in `apps/lingua-extension`: `yarn gen:pack`, `yarn test`.
- [ ] 5.3 `openspec validate add-lingua-pack-es-it --strict`; the archive order; the three MODIFIED blocks re-copied as their holders then stand, only D10's clauses differing; `docs/lingua/italian-programme.md` row 51's status.

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage by source, the sample and the golden, and approves the reduction's pull request.
