# Tasks

## 1. Sources and the pipeline (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py`: `KAIKKI["es-en"]` (es-fr's extract), `DUMPS["es-en"]` (`kaikki-es-traductions-en.jsonl`, `("translations", "es", "en")`); `fetch_live` of a reader pair reuses the extract its reference fetched in the same run (the asset cache) and records the reference's release, its own otherwise; the update publishes a pair's own assets only (each record's release); `record-build` writes the `studied` record; `check-reducer` compares it; `pack_report` names the table (D2, D3). `test_pack_sources.py`: *A shared extract*, *The studied side moves*, *A rules-only change of the reference*.
- [ ] 1.2 `build.sh`: `max_lemmas` 60,000 for a Spanish pair; the reduce job's asset cache `work/cache/<sha256>` (`.github/workflows/lingua-extension-check.yml`); `lingua-pack-update.yml`: `es-en` in the dispatch options and the monthly matrix; the publish step tags `release_tag(pair, snapshot)` — never `sources.kaikki.release`, which for a reader pair may be its reference's — and uploads only the assets of records whose `release` is that tag (`assets --pin` filtered by release) (D2); the reduce job's 45-minute timeout measured with the extract's fetch.
- [ ] 1.3 `gloss_coverage.py --pair` and `--floor`; the reduce job runs it for es-en (D6; *Coverage*).

## 2. The reducer and the tables

- [ ] 2.1 `reduce-es-en.py`: the native side alone from the committed `tables/es/`, `EDITION = english.EN`, entries from the English Wiktionary's Spanish section, direct fallback from the Spanish Wiktionary's English translations, NOTICE and manifest crediting both sides (D1, D4); `reduce_edition_en.py`: `merge_same_pos_etymologies` behind an `EN` setting (D5). `test_reduce_editions.py`: *A gloss from a translation table*, the pre-pass on recorded entries; the digest test shows es-fr's and en-fr's rule sets unchanged (*A setting of the English edition*).
- [ ] 2.2 The two settings: the reducer run both ways on the top 10,000, two samples of 100 in the pull request, the owner's values committed before the pin is recorded (D5).
- [ ] 2.3 [manual, on the branch] `lingua-pack-update` dispatched for es-en (publishes its derived file); then `build.sh --reduce es-en` from the pinned sources: `tables/es-en/` committed with its pin; es-fr's and en-fr's tables, pins and packs byte for byte unchanged (*The first reader pair*).

## 3. Checks and fixtures

- [ ] 3.1 `crates/lingua-pack/tests/committed_tables.rs`: es-en's exact file set, *The credits*; `crates/lingua-wasm/tests/cross_native.rs` on the real es-en pack; `scripts/lingua-data/testdata/es-en/`; the reduce job lists es-en after es-fr (*The reduce job*).

## 4. Gates and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-pack -p lingua-wasm`; the Python tests; `yarn gen:pack` and `yarn gen:pack:real` in `apps/lingua-extension` (two packs, unchanged); actionlint; the `reduce` job green on the pull request; the pack under 5 MiB.
- [ ] 4.2 `SOURCES.md`, `tables/es-en/README.md`, `tables/es-fr/README.md` (its reader named), `REVIEWERS.md`; `openspec validate add-lingua-pack-es-en --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-pack-es-en` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 21 is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Owner

- [ ] 5.1 [manual] The owner reviews the two samples and a sample of 100 glosses from the top 10,000 (risk 4), and picks the two settings (M20).
