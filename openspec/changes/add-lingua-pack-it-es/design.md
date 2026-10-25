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
  - *the Italian Wiktionary's notes* — reading its Spanish section backwards, its notes to its readers,
    « vedi italiano » and « definizione mancante; se vuoi, aggiungila tu » (36 and 3 senses that pass
    the reading's two-word filter, 11 of the 36 with a word before the note: « analogia, vedi
    italiano »), are taken out and a sense left empty is dropped, so that no note becomes an Italian
    key; it-es imports no Italian edition module (`add-lingua-italian-edition-rules`), whose later
    edits would re-pin it;
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

**Criteria and method, written before any it-es figure was read (2026-10-25), and committed before
the study ran:**

- *The bytes.* The sources are the files `DUMPS["it-es"]` names, derived by `pack_sources.py`'s own
  update path (`fetch_live` → `read_edition` → `derive`, run locally into a scratch folder with the
  pinned interpreter and wordfreq) from the dumps kaikki and Wikimedia serve on the day of the study —
  the bytes the dispatch's sources release would hold if run that day. Each dump is recorded by its
  address, Last-Modified, the sha256 and size of its decompressed bytes and its size as served, and each
  derived file by its sha256 and size, for the owner to approve the release. A dump regenerated before
  the dispatch moves the figures a little; the study is not re-run for it unless the owner asks.
- *The study.* The coverage of `tables/it/freq.tsv` as committed on `main` — the 5,000 / 10,000 / 20,000
  commonest, as `gloss_coverage.measure` counts them — by `gloss.tsv` exactly as `reduce-it-es.py`
  writes it, every source in D2's order with D4's pre-pass and D3's pivot; the acronym rows, unranked,
  are not counted. Also measured, from the same reduction: the coverage after each source in D2's order
  (cumulative), the glossed lemmas by source (the pivot by edition), and the rows that change when
  Wikidata's items and the Spanish section swap places (D2).
- *The floor proposed.* Each of the three figures less 2.0 points, to one decimal — never rounded up,
  never moved after the figures are read. A defect fixed afterwards in a class D3 or D4 names re-runs the
  study, and both measurements are reported; the floor proposed is the later one's, less two, and the
  owner settles it.
- *What may be fixed before the dispatch.* Only a class D3 or D4 names, in it-es's own files; each
  measured alone against the reduction without it, by rows changed among the 5,000 and the 20,000
  commonest; kept when every changed row of the 5,000 reads right or no worse and most of 5,001–20,000
  read right, narrowed (and recorded) otherwise; never taking the only gloss of a ranked lemma of the
  20,000 unless that gloss is pure noise. A reordering of senses, a wrong row no rule tells from a right
  one, or a matter of taste goes to change 28 (*Known data defects*).
- *The sample.* 100 glossed lemmas of the 20,000 commonest, drawn by it-fr's and it-en's seed
  (20261024), each marked by its source, a pivot row by its edition, entry and sense; read for the owner,
  not graded against a target.
- *Never read*: UD Italian-PUD and MarkIT's test section (change 9's gates).

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

## Implementation

What the implementation decided where the decisions above left it open, where it departs from them, and
what it measured (2026-10-25). Every figure is reduced by `reduce-it-es.py` itself, with Python 3.12 and
`requirements-reduce.txt`, over the bytes below; D8's criteria were committed (`174d4581c`) before any
of them was read.

### The bytes

`pack_sources.fetch_live` for it-es, run locally as `build.sh --update it-es` runs it (snapshot
2026.10.25), the Italian dump copied from change 22's cache once a `HEAD` request showed it served the same
(Last-Modified and Content-Length): what the dispatch's release, `lingua-pack-sources-it-es-<snapshot>`,
would hold if run that day. Every dump is CC BY-SA 4.0 + GFDL (Wiktionary) but Wikidata's, CC0 1.0.

