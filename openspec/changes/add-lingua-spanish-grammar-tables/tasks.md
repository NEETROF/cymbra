## 1. The reducer

- [ ] 1.1 `reduce-es-fr.py` writes `grammar.tsv` (design D1–D5):
  - verb tags, and nominal tags with a noun's gender;
  - a lemma's table first, a form's own entry for the pairs no table lists;
  - `other` on each reading of another kept lemma.
- [ ] 1.2 `test_reduce_es_fr.py`, each mapping on fixture entries:
  - the moods and tenses, the conditional, *usted* and *vos*;
  - the negative imperative and the bare participle left out;
  - a gender from the head and from the senses, a plural-only noun, an adjective's agreement;
  - a pronominal form from its own entry, a combined form left out, the `other` mark.

## 2. The tables

- [ ] 2.1 Reduce the pinned sources again: `grammar.tsv` committed, `forms.tsv` and `freq.tsv` byte-identical, `pin.json` re-recorded. The README and `SOURCES.md` name the table.
- [ ] 2.2 `crates/lingua-pack/tests/es_fr_grammar.rs`: the pack built from the committed tables answers the card (design D7).

## 3. Gates

- [ ] 3.1 Checks:
  - the Python reducer tests;
  - `cargo test -p lingua-pack`, `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - `build.sh es-fr` matches `pin.json`, and `check-reducer` passes for both pairs;
  - the English baseline does not move.
- [ ] 3.2 `openspec validate add-lingua-spanish-grammar-tables --strict` passes. In `docs/lingua/spanish-programme.md`, change 21 is marked done with the figures.
