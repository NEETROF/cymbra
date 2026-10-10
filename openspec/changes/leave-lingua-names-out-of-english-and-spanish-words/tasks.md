# Tasks

## 0. Owner, before the implementation

- [x] 0.1 [manual] Settled by the owner on 2026-10-10 (in session): English's names rule here (Q1), every ladder follows English's new figures (Q2), the words the French Wiktionary files only as names accepted (Q3), both English readings (Q4, as recommended), no mention in the release notes (Q5). The owner settles Q1–Q5 (design, *Open questions for the owner*). The tasks below follow the recommended answers. If English's names rule is left for later (Q1), tasks 2.1–2.3 drop and English moves only in its vocabulary figures. If the frozen figures are kept (Q2), task 3.1 drops and the `level-ladder` probes of es-fr, es-en, fr-en and fr-es do not move. A reviewed list of words kept as words (Q3) adds it to 1.1 and to `check_studied`.
- [x] 0.2 [manual] Approved by the owner on 2026-10-10 (in session). Before the implementation, the owner approves that en-fr's and es-fr's output moves, which the programme's rule forbids without that approval (« en-fr and es-fr output does not move »). What moves: their packs are pinned again with a lexical table, English's analyser version is bumped, and the six goldens are re-blessed with design D7's counts (`en-fr` 19 of 142 probes, `en-es` 19 of 182, `es-fr` 7 of 137, `es-en` 6 of 176, `fr-en` and `fr-es` 2 of 213 each). The count the implementation re-measures is the one approved; any other count goes back to the owner first.

## 1. The dictionary words (scripts/lingua-data)

