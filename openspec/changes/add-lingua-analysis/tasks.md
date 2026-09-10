# Tasks — add-lingua-analysis

## 1. Monorepo foundations

- [x] 1.1 Declare the `lingua-*` domain in `openspec/config.yaml` (prefix table + per-artifact rules if needed)
- [x] 1.2 Create `crates/lingua-core` (lib) in the root Cargo workspace; dependencies `unicode-segmentation`, `fst`, `whichlang`, `serde`; module layout `analysis/`, `knowledge/`, `decks/`, `packs/`
- [x] 1.3 Wire the crate into the Rust CI lane (fmt/clippy/llvm-cov) and add the wasm-bindgen glue exclusion to `--ignore-filename-regex` (the CI lane **and** the command documented in CLAUDE.md) — the logic stays host-tested, only the bindings are excluded

## 2. Analysis pipeline (spec lingua-analysis)

_This change tests against synthetic mini-fixtures (test FST/frequency tables); the real pack arrives with `add-lingua-data-pack`._

- [x] 2.1 UAX #29 tokenisation + English pre-pass (contractions, edge apostrophes, single-letter words from the lexicon) with tests
- [x] 2.2 FST forms→lemmas format: read from a slice (`include_bytes!`-compatible), lookup API, test mini-FST
- [x] 2.3 Lemmatisation cascade: irregulars → FST → morphy fallback → out-of-lexicon plural fallback → identity; non-regression fixtures (min. 100 cases, including `endeavors→endeavor`, `bigger→big`, `went→go`)
- [x] 2.4 Per-block language detection (`whichlang`) + the "page not analysable" rule; mixed FR/EN tests
- [x] 2.5 Known-token percentage computed from classifications supplied as input (occurrences, ignored=known, learning=unknown, out-of-lexicon proper nouns excluded) — integration with the real statuses arrives with `add-lingua-knowledge-model`
- [x] 2.6 `analyzer_version` exposed + a determinism test over a fixture corpus
