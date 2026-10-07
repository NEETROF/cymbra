# Tasks

## 1. Sources and the pipeline (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py`: `DUMPS["en-es"]` (`kaikki-es-English.jsonl` from es-fr's `kaikki-es` dump; `kaikki-en-traductions-es.jsonl` from the new `kaikki-en` source), a pair whose sources are dumps alone (no `KAIKKI` entry), `derive` reading a plain or a gzipped dump, `kaikki-es-traductions-en.jsonl` derived in the same pass as `kaikki-es-English.jsonl` (D1, D2). `test_pack_sources.py`: *An extract served plain*, the dumps-only pair.
- [ ] 1.2 `.github/workflows/lingua-pack-update.yml`: `en-es` in the dispatch options and the monthly matrix; the publish step's tag from `release_tag(pair, snapshot)` and its notes without an extract when `sources.kaikki` is absent; `pack_sources.py assets` lists the dump records' files alone for such a pair; `pack_report.py` names the pair and prints the share (D4); the reduce job and the `check` job's tests run `gloss_coverage.py --pair en-es --floor` (D3).

## 2. The reducer and the tables

- [ ] 2.1 `reduce-en-es.py`: the native side alone from the committed `tables/en/`, `EDITION = spanish.ES`, entries, direct and inverted tables, the share computed from the lemma sets of each step (`reduce_common.py` not edited), NOTICE and manifest crediting both sides (D1, D4; the studied side's credits as en-fr's). `test_reduce_editions.py`: *A definition first*, *A direct-table gloss*, *An inverted-table gloss*; en-fr's digest unchanged.
- [ ] 2.2 [manual, on the branch] `lingua-pack-update` dispatched for en-es (publishes its derived files; records the extract's size); then `build.sh --reduce en-es` from the pinned sources: `tables/en-es/` committed with its pin; en-fr's, es-fr's and es-en's tables, pins and packs byte for byte unchanged.
- [ ] 2.3 The measurements: coverage against the proposed floor, the share, the sample of 100 glosses (D3–D5).

## 3. Checks and fixtures

- [ ] 3.1 `tests/committed_tables.rs`: en-es's exact file set and credits; `testdata/en-es/`; the reduce job lists en-es after en-fr.

## 4. Gates and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-pack -p lingua-wasm`; the Python tests; `yarn gen:pack` and `yarn gen:pack:real` unchanged; actionlint; the `reduce` job green on the pull request; the pack under 5 MiB.
- [ ] 4.2 `SOURCES.md` (the extract and change 38), `tables/en-es/README.md` (the share), `tables/en-fr/README.md` (its reader named); `openspec validate add-lingua-pack-en-es --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-pack-en-es` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived) or waits for change 21; change 22 is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Owner

- [ ] 5.1 [manual] The owner sets the floor on this pull request (risk 5; M6's rule; 91.4 / 83.2 / 69.9 % proposed); the value is written into the requirement *English is glossed in Spanish…* and into the reduce job's `gloss_coverage.py --pair en-es --floor` before merge; the owner reviews the sample of 100 glosses and the share.
