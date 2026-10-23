# Tasks

## 1. Italian's pre-pass (crates/lingua-core)

- [x] 1.1 `analysis/tokenize.rs`: Italian takes an arm of its own in `tokenize`, the language-neutral rules kept (segmentation, the hyphen run, the digit drop, the apostrophe mapping), no U+202F rule (D2); unit tests pin that English, Spanish and French arms are untouched (`cargo test -p lingua-core` passes with their tests unchanged).
- [x] 1.2 Elision (D3): the table of elided forms and the words they stand for, as one sorted constant with a doc line per entry; `'` and `’`; a following letter required; the read word capitalised on its own first letter; the rule rerun on the remainder; a hyphenated run's first piece goes through the elision rule before the compound rule, as `push_french_run` does; `diss'` read by the next word; tests: `l'anglo-americano` → `lo` + `anglo-americano`, `diss'io` → `dissi` + `io`, `diss'egli` → `disse` + `egli`, each entry, `L'uomo`, `l’amica`, `c'è`, `un'amica`, `dov'è`, `cos'è`, `quand'ero`, `dev'essere`, `cent'anni`, `nessun'altra`, `quest'anno`, `anch'io`, `senz'altro`, `dall'altro`, a form outside the list kept whole, and an elided form on its own (`l’`) read as the one token `lo` spanning the apostrophe.
- [x] 1.3 The articulated prepositions (D4): the `di` family whole, `dell'` read as `dello`; the others split into preposition + article sharing the span, the elided ones with `lo`; `collo`, `colla`, `cogli`, `colle`, `pel`, `pei` whole; the capital kept on the preposition; nothing split inside a hyphenated compound; tests for each written form and for `Al`, `NEL`, `Nel mezzo`, `all'alba`, `dall'altro`, and a word card on `al`.
- [x] 1.4 Truncation (D5): Italian's words seam re-attaches a trailing apostrophe only for a listed elided form written alone or a listed truncation; the table of eight; a truncation's read word is final, not re-read as an articulated preposition; `is_elided`'s doc widened to « an elided piece or a truncation » (D6); no re-attachment after an opening quote; tests (the lexicon listing `e`): `po'`, `va'`, `di'`, `Fa'`, `po’`, `l’` alone → `lo`; `perche'`, `e'`, « ‘ciao’ » → `perche`, `e`, `ciao`; « ‘due anni fa’ » → `fa`; `da'` → `dai` one token; `dai` → `da` + `i`.
- [x] 1.5 `analysis/mod.rs`: `ITALIAN_ANALYZER_VERSION = "0.2.0"`, its doc saying what `0.2.0` adds (D7); the tests that read `0.1.0` rewritten with the rule they now state (`language.rs`, `tokenize.rs`, `engine.rs`, `packs/pack.rs`), and change 5's one-token tests in `tokenize.rs` (« l'uomo » one token, « po' » as `po`) rewritten as pieces and `poco`.
- [x] 1.6 `crates/lingua-core/tests/italian_fixtures.rs`: at least 80 cases over a stand-in lexicon that lists `è`, `a`, `e`, `o` (the single-letter words the tokeniser drops when unlisted), grouped by rule — each elided form; the words that stay whole; each articulated form, split or whole; each truncation; capitals; the typographic apostrophe; hyphenated compounds; a word card's single word — each asserting the tokens' texts and spans; `cargo test -p lingua-core --test italian_fixtures` passes.

## 2. The Italian baseline (crates/lingua-wasm, scripts/lingua-data/testdata)

- [x] 2.1 `scripts/lingua-data/testdata/it-fr/`: `manifest.json` at `0.2.0`; `forms.tsv`, `freq.tsv` and `gloss.tsv` gain the pieces' words of D7; verify `cargo test -p lingua-pack --test pipeline_testdata` builds the fixture.
- [x] 2.2 `tests/support/italian.rs`: `PHRASES` gains « C'è un po' di pane », « all'alba » and « Nel mezzo del cammin »; `GRAMMAR` gains (`l’`, `lo`) and (`al`, `a`); the module doc says Italian has its pre-pass at `0.2.0`.
- [x] 2.3 `tests/italian_baseline.rs`: `what_the_baseline_shows_today` asserts the pieces (« l'uomo » `lo` + `uomo`, « del » whole, « al » `a` + `il`, « po' » `poco`, « dirmelo » one token, `0.2.0`), and `the_fixture_lists_every_word_the_pre_pass_writes` holds the fixture to every piece and read word of D3–D5, as French's does; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`, `tests/baseline/it-fr.golden` committed; the pull request lists what moved (D8) and what did not; verify the test passes without re-blessing afterwards.
- [x] 2.4 `tests/languages.rs` and `tests/languages_wasm.rs` where they assert Italian's version or an Italian token's pieces: `0.2.0`, `"surface":"Lo"` / `"surface":"uomo"`; `cargo test -p lingua-wasm` passes.

## 3. The extension (apps/lingua-extension)

- [x] 3.1 `test/packs.spec.ts`: `coreAnalyzerVersion("it", modRs)` reads `0.2.0`; `yarn test test/packs.spec.ts` passes; `yarn build` and `yarn check:variants` pass with no package change.

## 4. Owner

- [ ] 4.1 [manual] The owner reads D3's table of elided forms, D4's table of articulated prepositions and D5's truncations, and confirms the placements of D1 (enclitics in change 7, the apostrophe-accent spellings in change 10, the keys in change 12).

## 5. Gates

- [x] 5.1 English, Spanish and French do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline --test fr_es_baseline --test cross_native` passes without re-blessing, and `git diff --stat` shows no golden but `it-fr.golden` and no `pin.json` touched.
- [x] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` passes.
- [x] 5.3 `openspec validate add-lingua-italian-tokenisation --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-tokenisation` names `add-lingua-italian-baseline` alone; `docs/lingua/italian-programme.md` row 6's status names this change's pull requests.
