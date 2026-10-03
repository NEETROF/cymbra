## 1. The sources

- [ ] 1.1 `pack_sources.py` (design D1):
  - `DUMPS` names, per pair, the dumps it reads and the files it derives;
  - `derive` writes them in one pass;
  - `fetch-live` keeps each derived file as an asset of the snapshot, recorded by sha256;
  - `fetch-pinned` fetches and checks them;
  - `assets` lists the release's files.
- [ ] 1.2 `test_pack_sources.py`: a language's entries kept as written, a table cut down to its translations, the derived files recorded and published, other bytes refused.
- [ ] 1.3 `lingua-pack-update` publishes every asset of the snapshot.

## 2. The reducer

- [ ] 2.1 `reduce-es-fr.py` (design D2–D5):
  - glosses and runs by the shared rules from the French Wiktionary's Spanish entries;
  - expressions;
  - the two fallbacks, proper nouns left out;
  - `LOCUTIONS`, empty;
  - `NOTICE` crediting the French and Spanish Wiktionaries.
- [ ] 2.2 `test_reduce_es_fr.py`: the translation files read direct and backwards, one sense per part of speech, the commonest French word first, the fallbacks in order and never over a gloss, expressions only where none is.

## 3. The tables

- [ ] 3.1 Reduce today's sources (`build.sh --update es-fr`): `gloss.tsv`, `senses.tsv` and `mwe.tsv` committed, `pin.json` recording the derived files. The README and `SOURCES.md` publish the coverage (design D6).
- [ ] 3.2 The PUD gates still pass, and the English baseline does not move.

## 4. Gates

- [ ] 4.1 Checks:
  - the Python reducer tests;
  - `cargo test -p lingua-pack`, `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`;
  - `build.sh es-fr` matches `pin.json`, and `check-reducer` passes for both pairs.
- [ ] 4.2 `openspec validate add-lingua-spanish-gloss-tables --strict` passes. In `docs/lingua/spanish-programme.md`, change 22 is marked done with the figures.
