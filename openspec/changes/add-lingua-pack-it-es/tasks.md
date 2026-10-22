# Tasks

## 0. Before the dispatch

- [ ] 0.1 [owner] The study re-run on change 9's committed ranks with every source in D2's order and D4's pre-pass (D8): `FLOORS["it-es"]` fixed at those figures less two points, the owner settling it before the first committed measurement; the figures written into the requirement *it-es is committed beside Italian's studied tables…* and a test holding `FLOORS` to them.

## 1. The shared Spanish table rules

- [ ] 1.1 `reduce_spanish_tables.py`: fr-es's letters, listed-once, typography and `read_backwards` rules moved out of `reduce-fr-es.py`, the studied language a parameter; `reduce-fr-es.py` imports them; fr-es re-reduced from its pin with tables byte for byte and its pin's reducer files and digest re-recorded (D4).

## 2. The reducer

- [ ] 2.1 `reduce_italian_es.py` (D4): names and acronyms, the thin-pair clause, pointers and colon meanings for headwords with no meaning, sense links and superscripts, capitalised openings, the reviewed function-word list; grave accents, « ° » and Italian-as-Spanish words out of the tables.
- [ ] 2.2 `reduce-it-es.py` (D1–D3, D6, D7): `tables/it/` as committed; the sources in D2's order; the pivot's groups, filters and order (D3); Wikidata's links then items; the Spanish section read backwards; the expressions; the acronym rows; writes `tables/it-es/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}` with every source credited (D9).
- [ ] 2.3 `test_reduce_it_es.py`: each source's place in the order; a pivot word by its support; a name, a capital and a « à trier » table left out; `aver` borrowing `avere`'s gloss, `sono` not borrowing « Sonido »; `tu` « Tú »; `secondo` without « Apellido »; a sense link and a superscript out; a grave-accented word out; `USA`'s row; the Python tests pass.

## 3. The pipeline and the tables

- [ ] 3.1 `gloss_coverage.py` `FLOORS["it-es"]`; `lingua-extension-check.yml`'s floor step runs `--pair it-es`; `lingua-pack-update.yml` offers it-es, reduced after it-fr; `test_reduce_editions.py` names it-es among the Spanish edition's readers (D5); `SOURCES.md`.
- [ ] 3.2 [owner-dispatched] `lingua-pack-update` in mode `update` for it-es, with the owner's go-ahead for its sources release: the workflow's pull request commits `tables/it-es/`, `tables/it/` byte for byte; the coverage at or above the floor; `tables/it-es/README.md` — coverage by source, the sample marked by source — and the programme's *Measured figures* (D9); the pack built against its pin.

## 4. The baseline and the checks

- [ ] 4.1 `crates/lingua-wasm/tests/it_es_baseline.rs` and `baseline/it-es.golden` over `pages-it.txt`, blessed in task 3.2's pull request; a test that the golden is it-fr's on the studied side; `cross_native.rs` with it-es; both workflows run it (D11).
- [ ] 4.2 `crates/lingua-pack/src/licence.rs`: `Cc0`; `tests/committed_tables.rs`: it-es built to its pin, its credits, listed in no package while the list does not name it.

## 5. Gates

- [ ] 5.1 Nothing else moves: every other pair's tables and goldens unchanged, fr-es's tables byte for byte (its pin's digest alone moves); `reduce_edition_es.py` and `reduce_common.py` unchanged.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; the Python tests; `gloss_coverage.py --check`.
- [ ] 5.3 `openspec validate add-lingua-pack-it-es --strict`; the archive order; the MODIFIED blocks re-copied as their holders then stand; `docs/lingua/italian-programme.md` row 27's status.

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage by source, the sample with its pivot rows, and the golden, and approves the reduction's pull request.
