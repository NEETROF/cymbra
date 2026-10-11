# keep-lingua-expression-glosses-whole — an expression's gloss keeps its senses whole

## Why

Dogfooding change 34 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`enable-lingua-english-speakers`, its draft pull request #810), the owner selected « sin embargo » and
read "however, notwithstanding, nevertheless, in". The English Wiktionary writes "however,
notwithstanding, nevertheless, in spite of, nonetheless": the reduction rules every pair shares cut an
expression's sense at 42 characters, mid-word and with no mark of the cut
(`reduce_common.reduce_expressions(…, per_sense=42)`, `clean_gloss` without `whole_words`), then the
joined senses at 80 characters, raw (`_join_senses`, `joined[:maxlen]`). fr-en's split words
(`split_words`) and its lent sense (`lent_senses`) repeat the same 42 / 80, and fr-es's labels
(`label_expressions`) rebuild them.

The cut is everywhere. On the committed tables (`main` at `61430101`), the rows holding a sense cut at
exactly 42 characters are es-en 1,283 of 15,515, es-fr 472, en-es 217, en-fr 2,467, fr-en 4,213 and
fr-es 129 — and the joined cut at 80 shortens more. A word's gloss has not been cut like this since
`add-lingua-word-grammar` (whole senses, a cut at a word with « … » only past 300 characters a sense);
an expression's gloss was left with the old limits (*A gloss groups its senses by part of speech*:
« An expression's gloss keeps its own limits »).

The owner decided on 2026-10-11, choosing between cutting at a word with an ellipsis and not cutting:
**an expression's gloss is no longer cut** — « sin embargo » reads "however, notwithstanding,
nevertheless, in spite of, nonetheless". This change is that decision, for every pair, outside the
programme's 57.

## What Changes

- **Whole senses** (design D1): an expression's senses are no longer cut at 42 characters, and the
  joined gloss no longer at 80. The senses are picked by the same rule — up to three, one per entry in
  turn —, so no row is added or left out and, but for one row whose cut had made two senses read alike
  (fr-en « saint-cyrien »), the same senses come back whole.
- **One safety bound** (D2): a sense longer than 300 characters — a word's sense bound — is cut after
  a whole word and ends with « … », as a word's is. It touches 29 rows of the 120,690 (en-fr 19, es-fr 7,
  fr-en 3, none in es-en, en-es or fr-es): encyclopaedic definitions of 301 to 1,132 characters
  (« sistema cardiovascular », « machete order », « maladie du sommeil ») that no card page can show.
  The owner kept it on 2026-10-11.
- **Every path reads the same rule** (D3): `reduce_common.reduce_expressions` and `_join_senses`,
  fr-en's `split_words` and `lent_senses`, fr-es's `label_expressions` (its labels still lie outside
  the bound) and en-fr's `--max-gloss-len`, which goes. `EXPRESSION_GLOSS_LEN` goes too.
- **Out of scope** (D4): a word's gloss (already whole senses within 300 / 800), the translation-table
  fallbacks (three words a part of speech, a count, not a cut), the number of senses an expression
  keeps (three), and the selection card's word-by-word row, which keeps showing an expression's first
  sense within 80 characters, cut after a whole word with « … », as it shows a word's.
- **The card** (D5): the expression card already renders through the word card's paging — 160
  characters a page, whole senses, at least one a page —; it now pages 438 expressions (en-fr 175,
  es-fr 25, es-en 53, en-es 11, fr-en 171, fr-es 3; three pages at most). No source change; a test pins
  it, and a requirement says it.
- **Measured** (design *Measured*): rows changed en-fr 2,878, es-fr 525, es-en 1,793, en-es 132,
  fr-en 5,178, fr-es 109 (10,615 in all); the longest gloss 537 characters (en-fr « situation
  awareness »); packs +1,012 B (fr-es) to +49,286 B (fr-en), the largest 3,052,887 B, 58.2 % of the
  5 MiB budget; the extension's package (en-fr and es-fr today) about 44 KB larger zipped.
- **What moves** (D6): every pair's `mwe.tsv`, `manifest.json` (`pack_version`) and `pin.json` (rule
  digest, pack); `gloss.tsv`, `senses.tsv`, `NOTICE`, the studied tables and every source record byte
  for byte. All six goldens' `pack` lines; expression glosses on 1 (en-es), 2 (en-fr), 8 (es-en), 19
  (fr-en) and 5 (fr-es) probe lines, none in es-fr; `word-card-fr-en.txt` 11 lines, `word-card-fr-es.txt`
  5. **en-fr and es-fr move**, by the owner's decision: the programme's rule that their output never
  moves is set aside for this change alone, and only their expressions move.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *An expression's gloss keeps its senses whole*. MODIFIED *A gloss groups
  its senses by part of speech* (its sentence « An expression's gloss keeps its own limits » now
  points at the new requirement; no open change holds it) and *fr-es's glosses show a sense's register,
  age and place in the Spanish Wiktionary's words* (held by `refine-lingua-fr-es-glosses`, in
  `archiveAfter`: its expression scenario showed the 42-character cut).
- `lingua-browser-extension`: ADDED *An expression's card shows its whole gloss, paged*. It builds on
  *An expression is the card's answer*, held by `add-lingua-expression-table` (in `archiveAfter`).
- `lingua-analysis`: none. The matcher and the core hand the gloss through as the pack holds it.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce_common.py`, `reduce-en-fr.py`, `reduce-fr-en.py`,
    `reduce-fr-es.py`, their tests (`test_reduce_editions.py`, `test_reduce_en_fr.py`,
    `test_reduce_fr_en.py`, `test_reduce_fr_es.py`), the six pairs' `mwe.tsv`, `manifest.json`,
    `pin.json` and `README.md`, `SOURCES.md`; *consumed*: the pinned sources, unchanged.
  - `crates/lingua-wasm/tests/baseline/` — the six goldens re-blessed.
  - `apps/lingua-extension/test/` — `baseline/word-card-fr-en.txt` and `word-card-fr-es.txt`
    re-blessed; `row-gloss-tables.spec.ts` gains the expressions; a card test of a paged expression.
    No source file of the extension changes.

  ID, Music, Live, the back office, the site, the backend, the core and the Apple host app are
  untouched. The agent plugin keeps its own French glosses (M17).
- **Licences, sources, snapshot.** None change: each pair is reduced again from its pinned release
  assets; nothing is fetched anew.
- **Release.** en-fr and es-fr ship today, so the next extension release carries their longer
  expression glosses: the owner decides when (M18). es-en, en-es, fr-en and fr-es ship with changes
  34, 35 and 52, which list them.
- **Order.** After 49b (`refine-lingua-fr-es-glosses`, on `main`). Any change that re-pins a pair
  while this one is in flight re-pins the same files: whichever lands second reduces again on the
  other (design D7).
- **Effort**: 1–1.75 ideal days (design, *Effort*).
