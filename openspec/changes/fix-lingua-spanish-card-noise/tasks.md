## 1. The reduction

- [ ] 1.1 `reduce-es-fr.py`: `read_kaikki` returns the words whose lemma entries are all proper names. `name_or_word` keeps, for a form that is its own name and another word's, the commoner reading (design D1). Tests: `miró`, `argentina`, and `rosa`, which is a name and a common word.
- [ ] 1.2 `without_letters` writes the French Wiktionary's Spanish entries without the `character` entries and the senses naming a letter, for the shared gloss reduction (D2). Tests: `a`, `de`, `be`, `ese`, `carta`, `carta de amor`.
- [ ] 1.3 Reduce the es-fr tables again from the pinned snapshot, and record the build in `pin.json`. Update the README's and `SOURCES.md`'s figures (D3).
- [ ] 1.4 `crates/lingua-pack/tests/es_fr_grammar.rs`, on the built pack:
  - `a`'s gloss opens on « À », and `de` is glossed « De »;
  - `miró` reads as *mirar*'s preterite;
  - `dolores` as *dolor*;
  - `argentina` as itself.

## 2. Gates

- [ ] 2.1 Checks:
  - the Python suites of `scripts/lingua-data`;
  - `cargo test -p lingua-pack`, `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - `pack_sources.py check-reducer` for both pairs and `check-pack` for es-fr;
  - `measure/es-pud.sh` within the gates.
- [ ] 2.2 `openspec validate fix-lingua-spanish-card-noise --strict` passes.
