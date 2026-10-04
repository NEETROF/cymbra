## 1. The engine

- [ ] 1.1 `lingua-wasm` `level_ladder`: for a pack whose levels are estimated, the typical vocabularies come from the loaded English pack with CEFR-list levels, and each row says `typicalFrom: "en"`. The key is absent otherwise (design D1). Test in `tests/languages.rs`: a Spanish pack with estimated levels borrows English's figures and says so, and English's rows carry no `typicalFrom`.

## 2. The extension

- [ ] 2.1 `LevelRow.typicalFrom`. `ladderView`'s legend ends with `borrowedTypicalNote` (`language-labels.ts`) when the rows carry it (D2). Test in `test/stats.spec.ts`: the Spanish legend names English and drops « extrapolé », and the English legend keeps it.

## 3. Gates

- [ ] 3.1 Checks:
  - `cargo test -p lingua-wasm`, with the English baseline unmoved;
  - `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - in `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test` and `yarn format:check`.
- [ ] 3.2 `openspec validate fix-lingua-spanish-ladder-estimates --strict` passes.
