# Tasks

## 0. Owner, before the implementation

- [x] 0.1 [manual] Settled by the owner on 2026-10-10 (in session), each as recommended: the last piece written in full meets either form (1); `c'est` on forms written in full accepted (2); contractions (`au fait` on « à le ») a small change of its own (3). The owner answers the design's open questions: the last piece written in full meets either form (1), `c'est` and the other forms written in full no longer answered (2), contractions left to a change of their own (3). Each answer that departs from the design's recommendation is written into the design, the spec and these tasks before 1.1.

## 1. The core (crates/lingua-core)

- [x] 1.1 `analysis/tokenize.rs`: `pub fn is_elided(text: &str, token: &Token) -> bool` — the token's source span ends on a straight or typographic apostrophe, which French's pre-pass gives an elided piece alone (design D2); its doc comment says why no other token's span can. Unit tests: « L’homme » → `Le` elided, `homme` not; « l’ homme » (written alone) elided; « jusqu'au » → `jusque` elided, `à` and `le` not; « aujourd'hui », « presqu'île », « vélib' » and an English « don't » → none elided.
- [x] 1.2 `engine.rs` `gloss_phrase`: one elision flag per token, from `is_elided` over the tokens it has just made, beside `shares_span`, handed to `match_expressions`. No `Token`, `PhraseToken` or JSON field (D2).
- [x] 1.3 `engine.rs` `match_expressions`: for a French run that matched a key, the headword's pieces — the name `Pack::expression_name` gives, else the key — are read by `headword_reading` when the name holds an apostrophe (one holding none is written in full), each piece's elision taken by `is_elided` on the name; the run is kept only when every piece meets (an elided piece an elided token, a piece in full a token in full, the last piece written in full either), else the next shorter run is tried. A name whose reading does not line up with the run is matched as today. English and Spanish runs untouched (D1, D3). The function's doc comment states the rule and names this change.
- [x] 1.4 Host tests in `engine.rs`, one per scenario of the delta, on an inline French pack holding `de l'`, `de un`, `un peu`, `parce que`, `qu'est-ce que` and `c'est`: « Il a décidé de le faire » none; « de l’eau » `de l'` over `de`, `le`; « d’un hiver » none; « d’un peu plus » `un peu`; « parce qu’il pleut » `parce que`; « Qu’est-ce qu’il attend » `qu'est-ce que`; « Ce sont mes amis » none and « C’était l’hiver » `c'est`. And: a French run whose name is unnamed and written in full (`de un`) is refused on « d’un » without reading it; an English and a Spanish selection gloss as before (`spanish_spans_for`, `french_and_english_matches_never_take_a_chain` unchanged).

## 2. The committed tables (crates/lingua-pack)

- [x] 2.1 `tests/committed_tables.rs`: for fr-en and fr-es, every expression whose winning headword reads, through `headword_reading`, with an elided piece is named by the pack (`Pack::expression_name`) — the invariant D3 reads the headword's elision from (measured: 1,541 and 869). No builder code changes; the six committed packs still build to their pins.

## 3. The French baseline (crates/lingua-wasm, apps/lingua-extension)

- [x] 3.1 `tests/support/french.rs`: `PHRASES` gains « de l’eau », « Il a décidé de le faire », « parce qu’il pleut » and « d’un hiver », after « à la maison » (D5).
- [x] 3.2 `tests/french_baseline.rs`: the doc comment names this change among those that add probes.
- [x] 3.3 Re-bless once: `LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline --test fr_es_baseline`. Check the diff is D5's — 8 lines added to each golden, none removed; over fr-en « de l’eau » `de l'`, « parce qu’il pleut » `parce que`, « Il a décidé de le faire » and « d’un hiver » none; over fr-es « parce qu’il pleut » `parce que`, the three others none — and say so in the pull request, with D1's figures re-measured on the implementation's base.
- [x] 3.4 `apps/lingua-extension`: `test/word-card-fr-en.spec.ts` and `test/word-card-fr-es.spec.ts` count 34 phrase probes (their comment: 33 phrases and the reader's); `yarn vitest run test/word-card-fr-en.spec.ts test/word-card-fr-es.spec.ts -u` adds the four probes' cards to `test/baseline/word-card-fr-en.txt` and `word-card-fr-es.txt`, every block before byte for byte.

## 4. Gates

- [x] 4.1 English and Spanish do not move: `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test cross_native --test parity --test card_gloss_language --test spanish_expression_keys` passes without re-blessing; `cargo test -p lingua-pack --test committed_tables` builds the six committed packs to their pins; `git diff --stat origin/main -- crates/lingua-wasm/tests/baseline/en-fr.golden crates/lingua-wasm/tests/baseline/es-fr.golden crates/lingua-wasm/tests/baseline/es-en.golden crates/lingua-wasm/tests/baseline/en-es.golden scripts/lingua-data` is empty (D6).
- [x] 4.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p lingua-core -p lingua-pack -p lingua-wasm`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; `wasm-pack test --node crates/lingua-wasm`; `cargo test` in `apps/lingua-agent/rust` (it compiles the core).
- [x] 4.3 In `apps/lingua-extension`: `yarn gen:wasm`, `yarn typecheck`, `yarn lint`, `yarn test`, `yarn build`, `yarn check:variants`; the bundles grow by the check alone.
- [x] 4.4 `openspec validate match-lingua-french-elided-pieces --strict` passes; `python3 scripts/openspec_archive_order.py match-lingua-french-elided-pieces` exits 10 naming the changes of `archiveAfter` still open; row 44c is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Owner

- [x] 5.1 [manual] Approved by the owner on 2026-10-10 (in session), before the implementation. The owner approves the re-bless before the pull request merges: `fr-en.golden` and `fr-es.golden`, four phrase probes added and none moved (two of fr-en's recording no expression where the engine today reports `de l'` and `de un`), and the four cards added to `word-card-fr-en.txt` and `word-card-fr-es.txt`.
- [ ] 5.2 [manual] In change 52's dogfood, the owner checks on a real page in fr-en that « de l'eau » shows `de l'` and « de le faire » no expression.
