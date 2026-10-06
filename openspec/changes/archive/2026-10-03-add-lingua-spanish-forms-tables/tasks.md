## 1. The reducer

- [x] 1.1 `scripts/lingua-data/reduce-es-fr.py`, on `reduce_common.py` (design D1–D3):
  - forms from the tagged tables and the form-of senses, without combined forms;
  - one lemma per form (overrides, GSD counts, own entry, frequency, order), a combined form keeping the plain lemma it also has;
  - the 60k lemmas;
  - the attested forms;
  - `NOTICE` and `manifest.json` with the Spanish analyser version.
- [x] 1.2 `test_reduce_es_fr.py`: each rule on fixture entries — a combined form left out, a combined form that is also a plural, a homograph by counts, by an override, by its own entry, by frequency, and a multi-word target refused.

## 2. The tables

- [x] 2.1 Reduce the 2026-09-28 kaikki dump, GSD at its commit and wordfreq 3.1.1 into `tables/es-fr/`, with a `README.md` and `pin.json` recording the sources, the reducer and the pack (design D4, D6). The override list and its reasons live in the reducer. `SOURCES.md` gets an ES → FR section.
- [x] 2.2 `build.sh es-fr` builds the pack and matches `pin.json`. `lingua-pack-update` offers `es-fr`, and its monthly dry run checks both pairs (design D7).

## 3. The measurement

- [x] 3.1 `lingua-pack-measure` in `crates/lingua-pack` (design D5), with a unit test on a tiny CoNLL-U fixture.
- [x] 3.2 `measure/es-pud.sh`: PUD at its pinned commit, the pack from the committed tables, the gates. Run it, and record the figures in `tables/es-fr/README.md` and in the programme.

## 4. Gates

- [x] 4.1 Python reducer tests:
  - `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`;
  - `cargo test -p lingua-pack`;
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`.

  The extension check's pack loop builds both pairs.
- [x] 4.2 `openspec validate add-lingua-spanish-forms-tables --strict` passes. In `docs/lingua/spanish-programme.md`, change 20 is marked done with the measured figures.
