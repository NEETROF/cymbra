## 1. The labels

- [ ] 1.1 `reading/grammar-labels.ts` (design D1–D4):
  - the Spanish verb-form names, and persons merged within a tense;
  - articles and elision;
  - the agreement of nominal forms;
  - no line for the dictionary form itself.
- [ ] 1.2 `test/word-grammar.spec.ts`:
  - each Spanish name;
  - `hablaba`, `hable`, `rápidas`, `hablar` and `casa`;
  - elision before an accented vowel;
  - English's lines unchanged.

## 2. The card

- [ ] 2.1 `WordPopupContent.language`, set by the reading session and passed by `renderGrammar` (design D5); tests that a Spanish card names Spanish forms and a card with no language reads as English.

## 3. The runs

- [ ] 3.1 `reduce-es-fr.py` gives a noun run its gender (design D6), with tests. Reduce the pinned sources again: only `senses.tsv` and the pin change.

## 4. Gates

- [ ] 4.1 Checks:
  - the Python reducer tests;
  - `cargo test -p lingua-pack`, `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - the English baseline does not move, and `build.sh` matches both pins;
  - in `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check` and `yarn build`.
- [ ] 4.2 `openspec validate add-lingua-spanish-word-card --strict` passes. In `docs/lingua/spanish-programme.md`, change 24 is marked done.
