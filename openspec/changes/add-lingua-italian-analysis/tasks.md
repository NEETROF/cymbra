# Tasks

## 1. NFC in Italian's pre-pass (crates/lingua-core)

- [x] 1.1 `analysis/tokenize.rs`: `nfc_for` answers NFC for Italian; the elided forms, the articulated prepositions, the truncations and a run the pack lists are compared composed; tests: a decomposed « città », « l'età », « dall'età » and a decomposed run the pack lists; English keeps its text as it came; `cargo test -p lingua-core` passes.

## 2. The cascade (crates/lingua-core)

- [x] 2.1 `analysis/italian.rs` (new, beside `spanish.rs` and `french.rs`): `lemmatize` in the order of D2 — the pack's forms; the enclitic rule (the combined clitics then the singles, longest first; a base accepted as an infinitive with its `-e`/`-re` restored, a gerund, an imperative the lexicon files under a verb, one of the five monosyllabic imperatives with the doubling undone and read by the table of five, or `ecco`; one split only); the unlisted plural with its guards; the form; `lemmatize.rs` dispatches to it; the module doc states the contract and the rules measured and dropped (D6). *Done as D6's measurement decided (design, *Measured at implementation*): the unlisted-plural step was measured out and dropped, a regular past participle added to the enclitic rule's bases; the order is the pack's forms, the enclitic rule, the form.*
- [x] 2.2 Unit tests in `italian.rs`: a listed form wins; `parlarmi`, `dirlo`, `porlo`, `condurlo` → their infinitives; `dirmelo`, `dargliela`, `andarsene`, `vedermene` → theirs; `parlandogli`, `scrivimi`, `prendetelo`, `andiamocene` → theirs; `dammi`, `fallo`, `vattene`, `dimmi`, `stacci`, `digli`, `fagli` → `dare`, `fare`, `andare`, `dire`, `stare`, `dire`, `fare`; `eccolo`, `eccomi` → `ecco`; `cinema`, `sistema`, `tema`, `vela` listed are never split; `zorbarmi` with no `zorbare` stays; `tavoli` → `tavolo`, `amiche` → `amica`, `botteghe` → `bottega`, `fuochi` → `fuoco`, `laghi` → `lago`; `vite` with `vita` listed stays `vite`; `porte` filed under `porta` is `porta`; `andai`, `tre`, `l'ora` left alone; the result lowercase and in NFC. *`tavoli`, `amiche`, `botteghe`, `fuochi`, `laghi` stay as written (the plural step measured out), and a past participle with a clitic is tested (`sviluppatasi` → `sviluppare`).*
- [x] 2.3 `cargo llvm-cov -p lingua-core` keeps the crate's line coverage at or above its current figure; `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check` pass.

## 3. Italian's closed classes (crates/lingua-core)

- [x] 3.1 `analysis/function_words.rs`: six sorted Italian tables (D3) — determiners, pronouns (the clitics and their combined forms included), prepositions, conjunctions, auxiliaries and modals, negation — behind `is_function_word` and `is_preposition` for Italian; the quantifiers among the determiners; the words left out (`volere`, `secondo`, `lungo`, `mai`, `più`, `stare`…) named in a doc comment with their reason. *The pronouns hold the `glie-` combined clitics only — `mela`, `vela`, `tela`, `velo`, `cene` are nouns standing alone —; `prima` and `salvo` left out of the prepositions by their counts, `stare` kept out; `dove`, `fino`, `nonostante`, `tramite`, `malgrado`, `eccetto`, `qualsiasi`, `qualunque`, `colui`, `colei`, `coloro`, `costui`, `costei`, `costoro` added at review by the same two thirds (design, *Measured at implementation*).*
- [x] 3.2 Tests: every Italian table sorted, so free of duplicates; « Non ho visto il cane di Marco », « non lo so », « né oggi né mai », « più », « mai », « sto parlando », « voglio », « il secondo giorno », « molto tempo », « al mare », « l'uomo » answer as the spec's scenarios say; `the` as Italian and `il` as English, Spanish and French are not function words; the English, Spanish and French tables' tests unchanged. *`il` is French's « he »: the cross-language check reads `di` for French.*

## 4. An Italian document's names (crates/lingua-core)

- [x] 4.1 `engine.rs`: `document_names` gains Italian's arm (D4) — Spanish's base rule, a capital after an elided piece counting as mid-sentence, a hyphenated run the pack does not list as one form — and change 5's early return goes; tests: « Sono andato a Roma. Roma è grande. », « L'Italia è lunga », « dell'Europa », « con Anna-Maria Rossi », « a Firenze, Marco legge » (`marco` a dictionary word), « Ecco. Ecco la casa. », « il Sale e il sale »; the Spanish and French names tests unchanged.

