## 1. The reduction

- [ ] 1.1 `reduce-es-fr.py`: a `standard` row is neither a candidate's form nor a reading (design D1). The apocopic sense of an `adj` or `det` entry is a form of its first `alt_of` target's first word (D2). Tests in `test_reduce_es_fr.py`:
  - `buen` and `bueno`;
  - `algún`, and `cualesquier`'s split targets;
  - `muy` and `un` staying their own words.
- [ ] 1.2 Reduce the es-fr tables again from the pinned snapshot, and record the build in `pin.json` (D3). Update the README's and `SOURCES.md`'s figures.
- [ ] 1.3 `crates/lingua-pack/tests/es_fr_grammar.rs`: on the built pack, `buen`, `bueno` and `buenos` read as *bueno* glossed « Bon », `malo` as *malo*, `gran` as *grande*, `algún` as *alguno*, and `muy` as *muy* glossed « Très ».

## 2. Gates

- [ ] 2.1 Checks:
  - the Python suites of `scripts/lingua-data`;
  - `cargo test -p lingua-pack`, `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - `pack_sources.py check-reducer` for both pairs and `check-pack` for es-fr;
  - `measure/es-pud.sh` within the gates.
- [ ] 2.2 `openspec validate fix-lingua-spanish-apocopes --strict` passes.
