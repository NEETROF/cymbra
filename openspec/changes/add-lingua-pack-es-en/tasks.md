# Tasks

## 1. Sources and the pipeline (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py`: `KAIKKI["es-en"]` (es-fr's extract), `DUMPS["es-en"]` (`kaikki-es-traductions-en.jsonl`, `("translations", "es", "en")`); `fetch_live` derives both pairs' files from one pass; `record-build` writes the `studied` record for a non-reference pair (D2, D3). `test_pack_sources.py`: the record, the shared asset.
- [ ] 1.2 `build.sh`: `max_lemmas` 60,000 for a Spanish pair; the studied folder's sha256s handed to `record-build` (D3).
- [ ] 1.3 `.github/workflows/lingua-pack-update.yml`: `es-en` in the dispatch options and the monthly matrix; `pack_report.py` names the pair (D2).

## 2. The reducer and the tables

- [ ] 2.1 `reduce-es-en.py`: es-fr's studied side by import, `EDITION = english.EN`, entries from the English Wiktionary's Spanish section, direct fallback from the Spanish Wiktionary's English translations, NOTICE and manifest crediting both sides (D1, D4). `test_reduce_editions.py`: *A gloss from a translation table*; a test that es-fr's digest is unchanged by this file's existence.
- [ ] 2.2 The two measurements: the reducer run both ways on the top 10,000, two samples of 100 in the pull request; the owner's values committed in `reduce_edition_en.py` (D5; *The owner's settings*).
- [ ] 2.3 `build.sh --reduce es-en` from the pinned sources: `tables/es-en/` committed, `pin.json` with its `studied` record; es-fr's and en-fr's tables, pins and packs byte for byte unchanged (*The first reader pair*).

## 3. Checks and fixtures

- [ ] 3.1 `crates/lingua-pack/src/tables.rs`: the `studied` record checked against `tables/<studied>/` (*The studied side moves*); `tests/committed_tables.rs`: es-en's exact file set, *The credits*; `crates/lingua-wasm/tests/cross_native.rs` on the real es-en pack.
- [ ] 3.2 `scripts/lingua-data/testdata/es-en/`; `gloss_coverage.py --pair` (*Coverage*, not published); the reduce job lists es-en after es-fr (*The reduce job*).

## 4. Gates and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-pack -p lingua-wasm`; the Python tests; `yarn gen:pack` and `yarn gen:pack:real` in `apps/lingua-extension` (two packs, unchanged); actionlint; the `reduce` job green on the pull request; the pack under 5 MiB.
- [ ] 4.2 `SOURCES.md`, `tables/es-en/README.md`, `tables/es-fr/README.md` (its reader named), `REVIEWERS.md` wording; `openspec validate add-lingua-pack-es-en --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-pack-es-en` exits 0; change 21 is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Owner

- [ ] 5.1 [manual] The owner reviews the two samples and a sample of 100 glosses from the top 10,000 (M9), and picks the two settings (M20).
