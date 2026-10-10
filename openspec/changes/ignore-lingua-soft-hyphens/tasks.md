# Tasks

## 1. The core (crates/lingua-core)

- [ ] 1.1 `analysis/tokenize.rs`: `SOFT_HYPHEN` and `without_soft_hyphens(&str) -> Cow<str>`, borrowed when the text holds no U+00AD (D2). It is read in:
  - `push_word`, before the apostrophe is normalised;
  - `push_compound`, for the compound's text and its parts;
  - `french_lowercase`;
  - `push_french_elisions`'s look at the next word;
  - `is_french_inversion`'s pieces, the euphonic `t`'s next piece among them.

  Spans, segmentation and the hyphen run are untouched. The module's and `Token`'s doc comments say that a token's text is the word without its soft hyphens and its span the source's, as for NFC.
- [ ] 1.2 `analysis/language.rs`: `block_is_studied` and `detect_document_language` read each block through `without_soft_hyphens` before trimming it (D3). `detect`, `iberian_neighbour`, `romance_neighbour`, `guard_words` and the tables are untouched; their doc comments name the rule.
- [ ] 1.3 `engine.rs`: `word_grammar`'s fallback for a written word that yields no token reads it without soft hyphens (D4).
- [ ] 1.4 Unit tests (D8):
  - the spec's scenarios, each sentence asserted as written and with its soft hyphens. The Spanish and French lines are first read with each soft hyphen as a space: the detector still reads them as Spanish or French, and the guard refuses them, since its syllables are what today's reading counts;
  - `without_soft_hyphens` borrowing;
  - every span of a hyphenated sentence slicing its word with the hyphens;
  - one page per language from the committed corpora, hyphenated in the test at hand-chosen breaks, analysing, voting and glossing as its clean text, spans aside;
  - a soft hyphen opening a block, ending a word, or doubled.

  Every existing test passes, except the version literals of 1.5.
- [ ] 1.5 Versions (D6):
  - `ANALYZER_VERSION` = `1.2.0`, `SPANISH_ANALYZER_VERSION` = `1.4.0`, `FRENCH_ANALYZER_VERSION` = `1.2.0`, each doc comment ending « …; `x.y.0` since ignore-lingua-soft-hyphens reads a word without its soft hyphens »;
  - `english_keeps_its_analyser_version_and_spanish_has_its_own`, `spec_scenario_the_page_analysis_does_not_move` and `spec_scenario_loading_the_en_fr_pack_names_english` follow;
  - the comments of `pack.rs` and of lingua-pack's `lib.rs` that say « English's `1.1.0` is French's own » are reworded.

## 2. Packs and fixtures (scripts/lingua-data, crates/lingua-pack, crates/lingua-wasm, apps/lingua-agent, apps/lingua-extension)

- [ ] 2.1 Re-reduce en-fr, en-es, es-fr, es-en and fr-en from their pinned sources (`scripts/lingua-data/build.sh --reduce <pair> <out>`, or `lingua-pack-update`'s reduction). Only each `tables/<pair>/manifest.json` (`analyzer_version`) and `pin.json` (the pack's sha256) move (D7). On `eddaf712` the scratch measured these, sizes as before:
  - en-fr `582f7762…`;
  - en-es `a95af250…`;
  - es-fr `d617856d…`;
  - es-en `507abaae…`;
  - fr-en `d013a548…`.

  `tables/en/`, `tables/es/`, `tables/fr/` and every other table stay byte for byte, and `pack_version` is unchanged. If change 49 committed `tables/fr-es/` first, it is re-reduced too.
- [ ] 2.2 Fixtures:
  - `testdata/{en-fr,en-es,es-fr,es-en,fr-en}/manifest.json`: `analyzer_version` follows;
  - `crates/lingua-pack/tests/pipeline_testdata.rs` re-records the four fixtures' bytes (en-fr `ebb1f3c7…`, en-es `51066821…`, es-fr `652ab141…`, es-en `636e693e…`, sizes unchanged), its comment saying why;
  - `crates/lingua-wasm/tests/fixtures/pack.lingua` and `apps/lingua-extension/test/fixtures/en-fr.testdata.lingua` are rebuilt (`build.sh --testdata en-fr`, `yarn gen:fixtures`);
  - `crates/lingua-wasm/tests/fixtures/golden.json` is regenerated (`LINGUA_UPDATE_GOLDEN=1 cargo test -p lingua-wasm --test parity`), its version alone moving.
