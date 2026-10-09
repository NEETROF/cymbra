# Tasks

## 1. The guard (crates/lingua-core)

- [x] 1.1 `analysis/language.rs`: `detect(trimmed, languages) -> Option<StudiedLanguage>` — whichlang's answer when it is one of `languages`, unless that language's guard refuses the block (none for English, `iberian_neighbour` for Spanish, unchanged, `romance_neighbour` for French); `block_is_studied` asks it with `[studied]`, `detect_document_language` gives a block's weight to the candidate it returns; Spanish's « Portuguese » sentinel goes (design D5). The two tests that call `detect` directly (`the_spanish_guard_leaves_french_detection_alone`, `the_guard_leaves_english_detection_alone`) are rewritten for the signature, their assertions kept; every other English and Spanish test passes unchanged.
- [x] 1.2 `analysis/language.rs`: `romance_neighbour` and `guard_words` as D4 reads a block (letters and combining marks, joined across `-`, U+00B7, U+2011 and `.` between letters; apostrophes split; a run holding a digit dropped; NFC before lookup; a neighbour marker counted in lowercase only; words over `LONGEST_MARKER_BYTES` skipped), and the two tables of D3 (152 neighbour markers, 69 French), sorted, with a doc comment naming the measurement (design, Measurement) and what was left out. Unit tests (D8):
  - the spec's scenarios, each sentence first asserted to be read as French by whichlang, and the corpus's Occitan block;
  - the tables sorted, disjoint, `LONGEST_MARKER_BYTES` their longest entry;
  - the word reading of `L'e-mail des étudiant·e·s, 2e cycle, XIIe — qu'i vient.`;
  - a decomposed Catalan block refused, a decomposed French one kept;
  - a capitalised marker not counted (« El Niño », « Los Angeles »);
  - a tie or no marker kept French, and « Les uns disent oui, les autres non. » kept;
  - `detect` answering English and Spanish as before for any candidates, and nothing for a Catalan block read as French among `[English, Spanish]`.
- [x] 1.3 French's analyser version bumped to `1.1.0` (D6), on top of change 41 — not before it: between 40 and 41 a bump contradicts change 40's `0.2.0`. `FRENCH_ANALYZER_VERSION` and its doc line in `analysis/mod.rs` (« … since add-lingua-french-detection-guard keeps Catalan, Occitan and Romanian blocks out »), and every test naming French's version: `language.rs`, `engine.rs`, `packs/pack.rs`, `crates/lingua-pack/src/lib.rs`, `crates/lingua-pack/tests/pipeline_testdata.rs`, `crates/lingua-wasm/tests/french_baseline.rs`, `crates/lingua-wasm/tests/languages.rs`. English and Spanish constants untouched.

## 2. The French baseline (crates/lingua-wasm, scripts/lingua-data/testdata)

- [x] 2.1 `scripts/lingua-data/testdata/fr-en/manifest.json`: `analyzer_version` re-stamped to 1.3's number.
- [x] 2.2 `tests/french_baseline.rs`: one assertion beside the others — the `mixte` page's analysis holds tokens of blocks 0 and 6 only (its English, Spanish, Catalan, Occitan and Italian blocks excluded). The corpus (`pages-fr.txt`) is not touched (D7).
- [ ] 2.3 `LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline`, once, on top of changes 40 and 41; check the diff is D7's — the pack line's and every `analyse` line's `analyzer_version`, and the `mixte` page's Occitan block leaving `analyse new-reader mixte` and `analyse reader mixte` (on `main`: 46 → 29 counted, the reader's known 17 → 12, 37 → 41 %), nothing else — and say so in the pull request, with the figures as measured on that base.

## 3. The extension

- [x] 3.1 `apps/lingua-extension/test/packs.spec.ts`: `coreAnalyzerVersion("fr", modRs)` reads 1.3's number. `packs.json`, the types and the labels are not touched.

## 4. Owner

- [ ] 4.1 [manual] The owner settles the design's open questions before the pull request merges: Romanian kept in the guard (1), Spanish's Occitan leak left to a change of its own (2), Franco-Provençal left out (3), the order, after change 41 (4). Dropping Romanian removes its 32 words, its scenario and its test; adding the eight Arpitan words adds them to the table and one test.

## 5. Gates

- [x] 5.1 English and Spanish do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test cross_native` passes without re-blessing, and `git diff --stat origin/main -- crates/lingua-wasm/tests/baseline/en-fr.golden crates/lingua-wasm/tests/baseline/es-fr.golden crates/lingua-wasm/tests/baseline/es-en.golden crates/lingua-wasm/tests/baseline/en-es.golden scripts/lingua-data/tables` is empty (D9).
- [x] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p lingua-core -p lingua-pack -p lingua-wasm -p lingua-agent`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; `wasm-pack test --node crates/lingua-wasm`.
- [x] 5.3 In `apps/lingua-extension`: `yarn gen:wasm`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; from the repository root, `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes (nothing it reads moves).
- [x] 5.4 `openspec validate add-lingua-french-detection-guard --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-french-detection-guard` exits 10 naming the changes of its `archiveAfter` still open (`add-lingua-french-baseline`, `add-lingua-french-tokenisation`, `add-lingua-french-analysis`); change 42 is marked done in `docs/lingua/language-matrix-programme.md`.
