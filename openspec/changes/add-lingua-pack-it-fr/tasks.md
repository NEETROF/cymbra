# Tasks

## 0. Before the dispatch

- [ ] 0.1 [owner] The study re-run on change 9's committed ranks with both fallbacks, D4 and D5's pre-pass (D7): `FLOORS["it-fr"]` fixed at those figures less two points, up or down, the owner settling it before the first committed measurement; the figures written into the requirement *it-fr is committed…* and a test holding `FLOORS` to them. *Proposed in the implementation's pull request: the study's 89.2 / 81.8 / 73.3 (design, *The study*), the floor 87.2 / 79.8 / 71.3 in `FLOORS`, the requirement and `test_gloss_coverage.py`; the owner settles it.*

## 1. The sources (scripts/lingua-data)

- [x] 1.1 `pack_sources.py`: `EDITIONS["fr"]` gains `kaikki-fr-Italien.jsonl` and `kaikki-fr-traductions-it.jsonl`; `EDITIONS["it"]` (the Italian Wiktionary) with `kaikki-it-traductions-fr.jsonl`; `DUMPS["it-fr"]` names the three beside change 9's file (D3); `test_pack_sources.py` covers them; `gloss_coverage.py`: `FLOORS["it-fr"]` (D7); `lingua-extension-check.yml`'s floor step runs `gloss_coverage.py --pair it-fr`; `test_gloss_coverage.py` holds the committed tables to it; `SOURCES.md` names the sources with their licences and records the 0.77 GB the French derivations gain.

## 2. The reducer

- [x] 2.1 `reduce-it-fr.py` writes `tables/it-fr/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}` through `reduce_common.native_tables` with `reduce_edition_fr.FR`: definitions, the Italian Wiktionary's translations, the French tables read backwards, the typographic apostrophe read as `'` (D1, D2); D4's rule with `FR.form_of` and the exact-spelling test; D5's pre-pass (names and acronyms off common words); `lexical.tsv` as the dictionary words plus the acronym rows, the rows glossed by `acronym_gloss` with a proper-noun synonym read, the levels narrowed by `words` (D5); the expressions from the section, the tables and the split single words (D6); the report's coverage, sources and sample (D8).
- [x] 2.2 `test_reduce_it_fr.py`: each source's order and the three-word cap; `del` « Du » read as a gloss and `storie` « Pluriel de storia » kept a form; `soldi` and `te` not glossed by a surname or an acronym; `luna` keeping « Lune, satellite de la Terre »; `pelle` keeping its own gloss alone; `USA` joining `america`'s first sense and `Stati Uniti`'s; on fixture entries, a first name no dictionary word and a word glossed beside a name one, the `NATO`/`USA` rows; every levelled lemma a dictionary word; `all'improvviso` (section) and `d'accordo` (tables) expressions; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes. *The it-fr tests pass; until the dispatch, the suite holds one expected failure, `test_pack_sources.Record.test_the_committed_record_is_complete` — the pin records main's rule digest —, which the dispatch's pin clears.*

## 3. The tables

- [ ] 3.1 [owner-dispatched] `lingua-pack-update` in mode `update`, `keep: en`, for it-fr, with the owner's go-ahead for the sources release: the workflow's pull request commits `tables/it-fr/` and `tables/it/{lexical.tsv, level.tsv}`, every other studied table byte for byte; the coverage at or above the floor; `tables/it-fr/README.md` and the programme's *Measured figures* record it (D8); `bash scripts/lingua-data/build.sh it-fr /tmp/it-fr.lingua` passes against the pin. *Predicted locally before the dispatch (design, *Predicted dispatch output*); the dispatch command is in the implementation's pull request.*

- [ ] 3.2 Change 12's D10 figures re-measured on the committed `mwe.tsv` (keys, names, the window, the `di` retry) and change 11's names count, recorded in the pull request. *Predicted on the local tables (design, *Change 12's D10 figures, re-measured*); re-measured after the dispatch.*

## 4. The baseline and the checks

- [ ] 4.1 `crates/lingua-wasm/tests/italian_baseline.rs` and `support/italian.rs`: `PackSource::Tables` for it-fr; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; the pull request lists what moved (D9); verify the test passes without re-blessing afterwards. *After the dispatch: done on a scratch copy of the predicted tables and reverted — the predicted golden diff, and the fixture's checks the real tables move (`Lo` read `lo`, `città` « Cité, ville », six names read as words), are in the implementation's pull request; the checks are committed on the side branch `impl/add-lingua-pack-it-fr-task4`.*
- [ ] 4.2 `crates/lingua-pack/tests/committed_tables.rs`: it-fr built to its pin, whole and in parts; `lexical.tsv` and the levels held as D10 says; the six other packs to their pins. *After the dispatch: written and passed on the scratch copy of the predicted tables, then reverted; committed on the side branch `impl/add-lingua-pack-it-fr-task4`, with D10's « every glossed lemma ranked, the acronym rows excepted ».*

## 5. Gates

- [x] 5.1 The six pairs do not move: their baselines pass without re-blessing; `git diff --stat` shows no pin of the six touched.
- [x] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants` (no package change).
- [ ] 5.3 `openspec validate add-lingua-pack-it-fr --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-pack-it-fr` names its prerequisites, and changes 11 and 12 name it among theirs; `docs/lingua/italian-programme.md` row 16's status names this change's pull requests. *Validation and archive order pass in the implementation's pull request (changes 11 and 12 already name this change); row 16's status is the programme's, updated outside it.*

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage, the sample by source and the golden's diff, and approves the reduction's pull request.