| Record | Dump, Last-Modified | Served B | Decompressed B, sha256 | Derived file: B, sha256 (asset .zst B) |
|---|---|---:|---|---|
| `kaikki-es` | eswiktionary, Sat, 24 Oct 2026 12:07:21 GMT | 103,226,090 | 1,233,016,232, `923714a2…` | `kaikki-es-Italiano.jsonl` 5,362,930 `e9443572…` (362,819); `kaikki-es-traductions-it.jsonl` 966,789 `a4195bf0…` (113,402) |
| `kaikki-it` | itwiktionary, Sat, 24 Oct 2026 21:16:17 GMT | 43,849,715 | 566,646,259, `e007665a…` | `kaikki-it-traductions-es.jsonl` 1,096,876 `7536c275…` (133,468); `kaikki-it-Spagnolo.jsonl` 2,488,861 `1deac64f…` (169,039) |
| `kaikki-en` | enwiktionary, Wed, 21 Oct 2026 11:23:03 GMT | 3,082,192,147 | 26,611,939,151, `60a6073c…` | `kaikki-en-pivot-it-es.jsonl` 9,142,078 `4c4e1832…` (1,465,943) |
| `kaikki-fr` | frwiktionary, Fri, 23 Oct 2026 23:53:58 GMT | 736,596,796 | 6,865,138,291, `7ce93a41…` | `kaikki-fr-pivot-it-es.jsonl` 5,465,198 `9b993547…` (785,126) |
| `kaikki-de` | dewiktionary, Fri, 23 Oct 2026 21:11:17 GMT | 310,139,416 | 3,087,824,528, `e73ab6c1…` | `kaikki-de-pivot-it-es.jsonl` 7,274,397 `81237290…` (1,191,547) |
| `kaikki-pt` | ptwiktionary, Sat, 24 Oct 2026 20:34:28 GMT | 37,273,121 | 358,205,943, `48b16295…` | `kaikki-pt-pivot-it-es.jsonl` 1,779,911 `0962f01d…` (217,979) |
| `kaikki-ru` | ruwiktionary, Sat, 24 Oct 2026 13:47:45 GMT | 308,932,274 | 2,633,813,708, `30ee7430…` | `kaikki-ru-pivot-it-es.jsonl` 3,319,022 `5f3f3acc…` (480,691) |
| `kaikki-pl` | plwiktionary, Sat, 24 Oct 2026 17:18:14 GMT | 137,850,838 | 1,072,209,040, `21a106e4…` | `kaikki-pl-pivot-it-es.jsonl` 2,561,822 `48cb2376…` (291,398) |
| `kaikki-nl` | nlwiktionary, Sat, 24 Oct 2026 18:39:22 GMT | 134,749,715 | 1,275,188,244, `7c155acd…` | `kaikki-nl-pivot-it-es.jsonl` 1,246,166 `7d4d309a…` (206,577) |
| `wikidata` | `latest-lexemes.json.gz`, Wed, 21 Oct 2026 03:55:34 GMT | 655,273,999 | 7,591,401,100, `15d393d8…` | `wikidata-lexemes-it-es.jsonl` 1,046,290 `58505484…` (93,351) |

