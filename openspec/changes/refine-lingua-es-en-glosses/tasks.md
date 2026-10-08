# Tasks

## 1. The rules (scripts/lingua-data)

- [ ] 1.1 `reduce_edition_en.py`: `read_as_meanings(src, dst)`, its nested-sense rule — a sense whose parent ends on a colon, names senses, or is a pointer (a pointer sense of the same entry, or `_FORM_OF`) is read by `glosses[-1]`; any other parent stays (D2). `test_reduce_editions.py`, a new class `EsEnGlossesReadAsMeanings(Entries)`, on recorded entries: *Sense-group labels* (venir), *A list's introduction* (cusco), *A parent that is a meaning* (hacer), the nested pointer of « su » and « sí », a nested pointer that stays a pointer (« ellipsis of goma de mascar »).
- [ ] 1.2 `reduce_edition_en.py`: the shortened forms — the meaning a pointer carries (after a comma, a semicolon, a colon, or in “ ”), else its target's meaning senses in the same part of speech, following the target's own pointer once, in the sense's place, pointer tags and fields dropped, other tags kept; `_FORM_OF` gains « apocopic », « apheretic », « syncopic », « prepositional form of », « pronunciation spelling of », « eye dialect spelling of » (D3). Tests: *An apocope that carries its meaning* (mi, muy, un « (“one”) »), *An apocope that carries none* (su ← suyo), a two-step target (« toy » → « estoy » → « estar »), a form with no meaning and no lender read as a pointer.
- [ ] 1.3 `reduce_edition_en.py`: a capitalised headword's `name` entries written after every other line when the lower-case headword has a preposition, conjunction, pronoun, determiner or article entry (D4). Tests: *A function word spelled like a place* (como/Como); `chile`/`Chile` and `amor`/`Amor` left in file order.
- [ ] 1.4 `reduce_edition_en.py`: the typography — the closed list of description openers lowered (not « The » before a capital, not before a non-letter), `...` → `…` spaced between two words, an even number of `"` paired “ ”, « [sense N] » and a parenthesis naming « sense N » removed, a gloss cut at its first line break (D5). Tests: each scenario of *es-en's glosses are written in one English typography*, an odd number of quotes kept, single quotes kept, a final « … » kept by the shared cleaning (« whether … or … »), an expression getting the same rules.
- [ ] 1.5 `reduce-es-en.py`: `english.read_as_meanings` run after `without_letter_headwords` and before `merge_same_pos_etymologies`, the docstrings of both modules naming the rules and their census (design *Measured*) (D1). `test_reduce_editions.py`: the digest test shows en-fr's, es-fr's and en-es's rule sets unchanged and es-en's moved (*Nothing else moves*); `test_pack_sources.py` unchanged.

## 2. The re-pin (scripts/lingua-data/tables/es-en)

- [ ] 2.1 `build.sh --reduce es-en` from `lingua-pack-sources-es-en-2026.10.08`, nothing fetched beyond its assets: `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `manifest.json` and `pin.json` committed, the snapshot and the `studied` record unchanged; `gloss_coverage.py --pair es-en` passes against `FLOORS["es-en"]`; en-fr's, es-fr's and en-es's tables and pins byte for byte unchanged (D7).
- [ ] 2.2 `tables/es-en/README.md` and `SOURCES.md` (*ES → EN*, *The Wiktionary editions' rules*): the rules, the new figures (lemmas, expressions, coverage, the pack's size), and what is left (design D6).

## 3. The baselines and the sample (crates/lingua-wasm, apps/lingua-extension)

- [ ] 3.1 `LINGUA_BLESS=1 cargo test -p lingua-wasm --test es_en_baseline`: `baseline/es-en.golden` re-blessed; `es_en_baseline.rs`'s `VIAJE_FIRST_PAGE` follows D5; `the_golden_is_the_spanish_one_on_the_studied_side` passes unchanged; en-fr's and es-fr's goldens unchanged (D8).
- [ ] 3.2 `yarn vitest run test/word-card-es-en.spec.ts test/row-gloss-tables.spec.ts -u` in `apps/lingua-extension`: `test/baseline/word-card-es-en.txt` re-blessed, `test/baseline/selection-rows-fr.txt` unchanged, no row of any pair ending on an opening mark (were one to, change 23's D4 rule applies: the cut's trailing set gains the mark, and the French snapshot proves no French row moved) (D8).
- [ ] 3.3 In the pull request: every changed line of the golden and of the snapshot with the rule that moved it, and the before/after sample — every changed row of the top 10,000 with its first differing sense, the changed expressions by rule, 30 rows drawn from the rest; if M20 is settled first, the sample is drawn with its values, and if this change lands first, change 21's two samples are drawn again on its tables (D9).

## 4. Owner

- [ ] 4.1 [manual] The owner reviews the sample and every re-blessed line (M9).
- [ ] 4.2 [manual] The owner answers Q1–Q4 (design *For the owner*): an answer that is a rule of the English edition joins this change before it merges, es-en re-pinned again; any other is named as a follow-up in the pull request, with the shared ones of D6 (« etc »'s period in every pair, a case-aware card).

## 5. Gates and docs

- [ ] 5.1 The Python tests: `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`.
- [ ] 5.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-pack -p lingua-wasm` (`committed_tables.rs`: es-en's pack against its new pin).
- [ ] 5.3 `scripts/lingua-data/build.sh --reduce es-en <out>` reproduces the committed tables, manifest and pin byte for byte, and `pack_sources.py check-reducer` passes for every pair; the `reduce` job green on the pull request.
- [ ] 5.4 In `apps/lingua-extension`: `yarn test` (the snapshots), `yarn lint`, `yarn typecheck`.
- [ ] 5.5 `openspec validate refine-lingua-es-en-glosses --strict` passes, and `python3 scripts/openspec_archive_order.py refine-lingua-es-en-glosses` exits 10 naming `add-lingua-pack-es-en` and `add-lingua-english-card-wording` (0 once they are archived).
- [ ] 5.6 Row 23b of `docs/lingua/language-matrix-programme.md` says where the change stands.
