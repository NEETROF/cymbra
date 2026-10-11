# Tasks

## 1. The rules (scripts/lingua-data/reduce-fr-en.py)

- [x] 1.1 Build on change 48c (`add-lingua-fr-en-translated-expressions`): start from `main` once its implementation (#893) is merged, else stack on its branch and retarget before merging (design D6, D8). Re-run the design's harness on 48c's committed tables and record in the pull request each figure that differs from design *Measured* (matches, the list's matches, the gained ones, the pack); a section expression met ten times or more that reads wrong in most of its contexts on 48c's tables goes into `LEFT_OUT` with its reason, and is named in the pull request.
- [x] 1.2 `LEFT_OUT` gains the 111 expressions of design D1, each with its reason — its gloss, how it is met, one example from the corpora — grouped as D1 groups them, with a comment naming this change and the review's threshold (met ten times or more on the design's corpora).
- [x] 1.3 `_POINTS` (or a sibling read by `_meaning_of_pointer`) reads the notes of design D2 — « compound of X and Y », « (the|an|a) (interrogative )?form of X … in the first|second|third-person », « an interrogative form of », « for non-idiomatic uses », « used only in the », « equivalent to “ » —, on a sense and on each piece of a sense between semicolons; a sense no piece is left to goes; `expression_senses` records an expression left with no sense as pointer-only, as it records 48b's, so 48c's table leaves it out too. Never « a form of » alone (« pain d'épice »).
- [x] 1.4 `LENT_SENSES = {"de l'": "de la"}` with its reason (design D3), applied after the shared rules and before `with_etc_period`: the headword's gloss opens on the lender's first sense when its own senses lack it, cut as the shared rules cut an expression's gloss (`EXPRESSION_GLOSS_LEN`). A lender missing from the expressions fails the reduction, naming it.
- [x] 1.5 The module's doc and `native_side`'s name the three rules, their census (design *Measured*) and why no shared module is edited; the reduction's summary line counts the names left out, the pointer-only expressions and the lent senses.

## 2. Tests

- [x] 2.1 `test_reduce_fr_en.py` on recorded entries, one test per scenario of *fr-en's own expressions are read where they are met*: *A labelled sense met as plain words* (« pas de », « sa mère »), *A colloquial spelling met as the standard one* (« m'a »), *A headword met through another word's form* (« en fait »), *Words met as they are* (« première fois »), *A sense that only notes a form* (« puis-je » out, « j'ai » "I've, I have", « pain d'épice » "a form of French gingerbread" kept), *« de l' » reads "of the" first*, *The table does not bring them back* (a recorded table row for « en fait » and « de rien »); every `LEFT_OUT` row carries a reason.
- [x] 2.2 A test over the committed `tables/fr-en/mwe.tsv`: no `LEFT_OUT` name is a row, « de l' »'s gloss opens on "of the", no row's gloss holds a D2 note.
- [x] 2.3 `test_reduce_editions.py`: fr-en's rule digest moves, and es-en's, en-fr's, es-fr's, en-es's and fr-es's do not.

## 3. The tables and goldens

- [x] 3.1 `build.sh --reduce fr-en` at fr-en's pin: `tables/fr-en/mwe.tsv`, `manifest.json` (`pack_version`), `pin.json` (`reducer`, `pack`) move; `NOTICE`, `gloss.tsv`, `senses.tsv`, every table of `tables/fr/` and fr-es's tables and pin are byte for byte; `gloss_coverage.py --pair fr-en` passes. No source is fetched.
- [x] 3.2 `tables/fr-en/README.md` (*The expressions*: the review, the list by reason with its counts, the pointer notes, « de l' », the figures of 1.1, what is left — the expressions met fewer than ten times, the matcher's runs across a number or a punctuation mark, a split word met in another tense).
- [x] 3.3 `LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline` once; in `fr-en.golden` only `pack` and « de l’eau » move (design D7), each with its cause in the pull request.
- [x] 3.4 In `apps/lingua-extension`: `yarn vitest run test/word-card-fr-en.spec.ts -u` (the flag after the file); in `test/baseline/word-card-fr-en.txt` only « de l' — of the » for « de l' — some » moves, every other block byte for byte.

## 4. Owner

- [ ] 4.1 [manual] Approve what moves, before the implementation: the 111 names of design D1 (each with its contexts), the pointer notes of D2, « de l' » lent "of the" (D3), fr-en's `mwe.tsv` (17,635 → 17,521 rows in the prototype, re-measured on 48c's tables), its pin and pack, `fr-en.golden` (`pack`, « de l’eau ») and `word-card-fr-en.txt` (« de l' — of the »); the figures task 1.1 records are the ones approved. The 115 names `LEFT_OUT` gains on `main` (the design's 111 and task 1.1's four), « ça va » kept, read by the owner on 2026-10-11 (the owner, 2026-10-11: nothing removed); the pointer notes (D2), « de l' » (D3), `mwe.tsv`, its pin and pack, `fr-en.golden` and `word-card-fr-en.txt` remain to approve.

## 5. Gates

- [ ] 5.1 Nothing else moves (design D7): `git diff --stat origin/main --` over `tables/fr`, `tables/fr-es`, `tables/en`, `tables/es`, `tables/en-fr`, `tables/es-fr`, `tables/es-en`, `tables/en-es`, `reduce_common.py`, the three `reduce_edition_*.py`, the other reducers, `fr-es.golden`, the English, Spanish, es-en and en-es goldens, `word-card-fr-es.txt` and `selection-rows-fr.txt` is empty; `pack_sources.py check-reducer` passes for the six pairs; `cargo test -p lingua-wasm --test fr_es_baseline --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test cross_native` passes without re-blessing; the reduce job reproduces every committed byte.
- [x] 5.2 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p lingua-pack -p lingua-wasm`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test` (`row-gloss-tables.spec.ts` without `-u`), `yarn lint`, `yarn typecheck`.
- [x] 5.3 `openspec validate refine-lingua-fr-en-expressions --strict` passes, and `python3 scripts/openspec_archive_order.py refine-lingua-fr-en-expressions` exits 10 naming the changes of `archiveAfter` still open (0 once they are archived).
- [x] 5.4 Row 48d of `docs/lingua/language-matrix-programme.md` says where the change stands.
