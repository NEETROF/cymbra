## 1. Data pipeline

- [ ] 1.1 `reduce-en-fr.py`: emit `mwe.tsv` (headword TAB gloss, sorted) from the multi-word kaikki entries, reusing the existing `_FORM_OF` filter, the round-robin sense picker and the two cuts (42 characters per sense, 80 on the joined string) — dropping name-only entries, non-ASCII headwords and entries left senseless
- [ ] 1.2 In the multi-word path only, drop a sense whose wording `_FORM_OF` lacks (`Présent`, `Futur`, `Conjugaison`, a bare `Graphie`), with a local pattern, so the shared filter and the single-word tables are untouched — assert in a test that `forms.tsv`, `freq.tsv` and `gloss.tsv` are byte-identical before and after this change
- [ ] 1.3 Add `mwe.tsv` to `scripts/lingua-data/testdata/en-fr/` with a handful of expressions the other testdata words can reach, one of them reachable by two spellings
- [ ] 1.4 Bump `pack_version` in `scripts/lingua-data/testdata/en-fr/manifest.json` (`0.0.0-testdata` → `0.0.1-testdata`): the fixture pack's content moves, and the container requirement says a new table bumps it
- [ ] 1.5 Python tests for the new reduction, on the committed testdata, including a name-only entry and two spellings reaching one headword

## 2. Pack builder

- [ ] 2.1 `PackInputs` gains `expressions: Vec<(String, String)>`; `inputs_from_dir` reads the optional `mwe.tsv` exactly as `read_levels` reads `level.tsv` (absent → empty, no error)
- [ ] 2.2 The builder keys each entry by joining its words through `lingua_core::analysis::lemmatize::lemmatize` against the lexicon it has just built, and keeps a key only when every word is in that lexicon (`Lexicon`); a test pins `starting point` → `start point`
- [ ] 2.3 Where two entries reach one key, the entry whose headword is already the key wins; a test pins `break point` over `breaking point`
- [ ] 2.4 Write the section: an `fst::Map` of keys → offsets into a zstd gloss blob, byte-wise sorted and unique, emitted only when non-empty; a pack built from inputs carrying no `mwe.tsv` is byte-identical to one built before this change
- [ ] 2.5 The licence guard needs no new source: assert in a test that supplying expressions adds no entry to `PackInputs.sources` and leaves the NOTICE unchanged
- [ ] 2.6 `BuildError::OverBudget` names what is at fault rather than always "glossed lemmas", and a test drives a pack over the budget with a large expression table and reads the message

## 3. Core — reading and lookup

- [ ] 3.1 `packs::pack`: the new section constant and its parse, mirroring `levels`, plus `Pack::expression(key) -> Option<&str>`; tests that a pack without the section loads and answers `None`, and that a pack carrying an unknown section name loads unchanged — the additive contract
- [ ] 3.2 `engine::gloss_phrase` reports matches: longest run first, at most five tokens, non-overlapping, keyed by the tokens' dictionary forms — a new field, skipped in the JSON when empty so a pack without the table produces the bytes it produces today
- [ ] 3.3 Host tests, one per scenario of the `lingua-analysis` delta, on an inline pack carrying a few expressions
- [ ] 3.4 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-pack`, coverage ≥ 80 %

## 4. WASM, fixtures and parity

- [ ] 4.1 The phrase-gloss JSON carries the matches; `src/analyzer/types.ts` follows
- [ ] 4.2 Rebuild `crates/lingua-wasm/tests/fixtures/pack.lingua` from the testdata (it predates the level table too), regenerate `phrase_golden.json` with `LINGUA_UPDATE_PHRASE_GOLDEN`, and check a match appears in it
- [ ] 4.3 `git diff --exit-code crates/lingua-wasm/tests/fixtures/golden.json` is clean and the page-analysis parity test passes with no update variable: `analyse_page` did not move
- [ ] 4.4 `yarn gen:fixtures`: regenerate the committed `apps/lingua-extension/test/fixtures/en-fr.testdata.lingua`
- [ ] 4.5 `scripts/lingua-data/build.sh emit-manifest`: refresh `backend/lingua/packs-manifest.json`, whose entry records the testdata pack's size, and leave `check-manifest` clean

## 5. Extension — the card

- [ ] 5.1 `selection-card.ts`: a match covering the whole selection becomes the answer — headword and key = the expression's dictionary form, form seen = the words as selected, no rows passed (the card renders rows instead of the gloss line), and a word's actions
- [ ] 5.2 The expression's gloss travels on the `Gesture` so `cardGloss` stores it instead of asking the single-lemma port, which cannot answer a key with a space
- [ ] 5.3 A match covering part of the selection takes the place of the rows of the words it covers, in reading order and within the same bound, and gives no row when the reader has settled it
- [ ] 5.4 Tests, one per scenario of the ADDED requirement — including the two "+ Deck" presses that must reach one card and the settled expression — plus the amended `Nothing worth showing` scenario of the MODIFIED one; the other seven scenarios it carries are already covered by `add-lingua-phrase-gloss`
- [ ] 5.5 `yarn lint`, `format:check`, `typecheck`, `test`, `build`, `check:variants`

## 6. On real pages

- [ ] 6.1 `yarn gen:pack:real`, then on Chrome macOS: `put up with`, `gave up`, `raining cats and dogs`, `starting point`, `in spite of`, an expression inside a longer selection, and "+ Deck" on `gave up` then on `give up` landing on one card
- [ ] 6.2 Record the real pack's size against the budget

## 7. Close

- [ ] 7.1 `SOURCES.md`: the expression table's row beside the others — same kaikki source, same licence, its reduction and its key rule
- [ ] 7.2 `README.md` of the extension: an expression is the card's answer and its key
- [ ] 7.3 `openspec validate add-lingua-expression-table --strict`, and archive only after `add-lingua-phrase-gloss`
