## 1. Vocabulary (core)

- [x] 1.1 `lingua-core`: add a `grammar` module with the tag type. It covers the 17 UPOS tags and the closed feature subset of design D1, and parses and prints UD `FEATS` notation (features sorted by name). It has two modes: a lenient parse that skips unknown features, and a strict parse that fails naming the unknown part of speech, feature or value.
- [x] 1.2 Host tests for the tag type:
  - round-trip every part of speech and every feature value;
  - lenient parse skips an unknown feature, strict parse names it;
  - the tags of the design's Romance table (D1) all parse and print back unchanged.
- [x] 1.3 Check each row of the Romance table in design D1 against the UD feature documentation for French, Spanish, Italian and Portuguese. Correct the table, not the format, where one differs, and note the source of each row in `SOURCES.md`.

## 2. Data pipeline

- [x] 2.1 `reduce-en-fr.py`: keep each ESDB derived form's slot and emit its tag per design D2. This covers `<v>` with 4 or 3 entries, `<m>`, `<n>`, `<n_v>`, `<aj>`/`<av>`/`<d>`, and an explicit table for `be`'s eight slots. Spelling alternatives share their slot's tag, and lesser variants stay out.
- [x] 2.2 kaikki's regular form-of links take their tag from the ending `regular_inflection` checks (-ing, -ed/-d, -s, -er/-est), never from the French gloss wording.
- [x] 2.3 Keep the readings of forms spelled like their dictionary form (`put` as its own past, `sheep` as its own plural). `forms.tsv` stays as it is.
- [x] 2.4 Mark each relation believable or not with `own_words`' `believable` test, extracted to a shared function, as the fourth column of `grammar.tsv` (design D2).
- [x] 2.5 `_join_senses` (single-word path only):
  - carry each sense's kaikki `pos`, mapped to UPOS by the table in design D4;
  - after picking, group the senses by part of speech, stably;
  - replace a `;` inside a sense by `,`;
  - join and cut as today, then count the surviving senses into runs.
- [x] 2.6 Emit `grammar.tsv` (`form<TAB>lemma<TAB>tag<TAB>other|-`) and `senses.tsv` (`lemma<TAB>tag:count…`), sorted and deterministic. Add both to `TABLE_FILES` in `build.sh` and to the tables README with their licence, and describe them in `SOURCES.md`.
- [x] 2.7 `build.sh --reduce` stamps `pack_version` `<snapshot>+<reducer sha256, 7 hex digits>`, and an update from live sources keeps the bare snapshot. Update `pack_sources.py`'s docstring, which says the snapshot *is* the pack version.
- [x] 2.8 Python tests:
  - one per ESDB slot shape, on the fixture lines the tests already hold (`go`, `lie`, `be`, `focus`, `learn`, `datum`);
  - the kaikki ending rule;
  - `put`'s own-past reading;
  - believability: `uses` does not name `us`, `leaves` names `leaf`;
  - grouping order: prep, particle, prep becomes prep, prep, particle;
  - the `;` → `,` rewrite;
  - runs counted after the cut;
  - `mwe.tsv`, `forms.tsv` and `freq.tsv` byte-identical before and after.
- [x] 2.9 Testdata (`scripts/lingua-data/testdata/en-fr/`):
  - add `grammar.tsv` and `senses.tsv` for its words, with at least an irregular past, a regular past with two readings, a form spelled like its dictionary form, and a form of two dictionary forms;
  - add one glossed word with senses of two parts of speech;
  - bump its `pack_version`.
- [x] 2.10 A Spanish test fixture (inline in the builder tests, not a shipped pair) carrying `dijéramos` (`decir`, `VERB|Mood=Sub|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin`), `leche` (`NOUN|Gender=Fem`) and `me` (`PRON|Case=Dat|Number=Sing|Person=1|PronType=Prs`).
- [x] 2.11 `pack_report.py`: report readings and parts of speech of senses added, removed and changed, next to lemmas, glosses, levels and expressions, with tests.

## 3. Pack builder

- [x] 3.1 `PackInputs` gains `readings` and `senses`. `inputs_from_dir` reads the optional `grammar.tsv` and `senses.tsv` as it reads `level.tsv` (absent → empty, no error). Every tag is parsed strictly, and the build fails naming a code outside the vocabulary.
- [x] 3.2 Resolve every form of `grammar.tsv` through `lingua_core::analysis::lemmatize` against the lexicon just built. File the readings under their own dictionary form, and the believable readings of other dictionary forms as "also" entries under the resolved one (design D3). Drop readings whose dictionary form is not in the lexicon.
- [x] 3.3 Write the three sections (`tags`, `paradigms.zst`, `senses.zst`):
  - forms are stored as suffix edits in Unicode scalar values;
  - the order is lemma id, then form, then tag;
  - a section is emitted only when non-empty;
  - a pack built from inputs without the two tables is byte-identical to one built before this change.
- [x] 3.4 Check every run of `senses.tsv` against its gloss, split on `; `. The build fails naming the word when the counts disagree, or when a run names a word with no gloss.
- [x] 3.5 Size budget: `BuildError::OverBudget` names the readings of the rarest dictionary forms when, with no expression table, the grammar is what pushes the pack over. Add a test that drives it over. The `senses` section never leaves without its glosses.
- [x] 3.6 Builder tests: the testdata and the Spanish fixture round-trip through `Pack::load`, the output is deterministic (built twice, compared), and the licence guard needs no new source (`sources` and NOTICE unchanged).

