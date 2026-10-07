# Tasks

## 1. Layout and the split (scripts/lingua-data)

- [ ] 1.1 Move English's and Spanish's studied tables and `tags.tsv` into `tables/en/` and `tables/es/` (`git mv`). Add `studied.json` and the derived `lexical.tsv` (D1, D2).
- [ ] 1.2 `pack_sources.py split` and `pairs` (references first, `--after`). `build.sh` uses them, and `record-build` finds `tags.tsv` in the studied folder (D2, D3). `test_pack_sources.py`:
  - a reference reduction writes both folders, derives `lexical.tsv` and keeps `tags.tsv`;
  - a non-reference reduction (a fixture pair) never writes the studied folder;
  - `pairs` lists references first and skips studied folders;
  - a folder that is neither a pair nor a studied language fails.

## 2. The builder and the check (crates/lingua-pack)

- [ ] 2.1 `inputs_from_dirs` / `--studied`. A file on the wrong side is refused, naming it; a single folder still builds (D4). Tests:
  - the two-folder build equals the single-folder build;
  - en-fr and es-fr from the new layout match their pins;
  - the refusal names the file.
- [ ] 2.2 `check_committed_tables` and `committed_tables.rs`. Tests for each scenario of *A studied language's tables are kept once*, naming the pair, the file and the reference:
  - a studied folder holds exactly its six tables plus `studied.json`;
  - a pair folder holds no studied table;
  - `lexical.tsv` equals the reference's glossed lemmas;
  - a pair whose manifest studies another language fails.

## 3. Consumers

- [ ] 3.1 Workflows: `lingua-extension-check` (build/check loops over `tables/*-*/`, reduce job via `pairs`) and `lingua-pack-update` (`pair=all` via `pairs`; a reference update reduces its readers, D5).
- [ ] 3.2 `pack_report.py` reports the studied folder's changes and names every pair whose pack moves; `gloss_coverage.py` and `measure/` read both folders. Their tests are updated, and `gloss_coverage.py --check` passes unchanged.
- [ ] 3.3 `crates/lingua-wasm/tests/support`, `es_fr_grammar.rs`, `cross_native.rs` and the `lingua-pack` test helper read the studied folder. `english_baseline`, `spanish_baseline`, `cross_native` and `backup_format` pass without `LINGUA_BLESS`.
- [ ] 3.4 `apps/lingua-extension/tool/gen_pack.sh` builds from both folders, and `yarn gen:pack:real` produces both shipped packs at their pinned sha256. The `make_source_archive.sh` README wording is updated.

## 4. Gates and docs

- [ ] 4.1 Gates:
  - `build.sh --reduce en-fr` and `es-fr` from the pinned sources reproduce every committed byte in the new layout;
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-pack -p lingua-wasm -p lingua-agent`;
  - the Python tests;
  - in `apps/lingua-extension`: `yarn test`, `yarn build` and `yarn check:variants`;
  - actionlint on both workflows;
  - the `reduce` job green on the pull request.
- [ ] 4.2 Docs and spec:
  - `SOURCES.md`, the tables' READMEs and `REVIEWERS.md` describe the layout;
  - `openspec validate split-lingua-pack-tables-by-language --strict` passes, and `python3 scripts/openspec_archive_order.py split-lingua-pack-tables-by-language` exits 0;
  - change 7 is marked done in `docs/lingua/language-matrix-programme.md`.
