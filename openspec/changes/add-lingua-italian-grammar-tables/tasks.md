# Tasks

## 1. The reducer (scripts/lingua-data)

- [ ] 1.1 `reduce-it-fr.py` writes `grammar.tsv` (`form<TAB>lemma<TAB>tag<TAB>other|-`, sorted) beside the forms and ranks, in the same pass over `kaikki-Italian.jsonl`, on `reduce_common.py` without editing it (design D1): the forms of `forms.tsv` under the lemmas of `freq.tsv` (D2); verb forms by D3's table, the table forms read bare of their stress marks and a reflexive row without its pronoun as change 9 reads them, no reading from a negative or formal imperative row, the bookkeeping rows, a compound tense or a `-` cell; nouns, adjectives, determiners, pronouns and numerals by D4; D5's sources and exclusions; `other` marks by D6.
- [ ] 1.2 `test_reduce_it_fr.py`, each rule on fixture entries shaped as the English dump writes them: `parlò`, `parlavo`, `parli` (four readings), `parla` (two), `parlerebbe` (no tense), `parlando`, `parlante`, the table's bare past participle; `diedi` from `dièdi`; `lavo` from `mi làvo` under *lavare*; a negative imperative row, a formal one and a `-` cell giving nothing; `case`, `città` (both numbers), `uova` (feminine plural of *uovo*), `rosse`, `grandi`, `bel`; `la` as *il*; `creduta` through `creduto`; `attore` no form of *attrice*; an alternative-only entry and a capitalised headword giving nothing; an `other` mark and one toward a regional-only word refused; `python3 -m unittest discover -s scripts/lingua-data -p "test_reduce_it_fr.py"` passes.

## 2. The tables

- [ ] 2.1 [owner-dispatched] `lingua-pack-update` in mode `reduce` for it-fr from its pinned sources: `tables/it/grammar.tsv` committed; `forms.tsv`, `freq.tsv`, `lexical.tsv` and `studied.json` byte-identical; `tables/it-fr/{manifest.json, pin.json}` recorded again (base and layer); the figures of D2, D6 (M8's cost at 1,000 / 2,000 / 5,000) and D7 recorded in the pull request and the README; *Known data defects* filled.
- [ ] 2.2 Write `tables/it/tags.tsv` once from the reduction's readings (`cut -f3 grammar.tsv | LC_ALL=C sort -u`), replacing the placeholder `NOUN`, and review it in the pull request: every line a canonical tag, nothing an Italian form should not carry (D8); record the pack again.
- [ ] 2.3 `tables/it-fr/README.md` (the readings' rules D3–D6, the figures, the pool), `scripts/lingua-data/SOURCES.md` (the tag pool's row names Italian's pin; the Italian section names `grammar.tsv`).

## 3. Checks and measurement

- [ ] 3.1 `crates/lingua-pack/tests/it_fr_grammar.rs`, the it-fr pack built from the committed tables (D10): `parlò`, `parli` (four readings, in the pool's order), `parla`, `parlerebbe`, `parlando`, `parlante`, `diedi`, `lavo`, `case`, `città`, `rosse`, `grandi`, `la` as *il*, a form naming another ranked lemma, `porta` as *portare* naming nothing; every probe asks a lemma the tables keep.
- [ ] 3.2 `crates/lingua-pack/tests/committed_tables.rs`: `tables/it/` holds `grammar.tsv`; `tables/it/tags.tsv` holds exactly the tags of `tables/it/grammar.tsv`, canonical, each once, failing by name on a tag missing or left over; the six other packs build to their pins.
- [ ] 3.3 `scripts/lingua-data/measure/it_readings.py`, run by `measure/it-ud.sh` after its gates (D9): the readings' figures per part of speech on PUD and MarkIT's test section, never deciding the exit status; each rule switched off once, its effect recorded in the pull request; a test on a hand-written CoNLL-U fixture.

## 4. Gates

- [ ] 4.1 Nothing else moves (D10): `git diff --stat origin/main` shows no table of `tables/{en,es,fr}` or their pairs, no pin of the six, no golden; the six baselines and `cross_native` pass without re-blessing.
- [ ] 4.2 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`.
- [ ] 4.3 `openspec validate add-lingua-italian-grammar-tables --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-grammar-tables` names its prerequisites; `docs/lingua/italian-programme.md` row 13's status names this change's pull requests.

## 5. Owner

- [ ] 5.1 [manual] The owner reads the pin (2.2), the readings' figures and M8's cost, and the known data defects, and approves the reduction's pull request.
