# Design — add-lingua-pack-it-es

## Context

Change 27 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2: Spanish speakers
studying Italian. Like it-en (change 22), it-es is a reader pair: Italian's studied side is it-fr's
(changes 9, 16), read as committed. Its native side is the thinnest of the programme: the planned
sources — the Spanish Wiktionary's Italian section, the Italian Wiktionary's Spanish translations, the
Spanish Wiktionary's tables read backwards — gloss 57.8 / 46.8 / 35.0 % of the 5,000 / 10,000 / 20,000
commonest Italian lemmas, 25 / 24 / 22 points under fr-es. The owner settled I4 on 2026-10-22 (#1004): it-es
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
- The Spanish edition's shared rules untouched; every other pair — fr-es and en-es among them — byte for
  byte, its reducer, pin, pack and golden included.

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
  | + change 27a's sources, no pre-pass, every group (C) | 84.8 / 76.3 / 65.8 | 79.1 / 69.6 / 58.6 |
  | + the pre-pass and D3's groups (as designed) | **84.7 / 76.1 / 65.5** | **79.1 / 69.3 / 58.2** |

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
    nothing; with « a name's note also yields where the Italian or the Spanish Wiktionary's translation
    table glosses the word », 33 / 39 / 40 (`segreto` « Apellido » → « Secreto, oculto », `gesù` « Jesús »;
    the research's 34 / 40 / 41 also dropped two acronyms, `DOC` and `ARN`, which the clause leaves);
  - *pointers*: worded pointers never lend their base (« Variante de … »: `quel`, `aver`, `qual`, 18 of the
    20,000); lent for a headword with no meaning and no tagged pointer of its own, 14 gain a gloss (`aver`,
    `voler`, `dir`; 15 rows change, `accelerando`'s « Accelerando » giving way to « Acelerar »); with no
    meaning as the only limit, `sono` — its verb senses tagged forms of `essere`, its noun « Variante de
    suono » — would read « Sonido »; a meaning after a colon, for a headword with no meaning of its own,
    4 more (`sta` « Esta »), without the limit `sette` « Siete; Sectas »;
  - *notes*: « _(1, 5) » sense links in 10 / 13 / 17 rows (`quale` « …; Como_(1, 5) »), superscripts in 3
    (`vero` « Vero²₁ ») and subscripts beside them (`cazzo` « «Coño»₆ »), glosses opening in lowercase in
    24 / 29 / 32 (`vedere` « ver »);
  - *function words described, not glossed*: `tu` « Pronombre personal nominativo … », `le`, `lo`,
    `gli`, `li`, `loro`, `lei`, `ne`, `è`, `sono`, `essere`, `avere` « Verbo auxiliar … » — 12 of the top
    177; `della` (rank 17) « Contracción de la preposición di y el artículo la. » is the edition's gap
    (*Known data defects*);
  - *tables*: the same word repeated across parts of speech in 478 / 738 / 1,042 rows (`accaduto`
    « Hecho; Hecho; Hecho »), which fr-es's listed-once rule takes to 0; 18 words with a grave accent and 3
    « ° » from the Italian Wiktionary's editors (`abbonamento` « Suscripciòn »); the Italian word given as
    its own Spanish gloss, 11 rows (`corso` « Corso »), of which `credo` and `friulano` — likely
    `nutrimento` and `visivo` too — are Spanish words as well (DLE); fr-es's `read_backwards` hard-codes
    the language code « fr » (`_LANGUAGE_CODE`, refine-lingua-fr-es-glosses' D4), which here glosses `it`
    « Manzano, maletero »;
  - *the other table-shaped sources*, with D3's pivot: the pivot repeats a word across parts of speech in
    132 / 230 / 366 rows (`adattatore` « Adaptador; Adaptador ») and gives the Italian word as Spanish in
    4 / 8 / 15 (`correttivo`, `impresario`); the Spanish section read backwards repeats in 2 / 2 / 9
    (`certo` « Seguro; Seguro », rank 184).
- **The pivot's groups** (27a's *Measured*): 28.7 % of rows name no sense; keeping only named senses and
  one-sense entries costs 0.4 points at the 5,000 on four editions (83.4 against 83.8), 0.3 / 0.3 / 0.4 on
  seven with the pre-pass (84.7 / 76.1 / 65.5 against 85.0 / 76.4 / 65.9); the « à trier » tables cost
  nothing measurable.

## Decisions

### D1 — A reader pair

`reduce-it-es.py` reads `tables/it/` as committed and writes `tables/it-es/{gloss.tsv, senses.tsv,
mwe.tsv, NOTICE, manifest.json}` with the Spanish edition's rules (`reduce_edition_es.ES`); nothing of
the studied side is computed. It builds the native side step by step, as fr-es's `native_side`
(`reduce-fr-es.py`), not through `common.native_tables(…)`, whose primary count cannot give D9's
coverage by source and provenance by row: each source's glosses are kept apart, the pivot reader keeping
each Spanish word's edition, entry and sense. Its pin records the studied tables it read (es-en's D3).
Its rule digest is its own file, `reduce_italian_es.py` and `reduce_spanish_tables.py` (D4),
`reduce_italian_acronyms.py` (change 11), `reduce_common.py` and `reduce_edition_es.py`.

### D2 — The sources, direct first, the pivot last

In this order, at most three words per part of speech from each table-shaped source (every source but
the section's definitions): the section's definitions; the Italian Wiktionary's Spanish translations; the
Spanish tables read backwards; Wikidata's translation links (P5972) and its Spanish glosses of Italian
senses (27a's D3, 8 glosses, counted in the measurement), then its lexemes of one item (P5137); the
Italian Wiktionary's Spanish section read backwards (27a's D3 rule); the pivot. Wikidata's items before
the Spanish section: swapping the two leaves the coverage as it is and changes 20 / 28 / 43 rows,
Wikidata's mostly the better (`corrente` « Corriente eléctrica » against « Ala », `distretto` « Distrito »
against « Zona »). The order the research measured; task 0.1 re-measures it on the committed ranks.

### D3 — The pivot

- **Groups**: a table's group named by a sense or a sense number, and the whole entry only when the entry
  has one sense; the « Traductions à trier suivant le sens » tables never — precision over 0.3 / 0.3 /
  0.4 points, seven editions with the pre-pass; the « à trier » tables cost nothing measurable.
- **Filters** (27a's D6): no entry tagged name, abbreviation, symbol, character, letter, affix or
  punctuation, no one-letter headword; no capitalised headword but German nouns; no Italian or Spanish
  word capitalised but acronyms of at most six letters; no Spanish word with a grave accent, « ° », or
  longer than sixty characters.
- **Order**: a lemma's Spanish words by the number of tables and editions giving them, then Spanish
  frequency, then the alphabet; at most three per part of speech, the part of speech the pivot entry's.
- **No expressions**: the pivot glosses words; its expressions were not measured.

### D4 — it-es's own pre-pass, and the Spanish table rules copied

- `reduce_italian_es.py`, imported by `reduce-it-es.py` and its tests:
  - *names* — it-fr's amended D5 (an acronym's sense and a name's note keep off a word with a lowercase
    entry) and the thin-pair clause (a name's note also yields where the Italian or the Spanish
    Wiktionary's translation table glosses the word);
  - *pointers* — a pointer naming a word lends that word's gloss only to a headword with no meaning and
    no pointer the edition tags of its own (`aver` from `avere`; never `sono` from `suono`); a meaning a
    pointer carries after a colon is read only for a headword with no meaning of its own;
  - *notes* — « _(n, m) » sense links, superscripts and subscripts out, a gloss's opening capitalised;
  - *function words* — the commonest from a reviewed list, in fr-es's `CORRECTIONS` shape (`tu` « Tú »,
    `è` « Es », `li` « Los », …);
  - *the Italian word given as Spanish* — refused unless `ITALIAN_ALIKE`, a reviewed list, names it a
    Spanish word too (`credo`, `friulano`; the DLE the reference), as fr-es's `SPANISH_ALIKE`;
  - *split words* — the reader of the words without a space that Italian's pre-pass splits, for D6: a
    copy of it-en's D4.5 reader, which lives in `reduce_italian_en.py`, so that neither pair's digest
    follows the other's.

  Not ported: add-lingua-pack-fr-es's D7 (13 rows, most worse: `enorme` « Grande, muy, bueno »), its D5
  name rule (it-fr's amended D5 instead) and its determiner reading; refine-lingua-fr-es-glosses' D2
  labels, D7 corrections and D8 treebank order.
- **fr-es's table rules, copied** — letters, listed once, typography, `read_backwards` — into
  `reduce_spanish_tables.py`, imported by it-es alone, every fr-es global they read a parameter: the
  studied language and its code (`FR`, `_LANGUAGE_CODE`), the corrections and the other senses
  (`CORRECTIONS`, `OTHER_SENSES`), the alike list, and the language of the studied word's frequency
  (`is_the_french_word`'s `zipf(word, "fr")`). Not moved: a module in fr-es's rule files would move its
  `pack_version`, which embeds its rules digest (`2026.10.10+3f311ba.ba5ec32`) — its manifest, its pin,
  its golden's pack line, and a pack `packs.json` ships. fr-es itself copied en-es's `listed_once`
  rather than move en-es, and refine-lingua-fr-es-glosses' D8 wrote `reduce_french_treebank.py` so that
  fr-en's digest would not move; fr-es and en-es adopt the module at their next re-pin. Sonar does not
  analyse `scripts/`, so the copy meets no duplication gate.
- **Every table-shaped source** (D2) is read through them: no letter; no Spanish word listed under two
  parts of speech of one Italian word — listed once, under the first of them Italian's readings name, or
  else the first listed —; no word with a grave accent or « ° »; no Italian word given as Spanish but
  `ITALIAN_ALIKE`'s — the pivot's 132 / 230 / 366 repeats and 4 / 8 / 15 Italian words, and the Spanish
  section's 2 / 2 / 9 repeats, with the two Wiktionaries' tables' (*Measured*).

### D5 — The Spanish edition's rules, a third reader

`reduce_edition_es.py` is not edited. fr-es's requirement says its rules « SHALL be read by en-es and
fr-es and by no other committed pair »: MODIFIED to name it-es, its scenario on the rule digests too, as
change 22 did for the English edition; `test_reduce_editions.py`'s list gains it-es.

### D6 — Expressions

`mwe.tsv`: the section's glossed headwords with a space (198 after the shared filters), then the two
Wiktionaries' tables' (488 direct, 475 inverted; 1,161 in all), then the words without a space that
Italian's pre-pass splits, read by D4's copy of it-en's D4.5 reader; each keyed at build by change 12's
rules. Not the pivot (D3).

### D7 — The acronym rows

Glossed by *Italian's acronyms are a reviewed table, glossed through their entry's expansions or
synonyms* (change 11) from it-es's own glosses (`NATO`, `USA`).

### D8 — The floor (I4)

Task 0.1 re-runs the study on change 9's committed ranks with every source and the pre-pass, and fixes
`FLOORS["it-es"]` at those figures less two points, the owner settling it before the first committed
measurement; never lowered after it; the floor step of `lingua-extension-check.yml`'s `reduce` job runs
`--pair it-es`. On the study's ranking that would be about 82.7 / 74.1 / 63.5 (the designed figures,
84.7 / 76.1 / 65.5, less two).

### D9 — Coverage, provenance and credits

`tables/it-es/README.md`: the coverage against the floor, the glossed lemmas by source — definitions,
each table, Wikidata, the Spanish section, the pivot by edition —, a sample of 100 glosses, each marked
by its source and a pivot row by its edition, entry and sense, and the pack's size. The notice and the
manifest credit the Spanish Wiktionary, the Italian Wiktionary, the seven Wiktionaries of the pivot
(CC BY-SA 4.0 + GFDL), Wikidata (CC0 — a licence variant `Cc0` in `crates/lingua-pack/src/licence.rs`,
never denied; manifests name a licence, so the new variant moves no pack),
and the studied side's sources; the manifest says Italian's levels are estimated. `SOURCES.md` and the
programme's *Measured figures* record the committed coverage.

### D10 — The no-pivot requirement, the exception written

The main spec's *A gloss is in the reader's language, written by a person* says « never pivoted through a
third language », its scenario *No pivot through English* that neither of two translations of one sense
glosses the other: MODIFIED for it-es alone — Wikidata's items after the Spanish section and the two
Wiktionaries' tables, the pivot after every other source, each marked by its source —, the scenario kept, *A pivot for it-es* added. No open change holds the requirement.

### D11 — A baseline, and not shipped

`it_es_baseline.rs` and `baseline/it-es.golden` over the Italian corpus, a test that it is it-fr's on the
studied side, `cross_native.rs` with it-es, both workflows running it; and the Spanish card snapshot
change 18's D8 hands this change, `apps/lingua-extension/test/word-card-it-es.spec.ts` over the golden,
on `lingua-pack-update.yml`'s vitest line — blessed in the pull request that commits the tables. No
package lists it-es before change 30.

## Known data defects

For change 28 (`refine-lingua-it-es-glosses`), after the committed tables: the pivot's homographs and
participles (`sparato` « Bragueta », `sino` « Seno », `braccia` « Braza »); the tables' wrong rows at the
top (`posto` « En », `successo` « Consentimiento », `periodo` « Menstruación », `caso` « Azar »,
`controllo` « Aprobanza »); definitions repeating a word across parts of speech (69 rows, `riso`
« (Oryza sativa) Arroz; Arroz »); notes before the meaning (`cavallo` « (Equus caballus) Caballo »);
the Spanish edition's gaps also in fr-es and en-es (« Contracción de », `della`), named for a follow-up
of that edition rather than fixed here; « _(n) », fixed in it-es's pre-pass, is that follow-up's for
fr-es and en-es.

## Risks / Trade-offs

- **Pivoted glosses, wrong in places** → read last, marked by source, the coverage and sample published,
  change 28 refining; the owner's decision (I4, I11).
- **Three copies of the Spanish table rules** until fr-es and en-es re-pin → the module's tests hold the
  rules fr-es's tests hold; fr-es and en-es adopt it at their next re-pin, which moves their digests
  anyway.
- **5.3 GB of dumps per it-es update** → 27a's D5.

## Migration Plan

Nothing deploys: the tables are committed, the pack is not listed. Rollback is a revert.

## Effort

4–7 ideal days, against the programme's 3–5 before I4: the reducer and the pre-pass module 1.5–2.5; the
pivot's and Wikidata's readers 1–1.5; the copied Spanish table rules, parameterised, 0.5–1;
the reduction, the floor and the README 0.5–1; the baseline and the tests 0.5–1.
