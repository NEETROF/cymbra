# Tasks

## 1. The measurement (scripts/lingua-data/measure)

- [ ] 1.1 A measurement script, committed with its corpus list: samples of each candidate neighbour's Wikipedia (Catalan, Galician, Venetian, Sicilian, Neapolitan, Sardinian, Corsican, Friulian, Latin, Occitan, Romanian), the Italian corpora (UD Italian-PUD, MarkIT, Tatoeba's Italian export, Italian Wikipedia articles, the full texts of *I promessi sposi* and *Pinocchio* from Liber Liber or Project Gutenberg) and `pages-it.txt`, blocks of twelve bytes or more; the share of each neighbour whichlang reads as Italian, with and without the guard, per neighbour and pooled; every Italian block refused, printed with its markers; verify the script runs offline on the pinned samples and its figures are recorded in the pull request (D2, D3, D6).

## 2. The guard (crates/lingua-core)

- [ ] 2.1 `analysis/language.rs`: `italic_neighbour` — the two sorted Italian tables of D3 as the measurement settled them, disjoint, within `LONGEST_MARKER_BYTES`, a doc line per entry naming the neighbour, the words left out named with their Italian reading; `guard_words_opened`, the second entry point handing the apostrophe-opened word (D4), `guard_words` untouched; `detect`'s Italian arm (D5); unit tests: each spec scenario's sentence asserted read as Italian by whichlang (`read_as_italian`) then refused or kept, one sentence per kept neighbour, every kept marker counted once, the sorted/disjoint/length test, the Italian-naming sentence, capitals, the apostrophe straight and typographic, an NFD block, the leak; the French and Spanish guards' tests unchanged, `the_french_guard_reads_words_as_its_doc_says` gaining « 'et' »; `cargo test -p lingua-core` passes.
- [ ] 2.2 `analysis/mod.rs`: `ITALIAN_ANALYZER_VERSION = "1.1.0"` (or the next minor if change 10 landed first), its doc saying what it adds (D7); `apps/lingua-extension/test/packs.spec.ts` matches Italian's version by its pattern; `yarn test test/packs.spec.ts` passes.
- [ ] 2.3 `cargo llvm-cov -p lingua-core` keeps the crate's line coverage at or above its current figure; the guard's cost timed on the Italian corpora's blocks, native and WebAssembly, recorded in the pull request (D5).

## 3. The Italian baseline (crates/lingua-wasm, scripts/lingua-data/testdata)

- [ ] 3.1 `scripts/lingua-data/testdata/it-fr/manifest.json`: `analyzer_version` at the new version.
- [ ] 3.2 `tests/italian_baseline.rs`: the `misto` page's analysis holds tokens of its Italian blocks only, its Catalan and Galician blocks named as excluded (its Latin block too if the golden showed it read as Italian); re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`, `tests/baseline/it-fr.golden` committed; the pull request shows the diff is the `misto` page's and the version line's (D8); verify the test passes without re-blessing afterwards.

## 4. Owner

- [ ] 4.1 [manual] The owner reads the measurement's figures — each neighbour's share read as Italian, the words kept and left out, the Italian blocks refused with their reasons — and confirms or moves D2's defaults (Latin, Occitan and Romanian out) before the implementation merges.

## 5. Gates

- [ ] 5.1 English, Spanish and French do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline --test fr_es_baseline --test cross_native` passes without re-blessing, and `git diff --stat` shows no golden but `it-fr.golden` and no `pin.json` touched.
- [ ] 5.2 The recall gate (D6): at most one Italian block in a thousand refused across the corpora, none of `pages-it.txt`; the refused blocks listed in the pull request.
- [ ] 5.3 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` passes; in `apps/lingua-extension`: `yarn gen:wasm`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 5.4 `openspec validate add-lingua-italian-detection-guard --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-detection-guard` names its three prerequisites; `docs/lingua/italian-programme.md` row 8's status names this change's pull requests and its text and the *Detection* bullet name the neighbours as D2 settles them.