5.31 GB of dumps fetched (the English one in 62 s, its pass 156 s; Wikidata's in 147 s); twelve assets,
5,511,340 B compressed. The Spanish section is the research's file to the byte (5,362,930 B).

### The study (task 0.1)

On change 9's committed ranks (`tables/it/freq.tsv`), the coverage after each source in D2's order:

| | 5,000 | 10,000 | 20,000 |
|---|---:|---:|---:|
| the section's definitions, the Spanish edition's pre-passes and D4's | 31.7 | 22.6 | 15.1 |
| + the Italian Wiktionary's Spanish translations | 47.8 | 36.5 | 26.4 |
| + the Spanish tables read backwards | 59.2 | 47.2 | 35.2 |
| + Wikidata's translation links and Spanish glosses | 59.2 | 47.2 | 35.2 |
| + Wikidata's lexemes of one item | 61.3 | 49.4 | 37.1 |
| + the Italian Wiktionary's Spanish section read backwards | 66.9 | 53.9 | 40.8 |
| **+ the pivot (D8's study)** | **85.2** | **76.0** | **65.1** |
| **`FLOORS["it-es"]` proposed** (less two points) | **83.2** | **74.0** | **63.1** |

The research's designed figures were 84.7 / 76.1 / 65.5 on change 16's study ranking. Without the split
translations' filter (*Deviations*, D3), 85.3 / 76.1 / 65.1, a floor of 83.3 / 74.1 / 63.1. The
requirement states the proposed floor and `test_gloss_coverage.py` holds `FLOORS` to it; the owner
settles it. Glossed lemmas by source (every rank): definitions 3,973, the Italian Wiktionary's
translations 3,582, the Spanish tables read backwards 2,996, Wikidata's links and glosses 14, its items
738, the Spanish section 1,182, the pivot 12,219, and the `USA` row; 24,705 glosses, 1,165 expressions
(198 of the section, 963 of the two tables, 4 split words). The pivot by edition (lemmas whose shown
words a table of the edition gives): English 8,485, German 4,706, French 4,044, Russian 3,394, Polish
3,344, Portuguese 1,587, Dutch 646. Of the 7,605 glossed among the 10,000 commonest, 70.3 % from a
table-shaped source. **D2's order**: Wikidata's items before the Spanish section, or swapped, cover the
same; 22 / 36 / 47 rows differ, Wikidata's mostly the better (`corrente` « Corriente eléctrica » against
« Ala », `distretto` « Distrito » against « Zona », `poliziotto` « Policía » against « Cana, patrullero,
polizonte »), the Spanish section's in some (`decina` « Decena » against « Década »), both wrong in others (`domestico`
« Gato » against « Intestino »): D2's order kept.

### Each rule measured alone

Against the reduction without it (rows of `gloss.tsv` changed among the ranks 1–1,000 / 1,001–5,000 /
5,001–20,000 / beyond); no rule moves the coverage but the split translations' filter:

| Rule | Rows changed | Examples |
|---|---|---|
| names and acronyms, the thin-pair clause | 39 (6 / 27 / 6 / 1) | `secondo`, `volta`, `ragazzo`, `signore` lose « Apellido »; `segreto` « Apellido » → « Secreto, oculto », `gesù` → « Jesús » |
| pointers lending their word | 3 (0 / 0 / 3 / 5) | `lego` « Ligar, atar; Legar », `ke` « Qué; Que », `mastro` « Maestro » |
| a pointer's meaning after a colon | 2 (0 / 2 / 0 / 1) | `indicazioni` « Indicaciones », `pochissimo` « Sumamente poco » |
| notes (links, superscripts, subscripts, openings) | 57 (20 / 20 / 17 / 13) | `quale` « …; Como », `vero` « Vero, verdadero, cierto… », `vedere` « Ver », `livello` « Nivel, grado » |
| the function words' list | 7 (7 / 0 / 0 / 0) | `lo` « El; Lo », `loro` « Ellos, ellas; Ustedes », `ne`, `tu` « Tú », `lei` « Ella; Usted », `essere`, `avere` « Haber; Tener » |
| the Italian word given as Spanish | 24 (1 / 6 / 17 / 46) | `satira` « Sátira », `quarta` « Cuarto », `anteriore`, `triplo`; `corso` « Corso » → « Cambio » (see *Left for change 28*) |
| grave accents and « ° » | 11 (1 / 4 / 6 / 3) | `qualcuno` « Alguien », `abbonamento` « Suscripción », `menu` « Menú, carta, lista de platos » |
| a Spanish word listed once | 1,431 (157 / 463 / 811 / 612) | `fatto` « Hecho », `quanto` « Cuanto », `parte` « Parte », `mio` « Mi, mío » |
| split translations (the pivot) | 38 (9 / 10 / 19 / 25) | `degli` « De, estación, autobuses », `dei` « Porquero, porquerizo; Soviético », `delle` « De, las; Insinuar » lose them; `del` loses « Floristería, torrero, farero; Matutino… »; coverage 85.3 / 76.1 / 65.1 → 85.2 / 76.0 / 65.1 |
| the Spanish section's notes | 0 | no note becomes an Italian lemma's gloss either way; « vedi italiano » takes no key |

