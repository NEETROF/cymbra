# Tasks

## 1. The measurement (scripts/lingua-data/measure)

- [ ] 1.1 A measurement script after change 42's: samples of each candidate neighbour's Wikipedia (Catalan, Galician, Latin, Sicilian, Neapolitan, Venetian, Sardinian, Friulian, Corsican, Occitan, Romanian), the Italian corpora (UD Italian-PUD, MarkIT, Tatoeba's Italian export, Italian Wikipedia articles, the two public-domain books of the corpus), blocks of twelve bytes or more; the share of each neighbour whichlang reads as Italian, with and without the guard, and the Italian blocks refused, each printed; verify the script runs offline on the pinned samples and its figures are recorded in the pull request (D2, D3).

## 2. The guard (crates/lingua-core)

- [ ] 2.1 `analysis/language.rs`: `italic_neighbour` — the two sorted Italian tables of D3 as the measurement settled them, a doc line per entry naming the neighbour, the words left out named with their Italian register; `guard_words` read as D4 says, the Neapolitan elided articles looked up with their apostrophe; `detect`'s Italian arm (D5); unit tests: each spec scenario's paragraph, one sentence per kept neighbour, every kept marker counted once, the Italian-naming sentence, capitals, the apostrophe, an NFD block, the leak; the French and Spanish guards' tests unchanged; `cargo test -p lingua-core` passes.
- [ ] 2.2 `analysis/mod.rs`: `ITALIAN_ANALYZER_VERSION = "1.1.0"`, its doc saying what `1.1.0` adds (D6); `apps/lingua-extension/test/packs.spec.ts`: `coreAnalyzerVersion("it", modRs)` reads `1.1.0`; `yarn test test/packs.spec.ts` passes.
- [ ] 2.3 `cargo llvm-cov -p lingua-core` keeps the crate's line coverage at or above its current figure; the guard's cost timed on the Italian corpora's blocks, native and WebAssembly, recorded in the pull request (D5).

## 3. The Italian baseline (crates/lingua-wasm, scripts/lingua-data/testdata)

- [ ] 3.1 `scripts/lingua-data/testdata/it-fr/manifest.json`: `analyzer_version` at `1.1.0`.
- [ ] 3.2 `tests/italian_baseline.rs`: the `misto` page's analysis holds tokens of its Italian blocks only, its Catalan, Galician and Latin blocks named as excluded; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`, `tests/baseline/it-fr.golden` committed; the pull request shows the diff is the `misto` page's and the version line's (D7); verify the test passes without re-blessing afterwards.

## 4. Owner

- [ ] 4.1 [manual] The owner reads the measurement's figures and settles which neighbours are kept (Occitan and Romanian in particular), and reads the Italian blocks the guard refuses, if any.

## 5. Gates

- [ ] 5.1 English, Spanish and French do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline --test fr_es_baseline --test cross_native` passes without re-blessing, and `git diff --stat` shows no golden but `it-fr.golden` and no `pin.json` touched.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` passes; in `apps/lingua-extension`: `yarn gen:wasm`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 5.3 `openspec validate add-lingua-italian-detection-guard --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-detection-guard` names its three prerequisites; `docs/lingua/italian-programme.md` row 8's status names this change's pull requests.
