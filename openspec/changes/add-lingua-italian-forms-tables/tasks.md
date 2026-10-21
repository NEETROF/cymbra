# Tasks

## 1. The reducer (scripts/lingua-data)

- [ ] 1.1 `reduce-it-fr.py`: forms from the Italian section's inflections and form-of links, the doubtful ones out, the form of a form along one part of speech (D3); I7 as change 6 writes it (D4): the elided pieces' and truncations' words forms, the `di` family lemmas with `dell'` a form of `dello`, the other articulated prepositions out, no plain word beginning with an elided piece, no form carrying clitics; one lemma per form (D5) with the names rule; `test_reduce_it_fr.py` covers each rule on hand-written entries; `python3 -m unittest discover -s scripts/lingua-data -p "test_reduce_it_fr.py"` passes.
- [ ] 1.2 `reduce_italian_treebank.py`: PUD's and MarkIT's training section's counts of a form under each lemma and the part of speech of a word, read as `reduce_french_treebank.py` reads GSD; tests over a hand-written CoNLL-U sample.
- [ ] 1.3 Morph-it! (D3): its entries read, a form added under a ranked lemma only where the Wiktionary lists it under none, never moving a reading; the count of additions printed; tests.
- [ ] 1.4 Lemmas and ranks (D6): wordfreq `it`'s order, the elision stems out, a hyphenated word by the treebanks' evidence, a word whose own form reads as another no rank; `build.sh` keeps 60,000 lemmas for `it-*`; tests.

## 2. Sources, pipeline and tables

- [ ] 2.1 `pack_sources.py`: `DUMPS["it-fr"]` (the Italian section from the English edition's dump), `PINNED["it-fr"]` (UD Italian-PUD and MarkIT at a commit, sha256), Morph-it! as a pinned release asset, the `it` group; `test_pack_sources.py` covers them; `SOURCES.md` gains *IT → FR: Italian studied* with each source's licence and reduction.
- [ ] 2.2 `.github/workflows/lingua-pack-update.yml`: it-fr in `pair`'s options, the `it` group (it-fr writes `tables/it/`); `lingua-extension-check.yml`: it-fr built from the committed tables against its pin and checked like the six, the Italian baseline listed.
- [ ] 2.3 [owner-dispatched] `lingua-pack-update` in mode `update` for it-fr, with the owner's go-ahead for the sources release: the workflow's pull request commits `tables/it/` and `tables/it-fr/` (D9) with the measurement's figures; `bash scripts/lingua-data/build.sh it-fr /tmp/it-fr.lingua` passes against the pin.

## 3. Checks and measurement

- [ ] 3.1 `crates/lingua-pack/tests/committed_tables.rs`: Italian's tables held like French's — every word change 6's pre-pass writes is a form, no articulated preposition of the split families a form, every rank on its own lemma, the pin's studied record, the empty `tags.tsv` and `lexical.tsv` present; `cargo test -p lingua-pack --test committed_tables` passes.
- [ ] 3.2 `scripts/lingua-data/measure/it-ud.sh` (D8): PUD gated at 98.5 / 93.5 / 97, MarkIT's test section reported; its figures and D5's and D7's measurements (what decides the lemmas, M8's cost at 1,000 / 2,000 / 5,000, Morph-it!'s additions, the cut) recorded in the pull request and in `tables/it-fr/README.md`.
- [ ] 3.3 `crates/lingua-wasm/tests/italian_baseline.rs` and `support/italian.rs`: the pack from `PackSource::Tables`, the fixture kept for the tests that build it; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; the pull request says the diff is the pack's; verify the test passes without re-blessing afterwards.

## 4. Gates

- [ ] 4.1 The six pairs do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline --test fr_es_baseline` passes without re-blessing; `git diff --stat` shows no pin of the six touched.
- [ ] 4.2 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants` (no package change).
- [ ] 4.3 `openspec validate add-lingua-italian-forms-tables --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-forms-tables` names its three prerequisites; `docs/lingua/italian-programme.md` row 9's status names this change's pull requests.

## 5. Owner

- [ ] 5.1 [manual] The owner approves the publication of the sources release (`lingua-pack-sources-it-fr-<snapshot>`), reads the measurements of D5 and D7 and Morph-it!'s additions, and settles whether Morph-it! stays (I11's « verified at the forms change »).
