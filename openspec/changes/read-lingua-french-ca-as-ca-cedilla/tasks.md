# Tasks

## 0. Owner, before the implementation

- [ ] 0.1 [manual] The owner approves what moves (design D4, D5) before 1.1: `ca` read as `ça` and its own words lost, `CA` in capitals included (D2); one word up a band at each level boundary (`croissance` A2 → A1, `communiste`, `australien`, `naïveté`, `morphologie`, `crabe` C2); `fr-en.golden` and `fr-es.golden` moving 9 of their 213 probes each and gaining the three phrase probes; the two word-card snapshots gaining three cards. The owner answers open question 1 (`CA` in capitals); an answer that departs from the recommendation is written into the design, the spec and these tasks first. Questions 2–4 name follow-ups and block nothing.

## 1. The rule (scripts/lingua-data/reduce-fr-en.py)

- [ ] 1.1 `UNMARKED_SPELLINGS` beside `ELISIONS`, form → (word, reason), one row `ca` → *ça* with D1's reason; `Lexicon.read` reads a row's form as it reads an elided piece (the named word its one candidate, the entry returning before anything else is read), `_inflections` skips the form as it skips a piece. The table's comment says why it is no override and no spelling rule (D1), and that each row is a person's decision. The module's doc names the change.
- [ ] 1.2 `test_reduce_fr_en.py`: every row has a reason; no row's form is an elided piece or one of `OWN_WORDS`; on a small section holding `CA` (noun, plural `CAs`, four initialisms), `ca` (preposition, « abbreviation of circa ») and `ça` (pronoun), `ca` maps to *ça*, is no lemma and no rank, and `cas` reaches no `ca`; a word the table does not name (`ou`, `a`) keeps its own reading; `test_reduce_editions.py`: fr-en's rule digest moves and es-en's, en-fr's, es-fr's, en-es's and fr-es's do not.

## 2. The re-pin (scripts/lingua-data/tables)

- [ ] 2.1 `build.sh --reduce fr-en` from `lingua-pack-sources-fr-en-2026.10.09`, nothing fetched beyond the release's asset and GSD's two pinned files: `tables/fr/forms.tsv`, `freq.tsv`, `level.tsv`, `lexical.tsv` and `tables/fr-en/gloss.tsv`, `senses.tsv`, `manifest.json`, `pin.json` committed — in `pin.json` only `reducer` and `pack` move —; `grammar.tsv`, `tags.tsv`, `studied.json` and `mwe.tsv` byte for byte. The diff is D4's: `ca` → *ça*, `ca` out of the ranks, the dictionary words, the levels and the glosses, `cussac` in, five words up a band and `crabe` in C2; any other row is explained in the pull request.
- [ ] 2.2 `build.sh --reduce fr-es` on the new `tables/fr/`: its `gloss.tsv`, `senses.tsv`, `mwe.tsv` and `NOTICE` byte for byte, its `pin.json` (`studied`, `pack`) and `manifest.json` (`pack_version`) committed; `gloss_coverage.py --pair fr-en` and `--pair fr-es` pass against their floors (D4: 93.6 / 86.9 / 76.3 %, 83.2 / 70.8 / 56.8 %).
- [ ] 2.3 `scripts/lingua-data/measure/fr-ud.sh`: UD French-PUD 99.13 / 96.41 / 99.90 % and GSD's test section 98.89 / 95.86 / 99.72 %, as before (D4); the figures in the pull request.
- [ ] 2.4 `tables/fr-en/README.md` (*The forms*: the table and its row, beside the elided pieces; the forms' and lexical counts; *The levels*' first words per level; *What still reads wrong*: the acronym in capitals, D2), `tables/fr-es/README.md` (the top 10,000's glossed count 7,082 → 7,083), `SOURCES.md` (French's forms rules and counts).

## 3. The committed tables (crates/lingua-pack)

- [ ] 3.1 `tests/committed_tables.rs`: in fr-en's and fr-es's packs `ca` reads as *ça* and is no lemma, `ça` keeps its rank and level, and `ou` reads as *ou*; the six committed packs still build to their pins.

## 4. The goldens (crates/lingua-wasm, apps/lingua-extension)

- [ ] 4.1 `tests/support/french.rs`: `PHRASES` gains « comme ca », « c'est ca » and « Le CA a voté le budget. » before « l’homme », its doc naming this change; `tests/french_baseline.rs` and `fr_es_baseline.rs`: their doc comments name this change among those that move or add probes.
- [ ] 4.2 Re-bless once: `LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline --test fr_es_baseline`. Check the diff is D5's — in each golden the nine probes (`pack`, `analyse new-reader informel`, the two vocabulary estimates, the two `review-current`, `export-status-ops`, `export-card-ops`, `backup`) moved for D5's causes, and the three phrase probes added with D5's readings; every other probe byte for byte — and say so in the pull request with a probe-by-probe comparison over `origin/main`'s goldens.
- [ ] 4.3 `apps/lingua-extension`: `test/word-card-fr-en.spec.ts` and `test/word-card-fr-es.spec.ts` count 37 phrase probes (their comment: 36 phrases and the reader's); `yarn vitest run test/word-card-fr-en.spec.ts test/word-card-fr-es.spec.ts -u` adds the three probes' cards to `test/baseline/word-card-fr-en.txt` and `word-card-fr-es.txt`, every block before byte for byte.

## 5. Gates

- [ ] 5.1 Nothing else moves (D4): `git diff --stat origin/main --` over `tables/en`, `tables/es`, `tables/en-fr`, `tables/es-fr`, `tables/es-en`, `tables/en-es`, `reduce_common.py`, the `reduce_edition_*.py`, `reduce-fr-es.py`, the four other pairs' goldens and `apps/lingua-extension/test/baseline` (but the two French snapshots) is empty; `pack_sources.py check-reducer` passes for the six pairs; `cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test cross_native --test parity --test card_gloss_language --test deck` passes without re-blessing; the reduce job reproduces every committed byte.
- [ ] 5.2 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p lingua-pack -p lingua-wasm`; `cargo llvm-cov --workspace --fail-under-lines 80 --ignore-filename-regex "$(cat .github/coverage-ignore-regex.txt)"`; in `apps/lingua-extension`: `yarn test` (`row-gloss-tables.spec.ts` without `-u`), `yarn lint`, `yarn typecheck`.
- [ ] 5.3 `openspec validate read-lingua-french-ca-as-ca-cedilla --strict` passes, and `python3 scripts/openspec_archive_order.py read-lingua-french-ca-as-ca-cedilla` exits 10 naming the changes of `archiveAfter` still open (0 once they are archived).
- [ ] 5.4 Row 43b of `docs/lingua/language-matrix-programme.md` says where the change stands; the pull request names D5's *Found while measuring* (fr-en's `ça va` answered by `ça ira`) and open questions 2–4 as follow-ups for the owner.

## 6. Owner

- [ ] 6.1 [manual] The owner approves the re-bless before the pull request merges: `fr-en.golden` and `fr-es.golden` — the nine moved probes with their causes and the three added —, and the three cards added to `word-card-fr-en.txt` and `word-card-fr-es.txt`; the counts the implementation re-measures are the ones approved.
- [ ] 6.2 [manual] The owner reads the moved rows of `tables/fr/` and `tables/fr-en/` (D4: `ca`, `cussac`, the six level rows) and fr-es's re-pin, and decides whether open questions 2–4 become changes of their own.
- [ ] 6.3 [manual] In change 52's dogfood, the owner checks on a real page in fr-en and fr-es that « ca » in « ca va » opens `ça`'s card and « c'est ca » the expression `c'est ça`.