- [ ] 1.1 `reduce_edition_fr.py` gains `dictionary_words(glosses, runs)` (design D1, D2): the glossed lemmas, less those that have sense runs and whose every run is `PROPN`, byte-sorted. Its doc names the rule's three homes (this module for en-fr and es-fr, `reduce-fr-en.py` for fr-en) and why no shared module carries it. `test_reduce_editions.py`: *English's names* (`london`, `margaret` out; `bill` with its noun and verb runs in), *Spanish's names* (`madrid`, `maría` out; `dios`, `luna` in), a glossed lemma with no run kept, the byte order.
- [ ] 1.2 `reduce-en-fr.py` and `reduce-es-fr.py` `main` write `lexical.tsv` to the work folder, after `senses.tsv`, from `french.dictionary_words(glosses, runs)`; their module docs name it. es-fr's `estimated_levels` is not edited (D3). `test_reduce_en_fr.py` and `test_reduce_es_fr.py`: a reduction over recorded entries writes a `lexical.tsv` that leaves out its names-only lemma and keeps a word with a name's sense among its own.
- [ ] 1.3 `pack_sources.py` (`LEXICAL`, `split`'s doc) and `build.sh`'s comment say that every reference's reducer writes its dictionary words; `split` itself is not changed and is in no rule digest. `test_pack_sources.py`: `split` files en-fr's and es-fr's own `lexical.tsv`; `test_the_committed_dictionary_words_are_the_reference_s_glossed_lemmas` applies the names rule to English, Spanish and French alike (`london`, `madrid` and `paris` among the names).
- [ ] 1.4 `test_reduce_editions.py`: en-fr's and es-fr's rule digests move, and es-en's, en-es's, fr-en's and fr-es's do not.

## 2. English's names rule (crates/lingua-core)

- [ ] 2.1 `engine.rs` (design D4): `analyse_page` runs `document_names` for English as for Spanish. In `document_names`, a form that is `i` or opens on `i'` or `i’` gives no evidence for English, and a lemma `Pack::level` levels is kept as a word for English; neither reading runs for Spanish or French. Doc comments say so. Unit tests named after the scenarios of *An English document's names are set aside*: *A character of a story*, *A city*, *The pronoun I*, *A word a CEFR list levels*, *A dictionary word*, *Capitals at the head of sentences only*, *The same form as a word*. One more test checks that a Spanish document still sets aside a levelled lemma that is no dictionary word. Spanish's and French's names tests are unchanged.
- [ ] 2.2 `analysis/mod.rs`: `ANALYZER_VERSION` `1.2.0` → `1.3.0`, with a history line naming this change. The tests that write English's version follow the constant or the new number, as `ignore-lingua-soft-hyphens` D7 lists them: `language.rs` (`english_keeps_its_analyser_version_and_spanish_has_its_own`), `engine.rs` (`spec_scenario_the_page_analysis_does_not_move`), `pack.rs` (`spec_scenario_loading_the_en_fr_pack_names_english`), `tests/languages.rs`, and `tests/parity.rs` through `fixtures/golden.json` (`LINGUA_UPDATE_GOLDEN=1 cargo test -p lingua-wasm --test parity`, on its version alone). The comments that say « English's `1.2.0` » follow too (`pack.rs`, `pipeline_testdata.rs`).
- [ ] 2.3 The fixtures that carry English's version: `scripts/lingua-data/testdata/en-fr/manifest.json` and `testdata/en-es/manifest.json`, with their built bytes in `crates/lingua-pack/tests/pipeline_testdata.rs`; `crates/lingua-wasm/tests/fixtures/pack.lingua` and `apps/lingua-extension/test/fixtures/en-fr.testdata.lingua` (`build.sh --testdata en-fr`, `yarn gen:fixtures`); and the agent's English `apps/lingua-agent/rust/tests/fixtures/pack.lingua`, re-stamped through `read_container`/`write_container`.

## 3. The vocabulary figures (crates/lingua-core, D5)

- [ ] 3.1 `knowledge/vocabulary.rs`: `ENGLISH_TYPICAL_VOCABULARY` frozen again on the figures English's ladder computes over the committed en-fr pack, re-measured (`[0, 1_213, 3_074, 7_155, 14_433, 18_123]` in the prototype). Its doc gives the new provenance; `english_typical_vocabularies_are_frozen` pins the new figures. `tests/languages.rs`'s ladder tests read the constant.

## 4. The re-pin (scripts/lingua-data/tables)

- [ ] 4.1 `build.sh --reduce en-fr` and `build.sh --reduce es-fr` from their pins, fetching nothing beyond the pins' assets (design D6). Commit `tables/en/lexical.tsv`, `tables/es/lexical.tsv`, and `manifest.json` and `pin.json` of `tables/en-fr/` and `tables/es-fr/`. In each pin, `snapshot` and `sources` stay byte for byte; `reducer` and `pack` move. Every other file of `tables/en/` and `tables/es/`, and every `gloss.tsv`, `senses.tsv` and `mwe.tsv`, stays byte for byte. If any of them moves, stop and report it.
- [ ] 4.2 `build.sh --reduce es-en` and `build.sh --reduce en-es` on the new studied tables (*A change to a studied language's tables reaches every pair of that language*). Their glosses, senses and expressions stay byte for byte. Their pins' `studied` record, `pack_version` and pack are recorded again, and `pack_report.py` lists the dictionary words that moved.
- [ ] 4.3 Measure on the committed tables every figure of design *Measured*: the names-only lemmas and their ranks, the dictionary words, the universes, the typical vocabularies, the levelled names, the pack sizes. Record any that differs in the pull request.
- [ ] 4.4 `tables/en-fr/README.md` and `tables/es-fr/README.md`: the `lexical.tsv` row (« the lemmas en-fr glosses, less the 3,046 it glosses by a proper noun's senses alone »), the eleven levelled English names, and the rule's home. `tables/en-es/README.md` and `tables/es-en/README.md`: their *Dictionary words are en-fr's / es-fr's* figures. `SOURCES.md`: English's and Spanish's dictionary words are written by their reducers.

## 5. The tests that pin the data (crates/lingua-pack, crates/lingua-wasm)

- [ ] 5.1 `crates/lingua-pack/tests/committed_tables.rs`:
  - *The shipped packs keep their bytes*: each pack carries a lexical table listing its language's dictionary words;
  - *The reference packs carry a lexical table*: en-fr, es-fr and fr-en;
  - *English's names* and *Spanish's names*: `london`, `margaret`, `madrid` and `maría` are no dictionary words and are still glossed; `bill`, `dios` and `luna` are dictionary words; en-es's dictionary words are en-fr's, and es-en's are es-fr's;
  - *A month a CEFR list levels*: English's eleven levelled names, listed and counted by `Pack::dictionary_words`;
  - every Spanish levelled lemma is a Spanish dictionary word;
  - *A pair left behind* no longer assumes that Spanish's dictionary words are es-fr's glossed lemmas;
  - every pin recorded again.
- [ ] 5.2 `crates/lingua-wasm/tests/cross_native.rs`: English's universe and B1 figure (`22_337`, `3_074`) and Spanish's universe (`21_062`), re-measured.

## 6. The goldens (crates/lingua-wasm)

- [ ] 6.1 Run `LINGUA_BLESS=1 cargo test -p lingua-wasm --test english_baseline --test en_es_baseline --test spanish_baseline --test es_en_baseline --test french_baseline --test fr_es_baseline` once and commit the six goldens. The pull request carries a comparison script and its output over `origin/main`'s goldens (design D7):
  - the probe names in order;
  - every unmoved probe byte for byte;
  - each moved analysis with the tokens whose class moved: `Ruiz`, `Margaret`, `HTTP` and `Sam` in English, the nine `nombres` names in Spanish;
  - every other moved line differing only by the pack, the version, the estimates and the ladder figures;
  - the count per golden against task 0.2's.
- [ ] 6.2 In `apps/lingua-extension`, `yarn test` passes without `-u`: `word-card-es-en.txt`, `word-card-en-es.txt` and `selection-rows-fr.txt` as committed, and `packs.spec.ts` on the packs built at English's new version.

## 7. Gates

- [ ] 7.1 Nothing else moves (design D6):
  - `git diff --stat origin/main --` is empty over `tables/fr/`, `tables/fr-en/`, `tables/fr-es/`, `reduce_common.py`, `reduce_edition_en.py`, `reduce_edition_es.py`, `reduce-fr-en.py`, `apps/lingua-extension/src` and `apps/lingua-extension/test/baseline`;
  - no `gloss.tsv`, `senses.tsv` or `mwe.tsv` moves;
  - `SPANISH_ANALYZER_VERSION` and `FRENCH_ANALYZER_VERSION` are unchanged;
  - `pack_sources.py check-reducer` passes for the six pairs;
  - the reduce job reproduces every committed byte.
- [ ] 7.2 These pass:
  - `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`;
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo test -p lingua-core -p lingua-pack -p lingua-wasm -p lingua-agent`;
  - `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`;
  - in `apps/lingua-extension`: `yarn test`, `yarn lint`, `yarn typecheck`.
- [ ] 7.3 `openspec validate leave-lingua-names-out-of-english-and-spanish-words --strict` passes, and `python3 scripts/openspec_archive_order.py leave-lingua-names-out-of-english-and-spanish-words` exits 10, naming the `archiveAfter` changes still open (0 once they are archived).
- [ ] 7.4 Row 24c of `docs/lingua/language-matrix-programme.md` says where the change stands.

## 8. Owner, after the implementation

- [ ] 8.1 [manual] The owner reads:
  - the moved probes (task 6.1);
  - the names-only lemmas among English's and Spanish's 10,000 commonest (993 and 550 in the prototype);
  - the words a learner may want that design *Measured* lists.

  Then the owner decides when the next extension release carries the change (M18).
- [ ] 8.2 [manual] Once the agent plugin is rebuilt on this core, the owner reinstalls its English pack (`~/.lingua/pack.lingua`), since a pack built for `1.2.0` is refused (design *Risks*).