## 5. Italian's version (crates/lingua-core, apps/lingua-extension)

- [x] 5.1 `analysis/mod.rs`: `ITALIAN_ANALYZER_VERSION = "1.0.0"`, its doc saying what `1.0.0` adds (NFC, the cascade and its enclitic rule, the closed classes, the names rule) (D5); the tests that read `0.2.0` rewritten with the rule they now state; `apps/lingua-extension/test/packs.spec.ts`: `coreAnalyzerVersion("it", modRs)` matches a version at or above `1.0.0` (a pattern, as French's, so later bumps pass); `yarn test test/packs.spec.ts` passes.

## 6. Italian fixtures (crates/lingua-core)

- [x] 6.1 `crates/lingua-core/tests/italian_fixtures.rs` gains at least 70 cases over its stand-in lexicon, grouped by rule — NFC (decomposed elisions, articulated prepositions and plain words); the enclitic rule (each base shape, the combined clitics, the five monosyllabic imperatives with and without doubling, `ecco`, listed words never split, a base the lexicon does not list); the unlisted plural and its guards; the closed classes in a phrase; the names rule over a page — each asserting lemmas, function-word flags and names; `cargo test -p lingua-core --test italian_fixtures` passes.

## 7. The Italian baseline (crates/lingua-wasm, scripts/lingua-data/testdata)

- [x] 7.1 `scripts/lingua-data/testdata/it-fr/`: `manifest.json` at `1.0.0`; `forms.tsv` gains the verbs and forms D7 names (`dire`, `dare`, `andare`, `fare`, `parlare`, `scrivere`, `prendere`, their imperatives and gerunds, `ecco`); `gloss.tsv` their glosses; verify `cargo test -p lingua-pack --test pipeline_testdata` builds it.
- [x] 7.2 `tests/support/italian.rs`: `PHRASES` gains « Non lo so, dimmelo tu » and « Eccolo, è arrivato dall'Italia »; `GRAMMAR` gains (`dirmelo`, `dire`) and (`vattene`, `andare`); the module doc says Italian has its analysis at `1.0.0`. *(`dirmelo`, `dire`) was already a pair.*
- [x] 7.3 `tests/italian_baseline.rs`: `what_the_baseline_shows_today` asserts the clitic resolutions, the NFD block's words resolved, the function words dropped from the phrase probes, the names of `nomi` set aside, Italian's own version; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`, `tests/baseline/it-fr.golden` committed; the pull request lists what moved (D7) and what did not; verify the test passes without re-blessing afterwards.
- [x] 7.4 `tests/languages.rs` and `tests/languages_wasm.rs` where they assert Italian's version: the constant, no longer `0.x`; `cargo test -p lingua-wasm` passes.

## 8. Measurement

- [x] 8.1 `scripts/lingua-data/measure/it-ud.sh` after `fr-ud.sh`: the cascade's lemmas against UD Italian-PUD and MarkIT's test section at their pinned commits, the three figures printed; run once change 9's tables are on `main` and its figures recorded in that pull request; until then, D2's candidate rules measured on PUD and MarkIT with a pack built from a prototype of change 9's forms tables, `prima` and `stare` counted on MarkIT, the figures and the rules kept or dropped recorded in this change's pull request and in `italian.rs`'s doc; without such a prototype, the candidate rules are decided at change 9 (D6). *`it-ud.sh` committed, refusing to run until change 9's tables are; the candidate rules measured on a prototype of change 9's tables (`LINGUA_IT_PACK`), figures in `italian.rs`'s doc, the design and the pull request.*

## 9. Owner

- [x] 9.1 [manual] The owner reads D3's tables and their left-out words (`volere`, `secondo`, `lungo`, `stare`, `mai`, `più`, `anche`), D4's two Italian readings of the names rule, and D2's candidate rules kept or dropped, and may move a word before the golden is blessed. *Done 2026-10-24: read in an Italian-specialist review commissioned by the owner on 2026-10-24; approved, corrections in `refine-lingua-italian-reading` — D3's tables and left-out words approved, `stare` staying out (« sto bene », never « sono bene »), with fifteen words added (its D6); D4's names rule and D2's choices approved as they stand.*

## 10. Gates

- [x] 10.1 English, Spanish and French do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline --test fr_es_baseline --test cross_native` passes without re-blessing, and `git diff --stat` shows no golden but `it-fr.golden` and no `pin.json` touched.
- [x] 10.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"` passes; in `apps/lingua-extension`: `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test`, `yarn build`, `yarn check:variants`.
- [x] 10.3 `openspec validate add-lingua-italian-analysis --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-analysis` names its prerequisites; `docs/lingua/italian-programme.md` row 7's status names this change's pull requests.
