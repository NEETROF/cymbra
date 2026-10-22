# Design — add-lingua-pack-it-es

## Context

Change 27 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2: Spanish speakers
studying Italian. Like it-en (change 22), it-es is a reader pair: Italian's studied side is it-fr's
(changes 9, 16), read as committed. Its native side is the thinnest of the programme: the planned
sources — the Spanish Wiktionary's Italian section, the Italian Wiktionary's Spanish translations, the
Spanish Wiktionary's tables read backwards — gloss 57.8 / 46.8 / 35.0 % of the 5,000 / 10,000 / 20,000
commonest Italian lemmas, about 25 points under fr-es. The owner settled I4 on 2026-10-23 (#1004): it-es
also reads Wikidata Lexemes, the Italian Wiktionary's Spanish section read backwards, and seven other
Wiktionaries' translation tables as a pivot, every pair kept — an exception to M5 that I11 records for
this pair. Change 27a (`add-lingua-it-es-pivot-sources`) registers, derives and pins those files; its D6
lists what this change takes from them.

Precedents: fr-es (French glossed in Spanish, the Spanish edition's rules, its table rules — letters,
listed once, typography, `read_backwards` — in `reduce-fr-es.py`, and its floor fixed before measuring)
and it-en (a reader pair of Italian, its own pre-pass module).

## Goals / Non-Goals

**Goals:**
- it-es's native side committed at or above a floor fixed before measuring, every gloss Spanish written by
  a person, the pivot's after every direct source, each marked by its source.
- The Spanish edition's shared rules untouched; fr-es's tables byte for byte.

**Non-Goals:**
- Shipping it-es (change 30), its listings (31), its marks (29), its refinement (28).
- A pivot for any other pair.

## Measured

On change 16's study ranking (loose; strict where given), with the repository's own `derive` and shared
rules and the Spanish edition's; the Spanish dump of 2026-10-20 (103,226,087 B gzipped) reproduces fr-es's
pinned files byte for byte.

- **The planned sources**: the Spanish Wiktionary's Italian section, 6,874 entries, 7,810 glossed senses,
  4,587 glossed headwords (`kaikki-es-Italiano.jsonl`, 5,362,930 B); the Italian Wiktionary's Spanish
  translations, 13,018 of 7,029 Italian words; the Spanish tables read backwards, 13,837 of 10,244.

  | Sources | loose | strict |
  |---|---|---|
  | definitions | 31.2 / 22.6 / 15.2 | 30.7 / 21.2 / 13.9 |
  | + the Italian Wiktionary's translations | 46.8 / 36.4 / 26.4 | 43.6 / 32.8 / 23.0 |
  | + the Spanish tables backwards, fr-es's table rules | 57.8 / 46.8 / 35.0 | 54.6 / 42.4 / 31.0 |
  | + the pre-pass of D4 | 58.0 / 46.9 / 35.1 | 54.8 / 42.5 / 31.0 |
  | + change 27a's sources (C) | **84.8 / 76.3 / 65.8** | **79.1 / 69.6 / 58.6** |

  fr-es committed 83.2 / 70.9 / 57.1 (floor 81.4 / 68.8 / 54.5); it-fr's study 89.2 / 82.2 / 73.4,
  it-en's 92.0 / 85.1 / 76.7.
- **Samples** (judged by the research, not a native speaker): the two planned tables' rows in the top
  5,000, 91 right, 6 partly, 6 wrong of 103 — the direct table 2 wrong of 56 (`fiori` « Bastos »,
  `patente` « Paté »), the inverted 4 of 47 (`altrui` « Extraño », `seminario` « Taller »); the pivot's
  rows, 40 right, 8 partly, 2 wrong of 50, its errors on homographs and participles (`sparato`
  « Bragueta », `sino` « Seno », `braccia` « Braza »).
- **Defects at the top** (5,000 / 10,000 / 20,000):
  - *names*: a name's note beside common senses in 26 / 31 / 33 rows (`secondo` « Apellido; Segundo;
    Según »), a name's note alone in 65 / 100 / 150; it-fr's amended D5 cleans 20 / 24 / 24 rows, losing
    nothing; with « a name's note also yields where a table glosses the word », 34 / 40 / 41 (`segreto`
    « Apellido » → « Secreto, oculto », `gesù` « Jesús »);
  - *pointers*: worded pointers never lend their base (« Variante de … »: `quel`, `aver`, `qual`, 18 of the
    20,000); lent for a headword with no meaning of its own, 15 gain a gloss (`aver`, `voler`, `dir`),
    without that limit `sono` would read « Sonido »; a meaning after a colon, 4 more (`sta` « Esta »),
    without the limit `sette` « Siete; Sectas »;
  - *notes*: « _(1, 5) » sense links in 10 / 13 / 17 rows (`quale` « …; Como_(1, 5) »), superscripts in 3
    (`vero` « Vero²₁ »), glosses opening in lowercase in 24 / 29 / 32 (`vedere` « ver »);
  - *function words described, not glossed*: `tu` « Pronombre personal nominativo … », `le`, `lo`,
    `gli`, `loro`, `lei`, `ne`, `è`, `sono`, `essere`, `avere` « Verbo auxiliar … » — 11 of the top 177;
  - *tables*: the same word repeated across parts of speech in 478 / 738 / 1,042 rows (`accaduto`
    « Hecho; Hecho; Hecho »), which fr-es's listed-once rule takes to 0; 18 words with a grave accent and 3
    « ° » from the Italian Wiktionary's editors (`abbonamento` « Suscripciòn »); the Italian word given as
    its own Spanish gloss, 11 rows (`corso` « Corso »); fr-es's letters rule hard-codes the language code
    « fr » (`reduce-fr-es.py`), which here glosses `it` « Manzano, maletero ».
