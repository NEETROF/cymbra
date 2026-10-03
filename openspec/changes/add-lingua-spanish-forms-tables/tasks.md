## 1. The reducer

- [ ] 1.1 `scripts/lingua-data/reduce-es-fr.py`, on `reduce_common.py` (design D1–D3):
  - forms from the tagged tables and the form-of senses, without combined forms;
  - one lemma per form (overrides, GSD counts, own entry, frequency, order);
  - the 60k lemmas;
  - the attested forms;
  - `NOTICE` and `manifest.json` with the Spanish analyser version.
- [ ] 1.2 `test_reduce_es_fr.py`: each rule on fixture entries — a combined form left out, a homograph by counts, by an override, by its own entry, by frequency, and a multi-word target refused.

## 2. The tables

- [ ] 2.1 Reduce the 2026-09-28 kaikki dump, GSD at its commit and wordfreq 3.1.1 into `tables/es-fr/`, with `overrides.tsv` (reasons included), a `README.md`, and `pin.json` recording the sources, the reducer and the pack (design D4, D6). `SOURCES.md` gets an ES → FR section.
- [ ] 2.2 `build.sh es-fr` builds the pack and matches `pin.json`. `lingua-pack-update` offers `es-fr` (design D7).

## 3. The measurement

- [ ] 3.1 `lingua-pack-measure` in `crates/lingua-pack` (design D5), with a unit test on a tiny CoNLL-U fixture.
- [ ] 3.2 `measure/es-pud.sh`: PUD at its pinned commit, the pack from the committed tables, the gates. Run it, and record the figures in `tables/es-fr/README.md` and in the programme.

## 4. Gates

- [ ] 4.1 Python reducer tests:
  - `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`;
  - `cargo test -p lingua-pack`;
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`.

  The extension check's pack loop builds both pairs.
- [ ] 4.2 `openspec validate add-lingua-spanish-forms-tables --strict` passes. In `docs/lingua/spanish-programme.md`, change 20 is marked done with the measured figures.
