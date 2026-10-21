# Tasks

## 1. The variant and its seams (crates/lingua-core)

- [ ] 1.1 `analysis/language.rs`: `StudiedLanguage::Italian` appended after `French`, `ALL` of four, `tag()` `it`, `from_tag("it")`, `analyzer_version()`, whichlang's Italian class; `analysis/mod.rs`: `ITALIAN_ANALYZER_VERSION = "0.1.0"` with a doc comment naming what the baseline lacks; tests: the round trip of the four tags, Italian detected on an Italian paragraph and excluded for a learner of English, Spanish or French; `cargo test -p lingua-core` passes.
- [ ] 1.2 `analysis/tokenize.rs`: `nfc_for` returns Italian text as it came; `split_contraction` answers `None` for Italian (D2). Tests: « l'uomo », « un'amica », « dell'acqua », « c'è », « dirmelo », « po' » and « del » are one token each as Italian, « l'homme » and « don't » too.
- [ ] 1.3 `analysis/lemmatize.rs`: Italian takes `lemmatize_baseline` (D2). Tests: `as`, `has`, `dió` and `été` lemmatised as Italian come back as themselves; an Italian form the pack lists comes back as its lemma.
- [ ] 1.4 `analysis/function_words.rs`: Italian has no table (D2). Test: `non`, `di`, `il`, `che` are not function words as Italian; `the` as English, `de` as Spanish and `pas` as French answer as before.
- [ ] 1.5 `engine.rs`: `document_names` answers none for Italian (D2). Test: a page analysed as Italian over a synthetic Italian pack keeps its names as ordinary tokens and reports `0.1.0`.
- [ ] 1.6 `knowledge/profile.rs`: no native language maps to Italian (D3); test: a backup whose profile is French native, Italian studied is accepted, and no `NativeLanguage` answers Italian for `studied()`.
- [ ] 1.7 `packs/pack.rs`: an `it` pack at `0.1.0` loads and reports Italian; one at French's version is refused as `IncompatibleAnalyzer`; a pack studying `it` glossed in `it` is refused as `UnknownNative` (D3); the tests say so.
- [ ] 1.8 `cargo llvm-cov -p lingua-core` keeps the crate's line coverage at or above its current figure; `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check` pass.

## 2. Backup schema version 4 (crates/lingua-core, crates/lingua-wasm)

- [ ] 2.1 `decks/backup.rs`: `BACKUP_SCHEMA_VERSION = 4`; `backup_version()` answers 4 when Italian is among the profile's studied languages or among the languages the knowledge, exposures or deck hold, else as today; `from_backup` reads 1 to 4 and refuses 5 naming 4; the module doc lists version 4 (D7). `tests/backup_format.rs`: `fixtures/backup-v4-italian.json` pinned, the language-name rotation covers Italian, and the three older fixtures are written as before; `cargo test -p lingua-core --test backup_format` passes.
- [ ] 2.2 `crates/lingua-wasm/tests/languages.rs`: an engine built from the real en-fr pack with the it-fr fixture added (the Italian pack glossed in French) analyses an Italian page, backs up as version 4, and refuses an `it` pack glossed in English as another native; `cargo test -p lingua-wasm --test languages` passes.

## 3. The Italian invariance baseline (crates/lingua-wasm, scripts/lingua-data/testdata)

- [ ] 3.1 `scripts/lingua-data/testdata/it-fr/`: `forms.tsv` (about two hundred form→lemma pairs over the corpus's common words, the single-letter words `a`, `e`, `è`, `o` among them), `freq.tsv` (about a hundred ranks), `gloss.tsv` (about sixty French glosses, `città` « ville » among them), `mwe.tsv` (a handful), `level.tsv` (53 levels), `tags.tsv` (`NOUN`, change 4's convention), `manifest.json` (studies `it`, glossed in `fr`, `ITALIAN_ANALYZER_VERSION`, a « fixture » pack version, no `levels_estimated`), `NOTICE`; verify `cargo test -p lingua-pack --test pipeline_testdata` builds it and names its pair.
- [ ] 3.2 `crates/lingua-wasm/tests/baseline/pages-it.txt`: the thirteen pages of D5, in order — `notizie`, `narrativa`, `manzoni`, `omografi`, `elisioni`, `preposizioni`, `clitici`, `nomi`, `informale`, `ricetta`, `tecnico`, `misto`, `grammatica` — with the two excerpts, the real punctuation and the NFD block; `tests/support/italian.rs` holds the scenario (its corpus, probes and reader) and the excerpts' attribution in its doc comment; verify the corpus scenario test reads thirteen pages and finds the NFD block decomposed.
- [ ] 3.3 `tests/italian_baseline.rs`: `italian_output_has_not_moved` (the golden, with the re-bless message naming this change), `the_corpus_reads_as_expected`, `what_the_baseline_shows_today` (« l'uomo » one token with its lemma, « del » whole, « dirmelo » one token, no function word, the Catalan and Galician blocks of `misto` analysed as Italian, version `0.1.0`), `the_readers_backup_is_version_4`; `baseline/it-fr.golden` blessed once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; verify the test passes without re-blessing afterwards.
- [ ] 3.4 CI: `.github/workflows/lingua-extension-check.yml` runs `--test italian_baseline` with the others (the step renamed to name seven), and `.github/workflows/lingua-pack-update.yml` re-blesses it with them; verify on this change's pull request run.

## 4. The extension and the agent

- [ ] 4.1 `apps/lingua-extension/tool/packs.mjs`: `ANALYZER_CONSTANTS` gains `it: "ITALIAN_ANALYZER_VERSION"` (D4); verify `yarn test test/packs.spec.ts` passes (the hold to `language.rs`) and `yarn build` and `yarn check:variants` pass with no package change.
- [ ] 4.2 `apps/lingua-agent/rust/src/vocab.rs`: `language_name(Italian)` is « Italien » (D8); its test names the four; `cargo test -p lingua-agent` passes.

## 5. Owner

- [ ] 5.1 [manual] The owner reads `pages-it.txt` and the two excerpts' attribution in `support/italian.rs`, and may swap a page's text or an excerpt before the golden is blessed (task 3.3); a page swapped after the bless re-blesses with a line saying so.

## 6. Gates

- [ ] 6.1 English, Spanish and French do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline --test fr_es_baseline --test cross_native` passes without re-blessing, and `git diff --stat` shows no golden and no `pin.json` touched.
- [ ] 6.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` passes.
- [ ] 6.3 `openspec validate add-lingua-italian-baseline --strict` passes and `scripts/openspec_archive_order.py add-lingua-italian-baseline` names its four prerequisites; `docs/lingua/italian-programme.md` row 5's status names this change's pull requests.