## 4. Core — reading and answering

- [x] 4.1 `packs::pack`: the section constants and their parse, keeping the decompressed blobs as bytes with their index and decoding per lookup. Tests:
  - a pack without the sections loads and answers no reading;
  - a pack carrying them loads on the current reader path unchanged;
  - an unknown feature in the pool is skipped, not refused.
- [x] 4.2 `engine::word_grammar(written, lemma, studied, pack)` and its JSON (design D5):
  - pieces through `tokenize` + `resolve_lemmas`;
  - the piece matching `lemma`, or the first;
  - readings from `lemma`'s paradigm;
  - other dictionary forms from its "also" entries;
  - senses split and grouped by the runs;
  - one untagged group when there are no runs or they disagree;
  - tags serialised from ordered maps.
- [x] 4.3 Host tests, one per scenario of the `lingua-analysis` delta (`went`, `walked`, `doesn't`, `leaves`, `can`, a pack without tables). Assert that `analyse_page` over the fixture corpus is byte-identical and `ANALYZER_VERSION` is still `1.1.0`.
- [x] 4.4 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p lingua-core -p lingua-pack`, coverage ≥ 80 %.

## 5. WASM, fixtures and parity

- [x] 5.1 `lingua-wasm` exports `wordGrammar(written, lemma) -> String`.
- [x] 5.2 Rebuild `crates/lingua-wasm/tests/fixtures/pack.lingua` from the testdata. Add `grammar_golden.json`, with its update variable, and a parity test comparing native and WASM answers over the fixture words.
- [x] 5.3 `git diff --exit-code` is clean on `golden.json` and `phrase_golden.json`, and the page parity test passes with no update variable: neither `analyse_page` nor the phrase gloss moved.
- [x] 5.4 `yarn gen:fixtures` regenerates `apps/lingua-extension/test/fixtures/en-fr.testdata.lingua`.

## 6. Real tables and the pack

- [x] 6.1 Re-reduce `tables/en-fr` from the pinned sources (`build.sh --reduce en-fr`). Commit `grammar.tsv`, `senses.tsv` and the regrouped `gloss.tsv`, with `pin.json`'s reducer sha256, pack sha256 and size, and the new `pack_version`.
- [ ] 6.2 Record in the PR:
  - the number of readings, other dictionary forms and runs;
  - the glosses reordered and the separators rewritten;
  - the size of each new section, against 1.54 MB and the 5 MiB budget;
  - the words the pack reads as another dictionary form, the 30 most frequent.
- [x] 6.3 Spot-check the real pack against the design's cases: `went`, `gone`, `walked`, `put`, `leaves`, `lives`, `bored`, `uses`, `born`, `data`, `better`, `could`, `can`, `run`, `to`.

## 7. Extension — the card

- [x] 7.1 `AnalyzerPort.wordGrammar` in `WasmAnalyzerPort`, `MessagingLinguaPort` and `rpc-host`, with the types mirrored in `analyzer/types.ts`, and a messaging test for the forwarding.
- [x] 7.2 `PageHit` gains `written`, the text of the token's source span. A contraction's two halves share it.
- [x] 7.3 `selection-card.ts`:
  - every word card asks `wordGrammar(written, lemma)`;
  - for a known or ignored word it replaces the `gloss(lemma)` call;
  - for a loose word it follows `phraseGloss`;
  - for a highlighted word, the card waits up to `GRAMMAR_WAIT_MS` (250) and is never drawn pending: it is drawn once, complete, with the answer, or past the bound with the page token's gloss;
  - a late answer is dropped by `request()`'s generation check.
- [x] 7.4 `reading/grammar-labels.ts`:
  - names for parts of speech, gender, number, person and degree;
  - the English verb-form names, keyed by studied language;
  - statement composition (two readings of one form in one statement, "peut aussi être …", pieces joined by `+`);
  - a tag it cannot name gives no text.
- [x] 7.5 `wordpopup.ts`:
  - a grammar block between the listen row and the gloss carries the readings, the other dictionary forms and the pieces, one line each; the `.seen` line keeps `forme vue : « … »` as today;
  - the gloss renders one line per group, with its heading in italics and no heading for `SYM`/`X`/`PUNCT`/untagged;
  - everything is built from text nodes, and the CSS goes in `styles/wordpopup.css`.
- [x] 7.6 Tests, one per scenario of the three ADDED `lingua-browser-extension` requirements:
  - `went`, `walked`, `put`, `leaves` (actions on `leave`), `don't`, no tables;
  - `can`, `put`, a gendered noun from a fake answer, a heading-less group, "+ Deck" storing the flat gloss;
  - timing: in-page engine never pending, awake engine completes once, cold engine past the bound completes without grammar and ignores the late answer, known word gets both in one answer.
- [x] 7.7 Extend `lint-lemma.spec.ts`, or add a sibling test, so no UD code (`VERB`, `Tense=`, …) can reach a rendered card.
- [x] 7.8 `yarn lint`, `format:check`, `typecheck`, `test`, `build`, `check:variants`.

## 8. Validation and dogfooding

- [x] 8.1 `openspec validate add-lingua-word-grammar --strict`.
- [ ] 8.2 Dogfood on Chrome desktop, Firefox desktop and Safari (macOS and iOS), on an English article:
  - open `went`, `leaves`, `don't`, `can` and a known word;
  - measure on Safari how often a first card misses the 250 ms bound with the event page suspended and awake, and record the result in the design (keep the bound, or open the prefetch follow-up).
