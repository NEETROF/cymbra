# Tasks

## 1. The reducer (scripts/lingua-data)

- [ ] 1.1 `reduce-it-fr.py`: forms from the Italian section's inflections and form-of links, the doubtful ones out, the form of a form along one part of speech, the letter class of D3, the tables' stress marks dropped by the entry's spelling and a reflexive row's pronoun dropped, a derived word's row and a clitic cluster's row giving no form (D3); I7 as change 6 writes it (D4): the elided pieces' and truncations' words forms, the `di` family lemmas with `dell'` a form of `dello`, the other articulated prepositions out, no plain word beginning with an elided piece, no form carrying clitics; one lemma per form (D5) with the names rule; `test_reduce_it_fr.py` covers each rule on hand-written entries; `python3 -m unittest discover -s scripts/lingua-data -p "test_reduce_it_fr.py"` passes.
- [ ] 1.2 `reduce_italian_treebank.py`: MarkIT's training and development sections' counts of a form under each lemma — as `reduce-fr-en.py` takes GSD's — and the part of speech of a word, as `reduce_french_treebank.py` reads it; PUD and MarkIT's test section never read here (D2); tests over a hand-written CoNLL-U sample.
- [ ] 1.3 Morph-it! (D3), when pinned: its entries read, a form added under a ranked lemma only where the Wiktionary lists it under none, never moving a reading; the count of additions printed; tests.
- [ ] 1.4 Lemmas and ranks (D6): wordfreq `it`'s order, the elision and truncation stems and the whole tokens beginning with a piece out, `un` and `buon` kept, a hyphenated word by MarkIT's evidence, a word whose own form reads as another no rank; `build.sh` keeps 60,000 lemmas for `it-*`; tests.

## 2. Sources, pipeline and tables

- [ ] 2.1 `pack_sources.py`: `DUMPS["it-fr"]` (the Italian section from the English edition's dump), `PINNED["it-fr"]` (MarkIT at a commit for its counts, sha256; PUD is not pinned here — `it-ud.sh` pins it, task 3.2), Morph-it! as a `PINNED` entry at the one-off release the owner uploads before any dispatch (`lingua-pack-sources-morph-it-0.4.7`, task 5.1), the `it` group; `test_pack_sources.py` covers them; `SOURCES.md` gains *IT → FR: Italian studied* with each source's licence and reduction.
- [ ] 2.2 `.github/workflows/lingua-pack-update.yml`: it-fr in `pair`'s options, the `it` group (it-fr writes `tables/it/`); `lingua-extension-check.yml`: it-fr built from the committed tables, whole and in parts, against its pin and checked like the six; the Italian baseline listed as it is, over its fixture.
- [ ] 2.3 `tables/it/studied.json` (`{"reference": "it-fr"}`) and `tables/it/tags.tsv` (`NOUN`) committed on the branch first, as French's were (`record-build` raises without `tags.tsv`); then [owner-dispatched], after the owner's Morph-it! upload (5.1) and go-ahead for the sources release, `lingua-pack-update` in mode `update` for it-fr: the workflow's pull request commits `tables/it/` and `tables/it-fr/` (D9) with the measurement's figures; `bash scripts/lingua-data/build.sh it-fr /tmp/it-fr.lingua` passes against the pin.

## 3. Checks and measurement

- [ ] 3.1 `crates/lingua-pack/tests/committed_tables.rs`: Italian's tables held like French's — every word change 6's pre-pass writes is a form, no articulated preposition of the split families a form (`dai` excepted, the truncation's word), every rank on its own lemma, the pin's studied record, `tags.tsv` holding `NOUN` and `lexical.tsv` empty; `cargo test -p lingua-pack --test committed_tables` passes.
- [ ] 3.2 `scripts/lingua-data/measure/it-ud.sh` (D8): PUD at a commit and sha256 pinned in the script, as `fr-ud.sh` pins it, gated at 98.5 / 93.5 / 97, MarkIT's test section reported; its figures and D5's and D7's measurements (what decides the lemmas, M8's cost at 1,000 / 2,000 / 5,000, Morph-it!'s additions, the cut) recorded in the pull request and in `tables/it-fr/README.md`.
- [ ] 3.3 The Italian baseline keeps `PackSource::Testdata` (D9): `cargo test -p lingua-wasm --test italian_baseline` passes without re-blessing, and `git diff --stat` shows `tests/baseline/it-fr.golden` untouched.

## 4. Gates

- [ ] 4.1 The six pairs do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline --test fr_es_baseline` passes without re-blessing; `git diff --stat` shows no pin of the six touched.
- [ ] 4.2 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants` (no package change).
- [ ] 4.3 `openspec validate add-lingua-italian-forms-tables --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-forms-tables` names its three prerequisites; `docs/lingua/italian-programme.md` row 9's status names this change's pull requests.

## 5. Owner

- [ ] 5.1 [manual] The owner approves the publication of the sources release (`lingua-pack-sources-it-fr-<snapshot>`), uploads — before task 2.3's dispatch — Morph-it! 0.4.7 as the one-off release `lingua-pack-sources-morph-it-0.4.7` (D2), reads the measurements of D5 and D7 and Morph-it!'s additions, and settles whether Morph-it! stays among the pinned sources (I11's « verified at the forms change »).
