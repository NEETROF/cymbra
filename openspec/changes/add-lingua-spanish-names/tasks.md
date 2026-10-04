## 1. The analysis

- [x] 1.1 `lingua-core` `engine.rs`: `document_names` and `mid_sentence` (design D1, D2). `analyse_page` sets a Spanish document's names aside as `ProperNounOutOfLexicon`, only for tokens the knowledge model reads `Unknown` (D3). Unit tests for `mid_sentence`:
  - after a word, a comma, or a semicolon followed by a non-breaking space;
  - at the head of a block, after a full stop, `¿`, a dialogue dash or `«`;
  - an offset inside a character.
- [x] 1.2 `SPANISH_ANALYZER_VERSION` becomes `1.2.0` (D4), and the tests that name it follow. Reduce the es-fr tables again: only the manifest and the pin change.
- [x] 1.3 `crates/lingua-pack/tests/es_fr_grammar.rs`, on the built pack:
  - Augusto, Eugenia and la Nela set aside, the block-initial `Augusto` included;
  - `Augusto` beside `el augusto monarca`;
  - `Augusto` capitalised only at the head of sentences;
  - `Dios`, which the pack glosses.

## 2. Gates

- [x] 2.1 Checks:
  - `cargo test -p lingua-core -p lingua-wasm -p lingua-pack`, with the English baseline unmoved;
  - `wasm-pack test --node crates/lingua-wasm`;
  - `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - `pack_sources.py check-pack` for es-fr.
- [x] 2.2 `openspec validate add-lingua-spanish-names --strict` passes.
