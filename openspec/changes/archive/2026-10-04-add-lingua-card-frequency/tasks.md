## 1. The engine

- [x] 1.1 `lingua-wasm`: `frequencyRank(lemma, language?)` returns `Pack::rank` from that language's pack (design D3). Specs in `tests/languages.rs`:
  - a ranked lemma in each pack;
  - a lemma the pack does not rank;
  - a language with no pack, refused.

## 2. The extension

- [x] 2.1 `analyzer/engine.ts`: the wasm interface declares `frequencyRank`. `wordGrammar` returns the grammar with `rank`: a number, or `null` for an unranked lemma (D3). `analyzer/types.ts` documents `WordGrammar.rank`. Spec: the port merges the rank, and `null` for an unranked lemma.
- [x] 2.2 `reading/selection-card.ts`: `rarityText(cls, rank)` gives the five bands (D1, D4). Every word card path sets the line from its grammar answer, and the `calibration` option goes. `reading/wordpopup.ts` hides an empty line. Specs:
  - the bands at their bounds: 100, 101, 1 000, 1 001, 5 000, 5 001, 20 000 and 20 001, and `null`;
  - the deck line;
  - a pending card, and a card completed past the bound: no line;
  - `Es` at « Débutant »: « Très courant ».

## 3. Gates

- [x] 3.1 Checks:
  - `cargo test -p lingua-wasm` and `cargo test -p lingua-core`, with the English baseline unmoved;
  - `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - in `apps/lingua-extension`: `yarn gen:wasm`, `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`, `yarn build`, `yarn check:variants`.
- [x] 3.2 `openspec validate add-lingua-card-frequency --strict` passes.
