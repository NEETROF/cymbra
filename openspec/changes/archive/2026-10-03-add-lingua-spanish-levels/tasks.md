## 1. The level table

- [x] 1.1 `reduce-es-fr.py` writes `level.tsv` (design D1, D2): English's band sizes as constants, given in rank order to the lemmas whose French gloss is not only a proper noun's; the manifest's meta says `levels_estimated`.
- [x] 1.2 `test_reduce_es_fr.py`: the bands in rank order, the skipped lemmas, the sizes summing to 8,302.
- [x] 1.3 Reduce the pinned sources again; `level.tsv` committed, `pin.json` re-recorded; README and `SOURCES.md` describe the table and its measurement.

## 2. The flag

- [x] 2.1 `PackMeta.levels_estimated` (optional, absent when false) and `Pack::levels_estimated()` (design D3), with tests: the flag round-trips, a pack without it reads false, the en-fr pack's bytes match its pin.
- [x] 2.2 `lingua-wasm` `levelsEstimated(language)`; the extension's `LanguagePort.levelsEstimated()` through `WasmLanguagePort`, `MessagingLanguagePort`, the background's RPC handler and the test fakes.

## 3. The labels

- [x] 3.1 The language labels module: the estimated titles and the note, and the article form (design D4).
- [x] 3.2 Réglages, the popup, the statistics (ladder and « Renforcer un niveau ») and onboarding label estimated levels; tests for both an estimated and a CEFR pack.

## 4. Gates

- [x] 4.1 Checks:
  - the Python reducer tests;
  - `cargo test` for `lingua-core`, `lingua-pack` and `lingua-wasm`, `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - the English baseline does not move;
  - `build.sh` matches both pins;
  - in `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check` and `yarn build`.
- [x] 4.2 `openspec validate add-lingua-spanish-levels --strict` passes. In `docs/lingua/spanish-programme.md`, change 23 is marked done.
