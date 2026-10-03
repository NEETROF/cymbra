## 1. The pre-pass

- [ ] 1.1 `analysis/tokenize.rs`: NFC for Spanish tokens, and `al`/`del` split into two tokens sharing the span, the leading capital kept, nothing split inside a compound (design D1). Unit tests:
  - both contractions;
  - a capitalised one;
  - a decomposed accent;
  - `don't` still one Spanish token.

## 2. The cascade

- [ ] 2.1 `analysis/lemmatize.rs`: `lemmatize_spanish` in the order of design D2, with the accent retry. Unit tests for the order: a listed form wins over every rule, and the retry comes before the enclitics.
- [ ] 2.2 The enclitic rule (design D3):
  - the clitic order;
  - the base read as written or without its one accent, the accent checked against the base's stress;
  - the `-s`/`-d` restorations;
  - the shapes and the closed list of monosyllabic imperatives.

  Unit tests for each guard.
- [ ] 2.3 The plural fallback outside the lexicon (design D4). Unit tests for each rule and each exclusion.

## 3. Closed classes and version

- [ ] 3.1 `analysis/function_words.rs`: Spanish's six tables, sorted, with one test per class (design D5).
- [ ] 3.2 `SPANISH_ANALYZER_VERSION` = `1.0.0` (design D6). The analyser-version tests are updated.

## 4. Fixtures

- [ ] 4.1 `crates/lingua-core/tests/spanish_fixtures.rs`: at least 100 Spanish cases over a test lexicon, grouped by the stage that resolves them:
  - forms;
  - old spellings;
  - enclitics and their guards;
  - plurals and their exclusions;
  - contractions and closed classes.

## 5. Gates

- [ ] 5.1 Rust:
  - `cargo test -p lingua-core -p lingua-wasm -p lingua-pack`, the English baseline unchanged;
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`.

  In `apps/lingua-extension`: `yarn gen:wasm`, `yarn build`, `yarn check:variants` (the shipped en-fr pack still loads).
- [ ] 5.2 `openspec validate add-lingua-spanish-analysis --strict` passes. In `docs/lingua/spanish-programme.md`, change 18 is marked done.
