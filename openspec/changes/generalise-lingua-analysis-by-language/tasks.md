## 1. Language and versions

- [ ] 1.1 `analysis/language.rs`: append `StudiedLanguage::Spanish` (with a doc line: new variants go after the existing ones), `ALL`, `tag()` / `from_tag()` (ISO 639-1, exact), `analyzer_version()`, whichlang `Spa`. `analysis/mod.rs`: `SPANISH_ANALYZER_VERSION = "0.1.0"`, and `ANALYZER_VERSION` documented as English's. `knowledge/profile.rs` uses `tag()`. Tests, run with `cargo test -p lingua-core language`:
  - every variant round-trips through its tag, and `pt` and `EN` have no analyser;
  - English stays at `1.1.0` and Spanish's version differs;
  - `English < Spanish`, and English still serialises as `"English"`;
  - a Spanish block is studied by a Spanish learner only, and an English block is excluded for one.

## 2. Dispatch

- [ ] 2.1 `analysis/tokenize.rs`: the pre-pass takes a `contractions` flag (English true, Spanish false) through `push_word` and `push_compound`. Test: « don't » tokenised as Spanish is one token, and as English it is `do` + `not`; compounds, digits and single letters give the same tokens in both.
- [ ] 2.2 `analysis/function_words.rs`: the tables are selected by language, and Spanish has none. Test: no Spanish token is a function word, and the English tables answer as before.
- [ ] 2.3 `analysis/lemmatize.rs`: `lemmatize(form, studied, lexicon)` dispatches to `lemmatize_english` (the current cascade, moved verbatim) or `lemmatize_baseline` (the pack's forms, else the lowercased form). Its doc comment lists what the baseline drops (design D3). Tests:
  - `has`, `are`, `ate`, `more` and `mes`, lemmatised as Spanish, return themselves with a lexicon that does not list them, and the listed lemma with one that does;
  - the English cascade's tests are unchanged.
- [ ] 2.4 `analysis/pipeline.rs` and `engine.rs`: pass `studied` to `resolve_lemmas`, and set `PageAnalysis.analyzer_version` to `studied.analyzer_version()`. A page analysed as Spanish over a synthetic Spanish pack reports Spanish's version and no English lemma. `lemmatization_fixtures.rs`, `determinism.rs` and the engine and pipeline tests pass `English` and otherwise run unchanged. Run `cargo test -p lingua-core`.

## 3. Packs

- [ ] 3.1 `packs/pack.rs`: map `meta.studied` to a `StudiedLanguage` (`PackError::UnknownLanguage`, raised before any section is read), check compatibility against that language's version, and add `Pack::studied()`. Tests:
  - the en-fr test pack reports English at `1.1.0`;
  - a `pt` pack is refused with an error naming `pt`;
  - a Spanish pack at Spanish's version loads, and one at `1.1.0` is refused as `IncompatibleAnalyzer`.
- [ ] 3.2 `crates/lingua-pack`: map `meta.studied` (an unknown language is a `BuildError`) and pass the language to `lemmatize` for grammar paradigms and expression keys. `spec_scenario_a_romance_pack_fits_the_vocabulary` stamps Spanish's version. Test: a pack whose studied language the core cannot analyse is refused at build. Run `cargo test -p lingua-pack`.
- [ ] 3.3 `bash scripts/lingua-data/build.sh en-fr <out>` reproduces the sha256 in `pin.json`. No table, pin or manifest changes in the pull request.

## 4. Callers and readers

- [ ] 4.1 `cargo build --workspace` and `cargo test --workspace`: the wasm engine and the agent plugin compile and pass with no change of theirs. They keep passing English; `generalise-lingua-wasm-engine` owns the wasm constant.
- [ ] 4.2 `apps/lingua-extension/build.mjs`: anchor the version pattern (`\bANALYZER_VERSION`). Check with `node -e` on a source where `SPANISH_ANALYZER_VERSION` comes first: it returns `1.1.0`. `yarn build` passes.

## 5. Gates

- [ ] 5.1 The English invariance baseline (#622, on `main` first) passes without re-blessing: `cargo test -p lingua-wasm --test english_baseline` is green, and `git diff --stat origin/main -- crates/lingua-wasm/tests/baseline` is empty.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; `wasm-pack test --node crates/lingua-wasm`.
- [ ] 5.3 `openspec validate generalise-lingua-analysis-by-language --strict`; `python3 scripts/check_ci_units.py --list` is unchanged; change 5 is marked done in `docs/lingua/spanish-programme.md` (#621).