- [ ] 2.3 The agent's fixtures:
  - `apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` rebuilt with `build.sh --testdata es-fr`;
  - `pack.lingua` re-stamped at `1.2.0`, its metadata's version alone, through `read_container` / `write_container`.

  The agent's tests pass again.

## 3. The goldens (crates/lingua-wasm)

- [ ] 3.1 Run `LINGUA_BLESS=1 cargo test -p lingua-wasm --no-fail-fast --test english_baseline --test en_es_baseline --test spanish_baseline --test es_en_baseline --test french_baseline` once, then check the diff is D7's, the version string alone on each moved line:
  - `en-fr.golden` 16 lines;
  - `en-es.golden` 16;
  - `es-fr.golden` 19;
  - `es-en.golden` 18;
  - `fr-en.golden` 19.

  Say so in the pull request. `tests/languages.rs`'s two literals and `french_baseline.rs`'s « this core is 1.1.0 » follow.

## 4. The extension (apps/lingua-extension)

- [ ] 4.1 `src/reading/selection.ts` (D5):
  - `WORD_CHAR` gains U+00AD;
  - `collapse` drops U+00AD as it collapses whitespace, so `sentenceAndSelection`'s sentence and offsets are read without it;
  - `captureFrom`'s text is read without soft hyphens.
- [ ] 4.2 `src/reading/session.ts`: `pageHit`'s `written` is read without soft hyphens. Highlighting, hit-testing, `collectBlocks` and `byteToCharOffset` are untouched.
- [ ] 4.3 Tests (D8):
  - `selection.spec.ts`: the snap across U+00AD; `sentenceAndSelection` with soft hyphens before, inside and after the selection; a capture's text;
  - `reading-session.spec.ts`: a page hit's `written`, and a highlight over « vi&shy;da » covering the word through the fake engine.

## 5. Gates

- [ ] 5.1 en-fr and es-fr move on the version alone (3.1), and no table but the manifests and pins moves: `git diff --stat origin/main -- scripts/lingua-data/tables` lists only `manifest.json` and `pin.json` files.
- [ ] 5.2 The Rust gates:
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`;
  - `cargo test -p lingua-core -p lingua-pack -p lingua-wasm -p lingua-agent`;
  - `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`;
  - `wasm-pack test --node crates/lingua-wasm`;
  - `scripts/lingua-data/build.sh <pair> <out>` for the five pairs, each building to its new pin.
- [ ] 5.3 The extension's gates, in `apps/lingua-extension`: `yarn gen:wasm`, `yarn gen:pack`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`. Then, from the repository root, `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes.
- [ ] 5.4 The cost on text without soft hyphens stays within the 2 % the scratch measured natively. Time the page analysis over the committed corpora, native and WebAssembly, before and after, and say so in the pull request.
- [ ] 5.5 OpenSpec and the programme:
  - `openspec validate ignore-lingua-soft-hyphens --strict` passes;
  - `python3 scripts/openspec_archive_order.py ignore-lingua-soft-hyphens` exits 10 naming `add-lingua-french-analysis`, `add-lingua-french-detection-guard` and `add-lingua-spanish-occitan-guard`;
  - row 42d is marked done in `docs/lingua/language-matrix-programme.md`.

## 6. Owner

- [ ] 6.1 [manual] Approve en-fr's and es-fr's output moving, in the pull request, as the programme's rule *en-fr and es-fr output does not move* requires: `en-fr.golden`'s 16 and `es-fr.golden`'s 19 version lines (and en-es's, es-en's and fr-en's with them), and the re-reduced `tables/en-fr/` and `tables/es-fr/` that ship.
- [ ] 6.2 [manual] Settle the design's open questions before the pull request merges: words already saved with the hidden hyphen (1), and other invisible characters (2). Both defaults leave them as they are. Cleaning saved words is a change of its own. Reading U+2060 and U+FEFF as U+00AD adds them to `without_soft_hyphens`, with a test each.
- [ ] 6.3 [manual] After the merge, the next extension release carries every pack at its new version. Where the agent plugin is installed, rebuild it and copy the new packs beside it: an agent at the new versions skips the old packs.