`aver`, `sono`, `sta`, `è`, `credo` and `it` — D4's examples on change 16's study ranking — are forms of
other lemmas in change 9's committed tables, not ranked lemmas: the rules hold them (the tests), the
coverage does not count them.

### Deviations

- **D3, a filter added: a translation split into its words.** The Polish and Russian extracts list each
  linked word of a translation as a translation of its own (`dworzec autobusowy` « stazione, degli,
  autobus » / « estación, de, autobuses »; `ковровый` « di » / « de »): 219 Polish groups, 13 Russian,
  about 35 of the five other editions. Read as the pivot reads pairs, they glossed the articulated
  prepositions at the top of the ranks — `del` (rank 9) « …; Floristería, torrero, farero; Matutino… »,
  `dei` (17) « Porquero, porquerizo; Soviético », `delle` (22), `degli` (47) « De, estación,
  autobuses ». A group of a content word's entry whose Italian or Spanish words hold an article, a
  preposition or a conjunction of a closed list (`split_translation`) glosses nothing. Not a class D3
  names, so not a fix D8's criteria allow before the dispatch: implemented because the spec's pivot pairs
  « an Italian and a Spanish word a person wrote » under one sense, which a fragment is not, and **left
  for the owner to keep or drop** — the coverage with it 85.2 / 76.0 / 65.1, without 85.3 / 76.1 / 65.1.
- **D3's order**: a lemma's Spanish words by the number of tables giving them, then of editions, then
  Spanish frequency, then the alphabet — D3's wording. Against the two Wiktionaries' tables, on the 8,766
  lemmas they and the pivot gloss, the first word agrees 78.0 % so, 77.7 % editions first, 77.8 % by
  tables alone, 45.9 % by frequency alone.
- **D3's groups**: a table under a sense (`by: under`) counts as named by it; the dumps write none.
- **D4, the table rules**: `reduce_spanish_tables.py` copies fr-es's letters, corrections, other senses,
  `read_backwards`, the studied word given as Spanish, typography and `listed_once`, each fr-es global a
  `Rules` field; test_reduce_it_es.py holds the copy, given fr-es's globals, to fr-es's own functions on
  fr-es's own cases. fr-es's `listed_once_direct` is not copied: D4 lists every table-shaped source once by
  `listed_once`, under the first part of speech Italian's readings name. it-es's `Rules`: the language
  code « it », no correction of a table and no other sense named.
- **D4, the function words** (`FUNCTION_WORDS`, 18 rows for `tu`, `è`, `sono`, `essere`, `avere`, `lo`,
  `le`, `gli`, `li`, `ne`, `lei`/`Lei`, `loro`/`Loro`): fr-es's `CORRECTIONS` shape, keyed by the section's
  text, but a row's senses replace the entry's whole — pointers' tags included, since `è` and `sono` are
  tagged forms of `essere` — or leave the entry out (`essere`'s and `avere`'s auxiliaries). Their words come
  from the sources where one has them (the Italian Wiktionary's « tú », « ella », « usted », « soy »,
  « son »; the Spanish tables' « el », « los »; the section's own « las », « de él, de ella… »); « Es »,
  « Lo » and « De allí, de ahí » are written by a person. **For the owner to review.**
