# Tasks

## 1. The rule (crates/lingua-core)

- [ ] 1.1 `analysis/language.rs`: `GALICIAN_MARKERS_SPANISH_WRITES` (`da`, `das`), with a doc comment naming Spanish's *gives* and *you give* and the measurement (design, Measurement); in `iberian_neighbour`, a word of `GALICIAN_MARKERS` that is one of the two counted apart, and added to Galician's count only when the block holds another Galician marker (D1). Both words stay in `GALICIAN_MARKERS` (32 words, sorted, unchanged). The word reading, Catalan's elisions, Occitan's lowercase rule, the four tables and Catalan's and Occitan's comparisons untouched (D2); `detect`, `block_is_studied` and `detect_document_language` untouched. `GALICIAN_MARKERS`' and `iberian_neighbour`'s doc comments name the rule.
- [ ] 1.2 Unit tests in `language.rs` (D6):
  - the spec's new scenarios, each sentence first asserted to be read as Spanish by whichlang;
  - `da` and `das` in `GALICIAN_MARKERS`;
  - « ¿Cuánto se da de propina en España? » and « ¿Te das cuenta de la hora que es? » kept, `iberian_neighbour` false on them;
  - « O profesor Smith é recoñecido por ser un dos máis grandes eruditos da filoloxía inglesa. » refused; `da xa por` refused where `xa por` ties;
  - « A esperanza é a razón da vida. » kept (the leak the requirement states);
  - « Me da igual: nunca jugué al polo. » refused (`da` beside `polo`);
  - the Catalan, Galician and Occitan sentences of the existing tests refused as before.
  Every existing test of `analysis::language` passes unchanged.
- [ ] 1.3 `SPANISH_ANALYZER_VERSION` = `1.4.0` (D4), its doc line ending « …; `1.4.0` since refine-lingua-spanish-galician-markers keeps Spanish's *da* and *das* from reading as Galician »; `crates/lingua-wasm/tests/languages.rs`'s `"analyzer_version":"1.3.0"` literal follows. English's and French's constants untouched.

## 2. Packs and fixtures (scripts/lingua-data, crates/lingua-pack, apps/lingua-agent)

- [ ] 2.1 Re-reduce es-fr and es-en from their pinned sources (`scripts/lingua-data/build.sh --reduce es-fr <out>` then `--reduce es-en <out>`, or `lingua-pack-update`'s reduction): only `tables/es-fr/manifest.json` and `tables/es-en/manifest.json` (`analyzer_version`) and the two `pin.json` (the pack's sha256) move — on today's `main` the scratch measured es-fr `d617856d…` and es-en `507abaae…`, 2,224,439 and 2,608,413 bytes as before (D5); on top of another change that moved the packs first, the sha256 differ and the sizes stay that change's. `tables/es/` and every other table byte for byte; `pack_version` unchanged.
- [ ] 2.2 `testdata/es-fr/manifest.json` and `testdata/es-en/manifest.json`: `analyzer_version` `1.4.0`. `crates/lingua-pack/tests/pipeline_testdata.rs`: the es-fr and es-en fixtures' recorded sha256 re-recorded (`652ab141…`, `636e693e…`, sizes 1,342 and 1,356 unchanged), its comment saying Spanish's analyser moved them; en-fr's and en-es's untouched.
- [ ] 2.3 `apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` rebuilt with `scripts/lingua-data/build.sh --testdata es-fr apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` (the es-fr fixture's bytes of 2.2); the agent's tests pass again.

## 3. The goldens (crates/lingua-wasm)

- [ ] 3.1 `LINGUA_BLESS=1 cargo test -p lingua-wasm --test spanish_baseline --test es_en_baseline --test french_baseline`, once; check the diff is D5's — `es-fr.golden` and `es-en.golden` on the `pack` line and the 17 `analyse` lines, `fr-en.golden` on its `beside es-en` line, each on `analyzer_version` alone — and say so in the pull request. If another change moved the same lines first, rebase and resolve the conflict by re-blessing, never by hand: only `analyzer_version` moves on its lines.

## 4. The extension

- [ ] 4.1 Nothing to change: `packs.spec.ts` reads Spanish's version from `analysis/mod.rs`, and `yarn gen:pack` builds es-fr from the committed tables; its gates run (5.3).

## 5. Gates

- [ ] 5.1 en-fr and en-es do not move: `cargo test -p lingua-wasm --test english_baseline --test en_es_baseline --test cross_native` passes without re-blessing, and `git diff --stat origin/main -- crates/lingua-wasm/tests/baseline/en-fr.golden crates/lingua-wasm/tests/baseline/en-es.golden scripts/lingua-data/tables/en scripts/lingua-data/tables/es scripts/lingua-data/tables/fr scripts/lingua-data/tables/en-fr scripts/lingua-data/tables/en-es scripts/lingua-data/tables/fr-en` is empty (D7).
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p lingua-core -p lingua-pack -p lingua-wasm -p lingua-agent`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; `wasm-pack test --node crates/lingua-wasm`; `scripts/lingua-data/build.sh es-fr <out>` and `build.sh es-en <out>` build to their new pins.
- [ ] 5.3 In `apps/lingua-extension`: `yarn gen:wasm`, `yarn gen:pack`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; from the repository root, `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"` passes.
- [ ] 5.4 `openspec validate refine-lingua-spanish-galician-markers --strict` passes; `python3 scripts/openspec_archive_order.py refine-lingua-spanish-galician-markers` exits 10 naming `add-lingua-spanish-occitan-guard` alone while it is open, 0 once it is archived; row 42c is marked done in `docs/lingua/language-matrix-programme.md`.

## 6. Owner

- [x] 6.1 [manual] Approved by the owner on 2026-10-10 (in session), before the implementation. Approve es-fr's output moving — `es-fr.golden`'s 18 version lines, and es-en's and fr-en's with them — in the pull request, as the programme's rule *en-fr and es-fr output does not move* requires; and the re-reduced `tables/es-fr/` that ships.
- [x] 6.2 [manual] Settled by the owner on 2026-10-10 (in session), each as its default: `da` and `das` count only beside another Galician word (1); Galician's `é` left out (2). Settle the design's open questions before the pull request merges: the rule for *da* and *das* (1), Galician's `é` (2). Choosing another rule of 1 changes 1.1 and the tests of 1.2 that name it, and the measurement is re-run; adding `é` puts it in `GALICIAN_MARKERS` (33 words, sorted), adds a test that « A esperanza é a razón da vida. » is then refused and that a 19th-century Spanish line writing the conjunction « é » beside Spanish words stays Spanish, re-runs the measurement, and turns the requirement's last scenario into one whose Galician line writes no `é`.
- [ ] 6.3 [manual] After the merge, the next extension release carries the es-fr pack at `1.4.0`; where the agent plugin is installed, rebuild it and copy the new es-fr pack beside it (an agent at `1.4.0` skips a `1.3.0` pack).
