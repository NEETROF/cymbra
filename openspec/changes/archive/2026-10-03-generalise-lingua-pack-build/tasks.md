## 1. The list and its reader

- [x] 1.1 `apps/lingua-extension/packs.json` holds `{ "pairs": ["en-fr"] }`. `tool/packs.mjs` exports `shippedPairs`, `packFile`, `studiedOf`, `coreAnalyzerVersion` and `packMeta`, plus a `pairs` CLI for the shell scripts. A vitest spec covers:
  - list reading, paths, and the studied side;
  - the core versions read from a `mod.rs` source, with the `\b` boundary;
  - `packMeta` on the committed testdata fixture;
  - the language → constant map checked against `crates/lingua-core/src/analysis/language.rs`, failing on a missing or wrong entry.

## 2. Building per pair

- [x] 2.1 `tool/gen_pack.sh [--real] <dir>` builds every listed pair into `<dir>/<pair>.lingua`: testdata from `scripts/lingua-data/testdata/<pair>/`, real through `build.sh <pair>`, which checks the pin. A pair without its inputs fails with the path to add. The scripts become `gen:pack` → `assets/packs` and `gen:pack:real` → `--real assets/packs`, and `gen:fixtures` is unchanged. `.gitignore` and `.prettierignore` cover `assets/packs/`. Running `yarn gen:pack` and `yarn gen:pack:real` writes `assets/packs/en-fr.lingua`; the real one has the pinned sha256.
- [x] 2.2 `build.mjs` checks each listed pack (D3), copies each one, writes each into `web_accessible_resources` (replacing the old entry), and defines `__LINGUA_PACKS__`; `manifest.json` drops `assets/pack.lingua`. `src/analyzer/engine.ts` fetches `assets/packs/${__LINGUA_PACKS__[0]}.lingua`, and the global is declared with the other defines. `yarn build` passes, and a missing or mismatched pack fails with its message (checked by hand against a pack of the wrong version).

## 3. The gate, review and release

- [x] 3.1 `tool/check_variants.mjs`: every bundle holds exactly the listed packs and exposes them, and the list equals `SHIPPED_PAIRS = ["en-fr"]` (comment: widened by `enable-lingua-spanish`). Running `yarn build && yarn check:variants` passes; with `packs.json` listing a second pair, it fails.
- [x] 3.2 `tool/make_source_archive.sh` carries each listed pair's tables and pin, with one README line per pack. The `lingua-extension-check` reviewer rebuild compares each listed pack with `$RUNNER_TEMP/pack-<pair>.lingua`. REVIEWERS.md and README.md give the per-pair commands and paths. Check: the archive script runs locally and its README lists `assets/packs/en-fr.lingua` with the pinned sha256.

## 4. Gates

- [x] 4.1 In `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test` (coverage gate), `yarn format:check`, `yarn build` (three variants) and `yarn check:variants`. `bash scripts/lingua-data/build.sh en-fr <out>` matches `pin.json`, and `cargo test -p lingua-wasm --test english_baseline` passes.
- [x] 4.2 `openspec validate generalise-lingua-pack-build --strict`; `python3 scripts/check_ci_units.py --list` is unchanged; in `docs/lingua/spanish-programme.md`, change 8 is marked done and change 9 next.
