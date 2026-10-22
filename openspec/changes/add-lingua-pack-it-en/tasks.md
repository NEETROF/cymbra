# Tasks

## 0. Before the dispatch

- [ ] 0.1 [owner] The study re-run on change 9's committed ranks with D2's fallback and D4's pre-pass (D7): `FLOORS["it-en"]` fixed at those figures less two points, up or down, the owner settling it before the first committed measurement; the figures written into the requirement *it-en is committed beside Italian's studied tables…* and a test holding `FLOORS` to them.

## 1. The sources and the pipeline

- [ ] 1.1 `pack_sources.py`: `EDITIONS["it"]` gains `kaikki-it-traductions-en.jsonl` (`("translations", "it", "en")`); `DUMPS["it-en"]` names it beside the English Wiktionary's `kaikki-Italian.jsonl`, it-en pinning its own fetch (D3); `test_pack_sources.py` covers them; `gloss_coverage.py`: `FLOORS["it-en"]`; `SOURCES.md` names the sources and licences.
- [ ] 1.2 `lingua-pack-update.yml`: it-en among the pairs, reduced after it-fr, `--test it_en_baseline` run; `test_reduce_loops.py` holds the options; `lingua-extension-check.yml`: the gloss-coverage floor step runs `--pair it-en` and is renamed for the pairs it holds, and `--test it_en_baseline` runs beside the Italian baseline (D7, D9).

## 2. The reducer

- [ ] 2.1 `reduce_italian_en.py` (D4): the reviewed list of carried wordings without fr-en's `synonym`, Italian's three shapes added; the acronym half of it-fr's D5 with `pil`'s exemption, and names off a word with a function word's entry; Greek letters' names where the row has another sense; notes out; the split words of the section and the Italian Wiktionary — without editing `reduce_edition_en.py` or `reduce_common.py`.
- [ ] 2.2 `reduce-it-en.py`: reads `tables/it/` as committed; `IT` matching it-fr's token pattern; the section through the English edition's pre-passes, then D4's; `common.native_tables(…, fallbacks=[(direct, list)])` with the Italian Wiktionary's translations (D2); the expressions of D5; the acronym rows by change 11's rule (D6); the notice and manifest of D8; writes `tables/it-en/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}` (`record-build` writes the pin, a person the README).
- [ ] 2.3 `test_reduce_it_en.py`: `della` « of the, from the », `del` « some, of the, from the », `ed` « and », `ad` « to, at, in » without « CEO », `gli` keeping « the », `mi` « me » without « mu »; `quanti` keeping « how much, how many »; `pil` keeping « gross domestic product »; `eta` keeping its letter; `natale` keeping « Christmas »; `io` without the moon; `perfino` « even » from the Italian Wiktionary; a word only the English tables read backwards would gloss, left unglossed; `un po'`, `d'accordo` and a direct-table expression in `mwe.tsv`; `USA`'s row glossed through its synonyms; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes.

## 3. The tables

- [ ] 3.1 [owner-dispatched] `lingua-pack-update` in mode `update` for it-en, with the owner's go-ahead for its sources release: the workflow's pull request commits `tables/it-en/`, `tables/it/` byte for byte; the coverage at or above the floor; `tables/it-en/README.md` and the programme's *Measured figures* record it (D8); `bash scripts/lingua-data/build.sh it-en /tmp/it-en.lingua` passes against the pin; the golden and the English card snapshot blessed in that pull request (D9).

## 4. The baseline and the checks

- [ ] 4.1 `crates/lingua-wasm/tests/it_en_baseline.rs` and `baseline/it-en.golden` over `pages-it.txt`; a test that the golden is it-fr's on the studied side; `cross_native.rs`: the Italian corpus read with it-fr's and it-en's packs alike but for the glosses (D9).
- [ ] 4.2 `crates/lingua-pack/tests/committed_tables.rs`: it-en built to its pin, whole and in parts; its credits (the notice's sources, `levels_estimated`, the analyser version), as `spec_scenario_the_credits` holds es-en's; listed in no package while the list does not name it (D8, D10).
- [ ] 4.3 `apps/lingua-extension/test/word-card-it-en.spec.ts`: change 18's English card snapshot over the real it-en pack (its D8).

## 5. Gates

- [ ] 5.1 Nothing else moves: `git diff --stat origin/main -- scripts/lingua-data/reduce_edition_en.py scripts/lingua-data/reduce_common.py scripts/lingua-data/tables crates/lingua-wasm/tests/baseline ':!scripts/lingua-data/tables/it-en' ':!crates/lingua-wasm/tests/baseline/it-en.golden'` empty; `packs.json` unchanged.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; the Python tests; `gloss_coverage.py --check`; in `apps/lingua-extension`: `yarn test`.
- [ ] 5.3 `openspec validate add-lingua-pack-it-en --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-pack-it-en` names its prerequisites; the MODIFIED block re-copied from `add-lingua-pack-fr-en` as it then stands, only D11's clauses differing; `docs/lingua/italian-programme.md` row 22's status names this change's pull requests.

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage, the sample by source, the golden and the card snapshot, and approves the reduction's pull request.