- **D4, `ITALIAN_ALIKE`**: the design's four, `credo`, `friulano`, `nutrimento`, `visivo` (DLE). With it
  the rule refuses 88 Italian words of the 20,000 as their own Spanish gloss, each source alike — most
  rightly (`satira`, `anteriore`, `treno`); some the DLE may hold too (`tempo`, `finta`, `voluta`, `testa`,
  `fascia`, `cancro`, `triplo`, `sapiente`, `didascalia`, `bergamasco`, `peruviano`, `triestino`,
  `lubrificante`, `malcontento`, `tirocinio`), each losing one word of its row, none its gloss — **for the
  owner to list or not**.
- **D2, Wikidata**: a pair's part of speech is its Italian lexeme's lexical category; a proper noun's, a
  letter's or an affix's lexeme glosses nothing, as such an entry of a Wiktionary's table; a lemma's Spanish
  words ordered by the number of senses pairing them.
- **27a's D3 rule, the Spanish section read backwards**: a parenthesis is taken out before the split
  (« (di azienda) solido, sicuro » gives `solido`, `sicuro`), then the first three parts, each of at most two
  words; a lemma's Spanish words commonest first, as the Spanish tables read backwards.
- **D9**: `measures.json` adds the coverage after each source and the pivot by edition; the sample marks a
  pivot row « pivot (<edition>: <entry> (<part of speech>), « <sense> ») ». The README comes with the tables.
- **D11 and task 4**: `licence.rs`'s `Cc0` and the test that no package lists it-es are here; the baseline,
  its golden, `cross_native.rs`, the Spanish card snapshot, `committed_tables.rs`'s checks of the pack and
  its credits, and the two workflows' `--test it_es_baseline` and vitest lines need the committed tables:
  the update workflow cannot name a test target the branch lacks — its bless step would fail after the
  release step published the snapshot. They come with the tables, after the dispatch.
- **The release notes**: their header and licence footer name Wikidata's data and CC0 when the release
  holds a file derived from Wikidata (change 27a's review); « the date kaikki regenerated it » reads « the
  date it was regenerated ».

### The forecast (task 3.2)

`build.sh --update it-es`'s steps after `fetch-live`, over the bytes above, in a scratch root: `gloss.tsv`
24,705 rows (`0745b641…`), `senses.tsv` (`b1eed817…`), `mwe.tsv` 1,165 rows (`2cb47ffa…`), `NOTICE`
(`e9039b41…`); the pin records it-fr as the reference and the six studied tables as committed; the pack
1,894,353 B, its base it-fr's (`b1301e06…`, 1,551,196 B), its layer 343,397 B. The tables do not depend on
the day; `pack_version`, the manifest and the pack's sha256 do, and so does any edit of it-es's rule files.

### Left for change 28

Besides *Known data defects*, read in the forecast's 300 commonest rows and its sample:

- **Function words the pivot glosses from many forms at once**: `del` « Algo, un tanto, algún tanto; Para;
  Del », `ci` « Nos, nosotros, ahí; Aquí, por allí, para allá; Ce; No, hay », `ti` « Te, ti, la », `te`
  « La, lo, te », `vi`, `contro`; `della` keeps « Contracción de la preposición… » (the edition's gap).
- **The tables' wrong rows at the top**: `posto` « En », `caso` « Azar », `solo` « Único », `cui` « Quien »,
  `verso` « Sentido », `corso` « Cambio » once its « Corso » is refused.
- **Wikidata's items**: `aderenza` « Tractor », `don` « D. », `domestico` « Gato », `decina` « Década ».
- **A name's note alone**: 136 rows of the 20,000 (`maria` « Nombre de pila de mujer, equivalente del
  español María », `berlusconi` « Apellido »), as designed.
- **The pivot's last words**: `restringere` « Restringir, reducir, eliminar », `tuttora` « Aún, todavía,
  seguir », `soddisfatto` « …, qué bueno ».
