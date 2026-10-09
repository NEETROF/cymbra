# Tasks

## 1. The comparison (crates/lingua-core)

- [ ] 1.1 `analysis/language.rs`: `OCCITAN_MARKERS` — D2's 121 words, sorted by bytes for binary search, with a doc comment naming the measurement (design, Measurement) and what was left out and why —, the Spanish markers Occitan writes (`las`, `lo`, `los`, `sus`), Occitan's elisions (`qu'`, `m'`, `t'`), and `iberian_neighbour`'s third comparison (D1, D3, D4): an Occitan marker or elision counted only where the word holds no capital, against the Spanish markers outside the four; the block refused when Catalan's or Galician's count is higher than Spanish's, as today, or Occitan's higher than those. The word reading, Catalan's elisions and the three existing tables untouched; `detect`, `block_is_studied` and `detect_document_language` untouched (change 42's shape). `iberian_neighbour`'s and `detect`'s doc comments name Occitan. The merged-table shape of D5 is allowed; the tests do not change.
- [ ] 1.2 Unit tests in `language.rs` (D8):
  - the spec's scenarios, each sentence first asserted to be read as Spanish by whichlang;
  - `OCCITAN_MARKERS` sorted, disjoint from `SPANISH_MARKERS` and `CATALAN_MARKERS`; the four Spanish markers Occitan writes all in `SPANISH_MARKERS`;
  - « Los dròlles son totjorn dins lo jardin. » refused, its articles not defending it;
  - `qu'ei`, `m'agrada`, `t'agrada` counted, a capitalised `Qu'ei` not;
  - « Pas de la Casa », « Vielha e Mijaran », « Junts pel Sí » and « Mi mai cocinó yautía con salami frito. » kept; the leak sentence kept;
  - the Catalan and Galician sentences of the existing tests refused as before;
  - a page of Occitan blocks among English and Spanish giving Spanish no vote.
  Every existing test of `analysis::language` passes unchanged.
- [ ] 1.3 `SPANISH_ANALYZER_VERSION` = `1.3.0` (D6), its doc line ending « …; `1.3.0` since add-lingua-spanish-occitan-guard keeps Occitan blocks out »; `crates/lingua-wasm/tests/languages.rs`'s `"analyzer_version":"1.2.0"` literal follows. English's and French's constants untouched.

## 2. Packs and fixtures (scripts/lingua-data, crates/lingua-pack, apps/lingua-agent)

- [ ] 2.1 Re-reduce es-fr and es-en from their pinned sources (`scripts/lingua-data/build.sh --reduce es-fr` then `es-en`, or `lingua-pack-update`'s reduction): only `tables/es-fr/manifest.json` and `tables/es-en/manifest.json` (`analyzer_version`) and the two `pin.json` (the pack's sha256) move — the scratch measured es-fr `ce03a605…` and es-en `70030bf8…`, 2,190,188 and 2,567,804 bytes as before (D7). `tables/es/` and every other table byte for byte; `pack_version` unchanged.
- [ ] 2.2 `testdata/es-fr/manifest.json` and `testdata/es-en/manifest.json`: `analyzer_version` `1.3.0`. `crates/lingua-pack/tests/pipeline_testdata.rs`: the es-fr and es-en fixtures' recorded sha256 re-recorded (`ac75501f…`, `d63a846a…`, sizes 1,342 and 1,356 unchanged), its comment saying Spanish's analyser moved them; en-fr's and en-es's untouched.
- [ ] 2.3 `apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` rebuilt with `scripts/lingua-data/build.sh --testdata es-fr` (the es-fr fixture's bytes of 2.2); the agent's tests pass again.

## 3. The goldens (crates/lingua-wasm)

- [ ] 3.1 `LINGUA_BLESS=1 cargo test -p lingua-wasm --test spanish_baseline --test es_en_baseline --test french_baseline`, once; check the diff is D7's — `es-fr.golden` and `es-en.golden` on the `pack` line and the 17 `analyse` lines, `fr-en.golden` on its `beside es-en` line, each on `analyzer_version` alone — and say so in the pull request. If change 41b merged first, its lines are already there and do not move again.

## 4. The extension

- [ ] 4.1 Nothing to change: `packs.spec.ts` reads Spanish's version from `analysis/mod.rs`, and `yarn gen:pack` builds es-fr from the committed tables; its gates run (5.3).

## 5. Gates

- [ ] 5.1 en-fr and en-es do not move: `cargo test -p lingua-wasm --test english_baseline --test en_es_baseline --test cross_native` passes without re-blessing, and `git diff --stat origin/main -- crates/lingua-wasm/tests/baseline/en-fr.golden crates/lingua-wasm/tests/baseline/en-es.golden scripts/lingua-data/tables/en scripts/lingua-data/tables/es scripts/lingua-data/tables/fr scripts/lingua-data/tables/en-fr scripts/lingua-data/tables/en-es scripts/lingua-data/tables/fr-en` is empty (D9).
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p lingua-core -p lingua-pack -p lingua-wasm -p lingua-agent`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; `wasm-pack test --node crates/lingua-wasm`; `scripts/lingua-data/build.sh es-fr` and `es-en` build to their new pins.
- [ ] 5.3 In `apps/lingua-extension`: `yarn gen:wasm`, `yarn gen:pack`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; from the repository root, `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes.
- [ ] 5.4 `openspec validate add-lingua-spanish-occitan-guard --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-spanish-occitan-guard` exits 10 naming `add-lingua-french-detection-guard` alone; row 42b is marked done in `docs/lingua/language-matrix-programme.md`.

## 6. Owner

- [ ] 6.1 [manual] Approve es-fr's output moving — `es-fr.golden`'s 18 version lines, and es-en's and fr-en's with them — in the pull request, as the programme's rule *en-fr and es-fr output does not move* requires; and the re-reduced `tables/es-fr/` that ships.
- [ ] 6.2 [manual] Settle the design's open questions before the pull request merges: French words kept in the table (1), capitals holding `à`, `è`, `ò`, `ç` (2), Asturian and Aragonese (3), Galician's `da` and `das` (4). Dropping `pas`, `mon`, `quand` or `t'` removes them from the table and re-runs the measurement; adopting 2 adds one rule and one test; folding 4 in takes `da` (and `das`) out of `GALICIAN_MARKERS`, adds a test that « ¿Cuánto se da de propina en España? » is Spanish, and states it in the requirement.
- [ ] 6.3 [manual] After the merge, the next extension release carries the es-fr pack at `1.3.0`; where the agent plugin is installed, rebuild it and copy the new es-fr pack beside it (an agent at `1.3.0` skips a `1.2.0` pack).
