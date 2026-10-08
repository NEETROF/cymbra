# Tasks

## 1. Sources and the pipeline (scripts/lingua-data)

- [x] 1.1 `pack_sources.py`: `KAIKKI["es-en"]` (es-fr's address), `DUMPS["es-en"]` (`kaikki-es-traductions-en.jsonl`, `("translations", "es", "en")`); each pair pins its own extract, fetched live when it is updated, under its own release; the update publishes a pair's own assets only (each record's release); the asset cache, keyed by sha256, written whole or not at all, a bad entry deleted and named; `fetch_live` carries a reader's `studied` record over; `record-build` writes the `studied` record; `check-reducer` compares it; `pack_report` names the table; `version` folds the record into a reader pair's `pack_version` (D2, D3). `test_pack_sources.py`: *Each pair's own extract*, *The studied side moves*, *A rules-only change of the reference*, *An updated dictionary for a reader pair*.
- [x] 1.2 `build.sh`: `max_lemmas` 60,000 for a Spanish pair; the version from `pack_sources.py version` in both modes; the asset cache `work/cache/<sha256>`; `lingua-pack-update.yml`: `es-en` in the dispatch options and the monthly matrix, one run at a time per studied language; the publish step tags `release_tag(pair, snapshot)`, uploads only the assets of records whose `release` is that tag, and fails when it cannot list them (`test_reduce_loops.py`) (D2); the `check` job's loop checks the rules before it builds; the reduce job's 45-minute timeout measured with the extract's fetch. *Measured before the corrections: the update run with the live extract's fetch took 6 min, the `reduce` job 3 min 38 s for the three pairs.*
- [x] 1.3 `gloss_coverage.py --pair`, `FLOORS` (es-en's) and `--floor`; the reduce job runs it for es-en (D6; *Coverage*).

## 2. The reducer and the tables

- [x] 2.1 `reduce-es-en.py`: the native side alone from the committed `tables/es/`, `EDITION = english.EN`, entries from the English Wiktionary's Spanish section, direct fallback from the Spanish Wiktionary's English translations (its letters left out), NOTICE and manifest crediting both sides and the French Wiktionary (D1, D4); `reduce_edition_en.py`: `merge_same_pos_etymologies` behind an `EN` setting (D5), the letter rule and `without_letter_headwords` (D1). `test_reduce_editions.py`: *A gloss from a translation table*, the pre-passes and the letters on recorded entries, `native_fields`' cut, es-en's copy of Spanish against es-fr's; the digest test shows es-fr's and en-fr's rule sets unchanged (*A setting of the English edition*).
- [ ] 2.2 The two settings: the reducer run both ways on the top 10,000, two samples of 100 in the pull request, the owner's values committed before the pin is recorded (D5). *Half done: run both ways and sampled (1,057 and 247 of the top 10,000 glosses change, after the letter corrections), committed at the defaults (both off) and pinned; the owner's values are pending — a value chosen re-pins es-en alone.*
- [x] 2.3 [manual, on the branch] `lingua-pack-update` dispatched for es-en (publishes its extract and its derived file under `lingua-pack-sources-es-en-2026.10.08`); then `build.sh --reduce es-en` from the pinned sources: `tables/es-en/` committed with its pin; es-fr's and en-fr's tables, pins and packs byte for byte unchanged (*The first reader pair*).

## 3. Checks and fixtures

- [x] 3.1 `crates/lingua-pack/tests/committed_tables.rs`: es-en's exact file set, *The credits*; `crates/lingua-wasm/tests/cross_native.rs` on the real es-en pack; `scripts/lingua-data/testdata/es-en/`; the reduce job lists es-en after es-fr (*The reduce job*).

## 4. Gates and docs

- [ ] 4.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-pack -p lingua-wasm`; the Python tests; `yarn gen:pack` and `yarn gen:pack:real` in `apps/lingua-extension` (two packs, unchanged); actionlint; the `reduce` job green on the pull request; the pack under 5 MiB. *Before the corrections, the `reduce` and `check` jobs were green on a dispatch of the branch (run 37733849068). After them, the local gates pass and es-en's pack is 2,567,750 B; actionlint not run (not installed); the `reduce` job is to run again on the pull request.*
- [x] 4.2 `SOURCES.md`, `tables/es-en/README.md`, `tables/es-fr/README.md` (its reader named), `REVIEWERS.md`; `openspec validate add-lingua-pack-es-en --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-pack-es-en` exits 0; change 21's row in `docs/lingua/language-matrix-programme.md` says where it stands.

## 5. Owner

- [ ] 5.1 [manual] The owner reviews the two samples and a sample of 100 glosses from the top 10,000 (risk 4), and picks the two settings (M20). *Pending: the owner.*
