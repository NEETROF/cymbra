# Tasks

## 0. Before the implementation

- [x] 0.1 [manual] The owner answers open question 1 (design D2): answered on 2026-10-11 — the 300-character bound on an expression's sense (29 rows) is kept, cut after a whole word with « … ».

## 1. The rules (scripts/lingua-data/)

- [ ] 1.1 `reduce_common.py` (design D1–D3): `EXPRESSION_GLOSS_LEN` removed; `EXPRESSION_SENSE_LEN = WORD_GLOSS["per_sense"]` (300); `reduce_expressions(path, per_sense=EXPRESSION_SENSE_LEN, max_senses=3, *, studied, edition)` cleans each sense with `clean_gloss(…, per_sense, whole_words=True, edition=…)`; `_join_senses(per_entry, max_senses)` joins with no length; `native_tables` loses `expression_len`; the docstrings of `clean_gloss` (« The expressions keep the plain cut ») and `_join_senses` say the new rule. The raw cut of `clean_gloss` (`whole_words=False`) goes if nothing else reads it.
- [ ] 1.2 `reduce-en-fr.py`: `--max-gloss-len` and its `expression_len` argument removed, the comment on the word gloss's figures kept.
- [ ] 1.3 `reduce-fr-en.py`: `split_words` reads `common.EXPRESSION_SENSE_LEN` and `whole_words=True`, no `maxlen`; `lent_senses` joins with no cut (its docstring and `LENT_SENSES`'s comment follow); the comment of `with_etc_period` about « one character over the shared rules' 80 » removed.
- [ ] 1.4 `reduce-fr-es.py`: `EXPRESSION_SENSE = common.EXPRESSION_SENSE_LEN`, `label_expressions` cleans with `whole_words=True` and rebuilds with `_join_senses(…, EXPRESSION_SENSES)`; its docstring's 42 / 80 / « 13 run past 80 » figures replaced by the re-measured ones (task 2.2).
- [ ] 1.5 Tests, one per scenario of *An expression's gloss keeps its senses whole*: on recorded entries in `test_reduce_editions.py` (*A sense longer than the old cut*: « sin embargo »; *Senses longer than 80 characters together*; *A sense past the bound*: a 1,132-character sense cut after a whole word with « … », length ≤ 300; *Senses the cut made alike*: two senses sharing 42 characters both kept), `test_reduce_fr_en.py` (*A lent sense*: « de l' » uncut; `split_words` keeps a 60-character sense whole), `test_reduce_fr_es.py` (*An expression's label outside its cut*: « mal aux cheveux » whole with « (anticuado) »; « fils de pute » re-measured; the signature check of `reduce_expressions`' defaults follows `EXPRESSION_SENSE_LEN`). The cut's old tests move with it: `test_reduce_en_fr.py` (the « 42 characters per sense … joined cut » case and the `maxlen=40` call), `test_reduce_editions.py`'s `reduce_expressions(…, 80, …)` call. `test_reduce_editions.py`: every pair's rule digest moves (`reduce_common.py` is in each).

## 2. The re-pins (scripts/lingua-data/tables/)

- [ ] 2.1 `build.sh --reduce <pair>` from each pair's pinned release assets, nothing fetched anew, in the order en-fr, es-fr, fr-en (the references), es-en, en-es, fr-es (design D6): `mwe.tsv`, `manifest.json` (`pack_version`) and `pin.json` (`reducer`, `pack`) committed per pair; `gloss.tsv`, `senses.tsv`, `NOTICE`, the snapshots, the studied and source records, `tables/en/`, `tables/es/` and `tables/fr/` byte for byte (`git diff --stat` shows nothing else); `gloss_coverage.py` passes for every pair at its committed figures. If a change re-pinning a pair lands first, reduce on it and re-measure (D7).
- [ ] 2.2 Re-measure every figure of design *Measured* on the committed tables — rows changed, the longest gloss and sense, the rows past 80 / 160 / 300, the rows the bound cuts, the 87 rows whose senses do not align, the pack and zipped sizes, the cards that page, the rows' first senses —, and record any that differs in the pull request.
- [ ] 2.3 Each pair's `README.md` (its expressions' figures and the rule, the bound and its rows) and `SOURCES.md` (« expressions keep 42 / 80 » replaced by the rule); each shipped or listed pair's *Known data defects* gains what design *Found while measuring* names in it (es-fr « sistema cardiovascular », en-fr « look forward to », the inner « ; »).

## 3. The goldens and the card

- [ ] 3.1 `LINGUA_BLESS=1 cargo test -p lingua-wasm --test english_baseline --test spanish_baseline --test es_en_baseline --test en_es_baseline --test french_baseline --test fr_es_baseline`: the six goldens re-blessed; the diff holds the `pack` lines and the expression glosses of design *Measured* (en-fr 2, es-fr 0, es-en 8, en-es 1, fr-en 19, fr-es 5 probe lines), nothing else. The pull request lists each moved line.
- [ ] 3.2 In `apps/lingua-extension`: `yarn vitest run test/word-card-fr-en.spec.ts test/word-card-fr-es.spec.ts -u` (11 and 5 lines, the same expressions); `word-card-es-en.txt`, `word-card-en-es.txt` and `selection-rows-fr.txt` unchanged without `-u`.
- [ ] 3.3 `test/row-gloss-tables.spec.ts`: its two checks over every committed gloss also read every pair's `mwe.tsv` — no expression's row empty, none ending on an opening mark before its « … » (measured: none).
- [ ] 3.4 A card test (beside `test/wordpopup.spec.ts`'s paging tests), one per scenario of *An expression's card shows its whole gloss, paged*: « sin embargo »'s whole gloss on one page with no paging control; an expression of three 100-character senses paged, page 1 of its pages, each page whole senses; "+ Deck" on it storing the first page; a row of an expression whose first sense runs 95 characters cut within 80 with « … ».

## 4. Gates

- [ ] 4.1 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; `pack_sources.py check-reducer` for every pair; `scripts/lingua-data/build.sh <pair>` (build mode) reproduces every committed pin.
- [ ] 4.2 `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p lingua-pack -p lingua-wasm`; in `apps/lingua-extension`: `yarn test`, `yarn lint`, `yarn typecheck`, `yarn format:check`.
- [ ] 4.3 `openspec validate keep-lingua-expression-glosses-whole --strict`.

## 5. The owner

- [ ] 5.1 [manual] Dogfood on the built extension: « sin embargo » with es-en (change 34's draft), « ça va » with fr-en, a paged expression card (en-fr « look forward to » or fr-en « mardi gras »), on a computer and on a phone or tablet; « + Deck » on a paged one.
- [ ] 5.2 [manual] The release carrying en-fr's and es-fr's re-pinned packs (M18): the owner decides when.
