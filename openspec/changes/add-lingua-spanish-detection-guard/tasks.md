## 1. The guard

- [x] 1.1 `analysis/language.rs`: `detect` with the guard, the three marker tables sorted, and the elisions; `block_is_studied` and `detect_document_language` call it (design D1, D2). Unit tests:
  - the spec's scenarios;
  - the tables sorted;
  - a tie staying Spanish;
  - English's detection unchanged.
- [x] 1.2 `SPANISH_ANALYZER_VERSION` = `1.1.0` (design D3), with the version tests updated.

## 2. Gates

- [x] 2.1 `cargo test -p lingua-core -p lingua-wasm -p lingua-pack`, the English baseline unchanged; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`. In `apps/lingua-extension`: `yarn gen:wasm`, `yarn build`, `yarn check:variants`.
- [x] 2.2 `openspec validate add-lingua-spanish-detection-guard --strict` passes. In `docs/lingua/spanish-programme.md`, change 19 is marked done with the measurement.
