# Tasks

## 0. Before the dispatch

- [ ] 0.1 [owner] The study re-run on change 9's committed ranks with D2's fallback and D4's pre-pass (D7): `FLOORS["it-en"]` fixed at those figures less two points, up or down, the owner settling it before the first committed measurement; the figures written into the requirement *it-en is committed beside Italian's studied tables…* and a test holding `FLOORS` to them.

## 1. The sources (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py`: `EDITIONS["it"]` gains `kaikki-it-traductions-en.jsonl` (`("translations", "it", "en")`); `DUMPS["it-en"]` names it beside change 9's `kaikki-Italian.jsonl` (D3); `test_pack_sources.py` covers them; `gloss_coverage.py`: `FLOORS["it-en"]`; `lingua-extension-check.yml`'s floor step runs `gloss_coverage.py --pair it-en`; `test_gloss_coverage.py` holds the committed tables to it; `SOURCES.md` names the sources and licences.

## 2. The reducer

- [ ] 2.1 `reduce_italian_en.py`: the pre-pass of D4 — meanings carried by a pointer (`del` family, `ed`/`ad`, `gli`, `mi`), names off lowercase lemmas and off function words, notes out — without editing `reduce_edition_en.py` or `reduce_common.py`.
- [ ] 2.2 `reduce-it-en.py`: reads `tables/it/` as committed; the section through the English edition's pre-passes (as `reduce-es-en.py` calls them) and D4's; `common.native_tables(…, fallbacks=[(direct, list)])` with the Italian Wiktionary's translations (D2); the expressions of D5; the acronym rows glossed by change 11's rule (D6); writes `tables/it-en/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}` (`record-build` writes the pin, a person the README).
- [ ] 2.3 `test_reduce_it_en.py`: `del` « of the, from the », `gli` keeping « the », `mi` « me »; `torno` « lathe » alone; `io` without the moon; a letter glossing nothing; `perfino` « even » from the Italian Wiktionary; a word only the English tables read backwards would gloss, left unglossed; `un po'`, `d'accordo` and a direct-table expression in `mwe.tsv`; `USA`'s row glossed through its synonyms; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes.

## 3. The tables

- [ ] 3.1 [owner-dispatched] `lingua-pack-update` in mode `update` for it-en, with the owner's go-ahead for its sources release: the workflow's pull request commits `tables/it-en/`, `tables/it/` byte for byte; the coverage at or above the floor; `tables/it-en/README.md` and the programme's *Measured figures* record it (D8); `bash scripts/lingua-data/build.sh it-en /tmp/it-en.lingua` passes against the pin.

## 4. The baseline and the checks

- [ ] 4.1 `crates/lingua-wasm/tests/it_en_baseline.rs` and `baseline/it-en.golden` over `pages-it.txt`, blessed in this pull request; `cross_native.rs`: the Italian corpus read with it-fr's and it-en's packs alike but for the glosses (D9).
- [ ] 4.2 `crates/lingua-pack/tests/committed_tables.rs`: it-en built to its pin, whole and in parts; listed in no package (D10).

## 5. Gates

- [ ] 5.1 Nothing else moves: `reduce_edition_en.py`, `reduce_common.py` and every other pair's tables, pins and goldens unchanged (`git diff --stat origin/main -- scripts/lingua-data/tables ':!scripts/lingua-data/tables/it-en'` empty); `packs.json` unchanged.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; the Python tests; `gloss_coverage.py --check`.
- [ ] 5.3 `openspec validate add-lingua-pack-it-en --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-pack-it-en` names its prerequisites; `docs/lingua/italian-programme.md` row 22's status names this change's pull requests.

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage, the sample by source and the golden, and approves the reduction's pull request.
