# Tasks

## 1. The core (crates/lingua-core)

- [ ] 1.1 `engine.rs`: `ITALIAN_KEY_WRITTEN` (41 words, sorted, a test holding the sort) and `expression_piece`'s Italian arm (D2); `ITALIAN_EXPRESSION_WINDOW = 7` and `expression_window`'s arm (D3); tests: `al dente` → `a il dente`, `alla romana` → `a la romano`, `fare l'amore` → `fare lo amore`, `amico del cuore` → itself, `d'accordo` → `di accordo`, a nine-token headword → `None`, `perché` → `None`.
- [ ] 1.2 `engine.rs`: `whole_words()` includes Italian (D5); `expression_for_run` retries the `di` family as `di` for an Italian run (D5), the match covering the piece; tests « vicino al mare », « a causa del ritardo », « al posto dell'altro ».
- [ ] 1.3 `engine.rs`: `holds_french_contraction` becomes `holds_contraction(name, studied)` with Italian's split families — change 6's tables as refine-lingua-italian-reading leaves them: `coll'` among the elided prepositions, and the truncated articulated prepositions `a'`, `co'`, `da'`, `ne'`, `su'`, a `da'` read as the verb `dai` counted as a truncation's one token, not a contraction's two; `meets_as_written` reads an Italian name through Italian's pre-pass (D6); tests: « fare l'amore » / « fare lo amore », « d'un tratto » / « di un tratto », « andare al mare » / « andare a il mare », « anch'io » / « anche io », « un po' di pane » / « un poco di pane » with both `un po'` and `un poco` in the pack (each named as written: the last piece in full meets an elision, never a truncation), the last piece in full meeting an elided word.
- [ ] 1.4 `engine.rs`: `ExpressionScan::run`'s French gates become `as_written()`, true for French and Italian, so Italian takes the as-written arm of `written_as_the_page` (no clitic guard) (D4, D6); 44f's G2 guard gated on `whole_words()` with « che » as Italian's word (D7); tests: « vale la pena » named `vale la pena` and « valeva la pena » `valere la pena`; an Italian run with no grammar never gives its last word.
- [ ] 1.5 `cargo test -p lingua-core` passes; `cargo llvm-cov -p lingua-core` keeps the crate's line coverage at or above its current figure.

## 2. The builder (crates/lingua-pack)

- [ ] 2.1 `lib.rs`: the names section, `expr.others` and French's order among headwords written for an Italian pack (the `read` flag and the inline `others` test of `lib.rs` become French, Spanish, Italian) (D1, D4); the English arm untouched; tests over the it-fr fixture: the names present, `valere la pena`/`vale la pena` both kept under one key; `an_english_pack_carries_no_names` holds as it is.
- [ ] 2.2 `tests/committed_tables.rs`: the six committed packs build to their pins (nothing moves).

## 3. Review (crates/lingua-wasm)

- [ ] 3.1 `src/lib.rs` `pack_gloss`: the Italian arm (D8); a test: a card `al dente` glossed in French, reviewed by an English-native engine over an it-en fixture keyed `a il dente`, shows it-en's gloss; a card on the headword that does not win the key shows its own gloss.

## 4. The fixture and the golden

- [ ] 4.1 `scripts/lingua-data/testdata/it-fr/`: `mwe.tsv` and `forms.tsv` gain D9's rows, `manifest.json`'s `pack_version` moves; `tests/support/italian.rs`: the probes of D9 added; re-bless once with `LINGUA_BLESS=1 cargo test -p lingua-wasm --test italian_baseline`; the pull request lists each moved probe and names the baseline requirement's reason; verify the test passes without re-blessing afterwards.
- [ ] 4.2 The measurements of D2–D6 re-run with the real builder over the fixture and recorded in the pull request; the full figures are change 16's.

## 5. Gates

- [ ] 5.1 The six pairs do not move: the six baselines pass without re-blessing; `git diff --stat` shows no pin touched.
- [ ] 5.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test`, `yarn build`, `yarn check:variants` (no package change).
- [ ] 5.3 `openspec validate add-lingua-italian-expression-keys --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-expression-keys` names its prerequisites (change 16 added to `archiveAfter` once proposed); `docs/lingua/italian-programme.md` row 12's status names this change's pull requests.

## 6. Owner

- [ ] 6.1 [manual] The owner reads the re-blessed golden's diff (the pack line and the phrase probes) and the `di` retry's measured misreadings, and approves the re-bless before the implementation merges.
