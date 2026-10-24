# Tasks

## 0. Before the dispatch

- [x] 0.1 [owner] The study re-run on change 9's committed ranks with both fallbacks, D4 and D5's pre-pass (D7): `FLOORS["it-fr"]` fixed at those figures less two points, up or down, the owner settling it before the first committed measurement; the figures written into the requirement *Done 2026-10-24: the owner settled the floor at 87.2 / 79.8 / 71.3.*

## 1. The sources (scripts/lingua-data)

- [x] 1.1 `pack_sources.py`: `EDITIONS["fr"]` gains `kaikki-fr-Italien.jsonl` and `kaikki-fr-traductions-it.jsonl`; `EDITIONS["it"]` (the Italian Wiktionary) with `kaikki-it-traductions-fr.jsonl`; `DUMPS["it-fr"]` names the three beside change 9's file (D3); `test_pack_sources.py` covers them; `gloss_coverage.py`: `FLOORS["it-fr"]` (D7); `lingua-extension-check.yml`'s floor step runs `gloss_coverage.py --pair it-fr`; `test_gloss_coverage.py` holds the committed tables to it; `SOURCES.md` names the sources with their licences and records the 0.77 GB the French derivations gain.

## 2. The reducer

- [x] 2.1 `reduce-it-fr.py` writes `tables/it-fr/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json}` through `reduce_common.native_tables` with `reduce_edition_fr.FR`: definitions, the Italian Wiktionary's translations, the French tables read backwards, the typographic apostrophe read as `'` (D1, D2); D4's rule with `FR.form_of` and the exact-spelling test; D5's pre-pass (names and acronyms off common words); `lexical.tsv` as the dictionary words plus the acronym rows, the rows glossed by `acronym_gloss` with a proper-noun synonym read, the levels narrowed by `words` (D5); the expressions from the section, the tables and the split single words (D6); the report's coverage, sources and sample (D8).
- [x] 2.2 `test_reduce_it_fr.py`: each source's order and the three-word cap; `del` « Du » read as a gloss and `storie` « Pluriel de storia » kept a form; `soldi` and `te` not glossed by a surname or an acronym; `luna` keeping « Lune, satellite de la Terre »; `pelle` keeping its own gloss alone; `USA` joining `america`'s first sense and `Stati Uniti`'s; on fixture entries, a first name no dictionary word and a word glossed beside a name one, the `NATO`/`USA` rows; every levelled lemma a dictionary word; `all'improvviso` (section) and `d'accordo` (tables) expressions; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes. *Passes since the dispatch's pin recorded the rules: until then the suite held one expected failure, `test_pack_sources.Record.test_the_committed_record_is_complete`.*

## 3. The tables

- [x] 3.1 [owner-dispatched] `lingua-pack-update` in mode `update`, `keep: en`, for it-fr, with the owner's go-ahead for the sources release: the workflow's pull request commits `tables/it-fr/` and `tables/it/{lexical.tsv, level.tsv}`, every other studied table byte for byte; the coverage at or above the floor; `tables/it-fr/README.md` and the programme's *Done 2026-10-24: `lingua-pack-update` run 40130354122 (mode `update`, `keep: en`, on `af20e83d6`), the release `lingua-pack-sources-it-fr-2026.10.24` (`kaikki-fr-Italien.jsonl` 769,483,152 B, sha256 `eb79761b…`; `kaikki-fr-traductions-it.jsonl` 7,469,549 B, `a39a2ad5…`; `kaikki-it-traductions-fr.jsonl` 1,273,076 B, `424c527e…`), the commit `40aafd84f`: the pack 2,645,342 B, sha256 `0e6795d9…`, its base 1,551,534 B, `f8c78228…`, its layer 1,094,048 B, `0927f745…`; coverage 89.2 / 81.8 / 73.3 as the update's reduction measured it, at or above the floor; byte for byte the implementation's prediction; `tables/it/{forms,freq,grammar,tags}.tsv` unmoved.*

- [x] 3.2 Change 12's D10 figures re-measured on the committed `mwe.tsv` (keys, names, the window, the `di` retry) and change 11's names count, recorded in the pull request. *Measured on the committed tables, equal to the prediction: 14,776 keyed of 18,296, 61 past seven tokens, 4,677 named, 140 shared keys, 157 keys ending on `di`, 3,000 `di` retries on 91 expressions over change 11a's corpus; Italian's names count 8,476 (design, *Implementation*).*

## 4. The baseline and the checks

- [x] 4.1 `crates/lingua-wasm/tests/italian_baseline.rs` and `support/italian.rs`: `PackSource::Tables` for it-fr; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; the pull request lists what moved (D9); verify the test passes without re-blessing afterwards. *Re-blessed once on the committed tables, the diff equal to the predicted one (150 of 176 sections); passes unblessed.*
- [x] 4.2 `crates/lingua-pack/tests/committed_tables.rs`: it-fr built to its pin, whole and in parts; `lexical.tsv` and the levels held as D10 says; the six other packs to their pins. *With D10's « every glossed lemma ranked, the acronym rows excepted »; 63 checks pass.*

## 5. Gates

- [x] 5.1 The six pairs do not move: their baselines pass without re-blessing; `git diff --stat` shows no pin of the six touched.
- [x] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants` (no package change).
- [ ] 5.3 `openspec validate add-lingua-pack-it-fr --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-pack-it-fr` names its prerequisites, and changes 11 and 12 name it among theirs; `docs/lingua/italian-programme.md` row 16's status names this change's pull requests. *Validation and archive order pass in the implementation's pull request (changes 11 and 12 already name this change); row 16's status is the programme's, updated outside it.*

## 6. Owner

- [ ] 6.1 [manual] The owner reads the coverage, the sample by source and the golden's diff, and approves the reduction's pull request.