- **The pivot's groups** (27a's *Measured*): 28.7 % of rows name no sense; keeping only named senses and
  one-sense entries costs 0.4 points at the 5,000 (83.4 against 83.8, four editions).

## Decisions

### D1 — A reader pair

`reduce-it-es.py` reads `tables/it/` as committed and writes `tables/it-es/{gloss.tsv, senses.tsv,
mwe.tsv, NOTICE, manifest.json}` through `common.native_tables(…)` with the Spanish edition's rules
(`reduce_edition_es.ES`); nothing of the studied side is computed. Its pin records the studied tables it
read (es-en's D3).

### D2 — The sources, direct first, the pivot last

In this order, at most three words per part of speech from each table source: the section's definitions;
the Italian Wiktionary's Spanish translations; the Spanish tables read backwards; Wikidata's translation
links (P5972), then its lexemes of one item (P5137); the Italian Wiktionary's Spanish section read
backwards (27a's D3 rule); the pivot. The order the research measured (C); task 0.1 re-measures it on the
committed ranks.

### D3 — The pivot, read as measured

- **Groups**: a table's group named by a sense or a sense number, and the whole entry only when the entry
  has one sense; the « Traductions à trier suivant le sens » tables never — precision over 0.4 points at
  the 5,000.
- **Filters** (27a's D6): no entry tagged name, abbreviation, symbol, character, letter, affix or
  punctuation, no one-letter headword; no capitalised headword but German nouns; no Italian or Spanish
  word in capitals but acronyms of at most six letters; no Spanish word with a grave accent, « ° », or
  longer than sixty characters.
- **Order**: a lemma's Spanish words by the number of tables and editions giving them, then Spanish
  frequency, then the alphabet; at most three per part of speech, the part of speech the pivot entry's.
- **No expressions**: the pivot glosses words; its expressions were not measured.

### D4 — it-es's own pre-pass, and the Spanish table rules shared

- `reduce_italian_es.py`, imported by `reduce-it-es.py` and its tests: names — it-fr's amended D5 (an
  acronym's sense and a name's note keep off a word with a lowercase entry) and the thin-pair clause (a
  name's note also yields where a table glosses the word); worded pointers lending their base, and a
  meaning after a colon, only for a headword with no meaning of its own; « _(n, m) » sense links and
  superscripts out, a gloss's opening capitalised; the commonest function words from a reviewed list, in
  fr-es's `CORRECTIONS` shape (`tu` « Tú », `è` « Es », …). Not ported: fr-es's D7 (13 rows, most worse:
  `enorme` « Grande, muy, bueno »), its D4 and its determiner rule.
- **fr-es's table rules** — letters, listed once, typography, `read_backwards` — move out of
  `reduce-fr-es.py` into a module both reducers import, `reduce_spanish_tables.py`, the studied language a
  parameter (fr-es's « fr » is hard-coded today), as `reduce_french_treebank.py` serves fr-en and fr-es.
  fr-es's tables stay byte for byte; its pin's reducer files and digest are re-recorded. Copying them
  instead would duplicate a committed pair's code. it-es's tables also drop grave-accented words, « ° »
  and the Italian word given as Spanish (refine-lingua-fr-es-glosses' D5 test).

### D5 — The Spanish edition's rules, a third reader

`reduce_edition_es.py` is not edited. fr-es's requirement says its rules « SHALL be read by en-es and
fr-es and by no other committed pair »: MODIFIED to name it-es, its scenario on the rule digests too, as
change 22 did for the English edition; `test_reduce_editions.py`'s list gains it-es.

### D6 — Expressions

`mwe.tsv`: the section's glossed headwords with a space (198 after the shared filters), then the two
Wiktionaries' tables' (488 direct, 475 inverted; 1,161 in all), then the split words without a space, as
it-en's D4.5 reads them; each keyed at build by change 12's rules. Not the pivot (D3).

### D7 — The acronym rows

Glossed by change 11's rule from it-es's own glosses (`NATO`, `USA`).

### D8 — The floor (I4)

Task 0.1 re-runs the study on change 9's committed ranks with every source and the pre-pass, and fixes
`FLOORS["it-es"]` at those figures less two points, the owner settling it before the first committed
measurement; never lowered after it; the floor step of `lingua-extension-check.yml`'s `reduce` job runs
`--pair it-es`. On the study's ranking that would be about 82.8 / 74.3 / 63.8.

### D9 — Coverage, provenance and credits

`tables/it-es/README.md`: the coverage against the floor, the glossed lemmas by source — definitions,
each table, Wikidata, the Spanish section, the pivot by edition —, a sample of 100 glosses, each marked
by its source and a pivot row by its edition, entry and sense, and the pack's size. The notice and the
manifest credit the Spanish Wiktionary, the Italian Wiktionary, the seven Wiktionaries of the pivot
(CC BY-SA 4.0 + GFDL), Wikidata (CC0 — a licence variant `Cc0` in `crates/lingua-pack/src/licence.rs`),
and the studied side's sources; the manifest says Italian's levels are estimated. `SOURCES.md` and the
programme's *Measured figures* record the committed coverage.

### D10 — The no-pivot requirement, the exception written

The main spec's *A gloss is in the reader's language, written by a person* says « never pivoted through a
third language », its scenario *No pivot through English* that neither of two translations of one sense
glosses the other: MODIFIED for it-es alone — the pivot qualifies after every direct source, marked by its
source —, the scenario kept, *A pivot for it-es* added. No open change holds the requirement.

### D11 — A baseline, and not shipped

`it_es_baseline.rs` and `baseline/it-es.golden` over the Italian corpus, a test that it is it-fr's on the
studied side, `cross_native.rs` with it-es, both workflows running it; blessed in the pull request that
commits the tables. No package lists it-es before change 30.

## Known data defects

For change 28 (`refine-lingua-it-es-glosses`), after the committed tables: the pivot's homographs and
participles (`sparato` « Bragueta », `sino` « Seno », `braccia` « Braza »); the tables' wrong rows at the
top (`posto` « En », `successo` « Consentimiento », `periodo` « Menstruación », `caso` « Azar »,
`controllo` « Aprobanza »); definitions repeating a word across parts of speech (69 rows, `riso`
« (Oryza sativa) Arroz; Arroz »); notes before the meaning (`cavallo` « (Equus caballus) Caballo »);
the Spanish edition's gaps also in fr-es and en-es (« _(n) », « Contracción de »), named for a follow-up
of that edition rather than fixed here.

## Risks / Trade-offs

- **Pivoted glosses, wrong in places** → read last, marked by source, the coverage and sample published,
  change 28 refining; the owner's decision (I4, I11).
- **Moving fr-es's table rules** → fr-es's tables byte for byte, checked by the reduce job; its digest
  re-recorded.
- **5.3 GB of dumps per it-es update** → 27a's D5.

## Migration Plan

Nothing deploys: the tables are committed, the pack is not listed. Rollback is a revert (fr-es's module
move with it).

## Effort

4–7 ideal days, against the programme's 3–5 before I4: the reducer and the pre-pass module 1.5–2.5; the
pivot's and Wikidata's readers 1–1.5; the shared Spanish table rules and fr-es's re-recorded pin 0.5–1;
the reduction, the floor and the README 0.5–1; the baseline and the tests 0.5–1.
