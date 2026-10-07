# Tasks

## 1. Sources and the pipeline (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py`: `DUMPS["en-es"]` (`kaikki-es-English.jsonl` from es-fr's `kaikki-es` dump; `kaikki-en-traductions-es.jsonl` from the new `kaikki-en` source), the dump source recorded in the pin, derived at `fetch_live` only, published as assets, fetched as assets (D2). `test_pack_sources.py`: *An update*, *A pinned reduction*, *The extract moves* on fixtures.
- [ ] 1.2 `.github/workflows/lingua-pack-update.yml`: `en-es` in the dispatch options and the monthly matrix; `pack_report.py` names the pair; `gloss_coverage.py --pair --floor` in the reduce job (D3).

## 2. The reducer and the tables

- [ ] 2.1 `reduce-en-es.py`: en-fr's studied side by import, `EDITION = spanish.ES`, entries from the Spanish Wiktionary's English section, direct fallback from the English Wiktionary's Spanish translations, NOTICE and manifest crediting both sides (D1). `test_reduce_editions.py`: *A definition first*, *A translation-table gloss*; en-fr's digest unchanged.
- [ ] 2.2 `build.sh --reduce en-es` from the pinned sources: `tables/en-es/` committed, `pin.json` with its `studied` record and the dump source; en-fr's, es-fr's and es-en's tables, pins and packs byte for byte unchanged.
- [ ] 2.3 The measurements: coverage against the floor, the translation-table share, the sample of 100 glosses for the owner (D3, D4).

## 3. Checks and fixtures

- [ ] 3.1 `tests/committed_tables.rs`: en-es's exact file set and credits; `testdata/en-es/`; the reduce job lists en-es after en-fr.

## 4. Gates and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-pack -p lingua-wasm`; the Python tests; `yarn gen:pack` and `yarn gen:pack:real` unchanged; actionlint; the `reduce` job green on the pull request; the pack under 5 MiB.
- [ ] 4.2 `SOURCES.md`, `tables/en-es/README.md`, `tables/en-fr/README.md` (its reader named); `openspec validate add-lingua-pack-en-es --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-pack-en-es` exits 0 or waits for change 21; change 22 is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Owner

- [ ] 5.1 [manual] The owner reviews the sample of 100 glosses (M9) and the translation-table share.
